"""The ladder of single changes to the body, each with a settled search and a measured bat speed
(docs/body-motion.md 4.13, session 11; PLAN D42: "the fixes one at a time, with a measured bat speed after each").

    python3 analysis/research/body/ladder3d.py --rung r1                      the rung's search by the ladder's protocol, from start S
    python3 analysis/research/body/ladder3d.py --rung r1_low --from <json>    a body searched on from a saved result (--reach 0 --gens 0)
    python3 analysis/research/body/ladder3d.py --table                        the ladder's table from the results kept so far
    python3 analysis/research/body/ladder3d.py --show <json>                  a kept result re-read (no search)

THE PROTOCOL, the same for every rung (written in 4.13 before any run): session 10's own search (practice3d.cma, every
run at 0.25 ms, watching 0.10 s) from the same plan S (session 9's nearest plan, which came from the fit to [4]: flagged):
100 generations of the reach stage, 250 of the whole aim, then 500 more from that result; then blocks of 250 more until
the best score has fallen less than SETTLED over its last 50 generations (at most MAX_BLOCKS). Every rung gets the same
search from the same plan, so that a gain cannot be more searching. NOTHING IS FITTED TO A RECORDED SWING; the start is
still [4]'s launch posture moved by 4.12's rule (an input, flagged) until a rung says otherwise.

Each result: practice3d's report (every check of 4.10, never given to the search), the search's own checks (the result
at three steps, the line from the starting plan, slopes), the work of every element to contact by joint, and the gate's
rows with their margins. Kept under sim/target/codex-probes/body3d/s11/.
"""
import json
import math
import multiprocessing
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import practice3d as PR                                # noqa: E402
import probe3d as P                                    # noqa: E402
from probe3d import E, B, Dr                           # noqa: E402

OUT = os.path.join(P.OUT, "s11")
BASE = dict(grip="hands", feet="cleats", hold="let_go", wrist="oval", clear=True, h=2.5e-4, t_view=0.10)   # session 10's body
SETTLED, MAX_BLOCKS, BLOCK = 1.0, 2, 250               # the score's fall over the last 50 generations that counts as settled [E]
MPH = 0.44704
# ---- the rungs: each is the options of the rung before with one thing changed (4.13's table) ----
RUNGS = {
    "r0": {},                                          # session 10's body: its settled swing, run again
    "r2a": {"stand": "muscle"},                        # rung 2 alone, on rung 0's body: the posture held by muscle (4.13)
    "r1": {"damp": "read"},                            # rung 1: the joints' damping at the values read (drive.DAMPING, note 14)
    "r1_low": {"damp": "low"}, "r1_high": {"damp": "high"},   # the unread joints at the ends of their ranges, all together
    "r1_arms_low": {"damp": {"set": "read", "from": {"arms": "low"}}}, "r1_arms_high": {"damp": {"set": "read", "from": {"arms": "high"}}},
    "r1_trunk_low": {"damp": {"set": "read", "from": {"trunk": "low"}}}, "r1_trunk_high": {"damp": {"set": "read", "from": {"trunk": "high"}}},
    "r1_hips_low": {"damp": {"set": "read", "from": {"hip_L": "low", "hip_R": "low"}}}, "r1_hips_high": {"damp": {"set": "read", "from": {"hip_L": "high", "hip_R": "high"}}},
    "r2": {"damp": "read", "stand": "muscle"},         # rung 2: rung 1 with the posture held by muscle
    "r3": {"damp": "read", "stand": "muscle", "stance": "mlb"},      # rung 3: rung 2 on major leaguers' feet (body.START_STANCE, note 15)
    "r4": {"damp": "read", "stand": "muscle", "stance": "mlb", "rom": dict(B.ROM_ACTIVE_HIP)},   # rung 4: the hips' turn by their own muscles
    # rung 5: the launch state by a rule and not [4]'s: standing still (no starting velocity), the feet's loads by balance, their push by least hold
    "r5": {"damp": "read", "stand": "muscle", "stance": "mlb", "rom": dict(B.ROM_ACTIVE_HIP), "loads": "balance", "vel": 0.0},
}


