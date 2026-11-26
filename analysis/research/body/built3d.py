"""A swing built by hand and solved for its joint torques: the test of whether the body can swing, without a search
(docs/body-motion.md 4.12, part 2; PLAN D41).

    uv run --with numpy --with scipy python3 analysis/research/body/built3d.py --check        the solve's own checks
    uv run --with numpy --with scipy python3 analysis/research/body/built3d.py --base         the base motion, instant by instant
    uv run --with numpy --with scipy python3 analysis/research/body/built3d.py --table        every motion of 4.12's table (job queue)

THIS IS A DIAGNOSIS. The motion is no target, nothing is fitted to it and no score reads it (PLAN D37). It is built
from joint ranges, the aim's ball and stated time laws: 20 posture numbers, each on a cubic in time from the cleared
start (body.START_CLEAR: [4]'s launch posture moved by a rule, FLAGGED) to a hitter's contact posture with the sweet
spot passing through the aim's ball along the ball's line at a stated speed. Nothing here reads [4]'s curves but the
start.

The solve is the engine's own equations (as inverse3d.py): every SAMPLE the body is placed on the motion, velocities
and accelerations are differences of postures made consistent with the joints, and the loads that give those
accelerations are found: a torque about each of the 29 joint axes muscles act about, and a force at each sole point on
the ground (pushing only, inside the grip's cone). The arms and the bat are a ring and the two feet another, so twelve
numbers are free: they are chosen to make the hardest-worked muscle group's share of its strength AT THE SPEED IT IS
MOVING as small as it can be (a linear programme). Directions no muscle of this body acts in are left free and then
made as small as they can be; they are reported in N m.

What it cannot see: whether muscles with rise times and tendon springs can produce those torques in time; balance
after contact; whether the practice search can find the motion.
"""
import json
import math
import multiprocessing
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "..", "oracle"))
from body3d import engine as E, body as B, drive as Dr   # noqa: E402
import inverse3d as I3                                   # noqa: E402  (velocities and the consistency step only)

D2R = B.D2R
C = B.C
H, M = 1.84, 92.0
T0 = -0.250                      # the placed start's label on [4]'s clock; the motion's own clock starts at 0
OUT = os.path.join(HERE, "..", "..", "..", "sim", "target", "codex-probes", "body3d", "s10")
# ---- the aim's ball (practice3d.py, body-motion 4.10): nothing of [4]'s but the start it is measured from
BALL_DESCENT, BALL_ACROSS, BALL_Z, MEET_ACROSS, MEET_AHEAD = 5.27, 1.62, 0.78, 0.92, 0.72
# ---- the contact posture [E], each with its range (4.12)
CONTACT = {"pelvis_yaw": 75.0,       # deg from facing the plate [E: 60-90]
           "trunk_ahead": 15.0,      # deg the trunk is turned further than the pelvis [E]
           "trunk_flex": 30.0, "trunk_side": 20.0,     # deg [E: 25-40, 15-30; hitters' side tilt at contact 32, OpenBiomechanics]
           "shift": 0.10,            # m the pelvis is carried toward the pitcher [E: 0-0.17]
           "lead_knee": 14.0, "rear_knee": 70.0,       # deg (OpenBiomechanics 14 +- 12; [E: 55-90])
           "rear_foot_yaw": 60.0,    # deg the rear foot has turned toward the pitcher, on its ball [E]
           "bat_elev": -20.0,        # deg: the barrel below the hands [E: -10 to -30]
           "hands_share": 0.20}      # of the sweet spot's speed carried by the hands' travel [E: 0.15-0.30]
END_RATE = {"pelvis_yaw": 0.5, "trunk_yaw": 1.0}       # a number's rate at contact over its average rate [E]; the others: END_RATE_REST
END_RATE_REST = 0.5
AFTER = 0.020                    # s the laws run on after contact
SAMPLE, DIFF = 0.005, 0.002      # s: an instant every SAMPLE; differences of postures DIFF apart
MU = B.MU_CLEAT
CONE_SIDES = 8
ON_GROUND = 0.002                # m: a sole point this near the ground can carry load
# a measured strength for scale beside the trunk's bending, which this body has no muscle for (note 12 C1: Schultz 1987,
# 10 men standing, isometric about L3; mean + 2 SD, scaled by weight x height as 4.5 scales): FOR SCALE ONLY
_K = (M * H) / (71.0 * 1.78)
TRUNK_BEND_SCALE = {"forward": (191.0 + 2 * 70.0) * _K, "back": (204.0 + 2 * 78.0) * _K, "side": (164.0 + 2 * 56.0) * _K}
NUMBERS = ("pelvis_x", "pelvis_y", "pelvis_z", "pelvis_yaw", "pelvis_obl", "foot_yaw_L", "foot_yaw_R", "foot_pitch_L", "foot_pitch_R",
           "trunk_yaw", "trunk_flex", "trunk_side", "bat_az", "bat_elev", "bat_roll", "hand_x", "hand_y", "hand_z", "swivel_L", "swivel_R")
GROUP = {"legs": ("pelvis_x", "pelvis_y", "pelvis_z", "pelvis_yaw", "pelvis_obl", "foot_yaw_L", "foot_yaw_R", "foot_pitch_L", "foot_pitch_R"),
         "trunk": ("trunk_yaw", "trunk_flex", "trunk_side"),
         "arms": ("bat_az", "bat_elev", "bat_roll", "hand_x", "hand_y", "hand_z", "swivel_L", "swivel_R")}
UP = (0.0, 0.0, 1.0)


def new_model(contacts=False):
    m = B.Model(H, M, ranges=(), grip="hands", feet="cleats", wrist="oval", contacts=contacts)
    return m


# ---------------------------------------------------------------------------------------------------------------
# a posture from its 20 numbers
def ball_mid(m):
    """the midpoint of a foot's two forefoot points, in the foot's own frame."""
    cf = C.DE_LEVA["foot"][2]
    return ((B.BALL_FWD - cf) * m.len["foot"], 0.0, m.sole_z)


def frames(m, q, ball):
    """the pelvis's, trunk's and feet's poses for the numbers q (a dict): {body: (position, quaternion)}, the shoulders
    and the hips. ball: {side: the ground point its forefoot stands on}."""
    out = {}
    qp = E.qmul(E.qaxis(UP, q["pelvis_yaw"] * D2R), E.qaxis((1, 0, 0), q["pelvis_obl"] * D2R))
    xp = (q["pelvis_x"], q["pelvis_y"], q["pelvis_z"])
    Rp = E.qR(qp)
    out["pelvis"] = (xp, qp)
    qt = E.qmul(E.qmul(E.qaxis(UP, q["trunk_yaw"] * D2R), E.qaxis((0, 1, 0), q["trunk_flex"] * D2R)), E.qaxis((1, 0, 0), q["trunk_side"] * D2R))
    Rt = E.qR(qt)
    lum = E.add(xp, E.mv(Rp, m.jp["lumbar"][1]))
    xt = E.sub(lum, E.mv(Rt, m.jp["lumbar"][3]))
    out["trunk"] = (xt, qt)
    bm = ball_mid(m)
    for sd in "LR":
        yaw = q["foot_yaw_" + sd] * D2R
        fx = (math.cos(yaw), math.sin(yaw), 0.0)
        qf = E.qmul(E.axes_to_q(fx, E.cross(UP, fx), UP), E.qaxis((0, 1, 0), q["foot_pitch_" + sd] * D2R))
        out["foot_" + sd] = (E.sub(ball[sd], E.mv(E.qR(qf), bm)), qf)
    sho = {sd: E.add(xt, E.mv(Rt, m.jp["shoulder_" + sd][1])) for sd in "LR"}
    hip = {sd: E.add(xp, E.mv(Rp, m.jp["hip_" + sd][1])) for sd in "LR"}
    return out, sho, hip


