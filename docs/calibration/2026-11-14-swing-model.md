# The swing model, v2: hands on an arc and a lagging bat (2026-11-14)

> Superseded figures: current numbers are in the [metrics ledger](../metrics-ledger.md) (§1).

Branch `work/swing-model`, cut from main `bcbb91f`. Not merged: the owner rules on a change this size. Spec: [agent-models §3.6](../agent-models.md) (v2) and [physics-spec §6.3](../physics-spec.md). This note gives the claims checked, what was built, the measurements on main and on the branch, and the open questions.

**In one paragraph.** The Gemini diagnosis's first claim holds: v1's swing turned at a fixed angular speed, so its sweet spot ran 64 mph on inside pitches and 84 on outside ones, where Statcast has 71–72 at both, and the outer third was a home-run launchpad. v2 replaces it with a kinematic double pendulum laid out from the contact instant, at the batter's `BatSpeed` whatever the reach. It matches Statcast's swing length, path tilt, vertical bat angle, attack-angle slope and bat speed by location. With that, **home runs collapse from 5.1% of PAs to 0.56%** (MLB 3.0%) and **runs from 5.30 to 3.74** [5.51 → 3.86]. The missing home runs are the exit-speed tail; their causes are outside the swing (below). The second claim, that the swing shape makes the foul share low, is **not borne out**: the foul share moves 21.9 → 24.2% in games (MLB 52.5%), and K% and pitches per PA do not move. The early fouls MLB has come from timing, which v2 does not change.

## 1. The claims, checked on main `bcbb91f`

