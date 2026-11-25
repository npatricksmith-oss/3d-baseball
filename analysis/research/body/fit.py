#!/usr/bin/env python3
"""Fit the average batter's drive to [4]'s measured swing (docs/body-motion.md 4.3).

The forward model of chain.py, started from [4]'s mean state at -250 ms, with the drive's numbers
(per channel a strength, an onset and a rise; switch-offs for the legs and the trunk; the push and
the brake along the stride) found by Nelder-Mead so that the pelvis, upper-trunk and bat-azimuth rates
follow [4]'s Appendix E curves from -250 ms to contact, contact (the bat at [4]'s -8 deg azimuth) comes
at 0 ms, and the body's forward travel follows [4]'s pelvis. Each rate's misfit is scaled by a band
read off [4]'s graphs [E: pelvis 80, trunk 100, bat 250 deg/s, about one between-batter SD]; the
cost is a sum of squared scaled residuals per sampled time, not a statistic (docs 4.3).

Every parameter is visible to the cost: each onset and switch-off is kept inside the scored window
(-250 ms to contact), so no channel can drift unseen after it.

Two rule sets (--rules):
  free     strengths >= 0, rises 0-250 ms, onsets and switch-offs inside the window; the stiff
           (one-sided) wrist stop. This is the fit that reproduces [4].
  design   the design's own rules (docs 4.3, 5.1, 5.4): the arms' onsets (shoulder, wrist) no earlier
           than -170 ms ([4]'s first move, the commitment plus its latency); every rotational rise at
           least 50 ms [E: Maffiuletti 2016, abstract only, gives 50-75 ms of early force development];
           strengths inside the bounds of BOUNDS below, each with its source; the wrist's cock held by at
           most 20 N m (two wrists, note 5) instead of the stiff stop.
Bounds are enforced by projection (the cost is evaluated at the clamped point, plus a penalty on the
distance outside), so the search can run along a bound; the report says which parameters end on one.

Starts: start 0 is X0 (a documented hand estimate from inverse.py's torques); the others are X0 with
deterministic perturbations (random.Random(k)); each start is reported.

Usage: python3 analysis/research/body/fit.py [--rules free|design] [--starts N] [--iters N] [--resume]
       (cold start from X0 by default, so a rerun reproduces the tracked file; --resume starts from it)
Writes fit_params.json (free) or fit_design.json (design) beside it. About 1 min per start: run it
through the job queue (bin/job.sh).
"""
import json, math, sys, os, time, random
sys.path.insert(0, os.path.dirname(__file__))
from common import *
from chain import *

T0 = -0.25
HERE = os.path.dirname(os.path.abspath(__file__))
H0, M0 = 1.84, 92.0
WH = M0 * G * H0                                   # weight x height, the torque scale of Anderson 2007


def measured():
    m = {}
    for n in ("pelvis_rot", "trunk_rot", "pelvis_rot_vel", "trunk_rot_vel", "bat_az", "bat_az_vel", "bat_lag", "pelvis_x_cm"):
        m[n] = smooth(resample(load(n), -0.30, 0.05, 0.001), 4)
    return m


M_ = None


def mval(n, t):
    i = int(round((t + 0.30) / 0.001))
    return M_[n][i]


def start_state(body):
    d = D2R
    s0 = mval("pelvis_x_cm", T0) / 100 + 0.77 * body.H / 1.84     # [E] rear foot 0.77 m behind [4]'s pelvis zero (docs 4.2)
    sd = (mval("pelvis_x_cm", T0 + 0.005) - mval("pelvis_x_cm", T0 - 0.005)) / 100 / 0.01
    A = mval("bat_az", T0) * d; lag = mval("bat_lag", T0) * d
    Ad = mval("bat_az_vel", T0) * d
    return [T0, s0, mval("pelvis_rot", T0) * d, mval("trunk_rot", T0) * d, A + lag, A,
            sd, mval("pelvis_rot_vel", T0) * d, mval("trunk_rot_vel", T0) * d, Ad, Ad]


# fixed [E] maximum shortening speeds per channel (rad/s, m/s): docs 5.1 lists them as unsourced
WMAX = {"legs": 20.0, "core": 25.0, "shoulder": 30.0, "wrist": 40.0, "rear": 3.0, "lead": 3.0}
NAMES = ["T_legs", "on_legs", "T_core", "on_core", "T_sh", "on_sh", "T_wr", "on_wr", "T_rear", "T_lead", "on_lead",
         "rise_legs", "rise_core", "rise_sh", "rise_wr", "off_legs", "off_core"]