def bat_frame(m, q, sho, Rt):
    """(bat position, quaternion, {side: wrist}, lead grip point) for the numbers q: the bat's direction from its
    azimuth and elevation, its wrist side facing the shoulders and then rolled (as body.pose builds it), the lead
    hand's grip point where the trunk carries it (from the rear shoulder, in the trunk's frame)."""
    az, el = q["bat_az"] * D2R, q["bat_elev"] * D2R
    bz = (math.cos(el) * math.cos(az), math.cos(el) * math.sin(az), math.sin(el))
    p = E.add(sho["R"], E.mv(Rt, (q["hand_x"], q["hand_y"], q["hand_z"])))
    mid = E.scale(E.add(sho["L"], sho["R"]), 0.5)
    toward = E.sub(mid, p)
    bx = E.unit(E.sub(E.scale(bz, E.dot(toward, bz)), toward))
    by = E.cross(bz, bx)
    r = q["bat_roll"] * D2R
    bx = E.add(E.scale(bx, math.cos(r)), E.scale(by, math.sin(r)))
    by = E.cross(bz, bx)
    qb = E.axes_to_q(bx, by, bz)
    Rb = E.qR(qb)
    xb = E.sub(p, E.mv(Rb, (0.0, 0.0, B.GRIP_L - m.bat_ucm)))
    wr = {sd: E.add(xb, E.mv(Rb, m.wrist_b[sd])) for sd in "LR"}
    return xb, qb, wr, p


def elbow_of(m, sd, sho, wrist, Rt, swivel_deg):
    """body.pose's elbow: bent down and a little out, turned `swivel` about the line from the shoulder to the wrist."""
    sg = 1.0 if sd == "L" else -1.0
    toward = E.add((0.0, 0.0, -1.0), E.scale(E.mv(Rt, (0, 1, 0)), 0.3 * sg))
    dh = E.unit(E.sub(wrist, sho))
    toward = E.mv(E.qR(E.qaxis(dh, swivel_deg * D2R)), E.sub(toward, E.scale(dh, E.dot(toward, dh))))
    return B._two_link(sho, wrist, m.len["upperarm"], m.len["forearm"], toward)


def place(m, q, ball):
    """the body's state (at rest) in the posture of the 20 numbers q, its joints closed by the engine's projection with
    the feet held and the pelvis, trunk and bat a thousand times harder to move than the limbs (as body.pose)."""
    s, b = m.sys, m.b
    st = s.new_state()
    fr, sho, hip = frames(m, q, ball)
    for name, (x, qq) in fr.items():
        s.set_body(st, b[name], x, qq)
    Rt = E.qR(fr["trunk"][1])
    for sd in "LR":
        ank = E.add(fr["foot_" + sd][0], E.mv(E.qR(fr["foot_" + sd][1]), m.ankle_f))
        fdir = E.mv(E.qR(fr["foot_" + sd][1]), (1, 0, 0))
        knee = B._two_link(hip[sd], ank, m.len["thigh"], m.len["shank"], fdir)
        zt = E.unit(E.sub(hip[sd], knee)); zs = E.unit(E.sub(knee, ank))
        ax = E.unit(E.cross(zt, zs))
        for body, z, top, c in (("thigh_", zt, hip[sd], m.jp["hip_" + sd][3]), ("shank_", zs, knee, m.jp["knee_" + sd][3])):
            ex, ey, ez = B._frame_z(z, ax)
            qq = E.axes_to_q(ex, ey, ez)
            s.set_body(st, b[body + sd], E.sub(top, E.mv(E.qR(qq), c)), qq)
    xb, qb, wr, p = bat_frame(m, q, sho, Rt)
    s.set_body(st, b["bat"], xb, qb)
    Rb = E.qR(qb)
    for sd, sg in B.SIDES:
        elbow = elbow_of(m, sd, sho[sd], wr[sd], Rt, q["swivel_" + sd])
        zu = E.unit(E.sub(sho[sd], elbow)); zf = E.unit(E.sub(elbow, wr[sd]))
        ex, ey, ez = B._frame_z(zu, E.unit(E.cross(zu, zf)))
        qq = E.axes_to_q(ex, ey, ez)
        s.set_body(st, b["uarm_" + sd], E.sub(sho[sd], E.mv(E.qR(qq), m.jp["shoulder_" + sd][3])), qq)
        fxa = E.scale(E.unit(E.cross(E.mv(Rb, m.palm[sd]), zf)), -sg)
        qq = E.axes_to_q(fxa, E.cross(zf, fxa), zf)
        s.set_body(st, b["farm_" + sd], E.sub(elbow, E.mv(E.qR(qq), m.jp["elbow_" + sd][3])), qq)
    lin = [1.0 / mm for mm in s.m]
    ang = [1.0] * s.nb
    for n, i in b.items():
        if n.startswith("foot"):
            lin[i] = ang[i] = 0.0
        elif n in ("pelvis", "trunk", "bat"):
            lin[i] *= 1e-3; ang[i] = 1e-3
    return s.project(st, lin, ang)


# ---------------------------------------------------------------------------------------------------------------
# the start's numbers (read from the placement's own inputs and its placed state) and their rates
def start_numbers(dt=B.VEL_DT):
    """({number: value}, {number: rate}, {side: the forefoot's ground point}, the start's state) of the cleared start:
    the numbers body.pose builds [4]'s launch posture from, moved by START_CLEAR, read back from the placed states at
    the start and DT either side of it (so each rate is the slope the starting velocities are made of)."""
    m = B.Model(H, M, ranges=(), grip="hands", feet="cleats", wrist="oval", shift=B.START_CLEAR)
    st = B.place(m, T0)
    arms = m.arm_choices
    out = []
    for t in (T0 - dt, T0, T0 + dt):
        rep = {}
        with B._shifted(B.START_CLEAR, H):
            B._SHIFT_ON[0] = True
            s_, _ = B.pose(m, t, report=rep, arms=arms)
            q = {"pelvis_yaw": B.curve("pelvis_rot", t), "pelvis_obl": B.curve_obliquity(t), "trunk_yaw": B.curve("trunk_rot", t),
                 "trunk_flex": B.TRUNK_FLEX, "trunk_side": B.TRUNK_SIDE, "bat_az": B.curve("bat_az", t), "bat_elev": B.curve("bat_elev", t),
                 "foot_yaw_R": B.REAR_FOOT_YAW, "foot_yaw_L": 90.0 - B.curve_lead_foot(t) - B.LEAD_FOOT_SHIFT[0]}
        xp = m.sys.pose(s_, m.b["pelvis"])[0]
        q.update(pelvis_x=xp[0], pelvis_y=xp[1], pelvis_z=xp[2], foot_pitch_L=rep["lead_heel_deg"], foot_pitch_R=rep["rear_heel_deg"],
                 bat_roll=arms[0], swivel_L=arms[1], swivel_R=arms[2])
        h = rep["hands_from_rear_shoulder"]
        q.update(hand_x=h[0], hand_y=h[1], hand_z=h[2])
        ball = {}
        for sd in "LR":
            xf, qf, _, _ = m.sys.pose(s_, m.b["foot_" + sd])
            ball[sd] = E.add(xf, E.mv(E.qR(qf), ball_mid(m)))
        out.append((q, ball))
    q0 = out[1][0]
    rate = {k: ((out[2][0][k] - out[0][0][k] + 180.0) % 360.0 - 180.0 if k in ("bat_az",) else out[2][0][k] - out[0][0][k]) / (2 * dt) for k in q0}
    return q0, rate, out[1][1], st


