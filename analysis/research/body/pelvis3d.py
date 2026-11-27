"""What holds the pelvis: replays of kept swings with one thing changed (docs/body-motion.md 4.14, part 1; session 12).

    python3 analysis/research/body/pelvis3d.py <result.json> [replay ...]     the replays named (default: all), one line each
    python3 analysis/research/body/pelvis3d.py <result.json> --series 0 FR    the same, with a reading every 10 ms
    ... --rung r3_oldball                                                     the body by its name in ladder3d.RUNGS
    python3 analysis/research/body/pelvis3d.py --check                        the readings and the held foot against worked cases

A DIAGNOSIS, NO SEARCH. Each replay takes a kept plan and its body, changes ONE thing and runs once at the search's
step. The plan was tuned on the unchanged body, so a replay is not a swing he would make and its bat speed means little
(rule 4): what is read is what the legs and pelvis do with the same muscle bursts.

The replays (4.14's table): 0 none; F, R, FR the front, rear, both feet held from turning about the vertical; L1 the
ground's damping x 4; L2 a softer ground; L3 [4]'s launch velocities at half; H the hips' ends of turn 30 deg further out
(a diagnosis only: no stop is moved in the body); FR+H; W the sole's points further from the foot's line by GRIP_TOP (the
twisting hold at the top of the measured range; it also makes the foot wider to roll over, which is said); M the grip at
MU_TOP; W+M; B the front knee's extensor at all it has from 150 ms before contact (the plan's one muscle, not the body); B+F.
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

HOLD_K, HOLD_C = B.RANGE_K["big"], B.RANGE_C["big"]    # N m/rad, N m s/rad [E]: the held foot's spring, as stiff as a large joint's end
HIP_OUT = 30.0                                         # deg: how far replay H moves each end of the hips' turn
UPPER = ("pelvis", "trunk", "bat", "uarm_L", "farm_L", "uarm_R", "farm_R")
LEGS = ("thigh_L", "shank_L", "foot_L", "thigh_R", "shank_R", "foot_R")
REPLAYS = ("0", "F", "R", "FR", "L1", "L2", "L3", "H", "FR+H", "W", "M", "W+M", "B", "B+F")
GRIP_TOP = 100.1 / 68.5                                # Brosnan 2009: the baseball shoe's twisting hold on basepath dirt, its largest over its least (the model stands at the least)
BRACE_BEFORE = 0.150                                   # s before contact: when replay B's front knee extensor starts
MU_TOP = 1.27                                          # the same paper's largest static traction coefficient (the model's 1.15 is the middle of 1.04-1.27)
D2R = B.D2R


def hold_foot(ctx, sd):
    """the foot's heading held where it starts: the engine's stop with the world as its parent (passive), both ways from
    the start's heading. Its torque is recorded with every step as the stop "foothold_<side>"."""
    m, s = ctx.m, ctx.m.sys
    f = m.axis(ctx.st, "foot_" + sd, (1, 0, 0))
    e = E.unit((f[0], f[1], 0.0))
    s.stop("foothold_" + sd, -1, m.b["foot_" + sd], (0, 0, 1), e, (1, 0, 0), 0.0, 0.0, HOLD_K, HOLD_C, 0)


def build(d, replay, extra=None):
    """the kept result's body with the replay's one change: (ctx, the plan as the search's numbers)."""
    o = dict(d["options"])
    o["contact"] = "nearest"
    o.update(extra or {})
    parts = replay.split("+")
    kn, cn = B.KN, B.CN
    bw, sw = B.BALL_HALF_W, B.SOLE_HALF_W
    try:
        if "W" in parts:
            B.BALL_HALF_W, B.SOLE_HALF_W = bw * GRIP_TOP, sw * GRIP_TOP
        if "M" in parts:
            o["mu"] = MU_TOP
        if "L1" in parts:
            B.CN = cn * 4.0
        if "L2" in parts:
            B.KN, B.CN = kn / 4.0, cn / 2.0
        if "L3" in parts:
            o["vel"] = 0.5
        if "H" in parts:
            rom = dict(o.get("rom") or {})
            rom["hip_irot"] = rom.get("hip_irot", B.ROM["hip_irot"]) + HIP_OUT
            rom["hip_erot"] = rom.get("hip_erot", B.ROM["hip_erot"]) + HIP_OUT
            o["rom"] = rom
        ctx = PR.Ctx(**o)
    finally:
        B.KN, B.CN = kn, cn
        B.BALL_HALF_W, B.SOLE_HALF_W = bw, sw
    for p_ in parts:
        if p_ in ("F", "FR"):
            hold_foot(ctx, "L")
        if p_ in ("R", "FR"):
            hold_foot(ctx, "R")
    plan = d["plan"]
    plan = {n: [min(v[0], ctx.cap[n])] + list(v[1:]) for n, v in plan.items()}
    if "B" in parts:
        # the front knee's extensor asked for all it has from BRACE_BEFORE before the unchanged swing's contact (a change to
        # the PLAN, one muscle; the body is unchanged): does a front leg that pushes load its foot, and does the foot then hold?
        tx = PR.T0 + d["start_to_contact_ms"] * 1e-3
        plan["knee_ext_L"] = [ctx.cap["knee_ext_L"], tx - BRACE_BEFORE, PR.Dr.RISE_MIN, 0.150, plan["knee_ext_L"][4]]
    return ctx, ctx.u_of(plan)


