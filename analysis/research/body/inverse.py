#!/usr/bin/env python3
"""Inverse dynamics of the average swing of Fortenbaugh 2011 [4], about the vertical axis.

What it does (docs/body-motion.md 3.2): takes [4]'s Appendix E mean curves (pelvis and upper-trunk
rotation, bat azimuth, elevation and lag, pelvis forward position) and works back the yaw torques that
a pelvis -> trunk -> arms -> bat chain needs to move that way:
  tau_legs  ground -> pelvis (the legs: hip muscles reacting on the planted feet)
  tau_core  pelvis -> trunk
  tau_sh    trunk -> arms   (both arms as one link from the trunk's axis to the hands)
  tau_wr    arms -> bat     (about the vertical through the hands)
Then it compares tau_legs (= the whole chain's rate of vertical angular momentum about the pelvis) with
the yaw moment of [4]'s measured ground forces, under both possible signs of [4]'s GRFy, as an
independent check: the free moment of the feet is not in [4], so agreement can only be partial.

Segment masses and inertias: de Leva 1996 at [4]'s mean batter (1.84 m, 92 kg). Geometry [E]:
the arm link's horizontal reach from the axis to the hands L_A = 0.30 H; the arms' masses lumped at
0.22, 0.42 and 0.55 m (upper arms, forearms, hands); the pelvis yaw inertia counts half the thighs.
The arm direction is the bat's azimuth plus [4]'s lag (planar reading of a 3D angle; docs 3.2 says
what that costs). The bat is the game's default bat, projected by its elevation.

Usage: python3 analysis/research/body/inverse.py   (needs the CSVs of fortenbaugh_appendix_e.py)
"""
import math, sys, os
sys.path.insert(0, os.path.dirname(__file__))
from common import *

H, M = 1.84, 92.0
BW = M * G
DT = 0.001
T0, T1 = -0.30, 0.02
SM = int(sys.argv[1]) if len(sys.argv) > 1 else 8      # smoothing half-width (ms)


def series(name, scale=1.0):
    return [x * scale for x in smooth(resample(load(name), T0, T1, DT), SM)]


