"""Which way the ball leaves the practised swing's bat, and the same swing early and late (docs/body-motion.md 4.15, part 1; PLAN D44).

    python3 analysis/research/body/contact3d.py --static                 a bat square to the ball's line, and 10 and 20 deg early and late
    python3 analysis/research/body/contact3d.py --swing <result.json>    the kept swing's ball, and the swing 10 and 20 ms early and late
    python3 analysis/research/body/contact3d.py --check                  the detection and the directions against worked cases

THE GAME'S COLLISION MODEL, NOT A SECOND ONE. The impulse is analysis/oracle/bbphys.py's (physics-spec 8.1 with 8.2.1's
tangential restitution; the checking oracle's copy, held to the Rust by golden cases). Around it, from the same spec:
section 7's contact detection (the signed gap between ball and bat sampled every DT and bisected; the normal from the
bat's axis to the ball's centre, tilted by the taper) and 8.2's normal restitution (bb_physics::bat::BatBallParams's
numbers, named below). The bat is the game's free 34-inch bat moving as the body's bat moves at that instant.

NOTHING HERE IS GIVEN TO THE PRACTICE SEARCH. The search's score asks the bat square to the ball's line and moving along
it as the sweet spot reaches the ball's place; this file says where the ball then goes by the collision, as a check.

The early and late swings are WHAT-IFS BY REPLAY (indicative): the practised plan is not changed and not re-tuned; the
pitch is the same pitch arriving `shift` earlier or later against it. The ball moves along its line at its speed where
it is met (its slowing and its drop over 20 ms, under 1 m/s and 2 mm, are left out).
"""
import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "..", "oracle"))
import bbphys as BP                                    # noqa: E402

# physics-spec 8.2 (bb_physics::bat::BatBallParams::default): the normal restitution's numbers
SS_FROM_TIP, U_TAPER, F_TIP, F_TAPER, F_HANDLE = 0.15, 22.0 * BP.INCH, 0.75, 0.27, 0.35
E_REF, V_REF, S_E, MU = 0.460, 70.0, 0.001, 0.50
BAT_L = 34.0 * BP.INCH
DT = 5e-5                     # s: the gap's sampling (the spec's 0.1 ms, halved)
SPIN_RPM, SPIN_TILT_DEG = 2290.0, 30.0     # the game's average four-seamer (sim-cli pitch --type FF --n 200 --seed 1): 2,290 rpm at 1:00


def e_n(u, cn_abs):
    """physics-spec 8.2: e_peak(|c_n|) x f(u), as BatBallParams::e_n has it."""
    uss = BAT_L - SS_FROM_TIP
    if u >= uss:
        s = (u - uss) / SS_FROM_TIP
        f = 1.0 - (1.0 - F_TIP) * s * s
    else:
        s = (uss - u) / (uss - U_TAPER)
        f = 1.0 - (1.0 - F_TAPER) * s * s
    return min(0.9, max(0.05, (E_REF - S_E * (cn_abs - V_REF)) * max(f, F_HANDLE)))


def pitch_spin(nb):
    """the average four-seamer's spin vector, rad/s: across the ball's travel, its Magnus force 30 deg from straight up
    toward the pitcher's arm side (1:00 from his view; AGENTS.md's frame: a pitch moving -Y with backspin has w along -X)."""
    w = SPIN_RPM * BP.RPM
    t = math.radians(SPIN_TILT_DEG)
    return (-w * math.cos(t), 0.0, -w * math.sin(t))


def gap_at(b, knob, a):
    """physics-spec 7, step 2: (the signed gap between the ball's and the bat's surfaces, the station u on the bat)."""
    u = min(BAT_L, max(0.0, BP.dot(BP.sub(b, knob), a)))
    return BP.norm(BP.sub(b, BP.add(knob, BP.scale(a, u)))) - BP.bat_radius(u) - BP.RADIUS, u


def normal_at(b, knob, a, u):
    """physics-spec 7, step 4: from the bat into the ball, the radial direction tilted by the taper's slope."""
    radial = BP.unit(BP.sub(b, BP.add(knob, BP.scale(a, u))))
    du = 1e-5
    slope = (BP.bat_radius(min(BAT_L, u + du)) - BP.bat_radius(max(0.0, u - du))) / (min(BAT_L, u + du) - max(0.0, u - du))
    return BP.unit(BP.sub(radial, BP.scale(a, slope)))


