# Day-session review — 2026-10-31

Reviewed `a97783c..bafcd5f`, after reading `CLAUDE.md` and the relevant invariants/specs. Found one P1, three P2s, and one P3. Tests and the game sample pass, but legal tag-ups are repeatedly reversed, the closest-approach guard can discard catchable balls, and the new hit/FC labels are unreliable. No source, test, spec, or TASKS files were changed.

## Findings

### 1. [P1] Retouch is rearmed after every legal departure following a catch

**Location:** `sim/crates/bb_game/src/live.rs:879`, `:930`.

The condition tests whether a catch ever occurred and the runner is beyond his starting base; it does not test whether he actually owes a retouch. `tag_up` is cleared when the catch occurs, and `retouch` is cleared on returning. Consequently both a legal tag-up and a departure after completing a required retouch trigger another return on the following tick. This contradicts the live-review Resolution's finding 7 marked “Fixed.”

**Executed scenario:** A 10 m/s runner legally tags from third after a catch at 120 m in center. Successive calls to `move_runners` put him 0.002778 m forward, back on third, 0.008195 m forward, and back on third, emitting repeated `BaseLeft`/`BaseTouched` facts. The loop continues while the send decision remains favorable; ordinary tag-up advancement cannot complete.

**Suggested fix:** Determine the retouch obligation once when the catch is observed. Clear it permanently for that catch after retouching, and never create it for a runner who stayed on his base until the catch.

### 2. [P2] Capture deferral allows a wall contact to invalidate an available fly out

**Location:** `sim/crates/bb_game/src/live.rs:603`.

The deferral guard checks intervening ground contacts but not walls. A post-carom position closer to the fielder can therefore postpone a valid airborne capture until after the wall has made a catch illegal.

**Executed scenario:** At 2.000 s, a stationary, perfect-hands fielder stands 0.02 m inside the center-field wall. A ball at height 1.3 m is 0.13 m from him, traveling toward the wall at 10 m/s. Capture is skipped; the ball hits the wall at 2.015006 s and is secured at 2.016667 s. Facts correctly put the wall first, so no `Catch` occurs. The guard has turned an available out into a live carom.

**Suggested fix:** Bound any deferred capture interval by the next wall, ground, or out-of-play event; attempt the available capture before a boundary invalidates it.

### 3. [P2] Closest approach is measured against a frozen fielder position

**Location:** `sim/crates/bb_game/src/live.rs:602`, `:607`.

The ball's next-tick position is compared with the fielder's current position, although `move_fielders` runs before the next capture test. The ball can be approaching that old position while moving away from the actual fielder. A guaranteed current capture can consequently be skipped and lost.

**Executed scenario:** A perfect-hands fielder moving at 8 m/s away from a 1 m/s free ball has the ball 0.95 m behind him at catchable height. The guard skips capture because the ball approaches his old position. After movement, separation is 1.066667 m, outside the 1 m reach; the next capture test also fails.

**Suggested fix:** Find closest approach using relative ball/fielder motion over the interval, including the fielder's movement and reach window. Do not postpone a current contact unless the later contact remains physically available.

### 4. [P2] Hits followed by baserunning outs are classified as outs or fielder's choices

**Location:** `sim/crates/bb_game/src/game.rs:58`, `:268`; `sim/crates/bb_cli/src/game_lab.rs:101`.

`batter_safe && outs == 0` is not a hit definition. A batter can earn a hit and then be tagged stretching it, or remain safe while another runner is thrown out advancing on that hit. The latter is currently reported as an FC. Thus the report is not comparable to MLB BABIP even without errors.

**Executed scenarios:** In seed 0, play 37, batter 2 reaches first at 5.010797 s before the center fielder first secures the ball at 5.044130 s, then is tagged at second: `hit()` is false. In seed 2, play 209, batter 265 reaches first at 5.051018 s before the right fielder fields at 5.584352 s; runner 263 is subsequently tagged at home: the report calls this an FC rather than a hit with an advancement out.

**Suggested fix:** Preserve first-base attainment, defensive choices, and subsequent advancement outs separately, then derive hit/FC scoring from the ordered play facts. Until that exists, label these categories as safe-with-no-outs/safe-with-outs rather than hit/FC. Counting `outs >= 2` itself is correct for the stated “2+ outs” category.

### 5. [P3] Reconsidering an advance resets the same perception streams

**Location:** `sim/crates/bb_game/src/live.rs:875`, `:937`, `:826`, `:323`.

The new waiting/retouch paths can call `runner_decide` repeatedly for the same next base, but RNG sub-indices remain just `next` for the timing error and `100 + next` for the free-ball read. Each new decision resets those streams and reproduces the same standardized draws. This is deterministic, but does not separate repeated perception samples as required by data-formats §2.3.

**Code-derived scenario:** A runner first considers second while blocked, then reconsiders after the runner ahead clears it. Both decisions use timing sub-index 2 and, with a free ball, read sub-index 102. Repeated retouches reuse them again. Throw counters do not have this problem.

**Suggested fix:** Allocate a per-runner decision counter with disjoint sub-indices for each decision's read and timing error. If persistent perception error is intended, model and document it explicitly rather than resetting an observation stream.

## Verification

