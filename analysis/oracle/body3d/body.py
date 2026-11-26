"""The batter's three-dimensional body for the engine (docs/body-motion.md 4.1): thirteen rigid bodies
from de Leva's table by Height and Mass, their joints, the feet's contact points and the bat held by
both hands, and the placed posture from Fortenbaugh's [4] measured state.

Bodies: pelvis, trunk (middle + upper trunk + head), thigh/shank/foot x 2, upper arm/forearm x 2, and
the bat with both hands on it as ONE rigid body. A hand welded to the handle is the same rigid body as
the handle, so the weld of 4.1 is built in instead of being twelve more constraint rows; the loop of
the two arms closes through the two wrists. 13 bodies x 6 - 49 joint rows = 29 = 35 - 6 (4.1's count).

Own frames, in the anatomical standing pose: x forward, y to the body's left, z up, origin at the centre
of mass. The foot's x runs heel to toe. The bat's z runs knob to barrel; its -x points to the wrists.
"Lead" is the left side (a right-handed batter): L = lead, R = rear.

World: the sim frame (AGENTS.md): +X toward first base (a right-handed batter faces it), +Y toward the
pitcher, +Z up. [4]'s pelvis rotation and bat azimuth are then counter-clockwise angles about +Z from
+X (pelvis 0 = facing the plate, 90 = facing the pitcher; bat 0 = barrel toward the plate).
"""
import math
import os
import sys as _sys

from . import engine as E

_sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "research", "body"))
import common as C   # noqa: E402  (de Leva's checked rows, the bat, [4]'s curves)

D2R = math.pi / 180.0
if not os.path.isabs(C.DATA) and not os.path.isdir(C.DATA):      # run from another directory: the path is the repository's
    C.DATA = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", C.DATA)


def have_curves():
    """whether [4]'s digitised curves are on this machine (they are made under sim/target, untracked)."""
    return os.path.isfile(os.path.join(C.DATA, "pelvis_rot.csv"))

# ---- geometry de Leva's table does not give: all [E], with what would settle each -----------------
HIP_HALF = 0.049          # x H: half the distance between the hip joint centres [E: about 0.18 m in adult men; a pelvis scan]
SHOULDER_HALF = 0.112     # x H: shoulder joint from the trunk's axis [E: swing-model v3 5.2; Winter gives 0.129]
SHOULDER_UP = 0.80        # of the upper trunk's length above the xiphion [E]
ANKLE_H = 0.039           # x H: ankle joint above the sole [E: Winter 2009 fig. 4.1]
ANKLE_FWD = 0.25          # of the foot's length ahead of the heel [E]
BALL_FWD = 0.75           # of the foot's length: the ball of the foot (front contact points) [E]
SOLE_HALF_W = 0.04        # m, half the sole's width (body-motion 5.3) [E]
GRIP_L, GRIP_R = 0.08, 0.18   # m from the knob: lead (bottom) and rear (top) hand (body-motion 4.1) [E]
WRIST_TO_GRIP = 0.07      # m, wrist joint centre to the handle's axis [E: a closed fist]
HAND_R = 0.03             # m, a gripping hand's radius of gyration about the handle [E]
# contact [E]: stiff enough that a foot sinks under a millimetre, soft enough for a 1 ms RK4 step
KN, CN, KT, CT = 2.0e5, 300.0, 2.0e5, 300.0
MU = 0.8                  # [E] shoe on clay (the flat probe's value; no source, body-motion 5.1)
SH_ADD_MAX = 135.0        # deg [E] the shoulder's horizontal adduction, from the arm straight out to the side: the
#                           clinical normal range is 130-135 (AAOS); range tried 125-145. One-sided, as every stop.
FLEX_MAX = 150.0          # deg [E] knee's and elbow's full bend; 0 = straight, no hyperextension
STOP_K, STOP_C = 2000.0, 20.0   # [E] N m/rad, N m s/rad: a joint's end-of-range stop (one-sided, never pulls)
# posture [4] does not give, [E]
TRUNK_FLEX = 15.0         # deg, the trunk bent forward over the pelvis
TRUNK_TILT_IN_WORLD = False   # False: the forward bend turns with the trunk [E]. True: the lean stays toward the plate (trunk3d.py's case)
TRUNK_SIDE = 0.0          # deg [E], the trunk bent sideways (+: toward the rear side); [4] reports it but no curve is digitised
REAR_FOOT_YAW = 0.0       # deg from +X: the rear foot square to the plate
STRIDE = 0.42             # x H between the ankles ([4]: 42 % of height, toe to toe; measured)
STRIDE_CLOSED = 0.11      # m, lead foot toward the plate ([4], measured)
HEEL_MAX = 60.0           # deg [E] the most a heel can be raised on the ball of the foot
MU_START = 0.8            # the starting horizontal force on a foot is kept inside this friction cone
VEL_DT = 0.010            # s: the starting velocities are slopes of [4]'s curves over +-10 ms [E: 3, 5, 10 tried, body-motion 4.4]
HANDS_GUESS = (0.10, -0.05, 0.02)   # m, first guess of the hands from the rear shoulder (trunk frame); the solve moves it

SIDES = (("L", 1.0), ("R", -1.0))

# ---- the feet on dirt (body-motion 4.11, part 3; research note 12 D) ------------------------------
MU_CLEAT = 1.15           # a baseball shoe (metal studs) on a basepath mix: static 1.04-1.27, dynamic 1.02-1.20 (Brosnan, McNitt &
#                           Serensits 2009, FT). It replaces MU = 0.8, which had no source, in Model(feet="cleats").
BALL_HALF_W = 0.05        # m [E: 0.045-0.055]: the forefoot's studs either side of the foot's line (a forefoot is about 10 cm wide)

# ---- each hand's own grip (body-motion 4.11, session 9) ------------------------------------------
# A hand's own directions: d from the wrist toward the knuckles, r toward the thumb, p out of the palm (a right hand
# has r = d x p, a left hand r = p x d). Both hands hold the bat thumb toward the barrel with the palms facing each
# other across the handle: the rear (right) palm faces along the bat's +y, the lead's along -y, and so both wrists
# lie to the bat's -x side.
GRIP_OBLIQUE = 15.0       # deg [E: 10-20]: the barrel's end of the handle tilted away from the wrist in the closed hand, so
#                           that with the wrist straight forearm and barrel are 105 deg apart (Kapandji: 100-110, from memory,
#                           not re-read; ergonomics texts treat the power grip as square, 0). 4.11 says what 0 and 30 do.
GRIP_ROLL = {"L": 0.0, "R": 0.0}   # deg [E: +-20]: each hand's roll about the handle from the palms squarely opposed
# the closed hand's loss of wrist range (Gehrmann 2008, a hand on a 25 mm cylinder; research note 11 A1)
CLOSED_HAND = {"wrist_uln": 0.90, "wrist_flex": 0.84}
ARM_STOPS = ("wrist_dev", "wrist_flex", "forearm", "upper_arm")   # the arm ranges the hands' body holds at anatomical ends
# ---- the wrist's oblique range (body-motion 4.12, session 10; research note 11 A1) -----------------
# A wrist's reach is not a box: bent right back it cannot also be tilted right away from the thumb. Model(wrist="oval")
# holds each wrist's two angles inside the ellipse through the box's four ends (the single-direction ranges, which
# stay as measured), by one stop that reads both (engine kind 3), in place of the two box stops.
WRIST_OVAL_TILT = 0.0     # deg [E: 0-8]: the ellipse's long axis turned from pure bending toward bent-forward-and-away-from-
#                           the-thumb (Crisco 2011's cadaver wrists: 26.6 +- 4.4; no ellipse through the four measured ends
#                           has that tilt, body-motion 4.12)
START_CLEAR_DEG = 5.0     # deg [E: 3-8]: the cleared start has every joint this far inside its end (clear_start)


def wrist_oval(R=None, tilt=None):
    """(centre bend, centre tilt, half-axis bend, half-axis tilt, tilt of the long axis), deg, in the clinical senses
    of arm_range() (bent toward the palm +, tilted toward the thumb +): the ellipse through the box's four ends. Not
    tilted its centre is the box's and its half-axes the box's half-ranges; tilted (the long axis turned toward
    bend + with tilt -), the conic through the same four ends with that axis, which exists up to about 21 deg."""
    ar = arm_range(R)
    (x2, x1), (y2, y1) = ar["wrist_flex"], ar["wrist_dev"]
    tilt = WRIST_OVAL_TILT if tilt is None else tilt
    a11 = -1.0 / (x1 * x2); b1 = -a11 * (x1 + x2) / 2.0
    a22 = -1.0 / (y1 * y2); b2 = -a22 * (y1 + y2) / 2.0
    a12 = 0.5 * math.tan(-2.0 * tilt * D2R) * (a11 - a22)
    det = a11 * a22 - a12 * a12
    if det <= 0.0:
        raise ValueError("no ellipse through the wrist's four ends is tilted %.1f deg" % tilt)
    cx = (-b1 * a22 + b2 * a12) / det; cy = (-b2 * a11 + b1 * a12) / det
    k = 1.0 + a11 * cx * cx + 2 * a12 * cx * cy + a22 * cy * cy
    tr, d = a11 + a22, math.sqrt((a11 - a22) ** 2 + 4 * a12 * a12)
    return cx, cy, math.sqrt(2.0 * k / (tr - d)), math.sqrt(2.0 * k / (tr + d)), tilt


def oval_place(flex, dev, R=None, tilt=None):
    """(rho, deg inside): where a wrist bent `flex` and tilted `dev` (clinical senses, deg) lies in wrist_oval(): rho is
    1 on the ellipse's edge and under 1 inside; the second is how far inside the edge it is along the line from the
    centre (negative: outside), the engine's kind 3 reading with its sign turned."""
    cx, cy, ha, hb, tl = wrist_oval(R, tilt)
    ct, sn = math.cos(tl * D2R), math.sin(tl * D2R)
    x, y = flex - cx, dev - cy
    rho = math.hypot((x * ct - y * sn) / ha, (x * sn + y * ct) / hb)
    return rho, (math.hypot(x, y) * (1.0 / rho - 1.0) if rho > 1e-12 else min(ha, hb))


def oval_spec(sd):
    """engine.System.stop's `oval` for hand sd: the second angle (the wrist's bend, read from the forearm as the box's
    wrist_flex stop reads it) and the ellipse in the plane of the two READINGS (rad). Both readings run against the
    clinical sense by the same sign (clinical_sign), a turn of the plane through 180 deg about its origin, which
    leaves the ellipse's shape and tilt as they are and moves only its centre."""
    sg = 1.0 if sd == "L" else -1.0
    cx, cy, ha, hb, tl = wrist_oval()
    k = clinical_sign("wrist_dev", sg)
    # first angle the tilt (centre k x cy, half-axis hb), second the bend (centre k x cx, half-axis ha); the long axis
    # lies along (-sin, cos) of the tilt in (tilt, bend), so the first half-axis is turned +tilt toward the second
    return ((1, 0, 0), (0, -sg, 0), hand_frame(sd)[2], k * cy * D2R, k * cx * D2R, hb * D2R, ha * D2R, tl * D2R)


