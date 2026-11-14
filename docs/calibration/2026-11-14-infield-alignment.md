# Rank 2 calibration step: Baseball Savant 2024 infield alignment

- **Branch / commits:** `work/infield-alignment`, branched from `main` `a791a9e` (`f333496` diag, `6dfe643` spec, `d565cf6` alignment + tests)
- **Target gap:** Ground-ball hit rate (`0.348` [0.347] provisional vs MLB ~`.24`), diagnosed in [run-environment-diagnosis.md](2026-11-14-run-environment-diagnosis.md#q3-ground-ball-hits) as `67.8%` of through-the-hole grounders passing through `23°`/`25°` corner holes left by symmetric `±13°/+38°/−36°` `home_positions()`.

---

## 1. Sourced MLB league-average infield positioning

All four published non-shifted league-average infield alignments were pulled directly from Baseball Savant's Fielder Positioning dataset ([visual](https://baseballsavant.mlb.com/visuals/fielder-positioning), query endpoint `/visuals/position_data?type=player&teamId=&firstBase=<0|1|27>&shift=0&batSide=<R|L>&season=2024&position=&attempts=1`, PA-weighted with `attempts=1`) for the **2024 MLB season** under MLB's 2023+ two-infielders-per-side positioning rules, using **`shift=0` (`Shaded: No`, standard non-shifted alignment)**. Cross-checking 2023 and 2025 showed stability within `~0.5 ft` depth and `~0.5°` angle. Coordinates use home-plate origin, depth in feet (`0.3048 m/ft`), and horizontal spray angle in degrees (`0°` = CF `+Y`, `−45°` = 3B line, `+45°` = 1B line).

| Alignment (`season=2024`, `shift=0`) | `n` (PA) | `1B` depth / angle | `2B` depth / angle | `3B` depth / angle | `SS` depth / angle | Source filter |
|---|---:|---|---|---|---|---|
| **vs. RHB, Standard (`firstBase=0`)** | 53,775 (91.1% of RHB empty) | 114.1 ft / +29.6° | 152.9 ft / +6.8° | 122.0 ft / −36.4° | 148.3 ft / −18.3° | `batSide=R&firstBase=0&shift=0` |
| **vs. RHB, Runner on 1B / DP (`firstBase=1`)** | 17,792 | 88.0 ft / +40.5° | 148.7 ft / +6.7° | 118.8 ft / −36.5° | 144.8 ft / −17.3° | `batSide=R&firstBase=1&shift=0` |
| **vs. LHB, Standard (`firstBase=0`)** | 19,287 (43.6% of LHB empty) | 123.9 ft / +37.6° | 148.2 ft / +19.1° | 116.1 ft / −27.1° | 152.6 ft / −6.2° | `batSide=L&firstBase=0&shift=0` |
| **vs. LHB, Runner on 1B / DP (`firstBase=1`)** | 6,172 | 88.9 ft / +41.0° | 143.1 ft / +19.0° | 113.8 ft / −27.2° | 148.8 ft / −5.7° | `batSide=L&firstBase=1&shift=0` |

Confidence is **high** on all four sets (`n = 6,172–53,775` league-wide MLB PAs).

### What could not be sourced cleanly (omitted or noted `[E]`)

1. **Infield In / corners in:** Baseball Savant's positioning leaderboard publishes no separate filter for infield-in. Per the [MLB Statcast Shifts glossary](https://www.mlb.com/glossary/statcast/shifts), `Strategic` pools infield-in, guarding the lines, 5-infielder sets, and rovers; `firstBase=27` ("Other") pools 2B-only, 3B-only, 1st&2nd, 1st&3rd, 2nd&3rd, and bases loaded (`RHB n=23,835: 3B 116.7/−36.5°, SS 142.9/−17.8°, 2B 146.1/+6.5°, 1B 108.3/+30.4°`; `LHB n=15,281: 3B 109.7/−30.6°, SS 145.7/−5.5°, 2B 142.0/+19.0°, 1B 116.0/+37.7°`). Infield-in is omitted rather than guessed; custom manager presets remain PLAN §10.1.
2. **Out-count split on `firstBase=1`:** Savant's `firstBase=1` bucket pools 0, 1, and 2 outs (dominated by `< 2` outs where teams hold the runner and look for a double play). Applying `DoublePlay` when `runner_on_first && outs < 2` and reverting to `Standard` at 2 outs is documented `[E]`. Note also that with 1st&2nd or bases loaded a real first baseman does not hold the runner 2 m from the bag; keeping the clean two-state `runner_on_first && outs < 2` rule avoids mixing `firstBase=27`'s RISP/infield-in pool.
3. **`Shaded: Yes` (`shift=1`):** Heavily used against pull-heavy LHB (`n=24,999`, 56.4% of LHB empty: `3B 131.4/−21.9°, SS 153.4/−3.8°, 2B 147.7/+20.3°, 1B 124.8/+37.9°`) and sparingly vs. RHB (`n=5,275`, 8.9%). Awaits per-batter pull-tendency manager shading (PLAN §10.1). Note that even `shift=0` already rotates the whole infield toward the batter's pull side relative to `0°`.

---

## 2. Comparison with previous `home_positions()` and `agent-models.md §5.9`

- **Previous `home_positions()`** ([play.rs](../../sim/crates/bb_game/src/play.rs)) used one fixed symmetric-like table for every hitter and base-out state: `1B 105 ft / +38°`, `2B 150 ft / +13°`, `3B 110 ft / −36°`, `SS 150 ft / −13°`. That parked corners `7°–9°` off the foul lines, pinched middle infielders inside `±13°` (`26°` up the middle), and opened `25°` (`2B–1B`) and `23°` (`3B–SS`) corner holes regardless of batter side or base occupancy.
- **Savant 2024 vs. old layout:**
  - **Batter-side pull rotation:** Against RHB, `SS` slides `5.3°` toward LF (`−13° → −18.3°`), `2B` shades `6.2°` toward the middle (`+13° → +6.8°`), and `1B` shifts `8.4°` off the 1B line (`+38° → +29.6°`), tightening `3B–SS` from `23.0° → 18.1°`. Against LHB, `2B` slides `6.1°` toward RF (`+13° → +19.1°`), `SS` shades `6.8°` toward second (`−13° → −6.2°`), and `3B` shifts `8.9°` off the 3B line (`−36° → −27.1°`), tightening `2B–1B` from `25.0° → 18.5°`. The `SS–2B` middle window stays `25.1°` (RHB) / `25.3°` (LHB), rotated toward the pull side (`−5.7°` / `+6.5°`).
  - **Standard depths:** `1B` backs up `9–19 ft` (`105 → 114.1 / 123.9 ft`) and `3B` backs up `6–12 ft` (`110 → 116.1 / 122.0 ft`); `2B`/`SS` stay at `148–153 ft`.
  - **DP depth (`runner_on_first && outs < 2`):** `1B` holds near the bag (`88.0 / +40.5°` RHB = `2.215 m` from 1B bag; `88.9 / +41.0°` LHB = `1.932 m` from 1B bag), while `2B`, `SS`, and `3B` play `2.3–5.1 ft` shallower.
- **`agent-models.md §5.9`:** Previously described abstract preset data files by handedness (`[data file]`). Updated to cite Savant 2024 directly with the four named alignment tables, the `runner_on_first && outs < 2` rule, and what stays `[E]` or omitted.

---

## 3. Code & test changes

- [`docs/agent-models.md`](../agent-models.md) §5.9 (`6dfe643`): documented the sourced Savant 2024 table, pre-pitch selection rule, and omitted presets.
- [`sim/crates/bb_game/src/play.rs`](../../sim/crates/bb_game/src/play.rs) (`d565cf6`): added `InfieldAlignment`, named constants `SAVANT_INFIELD_{RHB,LHB}_{STANDARD,DP}`, `home_positions_for(bats, runner_on_first, outs)`, kept `home_positions()` as the neutral RHB empty wrapper, and passed batter hand + `(runner_on_first, outs)` in `simulate_play()`, `game::Game::step_pitch()` (`viz.standing` in [`game.rs`](../../sim/crates/bb_game/src/game.rs)), and `live::new_sim()` ([`live.rs`](../../sim/crates/bb_game/src/live.rs)). `P`, `C`, `LF`, `CF`, `RF`, throw-choice logic, and runner logic were left untouched.
- **Hand-worked unit tests:** `savant_standard_positions_match_hand_conversions` and `savant_dp_depth_holds_first_and_shortens_middle_only_before_two_outs` verify every `(x, y)` conversion in meters to `< 1 mm`, pull-side shading order, `1B` bag holding proximity (`< 2.3 m` vs `> 10.5 m`), and 2-out reversion to `Standard`.
- **Legacy fixture preservation / A4 fixture updates:**
  - `live.rs` jump-angle unit tests keep their frozen `symmetric_jump_test_positions()` (`[+38°, +13°, −36°, −13°]`) so hand-worked §5.10 math checks keep testing the anticipation model without rewriting historical constants.
  - In [`tests/throw_choices.rs`](../../sim/crates/bb_game/tests/throw_choices.rs), `1-64` and `17-203` pass unchanged. Frozen `9fdeec1` cases `13-140` (+35.4° LHB chopper that now skips past `1B` holding at `88.9 ft / +41.0°` into shallow RF with 0 holder choices) and `70-157` (102 mph `+11.0°` LHB grounder that now shoots cleanly between `SS −5.7°` and `2B +19.0°` into CF) were replaced by Savant-era captures `23-109` (loaded, 1 out, weighing `carry 1` vs throwing home `[4, 1]`) and `3-140` (tied 6th, 1st&3rd, 1 out, `[2, 1]` DP flipping to `4` at `continuation_weight = 0.0`).

---

## 4. Before vs. After measurements

### Standard 300-game runs (`sim-cli games --n 300`)

Measured on `main` `a791a9e` (before; identical to `27c2494` in `metrics-ledger.md`) and `d565cf6` (after) on both `--seed 0` (`0–299`) and `--seed 1000` (`1000–1299`).

| Metric | Seeds 0–299 (Before → After) | Δ (0–299) | Seeds 1000–1299 (Before → After) | Δ (1000–1299) |
|---|---:|---:|---:|---:|
| **Runs / team-game** | `5.19 → 5.10` | `−0.09` | `5.46 → 5.51` | `+0.05` |
| PA / team-game | `38.5 → 37.8` | `−0.7` | `38.8 → 38.4` | `−0.4` |
| Pitches / game | `224 → 221` | `−3` | `224 → 222` | `−2` |
| K% | `16.2 → 16.2` | `0.0 pp` | `15.8 → 16.0` | `+0.2 pp` |
| BB% | `7.2 → 7.1` | `−0.1 pp` | `7.4 → 7.5` | `+0.1 pp` |
| HBP% | `0.3 → 0.3` | `0.0 pp` | `0.3 → 0.2` | `−0.1 pp` |
| HR / PA | `4.99% → 5.09%` | `+0.10 pp` (`3.84 → 3.85`/g) | `5.22% → 5.47%` | `+0.25 pp` (`4.05 → 4.20`/g) |
| Reached on BIP | `33.5% → 31.6%` | `−1.9 pp` | `32.9% → 32.1%` | `−0.8 pp` |
| **GB (`<10°`) hit** | **`0.348 → 0.318`** | **`−0.030`** | **`0.347 → 0.317`** | **`−0.030`** |
| GB FC / 2+ outs | `0.032 → 0.033` / `0.100 → 0.093` | `+0.001` / `−0.007` | `0.028 → 0.035` / `0.095 → 0.090` | `+0.007` / `−0.005` |
| LD (`10–25°`) hit | `0.723 → 0.706` | `−0.017` | `0.713 → 0.716` | `+0.003` |
| FB (`25–50°`) hit | `0.140 → 0.141` | `+0.001` | `0.130 → 0.147` | `+0.017` |
| PU (`>50°`) hit | `0.023 → 0.026` | `+0.003` | `0.018 → 0.019` | `+0.001` |
| Foul outs / game | `2.09 → 2.05` | `−0.04` | `2.17 → 2.10` | `−0.07` |
| Throws out of play / game | `0.167 → 0.190` | `+0.023` | `0.143 → 0.133` | `−0.010` |

### `run_env_diag` ground-ball & line-double breakdown (`-- 100 0` before/after + `-- 300 0` after)

| Aggregate (`run_env_diag`) | Before 100g (`0..99`) | After 100g (`0..99`) | After 300g (`0..299`) |
|---|---:|---:|---:|
| Total GB (`LA < 10°`, PaResult) | `2,920` | `2,877` | `8,393` |
| Through (never touched by IF) | `653` (`22.4%`) | `640` (`22.2%`) | `1,707` (`20.3%`) |
| Infield hits (IFH) | `295` (`10.1%`) | `259` (`9.0%`) | `764` (`9.1%`) |
| Outs (`2+ outs`) | `1,972` (`67.5%`, `287`) | `1,978` (`68.8%`, `265`) | `5,922` (`70.6%`, `780`) |
| Total Hit% (`Through + IFH`) | `0.325` | `0.312` | `0.294` |
| `classify_bip` GB hit / FC-only | `0.352` / `0.029` | `0.334` / `0.032` | `0.318` / `0.033` |
| GB dives (`Through / IFH / Outs`) | `242` (`26 / 166 / 50`) | `216` (`32 / 129 / 55`) | `634` (`81 / 373 / 180`) |
| Diving-caught IFHs / Outs stretch | `158` / `428` | `115` / `530` | `333` / `1,611` |
| `Through` in `[+13..+38°]` / `[−36..−13°]` | `237 (36.3%)` / `223 (34.2%)` | `141 (22.0%)` / `109 (17.0%)` | `405 (23.7%)` / `292 (17.1%)` |
| `Routine` (`<95 mph, ≤3 m` of `P,1B,2B,3B,SS`) | `92.0%` (`n=921`; Thru 31, IFH 43) | `83.6%` (`n=934`; Thru 80, IFH 73) | `84.2%` (`n=2813`; Thru 224, IFH 221) |

#### Seven-lane GB breakdown (Before `100g 0..99` → After `100g 0..99` / After `300g 0..299`)

| Lane | `ALL` Thru% (Before → After 100g / 300g) | `ALL` Hit% | `RHB` Thru% | `RHB` Hit% | `LHB` Thru% | `LHB` Hit% |
|---|---|---|---|---|---|---|
| `3B line [−45..−35°)` | `6.9% → 3.2% / 4.0%` | `12.6% → 10.8% / 16.2%` | `7.1% → 3.3% / 4.1%` | `12.3% → 10.6% / 16.0%` | `0.0% → 0.0% / 0.0%` | `20.0% → 20.0% / 22.2%` |
| **`3B–SS [−35..−15°)`** | **`35.4% → 16.3% / 15.5%`** | **`53.0% → 30.0% / 29.0%`** | **`33.8% → 13.9% / 12.8%`** | **`52.1% → 27.6% / 26.9%`** | `54.2% → 45.5% / 51.3%` | `62.5% → 59.1% / 58.4%` |
| **`SS–mid [−15..−5°)`** | **`10.1% → 37.2% / 31.8%`** | **`21.0% → 48.0% / 43.9%`** | **`8.8% → 39.3% / 33.9%`** | **`18.9% → 50.2% / 46.6%`** | `15.0% → 29.8% / 24.8%` | `29.0% → 40.4% / 35.1%` |
| `P/mid [−5..+5°)` | `15.7% → 18.6% / 14.8%` | `19.0% → 21.5% / 17.2%` | `13.6% → 14.4% / 13.4%` | `16.1% → 17.1% / 15.9%` | `19.1% → 25.0% / 16.9%` | `23.7% → 28.1% / 19.4%` |
| **`mid–2B [+5..+15°)`** | **`15.8% → 35.6% / 35.0%`** | **`25.6% → 44.5% / 43.6%`** | `26.6% → 34.2% / 34.6%` | `36.1% → 40.8% / 41.5%` | **`9.4% → 36.6% / 35.3%`** | **`19.4% → 47.2% / 45.0%`** |
| **`2B–1B [+15..+35°)`** | **`46.1% → 25.7% / 23.5%`** | **`57.1% → 35.2% / 31.7%`** | `62.1% → 66.7% / 68.6%` | `68.4% → 72.8% / 73.1%` | **`42.0% → 17.3% / 14.3%`** | **`54.3% → 27.5% / 23.2%`** |
| `1B line [+35..+45°]` | `7.9% → 3.3% / 3.4%` | `13.9% → 10.4% / 9.2%` | `13.3% → 6.2% / 5.9%` | `20.0% → 6.2% / 8.8%` | `7.5% → 3.1% / 3.2%` | `13.4% → 10.8% / 9.2%` |

#### `PlayRuling` hit breakdown & line doubles (`run_env_diag`)

| Measure | Before 100g (`0..99`) | After 100g (`0..99`) | After 300g (`0..299`) |
|---|---:|---:|---:|
| BIP / Hits (Hit%) | `5,868 / 2,256 (0.384)` | `5,881 / 2,173 (0.369)` | `17,367 / 6,339 (0.365)` |
| `1B / 2B / 3B / HR` (% of BIP) | `1,181 (20.1%) / 450 (7.7%) / 209 (3.6%) / 416 (7.1%)` | `1,031 (17.5%) / 484 (8.2%) / 253 (4.3%) / 405 (6.9%)` | `2,907 (16.7%) / 1,524 (8.8%) / 728 (4.2%) / 1,180 (6.8%)` |
| **2B down 3B line (`≤−35°` / `≤−30°`)** | `6 (GB 6)` / `14 (GB 9)` | `7 (GB 6)` / `13 (GB 7)` | `19 (GB 17)` / `39 (GB 21)` |
| **2B down 1B line (`≥+35°` / `≥+30°`)** | `3 (GB 3)` / `13 (GB 6)` | `0 (GB 0)` / `14 (GB 4)` | `3 (GB 1)` / `23 (GB 7)` |

---

## 5. Causes of movements and what is left

### What moved as predicted (explained)
1. **`GB hit` fell cleanly by `−0.030` on both 300-game seed sets (`0.348 → 0.318`, `0.347 → 0.317`), `reached on BIP` fell `−1.9 pp` / `−0.8 pp`, and `PA/team-game` fell `−0.7` / `−0.4`:** Pull-side corner gaps collapsed (`3B–SS` `53.0% → 30.0%/29.0%`, `2B–1B` `57.1% → 35.2%/31.7%`; combined `Through` counts in `[+13..+38°]` and `[−36..−13°]` fell `460 → 250` in 100 games), GB dive attempts fell `242 → 216` with diving-caught IFHs falling `158 → 115` and standing stretches on Outs rising `428 → 530`, lowering `IFH` from `10.1% → 9.0%/9.1%`.
2. **Line doubles did not spike:** Moving corners off the lines did not leak line doubles down either line (`3B line ≤−35°` doubles `6 → 7`, `GB 6 → 6`; `1B line ≥+35°` `3 → 0`). RHB pull few GBs inside `−36.4°` vs `−36°` and `3B` backs to `122 ft`; LHB pull few GBs past `1B`'s `+37.6°`/`+41.0°` DP position.
3. **`Routine (<95 mph, ≤3 m)` drop (`92.0% → 83.6% / 84.2%`) is a pitcher-proximity geometry effect, not slower fielders:** `run_env_diag` counts any ball passing `≤ 3 m` of `P` (`±9.3°` at 60 ft) as routine. With `SS` and `2B` previously pinched at `±13°`, they scooped balls sneaking 2–3 m past the pitcher in `0.45 s`; shading `SS` to `−18.3°` (RHB) or `2B` to `+19.1°` (LHB) uncovers `−10..−5°` (RHB `Thru 8.8% → 39.3%/33.9%`) and `+5..+15°` (LHB `Thru 9.4% → 36.6%/35.3%`), turning pitcher-bypassed balls into center-field hits (`s=0 p=184 +8.9°`, `s=1 p=24 +5.9°`, `s=2 p=24 −6.1°`).

### Why runs barely moved (`−0.09` / `+0.05`) vs. the diagnosis's `−0.45 to −0.60 R/G` estimate
1. **Asymmetric Savant vs. symmetric balanced alignment (`−3.0 pp` vs. `−5.4 pp` GB hit):** The diagnosis's offline audit held middle infielders pinched at `±14.5°` alongside `±32°` corners (`−5.4 pp`). Real Savant positioning keeps a `25°` middle window (`SS −18.3° / 2B +6.8°` RHB; `SS −6.2° / 2B +19.1°` LHB) rotated toward the pull side. Because sim batters hit `47.9%` of GBs into `[−15..+15°]`, trading `−210` corner-hole `Through` hits for extra middle `Through` hits (`SS–mid +110`, `mid–2B +88`, `P/mid +18` in 100g) limited `Through` reduction to `22.4% → 20.3%` over 300 games, with `−1.0 pp` more coming from lower `IFH`.
2. **Extra-base composition shift (`1B` down, `2B/3B` slightly up):** Over `0..99`, `1B/BIP` dropped `20.1% → 17.5%` (`16.7%` over 300g), while `2B/BIP` (`7.7% → 8.2%/8.8%`) and `3B/BIP` (`3.6% → 4.3%/4.2%`) rose slightly; balls hit through the wider middle hole into deep CF (`320 ft`) yield more extra bases than corner singles pulled closer to shallow RF/LF (`290 ft`), though sample noise also contributes.
3. **PA-sequence divergence (`not explained` / RNG reshuffling on `--seed 1000`):** Every changed ground-ball out shifts future `play_index` RNG keys (`metrics-ledger.md` §3 arm-by-position). On seeds `1000–1299`, `HR/PA` drifted `+0.25 pp` (`5.22% → 5.47%`, `4.05 → 4.20` HRs/g) and `FB hit` (unchanged outfielders) drifted `.130 → .147` (`+0.017`), adding roughly `+0.2` R/G of seed noise that masked the GB out gain (`5.46 → 5.51`). On seeds `0–299` (`HRs/g` flat `3.84 → 3.85`), runs fell `5.19 → 5.10` (`−0.09`).
4. **Slightly fewer GB double plays (`2+outs`: `0.100 → 0.093` [0], `0.095 → 0.090` [1000]):** Holding `1B` near first and shading middle infielders off the keystone slightly reduces DP conversions (`~−0.3` fewer 2+out GBs/10 games), softening run savings marginally.

### What remains to close `GB hit 0.318 → ~0.24` (`~7.8 pp` gap)
1. **Batted-ball distribution (`+5..+10°` screamers and `<-15°` choppers):** `13.6%` of GBs are `+5..+10°` `100.5 mph` liners (`50.0%` Through in 300g); `42.2%` are `<-15°` topped choppers (`10.7%` IFH). Both await the Rank 1 swing-model update.
2. **Diving IFHs and throw delays:** In 300 games, `373` GB IFHs involve a dive attempt (`333` diving-caught at mean `t_sec = 3.22 s`) and `311` IFHs release before 1B touch (`218` with no 1B secure, `199` thrown to another base).
3. **Per-batter pull shading (`Shaded: Yes`) & pitcher fielding:** Average hitters currently use `shift=0`; adding `shift=1` manager tendencies for pull hitters will tighten the middle vs. high-pull LHB, while pitcher fielding (`21.7%` of GB outs vs `~8%` MLB) remains an active follow-up topic.