def options(rung):
    o = dict(BASE)
    o.update(RUNGS[rung])
    return o


def joint_of(m, name):
    """the joint (drive.JOINTS) a drive acts across: the one that joins its two bodies."""
    s = m.sys
    for a_ in s.acts:
        if a_[0] == name:
            pair = {a_[1], a_[2]}
            for j in Dr.JOINTS:
                if {m.jp[j][0], m.jp[j][2]} == pair:
                    return j
    return "?"


def energy(ctx, u):
    """the work of every element on the body from the start to contact, J, read at every step (0.25 ms): each muscle
    (its torque x its joint's speed about its axis, from the records), summed by joint; the hold-still torques and the
    joints' damping by joint (practice3d.hold_work); the stops and the ground (the engine's integrals); gravity (the
    weight x how far the centre of mass came down); and the body's gain in kinetic energy, which the rest must sum to."""
    m, s = ctx.m, ctx.m.sys
    fine = ctx.run(u, every=1)
    tx = ctx.contact(fine)[0]
    names = [a_[0] for a_ in s.acts]
    w = {n: 0.0 for n in names}
    prev = None
    last = None
    for rec in fine:
        if rec[0] > tx + 1e-12:
            break
        p = [a_[0] * a_[1] for a_ in rec[3]]
        if prev is not None:
            dt = rec[0] - prev[0]
            for n, p0, p1 in zip(names, prev[1], p):
                w[n] += 0.5 * (p0 + p1) * dt
        prev = (rec[0], p)
        last = rec

    def ke_pe(st):
        ke = pe = 0.0
        for i in range(s.nb):
            x, q, v, wv = s.pose(st, i)
            wl = E.mtv(E.qR(q), wv)
            ke += 0.5 * s.m[i] * E.dot(v, v) + 0.5 * sum(s.I[i][c] * wl[c] ** 2 for c in range(3))
            pe += s.m[i] * E.G * x[2]
        return ke, pe
    k0, p0 = ke_pe(fine[0][1])
    k1, p1 = ke_pe(last[1])
    tone_by, damp_by, damp_pk = PR.hold_work(s, fine, tx)
    ow = 13 * s.nb + len(s.acts)
    eng = [last[1][ow + q] - fine[0][1][ow + q] for q in range(4)]
    by_joint = {}
    for n, x in w.items():
        j = joint_of(m, n)
        by_joint[j] = by_joint.get(j, 0.0) + x
    hold_m = {n: x for n, x in w.items() if n.startswith("hold:")}
    out = {"muscles": sum(x for n, x in w.items() if not n.startswith("hold:")), "holding muscles": sum(hold_m.values()),
           "hold-still torques": sum(tone_by.values()), "damping": sum(damp_by.values()), "stops": eng[1], "ground": eng[3],
           "gravity": p0 - p1, "kinetic energy gained": k1 - k0,
           "muscles by channel": {n: x for n, x in w.items() if not n.startswith("hold:")}, "holding muscles by channel": hold_m,
           "muscles and holding muscles by joint": by_joint, "hold-still torques by joint": tone_by, "damping by joint": damp_by,
           "damping's largest torque by joint": damp_pk, "the engine's drives integral": eng[0]}
    out["sum of the elements less the kinetic energy gained"] = (out["muscles"] + out["holding muscles"] + out["hold-still torques"] + out["damping"]
                                                                 + out["stops"] + out["ground"] + out["gravity"] - out["kinetic energy gained"])
    return out


