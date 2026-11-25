"""Runs the compiled copy of the 3D body engine (body3d_probe.rs) and checks it against the Python one.

    python3 analysis/research/body/probe3d.py --check      state for state against analysis/oracle/body3d (50 s)
    python3 analysis/research/body/probe3d.py --bench      microseconds per RK4 step, dense and tree-ordered

The Python engine is the checking model (docs/body-motion.md 8.1); the probe exists so fits can run
(8.2). `simulate` here returns what body3d.drive.simulate returns, from the probe.
"""
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..", "..", "..")
sys.path.insert(0, os.path.join(HERE, "..", "..", "oracle"))
from body3d import engine as E, body as B, drive as Dr   # noqa: E402

OUT = os.path.join(ROOT, "sim", "target", "codex-probes", "body3d")
BIN = os.path.join(OUT, "body3d_probe")
SRC = os.path.join(HERE, "body3d_probe.rs")
# joints in an order that follows the body's tree, so each row's leading zeros can be skipped
TREE_ORDER = (("shank_L", "foot_L"), ("thigh_L", "shank_L"), ("pelvis", "thigh_L"),
              ("shank_R", "foot_R"), ("thigh_R", "shank_R"), ("pelvis", "thigh_R"),
              ("farm_L", "bat"), ("uarm_L", "farm_L"), ("trunk", "uarm_L"),
              ("farm_R", "bat"), ("uarm_R", "farm_R"), ("trunk", "uarm_R"), ("pelvis", "trunk"))
CHECK_TOL = 1e-9          # largest allowed difference of any state entry (m, quaternion part, m/s, rad/s) in --check


def build(force=False):
    os.makedirs(OUT, exist_ok=True)
    if force or not os.path.isfile(BIN) or os.path.getmtime(BIN) < os.path.getmtime(SRC):
        subprocess.run(["rustc", "-C", "opt-level=3", "-C", "linker=" + os.path.expanduser("~/.local/bin/zigcc"), "-o", BIN, SRC],
                       check=True, env=dict(os.environ, PATH=os.path.expanduser("~/.cargo/bin") + ":" + os.environ["PATH"]))


def tree_order(mdl):
    """the rows of engine._rows (points first, then axis pairs) regrouped joint by joint in TREE_ORDER."""
    s, b = mdl.sys, mdl.b
    pairs = [(p[0], p[1]) for p in s.pts for _ in range(3)] + [(d[0], d[1]) for d in s.dots]
    order = []
    for na, nb_ in TREE_ORDER:
        order += [k for k, pr in enumerate(pairs) if pr == (b[na], b[nb_])]
    assert sorted(order) == list(range(len(pairs)))
    return order


def write_model(path, s, st, aux, t0, nsteps, h, every, order=()):
    w = []
    f = lambda *xs: " ".join(repr(float(x)) for x in xs)
    w.append("g " + f(*s.g))
    w.append("baum " + f(s.baum_a, s.baum_b))
    w.append("consts " + f(E.TA_UP, E.TA_DN, E.K_FV, E.ECC, E.ECC_SLOPE, E.CONTACT_D0))
    w.append("bodies %d" % s.nb)
    for m, I in zip(s.m, s.I):
        w.append(f(m, *I))
    w.append("pts %d" % len(s.pts))
    for a, b, ra, rb in s.pts:
        w.append("%d %d %s %s" % (a, b, f(*ra), f(*rb)))
    w.append("dots %d" % len(s.dots))
    for a, b, ua, wb in s.dots:
        w.append("%d %d %s %s" % (a, b, f(*ua), f(*wb)))
    w.append("order %d %s" % (len(order), " ".join(str(i) for i in order)))
    w.append("acts %d" % len(s.acts))
    for name, a, b, ax, T, W, t_on, rise, t_off in s.acts:
        w.append("%d %d %s %s" % (a, b, f(*ax), f(T, W, t_on, rise, t_off)))
    w.append("stops %d" % len(s.stops))
    for name, a, b, ax, ea, fb, lo, hi, k, c in s.stops:
        w.append("%d %d %s %s %s %s" % (a, b, f(*ax), f(*ea), f(*fb), f(lo, hi, k, c)))
    w.append("holds %d" % len(s.holds))
    for name, a, b, qref, K, Cc, cap in s.holds:
        w.append("%d %d %s %s %s %s" % (a, b, f(*qref), f(*K), f(*Cc), f(cap)))
    w.append("tones %d" % len(s.tones))
    for name, a, b, tl in s.tones:
        w.append("%d %d %s" % (a, b, f(*tl)))
    w.append("cons %d" % len(s.contacts))
    for name, b, p, kn, cn, kt, ct, mu in s.contacts:
        w.append("%d %s %s" % (b, f(*p), f(kn, cn, kt, ct, mu)))
    w.append("anchors " + " ".join("0 0.0 0.0" if a is None else "1 " + f(*a) for a in aux))
    w.append("run %s %d %s %d" % (f(t0), nsteps, f(h), every))
    w.append("state %d %s" % (len(st), f(*st)))
    with open(path, "w") as fh:
        fh.write("\n".join(w) + "\n")


