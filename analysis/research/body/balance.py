#!/usr/bin/env python3
"""What keeps a standing, driven body on its path (docs/body-motion.md 5.3). Stdlib only.

The body over its support as an inverted pendulum in the line of the stride (lean theta, forward +),
the ankle torque the only drive, the centre of pressure kept inside the base of support: one foot,
0.12 m ahead of the ankle and 0.05 m behind it [E: a shoe's length about the ankle]; two feet in a
stance, 0.30 m either way [E].

Planned motion: hold upright, push the body into motion over 0.15 s (the rear leg's push), then a
'controlled fall' onto the front foot for the last 0.28 s (Fortenbaugh [4] 5.1). The planned torque is
worked back on the player's own model of his body. Three cases:
  open      the planned torque played back, with an error: the body model's mass 2 % off, or a lean
            0.5 deg off at the start, or the torque 1 % off;
  intrinsic the muscles' own stiffness and damping around the planned posture (no delay): Loram and
            Lakie 2002 put the ankle's intrinsic stiffness at about 0.9 of the toppling stiffness m g h
            (too weak alone); damping [E];
  reflex    intrinsic plus a delayed (100 ms [E: long-latency reflex 50-100 ms, voluntary 150+]) feedback
            on the error, gains [E] in the range that holds without oscillating.
Durations: 0.28 s (the batter's stride on the rear foot, [4] -621 -> -340 ms), 0.6 s (a batter's load
on two feet, then the stride on one), 1.5 s (a pitcher's leg lift and stride: on the pivot foot alone
throughout, the one-foot base). Over the rating range: stature 1.70-2.00 m, mass 75-120 kg.
What it does not test (docs 5.3): the frontal plane (a foot's width is about +-0.04 m), the lifted leg's
own motion, the landing's impulse, sensing noise in the reflex, and the reflex's gain read from the true
m g h. The fall threshold, 15 deg of lean, is [E].

Usage: python3 analysis/research/body/balance.py
"""
import math

G = 9.81


def run(T, H=1.84, Mb=92.0, mode="open", err="mass", delay=0.10, Kn=1.6, Dn=0.35, dt=0.0005, one_foot=None):
    m = Mb * (1 - 2 * 0.0137)            # body above the feet
    h = 0.53 * H                          # [E] COM height in an athletic stance (knees bent)
    I = m * h * h * 1.08                  # about the ankle: point mass plus the body's own (r_g ~ 0.28 h) [E]
    mgh = m * G * h
    # the player's own model of his body (mass 2 % low in the 'mass' error case)
    mm = m * (0.98 if err == "mass" else 1.0)
    Im, mghm = mm * h * h * 1.08, mm * G * h
    T1 = 0.28                                # the single-support fall: [4] foot off -> foot down
    T2 = T - T1                              # before it: two feet (the batter's load) or one (the pitcher's leg lift)
    if one_foot is None:
        one_foot = T >= 1.0                  # the 1.5 s delivery stands on the pivot foot throughout
    cop_b, cop_f = (0.05, 0.12) if (one_foot or T2 <= 0) else (0.30, 0.30)

    def tau_sched(t):
        """the planned ankle torque: hold upright, push the body into motion over the last 0.15 s before
        the fall (the rear leg's push, [4] trail GRFx about 20 % BW), then nothing: the fall is gravity's"""
        if t < T2 - 0.15:
            return 0.0
        if t < T2:
            return -0.08 * mghm * math.sin(math.pi * (t - (T2 - 0.15)) / 0.15)
        return 0.0

    # the plan: his model driven by the planned torque, from upright (T2 = 0: already moving, 0.12 rad/s)
    plan_tab = []
    pth, pthd = 0.0, (0.12 if T2 <= 0 else 0.0)
    tt = 0.0
    while tt < T + 0.25:
        plan_tab.append((pth, pthd))
        acc = (mghm * math.sin(pth) - tau_sched(tt)) / Im
        pth += pthd * dt + 0.5 * acc * dt * dt; pthd += acc * dt
        tt += dt

    def plan(t):
        return plan_tab[min(len(plan_tab) - 1, int(t / dt))] + (0.0,)

    def tau_plan(t):
        return tau_sched(t)

    th = LEAN * math.pi / 180 if err == "lean" else 0.0
    thd = 0.12 if T2 <= 0 else 0.0
    hist = []
    t = 0.0
    worst = 0.0
    fell = None
    while t < T:
        thp, thdp, _ = plan(t)
        tau = tau_plan(t) + (0.01 * mgh if err == "torque" else 0.0)
        if mode in ("intrinsic", "reflex"):
            tau += 0.9 * mgh * (th - thp) + 0.25 * mgh * 0.1 * (thd - thdp)
        if mode == "reflex":
            k = int(delay / dt)
            if len(hist) > k:
                tp, thpast, thdpast, thplan, thdplan = hist[-k]
                tau += Kn * mgh * (thpast - thplan) + Dn * mgh * 0.3 * (thdpast - thdplan)
        # centre of pressure under the foot: ankle torque bounded
        cb, cf = (cop_b, cop_f) if t < T2 else (0.05, 0.12)
        tau = max(-m * G * cb, min(m * G * cf, tau))   # CoP within the base of support
        # dynamics: I thdd = mgh sin(th) + tau  (tau > 0 pushes the body back)
        def f(a, b):
            return b, (m * G * h * math.sin(a) + (-tau)) / I
        a1, b1 = f(th, thd)
        a2, b2 = f(th + dt / 2 * a1, thd + dt / 2 * b1)
        a3, b3 = f(th + dt / 2 * a2, thd + dt / 2 * b2)
        a4, b4 = f(th + dt * a3, thd + dt * b3)
        th += dt / 6 * (a1 + 2 * a2 + 2 * a3 + a4); thd += dt / 6 * (b1 + 2 * b2 + 2 * b3 + b4)
        hist.append((t, th, thd, thp, thdp))
        t += dt
        e = abs(th - thp) * h
        if t <= T:
            worst = max(worst, e)
        if fell is None and abs(th) > 15 * math.pi / 180:
            fell = t
    return worst, fell