def hand_frame(sd, oblique=None, roll=None):
    """(d, r, p) of hand sd in the bat's own axes (the bat's z runs knob to barrel): see the block above."""
    sg = 1.0 if sd == "L" else -1.0
    be = (GRIP_OBLIQUE if oblique is None else oblique) * D2R
    cb, sb = math.cos(be), math.sin(be)
    Rz = E.qR(E.qaxis((0, 0, 1), (GRIP_ROLL[sd] if roll is None else roll) * D2R))
    return E.mv(Rz, (cb, 0.0, sb)), E.mv(Rz, (-sb, 0.0, cb)), E.mv(Rz, (0.0, -sg, 0.0))


def arm_range(R=None):
    """{angle: (lo, hi)} deg, each in its clinical sense: the wrist tilted toward the thumb +, bent toward the palm +,
    the forearm turned palm-down +, the upper arm turned in +; the wrist's as a closed hand has them (4.11)."""
    R = R or ROM
    return {"wrist_dev": (-R["wrist_uln"] * CLOSED_HAND["wrist_uln"], R["wrist_rad"]),
            "wrist_flex": (-R["wrist_ext"], R["wrist_flex"] * CLOSED_HAND["wrist_flex"]),
            "forearm": (-R["sup"], R["pron"]), "upper_arm": (-R["sh_erot"], R["sh_irot"])}


# which way each stop's own reading runs against the clinical sense above: clinical = sign x reading
def clinical_sign(name, sg):
    return sg if name == "forearm" else -sg


def arm_angles(sd, zu, zf, Rb, Rt):
    """the four clinical angles of arm_range(), deg, from directions alone (world axes): zu the upper arm from elbow to
    shoulder, zf the forearm from wrist to elbow, Rb the bat's and Rt the trunk's rotation. The placement's own
    geometry; the engine's stops read the same angles from the bodies' frames and the tests hold the two together."""
    sg = 1.0 if sd == "L" else -1.0
    d, r, p = (E.mv(Rb, v) for v in hand_frame(sd))
    xf = E.scale(E.unit(E.cross(p, zf)), -sg)                    # the forearm's radial line: the wrist's first axis, square to p
    dev = -sg * math.atan2(E.dot(p, E.cross(r, xf)), E.dot(r, xf))
    q = E.scale(E.cross(zf, xf), -sg)                            # the forearm's palm side
    flex = -sg * math.atan2(E.dot(xf, E.cross(q, p)), E.dot(q, p))
    yu = E.unit(E.cross(zu, zf))                                 # the elbow's hinge, flexion positive about it
    n0 = E.cross(zf, yu)                                         # where the radial line lies with the forearm in neutral
    pron = -sg * math.atan2(E.dot(zf, E.cross(n0, xf)), E.dot(n0, xf))
    u, e0 = E.mv(Rt, (0, 0, 1)), E.mv(Rt, (0, -1, 0))
    kx = E.cross(u, zu)
    ke = E.cross(kx, e0)
    ec = E.add(E.add(e0, ke), E.scale(E.cross(kx, ke), 1.0 / (1.0 + E.dot(u, zu))))
    irot = -sg * math.atan2(E.dot(zu, E.cross(ec, yu)), E.dot(ec, yu))
    arm = E.scale(zu, -1.0)                                      # shoulder toward elbow
    across = math.asin(max(-1.0, min(1.0, E.dot(E.mv(Rt, (0, -sg, 0)), arm))))    # as the "shoulder_" stop reads it
    back = math.asin(max(-1.0, min(1.0, E.dot(E.mv(Rt, (-1, 0, 0)), arm))))       # as "shoulder_back_"
    return {"wrist_dev": dev / D2R, "wrist_flex": flex / D2R, "forearm": pron / D2R, "upper_arm": irot / D2R,
            "across": across / D2R, "back": back / D2R}


SWEEP_SPAN = 45.0          # deg [E]: the shoulder's sweep counts toward the start's strain over the last 45 deg before its end


def strain(angles, rng=None, oval=False):
    """how near an arm's joint is to an end of its range, the worst of them, 1 = at an end: each of the four angles of
    arm_range() by its distance from the middle of its range in half-ranges, and (where given) the upper arm's sweep
    across the chest and back behind the shoulders' line, which have one end each, over the last SWEEP_SPAN before it.
    oval (4.12): the wrist's two angles count together, by their place in the wrist's ellipse (1 on its edge)."""
    rng = rng or arm_range()
    worst = max(abs((angles[k] - 0.5 * (lo + hi)) / (0.5 * (hi - lo))) for k, (lo, hi) in rng.items() if not (oval and k.startswith("wrist")))
    if oval:
        worst = max(worst, oval_place(angles["wrist_flex"], angles["wrist_dev"])[0])
    for k, hi in (("across", ROM["sh_hadd"] - 90.0), ("back", ROM["sh_habd"])):
        if k in angles:
            worst = max(worst, (angles[k] - (hi - SWEEP_SPAN)) / SWEEP_SPAN)
    return worst

# ---- the joints' ranges (body-motion 4.10, session 8; PLAN D37) ------------------------------------
# Where each joint's range ends, deg: the mean of a sample of adult men measured passively with a goniometer.
# One rule for which sample: Soucie 2011 (CDC; 114 men of 20-44) where it measured the motion; else Roaas &
# Andersson 1982 (105 men of 30-40; the hip's sideways and turning ranges); else Boone & Azen 1979 (109 males,
# active; the wrist, through Norkin & White's table). The trunk's twist is the largest that professional batters
# and pitchers were measured to reach (Fleisig 2013: 46 +- 9 and 55 +- 6 deg), so it is a floor on the range.
# The sources and how each was read are in body-motion 4.10's table. Nothing here is from [4].
ROM = {"hip_flex": 130.4, "hip_ext": 17.4, "hip_abd": 38.8, "hip_add": 30.5, "hip_irot": 32.6, "hip_erot": 33.6,
       "knee_flex": 137.7, "knee_hyper": 1.0, "ankle_df": 12.7, "ankle_pf": 54.6,
       "elbow_flex": 144.6, "elbow_hyper": 0.8, "pron": 76.9, "sup": 85.0,
       "wrist_rad": 22.0, "wrist_uln": 36.0, "wrist_flex": 76.0, "wrist_ext": 75.0,
       "trunk_twist": 55.0,                 # [E: 46-64]
       "trunk_fwd": 80.0, "trunk_back": 25.0, "trunk_side": 35.0,   # [E] AAOS's thoracolumbar values, not read this session
       "sh_hadd": SH_ADD_MAX, "sh_habd": 45.0,                      # [E] 135: clinical; 45 behind the frontal plane: not read
       "sh_irot": 69.0, "sh_erot": 104.0,                           # the upper arm's turn about its own length (Boone & Azen, as the wrist)
       "sh_raise": 168.8,                                           # how far the arm is raised from hanging: Soucie 2011, forward, men 20-44. ONE value in every
       #                                                              direction [E]: sideways a shoulder goes to 184 (Boone & Azen), so sideways this end is 15 deg tighter than anatomy
       # the ankle with weight on it (4.11, part 3): the shin forward over the foot with the knee bent 20 deg or more
       # (Baumbach 2014; 25 with the knee straight, which a stop cannot know: [E]); the foot pointed down and the sole
       # rolled in and out (Hallaceli 2014, passive; the roll its men's)
       "ankle_df_loaded": 39.0, "ankle_pf_loaded": 50.0, "ankle_inv": 34.6, "ankle_ever": 19.1}
# Past the end of a range the joint resists as a one-sided spring. NO measurement of that resistance was found for
# any joint here (body-motion 4.10): the stiffness is [E]. "big": the trunk, hips, knees, ankles and the shoulders'
# sweep; "small": the elbows, forearms and wrists (their strengths are 17-112 N m, and a stiffer spring on so
# little inertia would need a smaller step). Ranges tested: 2,000 and 6,000 for the big joints.
RANGE_K = {"big": 20000.0, "small": 2000.0}      # N m/rad
RANGE_C = {"big": 60.0, "small": 20.0}           # N m s/rad


def cock_reference(mdl, st):
    """{side: +1 or -1}: which way along the handle each forearm's x points in state st (it is set by how the
    placement closes the hands; drive.add_drives reads the same thing for the wrists' flexion drives). The cock is
    read from that end of the handle, so its reading starts within 90 deg of zero and its 58 deg cannot reach the wrap."""
    bz = mdl.axis(st, "bat", (0, 0, 1))
    return {sd: (1.0 if E.dot(mdl.axis(st, "farm_" + sd, (1, 0, 0)), bz) > 0 else -1.0) for sd, sg in SIDES}


