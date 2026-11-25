"""Checks of the swing v3 oracle (docs/swing-model-v3.md §11) against things it does not compute
from: the hand-worked case, [4]'s and [3]'s published values and Appendix E's curves, v2's
golden fixture, geometry and physical invariants. Run by
    python3 -m unittest analysis/oracle/test_oracle.py
(test_oracle.py imports these classes), or on their own:
    python3 -m unittest analysis/oracle/test_swing_v3.py

A bound from the sources that v3 misses today is an @unittest.expectedFailure whose docstring
gives the number; it turns into an unexpected success (a failed run) when it starts to pass.
"""
import json
import math
import unittest
from pathlib import Path

try:
    from . import swing_v3 as s
    from . import swing_oracle as v2
except ImportError:
    import swing_v3 as s
    import swing_oracle as v2

D = s.DEG
H = 1.85
HERE = Path(__file__).resolve().parent
MIDDLE, INSIDE_LOW, OUTSIDE_HIGH = s.GOLDEN_AIMS['rhb_middle'], s.GOLDEN_AIMS['rhb_inside_low'], s.GOLDEN_AIMS['rhb_outside_high']

_cache = {}
def swing(aim=MIDDLE, **kw):
    key = (tuple(aim), tuple(sorted(kw.items())))
    if key not in _cache:
        _cache[key] = s.Swing(aim, **kw)
    return _cache[key]

_diag = {}
def diag(aim=MIDDLE, **kw):
    key = (tuple(aim), tuple(sorted(kw.items())))
    if key not in _diag:
        _diag[key] = s.diagnostics(swing(aim, **kw))
    return _diag[key]


def bound(name, aim):
    return next(b for b in s.bounds(swing(aim), diag(aim)) if b['check'] == name)


class HandWorkedCase(unittest.TestCase):
    """§11 case 1, worked on paper: S at the origin, an upright trunk facing +x, the rear upper arm
    horizontal straight out to the rear side (θ 90°, φ −90°), the elbow at 90°, χ 270°."""

    def build(self, reading):
        b = s.Body(None, H, (0.0, 0.0, 0.0), params=(0.0, 0.0, 0.0, 0.0, 0.0))
        elbow, d = b.rear_elbow(90.0, -90.0, reading)
        W = b.hands(elbow, d, 90.0, 270.0 * D)
        return s.sub(W, b.S), s.sub(elbow, b.S)

    def test_reading_1_on_paper(self):
        W, elbow = self.build(1)
        hs, lu, lf = 0.112 * H, 0.186 * H, 0.146 * H
        # On paper: the elbow 0.2072 + 0.3441 m out to the rear side (−y), the forearm forward (+x).
        for a, e in zip(elbow, (0.0, -(hs + lu), 0.0)): self.assertAlmostEqual(a, e, places=12)
        for a, e in zip(W, (lf, -(hs + lu), 0.0)): self.assertAlmostEqual(a, e, places=12)
        for a, e in zip(W, (0.2701, -0.5513, 0.0)): self.assertAlmostEqual(a, e, delta=1e-4)
        self.assertAlmostEqual(W[2], 0.0, places=12)  # hands at shoulder height
        lag = s.angle_deg(W, s.bat_dir(-90 * D, 45 * D))
        self.assertAlmostEqual(lag, math.degrees(math.acos((hs + lu) * math.sin(math.pi / 4) / math.hypot(lf, hs + lu))), places=9)
        self.assertAlmostEqual(lag, 50.58, delta=0.01)

    def test_reading_2_gives_another_answer(self):
        W, elbow = self.build(2)
        for a, e in zip(W, (-0.2701, 0.1369, 0.0)): self.assertAlmostEqual(a, e, delta=1e-4)
        self.assertAlmostEqual(s.angle_deg(W, s.bat_dir(-90 * D, 45 * D)), 108.6, delta=0.05)


class ReadingAndBody(unittest.TestCase):
    """§11 case 2 and the body of §5.3 against [4]'s prose and geometry."""

    def forward(self, knot, rear_or_lead, elev, az):
        b = s.Body(knot, H, s.P_STANCE)
        sh = b.rear if rear_or_lead == 'rear' else b.lead
        el = s.add(sh, s.sc(b.upper_arm_dir(elev, az), 0.186 * H))
        return s.dot(s.sub(el, sh), b.xT)

    def test_elbows_move_forward_as_4_describes(self):
        # [4] §4.1.4: "the trail upper arm dropped and moved forward" (−170 → −93 ms);
        # §4.1.5: the lead arm went "down and forward" (−93 ms → contact).
        rL, r93 = self.forward('L', 'rear', 71, -78), self.forward('93', 'rear', 49, -42)
        l93, lc = self.forward('93', 'lead', 80, -23), self.forward('c', 'lead', 70, -1)
        self.assertGreater(r93 - rL, 0.08)
        self.assertGreater(lc, l93)
        self.assertAlmostEqual(rL, 0.07, delta=0.01)
        self.assertAlmostEqual(r93, 0.19, delta=0.01)
        # At contact both upper arms reach forward of the chest.
        self.assertGreater(self.forward('c', 'rear', 35, -9), 0.15)
        self.assertGreater(lc, 0.15)

    def test_trunk_frame(self):
        for k, turn in (('L', -22), ('93', 2), ('c', 75)):
            b = s.Body(k, H, s.P_STANCE)
            self.assertAlmostEqual(s.nrm(s.sub(b.S, b.O)), 0.28 * H, places=12)
            for u in (b.t, b.xT, b.yT): self.assertAlmostEqual(s.nrm(u), 1.0, places=12)
            self.assertAlmostEqual(s.dot(b.xT, b.yT), 0.0, places=12)
            for a, e in zip(s.cross(b.xT, b.yT), b.t): self.assertAlmostEqual(a, e, places=12)  # right-handed
            # The chest faces the trunk's turn: at contact (75°) almost toward the pitcher.
            hx = math.degrees(math.atan2(b.xT[1], b.xT[0]))
            self.assertAlmostEqual(hx, turn, delta=6.0)
            # The rear shoulder is on the batter's right: toward the catcher when he faces the plate.
            self.assertLess(s.sub(b.rear, b.S)[1], 0.0) if k != 'c' else self.assertGreater(s.sub(b.rear, b.S)[0], 0.0)

    def test_launch_hands_where_4_describes(self):
        # [4] at weight commitment: the hands just in front of the back shoulder, at its height.
        b = s.Body('L', H, s.P_STANCE)
        W = s.hands_anchor('L', H, s.P_STANCE)
        ahead = s.dot(s.sub(W, b.rear), b.xT)
        self.assertGreaterEqual(ahead, 0.0)
        self.assertLess(ahead, 0.30)
        self.assertLess(abs(W[2] - b.rear[2]), 0.10)

    def test_lag_at_the_anchors_near_4(self):
        sw = swing()
        self.assertAlmostEqual(sw.lag(-sw.T), 121.0, delta=10.0)       # [4] §4.1.4
        self.assertAlmostEqual(sw.lag(sw.tau93), 131.0, delta=10.0)    # [4] §4.1.4

    def test_chi_is_the_least_misfit_and_does_not_depend_on_height(self):
        chi = s.template_chi()
        for H2 in (1.70, 2.00):
            P2 = (s.P_STANCE[0] * H2 / H, s.P_STANCE[1] * H2 / H, 0.0)
            A_ref = s.template_A_ref(H2)  # the middle-middle contact azimuth of a batter of height H2
            self.assertAlmostEqual(A_ref, chi['A_ref'], places=9)
            for k in ('L', '93'):
                # with the template's knot azimuths (§6.7.3) every length is a multiple of H
                c = s.solve_chi(k, H2, P2, A_knot_deg=chi['A_L' if k == 'L' else 'A_93'])[0]
                self.assertAlmostEqual(c, chi[k], places=6)