# ---------------------------------------------------------------------------------------------------------------
# the contact posture
def least_strain(m, sho, Rt, wr_of, base=None, grid=5):
    """(strain, roll, swivel L, swivel R): the bat's roll and the two elbows' swivels that leave the arm joint nearest an
    end furthest from it (body.strain with the wrists' ellipses), the elbows ARM_CLEAR from the other arm: part 1's rule
    for the start's three free choices, here for a posture whose hands are given. wr_of(roll) -> (Rb, {side: wrist})."""
    best = None
    lu, lf = m.len["upperarm"], m.len["forearm"]
    for roll in range(-180, 180, grid):
        Rb, wr = wr_of(float(roll))
        if any(E.norm(E.sub(wr[sd], sho[sd])) > 0.999 * (lu + lf) for sd in "LR"):
            continue
        cand = {}
        for sd in "LR":
            cand[sd] = []
            for sw in range(-180, 180, grid):
                el = elbow_of(m, sd, sho[sd], wr[sd], Rt, float(sw))
                c = B.strain(B.arm_angles(sd, E.unit(E.sub(sho[sd], el)), E.unit(E.sub(el, wr[sd])), Rb, Rt), oval=True)
                cand[sd].append((c, sw, el))
        for cl, sl, el_l in cand["L"]:
            for cr, sr, el_r in cand["R"]:
                k_ = max(cl, cr)
                if best is not None and k_ >= best[0]:
                    continue
                if min(B._seg_dist(el_l, sho["R"], el_r), B._seg_dist(el_l, el_r, wr["R"]),
                       B._seg_dist(el_r, sho["L"], el_l), B._seg_dist(el_r, el_l, wr["L"])) < B.ARM_CLEAR:
                    continue
                best = (k_, float(roll), float(sl), float(sr))
    return best


def contact_numbers(m, q0, ball, Pc, opt):
    """the contact posture's 20 numbers (4.12): a hitter's contact posture for the aim's ball, from joint ranges and the
    stated [E]s (CONTACT, changed by opt). Returns (numbers, notes)."""
    o = dict(CONTACT, **{k: v for k, v in opt.items() if k in CONTACT})
    q = dict(q0)
    notes = {}
    q["pelvis_yaw"] = o["pelvis_yaw"]
    q["pelvis_y"] = q0["pelvis_y"] + o["shift"]
    q["trunk_yaw"] = o["pelvis_yaw"] + o["trunk_ahead"]
    q["trunk_flex"], q["trunk_side"] = o["trunk_flex"], o["trunk_side"]
    q["foot_pitch_L"] = 0.0                                       # the lead foot flat
    q["foot_yaw_R"] = q0["foot_yaw_R"] + o["rear_foot_yaw"]
    reach = lambda deg: math.sqrt(m.len["thigh"] ** 2 + m.len["shank"] ** 2 + 2 * m.len["thigh"] * m.len["shank"] * math.cos(deg * D2R))

    def leg(sd, qq):
        fr, sho, hip = frames(m, qq, ball)
        ank = E.add(fr["foot_" + sd][0], E.mv(E.qR(fr["foot_" + sd][1]), m.ankle_f))
        return E.norm(E.sub(hip[sd], ank))
    lo, hi = 0.5, 1.3                                             # the pelvis's height that puts the lead knee at its bend, the foot flat
    for _ in range(60):
        mid = 0.5 * (lo + hi)
        lo, hi = (mid, hi) if leg("L", dict(q, pelvis_z=mid)) < reach(o["lead_knee"]) else (lo, mid)
    q["pelvis_z"] = 0.5 * (lo + hi)
    gap = lambda pitch: leg("R", dict(q, foot_pitch_R=pitch)) - reach(o["rear_knee"])      # > 0: the leg too long for its knee's bend
    if gap(0.0) <= 0.0:
        q["foot_pitch_R"] = 0.0
    elif gap(B.HEEL_MAX) > 0.0:
        q["foot_pitch_R"] = B.HEEL_MAX
    else:
        lo, hi = 0.0, B.HEEL_MAX
        for _ in range(60):
            mid = 0.5 * (lo + hi)
            lo, hi = (mid, hi) if gap(mid) > 0.0 else (lo, mid)
        q["foot_pitch_R"] = 0.5 * (lo + hi)
    # the bat: square to the ball's line seen from above, the barrel toward the plate and below the hands; the sweet spot at the ball
    q["bat_az"], q["bat_elev"] = BALL_ACROSS, o["bat_elev"]
    az, el = q["bat_az"] * D2R, q["bat_elev"] * D2R
    bz = (math.cos(el) * math.cos(az), math.cos(el) * math.sin(az), math.sin(el))
    p = E.add(E.sub(Pc, E.scale(bz, C.BAT_SS)), E.scale(bz, B.GRIP_L))
    fr, sho, hip = frames(m, q, ball)
    Rt = E.qR(fr["trunk"][1])
    h = E.mtv(Rt, E.sub(p, sho["R"]))
    q["hand_x"], q["hand_y"], q["hand_z"] = h

    def wr_of(roll):
        xb, qb, wr, _ = bat_frame(m, dict(q, bat_roll=roll), sho, Rt)
        return E.qR(qb), wr
    best = least_strain(m, sho, Rt, wr_of)
    if best is None:
        return None, {"why": "no roll of the bat and swivels of the elbows reach the bat from these shoulders inside the arms' ranges"}
    near = lambda a, ref: ref + ((a - ref + 180.0) % 360.0 - 180.0)
    q["bat_roll"], q["swivel_L"], q["swivel_R"] = near(best[1], q0["bat_roll"]), near(best[2], q0["swivel_L"]), near(best[3], q0["swivel_R"])
    q["bat_az"] = q0["bat_az"] + (q["bat_az"] - q0["bat_az"]) % 360.0          # the bat comes round counter-clockwise seen from above
    notes["arm_strain_at_contact"] = best[0]
    return q, notes


