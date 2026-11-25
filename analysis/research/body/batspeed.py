"""Where the placed motion's bat speed goes (docs/body-motion.md 4.8; PLAN D31 phase 1, session 7, step 0).
A diagnosis: no fit, no drive, nothing in the model changed.

    python3 analysis/research/body/batspeed.py > sim/target/codex-probes/body3d/batspeed.out    (2 min; job queue)

The body moved exactly along Fortenbaugh's [4] mean curves (inverse3d.py's placed motion) has its sweet spot at
22.9 m/s at contact; [3] and Statcast have 29-34. This splits that speed into the hands' translation and the
bat's turning, and puts a number on each candidate for the difference:

1. [4]'s own curves read directly, with no body.
2. The placed motion split (hands; the bat's turn in azimuth and in elevation), as differenced and after the
   velocities are made consistent with the joints; over 5 and 10 ms; with the hands' path not smoothed.
3. [4]'s marker positions were passed through a 12 Hz Butterworth filter over the whole swing ([4] 3.4), so
   through the bat's impact with the ball. A low-pass filter run through a sudden loss of speed pulls the
   curve down BEFORE the loss. The true curve is modelled as smooth before contact, a step at contact, smooth
   after; the filter is applied; the pieces are fitted to [4]'s curve (linear least squares, the filter is
   linear). The filter's order and whether its cut-off was corrected for two passes are not stated in [4]: [E],
   four variants are run.
4. The placed motion on the curves with that smoothing undone (the bat's azimuth, elevation and lag).
5. The mean curve's peak against the mean of single swings' peaks ([4] Table 3).
6. What [4]'s own ball exit velocities need of the bat (its Eq. 14 and the oracle's collision).
7. Where on the bat each source measures: the bat's end against the sweet spot.
8. The body's dimensions: the shoulders' width.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "..", "oracle"))
import inverse3d as I3            # noqa: E402
import bbphys as P                # noqa: E402
from body3d import engine as E, body as B   # noqa: E402

C = B.C
D2R = math.pi / 180.0
H, M = 1.84, 92.0
U_SS, U_END, U_LEAD, U_MID = C.BAT_SS, C.BAT_L, B.GRIP_L, 0.5 * (B.GRIP_L + B.GRIP_R)
FS = 1000.0                 # Hz, the grid the filter is simulated on ([4] sampled at 300 Hz; the filter is set by its cut-off)
FC = 12.0                   # Hz, [4] section 3.4
WIN = 0.070                 # s either side of contact: the part of the curve the fitted pieces replace
# [4] Eq. 14 for the MIDDLE location (its reference level): BEV = 14.67 + 5.04e-4 x peak bat azimuth rate + 0.83 x pitch speed
EQ14 = (14.67, 5.04e-4, 0.83)
PITCH_RELEASE, PITCH_DIST = 25.0, 13.0   # m/s, m ([4] 3.3)
PEAK_MEAN_OF_SWINGS = 2486.0             # deg/s, [4] Table 3, MIDDLE: the mean of single swings' peaks
PEAK_OF_MEAN_CURVE = 2435.0              # deg/s, [4] 4.1.5: the peak of the mean curve


# ---- a Butterworth low-pass run forward and backward (zero lag), as motion-analysis software does ----
def _biquads(order, fc, fs):
    """second-order sections of a Butterworth low-pass of the given (even) order, bilinear transform."""
    k = math.tan(math.pi * fc / fs)
    out = []
    for i in range(order // 2):
        q = 1.0 / (2.0 * math.sin(math.pi * (2 * i + 1) / (2.0 * order)))
        n = 1.0 / (1.0 + k / q + k * k)
        out.append((k * k * n, 2 * k * k * n, k * k * n, 2 * (k * k - 1) * n, (1 - k / q + k * k) * n))
    return out


def _run(x, sos):
    for b0, b1, b2, a1, a2 in sos:
        y = [0.0] * len(x)
        x1 = x2 = x[0]
        y1 = y2 = x[0]              # started at rest on the first value
        for i, v in enumerate(x):
            y[i] = b0 * v + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2
            x2, x1 = x1, v
            y2, y1 = y1, y[i]
        x = y
    return x


def lowpass(x, order=2, fc=FC, corrected=True, fs=FS):
    """`order` forward and the same backward. corrected: the cut-off raised so the two passes together are 3 dB
    down at fc (Winter's correction); not corrected, the two passes together are 6 dB down at fc."""
    f = fc / (2.0 ** (1.0 / order) - 1.0) ** (1.0 / (2 * order)) if corrected else fc
    sos = _biquads(order, f, fs)
    pad = int(fs * 0.5)
    y = [x[0]] * pad + list(x) + [x[-1]] * pad
    y = _run(y, sos)
    y = _run(y[::-1], sos)[::-1]
    return y[pad:-pad]


def raw(name, t):
    """[4]'s digitised curve itself, whatever override a case has put on body.curve."""
    rows = B._CURVES.get(name)
    if rows is None:
        rows = B._CURVES[name] = C.load(name)
    return C.resample(rows, t, t, 1.0)[0]


FILTERS = (("2nd order each way, cut-off corrected", 2, True), ("2nd order each way, not corrected", 2, False),
           ("4th order each way, corrected", 4, True), ("4th order each way, not corrected", 4, False))


def _lstsq(cols, y):
    n = len(cols)
    A = [[sum(a * b for a, b in zip(cols[i], cols[k])) for k in range(n)] for i in range(n)]
    for i in range(n):
        A[i][i] *= 1.0 + 1e-10
    r = [sum(a * b for a, b in zip(cols[i], y)) for i in range(n)]
    for c in range(n):
        p = max(range(c, n), key=lambda k: abs(A[k][c]))
        A[c], A[p] = A[p], A[c]
        r[c], r[p] = r[p], r[c]
        for k in range(c + 1, n):
            f = A[k][c] / A[c][c]
            for j in range(c, n):
                A[k][j] -= f * A[c][j]
            r[k] -= f * r[c]
    x = [0.0] * n
    for c in range(n - 1, -1, -1):
        x[c] = (r[c] - sum(A[c][j] * x[j] for j in range(c + 1, n))) / A[c][c]
    return x


def unsmooth(name, order, corrected, kind, deg=2, win=WIN):
    """[4]'s curve `name` with the filter's smoothing through contact undone. kind = "rate": the true curve
    may step at contact (a rate curve); "angle": it is continuous and its slope may step (an angle curve);
    "none": one smooth piece, no break (the check that the data ask for a break). Inside +-win the true curve
    is a polynomial of degree `deg` before contact and another after; outside it is [4]'s own curve. Returns
    (true curve as a function of t, valid to +win; the pre-contact piece continued past contact as a function;
    the misfit RMS of the filtered model against [4] over +-win; the value and slope just before contact)."""
    t0, t1 = -0.40, 0.19
    n = int(round((t1 - t0) * FS)) + 1
    ts = [t0 + i / FS for i in range(n)]
    meas = [raw(name, t) for t in ts]
    inside = [abs(t) <= win for t in ts]
    base = [0.0 if ins else v for v, ins in zip(meas, inside)]
    basis = []
    if kind == "none":
        for d in range(deg + 2):
            basis.append([(t / win) ** d if ins else 0.0 for t, ins in zip(ts, inside)])
    else:
        for d in range(deg + 1):
            basis.append([(t / win) ** d if (ins and t < 0) else 0.0 for t, ins in zip(ts, inside)])
        for d in range(1 if kind == "angle" else 0, deg + 1):
            basis.append([(t / win) ** d if (ins and t >= 0) else 0.0 for t, ins in zip(ts, inside)])
        if kind == "angle":         # continuous at contact: the constant term is shared by both sides
            basis[0] = [1.0 if ins else 0.0 for ins in inside]
    fb = lowpass(base, order, FC, corrected)
    fcols = [lowpass(b, order, FC, corrected) for b in basis]
    sel = [i for i, t in enumerate(ts) if abs(t) <= win + 0.03]
    coef = _lstsq([[fc[i] for i in sel] for fc in fcols], [meas[i] - fb[i] for i in sel])
    fit = [fb[i] + sum(c * fc[i] for c, fc in zip(coef, fcols)) for i in range(n)]
    rms = math.sqrt(sum((fit[i] - meas[i]) ** 2 for i in sel) / len(sel))
    pre = coef[:deg + 1] if kind != "none" else coef

    def before(t, pre=tuple(pre), win=win):
        return sum(c * (t / win) ** d for d, c in enumerate(pre))

    def true(t, coef=tuple(coef), basis_n=len(basis)):
        if abs(t) > win:
            return raw(name, t)
        if kind == "none" or t < 0:
            return before(t)
        post = coef[deg + 1:]
        if kind == "angle":
            return coef[0] + sum(c * (t / win) ** (d + 1) for d, c in enumerate(post))
        return sum(c * (t / win) ** d for d, c in enumerate(post))
    slope0 = sum(d * c / win for d, c in enumerate(pre) if d == 1)
    return true, before, rms, before(0.0), slope0


# ---- the bat's speed split ----
def split(m, st, u=U_SS, hand=U_LEAD):
    """the speed of the bat's point u (m from the knob) as: the hands' translation, the bat's turning in
    azimuth, and in elevation, each as its part along the point's own velocity (they sum to the speed); and
    the hands' speed, the bat's azimuth and elevation rates (deg/s) and its roll about its own axis."""
    x, q, v, w = m.sys.pose(st, m.b["bat"])
    R = E.qR(q)
    bz = E.mv(R, (0.0, 0.0, 1.0))
    rh = E.scale(bz, hand - m.bat_ucm)
    vh = E.add(v, E.cross(w, rh))
    rho = u - hand
    dbz = E.cross(w, bz)
    ce = math.hypot(bz[0], bz[1])
    e_az = (-bz[1] / ce, bz[0] / ce, 0.0)
    e_el = E.cross(bz, e_az)
    az_rate = E.dot(dbz, e_az) / ce
    el_rate = E.dot(dbz, e_el)
    vs = E.add(vh, E.scale(dbz, rho))
    sp = E.norm(vs)
    uh = E.scale(vs, 1.0 / sp)
    return {"speed": sp, "hands": E.dot(vh, uh), "az": rho * az_rate * ce * E.dot(e_az, uh), "el": rho * el_rate * E.dot(e_el, uh),
            "hand_speed": E.norm(vh), "az_rate": az_rate / D2R, "el_rate": el_rate / D2R, "roll": E.dot(w, bz) / D2R,
            "turn": math.sqrt((az_rate * ce) ** 2 + el_rate ** 2) / D2R, "elev": math.asin(bz[2]) / D2R}


def placed(dt=0.010):
    """the placed motion's bat every dt from -250 ms: {t: (split as differenced, split made consistent with the
    joints, that split for the bat's end, and measured from the mid-point between the hands)}."""
    m = B.Model(H, M, contacts=False)
    s = m.sys
    s.stops, s.holds = [], []
    times, states, reps = I3.trajectory(m, dt)
    out = {}
    for i in range(1, len(times) - 1):
        t = times[i]
        if t < I3.T_A - 1e-9 or t > 1e-9:
            continue
        vel = I3.velocities(m, states[i - 1], states[i + 1], dt)
        st = list(states[i])
        for b_ in range(s.nb):
            st[13 * b_ + 7:13 * b_ + 13] = vel[b_]
        raw = split(m, st)
        vi, dv = I3.consistent(m, st, vel, 5)
        for b_ in range(s.nb):
            st[13 * b_ + 7:13 * b_ + 13] = vi[b_]
        out[round(t * 1e3)] = (raw, split(m, st), split(m, st, U_END), split(m, st, U_SS, U_MID))
    return out


def show(title, res, times=(-40, -30, -20, -10, 0)):
    print("   " + title)
    print("     t ms | sweet spot m/s = hands + azimuth turn + elevation turn | hands' speed | azimuth, elevation rate deg/s | the bat's end m/s"
          " | as differenced, before made consistent: sweet spot, azimuth rate")
    for t in times:
        if t not in res:
            continue
        raw, c, end, mid = res[t]
        print("     %4d | %5.1f = %4.1f + %4.1f + %4.1f | %4.1f | %5.0f %5.0f | %5.1f | %5.1f %5.0f"
              % (t, c["speed"], c["hands"], c["az"], c["el"], c["hand_speed"], c["az_rate"], c["el_rate"], end["speed"], raw["speed"], raw["az_rate"]))
    tb = max(res, key=lambda k: res[k][1]["speed"])
    print("     fastest: %.1f m/s at %d ms; at contact %.1f; the hands' fastest %.1f m/s at %d ms"
          % (res[tb][1]["speed"], tb, res[0][1]["speed"], max(r[1]["hand_speed"] for r in res.values()),
             max(res, key=lambda k: res[k][1]["hand_speed"])))


def main():
    rho_ss, rho_end = U_SS - U_LEAD, U_END - U_LEAD
    print("== 1. [4]'s own curves, no body. The sweet spot is %.3f m from the lead hand's grip, the bat's end %.3f m." % (rho_ss, rho_end))
    print("     t ms | azimuth rate: [4]'s rate curve, slope of its angle curve over +-10 ms | elevation, its slope | the bat's turning rate deg/s | x %.3f m" % rho_ss)
    for t in (-0.040, -0.030, -0.020, -0.010, 0.0):
        az = B.curve("bat_az_vel", t)
        az_s = (B.curve("bat_az", t + 0.010) - B.curve("bat_az", t - 0.010)) / 0.020
        el = B.curve("bat_elev", t)
        el_s = (B.curve("bat_elev", t + 0.010) - B.curve("bat_elev", t - 0.010)) / 0.020
        turn = math.hypot(az * math.cos(el * D2R), el_s)
        print("     %4.0f | %5.0f %5.0f | %5.1f %6.0f | %5.0f | %4.1f m/s" % (t * 1e3, az, az_s, el, el_s, turn, turn * D2R * rho_ss))
    print("     the digitised rate curve peaks at %.0f deg/s ([4]'s text: %.0f at -10 ms; %.0f at contact)"
          % (max(B.curve("bat_az_vel", -0.03 + 0.001 * k) for k in range(40)), PEAK_OF_MEAN_CURVE, 2233.0))

    print("\n== 2. the placed motion (inverse3d.py), split")
    base = placed(0.010)
    show("differenced over +-10 ms (inverse3d.py's own)", base)
    show("over +-5 ms", placed(0.005))
    keep = I3.HANDS_SMOOTH
    I3.HANDS_SMOOTH = 0
    show("+-10 ms, the hands' path not smoothed", placed(0.010))
    I3.HANDS_SMOOTH = keep
    I3.rotations_from_rates(True)
    show("+-10 ms, the pelvis's and trunk's rotation from [4]'s rate curves", placed(0.010))
    I3.rotations_from_rates(False)
    c0 = base[0][1]
    print("     at contact the hands' %.1f m/s gives the sweet spot %.1f: the hands move %.0f deg off the sweet spot's own direction"
          % (c0["hand_speed"], c0["hands"], math.acos(max(-1.0, min(1.0, c0["hands"] / c0["hand_speed"]))) / D2R))
    print("     measured from the mid-point between the hands instead of the lead grip: hands %.1f m/s (%.1f along), turning %.1f"
          % (base[0][3]["hand_speed"], base[0][3]["hands"], base[0][3]["az"] + base[0][3]["el"]))

    print("\n== 3. the 12 Hz filter run through the impact ([4] 3.4). The true curve: smooth, a break at contact, smooth; filtered; fitted to [4]'s curve over +-%.0f ms" % (WIN * 1e3))
    print("     the bat's azimuth RATE curve ([4]: %.0f deg/s at contact, peak %.0f):" % (B.curve("bat_az_vel", 0.0), max(B.curve("bat_az_vel", -0.03 + 0.001 * k) for k in range(40))))
    print("     %-40s | misfit RMS deg/s: no break, a step | true rate just before contact | the step | true rate at -10, -20, -40 ms" % "the filter [E]")
    rate_pre = []
    for label, order, corr in FILTERS:
        for deg in (1, 2):
            _, _, rms0, _, _ = unsmooth("bat_az_vel", order, corr, "none", deg)
            true, before, rms, v0, _ = unsmooth("bat_az_vel", order, corr, "rate", deg)
            print("     %-40s | %5.0f %5.0f | %5.0f | %6.0f | %5.0f %5.0f %5.0f   (pieces of degree %d)"
                  % (label, rms0, rms, v0, true(1e-6) - v0, before(-0.010), before(-0.020), before(-0.040), deg))
            rate_pre.append(v0)
    print("     so the bat's true azimuth rate just before contact is %.0f-%.0f deg/s where the filtered curve reads %.0f"
          % (min(rate_pre), max(rate_pre), B.curve("bat_az_vel", 0.0)))
    print("     the same from the bat's azimuth ANGLE curve (continuous, its slope breaks), and the elevation and the lag:")
    print("     %-40s | azimuth: slope just before contact (filtered curve's slope), angle at contact (filtered) | elevation: slope, angle | lag: slope, angle | misfit RMS deg" % "the filter [E]")
    for label, order, corr in FILTERS:
        r = [unsmooth(nm, order, corr, "angle", 2) for nm in ("bat_az", "bat_elev", "bat_lag")]
        sl = lambda nm: (B.curve(nm, 0.005) - B.curve(nm, -0.005)) / 0.010
        print("     %-40s | %5.0f (%5.0f) %6.1f (%5.1f) | %5.0f (%5.0f) %6.1f (%5.1f) | %6.0f (%6.0f) %5.1f (%5.1f) | %.2f %.2f %.2f"
              % (label, r[0][4], sl("bat_az"), r[0][3], B.curve("bat_az", 0.0), r[1][4], sl("bat_elev"), r[1][3], B.curve("bat_elev", 0.0),
                 r[2][4], sl("bat_lag"), r[2][3], B.curve("bat_lag", 0.0), r[0][2], r[1][2], r[2][2]))

    print("\n== 4. the placed motion on the curves with the smoothing undone (the bat's azimuth, elevation and lag; the piece before contact continued past it, so nothing is differenced across the impact)")
    undone = []
    for label, order, corr in FILTERS:
        B.CURVE_OVERRIDE.clear()
        for nm in ("bat_az", "bat_elev", "bat_lag"):
            true, before, rms, v0, s0 = unsmooth(nm, order, corr, "angle", 2)
            B.CURVE_OVERRIDE[nm] = (lambda t, true=true, before=before: before(t) if 0.0 <= t <= WIN else true(t))
        res = placed(0.010)
        show(label, res, times=(-20, -10, 0))
        undone.append(res[0][1]["speed"])
        B.CURVE_OVERRIDE.clear()
    print("     so with the impact's smoothing undone the placed motion has %.1f-%.1f m/s at contact, against %.1f as placed" % (min(undone), max(undone), c0["speed"]))

    print("\n== 5. the mean curve against single swings ([4] Table 3, MIDDLE): the mean of the swings' own peaks %.0f deg/s, the mean curve's peak %.0f: %.1f %% blunter"
          % (PEAK_MEAN_OF_SWINGS, PEAK_OF_MEAN_CURVE, 100 * (1 - PEAK_OF_MEAN_CURVE / PEAK_MEAN_OF_SWINGS)))
    print("     as bat speed at the sweet spot: %.1f m/s of the turning part" % ((PEAK_MEAN_OF_SWINGS - PEAK_OF_MEAN_CURVE) * D2R * rho_ss * math.cos(30 * D2R)))

    print("\n== 6. what [4]'s own batted balls need of the bat")
    bev = EQ14[0] + EQ14[1] * PEAK_MEAN_OF_SWINGS + EQ14[2] * PITCH_RELEASE
    print("     [4] Eq. 14 at the MIDDLE location, the mean pitch (%.1f m/s) and the mean peak bat rate: mean ball exit velocity %.1f m/s (%.0f mph), over every swing with a measurable batted ball"
          % (PITCH_RELEASE, bev, bev / 0.44704))
    rho_air = P.air_density()
    ball = P.integrate(P.State((0.0, PITCH_DIST, 1.5), (0.0, -PITCH_RELEASE, 1.0), (0.0, 0.0, 0.0)), rho_air, stop={"kind": "plane_y", "y": 0.0})
    v_plate = P.norm(ball["v"])
    print("     the pitch at the plate, by the oracle's flight from %.0f m: %.1f m/s" % (PITCH_DIST, v_plate))
    bat = P.bat_mass_properties()
    for e_n in (0.50, 0.55):
        def exit_speed(vb):
            # a level bat along +X swung toward +Y at the sweet spot, turning about a point 0.10 m beyond the knob; the ball met squarely
            a = (1.0, 0.0, 0.0)
            w = vb / (bat["L"] - 0.15 + 0.10)
            om = (0.0, 0.0, w)
            knob = (0.0, 0.0, 0.0)
            v_cm = (0.0, w * (bat["u_cm"] + 0.10), 0.0)
            br = (bat["L"] - 0.15, P.bat_radius(bat["L"] - 0.15) + P.RADIUS, 0.0)
            res = P.bat_ball_contact(br, (0.0, -v_plate, 0.0), (0.0, 0.0, 0.0), knob, a, v_cm, om, (0.0, 1.0, 0.0), e_n, 0.5)
            dv = P.norm(P.sub(P.add(res["V"], P.cross(res["W"], P.scale(a, bat["L"] - 0.15 - bat["u_cm"]))),
                              P.add(v_cm, P.cross(om, P.scale(a, bat["L"] - 0.15 - bat["u_cm"])))))
            return P.norm(res["v"]), dv, (res["W"][2] - w) / D2R
        lo, hi = 10.0, 45.0
        for _ in range(50):
            mid = 0.5 * (lo + hi)
            lo, hi = (mid, hi) if exit_speed(mid)[0] < bev else (lo, mid)
        need = 0.5 * (lo + hi)
        print("     normal restitution %.2f: a ball met squarely at the sweet spot leaves at %.1f m/s only if the sweet spot moves at %.1f m/s; that impact takes %.1f m/s off the sweet spot and %.0f deg/s off the turning rate of a bat nothing holds"
              % (e_n, bev, need, exit_speed(need)[1], -exit_speed(need)[2]))
    print("     every ball not met squarely needs more bat speed for the same exit velocity, so this is a floor for [4]'s batters' mean bat speed")

    print("\n== 7. where on the bat: the bat's end against the sweet spot, in the placed motion")
    for t in (-10, 0):
        print("     at %3d ms: the bat's end %.1f m/s, the sweet spot %.1f (%.2f of it)" % (t, base[t][2]["speed"], base[t][1]["speed"], base[t][1]["speed"] / base[t][2]["speed"]))
    r = base[-10][1]["speed"] / base[-10][2]["speed"]
    print("     [3] measured a marker on the bat's barrel end: 31 m/s at its fastest and 29 at contact are %.1f and %.1f m/s at the sweet spot by that ratio" % (31 * r, 29 * r))

    print("\n== 8. the body's dimensions: the shoulders %.3f of height from the trunk's axis (Winter) in place of %.3f" % (0.129, B.SHOULDER_HALF))
    keep = B.SHOULDER_HALF
    B.SHOULDER_HALF = 0.129
    show("wider shoulders", placed(0.010), times=(-10, 0))
    B.SHOULDER_HALF = keep


if __name__ == "__main__":
    main()
