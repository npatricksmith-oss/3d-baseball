"""The three-dimensional body's fit to Fortenbaugh's mean swing under the design's rules
(docs/body-motion.md 4.3's rules, 4.5's limits, 13's gate; PLAN D31 phase 1, sessions 2 and 5; PLAN D34).

    python3 analysis/research/body/fit3d.py --starts 3        the whole body under body-motion 4.5's limits (job queue)
    python3 analysis/research/body/fit3d.py --relax           one rule relaxed at a time, from that fit
    python3 analysis/research/body/fit3d.py --ranges          4.5's range ends, one at a time (--only a,b for some)
    python3 analysis/research/body/fit3d.py --small           PLAN D32's small frame
    python3 analysis/research/body/fit3d.py --wrists          with a stop on the wrists' cock
    python3 analysis/research/body/fit3d.py --lower forces    the legs and trunk alone (also: pelvis, rate)
    python3 analysis/research/body/fit3d.py --show fit3d_d34.json

This FITS THE MODEL'S DRIVE NUMBERS TO A RECORDED AVERAGE SWING in the laboratory. Nothing in a game is
fitted; nothing here tells the body where the bat should end, except as a recorded curve to compare with.

The body starts at -250 ms on [4]'s measured state, moving (body3d.body.place). 22 drive channels: 16 of the
legs and trunk, each (strength, onset, rise, switch-off, starting level), and 6 of the arms, each (strength,
onset, rise, switch-off): 104 numbers. The rules are the search's box: strengths inside body-motion 4.5's
bounds, rises of at least 50 ms, the arms' channels off until -170 ms at the earliest, every onset inside
the window, the legs' and trunk's starting levels between 0 and 1. The search is Levenberg-Marquardt on the
residuals in band units with a finite-difference Jacobian, projected onto the box, run through the compiled
probe (probe3d.py). It is staged: (A) the lower body and trunk against [4]'s ground forces, pelvis, trunk
and lead knee, the arms undriven; (B) the arms against the bat and elbows, A's numbers held; (C) all the
numbers against everything. The reported numbers are from one run of the whole body with C's result.
Starts: start 0 is a hand estimate from [4]'s event times at half strength; the others perturb it with a
fixed seed. The gate's curves are [4]'s RATE curves (4.5).
"""
import json
import math
import os
import random
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import probe3d as P                                    # noqa: E402
from probe3d import E, B, Dr                           # noqa: E402

T0, T_END, H_STEP, EVERY = -0.250, 0.060, 1e-3, 5
NP = 5                       # numbers per channel: strength, onset, rise, switch-off, starting level
LOWER = ("hip_ext_R", "hip_ext_L", "hip_flex_L", "hip_rot_R", "hip_rot_L", "hip_abd_R", "hip_add_R", "hip_abd_L", "hip_add_L",
         "knee_ext_R", "knee_ext_L", "knee_flex_R", "knee_flex_L", "ankle_pf_R", "ankle_pf_L", "core")
ARMS = Dr.ARM_CHANNELS
CH = LOWER + ARMS
# the bands the misfits are scaled by. Rates and travel: fit.py's (read off [4]'s graphs, about one
# between-batter SD). The rest [E], chosen before the first 3D fit was run.
BAND = {"pelvis_rot_vel": 80.0, "trunk_rot_vel": 100.0, "bat_az_vel": 250.0, "pelvis_y": 0.03,
        "lead_grf_z": 15.0, "trail_grf_z": 15.0, "lead_grf_x": 8.0, "trail_grf_x": 8.0, "lead_grf_y": 8.0, "trail_grf_y": 8.0,
        "lead_knee_flex": 8.0, "lead_elbow_flex": 10.0, "trail_elbow_flex": 10.0, "bat_elev": 8.0, "bat_lag": 10.0}
STAGE_A = ("pelvis_rot_vel", "trunk_rot_vel", "pelvis_y", "lead_grf_z", "trail_grf_z", "lead_grf_x", "trail_grf_x",
           "lead_grf_y", "trail_grf_y", "lead_knee_flex")
