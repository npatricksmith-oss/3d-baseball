"""The force-driven body's practised swing written out for the Unreal viewer (PLAN D43; docs/player-visuals.md 14).

    python3 analysis/research/body/swing_export.py                        the newest kept swing of this tree
    python3 analysis/research/body/swing_export.py <plan.json>            that kept swing
    python3 analysis/research/body/swing_export.py <plan.json> --tree <dir> [--out <file>]
        the swing run on the body code of another checkout <dir> (read only: its practice3d.py, probe3d.py and
        body3d are imported from there, its probe's source is compiled into THIS tree's sim/target); the file is
        written into this tree

Runs the kept plan forward once on the checking model, exactly as practice3d.py's report runs it (its Ctx, its
options, its integration step), and writes every rigid body's place and turn, every joint, the bat's knob, sweet
spot and tip, each foot's contact points and the ball's line, every millisecond, into one text file
(analysis/research/body/swings/<plan>.swing). Nothing is smoothed, fitted or made up: what the model has is written,
what it does not have is not.

THE FILE IS A VIEWING AID. It is not the timeline's pose format of body-motion 9 (phase 2 sets that), and no game
reads it. Its layout is in the file's own header and in docs/player-visuals.md 14.
"""
import glob
import math
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
OUT_DIR = os.path.join(HERE, "swings")
RATE_HZ = 1000.0               # samples a second in the file: linear interpolation keeps the bat's tip within 1 cm (measured, below)
TIP_TOL = 0.01                 # m: the most the tip, interpolated between samples, may be off the computed one
MPH = 0.44704
# The model has no plate: its coordinates have the sim frame's axes, the ground at z = 0, and the batter's feet about its
# origin. The file is moved into the sim frame by one translation in x and y: the meeting point put over the middle of the
# plate (x = 0) at its front edge (y = 17 in from the rear apex) [E: a viewing choice; nothing in the model sets it].
PLATE_FRONT_Y = 17 * 0.0254
FORMAT = "bb-body-swing 1"
BODY_ORDER = ("pelvis", "trunk", "thigh_L", "shank_L", "foot_L", "thigh_R", "shank_R", "foot_R",
              "uarm_L", "farm_L", "uarm_R", "farm_R", "bat")
JOINT_ORDER = ("lumbar", "hip_L", "knee_L", "ankle_L", "hip_R", "knee_R", "ankle_R",
               "shoulder_L", "elbow_L", "wrist_L", "shoulder_R", "elbow_R", "wrist_R")
# the bat's points by their distance from the knob (m); grip_L and grip_R are on the handle's axis under each hand
POINTS = ("knob", "sweet", "tip", "grip_L", "grip_R")


def newest_plan(tree):
    """the newest kept swing of a tree: the practice3d_s<N>_*.json with the highest session, then the newest file."""
    best = None
    for p in glob.glob(os.path.join(tree, "analysis", "research", "body", "practice3d_s*_*.json")):
        b = os.path.basename(p)
        try:
            n = int(b[len("practice3d_s"):].split("_")[0])
        except ValueError:
            continue
        key = (n, os.path.getmtime(p))
        if best is None or key > best[0]:
            best = (key, p)
    return best[1] if best else None