def run_raw(s, st, t0, nsteps, h, every, aux=None, order=(), tag="run", reps=0):
    """records [(t, state, contact forces, drives (tau, speed, activation), stops (tau, angle))] from the probe."""
    aux = s.new_aux() if aux is None else aux
    path = os.path.join(OUT, "model_%s_%d.txt" % (tag, os.getpid()))
    write_model(path, s, st, aux, t0, nsteps, h, every, order)
    try:
        r = subprocess.run([BIN, path] + ([str(reps)] if reps else []), capture_output=True, text=True)
    finally:
        os.remove(path)
    if r.returncode != 0:
        raise E.SolveError(r.stderr.strip() or "probe failed")
    if reps:
        return r.stdout.strip()
    n, nc, na, ns = len(st), len(s.contacts), len(s.acts), len(s.stops)
    recs = []
    for line in r.stdout.splitlines():
        v = [float(x) for x in line.split()]
        o = 1 + n
        recs.append((v[0], v[1:o], [tuple(v[o + 3 * i:o + 3 * i + 3]) for i in range(nc)],
                     [tuple(v[o + 3 * nc + 3 * i:o + 3 * nc + 3 * i + 3]) for i in range(na)],
                     [tuple(v[o + 3 * nc + 3 * na + 2 * i:o + 3 * nc + 3 * na + 2 * i + 2]) for i in range(ns)]))
    return recs


def simulate(mdl, st, t0, t1, h=1e-3, every=5, aux=None, tree=True):
    """body3d.drive.simulate, from the probe: (rows, final state); a row is (t, observables, {foot: force})."""
    s = mdl.sys
    n = int(round((t1 - t0) / h))
    recs = run_raw(s, st, t0, n, h, every, aux, tree_order(mdl) if tree else ())
    rows, prev = [], None
    for t, state, con, act, stop in recs:
        o = mdl.observe(state)
        if prev is None:
            if o["bat_az"] > 0.0:
                o["bat_az"] -= 360.0
        else:
            while o["bat_az"] - prev > 180.0:
                o["bat_az"] -= 360.0
            while o["bat_az"] - prev < -180.0:
                o["bat_az"] += 360.0
        prev = o["bat_az"]
        o["act"] = {a[0]: v for a, v in zip(s.acts, act)}
        o["stops"] = {a[0]: v for a, v in zip(s.stops, stop)}
        grf = {"L": [0.0, 0.0, 0.0], "R": [0.0, 0.0, 0.0]}
        for c, fc in zip(s.contacts, con):
            for k in range(3):
                grf[c[0][0]][k] += fc[k]
        rows.append((t, o, grf))
    return rows, recs[-1][1]


def _cases():
    """(label, model, state, t0, steps): undriven on the ground; every drive at its bound (stops and
    landings); a modest drive off the ground without gravity."""
    out = []
    m = B.Model()
    st = B.place(m, -0.340)
    Dr.solve_tone(m, st, -0.340)
    Dr.add_damping(m, st)
    out.append(("undriven from foot down", m, st, -0.340, 340))
    m = B.Model()
    st = B.place(m, -0.250)
    bd = Dr.bounds(m.H, m.M)
    st = Dr.add_drives(m, st, {n: (bd[n], -0.245 if n in ("core",) or n.startswith(("hip", "knee")) else -0.170, 0.05, 0.1) for n in bd})
    Dr.solve_tone(m, st, -0.250)
    Dr.add_damping(m, st)
    out.append(("every drive at its bound", m, st, -0.250, 310))
    m = B.Model(contacts=False)
    m.sys.g = (0.0, 0.0, 0.0)
    st = B.place(m, -0.250)
    st = Dr.add_drives(m, st, {n: (0.5 * bd[n], -0.250, 0.05, 1.0) for n in bd})
    Dr.add_damping(m, st)
    out.append(("off the ground, half drive", m, st, -0.250, 150))
    fitted = os.path.join(HERE, "fit3d_design.json")
    if os.path.isfile(fitted):                       # the fitted swing itself, from its own starting ground forces
        import json
        import fit3d
        ctx = fit3d.Ctx()
        for a, (n, v) in zip(ctx.m.sys.acts, json.load(open(fitted))["plan"].items()):
            assert a[0] == n
            a[4], a[6], a[7], a[8] = v
        ctx.m.check_aux = list(ctx.aux0)
        out.append(("the fitted swing (design rules)", ctx.m, ctx.st, fit3d.T0, 310))
    return out


def check():
    build()
    worst = 0.0
    for label, m, st, t0, n in _cases():
        ref = []
        aux0 = getattr(m, "check_aux", None)
        aux = m.sys.new_aux() if aux0 is None else list(aux0)
        m.sys.anchors(st, aux)
        s1 = list(st)
        for i in range(n):
            s1 = m.sys.step(t0 + i * 1e-3, s1, aux, 1e-3)
            if (i + 1) % 10 == 0:
                ref.append(list(s1))
        for nm, order in (("dense", ()), ("tree-ordered", tree_order(m))):
            recs = run_raw(m.sys, st, t0, n, 1e-3, 10, aux=None if aux0 is None else list(aux0), order=order)[1:]
            d = max(abs(a - b) for (t, state, *_), r in zip(recs, ref) for a, b in zip(state, r))
            dend = max(abs(a - b) for a, b in zip(recs[-1][1], ref[-1]))
            print("%-28s %-13s largest state difference over %d records: %.2e (at the end %.2e)" % (label, nm, len(ref), d, dend))
            worst = max(worst, d)
    print("worst %.2e, tolerance %.0e: %s" % (worst, CHECK_TOL, "PASS" if worst <= CHECK_TOL else "FAIL"))
    return worst <= CHECK_TOL


def bench():
    build()
    label, m, st, t0, n = _cases()[1]
    for nm, order in (("dense, engine.py's row order", ()), ("tree-ordered envelope", tree_order(m))):
        print("%-30s %s" % (nm, run_raw(m.sys, st, t0, n, 1e-3, 10, order=order, reps=20)))


if __name__ == "__main__":
    if "--check" in sys.argv:
        sys.exit(0 if check() else 1)
    elif "--bench" in sys.argv:
        bench()
    else:
        print(__doc__)
