"""Independent oracle for the v2 swing kinematics (agent-models §3.6, 2026-11-14).

Derived from the spec text only. It deliberately computes differently from a closed-form
implementation: the hand and bat angles are integrated numerically (adaptive Simpson) from the
rate bumps, rotations use Rodrigues' formula on explicit in-plane basis vectors, and velocities
are central finite differences of positions. Standard library only.
"""
import math

DEG = math.pi / 180.0

# Template values at grade 50 (agent-models §3.6 table), right-handed batter.
TEMPLATE = dict(
    t_swing=0.150, w_h=0.055, w_b=0.055, tau_b=0.010, q=0.32, lag_c=0.0,
    rho_min_frac=0.15, rho_max_frac=0.46, center_x=-1.10,
    beta0=32.0 * DEG, beta_h=-9.9 * DEG, beta_x=11.0 * DEG,
    alpha0=8.0 * DEG, alpha_h=-8.7 * DEG,
    psi0=0.0, psi_x=16.6 * DEG,
    zone_bottom_frac=0.27, zone_top_frac=0.535,
)
START_WIDTHS = math.sqrt(2.0 * math.log(10.0))


def add(a, b): return tuple(x + y for x, y in zip(a, b))
def sub(a, b): return tuple(x - y for x, y in zip(a, b))
def scale(a, s): return tuple(x * s for x in a)
def dot(a, b): return sum(x * y for x, y in zip(a, b))
def norm(a): return math.sqrt(dot(a, a))
def unit(a): return scale(a, 1.0 / norm(a))
def cross(a, b): return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def rotate(v, k, ang):
    """Rodrigues: rotate v about unit axis k by ang (right-hand rule)."""
    c, s = math.cos(ang), math.sin(ang)
    return add(add(scale(v, c), scale(cross(k, v), s)), scale(k, dot(k, v) * (1.0 - c)))


def simpson(f, a, b, tol=1e-13, depth=40):
    def rec(a, b, fa, fm, fb, whole, tol, d):
        m = 0.5 * (a + b)
        lm, rm = 0.5 * (a + m), 0.5 * (m + b)
        flm, frm = f(lm), f(rm)
        left = (m - a) / 6.0 * (fa + 4 * flm + fm)
        right = (b - m) / 6.0 * (fm + 4 * frm + fb)
        if d <= 0 or abs(left + right - whole) <= 15 * tol:
            return left + right + (left + right - whole) / 15.0
        return rec(a, m, fa, flm, fm, left, tol / 2, d - 1) + rec(m, b, fm, frm, fb, right, tol / 2, d - 1)
    if a == b:
        return 0.0
    fa, fb, fm = f(a), f(b), f(0.5 * (a + b))
    return rec(a, b, fa, fm, fb, (b - a) / 6.0 * (fa + 4 * fm + fb), tol, depth)


