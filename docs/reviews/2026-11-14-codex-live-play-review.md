# Codex review: final live-play implementation, 2026-11-14

Reviewed `a69dd50..e991794`, with the checkout at `e991794e502a62f8dfa5e270c0e2f9b9a86b8ee2`. This is a review of the combined implementation. I read `CLAUDE.md` first, metrics-ledger §§1 and 5, the requested spec sections, and all seven Gemini reviews and their Resolutions. Findings below distinguish remaining older problems from defects in the new interactions.

The result is **not ready to call invariant-clean**. Rebuilding and replaying succeeded, but that does not establish perception isolation or correct RNG stream allocation. The strongest new defects are the carry continuation's obsolete pivot clock, delayed dead balls after a throw bounces/caroms, wall-clipped dives that still make impossible catches, and overlapping bounce-noise keys. The two run-environment counterfactuals did **not** establish a defect explaining the roughly 1.5-run gap; their measured effects are reported without extrapolation.

## Findings

### P1-1 — Defensive decisions still consume another fielder's private target and uncommunicated belief

**Locations:** `live.rs:1492–1506, 1511–1548, 1680–1684, 1745, 1762–1806, 1813–1817, 1929–1932, 1961–1964`. **Status:** substantially predates this range; remains in the reviewed result and feeds the new option/coverage/pivot logic.

`fielding_point()` returns the primary's **future intended intercept**, `self.f[self.primary].target`, while the ball is loose. Cutoff placement consumes that value; backups read the same target directly. The sign of that private target selects second-base coverage in both cover functions. `recall()` selects its candidates by distance to that target. Its comment calls this something everyone can see, but seeing a teammate's present position and motion does not reveal his exact future destination. `relay_job()` also compares the proposed relay man's private `t_int` with the primary's private `t_int`.

There is a related communication gap in `assign_among`: it ranks every candidate's private intercept/call value, including tier 0 and tier 1, even though the code/spec say those men have not made a sure call. Sure calls could legitimately transmit an estimate; these non-calls do not. The specified 0.3-second communication delay is not implemented by this immediate arbitration. `CALL_SLACK_S` is a confidence margin, not delivery latency.

**Probe:** hold the ball, all body positions/velocities and all runner states fixed; change only CF's target from `(-40,85)` to `(40,85)`. Second's cover immediately changes from fielder index 3 to index 5. No intervening movement or public message occurs. `review_private_target` records this.

Use each observer's own estimate or an explicit communicated claim/assignment. Do not offer a teammate's `target`, `belief` or `t_int` as an observation. The passing crate-graph test cannot detect this: the tactical code is inside `bb_game`, outside the protected `bb_agents` dependency boundary.

### P2-1 — A bounced or caromed throw can remain live after crossing into dead territory

**Locations:** `live.rs:2321–2329, 2341–2375, 2478–2504`; `ground.rs:482–495`.

The direct-flight branch in `update_thrown()` correctly kills a throw at its boundary crossing. After a short hop or a wall carom, however, `set_free()` changes the representation to `Ball::Free` while the rules correctly retain `lp.throw`. `ball_facts()` then waits until `path.t_end()`—landing in the seats/dugout—to emit `OutOfPlay`. It uses the earlier crossing coordinates with the later landing timestamp. The direct-flight review fix did not cover this path.

**Probe:** a released throw, starting a hop 0.3 m inside the dugout opening at line coordinate `s=15 m`, crosses at **2.038788 s** and lands at **2.548893 s**. At **2.293841 s**, `done=false`, with no `BallDead`. Outs, securing, runner movement and scorer touch evidence are still possible in that half-second window. This conflicts with the dead-at-entry requirement for a thrown ball. [2026 OBR 5.06(b)(4)(G)](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf)

Preserve whether a free path originated as a throw and schedule its dead-ball fact at the crossing, regardless of intervening hops/caroms. The visual tail must not extend live play. Add a bounced-throw timing case; awards-021 scripts its own final fact and cannot verify when the orchestrator produces it.

### P2-2 — Clipping a dive to the wall does not clip its catching reach

**Locations:** `live.rs:2125–2146, 2183–2224`; compare the planner's rejection at `1466` and `beyond_reach_over_wall()` at `1125`.

