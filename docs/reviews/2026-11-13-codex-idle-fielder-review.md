# Codex Sol review: idle fielders stop instead of chasing their own intercept

Scope: `760afe8..fix/idle-fielder-target` (`077aab5`, `87344d1`), reviewed on branch `coord/review-fixes`, HEAD `ac26116`. Reviewed `live.rs`, agent-models §5.7, the pitcher-outfield diagnosis's “Part A applied,” and the idle-fielder calibration note. Locations below refer to HEAD. Read `CLAUDE.md` first and checked the binding PLAN invariants. Only this report changes the checkout; probes and source variants are under `/tmp`.

## Findings

### P2 — Retiring a relay man still leaves an Idle fielder running at an expired target

**Location:** `sim/crates/bb_game/src/live.rs:2005`; also `:1175` and `:1185`.

**Defect:** The new reset correctly handles every call through `cover_roles`, but these other transitions to Idle retain the relay/cutoff target. `back_up_throw` need not select the retired man, and `cover_roles_after_chasers` need not give him a base. He therefore keeps moving with no job. This is a pre-existing completeness gap, not a regression introduced by the reviewed line; the fix does not establish a universal Idle-means-stop invariant.

**Verified failing scenario:** With a runner on second, hold the ball in RF at `(25,62)` m. The first baseman becomes the cutoff to home, targeting `(8.17,20.27)`. Put the second-base receiver on his bag and let `holder_act` return the ball directly to second with calm runners. The first baseman becomes Idle with `backs=None`, but retains that cutoff target. Starting stationary at `(19.70,25.22)`, he travels **10.328 m in two seconds**. The independent assertion that a stationary retired Idle fielder holds his position fails. A second probe, ending his job when an infielder holds the ball and invoking `update_relay`, fails identically.

**Suggested fix:** Centralize entering Idle so it sets a braking target, then assign any actual cover or backup job explicitly. Cover both relay-retirement branches and the direct-throw path with transition tests. Do not indiscriminately reset the target every movement tick.

### P2 — Unconditional outfield priority rejects a substantially better shallow-ball call

**Location:** `sim/crates/bb_game/src/live.rs:1062`, selected at `:1014`.

**Defect:** Any confident outfielder outranks every confident infielder, regardless of their respective margins or whether the ball is effectively the infielder's play. Stopping the called-off infielder makes this pre-existing planner limitation consequential. It agrees with the documented interim implementation, but is broader than the general spec's outfield priority “on balls in between.” This is an exposed follow-up, not an error in the stop-target assignment.

**Verified failing scenario:** Seed **41, play 93**, launch **51.57°**, lands at `(27.11,60.67)` m. At the initial call the second baseman reports **2.274 s** spare, the CF **1.549 s**; CF wins by tier. At the next call the margins are **1.249 s** and **1.061 s**, and CF still wins. CF is **12.85 m** from the landing point at the first tick at/after landing, moving **8.16 m/s**. Replaying the identical input and rules state with confident air calls in a common tier, preserving perception errors and the re-call margin, produces a catch. Merely including all nine re-call candidates does not.

**Suggested fix:** In a separate planner change, define when outfield priority applies and how a confident, better-placed infielder can retain or reclaim a shallow ball, using communicated beliefs. Test this launch and cases that preserve legitimate outfield priority. Add the already acknowledged called-off backup behavior separately. No pop-up behavior was fixed in this review.

### P3 — The calibration explanation misidentifies the moving state and its example

**Location:** `docs/calibration/2026-11-13-idle-fielder-target.md:38` and `:49`.

**Defect:** The note says the nearest outfielders are usually standing on their chosen spots, and describes seed 6/play 84 as a second baseman called off by RF. Neither description reproduces at HEAD. This matters because a late route after an early depth error calls for a different diagnosis from a stationary fielder refusing to adjust.

