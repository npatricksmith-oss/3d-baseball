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
# the clock as measured (4.5, session 6): [4]'s bat rises through 236 deg/s at -170 ms; the model's must, within the
# tolerance; and no arm drive may rise above its holding level more than the electromechanical delay's range before it
FIRST_MOTION_T, FIRST_MOTION_RATE, FIRST_MOTION_TOL = -0.170, 236.0, 0.015
EMD_MIN, EMD_MAX = 0.010, 0.070
ARMS_EARLIEST = FIRST_MOTION_T - EMD_MAX
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
# the arms (4.5, session 6; note 9). Shoulders by direction: Silva 2006's junior tennis players, per 100 kg, mean + 2 SD,
# x 92 kg: the rear shoulder acts in horizontal adduction, the lead in horizontal abduction
SHOULDER_HADD = (90.9 + 2 * 21.3) / 100.0 * 92.0      # 123 N m
SHOULDER_HABD = (69.8 + 2 * 15.0) / 100.0 * 92.0      # 92 N m
ELBOW_EXT = (60.5 + 2 * 6.2) * (92.0 * 1.84) / (81.6 * 1.778)   # Holzbaur 2007, 5 men, isometric: 85 N m
# one wrist's ulnar deviation, two readings (4.5): Peleg 2025 (a Biodex handle at 1.57 rad/s) and Delp 1996 (isometric)
WRIST = {"peleg": (26.5 + 2 * 6.6) * (92.0 * 1.84) / (74.3 * 1.763), "delp": 9.5 + 2 * 2.2}    # 51 and 13.9 N m
WRIST_READING = "peleg"
REF_WH = 92.0 * E.G * 1.84
# the undriven freedoms' limits (4.5, session 7; notes 8 and 9), each for 92 kg, 1.84 m
_HOLZ = (92.0 * 1.84) / (81.6 * 1.778)                 # Holzbaur 2007: 5 men, isometric
_DODDS = (92.0 * 1.84) / (89.6 * 1.834)                # Dodds 2020: 9 position players at 3.14 rad/s
SHOULDER_RAISE = (74.4 + 2 * 10.8) * _HOLZ             # abduction: 112 N m
SHOULDER_LOWER = (93.7 + 2 * 11.3) * _HOLZ             # adduction: 136
SHOULDER_IROT = (37.2 + 2 * 15.4) * _DODDS             # internal rotation: 70
SHOULDER_EROT = (23.2 + 2 * 7.4) * _DODDS              # external rotation: 39
ELBOW_FLEX = (79.5 + 2 * 8.1) * _HOLZ                  # 112
FOREARM_PRON, FOREARM_SUP = 17.35, 18.30               # Kentel 2011's one player, as measured (no sample exists)
WRIST_FLEX = (25.6 + 2 * 7.0) * _HOLZ                  # 46 (the SD printed "7", the rest cut)
WRIST_EXT = (14.0 + 2 * 3.4) * _HOLZ                   # 24
TRUNK_BACK_REF = (131.83 + 2 * 27.87) * (92.0 * 1.84) / (74.7 * 1.765)   # Bae 2012, the golfers' other direction: 241 N m
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
def _hyp(wmax, wc):
    """Kentel 2011's fitted curve for one elite tennis player: fraction of isometric torque at speed w."""
    return lambda w: (wc / wmax) * (wmax - w) / (wc + w)


def _arm_curve(wmax, wc, lead, join, speeds=(4.0, 5.24, 6.1, 9.0, 13.0)):
    """`lead` (measured points up to the speed `join`), then the player's curve as a ratio to its own value at
    the join, to zero at his top speed."""
    f = _hyp(wmax, wc)
    base = lead[-1][1]
    return tuple(lead) + tuple((w, base * f(w) / f(join)) for w in speeds if w > join) + ((wmax, 0.0),)


