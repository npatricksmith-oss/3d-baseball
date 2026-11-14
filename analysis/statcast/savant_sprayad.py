import csv, sys, glob, math, statistics as st
from collections import defaultdict
g=defaultdict(list)
for f in sorted(glob.glob(sys.argv[1]+'/sc_*.csv')):
    with open(f, newline='', encoding='utf-8-sig') as fh:
        for r in csv.DictReader(fh):
            if not r['description'].startswith('hit_into_play'): continue
            if r['hc_x'] in ('','NA') or r['attack_direction'] in ('','NA') or r['bb_type']=='popup': continue
            sp=math.degrees(math.atan2(float(r['hc_x'])-125.42, 198.27-float(r['hc_y'])))
            mir = 1 if r['stand']=='R' else -1
            pull_spray=-mir*sp
            pull_ad=-float(r['attack_direction'])  # raw: negative = pull (inside pitches negative)
            g[(r['p_throws'],r['stand'])].append((pull_ad,pull_spray))
            g['all'].append((pull_ad,pull_spray))
for k,v in g.items():
    ad=[x[0] for x in v]; sp=[x[1] for x in v]; d=[x[1]-x[0] for x in v]
    n=len(v); mad,msp=st.mean(ad),st.mean(sp)
    cov=sum((a-mad)*(b-msp) for a,b in v)/n
    print(k, n, 'AD %+.1f spray %+.1f  spray-AD mean %+.1f sd %.1f  slope spray/AD %.2f r %.2f'%(mad,msp,st.mean(d),st.pstdev(d),cov/st.pvariance(ad),cov/(st.pstdev(ad)*st.pstdev(sp))))
