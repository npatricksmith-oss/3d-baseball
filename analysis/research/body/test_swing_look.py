"""Tests of the swing written by hand and made by muscles (swing_author.py, swing_inverse.py, swing_track.py; PLAN D48,
docs/body-motion.md 4.17). They need no licensed data and no numpy except where they say so.

    python3 -m unittest analysis/research/body/test_swing_look.py            (about 2 minutes the first time)

Each test holds the code to something it did not make: the engine's own joint rows, the ground's own rule, a joint
angle read by body.py, the Python engine against the compiled one, a forward run against the plan it was given.
"""
import json
import math
import os
import subprocess
import sys
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import swing_author as A      # noqa: E402
import swing_track as T       # noqa: E402

E, B = A.E, A.B
PLAN = os.path.join(HERE, "swing_look_plan.json")


def have_rustc():
    try:
        subprocess.run(["rustc", "--version"], capture_output=True, check=True, env=dict(os.environ, PATH=os.path.expanduser("~/.cargo/bin") + ":" + os.environ["PATH"]))
        return True
    except (OSError, subprocess.CalledProcessError):
        return False


class WrittenMotion(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.m, cls.ts, cls.states, cls.worst = A.build()

    def at(self, t):
        return min(range(len(self.ts)), key=lambda k: abs(self.ts[k] - t))

    def test_every_joint_is_closed(self):
        # the engine's own constraint rows on the posed bodies: no limb is short of what was written
        s = self.m.sys
        self.assertLessEqual(max(v for v, t in self.worst.values()), 0.0, "a limb cannot reach what is written: %r" % (self.worst,))
        for k in range(0, len(self.states), 50):
            self.assertLess(s.residual(self.states[k])[0], 1e-9)

    def test_a_planted_foot_does_not_move(self):
        m, s = self.m, self.m.sys
        ball = lambda st, sd: E.add(s.pose(st, m.b["foot_" + sd])[0], E.mv(E.qR(s.pose(st, m.b["foot_" + sd])[1]), A.ball_mid(m)))
        r0 = ball(self.states[0], "R")
        for k in range(0, len(self.states), 10):
            self.assertLess(E.norm(E.sub(ball(self.states[k], "R"), r0)), 1e-3, "the rear foot's ball moved at %.3f s" % self.ts[k])
        l0, l1 = ball(self.states[0], "L"), ball(self.states[-1], "L")
        for k in range(0, len(self.states), 10):
            t = self.ts[k]
            if t < A.T_LIFT:
                self.assertLess(E.norm(E.sub(ball(self.states[k], "L"), l0)), 1e-3)
            if t > A.T_HEEL + 0.05:
                self.assertLess(E.norm(E.sub(ball(self.states[k], "L"), l1)), 2e-3, "the lead foot's ball moved at %.3f s" % t)
                self.assertLess(abs(math.atan2(*reversed(m.axis(self.states[k], "foot_L", (1, 0, 0))[:2])) - math.atan2(*reversed(m.axis(self.states[-1], "foot_L", (1, 0, 0))[:2]))), 0.02,
                                "the lead foot turned on the dirt with his weight on it at %.3f s" % t)

    def test_the_ground_is_asked_only_what_feet_can_give(self):
        # read from the motion alone (second differences): the push needed is upward and modest along the ground, and
        # outside the swing itself it acts where it was written to act, which is under a foot that is down
        rows = A.read(self.m, self.ts, self.states, 5.0)
        beyond = []
        for r in rows[2:-2]:
            t = r["t"] * 1e-3
            self.assertGreater(r["fz"], 0.5, "at %+.0f ms the ground would have to push with under half his weight" % r["t"])
            self.assertLess(r["fz"], 2.0)
            self.assertLess(math.hypot(r["fh_x"], r["fh_y"]), 0.6, "at %+.0f ms" % r["t"])
            px, py = A.push_point(t)
            front = A.LEAD_Y0 if t < A.T_OFF else A.LEAD_Y1
            on_feet = A.REAR_Y - 0.08 < r["zmp_y"] < front + 0.08 and -1.02 < r["zmp_x"] < -0.70
            if -0.130 <= t <= 0.065:
                if not on_feet:
                    beyond.append(r["t"])
                continue
            self.assertTrue(on_feet, "at %+.0f ms the push is asked at %.2f, %.2f" % (r["t"], r["zmp_x"], r["zmp_y"]))
            self.assertLess(math.hypot(r["zmp_x"] - px, r["zmp_y"] - py), 0.15, "at %+.0f ms" % r["t"])
            if A.T_OFF + 0.03 < t < A.T_TOUCH - 0.03:      # on the rear foot alone: within the foot
                self.assertLess(abs(r["zmp_y"] - A.REAR_Y), 0.09)
        # A KNOWN FLAW, held here so that it cannot grow unseen (body-motion 4.17): through the swing itself (-130 to
        # +65 ms) the written motion still asks the ground, at some instants, to push from off the ground his feet stand
        # on (the bat's pull is not fully answered by the lean written for it). The forward run cannot give that and
        # parts from the written path there by a few centimetres.
        self.assertLessEqual(len(beyond), 25, beyond)

    def test_hips_then_chest_then_bat(self):
        # what every source has (swing-synthesis 2): each part's fastest turning comes later than the one below it
        rows = A.read(self.m, self.ts, self.states, 1.0)
        sw = [r for r in rows if -200 <= r["t"] <= 60]
        t_p = max(sw, key=lambda r: r["pelvis_rate"])["t"]
        t_t = max(sw, key=lambda r: r["trunk_rate"])["t"]
        t_b = max(sw, key=lambda r: r["sweet"])["t"]
        self.assertLess(t_p, t_t)
        self.assertLess(t_t, t_b)
        at0 = min(rows, key=lambda r: abs(r["t"]))
        self.assertGreater(at0["bat_down"], 10.0)      # the barrel below the hands at contact
        self.assertGreater(at0["attack"], 0.0)         # and coming up
        self.assertGreater(max(r["sep"] for r in sw), 15.0)   # the hips get ahead of the chest

    def test_arms_near_their_ranges_until_contact(self):
        # body.py's own reading of the arm joints against a closed hand's ranges (body.ROM): up to contact no arm joint
        # is more than PAST beyond its measured end (body-motion 4.17 lists these as loosened: the rear wrist cocked 23
        # deg past, the lead forearm 6)
        PAST = {"wrist_dev": 25.0, "wrist_flex": 8.0, "forearm": 8.0, "upper_arm": 8.0}
        rng = B.arm_range()
        for k in range(0, self.at(0.0) + 1, 10):
            cl = B.clinical(self.m, self.states[k])
            for (name, sd), v in cl.items():
                if name in rng:
                    lo, hi = rng[name]
                    self.assertGreater(v, lo - PAST[name], "%s %s at %.3f s" % (name, sd, self.ts[k]))
                    self.assertLess(v, hi + PAST[name], "%s %s at %.3f s" % (name, sd, self.ts[k]))

    def test_no_recorded_swing_is_read(self):
        # PLAN D48: nothing of the OpenBiomechanics data, and none of the lab study's curves, is an input
        for f in ("swing_author.py", "swing_track.py", "swing_inverse.py", "swing_track.rs"):
            src = open(os.path.join(HERE, f)).read()
            for word in ("real_swing", "obp", "c3d", "landmarks", "curve(", "C.load(", "BB_FORT_CSV", "practice3d"):
                self.assertNotIn(word, src.replace("practice3d.py:", ""), "%s mentions %s" % (f, word))


class Muscles(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tr = T.Track()
        T.zero_plan(cls.tr)

    def test_each_muscle_turns_its_joint_the_way_its_name_says(self):
        # one muscle axis at a time is given a planned torque on a body with no gravity, no ground and no hold to the
        # motion; the joint's angle is then read by body.py's own clinical readings, which know nothing of this file
        tr = self.tr
        sy = tr.system()
        sy.g = (0.0, 0.0, 0.0)
        sy.contacts = []
        sy.stops = []
        keep = (tr.fb_scale, [list(r) for r in tr.ff])
        tr.fb_scale = 0.0
        k0 = 0
        st0 = tr.start_state(k0)
        expect = {  # + muscle of the axis: (reading, the way it must move)
            "knee_flex_L": ("knee_L", +1), "knee_flex_R": ("knee_R", +1), "elbow_flex_L": ("elbow_L", +1), "elbow_flex_R": ("elbow_R", +1),
            "ankle_pf_L": ("ankle_L", +1), "ankle_pf_R": ("ankle_R", +1), "hip_ext_L": ("hip_flex_L", +1), "hip_ext_R": ("hip_flex_R", +1),
            "hip_abd_L": ("hip_abd_L", +1), "hip_add_R": ("hip_abd_R", +1), "core": ("trunk_twist", +1), "trunk_fwd": ("trunk_bend", +1),
            "trunk_side_R": ("trunk_side", +1), "sh_raise_L": ("shoulder_up_L", -1), "sh_raise_R": ("shoulder_up_R", -1),
            "wrist_rad_L": ("wrist_dev_L", -1), "wrist_uln_R": ("wrist_dev_R", -1), "wrist_ext_L": ("wrist_flex_L", +1), "wrist_flex_R": ("wrist_flex_R", +1),
            "sup_L": ("forearm_L", -1), "pron_R": ("forearm_R", -1)}
        def after(i):
            """the readings after 6 ms with muscle axis i's + muscle asked for half its strength (i None: no muscle)."""
            for r in tr.ff:
                for q in range(tr.nax):
                    r[q] = 0.0
                if i is not None:
                    r[i] = 0.5 * tr.axes[i]["Tp"]
            st = list(st0)
            aux = sy.new_aux()
            for step in range(int(round(0.006 / T.H_STEP))):
                st = sy.step(tr.ts[k0] + step * T.H_STEP, st, aux, T.H_STEP)
            return B.range_angles(tr.m, st)
        try:
            base = after(None)                      # the stance is not quite still (he is already easing back)
            seen = 0
            for i, ax in enumerate(tr.axes):
                if ax["plus"] not in expect:
                    continue
                name, sign = expect[ax["plus"]]
                d = after(i)[name] - base[name]
                self.assertGreater(sign * d, 1e-3, "%s moved %s by %+.4f deg" % (ax["plus"], name, d))
                seen += 1
            self.assertEqual(seen, len(expect))
        finally:
            tr.fb_scale = keep[0]
            tr.ff = keep[1]

    def test_a_steady_torque_is_held_by_one_muscle_of_the_pair(self):
        # the rule the levels follow, run by itself to its rest: the pair gives the torque wanted and the opposite
        # muscle is slack (BRAKE under 1 exists for this)
        for want in (30.0, -30.0):
            ap = am = 0.4
            Tp, Tm = 100.0, 80.0
            for _ in range(4000):
                up = max(0.0, min(1.0, (want + T.BRAKE * am * Tm) / Tp))
                um = max(0.0, min(1.0, (T.BRAKE * ap * Tp - want) / Tm))
                ap += 5e-4 * (up - ap) / (E.TA_UP if up > ap else E.TA_DN)
                am += 5e-4 * (um - am) / (E.TA_UP if um > am else E.TA_DN)
            self.assertAlmostEqual(ap * Tp - am * Tm, want, delta=0.05)
            self.assertLess(min(ap, am), 1e-3)

    @unittest.skipUnless(have_rustc(), "rustc is not on this machine")
    def test_the_compiled_run_is_the_python_one(self):
        tr = self.tr
        k0 = min(range(tr.n), key=lambda k: abs(tr.ts[k] + 0.060))
        n = 30
        recs, err = T.run(tr, t_end=tr.ts[k0] + n * T.H_STEP, k0=k0, every=1)
        self.assertIsNone(err)
        py = T.run_python(tr, n, k0=k0)
        for i in (1, 10, n):
            self.assertLess(max(abs(x - y) for x, y in zip(recs[i].st, py[i])), 1e-9)


@unittest.skipUnless(have_rustc(), "rustc is not on this machine")
class KeptSwing(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tr, cls.d = T.load_plan(PLAN)
        cls.recs, cls.err = T.forward(cls.tr)
        cls.fig = T.contact_of(cls.tr, cls.recs)

    def test_it_runs_to_its_end_and_is_the_run_the_plan_file_says(self):
        self.assertIsNone(self.err)
        sm = T.summary(self.tr, self.recs, self.fig)
        for k in ("contact_after_written_ms", "sweet_speed_mps", "pelvis_off_at_end_cm"):
            self.assertAlmostEqual(sm[k], self.d["summary"][k], delta=0.2, msg=k)

    def test_nothing_but_muscles_and_the_ground(self):
        # the kinetic energy at contact is paid for by the work booked for the muscles, the joints' friction, the joint
        # ends, the ground and gravity (the engine's own work integrals), within a hundredth
        s = self.tr.m.sys
        nb = s.nb

        def energy(st):
            ke = pe = 0.0
            for i in range(nb):
                x, q, v, w = s.pose(st, i)
                l = E.mtv(E.qR(q), w)
                ke += 0.5 * s.m[i] * E.dot(v, v) + 0.5 * sum(s.I[i][c] * l[c] * l[c] for c in range(3))
                pe += s.m[i] * E.G * x[2]
            return ke, pe
        k = self.fig["k"]
        ke0, pe0 = energy(self.recs[0].st)
        ke1, pe1 = energy(self.recs[k].st)
        work = self.recs[k].st[13 * nb:13 * nb + 4]
        self.assertGreater(ke1, 200.0)
        self.assertAlmostEqual((ke1 + pe1) - (ke0 + pe0), sum(work), delta=0.01 * ke1 + 1.0)

    def test_he_stands_on_his_feet_and_finishes_on_them(self):
        tr = self.tr
        k = self.fig["k"]
        self.assertTrue(self.fig["came"])
        fz = T.loads_of(tr, self.recs[k])
        self.assertGreater(fz["L"], fz["R"])            # more on the front foot at contact
        self.assertGreater(fz["L"] + fz["R"], 1.0)
        end = T.off_motion(tr, self.recs[-1])
        self.assertLess(end[0], 0.06)                   # within 6 cm of the written finish 0.8 s after contact
        o = tr.m.observe(self.recs[k].st)
        self.assertGreater(-o["bat_elev"], 10.0)        # the barrel below the hands
        self.assertGreater(o["attack"], 0.0)            # coming up through the ball


class FirstPlan(unittest.TestCase):
    def test_the_solved_plan_holds_the_stance(self):
        # the torques the equations ask for the stance, given to the forward engine, hold the stance: two pieces of
        # code that share only the engine's joint rows
        try:
            import numpy as np
        except ImportError:
            self.skipTest("numpy is not here (uv run --with numpy)")
        if not have_rustc():
            self.skipTest("rustc is not on this machine")
        import swing_inverse as SI
        tr = T.Track()
        acc = SI.accelerations(tr)
        mus, forces, share, resid = SI.solve(tr, 0, acc, np)
        s = tr.m.sys
        self.assertLess(resid, 1e-6)
        self.assertAlmostEqual(sum(f[2] for f in forces.values()), sum(s.m) * E.G, delta=0.01 * sum(s.m) * E.G)   # the stance is not quite still
        self.assertLess(share, 0.5)
        nk = (tr.n - 1 + T.FF_EVERY - 1) // T.FF_EVERY + 1
        T.set_plan(tr, [list(mus) for _ in range(nk)])
        recs, err = T.forward(tr, t_end=tr.t0 + 0.30)
        self.assertIsNone(err)
        self.assertLess(T.off_motion(tr, recs[-1])[0], 0.004)


if __name__ == "__main__":
    unittest.main()
