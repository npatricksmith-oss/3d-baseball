"""Which joints' ranges change the earlier swings (docs/body-motion.md 4.10; session 8's review, P2-5).

    python3 analysis/research/body/which_stops3d.py      (20 s; output sim/target/codex-probes/body3d/which_stops3d.out)

The two saved plans of body-motion 4.9 (the fit to [4]; the swing asked only for speed), each run ONCE on the practice
body (practice3d.Ctx) with one group of its joints' stops on at a time. One body, one measure: the sweet spot's speed
when the bat's azimuth passes -8 deg, the instant those plans were timed to (their own contact, [4]'s; it is used
here only to say "when"), with the work the stops did up to then beside it, at a 0.125 ms step.

HOW LITTLE THIS SHOWS: each plan was tuned on a body with other stops (soft ones, under a law that created energy),
so a single replay with a stop added says what that plan does, not what the body can do. A plan re-tuned with the
stop on may do far better or worse; that is what a search is for.
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import practice3d as Q                                 # noqa: E402
P, B, E, C = Q.P, Q.B, Q.E, Q.C

GROUPS = (("none", (), "cocked"), ("the wrists' (travel toward uncocking)", ("wrist",), "cocked"),
          ("the wrists' (travel the other way, the review's)", ("wrist",), "mirrored"),
          ("the wrists' (about the forearm square to the handle)", ("wrist",), "square"),
          ("the hips'", ("hip",), "cocked"), ("the trunk's", ("trunk",), "cocked"), ("the shoulders'", ("shoulder",), "cocked"),
          ("the knees' and elbows'", ("knee", "elbow"), "cocked"),
          ("all", ("wrist", "hip", "trunk", "shoulder", "knee", "elbow"), "cocked"),
          ("all, the wrists' travel the other way", ("wrist", "hip", "trunk", "shoulder", "knee", "elbow"), "mirrored"))
H = 1.25e-4


def one(plan, keep, wrists):
    ctx = Q.Ctx(wrists=wrists)
    s = ctx.m.sys
    s.stops = [x for x in s.stops if keep and x[0].startswith(tuple(keep))]
    try:
        recs = ctx.run(ctx.u_of(plan), every=8, h=H, t_end=0.100)
    except E.SolveError as e:
        return "the run fails (%s)" % str(e)[:60]
    ow = 13 * s.nb + len(s.acts)
    prev = None
    for t, state, *_ in recs:
        az = ctx.m.observe(state)["bat_az"]
        az = az - 360.0 if az > 90.0 else az
        if prev is not None and prev[0] < -8.0 <= az:
            f = (-8.0 - prev[0]) / (az - prev[0])
            v0, v1 = E.norm(ctx.m.bat_point(prev[2], C.BAT_SS)[1]), E.norm(ctx.m.bat_point(state, C.BAT_SS)[1])
            wd = [prev[2][ow + q] + f * (state[ow + q] - prev[2][ow + q]) - recs[0][1][ow + q] for q in range(2)]
            return "%5.1f m/s at %+4.0f ms; the muscles' work %4.0f J, the stops' %+5.0f J" % (v0 + f * (v1 - v0), (prev[1] + f * (t - prev[1])) * 1e3, wd[0], wd[1])
        prev = (az, t, state)
    return "the bat never comes round to -8 deg by +100 ms"


def main():
    P.build()
    out = open(os.path.join(P.OUT, "which_stops3d.out"), "w")

    def log(x):
        print(x)
        out.write(x + "\n")
    log("each saved plan run once on the practice body with one group of stops; the sweet spot's speed when the bat's azimuth passes -8 deg (0.125 ms step)")
    for label, f in (("the fit to [4] (fit3d_d34_all.json)", os.path.join(HERE, "fit3d_d34_all.json")),
                     ("the swing asked only for speed (fit3d_all_strongest_on_time.json)", os.path.join(P.OUT, "fit3d_all_strongest_on_time.json"))):
        plan = json.load(open(f))["plan"]
        log(label)
        for g, keep, wrists in GROUPS:
            log("   %-52s %s" % (g, one(plan, keep, wrists)))


if __name__ == "__main__":
    main()
