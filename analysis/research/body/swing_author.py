"""A big-league swing written by hand as a motion, on the force-driven body's own skeleton (PLAN D48; docs/body-motion.md 4.17).

    python3 analysis/research/body/swing_author.py --table                          what a baseball man would read, every 10 ms
    uv run --with matplotlib python3 analysis/research/body/swing_author.py --draw <dir>   line drawings, three sides, every 10 ms
    python3 analysis/research/body/swing_author.py --swing <file.swing>             the motion for the Unreal viewer (no forces)

THE MOTION HERE HAS NO FORCES IN IT. It is the picture the muscles are then asked to produce (swing_track.py). It is
written from how hitters move: the owner's walkthrough (docs/research/body/0-owner-walkthrough.md), the published
sources (docs/research/swing/swing-synthesis.md) and what a recorded swing and video show to the eye. NO COORDINATE,
ANGLE OR TIMING OF A RECORDED SWING IS READ BY THIS FILE (PLAN D48): every number below was typed by hand, looked at in
the drawings and changed until the drawings looked like a hitter.

How the motion is written. A pose is a few dozen numbers a coach could say aloud (CHANNELS): where the hips are and
how they face, how the spine leans, where each foot is and how it is turned, where the hands are from the chest, where
the barrel points, where each elbow points. KEYS gives each number at the instants where it matters; between them it
follows a curve that does not overshoot (a monotone cubic), and the whole is then smoothed a little so that nothing
starts or stops in an instant. The body's joints are then found from the numbers by geometry (the legs and arms are
two-bone chains; both hands stay on the handle; a planted foot stays where it is).

Frame: the sim's (AGENTS.md). +X toward first base (a right-handed batter faces it), +Y toward the pitcher, +Z up, the
origin at the back tip of home plate. Times are seconds from contact.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "..", "oracle"))
from body3d import engine as E, body as B   # noqa: E402

C = B.C
D2R = math.pi / 180.0
H, M = 1.84, 92.0
UP = (0.0, 0.0, 1.0)
DT = 0.001                      # s: the motion is sampled every millisecond
SMOOTH_S = 0.008                # s [E: 0.008-0.020]: the width (one sigma) of the smoothing of every number's curve
MID_HANDS_U = 0.5 * (B.GRIP_L + B.GRIP_R)     # m from the knob: the point between the two hands on the handle's axis
# The pitch the swing is drawn against: a fastball through the meeting point, the practice aim's own (practice3d.py:
# 37.98 m/s at the plate, coming down 5.27 deg and across 1.62 deg). A picture for the eye; the motion does not read it.
BALL_SPEED, BALL_DESCENT, BALL_ACROSS = 37.98, 5.27, 1.62

# ---- the numbers of a pose -----------------------------------------------------------------------------------------
# hip_x, hip_z  the point midway between the hip joints: toward the plate, and its height (m)
# com_y       the whole body's centre of mass toward the pitcher (m); the hips are put where that needs them. It is
#             the centre of mass that is written because that is what the ground can and cannot do: on one
#             foot a body can only fall away from that foot (com_y_curve)
# pel_yaw     which way the belt buckle faces, deg: 0 toward the plate, 90 toward the pitcher
# pel_lp/lq   the pelvis's own up-axis leaned toward the plate / toward the pitcher (deg)
# tr_yaw      which way the chest faces (deg, as the buckle)
# tr_lp/lq    the spine leaned toward the plate / toward the pitcher (deg): the lean stays in the world while the
#             chest turns under it, which is how a hitter's forward bend in the stance becomes side bend at contact
# fL_*, fR_*  each foot: the ball of the foot's place on the ground (x, y), its height off the ground (z), where the
#             toes point (yaw, deg from +X) and how far the heel is raised on the ball of the foot (pitch, deg)
# hand_f/l/u  the point between the hands, from the middle of the shoulders, in the chest's own directions: in front
#             of the chest, toward his left (lead) side, up along the spine (m)
# bat_az/el   where the barrel points: round from +X toward +Y (deg, not wrapped, so a swing runs on through 180) and
#             above level (deg)
# bat_roll    the bat turned about its own length from the rule (the wrists' side of the handle faces the chest), deg
# sw_L, sw_R  where each elbow points: 0 as far down as it can, 90 straight out to its own side (deg, elbow_at)
CHANNELS = ("hip_x", "com_y", "hip_z", "pel_yaw", "pel_lp", "pel_lq", "tr_yaw", "tr_lp", "tr_lq",
            "fL_x", "fL_y", "fL_z", "fL_yaw", "fL_pitch", "fR_x", "fR_y", "fR_z", "fR_yaw", "fR_pitch",
            "hand_f", "hand_l", "hand_u", "bat_az", "bat_el", "bat_roll", "sw_L", "sw_R")

# ---- the swing ------------------------------------------------------------------------------------------------------
# Each row: (time s, {number: value}). A number keeps its last value until its next row. Written position by position;
# the comments say what a hitter is doing there. docs/body-motion.md 4.17 has the account in full sentences.
T_START, T_END = -1.400, 0.800
T_LIFT, T_OFF, T_TOP, T_TOUCH, T_HEEL, T_CONTACT = -1.050, -0.700, -0.500, -0.215, -0.165, 0.0
REAR_Y, LEAD_Y0, LEAD_Y1 = -0.450, 0.190, 0.500       # the balls of the feet toward the pitcher: rear; lead in the stance; lead after the stride
KEYS = [
    # ---- the stance: feet a little wider than the shoulders, knees and hips bent, chest over the toes, hands by the
    # rear shoulder, the bat tipped back over it toward the pitcher's side of his head, a little more weight on the rear leg
    (T_START, dict(hip_x=-0.975, hip_z=0.880, pel_yaw=0.0, pel_lp=18.0, pel_lq=0.0,
                   tr_yaw=-4.0, tr_lp=31.0, tr_lq=0.0,
                   fR_x=-0.770, fR_y=REAR_Y, fR_z=0.0, fR_yaw=0.0, fR_pitch=0.0,
                   fL_x=-0.770, fL_y=LEAD_Y0, fL_z=0.0, fL_yaw=10.0, fL_pitch=0.0)),
    (T_LIFT, dict(hip_z=0.885, pel_yaw=0.0, tr_yaw=-4.0, tr_lq=0.0,
                  fL_pitch=0.0, fL_z=0.0, fL_y=LEAD_Y0, fL_x=-0.770, fL_yaw=10.0)),
    # ---- the load: the weight goes into the rear hip, the hips and shoulders turn back a little, the hands go back;
    # the lead heel comes up
    (-0.800, dict(fL_pitch=0.0, fL_z=0.0, pel_yaw=-4.0, tr_yaw=-9.0)),
    (T_OFF, dict(fL_pitch=20.0, fL_z=0.0, fL_y=LEAD_Y0, hip_z=0.875, pel_yaw=-6.0, tr_yaw=-13.0, tr_lq=-2.0)),
    # ---- the knee comes up and in; he stands on the rear leg and is already drifting toward the pitcher
    (T_TOP, dict(hip_z=0.865, hip_x=-0.970, pel_yaw=-9.0, pel_lq=-3.0, tr_yaw=-18.0, tr_lq=-3.0, tr_lp=31.0,
                 fL_z=0.120, fL_y=0.150, fL_x=-0.760, fL_pitch=24.0, fL_yaw=0.0)),
    # ---- the stride: a controlled fall toward the pitcher; the foot reaches out and stays closed; the hands stay back
    (-0.370, dict(fL_z=0.075, fL_y=0.320, fL_pitch=14.0, hip_z=0.850, pel_yaw=-11.0, tr_yaw=-21.0)),
    (-0.280, dict(fL_z=0.030, fL_y=0.440, fL_x=-0.740, fL_pitch=14.0, fL_yaw=14.0, hip_z=0.835)),
    # ---- toe touch: the front foot lands softly on its ball, closed; the hips are already starting to open; the
    # chest and the hands are still back (the stretch)
    (T_TOUCH, dict(fL_z=0.0, fL_y=LEAD_Y1, fL_x=-0.725, fL_pitch=12.0, fL_yaw=25.0, hip_z=0.825, hip_x=-0.960,
                   pel_yaw=-7.0, pel_lq=-4.0, pel_lp=17.0, tr_yaw=-24.0, tr_lq=-5.0, tr_lp=30.0,
                   fR_pitch=0.0, fR_yaw=0.0)),
    # ---- heel plant: the weight comes onto the front side; the hips fire
    (T_HEEL, dict(fL_pitch=0.0, fL_yaw=30.0, hip_z=0.820, pel_yaw=5.0, tr_yaw=-23.0)),
    # ---- hips lead; the rear elbow drops into the slot; the barrel tips back and flattens behind him
    (-0.120, dict(pel_yaw=22.0, tr_yaw=-14.0, hip_z=0.825, fR_pitch=5.0, fR_yaw=6.0, tr_lp=29.0, tr_lq=-7.0, hip_x=-0.945)),
    (-0.080, dict(pel_yaw=46.0, tr_yaw=14.0, hip_z=0.830, fR_pitch=22.0, fR_yaw=30.0, pel_lp=14.0, hip_x=-0.925, tr_lq=-9.0, tr_lp=27.0)),
    (-0.045, dict(pel_yaw=68.0, tr_yaw=46.0, fR_pitch=36.0, fR_yaw=52.0)),
    (-0.020, dict(pel_yaw=78.0, tr_yaw=66.0)),
    # ---- contact: the front leg firm, the rear knee in and the heel up, the belt buckle at the pitcher, the spine
    # tilted over the plate and a little back, both elbows still bent, the barrel below the hands and coming up
    (T_CONTACT, dict(pel_yaw=83.0, tr_yaw=79.0, hip_x=-0.900, hip_z=0.822, pel_lp=10.0, pel_lq=-4.0,
                     tr_lp=25.0, tr_lq=-12.0, fR_pitch=48.0, fR_yaw=68.0, fL_yaw=38.0)),
    # ---- extension: the arms straighten out through the ball toward the pitcher
    (0.022, dict(tr_yaw=90.0, pel_yaw=86.0)),
    (0.050, dict(tr_yaw=101.0, pel_yaw=89.0)),
    (0.090, dict(tr_yaw=112.0, pel_yaw=92.0,
                 fR_pitch=56.0, fR_yaw=80.0, tr_lq=-12.0, tr_lp=21.0, hip_z=0.828)),
    # ---- the wrap: the hands go up and round over the lead shoulder, the bat behind his head
    (0.150, dict(tr_yaw=124.0, pel_yaw=95.0, fL_yaw=46.0)),
    (0.240, dict(tr_yaw=133.0, pel_yaw=98.0,
                 tr_lp=14.0, tr_lq=-8.0, hip_z=0.840, fL_yaw=54.0, pel_lp=7.0, pel_lq=-3.0, hip_x=-0.900)),
    # ---- the finish: tall on a firm front leg, the rear foot on its toe, the chest past the pitcher, both hands high
    (0.380, dict(tr_yaw=138.0, pel_yaw=100.0,
                 tr_lp=9.0, tr_lq=-5.0, hip_z=0.845, fR_pitch=60.0, fR_yaw=85.0, fL_yaw=58.0, pel_lp=6.0, pel_lq=-2.0)),
    (0.560, dict(tr_yaw=139.0, pel_yaw=100.0, hip_z=0.845)),
]
# The hands and the bat. Written by hand first (where the hands are from the chest, where the barrel points), then each
# row moved as little as it took to bring every joint of both arms inside its range, with the bat's roll and the elbows
# where the arms are least strained (a search on this body's own joint ranges, sim/target/codex-probes/swing-look/
# fit_arms.py as it stood on 2026-11-27; body.strain). The wrists' end toward the barrel is taken 19 deg further out than
# a closed hand's (body-motion 4.17: the handle shifts in the fingers at full extension).
KEYS += [
    # stance, load, stride: the hands by the rear shoulder, the bat up and tipped back over it
    (T_START, dict(hand_f=0.250, hand_l=-0.180, hand_u=0.000, bat_az=-168, bat_el=55, bat_roll=40, sw_L=-10, sw_R=-20)),
    (T_LIFT, dict(hand_f=0.250, hand_l=-0.180, hand_u=0.000, bat_az=-168, bat_el=55, bat_roll=40, sw_L=-10, sw_R=-20)),
    (T_TOP, dict(hand_f=0.220, hand_l=-0.240, hand_u=0.000, bat_az=-185, bat_el=55, bat_roll=0, sw_L=-20, sw_R=-20)),
    (T_TOUCH, dict(hand_f=0.210, hand_l=-0.270, hand_u=0.000, bat_az=-195, bat_el=55, bat_roll=0, sw_L=-20, sw_R=-20)),
    (T_HEEL, dict(hand_f=0.203, hand_l=-0.270, hand_u=-0.010, bat_az=-195, bat_el=52, bat_roll=10, sw_L=-20, sw_R=-20)),
    # the hands start: the rear elbow comes down into the slot, the barrel tips back and flattens behind him
    (-0.120, dict(hand_f=0.220, hand_l=-0.255, hand_u=-0.015, bat_az=-178, bat_el=48, bat_roll=10, sw_L=-10, sw_R=-40)),
    (-0.100, dict(hand_f=0.223, hand_l=-0.268, hand_u=-0.048, bat_az=-158, bat_el=36, bat_roll=20, sw_L=-10, sw_R=-50)),
    (-0.080, dict(hand_f=0.242, hand_l=-0.280, hand_u=-0.105, bat_az=-134, bat_el=18, bat_roll=20, sw_L=0, sw_R=-70)),
    (-0.060, dict(hand_f=0.280, hand_l=-0.242, hand_u=-0.198, bat_az=-114, bat_el=0, bat_roll=20, sw_L=20, sw_R=-90)),
    (-0.045, dict(hand_f=0.307, hand_l=-0.212, hand_u=-0.260, bat_az=-97, bat_el=-12, bat_roll=25, sw_L=40, sw_R=-100)),
    (-0.030, dict(hand_f=0.320, hand_l=-0.167, hand_u=-0.310, bat_az=-73, bat_el=-20, bat_roll=25, sw_L=50, sw_R=-100)),
    (-0.015, dict(hand_f=0.330, hand_l=-0.130, hand_u=-0.340, bat_az=-39, bat_el=-24, bat_roll=10, sw_L=50, sw_R=-80)),
    # contact: both elbows bent, the barrel below the hands
    (T_CONTACT, dict(hand_f=0.335, hand_l=-0.100, hand_u=-0.360, bat_az=1, bat_el=-24, bat_roll=10, sw_L=40, sw_R=-70)),
    # extension, the wrap, the finish. FROM HERE THE ROWS ARE AS WRITTEN: the search finds no way through the top hand's
    # roll-over that keeps welded hands inside a wrist's range, so after contact the wrists' and forearms' ranges are not
    # held (body-motion 4.17, the list of what was loosened)
    (0.015, dict(hand_f=0.380, hand_l=-0.070, hand_u=-0.340, bat_az=40, bat_el=-19)),
    (0.030, dict(hand_f=0.420, hand_l=-0.030, hand_u=-0.320, bat_az=75, bat_el=-9)),
    (0.050, dict(hand_f=0.440, hand_l=0.015, hand_u=-0.270, bat_az=112, bat_el=6, bat_roll=0, sw_L=0, sw_R=-30)),
    (0.070, dict(hand_f=0.450, hand_l=0.055, hand_u=-0.200, bat_az=140, bat_el=20)),
    (0.090, dict(hand_f=0.450, hand_l=0.095, hand_u=-0.125, bat_az=163, bat_el=32, bat_roll=10, sw_L=20, sw_R=-20)),
    (0.120, dict(hand_f=0.440, hand_l=0.150, hand_u=-0.040, bat_az=190, bat_el=42)),
    (0.150, dict(hand_f=0.400, hand_l=0.175, hand_u=0.035, bat_az=212, bat_el=45, bat_roll=0, sw_L=20, sw_R=10)),
    (0.200, dict(hand_f=0.330, hand_l=0.190, hand_u=0.100, bat_az=240, bat_el=36)),
    (0.250, dict(hand_f=0.285, hand_l=0.190, hand_u=0.140, bat_az=265, bat_el=20, bat_roll=0, sw_L=0, sw_R=30)),
    (0.300, dict(hand_f=0.255, hand_l=0.185, hand_u=0.170, bat_az=285, bat_el=5)),
    (0.380, dict(hand_f=0.225, hand_l=0.180, hand_u=0.190, bat_az=303, bat_el=-8)),
    (0.560, dict(hand_f=0.200, hand_l=0.165, hand_u=0.200, bat_az=314, bat_el=-16, bat_roll=0, sw_L=10, sw_R=30)),
]
WRIST_ULN_LOOSE = 55.0         # deg: the wrists' end toward the barrel (body.ROM's 36 for a hand in the open, x 0.90 closed)

# ---- the centre of mass toward the pitcher ----------------------------------------------------------------------------
# A body standing on one foot cannot be pushed along the ground at will: its centre of mass can only fall away from
# where that foot pushes (an upside-down pendulum: its acceleration is g / height x how far it stands ahead of the push).
# So the stride is written as that fall, and not as a curve drawn by hand: at the instant the lead foot leaves the
# ground his centre is COM_AHEAD in front of where the rear foot pushes, still drifting back at COM_V_OFF, and from
# there it falls toward the pitcher until the front foot is down. Before that (both feet down) and after it (the front
# leg stopping him) the curve is a smooth join.
COM_STANCE = -0.175            # m: in the stance, 57 % of his weight on the rear foot [4]
COM_PUSH = -0.420              # m: where the rear foot's push acts while he stands on it alone (its inside half)
COM_AHEAD = 0.130              # m [E: 0.10-0.15]
COM_V_OFF = -0.065             # m/s [E]
COM_HEIGHT = 0.93              # m: the centre of mass's height in the stride (read off the motion)
COM_CONTACT = (-0.030, 0.25)   # m, m/s: his centre at contact and its speed there (the front leg has nearly stopped him)
COM_FINISH = (0.005, 0.250)    # m, s: where it comes to rest, and when


def _quintic(t, t0, t1, y0, v0, a0, y1, v1, a1):
    """the fifth-order curve from (y0, v0, a0) at t0 to (y1, v1, a1) at t1."""
    T = t1 - t0
    s = (t - t0) / T
    c0, c1, c2 = y0, v0 * T, 0.5 * a0 * T * T
    d0 = y1 - c0 - c1 - c2
    d1 = v1 * T - c1 - 2 * c2
    d2 = a1 * T * T - 2 * c2
    c3 = 10 * d0 - 4 * d1 + 0.5 * d2
    c4 = -15 * d0 + 7 * d1 - d2
    c5 = 6 * d0 - 3 * d1 + 0.5 * d2
    return c0 + s * (c1 + s * (c2 + s * (c3 + s * (c4 + s * c5))))


def com_y_curve(t):
    """his centre of mass toward the pitcher at time t (m)."""
    w = math.sqrt(E.G / COM_HEIGHT)
    A, Bc = COM_AHEAD, COM_V_OFF / w
    fall = lambda tt: (COM_PUSH + A * math.cosh(w * tt) + Bc * math.sinh(w * tt), w * (A * math.sinh(w * tt) + Bc * math.cosh(w * tt)),
                       w * w * (A * math.cosh(w * tt) + Bc * math.sinh(w * tt)))
    y0, v0, a0 = fall(0.0)
    y1, v1, a1 = fall(T_TOUCH - T_OFF)
    if t <= T_LIFT:
        return COM_STANCE
    if t <= T_OFF:
        return _quintic(t, T_LIFT, T_OFF, COM_STANCE, 0.0, 0.0, y0, v0, a0)
    if t <= T_TOUCH:
        return fall(t - T_OFF)[0]
    yc, vc = COM_CONTACT
    if t <= T_CONTACT:
        return _quintic(t, T_TOUCH, T_CONTACT, y1, v1, a1, yc, vc, -2.0)
    yf, tf = COM_FINISH
    if t <= tf:
        return _quintic(t, T_CONTACT, tf, yc, vc, -2.0, yf, 0.0, 0.0)
    return yf


EVENTS = (("start", T_START, "the stance"), ("lead_heel_up", T_LIFT, "the load begins"), ("lead_foot_off", T_OFF, "the lead foot leaves the ground"),
          ("leg_lift_top", T_TOP, "the top of the leg lift"), ("front_foot_down", T_TOUCH, "the front foot touches, on its ball"),
          ("foot_plant", T_HEEL, "the front heel is down"), ("contact", 0.0, "the sweet spot meets the ball"), ("end", T_END, "the finish held"))


# ---- curves through the rows -----------------------------------------------------------------------------------------
def _pchip(ts, vs):
    """the slopes of a monotone cubic through (ts, vs) (Fritsch & Carlson 1980): flat at a peak or a hollow, never
    beyond a row's value between two rows."""
    n = len(ts)
    if n == 1:
        return [0.0]
    h = [ts[i + 1] - ts[i] for i in range(n - 1)]
    d = [(vs[i + 1] - vs[i]) / h[i] for i in range(n - 1)]
    m = [0.0] * n
    m[0], m[-1] = 0.0, 0.0                       # at rest at both ends of what is written
    for i in range(1, n - 1):
        if d[i - 1] * d[i] > 0.0:
            w1, w2 = 2 * h[i] + h[i - 1], h[i] + 2 * h[i - 1]
            m[i] = (w1 + w2) / (w1 / d[i - 1] + w2 / d[i])
    return m


