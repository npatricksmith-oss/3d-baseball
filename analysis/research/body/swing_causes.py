"""What each part of a kept swing is doing: the readings and single replays behind docs/body-motion.md 4.15,
"What the swing looks like" (session 13; committed after its review, P3-3, so that each can be run again).

    python3 analysis/research/body/swing_causes.py <plan.json>              the readings (one forward run)
    python3 analysis/research/body/swing_causes.py <plan.json> --replays    and the replays (one run each)

NO SEARCH. One forward run of the kept plan, read; then the same plan with ONE THING CHANGED and nothing tuned again.
A REPLAY SHOWS WHAT THAT THING CONTRIBUTES IN THIS PLAN. IT DOES NOT SHOW WHY THE SEARCH CHOSE THE PLAN: a searched plan
with one muscle removed is not the plan a search would find without that muscle (the review of session 13, P2-6).

The readings:
 1. where he folds: the trunk on the pelvis (forward, to the side, twisted), the pelvis's own tip, both hips, the hands'
    and shoulders' heights, and what the trunk's holding muscles carry against their bounds;
 2. what the ground would have to carry for each part of him (its mass x (its upward acceleration + g)), against what
    the ground does carry: which part is dropping;
 3. the barrel into the zone: the sweet spot's height, speed, direction and distance from the ball's line, and for how
    long (and over how much of the line) it is within 3, 5 and 8 cm of it;
 4. the turns as angles and not only as turns since the start (the chest against the pelvis at the start and at
    contact), the forward travel of his centre of mass and head from the start and from his front foot's landing, and
    the front foot's turn and load.
The replays: one muscle's burst taken out (its holding level stays); and "a back that holds": the trunk held to the
pelvis at its starting bend by a stiff spring whose torque is capped (the engine's capped hold, as pelvis3d's what-if
foot hold; RANGE_K["big"], RANGE_C["big"]), about both bending axes or the forward one only, capped at the trunk
extensors' measured bound (482 N m) or at 150.
"""
import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import practice3d as PR                                # noqa: E402
import probe3d as P                                    # noqa: E402
from probe3d import E, B                               # noqa: E402

C = B.C
D2R = math.pi / 180.0
GROUPS = {"legs and feet": ("thigh_L", "shank_L", "foot_L", "thigh_R", "shank_R", "foot_R"), "pelvis": ("pelvis",),
          "trunk and head": ("trunk",), "arms and bat": ("uarm_L", "farm_L", "uarm_R", "farm_R", "bat")}
HANDS_U = 0.5 * (B.GRIP_L + B.GRIP_R)
OFF = (("without the front hip flexor's burst", ("hip_flex_L",)), ("without the trunk's twisting muscle", ("core",)),
       ("without the rear hip's extensor", ("hip_ext_R",)), ("without both shoulders' lowering muscles", ("sh_lower_L", "sh_lower_R")),
       ("without the muscle that raises the lead arm", ("sh_raise_L",)), ("without any leg muscle's burst", tuple(PR.LOWER[:-2])))
BACK = (("his back holding its bend and side bend (up to 482 N m)", 482.0, (1, 1, 0)), ("the same, up to 150 N m", 150.0, (1, 1, 0)),
        ("his back holding its forward bend only (482 N m)", 482.0, (0, 1, 0)))


def head_of(m, st):
    """where the viewer and swing_look draw his head: 0.20 m up the trunk's own axis from between the shoulders (the
    model has no neck and no head of its own: the head is part of the trunk)."""
    zt = m.axis(st, "trunk", (0, 0, 1))
    shm = tuple(0.5 * (a + b) for a, b in zip(m.joint_pos(st, "shoulder_L"), m.joint_pos(st, "shoulder_R")))
    return E.add(shm, E.scale(zt, 0.20))


