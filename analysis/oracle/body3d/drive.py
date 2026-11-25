"""The drives of the three-dimensional batter (docs/body-motion.md 5.1, 4.3): which joint groups get a
torque generator, the sourced bounds on their strengths (the design's rules), the muscles' level at
the start (worked back from the placed posture), and a run that records what [4] measured.

Nothing here knows where the swing should end.
"""
import math

from . import engine as E
from . import body as B

D2R = B.D2R

# ---- the design's rules (body-motion 4.3) and, since PLAN D34, the limits of body-motion 4.5 -------
# Every number below is body-motion 4.5's, written there with its source before any refit. Nothing here is
# chosen by what a swing does.
ARMS_EARLIEST = -0.170     # s: shoulders, elbows and wrists may not start before [4]'s first move
RISE_MIN = 0.050           # s [E]: every rise at least this long
# full strength: the source's mean + 2 SD, x weight x height (Anderson 2007 C1 and its SD; note 5 section 1, table)
C1 = {"hip_ext": (0.161, 0.049), "hip_flex": (0.113, 0.025), "knee_ext": (0.163, 0.040), "knee_flex": (0.087, 0.015),
      "ankle_pf": (0.095, 0.022)}
SD_COUNT = 2.0
HIP_ROT = 78.0             # [E] one hip's rotators, no source (4.3, 4.5)
# hip abduction: college pitchers' isometric trail-hip abduction 109 +- 16.4 N m at 96.1 kg, 1.89 m (note 5 section 2,
# second hand, hand-held dynamometer); adduction [E] equal to it (4.5)
HIP_ABD_REF, HIP_ABD_REF_WH = 109.0 + 2 * 16.4, 96.1 * E.G * 1.89
HIP_ADD_OVER_ABD = 1.0
TRUNK_REF = (140.6 + 2 * 30.9) * (92.0 * 1.84) / (74.7 * 1.765)   # Bae 2012 golfers at 0.52 rad/s, mean + 2 SD, scaled: 260 N m
SHOULDER = 100.0           # [E] one shoulder (4.3)
ELBOW_EXT = 70.0           # [E] one elbow's extension (4.4)
WRIST_ULNAR, WRIST_RADIAL = 9.5, 11.0   # one wrist, Delp 1996 (abstract only)
REF_WH = 92.0 * E.G * 1.84
# strength at speed: (rad/s, fraction of full strength), straight lines between (4.5's table, with its sources)
CURVES = {
    "hip_ext": ((0.0, 1.0), (1.05, 0.83), (3.14, 0.83 * 0.84), (4.19, 0.83 * 0.795), (14.2, 0.0)),
    "hip_ext_anderson": ((0.0, 1.0), (1.578, 0.75), (3.19, 0.50), (6.4, 0.0)),
    "hip_flex": ((0.0, 1.0), (2.095, 0.75), (4.267, 0.50), (8.6, 0.0)),
    "knee_ext": ((0.0, 1.0), (1.517, 0.75), (3.952, 0.50), (5.24, 0.82 * 0.525), (13.4, 0.0)),
    "knee_flex": ((0.0, 1.0), (2.008, 0.75), (5.233, 0.50), (11.7, 0.0)),
    "ankle_pf": ((0.0, 1.0), (0.987, 0.75), (3.558, 0.50), (8.7, 0.0)),
    "trunk": ((0.0, 1.0), (0.52, 1.0), (1.05, 127.4 / 140.6), (2.09, 117.0 / 140.6), (4.19, 117.0 / 140.6 * 53.5 / 66.3), (12.9, 0.0)),
}
HIP_SIDE_CURVE = "hip_ext"    # [E] hip abduction, adduction and rotators fall with speed as the hip's extensors do; None = the first law
# tendon stretch at full strength, rad: Wilson 2006's tabulated full strength over series stiffness (note 8 section 3.3)
SEC_STRETCH = {"hip_ext": 480.0 / 5092.0, "knee_ext": 328.0 / 857.0, "hip_flex": 289.0 / 496.0, "ankle_pf": 283.0 / 767.0}
TENDONS = True             # False: no series springs (a sensitivity case)
# the first law's top speeds W, rad/s [E] (5.1), still used by the arms and wherever a curve is None
W_TOP = {"hip": 20.0, "knee": 20.0, "ankle": 20.0, "core": 25.0, "shoulder": 30.0, "elbow": 35.0, "wrist": 40.0}
# intrinsic joint damping [E]: N m s/rad on every joint, capped; stiffness none
JOINT_DAMP = 1.0
JOINT_DAMP_CAP = 30.0
TONE_RIDGE = 1e-8          # relative to the largest diagonal of the normal equations


