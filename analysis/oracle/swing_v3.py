"""Independent oracle for the swing model v3 (docs/swing-model-v3.md), stdlib only.

Written from the spec's text (§4.2, §4.3, §5.2, §5.3, §6.1-§6.4, §6.7, §8.2, §8.4). The contact jet
is v2's (swing_oracle.Plan); here its first two derivatives are taken analytically from v2's
rate bumps and checked in the tests against finite differences of v2's own positions.

`Swing` is the swing family of §6.7 (PLAN D27): the template (the grade-50 batter's swing at the aim,
§6.3, in its own phase) scaled for the batter's travel and time, fitted at the stance of PLAN D28
(`P_STANCE`, §5.3.1) with the contact depth a parameter (`CONTACT_Y0`, Q51; `contact_depth(y0)`).
`SwingStep1` is the path as merged in build step 1 (one launch, one swing time, today's stance spot),
kept for the comparisons of §6.7.

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
SEG = dict(upper_arm=0.186, forearm=0.146, half_shoulder=0.112, clavicle=0.28, hip=0.45)  # × H (§5.2, §5.3; hip 0.50 before the D29 refit)
# Body knots (§5.3 table): pelvis forward of P (m), pelvis rise (m), trunk turn, flexion, lean (deg).
# P is the pelvis at the end of [4]'s stance phase (after its 6 cm drift back; the review's P1-1): then
# [4]'s 7 + 11 cm to the launch, 5 cm more to −93 ms and Appendix E's 1 cm to contact.
BODY = {'L': (0.18, 0.00, -22.0, 8.0, 10.0),
        '93': (0.23, 0.00, 2.0, 13.0, 23.0),
        'c': (0.24, 0.05, 75.0, 15.0, 31.0)}
# The contact body follows the pitch [E] (§5.3; the review's P1-1): per 0.1·H (0.185 m at 1.85 m), forward
# flexion +15° for an aim below the zone's middle height (0.4025·H) and +10° for an aim more than 0.10 m
# outside of the plate's centre (reaching out over the plate), and the lean toward the rear side −3° for an
# aim above the middle. No source gives the trunk by location ([4] Table 1 gives the lead knee, 15–18° at
# every location, and the trunk's turn); [3]'s tee swing ends at 20° of lateral flexion, 9° of extension.
FLEX_LOW_DEG = 15.0
FLEX_OUT_DEG = 10.0
OUT_FROM = 0.10
LEAN_HIGH_DEG = 3.0
BEND_PER_H = 0.10
ZONE_MID_FRAC = 0.4025


def contact_body_params(aim, side, H):
    fwd, rise, turn, flex, lean = BODY['c']
    below = max(0.0, ZONE_MID_FRAC * H - aim[2]) / (BEND_PER_H * H)
    above = max(0.0, aim[2] - ZONE_MID_FRAC * H) / (BEND_PER_H * H)
    outside = max(0.0, side * aim[0] - OUT_FROM) / (BEND_PER_H * H)
    return (fwd, rise, turn, flex + FLEX_LOW_DEG * below + FLEX_OUT_DEG * outside, lean - LEAN_HIGH_DEG * above)
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

# Stance spot of today's game (right-handed batter's frame; swing.rs pivot_x, pivot_y), kept for the
# comparisons of §5.3.
P_TODAY = (-0.95, 0.30, 0.0)
# PLAN D28 (§5.3): the batter stands where MLB's do. Savant's batting-stance leaderboard (2025, note 11):
# the hip midpoint 0.708 m behind the plate's front edge (y = 0.432 − 0.708) and 0.918 m off the middle
# line. Read as the position at the end of the stance, from which [4]'s stride carries him forward
# (§5.3's knots) [E: the reading; §5.3.1 says why and what would settle it].
P_STANCE = (-0.918, 0.432 - 0.708, 0.0)
# Where the middle pitch is met, y, and its slope across the plate (contact_k). PLAN D29 (owner Q51 = a):
# where MLB's batters meet it, Savant's average nearest approach 0.505 m less the middle pitch's own
# shortfall from the average (about 0.01 m in step 0's per-pitch sample; up to 0.04 m by note 11 §3)
# [E] (§6.7.8). Today's game value (swing.rs contact_y0 0.35) is kept for a reported comparison only.
CONTACT_Y0 = 0.49
CONTACT_Y0_TODAY = 0.35
CONTACT_K = 0.125


def aim_at(x, z, y0=None, side=1):
    """An aim at plate x, height z, on the contact depth rule: y0 + 0.125 m per metre inside of the
    plate's centre (inside is −x for a right-hander)."""
    return (x, (CONTACT_Y0 if y0 is None else y0) - side * CONTACT_K * x, z)

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


def smoothstep(x, k=0):
    """The quintic smoothstep 10x³ − 15x⁴ + 6x⁵ on [0, 1] (zero rate and acceleration at both ends)
    and its derivatives."""
    if x <= 0.0: return 0.0
    if x >= 1.0: return 1.0 if k == 0 else 0.0
    if k == 0: return x ** 3 * (10 - 15 * x + 6 * x * x)
    if k == 1: return 30 * x * x * (1 - x) ** 2
    return 60 * x * (1 - x) * (1 - 2 * x)


def bat_dir(A, E):
    """§4.2 (radians in)."""
    return (math.cos(E) * math.cos(A), math.cos(E) * math.sin(A), math.sin(E))


# ---------------------------------------------------------------- chi by least misfit (§6.1 steps 3-4)
def misfit_terms(knot, chi, H=1.85, P=P_TODAY, A_ref_deg=None, A_knot_deg=None):
    """M(χ) of §6.1 step 3; the knot's bat azimuth is A_knot_deg (the template's, §6.7), or the step-1
    anchors (−225°, A_ref − 168°) when it is not given."""
    b = Body(knot, H, P)
    m = ARMS[knot]
    elbow, d = b.rear_elbow(m['rear_elev'], m['rear_az'])
    W = b.hands(elbow, d, m['rear_elbow'], chi)
    A = A_knot_deg if A_knot_deg is not None else (A_LAUNCH if knot == 'L' else A_ref_deg - SWEEP_93)
    a = bat_dir(A * DEG, m['bat_elev'] * DEG)
    lag = angle_deg(sub(W, b.S), a)
    reach = nrm(sub(W, b.lead))
    R = lead_reach(H, m['lead_elbow'])
    cost = ((lag - m['lag']) / MISFIT_LAG_SD) ** 2 + ((reach - R) / (MISFIT_REACH_SD * H)) ** 2
    ahead = dot(sub(W, b.rear), b.xT)
    return dict(cost=cost, lag=lag, reach=reach, R=R, W=W, ahead=ahead, elbow=elbow, body=b)


def solve_chi(knot, H=1.85, P=P_TODAY, A_ref_deg=None, n=3600, A_knot_deg=None):
    """All local minima of M(chi) on a grid, each refined by golden section; keep those with the
    hands in front of the rear shoulder and return (chi of the lowest, list of minima)."""
    f = lambda c: misfit_terms(knot, c, H, P, A_ref_deg, A_knot_deg)['cost']
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
            t = misfit_terms(knot, c, H, P, A_ref_deg, A_knot_deg)
            mins.append((t['cost'], c, t['ahead']))
    ok = sorted(m for m in mins if m[2] >= 0.0)
    return ok[0][1], sorted(mins)


MIDDLE_MIDDLE = lambda H, y=None: (0.0, CONTACT_Y0 if y is None else y, (v2.TEMPLATE['zone_bottom_frac'] + 0.5 * (v2.TEMPLATE['zone_top_frac'] - v2.TEMPLATE['zone_bottom_frac'])) * H)


def template_A_ref(H=1.85):
    """Contact azimuth of the template's middle-middle aim (deg), §6.1 step 3."""
    j = Jet(v2.Plan(MIDDLE_MIDDLE(H), 0.0, 1, H, U_SS, 32.0), side=1)
    return j.A[0] / DEG


# ---------------------------------------------------------------- the template's own phase (§6.7)
T_MEAN = 0.170             # s, the grade-50 batter's swing time from rest to contact (§4.3, §6.7.3) [E: [4]'s]
V_TEMPLATE = 32.0          # m/s, the grade-50 BatSpeed (agent-models §10)
LAM_W_MAX = 1.10           # [E] the hands' travel factor at most (§6.7.3, D28 refit): the lead arm straight at −93 ms
I_HAT = -RATE4_RATIO.integral(0.0, -170.0) / 170.0     # [4]'s azimuth sweep from −170 ms over (contact rate × 170 ms)
I_93 = -RATE4_RATIO.integral(0.0, 1e3 * T4_93) / 170.0   # the same from −93 ms