Capture eligibility uses `REACH + ext_at(t)` before the lunge endpoint is clipped to the legal field. After clipping, the code never checks whether the glove can actually reach the ball from the resulting body path. The ball is secured solely from the original distance and glove-placement draw.

**Probe:** put CF 0.31 m inside the dugout boundary, with a committed dive and perfect hands. A descending batted ball is 1.5 m outside that boundary, 1 m high, moving outward at 5 m/s. The ball is **1.81 m from his body**, and the planner's own `beyond_reach_over_wall` is **+0.80 m**. `try_captures()` nevertheless emits a `Catch` and a fly out. Its recorded dive starts and ends at the **same body position** because the wall clipped the entire lunge. The claimed extension never physically happens.

Recompute reach/contact against the constrained lunge before securing the ball. This is an interaction between legal body movement, deferred dead-ball handling of batted flies, and the capture solver. It creates false foul outs and can suppress runs; it does not explain elevated runs.

### P2-3 — Carry-to-second continuation still charges the pivot removed from actual carriers

**Locations:** `live/holder.rs:93–106`; compare `live.rs:2892–2920, 3090–3104` and agent-models §§5.5–5.6.

`if option.base == 2` runs before `else if option.carry`. Consequently every carry to second is forecast with a new `pivot_release()` clock. The actual carrier keeps his original setup clock, as the later pivot review fix and its unit test require. The retained-setup branch in the continuation is unreachable for carries to second, the central 6U-3/4U-3 case.

**Probe:** SS at `(-5,36)`, time 2 s, holding since 1 s, first baseman on his bag, batter 2.3 s from first, runner from first halfway to second. The carry's estimated first out is at **3.227996 s**. Current continuation: second completion **4.953840 s**, `p_second=0.0000000254`. Using the already-spent ordinary setup: **4.054903 s**, `p_second=0.979448`, against a batter arrival of 4.3 s. This nearly deletes the second-out value of an available double play.

Restrict the fresh-pivot estimate to a **feed receiver**, and test the carrier's estimate as well as his eventual release. The 100-game counterfactual below changed 22 games, but its run delta was +0.045 after correction, so this sample does not support attributing the high run environment to this defect.

### P2-4 — Two wall contacts can make successive free paths reuse SurfaceBounce streams

**Locations:** `live.rs:2323–2328`; `ground.rs:429–438, 490–495`.

The callback's `k` counts trajectory segments, including wall caroms. `set_free()` reserves the next range using only `path.bounces.len() + 1`. These are different counters. After two caroms the next path can reuse a sub-index already used by the previous path, repeating its keyed normal draws.

**Probe:** default field/environment, start `(0,-16,1)`, velocity `(10,-10,0)`, zero spin and zero bounce tilt. The path has two wall contacts and three ground bounces. Callback indices are **[2,3,4]**, but the next reservation starts at **4**. With the production first-free-path offset, these are streams 1002, 1003, 1004, followed by a new path starting at 1004. A ground-starting next path immediately reuses that stream. Other reproduced index sequences include `[1,3,4]` with the same overlap. Repeating the first fixture with **production keyed roughness** (seed 11, play 3, starting sub-index 1000) used **[1002,1003,1004,1005]**, four bounces and two walls, then reserved **1005** for the next path; this is not dependent on the zero-tilt control.

This is repeatable correlation, **not nondeterministic replay**. Preserve unique stream allocation by advancing past the highest callback sub-index consumed, or count actual bounce draws separately from flight/wall segments. The richer foul-wall geometry makes the old allocation assumption unsafe. Do not renumber `Purpose` values to fix it.

### P2-5 — “First play” is incorrectly reduced to first throw with no recorded out

**Locations:** `bb_rules/src/engine.rs:330–337`; `LivePlay` in `state.rs`; rules-spec §7.2's corresponding implementation contract.

