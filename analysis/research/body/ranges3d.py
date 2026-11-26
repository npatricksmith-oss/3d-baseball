"""Where the body's joints are against their measured ranges (docs/body-motion.md 4.10; PLAN D37).

    python3 analysis/research/body/ranges3d.py [plan.json ...]

For the placed start and for each saved plan (a fit3d.py or practice3d.py json), run once on the body it was found
on: every joint's angle (body3d.body.range_angles) at the start and its least and greatest up to contact + 50 ms,
beside the range of body.ROM. No search; a diagnosis. Each plan is run on the body of sessions 1-7 (the old stops).
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import fit3d as F                                      # noqa: E402
from probe3d import B, E                               # noqa: E402
import probe3d as P                                    # noqa: E402

EXTRA = {"pron": ("sup", "pron"), "sh_rot": (None, None), "sh_elev": (None, None)}


def limits():
    out = {name: (lo, hi) for name, a, b, ax, ea, fb, lo, hi, size, kind in B.range_specs()}
    for sd in "LR":
        out["pron_" + sd] = (-B.ROM["sup"], B.ROM["pron"])
    return out


def excursions(ctx, u, t_to=0.050):
    st, aux = ctx.prepare(u)
    n = int(round((F.T_END - ctx.t0) / F.H_STEP))
    recs = P.run_raw(ctx.m.sys, st, ctx.t0, n, F.H_STEP, 5, aux, P.tree_order(ctx.m))
    rows = [(t, B.range_angles(ctx.m, state)) for t, state, *_ in recs if t <= t_to + 1e-9]
    return rows


def show(label, rows):
    lim = limits()
    print("\n%s: angle at the start; least and greatest to +50 ms; the range; how far outside (deg)" % label)
    for k in rows[0][1]:
        v = [r[1][k] for r in rows]
        lo, hi = lim.get(k, (None, None))
        out = "" if lo is None else ("%5.0f" % max(lo - min(v), max(v) - hi, 0.0))
        print("   %-14s %7.1f   %7.1f %7.1f   %s   %s" % (k, v[0], min(v), max(v), "   (no range held)" if lo is None else "%6.1f %6.1f" % (lo, hi),
                                                          out + ("  OUTSIDE" if out.strip() not in ("", "0") else "")))


def main():
    P.build()
    ctx = F.Ctx()
    files = sys.argv[1:] or [os.path.join(HERE, "fit3d_d34_all.json"), os.path.join(P.OUT, "fit3d_all_strongest_on_time.json")]
    for f in files:
        plan = json.load(open(f))["plan"]
        show(os.path.basename(f), excursions(ctx, ctx.u_of(plan)))


if __name__ == "__main__":
    main()