def gate(out):
    """the gate's rows (body-motion 13, PLAN D40) for a report: (check, what the swing has, the gate's figure, margin as
    text, passes?). The margin is how far outside (or inside) the gate's own test the figure lies."""
    rows = []

    def band(name, x, lo, hi, unit, fmt="%.1f"):
        ok = lo <= x <= hi
        mg = ("inside by " + fmt % min(x - lo, hi - x)) if ok else ("short by " + fmt % (lo - x) if x < lo else "over by " + fmt % (x - hi))
        rows.append((name, (fmt + " %s") % (x, unit), (fmt + " to " + fmt) % (lo, hi), mg + " " + unit, ok))
    miss = math.hypot(*out["miss_cm"])
    rows.append(("meets the ball", "%.1f cm" % miss, "1 cm", "%+.1f cm" % (miss - 1.0), miss <= 1.0 and out["came_round"]))
    band("the sweet spot at contact", out["sweet_speed"], 29.0, 34.0, "m/s")
    f = 100.0 * out["sweet_speed"] / out["sweet_fastest_before"][0]
    rows.append(("the bat at its fastest at contact", "%.0f %%" % f, "98 %", "%+.0f points" % (f - 98.0), f >= 98.0))
    band("the hands' fastest", out["hands_fastest"][0], 10.3, 13.0, "m/s")
    band("the attack angle", out["attack_deg"], -3.0, 11.0, "deg")
    band("the hips' fastest turning", out["pelvis_rot_vel_peak"][0], 555.0, 855.0, "deg/s", "%.0f")
    band("the trunk's fastest turning", out["trunk_rot_vel_peak"][0], 755.0, 1075.0, "deg/s", "%.0f")
    fr = out["first_rise_ms"]
    ok = fr["legs"][0] <= fr["trunk"][0] <= fr["arms"][0]
    rows.append(("first rises legs, trunk, arms", "%.0f, %.0f, %.0f ms" % (fr["legs"][0], fr["trunk"][0], fr["arms"][0]), "in that order", "", ok))
    allowed = out.get("clock_allowed_ms")
    if out.get("clock_ms") is not None and allowed is not None:
        rows.append(("no arm muscle before he could decide", "%.0f ms before contact" % out["clock_ms"], "%.0f ms" % allowed,
                     "%+.0f ms" % (out["clock_ms"] - allowed), out["clock_ms"] <= allowed + 0.5))
    band("the hands' first motion before contact", -out["bat_first_motion_ms"], 130.0, 220.0, "ms", "%.0f")
    band("the lead knee at contact", out["lead_knee"][2], 2.0, 26.0, "deg", "%.0f")
    band("the front foot's greatest load", out["lead_foot_peak"][0], 1.5, 2.6, "x body weight", "%.2f")
    xo, least, zr = out["balance"]
    rows.append(("in balance", "%.1f cm outside" % xo, "0", "%+.1f cm" % xo, xo <= 0.05))
    rows.append(("does not jump", "feet never under %.2f of his weight" % least, "0.10", "%+.2f" % (least - 0.10), least >= 0.10))
    rows.append(("legs have not given way", "ends at %.2f of its height" % zr, "0.90", "%+.2f" % (zr - 0.90), zr >= 0.90))
    past = max((x[3], k) for k, x in out["stops"].items())
    past_after = max((x[1], k) for k, x in out["stops"].items())
    rows.append(("no joint more than 2 deg past its range", "%.1f deg (%s) to contact; %.1f (%s) to 50 ms after" % (past + past_after), "2 deg",
                 "%+.1f deg" % (max(past[0], past_after[0]) - 2.0), max(past[0], past_after[0]) <= 2.0))
    lig = out["ligaments_over_50"]
    worst = max(((x[0], k) for k, x in out["stops"].items() if k != "shoulder_L"), default=(0.0, "-"))
    rows.append(("no joint end heavily loaded (D39, not built)", "%d ends over 50 N m; the most %.0f N m (%s)" % (len(lig), worst[0], worst[1]), "none", "", not lig))
    wk = out["work_J"]
    ok = wk["stops"] <= 0.5 and wk["stops to +50 ms"] <= 0.5 and wk["ground"] <= 0.5 and wk.get("damping", 0.0) <= 0.5
    rows.append(("no passive element does net positive work", "stops %+.0f J, ground %+.0f J, damping %+.0f J" % (wk["stops"], wk["ground"], wk.get("damping", 0.0)), "at most 0", "", ok))
    return rows


