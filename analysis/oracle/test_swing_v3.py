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
        b = s.Body(knot, H, s.P_TODAY)
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
            b = s.Body(k, H, s.P_TODAY)
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
        b = s.Body('L', H, s.P_TODAY)
        W = s.hands_anchor('L', H, s.P_TODAY)
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
            P2 = (s.P_TODAY[0] * H2 / H, s.P_TODAY[1] * H2 / H, 0.0)
            A_ref = s.template_A_ref(H2)  # the middle-middle contact azimuth of a batter of height H2
            self.assertAlmostEqual(A_ref, chi['A_ref'], places=9)
            for k in ('L', '93'):
                c = s.solve_chi(k, H2, P2, A_ref)[0]
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
                         bat_speed=g['bat_speed'], t_swing=g['t_swing'])
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
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            for T in (0.130, 0.150, 0.174):
                sw = swing(aim, t_swing=T)
                for k in range(3):
                    a1, e1 = s.poly_eval(sw.c1A, sw.u93, k), s.poly_eval(sw.c1E, sw.u93, k)
                    self.assertAlmostEqual(a1, sw._seg2A(sw.u93, k), delta=1e-9 * max(1, abs(a1)))
                    self.assertAlmostEqual(e1, sw._baseE(sw.u93, k), delta=1e-9 * max(1, abs(e1)))
                    self.assertAlmostEqual(s.poly_eval(sw.dE, sw.uw, k), 0.0, delta=1e-9)
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
        # D22: the same launch pose on every pitch, whatever the aim, timing or effort.
        ref = swing().pose(0.40 - 0.150)
        for kw in (dict(aim=INSIDE_LOW), dict(aim=OUTSIDE_HIGH), dict(aim=MIDDLE, effort=0.08)):
            p = swing(**kw).pose(0.40 - 0.150)
            for k in ('knob', 'axis', 'hands'):
                for a, e in zip(p[k], ref[k]): self.assertAlmostEqual(a, e, delta=1e-12)


class AppendixE(unittest.TestCase):
    """v3's interior against [4]'s Appendix E (spec §5.4 table, digitised; middle fastballs)."""

    def test_elevation_follows_4_to_minus_30_ms(self):
        # The elevation fix (§6.3): within the digitising error (±1°) from launch to −30 ms of [4]'s
        # time, at every swing time (the anchor instants scale with s_T).
        for T in (0.130, 0.150, 0.174):
            sw = swing(MIDDLE, t_swing=T)
            for row in s.appendix_e_compare(sw):
                if row['t4'] <= -30:
                    self.assertAlmostEqual(row['el'], row['el4'], delta=1.0, msg=(T, row['t4']))

    def test_elevation_before_step1_was_low(self):
        # What step 0 found, so the test above can tell the two paths apart: 8–13° low at −60..−25 ms.
        rows = {r['t4']: r for r in s.appendix_e_compare(swing(MIDDLE, path='spec'))}
        for t4 in (-60, -40, -30, -25):
            self.assertLess(rows[t4]['el'] - rows[t4]['el4'], -7.0)

    def test_azimuth_follows_4s_graph(self):
        # Appendix E's azimuth (read off its own graph, not used by the construction) within 8°
        # from −93 ms to contact.
        # 8° at grade 50; 10° at the quicker and slower swing times, where the fixed 168° still
        # to turn at −93 ms bends the shape more (§6.3).
        for T, tol in ((0.130, 10.0), (0.150, 8.0), (0.174, 10.0)):
            for row in s.appendix_e_compare(swing(MIDDLE, t_swing=T)):
                if row['t4'] >= -93:
                    self.assertAlmostEqual(row['az'], row['az4'], delta=tol, msg=(T, row['t4']))

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
        for r in self.rows:
            for f in ('finite', 'E range', 'bat through the body'):
                self.assertNotIn(f, r['failures'], r['case'])

    def test_planned_swings_turn_one_way(self):
        for r in self.rows:
            if 'jet_yaw_deg' not in r['case']:
                self.assertNotIn('A non-decreasing', r['failures'], r['case'])

    def test_speed_and_reach_in_the_zone(self):
        for x in (-0.30, 0.0, 0.30):
            for z in (0.55, 0.75, 0.95):
                d = diag((x, 0.35 - 0.125 * x, z))
                self.assertEqual(s.range_failures(d), [], (x, z))

    @unittest.expectedFailure
    def test_speed_bound_over_the_whole_range(self):
        """Fails (golden aim_range): the sweet spot reaches 1.12× its contact speed below and
        outside the zone at T 0.150, 1.18× for a 10% slow swing (effort −0.1), and 1.36× at
        T 0.130: the 168° still to turn at −93 ms is fixed while the contact rate scales with
        BatSpeed and effort and the time with s_T (spec §6.3)."""
        for r in self.rows:
            self.assertNotIn('sweet-spot speed', r['failures'], r['case'])
        for kw in (dict(effort=-0.10), dict(t_swing=0.130)):
            self.assertLessEqual(diag(MIDDLE, **kw)['ss_ratio_max'], s.SS_RATIO_MAX, kw)


