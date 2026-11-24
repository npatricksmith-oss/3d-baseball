"""Independent oracle for the swing model v3 (docs/swing-model-v3.md), stdlib only.

Written from the spec's text (§4.2, §4.3, §5.2, §5.3, §6.1-§6.4, §8.2, §8.4). The contact jet
is v2's (swing_oracle.Plan); here its first two derivatives are taken analytically from v2's
rate bumps and checked in the tests against finite differences of v2's own positions.

Run from the repository root:
    python3 analysis/oracle/swing_v3.py           # writes golden/swing_v3.json
    python3 -m unittest analysis/oracle/test_oracle.py
"""
import math

try:
    from . import swing_oracle as v2
except ImportError:
    import swing_oracle as v2

DEG = math.pi / 180.0
MPH = 0.44704

# ---------------------------------------------------------------- small vector helpers
def add(*v): return tuple(sum(x) for x in zip(*v))
def sub(a, b): return tuple(x - y for x, y in zip(a, b))
def sc(a, s): return tuple(x * s for x in a)
def dot(a, b): return sum(x * y for x, y in zip(a, b))
def nrm(a): return math.sqrt(dot(a, a))
def unit(a): return sc(a, 1.0 / nrm(a))
def cross(a, b): return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])
def angle_deg(a, b): return math.acos(max(-1.0, min(1.0, dot(unit(a), unit(b))))) / DEG
def mirror(v): return (-v[0], v[1], v[2])


# ---------------------------------------------------------------- template (spec §4.3, §5, §5.2, §5.3, §6.1)
T4 = 0.170                 # [4]'s commitment-to-contact time (§4.3)
T4_93 = -0.093             # [4]'s mid-swing anchor (§5)
U_G = 0.08                 # grip station, m from the knob (§3)
BAT_L = 34 * 0.0254        # the 34-in bat
U_SS = BAT_L - 0.15        # sweet spot (as v2's goldens)
U_CM = 0.5977425305663189  # golden/bat_default.json
SEG = dict(upper_arm=0.186, forearm=0.146, half_shoulder=0.112, clavicle=0.28, hip=0.50)  # × H (§5.2, §5.3)
# Body knots (§5.3 table): pelvis forward of P (m), pelvis rise (m), trunk turn, flexion, lean (deg)
BODY = {'L': (0.12, 0.00, -22.0, 8.0, 10.0),
        '93': (0.17, 0.00, 2.0, 13.0, 23.0),
        'c': (0.17, 0.05, 75.0, 15.0, 31.0)}
# Arms at the knots ([4] §4.1.4-4.1.5): rear shoulder elevation, azimuth, rear elbow flexion,
# lead elbow flexion, target lag; bat elevation (deg).
ARMS = {'L': dict(rear_elev=71.0, rear_az=-78.0, rear_elbow=128.0, lead_elbow=72.0, lag=121.0, bat_elev=41.0),
        '93': dict(rear_elev=49.0, rear_az=-42.0, rear_elbow=126.0, lead_elbow=68.0, lag=131.0, bat_elev=39.0)}
A_LAUNCH = -225.0          # bat azimuth at launch, deg (§5)
SWEEP_93 = 168.0           # azimuth still to go at -93 ms (§5)
RATE_93 = 0.558            # azimuth rate at -93 ms over the contact rate (§5)
MISFIT_LAG_SD = 10.0       # deg (§6.1 step 3)
MISFIT_REACH_SD = 0.03     # × H
BRAKE_S = 0.040            # §8.2
THETA_SWING = 0.0          # deg, the ruling's threshold (§8.4)
TRACK_AFTER_S = 0.030      # the track ends at t_c + 0.03 s (D24)

# Stance spot of today's game (right-handed batter's frame).
P_TODAY = (-0.95, 0.30, 0.0)

# ---------------------------------------------------------------- linear algebra
def solve(M, y):
    """Gaussian elimination with partial pivoting (the oracle solves each system; the Rust
    uses the precomputed basis, §6.3)."""
    n = len(y)
    A = [list(row) + [y[i]] for i, row in enumerate(M)]
    for c in range(n):
        piv = max(range(c, n), key=lambda r: abs(A[r][c]))
        A[c], A[piv] = A[piv], A[c]
        for r in range(n):
            if r != c and A[r][c] != 0.0:
                f = A[r][c] / A[c][c]
                A[r] = [x - f * z for x, z in zip(A[r], A[c])]
    return [A[i][n] / A[i][i] for i in range(n)]


def poly_fit(conds):
    """conds: [(x, k, value)] meaning d^k p/dx^k (x) = value. Returns c_0..c_{n-1}."""
    n = len(conds)
    rows = []
    for x, k, _ in conds:
        rows.append([0.0 if j < k else math.factorial(j) / math.factorial(j - k) * x ** (j - k) for j in range(n)])
    return solve(rows, [c[2] for c in conds])


def poly_eval(c, x, k=0):
    s = 0.0
    for j in range(len(c) - 1, k - 1, -1):
        s = s * x + c[j] * math.factorial(j) / math.factorial(j - k)
    return s


# ---------------------------------------------------------------- [4]'s measured curves (§5.4, §6.3)
# [4] Appendix E, middle fastballs, digitised in step 0 (spec §5.4 table), [4]'s own time t₄ (ms).
T4_MS = (-170, -130, -110, -93, -80, -70, -60, -50, -40, -30, -25, -20, -15, -10, -5, 0)
AZ4_DEG = (-215, -199, -183, -163, -147, -132, -117, -102, -86, -65, -57, -47, -34, -22, -7, 0)  # azimuth − contact
EL4_DEG = (41.5, 42.2, 41.7, 39.1, 33.9, 27.6, 20.4, 10.5, 0.0, -11.1, -16.8, -21.2, -24.8, -27.3, -29.1, -29.7)
RATE4_DPS = (240, 625, 970, 1250, 1406, 1482, 1536, 1604, 1740, 1967, 2131, 2253, 2364, 2419, 2350, 2214)
LAG4_DEG = (120.7, 123.7, 127.4, 131.4, 132.7, 131.9, 129.2, 123.6, 114.9, 104.9, 97.7, 91.0, 84.0, 77.1, 70.5, 64.4)
T4_W = -30.0               # ms of [4]'s time: the elevation follows [4] to here, then turns to the jet (§6.3)