def load_tree(tree):
    """practice3d (and through it probe3d, body3d) imported from a tree's own files; the probe compiled into this
    tree's sim/target (a directory of its own for each tree), so the other tree is only read."""
    if os.path.realpath(tree) != os.path.realpath(ROOT):
        sys.dont_write_bytecode = True                        # no __pycache__ written into the other tree
    sys.path.insert(0, os.path.join(tree, "analysis", "research", "body"))
    if "BB_FORT_CSV" not in os.environ:
        # [4]'s digitised curves (common.py reads them from a path relative to the working directory): this tree's
        # copy, else the other tree's (read only). They are made by analysis/research/fortenbaugh_appendix_e.py.
        for base in (ROOT, tree):
            csv = os.path.join(base, "sim", "target", "research-dl", "4", "csv")
            if os.path.isdir(csv):
                os.environ["BB_FORT_CSV"] = csv
                break
    import practice3d as Pr                                    # noqa: E402
    P = Pr.P
    here = os.path.realpath(tree) == os.path.realpath(ROOT)
    P.OUT = os.path.join(ROOT, "sim", "target", "codex-probes", "body3d" if here else "body3d-" + os.path.basename(os.path.realpath(tree)))
    P.BIN = os.path.join(P.OUT, "body3d_probe")
    for mod in ("practice3d", "probe3d", "body3d.body", "body3d.engine"):
        f = getattr(sys.modules[mod], "__file__", "")
        if not os.path.realpath(f).startswith(os.path.realpath(tree) + os.sep):
            raise SystemExit("%s was imported from %s, not from the tree %s" % (mod, f, tree))
    P.build()
    return Pr


