#!/usr/bin/env python3
"""Ratings and the owner's consolidation question, with numbers (docs/body-motion.md 7).

The model always integrates the same six drive channels; what varies is how many ratings feed them.
Each hitter's learned swing (docs 5.1) is the average batter's drive pattern (fit_params.json) with
his own strengths, re-timed as a whole so that contact comes at the planned instant, and with its
launch coil (the whole body's turn back at launch: pelvis, trunk, arms and bat together) chosen so that the swing takes the time his decision clock allows (the average batter's
time from his first drive to contact). Strength follows body mass (torque ~ M at equal build, by
geometric similarity [E]; Yanaka & Morishita 2026: torque follows muscle and body size).

Reports:
  A. the strength range: bat speed, swing time from the first drive, sweet-spot path length, and
     length / speed (MLB's per-batter length / speed is flat at 68-70 ms, note 10 1) for a single
     strength from x0.70 to x1.30, the launch re-learned for each;
  B. the same hitter swinging easier or harder (all drive x0.8..x1.2, nothing re-learned): length,
     speed, time (MLB: at a fixed depth a slower swing is about as long, note 10 2);
  C. two hitters with the same bat speed, built differently (strong legs / weak arms and the reverse;
     a tall lean and a short thick body): what still tells them apart;
  D. the levels compared on the breakdowns: how a late start, a late foot and lunging cost each.
Usage: python3 analysis/research/body/ratings.py
"""
import copy, math, sys, os, json, time
sys.path.insert(0, os.path.dirname(__file__))
from common import *
from chain import *
import fit, swing, breakdowns

LOWER = ("legs", "rear", "lead"); TRUNK = ("core",); ARMS = ("shoulder", "wrist")


def plan_for(body, scales, wrap=0.0, shift=0.0, base=None):
    """the average pattern with per-channel strength scales (torque ~ body mass), launch wrap reduced
    by `wrap` rad (bat and arms start further round), and the whole program moved by `shift` s."""
    p = base or fit.make_plan(swing.fitted(), Body())
    p = copy.deepcopy(p)
    for n in CHANNELS:
        p.ch[n][0] *= scales.get(n, 1.0) * (body.M / 92.0 if n not in ("rear", "lead") else body.M / 92.0)
        p.ch[n][1] += shift
        if p.ch[n][2] < 0.9:
            p.ch[n][2] += shift
    st = list(p.start)
    st[0] += shift
    for k in (2, 3, 4, 5):  # the launch coil: pelvis, trunk, arms and bat start `wrap` further round (less coiled)
        st[k] += wrap
    # stature: the base starts at the same share of the stride
    st[1] *= body.H / 1.84
    p.start = st
    p.t_fd += shift
    return p


def run(body, plan, t_end=0.15):
    sim = Sim(body, plan, dt=0.001, record=True)
    return sim, sim.run(t_end=t_end, stop_at_contact=True)


def metrics(body, plan):
    sim, out = run(body, plan)
    if out["contact"] is None:
        return None
    tc, xc = out["contact"]
    t_first = min(plan.ch[n][1] for n in ("legs", "core", "shoulder"))
    # sweet-spot path from the bat's start (its azimuth rate past 10 % of its contact rate) to contact
    wc = xc[9]
    L, prev, t_bat = 0.0, None, None
    for t, x in zip(out["t"], out["x"]):
        if t_bat is None and x[9] > 0.1 * wc:
            t_bat = t
        if t_bat is not None:
            pnt = sim.sweet_spot(x)
            if prev:
                L += math.hypot(pnt[0] - prev[0], pnt[1] - prev[1])
            prev = pnt
    v = sim.sweet_spot_speed(xc)
    return {"tc": tc, "v": v, "T_first": tc - t_first, "T_bat": tc - t_bat, "L": L, "L_v": L / v}


def timed(body, scales, wrap=0.0, base=None):
    """re-time the whole program so contact comes at 0 (secant on the shift)."""
    s = 0.0
    for _ in range(8):
        m = metrics(body, plan_for(body, scales, wrap, s, base))
        if m is None:
            s -= 0.02; continue
        if abs(m["tc"]) < 2e-4:
            return m, s
        s -= m["tc"]
    return m, s


def learned(body, scales, T_target, base=None, show=False):
    """choose the launch wrap (bat's cock at launch) so that the swing from the first drive takes
    T_target: a scan over -45..+45 deg (5 deg steps), then linear interpolation between the two
    scan points that bracket the target, then one check run there."""
    scan = []
    for wd in range(-45, 46, 5):
        m, s = timed(body, scales, wd * D2R, base)
        if m is not None:
            scan.append((wd * D2R, m["T_first"]))
    if show:
        print("   scan (coil deg, + = more coiled than the average: ms from the first drive): " + ", ".join("%+.0f: %.0f" % (-w / D2R, t * 1000) for w, t in scan))
    pick = None
    for (w0, t0), (w1, t1) in zip(scan, scan[1:]):
        if (t0 - T_target) * (t1 - T_target) <= 0 and t0 != t1:
            pick = w0 + (T_target - t0) * (w1 - w0) / (t1 - t0)
            break
    if pick is None:          # out of reach: the nearest end of the scan
        pick = min(scan, key=lambda p: abs(p[1] - T_target))[0]
    m, s = timed(body, scales, pick, base)
    return m, s, pick