LOWER_SETS = {"forces": None, "pelvis": ("pelvis_rot_vel", "pelvis_y", "lead_knee_flex"), "rate": ("pelvis_rot_vel",)}
# at contact: [4] Table 1's lag and bat elevation; the pelvis where [4]'s RATE curve takes it from -250 ms
# (-20.7 + 85.2 deg), since the gate reads the rate curves (body-motion 4.5)
AT_CONTACT = {"pelvis_rot": (64.5, 6.0), "bat_lag": (65.0, 10.0), "bat_elev": (-30.0, 5.0)}
CONTACT_AZ = -8.0            # [4]: the bat's azimuth at contact
CONTACT_BAND = 0.005         # s: the gate's +-5 ms
# start 0: onsets from [4]'s events, every strength at half its bound, rises 80 ms, the legs and trunk tensed at 0.2
START_ON = {"hip_ext_R": -0.245, "hip_ext_L": -0.200, "hip_flex_L": -0.245, "hip_rot_R": -0.245, "hip_rot_L": -0.245,
            "hip_abd_R": -0.245, "hip_add_R": -0.245, "hip_abd_L": -0.200, "hip_add_L": -0.200,
            "knee_ext_R": -0.245, "knee_ext_L": -0.200, "knee_flex_R": -0.200, "knee_flex_L": -0.100,
            "ankle_pf_R": -0.245, "ankle_pf_L": -0.200, "core": -0.200,
            "shoulder_L": -0.170, "shoulder_R": -0.170, "elbow_ext_L": -0.170, "elbow_ext_R": -0.120,
            "wrist_L": -0.090, "wrist_R": -0.090}
START_T, START_RISE, START_OFF, START_A0 = 0.5, 0.080, 0.050, 0.2

_c1 = lambda n: Dr.C1[n][0] / (Dr.C1[n][0] + Dr.SD_COUNT * Dr.C1[n][1])
_pre = lambda *ps: {n: None for n in CH if n.startswith(ps)}
def _sc(k, *ps):
    return {n: k for n in CH if n.startswith(ps)}
# one rule relaxed at a time (a diagnosis; not the design)
RELAX = {
    "continued": {},
    "arms_free": {"arms_earliest": T0},
    "shoulders_elbows_x2": {"scale": _sc(2.0, "shoulder", "elbow")},
    "no_rise_floor": {"rise_min": 0.005},
    "legs_trunk_x1.5": {"scale": {n: 1.5 for n in LOWER}},
    "wrists_x3": {"scale": _sc(3.0, "wrist")},
}
# the range ends of body-motion 4.5: what the answer rests on
RANGES = {
    "hip_extension_524": {"scale": _sc(524.0 / 430.1, "hip_ext")},
    "legs_at_the_sources_means": {"scale": {**_sc(_c1("hip_ext"), "hip_ext"), **_sc(_c1("hip_flex"), "hip_flex"), **_sc(_c1("knee_ext"), "knee_ext"),
                                            **_sc(_c1("knee_flex"), "knee_flex"), **_sc(_c1("ankle_pf"), "ankle_pf")}},
    "trunk_290": {"scale": {"core": 290.0 / 260.0}},
    "trunk_250": {"scale": {"core": 250.0 / 260.0}},
    "adduction_185": {"scale": _sc(1.4, "hip_add")},
    "adduction_92": {"scale": _sc(0.7, "hip_add")},
    "rotators_110": {"scale": _sc(110.0 / 78.0, "hip_rot")},
    "rotators_50": {"scale": _sc(50.0 / 78.0, "hip_rot")},
    "hip_extension_curve_andersons": {"hip_curve": "hip_ext_anderson"},
    "hip_side_muscles_on_the_first_law": {"side_curve": None},
    "no_tendon_springs": {"tendons": False},
    "nothing_tensed_beforehand": {"pretension": False},
}
RULES = {"design": {}, **RELAX, **RANGES}