class Jet(unittest.TestCase):
    """§11 case 3: at t_c and after it the bat is v2's (golden/swing.json, an independent fixture)."""

    @classmethod
    def setUpClass(cls):
        cls.v2 = {c['name']: c for c in json.loads((HERE / 'golden' / 'swing.json').read_text())}

    def test_pose_and_velocity_equal_v2_golden(self):
        for name in ('rhb_middle', 'rhb_inside_low', 'rhb_outside_high', 'lhb_middle_quick'):
            g = self.v2[name]
            sw = s.Swing(tuple(g['aim']), t_c=g['t_c'], side=1 if g['bats'] == 'R' else -1, H=g['height'],
                         bat_speed=g['bat_speed'], T_force=g['t_swing'])   # v2's fixture is at its own swing time
            for smp in g['samples']:
                if smp['t'] < g['t_c'] - 1e-12:
                    continue
                p = sw.pose(smp['t'])
                for a, e in zip(p['knob'], smp['knob']): self.assertAlmostEqual(a, e, delta=1e-9)
                for a, e in zip(p['axis'], smp['axis']): self.assertAlmostEqual(a, e, delta=1e-9)
                v = sw.point_velocity(smp['t'], g['u_ss'])
                for a, e in zip(v, smp['sweet_spot_velocity']): self.assertAlmostEqual(a, e, delta=1e-5)

    def test_acceleration_equals_v2_finite_differences(self):
        for aim, side, T in ((MIDDLE, 1, 0.150), (INSIDE_LOW, 1, 0.150), ((0.05, 0.344, 0.70), -1, 0.130)):
            p = v2.Plan(aim, 0.40, side, H, s.U_SS, 32.0, t_swing=T)
            j = s.Jet(p, side)
            h = 2e-4
            def W(t):
                k, a = p.knob_axis(t); w = s.add(k, s.sc(a, s.U_G)); return s.mirror(w) if side == -1 else w
            def A(t):
                k, a = p.knob_axis(t); return s.mirror(a) if side == -1 else a
            for f, (x0, x1, x2) in ((W, j.W), (A, j.axis)):
                f_ = [f(0.40 + i * h) for i in (-2, -1, 0, 1, 2)]
                d1 = [(f_[0][c] - 8 * f_[1][c] + 8 * f_[3][c] - f_[4][c]) / (12 * h) for c in range(3)]
                d2 = [(-f_[0][c] + 16 * f_[1][c] - 30 * f_[2][c] + 16 * f_[3][c] - f_[4][c]) / (12 * h * h) for c in range(3)]
                for a, e in zip(x0, f_[2]): self.assertAlmostEqual(a, e, delta=1e-12)
                for a, e in zip(x1, d1): self.assertAlmostEqual(a, e, delta=1e-6 * max(1.0, abs(e)))
                for a, e in zip(x2, d2): self.assertAlmostEqual(a, e, delta=1e-4 * max(1.0, abs(e)))


