# Run Environment Diagnosis: Average Players vs. 2024 MLB (2026-11-14)

**Status:** Diagnosis only — **no simulation behavior, physics, specs, `TASKS.md`, or `docs/metrics-ledger.md` were modified.**  
**Scope:** Current `main` baseline (through A3 hurried throws, A1 handedness, A5 dive rate, and positional arm strength, HEAD `8b3f491`) under `bb_game::pa::Knobs::calibrated_m3()` (`perception = 2.7, command = 0.8, swing = 1.0`).

---

## Executive Summary

With grade-50 players across full rosters (`run_env_diag -- 100 0`: 100 games, seeds `0..99`, 7,626 PA, 21,815 pitches, 5,906 fair BIP; cf. 300-game standard in [metrics-ledger.md](../metrics-ledger.md)), the simulation diverges from 2024 MLB on four coupled axes:

| Metric | Sim (100g `0..99`) | Sim Ledger (300g `0..299` / `1000..1299`) | Real MLB (2024 unless noted) | Gap |
|---|---:|---:|---:|---|
| **Runs / team-game** | **5.310** | 5.40 / 5.53 | **4.39** ([Baseball-Reference 2024](https://www.baseball-reference.com/leagues/majors/2024-standard-batting.shtml)) | **+0.92 to +1.14 R/G** |
| **K%** | **15.4%** | 15.3% / 14.8% | **22.58%** ([Baseball Savant 2024](https://baseballsavant.mlb.com/league?nav=hitting&season=2024&view=statcast)) | **−7.2 pp** (despite SwStr% +5 pp high!) |
| **BB%** | **6.9%** | 7.0% / 7.2% | **8.18%** ([Baseball Savant 2024](https://baseballsavant.mlb.com/league?nav=hitting&season=2024&view=statcast)) | **−1.3 pp** |
| **HR / PA** | **5.02%** | 5.14% / 4.97% | **2.99%** ([Baseball Savant 2024](https://baseballsavant.mlb.com/league?nav=hitting&season=2024&view=statcast)) | **+2.03 pp (+68% relative)** |
| **GB Hit Rate** | **0.334** (`0.361` w/ `classify_bip`) | .348 / .353 | **.245** (SP AVG, [MLB Data Warehouse 2024](https://www.mlbdatawarehouse.com/p/sp-luck-analysis-avg-and-slg-by-batted)) | **+8.9 to +11.6 pp** |
| **Platoon K% Gap (Same − Opp)** | **+5.8 pp** (`18.3%` vs `12.5%`) | +5.4 pp (hand. note) | **~1.5 pp** ([Retrosheet 2000–25](https://mlbanalytic.com/articles/platoon-splits-explained.html)) / **1.2–1.9 pp** ([MLBDW](https://www.mlbdatawarehouse.com/p/examining-pitcher-splits)) | **~3–4× real gap** |

Two overarching structural discoveries explain why naive knob retuning will fail unless underlying model mechanisms are addressed first:

1. **The Strikeout-Run Paradox (`Foul/Contact = 21.1%` & `Zone% = 55.6%` vs `SwStr% = 16.0%`):** Batters already swing and miss **more** per pitch (`16.0%` vs MLB `~11.0%`) and take called strikes (`17.6%`) at near-MLB rates. Strikeouts are missing not because contact is too easy, but because **78.9% of contacts go fair into play (`21.1%` foul vs `51.2–52.5%` MLB)** while **pitchers pound the zone (`55.6%` overall, `68.8%` behind in the count)** — terminating **65.8% of PAs before two strikes** (`0 str: 97.2% BIP`, `1 str: 91.1% BIP`) and collapsing `P/PA` to `2.86`. Meanwhile, opposite-handed platoon PAs (`~50%`) strip another `~2.6 pp` off league `K%` relative to the `R/R`-calibrated M3 baseline (`20.6%` in `sim-cli pa`).
2. **Rigid Angular Swing (`ω = 32 rad/s` fixed) + Post-Decision EKF Pursuit Collapse + Infield Alignment:**
   - Because `plan_swing` ([swing.rs](../../sim/crates/bb_agents/src/swing.rs#L257-L285)) sets fixed angular speed `ω = bat_speed / r_nominal` (`32.0 rad/s`), sweet-spot linear speed scales linearly with reach radius `(aim - pivot)` up to **`83.5–84.5 mph` mean / `91.8 mph` p90** on outside pitches (`64.0–65.0 mph` inside). Opposite-handed batters square up those outside pitches (`54.6–55.4%` Outside O-Contact vs `34.2–39.1%` same-handed), turning the outer third into a **12.3–16.5% HR/BIP** launchpad and pushing Fair BIP EV to `90.71 mph`, Hard-Hit% to `41.8%`, and Barrel% to `11.78%` (air density `ρ` and fences are minor factors).
   - Ground-ball hits come primarily from **`67.8%` of through-hits** exploiting wide `23°`/`25°` corner holes in `home_positions()` (`3B −36°, SS −13°, 2B +13°, 1B +38°`), compounded by **49% diving-catch infield hits** (`145/296` with `1.5 s` get-up time, `SS` diving `2.2×` real rate), **un-fieldable `54.0%`-through `+5..+10°` 101 mph low liners**, **late force/throw choices & wild/bounced throws** (`91/126` early-release IFHs threw to non-1B bases for zero outs; routine conversion `89.7%`), and **excess `<-15°` chopped rollers** (`42.8%` of GBs). Home-to-1B sprint time (`4.417 s` exact) is **not** too fast.
   - The `~3–4×` platoon K-gap arises because same-handed `SL/CU` release near the batter's eye plane (`x_rel = −0.720 m`, `eye_x = −0.75 m`) and sweep laterally away (`Bank::h_jac` azimuthal foreshortening cuts `SL Read%` from `66.8–70.0%` down to `48.0–49.5%`), and then **`Perception::angular_sigma(ω)` surges ~26× (`ω = 6.11 rad/s > pursuit_limit 1.2` on `99.9%` of swings) between `t_decide` and `t_cut`**, blinding late swing adjustments (`EstZ_SD` rises `9.0 → 11.2 cm`).

---

## 1. Strikeouts and Walks

### 1.1 Decomposition Across Tools

| Metric | `run_env_diag` (100g, 7,626 PA, mixed hands) | `sim-cli pa` (`2000 PA`, R/R, `M3`) | `platoon_diag` (`2000 PA` R/R, `M3`) | Real MLB (2024 unless noted) | Source & Status |
|---|---:|---:|---:|---|---|
| **K%** | **15.4%** | **20.6%** | **20.9%** | **22.58%** (41,197 / 182,449 PA) | [Baseball Savant 2024](https://baseballsavant.mlb.com/league?nav=hitting&season=2024&view=statcast) (Sourced) |
| **Called-K share of K** | **18.6%** (`219/1,177`) | **19.7%** | **17.5%** | ~25% `[R]` (exact 2024 ratio unsourced here) | [calibration-targets.md](../calibration-targets.md) (`unsourced` exact 2024) |
| **BB%** | **6.9%** | **7.2%** | **8.5%** | **8.18%** (14,929 / 182,449 PA) | [Baseball Savant 2024](https://baseballsavant.mlb.com/league?nav=hitting&season=2024&view=statcast) (Sourced) |
| **HBP%** | ~0.2%–0.3% (resid.) | **0.3%** | ~0.4% | ~1.1% `[R]` | `unsourced` exact 2024 |
| **BIP% (incl. HR)** | **77.4%** | **72.0%** | **70.2%** | **~68%** (1 − K% − BB% − HBP%) | Derived from Savant 2024 |
| **Pitches / PA** | **2.86** | **3.0** | — | ~3.88–3.9 | `calibration-targets.md` `[R]` (`unsourced` exact FG table export) |
| **Zone%** | **55.6%** | **54.7%** | **54.7%** | ~42.6%–43.0% (Statcast/BIS def.) / 48% `[R]` ABS | Web summary FG/MLB 2024; exact table export `unsourced` |
| **First-Pitch Strike%** | **62.5%** (`4,769/7,626`) | **60.9%** | — | ~61%–62% | Web summary FG 2024 (`unsourced` exact point value) |
| **Swing%** | **48.4%** | **47.9%** | **47.9%** | ~47%–48% | `calibration-targets.md` `[R]` |
| **Z-Swing%** | **68.3%** | **67.5%** | **67.8%** | ~68.5%–70.0% | Web summary FG 2024 (`unsourced` exact point value) |
| **O-Swing% (Chase)** | **23.4%** | **24.3%** | **24.0%** | ~29%–31% | Prompt / [calibration-targets.md](../calibration-targets.md) (`29%`); web FG `~31%` |
| **Contact%** | **66.9%** | **64.4%** | **61.9%** | ~76% | `calibration-targets.md` `[R]` (`unsourced` exact 2024) |
| **Z-Contact%** | **72.7%** | **71.3%** | **68.2%** | ~84.5%–85.0% | Web summary FG 2024 (`unsourced` exact point value) |
| **O-Contact%** | **46.0%** | **41.0%** | **40.3%** | ~57% `[R]` (Statcast/FG ~64% BIS) | `calibration-targets.md` `[R]` (`unsourced` exact 2024) |
| **SwStr%** | **16.0%** | **17.1%** | **18.3%** | ~11.0% | `calibration-targets.md` `[R]` (`unsourced` exact 2024) |
| **CStr%** | **17.6%** | — | — | ~16.0% | `unsourced` |
| **Foul / Contact%** | **21.1%** (`128g foul_in_games`: **22.9%**) | **23.3%** | **22.5%** | **52.5%** (w/ caught fouls) / **51.2%** (w/o) | R13 F1 via [foul-measures.md](2026-11-13-foul-measures.md#L33) (Sourced) |

> **Correction on prompt premise ("chase is 11.6% against a real 29%"):** The `10.9–11.6%` chase figure belonged to an earlier interim setting (`K_perception = 2.25`, [2026-10-31-m3-interim.md](2026-10-31-m3-interim.md#L113)). Under current `Knobs::calibrated_m3()` (`K_perception = 2.7`), `O-Swing%` is already **23.4%** in games (`24.3%` in `sim-cli pa`), with non-bounced OOZ chase splitting into **shadow (0–8.5 cm) `35.2%`**, **chase (8.5–25.5 cm) `15.2%`**, and **waste (>25.5 cm) `7.4%`** (vs `sim-cli pa` R/R: `35.9% / 16.1% / 11.7%`, with `43.5%` shadow / `46.8%` chase / `9.6%` waste OOZ shares).

#### Strike-Count, Count-Group, and Pitch-Type Decomposition (`run_env_diag`, `100 games`, `7,626 PA`)

| Cut | PA (% of total) | P | P/PA | K% | BB% | HR% | BIP% | Zone% | Sw% | ZSw% | OSw% | Con% | ZCon% | OCon% | SwStr% | CStr% | Foul/C% |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **0 strikes (end / state)** | 3,181 (**41.7%**) | 11,517 | 3.62 | 0.0 | 2.6 | 6.04 | **97.2** | **59.0** | 45.3 | 65.8 | **15.7** | 70.4 | 74.7 | 44.3 | 13.4 | 20.2 | 20.4 |
| **1 strike (end / state)** | 1,835 (**24.1%**) | 6,614 | 3.60 | 0.0 | 8.3 | 5.50 | **91.1** | 55.7 | 45.1 | 64.7 | 20.5 | 66.8 | 72.8 | 43.0 | 15.0 | 19.7 | 20.9 |
| **2 strikes (end / state)** | 2,610 (**34.2%**) | 3,684 | **1.41** | **45.1** | 11.1 | 3.45 | 43.8 | 44.7 | **64.0** | **86.7** | **45.7** | **59.4** | **65.9** | **49.4** | **26.0** | **5.9** | **23.1** |
| **CG0 behind** | 776 | 2,652 | — | — | — | 6.96 | 99.4 | 48.7 | 55.1 | 81.1 | 30.3 | 62.9 | 70.4 | 44.1 | 20.4 | 9.2 | 20.9 |
| **CG1 even** | 2,646 | 9,784 | — | — | — | 6.42 | 99.7 | 53.8 | 47.2 | 72.5 | 17.7 | 68.0 | 73.2 | 43.2 | 15.1 | 14.8 | 20.4 |
| **CG2 ahead (batter)** | 1,594 | 5,695 | — | — | **14.8** | 4.33 | 84.9 | **68.8** | **37.2** | **50.8** | **7.2** | 75.8 | 77.7 | 45.3 | 9.0 | 33.9 | 20.9 |
| **FourSeam (FF)** | 4,369 | 11,317 | 2.59 | **7.2** | 5.7 | 5.31 | 86.9 | 59.3 | 51.8 | 74.6 | 18.6 | 77.8 | 79.3 | 69.1 | 11.5 | 15.1 | 22.0 |
| **Slider (SL)** | 1,685 | 4,837 | 2.87 | **20.2** | 8.0 | 5.40 | 71.7 | 51.5 | 44.2 | 61.7 | 25.6 | 66.7 | 75.7 | 43.6 | 14.7 | 19.7 | 19.1 |
| **Changeup (CH)** | 671 | 2,866 | 4.27 | **36.7** | 12.4 | 3.58 | 50.7 | 54.9 | 44.8 | 56.3 | 30.8 | **31.5** | **38.7** | **15.6** | **30.7** | 24.0 | 18.0 |
| **Curveball (CU)** | 901 | 2,795 | 3.10 | **30.7** | 6.5 | 4.00 | 62.5 | 48.3 | 45.6 | 63.5 | 28.9 | 52.8 | 63.2 | 31.3 | 21.5 | 17.6 | 21.0 |

*(Note: 1,477 check swings occurred in 21,815 pitches → 1,047 balls, 373 called strikes, 12 swinging Ks, 45 fouls/BIP.)*

### 1.2 Which Mechanisms Account for the Missing Strikeouts & Walks?

1. **Primary K% & P/PA Mechanism — Missing Foul Share (`21.1%` vs `51.2%` MLB, [foul-measures.md](2026-11-13-foul-measures.md)):**
   - **Evidence:** Because only **21.1% of contacts go foul** (vs ~51% MLB), balls put on the bat end the PA in fair play almost immediately: **65.8% of all PAs (`5,016/7,626`) never reach two strikes** (`0 str: 97.2% BIP`, `1 str: 91.1% BIP`), and even once a PA reaches two strikes (`34.2%` of PAs), only **`1.41` pitches** are thrown from two strikes onward because `23.1%` foul/contact fails to extend the battle (`K%` once at 2 strikes is already `45.1%`, with `SwStr% = 26.0%`!). As [foul-measures.md](2026-11-13-foul-measures.md#L31-L85) established, the deficit is concentrated at **low contact (`<25°` LA fouls are `4–7%` in games vs `22–57%` MLB)** and **moderately early/well-timed contact** (`|AD| < 6°` fouls `20%` vs `40%`; `AD 12–20°` fouls `8%` vs `54%` and whiffs `43%` vs `23%`), rooted in the flat/long v1 swing geometry (`swing length 9.5 vs 7.2 ft, plane tilt 23.7° vs 32°`, no lag release, AD SD `±7°` vs `±14–19°`) and `cosmetic` alignment of spray with AD. Meanwhile `values.contact_value` ([batter.rs](../../sim/crates/bb_agents/src/batter.rs#L539)) hardcodes `FOUL_SHARE = 0.5`, overvaluing foul-prone swings compared to reality in the sim!
   - **Confidence:** **High.** Direct accounting across 7,626 game PAs + 7,066 contacts in `foul_in_games`.

2. **Primary BB% & Secondary K% Mechanism — Excessive `Zone%` (`55.6%` vs `42.6–48%`) + Command `0.8` + Count-Target Table:**
   - **Evidence:** `choose_pitch` ([pa.rs](../../sim/crates/bb_game/src/pa.rs#L330-L351)) draws in-zone targets with `p_zone = [0.42, 0.55, 0.72, 0.38]`, while `Knobs::calibrated_m3().command = 0.8` shrinks miss scatter and OOZ non-bounced pitches drop only `0.2%` in the dirt (`bounced = 40/21,815`). Consequently, hitters see **`55.6%` pitches in-zone** (`68.8%` when ahead in the count — `50.8%` Z-Swing / `33.9%` CStr — and **`59.0%` on 0 strikes**). With `P/PA = 2.86` and `Z-Swing = 68.3%`, batters put pitches in play before reaching 3 balls (`BB% = 6.9%`).
   - **Why raising `O-Swing%` alone does NOT fix K%:** Verified in **three** independent experiments:
     1. `sim-cli archetypes --n 1000`: **`Free swinger`** (`O-Sw% 45.3%` vs Average `22.7%`) posts **`K% 18.8%` vs `19.5%`** and **`BB% 2.6%` vs `8.5%`** (`P/PA 2.6 vs 3.1`)!
     2. `platoon_diag` Intervention #14 (`Discipline b = 1.4` on 2,000 R/R PA): raises `O-Sw%` `24.0% → 30.4%`, but **lowers** `K% 20.9% → 19.9%` and crashes `BB% 8.5% → 6.0%`.
     3. Even with `O-Contact = 46%`, chasing an OOZ pitch that goes fair ~79% of the time terminates the count as a weak BIP instead of taking a ball toward a deeper/two-strike count unless `Zone%` falls and `Foul/Contact%` rises.
   - **Confidence:** **High.** Observed monotonically across count splits, `archetypes`, and intervention #14.

3. **Two-Strike Approach — Yes, Behavioral Protection Exists, Mechanically Backwards:**
   - **Evidence:** At two strikes, **`Z-Swing%` leaps from `64.7–65.8%` to `86.7%`** and **`O-Swing%` leaps from `15.7–20.5%` to `45.7%`**, while `CStr%` collapses (`20.2% → 5.9%`, yielding `Called-K share = 18.6%` vs ~25%). This comes solely from `count_value` ([batter.rs](../../sim/crates/bb_agents/src/batter.rs#L510-L530)) where `after_strike(2) = -0.27` vs `after_ball` `+0.08..+0.26` penalizes taking strikes massively, driving down the swing indifference threshold without a separate "Protect" zone distortion.
   - **What is missing:** `swing.rs` and `pa.rs` contain **zero mechanical two-strike adaptation** (no choke-up / shorter swing / wider bat-control priority). In fact, because batters chase `45.7%` of pitcher-count waste/shadow pitches (`Z% = 44.7%` at 2 strikes) with `ω = 32 rad/s` fixed, **in-zone contact rate drops at two strikes (`Z-Contact% 74.7% [0 str] → 72.8% [1 str] → 65.9% [2 str]`)** (`unsourced` exact 2024 MLB two-strike Z-Contact delta, though real MLB batters choke up / trade power for contact).
   - **Confidence:** **High.**

4. **Platoon K-Collapse & Perception Swamping (`sensitivity`):**
   - `sim-cli pa` (all R/R) ALREADY gets `K% = 20.6%` (within ~2 pp of 2024 MLB), whereas full games drop to **`15.4%`** primarily because ~50% opposite-handed PAs average `12.5%` K (discussed in §4).
   - `sim-cli sensitivity --n 1000` shows `K_perception = 2.7` completely swamps bat-execution attributes: `BatControl` (`+0.22 pp Contact/10g`) and `Timing` (`+0.32 pp Contact/10g`) are essentially dead vs `Vision` (`+7.89 pp/10g`), and `SpinRecognition` (`−0.14 pp SwStr/10g`) & `SpinCapacity` (`+0.11 pp SwStr/10g`) are dead against the EKF's angular likelihood. Meanwhile `CH` strikes out **`36.7%`** (`38.7%` Z-Contact, only `21–34%` read right) while `FF` strikes out just **`7.2%`** despite 52–62% usage.
   - **Confidence:** **High.**

**Confirming Experiment (Q1):** Run a paired 3,000-PA `sim-cli pa` / game sweep testing: (a) lowering `p_zone` in `choose_pitch` (`~[0.36, 0.45, 0.62, 0.34]`) + `K_command = 1.0–1.15` to bring `Zone%` from `55.6% → ~45–48%`, paired with (b) `Discipline b` raised toward `~1.2–1.35` (`O-Sw% → ~29–30%`), and measure `BB%`, `P/PA`, `2-strike PA%`, and `K%` before vs after a hand-path/lag foul-share fix (Tier M §3.6).

---

## 2. Home Runs

### 2.1 Measured Distributions & Counterfactuals (`run_env_diag`, `5,906` Fair BIP)

| Metric | Sim (Fair BIP, 100g) | Sim (`AllContact` incl. fouls) | Real MLB (2024 unless noted) | Source & Status |
|---|---:|---:|---|---|
| **HR / PA** | **5.02%** (383 OF + 0 ITP) | — | **2.99%** (5,453 / 182,449) | [Baseball Savant 2024](https://baseballsavant.mlb.com/league?nav=hitting&season=2024&view=statcast) (Sourced) |
| **HR / Fair BIP** | **6.48%** | — | **~4.4%** (`2.99% / 68.1%`) | Derived from Savant 2024 |
| **Exit Velocity (mean / SD)** | **90.71 / 17.14 mph** | **~87.5–88.3 mph** | **~88.5–88.8 mph** (SD ~14–15 `unsourced`) | Savant 2024 web summary / [calibration-targets.md](../calibration-targets.md) `[R]` |
| **EV Percentiles (10/25/50/75/90/95/99/max)** | **67.5 / 80.1 / 91.6 / 102.9 / 112.1 / 117.2 / 125.5 / 135.1** | — | Max MLB ~118–122 mph (`unsourced` exact 2024 quantiles) | Observed `run_env_diag` |
| **Hard-Hit% (EV ≥ 95 mph)** | **41.8%** | **35.7%** (`sim-cli pa`) | **~36%–39%** | Savant 2024 web summary / `[R]` `38%` |
| **≥100 / ≥105 / ≥110 mph shares** | **30.7% / 20.7% / 12.7%** | — | **12.7% ≥110** is extreme (`unsourced` exact 2024 split) | Observed `run_env_diag` |
| **Statcast Barrel% (/ Fair BIP)** | **11.78%** (696/5,906; **9.13% / PA**) | — | **~7.0%–7.5% / BBE** (~5% / PA) | Savant 2024 web summary / `[R]` `7.5%` |
| **HR / Barrel** | **54.9%** (`382/696`; 1 non-Barrel HR) | — | — | `unsourced` |
| **Launch Angle (mean / SD)** | **10.51° / 31.46°** (`q10..q90: -31.4..+51.0°`) | `LA 4–14°, SD 29–32°` | Mean **~12–13°**; SD ~26–28° (`unsourced` exact 2024) | Web summary / `[R]` `12° / 27°` |
| **Sweet-Spot% (`8–32°`)** | **30.3%** (`15–40°`: `28.5%`) | — | **~33%–35%** | Savant 2024 web summary (`unsourced` exact table) |
| **GB / LD / FB / PU shares** | **47.4 / 19.6 / 22.5 / 10.6%** | — | `43 / 21 / 29 / 7%` `[R]` | [calibration-targets.md](../calibration-targets.md) `[R]` |
| **Bat Speed (Fair BIP mean / p50 / p90)** | **74.4 / 73.5 / 87.5 mph** | (Cage all swings `72.8 / 71.7 / 86.7` in [foul-measures](2026-11-13-foul-measures.md)) | **69.9 / 71.2 / 77.5 mph** | R13 F6 via [foul-measures.md](2026-11-13-foul-measures.md) (Sourced) |

#### Where the HRs Actually Live (`run_env_diag`)
- **Exit Speed dominance:** Zero HRs below `95 mph` (`0/3,437`), only `3/657` at `95–100 mph`, `36/591 (6.1%)` at `100–105`, `112/468 (23.9%)` at `105–110`, and **`232/753 (30.8%, mean fence clearance +31.7 ft)` at `≥110 mph` (`60.6%` of all HRs!)**. Even if walls were 4 ft taller (`12 ft`), those `≥110 mph` blasts clear the fence by `+31.7 ft` (`median OF-HR clearance above 8 ft wall = +20.4 ft`, `p10 = +4.4 ft`, `p75 = +36.3 ft`).
- **Launch Angles & Spin:** OF-HRs cluster at `15–20°` (`95`, mean EV `116.2`, spin `3,767 rpm`), `20–25°` (`110`, EV `111.5`, `4,518 rpm`), `25–30°` (`118`, EV `110.9`, `5,623 rpm`), and `30–35°` (`38`, EV `109.4`, **`6,505 rpm`**); `≥40° = 0`. Note also `631` wall-hit non-HRs (`459` outs) and `136/1,978` fly-outs landing within `10 ft` of the wall.

#### Trajectory & Environmental Counterfactuals (`2,081` air candidates; baseline = `388` OF-HRs; **Confidence: High**)

| Test | Value / Change | OF-HRs (Δ%) | Interpretation |
|---|---|---:|---|
| **Baseline (`FlightEnv::default_env`)** | `rho = 1.1938 kg/m³`, `cd0 = 0.3008`, `330/400/330 (8ft)` | **388** (ref) | Nathan `100 mph / 28° / 2000 rpm` carries **409.8 ft** (`410.4 ft` no decay) |
| **Sea-level ISA `rho = 1.225`** | +2.6% air density | **359** (−7.5%) | `1.1938` represents slightly warm/humid summer ball (~3.6 ft more carry vs 406.2 ft at 1.225); minor contributor (`~0.35 pp` HR/PA) |
| **`Calculator` (no spin decay)** | `spin_decay = None` | **379** (−2.3%) | Spin decay isn't artificially boosting HRs |
| **Fence `+10 ft` (`340/410/340`)** | Deeper park | **300** (−22.7%) | Current `Field::default()` (`330/400/330, 8ft`, alley ~`382.5 ft`, p50 HR dist `374.8 ft`) is standard MLB average (Rules §2/Targets §2.2 fixed) |
| **Wall `12 ft` / `16 ft`** | +4 ft / +8 ft height | **349 / 301** (−10% / −22%) | Clearance p50 is `+20.4 ft`; fence isn't cheap |
| **Exit Velo `−2.0 / −4.0 mph`** | Pure collision/bat-speed drop | **310 / 236** (−20% / −39%) | Matches closing `5.0% → ~3.0% HR/PA`! |

### 2.2 Which Mechanisms Actually Cause the HR Surplus (`5.02%` vs `2.99%`)?

1. **Rigid Angular Speed `ω = bat_speed / r_nominal` Inflates Outer-Half Bat Speed to `84–92 mph` (`swing.rs L257–285`) — Dominant Cause:**
   - **Mechanism:** `plan_swing` ([swing.rs](../../sim/crates/bb_agents/src/swing.rs#L271-L278)) computes fixed `ω = b.bat_speed / tpl.r_nominal` (`32.0 / 1.00 = 32.0 rad/s`), regardless of reach `r_knob = (to_aim.norm() - uss).clamp(tpl.r_knob_min, tpl.r_knob_max)` (`0.15` to `0.70` m). Because `pivot` is at `x = ∓0.95 m` (`-0.95` RHB), reaching an **outside pitch** extends `r = r_knob + uss` so linear sweet-spot speed rises from **`64.0–65.0 mph`** inside to **`83.5–84.5 mph` mean / `91.8 mph` p90** outside!
   - **Quantitative Evidence (`platoon_diag` horizontal splits & lever-arm run):**
     - **Inside:** `EV 84.0–85.0 mph`, `BatSpd 64.0–65.0 mph`, **`HR/BIP = 0.0%–0.9%`**!
     - **Middle:** `EV 89.6–90.3 mph`, `BatSpd 72.5–72.9 mph`, **`HR/BIP = 2.7%–5.4%`**!
     - **Outside:** `EV 98.1–99.0 mph`, `BatSpd 83.5–84.5 mph`, **`HR/BIP = 12.3%–16.5%`**!
     - **Counterfactual (`platoon_diag`, 2,000 PA R/R):** Clamping `r_knob_max = 0.28` removes the outer lever stretch (`BatSpd p50/p90: 73.5/86.3 → 67.7/71.6`) and collapses `OF-HR/PA` from **`4.10% → 0.05%`** (`EV 91.2 → 81.4`, `Brl% 10.8% → 0.3%`). Reducing nominal `bat_speed` from `32.0 → 30.5 rad/s` (`−4.7%`) drops `OF-HR/PA` by half (**`4.10% → 2.05%`**, `EV 87.9`, `HH% 33.6%`, `Brl% 6.7%`, `BatSpd p50/p90 70.1/82.3`). Note in `run_env_diag` all HRs averaged **`85.8 mph` bat speed** (`p50 85.2, p90 93.7`) vs Statcast `p90 = 77.5 mph`!
   - **Confidence:** **High.**

2. **Opposite-Handed Platoon Surge (`3.45–4.12%` Same → `6.20–6.60%` Opp HR/PA):**
   - Because opposite-handed hitters read sliders (`66.8–70.0%` vs `48.0–49.5%`) and track outside pitches (`Outside O-Contact 54.6–55.4%` vs `34.2–39.1%`), they barrel the `84 mph`-bat-speed outer third cleanly: **`Brl%` jumps from `9.2% (R/R)` → `13.2% (R/L)` / `14.3% (L/R)`** and **`LA mean` rises from `8.0° (R/R)` → `11.9° (R/L)` / `13.5° (L/R)`**, driving HR/PA from **`3.45%` (`R/R`) to `6.60%` (`R/L`)** in games! Historically ([metrics-ledger.md](../metrics-ledger.md#L109)), enabling handedness alone bumped HR/PA by **`+1.1 pp` (`4.09% → 5.21%`)**.
   - **Confidence:** **High.**

3. **Extreme Batted-Ball Backspin (`5,623–7,943 rpm` at `25–40°` LA) & Excess Fair-BIP Share (`77.4%` vs `68%`):**
   - As [agent-models.md §5.1](../agent-models.md#L357) notes (`"3,300–6,500 rpm ... R3 asks whether its spin is high"`), `run_env_diag` confirms OF-HRs at `25–30°` carry **`5,623 rpm`** and `30–40°` carry **`6,505–7,943 rpm`**, inflating carry on high-launch balls. Simultaneously, `77.4%` of PAs end as fair BIP vs `~68%` in MLB — multiplying total HR opportunities per PA by **`1.14×`** (`6.48%` HR/BIP × `0.774` vs × `0.68` adds **`+0.61 pp`** HR/PA by itself!).
   - **Confidence:** **High** (structural `BIP%` arithmetic & observed spin; **Medium** on exact MLB fly-ball spin fraction absent R3 collision ablation).

**Confirming Experiment (Q2):** Set linear tip/sweet-spot speed scaling (`omega = bat_speed / (r_knob + u_cm)` instead of fixed `32/r_nominal`, or cap `BatSpeed` mean `~30.5 rad/s` / implement §3.6 hand-path lag where outside speed stays `~72–76 mph`) + raise `BIP% → ~68%` (`Zone%/foul`), and measure Fair-BIP EV (`→ ~88.5`), `p90` bat speed (`→ ~77.5`), Barrel% (`→ ~7.5%`), and HR/PA (`→ ~3.0%`).

---

## 3. Ground-Ball Hits

### 3.1 Summary & Home-to-First Verification (`run_env_diag`: `2,797` GBs)

| Metric | Sim (100g, `LA < 10°`) | Real MLB (2024 unless noted) | Source & Status |
|---|---:|---|---|
| **GB Hit Rate (`Through + IFH`)** | **0.334** (`classify_bip`: **0.361**; ledger `.348–.353`) | **.245** (qualifying SP GB AVG; [Targets §6.1](../calibration-targets.md) `.240`; pooled HR-excl. BABIP `unsourced`) | [MLB Data Warehouse 2024](https://www.mlbdatawarehouse.com/p/sp-luck-analysis-avg-and-slg-by-batted) / [sources.md](2026-11-14-sources.md) (Medium confidence) |
| **Decomposition of 2,797 GBs** | **Through: 637 (22.8%) / IFH: 296 (10.6%) / Out: 1,864 (66.6%, 226 DP)** | ~`20%` through / ~`5–6%` IFH `[est]` | Exact 2024 league IFH% `unsourced` |
| **Home-to-1B Time (`n = 3,722`)** | **4.417 s ± 0.000 s** (RHB `4.417`, LHB `4.417`) | **~4.2–4.4 s** competitive (`~4.30–4.35 RHB, ~4.20–4.30 LHB`) | Statcast web summary (`unsourced` exact 2024 table); `4.4 s` `[R]` |
| **Routine GB (`<95 mph, ≤3 m`) Out%** | **89.7%** (`783/873`; 90 fail = `29 Thru, 61 IFH`) | **≥ 96%** target / **≥ 93%** envelope | [calibration-targets.md §4.6](../calibration-targets.md#L191) |
| **SS Dives / Team-Season** | **`0.48 / team-game` (`~78 / 162g`)** (`0.96/g`, `dive_probe`) | **36 / team-season (22% outs)** (`0.44 / both-team game`) | [SIS](https://www.sportsinfosolutions.com/?p=1965) via [2026-11-14-sources.md](2026-11-14-sources.md) (Sourced) |
| **Throws Taken in Air** | **96.2%** (`48.8/g`: set `96.3%`, >2/3 hurried `93.6%`, `throw_probe`) | — | Observed `throw_probe 100 0` |

**Home-to-1B is NOT the bug:** Every grade-50 batter-runner (left or right) reaches 1B in **exactly `4.417 s`**, slightly slower than average MLB runners (though LHB lacks the ~1–2 step box advantage).

### 3.2 Where the Extra ~9–11 pp Ground-Ball Hits Come From

1. **Fixed `home_positions()` Creates Huge Corner Gaps (`67.8%` of All Through Hits) — Biggest Single GB Driver:**
   - **Mechanism:** `home_positions()` ([play.rs](../../sim/crates/bb_game/src/play.rs#L178-L185)) parks `3B` at **`−36°` (`110 ft`)**, `SS` at **`−13°` (`150 ft`)**, `2B` at **`+13°` (`150 ft`)**, and `1B` at **`+38°` (`105 ft`)**. Placing 1B and 3B only `7°` and `9°` off the ±45° foul lines while pinching SS/2B inside `±13°` leaves a **`25°` canyon between 2B and 1B (`[+13°..+38°]`, ~66 ft / 20 m arc)** and a **`23°` canyon between 3B and SS (`[−36°..−13°]`)** — while guarding the foul lines (`−45..−40°`: `16.7%` hit; `+40..+45°`: **`6.7%` hit**) far tighter than real alignments.
   - **Evidence (`run_env_diag` 5° Spray Bins & Offline Reach Audit):**
     - **Hit rate explodes between corner and middle infielders:** `+25..+30°` **75.5%** (`Through 67.9%`), `+20..+25°` **63.5%** (`Thru 52.0%`), `+15..+20°` **52.7%** (`Thru 39.9%`), `−25..−20°` **60.7%** (`Thru 39.3%`), `−30..−25°` **58.5%** (`Thru 34.9%`). Meanwhile right at SS (`−15..−10°`) it is `17.8%` and at 2B (`+10..+15°`) `24.0%`.
     - **Audit of `637` Through GBs:** **`253` (`39.7%`)** went through the **2B–1B gap `[+13..+38°]`** and **`179` (`28.1%`)** went through the **3B–SS gap `[−36..−13°]`** (**67.8% combined**). Evaluating the sim's own `run_time` (`0.25 s` reaction, `7.6 m/s`, `1.0 m` standing reach, no dives) on those exact 637 trajectories shows shifting from `home_positions()` (`±38°/±36°/±13°`) to a balanced **`±32° / ±14.5°`** alignment puts **`230` (`36.1%`)** of those Through balls cleanly in standing reach vs only **`79` (`12.4%`)** at default positions — converting **`+151` outs (`−5.4 pp` GB hit rate!)** with zero physics changes.
   - **Confidence:** **High.** Directly verified geometrically and via trajectory replay.

2. **Infield Hits (`IFH = 296`, `10.6%` of GBs): Excessive Diving Costs (`1.5 s` Get-Up) + Throwing to Non-1B Bases + Choppers Dying in Dirt:**
   - **Sub-mechanism 2a — Diving Kills Converting Plays (`145/296 = 49.0%` of IFHs are Caught Dives):**
     In `dive_probe -- 100 0`, SS (`0.96/g`) and 2B (`0.79/g`) dive **`2.2×` real SIS rates**, with `38.6%` (`1.47/g`) within `< 5 m` of home position and `1.01/g` after `10–25+ m` runs. On GBs, fielders secured **`145` IFHs on dives vs only `44` diving outs** (`23.3%` out conversion on caught dives, plus `21` missed `Through` balls!). Those 145 diving IFHs have mean `t_sec = 3.09 s` (`p50 = 3.80 s`, `p90 = 4.64 s`) and `g2r = 1.10 s` (`DIVE_GETUP_S = 1.5 s + DIVE_FOOTWORK_S = 0.3 s`), forcing **`59` `no_throw` aborts** (`41%`) because `t_sec + 1.8 s + throw > 4.417 s`.
   - **Sub-mechanism 2b — Early-Release IFHs (`126` released `< 4.417 s` before batter touches 1B):**
     - **`91/126` (`72.2%`) threw to ANOTHER BASE (`2B/3B/H`) with `0` outs recorded** (`e.g., [IFH sample] s=1 p=138: EV=100, SS fields cleanly at 1.30s, releases at 2.07s to Home, runner scores at 4.02s`). Combined with `58` bounced throws, `31` late 1B arrivals (`mean throw_fly = 1.45 s` on IFHs vs `0.93 s` on Outs), and `5` throws arriving `≤ t_1B` where 1B wasn't holding the bag / missed, these reflect both the throwing choice defects cataloged in [2026-11-14-throw-choices.md](2026-11-14-throw-choices.md) (e.g., `s=0 p=20`, hurried post-dive throw `0.986` sailing into catcher foul chase `Sec(12.90)`!) and tail wild throws (`[RoutineFail] s=5 p=97`, `s=11 p=30`: `hurry = 0.0`, yet `arm_sigma = 1.2°` sends 1B chasing `(25.7, −1.0)` / `(28.6, 15.2)` at `6.5 s`!).
   - **Sub-mechanism 2c — Excess `<-15°` Steep Choppers Dying Slowly (`42.8%` of GBs, `EV 76.4 mph`, `12.5%` IFH, `3.24` bounces):**
     Look at `[RoutineFail]`: `s=2 p=152 (LA −33.5°, t_sec = 3.58 s)` and `s=5 p=161 (LA −59.5°, t_sec = 4.15 s)` — topped choppers hammer straight into front-of-plate `DIRT`, bleed nearly all kinetic energy on bounce 1–2, and roll to middle infielders at `3.1–4.9 s`! That pushes **`Routine (<95 mph, ≤3 m)` out rate down to `89.7%` (`783/873`, vs target `≥ 96%`)**. Note also [2026-11-13-throwing.md](2026-11-13-throwing.md) already quantified that hurried/off-balance footwork (`AWAY_FOOTWORK_S = 0.2 s, DIVE_FOOTWORK_S = 0.3 s`) alone added `+0.37 R/G` (`GB .337 → .352`).
   - **Confidence:** **High.**

3. **`+5..+10°` Hard Low-Liners (`13.0%` of GBs, `EV 101.0 mph`, `Through 54.0%`, `Out 38.4%`):**
   - **Mechanism:** Under the `<10°` GB definition, `+5..+10°` missiles average **`101.0 mph` EV** (`2.41` bounces, clearing infield depth in `< 0.9 s` against `0.25–0.3 s` reaction + `7 m/s²` turning). Nearly **35%** (`197/365`) of all `+5..+10°` balls go cleanly through for hits (`61.7%` total Hit%). Reducing excess outer-half bat speed (`>105 mph` hot shots) and LA spread directly shrinks both `+5..+10°` screamers and `<-15°` dead choppers.
   - **Confidence:** **High.**

**Confirming Experiment (Q3):** (1) Shift `home_positions()` to `1B/3B = ±32° (110–115 ft), 2B/SS = ±14.5° (150 ft)` + run 100 games to measure direct drop in `Through` (`22.8% → ~16–17%`, expected `−0.45 R/G`); (2) fix A4 non-1B force/stop throw choices + trim excessive late dives (`dive_gains` requiring plausible throw-in-time on grounders, not just `< t_on_feet − 1.5s`); re-check `Routine%` (`89.7% → ≥96%`) and `GB hit%` (`0.334 → ~0.24–0.26`).

---

## 4. The Platoon Gap

### 4.1 Measured Four-Cell Asymmetry vs Real League Bounds

| Source / Cell | Same: `R/R` | Same: `L/L` | Opp: `R/L` | Opp: `L/R` | Same−Opp K% Gap | Source / Year |
|---|---:|---:|---:|---:|---:|---|
| **`run_env_diag` (100g, 7,626 PA)** | K **17.8%** (HR 3.45%) | K **20.1%** (HR 4.12%) | K **12.2%** (HR 6.60%) | K **13.1%** (HR 6.20%) | **+5.8 pp** | Observed (`38.8 / 11.1 / 30.6 / 19.5%` PAs) |
| **`platoon_diag` (`2,000 PA/cell`, seed 3)** | K **20.9%** (SwStr 18.3%) | K **19.1%** (SwStr 16.7%) | K **13.6%** (SwStr 14.9%) | K **14.5%** (SwStr 16.2%) | **+5.93 pp** | Observed `steps/528/output.txt` |
| **MLB (Retrosheet 2000–25, non-switch)** | 20.1% disadv | 20.1% disadv | 18.6% adv | 18.6% adv | **+1.5 pp** | [mlbanalytic.com](https://mlbanalytic.com/articles/platoon-splits-explained.html) (Sourced) |
| **MLB (Pitcher splits, avg sample)** | K **23.5%** | K **24.0%** | K **22.3%** | K **22.1%** | **+1.2 to +1.9 pp** | [MLB Data Warehouse](https://www.mlbdatawarehouse.com/p/examining-pitcher-splits) (Sourced; season unspecified) |
| **Sim Acceptance Target (§5.2)** | — | — | — | — | wOBA **+0.015 to +0.045** (`[R≈0.025]`) | [calibration-targets.md §5.2](../calibration-targets.md#L232) |

Real MLB bounds the platoon strikeout gap to **~1.2–1.9 percentage points** (roughly **1.5 pp**), whereas the sim produces **5.4–5.9 pp** (`~3–4×` real).

### 4.2 Pitch-Type, Horizontal-Third, and Mechanism Isolation (`platoon_diag`)

#### Where the K-Gap Concentrates: **Slider & Curveball on the Outer Half** (`2,000 PA/cell`)

| Breakdown (`R/R` vs `R/L`) | `R/R` (Same) | `R/L` (Opp) | Diff (`R/R − R/L`) | Key Observation |
|---|---:|---:|---:|---|
| **FF (Four-Seam)** | SwStr **14.1%** / OCon **62.8%** / Read **96.9%** | SwStr **11.0%** / OCon **73.6%** / Read **97.4%** | +3.1 pp SwStr / −10.8 pp OCon | Modest gap (`EV 89.4 vs 89.9`) |
| **SL (Slider)** | SwStr **18.2%** / OCon **31.2%** / Read **49.5%** | SwStr **12.6%** / OCon **56.8%** / Read **66.8%** | **+5.6 pp SwStr / −25.6 pp OCon / −17.3 pp Read** | **Huge!** 47% of R/R SLs misread as FF |
| **CU (Curveball)** | SwStr **22.9%** / OCon **30.1%** / Read **39.9%** | SwStr **18.2%** / OCon **39.2%** / Read **54.3%** | **+4.7 pp SwStr / −9.1 pp OCon / −14.4 pp Read** | 47% misread as SL, 10% as FF |
| **CH (Changeup)** | SwStr **29.9%** / OCon **20.0%** / Read **33.0%** | SwStr **30.7%** / OCon **14.7%** / Read **21.6%** | **−0.8 pp SwStr (Reverse!)** / +11.4 pp Read | Reverse vs MLB! Vertical fade hurts R/L more |
| **Inside Third (batter frame)** | SwStr **8.3%** / ZCon 75.2% / **OCon 65.0%** | SwStr **8.1%** / ZCon 79.8% / OCon 57.3% | +0.2 pp SwStr | Symmetric inside! (`AimSD x=6.0 vs 5.5cm`) |
| **Middle Third** | SwStr **21.8%** / ZCon 67.8% / OCon 36.5% | SwStr **19.4%** / ZCon 72.9% / OCon 42.9% | +2.4 pp SwStr | Small difference |
| **Outside Third** | **SwStr 24.1%** / **ZCon 63.3%** / **OCon 34.2%** | **SwStr 16.4%** / **ZCon 72.0%** / **OCon 54.6%** | **+7.7 pp SwStr / −20.4 pp OCon** | **Outer-third collapse!** `AimSD (11.6,15.5cm,19.2ms)` vs `(8.3,12.6cm,17.2ms)` |

*(Symmetric mirror holds cleanly in `L/L` vs `L/R`: `L/L SL Read 48.0%, OCon 29.9%, Outside SwStr 22.4%` vs `L/R SL Read 70.0%, OCon 57.2%, Outside SwStr 16.7%`.)*

#### Exact Mechanism Intervention Table (`2,000 PA/cell`, seed 3)

| Intervention | `R/R` K% | `L/L` K% | `R/L` K% | `L/R` K% | **Gap K%** | **Gap SwStr%** | Same/Opp `ZCon%` | Same/Opp `OCon%` | What It Isolates |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| **0. Baseline (M3)** | 20.9 | 19.1 | 13.6 | 14.5 | **+5.93** | **+1.94** | 70.3 / 73.1 | 41.3 / 50.7 | Full asymmetry |
| **1. Pursuit OFF (`pursuit_limit = 1e6`)** | 15.2 | 14.1 | 11.2 | 11.2 | **+3.45** | **+0.41** | 76.9 / 77.7 | 50.4 / 57.3 | **Cuts 79% of SwStr gap & 42% of K gap!** |
| **2. No sideways break (`tilt 12/6`)** | 17.6 | 16.0 | 16.1 | 16.9 | **+0.32** | **+0.45** | 70.7 / 71.7 | 45.3 / 44.2 | **Removes 95% of K-gap** |
| **3. Centered release (`arm_slot x = 0`)** | 20.2 | 17.6 | 13.9 | 15.0 | **+4.50** | +1.45 | 71.2 / 73.3 | 42.1 / 50.7 | Release offset alone removes ~24% of gap |
| **4. Centered rel + No side break** | 17.1 | 15.2 | 15.4 | 17.0 | **−0.10** | **+0.14** | 71.7 / 71.9 | 45.8 / 44.9 | Total elimination (exact symmetry) |
| **6. Zero Weber (`tau noise = 0`)** | 2.2 | 2.6 | 0.8 | 1.0 | +1.50 | +2.57 | 90.1 / 94.7 | 85.2 / 89.0 | Relative timing error scales absolute Ks |
| **8. Zero spin noise (`sigma_s0 = 0`)** | 21.4 | 19.4 | 14.4 | 15.4 | **+5.50** | +1.78 | 70.1 / 72.7 | 40.3 / 49.3 | `spin_cue` is NOT driving the `Read%` gap |
| **9. Exact scouting (`model_error = None`)** | 11.7 | 11.7 | 7.1 | 8.0 | +4.15 | +0.68 | 80.7 / 81.5 | 48.1 / 53.9 | Scouting amplifies overall SwStr |
| **10. Flat `contact_y` (`contact_k = 0`)** | 19.7 | 18.0 | 12.7 | 14.2 | +5.45 | +1.99 | 71.0 / 73.7 | 41.8 / 51.8 | Minor contributor |
| **12. `K_perception = 1.0` (spec default)** | 1.1 | 1.4 | 1.4 | 1.6 | **−0.25** | −0.52 | 94.7 / 93.7 | 90.0 / 89.4 | Asymmetry blows up super-linearly with `K_perc = 2.7`! |

### 4.3 The Specific Terms Responsible (`agent-models §3`, `batter.rs`, `pitch.rs`, `pa.rs`)

Two specific mathematical terms in `bb_agents` + `bb_physics` combine to triple the platoon gap:

1. **Part A — Azimuthal Foreshortening in `Bank::h_jac` (`batter.rs:290–296`) + `release_point` (`pitch.rs:21–27`) + `eye_point` (`batter.rs:84–89`) + Lateral Break (`pitcher.rs:80–89`):**
   - For RHP vs RHB (`PitcherAttrs::average_rhp()`), `release_point()` sits at `x_rel = −0.720 m` (`−1 * (0.10*h + 0.40*h*cos(45°))`), almost colinear in X with RHB's `eye_point()` at `x_eye = −0.75 m`.
   - `Slider` (`tilt 9.6h`, speed `0.90`, eff `0.19`) and `Curveball` (`tilt 7.3h`) accelerate in `+X` across the plate away from the RHB's eyes. Because `Bank::h_jac` computes `g_az = (d.y / rho², −d.x / rho², 0)` with `d = p − eye`, early-flight observations have small `d.x` (near-radial motion along the eye's line of sight toward the outer corner), whereas **depth/timing uncertainty (`tau`) + late `+X` break** look indistinguishable from a fastball heading outer-third until late flight (`SL Read% = 48.0–49.5%` Same vs `66.8–70.0%` Opp). When sideways break is removed (`#2`), the K-gap vanishes (`+5.93 → +0.32 pp`).

2. **Part B — Post-Decision Smooth-Pursuit Explosion in `Perception::angular_sigma(ω)` (`batter.rs:40–43`, `pa.rs:434–449`) During `t ∈ [t_decide, t_cut]`:**
   - Look at [batter.rs#L40-L43](../../sim/crates/bb_agents/src/batter.rs#L40-L43):
     `r = (omega / pursuit_limit).max(1.0); sigma_ang * r * r` (`pursuit_limit = 1.2 rad/s`, `sigma_ang = 0.001` × `kp = 2.7`).
   - At `t_decide`, `omega_dec` averages only **`0.71–0.74 rad/s` (`> 1.2` on just `0.1%` of swings!)** — so smooth-pursuit degradation barely touches the initial `decide` step!
   - **HOWEVER**, verified in `platoon_diag`'s post-decide trace (`steps/528/output.txt`, `2,000 PA`), `simulate_pitch_ctx` continues feeding the EKF at 100 Hz from `t_decide` (`y ~ 6–7 m`) up to `t_cut = t_c − 60 ms` (`y_cut ≈ 2.88 m > d_blank 1.5 m`). Because `ω = ||dv × v|| / ||dv||²` scales inversely with **distance squared** (`1 / r²`), by `t_cut` angular speed surges to **`ω_cut = 6.11 rad/s`** (`> 1.2` on **`99.9%` of swings!**), **multiplying angular observation noise by `(6.11 / 1.2)² ≈ 26×` right during mid-swing adjustment (`t ∈ [t_decide, t_cut]`)!**
   - Even with `Read%` still broken (`SL Read% 49.9%` R/R vs `66.2%` R/L), setting `pursuit_limit = 1e6` (`#1`) drops post-decide `EstZ_SD` from **`11.2 cm → 9.0 cm`** (`Outside: 12.6 → 10.3 cm`) so R/R batters track down late-breaking sliders (`ZCon 68.2% → 75.7%`, `OCon 40.3% → 49.1%`), erasing **79% of the SwStr platoon gap (`+1.94 → +0.41 pp`)** and **42% of the K-gap (`+5.93 → +3.45 pp`)**.
   - **Residual `+3.45 pp` under Pursuit OFF:** Comes from the initial `SL` commit misread (`SL CommitErr R/R` vs `R/L` + `Adjustability = 0.6 m/s` saturation in `[t_decide, t_cut]` + `Outside` reach/aim SD `14.6 cm` vs `12.2 cm`) multiplied by **`K_perception = 2.7`** (`K_perc = 1.0` yields `−0.25 pp`). Wait: remember [2026-10-31-m3-interim.md](2026-10-31-m3-interim.md#L53) originally calibrated `pursuit_limit = 1.2` (`70°/s`, Bahill & LaRitz) **against `y ≈ 0.3–1.5 m` blackout timing**, whereas with **head/eye rotation (`θ_head` tracking ~`150–250°/s`) + predictive saccade**, real batters don't suffer a `26×` `(6.1/1.2)²` perceptual blackout already at `y = 2.9–5.0 m` (`9.5–16 ft` from the plate)! Note also why `CH` reverses (`R/R 30.4%` vs `R/L 31.8–34.2%`): `arm_side` horizontal run of `CH` moves *toward* an opposite-hand eye line (while fading down), creating larger speed/depth misreads (`CH Read 21%` vs `34%`), partially offsetting — yet overwhelmed by — `SL/CU`'s massive opposite-direction asymmetry!
   - **Confidence:** **High.** Proved causally via both Intervention #1 (`pursuit_off`), #2 (`tilt 12/6`), #8 (`sigma_s0=0`), #12 (`kp=1`), and direct post-decide `ω(t_cut)` tracing.

**Confirming Experiment (Q4):** Run `platoon_diag -- 3000 3` sweeping `pursuit_limit ∈ {1.2, 2.5, 4.0, 6.0}` together with moderate `K_perception` (`~1.8–2.2`) and slightly softer `SL` lateral tilt (`10.0–10.5h`) or realistic `r_rel` arm slot / head-tracking `ω`, targeting a **+1.2 to +1.8 pp K-gap** (`R/R − R/L`) and **`+0.020 to +0.035` wOBA gap** ([calibration-targets.md §5.2](../calibration-targets.md#L232)).

---

## 5. Ranked Proposed Changes by Expected Impact on Runs / Team-Game

Every proposed lever obeys [PLAN.md §13](../../PLAN.md#L359-L370) and [calibration-targets.md §2.2–2.3](../calibration-targets.md#L52-L74): **zero outcome dice rolls or stat overrides**, never tuning Tier A physics (`drag`, `COR`, `park`, `flight`), and fixing mis-specified `[E]` / Tier M models before retuning Tier B noise knobs/population means. *(Expected deltas below reflect isolated single-step magnitude estimates grounded in our counterfactual runs and historical [metrics-ledger.md](../metrics-ledger.md) rows; combined deployment requires joint B1/B2/B3 rebalancing because `Foul%`, `Zone%`, and `H-rate` interact non-linearly.)*

| Rank | Target / Mechanism | Model / Parameter Change (PLAN §13 Tier) | Primary Gap Fixed | Expected Effect on **R/G** | Expected Shifts on **K% / BB% / HR% / GB** | Coupling / Risk Warning |
|---:|---|---|---|---:|---|---|
| **1** | **Foul Share + Fixed-`ω` Outer-Half Bat Speed** ([swing.rs](../../sim/crates/bb_agents/src/swing.rs), [foul-measures §3.6](2026-11-13-foul-measures.md)) | Implement §3.6 hand-path + lag swing template (`~7.2 ft`, `32°` plane) with **calibrated tip speed** (`~71–72 mph`, `p90 ~77.5`, instead of `ω=32 rad/s × r` giving `84–92 mph` outside) & realistic spray spread | Q1 (`Foul/C 21%→~50%`, `K%`, `P/PA`) & **Q2** (`HR/PA 5.0%→~3.0%`, `EV 90.7→88.5`, `Brl% 11.8→~7.5%`) & Q3 (`−15°/+5°` extremes) | **−0.7 to −1.2 R/G** | `HR/PA −1.8 to −2.2 pp`, `Foul/C +20..+30 pp`, `P/PA +0.6..+0.9`, `K% +3..+5 pp` (once paired w/ `Zone%`) | Changes `spray_by_bat_angle`, `LEAD_FACE_*`, and batter-box pivot; refit fielder spray beliefs ([foul-measures §5](2026-11-13-foul-measures.md#L127)). Do NOT tune `K_perception` up until this lands! |
| **2** | **Infield Alignment (`home_positions()`, [play.rs](../../sim/crates/bb_game/src/play.rs#L178-L185))** | Tier B/M positioning (`agent-models §5.9` alignment): balance `1B/3B` from `±38°/−36°` → `~±31–33°` (`110–115 ft`) and `2B/SS` from `±13°` → `~±14–16°` (`148–155 ft`), plus batter-side shade | **Q3** (`67.8%` of `Through` GB hits pass through `23°`/`25°` corner holes) | **−0.45 to −0.60 R/G** | `GB Hit −4 to −6 pp` (offline replay `+151` outs = `−5.4 pp`), `BB%/K%/HR%` unchanged | Trivial, high-confidence, reversible; monitor `3B/1B` line doubles cleanly so foul-line rates don't spike. |
| **3** | **Throw Choices (A4) & Late/Excessive Dives ([live.rs](../../sim/crates/bb_game/src/live.rs))** | Fix [2026-11-14-throw-choices.md](2026-11-14-throw-choices.md) items 1–4 (`91/126` early IFH throws went to non-1B bases for `0` outs + hurried `0.986` airmails); gate `dive_gains` on grounders by **time-to-beat-1B**, not just `1.5 s` get-up vs running | **Q3** (`SS` `2.2×` dives, `145` diving IFHs, `Routine 89.7% → ≥96%`) & runner advancement | **−0.30 to −0.45 R/G** | `GB Hit −3 to −4 pp`, `Routine% +4..+6 pp`, fewer extra bases on off-bag/bounced throws | Remember `STRETCH = 0.4 m` is unvalidated ([dive-rate](2026-11-13-dive-rate.md)); decouple `dive_gains` from artificial late-dive loops before tuning `STRETCH` or `AWAY_FOOTWORK_S` (`+0.37 R/G`). |
| **4** | **Pitcher `Zone%` Target Table & `K_command` ([pa.rs](../../sim/crates/bb_game/src/pa.rs#L330-L351))** | Lower `p_zone` (`[0.42, 0.55, 0.72, 0.38]` → `~[0.36, 0.44, 0.60, 0.32]` + OOZ shadow/chase target tuning) and return `K_command` `0.8 → 0.95–1.10` (Tier B) | **Q1** (`Zone% 55.6% → ~46–48%`, `BB% 6.9% → ~8.2%`, `BIP% 77.4% → ~68%`) | **−0.15 to −0.25 R/G** *(net; higher OBP offset by fewer early-count HRs/BIP!)* | `BB% +1.3..+1.8 pp`, `P/PA +0.3..+0.5`, `BIP% −5..−7 pp`, `HR/PA −0.3..−0.5 pp` | Raising `BB%` alone adds runners, *but* cutting `Zone%` + early-count fastballs shrinks `HR/PA` (`CG0/CG1` are `6.4–7.0% HR`) and pairs with #1 & #5 so `O-Swing` pushes `K%` up instead of losing walks. |
| **5** | **Pursuit Cutoff & Perception Recalibration (`pursuit_limit`, `K_perception`, `Discipline`, `SpinCue`)** | Raise `pursuit_limit` (`1.2 → ~3.5–5.0 rad/s` for head+eye tracking before `1.5 m` blackout), rebalance `K_perception` (`2.7 → ~1.8–2.1`) + `K_swingplane` (`1.0 → ~1.25–1.5`, reviving `BatControl`/`Timing` sensitivity!) + `Discipline b_D` (`0.9 → ~1.25`, Tier B) | **Q4** (`Platoon K gap 5.8 pp → ~1.5 pp`) + **Q1** (`O-Sw% 23.4 → 29%`, `Called-K 19% → 25%`) + **Q2** (curbs `R/L` `6.6%` HR/PA) | **−0.10 to −0.20 R/G** | `Platoon K gap −3.5..−4.2 pp`, `O-Sw% +5..+6 pp`, `R/R vs R/L HR` narrows, `Contact%` shifts toward `~76%` if paired w/ #1 & #4 | Never crank `K_perception` upward alone with `pursuit_limit = 1.2`: it balloons `R/R` vs `R/L` gap and makes `BatControl`/`Timing` dead (`0.22–0.32 pp/10g`). |
| **6** | **Population `BatSpeed` Mean (`32.0 → 30.5–31.0 rad/s` or `71.5–72 mph` mean)** | Tier B population μ adjustment (`calibration-targets §2.3`) **only after** #1 fixes outer-half lever inflation (`r_knob`) | **Q2** (`HR/PA` fine-tuning, `EV` & `Barrel%` centering) | **−0.15 to −0.30 R/G** per 1 mph drop | `−1.5 mph` cut (`32→30.5`) halved `R/R` `OF-HR/PA` (`4.10→2.05%`) and `EV` (`−3.3 mph`) | Applying *before* fixing `r_knob` would leave `Inside` weak (`83.7→80 mph`) while `Outside` stays over-juiced! |
| **7** | **Footwork / Transfer Overlap & `FOUL_SHARE` Value Table** | Re-check [2026-11-13-throwing.md](2026-11-13-throwing.md) `OUTFIELD TRANSFER 0.85 s` vs `AWAY_FOOTWORK_S = 0.2 s` double-count; M6 `SwingW` replaces hardcoded `FOUL_SHARE = 0.5` | **Q3** (`−0.2s` OF throw time reduces close advances/XBH); **Q1** (`b_D`/`Protect` count realism) | **−0.10 to −0.20 R/G** | Modest BIP out/advance squeeze; cleaner two-strike value | Validate transfer times against Statcast exchange studies (`2026-11-14-sources.md`: 2B `1.2 s` DP exchange, Kita `0.640 s`) before editing `[E]` throwing constants. |

---

## Diagnostic Tools & Reproduction

All checks above are reproducible without touching simulation source via:
- `taskset -c 0-3 nice cargo run --manifest-path sim/Cargo.toml --release -p bb_game --example run_env_diag -- 100 0` ([run_env_diag.rs](../../sim/crates/bb_game/examples/run_env_diag.rs))
- `taskset -c 0-3 nice cargo run --manifest-path sim/Cargo.toml --release -p bb_game --example platoon_diag -- 2000 3` ([platoon_diag.rs](../../sim/crates/bb_game/examples/platoon_diag.rs))
- `taskset -c 0-3 nice sim/target/release/sim-cli pa --n 2000 --k-perception 2.7 --k-command 0.8`
- `taskset -c 0-3 nice sim/target/release/sim-cli sensitivity --n 1000`
- `taskset -c 0-3 nice sim/target/release/sim-cli archetypes --n 1000`
- `taskset -c 0-3 nice cargo run --manifest-path sim/Cargo.toml --release -p bb_game --example foul_in_games -- 100 0`
- `taskset -c 0-3 nice cargo run --manifest-path sim/Cargo.toml --release -p bb_game --example throw_probe -- 100 0`
- `taskset -c 0-3 nice cargo run --manifest-path sim/Cargo.toml --release -p bb_game --example dive_probe -- 100 0`
