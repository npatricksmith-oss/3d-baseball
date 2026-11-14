#!/usr/bin/env python3
"""Is the pull-side offset in (spray - attack direction) already there when the barrel travels straight up the middle?

Usage: python3 -I analysis/statcast/savant_square_contact_spray.py <dir with sc_<date>.csv> [window_deg=3]

Standard library only. Balls in play with hit coordinates, launch angle in [0, 50), attack direction. Conventions as in
savant_collision_targets.py: spray is pull-positive (mirrored for left-handers); the attack direction's CSV sign is
negative = pulled, so the pull-positive attack direction is p = -attack_direction. For swings with |p| <= window
it prints, by launch-angle band, n, mean spray, SD and SE; then the same table for a ladder of p bins, with the mean of
spray - p (the offset) and its SE, and the OLS slope of spray on p for balls in 0-25 deg launch angle.
"""
import csv, glob, math, statistics as st, sys

W = float(sys.argv[2]) if len(sys.argv) > 2 else 3.0
BANDS = [(0, 10), (10, 25), (25, 35), (35, 50)]
PBINS = [(-30, -8), (-8, -3), (-3, 3), (3, 8), (8, 13), (13, 20), (20, 60)]


def num(r, k):
    v = r.get(k, '')
    return None if v in ('', 'NA', 'null') else float(v)


def load(d):
    rows = []
    for f in sorted(glob.glob(d + '/sc_*.csv')):
        with open(f, newline='', encoding='utf-8-sig') as fh:
            for r in csv.DictReader(fh):
                if not r['description'].startswith('hit_into_play'):
                    continue
                hx, hy, la, ad = num(r, 'hc_x'), num(r, 'hc_y'), num(r, 'launch_angle'), num(r, 'attack_direction')
                if None in (hx, hy, la, ad) or not (0 <= la < 50):
                    continue
                mir = 1 if r['stand'] == 'R' else -1
                spray = -mir * math.degrees(math.atan2(hx - 125.42, 198.27 - hy))
                rows.append(dict(la=la, spray=spray, p=-ad, stand=r['stand']))
    return rows


def cell(v):
    if len(v) < 2:
        return '(%d)' % len(v)
    return '%+.1f (n=%d, SD %.1f, SE %.1f)' % (st.mean(v), len(v), st.stdev(v), st.stdev(v) / math.sqrt(len(v)))


def main():
    rows = load(sys.argv[1])
    print('Balls in play with spray, launch angle 0-50 and attack direction: %d' % len(rows))
    print('All swings, mean pull-positive attack direction p: %+.1f, mean spray: %+.1f\n' % (st.mean(r['p'] for r in rows), st.mean(r['spray'] for r in rows)))
    for label, sel in (('both hands', lambda r: True), ('right-handed', lambda r: r['stand'] == 'R'), ('left-handed', lambda r: r['stand'] == 'L')):
        sub = [r for r in rows if abs(r['p']) <= W and sel(r)]
        print('### |p| <= %.1f deg, %s: mean p = %+.2f, n = %d' % (W, label, st.mean(r['p'] for r in sub), len(sub)))
        print('\n| launch angle | mean spray (pull +) | mean spray - p |')
        print('|---|---|---|')
        for lo, hi in BANDS:
            s = [r for r in sub if lo <= r['la'] < hi]
            print('| %d-%d | %s | %s |' % (lo, hi, cell([r['spray'] for r in s]), cell([r['spray'] - r['p'] for r in s])))
        s = sub
        print('| 0-50 | %s | %s |\n' % (cell([r['spray'] for r in s]), cell([r['spray'] - r['p'] for r in s])))
    print('### Offset (spray - p) by pull-positive attack direction p, both hands\n')
    print('| p bin | n | mean p | 0-10: spray - p | 10-25: spray - p | 25-35: spray - p |')
    print('|---|---|---|---|---|---|')
    for lo, hi in PBINS:
        sub = [r for r in rows if lo <= r['p'] < hi]
        cs = [cell([r['spray'] - r['p'] for r in sub if a <= r['la'] < b]) for a, b in BANDS[:3]]
        print('| %d to %d | %d | %+.1f | %s |' % (lo, hi, len(sub), st.mean(r['p'] for r in sub), ' | '.join(cs)))
    print('\n### OLS slope of spray on p (all p), by launch band\n')
    for lo, hi in BANDS:
        s = [r for r in rows if lo <= r['la'] < hi]
        xs, ys = [r['p'] for r in s], [r['spray'] for r in s]
        mx, my = st.mean(xs), st.mean(ys)
        sxx = sum((x - mx) ** 2 for x in xs)
        b = sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / sxx
        a = my - b * mx
        se = math.sqrt(sum((y - a - b * x) ** 2 for x, y in zip(xs, ys)) / (len(s) - 2) / sxx)
        print('- launch %d-%d: n = %d, slope %.3f (SE %.3f), intercept (spray at p = 0) %+.2f' % (lo, hi, len(s), b, se, a))


main()