def ref_jet(H=1.85, bat_speed=V_TEMPLATE, side=1, effort=0.0, t_c=0.0):
    """The batter's middle-middle contact jet (his reference contact pose, §6.7.3)."""
    return Jet(v2.Plan(MIDDLE_MIDDLE(H), t_c, side, H, U_SS, bat_speed), side, effort)


def unwrap_about(A0, A_ref):
    """The contact azimuth within 180° of the batter's middle-middle one (§4.2, §6.7)."""
    while A0 <= A_ref - math.pi: A0 += 2 * math.pi
    while A0 > A_ref + math.pi: A0 -= 2 * math.pi
    return A0


E4_C = EL4(0.0) * DEG      # [4]'s bat elevation at contact (rad), −29.7°


def h_scale(E_c):
    """The rate shape's scale over the contact azimuth rate (§6.7.3, D28 refit): [4]'s shape is scaled to
    the bat's horizontal turning rate Ȧ·cos E at contact, at [4]'s own contact elevation, so a bat met
    steeper than [4]'s (a low pitch) turns faster in azimuth only where it is steep, near contact."""
    return math.cos(E_c) / math.cos(E4_C)


def template_knot_azimuths(H=1.85):
    """The template batter's bat azimuth at launch and at u_93 (rad): [4]'s rate shape from rest,
    scaled to his middle-middle contact rate (horizontal, h_scale) over T_MEAN (§6.7.3)."""
    j = ref_jet(H)
    a1 = j.A[1] * h_scale(j.E[0])
    return j.A[0] - a1 * T_MEAN * I_HAT, j.A[0] - a1 * T_MEAN * I_93


_CHI = {}
def template_chi():
    """χ_L, χ_93 (rad): template constants (§6.1 step 4), computed at H = 1.85 m with the template's own
    knot azimuths (§6.7). Every length is relative to the body, so χ depends on neither the stance spot
    nor the contact depth."""
    if not _CHI:
        A_ref = template_A_ref()
        AL, A93 = template_knot_azimuths()
        _CHI['A_ref'] = A_ref
        _CHI['A_L'], _CHI['A_93'] = AL / DEG, A93 / DEG
        _CHI['L'] = solve_chi('L', A_knot_deg=AL / DEG)[0]
        _CHI['93'] = solve_chi('93', A_knot_deg=A93 / DEG)[0]
    return _CHI


_CHI1 = {}
def step1_chi():
    """χ of the step-1 path (launch −225°, A_ref − 168° at −93 ms), for SwingStep1 only."""
    if not _CHI1:
        A_ref = template_A_ref()
        _CHI1['A_ref'] = A_ref
        _CHI1['L'] = solve_chi('L')[0]
        _CHI1['93'] = solve_chi('93', A_ref_deg=A_ref)[0]
    return _CHI1


def hands_anchor(knot, H, P, chi=None):
    chi = (chi or template_chi())[knot]
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
class SwingStep1:
    """One swing on the path as merged in build step 1 (kept for §6.7's comparisons; `Template`
    builds on it). aim, t_c in the sim frame; side +1 right-handed, -1 left-handed.

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
        self._anchors()
        (A0, A1, A2), (E0, E1, E2) = self.jet.A, self.jet.E
        A0 = self._unwrap(A0)
        self.A_c = (A0, A1, A2)
        self.E_c = (E0, E1, E2)
        self.path = path
        self._time()
        self._build()

    def _anchors(self):
        self.W_L = hands_anchor('L', self.H, self.P, step1_chi())
        self.W_93 = hands_anchor('93', self.H, self.P, step1_chi())
        self.A_L, self.E_L = A_LAUNCH * DEG, ARMS['L']['bat_elev'] * DEG

    def _unwrap(self, A0):
        # Unwrapped from L: the contact azimuth lies within (A_L + 180°, A_L + 540°]: a bat
        # cannot meet the ball pointing back at the catcher.
        while A0 <= self.A_L + math.pi: A0 += 2 * math.pi
        while A0 > self.A_L + 3 * math.pi: A0 -= 2 * math.pi
        return A0

    def _time(self):
        pass

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
        E0 = self.E_c[0]
        self.kappa = (E0 - ARMS['93']['bat_elev'] * DEG) / ((EL4(0.0) - EL4(1e3 * T4_93)) * DEG)
        bA = self._baseA
        self.dA = poly_fit([(u93, 0, A0 - SWEEP_93 * DEG - bA(u93)), (u93, 1, 0.0), (u93, 2, 0.0),
                            (0.0, 0, A0 - bA(0.0)), (0.0, 1, A1 * T - bA(0.0, 1)), (0.0, 2, A2 * T * T - bA(0.0, 2))])
        self.c1A = poly_fit([(-1.0, 0, self.A_L), (-1.0, 1, 0.0), (-1.0, 2, 0.0)] + [(u93, k, self._seg2A(u93, k)) for k in range(3)])
        self.c1E = poly_fit([(-1.0, 0, self.E_L), (-1.0, 1, 0.0), (-1.0, 2, 0.0)] + [(u93, k, self._seg2E(u93, k)) for k in range(3)])

    def _seg2A(self, u, k=0):
        return self._baseA(u, k) + poly_eval(self.dA, u, k)

    def _v2_elevation(self, u):
        """v2's own elevation at u and its first two derivatives in u (§6.3's blend). v2's bat turns
        about n̂ by θ(τ), the integral of its Gaussian rate bump (closed form in erf here; v2's own
        oracle integrates it numerically), times (1 + effort)."""
        p, e, tau = self.plan, 1.0 + self.effort, u * self.T
        c = math.sqrt(2.0) * p.w_b
        norm = math.exp(-0.5 * (p.tau_b / p.w_b) ** 2)
        th = e * p.k * p.w_b * math.sqrt(math.pi / 2) / norm * (math.erf((tau - p.tau_b) / c) - math.erf(-p.tau_b / c))
        th1 = e * p.k * math.exp(-0.5 * ((tau - p.tau_b) / p.w_b) ** 2) / norm
        th2 = -th1 * (tau - p.tau_b) / p.w_b ** 2
        a = v2.rotate(p.a_c, p.n, th)
        na = cross(p.n, a)
        a1 = sc(na, th1)
        a2 = add(sc(na, th2), sc(cross(p.n, na), th1 * th1))
        _, (E0, E1, E2) = angles_and_derivs(a, a1, a2)   # the elevation does not change under the mirror
        return E0, E1 * self.T, E2 * self.T ** 2

    def _seg2E(self, u, k=0):
        """[4]'s elevation (scaled by κ) to u_w, then blended into v2's own elevation curve by a C2
        smoothstep w over [u_w, 0] (§6.3): E = (1 − w)·B + w·V."""
        if u <= self.uw:
            return self._baseE(u, k)
        L = -self.uw
        x = (u - self.uw) / L
        w, w1, w2 = smoothstep(x), smoothstep(x, 1) / L, smoothstep(x, 2) / L ** 2
        B = [self._baseE(u, j) for j in range(k + 1)]
        V = self._v2_elevation(u)
        if k == 0: return (1 - w) * B[0] + w * V[0]
        if k == 1: return (1 - w) * B[1] + w * V[1] + w1 * (V[0] - B[0])
        return (1 - w) * B[2] + w * V[2] + 2 * w1 * (V[1] - B[1]) + w2 * (V[0] - B[0])

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


# ---------------------------------------------------------------- the template and the family (§6.7, PLAN D27)
def h_fade(u, u93, k=0):
    """1 at launch falling to 0 at u_93 (the quintic smoothstep, zero rate and acceleration at both
    ends), 0 after: carries the batter's launch (§6.7.3)."""
    if u >= u93: return 0.0
    L = u93 + 1.0
    x = (u + 1.0) / L
    return (1.0 - smoothstep(x)) if k == 0 else -smoothstep(x, k) / L ** k


