# Independent Review: Baseball Savant 2024 Infield Alignment (`work/infield-alignment`)

- **Branch reviewed:** `work/review-infield` at `f3dc029` (merge of `work/infield-alignment`: commits `f333496`, `6dfe643`, `d565cf6`, `306e06a` over `a791a9e`)
- **Scope:** `docs/calibration/2026-11-14-infield-alignment.md`, `docs/metrics-ledger.md`, `docs/agent-models.md` (§5.9), `sim/crates/bb_game/src/play.rs` (`InfieldAlignment`, `SAVANT_INFIELD_*`, `polar_m`, `home_positions_for`), `sim/crates/bb_game/src/game.rs`, `sim/crates/bb_game/src/live.rs`, `sim/crates/bb_game/examples/run_env_diag.rs`, `sim/crates/bb_game/tests/throw_choices.rs`, and `sim/crates/bb_game/tests/fixtures/a4/`.

---

## Findings (by severity)

### P1 — High risk / rule or invariant violation

None. All 16 depth/angle pairs and 4 sample sizes match the live 2024 Baseball Savant positioning endpoint (`shift=0`), the coordinate conversion uses `bb_math::m::sin` and `bb_math::m::cos` (Invariant 5), the pre-pitch selection reads only public pre-pitch state (Invariant 2), and simulation replayability is bit-for-bit deterministic (Invariant 3).

---

### P2 — Medium risk / tactical, physics-coupling, or test-coverage defects

