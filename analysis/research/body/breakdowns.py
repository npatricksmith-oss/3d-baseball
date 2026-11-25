#!/usr/bin/env python3
"""The owner's named breakdowns, each as an error in one link of the motion (docs/body-motion.md 6.2).

Every case starts from the fitted average swing (fit_params.json, the free fit) and changes one thing in
the plan; nothing earlier in the motion than the error is touched. The body then gives the result. A case
that needs two changes says so.

A timing case can move a channel in two ways, and both are run and printed:
  window  the onset and the switch-off move together: the same burst of drive, earlier or later;
  onset   the onset moves and the switch-off stays: the burst is lengthened or cut short.
docs 6.2 names which one each baseball fault means. (The first version moved the onset alone everywhere
and reported it as the fault: the 2026-11-25 reviews.)

The ball. Two tests, both with the real bat (0 to 0.784 m from the hands; past the end is a miss):
  fixed   a fixed point where the average swing's sweet spot is at contact: when the bat line passes it,
          where on the bat, how fast. A geometric sensitivity, not contact: a real ball is not there
          early or late;
  moving  a ball on the pitch line through that point at 40 m/s [E: a 90 mph pitch; [4]'s was 56 mph],
          timed to reach it at the average swing's contact: the first time the bat passes the ball.
          Still no collision, no height (the probe is planar), no batted ball.
Reported for each: meeting time against the average (ms, + = late), u (m from the hands; sweet spot
0.634, end 0.784; 'miss' past the end), the bat's speed there, its azimuth against the average's (+ =
further round, toward the pull side). Also: peak hip-shoulder separation; pelvis and trunk at the fixed
meeting; the friction cap on the legs' yaw torque at the moment the legs' drive peaks, and on how many
steps the drive was above the cap (0 = the cap never acted).

Cases (docs 6.2):
  late foot       the lead foot lands 60 / 120 / 160 ms late; the drive runs on its own clock (one change:
                  the touch-down time, a planar stand-in for a stride error, docs 6.1)
  late foot, wait the same, and the program waits for the sensed foot (two changes: the foot, and the
                  trigger's consequence). The trigger: the lead foot's load reaching 10 % BW ([4]'s foot-down
                  definition) plus a 30 ms latency [E]; the program may not start before it. Found
                  causally: the body is run with the program held to find the event, then from it.
  hips early      the legs' drive 20 / 40 ms early
  casting         an uncocking wrist drive the plan does not have, from -150 / -100 ms, at the two wrists'
                  measured strength (20 N m, note 5, abstract only) and half of it
  pulling off     the trunk's drive 20 / 40 ms early
  lunging         fooled by off-speed: the rotation and the brake held 60 / 100 ms; with and without the
                  stride's push running on for the same time (with it: two changes)
  late swing      the whole program after the foot 15 ms late (a plain timing error, for scale)
  arms committed  the shoulder's and wrist's drive 30 / 60 / 90 ms later (the shoulder never switches off,
                  so window and onset are the same)
Usage: python3 analysis/research/body/breakdowns.py
"""
import json, math, sys, os, copy
sys.path.insert(0, os.path.dirname(__file__))
from common import *
from chain import *
import fit, swing

BAT_END = BAT_L - GRIP_FROM_KNOB         # 0.784 m from the hands
V_BALL = 40.0                            # [E] m/s
LAT = 0.030                              # [E] the foot trigger's latency
ROT = ("legs", "core", "shoulder", "wrist")


def base_plan(body):
    return fit.make_plan(swing.fitted(), body)


def run(body, plan, t_end=0.14):
    sim = Sim(body, plan, dt=0.0005, record=True)
    out = sim.run(t_end=t_end)
    return sim, out


