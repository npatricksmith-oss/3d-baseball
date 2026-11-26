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
SELF_NUDGE, SELF_FACTOR = 1e-13, 10   # a sensitive case: the engines may differ by this many times the Python engine's own difference from itself so nudged


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
    assert not s.ext, "the probe has no external loads (inverse dynamics runs in the Python engine)"
    w = []
    f = lambda *xs: " ".join(repr(float(x)) for x in xs)
    w.append("g " + f(*s.g))
    w.append("baum " + f(s.baum_a, s.baum_b))
    w.append("consts " + f(E.TA_UP, E.TA_DN, E.K_FV, E.ECC, E.ECC_SLOPE, E.CONTACT_D0, E.TA_REL))
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
    w.append("fvtabs %d" % len(s.fvtabs))
    for k, tab in enumerate(s.fvtabs):
        w.append("%d %s %s" % (len(tab), f(s.tab_ecc(k)), " ".join(f(*pt) for pt in tab)))
    w.append("acts %d" % len(s.acts))
    for name, a, b, ax, T, W, t_on, rise, t_off, a0, tab, ksec, axb, held in s.acts:
        w.append("%d %d %s %s %d %s %s %d" % (a, b, f(*ax), f(T, W, t_on, rise, t_off, a0), tab, f(ksec), f(*axb), int(held)))
    w.append("stops %d" % len(s.stops))
    for name, a, b, ax, ea, fb, lo, hi, k, c, kind, axb in s.stops:
        w.append("%d %d %s %s %s %s %d %s" % (a, b, f(*ax), f(*ea), f(*fb), f(lo, hi, k, c), kind, f(*axb))
                 + (" " + f(*(s.ovals[name][0] + s.ovals[name][1] + s.ovals[name][2] + s.ovals[name][3:])) if kind == 3 else ""))
    w.append("holds %d" % len(s.holds))
    for name, a, b, qref, K, Cc, cap in s.holds:
        w.append("%d %d %s %s %s %s" % (a, b, f(*qref), f(*K), f(*Cc), f(cap)))
    w.append("tones %d" % len(s.tones))
    for name, a, b, tl in s.tones:
        w.append("%d %d %s" % (a, b, f(*tl)))
    w.append("cons %d" % len(s.contacts))
    for name, b, p, kn, cn, kt, ct, mu in s.contacts:
        w.append("%d %s %s" % (b, f(*p), f(kn, cn, kt, ct, mu)))
    nc = len(s.contacts)
    sec = list(aux[nc:]) + [0.0] * (len(s.acts) - max(0, len(aux) - nc))
    w.append("anchors " + " ".join("0 0.0 0.0" if a is None else "1 " + f(*a) for a in aux[:nc]))
    w.append("sec %d %s" % (len(sec), f(*sec)))
    w.append("run %s %d %s %d" % (f(t0), nsteps, f(h), every))
    w.append("state %d %s" % (len(st), f(*st)))
    with open(path, "w") as fh:
        fh.write("\n".join(w) + "\n")


class RunFailed(E.SolveError):
    """the probe's run failed (a singular posture, an undefined stop). recs: every record it made before the failure,
    in run_raw's form; the failure came after the last of them and before the next would have been made."""

    def __init__(self, msg, recs):
        E.SolveError.__init__(self, msg)
        self.recs = recs


