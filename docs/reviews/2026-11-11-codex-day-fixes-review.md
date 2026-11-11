# Day-review fixes — independent review, 2026-11-11

Reviewed **`9660aab..c31b227`**, after reading `CLAUDE.md`, the earlier day review and its Resolution, and the relevant specs. Locations below refer to **c31b227**, regardless of subsequent work in this shared checkout.

The legal-tag-up loop and the two original capture examples are fixed. The reported aggregate numbers reproduce. However, the retouch repair still misses a runner who has crossed home and uses the wrong reference event after a deflection; capture deferral still has an available-catch loss; and the new scoring rule is neither OBR-correct nor reproducible from the persisted event log. Findings 1–3 identify remaining holes in the claimed repairs, including paths already present in the parent, rather than asserting that this commit introduced every underlying problem.

## Findings

### 1. [P1] A home touch prevents the runner from ever observing a later catch

**Location:** `sim/crates/bb_game/src/live.rs:863–880`, also `:967–976`, `:1018–1022`.

`move_runners` skips `scored` runners before evaluating `catch_seen`. But `scored` is set on the physical home touch, while the rules engine treats that run as pending. A runner who crosses home before a catch therefore never acquires the agent-side retouch obligation, never returns, and is excluded from the holder's tag and throw-option loops. This is the home-base instance of having already passed the next base, and it is not covered by the new once-only check.

**Executed scenario:** Through the unmodified Game API, **seed 59, play 186**, runner **263** leaves third at **0.951199695 s** and touches home at **5.251199695 s**. The fly is caught at **6.101199695 s**. There is no subsequent retouch of third. The defense appeals runner 264 at second at 9.767866362 s, and the events then include **`RunScored { runner: 263 }`**. A separate controlled probe with a home touch before the catch leaves the runner `scored: true, catch_seen: false, retouch: false` throughout 12 seconds of runner updates, although the rules runner owes a retouch.

**Suggested fix:** Distinguish a pending home touch from a completed, irrevocable run. Process caught-fly obligations for these runners, allow their return, and keep them eligible for appeals. Returning must also invalidate/recompute the old pending run in the rules state; merely clearing the simulation's `scored` flag is insufficient because `base_touched` currently ignores runners with `scored_at` set.

### 2. [P2] Deflected flies still use secure possession instead of first fielder touch

**Location:** `sim/crates/bb_game/src/live.rs:667–675`, `:872–880`; `sim/crates/bb_rules/src/engine.rs:499–510`.

