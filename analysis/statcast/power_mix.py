#!/usr/bin/env python3
"""Who supplies the 105+ mph contact: the batter population or the contact model?

Real: Savant pitch-level CSVs (`sim/target/data/savant/sc_*.csv`, as ev_la_bins.py).
Sim: `ev_bat` example output (one row per fair ball: game,batter,bat_mph,pitch_mph,ev_mph,la_deg,...).

  python3 analysis/statcast/power_mix.py sim/target/data/savant sim/target/data/bat_0.csv sim/target/data/bat_1000.csv

Stdlib only. Prints the tables used in docs/calibration/2026-11-15-power-mix.md.
"""
import csv, glob, sys, statistics as st
from collections import defaultdict

MIN_SWINGS = 20          # tracked swings a batter needs for a bat-speed mean
EDGES = [80, 85, 90, 95, 100, 105, 110]
LABELS = ['<80', '80-85', '85-90', '90-95', '95-100', '100-105', '105-110', '110+']


def num(x):
    return None if x in ('', 'NA') else float(x)


def bin_ix(ev):
    for i, e in enumerate(EDGES):
        if ev < e:
            return i
    return len(EDGES)


def pct(v, p):
    s = sorted(v)
    return s[min(len(s) - 1, int(p * len(s)))]


def shares(evs):
    c = [0] * len(LABELS)
    for e in evs:
        c[bin_ix(e)] += 1
    return [100 * x / len(evs) for x in c]


def tail(evs):
    return [100 * sum(e >= t for e in evs) / len(evs) for t in (95, 100, 105, 110)]


def row(name, balls):
    evs = [b['ev'] for b in balls]
    s = shares(evs)
    t = tail(evs)
    hard = [b['la'] for b in balls if b['ev'] >= 100]
    la100 = (st.mean(hard), 100 * sum(20 <= x <= 40 for x in hard) / len(hard)) if hard else (float('nan'),) * 2
    print('| %s | %d | ' % (name, len(balls)) + ' | '.join('%.1f' % x for x in s) + ' | %.1f | %.1f | %.1f | %.1f | %.1f | %.1f | %.1f° | %.0f%% |' % (
        st.mean(evs), t[0], t[1], t[2], t[3], pct(evs, 0.9), la100[0], la100[1]))


HDR = '| group | balls | ' + ' | '.join(LABELS) + ' | EV mean | 95+ | 100+ | 105+ | 110+ | EV p90 | LA of 100+ | 100+ at 20-40° |'
SEP = '|' + '---|' * (len(LABELS) + 11)


def squared(b):
    return b['ev'] / (1.23 * b['bat'] + 0.23 * b['pitch'])


# ---------------------------------------------------------------- real
swings = defaultdict(list)   # batter -> bat speeds of competitive tracked swings
balls = []
for f in sorted(glob.glob(sys.argv[1] + '/sc_*.csv')):
    with open(f, newline='', encoding='utf-8-sig') as fh:
        for r in csv.DictReader(fh):
            bs = num(r['bat_speed'])
            if bs is not None and bs >= 60:      # competitive swings only (savant_sd.py's floor)
                swings[r['batter']].append(bs)
            ev, la, pv = num(r['launch_speed']), num(r['launch_angle']), num(r['release_speed'])
            if r['description'].startswith('hit_into_play') and ev is not None and la is not None:
                balls.append(dict(batter=r['batter'], ev=ev, la=la, bat=bs, pitch=pv, hr=r['events'] == 'home_run'))
bat_mean = {b: st.mean(v) for b, v in swings.items() if len(v) >= MIN_SWINGS}
print('real: %d fair balls; %d batters with >=%d competitive tracked swings (%d swings)' % (
    len(balls), len(bat_mean), MIN_SWINGS, sum(len(swings[b]) for b in bat_mean)))
within = (sum(st.pvariance(swings[b]) * len(swings[b]) for b in bat_mean) / sum(len(swings[b]) for b in bat_mean)) ** 0.5
nbar = st.mean(len(swings[b]) for b in bat_mean)
raw_sd = st.pstdev(bat_mean.values())
print('between-batter SD of bat-speed means %.2f mph (raw), sampling %.2f, true %.2f; within-batter SD %.2f; mean %.1f' % (
    raw_sd, within / nbar ** 0.5, max(raw_sd ** 2 - within ** 2 / nbar, 0) ** 0.5, within, st.mean(bat_mean.values())))

rb = [b for b in balls if b['batter'] in bat_mean]
print('balls by batters with a bat-speed mean: %d of %d' % (len(rb), len(balls)))


