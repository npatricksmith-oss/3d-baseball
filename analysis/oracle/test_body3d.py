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


# ---- 6b. the tendon spring in series with a drive (body-motion 4.5) -----------------------------------
def floating_axis_error(t_end=0.3, h=5e-4):
    """a rod on a ball joint at the origin, no gravity, turning about the vertical, with a constant torque about the
    FLOATING axis square to the vertical and to the rod (a drive with an axis_b; body-motion 4.5, session 7). In
    spherical coordinates (th from straight down, ph about the vertical) that torque is the generalised force on
    th alone, so, for a rod with no inertia about its own axis,
        th'' = sin th cos th ph'^2 + T / Ip,      sin^2 th ph' = constant,
    integrated here by RK4. Returns the engine's (th, ph) and these."""
    m, a, It, T = 2.0, 0.3, 0.05, 1.5
    th0, phd0 = 0.9, 3.0
    Ip = It + m * a * a
    sys = E.System(g=(0.0, 0.0, 0.0))
    b = sys.body("rod", m, (It, It, 1e-9))
    sys.ball(-1, b, (0, 0, 0), (0, 0, a))
    sys.actuator("raise", -1, b, (0, 0, 1), T, 1e12, -1.0, 0.0, 1e9, 1.0, -1, 0.0, (0, 0, 1))
    q = E.qaxis((0, 1, 0), th0)                      # own +z from the far end to the pivot: the rod hangs at th0 toward -x (ph = pi)
    s = sys.new_state()
    xc = E.scale(E.mv(E.qR(q), (0, 0, a)), -1.0)
    sys.set_body(s, b, xc, q)
    w = (0.0, 0.0, phd0)
    s[7:10] = list(E.cross(w, xc))
    s[10:13] = list(w)
    s[13] = 1.0                                      # the drive fully on from the start
    s = sys.run(s, 0.0, t_end, h)
    R = rot(s[3:7])
    d = (-R[0][2], -R[1][2], -R[2][2])               # from the pivot to the far end
    th = math.acos(-d[2])
    ph = math.atan2(d[1], d[0])
    hc = math.sin(th0) ** 2 * phd0

    def f(y):
        t_, td, p_ = y
        pd = hc / math.sin(t_) ** 2
        return [td, math.sin(t_) * math.cos(t_) * pd * pd + T / Ip, pd]
    ref = rk4(f, [th0, 0.0, math.pi], 1e-4, int(round(t_end / 1e-4)))
    return (th, ph), (ref[0], ref[2])


SEC_TAB = ((0.0, 1.0), (1.517, 0.75), (3.952, 0.50), (5.24, 0.43), (13.4, 0.0))


def tendon_ledger(h, ecc=None):
    """a hinged rod under gravity, driven through a series spring. The muscle's own work, from its
    strength-at-speed law at its own speed (the joint's plus the spring's rate of stretch), must equal the work
    the rod receives plus the energy the spring holds. Returns (the gap, the muscle's work, the spring's largest
    energy, the largest stretch); the gap is first order in the step."""
    T, K = 150.0, 800.0
    sys = E.System()
    b = sys.body("rod", 4.0, (0.3, 0.5, 0.05))
    sys.hinge(-1, b, (0, 0, 0), (0, 0, 0.3), (0, 1, 0), (0, 1, 0))
    sys.fvtabs = [SEC_TAB]
    if ecc is not None:
        sys.fvecc = [ecc]
    sys.actuator("m", -1, b, (0, 1, 0), T, 20.0, 0.02, 0.05, 0.25, 0.2, 0, K)
    s = sys.new_state()
    q = E.qaxis((0, 1, 0), 0.4)
    sys.set_body(s, b, E.scale(E.mv(E.qR(q), (0, 0, 0.3)), -1.0), q)
    s[13] = 0.2                                      # the drive already on at its starting level
    aux = sys.new_aux()
    sys.sec_start(s, aux)
    th = [aux[0]]
    log = []

    def each(t, st):
        th.append(aux[0])
        log.append((st[13], st[11], (th[-1] - th[-2]) / h))      # activation, the joint's speed about y, the stretch's rate
    s = sys.run(s, 0.0, 0.4, h, aux=aux, each=each)
    w_ce = sum(T * a * E.fv_tab(SEC_TAB, wj + dth, ecc) * (wj + dth) * h for a, wj, dth in log)
    stored = 0.5 * K * (th[-1] ** 2 - th[0] ** 2)
    return w_ce - (s[-E.NW] + stored), w_ce, 0.5 * K * max(th) ** 2, max(th)


def rod_on_a_stop(tilt_deg, kind, k=2000.0, c=0.0, T=0.1, h=1e-4):
    """a 2 kg rod on a ball joint at the world's origin, no gravity, turned into one stop (the reviewer's probe 5).
    kind 0: the stop reads the rod's direction about the vertical, from +x, and ends at 0; the rod starts along +x
    tilted `tilt_deg` below the horizontal, turning into the stop with some spin about its own projection.
    kind 1: the stop reads the rod's angle out of the x-z plane toward +y and ends at 0; the same start.
    Returns (energy at the start, at the end, the largest seen, the stop's booked work + what its spring still holds):
    energy is the kinetic energy plus the spring's stored half k x^2."""
    s = E.System(g=(0.0, 0.0, 0.0))
    I = (0.15, 0.15, 0.002)
    b = s.body("rod", 2.0, I)
    s.ball(-1, b, (0.0, 0.0, 0.0), (0.0, 0.0, 0.3))
    if kind == 0:
        s.stop("st", -1, b, (0, 0, 1), (1, 0, 0), (0, 0, -1), -10.0, 0.0, k, c, 0)
    else:
        s.stop("st", -1, b, (0, 1, 0), (1, 0, 0), (0, 0, -1), -10.0, 0.0, k, c, 1)
    st = s.new_state()
    a = math.radians(tilt_deg)
    d = (math.cos(a), 0.0, -math.sin(a))                 # from the joint to the rod's far end: the body's -z
    ez = (-d[0], -d[1], -d[2]); ey = (0.0, 1.0, 0.0)
    q = E.axes_to_q(E.cross(ey, ez), ey, ez)
    x = E.scale(d, 0.3)
    w = (0.3 * math.cos(a), 0.0, 3.0)                    # into the stop (+z turns the rod toward +y), with spin about its projection
    s.set_body(st, b, x, q, E.cross(w, x), w)
    aux = s.new_aux()
    ow = 13 * s.nb + len(s.acts)

    def energy(state):
        xx, qq, vv, ww = s.pose(state, b)
        R = E.qR(qq)
        wl = E.mtv(R, ww)
        f = E.mv(R, (0, 0, -1))
        th = math.atan2(f[1], f[0]) if kind == 0 else math.asin(max(-1.0, min(1.0, f[1])))
        pe = 0.5 * k * th * th if th > 0 else 0.0
        return 0.5 * 2.0 * E.dot(vv, vv) + 0.5 * sum(I[i] * wl[i] ** 2 for i in range(3)) + pe, pe
    e0 = energy(st)[0]
    emax = e0
    for i in range(int(round(T / h))):
        st = s.step(i * h, st, aux, h)
        if i % 10 == 0:
            emax = max(emax, energy(st)[0])
    e1, pe1 = energy(st)
    return e0, e1, emax, st[ow + 1] + pe1


class EngineChecks(unittest.TestCase):
    def test_speed_curve_and_its_inverse(self):
        """a measured strength-at-speed curve passes through its points, falls with speed, rises for a stretched
        muscle toward ECC, and its inverse undoes it on both sides."""
        for w, f in SEC_TAB:
            self.assertAlmostEqual(E.fv_tab(SEC_TAB, w), f, places=12)
        ws = [-6.0 + 0.37 * k for k in range(52)]
        fs = [E.fv_tab(SEC_TAB, w) for w in ws]
        self.assertTrue(all(a >= b for a, b in zip(fs, fs[1:])))
        self.assertLess(max(fs), E.ECC)
        for w in ws:
            if w < 13.4:
                self.assertAlmostEqual(E.fv_tab_inv(SEC_TAB, E.fv_tab(SEC_TAB, w)), w, places=9)
        # a curve with its own plateau for a stretched muscle (the arms' 1.4, body-motion 4.5)
        fa = [E.fv_tab(SEC_TAB, w, 1.4) for w in ws]
        self.assertLess(max(fa), 1.4)
        self.assertGreater(E.fv_tab(SEC_TAB, -2000.0, 1.4), 1.39)
        for w in ws:
            if w < 13.4:
                self.assertAlmostEqual(E.fv_tab_inv(SEC_TAB, E.fv_tab(SEC_TAB, w, 1.4), 1.4), w, places=9)

    def test_the_arms_curves_and_bounds_are_the_committed_ones(self):
        """body-motion 4.5's arm numbers, typed here from its tables (not read from drive.py): the bounds for
        [4]'s mean batter and a point on each curve."""
        bd = Dr.bounds(1.84, 92.0)
        for n, v in (("shoulder_R", 123.0), ("shoulder_L", 92.0), ("elbow_ext_L", 85.0), ("wrist_L", 51.0)):
            self.assertAlmostEqual(bd[n], v, delta=0.6, msg=n)
        for name, w, f in (("sh_hadd", 3.14, 1.0), ("sh_hadd", 5.24, 0.705), ("sh_habd", 3.14, 0.93), ("sh_habd", 6.1, 0.576),
                           ("elbow_ext", 3.0, 0.544), ("elbow_ext", 6.1, 0.319), ("wrist_peleg", 1.57, 1.0), ("wrist_peleg", 5.0, 0.541),
                           ("wrist_delp", 3.0, 0.55)):
            self.assertAlmostEqual(E.fv_tab(Dr.CURVES[name], w), f, delta=0.006, msg=name)
        for name, deg in (("sh_hadd", 2.8), ("sh_habd", 5.2), ("elbow_ext", 10.6)):
            self.assertAlmostEqual(math.degrees(Dr.SEC_STRETCH[name]), deg, delta=0.06)

    def test_session_7s_curves_and_bounds_are_the_committed_ones(self):
        """body-motion 4.5's last block, typed here from its tables (not read from drive.py)."""
        bd = Dr.bounds(1.84, 92.0)
        for n, v in (("sh_raise_L", 112.0), ("sh_lower_R", 136.0), ("sh_irot_R", 70.0), ("sh_erot_L", 39.0), ("elbow_flex_L", 112.0),
                     ("pron_R", 17.4), ("sup_L", 18.3), ("wrist_flex_R", 46.0), ("wrist_ext_L", 24.0), ("core_back", 241.0)):
            self.assertAlmostEqual(bd[n], v, delta=0.6, msg=n)
        small = Dr.bounds(1.75, 77.0)
        self.assertAlmostEqual(small["pron_L"], 17.4, delta=0.06)       # one player's value, not scaled by size
        self.assertAlmostEqual(small["sh_raise_L"] / bd["sh_raise_L"], 1.75 * 77.0 / (1.84 * 92.0), places=9)
        for name, w, f in (("sh_raise", 3.0, 0.59), ("sh_raise", 9.0, 0.24), ("sh_lower", 5.0, 0.63), ("sh_irot", 3.14, 1.0),
                           ("sh_irot", 5.24, 0.833), ("sh_irot", 7.85, 0.611), ("sh_irot", 13.0, 0.263), ("sh_erot", 7.85, 0.503),
                           ("sh_erot", 9.0, 0.414), ("trunk_back", 1.05, 0.893), ("trunk_back", 4.19, 0.648), ("trunk_back", 11.2, 0.0)):
            self.assertAlmostEqual(E.fv_tab(Dr.CURVES[name], w), f, delta=0.006, msg=name)
        hyp = lambda wm, wc, w: (wc / wm) * (wm - w) / (wc + w)        # Kentel's printed (top speed, omega_c), note 9
        for name, wm, wc in (("elbow_flex", 18.10, 5.43), ("pron", 27.88, 9.71), ("sup", 35.45, 16.90), ("wrist_flex", 21.40, 10.70),
                             ("wrist_ext", 18.08, 7.78), ("sh_lower", 31.93, 15.02)):
            for w in (2.0, 6.1, 13.0):
                self.assertAlmostEqual(E.fv_tab(Dr.CURVES[name], w), hyp(wm, wc, w), delta=1e-9, msg=name)
            pts = Dr.CURVES[name]
            self.assertTrue(all(a[1] > b_[1] for a, b_ in zip(pts, pts[1:])), name)     # falls all the way to zero at its top speed
            self.assertEqual(pts[-1], (wm, 0.0))
        for name, deg in (("sh_raise", 2.3), ("sh_lower", 2.9), ("sh_irot", 4.1), ("elbow_flex", 6.7)):
            self.assertAlmostEqual(math.degrees(Dr.SEC_STRETCH[name]), deg, delta=0.06)
        for name in ("sh_erot", "pron", "sup", "wrist_flex", "wrist_ext", "trunk_back"):
            self.assertNotIn(name, Dr.SEC_STRETCH)                    # no tendon spring: 4.5

    def test_floating_axis_against_spherical_coordinates(self):
        """a drive about the axis square to two directions (raising the arm) against the same motion written in
        spherical coordinates and integrated here."""
        (th, ph), (rth, rph) = floating_axis_error()
        self.assertGreater(rth - 0.9, 0.3)               # the torque raised the rod by a third of a radian or more
        self.assertGreater(rph - math.pi, 0.5)           # and it went round the vertical meanwhile, so the axis moved
        self.assertAlmostEqual(th, rth, delta=1e-6)      # 2e-7 at this step, 1e-8 at half of it (fourth order)
        self.assertLess(abs(math.atan2(math.sin(ph - rph), math.cos(ph - rph))), 1e-6)

    def test_tendon_spring_holds_the_muscles_full_strength_when_nothing_moves(self):
        """against a rod too heavy to move, the spring stretches until it carries exactly the muscle's
        isometric torque T a: stretch T a / K."""
        sys = E.System(g=(0.0, 0.0, 0.0))
        b = sys.body("anvil", 1e6, (1e6, 1e6, 1e6))
        sys.hinge(-1, b, (0, 0, 0), (0, 0, 0), (0, 1, 0), (0, 1, 0))
        sys.fvtabs = [SEC_TAB]
        sys.actuator("m", -1, b, (0, 1, 0), 120.0, 20.0, -1.0, 0.0, 9.0, 0.6, 0, 1000.0)
        s = sys.new_state()
        s[13] = 0.6
        aux = sys.new_aux()
        sys.diag = {}
        s = sys.run(s, 0.0, 0.5, 1e-3, aux=aux)
        self.assertAlmostEqual(aux[0], 120.0 * s[13] / 1000.0, delta=5e-6)     # the anvil creeps at 6e-5 rad/s
        sys.deriv(0.5, s, aux)
        self.assertAlmostEqual(sys.diag["act"]["m"][0], 120.0 * s[13], delta=5e-3)
        self.assertGreater(s[13], 0.99)                   # the excitation rose from its starting level to full

    def test_tendon_energy_is_accounted_for(self):
        """the muscle's work by its own law equals the work delivered to the joint plus the energy in the
        spring; the gap shrinks with the step (the spring is advanced by a first-order implicit step)."""
        g1, w1, e1, th1 = tendon_ledger(5e-4)
        g2, w2, e2, th2 = tendon_ledger(2.5e-4)
        self.assertGreater(e2, 2.0)                       # the spring really stored energy (J)
        self.assertGreater(abs(w2), 10.0)                 # and the muscle really worked
        self.assertLess(abs(g2), 0.02 * abs(w2))
        self.assertLess(abs(g2), 0.7 * abs(g1))
        g3, w3, e3, th3 = tendon_ledger(2.5e-4, ecc=1.4)      # the same account on a curve with the arms' plateau
        self.assertLess(abs(g3), 0.02 * abs(w3))


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

    def test_no_passive_stop_does_net_positive_work(self):
        """a rod alone on a ball joint, no gravity, no drive, turned into one stop: its kinetic energy plus the spring's
        stored energy never exceeds what it started with, whatever the rod's tilt out of the stop's plane and its spin
        (session 8's review, P1-1: the rod tilted 30 and 60 deg). With damping it only loses. Both kinds of stop."""
        for kind in (0, 1):
            for tilt in (0.0, 30.0, 60.0):
                for c in (0.0, 20.0):
                    e0, e1, emax, booked = rod_on_a_stop(tilt, kind, c=c)
                    msg = "kind %d, tilt %.0f, damping %.0f" % (kind, tilt, c)
                    self.assertLess(emax, e0 * (1.0 + 1e-3), msg=msg)
                    self.assertLess(booked, 1e-3 * e0, msg=msg)          # the work the engine books to the stop, with what it still stores
                    if c == 0.0:
                        self.assertAlmostEqual(e1, e0, delta=1e-3 * e0, msg=msg)     # and with no damper it gives it all back
                    else:
                        self.assertLess(e1, 0.9 * e0, msg=msg)

    def test_the_stop_law_before_the_review_fails_that_check(self):
        """the same rod with the stop's torque about its fixed axis (the law until session 8's review) gains energy
        many times over once it is tilted out of the stop's plane: the check above can fail."""
        E.MUT.add("stop_fixed_axis")
        try:
            e0, e1, emax, booked = rod_on_a_stop(30.0, 0)
        finally:
            E.MUT.discard("stop_fixed_axis")
        self.assertGreater(emax, 10.0 * e0)
        e0, e1, emax, booked = rod_on_a_stop(0.0, 0)                      # in the stop's own plane the two laws are the same
        self.assertLess(emax, e0 * (1.0 + 1e-3))

    def test_a_stop_fails_loudly_where_its_angle_is_undefined(self):
        """a stop whose direction lies along its own axis has no angle (kind 0) or no gradient (kind 1): the run
        raises instead of returning a clamped torque."""
        for kind, f in ((0, (0, 0, 1)), (1, (0, 0, 1))):
            sys = E.System(g=(0.0, 0.0, 0.0))
            b = sys.body("rod", 2.0, (0.15, 0.15, 0.002))
            sys.ball(-1, b, (0, 0, 0), (0, 0, 0.3))
            sys.stop("st", -1, b, (0, 0, 1), (1, 0, 0), f, -0.1, 0.1, 2000.0, 0.0, kind)
            st = sys.new_state()
            sys.set_body(st, b, (0.0, 0.0, -0.3), (1.0, 0.0, 0.0, 0.0))
            with self.assertRaises(E.SolveError):
                sys.deriv(0.0, st, sys.new_aux())
        sys = E.System(g=(0.0, 0.0, 0.0))                   # kind 2: the limb's own axis turned right over from its rest direction
        b = sys.body("rod", 2.0, (0.15, 0.15, 0.002))
        sys.ball(-1, b, (0, 0, 0), (0, 0, 0.3))
        sys.stop("st", -1, b, (0, 0, 1), (1, 0, 0), (1, 0, 0), -0.1, 0.1, 2000.0, 0.0, 2, (0, 0, 1))
        st = sys.new_state()
        for deg, fails in ((170.0, False), (177.0, True)):
            q = E.qaxis((0, 1, 0), math.radians(deg))
            sys.set_body(st, b, E.scale(E.mv(E.qR(q), (0, 0, 0.3)), -1.0), q)
            if fails:
                with self.assertRaises(E.SolveError):
                    sys.deriv(0.0, st, sys.new_aux())
            else:
                sys.deriv(0.0, st, sys.new_aux())

    def test_a_limbs_turn_is_read_as_its_swing_twist_and_its_gradient_is_its_rate(self):
        """the kind 2 stop (body-motion 4.11): its reading equals the turn left over when the limb's quaternion is
        split into a swing and a twist about its own axis (computed here, 2 atan2(q_z, q_w)), at swings up to 150 deg
        in any direction; it does not move when the limb is swung further without turning; and its rate under any
        angular velocity w is w . g with the engine's gradient g (finite differences of the reading)."""
        S = E.System
        e, n = (1.0, 0.0, 0.0), (0.0, 0.0, 1.0)
        rd = lambda q: S.stop_angle(n, e, E.mv(E.qR(q), e), 2, v=E.mv(E.qR(q), n))
        for az in (0.0, 70.0, 200.0, 310.0):
            ax = (math.cos(math.radians(az)), math.sin(math.radians(az)), 0.0)
            for swing in (0.0, 35.0, 90.0, 150.0):
                for turn in (-100.0, -20.0, 0.0, 60.0):
                    q = E.qmul(E.qaxis(ax, math.radians(swing)), E.qaxis(n, math.radians(turn)))
                    th, g = rd(q)
                    msg = "swing %.0f about %.0f, turn %.0f" % (swing, az, turn)
                    self.assertAlmostEqual(math.degrees(th), turn, delta=1e-9, msg=msg)          # a swing about a level axis is the shortest one
                    tw = 2.0 * math.atan2(q[3], q[0])
                    self.assertAlmostEqual((math.degrees(tw) - turn + 180.0) % 360.0 - 180.0, 0.0, delta=1e-9, msg=msg)
                    for w in ((0.3, -0.2, 0.9), (1.0, 0.4, -0.5), (-0.6, 0.8, 0.1)):
                        d = 1e-6
                        q2 = E.qmul(E.qaxis(w, E.norm(w) * d), q)
                        rate = (rd(q2)[0] - rd(E.qmul(E.qaxis(w, -E.norm(w) * d), q))[0]) / (2 * d)
                        self.assertAlmostEqual(rate, E.dot(w, g), delta=1e-6 * (1.0 + abs(rate)), msg=msg)

    def test_a_limbs_turn_stop_does_no_net_positive_work(self):
        """the passivity check for the kind 2 stop (session 8's lesson, before any search uses it): the rod alone,
        swung 0, 40, 90 and 130 deg from its rest direction, damped and not, and with a 30 N m torque driving it into
        the stop: its energy never exceeds what it started with plus what the drive put in, the stop's booked work
        plus what its spring holds is never positive, and the engine's reading agrees with the quaternion's."""
        for tilt in (0.0, 40.0, 90.0, 130.0):
            for c in (0.0, 20.0):
                for drive in (0.0, 30.0):
                    e0, e1, emax, booked, worst, wd = rod_on_a_turn_stop(tilt, c=c, drive=drive)
                    msg = "tilt %.0f, damping %.0f, drive %.0f" % (tilt, c, drive)
                    self.assertLess(worst, 1e-9, msg=msg)
                    self.assertLess(booked, 1e-3 * e0, msg=msg)
                    if not drive:
                        self.assertLess(emax, e0 * (1.0 + 1e-3), msg=msg)
                        if c == 0.0:
                            self.assertAlmostEqual(e1, e0, delta=1e-3 * e0, msg=msg)
                        else:
                            self.assertLess(e1, 0.95 * e0, msg=msg)
                    else:
                        self.assertLess(e1, e0 + wd + 1e-3 * e0, msg=msg)

    def test_a_limbs_turn_stop_with_the_torque_about_the_limbs_axis_alone_fails_that_check(self):
        """the mutation: the kind 2 stop's torque about the limb's own axis instead of the reading's gradient. A rod
        swung 130 deg from its rest direction gains a fifth of its energy from the stop (swung 40 deg it loses 3 %,
        which is as wrong); unswung the two laws are the same."""
        E.MUT.add("twist_fixed_axis")
        try:
            e0, e1, emax, booked, worst, wd = rod_on_a_turn_stop(130.0)
            f0 = rod_on_a_turn_stop(0.0)
        finally:
            E.MUT.discard("twist_fixed_axis")
        self.assertGreater(emax, 1.1 * e0)
        self.assertGreater(booked, 0.1 * e0)
        self.assertLess(f0[2], f0[0] * (1.0 + 1e-3))


