#!/usr/bin/env python3
"""Spec swing-model-v3 step 0: the v3 path against [4]'s Appendix E curves (digitised by
fortenbaugh_appendix_e.py). Needs the spec probe (sim/target/codex-probes/v3_review_probe.py, not in git)
and the CSVs it wrote under sim/target/research-dl/4/csv/. Run from the repo root.

Prints, for the three golden aims, [4]'s azimuth (from contact), elevation, azimuth rate (against its
contact value) and lag at [4]'s instants beside v3's at the scaled instants (tau = s_T * t4), and then
the same with two extra elevation conditions read from Appendix E (variant 'E2')."""
import sys, csv, math
sys.path.insert(0, 'sim/target/codex-probes'); sys.path.insert(0, 'analysis/oracle')
import v3_review_probe as v
D = v.D
d = 'sim/target/research-dl/4/csv/'
def load(n): return [(float(a), float(b)) for a, b in list(csv.reader(open(d + n + '.csv')))[1:]]
def at(c, t):
    near = [p[1] for p in c if abs(p[0] - t) <= 1.5]; return sum(near) / len(near)
az, el, lag, vel = load('bat_az'), load('bat_elev'), load('bat_lag'), load('bat_az_vel')
W93 = v.FH('93')[0]
sT = v.T_SW / 0.170
T4 = (-170, -130, -110, -93, -80, -70, -60, -50, -40, -30, -25, -20, -15, -10, -5, 0)

def build(aim, extraE):
    path = v.build_path(v.AIMS[aim], 'M', W93)
    if not extraE: return path
    p, cA, cE, cW, jt, A0 = path
    (A0_, A1, A2), (E0, E1, E2), _ = v.v2_derivs(p, 0.0)
    ts = -v.T_SW
    conds = [(ts, 0, 41 * D), (ts, 1, 0), (ts, 2, 0), (v.T93, 0, 39 * D), (0, 0, E0), (0, 1, E1), (0, 2, E2)]
    for t4, val in extraE: conds.append((sT * t4 / 1000, 0, val * D))
    return p, cA, v.poly(conds), cW, jt, A0

def report(aim, extraE=None):
    path = build(aim, extraE); Ac = path[5]
    rc = (v.pose(path, 1e-5)[0] - v.pose(path, -1e-5)[0]) / 2e-5 / D
    print(f'-- aim {aim}, elevation {"with E conditions " + str(extraE) if extraE else "as the spec"}; azimuth rate at contact {rc:.0f} deg/s')
    print(' t4  | [4]: A-Ac    E  rate/rc   lag | v3: A-Ac     E  rate/rc   lag  ss mph')
    for t4 in T4:
        tau = sT * t4 / 1000
        A, E, W = v.pose(path, tau)
        r = (v.pose(path, tau + 1e-5)[0] - v.pose(path, tau - 1e-5)[0]) / 2e-5 / D
        lg = v.ang(v.sub(W, v.S_curve(tau)), v.bat_dir(A, E))
        print(' %4d | %7.1f %6.1f %6.2f %6.1f | %7.1f %6.1f %6.2f %6.1f %6.1f' % (
            t4, at(az, t4) - at(az, 0), at(el, t4), at(vel, t4) / at(vel, 0), at(lag, t4),
            (A - Ac) / D, E / D, r / rc, lg, v.ss_speed(path, tau) / 0.44704))

for aim in v.AIMS:
    report(aim)
for aim in v.AIMS:
    report(aim, [(-60, 20.4), (-30, -11.1)])