class NaturalSpline:
    """Natural cubic spline through (xs, ys) (second derivative zero at both ends), §6.3."""

    def __init__(self, xs, ys):
        n = len(xs)
        self.xs, self.ys = list(map(float, xs)), list(map(float, ys))
        h = [self.xs[i + 1] - self.xs[i] for i in range(n - 1)]
        M = [[0.0] * n for _ in range(n)]
        r = [0.0] * n
        M[0][0] = M[-1][-1] = 1.0
        for i in range(1, n - 1):
            M[i][i - 1], M[i][i], M[i][i + 1] = h[i - 1], 2 * (h[i - 1] + h[i]), h[i]
            r[i] = 6 * ((self.ys[i + 1] - self.ys[i]) / h[i] - (self.ys[i] - self.ys[i - 1]) / h[i - 1])
        self.m = solve(M, r)

    def __call__(self, x, k=0):
        xs, ys, m = self.xs, self.ys, self.m
        i = 0
        while i < len(xs) - 2 and x > xs[i + 1]:
            i += 1
        h = xs[i + 1] - xs[i]
        a, b = xs[i + 1] - x, x - xs[i]
        ca, cb = ys[i] / h - m[i] * h / 6, ys[i + 1] / h - m[i + 1] * h / 6
        if k == 0: return m[i] * a ** 3 / (6 * h) + m[i + 1] * b ** 3 / (6 * h) + ca * a + cb * b
        if k == 1: return -m[i] * a * a / (2 * h) + m[i + 1] * b * b / (2 * h) - ca + cb
        if k == 2: return m[i] * a / h + m[i + 1] * b / h
        raise ValueError(k)

    def antiderivative(self, x):
        """∫ from xs[0] to x, exact (each piece is a cubic)."""
        xs, ys, m = self.xs, self.ys, self.m
        total, i = 0.0, 0
        while i < len(xs) - 1:
            h = xs[i + 1] - xs[i]
            b = min(x, xs[i + 1]) - xs[i]
            if b <= 0: break
            a0 = h - b
            ca, cb = ys[i] / h - m[i] * h / 6, ys[i + 1] / h - m[i + 1] * h / 6
            F = lambda a, bb: -m[i] * a ** 4 / (24 * h) + m[i + 1] * bb ** 4 / (24 * h) - ca * a * a / 2 + cb * bb * bb / 2
            total += F(a0, b) - F(h, 0.0)
            if x <= xs[i + 1]: break
            i += 1
        return total

    def integral(self, x0, x1):
        return self.antiderivative(x1) - self.antiderivative(x0)


RATE4_RATIO = NaturalSpline(T4_MS, [r / RATE4_DPS[-1] for r in RATE4_DPS])  # r(t₄): rate / contact rate
EL4 = NaturalSpline(T4_MS, EL4_DEG)                                       # e(t₄), deg


# ---------------------------------------------------------------- the body (§5.3)
class Body:
    """The body at one knot, right-handed batter's frame."""

    def __init__(self, knot, H, P, params=None):
        fwd, rise, turn, flex, lean = params if params is not None else BODY[knot]
        self.O = (P[0], P[1] + fwd, SEG['hip'] * H + rise)
        ps, ff, fl = turn * DEG, flex * DEG, lean * DEG
        f = (math.cos(ps), math.sin(ps), 0.0)
        r = (math.sin(ps), -math.cos(ps), 0.0)
        z = (0.0, 0.0, 1.0)
        self.t = add(sc(add(sc(z, math.cos(ff)), sc(f, math.sin(ff))), math.cos(fl)), sc(r, math.sin(fl)))
        self.S = add(self.O, sc(self.t, SEG['clavicle'] * H))
        self.rp = unit(sub(r, sc(self.t, dot(r, self.t))))
        self.xT = cross(self.t, self.rp)
        self.yT = cross(self.t, self.xT)
        self.rear = add(self.S, sc(self.rp, SEG['half_shoulder'] * H))
        self.lead = sub(self.S, sc(self.rp, SEG['half_shoulder'] * H))
        self.H = H

    def upper_arm_dir(self, elev_deg, az_deg, reading=1):
        """Shoulder-to-elbow unit vector d (§4.2 reading 1; reading 2 reverses the horizontal part)."""
        th, ph = elev_deg * DEG, az_deg * DEG + (math.pi if reading == 2 else 0.0)
        horiz = add(sc(self.xT, math.cos(ph)), sc(self.yT, math.sin(ph)))
        return add(sc(self.t, -math.cos(th)), sc(horiz, math.sin(th)))

    def rear_elbow(self, elev_deg, az_deg, reading=1):
        d = self.upper_arm_dir(elev_deg, az_deg, reading)
        return add(self.rear, sc(d, SEG['upper_arm'] * self.H)), d

    def hands(self, elbow, d, flex_deg, chi):
        """W(chi), §6.1 step 2."""
        p = unit(sub(self.t, sc(d, dot(self.t, d))))
        q = cross(d, p)
        k = flex_deg * DEG
        fd = add(sc(d, math.cos(k)), sc(add(sc(p, math.cos(chi)), sc(q, math.sin(chi))), math.sin(k)))
        return add(elbow, sc(fd, SEG['forearm'] * self.H))


def lead_reach(H, flex_deg):
    lu, lf = SEG['upper_arm'] * H, SEG['forearm'] * H
    return math.sqrt(lu * lu + lf * lf + 2 * lu * lf * math.cos(flex_deg * DEG))


def bat_dir(A, E):
    """§4.2 (radians in)."""
    return (math.cos(E) * math.cos(A), math.cos(E) * math.sin(A), math.sin(E))


# ---------------------------------------------------------------- chi by least misfit (§6.1 steps 3-4)
def misfit_terms(knot, chi, H=1.85, P=P_TODAY, A_ref_deg=None):
    b = Body(knot, H, P)
    m = ARMS[knot]
    elbow, d = b.rear_elbow(m['rear_elev'], m['rear_az'])
    W = b.hands(elbow, d, m['rear_elbow'], chi)
    A = A_LAUNCH if knot == 'L' else A_ref_deg - SWEEP_93
    a = bat_dir(A * DEG, m['bat_elev'] * DEG)
    lag = angle_deg(sub(W, b.S), a)
    reach = nrm(sub(W, b.lead))
    R = lead_reach(H, m['lead_elbow'])
    cost = ((lag - m['lag']) / MISFIT_LAG_SD) ** 2 + ((reach - R) / (MISFIT_REACH_SD * H)) ** 2
    ahead = dot(sub(W, b.rear), b.xT)
    return dict(cost=cost, lag=lag, reach=reach, R=R, W=W, ahead=ahead, elbow=elbow, body=b)


def solve_chi(knot, H=1.85, P=P_TODAY, A_ref_deg=None, n=3600):
    """All local minima of M(chi) on a grid, each refined by golden section; keep those with the
    hands in front of the rear shoulder and return (chi of the lowest, list of minima)."""
    f = lambda c: misfit_terms(knot, c, H, P, A_ref_deg)['cost']
    grid = [2 * math.pi * i / n for i in range(n)]
    vals = [f(c) for c in grid]
    mins = []
    for i in range(n):
        if vals[i] <= vals[i - 1] and vals[i] <= vals[(i + 1) % n]:
            lo, hi = grid[i] - 2 * math.pi / n, grid[i] + 2 * math.pi / n
            g = (math.sqrt(5) - 1) / 2
            x1, x2 = hi - g * (hi - lo), lo + g * (hi - lo)
            f1, f2 = f(x1), f(x2)
            for _ in range(80):
                if f1 < f2:
                    hi, x2, f2 = x2, x1, f1
                    x1 = hi - g * (hi - lo); f1 = f(x1)
                else:
                    lo, x1, f1 = x1, x2, f2
                    x2 = lo + g * (hi - lo); f2 = f(x2)
            c = (0.5 * (lo + hi)) % (2 * math.pi)
            t = misfit_terms(knot, c, H, P, A_ref_deg)
            mins.append((t['cost'], c, t['ahead']))
    ok = sorted(m for m in mins if m[2] >= 0.0)
    return ok[0][1], sorted(mins)


MIDDLE_MIDDLE = lambda H, y=0.35: (0.0, y, (v2.TEMPLATE['zone_bottom_frac'] + 0.5 * (v2.TEMPLATE['zone_top_frac'] - v2.TEMPLATE['zone_bottom_frac'])) * H)