def range_specs(cock_ref=None, grip="weld", feet="plate", wrist="box"):
    """every joint range as an engine stop: (name, body a, body b, axis in a, e in a, f in b, lo deg, hi deg, size, kind).
    grip="hands" (4.11): the wrists' two angles, the forearms' turn and the upper arms' turn at their anatomical ends,
    each read on the joint's own axis (the upper arm's by a kind 2 stop), in place of the weld's two wrist stops.
    The angle is the one engine.System.stop measures. kind 0: about the axis (fixed in a), from e (in a) to f (in b)
    projected square to the axis. kind 1: f's angle out of the plane square to the axis. Where each is undefined
    (f along the axis) is said beside it and in body-motion 4.10; the engine fails loudly within 5 deg of it.
    cock_ref: cock_reference()'s signs (default +1)."""
    R = ROM
    cock_ref = cock_ref or {"L": 1.0, "R": 1.0}
    out = [("trunk_twist", "pelvis", "trunk", (0, 0, 1), (1, 0, 0), (1, 0, 0), -R["trunk_twist"], R["trunk_twist"], "big"),
           ("trunk_bend", "pelvis", "trunk", (0, 1, 0), (0, 0, 1), (0, 0, 1), -R["trunk_back"], R["trunk_fwd"], "big"),
           ("trunk_side", "pelvis", "trunk", (1, 0, 0), (0, 0, 1), (0, 0, 1), -R["trunk_side"], R["trunk_side"], "big")]
    for sd, sg in SIDES:
        L = sg > 0
        out += [
            # the thigh swung forward (flexion) is a turn of its long axis about the pelvis's y by a negative angle
            ("hip_flex_" + sd, "pelvis", "thigh_" + sd, (0, 1, 0), (0, 0, 1), (0, 0, 1), -R["hip_flex"], R["hip_ext"], "big"),
            # the thigh's sideways angle as the joint's own (the ISB hip's ab/adduction): the angle of the thigh, hip toward
            # knee, out of the pelvis's fore-and-aft plane, positive toward the body's left (abduction on the left leg).
            # A stop of kind 1. (Until session 8's review this was the thigh projected about the pelvis's x, which reads
            # 39 deg for a true 30 when the hip is also bent 45.) Undefined only with the thigh straight out to the side.
            ("hip_abd_" + sd, "pelvis", "thigh_" + sd, (0, 1, 0), (0, 0, 1), (0, 0, -1),
             -(R["hip_add"] if L else R["hip_abd"]), R["hip_abd"] if L else R["hip_add"], "big", 1),
            # about the thigh's own axis, from the knee's hinge to the pelvis's y: positive is the left thigh turned in
            ("hip_rot_" + sd, "thigh_" + sd, "pelvis", (0, 0, 1), (0, 1, 0), (0, 1, 0),
             -(R["hip_erot"] if L else R["hip_irot"]), R["hip_irot"] if L else R["hip_erot"], "big"),
            ("knee_" + sd, "thigh_" + sd, "shank_" + sd, (0, 1, 0), (0, 0, 1), (0, 0, 1), -R["knee_hyper"], R["knee_flex"], "big"),
            # the foot's toe turned down about the shank's hinge is positive
            ("ankle_" + sd, "shank_" + sd, "foot_" + sd, (0, 1, 0), (1, 0, 0), (1, 0, 0), -R["ankle_df"], R["ankle_pf"], None)   # reported, not held: the range is a straight knee's, unloaded
            if feet != "cleats" else
            # feet="cleats" (4.11, part 3): both of the ankle's angles held at their ends with weight on. The bend as above;
            # the roll about the foot's own length, from the foot's y to the shank's hinge (which the ankle holds square
            # to the foot's length): positive is the LEFT sole rolled toward the midline (inversion), the right sole away
            ("ankle_" + sd, "shank_" + sd, "foot_" + sd, (0, 1, 0), (1, 0, 0), (1, 0, 0), -R["ankle_df_loaded"], R["ankle_pf_loaded"], "big"),
            # the upper arm across the chest: its angle, shoulder toward elbow, out of the trunk's fore-and-aft plane toward
            # the other side (kind 1). 0 is the arm anywhere in that plane, hanging included; the range's end is 135 deg
            # from straight out to its own side, so +45 here. (The projection "seen down the trunk's axis" of sessions
            # 2-8 has no value with the arm hanging, where its gradient grows without limit: 2.4 times at 25 deg from
            # hanging, which swings reach. This reading is undefined only with the arm pointing straight across.)
            ("shoulder_" + sd, "trunk", "uarm_" + sd, (0, -sg, 0), (0, 0, 1), (0, 0, -1), -91.0, R["sh_hadd"] - 90.0, "big", 1),
            # and drawn back behind the shoulders' line: its angle out of the trunk's side-to-side plane, backward
            ("shoulder_back_" + sd, "trunk", "uarm_" + sd, (-1, 0, 0), (0, 0, 1), (0, 0, -1), -91.0, R["sh_habd"], "big", 1),
            ("elbow_" + sd, "uarm_" + sd, "farm_" + sd, (0, 1, 0), (0, 0, 1), (0, 0, 1), -R["elbow_hyper"], R["elbow_flex"], "small")]
        if feet == "cleats":
            out.append(("ankle_roll_" + sd, "foot_" + sd, "shank_" + sd, (1, 0, 0), (0, 1, 0), (0, 1, 0),
                        -(R["ankle_ever"] if L else R["ankle_inv"]), R["ankle_inv"] if L else R["ankle_ever"], "big"))
        if grip == "hands":
            d, r, p = hand_frame(sd)
            ar = arm_range(R)

            def ends(name):
                lo, hi = ar[name]
                return (lo, hi) if clinical_sign(name, sg) > 0 else (-hi, -lo)
            box = None if wrist == "oval" else "small"      # wrist="oval" (4.12): the two angles are read and reported, and held together
            if wrist == "oval":
                # both of the wrist's angles inside its ellipse (kind 3): the first angle is the tilt's below; the second
                # (the bend's) and the ellipse are oval_spec()'s. The reading is how far outside the ellipse, 0 at its edge
                out.append(("wrist_oval_" + sd, "bat", "farm_" + sd, p, r, (1, 0, 0), -1e9, 0.0, "small", 3))
            out += [
                # the wrist's tilt toward and away from the thumb: about the hand's p (fixed in the bat), from the hand's r
                # to the forearm's radial line (its x), which the joint holds square to p
                ("wrist_dev_" + sd, "bat", "farm_" + sd, p, r, (1, 0, 0)) + ends("wrist_dev") + (box,),
                # its bend toward the palm and back: about the forearm's radial line, from the forearm's palm side to p
                ("wrist_flex_" + sd, "farm_" + sd, "bat", (1, 0, 0), (0, -sg, 0), p) + ends("wrist_flex") + (box,),
                # the forearm's turn: about its own length, from where the elbow's hinge lies in neutral (thumb toward the
                # upper arm with the elbow bent) to the hinge, which the elbow holds square to the forearm
                ("forearm_" + sd, "farm_" + sd, "uarm_" + sd, (0, 0, 1), (0, -1, 0), (0, 1, 0)) + ends("forearm") + ("small",),
                # the upper arm's turn about its own length (kind 2): from hanging along the trunk's z; zero is the elbow's
                # hinge along the trunk's -y (the elbow bending straight forward)
                ("upper_arm_" + sd, "trunk", "uarm_" + sd, (0, 0, 1), (0, -1, 0), (0, 1, 0)) + ends("upper_arm") + ("small", 2),
                # how far the arm is raised (kind 1): the angle of the upper arm's own z out of the trunk's level plane, 90 deg
                # hanging and 90 - (the raise) otherwise. A shoulder's end, and what keeps the stop above defined: its
                # reading has no value with the arm straight overhead, and session 9's first runs went there (4.11)
                ("shoulder_up_" + sd, "trunk", "uarm_" + sd, (0, 0, 1), (0, 0, 1), (0, 0, 1), 90.0 - R["sh_raise"], 91.0, "big", 1)]
            continue
        out += [
            # the wrist's cock, on the joint's own axis: the wrist is a universal joint whose second axis is the bat's
            # y, and the forearm's x (its first axis) is held square to it, so the angle of the forearm's x about the
            # bat's y, from the handle, IS the cock. (Until session 8's review this read the forearm's long axis
            # projected onto the bat's x-z plane, which turns over by 180 deg when the wrist's flexion passes 90.)
            # What the number means: with the wrist unflexed, 90 deg less the angle between the forearm (wrist toward elbow)
            # and the barrel (in general cos of that angle = sin(cock) x cos(flexion)). Zero is
            # the forearm square to the handle; positive is the barrel folded back toward the elbow (the thumb's side,
            # radial deviation, since both hands hold the bat thumb toward the barrel); negative is the bat opened out
            # toward a straight line with the forearm (the bat released). Which way is uncocking is not written
            # here: add_ranges takes it from the model.
            ("wrist_dev_" + sd, "bat", "farm_" + sd, (0, 1, 0), (0, 0, cock_ref[sd]), (1, 0, 0), -R["wrist_uln"], R["wrist_rad"], "small"),
            # the wrist's flexion and extension: about the forearm's x, from the forearm's y to the bat's y
            ("wrist_flex_" + sd, "farm_" + sd, "bat", (1, 0, 0), (0, 1, 0), (0, 1, 0), -R["wrist_ext"], R["wrist_flex"], "small")]
    return [t if len(t) == 10 else t + (0,) for t in out]      # the tenth entry: the stop's kind (engine.System.stop)


RANGE_CLEAR = 2.0          # deg: a stop begins at least this far beyond the start's own angle (the start is a hitter's posture)
WRIST_COCK_INSIDE = 5.0    # deg [E]: at the start each wrist is this far inside the cocked end of its 58 deg of travel


def uncock_sign(mdl, st, d=1e-3):
    """+1 if the wrists' cock readings RISE as the bat uncocks, -1 if they fall, found from the model itself: the bat
    is turned a little about its own y, whichever way brings its barrel toward the lead arm's line (the lag falls,
    as it does through a swing), and the readings are read again. Both wrists share the bat's y, so one sign serves both."""
    i = mdl.b["bat"]
    lag0 = mdl.observe(st)["bat_lag"]
    a0 = range_angles(mdl, st)
    by = mdl.axis(st, "bat", (0, 1, 0))
    for turn in (d, -d):
        s2 = list(st)
        s2[13 * i + 3:13 * i + 7] = E.qmul(E.qaxis(by, turn), tuple(st[13 * i + 3:13 * i + 7]))
        if mdl.observe(s2)["bat_lag"] < lag0:
            a1 = range_angles(mdl, s2)
            return 1.0 if a1["wrist_dev_L"] > a0["wrist_dev_L"] else -1.0
    raise ValueError("turning the bat about its y does not change the lag")


def add_ranges(mdl, st, k_big=None, wrists="cocked", rom=None):
    """the joints' ranges as one-sided stops on a model built with ranges=() and placed in state st (body-motion
    4.10). Every stop of range_specs() that has a size, with two rules: a wrist has no anatomical zero in this body
    (both hands are welded to the handle at one roll), so its TRAVEL is held, placed from its angle at the start
    (the cock: the start WRIST_COCK_INSIDE inside its COCKED end, so that the rest of the 58 deg lies the way the bat
    uncocks, which is found from the model by uncock_sign, not from a sign written here; flexion: the start in the
    middle); and no stop begins nearer than RANGE_CLEAR beyond the start's own angle. The other placings of the
    cock, each [E] and each a rerun's: wrists="mirrored" (the travel the other way, toward the barrel folding back on
    the forearm, which is the way the wrists of the swings fitted to [4] went), "middle", and "square" (the clinical
    22 and 36 deg either side of the forearm square to the handle, widened to clear the start).
    Returns {name: (lo, hi)} deg."""
    s, b = mdl.sys, mdl.b
    R = dict(ROM, **(rom or {}))
    keep = dict(ROM)
    ROM.update(R)
    hands = getattr(mdl, "grip", "weld") == "hands"
    mdl.cock_ref = None if hands else cock_reference(mdl, st)
    specs = range_specs(mdl.cock_ref, mdl.grip if hands else "weld", getattr(mdl, "feet", "plate"), getattr(mdl, "wrist", "box"))
    ROM.clear(); ROM.update(keep)
    ang = range_angles(mdl, st)
    out = {}
    for name, a, b_, ax, ea, fb, lo, hi, size, kind in specs:
        if size is None:
            continue
        a0 = ang[name]
        if kind == 3:                                   # a wrist's ellipse (4.12): its reading is in degrees outside, 0 at the edge
            s.stop(name, b[a], b[b_], ax, ea, fb, 0.0, 0.0, RANGE_K[size], RANGE_C[size], 3, oval=oval_spec(name[-1]))
            out[name] = (-1e9, 0.0)
            continue
        if hands and name.startswith(ARM_STOPS):
            # the hands' body (4.11): the anatomical ends as they are. No travel is placed from the start and no stop is
            # moved to clear it; that the start lies inside is a test's business, not this function's.
            k = RANGE_K[size]
            s.stop(name, b[a], b[b_], ax, ea, fb, lo * D2R, hi * D2R, k, RANGE_C[size], kind)
            out[name] = (lo, hi)
            continue
        if name.startswith("wrist_dev"):
            span = R["wrist_rad"] + R["wrist_uln"]
            up = uncock_sign(mdl, st) > 0                # the reading rises as the bat uncocks: the travel lies above the start
            if wrists == "square":                       # the range about the forearm square to the handle [E], wherever the start is
                lo, hi = -R["wrist_uln"], R["wrist_rad"]
            else:
                if wrists == "mirrored":                 # the review's placing: the travel the way the earlier swings' wrists went
                    up = not up
                inside = 0.5 * span if wrists == "middle" else WRIST_COCK_INSIDE
                if up:
                    lo = a0 - inside
                    hi = lo + span
                else:
                    hi = a0 + inside
                    lo = hi - span
        elif name.startswith("wrist_flex"):
            span = R["wrist_flex"] + R["wrist_ext"]
            lo, hi = a0 - 0.5 * span, a0 + 0.5 * span
        if getattr(mdl, "shift", None):
            # the cleared start (4.12): every stop stands at its anatomical end; none is moved to clear the start
            if not lo < a0 < hi:
                raise ValueError("%s: the start (%.1f deg) is not inside its range (%.1f to %.1f)" % (name, a0, lo, hi))
        else:
            lo, hi = min(lo, a0 - RANGE_CLEAR), max(hi, a0 + RANGE_CLEAR)
        k = RANGE_K[size] if (k_big is None or size != "big") else k_big
        c = RANGE_C[size] * (k / RANGE_K[size]) ** 0.5
        s.stop(name, b[a], b[b_], ax, ea, fb, lo * D2R, hi * D2R, k, c, kind)
        out[name] = (lo, hi)
    return out


