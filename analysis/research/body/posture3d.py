"""The launch posture set by practice with the plan (docs/body-motion.md 4.14, part 4; session 12).

    python3 analysis/research/body/posture3d.py --from <result.json> --rung r4 --seed 22 --tag _s2_posture   (job queue)
    python3 analysis/research/body/posture3d.py --margins <result.json>                                     a result's start: its numbers and margins

The numbers of the start that are still [4]'s (body.POSTURE_NUMBERS less the three the feet's measurement gives) are
given to the search WITH the plan: one search over the plan's 205 numbers and the posture's 14, carried to its end by
4.14's rule (ladder3d.settle's: one stage, first step END_SIGMA, until the best score has fallen under END_FALL over
END_WINDOW generations). Every run rebuilds its start: the posture placed, [4]'s launch velocities on it (the slopes of
the same curves, which a constant change does not alter), [4]'s loads on the feet, the hold re-solved onto his muscles.

- THE FEET DO NOT MOVE (Statcast's, rung 3) and THE BALL DOES NOT MOVE in the world (it is where rung 4 has it, level
  with the front ankle; Q64 is open): Ctx(Pc=...). A posture that moves the pelvis moves it relative to the ball.
- The start's three free choices (the bat's roll, each arm's swivel) are HELD where the body's own start has them
  (Ctx(arms=...)): the rule that searches for them flips between distant choices as the posture moves (4.12).
- In place of the compass rule that kept every joint START_CLEAR_DEG inside its end, each joint's shortfall of that
  margin AT THE START is charged in the score (MARGIN_UNIT a degree, squared as every term is). No stop is moved.

NOTHING IS FITTED TO A RECORDED SWING. The velocities and the feet's loads at launch are still [4]'s, flagged.
"""
import json
import multiprocessing
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import practice3d as PR                                # noqa: E402
import probe3d as P                                    # noqa: E402
import ladder3d as L                                   # noqa: E402
from probe3d import E, B                               # noqa: E402

GIVEN = ("stride_cm", "stride_closed_cm", "lead_foot")                 # the feet's place and the front foot's angle: Statcast's (4.13, rung 3)
NUMBERS = tuple(k for k in B.POSTURE_NUMBERS if k not in GIVEN)         # the fourteen still [4]'s
SPAN = 20.0                   # deg or cm [E]: how far either side of the body's own start a posture number may go
MARGIN_ASK = B.START_CLEAR_DEG   # deg: the margin the start's rule asks of every joint (4.12; [E: 3-8])
MARGIN_UNIT = 1.0             # deg [E]: a joint one degree short of that margin at the start costs one unit of score (1 m/s), squared


def margin_residuals(margins, ask=MARGIN_ASK, unit=MARGIN_UNIT):
    """the score's terms for the start's joints: each range's shortfall of `ask` degrees inside its end, in `unit`s
    (the score squares them). A joint `ask` or more inside its end costs nothing; one past its end is charged for the
    whole of ask plus how far past."""
    return [max(0.0, ask - v) / unit for k, v in sorted(margins.items())]


class PostureCtx:
    """the plan's numbers and the posture's, as one search: what practice3d.cma needs of a ctx (free, residuals)."""

    def __init__(self, o):
        base = PR.Ctx(**o)
        self.arms = tuple(base.m.arm_choices)
        self.Pc = tuple(base.Pc)
        self.o = dict(o, arms=list(self.arms), Pc=list(self.Pc))
        self.nplan = len(base.box)
        self.free = list(base.free) + [self.nplan + i for i in range(len(NUMBERS))]
        self.base = base

    def u_of(self, plan, posture=None):
        """the search's numbers for a plan and a posture change (deg or cm from the body's own start)."""
        posture = posture or {}
        return list(self.base.u_of(plan)) + [0.5 + posture.get(k, 0.0) / (2.0 * SPAN) for k in NUMBERS]

    def posture_of(self, u):
        return {k: (u[self.nplan + i] - 0.5) * 2.0 * SPAN for i, k in enumerate(NUMBERS)}

    def ctx_of(self, u):
        return PR.Ctx(**dict(self.o, posture=self.posture_of(u)))

    def residuals(self, u):
        try:
            ctx = self.ctx_of(u)
            mg = B.start_margins(ctx.m, ctx.st)
        except (E.SolveError, ValueError, ZeroDivisionError, ArithmeticError):
            return None                                     # a posture that cannot be placed ranks last
        r = ctx.residuals(u[:self.nplan])
        if r is None:
            return None
        return r + margin_residuals(mg)


