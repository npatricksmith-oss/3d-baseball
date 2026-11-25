"""Inverse dynamics of the three-dimensional body along Fortenbaugh's mean swing (docs/body-motion.md 4.4,
5.2; PLAN D31 phase 1, session 3). No search: the joint torques [4]'s motion needs, joint by joint, against
the design's bounds, and the turning moment the motion needs from the ground against what [4]'s forces give.

    python3 analysis/research/body/inverse3d.py > sim/target/codex-probes/body3d/inverse3d.out    (1 min; job queue)

How:
- The body is placed on [4]'s curves every `dt` from before -250 ms to after contact (body3d.body.pose: both
  knees, the heels, [4]'s pelvis obliquity). The hands' point is solved from the two elbows and the lag,
  continued in time; where the three cannot all be met (-85 to -30 ms, by up to 7 deg) the nearest point is
  taken, and the hands' path is then smoothed over +-HANDS_SMOOTH samples and the body re-placed on it, so
  the arms move smoothly (their angles then differ from [4]'s by what is printed).
- Velocities and accelerations are central differences of those postures (dt = 10 ms; 5 and 15 ms as the
  sensitivity to how the curves are differentiated).
- At each time the Python engine's equations are solved for the loads that give those accelerations:
  one torque vector per joint (39 numbers), plus what the measured ground forces leave unexplained: a force
  on the pelvis and a moment on it about the two horizontal axes (the reconstruction's inconsistency with
  [4]'s forces), and a turning moment about the vertical under the feet (the feet's free moments, which [4]
  does not report), shared by the feet in proportion to their vertical load. [4]'s ground forces act on each
  foot at its ball when its heel is up, else mid-sole [E: the centre of pressure is not reported].
- A joint torque is compared with its channel's bound, isometric and at the joint's own speed (the
  force-velocity law of 5.1). The two arms and the bat are a closed loop, so how a load is shared between the
  arms is not determined: the smallest torques are taken.
What this cannot see: anything the reconstruction gets wrong ([E]: the trunk's forward bend, no pelvis
tilt, the hands' path, the heels), and features shorter than about 4 dt.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "oracle"))
from body3d import engine as E, body as B, drive as Dr   # noqa: E402

H, M = 1.84, 92.0
HANDS_SMOOTH = 3            # samples either side (a triangular kernel), [E]
FREE_MOMENT_REACH = (0.05, 0.10)   # m [E]: a foot's free moment is at most mu x load x this (4.2 used 0.05)
RIDGE = 1e-9
T_A, T_B = -0.250, 0.0
# (label, joint, axis in the parent's frame, channel giving the bound or None, top-speed group)
AXES = [
    ("rear hip extension", "hip_R", (0, 1, 0), "hip_ext_R", "hip"), ("rear hip flexion", "hip_R", (0, -1, 0), "hip_flex_L", "hip"),
    ("rear hip abduction", "hip_R", (-1, 0, 0), "hip_abd_R", "hip"), ("rear hip adduction", "hip_R", (1, 0, 0), "hip_add_R", "hip"),
    ("rear hip rotators, pelvis toward the pitcher", "hip_R", (0, 0, -1), "hip_rot_R", "hip"),
    ("rear hip rotators, the other way", "hip_R", (0, 0, 1), "hip_rot_R", "hip"),
    ("lead hip extension", "hip_L", (0, 1, 0), "hip_ext_L", "hip"), ("lead hip flexion", "hip_L", (0, -1, 0), "hip_flex_L", "hip"),
    ("lead hip abduction", "hip_L", (1, 0, 0), "hip_abd_L", "hip"), ("lead hip adduction", "hip_L", (-1, 0, 0), "hip_add_L", "hip"),
    ("lead hip rotators, pelvis toward the pitcher", "hip_L", (0, 0, -1), "hip_rot_L", "hip"),
    ("lead hip rotators, the other way", "hip_L", (0, 0, 1), "hip_rot_L", "hip"),
    ("rear knee extension", "knee_R", (0, -1, 0), "knee_ext_R", "knee"), ("rear knee flexion", "knee_R", (0, 1, 0), "knee_flex_R", "knee"),
    ("lead knee extension", "knee_L", (0, -1, 0), "knee_ext_L", "knee"), ("lead knee flexion", "knee_L", (0, 1, 0), "knee_flex_L", "knee"),
    ("rear ankle plantar flexion", "ankle_R", (0, 1, 0), "ankle_pf_R", "ankle"), ("lead ankle plantar flexion", "ankle_L", (0, 1, 0), "ankle_pf_L", "ankle"),
    ("trunk rotation toward the pitcher", "lumbar", (0, 0, 1), "core", "core"), ("trunk rotation, the other way", "lumbar", (0, 0, -1), "core", "core"),
    ("lead shoulder about the trunk's axis", "shoulder_L", (0, 0, 1), "shoulder_L", "shoulder"),
    ("rear shoulder about the trunk's axis", "shoulder_R", (0, 0, 1), "shoulder_R", "shoulder"),
    ("lead elbow extension", "elbow_L", (0, -1, 0), "elbow_ext_L", "elbow"), ("rear elbow extension", "elbow_R", (0, -1, 0), "elbow_ext_R", "elbow"),
]
WHOLE = ("lumbar", "shoulder_L", "shoulder_R", "wrist_L", "wrist_R")     # joints whose whole torque vector's size is printed too


def allowed_at(bd, ch, speed, grp):
    """what channel ch's bound allows at the joint's shortening speed (rad/s): its measured strength-at-speed
    curve of body-motion 4.5, or the first law where it has none."""
    curve = Dr.CHANNELS[ch][4]
    if curve == "side":
        curve = Dr.HIP_SIDE_CURVE
    if curve is None:
        return bd[ch] * E.fv(speed / Dr.W_TOP[grp])
    return bd[ch] * E.fv_tab(Dr.CURVES[curve], speed)


def smooth(v, half):
    def ma(x):
        n = len(x)
        return [sum(x[max(0, i - half):min(n, i + half + 1)]) / (min(n, i + half + 1) - max(0, i - half)) for i in range(n)]
    return ma(ma(v)) if half > 0 else list(v)


def trajectory(m, dt, margin=3):
    """times and placed postures every dt from T_A - margin dt to T_B + margin dt, the hands' path continued
    and smoothed. Returns (times, states, per-time report)."""
    n0 = int(round((T_B - T_A) / dt))
    ks = list(range(-margin - HANDS_SMOOTH, n0 + margin + HANDS_SMOOTH + 1))
    times = [T_A + k * dt for k in ks]
    guess = {}
    g = None
    for k in [k for k in ks if k >= 0]:
        rep = {}
        B.pose(m, T_A + k * dt, "solve", rep, hands_guess=g)
        g = guess[k] = rep["hands_from_rear_shoulder"]
    g = guess[0]
    for k in [k for k in reversed(ks) if k < 0]:
        rep = {}
        B.pose(m, T_A + k * dt, "solve", rep, hands_guess=g)
        g = guess[k] = rep["hands_from_rear_shoulder"]
    comp = [smooth([guess[k][c] for k in ks], HANDS_SMOOTH) for c in range(3)]
    states, reps = [], []
    for i, k in enumerate(ks):
        rep = {}
        st, _ = B.pose(m, times[i], "solve", rep, hands_guess=(comp[0][i], comp[1][i], comp[2][i]), hands_solve=False)
        states.append(st); reps.append(rep)
    return times, states, reps


def velocities(m, sm, sp, dt):
    """each body's linear and angular velocity from the postures dt before and after."""
    out = []
    for i in range(m.sys.nb):
        o = 13 * i
        v = [(sp[o + c] - sm[o + c]) / (2 * dt) for c in range(3)]
        dq = E.qmul(tuple(sp[o + 3:o + 7]), E.qconj(tuple(sm[o + 3:o + 7])))
        if dq[0] < 0:
            dq = tuple(-c for c in dq)
        sn = math.sqrt(dq[1] ** 2 + dq[2] ** 2 + dq[3] ** 2)
        k = 2.0 * math.atan2(sn, dq[0]) / sn / (2 * dt) if sn > 1e-14 else 0.0
        out.append(v + [dq[1] * k, dq[2] * k, dq[3] * k])
    return out