X0 = [769, -0.25, 219, -0.185, 275, -0.196, 20, -0.05, 9, 253, -0.10, 0.08, 0.05, 0.05, 0.05, -0.05, -0.02]
SCALE = [100, 0.02, 80, 0.02, 80, 0.02, 10, 0.02, 40, 80, 0.02, 0.03, 0.02, 0.02, 0.02, 0.02, 0.02]

INF = 1e9
# (low, high) per parameter; times in s. Free rules: physical signs, and every time inside the window.
FREE = {"T_legs": (0, INF), "T_core": (0, INF), "T_sh": (0, INF), "T_wr": (0, INF), "T_rear": (0, INF), "T_lead": (0, INF),
        "on_legs": (T0, 0.0), "on_core": (T0, 0.0), "on_sh": (T0, 0.0), "on_wr": (T0, -0.005), "on_lead": (-0.34, 0.0),
        "rise_legs": (0.0, 0.25), "rise_core": (0.0, 0.25), "rise_sh": (0.0, 0.25), "rise_wr": (0.0, 0.25),
        "off_legs": (T0, 0.0), "off_core": (T0, 0.0)}
# The design's rules. Strength bounds (isometric T at the model's own force-velocity curve), each argued in docs 4.3:
#  legs  the pelvis's yaw drive is the couple of the two legs' horizontal foot forces across the stance plus the
#        hips' rotators through each foot's free moment, not one hip's torque. [4]'s lateral forces push the rear
#        foot toward the plate and the lead foot away: the rear hip's extension moment and the lead hip's flexion
#        moment, acting through legs about 0.85 m long [E]. Anderson 2007 (note 5, FT, recreational adults):
#        extension 0.161, flexion 0.113 x weight x height = 267 and 188 N m at 92 kg, 1.84 m, so foot forces of
#        314 and 221 N, a moment of (314 + 221) x 0.77 / 2 = 206 N m about the stance's middle; the rotators add
#        2 x 40-60 N m [E, no source]; athletes x1.3 [E, no source]: 380-430 N m. Bound: 0.30 x weight x height
#        = 498 N m, set above that range before the fit was run. The ground caps it again by friction (chain.py),
#        which with both feet loaded is about 300 N m: that, not the hips, is the binding limit.
#  core  pro golfers' trunk rotation 140.6 +- 30.9 N m at 0.52 rad/s (note 5, FT), scaled by weight x height to
#        92 kg, 1.84 m: 180 +- 40; mean + 2 SD = 260 N m at 0.52 rad/s, i.e. T = 260 / fv(0.52 / 25) = 288 N m.
#  sh    no horizontal ad/abduction strength is sourced; the nearest: pitchers' shoulder IR 66 N m concentric,
#        80 eccentric (note 5, TAB), golf models' shoulder 80 N m (2nd). Two shoulders, 100 N m each [E]: 200 N m.
#  wr    wrists' radial / ulnar deviation 11.0 / 9.5 N m each (Delp 1996, note 5, AB): two wrists, 20 N m.
#  rear, lead  1.5 x [4]'s peak mean trail and lead GRFx (21 % and 50 % BW) [E margin]: 0.32 and 0.75 BW.
BW0 = M0 * G
DESIGN = dict(FREE)
DESIGN.update({"T_legs": (0, 0.30 * WH), "T_core": (0, 288.0), "T_sh": (0, 200.0), "T_wr": (0, 20.0),
               "T_rear": (0, 0.32 * BW0), "T_lead": (0, 0.75 * BW0),
               "on_sh": (-0.170, 0.0), "on_wr": (-0.170, -0.005),
               "rise_legs": (0.05, 0.25), "rise_core": (0.05, 0.25), "rise_sh": (0.05, 0.25), "rise_wr": (0.05, 0.25)})
RULES = {"free": FREE, "design": DESIGN}


def body_for(rules):
    b = Body()
    if rules == "design":
        b.wrist_cap = 20.0
    return b


def make_plan(p, body):
    T_legs, on_legs, T_core, on_core, T_sh, on_sh, T_wr, on_wr, T_rear, T_lead, on_lead, r1, r2, r3, r4, off1, off2 = p
    ch = {"legs": [T_legs, on_legs, off1, WMAX["legs"], r1], "core": [T_core, on_core, off2, WMAX["core"], r2],
          "shoulder": [T_sh, on_sh, 1.0, WMAX["shoulder"], r3], "wrist": [T_wr, on_wr, 1.0, WMAX["wrist"], r4],
          "rear": [T_rear, -1.0, -0.10, WMAX["rear"]], "lead": [T_lead, on_lead, 1.0, WMAX["lead"]]}
    return Plan(ch, t_fd=-0.34, start=start_state(body))