def main():
    print("COM error (cm, largest over the move) for the planned stance, push and 0.28 s fall; 'fell' = lean past 15 deg [E]")
    print("support: 0.28 s one foot; 0.6 s two feet then one; 1.5 s one foot throughout (the pitcher's pivot foot, CoP -0.05..+0.12 m)")
    for T in (0.28, 0.6, 1.5):
        for mode in ("open", "intrinsic", "reflex"):
            row = []
            for err in ("mass", "lean", "torque"):
                w, fell = run(T, mode=mode, err=err)
                row.append("%-6s %6.2f cm%s" % (err, w * 100, (" fell at %.2f s" % fell) if fell else ""))
            print("T %.2f s  %-9s  %s" % (T, mode, " | ".join(row)))
    print()
    print("rating range, 1.5 s, lean error 0.5 deg (cm), open loop -> reflex:")
    for H in (1.70, 1.84, 2.00):
        print("  H %.2f: " % H + "  ".join("M %3.0f kg %.1f -> %.2f" % (Mb, run(1.5, H=H, Mb=Mb, mode="open", err="lean")[0] * 100, run(1.5, H=H, Mb=Mb, mode="reflex", err="lean")[0] * 100) for Mb in (75, 92, 120)))
    print()
    print("reflex delay sweep (1.5 s, lean error): " + "  ".join("%3.0f ms %.2f cm" % (d * 1000, run(1.5, mode="reflex", err="lean", delay=d)[0] * 100) for d in (0.05, 0.10, 0.15, 0.20)))
    print("reflex gain sweep  (1.5 s, lean error): " + "  ".join("Kn %.1f %.2f cm" % (k, run(1.5, mode="reflex", err="lean", Kn=k)[0] * 100) for k in (0.3, 0.8, 1.6, 2.5)))
    print()
    print("larger lean errors, 1.5 s on one foot (cm; the reflex case):")
    for lean in (1.0, 2.0, 3.0):
        w, fell = run_lean(1.5, lean)
        print("  lean %.1f deg: %s" % (lean, ("fell at %.2f s" % fell) if fell else "%.2f cm" % (w * 100)))


def run_lean(T, lean_deg):
    """the reflex case with a start lean of lean_deg (one foot)."""
    global LEAN
    LEAN = lean_deg
    try:
        return run(T, mode="reflex", err="lean")
    finally:
        LEAN = 0.5


LEAN = 0.5


if __name__ == "__main__":
    main()