class Template(SwingStep1):
    """The grade-50 batter's swing at an aim (§6.3 in its own phase, §6.7.3): BatSpeed V_TEMPLATE,
    effort 0, the launch azimuth of template_knot_azimuths, and the time T_t = (A_c − A_L)/(Ȧ_s·Î) so
    that the azimuth is [4]'s rate shape, from rest, scaled to the horizontal contact rate
    Ȧ_s = Ȧ_c·h_scale(E_c); δ_A carries the rest of the jet's rate (Ȧ_c − Ȧ_s, from a bat steeper than
    [4]'s at contact) and its second derivative."""

    def __init__(self, aim, t_c=0.40, side=1, H=1.85, P=None, alpha=None, jet_yaw_deg=0.0):
        self.side, self.H, self.t_c, self.effort = side, H, t_c, 0.0
        P = P if P is not None else ((P_STANCE[0] * side, P_STANCE[1], P_STANCE[2]))
        self.P = (P[0] * side, P[1], P[2])
        self.A_ref = ref_jet(H, side=side).A[0]
        AL, _ = template_knot_azimuths(H)
        self.A_L, self.E_L = AL, ARMS['L']['bat_elev'] * DEG
        j0 = Jet(v2.Plan(aim, t_c, side, H, U_SS, V_TEMPLATE, alpha=alpha), side)
        if jet_yaw_deg:
            j0.yaw(jet_yaw_deg * DEG)
        self.T = self.template_time(unwrap_about(j0.A[0], self.A_ref), j0.A[1] * h_scale(j0.E[0]), AL)
        self.plan = v2.Plan(aim, t_c, side, H, U_SS, V_TEMPLATE, t_swing=self.T, alpha=alpha)
        self.jet = Jet(self.plan, side)
        if jet_yaw_deg:
            self.jet.yaw(jet_yaw_deg * DEG)
        self.sT = self.T / T4
        self.tau93 = T4_93 * self.sT
        self.bodies = {k: Body(k, H, self.P) for k in BODY}
        self.bodies['c'] = Body('c', H, self.P, params=contact_body_params(aim, side, H))   # follows the pitch (§5.3)
        self.W_L = hands_anchor('L', H, self.P)
        self.W_93 = hands_anchor('93', H, self.P)
        (A0, A1, A2), (E0, E1, E2) = self.jet.A, self.jet.E
        self.A_c = (unwrap_about(A0, self.A_ref), A1, A2)
        self.E_c = (E0, E1, E2)
        self.A1s = A1 * h_scale(E0)
        self.path = 'v3'
        self._build()

    @staticmethod
    def template_time(A0, A1s, A_L):
        """T_t: the time in which [4]'s rate shape, scaled to the horizontal contact rate A1s, covers
        A0 − A_L."""
        return (A0 - A_L) / (A1s * I_HAT)

    def _baseA(self, u, k=0):
        """[4]'s rate shape scaled to the horizontal contact rate Ȧ_s (§6.7.3)."""
        A0, A1 = self.A_c[0], self.A1s
        t = 1e3 * T4 * u
        if k == 0: return A0 + A1 * self.T / (1e3 * T4) * RATE4_RATIO.integral(0.0, t)
        if k == 1: return A1 * self.T * RATE4_RATIO(t)
        return A1 * self.T * 1e3 * T4 * RATE4_RATIO(t, 1)

    def _baseE(self, u, k=0):
        """[4]'s elevation from −93 ms (39° there), and the aim's own contact elevation reached by an
        offset ΔE = E_c − [4]'s contact elevation (from 39°), the C2 smoothstep from u_93 to u_w: the bat
        on a low pitch is lower through the swing, not only at contact ([4] §4.2.1). Replaces step 1's
        κ-scaled fall (§6.3)."""
        t = 1e3 * T4 * u
        x, L = (u - self.u93) / (self.uw - self.u93), self.uw - self.u93
        if k == 0: return ARMS['93']['bat_elev'] * DEG + (EL4(t) - EL4(1e3 * T4_93)) * DEG + self.E_off * smoothstep(x)
        return (1e3 * T4) ** k * EL4(t, k) * DEG + self.E_off * smoothstep(x, k) / L ** k

    def _build_v3(self):
        T, u93, uw = self.T, self.tau93 / self.T, T4_W / (1e3 * T4)
        self.u93, self.uw = u93, uw
        A0, A1, A2 = self.A_c
        self.E_off = self.E_c[0] - (ARMS['93']['bat_elev'] * DEG + (EL4(0.0) - EL4(1e3 * T4_93)) * DEG)
        bA = self._baseA
        self.dA = poly_fit([(u93, 0, 0.0), (u93, 1, 0.0), (u93, 2, 0.0),
                            (0.0, 0, A0 - bA(0.0)), (0.0, 1, A1 * T - bA(0.0, 1)), (0.0, 2, A2 * T * T - bA(0.0, 2))])
        self.c1A = poly_fit([(-1.0, 0, self.A_L), (-1.0, 1, 0.0), (-1.0, 2, 0.0)] + [(u93, k, self._seg2A(u93, k)) for k in range(3)])
        self.c1E = poly_fit([(-1.0, 0, self.E_L), (-1.0, 1, 0.0), (-1.0, 2, 0.0)] + [(u93, k, self._seg2E(u93, k)) for k in range(3)])

    def hands(self, u, k=0):
        return tuple(poly_eval(c, u, k) for c in self.cW)