def show(ctx, u, log=print):
    out = PR.report(ctx, u, log)
    out["clock_allowed_ms"] = ctx.t_avail * 1e3
    en = energy(ctx, u)
    out["energy_J"] = en
    log("   THE WORK OF EVERY ELEMENT TO CONTACT (J): muscles %+.1f; holding muscles %+.1f; hold-still torques %+.1f; the joints' damping %+.1f; stops %+.1f; ground %+.1f; gravity %+.1f;   the body's kinetic energy gained %+.1f (the elements' sum less that: %+.2f)"
        % (en["muscles"], en["holding muscles"], en["hold-still torques"], en["damping"], en["stops"], en["ground"], en["gravity"], en["kinetic energy gained"],
           en["sum of the elements less the kinetic energy gained"]))
    srt = lambda d_, n=99: ", ".join("%s %+.1f" % kv for kv in sorted(d_.items(), key=lambda kv: -abs(kv[1]))[:n] if abs(kv[1]) >= 1.0)
    log("      muscles (with holding muscles) by joint: " + srt(en["muscles and holding muscles by joint"]))
    log("      muscles by channel (the twelve largest): " + srt(en["muscles by channel"], 12))
    if en["holding muscles by channel"]:
        log("      holding muscles by channel: " + srt(en["holding muscles by channel"]))
    log("      hold-still torques by joint: " + srt(en["hold-still torques by joint"]))
    log("      the joints' damping by joint: " + srt(en["damping by joint"]))
    rows = gate(out)
    out["gate"] = rows
    log("   THE GATE (body-motion 13), row by row:")
    for name, has, asks, mg, ok in rows:
        log("      %-46s %-58s [%s]  %s  %s" % (name, has, asks, mg, "passes" if ok else "FAILS"))
    return out


