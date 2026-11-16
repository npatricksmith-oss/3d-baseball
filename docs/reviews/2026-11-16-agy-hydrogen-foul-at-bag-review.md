# Independent review: fair or foul as the ball passes first or third (`30634cf..7188407`)

## Summary

Reviewed commits `327eed1` (sim) and `7188407` (client) against `CLAUDE.md`, `docs/rules-spec.md` (§2, §5.2), `docs/data-formats.md` (§5.4), OBR "Fair Ball" / "Foul Ball", and the Unreal Engine 5.8.4 headers.

- **`live.rs` (`passed_base`, `ball_facts`):** Core geometry, `BASE_L` (`27.432 m`), bisection, airborne vs. rolling boundary (`line >= -r_ball` vs. `line >= 0.0`), non-bounced fly exclusion, thrown-ball exclusion, time ordering, and determinism (Invariant 5: no transcendentals outside `bb_math::m`, no RNG, no hash iteration) are sound. Found one **P2** edge case where a ball bouncing in the infield and going over a foul wall/dugout before `BASE_L` crosses `BASE_L` while flying in the stands (`into_the_stands`) and terminates the play via `BallPassedBase` before `Touch::OutOfPlay` / `BallDead` is emitted, plus **P3** corner cases around `p.x * side < 0.0`, the `0.01 m` airborne margin, and rim deflections.
- **`describe.rs`:** Verified against `bb_rules/src/engine.rs` and a 50-game scan (`0` caught foul flies without `PlateAppearanceEnded`). Uncaught fouls (including 2-strike fouls) produce `"Foul"`; caught foul flies always emit `PlateAppearanceEnded` and enter the batted-ball branch (`"Fly out..."` / `"Pop out..."`). No defects found in normal completed-pitch usage.
- **`timeline.rs` and `bb_ffi`:** Found two **P2** issues: `sim/crates/bb_ffi/c/smoke.c` still rejects `dead_reason == 4` and fails `./crates/bb_ffi/c/run_smoke.sh`, and `timeline.rs` marks *any* `DeadBall::OutOfPlay` on a play containing `FairFoulDecided { fair: false }` as `4` — including a throw out of play after a caught foul fly. Also one **P3** spec doc gap in `docs/data-formats.md`.
- **`client/Source/BaseballClient/BbDirector.{h,cpp}`:** Checked `SoftDiscMaterial` against `~/UnrealEngine/5.8.4/Engine/Source/Runtime/Engine/Public/Materials/*.h` and `BbPark.cpp`: no compile errors found by header inspection. Replay (`ShowReplay`, `ReplayBack`, `ReplayForward`) and `HoldAfter` logic for `bFoulOut` are consistent for fouls into the seats, but inherit the **P2** misrouting if `dead_reason == 4` arrives on a caught foul fly + throw out of play.

---

## Findings

### P1

None found.

---

### P2