class Swing(SwingStep1):
    """One swing of the family (§6.7, PLAN D27): the template at the aim, every displacement from the
    contact pose multiplied by the travel factor (λ for A and E, λ_W ≤ 1.10·λ/λ_nom for the hands W),
    in the swing's phase u = τ/T with
    T = T_t·(T_b/T_MEAN)/(1 + effort); a second-order correction on [u_93, 0] so the contact jet is the
    batter's own; and the fade h(u) that puts the launch at his own L, the same on every pitch.

    bat_speed is the batter's BatSpeed, t_swing his own swing time T_b (QuickHands, the middle-middle
    swing at effort 0), effort the swing's draw. T_force holds T (a mid-swing re-plan keeps the time,
    §6.7.5); λ then follows it."""

    def __init__(self, aim, t_c=0.40, side=1, H=1.85, bat_speed=V_TEMPLATE, t_swing=T_MEAN, P=None, alpha=None,
                 effort=0.0, path='v3', plan=None, jet_yaw_deg=0.0, T_force=None):
        self.side, self.H, self.t_c, self.effort, self.path = side, H, t_c, effort, 'v3'
        self.T_b, self.bat_speed, self.plan_aim, self.plan_alpha = t_swing, bat_speed, aim, alpha
        tm = self.tmpl = Template(aim, t_c=t_c, side=side, H=H, P=P, alpha=alpha, jet_yaw_deg=jet_yaw_deg)
        self.P = tm.P
        self.lam_nom = self.travel(bat_speed, t_swing)
        T_nom = self.swing_time(tm.T, t_swing, effort)
        self.T = T_nom if T_force is None else T_force
        self.lam = self.replanned_travel(self.lam_nom, self.T, T_nom)
        self.plan = v2.Plan(aim, t_c, side, H, U_SS, bat_speed, t_swing=self.T, alpha=alpha)
        self.jet = Jet(self.plan, side, effort)
        if jet_yaw_deg:
            self.jet.yaw(jet_yaw_deg * DEG)
        self.sT = self.T / T4
        self.tau93 = T4_93 * self.sT
        self.u93, self.uw, self.E_off = tm.u93, tm.uw, tm.E_off
        self.bodies = tm.bodies
        self.S_knots = [(-self.T, self.bodies['L'].S), (self.tau93, self.bodies['93'].S), (0.0, self.bodies['c'].S)]
        (A0, A1, A2), (E0, E1, E2) = self.jet.A, self.jet.E
        self.A_c = (unwrap_about(A0, tm.A_ref), A1, A2)
        self.E_c = (E0, E1, E2)
        W0, W1, W2 = self.jet.W
        own = [self.A_c, self.E_c] + [(W0[i], W1[i], W2[i]) for i in range(3)]
        self._f = [lambda u, k: tm.curves(u, k)[0], lambda u, k: tm.curves(u, k)[1]] + \
                  [(lambda i: lambda u, k: tm.hands(u, k)[i])(i) for i in range(3)]
        T, u93 = self.T, self.u93
        # The hands' travel factor (D28 refit, §6.7.3): λ, but at most LAM_W_MAX: a long swing's hands
        # go back with it until the lead arm is straight at −93 ms (the template's lead arm has 7.4 cm
        # of slack there, about a tenth of the hands' way to contact); the rest of the travel is the
        # bat's (A, E). Scaled with λ by a re-plan.
        self.lam_W = self.lam * min(1.0, LAM_W_MAX / self.lam_nom)
        self._lams = [self.lam, self.lam] + [self.lam_W] * 3
        self._x0 = [o[0] for o in own]
        self._corr = [poly_fit([(u93, 0, 0.0), (u93, 1, 0.0), (u93, 2, 0.0), (0.0, 0, 0.0),
                                (0.0, 1, o[1] * T - lam * f(0.0, 1)), (0.0, 2, o[2] * T * T - lam * f(0.0, 2))])
                      for o, f, lam in zip(own, self._f, self._lams)]
        ref = ref_jet(H, bat_speed, side, effort, t_c)
        self.X_ref = [unwrap_about(ref.A[0], tm.A_ref), ref.E[0]] + list(ref.W[0])
        L_t = [f(-1.0, 0) for f in self._f]
        # The batter's launch: the bat's (A, E) the template's scaled about his middle-middle contact
        # pose by λ; the hands by min(λ, 1): a short swing starts with the hands nearer the ball, a long
        # one at [4]'s launch hands (built from [4]'s arm angles on his own segments, §6.1; scaled
        # past them they leave the lead arm's reach, review P2-1).
        lam_WL = min(self.lam_nom, 1.0)
        self.X_L = [r + self.lam_nom * (l - r) for r, l in zip(self.X_ref[:2], L_t[:2])] + \
                   [r + lam_WL * (l - r) for r, l in zip(self.X_ref[2:], L_t[2:])]
        self._fade = [xl - (x0 + lam * (l - f(0.0, 0))) for xl, x0, l, f, lam in zip(self.X_L, self._x0, L_t, self._f, self._lams)]
        self.A_L, self.E_L = self.X_L[0], self.X_L[1]
        self.W_L = tuple(self.X_L[2:])

    @staticmethod
    def travel(bat_speed, t_swing):
        """λ: his BatSpeed × his own swing time, over the template batter's (§6.7.3)."""
        return (bat_speed / V_TEMPLATE) * (t_swing / T_MEAN)

    @staticmethod
    def swing_time(T_t, t_swing, effort):
        """T: the template's time at the aim, scaled to his own and slowed by an easier swing."""
        return T_t * (t_swing / T_MEAN) / (1.0 + effort)

    @staticmethod
    def replanned_travel(lam_nom, T, T_nom):
        """A re-plan keeps T: the travel follows so the contact rate is the new plan's (§6.7.5)."""
        return lam_nom * T / T_nom

    def coord(self, i, u, k=0):
        """Coordinate i (A, E, W_x, W_y, W_z) and its k-th derivative in u, on [−1, 0]."""
        f, lam = self._f[i], self._lams[i]
        v = (self._x0[i] + lam * (f(u, 0) - f(0.0, 0))) if k == 0 else lam * f(u, k)
        if u >= self.u93:
            v += poly_eval(self._corr[i], u, k)
        return v + self._fade[i] * h_fade(u, self.u93, k)

    def curves(self, u, k=0):
        return self.coord(0, u, k), self.coord(1, u, k)

    def hands(self, u, k=0):
        return tuple(self.coord(2 + i, u, k) for i in range(3))

    def state_rh(self, tau):
        if tau > 0.0:
            return SwingStep1.state_rh(self, tau)
        u, T = tau / self.T, self.T
        A, E = self.curves(u)
        A1, E1 = (x / T for x in self.curves(u, 1))
        return A, A1, E, E1, self.hands(u), tuple(x / T for x in self.hands(u, 1))


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
    # The arm vector a = W − S turning about the trunk axis, as [3] measures segment rotation
    # ("about the trunk axis, mid-hips to mid-shoulders"): ω_arm·t̂ with ω_arm = a × ȧ / |a|² and t̂
    # the unit vector from the mid-hips to the clavicle.
    arm = []
    for x, t in zip(st[1:-1], taus[1:-1]):
        a, ad = sub(x[4], sw.S_rh(t)), sub(x[5], sw.S_rh(t, 1))
        tt = unit(sub(sw.S_rh(t), sw.O_rh(t)))
        arm.append(dot(cross(a, ad), tt) / dot(a, a) / DEG)
    iar = max(range(len(arm)), key=lambda i: arm[i])
    # One surge (§6.3): from −93 ms to contact the azimuth rate never falls below its −93 ms value.
    i93 = min(range(n + 1), key=lambda i: abs(taus[i] - sw.tau93))
    rate_dip = min(Ad[i93:]) / Ad[i93]
    # One surge (§6.7.4): the deepest valley of a speed, the drop below the lower of the maxima on
    # either side, as a share of that lower maximum (an early stall reads at its own size, review
    # P3-1); counted once both maxima pass `SURGE_FLOOR` of the speed at contact (below it the bat
    # cocks against the hands' first move, at a few per cent of its contact speed).
    def worst_dip(x, floor=None):
        n = len(x)
        fl = SURGE_FLOOR * x[-1] if floor is None else floor
        Lm, Rm, m = [0.0] * n, [0.0] * n, 0.0
        for k in range(n): m = max(m, x[k]); Lm[k] = m
        m = 0.0
        for k in range(n - 1, -1, -1): m = max(m, x[k]); Rm[k] = m
        return max([(min(Lm[k], Rm[k]) - x[k]) / min(Lm[k], Rm[k]) for k in range(n) if min(Lm[k], Rm[k]) > fl] or [0.0])
    ss_all = ss + [v0]
    hand_v = [nrm(x[5]) for x in st]
    knob_v = []
    for x in st:
        a = bat_dir(x[0], x[2])
        a1 = (-math.cos(x[2]) * math.sin(x[0]) * x[1] - math.sin(x[2]) * math.cos(x[0]) * x[3],
              math.cos(x[2]) * math.cos(x[0]) * x[1] - math.sin(x[2]) * math.sin(x[0]) * x[3], math.cos(x[2]) * x[3])
        knob_v.append(nrm(sub(x[5], sc(a1, U_G))))
    # The arms' reach (review P2-1): each shoulder to the hands at the body's knots, over a straight arm.
    arms = {}
    for knot, tk in (('L', -T), ('93', sw.tau93), ('c', 0.0)):
        b = sw.bodies[knot]
        W = sw.state_rh(tk)[4]
        straight = (SEG['upper_arm'] + SEG['forearm']) * sw.H
        arms[knot] = (nrm(sub(W, b.lead)) - straight, nrm(sub(W, b.rear)) - straight)
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
        arm_rate_peak=arm[iar], arm_rate_tau=taus[iar + 1], rate_dip_after_93=rate_dip, rate_93_over_c=Ad[i93] / (sw.A_c[1] / DEG),
        rate_worst_dip=worst_dip(Ad, floor=0.0), ss_worst_dip=worst_dip(ss_all), T=sw.T, A_L=sw.A_L / DEG, E_L=sw.E_L / DEG,
        hands_worst_dip=worst_dip(hand_v), knob_worst_dip=worst_dip(knob_v),
        hands_peak=max(hand_v), hands_peak_tau=taus[max(range(len(hand_v)), key=lambda i: hand_v[i])],
        lead_arm_over_L=arms['L'][0], rear_arm_over_L=arms['L'][1], lead_arm_over_93=arms['93'][0], rear_arm_over_93=arms['93'][1],
        lead_arm_over_c=arms['c'][0], rear_arm_over_c=arms['c'][1],
        lead_arm_vs_4_c=arms['c'][0] + (SEG['upper_arm'] + SEG['forearm']) * sw.H - lead_reach(sw.H, LEAD_ELBOW_4_C),
        start_slack=start_slack(sw),
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
E_RANGE = (-45.0, 65.0)   # [E] a plausibility guard: the longest swings (travel +3 sd) launch with the bat up to 62° (§6.7.4)


RATE_DIP_TOL = 5e-3    # [E] one surge: no valley in the azimuth rate deeper than 0.5% (its flat top near contact ripples by up to 0.33% when the attack angle is changed by 4°)
SPEED_DIP_TOL = 0.02   # [E] the sweet spot's and the hands' speed: 2% (a flat top before contact ripples by up to 1.9%)
KNOB_DIP_TOL = SPEED_DIP_TOL   # the knob's, the same 2% (at the build values no valley; at today's contact depth, reported,
                               # the hands and knob dip 2–3% on eight swings of the rating range)
SURGE_FLOOR = 0.25     # [E] valleys counted once the speed passes a quarter of its contact value
LEAD_ELBOW_4_C = 56.0  # deg, [4]'s lead elbow at contact (Table 1, middle pitch): the reported check of the lead arm (§11)
ARM_SLACK = 0.02       # m [E] each arm at most 2 cm past straight at the knots (the template's own residual at L is 0.9 cm, §6.1)
T_LATENCY = 0.050      # s, agent-models §3.4: the decision to the swing's start (the bat at 10% of its peak speed)
T_BAT10 = 0.150 / 0.170   # the share of the from-rest time at which the bat passes 10% of its peak angular speed (§4.3)


