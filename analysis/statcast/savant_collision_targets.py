#!/usr/bin/env python3
"""Spray of balls in play by launch angle, location and height, from Savant pitch CSVs.

Usage: python3 -I analysis/statcast/savant_collision_targets.py <dir with sc_<date>.csv> [--markdown]

Standard library only. Targets for the collision work (docs/calibration/2026-11-14-collision-targets.md).
Conventions (swing note §8): spray = atan2(hc_x - 125.42, 198.27 - hc_y) in degrees, + toward right field;
pull-positive spray = -mirror * spray with mirror = +1 for right-handed batters, -1 for left-handed.
Attack direction's CSV sign is negative = pulled, so pull-positive AD = -attack_direction.
Horizontal location is inside-positive: inside = -mirror * plate_x (catcher's view, + toward first base).
Height third: (plate_z - sz_bot) / (sz_top - sz_bot) in thirds of the batter's zone (below 0 / above 1 are separate).
Balls in play = description starting 'hit_into_play' with hit coordinates and launch angle.
"""
import csv, glob, math, statistics as st, sys
from collections import defaultdict

BANDS = [(-90, 0, '< 0'), (0, 10, '0-10'), (10, 25, '10-25'), (25, 35, '25-35'), (35, 50, '35-50'), (50, 91, '50+')]
HEIGHT = [(-9, 0.0, 'below zone'), (0.0, 1 / 3, 'bottom third'), (1 / 3, 2 / 3, 'middle third'), (2 / 3, 1.0, 'top third'), (1.0, 9, 'above zone')]
LOC_FT = [(-9, -0.55, 'outside'), (-0.55, -0.18, 'mid-away'), (-0.18, 0.18, 'middle'), (0.18, 0.55, 'mid-in'), (0.55, 9, 'inside')]


def num(r, k):
    v = r.get(k, '')
    return None if v in ('', 'NA', 'null') else float(v)


def load(d):
    rows = []
    files = sorted(glob.glob(d + '/sc_*.csv'))
    for f in files:
        with open(f, newline='', encoding='utf-8-sig') as fh:
            for r in csv.DictReader(fh):
                if not r['description'].startswith('hit_into_play'):
                    continue
                hx, hy, la = num(r, 'hc_x'), num(r, 'hc_y'), num(r, 'launch_angle')
                if hx is None or hy is None or la is None:
                    continue
                mir = 1 if r['stand'] == 'R' else -1
                spray = -mir * math.degrees(math.atan2(hx - 125.42, 198.27 - hy))  # pull-positive
                ad = num(r, 'attack_direction')
                px, pz, bot, top = num(r, 'plate_x'), num(r, 'plate_z'), num(r, 'sz_bot'), num(r, 'sz_top')
                rows.append(dict(
                    stand=r['stand'], la=la, spray=spray, bb=r['bb_type'],
                    ad=None if ad is None else -ad,
                    inside=None if px is None else -mir * px,
                    h=None if None in (pz, bot, top) else (pz - bot) / (top - bot),
                    ev=num(r, 'launch_speed'),
                ))
    return rows, files


def band_of(la):
    for lo, hi, n in BANDS:
        if lo <= la < hi:
            return n


def ols(xs, ys):
    n = len(xs)
    mx, my = sum(xs) / n, sum(ys) / n
    sxx = sum((x - mx) ** 2 for x in xs)
    sxy = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    syy = sum((y - my) ** 2 for y in ys)
    return sxy / sxx, sxy / math.sqrt(sxx * syy), n


def cell(v):
    if len(v) < 2:
        return '%d | | ' % len(v)
    return '%d | %+.1f | %.1f' % (len(v), st.mean(v), st.pstdev(v))


def table(title, rows, key, groups, val='spray'):
    print('\n### ' + title + '\n')
    print('| ' + ' | '.join(['group'] + ['n', 'mean', 'SD']) + ' |')
    print('|---|---|---|---|')
    for g in groups:
        v = [r[val] for r in rows if key(r) == g and r[val] is not None]
        print('| %s | %s |' % (g, cell(v)))


