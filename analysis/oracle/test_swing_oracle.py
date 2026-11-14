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
        for aim in [(-0.22, 0.35, 0.75), (0.0, 0.35, 0.75), (0.22, 0.35, 0.75), (0.0, 0.35, 0.55)]:
            p = so.Plan(aim, 0.4, 1, H, U, 32.0)
            self.assertAlmostEqual(so.norm(p.velocity(0.4, U)), 32.0, places=6)
            self.assertLess(so.norm(so.sub(p.point(0.4, U), aim)), 1e-12)

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