def start_slack(sw):
    """How much earlier than his planned start (t_c − T_b) this swing may leave rest without leaving
    before the decision (§6.7.5): the decision comes T_latency before the bat's 10% instant of his
    usual swing, which is (1 − T_BAT10)·T_b after his first motion; negative = the swing starts late."""
    T_b = getattr(sw, 'T_b', sw.T)
    return (T_LATENCY - (1.0 - T_BAT10) * T_b) - (sw.T - T_b)


def range_failures(d, H=1.85):
    """The §6.7.4 verification list for one swing; returns the names of the checks it fails."""
    bad = []
    if not all(math.isfinite(v) for v in d.values()): bad.append('finite')
    if d['min_Adot'] < -1e-6: bad.append('A non-decreasing')
    if d['rate_worst_dip'] > RATE_DIP_TOL or d['ss_worst_dip'] > SPEED_DIP_TOL: bad.append('one surge')
    if d['hands_worst_dip'] > SPEED_DIP_TOL or d['knob_worst_dip'] > KNOB_DIP_TOL: bad.append('hands one surge')
    if max(d['lead_arm_over_L'], d['rear_arm_over_L'], d['lead_arm_over_93'], d['rear_arm_over_93']) > ARM_SLACK: bad.append('arms reach')
    if max(d['lead_arm_over_c'], d['rear_arm_over_c']) > ARM_SLACK: bad.append('arms reach at contact')
    if not (E_RANGE[0] <= d['E_min'] and d['E_max'] <= E_RANGE[1]): bad.append('E range')
    if d['ss_ratio_max'] > SS_RATIO_MAX: bad.append('sweet-spot speed')
    if d['tip_ratio_max'] > TIP_RATIO_MAX: bad.append('tip speed')
    if not (0.15 <= d['reach_min'] and d['reach_max'] <= FULL_REACH * H): bad.append('hands within reach')
    if d['clear_torso'] < 0.0 or d['clear_head'] < 0.0: bad.append('bat through the body')
    return bad


SPEED_FACTORS = (0.883, 0.94, 1.06, 1.117)   # BatSpeed at ±1.5 and ±3 sd (σ 3.9%)


BATSPEED_SD = 1.25 / 32.0      # agent-models §10 (D19)
QUICKHANDS_RHO = 1.05          # agent-models §10
EFFORT_SD = 0.045              # motor::EFFORT_SIGMA
LAMBDA_SD = math.sqrt(BATSPEED_SD ** 2 + math.log(QUICKHANDS_RHO) ** 2)   # the travel factor's spread


def rating_cases(coarse=False):
    """§6.7.4: every rating and effort at ±3 sd (one at a time and together), on the zone's aims (the
    must-pass range); 'corner' marks BatSpeed +3 sd with QuickHands −3 sd at once (+4.2 sd of the travel
    factor, longer than MLB's longest swings), reported, not required."""
    xs = (-0.15, 0.0, 0.15) if not coarse else (0.0,)
    zs = (0.55, 0.75, 0.95) if not coarse else (0.75,)
    out = []
    lo, hi = 1 - 3 * BATSPEED_SD, 1 + 3 * BATSPEED_SD
    qlo, qhi = QUICKHANDS_RHO ** -3, QUICKHANDS_RHO ** 3
    for x in xs:
        for z in zs:
            aim = aim_at(x, z)
            for f, q in ((lo, 1.0), (hi, 1.0), (1.0, qlo), (1.0, qhi), (lo, qhi), (hi, qlo), (lo, qlo), (hi, qhi)):
                corner = f > 1.0 and q > 1.0       # the long corner: the longest swing the ratings give
                out.append(dict(aim=aim, bat_speed=V_TEMPLATE * f, t_swing=T_MEAN * q, corner=corner))
            for e in (-3 * EFFORT_SD, 3 * EFFORT_SD):
                out.append(dict(aim=aim, effort=e, corner=False))
                out.append(dict(aim=aim, effort=e, bat_speed=V_TEMPLATE * lo, t_swing=T_MEAN * qlo, corner=False))
    return out


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
            aim = aim_at(x, z)
            for T in ((T_MEAN,) if coarse else (T_MEAN * QUICKHANDS_RHO ** -3, T_MEAN, T_MEAN * QUICKHANDS_RHO ** 3)):
                base = v2.Plan(aim, 0.40, 1, H, U_SS, 32.0)
                for dal in ((0.0,) if coarse else (-4.0, 0.0, 4.0)):
                    kw = dict(aim=aim, t_swing=T, alpha=None if dal == 0.0 else base.alpha + dal * DEG)
                    out.append(kw)
    if not coarse:
        for e in (-0.10, 0.10):
            for aim in (aim_at(0.0, 0.745), aim_at(-0.45, 0.35), aim_at(0.45, 1.14)):
                out.append(dict(aim=aim, effort=e))
        for aim in (aim_at(0.0, 0.745), aim_at(0.45, 0.35, side=-1), aim_at(-0.45, 1.14, side=-1)):
            out.append(dict(aim=aim, side=-1))
    # Swing time crossed with BatSpeed and effort (±3 sd each), on the zone's corners, the middle and
    # the low and high edges (P2-2 of the step-1 review; §6.7.4).
    if not coarse:
        cross_aims = (aim_at(0.0, 0.745), aim_at(-0.25, 0.50), aim_at(0.25, 0.50), aim_at(-0.25, 0.99),
                      aim_at(0.25, 0.99), aim_at(0.45, 0.35), aim_at(-0.45, 0.35), aim_at(-0.45, 1.14))
        for aim in cross_aims:
            for q in (QUICKHANDS_RHO ** -3, 1.0, QUICKHANDS_RHO ** 3):
                for f in SPEED_FACTORS:
                    out.append(dict(aim=aim, t_swing=T_MEAN * q, bat_speed=32.0 * f))
        for f in SPEED_FACTORS:
            out.append(dict(aim=aim_at(0.0, 0.745), side=-1, bat_speed=32.0 * f))
        for e in (-0.135, 0.135):
            out.append(dict(aim=aim_at(0.0, 0.745), side=-1, effort=e))
    for target in range(-25, 51, 15 if coarse else 5):
        for aim in ((aim_at(0.0, 0.745),) if coarse else (aim_at(0.0, 0.745), aim_at(-0.20, 0.56), aim_at(0.22, 0.93))):
            a0 = Jet(v2.Plan(aim, 0.40, 1, H, U_SS, 32.0)).A[0] / DEG
            out.append(dict(aim=aim, jet_yaw_deg=target - a0))
    return out


def aim_range(coarse=False):
    rows = []
    for kw in range_cases(coarse):
        sw = Swing(**{k: v for k, v in kw.items() if k != 'corner'})
        d = diagnostics(sw, dt=0.001)
        rows.append(dict(case={k: (list(v) if isinstance(v, tuple) else v) for k, v in kw.items()}, failures=range_failures(d), **d))
    return rows


# ---------------------------------------------------------------- bounds from the sources (§11)
GOLDEN_AIMS = {'rhb_middle': aim_at(0.0, 0.745), 'rhb_inside_low': aim_at(-0.20, 0.56), 'rhb_outside_high': aim_at(0.22, 0.93)}
# [4] Table 1's five locations, on the golden aims' columns and rows.
LOCATION_AIMS = {'MIDDLE': aim_at(0.0, 0.745), 'HIGH IN': aim_at(-0.20, 0.93), 'HIGH OUT': aim_at(0.22, 0.93),
                 'LOW IN': aim_at(-0.20, 0.56), 'LOW OUT': aim_at(0.22, 0.56)}
LAG_AT_CONTACT_4 = {'MIDDLE': 65.0, 'HIGH IN': 79.0, 'HIGH OUT': 63.0, 'LOW IN': 58.0, 'LOW OUT': 51.0}


ATTACK_SLOPE_TOL = 0.15   # [E]: about a third of MLB's +0.41 per degree (agent-models §3.6); v2's games give +0.44


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
    chk("lead shoulder to hands at contact, over [4]'s arm at its 56 deg elbow (m)", d['lead_arm_vs_4_c'], -9.0, ARM_SLACK)
    sl3, sl2 = attack_slope(sw), attack_slope(sw, v2_path=True)
    chk('attack-angle slope on attack direction within the swing (±10 ms), v3 - v2', sl3 - sl2, -ATTACK_SLOPE_TOL, ATTACK_SLOPE_TOL)
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


