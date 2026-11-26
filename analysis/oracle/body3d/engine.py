"""A constrained rigid-body engine for the three-dimensional body (docs/body-motion.md 4.1, 5.1, 8.1).

Stdlib only. The checking model of PLAN D31 phase 1; nothing here is sim code.

Formulation: MAXIMAL coordinates. Every segment is a free rigid body (position of its centre of mass,
unit quaternion, linear velocity, angular velocity in the world frame); every joint is a set of
constraint rows built from two primitives:

  point   a point fixed in body a coincides with a point fixed in body b        (3 rows)
  dot     an axis fixed in body a stays perpendicular to an axis fixed in b     (1 row)

  ball = point; universal (two axes) = point + 1 dot; hinge = point + 2 dots; weld = point + 3 dots.

Each derivative evaluation solves  (J M^-1 J^T) lambda = gamma - J M^-1 F  by Cholesky in a fixed
order and returns  u' = M^-1 (F + J^T lambda). The two hands on the bat are an ordinary pair of
joints: a closed loop needs nothing special in this formulation, which is why it was chosen over
minimal coordinates (docs/body-motion.md 8.1 says more).

- Constraint drift: Baumgarte's stabilisation, C'' + 2 a C' + b^2 C = 0 (BAUM_A, BAUM_B). `residual`
  reports what is left; `project` assembles a starting posture.
- Contact: points on the feet against the ground plane z = 0. Normal force a one-sided spring and
  damper (never negative). Friction a stick spring to an anchor, capped by the cone mu N; the anchor
  is dragged when the cap is reached. Anchors move once per step and are held through the stages.
- Joint limits: one-sided spring-damper stops on an angle; a stop can push back, never pull.
- Drives: tau = T a(t) f_v(w / W) about an axis fixed in the parent (5.1); activation is a state.
- Fixed-step classical RK4; quaternions renormalised after each step.

The work of every drive, stop, hold and contact is integrated as a state beside the motion, so the
kinetic energy can be checked against it (test_body3d.py).
"""
import math
from operator import mul

# ---- named constants ---------------------------------------------------------------------------
G = 9.81
BAUM_A = 50.0             # [E] 1/s, Baumgarte velocity gain (range 10-200; drift is ~1e-9 m either way)
BAUM_B = 50.0             # [E] 1/s, Baumgarte position gain
TA_UP, TA_DN = 0.015, 0.050   # [E] activation time constants, s (OpenSim Thelen2003 defaults, body-motion 5.1)
K_FV = 0.25               # [E] Hill curvature (body-motion 5.1)
ECC = 1.5                 # [E] eccentric plateau
ECC_SLOPE = 4.0           # [E]
CONTACT_D0 = 1.0e-3       # [E] m, depth over which the contact's damping fades in (keeps the force continuous)
NW = 4                    # work integrals carried in the state: drives, stops, holds, contacts
FLOAT_EPS = 1e-9          # a floating drive axis (actuator's axis_b) is taken as absent where its two directions are this near parallel
STOP_SINGULAR = 0.0872      # sin 5 deg: a stop whose direction comes this near its own axis has no usable angle; the run fails
MUT = set()               # mutations, for the tests that show a check can fail (never set in real runs)


def fv(w):
    """Hill force-velocity factor; w = shortening speed / top speed (negative = lengthening)."""
    if w >= 1.0:
        return 0.0
    if w >= 0.0:
        return (1.0 - w) / (1.0 + w / K_FV)
    return ECC - (ECC - 1.0) / (1.0 - ECC_SLOPE * w)


def fv_tab(tab, w, ecc=None):
    """a measured strength-at-speed curve: tab = ((speed rad/s, fraction), ...) from (0, 1) down to (top speed, 0),
    straight lines between; w = shortening speed in rad/s. A muscle being stretched (w < 0) rises toward ECC
    as in fv, on the curve's own top speed."""
    ecc = ECC if ecc is None else ecc           # the stretched muscle's plateau: ECC, or the curve's own (4.5: the arms' 1.4)
    top = tab[-1][0]
    if w >= top:
        return 0.0
    if w >= 0.0:
        for (wa, fa), (wb, fb) in zip(tab, tab[1:]):
            if w <= wb:
                return fa + (fb - fa) * (w - wa) / (wb - wa)
        return 0.0
    return ecc - (ecc - 1.0) / (1.0 - ECC_SLOPE * (w / top))


def fv_tab_inv(tab, f, ecc=None):
    """the speed (rad/s) at which the curve gives the fraction f, 0 <= f < its plateau."""
    ecc = ECC if ecc is None else ecc
    top = tab[-1][0]
    if f <= 0.0:
        return top
    if f <= 1.0:
        for (wa, fa), (wb, fb) in zip(tab, tab[1:]):
            if f >= fb:
                return wa if fa == fb else wa + (wb - wa) * (fa - f) / (fa - fb)
        return top
    return top * (1.0 - (ecc - 1.0) / (ecc - f)) / ECC_SLOPE


