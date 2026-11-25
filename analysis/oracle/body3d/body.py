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
FLEX_MAX = 150.0          # deg [E] knee's and elbow's full bend; 0 = straight, no hyperextension
STOP_K, STOP_C = 2000.0, 20.0   # [E] N m/rad, N m s/rad: a joint's end-of-range stop (one-sided, never pulls)
# posture [4] does not give, [E]
TRUNK_FLEX = 15.0         # deg, the trunk bent forward over the pelvis
REAR_FOOT_YAW = 0.0       # deg from +X: the rear foot square to the plate
STRIDE = 0.42             # x H between the ankles ([4]: 42 % of height, toe to toe; measured)
STRIDE_CLOSED = 0.11      # m, lead foot toward the plate ([4], measured)
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

    def __init__(self, H=1.84, M=92.0, contacts=True, mu=MU):
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


def place(mdl, t0, report=None):
    """The body at [4]'s mean state at time t0 (s before contact, negative): feet flat on the ground a
    stride apart, pelvis and trunk at their measured rotations and the pelvis at its measured travel,
    the lead knee at its measured bend, the bat at its measured azimuth and elevation, the hands where
    both measured elbow bends and the measured lag put them. What [4] does not give is the [E] list at
    the top. The joints are then closed by the engine's projection (feet held, pelvis, trunk and bat
    hardly moved) and the velocities set from [4]'s rates the same way. Nothing here looks at where the
    swing ends. Returns the state; `report` (a dict) receives placed-against-measured angles."""
    s, b, H = mdl.sys, mdl.b, mdl.H
    st = s.new_state()
    L = mdl.len
    up = (0.0, 0.0, 1.0)
    stride = STRIDE * H
    # feet
    ank = {"R": (0.0, -0.5 * stride, ANKLE_H * H), "L": (STRIDE_CLOSED, 0.5 * stride, ANKLE_H * H)}
    yaw = {"R": REAR_FOOT_YAW * D2R, "L": (90.0 - curve_lead_foot(t0)) * D2R}
    fdir = {}
    BW = mdl.M * E.G
    for sd, sg in SIDES:
        fx = (math.cos(yaw[sd]), math.sin(yaw[sd]), 0.0)
        fdir[sd] = fx
        q = E.axes_to_q(fx, E.cross(up, fx), up)
        load = max(0.0, curve(("lead" if sd == "L" else "trail") + "_grf_z", t0)) / 100.0 * BW
        sink = load / (4 * KN)                      # the sole starts as deep as [4]'s vertical load presses it
        x = E.sub(ank[sd], E.mv(E.qR(q), mdl.ankle_f))
        x = (x[0], x[1], -mdl.sole_z - sink)
        ank[sd] = E.add(x, E.mv(E.qR(q), mdl.ankle_f))
        s.set_body(st, b["foot_" + sd], x, q)
    # pelvis: measured rotation and travel ([4]'s zero is at the lead ankle: the flat probe's reading, 4.2);
    # its height from the lead knee's measured bend; set back so the hips are over the ankles' line [E]
    psi = curve("pelvis_rot", t0) * D2R
    qp = E.qaxis(up, psi)
    a_, ra, _, rb = mdl.jp["hip_L"]
    hipL_loc = ra
    flex = curve("lead_knee_flex", t0) * D2R
    reach = math.sqrt(L["thigh"] ** 2 + L["shank"] ** 2 + 2 * L["thigh"] * L["shank"] * math.cos(flex))
    py = ank["L"][1] + curve("pelvis_x_cm", t0) / 100.0 * H / 1.84
    off = E.mv(E.qR(qp), hipL_loc)
    hx, hy = off[0], py + off[1]
    dz = math.sqrt(max(0.0, reach ** 2 - (hx - ank["L"][0]) ** 2 - (hy - ank["L"][1]) ** 2))
    xp = (0.0, py, ank["L"][2] + dz - off[2])
    s.set_body(st, b["pelvis"], xp, qp)
    Rp = E.qR(qp)
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
    # velocities from [4]'s rates: pelvis and trunk about the vertical, the pelvis forward, the bat's azimuth
    o = 13 * b["pelvis"]
    st[o + 8] = curve_rate("pelvis_x_cm", t0) / 100.0
    st[o + 12] = curve("pelvis_rot_vel", t0) * D2R
    st[13 * b["trunk"] + 12] = curve("trunk_rot_vel", t0) * D2R
    st[13 * b["trunk"] + 8] = st[o + 8]
    st[13 * b["bat"] + 12] = curve("bat_az_vel", t0) * D2R
    st[13 * b["bat"] + 8] = st[o + 8]
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
    if report is not None:
        after = mdl.observe(st)
        report["placed"] = after
        report["moved_by_closing"] = {k: after[k] - before[k] for k in after}
        report["measured"] = {k: curve(k, t0) for k in after
                              if k in ("pelvis_rot", "trunk_rot", "bat_az", "bat_elev", "bat_lag", "lead_elbow_flex",
                                       "trail_elbow_flex", "lead_knee_flex", "pelvis_rot_vel", "trunk_rot_vel", "bat_az_vel")}
    return st


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
