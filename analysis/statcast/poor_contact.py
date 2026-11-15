#!/usr/bin/env python3
"""Poor contact from competitive swings: real against sim, by contact depth and lateral intercept.

Real: Savant pitch CSVs (sim/target/data/savant/sc_*.csv). Sim: `poor_contact` example CSVs
(one row per contact, fair or foul; columns below). Usage:

  python3 -I analysis/statcast/poor_contact.py sim/target/data/savant [label=sim_a.csv,sim_b.csv ...]

Each `label=` argument is one sim variant, its CSVs pooled. Stdlib only. Prints the tables of
docs/calibration/2026-11-15-collision-q-and-poor-contact.md.

Definitions (both sides):
- competitive swing: bat speed >= 60 mph (savant_sd.py's floor);
- squared-up ratio: exit speed / (1.23 bat + 0.23 release speed) (power-mix §4);
- depth: Savant `intercept_ball_minus_batter_pos_y_inches` (+ out front) minus the batter's own
  mean over his competitive tracked swings (batters with >= 20); sim: the ball's y at contact
  minus the mean for the batter's hand (every sim batter stands in the same place);
- lateral: Savant `intercept_ball_minus_batter_pos_x_inches` (distance from the batter) minus
  the batter's mean; sim: the ball's |x - box x| at contact, minus the hand's mean.
"""
import csv, glob, statistics as st, sys
from collections import defaultdict

EDGES = [80, 85, 90, 95, 100, 105, 110]
LABELS = ['<80', '80-85', '85-90', '90-95', '95-100', '100-105', '105-110', '110+']
DEPTH_BINS = [(-99, -12), (-12, -6), (-6, -2), (-2, 2), (2, 6), (6, 12), (12, 99)]
LAT_BINS = [(-99, -6), (-6, -3), (-3, -1), (-1, 1), (1, 3), (3, 6), (6, 99)]
MIN_SWINGS = 20