def consistent(m, st, vec, level):
    """the smallest (mass-weighted) change of the bodies' velocities (level 5) or accelerations (level 6) that
    makes them exactly consistent with the joints at state st: differences of postures are consistent only to
    the differencing's own error, and a fast bat makes that error large."""
    s = m.sys
    X, R, V, Wv = s._kin(st)
    rows = s._rows(X, R, V, Wv)
    lin = [1.0 / mm for mm in s.m]
    A, per = s._assemble(rows, R, lin, [1.0] * s.nb)
    if level == 5:
        rhs = [-(sum(a * b for a, b in zip(r[1], vec[r[0]])) if r[0] >= 0 else 0.0)
               - (sum(a * b for a, b in zip(r[3], vec[r[2]])) if r[2] >= 0 else 0.0) for r in rows]
    else:
        rhs = [r[6] - (sum(a * b for a, b in zip(r[1], vec[r[0]])) if r[0] >= 0 else 0.0)
               - (sum(a * b for a, b in zip(r[3], vec[r[2]])) if r[2] >= 0 else 0.0) for r in rows]
    mu = E.chol_solve(A, rhs)
    out = [list(v) for v in vec]
    big = 0.0
    for i, lst in enumerate(per):
        for k, Jk, Yk in lst:
            for c in range(6):
                out[i][c] += mu[k] * Yk[c]
        big = max(big, max(abs(a - b) for a, b in zip(out[i], vec[i])))
    return out, big