class Ctx:
    """one placed, toned, damped body with its channels, reused for every evaluation."""

    def __init__(self, H=1.84, M=92.0, rules="design", cock_stop=None, t0=T0, shoulder_stops=True):
        self.rules = dict(RULES[rules]) if isinstance(rules, str) else dict(rules)
        r = self.rules
        self.t0 = t0
        self.m = m = B.Model(H, M, shoulder_stops=shoulder_stops)
        st = B.place(m, t0)
        self.aux0 = list(m.start_aux)
        self.bd = Dr.bounds(H, M)
        for n, k in r.get("scale", {}).items():
            self.bd[n] *= k
        keep = (Dr.HIP_SIDE_CURVE, Dr.CURVES["hip_ext"], Dr.TENDONS)
        if "side_curve" in r:
            Dr.HIP_SIDE_CURVE = r["side_curve"]
        if "hip_curve" in r:
            Dr.CURVES["hip_ext"] = Dr.CURVES[r["hip_curve"]]
        if "tendons" in r:
            Dr.TENDONS = r["tendons"]
        st = Dr.add_drives(m, st, {n: (0.0, 0.0, 0.05, 0.1, 0.0) for n in CH})
        Dr.HIP_SIDE_CURVE, Dr.CURVES["hip_ext"], Dr.TENDONS = keep
        self.tone, self.tone_rest = Dr.solve_tone(m, st, t0, aux=list(self.aux0))
        Dr.add_damping(m, st)
        if cock_stop is not None:
            add_cock_stop(m, st, cock_stop)
        self.st = st
        self.BW = M * E.G
        rise_min = r.get("rise_min", Dr.RISE_MIN)
        arms = r.get("arms_earliest", Dr.ARMS_EARLIEST)
        pre = 1.0 if r.get("pretension", True) else 0.0
        self.box = []                                   # (channel, field, lo, hi)
        for n in CH:
            self.box += [(n, "T", 0.0, self.bd[n]), (n, "on", arms if n in ARMS else t0, 0.0),
                         (n, "rise", rise_min, 0.250), (n, "off", -0.150, 0.100), (n, "start", 0.0, 0.0 if n in ARMS else pre)]
        self.free = [i for i, (n, f, lo, hi) in enumerate(self.box) if hi > lo]
        self.times = [t0 + i * H_STEP * EVERY for i in range(int(round((0.0 - t0) / (H_STEP * EVERY))) + 1)]
        self.target = {k: [B.curve(k, t) for t in self.times] for k in BAND if k != "pelvis_y"}
        y_lead = 0.5 * B.STRIDE * H
        self.target["pelvis_y"] = [y_lead + B.curve("pelvis_x_cm", t) / 100.0 * H / 1.84 for t in self.times]
        self.evals = 0

    def plan(self, u):
        p = {}
        for i, n in enumerate(CH):
            p[n] = tuple(lo + min(1.0, max(0.0, u[NP * i + k])) * (hi - lo) for k, (_, _, lo, hi) in enumerate(self.box[NP * i:NP * i + NP]))
        return p

    def u_of(self, plan):
        """the search's numbers for a plan {channel: (T, onset, rise, off[, start])}; a channel the plan lacks is off."""
        u = []
        for i, n in enumerate(CH):
            v = list(plan.get(n, (0.0, 0.0, 0.05, 0.1, 0.0)))
            v += [0.0] * (NP - len(v))
            for k, (_, _, lo, hi) in enumerate(self.box[NP * i:NP * i + NP]):
                u.append(min(1.0, max(0.0, (v[k] - lo) / (hi - lo))) if hi > lo else 0.0)
        return u

    def start(self, seed=0):
        rng = random.Random(seed)
        plan = {n: (START_T * self.bd[n], START_ON[n], START_RISE, START_OFF, 0.0 if n in ARMS else START_A0) for n in CH}
        u = self.u_of(plan)
        if seed:
            u = [min(1.0, max(0.0, x + rng.uniform(-0.2, 0.2))) if i in self.free else x for i, x in enumerate(u)]
        return u

    def prepare(self, u):
        """the model's drives set to u; returns the starting state (each channel at its starting level) and the
        per-step state (friction anchors, tendon springs loaded for muscles in steady pull)."""
        s = self.m.sys
        st = list(self.st)
        oa = 13 * s.nb
        for j, (a, (n, v)) in enumerate(zip(s.acts, self.plan(u).items())):
            a[4], a[6], a[7], a[8], a[9] = v
            st[oa + j] = v[4]
        aux = list(self.aux0)
        s.sec_start(st, aux)
        return st, aux

    def run(self, u, t_end=T_END, h=H_STEP):
        st, aux = self.prepare(u)
        self.evals += 1
        every = int(round(EVERY * H_STEP / h))
        return P.simulate(self.m, st, self.t0, t_end, h=h, every=every, aux=aux)[0]

    def measures(self, rows):
        """{quantity: model values at self.times}, the contact time and the state of things at contact."""
        BW = self.BW
        val = {k: [] for k in BAND}
        for (t, o, g) in rows[:len(self.times)]:
            for k in BAND:
                if k.endswith("grf_z"):
                    val[k].append(100.0 * g["L" if k[0] == "l" else "R"][2] / BW)
                elif k.endswith("grf_x"):                 # [4]'s X is toward the pitcher: world Y
                    val[k].append(100.0 * g["L" if k[0] == "l" else "R"][1] / BW)
                elif k.endswith("grf_y"):                 # [4]'s Y is toward home plate: world X
                    val[k].append(100.0 * g["L" if k[0] == "l" else "R"][0] / BW)
                else:
                    val[k].append(o[k])
        tc, oc = None, None
        for (ta, oa, _), (tb, ob, _) in zip(rows, rows[1:]):
            if oa["bat_az"] < CONTACT_AZ <= ob["bat_az"]:
                f = (CONTACT_AZ - oa["bat_az"]) / (ob["bat_az"] - oa["bat_az"])
                tc = ta + f * (tb - ta)
                oc = {k: oa[k] + f * (ob[k] - oa[k]) for k in oa if isinstance(oa[k], float)}
                break
        if tc is None:                                    # never came round: time it would need at [4]'s bat rate
            t_last, o_last, _ = rows[-1]
            tc = t_last + max(0.0, CONTACT_AZ - o_last["bat_az"]) / 2400.0
            oc = {k: v for k, v in o_last.items() if isinstance(v, float)}
        return val, tc, oc

    def residuals(self, u, which=None):
        try:
            rows = self.run(u)
        except E.SolveError:
            return None
        val, tc, oc = self.measures(rows)
        r = []
        n = len(self.times)
        for k in (which or BAND):
            w = 1.0 / (BAND[k] * math.sqrt(n))
            r += [(a - b) * w for a, b in zip(val[k], self.target[k])]
        if which is None:
            r.append(tc / CONTACT_BAND)
            for k, (mean, sd) in AT_CONTACT.items():
                r.append((oc[k] - mean) / sd)
        if any(x != x for x in r):
            return None
        return r