def template_A_ref(H=1.85):
    """Contact azimuth of the template's middle-middle aim (deg), §6.1 step 3."""
    j = Jet(v2.Plan(MIDDLE_MIDDLE(H), 0.0, 1, H, U_SS, 32.0), side=1)
    return j.A[0] / DEG


_CHI = {}
def template_chi():
    """χ_L, χ_93 (rad): template constants (§6.1 step 4), computed at H = 1.85 m, today's P."""
    if not _CHI:
        A_ref = template_A_ref()
        _CHI['A_ref'] = A_ref
        _CHI['L'] = solve_chi('L')[0]
        _CHI['93'] = solve_chi('93', A_ref_deg=A_ref)[0]
    return _CHI


def hands_anchor(knot, H, P):
    chi = template_chi()[knot]
    b = Body(knot, H, P)
    m = ARMS[knot]
    elbow, d = b.rear_elbow(m['rear_elev'], m['rear_az'])
    return b.hands(elbow, d, m['rear_elbow'], chi)


# ---------------------------------------------------------------- the contact jet (§6.2), from v2
class Jet:
    """Pose and first two derivatives at t_c of a v2 plan, in the batter's (right-handed) frame.
    A, E in radians; A on the atan2 branch (unwrapped against L by Swing)."""

    def __init__(self, plan, side=1, effort=0.0):
        p = plan
        e = 1.0 + effort
        n = p.n
        K = add(p.center, sc(p.e_c, p.rho))
        a = p.a_c
        ph1 = p.k * p.q * e
        th1 = p.k * e
        ph2 = ph1 * p.tau_h / p.w_h ** 2   # d/dτ of the Gaussian bump at τ = 0
        th2 = th1 * p.tau_b / p.w_b ** 2
        r = sub(K, p.center)
        nr = cross(n, r)
        K1 = sc(nr, ph1)
        K2 = add(sc(nr, ph2), sc(cross(n, nr), ph1 * ph1))
        na = cross(n, a)
        a1 = sc(na, th1)
        a2 = add(sc(na, th2), sc(cross(n, na), th1 * th1))
        W = add(K, sc(a, U_G)); W1 = add(K1, sc(a1, U_G)); W2 = add(K2, sc(a2, U_G))
        if side == -1:
            W, W1, W2, a, a1, a2 = (mirror(v) for v in (W, W1, W2, a, a1, a2))
        self.W = (W, W1, W2)
        self.axis = (a, a1, a2)
        self.A, self.E = angles_and_derivs(a, a1, a2)
        self.plan = p


    def yaw(self, g):
        """Stress test only (§11 case 8): turn the whole jet about the vertical through the hands,
        so its contact azimuth moves by g with nothing else changed."""
        c, s_ = math.cos(g), math.sin(g)
        rz = lambda v: (c * v[0] - s_ * v[1], s_ * v[0] + c * v[1], v[2])
        W0 = self.W[0]
        self.W = (W0, rz(self.W[1]), rz(self.W[2]))
        self.axis = tuple(rz(v) for v in self.axis)
        self.A, self.E = angles_and_derivs(*self.axis)


def angles_and_derivs(a, a1, a2):
    rho2 = a[0] ** 2 + a[1] ** 2
    N = a[0] * a1[1] - a[1] * a1[0]
    N1 = a[0] * a2[1] - a[1] * a2[0]
    r21 = 2 * (a[0] * a1[0] + a[1] * a1[1])
    A0 = math.atan2(a[1], a[0])
    A1 = N / rho2
    A2 = (N1 * rho2 - N * r21) / rho2 ** 2
    E0 = math.asin(a[2])
    c = math.sqrt(1 - a[2] ** 2)
    E1 = a1[2] / c
    E2 = a2[2] / c + a1[2] ** 2 * a[2] / c ** 3
    return (A0, A1, A2), (E0, E1, E2)


