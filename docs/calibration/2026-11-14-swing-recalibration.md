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

## 2. Results

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

### 2.3 C: the aim offset in games (`run_env_diag -- 100 0`, σ_pop 1.25; `BB_AIM_OFFSET_Z`)

| `aim_offset_z` | runs | K% | HR/PA | LA mean / SD | sweet spot | GB / LD / FB / PU | barrels | EV mean; ≥ 105 |
|---|---|---|---|---|---|---|---|---|
| −1.5 cm | 4.76 | 14.7 | 0.74% | 8.8 / 25.5 | 35.8% | 49.9 / 23.3 / 22.3 / 4.5 | 3.51% | 89.3; 6.2% |
| **−2.0 cm** (kept) | 4.54 | 15.8 | 0.96% | 11.8 / 25.1 | 38.0% | 44.5 / 25.0 / 25.0 / 5.5 | 4.55% | 89.6; 6.5% |
| −2.5 cm | 4.38 | 16.7 | 0.88% | 13.9 / 25.1 | 38.4% | 40.5 / 25.4 / 28.0 / 6.2 | 4.54% | 89.3; 5.5% |
| −3.0 cm | 3.93 | 18.2 | 1.06% | 16.6 / 25.5 | 37.9% | 36.4 / 24.5 / 31.1 / 7.9 | 5.13% | 88.6; 4.8% |
| MLB (2025 sample; targets §4.4) | 4.39 | 22.6 | 3.0% | 12.2 / 29.1 | 33% | 43 / 21 / 29 / 7 | ≈ 8.7% | 89.5; 10.6% |