def add_cock_stop(m, st, margin_deg):
    """a one-sided stop on each wrist's cock: the angle from the forearm's axis to the bat's axis about the
    bat's own y may not pass its value at the start by more than margin_deg [E: an anatomical range of
    radial deviation is about 20-25 deg; the starting posture's own angle is not known to be neutral]."""
    s = m.sys
    for sd in "LR":
        fa, bat = m.b["farm_" + sd], m.b["bat"]
        Rb = E.qR(s.pose(st, bat)[1])
        zf_in_bat = E.mtv(Rb, m.axis(st, "farm_" + sd, (0, 0, 1)))
        e = E.unit((zf_in_bat[0], 0.0, zf_in_bat[2]))      # the forearm's axis projected square to the bat's y
        # angle about the bat's y from e (fixed in the bat) to the forearm's axis: 0 at the start. The drive
        # that uncocks turns it one way (sign found as add_drives does); the stop blocks the other way.
        act = next(a for a in s.acts if a[0] == "wrist_" + sd)
        sign = 1.0 if act[3][1] > 0 else -1.0             # the uncocking torque on the forearm is about sign * y
        lim = margin_deg * B.D2R
        lo, hi = (-lim, 10.0) if sign > 0 else (-10.0, lim)
        s.stop("cock_" + sd, bat, fa, (0, 1, 0), e, (0, 0, 1), lo, hi, B.STOP_K, B.STOP_C)


def lm(ctx, u, free, which=None, iters=25, log=None, fd=0.02):
    """Levenberg-Marquardt on the residuals over the free parameters (indices), projected onto [0, 1]."""
    r = ctx.residuals(u, which)
    cost = sum(x * x for x in r)
    lam = 1e-2
    for it in range(iters):
        J = []
        for i in free:
            up = list(u)
            step = fd if u[i] + fd <= 1.0 else -fd
            up[i] = u[i] + step
            rp = ctx.residuals(up, which)
            J.append([0.0] * len(r) if rp is None else [(a - b) / step for a, b in zip(rp, r)])
        n = len(free)
        JtJ = [[sum(a * b for a, b in zip(J[i], J[k])) for k in range(i + 1)] for i in range(n)]
        g = [sum(a * b for a, b in zip(J[i], r)) for i in range(n)]
        improved = False
        for _ in range(8):
            A = [[JtJ[i][k] + (lam * (JtJ[i][i] + 1e-6) if i == k else 0.0) for k in range(i + 1)] for i in range(n)]
            try:
                d = E.chol_solve(A, [-x for x in g])
            except E.SolveError:
                lam *= 10.0
                continue
            un = list(u)
            for i, di in zip(free, d):
                un[i] = min(1.0, max(0.0, u[i] + max(-0.25, min(0.25, di))))
            rn = ctx.residuals(un, which)
            cn = None if rn is None else sum(x * x for x in rn)
            if cn is not None and cn < cost:
                gain = cost - cn
                u, r, cost = un, rn, cn
                lam = max(1e-6, lam / 3.0)
                improved = True
                break
            lam *= 4.0
        if log:
            log("      iteration %2d: cost %.4f, lambda %.1e, %d runs" % (it + 1, cost, lam, ctx.evals))
        if not improved or gain < 1e-5 * cost:
            break
    return u, cost