def rod_on_a_turn_stop(tilt_deg, c=0.0, k=2000.0, T=0.1, h=1e-4, drive=0.0):
    """a 2 kg rod on a ball joint at the world's origin, no gravity, turning about its own length into a kind 2 stop
    (a limb's turn, body-motion 4.11) whose rest direction is the world's z: the rod starts swung `tilt_deg` from that
    direction, spinning about its own axis into the stop and swinging as well. drive: a constant torque (N m) about the
    rod's own axis pushing it into the stop. The turn is read HERE from the rod's quaternion (its swing-twist split,
    2 atan2(q_z, q_w)), not by the engine's carried-direction formula.
    Returns (energy at the start, at the end, the largest seen, the stop's booked work + what its spring still holds,
    the largest difference between the engine's reading and this one, the drive's booked work)."""
    s = E.System(g=(0.0, 0.0, 0.0))
    I = (0.15, 0.15, 0.002)
    b = s.body("rod", 2.0, I)
    s.ball(-1, b, (0.0, 0.0, 0.0), (0.0, 0.0, 0.3))
    s.stop("st", -1, b, (0, 0, 1), (1, 0, 0), (1, 0, 0), -10.0, 0.0, k, c, 2, (0, 0, 1))
    if drive:
        s.actuator("push", b, -1, (0, 0, -1), drive, 1e9, -1.0, 0.0, 1e9, 1.0)     # on the rod about its own +z; no fall with speed
    st = s.new_state()
    q = E.qmul(E.qaxis((0, 1, 0), math.radians(tilt_deg)), E.qaxis((0, 0, 1), -0.05))
    R = E.qR(q)
    x = E.scale(E.mv(R, (0, 0, 0.3)), -1.0)
    w = E.add(E.scale(E.mv(R, (0, 0, 1)), 3.0), (0.7, 0.4, 0.0))       # spin into the stop, and a swing
    s.set_body(st, b, x, q, E.cross(w, x), w)
    if drive:
        st[13 * s.nb] = 1.0
    aux = s.new_aux()
    ow = 13 * s.nb + len(s.acts)

    def energy(state):
        xx, qq, vv, ww = s.pose(state, b)
        wl = E.mtv(E.qR(qq), ww)
        th = 2.0 * math.atan2(qq[3], qq[0])
        th = (th + math.pi) % (2.0 * math.pi) - math.pi
        pe = 0.5 * k * th * th if th > 0 else 0.0
        return 0.5 * 2.0 * E.dot(vv, vv) + 0.5 * sum(I[i] * wl[i] ** 2 for i in range(3)) + pe, pe, th
    e0 = energy(st)[0]
    emax, worst = e0, 0.0
    s.diag = {}
    for i in range(int(round(T / h))):
        st = s.step(i * h, st, aux, h)
        if i % 10 == 0:
            e, pe, th = energy(st)
            emax = max(emax, e)
            s.deriv(i * h, st, aux)
            worst = max(worst, abs(s.diag["stop"]["st"][1] - th))
    s.diag = None
    e1, pe1, th = energy(st)
    return e0, e1, emax, st[ow + 1] + pe1, worst, st[ow]


# ---- 7. the whole body ---------------------------------------------------------------------------
T0 = -0.250
MODEST = {"hip_rot_R": 0.6, "hip_rot_L": 0.6, "hip_ext_R": 0.3, "hip_flex_L": 0.3, "knee_ext_L": 0.1, "knee_ext_R": 0.1,
          "hip_abd_R": 0.3, "hip_add_L": 0.3, "knee_flex_R": 0.2, "ankle_pf_R": 0.2,
          "core": 0.8, "shoulder_L": 0.8, "shoulder_R": 0.8, "elbow_ext_L": 0.3, "elbow_ext_R": 0.5, "wrist_L": 1.0, "wrist_R": 1.0}
# session 7's channels (body-motion 4.5): every one on, so the momentum and energy checks see each of them
MODEST.update({"core_back": 0.3, "sh_raise_L": 0.5, "sh_lower_R": 0.5, "sh_raise_R": 0.2, "sh_lower_L": 0.2, "sh_irot_R": 0.8, "sh_erot_R": 0.3,
               "sh_irot_L": 0.3, "sh_erot_L": 0.8, "elbow_flex_L": 0.4, "elbow_flex_R": 0.2, "pron_R": 1.0, "sup_R": 0.5, "pron_L": 0.5,
               "sup_L": 1.0, "wrist_flex_R": 0.6, "wrist_ext_R": 0.3, "wrist_flex_L": 0.3, "wrist_ext_L": 0.6})


def drive_turns(name):
    """channel `name` alone at its bound on the placed body, off the ground: the change it makes to the angular
    acceleration of its joint, as (child's minus parent's, world axes), with the model."""
    m = B.Model(contacts=False)
    st = B.place(m, T0)
    bd = Dr.bounds(m.H, m.M)
    acc = []
    s1 = list(st)
    for T in (0.0, bd[name]):
        s1 = Dr.add_drives(m, s1, {name: (T, T0 - 1.0, 0.05, 1.0, 1.0)})
        aux = m.sys.new_aux()
        m.sys.sec_start(s1, aux)
        ds = m.sys.deriv(T0, s1, aux)
        acc.append([ds[13 * i + 10:13 * i + 13] for i in range(m.sys.nb)])
    return m, st, [E.sub(tuple(a1), tuple(a0)) for a0, a1 in zip(*acc)]


def anatomy(m, st, sd):
    """directions read from the joint centres alone (not from any body's own axes): the upper arm from shoulder to
    elbow, the forearm from elbow to wrist, the trunk's long axis from the lumbar joint to between the shoulders'
    level (its own z is used only for the head end's sense), the bat from knob to barrel."""
    S, El, W = m.joint_pos(st, "shoulder_" + sd), m.joint_pos(st, "elbow_" + sd), m.joint_pos(st, "wrist_" + sd)
    mid = E.scale(E.add(m.joint_pos(st, "shoulder_L"), m.joint_pos(st, "shoulder_R")), 0.5)
    zt = E.unit(E.sub(mid, m.joint_pos(st, "lumbar")))
    bz = E.unit(E.sub(m.bat_point(st, 0.8)[0], m.bat_point(st, 0.0)[0]))
    return E.unit(E.sub(El, S)), E.unit(E.sub(W, El)), zt, bz


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
    def test_attack_angle_of_a_constructed_swing(self):
        """the attack angle observe() reports is the angle above level of the sweet spot's path, for a bat placed
        level along +X and turned about the lead grip on an axis tilted by a known angle from the vertical: the
        sweet spot then moves toward +Y and climbs at exactly that angle (constructed here by hand; down for a
        negative tilt). The bat's own speed and the pose of the other bodies do not enter."""
        m = B.Model(1.84, 92.0, contacts=False)
        st = list(B.place(m, T0))
        i = 13 * m.b["bat"]
        h = math.sqrt(0.5)
        st[i + 3:i + 7] = [h, 0.0, h, 0.0]               # the bat's +z (knob to end) along world +X
        grip = (0.3, -0.1, 1.0)                          # the lead grip, anywhere
        for alpha_deg in (12.0, -20.0, 0.0):
            a = alpha_deg * math.pi / 180.0
            w = (0.0, -35.0 * math.sin(a), 35.0 * math.cos(a))
            xcm = (grip[0] + (m.bat_ucm - B.GRIP_L), grip[1], grip[2])
            r = (xcm[0] - grip[0], 0.0, 0.0)
            v = (w[1] * r[2] - w[2] * r[1], w[2] * r[0] - w[0] * r[2], w[0] * r[1] - w[1] * r[0])
            st[i:i + 3] = list(xcm)
            st[i + 7:i + 10] = list(v)
            st[i + 10:i + 13] = list(w)
            o = m.observe(st)
            self.assertAlmostEqual(o["attack"], alpha_deg, delta=1e-9)
            self.assertAlmostEqual(o["sweet_speed"], 35.0 * (B.C.BAT_SS - B.GRIP_L), delta=1e-9)

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
    def test_each_of_session_7s_drives_turns_its_joint_the_named_way(self):
        """each new channel, alone, against the anatomical meaning of its name, written here from the joint centres:
        raising takes the upper arm away from hanging along the trunk; internal rotation turns the right upper arm
        about its own axis the way that swings a forward-pointing forearm across the front of the body (a positive
        turn about the line from elbow to shoulder), the left the other way; flexion closes the elbow; pronation
        turns the right forearm positively about the line from wrist to elbow, the left the other way; with the thumb
        toward the barrel, wrist flexion turns the bat against the forearm about the knob-to-barrel line for the
        right hand and the other way for the left; the trunk's second channel turns it back against the pelvis."""
        for sd, right in (("L", -1.0), ("R", 1.0)):
            for name, parent, child, want in (
                    ("sh_raise_", "trunk", "uarm_", lambda u, f, zt, bz: E.unit(E.cross(u, zt))),
                    ("sh_lower_", "trunk", "uarm_", lambda u, f, zt, bz: E.unit(E.cross(zt, u))),
                    ("sh_irot_", "trunk", "uarm_", lambda u, f, zt, bz: E.scale(u, -right)),
                    ("sh_erot_", "trunk", "uarm_", lambda u, f, zt, bz: E.scale(u, right)),
                    ("elbow_flex_", "uarm_", "farm_", lambda u, f, zt, bz: E.unit(E.cross(f, E.scale(u, -1.0)))),
                    ("elbow_ext_", "uarm_", "farm_", lambda u, f, zt, bz: E.unit(E.cross(E.scale(u, -1.0), f))),
                    ("pron_", "uarm_", "farm_", lambda u, f, zt, bz: E.scale(f, -right)),
                    ("sup_", "uarm_", "farm_", lambda u, f, zt, bz: E.scale(f, right)),
                    ("wrist_flex_", "farm_", "bat", lambda u, f, zt, bz: E.scale(bz, right)),
                    ("wrist_ext_", "farm_", "bat", lambda u, f, zt, bz: E.scale(bz, -right))):
                m, st, dacc = drive_turns(name + sd)
                u, f, zt, bz = anatomy(m, st, sd)
                pa = m.b[parent if parent == "trunk" else parent + sd]
                ch = m.b[child if child == "bat" else child + sd]
                turn = E.dot(E.sub(dacc[ch], dacc[pa]), want(u, f, zt, bz))
                self.assertGreater(turn, 1.0, name + sd)           # rad/s^2 about the named direction
        for name, sign in (("core", 1.0), ("core_back", -1.0)):
            m, st, dacc = drive_turns(name)
            up = E.unit(E.sub(m.joint_pos(st, "lumbar"), E.scale(E.add(m.joint_pos(st, "hip_L"), m.joint_pos(st, "hip_R")), 0.5)))
            self.assertGreater(sign * E.dot(E.sub(dacc[m.b["trunk"]], dacc[m.b["pelvis"]]), up), 1.0, name)

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


class RangeChecks(unittest.TestCase):
    """the joints' ranges as stops (body-motion 4.10, session 8)."""

    @NEED
    def test_ranges_against_independent_measures_and_clear_of_the_start(self):
        """range_angles reads the knees' and elbows' bends as observe() does by another route (the angle between
        the segments' long axes) and the trunk's twist as the difference of the two measured rotations; every stop
        begins at least RANGE_CLEAR beyond the start, so none carries anything there; and the old body is unchanged."""
        m = B.Model(ranges=())
        st = B.place(m, T0)
        lim = B.add_ranges(m, st)
        a, o = B.range_angles(m, st), m.observe(st)
        for k, ok in (("knee_L", "lead_knee_flex"), ("knee_R", "trail_knee_flex"), ("elbow_L", "lead_elbow_flex"), ("elbow_R", "trail_elbow_flex")):
            self.assertAlmostEqual(a[k], o[ok], delta=0.05, msg=k)
        self.assertAlmostEqual(a["trunk_twist"], o["trunk_rot"] - o["pelvis_rot"], delta=1.5)   # the trunk leans 15 deg, so not exact
        self.assertEqual(len(lim), 21)
        for k, (lo, hi) in lim.items():
            self.assertGreaterEqual(a[k] - lo, B.RANGE_CLEAR - 1e-9, msg=k)
            self.assertGreaterEqual(hi - a[k], B.RANGE_CLEAR - 1e-9, msg=k)
        for k in ("wrist_dev_L", "wrist_dev_R"):          # a wrist's cock has 58 deg of travel, wherever it starts
            self.assertAlmostEqual(lim[k][1] - lim[k][0], 58.0, delta=1e-9)
        m.sys.diag = {}
        m.sys.deriv(T0, st, m.sys.new_aux())
        self.assertTrue(all(tau == 0.0 for tau, th in m.sys.diag["stop"].values()))
        m.sys.diag = None
        self.assertEqual([x[0] for x in B.Model().sys.stops], ["knee_L", "elbow_L", "shoulder_L", "knee_R", "elbow_R", "shoulder_R"])

    @NEED
    def test_the_cocks_reading_does_not_move_when_flexion_passes_90(self):
        """a wrist's cock read on the joint's own axis: turning the forearm about its own x (the wrist's flexion)
        through 130 deg changes the flexion reading by that much and the cock's reading not at all. The reading
        used until session 8's review (the forearm's long axis projected onto the bat's x-z plane) turns over by
        180 deg on the way, which is what this test is for (review P1-3)."""
        m = B.Model(ranges=())
        st0 = B.place(m, T0)
        m.cock_ref = B.cock_reference(m, st0)
        for sd in "LR":
            i = m.b["farm_" + sd]
            a0 = B.range_angles(m, st0)
            fx = m.axis(st0, "farm_" + sd, (1, 0, 0))
            old0 = None
            for sign in (1.0, -1.0):
                for deg in range(0, 131, 10):
                    st = list(st0)
                    q = E.qmul(E.qaxis(fx, sign * deg * B.D2R), tuple(st0[13 * i + 3:13 * i + 7]))
                    st[13 * i + 3:13 * i + 7] = q
                    a = B.range_angles(m, st)
                    self.assertAlmostEqual(a["wrist_dev_" + sd], a0["wrist_dev_" + sd], delta=1e-6, msg="%s %+.0f" % (sd, sign * deg))
                    turned = (a["wrist_flex_" + sd] - a0["wrist_flex_" + sd] + 180.0) % 360.0 - 180.0
                    self.assertAlmostEqual(abs(turned), deg, delta=1e-6)
                    zf, bx, bz = m.axis(st, "farm_" + sd, (0, 0, 1)), m.axis(st, "bat", (1, 0, 0)), m.axis(st, "bat", (0, 0, 1))
                    old = math.atan2(E.dot(zf, bz), -E.dot(zf, bx)) / B.D2R       # the reading before the review
                    old0 = old if deg == 0 else old0
                    worst = max(abs((old - old0 + 180.0) % 360.0 - 180.0), 0.0)
                    if deg == 130:
                        self.assertGreater(worst, 90.0, msg="the old reading should have turned over on side %s" % sd)

    @NEED
    def test_the_wrists_travel_lies_the_way_the_swing_uncocks(self):
        """the body driven by its two uncocking wrist muscles alone for 40 ms: the lag falls, and each wrist's cock
        moves toward the end of its range that has more than half of the 58 deg (review P1-2: the travel had been
        placed on the other side, leaving 5 deg to uncock). The motion is the engine's, not add_ranges' own sign."""
        m = B.Model(ranges=())
        st = B.place(m, T0)
        bd = Dr.bounds(m.H, m.M)
        st = Dr.add_drives(m, st, {n: (bd[n], T0, 0.05, 1.0) for n in ("wrist_L", "wrist_R")})
        lim = B.add_ranges(m, st)
        Dr.solve_tone(m, st, T0, aux=list(m.start_aux))
        Dr.add_damping(m, st)
        aux = list(m.start_aux)
        m.sys.sec_start(st, aux)
        a0, lag0 = B.range_angles(m, st), m.observe(st)["bat_lag"]
        s1 = list(st)
        for i in range(40):
            s1 = m.sys.step(T0 + i * 1e-3, s1, aux, 1e-3)
        a1, lag1 = B.range_angles(m, s1), m.observe(s1)["bat_lag"]
        self.assertLess(lag1, lag0 - 1.0)                                  # the bat uncocked (deg)
        for k in ("wrist_dev_L", "wrist_dev_R"):
            lo, hi = lim[k]
            moved = a1[k] - a0[k]
            self.assertGreater(abs(moved), 0.5, msg=k)
            room = hi - a0[k] if moved > 0 else a0[k] - lo
            self.assertGreater(room, 0.5 * (hi - lo), msg="%s can uncock only %.1f of %.1f deg" % (k, room, hi - lo))
            # what the reading means, by another route: 90 deg less the angle from the forearm (toward the elbow) to the
            # barrel; uncocking opens that angle (the bat toward a straight line with the forearm)
            for a_, s_ in ((a0, st), (a1, s1)):
                zf, bz = m.axis(s_, "farm_" + k[-1], (0, 0, 1)), m.axis(s_, "bat", (0, 0, 1))
                self.assertAlmostEqual(a_[k], 90.0 - math.acos(E.dot(zf, bz)) / B.D2R, delta=2.5, msg=k)
            self.assertLess(moved, 0.0, msg=k)
        mir = B.Model(ranges=())                                           # the review's placing fails this test, as it must
        stm = B.place(mir, T0)
        limm = B.add_ranges(mir, stm, wrists="mirrored")
        for k in ("wrist_dev_L", "wrist_dev_R"):
            self.assertLess(a0[k] - limm[k][0], 0.5 * (limm[k][1] - limm[k][0]))

    @NEED
    def test_the_hips_sideways_angle_is_the_joints_own(self):
        """a thigh set 30 deg out to the side reads 30 deg however far the hip is also bent forward (0, 45, 70 deg),
        and for either sign; the projection about the pelvis's x used until session 8's review reads 39 at 45 (the
        review's P3). The arm across the chest is read the same way: 0 with the arm hanging."""
        m = B.Model(ranges=())
        st0 = B.place(m, T0)
        qp = tuple(st0[13 * m.b["pelvis"] + 3:13 * m.b["pelvis"] + 7])
        i = m.b["thigh_L"]
        for flex in (0.0, 45.0, 70.0):
            for abd in (30.0, -20.0):
                st = list(st0)
                st[13 * i + 3:13 * i + 7] = E.qmul(qp, E.qmul(E.qaxis((0, 1, 0), -flex * B.D2R), E.qaxis((1, 0, 0), abd * B.D2R)))
                a = B.range_angles(m, st)
                self.assertAlmostEqual(a["hip_abd_L"], abd, delta=1e-6, msg="flexion %.0f" % flex)
                self.assertAlmostEqual(a["hip_flex_L"], -flex, delta=1e-6)
                zt, px, pz = m.axis(st, "thigh_L", (0, 0, 1)), m.axis(st, "pelvis", (1, 0, 0)), m.axis(st, "pelvis", (0, 0, 1))
                old = math.atan2(E.dot(px, E.cross(pz, zt)), E.dot(pz, zt)) / B.D2R
                if flex == 45.0 and abd == 30.0:
                    self.assertAlmostEqual(old, 39.2, delta=0.1)
        qt = tuple(st0[13 * m.b["trunk"] + 3:13 * m.b["trunk"] + 7])
        j = m.b["uarm_L"]
        st = list(st0)
        st[13 * j + 3:13 * j + 7] = qt                                     # the arm hanging along the trunk
        a = B.range_angles(m, st)
        self.assertAlmostEqual(a["shoulder_L"], 0.0, delta=1e-6)
        self.assertAlmostEqual(a["shoulder_back_L"], 0.0, delta=1e-6)
        st[13 * j + 3:13 * j + 7] = E.qmul(qt, E.qaxis((1, 0, 0), -60.0 * B.D2R))   # swung 60 deg across the body (the left arm toward the right)
        self.assertAlmostEqual(B.range_angles(m, st)["shoulder_L"], 60.0, delta=1e-6)
        st[13 * j + 3:13 * j + 7] = E.qmul(qt, E.qaxis((0, 1, 0), 50.0 * B.D2R))    # swung 50 deg back behind the shoulders' line
        self.assertAlmostEqual(B.range_angles(m, st)["shoulder_back_L"], 50.0, delta=1e-6)

    @NEED
    def test_an_arm_muscle_tensed_beforehand_only_holds(self):
        """with the hold-still torques re-solved for the starting levels (drive.hold_response), a body whose arm
        muscles are all tensed at 0.7 of their bounds starts with the accelerations of the body with none tensed; left
        as they were, the same tensing moves it (review P2-4: a starting level was net drive from the first instant)."""
        m = B.Model(ranges=())
        st = B.place(m, T0)
        bd = Dr.bounds(m.H, m.M)
        arms = Dr.ARM_CHANNELS + Dr.NEW_ARM_CHANNELS
        st = Dr.add_drives(m, st, {n: (0.0, 0.0, 0.05, 0.1, 0.0) for n in arms})
        B.add_ranges(m, st)
        tone, _ = Dr.solve_tone(m, st, T0, aux=list(m.start_aux))
        W = Dr.hold_response(m, st, T0, list(m.start_aux), arms, bd)
        s = m.sys

        def acc(state):
            aux = list(m.start_aux)
            s.anchors(state, aux)
            s.sec_start(state, aux)
            ds = s.deriv(T0, state, aux)
            return [ds[13 * i + 7 + c] for i in range(s.nb) for c in range(6)]
        base = acc(st)
        oa = 13 * s.nb
        st2 = list(st)
        for j, a in enumerate(s.acts):
            a[4], a[9] = bd[a[0]], 0.7
            st2[oa + j] = 0.7
        moved = max(abs(x - y) for x, y in zip(acc(st2), base))
        self.assertGreater(moved, 10.0)                                    # tensed and not held: it moves (rad/s^2)
        tv = [c for j in Dr.JOINTS for c in tone[j]]
        for n in arms:
            tv = [x + 0.7 * y for x, y in zip(tv, W[n])]
        s.tones = [(nm, a, b, tuple(tv[3 * k:3 * k + 3])) for k, (nm, a, b, _) in enumerate(s.tones)]
        held = max(abs(x - y) for x, y in zip(acc(st2), base))
        self.assertLess(held, 1e-3 * moved)

    @NEED
    def test_the_forearms_turn_does_not_move_when_the_cock_passes_the_line_of_the_bat(self):
        """the forearm's turn is read on the elbow's own axis: turning the BAT about the wrists' axis (the cock) through
        150 deg either way, the forearm and upper arm left where they are, changes the cock's reading by that much
        and the forearm's turn not at all. The reading used until session 8's second review chose which end of the
        forearm's x to read by the bat's direction and jumps 180 deg on the way (review P1-2: 106 deg read as 285)."""
        m = B.Model(ranges=())
        st0 = B.place(m, T0)
        m.cock_ref = B.cock_reference(m, st0)
        i = m.b["bat"]
        by = m.axis(st0, "bat", (0, 1, 0))
        a0 = B.range_angles(m, st0)
        for sd, sg in B.SIDES:
            jumped = False
            for sign in (1.0, -1.0):
                for deg in range(0, 151, 10):
                    st = list(st0)
                    st[13 * i + 3:13 * i + 7] = E.qmul(E.qaxis(by, sign * deg * B.D2R), tuple(st0[13 * i + 3:13 * i + 7]))
                    a = B.range_angles(m, st)
                    self.assertAlmostEqual(a["pron_" + sd], a0["pron_" + sd], delta=1e-6, msg="%s %+.0f" % (sd, sign * deg))
                    moved = (a["wrist_dev_" + sd] - a0["wrist_dev_" + sd] + 180.0) % 360.0 - 180.0
                    self.assertAlmostEqual(abs((moved + sign * deg + 180.0) % 360.0 - 180.0), 0.0, delta=1e-6)
                    fx, zf, bz = m.axis(st, "farm_" + sd, (1, 0, 0)), m.axis(st, "farm_" + sd, (0, 0, 1)), m.axis(st, "bat", (0, 0, 1))
                    g = fx if E.dot(fx, bz) > 0 else E.scale(fx, -1.0)          # the reading before the second review
                    r = E.cross(m.axis(st, "uarm_" + sd, (0, 1, 0)), zf)
                    old = -sg * math.atan2(E.dot(zf, E.cross(r, g)), E.dot(r, g)) / B.D2R
                    if abs((old - a0["pron_" + sd] + 180.0) % 360.0 - 180.0) > 90.0:
                        jumped = True
            self.assertTrue(jumped, msg="the old reading should have turned over on side %s" % sd)

    def _wrist_range_between_forearm_and_barrel(self, sd):
        """a wrist's stops as angles between the forearm (wrist toward elbow) and the barrel, with the wrist unflexed:
        90 deg less the cock's reading."""
        m = B.Model(ranges=())
        st = B.place(m, T0)
        lo, hi = B.add_ranges(m, st)["wrist_dev_" + sd]
        return 90.0 - hi, 90.0 - lo

    # what a wrist reaches, as an angle between forearm and barrel: from 22 deg toward the thumb of square (68 deg; Boone &
    # Azen through Norkin & White) to the most opened-out bat-to-lead-forearm angle measured on hitters at contact
    # (157 deg; Gelinas 1988, research note 11 A5), each with the stops' own 2 deg of clearance
    WRIST_REACH = (68.0 - B.RANGE_CLEAR, 157.0 + B.RANGE_CLEAR)

    @NEED
    def test_the_welds_lead_wrist_was_inside_a_wrists_reach_and_its_rear_wrist_was_not(self):
        """the record of session 8's second review, P1-1, on the old weld (kept for the dated results): the lead wrist's
        stops, as angles between forearm and barrel (91 to 149 deg), lie inside what a wrist reaches; the rear's (151
        deg through the line of the bat to 29 deg past it) do not. The hands' body's wrists are HandsChecks' business."""
        lo, hi = self._wrist_range_between_forearm_and_barrel("L")
        self.assertGreaterEqual(lo, self.WRIST_REACH[0])
        self.assertLessEqual(hi, self.WRIST_REACH[1])
        lo, hi = self._wrist_range_between_forearm_and_barrel("R")
        self.assertGreater(hi, 180.0)

    @NEED
    def test_a_stop_pushes_back_only_past_its_end(self):
        """a knee turned past its range is pushed back toward it by the stop's stiffness times the excess, and a knee
        inside its range is not pushed at all (the check fails if the stop's sign or its limits are wrong)."""
        m = B.Model(ranges=())
        st = B.place(m, T0)
        lim = B.add_ranges(m, st)
        j = [x[0] for x in m.sys.stops].index("knee_L")
        name, a_, b_, ax, ea, fb, lo, hi, k, c, kind, axb = m.sys.stops[j]
        self.assertAlmostEqual(hi / B.D2R, B.ROM["knee_flex"], delta=1e-9)
        m.sys.stops[j] = (name, a_, b_, ax, ea, fb, lo, (B.range_angles(m, st)["knee_L"] - 3.0) * B.D2R, k, 0.0, kind, axb)   # the end moved 3 deg inside the knee's angle
        m.sys.diag = {}
        m.sys.deriv(T0, st, m.sys.new_aux())
        tau, th = m.sys.diag["stop"]["knee_L"]
        m.sys.diag = None
        self.assertAlmostEqual(tau, -B.RANGE_K["big"] * 3.0 * B.D2R, delta=1e-6 * B.RANGE_K["big"])