def hip_generators(m, st, sd, tau):
    """a hip's torque (on the thigh; tau is in the pelvis's frame) along the THIGH's own three axes, which is
    how hip strength is tested and what each muscle group can give: about the thigh's side-to-side axis
    (extension +), about its front-to-back axis (toward the body's left +), about its own long axis (the
    rotators). Orthogonal components. Returns them, how much of the thigh's long and front-to-back axes lie
    along the pelvis's vertical, and the thigh's speeds about its three axes relative to the pelvis."""
    s = m.sys
    Rp = E.qR(s.pose(st, m.b["pelvis"])[1])
    Rth = E.qR(s.pose(st, m.b["thigh_" + sd])[1])
    tw = E.mv(Rp, tau)
    tl = E.mtv(Rth, tw)                                  # in the thigh's frame: x front-to-back, y side-to-side, z long
    wrel = E.mtv(Rth, E.sub(s.pose(st, m.b["thigh_" + sd])[3], s.pose(st, m.b["pelvis"])[3]))
    zp = E.mv(Rp, (0, 0, 1))
    # the second component is returned with its sign turned, so that + is adduction of the left thigh (as run() labels it)
    return (tl[1], -tl[0], tl[2]), (None, E.dot(E.mv(Rth, (1, 0, 0)), zp), E.dot(E.mv(Rth, (0, 0, 1)), zp)), (wrel[1], -wrel[0], wrel[2])


def momentum_z(m, st):
    """the whole body's angular momentum about the vertical through its centre of mass, and that centre."""
    s = m.sys
    xc, vc = B.com(m, st)
    hz = 0.0
    for i in range(s.nb):
        x, q, v, w = s.pose(st, i)
        R = E.qR(q)
        l = E.mtv(R, w)
        hw = E.mv(R, (s.I[i][0] * l[0], s.I[i][1] * l[1], s.I[i][2] * l[2]))
        hz += s.m[i] * ((x[0] - xc[0]) * (v[1] - vc[1]) - (x[1] - xc[1]) * (v[0] - vc[0])) + hw[2]
    return hz, xc


def ground(m, st, rep, t):
    """[4]'s ground force on each foot (world axes, N), its point of action in the foot's frame and in the world."""
    BW = M * E.G
    cf = B.C.DE_LEVA["foot"][2]
    Lf = m.len["foot"]
    out = {}
    for sd, nm, heel in (("L", "lead", rep["lead_heel_deg"]), ("R", "trail", rep["rear_heel_deg"])):
        f = (B.curve(nm + "_grf_y", t) / 100 * BW, B.curve(nm + "_grf_x", t) / 100 * BW, max(0.0, B.curve(nm + "_grf_z", t)) / 100 * BW)
        p = ((B.BALL_FWD - cf) * Lf, 0.0, m.sole_z) if heel > 0.5 else ((0.5 * B.BALL_FWD - cf) * Lf, 0.0, m.sole_z)
        x, q, _, _ = m.sys.pose(st, m.b["foot_" + sd])
        out[sd] = (f, p, E.add(x, E.mv(E.qR(q), p)))
    return out