def main():
    fit.M_ = fit.measured()
    t0 = time.time()
    avg = Body()
    base = fit.make_plan(swing.fitted(), avg)
    m0, s0 = timed(avg, {})
    T_target = m0["T_first"]
    print("average batter: bat %.1f m/s, from first drive %.0f ms, from the bat's start %.0f ms, path %.2f m, length/speed %.1f ms" % (
        m0["v"], m0["T_first"] * 1000, m0["T_bat"] * 1000, m0["L"], m0["L_v"] * 1000))
    print()
    print("A. one strength rating, launch re-learned to the same swing time (%.0f ms from the first drive):" % (T_target * 1000))
    print("   strength  bat m/s  time ms  from bat ms  coil+ deg  path m  length/speed ms")
    A = []
    for f in (0.70, 0.85, 1.0, 1.15, 1.30):
        m, s, w = learned(avg, {n: f for n in CHANNELS}, T_target, base, show=(f in (0.7, 1.0, 1.3)))
        A.append((f, m["v"], m["L"]))
        print("   x%.2f     %6.1f   %6.0f   %6.0f      %+6.1f   %5.2f   %5.1f" % (f, m["v"], m["T_first"] * 1000, m["T_bat"] * 1000, -w / D2R, m["L"], m["L_v"] * 1000))
    # slope of length on speed
    n = len(A); mv = sum(a[1] for a in A) / n; mL = sum(a[2] for a in A) / n
    sl = sum((a[1] - mv) * (a[2] - mL) for a in A) / sum((a[1] - mv) ** 2 for a in A)
    print("   path length against bat speed: %.4f m per m/s = %.3f ft per mph (MLB across batters 0.05-0.09, note 10 1)" % (sl, sl / 0.3048 * 0.44704))
    print()
    print("B. the same hitter, easier or harder (nothing re-learned, re-timed only):")
    print("   effort  bat m/s  time ms  path m  time x speed (m)")
    for f in (0.8, 0.9, 1.0, 1.1, 1.2):
        m, s = timed(avg, {n: f for n in CHANNELS}, 0.0, base)
        print("   x%.1f    %6.1f   %6.0f   %5.2f   %6.2f" % (f, m["v"], m["T_first"] * 1000, m["L"], m["T_first"] * m["v"]))
    print()
    print("C. two hitters at the same bat speed, built differently (strengths found to match the average's bat speed; launch as the average's, re-timed only):")

    def match(body, shape):
        """common factor k on the shape so that the bat speed equals the average's, launch re-learned"""
        lo, hi = 0.4, 2.5
        for _ in range(12):
            k = 0.5 * (lo + hi)
            sc = {n: k * shape.get(n, 1.0) for n in CHANNELS}
            m, s = timed(body, sc, 0.0, base); w = 0.0
            if m["v"] < m0["v"]:
                lo = k
            else:
                hi = k
        return sc, m, s, w
    hitters = [("strong legs, weak arms", avg, {"legs": 1.3, "rear": 1.3, "lead": 1.3, "core": 1.0, "shoulder": 0.75, "wrist": 0.75}),
               ("weak legs, strong arms", avg, {"legs": 0.75, "rear": 0.75, "lead": 0.75, "core": 1.0, "shoulder": 1.3, "wrist": 1.3}),
               ("tall lean 1.95 m 85 kg", Body(1.95, 85.0), {}),
               ("short thick 1.75 m 100 kg", Body(1.75, 100.0), {})]
    res = {}
    print("   hitter                      strengths                         bat  pelvis peak  trunk peak  arm peak  coil+  late-start cost  late-foot cost")
    for name, body, shape in hitters:
        sc, m, s, w = match(body, shape)
        pl = plan_for(body, sc, w, s, base)
        sim = Sim(body, pl, dt=0.001, record=True); out = sim.run(t_end=0.05)
        r = swing.summary(sim, out)
        # a 10 ms late start of everything; a 150 ms late foot (drive on its own clock)
        pl2 = breakdowns.shifted(pl, ("legs", "core", "shoulder", "wrist", "lead"), 0.010)
        m2 = metrics(body, pl2)
        pl3 = copy.deepcopy(pl); pl3.t_fd += 0.15
        m3 = metrics(body, pl3)
        res[name] = sc
        print("   %-26s %-34s %4.1f   %5.0f      %5.0f     %5.0f   %+5.1f   %+5.1f ms %+4.1f m/s  %+5.1f ms %+4.1f m/s" % (
            name, " ".join("%s %.2f" % (k[:2], v) for k, v in sc.items() if k in ("legs", "core", "shoulder", "wrist")),
            m["v"], r["pelvis"][0], r["trunk"][0], r["arm"][0], -w / D2R,
            (m2["tc"] - m["tc"]) * 1000, m2["v"] - m["v"], (m3["tc"] - m["tc"]) * 1000, m3["v"] - m["v"]))
    print("   (probe time %.0f s)" % (time.time() - t0))


if __name__ == "__main__":
    main()