# ---------------------------------------------------------------------------------------------------------------
# the motion: each number on a cubic in time
class Motion:
    """20 numbers as functions of time (s from the start). spec: T (start to contact), onset {group: s}, speed (m/s),
    and CONTACT's [E]s. Built in __init__; ok is False (with why) where the contact posture cannot be taken."""

    def __init__(self, base, **spec):
        self.spec = spec
        self.m, self.q0, self.v0, self.ball, self.Pc = base["m"], base["q0"], base["v0"], base["ball"], base["Pc"]
        self.T = spec.get("T", 0.200)
        on = spec.get("onset", (0.0, 0.030, 0.060))
        self.onset = {}
        for g, t_on in zip(("legs", "trunk", "arms"), on):
            for k in GROUP[g]:
                self.onset[k] = t_on
        self.q1, self.notes = contact_numbers(self.m, self.q0, self.ball, self.Pc, spec)
        self.ok = self.q1 is not None
        if not self.ok:
            return
        T = self.T
        self.v1 = {}
        for k in NUMBERS:
            avg = (self.q1[k] - self.q0[k]) / T
            self.v1[k] = END_RATE.get(k, END_RATE_REST) * avg
        self._bat_rates(spec.get("speed", 29.0), spec.get("hands_share", CONTACT["hands_share"]))

    def _bat_rates(self, V, share, eps=1e-4):
        """the bat's and the hands' rates at contact: the sweet spot's velocity V along the ball's line (coming up through
        it), the hands' travel carrying `share` of it and the bat's turning about them the rest, about the axis square to
        the bat and to that line; no spin about the bat's own length. Found by carrying the bat a moment on and reading
        its numbers back from where the trunk then is."""
        m, q1 = self.m, self.q1
        d, a = BALL_DESCENT * D2R, BALL_ACROSS * D2R
        nb = (math.cos(d) * math.sin(a), -math.cos(d) * math.cos(a), -math.sin(d))
        dh = E.scale(nb, -1.0)                                     # the way the sweet spot travels
        fr, sho, hip = frames(m, q1, self.ball)
        Rt = E.qR(fr["trunk"][1])
        xb, qb, wr, p = bat_frame(m, q1, sho, Rt)
        Rb = E.qR(qb)
        bz = E.mv(Rb, (0, 0, 1))
        r = C.BAT_SS - B.GRIP_L
        perp = E.unit(E.sub(dh, E.scale(bz, E.dot(dh, bz))))
        w = E.scale(E.unit(E.cross(bz, dh)), (1.0 - share) * V / r)
        vh = E.sub(E.scale(dh, V), E.scale(perp, (1.0 - share) * V))
        self.contact_velocity = E.add(vh, E.cross(w, E.scale(bz, r)))
        # a moment on: the trunk by its own law's rates, the bat by (vh, w)
        q2 = {k: q1[k] + eps * self.v1[k] for k in NUMBERS}
        fr2, sho2, _ = frames(m, q2, self.ball)
        Rt2 = E.qR(fr2["trunk"][1])
        Rb2 = E.qR(E.qmul(E.qaxis(w, E.norm(w) * eps), qb))
        bz2, bx2 = E.mv(Rb2, (0, 0, 1)), E.mv(Rb2, (1, 0, 0))
        p2 = E.add(p, E.scale(vh, eps))
        h2 = E.mtv(Rt2, E.sub(p2, sho2["R"]))
        az2 = math.atan2(bz2[1], bz2[0]) / D2R
        el2 = math.asin(max(-1.0, min(1.0, bz2[2]))) / D2R
        mid = E.scale(E.add(sho2["L"], sho2["R"]), 0.5)
        toward = E.sub(mid, p2)
        ref = E.unit(E.sub(E.scale(bz2, E.dot(toward, bz2)), toward))
        roll2 = math.atan2(E.dot(bz2, E.cross(ref, bx2)), E.dot(ref, bx2)) / D2R
        wrap = lambda x: (x + 180.0) % 360.0 - 180.0
        self.v1["bat_az"] = wrap(az2 - q1["bat_az"]) / eps
        self.v1["bat_elev"] = (el2 - q1["bat_elev"]) / eps
        self.v1["bat_roll"] = wrap(roll2 - q1["bat_roll"]) / eps
        for k, c in zip(("hand_x", "hand_y", "hand_z"), range(3)):
            self.v1[k] = (h2[c] - q1[k]) / eps

    def numbers(self, t):
        out = {}
        for k in NUMBERS:
            a, va, b, vb, t_on, T = self.q0[k], self.v0.get(k, 0.0), self.q1[k], self.v1[k], self.onset[k], self.T
            if t <= t_on:
                out[k] = a + va * t
                continue
            a1 = a + va * t_on
            h = T - t_on
            u = (t - t_on) / h
            h00, h10, h01, h11 = 2 * u ** 3 - 3 * u ** 2 + 1, u ** 3 - 2 * u ** 2 + u, -2 * u ** 3 + 3 * u ** 2, u ** 3 - u ** 2
            out[k] = h00 * a1 + h10 * h * va + h01 * b + h11 * h * vb
        return out

    def state(self, t):
        return place(self.m, self.numbers(t), self.ball)


def base_setup():
    q0, v0, ball, st0 = start_numbers()
    m = new_model()
    xp = (q0["pelvis_x"], q0["pelvis_y"])
    Pc = (xp[0] + MEET_ACROSS, xp[1] + MEET_AHEAD, BALL_Z)
    Dr.add_damping(m, place(m, q0, ball))               # the joints' own damping [E] is part of the body
    m.sys.baum_a = m.sys.baum_b = 0.0
    return {"m": m, "q0": q0, "v0": v0, "ball": ball, "Pc": Pc, "st0": st0, "bd": Dr.bounds(H, M)}