- **The aim moves the mean, not the spread.** Each 0.5 cm under the ball adds about 2.6° of mean launch in games (8° per cm on time in the cage, diluted by mistimed contact); the SD stays at 25.1–25.5°. Barrels and home runs barely move (within about ±0.1 points of HR/PA, the noise of 100 games): they are limited by exit speed, not by launch.
- **Decision: `aim_offset_z` stays at −2.0 cm.** It gives 11.8° against Statcast's 12.2° (−2.1 cm would close the gap; not worth chasing), inside the band of targets §4.4 (12°, 6–18). Under §2.3 item 3 an agent [E] parameter moves only when its band is off; this one is not. Only the doc comment changed (it now says it was re-checked on v2).
- **What is off is the shape.** At the right mean, the sim's launch angles are too concentrated (SD 25.1° against 29.1°; inside the envelope 20–35, under the ~27° target): too many line drives (25.0% against 21%) and too few fly balls and pop-ups (25.0 + 5.5 against 29 + 7). That is part of why balls in play fall for hits so often. Getting MLB's mix by aiming higher (−2.5 cm gives GB 40.5 / FB 28.0) puts the mean at 13.9° and K% does not care; the band says no.
- **Counterfactual: the swing-noise knob** `K_swingplane` 1.0 → 1.5 (`BB_K_SWING`): LA SD 25.1 → 27.2°, mean 10.2°, sweet spot 34.8%, but runs 5.10, max exit speed 128.7 mph (the effort σ scales with the knob too, so the tail grows past targets §4.4's "none over 125") and GB hits .361. Not adopted: it mixes the vertical miss with the effort, and the ruling did not ask for it. The vertical miss that sets the LA spread is mostly perception's (the batter's height estimate) and `BatControl`'s; a `BatControl` population draw (M7) would widen it between batters.

### 2.4 Spray (diagnosis; `foul_diag -- 3000`, average R/R, pull-positive by first landing)

| | outside | mid-away | middle | mid-in | inside |
|---|---|---|---|---|---|
| Batter's timing error, ms (− early) | −17.2 | −11.8 | −9.6 | −6.4 | −2.3 |
| All fair contact: spray | −5.7 | −5.7 | −1.3 | −0.5 | +1.3 |
| All fair contact: spray − AD | −6.9 | −7.4 | −4.5 | −5.1 | −3.2 |
| On time (±5 ms): spray | −15.5 | −13.6 | −9.8 | −8.5 | −3.2 |
| On time: AD | −4.0 | −2.3 | −1.0 | +0.7 | +2.0 |
| On time: spray − AD | −11.4 | −11.3 | −8.8 | −9.2 | −5.2 |
| MLB spray (Savant 2025) | −1.3 | +4.5 | +6.9 | +8.4 | +14.3 |
| MLB spray − AD | +7.2 (R/R) to +8.9 (all) | | | | |

Both mechanisms of the plan are confirmed:
1. **On time, the trend by location is there** (−15.5° outside to −3.2° inside, a 12° range against MLB's 15.6°), but the whole curve sits on the opposite-field side: on-time contact goes 5–11° to the opposite side of the sweet spot's path, where MLB's goes 7–9° to the pull side. That is a 15–20° offset at contact, and it is not the batters'. The swing meets the ball with the bat ⊥ to its path (λ_c = 0, an assumption with no source, swing note Open 7), and an undercut ball off a barrel tilted 28° down leaves toward the tip's side. A bat already turned a little past the hands' line at contact (λ_c > 0) would face the pull side of its path. That is a swing [E] parameter that could be fitted to Statcast's spray − AD; it is outside B and C and would refit the fielders' spray beliefs again, so it is **left for the owner** as a proposal.
2. **The batters' timing flattens it.** They are 17 ms early on outside pitches and 2 ms inside (perception, diagnosis rank 5). Early contact is pulled, so outside balls are pulled back toward the middle and the location trend collapses to 7°. Fixing the timing alone would make spray worse (outside balls would go 15° the other way) unless λ_c moves with it: **the two belong together.**
3. The infield's alignment for the pull is the fielding lead's; it is fit to where balls go, so it follows whatever these two do.

### 2.5 Standard run (300 games, seeds 0–299 [1000–1299]; `sim/out/recalib-seed0.txt`, `recalib-seed1000.txt`)

| | `9f71441` | B (`cf1cce4`) | MLB |
|---|---|---|---|
| Runs / team-game | 4.57 [4.53] | 4.79 [4.50] | 4.39 |
| K% / BB% | 15.3 / 7.3 [15.6 / 7.3] | 15.5 / 7.4 [15.8 / 7.5] | 22.6 / 8.2 |
| HR / PA | 0.57% [0.53%] | **1.01% [0.94%]** | 3.0% |
| Reached on BIP | 39.3% [39.7%] | 39.3% [38.1%] | |
| GB share / hit | 45.5% / .364 [44.0% / .364] | 45.1% / .367 [43.8% / .355] | 43% / .245 |
| LD share / hit | 24.5% / .668 [25.0% / .673] | 24.3% / .677 [24.7% / .652] | 21% / .68 |
| FB share / hit | 24.7% / .174 [25.5% / .177] | 24.9% / .168 [26.1% / .170] | 29% / .13 |
| PU share / hit | 5.3% / .017 [5.5% / .021] | 5.6% / .026 [5.4% / .025] | 7% / .02 |
| Home runs / game (out of play) | 0.45 [0.42] | 0.79 [0.73] | 2.2 |

Runs move +0.22 and −0.03 on the two sets: within their noise. Home runs move from a fifth to a third of MLB's.

### 2.6 Attack angle by zone third (step 4)

The Savant sample (re-run, `savant_bat.py`, competitive swings) gives 11.9 / 7.8 / 6.1° bottom / middle / top (17.4 below, 6.0 above): the spec's text quotes the data correctly. The linear formula 8 − 8.7·(h − ½) gives 10.9 / 8.0 / 5.1° at the thirds' centers, about 1° low at both ends (the data are convex in height). The slope is exactly the outer thirds' difference over the 2/3 of a zone height between their centers ((6.1 − 11.9)/(2/3) = −8.7), and the level, 8°, sits near the middle third's 7.8: no line passes through all three. **The wrong one was the spec's text**, which read as if the line reproduced the thirds; it now says how the line was made and what it misses (agent-models §3.6 step 2). The code is unchanged. A convex curve (or a least-squares refit over swings) would put about 1° more lift on low and high pitches; that changes games and is left as an option, not done here.

### 2.7 Named literals (step 4)

`motor::PLANE_TILT_PER_M` = 1.5 rad per m of δ_v's σ (1.0° at grade 50; spec value now given, marked unsourced: it dates from M2, `7358907`), `motor::REACH_PER_BAT_CONTROL` = 0.7, `swing::TILT_GUARD_MARGIN` = 1°; agent-models §3.6 names them. Checked: game digests for seeds 1, 2, 3, 42 and the whole `sim-cli games --n 300 --seed 0` output are identical before and after.

## 3. Where this leaves the gaps

1. **Home runs: 1.0% of PA against 3.0%.** B was worth what the proxy said, and C has no home runs to give: the aim's sweep moves launch, not exit speed. What is left is outside the swing's abilities:
   - **Collision efficiency** (D, ruled "not now"): the sim's squared-up average swing leaves at 103 mph, Statcast's squared-up formula would give about 107.5 at the same bat and pitch speeds.
   - **Batted backspin** (§2.2): about twice the published values at home-run launch angles, costing 15–30 ft of carry. A Tier A question for the collision (physics-spec §8: e_t 0.40 from KNS 2017; test A10's magnitudes still pending). The swing note's counterfactual (q 0.23, plus the spread proxy) reached 3.06% HR, but with mean exit speed 93 against 89.5. Lower spin would add carry without raising exit speed, so **the sourcing question for D should take spin first**. Nothing here was changed: per the ruling, a sourcing question, not a knob.
   - The sim's home runs need faster swings than MLB's (77.7 against 75.4 mph): the same two causes, read from the other side.
2. **Runs are right for the wrong reasons**, still: 4.79 [4.50] against 4.39, with a third of the home runs. The surplus is hits on balls in play: grounders (.36 against .245, fielding's) and the over-concentrated launch angles (too many line drives, §2.3).
3. **Spray** (§2.4): the bat's face at contact (λ_c) and the batters' timing by location, together. Proposal for the owner.
4. **K% and fouls**: unchanged by B and C, as the plan said (15.5 [15.8] against 22.6).
