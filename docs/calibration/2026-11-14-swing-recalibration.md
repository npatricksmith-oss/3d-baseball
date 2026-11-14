# Swing v2 recalibration: per-player bat speed and the aim under the ball (2026-11-14)

Branch `work/swing-recalib`, cut from `work/swing-model` at the merge `9f71441`. Owner's ruling on the swing note's proposals (PLAN §1 D19; [swing note §6](2026-11-14-swing-model.md)): B (per-player `BatSpeed`, Statcast between-batter SD 2.8 mph) and C (re-author `aim_offset_z`, check the launch-angle spread) go ahead; D (collision q 0.20 → 0.23) does not. Calibration rules: [calibration-targets §2.3–§2.4](../calibration-targets.md). Tier B numbers move only through population distributions and noise scales; an agent [E] parameter moves only when a Tier M band shows its mechanism is off.

**Where it starts** (`9f71441`; standard run, seeds 0–299 [1000–1299]; MLB 2024 or the 2025 Savant sample of the swing note):

| | sim | MLB |
|---|---|---|
| Runs / team-game | 4.57 [4.53] | 4.39 |
| HR / PA | 0.57% [0.53%] | 3.0% |
| K% / BB% | 15.3 / 7.3 [15.6] | 22.6 / 8.2 |
| GB hit rate | .364 [.364] | .245 |
| LD share, hit rate | 24.5%, .668 | 21%, .68 |
| FB share, hit rate | 24.7%, .174 | 29%, .13 |

`run_env_diag -- 100 0` at `9f71441` (the cheap instrument; MLB from the 2025 sample, balls in play): exit speed 89.0 / SD 12.4 mph (89.5); p90 / p99 / max 102.0 / 107.4 / 113.1 (105.2 / 111.0 / 116.7); ≥ 105 mph 3.4% (10.6%); launch angle 11.1 / SD 25.5° (12.2 / 29.1); barrels 3.15% of BIP (≈ 8.7%); HR 0.49% of PA; runs 4.57.

## 1. Plan