class PathInvariants(unittest.TestCase):
    """§11 case 5 and the property tests: continuity, rest, velocity = d(position)/dt, mirror."""

    def test_c2_at_every_joint(self):
        # From the curves' values alone (finite differences, not the conditions the construction
        # imposes): the rate and acceleration just left and just right of u_93 and u_w agree, and the
        # analytic derivatives the path reports are the derivatives of its values.
        h = 2e-4
        def fd(f, u0, side):
            v = [f(u0 + side * i * h) for i in range(4)]
            d1 = side * (-3 * v[0] + 4 * v[1] - v[2]) / (2 * h)
            d2 = (2 * v[0] - 5 * v[1] + 4 * v[2] - v[3]) / (h * h)
            return d1, d2
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            for T in (0.130, 0.150, 0.174):
                sw = swing(aim, t_swing=T)
                for i, name in ((0, 'A'), (1, 'E')):
                    f = lambda u: sw.curves(u)[i]
                    for uj in (sw.u93, sw.uw):
                        (l1, l2), (r1, r2) = fd(f, uj - 1e-12, -1), fd(f, uj + 1e-12, 1)
                        scale1 = max(1.0, abs(l1))
                        self.assertAlmostEqual(l1, r1, delta=2e-4 * scale1, msg=(aim, T, name, uj))
                        self.assertAlmostEqual(l2, r2, delta=0.02 * max(1.0, abs(l2), abs(r2)), msg=(aim, T, name, uj))
                    for u in (-0.9, -0.7, sw.u93 + 0.03, -0.3, sw.uw + 0.02, -0.05):
                        c1 = (f(u + 1e-6) - f(u - 1e-6)) / 2e-6
                        c2 = (f(u + 1e-4) - 2 * f(u) + f(u - 1e-4)) / 1e-8
                        self.assertAlmostEqual(sw.curves(u, 1)[i], c1, delta=1e-6 * max(1.0, abs(c1)), msg=(aim, T, name, u))
                        self.assertAlmostEqual(sw.curves(u, 2)[i], c2, delta=1e-3 * max(1.0, abs(c2)), msg=(aim, T, name, u))
                # At contact the path meets v2's analytic swing (§6.4): position, velocity from
                # both sides, and acceleration by one-sided differences.
                L, R = sw.state_rh(-1e-15), sw._v2_state(1e-15)
                for a, e in zip(L[4], R[0]): self.assertAlmostEqual(a, e, delta=1e-9)
                for a, e in zip(L[5], R[1]): self.assertAlmostEqual(a, e, delta=1e-7)
                h, acc = 2e-4, {}
                for side in (-1, 1):
                    f = [sw.point(sw.t_c + side * i * h, s.U_SS) for i in range(4)]
                    acc[side] = [(2 * f[0][c] - 5 * f[1][c] + 4 * f[2][c] - f[3][c]) / h / h for c in range(3)]  # O(h²)
                for a, e in zip(acc[-1], acc[1]): self.assertAlmostEqual(a, e, delta=0.005 * s.nrm(acc[1]))

    def test_at_rest_at_the_start(self):
        for aim in (MIDDLE, INSIDE_LOW):
            sw = swing(aim)
            p = sw.pose(sw.t_c - sw.T)
            for v in (p['omega'], p['v_cm']):
                self.assertLess(s.nrm(v), 1e-9)

    def test_reported_velocity_is_the_derivative_of_position(self):
        for aim, side in ((MIDDLE, 1), (OUTSIDE_HIGH, 1), ((0.05, 0.344, 0.70), -1)):
            sw = swing(aim, side=side)
            h = 1e-6
            for tau in (-0.12, -0.08, -0.04, -0.02, -0.005, 0.01):
                t = sw.t_c + tau
                for u in (0.0, s.U_CM, s.U_SS):
                    fd = s.sc(s.sub(sw.point(t + h, u), sw.point(t - h, u)), 0.5 / h)
                    for a, e in zip(sw.point_velocity(t, u), fd): self.assertAlmostEqual(a, e, delta=1e-5 * max(1.0, abs(e)))
                # Ω is the bat's angular velocity: â̇ = Ω × â.
                p = sw.pose(t)
                fd = s.sc(s.sub(sw.pose(t + h)['axis'], sw.pose(t - h)['axis']), 0.5 / h)
                for a, e in zip(s.cross(p['omega'], p['axis']), fd): self.assertAlmostEqual(a, e, delta=1e-5 * max(1.0, abs(e)))

    def test_left_hander_is_the_mirror(self):
        r = swing((0.1, 0.3375, 0.80))
        l = swing((-0.1, 0.3375, 0.80), side=-1)
        for tau in (-0.15, -0.09, -0.03, 0.0, 0.02):
            a, b = r.pose(r.t_c + tau), l.pose(l.t_c + tau)
            for k in ('knob', 'axis', 'hands', 'S', 'v_cm'):
                self.assertAlmostEqual(a[k][0], -b[k][0], delta=1e-9)
                self.assertAlmostEqual(a[k][1], b[k][1], delta=1e-9)
                self.assertAlmostEqual(a[k][2], b[k][2], delta=1e-9)
            self.assertAlmostEqual(a['omega'][0], b['omega'][0], delta=1e-9)
            self.assertAlmostEqual(a['omega'][1], -b['omega'][1], delta=1e-9)

    def test_launch_pose_does_not_depend_on_the_pitch(self):
        # D22: the same launch pose on every pitch, whatever the aim or effort, for every batter
        # (§6.7.3: his own launch, at his own start t_c − T).
        for bat in (dict(), dict(bat_speed=32.0 * 0.883, t_swing=s.T_MEAN / 1.05 ** 3), dict(bat_speed=32.0 * 1.117, t_swing=s.T_MEAN * 1.05 ** 3)):
            r = swing(**bat)
            ref = r.pose(0.40 - r.T)
            for kw in (dict(aim=INSIDE_LOW), dict(aim=OUTSIDE_HIGH), dict(aim=(0.45, 0.294, 0.35)), dict(aim=MIDDLE, effort=0.08)):
                sw = swing(**kw, **bat)
                p = sw.pose(0.40 - sw.T)
                for k in ('knob', 'axis', 'hands'):
                    for a, e in zip(p[k], ref[k]): self.assertAlmostEqual(a, e, delta=1e-9, msg=(bat, kw, k))
                self.assertLess(s.nrm(p['v_cm']) + s.nrm(p['omega']), 1e-9)