# ---------------------------------------------------------------------------------------------------------------
# the 29 joint axes muscles act about, and which muscle group gives a torque each way
def generators(m, st):
    """[(label, joint, parent body, child body, world axis n, channel for a torque on the child about +n or None,
    channel for -n or None, the first law's speed group)] in state st. The axes are the engine's own drive axes
    (drive.CHANNELS): --check holds every one of the 41 channels to exactly one of these and its sign."""
    ax = lambda body, v: m.axis(st, body, v)
    out = []
    out += [("trunk side bend (toward the lead side +)", "lumbar", "pelvis", "trunk", ax("pelvis", (1, 0, 0)), None, None, "core"),
            ("trunk forward bend (+)", "lumbar", "pelvis", "trunk", ax("pelvis", (0, 1, 0)), None, None, "core"),
            ("trunk turn toward the pitcher (+)", "lumbar", "pelvis", "trunk", ax("pelvis", (0, 0, 1)), "core", "core_back", "core")]
    along = {sd: E.dot(m.axis(st, "farm_" + sd, (1, 0, 0)), m.axis(st, "bat", (0, 0, 1))) for sd in "LR"}
    for sd, sg in B.SIDES:
        L = sd == "L"
        th, sh, ft, ua, fa = "thigh_" + sd, "shank_" + sd, "foot_" + sd, "uarm_" + sd, "farm_" + sd
        out += [("hip %s extension (+) / flexion" % sd, "hip_" + sd, "pelvis", th, ax(th, (0, 1, 0)), "hip_ext_" + sd, "hip_flex_L" if L else None, "hip"),
                ("hip %s out to the side / in" % sd, "hip_" + sd, "pelvis", th, ax(th, (1, 0, 0)), "hip_abd_L" if L else "hip_add_R", "hip_add_L" if L else "hip_abd_R", "hip"),
                ("hip %s rotators (pelvis toward the pitcher: -)" % sd, "hip_" + sd, "pelvis", th, ax(th, (0, 0, 1)), None, "hip_rot_" + sd, "hip"),
                ("knee %s flexion (+) / extension" % sd, "knee_" + sd, th, sh, ax(th, (0, 1, 0)), "knee_flex_" + sd, "knee_ext_" + sd, "knee"),
                ("ankle %s push-off (+) / pull-up" % sd, "ankle_" + sd, sh, ft, ax(sh, (0, 1, 0)), "ankle_pf_" + sd, None, "ankle"),
                ("ankle %s roll" % sd, "ankle_" + sd, sh, ft, ax(ft, (1, 0, 0)), None, None, "ankle"),
                ("shoulder %s about the trunk's axis (+: swept toward the pitcher)" % sd, "shoulder_" + sd, "trunk", ua, ax("trunk", (0, 0, 1)), "shoulder_" + sd, None, "shoulder"),
                ("shoulder %s raise (+) / lower" % sd, "shoulder_" + sd, "trunk", ua, E.System.float_axis(ax("trunk", (0, 0, 1)), ax(ua, (0, 0, 1))), "sh_raise_" + sd, "sh_lower_" + sd, "shoulder"),
                ("upper arm %s turn" % sd, "shoulder_" + sd, "trunk", ua, ax(ua, (0, 0, 1)), "sh_erot_L" if L else "sh_irot_R", "sh_irot_L" if L else "sh_erot_R", "shoulder"),
                ("elbow %s flexion (+) / extension" % sd, "elbow_" + sd, ua, fa, ax(ua, (0, 1, 0)), "elbow_flex_" + sd, "elbow_ext_" + sd, "elbow"),
                ("forearm %s turn" % sd, "elbow_" + sd, ua, fa, ax(fa, (0, 0, 1)), "sup_L" if L else "pron_R", "pron_L" if L else "sup_R", "elbow")]
        s1 = (1.0 if along[sd] > 0 else -1.0) * (1.0 if sd == "R" else -1.0)       # drive.add_drives' sign of wrist flexion about the forearm's x
        out += [("wrist %s bend" % sd, "wrist_" + sd, fa, "bat", ax(fa, (1, 0, 0)), "wrist_flex_" + sd if s1 > 0 else "wrist_ext_" + sd, "wrist_ext_" + sd if s1 > 0 else "wrist_flex_" + sd, "wrist"),
                ("wrist %s tilt" % sd, "wrist_" + sd, fa, "bat", E.mv(E.qR(m.sys.pose(st, m.b["bat"])[1]), m.palm[sd]), None if L else "wrist_R", "wrist_L" if L else None, "wrist")]
    return out


def allowed(bd, ch, speed, grp):
    """what muscle group ch can give at shortening speed `speed` (rad/s; negative: being stretched): its bound times its
    measured strength-at-speed curve (4.5), or the first law where it has none (inverse3d.allowed_at)."""
    return I3.allowed_at(bd, ch, speed, grp)


def kinematics(mo, t, dt=DIFF):
    """(state with consistent velocities, consistent accelerations [per body 6]) of motion mo at time t, from postures
    dt apart (velocities over +-dt, accelerations over +-2 dt, as inverse3d.run)."""
    m = mo.m
    S = {k: mo.state(t + k * dt) for k in (-2, -1, 0, 1, 2)}
    vel = {k: I3.velocities(m, S[k - 1], S[k + 1], dt) for k in (-1, 0, 1)}
    st = list(S[0])
    vi, dv = I3.consistent(m, st, vel[0], 5)
    for b_ in range(m.sys.nb):
        st[13 * b_ + 7:13 * b_ + 13] = vi[b_]
    acc = [[(p - q) / (2 * dt) for p, q in zip(vel[1][b_], vel[-1][b_])] for b_ in range(m.sys.nb)]
    acc, da = I3.consistent(m, st, acc, 6)
    return st, acc


def loads_problem(m, st, acc):
    """the linear problem of one instant: (generators, sole points on the ground [(name, body, point)], a0, columns
    [78 accelerations per unit load], b) with accelerations = a0 + sum(load x column) to equal b."""
    s = m.sys
    nb = s.nb
    aux = []

    def accel(tones, ext):
        s.tones, s.ext = tones, ext
        ds = s.deriv(0.0, st, aux)
        return [ds[13 * i + 7 + c] for i in range(nb) for c in range(6)]
    a0 = accel([], [])
    gens = generators(m, st)
    cols = []
    for label, j, pa, ch, n, cp, cm, grp in gens:
        a = m.b[pa]
        Ra = E.qR(s.pose(st, a)[1])
        cols.append(accel([(j, a, m.b[ch], E.mtv(Ra, n))], []))
    pts = []
    cf = C.DE_LEVA["foot"][2]
    for sd, sg in B.SIDES:
        fb = m.b["foot_" + sd]
        xf, qf, _, _ = s.pose(st, fb)
        Rf = E.qR(qf)
        for nm, px in (("heel", -cf * m.len["foot"]), ("ball", (B.BALL_FWD - cf) * m.len["foot"])):
            hw = B.BALL_HALF_W if nm == "ball" else B.SOLE_HALF_W
            for wn, py in (("in", -sg * hw), ("out", sg * hw)):
                p = (px, py, m.sole_z)
                if E.add(xf, E.mv(Rf, p))[2] < ON_GROUND:
                    pts.append(("%s_%s_%s" % (sd, nm, wn), fb, p))
    for name, fb, p in pts:
        for c in range(3):
            cols.append(accel([], [(fb, p, E.E3[c], E.ZERO)]))
    s.tones, s.ext = [], []
    cols = [[x - y for x, y in zip(col, a0)] for col in cols]
    b = [c for row in acc for c in row]
    return gens, pts, a0, cols, b