def run(rung, path, seed, tag, log, cap=L.END_CAP, square=False):
    o = dict(L.options(rung), contact="nearest")
    if square:
        o["square"] = "contact"                           # the bat's squareness read at contact (4.14, part 3)
    d = json.load(open(path))
    pc = PostureCtx(o)
    plan = {n: [min(v[0], pc.base.cap[n])] + list(v[1:]) for n, v in d["plan"].items()}
    u0 = pc.u_of(plan, d.get("options", {}).get("posture"))
    log("== the launch posture by practice (4.14, part 4): %s carried to its end on rung %s with the posture's %d numbers given to the search; body %s; first step %.3f; until the best score has fallen under %.2f over %d generations, %d at least (at most %d); seed %d"
        % (os.path.basename(path), rung, len(NUMBERS), pc.o, L.END_SIGMA, L.END_FALL, L.END_WINDOW, L.END_LEAST, cap, seed))
    r0 = pc.residuals(u0)
    c0 = PR.cost_of(r0)
    mg0 = B.start_margins(pc.base.m, pc.base.st)
    log("   the starting plan's score with the start's margins charged: %.2f (the plan alone %.2f; the margins %.2f: %s)" % (
        c0, PR.cost_of(pc.base.residuals(pc.base.u_of(plan))), sum(x * x for x in margin_residuals(mg0)),
        ", ".join("%s %.2f" % (k, v) for k, v in sorted(mg0.items(), key=lambda kv: kv[1]) if v < MARGIN_ASK)))
    t = time.time()
    PR._CTX = pc
    with multiprocessing.get_context("fork").Pool(PR.WORKERS) as pool:
        u, cost, hist, mfull, sp = PR.cma(pc, u0, cap, log, pool, seed, sigma0=L.END_SIGMA, stop=(L.END_WINDOW, L.END_FALL, L.END_LEAST))
    gens = hist[-1][0]
    fall = hist[max(0, len(hist) - 1 - L.END_WINDOW)][1] - hist[-1][1]
    at_end = fall < L.END_FALL
    first = next((h_[0] for h_ in hist if h_[1] < hist[0][1] - 1e-9), None)
    log("   %s after %d generations (%.0f s): the best score %.2f from %.2f; it fell %.3f over its last %d; the first generation with a better plan: %s; failed runs %d"
        % ("AT ITS END" if at_end else "NOT AT ITS END (the cap)", gens, time.time() - t, cost, c0, fall, L.END_WINDOW, first, sum(h_[5] for h_ in hist)))
    ctx = pc.ctx_of(u)
    posture = pc.posture_of(u)
    mg = B.start_margins(ctx.m, ctx.st)
    log("   THE POSTURE'S NUMBERS, changed from the body's own start (deg; the pelvis's place in cm): " + "; ".join("%s %+.2f" % (k, posture[k]) for k in NUMBERS))
    log("   the start's margins under %.0f deg: %s; their charge %.2f (it was %.2f)" % (MARGIN_ASK, ", ".join("%s %.2f" % (k, v) for k, v in sorted(mg.items(), key=lambda kv: kv[1]) if v < MARGIN_ASK) or "none",
                                                                                 sum(x * x for x in margin_residuals(mg)), sum(x * x for x in margin_residuals(mg0))))
    up = u[:pc.nplan]
    out = L.show(ctx, up, log)
    out.update(cost=cost, plan_cost=PR.cost_of(ctx.residuals(up)), rung=rung, start=path, seed=seed, history=[("the posture with the plan, carried to its end", hist)],
               settled=at_end, at_end=at_end, fall_last_100=fall, generations=gens, first_better=first, start_cost=c0, start_speed=d.get("sweet_speed"),
               posture=posture, margins=mg, margins_before=mg0, step_check=PR.verify(ctx, up, log), failed_differences=PR.failed_differences(ctx, up, log), seconds=time.time() - t)
    os.makedirs(L.OUT, exist_ok=True)
    dest = os.path.join(L.OUT, "%s%s.json" % (rung, tag))
    json.dump(out, open(dest, "w"), indent=1)
    log("   kept: %s   (%.0f s)" % (os.path.relpath(dest, os.path.join(HERE, "..", "..", "..")), time.time() - t))
    return out


def main():
    P.build()
    a = sys.argv[1:]
    arg = lambda k, d: a[a.index(k) + 1] if k in a else d
    log = lambda s_: print(s_, flush=True)
    if "--dir" in a:
        L.OUT = os.path.join(P.OUT, arg("--dir", "s12"))
    if "--margins" in a:
        d = json.load(open(arg("--margins", "")))
        ctx = PR.Ctx(**d["options"])
        mg = B.start_margins(ctx.m, ctx.st)
        log("posture: %s" % d["options"].get("posture"))
        log("margins (deg inside the nearer end), the ten least: " + ", ".join("%s %.2f" % (k, v) for k, v in sorted(mg.items(), key=lambda kv: kv[1])[:10]))
        return
    if "--from" in a:
        run(arg("--rung", "r4"), arg("--from", ""), int(arg("--seed", 22)), arg("--tag", "_posture"), log, cap=int(arg("--cap", L.END_CAP)), square="--square" in a)
        return
    print(__doc__)


if __name__ == "__main__":
    main()