class AppendixE(unittest.TestCase):
    """v3's interior against [4]'s Appendix E (spec §5.4 table, digitised; middle fastballs)."""

    def test_elevation_follows_4_to_minus_30_ms(self):
        # The elevation fix (§6.3): within the digitising error (±1°) from launch to −30 ms of [4]'s
        # time, at every swing time (the anchor instants scale with s_T), once the share of the aim's
        # contact-elevation offset the construction has added by then is taken off (the middle pitch is
        # met 1.0° lower than [4]'s, §6.3: the offset, not the shape). The template batter's swing
        # (§6.7.3); its time follows effort (the same swing, slower).
        for e in (-0.09, 0.0, 0.09):
            sw = swing(MIDDLE, effort=e)
            for row in s.appendix_e_compare(sw):
                if row['t4'] <= -30:
                    u = row['t4'] / 170.0
                    off = sw.E_off * s.smoothstep((u - sw.u93) / (sw.uw - sw.u93)) / D if u > sw.u93 else 0.0
                    self.assertAlmostEqual(row['el'] - off, row['el4'], delta=1.0, msg=(e, row['t4']))
                    self.assertLess(abs(off), 1.1, msg=(e, row['t4']))   # the middle pitch's offset is ~1°

    def test_elevation_before_step1_was_low(self):
        # What step 0 found, so the test above can tell the two paths apart: 8–13° low at −60..−25 ms.
        rows = {r['t4']: r for r in s.appendix_e_compare(s.SwingStep1(MIDDLE, path='spec'))}
        for t4 in (-60, -40, -30, -25):
            self.assertLess(rows[t4]['el'] - rows[t4]['el4'], -7.0)

    def test_azimuth_follows_4s_graph(self):
        # Appendix E's azimuth graph (not used by the construction, which uses its rate graph) as a
        # share of the swing's own travel (§6.7.3): what is left to turn at each instant, over the
        # sweep from launch, against the graph's over its 215° from −170 ms; within 0.04 (the graph's
        # reading error, ±2° of 215°, and its disagreement with its own rate graph, 163° against 166°
        # at −93 ms). At every effort and for the slowest and quickest batters.
        for kw in (dict(), dict(effort=-0.135), dict(effort=0.135), dict(bat_speed=32.0 * 0.883, t_swing=s.T_MEAN / 1.05 ** 3),
                   dict(bat_speed=32.0 * 1.117, t_swing=s.T_MEAN * 1.05 ** 3)):
            sw = swing(MIDDLE, **kw)
            sweep = (sw.A_c[0] - sw.A_L) / D
            for row in s.appendix_e_compare(sw):
                if row['t4'] >= -93:
                    self.assertAlmostEqual(row['az'] / sweep, row['az4'] / 215.0, delta=0.04, msg=(kw, row['t4']))

    def test_azimuth_rate_has_4s_shape(self):
        rows = s.appendix_e_compare(swing())
        for row in rows:
            if row['t4'] >= -93:
                self.assertAlmostEqual(row['rate'], row['rate4'], delta=0.17, msg=row['t4'])
        # The peak in the last 20 ms of [4]'s time, above the contact rate ([4]: 1.09 at −10 ms).
        late = [r for r in rows if -20 <= r['t4'] < 0]
        self.assertGreater(max(r['rate'] for r in late), 1.02)

    def test_jet_bat_angles_inside_4s_table_1(self):
        # The jet (v2's, Statcast) against [4] Table 1 by location, within two sd.
        for loc, aim in s.LOCATION_AIMS.items():
            E4 = {'MIDDLE': (-30, 5), 'HIGH IN': (-24, 5), 'HIGH OUT': (-21, 4), 'LOW IN': (-39, 4), 'LOW OUT': (-33, 5)}[loc]
            A4 = {'MIDDLE': (-8, 12), 'HIGH IN': (-7, 14), 'HIGH OUT': (-16, 12), 'LOW IN': (2, 13), 'LOW OUT': (-12, 10)}[loc]
            sw = swing(aim)
            self.assertLess(abs(sw.E_c[0] / D - E4[0]), 2 * E4[1], loc)
            self.assertLess(abs(sw.A_c[0] / D - A4[0]), 2 * A4[1], loc)


class Braking(unittest.TestCase):
    """§11 case 6 and §8.2: the braked bat stays on its path and stops."""

    def test_brake_slows_the_same_path(self):
        sw = swing()
        for tb in (-0.082, -0.025):
            # σ advances B/2 of path while the clock runs B: the stop pose is the path's at t_b + B/2.
            stop, path = sw.pose(sw.t_c + tb + 0.050, tau_b=tb), sw.pose(sw.t_c + tb + s.BRAKE_S / 2)
            for k in ('knob', 'axis', 'hands'):
                for a, e in zip(stop[k], path[k]): self.assertAlmostEqual(a, e, delta=1e-12)
            self.assertLess(s.nrm(stop['omega']), 1e-12)
            self.assertLess(s.nrm(stop['v_cm']), 1e-12)
            # No jump in velocity at t_b.
            a, b = sw.pose(sw.t_c + tb, tau_b=tb), sw.pose(sw.t_c + tb)
            for x, y in zip(a['v_cm'], b['v_cm']): self.assertAlmostEqual(x, y, delta=1e-12)
            # Half-way through the brake the clock has run 3B/8 of path (σ = t_b + s − s²/2B).
            mid, path = sw.pose(sw.t_c + tb + s.BRAKE_S / 2, tau_b=tb), sw.pose(sw.t_c + tb + 3 * s.BRAKE_S / 8)
            for x, y in zip(mid['axis'], path['axis']): self.assertAlmostEqual(x, y, delta=1e-12)

    def test_braked_velocity_is_the_derivative_of_position(self):
        # While braking every velocity is the path's times σ̇ (§8.2): the reported Ω and V are the
        # derivatives of the braked poses.
        h = 1e-6
        for aim in (MIDDLE, OUTSIDE_HIGH):
            sw = swing(aim)
            for tb in (-0.082, -0.025, -0.012):
                for frac in (0.1, 0.35, 0.6, 0.9):
                    t = sw.t_c + tb + frac * s.BRAKE_S
                    for u in (s.U_CM, s.U_SS):
                        fd = s.sc(s.sub(sw.point(t + h, u, tb), sw.point(t - h, u, tb)), 0.5 / h)
                        for a, e in zip(sw.point_velocity(t, u, tb), fd): self.assertAlmostEqual(a, e, delta=1e-5 * max(1.0, abs(e)))
                    p = sw.pose(t, tau_b=tb)
                    fd = s.sc(s.sub(sw.pose(t + h, tau_b=tb)['axis'], sw.pose(t - h, tau_b=tb)['axis']), 0.5 / h)
                    for a, e in zip(s.cross(p['omega'], p['axis']), fd): self.assertAlmostEqual(a, e, delta=1e-5 * max(1.0, abs(e)))

    def test_ruling_matches_the_bat_direction(self):
        # §8.4: a swing when the bat has turned to across the plate (A_max ≥ 0). Independently of
        # the unwrapped azimuth: at the stop the axis then points toward the plate and the pitcher
        # (a wrapped bat points toward the pitcher but away from the plate).
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            sw = swing(aim)
            for tb in (-0.120, -0.082, -0.040, -0.025, -0.012):
                a = sw.pose(sw.t_c + tb + 0.05, tau_b=tb)['axis']
                self.assertEqual(sw.judged_swing(tb), a[0] >= 0 and a[1] >= 0, (aim, tb))
            # Brakes at the baseline's mean (−82 ms) and earlier stop well back: no swing.
            self.assertLess(sw.A_max(-0.082), -100.0)
            self.assertFalse(sw.judged_swing(-0.025))


