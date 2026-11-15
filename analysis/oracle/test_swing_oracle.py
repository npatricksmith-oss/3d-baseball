"""Spec checks of the swing oracle itself (agent-models §3.6 v2)."""
import math
import unittest

try:
    from . import swing_oracle as so
except ImportError:
    import swing_oracle as so

H, U = 1.85, 34 * 0.0254 - 0.15


class SwingOracle(unittest.TestCase):
    def test_speed_is_bat_speed_at_every_aim(self):
        full = dict(so.TEMPLATE, reach_gain=1.0)
        for aim in [(-0.22, 0.35, 0.75), (0.0, 0.35, 0.75), (0.22, 0.35, 0.75), (0.0, 0.35, 0.55)]:
            for tpl in (so.TEMPLATE, full):
                p = so.Plan(aim, 0.4, 1, H, U, 32.0, tpl=tpl)
                self.assertAlmostEqual(so.norm(p.velocity(0.4, U)), 32.0, places=6)
                # The aim is always on the bat's line at contact (step 7: C keeps its lateral
                # position, so only the station changes) ...
                d = so.sub(aim, p.point(0.4, U))
                self.assertLess(so.norm(so.sub(d, so.scale(p.a_c, so.dot(d, p.a_c)))), 1e-12)
            # ... and with full reach adjustment the sweet spot meets it.
            self.assertLess(so.norm(so.sub(so.Plan(aim, 0.4, 1, H, U, 32.0, tpl=full).point(0.4, U), aim)), 1e-12)

    def test_partial_reach_jams_inside_and_reaches_outside(self):
        # Step 7 with g = 0.6: the ball at the aim meets the bat (1 - g) of the way between the
        # sweet spot and where a middle-middle hand radius would put it. Inside pitches meet
        # the bat inboard of the sweet spot (jammed), outside ones toward the tip.
        full = dict(so.TEMPLATE, reach_gain=1.0)
        mid_z = (0.27 + 0.5 * (0.535 - 0.27)) * H
        ref = so.Plan((0.0, 0.35, mid_z), 0.4, 1, H, U, 32.0, tpl=full).rho
        for x, sign in [(-0.15, -1), (0.0, 0), (0.15, 1)]:
            aim = (x, 0.35, mid_z)
            need = so.Plan(aim, 0.4, 1, H, U, 32.0, tpl=full).rho
            p = so.Plan(aim, 0.4, 1, H, U, 32.0)
            station = so.dot(so.sub(aim, p.point(0.4, U)), p.a_c)  # + toward the tip
            self.assertAlmostEqual(station, 0.4 * (need - ref), places=12)
            if sign:
                self.assertEqual(math.copysign(1, station), sign)
                self.assertGreater(abs(station), 0.02)

    def test_start_is_t_swing_before_contact(self):
        for t_sw in (0.13, 0.15):
            p = so.Plan((0.0, 0.35, 0.75), 0.4, 1, H, U, 32.0, t_swing=t_sw)
            self.assertAlmostEqual(p.t_start(), 0.4 - t_sw, places=12)
            self.assertAlmostEqual(p.hand_rate(p.t_start()) / p.hand_rate(0.4 + p.tau_h), 0.1, places=9)

    def test_mirror(self):
        r = so.Plan((0.1, 0.33, 0.8), 0.4, 1, H, U, 32.0)
        l = so.Plan((-0.1, 0.33, 0.8), 0.4, -1, H, U, 32.0)
        for t in (0.3, 0.38, 0.42):
            a, b = r.point(t, U), l.point(t, U)
            self.assertAlmostEqual(a[0], -b[0], places=12)
            self.assertAlmostEqual(a[1], b[1], places=12)

    def test_swing_length_near_statcast(self):
        # Fitted target (spec table): about 7.2 ft at a middle aim.
        p = so.Plan((0.0, 0.35, 0.745), 0.4, 1, H, U, 32.0)
        self.assertAlmostEqual(p.swing_length(0.4) / 0.3048, 7.07, delta=0.05)


if __name__ == '__main__':
    unittest.main()
