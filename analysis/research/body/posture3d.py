"""The launch posture set by practice with the plan (docs/body-motion.md 4.14, part 4; session 12).

    python3 analysis/research/body/posture3d.py --from <result.json> --rung r4 --seed 22 --tag _s2_posture   (job queue)
    ... --resume <its result.json> --cap 6000        the same search gone on from its kept state (every block of 500 generations is kept with it)
    ... --meet --resume <a result without it>        the same search gone on with one more number: where the ball is met along its line (4.15, PLAN D44)
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


MEET_RANGE = (-0.40, 0.60)    # m [E]: how far toward the catcher and toward the pitcher of the body's own place the search may put the ball (4.15, part 1)


class PostureCtx:
    """the plan's numbers and the posture's, as one search: what practice3d.cma needs of a ctx (free, residuals).
    meet (4.15, part 1; PLAN D44): True gives the search one more number, WHERE THE BALL IS MET ALONG ITS LINE (metres
    toward the pitcher from the body's own place, inside MEET_RANGE). The ball's place in the world (Pc) is then the
    body's own place and the number moves it from there (practice3d.Ctx(meet=...), which also shortens the arms' time)."""

    def __init__(self, o, meet=False):
        o = {k: v for k, v in o.items() if k != "meet"}
        base = PR.Ctx(**o)                                  # the body's own start and the ball's own place (meet 0)
        self.arms = tuple(base.m.arm_choices)
        self.Pc = tuple(base.Pc)
        self.o = dict(o, arms=list(self.arms), Pc=list(self.Pc))
        self.nplan = len(base.box)
        self.free = list(base.free) + [self.nplan + i for i in range(len(NUMBERS))]
        self.meet = bool(meet)
        if self.meet:
            self.free.append(self.nplan + len(NUMBERS))
        self.base = base

    def u_of(self, plan, posture=None, meet=0.0):
        """the search's numbers for a plan, a posture change (deg or cm from the body's own start) and, with meet, the ball's place."""
        posture = posture or {}
        u = list(self.base.u_of(plan)) + [0.5 + posture.get(k, 0.0) / (2.0 * SPAN) for k in NUMBERS]
        return u + [(meet - MEET_RANGE[0]) / (MEET_RANGE[1] - MEET_RANGE[0])] if self.meet else u

    def posture_of(self, u):
        return {k: (u[self.nplan + i] - 0.5) * 2.0 * SPAN for i, k in enumerate(NUMBERS)}

    def meet_of(self, u):
        return MEET_RANGE[0] + u[self.nplan + len(NUMBERS)] * (MEET_RANGE[1] - MEET_RANGE[0]) if self.meet else 0.0

    def ctx_of(self, u):
        o = dict(self.o, posture=self.posture_of(u))
        if self.meet:
            o["meet"] = self.meet_of(u)
        return PR.Ctx(**o)

    def extra_of(self, u):
        """what a result keeps of the ball's place (4.15): the number, its range and whether an end of the range binds."""
        if not self.meet:
            return {}
        v = self.meet_of(u)
        return {"meet_m": v, "meet_range_m": list(MEET_RANGE), "meet_at_an_end": min(v - MEET_RANGE[0], MEET_RANGE[1] - v) < 0.02}

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


def rescored(pc, state, log):
    """a kept state brought to the score as it is NOW (4.15, part 0): a search resumed after the score's reading has
    changed holds a best plan scored by the old reading. Its best plan is run again; if its score has changed, the kept
    best and the history's best-ever column are moved by the difference, so that the end rule (how far the best has
    fallen over its window) reads what it read before and the search has no phantom score to beat. A rule for the
    search; nothing of the body."""
    now = PR.cost_of(pc.residuals(state["best_u"]))
    was = state["best"]
    if abs(now - was) > 1e-9:
        log("   the kept best plan scores %.4f by the score as it is now (it was kept at %.4f): the kept best and the history's best-ever column moved by %+.4f" % (now, was, now - was))
        state = dict(state, best=now, hist=[[h_[0], h_[1] + (now - was)] + list(h_[2:]) for h_ in state["hist"]])
    return state


