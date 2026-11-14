"""agent-models §5.10 jump arithmetic from its formulas, with the v2-swing refit constants
(2026-11-14): the hand-worked values in bb_game live.rs tests. Standard library only."""
import math
A,B,R = 6.91, 0.939, 18.0          # spray = A + B*face, resid
LA,LB,LR = 184.10, -1213.3, 3.1     # face = LA + LB*lead (deg, deg/s), resid
MEAN_PULL, SCAT = 1.23, 20.5
PULL0 = 0.6                          # anticipated_pull_deg(0)
ZONE=(20.0,25.0)
ANG=[0,0,38,13,-36,-13,-30,0,30]
def fuse(p,sp,c,sc):
    k=sp*sp/(sp*sp+sc*sc); return p+k*(c-p), sp*sc/math.sqrt(sp*sp+sc*sc)
def jumps(face, face_sigma, right=True, prior_only=False):
    prior = -(MEAN_PULL+PULL0) if right else (MEAN_PULL+PULL0)
    if prior_only: est,s = prior,SCAT
    else:
        cue = A + B*face; sc = math.sqrt((B*face_sigma)**2 + R*R)
        if not right: cue=-cue
        est,s = fuse(prior,SCAT,cue,sc)
    out=[]
    for i,phi in enumerate(ANG):
        if i<2: out.append(0.0); continue
        off=est-phi; zone=ZONE[1] if i>=6 else ZONE[0]
        out.append(0.0 if abs(off)>zone else math.copysign(min(abs(off)/(2*s),1.0),off))
    return est,s,[round(x,4) for x in out]
face = 60.4412-90
print('face',face,'cue',A+B*face)
print('bat alone R', jumps(face,0.0))
print('bat alone L', jumps(face,0.0,right=False))
print('prior only', jumps(0,0,prior_only=True))
lead=(LA-face)/(-LB); print('lead giving that face %.6f'%lead, 'face check', LA+LB*round(lead,6))
L=round(lead,6); fl=LA+LB*L
print('lead alone R', jumps(fl,LR)); print('lead alone L', jumps(fl,LR,right=False))
# both: fuse faces (bat sigma 10, lead sigma 3.1)
ff,fs=fuse(face,10.0,fl,LR); print('fused face %.4f sigma %.4f'%(ff,fs)); print('both R', jumps(ff,fs))
print('two cues instead (bat as a 2nd spray cue):')
e,s,_=jumps(fl,LR); c2=A+B*face; sc2=math.sqrt((B*10)**2+R*R); print('  %.2f'%fuse(e,s,c2,sc2)[0])
print('seam (c): cue at face 0 =',A)