def clinical_arm(m, st, sd, raise_axis=(1.0, 0.0, 0.0), raise_deg=0.0, turn_in=0.0, elbow=90.0, palm_down=0.0, tilt=0.0, bend=0.0):
    """the state st with arm sd rebuilt from the clinical definitions on the trunk as it stands (orientations only):
    the upper arm hanging along the trunk with the elbow bending straight forward, then turned in `turn_in` about its
    own length, then carried `raise_deg` about the trunk's level axis `raise_axis` by a single turn; the elbow bent
    `elbow`; the forearm in neutral (thumb toward the upper arm's side, the way it lies with the thumb up when the
    elbow is bent at the side) and then turned palm-down by `palm_down`; the hand in line with the forearm, then tilted
    `tilt` toward the thumb about the palm's own normal and bent `bend` toward the palm about the forearm's radial line.
    Built from rotations only: no reading of body.py is used. Returns (state, forearm z, forearm x, palm direction)."""
    sg = 1.0 if sd == "L" else -1.0
    r = math.radians
    qt = tuple(st[13 * m.b["trunk"] + 3:13 * m.b["trunk"] + 7])
    turn = lambda ax, deg: E.qaxis(ax, r(deg))
    qu = E.qmul(E.qmul(qt, turn(raise_axis, raise_deg)), E.qmul(turn((0, 0, 1), 180.0), turn((0, 0, 1), -sg * turn_in)))
    qf = E.qmul(E.qmul(qu, turn((0, 1, 0), elbow)), E.qmul(turn((0, 0, 1), 180.0), turn((0, 0, 1), -sg * palm_down)))
    Rf = E.qR(qf)
    zf, xf = E.mv(Rf, (0, 0, 1)), E.mv(Rf, (1, 0, 0))
    palm = E.scale(E.mv(Rf, (0, 1, 0)), -sg)                 # the forearm's palm side
    # the hand: knuckles along -zf, thumb side along xf, palm along `palm`; then tilted and bent
    d, rr, pp = E.scale(zf, -1.0), xf, palm
    qh = E.qaxis(pp, -r(tilt) if sd == "R" else r(tilt))     # toward the thumb: the knuckles' line turns toward the thumb's side
    d, rr = E.mv(E.qR(qh), d), E.mv(E.qR(qh), rr)
    assert E.dot(d, xf) * tilt >= 0.0                        # the knuckles did move toward the thumb's side of the forearm
    qb_ = E.qaxis(xf, r(bend) if sd == "R" else -r(bend))    # toward the palm
    d, rr, pp = (E.mv(E.qR(qb_), v) for v in (d, rr, pp))
    assert E.dot(d, palm) * bend >= 0.0                      # the knuckles did move toward the palm's side
    db, rb, pb = B.hand_frame(sd)
    # the bat's rotation takes the hand's own directions, as fixed in the bat, to where the hand now is
    Rb = [sum(w[i] * b_[j] for w, b_ in ((d, db), (rr, rb), (pp, pb))) for i in range(3) for j in range(3)]
    out = list(st)
    for body, q in (("uarm_" + sd, qu), ("farm_" + sd, qf), ("bat", E.R_to_q(tuple(Rb)))):
        out[13 * m.b[body] + 3:13 * m.b[body] + 7] = q
    return out, zf, xf, palm


class HandsChecks(unittest.TestCase):
    """each hand's own grip (body-motion 4.11, session 9): the hands' body's readings as a clinician defines them,
    each side separately; each carried through its joint's other freedoms; the start; a hitter's contact posture."""

    @classmethod
    def setUpClass(cls):
        if not B.have_curves():
            raise unittest.SkipTest("[4]'s digitised curves are not on this machine (docs/body-motion.md 3)")
        cls.m = B.Model(ranges=(), grip="hands")
        cls.st = B.place(cls.m, T0)
        cls.lim = B.add_ranges(cls.m, cls.st)

    def test_the_forearms_turn_reads_as_a_clinician_reads_it_and_does_not_move_with_the_elbow_or_the_wrist(self):
        """a forearm turned 40 deg palm-down reads 40 on either side, whatever the elbow's bend (10 to 140 deg), the upper
        arm's posture or the wrist's two angles; in neutral, with the arm hanging and the elbow at 90, the thumb is up
        and the palm faces the body's midline; turned 90 palm-down the palm faces the ground."""
        m, st = self.m, self.st
        zt, yt = m.axis(st, "trunk", (0, 0, 1)), m.axis(st, "trunk", (0, 1, 0))
        for sd, sg in B.SIDES:
            s0, zf, xf, palm = clinical_arm(m, st, sd)
            self.assertAlmostEqual(E.dot(xf, zt), 1.0, delta=1e-9)                       # thumb up
            self.assertAlmostEqual(E.dot(palm, E.scale(yt, -sg)), 1.0, delta=1e-9)       # palm to the midline
            s90, zf, xf, palm = clinical_arm(m, st, sd, palm_down=90.0)
            self.assertAlmostEqual(E.dot(palm, zt), -1.0, delta=1e-9)                    # palm down
            for want in (-80.0, -30.0, 0.0, 40.0, 75.0):
                for kw in ({}, {"elbow": 10.0}, {"elbow": 140.0}, {"raise_deg": sg * 80.0, "turn_in": 30.0}, {"tilt": -30.0, "bend": 40.0},
                           {"tilt": 20.0, "bend": -70.0}, {"raise_axis": (0, 1, 0), "raise_deg": -100.0, "turn_in": -60.0, "elbow": 60.0}):
                    c = B.clinical(m, clinical_arm(m, st, sd, palm_down=want, **kw)[0])
                    self.assertAlmostEqual(c[("forearm", sd)], want, delta=1e-7, msg="%s %s" % (sd, kw))

    def test_the_upper_arms_turn_reads_as_a_clinician_reads_it_and_does_not_move_as_the_arm_is_raised(self):
        """an upper arm turned out 60 deg reads -60 hanging, raised forward, raised to the side and raised obliquely to
        150 deg, on either side, whatever the elbow and forearm do. By the clinical picture: hanging with the elbow at
        90 and no turn the forearm points forward, turned in 90 it lies across the belly; straight out to the side with
        no turn the forearm is level and forward, turned out 90 it points up."""
        m, st = self.m, self.st
        xt, yt, zt = (m.axis(st, "trunk", e) for e in E.E3)
        for sd, sg in B.SIDES:
            fore = lambda **kw: E.scale(clinical_arm(m, st, sd, **kw)[1], -1.0)          # from the elbow toward the wrist
            self.assertAlmostEqual(E.dot(fore(), xt), 1.0, delta=1e-9)
            self.assertAlmostEqual(E.dot(fore(turn_in=90.0), E.scale(yt, -sg)), 1.0, delta=1e-9)
            self.assertAlmostEqual(E.dot(fore(raise_deg=sg * 90.0), xt), 1.0, delta=1e-9)
            self.assertAlmostEqual(E.dot(fore(raise_deg=sg * 90.0, turn_in=-90.0), zt), 1.0, delta=1e-9)
            up = E.scale(m.axis(clinical_arm(m, st, sd, raise_deg=sg * 90.0)[0], "uarm_" + sd, (0, 0, 1)), -1.0)
            self.assertAlmostEqual(E.dot(up, E.scale(yt, sg)), 1.0, delta=1e-9)          # that arm does point out to its own side
            # how far the arm is raised reads the raise, whichever way it is raised and however it is turned
            for kw, want in (({}, 0.0), ({"raise_deg": sg * 90.0, "turn_in": 40.0}, 90.0), ({"raise_axis": (0, 1, 0), "raise_deg": -150.0}, 150.0),
                             ({"raise_axis": (0.6, 0.8, 0.0), "raise_deg": 165.0, "turn_in": -70.0, "elbow": 30.0}, 165.0)):
                self.assertAlmostEqual(B.clinical(m, clinical_arm(m, st, sd, **kw)[0])[("raised", sd)], want, delta=1e-6, msg="%s %s" % (sd, kw))
            for want in (-100.0, -60.0, 0.0, 35.0, 65.0):
                for kw in ({}, {"raise_deg": sg * 90.0}, {"raise_axis": (0, 1, 0), "raise_deg": -90.0}, {"raise_axis": (0.6, -0.8, 0.0), "raise_deg": 150.0},
                           {"raise_axis": (0.6, 0.8, 0.0), "raise_deg": -120.0, "elbow": 20.0, "palm_down": 50.0, "tilt": 15.0}):
                    c = B.clinical(m, clinical_arm(m, st, sd, turn_in=want, **kw)[0])
                    self.assertAlmostEqual(c[("upper_arm", sd)], want, delta=1e-7, msg="%s %s" % (sd, kw))

    def test_the_wrists_two_angles_read_as_a_clinician_reads_them_and_neither_moves_with_the_other(self):
        """with the hand in line with the forearm both read 0 and forearm and barrel are 90 deg + the grip's obliqueness
        apart; a wrist tilted 20 deg toward the thumb reads +20 and closes that angle by 20, tilted 30 away it opens it
        by 30, on either side; bent toward the palm reads +, back -, the tilt's reading unmoved through 70 deg of bend
        either way and the bend's unmoved through the tilt; neither moves with the forearm's turn or the elbow."""
        m, st = self.m, self.st
        for sd, sg in B.SIDES:
            c = B.clinical(m, clinical_arm(m, st, sd)[0])
            self.assertAlmostEqual(c[("wrist_dev", sd)], 0.0, delta=1e-7)
            self.assertAlmostEqual(c[("wrist_flex", sd)], 0.0, delta=1e-7)
            self.assertAlmostEqual(c[("forearm_to_barrel", sd)], 90.0 + B.GRIP_OBLIQUE, delta=1e-7)
            for tilt in (-30.0, -10.0, 20.0):
                c = B.clinical(m, clinical_arm(m, st, sd, tilt=tilt)[0])
                self.assertAlmostEqual(c[("forearm_to_barrel", sd)], 90.0 + B.GRIP_OBLIQUE - tilt, delta=1e-7)
                for bend in (-70.0, -20.0, 0.0, 30.0, 60.0):
                    for kw in ({}, {"palm_down": 60.0, "elbow": 30.0}, {"raise_deg": sg * 70.0, "turn_in": -40.0, "palm_down": -50.0}):
                        c = B.clinical(m, clinical_arm(m, st, sd, tilt=tilt, bend=bend, **kw)[0])
                        self.assertAlmostEqual(c[("wrist_dev", sd)], tilt, delta=1e-7, msg="%s tilt %.0f bend %.0f %s" % (sd, tilt, bend, kw))
                        self.assertAlmostEqual(c[("wrist_flex", sd)], bend, delta=1e-7, msg="%s tilt %.0f bend %.0f %s" % (sd, tilt, bend, kw))

    WRIST_REACH = RangeChecks.WRIST_REACH

    def test_each_wrists_range_is_one_a_wrist_reaches(self):
        """each wrist's stops, as the angle between its forearm and the barrel with the wrist posed at each end of its
        tilt (unbent, and bent to each end of its bend), lie inside what a wrist reaches (68 to 157 deg: session 8's
        second review, P1-1, whose test this replaces: it was an expected failure for the rear wrist on the weld)."""
        m, st = self.m, self.st
        ar = B.arm_range()
        for sd, sg in B.SIDES:
            sign = B.clinical_sign("wrist_dev", sg)
            lo, hi = sorted(sign * x for x in self.lim["wrist_dev_" + sd])
            self.assertAlmostEqual(lo, ar["wrist_dev"][0], delta=1e-9)
            self.assertAlmostEqual(hi, ar["wrist_dev"][1], delta=1e-9)
            for tilt in (lo, hi):
                for bend in (0.0,) + ar["wrist_flex"]:
                    ang = B.clinical(m, clinical_arm(m, st, sd, tilt=tilt, bend=bend)[0])[("forearm_to_barrel", sd)]
                    self.assertGreaterEqual(ang, self.WRIST_REACH[0], msg="%s tilt %.1f bend %.1f" % (sd, tilt, bend))
                    self.assertLessEqual(ang, self.WRIST_REACH[1], msg="%s tilt %.1f bend %.1f" % (sd, tilt, bend))

    def test_the_start_lies_inside_every_arm_range_with_no_stop_moved_and_its_elbows_clear(self):
        """the start placed on [4] (flagged), with the bat's roll and the two elbows' swivels set by pose()'s rule: every
        wrist, forearm and upper-arm angle is inside its anatomical range as the stops have it (none widened), no stop
        carries anything, each elbow is ARM_CLEAR from the other arm, [4]'s elbows and lag are met, and the placement's
        own geometry (arm_angles) reads what the stops read."""
        m, st, lim = self.m, self.st, self.lim
        a, c, ar = B.range_angles(m, st), B.clinical(m, st), B.arm_range()
        for sd, sg in B.SIDES:
            for k in B.ARM_STOPS:
                lo, hi = lim[k + "_" + sd]
                self.assertEqual(sorted(B.clinical_sign(k, sg) * x for x in (lo, hi)), list(ar[k]), msg=k + sd)   # the anatomical ends, not moved
                self.assertGreater(a[k + "_" + sd], lo, msg=k + sd)
                self.assertLess(a[k + "_" + sd], hi, msg=k + sd)
            self.assertLess(a["shoulder_" + sd], B.ROM["sh_hadd"] - 90.0)
            self.assertLess(a["shoulder_back_" + sd], B.ROM["sh_habd"])
            self.assertEqual(lim["shoulder_up_" + sd][0], 90.0 - B.ROM["sh_raise"])      # the raise's end, not moved
            self.assertLess(c[("raised", sd)], B.ROM["sh_raise"] - 30.0)                 # and the start well inside it (the rear arm 132 deg)
            zu, zf = m.axis(st, "uarm_" + sd, (0, 0, 1)), m.axis(st, "farm_" + sd, (0, 0, 1))
            g = B.arm_angles(sd, zu, zf, E.qR(m.sys.pose(st, m.b["bat"])[1]), E.qR(m.sys.pose(st, m.b["trunk"])[1]))
            for k in B.ARM_STOPS:
                self.assertAlmostEqual(g[k], c[(k, sd)], delta=1e-6, msg=k + sd)
            other = "R" if sd == "L" else "L"
            el, S, El, W = m.joint_pos(st, "elbow_" + sd), m.joint_pos(st, "shoulder_" + other), m.joint_pos(st, "elbow_" + other), m.joint_pos(st, "wrist_" + other)
            self.assertGreaterEqual(min(B._seg_dist(el, S, El), B._seg_dist(el, El, W)), B.ARM_CLEAR - 1e-3)
        o = m.observe(st)
        for k in ("lead_elbow_flex", "trail_elbow_flex", "bat_lag", "bat_az", "bat_elev"):
            self.assertAlmostEqual(o[k] % 360.0, B.curve(k, T0) % 360.0, delta=0.6, msg=k)
        m.sys.diag = {}
        m.sys.deriv(T0, st, m.sys.new_aux())
        self.assertTrue(all(tau == 0.0 for tau, th in m.sys.diag["stop"].values()))
        m.sys.diag = None

    def test_a_hitters_contact_posture_lies_inside_every_arm_range(self):
        """a posture a hitter is measured in at contact can be taken without leaving any arm range (for checking; nothing
        here is given to a search): the sweet spot at the aim's ball (body-motion 4.10: 0.92 m toward the plate and
        0.72 m toward the pitcher from the hips at the start, 0.78 m up), the bat within 10 deg of square to the pitch
        and its barrel below the hands, the lead elbow bent 53 +- 15 deg and the rear 79 +- 13 (OpenBiomechanics), the
        trunk turned to the pitcher and leaning over the plate. Among such postures (the trunk's lean, the bat's roll
        and the elbows' swivels free) at least one has every wrist, forearm and upper-arm angle inside its range and
        the elbows clear of each other."""
        m, st = self.m, self.st
        s = m.sys
        lu, lf = m.len["upperarm"], m.len["forearm"]
        xp = s.pose(st, m.b["pelvis"])[0]
        ball = (xp[0] + 0.92, xp[1] + 0.72, 0.78)
        up = (0.0, 0.0, 1.0)
        r = math.radians
        found = []
        for yaw, fwd, side, shift in ((90.0, 40.0, 30.0, 0.20), (90.0, 40.0, 30.0, 0.10), (75.0, 25.0, 30.0, 0.20), (90.0, 25.0, 15.0, 0.20)):
            lum = E.add((xp[0], xp[1] + shift, xp[2] - 0.05), E.mv(E.qR(E.qaxis(up, r(yaw - 15.0))), m.jp["lumbar"][1]))
            Rt = E.qR(E.qmul(E.qmul(E.qaxis(up, r(yaw)), E.qaxis((0, 1, 0), r(fwd))), E.qaxis((1, 0, 0), r(side))))
            xt = E.sub(lum, E.mv(Rt, m.jp["lumbar"][3]))
            sho = {sd: E.add(xt, E.mv(Rt, m.jp["shoulder_" + sd][1])) for sd in "LR"}
            for az in (-10.0, 0.0, 10.0):
                for el in (-10.0, -20.0, -30.0):
                    bz = (math.cos(r(el)) * math.cos(r(az)), math.cos(r(el)) * math.sin(r(az)), math.sin(r(el)))
                    knob = E.sub(ball, E.scale(bz, B.C.BAT_SS))
                    bx0 = E.unit(E.cross(E.cross(bz, up), bz))
                    for roll in range(-180, 180, 20):
                        bx = E.mv(E.qR(E.qaxis(bz, r(roll))), bx0)
                        Rb = E.qR(E.axes_to_q(bx, E.cross(bz, bx), bz))
                        best, elbows = {}, {}
                        for sd, sg in B.SIDES:
                            w = E.add(E.add(knob, E.scale(bz, m.bat_ucm)), E.mv(Rb, m.wrist_b[sd]))
                            if E.norm(E.sub(w, sho[sd])) > 0.999 * (lu + lf):
                                break
                            dh = E.unit(E.sub(w, sho[sd]))
                            ref = E.sub((0.0, 0.0, -1.0), E.scale(dh, -dh[2]))
                            for sw in range(-180, 180, 20):
                                elb = B._two_link(sho[sd], w, lu, lf, E.mv(E.qR(E.qaxis(dh, r(sw))), ref))
                                zu, zf = E.unit(E.sub(sho[sd], elb)), E.unit(E.sub(elb, w))
                                bend = math.degrees(math.acos(max(-1.0, min(1.0, E.dot(zu, zf)))))
                                if abs(bend - (53.0 if sd == "L" else 79.0)) > (15.0 if sd == "L" else 13.0):
                                    continue
                                c = B.strain(B.arm_angles(sd, zu, zf, Rb, Rt))
                                if sd not in best or c < best[sd]:
                                    best[sd], elbows[sd] = c, (elb, w)
                        if len(best) == 2 and max(best.values()) < 1.0:
                            (eL, wL), (eR, wR) = elbows["L"], elbows["R"]
                            if min(B._seg_dist(eL, sho["R"], eR), B._seg_dist(eL, eR, wR), B._seg_dist(eR, sho["L"], eL), B._seg_dist(eR, eL, wL)) >= B.ARM_CLEAR:
                                found.append((max(best.values()), yaw, fwd, side, shift, az, el, roll))
        self.assertTrue(found, msg="no hitter-like contact posture inside every arm range")
        self.assertLess(min(found)[0], 0.9)                 # and not only just: the best is a tenth of a half-range inside

    def test_each_wrists_drive_tilts_it_away_from_the_thumb_and_lets_the_bat_out(self):
        """the body driven by its two "wrist" muscles alone for 40 ms: each wrist's tilt moves away from the thumb (the
        reading falls), the lag falls (the direction a swing uncocks), and the lead forearm-to-barrel angle opens. (The
        rear wrist starts bent back 74 deg, where the angle between forearm and barrel hardly depends on the tilt:
        cos(angle) = sin(tilt - obliqueness) x cos(bend); so that angle is asked of the lead wrist only.)"""
        m = B.Model(ranges=(), grip="hands")
        st = B.place(m, T0)
        bd = Dr.bounds(m.H, m.M)
        st = Dr.add_drives(m, st, {n: (bd[n], T0, 0.05, 1.0) for n in ("wrist_L", "wrist_R")})
        B.add_ranges(m, st)
        Dr.solve_tone(m, st, T0, aux=list(m.start_aux))
        Dr.add_damping(m, st)
        aux = list(m.start_aux)
        m.sys.sec_start(st, aux)
        c0, lag0 = B.clinical(m, st), m.observe(st)["bat_lag"]
        s1 = list(st)
        for i in range(40):
            s1 = m.sys.step(T0 + i * 1e-3, s1, aux, 1e-3)
        c1, lag1 = B.clinical(m, s1), m.observe(s1)["bat_lag"]
        self.assertLess(lag1, lag0 - 1.0)
        for sd in "LR":
            self.assertLess(c1[("wrist_dev", sd)], c0[("wrist_dev", sd)] - 0.5, msg=sd)
        self.assertGreater(c1[("forearm_to_barrel", "L")], c0[("forearm_to_barrel", "L")] + 0.3)

    def test_the_arm_stops_take_and_never_give_when_the_body_is_driven_into_them(self):
        """the passivity check on the whole hands' body: eight arm muscles that drive joints toward the ends of their
        ranges held at their bounds for 100 ms. At least four of the new stops engage, each only ever pushes back, the
        stops' net work is not positive, and the energy account closes (energy gained = work booked). Run at a
        0.25 ms step: muscles at full strength against stiff stops do not integrate at 1 ms (the account is then
        hundreds of joules out, which is how the step was found too coarse; body-motion 4.10 has the same for swings)."""
        m = B.Model(ranges=(), grip="hands")
        st = B.place(m, T0)
        bd = Dr.bounds(m.H, m.M)
        push = ("wrist_L", "wrist_R", "wrist_ext_L", "wrist_ext_R", "pron_L", "sup_R", "sh_irot_L", "sh_erot_R")
        st = Dr.add_drives(m, st, {n: (bd[n], T0, 0.05, 1.0) for n in push})
        lim = B.add_ranges(m, st)
        Dr.add_damping(m, st)
        aux = list(m.start_aux)
        m.sys.sec_start(st, aux)
        e0 = kinetic(m.sys, st) + potential(m.sys, st)
        s1, peak = list(st), {}
        m.sys.diag = {}
        h = 2.5e-4
        for i in range(400):
            s1 = m.sys.step(T0 + i * h, s1, aux, h)
            if i % 4 != 3:
                continue
            m.sys.deriv(T0 + (i + 1) * h, s1, aux)
            for k, (tau, th) in m.sys.diag["stop"].items():
                lo, hi = lim[k]
                if tau != 0.0:
                    self.assertTrue((tau < 0.0 and math.degrees(th) > hi) or (tau > 0.0 and math.degrees(th) < lo), msg=k)
                    peak[k] = max(peak.get(k, 0.0), abs(tau))
        m.sys.diag = None
        arm = [k for k in peak if k.startswith(B.ARM_STOPS)]
        self.assertGreaterEqual(len(arm), 4, msg=str(peak))
        ow = 13 * m.sys.nb + len(m.sys.acts)
        self.assertLess(s1[ow + 1], 1e-6)                    # the stops' net work
        e1 = kinetic(m.sys, s1) + potential(m.sys, s1)
        self.assertAlmostEqual(e1 - e0, work(m.sys, s1), delta=0.03 * (abs(s1[ow]) + abs(s1[ow + 2]) + 1.0))   # 3 % of the gross work booked