# ---------------------------------------------------------------- the swing (§6.3, §6.4, §8)
class Swing:
    """One v3 swing. aim, t_c in the sim frame; side +1 right-handed, -1 left-handed.

    path: 'spec' = §6.3 as written before step 1 (the curves of §5.4's comparison);
          'v3' = §6.3 as revised in step 1 (the elevation fix)."""

    def __init__(self, aim, t_c=0.40, side=1, H=1.85, bat_speed=32.0, t_swing=0.150, P=None, alpha=None,
                 effort=0.0, path='v3', plan=None, jet_yaw_deg=0.0):
        self.side, self.H, self.t_c, self.T = side, H, t_c, t_swing
        self.sT = t_swing / T4
        self.tau93 = T4_93 * self.sT
        P = P if P is not None else ((P_TODAY[0] * side, P_TODAY[1], P_TODAY[2]))
        self.P = (P[0] * side, P[1], P[2])  # batter's frame
        self.plan = plan if plan is not None else v2.Plan(aim, t_c, side, H, U_SS, bat_speed, t_swing=t_swing, alpha=alpha)
        self.effort = effort
        self.jet = Jet(self.plan, side, effort)
        if jet_yaw_deg:
            self.jet.yaw(jet_yaw_deg * DEG)
        self.bodies = {k: Body(k, H, self.P) for k in BODY}
        self.W_L = hands_anchor('L', H, self.P)
        self.W_93 = hands_anchor('93', H, self.P)
        self.A_L, self.E_L = A_LAUNCH * DEG, ARMS['L']['bat_elev'] * DEG
        (A0, A1, A2), (E0, E1, E2) = self.jet.A, self.jet.E
        # Unwrapped from L: the contact azimuth lies within (A_L + 180°, A_L + 540°]: a bat
        # cannot meet the ball pointing back at the catcher.
        while A0 <= self.A_L + math.pi: A0 += 2 * math.pi
        while A0 > self.A_L + 3 * math.pi: A0 -= 2 * math.pi
        self.A_c = (A0, A1, A2)
        self.E_c = (E0, E1, E2)
        self.path = path
        self._build()

    # Conditions are written in u = τ/T_swing (derivative k scaled by T^k), §6.3 closed form.
    def _conds_A(self):
        T, u0, u93 = self.T, -1.0, self.tau93 / self.T
        A0, A1, A2 = self.A_c
        c = [(u0, 0, self.A_L), (u0, 1, 0.0), (u0, 2, 0.0),
             (u93, 0, A0 - SWEEP_93 * DEG), (u93, 1, RATE_93 * A1 * T),
             (0.0, 0, A0), (0.0, 1, A1 * T), (0.0, 2, A2 * T * T)]
        return c

    def _conds_E(self):
        T, u0, u93 = self.T, -1.0, self.tau93 / self.T
        E0, E1, E2 = self.E_c
        return [(u0, 0, self.E_L), (u0, 1, 0.0), (u0, 2, 0.0), (u93, 0, ARMS['93']['bat_elev'] * DEG),
                (0.0, 0, E0), (0.0, 1, E1 * T), (0.0, 2, E2 * T * T)]

    def _conds_W(self, i):
        T, u0, u93 = self.T, -1.0, self.tau93 / self.T
        W0, W1, W2 = self.jet.W
        return [(u0, 0, self.W_L[i]), (u0, 1, 0.0), (u0, 2, 0.0), (u93, 0, self.W_93[i]),
                (0.0, 0, W0[i]), (0.0, 1, W1[i] * T), (0.0, 2, W2[i] * T * T)]

    def _build(self):
        self.cA = poly_fit(self._conds_A())
        self.cE = poly_fit(self._conds_E())
        self.cW = [poly_fit(self._conds_W(i)) for i in range(3)]
        # S(τ): quadratic through the three knots (Lagrange), §5.3.
        self.S_knots = [(-self.T, self.bodies['L'].S), (self.tau93, self.bodies['93'].S), (0.0, self.bodies['c'].S)]
        if self.path == 'v3':
            self._build_v3()

    # ---- §6.3 as revised in step 1: [4]'s measured curves from −93 ms, joined to the jet -------
    # In u = τ/T_swing, [4]'s time is t₄ = 170·u ms (τ = s_T·t₄ and T_swing = s_T·T₄).
    def _baseA(self, u, k=0):
        A0, A1, _ = self.A_c
        t = 1e3 * T4 * u
        if k == 0: return A0 + A1 * self.T / (1e3 * T4) * RATE4_RATIO.integral(0.0, t)
        if k == 1: return A1 * self.T * RATE4_RATIO(t)
        return A1 * self.T * 1e3 * T4 * RATE4_RATIO(t, 1)

    def _baseE(self, u, k=0):
        t = 1e3 * T4 * u
        e93 = EL4(1e3 * T4_93)
        if k == 0: return ARMS['93']['bat_elev'] * DEG + self.kappa * (EL4(t) - e93) * DEG
        return self.kappa * (1e3 * T4) ** k * EL4(t, k) * DEG

    def _build_v3(self):
        T, u93, uw = self.T, self.tau93 / self.T, T4_W / (1e3 * T4)
        self.u93, self.uw = u93, uw
        A0, A1, A2 = self.A_c
        E0, E1, E2 = self.E_c
        self.kappa = (E0 - ARMS['93']['bat_elev'] * DEG) / ((EL4(0.0) - EL4(1e3 * T4_93)) * DEG)
        bA, bE = self._baseA, self._baseE
        self.dA = poly_fit([(u93, 0, A0 - SWEEP_93 * DEG - bA(u93)), (u93, 1, 0.0), (u93, 2, 0.0),
                            (0.0, 0, A0 - bA(0.0)), (0.0, 1, A1 * T - bA(0.0, 1)), (0.0, 2, A2 * T * T - bA(0.0, 2))])
        self.dE = poly_fit([(uw, 0, 0.0), (uw, 1, 0.0), (uw, 2, 0.0),
                            (0.0, 0, E0 - bE(0.0)), (0.0, 1, E1 * T - bE(0.0, 1)), (0.0, 2, E2 * T * T - bE(0.0, 2))])
        self.c1A = poly_fit([(-1.0, 0, self.A_L), (-1.0, 1, 0.0), (-1.0, 2, 0.0)] + [(u93, k, self._seg2A(u93, k)) for k in range(3)])
        self.c1E = poly_fit([(-1.0, 0, self.E_L), (-1.0, 1, 0.0), (-1.0, 2, 0.0)] + [(u93, k, self._seg2E(u93, k)) for k in range(3)])

    def _seg2A(self, u, k=0):
        return self._baseA(u, k) + poly_eval(self.dA, u, k)

    def _seg2E(self, u, k=0):
        return self._baseE(u, k) + (poly_eval(self.dE, u, k) if u > self.uw else 0.0)

    def curves(self, u, k=0):
        """A, E (rad) and their k-th derivatives in u, on [−1, 0]."""
        if self.path != 'v3':
            return poly_eval(self.cA, u, k), poly_eval(self.cE, u, k)
        if u < self.u93:
            return poly_eval(self.c1A, u, k), poly_eval(self.c1E, u, k)
        return self._seg2A(u, k), self._seg2E(u, k)

    # ---- the batter's frame (right-handed) ------------------------------------------------
    def S_rh(self, tau, k=0):
        return self._lagrange(self.S_knots, tau, k)

    def O_rh(self, tau):
        """Mid-hips, the same quadratic through the knots (used only by the body-clearance check)."""
        return self._lagrange([(t, self.bodies[n].O) for (t, _), n in zip(self.S_knots, ('L', '93', 'c'))], tau, 0)

    @staticmethod
    def _lagrange(knots, tau, k=0):
        out = [0.0, 0.0, 0.0]
        ts = [t for t, _ in knots]
        for i, (ti, Si) in enumerate(knots):
            o = [tj for j, tj in enumerate(ts) if j != i]
            den = (ti - o[0]) * (ti - o[1])
            if k == 0: w = (tau - o[0]) * (tau - o[1]) / den
            elif k == 1: w = (2 * tau - o[0] - o[1]) / den
            else: w = 2.0 / den
            out = [x + w * s for x, s in zip(out, Si)]
        return tuple(out)

    def _v2_state(self, tau):
        """v2's analytic swing after contact (§6.4), batter's frame: (A, E, W) and rates."""
        p = self.plan
        e = 1.0 + self.effort
        t = self.t_c + tau
        dphi, dth = p.angles(t)
        K = add(p.center, sc(v2.rotate(p.e_c, p.n, dphi * e), p.rho))
        a = v2.rotate(p.a_c, p.n, dth * e)
        ph1, th1 = p.hand_rate(t) * e, p.bat_rate(t) * e
        K1 = sc(cross(p.n, sub(K, p.center)), ph1)
        a1 = sc(cross(p.n, a), th1)
        W, W1 = add(K, sc(a, U_G)), add(K1, sc(a1, U_G))
        if self.side == -1:
            W, W1, a, a1 = (mirror(v) for v in (W, W1, a, a1))
        return W, W1, a, a1

    def state_rh(self, tau):
        """(A, Adot, E, Edot, W, Wdot) in the batter's frame at τ (no brake)."""
        if tau <= 0.0:
            u = tau / self.T
            A, E = self.curves(u)
            A1, E1 = (x / self.T for x in self.curves(u, 1))
            W = tuple(poly_eval(c, u) for c in self.cW)
            W1 = tuple(poly_eval(c, u, 1) / self.T for c in self.cW)
            return A, A1, E, E1, W, W1
        W, W1, a, a1 = self._v2_state(tau)
        (A, A1, _), (E, E1, _) = angles_and_derivs(a, a1, (0.0, 0.0, 0.0))
        while A < self.A_c[0] - math.pi: A += 2 * math.pi
        while A > self.A_c[0] + math.pi: A -= 2 * math.pi
        return A, A1, E, E1, W, W1

    # ---- the clock (§8.2) -------------------------------------------------------------------
    def sigma(self, tau, tau_b=None):
        if tau_b is None or tau <= tau_b:
            return tau, 1.0
        s = min(tau - tau_b, BRAKE_S)
        return tau_b + s - s * s / (2 * BRAKE_S), 1.0 - s / BRAKE_S

    # ---- world frame output -------------------------------------------------------------------
    def pose(self, t, tau_b=None):
        """knob, axis, Omega, centre-of-mass velocity, hands W, S at sim time t (world frame)."""
        tau, rate = self.sigma(t - self.t_c, tau_b)
        A, A1, E, E1, W, W1 = self.state_rh(tau)
        A1, E1, W1 = A1 * rate, E1 * rate, sc(W1, rate)
        a = bat_dir(A, E)
        a1 = (-math.cos(E) * math.sin(A) * A1 - math.sin(E) * math.cos(A) * E1,
              math.cos(E) * math.cos(A) * A1 - math.sin(E) * math.sin(A) * E1,
              math.cos(E) * E1)
        Om = cross(a, a1)
        Vcm = add(W1, cross(Om, sc(a, U_CM - U_G)))
        knob = sub(W, sc(a, U_G))
        S = self.S_rh(tau)
        if self.side == -1:
            knob, a, W, S, Vcm = (mirror(v) for v in (knob, a, W, S, Vcm))
            Om = (Om[0], -Om[1], -Om[2])  # a pseudovector under the mirror
        return dict(knob=knob, axis=a, omega=Om, v_cm=Vcm, hands=W, S=S, A=A, E=E)

    def point(self, t, u, tau_b=None):
        p = self.pose(t, tau_b)
        return add(p['knob'], sc(p['axis'], u))

    def point_velocity(self, t, u, tau_b=None):
        p = self.pose(t, tau_b)
        W1 = sub(p['v_cm'], cross(p['omega'], sc(p['axis'], U_CM - U_G)))
        return add(W1, cross(p['omega'], sc(p['axis'], u - U_G)))

    # ---- the ruling (§8.4) --------------------------------------------------------------------
    def A_max(self, tau_b=None):
        """Furthest unwrapped azimuth reached (deg) by the end of the track; A is non-decreasing,
        so it is the azimuth at the last instant (a braked swing: where it stopped)."""
        end = TRACK_AFTER_S if tau_b is None else min(TRACK_AFTER_S, tau_b + BRAKE_S)
        return self.state_rh(self.sigma(end, tau_b)[0])[0] / DEG

    def judged_swing(self, tau_b):
        return self.A_max(tau_b) >= THETA_SWING

    # ---- checks (§11) -------------------------------------------------------------------------
    def lag(self, tau):
        A, _, E, _, W, _ = self.state_rh(tau)
        return angle_deg(sub(W, self.S_rh(tau)), bat_dir(A, E))