def heading(m, st, body):
    """the direction of a body's own x (a foot's length, the pelvis's and a thigh's front) seen from above, deg."""
    a = m.axis(st, body, (1, 0, 0))
    return math.atan2(a[1], a[0]) / D2R


def wrap(a):
    return (a + 180.0) % 360.0 - 180.0


def turning_momentum(s, st, bodies, c, vc):
    """the bodies' angular momentum about the vertical line through the point c moving at vc (kg m^2/s): each body's
    m (x - c) x (v - vc) and its own R I R^T w, the vertical parts."""
    hz = 0.0
    for i in bodies:
        x, q, v, w = s.pose(st, i)
        r, dv = E.sub(x, c), E.sub(v, vc)
        hz += s.m[i] * (r[0] * dv[1] - r[1] * dv[0])
        R = E.qR(q)
        wl = E.mtv(R, w) if hasattr(E, "mtv") else tuple(sum(R[k][j] * w[k] for k in range(3)) for j in range(3))
        hl = (s.I[i][0] * wl[0], s.I[i][1] * wl[1], s.I[i][2] * wl[2])
        hz += E.mv(R, hl)[2]
    return hz


def ground_twist(m, st, forces, c):
    """the ground's moment on the whole body about the vertical through c (N m), in its parts: for each foot (its load N,
    the moment of its net level force acting at its centre of pressure, its own twisting moment about that centre, the
    number of its sole points carrying load)."""
    s = m.sys
    out = {}
    for sd in "LR":
        pts = []
        for cdef, f in zip(s.contacts, forces):
            if cdef[0][0] != sd or f[2] <= 0.0:
                continue
            xb, qb, _, _ = s.pose(st, cdef[1])
            pts.append((E.add(xb, E.mv(E.qR(qb), cdef[2])), f))
        N = sum(f[2] for p, f in pts)
        if N <= 0.0:
            out[sd] = (0.0, 0.0, 0.0, 0)
            continue
        cx = sum(p[0] * f[2] for p, f in pts) / N
        cy = sum(p[1] * f[2] for p, f in pts) / N
        fx, fy = sum(f[0] for p, f in pts), sum(f[1] for p, f in pts)
        lever = (cx - c[0]) * fy - (cy - c[1]) * fx
        free = sum((p[0] - cx) * f[1] - (p[1] - cy) * f[0] for p, f in pts)
        out[sd] = (N, lever, free, len(pts))
    return out


