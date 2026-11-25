#!/usr/bin/env python3
"""The fitted average swing against [4] and [3] (docs/body-motion.md 4.3), with the integrator checks.

Usage: python3 analysis/research/body/swing.py   (after fit.py has written fit_params.json)
"""
import json, math, sys, os, time
sys.path.insert(0, os.path.dirname(__file__))
from common import *
from chain import *
import fit

HERE = os.path.dirname(os.path.abspath(__file__))


def fitted():
    return [json.load(open(os.path.join(HERE, "fit_params.json")))["params"][k] for k in fit.NAMES]


def summary(sim, out):
    """peaks and times of the rates, separation, lag, sweet-spot speed at contact."""
    ts, xs = out["t"], out["x"]
    tc, xc = out["contact"] if out["contact"] else (None, None)
    r = {"tc": tc}
    pre = [(t, x) for t, x in zip(ts, xs) if tc is None or t <= tc]

    def peak(i):
        t, x = max(pre, key=lambda p: p[1][i]); return x[i] / D2R, (t - (tc or 0)) * 1000
    r["pelvis"] = peak(6); r["trunk"] = peak(7); r["arm"] = peak(8); r["bat"] = peak(9)
    sep = min(pre, key=lambda p: p[1][2] - p[1][1])
    r["sep"] = ((sep[1][1] - sep[1][2]) / D2R, (sep[0] - (tc or 0)) * 1000)
    if xc:
        r["ss"] = sim.sweet_spot_speed(xc)
        r["lag_c"] = (xc[3] - xc[4]) / D2R
        r["pelvis_c"] = xc[1] / D2R; r["trunk_c"] = xc[2] / D2R
        r["s_c"] = xc[0]
        hx, hy = sim.hands(xc)
        r["hands_r"] = math.hypot(hx, hy - xc[0])
    r["lagmax"] = max((x[3] - x[4]) / D2R for _, x in pre)
    return r


def main():
    fit.M_ = fit.measured()
    body = Body()
    p = fitted()
    plan = fit.make_plan(p, body)
    sim = Sim(body, plan, dt=0.0005, record=True)
    t = time.time(); out = sim.run(t_end=0.02); el = time.time() - t
    r = summary(sim, out)
    print("fitted drive (fit.py, %d params): " % len(p) + ", ".join("%s %.3g" % kv for kv in zip(fit.NAMES, p)))
    print("contact at %.1f ms ([4]: 0)" % (r["tc"] * 1000))
    print("%-22s %-22s %-30s" % ("quantity", "model", "measured"))
    rows = [
        ("pelvis peak", "%.0f deg/s at %.0f ms" % r["pelvis"], "581 at -70 [4]; 714 (SD 76) at -75 [3]"),
        ("trunk peak", "%.0f deg/s at %.0f ms" % r["trunk"], "766 at -57 [4]; 937 (SD 102) at -65 [3]"),
        ("arm peak", "%.0f deg/s at %.0f ms" % r["arm"], "1160 (SD 96) at -65 [3]"),
        ("bat azimuth peak", "%.0f deg/s at %.0f ms" % r["bat"], "2435 at -10 [4]"),
        ("separation peak", "%.1f deg at %.0f ms" % r["sep"], "18 at -103 [4] (17.1 on the graph)"),
        ("bat lag max / contact", "%.0f / %.0f deg" % (r["lagmax"], r["lag_c"]), "133 / 65 [4]"),
        ("pelvis / trunk at contact", "%.0f / %.0f deg" % (r["pelvis_c"], r["trunk_c"]), "77 / 75 [4]"),
        ("sweet-spot speed at contact", "%.1f m/s" % r["ss"], "29 m/s (barrel end) [3]; [4] none"),
        ("forward travel at contact", "%.3f m from the rear foot" % r["s_c"], "%.3f m ([4] pelvis, feet [E])" % (fit.mval("pelvis_x_cm", 0.0) / 100 + 0.77)),
    ]
    for a, b, c in rows:
        print("%-28s %-26s %s" % (a, b, c))
    print("rates against [4]'s curves (model / measured, deg/s):")
    print("  t ms   pelvis        trunk         bat")
    for tt, x in zip(out["t"], out["x"]):
        ms = round(tt * 1000)
        if ms % 20 == 10 and abs(tt * 1000 - ms) < 1e-6 and ms <= 10:
            print("  %4d  %5.0f / %4.0f  %5.0f / %4.0f  %5.0f / %4.0f" % (ms, x[6] / D2R, fit.mval("pelvis_rot_vel", tt), x[7] / D2R, fit.mval("trunk_rot_vel", tt), x[9] / D2R, fit.mval("bat_az_vel", tt)))
    # applied torques and the friction cap
    tl = max(out["tau"], key=lambda d: d["legs"])
    print("largest applied yaw torques (N m): legs %.0f, core %.0f, shoulder %.0f, wrist %.0f; brake %.0f N, push %.0f N" % (
        max(d["legs"] for d in out["tau"]), max(d["core"] for d in out["tau"]), max(d["shoulder"] for d in out["tau"]),
        max(d["wrist"] for d in out["tau"]), max(d["lead"] for d in out["tau"]), max(d["rear"] for d in out["tau"])))
    # energy check: KE change = work of all forces (drives, the wrist stop, ground)
    ke0, ke1 = out["ke"][0], out["ke"][-1]
    print("energy: KE %.2f -> %.2f J, change %.3f J; drives' work %.3f J; mismatch %.2e J (%.1e of the change)" % (
        ke0, ke1, ke1 - ke0, out["work"], (ke1 - ke0) - out["work"], abs((ke1 - ke0) - out["work"]) / abs(ke1 - ke0)))
    print("time per swing in this probe: %.2f s for %.0f ms at dt 0.5 ms (%d steps)" % (el, (out["t_end"] - out["t"][0]) * 1000, len(out["t"])))
    return sim, out


if __name__ == "__main__":
    main()
