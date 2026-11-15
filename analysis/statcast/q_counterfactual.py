#!/usr/bin/env python3
"""Collision-efficiency counterfactual tables and contact-station histograms.

Real: Savant pitch CSVs (sim/target/data/savant/sc_*.csv). Sim: `q_contact` example CSVs, one set per
BB_E_REF value (see docs/calibration/2026-11-15-q-counterfactual.md for how they were produced).

  python3 -I analysis/statcast/q_counterfactual.py sim/target/data/savant sim/target/data/q \
      0.442 0.460 0.478

Reads <dir>/qc_<e_ref>_0.csv and qc_<e_ref>_1000.csv. Stdlib only.
"""
import csv, glob, random, statistics as st, sys

EDGES = [80, 85, 90, 95, 100, 105, 110]
LABELS = ['<80', '80-85', '85-90', '90-95', '95-100', '100-105', '105-110', '110+']


def num(x):
    return None if x in ('', 'NA') else float(x)


def pct(v, p):
    s = sorted(v)
    return s[min(len(s) - 1, int(p * len(s)))]


def shares(evs):
    c = [0] * len(LABELS)
    for e in evs:
        i = 0
        while i < len(EDGES) and e >= EDGES[i]:
            i += 1
        c[i] += 1
    return [100 * x / len(evs) for x in c]


def barrel(ev, la):
    """Approximate Savant barrel: EV >= 98, LA window 26-30 at 98 mph widening linearly to 8-50 at 116."""
    if ev < 98:
        return False
    d = min(ev, 116) - 98
    return 26 - d * 18 / 18 <= la <= 30 + d * 20 / 18


def sq(b):
    return b['ev'] / (1.23 * b['bat'] + 0.23 * b['pitch'])


# ---------------------------------------------------------------- real
real = []
for f in sorted(glob.glob(sys.argv[1] + '/sc_*.csv')):
    with open(f, newline='', encoding='utf-8-sig') as fh:
        for r in csv.DictReader(fh):
            if not r['description'].startswith('hit_into_play'):
                continue
            ev, la, pv = num(r['launch_speed']), num(r['launch_angle']), num(r['release_speed'])
            if ev is None or la is None:
                continue
            real.append(dict(ev=ev, la=la, bat=num(r['bat_speed']), pitch=pv, hr=r['events'] == 'home_run'))
print('real fair balls %d; with bat_speed %d' % (len(real), sum(b['bat'] is not None for b in real)))


def load(e):
    out = []
    for s in (0, 1000):
        with open('%s/qc_%s_%d.csv' % (sys.argv[2], e, s)) as fh:
            for r in csv.DictReader(fh):
                out.append(dict(ev=float(r['ev_mph']), la=float(r['la_deg']), bat=float(r['bat_mph']), pitch=float(r['pitch_mph']),
                                hr=r['hr'] == '1', u=float(r['u_rel_in']), off=float(r['off_deg']), nz=float(r['nz']), nom=float(r['nominal_mph'])))
    return out


variants = sys.argv[3:]
sims = {e: load(e) for e in variants}


def line(name, g):
    evs = [b['ev'] for b in g]
    s = shares(evs)
    print('| %s | %d | ' % (name, len(g)) + ' | '.join('%.1f' % x for x in s) + ' | %.1f | %.1f | %.1f | %.1f | %.1f | %.1f | %.1f | %.2f%% | %.1f%% |' % (
        st.mean(evs), pct(evs, .9), 100 * sum(e >= 95 for e in evs) / len(evs), 100 * sum(e >= 100 for e in evs) / len(evs),
        100 * sum(e >= 105 for e in evs) / len(evs), 100 * sum(e >= 110 for e in evs) / len(evs),
        100 * sum(barrel(b['ev'], b['la']) for b in g) / len(g), 100 * sum(b['hr'] for b in g) / len(g), 100 * sum(b['la'] < 10 for b in g) / len(g)))


print('\n## 1. Exit-speed rows (fair balls)')
print('| group | n | ' + ' | '.join(LABELS) + ' | EV mean | EV p90 | 95+ (hard-hit) | 100+ | 105+ | 110+ | barrel* | HR/fair ball | LA<10° share |')
print('|' + '---|' * (len(LABELS) + 12))
line('real (Savant, 12 dates)', real)
for e in variants:
    line('sim e_ref %s' % e, sims[e])

print('\n## 2. Squared-up ratio ev/(1.23 bat + 0.23 release)')
print('| group | n | ratio mean | SD | share >= 0.8 | share >= 0.9 | p90 | EV mean at >= 0.9 |')
print('|---|---|---|---|---|---|---|---|')