# ---------------------------------------------------------------- diagnostics (§6.3 verification, §11)
TORSO_R = 0.10   # m [E], half the chest's depth: the trunk as a capsule from the mid-hips to the clavicle
HEAD_R = 0.10    # m [E]
HEAD_UP = 0.13   # × H [E], clavicle to the head's centre, straight up


def seg_seg_dist(p0, p1, q0, q1, n=40):
    """Least distance between two segments (sampled on one, exact on the other)."""
    d = sub(q1, q0)
    L2 = dot(d, d)
    best = 1e9
    for i in range(n + 1):
        x = add(p0, sc(sub(p1, p0), i / n))
        t = max(0.0, min(1.0, dot(sub(x, q0), d) / L2))
        best = min(best, nrm(sub(x, add(q0, sc(d, t)))))
    return best


def pt_seg_dist(x, q0, q1):
    d = sub(q1, q0)
    t = max(0.0, min(1.0, dot(sub(x, q0), d) / dot(d, d)))
    return nrm(sub(x, add(q0, sc(d, t))))


def plane_tilt_deg(points):
    """Tilt of the least-squares plane through points from the horizontal (Jacobi eigen)."""
    n = len(points)
    c = sc(add(*points), 1.0 / n)
    C = [[sum((p[i] - c[i]) * (p[j] - c[j]) for p in points) for j in range(3)] for i in range(3)]
    V = [[1.0 if i == j else 0.0 for j in range(3)] for i in range(3)]
    for _ in range(60):
        i, j = max(((0, 1), (0, 2), (1, 2)), key=lambda ij: abs(C[ij[0]][ij[1]]))
        if abs(C[i][j]) < 1e-18: break
        th = 0.5 * math.atan2(2 * C[i][j], C[j][j] - C[i][i])
        cs, sn = math.cos(th), math.sin(th)
        for k in range(3):
            cik, cjk = C[i][k], C[j][k]
            C[i][k], C[j][k] = cs * cik - sn * cjk, sn * cik + cs * cjk
        for k in range(3):
            cki, ckj = C[k][i], C[k][j]
            C[k][i], C[k][j] = cs * cki - sn * ckj, sn * cki + cs * ckj
        for k in range(3):
            vki, vkj = V[k][i], V[k][j]
            V[k][i], V[k][j] = cs * vki - sn * vkj, sn * vki + cs * vkj
    m = min(range(3), key=lambda k: C[k][k])
    normal = (V[0][m], V[1][m], V[2][m])
    return math.acos(min(1.0, abs(normal[2]) / nrm(normal))) / DEG


def diagnostics(sw, dt=0.0005):
    """Everything §6.3 and §11 check, on one swing (batter's frame, deg, s, m)."""
    T, sT = sw.T, sw.sT
    n = int(round(T / dt))
    taus = [-T + i * dt for i in range(n + 1)]
    st = [sw.state_rh(t) for t in taus]
    v0 = nrm(sw.point_velocity(sw.t_c, U_SS))
    vt0 = nrm(sw.point_velocity(sw.t_c, BAT_L))
    rel = lambda u: [nrm(sw.point_velocity(sw.t_c + t, u)) for t in taus[:-1]]
    ss, tip = rel(U_SS), rel(BAT_L)
    iss = max(range(len(ss)), key=lambda i: ss[i])
    itp = max(range(len(tip)), key=lambda i: tip[i])
    Ad = [x[1] / DEG for x in st]
    Es = [x[2] / DEG for x in st]
    lag = [angle_deg(sub(x[4], sw.S_rh(t)), bat_dir(x[0], x[2])) for x, t in zip(st, taus)]
    ilm = max(range(len(lag)), key=lambda i: lag[i])
    lagr = [(lag[i + 1] - lag[i - 1]) / (2 * dt) for i in range(1, n)]
    iun = min(range(len(lagr)), key=lambda i: lagr[i])
    def armaz(i):
        a = sub(st[i][4], sw.S_rh(taus[i])); return math.atan2(a[1], a[0])
    arm = []
    for i in range(1, n):
        d = armaz(i + 1) - armaz(i - 1)
        d = (d + math.pi) % (2 * math.pi) - math.pi
        arm.append(d / (2 * dt) / DEG)
    iar = max(range(len(arm)), key=lambda i: arm[i])
    tipv = [sw.point_velocity(sw.t_c + t, BAT_L) for t in taus[:-1]]
    if sw.side == -1:
        tipv = [mirror(v) for v in tipv]
    ix = max(range(len(tipv)), key=lambda i: tipv[i][0])
    iz = min(range(len(tipv)), key=lambda i: tipv[i][2])
    reach = [nrm(sub(x[4], sw.S_rh(t))) for x, t in zip(st, taus)]
    clear_torso, clear_head = 1e9, 1e9
    for x, t in list(zip(st, taus))[::4]:
        a = bat_dir(x[0], x[2])
        knob, tipp = sub(x[4], sc(a, U_G)), add(x[4], sc(a, BAT_L - U_G))
        S, O = sw.S_rh(t), sw.O_rh(t)
        clear_torso = min(clear_torso, seg_seg_dist(knob, tipp, O, S) - TORSO_R)
        head = add(S, (0.0, 0.0, HEAD_UP * sw.H))
        clear_head = min(clear_head, pt_seg_dist(head, knob, tipp) - HEAD_R)
    bc = sw.bodies['c']
    Wc = st[-1][4]
    pts40 = []
    for i in range(41):
        p = sw.point(sw.t_c - 0.040 + i * 0.001, U_SS)
        pts40.append(p)
    head_len = 0.0
    prev = sw.point(sw.t_c - 0.065, BAT_L)
    for i in range(1, 131):
        q = sw.point(sw.t_c - 0.065 + i * 0.0005, BAT_L)
        head_len += nrm(sub(q, prev)); prev = q
    Adc = sw.A_c[1] / DEG
    return dict(
        A_c=sw.A_c[0] / DEG, E_c=sw.E_c[0] / DEG, Adot_c=Adc,
        min_Adot=min(Ad), E_min=min(Es), E_max=max(Es),
        ss_ratio_max=ss[iss] / v0, ss_ratio_tau=taus[iss], tip_ratio_max=tip[itp] / vt0, tip_ratio_tau=taus[itp],
        ss_mph_c=v0 / MPH,
        reach_min=min(reach), reach_max=max(reach),
        clear_torso=clear_torso, clear_head=clear_head,
        lag_L=lag[0], lag_93=sw.lag(sw.tau93), lag_c=lag[-1], lag_max=lag[ilm], lag_max_tau=taus[ilm],
        uncock_peak=-lagr[iun], uncock_tau=taus[iun + 1],
        arm_rate_peak=arm[iar], arm_rate_tau=taus[iar + 1],
        tip_toward_plate_peak=tipv[ix][0], tip_toward_plate_tau=taus[ix],
        tip_down_peak=-tipv[iz][2], tip_down_tau=taus[iz],
        hands_ahead_c=dot(sub(Wc, sw.S_rh(0.0)), bc.xT),
        plane_tilt_40ms=plane_tilt_deg(pts40), beta=sw.plan.beta / DEG,
        head_travel_65ms_ft=head_len / 0.3048,
        hands_speed_c=nrm(sw.jet.W[1]), knob_speed_c=nrm(sub(sw.jet.W[1], sc(sw.jet.axis[1], U_G))),
    )


