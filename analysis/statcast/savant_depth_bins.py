"""MLB (Savant, the six 2025 dates of docs/research/swing/9-step0-data.md): bat speed, attack angle and swing-path
tilt by contact depth relative to the batter's own mean (intercept_ball_minus_batter_pos_y_inches), in Savant's
depth bins, competitive swings (each batter's fastest 90%). Spec swing-model-v3 §6.7.6 (PLAN D30).
Run from the repository root: python3 -I analysis/statcast/savant_depth_bins.py (reads sim/target/research-dl/savant)."""
import csv, glob, collections, math
rows = []
for f in sorted(glob.glob('sim/target/research-dl/savant/sc_2025-*.csv')):
    for r in csv.DictReader(open(f)):
        try:
            rows.append(dict(b=r['batter'], v=float(r['bat_speed']), y=float(r['intercept_ball_minus_batter_pos_y_inches']),
                             aa=float(r['attack_angle']), tilt=float(r['swing_path_tilt'])))
        except (ValueError, KeyError):
            pass
by = collections.defaultdict(list)
for r in rows: by[r['b']].append(r)
comp = []
for b, rs in by.items():
    rs = sorted(rs, key=lambda r: r['v'])
    keep = rs[int(0.1 * len(rs)):]
    if len(keep) < 5: continue
    my = sum(r['y'] for r in keep) / len(keep)
    for r in keep: r['dy'] = r['y'] - my; comp.append(r)
bins = [(-99, -12), (-12, -6), (-6, -2), (-2, 2), (2, 6), (6, 12), (12, 99)]
print('n', len(comp))
ref = None
out = []
for lo, hi in bins:
    s = [r for r in comp if lo <= r['dy'] < hi]
    m = lambda k: sum(r[k] for r in s) / len(s)
    out.append((lo, hi, len(s), m('v'), m('aa'), m('tilt'), m('dy')))
ref = [o for o in out if o[0] == -2][0]
for lo, hi, n, v, aa, tilt, dy in out:
    print('%4d..%-4d n %5d  mean depth %+5.1f in  speed %.1f (%+.1f)  attack %.1f (%+.1f)  tilt %.1f (%+.1f)' % (lo, hi, n, dy, v, v - ref[3], aa, aa - ref[4], tilt, tilt - ref[5]))
