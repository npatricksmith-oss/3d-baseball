"""Exit speed x launch angle table: sim (CSV from the ev_la_bins example) against Savant, with the HR-deficit
decomposition and a direction check. Stdlib only.
python3 -I analysis/statcast/ev_la_bins.py sim/target/data/savant sim/target/data/sim_bip_0.csv sim/target/data/sim_bip_1000.csv
"""
import csv, glob, math, sys
EVE = [0, 80, 85, 90, 95, 100, 105, 110, 999]; LAE = [-999, 0, 10, 20, 25, 30, 35, 40, 50, 999]
def b(x, e):
    for i in range(len(e) - 1):
        if e[i] <= x < e[i + 1]: return i
def evl(i): return ['<80','80-85','85-90','90-95','95-100','100-105','105-110','110+'][i]
def lal(i): return ['<0','0-10','10-20','20-25','25-30','30-35','35-40','40-50','50+'][i]
def real(d):
    out = []
    for f in sorted(glob.glob(d + '/sc_*.csv')):
        with open(f, newline='', encoding='utf-8-sig') as fh:
            for r in csv.DictReader(fh):
                if not r['description'].startswith('hit_into_play'): continue
                if r['launch_speed'] in ('', 'NA') or r['launch_angle'] in ('', 'NA'): continue
                sp = None
                if r['hc_x'] not in ('', 'NA') and r['hc_y'] not in ('', 'NA'):
                    a = math.degrees(math.atan2(float(r['hc_x']) - 125.42, 198.27 - float(r['hc_y'])))
                    sp = -a if r['stand'] == 'R' else a   # pull-positive
                out.append((float(r['launch_speed']), float(r['launch_angle']), sp, int(r['events'] == 'home_run')))
    return out
def sim(paths):
    out = []
    for p in paths:
        with open(p) as fh:
            for r in csv.DictReader(fh): out.append((float(r['ev_mph']), float(r['la_deg']), float(r['spray_pull_deg']), int(r['hr'])))
    return out
def grid(rows):
    n = [[0]*9 for _ in range(8)]; h = [[0]*9 for _ in range(8)]
    for ev, la, sp, hr in rows:
        i, j = b(ev, EVE), b(la, LAE); n[i][j] += 1; h[i][j] += hr
    return n, h
def table(name, rows):
    n, h = grid(rows); N = len(rows)
    print(f'\n#### {name}: {N} fair balls, {sum(map(sum,h))} HR ({100*sum(map(sum,h))/N:.2f}% of balls)\n')
    print('Share of fair balls (%), rows exit speed mph, columns launch angle deg\n')
    print('| EV \\ LA | ' + ' | '.join(lal(j) for j in range(9)) + ' | all |'); print('|---' * 11 + '|')
    for i in range(8): print(f'| {evl(i)} | ' + ' | '.join(f'{100*n[i][j]/N:.2f}' for j in range(9)) + f' | {100*sum(n[i])/N:.1f} |')
    print('| all | ' + ' | '.join(f'{100*sum(n[i][j] for i in range(8))/N:.1f}' for j in range(9)) + ' | 100 |')
    print('\nHR per ball (%); `-` = fewer than 10 balls, `*` = 10-29 balls (noisy)\n')
    print('| EV \\ LA | ' + ' | '.join(lal(j) for j in range(9)) + ' | all |'); print('|---' * 11 + '|')
    def c(a, k): return '-' if k < 10 else f'{100*a/k:.1f}' + ('*' if k < 30 else '')
    for i in range(8): print(f'| {evl(i)} | ' + ' | '.join(c(h[i][j], n[i][j]) for j in range(9)) + f' | {c(sum(h[i]), sum(n[i]))} |')
    print('| all | ' + ' | '.join(c(sum(h[i][j] for i in range(8)), sum(n[i][j] for i in range(8))) for j in range(9)) + f' | {c(sum(map(sum,h)), N)} |')
    return n, h
