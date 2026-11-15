# What a sourced collision efficiency does, and where sim contact lands on the barrel (2026-11-15)

Branch `work/q-counterfactual`, from main `2223f4e`. A measuring job: **no sim behaviour changed and no default changed.** Follows [collision-efficiency-sources §4–§5](2026-11-15-collision-efficiency-sources.md) and [power-mix §4](2026-11-15-power-mix.md). Tools: example `sim/crates/bb_game/examples/q_contact.rs` (read-only; `ev_bat`'s columns plus the contact station, the off-centre angle and the contact normal's vertical component) and `analysis/statcast/q_counterfactual.py` (every table below; usage in its header). The CSV has the columns `ev_la_bins.py` and `power_mix.py` read, so those scripts also run on it (the HR decomposition in §1 is `ev_la_bins.py`'s).

**How the counterfactuals were run.** No override of `BatBallParams` exists. I patched `BatBallParams::default()` locally to read `e_ref` from an environment variable (`BB_E_REF`, one line in `bb_physics/src/bat.rs`), ran everything, and reverted it; it is **not committed** (`git diff main -- sim/crates/*/src` is empty). The baseline reproduces the power-mix digests exactly (seed 0 `ee562b47…`, seed 1000 `10c56db3…`; 15,869 + 15,988 fair balls), so the patch with the variable unset changes nothing. Values: `e_ref` 0.442 (q 0.200, main), 0.460 (q 0.215, the sourced best estimate), 0.478 (q 0.23, Statcast's own line; the swing-model §6 counterfactual). Everything else, including `s_e`, `v_ref`, `e_t`, μ and the bat, is as on main. 300 games at seed 0 and 300 at seed 1000; the exit-speed tables pool the two (31,857 / 32,218 / 32,922 fair balls). The real column is the same twelve Savant dates (9,165 fair balls; 8,761 with a tracked bat speed).

## Summary for the lead

1. **At q = 0.215 the 105+ mph and HR gaps close.** 105+ is 10.6% of fair balls (real 10.2, main 5.5), p90 exit speed 105.2 (real 105.1, main 102.9), 110+ 2.3% (real 1.9), barrels 7.8% (real 8.1, main 5.0), HR per fair ball 4.50% (real 4.54, main 2.61: **98% of the 1.93-point gap**), and the p90 squared-up ratio 0.955 (real 0.957). HR/PA goes 1.78% → 2.97–3.11%.
2. **What overshoots is the middle and the run environment.** Mean exit speed 91.5 (real 88.5; main 89.6, already 1.1 over), hard-hit (95+) 47.5% (real 40.7), squared-up share ≥ 0.9 41% (real 31%), ≥ 0.8 73% (real 62%), runs/team-game 5.32 / 5.57 (main 4.31 / 4.68), and the ground-ball hit rate 0.340 / 0.345 against a real BABIP of about 0.24 (main 0.322 / 0.326). The sourced peak value is not the problem; the sim already has the real *middle* of the distribution and the top now matches, so the whole distribution is too good.
3. **"More best contact" and "more poor contact" are two gaps, not one.** The real sample's slow swings (< 60 mph bat speed) are **3.7%** of its tracked fair balls (323 of 8,761), not the 7–8% the earlier notes said (§3); they explain about 2 of the 5 points by which real < 80 mph (24.9%) exceeds the sim's (19.6%), and the other 3 points are poor contact from *competitive* swings (real, competitive only: 23.1% < 80). Raising q makes the < 80 share *smaller* (17.5% at 0.460), so q cannot close it.
4. **Where sim contact lands** (baseline): the mean contact is **1.29 in toward the tip of the sweet spot** (SD 2.70 in); only 31% falls in the 2.4-in zone around the sweet spot; 7% is clamped at the tip end; and poor contact (ratio < 0.8) is **almost all outboard** (mean +3.3 in, 71% beyond 3 in toward the tip, 14% inboard of the sweet spot). The real distribution of contact points is not published, so this cannot be compared (§2).
5. The second mechanism is therefore a *spread* one: something that adds poor contact (below ratio 0.8) from competitive swings and trims the 0.8–0.95 middle, at 0.460, and does not touch the top. Contact spread along the barrel, with a heavier inboard side, is the lead candidate; slow swings and the real bat speed at contact are the others (§4).

## 1. Counterfactuals on the standard run

Run environment (`sim-cli games --n 300`; seed 0 / seed 1000). Real targets: MLB K% ≈ 22.5 and the BABIPs named by the CLI; runs/team-game for the real league is about 4.5 (not re-sourced here).

| | e_ref 0.442 (q 0.200, main) | 0.460 (q 0.215) | 0.478 (q 0.23) |
|---|---|---|---|
| runs/team-game | 4.31 / 4.68 | 5.32 / 5.57 | 6.57 / 6.24 |
| K% | 21.5 / 21.5 | 21.8 / 21.6 | 21.5 / 21.5 |
| BB% | 7.9 / 8.4 | 8.2 / 8.5 | 8.2 / 8.4 |
| HR/PA | 1.78% / 1.77% | 2.97% / 3.11% | 4.59% / 4.43% |
| GB hit rate (share) | 0.322 (49.5%) / 0.326 (47.5%) | 0.340 (50.9%) / 0.345 (48.7%) | 0.363 (50.8%) / 0.352 (50.1%) |
| LD hit rate | 0.761 / 0.769 | 0.768 / 0.777 | 0.783 / 0.780 |
| FB hit rate | 0.212 / 0.215 | 0.218 / 0.198 | 0.201 / 0.192 |
| PU hit rate | 0.036 / 0.028 | 0.038 / 0.026 | 0.035 / 0.026 |

(Real BABIP by type: GB 0.24, LD 0.68, FB 0.13, PU 0.02. Hit rate here counts home runs among hits in FB only through the CLI's ruling; the FB rate falls slightly because balls that become home runs leave the in-play pool.) K% and BB% do not move with q (they come before contact); the hit rates move up, and the ground-ball and line-drive rates were already above real at q = 0.200.

Exit speed, pooled seeds (`q_counterfactual.py` §1; the real row is the 12 Savant dates; \*barrel is an approximation of Savant's: EV ≥ 98 with launch angles 26–30° at 98 mph widening linearly to 8–50° at 116):

| group | n | <80 | 80-85 | 85-90 | 90-95 | 95-100 | 100-105 | 105-110 | 110+ | EV mean | EV p90 | 95+ | 100+ | 105+ | 110+ | barrel\* | HR/fair ball |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| real | 9165 | 24.9 | 9.1 | 11.0 | 14.2 | 15.6 | 14.9 | 8.3 | 1.9 | 88.5 | 105.1 | 40.7 | 25.2 | 10.2 | 1.9 | 8.1 | 4.54% |
| sim 0.442 | 31857 | 19.6 | 9.0 | 13.4 | 18.9 | 20.3 | 13.3 | 4.7 | 0.8 | 89.6 | 102.9 | 39.1 | 18.8 | 5.5 | 0.8 | 5.0 | 2.61% |
| sim 0.460 | 32218 | 17.5 | 7.6 | 11.3 | 16.1 | 19.8 | 17.1 | 8.3 | 2.3 | 91.5 | 105.2 | 47.5 | 27.6 | 10.6 | 2.3 | 7.8 | 4.50% |
| sim 0.478 | 32922 | 15.6 | 6.8 | 9.6 | 13.8 | 18.4 | 18.9 | 12.1 | 4.7 | 93.4 | 107.3 | 54.1 | 35.7 | 16.8 | 4.7 | 11.0 | 6.63% |

Squared-up ratio = exit speed ÷ (1.23 × bat + 0.23 × release speed), balls with a tracked bat speed:

| | n | mean | SD | share ≥ 0.8 | share ≥ 0.9 | p90 | EV mean when ≥ 0.9 |
|---|---|---|---|---|---|---|---|
| real | 8761 | 0.821 | 0.125 | 62% | 31% | 0.957 | 100.7 |
| sim 0.442 | 31857 | 0.830 | 0.107 | 70% | 31% | 0.934 | 99.6 |
| sim 0.460 | 32218 | 0.847 | 0.110 | 73% | 41% | 0.955 | 101.0 |
| sim 0.478 | 32922 | 0.865 | 0.113 | 76% | 49% | 0.975 | 102.3 |

Exit speed of balls squared up to ≥ 0.9, by bat speed (n: mean, p90):

| bat speed | real | sim 0.442 | sim 0.460 | sim 0.478 |
|---|---|---|---|---|
| 60–66 | 353: 94.6, 99.5 | 1060: 92.3, 95.3 | 1392: 93.5, 97.2 | 1738: 94.6, 99.0 |
| 66–70 | 644: 99.2, 103.7 | 2840: 96.6, 99.5 | 3685: 97.9, 101.4 | 4472: 99.1, 103.3 |
| 70–74 | 903: 102.8, 107.2 | 3428: 100.7, 103.6 | 4722: 102.0, 105.6 | 5680: 103.4, 107.6 |
| 74–78 | 588: 106.3, 110.1 | 1911: 104.9, 107.9 | 2610: 106.2, 109.7 | 3218: 107.6, 111.7 |
| 78+ | 135: 110.4, 114.6 | 583: 109.6, 113.0 | 798: 111.0, 114.8 | 934: 112.6, 117.1 |

(The real rows differ slightly from power-mix §4 because that note limited itself to batters with at least 20 tracked swings; here every tracked ball is used.)

HR decomposition by `ev_la_bins.py` at q = 0.215: deficit 0.04 points (main 1.93); at q = 0.23 the sim overshoots by 2.09 points (HR/ball 6.63%).

**Toward or away from each real target** (0.460 / 0.478, against main):

| target (real) | main | 0.460 | 0.478 |
|---|---|---|---|
| 105+ share 10.2% | 5.5, short | 10.6, **on it** | 16.8, over |
| 110+ 1.9% | 0.8 | 2.3, on it | 4.7, over |
| EV p90 105.1 | 102.9 | 105.2, **on it** | 107.3, over |
| barrel 8.1% | 5.0 | 7.8, on it | 11.0, over |
| HR per fair ball 4.54% | 2.61 | 4.50, **on it** | 6.63, over |
| 100+ 25.2% | 18.8 | 27.6, slightly over | 35.7, far over |
| squared-up p90 0.957 | 0.934 | 0.955, on it | 0.975, over |
| EV by bat band, ≥ 0.9 balls | 1–4 mph low | 0–2 low, p90 within 0.5–2.3 | at or above real, p90 over at 70+ |
| 90–100 share 29.8% | 39.2 | 35.9, **toward** (still 6 over) | 32.2, toward |
| EV mean 88.5 | 89.6, 1.1 over | **91.5, away** | 93.4, away |
| hard-hit 95+ 40.7% | 39.1, 1.6 under | 47.5, **away** (6.8 over) | 54.1, away |
| ≥ 0.9 share 31% | 31, on it | 41, **away** | 49, away |
| ≥ 0.8 share 62% | 70, 8 over | 73, **away** | 76, away |
| < 80 share 24.9% | 19.6, short | 17.5, **away** | 15.6, away |
| GB hit rate (BABIP 0.24) | 0.32, over | 0.34, away | 0.36, away |
| runs/team-game (about 4.5) | 4.3 / 4.7, on it | 5.3 / 5.6, over | 6.6 / 6.2, far over |

q = 0.215 fixes the top (everything from 100+ up, the squared-up p90, barrels, HR) and worsens the whole middle and the low end; q = 0.23 overshoots even the top, which agrees with the sources note's reading that 0.23 is Statcast's rough line and the sourced estimate is 0.215.

## 2. Where sim contact lands on the barrel (baseline, 31,857 fair balls)

`u` is the axial station of the ball centre on the bat axis at the executed contact instant (the contact point of the world's resolution), minus the sweet-spot station `u_ss` (sweet spot 0.15 m = 5.9 in from the tip), in inches, positive toward the tip. The station is clamped to the bat's length, so a ball centre beyond the end of the bat reads as the tip (5.9 in).

- Mean **+1.29 in**, median +1.24, **SD 2.70 in**.
- Within ±1.2 in of the sweet spot (a 2.4-in zone) 31.2% of balls; within ±2 in 49.5%; within ±3 in 67.1%.
- At the tip clamp (u − u_ss ≥ 5.9 in, the ball centre at or beyond the end of the bat): **7.2%** of fair balls (7.5% at q = 0.215); within 0.4 in of the end (≥ 5.5 in), 8.8%.

By squared-up ratio (share of the group in each 0.5-in bin, in %; positive toward the tip):

| bin (in) | all | ratio < 0.8 | 0.8–0.9 | ≥ 0.9 |
|---|---|---|---|---|
| < −5.0 | 0.9 | 3.0 | 0.0 | 0.0 |
| −5.0 to −4.0 | 1.5 | 3.6 | 0.9 | 0.0 |
| −4.0 to −3.0 | 3.2 | 2.1 | 6.5 | 0.0 |
| −3.0 to −2.0 | 5.7 | 1.6 | 8.8 | 5.8 |
| −2.0 to −1.0 | 9.1 | 1.6 | 7.4 | 18.6 |
| −1.0 to 0.0 | 12.5 | 2.3 | 7.8 | 28.2 |
| 0.0 to 1.0 | 13.6 | 3.1 | 10.7 | 27.6 |
| 1.0 to 2.0 | 14.2 | 4.7 | 18.0 | 18.9 |
| 2.0 to 3.0 | 11.9 | 7.1 | 24.6 | 0.9 |
| 3.0 to 4.0 | 9.1 | 12.1 | 13.9 | 0.0 |
| 4.0 to 5.0 | 6.9 | 21.2 | 1.3 | 0.0 |
| 5.0 to 6.0 | 11.4 | 37.6 | 0.0 | 0.0 |

(The 1-in rows are sums of the 0.5-in bins the script prints; the full 0.5-in table is `q_counterfactual.py` §4.)

| | all | ratio < 0.8 | 0.8–0.9 | ≥ 0.9 |
|---|---|---|---|---|
| n | 31,857 | 9,648 | 12,306 | 9,903 |
| mean u − u_ss (in) | +1.29 | +3.31 | +0.84 | **−0.12** |
| SD (in) | 2.70 | 3.19 | 2.21 | 1.14 |
| within ±1.2 in | 31.2% | 6.5% | 22.6% | 66.1% |
| off-centre angle, mean (p90), degrees | 18.0 (35.8) | 28.5 (49.7) | 17.3 (29.1) | 8.5 (16.4) |
| mean exit speed | 89.6 | 75.3 | 92.8 | 99.6 |

The off-centre angle is the angle between the contact normal (bat into ball) and the velocity of the bat point perpendicular to the axis: 0° is a head-on hit, larger means the ball met the barrel above or below the line of the swing. (Nine rows have no defined angle and are skipped.) The vertical component of the normal, `nz`, has mean +0.15 and SD 0.32 (0.18 in the ≥ 0.9 group, 0.42 in the < 0.8 group); it is in the CSV.

By station (1-in bins; mean exit speed, mean ratio): 97.8 mph and 0.906 at −1 to 0 in; 96.4 and 0.895 at 0 to 1; 94.4 and 0.874 at 1 to 2; 90.4 and 0.836 at 2 to 3; 85.9 and 0.793 at 3 to 4; 80.3 and 0.744 at 4 to 5; 68.8 and 0.637 at 5 to 6; and 94.2 and 0.869 at −3 to −2; 89.4 and 0.828 at −4 to −3; 82.5 and 0.766 at −5 to −4. Bat speed at the sweet spot is constant across stations (71.0 to 71.4 mph); the loss with station comes from the collision (f(u), the effective mass), not from the bat's speed at the sweet spot. The ratio peaks at about −0.5 in rather than 0; I did not trace why.

**Compared with the sources.** The real sweet zone (within 0.01 of the BBCOR peak) is 2.4 in wide, and so is the sim's (sources note §2); 31% of sim contact falls in that zone, and the squared-up ≥ 0.9 balls fall in it 66% of the time, in a band of about ±2.5 in. **The real distribution of contact points is not published** (Statcast does not report the contact station, and nothing I read measures it), so the sim's mean of +1.3 in toward the tip, its SD of 2.7 in and its 7% tip clamp cannot be called right or wrong; they can only be said to produce the sim's low-contact tail. They come from the swing model's aim noise on where the ball is met (I did not separate the sources), not from the collision.

## 3. The low end

**What the real sample has.** Of the 8,761 real fair balls with a tracked bat speed, **323 (3.69%) come from swings under 60 mph**, with mean exit speed 63.2, 74% of them under 80 mph and a p90 of 88.3. A further 404 fair balls (4.4% of the 9,165) have no tracked bat speed at all. Power-mix's "7–8% of balls from swings under 60 mph" divided by a count that included those untracked balls (its table's "share of balls at ≥ 60 mph bat" counts a missing speed as not competitive), so **the checked-and-defensive-swing share is 3.7% of tracked balls**, not 7–8%; if the untracked balls are the same kind of swing the share is up to 8%. I show both. The sim has **0.54%** of its balls from swings under 60 mph (0.59% at q 0.215).

**Real, competitive swings only (≥ 60 mph):** < 80 mph 23.1%, mean 89.5, 95+ 41.7%, 105+ 10.3% (the whole real sample: 24.9, 88.5, 40.7, 10.2). So the slow swings account for 1.8 points of the real < 80 share and 1.0 mph of its mean.

**Reweighting the sim.** A fraction w of the fair balls is replaced by a slow-swing ball, the rest are the sim's own. Assumption A: those balls have the real slow-swing exit-speed distribution. Assumption B: they have the sim's own contact-quality ratios on a real slow-swing bat speed (and the sim's pitch speeds). Both are exact mixtures, not sampled. w = 3.7% is the real share of tracked balls, 7% the earlier note's figure:

| sim variant | assumption | < 80 share | mean EV | 95+ | 105+ |
|---|---|---|---|---|---|
| 0.442 | as is | 19.6 | 89.6 | 39.1 | 5.5 |
| 0.442 | A, w 3.7% | 21.6 | 88.6 | 37.7 | 5.3 |
| 0.442 | B, w 3.7% | 21.5 | 88.7 | 37.6 | 5.3 |
| 0.442 | A, w 7% | 23.4 | 87.8 | 36.5 | 5.2 |
| 0.460 | as is | 17.5 | 91.5 | 47.5 | 10.6 |
| 0.460 | A, w 3.7% | 19.6 | 90.5 | 45.8 | 10.2 |
| 0.460 | B, w 3.7% | 19.3 | 90.6 | 45.7 | 10.2 |
| 0.460 | A, w 7% | 21.4 | 89.6 | 44.3 | 9.8 |
| 0.460 | B, w 7% | 20.9 | 89.8 | 44.1 | 9.9 |
| 0.478 | A, w 3.7% | 17.8 | 92.3 | 52.2 | 16.2 |
| 0.478 | A, w 7% | 19.7 | 91.3 | 50.5 | 15.6 |

(real: < 80 24.9, mean 88.5, 95+ 40.7, 105+ 10.2. The full table with w = 8% is in the script output.)

**One gap or two?** Two.
- **Slow swings** at the real 3.7% move the sim's < 80 share by +2.0 points and its mean exit speed by −1.0 mph, the same as they cost the real sample. They are a *real-sample* feature the sim lacks; they bring the baseline's < 80 share from 19.6 to 21.6 and leave 3.3 points to real's 24.9.
- **The rest is competitive-swing poor contact.** Real competitive swings have 23.1% under 80 mph; the sim's 19.6% (21.6 with slow swings added). That 3.5-point gap, and the real ratio ≥ 0.8 share of 62% against 70%, sit in swings that are full-effort in both samples. It is contact quality, not swing speed.
- **q goes the wrong way on both.** At q = 0.215 the < 80 share falls from 19.6 to 17.5 (stronger collision at every station lifts a ball over the bin edge) and the mean rises 1.9 mph. With 3.7% slow swings added the mean is 90.5 (real 88.5) and hard-hit 45.8% (real 40.7); even at w = 7% the mean (89.6) and hard-hit (44.3) stay over. So slow swings do not rescue q = 0.215; they cover about half of its mean-exit-speed overshoot and a fifth of its hard-hit overshoot at the real share.

## 4. Conclusion for the lead

- **How much closes.** With e_ref 0.460 the 105+ share goes from 5.5% to 10.6% (real 10.2%), p90 exit speed to 105.2 (real 105.1), barrels to 7.8%, HR per fair ball from 2.61% to 4.50% (real 4.54%), HR/PA from 1.8% to 3.0–3.1%: the whole 105+/HR gap, with no other change. The sourced value does what the sources note's §3 predicted (the sim's p90 ratio reaches the real 0.957).
- **What overshoots.** Everything below the top: mean exit speed 91.5 (real 88.5), hard-hit 47.5% (40.7), ratio ≥ 0.9 41% (31), ≥ 0.8 73% (62), ground-ball hit rate 0.34 (0.24), runs/team-game 5.3 / 5.6 where main's 4.3 / 4.7 was about right. A change to q alone moves the run environment by about one run per game and is not acceptable without a second change; the 0.478 counterfactual (6.2–6.6 runs, 4.4–4.6% HR/PA) is worse.
- **The second mechanism has to act on the middle and low end only.** The sim needs about 8–10 points of squared-up share to move from the 0.8–0.95 band to below 0.8 (real 62% ≥ 0.8; 0.460 gives 73%), while the top (p90 ratio 0.955, the 100+ tail) stays. The candidates, with what this note shows about each:
  1. **Contact spread along the barrel.** The ratio loses about 0.03 per inch from the peak on either side and the sim's inboard side is thin (its mean contact is 1.3 in toward the tip, only 14% of ratio < 0.8 balls are inboard). A heavier inboard tail and a wider spread would make more mid-quality balls poor, without changing the peak. The real contact-point distribution is unknown, so this can be argued from physics (the real sweet zone width, the real ratio SD of 0.125 against 0.107) but not measured. This is the best fit for "more poor contact from competitive swings".
  2. **Off-centre (vertical) miss.** The ratio ≥ 0.9 balls have an off-centre angle of 8.5° and the < 0.8 balls 28.5°; a wider vertical miss does the same job as a wider station spread. Not separable from candidate 1 with the data.
  3. **Slow swings** (checked and defensive swings, 3.7% of real balls): an addition, not a trade: −1.0 mph mean, +2.0 points under 80, no effect on the top. Worth adding for its own sake if the owner wants the real mix, but it is not the main gap at q = 0.215.
  4. **Bat speed at contact** (+0.8 mph mean and +1.2 mph at p90, power-mix §4): a *substitute* route to the 105+ tail, not an additional one. At q = 0.215 the tail already matches, so raising bat speed as well would overshoot it; if the owner prefers to keep q at 0.200 the same tail could be reached partly by bat speed, but the sources do not support that value of q's low end.
- **The decision for the owner** is thus q (0.215, sourced) *together with* a contact-spread change, not q alone, as the sources note's §5 already said; this note quantifies it: q alone gives the right top and a run environment about 1.1 runs too high.

**Caveats.** One sim seed set per `e_ref` (two seeds of 300 games); the HR/PA and run differences between 0.442 and 0.460 are far larger than the two seeds' spread (0.1–0.2 runs). The real column is twelve dates, with the sampling uncertainty power-mix §3 gives (the real 105+ share ± about 1.2 points). The barrel is an approximation of Savant's definition. The "off-centre angle" is my own measure: the angle between the contact normal and the perpendicular bat-point velocity. The slow-swing weights are assumptions (A and B, stated above) and not a sim result: no slow swings were simulated. The e_ref patch was local and is not in the repository; to repeat, set `e_ref` in `BatBallParams::default()` (`bat.rs`) and run `q_contact 0 300 out.csv` and `q_contact 1000 300 out.csv`, then `q_counterfactual.py`.

## Resolution

Not reviewed by Codex: one read-only example and one Python script; no sim source or default changed (`git diff main -- sim/crates/*/src` is empty). Open until the owner rules on the collision efficiency.