def run_rung(rung, start, seed, g_reach, g_aim, g_more, tag, log):
    o = options(rung)
    ctx = PR.Ctx(contact="nearest", **o)
    reach = PR.Ctx(stage="reach", **o)
    if start == "S":
        plan = json.load(open(os.path.join(HERE, "practice3d_s9_cleats_S.json")))["plan"]
    else:
        plan = json.load(open(start))["plan"]
    plan = ctx.fit_plan(plan) if hasattr(ctx, "fit_plan") else {n: [min(v[0], ctx.bd[n])] + list(v[1:]) for n, v in plan.items()}
    u0 = ctx.u_of(plan)
    log("== ladder rung %s%s from %s; body %s; %d generations of the reach stage, %d of the whole aim, %d more, then blocks of %d until it has fallen under %.1f over its last 50 (at most %d blocks); %d plans each, every run at %.3f ms; seed %d"
        % (rung, tag, start, o, g_reach, g_aim, g_more, BLOCK, SETTLED, MAX_BLOCKS, PR.CMA_LAMBDA, o["h"] * 1e3, seed))
    r0 = ctx.residuals(u0)
    log("   the starting plan's score on this body: %s; by term: %s" % ("THE RUN FAILS" if r0 is None else "%.1f" % PR.cost_of(r0), "" if r0 is None else "; ".join("%s %.1f" % kv for kv in PR.terms(ctx, r0).items())))
    t = time.time()
    hist_all, u = [], u0
    stages = ([("the reach stage", reach, g_reach, seed)] if g_reach else []) + ([("the whole aim", ctx, g_aim, seed + 1)] if g_aim else []) \
        + ([("carried on", ctx, g_more, seed + 11)] if g_more else [])
    blocks = 0
    spread = (0.0, 0.0)
    cost = PR.cost_of(r0)
    while stages:
        name, c, gens, sd = stages.pop(0)
        PR._CTX = c
        log("-- %s: %d generations, seed %d" % (name, gens, sd))
        with multiprocessing.get_context("fork").Pool(PR.WORKERS) as pool:
            u, cost, hist, mfull, sp = PR.cma(c, u, gens, log, pool, sd)
        spread = (max(sp), sorted(sp)[len(sp) // 2])
        fall = hist[max(0, len(hist) - 51)][1] - hist[-1][1]
        hist_all.append((name, hist))
        log("   %s: %d generations, %.0f s so far; best score %.2f; it fell %.2f over the last 50 generations; spreads at the end: largest %.4f, median %.4f; failed runs %d"
            % (name, gens, time.time() - t, cost, fall, spread[0], spread[1], sum(h_[5] for h_ in hist)))
        if not stages and c is ctx and fall >= SETTLED and blocks < MAX_BLOCKS:
            blocks += 1
            stages.append(("carried on, block %d" % blocks, ctx, BLOCK, seed + 11 + 10 * blocks))
    settled = fall < SETTLED
    log("   %s: %.2f over its last 50 generations (the rule: under %.1f)" % ("SETTLED" if settled else "NOT SETTLED", fall, SETTLED))
    out = show(ctx, u, log)
    out.update(cost=cost, rung=rung, start=start, seed=seed, history=hist_all, settled=settled, fall_last_50=fall, spread_end=spread,
               step_check=PR.verify(ctx, u, log), failed_differences=PR.failed_differences(ctx, u, log),
               search_checks=PR.search_checks(dict(o, contact="nearest"), u0, u, log), start_plan=plan, seconds=time.time() - t)
    os.makedirs(OUT, exist_ok=True)
    path = os.path.join(OUT, "%s%s.json" % (rung, tag))
    json.dump(out, open(path, "w"), indent=1)
    log("   kept: %s   (%.0f s)" % (os.path.relpath(path, os.path.join(HERE, "..", "..", "..")), time.time() - t))
    return out


def table(log=print):
    """the ladder so far, from the results kept: one line a result."""
    log("| Result | Start | Sweet spot at contact | Miss | Score | Settled | Muscles; holding; hold-still; damping (J) |")
    log("| :-- | :-- | --: | --: | --: | :-- | :-- |")
    if not os.path.isdir(OUT):
        return
    for f in sorted(os.listdir(OUT)):
        if not f.endswith(".json"):
            continue
        d = json.load(open(os.path.join(OUT, f)))
        if "sweet_speed" not in d:
            continue
        en = d.get("energy_J", {})
        log("| %s | %s | %.2f m/s (%.1f mph) | %.1f cm | %.1f | %s | %s |" % (f[:-5], os.path.basename(str(d.get("start"))), d["sweet_speed"], d["sweet_speed"] / MPH, math.hypot(*d["miss_cm"]), d.get("cost", float("nan")),
            ("yes (%.2f)" if d.get("settled") else "NO (%.2f)") % d.get("fall_last_50", float("nan")),
            "%.0f; %.0f; %.0f; %.0f" % (en.get("muscles", 0), en.get("holding muscles", 0), en.get("hold-still torques", 0), en.get("damping", 0)) if en else "-"))


def main():
    P.build()
    a = sys.argv[1:]
    arg = lambda k, d: a[a.index(k) + 1] if k in a else d
    log = lambda s_: print(s_, flush=True)
    if "--table" in a:
        table(log)
        return
    if "--show" in a:
        d = json.load(open(arg("--show", "")))
        o = {k: v for k, v in d.get("options", {}).items()}
        if "--rung" in a:
            o = dict(options(arg("--rung", "r0")), contact="nearest")
        ctx = PR.Ctx(**o)
        plan = ctx.fit_plan(d["plan"]) if hasattr(ctx, "fit_plan") else d["plan"]
        out = show(ctx, ctx.u_of(plan), log)
        if "--verify" in a:
            PR.verify(ctx, ctx.u_of(plan), log)
        if "--out" in a:
            json.dump(out, open(arg("--out", ""), "w"), indent=1)
        return
    if "--rung" in a:
        run_rung(arg("--rung", "r0"), arg("--from", "S"), int(arg("--seed", 1)), int(arg("--reach", 100)), int(arg("--gens", 250)), int(arg("--more", 500)), arg("--tag", ""), log)
        return
    print(__doc__)


if __name__ == "__main__":
    main()