1. **C ABI smoke test fails on `dead_reason == 4`**
   - **Location:** [sim/crates/bb_ffi/c/smoke.c:47](file:///home/natex/Projects/3D%20Baseball/sim/crates/bb_ffi/c/smoke.c#L47)
   - **Concrete case:** Commit `327eed1` updated `bb_ffi::tests::plays_a_game_through_the_abi` (`sim/crates/bb_ffi/src/lib.rs:862`) to accept `(1..=4).contains(&info.dead_reason)`, but left `smoke.c:47` checking `info.dead_reason < 1 || info.dead_reason > 3`. On seed `3`, the first foul ball into the seats has `dead_reason == 4` and aborts the C smoke test with `bad dead ball`.
   - **How checked:** Executed `sim/crates/bb_ffi/c/run_smoke.sh` (seed `3`); it compiled and exited with code `1` printing `bad dead ball`.

2. **Caught foul fly followed by a throw out of play is marked `dead_reason = 4` (foul into seats) instead of `1` (throw out of play)**
   - **Location:** [sim/crates/bb_game/src/timeline.rs:334-340](file:///home/natex/Projects/3D%20Baseball/sim/crates/bb_game/src/timeline.rs#L334-L340) and [client/Source/BaseballClient/BbDirector.cpp:625, 655, 3219, 3234](file:///home/natex/Projects/3D%20Baseball/client/Source/BaseballClient/BbDirector.cpp#L625)
   - **Concrete case:** On a caught foul fly (`Event::Catch`, `Event::FairFoulDecided { fair: false }`) where a runner tags up and the fielder's subsequent throw goes out of play (`BallReleasedFromGlove { kind: Throw }`, `Event::BallDead { reason: DeadBall::OutOfPlay }`), `timeline.rs:334` sets `foul = true` and line 337 assigns `MarkerKind::BallDead.base() = 4` instead of `1` (unlike `game.rs:453`, which checks for a prior `released` timestamp before classifying `OutOfPlay` as foul). In `BbDirector.cpp`, `dead_reason == 4` sets `bFoulOut = true`, cutting `HoldAfter` to `1.0 s` and skipping `StartReset`, even though the batter was retired, runners received two-base awards, or the half-inning ended.
   - **How checked:** Inspected `timeline.rs:334-343` and `bb_rules/src/engine.rs:788-799, 873-898`; encoded a synthetic `PitchStep` containing `Catch`, `FairFoulDecided { fair: false }`, `BallReleasedFromGlove { Throw }`, and `BallDead { OutOfPlay }` via `bb_game::timeline::encode`, which produced `MarkerKind::BallDead` with `.base() == Some(4)`. (In a 50-game scan of seeds `0..50`, `0` plays happened to combine a caught foul fly with a throw out of play, so this is rare in current play distributions.)

3. **Batted ball bouncing in the infield and exiting over a foul wall/dugout before `BASE_L` triggers `BallPassedBase` in the stands, suppressing `BallDead { OutOfPlay }`**
   - **Location:** [sim/crates/bb_game/src/live.rs:3769-3773](file:///home/natex/Projects/3D%20Baseball/sim/crates/bb_game/src/live.rs#L3769-L3773) and [sim/crates/bb_game/src/live.rs:5080-5115](file:///home/natex/Projects/3D%20Baseball/sim/crates/bb_game/src/live.rs#L5080-L5115)
   - **Concrete case:** When a batted ball bounces short of `BASE_L` (`27.432 m`) and then passes over a foul-territory wall or enters a dugout (`SegmentEnd::OverFence { fair: false }`) before `s = BASE_L`, `batted_path_dt` continues the trajectory inside `into_the_stands` (`bb_physics/src/ground.rs:486`) until `t_end`, and `ball_facts` does not emit `Touch::OutOfPlay` until `t >= path.t_end()`. If the ball's in-stands path crosses `s = BASE_L` before `t_end`, `passed_base` emits `Input::BallPassedBase { t, fair: false }` while the ball is already out of play in the stands/dugout. `bb_rules::engine::decide` sets `st.play = PlayState::Dead`, ending `simulate_live` before `path.t_end()`. No `Touch::OutOfPlay` fact, `Event::BallDead { reason: OutOfPlay }`, or `MarkerKind::BallDead` marker is emitted (`dead_reason = 0`), so the client never cuts back from the stands or skips the reset.
   - **How checked:** Ran `simulate_live` in a throwaway probe (`review_probe.rs`, deleted after verification) with `BallState { r: (0.0, 0.5, 0.5), v: (35*cos 12°, 35*sin 12°, -14.0), w: ZERO }` and `seed = 4`: the ball bounced at `(1.12, 0.74)`, `(16.95, 4.11)`, `(26.01, 5.96)` (`s = 22.61 m`), entered the dugout, and emitted `BallPassedBase { t: 1.841, fair: false }` before `t_end = 1.895` (`p_end = (32.72, 7.33, 0.04)`, `s = 28.32 m`), yielding `events = [FairFoulDecided { fair: false }, PitchResult { call: Foul, balls: 0, strikes: 1 }]` with no `BallDead`.

---

### P3

1. **Discontinuous `p.x * side < 0.0` guard can emit `BallPassedBase` near second base or redundantly after a `Touch::Ground` beyond `BASE_L`**
   - **Location:** [sim/crates/bb_game/src/live.rs:5087-5096](file:///home/natex/Projects/3D%20Baseball/sim/crates/bb_game/src/live.rs#L5087-L5096) and [sim/crates/bb_game/src/live.rs:3767-3773](file:///home/natex/Projects/3D%20Baseball/sim/crates/bb_game/src/live.rs#L3767-L3773)
   - **Concrete case:** `past(t, side)` returns `f64::NEG_INFINITY` whenever `p.x * side < 0.0`. If a ball bounces in the infield (`s < BASE_L`) or bounces beyond `BASE_L` in the same tick near `x = 0` (e.g., around `(-0.1, 38.9)` to `(+0.1, 39.0)` near second base, where the two perpendicular bag lines intersect at `(0, 38.795 m)`), `past(t0, 1.0)` is `-inf` while `past(t1, 1.0)` is `>= 0.0`, so `passed_base` fires at `x = 0` (near 2B) or emits a redundant `BallPassedBase { fair: true }` in `out.facts` after `Touch::Ground` already decided fair earlier in the same tick. (Rules outcome is unaffected because `bb_rules` ignores `BallPassedBase` when `lp.fair != Pending`, and near 2B any crossing is fair.)
   - **How checked:** Code inspection of `live.rs:3767-3817, 5087-5113`. Across 50 simulated games (`969` `BallPassedBase` facts), `0` crossed within `|x| < 1.0 m` and `0` were preceded by an earlier beyond-bag `Touch::Ground`.

2. **Airborne threshold `p.z > r_ball + 0.01` leaves a 1 cm band above ground judged by center**
   - **Location:** [sim/crates/bb_game/src/live.rs:5110-5111](file:///home/natex/Projects/3D%20Baseball/sim/crates/bb_game/src/live.rs#L5110-L5111)
   - **Concrete case:** On `Segment::Roll`, `p.z` is set identically to `r_ball` (`bb_physics/src/ground.rs:446`). A ball bounding across the bag line at height `r_ball < p.z <= r_ball + 0.01` (up to `1 cm` clear of the ground) has `airborne == false` and is judged by `line >= 0.0` instead of `line >= -r_ball`.
   - **How checked:** Inspected `live.rs:5110-5111` and `bb_physics/src/ground.rs:446`.

3. **Glove-rim deflection resets `BallPath` without notifying `bb_rules` or preserving prior ground-touch state for `passed_base`**
   - **Location:** [sim/crates/bb_game/src/live.rs:3702-3710](file:///home/natex/Projects/3D%20Baseball/sim/crates/bb_game/src/live.rs#L3702-L3710) and [sim/crates/bb_game/src/live.rs:5081](file:///home/natex/Projects/3D%20Baseball/sim/crates/bb_game/src/live.rs#L5081)
   - **Concrete case:** When a ball deflects off a fielder's glove rim (`POCKET < d <= RIM`), `try_captures` calls `self.set_free(s0)` without emitting a glove-touch fact to `bb_rules`, and replaces `self.ball`'s `BallPath`. If the ball had already bounced before the deflection and is popped over 1B/3B without bouncing again before the bag, `path.bounces` on the new `BallPath` is empty, so `passed_base` returns `None`. (Pre-existing rim-deflection omission; rules-spec §2 lists `Glove{agent}` under `BallTouched`, which `bb_rules::Touch` does not yet implement.)
   - **How checked:** Inspected `live.rs:3702-3710, 3739-3747` and `bb_rules/src/state.rs:226-239`.

4. **`docs/data-formats.md` not updated for `MarkerKind::BallDead.base() == 4`**
   - **Location:** [docs/data-formats.md:384](file:///home/natex/Projects/3D%20Baseball/docs/data-formats.md#L384)
   - **Concrete case:** `docs/data-formats.md` §5.4 still states that `MarkerKind::BallDead.base` uses `DeadBallReason` numbering (`1` out of play for a foul ball or a throw, `2` home run, `3` ground-rule double), without noting presentation code `4` added in `timeline.rs:337` and `bb_ffi.h:54-55` (CLAUDE.md Invariant 7).
   - **How checked:** Read `docs/data-formats.md:384` and compared with `timeline.rs:330-340` and `bb_ffi.h:54-55`.

5. **Minor client state hygiene: `bFoulOut` not reset in `BeginPlayback` when `!Current.bValid`**
   - **Location:** [client/Source/BaseballClient/BbDirector.cpp:588-606, 625](file:///home/natex/Projects/3D%20Baseball/client/Source/BaseballClient/BbDirector.cpp#L588-L606)
   - **Concrete case:** `BeginPlayback` resets `bBallInPlay`, `bBallDead`, `bDeadCut`, `bHomeRunCut` before the `if (!Current.bValid)` early return, whereas `bFoulOut` is assigned only at line 625. Harmless because `bGameOver = true` halts playback, and a foul out of play cannot end a game.
   - **How checked:** Inspected `BbDirector.cpp:588-630`.

---

## Verification

1. **Requested tests and probe:**
   - `cargo test --release -p bb_game --lib -- a_ball_rolling_past_third describe`: **3 passed, 0 failed** (`caption_uses_log_scorer_when_provisional_classifier_disagrees`, `plays_read_the_way_they_are_called`, `a_ball_rolling_past_third_is_judged_as_it_passes_the_bag`).
   - `cargo run --release -q -p bb_game --example foul_caption_probe -- 3647 15 20 62`:
     - Play `15`: `call Foul`, `ev FairFoulDecided { fair: false }`, `PitchResult { call: Foul, balls: 1, strikes: 2 }`, 4 ground touches, `BallPassedBase { t: 1.5346490758823592, fair: false }`.
     - Play `20`: `call InPlay`, `ev FairFoulDecided { fair: false }`, `PitchResult { call: Foul, balls: 0, strikes: 1 }`, 5 ground touches, `BallPassedBase { t: 1.4442206282624885, fair: false }`.
     - Play `62`: `call Foul`, `ev BallDead { reason: OutOfPlay }`, `FairFoulDecided { fair: false }`, `PitchResult { call: Foul, balls: 0, strikes: 1 }`, `BallTouched { t: 4.0024378540580425, what: OutOfPlay { x: 26.00293508470218, y: 4.502488096015948, z: 16.969793920640726, in_flight: true } }`.
   - `cargo test --release -p bb_ffi`: **8 passed, 0 failed** (`plays_a_game_through_the_abi` and 7 other FFI tests).
2. **Additional tests and probes run:**
   - `sim/crates/bb_ffi/c/run_smoke.sh` (seed `3`): **FAILED** (exit code `1`), printed `bad dead ball` (Finding P2-1).
   - `cargo test --release -p bb_rules`: **1 passed, 0 failed** (`scenario_corpus`).
   - `cargo test --release -p bb_game --test play_area`: **1 passed, 0 failed** (`balls_out_of_play_end_the_play_and_fielders_stay_inside_the_walls`).
   - Throwaway probe `examples/review_probe.rs` (deleted after use):
     - Scanned 50 full games (`seeds 0..50`): `passed_total=969 (fair=871, foul=98)`, `redundant=0`, `warnings=0`, `caught_foul_no_pa_end=0`, `caught_foul_throw_oop=0`, `0` crossings with `|x| < 1.0`.
     - Synthetic sweep reproduced Finding P2-3 (9 cases, e.g. `sp=35 ang=12 vz=-14 seed=4`: `BallPassedBase` at `t=1.841` in the stands/dugout before `t_end=1.895`, `BallDead` omitted).
     - Synthetic `timeline::encode` on caught foul fly + `BallReleasedFromGlove { Throw }` + `BallDead { OutOfPlay }` reproduced Finding P2-2 (`BallDead.base() = Some(4)`).
3. **Unreal client (`BbDirector.{h,cpp}`):** Not compiled (per review request); verified `#include` paths, class/member declarations, and parameter names against `~/UnrealEngine/5.8.4/Engine/Source/Runtime/Engine/Public/Materials/` (`Material.h`, `MaterialExpressionConstant2Vector.h`, `MaterialExpressionDistance.h`, `MaterialExpressionDivide.h`, `MaterialExpressionMultiply.h`, `MaterialExpressionOneMinus.h`, `MaterialExpressionSaturate.h`, `MaterialExpressionTextureCoordinate.h`, `MaterialExpressionVectorParameter.h`) and `BbPark.cpp`.

---

## Resolution

Run headless by the lead (Claude Opus 5.5) with `agy --model gemini-4-hydrogen-high` on the owner's request; the reviewer changed no file but this one. Every finding was checked against the code before acting.

- **P2-1, fixed.** `sim/crates/bb_ffi/c/smoke.c` accepts dead-ball kind 4; `run_smoke.sh` passes again.
- **P2-2, fixed.** `timeline.rs` marks kind 4 only when the rules scored the pitch a foul (`PitchResult { call: Foul }`), which only an uncaught foul does: a caught foul fly is an out (`decide` returns before `apply_call`), so a throw out of play after it stays kind 1. No new test: the reviewer's case did not occur in 50 games, and the condition is one rules event.
- **P2-3, fixed.** `passed_base` takes the field and sends no crossing made outside it (`ball_clearance(...).dist < 0`), so a bouncer that goes into a dugout or over a foul-ground wall short of the bag is dead out of play where it left. Test `a_ball_gone_out_of_play_short_of_the_bag_does_not_pass_it` runs the reviewer's ball through the live play over 300 seeds; with the check switched off it fails on seed 8 (crossing at 1.855 s from inside the first-base dugout), with it on it passes and every such play ends dead out of play.
- **P3-4, fixed.** `docs/data-formats.md` §5.4 gives kind 4.
- **P3-5, fixed.** `BeginPlayback` resets `bFoulOut` with the other per-play flags.
- **P3-1, deferred.** The side guard can only misfire at x = 0 near second base, where any crossing is fair, or after the ball was already ruled; the rules ignore a `BallPassedBase` once the ball is ruled. None in the reviewer's 969 crossings over 50 games.
- **P3-2, deferred.** A ball within 1 cm of the ground as it crosses is judged as touching it: the band is smaller than the ball's own radius, and the outcome differs only for a ball within 3.7 cm of the line.
- **P3-3, deferred.** A glove-rim deflection that sends no touch to the rules is older than this change and belongs to the rules-spec §2 `Glove` touch, not yet built.