def range_angles(mdl, st):
    """{name: angle, deg} of every range_specs() entry in state st, measured as the engine's stop measures it, and
    three more that no stop holds: each forearm's turn (pronation +) and each upper arm's turn about its own axis
    (internal rotation +, by the swing-twist split from the trunk; zero with the arm hanging and the elbow's hinge
    along the trunk's y), and the arm's angle from hanging. The cock is read from the end of the handle the model
    was given at its start (mdl.cock_ref, set by add_ranges; otherwise taken from st itself)."""
    s = mdl.sys
    out = {}
    Rm = {n: E.qR(s.pose(st, i)[1]) for n, i in mdl.b.items()}
    hands = getattr(mdl, "grip", "weld") == "hands"
    ref = {"L": 1.0, "R": 1.0} if hands else (getattr(mdl, "cock_ref", None) or cock_reference(mdl, st))
    for name, a, b_, ax, ea, fb, lo, hi, size, kind in range_specs(ref, "hands" if hands else "weld", getattr(mdl, "feet", "plate"), getattr(mdl, "wrist", "box")):
        n = E.mv(Rm[a], E.unit(ax)); e = E.mv(Rm[a], E.unit(ea)); f = E.mv(Rm[b_], E.unit(fb))
        if kind == 3:                                   # how far outside the wrist's ellipse, deg (negative inside)
            n2, e2, f2, c1, c2, h1, h2, tl = oval_spec(name[-1])
            t1 = math.atan2(E.dot(n, E.cross(e, f)), E.dot(e, f))
            m2, g2, k2 = E.mv(Rm[b_], E.unit(n2)), E.mv(Rm[b_], E.unit(e2)), E.mv(Rm[a], E.unit(f2))
            t2 = math.atan2(E.dot(m2, E.cross(g2, k2)), E.dot(g2, k2))
            out[name] = s.oval_reading(t1, t2, c1, c2, h1, h2, math.cos(tl), math.sin(tl))[0] / D2R
            continue
        if kind == 2:
            out[name] = s.stop_angle(n, e, f, 2, v=E.mv(Rm[b_], (0.0, 0.0, 1.0)))[0] / D2R
            continue
        out[name] = (math.asin(max(-1.0, min(1.0, E.dot(n, f)))) if kind == 1 else math.atan2(E.dot(n, E.cross(e, f)), E.dot(e, f))) / D2R
    for sd, sg in SIDES:
        zf = E.mv(Rm["farm_" + sd], (0, 0, 1))
        fx = E.mv(Rm["farm_" + sd], (1, 0, 0))
        # the forearm's turn on the elbow's own second axis: the forearm's x about its z, from a reference the upper arm
        # carries (the elbow's hinge crossed with the forearm). Which end of its x counts is fixed once, from the start
        # (ref, the same sign the cock is read with), so the reading cannot turn over. (Until session 8's second review
        # the end was chosen at every instant by which way the forearm's x lay along the bat, which changes when the
        # wrist's cock passes the line of the bat: a 180 deg jump that read a 106 deg turn as 285.)
        g = E.scale(fx, ref[sd])
        r = E.cross(E.mv(Rm["uarm_" + sd], (0, 1, 0)), zf)       # where it points with the forearm neither pronated nor supinated
        ang = math.atan2(E.dot(zf, E.cross(r, g)), E.dot(r, g)) / D2R
        out["pron_" + sd] = -sg * ang                            # the right forearm pronates about its own +z, the left about -z
        qt = s.pose(st, mdl.b["trunk"])[1]; qu = s.pose(st, mdl.b["uarm_" + sd])[1]
        q = E.qmul(E.qconj(qt), qu)
        tw = 2.0 * math.atan2(q[3], q[0]) / D2R
        tw = (tw + 180.0) % 360.0 - 180.0
        out["sh_rot_" + sd] = -sg * tw                           # the right arm turns in about its own +z, the left about -z
        zt = E.mv(Rm["trunk"], (0, 0, 1)); zu = E.mv(Rm["uarm_" + sd], (0, 0, 1))
        out["sh_elev_" + sd] = math.acos(max(-1.0, min(1.0, E.dot(zu, zt)))) / D2R
    return out


def clinical(mdl, st):
    """{(angle, side): deg} of the hands' body's four arm angles a side, in arm_range()'s clinical senses, read as the
    engine's stops read them (range_angles), and each forearm's angle to the barrel."""
    a = range_angles(mdl, st)
    out = {}
    for sd, sg in SIDES:
        for k in ARM_STOPS:
            out[(k, sd)] = clinical_sign(k, sg) * a[k + "_" + sd]
        out[("raised", sd)] = 90.0 - a["shoulder_up_" + sd]
        out[("forearm_to_barrel", sd)] = math.acos(max(-1.0, min(1.0, E.dot(mdl.axis(st, "farm_" + sd, (0, 0, 1)), mdl.axis(st, "bat", (0, 0, 1)))))) / D2R
    return out


def curve(name, t):
    """[4]'s digitised mean curve at time t (s), by linear interpolation."""
    if name in CURVE_OVERRIDE:                      # a sensitivity case's own reading of a curve (inverse3d.py)
        return CURVE_OVERRIDE[name](t)
    rows = _CURVES.get(name)
    if rows is None:
        rows = _CURVES[name] = C.load(name)
    return C.resample(rows, t, t, 1.0)[0]


_CURVES = {}
CURVE_OVERRIDE = {}


def curve_rate(name, t, h=0.005):
    return (curve(name, t + h) - curve(name, t - h)) / (2 * h)


