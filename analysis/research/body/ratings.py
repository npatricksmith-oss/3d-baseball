#!/usr/bin/env python3
"""Ratings and the owner's consolidation question, with numbers (docs/body-motion.md 7).

The model always integrates the same six drive channels; what varies is how many ratings feed them.
No hitter's swing is re-learned here: every hitter uses the average batter's drive pattern
(fit_params.json, the free fit) with his own strengths. Where a table says 're-timed', the whole
program (onsets and switch-offs together, their order and gaps the average's) is moved so that the
bat reaches -8 deg at 0 ms: a description of where his swing would have to start, not a motion the
game would run. Strength follows body mass (torque ~ M at equal build, by geometric similarity [E]);
SCALING = "MH" uses Anderson 2007's normalisation by weight x height instead (note 5).

Reports:
  A. one strength rating from x0.70 to x1.30, re-timed, with a launch coil (pelvis, trunk, arms and
     bat all started further round or further back together) chosen by a scan so that the time from
     the first drive stays the average's: what such a coil would have to be to give MLB's flat
     length / speed (note 10 1). A construction, not a finding about real hitters (docs 7.2);
  B. the same hitter swinging easier or harder, all drive x0.8..x1.2, NOT re-timed: the swing starts
     when planned and arrives when the body gets there (D31); the re-timed version follows, labelled;
  C. two hitters with the same bat speed under the recommended split (lower body: legs, push, brake;
     upper body: trunk, shoulders, wrists), and two body shapes at one rating: what still tells them
     apart.
Usage: python3 analysis/research/body/ratings.py
"""
import copy, math, sys, os, json, time
sys.path.insert(0, os.path.dirname(__file__))
from common import *
from chain import *
import fit, swing, breakdowns

LOWER = ("legs", "rear", "lead"); UPPER = ("core", "shoulder", "wrist")
SCALING = os.environ.get("BB_BODY_SCALING", "M")      # "M": torque ~ mass; "MH": ~ weight x height (Anderson 2007)


def plan_for(body, scales, wrap=0.0, shift=0.0, base=None):
    """the average pattern with per-channel strength scales (torque ~ body mass), launch wrap reduced
    by `wrap` rad (bat and arms start further round), and the whole program moved by `shift` s."""
    p = base or fit.make_plan(swing.fitted(), Body())
    p = copy.deepcopy(p)
    size = body.M / 92.0 if SCALING == "M" else body.M * body.H / (92.0 * 1.84)
    for n in CHANNELS:
        # torques and the legs' linear forces alike: at equal build both go as mass (cross-section x lever,
        # and cross-section alone for a force over a leg whose length scales out) [E]
        p.ch[n][0] *= scales.get(n, 1.0) * size
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
    print("A. one strength rating, re-timed, the launch coil scanned to keep the time from the first drive at %.0f ms (a construction, docs 7.2):" % (T_target * 1000))
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
    print("B. the same hitter, easier or harder: every drive x0.8..x1.2, the program NOT moved (it starts when")
    print("   planned; D31). Bat at -8 deg: when (ms, + = late) and how fast; the fixed ball point: when, where, how fast:")
    print("   effort  bat-8 ms  m/s   ball ms   u m    m/s   from first drive ms  path m")
    pl1 = plan_for(avg, {}, 0.0, 0.0, base); s1 = Sim(avg, pl1, dt=0.0005, record=True); o1 = s1.run(t_end=0.12)
    P = s1.sweet_spot(o1["contact"][1]); t_ref = breakdowns.meet(s1, o1, P)[0]
    for f in (0.8, 0.9, 1.0, 1.1, 1.2):
        pl = plan_for(avg, {n: f for n in CHANNELS}, 0.0, 0.0, base)
        sim = Sim(avg, pl, dt=0.0005, record=True)
        out = sim.run(t_end=0.12)
        mm = metrics(avg, pl)
        mt = breakdowns.meet(sim, out, P)
        print("   x%.1f    %+6.1f   %5.1f   %+6.1f  %s  %5.1f      %4.0f            %5.2f" % (
            f, mm["tc"] * 1000, mm["v"], (mt[0] - t_ref) * 1000, ("%.3f" % mt[1]) if mt[5] else " miss", mt[2], mm["T_first"] * 1000, mm["L"]))
    print("   the same, re-timed so the bat reaches -8 deg at 0 (a description: how much earlier a planned easier or harder")
    print("   swing would have to start; not what an unplanned weak swing does):")
    print("   effort  bat m/s  time ms  path m  time x speed (m)  start moved ms")
    for f in (0.8, 0.9, 1.0, 1.1, 1.2):
        m, s = timed(avg, {n: f for n in CHANNELS}, 0.0, base)
        print("   x%.1f    %6.1f   %6.0f   %5.2f   %6.2f            %+5.1f" % (f, m["v"], m["T_first"] * 1000, m["L"], m["T_first"] * m["v"], s * 1000))
    print()
    print("C. hitters at the same bat speed (the average's), the strengths scaled together until it matches; the program")
    print("   the average's, re-timed as a whole (not re-learned for the hitter). Torque scaling: %s." % SCALING)

    def match(body, shape):
        """common factor k on the shape so that the bat speed equals the average's; the average's program,
        re-timed as a whole (nothing re-learned)"""
        lo, hi = 0.4, 2.5
        for _ in range(12):
            k = 0.5 * (lo + hi)
            sc = {n: k * shape.get(n, 1.0) for n in CHANNELS}
            m, s = timed(body, sc, 0.0, base)
            if m["v"] < m0["v"]:
                lo = k
            else:
                hi = k
        return sc, m, s
    lo_up = lambda lo_, up_: dict([(n, lo_) for n in LOWER] + [(n, up_) for n in UPPER])
    hitters = [("strong lower, weak upper", avg, lo_up(1.3, 0.75)),
               ("weak lower, strong upper", avg, lo_up(0.75, 1.3)),
               ("tall lean 1.95 m 85 kg", Body(1.95, 85.0), {}),
               ("short thick 1.75 m 100 kg", Body(1.75, 100.0), {})]
    print("   hitter                      lower upper   bat  pelvis peak  trunk peak  arm peak  pelvis at contact  10 ms late start   foot 150 ms late")
    for name, body, shape in hitters:
        sc, m, s = match(body, shape)
        pl = plan_for(body, sc, 0.0, s, base)
        sim = Sim(body, pl, dt=0.0005, record=True); out = sim.run(t_end=0.05)
        r = swing.summary(sim, out)
        # a 10 ms late start of the whole program (window moved); a 150 ms late foot (the program on its own clock)
        pl2 = breakdowns.shifted(pl, ("legs", "core", "shoulder", "wrist", "lead"), 0.010, window=True)
        m2 = metrics(body, pl2)
        pl3 = copy.deepcopy(pl); pl3.t_fd += 0.15
        m3 = metrics(body, pl3)
        print("   %-26s %5.2f %5.2f   %4.1f   %5.0f      %5.0f     %5.0f      %5.0f          %+5.1f ms %+4.1f m/s  %+5.1f ms %+4.1f m/s" % (
            name, sc["legs"], sc["core"], m["v"], r["pelvis"][0], r["trunk"][0], r["arm"][0], r["pelvis_c"],
            (m2["tc"] - m["tc"]) * 1000, m2["v"] - m["v"], (m3["tc"] - m["tc"]) * 1000, m3["v"] - m["v"]))
    print("   (probe time %.0f s)" % (time.time() - t0))


if __name__ == "__main__":
    main()