def readings(d, log=print):
    ctx = PR.Ctx(**d["options"])
    u = ctx.u_of(d["plan"])
    m, s = ctx.m, ctx.m.sys
    recs = ctx.run(u, every=int(round(1e-3 / ctx.h)))           # 1 ms records
    tx = ctx.contact(recs)[0]
    at = lambda tms: min(range(len(recs)), key=lambda k: abs((recs[k][0] - tx) * 1e3 - tms))
    names = [a[0] for a in s.acts]
    hb = ctx.hold_bd
    log("1. WHERE HE FOLDS. t ms | trunk bent forward on the pelvis, to his side, twisted | pelvis tipped forward, sideways | hip flexion L, R | hands' height cm, shoulders' L, R | the trunk's holding muscles' torque, N m")
    for t in (-270, -200, -150, -100, -80, -60, -40, -20, 0, 20, 40, 60):
        k = at(t)
        st = recs[k][1]
        a = B.range_angles(m, st)
        xp, yp = m.axis(st, "pelvis", (1, 0, 0)), m.axis(st, "pelvis", (0, 1, 0))
        hands = m.bat_point(st, HANDS_U)[0]
        ho = {n[5:]: recs[k][3][j][0] for j, n in enumerate(names) if n.startswith("hold:trunk")}
        log("%+5d | %+5.1f %+5.1f %+5.1f | %+5.1f %+5.1f | %5.1f %5.1f | %4.0f, %4.0f %4.0f | %s" % (
            t, a["trunk_bend"], a["trunk_side"], a["trunk_twist"], math.asin(max(-1, min(1, -xp[2]))) / D2R, math.asin(max(-1, min(1, yp[2]))) / D2R, a["hip_flex_L"], a["hip_flex_R"],
            hands[2] * 100, m.joint_pos(st, "shoulder_L")[2] * 100, m.joint_pos(st, "shoulder_R")[2] * 100, " ".join("%s %+.0f" % kv for kv in sorted(ho.items()))))
    log("   the holding groups' bounds, N m: %s; their holding strengths: %s" % ({k: round(v) for k, v in hb.items() if k.startswith("trunk")}, {k: round(v) for k, v in ctx.hold_T.items() if k.startswith("trunk")}))
    pl = ctx.plan(u)
    log("   the hips' extensors in the plan, N m of bound: " + ", ".join("%s %.1f of %.0f" % (n, pl[n][0], ctx.bd[n]) for n in ("hip_ext_L", "hip_ext_R")) + "; the front hip's flexor %.0f of %.0f" % (pl["hip_flex_L"][0], ctx.bd["hip_flex_L"]))
    vz = lambda k, bs: sum(s.m[m.b[b]] * s.pose(recs[k][1], m.b[b])[2][2] for b in bs)
    log("\n2. WHAT THE GROUND WOULD HAVE TO CARRY FOR EACH PART (its mass x (its upward acceleration + g), of body weight), mean over each 20 ms; and what the ground carries")
    log("   window ms    | " + " | ".join("%s (%.0f kg)" % (g, sum(s.m[m.b[b]] for b in bs)) for g, bs in GROUPS.items()) + " | sum | the ground carries")
    for t0 in range(-120, 60, 20):
        k0, k1 = at(t0), at(t0 + 20)
        dt = recs[k1][0] - recs[k0][0]
        row = [((vz(k1, bs) - vz(k0, bs)) / dt + sum(s.m[m.b[b]] for b in bs) * E.G) / ctx.BW for bs in GROUPS.values()]
        grf = sum(sum(c[2] for c in recs[k][2]) for k in range(k0, k1 + 1)) / (k1 - k0 + 1) / ctx.BW
        log("   %+4d to %+4d | " % (t0, t0 + 20) + " | ".join("%+.2f" % x for x in row) + " | %+.2f | %.2f" % (sum(row), grf))
    log("   standing still each part needs its own share: " + ", ".join("%s %.2f" % (g, sum(s.m[m.b[b]] for b in bs) * E.G / ctx.BW) for g, bs in GROUPS.items()))
    log("   at contact, moving down (m/s): " + ", ".join("%s %.2f" % (g, -vz(at(0), bs) / sum(s.m[m.b[b]] for b in bs)) for g, bs in GROUPS.items()))
    log("\n3. THE BARREL INTO THE ZONE. t ms | sweet spot's height cm | speed m/s | going down (-) or up (+) deg | from the ball's line cm: above (+), in all | barrel below level deg | hands' height cm, their distance from between the shoulders cm")
    off = lambda st: (lambda dd: E.sub(dd, E.scale(ctx.nb, E.dot(dd, ctx.nb))))(E.sub(m.bat_point(st, C.BAT_SS)[0], ctx.Pc))
    for t in (-100, -80, -60, -50, -40, -30, -20, -10, 0, 10, 20):
        st = recs[at(t)][1]
        ss, v = m.bat_point(st, C.BAT_SS)
        o = m.observe(st)
        perp = off(st)
        hands = m.bat_point(st, HANDS_U)[0]
        shm = tuple(0.5 * (a + b) for a, b in zip(m.joint_pos(st, "shoulder_L"), m.joint_pos(st, "shoulder_R")))
        log("%+5d | %5.0f | %5.1f | %+5.0f | %+6.1f, %5.1f | %+4.0f | %4.0f, %4.0f" % (t, ss[2] * 100, E.norm(v), math.asin(v[2] / E.norm(v)) / D2R, E.dot(perp, ctx.ev) * 100, E.norm(perp) * 100, -o["bat_elev"], hands[2] * 100, E.norm(E.sub(hands, shm)) * 100))
    for lim in (0.03, 0.05, 0.08):
        ks = [k for k in range(len(recs)) if abs(recs[k][0] - tx) < 0.06 and E.norm(off(recs[k][1])) <= lim]
        k0, k1 = min(ks), max(ks)
        along = lambda k: -E.dot(E.sub(m.bat_point(recs[k][1], C.BAT_SS)[0], ctx.Pc), ctx.nb)
        log("   the sweet spot is within %.0f cm of the ball's line from %+.1f to %+.1f ms (%.0f ms; %.0f cm of the line)" % (lim * 100, (recs[k0][0] - tx) * 1e3, (recs[k1][0] - tx) * 1e3, (recs[k1][0] - recs[k0][0]) * 1e3, (along(k1) - along(k0)) * 100))
    log("\n4. ANGLES, TRAVEL AND THE FRONT FOOT")
    o0, oc = m.observe(recs[0][1]), m.observe(recs[at(0)][1])
    wrap = lambda x: (x + 180.0) % 360.0 - 180.0
    log("   the chest against the pelvis (the trunk's turn less the pelvis's, as angles; + = the chest further round): %+.1f deg at the start, %+.1f at contact; since the start the pelvis has turned %+.1f and the chest %+.1f"
        % (wrap(o0["trunk_rot"] - o0["pelvis_rot"]), wrap(oc["trunk_rot"] - oc["pelvis_rot"]), wrap(oc["pelvis_rot"] - o0["pelvis_rot"]), wrap(oc["trunk_rot"] - o0["trunk_rot"])))
    fz = lambda k, sd: sum(f[2] for cn, f in zip(s.contacts, recs[k][2]) if cn[0][0] == sd) / ctx.BW
    kd = next(k for k in range(len(recs)) if any(f[2] > 0.0 for cn, f in zip(s.contacts, recs[k][2]) if cn[0].startswith("L_heel")))
    com = lambda k: B.com(m, recs[k][1])
    log("   his front heel first carries load %.0f ms before contact; his centre of mass is then moving toward the pitcher at %.2f m/s (%.2f at the start)" % (-(recs[kd][0] - tx) * 1e3, com(kd)[1][1], com(0)[1][1]))
    for label, k in (("contact", at(0)), ("60 ms after", at(60)), ("170 ms after", at(170))):
        h, h0 = head_of(m, recs[k][1]), head_of(m, recs[0][1])
        hd = head_of(m, recs[kd][1])
        log("   by %s: his centre of mass has gone %+.1f cm toward the pitcher from the start and %+.1f from his front foot's landing; the top of his trunk (the drawn head) %.1f cm from the start (%+.1f toward the pitcher, %+.1f up) and %+.1f toward the pitcher from the landing"
            % (label, (com(k)[0][1] - com(0)[0][1]) * 100, (com(k)[0][1] - com(kd)[0][1]) * 100, E.norm(E.sub(h, h0)) * 100, (h[1] - h0[1]) * 100, (h[2] - h0[2]) * 100, (h[1] - hd[1]) * 100))
    yaw = lambda k: math.atan2(*reversed(m.axis(recs[k][1], "foot_L", (1, 0, 0))[:2])) / D2R
    log("   his front foot points %+.0f deg from straight at the plate at the start and %+.0f at contact (+ = the toe toward the pitcher, an open foot): it has turned %+.0f deg, carrying %.2f of his weight at contact and %.2f on average over the last 100 ms"
        % (yaw(0), yaw(at(0)), wrap(yaw(at(0)) - yaw(0)), fz(at(0), "L"), sum(fz(k, "L") for k in range(at(-100), at(0) + 1)) / (at(0) + 1 - at(-100))))
    hands0, handsc = m.bat_point(recs[0][1], HANDS_U)[0], m.bat_point(recs[at(0)][1], HANDS_U)[0]
    log("   his hands are %.0f cm up at the start and %.0f at contact; the ball's place is %.0f cm up; elbows at contact: lead %.0f deg bent, rear %.0f" % (hands0[2] * 100, handsc[2] * 100, ctx.Pc[2] * 100, oc["lead_elbow_flex"], oc["trail_elbow_flex"]))