class Model:
    """the engine System of a batter of stature H (m) and mass M (kg), with named bodies and joint points."""

    def __init__(self, H=1.84, M=92.0, contacts=True, mu=None, shoulder_stops=True, ranges=None, grip="weld", feet="plate", wrist="box", shift=None):
        """ranges: None, the stops of sessions 1-7 (the knees' and elbows' bend, the shoulders' sweep across, at
        STOP_K); or a set of names from range_specs() ("all" for every one): those ranges as stops at RANGE_K, in
        place of the old ones (body-motion 4.10).
        grip: "weld", the body of sessions 1-8 (both wrists 7 cm square off the handle, the start's free choices set by
        hand, the wrists' travel held from the start), kept so that the dated results of body-motion 4.4-4.10 and the
        probe's check still run; "hands", each hand's own grip (4.11): the handle oblique in the hand, anatomical wrist,
        forearm and upper-arm ranges, the start's free choices by a rule (pose).
        feet: "plate", the feet of sessions 1-8 (the grip MU with no source, the ball's contact points SOLE_HALF_W either
        side, no end to either of the ankle's angles); "cleats" (4.11, part 3): the measured grip of a baseball shoe on
        dirt, the forefoot's studs BALL_HALF_W either side, and add_ranges() holds both of the ankle's angles."""
        self.H, self.M = H, M
        self.ranges = ranges
        self.grip = grip
        self.feet = feet
        # wrist: "box" (session 9: each of the wrist's two angles held at its own ends) or "oval" (4.12: both inside the
        # wrist's ellipse, one stop reading both). shift: {posture number: change} applied to the launch posture's numbers
        # by pose() (POSTURE_NUMBERS; START_CLEAR is the cleared start's); with one, add_ranges() moves no stop.
        self.wrist = wrist
        self.shift = dict(shift) if shift else None
        if mu is None:
            mu = MU_CLEAT if feet == "cleats" else MU
        self.mu = mu
        s = self.sys = E.System()
        seg = lambda n: C.segment(n, H, M)
        self.b = b = {}
        self.jp = jp = {}           # joint name -> (body a, anchor in a, body b, anchor in b)
        # pelvis (de Leva's lower trunk: omphalion to mid-hip, centre of mass 61.15 % down)
        Lp, mp, cp, Is, It, Il = seg("lpt")
        b["pelvis"] = s.body("pelvis", mp, (Is, It, Il))
        lumbar_p = (0.0, 0.0, cp * Lp)
        # trunk: middle trunk, upper trunk and head stacked on the lumbar joint, parallel axes along z
        parts, z = [], 0.0
        for n in ("mpt", "upt", "head"):
            L, m, c, a, t, l = seg(n)
            parts.append((m, z + (1.0 - c) * L, a, t, l))     # each row's origin is its upper end
            if n == "mpt":
                z_sh = L
            if n == "upt":
                z_sh += SHOULDER_UP * L
            z += L
        mt = sum(p[0] for p in parts)
        zc = sum(p[0] * p[1] for p in parts) / mt
        b["trunk"] = s.body("trunk", mt, (sum(p[2] + p[0] * (p[1] - zc) ** 2 for p in parts),
                                          sum(p[3] + p[0] * (p[1] - zc) ** 2 for p in parts), sum(p[4] for p in parts)))
        self.eye_z = z - zc
        s.ball(b["pelvis"], b["trunk"], lumbar_p, (0.0, 0.0, -zc))
        jp["lumbar"] = (b["pelvis"], lumbar_p, b["trunk"], (0.0, 0.0, -zc))
        Lth, mth, cth, a, t, l = seg("thigh"); Ith = (a, t, l)
        Lsh, msh, csh, a, t, l = seg("shank"); Ish = (a, t, l)
        Lf, mf, cf, a, t, l = seg("foot"); Ift = (l, t, a)      # the foot's long axis is its x
        Lu, mu_, cu, a, t, l = seg("upperarm"); Iu = (a, t, l)
        Lfa, mfa, cfa, a, t, l = seg("forearm"); Ifa = (a, t, l)
        mh = seg("hand")[1]
        self.len = {"thigh": Lth, "shank": Lsh, "foot": Lf, "upperarm": Lu, "forearm": Lfa}
        ank_h = ANKLE_H * H
        fcm_h = 0.5 * ank_h                                     # [E] the foot's centre of mass at half the ankle's height
        ankle_f = ((ANKLE_FWD - cf) * Lf, 0.0, ank_h - fcm_h)
        self.ankle_f, self.sole_z = ankle_f, -fcm_h
        # the bat with both hands on it
        mb = C.BAT_M + 2 * mh
        ucm = (C.BAT_M * C.BAT_UCM + mh * (GRIP_L + GRIP_R)) / mb
        It_b = (C.BAT_ICM + C.BAT_M * (C.BAT_UCM - ucm) ** 2
                + mh * ((GRIP_L - ucm) ** 2 + (GRIP_R - ucm) ** 2 + 2 * HAND_R ** 2))
        b["bat"] = s.body("bat", mb, (It_b, It_b, C.BAT_IAX + 2 * mh * HAND_R ** 2))
        self.bat_ucm = ucm
        self.wrist_b = {"L": (-WRIST_TO_GRIP, 0.0, GRIP_L - ucm), "R": (-WRIST_TO_GRIP, 0.0, GRIP_R - ucm)}
        self.palm = {"L": (0.0, 1.0, 0.0), "R": (0.0, 1.0, 0.0)}          # the wrists' second axis, fixed in the bat (the weld's: its y)
        if grip == "hands":
            for sd, g in (("L", GRIP_L), ("R", GRIP_R)):
                d, r, p_ = hand_frame(sd)
                self.wrist_b[sd] = E.sub((0.0, 0.0, g - ucm), E.scale(d, WRIST_TO_GRIP))   # the wrist's centre, back along the hand from the handle's axis
                self.palm[sd] = p_
        for sd, sg in SIDES:
            th = b["thigh_" + sd] = s.body("thigh_" + sd, mth, Ith)
            sh = b["shank_" + sd] = s.body("shank_" + sd, msh, Ish)
            ft = b["foot_" + sd] = s.body("foot_" + sd, mf, Ift)
            ua = b["uarm_" + sd] = s.body("uarm_" + sd, mu_, Iu)
            fa = b["farm_" + sd] = s.body("farm_" + sd, mfa, Ifa)
            hip_p = (0.0, sg * HIP_HALF * H, -(1.0 - cp) * Lp)
            sh_t = (0.0, sg * SHOULDER_HALF * H, z_sh - zc)
            J = {"hip_" + sd: (b["pelvis"], hip_p, th, (0.0, 0.0, cth * Lth)),
                 "knee_" + sd: (th, (0.0, 0.0, -(1 - cth) * Lth), sh, (0.0, 0.0, csh * Lsh)),
                 "ankle_" + sd: (sh, (0.0, 0.0, -(1 - csh) * Lsh), ft, ankle_f),
                 "shoulder_" + sd: (b["trunk"], sh_t, ua, (0.0, 0.0, cu * Lu)),
                 "elbow_" + sd: (ua, (0.0, 0.0, -(1 - cu) * Lu), fa, (0.0, 0.0, cfa * Lfa)),
                 "wrist_" + sd: (fa, (0.0, 0.0, -(1 - cfa) * Lfa), b["bat"], self.wrist_b[sd])}
            jp.update(J)
            s.ball(*_j(J["hip_" + sd]))
            s.hinge(*_j(J["knee_" + sd]), (0, 1, 0), (0, 1, 0))
            s.universal(*_j(J["ankle_" + sd]), (0, 1, 0), (1, 0, 0))       # flexion (shank's y), inversion (foot's x)
            s.ball(*_j(J["shoulder_" + sd]))
            s.universal(*_j(J["elbow_" + sd]), (0, 1, 0), (0, 0, 1))       # flexion (upper arm's y), forearm rotation
            s.universal(*_j(J["wrist_" + sd]), (1, 0, 0), self.palm[sd])   # flexion (forearm's x), the cock (the hand's p: the bat's y in the weld)
            # no hyperextension: one-sided stops on the knee's and elbow's bend (place() makes each joint's own
            # +y its flexion axis, so the angle from the upper segment's z to the lower one's about +y is the bend)
            if ranges is None:
                s.stop("knee_" + sd, th, sh, (0, 1, 0), (0, 0, 1), (0, 0, 1), 0.0, FLEX_MAX * D2R, STOP_K, STOP_C)
                s.stop("elbow_" + sd, ua, fa, (0, 1, 0), (0, 0, 1), (0, 0, 1), 0.0, FLEX_MAX * D2R, STOP_K, STOP_C)
            if shoulder_stops and ranges is None:
                # the upper arm cannot be brought further across the chest than its end of range: the angle,
                # seen down the trunk's axis, from straight out to its own side round to the front and across
                s.stop("shoulder_" + sd, b["trunk"], ua, (0, 0, -sg), (0, sg, 0), (0, 0, -1), -10.0, SH_ADD_MAX * D2R, STOP_K, STOP_C)
            if contacts:
                for nm, px in (("heel", -cf * Lf), ("ball", (BALL_FWD - cf) * Lf)):
                    hw = BALL_HALF_W if (feet == "cleats" and nm == "ball") else SOLE_HALF_W
                    for wn, py in (("in", -sg * hw), ("out", sg * hw)):
                        s.contact("%s_%s_%s" % (sd, nm, wn), ft, (px, py, -fcm_h), KN, CN, KT, CT, mu)

    # ---- measures ----
    def joint_pos(self, st, name):
        a, ra, bb, rb = self.jp[name]
        x, q, v, w = self.sys.pose(st, bb)
        return E.add(x, E.mv(E.qR(q), rb))

    def axis(self, st, body, local):
        x, q, v, w = self.sys.pose(st, self.b[body])
        return E.mv(E.qR(q), local)

    def flexion(self, st, a, b_):
        """angle (deg) between two segments' long axes: 0 = straight."""
        return math.acos(max(-1.0, min(1.0, E.dot(self.axis(st, a, (0, 0, 1)), self.axis(st, b_, (0, 0, 1)))))) / D2R

    def bat_point(self, st, u):
        """world position and velocity of the bat's point u metres from the knob."""
        x, q, v, w = self.sys.pose(st, self.b["bat"])
        r = E.mv(E.qR(q), (0.0, 0.0, u - self.bat_ucm))
        return E.add(x, r), E.add(v, E.cross(w, r))

    def observe(self, st):
        """the quantities [4] reports, read off the body's state the way [4] defines them (as far as its
        text says; the shoulders' polar angles are not reproduced)."""
        yaw = lambda body: math.atan2(*reversed(self.axis(st, body, (1, 0, 0))[:2])) / D2R
        bz = self.axis(st, "bat", (0, 0, 1))
        wl = self.joint_pos(st, "wrist_L")
        arm = E.unit(E.sub(wl, self.joint_pos(st, "shoulder_L")))
        o = {"pelvis_rot": yaw("pelvis"), "trunk_rot": yaw("trunk"),
             "bat_az": math.atan2(bz[1], bz[0]) / D2R, "bat_elev": math.asin(max(-1.0, min(1.0, bz[2]))) / D2R,
             "bat_lag": math.acos(max(-1.0, min(1.0, E.dot(arm, bz)))) / D2R,
             "lead_elbow_flex": self.flexion(st, "uarm_L", "farm_L"),
             "trail_elbow_flex": self.flexion(st, "uarm_R", "farm_R"),
             "lead_knee_flex": self.flexion(st, "thigh_L", "shank_L"),
             "trail_knee_flex": self.flexion(st, "thigh_R", "shank_R")}
        for body, k in (("pelvis", "pelvis_rot_vel"), ("trunk", "trunk_rot_vel"), ("bat", "bat_az_vel")):
            x, q, v, w = self.sys.pose(st, self.b[body])
            if body == "bat":       # the rate of the azimuth of the bat's axis
                h2 = bz[0] * bz[0] + bz[1] * bz[1]
                d = E.cross(w, bz)
                o[k] = (bz[0] * d[1] - bz[1] * d[0]) / h2 / D2R
            else:
                o[k] = w[2] / D2R
        vss = self.bat_point(st, C.BAT_SS)[1]
        o["sweet_speed"] = E.norm(vss)
        # the attack angle: the sweet spot's velocity above (+) or below level (deg). A check, never fitted
        # (body-motion 4.5's held-out checks; OpenBiomechanics +4.1 +- 5.9 at contact, research note swing/6)
        o["attack"] = math.asin(max(-1.0, min(1.0, vss[2] / o["sweet_speed"]))) / D2R if o["sweet_speed"] > 1e-9 else 0.0
        # held out of every fit (body-motion 4.5, session 7): each upper arm's angle from hanging at the side, in the
        # trunk's frame ([4]'s shoulder elevation as its polar angle is read here), and the azimuth of the line from
        # the lead shoulder to the lead wrist ([3]'s "arms", [E])
        zt = self.axis(st, "trunk", (0, 0, 1))
        for sd, k in (("L", "lead_sh_elev"), ("R", "trail_sh_elev")):
            d = E.unit(E.sub(self.joint_pos(st, "elbow_" + sd), self.joint_pos(st, "shoulder_" + sd)))
            o[k] = math.acos(max(-1.0, min(1.0, -E.dot(d, zt)))) / D2R
        o["lead_arm_az"] = math.atan2(arm[1], arm[0]) / D2R
        o["pelvis_y"] = self.sys.pose(st, self.b["pelvis"])[0][1]
        return o

    def grf(self, diag):
        """each foot's ground force (N) from the last deriv's diagnostics."""
        out = {"L": [0.0, 0.0, 0.0], "R": [0.0, 0.0, 0.0]}
        for n, f in diag.get("contact", {}).items():
            for c in range(3):
                out[n[0]][c] += f[c]
        return out


def _j(t):
    return t[0], t[2], t[1], t[3]


def _frame_z(z, hint_y):
    """axes with the given z and y as near hint_y as it can be."""
    x = E.unit(E.cross(hint_y, z))
    return x, E.cross(z, x), z


def _two_link(top, end, l1, l2, toward):
    """the middle joint of a two-link limb from `top` to `end`, bent toward the direction `toward`."""
    d = E.sub(end, top)
    D = min(E.norm(d), 0.9999 * (l1 + l2))
    dh = E.unit(d)
    a = (l1 * l1 - l2 * l2 + D * D) / (2 * D)
    h = math.sqrt(max(0.0, l1 * l1 - a * a))
    perp = E.sub(toward, E.scale(dh, E.dot(toward, dh)))
    return E.add(E.add(top, E.scale(dh, a)), E.scale(E.unit(perp), h))


# ---- the launch posture's numbers, and the cleared start (body-motion 4.12, session 10) --------------------------
# What pose() builds the launch posture from: [4]'s digitised curves (deg; the pelvis's travel in cm), [4]'s text values
# (the pelvis's side tilt, the rear knee, the lead foot's angle, the stride's length and how closed it is, cm) and the
# three the placement sets by hand ([E]: the trunk's forward and side bend, the rear foot's angle). A shift is a change
# to some of them, a degree or a centimetre each: Model(shift=...).
POSTURE_CURVES = ("pelvis_rot", "trunk_rot", "bat_az", "bat_elev", "bat_lag", "lead_elbow_flex", "trail_elbow_flex", "lead_knee_flex", "pelvis_x_cm")
POSTURE_NUMBERS = POSTURE_CURVES + ("trunk_flex", "trunk_side", "rear_foot_yaw", "stride_cm", "stride_closed_cm", "obliquity", "trail_knee", "lead_foot")
# The cleared start: clear_start()'s result for the average batter (1.84 m, 92 kg) on the hands' body with cleats and the
# wrists' ellipses, the elbows ARM_CLEAR apart (job 391, 264 s; test_body3d.StartChecks holds it to the rule). Still
# [4]'s launch posture, moved by the rule: every joint START_CLEAR_DEG inside its end, no number changed by more than 2.5.
START_CLEAR = {"pelvis_rot": 1.75, "trunk_rot": -1.75, "bat_az": 2.0, "bat_elev": 2.125, "bat_lag": 2.5, "trail_elbow_flex": -0.625,
               "lead_knee_flex": -0.75, "trunk_side": -0.25, "stride_cm": -0.375, "stride_closed_cm": -0.625, "obliquity": 1.5,
               "trail_knee": 1.0, "lead_foot": 0.625}
LEAD_FOOT_SHIFT = [0.0]
_SHIFT_ON = [False]