def summary(ctx, u, h=H_STEP):
    """the fitted swing against [4] and the gate of body-motion 13: a dict of numbers, from one run."""
    rows = ctx.run(u, h=h)
    val, tc, oc = ctx.measures(rows)
    n = len(ctx.times)
    out = {"contact_ms": tc * 1e3, "at_contact": {k: oc[k] for k in ("pelvis_rot", "trunk_rot", "bat_lag", "bat_elev", "sweet_speed",
                                                                     "lead_knee_flex", "lead_elbow_flex", "trail_elbow_flex", "bat_az_vel")}}
    out["rms_band_units"] = {k: math.sqrt(sum(((a - b) / BAND[k]) ** 2 for a, b in zip(val[k], ctx.target[k])) / n) for k in BAND}
    out["peaks"] = {}
    for k in ("pelvis_rot_vel", "trunk_rot_vel", "bat_az_vel"):
        v, t = max((o[k], t) for t, o, g in rows)
        out["peaks"][k] = (v, t * 1e3)
    out["grf_peak_lead_z"] = max((100 * g["L"][2] / ctx.BW, t * 1e3) for t, o, g in rows)
    out["came_round"] = any(o["bat_az"] >= CONTACT_AZ for t, o, g in rows)
    at = lambda tt, k: next(o[k] for t, o, g in rows if abs(t - tt) < 1e-9)
    out["bat_az_at_0"] = at(0.0, "bat_az")
    out["sweet_speed_at_0"] = at(0.0, "sweet_speed")
    out["sweet_speed_peak"] = max((o["sweet_speed"], t * 1e3) for t, o, g in rows)
    out["pelvis_rate_at_-93"] = 0.4 * at(-0.095, "pelvis_rot_vel") + 0.6 * at(-0.090, "pelvis_rot_vel")
    names = [a[0] for a in ctx.m.sys.acts]
    out["applied_peak"] = {nm: max(abs(o["act"][nm][0]) for t, o, g in rows) for nm in names}
    out["applied_at_speed"] = {nm: max((abs(o["act"][nm][0]), o["act"][nm][1]) for t, o, g in rows) for nm in names}
    out["stop_peak"] = {nm: max(abs(o["stops"][nm][0]) for t, o, g in rows) for nm in rows[0][1]["stops"]}
    nc = len(ctx.m.sys.contacts)
    out["tendon_peak_J"] = {a[0]: max(0.5 * a[11] * o["sec"][j] ** 2 for t, o, g in rows) for j, a in enumerate(ctx.m.sys.acts) if a[11] > 0}
    p = ctx.plan(u)
    out["plan"] = {nm: list(v) for nm, v in p.items()}
    out["bounds"] = dict(ctx.bd)
    on = []
    for i, (nm, f, lo, hi) in enumerate(ctx.box):
        if hi > lo and (u[i] <= 1e-6 or u[i] >= 1.0 - 1e-6):
            on.append("%s %s %s" % (nm, f, "low" if u[i] <= 1e-6 else "HIGH"))
    out["on_bounds"] = on
    out["n_free"] = len(ctx.free)
    out["rules_broken"] = Dr.check_rules(ctx.m, p) if not ctx.rules else "changed for this run: %s" % ", ".join(ctx.rules)
    return out, rows


