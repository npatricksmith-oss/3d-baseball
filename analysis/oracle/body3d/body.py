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
REAR_FOOT_YAW = 0.0       # deg from +X: the rear foot square to the plate
STRIDE = 0.42             # x H between the ankles ([4]: 42 % of height, toe to toe; measured)
STRIDE_CLOSED = 0.11      # m, lead foot toward the plate ([4], measured)
HEEL_MAX = 60.0           # deg [E] the most a heel can be raised on the ball of the foot
MU_START = 0.8            # the starting horizontal force on a foot is kept inside this friction cone
VEL_DT = 0.010            # s: the starting velocities are slopes of [4]'s curves over +-10 ms [E: 3, 5, 10 tried, body-motion 4.4]
HANDS_GUESS = (0.10, -0.05, 0.02)   # m, first guess of the hands from the rear shoulder (trunk frame); the solve moves it

SIDES = (("L", 1.0), ("R", -1.0))


def curve(name, t):
    """[4]'s digitised mean curve at time t (s), by linear interpolation."""
    rows = _CURVES.get(name)
    if rows is None:
        rows = _CURVES[name] = C.load(name)
    return C.resample(rows, t, t, 1.0)[0]


_CURVES = {}


def curve_rate(name, t, h=0.005):
    return (curve(name, t + h) - curve(name, t - h)) / (2 * h)


class Model:
    """the engine System of a batter of stature H (m) and mass M (kg), with named bodies and joint points."""

    def __init__(self, H=1.84, M=92.0, contacts=True, mu=MU, shoulder_stops=True):
        self.H, self.M = H, M
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
            s.universal(*_j(J["wrist_" + sd]), (1, 0, 0), (0, 1, 0))       # flexion (forearm's x), the cock (bat's y)
            # no hyperextension: one-sided stops on the knee's and elbow's bend (place() makes each joint's own
            # +y its flexion axis, so the angle from the upper segment's z to the lower one's about +y is the bend)
            s.stop("knee_" + sd, th, sh, (0, 1, 0), (0, 0, 1), (0, 0, 1), 0.0, FLEX_MAX * D2R, STOP_K, STOP_C)
            s.stop("elbow_" + sd, ua, fa, (0, 1, 0), (0, 0, 1), (0, 0, 1), 0.0, FLEX_MAX * D2R, STOP_K, STOP_C)
            if shoulder_stops:
                # the upper arm cannot be brought further across the chest than its end of range: the angle,
                # seen down the trunk's axis, from straight out to its own side round to the front and across
                s.stop("shoulder_" + sd, b["trunk"], ua, (0, 0, -sg), (0, sg, 0), (0, 0, -1), -10.0, SH_ADD_MAX * D2R, STOP_K, STOP_C)
            if contacts:
                for nm, px in (("heel", -cf * Lf), ("ball", (BALL_FWD - cf) * Lf)):
                    for wn, py in (("in", -sg * SOLE_HALF_W), ("out", sg * SOLE_HALF_W)):
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
        o["sweet_speed"] = E.norm(self.bat_point(st, C.BAT_SS)[1])
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


def pose(mdl, t0, heel="solve", report=None):
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

    What [4] does not give is the [E] list at the top. The joints are closed by the engine's projection
    (feet held, pelvis, trunk and bat hardly moved). Each sole point in contact starts as deep as [4]'s
    vertical load on that foot presses it, and with its friction spring stretched to carry [4]'s
    horizontal force. Nothing here looks at where the swing ends. Returns (state, friction anchors)."""
    s, b, H = mdl.sys, mdl.b, mdl.H
    st = s.new_state()
    L = mdl.len
    up = (0.0, 0.0, 1.0)
    stride = STRIDE * H
    BW = mdl.M * E.G
    reach_of = lambda deg: math.sqrt(L["thigh"] ** 2 + L["shank"] ** 2 + 2 * L["thigh"] * L["shank"] * math.cos(deg * D2R))
    ank0 = {"R": (0.0, -0.5 * stride, ANKLE_H * H), "L": (STRIDE_CLOSED, 0.5 * stride, ANKLE_H * H)}
    yaw = {"R": REAR_FOOT_YAW * D2R, "L": (90.0 - curve_lead_foot(t0)) * D2R}
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
    qt = E.qmul(E.qaxis(up, curve("trunk_rot", t0) * D2R), E.qaxis((0, 1, 0), TRUNK_FLEX * D2R))
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

    def hands(p):
        """bat frame, wrists and the three measured angles for the lead grip point at p."""
        mid = E.scale(E.add(sho["L"], sho["R"]), 0.5)
        toward = E.sub(mid, p)
        bx = E.unit(E.sub(E.scale(bz, E.dot(toward, bz)), toward))     # -x of the bat faces the shoulders
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

    p = E.add(sho["R"], E.mv(Rt, HANDS_GUESS))
    for it in range(60):                          # Newton on the hands' point: three angles, three unknowns
        _, _, _, a0 = hands(p)
        r = [a0[i] - want[i] for i in range(3)]
        if max(abs(c) for c in r) < 1e-9:
            break
        Jm = [[0.0] * 3 for _ in range(3)]
        for k in range(3):
            pk = list(p); pk[k] += 1e-6
            ak = hands(tuple(pk))[3]
            for i in range(3):
                Jm[i][k] = (ak[i] - a0[i]) / 1e-6
        try:
            dp = _solve3(Jm, [-c for c in r])
        except ZeroDivisionError:
            break
        n = E.norm(dp)
        if n > 0.05:
            dp = E.scale(dp, 0.05 / n)
        p = E.add(p, dp)
    xb, qb, wr, ang = hands(p)
    if report is not None:
        report["hands_solve_residual_deg"] = [ang[i] - want[i] for i in range(3)]
        report["hands_from_rear_shoulder"] = E.mtv(Rt, E.sub(p, sho["R"]))
    s.set_body(st, b["bat"], xb, qb)
    by = E.mv(E.qR(qb), (0, 1, 0))
    for sd, sg in SIDES:
        elbow = _two_link(sho[sd], wr[sd], lu, lf, E.add((0.0, 0.0, -1.0), E.scale(E.mv(Rt, (0, 1, 0)), 0.3 * sg)))
        zu = E.unit(E.sub(sho[sd], elbow)); zf = E.unit(E.sub(elbow, wr[sd]))
        ex, ey, ez = _frame_z(zu, E.unit(E.cross(zu, zf)))
        q = E.axes_to_q(ex, ey, ez)
        s.set_body(st, b["uarm_" + sd], E.sub(sho[sd], E.mv(E.qR(q), mdl.jp["shoulder_" + sd][3])), q)
        fxa = E.unit(E.cross(by, zf))             # the wrist's flexion axis, square to the cock's axis
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
    sm, _ = pose(mdl, t0 - dt, heel)
    sp, _ = pose(mdl, t0 + dt, heel)
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