def read(ctx, u, t_read=None):
    """one replay's readings. t_read: the instant (s after the start) the readings are taken to: the unchanged swing's
    contact; None takes this replay's own contact (replay 0)."""
    m, s = ctx.m, ctx.m.sys
    recs = ctx.run(u, every=1)
    t0 = recs[0][0]
    tx, px, vx, ix, came = ctx.contact(recs)
    tn, pn, vn, _ = ctx.nearest(recs)
    T = (tx - t0) if t_read is None else t_read
    names = [x[0] for x in s.stops]
    ix_of = {n: names.index(n) for n in names}
    idx = {k: m.b[k] for k in m.b}
    upper, legs = [idx[k] for k in UPPER], [idx[k] for k in LEGS]
    h = recs[1][0] - recs[0][0]
    rows = []
    imp = 0.0                                             # the time integral of the ground's and the holds' twist: what they gave the body
    prev = None
    for rec in recs:
        t = rec[0] - t0
        if t > T + 1e-9:
            break
        st = rec[1]
        c, vc = B.com(m, st)
        gt = ground_twist(m, st, rec[2], c)
        holds = {sd: (rec[4][ix_of["foothold_" + sd]][0] if "foothold_" + sd in ix_of else 0.0) for sd in "LR"}
        mz = sum(gt[sd][1] + gt[sd][2] for sd in "LR") + holds["L"] + holds["R"]
        if prev is not None:
            imp += 0.5 * (mz + prev) * h
        prev = mz
        rows.append(dict(t=t, pelvis=heading(m, st, "pelvis"), thigh_L=heading(m, st, "thigh_L"), thigh_R=heading(m, st, "thigh_R"),
                         foot_L=heading(m, st, "foot_L"), foot_R=heading(m, st, "foot_R"),
                         pelvis_rate=s.pose(st, idx["pelvis"])[3][2] / D2R,
                         H_upper=turning_momentum(s, st, upper, c, vc), H_legs=turning_momentum(s, st, legs, c, vc),
                         ground=gt, hold=holds, twist=mz, given=imp,
                         hip_L=rec[4][ix_of["hip_rot_L"]], hip_R=rec[4][ix_of["hip_rot_R"]], knee_L=m.flexion(st, "thigh_L", "shank_L")))
    r0, r1 = rows[0], rows[-1]
    BW = ctx.BW
    last = [r for r in rows if r["t"] >= T - 0.100]
    mean = lambda f: sum(f(r) for r in last) / len(last)
    lim = ctx.limits
    out = dict(T_ms=T * 1e3, came=came, own_contact_ms=(tx - t0) * 1e3 if came else None,
               sweet_speed=E.norm(vx) if came else None, miss_cm=100.0 * E.norm(E.sub(px, ctx.Pc)) if came else None,
               nearest_cm=100.0 * E.norm(E.sub(pn, ctx.Pc)), nearest_speed=E.norm(vn), nearest_ms=(tn - t0) * 1e3,
               pelvis_turn=wrap(r1["pelvis"] - r0["pelvis"]), pelvis_rate_peak=max(r["pelvis_rate"] for r in rows),
               turn={k: wrap(r1[k] - r0[k]) for k in ("thigh_L", "thigh_R", "foot_L", "foot_R")},
               hip_end={sd: None for sd in "LR"}, hip_peak={sd: max(abs(r["hip_" + sd][0]) for r in rows) for sd in "LR"},
               load_last100={sd: mean(lambda r: r["ground"][sd][0]) / BW for sd in "LR"},
               load_at_T={sd: r1["ground"][sd][0] / BW for sd in "LR"},
               few_points_ms={sd: 1e3 * h * sum(1 for r in last if 0 < r["ground"][sd][3] < 2) for sd in "LR"},
               off_ms={sd: 1e3 * h * sum(1 for r in last if r["ground"][sd][3] == 0) for sd in "LR"},
               H_upper=r1["H_upper"], H_legs=r1["H_legs"], H_upper0=r0["H_upper"], H_legs0=r0["H_legs"], given=r1["given"],
               H_upper_peak=max(r["H_upper"] for r in rows),
               lever_last100={sd: mean(lambda r: r["ground"][sd][1]) for sd in "LR"}, free_last100={sd: mean(lambda r: r["ground"][sd][2]) for sd in "LR"},
               hold_peak={sd: max(abs(r["hold"][sd]) for r in rows) for sd in "LR"}, knee_L=r1["knee_L"],
               limits={k: lim[k] for k in ("hip_rot_L", "hip_rot_R")}, rows=rows)
    for sd in "LR":
        lo, hi = lim["hip_rot_" + sd]
        for r in rows:
            if abs(r["hip_" + sd][0]) > 1.0:              # the stop carries load: the hip is on an end of its turn
                out["hip_end"][sd] = (1e3 * (r["t"] - T), r["hip_" + sd][1] / D2R)
                break
    # with a held foot: the hold's largest torque with the foot's load at that instant, and the largest of hold / load
    out["hold_at_peak"] = {}
    for sd in "LR":
        rs = [r for r in rows if r["hold"][sd] != 0.0]
        if rs:
            rp = max(rs, key=lambda r: abs(r["hold"][sd]))
            out["hold_at_peak"][sd] = (abs(rp["hold"][sd]), rp["ground"][sd][0], 1e3 * (rp["t"] - T))
    return out