- Ran `git log --oneline a97783c..HEAD`, `git diff a97783c..HEAD -- sim/`, and the changed documentation diff.
- `cargo test --workspace --release`: **90 passed, 0 failed**, including determinism and crate-boundary checks.
- `cargo build --release -p bb_cli`, then `./target/release/sim-cli games --n 128`: successful; **5.09 runs/team-game**, no rules warnings, including no timeout warnings. Reported GB hit/FC rates were .347/.190, subject to finding 4. The documentation's earlier 5.6 figure is not the current result.
- `cargo run --release -p bb_game --example gb_through -- 64`: successful; routine grounders **639**, **89.5%** converted to an out, below the diagnostic's 93% envelope floor.
- Temporary Rust probes under `/tmp/baseball-day-review`, run offline in release mode: reproduced findings 1–3 using copies of the unchanged live source with appended drivers; reproduced finding 4 through the actual `bb_game::Game` API. No repository implementation files were modified.
- Batter probes: **180** analytic-Jacobian entries matched central differences, including negative progress and times beyond the nominal span; **9,000** finite covariance entries matched the old calculation bit for bit; arrival matched the posterior calculation. Six ordinary contact-depth queries matched old Some/None results within 0.1 mm and 10 µs. Endpoint semantics are not identical: querying the exact mid-flight starting depth returns `Some` now, whereas the old strict-crossing march returned `None`.
- Aim probes: **700** comparisons with the old full-step solver across all ten pitch types, 35 targets, and two initialization modes agreed on acceptance. Every accepted answer's residual was independently confirmed at `DT_PITCH`; maximum was 9.98e-6 m. Seeded/unseeded scouting crossings were identical for four arsenal types with nonzero model offsets.
- Code inspection: recall reuses an exact same-tick prediction/intercept because no intervening operation changes that observer's inputs. The single intercept scan preserves standing-before-dive priority, and `t >= start` preserves the reaction gate when distance is already within reach. No new outcome roll or receiver-choice deadlock was established; the finite play limit does not itself prove liveness for arbitrary attributes.
- The 45 m settle radius and the documented aim/Jacobian implementations match code. The Resolution's acknowledged M5 observation-boundary limitations remain: `visible_target` and blocking still infer direction from private goals rather than an observation snapshot. Passing and shared-base entitlement also remain deferred; zero warnings do not establish those invariants.

## Resolution

Resolved 2026-11-11. All five findings fixed, each with a regression test (`bb_game` unit tests in `live.rs` and `game.rs`); `cargo test --workspace --release` passes.

| # | Status | What changed |
|---|---|---|
| 1 | Fixed | The retouch obligation is settled once, on the first tick a runner sees the catch (`Runner::catch_seen`): off his base then ⇒ he owes a retouch; on it (tagging or just standing) ⇒ he owes nothing. The per-tick re-arming condition is gone. Tests: `legal_tag_up_from_third_scores` (leaves third once, touches home, no rules warning), `runner_off_base_at_the_catch_retouches_once` |
| 2 | Fixed | A capture is put off a tick only if the ball touches nothing before the next tick: no ground contact, no wall contact, and the path does not end (over the fence, at rest). Test: `fly_catchable_just_before_the_wall_is_caught` (the review's scenario: `Catch`, no wall fact) |
| 3 | Fixed | The deferral compares against where the fielder will be next tick (`fielder_step`, the same function `move_fielders` uses, so the position is exact) and requires the ball to be closer **and still within reach** there. Tests: `ball_in_reach_of_a_fielder_moving_away_is_caught_now` (the review's scenario), and `standing_catch_happens_at_closest_approach` guards the §5.4 behavior the deferral exists for |
| 4 | Fixed (interim ruling) | `BipDetail` now carries `reached_first` and `hit`, derived by `bb_game::game::classify_bip` from the play's ordered facts and rules events: the batter reached first unless put out before the bag (fly out, force, tag short of first); it is a hit unless another runner was forced out (OBR 9.05(b)(1)) or tagged out before the batter touched first. Outs on the bases after the batter reached leave the hit standing. This is still not the official scorer (rules-spec §10, no errors or `ScorerEvidence` yet) and the doc comment says so. Test: `interim_hit_rulings_follow_the_scoring_rules`, including both of the review's plays as synthetic cases |
| 5 | Fixed | Each advance decision takes fresh draws: decision *n* (from 1) uses `RunnerPerception` sub-index 2*n* for its timing error and 2*n* + 1 for its read of a free ball; sub-index 1 is the first read at contact. Documented on `Runner::decisions` and in data-formats §2.3. Test: `repeated_runner_decisions_draw_fresh_reads`. This changes the replay stream of every play with a runner decision (no released logs depend on it) |

**Effect on the numbers** (`sim-cli games --n 512 --seed 2`): 4.98 runs/team-game (4.91 before; tag-up advances now complete, and the runner noise streams changed), no rules warnings. With the hit ruling corrected, ground balls are hits .398 of the time and fielder's choices .161 (the old labels read .331 / FC); LD .710, FB .136, PU .015 against MLB ≈ .68 / .13 / .02. So the ground-ball excess is larger than the old label showed, and it is the whole of the run-environment gap. Routine-grounder conversion is unchanged at 88.6% (`gb_through`, 128 games).

**Not verified:** the new tests were not run against the pre-fix code; the review's own probes are the evidence that the old code failed these scenarios.