def strike(ball_at, ball_v, ball_w, bat_at, t_lo, t_hi, mu=MU):
    """The ball met by the bat between t_lo and t_hi, by the game's rules. ball_at(t): its centre; bat_at(t): (knob,
    axis, the bat's own centre of mass's velocity, its angular velocity). Returns a dict: hit (False: no touch; then the
    nearest approach, its time and station), t, u (station from the knob, m), spray_deg (+ toward first base: for a
    right-hander the opposite field), launch_deg, exit_mps, bat_mps (the sweet spot's), square_deg (the bat's axis from
    square to the ball's line seen from above, + = not yet round, as the score reads it), where (the ball's centre)."""
    n = int(round((t_hi - t_lo) / DT))
    prev, near = None, (1e9, None, None)
    for k in range(n + 1):
        t = t_lo + k * DT
        knob, a, vcm, om = bat_at(t)
        g, u = gap_at(ball_at(t), knob, a)
        if g < near[0]:
            near = (g, t, u)
        if prev is not None and prev[1] > 0.0 >= g:
            lo, hi = prev[0], t
            for _ in range(40):
                mid = 0.5 * (lo + hi)
                knob, a, vcm, om = bat_at(mid)
                gm, u = gap_at(ball_at(mid), knob, a)
                lo, hi = (mid, hi) if gm > 0.0 else (lo, mid)
            t = hi
            knob, a, vcm, om = bat_at(t)
            b = ball_at(t)
            g, u = gap_at(b, knob, a)
            nrm = normal_at(b, knob, a, u)
            bat = BP.bat_mass_properties()
            c = BP.add(knob, BP.scale(a, bat["u_cm"]))
            r1 = BP.scale(nrm, -BP.RADIUS)
            r2 = BP.sub(BP.sub(b, BP.scale(nrm, BP.RADIUS)), c)
            cn = BP.dot(BP.sub(BP.add(ball_v, BP.cross(ball_w, r1)), BP.add(vcm, BP.cross(om, r2))), nrm)
            if cn >= 0.0:
                return dict(hit=False, gap=g, t=t, u=u, why="touching but not closing")
            res = BP.bat_ball_contact(b, ball_v, ball_w, knob, a, vcm, om, nrm, e_n(u, abs(cn)), mu, ss_from_tip=SS_FROM_TIP)
            v = res["v"]
            lh = math.hypot(ball_v[0], ball_v[1])
            sq = math.degrees(math.asin(max(-1.0, min(1.0, (a[0] * ball_v[0] + a[1] * ball_v[1]) / max(1e-12, math.hypot(a[0], a[1]) * lh)))))
            return dict(hit=True, t=t, u=u, where=b, spray_deg=math.degrees(math.atan2(v[0], v[1])), launch_deg=math.degrees(math.atan2(v[2], math.hypot(v[0], v[1]))),
                        exit_mps=BP.norm(v), bat_mps=res["bat_speed"], square_deg=sq, cn=abs(cn), e_t=res["e_t"], impulses=res["impulses"])
        prev = (t, g)
    return dict(hit=False, gap=near[0], t=near[1], u=near[2], why="the surfaces never touch")


