# Peel-off backup review — 2026-11-13

Reviewed `a23f7c2..aedd492`, after reading `CLAUDE.md` and agent-models §5.7. No P1 findings. The reported game statistics reproduce. Two findings remain: the offset does not enforce leaving the caller's ball, and expiration depends on another assignment rather than the visible end of the job.

## Findings

### P2 — A peeled backup can take a catch from the sure caller

**Location:** `sim/crates/bb_game/src/live.rs:1155–1160`, interacting with `try_captures` at `1567–1629`; `docs/calibration/2026-11-13-peel-off-backup.md:33–37`.

The 5 m is measured from the backup's *predicted intercept*, not from the actual ball or the caller. Prediction error can cancel that offset. Also, a fielder already inside the 5 m circle needs time to retreat. `try_captures` permits standing captures by either man and tests them in position-index order. Keeping the backup out of `Chase` prevents diving, but does not protect the caller's standing catch.

**Verified synthetic reproduction:** use perfect hands, zero initial velocities and otherwise average attributes. At `t = 1.0`, put CF at `(20, 60)` and 2B at `(20, 50)`. Set a free ball at `(20, 60, 3)` m with velocity `(0, 0, 25)` m/s and zero spin. Give everyone error-free reads except 2B's velocity-depth draw, `err[2] = 0.25`. Run the normal assignment and movement/capture functions. CF is the sure caller, index 7; 2B, index 3, gets `Backup` with peel target `(20, 60.037317)`.

At `t = 5.766667`, 2B is standing at that target, still `Backup` with `peel = Some(...)`, and secures the descending ball. CF is standing directly underneath it at `(20, 60)`. No re-call handed the ball to 2B. In an otherwise identical counterfactual that clears only 2B's peel job and makes him idle at his starting position, CF catches it at the same time. This is a controlled geometry probe, not a claimed naturally occurring game seed or a measured contribution to the calibration change.

A separate close-range probe also reproduces interception during retreat: ball `(20, 60, 3)` with zero velocity at `t = 3`, CF at `(20, 60)`, 2B at `(20.2, 60)`, error-free reads. Assignment gives CF the call and 2B a peel target `(25, 60)`. At `t = 3.35`, 2B is still only 0.790 m from the ball's line and catches it as a backup. The target really is 5 m away; the moving fielder is not.

**Impact:** the calibration note's assertion that he cannot take a ball the caller would catch is false. This can change catch attribution and which fielder's glove-placement execution gets the first attempt; a rim deflection by that backup can also remove the caller's opportunity.

**Suggested fix:** enforce yielding while the sure call remains active, using the observed call/catch/deflection state, or explicitly revise the behavioral contract. Preserve legitimate recovery of a dropped/deflected ball and a later re-call. Add independent cases for an erroneous read and for a fielder already inside the offset; increasing the offset alone cannot guarantee either case.

### P3 — The peel job does not expire when the ball visibly comes down

**Location:** `sim/crates/bb_game/src/live.rs:1354–1368`; production tick loop at `2440–2472`; new test at `3776–3780`.

The bounce check is evaluated only inside `assign_backups`, which production calls from `assign_among`. Neither `ball_facts` nor the ordinary tick loop expires a peel on ground contact or possession. A recall additionally requires a primary replan, the recall timer, and horizontal distance greater than 8 m from the ball. Consequently a backup can keep running to, or waiting at, the old airborne spot after the ball has bounced, until another assignment or a throw-backup job changes it. A nearby primary can prevent that recall for the remainder of the chase.

**Verified probe:** initialize the new test's error-free `(7, 12.5, 28)` m/s hit and assign it. Advance to the first bounce plus one tick and call `ball_facts`. 2B remains `Backup`, with both `peel` and target equal to `(15.993622, 38.931242)`. The ground fact is emitted, but the job survives. The regression test explicitly calls `assign_backups` after advancing the clock, so it checks cleanup *when reassignment runs*, not whether normal play actually triggers cleanup at the promised boundary.

**Suggested fix:** expire the airborne peel on the corresponding visible fact and explicitly choose its next job. Avoid resetting a cover/chase/relay job acquired in the meantime. Add a tick-driven expiration check that does not manually invoke `assign_backups`. I did not establish an extra run caused by this delay.

## Other checks

- **Real jobs survive reassignment:** the chosen primary clears `peel` immediately. Secondary ground-ball/air-ball chasers and cover/relay assignees are non-idle when `assign_backups` examines them, so its fallback clears their peel instead of replacing their job. `cover_roles_after_chasers` protects chasers and relay men. `back_up_throw` explicitly clears peel on its recipient. I found no demonstrated path where the new branch overrides one of these active jobs. Finding 2 concerns expiration between assignment calls.
- **Information boundary:** new peel targets come from each participating fielder's own freshly computed/reused intercept, saved before cover assignment. They do not use the true landing point or the caller's read. Although the airborne check accesses the world path in the orchestrator, it asks only whether a bounce has already happened: every possible future bounce gives the same result before contact. That is an observable historical fact, not knowledge of the future bounce time or location.
- **Test independence:** the new test checks a physical fixture's approximate landing point, side, role, and persistence. It is a meaningful regression check: running it against the old production implementation fails at the 2B role assertion (`Idle` versus `Backup`). Its error-free read and direct cleanup call leave the two findings above uncovered. Using `PEEL_BACKUP_M` in the expected distance also does not independently pin the specified 5 m value.
- **Invariants:** the diff adds no outcome roll, RNG call, unordered collection iteration, transcendental operation, or statistic counter. Ordering uses existing indexed vectors. The spec changes with the implementation. Event sourcing and crate boundaries remain intact; workspace determinism, recording-invariance, log verification, and crate-graph tests pass.

