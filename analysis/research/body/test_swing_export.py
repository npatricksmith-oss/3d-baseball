"""Checks of the body-swing file (swing_export.py; docs/player-visuals.md 14) against things the exporter did not make.

    python3 -m unittest analysis/research/body/test_swing_export.py

- the sweet spot's speed and the miss at contact equal the practice search's own figures for that plan (its report,
  kept in the plan's file: 21.11 m/s and 0.035, 0.019 cm for rung 4);
- every rigid segment's length between its two joints, read from the file's joint columns, does not change;
- the hands do not slide on the handle: each wrist's place along the bat, read from the file's knob and tip, does not change;
- the ball passes the meeting point at contact at the pitch's speed, and the file's sweet spot at contact is the
  contact line's.
The file is parsed here by its own few lines, not by the exporter.
"""
import json
import math
import os
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
PLAN = os.path.join(HERE, "practice3d_s11_rung4.json")
LEN_TOL = 1e-3          # m: a segment's length may wander by the engine's joint gap (0.2 mm in rung 4's file), no more


def parse(path):
    head, rows = {}, []
    for line in open(path).read().splitlines():
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        k, _, rest = line.partition(" ")
        if k == "s":
            rows.append([float(x) for x in rest.split()])
        else:
            head.setdefault(k, []).append(rest)
    return head, rows


def dist(a, b):
    return math.sqrt(sum((x - y) ** 2 for x, y in zip(a, b)))


@unittest.skipUnless(os.path.isdir(os.environ.get("BB_FORT_CSV", os.path.join(HERE, "..", "..", "..", "sim", "target", "research-dl", "4", "csv"))),
                     "needs [4]'s digitised curves in sim/target/research-dl/4/csv (analysis/research/fortenbaugh_appendix_e.py)")