def clamp(p, rules):
    b = RULES[rules]
    q = [min(b[k][1], max(b[k][0], v)) for k, v in zip(NAMES, p)]
    out = sum(((v - c) / s) ** 2 for v, c, s in zip(p, q, SCALE))
    # a switch-off must follow its onset's ramp
    i_on, i_r, i_off = NAMES.index("on_legs"), NAMES.index("rise_legs"), NAMES.index("off_legs")
    for a, r, o in ((i_on, i_r, i_off), (NAMES.index("on_core"), NAMES.index("rise_core"), NAMES.index("off_core"))):
        lo = q[a] + q[r] + 0.01
        if q[o] < lo:
            out += ((lo - q[o]) / 0.02) ** 2
            q[o] = min(lo, 0.0)
    return q, out


def evaluate(p, body):
    """the swing for parameters p, and its residuals in band units."""
    sim = Sim(body, make_plan(p, body), dt=0.001, record=True)
    out = sim.run(t_end=0.03)
    r = {"contact": None}
    if out["contact"] is None:
        r["miss_deg"] = (-8 * D2R - out["x_end"][4]) / D2R
        return r, sim, out
    tc, xc = out["contact"]
    e1 = e2 = e4 = es = 0.0
    npts = 0
    for t, x in zip(out["t"], out["x"]):
        if t > 0.0 or abs(round(t * 1000) % 5) > 0:
            continue
        e1 += ((x[6] / D2R - mval("pelvis_rot_vel", t)) / 80) ** 2
        e2 += ((x[7] / D2R - mval("trunk_rot_vel", t)) / 100) ** 2
        e4 += ((x[9] / D2R - mval("bat_az_vel", t)) / 250) ** 2
        es += ((x[0] - (mval("pelvis_x_cm", t) / 100 + 0.77)) / 0.03) ** 2
        npts += 1
    sep = min((x[2] - x[1]) / D2R for t, x in zip(out["t"], out["x"]) if t <= tc)
    r.update({"contact": tc, "npts": npts, "pelvis_rms": math.sqrt(e1 / npts), "trunk_rms": math.sqrt(e2 / npts),
              "bat_rms": math.sqrt(e4 / npts), "s_rms": math.sqrt(es / npts),
              "lag_c": (xc[3] - xc[4]) / D2R, "sep_peak": sep, "ss": sim.sweet_spot_speed(xc),
              "pelvis_c": xc[1] / D2R, "trunk_c": xc[2] / D2R,
              "sums": (e1, e2, e4, es)})
    return r, sim, out


def cost_of(r):
    if r["contact"] is None:
        return 1e3 + r["miss_deg"] ** 2                    # graded, so the search can climb out
    e1, e2, e4, es = r["sums"]
    c = (r["contact"] / 0.003) ** 2 * 4                   # contact time, 3 ms scale [E], weight of 4 points
    c += e1 + e2 + e4 + 0.25 * es
    c += (r["lag_c"] - 65.0) ** 2 / 10.0 ** 2 * 2         # [4] Table 1: 65 +- 10 at contact
    c += (r["sep_peak"] + 17.1) ** 2 / 4.0 ** 2 * 2       # [4]'s peak, its +-4 deg [E: Table 1's SD is at contact]
    return c / max(1, r["npts"])


def cost(p, body, rules="free"):
    q, pen = clamp(p, rules)
    r, _, _ = evaluate(q, body)
    return cost_of(r) + pen


def nelder_mead(f, x0, scale, iters, log=True):
    n = len(x0)
    pts = [x0[:]] + [[x0[j] + (scale[j] if j == i else 0) for j in range(n)] for i in range(n)]
    vals = [f(p) for p in pts]
    for it in range(iters):
        o = sorted(range(n + 1), key=lambda i: vals[i]); pts = [pts[i] for i in o]; vals = [vals[i] for i in o]
        cen = [sum(p[j] for p in pts[:-1]) / n for j in range(n)]
        xr = [cen[j] + (cen[j] - pts[-1][j]) for j in range(n)]; fr = f(xr)
        if fr < vals[0]:
            xe = [cen[j] + 2 * (cen[j] - pts[-1][j]) for j in range(n)]; fe = f(xe)
            pts[-1], vals[-1] = (xe, fe) if fe < fr else (xr, fr)
        elif fr < vals[-2]:
            pts[-1], vals[-1] = xr, fr
        else:
            xc = [cen[j] + 0.5 * (pts[-1][j] - cen[j]) for j in range(n)]; fc = f(xc)
            if fc < vals[-1]:
                pts[-1], vals[-1] = xc, fc
            else:
                for i in range(1, n + 1):
                    pts[i] = [pts[0][j] + 0.5 * (pts[i][j] - pts[0][j]) for j in range(n)]; vals[i] = f(pts[i])
        if log and it % 100 == 0:
            print("   iter %d cost %.3f" % (it, vals[0]), flush=True)
    i = min(range(n + 1), key=lambda i: vals[i])
    return pts[i], vals[i]


