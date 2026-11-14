# Check of the swing v2 note and spec §3.6 (2026-11-14)

Checked at `work/swing-chores` HEAD `014d346` (contains `work/swing-model` at `63f58ab`). Subjects: [`2026-11-14-swing-model.md`](2026-11-14-swing-model.md) and [agent-models §3.6](../agent-models.md) (lines 284–354). Read-only: no build, no Rust run, no source edits. Statcast CSVs are not in the repo, so every Statcast figure below is checked against the repo's own copies (R13 tables, script code), not against raw data.

Abbreviations: "L" = line in the note; "spec" = agent-models §3.6 unless stated; "R13" = [docs/research/r13-foul-direction.md](../research/r13-foul-direction.md).

## 1. Links

- **Markdown links in the note: 4 checked, 0 broken.** L3: `../agent-models.md`, `../physics-spec.md`; L9: `../research/r13-foul-direction.md`; L30: `../agent-models.md`. No anchors are used. The named sections exist: agent-models "### 3.6 Swing model" (line 284), physics-spec "### 6.3 Bat pose" (line 200).
- **Markdown links in spec §3.6 (lines 284–354): 0.** Its plain-text reference to the note's path resolves.
- **Backticked or plain repo paths in the note: 17 checked, 0 broken.** Includes `analysis/oracle/swing_oracle.py`, `analysis/oracle/anticipation_jumps.py`, `analysis/oracle/golden/swing.json` (the note writes `golden/swing.json`, relative to `analysis/oracle/`), `analysis/statcast/`, `sim/crates/bb_agents/src/swing.rs`, `sim/crates/bb_game/src/live.rs`, and the examples `foul_diag`, `platoon_diag`, `run_env_diag`, `foul_in_games`, `spray_by_location`, `spray_by_bat_angle`, `spray_cues`, `bat_read_geometry`, `swing_cage_probe` (all under `sim/crates/bb_game/examples/`).
- Branch names in the note (`work/swing-model`, `bcbb91f`, `63f58ab`) exist. `63f58ab` is an ancestor of HEAD.

## 2. Cited figures against their sources

### 2a. Model parameters

Conversions: 1 m/s = 2.23694 mph; 1 mph = 0.44704 m/s; 1 rad = 57.2958°.