def run_raw(s, st, t0, nsteps, h, every, aux=None, order=(), tag="run", reps=0):
    """records [(t, state, contact forces, drives (tau, speed, activation), stops (tau, angle))] from the probe.
    A run that fails raises RunFailed, which carries the records made before the failure."""
    aux = s.new_aux() if aux is None else aux
    path = os.path.join(OUT, "model_%s_%d.txt" % (tag, os.getpid()))
    write_model(path, s, st, aux, t0, nsteps, h, every, order)
    try:
        r = subprocess.run([BIN, path] + ([str(reps)] if reps else []), capture_output=True, text=True)
    finally:
        os.remove(path)
    if r.returncode != 0 and reps:
        raise E.SolveError(r.stderr.strip() or "probe failed")
    if reps:
        return r.stdout.strip()
    n, nc, na, ns = len(st), len(s.contacts), len(s.acts), len(s.stops)
    width = 1 + n + 3 * nc + 3 * na + 2 * ns + na
    recs = []
    for line in r.stdout.splitlines():
        v = [float(x) for x in line.split()]
        if len(v) != width:                          # a line cut short by the failure: not a record
            break
        o = 1 + n
        recs.append((v[0], v[1:o], [tuple(v[o + 3 * i:o + 3 * i + 3]) for i in range(nc)],
                     [tuple(v[o + 3 * nc + 3 * i:o + 3 * nc + 3 * i + 3]) for i in range(na)],
                     [tuple(v[o + 3 * nc + 3 * na + 2 * i:o + 3 * nc + 3 * na + 2 * i + 2]) for i in range(ns)],
                     v[o + 3 * nc + 3 * na + 2 * ns:]))
    if r.returncode != 0:
        raise RunFailed(r.stderr.strip() or "probe failed", recs)
    return recs