The new decision is settled once, but at the wrong event for a bobbled fly. A rim deflection calls `set_free` without emitting a fielder-touch fact. `catch_made` becomes true only on `GloveSecured`, and the rules engine also assigns `first_touch` that later time. Rules-spec §5.3 requires the original fielder touch to be the retouch reference. The [2026 OBR definition of Catch](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf#page=160) likewise permits departure on first touch.

**Executed scenario:** With seed 0 and hands sigma 0.07, a controlled ball at `(0, 80, 1.3)` produces a rim deflection at **2.0 s** (`attempts = 1`, `blocked_until = 2.5`, no emitted fact). A runner on third leaves at **2.1 s**, and another fielder secures the still-airborne ball at **2.2 s**. The legal departure is converted to `retouch: true`, and the rules state records `first_touch: Some(2.2)` and `retouch_required: true`. A subsequent appeal would retire a runner who left legally.

**Suggested fix:** Emit and retain first fielder contact, including deflections, independently of eventual secure possession. When a catch is completed, derive the obligation from the runner's touch/leave history at that first contact, and use the same reference in the runner model and rules engine. Add a deflection → legal departure → catch → unsuccessful appeal regression.

### 3. [P2] Crossing the wall-reach threshold can still discard a certain catch

**Location:** `sim/crates/bb_game/src/live.rs:586–590`, `:611–622`.

The deferral tests the future ball against the **current** `zmax`. But `zmax` changes from 3.0 m to 2.4 m when the ball moves more than 3 m inside the fence. Thus the next position can satisfy the guard and fail the actual next capture test, even with an exactly predicted stationary fielder and no intervening surface contact.

**Executed scenario:** At 2.0 s, put a stationary, perfect-hands fielder at `(0, fence_cf - 3.8, 0)` and a free ball at `(0, fence_cf - 2.99, 2.9)`, moving `(0, -20, 0)` m/s. The current 0.81 m horizontal separation and 2.9 m height permit a standing capture. Next tick the separation is about 0.477 m and the height about 2.899 m, so the guard defers using the 3.0 m limit. On that next tick the ball is more than 3 m inside the fence, so the real test uses 2.4 m and rejects it. The probe emits **no `GloveSecured`** during the next ten ticks as the ball passes the fielder; when it becomes low enough, it is already outside reach.

**Suggested fix:** Evaluate the complete future capture predicate, including the future wall-dependent height limit, rather than reusing `zmax`. Also bound thrown-ball deferral by its ground conversion: `0 <= bn.z < ball.radius` currently passes the height guard although `update_thrown` will replace that predicted trajectory before the next capture test. I did not establish a permanently lost throw from that latter interval, but the promised unchanged next contact is not guaranteed there.

### 4. [P2] First-base arrival time is not the hit/fielder's-choice test

**Location:** `sim/crates/bb_game/src/game.rs:66–106`, `:421–435`; `sim/crates/bb_cli/src/game_lab.rs:92–104`; earlier review Resolution finding 4 and its numerical interpretation.

OBR 9.05(b)(3) excludes a hit when an infielder, pitcher, or catcher handles the batted ball and the defense retires a preceding runner advancing one base or returning. It does not depend on whether the batter touched first before the tag. Conversely, an outfield-hit advancement out does not become an FC merely because its tag preceded that touch. Failed defensive choices also require the counterfactual in 9.05(b)(4), even without errors. These distinctions are explicit in [2026 OBR 9.05](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf#page=128). Acknowledging that errors and an official scorer are deferred does not make the timing substitute valid for the cases it claims to score.

**Executed Game API counterexamples:**

| Seed / play | Actual play facts | Current / expected ruling |
| :--- | :--- | :--- |
| **6 / 112** | Pitcher 129 fields at **0.964778224 s**; runner 257 advances from second toward third; batter 258 touches first at **5.014778224 s**; 3B tags 257 at **5.081444891 s**. | Hit / **FC**, under 9.05(b)(3). |
| **6 / 246** | Pitcher 129 fields at **0.922195016 s**; runner 263 attempts third → home; batter 258 touches first at **5.005528349 s**; catcher tags 263 at **5.088861682 s**. | Hit / **FC**. |
| **18 / 206** | Pitcher 129 fields at **0.905984041 s** and the catcher receives at **2.139317375 s**; runner 265 attempts third → home; batter 259 touches first at **5.005984041 s**; tag at **5.072650708 s**. | Hit / **FC**. |

There is also a separate fact-association error inside this timing heuristic. `tag_time` picks the first held-ball tag fact, not the fact that caused the out. In an executed rules replay, a tag while the runner remains on third at 2.0 s emits no event; the batter touches first at 3.0 s; the runner leaves at 3.1 s and is tagged out at 4.0 s. `classify_bip` returns `(true, false)`. Removing only the ignored 2.0 s tag leaves **identical rules events** but changes the result to `(true, true)`. No such first-tag mismatch was found in the 64 ordinary games inspected, but the public classifier's first-fact assumption is unsound. Equal timestamps also require input order, not just `<`, to identify an earlier fact.

`reached_first` is likewise not an actual attainment flag: **seed 0, play 7** ends on a third-out force before the batter touches first, yet returns `reached_first = true` because the batter himself has no out event. Its field documentation promises more than the implementation records.

**Suggested fix:** Preserve actual base attainment/awards separately from the batter's surviving the play. Implement the applicable scoring evidence from rules-spec §10, including the original handler, the preceding runner's attempted advance, and failed choices. Associate outs with their producing facts. Until that exists, report explicitly provisional diagnostic categories rather than claiming these hit rates are comparable to MLB BABIP. The .398 figure is reproducible but is not a verified ground-ball hit rate, and it does not establish that grounders explain the whole run-environment gap.

### 5. [P2] The new scoring statistics depend on evidence absent from the event log

**Location:** `sim/crates/bb_game/src/game.rs:308–316`; `sim/crates/bb_game/src/eventlog.rs:47–57`, `:122–146`.

`step_pitch` writes the persisted play before calling `classify_bip`, then stores `reached_first` and `hit` only in the accumulating `GameResult`. The classifier consumes `BaseTouched` and `TagContact` input times. `write_pitch` accepts no input facts, does not serialize those contacts, gives rules outs/advances time zero, and writes batted-ball PA results as `Unknown`. Neither the evidence nor the new ruling is persisted. Consequently the advertised hit/FC totals cannot be reconstructed from the existing log without rerunning the simulation. This violates CLAUDE.md invariant 6 and rules-spec §10's pure event-log scorer requirement.

**Code-derived scenario:** Two plays with the same pitch/contact record and ordered `RunnerOut`, `RunnerAdvanced`, and PA events can place the first-base touch on opposite sides of the tag. This classifier produces different hit totals, while the event encoder has no field carrying that distinguishing timing evidence. The existing log round-trip/accounting tests do not check reconstruction of `bip_detail` or its hit/FC counts.

**Suggested fix:** Persist the necessary timed contacts and scoring evidence/ruling, then derive the report from that log with a pure scorer. Add a round-trip assertion comparing hit/FC and first-base-attainment results with reconstruction from serialized bytes. Calling this ruling interim should not waive the event-sourcing invariant.

### 6. [P3] The fresh-draw regression generates its own draws instead of observing the decision's draws

**Location:** `sim/crates/bb_game/src/live.rs:1528–1538`; related retouch coverage at `:1502–1514`.

After calling `runner_decide`, the RNG test independently calls `runner_predict(2*n + 1)` and constructs a timing RNG with `2*n`. It does not inspect the read or timing error consumed by that decision. It therefore proves that different explicitly supplied keys differ, not that production decisions use those keys.

**Executed mutation:** In a temporary copy, leave `decisions += 1` intact but make every production decision use read sub-index **3** and timing sub-index **2**. The unchanged `repeated_runner_decisions_draw_fresh_reads` test still passes. The production implementation currently uses the correct indices; this finding concerns its claimed regression coverage.

The off-base retouch test also passes on the parent. It moves the simulated runner's position without sending `BaseLeft`, so the rules engine never acquires the corresponding debt, and its conservative runner stops after returning. It cannot detect the old re-arming bug after a subsequent legal departure. The closest-approach test also passes before the fix, appropriately serving as a behavior-preservation control.

**Suggested fix:** Observe or inject the actual decision inputs/stream keys, and assert independently specified draws and their order. For retouch, emit a consistent departure history, check the rules-side debt and appeals, then provide a favorable advance after retouch and require completion without another return.

## Verification

- Read `CLAUDE.md` first; inspected `git diff 9660aab c31b227`, the prior review including its Resolution, `TASKS.md`, data-formats §2.3, physics-spec §9.1, agent capture behavior, rules-spec §§5–6 and §10, the rules engine, event encoder, RNG registry, and ground implementation. Consulted the official 2026 OBR for catch/retouch and scoring references.
- Ran `export PATH=$HOME/.cargo/bin:$PATH; cd sim && cargo test --workspace --release`: **97 passed, 0 failed**, including determinism, log round-trip, rules corpus, and dependency-boundary tests.
- Ran `cargo build --release -p bb_cli`, then `./target/release/sim-cli games --n 512 --seed 2`: **4.98 runs/team-game**, **no rules warnings**, GB hit/FC **.398/.161**, LD hit **.710**, FB **.136**, PU **.015**. These match the Resolution and TASKS numerically, subject to findings 4–5.
- Ran `cargo run --release -p bb_game --example gb_through -- 128`: **1,278** routine grounders, **88.6%** converted to an out; below the diagnostic's 93% envelope and 96% target. The through/infield-hit/out totals were **1,108 / 359 / 2,848**. No new claim of routine-grounder improvement is supported or made by these results.
- Independently ran the parent game implementation on the CLI's same 512 seeds, `2 * 1_000_003 + i`, in a temporary copy: **4.91406250 runs/team-game**, zero warnings. This confirms the rounded **4.91 before** figure. The changed perception streams prevent assigning the entire before/after difference specifically to tag-ups.
- Temporary code and outputs reside under `/tmp/baseball-fixes-review`. Additional Game API probes used archived **c31b227** dependencies, and inspected every pitch in **seeds 0–63**. The earlier 16-game output was identical after switching to the archived dependencies. Subsequent commits and unrelated working-tree edits in this shared checkout were not reviewed or modified.
- Built parent `live.rs` in the temporary harness, extracting its unchanged initializer into `new_sim` only to support the added fixtures. Of the five directly portable new tests, **legal tag-up**, **wall capture**, and **moving-away capture** fail on the parent; **off-base retouch** and **closest approach** pass. The new game test is not directly portable because its API did not exist; code inspection of the parent's `batter_safe && outs == 0` establishes failure of its two hit-with-baserunning-out cases. Its new synthetic FC examples do not validate the timing cutoff. The RNG mutation result is described in finding 6.
- Controlled runner probes confirmed return through an already-passed second base to the original first base, and return after waiting on second, each with a consistent rules-fact history. A runner standing on his original base without `tag_up` acquires no debt and remains stopped according to his existing goal. The new legal-tag-up regression demonstrates one departure and a home touch. No ordinary retouch deadlock was established in these cases; the pending-home exception is finding 1.
- Appeals at the original base still work. An off-base returning runner is also retired on a held-ball tag. `holder_act` suppresses a tag while the runner is on another base, even though the rules engine permits that appeal; in the controlled probe the appeal occurs after the runner moves beyond the 0.3 m on-base tolerance. The new tests omit holder behavior entirely.
- For an unchanged free/thrown ball, `fielder_step(fd, self.t, ...)` predicts the movement actually performed at the end of this tick. Planning, `update_thrown`, and close re-reading precede that prediction; `holder_act` returns while the ball remains free/thrown. Recovery and the reaction gate use the same current time. Moving the `dove` reset ahead of the reaction gate changes the flag only for a recovered dive before initial reaction, a state normal capture gating cannot produce; no reachable movement change was found. Another fielder's intervening capture/deflection can of course change the ball and assignments.
- Free-path ground, wall, and path-end checks now prevent the originally reported boundary losses. Controlled thrown-ball probes also capture immediately when next height would be below zero or above the ordinary height limit. These checks do not cover the wall-height threshold loss in finding 3, nor prove safety across a thrown-ball ground conversion.
- Current runner RNG allocation is disjoint within the live play: initial read **1**, decision timing **2n**, decision free-ball read **2n+1**. Every valid decision increments its counter, including decisions without a free-ball read. Timing draws one truncated normal; a free-ball read draws depth, lateral, velocity-depth, velocity-lateral errors in that order. The old skeleton's `RunnerPerception` use in `play.rs` is a separate simulation path, not another consumer in a live play. Purpose values were not renumbered. The sub-index convention is documented in data-formats §2.3; the four-component read order is apparent in the implementation rather than spelled out there.
- The physics table edit matches `ground::DIRT.mu_r = .22` and `GRASS.mu_r = .28`. The new paragraph correctly acknowledges that all grass currently uses the same implemented surface, so the outfield/warning-track/turf rows are not active code values. The retained older parenthetical saying “the values above” allowed a 250 m roll is now stale wording because the table itself has been updated. Aggregate agreement and zero warnings do not establish complete retouch legality, scoring correctness, or the causal “whole gap” claim.

## Resolution

Resolved 2026-11-11: three findings fixed, three deferred to the rules-engine and scorer work they depend on. `cargo test --workspace --release` passes.

| # | Status | What was done, or why it waits |
|---|---|---|
| 1 | Deferred (M4 rules) | A runner who touched home before the catch never goes back. Fixing the agent alone is not enough: as the review says, the rules engine ignores base touches by a runner with a pending run, so a return has to be supported there first (un-scoring a pending run). The result today is legal but poor play by the runner: he stays put, and the defense can still put him out on appeal at his original base. Listed first under the rules-engine item in TASKS |
| 2 | Deferred (M4 rules) | The retouch reference should be the first fielder touch, not secure possession. There is no fact for a fielder touching the ball without securing it (`Touch` has no fielder variant), so this needs a new rules input, the engine's `first_touch` set from it, and the schema. Already on the list as "first-touch facts"; now tied to this finding |
| 3 | Fixed | The deferral judges the next tick's ball by the height limit where the ball will be (`zmax_at`), and a throw about to reach the ground is not deferred. Test: `catch_is_not_deferred_out_of_the_wall_leap_zone` (the review's scenario) |
| 4 | Fixed in part | `classify_bip` no longer uses first-base arrival time. A hit is now refused when another runner is forced out (9.05(b)(1)) or when the pitcher, catcher, or an infielder first fielded the ball and another runner was put out (9.05(b)(3)); an out on a ball first fielded by an outfielder leaves the hit standing. The first-tag-fact lookup is gone. `reached_first` now needs the batter to have touched first (or been awarded it), so a batter short of first at a third out elsewhere has not reached. Tests: `provisional_hit_rulings_follow_obr_9_05`, with the review's seed 6 play 112 and seed 0 play 7 as cases. Not done: the "advancing one base or returning" qualifier and the failed choices of 9.05(b)(4), which need `ScorerEvidence`. The CLI and the doc comment now call the ruling provisional and say it is not the scorer's |
| 5 | Deferred (scorer) | `BipDetail` is a lab diagnostic in `GameResult`, like the `batter_safe` and `outs` fields it sits beside, and its doc comment now says it is not a statistic. Making hits reconstructible from the log is the scorer item already open in TASKS (rules-spec §10: persist `ScorerEvidence` and rulings, derive in `bb_stats`); no statistic in `bb_stats` uses this ruling |
| 6 | Fixed | The RNG test now reads back the timing draw each production decision actually used (`Runner::last_timing_draw`) and requires four decisions to differ; the review's mutation (constant sub-indices) fails it. It does not observe the free-ball read's stream. The retouch test now tells the rules engine about the departure, and requires one retouch followed by a completed advance with no second return (`runner_off_base_at_the_catch_retouches_once_then_advances`) |

**Numbers** (`sim-cli games --n 512 --seed 2`, after these fixes and the fielding changes of `82dbdf7`): 3.87 runs/team-game, no rules warnings; provisional hit rates GB .287, LD .691, FB .121, PU .018; GB fielder's choices .071. The earlier Resolution's ".398 ground-ball hit rate" and "grounders are the whole gap" were read off the old ruling and are withdrawn; see docs/calibration/2026-11-11-fielding-planner.md for the current picture.