def held_rod(T=40.0, a0=0.6, t_on=0.020, held=True, t_end=0.40, h=1e-3):
    """a heavy rod on a hinge, no gravity, one drive about the hinge tensed at a0 before its onset at t_on (rise 50 ms),
    held or not. Returns [(t, torque on the joint, joint speed, activation)] after every step."""
    s = E.System(g=(0.0, 0.0, 0.0))
    b = s.body("rod", 30.0, (4.0, 4.0, 0.5))
    s.hinge(-1, b, (0, 0, 0), (0, 0, 0.5), (0, 1, 0), (0, 1, 0))
    s.actuator("m", -1, b, (0, 1, 0), T, 20.0, t_on, 0.05, 1e9, a0, held=held)
    st = s.new_state()
    s.set_body(st, b, (0.0, 0.0, -0.5), (1.0, 0.0, 0.0, 0.0))
    st[13 * s.nb] = a0
    aux = s.new_aux()
    s.diag = {}
    out = []
    for i in range(int(round(t_end / h))):
        st = s.step(i * h, st, aux, h)
        s.deriv((i + 1) * h, st, aux)
        out.append(((i + 1) * h,) + tuple(s.diag["act"]["m"]))
    return out, st, s


class HeldChecks(unittest.TestCase):
    """a muscle tensed before the pitch only holds (body-motion 4.11, part 2)."""

    def test_a_held_muscle_gives_nothing_before_its_onset_then_rises_without_a_step_to_its_whole_torque(self):
        """the rod's drive, tensed at 0.6 and held: no torque and no motion before its onset; from the onset the torque
        rises with no step larger than a twentieth of its strength per millisecond; a quarter of a second later it is
        the whole torque of the same muscle un-held (strength x activation x the speed curve, computed here). Not held,
        the same muscle pushes from the first instant."""
        T = 40.0
        rows, st, s = held_rod(T=T)
        before = [r for r in rows if r[0] < 0.020 - 1e-9]
        self.assertTrue(all(r[1] == 0.0 and r[2] == 0.0 for r in before))
        taus = [0.0] + [r[1] for r in rows]
        self.assertLess(max(abs(b_ - a_) for a_, b_ in zip(taus, taus[1:])), 0.05 * T)
        self.assertGreater(max(taus), 0.5 * T)
        t, tau, w, act = rows[-1]
        self.assertGreater(t, 0.020 + 5 * E.TA_REL)
        self.assertAlmostEqual(tau, T * act * E.fv(w / 20.0), delta=0.01 * T)
        loose, _, _ = held_rod(T=T, held=False)
        self.assertAlmostEqual(loose[0][1], T * 0.6, delta=0.02 * T)

    def test_a_hold_that_never_lets_go_fails_that_check(self):
        """the mutation, and session 8's dead end (the balancing torque left on all swing): the muscle never gets its
        whole torque."""
        E.MUT.add("hold_stays")
        try:
            rows, st, s = held_rod()
        finally:
            E.MUT.discard("hold_stays")
        t, tau, w, act = rows[-1]
        self.assertLess(tau, 0.6 * 40.0 * act * E.fv(w / 20.0))

    def test_the_holds_share_is_zero_to_the_onset_never_negative_and_ends_at_one(self):
        """hold_share over time for a muscle tensed at 0.7: 0 before the onset whatever its activation is doing below
        the level; between 0 and 1 and never falling while the activation rises; within 1 % of 1 after five time
        constants; and 0, not negative, for a muscle switched off under its hold."""
        f = E.System.hold_share
        self.assertEqual(f(0.0, 0.1, 0.7, 0.7), 0.0)
        self.assertEqual(f(0.05, 0.1, 0.7, 0.3), 0.0)
        prev = 0.0
        for k in range(1, 400):
            t = 0.1 + k * 1e-3
            act = min(1.0, 0.7 + 0.3 * k / 60.0)
            x = f(t, 0.1, 0.7, act)
            self.assertGreaterEqual(x, prev - 1e-12)
            self.assertLessEqual(x, 1.0)
            prev = x
        self.assertGreater(prev, 0.99)
        self.assertEqual(f(0.11, 0.1, 0.7, 0.05), 0.0)
        self.assertEqual(f(0.5, 0.1, 0.0, 0.3), 1.0)             # nothing tensed: nothing held

    @NEED
    def test_a_body_with_every_muscle_tensed_and_held_moves_as_the_body_with_none_tensed_until_the_first_onset(self):
        """the hands' body under gravity, its hold-still torques and damping, every one of its 41 muscles at its bound
        with a late onset: tensed at 0.7 and held, its segments' states are the untensed body's, entry for entry, for
        the 55 ms before the first onset, while the body moves (not only at the first instant), and the drives have
        booked no work. Tensed and not held it is elsewhere within 5 ms."""
        on = T0 + 0.060
        runs = {}
        for tag, level, held in (("none", 0.0, False), ("held", 0.7, True), ("loose", 0.7, False)):
            m = B.Model(ranges=(), grip="hands")
            st = B.place(m, T0)
            bd = Dr.bounds(m.H, m.M)
            # "none": no muscle at all (strength 0). A muscle at its bound with activation 0 is not quite nothing: its
            # tendon spring is worked as if a thousandth active (engine.MUT_A_MIN), up to 0.15 % of its strength.
            st = Dr.add_drives(m, st, {n: (bd[n] if level else 0.0, on, 0.05, 1.0, 0.0) for n in bd}, held=held)
            B.add_ranges(m, st)
            Dr.solve_tone(m, st, T0, aux=list(m.start_aux))
            Dr.add_damping(m, st)
            oa = 13 * m.sys.nb
            for j, a in enumerate(m.sys.acts):
                a[9] = level
                st[oa + j] = level
            aux = list(m.start_aux)
            m.sys.sec_start(st, aux)
            s1, seq = list(st), []
            for i in range(55):
                s1 = m.sys.step(T0 + i * 1e-3, s1, aux, 1e-3)
                seq.append(s1[:oa])
            runs[tag] = (seq, s1, m)
        moved = max(abs(a - b) for a, b in zip(runs["none"][0][-1], B.place(B.Model(ranges=(), grip="hands"), T0)[:13 * runs["none"][2].sys.nb]))
        self.assertGreater(moved, 1e-3)                              # the body did move in those 55 ms
        for a_, b_ in zip(runs["held"][0], runs["none"][0]):
            self.assertLess(max(abs(x - y) for x, y in zip(a_, b_)), 1e-12)
        seq, s1, m = runs["held"]
        self.assertEqual(s1[13 * m.sys.nb + len(m.sys.acts)], 0.0)   # the drives' booked work
        self.assertGreater(max(abs(x - y) for x, y in zip(runs["loose"][0][4], runs["none"][0][4])), 1e-3)

    @NEED
    def test_the_energy_account_closes_with_held_muscles_firing(self):
        """every channel of the driven test body tensed at 0.5 and held, firing 10 ms after the start: the energy gained
        equals the work booked, as for the un-held body (test_work_against_energy), over 80 ms."""
        m = B.Model()
        st = B.place(m, T0)
        bd = Dr.bounds(m.H, m.M)
        st = Dr.add_drives(m, st, {n: (bd[n] * k, T0 + 0.010, 0.05, 1.0, 0.5) for n, k in MODEST.items()}, held=True)
        Dr.add_damping(m, st)
        aux = m.sys.new_aux()
        m.sys.anchors(st, aux)
        nc = len(m.sys.contacts)
        tend = lambda: sum(0.5 * a[11] * aux[nc + j] ** 2 for j, a in enumerate(m.sys.acts))
        m.sys.sec_start(st, aux)
        e0 = kinetic(m.sys, st) + potential(m.sys, st)
        s1 = m.sys.run(list(st), T0, T0 + 0.080, 2.5e-4, aux=aux)
        e1 = kinetic(m.sys, s1) + potential(m.sys, s1)
        gross = sum(abs(w) for w in s1[-E.NW:])
        self.assertGreater(abs(s1[-E.NW]), 5.0)                      # the drives did work (J)
        self.assertAlmostEqual(e1 - e0, work(m.sys, s1), delta=0.03 * gross)


def forefoot_rig(load=1190.0, half_w=None, mu=None, h=2.5e-4):
    """Brosnan, McNitt & Serensits' rig in the engine (research note 12 D5): a shoe's forefoot alone on the ground under
    `load` N, kept level, twisted about the vertical by a moment that rises 400 N m a second until the forefoot has
    turned 40 deg. The forefoot's two stud points are BALL_HALF_W either side of its line; the grip MU_CLEAT; the
    contact law and its stiffness are the body's own. Returns (the largest twisting moment the ground gave, N m, read
    here from the two points' forces; the angle turned, deg; the ground's booked work, J; the largest the two normal
    forces summed to, N)."""
    half_w = B.BALL_HALF_W if half_w is None else half_w
    mu = B.MU_CLEAT if mu is None else mu
    s = E.System(g=(0.0, 0.0, 0.0))
    b = s.body("forefoot", 1.3, (0.002, 0.006, 0.006))
    s.dotc(-1, b, (1, 0, 0), (0, 0, 1))                      # kept level: only sliding and turning about the vertical are free
    s.dotc(-1, b, (0, 1, 0), (0, 0, 1))
    pts = [(0.0, half_w, -0.02), (0.0, -half_w, -0.02)]
    for k, p in enumerate(pts):
        s.contact("stud%d" % k, b, p, B.KN, B.CN, B.KT, B.CT, mu)
    st = s.new_state()
    s.set_body(st, b, (0.0, 0.0, 0.02 - load / (2 * B.KN)), (1.0, 0.0, 0.0, 0.0))
    aux = s.new_aux()
    s.anchors(st, aux)
    s.diag = {}
    peak, nmax, t, mz = 0.0, 0.0, 0.0, 0.0
    yaw = lambda state: math.degrees(2.0 * math.atan2(state[6], state[3]))
    while yaw(st) < 40.0 and t < 1.5:
        mz = min(400.0 * max(0.0, t - 0.05), 1.05 * mu * load * half_w + 3.0)     # after it lets go: a little over what it held
        s.ext = [(b, (0.0, 0.0, 0.0), (0.0, 0.0, -load), (0.0, 0.0, mz))]
        st = s.step(t, st, aux, h)
        t += h
        s.deriv(t, st, aux)
        x, q, _, _ = s.pose(st, b)
        R = E.qR(q)
        tw = sum(E.cross(E.mv(R, p), s.diag["contact"]["stud%d" % k])[2] for k, p in enumerate(pts))
        peak = max(peak, abs(tw))
        nmax = max(nmax, sum(s.diag["contact"]["stud%d" % k][2] for k in range(2)))
    s.diag = None
    return peak, yaw(st), st[13 * s.nb + len(s.acts) + 3], nmax


def clinical_ankle(m, st, sd, down=0.0, inward=0.0):
    """the state st with foot sd rebuilt on its shank from the clinical definitions (orientation only): square to the
    shin, then pointed `down` (deg; negative is the shin forward over the foot) about the shin's side-to-side axis, then
    its sole rolled `inward` (toward the body's midline) about the foot's own length. Returns (state, the sole's normal)."""
    sg = 1.0 if sd == "L" else -1.0
    qs = tuple(st[13 * m.b["shank_" + sd] + 3:13 * m.b["shank_" + sd] + 7])
    q = E.qmul(qs, E.qaxis((0, 1, 0), math.radians(down)))
    length = E.mv(E.qR(q), (1, 0, 0))
    medial = E.scale(E.mv(E.qR(qs), (0, 1, 0)), -sg)
    for sign in (1.0, -1.0):                                 # whichever sense turns the sole toward the midline
        q2 = E.qmul(E.qaxis(length, sign * math.radians(inward)), q)
        sole = E.scale(E.mv(E.qR(q2), (0, 0, 1)), -1.0)
        if E.dot(sole, medial) * inward >= 0.0:
            break
    out = list(st)
    out[13 * m.b["foot_" + sd] + 3:13 * m.b["foot_" + sd] + 7] = q2
    return out, sole


class RaiseChecks(unittest.TestCase):
    """the end of how far an arm is raised, which keeps the upper arm's turn defined (body-motion 4.11)."""

    def test_an_arm_driven_overhead_meets_its_raises_end_before_its_turn_goes_undefined(self):
        """session 9's first practice runs failed wherever a swing's follow-through took the lead arm straight overhead:
        the upper arm's turn has no value there and the run raised an error, which the search read as a refused step.
        An upper arm alone on a ball joint, with the hands' body's two stops (its turn, kind 2; its raise, 168.8 deg)
        and a 5 N m muscle raising it from 30 deg for 0.6 s: it is stopped within 3 deg of its end, the run never
        fails, and the stops' work is not positive. Without the raise's stop the same run does fail: the fault."""
        def go(with_stop):
            s = E.System(g=(0.0, 0.0, 0.0))
            b = s.body("uarm", 2.5, (0.15, 0.15, 0.004))
            s.ball(-1, b, (0, 0, 0), (0, 0, 0.13))
            lo, hi = sorted(-x for x in B.arm_range()["upper_arm"])
            s.stop("turn", -1, b, (0, 0, 1), (0, -1, 0), (0, 1, 0), math.radians(lo), math.radians(hi), B.RANGE_K["small"], B.RANGE_C["small"], 2)
            if with_stop:
                s.stop("up", -1, b, (0, 0, 1), (0, 0, 1), (0, 0, 1), math.radians(90.0 - B.ROM["sh_raise"]), math.radians(91.0), B.RANGE_K["big"], B.RANGE_C["big"], 1)
            s.actuator("raise", -1, b, (0, 0, 1), 5.0, 1e9, -1.0, 0.0, 1e9, 1.0, axis_b=(0, 0, 1))
            st = s.new_state()
            q = E.qmul(E.qaxis((0.6, 0.8, 0.0), math.radians(30.0)), E.qaxis((0, 0, 1), math.radians(180.0)))
            s.set_body(st, b, E.scale(E.mv(E.qR(q), (0, 0, 0.13)), -1.0), q)
            st[13 * s.nb] = 1.0
            aux = s.new_aux()
            top = 0.0
            for i in range(600):
                st = s.step(i * 1e-3, st, aux, 1e-3)
                zu = E.mv(E.qR(s.pose(st, b)[1]), (0, 0, 1))
                top = max(top, math.degrees(math.acos(max(-1.0, min(1.0, zu[2])))))
            return top, st[13 * s.nb + len(s.acts) + 1]
        top, stops_work = go(True)
        self.assertGreater(top, B.ROM["sh_raise"] - 1.0)
        self.assertLess(top, B.ROM["sh_raise"] + 3.0)
        self.assertLess(stops_work, 1e-6)
        with self.assertRaises(E.SolveError):
            go(False)


