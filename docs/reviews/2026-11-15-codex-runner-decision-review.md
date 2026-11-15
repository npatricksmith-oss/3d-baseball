Request changes: home clearance permits an avoidable passing out, and the advance values omit a lead run that a third out can erase.

## What was reviewed

`a9b3341..b36a428` on `work/runner-decision` (implementation `0da6c5e`), after reading `CLAUDE.md`: agent-models §6.3's “Built 2026-11-15” paragraph; the design note §§3 and 9; the metrics ledger row; `running.rs`, the named runner functions and first-look gate in `live.rs`, `live/holder.rs::holder_options`, the probe, and the changed input construction sites. No implementation, test, spec, ledger, or TASKS file was edited.

## Findings

### P1 — Allowing the path home needs a following-distance rule

**Location:** `sim/crates/bb_game/src/live.rs:4127`, with movement at `live.rs:4575`.

The new exception opens home as soon as the runner ahead is more than 0.5 m past third and moving forward. Nothing then prevents the trailing runner from overtaking him. `settle_guard` deliberately excludes home; `base_held_ahead` does not block a runner visibly moving forward. The rules correctly retire the passer, but the running decision creates an out a runner watching the man immediately ahead would avoid.

**Evidence:** disposable probe `review_home_following`: runner 7 arrives at third at 8.1 m/s; runner 8 is one metre ahead at 1.0 m/s, heading home. Both have the normal 8.2 m/s maximum and league acceleration. With the ball held in center at 115 m and the holder recovering for another 0.6 s, `runner_decide` sends runner 7 home and reports no blockage. Moving and sampling the runners for one second produces `RunnerPassed { runner: 7, passed: 8 }` and `RunnerOut { runner: 7, base: 4, kind: Tag }`, without a rules warning. The original home-clearance predicate would have held him at third.

**Suggested fix:** retain permission to follow a runner home, but regulate departure/speed using the visible gap, signed velocities, and braking distance so he follows without passing. Extend the new home-clearance test through movement and rules sampling, including a slow lead runner and a lead runner turning back.

### P2 — A runner heading home is not necessarily a common, ignorable run

**Location:** `sim/crates/bb_game/src/live.rs:4167`, and the value construction at `live.rs:4181`.

When another runner's visible destination is home (`seen == 4`), he contributes neither occupancy nor a run to any alternative. Omitting him as a common constant works if his run counts under every alternative. With two outs, an out on the deciding runner before that home touch instead loses the lead run. The resulting break-even understates the cost of the third out.

**Evidence:** disposable probe `review_pending_run_value`, with two outs after the batter is retired: a runner from first is rounding second; the lead runner is one metre past third and moving home. The function returns `{ hold: 0.33, safe: 0.38, out: 0.0 }`, break-even **86.84%**. The same motion/ball estimates put the deciding runner at third after 3.355 s, the tag there after 3.617 s, the lead runner home after 3.887 s, and the ball home after 4.726 s. Thus holding preserves a lead run, whereas an unsuccessful advance makes the third out before it scores. The appropriate values are `{ hold: 1.33, safe: 1.38, out: 0.0 }`, break-even **96.38%**. At the actual seeded timing read (margin 0.425 s), `runner_decide` goes; the corrected values hold. This is a change in the decision, not merely a difference in absolute expectancy.

**Suggested fix:** include expected home arrivals in the alternatives, using the runner's estimates of home and out times and the third-out exceptions. Omit only runs guaranteed to count in every alternative. Add direct `advance_values` tests distinguishing a lead run arriving before versus after the possible third out, as well as an already completed home touch.

### P3 — The calibration claim is stronger than the recorded work

**Location:** `sim/crates/bb_agents/src/running.rs:141`; `docs/agent-models.md:671`.

Both say the default caution is fitted to MLB attempt and success rates. The note's §9 instead explicitly puts fitting caution after resolving the race discrepancy; the ledger says it is fitted only to outs, not attempts. The reproduced scoring advances have roughly 99% success among attempts and substantially different attempt rates from the supplied MLB targets. Matching aggregate outs on the bases does not establish the claimed fit.

**Suggested fix:** label the current default provisional and state precisely which quantity informed it. Preserve the documented next step of resolving the race, then fit and report attempts and success together by situation/outs before claiming that owner requirement complete.

## Verification

All cargo builds, tests, and simulations ran in `sim/`, prefixed with `taskset -c 0-7 nice`, with `CARGO_BUILD_JOBS=2` and the requested PATH/Zig cache settings; one heavy job ran at a time. Artifacts are under `sim/target/runner-review-*`.