class SwingFile(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        sys.path.insert(0, HERE)
        import swing_export
        cls.dir = tempfile.TemporaryDirectory()
        out = os.path.join(cls.dir.name, "rung4.swing")
        swing_export.export(PLAN, out=out, log=lambda s: None)
        cls.head, cls.rows = parse(out)
        cls.plan = json.load(open(PLAN))
        h = cls.head
        cls.bodies = h["bodies"][0].split()[1:]
        cls.joints = h["joints"][0].split()[1:]
        cls.points = h["points"][0].split()[1:]
        cls.ncon = int(h["contacts"][0].split()[0])
        j0 = 1 + 7 * len(cls.bodies)
        p0 = j0 + 3 * len(cls.joints)
        cls.J = lambda self_, r, n: r[j0 + 3 * self_.joints.index(n):j0 + 3 * self_.joints.index(n) + 3]
        cls.P = lambda self_, r, n: r[p0 + 3 * self_.points.index(n):p0 + 3 * self_.points.index(n) + 3]
        cls.width = p0 + 3 * len(cls.points) + 1 + 4 * cls.ncon + 3

    @classmethod
    def tearDownClass(cls):
        cls.dir.cleanup()

    def test_layout(self):
        self.assertEqual(int(self.head["columns"][0]), self.width)
        self.assertEqual(int(self.head["samples"][0]), len(self.rows))
        for r in self.rows:
            self.assertEqual(len(r), self.width)
        dt = [b[0] - a[0] for a, b in zip(self.rows, self.rows[1:])]
        self.assertAlmostEqual(min(dt), 1e-3, delta=1e-7)
        self.assertAlmostEqual(max(dt), 1e-3, delta=1e-7)

    def test_contact_matches_the_practice_result(self):
        self.assertAlmostEqual(float(self.head["contact_speed_mps"][0]), self.plan["sweet_speed"], delta=1e-3)
        self.assertAlmostEqual(float(self.head["contact_speed_mph"][0]), self.plan["sweet_speed"] / 0.44704, delta=0.01)
        f = self.head["miss_cm"][0].split()
        self.assertAlmostEqual(float(f[1]), self.plan["miss_cm"][0], delta=1e-3)
        self.assertAlmostEqual(float(f[3]), self.plan["miss_cm"][1], delta=1e-3)
        # the contact's time is the practice report's, measured from its start
        ev = {e.split()[0]: float(e.split()[1]) for e in self.head["event"]}
        self.assertAlmostEqual(-ev["start"] * 1e3, self.plan["start_to_contact_ms"], delta=1e-3)
        self.assertAlmostEqual(ev["launch"] * 1e3, self.plan["bat_first_motion_ms"], delta=1e-3)

    def test_segments_keep_their_lengths(self):
        pairs = [("hip_L", "knee_L"), ("knee_L", "ankle_L"), ("hip_R", "knee_R"), ("knee_R", "ankle_R"),
                 ("shoulder_L", "elbow_L"), ("elbow_L", "wrist_L"), ("shoulder_R", "elbow_R"), ("elbow_R", "wrist_R"),
                 ("lumbar", "hip_L"), ("lumbar", "hip_R"), ("hip_L", "hip_R"),
                 ("lumbar", "shoulder_L"), ("lumbar", "shoulder_R"), ("shoulder_L", "shoulder_R"), ("wrist_L", "wrist_R")]
        for a, b in pairs:
            ls = [dist(self.J(r, a), self.J(r, b)) for r in self.rows]
            self.assertLess(max(ls) - min(ls), LEN_TOL, "%s-%s from %.4f to %.4f m" % (a, b, min(ls), max(ls)))
        for a, b in [("knob", "tip"), ("knob", "sweet")]:
            ls = [dist(self.P(r, a), self.P(r, b)) for r in self.rows]
            self.assertLess(max(ls) - min(ls), 1e-5)    # the file keeps 7 significant digits

    def test_hands_stay_on_the_handle(self):
        for w in ("wrist_L", "wrist_R"):
            along, off = [], []
            for r in self.rows:
                k, t = self.P(r, "knob"), self.P(r, "tip")
                ax = [(y - x) / dist(k, t) for x, y in zip(k, t)]
                d = [y - x for x, y in zip(k, self.J(r, w))]
                a = sum(x * y for x, y in zip(d, ax))
                along.append(a)
                off.append(math.sqrt(max(0.0, sum(x * x for x in d) - a * a)))
            self.assertLess(max(along) - min(along), LEN_TOL, w)
            self.assertLess(max(off) - min(off), LEN_TOL, w)
            self.assertLess(max(along), 0.25, w)       # on the handle, not the barrel

    def test_ball_line(self):
        c = self.head["contact"][0].split()
        meet = [float(x) for x in c[c.index("meet_point") + 1:c.index("meet_point") + 4]]
        r0 = min(self.rows, key=lambda r: abs(r[0]))
        nxt = self.rows[self.rows.index(r0) + 1]
        b0, b1 = r0[-3:], nxt[-3:]
        speed = dist(b0, b1) / (nxt[0] - r0[0])
        self.assertAlmostEqual(speed, 37.98, delta=1e-3)
        at0 = [x + (y - x) * (0.0 - r0[0]) / (nxt[0] - r0[0]) for x, y in zip(b0, b1)]
        self.assertLess(dist(at0, meet), 1e-5)
        # the sweet spot at contact is on the ball's plane, within the miss the plan reports
        sw = [x + (y - x) * (0.0 - r0[0]) / (nxt[0] - r0[0]) for x, y in zip(self.P(r0, "sweet"), self.P(nxt, "sweet"))]
        self.assertLess(dist(sw, meet), 0.01)

    def test_feet_on_the_ground_at_the_start(self):
        # the sole's points sit at the ground (z = 0) within the contact's sink where the model says they carry load
        c0 = 1 + 7 * len(self.bodies) + 3 * len(self.joints) + 3 * len(self.points) + 1
        r = self.rows[0]
        for i in range(self.ncon):
            x, y, z, fz = r[c0 + 4 * i:c0 + 4 * i + 4]
            if fz > 0.0:
                self.assertLess(abs(z), 0.01)


if __name__ == "__main__":
    unittest.main()