def show(out, log=print):
    log("   contact (bat azimuth -8 deg) at %+.1f ms%s; bat azimuth at 0 ms %.0f deg; sweet spot %.1f m/s at 0 ms, %.1f at contact, %.1f at its fastest (%.0f ms)"
        % (out["contact_ms"], "" if out["came_round"] else " (EXTRAPOLATED: the bat did not come round by +60 ms)",
           out["bat_az_at_0"], out["sweet_speed_at_0"], out["at_contact"]["sweet_speed"], out["sweet_speed_peak"][0], out["sweet_speed_peak"][1]))
    c = out["at_contact"]
    log("   at contact: pelvis %.0f deg (65 +- 6 on the rate curve; 77 on the angle curve), trunk %.0f (62; 75), lag %.0f (65 +- 10), bat elevation %.0f (-30 +- 5), lead knee %.0f (18), elbows %.0f, %.0f (56, 81)"
        % (c["pelvis_rot"], c["trunk_rot"], c["bat_lag"], c["bat_elev"], c["lead_knee_flex"], c["lead_elbow_flex"], c["trail_elbow_flex"]))
    pk = out["peaks"]
    log("   peaks: pelvis %.0f deg/s at %.0f ms (581 at -70), %.0f at -93 ms (558); trunk %.0f at %.0f (766 at -57); bat azimuth %.0f at %.0f (2,435 at -10); lead foot %.0f %% BW at %.0f ms (130 at -93)"
        % (pk["pelvis_rot_vel"][0], pk["pelvis_rot_vel"][1], out["pelvis_rate_at_-93"], pk["trunk_rot_vel"][0], pk["trunk_rot_vel"][1],
           pk["bat_az_vel"][0], pk["bat_az_vel"][1], out["grf_peak_lead_z"][0], out["grf_peak_lead_z"][1]))
    log("   residual RMS in band units, -250 ms to 0: " + ", ".join("%s %.2f" % (k, v) for k, v in out["rms_band_units"].items()))
    log("   plan: channel  strength of bound [applied peak, at rad/s]  onset rise off ms  starting level")
    for nm, v in out["plan"].items():
        ap = out["applied_at_speed"][nm]
        log("      %-12s %4.0f of %4.0f [%4.0f at %+5.1f]  %5.0f %4.0f %5.0f  %.2f" % (nm, v[0], out["bounds"][nm], ap[0], ap[1], v[1] * 1e3, v[2] * 1e3, v[3] * 1e3, v[4]))
    log("   stops' largest torques (N m): " + ", ".join("%s %.0f" % (k, v) for k, v in out["stop_peak"].items()))
    log("   tendon springs' largest stored energy (J): " + (", ".join("%s %.0f" % (k, v) for k, v in out["tendon_peak_J"].items()) or "none"))
    log("   on a bound (%d of %d numbers): " % (len(out["on_bounds"]), out["n_free"]) + ("; ".join(out["on_bounds"]) or "none"))
    log("   rules: %s" % (out["rules_broken"] or "none broken"))


def fit(rules="design", starts=3, H=1.84, M=92.0, log=print, iters=(20, 20, 30), u_from=None, cock_stop=None):
    ctx = Ctx(H, M, rules, cock_stop)
    iA = [i for i in ctx.free if ctx.box[i][0] in LOWER]
    iB = [i for i in ctx.free if ctx.box[i][0] in ARMS]
    best = None
    for sidx in range(starts):
        t = time.time()
        if u_from is not None:
            u = ctx.u_of(u_from)
            log("  from a given plan, all numbers free")
            u, c = lm(ctx, u, iA + iB, None, iters[2], log)
        else:
            u = ctx.start(sidx)
            ub = list(u)
            for i, n in enumerate(CH):                    # stage A: the arms undriven
                if n in ARMS:
                    u[NP * i] = 0.0
            log("  start %d, stage A (lower body and trunk; arms undriven)" % sidx)
            u, c = lm(ctx, u, iA, STAGE_A, iters[0], log)
            for i in iB:
                u[i] = ub[i]
            log("  start %d, stage B (arms; A's numbers held)" % sidx)
            u, c = lm(ctx, u, iB, None, iters[1], log)
            log("  start %d, stage C (all %d numbers)" % (sidx, len(iA + iB)))
            u, c = lm(ctx, u, iA + iB, None, iters[2], log)
        out, rows = summary(ctx, u)
        out["cost"], out["start"], out["runs"], out["seconds"] = c, sidx, ctx.evals, time.time() - t
        log("  start %d done: cost %.3f, %d runs so far, %.0f s" % (sidx, c, ctx.evals, time.time() - t))
        show(out, log)
        if best is None or c < best["cost"]:
            best = out
    return ctx, best


