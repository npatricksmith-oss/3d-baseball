import csv, sys, glob, statistics as st
from collections import defaultdict
rows=[]
for f in sorted(glob.glob(sys.argv[1]+'/sc_*.csv')):
    with open(f, newline='', encoding='utf-8-sig') as fh:
        for r in csv.DictReader(fh):
            if r['bat_speed'] in ('','NA'): continue
            ev = float(r['launch_speed']) if r['launch_speed'] not in ('','NA') else None
            rows.append((r['batter'], float(r['bat_speed']), ev, r['description'], r['events'], float(r['launch_angle']) if r['launch_angle'] not in ('','NA') else None, r['balls'], r['strikes']))
byb=defaultdict(list)
for b,bs,ev,d,e,la,ba,stk in rows: byb[b].append(bs)
cut={b:sorted(v)[int(0.1*len(v))] for b,v in byb.items()}
comp=[r for r in rows if r[1]>=cut[r[0]] or (r[1]>=60 and r[2] and r[2]>=90)]
cb=defaultdict(list)
for r in comp: cb[r[0]].append(r[1])
big={b:v for b,v in cb.items() if len(v)>=25}
within=[st.pstdev(v) for v in big.values()]
means=[st.mean(v) for v in big.values()]
pooled=(sum(st.pvariance(v)*len(v) for v in big.values())/sum(len(v) for v in big.values()))**0.5
print('batters with >=25 competitive swings:',len(big),'swings',sum(len(v) for v in big.values()))
print('within-batter SD: pooled %.2f mph, median %.2f'%(pooled, st.median(within)))
print('between-batter SD of means %.2f (mean of means %.2f) (includes sampling error ~ pooled/sqrt(n))'%(st.pstdev(means), st.mean(means)))
n_avg=st.mean(len(v) for v in big.values()); print('mean n per batter %.1f; sampling var contribution %.2f'%(n_avg, pooled/n_avg**0.5))
print('total SD competitive %.2f'%st.pstdev([r[1] for r in comp]))
# home runs: bat speed of HRs
hr=[r[1] for r in rows if r[4]=='home_run']
print('HR n %d bat speed mean %.1f p10 %.1f p50 %.1f'%(len(hr), st.mean(hr), sorted(hr)[len(hr)//10], st.median(hr)))
# residual bat speed for HR relative to batter mean
res=[r[1]-st.mean(cb[r[0]]) for r in rows if r[4]=='home_run' and r[0] in big]
print('HR bat speed minus batter mean: mean %.2f'%st.mean(res))
ip=[r for r in comp if r[3].startswith('hit_into_play') and r[2]]
evs=sorted(r[2] for r in ip); print('in play n %d EV mean %.1f p90 %.1f p99 %.1f max %.1f'%(len(evs),st.mean(evs),evs[int(.9*len(evs))],evs[int(.99*len(evs))],evs[-1]))
# by strikes
for s in '012':
    v=[r[1] for r in comp if r[7]==s]; print('strikes',s,'bat %.2f n %d'%(st.mean(v),len(v)))
# within-batter SD among swings met within ±3 in of the batter's mean depth, in play or foul (made contact)
rows2=[]
for f in sorted(glob.glob(sys.argv[1]+'/sc_*.csv')):
    with open(f, newline='', encoding='utf-8-sig') as fh:
        for r in csv.DictReader(fh):
            if r['bat_speed'] in ('','NA') or r['intercept_ball_minus_batter_pos_y_inches'] in ('','NA'): continue
            rows2.append((r['batter'], float(r['bat_speed']), float(r['intercept_ball_minus_batter_pos_y_inches']), float(r['plate_x'])*(-1 if r['stand']=='R' else 1), r['description']))
cs=set((r[0],r[1]) for r in comp)
r2=[r for r in rows2 if (r[0],r[1]) in cs]
md=defaultdict(list)
for r in r2: md[r[0]].append(r[2])
mdm={b:st.mean(v) for b,v in md.items() if len(v)>=25}
sel=defaultdict(list)
for r in r2:
    if r[0] in mdm and abs(r[2]-mdm[r[0]])<3 and abs(r[3])<0.5: sel[r[0]].append(r[1])
sel={b:v for b,v in sel.items() if len(v)>=5}
pooled=(sum(st.pvariance(v)*len(v) for v in sel.values())/sum(len(v) for v in sel.values()))**0.5
print('within-batter SD, depth within ±3 in of own mean and |x|<0.5 ft: %.2f mph (%d swings, %d batters)'%(pooled, sum(len(v) for v in sel.values()), len(sel)))