_SPEEDS0 = (1.0, 2.0, 3.0, 4.0, 5.0, 6.1, 9.0, 13.0)
CURVES.update({
    "sh_hadd": _arm_curve(18.05, 6.43, ((0.0, 1.0), (3.14, 1.0)), 3.14),                     # Silva: 100 % kept to 3.14 rad/s
    "sh_habd": _arm_curve(18.0, 6.93, ((0.0, 1.0), (1.05, 1.0), (3.14, 0.93)), 3.14),          # Silva: 93 %
    "sh_hadd_kentel": _arm_curve(18.05, 6.43, ((0.0, 1.0),), 0.0, _SPEEDS0),
    "sh_habd_kentel": _arm_curve(18.0, 6.93, ((0.0, 1.0),), 0.0, _SPEEDS0),
    "elbow_ext": _arm_curve(18.45, 5.55, ((0.0, 1.0),), 0.0, _SPEEDS0),
    "wrist_peleg": _arm_curve(18.02, 5.83, ((0.0, 1.0), (1.57, 1.0)), 1.57, (2.0, 3.0, 4.0, 5.0, 6.1, 9.0, 13.0)),
    "wrist_delp": _arm_curve(18.02, 5.83, ((0.0, 1.0),), 0.0, _SPEEDS0),
})
_IR53, _IR85 = 31.0 / 37.2, 31.0 / 37.2 * 37.14 / 50.66      # Dodds 3.14 -> 5.24 rad/s; Wilkin 5.24 -> 7.85
_ER53, _ER85 = 19.9 / 23.2, 19.9 / 23.2 * 17.68 / 30.16
_TB = (117.67 / 131.83, 111.04 / 131.83, 111.04 / 131.83 * 50.0 / 65.0)   # Bae's other direction; Gifford's other row
CURVES.update({
    "sh_raise": _arm_curve(20.74, 6.70, ((0.0, 1.0),), 0.0, _SPEEDS0),          # Kentel's shoulder flexion
    "sh_lower": _arm_curve(31.93, 15.02, ((0.0, 1.0),), 0.0, _SPEEDS0 + (18.0, 24.0)),   # his shoulder extension
    "sh_irot": _arm_curve(19.47, 9.71, ((0.0, 1.0), (3.14, 1.0), (5.24, _IR53), (7.85, _IR85)), 7.85, (9.0, 13.0)),
    "sh_erot": _arm_curve(18.00, 7.20, ((0.0, 1.0), (3.14, 1.0), (5.24, _ER53), (7.85, _ER85)), 7.85, (9.0, 13.0)),
    "elbow_flex": _arm_curve(18.10, 5.43, ((0.0, 1.0),), 0.0, _SPEEDS0),
    "pron": _arm_curve(27.88, 9.71, ((0.0, 1.0),), 0.0, _SPEEDS0 + (18.0, 22.0)),
    "sup": _arm_curve(35.45, 16.90, ((0.0, 1.0),), 0.0, _SPEEDS0 + (18.0, 24.0, 30.0)),
    "wrist_flex": _arm_curve(21.40, 10.70, ((0.0, 1.0),), 0.0, _SPEEDS0 + (17.0,)),
    "wrist_ext": _arm_curve(18.08, 7.78, ((0.0, 1.0),), 0.0, _SPEEDS0),
    "sh_irot_kentel": _arm_curve(19.47, 9.71, ((0.0, 1.0),), 0.0, _SPEEDS0),
    "sh_erot_kentel": _arm_curve(18.00, 7.20, ((0.0, 1.0),), 0.0, _SPEEDS0),
    "trunk_back": ((0.0, 1.0), (0.52, 1.0), (1.05, _TB[0]), (2.09, _TB[1]), (4.19, _TB[2]),
                   (4.19 + _TB[2] * 2.10 / (_TB[1] - _TB[2]), 0.0)),
})
ARM_ECC = 1.4                 # Kentel: an arm muscle being stretched rises to 1.4 x (the legs and trunk: Wilson's 1.5)
NEW_SHOULDER_CURVES = "samples"   # "kentel": the shoulder's rotation on the one player's curve alone (a range end)
SHOULDER_CURVES = "silva"     # "kentel": the one player's curve alone from zero speed (a range end)
HIP_SIDE_CURVE = "hip_ext"    # [E] hip abduction, adduction and rotators fall with speed as the hip's extensors do; None = the first law
# tendon stretch at full strength, rad: Wilson 2006's tabulated full strength over series stiffness (note 8 section 3.3)
SEC_STRETCH = {"hip_ext": 480.0 / 5092.0, "knee_ext": 328.0 / 857.0, "hip_flex": 289.0 / 496.0, "ankle_pf": 283.0 / 767.0,
               # the arms: Kentel 2011's printed full strength over series stiffness (note 9 section 1)
               "sh_hadd": 97.48 / 1968.5, "sh_habd": 71.90 / 785.9, "elbow_ext": 51.95 / 280.3, "wrist": 41.44 / 23.3,
               # session 7's channels: Kentel's shoulder flexion, extension, internal rotation and elbow flexion rows
               "sh_raise": 56.29 / 1386.9, "sh_lower": 119.13 / 2366.9, "sh_irot": 42.67 / 603.1, "elbow_flex": 75.82 / 651.5}