def hand_on_an_oval(az_deg, c=0.0, k=2000.0, T=0.25, h=1e-4, drive=0.0, tilt=0.3, speed=10.0):
    """a 1.2 kg forearm-like body on a two-axis joint at the world's origin (the world's z and its own x, held square:
    the wrist's joint, with the world as the hand), no gravity, thrown outward from inside an ellipse of its two angles
    (kind 3, body-motion 4.12: half-axes 0.3 and 0.6 rad, centre (0.05, -0.10), tilted `tilt`) in the direction
    `az_deg` of the plane of the two angles. drive: a constant torque (N m) about the world's z pushing the first angle up.
    The two angles and the distance outside the ellipse are worked out HERE from the body's rotation, by the test's own
    formulas. Returns (energy at the start, at the end, the largest seen, the stop's booked work + what its spring
    holds, the largest difference between the engine's reading and this one while outside, the drive's booked work,
    the deepest it went, the distance at the end)."""
    s = E.System(g=(0.0, 0.0, 0.0))
    I = (0.05, 0.05, 0.004)
    b = s.body("forearm", 1.2, I)
    s.universal(-1, b, (0.0, 0.0, 0.0), (0.0, 0.0, 0.12), (0, 0, 1), (1, 0, 0))
    c1, c2, h1, h2 = 0.05, -0.10, 0.3, 0.6
    s.stop("oval", -1, b, (0, 0, 1), (1, 0, 0), (1, 0, 0), 0.0, 0.0, k, c, 3, oval=((1, 0, 0), (0, 0, 1), (0, 0, 1), c1, c2, h1, h2, tilt))
    if drive:
        s.actuator("push", -1, b, (0, 0, 1), drive, 1e9, -1.0, 0.0, 1e9, 1.0)
    st = s.new_state()
    # the posture with the two angles at the ellipse's centre: turned c1 about the world's z, then -c2 about its own x
    # (the second angle is read about the body's x from the body's z to the world's z, so it falls as the body turns)
    q = E.qmul(E.qaxis((0, 0, 1), c1), E.qaxis((1, 0, 0), -c2))
    R = E.qR(q)
    x = E.scale(E.mv(R, (0, 0, 0.12)), -1.0)
    a = math.radians(az_deg)
    w = E.add(E.scale((0, 0, 1), speed * math.cos(a)), E.scale(E.mv(R, (1, 0, 0)), -speed * math.sin(a)))
    s.set_body(st, b, x, q, E.cross(w, x), w)
    if drive:
        st[13 * s.nb] = 1.0
    aux = s.new_aux()
    ow = 13 * s.nb + len(s.acts)
    ct, sn = math.cos(tilt), math.sin(tilt)

    def reading(state):
        Rb = E.qR(s.pose(state, b)[1])
        bx, bz = E.mv(Rb, (1, 0, 0)), E.mv(Rb, (0, 0, 1))
        t1 = math.atan2(bx[1], bx[0])                              # the body's x about the world's z, from the world's x
        t2 = math.atan2(E.dot(bx, E.cross(bz, (0, 0, 1))), bz[2])  # the world's z about the body's x, from the body's z
        u, v = t1 - c1, t2 - c2
        rho = math.hypot((u * ct + v * sn) / h1, (-u * sn + v * ct) / h2)
        return math.hypot(u, v) * (1.0 - 1.0 / rho) if rho > 1e-9 else -1.0

    def energy(state):
        xx, qq, vv, ww = s.pose(state, b)
        wl = E.mtv(E.qR(qq), ww)
        d = reading(state)
        pe = 0.5 * k * d * d if d > 0 else 0.0
        return 0.5 * 1.2 * E.dot(vv, vv) + 0.5 * sum(I[i] * wl[i] ** 2 for i in range(3)) + pe, pe, d
    e0 = energy(st)[0]
    emax, worst, deep = e0, 0.0, 0.0
    s.diag = {}
    for i in range(int(round(T / h))):
        st = s.step(i * h, st, aux, h)
        if i % 5 == 0:
            e, pe, d = energy(st)
            emax, deep = max(emax, e), max(deep, d)
            s.deriv((i + 1) * h, st, aux)
            if d > 0:
                worst = max(worst, abs(s.diag["stop"]["oval"][1] - d))
    s.diag = None
    e1, pe1, d1 = energy(st)
    return e0, e1, emax, st[ow + 1] + pe1, worst, st[ow], deep, d1


class OvalChecks(unittest.TestCase):
    """the wrist's oblique range (body-motion 4.12, session 10): the ellipse, the stop that reads both of the wrist's
    angles (engine kind 3), its passivity, and the cleared start."""

    def test_the_wrists_ellipse_passes_through_the_four_measured_ends(self):
        """worked by hand for the untilted ellipse: centred on the box's centre, its two half-axes from the two
        conditions that it pass through the ends on each axis (a 2 x 2 linear solve for 1/a^2 and 1/b^2); the body's
        ellipse is that one; each measured end lies on it; the corner of the box where session 9's rear wrist started
        is 1.40 times its radius; tilted 8 deg it still passes through the four ends; tilted 25 it does not exist."""
        ar = B.arm_range()
        (f2, f1), (d2, d1) = ar["wrist_flex"], ar["wrist_dev"]
        cx, cy, hx, hy = 0.5 * (f1 + f2), 0.5 * (d1 + d2), 0.5 * (f1 - f2), 0.5 * (d1 - d2)
        # hx^2 A + cy^2 B = 1 and cx^2 A + hy^2 B = 1
        det = hx * hx * hy * hy - cy * cy * cx * cx
        A, Bq = (hy * hy - cy * cy) / det, (hx * hx - cx * cx) / det
        ov = B.wrist_oval()
        for got, want in zip(ov, (cx, cy, 1.0 / math.sqrt(A), 1.0 / math.sqrt(Bq), 0.0)):
            self.assertAlmostEqual(got, want, delta=1e-9)
        for tilt in (0.0, 8.0):
            for f, d in ((f1, 0.0), (f2, 0.0), (0.0, d1), (0.0, d2)):
                rho, inside = B.oval_place(f, d, tilt=tilt)
                self.assertAlmostEqual(rho, 1.0, delta=1e-9)
                self.assertAlmostEqual(inside, 0.0, delta=1e-7)
        self.assertLess(B.oval_place(0.0, 0.0)[0], 0.25)
        self.assertAlmostEqual(B.oval_place(f2, d2)[0], 1.40, delta=0.01)          # bent right back and tilted right away from the thumb
        self.assertGreater(B.oval_place(f2, d2, tilt=8.0)[0], 1.6)                 # tilted, that corner is further outside
        with self.assertRaises(ValueError):
            B.wrist_oval(tilt=25.0)

    def test_the_ellipses_reading_is_the_distance_outside_and_does_not_move_with_the_arms_other_joints(self):
        """the body's reading of each wrist's ellipse (range_angles, as the engine's stop reads it) for a wrist posed
        from the clinical definitions (clinical_arm: rotations only) equals the distance outside the ellipse worked
        from the posed angles, inside and outside, in the four quadrants, on either hand; and it is the same whatever
        the forearm's turn, the elbow and the upper arm do."""
        if not B.have_curves():
            self.skipTest("[4]'s digitised curves are not on this machine")
        m = B.Model(ranges=(), grip="hands", wrist="oval")
        st = B.place(m, T0)
        for sd, sg in B.SIDES:
            for tilt, bend in ((0.0, 0.0), (15.0, 30.0), (-30.0, -70.0), (20.0, -74.0), (-31.0, 60.0), (10.0, 50.0), (-20.0, -40.0)):
                want = -B.oval_place(bend, tilt)[1]
                for kw in ({}, {"palm_down": 60.0, "elbow": 30.0}, {"raise_deg": sg * 70.0, "turn_in": -40.0, "palm_down": -50.0, "elbow": 120.0}):
                    got = B.range_angles(m, clinical_arm(m, st, sd, tilt=tilt, bend=bend, **kw)[0])["wrist_oval_" + sd]
                    self.assertAlmostEqual(got, want, delta=1e-7, msg="%s tilt %.0f bend %.0f %s" % (sd, tilt, bend, kw))

    def test_the_ellipses_gradient_is_its_readings_rate(self):
        """the kind 3 stop's gradient g against finite differences: on the reading's two partial derivatives (the pure
        function), and on the whole stop, a forearm turned a little about each of the wrist's two axes and about an
        oblique one: the reading's rate is w . g."""
        S = E.System
        ov = (0.05, -0.10, 0.3, 0.6, math.cos(0.3), math.sin(0.3))
        for t1, t2 in ((0.5, 0.2), (-0.4, 0.5), (0.1, -0.9), (-0.3, -0.6), (0.45, -0.1)):
            d, d1, d2 = S.oval_reading(t1, t2, *ov)
            self.assertGreater(d, 0.0)
            e = 1e-6
            self.assertAlmostEqual(d1, (S.oval_reading(t1 + e, t2, *ov)[0] - S.oval_reading(t1 - e, t2, *ov)[0]) / (2 * e), delta=1e-7)
            self.assertAlmostEqual(d2, (S.oval_reading(t1, t2 + e, *ov)[0] - S.oval_reading(t1, t2 - e, *ov)[0]) / (2 * e), delta=1e-7)
        s = S()
        s.ovals["o"] = ((1.0, 0.0, 0.0), (0.0, 0.0, 1.0), (0.0, 0.0, 1.0)) + ov
        Ra = E.qR(E.qaxis((0.3, -0.5, 0.8), 0.7))                    # the hand, anyhow in space
        n, e_ = E.mv(Ra, (0, 0, 1)), E.mv(Ra, (1, 0, 0))
        for a1, a2 in ((0.5, 0.3), (-0.5, 0.6), (0.2, -0.9)):
            qb = E.qmul(E.R_to_q(Ra), E.qmul(E.qaxis((0, 0, 1), a1), E.qaxis((1, 0, 0), a2)))
            rd = lambda q: s.oval_angle("o", Ra, E.qR(q), n, e_, E.mv(E.qR(q), (1, 0, 0)))
            d, g, t1, t2 = rd(qb)
            self.assertGreater(d, 0.0)
            for w in (n, E.mv(E.qR(qb), (1, 0, 0)), E.add(E.scale(n, 0.6), E.scale(E.mv(E.qR(qb), (1, 0, 0)), -0.8))):
                dl = 1e-6
                rate = (rd(E.qmul(E.qaxis(w, E.norm(w) * dl), qb))[0] - rd(E.qmul(E.qaxis(w, -E.norm(w) * dl), qb))[0]) / (2 * dl)
                self.assertAlmostEqual(rate, E.dot(w, g), delta=1e-6 * (1.0 + abs(rate)))

    def test_the_ellipses_stop_does_no_net_positive_work(self):
        """the passivity check for the kind 3 stop, before any search uses it: a body on the wrist's two-axis joint
        thrown into the ellipse in eight directions. Elastic, its energy comes back (to within an error that shrinks
        with the step) and never exceeds the start's; damped, it loses energy; driven in by a 15 N m torque it never has
        more than it started with plus the drive's work; and the engine's reading is the distance worked out here."""
        for az in (0.0, 50.0, 90.0, 140.0, 180.0, 235.0, 270.0, 320.0):
            e0, e1, emax, booked, worst, wd, deep, d1 = hand_on_an_oval(az)
            msg = "direction %.0f" % az
            self.assertGreater(deep, 0.01, msg=msg)                        # it did go into the stop
            self.assertLess(worst, 1e-9, msg=msg)
            self.assertLess(emax, e0 * (1.0 + 1e-3), msg=msg)
            self.assertLess(booked, 1e-3 * e0, msg=msg)
            self.assertAlmostEqual(e1, e0, delta=2e-3 * e0, msg=msg)
            e0, e1, emax, booked, worst, wd, deep, d1 = hand_on_an_oval(az, c=20.0)
            self.assertLess(emax, e0 * (1.0 + 1e-3), msg=msg)
            self.assertLess(e1, 0.9 * e0, msg=msg)
            e0, e1, emax, booked, worst, wd, deep, d1 = hand_on_an_oval(az, c=20.0, drive=15.0)
            self.assertLess(e1, e0 + wd + 1e-3 * e0, msg=msg)
            self.assertLess(booked, 1e-3 * e0, msg=msg)
        # the elastic run's error takes either sign from one direction to the next and shrinks with the step (over the
        # eight directions 8.0e-4 of the energy at 0.1 ms, 2.6e-5 at 0.025 ms): the integrator at the stop's edge, not the law
        err = lambda h: sum(abs(r[1] - r[0]) / r[0] for r in (hand_on_an_oval(az, h=h) for az in (0.0, 50.0, 90.0, 140.0, 180.0, 235.0, 270.0, 320.0)))
        self.assertLess(err(2.5e-5), 0.25 * err(1e-4))

    def test_the_ellipses_stop_with_the_torque_along_one_angle_alone_fails_that_check(self):
        """the mutation: the stop's torque along the first angle's gradient only. Thrown obliquely into the ellipse the
        body ends with hundreds of times the energy it started with."""
        E.MUT.add("oval_one_angle")
        try:
            bad = [hand_on_an_oval(az) for az in (50.0, 140.0, 235.0, 320.0)]
        finally:
            E.MUT.discard("oval_one_angle")
        self.assertGreater(min((r[1] - r[0]) / r[0] for r in bad), 10.0)


class StartChecks(unittest.TestCase):
    """the cleared start (body-motion 4.12, part 1): [4]'s launch posture moved by a stated rule so that no joint
    starts at an end."""

    @classmethod
    def setUpClass(cls):
        if not B.have_curves():
            raise unittest.SkipTest("[4]'s digitised curves are not on this machine (docs/body-motion.md 3)")
        cls.kw = dict(grip="hands", feet="cleats", wrist="oval")
        cls.m = B.Model(ranges=(), shift=B.START_CLEAR, **cls.kw)
        cls.st = B.place(cls.m, T0)

    def margins(self, shift):
        m = B.Model(ranges=(), shift=shift or None, **self.kw)
        return B.start_margins(m, B.pose(m, T0)[0])

    def test_every_joint_starts_five_degrees_inside_its_end_and_no_stop_is_moved(self):
        """every range the body holds, each side: the start is START_CLEAR_DEG inside its anatomical end (the wrists
        that far inside their ellipses); add_ranges() puts every stop at its anatomical end and none carries anything;
        both wrists are inside the ellipse and the elbows are clear of the other arm. Session 9's start fails this at
        five joints (six readings on its own body, where the rear wrist's two angles were held apart; five here, where
        they are one ellipse), and on its own free choices its rear wrist is outside the ellipse."""
        m, st = self.m, self.st
        mg = B.start_margins(m, st)
        self.assertEqual(len(mg), 3 + 2 * (5 + 2 + 5 + 1))          # the trunk's 3; a side: hip 3, knee, ankle 2 | elbow, shoulder 3, forearm, upper arm | the wrist's ellipse
        for k, v in mg.items():
            self.assertGreaterEqual(v, B.START_CLEAR_DEG - 0.02, msg=k)
        lim = B.add_ranges(m, st)
        want = {t[0]: (t[6], t[7]) for t in B.range_specs(None, "hands", "cleats", "oval") if t[8] is not None and t[9] != 3}
        for k, (lo, hi) in want.items():
            self.assertEqual(lim[k], (lo, hi), msg=k)                 # the anatomical ends, none moved to clear the start
        self.assertAlmostEqual(lim["shoulder_L"][1], B.ROM["sh_hadd"] - 90.0)
        m.sys.diag = {}
        m.sys.deriv(T0, st, m.sys.new_aux())
        self.assertTrue(all(tau == 0.0 for tau, th in m.sys.diag["stop"].values()))
        m.sys.diag = None
        cl = B.clinical(m, st)
        for sd in "LR":
            self.assertLess(B.oval_place(cl[("wrist_flex", sd)], cl[("wrist_dev", sd)])[0], 0.95, msg=sd)
            other = "R" if sd == "L" else "L"
            el, S, El, W = m.joint_pos(st, "elbow_" + sd), m.joint_pos(st, "shoulder_" + other), m.joint_pos(st, "elbow_" + other), m.joint_pos(st, "wrist_" + other)
            self.assertGreaterEqual(min(B._seg_dist(el, S, El), B._seg_dist(el, El, W)), B.ARM_CLEAR - 1e-3)
        old = self.margins(None)
        self.assertEqual(sorted(k for k, v in old.items() if v < B.START_CLEAR_DEG),
                         ["ankle_roll_L", "hip_abd_L", "shoulder_L", "upper_arm_L", "wrist_oval_R"])
        m9 = B.Model(ranges=(), grip="hands", feet="cleats")          # session 9's own start (the wrists' box, its free choices by the box)
        c9 = B.clinical(m9, B.pose(m9, T0)[0])
        self.assertGreater(B.oval_place(c9[("wrist_flex", "R")], c9[("wrist_dev", "R")])[0], 1.3)

    def test_the_cleared_start_is_the_least_change_the_rule_finds(self):
        """the stored change against the rule's own conditions, without rerunning its search: no number is changed by
        more than 2.5 (deg or cm); and it is A least change, one number at a time: each changed number moved an eighth
        back toward [4]'s value leaves some joint short of 5 deg, so no smaller change of that number alone would do.
        That is a condition number by number, as the rule's compass search is; a smaller change of several numbers
        together is not ruled out by it. (The whole rule is rerun, 4 minutes, with BB_SLOW=1: it must return the stored
        change.)"""
        x = B.START_CLEAR
        self.assertGreater(min(self.margins(x).values()), B.START_CLEAR_DEG - 0.005)
        self.assertLessEqual(max(abs(v) for v in x.values()), 2.5)
        self.assertLess(math.sqrt(sum(v * v for v in x.values())), 5.2)
        for k, v in x.items():
            back = dict(x)
            back[k] = v - math.copysign(0.125, v)
            self.assertLess(min(self.margins(back).values()), B.START_CLEAR_DEG - 0.01, msg=k)      # the stored start's least margin is 4.998
        import os
        if os.environ.get("BB_SLOW"):
            got, mg = B.clear_start(**self.kw)
            self.assertEqual(got, x)

    def test_the_start_is_still_the_launch_posture_and_its_slopes(self):
        """what the rule leaves alone: the placed posture reads [4]'s numbers plus the stored changes (the pelvis's and
        the trunk's turn, the bat's direction, the lag, both elbows), and the starting velocities are the slopes of
        [4]'s curves as before (a constant change does not alter a slope): the pelvis's, the trunk's and the bat's
        rates equal the unchanged start's within what the changed posture moves them."""
        o = self.m.observe(self.st)
        m0 = B.Model(ranges=(), grip="hands", feet="cleats")
        o0 = m0.observe(B.place(m0, T0))
        for k in ("pelvis_rot", "trunk_rot", "bat_az", "bat_elev", "bat_lag", "lead_elbow_flex", "trail_elbow_flex"):
            self.assertAlmostEqual((o[k] - B.curve(k, T0) - B.START_CLEAR.get(k, 0.0) + 180.0) % 360.0 - 180.0, 0.0, delta=0.6, msg=k)
        for k, tol in (("pelvis_rot_vel", 5.0), ("trunk_rot_vel", 5.0), ("bat_az_vel", 25.0)):
            self.assertAlmostEqual(o[k], o0[k], delta=tol, msg=k)