def solve_instant(m, st, acc, bd):
    """the loads at one instant with the hardest-worked muscle group's share of its strength at its speed made least.
    Returns a dict: t (that share), by (the muscle group that sets it), share {channel: share}, torque {generator
    label: N m}, undriven {label: N m asked of a direction with no muscle}, ground {point: force}, miss (accelerations
    not met, relative), feasible."""
    import numpy as np
    from scipy.optimize import linprog
    s = m.sys
    gens, pts, a0, cols, b = loads_problem(m, st, acc)
    ng, npt = len(gens), len(pts)
    Cm = np.array(cols).T                                   # 78 x (ng + 3 npt)
    rhs = np.array(b) - np.array(a0)
    U, sv, Vt = np.linalg.svd(Cm, full_matrices=False)
    r = int((sv > 1e-9 * sv[0]).sum())
    Aeq_s = U[:, :r].T @ Cm                                 # r equations in the signed loads
    beq = U[:, :r].T @ rhs
    miss = float(np.linalg.norm(rhs - U[:, :r] @ beq) / max(1e-12, np.linalg.norm(rhs)))
    # variables: x+ and x- for each generator (2 ng), the points' forces (3 npt), t
    nv = 2 * ng + 3 * npt + 1
    Aeq = np.zeros((r, nv))
    Aeq[:, 0:ng] = Aeq_s[:, :ng]
    Aeq[:, ng:2 * ng] = -Aeq_s[:, :ng]
    Aeq[:, 2 * ng:2 * ng + 3 * npt] = Aeq_s[:, ng:]
    A_ub, b_ub = [], []
    cap, undriven = [], []
    for g, (label, j, pa, ch, n, cp, cm, grp) in enumerate(gens):
        wrel = E.dot(E.sub(s.pose(st, m.b[ch])[3], s.pose(st, m.b[pa])[3]), n)
        for sign, chn, col in ((1.0, cp, g), (-1.0, cm, ng + g)):
            if chn is None:
                undriven.append(col)
                cap.append(None)
                continue
            A = allowed(bd, chn, sign * wrel, grp)
            cap.append((chn, A, sign * wrel))
            row = np.zeros(nv); row[col] = 1.0; row[-1] = -A        # x <= t A
            A_ub.append(row); b_ub.append(0.0)
    cap_of = {}
    k = 0
    for g in range(ng):
        cap_of[g], cap_of[ng + g] = cap[2 * g], cap[2 * g + 1]
    for i in range(npt):
        o = 2 * ng + 3 * i
        for kk in range(CONE_SIDES):                               # an inscribed pyramid of the friction cone
            th = 2 * math.pi * kk / CONE_SIDES
            row = np.zeros(nv)
            row[o], row[o + 1], row[o + 2] = math.cos(th), math.sin(th), -MU * math.cos(math.pi / CONE_SIDES)
            A_ub.append(row); b_ub.append(0.0)
    bounds = [(0, None)] * (2 * ng) + [(None, None), (None, None), (0, None)] * npt + [(0, None)]
    c1 = np.zeros(nv); c1[-1] = 1.0
    res = linprog(c1, A_ub=np.array(A_ub), b_ub=np.array(b_ub), A_eq=Aeq, b_eq=beq, bounds=bounds, method="highs")
    if res.status != 0:
        return {"feasible": False, "why": res.message, "miss": miss, "points": [p[0] for p in pts]}
    t = float(res.x[-1])
    # second stage: the worst share held, what is asked of directions without a muscle made least, then the driven shares' sum
    c2 = np.zeros(nv)
    for col in undriven:
        c2[col] = 1.0
    for col, cp_ in cap_of.items():
        if cp_ is not None and cp_[1] > 1e-9:
            c2[col] = 1e-3 * 100.0 / cp_[1]
    bounds2 = list(bounds)
    bounds2[-1] = (0, t * (1.0 + 1e-7) + 1e-9)
    res2 = linprog(c2, A_ub=np.array(A_ub), b_ub=np.array(b_ub), A_eq=Aeq, b_eq=beq, bounds=bounds2, method="highs")
    x = res2.x if res2.status == 0 else res.x
    out = {"feasible": True, "t": t, "miss": miss, "share": {}, "torque": {}, "undriven": {}, "ground": {}, "speed": {}}
    worst = (0.0, None)
    for g, (label, j, pa, ch, n, cp, cm, grp) in enumerate(gens):
        out["torque"][label] = float(x[g] - x[ng + g])
        for col in (g, ng + g):
            if x[col] > 1e-7:
                if cap_of[col] is None:
                    out["undriven"][label + (" +" if col < ng else " -")] = float(x[col])
                else:
                    chn, A, sp = cap_of[col]
                    sh = float(x[col] / A) if A > 1e-9 else float("inf")
                    out["share"][chn] = max(out["share"].get(chn, 0.0), sh)
                    out["speed"][chn] = (float(x[col]), A, sp)
                    if sh > worst[0]:
                        worst = (sh, chn)
    out["by"] = worst[1]
    for i, (name, fb, p) in enumerate(pts):
        o = 2 * ng + 3 * i
        out["ground"][name] = (float(x[o]), float(x[o + 1]), float(x[o + 2]))
    # the solved loads put back into the engine's equations
    z = np.concatenate([x[:ng] - x[ng:2 * ng], x[2 * ng:2 * ng + 3 * npt]])
    out["back"] = float(np.linalg.norm(Cm @ z - rhs) / max(1e-12, np.linalg.norm(rhs)))
    return out


def geometry(mo, step=SAMPLE):
    """the motion as geometry: at every instant each range's margin (deg inside its end), the elbows' clearance, the
    elbows' bends; the least of each and when."""
    m = mo.m
    least, when, clear = {}, {}, 1e9
    n = int(round((mo.T) / step))
    rows = []
    for i in range(n + 1):
        t = i * step
        st = mo.state(t)
        mg = B.start_margins(m, st)
        for k, v in mg.items():
            if v < least.get(k, 1e9):
                least[k], when[k] = v, t
        el = {sd: m.joint_pos(st, "elbow_" + sd) for sd in "LR"}
        sh = {sd: m.joint_pos(st, "shoulder_" + sd) for sd in "LR"}
        wr = {sd: m.joint_pos(st, "wrist_" + sd) for sd in "LR"}
        clear = min(clear, B._seg_dist(el["L"], sh["R"], el["R"]), B._seg_dist(el["L"], el["R"], wr["R"]),
                    B._seg_dist(el["R"], sh["L"], el["L"]), B._seg_dist(el["R"], el["L"], wr["L"]))
        rows.append((t, m.observe(st)))
    return {"least": least, "when": when, "elbow_clear": clear, "obs_contact": rows[-1][1], "obs_start": rows[0][1]}


def solve_motion(mo, bd, step=SAMPLE, dt=DIFF, log=None):
    """every instant of the motion solved. Returns the list of solve_instant()'s results with their times and what the
    body is doing there."""
    out = []
    n = int(round(mo.T / step))
    BW = M * E.G
    for i in range(n + 1):
        t = i * step
        st, acc = kinematics(mo, t, dt)
        r = solve_instant(mo.m, st, acc, bd)
        o = mo.m.observe(st)
        r["share_worst"] = r.get("t", float("nan"))
        r.update(t=t, sweet=o["sweet_speed"], hands=E.norm(mo.m.bat_point(st, 0.15)[1]), pelvis_rate=o["pelvis_rot_vel"], trunk_rate=o["trunk_rot_vel"])
        if r["feasible"]:
            r["front"] = sum(f[2] for k, f in r["ground"].items() if k[0] == "L") / BW
            r["rear"] = sum(f[2] for k, f in r["ground"].items() if k[0] == "R") / BW
        out.append(r)
        if log:
            if r["feasible"]:
                und = sorted(r["undriven"].items(), key=lambda kv: -kv[1])[:3]
                log("   %4.0f ms: sweet spot %5.1f m/s, hands %4.1f; hardest-worked %-12s at %.2f of its strength at speed (%.0f of %.0f N m at %+.1f rad/s); feet %.2f + %.2f x body weight; no muscle for: %s"
                    % ((t - mo.T) * 1e3, r["sweet"], r["hands"], r["by"], r["share_worst"], r["speed"][r["by"]][0], r["speed"][r["by"]][1], r["speed"][r["by"]][2], r["front"], r["rear"],
                       ", ".join("%s %.0f" % kv for kv in und) or "nothing"))
            else:
                log("   %4.0f ms: NO LOADS GIVE THIS MOTION (%s); sole points on the ground: %s" % ((t - mo.T) * 1e3, r["why"][:60], r["points"]))
    return out