def solve(m, st, acc, grf, free_at="feet"):
    """the loads that give the accelerations acc at state st. Returns ({joint: torque vector in its parent's
    frame}, pelvis residual force, pelvis residual moment (x, y), free moment about the vertical, misfit)."""
    s = m.sys
    nb = s.nb
    aux = []
    fz = {sd: grf[sd][0][2] for sd in "LR"}
    share = {sd: fz[sd] / max(1e-9, fz["L"] + fz["R"]) for sd in "LR"}
    base_ext = [(m.b["foot_" + sd], grf[sd][1], grf[sd][0], E.ZERO) for sd in "LR"]

    def accel(tones, ext):
        s.tones, s.ext = tones, base_ext + ext
        ds = s.deriv(0.0, st, aux)
        return [ds[13 * i + 7 + c] for i in range(nb) for c in range(6)]
    a0 = accel([], [])
    cols = []
    for j in Dr.JOINTS:
        a, ra, bb, rb = m.jp[j]
        for c in range(3):
            cols.append(accel([(j, a, bb, E.E3[c])], []))
    pel = m.b["pelvis"]
    for c in range(3):
        cols.append(accel([], [(pel, E.ZERO, E.E3[c], E.ZERO)]))
    for c in range(2):
        cols.append(accel([], [(pel, E.ZERO, E.ZERO, E.E3[c])]))
    if free_at == "feet":
        cols.append(accel([], [(m.b["foot_" + sd], E.ZERO, E.ZERO, (0.0, 0.0, share[sd])) for sd in "LR"]))
    else:                                           # the unexplained turning moment put on the pelvis directly
        cols.append(accel([], [(pel, E.ZERO, E.ZERO, (0.0, 0.0, 1.0))]))
    cols = [[x - y for x, y in zip(col, a0)] for col in cols]
    b = [x - y for x, y in zip([c for row in acc for c in row], a0)]
    n = len(cols)
    A = [[sum(x * y for x, y in zip(cols[i], cols[k])) for k in range(i + 1)] for i in range(n)]
    ridge = RIDGE * max(A[i][i] for i in range(n))
    for i in range(n):
        A[i][i] += ridge
    x = E.chol_solve(A, [sum(p * q for p, q in zip(cols[i], b)) for i in range(n)])
    fit = [sum(x[i] * cols[i][r] for i in range(n)) for r in range(len(b))]
    miss = max(abs(p - q) for p, q in zip(fit, b))
    tau = {j: tuple(x[3 * k:3 * k + 3]) for k, j in enumerate(Dr.JOINTS)}
    o = 3 * len(Dr.JOINTS)
    s.tones, s.ext = [], []
    return tau, tuple(x[o:o + 3]), tuple(x[o + 3:o + 5]), x[o + 5], miss


def rotations_from_rates(on):
    """[4]'s pelvis and trunk rotation ANGLE curves turn 12 and 13 deg further from -250 ms to contact than its
    own RATE curves integrate to (body-motion 4.4). on = True reads the two angles as the integral of the rate
    curves from their values at -250 ms; False restores the angle curves."""
    B.CURVE_OVERRIDE.clear()
    if not on:
        return
    for ang, rate in (("pelvis_rot", "pelvis_rot_vel"), ("trunk_rot", "trunk_rot_vel")):
        a0 = B.curve(ang, T_A)
        h = 0.001
        grid = {}
        acc = 0.0
        for k in range(0, 400):                  # forward from -250 ms
            grid[k] = acc
            acc += 0.5 * h * (B.curve(rate, T_A + k * h) + B.curve(rate, T_A + (k + 1) * h))
        acc = 0.0
        for k in range(0, -200, -1):
            grid[k] = acc
            acc -= 0.5 * h * (B.curve(rate, T_A + k * h) + B.curve(rate, T_A + (k - 1) * h))

        def f(t, a0=a0, grid=grid, h=h):
            x = (t - T_A) / h
            k = int(math.floor(x))
            return a0 + grid[k] + (x - k) * (grid[k + 1] - grid[k])
        B.CURVE_OVERRIDE[ang] = f