- `cargo test --workspace --no-default-features`: **428 passed, 0 failed, 4 ignored**. An initial default-feature build was stopped before tests to avoid building the optional Rerun dependency tree. The completed pass includes the four new live tests, `advance_tests`, rules scenarios, crate-graph checks, and `same_seed_same_digest_across_threads` (seed 42 twice, identical digest and log).
- Two focused probes in a disposable copy of `bb_game` under `sim/target/runner-review-probe`, with the reviewed decision code unchanged: **2 passed**, asserting the counterexamples above. Output: `sim/target/runner-review-probes.log`.
- `cargo run --release -p bb_cli --no-default-features -- games --n 300 --seed 0`: matches the ledger at printed precision: runs/team-game **4.11**, PA **38.3**, pitches/game **241**, K **22.1%**, BB **8.2%**, HR/PA **2.86%**, reached/BIP **35.1%**, GB/LD/FB/PU hit rates **.308/.668/.125/.010**. **Rules warnings: `{}`**. Output: `sim/target/runner-review-standard.log`.
- `cargo run --release -p bb_game --example runner_rates -- 300 0`: matches §9 and the ledger: first→third singles **23.5% / 0.9% out** (n=1125); second→home singles **34.2% / 0.2%** (n=523); first→home doubles **54.5% / 0.4%** (n=235, including 167 with two outs); tag scoring **34.7% / 37.3%**; outs on the bases/team-game **.273**, split **.063/.022/.083/.105**. These are the same seeds and fielders as the warning-free standard run. Output: `sim/target/runner-review-rates.log`. The seed-1000 runs were not repeated.

The new reads use public scoreboard/rules facts and visible runner motion, not another runner's private goal. The holder estimate retains the existing noisy arm read and visible capture/setup information; no new private plan, true future ball path, execution output, or outcome roll was found. Probabilities select intentions; physical events resolve the races. The catch is stepped through the rules before the tag-up decision, so `st.outs` already includes the catch. Before-first forced movement bypasses the optional value choice; reaching home adds a run, and the out alternative with three outs has zero remaining expectancy.

`with_expected_play` reproduces the admission threshold and p² × weight shape, including force versus tag time. It approximates the holder's timing and omits carries, continuation, and the holder's tie-breakers. `own + CUT_EXTRA_S` is an empirical estimate, not a geometric guarantee: it does not inspect the cutoff position or catchable height. This review established no additional concrete failure from those approximations. `runner_relook` still uses signed-motion travel times; `MAX_TURNS` bounds trapped reversals only, not every reversal when a base is believed reachable. No additional oscillation regression was established.

The batting-side lookup is correct (Top = visitors/index 0); extra innings use the late weight. Positive gain scaling preserves a sure beneficial advance and is coherent as the authorized interim heuristic, while lacking full win-expectancy context. RNG key assignments are unchanged: one timing sample at `2n`, free-ball perception at `2n + 1` (the two relook races reuse that keyed read), and arm reads at `ARM_READ_SUB + fielder`. Changed decisions can change subsequent look counts.

The arithmetic tests have fixed external-table expectations, and the outs ordering/two-out fly tests check useful independent behavior. The expected-play test partly restates the chosen delay formula; the home-clearance test checks a predicate without subsequent motion. Neither covers the two reproduced failures.

## Resolution

Resolved 2026-11-15 (lead, Claude Fable 5.2), in the commit that adds this section.

- **P1, fixed.** `base_uncleared_ahead` lets a runner follow another home only when the man ahead, by a typical runner's legs from his visible position and speed, will be home `FOLLOW_GAP_S` (0.3 s [E]) before this runner's own best time there. The reviewer's case (a metre ahead at 1 m/s) is in `a_runner_ahead_running_home_is_not_in_the_way`. **Deferred:** the test checks the predicate, not movement through the rules sampling, and a runner ahead who turns back after the man behind has left third is not handled (he was not before either); both go with the rundown work, which owns runners trapped between bases.
- **P2, fixed.** `advance_values` takes the time the out on this runner would come. A runner ahead visibly on his way home is a run in hold and safe, and in out unless there are two out and he would not be home first. The reviewer's case and its values (1.33 / 1.38 / 0, break-even over 96%) are `a_run_on_its_way_home_is_lost_to_an_earlier_third_out`, with the after-the-run and one-out cases.
- **P3, fixed.** `AdvanceParams::caution_z` and agent-models §6.3 now say the default is provisional, set by outs on the bases only; the fit to attempts and success waits for the race (note §9).
- **Noted, not changed:** `with_expected_play` omits carries, the double-play continuation and the holder's tie-breaks, and `CUT_EXTRA_S` is an estimate, not geometry (the reviewer found no failure from either); the expected-play test partly restates the delay formula.

After the fixes: `cargo test --workspace --release --no-default-features` 429 pass, 0 fail, 4 ignored. The standard run and the `runner_rates` probe on seeds 0–299 and 1000–1299 are unchanged at printed precision; no rules warnings.
