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
   curve down BEFORE the loss. 3a: session 7's reading (free polynomial pieces inside +-70 ms, which jump at
   the window's edges; the jump is printed). 3b (session 7 fix, review P1-2): the true rate as a rise of a
   stated SHAPE [E] (straight, accelerating u^k, joined in slope too, levelling off) joined to [4]'s curve at
   -70 and +70 ms, a step at contact free or capped at a free bat's loss, fitted through the filter (linear
   least squares for each shape). 3c: the elevation and lag undone as session 7 did. The filter's order and
   whether its cut-off was corrected for two passes are not stated in [4]: [E], four variants are run.
4. The placed motion on each shape that fits, with the azimuth undone alone, with the lag, and with the
   elevation and lag as session 7 undid them: the sweet spot's speed and the attack angle at contact.
5. The mean curve's peak against the mean of single swings' peaks ([4] Table 3).
6. What [4]'s own ball exit velocities need of the bat (its Eq. 14 and the oracle's collision).
   Also: whether a held bat loses less at the impact (the grip as a spring [E], a rigidly held handle) and how
   much the filter could change the hands' speed near contact.
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
    down at fc (Winter's correction for two passes, (2^(1/2) - 1)^(1/(2 order)); session 7 had 2^(1/order) in
    place of 2^(1/2), the same for order 2 and 14.8 Hz in place of 13.4 for order 4: review P3-1); not corrected,
    the two passes together are 6 dB down at fc."""
    f = fc / (2.0 ** 0.5 - 1.0) ** (1.0 / (2 * order)) if corrected else fc
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
    # the pieces are free inside the window and [4]'s curve outside, so the "true" curve may jump at +-win (review
    # P1-2): reported beside every figure from this function
    jump = (before(-win) - raw(name, -win), true(win) - raw(name, win))
    return true, before, rms, before(0.0), slope0, jump


# ---- the smoothing undone with pieces joined to [4]'s curve at the window's edges (session 7 fix, review P1-2) ----
# The true azimuth rate is [4]'s curve outside +-WIN. Inside: before contact a rise of a stated SHAPE [E] from [4]'s
# value at -WIN to r0 just before contact; a step s at contact; after contact a piece from r0 - s back to [4]'s value
# at +WIN (straight, plus one bump term). Every shape joins [4]'s curve in value at both edges; "accelerating, joined
# in slope" also in slope at -WIN. Given the shape, the model is linear in (r0, s, bump), so each is a linear least
# squares through the (linear) filter. Shapes: (label, kind, parameter); a parameter of None is scanned and the best
# misfit kept.
SHAPES = (("straight rise", "line", 0.0),
          ("accelerating, u^2", "pow", 2.0),
          ("accelerating, u^3", "pow", 3.0),
          ("accelerating, u^k, k fitted", "pow", None),
          ("accelerating, u^k, joined in slope too, k fitted", "powc1", None),
          ("rise, then flat for the last 10 ms", "plateau", 0.010),
          ("rise, then flat for the last 20 ms", "plateau", 0.020),
          ("rise, then flat for the last 30 ms", "plateau", 0.030))
K_SCAN = [1.0 + 0.1 * i for i in range(51)]     # k from 1 to 6
FIT_RMS_OK = 50.0            # deg/s [E]: a shape "fits" [4]'s curve if its misfit RMS over +-100 ms is under this,
                             # a fifth of the fit's band on this curve (250, fit3d.BAND); stated before the run


def _grid():
    t0, t1 = -0.40, 0.19
    n = int(round((t1 - t0) * FS)) + 1
    ts = [t0 + i / FS for i in range(n)]
    return ts, [raw("bat_az_vel", t) for t in ts]


def shape_rate(kind, par, a, a_slope, r0, t, win=WIN):
    """the rise before contact (t < 0; continued past contact for t >= 0, the way the bat would have gone on
    without the ball) from [4]'s value a at -win to r0 at contact."""
    u = (t + win) / win
    if kind == "line":
        return a + (r0 - a) * u
    if kind == "pow":
        return a + (r0 - a) * u ** par
    if kind == "powc1":
        return a + a_slope * win * (u - u ** par) + (r0 - a) * u ** par
    if kind == "plateau":
        return a + (r0 - a) * min(1.0, u * win / (win - par))
    raise ValueError(kind)


