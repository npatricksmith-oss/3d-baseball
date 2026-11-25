"""The three-dimensional body's fit to Fortenbaugh's mean swing under the design's rules
(docs/body-motion.md 4.3's rules, 4.4, 13's gate; PLAN D31 phase 1, session 2).

    python3 analysis/research/body/fit3d.py --rules design --starts 3       (through bin/job.sh: minutes)
    python3 analysis/research/body/fit3d.py --limits                         one rule relaxed at a time, from the design fit
    python3 analysis/research/body/fit3d.py --show fit3d_design.json         the fitted swing against [4] and the gate
    python3 analysis/research/body/fit3d.py --small                          PLAN D32's small frame
    python3 analysis/research/body/fit3d.py --wrists                         with and without a stop on the wrists' cock

This FITS THE MODEL'S DRIVE NUMBERS TO A RECORDED AVERAGE SWING in the laboratory. Nothing in a game is
fitted; nothing here tells the body where the bat should end, except as a recorded curve to compare with.

The body starts at -250 ms on [4]'s measured state, moving (body3d.body.place). Sixteen drive channels,
each (strength T, onset, rise time, switch-off): 64 numbers. The rules are the search's box:
strengths inside the sourced bounds, rises of at least 50 ms, the arms' channels no earlier than
-170 ms, every onset inside the window. The search is Levenberg-Marquardt on the residuals in band
units with a finite-difference Jacobian, projected onto the box, run through the compiled probe
(probe3d.py). It is staged: (A) the lower body and trunk against [4]'s ground forces, pelvis, trunk and
lead knee, the arms undriven; (B) the arms against the bat and elbows, A's numbers held; (C) all 64
numbers against everything. The reported numbers are from one run of the whole body with C's result.
Starts: start 0 is START (a hand estimate from [4]'s event times, half strength); the others perturb it
with a fixed seed.
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
LOWER = ("hip_ext_R", "hip_flex_L", "hip_ext_L", "hip_rot_R", "hip_rot_L", "hip_abd_R", "hip_add_R", "hip_abd_L", "hip_add_L",
         "knee_ext_R", "knee_ext_L", "ankle_pf_R", "ankle_pf_L", "core")
# the lower-body fits' weightings (session 3), each stated before its run:
#   forces: session 2's stage A (pelvis and trunk rates, travel, both feet's three forces, the lead knee)
#   pelvis: the pelvis's rate and travel and the lead knee only: no ground forces
#   rate:   the pelvis's rate alone: what the legs can do for the turn if nothing else is asked of them
LOWER_SETS = {"forces": None, "pelvis": ("pelvis_rot_vel", "pelvis_y", "lead_knee_flex"), "rate": ("pelvis_rot_vel",)}
ARMS = Dr.ARM_CHANNELS
CH = LOWER + ARMS
# the bands the misfits are scaled by. Rates and travel: fit.py's (read off [4]'s graphs, about one
# between-batter SD). The rest [E], chosen before the fit was run: [4]'s Table 1 SDs at contact where it has them.
BAND = {"pelvis_rot_vel": 80.0, "trunk_rot_vel": 100.0, "bat_az_vel": 250.0, "pelvis_y": 0.03,
        "lead_grf_z": 15.0, "trail_grf_z": 15.0, "lead_grf_x": 8.0, "trail_grf_x": 8.0, "lead_grf_y": 8.0, "trail_grf_y": 8.0,
        "lead_knee_flex": 8.0, "lead_elbow_flex": 10.0, "trail_elbow_flex": 10.0, "bat_elev": 8.0, "bat_lag": 10.0}
STAGE_A = ("pelvis_rot_vel", "trunk_rot_vel", "pelvis_y", "lead_grf_z", "trail_grf_z", "lead_grf_x", "trail_grf_x",
           "lead_grf_y", "trail_grf_y", "lead_knee_flex")
AT_CONTACT = {"pelvis_rot": (77.0, 6.0), "bat_lag": (65.0, 10.0), "bat_elev": (-30.0, 5.0)}   # [4] Table 1, mean and SD
CONTACT_AZ = -8.0            # [4]: the bat's azimuth at contact
CONTACT_BAND = 0.005         # s: the gate's +-5 ms
# start 0: onsets from [4]'s events (pelvis rate rises from -245 ms; the lead foot loads from -200; first
# move -170; the bat's second acceleration from about -90), every strength at half its bound, rises 80 ms
START_ON = {"hip_abd_R": -0.245, "hip_add_R": -0.245, "hip_abd_L": -0.200, "hip_add_L": -0.200, "hip_ext_R": -0.245, "hip_flex_L": -0.245, "hip_ext_L": -0.200, "hip_rot_R": -0.245, "hip_rot_L": -0.245,
            "knee_ext_R": -0.245, "knee_ext_L": -0.200, "ankle_pf_R": -0.245, "ankle_pf_L": -0.200, "core": -0.200,
            "shoulder_L": -0.170, "shoulder_R": -0.170, "elbow_ext_L": -0.170, "elbow_ext_R": -0.120,
            "wrist_L": -0.090, "wrist_R": -0.090}
START_T, START_RISE, START_OFF = 0.5, 0.080, 0.050

RULES = {
    "design": {},
    "continued": {},                                       # no rule relaxed: the same extra iterations, for a fair comparison
    "arms_free": {"arms_earliest": T0},                    # the arms may start any time in the window
    "shoulders_x2": {"scale": {"shoulder_L": 2.0, "shoulder_R": 2.0, "elbow_ext_L": 2.0, "elbow_ext_R": 2.0}},
    "no_rise_floor": {"rise_min": 0.005},
    "legs_trunk_x1.5": {"scale": {n: 1.5 for n in LOWER}},
    "wrists_x3": {"scale": {"wrist_L": 3.0, "wrist_R": 3.0}},
}


class Ctx:
    """one placed, toned, damped body with its sixteen channels, reused for every evaluation."""

    def __init__(self, H=1.84, M=92.0, rules="design", cock_stop=None, t0=T0, shoulder_stops=True):
        self.rules = dict(RULES[rules]) if isinstance(rules, str) else dict(rules)
        self.t0 = t0
        self.m = m = B.Model(H, M, shoulder_stops=shoulder_stops)
        st = B.place(m, t0)
        self.aux0 = list(m.start_aux)
        self.bd = Dr.bounds(H, M)
        for n, k in self.rules.get("scale", {}).items():
            self.bd[n] *= k
        st = Dr.add_drives(m, st, {n: (0.0, 0.0, 0.05, 0.1) for n in CH})
        self.tone, self.tone_rest = Dr.solve_tone(m, st, t0, aux=list(self.aux0))
        Dr.add_damping(m, st)
        if cock_stop is not None:
            add_cock_stop(m, st, cock_stop)
        self.st = st
        self.BW = M * E.G
        rise_min = self.rules.get("rise_min", Dr.RISE_MIN)
        arms = self.rules.get("arms_earliest", Dr.ARMS_EARLIEST)
        self.box = []                                   # (channel, field, lo, hi)
        for n in CH:
            self.box += [(n, "T", 0.0, self.bd[n]), (n, "on", arms if n in ARMS else t0, 0.0),
                         (n, "rise", rise_min, 0.250), (n, "off", -0.150, 0.100)]
        self.times = [t0 + i * H_STEP * EVERY for i in range(int(round((0.0 - t0) / (H_STEP * EVERY))) + 1)]
        self.target = {k: [B.curve(k, t) for t in self.times] for k in BAND if k != "pelvis_y"}
        y_lead = 0.5 * B.STRIDE * H
        self.target["pelvis_y"] = [y_lead + B.curve("pelvis_x_cm", t) / 100.0 * H / 1.84 for t in self.times]
        self.evals = 0

    def plan(self, u):
        p = {}
        for i, n in enumerate(CH):
            v = [lo + min(1.0, max(0.0, u[4 * i + k])) * (hi - lo) for k, (_, _, lo, hi) in enumerate(self.box[4 * i:4 * i + 4])]
            p[n] = tuple(v)
        return p

    def start(self, seed=0):
        u = []
        rng = random.Random(seed)
        for i, n in enumerate(CH):
            vals = (START_T * self.bd[n], max(START_ON[n], self.box[4 * i + 1][2]), START_RISE, START_OFF)
            for k, (_, _, lo, hi) in enumerate(self.box[4 * i:4 * i + 4]):
                x = (vals[k] - lo) / (hi - lo)
                if seed:
                    x += rng.uniform(-0.2, 0.2)
                u.append(min(1.0, max(0.0, x)))
        return u

    def run(self, u, t_end=T_END, h=H_STEP):
        s = self.m.sys
        for a, (n, v) in zip(s.acts, self.plan(u).items()):
            a[4], a[6], a[7], a[8] = v
        self.evals += 1
        every = int(round(EVERY * H_STEP / h))
        return P.simulate(self.m, self.st, self.t0, t_end, h=h, every=every, aux=list(self.aux0))[0]

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
    out["pelvis_rate_at_-93"] = 0.4 * at(-0.095, "pelvis_rot_vel") + 0.6 * at(-0.090, "pelvis_rot_vel")
    out["pelvis_rate_at_-70"] = at(-0.070, "pelvis_rot_vel")
    dt_rec = rows[1][0] - rows[0][0]
    out["ground_mz_impulse_-170_0"] = sum(o["ground_mz"] for t, o, g in rows if -0.170 <= t < 0.0) * dt_rec
    out["ground_mz_peak"] = max((o["ground_mz"], t * 1e3) for t, o, g in rows if t <= 0.0)
    out["pelvis_turn_at_0"] = at(0.0, "pelvis_rot")
    out["bat_az_at_0"] = next(o["bat_az"] for t, o, g in rows if abs(t) < 1e-9)
    out["sweet_speed_at_0"] = next(o["sweet_speed"] for t, o, g in rows if abs(t) < 1e-9)
    out["applied_peak"] = {a[0]: max(abs(o["act"][a[0]][0]) for t, o, g in rows) for a in ctx.m.sys.acts}
    out["stop_peak"] = {n: max(abs(o["stops"][n][0]) for t, o, g in rows) for n in rows[0][1]["stops"]}
    p = ctx.plan(u)
    out["plan"] = {n: list(v) for n, v in p.items()}
    on = []
    for i, (n, f, lo, hi) in enumerate(ctx.box):
        if u[i] <= 1e-6 or u[i] >= 1.0 - 1e-6:
            on.append("%s %s at its %s bound (%.3g)" % (n, f, "lower" if u[i] <= 1e-6 else "upper", lo if u[i] <= 1e-6 else hi))
    out["on_bounds"] = on
    out["rules_broken"] = Dr.check_rules(ctx.m, {n: v for n, v in p.items()}) if not ctx.rules else "rules relaxed: %s" % ctx.rules
    return out, rows


def show(out, log=print):
    log("   contact (bat azimuth -8 deg) at %+.1f ms%s; bat azimuth at 0 ms %.0f deg; sweet spot %.1f m/s at 0 ms, %.1f at contact"
        % (out["contact_ms"], "" if out["came_round"] else " (EXTRAPOLATED: the bat did not come round by +60 ms)",
           out["bat_az_at_0"], out["sweet_speed_at_0"], out["at_contact"]["sweet_speed"]))
    c = out["at_contact"]
    log("   at contact: pelvis %.0f deg (77 +- 6), trunk %.0f (75), lag %.0f (65 +- 10), bat elevation %.0f (-30 +- 5), lead knee %.0f (18), elbows %.0f, %.0f (56, 81)"
        % (c["pelvis_rot"], c["trunk_rot"], c["bat_lag"], c["bat_elev"], c["lead_knee_flex"], c["lead_elbow_flex"], c["trail_elbow_flex"]))
    pk = out["peaks"]
    log("   peaks: pelvis %.0f deg/s at %.0f ms (581 at -70), trunk %.0f at %.0f (766 at -57), bat azimuth %.0f at %.0f (2,435 at -10); lead foot %.0f %% BW at %.0f ms (130 at -93)"
        % (pk["pelvis_rot_vel"][0], pk["pelvis_rot_vel"][1], pk["trunk_rot_vel"][0], pk["trunk_rot_vel"][1],
           pk["bat_az_vel"][0], pk["bat_az_vel"][1], out["grf_peak_lead_z"][0], out["grf_peak_lead_z"][1]))
    if "pelvis_rate_at_-93" in out:
        log("   pelvis rate %.0f deg/s at -93 ms (558), %.0f at -70 (581); pelvis turned to %.0f deg at 0 ms (77); the ground's turning moment about the pelvis's vertical: peak %.0f N m at %.0f ms, impulse %.1f N m s over -170 ms to 0"
            % (out["pelvis_rate_at_-93"], out["pelvis_rate_at_-70"], out["pelvis_turn_at_0"], out["ground_mz_peak"][0], out["ground_mz_peak"][1], out["ground_mz_impulse_-170_0"]))
    log("   residual RMS in band units, -250 ms to 0: " + ", ".join("%s %.2f" % (k, v) for k, v in out["rms_band_units"].items()))
    log("   plan (T N m [applied peak] of bound, onset, rise, off ms): " + "; ".join(
        "%s %.0f [%.0f] %.0f %.0f %.0f" % (n, v[0], out["applied_peak"][n], v[1] * 1e3, v[2] * 1e3, v[3] * 1e3) for n, v in out["plan"].items()))
    log("   stops' largest torques (N m): " + ", ".join("%s %.0f" % (k, v) for k, v in out.get("stop_peak", {}).items()))
    log("   on a bound (%d of %d numbers): " % (len(out["on_bounds"]), 4 * len(CH)) + ("; ".join(out["on_bounds"]) or "none"))
    log("   rules: %s" % (out["rules_broken"] or "none broken"))


def fit(rules="design", starts=3, H=1.84, M=92.0, log=print, iters=(20, 20, 30), u_from=None, cock_stop=None, shoulder_stops=True):
    ctx = Ctx(H, M, rules, cock_stop, shoulder_stops=shoulder_stops)
    iA = [4 * i + k for i, n in enumerate(CH) if n in LOWER for k in range(4)]
    iB = [4 * i + k for i, n in enumerate(CH) if n in ARMS for k in range(4)]
    best = None
    for sidx in range(starts):
        t = time.time()
        if u_from is not None:
            u = [min(1.0, max(0.0, (u_from.get(n, (0.0, lo if k else 0.0, 0.05, 0.1))[k] - lo) / (hi - lo))) for n in CH
                 for k, (_, _, lo, hi) in enumerate(ctx.box[4 * CH.index(n):4 * CH.index(n) + 4])]
            log("  start %d: from a given plan, all numbers free" % sidx)
            u, c = lm(ctx, u, iA + iB, None, iters[2], log)
        else:
            u = ctx.start(sidx)
            for i in iB:                                  # stage A: the arms undriven
                if i % 4 == 0:
                    u[i] = 0.0
            log("  start %d, stage A (lower body and trunk; arms undriven)" % sidx)
            u, c = lm(ctx, u, iA, STAGE_A, iters[0], log)
            ub = ctx.start(sidx)
            for i in iB:
                u[i] = ub[i]
            log("  start %d, stage B (arms; A's numbers held)" % sidx)
            u, c = lm(ctx, u, iB, None, iters[1], log)
            log("  start %d, stage C (all 64 numbers)" % sidx)
            u, c = lm(ctx, u, iA + iB, None, iters[2], log)
        out, rows = summary(ctx, u)
        out["cost"], out["start"], out["runs"], out["seconds"] = c, sidx, ctx.evals, time.time() - t
        log("  start %d done: cost %.3f, %d runs so far, %.0f s" % (sidx, c, ctx.evals, time.time() - t))
        show(out, log)
        if best is None or c < best["cost"]:
            best = out
    return ctx, best


def fit_lower(weighting, starts=3, iters=40, log=print, scale=None):
    """the lower body alone (the 14 leg and trunk channels; the arms undriven), under the design's rules, against
    one of LOWER_SETS. scale = {channel: factor} relaxes bounds for a diagnosis."""
    ctx = Ctx(rules="design" if not scale else {"scale": scale})
    which = LOWER_SETS[weighting] or STAGE_A
    iA = [4 * i + k for i, n in enumerate(CH) if n in LOWER for k in range(4)]
    best = None
    for sidx in range(starts):
        t = time.time()
        u = ctx.start(sidx)
        for i, n in enumerate(CH):
            if n in ARMS:
                u[4 * i] = 0.0
        log("  start %d (lower body alone, weighting '%s': %s)" % (sidx, weighting, ", ".join(which)))
        u, c = lm(ctx, u, iA, which, iters, log)
        out, rows = summary(ctx, u)
        out["cost"], out["start"], out["runs"] = c, sidx, ctx.evals
        log("  start %d done: cost %.3f, %d runs so far, %.0f s" % (sidx, c, ctx.evals, time.time() - t))
        show(out, log)
        if best is None or c < best["cost"]:
            best = out
    return ctx, best


def main():
    P.build()
    a = sys.argv[1:]
    arg = lambda k, d: a[a.index(k) + 1] if k in a else d
    stops = "--no-shoulder-stops" not in a
    tag = "" if stops else "_nostop"
    base_file = os.path.join(HERE, "fit3d_design%s.json" % tag)
    it = int(arg("--iters", 25))
    if "--lower" in a:
        w = arg("--lower", "forces")
        sc = None
        if "--scale-legs" in a:
            k = float(arg("--scale-legs", 1.5))
            sc = {n: k for n in LOWER}
        print("the lower body alone, weighting '%s', %d starts%s" % (w, int(arg("--starts", 3)), "" if not sc else ", EVERY LEG AND TRUNK BOUND x %.2f (a diagnosis, not the design)" % k))
        ctx, best = fit_lower(w, int(arg("--starts", 3)), int(arg("--iters", 40)), scale=sc)
        print("\nbest start: %d, cost %.3f" % (best["start"], best["cost"]))
        show(best)
        json.dump(best, open(os.path.join(P.OUT, "fit3d_lower_%s%s.json" % (w, "" if not sc else "_x%.2f" % k)), "w"), indent=1)
        return
    if "--limits" in a:
        base = json.load(open(base_file))
        for name in RULES:
            if name == "design":
                continue
            print("\n== relaxed: %s (re-fitted from the design fit, all numbers free)" % name)
            ctx, best = fit(name, 1, u_from=base["plan"], iters=(0, 0, it), shoulder_stops=stops)
            json.dump(best, open(os.path.join(P.OUT, "fit3d_%s%s.json" % (name, tag)), "w"), indent=1)
        return
    if "--show" in a:
        show(json.load(open(os.path.join(HERE, arg("--show", "fit3d_design.json")))))
        return
    if "--small" in a:
        base = json.load(open(base_file))
        Hs, Ms = float(arg("--height", 1.75)), float(arg("--mass", 77.0))
        print("the small frame (PLAN D32): %.2f m, %.0f kg [E]; his bounds are the same sourced bounds at his weight x height;" % (Hs, Ms))
        print("re-fitted under the design's rules from the average batter's plan (strengths over his bounds start at his bounds)")
        ctx, best = fit("design", 1, Hs, Ms, u_from=base["plan"], iters=(0, 0, it), shoulder_stops=stops)
        print("\nthe average batter (1.84 m, 92 kg), for comparison:")
        show(base)
        json.dump(best, open(os.path.join(P.OUT, "fit3d_small%s.json" % tag), "w"), indent=1)
        return
    if "--wrists" in a:
        base = json.load(open(base_file))
        print("WITHOUT a stop on the wrists' cock (the design fit):")
        show(base)
        for margin in (5.0, 20.0):
            print("\nWITH a one-sided stop %.0f deg past the wrists' cock at the start [E], re-fitted from the design fit:" % margin)
            ctx, best = fit("design", 1, u_from=base["plan"], iters=(0, 0, it), cock_stop=margin, shoulder_stops=stops)
            json.dump(best, open(os.path.join(P.OUT, "fit3d_cock%d%s.json" % (margin, tag)), "w"), indent=1)
        return
    rules = arg("--rules", "design")
    starts = int(arg("--starts", 3))
    print("rules %s, %d start(s), body 1.84 m 92 kg, from %.0f ms, step %.1f ms, shoulders' range stops %s" % (rules, starts, T0 * 1e3, H_STEP * 1e3, "on" if stops else "off"))
    ctx, best = fit(rules, starts, iters=(it, it, int(1.5 * it)), shoulder_stops=stops)
    print("\nbest start: %d, cost %.3f" % (best["start"], best["cost"]))
    show(best)
    if rules == "design":
        json.dump(best, open(base_file, "w"), indent=1)
        out, rows = summary(ctx, [min(1.0, max(0.0, (best["plan"][n][k] - lo) / (hi - lo))) for n in CH
                                  for k, (_, _, lo, hi) in enumerate(ctx.box[4 * CH.index(n):4 * CH.index(n) + 4])], h=5e-4)
        print("\nthe same plan at a 0.5 ms step:")
        show(out)


if __name__ == "__main__":
    main()