def attack(v, side):
    """Attack angle (the velocity's elevation) and attack direction (its horizontal direction,
    pull +: pull is −x for a right-hander), deg, from a sim-frame velocity (agent-models §3.6)."""
    return (math.degrees(math.atan2(v[2], math.hypot(v[0], v[1]))),
            math.degrees(math.atan2(-side * v[0], v[1])))


V2_DIFF_TAUS_MS = (-25.0, -15.0, -10.0, -8.0, -5.0, -3.0)


def v2_difference(sw):
    """§6.5: v3 − v2 bat azimuth and elevation, the sweet spot's attack angle and direction, and
    the sweet-spot speeds, before contact."""
    p = sw.plan
    out = []
    for ms in V2_DIFF_TAUS_MS:
        tau = ms * 1e-3
        k, a = p.knob_axis(sw.t_c + tau)
        if sw.side == -1: a = mirror(a)
        A2, E2 = math.atan2(a[1], a[0]), math.asin(a[2])
        A, _, E, _, _, _ = sw.state_rh(tau)
        while A2 < A - math.pi: A2 += 2 * math.pi
        while A2 > A + math.pi: A2 -= 2 * math.pi
        v3v, v2v = sw.point_velocity(sw.t_c + tau, U_SS), p.velocity(sw.t_c + tau, U_SS)
        (aa3, ad3), (aa2, ad2) = attack(v3v, sw.side), attack(v2v, sw.side)
        out.append(dict(tau_ms=ms, dA=(A - A2) / DEG, dE=(E - E2) / DEG,
                        attack_angle_v3=aa3, attack_angle_v2=aa2, attack_direction_v3=ad3, attack_direction_v2=ad2,
                        ss_v3_mph=nrm(v3v) / MPH, ss_v2_mph=nrm(v2v) / MPH))
    return out


def attack_slope(sw, lo_ms=-10.0, hi_ms=10.0, step_ms=0.25, v2_path=False):
    """Within one swing, the least-squares slope of attack angle on attack direction over contact
    timings from lo to hi ms (MLB's slope across swings: +0.41 per degree; v2's games +0.44)."""
    xs, ys = [], []
    n = int(round((hi_ms - lo_ms) / step_ms))
    for i in range(n + 1):
        t = sw.t_c + (lo_ms + i * step_ms) * 1e-3
        v = sw.plan.velocity(t, U_SS) if v2_path else sw.point_velocity(t, U_SS)
        aa, ad = attack(v, sw.side)
        xs.append(ad); ys.append(aa)
    mx, my = sum(xs) / len(xs), sum(ys) / len(ys)
    return sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / sum((x - mx) ** 2 for x in xs)


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


def today_plan(sw):
    """Today's v2 for the same batter, aim and effort: its swing time `t_swing` = his T_b × 0.150/0.170
    (the bat's 10% start, §6.7.7), the effort in k (v2's effort multiplies both bumps, as k does)."""
    return v2.Plan(sw.plan_aim, sw.t_c, sw.side, sw.H, U_SS, sw.bat_speed * (1.0 + sw.effort), t_swing=sw.T_b * T_BAT10,
                   alpha=sw.plan_alpha)


def late_vs_today(sw):
    """§6.7.6 over the family (review P2-2): v3 against today's v2 before contact."""
    p = today_plan(sw)
    out = {}
    for ms in (-25.0, -15.0, -10.0):
        t = sw.t_c + ms * 1e-3
        v3v, v2v = sw.point_velocity(t, U_SS), p.velocity(t, U_SS)
        (aa3, _), (aa2, _) = attack(v3v, sw.side), attack(v2v, sw.side)
        out['%d' % ms] = dict(attack_angle_diff=aa3 - aa2, ss_v3_mph=nrm(v3v) / MPH, ss_v2_mph=nrm(v2v) / MPH)
    xs3, ys3, xs2, ys2 = [], [], [], []
    for i in range(81):
        t = sw.t_c + (-10.0 + i * 0.25) * 1e-3
        a3, d3 = attack(sw.point_velocity(t, U_SS), sw.side); a2, d2 = attack(p.velocity(t, U_SS), sw.side)
        xs3.append(d3); ys3.append(a3); xs2.append(d2); ys2.append(a2)
    def slope(xs, ys):
        mx, my = sum(xs) / len(xs), sum(ys) / len(ys)
        return sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / sum((x - mx) ** 2 for x in xs)
    out['attack_slope_diff'] = slope(xs3, ys3) - slope(xs2, ys2)
    pts = [sw.point(sw.t_c - 0.040 + i * 0.001, U_SS) for i in range(41)]
    out['tilt_minus_beta'] = plane_tilt_deg(pts) - sw.plan.beta / DEG
    return out


FAMILY_LATE_CASES = [('grade 50', {}), ('easy -2 sd', dict(effort=-0.09)), ('hard +2 sd', dict(effort=0.09)),
                     ('slow bat -2 sd', dict(bat_speed=32.0 * (1 - 2 * 1.25 / 32))), ('fast bat +2 sd', dict(bat_speed=32.0 * (1 + 2 * 1.25 / 32))),
                     ('quick hands +2 sd', dict(t_swing=T_MEAN / 1.05 ** 2)), ('slow hands -2 sd', dict(t_swing=T_MEAN * 1.05 ** 2))]


def family_late():
    return {name: {n: late_vs_today(Swing(a, **kw)) for n, a in GOLDEN_AIMS.items()} for name, kw in FAMILY_LATE_CASES}


def family_report():
    """§6.7.4: across batters (BatSpeed at fixed T_b) and within a batter (effort), middle aim; the
    template's time by aim. Head travel from the start (rest) and over the last 65 ms."""
    def head_travel(sw, t0):
        n = 400
        pts = [sw.point(sw.t_c + t0 + (-t0) * i / n, BAT_L) for i in range(n + 1)]
        return sum(nrm(sub(pts[i + 1], pts[i])) for i in range(n)) / 0.3048
    M = aim_at(0.0, 0.745)
    across, within = [], []
    for f in (1 - 3 * BATSPEED_SD, 1 - 1.5 * BATSPEED_SD, 1.0, 1 + 1.5 * BATSPEED_SD, 1 + 3 * BATSPEED_SD):
        sw = Swing(M, bat_speed=V_TEMPLATE * f)
        d = diagnostics(sw, dt=0.001)
        across.append(dict(bat_speed_mph=sw.bat_speed / MPH, ss_mph_c=d['ss_mph_c'], T=sw.T, travel=sw.lam, A_L_deg=sw.A_L / DEG, E_L_deg=sw.E_L / DEG,
                           head_from_start_ft=head_travel(sw, -sw.T), head_last_65ms_ft=head_travel(sw, -0.065), ss_ratio_max=d['ss_ratio_max'],
                           lag_L=d['lag_L']))
    for e in (-0.135, -0.09, -0.045, 0.0, 0.045, 0.09, 0.135):
        sw = Swing(M, effort=e)
        d = diagnostics(sw, dt=0.001)
        within.append(dict(effort=e, ss_mph_c=d['ss_mph_c'], T=sw.T, head_from_start_ft=head_travel(sw, -sw.T), ss_ratio_max=d['ss_ratio_max'],
                           ss_ratio_tau=d['ss_ratio_tau']))
    grid = {}
    for z in (0.35, 0.55, 0.75, 0.95, 1.14):
        grid['%.2f' % z] = [Template(aim_at(x, z)).T for x in (-0.45, -0.30, -0.15, 0.0, 0.15, 0.30, 0.45)]
    return dict(across=across, within=within, template_T_by_aim=grid, late_vs_today=family_late())


class contact_depth:
    """Run the oracle with the middle pitch met at y0 (Q51): `with contact_depth(0.50): ...`. The
    batter's reference (middle-middle) jet follows it; aims are given with aim_at(x, z, y0)."""
    def __init__(self, y0): self.y0 = y0
    def __enter__(self):
        global CONTACT_Y0
        self.old, CONTACT_Y0 = CONTACT_Y0, self.y0
    def __exit__(self, *a):
        global CONTACT_Y0
        CONTACT_Y0 = self.old


GOLDEN_XZ = {'rhb_middle': (0.0, 0.745), 'rhb_inside_low': (-0.20, 0.56), 'rhb_outside_high': (0.22, 0.93)}
LOCATION_XZ = {'MIDDLE': (0.0, 0.745), 'HIGH IN': (-0.20, 0.93), 'HIGH OUT': (0.22, 0.93), 'LOW IN': (-0.20, 0.56), 'LOW OUT': (0.22, 0.56)}
# §5.3: the stance spot read at the stance (D28's), at contact (the hips 0.17 m further back), and today's.
STANCES = {'today (P_y +0.30)': P_TODAY, 'measured, stance reading (P_y -0.276)': P_STANCE,
           'measured, contact reading (P_y -0.516)': (P_STANCE[0], P_STANCE[1] - BODY['c'][0], 0.0)}
