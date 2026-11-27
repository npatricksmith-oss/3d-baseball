"""What the written swing asks of the muscles and the ground: the torque about every muscle axis and the force at every
sole point that give the written motion its accelerations (PLAN D48; docs/body-motion.md 4.17).

    uv run --with numpy python3 analysis/research/body/swing_inverse.py [--out plan.json] [--table]

It is the FIRST PLAN handed to the forward run (swing_track.py), and the list of where the body cannot do the motion.
It moves nothing: the forward run alone says what the body does.

The equations are the engine's own (analysis/oracle/body3d/engine.py, every body free, every joint a set of constraint
rows). At each millisecond the written motion gives every body's acceleration (differences of its poses). Unknown are
the joints' constraint forces, a torque about each of the 29 muscle axes and a force at each sole point that is on the
ground. The feet's two legs and the two arms on one bat each close a ring, so the split is not unique: of all splits
that give the accelerations exactly, the one is taken that asks least of the muscles (each torque counted against its
strength held still) and, a little, least of the ground; a sole point may push but not pull, and a foot that is coming
down or about to leave takes its load in or gives it up over HANDOVER_S. What it cannot see: whether muscles with rise
times can make those torques in time, and whether the ground's grip holds (reported, not enforced).
"""
import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import swing_author as A      # noqa: E402
import swing_track as T       # noqa: E402

E, B = T.E, T.B
ON_GROUND = 0.002              # m: a sole point this near the ground may carry load
HANDOVER_S = 0.080             # s [E: 0.05-0.12]: a foot coming down takes load over this long (and gives it up over this long before it leaves)
GROUND_W = 0.02                # how much the ground's forces count beside the muscles' torques in "asks least" [E]
RISE_LEAD_S = E.TA_UP          # the plan asks each torque this much early: a muscle's level rises in about this long


def accelerations(tr):
    """every body's (linear acceleration, angular acceleration) at every sample of the written motion: differences of
    its velocities, which are differences of its poses."""
    s = tr.m.sys
    n = tr.n
    out = []
    for k in range(n):
        a, c = tr.states[max(0, k - 1)], tr.states[min(n - 1, k + 1)]
        span = tr.dt * (min(n - 1, k + 1) - max(0, k - 1))
        row = []
        for i in range(s.nb):
            o = 13 * i
            row.append(([(c[o + 7 + q] - a[o + 7 + q]) / span for q in range(3)], [(c[o + 10 + q] - a[o + 10 + q]) / span for q in range(3)]))
        out.append(row)
    return out


def handover(t, sd):
    """the share of its full say a foot's sole points have at time t (1: on the ground for good)."""
    if sd == "R":
        return 1.0
    if t <= A.T_OFF:
        return min(1.0, max(0.0, (A.T_OFF - t) / HANDOVER_S))
    if t >= A.T_TOUCH:
        return min(1.0, max(0.0, (t - A.T_TOUCH) / HANDOVER_S))
    return 0.0


def solve(tr, k, acc, np):
    """sample k: (torque about each muscle axis by the muscles N m, {sole point: force}, the worst share of a muscle's
    strength held still asked, how far the equations are from met)."""
    m, s = tr.m, tr.m.sys
    st = tr.states[k]
    X, R, V, Wv = s._kin(st)
    rows = s._rows(X, R, V, Wv)
    nb, nr, nax = s.nb, len(rows), tr.nax
    pts = []
    for ci, (name, b, p, *_r) in enumerate(s.contacts):
        P = E.add(X[b], E.mv(R[b], p))
        share = handover(tr.ts[k], name[0])
        if P[2] < ON_GROUND and share > 0.0:
            pts.append((ci, b, P, share))
    ncol = nr + nax + 3 * len(pts)
    Am = np.zeros((6 * nb, ncol))
    bv = np.zeros(6 * nb)
    for i in range(nb):
        al, aa = acc[k][i]
        I = s.I[i]
        w = Wv[i]
        l = E.mtv(R[i], w)
        h = E.mv(R[i], (I[0] * l[0], I[1] * l[1], I[2] * l[2]))
        la = E.mtv(R[i], aa)
        Ia = E.mv(R[i], (I[0] * la[0], I[1] * la[1], I[2] * la[2]))
        gy = E.cross(w, h)
        for q in range(3):
            bv[6 * i + q] = s.m[i] * (al[q] - s.g[q])
            bv[6 * i + 3 + q] = Ia[q] + gy[q]
    for r_, (a, Ja, b, Jb, *_x) in enumerate(rows):
        if a >= 0:
            Am[6 * a:6 * a + 6, r_] += Ja
        if b >= 0:
            Am[6 * b:6 * b + 6, r_] += Jb
    damp = np.zeros(nax)
    wts = np.zeros(ncol)
    for j, (name, a, b, fam, k0, na) in enumerate(tr.joints):
        wrel = E.sub(Wv[b], Wv[a])
        for q in range(na):
            ax = tr.axes[k0 + q]
            if ax["kind"] == 0:
                n = E.mv(R[a], ax["ax"])
            elif ax["kind"] == 1:
                n = E.mv(R[b], ax["ax"])
            else:
                n = s.float_axis(E.mv(R[a], ax["ax"]), E.mv(R[b], ax["axb"]))
            col = nr + k0 + q
            Am[6 * b + 3:6 * b + 6, col] += n
            Am[6 * a + 3:6 * a + 6, col] -= n
            damp[k0 + q] = -ax["damp"] * E.dot(wrel, n)
            wts[col] = 1.0 / (0.5 * (ax["Tp"] + ax["Tm"])) ** 2
    BW = sum(s.m) * E.G
    for pi, (ci, b, P, share) in enumerate(pts):
        r = E.sub(P, X[b])
        for q in range(3):
            e = [0.0, 0.0, 0.0]
            e[q] = 1.0
            col = nr + nax + 3 * pi + q
            Am[6 * b + q, col] += 1.0
            Am[6 * b + 3:6 * b + 6, col] += E.cross(r, e)
            wts[col] = GROUND_W / (BW * share) ** 2 * (4.0 if q < 2 else 1.0)
    live = list(range(len(pts)))
    for _ in range(len(pts) + 1):
        cols = list(range(nr + nax)) + [nr + nax + 3 * pi + q for pi in live for q in range(3)]
        Ar = Am[:, cols]
        Wd = np.diag(wts[cols])
        K = np.block([[Wd, Ar.T], [Ar, np.zeros((6 * nb, 6 * nb))]])
        rhs = np.concatenate([np.zeros(len(cols)), bv])
        z = np.linalg.lstsq(K, rhs, rcond=None)[0][:len(cols)]
        f = z[nr + nax:].reshape(-1, 3) if live else np.zeros((0, 3))
        bad = [live[i] for i in range(len(live)) if f[i][2] < -1e-6]
        if not bad:
            break
        worst = min(range(len(live)), key=lambda i: f[i][2])
        live.pop(worst)                              # a sole point asked to pull: it carries nothing
    resid = float(np.max(np.abs(Ar @ z - bv)))
    c = z[nr:nr + nax]
    mus = c - damp
    forces = {s.contacts[pts[pi][0]][0]: tuple(float(x) for x in f[i]) for i, pi in enumerate(live)}
    share = max(abs(mus[i]) / (tr.axes[i]["Tp"] if mus[i] > 0 else tr.axes[i]["Tm"]) for i in range(nax))
    return [float(x) for x in mus], forces, share, resid