def bat_speed_at(sim, x, u):
    q, qd = x[:5], x[5:]
    vx, vy = 0.0, qd[0]
    for k, r, off in ((2, sim.b.r_sh, math.pi / 2), (3, sim.b.L_arm, 0.0), (4, u, 0.0)):
        aa = q[k] + off
        vx += -r * math.sin(aa) * qd[k]; vy += r * math.cos(aa) * qd[k]
    return math.hypot(vx, vy)


def meet(sim, out, P, t0=None):
    """first time the bat line passes the ball, ahead of the hands. The ball is the fixed point P, or (t0
    given) moving along -y through P, there at t0. Returns (t, u, speed at u, azimuth, x, hit) with
    hit False when u is past the bat's end."""
    prev = None
    for t, x in zip(out["t"], out["x"]):
        bx, by = P if t0 is None else (P[0], P[1] - V_BALL * (t - t0))
        hx, hy = sim.hands(x)
        a = x[4]
        dx, dy = bx - hx, by - hy
        along = dx * math.cos(a) + dy * math.sin(a)
        perp = -dx * math.sin(a) + dy * math.cos(a)        # > 0: the ball is ahead of the bat in the swing
        if prev is not None and prev[1] > 0 >= perp and along > 0:
            f = prev[1] / (prev[1] - perp)
            tm = prev[0] + f * (t - prev[0])
            return tm, along, bat_speed_at(sim, x, min(along, BAT_END)), a, x, along <= BAT_END
        prev = (t, perp)
    return None


def shifted(plan, names, dt, window=True):
    """move each named channel's onset by dt; with window=True its switch-off too (when it has one)."""
    p = copy.deepcopy(plan)
    for n in names:
        p.ch[n][1] += dt
        if window and p.ch[n][2] < 0.9:
            p.ch[n][2] += dt
    return p


def foot_event(body, plan, frac=0.10):
    """the time the lead foot's load reaches frac x BW, with the program held (nothing has started that
    the event could depend on, so it is the same in the run that follows)."""
    p = shifted(plan, ROT + ("lead",), 1.0, window=True)
    sim = Sim(body, p, dt=0.0005, record=True)
    out = sim.run(t_end=0.10)
    W = body.M * G
    for t, x in zip(out["t"], out["x"]):
        Fl, _ = sim.loads(t, x[0])
        if Fl >= frac * W:
            return t
    return None


def legs_cap(out, t_stop):
    """the friction cap when the legs' drive peaks before t_stop, and the steps with the drive above the cap."""
    pre = [(t, raw, cap) for t, raw, cap in zip(out["t"], out["legs_raw"], out["cap"]) if t <= t_stop]
    tp, rawp, capp = max(pre, key=lambda r: r[1])
    n_capped = sum(1 for _, raw, cap in pre if abs(raw) > cap)
    return rawp, capp, tp, n_capped


