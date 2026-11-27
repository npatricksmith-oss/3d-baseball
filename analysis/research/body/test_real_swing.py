"""Checks of a recorded swing's viewer file (real_swing.py; docs/research/swing/12-real-swing-on-the-figure.md) against
things the exporter did not make: the raw C3D markers read here directly, HitTrax's own measurement of where the ball was met,
and the body model's de Leva legs.

How to run it (from the repository root; numpy and the C3D reader come through uv, the system Python has neither):

    uv run --with numpy --with c3d python3 -m unittest analysis/research/body/test_real_swing.py

SKIPPED, not failed, when numpy or the c3d module is missing (plain `python3 -m unittest`) and when OpenBiomechanics' files
are not on this machine (real_swing.py's docstring says how to fetch them; they are CC BY-NC-SA and never in git), which is
everywhere but the machine that fetched them: a skip here says nothing about the mapping. The skip's reason is printed. The file is parsed here by its own few lines and the lab-to-sim turn is written out again from the dataset's
README ("+x from home plate toward the pitcher's mound, +y toward the right-handed batter's box, +z up").
"""
import math
import os
import sys
import tempfile
import unittest

import importlib.util

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
# a missing module skips the tests (with the reason), it does not error
MISSING = [m for m in ("numpy", "c3d") if importlib.util.find_spec(m) is None]
if not MISSING:
    import numpy as np
    import real_swing as RS   # noqa: E402  (only for where the data are and the swing's id: nothing it computes is trusted)
    SID = RS.DEFAULT_SWING
    HAVE = os.path.isdir(os.path.join(RS.OBP, "c3d")) and os.path.isfile(os.path.join(RS.OBP, "landmarks.csv"))
    WHY = "needs OpenBiomechanics' hitting C3D and landmarks under %s (real_swing.py's docstring)" % RS.OBP
else:
    HAVE = False
    WHY = "needs %s: run it with uv run --with numpy --with c3d python3 -m unittest %s" % (" and ".join(MISSING), os.path.relpath(__file__))


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
    return head, np.array(rows)


