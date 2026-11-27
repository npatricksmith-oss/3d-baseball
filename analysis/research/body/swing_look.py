"""The practised swing looked at position by position (docs/body-motion.md 4.15, "What the swing looks like").

    python3 analysis/research/body/swing_look.py <plan.json> [--step 10] [--to 60]            the table, every `step` ms
    uv run --with matplotlib python3 analysis/research/body/swing_look.py <plan.json> --draw <dir>   and drawings

A LOOKING AID, READ BY EYE; IT GATES NOTHING AND NOTHING HERE IS GIVEN TO THE SEARCH. The table is read off the model's
own states (one run of the plan, as practice3d's report runs it); the drawings are the model's joints, bat, sole points
and ball as lines, from three fixed sides, six instants a sheet: from the pitcher's side (first base to the left),
from the open side (the side his chest faces; the pitcher to the right) and from above (the pitcher at the top). The
lead (left) arm and leg are blue, the rear red, the trunk and pelvis black, the bat brown, the ball orange; under each
foot is the share of his weight it carries. The head is drawn where the trunk carries it (the model has no neck).
Times are ms from contact.
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


def unwrap(prev, x):
    while x - prev > 180.0:
        x -= 360.0
    while x - prev < -180.0:
        x += 360.0
    return x


def read(ctx, u, step_ms=10.0, to_ms=60.0):
    """one row a `step_ms` from the run's start to `to_ms` after contact: a dict of what a baseball man would look at."""
    m, s = ctx.m, ctx.m.sys
    recs = ctx.run(u, every=int(round(1e-3 / ctx.h)))                 # 1 ms records
    tx = ctx.contact(recs)[0]
    at = lambda t: min(range(len(recs)), key=lambda k: abs(recs[k][0] - t))
    k0 = math.ceil((recs[0][0] - tx) * 1e3 / step_ms)
    times = [k * step_ms for k in range(k0, int(to_ms / step_ms) + 1)]
    rows, first = [], None
    yaw = {"pelvis": None, "trunk": None, "foot_L": None, "foot_R": None, "bat": None}
    for i, rec in enumerate(recs):                                    # the turns followed through every record, so that none wraps
        o = m.observe(rec[1])
        cur = {"pelvis": o["pelvis_rot"], "trunk": o["trunk_rot"], "bat": o["bat_az"]}
        for sd in "LR":
            fx = m.axis(rec[1], "foot_" + sd, (1, 0, 0))
            cur["foot_" + sd] = math.atan2(fx[1], fx[0]) / D2R
        for k_, v in cur.items():
            yaw[k_] = v if yaw[k_] is None else unwrap(yaw[k_], v)
        rec_yaw = dict(yaw)
        if first is None:
            first = dict(yaw)
        t_ms = (rec[0] - tx) * 1e3
        if not any(abs(t_ms - t) < 0.5 for t in times):
            continue
        st = rec[1]
        jp = lambda n: m.joint_pos(st, n)
        zt = m.axis(st, "trunk", (0, 0, 1))
        com, vcom = B.com(m, st)
        aL, aR = jp("ankle_L"), jp("ankle_R")
        fz = {"L": 0.0, "R": 0.0}
        for cn, f in zip(s.contacts, rec[2]):
            fz[cn[0][0]] += f[2] / ctx.BW
        heel = {}
        for sd in "LR":
            fb = m.b["foot_" + sd]
            xb, qb, _, _ = s.pose(st, fb)
            pts = {c_[0]: E.add(xb, E.mv(E.qR(qb), c_[2]))[2] for c_ in s.contacts if c_[0].startswith(sd)}
            heel[sd] = (min(v for k_, v in pts.items() if "heel" in k_), min(v for k_, v in pts.items() if "ball" in k_))
        hands = m.bat_point(st, 0.5 * (B.GRIP_L + B.GRIP_R))
        shR, shL = jp("shoulder_R"), jp("shoulder_L")
        hipm = tuple(0.5 * (a + b) for a, b in zip(jp("hip_L"), jp("hip_R")))
        shm = tuple(0.5 * (a + b) for a, b in zip(shL, shR))
        fwd = (math.cos(first["pelvis"] * D2R), math.sin(first["pelvis"] * D2R), 0.0)      # the way his belt buckle faced at the start (toward the plate)
        bz = m.axis(st, "bat", (0, 0, 1))
        ss, vss = m.bat_point(st, C.BAT_SS)
        knob, _ = m.bat_point(st, 0.0)
        rows.append(dict(
            t=t_ms, pelvis=rec_yaw["pelvis"] - first["pelvis"], trunk=rec_yaw["trunk"] - first["trunk"], pelvis_rate=o["pelvis_rot_vel"], trunk_rate=o["trunk_rot_vel"],
            # the trunk's lean from upright, and its parts: toward the plate (the way he faced at the start) and toward the pitcher
            lean=math.acos(max(-1.0, min(1.0, zt[2]))) / D2R, lean_plate=math.asin(max(-1.0, min(1.0, E.dot(zt, fwd)))) / D2R, lean_pitcher=math.asin(max(-1.0, min(1.0, zt[1]))) / D2R,
            com_z=com[2], com_share=(com[1] - aR[1]) / (aL[1] - aR[1]), com_vy=vcom[1], com_vz=vcom[2], hips_z=hipm[2],
            hips_share=(hipm[1] - aR[1]) / (aL[1] - aR[1]), head_share=(shm[1] + 0.25 * zt[1] - aR[1]) / (aL[1] - aR[1]),
            front=fz["L"], rear=fz["R"], knee_L=o["lead_knee_flex"], knee_R=o["trail_knee_flex"],
            heel_L=heel["L"][0] * 100, heel_R=heel["R"][0] * 100, ball_R=heel["R"][1] * 100, ball_L=heel["L"][1] * 100,
            foot_L=rec_yaw["foot_L"] - first["foot_L"], foot_R=rec_yaw["foot_R"] - first["foot_R"],
            hands_speed=E.norm(hands[1]), hands_behind=shR[1] - hands[0][1], hands_z=hands[0][2], hands_below_shoulder=shR[2] - hands[0][2],
            elbow_L=o["lead_elbow_flex"], elbow_R=o["trail_elbow_flex"], bat_down=-o["bat_elev"], bat_az=rec_yaw["bat"], sweet=E.norm(vss), sweet_z=ss[2], knob_z=knob[2],
            attack=o["attack"], ball_dist=E.norm(E.sub(ss, ctx.Pc)), stride=aL[1] - aR[1]))
    return rows, tx, recs