def static(yaw_deg, bat_mps=31.0, tilt_deg=25.0, ball_mps=37.98, descent_deg=5.27, across_deg=1.62, attack_deg=8.0, turn_dps=2000.0, path="arc", offset_m=0.0, mu=MU, spin=True):
    """A bat met by the average fastball with the bat turned `yaw_deg` past square (+ = further round, an early swing
    for a right-hander: barrel toward the pitcher), the barrel `tilt_deg` below level, the sweet spot moving at
    `bat_mps` up at `attack_deg`. path "arc": the bat moves square to its own length, as a turning bat does, so an
    early bat also moves toward the pull side; "line": it moves along the ball's line whatever its turn. The bat turns
    about the vertical at `turn_dps`. The ball's centre is on the bat's axis's level but for `offset_m` above it.
    Returns strike()'s dict. A right-hander: the barrel toward +X (first base) when square."""
    d, ac = math.radians(descent_deg), math.radians(across_deg)
    nb = (math.cos(d) * math.sin(ac), -math.cos(d) * math.cos(ac), -math.sin(d))
    y, tl, at = math.radians(yaw_deg), math.radians(tilt_deg), math.radians(attack_deg)
    # square: the axis level-square to the ball's line, barrel toward first base; then turned y further round (counter-clockwise from above)
    hx, hy = math.cos(ac), math.sin(ac)                        # level, square to the ball's line, toward +X
    ax = (math.cos(tl) * (hx * math.cos(y) - hy * math.sin(y)), math.cos(tl) * (hx * math.sin(y) + hy * math.cos(y)), -math.sin(tl))
    fwd = (-nb[0], -nb[1], 0.0)
    fwd = BP.unit(fwd)
    if path == "arc":
        fwd = (fwd[0] * math.cos(y) - fwd[1] * math.sin(y), fwd[0] * math.sin(y) + fwd[1] * math.cos(y), 0.0)
    vss = (bat_mps * math.cos(at) * fwd[0], bat_mps * math.cos(at) * fwd[1], bat_mps * math.sin(at))
    om = (0.0, 0.0, math.radians(turn_dps))
    uss = BAT_L - SS_FROM_TIP
    ucm = BP.bat_mass_properties()["u_cm"]
    ss0 = (0.0, 0.0, 0.8)
    t_c = 0.004
    # the bat as a rigid body with this velocity and turn; the ball timed so that its centre and the sweet spot would coincide at t_c
    def bat_at(t):
        ang = math.radians(turn_dps) * (t - t_c)
        c_, s_ = math.cos(ang), math.sin(ang)
        a = (ax[0] * c_ - ax[1] * s_, ax[0] * s_ + ax[1] * c_, ax[2])
        ss = BP.add(ss0, BP.scale(vss, t - t_c))
        knob = BP.sub(ss, BP.scale(a, uss))
        vcm = BP.add(vss, BP.cross(om, BP.scale(a, ucm - uss)))
        return knob, a, vcm, om
    bv = BP.scale(nb, ball_mps)
    up = BP.unit(BP.sub((0.0, 0.0, 1.0), BP.scale(nb, nb[2])))
    ball_at = lambda t: BP.add(BP.add(ss0, BP.scale(up, offset_m)), BP.scale(bv, t - t_c))
    return strike(ball_at, bv, pitch_spin(nb) if spin else BP.ZERO, bat_at, 0.0, 2.0 * t_c, mu=mu)


def swing_bat(ctx, u):
    """the practised swing's bat as a function of time, from a run recorded at every step: bat_at(t) for strike(),
    and the records. Between two steps (0.25 ms) the knob's and the tip's places and the velocities go evenly."""
    fine = ctx.run(u, every=1)
    m, bi = ctx.m, ctx.m.b["bat"]
    ucm_own = BP.bat_mass_properties()["u_cm"]
    pts = []
    for rec in fine:
        k0, _ = m.bat_point(rec[1], 0.0)
        tip, _ = m.bat_point(rec[1], BAT_L)
        _, vcm = m.bat_point(rec[1], ucm_own)
        pts.append((rec[0], k0, tip, vcm, tuple(rec[1][13 * bi + 10:13 * bi + 13])))
    t0, h = pts[0][0], pts[1][0] - pts[0][0]

    def bat_at(t):
        q = (t - t0) / h
        i = min(len(pts) - 2, max(0, int(math.floor(q))))
        f = q - i
        mix = lambda x, y: tuple((1.0 - f) * p + f * r for p, r in zip(x, y))
        knob, tip = mix(pts[i][1], pts[i + 1][1]), mix(pts[i][2], pts[i + 1][2])
        return knob, BP.unit(BP.sub(tip, knob)), mix(pts[i][3], pts[i + 1][3]), mix(pts[i][4], pts[i + 1][4])
    return bat_at, fine


