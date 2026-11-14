import csv, sys, glob, statistics as st
rows=[]
for f in sorted(glob.glob(sys.argv[1]+'/sc_*.csv')):
    with open(f, newline='', encoding='utf-8-sig') as fh:
        for r in csv.DictReader(fh):
            if not r['description'].startswith('hit_into_play') or r['launch_speed'] in ('','NA'): continue
            xin = (-1 if r['stand']=='R' else 1)*float(r['plate_x'])
            rows.append((xin, float(r['launch_speed']), r['events'], float(r['launch_angle']) if r['launch_angle'] not in ('','NA') else None))
print('MLB balls in play by location (ft, + inside): EV mean, HR per BIP')
for lo,hi,l in ((-9,-0.55,'outside'),(-0.55,-0.18,'mid-away'),(-0.18,0.18,'middle'),(0.18,0.55,'mid-in'),(0.55,9,'inside')):
    s=[x for x in rows if lo<=x[0]<hi]
    print('%-9s n %4d EV %.1f HR/BIP %.1f%%'%(l,len(s),st.mean(x[1] for x in s),100*sum(x[2]=='home_run' for x in s)/len(s)))
la=[x[3] for x in rows if x[3] is not None]
print('LA mean %.1f sd %.1f; HR/BIP %.2f%%; EV>=95 %.1f%%  >=100 %.1f%% >=105 %.1f%% >=110 %.1f%%'%(st.mean(la),st.pstdev(la),100*sum(x[2]=='home_run' for x in rows)/len(rows),*[100*sum(x[1]>=t for x in rows)/len(rows) for t in (95,100,105,110)]))
ev=sorted(x[1] for x in rows); q=lambda p: ev[int(p*(len(ev)-1))]
print('EV q10 %.1f q25 %.1f q50 %.1f q75 %.1f q90 %.1f q99 %.1f'%(q(.1),q(.25),q(.5),q(.75),q(.9),q(.99)))
bar=0
for x in rows:
    e,a=x[1],x[3]
    if a is None or e<98: continue
    # approximate Statcast barrel window: 26–30° at 98 mph, widening with exit speed
    if 26-max(0,(e-98))*1.0 <= a <= 30+max(0,(e-98))*1.25: bar+=1
print('approx barrel%% of BIP %.1f'%(100*bar/len(rows)))
