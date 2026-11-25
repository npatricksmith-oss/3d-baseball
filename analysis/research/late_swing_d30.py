"""Probe, not adopted (PLAN D30; spec swing-model-v3 §6.3 "The late swing", §6.7.6): the last of the swing
following MLB's tracking. The oracle's swing family with its template's late part joined into v2's own swing
for the plan (Statcast's plane and attack angle, v2's speed profile, at the swing's own time): the bat's
azimuth and elevation over BAT_MS of [4]'s time (C2 smoothstep), v2's after; the hands over HANDS_MS, to
contact. Reports, against the oracle's present path (the lab's to the ball): the sweet spot's speed and attack
angle by Savant's contact-depth bins, against MLB's (analysis/statcast/savant_depth_bins.py); the golden aims'
checks and bounds; the rating range's must-pass checks; and the two swings' disagreement in mid-swing, which
is what blocks the join (the lab's bat has more to turn, faster, at −40 ms).

Run from the repository root (about ten minutes): python3 analysis/research/late_swing_d30.py"""
import sys, math, collections
sys.path.insert(0, 'analysis/oracle')
import swing_v3 as S
import swing_oracle as v2

BAT_MS, HANDS_MS = (-93.0, -30.0), (-93.0, 0.0)
# Savant, six 2025 dates, 11,059 competitive swings: mean depth of each bin (in), bat speed and attack angle
# against the ±2 in bin (savant_depth_bins.py).
MLB = [(-16.4, -4.7, -13.9), (-8.7, -2.3, -7.9), (-4.0, -0.9, -3.6), (-0.1, 0.0, 0.0), (4.0, 0.4, 3.4), (8.8, 0.6, 7.5), (18.0, -0.3, 13.4)]
T0 = S.Template


class LateJoin(T0):
    def v2_u(self, u):
        p, T, tau = self.plan, self.T, u * self.T
        def bump(k, peak, w):
            c, norm = math.sqrt(2.0) * w, math.exp(-0.5 * (peak / w) ** 2)
            th = k * w * math.sqrt(math.pi / 2) / norm * (math.erf((tau - peak) / c) - math.erf(-peak / c))
            th1 = k * math.exp(-0.5 * ((tau - peak) / w) ** 2) / norm
            return th, th1, -th1 * (tau - peak) / w ** 2
        th, th1, th2 = bump(p.k, p.tau_b, p.w_b)
        ph, ph1, ph2 = bump(p.k * p.q, p.tau_h, p.w_h)
        a = v2.rotate(p.a_c, p.n, th); na = S.cross(p.n, a)
        a1 = S.sc(na, th1); a2 = S.add(S.sc(na, th2), S.sc(S.cross(p.n, na), th1 * th1))
        r = S.sc(v2.rotate(p.e_c, p.n, ph), p.rho); K = S.add(p.center, r); nr = S.cross(p.n, r)
        K1 = S.sc(nr, ph1); K2 = S.add(S.sc(nr, ph2), S.sc(S.cross(p.n, nr), ph1 * ph1))
        W = [S.add(K, S.sc(a, S.U_G)), S.add(K1, S.sc(a1, S.U_G)), S.add(K2, S.sc(a2, S.U_G))]
        if self.side == -1:
            W = [S.mirror(v) for v in W]; a, a1, a2 = S.mirror(a), S.mirror(a1), S.mirror(a2)
        (A0, A1, A2), (E0, E1, E2) = S.angles_and_derivs(a, a1, a2)
        while A0 < self.A_c[0] - math.pi: A0 += 2 * math.pi
        while A0 > self.A_c[0] + math.pi: A0 -= 2 * math.pi
        return (A0, A1 * T, A2 * T * T), (E0, E1 * T, E2 * T * T), [tuple(W[j][i] * T ** j for j in range(3)) for i in range(3)]

    @staticmethod
    def _join(lab, late, x, L, k):
        w = (S.smoothstep(x), S.smoothstep(x, 1) / L, S.smoothstep(x, 2) / L ** 2)
        if k == 0: return (1 - w[0]) * lab[0] + w[0] * late[0]
        if k == 1: return (1 - w[0]) * lab[1] + w[0] * late[1] + w[1] * (late[0] - lab[0])
        return (1 - w[0]) * lab[2] + w[0] * late[2] + 2 * w[1] * (late[1] - lab[1]) + w[2] * (late[0] - lab[0])

    def curves(self, u, k=0):
        a, b = BAT_MS[0] / 170.0, BAT_MS[1] / 170.0
        if u <= a: return T0.curves(self, u, k)
        A, E, _ = self.v2_u(u)
        if u >= b: return A[k], E[k]
        lab = [T0.curves(self, u, j) for j in range(3)]
        x, L = (u - a) / (b - a), b - a
        return self._join([l[0] for l in lab], A, x, L, k), self._join([l[1] for l in lab], E, x, L, k)

    def hands(self, u, k=0):
        a, b = HANDS_MS[0] / 170.0, HANDS_MS[1] / 170.0
        if u <= a: return T0.hands(self, u, k)
        _, _, W = self.v2_u(u)
        lab = [T0.hands(self, u, j) for j in range(3)]
        x, L = (u - a) / (b - a), b - a
        return tuple(self._join([l[i] for l in lab], W[i], x, L, k) for i in range(3))