TENDONS = True             # False: no series springs at the legs (a sensitivity case)
ARM_TENDONS = {"sh_hadd": True, "sh_habd": True, "elbow_ext": True, "wrist": False,   # 4.5: the wrists' printed value is doubted
               "sh_raise": True, "sh_lower": True, "sh_irot": True, "elbow_flex": True}
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
         "core": TRUNK_REF * k, "shoulder_L": SHOULDER_HABD * k, "shoulder_R": SHOULDER_HADD * k,
         "elbow_ext_L": ELBOW_EXT * k, "elbow_ext_R": ELBOW_EXT * k,
         "wrist_L": WRIST[WRIST_READING] * k, "wrist_R": WRIST[WRIST_READING] * k, "core_back": TRUNK_BACK_REF * k}
    for sd in "LR":
        b.update({"sh_raise_" + sd: SHOULDER_RAISE * k, "sh_lower_" + sd: SHOULDER_LOWER * k, "sh_irot_" + sd: SHOULDER_IROT * k,
                  "sh_erot_" + sd: SHOULDER_EROT * k, "elbow_flex_" + sd: ELBOW_FLEX * k, "pron_" + sd: FOREARM_PRON, "sup_" + sd: FOREARM_SUP,
                  "wrist_flex_" + sd: WRIST_FLEX * k, "wrist_ext_" + sd: WRIST_EXT * k})
    for n, f in BOUND_SCALE.items():
        b[n] *= f
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
    "shoulder_L": ("trunk", "uarm_L", (0, 0, 1), "shoulder", "sh_habd", "sh_habd"),  # the lead arm swept out toward the pitcher: horizontal abduction
    "shoulder_R": ("trunk", "uarm_R", (0, 0, 1), "shoulder", "sh_hadd", "sh_hadd"),  # the rear arm brought forward and round: horizontal adduction
    "elbow_ext_L": ("uarm_L", "farm_L", (0, -1, 0), "elbow", "elbow_ext", "elbow_ext"),
    "elbow_ext_R": ("uarm_R", "farm_R", (0, -1, 0), "elbow", "elbow_ext", "elbow_ext"),
    "wrist_L": ("bat", "farm_L", None, "wrist", "wrist", "wrist"),         # axis: the bat's own y, signed in add_drives so it uncocks (ulnar deviation)
    "wrist_R": ("bat", "farm_R", None, "wrist", "wrist", "wrist"),
}


# ---- session 7's channels (4.5) ---------------------------------------------------------------
# A seventh field, where present, is a direction fixed in the CHILD: the drive then acts about the floating axis
# along (axis in the parent) x (that direction) (engine.actuator's axis_b). The upper arm's own z runs from the
# elbow up to the shoulder and the forearm's from the wrist up to the elbow (body.pose), so:
# - raising the arm turns the upper arm's z away from the trunk's +z (the arm, which points along -z of itself,
#   toward the trunk's upper end): the floating axis of (trunk +z, upper arm +z). Lowering: of (trunk -z, arm +z).
# - internal rotation turns the RIGHT upper arm about its own +z (a bent forearm pointing forward swings across
#   the front of the body) and the LEFT about its own -z. These and the forearm's rotation act about an axis of
#   the child, so, as the hips do, the segment is the "parent" and the axis below is the torque on the OTHER body
#   (the reaction of the one meant: the right arm's internal rotation is the trunk driven about the arm's -z).
# - pronation turns the RIGHT forearm about its own +z (the thumb from outside, through the front, to inside)
#   and the LEFT about its own -z.
# - wrist flexion and extension act about the forearm's x, the wrist joint's first axis; which way is flexion is
#   signed from the placed posture in add_drives (both hands hold the bat thumb toward the barrel).
for _sd, _sg in (("L", 1.0), ("R", -1.0)):
    CHANNELS.update({
        "sh_raise_" + _sd: ("trunk", "uarm_" + _sd, (0, 0, 1), "shoulder", "sh_raise", "sh_raise", (0, 0, 1)),
        "sh_lower_" + _sd: ("trunk", "uarm_" + _sd, (0, 0, -1), "shoulder", "sh_lower", "sh_lower", (0, 0, 1)),
        "sh_irot_" + _sd: ("uarm_" + _sd, "trunk", (0, 0, _sg), "shoulder", "sh_irot", "sh_irot"),
        "sh_erot_" + _sd: ("uarm_" + _sd, "trunk", (0, 0, -_sg), "shoulder", "sh_erot", None),
        "elbow_flex_" + _sd: ("uarm_" + _sd, "farm_" + _sd, (0, 1, 0), "elbow", "elbow_flex", "elbow_flex"),
        "pron_" + _sd: ("farm_" + _sd, "uarm_" + _sd, (0, 0, _sg), "elbow", "pron", None),
        "sup_" + _sd: ("farm_" + _sd, "uarm_" + _sd, (0, 0, -_sg), "elbow", "sup", None),
        "wrist_flex_" + _sd: ("farm_" + _sd, "bat", "flex", "wrist", "wrist_flex", None),
        "wrist_ext_" + _sd: ("farm_" + _sd, "bat", "ext", "wrist", "wrist_ext", None),
    })