class SourceBounds(unittest.TestCase):
    """§11's bounds table on the three golden aims (today's stance)."""

    def test_lag_maximum(self):
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
        """Fails with today's stance: peak 2,513°/s at −46 ms (middle), 2,777°/s at −42 ms
        (outside-high), against [3]'s 1,588 scaled (1,670 ± 25%) at −18 ± 15 ms (spec §5.3)."""
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            self.assertTrue(bound('uncocking peak (deg/s)', aim)['passed'])
            self.assertTrue(bound('uncocking peak time (ms)', aim)['passed'])

    @unittest.expectedFailure
    def test_arm_vector_turning_rate(self):
        """Fails in time with either stance: peak 1,257°/s at −104 ms (middle) against [3]'s
        −57 ± 20 ms (size inside the bound). Appendix E has no arm-vector curve (spec §15)."""
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            self.assertTrue(bound('arm-vector turning peak (deg/s)', aim)['passed'])
            self.assertTrue(bound('arm-vector turning peak time (ms)', aim)['passed'])

    @unittest.expectedFailure
    def test_hands_ahead_of_the_clavicle(self):
        """Fails with today's stance: the hands 0.06–0.08 m behind the clavicle at contact, against
        ≥ 0.25 m in front ([4]'s arms; spec §5.3, Q46)."""
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            self.assertTrue(bound('hands ahead of the clavicle at contact (m)', aim)['passed'])

    @unittest.expectedFailure
    def test_lag_at_contact_by_location(self):
        """Fails at HIGH OUT only: 30° against [4]'s 63 ± 15° (middle 51 against 65, HIGH IN 72 / 79,
        LOW IN 62 / 58, LOW OUT 40 / 51 pass); with today's stance (spec §5.3)."""
        for loc, aim in s.LOCATION_AIMS.items():
            self.assertAlmostEqual(swing(aim).lag(0.0), s.LAG_AT_CONTACT_4[loc], delta=15.0, msg=loc)

    @unittest.expectedFailure
    def test_plane_tilt_over_the_last_40_ms(self):
        """Fails: the sweet spot's path over the last 40 ms is 5.0° (middle) and 8.8° (inside-low)
        steeper than the jet's tilt β (outside-high +1.1°, passes). The hands' path is up to 11 cm off
        the jet's plane there (as it was before step 1), and the bat, following [4]'s elevation to
        −30 ms, is up to 13° off it (spec §6.3)."""
        for aim in (MIDDLE, INSIDE_LOW, OUTSIDE_HIGH):
            self.assertTrue(bound('plane tilt over the last 40 ms - beta (deg)', aim)['passed'])


class GoldenFile(unittest.TestCase):
    """golden/swing_v3.json is the oracle's current output (regenerate with
    python3 analysis/oracle/swing_v3.py after any change)."""

    def test_paths_match_the_oracle(self):
        g = json.loads((HERE / 'golden' / 'swing_v3.json').read_text())
        for case in g['paths'][:3]:
            sw = swing(tuple(case['aim']), side=1 if case['bats'] == 'R' else -1, t_swing=case['t_swing'])
            for smp in case['samples'][::6]:
                p = sw.pose(smp['t'])
                for k in ('knob', 'axis', 'omega', 'v_cm', 'hands', 'S'):
                    for a, e in zip(p[k], smp[k]): self.assertAlmostEqual(a, e, delta=1e-12)
        self.assertAlmostEqual(g['meta']['chi_L_deg'], s.template_chi()['L'] / D, places=9)
        self.assertAlmostEqual(g['meta']['chi_93_deg'], s.template_chi()['93'] / D, places=9)


if __name__ == '__main__':
    unittest.main()