def _hermite(ts, vs, ms, t):
    if t <= ts[0]:
        return vs[0]
    if t >= ts[-1]:
        return vs[-1]
    lo, hi = 0, len(ts) - 1
    while hi - lo > 1:
        mid = (lo + hi) // 2
        if ts[mid] <= t:
            lo = mid
        else:
            hi = mid
    h = ts[hi] - ts[lo]
    s = (t - ts[lo]) / h
    return ((2 * s ** 3 - 3 * s ** 2 + 1) * vs[lo] + (s ** 3 - 2 * s ** 2 + s) * h * ms[lo]
            + (-2 * s ** 3 + 3 * s ** 2) * vs[hi] + (s ** 3 - s ** 2) * h * ms[hi])


def _gauss(v, sigma_n):
    """v smoothed by a bell of sigma_n samples, the ends held (the motion is at rest there)."""
    if sigma_n <= 0.0:
        return list(v)
    half = int(math.ceil(4 * sigma_n))
    ker = [math.exp(-0.5 * (k / sigma_n) ** 2) for k in range(-half, half + 1)]
    tot = sum(ker)
    n = len(v)
    out = []
    for i in range(n):
        acc = 0.0
        for k in range(-half, half + 1):
            j = min(n - 1, max(0, i + k))
            acc += ker[k + half] * v[j]
        out.append(acc / tot)
    return out