| Quantity | Note | Spec | Code (file:line) | Match? |
|---|---|---|---|---|
| `BatSpeed` at grade 50 | 71.6 mph (L132); **71.5 mph** (L36) | 32.0 m/s = 71.6 mph (§10 table) | `swing.rs:44` `bat_speed: 32.0`; `attributes.rs:67` `linear_from_grade(50, 32.0, 1.1)` = 32.0 | 71.6 yes. L36's 71.5 **no**: 32.0 × 2.23694 = 71.58 mph |
| `BatSpeed` σ_pop | 1.1 m/s (L133) | σ 1.1 (§10) | `attributes.rs:67` (1.1) | yes |
| Between-batter bat-speed SD | 2.8 mph = 1.25 m/s (L133, L21) | none in spec §3.6 | none (data) | conversion 2.8 × 0.44704 = 1.25 yes. Source `analysis/statcast/savant_sd.py` (second block); data not in repo |
| Effort σ | 4.5% (L36) | 4.5% [E] (table, line 342) | `motor.rs:88` `EFFORT_SIGMA = 0.045`; `motor.rs:81` `effort: EFFORT_SIGMA * k` | yes. Derivation 3.2 / 71.7 = 4.46% from `savant_sd.py` (not verifiable: no CSV) |
| Effort RNG purpose | `Purpose::SwingEffort` (14) (L36) | 14 (`docs/data-formats.md` table) | `sim/crates/bb_math/src/rng.rs:41` `SwingEffort = 14` | yes |
| `T_swing` | 0.150 s; Blast 0.149 s (vendor) | 0.150 s (line 306) | `swing.rs:22` `T_SWING_REF = 0.150`; `swing.rs:44` `t_swing: 0.150`; `attributes.rs:68` | yes. Blast 0.149 from R13 L136 |
| Hands' bump width | 55 ms (L35) | 0.055 s (line 307) | `swing.rs:111` `hand_width: 0.055` | yes |
| Bat's bump width | 55 ms (L35) | 0.055 s (line 309) | `swing.rs:112` `bat_width: 0.055` | yes |
| Bat's peak offset | +10 ms (L35) | +0.010 s (line 310) | `swing.rs:113` `bat_peak: 0.010` | yes |
| Hands' rate ratio q | 0.32 (L35) | 0.32 (line 311) | `swing.rs:109` `hand_rate_ratio: 0.32` | yes |
| Hands' peak τ_h | −32 ms (L32) | −0.032 s, derived (line 308) | derived in `swing.rs:435`: 2.145966 × 0.055 − 0.150 = −0.03197 s | yes |
| Lag at contact λ_c | 0 (L35) | 0 [E] (line 312) | `swing.rs:110` `lag_c: 0.0` | yes |
| Contact depth slope k | 0.125 (L33) | 0.125 [E] (line 326) | `swing.rs:115` `contact_k: 0.125` | yes. v1 was 0.6 at `bcbb91f` `swing.rs:87`, as the note says |
| Hand-arc centre x_C | −1.10 m, right-handed (L34) | −1.10 m [E] (line 325) | `swing.rs:106` `center_x: -1.10` | yes |
| ρ range | 0.15–0.46 × height (L34) | 0.15·H to 0.46·H (line 314) | `swing.rs:107–108` | yes |
| Tilt β at aim | Statcast-based (L33) | β₀ 32°, β_h −9.9°, β_x +11°/m (line 319) | `swing.rs:118–120` (32°, −9.9°, 11°) | yes. Check: fitted 35.3 / 32.0 / 28.7° by zone third against the sample's 34.8 / 32.2 / 28.2 (within 0.5°) |
| Attack angle α | Statcast mean 9.0° (§3 MLB column) | α₀ 8°, α_h −8.7°/zone height (line 320) | `swing.rs:116–117` (8°, −8.7°) | parameters match. **Spec's own fit does not reproduce its stated thirds**: fitted 10.9 / 8.0 / 5.1° against the stated 11.9 / 7.8 / 6.1° (1° off at bottom and top) |
| Attack direction ψ | MLB by location, −1.6 to +6.5° (§3) | ψ₀ 0, ψ_x +16.6°/m inside (line 321) | `swing.rs:121–122` (0°, 16.6°) | yes |
| Bat yaw rate | 2,312°/s (L38) | 2,312°/s, 2,747 balls, sd 209 (`agent-models.md:577`, §5.10) | `play.rs:149` `BAT_YAW_RATE_DPS = 2312.0`; v1 1915 at `bcbb91f` `play.rs:149` | yes |
| Bat-read σ at grade 50 | 92° against v1's 77° (L38) | 40 ms timing σ | `play.rs:141` `BAT_READ_TIME_SIGMA_50_S = 0.040`; test `play.rs:770` asserts 92.48°. 2312 × 0.040 = 92.48; 1915 × 0.040 = 76.6 | yes |
| Spray fit, v2 | 6.91 + 0.939 × face, σ 18.0° (L38) | §5.10 (not re-derived) | `live.rs:218–220` (6.91, 0.939, 18.0) | yes. v1 5.2 + 1.36, 18.6° at `bcbb91f` `live.rs:216–217` and in the code comment |
| Lead-face fit, v2 | 184.10 − 1,213.3 × lead, σ 3.1° (L38) | §5.10 (not re-derived) | `live.rs:226–228` (184.10, −1213.3, 3.1) | yes. v1 161.93 / −1063.1 / 3.6 at `bcbb91f` `live.rs:224–226` |
| Batter's typical pull | 5.0 → 1.23° (L38) | — | `live.rs:207` `MEAN_PULL_DEG = 1.23`; v1 5.0 at `bcbb91f` `live.rs:206` | yes |
| Scatter about the location line | 22 → 20.5° (L38) | — | `live.rs:210` `SPRAY_SCATTER_DEG = 20.5`; v1 22.0 at `bcbb91f` `live.rs:209` | yes |
| Pull by location | v1 −6.5 to +7.7 → −2.8 to +0.6° (L38) | — | `live.rs:787–789` `PULL` ends at −2.8 and +0.6; v1 table at `bcbb91f` `live.rs:788` | yes |
| Collision q and e_ref | q = 0.20 tuned (§5, "Tier A") | — | `bb_physics/src/bat.rs:192–193`: comment "q = 0.20 … needs e ≈ 0.442", `e_ref: 0.442` | consistent with the comment; q itself not computed here |
| Hands' speed at contact | "about 14 mph" (spec line 311; not in note) | 14 mph | not in code. Recomputed with the spec's constants, `ss_from_tip` = 0.15 assumed: 14.3 mph | consistent |
| Hands' and bat's turn, start to contact | 76° and 123° (spec line 316; not in note) | same | not in code. Recomputed from the spec's bumps: 75.6° and 122.9° | consistent |
| δ_r reach noise | — | BatControl × 0.7 [E] (line 340) | `motor.rs:81` `reach: 0.7 * b.bat_control * k` | yes, but 0.7 is an unnamed literal (see §3) |
| Plane-tilt noise ratio | — | "fixed ratio to δ_v [E]", **no value given** (line 339) | `motor.rs:81` `tilt_per_m: 1.5` (rad per m of δ_v σ) | **spec has no value**; code value unnamed |