def widened(pc, state, start_u, log, step=L.END_SIGMA):
    """a kept state of the search WITHOUT the ball's place, given that one more number (4.15, part 1), so that the search
    with it is the same search gone on and not one started afresh (4.14: a fresh search has lost every number's learned
    spread). The new number starts at the body's own place (meet 0) with a first step of `step` of its range, whatever
    the search's overall step has become; its paths start at nothing. Returns (state, start_u)."""
    n_old = len(state["m"])
    if n_old == len(pc.free):
        return state, start_u
    assert pc.meet and n_old == len(pc.free) - 1, "the kept state does not fit this search's numbers"
    u_meet = (0.0 - MEET_RANGE[0]) / (MEET_RANGE[1] - MEET_RANGE[0])
    st = dict(state)
    st["m"] = list(state["m"]) + [u_meet]
    st["C"] = list(state["C"]) + [(step / state["sigma"]) ** 2]
    st["ps"], st["pc"] = list(state["ps"]) + [0.0], list(state["pc"]) + [0.0]
    st["best_u"] = list(state["best_u"]) + [u_meet]
    st["m_marks"] = {k_: list(v_) + [u_meet] for k_, v_ in state["m_marks"].items()}
    log("   THE BALL'S PLACE GIVEN TO THE SEARCH at generation %d: one more number beside the kept state's %d, starting at the body's own place with a first step of %.1f cm (the overall step is %.4f)"
        % (state["g"], n_old, step * (MEET_RANGE[1] - MEET_RANGE[0]) * 100, state["sigma"]))
    return st, list(start_u) + [u_meet]