@unittest.skipUnless(HAVE, WHY)
class RealSwingFile(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.dir = tempfile.TemporaryDirectory()
        out = os.path.join(cls.dir.name, "real.swing")
        sw = RS.Swing(SID)
        sw.write(out, log=lambda s: None)
        cls.head, cls.rows = parse(out)
        h = cls.head
        cls.bodies = h["bodies"][0].split()[1:]
        cls.joints = h["joints"][0].split()[1:]
        cls.points = h["points"][0].split()[1:]
        cls.contacts = h["contacts"][0].split()[1:]
        cls.j0 = 1 + 7 * len(cls.bodies)
        cls.p0 = cls.j0 + 3 * len(cls.joints)
        cls.c0 = cls.p0 + 3 * len(cls.points) + 1
        cls.t = cls.rows[:, 0]
        cls.ev = {e.split()[0]: float(e.split()[1]) for e in h["event"]}
        cls.scale = float(h["scale"][0].split()[0])
        cls.placed = np.array([float(x) for x in h["placed"][0].split()[:3]])
        # the raw markers, read here
        import c3d
        with open(sw.c3d, "rb") as f:
            r = c3d.Reader(f)
            labels = [l.strip() for l in r.point_labels]
            X = np.array([p[:, :3].copy() for _, p, _ in r.read_frames()])
        cls.mk = {l: X[:, k] for k, l in enumerate(labels)}
        cls.lm = sw.lm
        cls.hit = sw.hit

    @classmethod
    def tearDownClass(cls):
        cls.dir.cleanup()

    def J(self, n):
        k = self.j0 + 3 * self.joints.index(n)
        return self.rows[:, k:k + 3]

    def P(self, n):
        k = self.p0 + 3 * self.points.index(n)
        return self.rows[:, k:k + 3]

    def C(self, n):
        k = self.c0 + 4 * self.contacts.index(n)
        return self.rows[:, k:k + 3]

    def sim(self, lab):
        """the README's lab frame to the sim frame (AGENTS.md), then the file's own scale and placement."""
        x, y, z = lab[:, 0], lab[:, 1], lab[:, 2]
        return np.stack([-y, x, z], axis=1) * self.scale + self.placed

    def test_layout(self):
        self.assertEqual(int(self.head["columns"][0]), self.rows.shape[1])
        self.assertEqual(int(self.head["samples"][0]), len(self.rows))
        self.assertEqual(len(self.rows), len(self.mk["LANK"]))           # every recorded sample, none made up
        self.assertTrue(np.allclose(np.diff(self.t), 1 / 360.0, atol=2e-4))
        self.assertAlmostEqual(self.t[np.argmin(np.abs(self.t))], 0.0, delta=1.5e-3)

    def test_joints_are_his_markers(self):
        # each joint the file has against the middle of the two markers either side of it on his skin (the dataset's joint
        # centres are its own model's, not those midpoints: they agree within a few mm), scaled and placed as the header says
        tol = 0.010
        for j, a, b in (("ankle_L", "LANK", "LMANK"), ("ankle_R", "RANK", "RMANK"), ("knee_L", "LKNE", "LMKNE"), ("knee_R", "RKNE", "RMKNE"),
                        ("elbow_L", "LELB", "LMELB"), ("elbow_R", "RELB", "RMELB"), ("wrist_L", "LWRA", "LWRB"), ("wrist_R", "RWRA", "RWRB")):
            d = np.linalg.norm(self.J(j) - self.sim((self.mk[a] + self.mk[b]) / 2), axis=1)
            self.assertLess(d.max(), tol, "%s off the middle of %s and %s by %.1f mm" % (j, a, b, 1e3 * d.max()))
        # the bat: its knob at the knob-end marker, its tip toward the middle of the two barrel-end markers, its length theirs
        self.assertLess(np.linalg.norm(self.P("knob") - self.sim(self.mk["Marker1"]), axis=1).max(), 1e-4)
        far = (self.mk["Marker2"] + self.mk["Marker3"]) / 2
        d = np.linalg.norm(self.P("tip") - (self.P("knob") + (self.sim(far) - self.sim(self.mk["Marker1"])) / self.scale), axis=1)
        self.assertLess(d.max(), 0.003)
        # the shoulders: squared to the trunk's axis, at most 25 mm from the dataset's shoulder centres (scaled)
        for sd, k in (("L", "lsjc"), ("R", "rsjc")):
            ref = self.sim(np.stack([self.lm[k + "_x"], self.lm[k + "_y"], self.lm[k + "_z"]], axis=1))
            self.assertLess(np.linalg.norm(self.J("shoulder_" + sd) - ref, axis=1).max(), 0.025)

    def test_scale_is_the_model_leg(self):
        sys.path.insert(0, os.path.join(HERE, "..", "..", "oracle"))
        import common as C
        model_leg = C.segment("thigh", 1.84, 92.0)[0] + C.segment("shank", 1.84, 92.0)[0]
        legs = [np.median(np.linalg.norm(self.J("hip_" + s) - self.J("knee_" + s), axis=1)) + np.median(np.linalg.norm(self.J("knee_" + s) - self.J("ankle_" + s), axis=1)) for s in "LR"]
        self.assertAlmostEqual(np.mean(legs) / model_leg, 1.0, delta=0.005)

    def test_contact_where_hittrax_saw_it(self):
        # HitTrax (cameras over the plate, not the motion capture) measured the impact's height; the motion capture's sweet
        # spot at the contact instant, scaled, must be near it. The x and y are the placement itself and are not a check.
        ic = int(np.argmin(np.abs(self.t)))
        z = self.P("sweet")[ic, 2]
        self.assertLess(abs(z / self.scale - float(self.hit["poi_y"]) * 0.0254), 0.04)
        # the ball touches the barrel's face there: its centre a barrel's and a ball's radius from the sweet spot
        ball = self.rows[ic, -3:]
        self.assertAlmostEqual(np.linalg.norm(ball - self.P("sweet")[ic]), 0.0333 + 0.0366, delta=1e-3)
        # and it is coming from the pitcher: before contact it is farther toward +Y
        self.assertGreater(self.rows[ic - 36, -2], ball[1] + 1.0)

    def test_right_hander_in_the_box(self):
        a0L, a0R = self.J("ankle_L")[0], self.J("ankle_R")[0]
        self.assertLess(max(a0L[0], a0R[0]), -0.368)                   # both feet off the plate, behind the box's inner line
        self.assertGreater(a0L[1], a0R[1])                                # lead foot toward the pitcher
        hips = (self.J("hip_L")[0] + self.J("hip_R")[0]) / 2
        left = self.J("hip_L")[0] - self.J("hip_R")[0]
        facing = np.cross(left, [0, 0, 1.0])                              # forward = left x up
        self.assertGreater(facing[0], 0.0)                                # facing the plate (+X)
        self.assertLess(hips[0], 0.0)

    def test_feet_still_while_planted(self):
        # in the stance (until 100 ms before his lead heel is 2 cm up: it starts to rise before that) every sole point stays
        # within 1.5 cm across the ground; from 30 ms after foot plant to 300 ms after contact the lead heel stays within 3 cm (he then
        # opens the toe on it and, later, steps): no skating
        stance = self.t < self.ev["lead_heel_up"] - 0.100
        for c in self.contacts:
            p = self.C(c)[stance][:, :2]
            self.assertLess(np.linalg.norm(p - p[0], axis=1).max(), 0.015, c)
        sl = (self.t >= self.ev["foot_plant"] + 0.030) & (self.t <= 0.300)        # the heel settles 2-3 cm in its first 30 ms down
        p = ((self.C("L_heel_in") + self.C("L_heel_out")) / 2)[sl][:, :2]          # the heel's middle: its edges swing round it as the toe opens
        self.assertLess(np.linalg.norm(p - p[0], axis=1).max(), 0.030)
        # and the soles are on the ground where they carry him: each foot's lowest point within 2 cm of z = 0 in the stance
        for sd in "LR":
            low = np.min(np.stack([self.C(sd + "_" + k)[stance, 2] for k in ("heel_in", "heel_out", "ball_in", "ball_out")]), axis=0)
            self.assertLess(np.abs(low).max(), 0.02, sd)

    def test_hands_on_the_bat(self):
        # each hand's far point (the file's grip point, his hand's centre) on the handle at every sample: within 7 cm of the
        # bat's axis and between the knob and 25 cm up it (he never lets go in this swing)
        k, tip = self.P("knob"), self.P("tip")
        ax = (tip - k) / np.linalg.norm(tip - k, axis=1, keepdims=True)
        for g in ("grip_L", "grip_R"):
            d = self.P(g) - k
            along = np.sum(d * ax, axis=1)
            off = np.linalg.norm(d - ax * along[:, None], axis=1)
            self.assertLess(off.max(), 0.07, g)
            self.assertGreater(along.min(), -0.02, g)
            self.assertLess(along.max(), 0.25, g)


if __name__ == "__main__":
    unittest.main()
