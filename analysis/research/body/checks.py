#!/usr/bin/env python3
"""Integrator and model checks on the driven chain (docs/body-motion.md 8.1), with what each cannot see.

1. Energy: kinetic energy gained = work of every force (drives, stops, the ground's push and brake),
   both integrated with the same RK4 stages.
2. No forces from -100 ms: kinetic energy constant; the stops are checked to be inactive.
3. Internal drives only (no ground forces, no legs' torque): forward momentum constant (s is cyclic).
4. Step size: contact time and sweet-spot speed at dt 2, 1, 0.5, 0.25 ms.
5. The wrist's cock: the stiff one-sided stop (a device), the stop capped at 100, 50, 20 N m (20: the two
   wrists' measured holding strength, note 5, abstract only), and no stop at all. Contact at -8 deg, the
   bat's speed there, the largest lag and the lag at contact, the stop's peak torque and work, and where the
   bat meets the fixed ball point (breakdowns.py's test, with the bat's real length).
6. Segment table +-10 % (note 4: expect 10 % errors for athletes), the fitted drive unchanged; contact
   against the baseline's.
7. Angular momentum about the base point A = (0, s): its change equals the integral of the external yaw
   loads (the legs' couple from the ground, and -v_A x P), same RK4 stages. Internal pairs cancel, so a
   missing or wrong internal reaction shows here and not in 1.
8. The Euler-Lagrange identity at random states: M qdd + h against d/dt(dT/dqd) - dT/dq by central
   differences of the kinetic energy. Catches a wrong velocity term that does no work (skew-symmetric),
   which 1-3 cannot.
9. Mutations, to show what each check sees: a skew-symmetric velocity term, a dropped trunk reaction on
   the pelvis, a flipped brake sign.
Blind spots (printed at the end): masses and lengths themselves; a force direction that is wrong the same
way in the equations and the power; the ground model's realism (only its bookkeeping); 3D terms.
Usage: python3 analysis/research/body/checks.py
"""
import copy, math, random, sys, os
sys.path.insert(0, os.path.dirname(__file__))
from common import *
from chain import *
import chain as C
import fit, swing, breakdowns


def ang_check(sim, out):
    L0, L1 = sim.ang_mom(out["x"][0]), sim.ang_mom(out["x_end"])
    return L1 - L0, out["ang"]


def el_check(body, plan, n=20, eps=1e-5, seed=1):
    """max |M qdd + h - (d/dt dT/dqd - dT/dq)| over n random states, relative to |M qdd + h|."""
    sim = Sim(body, plan); sim.setup()
    rng = random.Random(seed)
    worst = 0.0
    for _ in range(n):
        q = [rng.uniform(0.2, 0.6)] + [rng.uniform(-2, 2) for _ in range(4)]
        qd = [rng.uniform(-1, 1)] + [rng.uniform(-30, 30) for _ in range(4)]
        qdd = [rng.uniform(-50, 50)] + [rng.uniform(-3000, 3000) for _ in range(4)]
        Mm, h = sim.mass_h(q + qd)
        lhs = [sum(Mm[i][j] * qdd[j] for j in range(5)) + h[i] for i in range(5)]

        def dT_dqd(qq, qqd, i):
            a = list(qqd); b = list(qqd); a[i] += eps; b[i] -= eps
            return (sim.ke(qq + a) - sim.ke(qq + b)) / (2 * eps)

        def dT_dq(qq, qqd, i):
            a = list(qq); b = list(qq); a[i] += eps; b[i] -= eps
            return (sim.ke(a + qqd) - sim.ke(b + qqd)) / (2 * eps)
        e = 1e-4
        qp = [q[k] + qd[k] * e for k in range(5)]; qdp = [qd[k] + qdd[k] * e for k in range(5)]
        qm = [q[k] - qd[k] * e for k in range(5)]; qdm = [qd[k] - qdd[k] * e for k in range(5)]
        for i in range(5):
            rhs = (dT_dqd(qp, qdp, i) - dT_dqd(qm, qdm, i)) / (2 * e) - dT_dq(q, qd, i)
            worst = max(worst, abs(lhs[i] - rhs) / max(1.0, abs(lhs[i])))
    return worst