class Plan:
    """The contact pose and timing of one planned swing."""

    def __init__(self, aim, t_c, side, height, u_ss, bat_speed, t_swing=None, tpl=TEMPLATE, alpha=None):
        T = tpl if t_swing is None else dict(tpl, t_swing=t_swing)
        s = T['t_swing'] / 0.150
        self.w_h, self.w_b, self.tau_b = T['w_h'] * s, T['w_b'] * s, T['tau_b'] * s
        self.tau_h = START_WIDTHS * self.w_h - T['t_swing']
        self.q, self.t_c, self.u_ss = T['q'], t_c, u_ss
        bottom, top = T['zone_bottom_frac'] * height, T['zone_top_frac'] * height
        h = (aim[2] - bottom) / (top - bottom)
        x_in = -side * aim[0]
        beta = T['beta0'] + T['beta_h'] * (h - 0.5) + T['beta_x'] * x_in
        if alpha is None:
            alpha = T['alpha0'] + T['alpha_h'] * (h - 0.5)
        psi = T['psi0'] + T['psi_x'] * x_in
        if abs(alpha) >= beta:
            beta = abs(alpha) + DEG
        # Horizontal direction at psi, pull-positive: pull is -X for a right-handed batter.
        hh = (-side * math.sin(psi), math.cos(psi), 0.0)
        z = (0.0, 0.0, 1.0)
        v_hat = add(scale(hh, math.cos(alpha)), scale(z, math.sin(alpha)))
        p_hat = cross(z, hh)
        ck = -math.tan(alpha) / math.tan(beta)
        sk = -side * math.sqrt(1.0 - ck * ck)
        nu = add(scale(z, math.cos(beta)), scale(add(scale(hh, ck), scale(p_hat, sk)), math.sin(beta)))
        self.n = scale(nu, side)
        self.a_c = unit(cross(v_hat, self.n))
        x_c = side * T['center_x']
        rho = (aim[0] - x_c) / self.a_c[0] - u_ss
        self.rho_raw = rho
        rho = min(max(rho, T['rho_min_frac'] * height), T['rho_max_frac'] * height)
        self.rho = rho
        # C keeps its lateral position; a clamped swing is short or long along a_c.
        c = sub(aim, scale(self.a_c, u_ss + rho))
        shift = (x_c - c[0]) / self.a_c[0]
        self.center = add(c, scale(self.a_c, shift))
        # The hands' unit vector at contact: rotate a_c back by the lag at contact.
        self.e_c = rotate(self.a_c, self.n, -T['lag_c'])
        self.k = bat_speed / (self.q * rho + u_ss)
        self.alpha, self.beta, self.psi, self.v_hat = alpha, beta, psi, v_hat

    def hand_rate(self, t):
        tau = t - self.t_c
        return self.k * self.q * math.exp(-0.5 * ((tau - self.tau_h) / self.w_h) ** 2) / math.exp(-0.5 * (self.tau_h / self.w_h) ** 2)

    def bat_rate(self, t):
        tau = t - self.t_c
        return self.k * math.exp(-0.5 * ((tau - self.tau_b) / self.w_b) ** 2) / math.exp(-0.5 * (self.tau_b / self.w_b) ** 2)

    def angles(self, t):
        return simpson(self.hand_rate, self.t_c, t), simpson(self.bat_rate, self.t_c, t)

    def knob_axis(self, t):
        dphi, dth = self.angles(t)
        knob = add(self.center, scale(rotate(self.e_c, self.n, dphi), self.rho))
        return knob, rotate(self.a_c, self.n, dth)

    def point(self, t, u):
        k, a = self.knob_axis(t)
        return add(k, scale(a, u))

    def velocity(self, t, u, dt=1e-6):
        return scale(sub(self.point(t + dt, u), self.point(t - dt, u)), 0.5 / dt)

    def t_start(self):
        return self.t_c + self.tau_h - START_WIDTHS * self.w_h

    def swing_length(self, t_end, window=0.150, n=600):
        """Sweet-spot path length over [t_end - window, t_end], chord sum."""
        pts = [self.point(t_end - window + window * i / n, self.u_ss) for i in range(n + 1)]
        return sum(norm(sub(pts[i + 1], pts[i])) for i in range(n))


def golden_cases():
    """Fixture cases for sim/crates/bb_agents/tests/swing_oracle.rs (golden/swing.json)."""
    H, u_ss = 1.85, 34 * 0.0254 - 0.15
    cases = []
    for name, aim, side, t_sw, alpha in [
        ('rhb_middle', (0.0, 0.35, 0.745), 1, 0.150, None),
        ('rhb_inside_low', (-0.20, 0.375, 0.56), 1, 0.150, None),
        ('rhb_outside_high', (0.22, 0.3225, 0.93), 1, 0.150, None),
        ('rhb_out_of_reach', (0.55, 0.28, 0.75), 1, 0.150, None),
        ('lhb_middle_quick', (0.05, 0.344, 0.70), -1, 0.130, None),
        ('rhb_steep_uppercut', (0.0, 0.35, 0.80), 1, 0.150, 20.0 * DEG),
    ]:
        p = Plan(aim, 0.40, side, H, u_ss, 32.0, t_swing=t_sw, alpha=alpha)
        times = [0.40 - t_sw, 0.33, 0.37, 0.39, 0.40, 0.41, 0.43]
        samples = []
        for t in times:
            k, a = p.knob_axis(t)
            samples.append(dict(t=t, knob=list(k), axis=list(a), sweet_spot_velocity=list(p.velocity(t, u_ss))))
        cases.append(dict(name=name, aim=list(aim), bats='R' if side == 1 else 'L', height=H, u_ss=u_ss,
                          bat_speed=32.0, t_swing=t_sw, t_c=0.40, attack_angle=p.alpha,
                          rho=p.rho, center=list(p.center), n=list(p.n), axis_c=list(p.a_c),
                          t_start=p.t_start(), swing_length_150ms=p.swing_length(0.40), samples=samples))
    return cases


if __name__ == '__main__':
    import json
    from pathlib import Path
    out = Path(__file__).resolve().parent / 'golden' / 'swing.json'
    out.write_text(json.dumps(golden_cases(), indent=2) + '\n')
    print('wrote', out)
