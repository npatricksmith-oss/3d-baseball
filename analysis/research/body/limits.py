#!/usr/bin/env python3
"""Why the flat chain cannot swing under the design's own rules (docs/body-motion.md 4.3).

Takes the design fit (fit_design.json, fit.py --rules design) and reports:
  1. its swing run on until the bat does reach -8 deg: how late, how fast, the body at that moment;
  2. a diagnosis, one rule relaxed at a time (each followed by a short re-fit from the design fit), to
     find which rule binds. This is a diagnosis of the reduced model, not a fit the design adopts: the
     rules stay as they are (docs 4.3), and what binds names what the 3D body must supply.
Relaxations: the arms' onset free (back to the window's start); the shoulder's bound doubled (400 N m);
the wrists' cock held by the stiff stop instead of 20 N m; the 50 ms rise floor removed; the legs' and
trunk's bounds x1.5.
Usage: python3 analysis/research/body/limits.py   (about 2 min; through the job queue)
"""
import copy, json, math, sys, os, time
sys.path.insert(0, os.path.dirname(__file__))
from common import *
from chain import *
import fit

HERE = os.path.dirname(os.path.abspath(__file__))


def late_contact(p, body, t_end=0.25):
    sim = Sim(body, fit.make_plan(p, body), dt=0.0005, record=True)
    out = sim.run(t_end=t_end, stop_at_contact=False)
    if out["contact"] is None:
        return None
    tc, xc = out["contact"]
    pre = [(t, x) for t, x in zip(out["t"], out["x"]) if t <= tc]
    pk = lambda i: max(pre, key=lambda tx: tx[1][i])
    return {"tc": tc, "ss": sim.sweet_spot_speed(xc), "pelvis_c": xc[1] / D2R, "trunk_c": xc[2] / D2R, "lag_c": (xc[3] - xc[4]) / D2R,
            "pk": [(pk(i)[1][i] / D2R, (pk(i)[0] - tc) * 1000) for i in (6, 7, 8, 9)],
            "sh_max": max(d["shoulder"] for d, t in zip(out["tau"], out["t"]) if t <= tc)}


def main():
    fit.M_ = fit.measured()
    d = json.load(open(os.path.join(HERE, "fit_design.json")))
    p0 = [d["params"][k] for k in fit.NAMES]
    body = fit.body_for("design")
    print("design fit (cost %.1f): %s" % (d["cost"], ", ".join("%s %.4g" % kv for kv in zip(fit.NAMES, p0))))
    r = late_contact(p0, body)
    if r is None:
        print("1. the bat never reaches -8 deg by +250 ms")
    else:
        print("1. run on: the bat reaches -8 deg at %+.0f ms (planned 0), sweet spot %.1f m/s; pelvis / trunk then %.0f / %.0f deg, lag %.0f; "
              "peaks pelvis %.0f, trunk %.0f, arm %.0f, bat %.0f deg/s; the shoulder's largest applied torque %.0f N m" % (
                  r["tc"] * 1000, r["ss"], r["pelvis_c"], r["trunk_c"], r["lag_c"], r["pk"][0][0], r["pk"][1][0], r["pk"][2][0], r["pk"][3][0], r["sh_max"]))
    print("2. one rule relaxed at a time (diagnosis only), re-fitted for 200 iterations from the design fit:")
    base = copy.deepcopy(fit.DESIGN)
    relax = [
        ("arms' onset free", {"on_sh": (fit.T0, 0.0), "on_wr": (fit.T0, -0.005)}, None),
        ("shoulder bound x2 (400 N m)", {"T_sh": (0, 400.0)}, None),
        ("cock held by the stiff stop", {}, "stop"),
        ("no 50 ms rise floor", {"rise_legs": (0.0, 0.25), "rise_core": (0.0, 0.25), "rise_sh": (0.0, 0.25), "rise_wr": (0.0, 0.25)}, None),
        ("legs and trunk bounds x1.5", {"T_legs": (0, 1.5 * base["T_legs"][1]), "T_core": (0, 1.5 * base["T_core"][1])}, None),
    ]
    for label, ch, wr in relax:
        rules = dict(base); rules.update(ch)
        fit.RULES["diag"] = rules
        b = fit.body_for("design")
        if wr == "stop":
            b.wrist_cap = None
        t = time.time()
        best, val = fit.nelder_mead(lambda p: fit.cost(p, b, "diag"), p0, fit.SCALE, 200, log=False)
        best, _ = fit.clamp(best, "diag")
        val = fit.cost(best, b, "diag")
        rr, _, _ = fit.evaluate(best, b)
        if rr["contact"] is None:
            res = "no contact by +30 ms (%.1f deg short)" % rr["miss_deg"]
        else:
            res = "contact %+.1f ms, pelvis %.2f trunk %.2f bat %.2f band units, lag %.0f, sweet spot %.1f m/s" % (
                rr["contact"] * 1000, rr["pelvis_rms"], rr["trunk_rms"], rr["bat_rms"], rr["lag_c"], rr["ss"])
        on_b = [k for k, v in zip(fit.NAMES, best) if abs(v - rules[k][0]) < 1e-6 or abs(v - rules[k][1]) < 1e-6]
        print("   %-30s cost %8.2f  %s; on a bound: %s (%.0f s)" % (label, val, res, ", ".join(on_b), time.time() - t))
        print("      " + ", ".join("%s %.4g" % kv for kv in zip(fit.NAMES, best)))


if __name__ == "__main__":
    main()