## Verification

Repository source, tests, specs, and `TASKS.md` were left unchanged. The only review output is this file. Temporary baseline archives, logs, and standalone probe harnesses were placed under `/tmp`; Cargo targets and probe binaries stayed under this checkout's `sim/target`.

Commands ran with `export PATH=$HOME/.cargo/bin:$PATH`. The initial workspace build failed because Zig attempted to write its cache outside the sandbox. Retried successfully with `ZIG_GLOBAL_CACHE_DIR=/tmp/peel-zig-cache` and `ZIG_LOCAL_CACHE_DIR=/tmp/peel-zig-local`.

1. `git diff a23f7c2..aedd492`, relevant role/capture/tick paths, and `git diff --check a23f7c2..aedd492` — inspected; whitespace check passed.
2. From `sim`: `cargo test --workspace --release` — **144 passed, 0 failed**, including the new regression and three determinism tests.
3. From `sim`: `cargo run --release -p bb_cli -- games --n 1024 --seed 5`, repeated with `--seed 2` — reported after-values reproduced; no rules warnings.
4. Unmodified baseline from `git archive a23f7c2 sim`, built with `--no-default-features` and an isolated target at `sim/target/peel-review-base`; same two CLI samples — before-values reproduced; no rules warnings. The disabled default feature is Rerun visualization. An earlier attempt sharing the head target reused artifacts and was discarded; the results below use the isolated baseline build.

| Seed | Runs/team-game, before → after | PU hit | FB hit | LD hit | GB hit |
|---|---|---|---|---|---|
| 5 | 5.47 → 5.43 | .018 → .008 | .122 → .120 | .723 → .723 | .411 → .411 |
| 2 | 5.27 → 5.26 | .018 → .008 | .120 → .119 | .712 → .715 | .407 → .406 |

5. `cargo run --release -p bb_game --example trace_play -- 29 135`, head and isolated baseline — head SS (`fielder = 134`, position 6) secures at **5.793090 s**, before the predicted first ground contact at **5.897555 s**. Baseline CF (`136`, position 8) first secures at **6.809757 s**, after ground contact.
6. `cargo run --release -p bb_game --example pu_hits -- 128`, head and isolated baseline — **12 → 7** uncaught pop-ups. The seven remaining printed cases have 50–53° launch, 81–100 mph exit speed, and first possession by an outfielder at 6.06–8.98 s. Rounded launch-angle labels follow the existing example.
7. Standalone `rustc --test -C opt-level=3` harnesses containing the production `live.rs` verbatim, unchanged helper copies, and additional probes under `/tmp` — reproduced noisy-read capture, capture while retreating, the caller-catching counterfactual, and surviving peel after a ground fact. A preliminary 33-point velocity-depth sweep of the original fixture did not produce a peeled capture. The synthetic capture probes above establish possibility, not corpus frequency.
8. Standalone old-code harness plus the added regression test — expected failure at `assert_eq!(fd.role, Role::Backup, "2B")`. The absent old-code `PEEL_BACKUP_M` name was replaced by the literal `5.0` in the temporary copied assertion only.

The numerical rate claims are verified. The performance claim was not assessed under comparable load: the two head samples ran concurrently, so their timings cannot establish unchanged cost per game.

## Resolution

Both findings fixed (2026-11-13, same branch).

- **P2, a peeled backup can take the caller's catch: fixed.** The yield is now explicit: in `try_captures` a fielder whose job is a peel-off backup (`peel` set, role `Backup`) makes no glove attempt, however close the ball comes. The job ends (below) when the ball comes down, is caught or is thrown; a glove deflection goes through `set_free`, which now clears every peel before its full re-call, so a ball off the caller's glove is a new ball that anyone may play; a re-call that hands him the ball makes him the chaser and clears it. `assign_among` now also clears the peel of every infielder in a call with no sure caller, so a spot from an earlier call does not survive into a later one. The test checks that, with the ball within his reach while the call stands, he makes no attempt (on the code before the fix he catches it: `(1, false)` against `(0, true)`). The calibration note's claim that the 5 m alone kept him off the ball is corrected.
- **P3, the job does not expire when the ball comes down: fixed.** `Sim::expire_peels` runs every tick, after the ball facts and before the capture test: once the ball is no longer up (bounced, held, or thrown; `ball_still_up`, the same visible fact as before), every peel is cleared, a man still on that backup job is stopped (`go_idle`), and a loose ball is re-called at once after the chaser's replan (`replan_primary`, now shared with the tick loop's replan). The test ends the job through `expire_peels`, not a manual `assign_backups`.

After the fixes: 144 tests pass; 1,024 games seed 5 5.44 runs/team-game (5.43 before the fixes), seed 2 5.26 (unchanged), pop-up hits .008 on both; seed 29 play 135 still caught by the shortstop at 5.79 s; 7 uncaught pop-ups in 128 games (unchanged). The performance claim was withdrawn from the note (the timing runs overlapped).