class FeetChecks(unittest.TestCase):
    """the feet on dirt (body-motion 4.11, part 3)."""

    def test_the_forefoots_twisting_resistance_is_a_cleats_on_dirt(self):
        """the forefoot in Brosnan's rig (a baseball shoe on basepath dirt, 1,190 N on the forefoot, turned through 40 deg:
        68.5-100.1 N m over three compactions and two dates): the engine's forefoot gives 68 N m, the low end of what was
        measured, because nothing ahead of the ball carries load (the toes have no flexor, body-motion 4.11); it is what
        Coulomb's law gives two points (grip x load x half-width, computed here); the ground takes work and gives none;
        and with the old grip of 0.8 and studs 4 cm either side it gave 38, about half of the least measured."""
        peak, turned, work_, nmax = forefoot_rig()
        self.assertGreaterEqual(turned, 40.0)
        self.assertAlmostEqual(nmax, 1190.0, delta=25.0)
        self.assertAlmostEqual(peak, B.MU_CLEAT * 1190.0 * B.BALL_HALF_W, delta=0.04 * peak)
        self.assertGreaterEqual(peak, 0.97 * 68.5)
        self.assertLessEqual(peak, 100.1)
        self.assertLess(work_, -10.0)                        # it slid 40 deg against about 68 N m: the ground took the work
        old = forefoot_rig(half_w=B.SOLE_HALF_W, mu=B.MU)[0]
        self.assertLess(old, 0.6 * 68.5)

    @NEED
    def test_the_ankles_two_angles_read_as_a_clinician_reads_them_and_the_start_is_inside_both(self):
        """on the body with cleats: a foot pointed down 30 deg reads +30 and the shin 25 deg forward over the foot -25; a
        sole rolled 20 deg toward the midline reads 20 toward its inversion end on either side (the left's positive, the
        right's negative, as range_specs says) and its normal does face the midline; neither reading moves with the
        other; the ends are the loaded ankle's (39 and 50; 34.6 in, 19.1 out) with no stop moved; the start lies inside
        all four and no stop carries anything there."""
        m = B.Model(ranges=(), grip="hands", feet="cleats")
        st = B.place(m, T0)
        lim = B.add_ranges(m, st)
        a0 = B.range_angles(m, st)
        for sd, sg in B.SIDES:
            self.assertEqual(lim["ankle_" + sd], (-39.0, 50.0))
            self.assertEqual(sorted(sg * x for x in lim["ankle_roll_" + sd]), [-19.1, 34.6])
            for k in ("ankle_", "ankle_roll_"):
                self.assertGreater(a0[k + sd], lim[k + sd][0] + 2.0, msg=k + sd)
                self.assertLess(a0[k + sd], lim[k + sd][1] - 2.0, msg=k + sd)
            medial = E.scale(m.axis(st, "shank_" + sd, (0, 1, 0)), -sg)
            for down in (-25.0, 0.0, 30.0):
                for inward in (-15.0, 0.0, 20.0):
                    s2, sole = clinical_ankle(m, st, sd, down, inward)
                    a = B.range_angles(m, s2)
                    self.assertAlmostEqual(a["ankle_" + sd], down, delta=1e-7, msg="%s %s %s" % (sd, down, inward))
                    self.assertAlmostEqual(sg * a["ankle_roll_" + sd], inward, delta=1e-7, msg="%s %s %s" % (sd, down, inward))
                    self.assertAlmostEqual(E.dot(sole, medial), math.sin(math.radians(inward)), delta=1e-7)
        m.sys.diag = {}
        m.sys.deriv(T0, st, list(m.start_aux))
        self.assertTrue(all(tau == 0.0 for tau, th in m.sys.diag["stop"].values()))
        m.sys.diag = None
        self.assertEqual(m.mu, B.MU_CLEAT)
        self.assertEqual(B.Model(ranges=()).mu, B.MU)        # the plate's body is as it was


def _practice():
    """the practice search's module (analysis/research/body/practice3d.py), for the checks of its score's own rules."""
    import os
    import sys
    here = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "research", "body")
    if here not in sys.path:
        sys.path.insert(0, here)
    import practice3d
    return practice3d


class SearchChecks(unittest.TestCase):
    """the practice search's own rules (body-motion 4.11, after session 9's review)."""

    def test_the_hold_still_torques_work_is_told_apart_from_the_dampings(self):
        """session 10's review, P1-1: the engine books the constant hold-still torques' work and the joints' damping's as
        one integral; practice3d.hold_work splits them from the states. On a rod on a hinge with a constant torque of
        5 N m about the hinge and a damper of 0.7 N m s/rad (capped at 1.5 N m, which it reaches), swung from rest for
        0.3 s: the torque's work is 5 N m x the angle turned, worked here from the rod's own orientation; the two parts
        sum to the engine's integral; the damping's part is negative and its largest torque is the cap; and with the
        constant torque turned the other way its work changes sign (the damper's does not)."""
        PR = _practice()
        for tau in (5.0, -5.0):
            s = E.System(g=(0.0, 0.0, 0.0))
            b = s.body("rod", 2.0, (0.15, 0.15, 0.002))
            s.hinge(-1, b, (0.0, 0.0, 0.0), (0.0, 0.0, 0.3), (0, 1, 0), (0, 1, 0))
            s.tone("hinge", -1, b, (0.0, tau, 0.0))
            s.hold("hinge", -1, b, (1.0, 0.0, 0.0, 0.0), (0.0, 0.0, 0.0), (0.7, 0.7, 0.7), 1.5)
            st = s.new_state()
            s.set_body(st, b, (0.0, 0.0, -0.3), (1.0, 0.0, 0.0, 0.0))
            aux, h = s.new_aux(), 1e-4
            recs = [(0.0, list(st))]
            for i in range(3000):
                st = s.step(i * h, st, aux, h)
                recs.append(((i + 1) * h, list(st)))
            q = s.pose(st, b)[1]
            turned = 2.0 * math.atan2(q[2], q[0])                      # about the hinge's y, from the rod's own quaternion
            tone, damp, peak = PR.hold_work(s, recs, 0.3)
            ow = 13 * s.nb + len(s.acts)
            self.assertGreater(abs(turned), 0.3)
            self.assertAlmostEqual(tone["hinge"], tau * turned, delta=1e-6 * abs(tau * turned))
            self.assertGreater(tone["hinge"], 0.0)                      # either way round, a torque that starts the rod does positive work
            self.assertLess(damp["hinge"], -0.5)
            self.assertAlmostEqual(tone["hinge"] + damp["hinge"], st[ow + 2], delta=1e-5 * abs(tone["hinge"]))
            self.assertAlmostEqual(peak["hinge"], 1.5, delta=1e-9)
            half = PR.hold_work(s, recs, 0.15)                           # to an earlier instant: less of each
            self.assertLess(half[0]["hinge"], tone["hinge"])
            self.assertGreater(half[1]["hinge"], damp["hinge"])

    def test_the_search_without_slopes_finds_a_known_minimum_on_a_stepped_score_with_failing_runs(self):
        """the evolution strategy of body-motion 4.12 (practice3d.cma) on a score with a known answer, built here: 30
        numbers in the unit box, the score the squared distance from a point known in advance (some of its numbers ON
        the box's edge), plus a step of 0.05 at every 0.003 of each number (a score that jumps at a scale far below the
        search's spread, as the practice score does) and one plan in fifty that fails to run. From 0.2 away in every number the search ends within 0.02 of the known point in every number and lowers the
        smooth part of the score a hundredfold; it gives the same result twice from the same seed and another from
        another; it never returns a plan outside the box; a failed run is never the best."""
        PR = _practice()
        n = 30
        target = [0.0 if k % 7 == 0 else 1.0 if k % 7 == 3 else 0.15 + 0.7 * ((k * 37) % 11) / 10.0 for k in range(n)]

        class Fake:
            free = list(range(n))
            calls = 0

            def residuals(self, u):
                Fake.calls += 1
                assert all(0.0 <= x <= 1.0 for x in u)                      # the search never runs a plan outside the box
                if int(sum(u) * 1e6) % 50 == 7:
                    return None                                             # a run that fails
                smooth = [10.0 * (x - t) for x, t in zip(u, target)]
                rough = math.sqrt(0.05 * sum(math.floor(x / 0.003) % 2 for x in u))
                return smooth + [rough]

        class Pool:
            @staticmethod
            def map(f, xs, chunksize=1):
                return [f(x) for x in xs]
        fake = Fake()
        PR._CTX = fake
        u0 = [min(1.0, max(0.0, t + (0.2 if k % 2 else -0.2))) for k, t in enumerate(target)]
        smooth = lambda u: sum((10.0 * (x - t)) ** 2 for x, t in zip(u, target))
        u, cost, hist, centre, spread = PR.cma(fake, u0, 250, lambda s_: None, Pool, seed=3)
        self.assertLess(max(abs(x - t) for x, t in zip(u, target)), 0.02)
        self.assertLess(smooth(u), 0.01 * smooth(u0))
        self.assertEqual(cost, PR.cost_of(fake.residuals(u)))                # the best is a plan that ran, with its own score
        self.assertTrue(all(h[1] <= g[1] for g, h in zip(hist, hist[1:])))   # the best ever never rises
        self.assertGreater(sum(h[5] for h in hist), 0)                       # and some runs did fail on the way
        u2 = PR.cma(fake, u0, 250, lambda s_: None, Pool, seed=3)[0]
        self.assertEqual(u, u2)
        self.assertNotEqual(u, PR.cma(fake, u0, 60, lambda s_: None, Pool, seed=4)[0])

    def test_the_clock_charges_each_arm_muscle_its_own_overrun(self):
        """review P1-1. Five arm muscles tied at the run's first instant, contact 30 ms later than the clock allows them,
        and a sixth inside it. Worked by hand here: moving ALL the tied onsets 10 ms later lowers the charge to (20/30)^2
        of itself; moving ONE of them 10 ms later, alone, lowers it (its own slope has the right sign), and still does
        when that delays contact by a tenth of the move; with every muscle tied the charge is the old form's; a muscle
        inside the clock is charged nothing, and the charge is zero when all are inside. The old form (the earliest
        onset alone) FAILS the one-muscle check: its charge does not fall, and rises with the later contact."""
        PR = _practice()
        sq = lambda xs: sum(x * x for x in xs)
        t_avail, t0 = 0.253, -0.250
        tx = t0 + t_avail + 0.030
        tied = [(t0, w) for w in (1.0, 0.6, 0.9, 0.3, 0.7)]
        late = [(tx - 0.200, 0.8)]
        f = lambda fired, tx_=tx, form="each": sq(PR.clock_residuals(fired, tx_, t_avail, form))
        unit = PR.CLOCK_UNIT
        base = f(tied + late)
        share = sum(w for t, w in tied) / (sum(w for t, w in tied) + 0.8)
        self.assertAlmostEqual(base, share * (0.030 / unit) ** 2, delta=1e-9 * base)             # by hand: the tied ones' share
        self.assertAlmostEqual(f(tied), (0.030 / unit) ** 2, delta=1e-9 * base)                   # all tied: the old form's value
        self.assertAlmostEqual(f(tied), f(tied, form="earliest"), delta=1e-9 * base)
        together = [(t + 0.010, w) for t, w in tied]
        self.assertAlmostEqual(f(together + late), base * (20.0 / 30.0) ** 2, delta=1e-9 * base)
        self.assertEqual(f([(tx - 0.100, 1.0), (tx - 0.250, 0.5)]), 0.0)
        for k in range(len(tied)):
            one = [(t + (0.010 if j == k else 0.0), w) for j, (t, w) in enumerate(tied)]
            self.assertLess(f(one + late), base, msg="muscle %d alone" % k)
            if tied[k][1] >= 0.6:                                 # a muscle asked for enough: still downhill with contact a tenth later
                self.assertLess(f(one + late, tx + 0.001), base, msg="muscle %d alone, contact 1 ms later" % k)
            old0 = f(tied + late, form="earliest")
            self.assertEqual(f(one + late, form="earliest"), old0)                              # the old form: no slope at all
            self.assertGreater(f(one + late, tx + 0.001, form="earliest"), old0)                 # and uphill once contact moves


    def test_a_failed_run_hands_back_its_records_and_a_failure_after_the_last_instant_read_does_not_fail_it(self):
        """review P2-2. (1) The compiled probe on a run that fails (the lone upper arm of RaiseChecks without its raise's
        end, driven overhead): it raises, and the error carries every record made before the failure, each the Python
        engine's own state to 1e-9; the Python engine fails within one record's interval after the last of them.
        (2) The rule, on records built here (a bat whose sweet spot crosses the ball's plane 100 ms into a run): if the
        probe fails 60 ms or more after that crossing the run is used as it is and says so; if it fails 40 ms after,
        or before the crossing, the failure is raised as before."""
        PR = _practice()
        P = PR.P
        try:
            P.build()
        except Exception as e:                               # no compiler on this machine
            self.skipTest("the compiled probe cannot be built here: %s" % e)
        s = E.System(g=(0.0, 0.0, 0.0))
        b = s.body("uarm", 2.5, (0.15, 0.15, 0.004))
        s.ball(-1, b, (0, 0, 0), (0, 0, 0.13))
        lo, hi = sorted(-x for x in B.arm_range()["upper_arm"])
        s.stop("turn", -1, b, (0, 0, 1), (0, -1, 0), (0, 1, 0), math.radians(lo), math.radians(hi), B.RANGE_K["small"], B.RANGE_C["small"], 2)
        s.actuator("raise", -1, b, (0, 0, 1), 5.0, 1e9, -1.0, 0.0, 1e9, 1.0, axis_b=(0, 0, 1))
        st = s.new_state()
        q = E.qmul(E.qaxis((0.6, 0.8, 0.0), math.radians(30.0)), E.qaxis((0, 0, 1), math.radians(180.0)))
        s.set_body(st, b, E.scale(E.mv(E.qR(q), (0, 0, 0.13)), -1.0), q)
        st[13 * s.nb] = 1.0
        with self.assertRaises(P.RunFailed) as cm:
            P.run_raw(s, st, 0.0, 600, 1e-3, 10)
        recs = cm.exception.recs
        self.assertGreater(len(recs), 20)
        aux, s1, ref, failed_at = s.new_aux(), list(st), {}, None
        for i in range(600):
            try:
                s1 = s.step(i * 1e-3, s1, aux, 1e-3)
            except E.SolveError:
                failed_at = i * 1e-3
                break
            if (i + 1) % 10 == 0:
                ref[i + 1] = list(s1)
        self.assertIsNotNone(failed_at)
        t_last = recs[-1][0]
        self.assertLess(t_last, failed_at + 1e-3 + 1e-9)
        self.assertGreater(t_last, failed_at - 0.010 - 1e-9)
        for t, state, *_ in recs[1:]:
            r = ref[int(round(t * 1e3))]
            self.assertLess(max(abs(x - y) for x, y in zip(state, r)), 1e-9)
        # (2) the rule
        m = B.Model(contacts=False)
        ctx = PR.Ctx.__new__(PR.Ctx)
        ctx.m, ctx.evals, ctx.order = m, 0, ()
        ctx.Pc, ctx.nb = (0.5, 0.0, 1.0), (0.0, -1.0, 0.0)   # the ball's place; its line of travel (toward -y)
        ctx.prepare = lambda u: ([], [])
        zero = m.sys.new_state()

        def rec_at(t):                                       # the bat level along +x, its sweet spot moving 10 m/s against the ball
            state = list(zero)
            qb = E.axes_to_q((0.0, 0.0, -1.0), (0.0, 1.0, 0.0), (1.0, 0.0, 0.0))
            ss = (0.5, -1.0 + 10.0 * t, 1.0)
            x = E.sub(ss, E.mv(E.qR(qb), (0.0, 0.0, B.C.BAT_SS - m.bat_ucm)))
            m.sys.set_body(state, m.b["bat"], x, qb, (0.0, 10.0, 0.0))
            return (PR.T0 + t, state, [], [], [], [])
        keep = P.run_raw
        try:
            for fail_after, used in ((0.060, True), (0.200, True), (0.040, False), (-0.030, False)):
                made = [rec_at(k * 0.005) for k in range(int(round((0.100 + fail_after) / 0.005)) + 1)]

                def fake(*a, **kw):
                    raise P.RunFailed("made to fail", made)
                P.run_raw = fake
                if used:
                    got = ctx.run(None)
                    self.assertEqual(len(got), len(made))
                    self.assertAlmostEqual(ctx.failed_after[0], fail_after, delta=1e-6)
                    tx, p, v, i, came = ctx.contact(got)
                    self.assertTrue(came)
                    self.assertAlmostEqual(tx, PR.T0 + 0.100, delta=1e-6)
                else:
                    with self.assertRaises(PR.E.SolveError):       # the search's own engine module (the same class under another import)
                        ctx.run(None)
                    self.assertIsNone(ctx.failed_after)
                    self.assertEqual(ctx.run_end, PR.T_END)
        finally:
            P.run_raw = keep


class LadderChecks(unittest.TestCase):
    """session 11's changes to the body (body-motion 4.13): the joints' damping by joint, the posture held by muscle."""

    O = dict(grip="hands", feet="cleats", hold="let_go", wrist="oval", clear=True, h=2.5e-4)

    def test_a_damper_takes_a_spinning_rods_speed_as_the_worked_decay(self):
        """a rod on a hinge, no gravity, spun at 6 rad/s against a damper of C N m s/rad with no cap: its speed must
        fall as exp(-C t / I), I its moment of inertia about the hinge worked here by hand (0.15 + 2 x 0.3^2); and the
        work booked to the damper is the kinetic energy lost. With drive.damping_of's cap (a number: no cap) and with
        the old capped form the law is the same below the cap."""
        for C in (0.05, 0.4, 1.0):
            s = E.System(g=(0.0, 0.0, 0.0))
            b = s.body("rod", 2.0, (0.15, 0.15, 0.002))
            s.hinge(-1, b, (0.0, 0.0, 0.0), (0.0, 0.0, 0.3), (0, 1, 0), (0, 1, 0))
            s.hold("hinge", -1, b, (1.0, 0.0, 0.0, 0.0), (0.0, 0.0, 0.0), (C, C, C), Dr.NO_CAP)
            st = s.new_state()
            w0 = 6.0
            s.set_body(st, b, (0.0, 0.0, -0.3), (1.0, 0.0, 0.0, 0.0), (-0.3 * w0, 0.0, 0.0), (0.0, w0, 0.0))
            aux, h = s.new_aux(), 1e-4
            for i in range(2000):
                st = s.step(i * h, st, aux, h)
            I = 0.15 + 2.0 * 0.3 ** 2
            w = s.pose(st, b)[3][1]
            self.assertAlmostEqual(w, w0 * math.exp(-C * 0.2 / I), delta=1e-6 * w0)
            ow = 13 * s.nb + len(s.acts)
            self.assertAlmostEqual(st[ow + 2], 0.5 * I * (w * w - w0 * w0), delta=1e-6)
            self.assertLess(st[ow + 2], 0.0)

    def test_the_damping_table_is_read_as_written(self):
        """drive.damping_of: no option is sessions 1-10's body (1.0 everywhere, capped at 30 N m); a number is that value
        everywhere with no cap; a set by name is the set, with no cap; "from" takes only the named group's joints from
        another set and "scale" multiplies only those. The three sets differ only at the joints nothing was read for
        (a read joint has one value in all three; an unread one's low end is not above its high end). The values
        themselves are not restated here (session 11's review, P3-4): they are written once, in drive.DAMPING, from
        note 14."""
        t, cap = Dr.damping_of(None)
        self.assertEqual(set(t.values()), {(1.0, 1.0, 1.0)})
        self.assertEqual(cap, 30.0)
        t, cap = Dr.damping_of(0.3)
        self.assertEqual(set(t.values()), {(0.3, 0.3, 0.3)})
        self.assertEqual(cap, Dr.NO_CAP)
        read, cap = Dr.damping_of("read")
        self.assertEqual(cap, Dr.NO_CAP)
        for name in ("low", "high"):
            other = Dr.damping_of(name)[0]
            self.assertEqual(set(other), set(Dr.JOINTS))
            for j in Dr.JOINTS:
                if j.startswith(("knee", "ankle", "shoulder")):
                    self.assertEqual(other[j], read[j])
                elif j.startswith("wrist"):
                    self.assertEqual(other[j][0], read[j][0])            # the bend is read; the tilt is not
                    self.assertNotEqual(other[j][1], read[j][1])
                else:
                    self.assertNotEqual(other[j], read[j])
            for a_, b_ in zip(sorted(Dr.damping_of("low")[0].items()), sorted(Dr.damping_of("high")[0].items())):
                self.assertTrue(all(x <= y for x, y in zip(a_[1], b_[1])))
        t = Dr.damping_of({"set": "read", "from": {"trunk": "high"}, "scale": {"arms": 2.0}})[0]
        for j in Dr.JOINTS:
            want = Dr.damping_of("high")[0][j] if j == "lumbar" else tuple(x * (2.0 if j in Dr.DAMP_GROUPS["arms"] else 1.0) for x in read[j])
            self.assertEqual(t[j], want)

    @NEED
    def test_a_joints_two_dampings_act_on_its_two_movements(self):
        """the elbow's damping is written as (turn, bend, turn) about the upper arm's axes and the wrist's as (bend, tilt,
        tilt) about the forearm's. On the placed body: with the forearm given a turning rate about the elbow's HINGE alone
        (the upper arm's y) the damper's torque is the bend's coefficient x that rate about the hinge; with a rate about
        the forearm's own LENGTH alone it is the turn's coefficient x that rate; the two coefficients differ fourteenfold
        in the central set, so a mix-up would show. The same at the wrist for its bend (the forearm's x)."""
        m = B.Model(ranges=(), grip="hands", feet="cleats", wrist="oval")
        st = B.place(m, T0, moving=False)
        Dr.add_damping(m, st, "read")
        s = m.sys
        holds = {h[0]: h for h in s.holds}

        def torque(joint, axis_w, rate):
            name, a, b, qref, K, C, cap = holds[joint]
            s2 = list(st)
            s2[13 * b + 10:13 * b + 13] = E.scale(axis_w, rate)          # the parent at rest
            Ra = E.qR(s.pose(s2, a)[1])
            wl = E.mtv(Ra, E.scale(axis_w, rate))
            return E.mv(Ra, tuple(-c * w for c, w in zip(C, wl)))
        hinge = m.axis(st, "uarm_L", (0, 1, 0))
        length = m.axis(st, "farm_L", (0, 0, 1))
        self.assertLess(abs(E.dot(hinge, length)), 1e-9)                   # the hinge holds the forearm's length square to it
        t1 = torque("elbow_L", hinge, 3.0)
        self.assertAlmostEqual(E.dot(t1, hinge), -0.43 * 3.0, delta=1e-12)
        t2 = torque("elbow_L", length, 3.0)
        self.assertAlmostEqual(E.dot(t2, length), -0.03 * 3.0, delta=1e-12)
        self.assertLess(E.norm(E.sub(t2, E.scale(length, -0.09))), 1e-12)
        bend = m.axis(st, "farm_R", (1, 0, 0))
        t3 = torque("wrist_R", bend, 5.0)
        self.assertAlmostEqual(E.dot(t3, bend), -0.04 * 5.0, delta=1e-12)

    @NEED
    def test_the_posture_held_by_muscle_starts_as_the_constant_torques_did_and_then_obeys_the_muscles_rules(self):
        """body-motion 4.13, rung 2. With each hold-still torque handed to the muscle group that pulls that way:
        (1) at the first instant every body's acceleration is what the constant torques gave (the split is exact);
        (2) every holding strength is positive and, with the most its group's burst may ask, makes the group's measured
        bound; a knee, which has one axis and a muscle each way, keeps no constant torque, and neither does the trunk,
        which has holding groups on all three axes; the ankle's roll, which has no muscle, keeps its torque;
        (3) the front knee extensor's holding torque follows that group's measured strength-at-speed curve (4.5: half its
        strength at 3.952 rad/s, none at 13.4 rad/s and beyond), read here from the curve's printed points and not from
        the engine: with the shank turning that fast about the knee the holding drive gives half, and nothing; forced
        the other way it gives more, never over the stretched muscle's plateau of 1.5;
        (4) it goes when the plan switches its muscle off, and a hold-only group (the trunk's bending) never fires;
        (5) held and doing nothing else, a group's effort is (its holding strength / its bound) squared."""
        PR = _practice()
        acc = {}
        for stand in ("tone", "muscle"):
            c = PR.Ctx(stand=stand, **self.O)
            s = c.m.sys
            st, aux = c.prepare(c.u_of({}))
            ds = s.deriv(PR.T0, st, aux)
            acc[stand] = [ds[13 * i + 7 + k] for i in range(s.nb) for k in range(6)]
        self.assertLess(max(abs(x - y) for x, y in zip(acc["tone"], acc["muscle"])), 1e-9 * max(abs(x) for x in acc["tone"]))
        self.assertGreater(len(c.hold_T), 15)
        for n, T in c.hold_T.items():
            self.assertGreater(T, 0.0)
            self.assertLessEqual(T, c.hold_bd[n])
            if n in c.cap:
                self.assertAlmostEqual(c.cap[n] + T, c.bd[n], delta=1e-9)
        for n, f, lo, hi in c.box:
            if f == "T":
                self.assertAlmostEqual(hi, c.cap[n], delta=1e-12)
        for j in ("knee_L", "knee_R", "lumbar"):
            self.assertLess(E.norm(c.tone_left[j]), 1e-6)
        self.assertGreater(E.norm(c.tone_left["ankle_R"]), 5.0)
        self.assertGreater(c.hold_at_start["knee_ext_L"], 50.0)                 # the bent front knee is held by its extensor
        self.assertNotIn("knee_flex_L", c.hold_T)
        # (3) the holding drive at speed: the lead shank given a turning rate about the knee's hinge
        j = c.hold_ix["knee_ext_L"]
        a_ = s.acts[j]
        thigh, shank = a_[1], a_[2]
        n_w = E.mv(E.qR(s.pose(st, thigh)[1]), a_[3])
        T = c.hold_T["knee_ext_L"]

        def torque_at(w):
            s2 = list(st)
            wt = s.pose(st, thigh)[3]
            s2[13 * shank + 10:13 * shank + 13] = E.add(wt, E.scale(n_w, w))
            s.diag = {}
            s.deriv(PR.T0, s2, aux)
            tau, wrel, act = s.diag["act"][a_[0]]
            s.diag = None
            self.assertAlmostEqual(wrel, w, delta=1e-9)
            return tau
        self.assertAlmostEqual(torque_at(0.0), T, delta=1e-9 * T)
        self.assertAlmostEqual(torque_at(3.952), 0.50 * T, delta=1e-9 * T)
        self.assertAlmostEqual(torque_at(1.517), 0.75 * T, delta=1e-9 * T)
        self.assertEqual(torque_at(13.4), 0.0)
        self.assertEqual(torque_at(20.0), 0.0)
        self.assertGreater(torque_at(-3.0), T)
        self.assertLess(torque_at(-50.0), 1.5 * T)
        # (4) letting go
        pl = {n: (0.0, PR.T0, 0.05, 0.02 + 0.001 * k, 0.0) for k, n in enumerate(PR.CH)}
        st, aux = c.prepare(c.u_of(pl))
        off = c.plan(c.u_of(pl))["knee_ext_L"][3]
        self.assertEqual(s.excitation(j, off - 1e-6), 1.0)
        self.assertEqual(s.excitation(j, off + 1e-6), 0.0)
        jt = c.hold_ix["trunk_side_L"]
        self.assertEqual(s.excitation(jt, 10.0), 1.0)
        self.assertNotIn("trunk_side_L", PR.CH)
        # (5) effort
        act = [(0.0, 0.0, 0.0)] * len(PR.CH) + [(0.0, 0.0, 1.0)] * len(c.hold_ix)
        recs = [(PR.T0 + 0.001 * k, None, None, act, None, None) for k in range(5)]
        eff = c.effort(recs, PR.T0 + 0.1, c.plan(c.u_of({})))
        self.assertAlmostEqual(eff["knee_ext_L"], (T / c.bd["knee_ext_L"]) ** 2, delta=1e-12)
        self.assertAlmostEqual(eff["hip_flex_L"], (c.hold_T["hip_flex_L"] / c.bd["hip_flex_L"]) ** 2, delta=1e-12)
        self.assertEqual(eff["knee_flex_L"], 0.0)