# ---------------------------------------------------------------- the aim range (§11 case 8)
FULL_REACH = SEG['half_shoulder'] + SEG['upper_arm'] + SEG['forearm']   # × H, clavicle to the hands, arms straight
SS_RATIO_MAX = 1.10    # §6.3 [E]: the sweet spot's speed before contact, against its contact value
TIP_RATIO_MAX = 1.15   # [3]'s shape bound (§11): the tip's speed peak before contact
E_RANGE = (-45.0, 50.0)


def range_failures(d, H=1.85):
    """The §6.3 verification list for one swing; returns the names of the checks it fails."""
    bad = []
    if not all(math.isfinite(v) for v in d.values()): bad.append('finite')
    if d['min_Adot'] < -1e-6: bad.append('A non-decreasing')
    if not (E_RANGE[0] <= d['E_min'] and d['E_max'] <= E_RANGE[1]): bad.append('E range')
    if d['ss_ratio_max'] > SS_RATIO_MAX: bad.append('sweet-spot speed')
    if d['tip_ratio_max'] > TIP_RATIO_MAX: bad.append('tip speed')
    if not (0.15 <= d['reach_min'] and d['reach_max'] <= FULL_REACH * H): bad.append('hands within reach')
    if d['clear_torso'] < 0.0 or d['clear_head'] < 0.0: bad.append('bat through the body')
    return bad


def range_cases(coarse=False):
    """Aims over the baseline's whole range (x ±0.45 m of the plate's centre, from 0.15 m below the
    zone to 0.15 m above it, the zone's depth rule), planned attack angle ±4°, T_swing 0.130-0.174 s,
    effort ±10%, and the jet turned to contact azimuths −25° to +50° (stress)."""
    H = 1.85
    xs = (-0.45, 0.0, 0.45) if coarse else (-0.45, -0.30, -0.15, 0.0, 0.15, 0.30, 0.45)
    zs = (0.35, 0.75, 1.14) if coarse else (0.35, 0.55, 0.75, 0.95, 1.14)
    out = []
    for x in xs:
        for z in zs:
            aim = (x, 0.35 - 0.125 * x, z)
            for T in ((0.150,) if coarse else (0.130, 0.150, 0.174)):
                base = v2.Plan(aim, 0.40, 1, H, U_SS, 32.0, t_swing=T)
                for dal in ((0.0,) if coarse else (-4.0, 0.0, 4.0)):
                    kw = dict(aim=aim, t_swing=T, alpha=None if dal == 0.0 else base.alpha + dal * DEG)
                    out.append(kw)
    if not coarse:
        for e in (-0.10, 0.10):
            for aim in ((0.0, 0.35, 0.745), (-0.45, 0.406, 0.35), (0.45, 0.294, 1.14)):
                out.append(dict(aim=aim, effort=e))
        for aim in ((0.0, 0.35, 0.745), (0.45, 0.406, 0.35), (-0.45, 0.294, 1.14)):
            out.append(dict(aim=aim, side=-1))
    for target in range(-25, 51, 15 if coarse else 5):
        for aim in (((0.0, 0.35, 0.745),) if coarse else ((0.0, 0.35, 0.745), (-0.20, 0.375, 0.56), (0.22, 0.3225, 0.93))):
            a0 = Jet(v2.Plan(aim, 0.40, 1, H, U_SS, 32.0)).A[0] / DEG
            out.append(dict(aim=aim, jet_yaw_deg=target - a0))
    return out


def aim_range(coarse=False):
    rows = []
    for kw in range_cases(coarse):
        sw = Swing(**kw)
        d = diagnostics(sw, dt=0.001)
        rows.append(dict(case={k: (list(v) if isinstance(v, tuple) else v) for k, v in kw.items()}, failures=range_failures(d), **d))
    return rows


# ---------------------------------------------------------------- bounds from the sources (§11)
GOLDEN_AIMS = {'rhb_middle': (0.0, 0.35, 0.745), 'rhb_inside_low': (-0.20, 0.375, 0.56), 'rhb_outside_high': (0.22, 0.3225, 0.93)}
# [4] Table 1's five locations, on the golden aims' columns and rows.
LOCATION_AIMS = {'MIDDLE': (0.0, 0.35, 0.745), 'HIGH IN': (-0.20, 0.375, 0.93), 'HIGH OUT': (0.22, 0.3225, 0.93),
                 'LOW IN': (-0.20, 0.375, 0.56), 'LOW OUT': (0.22, 0.3225, 0.56)}
LAG_AT_CONTACT_4 = {'MIDDLE': 65.0, 'HIGH IN': 79.0, 'HIGH OUT': 63.0, 'LOW IN': 58.0, 'LOW OUT': 51.0}