def swing_ball(ctx, u, shifts_ms=(0.0,), log=print):
    """The kept swing's ball by the collision, and the same swing with the pitch arriving `shift` later against it
    (+ = the swing is that much EARLY). For each: where the bat meets the ball's line (m toward the pitcher from the
    practised ball's place), which way the ball leaves, the bat's speed. Returns {shift_ms: strike()'s dict + extras}."""
    import practice3d as PR
    bat_at, fine = swing_bat(ctx, u)
    tx, p, v, i, came = ctx.contact(fine)
    nb = ctx.nb
    bv = BP.scale(nb, PR.BALL_SPEED)
    out = {}
    for sh in shifts_ms:
        tb = tx + sh * 1e-3                                 # when the ball's centre is at the practised place
        ball_at = lambda t, tb=tb: BP.add(ctx.Pc, BP.scale(bv, t - tb))
        r = strike(ball_at, bv, pitch_spin(nb), bat_at, max(fine[0][0], tx - 0.045), min(fine[-1][0], tx + 0.045))
        r["shift_ms"] = sh
        if r["hit"]:
            r["out_m"] = -BP.dot(BP.sub(r["where"], ctx.Pc), nb)          # toward the pitcher along the ball's line
            r["t_from_practised_ms"] = (r["t"] - tx) * 1e3
            r["u_from_sweet_cm"] = (r["u"] - (BAT_L - SS_FROM_TIP)) * 100
        out[sh] = r
        if r["hit"]:
            log("   the swing %s: the ball is met %+.1f cm %s along its line, %+.1f ms from the practised contact, %+.1f cm from the sweet spot along the bat, the bat %+.1f deg from square at %.2f m/s (%.1f mph); THE BALL LEAVES %+.1f deg (%s), %+.1f deg up, at %.1f m/s (%.0f mph)"
                % ("on time" if sh == 0 else "%.0f ms %s" % (abs(sh), "EARLY" if sh > 0 else "LATE"), r["out_m"] * 100, "further out" if r["out_m"] >= 0 else "deeper", r["t_from_practised_ms"], r["u_from_sweet_cm"],
                   r["square_deg"], r["bat_mps"], r["bat_mps"] / BP.MPH, r["spray_deg"], side(r["spray_deg"]), r["launch_deg"], r["exit_mps"], r["exit_mps"] / BP.MPH))
        else:
            log("   the swing %s: NO CONTACT (%s; the surfaces come within %.1f cm, %.1f cm from the sweet spot along the bat)"
                % ("on time" if sh == 0 else "%.0f ms %s" % (abs(sh), "EARLY" if sh > 0 else "LATE"), r["why"], r["gap"] * 100, (r["u"] - (BAT_L - SS_FROM_TIP)) * 100))
    return out


def side(spray):
    """a right-hander's field for a spray angle (+ toward first base), in the bands of analysis/statcast/savant_contact_by_spray.py."""
    return ("pulled hard" if spray < -27 else "pulled" if spray < -9 else "up the middle" if spray < 9 else "the other way" if spray < 27 else "far the other way") + (", foul" if abs(spray) > 45 else "")


def check():
    ok = True
    flat = dict(bat_mps=30.0, tilt_deg=0.0, descent_deg=0.0, across_deg=0.0, attack_deg=0.0, turn_dps=0.0, path="line", spin=False)
    bat = BP.bat_mass_properties()
    uss = BAT_L - SS_FROM_TIP
    b = uss - bat["u_cm"]
    rr = BP.MASS * (1.0 / bat["M"] + b * b / bat["I_cm"])
    en = e_n(uss, 67.98)
    q = (en - rr) / (1.0 + rr)
    want = q * 37.98 + (1.0 + q) * 30.0
    # 1. a level bat square to a level, spinless ball that comes straight down the Y line, met dead centre at the sweet spot:
    #    physics-spec 8.1's head-on case, the ball straight back at q v_ball + (1 + q) v_bat with the bat's effective mass at
    #    the sweet spot. Not exact with the game's friction: the bat's recoil (it turns about its own centre, 11.7 cm from the
    #    sweet spot) drags the ball's surface along the bat, and the ball leaves under half a degree and 0.01 m/s off that.
    r = static(0.0, **flat)
    good = r["hit"] and abs(r["spray_deg"]) < 0.5 and abs(r["launch_deg"]) < 1e-9 and abs(r["exit_mps"] - want) < 0.01 and abs(r["u"] - uss) < 1e-9
    print("head-on at the sweet spot: the ball leaves %+.3f deg at %.4f m/s; worked q v + (1 + q) V = %.4f (q %.4f): %s" % (r["spray_deg"], r["exit_mps"], want, q, "PASS" if good else "FAIL"))
    ok = ok and good
    # 2. the detection: the surfaces touch when the centres are one bat radius and one ball radius apart: closing head-on at
    #    30 + 37.98 m/s, that is (R_bat + R) / 67.98 s before the centres would coincide
    t_touch = 0.004 - (BP.bat_radius(uss) + BP.RADIUS) / 67.98
    good = abs(r["t"] - t_touch) < 2e-9
    print("the touch: %.6f ms before the centres would meet; worked (R_bat + R) / closing speed %.6f: %s" % ((0.004 - r["t"]) * 1e3, (0.004 - t_touch) * 1e3, "PASS" if good else "FAIL"))
    ok = ok and good
    # 3. a level bat turned y past square sends a straight level ball to the pull side (-X), one turned y short of square to the
    #    other side. With no friction the ball keeps its speed along the bat's face and leaves beyond the face's own turn by
    #    atan((v sin y) / (its speed out along the normal)), worked here with the head-on law on the normal parts. The worked
    #    figure leaves out that the ball touches 1 ms before the centres would meet, a few mm along the bat from the sweet
    #    spot (toward the tip on one side, the handle on the other), so the two sides differ from it by 0.1-0.2 deg: held to 0.3
    rp, rm = (static(yv, mu=0.0, **flat) for yv in (10.0, -10.0))
    y = math.radians(10.0)
    out_n = q * 37.98 * math.cos(y) + (1.0 + q) * 30.0 * math.cos(y)
    want_deg = 10.0 + math.degrees(math.atan2(37.98 * math.sin(y), out_n))
    good = rp["hit"] and rm["hit"] and abs(rp["spray_deg"] + want_deg) < 0.3 and abs(rm["spray_deg"] - want_deg) < 0.3
    print("a bat 10 deg past square, no friction: %+.3f deg (pulled; worked %+.3f); 10 deg short of square: %+.3f: %s" % (rp["spray_deg"], -want_deg, rm["spray_deg"], "PASS" if good else "FAIL"))
    ok = ok and good
    rp, rm = (static(yv, **flat) for yv in (10.0, -10.0))
    good = -want_deg < rp["spray_deg"] < -10.0 and 10.0 < rm["spray_deg"] < want_deg
    print("the same with the game's friction: %+.2f and %+.2f deg, each between the bat's turn and the frictionless angle: %s" % (rp["spray_deg"], rm["spray_deg"], "PASS" if good else "FAIL"))
    ok = ok and good
    return ok


