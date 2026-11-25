"""The forward driven batter of the body-motion probes (docs/body-motion.md 4): stdlib only.

A reduced model, in the horizontal plane (the swing plane taken as horizontal; docs 4.1 says what that
leaves out), that answers the design questions with real dynamics:

  s      the body's forward position over the feet (m, along the line to the pitcher; rear foot at 0)
  phi1   pelvis yaw              (rotor about the vertical through the base; 0 = facing the plate)
  phi2   upper trunk yaw         (rotor; carries the lead shoulder at r_sh on its lead side)
  phi3   the arms                (a link from the lead shoulder to the hands, both arms' masses on it)
  phi4   the bat                 (hinged at the hands; its cock is lag = phi3 - phi4)

Muscle drive, per channel (docs 4.2): tau = T * a(t) * fv(w_rel / W), the excitation a step on at
t_on (off at t_off), activation first order (15 ms up, 50 ms down), Hill's force-velocity relation
(curvature 0.25 concentric, up to 1.5 eccentric). Channels: legs (ground -> pelvis yaw), core
(pelvis -> trunk), shoulder (trunk -> arms), wrist (arms -> bat, uncocking), rear leg (push along s),
lead leg (brake along s, from foot-down).

Ground (docs 4.1): feet fixed; the vertical load split by the lever rule; the legs' yaw torque is capped
by friction (a lateral couple across the two feet plus each foot's free moment); the push and the brake
along s are capped by friction on the foot that makes them. The wrist has a passive stop at the
largest cock, one-sided (it never pulls; docs 4.3 gives the swing with it, capped and without).
Integrator: classical RK4 at a fixed step.
"""
import math, sys, os
sys.path.insert(0, os.path.dirname(__file__))
from common import *

D2R = math.pi / 180

CHANNELS = ("legs", "core", "shoulder", "wrist", "rear", "lead")


class Body:
    """Segment and geometry parameters for a batter of stature H (m) and mass M (kg)."""

    def __init__(self, H=1.84, M=92.0, reach=1.0):
        self.H, self.M = H, M
        _, m_lpt, _, _, _, I_lpt = segment("lpt", H, M)
        _, m_th, _, _, _, I_th = segment("thigh", H, M)
        # [E] half the thighs turn with the pelvis (the feet are planted; the hips rotate inside them)
        self.I1 = I_lpt + 0.5 * 2 * (I_th + m_th * (0.054 * H) ** 2)
        self.I2 = sum(segment(s, H, M)[5] for s in ("upt", "mpt", "head"))
        ua, m_ua = segment("upperarm", H, M)[0], 2 * segment("upperarm", H, M)[1]
        fa, m_fa = segment("forearm", H, M)[0], 2 * segment("forearm", H, M)[1]
        m_ha = 2 * segment("hand", H, M)[1]
        self.r_sh = 0.112 * H                    # [E] lead shoulder from the trunk axis (swing-model-v3 5.2)
        self.L_arm = 0.30 * H * reach            # [E] lead shoulder -> hands, horizontal, through the swing
        f = self.L_arm / (ua + fa)
        self.arm_pts = [(m_ua, 0.5772 * ua * f), (m_fa, (ua + 0.4574 * fa) * f), (m_ha, self.L_arm)]
        self.I3 = 2 * (segment("upperarm", H, M)[4] + segment("forearm", H, M)[4])   # own transverse
        self.cb = BAT_UCM - GRIP_FROM_KNOB
        self.ss = BAT_SS - GRIP_FROM_KNOB
        self.Ib = BAT_ICM
        self.mb = BAT_M
        # translating mass: all but the shanks and feet [E]
        self.m_tr = M * (1 - 2 * (0.0433 + 0.0137))
        self.stride = 0.42 * H                   # [4] toe-to-toe at foot down, 42 % of stature
        self.lag_stop = 133 * D2R                # [4] largest bat lag (-80 ms): the wrist's cock limit [E as a stop]
        # what holds the cock at lag_stop: None = a stiff one-sided stop (a device: it may push the bat
        # forward without limit but never pull it back); a number = that stop's torque capped there, N m
        # (two wrists' measured holding strength is about 20, note 5, abstract only); lag_stop = 10 rad
        # removes the stop altogether
        self.wrist_cap = None


class Plan:
    """A learned swing: per channel (T peak, t_on, t_off, W max speed); foot-down time; start state."""

    def __init__(self, ch, t_fd, start, mu=0.8, r_free=0.05):
        self.ch = ch                  # {name: [T, t_on, t_off, W]}
        self.t_fd = t_fd
        self.start = start            # (t0, s, s', phi1..4, w1..4)
        self.mu, self.r_free = mu, r_free