def marg(rows):
    N = len(rows); g = lambda f: 100 * sum(1 for r in rows if f(r)) / N
    print(f'hard-hit (>=95) {g(lambda r: r[0] >= 95):.1f}%  >=100 {g(lambda r: r[0] >= 100):.1f}%  >=105 {g(lambda r: r[0] >= 105):.1f}%  >=110 {g(lambda r: r[0] >= 110):.1f}%; EV mean {sum(r[0] for r in rows)/N:.1f}; LA mean {sum(r[1] for r in rows)/N:.1f} sd {(sum((r[1]-sum(x[1] for x in rows)/N)**2 for r in rows)/N)**.5:.1f}; 25-35 deg {g(lambda r: 25 <= r[1] < 35):.1f}%')
def rates(n, h, N):
    s = [[n[i][j]/N for j in range(9)] for i in range(8)]
    return s
def decomp(sn, sh, rn, rh):
    Ns, Nr = sum(map(sum, sn)), sum(map(sum, rn))
    s = [[sn[i][j]/Ns for j in range(9)] for i in range(8)]; q = [[rn[i][j]/Nr for j in range(9)] for i in range(8)]
    # per-cell HR rate; where the sim cell is empty its rate is 0 (nothing to apply); where the real cell is empty use 0 weight anyway.
    r = [[sh[i][j]/sn[i][j] if sn[i][j] else 0.0 for j in range(9)] for i in range(8)]
    hh = [[rh[i][j]/rn[i][j] if rn[i][j] else 0.0 for j in range(9)] for i in range(8)]
    T = lambda w, x: sum(w[i][j]*x[i][j] for i in range(8) for j in range(9))
    S, R = T(s, r), T(q, hh)
    print(f'\nsim HR/ball {100*S:.3f}%, real {100*R:.3f}%, deficit {100*(R-S):.3f} points of balls ({100*(R-S)/R:.0f}% of the real rate)')
    A1 = T(q, r) - S; B1 = R - T(q, r)       # population first, then rate
    A2 = R - T(s, hh); B2 = T(s, hh) - S      # rate first, then population (reverse)
    print(f'order 1 (real shares x sim rates): population {100*A1:.3f} ({100*A1/(R-S):.0f}%), rate-within-bin {100*B1:.3f} ({100*B1/(R-S):.0f}%)')
    print(f'order 2 (sim shares x real rates): population {100*A2:.3f} ({100*A2/(R-S):.0f}%), rate-within-bin {100*B2:.3f} ({100*B2/(R-S):.0f}%)')
    # within the population term: EV marginal first (keep sim LA|EV), then LA|EV remainder
    sEV = [sum(s[i]) for i in range(8)]; qEV = [sum(q[i]) for i in range(8)]
    m = [[(s[i][j]/sEV[i] if sEV[i] else 0)*qEV[i] for j in range(9)] for i in range(8)]   # real EV marginal, sim LA | EV
    ev_part = T(m, r) - S; la_part = T(q, r) - T(m, r)
    sLA = [sum(s[i][j] for i in range(8)) for j in range(9)]; qLA = [sum(q[i][j] for i in range(8)) for j in range(9)]
    m2 = [[(s[i][j]/sLA[j] if sLA[j] else 0)*qLA[j] for j in range(9)] for i in range(8)]  # real LA marginal, sim EV | LA
    la_first = T(m2, r) - S; ev_after = T(q, r) - T(m2, r)
    print(f'population split, EV marginal first: EV {100*ev_part:.3f}, LA given EV {100*la_part:.3f}')
    print(f'population split, LA marginal first: LA {100*la_first:.3f}, EV given LA {100*ev_after:.3f}')
    print(f'population split, average of the two orders (Shapley): EV {100*(ev_part+ev_after)/2:.3f}, LA {100*(la_part+la_first)/2:.3f}')
    print('\nPer-bin contribution to the population term (real share - sim share) x sim rate, points of balls; top cells:')
    cells = sorted(((q[i][j]-s[i][j])*r[i][j]*100, i, j, q[i][j]*100, s[i][j]*100, r[i][j]*100) for i in range(8) for j in range(9))
    for v, i, j, qq, ss, rr in cells[:8]: print(f'  EV {evl(i)}, LA {lal(j)}: {v:+.3f} (real share {qq:.2f}%, sim {ss:.2f}%, sim rate {rr:.1f}%)')
    print('Per-bin contribution to the rate term, real share x (real rate - sim rate); top cells:')
    cells = sorted(((q[i][j]*(hh[i][j]-r[i][j]))*100, i, j, rn[i][j], hh[i][j]*100, r[i][j]*100) for i in range(8) for j in range(9))
    for v, i, j, k, a, c in cells[:8]: print(f'  EV {evl(i)}, LA {lal(j)}: {v:+.3f} (real n {k}, real rate {a:.1f}%, sim rate {c:.1f}%)')