def quantile_groups(items, key, k):
    """Split balls into k groups by their batter's key, equal numbers of balls per group."""
    order = sorted(items, key=key)
    n = len(order)
    return [order[i * n // k:(i + 1) * n // k] for i in range(k)]


print('\n## Real, by batter bat-speed quintile (batter mean competitive bat speed)')
print(HDR); print(SEP)
q5 = quantile_groups(rb, lambda b: bat_mean[b['batter']], 5)
for i, g in enumerate(q5):
    lo, hi = bat_mean[g[0]['batter']], bat_mean[g[-1]['batter']]
    row('Q%d (%.1f-%.1f mph)' % (i + 1, lo, hi), g)
row('all real', rb)
print('\n## Real, by thirds')
print(HDR); print(SEP)
q3 = quantile_groups(rb, lambda b: bat_mean[b['batter']], 3)
for i, g in enumerate(q3):
    row('T%d (%.1f-%.1f)' % (i + 1, bat_mean[g[0]['batter']], bat_mean[g[-1]['batter']]), g)

print('\n## Share of all real balls at EV >= t that come from each third of batters, and each third\'s own share of balls')
for t in (100, 105, 110):
    tot = sum(b['ev'] >= t for b in rb)
    print('EV >= %d (n %d): ' % (t, tot) + ', '.join('T%d %.0f%%' % (i + 1, 100 * sum(b['ev'] >= t for b in g) / tot) for i, g in enumerate(q3)))

# batters by own mean EV (circular for the tail, so only mean EV on >=10 balls)
byb = defaultdict(list)
for b in rb:
    byb[b['batter']].append(b['ev'])
big = {b for b, v in byb.items() if len(v) >= 10}
print('\n## Real, by batter\'s own mean exit speed (batters with >=10 balls: %d, %d balls); split-half to avoid circularity' % (
    len(big), sum(len(byb[b]) for b in big)))
# classify by even-indexed balls, tabulate the odd-indexed ones
cls, test = {}, []
for b in big:
    v = [x for x in rb if x['batter'] == b]
    cls[b] = st.mean(x['ev'] for x in v[0::2])
    test += v[1::2]
print(HDR); print(SEP)
for i, g in enumerate(quantile_groups(test, lambda x: cls[x['batter']], 5)):
    row('EV-Q%d (%.1f-%.1f)' % (i + 1, cls[g[0]['batter']], cls[g[-1]['batter']]), g)

# ---------------------------------------------------------------- sim
sim = []
for ix, path in enumerate(sys.argv[2:]):
    with open(path) as fh:
        for r in csv.DictReader(fh):
            sim.append(dict(batter='%d/%s/%s' % (ix, r['game'], r['batter']), nominal=float(r['nominal_mph']), ev=float(r['ev_mph']), la=float(r['la_deg']), bat=float(r['bat_mph']),
                            pitch=float(r['pitch_mph']), hr=r['hr'] == '1'))
sbat = defaultdict(list)
nom = {}
for b in sim:
    sbat[b['batter']].append(b['bat'])
    nom[b['batter']] = b['nominal']
sm = nom     # the batter's drawn BatSpeed (mph): known exactly in the sim
nv = list(nom.values())
swi = (sum(st.pvariance(v) * len(v) for v in sbat.values()) / sum(len(v) for v in sbat.values())) ** 0.5
print('\nsim: %d fair balls from %d batters (batter-games, ball-weighted); drawn BatSpeed mean %.2f SD %.2f mph (unweighted over batters); ball-weighted SD %.2f; within-batter SD of contact bat speed %.2f; mean contact bat speed %.1f' % (
    len(sim), len(nom), st.mean(nv), st.pstdev(nv), st.pstdev([b['nominal'] for b in sim]), swi, st.mean(b['bat'] for b in sim)))
print('\n## Sim, by quintile of the drawn BatSpeed')
print(HDR); print(SEP)
sb = sim
sq5 = quantile_groups(sb, lambda b: b['nominal'], 5)
for i, g in enumerate(sq5):
    row('Q%d (%.1f-%.1f mph)' % (i + 1, g[0]['nominal'], g[-1]['nominal']), g)
row('all sim', sim)
sq3 = quantile_groups(sb, lambda b: b['nominal'], 3)
for t in (100, 105, 110):
    tot = sum(b['ev'] >= t for b in sb)
    print('sim EV >= %d (n %d): ' % (t, tot) + ', '.join('T%d %.0f%%' % (i + 1, 100 * sum(b['ev'] >= t for b in g) / tot) for i, g in enumerate(sq3)))

print('\n## Comparison: real middle quintile against sim middle quintile and sim all')
print(HDR); print(SEP)
row('real Q3', q5[2]); row('sim Q3', sq5[2]); row('real all', rb); row('sim all', sim)

# ---------------------------------------------------------------- bat speed at contact, squared-up
print('\n## Bat speed at contact (fair balls) and squared-up ratio ev/(1.23 bat + 0.23 pitch)')
print('| group | n | bat mean | bat SD | bat p90 | sq mean | sq SD | sq>=0.8 | sq>=0.9 | sq p90 | EV mean at sq>=0.9 |')
print('|---|---|---|---|---|---|---|---|---|---|---|')


def brow(name, g):
    g = [b for b in g if b['bat'] is not None and b['pitch'] is not None]
    bs = [b['bat'] for b in g]
    sq = [squared(b) for b in g]
    hi = [b['ev'] for b in g if squared(b) >= 0.9]
    print('| %s | %d | %.1f | %.2f | %.1f | %.3f | %.3f | %.0f%% | %.0f%% | %.3f | %.1f |' % (
        name, len(g), st.mean(bs), st.pstdev(bs), pct(bs, 0.9), st.mean(sq), st.pstdev(sq),
        100 * sum(x >= 0.8 for x in sq) / len(sq), 100 * sum(x >= 0.9 for x in sq) / len(sq), pct(sq, 0.9), st.mean(hi)))


brow('real Q3', q5[2]); brow('sim Q3', sq5[2]); brow('real all', rb); brow('sim all', sim)
# bat-speed-restricted: sim Q3 vs real Q3 EV given similar bat speed, squared-up balls only
print('\nEV of balls with squared-up >= 0.9 in bat-speed windows (mph): real, sim')
for lo, hi in ((60, 66), (66, 70), (70, 74), (74, 78), (78, 100)):
    r = [b['ev'] for b in rb if b['bat'] is not None and lo <= b['bat'] < hi and b['pitch'] and squared(b) >= 0.9]
    s = [b['ev'] for b in sim if lo <= b['bat'] < hi and squared(b) >= 0.9]
    if r and s:
        print('bat %d-%d: real n %d mean %.1f p90 %.1f | sim n %d mean %.1f p90 %.1f' % (lo, hi, len(r), st.mean(r), pct(r, .9), len(s), st.mean(s), pct(s, .9)))

# ---------------------------------------------------------------- extras
print('\n## Sim by thirds of drawn BatSpeed')
print(HDR); print(SEP)
for i, g in enumerate(sq3):
    row('T%d (%.1f-%.1f)' % (i + 1, g[0]['nominal'], g[-1]['nominal']), g)

print('\n## Contact bat speed, competitive swings only (>= 60 mph), fair balls')
for name, g in (('real Q3', q5[2]), ('real all', rb), ('sim Q3', sq5[2]), ('sim all', sim)):
    v = [b['bat'] for b in g if b['bat'] is not None and b['bat'] >= 60]
    print('%s: n %d (%.0f%% of balls) mean %.1f SD %.2f p10 %.1f p90 %.1f' % (name, len(v), 100 * len(v) / len(g), st.mean(v), st.pstdev(v), pct(v, .1), pct(v, .9)))

# cluster bootstrap over batters (real) and over games (sim) for the middle-quintile 105+ share
import random
random.seed(1)
def boot(groups_by_key, stat, n=400):
    keys = list(groups_by_key)
    out = []
    for _ in range(n):
        s = [b for k in random.choices(keys, k=len(keys)) for b in groups_by_key[k]]
        out.append(stat(s))
    out.sort()
    return out[int(.025 * n)], out[int(.975 * n)]
def by(items, f):
    d = defaultdict(list)
    for b in items:
        d[f(b)].append(b)
    return d
s105 = lambda s: 100 * sum(b['ev'] >= 105 for b in s) / len(s)
print('\n## 95%% cluster-bootstrap intervals for the 105+ share: real Q3 %.1f %s (by batter); sim Q3 %.1f %s (by game); sim all %.1f %s' % (
    s105(q5[2]), '[%.1f, %.1f]' % boot(by(q5[2], lambda b: b['batter']), s105), s105(sq5[2]),
    '[%.1f, %.1f]' % boot(by(sq5[2], lambda b: b['batter'].split('/')[0] + b['batter'].split('/')[1]), s105), s105(sim),
    '[%.1f, %.1f]' % boot(by(sim, lambda b: b['batter'].split('/')[0] + b['batter'].split('/')[1]), s105)))
