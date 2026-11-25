"""The placed three-dimensional body: the two knees, the moving start, and what it does undriven
(docs/body-motion.md 4.4; PLAN D31 phase 1, sessions 1 and 2).

    python3 analysis/research/body/placed3d.py > sim/target/codex-probes/body3d/placed3d.out     (about 20 s)

1. The knees: what pelvis height each knee's measured bend asks for with both feet flat and the pelvis
   level, and how the heels and [4]'s pelvis obliquity close the gap; against [4]'s ground forces.
2. The start at -250 ms: every velocity the placed body has, and where it comes from in [4].
3. Undriven runs from that start (the starting muscle level and joint damping only), with the [E] parts of
   the start varied: does the weight come onto the front foot?
Session 1's hand-set runs (every drive on its bound) are gone: fit3d.py replaces them.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import probe3d as P                                    # noqa: E402
from probe3d import E, B, Dr                           # noqa: E402

H, M = 1.84, 92.0           # [4]'s mean batter
TIMES = (-0.340, -0.300, -0.250, -0.200, -0.170, -0.120, -0.093, -0.050, 0.0)


def knees():
    print("1. THE KNEES. Both feet flat, [4]'s pelvis travel read from the lead ankle:")
    print("   t ms | pelvis height asked by the lead knee, by the rear knee, pelvis level (m) | gap cm: level, with [4]'s obliquity | centre of mass along the stride: placed, from [4]'s forces (m from mid-stance)")
    m = B.Model(H, M)
    stride = B.STRIDE * H
    for t in TIMES:
        rep = {}
        st, _ = B.pose(m, t, "solve", rep)
        zo = rep["pelvis_z_from_each_knee_flat"]
        rep0 = {}
        B.pose(m, t, "flat", rep0)
        zl, zr = rep0["pelvis_z_from_each_knee_flat"]
        fz = (B.curve("lead_grf_z", t), B.curve("trail_grf_z", t))
        fy = B.curve("lead_grf_x", t) + B.curve("trail_grf_x", t)
        x, _ = B.com(m, st)
        # moment balance about the centre of mass, no change of angular momentum [E]: where it must be along the stride
        yc = (0.5 * stride * (fz[0] - fz[1]) + x[2] * fy) / (fz[0] + fz[1])
        print("  %5.0f | %.3f  %.3f | %+5.1f  %+5.1f | %+.3f  %+.3f" % (t * 1e3, zl, zr, (zr - zl) * 100, (zo[1] - zo[0]) * 100, x[1], yc))
    print("   placed with the heels and [4]'s obliquity (body3d.body.pose):")
    print("   t ms | lead heel up, rear heel up (deg) | pelvis obliquity ([4]) | lead knee placed ([4]) | rear knee placed ([4] text) | pelvis height m")
    for t in TIMES:
        rep = {}
        st, _ = B.pose(m, t, "solve", rep)
        o = rep["placed"]
        print("  %5.0f | %4.1f  %4.1f | %4.1f | %5.1f (%5.1f) | %5.1f (%5.1f) | %.3f"
              % (t * 1e3, rep["lead_heel_deg"], rep["rear_heel_deg"], B.curve_obliquity(t), o["lead_knee_flex"], B.curve("lead_knee_flex", t),
                 o["trail_knee_flex"], B.curve_trail_knee(t), m.sys.pose(st, m.b["pelvis"])[0][2]))


def start():
    t0 = -0.250
    print("\n2. THE START at %.0f ms: the placed body's velocities (slopes of [4]'s curves over +-%.0f ms)" % (t0 * 1e3, B.VEL_DT * 1e3))
    m = B.Model(H, M)
    rep = {}
    st = B.place(m, t0, rep)
    o = rep["placed"]
    x, v = B.com(m, st)
    print("   centre of mass: %+.2f m/s toward the pitcher, %+.2f toward the plate, %+.2f up; momentum toward the pitcher %.0f kg m/s"
          % (v[1], v[0], v[2], sum(m.sys.m) * v[1]))
    print("   pelvis forward %.2f m/s ([4]'s pelvis travel curve: slope over +-10 ms %.2f, over +-20 ms %.2f)"
          % (st[13 * m.b["pelvis"] + 8], B.curve_rate("pelvis_x_cm", t0, 0.01) / 100, B.curve_rate("pelvis_x_cm", t0, 0.02) / 100))
    for k, src in (("pelvis_rot_vel", "pelvis_rot"), ("trunk_rot_vel", "trunk_rot"), ("bat_az_vel", "bat_az")):
        print("   %-15s %+5.0f deg/s (slope of [4]'s angle curve %+.0f; [4]'s own rate curve, not used, %+.0f)"
              % (k, o[k], B.curve_rate(src, t0, B.VEL_DT), B.curve(k, t0)))
    sm, _ = B.pose(m, t0 - B.VEL_DT)
    sp, _ = B.pose(m, t0 + B.VEL_DT)
    om, op = m.observe(sm), m.observe(sp)
    print("   " + "; ".join("%s %+.0f deg/s" % (k, (op[k] - om[k]) / (2 * B.VEL_DT)) for k in
                            ("bat_elev", "bat_lag", "lead_elbow_flex", "trail_elbow_flex", "lead_knee_flex", "trail_knee_flex")))
    fw = m.sys.pose(st, m.b["foot_L"])[3]
    print("   lead heel coming down at %.0f deg/s (from the heel angles placed 10 ms either side); sweet spot %.2f m/s" % (E.norm(fw) / B.D2R, o["sweet_speed"]))
    print("   each foot's ground force at the start is [4]'s: lead %.0f %% BW up, %.0f toward the pitcher; rear %.0f up, %.0f toward the pitcher"
          % tuple(B.curve(k, t0) for k in ("lead_grf_z", "lead_grf_x", "trail_grf_z", "trail_grf_x")))


def undriven(label, t0=-0.250, **kw):
    m = B.Model(H, M)
    st = B.place(m, t0, **kw)
    _, v = B.com(m, st)
    tone, rest = Dr.solve_tone(m, st, t0, aux=list(m.start_aux))
    Dr.add_damping(m, st)
    rows, _ = P.simulate(m, st, t0, 0.0, every=5, aux=list(m.start_aux))
    BW = M * E.G
    y_lead = 0.5 * B.STRIDE * H
    print("\n   %s: centre of mass %+.2f m/s toward the pitcher at the start; largest starting joint torque %.0f N m"
          % (label, v[1], max(E.norm(t) for t in tone.values())))
    print("      t ms | pelvis travel m (model, [4]) | lead foot % BW (model, [4]) | rear foot (model, [4]) | lead knee, rear knee")
    for t, o, g in rows[::10]:
        print("     %5.0f | %+.3f %+.3f | %4.0f %4.0f | %4.0f %4.0f | %3.0f %3.0f"
              % (t * 1e3, o["pelvis_y"], y_lead + B.curve("pelvis_x_cm", t) / 100, 100 * g["L"][2] / BW, B.curve("lead_grf_z", t),
                 100 * g["R"][2] / BW, B.curve("trail_grf_z", t), o["lead_knee_flex"], o["trail_knee_flex"]))
    held = [100 * g["L"][2] / BW for t, o, g in rows]
    print("      lead foot: mean load %.0f %% BW over -250 to 0 ms ([4]: %.0f), over 50 %% BW for %d ms ([4]: from -170 ms on)"
          % (sum(held) / len(held), sum(B.curve("lead_grf_z", t) for t, o, g in rows) / len(rows), 5 * sum(1 for x in held if x > 50.0)))


def main():
    P.build()
    print("the three-dimensional placed body: H %.2f m, M %.1f kg" % (H, M))
    knees()
    start()
    print("\n3. UNDRIVEN from -250 ms: the starting muscle level and joint damping only (no drive, no feedback)")
    undriven("the start as placed")
    undriven("at rest (no starting velocities)", moving=False)
    undriven("velocities as slopes over +-5 ms", dt=0.005)
    undriven("velocities as slopes over +-3 ms", dt=0.003)
    undriven("session 1's placement (lead foot flat, pelvis low), moving", heel="flat")


if __name__ == "__main__":
    main()