STANCE_KEYS = ('ss_ratio_max', 'ss_ratio_tau', 'tip_ratio_max', 'tip_ratio_tau', 'hands_worst_dip', 'knob_worst_dip', 'lag_max', 'lag_max_tau',
               'lag_c', 'uncock_peak', 'uncock_tau', 'arm_rate_peak', 'arm_rate_tau', 'hands_ahead_c', 'plane_tilt_40ms', 'beta',
               'lead_arm_over_93', 'lead_arm_over_c', 'rear_arm_over_c', 'T', 'head_travel_65ms_ft')


def stance_report():
    """§5.3's table: the golden aims' diagnostics, bounds and lag at contact by location at each stance
    reading and contact depth."""
    out = {}
    for sname, P in STANCES.items():
        for y0 in (CONTACT_Y0, CONTACT_Y0_TODAY):
            with contact_depth(y0):
                rows = {}
                for n, (x, z) in GOLDEN_XZ.items():
                    sw = Swing(aim_at(x, z), P=P)
                    d = diagnostics(sw, dt=0.001)
                    rows[n] = dict({k: d[k] for k in STANCE_KEYS}, bounds=[dict(check=b['check'], value=b['value'], passed=b['passed']) for b in bounds(sw, d)])
                rows['lag_at_contact_by_location'] = {loc: Swing(aim_at(x, z), P=P).lag(0.0) for loc, (x, z) in LOCATION_XZ.items()}
            out['%s, contact y0 %.2f' % (sname, y0)] = rows
    return out


# [4] Table 2, mean times of the peaks by location (ms before contact): the lead shoulder's azimuth rate
# (the lead arm's turning) and the bat's azimuth rate.
LEAD_ARM_PEAK_4 = {'HIGH IN': -41, 'HIGH OUT': -43, 'LOW IN': -48, 'LOW OUT': -49, 'MIDDLE': -44}
BAT_RATE_PEAK_4 = {'HIGH IN': -11, 'HIGH OUT': -14, 'LOW IN': -8, 'LOW OUT': -10, 'MIDDLE': -10}


def location_report():
    """§6.7.4 (the review's P3-1): at [4]'s five locations, the hands' speed peak and its time against
    [4]'s lead-arm turning peak, the lag maximum (133 ± 8° at −80 ± 20 ms, [4]'s middle) and the arm
    vector's peak, and the bat's azimuth-rate peak time against [4] Table 2. Reported, not tested."""
    out = {}
    for loc, aim in LOCATION_AIMS.items():
        sw = Swing(aim)
        d = diagnostics(sw, dt=0.001)
        best = max((sw.state_rh(-i * 1e-4)[1], -i * 1e-4) for i in range(0, 600))
        out[loc] = dict(hands_peak_mps=d['hands_peak'], hands_peak_ms=d['hands_peak_tau'] * 1e3, hands_contact_mps=d['hands_speed_c'],
                        lead_arm_peak_4_ms=LEAD_ARM_PEAK_4[loc], lag_max_deg=d['lag_max'], lag_max_ms=d['lag_max_tau'] * 1e3,
                        arm_rate_peak_dps=d['arm_rate_peak'], arm_rate_peak_ms=d['arm_rate_tau'] * 1e3,
                        bat_rate_peak_ms=best[1] * 1e3, bat_rate_peak_4_ms=BAT_RATE_PEAK_4[loc], T=sw.T)
    return out


def late_by_depth(y0):
    """§6.7.6 at contact depth y0: the late-contact figures against today's v2 and v2's own path."""
    with contact_depth(y0):
        aims = {n: aim_at(x, z) for n, (x, z) in GOLDEN_XZ.items()}
        return dict(v2_difference={n: v2_difference(Swing(a)) for n, a in aims.items()},
                    family={name: {n: late_vs_today(Swing(a, **kw)) for n, a in aims.items()} for name, kw in FAMILY_LATE_CASES})


def rating_summary(y0):
    """The rating range (§6.7.4) at contact depth y0: failures and the worst values."""
    with contact_depth(y0):
        rows = []
        for kw in rating_cases():
            d = diagnostics(Swing(**{k: v for k, v in kw.items() if k != 'corner'}), dt=0.001)
            rows.append(dict(case={k: (list(v) if isinstance(v, tuple) else v) for k, v in kw.items()}, failures=range_failures(d),
                             ss_ratio_max=d['ss_ratio_max'], hands_worst_dip=d['hands_worst_dip'], knob_worst_dip=d['knob_worst_dip'],
                             lead_arm_over_93=d['lead_arm_over_93'], lead_arm_over_c=d['lead_arm_over_c'], rear_arm_over_c=d['rear_arm_over_c']))
    return rows