def _fit_shape(ts, meas, kind, par, order, corrected, cap, win=WIN, cache=None):
    """one shape, one filter: returns a dict of the fitted numbers and the filtered result."""
    a, b = raw("bat_az_vel", -win), raw("bat_az_vel", win)
    a_slope = (raw("bat_az_vel", -win + 0.005) - raw("bat_az_vel", -win - 0.005)) / 0.010
    n = len(ts)
    base, c_r0, c_s, c_p = list(meas), [0.0] * n, [0.0] * n, [0.0] * n
    for i, t in enumerate(ts):
        if -win <= t < 0.0:
            g = shape_rate(kind, par, 0.0, 0.0, 1.0, t, win)        # the coefficient of r0
            base[i] = shape_rate(kind, par, a, a_slope, 0.0, t, win)
            c_r0[i] = g
        elif 0.0 <= t <= win:
            u2 = t / win
            base[i] = b * u2
            c_r0[i], c_s[i], c_p[i] = 1.0 - u2, -(1.0 - u2), u2 * (1.0 - u2)
    key = (kind, par, order, corrected)
    if cache is not None and key in cache:
        fb, fr, fs, fp = cache[key]
    else:
        fb, fr, fs, fp = (lowpass(x, order, FC, corrected) for x in (base, c_r0, c_s, c_p))
        if cache is not None:
            cache[key] = (fb, fr, fs, fp)
    sel = [i for i, t in enumerate(ts) if abs(t) <= win + 0.03]
    y = [meas[i] - fb[i] for i in sel]
    coef = _lstsq([[fr[i] for i in sel], [fs[i] for i in sel], [fp[i] for i in sel]], y)
    capped = False
    if cap is not None and coef[1] > cap:
        capped = True
        y2 = [yy - cap * fs[i] for yy, i in zip(y, sel)]
        c2 = _lstsq([[fr[i] for i in sel], [fp[i] for i in sel]], y2)
        coef = [c2[0], cap, c2[1]]
    r0, s, p = coef
    filt = [fb[i] + r0 * fr[i] + s * fs[i] + p * fp[i] for i in range(n)]
    rms = math.sqrt(sum((filt[i] - meas[i]) ** 2 for i in sel) / len(sel))
    slope_in = {"line": (r0 - a) / win, "pow": (r0 - a) / win if par == 1.0 else 0.0, "powc1": a_slope,
                "plateau": (r0 - a) / (win - par) if kind == "plateau" else 0.0}[kind]
    return {"kind": kind, "par": par, "r0": r0, "step": s, "bump": p, "rms": rms, "capped": capped, "filtered": filt,
            "a": a, "a_slope": a_slope, "slope_jump": slope_in - a_slope,
            "rate": lambda t, r0=r0, kind=kind, par=par: shape_rate(kind, par, a, a_slope, r0, t, win)}


def shape_of(ts, x):
    """a curve's shape as [4]'s is read: the peak and its time within +-50 ms, the fraction lost by contact, the
    values at +15 and +35 ms."""
    w = [(v, t) for t, v in zip(ts, x) if -0.05 < t < 0.05]
    pk, tp = max(w)
    at = lambda tt: x[min(range(len(ts)), key=lambda i: abs(ts[i] - tt))]
    return {"peak": pk, "peak_ms": tp * 1e3, "lost": 1.0 - at(0.0) / pk, "p15": at(0.015), "p35": at(0.035)}