def summary(mo, sol):
    """a motion's result in a few numbers: the worst share over the motion (and by whom, when), over its last 100 ms,
    each muscle group's own worst share, the most asked of each direction without a muscle, the feet."""
    ok = [r for r in sol if r["feasible"]]
    bad = [r for r in sol if not r["feasible"]]
    out = {"spec": mo.spec, "instants": len(sol), "no_loads_at_ms": [round((r["t"] - mo.T) * 1e3) for r in bad]}
    if not ok:
        return out
    wr = max(ok, key=lambda r: r["share_worst"])
    out["worst"] = {"share": wr["share_worst"], "by": wr["by"], "ms": round((wr["t"] - mo.T) * 1e3)}
    by = {}
    for r in ok:
        for ch, sh in r["share"].items():
            if sh > by.get(ch, (0.0,))[0]:
                by[ch] = (sh, round((r["t"] - mo.T) * 1e3))
    out["by_muscle"] = dict(sorted(by.items(), key=lambda kv: -kv[1][0]))
    und = {}
    for r in ok:
        for k, v in r["undriven"].items():
            if v > und.get(k, (0.0,))[0]:
                und[k] = (v, round((r["t"] - mo.T) * 1e3))
    out["undriven"] = dict(sorted(und.items(), key=lambda kv: -kv[1][0]))
    out["front_peak"] = max(r["front"] for r in ok)
    out["feet_least"] = min(r["front"] + r["rear"] for r in ok)
    out["over_1_ms"] = sum(1 for r in ok if r["share_worst"] > 1.0) * SAMPLE * 1e3
    out["sweet_at_contact"] = sol[-1]["sweet"]
    out["hands_fastest"] = max(r["hands"] for r in sol)
    out["pelvis_fastest"] = max(r["pelvis_rate"] for r in sol)
    out["trunk_fastest"] = max(r["trunk_rate"] for r in sol)
    out["miss"] = max(r["miss"] for r in ok)
    out["back"] = max(r["back"] for r in ok)
    return out


_BASE = None


def _one(spec):
    """one motion of the table, in a worker: its geometry, then its solve if it stays inside every range."""
    global _BASE
    if _BASE is None:
        _BASE = base_setup()
    mo = Motion(_BASE, **spec)
    if not mo.ok:
        return {"spec": spec, "built": False, "why": mo.notes.get("why")}
    geo = geometry(mo, 0.010)
    out = {"spec": spec, "built": True, "least_margin": min(geo["least"].values()), "least_at": min(geo["least"], key=geo["least"].get),
           "elbow_clear": geo["elbow_clear"], "arm_strain_at_contact": mo.notes["arm_strain_at_contact"],
           "elbows_at_contact": (geo["obs_contact"]["lead_elbow_flex"], geo["obs_contact"]["trail_elbow_flex"]),
           "knees_at_contact": (geo["obs_contact"]["lead_knee_flex"], geo["obs_contact"]["trail_knee_flex"]),
           "margins_under_0": {k: (v, round((geo["when"][k] - mo.T) * 1e3)) for k, v in geo["least"].items() if v < 0.0}}
    if out["least_margin"] < 0.0 and not spec.get("solve_anyway"):
        return out
    out.update(summary(mo, solve_motion(mo, _BASE["bd"])))
    return out


def label(spec):
    return "T %3.0f ms, onsets %s ms, shift %2.0f cm, %2.0f m/s" % (spec.get("T", 0.2) * 1e3, "/".join("%.0f" % (x * 1e3) for x in spec.get("onset", (0, 0.03, 0.06))[1:]),
                                                                   spec.get("shift", CONTACT["shift"]) * 100, spec.get("speed", 29.0)) + "".join(
        ", %s %s" % (k, v) for k, v in spec.items() if k not in ("T", "onset", "shift", "speed"))


def check():
    """the solve's own checks: (1) each of the engine's 41 drive channels is exactly one of the 29 generators with the
    stated sign (the engine's own actuator, alone, at a posture mid-swing, against the generator's column); (2) the
    start's numbers put back through place() give the placed start; (3) a solved instant's loads reproduce its
    accelerations; (4) the linear programme on a problem with a known answer."""
    import numpy as np
    base = base_setup()
    m = base["m"]
    ok = True
    # (2) the start
    st = place(m, base["q0"], base["ball"])
    d = 0.0
    for i in range(m.sys.nb):
        d = max(d, max(abs(a - b) for a, b in zip(st[13 * i:13 * i + 7], base["st0"][13 * i:13 * i + 7])))
    print("(2) the start's 20 numbers placed again against the placed start: largest difference of a position or quaternion entry %.2e" % d)
    ok = ok and d < 1e-6
    # (1) channels against generators, at a posture mid-swing, at rest
    mo = Motion(base)
    stm = mo.state(0.150)
    mm = B.Model(H, M, ranges=(), grip="hands", feet="cleats", wrist="oval", contacts=False)
    mm.sys.baum_a = mm.sys.baum_b = 0.0
    stp = Dr.add_drives(mm, list(stm[:13 * mm.sys.nb]) + [0.0] * E.NW, {n: (1.0, -1.0, 0.0, 1e9, 1.0) for n in Dr.CHANNELS})
    s = mm.sys
    nb = s.nb
    acc_of = lambda ds: [ds[13 * i + 7 + c] for i in range(nb) for c in range(6)]
    oa = 13 * nb
    keepT = [a[4] for a in s.acts]
    base_state = list(stp)
    for j in range(len(s.acts)):
        base_state[oa + j] = 1.0
    def engine_col(j):
        for k, a in enumerate(s.acts):
            a[4] = 1.0 if k == j else 0.0
        aux = [0.0] * len(s.acts)
        if s.acts[j][11] > 0.0:
            aux[j] = 1.0 / s.acts[j][11]                  # a channel with a tendon spring: the spring stretched to carry 1 N m
        return acc_of(s.deriv(0.0, base_state, aux))
    for a in s.acts:
        a[4] = 0.0
    a0 = acc_of(s.deriv(0.0, base_state, [0.0] * len(s.acts)))
    gens, pts, ga0, gcols, _ = loads_problem(m, list(stm), [[0.0] * 6] * nb)
    by_ch = {}
    for g, (label_, j, pa, ch, n, cp, cm, grp) in enumerate(gens):
        if cp:
            by_ch[cp] = (g, 1.0)
        if cm:
            by_ch[cm] = (g, -1.0)
    worst = 0.0
    missing = [a[0] for a in s.acts if a[0] not in by_ch]
    for j, a in enumerate(s.acts):
        if a[0] in by_ch:
            g, sign = by_ch[a[0]]
            ec = [x - y for x, y in zip(engine_col(j), a0)]
            dd = max(abs(x - sign * y) for x, y in zip(ec, gcols[g]))
            worst = max(worst, dd / max(1e-12, max(abs(x) for x in ec)))
    print("(1) the engine's %d drive channels against the generators: %d matched, missing %s; the largest relative difference of an acceleration %.2e" % (len(s.acts), len(by_ch), missing, worst))
    ok = ok and not missing and worst < 1e-9 and len(by_ch) == len(Dr.CHANNELS)
    # (3) a solved instant
    stk, acc = kinematics(mo, 0.150)
    r = solve_instant(m, stk, acc, base["bd"])
    print("(3) an instant of the base motion (50 ms before contact): feasible %s, worst share %.3f by %s; accelerations outside what any loads can give %.1e of their size; the solved loads put back: %.1e"
          % (r["feasible"], r.get("t", float("nan")), r.get("by"), r["miss"], r.get("back", float("nan"))))
    ok = ok and r["feasible"] and r["miss"] < 1e-6 and r["back"] < 1e-6
    # (4) a known answer: the body hanging still in that posture off the ground cannot be held (no ground), on it it can;
    # and the worst share scales with the loads: the same posture at rest, gravity doubled, asks twice the share
    rest = list(stk)
    for i in range(nb):
        rest[13 * i + 7:13 * i + 13] = [0.0] * 6
    zero = [[0.0] * 6] * nb
    r1 = solve_instant(m, rest, zero, base["bd"])
    g_keep = m.sys.g
    m.sys.g = (0.0, 0.0, -2.0 * E.G)
    r2 = solve_instant(m, rest, zero, base["bd"])
    m.sys.g = g_keep
    tot1 = sum(f[2] for f in r1["ground"].values())
    print("(4) holding that posture still: worst share %.3f, the ground carries %.1f N of the weight's %.1f; under twice the gravity the worst share is %.3f (twice: %.3f)"
          % (r1["t"], tot1, M * E.G, r2["t"], 2 * r1["t"]))
    ok = ok and abs(tot1 - sum(m.sys.m) * E.G) < 1e-3 and abs(r2["t"] - 2 * r1["t"]) < 1e-6 * max(1.0, r1["t"])
    print("PASS" if ok else "FAIL")
    return ok