CHANNELS["core_back"] = ("pelvis", "trunk", (0, 0, -1), "core", "trunk_back", None)   # the trunk's rotators away from the pitcher: its brake
NEW_ARM_CHANNELS = tuple(n + sd for sd in "LR" for n in ("sh_raise_", "sh_lower_", "sh_irot_", "sh_erot_", "elbow_flex_", "pron_", "sup_",
                                                          "wrist_flex_", "wrist_ext_"))
BOUND_SCALE = {}           # {channel: factor} on a bound, for a rerun at a range end; empty in the design


def curve_of(ch):
    """the name in CURVES of channel ch's strength-at-speed curve as the switches stand, or None (the first law)."""
    curve = CHANNELS[ch][4]
    if curve == "side":
        return HIP_SIDE_CURVE
    if curve == "wrist":
        return "wrist_" + WRIST_READING
    if curve in ("sh_hadd", "sh_habd") and SHOULDER_CURVES == "kentel":
        return curve + "_kentel"
    if curve in ("sh_irot", "sh_erot") and NEW_SHOULDER_CURVES == "kentel":
        return curve + "_kentel"
    return curve


def tendon_of(ch):
    """the name in SEC_STRETCH of channel ch's tendon spring as the switches stand, or None."""
    t = CHANNELS[ch][5]
    if t is None:
        return None
    if t in ARM_TENDONS:
        return t if ARM_TENDONS[t] else None
    return t if TENDONS else None


def curve_ecc(curve):
    return ARM_ECC if curve and curve.startswith(("sh_", "elbow", "wrist", "pron", "sup")) else E.ECC
ARM_CHANNELS = ("shoulder_L", "shoulder_R", "elbow_ext_L", "elbow_ext_R", "wrist_L", "wrist_R")   # session 6's; with NEW_ARM_CHANNELS, the arms'
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
        if n in ARM_CHANNELS + NEW_ARM_CHANNELS and t_on < ARMS_EARLIEST - 1e-12:
            bad.append("%s: rises from %.0f ms, more than the electromechanical delay before the bat's first motion (%.0f)" % (n, t_on * 1e3, ARMS_EARLIEST * 1e3))
    return bad