`lp.throws == 0 && lp.outs.is_empty()` misses an unsuccessful tag attempt or a carry toward a force/tag. A legitimate attempt while possessing the ball is already a play; it need not retire anyone. The authorized MLB-derived interpretation explicitly includes carrying toward a base and unsuccessful swipes. [Baseball Canada Rule Interpretations, §15](https://baseball.ca/uploads/files/2017%20Baseball%20Canada%20Umpire%20Interpretation%20Manual.pdf)

**Probe:** R1 reaches second at 1.0 s; SS attempts the force by touching second with the ball at 1.1 s, too late; SS makes the first throw at 1.2 s while the batter is still short of first. The engine fixes `from_pitch=true`. When that throw goes out, R1 is awarded **third**, rather than home from his time-of-throw second base. The carry-to-throw transition can arise in live play, but the engine receives no fact identifying an unsuccessful carrying attempt.

Track actual attempted plays, independently of outs and throws, and supply the missing fact from live play. A feint must remain distinct. Add an unsuccessful-carry/swipe scenario; the current awards corpus tests throws and successful prior outs, not this distinction. Here the spec's simplification also needs correction.

### P2-6 — The runner's new tag allowance uses initial force membership instead of force at the destination

**Locations:** `live.rs:2561–2566, 2612–2629`.

`forced(k)` answers whether the runner belonged to the initial force chain; it returns true for the batter throughout his life as a runner. `runner_decide()` now uses that answer to omit `typical_tag_s` for **every** later destination. A batter rounding first for second must beat a tag, and so must an initially forced runner taking an additional base beyond his force destination.

**Probe:** batter has reached first (`s=BASE_L`, `last_base=1`), with the next decision aimed at second. `forced(0)` remains true and the new tag estimate is **0**, rather than 0.25 s. The `must` gate already checks `at_base == start`; the tag branch does not.

Determine force for `next`, including whether that forced advance has already been completed. This defect makes some extra-base decisions too conservative; it is not evidence of run inflation. The world's force adjudication is separate and passed the force/tag interaction probe below.

### P2-7 — A pivot can choose 80% effort despite the full-arm pivot contract and forecast

**Locations:** `live.rs:3054–3065`; `live/holder.rs:99`; `bb_agents/src/throwing.rs:178–182`; agent-models §5.5, “No early release”.

The effort match protects the **feed** when it has a likely continuation, but does not protect the **pivot throw** itself. Once the force at second is recorded, that throw to first normally has no continuation and falls into the ordinary least-effort branch. The pivot spec and the holder's pivot estimate both say full arm/normal accuracy.

**Probe:** existing `feed_to_shortstop(base_pos(2), 5.0)` produces a pivot release with `out.effort == [0.8]`. Setup still has no hurry; the speed reduction comes from the later effort decision. Thus “pivot is set” does not imply the specified pivot execution.

Either enforce the full-arm contract or explicitly revise the pivot spec and continuation forecast together. A counterfactual protecting pivot effort changed seven of 100 digests but **no scores or aggregate batting results**. This is a verified contract mismatch, not demonstrated run inflation.

### P2-8 — Stopping walk-off awards deletes the batter's required minimum advance

**Locations:** `bb_rules/src/engine.rs:762–771, 805–808`; awards-014/015 and rules-spec §7.2.

Breaking the award loop prevents extra trailing runs, but also drops every batter advancement if a preceding runner supplies the winning run. In a bases-loaded forced-award finish, the game must await the batter-runner's first-base touch as well as the winning home touch. [2026 OBR 5.08(b)](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf)

**Probe:** tied bottom ninth, bases loaded, ground-rule double before any runner touches another base. Events are `BallDead`, third-base runner advanced home, `RunScored`, PA ended, `GameEnded`. There is **no advancement or touch of first for the batter**, nor a deferred award phase. This is incomplete physical/event evidence even where the final score is right. The two existing walk-off scenarios do not start with bases loaded and explicitly prohibit all batter advancement.

Preserve the minimum required advances/touches before finalization without restoring extra trailing scores. Update the scenario/spec expectation accordingly. This is distinct from deciding how many bases the scorer credits on a walk-off hit.

### P3-1 — Pivot/setup gates still acquire an extra tick from floating-point clock origin

**Locations:** `live.rs:3021–3030`; also the tick accumulation at `3464–3465`.

The 0.25 s tag and 0.2 s hold comparisons gained tolerances, but `since < su.earliest_s` and `since < su.set_s` do not. The new no-hurry pivot makes the same on-grid threshold both gates.

**Probe:** repeatedly add `DT=1/60` until `t-t0 >= 0.8`. Origins 0 and 0.4 need **48 ticks**; origins 1, 2 and 5 need **49**; origin 10 needs 48. At origin 1, the nominal 48-tick elapsed value is just below 0.8, and release is postponed a full 16.67 ms.

This does **not** contradict the successful same-seed replay. It is a time-origin-sensitive physical boundary that can decide a close play. Add translated-origin boundary cases and use a consistent tick/elapsed-time convention for grid-scheduled actions.

### P3-2 — The continuation regression no longer proves that continuation changes the choice

**Locations:** `tests/throw_choices.rs:103–116`; `live.rs:6832–6871, 7024–7078`.

The new 3/140 fixture asserts throw-second, outs `[2,1]`, and `second_out_p > 0.5`, but does not require the continuation term to affect selection. I compiled the integration test from stdin with only its replay parameter changed to `continuation_weight=0.0`; **the test still passed**. This is also explicitly deferred in the Gemini tag-up Resolution, so it remains an open finding rather than a newly discovered resolution failure.

The carrier test checks his eventual release but never checks `force_continuation`, allowing P2-3. The holder/pivot agreement test covers a receiver already on the bag; it does not cover a carrier or the effort reduction in P2-7. Add discriminating constructed cases rather than another fixed game's final out sequence.

## Input audit: Invariant 2

The following is the complete input inventory for the decision paths named in the request. “Public, exact” means a raw world quantity is read directly but the interim model supplies **no observation noise** for it. That is an explicit simplification, not evidence that a noisy observation layer exists. I found no remaining cross-runner `goal` read in the new holder/continuation/effort or runner-ahead decision paths.

| Decision | Input provenance and noise | Assessment |
| --- | --- | --- |
| Holder destination and ETA | `Runner.s`, signed `direction*v`, identity, starting base, out/scored state, visible first-base overrun; `ChoiceParams` moving/on-bag thresholds. `runner_eta` uses observed current speed and **typical** 8.2 m/s/0.68 s acceleration, not the opponent's private `vmax` or `goal`. Public rules/outs reconstruct forces. | Exact public motion, explicitly noiseless under §5.6. The 0.12 s Φ width is uncertainty in the race/value model, not independently sampled position noise. |
| Carry and direct throw time | Holder's own position, velocity, readiness/get-up, catch-facing/motion, diving history, arm/transfer/hand/tag attributes; known base locations; his own deterministic movement/throw model. | Own state/proprioception plus known geometry. No realized future throw trajectory or motor noise draw is consulted. |
| Receiver/cutoff routing | Actual teammate positions; cover/relay role; teammate arm, transfer and tag time; own setup; reference-atmosphere `ThrowTable`. Receiver travel uses typical 8 m/s, τ=0.9, distance less 0.5 m. | Teammate attributes are expressly treated as known in the interim spec; positions are exact public reads. Shared assignments can be public, but their derivation currently includes the private-target/belief leaks in P1-1. |
| Continuation and pivot estimate | Public batter position/speed screened by `visible_target`; own first action's estimated completion; proposed receiver's position and known transfer/arm/hand; typical covering time; modeled approach direction/braking speed or feed-facing direction. | No batter `goal` or future execution read. It is a forecast, not an outcome roll. Its carrier clock is wrong (P2-3); reduced pivot effort is absent from the forecast (P2-7). |
| Chosen effort/hurry | Same option table, own setup and arm, public moving-runner screen, receiver readiness, known tag time; thresholds and continuation probability. | No opponent goal/true future consulted. Effort and hurry multiply in both estimated speed and execution; execution σ multiplies hurry, effort and the slide seam. The full-effort feed/cutoff protections are present. |
| Runner reading a holder | Exact current ball/holder position; observed catch/get-up clock, catch-facing/motion, throwing hand and dive history; position's **grade-50** transfer, not holder's private transfer; holder's scouted arm plus a fixed per-runner/per-fielder/per-play truncated normal, σ=1.1 m/s. | Arm uses `RunnerPerception`, sub-index `2^20 + fielder_index`. Other visible quantities have no separate spatial/motion noise here. The final advance comparison adds its own 0.15 s timing noise. Uses ordinary receiver/batted-ball setup, so it also lacks a special pivot-time belief. |
| Runner reading a thrown/free ball | Thrown: exact current `ball_at(t)` position/velocity, heading inferred geometrically rather than reading `to_base`, typical onward transfer/arm; final timing noise. Free: fresh four-draw noisy read/prediction, then visible fielder positions with typical reach/speed. | The thrown branch is a raw present-state read with scalar timing error, not a noisy kinematic observation. Free-ball prediction applies the modeled perception noise and never copies the world's future path. |
| Runner reading the base ahead | Other runner's identity/start order, exact path position, out/scored state and `still_since`, which is updated from the previous tick's signed physical velocity; 0.2 s observation duration. `base_uncleared_ahead` uses public position alone. | No other runner's `goal`. `still_since` is stored on that runner but reconstructible from visible motion; it is not his intention. Reads are exact and sequential within the tick, not independently noisy. |
| Fielder reading walls | Fixed, known `Field` geometry; own body position/velocity/radius and braking law; own noisy predicted ball state for interception, wall delay, catch ceiling and “beyond reach”. | No wall-perception noise is specified. Knowing park geometry is legitimate. The planner includes braking and rejects unreachable points, but the world capture implementation bypasses the constrained reach (P2-2). |

Other surviving private/truth reads in these files:

- **Other-agent private planning:** the `target`/`t_int` and non-call arbitration consumers listed in P1-1 are the remaining tactical leaks. `Role::Cover/Relay` reads are also direct shared-state reads; they need a public-assignment interpretation, not access to another agent's unannounced intention.
- **True future used in action timing:** `try_captures` at `2108–2120` and `2168–2179` samples `ball_at(t+DT)` and true future ground/wall/end times to wait for a closer catch. For standing glove contact this could be implemented as a world collision solver. The same machinery postpones **stretch/dive initiation**, and its eligibility uses true ball speed/height/range. Thus it is not all safely described as observation-driven agent choice. The dive's “worth it” gate uses belief, but the initiation/closest-approach shortcut still uses truth. This machinery largely predates the range; P2-2 shows a new wall interaction it fails to constrain.
- **Direct present-world reads:** exact runner and teammate coordinates/speeds above; actual thrown-ball kinematics in `ball_time_to`; near, slow-ball chaser retargeting at `3390–3395`; actual ground/catch/possession facts used to change runner plans. These are observable quantities, but are read from `Sim` rather than carried through `bb_observe`. The ball predictor's true current state is acceptable only at the sensor boundary, where its four modeled errors are applied.
- **Not agent information:** `trace_choice` copies `intended_goal` only into a read-only diagnostic; `ScorerThrowFact`/`ScorerCarryFact.batter_goal` are output evidence; `absorb` uses goal/true positions for an umpire margin; `settled` reads goals to test world liveness. None feeds the holder's decision. Trace-on/off equality was independently checked. Scoring's use of goal as an advance-on-throw proxy remains a documented approximation, not a perception input.

## Cross-change checks and rules coverage

| Interaction | Verified result |
| --- | --- |
| Effort × hurry × pivot | Ordinary effort selection uses the same hurry-reduced arm as execution. Feed with a likely continuation and cutoff throws stay at full effort. Pivot setup cannot hurry, but later effort selection bypasses its full-arm promise (P2-7). |
| 0.25 s tag × forces | A loaded force at home is made on the secure tick with **zero TagContact facts**, even though tag time has not elapsed. Rules classify a tag on a genuinely forced runner as `OutKind::Force`, preserving third-out run cancellation. The actual world is correct here; the runner's tag estimate is not (P2-6). |
| Tag × scorer evidence | World tag facts/out kinds, not the holder's predicted `ball_at`, determine outs. Touches and possession are recorded. However P2-1 can admit touches after the ball should be dead. The already-deferred Gemini tag-plan mismatch also remains: a carry adds a fresh 0.25 s even if the possession clock is already paid, and estimates compare at the bag while the world can tag short of it. |
| Runner τ × overrun/leads | Ordinary running and `runner_eta` use 0.68 s; default tag-up τ agrees. Stopping runners lose speed on the arrival tick; a protected first-base overrun retains crossing speed, brakes at 4 m/s², then returns at the walk-speed law. Holder options exclude that overrun. Leads remain 5.5/6.5/4.5 m and start at zero speed. I found no extra acceleration factor newly applied to the overrun. The existing 0.75 running-start ETA approximation and the source-fit concerns already deferred by Gemini are model limitations, not independently proven causes of this night's inflation. |
| Wall braking × plan/dive | The planner and call confidence include `wall_delay`; the stepped movement caps the normal velocity and keeps the body inside. Existing normal-approach timing tests pass. The missing constraint is the glove/contact after a clipped lunge (P2-2); body containment tests alone miss it. |
| Alignment × second-base cover | `home_positions_for` selects Standard vs DoublePlay from side/first occupied/outs as specified. A direct movement probe from those code positions found the cover times below. The asymmetric distances are real inputs, not a pivot-time multiplier. Coverage still depends on P1-1, and the owner-deferred two-chasers/uncovered-second test remains ignored. |

Second-base coverage measured from rest at an already-reacted time; arrival means within the world's 0.6 m touch radius. These are not game averages:

| Batter/alignment | Cover candidate | Distance (m) | Holder estimate (s) | Stepped arrival (s) |
| --- | --- | ---: | ---: | ---: |
| RHB Standard | 2B / SS | 9.296 / 14.779 | 1.889 / 2.637 | 1.950 / 2.717 |
| RHB DP | 2B / SS | 8.164 / 13.544 | 1.726 / 2.473 | 1.783 / 2.550 |
| LHB Standard | 2B / SS | 15.284 / 8.982 | 2.703 / 1.844 | 2.783 / 1.900 |
| LHB DP | 2B / SS | 14.409 / 7.773 | 2.588 / 1.668 | 2.667 / 1.717 |

The static-start receiver estimate is early by 0.049–0.080 s here. It does not include full heading/braking dynamics; close off-bag feeds still need end-to-end tests. I did not label this bounded approximation a new major defect from these measurements.

For rules, the engine correctly preserves release-time awards across hops/caroms, clears the thrown-ball status on securing, handles the all-runners-advanced exception, retreats/retouch obligations, already legally scored runs, blocking and lead-first placement. The caught-fly carry-out branch gives one base from legally held bases; its current fair-fly scenario passes. That matches the scoped carry-out rule. [2026 OBR 5.06(b)(3)(C)](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf)

The live sim cannot presently produce a caught fielder carrying his body out: players are kept inside and a held ball has no carry-out transition. The rules branch is scenario coverage, not a tested physical flow. Rubber pickoff/pitch awards, drops/knocked-ball producers, and dead-ball appeals remain declared seams. The remaining supported-flow problems are P2-1, P2-5 and P2-8; passing the awards corpus does not resolve them.

## Run environment: two counterfactuals

Both were compiled from the reviewed source **in memory**, against the independently built release dependencies. No checkout source was edited. Seeds 0–99, same average-player game entry point, one simulation process at a time:

| Build | Exact intervention | Runs / team-game | Changed game digests | Runs (away + home) | PA / K / BB / HR / reached BIP |
| --- | --- | ---: | ---: | ---: | --- |
| Reviewed result | None | **5.905** | — | 613 + 568 | 7740 / 1227 / 548 / 389 / 1901 |
| Correct carrier forecast | Only `option.base == 2` → `option.base == 2 && !option.carry` in `force_continuation` | **5.950** | 22 / 100 | 619 + 571 | 7747 / 1228 / 548 / 387 / 1914 |
| Full-effort pivots | Only insert `Some(_) if pivot_throw(&fd, tgt) => 1.0` before the effort match's continuation branch | **5.905** | 7 / 100 | 613 + 568 | 7740 / 1227 / 548 / 389 / 1901 |

These test the two most direct code-based inflation suspicions I found in the new decision interactions: undervaluing an available unassisted double play, and slowing the second throw despite a full-speed forecast. The first is a large local forecast error but correcting it **raised**, rather than lowered, runs in this sample; the second changed execution/logs without changing any aggregate batting result or score. Whole games diverge after a changed play, so these deltas are descriptive, not estimates of a population effect. No claim of statistical significance or of a 300-game standard-run reproduction is made.

From the code, faster ordinary running and a nonzero receive/tag interval can legitimately turn races into safe advances; the pivot's longer physical clock can legitimately remove double plays. Conversely, the impossible wall catch creates outs incorrectly in the defense's favor, while delayed dead balls and bad attempt bookkeeping can change individual awards/runs. None of my measurements identifies an extra 1–1.5 runs from a defect. Do not tune attributes/noise to cover these unresolved mechanics. The reviewed 100-game build itself has 389 HR / 7740 PA (5.03%) and 1227 K / 7740 PA (15.85%), so the already-known batting-side gap remains present independently of these two counterfactuals.

## Test-quality sample: ten new tests

All ten passed in the unmodified focused suite. “Spec” means the assertion has an external behavioral/formula justification; it does not mean the surrounding subsystem is fully covered.

| Test | Spec check or output pin? | What it establishes / misses |
| --- | --- | --- |
| `throw_choices::seed_1_play_64_never_projects_the_stopped_third_base_runner_home` | **Spec regression** | Nonempty decisions and no fictitious throw home. Appropriate visible-destination invariant on a frozen contact. |
| `throw_choices::seed_23_play_109_takes_a_sure_lead_force_instead_of_automatic_first` | **Hybrid** | Explicitly proves carry-first was compared and a sure lead force selected. Final `[4,1]` outs additionally pin execution, which could change without breaking the choice rule. |
| `throw_choices::seed_3_play_140_keeps_the_force_double_play_against_a_single_home_tag` | **Mostly output pin for its claimed purpose** | Checks `[2,1]` and that a continuation was computed. Passes with continuation weight zero; does not prove the promised trade-off (P3-2). |
| `throw_choices::seed_17_play_203_does_not_run_at_third_for_a_runner_stopped_at_second` | **Spec plus incidental pin** | “No third-base option” checks observable targeting. Requiring the first action to be throw-first is more implementation-specific. |
| `live::a_tag_up_runs_the_set_start_he_judged` | **Spec/formula** | Hand-solved 90 ft acceleration time, 0.2 s wait and 60 Hz tolerance; independently checks decision/legs agreement. |
| `live::no_speed_carries_over_into_a_tag_up` | **Spec plus integrator pin** | Stale speed cannot become a flying start. Exact first-tick `8.2*DT/tau` also pins the Euler update, so a more accurate integrator would require an intentional test change. |
| `live::a_tag_needs_its_time` | **Spec/physical race** | Independent 1 m / 8 m/s argument distinguishes immediate tagging from 0.3 s receive/tag. Does not cover a force or a tag by a long-standing holder; force was probed separately. |
| `live::the_base_ahead_is_held_by_what_the_runner_sees` | **Spec / information isolation** | Varies the other runner's private goal while public position/motion history stays fixed, and brackets the 0.2 s read interval. Strong discriminating test. |
| `live::the_plan_includes_the_slow_down_for_the_wall` | **Spec/formula and model consistency** | Derives braking delay independently and compares against stepped movement. Good normal-approach coverage; not a wall-clipped dive/contact test. |
| `live::a_carrier_to_second_keeps_the_set_up_he_spent_running_with_the_ball` | **Spec regression with a gap** | Verifies eventual release keeps spent setup. Never checks the holder's continuation estimate, which still charges the obsolete pivot (P2-3). |

Other coverage holes found while sampling: `the_decision_uses_the_set_start` changes both reaction delay and fly depth between its two cases, so it is a weaker discriminator than its comment suggests. Existing ignored tests still cover uncovered second when both middle infielders chase and a chaser outside the recall candidate set being called off prematurely; these are owner-deferred, not silently passed tests.

## Verification

All cargo and simulation/compiler executions used `taskset -c 0-7 nice`; cargo used `CARGO_BUILD_JOBS=2`, `PATH=$HOME/.cargo/bin:$PATH`, and `ZIG_GLOBAL_CACHE_DIR=$PWD/target/zig-cache` from `sim`. Builds/probe binaries/logs went under `sim/target`, never `/tmp`. Runs were sequential. No privileged operation was needed.

1. **Unmodified tests:** `cargo test -p bb_game -p bb_agents -p bb_physics -p bb_rules`: **237 passed, 0 failed, 3 ignored**, including the crate-graph guard, all four throw-choice integration tests and the complete rules scenario corpus. Log: `sim/target/codex-review-tests.log`. Separately `cargo test -p bb_stats`: **43 passed**, including all 23 scorer-rules integration cases; log `sim/target/codex-review/stats-tests.log`. These are focused packages, not a claimed full-workspace run.
2. **Independent rebuild/replay:** two fresh release target directories, `cargo build --release -p bb_game --target-dir target/codex-review/build-a` and `build-b`. Separately linked drivers called `simulate_game(seed)` for **every seed 0–49**. All 50 digests matched. The matching digest-text files have SHA-256 **`0ebcc2873301a3c29532c7f85e2820aa730b831e98609b67b1cadf9337df8c2c`**. Files: `digests-a.txt`, `digests-b.txt` under `sim/target/codex-review`. Counterfactual baseline digests also match these for 0–49.
3. **Trace noninterference:** `cargo run --release -p bb_game --example throw_choice_probe -- 50 0`: **all 50 traced/untraced log byte streams and digests identical**, 2777 commitments, 660 total runs. No “ahead of stopped runner” commitment. Log `choice-probe-50.log`.
4. **Private/function probes without source edits:** constructed the crate in memory using absolute paths for existing modules, inlined `live.rs`/`holder.rs`, appended probe functions inside its test module, and sent it to `rustc --test ... -` over stdin. **12 diagnostic probes passed**, reproducing P1-1, P2-1 through P2-8 and P3-1, and measuring alignment/force interactions. Logs `probe-results.log`, `extra-probe-results.log`, `interaction-results.log`. An additional standalone production-noise reproduction of the overlapping keys passed (`production-bounce-keys.log`). A passing diagnostic here means it successfully reproduced the reported current behavior, **not** that the behavior satisfies the spec.
5. **Two release counterfactuals:** in-memory copies with only the interventions printed in the table; logs/data `counterfactual-base.txt`, `counterfactual-carry.txt`, `counterfactual-pivot-effort.txt`. Each contains per-seed digest and game counters for all 100 seeds. The reviewed source files were never overwritten.
6. **Test discriminator probe:** separately compiled `tests/throw_choices.rs` from stdin with its default replay parameter changed to continuation weight zero. `seed_3_play_140...` still passed. Log `mutation-results.log`.
7. **Static replay audit:** no production `HashMap`/`HashSet` iteration, direct platform transcendental calls or `mul_add` found in the scoped implementation. Stable vector/index order, explicit comparator tie breaks and total float ordering are used. The `Purpose` registry is unchanged across the range. `execute_throw` still draws speed before direction; effort/pivot changes add no motor draws. Runner decision streams retain timing `2n` / free-ball read `2n+1`; contact uses 1, and arm reads use the disjoint `2^20+fielder` region. Receiver perception keeps sub-index 0 for the batted ball and `1+release_count` for throws. The confirmed exception to safe stream allocation is **SurfaceBounce sub-index reuse**, P2-4. Same-seed determinism passing does not exonerate it.

Probe reconstruction details, to make the numerical claims reviewable without adding repository tests:

- Carrier forecast: fixture R1 on first; SS `(-5,36,0)`, first baseman at first, `t=2`, `up_at=1`, ordinary transfer, zero catch velocity, `face=None`; batter at `BASE_L-8.2*2.3`, speed 8.2, goal first; R1 at `1.5*BASE_L`, speed 8.2, goal second. Read the carry-second option. Retained completion is its first-out time plus `throw_time(SS.arm, BASE_L)+0.05` in this already-set fixture.
- Bounced throw: mark a fair ground ball and a throw released at 1.8; at `t=2`, use default field `s=15`, `c=foul_clear_m-0.3`, ball at ground radius, outward horizontal speed 10 and vertical speed -5; call `set_free`. Compare the last flight's boundary crossing with `path.t_end`, then call `ball_facts` halfway between.
- Bounce keys: call `batted_path` at time 2 from `(0,-16,1)`, velocity `(10,-10,0)`, zero spin; record callback indices while returning zero tilt. Compare their maximum with `bounces.len()+1`. Repeat using the production `SurfaceBounce` key `(seed=11,play=3,agent=0x0200,sub=1000+k)` and the surface roughness times the two truncated-normal draws.
- Wall dive: default dugout at `s=15`; fielder center `c=C-0.31`, ball `c=C+1.5,z=1`, outward speed 5, vertical speed -1, `t=2`; zero hands error, Chase role, an exact controlled belief, `must_dive_since=1`, last dive/close read 2. `try_captures` secures it while the recorded lunge has identical endpoints.
- Pivot effort: reuse `feed_to_shortstop(base_pos(2),5.0)` and inspect `out.effort`. Force timing: loaded fixture, catcher holding at home at the secure instant, forced R3 0.8 m short of home; call `holder_act` before moving runners.

Only this review is a tracked deliverable. No source, test, spec, ledger or TASKS file was edited.

## Resolution

Open.
