"""The three-dimensional body's fit to Fortenbaugh's mean swing under the design's rules
(docs/body-motion.md 4.3's rules, 4.5's limits, 13's gate; PLAN D31 phase 1, sessions 2, 5 and 6; PLAN D34).

    python3 analysis/research/body/fit3d.py --starts 3        the whole body under body-motion 4.5's limits (job queue)
    python3 analysis/research/body/fit3d.py --relax           one rule relaxed at a time, from that fit
    python3 analysis/research/body/fit3d.py --ranges          4.5's range ends, one at a time (--only a,b for some)
    python3 analysis/research/body/fit3d.py --small           PLAN D32's small frame
    python3 analysis/research/body/fit3d.py --wrists          with a stop on the wrists' cock
    python3 analysis/research/body/fit3d.py --lower forces    the legs and trunk alone (also: pelvis, rate)
    python3 analysis/research/body/fit3d.py --rules wrist_delp --starts 2   the same with the wrists on Delp's reading (4.5)
    python3 analysis/research/body/fit3d.py --show fit3d_d34_arms.json

This FITS THE MODEL'S DRIVE NUMBERS TO A RECORDED AVERAGE SWING in the laboratory. Nothing in a game is
fitted; nothing here tells the body where the bat should end, except as a recorded curve to compare with.

The body starts at -250 ms on [4]'s measured state, moving (body3d.body.place). 22 drive channels, each
(strength, onset, rise, switch-off, starting level): 110 numbers. The rules are the search's box: strengths
inside body-motion 4.5's bounds, rises of at least 50 ms, every onset inside the window, no arm drive rising
above its holding level more than 70 ms (the measured electromechanical delay's top) before -170 ms, and the
bat's first motion (its azimuth rate through [4]'s 236 deg/s) asked for at -170 ms. The search is
Levenberg-Marquardt on the residuals in band units with a finite-difference Jacobian, projected onto the
box, run through the compiled probe (probe3d.py). It is staged: (A) the lower body and trunk against [4]'s
ground forces, pelvis, trunk and lead knee, the arms undriven; (B) the arms against the bat and elbows, A's
numbers held; (C) all the numbers against everything. The reported numbers are from one run of the whole
body with C's result. Starts: start 0 is a hand estimate from [4]'s event times at half strength; the
others perturb it with a fixed seed. The gate's curves are [4]'s RATE curves (4.5).
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
         "knee_ext_R", "knee_ext_L", "knee_flex_R", "knee_flex_L", "ankle_pf_R", "ankle_pf_L", "core", "core_back")
ARMS = Dr.ARM_CHANNELS + Dr.NEW_ARM_CHANNELS
NEW = ("core_back",) + Dr.NEW_ARM_CHANNELS      # session 7's channels (body-motion 4.5)
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
            "wrist_L": -0.090, "wrist_R": -0.090, "core_back": -0.080}
START_ON.update({n: -0.170 for n in Dr.NEW_ARM_CHANNELS})
START_T, START_RISE, START_OFF, START_A0, START_A0_ARMS = 0.5, 0.080, 0.050, 0.2, 0.1
START_T_NEW, START_A0_NEW = 0.25, 0.0     # session 7's channels start at a quarter strength, not tensed: a pair pulls both ways
FIRST_MOTION_BAND = 0.5 * Dr.FIRST_MOTION_TOL     # the bat's first motion: the tolerance's edge costs as much as 2 band units
SPEED_ASK, SPEED_BAND = 45.0, 0.5                 # m/s: the strongest-batter run's (unreachable) ask and its unit

_c1 = lambda n: Dr.C1[n][0] / (Dr.C1[n][0] + Dr.SD_COUNT * Dr.C1[n][1])
_pre = lambda *ps: {n: None for n in CH if n.startswith(ps)}
def _sc(k, *ps):
    return {n: k for n in CH if n.startswith(ps)}
# one rule relaxed at a time (a diagnosis; not the design)
RELAX = {
    "continued": {},
    "no_first_motion_rule": {"first_motion": False, "emd": 0.080},     # the arms' drive from the window's start, the bat's first motion not asked for
    "shoulders_elbows_x2": {"scale": _sc(2.0, "shoulder", "elbow", "sh_")},
    "no_rise_floor": {"rise_min": 0.005},
    "legs_trunk_x1.5": {"scale": {n: 1.5 for n in LOWER}},
    "wrists_x3": {"scale": _sc(3.0, "wrist")},
}
# the range ends of body-motion 4.5: what the answer rests on (session 5's leg and trunk ends, then the arms')
RANGES = {
    "hip_extension_524": {"scale": _sc(524.0 / 430.1, "hip_ext")},
    "legs_at_the_sources_means": {"scale": {**_sc(_c1("hip_ext"), "hip_ext"), **_sc(_c1("hip_flex"), "hip_flex"), **_sc(_c1("knee_ext"), "knee_ext"),
                                            **_sc(_c1("knee_flex"), "knee_flex"), **_sc(_c1("ankle_pf"), "ankle_pf")}},
    "trunk_290": {"scale": {"core": 290.0 / 260.0}},
    "hip_extension_curve_andersons": {"hip_curve": "hip_ext_anderson"},
    "no_tendon_springs_at_the_legs": {"tendons": False},
    "legs_and_trunk_not_tensed_beforehand": {"pretension": False},
    "shoulders_kentels_player": {"scale": {"shoulder_R": 97.48 / Dr.SHOULDER_HADD, "shoulder_L": 71.90 / Dr.SHOULDER_HABD}, "shoulder_curves": "kentel"},
    "shoulders_136_112": {"scale": {"shoulder_R": 136.0 / Dr.SHOULDER_HADD, "shoulder_L": 112.0 / Dr.SHOULDER_HABD}},
    "elbows_52": {"scale": _sc(52.0 / Dr.ELBOW_EXT, "elbow")},
    "elbows_76": {"scale": _sc(76.0 / Dr.ELBOW_EXT, "elbow")},
    "no_tendon_springs_at_the_arms": {"arm_tendons": {"sh_hadd": False, "sh_habd": False, "elbow_ext": False, "wrist": False}},
    "elbows_without_tendon_springs": {"arm_tendons": {"sh_hadd": True, "sh_habd": True, "elbow_ext": False, "wrist": False}},
    "wrists_with_their_printed_tendon": {"arm_tendons": {"sh_hadd": True, "sh_habd": True, "elbow_ext": True, "wrist": True}},
    "arms_not_tensed_beforehand": {"arm_pretension": False},
    "delay_at_most_10_ms": {"emd": Dr.EMD_MIN},
    # session 7's range ends (body-motion 4.5's last block)
    "new_shoulder_channels_kentels_player": {"bound_scale": {**_sc(56.29 / Dr.SHOULDER_RAISE, "sh_raise"), **_sc(119.13 / Dr.SHOULDER_LOWER, "sh_lower"),
                                                             **_sc(42.67 / Dr.SHOULDER_IROT, "sh_irot"), **_sc(33.75 / Dr.SHOULDER_EROT, "sh_erot")},
                                             "new_shoulder_curves": "kentel"},
    "elbow_flexors_76": {"bound_scale": _sc(75.82 / Dr.ELBOW_FLEX, "elbow_flex")},
    "forearm_x2_a_diagnosis": {"bound_scale": _sc(2.0, "pron", "sup")},
    "wrist_extension_16": {"bound_scale": _sc(15.77 / Dr.WRIST_EXT, "wrist_ext")},
    "trunk_brake_260": {"bound_scale": {"core_back": Dr.TRUNK_REF / Dr.TRUNK_BACK_REF}},
    "new_channels_without_tendon_springs": {"arm_tendons": {"sh_raise": False, "sh_lower": False, "sh_irot": False, "elbow_flex": False}},
}
# one of session 7's drives taken out at a time, and the body as session 6 had it (a diagnosis of which drive does what)
WITHOUT = {
    "session_6_body": {"only": LOWER[:-1] + Dr.ARM_CHANNELS},
    "without_the_trunks_brake": {"only": tuple(n for n in CH if n != "core_back")},
    "without_shoulder_rotation": {"only": tuple(n for n in CH if "rot_" not in n or n.startswith("hip"))},
    "without_raising_and_lowering": {"only": tuple(n for n in CH if not n.startswith(("sh_raise", "sh_lower")))},
    "without_elbow_flexors": {"only": tuple(n for n in CH if not n.startswith("elbow_flex"))},
    "without_forearm_rotation": {"only": tuple(n for n in CH if not n.startswith(("pron", "sup")))},
    "without_wrist_flexion_and_extension": {"only": tuple(n for n in CH if not n.startswith(("wrist_flex", "wrist_ext")))},
}
STRONGEST = {"strongest": {"goal": "speed"}, "strongest_on_time": {"goal": "speed", "tight": 10.0},
             "strongest_held": {"goal": "speed", "tight": 10.0, "held": True}}
# the strongest batter held to a hitter's swing (body-motion 4.9, session 7 fix; stated there before the run): asked for
# speed as strongest_on_time, and in addition, each a one-sided penalty that is zero inside its limit:
HELD_CURVES = ("pelvis_rot_vel", "pelvis_y", "lead_knee_flex", "lead_grf_z", "trail_grf_z", "lead_grf_x", "trail_grf_x",
               "lead_grf_y", "trail_grf_y")     # the gate's ground-force and lower-body curves: RMS at most 1 band unit
HELD_W = 30.0                # the penalty's weight per band unit (or unit below) past its limit [E]; stated before the run
STOP_SLACK, STOP_UNIT = 5.0, 10.0   # N m: no stop may carry more than the average fit's swing makes it carry (+5), per 10 N m
ATTACK_RANGE, ATTACK_UNIT = (-3.3, 10.7), 2.0   # deg: OpenBiomechanics' p10-p90 at contact (note 6); per 2 deg past it
STOP_REF = {}                # the average fit's largest stop torques, set before a held run
RULES = {"design": {}, "wrist_delp": {"wrist": "delp"}, **RELAX, **RANGES, **WITHOUT, **STRONGEST}


class Ctx:
    """one placed, toned, damped body with its channels, reused for every evaluation."""

    def __init__(self, H=1.84, M=92.0, rules="design", cock_stop=None, t0=T0, shoulder_stops=True):
        self.rules = dict(RULES[rules]) if isinstance(rules, str) else dict(rules)
        r = self.rules
        self.t0 = t0
        self.m = m = B.Model(H, M, shoulder_stops=shoulder_stops)
        st = B.place(m, t0)
        self.aux0 = list(m.start_aux)
        keep = (Dr.HIP_SIDE_CURVE, Dr.CURVES["hip_ext"], Dr.TENDONS, Dr.WRIST_READING, Dr.SHOULDER_CURVES, dict(Dr.ARM_TENDONS), Dr.NEW_SHOULDER_CURVES)
        if "new_shoulder_curves" in r:
            Dr.NEW_SHOULDER_CURVES = r["new_shoulder_curves"]
        if "side_curve" in r:
            Dr.HIP_SIDE_CURVE = r["side_curve"]
        if "hip_curve" in r:
            Dr.CURVES["hip_ext"] = Dr.CURVES[r["hip_curve"]]
        if "tendons" in r:
            Dr.TENDONS = r["tendons"]
        if "wrist" in r:
            Dr.WRIST_READING = r["wrist"]
        if "shoulder_curves" in r:
            Dr.SHOULDER_CURVES = r["shoulder_curves"]
        if "arm_tendons" in r:
            Dr.ARM_TENDONS.update(r["arm_tendons"])
        Dr.BOUND_SCALE.clear(); Dr.BOUND_SCALE.update(r.get("bound_scale", {}))   # a bound itself (its tendon spring with it)
        self.bd = Dr.bounds(H, M)                        # after the switches: the wrists' reading sets their bound
        for n, k in r.get("scale", {}).items():
            self.bd[n] *= k
        st = Dr.add_drives(m, st, {n: (0.0, 0.0, 0.05, 0.1, 0.0) for n in CH})
        Dr.HIP_SIDE_CURVE, Dr.CURVES["hip_ext"], Dr.TENDONS, Dr.WRIST_READING, Dr.SHOULDER_CURVES = keep[:5]
        Dr.ARM_TENDONS.clear(); Dr.ARM_TENDONS.update(keep[5])
        Dr.NEW_SHOULDER_CURVES = keep[6]
        Dr.BOUND_SCALE.clear()
        self.tone, self.tone_rest = Dr.solve_tone(m, st, t0, aux=list(self.aux0))
        Dr.add_damping(m, st)
        if cock_stop is not None:
            add_cock_stop(m, st, cock_stop)
        self.st = st
        self.BW = M * E.G
        rise_min = r.get("rise_min", Dr.RISE_MIN)
        self.emd = r.get("emd", Dr.EMD_MAX)
        arms = max(t0, Dr.FIRST_MOTION_T - self.emd)     # no arm drive rises above its holding level before this
        self.first_motion = r.get("first_motion", True)
        only = r.get("only")                            # session 6's body: the channels named, the rest switched off
        if only is not None:
            for n in CH:
                if n not in only:
                    self.bd[n] = 0.0
        pre = 1.0 if r.get("pretension", True) else 0.0
        pre_arms = 1.0 if r.get("arm_pretension", True) else 0.0
        self.box = []                                   # (channel, field, lo, hi)
        for n in CH:
            self.box += [(n, "T", 0.0, self.bd[n]), (n, "on", arms if n in ARMS else t0, 0.0),
                         (n, "rise", rise_min, 0.250), (n, "off", -0.150, 0.100), (n, "start", 0.0, pre_arms if n in ARMS else pre)]
        self.free = [i for i, (n, f, lo, hi) in enumerate(self.box) if hi > lo and self.bd[n] > 0.0]
        self.goal = r.get("goal")                       # "speed": the strongest batter (residuals())
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
        plan = {n: ((START_T_NEW if n in NEW else START_T) * self.bd[n], START_ON[n], START_RISE, START_OFF,
                    START_A0_NEW if n in NEW else START_A0_ARMS if n in ARMS else START_A0) for n in CH}
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

    def first_motion_time(self, rows):
        """when the bat's azimuth rate first rises through [4]'s 236 deg/s (4.5): the bat's first visible motion.
        If it starts above or never gets there, the window's start or the run's end."""
        thr = Dr.FIRST_MOTION_RATE
        if rows[0][1]["bat_az_vel"] >= thr:
            return rows[0][0]
        for (ta, oa, _), (tb, ob, _) in zip(rows, rows[1:]):
            if oa["bat_az_vel"] < thr <= ob["bat_az_vel"]:
                return ta + (thr - oa["bat_az_vel"]) / (ob["bat_az_vel"] - oa["bat_az_vel"]) * (tb - ta)
        return rows[-1][0]

    def residuals(self, u, which=None):
        try:
            rows = self.run(u)
        except E.SolveError:
            return None
        val, tc, oc = self.measures(rows)
        r = []
        n = len(self.times)
        if self.goal == "speed":
            # the strongest batter the limits allow: nothing asked but the sweet spot's speed at contact, the ball met
            # on time and the bat first moving when [4]'s does. SPEED_ASK is out of reach on purpose.
            tight = self.rules.get("tight", 1.0)        # > 1: the timing held that much harder (the second strongest-batter run)
            r = [(SPEED_ASK - oc["sweet_speed"]) / SPEED_BAND, tc / CONTACT_BAND * tight]
            if self.first_motion:
                tfm = self.first_motion_time(rows)
                r.append((tfm - Dr.FIRST_MOTION_T) / FIRST_MOTION_BAND * tight)
                if tight > 1.0:                         # and no arm drive rising more than the measured delay before the bat moves
                    pl = self.plan(u)
                    on = [pl[nm][1] for nm in ARMS if pl[nm][0] > 0.01 * self.bd[nm]]
                    r.append(max(0.0, (tfm - min(on)) - self.emd) / 0.001 if on else 0.0)
            if self.rules.get("held"):
                n = len(self.times)
                for k in HELD_CURVES:
                    rms = math.sqrt(sum(((a - b) / BAND[k]) ** 2 for a, b in zip(val[k], self.target[k])) / n)
                    r.append(HELD_W * max(0.0, rms - 1.0))
                upto = [o for t, o, g in rows if t <= tc + 1e-9]
                for nm, ref in STOP_REF.items():
                    pk = max(abs(o["stops"][nm][0]) for o in upto)
                    r.append(HELD_W * max(0.0, pk - ref - STOP_SLACK) / STOP_UNIT)
                lo, hi = ATTACK_RANGE
                r.append(HELD_W * max(0.0, lo - oc["attack"], oc["attack"] - hi) / ATTACK_UNIT)
            return None if any(x != x for x in r) else r
        for k in (which or BAND):
            w = 1.0 / (BAND[k] * math.sqrt(n))
            r += [(a - b) * w for a, b in zip(val[k], self.target[k])]
        if which is None:
            r.append(tc / CONTACT_BAND)
            for k, (mean, sd) in AT_CONTACT.items():
                r.append((oc[k] - mean) / sd)
            if self.first_motion:
                r.append((self.first_motion_time(rows) - Dr.FIRST_MOTION_T) / FIRST_MOTION_BAND)
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
            log("      iteration %2d: cost %.4f, lambda %.1e, %d runs%s" % (it + 1, cost, lam, ctx.evals,
                ", sweet spot at contact %.2f m/s" % (SPEED_ASK - SPEED_BAND * r[0]) if ctx.goal == "speed" else ""))
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
    tfm = ctx.first_motion_time(rows)
    pl = ctx.plan(u)
    arm_on = [(pl[nm][1], nm) for nm in ARMS if pl[nm][0] > 0.01 * ctx.bd[nm]]
    out["first_motion_ms"] = tfm * 1e3
    out["arm_onset_first"] = (min(arm_on)[0] * 1e3, min(arm_on)[1]) if arm_on else None
    out["delay_ms"] = (tfm - min(arm_on)[0]) * 1e3 if arm_on else None
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
        if i in ctx.free and (u[i] <= 1e-6 or u[i] >= 1.0 - 1e-6):
            on.append("%s %s %s" % (nm, f, "low" if u[i] <= 1e-6 else "HIGH"))
    out["on_bounds"] = on
    out["n_free"] = len(ctx.free)
    broken = Dr.check_rules(ctx.m, p)
    if abs(tfm - Dr.FIRST_MOTION_T) > Dr.FIRST_MOTION_TOL + 1e-9:
        broken.append("the bat's first motion is at %.0f ms, outside -170 +- 15" % (tfm * 1e3))
    if out["delay_ms"] is not None and out["delay_ms"] > Dr.EMD_MAX * 1e3 + 1e-6:
        broken.append("the first arm drive rises %.0f ms before the bat's first motion (at most 70)" % out["delay_ms"])
    out["rules_broken"] = broken if not ctx.rules or set(ctx.rules) == {"wrist"} else "changed for this run: %s%s" % (", ".join(ctx.rules), ("; and " + "; ".join(broken)) if broken else "")
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
    if "first_motion_ms" in out:
        ao = out["arm_onset_first"]
        log("   the bat's first motion (its rate through 236 deg/s) at %.0f ms (-170 +- 15); the first arm drive rises at %s; the delay between them %s ms (measured range 10-70)"
            % (out["first_motion_ms"], "-" if not ao else "%.0f ms (%s)" % tuple(ao), "-" if out["delay_ms"] is None else "%.0f" % out["delay_ms"]))
    log("   residual RMS in band units, -250 ms to 0: " + ", ".join("%s %.2f" % (k, v) for k, v in out["rms_band_units"].items()))
    log("   plan: channel  strength of bound [applied peak, at rad/s]  onset rise off ms  starting level")
    for nm, v in out["plan"].items():
        ap = out["applied_at_speed"][nm]
        log("      %-14s %4.0f of %4.0f [%4.0f at %+5.1f]  %5.0f %4.0f %5.0f  %.2f" % (nm, v[0], out["bounds"][nm], ap[0], ap[1], v[1] * 1e3, v[2] * 1e3, v[3] * 1e3, v[4]))
    log("   stops' largest torques (N m): " + ", ".join("%s %.0f" % (k, v) for k, v in out["stop_peak"].items()))
    log("   tendon springs' largest stored energy (J): " + (", ".join("%s %.0f" % (k, v) for k, v in out["tendon_peak_J"].items()) or "none"))
    log("   on a bound (%d of %d numbers): " % (len(out["on_bounds"]), out["n_free"]) + ("; ".join(out["on_bounds"]) or "none"))
    log("   rules: %s" % (out["rules_broken"] or "none broken"))


HELD_BAND = 10.0             # deg [E]: the held-out shoulder elevations' unit (body-motion 4.5, session 7)
ATTACK_MEASURED = (4.1, 5.9)  # deg: OpenBiomechanics' attack angle at contact, mean and SD (research note swing/6; a check, never fitted)
ATTACK_PLACED = 5.6          # deg: [4]'s motion as placed on this body, at contact (batspeed.py section 4)


def held_out(ctx, u, log=print):
    """the measured things no fit is given (body-motion 4.5, session 7), for the plan u: [4]'s shoulder elevation
    curves, [3]'s arm rate, the batting electromyography of note 9, and whether the bat slows before contact."""
    rows = ctx.run(u)
    val, tc, oc = ctx.measures(rows)
    pl = ctx.plan(u)
    upto = [r for r in rows if r[0] <= 1e-9]
    out = {}
    m2 = B.Model(ctx.m.H, ctx.m.M, contacts=False)
    for k, nm in (("trail_sh_elev", "rear"), ("lead_sh_elev", "lead")):
        rms = math.sqrt(sum((o[k] - B.curve(k, t)) ** 2 for t, o, g in upto) / len(upto)) / HELD_BAND
        pts = [-0.250 + 0.010 * i for i in range(26)]
        placed = math.sqrt(sum((m2.observe(B.pose(m2, t)[0])[k] - B.curve(k, t)) ** 2 for t in pts) / len(pts)) / HELD_BAND
        at = lambda tt: next(o[k] for t, o, g in rows if abs(t - tt) < 1e-9)
        out[k] = (rms, placed)
        log("   held out 1, the %s shoulder's elevation: RMS %.2f units of %.0f deg (the placed motion itself: %.2f); at -250, -170, -95, 0 ms: %.0f, %.0f, %.0f, %.0f ([4]: %.0f, %.0f, %.0f, %.0f)"
            % (nm, rms, HELD_BAND, placed, at(-0.250), at(-0.170), at(-0.095), at(0.0), B.curve(k, -0.250), B.curve(k, -0.170), B.curve(k, -0.095), B.curve(k, 0.0)))
    az = [o["lead_arm_az"] for t, o, g in rows]
    for i in range(1, len(az)):
        while az[i] - az[i - 1] > 180.0:
            az[i] -= 360.0
        while az[i] - az[i - 1] < -180.0:
            az[i] += 360.0
    rate = [((az[i + 1] - az[i - 1]) / (rows[i + 1][0] - rows[i - 1][0]), rows[i][0]) for i in range(1, len(rows) - 1) if rows[i][0] <= tc]
    pk = max(rate)
    out["arm_rate_peak"] = (pk[0], pk[1] * 1e3)
    log("   held out 2, the lead arm's turning rate (shoulder to wrist, about the vertical [E]): peak %.0f deg/s at %.0f ms ([3]: 1,160 +- 96 at -65 ms)" % (pk[0], pk[1] * 1e3))
    used = lambda n: pl[n][0] > 0.05 * ctx.bd[n] if ctx.bd[n] > 0 else False
    eff = lambda n, t: next(o["act"][n][2] for tt, o, g in rows if abs(tt - t) < 1e-9) * (pl[n][0] / ctx.bd[n] if ctx.bd[n] > 0 else 0.0)
    mean_eff = lambda n, a, b_: (lambda v: sum(v) / len(v))([o["act"][n][2] for tt, o, g in rows if a - 1e-9 <= tt <= b_ + 1e-9]) * (pl[n][0] / ctx.bd[n] if ctx.bd[n] > 0 else 0.0)
    first = {}
    for grp, names in (("legs", [n for n in LOWER if not n.startswith("core")]), ("trunk", ["core"]), ("arms", list(ARMS))):
        ons = [(pl[n][1], n) for n in names if used(n)]
        first[grp] = min(ons) if ons else (float("nan"), "-")
    out["first_rise_ms"] = {g: (v[0] * 1e3, v[1]) for g, v in first.items()}
    out["tensed_beforehand"] = {n: pl[n][4] for n in CH if used(n) and pl[n][4] > 0.05}
    log("   held out 3a, the first rise above its holding level: legs %.0f ms (%s), trunk %.0f (%s), arms %.0f (%s)   [recorded order: legs, trunk, arms]"
        % (first["legs"][0] * 1e3, first["legs"][1], first["trunk"][0] * 1e3, first["trunk"][1], first["arms"][0] * 1e3, first["arms"][1]))
    log("      tensed beforehand (starting level over 0.05): " + (", ".join("%s %.2f" % kv for kv in out["tensed_beforehand"].items()) or "none"))
    e_b = [eff("shoulder_L", t) for t in (-0.250, -0.170, -0.100, -0.050, 0.0)]
    log("   held out 3b, the lead shoulder's sweep (posterior deltoid, 76-101 %% of a maximum test from pre-swing on): effort at -250, -170, -100, -50, 0 ms: %s of its bound"
        % ", ".join("%.2f" % x for x in e_b))
    e_c = (mean_eff("elbow_ext_L", -0.170, -0.110), mean_eff("elbow_ext_L", -0.110, -0.055), mean_eff("elbow_ext_L", -0.055, 0.0))
    log("   held out 3c, the lead elbow's extensor (triceps, highest early: 92 %%, then 73 %%): mean effort early, middle, late in the swing: %.2f, %.2f, %.2f" % e_c)
    e_d = max(o["act"]["sh_raise_L"][2] for tt, o, g in upto) * (pl["sh_raise_L"][0] / ctx.bd["sh_raise_L"] if ctx.bd["sh_raise_L"] > 0 else 0.0)
    log("   held out 3d, raising the lead arm (supraspinatus, under 40 %% throughout): peak effort %.2f" % e_d)
    out["emg"] = {"lead_sweep": e_b, "lead_triceps": e_c, "lead_raise_peak": e_d}
    out["new_drives"] = {}
    for n in NEW:
        if ctx.bd[n] <= 0:
            continue
        pe = max((o["act"][n][2] * pl[n][0] / ctx.bd[n], tt) for tt, o, g in upto)
        pt = max((abs(o["act"][n][0]), tt, o["act"][n][1]) for tt, o, g in upto)
        out["new_drives"][n] = (pe[0], pe[1] * 1e3, pt[0], pt[1] * 1e3, pt[2])
    log("   session 7's drives: peak effort (of the bound) and when; largest torque applied, when, at what joint speed")
    for n, v in out["new_drives"].items():
        log("      %-14s %.2f at %4.0f ms; %5.1f N m at %4.0f ms, %+5.1f rad/s; onset %4.0f ms, starting level %.2f" % (n, v[0], v[1], v[2], v[3], v[4], pl[n][1] * 1e3, pl[n][4]))
    sp = [(o["sweet_speed"], t) for t, o, g in rows if t <= tc + 1e-9] + [(oc["sweet_speed"], tc)]
    pk = max(sp)
    out["bat_peak_before_contact"] = (pk[0], pk[1] * 1e3, oc["sweet_speed"] / pk[0])
    log("   held out 4, the bat before contact: its fastest %.1f m/s at %.0f ms; at contact %.0f %% of that (measured hitters: 98 %%, the peak 2 +- 5 ms before)"
        % (pk[0], pk[1] * 1e3, 100 * oc["sweet_speed"] / pk[0]))
    at = lambda tt: next(o["attack"] for t, o, g in rows if abs(t - tt) < 1e-9)
    out["attack"] = (oc["attack"], at(-0.020), at(-0.010), at(0.0))
    log("   held out 5, the attack angle (the sweet spot's path above level) at contact: %+.1f deg; at -20, -10, 0 ms: %+.1f, %+.1f, %+.1f"
        "   [OpenBiomechanics %+.1f +- %.1f at contact, 677 swings of mostly college hitters (note 6); [4]'s motion placed on this body %+.1f]"
        % (out["attack"] + ATTACK_MEASURED + (ATTACK_PLACED,)))
    return out


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
    s6_file = os.path.join(HERE, "fit3d_d34_arms.json")         # session 6's fit: 22 channels (session 5's stays in fit3d_d34.json)
    base_file = os.path.join(HERE, "fit3d_d34_all.json")        # session 7's: 41 channels
    if "--held" in a:
        which = arg("--held", "all")
        f, rules = (s6_file, "session_6_body") if which == "s6" else (which if os.path.isabs(which) or os.path.exists(which) else base_file, arg("--rules", "design"))
        plan = json.load(open(f))["plan"]
        ctx = Ctx(rules=rules)
        print("the held-out checks of body-motion 4.5 (session 7) for %s under rules '%s'" % (os.path.basename(f), rules))
        u = ctx.u_of(plan)
        out, rows = summary(ctx, u)
        show(out)
        held_out(ctx, u)
        return
    it = int(arg("--iters", 25))
    if "--show" in a:
        show(json.load(open(os.path.join(HERE, arg("--show", "fit3d_d34_arms.json")))))
        return
    if "--lower" in a:
        w = arg("--lower", "forces")
        ctx, best = fit_lower(w, int(arg("--starts", 3)), int(arg("--iters", 40)))
        json.dump(best, open(os.path.join(P.OUT, "fit3d_d34_lower_%s.json" % w), "w"), indent=1)
        return
    if "--strongest" in a:
        base = json.load(open(base_file))
        print("the strongest batter the limits allow: from the design fit, every number free inside the rules, asked only for the sweet spot's speed at contact, the ball met on time and the bat's first motion at -170 ms")
        name = "strongest_on_time" if "--on-time" in a else "strongest_held" if "--held-swing" in a else "strongest"
        if name == "strongest_held":
            ref = Ctx(rules="design")
            rows_ref = ref.run(ref.u_of(base["plan"]))
            tc_ref = ref.measures(rows_ref)[1]
            STOP_REF.update({nm: max(abs(o["stops"][nm][0]) for t, o, g in rows_ref if t <= tc_ref + 1e-9) for nm in rows_ref[0][1]["stops"]})
            print("held to a hitter's swing: the gate's ground-force and lower-body curves within 1 band unit (%s), no stop past the average fit's"
                  " largest (%s N m, +%.0f), the attack angle at contact in %.1f to %.1f deg, the timing held hard; each a one-sided penalty of weight %.0f"
                  % (", ".join(HELD_CURVES), ", ".join("%s %.0f" % kv for kv in STOP_REF.items()), STOP_SLACK, ATTACK_RANGE[0], ATTACK_RANGE[1], HELD_W))
        if "--from" in a:
            base = json.load(open(arg("--from", "")))
        ctx, best = fit(name, 1, u_from=base["plan"], iters=(0, 0, it))
        held_out(ctx, ctx.u_of(best["plan"]))
        json.dump(best, open(os.path.join(P.OUT, "fit3d_all_%s.json" % name), "w"), indent=1)
        return
    if "--relax" in a or "--ranges" in a or "--without" in a:
        base = json.load(open(base_file))
        sets = RELAX if "--relax" in a else RANGES if "--ranges" in a else WITHOUT
        only = arg("--only", None)
        for name in sets:
            if only and name not in only.split(","):
                continue
            print("\n== %s (re-fitted from the design fit, all numbers free)" % name)
            ctx, best = fit(name, 1, u_from=base["plan"], iters=(0, 0, it))
            json.dump(best, open(os.path.join(P.OUT, "fit3d_all_%s.json" % name), "w"), indent=1)
        return
    if "--small" in a:
        base = json.load(open(base_file))
        Hs, Ms = float(arg("--height", 1.75)), float(arg("--mass", 77.0))
        print("the small frame (PLAN D32): %.2f m, %.0f kg [E]; his bounds are the same sourced bounds at his weight x height;" % (Hs, Ms))
        print("re-fitted under the design's rules from the average batter's plan (a strength over his bound starts at his bound)")
        ctx, best = fit("design", 1, Hs, Ms, u_from=base["plan"], iters=(0, 0, it))
        print("\nthe average batter (1.84 m, 92 kg), for comparison:")
        show(base)
        json.dump(best, open(os.path.join(P.OUT, "fit3d_all_small.json"), "w"), indent=1)
        return
    if "--wrists" in a:
        base = json.load(open(base_file))
        for margin in (5.0, 20.0):
            print("\nWITH a one-sided stop %.0f deg past the wrists' cock at the start [E], re-fitted from the design fit:" % margin)
            ctx, best = fit("design", 1, u_from=base["plan"], iters=(0, 0, it), cock_stop=margin)
            json.dump(best, open(os.path.join(P.OUT, "fit3d_all_cock%d.json" % margin), "w"), indent=1)
        return
    starts = int(arg("--starts", 3))
    rules = arg("--rules", "design")                 # "wrist_delp": the wrists on Delp's reading (4.5), a whole fit of its own
    if rules != "design":
        base_file = os.path.join(HERE, "fit3d_d34_arms_%s.json" % rules)
    print("body-motion 4.5's limits (%s), %d start(s), body 1.84 m 92 kg, from %.0f ms, step %.1f ms" % (rules, starts, T0 * 1e3, H_STEP * 1e3))
    ctx, best = fit(rules, starts, iters=(it, it, int(arg("--iters-all", int(1.5 * it)))))
    if "--warm" in a:                                # one more start: session 6's fitted plan, the new channels off
        print("\none more start: session 6's fit (fit3d_d34_arms.json), session 7's channels switched off, every number free")
        ctx2, warm = fit(rules, 1, u_from=json.load(open(s6_file))["plan"], iters=(0, 0, int(arg("--iters-all", int(1.5 * it)))))
        warm["start"] = "session 6's fit"
        if warm["cost"] < best["cost"]:
            ctx, best = ctx2, warm
    print("\nbest start: %s, cost %.3f" % (best["start"], best["cost"]))
    show(best)
    json.dump(best, open(base_file, "w"), indent=1)
    out, rows = summary(ctx, ctx.u_of(best["plan"]), h=5e-4)
    print("\nthe same plan at a 0.5 ms step:")
    show(out)
    print("\nthe held-out checks:")
    held_out(ctx, ctx.u_of(best["plan"]))


if __name__ == "__main__":
    main()