def bounds(H, M):
    """upper bound on each channel's full strength T (N m) for a batter of stature H and mass M (4.5).
    Scaling by weight x height is [E] (7.1)."""
    wh = M * E.G * H
    k = wh / REF_WH
    c = lambda n: (C1[n][0] + SD_COUNT * C1[n][1]) * wh
    abd = HIP_ABD_REF * wh / HIP_ABD_REF_WH
    b = {"hip_ext_R": c("hip_ext"), "hip_ext_L": c("hip_ext"), "hip_flex_L": c("hip_flex"),
         "hip_rot_R": HIP_ROT * k, "hip_rot_L": HIP_ROT * k,
         "hip_abd_R": abd, "hip_abd_L": abd, "hip_add_R": HIP_ADD_OVER_ABD * abd, "hip_add_L": HIP_ADD_OVER_ABD * abd,
         "knee_ext_L": c("knee_ext"), "knee_ext_R": c("knee_ext"), "knee_flex_L": c("knee_flex"), "knee_flex_R": c("knee_flex"),
         "ankle_pf_R": c("ankle_pf"), "ankle_pf_L": c("ankle_pf"),
         "core": TRUNK_REF * k, "shoulder_L": SHOULDER * k, "shoulder_R": SHOULDER * k,
         "elbow_ext_L": ELBOW_EXT * k, "elbow_ext_R": ELBOW_EXT * k,
         "wrist_L": WRIST_ULNAR * k, "wrist_R": WRIST_ULNAR * k}
    return b


# channel -> (parent body, child body, axis in the parent's frame, first-law speed group, curve, tendon).
# The torque acts on the child about +axis, its reaction on the parent. The hip channels act about the THIGH's
# own axes (its y side-to-side, x front-to-back, z long; as place() builds them), the way hip strength is
# tested: the thigh is the parent there and the pelvis the child, so each axis below is the torque on the pelvis.
CHANNELS = {
    "hip_ext_R": ("thigh_R", "pelvis", (0, -1, 0), "hip", "hip_ext", "hip_ext"),     # the thigh driven back under the pelvis
    "hip_ext_L": ("thigh_L", "pelvis", (0, -1, 0), "hip", "hip_ext", "hip_ext"),     # the lead hip's extensors: the block
    "hip_flex_L": ("thigh_L", "pelvis", (0, 1, 0), "hip", "hip_flex", "hip_flex"),   # the lead thigh driven forward
    "hip_rot_R": ("thigh_R", "pelvis", (0, 0, 1), "hip", "side", None),              # rotators: the pelvis turned toward the pitcher
    "hip_rot_L": ("thigh_L", "pelvis", (0, 0, 1), "hip", "side", None),
    "hip_abd_L": ("thigh_L", "pelvis", (-1, 0, 0), "hip", "side", None), "hip_add_L": ("thigh_L", "pelvis", (1, 0, 0), "hip", "side", None),
    "hip_abd_R": ("thigh_R", "pelvis", (1, 0, 0), "hip", "side", None), "hip_add_R": ("thigh_R", "pelvis", (-1, 0, 0), "hip", "side", None),
    "knee_ext_L": ("thigh_L", "shank_L", (0, -1, 0), "knee", "knee_ext", "knee_ext"),  # the block
    "knee_ext_R": ("thigh_R", "shank_R", (0, -1, 0), "knee", "knee_ext", "knee_ext"),  # the push
    "knee_flex_L": ("thigh_L", "shank_L", (0, 1, 0), "knee", "knee_flex", None),
    "knee_flex_R": ("thigh_R", "shank_R", (0, 1, 0), "knee", "knee_flex", None),
    "ankle_pf_R": ("shank_R", "foot_R", (0, 1, 0), "ankle", "ankle_pf", "ankle_pf"),   # plantar flexion: the push off the rear foot
    "ankle_pf_L": ("shank_L", "foot_L", (0, 1, 0), "ankle", "ankle_pf", "ankle_pf"),
    "core": ("pelvis", "trunk", (0, 0, 1), "core", "trunk", None),
    "shoulder_L": ("trunk", "uarm_L", (0, 0, 1), "shoulder", None, None),  # the lead arm swept toward the pitcher about the trunk's axis
    "shoulder_R": ("trunk", "uarm_R", (0, 0, 1), "shoulder", None, None),  # the rear arm brought forward and round
    "elbow_ext_L": ("uarm_L", "farm_L", (0, -1, 0), "elbow", None, None),
    "elbow_ext_R": ("uarm_R", "farm_R", (0, -1, 0), "elbow", None, None),
    "wrist_L": ("bat", "farm_L", None, "wrist", None, None),               # axis: the bat's own y, signed in add_drives so it uncocks
    "wrist_R": ("bat", "farm_R", None, "wrist", None, None),
}
ARM_CHANNELS = ("shoulder_L", "shoulder_R", "elbow_ext_L", "elbow_ext_R", "wrist_L", "wrist_R")
JOINTS = ("lumbar", "hip_L", "hip_R", "knee_L", "knee_R", "ankle_L", "ankle_R",
          "shoulder_L", "shoulder_R", "elbow_L", "elbow_R", "wrist_L", "wrist_R")