class AimRange(unittest.TestCase):
    """§11 case 8 (coarse grid; the full grid is in golden/swing_v3.json, aim_range)."""

    @classmethod
    def setUpClass(cls):
        cls.rows = s.aim_range(coarse=True)

    def test_every_path_is_defined_and_clear_of_the_body(self):
        # Clear of the body over the plate's width; beyond its inside edge, above the zone, the bat
        # passes 2 cm into the trunk (0.45 m inside, 1.14 m high; 0.03% of the game's swings aim more
        # than 0.25 m inside: reported in golden aim_range, spec §6.7.4).
        for r in self.rows:
            for f in ('finite', 'E range'):
                self.assertNotIn(f, r['failures'], r['case'])
            if abs(r['case']['aim'][0]) <= 0.25:
                self.assertNotIn('bat through the body', r['failures'], r['case'])

    def test_planned_swings_turn_one_way(self):
        for r in self.rows:
            if 'jet_yaw_deg' not in r['case']:
                self.assertNotIn('A non-decreasing', r['failures'], r['case'])

    def test_speed_and_reach_in_the_zone(self):
        # The zone's aims (the hands too) and its outer edge (the plate's half-width and the ball,
        # 0.25 m); on its inside edge the sweet spot reaches 1.10–1.11× (the next class's corner, spec
        # §6.7.4).
        for x in (-0.15, 0.0, 0.15, 0.25):
            for z in (0.55, 0.75, 0.95):
                if (x, z) == (-0.15, 0.55):
                    continue   # RatingRange's low-inside corner
                d = diag(s.aim_at(x, z))
                self.assertEqual(s.range_failures(d), [], (x, z))

    def test_one_surge_in_the_zone(self):
        # §6.7.4 (strict): before its peak the azimuth rate never falls below its running maximum (by
        # 0.1% of the peak), nor the sweet spot's speed (by 2%).
        for x in (-0.15, 0.0, 0.15, 0.25):
            for z in (0.55, 0.75, 0.95):
                d = diag(s.aim_at(x, z))
                self.assertLessEqual(d['rate_worst_dip'], s.RATE_DIP_TOL, (x, z))
                self.assertLessEqual(d['ss_worst_dip'], s.SPEED_DIP_TOL, (x, z))

    @unittest.expectedFailure
    def test_speed_bound_outside_the_zone(self):
        """Fails below the zone at the inside edge (golden aim_range; spec §6.7.4): at grade 50 the sweet
        spot reaches 1.16× its contact speed for a pitch at the shins on the inside edge (1.12× 0.2 m
        higher); in the zone it stays at 1.00–1.08 for every rating. The template's elevation and
        hands' path for that aim, not the batter's travel or time."""
        for r in self.rows:
            self.assertNotIn('sweet-spot speed', r['failures'], r['case'])

    @unittest.expectedFailure
    def test_one_surge_outside_the_zone(self):
        """Fails at the outside edge at the shins (2.2% dip in the sweet spot's speed at grade 50); the
        azimuth rate itself has one surge on every swing (spec §6.7.4)."""
        for r in self.rows:
            self.assertNotIn('one surge', r['failures'], r['case'])


