"""How solid is the trunk's torque? (docs/body-motion.md 4.4; PLAN D31 phase 1, session 4.) Diagnosis only:
no bound, law, drive or fit is touched.

    python3 analysis/research/body/trunk3d.py > sim/target/codex-probes/body3d/trunk3d.out      (2 min)

The torque at the lumbar joint (on the trunk, from the pelvis) by Newton's and Euler's equations summed over
segments, with no solve, from the placed postures of inverse3d.py:
  from above: what the upper body (trunk with head, both arms, the bat with the hands) needs, about the joint;
  from below: what [4]'s measured ground forces deliver to the joint, less what the pelvis and legs need.
They differ by the whole body's unexplained moment (the feet's free moments, which [4] does not report, plus
every error of the reconstruction). Components: about the trunk's own long axis (how strength is tested), its
forward axis (side bending) and its side-to-side axis (bending forward and back).
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import inverse3d as I                                   # noqa: E402
from inverse3d import E, B                              # noqa: E402

H, M = I.H, I.M
UPPER = ("trunk", "uarm_L", "farm_L", "uarm_R", "farm_R", "bat")
ARMS = ("uarm_L", "farm_L", "uarm_R", "farm_R", "bat")
BOUND = 288.0               # the trunk's bound in the design (body-motion 4.3), N m
SD_ELBOW, SD_LAG = 8.0, 10.0    # [4] Table 1's between-batter SDs at contact (trail elbow 8, lag 10)
_ROWS = {}


def raw(name, shift=0.0):
    def f(t):
        if name not in _ROWS:
            _ROWS[name] = B.C.load(name)
        return B.C.resample(_ROWS[name], t, t, 1.0)[0] + shift
    return f


def motion(dt, arms_scale=1.0, smooth=3):
    """the placed motion: model and [(t, state with consistent velocities, consistent accelerations, report)]."""
    m = B.Model(H, M, contacts=False)
    s = m.sys
    s.stops, s.holds = [], []
    s.baum_a = s.baum_b = 0.0
    if arms_scale != 1.0:
        for n in ARMS:
            i = m.b[n]
            s.m[i] *= arms_scale
            s.I[i] = tuple(c * arms_scale for c in s.I[i])
    I.HANDS_SMOOTH = smooth
    times, states, reps = I.trajectory(m, dt)
    I.HANDS_SMOOTH = 3
    idx = [i for i, t in enumerate(times) if I.T_A - 1e-9 <= t <= I.T_B + 1e-9]
    vel = {i: I.velocities(m, states[i - 1], states[i + 1], dt) for i in range(1, len(times) - 1)}
    out = []
    for i in idx:
        st = list(states[i])
        vi, _ = I.consistent(m, st, vel[i], 5)
        for b_ in range(s.nb):
            st[13 * b_ + 7:13 * b_ + 13] = vi[b_]
        acc = [[(p - q) / (2 * dt) for p, q in zip(vel[i + 1][b_], vel[i - 1][b_])] for b_ in range(s.nb)]
        acc, _ = I.consistent(m, st, acc, 6)
        out.append((times[i], st, acc, reps[i]))
    return m, out


def needs(m, st, acc, names, about):
    """the force and the moment about `about` that the named segments need: sum of m (a - g) and of
    (x - about) x m (a - g) + I alpha + w x (I w)."""
    s = m.sys
    F = [0.0, 0.0, 0.0]
    N = [0.0, 0.0, 0.0]
    for n in names:
        i = m.b[n]
        x, q, v, w = s.pose(st, i)
        R = E.qR(q)
        f = tuple(s.m[i] * (acc[i][c] - s.g[c]) for c in range(3))
        Iw = lambda u: E.mv(R, tuple(s.I[i][c] * e for c, e in enumerate(E.mtv(R, u))))
        n_ = E.add(Iw(tuple(acc[i][3:6])), E.cross(w, Iw(w)))
        N = E.add(N, E.add(E.cross(E.sub(x, about), f), n_))
        F = E.add(F, f)
    return F, N


def analyse(m, mot, cop="default"):
    s = m.sys
    lower = [n for n in m.b if n not in UPPER]
    rows = []
    for t, st, acc, rep in mot:
        J = m.joint_pos(st, "lumbar")
        Fu, Nu = needs(m, st, acc, UPPER, J)                 # from above: the joint's force and torque on the trunk
        Fl, Nl = needs(m, st, acc, lower, J)
        grf = I.ground(m, st, rep, t)
        Mg = [0.0, 0.0, 0.0]
        Fg = [0.0, 0.0, 0.0]
        for sd in "LR":
            f, p, pw = grf[sd]
            if cop != "default":
                x, q, _, _ = s.pose(st, m.b["foot_" + sd])
                cf = B.C.DE_LEVA["foot"][2]
                px = {"ball": B.BALL_FWD - cf, "ankle": B.ANKLE_FWD - cf}[cop] * m.len["foot"]
                pw = E.add(x, E.mv(E.qR(q), (px, 0.0, m.sole_z)))
            Mg = E.add(Mg, E.cross(E.sub(pw, J), f))
            Fg = E.add(Fg, f)
        bu = E.sub(Mg, Nl)                                   # from below: what the ground delivers less what pelvis and legs need
        Rt = E.qR(s.pose(st, m.b["trunk"])[1])
        Rp = E.qR(s.pose(st, m.b["pelvis"])[1])
        # the upper body's own centre of mass and angular momentum about the vertical through it
        mu = sum(s.m[m.b[n]] for n in UPPER)
        rc = [sum(s.m[m.b[n]] * st[13 * m.b[n] + c] for n in UPPER) / mu for c in range(3)]
        vc = [sum(s.m[m.b[n]] * st[13 * m.b[n] + 7 + c] for n in UPPER) / mu for c in range(3)]
        hz = {}
        for n in UPPER:
            i = m.b[n]
            x, q, v, w = s.pose(st, i)
            R = E.qR(q)
            hw = E.mv(R, tuple(s.I[i][c] * e for c, e in enumerate(E.mtv(R, w))))
            hz[n] = s.m[i] * ((x[0] - rc[0]) * (v[1] - vc[1]) - (x[1] - rc[1]) * (v[0] - vc[0])) + hw[2]
        zt = E.mv(Rt, (0, 0, 1))
        # the upper body's moment of inertia about the trunk's long axis through the joint, as placed (if it were carried rigidly)
        Irig = 0.0
        for n in UPPER:
            i = m.b[n]
            x, q, v, w = s.pose(st, i)
            R = E.qR(q)
            d = E.sub(x, J)
            perp2 = E.dot(d, d) - E.dot(d, zt) ** 2
            zl = E.mtv(R, zt)
            Irig += s.m[i] * perp2 + sum(s.I[i][c] * zl[c] ** 2 for c in range(3))
        rows.append({"t": t, "td": Nu, "bu": bu, "Fu": Fu, "Fres": E.sub(E.add(Fu, Fl), Fg), "td_t": E.mtv(Rt, Nu), "bu_t": E.mtv(Rt, bu),
                     "td_pz": E.dot(Nu, E.mv(Rp, (0, 0, 1))), "force_arm_z": E.cross(E.sub(J, rc), Fu)[2], "hz": hz,
                     "Irig": Irig, "Itrunk": sum(s.I[m.b["trunk"]][c] * E.mtv(Rt, zt)[c] ** 2 for c in range(3)), "J": J, "zt": zt,
                     "lever": math.hypot(J[0] - rc[0], J[1] - rc[1])})
    return rows


def peak(rows, key, lo=-0.250, hi=0.0):
    sel = [r for r in rows if lo - 1e-9 <= r["t"] <= hi + 1e-9]
    r = max(sel, key=key)
    return key(r), r["t"]


def case(label, dt=0.010, setup=None, **kw):
    B.CURVE_OVERRIDE.clear()
    keep = (B.TRUNK_FLEX, B.TRUNK_SIDE)
    B.TRUNK_TILT_IN_WORLD = False
    if setup:
        setup()
    m, mot = motion(dt, **kw)
    rows = analyse(m, mot)
    B.CURVE_OVERRIDE.clear()
    B.TRUNK_FLEX, B.TRUNK_SIDE = keep
    B.TRUNK_TILT_IN_WORLD = False
    a, ta = peak(rows, lambda r: r["td_t"][2])
    b, tb = peak(rows, lambda r: r["bu_t"][2])
    v, tv = peak(rows, lambda r: r["td"][2])
    early, _ = peak(rows, lambda r: r["td_t"][2], hi=-0.060)
    print("   %-58s | %4.0f (%4.0f) | %4.0f | %4.0f (%4.0f) | %4.0f (%4.0f)" % (label, a, ta * 1e3, early, v, tv * 1e3, b, tb * 1e3))
    return m, mot, rows, (a, ta, early, b, tb)


def main():
    dt = 0.010
    print("THE TRUNK'S TORQUE at the lumbar joint, [4]'s mean batter (%.2f m, %.0f kg); the design's bound is %.0f N m" % (H, M, BOUND))
    B.CURVE_OVERRIDE.clear()
    m, mot = motion(dt)
    rows = analyse(m, mot)
    s = m.sys
    print("\n1. THROUGH TIME (dt 10 ms). Torque on the trunk about its own axes, N m: long axis (axial, + toward the pitcher), forward axis (side bend), side axis (bending forward-back)")
    print("     t ms | from above: axial, side bend, forward-back | about the vertical | about the pelvis's long axis | from below (forces only): axial, side bend, forward-back | unexplained force on the body N: toward plate, toward pitcher, up")
    for r in rows[::2]:
        print("    %5.0f | %5.0f %5.0f %5.0f | %5.0f | %5.0f | %5.0f %5.0f %5.0f | %6.0f %6.0f %6.0f"
              % (r["t"] * 1e3, r["td_t"][2], r["td_t"][0], r["td_t"][1], r["td"][2], r["td_pz"], r["bu_t"][2], r["bu_t"][0], r["bu_t"][1],
                 r["Fres"][0], r["Fres"][1], r["Fres"][2]))
    for nm, k in (("axial", 2), ("side bend", 0), ("forward-back", 1)):
        a, ta = peak(rows, lambda r: abs(r["td_t"][k]))
        b, tb = peak(rows, lambda r: abs(r["bu_t"][k]))
        d, td_ = peak(rows, lambda r: abs(r["td_t"][k] - r["bu_t"][k]))
        print("   %-12s largest size: from above %4.0f N m (%4.0f ms), from below %4.0f (%4.0f ms); largest difference between the two %4.0f (%4.0f ms)"
              % (nm, a, ta * 1e3, b, tb * 1e3, d, td_ * 1e3))
    tot, tt = peak(rows, lambda r: E.norm(r["td"]))
    print("   the whole torque vector from above: largest %.0f N m at %.0f ms" % (tot, tt * 1e3))
    fr = lambda c: max(abs(r["Fres"][c]) for r in rows)
    print("   the unexplained force: largest toward the plate %.0f N, toward the pitcher %.0f N, vertical %.0f N" % (fr(0), fr(1), fr(2)))
    fh = max(math.hypot(r["Fres"][0], r["Fres"][1]) for r in rows if -0.150 <= r["t"] <= -0.060)
    print("   a vertical force has no moment about the vertical. The horizontal part (up to %.0f N from -150 to -60 ms) could move the axial estimate from below by its size times how far from the joint it acts: %.0f N m per 0.10 m"
          % (fh, fh * 0.10))
    for cop in ("ball", "ankle"):
        r2 = analyse(m, mot, cop)
        b, tb = peak(r2, lambda r: r["bu_t"][2])
        print("   from below with each foot's force acting at its %s instead: axial peak %.0f N m at %.0f ms" % (cop, b, tb * 1e3))
    r90 = min(rows, key=lambda r: abs(r["t"] + 0.090))
    print("   the feet's free moments are not in 'from below'. At -90 ms it would take %.0f N m of them to bring it up to 'from above'; friction allows about 58-117 (body-motion 4.4)"
          % (r90["td"][2] - r90["bu"][2]))

    print("\n2. THE UPPER BODY'S ANGULAR MOMENTUM about the vertical through its own centre of mass (trunk with head, arms, bat with hands: %.1f kg)" % sum(s.m[m.b[n]] for n in UPPER))
    at = lambda t: min(rows, key=lambda r: abs(r["t"] - t))
    for t in (-0.250, -0.170, -0.093, -0.050, 0.0):
        r = at(t)
        h = r["hz"]
        print("     %5.0f ms: %5.1f kg m^2/s = trunk %4.1f + arms %4.1f + bat and hands %4.1f; the joint is %.2f m (horizontally) from that centre"
              % (t * 1e3, sum(h.values()), h["trunk"], sum(h[n] for n in ARMS[:4]), h["bat"], r["lever"]))
    for a, b in ((-0.250, 0.0), (-0.170, 0.0), (-0.170, -0.050), (-0.130, -0.060), (-0.050, 0.0)):
        sel = [r for r in rows if a - 1e-9 <= r["t"] < b - 1e-9]
        dh = sum(at(b)["hz"].values()) - sum(at(a)["hz"].values())
        jt = sum(r["td"][2] for r in sel) * dt
        jf = sum(r["force_arm_z"] for r in sel) * dt
        print("     %4.0f to %4.0f ms: gain %5.1f N m s; through the joint's torque %5.1f (mean %4.0f N m), through the joint's force acting off-centre %5.1f (mean %4.0f N m); sum %5.1f"
              % (a * 1e3, b * 1e3, dh, jt, jt / (b - a), jf, jf / (b - a), jt + jf))
    # bounds from [4]'s trunk rate curve alone
    al = lambda t: (raw("trunk_rot_vel")(t + 0.010) - raw("trunk_rot_vel")(t - 0.010)) / 0.020 * B.D2R
    best = max(((al(r["t"]), r) for r in rows), key=lambda x: x[0])
    carried = max(((al(r["t"]) * r["Irig"], r["t"]) for r in rows), key=lambda x: x[0])
    print("   from [4]'s trunk RATE curve alone: its steepest rise is %.0f rad/s^2 at %.0f ms (slope over +-10 ms)" % (best[0], best[1]["t"] * 1e3))
    print("     the trunk and head alone about the trunk's axis (%.2f kg m^2): %.0f N m: a floor" % (best[1]["Itrunk"], best[0] * best[1]["Itrunk"]))
    print("     the arms and bat carried rigidly with the trunk as placed (%.2f kg m^2 about the trunk's axis through the joint at that time): %.0f N m at %.0f ms"
          % (at(carried[1])["Irig"], carried[0], carried[1] * 1e3))
    print("     (the placed upper body's inertia about the trunk's axis runs from %.2f to %.2f kg m^2 over the swing)" % (min(r["Irig"] for r in rows), max(r["Irig"] for r in rows)))

    print("\n3. SENSITIVITY of the axial torque (about the trunk's long axis), N m")
    print("   %-58s | from above: peak (ms) | its peak before -60 ms | about the vertical: peak (ms) | from below, forces only: peak (ms)" % "case")
    res = {}
    def run(label, **kw):
        res[label] = case(label, **kw)[3]
    run("as placed, differenced over 10 ms")
    run("differenced over 15 ms", dt=0.015)
    run("pelvis and trunk rotation from [4]'s rate curves", setup=lambda: I.rotations_from_rates(True))
    def ov(**sh):
        def f():
            for k, v in sh.items():
                B.CURVE_OVERRIDE[k] = raw(k, v)
        return f
    run("hands closer: both elbows +%.0f deg (1 SD)" % SD_ELBOW, setup=ov(lead_elbow_flex=SD_ELBOW, trail_elbow_flex=SD_ELBOW))
    run("hands further: both elbows -%.0f deg" % SD_ELBOW, setup=ov(lead_elbow_flex=-SD_ELBOW, trail_elbow_flex=-SD_ELBOW))
    run("lag +%.0f deg (1 SD)" % SD_LAG, setup=ov(bat_lag=SD_LAG))
    run("lag -%.0f deg" % SD_LAG, setup=ov(bat_lag=-SD_LAG))
    run("the hands' path smoothed twice as wide", smooth=6)
    def bend(flex=None, side=None):
        def f():
            if flex is not None:
                B.TRUNK_FLEX = flex
            if side is not None:
                B.TRUNK_SIDE = side
        return f
    run("15 ms and the rate curves together", dt=0.015, setup=lambda: I.rotations_from_rates(True))
    def world():
        B.TRUNK_TILT_IN_WORLD = True
    run("the 15 deg lean kept toward the plate as the trunk turns", setup=world)
    run("trunk upright (forward bend 0 deg, not 15)", setup=bend(flex=0.0))
    run("trunk bent forward 30 deg", setup=bend(flex=30.0))
    run("trunk bent 10 deg toward the rear side", setup=bend(side=10.0))
    run("trunk bent 10 deg toward the lead side", setup=bend(side=-10.0))
    run("arms' and bat's masses and inertias x 0.9", arms_scale=0.9)
    run("arms' and bat's masses and inertias x 1.1", arms_scale=1.1)
    # the joint's place: the same motion, the moment taken about a point 3 cm away
    print("   the lumbar joint placed elsewhere (as placed motion; the torque about the moved point):")
    for label, d in (("3 cm higher along the trunk", (0, 0, 0.03)), ("3 cm lower", (0, 0, -0.03)), ("3 cm forward", (0.03, 0, 0)), ("3 cm back", (-0.03, 0, 0)),
                     ("3 cm toward the lead side", (0, 0.03, 0)), ("3 cm toward the rear side", (0, -0.03, 0))):
        best = (-1e9, 0)
        for r in rows:
            Rt_d = None
        vals = []
        for (t, st, acc, rep), r in zip(mot, rows):
            Rt = E.qR(s.pose(st, m.b["trunk"])[1])
            dw = E.mv(Rt, d)
            tq = E.sub(r["td"], E.cross(dw, r["Fu"]))
            vals.append((E.dot(tq, r["zt"]), t))
        a, ta = max(vals)
        res["joint " + label] = (a, ta, max(v for v, t in vals if t <= -0.060), None, None)
        print("   %-58s | %4.0f (%4.0f) | %4.0f |" % ("  " + label, a, ta * 1e3, res["joint " + label][2]))
    lo = min(v[0] for v in res.values()); hi = max(v[0] for v in res.values())
    print("\n4. SUMMARY. Axial torque from above: %.0f to %.0f N m over every case (10 ms case %.0f); from below with the measured forces alone: %.0f to %.0f"
          % (lo, hi, res["as placed, differenced over 10 ms"][0], min(v[3] for v in res.values() if v[3] is not None), max(v[3] for v in res.values() if v[3] is not None)))
    B.CURVE_OVERRIDE.clear()


if __name__ == "__main__":
    main()