def plan_from(tr, log=None, every=T.FF_EVERY):
    """the first plan: the torques solve() finds, every `every` samples, asked RISE_LEAD_S early. Returns (knots, a
    report: per knot the time, the feet's loads, the muscles most asked)."""
    import numpy as np
    acc = accelerations(tr)
    lead = int(round(RISE_LEAD_S / tr.dt))
    nk = (tr.n - 1 + every - 1) // every + 1
    knots, rep = [], []
    BW = sum(tr.m.sys.m) * E.G
    for kn in range(nk):
        k = min(tr.n - 1, kn * every + lead)
        mus, forces, share, resid = solve(tr, k, acc, np)
        caps = [max(-T.FF_CAP * ax["Tm"], min(T.FF_CAP * ax["Tp"], v)) for v, ax in zip(mus, tr.axes)]
        knots.append(caps)
        fz = {"L": 0.0, "R": 0.0}
        mu = 0.0
        for nm, f in forces.items():
            fz[nm[0]] += f[2] / BW
        for sd in "LR":
            fh = [sum(f[q] for nm, f in forces.items() if nm[0] == sd) for q in range(2)]
            if fz[sd] > 0.02:
                mu = max(mu, math.hypot(*fh) / (fz[sd] * BW))
        over = sorted(((abs(v) / (ax["Tp"] if v > 0 else ax["Tm"]), ax["plus"] if v > 0 else ax["minus"], v) for v, ax in zip(mus, tr.axes)), reverse=True)[:4]
        rep.append(dict(t=tr.ts[k], front=fz["L"], rear=fz["R"], grip=mu, resid=resid, most=[(round(a, 2), n, round(v)) for a, n, v in over]))
        if log is not None and kn % 25 == 0:
            log("%+6.0f ms: feet front %.2f rear %.2f of his weight (grip used %.2f); most asked: %s"
                % (1e3 * tr.ts[k], fz["L"], fz["R"], mu, ", ".join("%s %.0f%% (%+.0f N m)" % (n, 100 * a, v) for a, n, v in over)))
    return knots, rep


def over_strength(tr, rep, log=print):
    """where the written motion asks a muscle for more than it has held still: each such muscle, its worst, and when."""
    worst = {}
    for r in rep:
        for a, n, v in r["most"]:
            if a > worst.get(n, (0.0, 0.0, 0.0))[0]:
                worst[n] = (a, r["t"], v)
    log("muscles the written motion asks for more than their strength held still (the worst of each):")
    any_ = False
    for n, (a, t, v) in sorted(worst.items(), key=lambda kv: -kv[1][0]):
        if a > 1.0:
            any_ = True
            log("   %-16s %.2f times (%+.0f N m) at %+.0f ms" % (n, a, v, 1e3 * t))
    if not any_:
        log("   none")
    return worst


def main():
    a = sys.argv[1:]
    arg = lambda k, d: a[a.index(k) + 1] if k in a else d
    tr = T.Track(strength=float(arg("--strength", T.STRENGTH)), hip_side=float(arg("--hip-side", T.HIP_SIDE)), log=print)
    knots, rep = plan_from(tr, log=print if "--table" in a else None)
    over_strength(tr, rep)
    print("the equations met within %.2e N or N m; the ground's grip used up to %.2f (a cleat on dirt holds %.2f)" % (max(r["resid"] for r in rep), max(r["grip"] for r in rep), B.MU_CLEAT))
    T.set_plan(tr, knots)
    out = arg("--out", os.path.join(T.OUT, "plan_inverse.json"))
    T.save_plan(tr, out, {"from": "swing_inverse.py"})
    print("kept " + out)


if __name__ == "__main__":
    main()