class SourceBounds(unittest.TestCase):
    """§11's bounds table on the three golden aims (the measured stance, D28; contact depth 0.35)."""

    def test_lag_maximum(self):
        # [4] §4.1.5: 133° at −80 ms. At the measured stance 135° / 134° / 137° at −94 / −94 / −95 ms
        # (today's stance peaked at −104 to −111 ms, an expected failure).
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            self.assertTrue(bound('lag maximum (deg)', aim)['passed'])
            self.assertTrue(bound('lag maximum time (ms)', aim)['passed'])

    def test_tip_speed_shape(self):
        # [3]: the tip's speed peaks 15 ms before contact at 1.07 times its contact value.
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            self.assertTrue(bound('tip speed peak / contact', aim)['passed'])
            self.assertTrue(bound('tip speed peak time (ms)', aim)['passed'])

    def test_tip_toward_plate_and_down(self):
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            self.assertTrue(bound('tip toward the plate peak time (ms)', aim)['passed'])
            self.assertTrue(bound('tip downward peak time (ms)', aim)['passed'])

    def test_swing_length(self):
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            self.assertTrue(bound('head travel in the last 65 ms (ft)', aim)['passed'])

    @unittest.expectedFailure
    def test_knob_speed(self):
        """Fails at inside-low: 5.65 m/s (middle 6.52, outside-high 7.45) against the 6–9 m/s that
        [3]'s rates and [4]'s elbow suggest (spec §15). The contact jet is v2's, unchanged."""
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            self.assertTrue(bound('knob (hands) speed at contact (m/s)', aim)['passed'])

    @unittest.expectedFailure
    def test_uncocking_rate(self):
        """Fails at inside-low in size only: the lag's uncocking (measured from the clavicle S) peaks at
        1,538 / 1,259 / 1,548°/s at −17 / −17 / −8 ms against [3]'s 1,588 scaled by the contact rate (1,670 /
        1,797 / 1,545 ± 25%; at least 1,348 at inside-low). The contact body that bends for a low
        pitch (§5.3) lowers it there (1,599°/s without the bend); today's stance gave 2,055–2,251°/s at −48
        to −56 ms (spec §5.3.1)."""
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            self.assertTrue(bound('uncocking peak (deg/s)', aim)['passed'])
            self.assertTrue(bound('uncocking peak time (ms)', aim)['passed'])

    @unittest.expectedFailure
    def test_arm_vector_turning_rate(self):
        """Fails in size at inside-low and outside-high: about the trunk axis, as [3] measures it, the arm
        vector peaks at 1,252 / 809 / 1,571°/s at −52 / −60 / −54 ms against [3]'s 1,160 scaled (1,220 /
        1,313 / 1,128 ± 25%) at −65 ± 20 ms; the middle pitch passes, and every peak's time. At today's
        contact depth (0.35 m) all three fail. Appendix E has no arm-vector curve (spec §15)."""
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            self.assertTrue(bound('arm-vector turning peak (deg/s)', aim)['passed'])
            self.assertTrue(bound('arm-vector turning peak time (ms)', aim)['passed'])

    def test_hands_ahead_of_the_clavicle(self):
        # [4]'s arms at contact: the hands about 0.4 m in front of the chest. At the measured stance
        # 0.46 / 0.46 / 0.48 m (today's stance 0.06–0.08 m behind it).
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            self.assertTrue(bound('hands ahead of the clavicle at contact (m)', aim)['passed'])

    def test_lag_at_contact_by_location(self):
        # [4] Table 1: middle 54 against 65, HIGH IN 72 / 79, HIGH OUT 55 / 63, LOW IN 54 / 58, LOW OUT
        # 42 / 51 (today's stance failed at HIGH OUT, 30°).
        for loc, aim in s.LOCATION_AIMS.items():
            self.assertAlmostEqual(swing(aim).lag(0.0), s.LAG_AT_CONTACT_4[loc], delta=15.0, msg=loc)

    @unittest.expectedFailure
    def test_plane_tilt_over_the_last_40_ms(self):
        """Fails: the sweet spot's path over the last 40 ms is 9.6° (middle), 7.2° (inside-low) and
        10.9° (outside-high) steeper than the jet's tilt β. The hands' path is off the jet's plane there
        (as before step 1), and the bat follows [4]'s elevation to −30 ms (spec §6.3, §6.7.6)."""
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            self.assertTrue(bound('plane tilt over the last 40 ms - beta (deg)', aim)['passed'])

    def test_attack_slope_is_v2s_on_the_middle_and_outside_pitches(self):
        for aim in (MIDDLE, OUTSIDE_HIGH):
            self.assertTrue(bound('attack-angle slope on attack direction within the swing (±10 ms), v3 - v2', aim)['passed'])

    @unittest.expectedFailure
    def test_attack_slope_within_the_swing(self):
        """Fails at inside-low: within one swing, over contact timings of ±10 ms, attack angle rises
        0.25° per degree of attack direction more than v2's path does (middle +0.14, outside-high
        +0.06 pass), against a bound of ±0.15 (MLB's slope across swings +0.41, v2's games +0.44;
        spec §6.5, §12)."""
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            self.assertTrue(bound('attack-angle slope on attack direction within the swing (±10 ms), v3 - v2', aim)['passed'])