CLEAT_M = (0.055, 0.084)      # m: a baseball shoe's measured twisting hold on basepath dirt over its load (Brosnan 2009: 68.5-100.1 N m at 1,190 N;
#                               Goodall 2005, second hand: about 24 N m at 441 N, 0.054 m); the model's forefoot gives 0.0575 m (4.11)


def held_against_cleat(out, BW):
    """for each held foot: over the last 150 ms before T, the share of the time the hold asks for more than a cleat's
    measured hold at the load the foot carries then (CLEAT_M's top), and the largest hold with that load."""
    res = {}
    T = out["T_ms"] * 1e-3
    for sd in "LR":
        rs = [r for r in out["rows"] if r["t"] >= T - 0.150 and r["hold"][sd] != 0.0]
        if not rs:
            continue
        over = sum(1 for r in rs if abs(r["hold"][sd]) > CLEAT_M[1] * r["ground"][sd][0])
        rp = max(rs, key=lambda r: abs(r["hold"][sd]))
        res[sd] = dict(share_over=over / len(rs), peak=abs(rp["hold"][sd]), load_at_peak=rp["ground"][sd][0] / BW,
                       cleat_at_peak=(CLEAT_M[0] * rp["ground"][sd][0], CLEAT_M[1] * rp["ground"][sd][0]),
                       mean_hold=sum(abs(r["hold"][sd]) for r in rs) / len(rs),
                       mean_cleat=CLEAT_M[1] * sum(r["ground"][sd][0] for r in rs) / len(rs))
    return res


def line(tag, o, BW):
    s_ = "%-5s pelvis %+6.1f deg (fastest %4.0f deg/s); thighs F %+6.1f R %+6.1f; feet F %+6.1f R %+6.1f | hips on an end from: F %s R %s; heaviest F %4.0f R %4.0f N m" % (
        tag, o["pelvis_turn"], o["pelvis_rate_peak"], o["turn"]["thigh_L"], o["turn"]["thigh_R"], o["turn"]["foot_L"], o["turn"]["foot_R"],
        "%+4.0f ms" % o["hip_end"]["L"][0] if o["hip_end"]["L"] else "never", "%+4.0f ms" % o["hip_end"]["R"][0] if o["hip_end"]["R"] else "never",
        o["hip_peak"]["L"], o["hip_peak"]["R"])
    s_ += "\n      load, mean of the last 100 ms: F %.2f R %.2f BW (at the end %.2f, %.2f); on one point: F %3.0f R %3.0f ms; off: F %3.0f R %3.0f ms | turning momentum at the end: upper %+6.2f legs %+6.2f (start %+5.2f %+5.2f); the ground and holds gave %+6.2f" % (
        o["load_last100"]["L"], o["load_last100"]["R"], o["load_at_T"]["L"], o["load_at_T"]["R"], o["few_points_ms"]["L"], o["few_points_ms"]["R"], o["off_ms"]["L"], o["off_ms"]["R"],
        o["H_upper"], o["H_legs"], o["H_upper0"], o["H_legs0"], o["given"])
    s_ += "\n      ground's twist, mean of the last 100 ms (N m): level forces F %+6.1f R %+6.1f; the feet's own twist F %+5.1f R %+5.1f | front knee %2.0f deg | bat (indicative): %s; nearest %.1f cm at %.1f m/s" % (
        o["lever_last100"]["L"], o["lever_last100"]["R"], o["free_last100"]["L"], o["free_last100"]["R"], o["knee_L"],
        "%.2f m/s, %.1f cm off, %.0f ms after the start" % (o["sweet_speed"], o["miss_cm"], o["own_contact_ms"]) if o["came"] else "never crosses the ball's plane",
        o["nearest_cm"], o["nearest_speed"])
    hc = held_against_cleat(o, BW)
    for sd, nm in (("L", "front"), ("R", "rear")):
        if sd in hc:
            x = hc[sd]
            s_ += "\n      the %s foot's hold over the last 150 ms: mean %.0f N m where a cleat's measured hold at its load averages %.0f; at its most %.0f N m with %.2f BW on the foot (a cleat there: %.0f-%.0f); over a cleat's hold %.0f%% of the time" % (
                nm, x["mean_hold"], x["mean_cleat"], x["peak"], x["load_at_peak"], x["cleat_at_peak"][0], x["cleat_at_peak"][1], 100 * x["share_over"])
    return s_


