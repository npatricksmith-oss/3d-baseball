#!/usr/bin/env python3
"""League means/sds of Savant's Batting Stance & Intercept leaderboard (per-batter averages).
Input: page saved by curl from
 https://baseballsavant.mlb.com/visuals/batting-stance?seasonStart=2025&seasonEnd=2025&minSwings=q&gameType=Regular
(the table is embedded as `const vizData = [...]`). Usage: savant_stance_depth.py <page.html>
Units on the page: inches. Output also in the game frame (metres)."""
import sys, json, statistics as st
s = open(sys.argv[1], encoding='utf8').read()
i = s.find('const vizData = ') + len('const vizData = ')
d = json.JSONDecoder().raw_decode(s[i:])[0]
IN = 0.0254
PLATE_FRONT_M, HALF_PLATE_M = 0.432, 0.2159   # game frame: front edge y; half the plate's width
# One row per (batter, side): a batter who moved teams has one row per team plus a
# 'zMLB' total row; keep the total, else the single row.
by = {}
for r in d:
    k = (r['savant_batter_id'], r['bat_side'])
    if k not in by or r['team_name'] == 'zMLB':
        by[k] = r
d = list(by.values())
FIELDS = [('avg_batter_y_position', 'depth in box'), ('avg_batter_x_position', 'off plate'),
          ('avg_foot_sep0', 'foot sep'), ('avg_foot_angle0', 'stance angle'),
          ('avg_intercept_y_in_vs_plate', 'intercept y vs front of plate'),
          ('avg_intercept_y_vs_batter', 'intercept y vs COM'),
          ('avg_intercept_x_in', 'intercept x (in)')]
def stats(rows, k):
    v = [r[k] for r in rows if r.get(k) is not None]
    return len(v), st.mean(v), (st.stdev(v) if len(v) > 1 else 0.0)
print(f'rows {len(d)}')
for label, rows in [('all', d)] + [(h, [r for r in d if r['bat_side'] == h]) for h in 'LR']:
    print(f'-- {label}')
    for k, nm in FIELDS:
        if k in rows[0]:
            n, m, sd = stats(rows, k)
            print(f'{nm:32s} n={n:3d} mean {m:7.2f} sd {sd:6.2f}  ({m*IN:6.3f} m, sd {sd*IN:.3f} m)')

def corr(a, b):
    ma, mb = st.mean(a), st.mean(b)
    return sum((x-ma)*(y-mb) for x, y in zip(a, b)) / (sum((x-ma)**2 for x in a) * sum((y-mb)**2 for y in b)) ** 0.5
dep = [r['avg_batter_y_position'] for r in d]; icp = [r['avg_intercept_y_in_vs_plate'] for r in d]
print(f'corr(depth, intercept vs plate) = {corr(dep, icp):.2f}')
md, mi = st.mean(dep), st.mean(icp)
print(f'game frame: COM y at stance {PLATE_FRONT_M - md*IN:.3f} m; ball met at y {PLATE_FRONT_M + mi*IN:.3f} m; '
      f'COM off plate middle {md and st.mean(r["avg_batter_x_position"] for r in d)*IN + HALF_PLATE_M:.3f} m')