def simulate(mdl, st, t0, t1, h=1e-3, every=5, aux=None, tree=True):
    """body3d.drive.simulate, from the probe: (rows, final state); a row is (t, observables, {foot: force})."""
    s = mdl.sys
    n = int(round((t1 - t0) / h))
    recs = run_raw(s, st, t0, n, h, every, aux, tree_order(mdl) if tree else ())
    rows, prev = [], None
    for t, state, con, act, stop, sec in recs:
        o = mdl.observe(state)
        o["sec"] = sec
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
        xp = s.pose(state, mdl.b["pelvis"])[0]
        mz = 0.0                                      # the ground's turning moment about the vertical through the pelvis
        for c, fc in zip(s.contacts, con):
            for k in range(3):
                grf[c[0][0]][k] += fc[k]
            if fc[2] > 0.0:
                xb, qb, _, _ = s.pose(state, c[1])
                pc = E.add(xb, E.mv(E.qR(qb), c[2]))
                mz += (pc[0] - xp[0]) * fc[1] - (pc[1] - xp[1]) * fc[0]
        o["ground_mz"] = mz
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
    arm = lambda n: n.startswith(("shoulder", "elbow", "wrist"))
    st = Dr.add_drives(m, st, {n: (bd[n], -0.170 if arm(n) else -0.245, 0.05, 0.1, 0.2 if arm(n) else 0.3) for n in bd})
    Dr.solve_tone(m, st, -0.250)
    Dr.add_damping(m, st)
    aux = m.sys.new_aux()
    m.sys.anchors(st, aux)
    m.sys.sec_start(st, aux)                         # the leg drives tensed at 0.3, their tendon springs loaded to match
    m.check_aux = aux
    out.append(("every drive at its bound, all tensed, tendons", m, st, -0.250, 310))
    m = B.Model(contacts=False)
    m.sys.g = (0.0, 0.0, 0.0)
    st = B.place(m, -0.250)
    st = Dr.add_drives(m, st, {n: (0.5 * bd[n], -0.250, 0.05, 1.0) for n in bd})
    Dr.add_damping(m, st)
    out.append(("off the ground, half drive", m, st, -0.250, 150))
    m = B.Model(ranges=())                           # the body with its joints' ranges as stops (both kinds), driven hard into them
    st = B.place(m, -0.250)
    st = Dr.add_drives(m, st, {n: (bd[n], -0.170 if arm(n) else -0.245, 0.05, 0.1, 0.2 if arm(n) else 0.3) for n in bd})
    B.add_ranges(m, st)
    Dr.solve_tone(m, st, -0.250)
    Dr.add_damping(m, st)
    aux = m.sys.new_aux()
    m.sys.anchors(st, aux)
    m.sys.sec_start(st, aux)
    m.check_aux = aux
    out.append(("every drive at its bound, the joints' ranges on", m, st, -0.250, 200))
    m = B.Model(ranges=(), grip="hands")             # the hands' body (4.11): its arm stops (the upper arm's is the new kind 2) driven into
    st = B.place(m, -0.250)
    st = Dr.add_drives(m, st, {n: (bd[n], -0.245, 0.05, 0.1, 0.2 if arm(n) or n.startswith(("sh_", "pron", "sup")) else 0.3) for n in bd})
    B.add_ranges(m, st)
    Dr.solve_tone(m, st, -0.250)
    Dr.add_damping(m, st)
    aux = m.sys.new_aux()
    m.sys.anchors(st, aux)
    m.sys.sec_start(st, aux)
    m.check_aux = aux
    out.append(("the hands' body, every drive at its bound", m, st, -0.250, 200))
    m = B.Model(ranges=(), grip="hands")             # held muscles (4.11, part 2): tensed at 0.6, firing at staggered onsets, tendons loaded
    st = B.place(m, -0.250)
    names = sorted(bd)
    st = Dr.add_drives(m, st, {n: (bd[n], -0.240 + 0.004 * k, 0.05, 0.1, 0.6) for k, n in enumerate(names)}, held=True)
    B.add_ranges(m, st)
    Dr.solve_tone(m, st, -0.250)
    Dr.add_damping(m, st)
    aux = m.sys.new_aux()
    m.sys.anchors(st, aux)
    m.sys.sec_start(st, aux)
    m.check_aux = aux
    out.append(("the hands' body, every muscle tensed and held", m, st, -0.250, 200))
    m = B.Model(ranges=(), grip="hands", feet="cleats")   # the feet on dirt (4.11, part 3): the measured grip, the ankles' ends
    st = B.place(m, -0.250)
    st = Dr.add_drives(m, st, {n: (bd[n], -0.245, 0.05, 0.1, 0.3) for n in bd if not (arm(n) or n.startswith(("sh_", "pron", "sup")))}, held=True)
    B.add_ranges(m, st)
    Dr.solve_tone(m, st, -0.250)
    Dr.add_damping(m, st)
    aux = m.sys.new_aux()
    m.sys.anchors(st, aux)
    m.sys.sec_start(st, aux)
    m.check_aux = aux
    # A SENSITIVE case, judged against itself (the sixth entry; session 9's review, P3-1): with every leg muscle at its
    # bound the body is on two of its sole's points by 80 ms, and from 100 ms the motion multiplies rounding differences
    # (about three times every 10 ms on average, in bursts of 10 to 40). The two engines cannot agree to 1e-9 over
    # 200 ms of that, and need not: each record's difference is held to ten times what the Python engine differs from
    # ITSELF when one velocity of its start is moved by 1e-13 (and to CHECK_TOL wherever that is larger).
    out.append(("cleats, the legs and trunk at their bounds", m, st, -0.250, 200, "sensitive"))
    m = B.Model(ranges=(), grip="hands", wrist="oval")    # the wrists' ellipses (4.12): the stop that reads both of a wrist's angles, driven into
    st = B.place(m, -0.250)
    wr = lambda n: n.startswith("wrist")
    st = Dr.add_drives(m, st, {n: (bd[n], -0.245, 0.05, 0.1, 0.2 if arm(n) or n.startswith(("sh_", "pron", "sup")) else 0.3) for n in bd})
    B.add_ranges(m, st)
    Dr.solve_tone(m, st, -0.250)
    Dr.add_damping(m, st)
    aux = m.sys.new_aux()
    m.sys.anchors(st, aux)
    m.sys.sec_start(st, aux)
    m.check_aux = aux
    # 100 ms, not 200: both wrists' ellipses are engaged inside it (the rear at 30 and at 90 ms, up to 1,275 N m) and the
    # engines agree to 3e-11; from 110 ms this motion multiplies rounding differences five-hundredfold in 20 ms and the
    # engines' difference (1.3e-8) then tracks the Python engine's own difference from itself under a 1e-13 nudge (1.5e-8),
    # as the cleats case does (sim/target/codex-probes/body3d/s10/oval_diverge.py)
    out.append(("the wrists' ellipses, every drive at its bound", m, st, -0.250, 100))
    fitted = os.path.join(HERE, "fit3d_d34_all.json")
    if not os.path.isfile(fitted):                   # session 6's fit (its 22 channels; session 7's then switched off)
        fitted = os.path.join(HERE, "fit3d_d34_arms.json")
    if os.path.isfile(fitted):                       # the fitted swing itself (4.5's limits), from its own start
        import json
        import fit3d
        ctx = fit3d.Ctx()
        st, aux = ctx.prepare(ctx.u_of(json.load(open(fitted))["plan"]))
        ctx.m.check_aux = aux
        out.append(("the fitted swing (4.5's limits)", ctx.m, st, fit3d.T0, 310))
    return out