def main():
    fit.M_ = fit.measured()
    body = Body()
    plan = fit.make_plan(swing.fitted(), body)
    sim = Sim(body, plan, dt=0.0005, record=True); out = sim.run(t_end=0.02)
    dke = sim.ke(out["x_end"]) - out["ke"][0]
    print("1. fitted swing: KE change %.4f J, work %.4f J, mismatch %.2e J (%.1e relative)" % (dke, out["work"], dke - out["work"], abs(dke - out["work"]) / dke))
    i = min(range(len(out["t"])), key=lambda k: abs(out["t"][k] + 0.10))
    st = [out["t"][i]] + list(out["x"][i])
    p2 = copy.deepcopy(plan); p2.start = st
    for n in CHANNELS:
        p2.ch[n][0] = 0.0
    body2 = Body(); body2.lag_stop = 10.0          # no wrist stop: nothing but inertia
    s2 = Sim(body2, p2, dt=0.0005, record=True); o2 = s2.run(t_end=0.0)
    ke = [s2.ke(x) for x in o2["x"]] + [s2.ke(o2["x_end"])]
    sep_max = max(abs(x[2] - x[1]) for x in o2["x"]) / D2R
    print("2. no forces from -100 ms: KE %.6f -> %.6f J, drift %.2e relative; largest trunk-pelvis angle %.1f deg (stop at %.0f: %s)" % (
        ke[0], ke[-1], abs(ke[-1] - ke[0]) / ke[0], sep_max, SEP_STOP / D2R, "inactive" if sep_max < SEP_STOP / D2R else "ACTIVE"))
    p3 = copy.deepcopy(plan); p3.start = st
    for n in ("legs", "rear", "lead"):
        p3.ch[n][0] = 0.0
    s3 = Sim(body2, p3, dt=0.0005, record=True); o3 = s3.run(t_end=0.0)

    def py(sm, x):
        return sum(m * vy for m, px_, py_, vx, vy in sm.vel_pts(x))
    a, b = py(s3, o3["x"][0]), py(s3, o3["x_end"])
    print("3. internal drives only from -100 ms: forward momentum %.6f -> %.6f N s (change %.2e)" % (a, b, b - a))
    print("4. step size: dt ms, contact ms, sweet-spot m/s")
    for dt in (0.002, 0.001, 0.0005, 0.00025):
        s4 = Sim(body, plan, dt=dt, record=True); o4 = s4.run(t_end=0.02)
        tc, xc = o4["contact"]
        print("   %.2f  %+.3f  %.4f" % (dt * 1000, tc * 1000, s4.sweet_spot_speed(xc)))

    print("5. the wrist's cock held by: contact ms, bat m/s at -8 deg, largest lag, lag at contact, stop's peak N m and work J; the fixed ball point: meet ms, u m, m/s")
    P = sim.sweet_spot(out["contact"][1])
    t_ref = breakdowns.meet(sim, out, P)[0]
    for label, cap, stop in (("stiff stop (one-sided)", None, True), ("stop capped 100 N m", 100.0, True), ("stop capped 50 N m", 50.0, True),
                             ("stop capped 20 N m (sourced)", 20.0, True), ("no stop", None, False)):
        b5 = Body(); b5.wrist_cap = cap
        if not stop:
            b5.lag_stop = 10.0
        s5 = Sim(b5, plan, dt=0.0005, record=True); o5 = s5.run(t_end=0.12)
        r = swing.summary(s5, {"t": o5["t"], "x": o5["x"], "contact": o5["contact"]}) if o5["contact"] else None
        stop_w = sum(st_ * (x[9] - x[8]) for st_, x in zip(o5["stop"], o5["x"])) * 0.0005      # the stop's work on the uncocking
        m = breakdowns.meet(s5, o5, P)
        ms = ("%+.1f %s %.1f" % ((m[0] - t_ref) * 1000, ("%.3f" % m[1]) if m[5] else "miss (%.3f, past the end)" % m[1], m[2])) if m else "never"
        if r:
            print("   %-30s %+.1f  %.2f  %.1f  %.1f  %5.0f %6.1f | %s" % (label, r["tc"] * 1000, r["ss"], r["lagmax"], r["lag_c"], max(o5["stop"]), stop_w, ms))
        else:
            print("   %-30s no -8 deg crossing | %s" % (label, ms))
    print("6. segment table +-10 %, the drive unchanged: contact ms against the baseline's, bat m/s")
    s0 = Sim(body, plan, dt=0.0005, record=True); o0 = s0.run(t_end=0.03); tc0 = o0["contact"][0]
    for name, fn in (("trunk I x1.1", lambda b: setattr(b, "I2", b.I2 * 1.1)), ("trunk I x0.9", lambda b: setattr(b, "I2", b.I2 * 0.9)),
                     ("pelvis I x1.1", lambda b: setattr(b, "I1", b.I1 * 1.1)), ("pelvis I x0.9", lambda b: setattr(b, "I1", b.I1 * 0.9)),
                     ("arms mass x1.1", lambda b: setattr(b, "arm_pts", [(m * 1.1, r) for m, r in b.arm_pts])),
                     ("arms mass x0.9", lambda b: setattr(b, "arm_pts", [(m * 0.9, r) for m, r in b.arm_pts]))):
        b6 = Body(); fn(b6)
        s6 = Sim(b6, plan, dt=0.0005, record=True); o6 = s6.run(t_end=0.03)
        tc, xc = o6["contact"]
        print("   %-15s %+.1f  %.2f" % (name, (tc - tc0) * 1000, s6.sweet_spot_speed(xc)))
    dL, imp = ang_check(sim, out)
    print("7. angular momentum about the base point, fitted swing: change %.4f N m s, external impulse %.4f, mismatch %.1e (%.1e relative)" % (
        dL, imp, dL - imp, abs(dL - imp) / abs(dL)))
    print("8. Euler-Lagrange identity, 20 random states: largest relative residual %.1e" % el_check(body, plan))

    print("9. mutations: which check sees each (energy mismatch relative | angular momentum mismatch relative | E-L residual)")
    orig_mh, orig_f = C.Sim.mass_h, C.Sim.forces

    def skew(self, x):
        Mm, h = orig_mh(self, x); qd = x[5:]
        h = list(h); h[3] += 0.5 * qd[2] * qd[4]; h[4] -= 0.5 * qd[2] * qd[3]
        return Mm, h

    def no_reaction(self, t, x, act):
        """the trunk's drive no longer reacts on the pelvis (a missing pair), booked consistently in the power"""
        q, qd = x[:5], x[5:]
        Mm, h = self.mass_h(x)
        tau = self.forces(t, q, qd, act)
        Q = [tau["rear"] - tau["lead"], tau["legs"], tau["core"] - tau["shoulder"], tau["shoulder"] - tau["wrist"], tau["wrist"]]
        return list(qd) + solve(Mm, [Q[i] - h[i] for i in range(5)]), tau

    def power_nr(self, x, tau):
        qd = x[5:]
        return (tau["rear"] - tau["lead"]) * qd[0] + tau["legs"] * qd[1] + tau["core"] * qd[2] + \
            tau["shoulder"] * (qd[3] - qd[2]) + tau["wrist"] * (qd[4] - qd[3])

    def flip_brake(self, t, q, qd, act):
        tau = orig_f(self, t, q, qd, act)
        tau["lead"] = -tau["lead"]          # wrong in the equations and the power alike
        return tau
    orig_eom, orig_pw = C.Sim.eom, C.Sim.power
    for label, mh, fo, eo, pw in (("none", orig_mh, orig_f, orig_eom, orig_pw), ("skew-symmetric velocity term", skew, orig_f, orig_eom, orig_pw),
                                  ("trunk reaction on the pelvis dropped", orig_mh, orig_f, no_reaction, power_nr),
                                  ("brake sign flipped", orig_mh, flip_brake, orig_eom, orig_pw)):
        C.Sim.mass_h, C.Sim.forces, C.Sim.eom, C.Sim.power = mh, fo, eo, pw
        try:
            s9 = Sim(body, plan, dt=0.0005, record=True); o9 = s9.run(t_end=-0.10)
            dk = s9.ke(o9["x_end"]) - o9["ke"][0]
            dL, imp = ang_check(s9, o9)
            el = el_check(body, plan, n=5)
            print("   %-38s %.1e | %.1e | %.1e" % (label, abs(dk - o9["work"]) / abs(dk), abs(dL - imp) / max(1e-9, abs(dL)), el))
        finally:
            C.Sim.mass_h, C.Sim.forces, C.Sim.eom, C.Sim.power = orig_mh, orig_f, orig_eom, orig_pw
    print("   (a sign wrong alike in the equations and the power, like the flipped brake, passes every check)")
    print("blind spots: segment masses and lengths; a force or torque direction wrong alike in the equations and the power;")
    print("whether the ground model is realistic (only its bookkeeping is checked); everything vertical and 3D.")


if __name__ == "__main__":
    main()