def fit_lower(weighting, starts=3, iters=40, log=print):
    """the lower body alone (the leg and trunk channels; the arms undriven) against one of LOWER_SETS: forces
    (pelvis and trunk rates, travel, both feet's forces, the lead knee), pelvis (rate, travel, lead knee), rate.
    Session 3's numbers in body-motion 4.4 came from this mode at commit 876b3aa, before 4.5's limits."""
    ctx = Ctx()
    which = LOWER_SETS[weighting] or STAGE_A
    iA = [i for i in ctx.free if ctx.box[i][0] in LOWER]
    best = None
    for sidx in range(starts):
        u = ctx.start(sidx)
        for i, n in enumerate(CH):
            if n in ARMS:
                u[NP * i] = 0.0
        log("  start %d (lower body alone, weighting '%s')" % (sidx, weighting))
        u, c = lm(ctx, u, iA, which, iters, log)
        out, rows = summary(ctx, u)
        out["cost"], out["start"] = c, sidx
        show(out, log)
        if best is None or c < best["cost"]:
            best = out
    return ctx, best


def main():
    P.build()
    a = sys.argv[1:]
    arg = lambda k, d: a[a.index(k) + 1] if k in a else d
    base_file = os.path.join(HERE, "fit3d_d34.json")
    it = int(arg("--iters", 25))
    if "--show" in a:
        show(json.load(open(os.path.join(HERE, arg("--show", "fit3d_d34.json")))))
        return
    if "--lower" in a:
        w = arg("--lower", "forces")
        ctx, best = fit_lower(w, int(arg("--starts", 3)), int(arg("--iters", 40)))
        json.dump(best, open(os.path.join(P.OUT, "fit3d_d34_lower_%s.json" % w), "w"), indent=1)
        return
    if "--relax" in a or "--ranges" in a:
        base = json.load(open(base_file))
        sets = RELAX if "--relax" in a else RANGES
        only = arg("--only", None)
        for name in sets:
            if only and name not in only.split(","):
                continue
            print("\n== %s (re-fitted from the design fit, all numbers free)" % name)
            ctx, best = fit(name, 1, u_from=base["plan"], iters=(0, 0, it))
            json.dump(best, open(os.path.join(P.OUT, "fit3d_d34_%s.json" % name), "w"), indent=1)
        return
    if "--small" in a:
        base = json.load(open(base_file))
        Hs, Ms = float(arg("--height", 1.75)), float(arg("--mass", 77.0))
        print("the small frame (PLAN D32): %.2f m, %.0f kg [E]; his bounds are the same sourced bounds at his weight x height;" % (Hs, Ms))
        print("re-fitted under the design's rules from the average batter's plan (a strength over his bound starts at his bound)")
        ctx, best = fit("design", 1, Hs, Ms, u_from=base["plan"], iters=(0, 0, it))
        print("\nthe average batter (1.84 m, 92 kg), for comparison:")
        show(base)
        json.dump(best, open(os.path.join(P.OUT, "fit3d_d34_small.json"), "w"), indent=1)
        return
    if "--wrists" in a:
        base = json.load(open(base_file))
        for margin in (5.0, 20.0):
            print("\nWITH a one-sided stop %.0f deg past the wrists' cock at the start [E], re-fitted from the design fit:" % margin)
            ctx, best = fit("design", 1, u_from=base["plan"], iters=(0, 0, it), cock_stop=margin)
            json.dump(best, open(os.path.join(P.OUT, "fit3d_d34_cock%d.json" % margin), "w"), indent=1)
        return
    starts = int(arg("--starts", 3))
    print("body-motion 4.5's limits, %d start(s), body 1.84 m 92 kg, from %.0f ms, step %.1f ms" % (starts, T0 * 1e3, H_STEP * 1e3))
    ctx, best = fit("design", starts, iters=(it, it, int(1.5 * it)))
    print("\nbest start: %d, cost %.3f" % (best["start"], best["cost"]))
    show(best)
    json.dump(best, open(base_file, "w"), indent=1)
    out, rows = summary(ctx, ctx.u_of(best["plan"]), h=5e-4)
    print("\nthe same plan at a 0.5 ms step:")
    show(out)


if __name__ == "__main__":
    main()
