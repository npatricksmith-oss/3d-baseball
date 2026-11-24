"""Swing length against bat speed, across batters and within a batter (swing v3 spec §14.1; research note 10).
Savant: swing_length = total distance (ft) of the bat head from start of tracking to impact; bat_speed at 6 in from the head.
Competitive swings as savant_intercept.py: each batter's fastest 90% of tracked swings.
Usage: python3 -I analysis/statcast/savant_swing_length.py <dir with sc_*.csv>   (stdlib only)"""
import csv, sys, glob, statistics as st
from collections import defaultdict

def load(d):
    rows = []
    for f in sorted(glob.glob(d + '/sc_*.csv')):
        for r in csv.DictReader(open(f, newline='', encoding='utf-8-sig')):
            try:
                r['bs'] = float(r['bat_speed']); r['sl'] = float(r['swing_length'])
                r['iy'] = float(r['intercept_ball_minus_batter_pos_y_inches'])
                r['ix'] = float(r['intercept_ball_minus_batter_pos_x_inches'])
            except (ValueError, KeyError): continue
            rows.append(r)
    return rows

def ols(X, y):
    """least squares via normal equations (Gauss-Jordan); X rows include the constant if wanted. Returns coefficients."""
    k = len(X[0]); A = [[sum(x[i] * x[j] for x in X) for j in range(k)] + [sum(x[i] * v for x, v in zip(X, y))] for i in range(k)]
    for c in range(k):
        p = max(range(c, k), key=lambda r: abs(A[r][c])); A[c], A[p] = A[p], A[c]
        A[c] = [v / A[c][c] for v in A[c]]
        for r in range(k):
            if r != c: A[r] = [a - A[r][c] * b for a, b in zip(A[r], A[c])]
    return [A[i][k] for i in range(k)]

def corr(a, b):
    ma, mb = st.mean(a), st.mean(b); sa = sum((x - ma) ** 2 for x in a) ** .5; sb = sum((x - mb) ** 2 for x in b) ** .5
    return sum((x - ma) * (y - mb) for x, y in zip(a, b)) / (sa * sb)