def series(o, BW, every_ms=10.0):
    T = o["T_ms"] * 1e-3
    out = ["   ms  | pelvis thighF thighR footF footR (deg from the start) | hip stops F, R (deg; N m) | load F, R (BW; points) | twist: level forces F R; own F R; holds F R (N m) | momentum upper, legs; given by the ground"]
    r0 = o["rows"][0]
    k = max(1, int(round(every_ms * 1e-3 / (o["rows"][1]["t"] - o["rows"][0]["t"]))))
    n = len(o["rows"]) - 1
    for j in range(n % k, n + 1, k):
        r = o["rows"][j]
        g = r["ground"]
        out.append("%6.0f | %+6.1f %+6.1f %+6.1f %+6.1f %+6.1f | %+5.1f %4.0f  %+5.1f %4.0f | %.2f(%d) %.2f(%d) | %+6.1f %+6.1f; %+5.1f %+5.1f; %+6.1f %+6.1f | %+6.2f %+6.2f; %+6.2f" % (
            (r["t"] - T) * 1e3, wrap(r["pelvis"] - r0["pelvis"]), wrap(r["thigh_L"] - r0["thigh_L"]), wrap(r["thigh_R"] - r0["thigh_R"]),
            wrap(r["foot_L"] - r0["foot_L"]), wrap(r["foot_R"] - r0["foot_R"]),
            r["hip_L"][1] / D2R, r["hip_L"][0], r["hip_R"][1] / D2R, r["hip_R"][0],
            g["L"][0] / BW, g["L"][3], g["R"][0] / BW, g["R"][3], g["L"][1], g["R"][1], g["L"][2], g["R"][2], r["hold"]["L"], r["hold"]["R"],
            r["H_upper"], r["H_legs"], r["given"]))
    return "\n".join(out)