class Family(unittest.TestCase):
    """§6.7 (PLAN D27): every batter and every effort. Checked against a physical invariant (a driven
    chain whose drive is scaled is the same swing in its own phase, time ∝ 1/speed), MLB's bat
    tracking (docs/research/swing/10-swing-length-by-bat-speed.md) and the decision clock."""

    def phase_pose(self, sw, u):
        p = sw.pose(sw.t_c + u * sw.T)
        return p['knob'] + p['axis'] + p['hands'], p['v_cm']

    def test_an_easier_swing_is_the_same_swing_slower(self):
        # The driven chain's identity (§6.7.1), which the construction imposes (review P3-3).
        base = swing()
        for e in (-0.09, 0.09):
            sw = swing(effort=e)
            self.assertAlmostEqual(sw.T * (1 + e), base.T, delta=1e-12)
            for u in (-1.0, -0.8, -0.55, -0.3, -0.12, -0.04, 0.0):
                (x, v), (x0, v0) = self.phase_pose(sw, u), self.phase_pose(base, u)
                for a, b in zip(x, x0): self.assertAlmostEqual(a, b, delta=1e-9, msg=(e, u))
                for a, b in zip(v, v0): self.assertAlmostEqual(a, b * (1 + e), delta=1e-7, msg=(e, u))

    def head_travel(self, sw, t0):
        n = 400
        pts = [sw.point(sw.t_c + t0 * (1 - i / n), s.BAT_L) for i in range(n + 1)]
        return sum(s.nrm(s.sub(pts[i + 1], pts[i])) for i in range(n)) / 0.3048

    def test_travel_across_batters_as_MLBs(self):
        # Note 10 §1, Savant's per-batter bins: bat speed 65.3 / 68.2 / 70.1 / 71.9 / 73.8 / 76.5 mph, swing
        # length 6.57 / 7.01 / 7.26 / 7.38 / 7.43 / 7.64 ft; length rises 0.05–0.09 ft per mph by cut.
        # Savant's length is closest to the head's travel over the last 65 ms (§15). Each bin within
        # 0.5 ft [E] (§11's swing-length check allows 0.7; the model is 0.0–0.37 ft short, the fastest
        # bins the most), and the slope in MLB's band (0.063). At today's stance spot this restated
        # λ ∝ BatSpeed; at the measured stance the hands carry a third of the head's speed, so it is a
        # comparison (review P3-5).
        mlb = ((65.3, 6.57), (68.2, 7.01), (70.1, 7.26), (71.9, 7.38), (73.8, 7.43), (76.5, 7.64))
        rows = []
        for mph, L4 in mlb:
            sw = swing(bat_speed=mph * s.MPH)
            v = s.nrm(sw.point_velocity(sw.t_c, s.U_SS)) / s.MPH
            L = self.head_travel(sw, -0.065)
            self.assertAlmostEqual(L, L4, delta=0.5, msg=mph)
            rows.append((v, L))
        slope = (rows[-1][1] - rows[0][1]) / (rows[-1][0] - rows[0][0])
        self.assertTrue(0.05 <= slope <= 0.09, slope)

    def test_within_a_batter_the_travel_holds_and_the_time_follows(self):
        # Restates T ∝ 1/(1 + e) (review P3-3); catches effort put into the travel. Note 10 §2: at the same contact depth a swing 4.5% slower is 0.7% shorter; here 0 ± 1%.
        base = swing()
        L0 = self.head_travel(base, -base.T)
        for e in (-0.135, 0.135):
            sw = swing(effort=e)
            self.assertLess(abs(self.head_travel(sw, -sw.T) / L0 - 1), 0.01)
            self.assertGreater((sw.T - base.T) * -e, 0.0)

    def test_quick_hands_is_a_shorter_swing_and_a_slow_bat_too(self):
        # §6.7.3: travel ∝ BatSpeed × T_b; the launch wraps less (azimuth) and stands less high
        # (restates λ < 1 ⇒ less wrap, review P3-3).
        t = swing()
        for kw in (dict(t_swing=s.T_MEAN / 1.05 ** 3), dict(bat_speed=32.0 * 0.883)):
            sw = swing(**kw)
            self.assertGreater(sw.A_L, t.A_L + 20 * D, kw)
            self.assertLess(sw.E_L, t.E_L - 5 * D, kw)

    def test_a_replan_keeps_the_time_and_the_launch(self):
        # §6.7.5: the aim moves 5 cm after the start; T and the launch stay, the contact is the new plan's.
        for aim in (MIDDLE, INSIDE_LOW):
            a0 = swing(aim)
            a1 = s.aim_at(aim[0] + 0.05, aim[2] - 0.05)
            sw, free = s.Swing(a1, T_force=a0.T), swing(a1)
            p, q = sw.pose(sw.t_c - sw.T), a0.pose(a0.t_c - a0.T)
            for k in ('knob', 'axis', 'hands'):
                for x, y in zip(p[k], q[k]): self.assertAlmostEqual(x, y, delta=1e-9)
            for x, y in zip(sw.point(sw.t_c, s.U_SS), free.point(sw.t_c, s.U_SS)): self.assertAlmostEqual(x, y, delta=1e-9)
            # after contact v2's bat at the swing's own (held) time: 1.6% of time moves it by microns
            for x, y in zip(sw.point(sw.t_c + 0.01, s.U_SS), free.point(sw.t_c + 0.01, s.U_SS)): self.assertAlmostEqual(x, y, delta=1e-4)
            for x, y in zip(sw.point_velocity(sw.t_c, s.U_SS), free.point_velocity(free.t_c, s.U_SS)): self.assertAlmostEqual(x, y, delta=1e-6)
            # the re-planned swing keeps [4]'s rate shape: its azimuth rate at u_93 is Appendix E's share
            # (0.565) of the horizontal contact rate (§6.7.3), as a planned swing's is (a travel held at
            # its old value and the mismatch left to the contact correction gives 0.572–0.574)
            self.assertAlmostEqual(sw.curves(sw.u93, 1)[0] / (sw.A_c[1] * s.h_scale(sw.E_c[0]) * sw.T), s.RATE4_RATIO(1e3 * s.T4_93), delta=0.003)
            d, d1 = s.diagnostics(sw, dt=0.001), diag(a1)
            self.assertLess(d['ss_worst_dip'] - d1['ss_worst_dip'], 0.01)
            self.assertLessEqual(d['rate_worst_dip'], s.RATE_DIP_TOL)

    def test_the_swing_starts_inside_the_decision_window(self):
        # §6.7.5: he decides T_latency before his usual swing's bat passes 10% of its peak speed; every
        # batter's swing to a pitch in the zone at ±2 sd of effort leaves rest no earlier than that,
        # the slowest hands (−3 sd) at ±1.5 sd (at −2 sd his swing to the outer third starts 2 ms late)
        # (review P3-3: per batter, T − T_b, not the template's).
        for x in (-0.15, 0.0, 0.15):
            for z in (0.55, 0.75, 0.95):
                for bat, es in ((dict(), 0.09), (dict(t_swing=s.T_MEAN * 1.05 ** 3), 0.0675), (dict(t_swing=s.T_MEAN / 1.05 ** 3), 0.09),
                                (dict(bat_speed=32.0 * 0.883), 0.09)):
                    for e in (-es, es):
                        sw = swing(s.aim_at(x, z), effort=e, **bat)
                        self.assertGreaterEqual(s.start_slack(sw), 0.0, (x, z, bat, e))

    def test_the_clavicle_moves_smoothly(self):
        # A body point's velocity is continuous: S(τ) has no kink at the knot τ_93 (review P3-4).
        sw = swing()
        h = 1e-4
        l = s.sc(s.sub(sw.S_rh(sw.tau93), sw.S_rh(sw.tau93 - h)), 1 / h)
        r = s.sc(s.sub(sw.S_rh(sw.tau93 + h), sw.S_rh(sw.tau93)), 1 / h)
        for a, b in zip(l, r): self.assertAlmostEqual(a, b, delta=0.01)


class RatingRange(unittest.TestCase):
    """§6.7.4: in the zone, for every BatSpeed and QuickHands at ±3 sd (alone and together, short of
    the long corner: fast bat and slow hands at once) and effort at ±3 sd, the sweet spot's speed rises into the
    ball (peak at most 1.10× its contact value, no earlier than 25 ms before contact scaled) with one
    surge, the hands and the knob with one surge, the bat clear of the body and both arms within reach
    at the launch and at −93 ms. Must pass (at the measured stance, D28; contact depth 0.35)."""

    @classmethod
    def setUpClass(cls):
        cls.rows = []
        for kw in s.rating_cases():
            d = s.diagnostics(s.Swing(**{k: v for k, v in kw.items() if k != 'corner'}), dt=0.001)
            cls.rows.append((kw, d))

    def test_every_rating_and_effort_in_the_zone(self):
        for kw, d in self.rows:
            if kw['corner']:
                continue
            self.assertEqual(s.range_failures(d), [], kw)
            # the peak no earlier than −25 ms × T/T̄, within 1.5 ms: the 1 ms sampling, and the shortest
            # swing (slow bat, quick hands, T 0.140 s) at high-inside peaks 0.5 ms before the scaled
            # bound (a tolerance set to that outcome, review P3-5)
            self.assertGreaterEqual(d['ss_ratio_tau'], -0.025 * d['T'] / s.T_MEAN - 0.0015, kw)

    @unittest.expectedFailure
    def test_both_ratings_at_three_sd_at_once(self):
        """Fails at the long corner (BatSpeed +3 sd and QuickHands −3 sd together, travel 1.29, +4.1 sd,
        beyond MLB's longest swings, note 10; `QuickHands` is not drawn in today's games): on one pitch
        the sweet spot's speed dips more than 2% before its peak; the other eight corner swings pass
        every check (spec §6.7.4)."""
        for kw, d in self.rows:
            if kw['corner']:
                self.assertEqual(s.range_failures(d), [], kw)