def check_rules(mdl, plan):
    """the design's rules broken by a plan {channel: (T, t_on, t_rise, t_off)}; empty when it obeys them."""
    bad = []
    bd = bounds(mdl.H, mdl.M)
    for n, v in plan.items():
        T, t_on, rise, t_off = v[:4]
        if T > bd[n] * (1 + 1e-9):
            bad.append("%s: strength %.0f over its bound %.0f N m" % (n, T, bd[n]))
        if rise < RISE_MIN - 1e-12:
            bad.append("%s: rise %.0f ms under %.0f" % (n, rise * 1e3, RISE_MIN * 1e3))
        if n in ARM_CHANNELS and len(plan[n]) > 4 and plan[n][4] > 1e-12:
            bad.append("%s: tensed before the decision allows" % n)
        if n in ARM_CHANNELS and t_on < ARMS_EARLIEST - 1e-12:
            bad.append("%s: starts at %.0f ms, before the decision allows (%.0f)" % (n, t_on * 1e3, ARMS_EARLIEST * 1e3))
    return bad


def add_drives(mdl, st, plan):
    """add the plan's channels {name: (T, onset, rise, off[, starting level])} to the model, each with its
    strength-at-speed curve and, at the leg joints that have one, its tendon spring (4.5). Returns the state
    with the channels' activations at their starting levels. The placed state is only read to sign the
    wrists' axis: the drive must reduce the lag (uncock). Call sec_start afterwards for the springs."""
    s, b = mdl.sys, mdl.b
    nold = len(s.acts)
    s.acts = []
    s.fvtabs = []
    tab_of = {}
    bd = bounds(mdl.H, mdl.M)
    for n, v in plan.items():
        T, t_on, rise, t_off = v[:4]
        a0 = v[4] if len(v) > 4 else 0.0
        pa, ch, ax, grp, curve, tendon = CHANNELS[n]
        if ax is None:
            sd = n[-1]
            bz = mdl.axis(st, "bat", (0, 0, 1)); by = mdl.axis(st, "bat", (0, 1, 0))
            arm = E.unit(E.sub(mdl.joint_pos(st, "wrist_" + sd), mdl.joint_pos(st, "shoulder_" + sd)))
            # turning the bat by +d about its y moves its axis by d (by x bz); the lag falls if that is toward the arm
            toward = E.dot(E.cross(by, bz), arm)
            ax = (0, -1, 0) if toward > 0 else (0, 1, 0)      # the torque on the forearm is the reaction of the bat's
        if curve == "side":
            curve = HIP_SIDE_CURVE
        tab = -1
        if curve is not None:
            if curve not in tab_of:
                tab_of[curve] = len(s.fvtabs)
                s.fvtabs.append(tuple(CURVES[curve]))
            tab = tab_of[curve]
        ksec = bd[n] / SEC_STRETCH[tendon] if (tendon and TENDONS and tab >= 0) else 0.0
        s.actuator(n, b[pa], b[ch], ax, T, W_TOP[grp], t_on, rise, t_off, a0, tab, ksec)
    nb13 = 13 * s.nb
    return st[:nb13] + [a[9] for a in s.acts] + st[nb13 + nold:]