def by_depth(sw):
    """Sweet-spot speed (mph) and attack angle (deg) against τ = 0 at Savant's bins' mean depths, the depth
    measured along the pitch line by the sweet spot's own travel."""
    y0 = sw.point(sw.t_c, S.U_SS)[1]
    v0 = sw.point_velocity(sw.t_c, S.U_SS)
    dep = [((sw.point(sw.t_c + i * 1e-4, S.U_SS)[1] - y0) / 0.0254, i * 1e-4) for i in range(-250, 251)]
    out = []
    for d, _, _ in MLB:
        t = min(dep, key=lambda p: abs(p[0] - d))[1]
        v = sw.point_velocity(sw.t_c + t, S.U_SS)
        out.append((t * 1e3, (S.nrm(v) - S.nrm(v0)) / S.MPH, S.attack(v, sw.side)[0] - S.attack(v0, sw.side)[0]))
    return out


def report(label):
    print('==', label, flush=True)
    batters = [dict()] + [dict(bat_speed=32 * f, t_swing=S.T_MEAN * q) for f, q in ((0.883, 1), (1.117, 1), (1, 1.05 ** -3), (1, 1.05 ** 3))] + [dict(effort=e) for e in (-0.135, 0.135)]
    acc = collections.defaultdict(list)
    for kw in batters:
        for x in (-0.15, 0.0, 0.15):
            for z in (0.55, 0.75, 0.95):
                for i, row in enumerate(by_depth(S.Swing(S.aim_at(x, z), **kw))): acc[i].append(row)
    print("   by Savant's depth bins (the zone's nine pitches x seven batters): depth, ms before t_c, speed vs on time [range] (MLB), attack angle (MLB)")
    for i, (d, mv, ma) in enumerate(MLB):
        rows = acc[i]; m = lambda j: sum(r[j] for r in rows) / len(rows)
        print('   %+6.1f in  %+5.1f ms  %+5.1f mph [%+.1f..%+.1f] (%+.1f)  %+6.1f deg (%+.1f)' % (d, m(0), m(1), min(r[1] for r in rows), max(r[1] for r in rows), mv, m(2), ma))
    for n, a in S.GOLDEN_AIMS.items():
        sw = S.Swing(a); d = S.diagnostics(sw, dt=0.001)
        sl = S.attack_slope(sw) - S.attack_slope(sw, v2_path=True)
        rows = {r['t4']: r for r in S.appendix_e_compare(sw)}
        print('   %-17s sweet spot %.3f at %.0f ms, tilt - beta %+.1f, attack slope v3 - v2 %+.2f, hands/knob valley %.3f/%.3f; elevation - [4] at -60/-40/-30/-20 ms %s; failing bounds %s' % (
            n, d['ss_ratio_max'], d['ss_ratio_tau'] * 1e3, d['plane_tilt_40ms'] - d['beta'], sl, d['hands_worst_dip'], d['knob_worst_dip'],
            ' '.join('%+.1f' % (rows[t]['el'] - rows[t]['el4']) for t in (-60, -40, -30, -20)), [b['check'][:32] for b in S.bounds(sw, d) if not b['passed']]), flush=True)
    c = collections.Counter(); worst = collections.defaultdict(float)
    for kw in S.rating_cases():
        corner = kw.pop('corner')
        d = S.diagnostics(S.Swing(**kw), dt=0.001)
        for f in S.range_failures(d): c[f + (' (corner)' if corner else '')] += 1
        if not corner:
            for k in ('ss_ratio_max', 'rate_worst_dip', 'ss_worst_dip', 'hands_worst_dip', 'knob_worst_dip'): worst[k] = max(worst[k], d[k])
    print('   rating range (108 swings): failures', dict(c), 'worst', {k: round(v, 3) for k, v in worst.items()}, flush=True)


def blocker():
    print("== the two swings in mid-swing (template, the lab's path against v2's own for the plan)")
    for name, a in (('middle', S.GOLDEN_AIMS['rhb_middle']), ('inside-low', S.GOLDEN_AIMS['rhb_inside_low'])):
        tm = T0(a); lj = LateJoin(a)
        for ms in (-93, -60, -40, -30, -20):
            u = ms / 170.0
            A, E, _ = lj.v2_u(u)
            la = tm.curves(u); lr = tm.curves(u, 1)[0] / tm.T
            print('   %-10s %4d ms: to go lab %6.1f v2 %6.1f deg; rate / contact lab %.2f v2 %.2f; elevation lab %6.1f v2 %6.1f deg' % (
                name, ms, (tm.A_c[0] - la[0]) / S.DEG, (tm.A_c[0] - A[0]) / S.DEG, lr / tm.A_c[1], A[1] / tm.T / tm.A_c[1], la[1] / S.DEG, E[0] / S.DEG))


if __name__ == '__main__':
    report("the present path (the lab's to the ball)")
    S.Template = LateJoin
    try:
        report("joined into v2's swing: the bat over %s ms, the hands over %s ms" % (BAT_MS, HANDS_MS))
    finally:
        S.Template = T0
    blocker()
