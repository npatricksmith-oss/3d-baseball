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


# ---------------------------------------------------------------- the body (§5.3)
class Body:
    """The body at one knot, right-handed batter's frame."""

    def __init__(self, knot, H, P):
        fwd, rise, turn, flex, lean = BODY[knot]
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
                 effort=0.0, path='v3', plan=None):
        self.side, self.H, self.t_c, self.T = side, H, t_c, t_swing
        self.sT = t_swing / T4
        self.tau93 = T4_93 * self.sT
        P = P if P is not None else ((P_TODAY[0] * side, P_TODAY[1], P_TODAY[2]))
        self.P = (P[0] * side, P[1], P[2])  # batter's frame
        self.plan = plan if plan is not None else v2.Plan(aim, t_c, side, H, U_SS, bat_speed, t_swing=t_swing, alpha=alpha)
        self.effort = effort
        self.jet = Jet(self.plan, side, effort)
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
        c = [(u0, 0, self.E_L), (u0, 1, 0.0), (u0, 2, 0.0), (u93, 0, ARMS['93']['bat_elev'] * DEG),
             (0.0, 0, E0), (0.0, 1, E1 * T), (0.0, 2, E2 * T * T)]
        if self.path == 'v3':
            c += [(u * self.sT / T * 1e-3 * 1.0, 0, val * DEG) for u, val in ()]
        return c

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

    # ---- the batter's frame (right-handed) ------------------------------------------------
    def S_rh(self, tau, k=0):
        out = [0.0, 0.0, 0.0]
        ts = [t for t, _ in self.S_knots]
        for i, (ti, Si) in enumerate(self.S_knots):
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
            A, A1 = poly_eval(self.cA, u), poly_eval(self.cA, u, 1) / self.T
            E, E1 = poly_eval(self.cE, u), poly_eval(self.cE, u, 1) / self.T
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
        """Furthest unwrapped azimuth reached (deg); A is non-decreasing, so it is the last."""
        end = TRACK_AFTER_S if tau_b is None else min(TRACK_AFTER_S, tau_b + BRAKE_S)
        if tau_b is not None and tau_b + BRAKE_S <= 0.0:
            return self.state_rh(self.sigma(tau_b + BRAKE_S, tau_b)[0])[0] / DEG
        return max(self.state_rh(self.sigma(end, tau_b)[0])[0], self.state_rh(min(0.0, end))[0]) / DEG

    def judged_swing(self, tau_b):
        return self.A_max(tau_b) >= THETA_SWING

    # ---- checks (§11) -------------------------------------------------------------------------
    def lag(self, tau):
        A, _, E, _, W, _ = self.state_rh(tau)
        return angle_deg(sub(W, self.S_rh(tau)), bat_dir(A, E))