def main():
    rows, files = load(sys.argv[1])
    print('Files: %d dates; balls in play with coordinates and launch angle: %d (R %d, L %d)' % (
        len(files), len(rows), sum(r['stand'] == 'R' for r in rows), sum(r['stand'] == 'L' for r in rows)))
    names = [b[2] for b in BANDS]
    # 1. By launch-angle band and batter side.
    for side, sel in (('all (pull-positive, sides pooled)', lambda r: True), ('right-handed', lambda r: r['stand'] == 'R'), ('left-handed', lambda r: r['stand'] == 'L')):
        sub = [r for r in rows if sel(r)]
        table('Pull-positive spray by launch-angle band, ' + side, sub, lambda r: band_of(r['la']), names)
        sl, rr, n = ols([r['la'] for r in sub], [r['spray'] for r in sub])
        print('\nSlope of spray on launch angle: %+.3f deg per deg (r %.2f, n %d)' % (sl, rr, n))
        sub2 = [r for r in sub if 0 <= r['la'] < 50]
        sl, rr, n = ols([r['la'] for r in sub2], [r['spray'] for r in sub2])
        print('Same, launch angle 0-50 only: %+.3f deg per deg (r %.2f, n %d)' % (sl, rr, n))
    # Savant batted-ball types (the sim's GB/LD/FB/PU).
    table('Pull-positive spray by Savant batted-ball type, sides pooled', rows, lambda r: r['bb'], ['ground_ball', 'line_drive', 'fly_ball', 'popup'])
    def mean_of(bt):
        return st.mean([r['spray'] for r in rows if r['bb'] == bt])
    print('\nFall from ground balls to fly balls: %.1f deg (ground_ball %+.1f, fly_ball %+.1f); to popups %.1f' % (
        mean_of('ground_ball') - mean_of('fly_ball'), mean_of('ground_ball'), mean_of('fly_ball'), mean_of('ground_ball') - mean_of('popup')))
    # 2. By horizontal location and height third: marginal, then cross with launch-angle band.
    loc = lambda r: None if r['inside'] is None else next((n for lo, hi, n in LOC_FT if lo <= r['inside'] / 0.3048 < hi), None)
    hgt = lambda r: None if r['h'] is None else next((n for lo, hi, n in HEIGHT if lo <= r['h'] < hi), None)
    lnames = [x[2] for x in LOC_FT]
    hnames = [x[2] for x in HEIGHT]
    table('Pull-positive spray by horizontal location (ft from the plate center line, + inside), sides pooled', rows, loc, lnames)
    table('Pull-positive spray by pitch-height third of the batter\'s zone, sides pooled', rows, hgt, hnames)
    for title, f, gn in (('horizontal location', loc, lnames), ('height third', hgt, hnames)):
        print('\n### Spray against launch angle by %s: slope (deg per deg, r, n), sides pooled\n' % title)
        print('| group | slope | r | n | mean spray |')
        print('|---|---|---|---|---|')
        for g in gn:
            sub = [r for r in rows if f(r) == g]
            if len(sub) > 30:
                sl, rr, n = ols([r['la'] for r in sub], [r['spray'] for r in sub])
                print('| %s | %+.3f | %.2f | %d | %+.1f |' % (g, sl, rr, n, st.mean([r['spray'] for r in sub])))
        print('\n### Mean pull-positive spray (n; SD) by %s x launch-angle band, sides pooled\n' % title)
        print('| | ' + ' | '.join(names) + ' |')
        print('|---|' + '---|' * len(names))
        for g in gn:
            cells = []
            for b in names:
                v = [r['spray'] for r in rows if f(r) == g and band_of(r['la']) == b]
                cells.append('%+.1f (%d; %.1f)' % (st.mean(v), len(v), st.pstdev(v)) if len(v) > 1 else '(%d)' % len(v))
            print('| %s | %s |' % (g, ' | '.join(cells)))
    # 3. Spray minus attack direction by launch-angle band.
    ad_rows = [r for r in rows if r['ad'] is not None and r['bb'] != 'popup']
    for r in rows:
        r['sad'] = None if r['ad'] is None else r['spray'] - r['ad']
    print('\n### Spray minus attack direction (pull-positive, deg) by launch-angle band; popups left out as in savant_sprayad.py\n')
    print('| band | all: n | mean | SD | right-handed batters: n | mean | SD |')
    print('|---|---|---|---|---|---|---|')
    for b in names:
        a = [r['sad'] for r in ad_rows if band_of(r['la']) == b]
        rr = [r['sad'] for r in ad_rows if band_of(r['la']) == b and r['stand'] == 'R']
        if len(a) > 1:
            print('| %s | %s | %s |' % (b, cell(a), cell(rr)))
    a = [r['sad'] for r in ad_rows]
    print('\nAll non-popup balls in play with attack direction: n %d, spray - AD mean %+.1f SD %.1f' % (len(a), st.mean(a), st.pstdev(a)))
    print('\n### Attack direction (pull-positive) by launch-angle band, same rows\n')
    print('| band | n | mean AD | SD |')
    print('|---|---|---|---|')
    for b in names:
        v = [r['ad'] for r in ad_rows if band_of(r['la']) == b]
        if len(v) > 1:
            print('| %s | %s |' % (b, cell(v)))
    sl, rr, n = ols([r['la'] for r in ad_rows], [r['sad'] for r in ad_rows])
    print('\nSlope of (spray - AD) on launch angle: %+.3f deg per deg (r %.2f, n %d)' % (sl, rr, n))


main()