def sqrow(name, g):
    g = [b for b in g if b['bat'] is not None and b['pitch'] is not None]
    x = [sq(b) for b in g]
    hi = [b['ev'] for b in g if sq(b) >= .9]
    print('| %s | %d | %.3f | %.3f | %.0f%% | %.0f%% | %.3f | %.1f |' % (name, len(g), st.mean(x), st.pstdev(x), 100 * sum(v >= .8 for v in x) / len(x),
                                                                            100 * sum(v >= .9 for v in x) / len(x), pct(x, .9), st.mean(hi)))


sqrow('real', real)
for e in variants:
    sqrow('sim e_ref %s' % e, sims[e])

print('\n## 3. EV of balls squared up >= 0.9, by bat speed band (n: mean, p90)')
bands = ((60, 66), (66, 70), (70, 74), (74, 78), (78, 100))
print('| bat speed | real | ' + ' | '.join('sim ' + e for e in variants) + ' |')
print('|---|---|' + '---|' * len(variants))
for lo, hi in bands:
    cells = []
    for g in [real] + [sims[e] for e in variants]:
        v = [b['ev'] for b in g if b['bat'] is not None and b['pitch'] and lo <= b['bat'] < hi and sq(b) >= .9]
        cells.append('%d: %.1f, %.1f' % (len(v), st.mean(v), pct(v, .9)))
    print('| %d-%d | ' % (lo, hi) + ' | '.join(cells) + ' |')

# ---------------------------------------------------------------- contact station
base = sims[variants[0]]
print('\n## 4. Contact station relative to the sweet spot (inches, + toward the tip), baseline %s, %d balls' % (variants[0], len(base)))
u = [b['u'] for b in base]
print('mean %.2f in, SD %.2f in, median %.2f; share within +-1.2 in (the 2.4-in zone) %.1f%%, within +-2 in %.1f%%, within +-3 in %.1f%%' % (
    st.mean(u), st.pstdev(u), st.median(u), *[100 * sum(abs(x) <= w for x in u) / len(u) for w in (1.2, 2, 3)]))
groups = (('all', lambda b: True), ('ratio < 0.8', lambda b: sq(b) < .8), ('0.8 - 0.9', lambda b: .8 <= sq(b) < .9), ('>= 0.9', lambda b: sq(b) >= .9))
print('| bin (in from sweet spot) | ' + ' | '.join(n for n, _ in groups) + ' |')
print('|---|' + '---|' * len(groups))
edges = [-99] + [x * 0.5 for x in range(-12, 13)] + [99]
for lo, hi in zip(edges[:-1], edges[1:]):
    cells = []
    for _, f in groups:
        g = [b for b in base if f(b)]
        cells.append('%.1f' % (100 * sum(lo <= b['u'] < hi for b in g) / len(g)))
    lab = ('< %.1f' % hi) if lo == -99 else (('>= %.1f' % lo) if hi == 99 else '%.1f to %.1f' % (lo, hi))
    print('| %s | ' % lab + ' | '.join(cells) + ' |')
print('| n | ' + ' | '.join(str(sum(1 for b in base if f(b))) for _, f in groups) + ' |')
print('| mean | ' + ' | '.join('%.2f' % st.mean(b['u'] for b in base if f(b)) for _, f in groups) + ' |')
print('| SD | ' + ' | '.join('%.2f' % st.pstdev([b['u'] for b in base if f(b)]) for _, f in groups) + ' |')
print('| share within +-1.2 in | ' + ' | '.join('%.1f%%' % (100 * sum(abs(b['u']) <= 1.2 for b in base if f(b)) / sum(1 for b in base if f(b))) for _, f in groups) + ' |')
print('\noff-centre angle between contact normal and bat-point velocity (deg), and normal vertical component nz:')
print('| group | off mean | off SD | off p50 | off p90 | nz mean | nz SD | EV mean |')
print('|---|---|---|---|---|---|---|---|')
for n, f in groups:
    g = [b for b in base if f(b)]
    o = [b['off'] for b in g if b['off'] == b['off']]
    z = [b['nz'] for b in g if b['nz'] == b['nz']]
    print('| %s | %.1f | %.1f | %.1f | %.1f | %.3f | %.3f | %.1f |' % (n, st.mean(o), st.pstdev(o), pct(o, .5), pct(o, .9), st.mean(z), st.pstdev(z), st.mean(b['ev'] for b in g)))