def fit_shapes(cap_value, log=print):
    """every shape, every filter variant, the step free and capped at cap_value: list of result dicts."""
    ts, meas = _grid()
    out = []
    cache = {}
    for label, order, corr in FILTERS:
        for name, kind, par in SHAPES:
            for cap in (None, cap_value):
                if par is None:
                    best = min((_fit_shape(ts, meas, kind, k, order, corr, cap, cache=cache) for k in K_SCAN), key=lambda d: d["rms"])
                else:
                    best = _fit_shape(ts, meas, kind, par, order, corr, cap, cache=cache)
                best.update(filter=label, shape=name, cap=cap, shp=shape_of(ts, best["filtered"]))
                out.append(best)
    return ts, meas, out


def az_override(res, win=WIN, h=0.001):
    """[4]'s azimuth ANGLE curve with the rate correction of a fitted shape added: theta4(t) + the integral from -win
    of (the shape's rate - [4]'s rate curve), the shape continued past contact. Outside the window, theta4 (with the
    whole window's correction held after it)."""
    n = int(round(2 * win / h))
    acc, cum = 0.0, [0.0]
    for i in range(n):
        tm = -win + (i + 0.5) * h
        acc += (res["rate"](tm) - raw("bat_az_vel", tm)) * h
        cum.append(acc)

    def f(t):
        if t <= -win:
            return raw("bat_az", t)
        x = min(n * 1.0, (t + win) / h)
        i = min(n - 1, int(x))
        return raw("bat_az", t) + cum[i] + (x - i) * (cum[i + 1] - cum[i])
    return f


ELEV_AT_CONTACT = -29.7     # deg, [4]'s bat elevation at contact as digitised


def collision():
    """[4]'s mean exit velocity (its Eq. 14) and what a free bat (the oracle's rigid bat, physics-spec 8) must do to
    give it: per normal restitution, (e_n, the sweet spot's speed needed, the speed and turning rate the impact takes
    off it). Also the ball's impulse and the free bat's handle velocity change, for held_bat()."""
    bev = EQ14[0] + EQ14[1] * PEAK_MEAN_OF_SWINGS + EQ14[2] * PITCH_RELEASE
    rho_air = P.air_density()
    ball = P.integrate(P.State((0.0, PITCH_DIST, 1.5), (0.0, -PITCH_RELEASE, 1.0), (0.0, 0.0, 0.0)), rho_air, stop={"kind": "plane_y", "y": 0.0})
    v_plate = P.norm(ball["v"])
    bat = P.bat_mass_properties()
    rows, extra = [], []
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
            at = lambda V, W, u: P.add(V, P.cross(W, P.scale(a, u - bat["u_cm"])))
            dv = P.norm(P.sub(at(res["V"], res["W"], bat["L"] - 0.15), at(v_cm, om, bat["L"] - 0.15)))
            dgrip = P.norm(P.sub(at(res["V"], res["W"], GRIP_MID), at(v_cm, om, GRIP_MID)))
            return P.norm(res["v"]), dv, (res["W"][2] - w) / D2R, P.MASS * (res["v"][1] + v_plate), dgrip
        lo, hi = 10.0, 45.0
        for _ in range(50):
            mid = 0.5 * (lo + hi)
            lo, hi = (mid, hi) if exit_speed(mid)[0] < bev else (lo, mid)
        need = 0.5 * (lo + hi)
        r = exit_speed(need)
        rows.append((e_n, need, r[1], -r[2]))
        extra.append((r[3], r[4]))
    return {"bev": bev, "v_plate": v_plate, "rows": rows, "impulse": extra, "bat": bat}


GRIP_MID = 0.5 * (B.GRIP_L + B.GRIP_R)     # m from the knob: between the hands
CONTACT_TIME = (0.0006, 0.0010)     # s: the bat-ball contact's duration [E: about 0.7 ms in the literature physics-spec 6.3 rests on]
GRIP_K = (1.0e4, 1.0e6)     # N/m: the hands' grip on the handle as a spring in the impact's direction [E: a range spanning a soft
                             # palm to nearly rigid; no measurement was read for this job]