def add_drives(mdl, st, plan, held=False):
    """add the plan's channels {name: (T, onset, rise, off[, starting level])} to the model, each with its
    strength-at-speed curve and, at the leg joints that have one, its tendon spring (4.5). Returns the state
    with the channels' activations at their starting levels. The placed state is only read to sign the
    wrists' axis: the drive must reduce the lag (uncock). Call sec_start afterwards for the springs.
    held: every channel's starting level is held without net torque until its onset (engine.actuator, 4.11)."""
    s, b = mdl.sys, mdl.b
    nold = len(s.acts)
    s.acts = []
    s.fvtabs = []
    s.fvecc = []
    tab_of = {}
    bd = bounds(mdl.H, mdl.M)
    for n, v in plan.items():
        T, t_on, rise, t_off = v[:4]
        a0 = v[4] if len(v) > 4 else 0.0
        pa, ch, ax, grp, curve, tendon = CHANNELS[n][:6]
        axb = CHANNELS[n][6] if len(CHANNELS[n]) > 6 else None
        if ax in ("flex", "ext"):
            # the wrist's first axis is the forearm's x; in a hand closed round the handle it lies along the handle.
            # Thumb toward the barrel, flexion turns the bat (relative to the forearm) about the handle's knob-to-barrel
            # direction for the right (rear) hand and against it for the left (4.5)
            along = E.dot(mdl.axis(st, pa, (1, 0, 0)), mdl.axis(st, "bat", (0, 0, 1)))
            sgn = (1.0 if along > 0 else -1.0) * (1.0 if n[-1] == "R" else -1.0) * (1.0 if ax == "flex" else -1.0)
            ax = (sgn, 0, 0)
        if ax is None:
            sd = n[-1]
            bz = mdl.axis(st, "bat", (0, 0, 1)); by = mdl.axis(st, "bat", (0, 1, 0))
            arm = E.unit(E.sub(mdl.joint_pos(st, "wrist_" + sd), mdl.joint_pos(st, "shoulder_" + sd)))
            # turning the bat by +d about its y moves its axis by d (by x bz); the lag falls if that is toward the arm
            toward = E.dot(E.cross(by, bz), arm)
            ax = (0, -1, 0) if toward > 0 else (0, 1, 0)      # the torque on the forearm is the reaction of the bat's
            if getattr(mdl, "grip", "weld") == "hands":
                # the hands' body (4.11): the wrist's tilt AWAY from the thumb, about the hand's own p, by anatomy and not
                # by which way the lag falls at the start: the right forearm turns about -p against its hand, the left about +p
                ax = E.scale(mdl.palm[sd], 1.0 if sd == "L" else -1.0)
        curve, tendon = curve_of(n), tendon_of(n)
        tab = -1
        if curve is not None:
            if curve not in tab_of:
                tab_of[curve] = len(s.fvtabs)
                s.fvtabs.append(tuple(CURVES[curve]))
                s.fvecc.append(curve_ecc(curve))
            tab = tab_of[curve]
        ksec = bd[n] / SEC_STRETCH[tendon] if (tendon and tab >= 0) else 0.0
        s.actuator(n, b[pa], b[ch], ax, T, W_TOP[grp], t_on, rise, t_off, a0, tab, ksec, axb, held)
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


def hold_response(mdl, st, t0, aux, names, bd):
    """{channel: the change of the hold-still torques (solve_tone's, three per joint in JOINTS' order) that cancels that
    channel held at its full bound from the start}. The body's accelerations at the start are linear in the hold-still
    torques and in each channel's starting torque (strength x starting level, through its strength-at-speed curve and
    its tendon spring as sec_start loads it), so the answer for any plan is the sum of these times strength x level /
    bound. With it a muscle tensed beforehand only holds: it moves nothing until it fires or its joint moves
    (body-motion 4.10; session 8's review, P2-4). st: the placed state with every channel at zero; the model's
    tones are left as they were."""
    s = mdl.sys
    nb, oa = s.nb, 13 * s.nb
    tones = list(s.tones)

    def acc(state, ax):
        ds = s.deriv(t0, state, ax)
        return [ds[13 * i + 7 + c] for i in range(nb) for c in range(6)]
    ax0 = list(aux)
    s.anchors(st, ax0)
    s.sec_start(st, ax0)
    base = acc(st, ax0)
    cols = []
    for j in JOINTS:
        a, ra, bb, rb = mdl.jp[j]
        for c in range(3):
            s.tones = tones + [(j, a, bb, E.E3[c])]
            cols.append([x - y for x, y in zip(acc(st, ax0), base)])
    s.tones = tones
    n = len(cols)
    A = [[sum(x * y for x, y in zip(cols[i], cols[k])) for k in range(i + 1)] for i in range(n)]
    ridge = TONE_RIDGE * max(A[i][i] for i in range(n))
    for i in range(n):
        A[i][i] += ridge
    index = {a[0]: j for j, a in enumerate(s.acts)}
    out = {}
    for nm in names:
        j = index[nm]
        keep = (s.acts[j][4], s.acts[j][9])
        s.acts[j][4], s.acts[j][9] = bd[nm], 1.0
        st2 = list(st)
        st2[oa + j] = 1.0
        ax2 = list(aux)
        s.anchors(st2, ax2)
        s.sec_start(st2, ax2)
        resp = [x - y for x, y in zip(acc(st2, ax2), base)]
        s.acts[j][4], s.acts[j][9] = keep
        out[nm] = E.chol_solve([row[:] for row in A], [-sum(x * y for x, y in zip(cols[i], resp)) for i in range(n)])
    return out


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
