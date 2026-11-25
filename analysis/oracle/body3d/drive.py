"""The drives of the three-dimensional batter (docs/body-motion.md 5.1, 4.3): which joint groups get a
torque generator, the sourced bounds on their strengths (the design's rules), the muscles' level at
the start (worked back from the placed posture), and a run that records what [4] measured.

Nothing here knows where the swing should end.
"""
import math

from . import engine as E
from . import body as B

D2R = B.D2R

# ---- the design's rules (body-motion 4.3), set before any 3D fit was run ---------------------------
ARMS_EARLIEST = -0.170     # s: shoulders, elbows and wrists may not start before [4]'s first move
RISE_MIN = 0.050           # s [E]: every rise at least this long
ATHLETE = 1.3              # [E] professional over recreationally active adults (4.3; no source)
# strength bounds, N m. WH = weight x height (N m). Anderson 2007's C1 (note 5 section 1, full text) for the legs.
C1_HIP_EXT, C1_HIP_FLEX, C1_KNEE_EXT, C1_ANKLE_PF = 0.161, 0.113, 0.163, 0.095
HIP_ROT = 60.0             # [E] one hip's rotators, 40-60 N m, no source (4.3)
CORE_REF = 288.0           # golfers' mean + 2 SD scaled to 92 kg, 1.84 m, over the f-v factor at the tested speed (4.3)
SHOULDER = 100.0           # [E] one shoulder (4.3: pitchers' internal rotation 66-80 measured)
ELBOW_EXT = 70.0           # [E] one elbow's extension, range 50-100; note 5 found no N m value
WRIST_ULNAR, WRIST_RADIAL = 9.5, 11.0   # one wrist, Delp 1996 (abstract only): uncocking, holding the cock
REF_WH = 92.0 * E.G * 1.84
# top speeds W, rad/s [E] (5.1): legs 20, core 25, shoulder 30, wrist 40; elbow and knee assumed
W_TOP = {"hip": 20.0, "knee": 20.0, "ankle": 20.0, "core": 25.0, "shoulder": 30.0, "elbow": 35.0, "wrist": 40.0}
# intrinsic joint damping [E]: N m s/rad on every joint, capped; stiffness none (a posture reference is next session's)
JOINT_DAMP = 1.0
JOINT_DAMP_CAP = 30.0
TONE_RIDGE = 1e-8          # relative to the largest diagonal of the normal equations


def bounds(H, M):
    """upper bound on each channel's isometric strength T (N m) for a batter of stature H and mass M.
    Scaling by weight x height is [E] (7.1)."""
    wh = M * E.G * H
    k = wh / REF_WH
    b = {"hip_ext_R": C1_HIP_EXT * wh * ATHLETE, "hip_flex_L": C1_HIP_FLEX * wh * ATHLETE,
         "hip_ext_L": C1_HIP_EXT * wh * ATHLETE,
         "ankle_pf_R": C1_ANKLE_PF * wh * ATHLETE, "ankle_pf_L": C1_ANKLE_PF * wh * ATHLETE,
         "hip_rot_R": HIP_ROT * ATHLETE * k, "hip_rot_L": HIP_ROT * ATHLETE * k,
         "knee_ext_L": C1_KNEE_EXT * wh * ATHLETE, "knee_ext_R": C1_KNEE_EXT * wh * ATHLETE,
         "core": CORE_REF * k, "shoulder_L": SHOULDER * k, "shoulder_R": SHOULDER * k,
         "elbow_ext_L": ELBOW_EXT * k, "elbow_ext_R": ELBOW_EXT * k,
         "wrist_L": WRIST_ULNAR * k, "wrist_R": WRIST_ULNAR * k}
    return b