SEC_ITERS = 40            # bisections in the series spring's implicit step (fixed, for replay)
SEC_MARGIN = 1e-6         # the spring is never asked for more than the stretched muscle's plateau less this
MUT_A_MIN = 1e-3          # activation floor inside the spring's step (a switched-off muscle lets the spring go)


# ---- small vector and quaternion algebra (tuples) ----------------------------------------------
def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def dot(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def add(a, b):
    return (a[0] + b[0], a[1] + b[1], a[2] + b[2])


def sub(a, b):
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def scale(a, k):
    return (a[0] * k, a[1] * k, a[2] * k)


def norm(a):
    return math.sqrt(dot(a, a))


def unit(a):
    n = norm(a)
    return (a[0] / n, a[1] / n, a[2] / n)


def qmul(p, q):
    return (p[0] * q[0] - p[1] * q[1] - p[2] * q[2] - p[3] * q[3],
            p[0] * q[1] + p[1] * q[0] + p[2] * q[3] - p[3] * q[2],
            p[0] * q[2] - p[1] * q[3] + p[2] * q[0] + p[3] * q[1],
            p[0] * q[3] + p[1] * q[2] - p[2] * q[1] + p[3] * q[0])


def qconj(q):
    return (q[0], -q[1], -q[2], -q[3])


def qaxis(axis, ang):
    a = unit(axis)
    s = math.sin(0.5 * ang)
    return (math.cos(0.5 * ang), a[0] * s, a[1] * s, a[2] * s)


def qR(q):
    """rotation matrix (row-major 9-tuple) of a unit quaternion (w, x, y, z): world = R local."""
    w, x, y, z = q
    return (1 - 2 * (y * y + z * z), 2 * (x * y - w * z), 2 * (x * z + w * y),
            2 * (x * y + w * z), 1 - 2 * (x * x + z * z), 2 * (y * z - w * x),
            2 * (x * z - w * y), 2 * (y * z + w * x), 1 - 2 * (x * x + y * y))


def mv(R, v):
    return (R[0] * v[0] + R[1] * v[1] + R[2] * v[2],
            R[3] * v[0] + R[4] * v[1] + R[5] * v[2],
            R[6] * v[0] + R[7] * v[1] + R[8] * v[2])


def mtv(R, v):
    return (R[0] * v[0] + R[3] * v[1] + R[6] * v[2],
            R[1] * v[0] + R[4] * v[1] + R[7] * v[2],
            R[2] * v[0] + R[5] * v[1] + R[8] * v[2])


def R_to_q(R):
    """unit quaternion of a rotation matrix (row-major 9-tuple)."""
    tr = R[0] + R[4] + R[8]
    if tr > 0:
        s = math.sqrt(tr + 1.0) * 2
        q = (0.25 * s, (R[7] - R[5]) / s, (R[2] - R[6]) / s, (R[3] - R[1]) / s)
    elif R[0] > R[4] and R[0] > R[8]:
        s = math.sqrt(1.0 + R[0] - R[4] - R[8]) * 2
        q = ((R[7] - R[5]) / s, 0.25 * s, (R[1] + R[3]) / s, (R[2] + R[6]) / s)
    elif R[4] > R[8]:
        s = math.sqrt(1.0 + R[4] - R[0] - R[8]) * 2
        q = ((R[2] - R[6]) / s, (R[1] + R[3]) / s, 0.25 * s, (R[5] + R[7]) / s)
    else:
        s = math.sqrt(1.0 + R[8] - R[0] - R[4]) * 2
        q = ((R[3] - R[1]) / s, (R[2] + R[6]) / s, (R[5] + R[7]) / s, 0.25 * s)
    n = math.sqrt(sum(c * c for c in q))
    return tuple(c / n for c in q)


def axes_to_q(ex, ey, ez):
    """quaternion of the frame whose axes, in world coordinates, are ex, ey, ez (orthonormal, right-handed)."""
    return R_to_q((ex[0], ey[0], ez[0], ex[1], ey[1], ez[1], ex[2], ey[2], ez[2]))


IDENT = (1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)
ZERO = (0.0, 0.0, 0.0)
E3 = ((1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0))


class SolveError(Exception):
    """The constraint matrix lost rank (a singular posture or redundant joints). Never papered over."""


def chol_solve(A, b):
    """Solve A x = b for symmetric positive-definite A given by its lower triangle (rows of a full matrix).
    Fixed order, no pivoting; raises SolveError on a pivot that is not safely positive."""
    n = len(b)
    L = [None] * n
    for i in range(n):
        Ai = A[i]
        Li = [0.0] * (i + 1)
        for j in range(i):
            Lj = L[j]
            Li[j] = (Ai[j] - sum(map(mul, Li[:j], Lj[:j]))) / Lj[j]
        d = Ai[i] - sum(map(mul, Li[:i], Li[:i]))
        if not d > 1e-12 * abs(Ai[i]):
            raise SolveError("constraint matrix not positive definite at row %d (pivot %.3e of %.3e)" % (i, d, Ai[i]))
        Li[i] = math.sqrt(d)
        L[i] = Li
    y = [0.0] * n
    for i in range(n):
        Li = L[i]
        y[i] = (b[i] - sum(map(mul, Li[:i], y[:i]))) / Li[i]
    x = [0.0] * n
    for i in range(n - 1, -1, -1):
        s = y[i]
        for k in range(i + 1, n):
            s -= L[k][i] * x[k]
        x[i] = s / L[i][i]
    return x


class System:
    """Bodies, joints, drives, stops, holds and contacts. Body index -1 is the world."""

    def __init__(self, g=(0.0, 0.0, -G)):
        self.g = g
        self.names, self.m, self.I = [], [], []
        self.pts, self.dots = [], []        # constraint primitives
        self.acts, self.stops, self.holds, self.contacts = [], [], [], []
        self.tones = []
        self.fvtabs = []                    # measured strength-at-speed curves (fv_tab), by index
        self.fvecc = []                     # each curve's plateau for a stretched muscle (ECC where not given)
        self.ext = []                       # external loads (body, point in the body, force, moment; world axes): inverse dynamics only
        self.baum_a, self.baum_b = BAUM_A, BAUM_B
        self.diag = None                    # set to {} to have deriv() record forces of its last call

    # ---- building ----
    def body(self, name, m, I):
        """a rigid body; I = principal moments about its centre of mass along its own x, y, z."""
        self.names.append(name); self.m.append(float(m)); self.I.append(tuple(float(c) for c in I))
        return len(self.m) - 1

    @property
    def nb(self):
        return len(self.m)

    def point(self, a, b, ra, rb):
        self.pts.append((a, b, tuple(ra), tuple(rb)))

    def dotc(self, a, b, ua, wb):
        self.dots.append((a, b, unit(ua), unit(wb)))

    ball = point

    def universal(self, a, b, ra, rb, axis_a, axis_b):
        """two axes: one fixed in a, one fixed in b, held perpendicular."""
        self.point(a, b, ra, rb)
        self.dotc(a, b, axis_a, axis_b)

    def hinge(self, a, b, ra, rb, axis_a, axis_b):
        """one axis, given in each body's own frame."""
        self.point(a, b, ra, rb)
        n = unit(axis_b)
        t = (1.0, 0.0, 0.0) if abs(n[0]) < 0.9 else (0.0, 1.0, 0.0)
        e1 = unit(cross(n, t)); e2 = cross(n, e1)
        self.dotc(a, b, axis_a, e1)
        self.dotc(a, b, axis_a, e2)

    def weld(self, a, b, ra, rb, qa, qb):
        """no relative motion; qa, qb are the bodies' orientations in the posture being welded."""
        self.point(a, b, ra, rb)
        Ra = IDENT if a < 0 else qR(qa)
        Rb = IDENT if b < 0 else qR(qb)
        for i in range(3):
            self.dotc(a, b, E3[i], mtv(Rb, mv(Ra, E3[(i + 1) % 3])))

    def actuator(self, name, a, b, axis_a, T, W, t_on, t_rise, t_off, a0=0.0, tab=-1, ksec=0.0, axis_b=None):
        """a drive between a and b about an axis fixed in a (5.1): on b the torque T act f_v about +axis,
        on a its reaction. Its shortening speed is b's angular velocity relative to a about the axis.
        a0: the excitation it holds before its onset (a muscle tensed before it fires; 4.5).
        tab: index of its measured strength-at-speed curve in fvtabs (-1: the first law with top speed W).
        ksec > 0: the muscle acts through a spring of that stiffness in series (tendon stretch, 4.5): the joint
        gets ksec x the spring's stretch, and the stretch follows the muscle (sec_update).
        axis_b (a direction fixed in b): the drive acts about the FLOATING axis square to both, the unit vector
        along axis_a x axis_b, which turns axis_b away from axis_a (4.5, session 7: raising and lowering the arm, the axis square to the trunk's long axis and the
        upper arm). Where the two are parallel there is no such axis and the drive gives nothing."""
        self.acts.append([name, a, b, unit(axis_a), float(T), float(W), float(t_on), float(t_rise), float(t_off),
                          float(a0), int(tab), float(ksec), ZERO if axis_b is None else unit(axis_b)])
        return len(self.acts) - 1

    @staticmethod
    def float_axis(na, nb):
        """the floating axis of two directions (world): the unit vector along na x nb, or zero where they are parallel."""
        c = cross(nb, na) if "float_swap" in MUT else cross(na, nb)
        l = math.sqrt(dot(c, c))
        return scale(c, 1.0 / l) if l > FLOAT_EPS else ZERO

    def stop(self, name, a, b, axis_a, e_a, f_b, lo, hi, k, c, kind=0):
        """a one-sided limit on an angle between a and b: a spring k and a damper c once the angle is past lo or hi.
        kind 0: the angle about axis_a (fixed in a) from e_a (in a) to f_b (in b), f_b projected onto the plane
        square to the axis. kind 1: the angle of f_b (in b) out of the plane square to axis_a, toward the axis
        (asin of their dot product; e_a is not used).
        The stop's torque acts along the read angle's own gradient g, the vector with d(angle)/dt = w_rel . g, and its
        work is booked with the same g, so a stop can never give back more than it took in (body-motion 8.1; until
        session 8's review the torque acted about axis_a, which is the gradient only on a hinge, and on a ball joint
        did work nothing stored paid for). kind 0: g = n - (n.f) f_p / |f_p|^2, with f_p the projection. kind 1:
        g = (f x n) / |f x n|. Each is undefined where f lies along the axis (kind 0: the angle itself is; kind 1:
        the angle is 90 deg); within STOP_SINGULAR of that the run fails loudly, not silently."""
        self.stops.append((name, a, b, unit(axis_a), unit(e_a), unit(f_b), lo, hi, k, c, int(kind)))

    @staticmethod
    def stop_angle(n, e, f, kind):
        """(angle, gradient g) of a stop from its axis n and directions e, f in world axes; raises SolveError where the
        angle's gradient is undefined."""
        nf = dot(n, f)
        if kind == 1:
            gv = cross(f, n)
            l = math.sqrt(dot(gv, gv))
            if l < STOP_SINGULAR:
                raise SolveError("a stop's direction lies along its axis: its angle has no gradient there")
            return math.asin(max(-1.0, min(1.0, nf))), scale(gv, 1.0 / l)
        th = math.atan2(dot(n, cross(e, f)), dot(e, f))
        if "stop_fixed_axis" in MUT:                 # the law before session 8's review: the torque about the fixed axis
            return th, n
        fp = sub(f, scale(n, nf))
        l2 = dot(fp, fp)
        if l2 < STOP_SINGULAR * STOP_SINGULAR:
            raise SolveError("a stop's direction lies along its axis: the angle it reads is undefined there")
        return th, sub(n, scale(fp, nf / l2))

    def hold(self, name, a, b, qref, K, C, cap):
        """intrinsic stiffness and damping of a joint about a reference relative orientation qref (b in a's
        frame): K and C per axis of a's frame, the torque's size capped. Bounded; never a tracking device
        (body-motion 5.3)."""
        self.holds.append((name, a, b, tuple(qref), tuple(K), tuple(C), cap))

    def tone(self, name, a, b, tau_a):
        """a constant torque on b (its reaction on a), fixed in a's frame: the muscles' level at the start.
        Its work is booked with the holds'."""
        self.tones.append((name, a, b, tuple(tau_a)))

    def contact(self, name, b, p, kn, cn, kt, ct, mu):
        self.contacts.append((name, b, tuple(p), kn, cn, kt, ct, mu))

    # ---- state ----
    @property
    def n_state(self):
        return 13 * self.nb + len(self.acts) + NW

    def new_state(self):
        s = [0.0] * self.n_state
        for i in range(self.nb):
            s[13 * i + 3] = 1.0
        return s

    def new_aux(self):
        """what is advanced once per step and held through its stages: the contacts' friction anchors, then
        each drive's series-spring stretch (rad)."""
        return [None] * len(self.contacts) + [0.0] * len(self.acts)

    @staticmethod
    def set_body(s, i, x, q, v=ZERO, w=ZERO):
        o = 13 * i
        s[o:o + 13] = list(x) + list(q) + list(v) + list(w)

    @staticmethod
    def pose(s, i):
        o = 13 * i
        return tuple(s[o:o + 3]), tuple(s[o + 3:o + 7]), tuple(s[o + 7:o + 10]), tuple(s[o + 10:o + 13])

    def excitation(self, j, t):
        a = self.acts[j]
        t_on, rise, t_off, a0 = a[6], a[7], a[8], a[9]
        if t >= t_off:
            return 0.0
        if t < t_on:
            return a0
        if rise > 0.0 and t < t_on + rise:
            return a0 + (1.0 - a0) * (t - t_on) / rise
        return 1.0

    def tab_ecc(self, tab):
        return self.fvecc[tab] if tab < len(self.fvecc) else ECC

    def act_fv(self, j, w):
        """drive j's strength fraction at shortening speed w (rad/s)."""
        a = self.acts[j]
        return fv(w / a[5]) if a[10] < 0 else fv_tab(self.fvtabs[a[10]], w, self.tab_ecc(a[10]))

    def act_speed(self, j, s):
        """drive j's shortening speed in state s: its child's angular velocity relative to its parent about the axis."""
        name, a, b, ax = self.acts[j][:4]
        n = mv(qR(tuple(s[13 * a + 3:13 * a + 7])), ax) if a >= 0 else ax
        axb = self.acts[j][12]
        if axb != ZERO:
            n = self.float_axis(n, mv(qR(tuple(s[13 * b + 3:13 * b + 7])), axb))
        wa = tuple(s[13 * a + 10:13 * a + 13]) if a >= 0 else ZERO
        return dot(sub(tuple(s[13 * b + 10:13 * b + 13]), wa), n)

    def sec_start(self, s, aux):
        """each series spring's stretch for a muscle in steady pull at the state's speed and activation."""
        nc, oa = len(self.contacts), 13 * self.nb
        while len(aux) < nc + len(self.acts):
            aux.append(0.0)
        for j, a in enumerate(self.acts):
            if a[11] > 0.0:
                aux[nc + j] = a[4] * s[oa + j] * self.act_fv(j, self.act_speed(j, s)) / a[11]

    def sec_update(self, s, aux, h):
        """advance each series spring by one implicit (backward Euler) step of
            stretch' = (muscle's shortening speed) - (joint's shortening speed),
        where the muscle's speed is the one at which its strength-at-speed curve gives the spring's torque:
        K stretch = T act f(speed). Solved by SEC_ITERS bisections, so any stiffness is stable."""
        nc, oa = len(self.contacts), 13 * self.nb
        while len(aux) < nc + len(self.acts):
            aux.append(0.0)
        for j, a in enumerate(self.acts):
            K = a[11]
            if K <= 0.0:
                continue
            tab = self.fvtabs[a[10]]
            ecc = self.tab_ecc(a[10])
            cap = a[4] * max(s[oa + j], MUT_A_MIN)
            wj = self.act_speed(j, s)
            if "sec_sign" in MUT:
                wj = -wj
            old = aux[nc + j]
            g = lambda th: th - old - h * (fv_tab_inv(tab, K * th / cap, ecc) - wj)
            hi = (ecc - SEC_MARGIN) * cap / K
            if cap <= 0.0 or g(0.0) >= 0.0:
                aux[nc + j] = 0.0
            elif g(hi) <= 0.0:
                aux[nc + j] = hi
            else:
                lo = 0.0
                for _ in range(SEC_ITERS):
                    mid = 0.5 * (lo + hi)
                    if g(mid) > 0.0:
                        hi = mid
                    else:
                        lo = mid
                aux[nc + j] = 0.5 * (lo + hi)

    # ---- constraint rows ----
    def _kin(self, s):
        X, R, V, Wv = [], [], [], []
        for i in range(self.nb):
            o = 13 * i
            X.append((s[o], s[o + 1], s[o + 2]))
            R.append(qR((s[o + 3], s[o + 4], s[o + 5], s[o + 6])))
            V.append((s[o + 7], s[o + 8], s[o + 9]))
            Wv.append((s[o + 10], s[o + 11], s[o + 12]))
        return X, R, V, Wv

    def _rows(self, X, R, V, Wv):
        """constraint rows: (a, Ja6, b, Jb6, C, Cdot, gamma) with J u = Cdot and J u' = gamma."""
        rows = []
        gsign = -1.0 if "gamma_sign" in MUT else 1.0
        for a, b, ra, rb in self.pts:
            if a >= 0:
                sa = mv(R[a], ra); wa = Wv[a]; pa = add(X[a], sa)
                va = add(V[a], cross(wa, sa)); ca = cross(wa, cross(wa, sa))
            else:
                sa = ra; pa = ra; va = ZERO; ca = ZERO
            if b >= 0:
                sb = mv(R[b], rb); wb = Wv[b]; pb = add(X[b], sb)
                vb = add(V[b], cross(wb, sb)); cb = cross(wb, cross(wb, sb))
            else:
                sb = rb; pb = rb; vb = ZERO; cb = ZERO
            for k in range(3):
                e = E3[k]
                Ja = (e + cross(sa, e)) if a >= 0 else None
                Jb = (scale(e, -1.0) + cross(e, sb)) if b >= 0 else None
                rows.append((a, Ja, b, Jb, pa[k] - pb[k], va[k] - vb[k], -gsign * (ca[k] - cb[k])))
        for a, b, ua, wb_ in self.dots:
            u = mv(R[a], ua) if a >= 0 else ua
            w = mv(R[b], wb_) if b >= 0 else wb_
            wa = Wv[a] if a >= 0 else ZERO
            wb = Wv[b] if b >= 0 else ZERO
            n = cross(u, w)
            wr = sub(wa, wb)
            nd = add(cross(cross(wa, u), w), cross(u, cross(wb, w)))
            Ja = (ZERO + n) if a >= 0 else None
            Jb = (ZERO + scale(n, -1.0)) if b >= 0 else None
            rows.append((a, Ja, b, Jb, dot(u, w), dot(wr, n), -dot(wr, nd)))
        return rows

    @staticmethod
    def _winv(R, I, lin, ang, J):
        """M^-1 J^T for one body: (J_lin * lin, R diag(ang / I) R^T J_ang)."""
        l = mtv(R, J[3:6])
        y = mv(R, (l[0] * ang / I[0], l[1] * ang / I[1], l[2] * ang / I[2]))
        return (J[0] * lin, J[1] * lin, J[2] * lin, y[0], y[1], y[2])

    def _assemble(self, rows, R, lin, ang):
        """A = J W J^T (lower triangle) and, per body, the rows touching it with W J^T.
        lin[i], ang[i]: the body's inverse mass and a factor on its inverse inertia."""
        n = len(rows)
        per = [[] for _ in range(self.nb)]
        for k, r in enumerate(rows):
            a, Ja, b, Jb = r[0], r[1], r[2], r[3]
            if a >= 0:
                per[a].append((k, Ja, self._winv(R[a], self.I[a], lin[a], ang[a], Ja)))
            if b >= 0:
                per[b].append((k, Jb, self._winv(R[b], self.I[b], lin[b], ang[b], Jb)))
        A = [[0.0] * (k + 1) for k in range(n)]
        for lst in per:
            for p, (k, Jk, Yk) in enumerate(lst):
                Ak = A[k]
                for q in range(p + 1):
                    l, Jl, Yl = lst[q]
                    Ak[l] += (Jl[0] * Yk[0] + Jl[1] * Yk[1] + Jl[2] * Yk[2]
                              + Jl[3] * Yk[3] + Jl[4] * Yk[4] + Jl[5] * Yk[5])
        return A, per

    # ---- forces ----
    def _forces(self, t, s, aux, X, R, V, Wv, F, Tq, ds):
        nb = self.nb
        oa = 13 * nb
        ow = oa + len(self.acts)
        diag = self.diag
        p_act = p_stop = p_hold = p_con = 0.0
        nc = len(self.contacts)
        for j, (name, a, b, ax, T, W, t_on, rise, t_off, a0, tab, ksec, axb) in enumerate(self.acts):
            act = s[oa + j]
            u = self.excitation(j, t)
            ds[oa + j] = (u - act) / (TA_UP if u > act else TA_DN)
            n = mv(R[a], ax) if a >= 0 else ax
            if axb != ZERO:
                n = self.float_axis(n, mv(R[b], axb))
            wrel = dot(sub(Wv[b], Wv[a] if a >= 0 else ZERO), n)
            if ksec > 0.0:
                tau = ksec * (aux[nc + j] if len(aux) > nc + j else 0.0)
            elif tab >= 0:
                tau = T * act * fv_tab(self.fvtabs[tab], wrel, self.tab_ecc(tab))
            else:
                tau = T * act * fv(wrel / W)
            tv = scale(n, tau)
            Tq[b] = add(Tq[b], tv)
            if a >= 0 and "no_reaction" not in MUT:
                Tq[a] = sub(Tq[a], tv)
            p_act += tau * wrel
            if diag is not None:
                diag.setdefault("act", {})[name] = (tau, wrel, act)
        for name, a, b, ax, ea, fb, lo, hi, k, c, kind in self.stops:
            n = mv(R[a], ax) if a >= 0 else ax
            e = mv(R[a], ea) if a >= 0 else ea
            f = mv(R[b], fb)
            th, g = self.stop_angle(n, e, f, kind)
            rate = dot(sub(Wv[b], Wv[a] if a >= 0 else ZERO), g)
            tau = 0.0
            if th > hi:
                tau = -max(0.0, k * (th - hi) + c * rate)
            elif th < lo:
                tau = max(0.0, k * (lo - th) - c * rate)
            if tau != 0.0:
                tv = scale(g, tau)
                Tq[b] = add(Tq[b], tv)
                if a >= 0:
                    Tq[a] = sub(Tq[a], tv)
                p_stop += tau * rate
            if diag is not None:
                diag.setdefault("stop", {})[name] = (tau, th)
        for name, a, b, qref, K, C, cap in self.holds:
            o = 13 * b
            qb = (s[o + 3], s[o + 4], s[o + 5], s[o + 6])
            if a >= 0:
                o = 13 * a
                qrel = qmul(qconj((s[o + 3], s[o + 4], s[o + 5], s[o + 6])), qb)
            else:
                qrel = qb
            qe = qmul(qrel, qconj(qref))
            if qe[0] < 0:
                qe = (-qe[0], -qe[1], -qe[2], -qe[3])
            sn = math.sqrt(qe[1] * qe[1] + qe[2] * qe[2] + qe[3] * qe[3])
            if sn > 1e-12:
                ang = 2.0 * math.atan2(sn, qe[0])
                rv = (qe[1] / sn * ang, qe[2] / sn * ang, qe[3] / sn * ang)
            else:
                rv = (2.0 * qe[1], 2.0 * qe[2], 2.0 * qe[3])
            wr = sub(Wv[b], Wv[a] if a >= 0 else ZERO)
            wl = mtv(R[a], wr) if a >= 0 else wr
            tl = (-K[0] * rv[0] - C[0] * wl[0], -K[1] * rv[1] - C[1] * wl[1], -K[2] * rv[2] - C[2] * wl[2])
            mag = norm(tl)
            if mag > cap:
                tl = scale(tl, cap / mag)
            tv = mv(R[a], tl) if a >= 0 else tl
            Tq[b] = add(Tq[b], tv)
            if a >= 0:
                Tq[a] = sub(Tq[a], tv)
            p_hold += dot(tv, wr)
            if diag is not None:
                diag.setdefault("hold", {})[name] = min(mag, cap)
        for name, a, b, tl in self.tones:
            tv = mv(R[a], tl) if a >= 0 else tl
            Tq[b] = add(Tq[b], tv)
            if a >= 0:
                Tq[a] = sub(Tq[a], tv)
            p_hold += dot(tv, sub(Wv[b], Wv[a] if a >= 0 else ZERO))
        for b, p, f, mo in self.ext:
            F[b] = add(F[b], f)
            Tq[b] = add(add(Tq[b], cross(mv(R[b], p), f)), mo)
        for j, (name, b, p, kn, cn, kt, ct, mu) in enumerate(self.contacts):
            sp = mv(R[b], p)
            P = add(X[b], sp)
            d = -P[2]
            f = ZERO
            if d > 0.0:
                vP = add(V[b], cross(Wv[b], sp))
                N = max(0.0, kn * d - cn * min(1.0, d / CONTACT_D0) * vP[2])
                fx = fy = 0.0
                anc = aux[j]
                if anc is not None and N > 0.0:
                    fx = -kt * (P[0] - anc[0]) - ct * vP[0]
                    fy = -kt * (P[1] - anc[1]) - ct * vP[1]
                    ft = math.sqrt(fx * fx + fy * fy)
                    if ft > mu * N:
                        fx *= mu * N / ft; fy *= mu * N / ft
                f = (fx, fy, N)
                F[b] = add(F[b], f)
                Tq[b] = add(Tq[b], cross(sp, f))
                p_con += dot(f, vP)
            if diag is not None:
                diag.setdefault("contact", {})[name] = f
        ds[ow] = p_act; ds[ow + 1] = p_stop; ds[ow + 2] = p_hold; ds[ow + 3] = p_con

    # ---- the equations of motion ----
    def deriv(self, t, s, aux):
        nb = self.nb
        X, R, V, Wv = self._kin(s)
        ds = [0.0] * len(s)
        g = self.g
        F = [(self.m[i] * g[0], self.m[i] * g[1], self.m[i] * g[2]) for i in range(nb)]
        Tq = [ZERO] * nb
        self._forces(t, s, aux, X, R, V, Wv, F, Tq, ds)
        a0 = []
        lin = [1.0 / m for m in self.m]
        ang = [1.0] * nb
        for i in range(nb):
            I = self.I[i]
            w = Wv[i]
            l = mtv(R[i], w)
            h = mv(R[i], (I[0] * l[0], I[1] * l[1], I[2] * l[2]))
            tq = Tq[i] if "no_gyro" in MUT else sub(Tq[i], cross(w, h))
            lt = mtv(R[i], tq)
            al = mv(R[i], (lt[0] / I[0], lt[1] / I[1], lt[2] / I[2]))
            a0.append((F[i][0] * lin[i], F[i][1] * lin[i], F[i][2] * lin[i], al[0], al[1], al[2]))
        rows = self._rows(X, R, V, Wv)
        acc = [list(a) for a in a0]
        if rows:
            A, per = self._assemble(rows, R, lin, ang)
            rhs = []
            a2, b2 = 2.0 * self.baum_a, self.baum_b * self.baum_b
            for (a, Ja, b, Jb, C, Cd, gam) in rows:
                r = gam - a2 * Cd - b2 * C
                if a >= 0:
                    r -= sum(map(mul, Ja, a0[a]))
                if b >= 0:
                    r -= sum(map(mul, Jb, a0[b]))
                rhs.append(r)
            lam = chol_solve(A, rhs)
            for i, lst in enumerate(per):
                ai = acc[i]
                for k, Jk, Yk in lst:
                    lk = lam[k]
                    ai[0] += lk * Yk[0]; ai[1] += lk * Yk[1]; ai[2] += lk * Yk[2]
                    ai[3] += lk * Yk[3]; ai[4] += lk * Yk[4]; ai[5] += lk * Yk[5]
            if self.diag is not None:
                self.diag["lam"] = lam
        for i in range(nb):
            o = 13 * i
            w = Wv[i]
            q = (s[o + 3], s[o + 4], s[o + 5], s[o + 6])
            qd = qmul((0.0, w[0], w[1], w[2]), q)
            ds[o], ds[o + 1], ds[o + 2] = V[i]
            ds[o + 3] = 0.5 * qd[0]; ds[o + 4] = 0.5 * qd[1]; ds[o + 5] = 0.5 * qd[2]; ds[o + 6] = 0.5 * qd[3]
            ds[o + 7:o + 13] = acc[i]
        return ds

    def step(self, t, s, aux, h):
        """one classical RK4 step; returns the new state (aux, the friction anchors, is updated in place)."""
        f = self.deriv
        k1 = f(t, s, aux)
        k2 = f(t + 0.5 * h, [x + 0.5 * h * k for x, k in zip(s, k1)], aux)
        k3 = f(t + 0.5 * h, [x + 0.5 * h * k for x, k in zip(s, k2)], aux)
        k4 = f(t + h, [x + h * k for x, k in zip(s, k3)], aux)
        h6 = h / 6.0
        n = [x + h6 * (a + 2.0 * b + 2.0 * c + d) for x, a, b, c, d in zip(s, k1, k2, k3, k4)]
        for i in range(self.nb):
            o = 13 * i + 3
            qn = math.sqrt(n[o] * n[o] + n[o + 1] * n[o + 1] + n[o + 2] * n[o + 2] + n[o + 3] * n[o + 3])
            n[o] /= qn; n[o + 1] /= qn; n[o + 2] /= qn; n[o + 3] /= qn
        self.anchors(n, aux)
        self.sec_update(n, aux, h)
        return n

    def anchors(self, s, aux):
        """move the friction anchors (once per step): set on touch-down, cleared on lift-off, dragged to the
        cone's edge when the stick spring would exceed mu N."""
        for j, (name, b, p, kn, cn, kt, ct, mu) in enumerate(self.contacts):
            x, q, v, w = self.pose(s, b)
            P = add(x, mv(qR(q), p))
            d = -P[2]
            if d <= 0.0:
                aux[j] = None
            elif aux[j] is None:
                aux[j] = (P[0], P[1])
            else:
                dx, dy = P[0] - aux[j][0], P[1] - aux[j][1]
                r = math.sqrt(dx * dx + dy * dy)
                lim = mu * kn * d / kt
                if r > lim:
                    aux[j] = (P[0] - dx * lim / r, P[1] - dy * lim / r)

    # ---- measures the tests and the probes read ----
    def residual(self, s):
        """largest position-level and velocity-level constraint errors (m or rad-like; per second)."""
        X, R, V, Wv = self._kin(s)
        rows = self._rows(X, R, V, Wv)
        if not rows:
            return 0.0, 0.0
        return max(abs(r[4]) for r in rows), max(abs(r[5]) for r in rows)

    def project(self, s, lin=None, ang=None, tol=1e-12, itmax=60):
        """Assemble a posture: move the bodies as little as possible (weights lin, ang: bigger moves more,
        0 holds the body's position or orientation) until every joint closes. Then make the velocities
        consistent the same way. Gauss-Newton with a fixed order; raises if it does not converge."""
        nb = self.nb
        lin = [1.0 / m for m in self.m] if lin is None else lin
        ang = [1.0] * nb if ang is None else ang
        s = list(s)
        for it in range(itmax):
            X, R, V, Wv = self._kin(s)
            rows = self._rows(X, R, V, Wv)
            err = max(abs(r[4]) for r in rows)
            if err < tol:
                break
            A, per = self._assemble(rows, R, lin, ang)
            mu_ = chol_solve(A, [-r[4] for r in rows])
            for i, lst in enumerate(per):
                d = [0.0] * 6
                for k, Jk, Yk in lst:
                    for c in range(6):
                        d[c] += mu_[k] * Yk[c]
                o = 13 * i
                s[o] += d[0]; s[o + 1] += d[1]; s[o + 2] += d[2]
                th = math.sqrt(d[3] * d[3] + d[4] * d[4] + d[5] * d[5])
                if th > 0.0:
                    q = qmul(qaxis((d[3], d[4], d[5]), th), (s[o + 3], s[o + 4], s[o + 5], s[o + 6]))
                    qn = math.sqrt(sum(c * c for c in q))
                    s[o + 3:o + 7] = [c / qn for c in q]
        else:
            raise SolveError("posture did not assemble: constraint error %.3e" % err)
        X, R, V, Wv = self._kin(s)
        rows = self._rows(X, R, V, Wv)
        A, per = self._assemble(rows, R, lin, ang)
        mu_ = chol_solve(A, [-r[5] for r in rows])
        for i, lst in enumerate(per):
            o = 13 * i + 7
            for k, Jk, Yk in lst:
                for c in range(6):
                    s[o + c] += mu_[k] * Yk[c]
        return s

    def run(self, s, t0, t1, h, aux=None, each=None):
        """integrate from t0 to t1 in steps of h; each(t, s) is called after every step."""
        aux = self.new_aux() if aux is None else aux
        self.anchors(s, aux)
        n = int(round((t1 - t0) / h))
        for i in range(n):
            s = self.step(t0 + i * h, s, aux, h)
            if each is not None:
                each(t0 + (i + 1) * h, s)
        return s