def run(rung, path, seed, tag, log, cap=L.END_CAP, square=False, leap=False, resume=None, block=500, extra=None, meet=False):
    """the posture given to the search with the plan, as ONE search with its state kept (4.14, part 4; 4.15, part 0): after
    every `block` generations the result is read whole (so the bat's speed is known at each) and kept as
    <rung><tag>_g<generations>.json WITH THE SEARCH'S STATE, and the search goes on from that state exactly as if it had
    not stopped (ladder3d.carry's rule), to its end by 4.14's rule or to `cap`. resume: a kept result to go on from.
    extra: further options of the body or the score (4.15)."""
    o = dict(L.options(rung), contact="nearest")
    if square:
        o["square"] = "contact"                           # the bat's squareness read at contact (4.14, part 3)
    o.update(extra or {})
    d = json.load(open(path))
    pc = PostureCtx(o, meet=meet)
    plan = {n: [min(v[0], pc.base.cap[n])] + list(v[1:]) for n, v in d["plan"].items()}
    u0 = pc.u_of(plan, d.get("options", {}).get("posture"), d.get("options", {}).get("meet", 0.0))
    state = None
    since = 0                                             # the generation at which the search last got a new number: its end rule counts from there
    if resume:
        # the same search gone on from its kept state (practice3d.cma): the result's file holds it and the numbers it started from
        dres = json.load(open(resume))
        state, u0 = dres["state"], list(dres["start_u"])
        since = dres.get("since", 0)
        if len(state["m"]) != len(pc.free):
            state, u0 = widened(pc, state, u0, log)
            since = state["g"]
    least = since + L.END_LEAST
    log("== the launch posture by practice (4.14, part 4): %s carried to its end on rung %s with the posture's %d numbers given to the search; body %s; first step %.3f; blocks of %d generations, each read and kept with the search's state; until the best score has fallen under %.2f over %d generations, not before generation %d (at most %d); seed %d%s"
        % (os.path.basename(path), rung, len(NUMBERS), pc.o, L.END_SIGMA, block, L.END_FALL, L.END_WINDOW, least, cap, seed,
           " (RESUMED from %s at generation %d)" % (resume, state["g"]) if resume else ""))
    r0 = pc.residuals(u0)
    c0 = PR.cost_of(r0)
    mg0 = B.start_margins(pc.base.m, pc.base.st)
    log("   the starting plan's score with the start's margins charged: %.2f (the plan alone %.2f; the margins %.2f: %s)" % (
        c0, PR.cost_of(pc.base.residuals(pc.base.u_of(plan))), sum(x * x for x in margin_residuals(mg0)),
        ", ".join("%s %.2f" % (k, v) for k, v in sorted(mg0.items(), key=lambda kv: kv[1]) if v < MARGIN_ASK)))
    if state is not None:
        state = rescored(pc, state, log)
    t = time.time()
    PR._CTX = pc
    g = state["g"] if state else 0
    out = None
    os.makedirs(L.OUT, exist_ok=True)
    while g < cap:
        target = min(cap, (g // block + 1) * block)
        keep = {}
        with multiprocessing.get_context("fork").Pool(PR.WORKERS) as pool:
            u, cost, hist, mfull, sp = PR.cma(pc, u0, target, log, pool, seed, sigma0=L.END_SIGMA, stop=(L.END_WINDOW, L.END_FALL, least), leap=L.LEAP if leap else None, state=state, keep=keep)
        state, g = keep, hist[-1][0]
        at_end = g < target
        fall = hist[max(0, len(hist) - 1 - L.END_WINDOW)][1] - hist[-1][1]
        first = next((h_[0] for h_ in hist if h_[1] < hist[0][1] - 1e-9), None)
        log("-- %s after %d generations (%.0f s): the best score %.2f from %.2f; it fell %.3f over its last %d; the first generation with a better plan: %s; failed runs %d"
            % ("AT ITS END (ended by the rule)" if at_end else "not at its end", g, time.time() - t, cost, c0, fall, L.END_WINDOW, first, sum(h_[5] for h_ in hist)))
        ctx = pc.ctx_of(u)
        posture = pc.posture_of(u)
        mg = B.start_margins(ctx.m, ctx.st)
        log("   THE POSTURE'S NUMBERS, changed from the body's own start (deg; the pelvis's place in cm): " + "; ".join("%s %+.2f" % (k, posture[k]) for k in NUMBERS))
        log("   the start's margins under %.0f deg: %s; their charge %.2f (it was %.2f)" % (MARGIN_ASK, ", ".join("%s %.2f" % (k, v) for k, v in sorted(mg.items(), key=lambda kv: kv[1]) if v < MARGIN_ASK) or "none",
                                                                                     sum(x * x for x in margin_residuals(mg)), sum(x * x for x in margin_residuals(mg0))))
        up = u[:pc.nplan]
        out = L.show(ctx, up, log)
        out.update(cost=cost, plan_cost=PR.cost_of(ctx.residuals(up)), rung=rung, start=path, seed=seed, history=[("the posture with the plan, carried to its end", hist)],
                   settled=at_end, at_end=at_end, fall_last_100=fall, generations=g, first_better=first, start_cost=c0, start_speed=d.get("sweet_speed"),
                   posture=posture, margins=mg, margins_before=mg0, state=state, start_u=list(u0), since=since, step_check=PR.verify(ctx, up, log), seconds=time.time() - t)
        out.update(pc.extra_of(u))
        if pc.meet:
            log("   WHERE THE BALL IS MET (the search's): %+.1f cm %s of the body's own place along the ball's line (the range %+.0f to %+.0f cm%s); the arms may fire %.1f ms before contact"
                % (out["meet_m"] * 100, "toward the pitcher" if out["meet_m"] >= 0 else "toward the catcher", MEET_RANGE[0] * 100, MEET_RANGE[1] * 100, "; AT AN END OF ITS RANGE" if out["meet_at_an_end"] else "", ctx.t_avail * 1e3))
        dest = os.path.join(L.OUT, "%s%s_g%d.json" % (rung, tag, g))
        json.dump(out, open(dest, "w"), indent=1)
        log("   kept: %s   (%.0f s)" % (os.path.relpath(dest, os.path.join(HERE, "..", "..", "..")), time.time() - t))
        if at_end:
            break
    if out is not None:
        out["failed_differences"] = PR.failed_differences(ctx, up, log)
        json.dump(out, open(os.path.join(L.OUT, "%s%s.json" % (rung, tag)), "w"), indent=1)
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
        run(arg("--rung", "r4"), arg("--from", ""), int(arg("--seed", 22)), arg("--tag", "_posture"), log, cap=int(arg("--cap", L.END_CAP)), square="--square" in a, leap="--leap" in a, resume=arg("--resume", None),
            block=int(arg("--block-gens", 500)), meet="--meet" in a)
        return
    print(__doc__)


if __name__ == "__main__":
    main()