**Verified failing scenario:** Across the reproduced **26** uncaught fair pop-ups landing within 75 m in seeds 0–127, snapshots at the first tick at/after the original landing time show **24** nearest fielders moving above **4 m/s**, median **8.14 m/s**; none moves below 0.5 m/s. Twenty-five nearest fielders are outfielders, including two left fielders; median landing-point distance is **4.93 m**. Seed **6/play 84** has the second baseman as Chase throughout its pre-deflection calls, not called off; at landing he is 1.30 m away and moving 1.09 m/s. He attempts a dive before landing and rims the ball. RF is Backup.

**Suggested fix:** Replace the stationary-target explanation and the example with recorded roles, velocities, and call history. Seed **35/play 74** is a reproducible direct Idle rescue: with the old target behavior, the Idle second baseman pockets the ball 0.81 m away before landing. Also distinguish direct Idle catches from drift enabling a later re-call: of 23 paired old-behavior rescues, **19 catchers have regained Chase** and only **4 remain Idle** at their first successful glove contact.

## Other checks

The one-line change is correct for its narrow purpose. `assign`, `recall`, and `set_free` all reach `assign_among` → `cover_roles`. A nearby previously Idle pitcher was reassigned Chase and secured a newly loose ball in a focused probe. Deflections and throws reaching the ground call `set_free`, so they do not leave the whole defense frozen on its old assignments.

The ground-ball break saves `own` before coverage resets targets and restores it only for fresh-read secondary chasers. Relay eligibility still uses `t_int` and belief, not an Idle fielder's reset target. Backup jobs supply their own targets; in the production call path `cover_roles` has already removed prior Backup roles before `assign_backups`. The bases-empty catcher is immediately eligible for his explicit first-base backup. A successful relay catch already sets its Idle holder's braking target. `try_captures` still permits any nearby fielder to make a standing catch; only chasers receive close re-reads and dive permission. These observations explain both the old safety net and why a nearby man need not chase to catch a ball within reach.

`stop_point` uses the fielder's full horizontal velocity vector: `pos + velocity * speed / 12`, giving `v²/12` metres along motion, independent of the previous target. A sweep of **24 directions × 8 speeds** (0–20 m/s), stepped for ten seconds after the reaction gate, stopped within 0.05 m of independently calculated endpoints with zero final velocity. No arbitrary-direction braking defect was found. Before his reaction gate, a fielder still follows his anticipation jump as specified.

**Pop-up root causes, ranked by probe evidence:** Replay each of the 26 HEAD misses from exactly the same contact input and cloned rules state; return the HEAD result to the game so counterfactuals cannot change later pitches.

| Diagnostic change, one at a time | Catches among those 26 misses |
|---|---:|
| Old Idle target behavior | 23 |
| Zero fielder position/velocity read errors; retain typical-spin model, anticipation and glove noise | 25 |
| True launch spin for pre-bounce prediction; retain read errors | 5 |
| Common tier for confident infielder/outfielder air calls | 12 |
| All nine re-call candidates | 0 |
| Remove the 8 m re-call exclusion; retain normal cadence | 0 |
| Remove that exclusion and re-call at each primary replan | 2 |

1. **Early perception error and the resulting late route dominate.** Outfield `read_sigma0` is 3.0 m versus 0.3 m infield (`play.rs:74`), with depth stretching and fixed per-play error signs (`live.rs:789`). For example, seed 9/play 197's LF predicts a landing near `(-28.63,92.19)` at 0.65 s after contact; truth is `(-7.92,58.79)`. His target corrects inward all flight, but he is still running when it drops. Zeroing read errors recovers 25/26; this is an isolation experiment, not a recommendation to remove noise or proof that current noise is unrealistic. Next test: separate position-depth and velocity-depth errors, measure early route loss, and validate observation accuracy against independent physical/reference cases.
2. **Call priority prevents the better read from helping** (finding above). The drift previously supplied both direct incidental catches and moving candidates who could regain Chase. Common-tier calls recover 12/26 without improving perception. Test a scoped priority/handoff rule with the same launches and defensive-coverage checks.
3. **Spin estimation contributes, but is not the main explanation in this sample.** The table clamps above 55°, and launch-angle reconstruction adds gravity back without undoing drag/Magnus effects (`live.rs:815`). True-spin prediction recovers only 5/26 while retaining noisy reads; perfect reads with the existing spin/drag prediction recover 25/26. Test true spin and zero read error factorially, binning signed spin and landing errors above 50°, before changing the learned belief table. The oracle-spin probe is diagnostic only and would violate observation isolation as production logic.
4. **Candidate filtering and re-call cadence are lower-priority hypotheses here.** All-nine selection and removing the distance gate individually recover zero; frequent unrestricted re-calls recover two. This does not prove those policies correct for every trajectory, but does not support them as the main cause of these 26 misses.