def check():
    """the readings against worked cases, and the held foot in the compiled engine against the Python one."""
    ok = True
    # (1) the turning momentum of one body: a rod of mass 2 kg, I = (0.1, 0.2, 0.3) about its own axes, lying with its own
    #     z along the world's x, spinning at 5 rad/s about the vertical (its own -x ... worked below), its centre 0.5 m
    #     from the reference point along x and moving at 3 m/s along y: m r v = 2 * 0.5 * 3 = 3.0; its own part is the
    #     moment of inertia about the world's vertical, which for own-z along world-x and own-x along world -z is I_x = 0.1,
    #     times 5 = 0.5. Sum 3.5.
    s = E.System()
    b = s.body("rod", 2.0, (0.1, 0.2, 0.3))
    st = s.new_state()
    q = E.qaxis((0, 1, 0), math.pi / 2)                   # own z -> world x; own x -> world -z
    s.set_body(st, b, (0.5, 0.0, 0.0), q, (0.0, 3.0, 0.0), (0.0, 0.0, 5.0))
    got = turning_momentum(s, st, [b], (0.0, 0.0, 0.0), (0.0, 0.0, 0.0))
    print("turning momentum of a worked body: %.6f (3.5 by hand)" % got)
    ok = ok and abs(got - 3.5) < 1e-9
    # (2) the ground's twist in its two parts: two points of one foot at (0.1, +-0.05) carrying 100 N each with level forces
    #     (0, +30) and (0, -10): the centre of pressure is (0.1, 0); the net level force (0, 20) there has the moment
    #     0.1 * 20 = 2.0 about the origin; the foot's own twist about its centre is 0.05*0 ... worked: sum (dx fy - dy fx)
    #     with dx = 0 for both = 0; with level forces (+30, 0) and (-10, 0) instead: own twist = -(0.05*30) - (-0.05*-10)
    #     = -1.5 - 0.5 = -2.0, and the net (20, 0) at (0.1, 0) has the moment -(0)(20) = 0.
    class M_:
        pass
    m = M_()
    m.sys = E.System()
    fb = m.sys.body("foot", 1.0, (1.0, 1.0, 1.0))
    m.sys.contact("L_a", fb, (0.1, 0.05, 0.0), 1, 1, 1, 1, 1)
    m.sys.contact("L_b", fb, (0.1, -0.05, 0.0), 1, 1, 1, 1, 1)
    st = m.sys.new_state()
    g1 = ground_twist(m, st, [(0.0, 30.0, 100.0), (0.0, -10.0, 100.0)], (0.0, 0.0, 0.0))["L"]
    g2 = ground_twist(m, st, [(30.0, 0.0, 100.0), (-10.0, 0.0, 100.0)], (0.0, 0.0, 0.0))["L"]
    print("the ground's twist, worked: level force's moment %.3f (2.0), own %.3f (0.0); second case %.3f (0.0), own %.3f (-2.0)" % (g1[1], g1[2], g2[1], g2[2]))
    ok = ok and abs(g1[1] - 2.0) < 1e-12 and abs(g1[2]) < 1e-12 and abs(g2[1]) < 1e-12 and abs(g2[2] + 2.0) < 1e-12
    # (3) on rung 4's kept swing, unchanged and with both feet held: what the ground (and the holds) gave is the change of the
    #     whole body's turning momentum about its centre of mass (gravity has no moment about it), read independently
    d = json.load(open(os.path.join(HERE, "practice3d_s11_rung4.json")))
    for rp in ("0", "FR"):
        ctx, u = build(d, rp)
        o = read(ctx, u)
        dH = (o["H_upper"] + o["H_legs"]) - (o["H_upper0"] + o["H_legs0"])
        print("replay %-2s: the body's turning momentum changed by %+.3f kg m2/s; the ground's and holds' twist, summed over time, gave %+.3f (difference %.3f)" % (rp, dH, o["given"], dH - o["given"]))
        ok = ok and abs(dH - o["given"]) < 0.05
        if rp == "0":
            print("   the kept swing replays at %.2f m/s (stored %.2f)" % (o["sweet_speed"], d["sweet_speed"]))
            ok = ok and abs(o["sweet_speed"] - d["sweet_speed"]) < 0.005
    # (4) the held feet in the compiled engine against the Python engine: 40 ms of the FR replay, state for state
    st, aux = ctx.prepare(u)
    h, n = ctx.h, 160
    a2 = list(aux)
    ctx.m.sys.anchors(st, a2)
    s1 = list(st)
    for i in range(n):
        s1 = ctx.m.sys.step(PR.T0 + i * h, s1, a2, h)
    recs = P.run_raw(ctx.m.sys, st, PR.T0, n, h, n, list(aux), ctx.order)
    dd = max(abs(a - b_) for a, b_ in zip(recs[-1][1], s1))
    print("both feet held, 40 ms at 0.25 ms: the compiled engine against the Python one, largest state difference %.2e" % dd)
    ok = ok and dd < 1e-7
    print("PASS" if ok else "FAIL")
    return ok


def main():
    P.build()
    a = sys.argv[1:]
    if "--check" in a:
        sys.exit(0 if check() else 1)
    want_series = "--series" in a
    a = [x for x in a if x != "--series"]
    rung = None
    if "--rung" in a:                                     # the body by its name in ladder3d.RUNGS (a stored result whose own options do not say all of it)
        rung = a[a.index("--rung") + 1]
        del a[a.index("--rung"):a.index("--rung") + 2]
    path = a[0]
    reps = a[1:] or list(REPLAYS)
    d = json.load(open(path))
    if rung:
        import ladder3d as L
        d["options"] = L.options(rung)
    ctx, u = build(d, "0")
    base = read(ctx, u)
    BW = ctx.BW
    print("%s: stored %.2f m/s; replayed %.2f m/s, contact %.1f ms after the start; the hips' ends of turn (deg): %s" % (
        os.path.basename(path), d["sweet_speed"], base["sweet_speed"], base["T_ms"], {k: tuple(round(x, 1) for x in v) for k, v in base["limits"].items()}))
    for rp in reps:
        if rp == "0":
            o = base
        else:
            ctx, u = build(d, rp)
            try:
                o = read(ctx, u, base["T_ms"] * 1e-3)
            except E.SolveError as e:
                print("%-5s THE RUN FAILS: %s" % (rp, e))
                continue
        print(line(rp, o, BW))
        if want_series:
            print(series(o, BW))


if __name__ == "__main__":
    main()