class _shifted:
    """the launch posture's numbers changed by `shift` while a posture is placed, and put back afterwards. A changed
    curve is the curve plus a constant (over whatever CURVE_OVERRIDE already holds for it), so its slope is unchanged."""

    def __init__(self, shift, H):
        self.shift, self.H = shift, H

    def __enter__(self):
        global TRUNK_FLEX, TRUNK_SIDE, REAR_FOOT_YAW, STRIDE, STRIDE_CLOSED, OBLIQUITY, TRAIL_KNEE
        d = lambda k: float(self.shift.get(k, 0.0))
        bad = set(self.shift) - set(POSTURE_NUMBERS)
        if bad:
            raise ValueError("not a posture number: %s" % sorted(bad))
        self.keep = (dict(CURVE_OVERRIDE), TRUNK_FLEX, TRUNK_SIDE, REAR_FOOT_YAW, STRIDE, STRIDE_CLOSED, OBLIQUITY, TRAIL_KNEE, LEAD_FOOT_SHIFT[0])
        for k in POSTURE_CURVES:
            if d(k) != 0.0:
                under = CURVE_OVERRIDE.get(k)
                CURVE_OVERRIDE[k] = (lambda kk, sh, un: (lambda t: (un(t) if un else C.resample(_CURVES.setdefault(kk, C.load(kk)), t, t, 1.0)[0]) + sh))(k, d(k), under)
        TRUNK_FLEX += d("trunk_flex"); TRUNK_SIDE += d("trunk_side"); REAR_FOOT_YAW += d("rear_foot_yaw")
        STRIDE += d("stride_cm") / 100.0 / self.H; STRIDE_CLOSED += d("stride_closed_cm") / 100.0
        OBLIQUITY = tuple((t, v + d("obliquity")) for t, v in OBLIQUITY)
        TRAIL_KNEE = tuple((t, v + d("trail_knee")) for t, v in TRAIL_KNEE)
        LEAD_FOOT_SHIFT[0] += d("lead_foot")
        _SHIFT_ON[0] = True

    def __exit__(self, *exc):
        global TRUNK_FLEX, TRUNK_SIDE, REAR_FOOT_YAW, STRIDE, STRIDE_CLOSED, OBLIQUITY, TRAIL_KNEE
        ov, TRUNK_FLEX, TRUNK_SIDE, REAR_FOOT_YAW, STRIDE, STRIDE_CLOSED, OBLIQUITY, TRAIL_KNEE, LEAD_FOOT_SHIFT[0] = self.keep
        CURVE_OVERRIDE.clear(); CURVE_OVERRIDE.update(ov)
        _SHIFT_ON[0] = False
        return False


def start_margins(mdl, st):
    """{range: deg inside its anatomical end} for every range the body holds (range_specs with a size), in state st: the
    distance from the nearer end; a wrist's ellipse by how far inside its edge along the line from its centre.
    Negative: past the end."""
    ang = range_angles(mdl, st)
    out = {}
    for name, a, b_, ax, ea, fb, lo, hi, size, kind in range_specs(None, mdl.grip, mdl.feet, mdl.wrist):
        if size is not None:
            out[name] = -ang[name] if kind == 3 else min(ang[name] - lo, hi - ang[name])
    return out


def clear_start(t0=-0.250, clear=None, log=None, given=None, **model):
    """The rule for the start (body-motion 4.12): a least change to the launch posture's numbers (POSTURE_NUMBERS; the
    sum of the changes' squares, a degree and a centimetre counted alike [E]) that puts every joint at least `clear`
    (START_CLEAR_DEG) inside its anatomical end and each wrist that far inside its ellipse, FOUND BY A COMPASS SEARCH
    WITH A PENALTY: what it returns is least number by number, not shown to be the least of all (session 10's review,
    P3-5). From no change, each number
    in turn is tried a step up and a step down and what lowers (the sum of squares + a penalty on every margin's
    shortfall, squared) is kept; the steps halve from 2 to 1/8 and the penalty's weight rises in four stages.
    Deterministic. model: Model()'s options (the hands' body). Returns ({number: change}, {range: margin}).
    given (body-motion 4.13, rung 3): {number: change} for numbers that are SET from outside (the front foot's place and
    angle, from MLB's data): they are held at those changes, are not counted in the sum of squares, and the rule finds
    the least change to the OTHER numbers on them."""
    clear = START_CLEAR_DEG if clear is None else clear
    n = len(POSTURE_NUMBERS)
    given = given or {}
    free = [i for i, k in enumerate(POSTURE_NUMBERS) if k not in given]

    def margins(x):
        m = Model(ranges=(), shift={k: v for k, v in zip(POSTURE_NUMBERS, x) if v != 0.0} or None, **model)
        return start_margins(m, pose(m, t0)[0])

    def cost(x, mu):
        try:
            mg = margins(x)
        except E.SolveError:
            return 1e12, None
        return sum(x[i] * x[i] for i in free) + mu * sum(max(0.0, clear - v) ** 2 for v in mg.values()), mg
    x = [float(given.get(k, 0.0)) for k in POSTURE_NUMBERS]
    for mu in (10.0, 100.0, 1000.0, 1e4):
        step = 2.0
        f, mg = cost(x, mu)
        while step >= 0.124:
            moved = False
            for i in free:
                for sgn in (1.0, -1.0):
                    xn = list(x); xn[i] += sgn * step
                    fn, mgn = cost(xn, mu)
                    if fn < f - 1e-12:
                        x, f, mg, moved = xn, fn, mgn, True
                        break
            if not moved:
                step *= 0.5
        if log:
            log("   penalty %.0f: sum of squares %.2f, least margin %.2f deg (%s)" % (mu, sum(x[i] * x[i] for i in free), min(mg.values()), min(mg, key=mg.get)))
    return {k: v for k, v in zip(POSTURE_NUMBERS, x) if v != 0.0}, mg


ARM_GRID, ARM_FINE = 5, 1      # deg: the search for the start's free choices (pose): a grid, then a finer one about its best
ARM_CLEAR = 0.10               # m [E: 0.08-0.12, about an upper arm's thickness]: an elbow's centre stays this far from the other arm's bones


def _seg_dist(p, a, b):
    """distance from point p to the segment a-b."""
    ab = E.sub(b, a)
    t = max(0.0, min(1.0, E.dot(E.sub(p, a), ab) / E.dot(ab, ab)))
    return E.norm(E.sub(p, E.add(a, E.scale(ab, t))))
_ARM_CACHE = {}