Ranked by expected effect on the gap that matters most (home runs, then the run environment's composition).

### 1.1 Home runs and the exit-speed tail (in reach: B)

- **Mechanism.** Every rostered batter has `BatSpeed` 32.0 m/s (71.6 mph). With q = 0.20, a perfectly squared-up four-seamer at that bat speed leaves at about 103 mph (cage, `aim_offset_probe`), so balls of 105+ mph need an above-average swing, and the only spread is the within-batter effort σ (4.5%). Home runs are convex in exit speed (sim: 2.3% of BIP at 100–105 mph, 8% at 105–110, 30% above 110), so a league with no between-batter spread loses most of them. MLB's home runs come from fast swings: their bat speed is 75.4 mph on average, 3.0 mph above the hitter's own mean (Savant sample).
- **Lever (population distribution, §2.3 item 1).** Draw each batter's `BatSpeed` once, before the first pitch: grade `g = 50 + 10·z`, z standard normal truncated at ±3 (calibration-targets §11.1), physical value `32.0 + σ_pop·(g − 50)/10`. σ_pop = 1.25 m/s (2.8 mph).
- **Source.** Savant 2025 sample (`analysis/statcast/savant_sd.py`, re-run for this report): SD of 216 batters' mean competitive bat speed 2.90 mph, of which sampling error is 4.29/√35.6 = 0.72 mph, so the spread of true means is √(2.90² − 0.72²) = 2.81 mph = 1.256 m/s. Mean of means 71.68 mph (the grade-50 value, 32.0 m/s = 71.58, stays). Calibration-targets §4.3 has ~2.5 mph (envelope 1.5–4); agent-models §10 had σ_pop 1.1 m/s (2.46 mph). The owner ruled 2.8.
- **Not included, and why.** Position shifts (§11.2, BatSpeed +5 at 1B, −3 at SS, +3 at LF/RF, +6 at DH) are first guesses for M7 and would move the league mean off the sourced 71.7; the `BatSpeed`–`BatControl` correlation needs `BatControl` drawn too. Both wait for the league generator. Within-batter effort stays at 4.5%.
- **Measure.** `run_env_diag -- 100 0` at σ_pop 0 / 1.1 / 1.25 m/s: HR/PA, exit-speed p90 / p99 / max and ≥ 105 share, bat speed on fair BIP and on HRs (MLB 75.4), runs. Expected from the swing note's proxy (effort σ widened to stand for the between-batter spread): HR 0.5 → about 1.0% of PA.
- **Not touched.** The collision (q 0.20, e_ref, e_t, μ), the flight model, the effort σ.

### 1.2 Barrels and launch angle (in reach: C, partly)

- **Mechanism.** The aim offset sets where the batter puts the sweet spot against the ball's predicted center; in the cage each centimeter under the ball adds about 8° of launch at an on-time middle swing, and costs exit speed (103 mph at 0 cm, 101 at −2 cm, 97.7 at −3 cm, 27°). `aim_offset_z` = −2.0 cm was authored on v1 against the mean launch angle. The launch-angle spread is not the aim's: it comes from the vertical miss (perception error, `BatControl`, timing through the attack angle). Barrels (98+ mph at 26–30°, wider above) need exit speed the average swing barely has at that launch, so they are limited by the exit-speed tail (1.1) as much as by the launch angle.
- **Lever.** `aim_offset_z`, an agent [E] parameter (§2.3 item 3), only if a Tier M band (calibration-targets §4.4: mean LA 12° [6–18], LA SD ~27° [20–35], barrels 7.5% [4–11], sweet spot 33% [26–40]) shows it off after B. Today mean LA 11.1° and SD 25.5° are inside their bands; barrels (3.15%) are not.
- **Source.** Savant 2025 sample: LA in play mean 12.2°, SD 29.1°, approximate barrels 8.7% of BIP.
- **Measure.** Cage sweep of exit speed, launch and carry against the vertical offset (`aim_offset_probe`), then `run_env_diag` at `aim_offset_z` −1.5 / −2.0 / −2.5 / −3.0 cm, with B in place: LA mean / SD, sweet-spot share, barrels, HR/PA, GB / LD / FB / PU shares.
- **Not touched.** The perception scales (they set the LA spread; diagnosis ranks 4–5, not the swing's), the template's attack angle (fitted to Statcast), the collision.

### 1.3 Spray (diagnosis only; the fix is not B or C)

- **What is off.** MLB balls in play leave 7.2° (R/R) to 8.9° (all) more toward the pull side than the sweet spot's path (spray − AD); v2's on-time cage contact goes about 5° the other way. In games v2's spray by location is nearly flat (−0.5° to +1.6°), where MLB runs from +14° pulled inside to −1° outside; the infield is aligned for the pull (swing note Open 1–3).
- **Mechanisms to test.** (i) The swing's lag at contact λ_c = 0 (face ⊥ to the path by assumption, swing note Open 7): a bat already turned past the hands' line at contact would face the pull side of its path. (ii) The batters' early bias on outside pitches (−17 ms outside, −2 ms inside) cancels the on-time location trend.
- **Measure.** Spray by location on time (cage) against all contact and against contact within ±5 ms of on time, in the PA model (`foul_diag`), with spray − AD.
- **Out of reach of B and C.** λ_c is a swing [E] parameter with no sourced value; fitting it to spray − AD is a separate proposal for the owner. The timing bias is perception (diagnosis rank 5). Infield alignment is the fielding lead's.

### 1.4 Ground-ball hits (out of reach)

GB hit rate .364 against .245 is the fielding side's (diagnosis ranks 2–3: holes in the alignment, diving infield hits, throw choices; the other lead's). The swing only sets the grounders' mix: C can move some hard 0–10° grounders into the air and B adds a few harder ones. Reported, not tuned.

### 1.5 K% and fouls (out of reach)

K% 15.3 against 22.6 comes from the foul share (23% of contact against 52%) and the zone rate (55.5% against about 43–48%): the batter's timing and aim (perception) and the pitchers' targets (diagnosis ranks 1, 4–5). B moves K% only slightly (faster swings are not more accurate). Reported, not tuned.

### 1.6 Also in this branch (no game digest may change)

- Name the v2 literals the swing-note check found: the 1° tilt guard (`swing.rs`), the plane-tilt noise ratio 1.5 rad/m and the reach ratio 0.7 (`motor.rs`); give the 1.5 a spec value.
- Resolve the attack-angle discrepancy in agent-models §3.6: the formula 8° − 8.7°·(h − ½) gives 10.9 / 8.0 / 5.1° at the centers of the zone thirds; the text says the sample gives 11.9 / 7.8 / 6.1°.
- Checked by the game digests (`sim-cli games` / the `digest` example) before and after.

### 1.7 What would stop this

If home runs stay far short after B and C without a fudge factor, the remaining gap is the collision's (q, D) or the batted ball's spin (§2.4 below), both Tier A. That is a result to report, not a reason to tune a Tier B knob against it.

## 2. Results (in progress)

### 2.1 B: per-player `BatSpeed` (`run_env_diag -- 100 0`; outputs in `sim/out/recalib/`)

Built: `bb_game::population` (stream `RosterAbilities` = 15, one truncated-normal z per batter at ±3, `BatSpeed = 32.0 + σ_pop·z`), one matchup per batting slot, `Game::with_population` and the lab hook `Game::map_matchups`; `run_env_diag` reads `BB_BAT_SPEED_SIGMA` and `BB_AIM_OFFSET_Z`. σ_pop = 0 reproduces `9f71441`'s output line for line (checked by diff).

| σ_pop, m/s | runs | K% | HR/PA | EV mean / SD | p90 / p99 / max | ≥ 105 / ≥ 110 | LA mean / SD | barrels | bat on fair BIP mean / p90; on HRs |
|---|---|---|---|---|---|---|---|---|---|
| 0 (`9f71441`) | 4.57 | 15.8 | 0.49% | 89.0 / 12.4 | 102.0 / 107.4 / 113.1 | 3.4 / 0.2% | 11.1 / 25.5 | 3.15% | 71.3 / 75.5; 76.6 |
| 1.10 (old §10) | 4.62 | 16.0 | 0.82% | 89.5 / 12.7 | 103.1 / 109.6 / 118.6 | 5.6 / 0.7% | 11.6 / 25.1 | 4.37% | 71.4 / 76.7; 78.4 |
| **1.25** (D19) | 4.54 | 15.8 | **0.96%** | 89.6 / 12.6 | 103.2 / 109.8 / 119.6 | 6.5 / 1.0% | 11.8 / 25.1 | 4.55% | 71.4 / 77.1; 77.7 |
| MLB | 4.39 | 22.6 | 3.0% | 89.5 | 105.2 / 111.0 / 116.7 | 10.6 / 2.1% | 12.2 / 29.1 | ≈ 8.7% | 71.7; HRs 75.4 |

B doubles home runs, as the proxy said, and leaves runs where they were (within noise). The sim's home runs need a faster swing than MLB's (77.7 against 75.4 mph).

### 2.2 The cage: offset under the ball (`aim_offset_probe`, four-seamer, middle height, on time)

| offset cm | 0 | −1 | −2 | −2.5 | −3 | −4 |
|---|---|---|---|---|---|---|
| EV mph | 103.2 | 102.9 | 101.1 | 99.6 | 97.7 | 92.6 |
| LA ° | 2.8 | 10.9 | 19.0 | 23.1 | 27.3 | 35.8 |
| carry ft | 88 | 280 | 356 | 357 | 346 | 305 |
| backspin rpm | 1,190 | 2,258 | 4,028 | 4,954 | 5,890 | 7,773 |
| carry at 2,500 rpm, ft | 82 | 283 | 360 | 373 | 375 | 353 |

About 8° of launch per cm. A perfectly squared-up average swing in the barrel window (27°) leaves at 97.7 mph, just under the barrel line. **Batted backspin is about twice the published range** (≈ 5,900 rpm at 27°, where Statcast-era measurements put fly balls near 2,500): it costs 15–30 ft of carry at 23–27°. That is the collision's tangential behaviour (physics-spec §8, e_t 0.40, test A10 "values pending"), Tier A, outside B and C; it goes with D as a sourcing question.

### 2.3 Attack angle by zone third (step 4)

The Savant sample (re-run, `savant_bat.py`, competitive swings) gives 11.9 / 7.8 / 6.1° bottom / middle / top (17.4 below, 6.0 above): the spec's text quotes the data correctly. The linear formula 8 − 8.7·(h − ½) gives 10.9 / 8.0 / 5.1° at the thirds' centers, about 1° low at both ends (the data are convex in height). To settle: re-fit α₀ and α_h by least squares over swings, or state that the fit is a line through a convex curve; a change to α₀/α_h changes games, so it is not part of the digest-neutral step.

### Next
C sweep (`BB_AIM_OFFSET_Z` −1.5 / −2.5 / −3.0 cm with σ_pop 1.25), spray diagnostics (c), spec edits for B (agent-models §10 σ_pop 1.1 → 1.25, data-formats §2.3 purpose 15 and §2.4 step 5, calibration-targets §4.3), tests, standard runs, ledger row, the named literals.
