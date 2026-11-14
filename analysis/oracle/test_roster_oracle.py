"""Checks on the roster oracle itself; run with python3 -m unittest analysis/oracle/test_roster_oracle.py."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import roster_oracle as o  # noqa: E402  (its import asserts the Random123 known answer)


class RosterOracle(unittest.TestCase):
    def test_golden_seeds_reject_a_family_name_and_a_number(self):
        for seed in (18, 19):
            _, log = o.generate(seed)
            self.assertGreater(log["family_rejections"], 0, seed)
            self.assertGreater(log["number_rejections"], 0, seed)

    def test_golden_file_is_the_oracles_output(self):
        want = []
        for seed in (18, 19):
            teams, _ = o.generate(seed)
            want += [f"{seed} {side} {p[0]} {p[1]} {p[2]} {p[3]} {p[4]} {p[5]}" for side, t in enumerate(teams) for p in t]
        got = [l for l in (Path(__file__).parent / "golden/rosters.txt").read_text().splitlines() if not l.startswith("#")]
        self.assertEqual(got, want)


if __name__ == "__main__":
    unittest.main()