class Motion:
    """the written motion: every number of CHANNELS on a time grid of DT from t0 to t1."""

    def __init__(self, keys=None, t0=T_START, t1=T_END, dt=DT, smooth=SMOOTH_S):
        keys = KEYS if keys is None else keys
        self.t0, self.t1, self.dt = t0, t1, dt
        self.n = int(round((t1 - t0) / dt)) + 1
        self.ts = [t0 + i * dt for i in range(self.n)]
        self.ch = {}
        for c in CHANNELS:
            if c == "com_y":
                self.ch[c] = [com_y_curve(t) for t in self.ts]
                continue
            rows = [(t, d[c]) for t, d in keys if c in d]
            if not rows:
                raise ValueError("no row gives " + c)
            kt, kv = [r[0] for r in rows], [r[1] for r in rows]
            if any(b <= a for a, b in zip(kt, kt[1:])):
                raise ValueError("the rows of %s are not in order of time" % c)
            ms = _pchip(kt, kv)
            raw = [_hermite(kt, kv, ms, t) for t in self.ts]
            self.ch[c] = _gauss(raw, smooth / dt)

    def at(self, i):
        return {c: self.ch[c][i] for c in CHANNELS}

    def index(self, t):
        return min(self.n - 1, max(0, int(round((t - self.t0) / self.dt))))