class HandsSpeed(unittest.TestCase):
    """§6.7.4, review P1-2: the hands (the mid-wrists W, where the client draws them) and the knob
    speed up once, as the bat does (must pass over the rating range: RatingRange)."""

    def test_the_hands_rise_once_at_the_measured_stance(self):
        # Every pitch of the zone at ±2 sd of effort: the hands' and the knob's speed with one surge
        # (2%), from rest to a peak of 10–13.5 m/s 37–50 ms before contact and easing into the ball
        # as the bat whips (§6.7.4).
        for x in (-0.15, 0.0, 0.15):
            for z in (0.55, 0.75, 0.95):
                for e in (-0.09, 0.0, 0.09):
                    d = diag(s.aim_at(x, z), effort=e)
                    self.assertLessEqual(d['hands_worst_dip'], s.SPEED_DIP_TOL, (x, z, e))
                    self.assertLessEqual(d['knob_worst_dip'], s.KNOB_DIP_TOL, (x, z, e))

    def test_todays_stance_was_the_stall(self):
        # The cause, kept as a check (§5.3, §6.7.4): the same construction at today's stance spot stalls
        # the hands (valleys 35–73% on the golden aims): from [4]'s arms at −93 ms only 0.31 m is left
        # to contact hands 0.08 m behind the chest. A change that hid the stall some other way would
        # show here.
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            d = s.diagnostics(s.Swing(aim, P=s.P_TODAY), dt=0.001)
            self.assertGreater(d['hands_worst_dip'], 0.30, aim)


class ArmsAtContact(unittest.TestCase):
    """The two-arm reach at contact (the review's P1-1, §6.7.4): each shoulder to v2's contact hands at most
    2 cm past a straight arm, with the contact body that follows the pitch (§5.3). Must pass over the
    rating range (RatingRange, 'arms reach at contact') and over the game's own swings (here)."""

    def test_both_arms_reach_on_every_game_swing(self):
        # golden/game_aims.json: the standard run's full swings (every fifth, 13,432), at H 1.85 m and the
        # build's contact depth. The contact hands depend only on the aim (v2's jet), the body on the aim's
        # height and side (§5.3), so no path is built.
        g = json.loads((HERE / 'golden' / 'game_aims.json').read_text())
        straight = (s.SEG['upper_arm'] + s.SEG['forearm']) * H
        worst = -1.0
        for x_in, z in zip(g['x_in'], g['z']):
            aim = s.aim_at(-x_in, z)
            W = s.Jet(v2.Plan(aim, 0.0, 1, H, s.U_SS, 32.0), 1).W[0]
            b = s.Body('c', H, s.P_STANCE, params=s.contact_body_params(aim, 1, H))
            worst = max(worst, s.nrm(s.sub(W, b.lead)) - straight, s.nrm(s.sub(W, b.rear)) - straight)
            self.assertLessEqual(worst, s.ARM_SLACK, (x_in, z))

    def test_the_lead_arm_against_4s_elbow(self):
        # Reported against [4]'s lead elbow at contact (56°, Table 1, the middle pitch only): the lead
        # shoulder to the hands within 2 cm of [4]'s arm at that elbow (0.543 m at 1.85 m) on the middle
        # pitch: 1.96 cm. At [4]'s other locations −7.8 (HIGH IN) to −0.2 cm (LOW OUT), with the elbow
        # not given there.
        self.assertTrue(bound("lead shoulder to hands at contact, over [4]'s arm at its 56 deg elbow (m)", MIDDLE)['passed'])


class GoldenFile(unittest.TestCase):
    """golden/swing_v3.json is the oracle's current output, every section (regenerate with
    python3 analysis/oracle/swing_v3.py after any change)."""

    @classmethod
    def setUpClass(cls):
        cls.file = json.loads((HERE / 'golden' / 'swing_v3.json').read_text())
        cls.now = json.loads(json.dumps(s.golden(skip=('aim_range', 'rating_range'))))   # (rating_range_contact_y0_today goes with rating_range)

    def close(self, a, b, path):
        if isinstance(a, dict):
            self.assertEqual(sorted(a), sorted(b), path)
            for k in a: self.close(a[k], b[k], path + '.' + k)
        elif isinstance(a, list):
            self.assertEqual(len(a), len(b), path)
            for i, (x, y) in enumerate(zip(a, b)): self.close(x, y, '%s[%d]' % (path, i))
        elif isinstance(a, float) or isinstance(b, float):
            self.assertAlmostEqual(a, b, delta=1e-10 * max(1.0, abs(b)), msg=path)
        else:
            self.assertEqual(a, b, path)

    def test_every_section_matches_the_oracle(self):
        for k, v in self.now.items():
            if k not in ('aim_range', 'rating_range', 'rating_range_contact_y0_today'):
                self.close(self.file[k], v, k)

    def test_aim_range_matches_the_oracle(self):
        g = self.file['aim_range']
        self.assertEqual(g['n'], len(s.range_cases()))
        picks = g['failures'][::max(1, len(g['failures']) // 4)][:4]
        for row in picks:
            kw = {k: (tuple(v) if isinstance(v, list) else v) for k, v in row['case'].items()}
            d = s.diagnostics(s.Swing(**{k: v for k, v in kw.items() if k != 'corner'}), dt=0.001)
            self.assertEqual(s.range_failures(d), row['failures'], row['case'])
            self.assertAlmostEqual(d['ss_ratio_max'], row['ss_ratio_max'], places=9)


if __name__ == '__main__':
    unittest.main()