def replay(d, label, off=(), back=None, log=print):
    ctx = PR.Ctx(**d["options"])
    m, s = ctx.m, ctx.m.sys
    plan = {n: list(v) for n, v in d["plan"].items()}
    for n in off:
        plan[n][0] = 0.0
    if back:
        cap, axes = back
        a, b = m.b["pelvis"], m.b["trunk"]
        qa, qb = tuple(ctx.st[13 * a + 3:13 * a + 7]), tuple(ctx.st[13 * b + 3:13 * b + 7])
        s.hold("back", a, b, E.qmul(E.qconj(qa), qb), tuple(B.RANGE_K["big"] * x for x in axes), tuple(B.RANGE_C["big"] * x for x in axes), float(cap))
    u = ctx.u_of(plan)
    try:
        recs = ctx.run(u, every=4)
    except E.SolveError as e:
        log("%-56s THE RUN FAILS (%s)" % (label, str(e)[:60]))
        return
    tx, p, v, i = ctx.nearest(recs)
    dd = E.sub(p, ctx.Pc)
    k = min(range(len(recs)), key=lambda q: abs(recs[q][0] - tx))
    st = recs[k][1]
    zt, xp = m.axis(st, "trunk", (0, 0, 1)), m.axis(st, "pelvis", (1, 0, 0))
    o, o0, a = m.observe(st), m.observe(recs[0][1]), B.range_angles(m, st)
    load = lambda q: sum(c[2] for c in recs[q][2]) / ctx.BW
    k50 = min(range(len(recs)), key=lambda q: abs(recs[q][0] - (tx - 0.05)))
    hands = m.bat_point(st, HANDS_U)[0]
    log("%-56s nearest the ball %5.1f cm (%+5.1f up) at %.1f m/s, %3.0f ms after the start; lean %2.0f (pelvis tipped %+3.0f, waist %+3.0f, to the side %+3.0f); barrel %2.0f below level; hands %3.0f cm up; pelvis turned %+3.0f, chest %+3.0f; feet carry %.2f there, %.2f mean of the last 50 ms; lead hip %+.0f; body going down %.2f m/s"
        % (label, E.norm(dd) * 100, E.dot(dd, ctx.ev) * 100, E.norm(v), (tx - PR.T0) * 1e3, math.acos(zt[2]) / D2R, math.asin(-xp[2]) / D2R, a["trunk_bend"], a["trunk_side"], -o["bat_elev"], hands[2] * 100,
           o["pelvis_rot"] - o0["pelvis_rot"], o["trunk_rot"] - o0["trunk_rot"], load(k), sum(load(q) for q in range(k50, k + 1)) / (k + 1 - k50), a["hip_flex_L"], -B.com(m, st)[1][2]))


def main():
    a = sys.argv[1:]
    if not a or a[0].startswith("-"):
        print(__doc__)
        return
    P.build()
    d = json.load(open(a[0]))
    print("%s (%.2f m/s at contact)" % (os.path.basename(a[0]), d.get("sweet_speed", float("nan"))))
    readings(d)
    if "--replays" in a:
        print("\n5. REPLAYS: the kept plan with one thing changed, nothing tuned again (what the thing contributes in this plan; not why the search chose it)")
        replay(d, "the kept swing")
        for label, cap, axes in BACK:
            replay(d, label, back=(cap, axes))
        for label, off in OFF:
            replay(d, label, off=off)


if __name__ == "__main__":
    main()