def table(rows, log=print):
    log("t ms | belt buckle; chest turned (deg from the start) | their rates deg/s | trunk's lean from upright (toward the plate; toward the pitcher) | hips' height; centre of mass's height cm (its speed down m/s) | weight on front; rear foot (x his weight) | centre of mass, hips, head between the feet (0 rear ankle, 1 front) | knees front; rear | heels front; rear cm up | feet turned front; rear | hands: speed m/s, behind the rear shoulder cm, below it cm | elbows lead; rear | barrel below level deg | sweet spot m/s, its height cm, rising deg")
    for r in rows:
        log("%+5.0f | %+4.0f %+4.0f | %4.0f %4.0f | %2.0f (%+3.0f; %+3.0f) | %3.0f; %3.0f (%+.2f) | %.2f; %.2f | %.2f %.2f %.2f | %3.0f; %3.0f | %4.1f; %4.1f | %+3.0f; %+3.0f | %4.1f, %+4.0f, %+4.0f | %3.0f; %3.0f | %+4.0f | %4.1f, %3.0f, %+4.0f"
            % (r["t"], r["pelvis"], r["trunk"], r["pelvis_rate"], r["trunk_rate"], r["lean"], r["lean_plate"], r["lean_pitcher"], r["hips_z"] * 100, r["com_z"] * 100, -r["com_vz"],
               r["front"], r["rear"], r["com_share"], r["hips_share"], r["head_share"], r["knee_L"], r["knee_R"], r["heel_L"], r["heel_R"], r["foot_L"], r["foot_R"],
               r["hands_speed"], r["hands_behind"] * 100, r["hands_below_shoulder"] * 100, r["elbow_L"], r["elbow_R"], r["bat_down"], r["sweet"], r["sweet_z"] * 100, r["attack"]))