def add_damping(mdl, st, damp=JOINT_DAMP, cap=JOINT_DAMP_CAP):
    """intrinsic joint damping on every joint [E]; no stiffness, so nothing pulls toward a posture."""
    s = mdl.sys
    s.holds = []
    for j in JOINTS:
        a, ra, bb, rb = mdl.jp[j]
        qa = s.pose(st, a)[1]; qb = s.pose(st, bb)[1]
        s.hold(j, a, bb, E.qmul(E.qconj(qa), qb), (0.0, 0.0, 0.0), (damp, damp, damp), cap)


def solve_tone(mdl, st, t0, aux=None):
    """The joint torques the placed body carries at the start: the constant torques (one vector per joint,
    fixed in the parent's frame) that bring every segment's acceleration at t0 as near zero as the ground
    forces allow (least squares, a small ridge picking the smallest torques among equals). It is the
    muscles' level in the measured posture, held constant afterwards [E: a stand-in for the postural
    program of body-motion 5.3, which is a later session's]. Returns {joint: torque vector} and the
    remaining accelerations (largest linear m/s^2, largest angular rad/s^2)."""
    s = mdl.sys
    aux = s.new_aux() if aux is None else aux
    s.anchors(st, aux)
    s.tones = []
    nb = s.nb

    def acc():
        ds = s.deriv(t0, st, aux)
        return [ds[13 * i + 7 + c] for i in range(nb) for c in range(6)]
    a0 = acc()
    cols = []
    for j in JOINTS:
        a, ra, bb, rb = mdl.jp[j]
        for c in range(3):
            s.tones = [(j, a, bb, E.E3[c])]
            ak = acc()
            cols.append([x - y for x, y in zip(ak, a0)])
    s.tones = []
    n = len(cols)
    A = [[sum(x * y for x, y in zip(cols[i], cols[k])) for k in range(i + 1)] for i in range(n)]
    ridge = TONE_RIDGE * max(A[i][i] for i in range(n))
    for i in range(n):
        A[i][i] += ridge
    rhs = [-sum(x * y for x, y in zip(cols[i], a0)) for i in range(n)]
    tau = E.chol_solve(A, rhs)
    out = {}
    for k, j in enumerate(JOINTS):
        a, ra, bb, rb = mdl.jp[j]
        out[j] = (tau[3 * k], tau[3 * k + 1], tau[3 * k + 2])
        s.tone(j, a, bb, out[j])
    rest = acc()
    lin = max(abs(rest[6 * i + c]) for i in range(nb) for c in range(3))
    ang = max(abs(rest[6 * i + 3 + c]) for i in range(nb) for c in range(3))
    return out, (lin, ang)


def simulate(mdl, st, t0, t1, h=1e-3, every=5, on_step=None):
    """integrate the placed, driven body and record what [4] measured every `every` steps.
    Returns (rows, final state); a row is (t, observables, {foot: ground force N})."""
    s = mdl.sys
    aux = s.new_aux()
    s.anchors(st, aux)
    rows = []
    prev_az = [None]

    def record(t, state):
        s.diag = {}
        s.deriv(t, state, aux)
        o = mdl.observe(state)
        if prev_az[0] is not None:
            while o["bat_az"] - prev_az[0] > 180.0:
                o["bat_az"] -= 360.0
            while o["bat_az"] - prev_az[0] < -180.0:
                o["bat_az"] += 360.0
        elif o["bat_az"] > 0.0:
            o["bat_az"] -= 360.0            # [4]'s bat starts near -180 to -230
        prev_az[0] = o["bat_az"]
        o["stops"] = dict(s.diag.get("stop", {}))
        o["act"] = dict(s.diag.get("act", {}))
        rows.append((t, o, mdl.grf(s.diag)))
        s.diag = None
    record(t0, st)
    n = int(round((t1 - t0) / h))
    for i in range(n):
        st = s.step(t0 + i * h, st, aux, h)
        if on_step is not None:
            on_step(t0 + (i + 1) * h, st)
        if (i + 1) % every == 0:
            record(t0 + (i + 1) * h, st)
    return rows, st
