"""The placed three-dimensional body, first runs (docs/body-motion.md 4.4; PLAN D31 phase 1, session 1).

    python3 analysis/research/body/placed3d.py > sim/target/codex-probes/body3d/placed3d.out     (about 30 s)

It places the body of analysis/oracle/body3d on [4]'s measured state, reports what could and could not
be matched, works back the muscles' level at the start, runs the body undriven, times a step, and runs
three HAND-SET drives. None of this is a fit: the hand-set runs only show what the body does when every
drive sits on its bound, which is where the flat chain's fit under the design's rules ended (4.3).
The fit under the rules needs the compiled probe (8.2) and is the next session's.
"""
import os
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "oracle"))
from body3d import engine as E, body as B, drive as Dr   # noqa: E402

H, M = 1.84, 92.0           # [4]'s mean batter
ONSETS = {"hip_ext_R": -0.245, "hip_flex_L": -0.245, "hip_rot_R": -0.245, "hip_rot_L": -0.245,   # [4]: pelvis rate rises from -245 ms
          "knee_ext_R": -0.245, "knee_ext_L": -0.170, "core": -0.220,
          "shoulder_L": -0.170, "shoulder_R": -0.170, "elbow_ext_L": -0.170, "elbow_ext_R": -0.170,  # the decision's earliest
          "wrist_L": -0.060, "wrist_R": -0.060}


def show(rows, step=1):
    print("    t ms | pelvis deg (deg/s) | trunk | bat azimuth | bat elev | lag | knee L, R | elbow L, R | sweet spot m/s | ground Fz L, R (N) | pelvis y m")
    for t, o, g in rows[::step]:
        print("  %5.0f | %6.1f (%5.0f) | %6.1f (%5.0f) | %7.1f (%5.0f) | %5.1f | %5.1f | %4.0f %4.0f | %4.0f %4.0f | %5.2f | %5.0f %5.0f | %+.3f"
              % (t * 1e3, o["pelvis_rot"], o["pelvis_rot_vel"], o["trunk_rot"], o["trunk_rot_vel"], o["bat_az"], o["bat_az_vel"],
                 o["bat_elev"], o["bat_lag"], o["lead_knee_flex"], o["trail_knee_flex"], o["lead_elbow_flex"], o["trail_elbow_flex"],
                 o["sweet_speed"], g["L"][2], g["R"][2], o["pelvis_y"]))


def placed(t0):
    m = B.Model(H, M)
    rep = {}
    st = B.place(m, t0, rep)
    print("\nplaced at %.0f ms: joints closed to %.1e m" % (t0 * 1e3, m.sys.residual(st)[0]))
    for k in sorted(rep["measured"]):
        d = rep["placed"][k] - rep["measured"][k]
        if k == "bat_az":
            d = (d + 180.0) % 360.0 - 180.0
        print("   %-18s placed - [4] = %+7.2f   (moved by closing the joints %+.2f)" % (k, d, rep["moved_by_closing"][k]))
    print("   trail knee bend %.1f deg: NOT placed, it follows from the rest ([4]: about 50 at foot down, 44 at -220 ms)" % rep["placed"]["trail_knee_flex"])
    print("   pelvis centre %.3f m above the ground" % m.sys.pose(st, m.b["pelvis"])[0][2])
    return m, st


def main():
    print("the three-dimensional placed body: H %.2f m, M %.1f kg" % (H, M))
    m, st = placed(-0.340)
    tone, rest = Dr.solve_tone(m, st, -0.340)
    print("muscles' level at the start (N m, each joint's vector in its parent's frame; size):")
    for j, v in tone.items():
        print("   %-11s %7.1f %7.1f %7.1f   %5.0f" % (j, v[0], v[1], v[2], E.norm(v)))
    print("   accelerations left: %.2f m/s^2, %.2f rad/s^2 (the ground carries [4]'s 93 %% of body weight at foot down)" % rest)
    Dr.add_damping(m, st)
    t = time.time()
    rows, s1 = Dr.simulate(m, st, -0.340, 0.0, every=20)
    dt = (time.time() - t) / 340
    print("\nundriven from foot down: only the starting muscle level and joint damping (no drive, no feedback)")
    show(rows)
    print("   joints' drift at the end %.1e m" % m.sys.residual(s1)[0])
    print("\nbenchmark: %.1f ms per RK4 step in Python (13 bodies, 49 joint rows, 8 contact points; includes recording)" % (dt * 1e3))
    bd = Dr.bounds(H, M)
    print("\nstrength bounds (N m):", ", ".join("%s %.0f" % (k, v) for k, v in bd.items()))
    for label, knee, hipx in (("every drive at its bound", 1.0, 1.0), ("knees' extension off, the rest at their bounds", 0.0, 1.0),
                              ("knees' and hips' extension and flexion off (rotators, trunk and arms at their bounds)", 0.0, 0.0)):
        m, st = placed(-0.250) if label.startswith("every") else (B.Model(H, M), None)
        if st is None:
            st = B.place(m, -0.250)
        plan = {n: (bd[n] * (knee if n.startswith("knee") else hipx if n[:7] in ("hip_ext", "hip_fle") else 1.0), ONSETS[n], Dr.RISE_MIN, 0.100) for n in bd}
        st = Dr.add_drives(m, st, plan)
        Dr.solve_tone(m, st, -0.250)
        Dr.add_damping(m, st)
        rows, s1 = Dr.simulate(m, st, -0.250, 0.060, every=5)
        print("\nHAND-SET, not a fit: %s; rises 50 ms; rules broken: %s" % (label, Dr.check_rules(m, plan) or "none"))
        show(rows, 5)
        for k, nm in (("pelvis_rot_vel", "pelvis"), ("trunk_rot_vel", "trunk"), ("bat_az_vel", "bat azimuth")):
            v, tp = max((o[k], t) for t, o, g in rows if t <= 0.0)
            print("   %s rate, largest up to 0 ms (5 ms samples): %.0f deg/s at %.0f ms" % (nm, v, tp * 1e3))
        for sd, nm in (("L", "lead"), ("R", "rear")):
            off = [t for t, o, g in rows if g[sd][2] <= 0.0]
            print("   %s foot: largest load %.0f N; %s" % (nm, max(g[sd][2] for t, o, g in rows),
                                                          "first unloaded at %.0f ms" % (off[0] * 1e3) if off else "never unloaded"))
        print("   work J: drives %.0f, stops %.0f, damping and starting level %.0f, ground %.0f; joints' drift %.1e m"
              % (s1[-4], s1[-3], s1[-2], s1[-1], m.sys.residual(s1)[0]))


if __name__ == "__main__":
    main()
