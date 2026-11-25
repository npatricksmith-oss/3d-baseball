#!/usr/bin/env python3
"""Integrator checks on the driven chain (docs/body-motion.md 8.1): energy, momentum, step size.

1. The fitted swing: kinetic energy gained = work of every force (drives, the wrist stop, the ground's
   push and brake), both integrated with the same RK4 stages.
2. No drive at all, from the fitted swing's state at -100 ms: kinetic energy constant.
3. Internal drives only (no ground forces, no legs' torque): forward momentum constant (s is cyclic).
4. Step size: contact time and sweet-spot speed at dt 2, 1, 0.5, 0.25 ms.
5. The wrist's cock stop (a device) replaced by the wrists' holding strength capped at 100, 50, 20 N m
   (note 5: about 10 N m per wrist in radial-ulnar deviation).
6. Segment table sensitivity (note 4: expect 10 % errors for athletes): trunk, pelvis and arm
   inertia and mass +-10 %, the fitted drive unchanged.
Usage: python3 analysis/research/body/checks.py
"""
import copy, math, sys, os
sys.path.insert(0, os.path.dirname(__file__))
from common import *
from chain import *
import fit, swing


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
    p2.ch["wrist"][0] = 0.0
    body2 = Body(); body2.lag_stop = 10.0          # no wrist stop: nothing but inertia
    s2 = Sim(body2, p2, dt=0.0005, record=True); o2 = s2.run(t_end=0.0)
    ke = [s2.ke(x) for x in o2["x"]] + [s2.ke(o2["x_end"])]
    print("2. no forces from -100 ms: KE %.6f -> %.6f J, drift %.2e relative" % (ke[0], ke[-1], abs(ke[-1] - ke[0]) / ke[0]))
    p3 = copy.deepcopy(plan); p3.start = st
    for n in ("legs", "rear", "lead"):
        p3.ch[n][0] = 0.0
    s3 = Sim(body2, p3, dt=0.0005, record=True); o3 = s3.run(t_end=0.0)

    def py(sm, x):
        q, qd = x[:5], x[5:]
        tot = 0.0
        for m, terms in sm.pts:
            vy = qd[0]
            for k, r, off in terms:
                vy += r * math.cos(q[k] + off) * qd[k]
            tot += m * vy
        return tot
    a, b = py(s3, o3["x"][0]), py(s3, o3["x_end"])
    print("3. internal drives only from -100 ms: forward momentum %.6f -> %.6f N s (change %.2e)" % (a, b, b - a))
    print("4. step size: dt ms, contact ms, sweet-spot m/s")
    for dt in (0.002, 0.001, 0.0005, 0.00025):
        s4 = Sim(body, plan, dt=dt, record=True); o4 = s4.run(t_end=0.02)
        tc, xc = o4["contact"]
        print("   %.2f  %+.3f  %.4f" % (dt * 1000, tc * 1000, s4.sweet_spot_speed(xc)))

    import chain as C
    print("5. wrist hold capped (N m): contact ms, bat m/s, largest lag, lag at contact")
    orig = C.Sim.forces
    for cap in (None, 100.0, 50.0, 20.0):
        def f(self, t, q, qd, act, cap=cap):
            tau = orig(self, t, q, qd, act)
            if cap is not None and q[3] - q[4] > self.b.lag_stop:
                stop = 2.0e4 * (q[3] - q[4] - self.b.lag_stop) + 40.0 * (qd[3] - qd[4])
                tau["wrist"] = tau["wrist"] - stop + max(-cap, min(cap, stop))
            return tau
        C.Sim.forces = f
        s5 = Sim(body, plan, dt=0.0005, record=True); o5 = s5.run(t_end=0.02)
        r = swing.summary(s5, o5)
        print("   %-5s %+.1f  %.2f  %.1f  %.1f" % (cap, r["tc"] * 1000, r["ss"], r["lagmax"], r["lag_c"]))
    C.Sim.forces = orig
    print("6. inertia +-10 %: contact ms, bat m/s")
    for name, fn in (("trunk I x1.1", lambda b: setattr(b, "I2", b.I2 * 1.1)), ("trunk I x0.9", lambda b: setattr(b, "I2", b.I2 * 0.9)),
                     ("pelvis I x1.1", lambda b: setattr(b, "I1", b.I1 * 1.1)),
                     ("arms mass x1.1", lambda b: setattr(b, "arm_pts", [(m * 1.1, r) for m, r in b.arm_pts])),
                     ("arms mass x0.9", lambda b: setattr(b, "arm_pts", [(m * 0.9, r) for m, r in b.arm_pts]))):
        b6 = Body(); fn(b6)
        s6 = Sim(b6, plan, dt=0.0005, record=True); o6 = s6.run(t_end=0.03)
        tc, xc = o6["contact"]
        print("   %-15s %+.1f  %.2f" % (name, tc * 1000, s6.sweet_spot_speed(xc)))


if __name__ == "__main__":
    main()