rows = load(sys.argv[1])
by = defaultdict(list)
for r in rows: by[r['batter']].append(r['bs'])
cut = {b: sorted(v)[int(.1 * len(v))] for b, v in by.items()}
S = [r for r in rows if r['bs'] >= cut[r['batter']]]
print('tracked rows', len(rows), 'competitive', len(S))
grp = defaultdict(list)
for r in S: grp[(r['batter'], r['stand'])].append(r)
cnt = sorted(len(v) for v in grp.values())
print('batters', len(grp), 'swings per batter: median', cnt[len(cnt)//2], 'p10', cnt[len(cnt)//10], 'p90', cnt[9*len(cnt)//10], 'max', cnt[-1])

for NMIN in (20, 40):
    print(f'\n== 1. ACROSS batters, per-batter means, batters with >= {NMIN} competitive swings')
    B = [(st.mean(r['bs'] for r in v), st.mean(r['sl'] for r in v), st.mean(r['iy'] for r in v), len(v)) for v in grp.values() if len(v) >= NMIN]
    bs, sl, iy = [b[0] for b in B], [b[1] for b in B], [b[2] for b in B]
    c0, c1 = ols([[1, x] for x in bs], sl)
    print(f'batters {len(B)}; bat speed mean {st.mean(bs):.2f} sd {st.pstdev(bs):.2f} mph; swing length mean {st.mean(sl):.3f} sd {st.pstdev(sl):.3f} ft')
    print(f'swing length = {c0:.3f} + {c1:.4f} x bat speed  (ft per mph; {c1*30.48:.1f} cm per mph); r = {corr(bs, sl):.3f}; r with depth {corr(iy, sl):.3f}')
    d = ols([[1, x, y] for x, y in zip(bs, iy)], sl); print(f'  controlling mean contact depth: slope {d[1]:.4f} ft/mph, depth {d[2]:.4f} ft/in')
    print('  by bat-speed band (per-batter mean):')
    for lo, hi in ((0, 67), (67, 69), (69, 71), (71, 73), (73, 75), (75, 99)):
        s = [b for b in B if lo <= b[0] < hi]
        if s: print(f'   [{lo:2d},{hi:2d}) batters {len(s):3d} bat {st.mean(b[0] for b in s):5.2f} mph  len {st.mean(b[1] for b in s):5.3f} ft  depth {st.mean(b[2] for b in s):5.1f} in')
    if NMIN == 20: Bsave = B

print('\n== 2. WITHIN a batter (deviations from his own means; batters >= 20 swings)')
D = []
for v in grp.values():
    if len(v) < 20: continue
    mb, ml, mi = st.mean(r['bs'] for r in v), st.mean(r['sl'] for r in v), st.mean(r['iy'] for r in v)
    D += [(r['bs'] - mb, r['sl'] - ml, r['iy'] - mi, r['bs']) for r in v]
dbs, dsl, diy = [d[0] for d in D], [d[1] for d in D], [d[2] for d in D]
print(f'n {len(D)}; within-batter sd: bat speed {st.pstdev(dbs):.2f} mph, swing length {st.pstdev(dsl):.3f} ft, depth {st.pstdev(diy):.2f} in')
a = ols([[x] for x in dbs], dsl)[0]
print(f'swing length dev = {a:.4f} ft/mph x bat speed dev; r = {corr(dbs, dsl):.3f}   (no depth control)')
b = ols([[x, y] for x, y in zip(dbs, diy)], dsl)
print(f'  with contact depth dev: {b[0]:.4f} ft/mph and {b[1]:.4f} ft/in of depth; r(depth, length) {corr(diy, dsl):.3f}; r(depth, speed) {corr(diy, dbs):.3f}')
print('  at a fixed depth (within +-2 in of his own mean depth):')
F = [d for d in D if abs(d[2]) <= 2]
a2 = ols([[d[0]] for d in F], [d[1] for d in F])[0]
print(f'   n {len(F)} slope {a2:.4f} ft/mph r {corr([d[0] for d in F], [d[1] for d in F]):.3f}')
print('  swing length dev by bat-speed dev band, all depths | fixed depth (mph, ft):')
for lo, hi in ((-99, -6), (-6, -3), (-3, -1), (-1, 1), (1, 3), (3, 99)):
    s = [d for d in D if lo <= d[0] < hi]; f = [d for d in F if lo <= d[0] < hi]
    print(f'   [{lo:3d},{hi:3d}) n {len(s):5d} speed {st.mean(d[0] for d in s):+5.2f} len {st.mean(d[1] for d in s):+6.3f} depth {st.mean(d[2] for d in s):+5.1f} in | fixed n {len(f):5d} len {st.mean(d[1] for d in f):+6.3f}')
print('  same, as % of his mean swing length / speed (log-log slope: elasticity)')
E = []
for v in grp.values():
    if len(v) < 20: continue
    mb, ml, mi = st.mean(r['bs'] for r in v), st.mean(r['sl'] for r in v), st.mean(r['iy'] for r in v)
    E += [(r['bs'] / mb - 1, r['sl'] / ml - 1, r['iy'] - mi) for r in v]
e = ols([[d[0], d[2]] for d in E], [d[1] for d in E]); print(f'   %length = {e[0]:.3f} x %speed + {e[1]*100:.3f}%/in depth  (depth-controlled)')
e0 = ols([[d[0]] for d in E], [d[1] for d in E])[0]; print(f'   %length = {e0:.3f} x %speed (uncontrolled)')

# implied time: length / mean speed. Only a bound: bat speed is at contact, swing length from an unstated tracking start.
print('\n== 3. implied "time" = swing length / bat speed at contact (an upper bound on time if speed rises into contact; not a swing time)')
fts = 1.46667
for lo, hi in ((0, 67), (67, 69), (69, 71), (71, 73), (73, 75), (75, 99)):
    s = [b for b in Bsave if lo <= b[0] < hi]
    if s: print(f'   [{lo:2d},{hi:2d}) bat {st.mean(b[0] for b in s):5.2f} mph len {st.mean(b[1] for b in s):5.3f} ft  len/speed {st.mean(b[1] for b in s)/(st.mean(b[0] for b in s)*fts)*1000:6.1f} ms')
mbs = st.mean(b[0] for b in Bsave); sdb = st.pstdev([b[0] for b in Bsave]); mls = st.mean(b[1] for b in Bsave)
print(f'   all: bat {mbs:.2f} sd {sdb:.2f} mph, len {mls:.3f} ft, len/speed {mls/(mbs*fts)*1000:.1f} ms')
# model arithmetic
c0, c1 = ols([[1, b[0]] for b in Bsave], [b[1] for b in Bsave])
w = ols([[x] for x in dbs], dsl)[0]
print('\n== model arithmetic: batter k sd slow in bat speed (sd of per-batter mean = %.2f mph, in swing speed %.1f%%)' % (sdb, sdb / mbs * 100))
for k in (1, 2):
    ds = -k * sdb; v = mbs + ds
    L_across = mls + c1 * ds
    # time if mean speed over swing ~ proportional to contact speed (same shape): t ~ L/v
    t0 = mls / mbs; t1 = L_across / v
    print(f'  {k} sd slow ({v:.1f} mph, {ds/mbs*100:+.1f}%): length across-batters {c1*ds:+.3f} ft ({c1*ds/mls*100:+.1f}%), time scale {t1/t0:.3f} (swing time {0.150*t1/t0:.3f} s from 0.150 s);'
          f' length if length followed speed 1:1 {mls*ds/mbs:+.3f} ft; within-batter slope {w*ds:+.3f} ft')