def run(dt, free_at="feet"):
    m = B.Model(H, M, contacts=False)
    s = m.sys
    s.stops, s.holds = [], []                  # the net joint torque: muscles and passive parts together
    s.baum_a = s.baum_b = 0.0
    times, states, reps = trajectory(m, dt)
    bd = Dr.bounds(H, M)
    idx = [i for i, t in enumerate(times) if T_A - 1e-9 <= t <= T_B + 1e-9]
    vel = {i: velocities(m, states[i - 1], states[i + 1], dt) for i in range(1, len(times) - 1)}
    rows = []
    for i in idx:
        st = list(states[i])
        vi, dv = consistent(m, st, vel[i], 5)
        for b_ in range(s.nb):
            st[13 * b_ + 7:13 * b_ + 13] = vi[b_]
        acc = [[(p - q) / (2 * dt) for p, q in zip(vel[i + 1][b_], vel[i - 1][b_])] for b_ in range(s.nb)]
        acc, da = consistent(m, st, acc, 6)
        grf = ground(m, st, reps[i], times[i])
        tau, fres, mres, free, miss = solve(m, st, acc, grf, free_at)
        hz, xc = momentum_z(m, st)
        stm, stp = list(states[i - 1]), list(states[i + 1])
        for b_ in range(s.nb):
            stm[13 * b_ + 7:13 * b_ + 13] = vel[i - 1][b_] if i - 1 in vel else vel[i][b_]
            stp[13 * b_ + 7:13 * b_ + 13] = vel[i + 1][b_] if i + 1 in vel else vel[i][b_]
        need = (momentum_z(m, stp)[0] - momentum_z(m, stm)[0]) / (2 * dt)
        given = sum((grf[sd][2][0] - xc[0]) * grf[sd][0][1] - (grf[sd][2][1] - xc[1]) * grf[sd][0][0] for sd in "LR")
        xp = s.pose(st, m.b["pelvis"])[0]
        given_pelvis = sum((grf[sd][2][0] - xp[0]) * grf[sd][0][1] - (grf[sd][2][1] - xp[1]) * grf[sd][0][0] for sd in "LR")
        ax = {}
        for label, j, axis, ch, grp in AXES:
            a, ra, bb, rb = m.jp[j]
            Ra = E.qR(s.pose(st, a)[1])
            wrel = E.mtv(Ra, E.sub(s.pose(st, bb)[3], s.pose(st, a)[3]))
            need_t = E.dot(tau[j], axis)
            speed = E.dot(wrel, axis)
            allowed = None if ch is None else allowed_at(bd, ch, speed, grp)
            ax[label] = (need_t, speed, allowed)
        for sd, nm in (("R", "rear"), ("L", "lead")):
            (c1, c2, c3), axes, sp = hip_generators(m, st, sd, tau["hip_" + sd])
            sgn = 1.0 if sd == "L" else -1.0              # about the floating axis, + is toward the body's left
            for label, need_t, speed, ch in ((nm + " hip: extension", c1, sp[0], "hip_ext_R"), (nm + " hip: flexion", -c1, -sp[0], "hip_flex_L"),
                                             (nm + " hip: adduction", sgn * c2, sgn * sp[1], "hip_add_" + sd), (nm + " hip: abduction", -sgn * c2, -sgn * sp[1], "hip_abd_" + sd),
                                             (nm + " hip: rotators turning the pelvis toward the pitcher", -c3, -sp[2], "hip_rot_" + sd),
                                             (nm + " hip: rotators the other way", c3, sp[2], "hip_rot_" + sd)):
                ax["GEN " + label] = (need_t, speed, allowed_at(bd, ch, speed, "hip"), bd[ch])
            ax["_yaw_" + sd] = (-tau["hip_" + sd][2], axes[2], axes[1])     # torque on the pelvis about its own vertical; how much of the rotators' and of the floating axis points along it
        o = m.observe(st)
        rows.append({"dv": dv, "da": da,"t": times[i], "tau": tau, "ax": ax, "fres": fres, "mres": mres, "free": free, "miss": miss, "hz": hz,
                     "need": need, "given": given, "given_pelvis": given_pelvis, "obs": o, "rep": reps[i],
                     "load": sum(grf[sd][0][2] for sd in "LR")})
    return m, bd, rows