def check():
    build()
    worst, ok = 0.0, True
    for case in _cases():
        label, m, st, t0, n = case[:5]
        sensitive = len(case) > 5
        aux0 = getattr(m, "check_aux", None)

        def python_run(start):
            aux = m.sys.new_aux() if aux0 is None else list(aux0)
            m.sys.anchors(start, aux)
            s1, out = list(start), []
            for i in range(n):
                s1 = m.sys.step(t0 + i * 1e-3, s1, aux, 1e-3)
                if (i + 1) % 10 == 0:
                    out.append(list(s1))
            return out
        ref = python_run(st)
        allow = [CHECK_TOL] * len(ref)
        if sensitive:                                    # the Python engine against itself, one velocity moved by 1e-13
            st2 = list(st)
            st2[7] += SELF_NUDGE
            own = [max(abs(a - b) for a, b in zip(x, y)) for x, y in zip(python_run(st2), ref)]
            allow = [max(CHECK_TOL, SELF_FACTOR * d) for d in own]
        for nm, order in (("dense", ()), ("tree-ordered", tree_order(m))):
            recs = run_raw(m.sys, st, t0, n, 1e-3, 10, aux=None if aux0 is None else list(aux0), order=order)[1:]
            ds = [max(abs(a - b) for a, b in zip(state, r)) for (t, state, *_), r in zip(recs, ref)]
            d, dend = max(ds), ds[-1]
            if sensitive:
                good = all(x <= lim for x, lim in zip(ds, allow))
                print("%-46s %-13s largest state difference over %d records: %.2e (at the end %.2e); against itself moved %.0e the Python engine ends %.2e apart: %s"
                      % (label, nm, len(ref), d, dend, SELF_NUDGE, own[-1], "within %d times that at every record" % SELF_FACTOR if good else "MORE THAN %d TIMES THAT" % SELF_FACTOR))
                ok = ok and good
            else:
                print("%-46s %-13s largest state difference over %d records: %.2e (at the end %.2e)" % (label, nm, len(ref), d, dend))
                worst = max(worst, d)
    ok = ok and worst <= CHECK_TOL
    print("worst %.2e (the sensitive case apart), tolerance %.0e: %s" % (worst, CHECK_TOL, "PASS" if ok else "FAIL"))
    return ok


def bench():
    build()
    label, m, st, t0, n = _cases()[1][:5]
    for nm, order in (("dense, engine.py's row order", ()), ("tree-ordered envelope", tree_order(m))):
        print("%-30s %s" % (nm, run_raw(m.sys, st, t0, n, 1e-3, 10, order=order, reps=20)))


if __name__ == "__main__":
    if "--check" in sys.argv:
        sys.exit(0 if check() else 1)
    elif "--bench" in sys.argv:
        bench()
    else:
        print(__doc__)
