No P1 found; merge.

Reviewed `8fa55c6..d99a3f8`, particularly code `12cdb26`, `2969aee` and `3b7cb38`, against main `57f12b7`. Read `CLAUDE.md` first, both fourth reviews and their Resolutions, agent-models §5.7, and the calibration note. The checkout's additional `c21f9e8` is the review prompt only. Production source, committed tests, spec, ledger and TASKS were not edited. All instrumentation and counterfactual sources/builds are under `sim/target/codex5/`; no commit was made.

## Findings

No P1 or P2 findings.

### P3-1 — The 4/280 replay names the wrong input revision; its residual attribution needs qualification

**Locations:** `docs/calibration/2026-11-14-fielder-communication.md:101`; `docs/reviews/2026-11-14-codex-fielder-communication-fourth-review.md:120`.

Both say the preserved contact comes from `74ea51b`'s game. A fresh seed-4 game on that revision reproduces digest `6e724e74dfa9491c19201404113a04b92b983d0eb72e2a6a6b6b0c9d561610ab` and does not supply this contact. The actual legacy driver is **`f3d9e7a`**: its `live.rs` matches the earlier scratch driver's source except for read-only tracing. Rebuilding that driver reproduces the preserved digest `7682817e8549cd22b9c02df3678c6a9cc5faf341979c4222f13d629de774a0ae` and the contact at **0.485333892 s**.

On that exact input, fresh defenses reproduce main LF's catch at **6.285334**, the bounce at **6.305548**, stale-call `74ea51b` CF's fielding at **7.268667**, and current LF's fielding at **6.568667**. Thus the stale-call repair really saves **0.700 s**.

The remaining explanation should distinguish a delayed decision from a complete attribution of the missed catch. Current LF becomes sure at **4.552001**, but yields until the CF withdrawal is heard at **5.268667**. An isolated counterfactual changes only the shared priority table to put LF above CF: LF starts chasing at **4.552001**, **0.717 s earlier**, yet still fields after the bounce at **6.518667**, only **0.050 s earlier**. Main's LF remains **Backup** through his catch. The priority delay exists, but backup geometry and LF's own read still matter; removing that delay does not restore the catch in this probe.

**Suggested fix:** name `f3d9e7a` as the preserved-input driver in both places and qualify the residual explanation with these distinctions. Retain the input/digest identity so this case is reproducible without guessing its historical revision. This is a documentation/attribution correction, not a remaining stale-call code defect or merge blocker.

## Review conclusions

**Unpublished state and requests — `live.rs:3689`, `3819`, `3863`.** The pre-release probe passes on literal seeds **0–9**, then on **0–19**: **145,758 held snapshots, 658,321 comparisons, zero leaks**. Setup duration, request, plan and cleared belief are flipped at every checked held snapshot; the full committed flip set is used every 60th such snapshot. Comparisons include the subsequent `heed_requests`, before any new publication. Ten perturbations that release the ball are excluded as public events. Restoring the old direct target write fails on **1/195 at 7.219582563**, with only `set_base += 1.0`, reproducing the original counterexample. The two committed snapshot regressions also pass in the workspace suite.

`holder_act` clears/reconsiders its request each invocation. Publication carries only **base and named receiver**, not the holder's race estimates or preparation clock. Heeding uses the delayed publication, the currently visible holder, and the possession boundary (`secured_at`/`ball_since`). A release or loss of possession stops heeding immediately; a later possession cannot reuse the old request. Only one holder's request can apply.

A superseded request **can** first redirect a man before its cancellation is heard: on 1/195, LF hears the earlier request while the holder already privately wants SS. Checking the holder's current private `asks` to suppress this would reintroduce the isolation defect. Cancellation stops reasserting the target after the same hearing delay; the man then keeps the bag until his job is reworked, as specified. The constructed lifetime probe passes. In the 50-game trace, eight stale request/relay conflicts on **8/121** are overridden by `update_relay` before movement; no conflicting cover redirect was found. Most importantly, a paired defense restoring only the instant redirect produces **identical events and outs on all 300 plays with holder requests**, on the current branch's identical inputs. No out lost to the later start was observed. A loss caused solely by the specified hearing delay would be an honest information cost.