def report(dt, rows, bd, full=True):
    print("\n== differentiated over dt = %.0f ms (velocities over +-dt, accelerations over +-2 dt)" % (dt * 1e3))
    if full:
        print("   the reconstruction against [4] (model, [4]):  t ms | pelvis rate | trunk rate | bat azimuth rate | lead elbow, rear elbow, lag | hands' misfit taken out by smoothing")
        for r in rows[::max(1, int(round(0.025 / dt)))]:
            t, o = r["t"], r["obs"]
            print("     %5.0f | %5.0f %5.0f | %5.0f %5.0f | %5.0f %5.0f | %3.0f %3.0f, %3.0f %3.0f, %3.0f %3.0f | %s"
                  % (t * 1e3, o["pelvis_rot_vel"], B.curve("pelvis_rot_vel", t), o["trunk_rot_vel"], B.curve("trunk_rot_vel", t),
                     o["bat_az_vel"], B.curve("bat_az_vel", t), o["lead_elbow_flex"], B.curve("lead_elbow_flex", t),
                     o["trail_elbow_flex"], B.curve("trail_elbow_flex", t), o["bat_lag"], B.curve("bat_lag", t),
                     " ".join("%+.1f" % c for c in r["rep"]["hands_solve_residual_deg"])))
    # the turn about the vertical
    w = lambda a, b_: [r for r in rows if a - 1e-9 <= r["t"] < b_ - 1e-9]
    print("   THE TURN ABOUT THE VERTICAL through the centre of mass (N m s; the moment the motion needs is the rate of the body's angular momentum):")
    for a, b_ in ((-0.250, 0.0), (-0.170, 0.0), (-0.250, -0.093), (-0.170, -0.093)):
        sel = w(a, b_)
        need = sum(r["need"] for r in sel) * dt
        given = sum(r["given"] for r in sel) * dt
        print("     %4.0f to %4.0f ms: the motion needs %5.1f, [4]'s ground forces give %5.1f, left for the feet's free moments %5.1f"
              % (a * 1e3, b_ * 1e3, need, given, need - given))
    pk = max(rows, key=lambda r: r["need"])
    pg = max(rows, key=lambda r: r["given"])
    pf = max(rows, key=lambda r: abs(r["need"] - r["given"]))
    cap = [0.8 * pf["load"] * x for x in FREE_MOMENT_REACH]
    print("     peak moment needed %.0f N m at %.0f ms; peak given by [4]'s forces %.0f N m at %.0f ms (about the pelvis's vertical instead: %.0f)"
          % (pk["need"], pk["t"] * 1e3, pg["given"], pg["t"] * 1e3, max(r["given_pelvis"] for r in rows)))
    print("     largest free moment needed %.0f N m at %.0f ms; friction allows %.0f-%.0f N m there (mu 0.8 x load x %.2f-%.2f m [E])"
          % (pf["need"] - pf["given"], pf["t"] * 1e3, cap[0], cap[1], FREE_MOMENT_REACH[0], FREE_MOMENT_REACH[1]))
    print("     the body's angular momentum about the vertical: %.1f kg m^2/s at -250 ms, %.1f at -93, %.1f at 0"
          % (rows[0]["hz"], min(rows, key=lambda r: abs(r["t"] + 0.093))["hz"], rows[-1]["hz"]))
    print("   made consistent with the joints: velocities changed by up to %.2f (m/s or rad/s), accelerations by up to %.0f (m/s^2 or rad/s^2)"
          % (max(r["dv"] for r in rows), max(r["da"] for r in rows)))
    print("   what [4]'s forces leave unexplained on the pelvis (the reconstruction's inconsistency): force up to %.0f N, moment about the horizontal axes up to %.0f N m; accelerations not met: %.1e"
          % (max(E.norm(r["fres"]) for r in rows), max(math.hypot(*r["mres"]) for r in rows), max(r["miss"] for r in rows)))
    print("   JOINT TORQUES NEEDED against the bounds (N m). Over = needed more than the bound allows at the joint's speed at that instant.")
    print("     %-46s | peak needed (ms) | bound | allowed at that speed (rad/s) | needed/allowed | ms over the allowed, -250 to 0 | peak before -93 ms" % "joint and direction")
    table = {}
    for label, j, axis, ch, grp in AXES:
        pk = max(rows, key=lambda r: r["ax"][label][0])
        need, speed, allowed = pk["ax"][label]
        over = sum(1 for r in rows if r["ax"][label][2] is not None and r["ax"][label][0] > r["ax"][label][2] + 1e-9) * dt * 1e3
        early = max(r["ax"][label][0] for r in rows if r["t"] <= -0.093 + 1e-9)
        table[label] = (need, pk["t"], None if ch is None else bd[ch], allowed, over, early)
        if need < 5.0:
            continue
        print("     %-46s | %5.0f (%4.0f) | %5s | %5s (%+5.1f) | %5s | %4.0f | %5.0f"
              % (label, need, pk["t"] * 1e3, "-" if ch is None else "%.0f" % bd[ch], "-" if allowed is None else "%.0f" % allowed, speed,
                 "-" if not allowed else "%.2f" % (need / allowed), over, early))
    print("   THE HIPS along the thigh's own axes (extension-flexion about its side-to-side axis, abduction-adduction about its front-to-back axis, the rotators about its long axis):")
    for label in [k for k in rows[0]["ax"] if k.startswith("GEN ")]:
        pk = max(rows, key=lambda r: r["ax"][label][0])
        need, speed, allowed, bound = pk["ax"][label]
        over = sum(1 for r in rows if r["ax"][label][0] > r["ax"][label][2] + 1e-9) * dt * 1e3
        early = max(r["ax"][label][0] for r in rows if r["t"] <= -0.093 + 1e-9)
        table[label] = (need, pk["t"], bound, allowed, over, early)
        if need >= 5.0:
            print("     %-58s | %5.0f (%4.0f) | %5.0f | %5.0f (%+5.1f) | %5s | %4.0f | %5.0f"
                  % (label[4:], need, pk["t"] * 1e3, bound, allowed, speed, "-" if not allowed else "%.2f" % (need / allowed), over, early))
    for sd, nm in (("R", "rear"), ("L", "lead")):
        pk = max(rows, key=lambda r: r["ax"]["_yaw_" + sd][0])
        y, c3z, c2z = pk["ax"]["_yaw_" + sd]
        print("     %s hip's torque on the pelvis about the pelvis's own vertical: peak %.0f N m at %.0f ms; there the thigh's long axis is %.2f along that vertical and its front-to-back axis %.2f"
              % (nm, y, pk["t"] * 1e3, c3z, c2z))
    for j in WHOLE:
        pk = max(rows, key=lambda r: E.norm(r["tau"][j]))
        print("     %-46s | %5.0f (%4.0f)   the whole torque vector's size" % (j, E.norm(pk["tau"][j]), pk["t"] * 1e3))
    return table