def pose(mdl, t0, heel="solve", report=None, hands_guess=None, hands_solve=True, arms=None):
    """The body's posture at [4]'s mean state at time t0 (s before contact, negative), at rest:

    - feet a stride apart ([4]: 42 % of height between the ankle joints, the lead one 11 cm closed), the
      rear foot flat; pelvis and trunk at their measured rotations; the pelvis's travel as [4] gives it,
      its zero read as the lead ankle ([4] measures it from the centre of home plate and does not say
      where the feet are; the flat probe's reading, checked against [4]'s ground forces in body-motion 4.4);
    - the pelvis's height from the REAR knee's measured bend, and the lead knee's measured bend met by
      the lead heel's height (heel = "solve"): at foot down the lead foot is on its ball and the heel
      comes down as the weight comes on. heel = "flat" is session 1's placement (lead foot flat, the
      pelvis's height from the lead knee, the rear knee left to come out wrong);
    - the bat at its measured azimuth and elevation, the hands where both measured elbow bends and the
      measured lag put them.

    The hands' body (mdl.grip == "hands", body-motion 4.11): [4] does not give how the bat is rolled about its own
    length nor where each elbow points (each arm can swivel about the line from its shoulder to its wrist). The weld's
    body faces the bat's wrist side at the shoulders and points both elbows down and a little out. The hands' body
    sets the three by a rule: the arm joint nearest an end of its range (arm_range: both wrists' two angles, both
    forearms' turns, both upper arms' turns) is as far from it as can be (strain), searched on a grid and held in
    mdl.arm_choices (deg: the bat's roll from the weld's, the lead and rear swivels from the weld's). arms: those
    three given, not searched (place() gives the start's to the two neighbouring postures).

    What [4] does not give is the [E] list at the top. The joints are closed by the engine's projection
    (feet held, pelvis, trunk and bat hardly moved). Each sole point in contact starts as deep as [4]'s
    vertical load on that foot presses it, and with its friction spring stretched to carry [4]'s
    horizontal force. Nothing here looks at where the swing ends. Returns (state, friction anchors)."""
    if getattr(mdl, "shift", None) and not _SHIFT_ON[0]:
        with _shifted(mdl.shift, mdl.H):
            return pose(mdl, t0, heel, report, hands_guess, hands_solve, arms)
    s, b, H = mdl.sys, mdl.b, mdl.H
    st = s.new_state()
    L = mdl.len
    up = (0.0, 0.0, 1.0)
    stride = STRIDE * H
    BW = mdl.M * E.G
    reach_of = lambda deg: math.sqrt(L["thigh"] ** 2 + L["shank"] ** 2 + 2 * L["thigh"] * L["shank"] * math.cos(deg * D2R))
    ank0 = {"R": (0.0, -0.5 * stride, ANKLE_H * H), "L": (STRIDE_CLOSED, 0.5 * stride, ANKLE_H * H)}
    yaw = {"R": REAR_FOOT_YAW * D2R, "L": (90.0 - curve_lead_foot(t0) - LEAD_FOOT_SHIFT[0]) * D2R}
    fdir = {sd: (math.cos(yaw[sd]), math.sin(yaw[sd]), 0.0) for sd, sg in SIDES}
    load = {sd: max(0.0, curve(("lead" if sd == "L" else "trail") + "_grf_z", t0)) / 100.0 * BW for sd, sg in SIDES}
    cf_L = C.DE_LEVA["foot"][2]
    ball_loc = ((BALL_FWD - cf_L) * L["foot"], 0.0, mdl.sole_z)

    def foot(sd, pitch):
        """the foot with its heel raised by `pitch` on the ball of the foot, the ankle joint staying over
        its place on the ground ([4]'s stride is between the ankle joints). Flat (pitch 0) it sinks on
        four points, raised on the ball's two. Returns position, orientation, ankle joint."""
        fx = fdir[sd]
        q0 = E.axes_to_q(fx, E.cross(up, fx), up)
        q = E.qmul(q0, E.qaxis((0, 1, 0), pitch))
        Rf = E.qR(q)
        sink = load[sd] / ((4 if pitch == 0.0 else 2) * KN)
        da = E.mv(Rf, E.sub(mdl.ankle_f, ball_loc))            # from the ball's point to the ankle
        ankle = (ank0[sd][0], ank0[sd][1], -sink + da[2])
        return E.sub(ankle, E.mv(Rf, mdl.ankle_f)), q, ankle

    psi = curve("pelvis_rot", t0) * D2R
    obl = 0.0 if heel == "flat" else curve_obliquity(t0) * D2R
    qp = E.qmul(E.qaxis(up, psi), E.qaxis((1, 0, 0), obl))      # +: the trail hip toward the ground ([4])
    Rp = E.qR(qp)
    py = ank0["L"][1] + curve("pelvis_x_cm", t0) / 100.0 * H / 1.84
    hip_off = {sd: E.mv(Rp, mdl.jp["hip_" + sd][1]) for sd, sg in SIDES}
    knee_deg = {"L": curve("lead_knee_flex", t0), "R": curve_trail_knee(t0)}

    def pelvis_z(sd, ankle):
        """the pelvis's height that puts side sd's knee at its measured bend, the pelvis over (0, py)."""
        hx, hy = hip_off[sd][0], py + hip_off[sd][1]
        return ankle[2] + math.sqrt(max(0.0, reach_of(knee_deg[sd]) ** 2 - (hx - ankle[0]) ** 2 - (hy - ankle[1]) ** 2)) - hip_off[sd][2]

    flat = {sd: foot(sd, 0.0) for sd, sg in SIDES}
    zf = {sd: pelvis_z(sd, flat[sd][2]) for sd, sg in SIDES}
    pitch = {"L": 0.0, "R": 0.0}
    if heel == "flat":
        pz = zf["L"]
    else:
        # the foot whose flat placement asks for the higher pelvis stays flat; the other heel is raised
        # until its knee has its measured bend too (early: the lead heel, toe touch; late: the rear heel)
        low, high = ("L", "R") if zf["R"] >= zf["L"] else ("R", "L")
        pz = zf[high]
        hip = E.add((0.0, py, pz), hip_off[low])
        want = reach_of(knee_deg[low])
        gap = lambda th: E.norm(E.sub(hip, foot(low, th)[2])) - want     # > 0: the leg too long for its knee's bend
        if heel != "solve":
            pitch[low] = float(heel) * D2R
        elif gap(0.0) > 0.0:
            lo, hi = 0.0, HEEL_MAX * D2R
            if gap(hi) > 0.0:
                pitch[low] = hi
            else:
                for _ in range(60):
                    mid = 0.5 * (lo + hi)
                    lo, hi = (mid, hi) if gap(mid) > 0.0 else (lo, mid)
                pitch[low] = 0.5 * (lo + hi)
    xR, qR_, ankR = foot("R", pitch["R"])
    xL, qL, ankL = foot("L", pitch["L"])
    ank = {"R": ankR, "L": ankL}
    s.set_body(st, b["foot_R"], xR, qR_)
    s.set_body(st, b["foot_L"], xL, qL)
    xp = (0.0, py, pz)
    s.set_body(st, b["pelvis"], xp, qp)
    if report is not None:
        report["lead_heel_deg"] = pitch["L"] / D2R
        report["rear_heel_deg"] = pitch["R"] / D2R
        report["pelvis_z_from_each_knee_flat"] = (zf["L"], zf["R"])
    for sd, sg in SIDES:
        hip = E.add(xp, E.mv(Rp, mdl.jp["hip_" + sd][1]))
        knee = _two_link(hip, ank[sd], L["thigh"], L["shank"], fdir[sd])
        zt = E.unit(E.sub(hip, knee)); zs = E.unit(E.sub(knee, ank[sd]))
        ax = E.unit(E.cross(zt, zs))
        for body, z, top, c in (("thigh_", zt, hip, mdl.jp["hip_" + sd][3]), ("shank_", zs, knee, mdl.jp["knee_" + sd][3])):
            ex, ey, ez = _frame_z(z, ax)
            q = E.axes_to_q(ex, ey, ez)
            s.set_body(st, b[body + sd], E.sub(top, E.mv(E.qR(q), c)), q)
    # trunk: measured rotation, bent forward [E]
    qt = E.qmul(E.qmul(E.qaxis(up, curve("trunk_rot", t0) * D2R), E.qaxis((0, 1, 0), TRUNK_FLEX * D2R)), E.qaxis((1, 0, 0), TRUNK_SIDE * D2R))
    if TRUNK_TILT_IN_WORLD:      # the lean kept toward the plate while the trunk turns under it (a sensitivity case)
        qt = E.qmul(E.qaxis((0, 1, 0), TRUNK_FLEX * D2R), E.qaxis(up, curve("trunk_rot", t0) * D2R))
    Rt = E.qR(qt)
    lum = E.add(xp, E.mv(Rp, mdl.jp["lumbar"][1]))
    xt = E.sub(lum, E.mv(Rt, mdl.jp["lumbar"][3]))
    s.set_body(st, b["trunk"], xt, qt)
    sho = {sd: E.add(xt, E.mv(Rt, mdl.jp["shoulder_" + sd][1])) for sd, sg in SIDES}
    # bat: measured azimuth and elevation; the hands where the measured elbows and lag put them
    az, el = curve("bat_az", t0) * D2R, curve("bat_elev", t0) * D2R
    bz = (math.cos(el) * math.cos(az), math.cos(el) * math.sin(az), math.sin(el))
    lu, lf = L["upperarm"], L["forearm"]
    want = (curve("lead_elbow_flex", t0), curve("trail_elbow_flex", t0), curve("bat_lag", t0))

    hands_body = mdl.grip == "hands"
    roll = [0.0]                                   # the bat's roll about its own length from the weld's (rad): the hands' body only

    def hands(p):
        """bat frame, wrists and the three measured angles for the lead grip point at p."""
        mid = E.scale(E.add(sho["L"], sho["R"]), 0.5)
        toward = E.sub(mid, p)
        bx = E.unit(E.sub(E.scale(bz, E.dot(toward, bz)), toward))     # -x of the bat faces the shoulders
        by = E.cross(bz, bx)
        if roll[0] != 0.0:
            bx = E.add(E.scale(bx, math.cos(roll[0])), E.scale(by, math.sin(roll[0])))
            by = E.cross(bz, bx)
        qb = E.axes_to_q(bx, by, bz)
        Rb = E.qR(qb)
        xb = E.sub(p, E.mv(Rb, (0.0, 0.0, GRIP_L - mdl.bat_ucm)))
        wr = {sd: E.add(xb, E.mv(Rb, mdl.wrist_b[sd])) for sd, sg in SIDES}
        ang = []
        for sd in ("L", "R"):
            d = min(E.norm(E.sub(wr[sd], sho[sd])), 0.9999 * (lu + lf))
            ang.append(math.acos(max(-1.0, min(1.0, (d * d - lu * lu - lf * lf) / (2 * lu * lf)))) / D2R)
        ang.append(math.acos(max(-1.0, min(1.0, E.dot(E.unit(E.sub(wr["L"], sho["L"])), bz)))) / D2R)
        return xb, qb, wr, ang

    # the hands' point: three measured angles, three unknowns. Damped least squares (Levenberg-Marquardt), so
    # that where the three angles cannot all be met (the arms too short for them from these shoulders) the
    # nearest posture is taken and its misfit reported, instead of a failed solve. hands_guess (the lead grip's
    # point in the trunk's frame from the rear shoulder) continues a solution from a neighbouring time.
    p = E.add(sho["R"], E.mv(Rt, HANDS_GUESS if hands_guess is None else hands_guess))
    resid = lambda q_: [a_ - w_ for a_, w_ in zip(hands(q_)[3], want)]

    def elbow_of(sd, sg, wrist, swivel):
        """the elbow of arm sd: bent down and a little out (the weld's choice), turned `swivel` (rad) about the line
        from the shoulder to the wrist."""
        toward = E.add((0.0, 0.0, -1.0), E.scale(E.mv(Rt, (0, 1, 0)), 0.3 * sg))
        if swivel != 0.0:
            dh = E.unit(E.sub(wrist, sho[sd]))
            toward = E.mv(E.qR(E.qaxis(dh, swivel)), E.sub(toward, E.scale(dh, E.dot(toward, dh))))
        return _two_link(sho[sd], wrist, lu, lf, toward)

    swivel = {"L": 0.0, "R": 0.0}
    if hands_body:
        oval = getattr(mdl, "wrist", "box") == "oval"
        key = (repr(sho), repr(Rt), repr(bz), repr(want), GRIP_OBLIQUE, repr(GRIP_ROLL), repr(arm_range()), repr(p), ROM["sh_hadd"], ROM["sh_habd"], ARM_CLEAR,
               oval, WRIST_OVAL_TILT)
        if arms is None and key in _ARM_CACHE:
            arms = _ARM_CACHE[key]
        if arms is None:
            p_start = p

            def try_roll(deg, around=None):
                """(strain of the worse arm, of the other, roll, {side: swivel}) at this roll of the bat: the hands solved
                for [4]'s three angles, then the pair of swivels that strains the arms' joints least among those that
                keep each elbow ARM_CLEAR from the other arm (two arms cannot pass through each other)."""
                roll[0] = deg * D2R
                pk = _hands_lm(resid, p_start)
                rr = resid(pk)
                if max(abs(x) for x in rr) > 0.5:               # this roll cannot take [4]'s elbows and lag (deg)
                    return None
                xb_, qb_, wr_, _ = hands(pk)
                Rb_ = E.qR(qb_)
                cand = {}
                for sd, sg in SIDES:
                    rng_ = range(-180, 180, ARM_GRID) if around is None else range(around[sd] - ARM_GRID + 1, around[sd] + ARM_GRID, ARM_FINE)
                    cand[sd] = []
                    for sdeg in rng_:
                        el = elbow_of(sd, sg, wr_[sd], sdeg * D2R)
                        cand[sd].append((strain(arm_angles(sd, E.unit(E.sub(sho[sd], el)), E.unit(E.sub(el, wr_[sd])), Rb_, Rt), oval=oval), sdeg, el))
                best = None
                for cl, sl, el_l in cand["L"]:
                    for cr, sr, el_r in cand["R"]:
                        k_ = (max(cl, cr), min(cl, cr))
                        if best is not None and k_ >= best[0]:
                            continue
                        if min(_seg_dist(el_l, sho["R"], el_r), _seg_dist(el_l, el_r, wr_["R"]),
                               _seg_dist(el_r, sho["L"], el_l), _seg_dist(el_r, el_l, wr_["L"])) < ARM_CLEAR:
                            continue
                        best = (k_, sl, sr)
                if best is None:
                    return None
                return (best[0][0], best[0][1], deg, {"L": best[1], "R": best[2]})
            tried = [x for x in (try_roll(deg) for deg in range(-90, 91, ARM_GRID)) if x is not None]
            if not tried:
                raise E.SolveError("no roll of the bat takes [4]'s elbows and lag")
            best = min(tried, key=lambda x: (x[0], x[1], abs(x[2])))
            fine = [x for x in (try_roll(deg, best[3]) for deg in range(best[2] - ARM_GRID + 1, best[2] + ARM_GRID, ARM_FINE)) if x is not None]
            best = min(fine + [best], key=lambda x: (x[0], x[1], abs(x[2])))
            arms = (float(best[2]), float(best[3]["L"]), float(best[3]["R"]))
            _ARM_CACHE[key] = arms
        mdl.arm_choices = tuple(arms)
        roll[0] = arms[0] * D2R
        swivel = {"L": arms[1] * D2R, "R": arms[2] * D2R}
        if report is not None:
            report["arm_choices_deg"] = tuple(arms)
    r = resid(p)
    c0 = sum(x * x for x in r)
    lam = 1e-3
    for it in range(200 if hands_solve else 0):      # hands_solve = False: the hands exactly at hands_guess
        if c0 < 1e-20:
            break
        Jm = [[0.0] * 3 for _ in range(3)]
        for k in range(3):
            pk = list(p); pk[k] += 1e-6
            rk = resid(tuple(pk))
            for i in range(3):
                Jm[i][k] = (rk[i] - r[i]) / 1e-6
        JtJ = [[sum(Jm[i][a_] * Jm[i][b_] for i in range(3)) for b_ in range(3)] for a_ in range(3)]
        g = [sum(Jm[i][a_] * r[i] for i in range(3)) for a_ in range(3)]
        moved = False
        for _ in range(30):
            A = [[JtJ[a_][b_] + (lam * (JtJ[a_][a_] + 1e-9) if a_ == b_ else 0.0) for b_ in range(3)] for a_ in range(3)]
            try:
                dp = _solve3(A, [-x for x in g])
            except ZeroDivisionError:
                lam *= 10.0
                continue
            n = E.norm(dp)
            if n > 0.05:
                dp = E.scale(dp, 0.05 / n)
            pn = E.add(p, dp)
            rn = resid(pn)
            cn = sum(x * x for x in rn)
            if cn < c0:
                moved = n > 1e-12 and (c0 - cn) > 1e-14 * c0
                p, r, c0 = pn, rn, cn
                lam = max(1e-9, lam / 5.0)
                break
            lam *= 5.0
        if not moved:
            break
    xb, qb, wr, ang = hands(p)
    if report is not None:
        report["hands_solve_residual_deg"] = [ang[i] - want[i] for i in range(3)]
        report["hands_from_rear_shoulder"] = E.mtv(Rt, E.sub(p, sho["R"]))
    s.set_body(st, b["bat"], xb, qb)
    by = E.mv(E.qR(qb), (0, 1, 0))
    for sd, sg in SIDES:
        if hands_body:
            elbow = elbow_of(sd, sg, wr[sd], swivel[sd])
        else:
            elbow = _two_link(sho[sd], wr[sd], lu, lf, E.add((0.0, 0.0, -1.0), E.scale(E.mv(Rt, (0, 1, 0)), 0.3 * sg)))
        zu = E.unit(E.sub(sho[sd], elbow)); zf = E.unit(E.sub(elbow, wr[sd]))
        ex, ey, ez = _frame_z(zu, E.unit(E.cross(zu, zf)))
        q = E.axes_to_q(ex, ey, ez)
        s.set_body(st, b["uarm_" + sd], E.sub(sho[sd], E.mv(E.qR(q), mdl.jp["shoulder_" + sd][3])), q)
        fxa = E.unit(E.cross(by, zf))             # the wrist's flexion axis, square to the cock's axis
        if hands_body:                            # the forearm's radial line (toward the thumb), square to the hand's p
            fxa = E.scale(E.unit(E.cross(E.mv(E.qR(qb), mdl.palm[sd]), zf)), -sg)
        q = E.axes_to_q(fxa, E.cross(zf, fxa), zf)
        s.set_body(st, b["farm_" + sd], E.sub(elbow, E.mv(E.qR(q), mdl.jp["elbow_" + sd][3])), q)
    # close the joints: feet held, pelvis, trunk and bat a thousand times harder to move than the limbs
    lin = [1.0 / m for m in s.m]
    ang_w = [1.0] * s.nb
    for n, i in b.items():
        if n.startswith("foot"):
            lin[i] = ang_w[i] = 0.0
        elif n in ("pelvis", "trunk", "bat"):
            lin[i] *= 1e-3; ang_w[i] = 1e-3
    before = mdl.observe(st)
    st = s.project(st, lin, ang_w)
    # friction anchors: each sole point in contact carries its share of [4]'s horizontal force on that foot
    # ([4]'s GRFx is toward the pitcher, world +Y; its GRFy toward home plate, world +X: body-motion 5.2)
    aux = s.new_aux()
    s.anchors(st, aux)
    for sd, nm in (("L", "lead"), ("R", "trail")):
        idx = [j for j, c in enumerate(s.contacts) if c[0][0] == sd and aux[j] is not None]
        if not idx:
            continue
        fx = curve(nm + "_grf_y", t0) / 100.0 * BW / len(idx)
        fy = curve(nm + "_grf_x", t0) / 100.0 * BW / len(idx)
        lim = MU_START * load[sd] / len(idx)
        f = math.hypot(fx, fy)
        if f > lim > 0.0:
            fx, fy = fx * lim / f, fy * lim / f
        for j in idx:
            aux[j] = (aux[j][0] + fx / KT, aux[j][1] + fy / KT)
    if report is not None:
        after = mdl.observe(st)
        report["placed"] = after
        report["moved_by_closing"] = {k: after[k] - before[k] for k in after}
        report["measured"] = {k: curve(k, t0) for k in after
                              if k in ("pelvis_rot", "trunk_rot", "bat_az", "bat_elev", "bat_lag", "lead_elbow_flex",
                                       "trail_elbow_flex", "lead_knee_flex", "pelvis_rot_vel", "trunk_rot_vel", "bat_az_vel")}
        report["measured"]["trail_knee_flex"] = curve_trail_knee(t0)
    return st, aux