def git(tree, *a):
    try:
        return subprocess.run(["git", "-C", tree] + list(a), capture_output=True, text=True, check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def lerp(a, b, q):
    return tuple(x + q * (y - x) for x, y in zip(a, b))


def export(plan_path, tree=ROOT, out=None, log=print):
    import json
    tree = os.path.abspath(tree)
    Pr = load_tree(tree)
    E, B, C = Pr.E, Pr.B, Pr.C
    d = json.load(open(plan_path))
    ctx = Pr.Ctx(**d.get("options", {}))
    m, s = ctx.m, ctx.m.sys
    u = ctx.u_of(d["plan"])
    h = ctx.h
    every = int(round(1.0 / (RATE_HZ * h)))
    if abs(every * h * RATE_HZ - 1.0) > 1e-9:
        raise SystemExit("the integration step %.6g s does not divide the file's sample interval" % h)
    # one run, every step recorded; the file keeps every `every`-th record, which are the records report() reads
    fine = ctx.run(u, every=1)
    recs = fine[::every]
    tx, p, v, i, came = ctx.contact(recs)
    if not came:
        raise SystemExit("the bat never reaches the ball's plane in this run: no contact to show")
    failed = ctx.failed_after
    bix = [m.b[n] for n in BODY_ORDER]
    us = {"knob": 0.0, "sweet": C.BAT_SS, "tip": C.BAT_L, "grip_L": B.GRIP_L, "grip_R": B.GRIP_R}
    ball_v = E.scale(ctx.nb, Pr.BALL_SPEED)
    ball_at = lambda t: E.add(ctx.Pc, E.scale(ball_v, t - tx))
    shift = (-ctx.Pc[0], PLATE_FRONT_Y - ctx.Pc[1], 0.0)
    mv = lambda x: [x[0] + shift[0], x[1] + shift[1], x[2] + shift[2]]

    def sample(rec):
        t, st, con = rec[0], rec[1], rec[2]
        row = [t - tx]
        for b in bix:
            x, q, _, _ = s.pose(st, b)
            row += mv(x) + list(q)
        gap = 0.0
        for j in JOINT_ORDER:
            a, ra, bb, rb = m.jp[j]
            pb = m.joint_pos(st, j)
            xa, qa, _, _ = s.pose(st, a)
            gap = max(gap, E.norm(E.sub(pb, E.add(xa, E.mv(E.qR(qa), ra)))))
            row += mv(pb)
        for k in POINTS:
            row += mv(m.bat_point(st, us[k])[0])
        row.append(E.norm(m.bat_point(st, C.BAT_SS)[1]))
        for (name, b, pl, *_), f in zip(s.contacts, con):
            xb, qb, _, _ = s.pose(st, b)
            row += mv(E.add(xb, E.mv(E.qR(qb), pl))) + [f[2]]
        row += mv(ball_at(t))
        return row, gap

    rows, gap = [], 0.0
    for rec in recs:
        r, g = sample(rec)
        rows.append(r)
        gap = max(gap, g)

    # how far the tip is off the computed one when drawn between samples: the fine records between each pair of
    # samples against (a) the tip's place interpolated straight and (b) the bat's place lerped and its turn slerped,
    # the tip then carried on it (the way the viewer draws the bat)
    def slerp(q0, q1, a):
        dt = sum(x * y for x, y in zip(q0, q1))
        if dt < 0.0:
            q1, dt = tuple(-x for x in q1), -dt
        if dt > 0.9995:
            q = lerp(q0, q1, a)
        else:
            th = math.acos(dt)
            q = tuple((math.sin((1 - a) * th) * x + math.sin(a * th) * y) / math.sin(th) for x, y in zip(q0, q1))
        n = math.sqrt(sum(x * x for x in q))
        return tuple(x / n for x in q)
    ib = m.b["bat"]
    tip_err = [0.0, 0.0, 0.0]          # straight, carried on the bat, the time of the larger
    for k in range(len(recs) - 1):
        s0, s1 = recs[k][1], recs[k + 1][1]
        t0p, t1p = m.bat_point(s0, C.BAT_L)[0], m.bat_point(s1, C.BAT_L)[0]
        x0, q0, _, _ = s.pose(s0, ib)
        x1, q1, _, _ = s.pose(s1, ib)
        for j in range(1, every):
            fr = fine[k * every + j]
            truth = m.bat_point(fr[1], C.BAT_L)[0]
            a = j / every
            e1 = E.norm(E.sub(lerp(t0p, t1p, a), truth))
            xb, qb = lerp(x0, x1, a), slerp(q0, q1, a)
            e2 = E.norm(E.sub(E.add(xb, E.mv(E.qR(qb), (0.0, 0.0, C.BAT_L - m.bat_ucm))), truth))
            if max(e1, e2) > max(tip_err[0], tip_err[1]):
                tip_err[2] = fr[0] - tx
            tip_err[0], tip_err[1] = max(tip_err[0], e1), max(tip_err[1], e2)
    if max(tip_err[:2]) > TIP_TOL:
        raise SystemExit("the tip interpolated between samples is %.1f mm off the computed one: raise RATE_HZ" % (1e3 * max(tip_err[:2])))

    # the events the model has
    ts = [r[0] for r in rows]
    hs = [E.norm(m.bat_point(rec[1], Pr.HANDS_U)[1]) for rec in recs]
    pre = [q for q in range(len(recs)) if recs[q][0] <= tx + 1e-9]
    kpk = max(pre, key=lambda q: hs[q])
    k = kpk
    while k > 0 and hs[k] > 0.1 * hs[kpk]:
        k -= 1
    heel = [q for q, (name, *_r) in enumerate(s.contacts) if name.startswith("L_heel")]
    heel_down = None
    if not any(recs[0][2][q][2] > 0.0 for q in heel):
        for rec in recs:
            if any(rec[2][q][2] > 0.0 for q in heel):
                heel_down = rec[0] - tx
                break
    events = [("start", ts[0], "the run's first instant: the posture placed at [4]'s -250 ms (body.place), already moving"),
              ("launch", recs[k][0] - tx, "the hands start: their speed (%.2f m from the knob) last rises through a tenth of its fastest before contact (practice3d.report's swing-time rule)" % Pr.HANDS_U)]
    if heel_down is not None:
        events.append(("front_foot_down", heel_down, "the lead heel first carries load (the 3D foot's heel plant, body-motion 5.4)"))
    events += [("contact", 0.0, "the sweet spot crosses the plane square to the ball's line at the meeting point (practice3d Ctx.contact)"),
               ("end", ts[-1], "the integration's last record" + ("" if failed is None else "; THE RUN FAILED THERE: " + failed[1][:120]))]

    miss = E.sub(p, ctx.Pc)
    miss_up, miss_along = E.dot(miss, ctx.ev), E.dot(miss, ctx.el)
    sp = E.norm(v)
    name = os.path.splitext(os.path.basename(plan_path))[0]
    if out is None:
        os.makedirs(OUT_DIR, exist_ok=True)
        out = os.path.join(OUT_DIR, name + ".swing")
    rel = lambda path: os.path.relpath(os.path.abspath(path), tree) if os.path.abspath(path).startswith(tree + os.sep) else os.path.abspath(path)
    commit = git(tree, "rev-parse", "--short", "HEAD")
    dirty = git(tree, "status", "--porcelain", "--", os.path.abspath(plan_path))
    nb_, nc_ = len(BODY_ORDER), len(s.contacts)
    w = []
    w.append("# %s" % FORMAT)
    w.append("# The force-driven body's practised swing, run forward once on the checking model (analysis/oracle/body3d)")
    w.append("# by analysis/research/body/swing_export.py. A VIEWING AID for the Unreal viewer (client/run.sh -BbBodySwing=<file>,")
    w.append("# PLAN D43): NOT the timeline's pose format of body-motion 9, and no game reads it. Layout: docs/player-visuals.md 14.")
    w.append("# Sim frame (Statcast): metres, +X toward first base, +Y toward the pitcher, +Z up; a right-handed batter on the")
    w.append("# third-base side (-X) facing +X. L is his lead (left) side, R his rear. Quaternions (w, x, y, z): world = R(q) local.")
    w.append("# The model has no plate: its coordinates are moved by `placed` (x, y only) so that its meeting point is over the")
    w.append("# middle of the plate at the plate's front edge, a viewing choice. Every position below is moved; `contact` gives both.")
    w.append("# Times are seconds from contact. Each body's local frame: pelvis and trunk x forward, y to his left, z up at the")
    w.append("# start; the limbs' z runs from the far joint toward the near one; the foot's x heel to toe; the bat's z knob to tip,")
    w.append("# its origin the bat-and-hands' centre of mass. The model has no neck, fingers or face: the head is part of the trunk.")
    w.append("# The ball is the model's: a straight line through the meeting point at the pitch's speed and angles there (the")
    w.append("# practice aims at that line only; no flight before it is computed). After contact it carries on along it.")
    w.append("format %s" % FORMAT)
    w.append("plan %s" % rel(plan_path))
    w.append("tree %s" % tree)
    w.append("commit %s%s" % (commit, " (the plan file has uncommitted changes)" if dirty else ""))
    w.append("plan_rung %s" % d.get("rung", "-"))
    w.append("options %s" % json.dumps(d.get("options", {}), sort_keys=True))
    w.append("batter_height_m %.4f" % m.H)
    w.append("batter_mass_kg %.3f" % m.M)
    w.append("bat length_m %.4f mass_kg %.4f com_from_knob_m %.4f i_cm_kgm2 %.5f i_axis_kgm2 %.6g sweet_from_knob_m %.4f" % (C.BAT_L, C.BAT_M, C.BAT_UCM, C.BAT_ICM, C.BAT_IAX, C.BAT_SS))
    w.append("bat_body com_from_knob_m %.6f mass_kg %.4f    # the bat with both hands on it: one rigid body" % (m.bat_ucm, s.m[m.b["bat"]]))
    w.append("hands grip_L_from_knob_m %.3f grip_R_from_knob_m %.3f wrist_to_grip_m %.3f" % (B.GRIP_L, B.GRIP_R, B.WRIST_TO_GRIP))
    w.append("contact_speed_mps %.4f" % sp)
    w.append("contact_speed_mph %.2f" % (sp / MPH))
    w.append("contact_speed_along_mps %.4f" % -E.dot(v, ctx.nb))
    w.append("plan_file_speed_mps %s" % ("%.4f" % d["sweet_speed"] if "sweet_speed" in d else "-"))
    w.append("miss_cm up %.3f along_bat %.3f total %.3f" % (100 * miss_up, 100 * miss_along, 100 * math.hypot(miss_up, miss_along)))
    w.append("plan_file_miss_cm %s" % (" ".join("%.3f" % x for x in d["miss_cm"]) if "miss_cm" in d else "-"))
    w.append("placed %.5f %.5f %.5f    # added to the model's coordinates (m): the meeting point to x 0, y %.4f" % (shift + (PLATE_FRONT_Y,)))
    w.append("contact t_s 0 model_clock_s %.6f sweet_at %.5f %.5f %.5f sweet_vel %.4f %.4f %.4f ball_vel %.4f %.4f %.4f meet_point %.5f %.5f %.5f    # positions in the file's frame (placed)"
             % ((tx,) + tuple(mv(p)) + tuple(v) + tuple(ball_v) + tuple(mv(ctx.Pc))))
    w.append("ball speed_mps %.3f descent_deg %.3f across_deg %.3f radius_m 0.0366" % (Pr.BALL_SPEED, Pr.BALL_DESCENT, Pr.BALL_ACROSS))
    for nm, t, why in events:
        w.append("event %s %.6f    # %s" % (nm, t, why))
    if heel_down is None:
        w.append("# no front_foot_down event: the lead heel already carries load at the run's first instant")
    w.append("integration step_s %.6g every %d    # the file keeps every %d-th step" % (h, every, every))
    w.append("rate_hz %.0f" % RATE_HZ)
    w.append("tip_interp_err_m straight %.6f carried_on_bat %.6f at_t %.4f    # the bat's tip between samples, against the steps not kept" % tuple(tip_err))
    w.append("joint_gap_m %.3g    # the largest distance between a joint's two halves in the file (the engine's constraint)" % gap)
    w.append("bodies %d %s" % (nb_, " ".join(BODY_ORDER)))
    w.append("joints %d %s" % (len(JOINT_ORDER), " ".join(JOINT_ORDER)))
    w.append("points %d %s" % (len(POINTS), " ".join(POINTS)))
    w.append("contacts %d %s" % (nc_, " ".join(c[0] for c in s.contacts)))
    w.append("# each sample row: s t | bodies x y z qw qx qy qz | joints x y z | points x y z | sweet_speed_mps | contacts x y z fz_N | ball x y z")
    w.append("columns %d" % (1 + 7 * nb_ + 3 * len(JOINT_ORDER) + 3 * len(POINTS) + 1 + 4 * nc_ + 3))
    w.append("samples %d" % len(rows))
    for r in rows:
        w.append("s " + " ".join("%.7g" % x if abs(x) >= 1e-3 or x == 0.0 else "%.6e" % x for x in r))
    with open(out, "w") as fh:
        fh.write("\n".join(w) + "\n")
    log("wrote %s: %d samples, %.0f ms before contact to %.0f ms after; the sweet spot at contact %.2f m/s (%.1f mph), the plan's file says %s;"
        " missed by %.2f cm; the tip between samples within %.2f mm; joints' halves within %.2g m"
        % (rel(out), len(rows), -1e3 * ts[0], 1e3 * ts[-1], sp, sp / MPH, d.get("sweet_speed", "-"), 100 * math.hypot(miss_up, miss_along),
           1e3 * max(tip_err[:2]), gap))
    return out


def main():
    a = sys.argv[1:]
    arg = lambda k, dflt: a[a.index(k) + 1] if k in a else dflt
    tree = os.path.abspath(arg("--tree", ROOT))
    pos = [x for j, x in enumerate(a) if not x.startswith("--") and (j == 0 or a[j - 1] not in ("--tree", "--out"))]
    plan = pos[0] if pos else newest_plan(tree)
    if plan is None:
        raise SystemExit("no kept swing (practice3d_s*_*.json) in %s" % tree)
    if not os.path.isabs(plan) and not os.path.exists(plan):
        plan = os.path.join(tree, plan)
    export(plan, tree, arg("--out", None))


if __name__ == "__main__":
    main()
