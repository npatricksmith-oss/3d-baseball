"""Where the ball is met relative to the batter (swing v3 spec step 0, Q46) and bat speed by depth.
Savant columns intercept_ball_minus_batter_pos_{x,y}_inches: distance between the bat/ball intercept
point and the batter's center of mass, X horizontal, Y mound-to-plate (csv-docs; no origin or sign stated).
Usage: python3 -I analysis/statcast/savant_intercept.py <dir with sc_*.csv>  (see README for the fetch)"""
import csv, sys, glob, statistics as st
from collections import defaultdict
rows = []
for f in sorted(glob.glob(sys.argv[1] + '/sc_*.csv')):
    for r in csv.DictReader(open(f, newline='', encoding='utf-8-sig')):
        try:
            r['bs'] = float(r['bat_speed']); r['ix'] = float(r['intercept_ball_minus_batter_pos_x_inches'])
            r['iy'] = float(r['intercept_ball_minus_batter_pos_y_inches']); r['px'] = float(r['plate_x']); r['pz'] = float(r['plate_z'])
        except (ValueError, KeyError): continue
        rows.append(r)
by = defaultdict(list)
for r in rows: by[r['batter']].append(r['bs'])
cut = {b: sorted(v)[int(.1 * len(v))] for b, v in by.items()}
S = [r for r in rows if r['bs'] >= cut[r['batter']]]
M = 0.0254
def line(l, v): print(f'{l:34s} n {len(v):6d} mean {st.mean(v):6.2f} in = {st.mean(v)*M:5.3f} m  sd {st.pstdev(v):5.2f} in  p10 {sorted(v)[len(v)//10]:5.1f} p90 {sorted(v)[9*len(v)//10]:5.1f}')
print('competitive swings (fastest 90% per batter):', len(S))
for stand in ('R', 'L'):
    sel = [r for r in S if r['stand'] == stand]
    line(f'{stand} intercept x (signed)', [r['ix'] for r in sel]); line(f'{stand} intercept y', [r['iy'] for r in sel])
line('all intercept y (out front +)', [r['iy'] for r in S]); line('all |intercept x|', [abs(r['ix']) for r in S])
# per-batter means: between-batter spread
pb = defaultdict(list)
for r in S: pb[(r['batter'], r['stand'])].append(r['iy'])
m = [st.mean(v) for v in pb.values() if len(v) >= 20]
print('per-batter mean y: n batters', len(m), 'mean %.1f sd %.1f in' % (st.mean(m), st.pstdev(m)))
# y against height and inside location (does the depth depend on where the pitch is?)
def inside(r): return -r['px'] if r['stand'] == 'R' else r['px']
for lo, hi, l in ((-9, -.55, 'outside'), (-.55, .55, 'middle'), (.55, 9, 'inside')):
    sel = [r['iy'] for r in S if lo <= inside(r) < hi]; print(f'y, {l:8s} n {len(sel):5d} mean {st.mean(sel):5.1f} in')
# bat speed by depth relative to the batter's own mean
mb = {k: st.mean(v) for k, v in pb.items() if len(v) >= 20}
print('bat speed by y relative to the batter mean (in, + = out front); sweet-spot speed, mph')
allm = st.mean(r['bs'] for r in S)
for lo, hi in ((-99, -12), (-12, -6), (-6, -2), (-2, 2), (2, 6), (6, 12), (12, 99)):
    sel = [r['bs'] for r in S if (r['batter'], r['stand']) in mb and lo <= r['iy'] - mb[(r['batter'], r['stand'])] < hi]
    print(f'[{lo:4d},{hi:4d}) n {len(sel):5d} bat {st.mean(sel):5.2f}  vs the 2..6 in row {st.mean(sel) - st.mean([r["bs"] for r in S if (r["batter"], r["stand"]) in mb and 2 <= r["iy"] - mb[(r["batter"], r["stand"])] < 6]):+5.2f}')