def held_bat(col):
    """does a held bat lose less at the impact than a free one? (body-motion 4.8; review P1-2.) Three readings:
    the physics-spec's own (the free-bat approximation, physics-spec 6.3 [S]); a grip spring [E] loaded by the
    handle's motion during the contact; and the body as it is built, where the hands are lumped rigidly into the
    bat (body.Model: the bat body carries both hands' mass), which is the case in which a held bat would lose less."""
    bat = col["bat"]
    J, dgrip = col["impulse"][0]
    print("     does a held bat lose less? The ball's impulse is %.1f N s; on a free bat the handle between the hands changes speed by %.1f m/s in the impact"
          % (J, dgrip))
    for tau in CONTACT_TIME:
        for k in GRIP_K:
            # the handle's displacement grows about as dv t^2 / (2 tau) over the contact; a spring's impulse over it is k dv tau^2 / 6
            extra = k * dgrip * tau * tau / 6.0
            print("       contact %.1f ms, grip %.0e N/m: the grip's extra impulse in the contact %.3f N s, %.1f %% of the ball's" % (tau * 1e3, k, extra, 100 * extra / J))
    # the body as built: the hands' mass rigidly on the bat, the impact's effective mass at the sweet spot with and without it
    m = B.Model(H, M, contacts=False)
    mb, ib = m.sys.m[m.b["bat"]], m.sys.I[m.b["bat"]][0]
    if True:
        d0, d1 = C.BAT_SS - C.BAT_UCM, C.BAT_SS - m.bat_ucm
        me_free = 1.0 / (1.0 / C.BAT_M + d0 * d0 / C.BAT_ICM)
        me_body = 1.0 / (1.0 / mb + d1 * d1 / ib)
        print("       the body as built lumps both hands (%.2f kg) rigidly into the bat: the impact's effective mass at the sweet spot is %.3f kg against the bare bat's %.3f,"
              " so a rigid grip would take %.0f %% less speed off the bat; the arms behind rigid wrists would add more"
              % (mb - C.BAT_M, me_body, me_free, 100 * (1 - me_free / me_body)))
        ip = C.BAT_ICM + C.BAT_M * (C.BAT_UCM - GRIP_MID) ** 2
        me_pin = ip / (C.BAT_SS - GRIP_MID) ** 2
        # the same impulse J (the ball's change of momentum hardly depends on this) on a bat turning about the held handle
        w_free = J * (C.BAT_SS - C.BAT_UCM) / C.BAT_ICM / D2R
        w_pin = J * (C.BAT_SS - GRIP_MID) / ip / D2R
        print("       the stiffest grip there could be, the handle held still between the hands: effective mass %.3f kg, %.0f %% less speed off the sweet spot than free;"
              " the turning rate's step %.0f deg/s against the free bat's %.0f (%.0f %%): a free bat's sweet-spot loss is part turning, part the whole bat recoiling"
              % (me_pin, 100 * (1 - me_free / me_pin), w_pin, w_free, 100 * w_pin / w_free))
        print("       so in the contact itself a held bat's step is %.0f-100 %% of a free bat's over the grip's range [E]; a rigidly held handle would give %.0f %%, and the body as built"
              " (rigid wrists) is nearer that case than a real grip is" % (100 - 100 * GRIP_K[1] * dgrip * CONTACT_TIME[1] ** 2 / 6.0 / J, 100 * w_pin / w_free))


def hands_through_filter(col):
    """how much the 12 Hz filter could change the hands' speed near contact (owner's question, session 7 fix): the
    impact kicks a free bat's handle back by dgrip; held, that loss reaches the hands and is shared with the forearms
    over the next milliseconds. A loss of the hands' speed at contact shows through the zero-lag filter as a fraction
    of it before contact; the fractions are the filter's step response, run here."""
    J, dgrip = col["impulse"][0]
    m = B.Model(H, M, contacts=False)
    share = m.sys.m[m.b["bat"]] + m.sys.m[m.b["farm_L"]] + m.sys.m[m.b["farm_R"]]
    print("     the hands' speed through the filter: a step loss of the hands' speed at contact, between nothing and the free handle's %.1f m/s kick [E];"
          " the ball's %.1f N s shared by the bat, both hands and both forearms (%.1f kg) is %.1f m/s" % (dgrip, J, share, J / share))
    ts = [-0.4 + i / FS for i in range(800)]
    step = [0.0 if t < 0 else -1.0 for t in ts]
    for label, order, corr in FILTERS:
        y = lowpass(step, order, FC, corr)
        at = lambda tt: -y[min(range(len(ts)), key=lambda i: abs(ts[i] - tt))]
        print("       %-40s shows %.2f of the step at contact, %.2f at -10 ms, %.2f at -20, %.2f at -50: so %.1f-%.1f m/s at contact, %.1f-%.1f at -10 ms"
              % (label, at(0.0), at(-0.010), at(-0.020), max(0.0, at(-0.050)), at(0.0) * J / share, at(0.0) * dgrip, at(-0.010) * J / share, at(-0.010) * dgrip))