TAU_LOAD = 0.17                       # lead foot's loading time after touch-down [E from [4]'s 10 -> 50 % BW]
SEP_STOP = 45 * D2R                   # trunk-on-pelvis axial rotation limit [E: a range of motion, unsourced]
TA_UP, TA_DN = 0.015, 0.050           # activation time constants (s), OpenSim Thelen2003Muscle defaults [E]
K_FV = 0.25                           # Hill curvature a/F0 [E: Hill 1938's order]
ECC = 1.5                             # eccentric plateau [E]


def fv(w):
    """force-velocity factor, w = shortening speed / max speed (negative = lengthening)."""
    if w >= 1.0:
        return 0.0
    if w >= 0.0:
        return (1.0 - w) / (1.0 + w / K_FV)
    return ECC - (ECC - 1.0) / (1.0 - 4.0 * w)


def solve(A, b):
    n = len(b)
    A = [row[:] + [b[i]] for i, row in enumerate(A)]
    for c in range(n):
        p = max(range(c, n), key=lambda r: abs(A[r][c]))
        A[c], A[p] = A[p], A[c]
        for r in range(c + 1, n):
            f = A[r][c] / A[c][c]
            for k in range(c, n + 1):
                A[r][k] -= f * A[c][k]
    x = [0.0] * n
    for r in range(n - 1, -1, -1):
        x[r] = (A[r][n] - sum(A[r][k] * x[k] for k in range(r + 1, n))) / A[r][r]
    return x