Sources for MLB: a Baseball Savant pitch-level sample I pulled for this note (regular season 2025, six dates: Apr 22, May 20, Jun 17, Jul 22, Aug 19, Sep 16; 26,489 pitches, 12,180 tracked swings, 11,238 competitive by Savant's definition: a batter's fastest 90% plus any 60+ mph swing with a 90+ mph exit speed; same query template as [R13](../research/r13-foul-direction.md)). Analysis scripts: `analysis/statcast/` (the CSVs are not committed; §8 has the query). R13's 2025 sample (12 dates, these six among them) gives the same means to within 0.3 (R13 against the six-date sample: bat speed, all tracked swings, 69.9 against 69.7; swing length 7.2 against 7.2; attack angle 9.0 against 9.0; tilt 32 against 32.2). **Which sample is which:** §3's MLB figures for the whole-population measures (bat speed all tracked, swing length, tilt overall and its p10/p90, tilt by third "R13", attack angle, attack direction, the AA–AD and AD–depth slopes, fair-contact AD SD) are R13's twelve-date numbers, copied from its §F6; the figures marked "competitive", the "this sample" tilt by third, and every by-location and by-depth figure (§1, §3) are from the six-date sample. The exit-speed and launch-angle figures in §5 come from the same six-date CSVs by the scripts' inputs (the CSVs are not in the repo, so this and the six-date figures themselves cannot be rechecked from the repo; the independent check of this note found R13's numbers match the whole-population ones exactly).

**Claim 1, bat speed grows with reach: confirmed.**

| | sim v1 (platoon_diag, R/R, 2,000 PA) | MLB (competitive swings) |
|---|---|---|
| Sweet-spot speed, inside / middle / outside third, mph | 64.0 / 72.5 / 83.5 | 71.4 / 72.1 / 70.1 (inside / middle / outside bins; far outside 68.3) |
| Exit speed by horizontal third, mph | 84.0 / 90.3 / 99.0 | 79.3 inside, 89.6 mid-in, 93.1 middle, 90.9 mid-away, 83.9 outside |
| HR per ball in play, outside third | 14.2% | 1.8% (outside bin) |
| Exit speed in play, p90 / p99 / max, mph (run_env_diag, 100 games) | 112.1 / 125.9 / 136.2 | 105.2 / 111.0 / 116.7 |
| HR / PA (standard run, seeds 0–299) | 5.11% | 3.0% (2024; 5,453 HR in 182,449 PA = 2.99%, Savant league totals, [sources](2026-11-14-sources.md)) |

Statcast's bat speed is flat across the plate within ±2 mph, rising only with contact depth out front (68.3 mph 12+ in deep to 73.4 at 6–12 in out front). Its lateral intercept moves 17 in further from the batter from inside to outside with no rise in speed: the batter reaches, he does not swing faster. Within a batter, at fixed depth (±3 in of his own mean) over the middle of the plate, bat speed still varies by 3.2 mph SD (4.5%); between batters the SD of means is 2.8 mph.

**Claim 2, fouls and short at-bats: the numbers are confirmed, the causal link to the swing's shape is not.**
- Foul share 21.9% of contact in games (`foul_in_games`, 128 games; MLB 52.5% with caught fouls), 23.7% in the PA model. Pitches per PA 2.92, K% 15.9 [16.0] (standard run).
- The swing measures of the foul note reproduce exactly (`foul_diag`, 3,000 PAs): swing length 9.5 ft (MLB 7.2), swing path tilt 23.7° (32), fair-contact AD SD 6.9 / 6.8° (14 / 19), vertical bat angle −18.5° (−25 to −35).
- As the lead's note and the foul note's §6–§8 say, the early whiffs come mostly from the batter's final aim (misread at commit, a correction budget of about 8 cm), not from the barrel's path. v2 tests the shape directly: it fixes length, tilt and bat angle, and the foul share barely moves (§4).

## 2. What was built

Spec first ([agent-models §3.6](../agent-models.md), commit `bade326`), then an independent oracle (`analysis/oracle/swing_oracle.py`: numerical quadrature of the rate bumps, Rodrigues rotations, finite-difference velocities; fixture `golden/swing.json`; Rust agrees to 1e-9 m), then the code (`sim/crates/bb_agents/src/swing.rs`).

- **Kinematics.** The hands move on an arc of radius ρ about a hand-arc center C in one swing plane; the bat turns about the knob, lagging the hands early and releasing by the nominal contact instant (lag 0 at t_c [E]). Hands' and bat's angular rates are Gaussian bumps, the hands' peaking 32 ms before contact, the bat's 10 ms after (proximal to distal). Closed-form angles (erf), analytic velocities.
- **Speed set per swing.** The rate scale is `k = BatSpeed / (q·ρ + u_ss)`, so every planned swing meets its aim at `BatSpeed`. A long reach turns more slowly.
- **Contact-relative plan.** At the aim: tilt β (Statcast path tilt by zone height and location), attack angle α (Statcast by height), attack direction ψ (Statcast by location), contact depth slope 0.125 (Statcast intercept depth by location; v1's 0.6 was authored for its radial bat). The bat axis is ⊥ to the path in the plane; the vertical bat angle follows from α and β (sin² = sin²β − sin²α). C keeps its lateral position (−1.10 m for a right-hander [E]); ρ is solved from it and clamped to 0.15–0.46 × height (a clamped swing comes up short along the bat).
- **Timing constants** fitted to Statcast's swing length (7.2–7.3 ft) and Blast's 0.149 s time to contact (vendor): hands' width 55 ms, bat's width 55 ms, bat's peak +10 ms, hands' rate 0.32 of the bat's at contact [E].
- **Execution noise.** Unchanged in kind: δ_v, plane tilt, reach, timing from `Purpose::SwingExecution`. New: **effort**, the swing's speed relative to `BatSpeed`, σ 4.5% [E from the Statcast within-batter SD above], drawn from a new appended `Purpose::SwingEffort` (14) so the existing draws are untouched. Without it every grade-50 swing would be exactly 71.6 mph (32.0 m/s is 71.58 mph) and exit speed would top out at 103.8.
- **Decision code** is unchanged: the batter still plans from his own noisy read; the planner now takes the template's attack angle at the aim (`SwingTemplate::attack_angle_at`).
- **Fielders' beliefs refit, mechanically**, with the same tools (`live.rs`, agent-models §5.10): spray from face 5.2 + 1.36·face (18.6°) → 6.91 + 0.939·face (18.0°); face from lead 161.93 − 1,063.1·L (3.6°) → 184.10 − 1,213.3·L (3.1°); typical pull 5.0° → 1.23°; scatter 22° → 20.5°; pull by location (v1 −6.5° to +7.7°) → −2.8° to +0.6°; `BAT_YAW_RATE_DPS` 1,915 → 2,312°/s (so the grade-50 bat-read σ is 92° instead of 77°). Two of the fitting tools, `spray_cues` and `spray_by_location`, still assumed every batter was right-handed (true before A1); they now mirror left-handers. The hand-worked jump tests in `live.rs` are recomputed by `analysis/oracle/anticipation_jumps.py` from §5.10's formulas.
- `foul_diag` gains a by-location table and bat speed by depth, with the MLB columns from the sample; `examples/swing_cage_probe.rs` prints the on-time cage at five locations.

Tests: `cargo test --workspace --release` 294 passed, 0 failed, 2 ignored; Python oracle tests pass. `home_run_trot` now runs games until it has 8 home runs (it took 10 games before; v2 hits far fewer).

## 3. The swing against Statcast (`foul_diag`, 3,000 PAs, average RHP against average RHB)

| Measure | v1 (main) | v2 (branch) | MLB |
|---|---|---|---|
| Bat speed at the intercept, mph, mean / p10 / p50 / p90 | 72.8 / 61.5 / 71.7 / 86.7 | 70.7 / 66.1 / 70.9 / 75.1 | 69.9 / 62.3 / 71.2 / 77.5 (all tracked); competitive 71.4 / 64.9 / 71.8 / 77.7 |
| Swing length, ft, mean / p10 / p90 | 9.5 / 7.9 / 11.2 | 7.4 / 6.5 / 8.6 | 7.2 / 6.0 / 8.4 |
| Swing path tilt, °, mean / p10 / p90 | 23.7 / 14.3 / 33.4 | 32.1 / 27.8 / 36.5 | 32 / 23 / 41 |
| Tilt by zone third, bottom / middle / top | 28.2 / 22.9 / 16.7 | 34.1 / 31.8 / 29.2 | 37 / 32 / 25 (R13); 34.8 / 32.2 / 28.2 (this sample) |
| Vertical bat angle, °, mean / p10 / p90 | −18.5 / −30.5 / −6.6 | −28.0 / −33.8 / −21.1 | −25 to −35 (Blast, vendor) |
| Attack angle, °, mean / p10 / p90 | 12.1 / 6.3 / 18.9 | 11.5 / 3.2 / 21.6 | 9.0 / −5 / 23 |
| AA against AD, ° per ° toward pull | +0.26 (r 0.83) | +0.44 (r 0.87) | +0.41 (r 0.71) |
| AD against depth, ° per inch out front | +1.04 (r 0.94) | +0.98 (r 0.95) | +1.48 (r 0.87, the mean of R13's 0.89 right-handed and 0.85 left-handed; R13 states −1.48 ° per inch *deeper*) |
| Attack direction, °, p10 / p50 / p90 (pull +) | −8.7 / 4.7 / 31.1 | −7.0 / 4.1 / 28.1 | −25 / ≈0 / 23 |
| Fair-contact AD SD, pulled / opposite | 6.9 / 6.8 | 6.2 / 6.9 | 14 / 19 |

The tilt is set from the Statcast sample, so its match is by construction; its spread is not (p10–p90 27.8–36.5° against 23–41°: the sim has one template, MLB has many swings). Swing length is fitted through two timing constants; its spread and its rise out front and outside are not. The vertical bat angle, the AA–AD slope and bat speed by location come out of the geometry.

**By horizontal location** (v2; MLB in brackets; ft from the plate's center line, + inside):

| | outside | mid-away | middle | mid-in | inside |
|---|---|---|---|---|---|
| Bat speed, mph | 70.1 (70.1) | 70.7 (71.4) | 70.8 (72.1) | 71.1 (72.0) | 71.1 (71.4) |
| Tilt, ° | 30.6 (29.7) | 31.3 (31.2) | 32.2 (32.3) | 33.4 (33.8) | 34.1 (35.1) |
| Swing length, ft | 7.95 (7.45) | 7.61 (7.31) | 7.38 (7.23) | 7.10 (7.19) | 6.76 (7.13) |
| Attack direction, ° pull + | +9.6 (−1.6) | +7.6 (−1.4) | +7.4 (+0.6) | +6.5 (+3.6) | +4.5 (+6.5) |
| Fair spray, ° pull + | −5.7 (−1.3) | −5.7 (+4.5) | −1.3 (+6.9) | −0.5 (+8.4) | +1.3 (+14.3) |
| Batter's timing error, ms (− early) | −17.2 | −11.8 | −9.6 | −6.4 | −2.3 |

Bat speed and tilt now match by location. Attack direction does not, and the last row says why: the sim's batters are 17 ms early on outside pitches and 2 ms early inside (on outside pitches the breaking balls fool them, foul note §6). On time, in the cage (`swing_cage_probe`), v2's attack direction does run from +1.7° inside to −4.6° outside, as planned. The inverted AD by location is perception, not the swing.

**Bat speed by intercept depth** (relative to the batter's mean, in; MLB in brackets): −12 or deeper 70.5 (68.3); −12 to −6 71.3 (70.6); −6 to −2 71.4 (71.8); ±2 71.2 (72.9); 2 to 6 71.4 (73.1); 6 to 12 70.5 (73.4); 12+ 68.1 (72.6). Statcast's speed keeps rising out front; v2's falls off after the bat's peak. Moving the peak later would lengthen the swing (Open 6).

## 4. Fouls

| | v1 | v2 | MLB |
|---|---|---|---|
| Foul share, PA model (R/R) | 23.7% | 27.3% | 51.2% |
| Foul share, games (`foul_in_games`, 128 games, rules engine) | 21.9% | 24.2% | 52.5% |
| \|AD\| < 6°: foul per contact | 20.4 | 25.6 | ≈40 |
| AD 12–20°: foul per contact / whiff per swing | 8.1 / 42.7 | 20.3 / 51.2 | 54 / 23 |
| Foul by height: below / bottom / middle / top / above | 16 / 30 / 23 / 20 / 19 | 18 / 30 / 25 / 27 / 38 | 52 / 47 / 47 / 58 / 69 |
| Fouls with pull-side AD | 34% | 40% | 45% |
| Early fouls (AD > 10°), LA p50 | 7° (n 75) | 21° (n 82) | 0° |
| "Close" swings (±15 ms, ±4 cm): foul per contact | 9.8% | 15.6% | (≈40 for \|AD\| < 6°) |
| P(foul) by LA, games: < −30 / −30..−10 / −10..10 / 10..25 / 25..40 / 40..50 | 17 / 7 / 4 / 8 / 23 / 44 | 23 / 8 / 4 / 10 / 28 / 57 | 57 / 41 / 22 / 37 / 47 / 66 |

The steeper plane makes high pitches and well-timed contact foul more often (top third 20 → 27%, above the zone 19 → 38%, close swings 10 → 16%). The deficit stays where the foul note put it: low-angle contact (below 25° the sim fouls 4–10% against 22–57%) and early contact. The AD spread is unchanged (fair contact ±6–7° against ±14–19°): it is set by how far off the batter's timing is, and v2 does not change the batter. **What the swing shape cannot do here is supply the moderately mistimed contact MLB fouls off.** The candidates left are the ones the foul note named: the batter's timing and aim (perception and correction, Tier B), and the on-time foul band of the contact map (topped and undercut balls staying fair).

## 5. The run environment

**Standard run** (300 games; seeds 0–299 [1000–1299]; main `bcbb91f` → branch `63f58ab`):

| | main | branch | MLB |
|---|---|---|---|
| Runs / team-game | 5.30 [5.51] | **3.74 [3.86]** | 4.4–4.6 |
| K% | 15.9 [16.0] | 15.9 [15.6] | 22.6 |
| BB% | 7.2 [7.4] | 7.1 [7.4] | 8.2 |
| HR / PA | 5.11% [5.35%] | **0.56% [0.55%]** | 3.0% |
| Reached on BIP | 33.6% [33.6%] | 38.3% [39.0%] | |
| GB share / hit | 51.9% / .322 [51.5% / .314] | 45.0% / .357 [44.7% / .350] | 43% / .24 |
| LD share / hit | 17.6% / .703 [17.7% / .716] | 24.4% / .654 [24.5% / .682] | 21% / .68 |
| FB share / hit | 21.8% / .139 [21.9% / .141] | 25.4% / .172 [25.3% / .169] | 29% / .13 |
| PU share / hit | 8.7% / .017 [8.9% / .018] | 5.2% / .027 [5.4% / .017] | 7% / .02 |
| PA / team-game; pitches / game | 38.4; 224 [38.7; 224] | 38.7; 224 [39.3; 228] | |
| Fouls / game; foul outs / game | 14.1; 2.07 [14.1; 2.10] | 17.4; 1.50 [18.0; 1.66] | ; about 1.4 |

**Where the MLB column comes from.** K% 22.6, BB% 8.2 and HR/PA 3.0% are the 2024 Baseball Savant league totals (41,197 K, 14,929 BB and 5,453 HR in 182,449 PA = 22.58%, 8.18%, 2.99%; BB includes intentional walks), and 4.39 runs a team-game is Baseball-Reference 2024, all from [sources](2026-11-14-sources.md). [calibration-targets](../calibration-targets.md) B1.1 and B1.2 give 22.5% and 8.3%, which are approximate \[R] values from general knowledge awaiting its R10 refresh (K% band 15–28%, BB% 5–12%), so the 0.1-point differences are two sources, not an error. Its B3.3 (1.1 HR per team-game, band 0.6–1.6) agrees with the totals: 5,453 HR over 4,860 team-games is 1.12. The runs column's 4.4–4.6 is a range, not a sourced figure: the repo's sourced value is 4.39 (2024) and calibration-targets B3.1 says 4.4.

**Batted balls** (`run_env_diag`, 100 games, seeds 0–99; MLB 2025 sample, balls in play):

| | main | branch | MLB |
|---|---|---|---|
| Exit speed mean / SD, mph | 91.4 / 16.8 | 89.1 / 12.5 | 89.5 / (sample) |
| p90 / p99 / max | 112.1 / 125.9 / 136.2 | 102.0 / 107.3 / 112.9 | 105.2 / 111.0 / 116.7 |
| ≥ 95 / 100 / 105 / 110 mph | 42.6 / 31.4 / 20.6 / 12.9% | 39.5 / 17.0 / 3.2 / 0.1% | 40.7 / 25.1 / 10.6 / 2.1% |
| Launch angle mean / SD | 8.9 / 30.0° | 11.7 / 25.1° | 12.2 / 29.1° |
| Barrels / BIP | 11.9% | 3.3% | ≈8.7% (approximate window) |
| Bat speed on fair BIP, mean / p90; on HRs | 74.3 / 87.8; 86.3 | 71.3 / 75.5; 76.1 | ; HRs 75.4 |
| Runs / team-game; HR / PA | 5.52; 5.09% | 3.57; 0.48% | |

**Platoon** (`platoon_diag`, 2,000 PA per cell, seed 3): K% R/R 20.9 → 20.5, L/L 19.1 → 18.8, R/L 13.6 → 12.7, L/R 14.5 → 12.3, so the same-minus-opposite gap widens from 6.0 to 7.2 points (MLB about 1.5). OF-HR/PA R/R 4.10 → 0.65%, L/L 3.95 → 0.55%, R/L 5.15 → 0.35%, L/R 6.10 → 0.60%: v1's opposite-handed home-run surplus (the outer third at 84 mph) is gone. Exit speed by third, R/R: inside / middle / outside 84.0 / 90.3 / 99.0 → 91.6 / 89.6 / 87.7 (MLB peaks over the middle). The platoon K gap is the perception mechanism of the diagnosis's §4; the swing does not touch it.

**Where it lands, and what is still off.**
1. **Home runs are a third of MLB's at the right mean exit speed.** v2's mean exit speed (89.1) and hard-hit share (39.5%) match MLB; its tail does not (≥105 mph 3.2% against 10.6%). Every rostered player is grade 50, so the league has no between-batter bat-speed spread, and home runs are convex in bat speed. Counterfactual, effort σ widened to 5.96% to stand in for Statcast's 2.8 mph between-batter SD: HR 0.48 → 0.95% of PAs, exit speed p99 109.7, max 116.0, runs 3.57 → 3.99 (100 games).
2. **Barrels are short too** (3.3% against about 8.7%). In the cage (`Cage::plan_perfect` aims at the ball's center and does not apply the PA model's `aim_offset_z`), an on-time squared-up four-seamer leaves at 2.8° at zero offset (planned AA about 8°; `aim_offset_probe`, middle height). The 2–4° this note first gave was labelled "aim 2 cm under center", which was wrong: with the bat 2 cm under the ball the cage gives 19.0°, and 27.3° at 3 cm, where exit speed is 97.7 mph ([recalibration report](2026-11-14-swing-recalibration.md) §2.2). A ball launched at 25–30° is undercut and loses exit speed. Whether that is the batter's aim offset (`aim_offset_z`, authored against mean launch angle on v1) or the collision is open.
3. **Collision efficiency.** Statcast's squared-up definition uses 1.23 × bat speed + 0.23 × pitch speed at the plate, a q of 0.23; the sim's wood bat is tuned to q = 0.20 (Nathan; Tier A). Counterfactual, e_ref 0.442 → 0.478 (q 0.23): HR 2.51%, but mean exit speed 93.1 and hard-hit 54.7% (MLB 89.5, 40.7%). Both counterfactuals together: HR 3.06%, mean 93.3, runs 5.74. **The data do not support q alone as the fix**: it moves the whole distribution, where the gap is in the tail and the joint with launch angle. Nothing in physics was changed.
4. **More hits on balls in play.** Reached on BIP 33.6 → 38.3%: more line drives (17.6 → 24.4%), fly balls that die at the wall (wall hits that are not home runs 743 → 1,029 per 100 games; FB hit rate .139 → .172), and grounders still hit at .357 (the infield items of the diagnosis's rank 2–3).
5. **K%, BB% and pitches per PA do not move.** Those need the foul share (§4) and the zone rate (diagnosis rank 4).

## 6. The owner's ruling (2026-11-14)

The owner ruled on proposals A–D on 2026-11-14 ("I'll take your recommendations", through the lead). PLAN §1 D19.

- **A, decided: v2 is the swing.** It stays off main until the recalibration brings home runs and runs back; merging to main still waits for the owner's word.
- **B, decided, now:** per-player `BatSpeed` drawn from a population distribution (Statcast between-batter SD 2.8 mph). Work: [2026-11-14-swing-recalibration.md](2026-11-14-swing-recalibration.md).
- **C, decided, before D:** re-author `aim_offset_z` and check the launch-angle spread against Statcast (mean 12.2°, SD 29.1° in play). Same report.
- **D, not now:** collision efficiency stays at q = 0.20. Revisit only if B and C leave a gap in the exit-speed tail, and then as a sourcing question, not a tuning knob.

What was proposed, for the record: A, take v2 (it takes the run environment from 0.8 runs high to 0.7 low; population means at sourced values, nothing tuned to a run total). B, compare against a population, not an average league (largest lever found on home runs, +0.5 points of HR/PA by the effort-σ proxy). C, re-author `aim_offset_z` (authored on v1) before touching the collision. D, ask whether q = 0.20 or Statcast's squared-up 0.23 is right for game wood (the counterfactual says not alone).

## 7. Open questions

1. **Spray against attack direction.** MLB's balls in play go 7.2° (R/R) to 8.9° (all) more toward the pull side than the barrel's path at contact; v2's on-time contact goes about 5° the other way (cage: AD −1.4°, spray +4.1° at the middle). The collision decides this (where the ball meets the round bat, the pitch's incoming angle, the bat's 30° slope); the swing is ⊥ to its path at contact by design (λ_c = 0). Not resolved.
2. **Spray by location is now nearly flat in games** (−0.5° to +1.6° batter-relative across the plate; MLB −12.6° to +0.6°, v1 −6.5° to +7.7° relative). On time the swing gives the right trend (§3); the batters' early bias on outside pitches cancels it. The fielders' location prior was refit to what the sim does, as before.
3. **Batter timing bias by location** (−17 ms outside, −2 ms inside): perception, the diagnosis's rank 5 (pursuit limit, sideways break). It also drives the attack-direction mismatch and much of the platoon gap.
4. **Count effects on bat speed**: MLB swings 0.9 mph slower per strike (2024 analysis, vavetsbarets/mlb-swing-speed-analysis; this sample: 72.3 / 71.7 / 70.3 mph at 0 / 1 / 2 strikes). No two-strike swing is modeled.
5. **Out of plane and the load.** v2's swing is planar, so its start pose is flatter than a real load (the bat starts nearly level, pointing at the catcher, instead of up over the shoulder). The client draws the bat from the swing; not built or checked here.
6. **Bat speed out front.** Statcast's speed holds 6–12 in out front; v2's falls off. A later bat peak would lengthen the swing past 7.2 ft unless the widths change too.
7. **The lag at contact** is 0 by assumption (the first draft's spec value). No sourced value was found for the lead-arm-to-bat angle at contact, nor for hand speed at contact (q = 0.32 gives 14 mph).

## 8. Reproduction

- Before: `git checkout bcbb91f`; after: `work/swing-model` at `63f58ab`. `cargo run --release -p bb_game --example foul_diag -- 3000`; `... platoon_diag -- 2000 3`; `... run_env_diag -- 100 0`; `... foul_in_games -- 128`; `sim-cli games --n 300 --seed 0` and `--seed 1000`; refits: `spray_by_bat_angle -- 96`, `spray_cues -- 200`, `spray_by_location -- 96`, `bat_read_geometry -- 48`; cage: `swing_cage_probe`.
- Counterfactuals were temporary edits, reverted: `motor::EFFORT_SIGMA` 0.045 → 0.0596; `bb_physics::bat` `e_ref` 0.442 → 0.478.
- Statcast: `https://baseballsavant.mlb.com/statcast_search/csv?all=true&hfGT=R%7C&hfSea=2025%7C&player_type=batter&game_date_gt=<d>&game_date_lt=<d>&type=details` for the six dates above, retrieved 2026-11-14. Attack direction's CSV sign: negative = pulled (inside pitches and out-front contact are negative). Spray from hit coordinates: `atan2(hc_x − 125.42, 198.27 − hc_y)`, mirrored for left-handed batters. Within-batter SD: batters with ≥ 25 competitive swings (216, 7,697 swings). Other sources: Statcast league average bat speed 71.5 mph and swing length 7.3 ft (ESPN, May 2024 rollout); squared-up formula (FanGraphs, "When Squaring It up Goes Sideways"); Blast time to contact 0.149 s and vertical bat angle (vendor, via R13).