def draw(ctx, u, rows, tx, recs, out_dir, per_sheet=6, tag="swing"):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    m, s = ctx.m, ctx.m.sys
    os.makedirs(out_dir, exist_ok=True)
    at = lambda t_ms: min(range(len(recs)), key=lambda k: abs((recs[k][0] - tx) * 1e3 - t_ms))
    views = (("from the pitcher's side", lambda p: (-p[0], p[2])), ("from the open side (pitcher to the right)", lambda p: (p[1], p[2])), ("from above (pitcher at the top)", lambda p: (p[0], p[1])))
    xp0 = s.pose(recs[0][1], m.b["pelvis"])[0]
    paths = []
    for sheet in range(0, len(rows), per_sheet):
        part = rows[sheet:sheet + per_sheet]
        fig, axs = plt.subplots(3, per_sheet, figsize=(3.0 * per_sheet, 9.6), dpi=100)
        for c, r in enumerate(part):
            st = recs[at(r["t"])][1]
            con = recs[at(r["t"])][2]
            jp = lambda n: m.joint_pos(st, n)
            zt = m.axis(st, "trunk", (0, 0, 1))
            shm = tuple(0.5 * (a + b) for a, b in zip(jp("shoulder_L"), jp("shoulder_R")))
            head = E.add(shm, E.scale(zt, 0.20))
            knob, tip = m.bat_point(st, 0.0)[0], m.bat_point(st, C.BAT_L)[0]
            ball = E.add(ctx.Pc, E.scale(ctx.nb, PR.BALL_SPEED * (r["t"] * 1e-3)))
            feet = {}
            for sd in "LR":
                fb = m.b["foot_" + sd]
                xb, qb, _, _ = s.pose(st, fb)
                feet[sd] = {c_[0][2:]: E.add(xb, E.mv(E.qR(qb), c_[2])) for c_ in s.contacts if c_[0].startswith(sd)}
            for v, (vname, pr) in enumerate(views):
                ax = axs[v][c]
                line = lambda a, b_, col, lw=2.5, z=3: ax.plot(*zip(pr(a), pr(b_)), color=col, lw=lw, solid_capstyle="round", zorder=z)
                if v < 2:
                    ax.axhline(0.0, color="#8a6", lw=1)
                for sd, col in (("R", "#c33"), ("L", "#25c")):
                    line(jp("hip_" + sd), jp("knee_" + sd), col); line(jp("knee_" + sd), jp("ankle_" + sd), col)
                    f = feet[sd]
                    heel = tuple(0.5 * (a + b) for a, b in zip(f["heel_in"], f["heel_out"]))
                    toe = tuple(0.5 * (a + b) for a, b in zip(f["ball_in"], f["ball_out"]))
                    line(jp("ankle_" + sd), heel, col, 2); line(heel, toe, col, 3); line(jp("ankle_" + sd), toe, col, 1.5)
                    line(jp("shoulder_" + sd), jp("elbow_" + sd), col, 2.2, 5 if sd == "L" else 2); line(jp("elbow_" + sd), jp("wrist_" + sd), col, 2.2, 5 if sd == "L" else 2)
                line(jp("hip_L"), jp("hip_R"), "k", 3); line(jp("shoulder_L"), jp("shoulder_R"), "k", 3)
                hipm = tuple(0.5 * (a + b) for a, b in zip(jp("hip_L"), jp("hip_R")))
                line(hipm, jp("lumbar"), "k", 3); line(jp("lumbar"), shm, "k", 3); line(shm, head, "k", 2)
                ax.add_patch(plt.Circle(pr(head), 0.10, fill=False, color="k", lw=1.5, zorder=4))
                line(knob, tip, "#850", 4, 6)
                if r["t"] <= 0.5 and abs(ball[1] - xp0[1]) < 2.2:
                    ax.add_patch(plt.Circle(pr(ball), 0.0366, color="#f80", zorder=7))
                ax.plot(*pr(ctx.Pc), marker="+", color="#f80", ms=8, zorder=1)
                ax.set_aspect("equal")
                if v == 0:
                    ax.set_xlim(-xp0[0] - 1.25, -xp0[0] + 1.0); ax.set_ylim(-0.1, 2.15)
                    ax.set_title("%+.0f ms   bat %.0f mph" % (r["t"], r["sweet"] / 0.44704), fontsize=10)
                    ax.text(0.02, 0.02, "front %.2f  rear %.2f" % (r["front"], r["rear"]), transform=ax.transAxes, fontsize=8)
                elif v == 1:
                    ax.set_xlim(xp0[1] - 1.1, xp0[1] + 1.25); ax.set_ylim(-0.1, 2.15)
                    ax.text(0.02, 0.02, "barrel %.0f below level; lean %.0f" % (r["bat_down"], r["lean"]), transform=ax.transAxes, fontsize=8)
                else:
                    ax.set_xlim(xp0[0] - 0.9, xp0[0] + 1.5); ax.set_ylim(xp0[1] - 1.1, xp0[1] + 1.3)
                    ax.text(0.02, 0.02, "buckle %+.0f  chest %+.0f" % (r["pelvis"], r["trunk"]), transform=ax.transAxes, fontsize=8)
                ax.set_xticks([]); ax.set_yticks([])
                if c == 0:
                    ax.set_ylabel(vname, fontsize=8)
        for c in range(len(part), per_sheet):
            for v in range(3):
                axs[v][c].axis("off")
        fig.tight_layout(pad=0.3)
        path = os.path.join(out_dir, "%s_%02d_%+04.0f_to_%+04.0f_ms.png" % (tag, sheet // per_sheet + 1, part[0]["t"], part[-1]["t"]))
        fig.savefig(path)
        plt.close(fig)
        paths.append(path)
    return paths


def main():
    a = sys.argv[1:]
    arg = lambda k, d: a[a.index(k) + 1] if k in a else d
    if not a or a[0].startswith("-"):
        print(__doc__)
        return
    P.build()
    d = json.load(open(a[0]))
    ctx = PR.Ctx(**d["options"])
    u = ctx.u_of(d["plan"])
    rows, tx, recs = read(ctx, u, float(arg("--step", 10)), float(arg("--to", 60)))
    print("%s: the sweet spot at %.2f m/s (%.1f mph) at contact, %.0f ms after the run's start" % (os.path.basename(a[0]), d.get("sweet_speed", float("nan")), d.get("sweet_speed", float("nan")) / 0.44704, (tx - PR.T0) * 1e3))
    table(rows)
    if "--draw" in a:
        for p_ in draw(ctx, u, rows, tx, recs, arg("--draw", "."), tag=os.path.basename(a[0])[:-5]):
            print("drew " + p_)


if __name__ == "__main__":
    main()