LAG_RATE_3 = (1588.0, 162.0)        # deg/s, [3] Table 2: the bat lag's largest closing rate, at -20 ms
ATTACK_OBM = (4.1, 5.9)            # deg, note 6: OpenBiomechanics' attack angle at contact, mean and SD over 677 swings


def show_attack(title, res):
    print("   " + title)
    for t in (-20, -10, 0):
        c = res[t][1]
        print("     %4d ms: sweet spot %4.1f m/s = hands %4.1f + azimuth %4.1f + elevation %4.1f; its velocity %+5.1f m/s vertical, attack angle %+5.1f deg; bat elevation %5.1f deg"
              % (t, c["speed"], c["hands"], c["az"], c["el"], c["vz"], c["attack"], c["elev"]))


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
    return {"speed": sp, "attack": math.asin(max(-1.0, min(1.0, uh[2]))) / D2R, "vz": vs[2], "hands": E.dot(vh, uh), "az": rho * az_rate * ce * E.dot(e_az, uh), "el": rho * el_rate * E.dot(e_el, uh),
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

    col = collision()
    cap_turn = max(r[3] for r in col["rows"])
    cap_az = cap_turn / math.cos(ELEV_AT_CONTACT * D2R)
    print("\n== 3. the 12 Hz filter run through the impact ([4] 3.4): the true curve rises before contact, steps at contact, recovers after; filtered; fitted to [4]'s curve over +-%.0f ms" % ((WIN + 0.03) * 1e3))
    print("   3a. session 7's reading: polynomial pieces free inside +-%.0f ms, [4]'s curve outside (so the true curve may JUMP at the window's edges: review P1-2), Winter's correction fixed (P3-1)" % (WIN * 1e3))
    print("     %-40s | misfit RMS deg/s: no break, a step | true rate just before contact | the step | jump at -70 ms, +70 ms" % "the filter [E]")
    for label, order, corr in FILTERS:
        for deg in (1, 2):
            rms0 = unsmooth("bat_az_vel", order, corr, "none", deg)[2]
            true, before, rms, v0, _, jump = unsmooth("bat_az_vel", order, corr, "rate", deg)
            print("     %-40s | %5.0f %5.0f | %5.0f | %6.0f | %+6.0f %+6.0f   (pieces of degree %d)" % (label, rms0, rms, v0, true(1e-6) - v0, jump[0], jump[1], deg))
    print("   3b. shapes joined to [4]'s curve at -70 and +70 ms in value (and in slope at -70 where said). [4]'s curve: peak %.0f deg/s at %.0f ms, %.1f %% lost by contact, %.0f at +15 ms, %.0f at +35"
          % (lambda d: (d["peak"], d["peak_ms"], 100 * d["lost"], d["p15"], d["p35"]))(shape_of(*_grid())))
    print("     the step's ceiling, a free bat's loss (6 below): %.0f deg/s of turning rate, %.0f deg/s of [4]'s azimuth rate with the bat %.0f deg below level; a held bat loses about the same (6)"
          % (cap_turn, cap_az, -ELEV_AT_CONTACT))
    print("     a shape FITS if its misfit is under %.0f deg/s [E] and its step is at most a free bat's (%.0f): larger steps are ruled out by the collision" % (FIT_RMS_OK, cap_az))
    ts, meas, res = fit_shapes(cap_az)
    print("     %-38s %-48s | step   | just before contact | step (of a free bat's) | misfit | slope jump at -70 | filtered: peak, when, lost by contact, +15, +35 | fits"
          % ("the filter [E]", "the shape [E]"))
    survivors = []
    for r in res:
        if r["cap"] is not None and not r["capped"]:
            continue                                  # the cap did not bind: the same as the free row
        sh = r["shp"]
        ok = r["rms"] <= FIT_RMS_OK and r["step"] <= cap_az + 1e-6
        r["fits"] = ok
        par = "" if r["par"] in (None, 0.0) or r["kind"] == "plateau" else " (k %.1f)" % r["par"]
        print("     %-38s %-48s | %-6s | %5.0f deg/s | %5.0f (%.2f) | %4.0f | %+6.0f deg/s^2 | %5.0f %4.0f ms %4.1f %% %5.0f %5.0f | %s"
              % (r["filter"], r["shape"] + par, "capped" if r["capped"] else "free", r["r0"], r["step"], r["step"] / cap_az, r["rms"], r["slope_jump"],
                 sh["peak"], sh["peak_ms"], 100 * sh["lost"], sh["p15"], sh["p35"], "FITS" if ok else "no" + (" (step over a free bat's)" if r["step"] > cap_az else "")))
        if ok:
            survivors.append(r)
    r0s = [r["r0"] for r in survivors]
    print("     the shapes that fit: the bat's true azimuth rate just before contact %.0f-%.0f deg/s, where the filtered curve reads %.0f (session 7's straight pieces: 2,649-2,841)"
          % (min(r0s), max(r0s), B.curve("bat_az_vel", 0.0)))
    print("   3c. the elevation and the lag undone as session 7 did (pieces of degree 2, angle continuous at contact, slope free to break), with the jump at the window's edges")
    print("     %-40s | elevation: slope before contact, angle at contact, jump at -70/+70 | lag: slope, angle, jump | misfit RMS deg" % "the filter [E]")
    for label, order, corr in FILTERS:
        e = unsmooth("bat_elev", order, corr, "angle", 2)
        g = unsmooth("bat_lag", order, corr, "angle", 2)
        print("     %-40s | %5.0f deg/s %6.1f deg %+5.1f/%+5.1f | %6.0f deg/s %5.1f deg %+5.1f/%+5.1f | %.2f %.2f"
              % (label, e[4], e[3], e[5][0], e[5][1], g[4], g[3], g[5][0], g[5][1], e[2], g[2]))
    sl = lambda nm: (B.curve(nm, 0.005) - B.curve(nm, -0.005)) / 0.010
    print("     [4]'s curves as filtered: elevation %.0f deg/s, %.1f deg at contact; lag %.0f deg/s, %.1f deg. [3]'s lag closes at most at %.0f +- %.0f deg/s, at -20 ms (its arm line from mid-shoulders"
          " to mid-wrists, not the lead arm; also filtered through the impact)" % (sl("bat_elev"), B.curve("bat_elev", 0.0), sl("bat_lag"), B.curve("bat_lag", 0.0), LAG_RATE_3[0], LAG_RATE_3[1]))

    print("\n== 4. the placed motion with the smoothing undone: the sweet spot's speed and the attack angle at contact (its velocity above level; OpenBiomechanics %+.1f +- %.1f deg, mostly college hitters)" % ATTACK_OBM)
    show_attack("as placed", base)
    session7 = {}
    for label, order, corr in FILTERS:
        session7[label] = {}
        for nm in ("bat_elev", "bat_lag"):
            true, before, *_ = unsmooth(nm, order, corr, "angle", 2)
            session7[label][nm] = (lambda t, true=true, before=before: before(t) if 0.0 <= t <= WIN else true(t))
    undone = {}
    for r in survivors + [x for x in res if x["kind"] == "line" and x["cap"] is None]:
        tag = "%s; %s%s%s" % (r["filter"][:24], r["shape"][:30], " k %.1f" % r["par"] if r["kind"] in ("pow", "powc1") else "", ", step capped" if r["capped"] else "")
        if not r.get("fits"):
            tag += " (does NOT fit; session 7's family)"
        azo = az_override(r)
        off = azo(0.0) - raw("bat_az", 0.0)
        print("   " + tag + ": the bat %.1f deg further round at contact than [4]'s angle curve (%.1f), so at [4]'s -8 deg %.1f ms earlier"
              % (off, raw("bat_az", 0.0), off / r["r0"] * 1e3))
        for what, extra in (("azimuth only", ()), ("azimuth and lag", ("bat_lag",)), ("azimuth, elevation, lag (session 7's)", ("bat_elev", "bat_lag"))):
            B.CURVE_OVERRIDE.clear()
            B.CURVE_OVERRIDE["bat_az"] = az_override(r)
            for nm in extra:
                B.CURVE_OVERRIDE[nm] = session7[r["filter"]][nm]
            try:
                c = placed(0.010)[0][1]
                print("     %-38s sweet spot %4.1f m/s (%2.0f mph) = hands %4.1f + azimuth %4.1f + elevation %4.1f; attack angle %+5.1f deg; azimuth rate %5.0f deg/s"
                      % (what, c["speed"], c["speed"] / 0.44704, c["hands"], c["az"], c["el"], c["attack"], c["az_rate"]))
                undone.setdefault(what, []).append((c["speed"], c["attack"], r.get("fits", False)))
            except E.SolveError as ex:
                print("     %-38s the posture did not assemble (%s)" % (what, ex))
        B.CURVE_OVERRIDE.clear()
    for what, v in undone.items():
        f = [x for x in v if x[2]]
        if f:
            print("   over the shapes that fit, %s: the sweet spot %.1f-%.1f m/s (%.0f-%.0f mph), the attack angle %+.1f to %+.1f deg"
                  % (what, min(x[0] for x in f), max(x[0] for x in f), min(x[0] for x in f) / 0.44704, max(x[0] for x in f) / 0.44704, min(x[1] for x in f), max(x[1] for x in f)))

    print("\n== 5. the mean curve against single swings ([4] Table 3, MIDDLE): the mean of the swings' own peaks %.0f deg/s, the mean curve's peak %.0f: %.1f %% blunter"
          % (PEAK_MEAN_OF_SWINGS, PEAK_OF_MEAN_CURVE, 100 * (1 - PEAK_OF_MEAN_CURVE / PEAK_MEAN_OF_SWINGS)))
    print("     as bat speed at the sweet spot: %.1f m/s of the turning part" % ((PEAK_MEAN_OF_SWINGS - PEAK_OF_MEAN_CURVE) * D2R * rho_ss * math.cos(30 * D2R)))

    print("\n== 6. what [4]'s own batted balls need of the bat")
    col = collision()
    print("     [4] Eq. 14 at the MIDDLE location, the mean pitch (%.1f m/s) and the mean peak bat rate: mean ball exit velocity %.1f m/s (%.0f mph), over every swing with a measurable batted ball"
          % (PITCH_RELEASE, col["bev"], col["bev"] / 0.44704))
    print("     the pitch at the plate, by the oracle's flight from %.0f m: %.1f m/s" % (PITCH_DIST, col["v_plate"]))
    for e_n, need, dv, dw in col["rows"]:
        print("     normal restitution %.2f: a ball met squarely at the sweet spot leaves at %.1f m/s only if the sweet spot moves at %.1f m/s; that impact takes %.1f m/s off the sweet spot and %.0f deg/s off the turning rate of a bat nothing holds"
              % (e_n, col["bev"], need, dv, dw))
    print("     as the azimuth rate [4] reports, with the bat %.1f deg below level at contact ([4]), a turning-rate loss L is L / cos(elevation): %s deg/s"
          % (-ELEV_AT_CONTACT, ", ".join("%.0f" % (r[3] / math.cos(ELEV_AT_CONTACT * D2R)) for r in col["rows"])))
    held_bat(col)
    hands_through_filter(col)
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