def main():
    d2r = math.pi / 180
    n = int(round((T1 - T0) / DT)) + 1
    ts = [T0 + i * DT for i in range(n)]
    p1 = series("pelvis_rot", d2r); p2 = series("trunk_rot", d2r)
    w1 = series("pelvis_rot_vel", d2r); w2 = series("trunk_rot_vel", d2r)   # [4]'s own rate curves
    A = series("bat_az", d2r); E = series("bat_elev", d2r); lag = series("bat_lag", d2r)
    w4 = series("bat_az_vel", d2r)
    yp = series("pelvis_x_cm", 0.01)
    p3 = [a + l for a, l in zip(A, lag)]
    w3 = [a + b for a, b in zip(w4, deriv(lag, DT))]
    a1 = deriv(w1, DT); a2 = deriv(w2, DT); a3 = deriv(w3, DT); a4 = deriv(w4, DT)
    Ed = deriv(E, DT); Edd = deriv(Ed, DT)
    ypa = deriv(deriv(yp, DT), DT)

    # inertias
    _, m_lpt, _, _, _, I_lpt = segment("lpt", H, M)
    _, m_th, _, _, _, I_th = segment("thigh", H, M)
    I1 = I_lpt + 0.5 * 2 * (I_th + m_th * (0.1 * H / 1.84) ** 2)          # [E] half the thighs turn with it
    I2 = sum(segment(s, H, M)[5] for s in ("upt", "mpt", "head"))
    m_ua = 2 * segment("upperarm", H, M)[1]; m_fa = 2 * segment("forearm", H, M)[1]; m_ha = 2 * segment("hand", H, M)[1]
    LA = 0.30 * H
    arm_pts = [(m_ua, 0.22), (m_fa, 0.42), (m_ha, LA)]
    cb = BAT_UCM - GRIP_FROM_KNOB
    ss = BAT_SS - GRIP_FROM_KNOB

    def rot_pt(r, ph, w, al):
        """position (relative to B), acceleration of a point at radius r on a link at angle ph"""
        c, s_ = math.cos(ph), math.sin(ph)
        return (r * c, r * s_), (r * (-al * s_ - w * w * c), r * (al * c - w * w * s_))
    hands = []; ha = []
    for i in range(n):
        p, a = rot_pt(LA, p3[i], w3[i], a3[i]); hands.append(p); ha.append((a[0], a[1] + ypa[i]))
    bcm, ab, ssv = [], [], []
    for i in range(n):
        # bat point at distance u along the bat from the hands: horizontal radius u cos E on azimuth A
        def bat_pt(u):
            rho = u * math.cos(E[i]); rhod = -u * math.sin(E[i]) * Ed[i]
            rhodd = -u * (math.cos(E[i]) * Ed[i] ** 2 + math.sin(E[i]) * Edd[i])
            c, s_ = math.cos(A[i]), math.sin(A[i])
            pos = (hands[i][0] + rho * c, hands[i][1] + rho * s_)
            acc = (ha[i][0] + rhodd * c - 2 * rhod * w4[i] * s_ - rho * (a4[i] * s_ + w4[i] ** 2 * c),
                   ha[i][1] + rhodd * s_ + 2 * rhod * w4[i] * c + rho * (a4[i] * c - w4[i] ** 2 * s_))
            vh = (-LA * w3[i] * math.sin(p3[i]) + rhod * c - rho * w4[i] * s_,
                  LA * w3[i] * math.cos(p3[i]) + rhod * s_ + rho * w4[i] * c,
                  u * math.cos(E[i]) * Ed[i])
            return pos, acc, vh
        p, a, _ = bat_pt(cb); bcm.append(p); ab.append(a)
        _, _, v = bat_pt(ss); ssv.append(math.sqrt(v[0] ** 2 + v[1] ** 2 + v[2] ** 2))
    ab = ([a[0] for a in ab], [a[1] for a in ab])
    arm_acc = []
    for m, r in arm_pts:
        pos, acc_ = [], []
        for i in range(n):
            p, a = rot_pt(r, p3[i], w3[i], a3[i]); pos.append(p); acc_.append(a)
        arm_acc.append((m, pos, ([a[0] for a in acc_], [a[1] + ypa[i] for i, a in enumerate(acc_)])))
    Hb = [(BAT_ICM * math.cos(E[i]) ** 2 + BAT_IAX * math.sin(E[i]) ** 2) * w4[i] for i in range(n)]
    dHb = deriv(Hb, DT)
    ab = (ab[0], [ab[1][i] + ypa[i] for i in range(n)])
    yp0 = yp
    yp = [0.0] * n            # positions above are relative to B

    def cross(r, f):
        return r[0] * f[1] - r[1] * f[0]
    rows = []
    for i in range(n):
        h = hands[i]; B = (0.0, 0.0)
        fb = (BAT_M * ab[0][i], BAT_M * ab[1][i])
        tw = cross((bcm[i][0] - h[0], bcm[i][1] - h[1]), fb) + dHb[i]
        ts_ = cross((bcm[i][0] - B[0], bcm[i][1] - B[1]), fb) + dHb[i]
        for m, pos, a in arm_acc:
            ts_ += cross((pos[i][0] - B[0], pos[i][1] - B[1]), (m * a[0][i], m * a[1][i]))
        tc = ts_ + I2 * a2[i]
        tl = tc + I1 * a1[i]
        rows.append((ts[i], tw, ts_, tc, tl))

    # ground yaw moment from [4]'s forces about B, feet [E]: rear (0, -0.80), lead (+0.11, -0.80 + 0.42 H)
    gx_t = resample(load("trail_grf_x"), T0, T1, DT); gx_l = resample(load("lead_grf_x"), T0, T1, DT)
    gy_t = resample(load("trail_grf_y"), T0, T1, DT); gy_l = resample(load("lead_grf_y"), T0, T1, DT)
    fr, fl = (0.0, -0.80), (0.11, -0.80 + 0.42 * H)

    def ground(i, sgn):
        # [4] GRFx is along +y here (toward the pitcher); GRFy is along x with sign sgn (unknown: +1 = toward the plate)
        B = (0.0, yp0[i]); tot = 0.0
        for f, gx, gy in ((fr, gx_t, gy_t), (fl, gx_l, gy_l)):
            F = (sgn * gy[i] / 100 * BW, gx[i] / 100 * BW)
            tot += cross((f[0] - B[0], f[1] - B[1]), F)
        return tot

    print("inverse dynamics of [4]'s mean swing, H %.2f m, M %.0f kg; smoothing %d ms half-width (x2)" % (H, M, SM))
    print("inertias about the vertical: pelvis(+half thighs) %.3f, trunk+head %.3f kg m^2; arms %.1f kg, L_A %.3f m; bat %.3f kg" % (I1, I2, m_ua + m_fa + m_ha, LA, BAT_M))
    print("%7s %8s %8s %8s %8s %8s | %9s %9s | %7s %7s %7s %7s" % ("t ms", "wrist", "shoulder", "core", "legs", "", "gnd(+y)", "gnd(-y)", "pel/s", "trk/s", "arm/s", "ss m/s"))
    for k in range(0, n, 10):
        t, tw, ts_, tc, tl = rows[k]
        print("%7.0f %8.0f %8.0f %8.0f %8.0f %8s | %9.0f %9.0f | %7.0f %7.0f %7.0f %7.1f" % (t * 1000, tw, ts_, tc, tl, "", ground(k, +1), ground(k, -1),
              w1[k] / d2r, w2[k] / d2r, w3[k] / d2r, ssv[k]))
    # peaks and work
    for j, nm in ((1, "wrist"), (2, "shoulder"), (3, "core"), (4, "legs")):
        lo = [r for r in rows if -0.25 <= r[0] <= 0.0]
        mx = max(lo, key=lambda r: r[j]); mn = min(lo, key=lambda r: r[j])
        print("%-8s peak %+6.0f N m at %4.0f ms, min %+6.0f at %4.0f ms" % (nm, mx[j], mx[0] * 1000, mn[j], mn[0] * 1000))
    # joint powers and work from -170 ms to contact
    rel = {"legs": (lambda i: w1[i]), "core": (lambda i: w2[i] - w1[i]), "shoulder": (lambda i: w3[i] - w2[i]), "wrist": (lambda i: w4[i] - w3[i])}
    i0 = int(round((-0.17 - T0) / DT)); ic = int(round((0.0 - T0) / DT))
    for j, nm in ((4, "legs"), (3, "core"), (2, "shoulder"), (1, "wrist")):
        Wk = sum(rows[i][j] * rel[nm](i) for i in range(i0, ic)) * DT
        Pk = max(rows[i][j] * rel[nm](i) for i in range(i0, ic))
        print("work %-8s %+7.1f J (-170 ms to contact), peak power %6.0f W" % (nm, Wk, Pk))
    # ground check: integral of moments -170..0
    for sgn in (+1, -1):
        Ig = sum(ground(i, sgn) for i in range(i0, ic)) * DT
        Il = sum(rows[i][4] for i in range(i0, ic)) * DT
        print("angular impulse -170..0: chain needs %.1f N m s; [4]'s forces give %.1f (GRFy sign %+d)" % (Il, Ig, sgn))
    ke_b = 0.5 * BAT_M * ssv[ic] ** 2
    print("sweet-spot speed at contact %.1f m/s (planar reading of [4]'s bat angles at L_A, [4] gives none; [3] 29 m/s)" % ssv[ic])


if __name__ == "__main__":
    main()
