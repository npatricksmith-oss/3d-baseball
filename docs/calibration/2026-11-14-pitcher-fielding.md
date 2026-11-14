# Pitcher fielding calibration: post-delivery follow-through, reaction, and late comebacker hands

- **Branch / base:** `work/pitcher-fielding`, branched from & fast-forwarded to `main` `e991794`
- **Target gap:** The pitcher previously fielded **`20.14%` of infield-fielded grounders** (`15.36%` of all grounders, `21.47%` of GB outs, `19.53%` with a runner on 1st and `< 2` outs) against **`~8.2%–9.2%`** in MLB, turning the middle of the diamond (`[-5..+5°]`) into a `.190` GB hit lane (against `.478`–`.484` beside him) by snagging `252` `95+ mph` comebackers (`56.4%` of his catches) and stretching `1.4 m` across the mound in `< 0.60 s`.

---

## 1. Sourced MLB pitcher fielding rates & biomechanics

### Real-league pitcher ground-ball share (three independent sources)

1. **Michael Baumann, FanGraphs (2 July 2025)** ([article](https://blogs.fangraphs.com/if-junior-caminero-had-any-more-gdps-hed-be-macroeconomics/), Savant/Statcast `2023–Present`, runner on 1st and `< 2` outs):
   - Share of infield grounders (`% of IFGB`) / `GDP%` by first fielder: **`P 8.2%` (`19.6%`)**, `C 0.7%` (`3.8%`), `1B 14.3%` (`21.5%`), `2B 25.4%` (`44.7%`), `3B 23.6%` (`37.8%`), `SS 27.8%` (`54.1%`). Confidence: **high**.
2. **Baseball Savant Full 2024 Regular Season Ground-Ball Search (`2024-03-20` to `2024-10-05`, `N = 53,085` GBs, `53,076` with `hit_location`, `44,260` infield-fielded)** ([Statcast search](https://baseballsavant.mlb.com/statcast_search)):
   - **All grounders (`n = 53,076`):** **`P 4,085 (7.70%)`**, `C 449 (0.85%)`, `1B 6,297 (11.86%)`, `2B 10,983 (20.69%)`, `3B 10,407 (19.61%)`, `SS 12,039 (22.68%)`, `OF 8,816 (16.61%: LF 2,931 / CF 2,650 / RF 3,235)`.
   - **Infield-fielded (`pos 1–6`, `n = 44,260`):** **`P 9.23%`**, `C 1.01%`, `1B 14.23%`, `2B 24.81%`, `3B 23.51%`, `SS 27.20%`.
   - **Runner on 1st, `< 2` outs (`All n = 10,257`, `IF n = 8,318`):** `P 869` (`8.47%` All / **`10.45%` IF**), `C 103`, `1B 1,051`, `2B 2,066`, `3B 2,015`, `SS 2,215`, `OF 1,939`.
   - **Savant 2024 `P` share by EV bin (All GBs and middle `|-6..+6°|`):**
     - `0–60 mph` (`n=4,580`): `P 36.03%` | `Mid n=355`: `P 62.5%, SS 20.3%, 2B 15.5%, OF 0.3%, Hit 9.0%`
     - `60–75 mph` (`n=8,874`): `P 12.14%` | `Mid n=649`: `P 38.8%, SS 32.4%, 2B 24.2%, OF 4.5%, Hit 15.9%`
     - `75–85 mph` (`n=9,021`): `P 4.66%` | `Mid n=538`: `P 19.1%, SS 39.2%, 2B 25.3%, OF 15.8%, Hit 26.8%`
     - `85–95 mph` (`n=12,183`): `P 2.88%` | `Mid n=856`: `P 11.0%, SS 35.5%, 2B 22.7%, OF 29.6%, Hit 39.1%`
     - `95–105 mph` (`n=14,233`): `P 2.77%` | `Mid n=1,380`: `P 8.7%, SS 26.3%, 2B 16.7%, OF 48.2%, Hit 56.4%`
     - `105+ mph` (`n=3,849`): `P 2.60%` | `Mid n=485`: `P 6.0%, SS 18.6%, 2B 14.0%, OF 61.4%, Hit 65.6%`
   Confidence: **high** (`n = 53,076`; note weak topped dribblers/bunts under `75 mph` account for two-thirds of MLB `P` fieldings).
3. **Official MLB Stats API 2024 Regular-Season Fielding (`43,116.3` IP/pos):**
   - **`P`: `PO 2,645, A 3,738, PO+A 6,383, DP 246`** (`9.58%` of `{P,1B,2B,3B,SS}` `39,033` assists; `10.40%` of `{P,2B,3B,SS}` `35,944` non-1B infield assists); `C` `PO 41,891 / A 1,756`, `1B` `36,002 / 3,089 / DP 3,415`, `2B` `7,534 / 11,916 / DP 2,734`, `3B` `3,386 / 8,640 / DP 838`, `SS` `6,374 / 11,650 / DP 2,424`. Confidence: **high**.

### Pitching follow-through & recovery biomechanics

- **Release to deceleration (Fleisig / ASMI biomechanics):** Foot contact to ball release takes `~0.145–0.150 s`; arm acceleration from maximum external rotation to release takes `40–60 ms` (`42–58 ms`); arm deceleration (`release → max internal rotation`) completes ~`50 ms` after release while trunk flexion (`30–45°` forward) and trail-leg swing stabilize the landing over ~`0.40–0.50 s` post-release.
- **Timeline vs. bat contact:** A 90–94 mph pitch arrives at the plate ~`0.40–0.44 s` after release, so much of the delivery follow-through occurs while the ball is in flight, leaving residual landing weight stabilization (`~0.10–0.15 s` past contact) plus `0.30 s` [E] visual-motor reaction (`PITCHER_RECOVER_S = 0.55 s` after contact ≈ `0.95 s` post-release). By `t_contact`, he lands forward (`PITCHER_FOLLOW_THROUGH_FORWARD_M = 1.80 m` ahead of `RUBBER_Y = 18.440 m`, vs Savant 2024 mean release extension `6.47 ft = 1.97 m`), shifted toward his glove side (`0.35 m`, `+0.8 m/s` drift), bent at `PITCHER_RECOVER_MAX_Z = 1.9 m` [E].
- **Reaction & first-base coverage:** `95+ mph` (`≥42.5 m/s`) comebackers arrive at `y = 16.64 m` in `≤ 0.39 s` after contact against `0.25–0.30 s` human visual-motor reaction and `6.0 m/s` [E] hand travel; meanwhile a pitcher covering first (`19.24 m` diagonal) must reach the bag in `~3.5–4.0 s` from contact to beat `4.2–4.4 s` batters on routine `3-1` grounders.

---

## 2. Mechanism diagnosis on `main` (`e991794`)

Extending `run_env_diag` (`Q4`, `100 games`, `0..99`) exposed four physical gaps in how the pitcher previously fielded:

1. **Wrong starting position & instant push-off:** `P` started upright and stationary on the mound (`y = 18.288 m`, `v = 0`) and pushed off at `0.25 s` with infielder sprint speed (`7.9 m/s`) and `τ_a = 0.7 s`. On `≥ 0.90 s` slow grounders (`117` of `447` catches), `P` ran a mean `6.89 m` behind the mound, taking `226 (50.6%)` balls that `SS` or `2B` could reach on their feet (`214` outs in `< 4.35 s`).
2. **Streching through recovery:** `PITCHER_RECOVER_S` (`0.7 s`) previously gated only `DIVE`; `STRETCH` (`1.4 m` wide across the mound!) stayed open during recovery, producing **`296 (66.2%)`** of all pitcher catches, `70.5%` (`315/447`) arriving in **`< 0.60 s`** and `26.2%` (`117/447`) in **`< 0.45 s`**.
3. **Full stationary `1.0 m` / `2.4 m` envelope during follow-through:** A pitcher bent forward with his weight moving had the same instant `1.0 m` horizontal reach and `2.4 m` vertical leap as a balanced shortstop.
4. **Stationary glove accuracy on sub-0.4 s comebackers:** With `a.hands = 1.8 cm`, `globe_sigma_mult` at `45 m/s` (`100 mph`) gave `σ_g ≈ 2.46 cm`, catching **`95%+`** of balls within `1.4 m` (`EV ≥ 95 mph`: `252` = `56.4%` of pitcher catches vs **`12.0%`** in Savant 2024!).

---

## 3. Code & spec changes

All changes are purely physical inputs documented first in [`docs/agent-models.md`](../agent-models.md) (§5.3, §5.4, §5.6, §5.9, §10), with zero outcome rolls, rate targets, or edits to `live/holder.rs`, `runner_decide`, or `move_runners`:

- **[`play.rs`](../../sim/crates/bb_game/src/play.rs):**
  - Added named constants `PITCHER_FOLLOW_THROUGH_FORWARD_M = 1.80` [E], `PITCHER_GLOVE_DRIFT_M = 0.35` [E], `PITCHER_FOLLOW_THROUGH_MPS = 0.8` [E], and `pitcher_follow_through(throws: Hand) -> (Vec3, Vec3)` (`y = 16.640 m`; `+X` glove drift for RHP facing home, `-X` for LHP).
  - Updated `FielderAttrs::average(0)`: `speed = 7.5` [E], `tau_a = 0.80` [E], `reaction = 0.30` [E].
- **[`live.rs`](../../sim/crates/bb_game/src/live.rs):**
  - During delivery recovery (`t < t_contact + PITCHER_RECOVER_S`, `PITCHER_RECOVER_S = 0.55` [E]): `fielder_step` passively brakes follow-through drift at `BRAKE` (`6 m/s²`) with no active chase push-off; `planning_state` (`move_ready_at`) starts intercept/cover running only once recovered; `try_captures` blocks `STRETCH` and `DIVE`, caps overhead reach at `PITCHER_RECOVER_MAX_Z = 1.9 m` [E], and limits standing arm reach via `standing_bounds` (`min(PITCHER_RECOVER_REACH_M = 0.70, PITCHER_GLOVE_BASE_M = 0.25 + HAND_SPEED_MPS * dt_glove)` with `HAND_SPEED_MPS = 6.0 m/s` [E] and `dt_glove = max(0, t - t_c - reaction)`).
  - `batted_glove_sigma_mult`: adds rushed post-reaction tracking `K_LATE_GLOVE * clamp((GLOVE_SETTLE_S - dt_view) / GLOVE_SETTLE_S, 0, 1)` (`GLOVE_SETTLE_S = 0.30 s` [E], `K_LATE_GLOVE = 5.0` [E]) and pitcher recovery imbalance `K_RECOVER_GLOVE * clamp((t_c + PITCHER_RECOVER_S - t) / PITCHER_RECOVER_S, 0, 1)` (`K_RECOVER_GLOVE = 3.5` [E]).
  - `receiver_ready`: judges a covering receiver's bag arrival from visible position, visible velocity along the bag line (`run_time_from`), and `move_ready_at` (fixes an artifact exposed during testing where standing-start estimates grew later tick by tick while `P` accelerated toward first base).
- **Hand-worked unit & integration tests (6 new tests):** `pitcher_follow_through_lands_forward_and_glove_side` (`play.rs`), plus 5 tests in `live.rs` verifying passive follow-through braking & push-off gate, no `STRETCH` during recovery, `HAND_SPEED_MPS` + `0.70 m` + `1.9 m` bounds, hand-worked `batted_glove_sigma_mult` formula, and `3-1` first-base coverage (`pitcher_covers_first_on_grounder_pulled_away_from_first_base`).
  - *Note on intermediate ablation:* With `PITCHER_RECOVER_S = 0.70 s`, `tau_a = 1.0 s`, and standing-start `receiver_ready`, `P` arrived at first base at `~4.56 s` (vs `4.26 s` runner), causing `1B` outs to collapse `218 → 141` (`101` IFHs, `0.0%` GDP) and cutting `0–75 mph` topper shares in half (`P` `4.94%` IF GB). Restoring realistic post-contact completion (`0.55 s` ≈ `0.95 s` post-release), mound `tau_a = 0.80 s`, and visible-velocity `receiver_ready` restored `1B` cleanly (`212/230 = 92.2%` outs, `31.8%` GDP).

---

## 4. Before vs. After measurements

### Ground-ball shares by fielder (`run_env_diag -- 100 0`, seeds `0..99`) vs. MLB

| Pos | Sim Before (`e991794`) All / IF / `R1<2` [GDP%] | Sim After All / IF / `R1<2` [GDP%] | MLB (`Savant 2024` All / IF) | MLB `Baumann` (`R1<2` IF / [GDP%]) |
|---|---|---|---|---|
| **P** | `15.36% / 20.14% / 19.53% [35.8%]` (`n=447`) | **`5.47% / 7.57% / 7.37% [12.0%]`** (`n=165`) | `7.70% / 9.23%` | `8.2%` `[19.6%]` |
| **C** | `0.14% / 0.18% / 0.00%` | `0.10% / 0.14% / 0.29%` | `0.85% / 1.01%` | `0.7%` `[3.8%]` |
| **1B** | `8.07% / 10.59% / 7.00% [37.5%]` | `7.62% / 10.55% / 6.49% [31.8%]` | `11.86% / 14.23%` | `14.3%` `[21.5%]` |
| **2B** | `18.86% / 24.73% / 23.03% [35.4%]` | `22.73% / 31.45% / 31.86% [35.2%]` | `20.69% / 24.81%` | `25.4%` `[44.7%]` |
| **3B** | `9.96% / 13.06% / 15.16% [48.1%]` | `11.03% / 15.27% / 14.45% [53.1%]` | `19.61% / 23.51%` | `23.6%` `[37.8%]` |
| **SS** | `23.87% / 31.31% / 35.28% [54.5%]` | `25.31% / 35.03% / 39.53% [52.2%]` | `22.68% / 27.20%` | `27.8%` `[54.1%]` |
| **OF** | `23.74%` (`LF 4.16 / CF 14.22 / RF 5.36%`) | `27.73%` (`LF 4.41 / CF 18.59 / RF 4.74%`) | `16.61%` | — |

#### Pitcher share by exit-velocity bin (`run_env_diag -- 100 0` vs. Savant 2024)

| EV bin | Sim Before (`All P%` \| `Mid P% / Hit%`) | Sim After (`All P%` \| `Mid P% / Hit%`) | Savant 2024 (`All P%` \| `Mid P% / Hit%`) |
|---|---|---|---|
| `0–60 mph` | `22.55%` \| `100.0% / 18.2%` | `15.61%` \| `75.0% / 8.3%` | `36.03%` \| `62.5% / 9.0%` |
| `60–75 mph` | `7.20%` \| `77.3% / 9.1%` | `5.03%` \| `70.0% / 10.0%` | `12.14%` \| `38.8% / 15.9%` |
| `75–85 mph` | `6.55%` \| `75.9% / 13.8%` | `4.22%` \| `42.9% / 25.7%` | `4.66%` \| `19.1% / 26.8%` |
| `85–95 mph` | `12.58%` \| `68.9% / —` | `4.77%` \| `23.9% / 58.4%` | `2.88%` \| `11.0% / 39.1%` |
| `95–105 mph` | `21.02%` \| `59.6% / —` | **`6.13%` \| `16.7% / 55.2%`** | `2.77%` \| `8.7% / 56.4%` |
| `105+ mph` | `22.58%` \| `56.2% / —` | **`3.04%` \| `7.5% / 46.7%`** | `2.60%` \| `6.0% / 65.6%` |

- **Pitcher deep audit (`447 → 165` catches):** `EV ≥ 95 mph` fell `252 (56.4%) → 54 (32.7%)`, `< 0.45 s` fell `117 → 22`, `< 0.60 s` fell `315 (70.5%) → 59 (35.8%)`, and `≥ 0.90 s` behind `SS|2B` reach fell `226 → 84` (`SS|2B` out `< 4.35 s` `214 → 76`).
- **`P/mid [-5..+5°)` lane:** `Through` rose `15.5% → 36.6%` and `Hit%` rose **`19.0% → 47.6%`** (`SS-mid` `48.6%`, `mid-2B` `48.8%`).
- **`dive_probe` (`100` games, seed `0`):** `3.7` dives/game (`40%` caught); `P` `0.17` dives/game (`12%` caught, all on ground after recovery).

### Standard 300-game runs (`sim-cli games --n 300`, `main` `e991794` → `work/pitcher-fielding`)

| Metric | Seeds `0–299` (Before → After) | Δ (`0–299`) | Seeds `1000–1299` (Before → After) | Δ (`1000–1299`) |
|---|---|---|---|---|
| **Runs / team-game** | `5.83 → 6.79` | **`+0.96`** | `6.21 → 6.94` | **`+0.73`** |
| PA / team-game | `38.5 → 40.1` | `+1.6` | `39.4 → 40.4` | `+1.0` |
| Pitches / game | `224 → 233` | `+9` | `228 → 235` | `+7` |
| K% / BB% / HBP% | `15.9→16.0` / `7.1→7.1` / `0.3→0.3` | `+0.1 / 0.0 / 0.0 pp` | `15.9→16.1` / `7.4→7.4` / `0.2→0.2` | `+0.2 / 0.0 / 0.0 pp` |
| HR / PA | `5.16% → 5.15%` (`3.97 → 4.14`/g) | `−0.01 pp` | `5.34% → 5.22%` (`4.21 → 4.22`/g) | `−0.12 pp` |
| Reached on BIP | `33.7% → 37.5%` | `+3.8 pp` | `34.6% → 37.9%` | `+3.3 pp` |
| **GB (`<10°`) hit** | **`0.327 → 0.397`** | **`+0.070`** | **`0.333 → 0.400`** | **`+0.067`** |
| GB FC / 2+ outs | `0.055→0.056` / `0.055→0.054` | `+0.001 / −0.001` | `0.062→0.058` / `0.057→0.059` | `−0.004 / +0.002` |
| LD (`10–25°`) hit | `0.713 → 0.708` | `−0.005` | `0.712 → 0.722` | `+0.010` |
| FB (`25–50°`) hit | `0.131 → 0.132` | `+0.001` | `0.145 → 0.135` | `−0.010` |
| PU (`>50°`) hit | `0.020 → 0.014` | `−0.006` | `0.019 → 0.024` | `+0.005` |
| Foul outs / Throws OOP | `2.16→2.26` / `0.147→0.150` | `+0.10 / +0.003` | `2.23→2.14` / `0.107→0.143` | `−0.09 / +0.036` |

---

## 5. Causes of movements & open gaps

1. **Why runs rose `+0.96` / `+0.73` (`GB hit +0.070 / +0.067`):** Removing the pitcher's unrealistic `< 0.60 s` vacuum on `95+ mph` middle rockets (`315 → 59` catches, `P` IF GB share `20.1% → 7.6%`) let middle grounders actually get through the infield (`P/mid Hit% 19.0% → 47.6%`, `CF` GB share `14.2% → 18.6%`, `run_env_diag` `Through 21.5% → 24.8%`, `IFH 9.7% → 11.6%`). This exposes another hidden factor in `metrics-ledger.md`: the pitcher previously masked a wide middle infield (`P` catching `~280` extra balls/100 games, most of them hits or balls sneaking past him at `100+ mph` where `sim` batters generate `21%` `105+` GBs vs `7.3%` in Savant 2024 and `CF` plays `320 ft` deep). Nothing was tuned back to suppress runs.
2. **Pitcher `0–75 mph` slow-roller deficit (`15.6%` / `5.0%` vs Savant `36.0%` / `12.1%`):** Two causes remain: (a) the sim has no bunts yet (which inflate Savant weak-EV `P`/`C` counts), and (b) `SS`/`2B` still take slightly higher overall GB shares (`35.0% / 31.5%` vs Savant `27.2% / 24.8%`).
3. **Provisional vs. scorer `2+ outs`:** CLI provisional `GB 2+ outs` remained flat (`0.055→0.054` / `0.057→0.059`), while `run_env_diag` showed `P` `R1<2` GDP% falling `35.8% → 12.0%` (toward Baumann's `19.6%`) alongside `P`'s lower overall volume.