def bounds(sw, d=None):
    """Each §11 bound on one swing: (name, value, low, high, passed)."""
    d = d or diagnostics(sw)
    sT, Adc = sw.sT, sw.A_c[1] / DEG
    out = []
    def chk(name, v, lo, hi):
        out.append(dict(check=name, value=v, low=lo, high=hi, passed=bool(lo <= v <= hi)))
    chk('lag maximum (deg)', d['lag_max'], 133 - 8, 133 + 8)
    chk('lag maximum time (ms)', d['lag_max_tau'] * 1e3, -80 * sT - 20, -80 * sT + 20)
    u = 1588 * Adc / 2233
    chk('uncocking peak (deg/s)', d['uncock_peak'], 0.75 * u, 1.25 * u)
    chk('uncocking peak time (ms)', d['uncock_tau'] * 1e3, -20 * sT - 15, -20 * sT + 15)
    a = 1160 * Adc / 2233
    chk('arm-vector turning peak (deg/s)', d['arm_rate_peak'], 0.75 * a, 1.25 * a)
    chk('arm-vector turning peak time (ms)', d['arm_rate_tau'] * 1e3, -65 * sT - 20, -65 * sT + 20)
    chk('tip speed peak / contact', d['tip_ratio_max'], 1.0, 1.15)
    chk('tip speed peak time (ms)', d['tip_ratio_tau'] * 1e3, -30.0, -0.5)
    chk('tip toward the plate peak time (ms)', d['tip_toward_plate_tau'] * 1e3, -40 * sT - 15, -40 * sT + 15)
    chk('tip downward peak time (ms)', d['tip_down_tau'] * 1e3, -40 * sT - 15, -40 * sT + 15)
    chk('hands ahead of the clavicle at contact (m)', d['hands_ahead_c'], 0.25, 9.0)
    chk('plane tilt over the last 40 ms - beta (deg)', d['plane_tilt_40ms'] - d['beta'], -1.5, 1.5)
    chk('head travel in the last 65 ms (ft)', d['head_travel_65ms_ft'], 7.3 - 0.7, 7.3 + 0.7)
    chk('knob (hands) speed at contact (m/s)', d['knob_speed_c'], 6.0, 9.0)
    return out


def appendix_e_compare(sw):
    """v3 against [4]'s Appendix E table at τ = s_T·t₄ (middle pitch): azimuth − contact, elevation,
    rate / contact rate, lag."""
    rows = []
    for t4, az, el, rt, lg in zip(T4_MS, AZ4_DEG, EL4_DEG, RATE4_DPS, LAG4_DEG):
        tau = sw.sT * t4 * 1e-3
        A, A1, E, _, _, _ = sw.state_rh(tau)
        rows.append(dict(t4=t4, az4=az, el4=el, rate4=rt / RATE4_DPS[-1], lag4=lg,
                         az=(A - sw.A_c[0]) / DEG, el=E / DEG, rate=A1 / sw.A_c[1], lag=sw.lag(tau)))
    return rows


def v2_difference(sw):
    """§6.5: v3 − v2 azimuth and elevation, sweet-spot speeds, before contact."""
    p = sw.plan
    out = []
    for tau in (-0.025, -0.015, -0.010, -0.005):
        k, a = p.knob_axis(sw.t_c + tau)
        if sw.side == -1: a = mirror(a)
        A2, E2 = math.atan2(a[1], a[0]), math.asin(a[2])
        A, _, E, _, _, _ = sw.state_rh(tau)
        while A2 < A - math.pi: A2 += 2 * math.pi
        while A2 > A + math.pi: A2 -= 2 * math.pi
        out.append(dict(tau_ms=tau * 1e3, dA=(A - A2) / DEG, dE=(E - E2) / DEG,
                        ss_v3_mph=nrm(sw.point_velocity(sw.t_c + tau, U_SS)) / MPH,
                        ss_v2_mph=nrm(p.velocity(sw.t_c + tau, U_SS)) / MPH))
    return out


# ---------------------------------------------------------------- the golden file (§11)
def _v(x): return [float(c) for c in x]


def basis_matrix(nodes):
    """Coefficients (in u) of the polynomial meeting conditions [(u, k)] as columns: entry [j][i] is
    c_j's weight on condition i. The Rust stores these constants (§6.3, §9)."""
    n = len(nodes)
    cols = []
    for i in range(n):
        e = [0.0] * n; e[i] = 1.0
        cols.append(poly_fit([(u, k, e[m]) for m, (u, k) in enumerate(nodes)]))
    return [[cols[i][j] for i in range(n)] for j in range(n)]