### 2b. Statcast and external figures in the note

Abbreviation: "script" = a file under `analysis/statcast/` that computes the figure. The CSVs are not committed, so script output cannot be reproduced here.

| Quantity | Note value | Source in repo | Match? |
|---|---|---|---|
| Bat speed, all tracked: mean / p10 / p50 / p90 (§3) | 69.9 / 62.3 / 71.2 / 77.5 mph | R13 L128: identical; `savant_bat.py` prints these | matches R13. **Provenance:** note says the six-date sample is the MLB source (§1 text), but these figures equal R13's 12-date numbers exactly |
| Bat speed in §1 text | "69.9 against 69.7" (L9) | R13 L128 = 69.9; 69.7 is not in R13 | ambiguous which figure is which; the six-date mean is not shown anywhere in the note |
| Swing length: mean / p10 / p90 | 7.2 / 6.0 / 8.4 ft (§3) | R13 L132: identical; glossary 7.3 ft (R13 L132) | yes (7.3 is the glossary, not the sample) |
| Attack angle: mean / p10 / p90 | 9.0 / −5 / 23° (§3) | R13 L129: identical | yes (provenance as above) |
| Path tilt: mean / p10 / p90 | 32 / 23 / 41° | R13 L131: identical | yes |
| Path tilt by zone third (bottom / middle / top) | 37 / 32 / 25 (R13); 34.8 / 32.2 / 28.2 (this sample) | R13 L131 for the first; `savant_bat.py` height block for the second (no output in repo) | R13 yes; this-sample not verifiable |
| Attack angle against attack direction | +0.41°/° (r 0.71) (§3) | R13 L135: identical | value matches R13; **no script computes this slope** ("not found" in `analysis/statcast/`) |
| Attack direction against intercept depth | +1.48° per inch out front (r 0.87) (§3) | R13 L134: −1.48° per inch deeper, correlations −0.89 (RHB) and −0.85 (LHB) | magnitude matches; **r 0.87 is not in R13** and no script computes the slope |
| Attack direction, p10 / p50 / p90, pull-positive | −25 / ≈0 / +23° (§3) | R13 L130: identical in pull-positive form | matches R13. Scripts print raw values (negative = pulled, `savant_bat.py`, `savant_sprayad.py`); the sign flip was done by hand, no script does it |
| Fair-contact AD SD, pulled / opposite | 14 / 19° (§3) | R13 by-outcome table (after L139): 14 and 19 | yes |
| Vertical bat angle | −25 to −35° (Blast, vendor) | R13 L137 (Blast blog, "unverified") | yes; vendor source |
| Bat speed by intercept depth (12+ deep 68.3; 6–12 out front 73.4) | §1, §3 | `savant_bat.py` depth block; no output in repo | not verifiable |
| Bat speed by location (MLB 70.1 / 72.1 / 71.4) | §1, §3 | `savant_bat.py` location bins | not verifiable |
| Lateral intercept moves 17 in | §1 | `savant_bat.py` lateral block | not verifiable |
| Within-batter SD, 3.2 mph (216 batters, 7,697 swings) | §1, L36 | `savant_sd.py`, second block | not verifiable; the within-batter pooled SD also printed by the first block (4.3 mph unrestricted) |
| Between-batter SD of means, 2.8 mph | §1, L133 | `savant_sd.py` | not verifiable (includes sampling error, per the script's own note) |
| Exit speed p90 / p99 / max, 105.2 / 111.0 / 116.7 | §5 | `savant_sd.py` in-play line | not verifiable |
| Launch angle mean / SD, 12.2 / 29.1° | §5, §6 | `savant_ev.py` | not verifiable |
| Barrels ≈ 8.7% | §5 | `savant_ev.py` approximate window | yes as "approximate", as the note says |
| HR per BIP, outside third 1.8% | §1 | `savant_ev.py` | not verifiable |
| Spray by location, MLB −12.6 to +0.6° | §7 | `savant_spray.py` | not verifiable |
| Foul share, MLB 51.2% (no caught fouls) and 52.5% (with) | §1, §4 | R13 L34 | yes |
| MLB HR/PA 3.0% (2024) | §5 | **not found in repo**. `calibration-targets.md` B2.5 and B3.3 are different measures | not found |
| MLB K% 22.6, BB% 8.2 | §5 | `calibration-targets.md` B1.1 22.5%, B1.2 8.3% (band, not the same numbers) | **differ by 0.1**; the note's 22.6 / 8.2 not found |
| Bat speed falls 0.9 mph per strike (2024, external) | §7 | **no source in repo**. `savant_sd.py` strikes block would give this sample's 72.3 / 71.7 / 70.3 (not in repo as output) | no source in repo |
| Statcast average bat speed 71.5 mph; swing length 7.3 ft (ESPN, May 2024) | §8, L151 | ESPN: **no source in repo**. 71.5 appears in §10 (agent-models line 702) and as a band in `calibration-targets.md` line 147 (71.5–72 mph). 7.3 ft is in R13 L132 (MLB glossary) | ESPN no source in repo |
| Squared-up formula, 1.23 × bat speed + 0.23 × pitch speed (FanGraphs) | §5, §6 | **no source in repo** | no source in repo |
| Blast time to contact 0.149 s | L151, §2 | R13 L136 (vendor FAQ) | yes |

## 3. Literals that should be named parameters

Scope: the Rust files in the swing diff (`git diff bcbb91f..HEAD --stat`), non-test code. Test modules are excluded (`swing.rs` from line 458, `live.rs` from 3329 and the single `#[cfg(test)] fn` at 858, `play.rs` 294 and 723). `bb_physics` is **not** in the swing diff, and `batter.rs` is not in it either. The examples under `bb_game/examples/` are diagnostic tools and were not scanned beyond their changed hunks. `live.rs` (about 3,300 non-test lines) and `play.rs` were scanned by grep for decimal literals, not read line by line, so the list is a sample for those two files. "Likely" means a model parameter that the spec tags [E] or that is a sourced value; "unsure" means I could not tell.

| file:line | Code | What it seems to be | Confidence |
|---|---|---|---|
| `bb_agents/src/swing.rs:44` (`average_rhb`) | `height: 1.85, bat_speed: 32.0, t_swing: 0.150, bat_control: 0.012, timing: 0.006, adjustability: 0.6` | Grade-50 population values, duplicated from the `attributes.rs:67–68` mapping rather than derived from `BatterGrades::default()`. Two sources of truth | likely (duplication) |
| `bb_agents/src/swing.rs:190` (`guard_tilt`) | `alpha.abs() + DEG` | The spec's "β raised to |α| + 1°" guard, a bare 1° literal in logic | likely |
| `bb_agents/src/swing.rs:123–124` (`Default for SwingTemplate`) | `zone_bottom_frac: 0.27, zone_top_frac: 0.535` | Strike-zone height fractions. Not in spec §3.6; source unknown | unsure |
| `bb_agents/src/swing.rs:103–105` | `pivot_x: -0.95, pivot_y: 0.30, pivot_z_frac: 0.60` | Stance reference, used by `game.rs:305,341`. Named fields, but spec gives no value | unsure |
| `bb_agents/src/motor.rs:81` (`SwingNoise::new`) | `tilt_per_m: 1.5` | Plane-tilt σ per m of δ_v σ (spec: "fixed ratio [E]", no number). A model parameter with no spec value and no name | likely |
| `bb_agents/src/motor.rs:81` (`SwingNoise::new`) | `reach: 0.7 * b.bat_control * k` | Spec's δ_r = 0.7 × BatControl [E], unnamed | likely |
| `bb_agents/src/motor.rs:35` (`ReleaseNoise::new`), pre-existing | `speed: 0.45, spin_rate_rel: 0.02, spin_axis_deg: 3.0, point: 0.02` | Release-noise [E] values, not named constants. Not from the swing work, but in a changed file | likely (pre-existing) |
| `bb_game/src/live.rs:787–789` (`anticipated_pull_deg`, changed hunk) | `const X: [f64; 7] = [-0.30, …]`, `const PULL: [f64; 7] = [-2.8, -0.5, 0.1, 0.6, 0.4, 0.2, -0.7]`, bin width `0.10` at 793–794 | Fitted table from `examples/spray_by_location.rs`, buried in a function body. Should be named data or config | likely |
| `bb_game/src/live.rs` (10 non-test lines, e.g. 936, 939, 943, 1284, 2122, 2259, 2332, 3072, 3087) | `truncated_normal(4.0)` | The truncation literal repeated, instead of `motor::TRUNC_SIGMA` (4.0, spec §1.3). Same value today, but a second source of truth | likely (pre-existing) |
| `bb_game/src/pa.rs:191–194, 407, 425, 427, 442, 446, 447, 875` (11 lines) | `truncated_normal(4.0)` | Same as above | likely (pre-existing) |
| `bb_game/src/play.rs:88–103` (`FielderAttrs::average`, pre-existing) | `speed: 8.2/7.9, tau_a 1.0/0.7, reaction 0.35/0.25, transfer 0.85/0.65, read_sigma0 3.5/0.3, tau_read 0.6, hands 0.018, arm_sigma 1.2°` | Grade-50 fielder defaults; same kind as `average_rhb` above | likely (pre-existing) |
| `bb_game/src/live.rs:1373` | `let dive_extra = REACH + 0.6 * DIVE - REACH * 0.7;` | Dive timing mix, unnamed | unsure |
| `bb_game/src/live.rs:2545` | `let noise = 0.15 * draw;` | Noise scale of a run decision, unnamed | unsure |
| `bb_game/src/live.rs:2207, 2819` | `>= 0.6` against `BAG_REACH_FRAC` (0.9) elsewhere | Bag-reach distance; a second, unnamed value | unsure |
| `bb_game/src/live.rs:1963` | `if d < 4.0` | Chase distance threshold | unsure |
| `bb_game/src/pa.rs:165` | `spin_axis_deg: 15.0, spin_rate_rel: 0.08, efficiency: 0.08, speed: 0.6` | Model-error default for a learned pitch shape | unsure |
| `bb_game/src/pa.rs:34–37` | `[0.48, 0.52, 0.62, 0.40]` etc. | Pitch-mix shares per situation, unnamed | unsure |

Not flagged: `motor.rs:119` (`SPEED_REL` is a named const); `pa.rs:58` `Knobs { perception: 2.7, command: 0.8, swing: 1.0 }` is named fields by design; `swing.rs:19–22` constants are named; `motor.rs:13` `TRUNC_SIGMA` is named.