def place(mdl, t0, report=None, heel="solve", dt=VEL_DT, moving=True):
    """The body at [4]'s mean state at t0, MOVING: the posture of pose(t0), and every segment's velocity
    from the postures a moment before and after (pose(t0 - dt), pose(t0 + dt)), so each velocity is the
    slope of the same curves of [4] the posture is built on: the pelvis's travel and rotation, the trunk's
    rotation, the bat's azimuth and elevation, the lag, both elbows, both knees, the lead heel coming
    down. What is held fixed in the posture ([E]: the trunk's forward bend, the pelvis's tilt, the feet's
    places) has no rate. The velocities are then made exactly consistent with the joints (least change).
    moving = False starts at rest (a sensitivity case). The friction anchors are left in mdl.start_aux.
    Returns the state."""
    s = mdl.sys
    st, aux = pose(mdl, t0, heel, report)
    mdl.start_aux = aux
    if not moving:
        return st
    held = getattr(mdl, "arm_choices", None) if mdl.grip == "hands" else None    # the start's free choices, held for its neighbours
    sm, _ = pose(mdl, t0 - dt, heel, arms=held)
    sp, _ = pose(mdl, t0 + dt, heel, arms=held)
    if held is not None:
        mdl.arm_choices = held
    for i in range(s.nb):
        o = 13 * i
        for c in range(3):
            st[o + 7 + c] = (sp[o + c] - sm[o + c]) / (2 * dt)
        dq = E.qmul(tuple(sp[o + 3:o + 7]), E.qconj(tuple(sm[o + 3:o + 7])))
        if dq[0] < 0:
            dq = tuple(-c for c in dq)
        sn = math.sqrt(dq[1] ** 2 + dq[2] ** 2 + dq[3] ** 2)
        k = 2.0 * math.atan2(sn, dq[0]) / sn / (2 * dt) if sn > 1e-14 else 0.0
        st[o + 10:o + 13] = [dq[1] * k, dq[2] * k, dq[3] * k]
    X, R, V, Wv = s._kin(st)
    rows = s._rows(X, R, V, Wv)
    lin = [1.0 / m for m in s.m]
    A, per = s._assemble(rows, R, lin, [1.0] * s.nb)
    mu_ = E.chol_solve(A, [-r[5] for r in rows])
    for i, lst in enumerate(per):
        o = 13 * i + 7
        for k, Jk, Yk in lst:
            for c in range(6):
                st[o + c] += mu_[k] * Yk[c]
    if report is not None:
        report["placed"] = mdl.observe(st)
    return st


def com(mdl, st):
    """the whole body's centre of mass (with the bat): position and velocity."""
    s = mdl.sys
    M = sum(s.m)
    x = [sum(s.m[i] * st[13 * i + c] for i in range(s.nb)) / M for c in range(3)]
    v = [sum(s.m[i] * st[13 * i + 7 + c] for i in range(s.nb)) / M for c in range(3)]
    return x, v


OBLIQUITY = ((-0.170, 1.0), (-0.093, 14.0), (0.0, 27.0))


def curve_obliquity(t0):
    """[4]'s pelvis obliquity from its text (trail hip toward the ground): 1 deg at -170 ms, 14 at -93,
    27 at contact; before -170 ms the text gives none and it is held at 1 [E: range -10 to +5]."""
    pts = OBLIQUITY
    if t0 <= pts[0][0]:
        return pts[0][1]
    for (ta, va), (tb, vb) in zip(pts, pts[1:]):
        if t0 <= tb:
            return va + (vb - va) * (t0 - ta) / (tb - ta)
    return pts[-1][1]


TRAIL_KNEE = ((-0.621, 56.0), (-0.340, 50.0), (-0.220, 44.0), (-0.170, 47.0), (-0.093, 54.0), (0.0, 54.0))


def curve_trail_knee(t0):
    """[4]'s trail knee bend from its text (no digitised curve): 56 deg at foot off, 50 at foot down, 44
    (its straightest) at -220 ms, 47 at -170, 54 at -93; held at 54 to contact [E: the text gives none]."""
    pts = TRAIL_KNEE
    if t0 <= pts[0][0]:
        return pts[0][1]
    for (ta, va), (tb, vb) in zip(pts, pts[1:]):
        if t0 <= tb:
            return va + (vb - va) * (t0 - ta) / (tb - ta)
    return pts[-1][1]


def curve_lead_foot(t0):
    """[4]'s lead foot angle to its X axis (toward the pitcher): 63 deg at foot down, 56 by the end of
    coiling (text values; no digitised curve), interpolated between -340 and -170 ms."""
    f = min(1.0, max(0.0, (t0 + 0.340) / 0.170))
    return 63.0 + (56.0 - 63.0) * f


def _hands_lm(resid, p):
    """pose()'s damped least squares for the hands' point, as a function (the hands' body tries it at many rolls of the
    bat): the point nearest [4]'s three angles from the guess p."""
    r = resid(p)
    c0 = sum(x * x for x in r)
    lam = 1e-3
    for it in range(200):
        if c0 < 1e-20:
            break
        Jm = [[0.0] * 3 for _ in range(3)]
        for k in range(3):
            pk = list(p); pk[k] += 1e-6
            rk = resid(tuple(pk))
            for i in range(3):
                Jm[i][k] = (rk[i] - r[i]) / 1e-6
        JtJ = [[sum(Jm[i][a_] * Jm[i][b_] for i in range(3)) for b_ in range(3)] for a_ in range(3)]
        g = [sum(Jm[i][a_] * r[i] for i in range(3)) for a_ in range(3)]
        moved = False
        for _ in range(30):
            A = [[JtJ[a_][b_] + (lam * (JtJ[a_][a_] + 1e-9) if a_ == b_ else 0.0) for b_ in range(3)] for a_ in range(3)]
            try:
                dp = _solve3(A, [-x for x in g])
            except ZeroDivisionError:
                lam *= 10.0
                continue
            n = E.norm(dp)
            if n > 0.05:
                dp = E.scale(dp, 0.05 / n)
            pn = E.add(p, dp)
            rn = resid(pn)
            cn = sum(x * x for x in rn)
            if cn < c0:
                moved = n > 1e-12 and (c0 - cn) > 1e-14 * c0
                p, r, c0 = pn, rn, cn
                lam = max(1e-9, lam / 5.0)
                break
            lam *= 5.0
        if not moved:
            break
    return p


def _solve3(A, r):
    M = [list(A[i]) + [r[i]] for i in range(3)]
    for c in range(3):
        p = max(range(c, 3), key=lambda k: abs(M[k][c]))
        M[c], M[p] = M[p], M[c]
        for k in range(c + 1, 3):
            f = M[k][c] / M[c][c]
            for j in range(c, 4):
                M[k][j] -= f * M[c][j]
    x = [0.0] * 3
    for c in (2, 1, 0):
        x[c] = (M[c][3] - sum(M[c][j] * x[j] for j in range(c + 1, 3))) / M[c][c]
    return tuple(x)