def golden(skip=()):
    """Every section of golden/swing_v3.json; `skip` leaves out the slow ones (the tests)."""
    H, P = 1.85, P_STANCE
    chi = template_chi()
    u93, uw = T4_93 / T4, T4_W / (1e3 * T4)
    meta = dict(
        spec='docs/swing-model-v3.md (the swing family, §6.7, PLAN D27; fitted at the measured stance, PLAN D28)', H=H, stance_P=list(P),
        contact_y0=CONTACT_Y0, contact_k=CONTACT_K, hands_travel_max=LAM_W_MAX, E4_contact_deg=E4_C / DEG,
        contact_body=dict(flex_low_deg=FLEX_LOW_DEG, flex_out_deg=FLEX_OUT_DEG, out_from=OUT_FROM, lean_high_deg=LEAN_HIGH_DEG, per_H=BEND_PER_H,
                          zone_mid_frac=ZONE_MID_FRAC, rule='flexion + flex_low*below + flex_out*outside, lean - lean_high*above; below/above/outside in units of per_H*H'),
        rate_shape_scale='A1s = A1 cos(E_c) / cos(E4_contact): the rate shape on the horizontal contact rate',
        elevation='B(u) = 39 deg + (e(t4) - e(-93 ms)) + dE smoothstep((u - u93)/(uw - u93)), dE = E_c - (39 deg + e(0) - e(-93 ms))',
        u_g=U_G, u_ss=U_SS, u_cm=U_CM, bat_length=BAT_L,
        T4=T4, t4_93=T4_93, t4_w_ms=T4_W, segments=SEG,
        body_knots={k: list(v) for k, v in BODY.items()}, arms=ARMS,
        chi_L_deg=chi['L'] / DEG, chi_93_deg=chi['93'] / DEG, A_ref_deg=chi['A_ref'],
        family=dict(T_mean=T_MEAN, v_template=V_TEMPLATE, I_hat=I_HAT, I_93=I_93,
                    template_launch_azimuth_deg=chi['A_L'], template_93_azimuth_deg=chi['A_93'],
                    template_time='T_t = (A_c - A_L,t) / (Adot_c h_scale(E_c) I_hat), the template batter at the aim',
                    swing_time='T = T_t (T_b / T_mean) / (1 + effort)', travel='lambda = (BatSpeed / v_template)(T_b / T_mean) (x T / T_nominal after a re-plan)',
                    hands_travel='lambda_W = lambda min(1, 1.10 / lambda_nominal); the launch hands min(lambda_nominal, 1)',
                    fade='h(u) = 1 - (10x^3 - 15x^4 + 6x^5), x = (u + 1)/(u93 + 1) on [-1, u93], 0 after',
                    correction_nodes=[[u93, 0], [u93, 1], [u93, 2], [0.0, 0], [0.0, 1], [0.0, 2]]),
        step1=dict(launch_azimuth_deg=A_LAUNCH, sweep_93_deg=SWEEP_93, chi_L_deg=step1_chi()['L'] / DEG, chi_93_deg=step1_chi()['93'] / DEG),
        appendix_e=dict(t4_ms=list(T4_MS), rate_dps=list(RATE4_DPS), elevation_deg=list(EL4_DEG),
                        rate_ratio_spline_m=RATE4_RATIO.m, elevation_spline_m=EL4.m,
                        rate_ratio_integral_from_0=[RATE4_RATIO.integral(0.0, t) for t in T4_MS]),
        basis=dict(
            u93=u93, uw=uw,
            launch_quintic=dict(nodes=[[-1.0, 0], [-1.0, 1], [-1.0, 2], [u93, 0], [u93, 1], [u93, 2]],
                                coeffs=basis_matrix([(-1.0, 0), (-1.0, 1), (-1.0, 2), (u93, 0), (u93, 1), (u93, 2)])),
            azimuth_correction=dict(nodes=[[u93, 0], [u93, 1], [u93, 2], [0.0, 0], [0.0, 1], [0.0, 2]],
                                    coeffs=basis_matrix([(u93, 0), (u93, 1), (u93, 2), (0.0, 0), (0.0, 1), (0.0, 2)])),
            elevation_blend='E = (1 - w) B + w V on [uw, 0], w = 10x^3 - 15x^4 + 6x^5, x = (u - uw)/(-uw); V = v2 elevation',
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
        Ak = chi['A_L'] if k == 'L' else chi['A_93']
        t = misfit_terms(k, chi[k], H, P, A_knot_deg=Ak)
        a = bat_dir(Ak * DEG, ARMS[k]['bat_elev'] * DEG)
        hands[k] = dict(W=_v(t['W']), rear_elbow=_v(t['elbow']), knob=_v(sub(t['W'], sc(a, U_G))), axis=_v(a),
                        lag_deg=t['lag'], lead_shoulder_to_hands=t['reach'], lead_arm_reach_at_elbow_flexion=t['R'],
                        ahead_of_rear_shoulder=t['ahead'], misfit=t['cost'],
                        other_minima=[dict(chi_deg=c / DEG, misfit=m, ahead=a_) for m, c, a_ in solve_chi(k, H, P, A_knot_deg=Ak)[1]])
    # 3. the jets; 4. path samples; 7. T_swing
    jets, paths = [], []
    q3, f3 = QUICKHANDS_RHO ** 3, 3 * BATSPEED_SD
    M = aim_at(0.0, 0.745)
    cases = [('rhb_middle', M, 1, T_MEAN, 32.0, 0.0), ('rhb_inside_low', aim_at(-0.20, 0.56), 1, T_MEAN, 32.0, 0.0),
             ('rhb_outside_high', aim_at(0.22, 0.93), 1, T_MEAN, 32.0, 0.0), ('lhb_middle_quick', aim_at(0.05, 0.70, side=-1), -1, T_MEAN / q3, 32.0, 0.0),
             ('rhb_middle_quick_hands', M, 1, T_MEAN / q3, 32.0, 0.0), ('rhb_middle_slow_hands', M, 1, T_MEAN * q3, 32.0, 0.0),
             ('rhb_middle_slow_bat', M, 1, T_MEAN, 32.0 * (1 - f3), 0.0), ('rhb_middle_fast_bat', M, 1, T_MEAN, 32.0 * (1 + f3), 0.0),
             ('rhb_middle_easy', M, 1, T_MEAN, 32.0, -0.09), ('rhb_middle_hard', M, 1, T_MEAN, 32.0, 0.09),
             ('rhb_inside_low_long', aim_at(-0.20, 0.56), 1, T_MEAN * q3, 32.0 * (1 + f3), 0.0)]
    for name, aim, side, Tb, v, e in cases:
        sw = Swing(aim, side=side, t_swing=Tb, bat_speed=v, effort=e)
        T = sw.T
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
        paths.append(dict(name=name, aim=list(aim), bats='R' if side == 1 else 'L', height=H, t_swing=Tb, T=T, t_c=sw.t_c,
                          bat_speed=v, effort=e, travel=sw.lam, template_T=sw.tmpl.T, tau_93=sw.tau93, tau_w=sw.uw * T, elevation_offset_rad=sw.E_off, hands_travel=sw.lam_W,
                          launch_azimuth_unwrapped_rad=sw.A_L, launch_elevation_rad=sw.E_L, launch_hands=_v(sw.W_L), samples=samples))
    # 6. braking
    brakes = []
    for name in ('rhb_middle', 'rhb_inside_low', 'rhb_outside_high'):
        sw = Swing(GOLDEN_AIMS[name])
        for tb in (-0.120, -0.082, -0.040, -0.025, -0.012):
            p = sw.pose(sw.t_c + tb + BRAKE_S + 0.005, tau_b=tb)
            samples = []
            for i in range(10):   # every 5 ms from t_b to t_b + B + 5 ms (P2-1 of the step-1 review)
                t = sw.t_c + tb + 0.005 * i
                q = sw.pose(t, tau_b=tb)
                samples.append(dict(t=t, knob=_v(q['knob']), axis=_v(q['axis']), omega=_v(q['omega']), v_cm=_v(q['v_cm']), hands=_v(q['hands'])))
            brakes.append(dict(name=name, t_c=sw.t_c, tau_brake=tb, stop_azimuth_deg=sw.A_max(tb), judged_swing=sw.judged_swing(tb),
                               stop_knob=_v(p['knob']), stop_axis=_v(p['axis']), samples=samples))
    rng = None
    if 'aim_range' not in skip:
        rows = aim_range()
        keys = ('A_c', 'E_c', 'min_Adot', 'E_min', 'E_max', 'ss_ratio_max', 'tip_ratio_max', 'reach_min', 'reach_max',
                'clear_torso', 'clear_head', 'lag_93', 'lag_c', 'plane_tilt_40ms', 'head_travel_65ms_ft', 'rate_dip_after_93',
                'rate_worst_dip', 'ss_worst_dip', 'T', 'A_L', 'E_L')
        planned = [r for r in rows if 'jet_yaw_deg' not in r['case']]
        rng = dict(n=len(rows), n_planned=len(planned),
                   ranges={k: [min(r[k] for r in rows), max(r[k] for r in rows)] for k in keys},
                   failures=[dict(case=r['case'], failures=r['failures'], ss_ratio_max=r['ss_ratio_max'], tip_ratio_max=r['tip_ratio_max'],
                                  reach_max=r['reach_max'], min_Adot=r['min_Adot'], rate_worst_dip=r['rate_worst_dip'], ss_worst_dip=r['ss_worst_dip'])
                             for r in rows if r['failures']])
    bnd = {}
    for name, aim in GOLDEN_AIMS.items():
        sw = Swing(aim)
        bnd[name] = bounds(sw)
    lag_loc = {loc: dict(value=Swing(aim).lag(0.0), source=LAG_AT_CONTACT_4[loc]) for loc, aim in LOCATION_AIMS.items()}
    sw = Swing(GOLDEN_AIMS['rhb_middle'])
    report = dict(appendix_e_middle=appendix_e_compare(sw),
                  appendix_e_middle_before_step1=appendix_e_compare(SwingStep1(GOLDEN_AIMS['rhb_middle'], path='spec')),
                  v2_difference={n: v2_difference(Swing(a)) for n, a in GOLDEN_AIMS.items()},
                  v2_difference_step1={n: v2_difference(SwingStep1(a)) for n, a in GOLDEN_AIMS.items()},
                  diagnostics={n: diagnostics(Swing(a)) for n, a in GOLDEN_AIMS.items()},
                  family=family_report(),
                  stance_and_depth=stance_report(),
                  locations=location_report(),
                  late_contact_by_depth={'%.2f' % y0: late_by_depth(y0) for y0 in (CONTACT_Y0, CONTACT_Y0_TODAY)})
    rating, rating_today_depth = None, None
    if 'rating_range' not in skip:
        rating_today_depth = dict(contact_y0=CONTACT_Y0_TODAY, rows=rating_summary(CONTACT_Y0_TODAY))   # reported only
        rr = []
        for kw in rating_cases():
            d = diagnostics(Swing(**{k: v for k, v in kw.items() if k != 'corner'}), dt=0.001)
            rr.append(dict(case={k: (list(v) if isinstance(v, tuple) else v) for k, v in kw.items()}, failures=range_failures(d),
                           ss_ratio_max=d['ss_ratio_max'], ss_ratio_tau=d['ss_ratio_tau'], rate_worst_dip=d['rate_worst_dip'],
                           ss_worst_dip=d['ss_worst_dip'], T=d['T'], A_L=d['A_L'], E_L=d['E_L']))
        rating = rr
    return dict(meta=meta, hand_case=hand_case, body=body, reading_check=reading, hands=hands, jets=jets, paths=paths,
                brakes=brakes, aim_range=rng, rating_range=rating, rating_range_contact_y0_today=rating_today_depth, bounds=bnd, lag_at_contact_by_location=lag_loc, report=report)


if __name__ == '__main__':
    import json
    from pathlib import Path
    out = Path(__file__).resolve().parent / 'golden' / 'swing_v3.json'
    out.write_text(json.dumps(golden(), indent=1) + '\n')
    print('wrote', out)
