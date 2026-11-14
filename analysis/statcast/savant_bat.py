import csv, sys, glob, statistics as st
from collections import defaultdict
rows=[]
for f in sorted(glob.glob(sys.argv[1]+'/sc_*.csv')):
    with open(f, newline='', encoding='utf-8-sig') as fh:
        for r in csv.DictReader(fh):
            if r['bat_speed'] in ('', 'NA'): continue
            try:
                r['bs']=float(r['bat_speed']); r['sl']=float(r['swing_length'] or 'nan')
                r['aa']=float(r['attack_angle'] or 'nan'); r['ad']=float(r['attack_direction'] or 'nan')
                r['tilt']=float(r['swing_path_tilt'] or 'nan')
                r['ix']=float(r['intercept_ball_minus_batter_pos_x_inches'] or 'nan')
                r['iy']=float(r['intercept_ball_minus_batter_pos_y_inches'] or 'nan')
                r['px']=float(r['plate_x']); r['pz']=float(r['plate_z'])
                r['top']=float(r['sz_top']); r['bot']=float(r['sz_bot'])
                r['ev']=float(r['launch_speed']) if r['launch_speed'] not in ('','NA') else None
            except ValueError: continue
            rows.append(r)
# competitive: per batter fastest 90% + any >=60 with EV>=90
byb=defaultdict(list)
for r in rows: byb[r['batter']].append(r['bs'])
cut={b:sorted(v)[int(0.1*len(v))] for b,v in byb.items()}
comp=[r for r in rows if r['bs']>=cut[r['batter']] or (r['bs']>=60 and r['ev'] and r['ev']>=90)]
def q(v,p): v=sorted(v); return v[min(len(v)-1,int(p*(len(v)-1)))]
def summ(lbl, v):
    v=[x for x in v if x==x]
    if len(v)<20: print(f'{lbl:28s} n {len(v)}'); return
    print(f'{lbl:28s} n {len(v):6d} mean {st.mean(v):6.2f} sd {st.pstdev(v):5.2f} p10 {q(v,.1):6.1f} p50 {q(v,.5):6.1f} p90 {q(v,.9):6.1f}')
for name,S in (('all tracked',rows),('competitive',comp)):
    print('==',name,len(S))
    for k,l in (('bs','bat speed mph'),('sl','swing length ft'),('aa','attack angle'),('ad','attack direction'),('tilt','path tilt'),('iy','intercept y in'),('ix','intercept x in')):
        summ(l,[r[k] for r in S])
S=comp
def inside(r): return -r['px'] if r['stand']=='R' else r['px']
print('\n== competitive swings by horizontal location (ft, batter frame, + = inside; plate half-width 0.83)')
bins=[(-9,-1.1,'far outside'),(-1.1,-0.55,'outside'),(-0.55,-0.18,'mid-away'),(-0.18,0.18,'middle'),(0.18,0.55,'mid-in'),(0.55,1.1,'inside'),(1.1,9,'far inside')]
for lo,hi,l in bins:
    sel=[r for r in S if lo<=inside(r)<hi]
    if not sel: continue
    print(f'{l:12s} n {len(sel):5d} bat {st.mean(r["bs"] for r in sel):5.1f} p90 {q([r["bs"] for r in sel],.9):5.1f} | len {st.mean(r["sl"] for r in sel if r["sl"]==r["sl"]):4.2f} | tilt {st.mean(r["tilt"] for r in sel if r["tilt"]==r["tilt"]):4.1f} | AA {st.mean(r["aa"] for r in sel if r["aa"]==r["aa"]):5.1f} | AD {st.mean(r["ad"] for r in sel if r["ad"]==r["ad"]):5.1f} | iy {st.mean(r["iy"] for r in sel if r["iy"]==r["iy"]):5.1f} | ix {st.mean(r["ix"] for r in sel if r["ix"]==r["ix"]):5.1f}')