# ---- a pose from its numbers -----------------------------------------------------------------------------------------
def new_model(contacts=True):
    """the force-driven batter's body (body-motion 4.11-4.13): each hand's own grip, cleats, the wrists' ellipses; no
    joint stop is added here (swing_track adds them for the forward run)."""
    return B.Model(H, M, ranges=(), grip="hands", feet="cleats", wrist="oval", contacts=contacts)


def _lean(lp, lq):
    """the turn that leans an upright axis lp deg toward +X (the plate) and lq deg toward +Y (the pitcher)."""
    return E.qmul(E.qaxis((1.0, 0.0, 0.0), -lq * D2R), E.qaxis((0.0, 1.0, 0.0), lp * D2R))


def ball_mid(m):
    cf = C.DE_LEVA["foot"][2]
    return ((B.BALL_FWD - cf) * m.len["foot"], 0.0, m.sole_z)


def bat_frame(m, p, hands, mid, bz):
    """the bat's place and turn: its axis along bz through the point between the hands; rolled about its own length so
    that the wrists' side of the handle (its -x) faces the middle of the shoulders, then by bat_roll more."""
    toward = E.sub(mid, hands)
    bx = E.unit(E.sub(E.scale(bz, E.dot(toward, bz)), toward))
    by = E.cross(bz, bx)
    r = p["bat_roll"] * D2R
    if r != 0.0:
        bx = E.add(E.scale(bx, math.cos(r)), E.scale(by, math.sin(r)))
        by = E.cross(bz, bx)
    qb = E.axes_to_q(bx, by, bz)
    Rb = E.qR(qb)
    return E.sub(hands, E.mv(Rb, (0.0, 0.0, MID_HANDS_U - m.bat_ucm))), qb, Rb


def elbow_at(sho, wrist, lu, lf, sg, swivel_deg):
    """the elbow of an arm from sho to wrist: pointing as far down as it can at swivel 0, and turned about the line from
    the shoulder to the wrist by `swivel_deg`, positive outward (the lead elbow toward his left, the rear toward his right,
    for an arm reaching forward)."""
    dh = E.unit(E.sub(wrist, sho))
    down = (0.0, 0.0, -1.0)
    perp = E.sub(down, E.scale(dh, E.dot(down, dh)))
    if E.norm(perp) < 1e-6:                                     # the arm straight up or down: any side will do
        perp = (1.0, 0.0, 0.0)
    toward = E.mv(E.qR(E.qaxis(dh, sg * swivel_deg * D2R)), E.unit(perp))
    return B._two_link(sho, wrist, lu, lf, toward)


def pose(m, p, report=None, close=True):
    """the body's state (at rest) for the numbers p. report (a dict): how far each leg and arm is from reaching (m; over
    0 the limb is too short for what was written and the drawing is wrong there)."""
    s, b = m.sys, m.b
    st = s.new_state()
    qp = E.qmul(_lean(p["pel_lp"], p["pel_lq"]), E.qaxis(UP, p["pel_yaw"] * D2R))
    Rp = E.qR(qp)
    hip_mid_local = (0.0, 0.0, m.jp["hip_L"][1][2])
    xp = E.sub((p["hip_x"], p["hip_y"], p["hip_z"]), E.mv(Rp, hip_mid_local))
    s.set_body(st, b["pelvis"], xp, qp)
    qt = E.qmul(_lean(p["tr_lp"], p["tr_lq"]), E.qaxis(UP, p["tr_yaw"] * D2R))
    Rt = E.qR(qt)
    lum = E.add(xp, E.mv(Rp, m.jp["lumbar"][1]))
    xt = E.sub(lum, E.mv(Rt, m.jp["lumbar"][3]))
    s.set_body(st, b["trunk"], xt, qt)
    Lth, Lsh, lu, lf = m.len["thigh"], m.len["shank"], m.len["upperarm"], m.len["forearm"]
    bm = ball_mid(m)
    short = {}
    for sd, sg in B.SIDES:
        yaw = p["f%s_yaw" % sd] * D2R
        fx = (math.cos(yaw), math.sin(yaw), 0.0)
        qf = E.qmul(E.axes_to_q(fx, E.cross(UP, fx), UP), E.qaxis((0.0, 1.0, 0.0), p["f%s_pitch" % sd] * D2R))
        Rf = E.qR(qf)
        xf = E.sub((p["f%s_x" % sd], p["f%s_y" % sd], p["f%s_z" % sd]), E.mv(Rf, bm))
        s.set_body(st, b["foot_" + sd], xf, qf)
        ankle = E.add(xf, E.mv(Rf, m.ankle_f))
        hip = E.add(xp, E.mv(Rp, m.jp["hip_" + sd][1]))
        short["leg_" + sd] = E.norm(E.sub(hip, ankle)) - 0.9999 * (Lth + Lsh)
        # the knee bends in the plane of the foot's own length: the ankle has no turn about the shin
        knee = B._two_link(hip, ankle, Lth, Lsh, E.mv(Rf, (1.0, 0.0, 0.0)))
        zt = E.unit(E.sub(hip, knee)); zs = E.unit(E.sub(knee, ankle))
        ax = E.unit(E.cross(zt, zs))
        for body, z, top, c in (("thigh_", zt, hip, m.jp["hip_" + sd][3]), ("shank_", zs, knee, m.jp["knee_" + sd][3])):
            ex, ey, ez = B._frame_z(z, ax)
            q = E.axes_to_q(ex, ey, ez)
            s.set_body(st, b[body + sd], E.sub(top, E.mv(E.qR(q), c)), q)
    sho = {sd: E.add(xt, E.mv(Rt, m.jp["shoulder_" + sd][1])) for sd, sg in B.SIDES}
    mid = E.scale(E.add(sho["L"], sho["R"]), 0.5)
    hands = E.add(mid, E.mv(Rt, (p["hand_f"], p["hand_l"], p["hand_u"])))
    az, el = p["bat_az"] * D2R, p["bat_el"] * D2R
    bz = (math.cos(el) * math.cos(az), math.cos(el) * math.sin(az), math.sin(el))
    xb, qb, Rb = bat_frame(m, p, hands, mid, bz)
    s.set_body(st, b["bat"], xb, qb)
    for sd, sg in B.SIDES:
        wrist = E.add(xb, E.mv(Rb, m.wrist_b[sd]))
        short["arm_" + sd] = E.norm(E.sub(wrist, sho[sd])) - 0.9999 * (lu + lf)
        elbow = elbow_at(sho[sd], wrist, lu, lf, sg, p["sw_" + sd])
        zu = E.unit(E.sub(sho[sd], elbow)); zf = E.unit(E.sub(elbow, wrist))
        ex, ey, ez = B._frame_z(zu, E.unit(E.cross(zu, zf)))
        q = E.axes_to_q(ex, ey, ez)
        s.set_body(st, b["uarm_" + sd], E.sub(sho[sd], E.mv(E.qR(q), m.jp["shoulder_" + sd][3])), q)
        fxa = E.scale(E.unit(E.cross(E.mv(Rb, m.palm[sd]), zf)), -sg)   # the forearm's radial line, square to the hand's palm
        q = E.axes_to_q(fxa, E.cross(zf, fxa), zf)
        s.set_body(st, b["farm_" + sd], E.sub(elbow, E.mv(E.qR(q), m.jp["elbow_" + sd][3])), q)
    if report is not None:
        report.update(short)
    if close and (max(short.values()) > 0.0 or s.residual(st)[0] > 1e-9):
        # a limb that cannot reach, or the ankle's two axes not square: close the joints with the feet, pelvis, trunk and
        # bat held as hard as they can be (the limbs give)
        lin = [1.0 / mm for mm in s.m]
        ang = [1.0] * s.nb
        for n, i in b.items():
            if n.startswith("foot"):
                lin[i] = ang[i] = 0.0
            elif n in ("pelvis", "trunk", "bat"):
                lin[i] *= 1e-3; ang[i] = 1e-3
        st = s.project(st, lin, ang)
    return st


