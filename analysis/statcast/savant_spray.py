import csv, sys, glob, math, statistics as st
rows=[]
for f in sorted(glob.glob(sys.argv[1]+'/sc_*.csv')):
    with open(f, newline='', encoding='utf-8-sig') as fh:
        for r in csv.DictReader(fh):
            if not r['description'].startswith('hit_into_play'): continue
            if r['hc_x'] in ('','NA') or r['plate_x'] in ('','NA'): continue
            sp=math.degrees(math.atan2(float(r['hc_x'])-125.42, 198.27-float(r['hc_y'])))
            mir = 1 if r['stand']=='R' else -1
            rows.append((mir*sp, mir*float(r['plate_x'])*0.3048, r['bb_type'], r['attack_direction'], float(r['launch_speed']) if r['launch_speed'] not in ('','NA') else None))
v=[x[0] for x in rows]
print('MLB balls in play n %d spray (RHB-equivalent, + = RF/oppo) mean %.2f sd %.2f'%(len(v),st.mean(v),st.pstdev(v)))
print('plate_x bin (m, RHB frame)  n  mean spray  sd')
for k in range(7):
    lo=-0.35+0.1*k
    s=[x[0] for x in rows if lo<=x[1]<lo+0.1]
    if len(s)>20: print('%+.2f..%+.2f %5d %+6.1f %5.1f'%(lo,lo+0.1,len(s),st.mean(s),st.pstdev(s)))
for bt in ('ground_ball','line_drive','fly_ball','popup'):
    s=[x[0] for x in rows if x[2]==bt]; print(bt, len(s), '%+.1f sd %.1f'%(st.mean(s),st.pstdev(s)))
print('pull-positive spray of balls in play by Statcast ft bins (+ = inside):')
for lo,hi,l in ((-9,-0.55,'outside'),(-0.55,-0.18,'mid-away'),(-0.18,0.18,'middle'),(0.18,0.55,'mid-in'),(0.55,9,'inside')):
    s=[-x[0] for x in rows if lo<=-x[1]/0.3048<hi]
    print(l,len(s),'%+.1f'%st.mean(s))