print('\n== by height (normalized: 0 = bottom of zone, 1 = top)')
for lo,hi,l in ((-9,0,'below'),(0,1/3,'bottom'),(1/3,2/3,'middle'),(2/3,1,'top'),(1,9,'above')):
    sel=[r for r in S if lo<=(r['pz']-r['bot'])/(r['top']-r['bot'])<hi]
    print(f'{l:8s} n {len(sel):5d} bat {st.mean(r["bs"] for r in sel):5.1f} | len {st.mean(r["sl"] for r in sel if r["sl"]==r["sl"]):4.2f} | tilt {st.mean(r["tilt"] for r in sel if r["tilt"]==r["tilt"]):4.1f} | AA {st.mean(r["aa"] for r in sel if r["aa"]==r["aa"]):5.1f}')
# intercept depth relative to batter mean
mb=defaultdict(list)
for r in S:
    if r['iy']==r['iy']: mb[(r['batter'],r['stand'])].append(r['iy'])
mbm={k:st.mean(v) for k,v in mb.items() if len(v)>=20}
print('\n== by intercept depth relative to batter mean (in, + = out front)')
for lo,hi in ((-99,-12),(-12,-6),(-6,-2),(-2,2),(2,6),(6,12),(12,99)):
    sel=[r for r in S if (r['batter'],r['stand']) in mbm and r['iy']==r['iy'] and lo<=r['iy']-mbm[(r['batter'],r['stand'])]<hi]
    if len(sel)<20: continue
    print(f'[{lo:4d},{hi:4d}) n {len(sel):5d} bat {st.mean(r["bs"] for r in sel):5.1f} | len {st.mean(r["sl"] for r in sel if r["sl"]==r["sl"]):4.2f} | AD {st.mean(r["ad"] for r in sel if r["ad"]==r["ad"]):5.1f} | AA {st.mean(r["aa"] for r in sel if r["aa"]==r["aa"]):5.1f}')
print('\n== by lateral intercept from batter (ix, in): a direct test of speed against radius')
for lo,hi in ((0,20),(20,25),(25,30),(30,35),(35,40),(40,45),(45,99)):
    sel=[r for r in S if r['ix']==r['ix'] and lo<=abs(r['ix'])<hi]
    if len(sel)<20: continue
    print(f'|ix| [{lo},{hi}) n {len(sel):5d} bat {st.mean(r["bs"] for r in sel):5.1f} p90 {q([r["bs"] for r in sel],.9):5.1f} | len {st.mean(r["sl"] for r in sel if r["sl"]==r["sl"]):4.2f}')
# regression bat speed on inside coordinate and depth
import math
xs=[(inside(r), r['iy']-mbm[(r['batter'],r['stand'])], r['bs']) for r in S if (r['batter'],r['stand']) in mbm and r['iy']==r['iy']]
def slope(a,b):
    ma,mb_=st.mean(a),st.mean(b); c=sum((x-ma)*(y-mb_) for x,y in zip(a,b)); v=sum((x-ma)**2 for x in a); return c/v
print('\nslope bat speed per ft inside: %.2f mph; per inch out front: %.3f mph'%(slope([x[0] for x in xs],[x[2] for x in xs]), slope([x[1] for x in xs],[x[2] for x in xs])))
print('\n== by outcome')
def oc(r):
    d=r['description']
    if 'swinging_strike' in d or d=='missed_bunt': return 'whiff'
    if 'foul' in d: return 'foul'
    if d.startswith('hit_into_play'): return 'in play'
    return d
g=defaultdict(list)
for r in S: g[oc(r)].append(r)
for k,v in g.items():
    if len(v)>50: print(f'{k:10s} n {len(v):5d} bat {st.mean(r["bs"] for r in v):5.1f} len {st.mean(r["sl"] for r in v if r["sl"]==r["sl"]):4.2f} AD sd {st.pstdev([r["ad"] for r in v if r["ad"]==r["ad"]]):4.1f}')
ip=[r['ev'] for r in S if oc(r)=='in play' and r['ev']]
print('EV in play: mean %.1f p99 %.1f max %.1f'%(st.mean(ip),q(ip,.99),max(ip)))
