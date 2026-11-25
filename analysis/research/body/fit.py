#!/usr/bin/env python3
"""Fit the average batter's drive to [4]'s measured swing (docs/body-motion.md 4.3).

The forward model of chain.py, started from [4]'s mean state at -250 ms, with the drive's numbers
(per channel a peak and an onset; the push and the brake along the stride) found by Nelder-Mead so
that the pelvis, upper-trunk and bat-azimuth rates follow [4]'s Appendix E curves from -250 ms to
contact, contact (the bat at [4]'s -8 deg azimuth) comes at 0 ms, and the body's forward travel follows
[4]'s pelvis. Each rate's misfit is scaled by [4]'s own between-batter band (about one SD read off the
graphs: pelvis 80, trunk 100, bat 250 deg/s), so a cost of 1 per point means 'one SD off'.

Usage: python3 analysis/research/body/fit.py [iterations]   -> writes fit_params.json beside it.
"""
import json, math, sys, os, time
sys.path.insert(0, os.path.dirname(__file__))
from common import *
from chain import *

T0 = -0.25
HERE = os.path.dirname(os.path.abspath(__file__))


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
    s0 = mval("pelvis_x_cm", T0) / 100 + 0.77 * body.H / 1.84     # [E] rear foot 0.77 m behind [4]'s plate zero... see docs 4.1
    sd = (mval("pelvis_x_cm", T0 + 0.005) - mval("pelvis_x_cm", T0 - 0.005)) / 100 / 0.01
    A = mval("bat_az", T0) * d; lag = mval("bat_lag", T0) * d
    Ad = mval("bat_az_vel", T0) * d
    return [T0, s0, mval("pelvis_rot", T0) * d, mval("trunk_rot", T0) * d, A + lag, A,
            sd, mval("pelvis_rot_vel", T0) * d, mval("trunk_rot_vel", T0) * d, Ad, Ad]


# fixed [E] maximum shortening speeds per channel (rad/s, m/s): docs 4.2 lists them as unsourced
WMAX = {"legs": 20.0, "core": 25.0, "shoulder": 30.0, "wrist": 40.0, "rear": 3.0, "lead": 3.0}
NAMES = ["T_legs", "on_legs", "T_core", "on_core", "T_sh", "on_sh", "T_wr", "on_wr", "T_rear", "T_lead", "on_lead",
         "rise_legs", "rise_core", "rise_sh", "rise_wr"]
X0 = [769, -0.25, 219, -0.185, 275, -0.196, 336, -0.05, 9, 253, -0.10, 0.08, 0.03, 0.03, 0.02]
SCALE = [100, 0.02, 80, 0.02, 80, 0.02, 60, 0.02, 40, 80, 0.02, 0.03, 0.02, 0.02, 0.02]


def make_plan(p, body):
    T_legs, on_legs, T_core, on_core, T_sh, on_sh, T_wr, on_wr, T_rear, T_lead, on_lead, r1, r2, r3, r4 = p
    ch = {"legs": [T_legs, on_legs, 1.0, WMAX["legs"], r1], "core": [T_core, on_core, 1.0, WMAX["core"], r2],
          "shoulder": [T_sh, on_sh, 1.0, WMAX["shoulder"], r3], "wrist": [T_wr, on_wr, 1.0, WMAX["wrist"], r4],
          "rear": [T_rear, -1.0, -0.10, WMAX["rear"]], "lead": [T_lead, on_lead, 1.0, WMAX["lead"]]}
    return Plan(ch, t_fd=-0.34, start=start_state(body))


def cost(p, body, verbose=False):
    if any(v < 0 for v in (p[0], p[2], p[4], p[6], p[8], p[9])) or any(v < 0.0 or v > 0.25 for v in p[11:15]):
        return 1e9
    sim = Sim(body, make_plan(p, body), dt=0.001, record=True)
    out = sim.run(t_end=0.03)
    if out["contact"] is None:
        return 1e6
    tc = out["contact"][0]
    c = (tc / 0.003) ** 2 * 4            # contact time, 3 ms scale, weight of 4 points
    npts = 0
    for t, x in zip(out["t"], out["x"]):
        if t > 0.0 or abs(round(t * 1000) % 5) > 0:
            continue
        e1 = (x[6] / D2R - mval("pelvis_rot_vel", t)) / 80
        e2 = (x[7] / D2R - mval("trunk_rot_vel", t)) / 100
        e4 = (x[9] / D2R - mval("bat_az_vel", t)) / 250
        es = (x[0] - (mval("pelvis_x_cm", t) / 100 + 0.77)) / 0.03
        c += e1 * e1 + e2 * e2 + e4 * e4 + 0.25 * es * es
        npts += 1
    xc = out["contact"][1]
    c += ((xc[3] - xc[4]) / D2R - 65.0) ** 2 / 10.0 ** 2 * 2
    sep = min((x[2] - x[1]) / D2R for t, x in zip(out["t"], out["x"]) if t <= tc)
    c += (sep + 17.1) ** 2 / 4.0 ** 2 * 2
    return c / max(1, npts)


def nelder_mead(f, x0, scale, iters):
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
        if it % 50 == 0:
            print("iter %d cost %.3f" % (it, vals[0]), flush=True)
    i = min(range(n + 1), key=lambda i: vals[i])
    return pts[i], vals[i]


def main():
    global M_
    M_ = measured()
    body = Body()
    iters = int(sys.argv[1]) if len(sys.argv) > 1 else 400
    path = os.path.join(HERE, "fit_params.json")
    x0 = X0
    if os.path.exists(path):
        old = json.load(open(path))["params"]
        x0 = [old.get(k, X0[i]) for i, k in enumerate(NAMES)]
    t = time.time(); c0 = cost(x0, body); print("start cost %.3f (%.2f s per eval)" % (c0, time.time() - t))
    best, val = x0, c0
    for rnd in range(3):
        best, val = nelder_mead(lambda p: cost(p, body), best, SCALE, iters)
        print("round %d cost %.3f" % (rnd, val))
    json.dump({"params": dict(zip(NAMES, best)), "cost": val, "wmax": WMAX}, open(path, "w"), indent=1)
    print(json.dumps(dict(zip(NAMES, [round(v, 4) for v in best]))))


if __name__ == "__main__":
    main()