class Sim:
    def __init__(self, body, plan, dt=0.0005, record=False):
        self.b, self.p, self.dt, self.record = body, plan, dt, record

    # ---- kinematics: point masses as sums of terms (angle index 1..4, radius, angle offset)
    def points(self):
        b = self.b
        pts = [(b.m_tr - sum(m for m, _ in b.arm_pts), [])]     # body mass on the axis
        sh = (2, b.r_sh, math.pi / 2)                             # lead shoulder: trunk's lead side
        for m, r in b.arm_pts:
            pts.append((m, [sh, (3, r, 0.0)]))
        pts.append((b.mb, [sh, (3, b.L_arm, 0.0), (4, b.cb, 0.0)]))
        return pts

    def setup(self):
        self.pts = self.points()

    def mass_h(self, x):
        """the mass matrix M(q) and the velocity terms h(q, qd) of M qdd + h = Q (checks.py tests them
        against the Lagrangian by finite differences)."""
        b = self.b
        q, qd = x[:5], x[5:]
        Mm = [[0.0] * 5 for _ in range(5)]
        h = [0.0] * 5
        for m, terms in self.pts:
            J = [[0.0] * 5, [1.0] + [0.0] * 4]   # rows x, y
            cx = cy = 0.0
            for k, r, off in terms:
                a = q[k] + off
                c, s_ = math.cos(a), math.sin(a)
                J[0][k] += -r * s_; J[1][k] += r * c
                w = qd[k]
                cx += -r * w * w * c; cy += -r * w * w * s_
            for i in range(5):
                hi = m * (J[0][i] * cx + J[1][i] * cy)
                h[i] += hi
                for j in range(5):
                    Mm[i][j] += m * (J[0][i] * J[0][j] + J[1][i] * J[1][j])
        Mm[1][1] += b.I1; Mm[2][2] += b.I2; Mm[3][3] += b.I3; Mm[4][4] += b.Ib
        return Mm, h

    def eom(self, t, x, act):
        """x = [s, p1, p2, p3, p4, sd, w1, w2, w3, w4] -> derivatives, and the drive torques used."""
        q, qd = x[:5], x[5:]
        Mm, h = self.mass_h(x)
        tau = self.forces(t, q, qd, act)
        Q = [tau["rear"] - tau["lead"],
             tau["legs"] - tau["core"],
             tau["core"] - tau["shoulder"],
             tau["shoulder"] - tau["wrist"],
             tau["wrist"]]
        qdd = solve(Mm, [Q[i] - h[i] for i in range(5)])
        return list(qd) + qdd, tau

    def loads(self, t, s):
        """vertical loads (N) by the lever rule; the lead foot carries none before it is down."""
        b, W = self.b, self.b.M * G
        if t < self.p.t_fd:
            return 0.0, W
        fl = min(1.0, max(0.0, s / b.stride))
        # the lead foot takes weight over TAU_LOAD after it touches ([4]: 10 % BW at foot down, -340 ms,
        # to 50 % at the weight commitment, -170 ms): the vertical dynamics the planar model leaves out [E]
        fl *= min(1.0, (t - self.p.t_fd) / TAU_LOAD)
        return W * fl, W * (1 - fl)

    def forces(self, t, q, qd, act):
        b, p = self.b, self.p
        ch = p.ch
        tau = {}
        rel = {"legs": qd[1], "core": qd[2] - qd[1], "shoulder": qd[3] - qd[2], "wrist": qd[4] - qd[3],
               "rear": qd[0], "lead": -qd[0]}
        for n in CHANNELS:
            T, ton, toff, Wmax = ch[n][:4]
            tau[n] = T * act[n] * fv(rel[n] / Wmax)
        # lead leg brakes only once the foot is down, and only while the body moves forward
        if t < p.t_fd or qd[0] <= 0.0:
            tau["lead"] = 0.0
        Fl, Fr = self.loads(t, q[0])
        mu = p.mu
        # friction caps (docs 4.1): push on the rear foot, brake on the lead foot
        tau["rear"] = min(tau["rear"], mu * Fr)
        tau["lead"] = min(tau["lead"], mu * Fl)
        # the legs' yaw torque: a lateral couple across the feet plus each foot's free moment
        cap = mu * min(Fl, Fr) * b.stride + mu * p.r_free * (Fl + Fr)
        self.cap = cap
        self.legs_raw = tau["legs"]           # the legs' drive before the friction cap (for the reports)
        tau["legs"] = max(-cap, min(cap, tau["legs"]))
        # passive trunk-on-pelvis range of motion (both ways) [E]
        sep = q[2] - q[1]
        if abs(sep) > SEP_STOP:
            sg = 1.0 if sep > 0 else -1.0
            tau["core"] += -sg * 2.0e3 * (abs(sep) - SEP_STOP) - 20.0 * (qd[2] - qd[1]) * (1.0 if sg * (qd[2] - qd[1]) > 0 else 0.0)
        # passive wrist stop at the cock limit: a stiff spring-damper [E: 2e4 N m/rad, 40 N m s/rad], one-sided
        # in both senses: it acts only past the limit, and its torque is never negative (it may push the
        # bat forward, never hold it back while it uncocks); optionally capped at b.wrist_cap
        lag = q[3] - q[4]
        self.stop_tau = 0.0
        if lag > b.lag_stop:
            st = max(0.0, 2.0e4 * (lag - b.lag_stop) + 40.0 * (qd[3] - qd[4]))
            if b.wrist_cap is not None:
                st = min(b.wrist_cap, st)
            self.stop_tau = st
            tau["wrist"] += st
        return tau

    def excite(self, t):
        """excitation: 0 before t_on, a linear ramp over the channel's rise time, 1 until t_off."""
        u = {}
        for n in CHANNELS:
            c = self.p.ch[n]
            ton, toff = c[1], c[2]
            rise = c[4] if len(c) > 4 else 0.0
            if t < ton or t >= toff:
                u[n] = 0.0
            elif rise > 0 and t < ton + rise:
                u[n] = (t - ton) / rise
            else:
                u[n] = 1.0
        return u

    def run(self, t_end=0.06, A_c=-8 * D2R, stop_at_contact=False):
        self.setup()
        dt = self.dt
        st = list(self.p.start)
        t = st[0]; x = st[1:]
        act = {n: 0.0 for n in CHANNELS}
        out = {"t": [], "x": [], "tau": [], "ke": [], "cap": [], "legs_raw": [], "stop": [], "work": 0.0, "contact": None}
        work = 0.0
        ang = 0.0
        while t < t_end:
            u = self.excite(t)
            # activation: exact first-order step over dt with the rate set by rising or falling
            for n in CHANNELS:
                tc = TA_UP if u[n] > act[n] else TA_DN
                act[n] = u[n] + (act[n] - u[n]) * math.exp(-dt / tc)
            # RK4 with the activation held over the step
            k1, tau = self.eom(t, x, act)
            cap_k1, raw_k1, stop_k1 = self.cap, self.legs_raw, self.stop_tau
            x2 = [x[i] + dt / 2 * k1[i] for i in range(10)]
            k2, tau2 = self.eom(t + dt / 2, x2, act)
            x3 = [x[i] + dt / 2 * k2[i] for i in range(10)]
            k3, tau3 = self.eom(t + dt / 2, x3, act)
            x4 = [x[i] + dt * k3[i] for i in range(10)]
            k4, tau4 = self.eom(t + dt, x4, act)
            xn = [x[i] + dt / 6 * (k1[i] + 2 * k2[i] + 2 * k3[i] + k4[i]) for i in range(10)]
            # the forces' work, integrated with the same RK4 stages (the energy check of docs 6)
            work += dt / 6 * (self.power(x, tau) + 2 * self.power(x2, tau2) + 2 * self.power(x3, tau3) + self.power(x4, tau4))
            # the angular momentum's source about the base point, same stages (checks.py 7)
            ang += dt / 6 * (self.ang_rate(x, tau) + 2 * self.ang_rate(x2, tau2) + 2 * self.ang_rate(x3, tau3) + self.ang_rate(x4, tau4))
            if out["contact"] is None and x[4] < A_c <= xn[4]:
                f = (A_c - x[4]) / (xn[4] - x[4])
                xc = [x[i] + f * (xn[i] - x[i]) for i in range(10)]
                out["contact"] = (t + f * dt, xc)
                if stop_at_contact:
                    t += dt; x = xn
                    break
            if self.record:
                out["t"].append(t); out["x"].append(x); out["tau"].append(tau); out["ke"].append(self.ke(x))
                out["cap"].append(cap_k1); out["legs_raw"].append(raw_k1); out["stop"].append(stop_k1)
            t += dt; x = xn
        out["work"] = work
        out["ang"] = ang
        out["x_end"] = x; out["t_end"] = t
        return out

    def power(self, x, tau):
        qd = x[5:]
        return (tau["rear"] - tau["lead"]) * qd[0] + tau["legs"] * qd[1] + tau["core"] * (qd[2] - qd[1]) + \
            tau["shoulder"] * (qd[3] - qd[2]) + tau["wrist"] * (qd[4] - qd[3])

    def vel_pts(self, x):
        """each point mass's position relative to the base point (0, s) and its velocity."""
        q, qd = x[:5], x[5:]
        res = []
        for m, terms in self.pts:
            px = py = 0.0; vx, vy = 0.0, qd[0]
            for k, r, off in terms:
                a = q[k] + off
                px += r * math.cos(a); py += r * math.sin(a)
                vx += -r * math.sin(a) * qd[k]; vy += r * math.cos(a) * qd[k]
            res.append((m, px, py, vx, vy))
        return res

    def ang_mom(self, x):
        """vertical angular momentum about the moving base point A = (0, s)."""
        b, qd = self.b, x[5:]
        L = b.I1 * qd[1] + b.I2 * qd[2] + b.I3 * qd[3] + b.Ib * qd[4]
        for m, px, py, vx, vy in self.vel_pts(x):
            L += m * (px * vy - py * vx)
        return L

    def ang_rate(self, x, tau):
        """dL_A/dt from the external loads alone: the ground's yaw couple on the pelvis, plus -v_A x P
        (the base point moves at (0, s')); the ground's horizontal forces act at A. Internal drives and
        stops cancel in pairs, so a wrong internal term shows up here."""
        Px = sum(m * vx for m, px, py, vx, vy in self.vel_pts(x))
        return tau["legs"] + x[5] * Px

    def ke(self, x):
        q, qd = x[:5], x[5:]
        b = self.b
        e = 0.5 * (b.I1 * qd[1] ** 2 + b.I2 * qd[2] ** 2 + b.I3 * qd[3] ** 2 + b.Ib * qd[4] ** 2)
        for m, terms in self.pts:
            vx, vy = 0.0, qd[0]
            for k, r, off in terms:
                a = q[k] + off
                vx += -r * math.sin(a) * qd[k]; vy += r * math.cos(a) * qd[k]
            e += 0.5 * m * (vx * vx + vy * vy)
        return e

    def sweet_spot_speed(self, x):
        b = self.b
        q, qd = x[:5], x[5:]
        vx, vy = 0.0, qd[0]
        for k, r, off in ((2, b.r_sh, math.pi / 2), (3, b.L_arm, 0.0), (4, b.ss, 0.0)):
            a = q[k] + off
            vx += -r * math.sin(a) * qd[k]; vy += r * math.cos(a) * qd[k]
        return math.hypot(vx, vy)

    def hands(self, x):
        b = self.b; q = x[:5]
        a = q[2] + math.pi / 2
        return (b.r_sh * math.cos(a) + b.L_arm * math.cos(q[3]), q[0] + b.r_sh * math.sin(a) + b.L_arm * math.sin(q[3]))

    def sweet_spot(self, x):
        h = self.hands(x); q = x[:5]
        return (h[0] + self.b.ss * math.cos(q[4]), h[1] + self.b.ss * math.sin(q[4]))
