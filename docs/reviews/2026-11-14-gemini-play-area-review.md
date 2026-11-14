# Independent Review: A10 — Limits of the Play Area (`52d6632...dbcf92c`)

**Reviewer:** Gemini  
**Date:** 2026-11-14  
**Branch / Commit:** `work/a10-play-area` at `dbcf92c` (compared against `52d6632`)

---

## Summary of Answers to the Six Review Questions

1. **Rules (`bb_rules` & `scenarios/awards/`):** The core structure of `BallReleasedFromGlove`, `ThrowAward`, and `throw_out_of_play` (fixing the award at release, distinguishing an infielder's first play from subsequent plays, sorting lead-runner-first, and blocking trailing runners when a preceding runner does not score) matches OBR 5.06(b)(4)(G). However, there are three P1 edge-case bugs in `bb_rules` (walk-off awards emitting `RunnerAdvanced { to: 4 }` for trailing runners after `st.walk_off` is already true without `RunScored`; `r.last_base > r.start_base` ignoring runner retreats and early-tag-up retouch obligations; and a caught fair fly that leaves the field without a throw being ruled a home run or ground-rule double), plus a missing scenario file that both the commit message and `docs/rules-spec.md` §12.3 claim exists (`fair deflected fly out over foul ground`).
2. **Physics (`bb_physics/src/ground.rs`, `flight.rs`, `bb_game/src/live.rs`):** All new transcendental calls route through `bb_math::m`, and all reference-park dimensions are named `Field` parameters with `[S]` or `[E]` tags in `docs/physics-spec.md` §9.5. However, `Field::ball_clearance` and `Field::body_clearance` subtract `r` *after* `Field::pieces` clamps `taper.dist` to `-c`, causing the 4-ft taper wall to intrude by `r` (`3.7 cm` for the ball, `30 cm` for fielders) into fair territory near the foul poles (and along steep tapers), turning 4-to-8-ft fair line drives near the pole into home runs before they reach the 8-ft outfield fence (**P1**). In addition, dugouts have `wall_top == 0.0` with no roof height (**P2**), `Touch::OutOfPlay` and `MarkerKind::BallDead` are delayed until the end of `into_the_stands` (**P2**), and rolling balls only check `beyond(pos(hi))` at the endpoint of the roll (**P2**).
3. **Replayability (Invariant 5):** Clean. No `HashMap`/`HashSet` iteration exists in the change set (`BTreeMap` is used in `scenarios.rs`, `game_lab.rs`, and `play_area_probe.rs`), no randomness exists outside `KeyedRng`, and fact/runner sorting uses total deterministic keys (`f64::total_cmp` and `(Reverse(b), Reverse(r.start_base))`).
4. **Causal Agent Perception (Invariant 2):** No agent reads hidden world state or knows a throw will go out of play before it reaches the boundary (`update_thrown` transitions only when `self.t >= path.last().t`, and `read_prediction` uses the static `Field` boundary on the agent's own noisy trajectory estimate). However, because `read_prediction` includes `into_the_stands` and `intercept_scan` / `believed_field_time` never check whether an intercept point lies beyond a wall, fielders call sure catches on foul pops landing deep in the seats and run full speed into walls without braking until clamped by `keep_inside` (**P2**).
5. **Measurement Claims (`docs/calibration/2026-11-14-play-area.md` & `docs/metrics-ledger.md`):** The foul-territory polygon shoelace area (`15,400 + 6,000√2 = 23,885.28 sq ft`) is exact, and the walls-out-of-reach control reproduces the baseline Once the taper clamp is applied. However, the prose in "Why each number moved" and `docs/metrics-ledger.md` quotes stale deltas from the pre-merge `a69dd50 → 25fa2dd` table that contradict the primary `52d6632 → 980a7dc` table, and understates how completely the `+0.11`-point composition effect (`2.08%` of PAs converted from 0%-HR foul outs into continuing PAs with `~5.1%` HR) explains the `+0.08`-point mean HR/PA rise on `52d6632 → 980a7dc` (**P3**).
6. **Tests:** Most new tests check independent geometry and rules invariants (`foul_territory_area_matches_the_spec`, `backstop_and_side_walls_meet_where_the_spec_says`, `awards-004`..`awards-013`). However, `foul_walls_never_cut_fair_ground` tests `body_clearance(x, y, 0.0)` with `r = 0.0` instead of `BALL_R` or `BODY_R` (which masked P1-1), and `play_area.rs` checks `MarkerKind::BallDead` against `t_out` from `step.facts` (restating the delayed `path.t_end()` timestamp and ignoring `p`).

---

## Findings

### P1 — Wrong Behavior or Broken Invariant

#### P1-1. Taper wall intrudes into fair territory by `r` (`BALL_R = 3.7 cm`, `BODY_R = 30 cm`) near the foul poles and along steep tapers, turning 4-to-8-ft fair line drives into home runs in front of the 8-ft fence and blocking fielders in fair territory
- **File & Line:** `sim/crates/bb_physics/src/ground.rs:142-165` (`Field::pieces`, `Field::ball_clearance`, `Field::body_clearance`), `sim/crates/bb_physics/src/ground.rs:620-635` (`foul_walls_never_cut_fair_ground`)
- **Evidence:**
  - `docs/physics-spec.md` §9.5 (line 335) requires: *"The walls bound foul territory only: fair ground is bounded by the fence alone... a point fair of the foul line is never outside the taper (its distance is the larger of the two)."*
  - In `Field::pieces` (`ground.rs:142-146`), the fair-ground guard clamps the un-offset taper distance to `-c`:
    ```rust
    let taper = if -c > taper_d {
        WallAt { kind: WallKind::Taper, dist: -c, inward: Vec3::new(-sg, 1.0, 0.0) * (1.0 / r2), s }
    } else {
        WallAt { kind: WallKind::Taper, dist: taper_d, inward, s }
    };
    ```
  - Immediately afterward, `ball_clearance(x, y, r)` (`ground.rs:157-159`) and `body_clearance(x, y, r)` (`ground.rs:171`) subtract `r` from every non-`Fence` piece (`if w.kind != WallKind::Fence { w.dist -= r; }`).
  - Because `r` is subtracted *after* the `if -c > taper_d` clamp:
    1. Whenever `-c > taper_d` (which holds in fair territory near the foul poles on the default park because `taper_d = (k * (s_pole - s) - c) / sqrt(1 + k^2) < -c` at `s = s_pole`, and holds all along the foul line past `s_T` for a steeper taper), `w.dist` becomes `-c - r`. For any fair point within `r` of the foul line (`0 <= -c < r`), `w.dist < 0` in fair territory.
    2. Even just short of the pole where `-c <= taper_d` (e.g., `s = s_pole - 0.05 m`, `c = -0.02 m`, 5 cm in front of the 8-ft outfield fence and 2 cm fair of the foul line), `taper_d = 0.0302 m < BALL_R (0.0369 m)`, so `taper.dist = taper_d - BALL_R = -0.0067 m < 0` while `fence.dist = +0.05 m > 0`.
    3. Thus, a fair ball near the foul pole triggers `Crossing::Boundary` on `WallKind::Taper` *before* reaching `WallKind::Fence`. Because `wall_top(&w)` for `WallKind::Taper` is `foul_wall_height = 4 ft` (`1.22 m`) rather than `wall_height = 8 ft` (`2.44 m`), a fair line drive at `z = 1.5 m` (5 ft high — 3 ft below the top of the 8-ft outfield fence) is judged `(WallContact::Out, WallKind::Taper)` (`ground.rs:219`), exits with `fair_air(x, y) == true`, and is ruled a **home run** by `bb_rules`. Below 4 ft, it caroms off the phantom taper wall in front of the outfield fence.
    4. This directly explains the anomaly noted in `docs/calibration/2026-11-14-play-area.md` line 89 (`home run | taper 0.007` and `0.003`): *"Two balls over 300 games crossed the taper wall within a ball radius of the line near a pole, so they were fair in flight: home runs."*
    5. For fielders (`BODY_R = 0.3 m`), `body_clearance(x, y, 0.3)` is negative across a 30-cm strip of fair territory near the foul poles (and along steep tapers). `foul_walls_never_cut_fair_ground` (`ground.rs:629`) did not catch this because it tested `f.body_clearance(x, y, 0.0)` with `r = 0.0`.
- **Suggested Fix:**
  - Apply the fair-ground guard *after* subtracting `r` (or ensure that for any point fair of the foul line, `c <= 0`, or fair in air, `c <= r`, the taper's effective clearance after `- r` is never negative and never smaller than the outfield fence's `fence_at(theta) - rho`). Update `foul_walls_never_cut_fair_ground` to assert `ball_clearance(x, y, BALL_R).dist >= 0.0` and `body_clearance(x, y, 0.3).dist >= 0.0` on fair ground inside the fence.

#### P1-2. Walk-off on `throw_out_of_play` or `ground_rule_double` emits `RunnerAdvanced { to: 4, cause: Award }` without `RunScored` for trailing runners after `st.walk_off` is already true, and fails to cap awards (OBR 7.01(g) / rules-spec §4.1)
- **File & Line:** `sim/crates/bb_rules/src/engine.rs:756-764` (`ground_rule_double`), `sim/crates/bb_rules/src/engine.rs:794-809` (`throw_out_of_play`)
- **Evidence:**
  - `docs/rules-spec.md` §4.1 (line 138) specifies: *"When the winning run scores in the bottom of the 9th or later, the game ends immediately as a walk-off. Only the runs needed to win count, and the batter is credited with only as many bases as the winning runner needed, except a fair ball out of the park in flight (home run), on which the batter and all runners score [S: OBR 7.01(e)(3)]."*
  - In both `throw_out_of_play` (`engine.rs:794-809`) and `ground_rule_double` (`engine.rs:756-764`), runners are awarded from lead runner to batter-runner:
    ```rust
    if to != r.start_base {
        ev.push(Event::RunnerAdvanced { runner: r.runner, from: r.start_base, to, cause: AdvanceCause::Award });
    }
    if to == 4 {
        score_run(st, r.runner, ev);
    } else if to >= 1 {
        st.bases[(to - 1) as usize] = Some(r.runner);
    }
    ```
  - In a tied game in the bottom of the 9th with runners on 2nd (`A.2`) and 3rd (`A.3`), when `A.3` is awarded home (`to = 4`), `score_run` scores the winning run and sets `st.walk_off = true` (`engine.rs:522`). On the next loop iteration, `A.2` is *also* given `to = 4`: `Event::RunnerAdvanced { runner: A.2, from: 2, to: 4, cause: Award }` is emitted unconditionally, and then `score_run` returns early because `st.walk_off` is already `true` (`engine.rs:514`). This leaves `A.2` with a `RunnerAdvanced { to: 4 }` event and no `RunScored` event, and awards the batter-runner 2nd base instead of the 1 base needed by the winning run.
- **Suggested Fix:**
  - Check `st.walk_off` inside the award loops in `throw_out_of_play` and `ground_rule_double`: once the winning run scores (or by capping the award distance to `4 - winning_runner_from` per OBR 7.01(g)), stop awarding home to trailing runners and cap the batter-runner's base award accordingly.

#### P1-3. `BallReleasedFromGlove` checks `r.last_base > r.start_base` instead of `r.at > r.start_base` for the infielder first-play rule, and awards un-retouched advanced bases (or pre-catch home touches) to runners who left early on a caught fly (OBR 5.06(b)(4)(G))
- **File & Line:** `sim/crates/bb_rules/src/engine.rs:325-331` (`step_fact` for `Input::BallReleasedFromGlove`), `sim/crates/bb_rules/src/engine.rs:783-810` (`throw_out_of_play`)
- **Evidence:**
  - In `engine.rs:325-329`:
    ```rust
    let advanced = lp.runners.iter().filter(|r| !r.out).all(|r| r.scored_at.is_some() || r.last_base > r.start_base);
    let first_play = lp.throws == 0 && lp.outs.is_empty();
    let from_pitch = infielder && first_play && !advanced;
    let from = lp.runners.iter().map(|r| (r.runner, if from_pitch { r.start_base } else if r.scored_at.is_some() { 4 } else { r.at })).collect();
    ```
  - Three edge cases break OBR 5.06(b)(4)(G):
    1. **Retreated runner (`last_base` vs `at`):** `r.last_base` (`state.rs:38`) is the high-water mark and never decreases when a runner retreats, whereas `r.at` (`state.rs:40`, `engine.rs:883`) moves backward on a retreat. If a runner on 1st touches 2nd (`last_base = 2`) and retreats to 1st (`at = 1`) while the batter-runner reaches 1st (`at = 1`) before an infielder's first throw, `r.last_base > r.start_base` evaluates to `true` even though at the time of the throw the lead runner has not advanced beyond his time-of-pitch base (`at == start_base == 1`).
    2. **Runner who left early on a caught fly (`r.retouch_required && !r.retouched`):** If a runner on 1st leaves before a fly ball is caught and touches 2nd (`r.at = 2`), or a runner on 3rd touches home before the catch (`r.scored_at = Some(..)` while `returning(r)` is `true`), and the throw after the catch goes out of play before the runner retouches `r.start_base`, `engine.rs:328` sets `from = r.at = 2` (or `from = 4` if `r.scored_at.is_some()`), awarding home (`score_run`) based on a base touched illegally before the catch! Under OBR 5.06(b)(4)(G) and 5.09(b)(5), a runner who leaves early on a caught fly must retouch his time-of-pitch base; until he retouches, his last legally held base is `r.start_base`, not `r.at` or `4`.
    3. **Runner who already legally scored before the throw:** If a runner legally touched home (`r.scored_at.is_some() && (!r.retouch_required || r.retouched)`) before the throw was released, `throw_out_of_play` (`engine.rs:802`) emits `RunnerAdvanced { cause: AdvanceCause::Award }` instead of `AdvanceCause::OnPlay` (and if `from_pitch` is `true` while a runner from 1st had already touched home, `from = r.start_base = 1` resets him to `to = 3`, erasing his run).
- **Suggested Fix:**
  - Compute each runner's last legally held base at throw release as `let legal_at = if r.retouch_required && !r.retouched { r.start_base } else if r.scored_at.is_some() { 4 } else { r.at };`.
  - Compute `advanced` from `legal_at > r.start_base`.
  - Set each runner's `from` base to `if from_pitch { r.start_base.max(legal_at) } else { legal_at }`, and in `throw_out_of_play`, emit `AdvanceCause::OnPlay` for any runner who had already legally reached `from == 4` before the throw.

#### P1-4. A caught fair fly that subsequently leaves the field without a throw (`Touch::OutOfPlay` with `lp.caught == true` and `lp.throw.is_none()`) is ruled a home run or ground-rule double
- **File & Line:** `sim/crates/bb_rules/src/engine.rs:695-713` (`live_touch`)
- **Evidence:**
  - In `live_touch` (`engine.rs:683-714`), when `Touch::OutOfPlay` arrives with `lp.throw.is_none()`:
    - The `!fair` branch (`engine.rs:697-701`) explicitly handles `else if lp.caught { let lp = lp.clone(); close_play(st, &lp, ev); }`.
    - The `fair` branch (`engine.rs:703-712`) has no `lp.caught` guard: if `lp.caught == true` and `lp.fair == FairFoul::Fair`, it calls `home_run(st, ev)` (when `in_flight && !lp.grounded`) or `ground_rule_double(st, ev)`, awarding bases/runs after the batter was already retired by `OutKind::FlyOut` and emitting a duplicate `PlateAppearanceEnded`.
- **Suggested Fix:**
  - Check `if lp.caught` before branching on `!fair` vs `fair` when `lp.throw.is_none()` (or award one base from time of pitch per OBR 5.06(b)(3)(C) / 5.09(a)(1) Comment when a fielder carries a caught fly out of play, and close the play).

---

### P2 — Physics Edge Cases, Agent Perception at Walls, Timeline Semantics, and Missing Scenarios

#### P2-1. Dugouts have `wall_top == 0.0` with no roof height, so high foul pop-ups crossing `c = 50 ft` between `s = 20 ft` and `s = 80 ft` at any altitude are classified as entering the dugout
- **File & Line:** `sim/crates/bb_physics/src/ground.rs:196-203` (`Field::wall_top`), `sim/crates/bb_game/examples/play_area_probe.rs:44-48`, `docs/calibration/2026-11-14-play-area.md:79-86, 106`
- **Evidence:**
  - `Field::wall_top` (`ground.rs:200`) returns `0.0` whenever `w.kind == WallKind::Side && w.s >= self.dugout_from_m && w.s <= self.dugout_to_m`, regardless of the ball's height `z`.
  - A high foul pop-up that crosses the vertical plane `c = 50 ft` at `s = 50 ft` and `z = 20 m` (heading into the stands behind the dugout) is treated as entering the dugout (`top = 0.0`), flown down to `z = r` instead of `4 ft + r` by `into_the_stands`, and tallied as `"dugout"` by `play_area_probe.rs:46`.
  - Consequently, `docs/calibration/2026-11-14-play-area.md` (line 81) reports `dugout 2.55` foul balls per game (nearly half of all side-wall foul pops, since the 60-ft dugout spans 60% of the 100-ft side wall) and attributes this in line 106 to missing protective netting, when in fact high pop-ups passing 30–100 ft above the dugout opening land in the seats behind the dugout.
- **Suggested Fix:**
  - Cap the dugout opening vertically (e.g., only treat `WallKind::Side` for `s ∈ [dugout_from_m, dugout_to_m]` as a dugout opening when the ball crosses at `z <= foul_wall_height + r` or a named `dugout_height`, and as the stands above the dugout roof otherwise).

#### P2-2. `Touch::OutOfPlay` and `MarkerKind::BallDead` are emitted at `path.t_end()` (after `into_the_stands`) rather than when the ball crossed the boundary, and `MarkerKind::BallDead` hardcodes `p.z = 0.0`
- **File & Line:** `sim/crates/bb_game/src/live.rs:2157, 2175, 2280`, `sim/crates/bb_game/src/timeline.rs:299-303`, `docs/data-formats.md:355`
- **Evidence:**
  - `docs/data-formats.md` §5.4 (line 355) and `docs/rules-spec.md` §5.8 (line 240) state: *"`BallDead` (emitted since 2026-11-14, A10) is the moment the ball left the playing field (physics-spec §9.5; rules-spec §5.8): `p` is where it crossed the boundary, `agent` and `base` are 0. The ball track ends there and the play is over, so a client can cut back to the field from a ball in the seats or the dugout."*
  - In `batted_path_dt` (`ground.rs:444`) and `over_the_wall` (`ground.rs:240`), any ball crossing a non-`Fence` wall in flight is joined with `into_the_stands`, which flies the ball for up to 1–2 additional seconds until `z = top + r`. `live.rs:2157, 2175` then emits `Input::BallTouched { t: path.t_end(), what: Touch::OutOfPlay { x: q.x, y: q.y, in_flight } }` at `path.t_end()` (when the ball lands in the seats), while setting `(x, y)` to the earlier boundary crossing `q`.
  - For a **thrown ball** (`over_the_wall` at `live.rs:2280`), no catch in the seats is meaningful, yet delaying `Touch::OutOfPlay` until `path.t_end()` keeps the play live while the ball falls inside the dugout or stands — allowing runners to touch home (`try_walk_off`) or pass another runner (`RunnerPassed`) *after* the throw has already left the playing field.
  - Furthermore, `timeline.rs:301` emits `MarkerKind::BallDead` at `t = path.t_end()` (so the ball track in the timeline does *not* end at the boundary crossing) with `p = Vec3::new(x, y, 0.0)` (`z` hardcoded to `0.0`).
- **Suggested Fix:**
  - For throws (`update_thrown`, `live.rs:2280`), emit `Touch::OutOfPlay` at the boundary crossing (`path.last().t`) without flying `into_the_stands`.
  - For batted balls and timeline generation, record the boundary-crossing time `tc` and 3D position `q` (`Vec3::new(q.x, q.y, q.z)`) for `MarkerKind::BallDead` once the ball goes uncaught over the wall.

#### P2-3. Fielders and runners evaluate `into_the_stands` trajectories without checking wall boundaries, causing fielders to call sure catches on balls deep in the seats and slam into walls at full speed
- **File & Line:** `sim/crates/bb_game/src/live.rs:736` (`read_prediction`), `sim/crates/bb_game/src/live.rs:1200-1218` (`believed_field_time`), `sim/crates/bb_game/src/live.rs:1270-1320` (`intercept_scan`), `sim/crates/bb_game/src/live.rs:1458-1475` (`call_value`), `sim/crates/bb_game/src/live.rs:1837-1844` (`move_fielders`)
- **Evidence:**
  - Because `read_prediction` calls `batted_path_dt` (which appends `into_the_stands` down to `z = 1.256 m` in the seats or `z = 0.037 m` in a dugout), `pred` continues into the stands where `b.z <= CATCH_MAX_Z (2.4 m)`.
  - Neither `intercept_scan` (`live.rs:1284-1318`) nor `believed_field_time` (`live.rs:1200-1218`) checks whether the candidate intercept point `b = pred.at(t).0` is within reach from inside the wall (`field.body_clearance(b.x, b.y, 0.0).dist >= -(REACH - BODY_R)`).
  - As a result, a fielder who could reach `(b.x, b.y)` in an unobstructed plane finds an intercept in the seats, calls a `tier = 2` or `3` catch in `call_value`, causes baserunners to expect a catch in `believed_field_time`, and aims at `fd.target` deep in the seats (`live.rs:1239`), running into the wall at full speed (`~8 m/s`) without braking in `fielder_step` until clamped by `keep_inside` (`live.rs:1840`).
- **Suggested Fix:**
  - In `intercept_scan` and `believed_field_time`, filter out trajectory points `b` whose `field.body_clearance(b.x, b.y, 0.0).dist < -(REACH - BODY_R)` (or `-(REACH + STRETCH - BODY_R)`), and clamp `fd.target` with `field.keep_inside(fd.target, BODY_R)` so `fielder_step` decelerates smoothly as the fielder approaches the wall.

#### P2-4. Rolling ball boundary check (`ground.rs:410`) only tests `beyond(pos(hi))` at the stop point, glove deflections over a wall bypass `Crossing::Boundary`, and `Field::fair` ignores `BALL_R`
- **File & Line:** `sim/crates/bb_physics/src/ground.rs:115-117` (`Field::fair`), `sim/crates/bb_physics/src/ground.rs:407-434` (`batted_path_dt` roll segment), `sim/crates/bb_physics/src/flight.rs:288-292`
- **Evidence:**
  1. In `batted_path_dt` (`ground.rs:410`), a rolling ball tests for wall contact only via `if beyond(pos(hi))` at the final resting point `hi = t_stop - t`. Because the boundary is non-convex (re-entrant) at the foul poles where the taper wall (`c <= k * (s_pole - s)`) meets the fair outfield fence (`rho <= fence_at(theta)`), a ball rolling from foul territory near the line across the pole into fair territory can pass outside the taper wall mid-roll while `beyond(r0) == false` and `beyond(pos(hi)) == false`.
  2. If a fielder at the wall reaches over it (up to `REACH = 1.0 m`, i.e., `0.7 m` outside the wall) and deflects a ball (`live.rs:2085-2093`), `set_free(s0)` invokes `batted_path_dt` with `ball_clearance(s0.r) < 0`. Because `Crossing::Boundary` ignores a stop already satisfied at `t0` unless the path re-enters first (`flight.rs:288`), the ball flies to the ground in the stands (`z = r`), bounces, and enters the roll bisection with `beyond(r0) == true` (driving `hi` to `0.0` and settling out of reach behind the wall).
  3. `Field::fair(x, y)` (`ground.rs:115`) checks `y >= x.abs()` (ball center) rather than `line_dist(x, y) >= -r` (`rules-spec §5.2`), so `SegmentEnd::OverFence { fair }` (`ground.rs:239, 421, 441`) disagrees with `bb_rules`'s `fair_air(x, y)` when the ball center is within `BALL_R` on the foul side of the line.
- **Suggested Fix:**
  - If `field.ball_clearance(s0.r.x, s0.r.y, r).dist < 0.0` at the start of `batted_path_dt`, transition directly to `into_the_stands` / `SegmentEnd::OverFence`.
  - Check whether the roll segment intersects any wall along `[0, t_stop - t]` (not just at `hi`).
  - Use `y + x.abs() >= 0 && (y - x.abs()) * FRAC_1_SQRT_2 >= -r` for airborne fair/foul in `SegmentEnd::OverFence`.

#### P2-5. Claimed scenario for "fair deflected fly out over foul ground" (OBR 5.05(a)(8)) is missing from `scenarios/awards/`, along with wall-carom throws and 3-runner blocked awards
- **File & Line:** `docs/rules-spec.md:561` (§12.3 `awards` row), `sim/crates/bb_rules/src/engine.rs:706-712`, `sim/crates/bb_rules/scenarios/awards/awards-004.toml`..`awards-013.toml`
- **Evidence:**
  - Both commit `959bb2b`'s message (*"Rules: thrown ball out of play ..., BallDead event, deflected fair fly out over foul ground; 10 scenarios"*) and `docs/rules-spec.md` §12.3 (`awards` row, line 561: *"foul ball out of play; fair deflected fly out over foul ground"*) state that the `awards` scenario suite tests a fair fly deflected out over foul territory (`engine.rs:706-712`, OBR 5.05(a)(8)).
  - None of `awards-001.toml`..`awards-013.toml` (nor any other scenario file) tests a ball ruled fair in flight that then exits `OutOfPlay` with `in_flight = true` at foul coordinates.
  - Also missing from `scenarios/awards/`: a thrown ball that caroms off a wall (`Touch::Wall`) and stays in play (or then goes `OutOfPlay`), and a 3-runner blocked-award cascade (e.g., runners on 1st and 2nd held at their bases while the batter-runner reaches 1st: `from = 2, 1, 1` → `to = 4, 3, 2`).
- **Suggested Fix:**
  - Add scenarios in `sim/crates/bb_rules/scenarios/awards/` for: (1) a deflected fair fly leaving in flight over foul territory (`GroundRuleDouble`), (2) a throw caroming off `Wall`, and (3) a 3-runner blocked award.

---

### P3 — Calibration Note & Ledger Discrepancies

#### P3-1. "Why each number moved" and `docs/metrics-ledger.md` quote stale pre-merge `a69dd50 → 25fa2dd` deltas that contradict the primary `52d6632 → 980a7dc` table, and understate the HR/PA composition explanation
- **File & Line:** `docs/calibration/2026-11-14-play-area.md:75-100`, `docs/metrics-ledger.md:105`
- **Evidence:**
  1. **Stale deltas in `2026-11-14-play-area.md` (lines 75–97) and `docs/metrics-ledger.md` (line 105):**
     - Line 95 says *"PU .015 → .021 move in opposite directions on the two seed sets"*. That was true on `a69dd50 → 25fa2dd` (`.017 → .013` and `.015 → .021`), whereas on the primary `52d6632 → 980a7dc` table (lines 35–36), PU moved **up on both seed sets** (`.011 → .019` and `.015 → .021`).
     - Line 96 says *"Reached on BIP rose by 1.1–1.4 points... Measured: 33.6–33.9%"*, and `docs/metrics-ledger.md` line 105 says *"reached on BIP up 1.1–1.5 points by composition"*. On `52d6632 → 980a7dc` seeds 1000–1299 (line 28 and ledger line 105), Reached on BIP moved `32.7% → 33.2%` (**+0.5 points**, to **33.2%**).
     - Line 97 says *"K% rose by 0.5–0.8 and pitches by 5 a game. PA/team-game rose by 0.4."* On `52d6632 → 980a7dc` (lines 29, 31, 32), K% rose by **1.0 point on both sets** (`15.3 → 16.3` and `14.8 → 15.8`), and `PA/team-game` rose by **0.2** (`38.2 → 38.4` and `38.8 → 39.0`).
     - Lines 75–89 ("Where balls leave") report `foul` (`10.50` / `10.56`), `home run` (`4.08` / `4.14`), and `after a throw` (`0.17` / `0.10`, 51 and 30 of 300), which are the numbers from the pre-merge `25fa2dd` table (lines 66–68) rather than `980a7dc` (`10.49` / `10.59`, `3.98` / `3.97`, `0.177` / `0.113`).
  2. **HR/PA composition arithmetic (line 99):**
     - Line 99 states: *"HR/PA rose by 0.14 and 0.43 points on `a69dd50`, and by 0.04 and 0.12 on `52d6632` (mean of the four +0.18). Some rise is expected: PAs that used to end on a foul pop go on to other outcomes, about 5% of them home runs. That accounts for only about +0.1 point."*
     - Converting `~1.60` caught foul pops/game (`0.80` per team-game, or `0.80 / 38.5 = 2.08%` of all PAs) from a 0%-HR foul out into a continuing PA (with a `~5.1%` HR rate) raises HR/PA by `2.08% × 5.1% = +0.106` percentage points (`~+0.11` points). On the merged `52d6632 → 980a7dc` build, the measured HR/PA change is `+0.04` and `+0.12` points (mean **`+0.08` points**), which is **100% explained** by the `+0.11`-point composition effect. Even on `a69dd50`, because each game's trajectory diverges after the first changed foul pop in inning 1, the binomial standard error of the difference between two 300-game runs (`N ≈ 23,100` PAs, `p ≈ 0.051`) is `√2 × sqrt(p(1-p)/N) ≈ 0.205` percentage points, so `+0.14` and `+0.43` are within normal sampling noise of `+0.11`.
- **Suggested Fix:**
  - Update "Why each number moved" (`docs/calibration/2026-11-14-play-area.md:95-99`) and `docs/metrics-ledger.md:105` to reflect the `52d6632 → 980a7dc` deltas, note that the `+0.08`-point mean HR/PA rise on `52d6632 → 980a7dc` matches the `+0.11`-point composition prediction, and label the "Where balls leave" breakdown as measured on `25fa2dd` (or update it from `980a7dc`).

---

## Verification

### Commands Run
- `git log 52d6632..dbcf92c --oneline && git diff --stat 52d6632...dbcf92c`
- `git diff 52d6632...dbcf92c -- sim/` and `git diff 52d6632...dbcf92c -- docs/data-formats.md docs/physics-spec.md docs/rules-spec.md`
- `export PATH=$HOME/.cargo/bin:$PATH && CARGO_BUILD_JOBS=2 taskset -c 0-3 nice cargo test --release --manifest-path sim/Cargo.toml -p bb_rules --test scenarios` — **passed** (1 test, entire scenario corpus including `awards-001`..`awards-013`).
- `export PATH=$HOME/.cargo/bin:$PATH && CARGO_BUILD_JOBS=2 taskset -c 0-3 nice cargo test --release --manifest-path sim/Cargo.toml -p bb_physics -p bb_rules --lib` — **passed** (30 tests in `bb_physics`, including `foul_territory_area_matches_the_spec`, `backstop_and_side_walls_meet_where_the_spec_says`, `wall_tops_by_place`, `stands_wall_carom_and_over_the_top`, `foul_balls_meet_the_walls`, `foul_walls_never_cut_fair_ground`, `keep_inside_stops_a_body_short_of_the_wall`).
- `export PATH=$HOME/.cargo/bin:$PATH && CARGO_BUILD_JOBS=2 taskset -c 0-3 nice cargo test --release --manifest-path sim/Cargo.toml -p bb_game --test play_area` — **passed** (12-game integration test `balls_out_of_play_end_the_play_and_fielders_stay_inside_the_walls` in 3.99s).
- `export PATH=$HOME/.cargo/bin:$PATH && CARGO_BUILD_JOBS=2 taskset -c 0-3 nice cargo test --release --manifest-path sim/Cargo.toml -p bb_game --lib -- wall` — **passed** (7 tests, including `a_throw_over_the_stands_wall_is_dead_with_two_bases_from_the_pitch`, `a_low_throw_caroms_off_the_stands_wall_and_stays_live`, `a_fielder_chasing_into_the_seats_stops_at_the_wall`).

### Files Read Only (Not Executed as Full 300-Game Calibration Batches)
- `CLAUDE.md`
- `docs/calibration/2026-11-14-play-area.md`
- `docs/metrics-ledger.md`
- `docs/physics-spec.md` (§9.1–§9.5)
- `docs/rules-spec.md` (§2, §4.1, §5.2, §5.3, §5.5, §5.6, §5.8, §7.2, §12.3)
- `docs/data-formats.md` (§4.3, §5.4)
- `sim/crates/bb_rules/src/state.rs` and `sim/crates/bb_rules/src/engine.rs`
- `sim/crates/bb_rules/tests/scenarios.rs` and `sim/crates/bb_rules/scenarios/awards/awards-001.toml`..`awards-013.toml`
- `sim/crates/bb_physics/src/ground.rs`, `sim/crates/bb_physics/src/flight.rs`, `sim/crates/bb_physics/src/throw.rs`
- `sim/crates/bb_game/src/live.rs`, `sim/crates/bb_game/src/game.rs`, `sim/crates/bb_game/src/eventlog.rs`, `sim/crates/bb_game/src/timeline.rs`
- `sim/crates/bb_game/tests/play_area.rs` and `sim/crates/bb_game/examples/play_area_probe.rs`
- `sim/crates/bb_cli/src/game_lab.rs` and `sim/crates/bb_stats/tests/accounting.rs`

## Resolution

Resolved 2026-11-14 on `work/a10-review-fixes` (from main `0a17409`), by the A10 worker. Each finding was checked first with a failing test, a scenario or a probe. Full suite after the fixes: 230 passed, 0 failed (`cargo test --workspace --release`); 129 rules scenarios, 0 failed. Standard run main `0a17409` → `27c2494`: runs 5.07 → 5.19 [5.28 → 5.46], the cause measured by a counterfactual (P2-3 below); see the [note](../calibration/2026-11-14-play-area.md#after-the-gemini-review-fixes-main-0a17409--27c2494) and the ledger row.

**P1-1 — fixed (`54c45b0`).** Confirmed: `foul_walls_never_cut_fair_ground`, rewritten with the real radii (ball 0.036888 m, body 0.3 m) over the reference park and a steep-taper park, failed on main (a body 2.7 m fair of the third-base line was held by the taper). The radius now comes off each wall's distance first and the fair-ground rule applies after (`Field::pieces_for`), so a ball or body centred on fair ground never meets a foul-territory wall; the test passes, with the review's point (5 cm short of the fence, 2 cm fair) checked to meet the fence, not the taper. physics-spec §9.5 says so. On the note's "home run over the taper" rows: after the fix there are still two in 300 games (seeds 0–299), and the probe shows why they are legitimate: both crossed within its 5 cm print precision of the foul line, 17 cm short of the pole, centred just foul, so any part of the ball was over fair ground where it left (rules-spec §5.2). That is where the foul pole stands, and off the pole a ball is a home run. A fair line drive in front of the fence can no longer meet the taper.

**P1-2 — fixed (`3b5b62f`).** Confirmed with scenarios awards-014 (ground-rule double) and awards-015 (throw into the dugout), each a tie in the bottom of the 9th or 10th: on main the trailing runner got `RunnerAdvanced { to: 4 }` with no `RunScored`. The harness had no way to fail on an event that should not appear, so it gained `expect_no_events` (rules-spec §12.2). Both award loops now stop at the winning run, as `try_walk_off` does on any walk-off: nobody behind it, the batter included, is awarded anything more. On the batter, the suggested cap is not applied: the engine places no batter on a walk-off hit either, and the credit for bases is the scorer's (OBR 9.06(f), rules-spec §10), so the event log stays the same shape for every walk-off.

**P1-3 — fixed (`9d0b17f`), with part of the suggested fix not taken.** All three cases confirmed by scenarios that failed on main. awards-016: a runner who touched third and went back to second had "advanced", which turned an infielder's first wild throw into a time-of-throw award (batter to third instead of second). awards-017: a runner who left early on a caught fly was awarded home from the base he reached early. awards-018: a runner who had legally scored before an infielder's first throw was sent back to third. Each runner's base at the release is now his legal base: the base he last touched (a retreat counts), his own base while he owes a retouch, home once he has legally scored. A legally scored run stands as `OnPlay`. The suggestion `from_pitch → start_base.max(legal_at)` was not taken: it would award a runner who reached second before an infielder's first throw from second, against OBR 5.06(b)(4)(G) (time of the pitch; awards-006 pins that case and passes). Only a runner who had already scored keeps his base. rules-spec §7.2 is updated.

**P1-4 — fixed (`7b1bd03`).** Confirmed: scenario awards-019 (a caught fair fly, then `OutOfPlay` with no throw) was a two-run home run on main, with the batter already out. A caught fly, fair or foul, that leaves the field without a throw is now dead with one base for each runner from the base he holds (a retouch owed counts as his own base) [S: OBR 5.06(b)(3)(C), a fielder carrying the ball out of play]. The caught-foul branch now uses the same rule. The sim cannot produce the play yet (a caught ball leaves the fielder only by a throw); rules-spec §7.2 has a row for it.

**P2-1 — fixed (`9fbf77b`).** Confirmed in the code: the dugout's top was 0 at any height. A dugout now has a roof (`dugout_roof_m`, 4 ft [E]). A ball crossing the opening under it comes down to the dugout floor; over it, it comes down on the seats on the roof (`Field::landing_top`; test `a_dugout_takes_low_balls_and_the_roof_takes_high_ones`). Both are out of play, as before. The probe now separates them: of the 2.55 "dugout" fouls a game, about 2.4 were pops over the roof, 0.2 really into the dugout. Deferred: the client draws the roof at 2.4 m, not 4 ft (client work, after A16).

**P2-2 — partly fixed (`251109d`), partly kept with the reason.** Confirmed in the code. Fixed: a throw over a wall is now dead as it crosses (nobody catches a throw in the seats), so runners can no longer touch bases while it falls; the live test checks the out-of-play time against the moment the throw met the wall and its height above the 4-ft top. Fixed: the out-of-play fact (`Touch::OutOfPlay { z }`) and the `BallDead` marker carry the crossing point's height; `play_area.rs` now checks that the marker's point lies on the boundary (within 2 mm of a ball radius inside a wall's plane, or on the fence's curve), a geometric property rather than a restatement of the fact. Kept: a batted ball over a foul-territory wall stays live until it comes down in the seats, because until then a fielder at the rail may reach over for it (rules-spec §5.3, robbing a ball over a wall from inside the field); its marker time is that moment. The data-formats §5.4 text that said "the moment the ball left the playing field" was wrong for that case and now says exactly when each kind of ball is dead.

**P2-3 — fixed (`34726ad`).** Confirmed: a test with foul pops landing 0–12 m into the first-base seats had a fielder reach the stands wall at 7.21 m/s on main. Players see the walls: a point on a fielder's own predicted path farther past a wall than he can reach over it (0.7 m past the wall's plane) is no intercept, and the runners' estimate of when the ball is fielded skips such points too. In the movement step he never comes at the nearest wall faster than he can stop before it at `BRAKE`. No separate `keep_inside` clamp of `fd.target` was added: the braking and the existing position clamp cover it, and the test now passes (arrival at most √(2·6·0.35) m/s plus one sample's slack). agent-models §5.3 says so. Effect on games, by counterfactual: +0.08 [+0.18] runs. Fielders reach balls near the fence and the stands a little later; this is the only fix that moved the run environment.

**P2-4 — (1) rejected with evidence, (2) and (3) fixed (`9fbf77b`).** (1) The playing field is convex: the half-planes, the fence's curve (r(θ) = 400 − 70(θ/45°)² ft satisfies r² + 2r′² − r·r″ > 0) and the taper joined to fair ground at the pole, which lies on the fence. So a straight roll that starts and stops inside never leaves it. `the_playing_field_is_convex` checks over 10,000 chords between inside points, densely near both poles, and finds none leaving the field. The stop-point check stands. (2) A ball that starts beyond a wall (knocked off a glove reaching over) is now out of play at once, where it is (test `a_ball_starting_beyond_a_wall_is_out_of_play`); before, it flew on, bounced and rolled in the stands. (3) `SegmentEnd::OverFence { fair }` now uses the ball radius (`Field::fair_in_air`). The flag is not read by the rules, which judge fair or foul from the fact's position with the radius already (rules-spec §5.2), so this was a mislabel, not a misruling.

**P2-5 — fixed (`24b58d3`).** Confirmed: no scenario had a fly ruled fair leaving in flight over foul ground. awards-020 adds it: two bases, not a home run. The sim sends no glove-deflection fact, so the scenario reaches the fair ruling through the ball passing third on the fair side and pins the engine's rule; it says so. awards-021: a throw caroming off the stands wall stays live, then rolls into the dugout, still a throw. awards-022: three runners, blocked. All three passed on the engine as it was; rules-spec §12.3 lists them.

**P3 — fixed (`ae06ade`).** "Why each number moved" now quotes the merged pair (`52d6632` → `980a7dc`), with the branch-point pair in brackets where it differs; the "Where balls leave" table is labelled with its build (`25fa2dd`), and the counts after these fixes are in the note's new last section. The HR/PA arithmetic is right: +0.11 point predicted from the extra PAs against +0.08 measured on the merged pair. The note no longer calls the rise unexplained, and the ledger's §5 entry is closed.