def least_strain(m, p, roll0=None, sw0=None, span=180, step=10):
    """for the pose numbers p: the bat's roll and the two elbows' swivels at which the arm joint nearest an end of its
    range is furthest from it (body.strain, with the wrists' ellipses), searched on a grid; near (roll0, sw0) when given.
    A HELP FOR WRITING THE ROWS, not part of the motion: returns [(worse arm's strain, roll, swivel L, swivel R, the two
    arms' angles)], best first."""
    qt = E.qmul(_lean(p["tr_lp"], p["tr_lq"]), E.qaxis(UP, p["tr_yaw"] * D2R))
    Rt = E.qR(qt)
    st = pose(m, dict(p, bat_roll=0.0), close=False)
    sho = {sd: m.joint_pos(st, "shoulder_" + sd) for sd in "LR"}
    mid = E.scale(E.add(sho["L"], sho["R"]), 0.5)
    hands = E.add(mid, E.mv(Rt, (p["hand_f"], p["hand_l"], p["hand_u"])))
    az, el = p["bat_az"] * D2R, p["bat_el"] * D2R
    bz = (math.cos(el) * math.cos(az), math.cos(el) * math.sin(az), math.sin(el))
    lu, lf = m.len["upperarm"], m.len["forearm"]
    out = []
    rolls = range(-180, 180, step) if roll0 is None else range(int(roll0) - span, int(roll0) + span + 1, step)
    for roll in rolls:
        xb, qb, Rb = bat_frame(m, dict(p, bat_roll=float(roll)), hands, mid, bz)
        best = {}
        for sd, sg in B.SIDES:
            wrist = E.add(xb, E.mv(Rb, m.wrist_b[sd]))
            sws = range(-90, 181, step) if sw0 is None else range(int(sw0[sd]) - span, int(sw0[sd]) + span + 1, step)
            for sw in sws:
                elbow = elbow_at(sho[sd], wrist, lu, lf, sg, float(sw))
                ang = B.arm_angles(sd, E.unit(E.sub(sho[sd], elbow)), E.unit(E.sub(elbow, wrist)), Rb, Rt)
                c = B.strain(ang, oval=True)
                if sd not in best or c < best[sd][0]:
                    best[sd] = (c, sw, ang)
        out.append((max(best["L"][0], best["R"][0]), roll, best["L"][1], best["R"][1], best["L"][2], best["R"][2]))
    return sorted(out, key=lambda r: r[0])


def with_velocities(m, states, dt):
    """each state given the velocities its neighbours imply (central differences; one-sided at the ends)."""
    s = m.sys
    n = len(states)
    out = []
    for k in range(n):
        a, c = states[max(0, k - 1)], states[min(n - 1, k + 1)]
        span = dt * (min(n - 1, k + 1) - max(0, k - 1))
        st = list(states[k])
        for i in range(s.nb):
            o = 13 * i
            for q in range(3):
                st[o + 7 + q] = (c[o + q] - a[o + q]) / span
            dq = E.qmul(tuple(c[o + 3:o + 7]), E.qconj(tuple(a[o + 3:o + 7])))
            if dq[0] < 0:
                dq = tuple(-x for x in dq)
            sn = math.sqrt(dq[1] ** 2 + dq[2] ** 2 + dq[3] ** 2)
            kk = 2.0 * math.atan2(sn, dq[0]) / sn / span if sn > 1e-14 else 0.0
            st[o + 10:o + 13] = [dq[1] * kk, dq[2] * kk, dq[3] * kk]
        out.append(st)
    return out


def build(motion=None, m=None, log=None, close=True):
    """(model, times, states with velocities, the worst a limb falls short by): the written motion on the body."""
    motion = Motion() if motion is None else motion
    m = new_model() if m is None else m
    states, worst = [], {}
    off = 0.03                                   # the hips ahead of the centre of mass: found for each instant
    motion.hip_y = []
    for i in range(motion.n):
        p = motion.at(i)
        for _ in range(6):
            rep = {}
            p["hip_y"] = p["com_y"] + off
            st = pose(m, p, rep, close)
            err = B.com(m, st)[0][1] - p["com_y"]
            if abs(err) < 2e-6:
                break
            off -= err
        motion.hip_y.append(p["hip_y"])
        states.append(st)
        for k, v in rep.items():
            if v > worst.get(k, (-1e9, 0.0))[0]:
                worst[k] = (v, motion.ts[i])
    states = with_velocities(m, states, motion.dt)
    if log is not None:
        for k, (v, t) in sorted(worst.items()):
            if v > 0.0:
                log("TOO SHORT: %s cannot reach what is written by %.1f cm at %+.0f ms" % (k, 100 * v, 1e3 * t))
    return m, motion.ts, states, worst


# ---- what a baseball man would read ------------------------------------------------------------------------------------
def unwrap(prev, x):
    while x - prev > 180.0:
        x -= 360.0
    while x - prev < -180.0:
        x += 360.0
    return x


def com_of(m, st):
    return B.com(m, st)


def zmp(m, sm, s0, sp, dt):
    """where on the ground the feet's push must act for the body to move as written (the zero-moment point), from three
    states dt apart: the centre of mass's acceleration and the rate of change of the whole body's angular momentum."""
    s = m.sys
    Mt = sum(s.m)
    def lin_ang(st):
        c, v = B.com(m, st)
        L = [0.0, 0.0, 0.0]
        for i in range(s.nb):
            x, q, vv, w = s.pose(st, i)
            R = E.qR(q)
            I = s.I[i]
            l = E.mtv(R, w)
            h = E.mv(R, (I[0] * l[0], I[1] * l[1], I[2] * l[2]))
            r = E.sub(x, c)
            hh = E.add(h, E.scale(E.cross(r, E.sub(vv, v)), s.m[i]))
            L = [L[0] + hh[0], L[1] + hh[1], L[2] + hh[2]]
        return c, v, L
    cm, vm, Lm = lin_ang(sm)
    c0, v0, L0 = lin_ang(s0)
    cp, vp, Lp = lin_ang(sp)
    a = [(vp[k] - vm[k]) / (2 * dt) for k in range(3)]
    dL = [(Lp[k] - Lm[k]) / (2 * dt) for k in range(3)]
    fz = Mt * (a[2] + E.G)
    px = c0[0] - (c0[2] * Mt * a[0] + dL[1]) / fz
    py = c0[1] - (c0[2] * Mt * a[1] - dL[0]) / fz
    return (px, py), fz / (Mt * E.G), (Mt * a[0] / (Mt * E.G), Mt * a[1] / (Mt * E.G)), dL[2]


