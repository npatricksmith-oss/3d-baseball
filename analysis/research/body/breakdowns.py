#!/usr/bin/env python3
"""The owner's named breakdowns, each as an error in one link of the motion (docs/body-motion.md 6).

Every case starts from the fitted average swing (fit_params.json) and changes one thing in the plan or
the execution; nothing earlier in the motion than the error is touched. The body then gives the result.
The ball is a fixed point: where the average swing's sweet spot is at its contact. Reported:
  meet   when the bat line passes the ball point, ms against the planned contact (+ = late);
  u      where on the bat it meets it, m from the hands (sweet spot 0.634; the bat's end 0.784;
         under 0.45 the handle/jammed; past 0.784 a miss off the end) [physics-spec 6, 7];
  speed  the bat's speed there (m/s); the average swing's sweet spot is the reference;
  angle  the bat's azimuth error when it meets (deg, + = further round = pulled);
  sep    largest hip-shoulder separation (deg); pelvis and trunk at the meeting (deg).

Cases (docs 6.2):
  late foot       the lead foot lands 60 / 120 / 160 ms late; the drive runs on its own clock
  late foot, wait the same, but the legs' drive (and all after it) waits for the foot (heel plant
                  as the trigger, the owner's walkthrough): the timing error carried forward
  hips early      the legs' drive starts 20 / 40 ms early, the rest on time
  casting         the wrists' release starts 30 / 60 ms early
  pulling off     the trunk's drive starts 20 / 40 ms early (the front shoulder opens)
  lunging         fooled by off-speed: the rotation held 60 / 100 ms while the stride's drift runs on
                  (no block until the hips go)
  late swing      everything after the foot 15 ms late (a plain timing error, for scale)
  arms committed  the hips and trunk on their own clock (D22), the arms' drive 30 / 60 / 90 ms later
Usage: python3 analysis/research/body/breakdowns.py
"""
import json, math, sys, os, copy
sys.path.insert(0, os.path.dirname(__file__))
from common import *
from chain import *
import fit, swing


def base_plan(body):
    return fit.make_plan(swing.fitted(), body)


def run(body, plan, t_end=0.06):
    sim = Sim(body, plan, dt=0.0005, record=True)
    out = sim.run(t_end=t_end)
    return sim, out


def meet(sim, out, P):
    """first time the bat line passes the ball point P (sign change of the perpendicular distance, with
    the point ahead of the hands): (t, u along the bat, speed of the bat point at u, azimuth)."""
    prev = None
    for t, x in zip(out["t"], out["x"]):
        hx, hy = sim.hands(x)
        a = x[4]
        dx, dy = P[0] - hx, P[1] - hy
        along = dx * math.cos(a) + dy * math.sin(a)
        perp = -dx * math.sin(a) + dy * math.cos(a)        # > 0: P is ahead of the bat in the swing
        if prev is not None and prev[1] > 0 >= perp and along > 0:
            f = prev[1] / (prev[1] - perp)
            tm = prev[0] + f * (t - prev[0])
            # speed of the bat point at 'along'
            q, qd = x[:5], x[5:]
            vx, vy = 0.0, qd[0]
            for k, r, off in ((2, sim.b.r_sh, math.pi / 2), (3, sim.b.L_arm, 0.0), (4, along, 0.0)):
                aa = q[k] + off
                vx += -r * math.sin(aa) * qd[k]; vy += r * math.cos(aa) * qd[k]
            return tm, along, math.hypot(vx, vy), a, x
        prev = (t, perp)
    return None


def shifted(plan, names, dt):
    p = copy.deepcopy(plan)
    for n in names:
        p.ch[n][1] += dt
    return p


def main():
    fit.M_ = fit.measured()
    body = Body()
    plan0 = base_plan(body)
    sim0, out0 = run(body, plan0)
    tc0, xc0 = out0["contact"]
    P = sim0.sweet_spot(xc0)
    m0 = meet(sim0, out0, P)
    A0 = m0[3]
    ROT = ("legs", "core", "shoulder", "wrist")
    cases = [("average swing", plan0)]
    for d in (0.06, 0.12, 0.16):
        p = copy.deepcopy(plan0); p.t_fd += d
        cases.append(("late foot +%d ms" % (d * 1000), p))
    for d in (0.06, 0.12, 0.16):
        p = copy.deepcopy(plan0); p.t_fd += d
        wait = max(0.0, p.t_fd + 0.03 - p.ch["legs"][1])        # 30 ms from the plant to the drive [E]
        p = shifted(p, ROT + ("lead",), wait)
        cases.append(("late foot +%d, hips wait %d" % (d * 1000, wait * 1000), p))
    for d in (0.02, 0.04):
        cases.append(("hips early %d ms" % (d * 1000), shifted(plan0, ("legs",), -d)))
    for ton, Tw in ((-0.15, None), (-0.10, None), (-0.15, 20.0)):
        # casting: the hands throw the barrel out early, an uncocking drive the plan does not have, at
        # the batter's own wrist strength (the fit's T_wr), rising in 20 ms
        p = copy.deepcopy(plan0)
        p.ch["wrist"] = [Tw or plan0.ch["wrist"][0], ton, 1.0, plan0.ch["wrist"][3], 0.02]
        cases.append(("casting: wrists from %d ms%s" % (ton * 1000, ", 20 N m" if Tw else ""), p))
    for d in (0.02, 0.04):
        cases.append(("pulling off: trunk %d ms early" % (d * 1000), shifted(plan0, ("core",), -d)))
    for d in (0.06, 0.10):
        p = shifted(plan0, ROT + ("lead",), d)
        p.ch["rear"][2] += d                                     # the drift's push runs on
        cases.append(("lunging: hips held %d ms" % (d * 1000), p))
    cases.append(("late swing 15 ms", shifted(plan0, ROT + ("lead",), 0.015)))
    for d in (0.03, 0.06, 0.09):
        # the decision clock (docs 5.4): the hips go on their own clock (D22), the arms wait for the decision
        cases.append(("arms committed %d ms later" % (d * 1000), shifted(plan0, ("shoulder", "wrist"), d)))
    print("ball point (the average swing's sweet spot at contact): (%.3f, %.3f) m; its speed %.1f m/s; u_ss %.3f m" % (P[0], P[1], m0[2], body.ss))
    print("%-34s %7s %6s %6s %7s %6s %6s %6s %6s %6s" % ("case", "meet ms", "u m", "m/s", "angle", "sep", "pelvis", "trunk", "s m", "cap Nm"))
    res = {}
    for name, p in cases:
        sim, out = run(body, p, t_end=0.12)
        m = meet(sim, out, P)
        r = swing.summary(sim, out)
        if m is None:
            print("%-34s  never reaches the ball point" % name); continue
        tm, u, v, a, x = m
        res[name] = (tm - m0[0], u, v)
        print("%-34s %+7.1f %6.3f %6.1f %+7.1f %6.1f %6.0f %6.0f %6.3f %6.0f" % (name, (tm - m0[0]) * 1000, u, v, (a - A0) / D2R,
              r["sep"][0], x[1] / D2R, x[2] / D2R, x[0], sim.cap))
    json.dump(res, open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "breakdowns.json"), "w"), indent=1)


if __name__ == "__main__":
    main()