No new RNG calls, keys, or within-stream draw-order changes occur in the reviewed code. The new target depends only on the fielder's own position and velocity; it adds no world-state observation. Different routes naturally change which keyed glove/throw opportunities occur; identical old/new event logs are not expected. Existing cross-thread determinism, log verification, recording-independence, crate-graph, physics and rules checks pass.

The new regression test uses independent distance/direction/rest assertions rather than calling `stop_point` to construct its expected answer. Removing only the target assignment in a `/tmp` copy makes it fail on the Idle pitcher still aiming at `(62,88)`; it passes at HEAD.

Agent-models §5.7 accurately describes the new behavior **at calls and re-calls**, its braking distance, and the missing called-off backup behavior. It does not cure the separate retirement transitions in finding 1. The diagnosis and §5.7 correctly leave **Part B**, the pitcher's job on bases-empty outfield balls, open and awaiting the owner. No job or calibration parameter was added.

## Verification

All three requested commands completed successfully on the unmodified HEAD implementation, with Zig caches under `/tmp`:

```text
cd sim
cargo test --workspace --release -j 4
cargo run --release -p bb_game --example pitcher_roam -- 1 200
cargo run --release -p bb_cli -- games --n 1024 --seed 5
```

- Workspace: **137 tests passed**, zero failures, plus successful doc-test groups.
- HEAD pitcher probe: **39 / 14,856 (0.3%)** flagged; 7 secured-and-ended-far cases. Log: `/tmp/codex-idle-pitcher-roam.log`.
- HEAD games: **5.48 runs/team-game**, 40.1 PA/team-game; GB/LD/FB/PU hit rates **.410/.722/.131/.044**; no rules warnings.
- A `/tmp` copy with only the stop-target assignment removed reproduces **5.27 runs/team-game**, 39.7 PA/team-game, and **.409/.722/.122/.005**; no rules warnings. Log: `/tmp/codex-idle-baseline-games.log`.
- The same one-line-revert copy, using an isolated `/tmp` Cargo target, also reproduces the old pitcher result: **2,030 / 14,777 (13.7%)** flagged. Log: `/tmp/codex-idle-baseline-roam.log`.
- Negative regression test: expected failure, exit 101; `/tmp/codex-idle-old-test.log`.
- Focused target probes: braking sweep and nearby loose-ball reassignment pass; the two retired-relay hold-position assertions fail as described. `/tmp/codex-idle-target-probes.log`, `/tmp/codex-idle-direct-relay.log`; source/reconstruction in `/tmp/codex-idle-review/probes.rs` and `run_target_probes.py`.
- Paired 128-game probes and isolated re-call variants: `/tmp/codex-idle-popup-followup.log`; script `/tmp/codex-idle-review/instrument.py`. Exact landing snapshots: `/tmp/codex-idle-landing.log`; script `landing_instrument.py` in that directory. These are temporary diagnostic variants, not source changes or proposed fixes.
- Workspace test log: `/tmp/codex-idle-workspace-tests.log`; HEAD distribution log: `/tmp/codex-idle-games.log`.

## Resolution

Open.