#### P2-1: `InfieldAlignment::select` holds the runner at first (`88.0–88.9 ft / +40.5°–41.0°`) when runners are on first and second or the bases are loaded (`sim/crates/bb_game/src/play.rs:181-190`, `game.rs:300`, `live.rs:2957`)
- **What happens:** `InfieldAlignment::select(bats, runner_on_first, outs)` selects `DoublePlay` (`SAVANT_INFIELD_{RHB,LHB}_DP`, sourced from Savant's `firstBase=1` filter) whenever `runner_on_first && outs < 2`, and both callers (`game.rs:300` and `live.rs:2957`) pass `st.bases[0].is_some()`.
- **Why it is wrong:**
  1. On Baseball Savant (`<select id="ddlRunnerOn1b">`), `firstBase=1` is **"1B Only"** (`100`, second and third empty), whereas `1st & 2nd` (`110`) and `bases loaded` (`111`) belong to `firstBase=27` ("Other", where `1B` stands at `108.3 ft / +30.4°` vs. RHB and `116.0 ft / +37.7°` vs. LHB).
  2. In baseball, when second base is already occupied (`110` or `111`), the runner on first cannot steal second on his own, so the first baseman **never** holds the runner `1.9–2.2 m` from first base (`88.0 ft / +40.5°` or `88.9 ft / +41.0°`). Instead, `1B` plays behind the runner at standard/even depth while `2B`, `SS`, and `3B` play at double-play depth.
  3. Parking `1B` on the bag when `st.bases[1].is_some()` opens a `33.8°` (`27.45 m` chord) right-side hole vs. RHB and a `22.0°` (`21.10 m` chord) hole vs. LHB with multiple runners on base, and leaves `1B` motionless at `26.8–27.1 m` depth through contact (since the sim does not model `1B`'s pre-contact shuffle off the bag after pitch release).
  4. This directly broke the frozen bases-loaded fixture `13-140` (`bases: [6, 5, 4]`, 1 out, LHB), where a `+35.4°` chopper skipped past `1B` parked at `+41.0° / 88.9 ft` into right field (had `1B` been at his `LHB_STANDARD` spot `123.9 ft / +37.6°`, `+35.4°` is `1.4 m` off his line). Conversely, the replacement bases-loaded fixture `23-109` (`bases: [6, 4, 3]`, 1 out, LHB) only offers `carry 1` because `1B` was parked `1.93 m` from first base with the bases loaded.
- **Fix:** Without touching `firstBase=27`, pass `second_occupied: bool` (`st.bases[1].is_some()`) or split `1B`'s spot from the rest of the infield: when `runner_on_first && outs < 2 && second_occupied`, keep `1B` at `SAVANT_INFIELD_{RHB,LHB}_STANDARD[0]` (behind the runner) while placing `2B`, `3B`, and `SS` at `SAVANT_INFIELD_{RHB,LHB}_DP[1..4]`.

#### P2-2: Hardcoded `INFIELD_R = 45.0 m`, `[3usize, 4, 5]` ground-ball break list, and `carry_max_m = 9.0 m` in `live.rs` conflict with Savant infield depths (`sim/crates/bb_game/src/live.rs:178, 1471-1485, 2760, 2806, 2893`)
- **What happens:** Outfielders, pitcher, catcher, and `live.rs` constants were left unchanged, but three places in `live.rs` implicitly assumed the old `home_positions()` geometry (`1B` at `105 ft / +38°` = `5.83 m` from 1B; `2B` and `SS` at `150 ft / ±13°` = `45.72 m`):
  1. **`INFIELD_R = 45.0 m` (`147.64 ft`) vs. Savant `2B`/`SS` depths (`143.1–152.9 ft = 43.62–46.60 m`) in `assign_among` (`live.rs:1471-1485`):**
     - In 6 of the 8 `(2B, SS)` Savant spots (`RHB_STANDARD`: `2B 46.60 m`, `SS 45.20 m`; `LHB_STANDARD`: `2B 45.17 m`, `SS 46.51 m`; `RHB_DP`: `2B 45.32 m`; `LHB_DP`: `SS 45.35 m`), the middle infielder starts **outside `INFIELD_R = 45.0 m`**. When an outfielder (who receives `bias = -0.15 s` even on ground balls at `live.rs:1401`) or another infielder wins `self.primary` on the initial noisy read, the secondary middle infielder's intercept `own[i]` near his starting depth (`45.2–47.5 m`) fails `m::hypot(own[i].x, own[i].y) < INFIELD_R`, so he **does not break for the ground ball (`Role::Chase`)** and instead runs to second base (`Role::Cover(2)`) or stops (`Role::Idle`).
     - Conversely, in `RHB_DP` (`SS` at `144.8 ft = 44.14 m < 45.0 m`) and `LHB_DP` (`2B` at `143.1 ft = 43.62 m < 45.0 m`), the DP pivot man now starts **inside `INFIELD_R = 45.0 m`**. On a grounder where both `2B` and `SS` predict a roll intercept inside `45.0 m`, `assign_among` switches both `3` (`2B`) and `5` (`SS`) to `Role::Chase`, and `cover_roles_after_chasers()` (`live.rs:1819`, whose candidate list for base `2` is only `[3, 5]`) leaves **second base with no cover man at all** while the ball is being fielded.
  2. **`1B` (`i = 2`) omitted from the secondary ground-ball break loop (`live.rs:1471`):**
     - `docs/agent-models.md` §5.7 specifies: *"Every infielder who believes he can reach the ball in the infield breaks for it at once; nobody waits for a call."* Yet `live.rs:1471` loops `for i in [3usize, 4, 5]`, excluding `1B` (`i = 2`).
     - Under the old layout, `1B` stood at `+38°` (`5.83 m` from 1B). Under `RHB_STANDARD`, `1B` stands at `114.1 ft / +29.6°` (`11.07 m` from 1B, deep in the `1B–2B` hole). Whenever `2B` or `RF` wins `self.primary` on a grounder in `[+15°..+35°)`, `1B` is assigned `Role::Cover(1)` and immediately sprints `11.07 m` toward the first-base bag instead of breaking for the grounder (contributing to `RHB Thru%` in `2B–1B [+15°..+35°)` rising from `62.1%` to `68.6%`).
  3. **`settled()` (`live.rs:2893`), no-play return throws (`live.rs:2760`), and `1B` `Standard` distance (`11.1 m`) vs. `carry_max_m = 9.0 m` (`holder.rs:159`, `live.rs:2806`):**
     - Because `settled()` and `holder_act` check `m::hypot(fd.pos.x, fd.pos.y) < INFIELD_R` (`45.0 m`), a middle infielder holding the ball at his normal Savant depth (`45.2–46.6 m`) with all runners stopped cannot end the play until he throws the ball back to second base.
     - In `Standard` alignment, `1B` now starts `11.07 m` (RHB) / `11.14 m` (LHB) from first base (`> carry_max_m = 9.0 m`). When `1B` fields a grounder `9.1–12.0 m` from first base, `holder_options` does not offer a `carry` to first base, and the release gate (`race.direct < race.receiver_at - 0.05` at `live.rs:2806`) forces `1B` to hold the ball waiting for the pitcher (`19.43 m` from 1B, `receiver_ready = 3.24 s` from rest) to cover the bag.

#### P2-3: Swapping fixture `70-157` for `3-140` in `tests/throw_choices.rs` lost coverage of the `option.carry == true` DP continuation branch (`holder.rs:93-99`) and the `0`-out second-out weight (`holder.rs:150`)
- **What happens:**
  - `23-109` (replacing `13-140`) preserves and improves the bases-loaded lead-force vs. `carry 1` test: in `13-140` after `4937cb8`, `1B` was `9.1 m > 9.0 m` from first so `carry 1` was no longer in `first.options`, whereas `23-109` explicitly asserts `carry 1` is in `first.options`, is rejected in favor of throwing home, and records the `[4, 1]` double play.
  - However, `70-157` was a **0-out**, runners-on-1st-and-3rd play where `2B` chose `("carry", 2)` and threw to `1` (`[2, 1]`), whereas its replacement `3-140` is a **1-out**, runners-on-1st-and-3rd play where `2B` chooses `("throw", 2)` and `SS` relays to `1` (`[2, 1]`).
- **Coverage lost:**
  1. In `Sim::force_continuation` (`sim/crates/bb_game/src/live/holder.rs:93-103`), `70-157` tested the `if option.carry` branch (carrier's own post-touch turn/footwork setup clock to throw to first, lines `93-99`) as the winning choice; `3-140` tests the `else` branch (receiver's `RELAY_TRANSFER_FRAC * transfer`, lines `100-103`). No test in `tests/throw_choices.rs` selects a carry-first double play anymore.
  2. In `Sim::holder_options` (`holder.rs:150`, `let w_second = if self.st.outs + 1 >= 2 { 1.0 } else { 0.9 };`), `70-157` had `outs == 0` (`w_second = 0.9`, non-inning-ending DP), whereas both `23-109` and `3-140` have `outs == 1` (`w_second = 1.0`, inning-ending DP).

---

### P3 — Low risk / documentation, spec, and ledger hygiene

#### P3-1: Stale fixture names in `docs/agent-models.md` §5.6, hardcoded `Hand::Right` in `play::simulate_play`, polar-averaging description in the calibration note, and baseline ordering in `docs/metrics-ledger.md`
1. **`docs/agent-models.md:461` (§5.6):** Still references `(seed 13/play 140 fixture: tied 6th, bases loaded, 1 out)` and `(seed 70/play 157 fixture)`, which were deleted in `d565cf6` and replaced by `23-109` and `3-140`. Also, `23-109.json` and `3-140.json` omit the `"source_commit"` provenance field that `1-64.json` and `17-203.json` carry.
2. **`sim/crates/bb_game/src/play.rs:401` (`simulate_play`):** Calls `home_positions_for(bb_observe::Hand::Right, inp.runners.iter().any(|r| r.base == 1), inp.outs)` with `Hand::Right` hardcoded because `PlayInput` has no `bats` field (affecting only the M4 diagnostic skeleton).
3. **`docs/calibration/2026-11-14-infield-alignment.md:15` and `play.rs:194`:** State that the 16 polar pairs are PA-weighted (`n`) means of Savant's `avg_norm_start_distance` and `avg_norm_start_angle` fields. In Savant's JSON, `avg_norm_start_distance` and `avg_norm_start_angle` are rounded to whole integers per player-row (which yields `0.1 ft` or `0.1°` differences on 9 of the 32 values, e.g., `114.2 ft` vs. `114.1 ft` for RHB `1B` and `153.0 ft` vs. `152.9 ft` for RHB `2B`); the constants in `play.rs` were actually computed from the PA-weighted means of Savant's full-precision Cartesian fields `avg_norm_start_pos_x` and `avg_norm_start_pos_y` (`hypot(mean_x, mean_y)` and `atan2(mean_x, mean_y)`), which is more accurate and should be described as such.
4. **`docs/metrics-ledger.md:99-100, 113`:** In §2, `Savant 2024 infield alignment` (`5.10 [5.51]`) is listed after `A4 review fixes` (`c775441`, `5.20 [5.47]`), but `d565cf6` was branched from and measured against `a791a9e` (`5.19 [5.46]`, before `work/a4-review-fixes` merged at `f23fe57`). The stated `−0.09 [+0.05]` run delta is accurate against `a791a9e` (`−0.10 [+0.04]` against `c775441`), and the merged `main` commit `f3dc029` was not re-run.

---

## Answers to the Five Review Questions

### 1. The source: Savant 2024 positioning data, sample sizes, `"shift=0"` and `"firstBase=1"`, angle convention, and frame conversion
- **Live query verification:** Fetched `https://baseballsavant.mlb.com/visuals/position_data?type=player&teamId=&firstBase=<0|1|27>&shift=<0|1>&batSide=<R|L>&season=2024&position=&attempts=1` and the HTML `<select>` definitions on `https://baseballsavant.mlb.com/visuals/fielder-positioning`. All **4 sample sizes** and all **16 depth/angle pairs** in `SAVANT_INFIELD_*` match the PA-weighted (`n`) Cartesian means `(avg_norm_start_pos_x, avg_norm_start_pos_y)` converted to polar `(hypot, atan2)`:

| Alignment (`season=2024, shift=0`) | Savant `sum(n)` | Pos | `play.rs` `(d_ft, deg)` | Savant polar of mean `(x, y)` | Savant mean of row `(x, y)` polar | Savant mean of integer `(d, a)` strings | Sim `(x, y)` m |
|---|---:|---|---|---|---|---|---|
| **RHB Standard** (`firstBase=0`) | `53,775` | `1B` (`3`) | `(114.1, +29.6°)` | `(114.11, +29.57°)` | `(114.15, +29.56°)` | `(114.20, +29.56°)` | `(+17.18, 30.24)` |
| | `53,775` | `2B` (`4`) | `(152.9, +6.8°)` | `(152.86, +6.79°)` | `(152.91, +6.80°)` | `(152.96, +6.80°)` | `(+5.52, 46.28)` |
| | `53,775` | `3B` (`5`) | `(122.0, −36.4°)` | `(121.99, −36.45°)` | `(122.01, −36.44°)` | `(121.98, −36.46°)` | `(−22.07, 29.93)` |
| | `53,775` | `SS` (`6`) | `(148.3, −18.3°)` | `(148.33, −18.33°)` | `(148.36, −18.33°)` | `(148.40, −18.35°)` | `(−14.19, 42.92)` |
| **RHB DP** (`firstBase=1`) | `17,792` | `1B` (`3`) | `(88.0, +40.5°)` | `(87.97, +40.48°)` | `(87.99, +40.49°)` | `(87.99, +40.54°)` | `(+17.42, 20.40)` |
| | `17,792` | `2B` (`4`) | `(148.7, +6.7°)` | `(148.68, +6.71°)` | `(148.72, +6.71°)` | `(148.67, +6.65°)` | `(+5.29, 45.01)` |
| | `17,792` | `3B` (`5`) | `(118.8, −36.5°)` | `(118.78, −36.52°)` | `(118.79, −36.51°)` | `(118.80, −36.53°)` | `(−21.54, 29.11)` |
| | `17,792` | `SS` (`6`) | `(144.8, −17.3°)` | `(144.83, −17.26°)` | `(144.86, −17.27°)` | `(144.89, −17.29°)` | `(−13.12, 42.14)` |
| **LHB Standard** (`firstBase=0`) | `19,287` | `1B` (`3`) | `(123.9, +37.6°)` | `(123.86, +37.57°)` | `(123.88, +37.56°)` | `(123.89, +37.53°)` | `(+23.04, 29.92)` |
| | `19,287` | `2B` (`4`) | `(148.2, +19.1°)` | `(148.23, +19.08°)` | `(148.25, +19.08°)` | `(148.26, +19.06°)` | `(+14.78, 42.68)` |
| | `19,287` | `3B` (`5`) | `(116.1, −27.1°)` | `(116.12, −27.09°)` | `(116.14, −27.10°)` | `(116.16, −27.09°)` | `(−16.12, 31.50)` |
| | `19,287` | `SS` (`6`) | `(152.6, −6.2°)` | `(152.60, −6.22°)` | `(152.65, −6.22°)` | `(152.71, −6.28°)` | `(−5.02, 46.24)` |
| **LHB DP** (`firstBase=1`) | `6,172` | `1B` (`3`) | `(88.9, +41.0°)` | `(88.85, +41.04°)` | `(88.86, +41.05°)` | `(88.85, +41.00°)` | `(+17.78, 20.45)` |
| | `6,172` | `2B` (`4`) | `(143.1, +19.0°)` | `(143.10, +19.00°)` | `(143.15, +19.01°)` | `(143.12, +19.03°)` | `(+14.20, 41.24)` |
| | `6,172` | `3B` (`5`) | `(113.8, −27.2°)` | `(113.81, −27.25°)` | `(113.82, −27.27°)` | `(113.82, −27.27°)` | `(−15.86, 30.85)` |
| | `6,172` | `SS` (`6`) | `(148.8, −5.7°)` | `(148.76, −5.68°)` | `(148.79, −5.69°)` | `(148.77, −5.67°)` | `(−4.50, 45.13)` |

- **What `"shift=0"` and `"firstBase=1"` include:**
  - On Savant's page, `<select id="ddlShift">` has two options, both selected by default: `value="0"` (`Shaded: No`) and `value="1"` (`Shaded: Yes`, i.e., legal post-2023 two-infielders-per-side pull shading under OBR 5.02(c)). Against RHB (`firstBase=0`), `shift=0` covers `53,775 / 59,050 = 91.1%` of PAs (`shift=1` is `5,275` PAs, `8.9%`). Against LHB (`firstBase=0`), `shift=0` covers `19,287 / 44,286 = 43.6%` of PAs, while `shift=1` covers `24,999` PAs (`56.4%`, where `3B` shades to `131.4 ft / −21.9°` and `SS` to `153.4 ft / −3.8°`; pooling `shift=0` + `shift=1` vs. LHB gives `3B 124.8 ft / −24.2°`, `SS 153.1 ft / −4.8°`, `2B 147.9 ft / +19.8°`, `1B 124.4 ft / +37.8°`). Using `shift=0` as the unshaded baseline while deferring `shift=1` to batter-specific manager shading is sound and clearly documented in the calibration note.
  - `<select id="ddlRunnerOn1b">` has three mutually exclusive buckets: `value="0"` (`No Runners On`), `value="1"` (`1B Only` — runner on first, second and third empty, pooling 0, 1, and 2 outs), and `value="27"` (`Other` — all remaining base states: `2B` only, `3B` only, `1st & 2nd`, `1st & 3rd`, `2nd & 3rd`, and `bases loaded`).
- **Angle convention and conversion to the sim frame:**
  - Savant's `avg_norm_start_pos_x` and `avg_norm_start_pos_y` satisfy `x = d * sin(angle)`, `y = d * cos(angle)` in feet from the home-plate apex, with `0°` along `+Y` (through second base toward CF), `+angle` toward `+X` (first-base side, `+45°` foul line), and `−angle` toward `−X` (third-base side, `−45°` foul line).
  - `polar_m(d_ft, deg)` (`play.rs:226-230`) multiplies `d_ft * 0.3048` and computes `Vec3::new(d * m::sin(a), d * m::cos(a), 0.0)` through `bb_math::m`. Both the frame convention and conversion are exact.

### 2. Selection: switch hitters, pre-pitch knowledge, 1st & 2nd / bases loaded, and runner on third
- **Switch hitters:** In `Game::matchup` (`sim/crates/bb_game/src/game.rs:221-222`), `let bats = p.bats.against(throws);` resolves `Bats::Switch` to `Hand::Left` against RHP and `Hand::Right` against LHP before the pitch. Both `viz.standing` (`game.rs:300`) and `LiveInput.bats` (`game.rs:374` → `live.rs:2957`) use `mu.batter.bats`, so the defense aligns for the batter's box the switch hitter actually occupies against this pitcher. (Only the legacy M4 skeleton `play::simulate_play` at `play.rs:401` hardcodes `Hand::Right`; see **P3-1**.)
- **Pre-pitch knowledge (Invariant 2):** `home_positions_for(bats, runner_on_first, outs)` reads only the batter's current plate side, whether first base is occupied before the pitch, and the out count — all public pre-pitch facts.
- **Runners on first and second, or bases loaded:** Holding the runner at first (`88.0–88.9 ft / +40.5°–41.0°`) when second base is occupied is **not right** (see **P2-1**). With second base occupied, `1B` should play behind the runner at `Standard` depth while `2B`, `SS`, and `3B` play at `DoublePlay` depth.
- **Runner on third (`infield in` omitted):** With a runner on third and `< 2` outs, `001`/`011` use `Standard` depth (`2B`/`SS` at `148–153 ft`) and `101`/`111` use `DoublePlay` depth (`2B`/`SS` at `143–149 ft`). On a grounder to a middle infielder at `45–46.5 m`, the ball reaches the glove at `t_c + 1.4–1.8 s` and a throw home arrives around `t_c + 3.4–3.8 s` against a runner from third (`lead_s(3) = 86.80 m`) who decides at `t_c + 0.5 s` and reaches home at `t_c + 4.05 s`; with `1st & 3rd` and `< 2` outs, the holder's option table favors a likely `2 → 1` double play (`value ≈ 1.5–1.9`) over a single tag attempt at home (`weight = 1.3`), conceding the run for two outs as teams do in early/middle innings. Omitting late-game `infield in` (`~95–105 ft`) is explicitly noted in `docs/agent-models.md` §5.9.

### 3. Side effects and the middle-infield ground-ball lanes (`SS–mid` and `mid–2B`)
- **Cut-off and relay positions (`live.rs:1567-1683`):** `relay_spot` is computed from the fielding point `from` and `base_pos(b)`, independent of `home_positions`. `relay_job` still uses a fixed `from.x <= 10.0` split to pick `SS` (`5`) vs. `2B` (`3`) as deep relay man; against RHB (`2B` at `x = +5.52 m`, `SS` at `x = −14.19 m`), a deep ball at `x = +5..+10 m` assigns `SS` even though `2B` starts `20 m` closer to the line.
- **Bag coverage and double-play pivot distance to second (`live.rs:1805-1866, 2884`):**
  - On a **pulled** grounder (`target.x < 0` vs. RHB, `target.x >= 0` vs. LHB), the pivot man (`2B` vs. RHB at `8.16 m` in DP / `9.30 m` in Standard; `SS` vs. LHB at `7.77 m` in DP / `8.98 m` in Standard) starts **`2.5–4.0 m` closer to second base** than under the old symmetric layout (`11.78 m`), cutting `receiver_ready` at second from `2.24 s` to **`1.67–1.89 s`**.
  - On an **opposite-field** grounder (`target.x >= 0` vs. RHB, `target.x < 0` vs. LHB), the pivot man (`SS` vs. RHB at `13.54–14.78 m`; `2B` vs. LHB at `14.41–15.28 m`) starts **`1.8–3.5 m` further from second base**, raising `receiver_ready` from `2.24 s` to **`2.47–2.70 s`**.
  - However, in `RHB_DP` (`SS` at `44.14 m`) and `LHB_DP` (`2B` at `43.62 m`), the pivot man now starts inside `INFIELD_R = 45.0 m` and can be pulled off `Role::Cover(2)` into `Role::Chase` by `assign_among` (`live.rs:1477`; see **P2-2**).
- **First baseman holding the runner and then fielding / covering (`play.rs:202-222`, `holder.rs:159`):**
  - When holding at `DP` depth (`1.93–2.22 m` from 1B, `+40.5°–41.0°`), `1B` stays near the bag until contact (no release-to-contact shuffle off the bag), widening `2B–1B` to `33.8°` (`27.45 m`) vs. RHB and `22.0°` (`21.10 m`) vs. LHB.
  - When playing at `Standard` depth (`11.07–11.14 m` from 1B vs. `5.83 m` previously), `1B` still reaches first base (`receiver_ready = 2.15 s` + `0.18 s` reaction = `2.33 s`) well before routine throws from `SS`/`3B` arrive (`~3.0–3.5 s`), but when `1B` fields a ball near his `Standard` spot (`9.1–12.0 m` from 1B), `distance >= carry_max_m (9.0 m)` prevents an unassisted carry to first and forces him to wait for the pitcher (`19.43 m` away; see **P2-2**).
- **Pop-up priority (`live.rs:1496-1513`):** Decided by tier (outfielder sure call > infielder sure call > unsure call) and `spare` time from current position. Corners `6–19 ft` deeper in `Standard` alignment reach shallow flare pop-ups slightly easier; `PU hit` (`0.023 → 0.026`, `0.018 → 0.019`) and `foul outs / game` (`2.09 → 2.05`, `2.17 → 2.10`) were unaffected.
- **Client's reset staging through the C ABI (`game.rs:300`, `timeline.rs:118-132`, `bb_ffi/src/lib.rs`):** `game.rs:300` (`viz.standing`, used for `t = 0.0` through `t_contact` in `timeline::encode`) and `live.rs:2957` (`new_sim`, used from `t_contact` onward) call `home_positions_for(bats, st.bases[0].is_some(), st.outs)` with identical inputs. Every pitch's `PlayTimeline` stages fielders at their new alignment at `t = 0.0` with no jump or discontinuity at `t_contact`.
- **Why `"SS–mid"` (`10% → 37%` in 100g, `31.8%` in 300g) and `"mid–2B"` (`16% → 36%` in 100g, `35.0%` in 300g) jumped — real pull shift vs. middle under-coverage:**
  - **Both effects are present:**
    1. *Real pull-rotation geometry relative to fixed spray bins:* The total angular width between `SS` and `2B` did **not** widen — it actually narrowed slightly from `26.0°` (`20.57 m`) in the old `±13°` layout to **`25.1°` (`20.00 m`)** vs. RHB (`−18.3°` to `+6.8°`) and **`25.3°` (`20.12 m`)** vs. LHB (`−6.2°` to `+19.1°`). In the old layout, `SS` (`−13°`) stood directly inside `SS–mid [−15°..−5°)` and `2B` (`+13°`) stood directly inside `mid–2B [+5°..+15°)`, while the `SS–2B` midpoint (`0°`) was plugged by the pitcher (`0°`). Rotating the `25°` `SS–2B` window by `~6°` toward the pull side moves `SS` out of `[−15°..−5°)` (vs. RHB) and `2B` out of `[+5°..+15°)` (vs. LHB), placing the `SS–2B` gap midpoint at **`−5.75°` (inside `SS–mid`) vs. RHB** and **`+6.45°` (inside `mid–2B`) vs. LHB**, off the pitcher's `0°` line (`±9.3°` at `3 m` reach).
    2. *Genuine middle under-coverage (`~44%` Hit in `SS–mid` and `mid–2B`, and `.318` overall GB Hit vs. MLB `.240–.248` overall and `~.300–.330` up the middle):* Even accounting for the rotated window, a `.439` / `.436` hit rate in `[−15°..−5°)` and `[+5°..+15°)` (and `.611` / `.681` in the 100-game 15° bins on the pull-rotated side of second base) is higher than MLB's center-corridor ground-ball BA (`~.300–.330`, Statcast). Three factors drive the excess: (a) the `INFIELD_R = 45.0 m` cutoff in `live.rs:1477` prevents a secondary middle infielder at `45.2–46.6 m` from breaking on a ground ball when another fielder wins `self.primary` (**P2-2**); (b) using `shift=0` alone omits Savant's `shift=1` (`Shaded: Yes`) alignment used on `56.4%` of 2024 LHB PAs (where `SS` shades to `−3.8°`, `2.4°` closer to the middle); and (c) `13.6%` of sim ground balls are `LA +5°..+10°` hard liners averaging `100.5 mph` (`50.0%` Through).

### 4. The replaced fixtures in `tests/throw_choices.rs` (`13-140` → `23-109`, `70-157` → `3-140`)
- **`13-140` → `23-109`:** Same tactical intent (bases loaded, 1 out, LHB grounder to `1B`; must reject automatic `carry 1` and take the sure lead force at home for a `[4, 1]` double play), and **stronger coverage** than `13-140` had after `4937cb8` (where `1B` was at `9.1 m > 9.0 m` so `carry 1` was not in `first.options`; `23-109` explicitly asserts `carry 1` is in `first.options`). Note that `23-109` relies on `1B` starting at `DP` depth with the bases loaded (**P2-1**).
- **`70-157` → `3-140`:** Tests the same high-level §5.6 trade-off (runners on 1st and 3rd, `< 2` outs, `[2, 1]` double play beating a single home attempt with `continuation_weight = 1.0` and flipping to `base 4` at `continuation_weight = 0.0`), **but lost coverage of two branches** in `holder.rs` (**P2-3**): `70-157` was a `0`-out (`w_second = 0.9`) `("carry", 2)` → `throw 1` double play (`if option.carry` in `force_continuation`), whereas `3-140` is a `1`-out (`w_second = 1.0`) `("throw", 2)` → `throw 1` double play (`else` branch in `force_continuation`).

### 5. Replayability and the measurement table's arithmetic
- **Replayability (Invariant 3) and `bb_math::m` (Invariant 5):** `InfieldAlignment::select`, `spots`, `polar_m`, and `home_positions_for` are pure functions using static arrays and `bb_math::m::{sin, cos}` with no `HashMap` iteration or RNG draws. Replay determinism is preserved.
- **Measurement table arithmetic (`docs/calibration/2026-11-14-infield-alignment.md` and `docs/metrics-ledger.md`):**
  - Checked all 16 metric rows and deltas in the 300-game table (`0–299` and `1000–1299`), all sums and percentages in the `run_env_diag` GB breakdown (`2,920 = 653 + 295 + 1,972`, `2,877 = 640 + 259 + 1,978`, `8,393 = 1,707 + 764 + 5,922`), the `Routine` rates (`847 / 921 = 92.0%`, `781 / 934 = 83.6%`, `2,368 / 2,813 = 84.2%`), the corner-gap `Through` counts (`237 + 223 = 460` → `141 + 109 = 250`, `Δ = −210`), and the `PlayRuling` hit-type totals (`1,181 + 450 + 209 + 416 = 2,256`, `1,031 + 484 + 253 + 405 = 2,173`, `2,907 + 1,524 + 728 + 1,180 = 6,339`). Every sum, ratio, and delta in the tables is arithmetically exact.
  - Note the minor baseline-ordering mismatch in `docs/metrics-ledger.md` §2 (`a791a9e` vs. `c775441`) documented in **P3-1**.

---

## Verification

- Fetched 2024 MLB fielder positioning data directly from `https://baseballsavant.mlb.com/visuals/position_data` (`firstBase=0,1,27`, `shift=0,1`, `batSide=R,L`, `season=2024`, `attempts=1`) and inspected `<select>` definitions on `https://baseballsavant.mlb.com/visuals/fielder-positioning`; verified all 16 depth/angle pairs, sample sizes (`53,775`, `17,792`, `19,287`, `6,172`), and Cartesian conversions.
- Ran targeted unit and integration tests under the workspace heat rules:
  - `CARGO_BUILD_JOBS=2 taskset -c 0-3 nice cargo test --manifest-path sim/Cargo.toml -p bb_game --lib -- savant` (2 passed)
  - `CARGO_BUILD_JOBS=2 taskset -c 0-3 nice cargo test --manifest-path sim/Cargo.toml -p bb_game --test throw_choices` (4 passed)

---

## Resolution

Open.