def main():
    fit.M_ = fit.measured()
    body = Body()
    plan0 = base_plan(body)
    sim0, out0 = run(body, plan0)
    tc0, xc0 = out0["contact"]
    P = sim0.sweet_spot(xc0)
    m0 = meet(sim0, out0, P)
    mv0 = meet(sim0, out0, P, t0=tc0)
    A0 = m0[3]
    cases = [("average swing", "", plan0)]
    for d in (0.06, 0.12, 0.16):
        p = copy.deepcopy(plan0); p.t_fd += d
        cases.append(("late foot +%d ms" % (d * 1000), "", p))
    t_on_first = min(plan0.ch[n][1] for n in ROT + ("lead",))
    waits = {}
    for d in (0.06, 0.12, 0.16):
        p = copy.deepcopy(plan0); p.t_fd += d
        ev = foot_event(body, p)
        wait = max(0.0, ev + LAT - t_on_first)
        waits[d] = (ev, wait)
        for w in (True, False):
            cases.append(("late foot +%d, waits %d" % (d * 1000, wait * 1000), "window" if w else "onset", shifted(p, ROT + ("lead",), wait, w)))
    for d in (0.02, 0.04):
        for w in (True, False):
            cases.append(("hips %d ms early" % (d * 1000), "window" if w else "onset", shifted(plan0, ("legs",), -d, w)))
    for ton in (-0.15, -0.10):
        for Tw in (20.0, 10.0):
            p = copy.deepcopy(plan0)
            p.ch["wrist"] = [Tw, ton, 1.0, plan0.ch["wrist"][3], 0.02]   # rises in 20 ms [E]
            cases.append(("casting from %d ms, %d N m" % (ton * 1000, Tw), "", p))
    for d in (0.02, 0.04):
        for w in (True, False):
            cases.append(("pulling off: trunk %d ms early" % (d * 1000), "window" if w else "onset", shifted(plan0, ("core",), -d, w)))
    for d in (0.06, 0.10):
        for w in (True, False):
            p = shifted(plan0, ROT + ("lead",), d, w)
            cases.append(("lunging: held %d ms" % (d * 1000), "window" if w else "onset", p))
            p2 = copy.deepcopy(p); p2.ch["rear"][2] += d               # the stride's push runs on: a second change
            cases.append(("lunging: held %d, push on" % (d * 1000), "window" if w else "onset", p2))
    for w in (True, False):
        cases.append(("late swing 15 ms", "window" if w else "onset", shifted(plan0, ROT + ("lead",), 0.015, w)))
    for d in (0.03, 0.06, 0.09):
        cases.append(("arms committed %d ms later" % (d * 1000), "", shifted(plan0, ("shoulder", "wrist"), d)))
    print("ball point (the average swing's sweet spot at contact): (%.3f, %.3f) m; bat speed there %.1f m/s; sweet spot %.3f m, bat end %.3f m from the hands" % (
        P[0], P[1], m0[2], body.ss, BAT_END))
    print("foot trigger (load 10 %% BW + %d ms): " % (LAT * 1000) + ", ".join("foot +%d ms: event %.0f ms, program waits %.0f ms" % (d * 1000, ev * 1000, w * 1000) for d, (ev, w) in waits.items()))
    print("columns: fixed point: meet ms, u m, m/s, angle deg | moving ball: meet ms, u m, m/s | sep peak, pelvis / trunk at the fixed meeting | legs' peak drive N m, cap then, steps capped")
    res = {}
    for name, how, p in cases:
        sim, out = run(body, p)
        m = meet(sim, out, P)
        mv = meet(sim, out, P, t0=tc0)
        if m is None:
            print("%-30s %-6s never reaches the ball point" % (name, how)); continue
        tm, u, v, a, x, hit = m
        r = swing.summary(sim, {"t": out["t"], "x": out["x"], "contact": (tm, x)})
        rawp, capp, tp, ncap = legs_cap(out, tm)
        if mv:
            mvs = "%+6.1f %6s %5.1f" % ((mv[0] - mv0[0]) * 1000, ("%.3f" % mv[1]) if mv[5] else "miss", mv[2] if mv[5] else float("nan"))
        else:
            mvs = "  none"
        print("%-30s %-6s %+6.1f %6s %5.1f %+5.1f | %s | %5.1f %4.0f / %3.0f | %4.0f %4.0f %3d" % (
            name, how, (tm - m0[0]) * 1000, ("%.3f" % u) if hit else "miss", v, (a - A0) / D2R, mvs,
            r["sep"][0], x[1] / D2R, x[2] / D2R, rawp, capp, ncap))
        res[name + (" [" + how + "]" if how else "")] = {"fixed": [tm - m0[0], u, v, hit], "moving": ([mv[0] - mv0[0], mv[1], mv[2], mv[5]] if mv else None),
                                                          "pelvis": x[1] / D2R, "trunk": x[2] / D2R, "legs_peak": rawp, "cap_then": capp, "steps_capped": ncap}
    json.dump(res, open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "breakdowns.json"), "w"), indent=1)


if __name__ == "__main__":
    main()
