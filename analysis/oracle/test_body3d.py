"""Checks of the 3D body engine (analysis/oracle/body3d, docs/body-motion.md 8.1) against things the
engine did not produce: closed-form rigid-body motion, a pendulum's exact period, a double pendulum
and a 3D chain derived independently here (Lagrange's equations in joint angles, which the engine
never uses), conservation laws computed from the raw state, and Newton's law for a sliding block.

    python3 -m unittest analysis/oracle/test_body3d.py

The mutation tests switch on a deliberate fault in the engine and assert that the named check fails.
"""
import cmath
import math
import unittest

try:
    from .body3d import engine as E
    from .body3d import body as B
    from .body3d import drive as Dr
except ImportError:
    from body3d import engine as E
    from body3d import body as B
    from body3d import drive as Dr


# ---- measures computed here, from the raw state, not by the engine ------------------------------
def rot(q):
    w, x, y, z = q
    return [[1 - 2 * (y * y + z * z), 2 * (x * y - w * z), 2 * (x * z + w * y)],
            [2 * (x * y + w * z), 1 - 2 * (x * x + z * z), 2 * (y * z - w * x)],
            [2 * (x * z - w * y), 2 * (y * z + w * x), 1 - 2 * (x * x + y * y)]]


def bodies(sys, s):
    for i in range(sys.nb):
        o = 13 * i
        yield sys.m[i], sys.I[i], s[o:o + 3], rot(s[o + 3:o + 7]), s[o + 7:o + 10], s[o + 10:o + 13]


def kinetic(sys, s):
    T = 0.0
    for m, I, x, R, v, w in bodies(sys, s):
        wl = [sum(R[r][c] * w[r] for r in range(3)) for c in range(3)]
        T += 0.5 * m * sum(c * c for c in v) + 0.5 * sum(I[c] * wl[c] ** 2 for c in range(3))
    return T


def potential(sys, s):
    return -sum(m * sum(sys.g[c] * x[c] for c in range(3)) for m, I, x, R, v, w in bodies(sys, s))


def momenta(sys, s):
    """linear momentum and angular momentum about the world origin."""
    P = [0.0] * 3
    L = [0.0] * 3
    for m, I, x, R, v, w in bodies(sys, s):
        wl = [sum(R[r][c] * w[r] for r in range(3)) for c in range(3)]
        h = [sum(R[r][c] * I[c] * wl[c] for c in range(3)) for r in range(3)]
        for c in range(3):
            P[c] += m * v[c]
        L[0] += m * (x[1] * v[2] - x[2] * v[1]) + h[0]
        L[1] += m * (x[2] * v[0] - x[0] * v[2]) + h[1]
        L[2] += m * (x[0] * v[1] - x[1] * v[0]) + h[2]
    return P, L


def work(sys, s):
    return sum(s[-E.NW:])


def agm_period(inertia, mgl, th0):
    """exact period of a pendulum of amplitude th0: 2 pi sqrt(I / mgl) / agm(1, cos(th0 / 2))."""
    a, b = 1.0, math.cos(0.5 * th0)
    for _ in range(40):
        a, b = 0.5 * (a + b), math.sqrt(a * b)
    return 2.0 * math.pi * math.sqrt(inertia / mgl) / a


def rk4(f, y, h, n):
    for _ in range(n):
        k1 = f(y)
        k2 = f([a + 0.5 * h * b for a, b in zip(y, k1)])
        k3 = f([a + 0.5 * h * b for a, b in zip(y, k2)])
        k4 = f([a + h * b for a, b in zip(y, k3)])
        y = [a + h / 6 * (b + 2 * c + 2 * d + e) for a, b, c, d, e in zip(y, k1, k2, k3, k4)]
    return y