def report(p, body, rules):
    """one line of residuals and the parameters, with those on a bound marked."""
    r, sim, out = evaluate(p, body)
    if r["contact"] is None:
        print("   no contact: the bat ends %.1f deg short of -8 deg" % r["miss_deg"])
    else:
        print("   contact %+.1f ms; residual RMS in band units: pelvis %.2f, trunk %.2f, bat %.2f, travel %.2f; lag at contact %.0f (65 +- 10); "
              "separation peak %.1f (-17.1); pelvis / trunk at contact %.0f / %.0f (77 / 75); sweet spot %.1f m/s" % (
                  r["contact"] * 1000, r["pelvis_rms"], r["trunk_rms"], r["bat_rms"], r["s_rms"], r["lag_c"], r["sep_peak"],
                  r["pelvis_c"], r["trunk_c"], r["ss"]))
        pre = [(t, x) for t, x in zip(out["t"], out["x"]) if t <= r["contact"]]
        pk = lambda i: max(pre, key=lambda tx: tx[1][i])
        print("   peaks (deg/s at ms): pelvis %.0f at %.0f, trunk %.0f at %.0f, arm %.0f at %.0f, bat %.0f at %.0f" % tuple(
            v for i in (6, 7, 8, 9) for v in (pk(i)[1][i] / D2R, (pk(i)[0] - r["contact"]) * 1000)))
        capped = sum(1 for raw, c in zip(out["legs_raw"], out["cap"]) if abs(raw) > c)
        print("   legs' drive above the friction cap on %d of %d steps" % (capped, len(out["cap"])))
    b = RULES[rules]
    on_b = [k for k, v in zip(NAMES, p) if abs(v - b[k][0]) < 1e-6 or abs(v - b[k][1]) < 1e-6]
    print("   params: " + ", ".join("%s %.4g%s" % (k, v, "*" if k in on_b else "") for k, v in zip(NAMES, p)))
    print("   on a bound (*): " + (", ".join(on_b) if on_b else "none"))
    order = sorted(("legs", "core", "shoulder", "wrist"), key=lambda n: p[NAMES.index({"legs": "on_legs", "core": "on_core", "shoulder": "on_sh", "wrist": "on_wr"}[n])])
    print("   onset order: " + " < ".join(order))
    return r


def main():
    global M_
    M_ = measured()
    args = sys.argv[1:]
    rules = args[args.index("--rules") + 1] if "--rules" in args else "free"
    starts = int(args[args.index("--starts") + 1]) if "--starts" in args else 3
    iters = int(args[args.index("--iters") + 1]) if "--iters" in args else 300
    body = body_for(rules)
    path = os.path.join(HERE, "fit_params.json" if rules == "free" else "fit_design.json")
    x0 = X0
    if "--resume" in args and os.path.exists(path):
        old = json.load(open(path))["params"]
        x0 = [old.get(k, X0[i]) for i, k in enumerate(NAMES)]
        starts = 1
    print("rules %s, %d start(s), %d iterations x 2 rounds each, %s" % (rules, starts, iters, "resumed" if "--resume" in args else "cold start from X0"))
    results = []
    for k in range(starts):
        if k == 0:
            s0 = list(x0)
        else:
            rng = random.Random(k)
            s0 = [v * (1 + 0.3 * (2 * rng.random() - 1)) if n.startswith("T_") else v + 0.02 * (2 * rng.random() - 1)
                  for n, v in zip(NAMES, x0)]
        s0, _ = clamp(s0, rules)
        t = time.time()
        best, val = s0, cost(s0, body, rules)
        print("start %d: cost %.3f" % (k, val), flush=True)
        for rnd in range(2):
            best, val = nelder_mead(lambda p: cost(p, body, rules), best, SCALE, iters)
        best, _ = clamp(best, rules)
        val = cost(best, body, rules)
        print("start %d: final cost %.4f (%.0f s)" % (k, val, time.time() - t))
        report(best, body, rules)
        results.append((val, best))
    val, best = min(results, key=lambda vb: vb[0])
    json.dump({"rules": rules, "params": dict(zip(NAMES, best)), "cost": val, "wmax": WMAX,
               "starts": [{"cost": v, "params": dict(zip(NAMES, b))} for v, b in results]}, open(path, "w"), indent=1)
    print("best: start %d, cost %.4f -> %s" % ([v for v, _ in results].index(val), val, os.path.basename(path)))


if __name__ == "__main__":
    main()