def direction(name, rows):
    sel = [r for r in rows if r[0] >= 95 and 20 <= r[1] < 40 and r[2] is not None]
    hrs = [r for r in sel if r[3]]
    def sh(v):
        n = len(v)
        return f'n {n}: pull {100*sum(1 for x in v if x[2] > 15)/n:.0f}%, centre {100*sum(1 for x in v if -15 <= x[2] <= 15)/n:.0f}%, oppo {100*sum(1 for x in v if x[2] < -15)/n:.0f}%; mean {sum(x[2] for x in v)/n:+.1f} sd {(sum((x[2]-sum(y[2] for y in v)/n)**2 for x in v)/n)**.5:.1f}'
    print(f'- {name}, EV>=95 & LA 20-40 balls: {sh(sel)}')
    print(f'- {name}, those that were HR: {sh(hrs)}; HR per ball by third: ' + ', '.join(f'{k} {100*sum(x[3] for x in sel if f(x[2]))/max(1,sum(1 for x in sel if f(x[2]))):.1f}%' for k, f in (('pull', lambda s: s > 15), ('centre', lambda s: -15 <= s <= 15), ('oppo', lambda s: s < -15))))
def extras(name, rows):
    hard = [r for r in rows if r[0] >= 100]; n = len(hard)
    print(f'- {name}: EV>=100 n {n} ({100*n/len(rows):.1f}% of balls); of those LA<10 {100*sum(r[1]<10 for r in hard)/n:.0f}%, 10-20 {100*sum(10<=r[1]<20 for r in hard)/n:.0f}%, 20-40 {100*sum(20<=r[1]<40 for r in hard)/n:.0f}%, 40+ {100*sum(r[1]>=40 for r in hard)/n:.0f}%; HR per EV>=100 ball {100*sum(r[3] for r in hard)/n:.1f}%; mean spray by LA band (pull+): ' + ', '.join(f'{lal(j)} {sum(r[2] for r in rows if b(r[1],LAE)==j and r[2] is not None)/max(1,sum(1 for r in rows if b(r[1],LAE)==j and r[2] is not None)):+.1f}' for j in range(9)))
if __name__ == '__main__':
    R = real(sys.argv[1]); S0 = sim([sys.argv[2]]); S1 = sim([sys.argv[3]]); S = S0 + S1
    for nm, rows in (('Savant 2025 (12 dates)', R), ('sim seed 0', S0), ('sim seed 1000', S1), ('sim pooled', S)): print(nm); marg(rows)
    tabs = {nm: table(nm, rows) for nm, rows in (('Sim, seeds 0 and 1000 pooled', S), ('Sim seed 0', S0), ('Sim seed 1000', S1), ('Savant 2025, twelve dates', R))}
    print('\n### Decomposition (sim pooled vs Savant)'); decomp(*tabs['Sim, seeds 0 and 1000 pooled'], *tabs['Savant 2025, twelve dates'])
    print('\n### Decomposition (sim seed 0 vs Savant)'); decomp(*tabs['Sim seed 0'], *tabs['Savant 2025, twelve dates'])
    print('\n### Decomposition (sim seed 1000 vs Savant)'); decomp(*tabs['Sim seed 1000'], *tabs['Savant 2025, twelve dates'])
    print('\n### Direction'); direction('Savant', R); direction('sim (pooled)', [(a, b_, c, d) for a, b_, c, d in S])

    print('\n### Extras'); extras('Savant', R); extras('sim pooled', S)