# channel -> (parent body, child body, axis in the parent's frame, top-speed group). The torque acts on the
# child about +axis, its reaction on the parent. Axes follow body.py's frames: the pelvis's and trunk's
# own x forward, y left, z up; a knee's or elbow's own +y is its flexion axis as place() builds them.
CHANNELS = {
    "hip_ext_R": ("pelvis", "thigh_R", (0, 1, 0), "hip"),      # the rear thigh driven back under the pelvis
    "hip_flex_L": ("pelvis", "thigh_L", (0, -1, 0), "hip"),    # the lead thigh driven forward
    "hip_ext_L": ("pelvis", "thigh_L", (0, 1, 0), "hip"),      # the lead hip's extensors: the block (4.1)
    "ankle_pf_R": ("shank_R", "foot_R", (0, 1, 0), "ankle"),   # plantar flexion: the push off the rear foot
    "ankle_pf_L": ("shank_L", "foot_L", (0, 1, 0), "ankle"),
    "hip_rot_R": ("pelvis", "thigh_R", (0, 0, -1), "hip"),     # rotators: the pelvis turned toward the pitcher over each thigh
    "hip_rot_L": ("pelvis", "thigh_L", (0, 0, -1), "hip"),
    "knee_ext_L": ("thigh_L", "shank_L", (0, -1, 0), "knee"),  # the block
    "knee_ext_R": ("thigh_R", "shank_R", (0, -1, 0), "knee"),  # the push
    "core": ("pelvis", "trunk", (0, 0, 1), "core"),
    "shoulder_L": ("trunk", "uarm_L", (0, 0, 1), "shoulder"),  # the lead arm swept toward the pitcher about the trunk's axis
    "shoulder_R": ("trunk", "uarm_R", (0, 0, 1), "shoulder"),  # the rear arm brought forward and round
    "elbow_ext_L": ("uarm_L", "farm_L", (0, -1, 0), "elbow"),
    "elbow_ext_R": ("uarm_R", "farm_R", (0, -1, 0), "elbow"),
    "wrist_L": ("bat", "farm_L", None, "wrist"),               # axis: the bat's own y, signed in add_drives so it uncocks
    "wrist_R": ("bat", "farm_R", None, "wrist"),
}
ARM_CHANNELS = ("shoulder_L", "shoulder_R", "elbow_ext_L", "elbow_ext_R", "wrist_L", "wrist_R")
JOINTS = ("lumbar", "hip_L", "hip_R", "knee_L", "knee_R", "ankle_L", "ankle_R",
          "shoulder_L", "shoulder_R", "elbow_L", "elbow_R", "wrist_L", "wrist_R")


def check_rules(mdl, plan):
    """the design's rules broken by a plan {channel: (T, t_on, t_rise, t_off)}; empty when it obeys them."""
    bad = []
    bd = bounds(mdl.H, mdl.M)
    for n, (T, t_on, rise, t_off) in plan.items():
        if T > bd[n] * (1 + 1e-9):
            bad.append("%s: strength %.0f over its bound %.0f N m" % (n, T, bd[n]))
        if rise < RISE_MIN - 1e-12:
            bad.append("%s: rise %.0f ms under %.0f" % (n, rise * 1e3, RISE_MIN * 1e3))
        if n in ARM_CHANNELS and t_on < ARMS_EARLIEST - 1e-12:
            bad.append("%s: starts at %.0f ms, before the decision allows (%.0f)" % (n, t_on * 1e3, ARMS_EARLIEST * 1e3))
    return bad


def add_drives(mdl, st, plan):
    """add the plan's channels to the model and return the state with their activations (all 0) in it.
    The placed state is only read to sign the wrists' axis: the drive must reduce the lag (uncock)."""
    s, b = mdl.sys, mdl.b
    nold = len(s.acts)
    s.acts = []
    for n, (T, t_on, rise, t_off) in plan.items():
        pa, ch, ax, grp = CHANNELS[n]
        if ax is None:
            sd = n[-1]
            bz = mdl.axis(st, "bat", (0, 0, 1)); by = mdl.axis(st, "bat", (0, 1, 0))
            arm = E.unit(E.sub(mdl.joint_pos(st, "wrist_" + sd), mdl.joint_pos(st, "shoulder_" + sd)))
            # turning the bat by +d about its y moves its axis by d (by x bz); the lag falls if that is toward the arm
            toward = E.dot(E.cross(by, bz), arm)
            ax = (0, -1, 0) if toward > 0 else (0, 1, 0)      # the torque on the forearm is the reaction of the bat's
        s.actuator(n, b[pa], b[ch], ax, T, W_TOP[grp], t_on, rise, t_off)
    nb13 = 13 * s.nb
    return st[:nb13] + [0.0] * len(s.acts) + st[nb13 + nold:]


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