def read(m, ts, states, step_ms=10.0):
    """one row every step_ms: a dict of what is looked at (angles deg, lengths m, rates deg/s)."""
    rows = []
    yaw = {}
    first = None
    every = int(round(step_ms * 1e-3 / (ts[1] - ts[0])))
    for i, st in enumerate(states):
        o = m.observe(st)
        cur = {"pelvis": o["pelvis_rot"], "trunk": o["trunk_rot"], "bat": o["bat_az"]}
        for k, v in cur.items():
            yaw[k] = v if k not in yaw else unwrap(yaw[k], v)
        if first is None:
            first = dict(yaw)
        t_ms = ts[i] * 1e3
        if abs(t_ms / step_ms - round(t_ms / step_ms)) > 1e-6:
            continue
        jp = lambda n: m.joint_pos(st, n)
        zt = m.axis(st, "trunk", (0, 0, 1))
        com, vcom = B.com(m, st)
        hands = m.bat_point(st, MID_HANDS_U)
        ss, vss = m.bat_point(st, C.BAT_SS)
        shR = jp("shoulder_R")
        aL, aR = jp("ankle_L"), jp("ankle_R")
        r = dict(t=t_ms, pelvis=yaw["pelvis"], trunk=yaw["trunk"], sep=yaw["pelvis"] - yaw["trunk"], pelvis_rate=o["pelvis_rot_vel"], trunk_rate=o["trunk_rot_vel"],
                 lean_plate=math.asin(max(-1.0, min(1.0, zt[0]))) / D2R, lean_pitcher=math.asin(max(-1.0, min(1.0, zt[1]))) / D2R,
                 knee_L=o["lead_knee_flex"], knee_R=o["trail_knee_flex"], elbow_L=o["lead_elbow_flex"], elbow_R=o["trail_elbow_flex"],
                 com_y=com[1], com_x=com[0], com_z=com[2], com_vy=vcom[1], stride=aL[1] - aR[1],
                 hands_speed=E.norm(hands[1]), hands_z=hands[0][2], hands_behind=shR[1] - hands[0][1], hands_below=shR[2] - hands[0][2],
                 bat_down=-o["bat_elev"], bat_az=yaw["bat"], sweet=E.norm(vss), sweet_z=ss[2], attack=o["attack"], lag=o["bat_lag"],
                 sweet_x=ss[0], sweet_y=ss[1])
        if 0 < i < len(states) - 1:
            (zx, zy), fz, fh, dLz = zmp(m, states[i - 1], st, states[i + 1], ts[1] - ts[0])
            r.update(zmp_x=zx, zmp_y=zy, fz=fz, fh_x=fh[0], fh_y=fh[1], turn_moment=dLz)
        else:
            r.update(zmp_x=com[0], zmp_y=com[1], fz=1.0, fh_x=0.0, fh_y=0.0, turn_moment=0.0)
        rows.append(r)
    return rows


def table(rows, log=print):
    log("t ms | buckle; chest (deg, 0 at the plate, 90 at the pitcher); hips ahead of chest | their rates deg/s | spine leaned toward the plate; the pitcher | knees front; rear | elbows lead; rear"
        " | hands: m/s, behind the rear shoulder cm, below it cm | barrel: round deg, below level deg, behind the lead arm deg | sweet spot m/s (mph), height cm, rising deg"
        " | centre of mass toward the pitcher cm (its speed m/s) | where the ground must push: toward the pitcher cm, toward the plate cm; how hard (x his weight)")
    for r in rows:
        log("%+5.0f | %+4.0f %+4.0f %+4.0f | %5.0f %5.0f | %+3.0f %+3.0f | %3.0f %3.0f | %3.0f %3.0f | %4.1f %+4.0f %+4.0f | %+5.0f %+4.0f %4.0f | %4.1f (%2.0f) %3.0f %+4.0f | %+4.0f (%+.2f) | %+4.0f %+4.0f %.2f"
            % (r["t"], r["pelvis"], r["trunk"], r["sep"], r["pelvis_rate"], r["trunk_rate"], r["lean_plate"], r["lean_pitcher"], r["knee_L"], r["knee_R"],
               r["elbow_L"], r["elbow_R"], r["hands_speed"], 100 * r["hands_behind"], 100 * r["hands_below"], r["bat_az"], r["bat_down"], r["lag"],
               r["sweet"], r["sweet"] / 0.44704, 100 * r["sweet_z"], r["attack"], 100 * r["com_y"], r["com_vy"], 100 * r["zmp_y"], 100 * r["zmp_x"], r["fz"]))


def ranges_report(m, ts, states, log=print, every=5):
    """each joint's reading against the end of its measured range (body.ROM), over the motion: the furthest it goes past."""
    specs = {t[0]: (t[6], t[7]) for t in B.range_specs(None, "hands", "cleats", "oval")}
    worst = {}
    for i in range(0, len(states), every):
        a = B.range_angles(m, states[i])
        for name, (lo, hi) in specs.items():
            v = a[name]
            if name.startswith("wrist_oval"):
                over = v
            else:
                over = max(v - hi, lo - v)
            if over > worst.get(name, (-1e9, 0, 0))[0]:
                worst[name] = (over, ts[i], v)
    log("joints past the end of their measured range (deg past; when; the reading):")
    any_ = False
    for name, (over, t, v) in sorted(worst.items(), key=lambda kv: -kv[1][0]):
        if over > 0.0:
            any_ = True
            log("   %-16s %5.1f past at %+5.0f ms (reads %+.1f, range %+.1f to %+.1f)" % (name, over, 1e3 * t, v, specs[name][0], specs[name][1]))
    if not any_:
        log("   none")
    return worst


# ---- drawings ----------------------------------------------------------------------------------------------------------
def head_of(m, st, look_at):
    """(the head's centre, its facing as a quaternion): the head sits on the top of the trunk and turns to look at
    look_at, as far as a neck turns (80 deg from the chest's facing [E], 35 deg up or down). The model has no neck or head
    of its own: the head's mass is the trunk's, and this turn is drawn, not computed."""
    s = m.sys
    xt, qt, _, _ = s.pose(st, m.b["trunk"])
    Rt = E.qR(qt)
    shm = E.scale(E.add(m.joint_pos(st, "shoulder_L"), m.joint_pos(st, "shoulder_R")), 0.5)
    zt = E.mv(Rt, (0, 0, 1))
    centre = E.add(shm, E.scale(zt, 0.20))
    d = E.mtv(Rt, E.unit(E.sub(look_at, centre)))                 # the line of sight in the chest's own directions
    yaw = math.atan2(d[1], d[0])
    yaw = max(-80.0 * D2R, min(80.0 * D2R, yaw))
    pit = math.asin(max(-1.0, min(1.0, d[2])))
    pit = max(-35.0 * D2R, min(35.0 * D2R, pit))
    q = E.qmul(qt, E.qmul(E.qaxis((0, 0, 1), yaw), E.qaxis((0, 1, 0), -pit)))
    return centre, q