def main():
    a = sys.argv[1:]
    arg = lambda k, d: a[a.index(k) + 1] if k in a else d
    if "--check" in a:
        sys.exit(0 if check() else 1)
    if "--static" in a:
        for tilt, bat_mps, attack in ((25.0, 31.0, 8.0), (32.0, 31.0, 8.0), (59.0, 26.25, 2.8)):
            print("the average fastball (38.0 m/s, coming down 5.3 deg, 2,290 rpm) met at the sweet spot by a bat at %.1f m/s (%.0f mph), BARREL %.0f DEG BELOW LEVEL, rising %.1f deg, turning 2,000 deg/s; + turn = further round than square (an early swing)" % (bat_mps, bat_mps / BP.MPH, tilt, attack))
            print("   %-26s %s" % ("the bat's turn past square", "the ball leaves: as the bat moves square to its own length | as it moves along the ball's line"))
            for y in (-20.0, -10.0, -5.0, 0.0, 5.0, 10.0, 20.0):
                ra, rl = (static(y, path=p_, tilt_deg=tilt, bat_mps=bat_mps, attack_deg=attack) for p_ in ("arc", "line"))
                print("   %+6.0f deg                 %+6.1f deg (%s), %+.0f up, %.0f mph | %+6.1f deg (%s), %+.0f up, %.0f mph"
                      % (y, ra["spray_deg"], side(ra["spray_deg"]), ra["launch_deg"], ra["exit_mps"] / BP.MPH, rl["spray_deg"], side(rl["spray_deg"]), rl["launch_deg"], rl["exit_mps"] / BP.MPH))
            r0 = static(0.0, tilt_deg=tilt, bat_mps=bat_mps, attack_deg=attack, spin=False)
            print("   square, the pitch without its spin: %+.1f deg, %+.0f up" % (r0["spray_deg"], r0["launch_deg"]))
        return
    if "--swing" in a:
        import practice3d as PR
        import probe3d as P
        P.build()
        d = json.load(open(arg("--swing", "")))
        ctx = PR.Ctx(**d["options"])
        print("%s: the sweet spot at %.2f m/s at the practised contact" % (os.path.basename(arg("--swing", "")), d["sweet_speed"]))
        out = swing_ball(ctx, ctx.u_of(d["plan"]), shifts_ms=[float(x) for x in arg("--shifts", "0,10,20,-10,-20").split(",")])
        if "--out" in a:
            json.dump({str(k): {kk: vv for kk, vv in v.items()} for k, v in out.items()}, open(arg("--out", ""), "w"), indent=1)
        return
    print(__doc__)


if __name__ == "__main__":
    main()