# ---- 1. a free rigid body ------------------------------------------------------------------------
def free_body_error(t_end=1.0, h=1e-3):
    """torque-free axisymmetric body (Euler's closed form): in the body's frame the transverse angular
    velocity turns at (C - A) w3 / A; in the world the symmetry axis cones about the fixed angular
    momentum at |L| / A. Returns the largest error against both."""
    A, C = 0.30, 0.10
    sys = E.System(g=(0.0, 0.0, 0.0))
    b = sys.body("top", 2.0, (A, A, C))
    q0 = E.qmul(E.qaxis((1, 0, 0), 0.4), E.qaxis((0, 0, 1), 0.3))
    wl0 = (1.5, -0.7, 6.0)
    s = sys.new_state()
    sys.set_body(s, b, (0.1, 0.2, 0.3), q0, (0.3, -0.2, 0.1), E.mv(E.qR(q0), wl0))
    L0 = E.mv(E.qR(q0), (A * wl0[0], A * wl0[1], C * wl0[2]))
    Lhat = E.unit(L0)
    s = sys.run(s, 0.0, t_end, h)
    R = rot(s[3:7])
    wl = [sum(R[r][c] * s[10 + r] for r in range(3)) for c in range(3)]
    om = (C - A) * wl0[2] / A * t_end
    exp = (wl0[0] * math.cos(om) - wl0[1] * math.sin(om), wl0[0] * math.sin(om) + wl0[1] * math.cos(om), wl0[2])
    err = max(abs(wl[c] - exp[c]) for c in range(3))
    # the symmetry axis about L: azimuth advanced by |L| t / A, cone angle unchanged
    def az(axis):
        e1 = E.unit(E.cross(Lhat, (1.0, 0.0, 0.0))); e2 = E.cross(Lhat, e1)
        return math.atan2(E.dot(axis, e2), E.dot(axis, e1)), E.dot(axis, Lhat)
    a0, c0 = az(E.mv(E.qR(q0), (0, 0, 1)))
    a1, c1 = az((R[0][2], R[1][2], R[2][2]))
    turn = E.norm(L0) / A * t_end
    d = (a1 - a0 - turn + math.pi) % (2 * math.pi) - math.pi
    err = max(err, abs(d), abs(c1 - c0))
    err = max(err, max(abs(s[c] - ((0.1, 0.2, 0.3)[c] + (0.3, -0.2, 0.1)[c] * t_end)) for c in range(3)))
    return err


# ---- 2, 3. pendulums -------------------------------------------------------------------------------
def rod_angle(s, i):
    """angle from straight down of a rod whose own +z points from its far end to its pivot, about world y."""
    R = rot(s[13 * i + 3:13 * i + 7])
    return math.atan2(R[0][2], R[2][2])