def ball_line(pc, t):
    """the drawn pitch's centre at time t from contact, on its straight line through the meeting point pc."""
    d, a = BALL_DESCENT * D2R, BALL_ACROSS * D2R
    nb = (math.cos(d) * math.sin(a), -math.cos(d) * math.cos(a), -math.sin(d))      # the way the ball travels
    return E.add(pc, E.scale(nb, BALL_SPEED * t)), nb


def draw(m, frames, out_dir, per_sheet=6, tag="motion", pc=None, title=None):
    """line drawings of frames [(t s, state, {foot: share of his weight} or None, a line of text or None)], three sides,
    per_sheet instants a sheet: from the pitcher's side, from the open side (the pitcher to the right) and from above (the
    pitcher at the top). Lead arm and leg blue, rear red, trunk black, bat brown, ball orange."""
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    s = m.sys
    os.makedirs(out_dir, exist_ok=True)
    views = (("from the pitcher's side", lambda p: (-p[0], p[2])), ("from the open side (pitcher to the right)", lambda p: (p[1], p[2])),
             ("from above (pitcher at the top)", lambda p: (p[0], p[1])))
    x0 = s.pose(frames[0][1], m.b["pelvis"])[0]
    plate = [(0.0, 0.0), (0.216, 0.216), (0.216, 0.432), (-0.216, 0.432), (-0.216, 0.216), (0.0, 0.0)]
    paths = []
    for sheet in range(0, len(frames), per_sheet):
        part = frames[sheet:sheet + per_sheet]
        fig, axs = plt.subplots(3, per_sheet, figsize=(3.0 * per_sheet, 9.6), dpi=100)
        for c, (t, st, loads, text) in enumerate(part):
            jp = lambda n: m.joint_pos(st, n)
            shm = E.scale(E.add(jp("shoulder_L"), jp("shoulder_R")), 0.5)
            look = ball_line(pc, min(t, 0.0))[0] if pc is not None else (0.0, 18.0, 1.8)
            head, qh = head_of(m, st, look)
            nose = E.add(head, E.scale(E.mv(E.qR(qh), (1, 0, 0)), 0.13))
            knob, tip = m.bat_point(st, 0.0)[0], m.bat_point(st, C.BAT_L)[0]
            feet = {}
            for sd in "LR":
                xb, qb, _, _ = s.pose(st, m.b["foot_" + sd])
                feet[sd] = {c_[0][2:]: E.add(xb, E.mv(E.qR(qb), c_[2])) for c_ in s.contacts if c_[0].startswith(sd)}
            o = m.observe(st)
            for v, (vname, pr) in enumerate(views):
                ax = axs[v][c]
                line = lambda a, b_, col, lw=2.5, z=3: ax.plot(*zip(pr(a), pr(b_)), color=col, lw=lw, solid_capstyle="round", zorder=z)
                if v < 2:
                    ax.axhline(0.0, color="#8a6", lw=1)
                else:
                    ax.plot([q[0] for q in plate], [q[1] for q in plate], color="#999", lw=1, zorder=0)
                for sd, col in (("R", "#c33"), ("L", "#25c")):
                    line(jp("hip_" + sd), jp("knee_" + sd), col); line(jp("knee_" + sd), jp("ankle_" + sd), col)
                    f = feet[sd]
                    heel = E.scale(E.add(f["heel_in"], f["heel_out"]), 0.5)
                    toe = E.scale(E.add(f["ball_in"], f["ball_out"]), 0.5)
                    line(jp("ankle_" + sd), heel, col, 2); line(heel, toe, col, 3); line(jp("ankle_" + sd), toe, col, 1.5)
                    line(jp("shoulder_" + sd), jp("elbow_" + sd), col, 2.2, 5 if sd == "L" else 2); line(jp("elbow_" + sd), jp("wrist_" + sd), col, 2.2, 5 if sd == "L" else 2)
                line(jp("hip_L"), jp("hip_R"), "k", 3); line(jp("shoulder_L"), jp("shoulder_R"), "k", 3)
                hipm = E.scale(E.add(jp("hip_L"), jp("hip_R")), 0.5)
                line(hipm, jp("lumbar"), "k", 3); line(jp("lumbar"), shm, "k", 3); line(shm, head, "k", 2)
                ax.add_patch(plt.Circle(pr(head), 0.10, fill=False, color="k", lw=1.5, zorder=4))
                line(head, nose, "k", 1.5, 4)
                line(knob, tip, "#850", 4, 6)
                if pc is not None:
                    ball = ball_line(pc, t)[0]
                    if t <= 0.0005 and abs(ball[1] - pc[1]) < 2.4:
                        ax.add_patch(plt.Circle(pr(ball), 0.0366, color="#f80", zorder=7))
                    ax.plot(*pr(pc), marker="+", color="#f80", ms=8, zorder=1)
                ax.set_aspect("equal")
                if v == 0:
                    ax.set_xlim(-x0[0] - 1.25, -x0[0] + 1.0); ax.set_ylim(-0.1, 2.15)
                    ax.set_title("%+.0f ms   bat %.0f mph" % (1e3 * t, o["sweet_speed"] / 0.44704), fontsize=10)
                    if loads is not None:
                        ax.text(0.02, 0.02, "front %.2f  rear %.2f" % (loads["L"], loads["R"]), transform=ax.transAxes, fontsize=8)
                elif v == 1:
                    ax.set_xlim(x0[1] - 1.1, x0[1] + 1.4); ax.set_ylim(-0.1, 2.15)
                    ax.text(0.02, 0.02, "barrel %.0f below level" % (-o["bat_elev"]), transform=ax.transAxes, fontsize=8)
                else:
                    ax.set_xlim(x0[0] - 0.9, x0[0] + 1.5); ax.set_ylim(x0[1] - 1.1, x0[1] + 1.4)
                    ax.text(0.02, 0.02, text or "", transform=ax.transAxes, fontsize=8)
                ax.set_xticks([]); ax.set_yticks([])
                if c == 0:
                    ax.set_ylabel(vname, fontsize=8)
        for c in range(len(part), per_sheet):
            for v in range(3):
                axs[v][c].axis("off")
        if title:
            fig.suptitle(title, fontsize=9, y=0.999)
        fig.tight_layout(pad=0.3)
        path = os.path.join(out_dir, "%s_%02d_%+05.0f_to_%+05.0f_ms.png" % (tag, sheet // per_sheet + 1, 1e3 * part[0][0], 1e3 * part[-1][0]))
        fig.savefig(path)
        plt.close(fig)
        paths.append(path)
    return paths


def frames_of(m, ts, states, step_ms=10.0, t_from=None, t_to=None):
    rows = {round(r["t"]): r for r in read(m, ts, states, step_ms)}
    out = []
    for i, st in enumerate(states):
        t_ms = ts[i] * 1e3
        if abs(t_ms / step_ms - round(t_ms / step_ms)) > 1e-6:
            continue
        if (t_from is not None and ts[i] < t_from - 1e-9) or (t_to is not None and ts[i] > t_to + 1e-9):
            continue
        r = rows[round(t_ms)]
        out.append((ts[i], st, None, "buckle %+.0f  chest %+.0f" % (r["pelvis"], r["trunk"])))
    return out


# ---- the viewer's file ---------------------------------------------------------------------------------------------------
BODY_ORDER = ("pelvis", "trunk", "thigh_L", "shank_L", "foot_L", "thigh_R", "shank_R", "foot_R", "uarm_L", "farm_L", "uarm_R", "farm_R", "bat")
JOINT_ORDER = ("lumbar", "hip_L", "knee_L", "ankle_L", "hip_R", "knee_R", "ankle_R", "shoulder_L", "elbow_L", "wrist_L", "shoulder_R", "elbow_R", "wrist_R")
POINTS = ("knob", "sweet", "tip", "grip_L", "grip_R")
FORMAT = "bb-body-swing 1"


def contact_point(m, ts, states):
    """where and when the sweet spot is at the written contact (t = 0)."""
    i = min(range(len(ts)), key=lambda k: abs(ts[k]))
    p, v = m.bat_point(states[i], C.BAT_SS)
    return i, p, v


def write_swing(path, m, ts, states, title, kind, events, loads=None, pc=None, extra=(), name="swing_look", every=1):
    """the viewer's file (docs/player-visuals.md 14) of states on the model m. loads: per state, the upward force on each
    sole point (N) or None (written as 0). pc: the meeting point (default: where the sweet spot is at t = 0). kind:
    "recorded" has the viewer's pictures run to the last sample. extra: more header lines."""
    s = m.sys
    i0, p0, v0 = contact_point(m, ts, states)
    pc = p0 if pc is None else pc
    us = {"knob": 0.0, "sweet": C.BAT_SS, "tip": C.BAT_L, "grip_L": B.GRIP_L, "grip_R": B.GRIP_R}
    bix = [m.b[n] for n in BODY_ORDER]
    rows = []
    for k in range(0, len(states), every):
        st, t = states[k], ts[k]
        row = [t]
        for b in bix:
            x, q, _, _ = s.pose(st, b)
            row += list(x) + list(q)
        look = ball_line(pc, min(t, 0.0))[0]
        hc, hq = head_of(m, st, look)
        row += list(hc) + list(hq)
        for j in JOINT_ORDER:
            row += list(m.joint_pos(st, j))
        for kk in POINTS:
            row += list(m.bat_point(st, us[kk])[0])
        row.append(E.norm(m.bat_point(st, C.BAT_SS)[1]))
        for ci, (cname, b, pl, *_r) in enumerate(s.contacts):
            xb, qb, _, _ = s.pose(st, b)
            row += list(E.add(xb, E.mv(E.qR(qb), pl))) + [0.0 if loads is None else loads[k][ci]]
        row += list(ball_line(pc, t)[0])
        rows.append(row)
    sp = E.norm(v0)
    nb_, nc_ = len(BODY_ORDER) + 1, len(s.contacts)
    w = ["# %s" % FORMAT,
         "# A swing of the force-driven body's skeleton for the Unreal viewer (client/run.sh -BbBodySwing=<file>; PLAN D43, D48),",
         "# written by analysis/research/body/swing_author.py. A VIEWING AID: no game reads it. Layout: docs/player-visuals.md 14.",
         "# Sim frame (Statcast): metres, +X toward first base, +Y toward the pitcher, +Z up; a right-handed batter. L is his lead",
         "# (left) side. Quaternions (w, x, y, z): world = R(q) local. Times are seconds from contact.",
         "# The head is DRAWN looking at the ball as far as a neck turns; the model has no neck (the head's mass is the trunk's).",
         "format %s" % FORMAT, "kind %s" % kind, "title %s" % title, "plan %s" % name,
         "batter_height_m %.4f" % m.H, "batter_mass_kg %.3f" % m.M,
         "bat length_m %.4f mass_kg %.4f com_from_knob_m %.4f i_cm_kgm2 %.5f i_axis_kgm2 %.6g sweet_from_knob_m %.4f" % (C.BAT_L, C.BAT_M, C.BAT_UCM, C.BAT_ICM, C.BAT_IAX, C.BAT_SS),
         "contact_speed_mps %.4f" % sp, "contact_speed_mph %.2f" % (sp / 0.44704),
         "miss_cm up 0 along_bat 0 total %.3f" % (100 * E.norm(E.sub(p0, pc))),
         "contact t_s 0 sweet_at %.5f %.5f %.5f sweet_vel %.4f %.4f %.4f meet_point %.5f %.5f %.5f" % (tuple(p0) + tuple(v0) + tuple(pc)),
         "ball speed_mps %.3f descent_deg %.3f across_deg %.3f radius_m 0.0366" % (BALL_SPEED, BALL_DESCENT, BALL_ACROSS)]
    w += list(extra)
    for nm, t, why in events:
        w.append("event %s %.6f    # %s" % (nm, t, why))
    w += ["rate_hz %.0f" % (1.0 / ((ts[1] - ts[0]) * every)),
          "bodies %d %s head" % (nb_, " ".join(BODY_ORDER)), "joints %d %s" % (len(JOINT_ORDER), " ".join(JOINT_ORDER)),
          "points %d %s" % (len(POINTS), " ".join(POINTS)), "contacts %d %s" % (nc_, " ".join(c[0] for c in s.contacts)),
          "# each sample row: s t | bodies x y z qw qx qy qz | joints x y z | points x y z | sweet_speed_mps | contacts x y z fz_N | ball x y z",
          "columns %d" % (1 + 7 * nb_ + 3 * len(JOINT_ORDER) + 3 * len(POINTS) + 1 + 4 * nc_ + 3), "samples %d" % len(rows)]
    for r in rows:
        w.append("s " + " ".join("%.7g" % x if abs(x) >= 1e-3 or x == 0.0 else "%.6e" % x for x in r))
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    with open(path, "w") as fh:
        fh.write("\n".join(w) + "\n")
    return path


def main():
    a = sys.argv[1:]
    arg = lambda k, d: a[a.index(k) + 1] if k in a else d
    if not a:
        print(__doc__)
        return
    m, ts, states, worst = build(log=print, close="--fast" not in a)
    i0, pc, v0 = contact_point(m, ts, states)
    print("the written motion: %d ms of stance, load and stride, the swing, %d ms of finish; the sweet spot at contact %.1f m/s (%.0f mph) at x %+.2f y %+.2f z %.2f m"
          % (-1e3 * ts[0], 1e3 * ts[-1], E.norm(v0), E.norm(v0) / 0.44704, pc[0], pc[1], pc[2]))
    step = float(arg("--step", 10))
    if "--table" in a:
        t_a, t_b = float(arg("--from", ts[0])), float(arg("--to", ts[-1]))
        table([r for r in read(m, ts, states, step) if t_a * 1e3 - 1e-6 <= r["t"] <= t_b * 1e3 + 1e-6])
        ranges_report(m, ts, states)
    if "--draw" in a:
        fr = frames_of(m, ts, states, step, float(arg("--from", ts[0])), float(arg("--to", ts[-1])))
        for p in draw(m, fr, arg("--draw", "."), pc=pc, title="THE WRITTEN MOTION (no forces): what the muscles are asked to produce"):
            print("drew " + p)
    if "--arms" in a:
        mo = Motion()
        for t, d in KEYS:
            pp = mo.at(mo.index(t))
            pp["hip_y"] = mo.hip_y[mo.index(t)] if hasattr(mo, "hip_y") else pp["com_y"]
            r = least_strain(m, pp)[:3]
            print("%+6.3f now roll %+4.0f sw %+4.0f %+4.0f | least strain: " % (t, pp["bat_roll"], pp["sw_L"], pp["sw_R"])
                  + "; ".join("%.2f at roll %+4d sw %+4d %+4d" % x[:4] for x in r))
    if "--swing" in a:
        p = write_swing(arg("--swing", None), m, ts, states, "THE SWING AS WRITTEN BY HAND, NO FORCES IN IT (PLAN D48): what the muscles are then asked to produce",
                        "recorded", EVENTS, name="swing_author.py")
        print("wrote " + p)


if __name__ == "__main__":
    main()