print('\nEV and ratio by contact station (baseline), 1-in bins:')
print('| station | n | share | EV mean | ratio mean | bat-speed mean |')
print('|---|---|---|---|---|---|')
for lo in range(-6, 6):
    g = [b for b in base if lo <= b['u'] < lo + 1]
    if g:
        print('| %d to %d | %d | %.1f%% | %.1f | %.3f | %.1f |' % (lo, lo + 1, len(g), 100 * len(g) / len(base), st.mean(b['ev'] for b in g), st.mean(sq(b) for b in g), st.mean(b['bat'] for b in g)))

# ---------------------------------------------------------------- slow swings
print('\n## 5. Slow swings (< 60 mph bat speed), real fair balls')
rb = [b for b in real if b['bat'] is not None]
slow = [b for b in rb if b['bat'] < 60]
fast = [b for b in rb if b['bat'] >= 60]
w = len(slow) / len(rb)
print('real balls with tracked bat speed %d; slow %d = %.2f%%; slow EV mean %.1f, share <80 %.1f%%, p90 %.1f; competitive EV mean %.1f, <80 share %.1f%%' % (
    len(rb), len(slow), 100 * w, st.mean(b['ev'] for b in slow), shares([b['ev'] for b in slow])[0], pct([b['ev'] for b in slow], .9),
    st.mean(b['ev'] for b in fast), shares([b['ev'] for b in fast])[0]))
random.seed(1)
print('| sim variant | assumption | <80 share | mean EV | p90 | 95+ | 105+ |')
print('|---|---|---|---|---|---|---|')
for e in variants:
    s = [b['ev'] for b in sims[e]]
    print('| %s | as is | %.1f | %.1f | %.1f | %.1f | %.1f |' % (e, shares(s)[0], st.mean(s), pct(s, .9), 100 * sum(x >= 95 for x in s) / len(s), 100 * sum(x >= 105 for x in s) / len(s)))
    for wt in (0.037, 0.07, 0.08):
        # A: slow-swing fraction wt of fair balls has the real slow-swing exit-speed distribution; the rest are the sim's balls.
        # Expected-value mix (exact, no sampling): weights (1-wt) on sim, wt on real slow balls.
        sl = [b['ev'] for b in slow]
        mean = (1 - wt) * st.mean(s) + wt * st.mean(sl)
        sh = (1 - wt) * shares(s)[0] + wt * shares(sl)[0]
        ge = lambda x, t: 100 * sum(v >= t for v in x) / len(x)
        print('| %s | A: %.0f%% slow balls = real slow-swing EVs | %.1f | %.1f | n/a | %.1f | %.1f |' % (
            e, 100 * wt, sh, mean, (1 - wt) * ge(s, 95) + wt * ge(sl, 95), (1 - wt) * ge(s, 105) + wt * ge(sl, 105)))
        # B: the slow-swing balls get the sim's own contact-quality ratio on a real slow bat speed and the sim's pitch speed
        slb = [b['bat'] for b in slow]
        bs = [sq(b) * (1.23 * random.choice(slb) + 0.23 * b['pitch']) for b in sims[e]]
        mean = (1 - wt) * st.mean(s) + wt * st.mean(bs)
        sh = (1 - wt) * shares(s)[0] + wt * shares(bs)[0]
        print('| %s | B: %.0f%% slow, sim ratio on a real slow bat speed | %.1f | %.1f | n/a | %.1f | %.1f |' % (
            e, 100 * wt, sh, mean, (1 - wt) * ge(s, 95) + wt * ge(bs, 95), (1 - wt) * ge(s, 105) + wt * ge(bs, 105)))
# real less its slow swings
ev_f = [b['ev'] for b in fast]
print('real, competitive swings only (bat >= 60 mph): <80 %.1f%%, mean %.1f, 95+ %.1f%%, 105+ %.1f%%' % (
    shares(ev_f)[0], st.mean(ev_f), 100 * sum(x >= 95 for x in ev_f) / len(ev_f), 100 * sum(x >= 105 for x in ev_f) / len(ev_f)))

print('\nsim balls with bat speed < 60 mph: ' + ', '.join('%s %.2f%%' % (e, 100 * sum(b['bat'] < 60 for b in sims[e]) / len(sims[e])) for e in variants))
print('sim balls with contact at the tip clamp (u - u_ss >= 5.9 in): ' + ', '.join('%s %.1f%%' % (e, 100 * sum(b['u'] >= 5.9 for b in sims[e]) / len(sims[e])) for e in variants))
print('sim: share of balls by station 0.442/0.460/0.478 within +-1.2 in: ' + ', '.join('%.1f%%' % (100 * sum(abs(b['u']) <= 1.2 for b in sims[e]) / len(sims[e])) for e in variants))