def pendulum_error(h=1e-3):
    m, a, Iy = 3.0, 0.4, 0.11
    sys = E.System()
    b = sys.body("rod", m, (0.2, Iy, 0.03))
    sys.hinge(-1, b, (0, 0, 0), (0, 0, a), (0, 1, 0), (0, 1, 0))
    th0 = 2.0
    q = E.qaxis((0, 1, 0), th0)
    s = sys.new_state()
    sys.set_body(s, b, E.scale(E.mv(E.qR(q), (0, 0, a)), -1.0), q)
    T = agm_period(Iy + m * a * a, m * E.G * a, th0)
    n = int(round(T / h))
    hh = T / n
    quarter = sys.run(list(s), 0.0, hh * (n // 4), hh)
    rest = T / 4 - hh * (n // 4)
    quarter = sys.run(quarter, 0.0, rest, rest) if rest > 1e-12 else quarter
    s = sys.run(s, 0.0, T, hh)
    return max(abs(rod_angle(s, b) - th0), abs(rod_angle(quarter, b)))


def double_pendulum(h=1e-3, t_end=1.5):
    """engine angles and the independent Lagrangian angles of a compound double pendulum at t_end."""
    m1, l1, a1, I1 = 2.0, 0.5, 0.21, 0.05
    m2, a2, I2 = 1.2, 0.33, 0.03
    th = (1.1, -0.6); om = (0.5, 2.0)
    sys = E.System()
    b1 = sys.body("r1", m1, (0.07, I1, 0.01))
    b2 = sys.body("r2", m2, (0.04, I2, 0.02))
    sys.hinge(-1, b1, (0, 0, 0), (0, 0, a1), (0, 1, 0), (0, 1, 0))
    sys.hinge(b1, b2, (0, 0, a1 - l1), (0, 0, a2), (0, 1, 0), (0, 1, 0))
    s = sys.new_state()
    q1, q2 = E.qaxis((0, 1, 0), th[0]), E.qaxis((0, 1, 0), th[1])
    x1 = E.scale(E.mv(E.qR(q1), (0, 0, a1)), -1.0)
    elbow = E.add(x1, E.mv(E.qR(q1), (0, 0, a1 - l1)))
    x2 = E.sub(elbow, E.mv(E.qR(q2), (0, 0, a2)))
    w1, w2 = (0, om[0], 0), (0, om[1], 0)
    v1 = E.cross(w1, x1)
    v2 = E.add(E.cross(w1, elbow), E.cross(w2, E.sub(x2, elbow)))
    sys.set_body(s, b1, x1, q1, v1, w1)
    sys.set_body(s, b2, x2, q2, v2, w2)
    s = sys.run(s, 0.0, t_end, h)
    g = E.G
    J1, J2, c = I1 + m1 * a1 * a1 + m2 * l1 * l1, I2 + m2 * a2 * a2, m2 * l1 * a2

    def f(y):
        t1, t2, o1, o2 = y
        d = t1 - t2
        r1 = -c * math.sin(d) * o2 * o2 - (m1 * a1 + m2 * l1) * g * math.sin(t1)
        r2 = c * math.sin(d) * o1 * o1 - m2 * g * a2 * math.sin(t2)
        k = c * math.cos(d)
        det = J1 * J2 - k * k
        return [o1, o2, (r1 * J2 - k * r2) / det, (J1 * r2 - k * r1) / det]
    ref = rk4(f, [th[0], th[1], om[0], om[1]], 1e-4, int(round(t_end / 1e-4)))
    return (rod_angle(s, b1), rod_angle(s, b2)), (ref[0], ref[1]), sys, s


# ---- 4. the Euler-Lagrange identity for a 3D chain: ball, two-axis joint, hinge -------------------
def _rx(a):
    c, s = cmath.cos(a), cmath.sin(a)
    return [[1, 0, 0], [0, c, -s], [0, s, c]]


def _ry(a):
    c, s = cmath.cos(a), cmath.sin(a)
    return [[c, 0, s], [0, 1, 0], [-s, 0, c]]


def _rz(a):
    c, s = cmath.cos(a), cmath.sin(a)
    return [[c, -s, 0], [s, c, 0], [0, 0, 1]]


def _mm(A, Bm):
    return [[sum(A[i][k] * Bm[k][j] for k in range(3)) for j in range(3)] for i in range(3)]


def _mvv(A, v):
    return [sum(A[i][k] * v[k] for k in range(3)) for i in range(3)]


CH_M = (3.0, 2.0, 1.5)
CH_I = ((0.05, 0.08, 0.02), (0.03, 0.04, 0.012), (0.02, 0.015, 0.03))
CH_R1 = (0.05, -0.02, 0.25)                         # ball's anchor in body 1
CH_P1, CH_P2 = (0.02, 0.03, -0.30), (-0.01, 0.02, 0.22)   # two-axis joint: anchors in bodies 1 and 2
CH_P3, CH_P4 = (0.03, -0.01, -0.20), (0.02, 0.04, 0.15)   # hinge: anchors in bodies 2 and 3


def chain_fk(q):
    """the chain's own forward kinematics in joint angles (complex-step capable)."""
    R1 = _mm(_mm(_rz(q[0]), _ry(q[1])), _rx(q[2]))
    x1 = [-c for c in _mvv(R1, CH_R1)]
    R2 = _mm(_mm(R1, _rx(q[3])), _ry(q[4]))
    x2 = [a + b - c for a, b, c in zip(x1, _mvv(R1, CH_P1), _mvv(R2, CH_P2))]
    R3 = _mm(R2, _rz(q[5]))
    x3 = [a + b - c for a, b, c in zip(x2, _mvv(R2, CH_P3), _mvv(R3, CH_P4))]
    return [(x1, R1), (x2, R2), (x3, R3)]


def chain_vel(q, qd):
    """body velocities for joint rates qd, by the complex step (exact to rounding)."""
    hc = 1e-30
    fk = chain_fk([a + 1j * hc * b for a, b in zip(q, qd)])
    out = []
    for x, R in fk:
        Rr = [[R[i][j].real for j in range(3)] for i in range(3)]
        Rd = [[R[i][j].imag / hc for j in range(3)] for i in range(3)]
        S = _mm(Rd, [[Rr[j][i] for j in range(3)] for i in range(3)])     # Rdot R^T = [w]x
        out.append(([c.real for c in x], Rr, [c.imag / hc for c in x], [S[2][1], S[0][2], S[1][0]]))
    return out


def chain_T(q, qd):
    T = 0.0
    for (x, R, v, w), m, I in zip(chain_vel(q, qd), CH_M, CH_I):
        wl = [sum(R[r][c] * w[r] for r in range(3)) for c in range(3)]
        T += 0.5 * m * sum(c * c for c in v) + 0.5 * sum(I[c] * wl[c] ** 2 for c in range(3))
    return T


def chain_V(q):
    return sum(m * E.G * x[2].real for (x, R), m in zip(chain_fk(q), CH_M))


def chain_qdd(q, qd):
    """joint accelerations from Lagrange's equations, M q'' = -(M' q' - dT/dq + dV/dq), by differences."""
    n = 6
    unitv = lambda i: [1.0 if k == i else 0.0 for k in range(n)]

    def Mmat(qq):
        d = [chain_T(qq, unitv(i)) for i in range(n)]
        M = [[0.0] * n for _ in range(n)]
        for i in range(n):
            M[i][i] = 2 * d[i]
            for j in range(i):
                e = unitv(i); e[j] = 1.0
                M[i][j] = M[j][i] = chain_T(qq, e) - d[i] - d[j]
        return M
    mom = lambda qq: [sum(r[j] * qd[j] for j in range(n)) for r in Mmat(qq)]
    eps = 1e-5
    pp = mom([a + eps * b for a, b in zip(q, qd)]); pm = mom([a - eps * b for a, b in zip(q, qd)])
    rhs = []
    for k in range(n):
        qp = list(q); qp[k] += eps
        qm = list(q); qm[k] -= eps
        dT = (chain_T(qp, qd) - chain_T(qm, qd)) / (2 * eps)
        dV = (chain_V(qp) - chain_V(qm)) / (2 * eps)
        rhs.append(-((pp[k] - pm[k]) / (2 * eps) - dT + dV))
    M = Mmat(q)
    for c in range(n):                                # Gaussian elimination with partial pivoting
        p = max(range(c, n), key=lambda r: abs(M[r][c]))
        M[c], M[p] = M[p], M[c]; rhs[c], rhs[p] = rhs[p], rhs[c]
        for r in range(c + 1, n):
            f = M[r][c] / M[c][c]
            rhs[r] -= f * rhs[c]
            for k in range(c, n):
                M[r][k] -= f * M[c][k]
    x = [0.0] * n
    for c in range(n - 1, -1, -1):
        x[c] = (rhs[c] - sum(M[c][k] * x[k] for k in range(c + 1, n))) / M[c][c]
    return x


def chain_identity_error(q, qd):
    """largest difference between the engine's accelerations of the three centres of mass and those
    Lagrange's equations give, relative to the largest acceleration."""
    sys = E.System()
    bs = [sys.body("c%d" % i, CH_M[i], CH_I[i]) for i in range(3)]
    sys.ball(-1, bs[0], (0, 0, 0), CH_R1)
    sys.universal(bs[0], bs[1], CH_P1, CH_P2, (1, 0, 0), (0, 1, 0))
    sys.hinge(bs[1], bs[2], CH_P3, CH_P4, (0, 0, 1), (0, 0, 1))
    s = sys.new_state()
    for b, (x, R, v, w) in zip(bs, chain_vel(q, qd)):
        sys.set_body(s, b, x, E.R_to_q(tuple(R[i][j] for i in range(3) for j in range(3))), v, w)
    ds = sys.deriv(0.0, s, sys.new_aux())
    qdd = chain_qdd(q, qd)
    tau = 2e-4
    at = lambda t: [[c.real for c in x] for x, R in chain_fk([a + t * b + 0.5 * t * t * c for a, b, c in zip(q, qd, qdd)])]
    xp, x0, xm = at(tau), at(0.0), at(-tau)
    err = big = 0.0
    for i in range(3):
        for c in range(3):
            exp = (xp[i][c] - 2 * x0[i][c] + xm[i][c]) / (tau * tau)
            err = max(err, abs(ds[13 * i + 7 + c] - exp)); big = max(big, abs(exp))
    return err / big, sys.residual(s)


CHAIN_STATES = [
    ([0.3, -0.5, 0.8, 0.4, -0.7, 1.1], [1.0, -2.0, 0.5, 3.0, -1.5, 2.5]),
    ([-1.2, 0.4, -0.3, -0.9, 0.5, -2.0], [-0.7, 1.3, 2.2, -2.8, 0.9, -3.1]),
    ([2.0, 0.9, 1.7, 1.2, 1.0, 0.2], [4.0, 0.3, -1.1, 0.6, 2.4, 1.9]),
]


# ---- 5. a closed loop with an exact answer: the parallelogram linkage ------------------------------
def parallelogram(h=1e-3, baum=True):
    """two equal cranks and a coupler: the coupler only translates, so the linkage is a pendulum with
    (2 (Ic + mc a^2) + mk L^2) th'' = -(2 mc a + mk L) g sin th. Returns the angle error after one exact
    period and the loop's drift."""
    L, a, mc, Ic, mk, d, th0 = 0.5, 0.2, 1.5, 0.04, 2.5, 0.7, 1.0
    sys = E.System()
    if not baum:
        sys.baum_a = sys.baum_b = 0.0
    c1 = sys.body("crank1", mc, (0.05, Ic, 0.01))
    c2 = sys.body("crank2", mc, (0.05, Ic, 0.01))
    k = sys.body("coupler", mk, (0.02, 0.09, 0.1))
    sys.hinge(-1, c1, (0, 0, 0), (0, 0, a), (0, 1, 0), (0, 1, 0))
    sys.hinge(c1, k, (0, 0, a - L), (-d / 2, 0, 0), (0, 1, 0), (0, 1, 0))
    sys.universal(k, c2, (d / 2, 0, 0), (0, 0, a - L), (0, 1, 0), (1, 0, 0))
    sys.ball(-1, c2, (d, 0, 0), (0, 0, a))
    q = E.qaxis((0, 1, 0), th0)
    R = E.qR(q)
    s = sys.new_state()
    x1 = E.scale(E.mv(R, (0, 0, a)), -1.0)
    tip = E.mv(R, (0, 0, -L))
    sys.set_body(s, c1, x1, q)
    sys.set_body(s, c2, E.add(x1, (d, 0, 0)), q)
    sys.set_body(s, k, E.add(tip, (d / 2, 0, 0)), (1.0, 0.0, 0.0, 0.0))
    T = agm_period(2 * (Ic + mc * a * a) + mk * L * L, (2 * mc * a + mk * L) * E.G, th0)
    n = int(round(T / h))
    s = sys.run(s, 0.0, T, T / n)
    return abs(rod_angle(s, c1) - th0), sys.residual(s)[0]


# ---- 6. contact: a block on the ground --------------------------------------------------------------
def block(push, mu=0.6, t_end=0.5, h=1e-3):
    """a 10 kg block on four contact points, pushed along x by a constant force at its centre of mass
    height... applied at the centre of mass. Returns its travel, its speed and the smallest normal force seen."""
    m = 10.0
    sys = E.System(g=(push / m, 0.0, -E.G))     # the push as a uniform field: a force at the centre of mass
    b = sys.body("block", m, (0.2, 0.2, 0.3))
    for i, (px, py) in enumerate(((0.2, 0.15), (0.2, -0.15), (-0.2, 0.15), (-0.2, -0.15))):
        sys.contact("p%d" % i, b, (px, py, -0.05), 2e5, 300.0, 2e5, 300.0, mu)
    s = sys.new_state()
    sys.set_body(s, b, (0.0, 0.0, 0.05 - m * E.G / 4 / 2e5), (1.0, 0.0, 0.0, 0.0))
    aux = sys.new_aux()
    sys.anchors(s, aux)
    sys.diag = {}
    low = [1e9]

    def each(t, st):
        sys.deriv(t, st, aux)
        low[0] = min(low[0], min(f[2] for f in sys.diag["contact"].values()))
    s = sys.run(s, 0.0, t_end, h, aux=aux, each=each)
    return s[0], s[7], low[0]


class EngineChecks(unittest.TestCase):
    def test_free_body_closed_form(self):
        """Euler's torque-free motion of a symmetric body: body-frame precession, the cone about L, straight-line centre."""
        self.assertLess(free_body_error(), 2e-9)

    def test_pendulum_exact_period(self):
        """a hinged rod released at 2.0 rad is back there after the exact (elliptic) period and hangs straight at a quarter of it."""
        self.assertLess(pendulum_error(), 5e-9)

    def test_double_pendulum_independent(self):
        """two hinged rods against Lagrange's equations in the two angles, integrated here."""
        (t1, t2), (r1, r2), sys, s = double_pendulum()
        self.assertLess(abs(t1 - r1), 2e-8)
        self.assertLess(abs(t2 - r2), 2e-8)
        self.assertLess(sys.residual(s)[0], 1e-10)

    def test_euler_lagrange_identity_3d(self):
        """ball + two-axis joint + hinge at three moving states: the engine's accelerations equal those of
        Lagrange's equations built from the chain's joint-angle kinematics (differences' own level, 1e-6)."""
        for q, qd in CHAIN_STATES:
            err, (c, cd) = chain_identity_error(q, qd)
            self.assertLess(c, 1e-12); self.assertLess(cd, 1e-11)
            self.assertLess(err, 2e-6)

    def test_closed_loop_parallelogram(self):
        """a closed loop with an exact period; the loop stays shut."""
        err, drift = parallelogram()
        self.assertLess(err, 2e-8)
        self.assertLess(drift, 1e-10)

    def test_loop_drift_without_stabilisation(self):
        """Baumgarte's terms are not what makes the answer right: without them the drift over a period is
        still far below a micrometre, and with them it is smaller."""
        err0, drift0 = parallelogram(baum=False)
        err1, drift1 = parallelogram(baum=True)
        self.assertLess(drift0, 1e-7)
        self.assertLessEqual(drift1, drift0)
        self.assertLess(err0, 1e-6)

    def test_step_size_convergence(self):
        """halving the step cuts the double pendulum's error by about 16 (fourth order)."""
        errs = []
        for h in (4e-3, 2e-3, 1e-3):
            (t1, t2), (r1, r2), _, _ = double_pendulum(h=h)
            errs.append(math.hypot(t1 - r1, t2 - r2))
        self.assertGreater(errs[0] / errs[1], 11.0); self.assertLess(errs[0] / errs[1], 24.0)
        self.assertGreater(errs[1] / errs[2], 11.0); self.assertLess(errs[1] / errs[2], 24.0)

    def test_block_sticks_then_slides(self):
        """Coulomb: under mu m g the block stays; above it, it accelerates at F/m - mu g. The normal
        forces are never negative."""
        m, mu = 10.0, 0.6
        x, v, low = block(0.5 * mu * m * E.G)
        self.assertLess(abs(x), 3e-4); self.assertLess(abs(v), 1e-4); self.assertGreaterEqual(low, 0.0)
        F = 1.5 * mu * m * E.G
        x, v, low = block(F)
        acc = F / m - mu * E.G
        self.assertAlmostEqual(v, acc * 0.5, delta=0.02 * acc * 0.5)
        self.assertGreaterEqual(low, 0.0)

    def test_dropped_block_is_never_pulled(self):
        """a tilted block dropped from 10 cm bounces and settles: no contact force ever points down, and at rest
        the ground carries its weight."""
        sys = E.System()
        b = sys.body("block", 10.0, (0.2, 0.2, 0.3))
        for i, (px, py) in enumerate(((0.2, 0.15), (0.2, -0.15), (-0.2, 0.15), (-0.2, -0.15))):
            sys.contact("p%d" % i, b, (px, py, -0.05), 2e5, 300.0, 2e5, 300.0, 0.6)
        s = sys.new_state()
        sys.set_body(s, b, (0.0, 0.0, 0.10), E.qaxis((1, 0, 0), 0.05))
        aux = sys.new_aux()
        sys.diag = {}
        low = [1e9]

        def each(t, st):
            sys.deriv(t, st, aux)
            low[0] = min(low[0], min(f[2] for f in sys.diag["contact"].values()))
        s = sys.run(s, 0.0, 2.5, 1e-3, aux=aux, each=each)
        self.assertGreaterEqual(low[0], 0.0)
        sys.deriv(2.5, s, aux)
        self.assertAlmostEqual(sum(f[2] for f in sys.diag["contact"].values()), 10.0 * E.G, delta=0.5)

    def test_stop_never_pulls(self):
        """a pendulum swung into a stop: the stop's torque only ever pushes back out of the limit."""
        sys = E.System()
        b = sys.body("rod", 3.0, (0.2, 0.11, 0.03))
        sys.hinge(-1, b, (0, 0, 0), (0, 0, 0.4), (0, 1, 0), (0, 1, 0))
        sys.stop("lim", -1, b, (0, 1, 0), (0, 0, 1), (0, 0, 1), -0.3, 10.0, 2000.0, 20.0)
        q = E.qaxis((0, 1, 0), 1.0)
        s = sys.new_state()
        sys.set_body(s, b, E.scale(E.mv(E.qR(q), (0, 0, 0.4)), -1.0), q)
        aux = sys.new_aux()
        sys.diag = {}
        seen = []

        def each(t, st):
            sys.deriv(t, st, aux)
            seen.append(sys.diag["stop"]["lim"])
        s = sys.run(s, 0.0, 1.5, 1e-3, aux=aux, each=each)
        self.assertTrue(any(tau > 0 for tau, th in seen))
        self.assertTrue(all(tau >= 0.0 for tau, th in seen))
        self.assertTrue(all(tau == 0.0 for tau, th in seen if th >= -0.3))
        self.assertGreater(min(th for tau, th in seen), -0.6)


# ---- 7. the whole body ---------------------------------------------------------------------------
T0 = -0.250
MODEST = {"hip_rot_R": 0.6, "hip_rot_L": 0.6, "hip_ext_R": 0.3, "hip_flex_L": 0.3, "knee_ext_L": 0.1, "knee_ext_R": 0.1,
          "core": 0.8, "shoulder_L": 0.8, "shoulder_R": 0.8, "elbow_ext_L": 0.3, "elbow_ext_R": 0.5, "wrist_L": 1.0, "wrist_R": 1.0}


def driven_body(contacts=True, gravity=True, t_on=T0):
    """the placed body with every channel driven at a share of its bound from t_on (a test load, not a swing)."""
    m = B.Model(contacts=contacts)
    if not gravity:
        m.sys.g = (0.0, 0.0, 0.0)
    st = B.place(m, T0)
    bd = Dr.bounds(m.H, m.M)
    st = Dr.add_drives(m, st, {n: (bd[n] * k, t_on, 0.05, 1.0) for n, k in MODEST.items()})
    Dr.add_damping(m, st)
    return m, st


def body_free_momentum(steps=60):
    """no ground, no gravity, every drive on: change of linear and angular momentum (three axes each),
    with the sizes they are compared against."""
    m, st = driven_body(contacts=False, gravity=False)
    P0, L0 = momenta(m.sys, st)
    s1 = m.sys.run(list(st), T0, T0 + steps * 1e-3, 1e-3)
    P1, L1 = momenta(m.sys, s1)
    spin = sum(abs(s1[13 * i + 10 + c]) for i in range(m.sys.nb) for c in range(3))
    return max(abs(a - b) for a, b in zip(P0, P1)), max(abs(a - b) for a, b in zip(L0, L1)), spin, m.sys.residual(s1)


def body_energy(steps=80, h=1e-3):
    """energy gained minus work booked, the sizes of the four work integrals, the joints' drift."""
    m, st = driven_body()
    e0 = kinetic(m.sys, st) + potential(m.sys, st)
    s1 = m.sys.run(list(st), T0, T0 + steps * 1e-3, h)
    e1 = kinetic(m.sys, s1) + potential(m.sys, s1)
    return (e1 - e0) - work(m.sys, s1), [abs(w) for w in s1[-E.NW:]], m.sys.residual(s1)


NEED = unittest.skipUnless(B.have_curves(), "[4]'s digitised curves are not on this machine (docs/body-motion.md 3)")


class BodyChecks(unittest.TestCase):
    def test_table_and_counts(self):
        """de Leva's mass shares sum to the whole body; 13 bodies, 49 joint rows, so 29 freedoms (4.1: 35 - 6)."""
        m = B.Model(1.84, 92.0)
        self.assertAlmostEqual(sum(m.sys.m), 92.0 + B.C.BAT_M, delta=92.0 * 2e-4)
        self.assertEqual(m.sys.nb, 13)
        self.assertEqual(3 * len(m.sys.pts) + len(m.sys.dots), 49)

    @NEED
    def test_placed_on_the_measured_state(self):
        """the placed body closes every joint and sits on [4]'s curves at the start (read here from the
        digitised files): rotations, the bat, both elbows, the lead knee."""
        m = B.Model()
        st = B.place(m, T0)
        self.assertLess(m.sys.residual(st)[0], 1e-10)
        o = m.observe(st)
        for k in ("pelvis_rot", "trunk_rot", "bat_elev", "bat_lag", "lead_elbow_flex", "trail_elbow_flex", "lead_knee_flex"):
            self.assertAlmostEqual(o[k], B.curve(k, T0), delta=0.5, msg=k)
        self.assertAlmostEqual((o["bat_az"] - B.curve("bat_az", T0)) % 360.0, 0.0, delta=0.5)
        self.assertAlmostEqual(o["trail_knee_flex"], B.curve_trail_knee(T0), delta=0.5)
        for n, b_, p, *_ in m.sys.contacts:               # the rear foot flat, the lead foot on its ball, heel up
            x, q, v, w = m.sys.pose(st, b_)
            z = E.add(x, E.mv(E.qR(q), p))[2]
            if n.startswith("R_") or "_ball_" in n:
                self.assertLess(abs(z), 1.5e-3, n)
            else:
                self.assertGreater(z, 0.02, n)

    @NEED
    def test_starts_moving_as_measured(self):
        """the placed body's velocities are [4]'s slopes: the rotation rates of pelvis, trunk and bat (read
        here from [4]'s digitised rate curves, which the placement does not use) and the pelvis's travel."""
        m = B.Model()
        st = B.place(m, T0)
        o = m.observe(st)
        for k, tol in (("pelvis_rot_vel", 25.0), ("trunk_rot_vel", 25.0), ("bat_az_vel", 40.0)):
            self.assertAlmostEqual(o[k], B.curve(k, T0), delta=tol, msg=k)
        vy = st[13 * m.b["pelvis"] + 8]
        self.assertAlmostEqual(vy, B.curve_rate("pelvis_x_cm", T0, 0.02) / 100.0, delta=0.12)
        self.assertLess(m.sys.residual(st)[1], 1e-10)

    @NEED
    def test_momentum_three_axes(self):
        """internal drives cannot change the body's linear or angular momentum on any axis. Linear momentum
        holds to rounding. Angular momentum holds to the joints' drift (a joint force acting across a gap
        of under a micrometre is a small couple): under 1e-6 kg m^2/s here, against segment momenta of order 1."""
        dP, dL, spin, (c, cd) = body_free_momentum()
        self.assertGreater(spin, 5.0)                     # the drives did move the segments
        self.assertLess(dP, 1e-12)
        self.assertLess(dL, 1e-6)
        self.assertLess(c, 1e-6)                          # the joints' drift at a 1 ms step, from the moving start

    @NEED
    def test_work_against_energy(self):
        """on the ground, driven: the change of kinetic plus potential energy equals the work of the
        drives, stops, damping and contacts, each integrated beside the motion."""
        gap, parts, (c, cd) = body_energy()
        self.assertGreater(parts[0], 5.0)                 # the drives did real work (16 J)
        self.assertLess(abs(gap), 5e-3)                   # 2 mJ at a 1 ms step
        self.assertLess(c, 1e-6)                          # the joints' drift, 0.3 micrometres

    @NEED
    def test_body_step_size(self):
        """halving the step cuts the whole body's energy error and the joints' drift by about 16 or more
        (measured 2.0 mJ -> 0.07 mJ, 0.33 um -> 0.01 um): the 1 ms step's errors are truncation, not a fault."""
        g1, _, (c1, _) = body_energy(steps=40, h=1e-3)
        g2, _, (c2, _) = body_energy(steps=40, h=5e-4)
        self.assertLess(abs(g2), abs(g1) / 8.0)
        self.assertLess(c2, c1 / 8.0)


class Mutations(unittest.TestCase):
    """each fault is caught by a named check (and the checks pass without it, above)."""

    def tearDown(self):
        E.MUT.clear()

    def test_missing_gyroscopic_term_fails_the_free_body(self):
        E.MUT.add("no_gyro")
        self.assertGreater(free_body_error(), 1e-2)

    def test_wrong_centripetal_sign_fails_the_double_pendulum(self):
        E.MUT.add("gamma_sign")
        (t1, t2), (r1, r2), _, _ = double_pendulum()
        self.assertGreater(abs(t1 - r1) + abs(t2 - r2), 1e-2)

    @NEED
    def test_missing_reaction_fails_momentum(self):
        """a drive that turns the child without its reaction on the parent breaks angular momentum."""
        E.MUT.add("no_reaction")
        dP, dL, spin, _ = body_free_momentum(steps=20)
        self.assertGreater(dL, 1e-2)

    def test_wrong_centripetal_sign_fails_the_identity(self):
        E.MUT.add("gamma_sign")
        err, _ = chain_identity_error(*CHAIN_STATES[0])
        self.assertGreater(err, 1e-2)


if __name__ == "__main__":
    unittest.main()