def main():
    out = {}
    for dt in (0.010, 0.005, 0.015):
        m, bd, rows = run(dt)
        out[dt] = report(dt, rows, bd, full=(dt == 0.010))
    print("\n== the same at dt = 10 ms with the unexplained turning moment put on the pelvis directly, not under the feet")
    m, bd, rows = run(0.010, "pelvis")
    out["pelvis"] = report(0.010, rows, bd, full=False)
    print("\n== the same at dt = 10 ms with the pelvis's and trunk's rotation read from [4]'s RATE curves (integrated from -250 ms) instead of its angle curves")
    rotations_from_rates(True)
    m, bd, rows = run(0.010)
    out["rates"] = report(0.010, rows, bd, full=True)
    rotations_from_rates(False)
    print("\n== sensitivity: peak torque needed (N m) at dt = 5, 10, 15 ms | unexplained moment on the pelvis instead (dt 10) | rotations from [4]'s rate curves (dt 10) | bound")
    for label in out[0.010]:
        v = [out[k][label][0] for k in (0.005, 0.010, 0.015, "pelvis", "rates")]
        if max(v) >= 5.0:
            b_ = out[0.010][label][2]
            print("     %-62s | %5.0f %5.0f %5.0f | %5.0f | %5.0f | %s" % (label.replace("GEN ", ""), v[0], v[1], v[2], v[3], v[4], "-" if b_ is None else "%.0f" % b_))


if __name__ == "__main__":
    main()