def num(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return None


def pct(v, p):
    s = sorted(v)
    return s[min(len(s) - 1, int(p * len(s)))] if s else float('nan')


def ratio(b):
    return b['ev'] / (1.23 * b['bat'] + 0.23 * b['pitch'])


def shares(evs):
    c = [0] * len(LABELS)
    for e in evs:
        i = 0
        while i < len(EDGES) and e >= EDGES[i]:
            i += 1
        c[i] += 1
    return [100 * x / len(evs) for x in c]


def barrel(ev, la):
    if ev < 98:
        return False
    lo = max(8.0, 26 - (ev - 98) * (18 / 18))
    hi = min(50.0, 30 + (ev - 98) * (20 / 18))
    return lo <= la <= hi


def load_real(d):
    rows = []
    for f in sorted(glob.glob(d + '/sc_*.csv')):
        rows.extend(csv.DictReader(open(f)))
    global REAL_FAIR
    REAL_FAIR = [r for r in rows if r['description'].startswith('hit_into_play') and num(r['launch_speed']) is not None
                 and num(r['launch_angle']) is not None]
    swings = []
    for r in rows:
        bat = num(r['bat_speed'])
        y = num(r['intercept_ball_minus_batter_pos_y_inches'])
        x = num(r['intercept_ball_minus_batter_pos_x_inches'])
        if bat is None or y is None or x is None:
            continue
        fair = r['description'].startswith('hit_into_play')
        swings.append(dict(batter=r['batter'], bat=bat, y=y, x=x, fair=fair,
                           ev=num(r['launch_speed']), la=num(r['launch_angle']),
                           pitch=num(r['release_speed']), contact=r['description'] in (
                               'hit_into_play', 'foul', 'foul_tip')))
    comp = [s for s in swings if s['bat'] >= 60]
    by = defaultdict(list)
    for s in comp:
        by[s['batter']].append(s)
    out = []
    for b, ss in by.items():
        if len(ss) < MIN_SWINGS:
            continue
        my = st.mean(s['y'] for s in ss)
        mx = st.mean(s['x'] for s in ss)
        for s in ss:
            s['dy'] = s['y'] - my
            s['dx'] = s['x'] - mx
            out.append(s)
    return out, swings


def load_sim(paths):
    rows = []
    for p in paths:
        rows.extend(csv.DictReader(open(p)))
    out = []
    for r in rows:
        out.append(dict(batter=r['batter'], hand=r['hand'], bat=float(r['bat_mph']), y=float(r['depth_in']),
                        x=float(r['lat_in']), fair=r['fair'] == '1', ev=float(r['ev_mph']),
                        la=float(r['la_deg']), pitch=float(r['pitch_mph']), contact=True,
                        u=float(r['u_rel_in']), terr=float(r['t_err_ms']), effort=float(r['effort']),
                        off=float(r['off_deg']), nominal=float(r['nominal_mph']), hr=r['hr'] == '1',
                        voff=float(r['voff_mm'])))
    for h in ('R', 'L'):
        hs = [s for s in out if s['hand'] == h]
        if not hs:
            continue
        my = st.mean(s['y'] for s in hs)
        mx = st.mean(s['x'] for s in hs)
        for s in hs:
            s['dy'] = s['y'] - my
            s['dx'] = s['x'] - mx
    return [s for s in out if s['bat'] >= 60], out


def by_bin(name, data, key, bins):
    print(f'\n{name}: by {key} bin (in), competitive swings with contact | fair balls')
    print('| bin | contacts | bat mean | fair | ratio mean | >=0.8 | >=0.9 | EV mean | <80 |')
    print('|---|---|---|---|---|---|---|---|---|')
    for lo, hi in bins:
        c = [s for s in data if s['contact'] and lo <= s[key] < hi]
        f = [s for s in c if s['fair'] and s['ev'] is not None and s['pitch']]
        if not c:
            continue
        rr = [ratio(s) for s in f]
        print('| %+d..%+d | %d | %.1f | %d | %.3f | %.0f%% | %.0f%% | %.1f | %.0f%% |' % (
            lo, hi, len(c), st.mean(s['bat'] for s in c), len(f), st.mean(rr) if rr else 0,
            100 * sum(x >= 0.8 for x in rr) / max(1, len(rr)), 100 * sum(x >= 0.9 for x in rr) / max(1, len(rr)),
            st.mean(s['ev'] for s in f) if f else 0, 100 * sum(s['ev'] < 80 for s in f) / max(1, len(f))))
    allc = [s for s in data if s['contact']]
    print('SD of %s among contacts: %.2f in (n %d)' % (key, st.pstdev(s[key] for s in allc), len(allc)))


def summary(name, fair, all_fair=None):
    evs = [s['ev'] for s in fair]
    rr = [ratio(s) for s in fair if s['bat'] >= 60]
    sh = shares(evs)
    print('| %s | %d | ' % (name, len(fair)) + ' | '.join('%.1f' % x for x in sh) +
          ' | %.1f | %.1f | %.1f | %.1f | %.1f | %.1f | %.3f | %.3f | %.0f | %.0f | %.3f |' % (
              st.mean(evs), pct(evs, 0.9), 100 * sum(e >= 95 for e in evs) / len(evs),
              100 * sum(e >= 105 for e in evs) / len(evs),
              100 * sum(barrel(s['ev'], s['la']) for s in fair) / len(fair),
              st.mean(s['la'] for s in fair if s['ev'] >= 100),
              st.mean(rr), st.pstdev(rr), 100 * sum(x >= 0.8 for x in rr) / len(rr),
              100 * sum(x >= 0.9 for x in rr) / len(rr), pct(rr, 0.9)))


def by_bat(name, fair):
    print(f'\n{name}: exit speed of balls squared up >= 0.9, by bat speed (n: mean, p90)')
    for lo, hi in [(60, 66), (66, 70), (70, 74), (74, 78), (78, 99)]:
        e = [s['ev'] for s in fair if lo <= s['bat'] < hi and ratio(s) >= 0.9]
        if e:
            print('  %d-%d: %d: %.1f, %.1f' % (lo, hi, len(e), st.mean(e), pct(e, 0.9)))


def main():
    real, real_all = load_real(sys.argv[1])
    sims = []
    for a in sys.argv[2:]:
        lab, paths = a.split('=', 1)
        sims.append((lab, load_sim(paths.split(','))))
    print('real: competitive tracked swings with an intercept, batters with >= %d: %d' % (MIN_SWINGS, len(real)))
    by_bin('real', real, 'dy', DEPTH_BINS)
    by_bin('real', real, 'dx', LAT_BINS)
    for lab, (comp, _) in sims:
        by_bin(lab, comp, 'dy', DEPTH_BINS)
        by_bin(lab, comp, 'dx', LAT_BINS)
    print('\nFair balls: shares by exit speed, mean, p90, hard-hit, 105+, barrel, LA of 100+, ratio mean/SD/>=0.8/>=0.9/p90 (ratio over bat >= 60)')
    print('| group | n | ' + ' | '.join(LABELS) + ' | EV mean | p90 | 95+ | 105+ | barrel | LA 100+ | ratio | SD | >=0.8 | >=0.9 | p90 |')
    print('|' + '---|' * (len(LABELS) + 13))
    rf = [s for s in real_all if s['fair'] and s['ev'] is not None and s['la'] is not None and s['pitch']]
    summary('real (tracked)', rf)
    for lab, (_, allr) in sims:
        summary(lab, [s for s in allr if s['fair']])
    print('\nHR per fair ball: real %.2f%% (%d fair balls)' % (
        100 * sum(r['events'] == 'home_run' for r in REAL_FAIR) / len(REAL_FAIR), len(REAL_FAIR)))
    for lab, (_, allr) in sims:
        f = [s for s in allr if s['fair']]
        print('  %s %.2f%%' % (lab, 100 * sum(s['hr'] for s in f) / len(f)))
    by_bat('real', [s for s in rf if s['bat'] >= 60])
    for lab, (_, allr) in sims:
        by_bat(lab, [s for s in allr if s['fair'] and s['bat'] >= 60])
    for lab, (_, allr) in sims:
        f = [s for s in allr if s['fair']]
        print(f'\n{lab}: contact station u - u_ss (in, + toward tip), fair balls')
        us = [s['u'] for s in f]
        print('  mean %+.2f SD %.2f; within ±1.2 %.0f%%; >= 5.5 (tip) %.1f%%; < -3 (inboard) %.1f%%' % (
            st.mean(us), st.pstdev(us), 100 * sum(abs(u) <= 1.2 for u in us) / len(us),
            100 * sum(u >= 5.5 for u in us) / len(us), 100 * sum(u < -3 for u in us) / len(us)))
        hist = defaultdict(int)
        for u in us:
            hist[max(-6, min(5, int((u + 100) // 1) - 100))] += 1
        print('  ' + ' '.join('%+d:%.1f' % (k, 100 * hist[k] / len(us)) for k in sorted(hist)))
        poor = [s['u'] for s in f if s['bat'] >= 60 and ratio(s) < 0.8]
        print('  ratio < 0.8: mean u %+.2f, inboard of u_ss %.0f%%, beyond +3 in %.0f%%' % (
            st.mean(poor), 100 * sum(u < 0 for u in poor) / len(poor), 100 * sum(u > 3 for u in poor) / len(poor)))
        allc = allr
        for nm, grp in (('all contacts', allc), ('fair', f)):
            vo = [s['voff'] for s in grp if s['voff'] == s['voff']]
            us2 = [s['u'] for s in grp]
            print('  %s: off-centre offset (mm, + ball above path) mean %+.1f SD %.1f; along-bat SD %.1f mm (Higuchi 2016, pitched 90.7 mph: 19.2 / 53.3 mm; tee 11 / 13 mm)' % (
                nm, st.mean(vo), st.pstdev(vo), 25.4 * st.pstdev(us2)))
        te = [s['terr'] for s in f]
        print('  timing error at contact (ms, + late): mean %+.1f SD %.1f' % (st.mean(te), st.pstdev(te)))


if __name__ == '__main__':
    main()