def table_specs():
    specs = []
    Ts = (0.160, 0.200, 0.250)
    ons = ((0.0, 0.0, 0.0), (0.0, 0.030, 0.060), (0.0, 0.050, 0.100))
    shifts = (0.0, 0.10, 0.17)
    for sp in (29.0, 21.0):
        for T in Ts:
            for on in ons:
                for sh in shifts:
                    specs.append({"T": T, "onset": on, "shift": sh, "speed": sp})
    for sp in (17.0, 25.0, 32.0):
        specs.append({"speed": sp})
    for k, vals in (("hands_share", (0.15, 0.30)), ("bat_elev", (-10.0, -30.0)), ("pelvis_yaw", (60.0, 90.0))):
        for v in vals:
            specs.append({k: v})
    return specs


def main():
    a = sys.argv[1:]
    arg = lambda k, d: a[a.index(k) + 1] if k in a else d
    if "--check" in a:
        sys.exit(0 if check() else 1)
    if "--base" in a:
        base = base_setup()
        spec = json.loads(arg("--spec", "{}"))
        mo = Motion(base, **spec)
        print("the motion: %s" % label(spec))
        if not mo.ok:
            print("   NOT BUILT: %s" % mo.notes); return
        print("   the start's numbers: " + ", ".join("%s %.3g" % (k, base["q0"][k]) for k in NUMBERS))
        print("   at contact:          " + ", ".join("%s %.3g" % (k, mo.q1[k]) for k in NUMBERS))
        print("   rates at contact:    " + ", ".join("%s %.3g" % (k, mo.v1[k]) for k in NUMBERS))
        geo = geometry(mo)
        print("   geometry: least margins (deg inside the end; ms before contact): " + ", ".join("%s %.1f (%.0f)" % (k, v, (geo["when"][k] - mo.T) * 1e3) for k, v in sorted(geo["least"].items(), key=lambda kv: kv[1])[:10]))
        o = geo["obs_contact"]
        print("   the elbows never nearer the other arm than %.1f cm; at contact: lead elbow %.0f deg, rear %.0f [53 +- 15, 79 +- 13]; lead knee %.0f, rear %.0f; the arms' strain %.2f"
              % (geo["elbow_clear"] * 100, o["lead_elbow_flex"], o["trail_elbow_flex"], o["lead_knee_flex"], o["trail_knee_flex"], mo.notes["arm_strain_at_contact"]))
        stc = mo.state(mo.T)
        pss, vss = mo.m.bat_point(stc, C.BAT_SS)
        print("   the sweet spot at contact: %.1f mm from the ball; asked velocity %.2f m/s" % (1e3 * E.norm(E.sub(pss, base["Pc"])), E.norm(mo.contact_velocity)))
        sol = solve_motion(mo, base["bd"], log=print)
        sm = summary(mo, sol)
        print("   SUMMARY: " + json.dumps({k: v for k, v in sm.items() if k not in ("by_muscle", "undriven")}))
        print("   each muscle group's worst share (when, ms before contact): " + ", ".join("%s %.2f (%d)" % (k, v[0], v[1]) for k, v in list(sm.get("by_muscle", {}).items())[:16]))
        print("   most asked of a direction with no muscle (N m; when): " + ", ".join("%s %.0f (%d)" % (k, v[0], v[1]) for k, v in sm.get("undriven", {}).items()))
        return
    if "--table" in a:
        specs = table_specs()
        if "--only" in a:
            specs = specs[:int(arg("--only", 4))]
        t = time.time()
        with multiprocessing.get_context("fork").Pool(4) as pool:
            res = pool.map(_one, specs, chunksize=1)
        json.dump(res, open(os.path.join(OUT, arg("--out", "built3d_table.json")), "w"), indent=1)
        print("%d motions, %.0f s" % (len(res), time.time() - t))
        for r in res:
            if not r["built"]:
                print("%-64s NOT BUILT: %s" % (label(r["spec"]), r["why"])); continue
            head = "%-64s least margin %+5.1f deg (%s)" % (label(r["spec"]), r["least_margin"], r["least_at"])
            if "worst" not in r:
                print(head + ("  LEAVES A RANGE: not solved" if r["least_margin"] < 0 else "  no instant could be solved")); continue
            top = list(r["by_muscle"].items())[:4]
            und = list(r["undriven"].items())[:3]
            print(head + "; worst share %.2f by %s at %d ms; over 1 for %.0f ms; then %s; no muscle for: %s; front foot's peak %.2f, feet's least %.2f x body weight; no loads at %s"
                  % (r["worst"]["share"], r["worst"]["by"], r["worst"]["ms"], r["over_1_ms"], ", ".join("%s %.2f" % (k, v[0]) for k, v in top[1:]),
                     ", ".join("%s %.0f" % (k, v[0]) for k, v in und), r["front_peak"], r["feet_least"], r["no_loads_at_ms"] or "none"))
        return
    print(__doc__)


if __name__ == "__main__":
    main()