**Fresh calls and withdrawal — `live.rs:1643`, `2763`, `2049`.** Calls are refreshed from the caller's own reads and published through the delay. There is **no hysteresis**, and brief reversals exist: on **45/34**, RF publishes tier **1 → 3 → 1** at **5.333507, 5.383507, 5.433507**. Across 50 games there are five continuous-chaser A–B–A reversals involving a sure tier within 0.6 s. These produce no repeated teammate chase starts/stops; some hearings still rework cover jobs. I found no sustained tick-by-tick oscillation requiring a hysteresis change before merge. The delayed-withdrawal/takeover regression passes. Multiple unsure men may still chase temporarily, as the spec permits; recovery and coverage probes show no renewed all-nine swarm or sustained absence of a chaser. The preserved 4/280 result is discussed above.

**Harness — `live.rs:3096`, `4199`, `6838`, `7026`.** The previously omitted new-ball assignments, replans, close/dive reads, receiver reads and throw backups are hooked. Roles, backup bases, valid alternative reads, scheduling clocks and holder state are exercised; rare and job-changing calls receive dense sampling. The default real-game harness passes. Its cutoff negative control independently reproduces **227 violations in 102,969 checks** over seed 0's first 30 plays.

The comparison is not literally equality of every trajectory element: beliefs and free paths are sampled at current/end points, and throws compare endpoints (`6844`, `6858`, `6861`). A scratch comparator probe confirms that a deliberately edited interior flight knot can differ by 10 m without detection. The Resolution discloses those samples; this remains a coverage limit, not a newly found production leak. Canonical path comparisons would make the “whole state” wording literal. Excluding world motion/capture steps is appropriate for a pre-observable-event check. The runners' own decision exemption is acceptable for this defensive review after auditing their inputs: no new request, call, target or private-role dependency enters them. This is finite testing plus a source audit, not proof over every state.

**Tie order, cache and mechanical splits — `live.rs:1777`, `1800`, `1820`.** The dead-heat convention is sensible for mound pop-ups: agreed fielding priority calls the pitcher off. It uses public ball/body motion and known attributes, not private reads or targets. The cache now includes velocities. All **50** digests of `2969aee` match a fresh current build with only the old tie key restored, supporting the mechanical split/cache claim. Restoring that key changes exactly **3, 8, 14, 19, 26, 47**, the six claimed seeds. The mound tie and velocity-cache regressions pass.

**Numbers.** Fresh standard runs reproduce every requested headline:

| Standard `--seed 0` [`--seed 1000`], 300 games each | Main `57f12b7` | Current branch |
|---|---:|---:|
| Runs/team-game | 4.31 [4.68] | 4.42 [4.59] |
| Fly-ball hits | .212 [.215] | .219 [.228] |
| Rules warnings | 0 [0] | 0 [0] |

Fresh current coverage over literal seeds **0–49**, **3,750 live plays**, reproduces first **3**, second **15**, third **31** uncovered intervals over 0.3 s; longest second **1.333 s**. Sustained double covers, absent chasers and repeated all-nine chase starts are zero. Fresh paired flies reproduce **1,116 flies, 12 catches lost, 15 gained**, hits **377 main / 375 branch**. All 50 paired branch digests equal pristine traced games. These paired results do not establish defense equivalence or explain the aggregate movement between diverged games.

## Verification

- `cargo test --workspace --release`: **367 passed, zero failed, four ignored**, including 170 passing `bb_game` library tests, isolation, crate-graph and determinism checks.
- Fresh branch/main standard runs; 50-game coverage, paired fly and holder-request replays; 50-game split/cache comparison; preserved-input three-defense replay and isolated LF-priority counterfactual; holder sweep, lifetime probe and both negative controls.
- Seeds **0, 1, 42** repeat exactly and match their traced digests. No new hash iteration, direct transcendental math, unkeyed RNG, outcome roll or agent/world dependency was found.
- Builds/tests/simulator runs were serialized, from `sim/` directories, with `taskset -c 0-7 nice`, `CARGO_BUILD_JOBS=2`, the requested PATH/Zig cache environment, and separate targets under `sim/target/codex5/`. The author's `sim/target/release` was not rebuilt. Logs and scratch sources are under `sim/target/codex5/`.
- The author's pending **50-seed ignored isolation harness** was neither awaited nor rerun; its placeholder/result is not evidence used here. The standard 1,000-game attribution counterfactuals were not rerun.

## Resolution

Open.