def golden():
    H, P = 1.85, P_TODAY
    chi = template_chi()
    u93, uw = T4_93 / T4, T4_W / (1e3 * T4)
    meta = dict(
        spec='docs/swing-model-v3.md (build step 1)', H=H, stance_P=list(P), u_g=U_G, u_ss=U_SS, u_cm=U_CM, bat_length=BAT_L,
        T4=T4, t4_93=T4_93, t4_w_ms=T4_W, sweep_93_deg=SWEEP_93, launch_azimuth_deg=A_LAUNCH, segments=SEG,
        body_knots={k: list(v) for k, v in BODY.items()}, arms=ARMS,
        chi_L_deg=chi['L'] / DEG, chi_93_deg=chi['93'] / DEG, A_ref_deg=chi['A_ref'],
        appendix_e=dict(t4_ms=list(T4_MS), rate_dps=list(RATE4_DPS), elevation_deg=list(EL4_DEG),
                        rate_ratio_spline_m=RATE4_RATIO.m, elevation_spline_m=EL4.m,
                        rate_ratio_integral_from_0=[RATE4_RATIO.integral(0.0, t) for t in T4_MS]),
        basis=dict(
            u93=u93, uw=uw,
            launch_quintic=dict(nodes=[[-1.0, 0], [-1.0, 1], [-1.0, 2], [u93, 0], [u93, 1], [u93, 2]],
                                coeffs=basis_matrix([(-1.0, 0), (-1.0, 1), (-1.0, 2), (u93, 0), (u93, 1), (u93, 2)])),
            azimuth_correction=dict(nodes=[[u93, 0], [u93, 1], [u93, 2], [0.0, 0], [0.0, 1], [0.0, 2]],
                                    coeffs=basis_matrix([(u93, 0), (u93, 1), (u93, 2), (0.0, 0), (0.0, 1), (0.0, 2)])),
            elevation_correction=dict(nodes=[[uw, 0], [uw, 1], [uw, 2], [0.0, 0], [0.0, 1], [0.0, 2]],
                                      coeffs=basis_matrix([(uw, 0), (uw, 1), (uw, 2), (0.0, 0), (0.0, 1), (0.0, 2)])),
            hands_sextic=dict(nodes=[[-1.0, 0], [-1.0, 1], [-1.0, 2], [u93, 0], [0.0, 0], [0.0, 1], [0.0, 2]],
                              coeffs=basis_matrix([(-1.0, 0), (-1.0, 1), (-1.0, 2), (u93, 0), (0.0, 0), (0.0, 1), (0.0, 2)]))),
        note='Angles in degrees where named _deg, else radians; positions m; times s unless _ms; sim frame.')
    # 1. the hand-worked case
    hc = Body(None, H, (0.0, 0.0, 0.0), params=(0.0, -0.0, 0.0, 0.0, 0.0))
    hand_case = dict(inputs='S = (0,0,0) (O shifted down), upright trunk facing +x, rear arm θ 90° φ −90°, κ 90°, χ 270°, bat A −90° E 45°')
    for reading in (1, 2):
        S0 = hc.S
        shift = lambda v: sub(v, S0)
        elbow, d = hc.rear_elbow(90.0, -90.0, reading)
        W = hc.hands(elbow, d, 90.0, 270.0 * DEG)
        hand_case['reading_%d' % reading] = dict(elbow=_v(shift(elbow)), W=_v(shift(W)),
                                                  lag_deg=angle_deg(sub(W, S0), bat_dir(-90 * DEG, 45 * DEG)))
    # 2. body, hands, the reading check
    body = {}
    for k in ('L', '93', 'c'):
        b = Body(k, H, P)
        body[k] = dict(O=_v(b.O), S=_v(b.S), t=_v(b.t), xT=_v(b.xT), yT=_v(b.yT), rear_shoulder=_v(b.rear), lead_shoulder=_v(b.lead))
    reading = {}
    for rd in (1, 2):
        r = {}
        for k, (re, ra, le, la) in {'L': (71, -78, 88, -30), '93': (49, -42, 80, -23), 'c': (35, -9, 70, -1)}.items():
            b = Body(k, H, P)
            el, _ = b.rear_elbow(re, ra, rd)
            ll = add(b.lead, sc(b.upper_arm_dir(le, la, rd), SEG['upper_arm'] * H))
            r[k] = dict(rear_elbow_forward=dot(sub(el, b.rear), b.xT), lead_elbow_forward=dot(sub(ll, b.lead), b.xT))
        reading['reading_%d' % rd] = r
    hands = {}
    for k in ('L', '93'):
        t = misfit_terms(k, chi[k], H, P, chi['A_ref'])
        a = bat_dir((A_LAUNCH if k == 'L' else chi['A_ref'] - SWEEP_93) * DEG, ARMS[k]['bat_elev'] * DEG)
        hands[k] = dict(W=_v(t['W']), rear_elbow=_v(t['elbow']), knob=_v(sub(t['W'], sc(a, U_G))), axis=_v(a),
                        lag_deg=t['lag'], lead_shoulder_to_hands=t['reach'], lead_arm_reach_at_elbow_flexion=t['R'],
                        ahead_of_rear_shoulder=t['ahead'], misfit=t['cost'],
                        other_minima=[dict(chi_deg=c / DEG, misfit=m, ahead=a_) for m, c, a_ in solve_chi(k, H, P, chi['A_ref'])[1]])
    # 3. the jets; 4. path samples; 7. T_swing
    jets, paths = [], []
    cases = [('rhb_middle', (0.0, 0.35, 0.745), 1, 0.150), ('rhb_inside_low', (-0.20, 0.375, 0.56), 1, 0.150),
             ('rhb_outside_high', (0.22, 0.3225, 0.93), 1, 0.150), ('lhb_middle_quick', (0.05, 0.344, 0.70), -1, 0.130),
             ('rhb_middle_T130', (0.0, 0.35, 0.745), 1, 0.130), ('rhb_middle_T174', (0.0, 0.35, 0.745), 1, 0.174)]
    for name, aim, side, T in cases:
        sw = Swing(aim, side=side, t_swing=T)
        j = sw.jet
        W, Wd, Wdd = j.W
        a, ad, add_ = j.axis
        if side == -1:
            W, Wd, Wdd, a, ad, add_ = (mirror(v) for v in (W, Wd, Wdd, a, ad, add_))
        jets.append(dict(name=name, hands=_v(W), hands_velocity=_v(Wd), hands_acceleration=_v(Wdd),
                         axis=_v(a), axis_rate=_v(ad), axis_acceleration=_v(add_),
                         azimuth_unwrapped_rad=sw.A_c[0], azimuth_rate=sw.A_c[1], azimuth_acceleration=sw.A_c[2],
                         elevation_rad=sw.E_c[0], elevation_rate=sw.E_c[1], elevation_acceleration=sw.E_c[2]))
        t_s = sw.t_c - T
        samples = []
        for i in range(int(round((T + TRACK_AFTER_S) / 0.005)) + 1):
            t = t_s + 0.005 * i
            p = sw.pose(t)
            samples.append(dict(t=t, knob=_v(p['knob']), axis=_v(p['axis']), omega=_v(p['omega']), v_cm=_v(p['v_cm']),
                                hands=_v(p['hands']), S=_v(p['S'])))
        paths.append(dict(name=name, aim=list(aim), bats='R' if side == 1 else 'L', height=H, t_swing=T, t_c=sw.t_c,
                          bat_speed=32.0, effort=0.0, tau_93=sw.tau93, tau_w=sw.uw * T, kappa=sw.kappa,
                          launch_azimuth_unwrapped_rad=sw.A_L, samples=samples))
    # 6. braking
    brakes = []
    for name in ('rhb_middle', 'rhb_inside_low', 'rhb_outside_high'):
        sw = Swing(GOLDEN_AIMS[name])
        for tb in (-0.120, -0.082, -0.040, -0.025, -0.012):
            p = sw.pose(sw.t_c + tb + BRAKE_S + 0.005, tau_b=tb)
            brakes.append(dict(name=name, tau_brake=tb, stop_azimuth_deg=sw.A_max(tb), judged_swing=sw.judged_swing(tb),
                               stop_knob=_v(p['knob']), stop_axis=_v(p['axis'])))
    # 8. the aim range, and the bounds
    rows = aim_range()
    keys = ('A_c', 'E_c', 'min_Adot', 'E_min', 'E_max', 'ss_ratio_max', 'tip_ratio_max', 'reach_min', 'reach_max',
            'clear_torso', 'clear_head', 'lag_93', 'lag_c', 'plane_tilt_40ms', 'head_travel_65ms_ft')
    planned = [r for r in rows if 'jet_yaw_deg' not in r['case']]
    rng = dict(n=len(rows), n_planned=len(planned),
               ranges={k: [min(r[k] for r in rows), max(r[k] for r in rows)] for k in keys},
               failures=[dict(case=r['case'], failures=r['failures'], ss_ratio_max=r['ss_ratio_max'],
                              tip_ratio_max=r['tip_ratio_max'], reach_max=r['reach_max'], min_Adot=r['min_Adot'])
                         for r in rows if r['failures']])
    bnd = {}
    for name, aim in GOLDEN_AIMS.items():
        sw = Swing(aim)
        bnd[name] = bounds(sw)
    lag_loc = {loc: dict(value=Swing(aim).lag(0.0), source=LAG_AT_CONTACT_4[loc]) for loc, aim in LOCATION_AIMS.items()}
    sw = Swing(GOLDEN_AIMS['rhb_middle'])
    report = dict(appendix_e_middle=appendix_e_compare(sw), appendix_e_middle_before_step1=appendix_e_compare(Swing(GOLDEN_AIMS['rhb_middle'], path='spec')),
                  v2_difference={n: v2_difference(Swing(a)) for n, a in GOLDEN_AIMS.items()},
                  diagnostics={n: diagnostics(Swing(a)) for n, a in GOLDEN_AIMS.items()})
    return dict(meta=meta, hand_case=hand_case, body=body, reading_check=reading, hands=hands, jets=jets, paths=paths,
                brakes=brakes, aim_range=rng, bounds=bnd, lag_at_contact_by_location=lag_loc, report=report)


if __name__ == '__main__':
    import json
    from pathlib import Path
    out = Path(__file__).resolve().parent / 'golden' / 'swing_v3.json'
    out.write_text(json.dumps(golden(), indent=1) + '\n')
    print('wrote', out)