class ReviewElevenChecks(unittest.TestCase):
    """body-motion 4.13, after the independent review of session 11: the front foot's load read with its time; the
    strength scaling of rung 6's what-ifs; the reading of Statcast's foot points."""

    @staticmethod
    def _research(name):
        _practice()
        import importlib
        return importlib.import_module(name)

    def test_the_front_foots_load_row_says_when_the_peak_falls_and_what_the_feet_carry_at_contact(self):
        """ladder3d.front_foot_row (review P2-2), against three made-up swings worked by hand, each 270 ms from launch
        to contact. (1) A peak of 2.0 x body weight 200 ms before contact is 70 ms after the start, 0.26 of the way:
        inside the gate's band on its size, marked as passing on size only; the row says both times and the loads at
        contact, 0.04 on the front foot and 0.04 + 0.31 = 0.35 on both. (2) The same peak 60 ms before contact (210 ms
        after the start, 0.78 of the way) passes with no mark. (3) A peak of 1.2, late, fails on its size: short by
        0.30. (4) Without the time from launch to contact the row cannot place the peak and does not mark it; given how
        long the load lasted (13 ms at or over 1.5) and its mean over the last 100 ms (0.44), it says them.
        Then rung 4's kept swing, whose figures practice3d prints itself and the review read by its own replay: 2.35 x
        body weight 201 ms before contact, 70 ms after the start, the front foot 0.04 and both feet 0.35 at contact:
        passed on size only, so of the gate's eighteen rows eight pass outright, one so, nine fail."""
        L = self._research("ladder3d")
        r = L.front_foot_row({"lead_foot_peak": (2.0, -200.0), "feet_at_contact": (0.04, 0.31), "start_to_contact_ms": 270.0})
        self.assertTrue(r[4])
        self.assertIn(L.SIZE_ONLY, r[3])
        self.assertIn("inside by 0.50", r[3])
        self.assertIn("200 ms before contact", r[1])
        self.assertIn("70 ms after the start", r[1])
        self.assertIn("the front foot 0.04, both feet 0.35", r[1])
        r = L.front_foot_row({"lead_foot_peak": (2.0, -60.0), "feet_at_contact": (0.9, 0.2), "start_to_contact_ms": 270.0})
        self.assertTrue(r[4])
        self.assertNotIn(L.SIZE_ONLY, r[3])
        self.assertIn("210 ms after the start", r[1])
        self.assertIn("both feet 1.10", r[1])
        r = L.front_foot_row({"lead_foot_peak": (1.2, -60.0), "feet_at_contact": (0.9, 0.2), "start_to_contact_ms": 270.0})
        self.assertFalse(r[4])
        self.assertIn("short by 0.30", r[3])
        self.assertNotIn(L.SIZE_ONLY, r[3])
        r = L.front_foot_row({"lead_foot_peak": (2.0, -200.0), "feet_at_contact": (0.04, 0.31)})
        self.assertTrue(r[4])
        self.assertNotIn(L.SIZE_ONLY, r[3])
        self.assertNotIn("at or over 1.5", r[1])
        r = L.front_foot_row({"lead_foot_peak": (2.0, -60.0), "feet_at_contact": (0.9, 0.2), "start_to_contact_ms": 270.0,
                              "lead_foot_load": {"ms at or over 1.5": 13.0, "mean over the last 100 ms": (0.44, 0.29)}})
        self.assertIn("at or over 1.5 for 13 ms in all; the mean over the last 100 ms 0.44", r[1])
        import json
        import os
        with open(os.path.join(os.path.dirname(os.path.abspath(L.__file__)), "practice3d_s11_rung4.json")) as f:
            kept = json.load(f)
        rows = L.gate(kept)
        row = [x for x in rows if x[0] == "the front foot's greatest load"][0]
        self.assertIn("2.35 x body weight 201 ms before contact (70 ms after the start", row[1])
        self.assertIn("the front foot 0.04, both feet 0.35", row[1])
        self.assertIn(L.SIZE_ONLY, row[3])
        yes, half, no = L.passed(rows)
        self.assertEqual((len(yes), half, len(no)), (8, ["the front foot's greatest load"], 9))

    @NEED
    def test_a_scaled_strength_reaches_the_bound_the_tendon_spring_the_bursts_cap_and_the_effort(self):
        """practice3d's bound_scale (rung 6's what-if bodies; review P1-3 and P3-4), the body with every muscle at 1.5
        times beside the body as it is, each expectation worked from the unscaled body: (1) every group's bound is 1.5
        times; (2) every tendon spring is 1.5 times as stiff (a tendon stretches a fixed share of its length at the
        muscle's full strength, 4.5, so a muscle half as strong again has a spring half as stiff again), and the legs'
        have one; (3) holding the same posture takes the same torques, so the holding strengths are unchanged in N m
        and the most a burst may ask is 1.5 x the bound less the same holding strength: more than 1.5 times the old
        cap wherever the group holds; (4) the search's box lets the plan ask that; (5) held and doing nothing else, a
        group's effort is 1/2.25 of what it was (the same torque, of a bound 1.5 times as large, squared); (6) the
        hold-only trunk-bending groups, which cannot fire, are not scaled."""
        PR = _practice()
        o = dict(LadderChecks.O, stand="muscle", damp="read")
        a = PR.Ctx(**o)
        b = PR.Ctx(bound_scale={n: 1.5 for n in PR.CH}, **o)
        for n in PR.CH:
            self.assertAlmostEqual(b.bd[n], 1.5 * a.bd[n], delta=1e-9 * a.bd[n])
        ka = {x[0]: x[11] for x in a.m.sys.acts}
        kb = {x[0]: x[11] for x in b.m.sys.acts}
        self.assertEqual(set(ka), set(kb))
        sprung = [n for n in PR.CH if ka[n] > 0.0]
        self.assertTrue({"knee_ext_L", "hip_ext_R", "ankle_pf_L"} <= set(sprung))
        for n in sprung:
            self.assertAlmostEqual(kb[n], 1.5 * ka[n], delta=1e-9 * ka[n])
        self.assertEqual(set(a.hold_T), set(b.hold_T))
        self.assertGreater(len([n for n in PR.CH if n in a.hold_T]), 10)
        for n, T in a.hold_T.items():
            self.assertAlmostEqual(b.hold_T[n], T, delta=1e-6 * max(1.0, T))
        for n in PR.CH:
            self.assertAlmostEqual(b.cap[n], 1.5 * a.bd[n] - a.hold_T.get(n, 0.0), delta=1e-6 * a.bd[n])
            if a.hold_T.get(n, 0.0) > 1.0:
                self.assertGreater(b.cap[n], 1.5 * a.cap[n])
        hi = {n: h for n, f, lo, h in b.box if f == "T"}
        for n in PR.CH:
            self.assertAlmostEqual(hi[n], b.cap[n], delta=1e-12)
        effort = lambda c: c.effort([(PR.T0 + 0.001 * k, None, None, [(0.0, 0.0, 0.0)] * len(PR.CH) + [(0.0, 0.0, 1.0)] * len(c.hold_ix), None, None)
                                     for k in range(5)], PR.T0 + 0.1, c.plan(c.u_of({})))
        ea, eb = effort(a), effort(b)
        self.assertGreater(ea["knee_ext_L"], 1e-4)
        for n in ("knee_ext_L", "hip_flex_L", "hip_ext_R", "hip_abd_L"):
            self.assertGreater(ea[n], 0.0)
            self.assertAlmostEqual(eb[n], ea[n] / 2.25, delta=1e-6 * ea[n])
        self.assertGreater(eb["core"], ea["core"] / 2.25)     # booked with the hold-only trunk groups' effort, which is not scaled (6)
        self.assertLess(eb["core"], ea["core"])
        for n in Dr.HOLD_ONLY:
            self.assertAlmostEqual(b.hold_bd[n], a.hold_bd[n], delta=1e-12)

    def test_a_foots_ankle_and_turn_are_read_from_its_three_points_and_a_left_hander_is_mirrored(self):
        """statcast_feet.feet and .measures (review P3-3), on a made-up right-handed hitter worked by hand, in feet in the
        plate's frame as the page has them. His front (left) foot at the intercept: heel at (-2.0, 3.0), big toe at
        (-1.2, 3.7), small toe at (-1.4, 3.9), so the toe's midpoint is (-1.3, 3.8): heel to toe (0.7, 0.8), turned
        atan2(0.8, 0.7) = 48.81 deg toward the pitcher from square; the ankle a quarter of the way: (-1.825, 3.2).
        His rear (right) foot: heel (-2.4, -0.4), toes (-1.5, -0.3) and (-1.5, -0.5): toe (-1.5, -0.4), square (0 deg),
        ankle (-2.175, -0.4). So the ankles are 3.6 ft = 109.728 cm apart along the line to the pitcher and the lead one
        0.35 ft = 10.668 cm nearer the plate; with the intercept 3.1 ft ahead of the plate's rear apex the ball is
        3.5 ft = 106.68 cm ahead of the rear ankle and 0.1 ft = 3.048 cm behind the lead one. His hips in the stance
        30 in behind the plate's front edge are 17 - 30 = -13 in = -33.02 cm, so the ball is 94.488 + 33.02 = 127.508 cm
        ahead of them. The same hitter left-handed (every x negated, left and right feet swapped) gives every figure the
        same."""
        SF = self._research("statcast_feet")

        def hitter(side):
            m = 1.0 if side == "R" else -1.0
            lead, rear = ("l", "r") if side == "R" else ("r", "l")
            r = {"bat_side": side, "avg_intercept_y": 3.1, "avg_batter_y_position": 30.0, "avg_foot_sep0": 30.0, "avg_intercept_y_in": 37.2,
                 "avg_intercept_y_vs_batter": 50.2}
            pts = {(lead, "heel"): (-2.0, 3.0), (lead, "bigtoe"): (-1.2, 3.7), (lead, "smalltoe"): (-1.4, 3.9),
                   (rear, "heel"): (-2.4, -0.4), (rear, "bigtoe"): (-1.5, -0.3), (rear, "smalltoe"): (-1.5, -0.5)}
            for (f, part), (x, y) in pts.items():
                for t in (0, 1, 2):
                    r["avg_%s%s_x%d" % (f, part, t)] = m * x
                    r["avg_%s%s_y%d" % (f, part, t)] = y
            return r
        for side in "RL":
            F = SF.feet(hitter(side))
            self.assertAlmostEqual(F["lead", 2]["turn"], 48.8141, delta=1e-3)
            self.assertAlmostEqual(F["rear", 2]["turn"], 0.0, delta=1e-9)
            for got, want in zip(F["lead", 2]["ankle"], (-1.825 * 30.48, 3.2 * 30.48)):
                self.assertAlmostEqual(got, want, delta=1e-9)
            for got, want in zip(F["rear", 2]["ankle"], (-2.175 * 30.48, -0.4 * 30.48)):
                self.assertAlmostEqual(got, want, delta=1e-9)
            o = SF.measures([hitter(side)])
            self.assertAlmostEqual(o["ankles apart, along (cm)"][0], 109.728, delta=1e-9)
            self.assertAlmostEqual(o["lead ankle across (cm)"][0], 10.668, delta=1e-9)
            self.assertAlmostEqual(o["ball ahead of the rear ankle (cm)"][0], 106.68, delta=1e-9)
            self.assertAlmostEqual(o["ball ahead of the lead ankle (cm)"][0], -3.048, delta=1e-9)
            self.assertAlmostEqual(o["ball ahead of the hips in the stance (cm)"][0], 127.508, delta=1e-9)
            self.assertAlmostEqual(o["front foot's turn (deg)"][0], 48.8141, delta=1e-3)


