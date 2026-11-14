#!/usr/bin/env python3
"""Does the real spray-against-launch-angle slope depend on the swing's slope?

Usage: python3 -I analysis/statcast/savant_slope_by_tilt.py <dir with sc_<date>.csv> [min_swings_per_batter]

Standard library only. Balls in play (description 'hit_into_play') with hit coordinates, launch angle in
[0, 50), attack direction, swing_path_tilt and attack_angle. Pull-positive spray minus attack direction
(same conventions as savant_collision_targets.py). For each split (terciles of the swing's tilt, of the
batter's mean tilt over all his tracked swings in the sample, of the swing's attack angle) it prints, per tercile:
n, the tercile's range, the OLS slope of (spray - AD) on launch angle with its standard error, the band means
(0-10, 10-25, 25-35, 35-50) with SE, and the difference of slopes between the top and bottom terciles with SE.
"""
import csv, glob, math, statistics as st, sys
from collections import defaultdict

MIN_SW = int(sys.argv[2]) if len(sys.argv) > 2 else 30
BANDS = [(0, 10), (10, 25), (25, 35), (35, 50)]


def num(r, k):
    v = r.get(k, '')
    return None if v in ('', 'NA', 'null') else float(v)


def ols(xs, ys):
    n = len(xs)
    mx, my = sum(xs) / n, sum(ys) / n
    sxx = sum((x - mx) ** 2 for x in xs)
    b = sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / sxx
    a = my - b * mx
    rss = sum((y - a - b * x) ** 2 for x, y in zip(xs, ys))
    se = math.sqrt(rss / (n - 2) / sxx)
    return b, se


def load(d):
    rows, tilt_by_batter = [], defaultdict(list)
    for f in sorted(glob.glob(d + '/sc_*.csv')):
        with open(f, newline='', encoding='utf-8-sig') as fh:
            for r in csv.DictReader(fh):
                t = num(r, 'swing_path_tilt')
                if t is not None:
                    tilt_by_batter[r['batter']].append(t)
                if not r['description'].startswith('hit_into_play'):
                    continue
                hx, hy, la = num(r, 'hc_x'), num(r, 'hc_y'), num(r, 'launch_angle')
                ad, aa = num(r, 'attack_direction'), num(r, 'attack_angle')
                if None in (hx, hy, la, ad, aa, t) or not (0 <= la < 50):
                    continue
                mir = 1 if r['stand'] == 'R' else -1
                spray = -mir * math.degrees(math.atan2(hx - 125.42, 198.27 - hy))
                rows.append(dict(batter=r['batter'], la=la, sad=spray - (-ad), tilt=t, aa=aa))
    return rows, tilt_by_batter


def terciles(rows, key):
    vals = sorted(r[key] for r in rows if r[key] is not None)
    c1, c2 = vals[len(vals) // 3], vals[2 * len(vals) // 3]
    return [(lambda r, lo=lo, hi=hi: lo <= r[key] < hi) for lo, hi in ((-1e9, c1), (c1, c2), (c2, 1e9))], (c1, c2)


def report(title, rows, key):
    rows = [r for r in rows if r[key] is not None]
    sel, (c1, c2) = terciles(rows, key)
    print('\n### %s\n' % title)
    print('Cut points: %.1f and %.1f. n = %d balls in play.\n' % (c1, c2, len(rows)))
    print('| tercile | range | n | slope (deg per deg) | SE | ' + ' | '.join('%d-%d: mean (n, SE)' % b for b in BANDS) + ' |')
    print('|---|---|---|---|---|' + '---|' * len(BANDS))
    res = []
    for name, f in zip(('low', 'middle', 'high'), sel):
        sub = [r for r in rows if f(r)]
        b, se = ols([r['la'] for r in sub], [r['sad'] for r in sub])
        res.append((b, se))
        lo, hi = min(r[key] for r in sub), max(r[key] for r in sub)
        cells = []
        for lo_b, hi_b in BANDS:
            v = [r['sad'] for r in sub if lo_b <= r['la'] < hi_b]
            cells.append('%+.1f (%d, %.1f)' % (st.mean(v), len(v), st.pstdev(v) / math.sqrt(len(v))) if len(v) > 1 else '(%d)' % len(v))
        print('| %s | %.1f to %.1f | %d | %+.3f | %.3f | %s |' % (name, lo, hi, len(sub), b, se, ' | '.join(cells)))
    d = res[2][0] - res[0][0]
    se = math.hypot(res[2][1], res[0][1])
    print('\nHigh minus low tercile slope: %+.3f deg per deg (SE %.3f, %.1f sigma); per unit of the splitting variable: %+.4f (range of tercile means %.1f)' % (
        d, se, d / se, d / (st.mean([r[key] for r in rows if sel[2](r)]) - st.mean([r[key] for r in rows if sel[0](r)])),
        st.mean([r[key] for r in rows if sel[2](r)]) - st.mean([r[key] for r in rows if sel[0](r)])))


def main():
    rows, tb = load(sys.argv[1])
    print('Balls in play, launch angle 0-50, with spray, attack direction, tilt and attack angle: %d' % len(rows))
    b, se = ols([r['la'] for r in rows], [r['sad'] for r in rows])
    print('All: slope of (spray - AD) on launch angle %+.3f (SE %.3f)' % (b, se))
    report("Terciles of the swing's own swing_path_tilt (deg)", rows, 'tilt')
    mean_tilt = {k: st.mean(v) for k, v in tb.items() if len(v) >= MIN_SW}
    kept = [dict(r, btilt=mean_tilt.get(r['batter'])) for r in rows]
    have = [r for r in kept if r['btilt'] is not None]
    print('\nBatters with at least %d tracked swings in the sample: %d (their balls in play in the sample: %d of %d)' % (MIN_SW, len(mean_tilt), len(have), len(rows)))
    report("Terciles of the batter's mean swing_path_tilt over all his tracked swings (deg)", kept, 'btilt')
    report("Terciles of the swing's attack angle (deg)", rows, 'aa')


main()