class SessionTwelveChecks(unittest.TestCase):
    """body-motion 4.14 (session 12): the readings of the diagnosis of what holds the pelvis (pelvis3d.py)."""

    @staticmethod
    def _pelvis():
        _practice()
        import importlib
        return importlib.import_module("pelvis3d")

    def test_a_bodys_turning_momentum_about_the_vertical_is_the_worked_one(self):
        """pelvis3d.turning_momentum against a body worked by hand: 2 kg, moments of inertia 0.1, 0.2, 0.3 about its own
        x, y, z, lying with its own z along the world's x (so its own x is along the world's -z, and the moment of
        inertia about the vertical is its own x's, 0.1), turning at 5 rad/s about the vertical, its centre 0.5 m along x
        from the reference point and moving at 3 m/s along y. Its travel's part is m r v = 2 x 0.5 x 3 = 3.0, its own
        0.1 x 5 = 0.5: 3.5 kg m2/s. About a reference point moving with it at 3 m/s along y the travel's part is gone:
        0.5. And turned a further 90 deg about the vertical the vertical moment of inertia is unchanged: 3.5 again."""
        PV = self._pelvis()
        s = E.System()
        b = s.body("rod", 2.0, (0.1, 0.2, 0.3))
        st = s.new_state()
        q = E.qaxis((0, 1, 0), math.pi / 2)
        s.set_body(st, b, (0.5, 0.0, 0.0), q, (0.0, 3.0, 0.0), (0.0, 0.0, 5.0))
        self.assertAlmostEqual(PV.turning_momentum(s, st, [b], (0.0, 0.0, 0.0), (0.0, 0.0, 0.0)), 3.5, delta=1e-9)
        self.assertAlmostEqual(PV.turning_momentum(s, st, [b], (0.0, 0.0, 0.0), (0.0, 3.0, 0.0)), 0.5, delta=1e-9)
        s.set_body(st, b, (0.5, 0.0, 0.0), E.qmul(E.qaxis((0, 0, 1), math.pi / 2), q), (0.0, 3.0, 0.0), (0.0, 0.0, 5.0))
        self.assertAlmostEqual(PV.turning_momentum(s, st, [b], (0.0, 0.0, 0.0), (0.0, 0.0, 0.0)), 3.5, delta=1e-9)

    def test_the_grounds_twist_is_split_into_the_level_forces_lever_and_the_foots_own_twist(self):
        """pelvis3d.ground_twist against two cases worked by hand. One foot's two sole points at (0.1, +0.05) and
        (0.1, -0.05) m carry 100 N each, so its centre of pressure is (0.1, 0). (1) Level forces (0, +30) and (0, -10) N:
        the net (0, 20) at the centre has the moment 0.1 x 20 = 2.0 N m about the vertical through the origin, and about
        the foot's own centre the two forces have no lever: its own twist is 0. (2) Level forces (+30, 0) and (-10, 0):
        the net (20, 0) at (0.1, 0) has no moment about the origin; about the centre the first has -(0.05)(30) = -1.5
        and the second -(-0.05)(-10) = -0.5: the foot's own twist is -2.0. In both the two parts sum to the forces' whole
        moment about the origin. (3) Loads of 300 and 100 N move the centre to y = +0.025; a point carrying nothing is
        left out and counted as off the ground."""
        PV = self._pelvis()

        class M_:
            pass
        m = M_()
        m.sys = E.System()
        fb = m.sys.body("foot", 1.0, (1.0, 1.0, 1.0))
        m.sys.contact("L_a", fb, (0.1, 0.05, 0.0), 1, 1, 1, 1, 1)
        m.sys.contact("L_b", fb, (0.1, -0.05, 0.0), 1, 1, 1, 1, 1)
        st = m.sys.new_state()
        for fs, lever, own in ((((0.0, 30.0, 100.0), (0.0, -10.0, 100.0)), 2.0, 0.0), (((30.0, 0.0, 100.0), (-10.0, 0.0, 100.0)), 0.0, -2.0)):
            g = PV.ground_twist(m, st, list(fs), (0.0, 0.0, 0.0))["L"]
            self.assertAlmostEqual(g[0], 200.0, delta=1e-12)
            self.assertAlmostEqual(g[1], lever, delta=1e-12)
            self.assertAlmostEqual(g[2], own, delta=1e-12)
            whole = sum(p[0] * f[1] - p[1] * f[0] for p, f in zip(((0.1, 0.05), (0.1, -0.05)), fs))
            self.assertAlmostEqual(g[1] + g[2], whole, delta=1e-12)
            self.assertEqual(g[3], 2)
        g = PV.ground_twist(m, st, [(30.0, 0.0, 300.0), (-10.0, 0.0, 100.0)], (0.0, 0.0, 0.0))["L"]
        self.assertAlmostEqual(g[1], -(0.025) * 20.0, delta=1e-12)           # the net (20, 0) at (0.1, 0.025)
        self.assertAlmostEqual(g[1] + g[2], -(0.05 * 30.0) - (-0.05 * -10.0), delta=1e-12)
        g = PV.ground_twist(m, st, [(30.0, 0.0, 300.0), (0.0, 0.0, 0.0)], (0.0, 0.0, 0.0))["L"]
        self.assertEqual(g[3], 1)
        self.assertAlmostEqual(g[2], 0.0, delta=1e-12)                       # one point holds no twist about itself
        self.assertEqual(PV.ground_twist(m, st, [(0.0, 0.0, 0.0), (0.0, 0.0, 0.0)], (0.0, 0.0, 0.0))["R"], (0.0, 0.0, 0.0, 0))

    def test_the_first_stage_charges_a_bat_that_does_not_come_through_the_ball(self):
        """practice3d.through_charge (4.14, part 3: the parked basin's remedy), worked by hand at 5 m/s asked. A bat
        parked at the ball (0 m/s along the ball's line where it comes nearest) is charged 5, which the score squares
        to 25; one moving back at 2.3 m/s there (session 10's parked runs: -0.2 to -2.3) 7.3, squared 53.29; one
        passing through at exactly 5 m/s, or at a swing's 12, nothing."""
        PR = _practice()
        self.assertAlmostEqual(PR.through_charge(5.0, 0.0) ** 2, 25.0, delta=1e-12)
        self.assertAlmostEqual(PR.through_charge(5.0, -2.3) ** 2, 53.29, delta=1e-9)
        self.assertEqual(PR.through_charge(5.0, 5.0), 0.0)
        self.assertEqual(PR.through_charge(5.0, 12.0), 0.0)

    def test_the_search_carried_to_its_end_stops_by_its_rule_and_finds_a_known_least(self):
        """practice3d.cma's end rule (4.14, part 3), on a score whose least is known and not the search's own: six
        numbers, the score the sum of (10 (x - c))^2 with c = 0.2, 0.3, ..., 0.7, least 0 at x = c. From x = 0.5
        everywhere: (1) with stop = (40, 1e-9) it ends before the 5,000 generations allowed, at a score under 1e-6 and
        every number within 1e-4 of its c, and over its last 40 generations the best score fell less than 1e-9; (2) with
        no end rule the same search runs every generation asked (60) and has not yet reached that score; (3) a rule
        that can never be met (a fall under 0) runs to the cap."""
        PR = _practice()
        c = [0.2, 0.3, 0.4, 0.5, 0.6, 0.7]

        class Toy:
            free = list(range(6))

            @staticmethod
            def residuals(u):
                return [10.0 * (x - k) for x, k in zip(u, c)]

        class Pool:
            @staticmethod
            def map(f, us, chunksize=1):
                return [f(u) for u in us]
        old = PR._CTX
        PR._CTX = Toy
        try:
            quiet = lambda s_: None
            u, best, hist, _, _ = PR.cma(Toy, [0.5] * 6, 5000, quiet, Pool, seed=3, stop=(40, 1e-9))
            self.assertLess(hist[-1][0], 5000)
            self.assertLess(best, 1e-6)
            for x, k in zip(u, c):
                self.assertAlmostEqual(x, k, delta=1e-4)
            self.assertLess(hist[-41][1] - hist[-1][1], 1e-9)
            self.assertGreaterEqual(hist[-42][1] - hist[-2][1], 1e-9)       # it did not stop a generation late
            u2, best2, hist2, _, _ = PR.cma(Toy, [0.5] * 6, 60, quiet, Pool, seed=3)
            self.assertEqual(hist2[-1][0], 60)
            self.assertGreater(best2, best)
            self.assertEqual([h[1] for h in hist2], [h[1] for h in hist[:61]])   # the rule changes nothing before it ends the search
            u3, best3, hist3, _, _ = PR.cma(Toy, [0.5] * 6, 80, quiet, Pool, seed=3, stop=(40, 0.0))
            self.assertEqual(hist3[-1][0], 80)
        finally:
            PR._CTX = old

    def test_the_posture_given_to_the_search_is_rung_fours_start_until_it_is_moved_and_then_moves_what_it_names(self):
        """body-motion 4.14, part 4 (posture3d.py, practice3d.Ctx's posture, arms and Pc). (1) With no posture change, the
        start's three free choices given as rung 4's own and the ball's place given as rung 4's, the placed start is rung
        4's to the last digit: nothing else came in with the options. (2) The pelvis's place +3 cm moves the pelvis's
        centre 3.0 cm toward the pitcher (+Y) and not across; both ankles stay where they were along and across within
        0.5 mm (the feet's places are Statcast's, not the search's; the front ankle is lower, since the front heel, raised
        on the ball of the foot at launch, comes down as the pelvis comes over it); the ball's place is unchanged. (3) The front knee's number +2 bends the front knee 2.0 deg more and leaves the rear knee's bend.
        (4) A number that is the feet's (the stride) is not among the fourteen, and each of the fourteen is a posture
        number of body.py."""
        PR = _practice()
        import importlib
        Q = importlib.import_module("posture3d")
        L = importlib.import_module("ladder3d")
        o = dict(L.options("r4"), contact="nearest")
        pc = Q.PostureCtx(o)
        base = pc.base
        same = PR.Ctx(**dict(pc.o, posture={}))
        self.assertEqual(max(abs(a - b) for a, b in zip(same.st, base.st)), 0.0)
        self.assertEqual(tuple(same.Pc), tuple(base.Pc))
        ank = lambda c, sd: c.m.joint_pos(c.st, "ankle_" + sd)
        pel = lambda c: c.m.sys.pose(c.st, c.m.b["pelvis"])[0]
        moved = PR.Ctx(**dict(pc.o, posture={"pelvis_x_cm": 3.0}))
        self.assertAlmostEqual(pel(moved)[1] - pel(base)[1], 0.030, delta=2e-4)
        self.assertAlmostEqual(pel(moved)[0] - pel(base)[0], 0.0, delta=2e-4)
        for sd in "LR":
            for c in range(2):                                   # each ankle's place along and across is the feet's measurement
                self.assertAlmostEqual(ank(moved, sd)[c], ank(base, sd)[c], delta=5e-4)
        self.assertAlmostEqual(ank(moved, "R")[2], ank(base, "R")[2], delta=5e-4)
        self.assertLess(ank(moved, "L")[2], ank(base, "L")[2] - 0.02)   # the front heel comes down as the pelvis comes over it
        self.assertEqual(tuple(moved.Pc), tuple(base.Pc))
        free = PR.Ctx(**dict({k: v for k, v in pc.o.items() if k != "Pc"}, posture={"pelvis_rot": 2.0}))
        held = PR.Ctx(**dict(pc.o, posture={"pelvis_rot": 2.0}))
        self.assertEqual(tuple(held.Pc), tuple(base.Pc))        # given, the ball's place is exactly rung 4's
        for c in range(3):                                       # not given, these numbers would not carry it anyway: under 0.1 mm
            self.assertAlmostEqual(free.Pc[c], base.Pc[c], delta=1e-4)
        knee = lambda c, sd: c.m.flexion(c.st, "thigh_" + sd, "shank_" + sd)
        bent = PR.Ctx(**dict(pc.o, posture={"lead_knee_flex": 2.0}))
        self.assertAlmostEqual(knee(bent, "L") - knee(base, "L"), 2.0, delta=0.05)
        self.assertAlmostEqual(knee(bent, "R") - knee(base, "R"), 0.0, delta=0.05)
        self.assertEqual(len(Q.NUMBERS), 14)
        self.assertNotIn("stride_cm", Q.NUMBERS)
        self.assertTrue(set(Q.NUMBERS) <= set(B.POSTURE_NUMBERS))
        # the search's numbers for the kept posture are the middle of each range, and come back as no change
        u = pc.u_of({n: (0.0, 0.0, 0.05, 0.1, 0.0) for n in PR.CH}, {"trail_knee": -5.0})
        self.assertAlmostEqual(pc.posture_of(u)["trail_knee"], -5.0, delta=1e-12)
        self.assertAlmostEqual(pc.posture_of(u)["pelvis_rot"], 0.0, delta=1e-12)

    def test_the_starts_margins_are_charged_as_worked_by_hand(self):
        """posture3d.margin_residuals (4.14, part 4): each joint's shortfall of 5 deg inside its end at the start, one
        unit a degree; the score squares each. Worked: margins of 4.0, 4.7, 5.0, 9.0 and -1.0 deg (one past its end)
        give shortfalls 1.0, 0.3, 0, 0 and 6.0, whose squares sum to 1 + 0.09 + 36 = 37.09. And on rung 4's own start,
        whose margins body-motion 4.13 printed from the start's rule (the rear ankle's roll 4.0 deg inside its end, both
        hips' sideways margins and the lead ankle's roll at 4.7, every other joint 5 or more), the charge is
        1.0^2 + 3 x 0.3^2 = 1.27 by those rounded figures: 1.17 from the margins themselves, within 0.15 of it."""
        PR = _practice()
        import importlib
        Q = importlib.import_module("posture3d")
        L = importlib.import_module("ladder3d")
        r = Q.margin_residuals({"a": 4.0, "b": 4.7, "c": 5.0, "d": 9.0, "e": -1.0})
        self.assertAlmostEqual(sum(x * x for x in r), 37.09, delta=1e-9)
        self.assertEqual(sum(1 for x in r if x == 0.0), 2)
        base = PR.Ctx(**dict(L.options("r4"), contact="nearest"))
        mg = B.start_margins(base.m, base.st)
        self.assertAlmostEqual(sum(x * x for x in Q.margin_residuals(mg)), 1.27, delta=0.15)
        self.assertEqual(sum(1 for v in mg.values() if v < 4.9), 4)

    def test_what_the_ground_and_a_held_foot_give_is_the_change_of_the_bodys_turning_momentum(self):
        """The two readings against each other, by a law neither is built from: gravity has no moment about the centre
        of mass, so the whole body's turning momentum about the vertical through it changes only by the ground's twist
        (the feet's level forces and their own twist) and by whatever holds a foot's heading. On rung 4's body with both
        feet held (pelvis3d's replay FR) and its kept plan, 40 ms in the Python engine at the search's 0.25 ms step: the
        twist summed over the time (by the trapezium rule) is the momentum's change to 1.2% of the twists' summed sizes
        (0.7% at this step and half that at half the step: the rule's own error on the stiff contact forces). No part
        can be left out: the holds' share alone is more than ten times that allowance, and so are the level forces' and
        the feet's own twist. And the held feet do not turn (under 0.5 deg)."""
        import json
        import os
        PV = self._pelvis()
        PR = _practice()
        with open(os.path.join(os.path.dirname(os.path.abspath(PR.__file__)), "practice3d_s11_rung4.json")) as fh:
            d = json.load(fh)
        ctx, u = PV.build(d, "FR")
        m, s = ctx.m, ctx.m.sys
        st, aux = ctx.prepare(u)
        aux = list(aux)
        s.anchors(st, aux)
        h, n = ctx.h, 160
        bodies = list(range(s.nb))

        def reading(t, x):
            s.diag = {}
            s.deriv(t, x, aux)
            c, vc = B.com(m, x)
            gt = PV.ground_twist(m, x, [s.diag["contact"][cd[0]] for cd in s.contacts], c)
            parts = (sum(gt[sd][1] for sd in "LR"), sum(gt[sd][2] for sd in "LR"), sum(s.diag["stop"]["foothold_" + sd][0] for sd in "LR"))
            s.diag = None
            return parts, PV.turning_momentum(s, x, bodies, c, vc)
        prev, H0 = reading(PR.T0, st)
        given, size, x = [0.0, 0.0, 0.0], 0.0, list(st)
        f0 = {sd: PV.heading(m, st, "foot_" + sd) for sd in "LR"}
        for i in range(n):
            x = s.step(PR.T0 + i * h, x, aux, h)
            parts, H = reading(PR.T0 + (i + 1) * h, x)
            for k in range(3):
                given[k] += 0.5 * (parts[k] + prev[k]) * h
                size += 0.5 * (abs(parts[k]) + abs(prev[k])) * h
            prev = parts
        allow = 0.012 * size
        self.assertAlmostEqual(H - H0, sum(given), delta=allow)
        for part in given:                                    # the level forces' lever, the feet's own twist, the holds
            self.assertGreater(abs(part), 10.0 * allow)
        for sd in "LR":
            self.assertLess(abs(PV.wrap(PV.heading(m, x, "foot_" + sd) - f0[sd])), 0.5)


class StanceChecks(unittest.TestCase):
    """body-motion 4.13, rungs 3 and 4: the start on major leaguers' feet; the hips' turn by their own muscles."""

    @NEED
    def test_the_start_on_major_leaguers_feet_is_where_statcast_has_them(self):
        """body.START_STANCE. The placed ankle joints are 1.03 m apart along the line to the pitcher and level across it
        (Statcast's foot points at the intercept: 102.6 cm and 1 cm, note 15), the front foot points 44 deg toward the
        pitcher from square to the plate, the rear foot is flat; every joint is inside its anatomical end by at least the
        4.0 deg the rule reached (it asks 5: the rear ankle's roll, both hips sideways and the lead ankle's roll are the
        ones short, as stated with the stored result), and no stop is moved (add_ranges fails if the start is outside).
        On that start the practice search's ball is within 1.2 cm of level with the lead ankle, where Statcast has the
        intercept (-0.4 +- 9.4 cm; the 1.022 m from the rear ankle is not restated here, review P3-4). With BB_SLOW=1
        the rule is rerun on the given feet (7 minutes) and must return the stored numbers."""
        import os
        m = B.Model(ranges=(), grip="hands", feet="cleats", wrist="oval", shift=B.START_STANCE)
        st = B.place(m, T0)
        aL, aR = m.joint_pos(st, "ankle_L"), m.joint_pos(st, "ankle_R")
        self.assertAlmostEqual(aL[1] - aR[1], 1.03, delta=2e-3)
        self.assertAlmostEqual(aL[0] - aR[0], 0.0, delta=2e-3)
        fx = m.axis(st, "foot_L", (1, 0, 0))
        self.assertAlmostEqual(math.degrees(math.atan2(fx[1], fx[0])), 44.0, delta=0.5)
        self.assertLess(abs(m.axis(st, "foot_R", (0, 0, 1))[2] - 1.0), 1e-6)
        mg = B.start_margins(m, st)
        self.assertGreaterEqual(min(mg.values()), 4.0)
        self.assertEqual(sorted(k for k, v in mg.items() if v < 4.99), ["ankle_roll_L", "ankle_roll_R", "hip_abd_L", "hip_abd_R"])
        B.add_ranges(m, st)                                      # every stop at its anatomical end, the start inside
        PR = _practice()
        c = PR.Ctx(grip="hands", feet="cleats", hold="let_go", wrist="oval", clear=True, stance="mlb")
        aL, aR = c.m.joint_pos(c.st, "ankle_L"), c.m.joint_pos(c.st, "ankle_R")
        self.assertLess(abs(c.Pc[1] - aL[1]), 0.012)
        self.assertGreater(c.Pc[1] - aR[1], 0.9)
        self.assertAlmostEqual(c.Pc[2], 0.78, delta=1e-12)
        if os.environ.get("BB_SLOW"):
            given = {k: B.START_STANCE[k] for k in ("stride_cm", "stride_closed_cm", "lead_foot")}
            x, _ = B.clear_start(T0, given=given, grip="hands", feet="cleats", wrist="oval")
            self.assertEqual(x, B.START_STANCE)

    @NEED
    def test_the_feet_loaded_by_balance_carry_his_weight_and_no_more(self):
        """body-motion 4.13, rung 5 (built, not run in a search): Model(loads="balance") gives each foot the share of his
        weight that statics asks along the line of the feet, in place of [4]'s measured forces. On the start at rest:
        the ground's upward force, read from the engine's own contacts at the first instant, is his weight (92 kg x g)
        to a part in a thousand, where [4]'s loads at that instant are 0.79 of it; there is no level force; the lead
        foot's share is the lever rule worked here from the centre of mass and the two feet's points on the ground; and
        the centre of mass does not start to fall (under 0.2 m/s2, against 2.1 on [4]'s loads)."""
        got = {}
        for loads in ("measured", "balance"):
            m = B.Model(ranges=(), grip="hands", feet="cleats", wrist="oval", shift=B.START_STANCE, loads=loads)
            st = B.place(m, T0, moving=False)
            s = m.sys
            aux = list(m.start_aux)
            s.diag = {}
            ds = s.deriv(T0, st, aux)
            con = s.diag["contact"]
            s.diag = None
            N = {sd: sum(f[2] for n, f in con.items() if n[0] == sd) for sd in "LR"}
            level = max(abs(sum(f[k] for f in con.values())) for k in (0, 1))
            pts = {sd: [E.add(s.pose(st, c[1])[0], E.mv(E.qR(s.pose(st, c[1])[1]), c[2]))[1] for j, c in enumerate(s.contacts) if c[0][0] == sd and aux[j] is not None] for sd in "LR"}
            y = {sd: sum(v) / len(v) for sd, v in pts.items()}
            yc = B.com(m, st)[0][1]
            got[loads] = (N, level, (yc - y["R"]) / (y["L"] - y["R"]))
        BW = 92.0 * E.G
        N, level, share = got["balance"]
        self.assertAlmostEqual((N["L"] + N["R"]) / BW, 1.0, delta=1e-3)
        self.assertLess(level, 1e-6)
        self.assertAlmostEqual(N["L"] / (N["L"] + N["R"]), share, delta=5e-3)
        self.assertTrue(0.3 < share < 0.6)
        N0, level0, _ = got["measured"]
        self.assertLess((N0["L"] + N0["R"]) / BW, 0.85)
        self.assertGreater(level0, 50.0)

    @NEED
    def test_the_feet_push_apart_as_much_as_costs_the_hold_least(self):
        """rung 5: on the still start in balance, drive.solve_spread's push between the feet. The two feet's level forces,
        read from the engine's contacts, are equal and opposite along the line of the feet (the lead foot pushed toward
        the rear one) and inside the grip's cone; the hold-still torques' sum of squares is smaller at the solved push
        than at none and than at 30 N either side of it (worked here by solving the hold at each); and with the push the
        rear hip's adductors hold a wide base with a quarter of their strength where without it they could not hold it."""
        PR = _practice()
        o = dict(grip="hands", feet="cleats", hold="let_go", wrist="oval", clear=True, stance="mlb", stand="muscle", loads="balance", vel=0.0)
        c = PR.Ctx(**o)
        F, at_F, at_0 = c.spread
        self.assertGreater(F, 50.0)
        self.assertLess(at_F, 0.5 * at_0)
        m, s = c.m, c.m.sys
        st, aux = c.prepare(c.u_of({}))
        s.diag = {}
        s.deriv(PR.T0, st, aux)
        con = s.diag["contact"]
        s.diag = None
        fy = {sd: sum(f[1] for n, f in con.items() if n[0] == sd) for sd in "LR"}
        fz = {sd: sum(f[2] for n, f in con.items() if n[0] == sd) for sd in "LR"}
        self.assertAlmostEqual(fy["L"], -F, delta=1e-6 * F)
        self.assertAlmostEqual(fy["R"], F, delta=1e-6 * F)
        self.assertLess(F, c.m.mu * min(fz.values()))
        kt = s.contacts[0][5]
        idx = {sd: [j for j, cc in enumerate(s.contacts) if cc[0][0] == sd and c.aux0[j] is not None] for sd in "LR"}

        still = list(c.st)                                      # the same start with the holding drives let go, so that the
        oa = 13 * s.nb                                          # hold-still solve below is asked for the whole hold again
        for j in c.hold_ix.values():
            still[oa + j] = 0.0

        def hold_at(dF):
            a = list(c.aux0)
            for sd, sgn in (("L", -1.0), ("R", 1.0)):
                for j in idx[sd]:
                    a[j] = (a[j][0], a[j][1] + sgn * dF / len(idx[sd]) / kt)
            keep = list(s.tones)
            tone, _ = PR.Dr.solve_tone(m, still, PR.T0, aux=a)
            s.tones = keep
            return sum(PR.E.dot(v, v) for v in tone.values())
        here = hold_at(0.0)
        self.assertAlmostEqual(here, at_F, delta=1e-6 * at_F)
        self.assertGreater(hold_at(30.0), here)
        self.assertGreater(hold_at(-30.0), here)
        self.assertLess(c.hold_T["hip_add_R"] / c.bd["hip_add_R"], 0.3)

    def test_the_hips_active_turn_is_the_one_measured_with_the_hip_straight(self):
        """rung 4: the hips' ends of turn by their own muscles are Simoneau 1998's men measured prone, the hip straight
        (turning in 32 +- 9 deg, out 44 +- 7; note 15 C), not Boone & Azen's 44 and 44, which were measured seated with
        the hip bent to a right angle. With them the stops stand at those ends: for the lead (left) hip the turn in is
        the reading's positive end."""
        self.assertEqual(B.ROM_ACTIVE_HIP, {"hip_irot": 32.0, "hip_erot": 44.0})
        keep = dict(B.ROM)
        try:
            B.ROM.update(B.ROM_ACTIVE_HIP)
            specs = {t[0]: t for t in B.range_specs(None, "hands", "cleats", "oval")}
        finally:
            B.ROM.clear(); B.ROM.update(keep)
        self.assertEqual(specs["hip_rot_L"][6:8], (-44.0, 32.0))
        self.assertEqual(specs["hip_rot_R"][6:8], (-32.0, 44.0))
        self.assertEqual(B.ROM["hip_irot"], 32.6)                # the body's own table is as it was


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
    def test_spring_fed_the_wrong_joint_speed_fails_the_energy_account(self):
        """the spring advanced with the joint's speed reversed no longer carries the muscle's torque: the
        muscle's work and the delivered work plus stored energy part ways."""
        g0, w0, _, _ = tendon_ledger(2.5e-4)
        E.MUT.add("sec_sign")
        g1, w1, _, _ = tendon_ledger(2.5e-4)
        self.assertGreater(abs(g1), 10.0 * abs(g0))
        self.assertGreater(abs(g1), 0.1 * abs(w0))

    def test_floating_axis_the_wrong_way_round_fails_the_spherical_check(self):
        """the floating axis taken as (child's direction) x (parent's) lowers what it should raise."""
        E.MUT.add("float_swap")
        (th, ph), (rth, rph) = floating_axis_error()
        self.assertGreater(abs(th - rth), 0.3)

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
