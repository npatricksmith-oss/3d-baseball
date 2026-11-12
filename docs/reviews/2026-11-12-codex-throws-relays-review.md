# Independent review: drag throws, cutoffs, relays, and client staging

Reviewed `1f7ca4c..845a862` (`HEAD`), plus `StartReset` / `ResetFrame` and their playback integration in `1f7ca4c`, with the parent as context. Read `CLAUDE.md` first and applied its invariants and the relevant agent/physics specifications. Findings below are ordered by priority. No implementation, test, spec, or TASKS file was changed.

## Findings

### 1. P1 — The receiver plans from the throw's realized future and execution error

**Location:** `sim/crates/bb_game/src/live.rs:1121–1139` (`launch_throw`, `update_thrown`); related description in `docs/agent-models.md §5.5`.

`launch_throw` integrates the actual executed launch in the world's environment, searches all future trajectory knots for the best receiving point, and stores that point in `catch_at`. `update_thrown` immediately makes it the receiver's movement target. This is a decision consuming the true future, including the thrower's realized motor error, without the receiver making an observation or its own prediction. Calling the update “watches the throw” does not provide that missing observation boundary. This violates the binding agent/world separation and artificially helps receivers compensate for errant throws.

**Executed scenario:** At `t=5`, release from `(0,100,1.8)` toward second at `(0,38.794706,0)`, with elevation solved for a nominal 37 m/s throw. Vary only executed speed within the specified noise envelope. At the release instant, before the ball travels anywhere, `update_thrown` gives the receiver these ground targets:

| Executed speed | Receiver target Y | Offset from second |
| --- | ---: | ---: |
| 35 m/s | 41.040270 m | 2.245564 m toward the outfield |
| 37 m/s | 38.794706 m | On the bag |
| 39 m/s | 36.402158 m | 2.392548 m toward home |

The targets come directly from the realized future trajectory; they are not noisy estimated intercepts. More extreme, still permitted, speeds of 33 and 41 m/s produce offsets of 8.32 and 9.26 m respectively.

**Suggested fix:** Keep the realized path exclusively in world advancement/collision code. Let the receiver periodically observe the current throw, form its own noisy state estimate, and predict a receiving point with its own physics and reaction timing. Preserve the preference for staying on the bag when its believed receiving envelope permits it. Document and test that observation contract before implementing it; changing unobserved future trajectory data must not directly change the receiver's plan.

### 2. P2 — The throw solver rejects a reachable throw because height is not monotonic in elevation

**Location:** `sim/crates/bb_physics/src/throw.rs:68–84`; fallback in `sim/crates/bb_game/src/live.rs:1493`. Related claim: the “flatter elevation” solve in `docs/agent-models.md §5.5`.

The solver assumes target height increases throughout the interval ending at 35°. It returns `None` whenever the 35° shot passes below chest height. With drag and backspin, target height can peak below 35°, so that endpoint does not establish unreachability. Consequently both aiming and the throw-time belief can reject a valid direct route; the fallback actually launches the slower, lower-arriving 35° shot.

**Executed scenario:** Default atmosphere, release height 1.8 m, target height 1.3 m, speed **44 m/s**, distance **122 m**: `solve_throw` returns `None`. Actual flight at 31° reaches the target plane at height **1.571899 m**, while 35° reaches it at **0.995909 m**. Independently integrating the same case with the repository's Python oracle, at a 1 ms step, gives a flatter chest-height root of approximately **29.784088°**, arrival **5.168754 s**, height **1.300000 m**. This is not a comparison of the solver against itself.

**Suggested fix:** Locate the first upward crossing of target height over the permitted angle range, then refine that bracket. Do not infer that no root exists from the upper endpoint alone. Add an independent oracle/golden near-range case, including this nonmonotonic region. The existing vacuum analytic test is independent, but the drag landing and table tests mostly compare consumers of the same solver/flight implementation and do not cover this failure.

### 3. P2 — The promised early deep-ball relay assignment cannot activate

**Location:** `sim/crates/bb_game/src/live.rs:695`, with `intercept_inner` at `510–521`; mismatch with `docs/agent-models.md §5.7`.

Before the ball is down, the relay job requires the candidate's `t_int` to be infinite. Initial assignment computes an intercept for every fielder, and even a failed intercept scan returns a **finite** travel-to-rest fallback. Thus a middle infielder whose read says he cannot reach the airborne ball still cannot receive the early relay job. A deep fly must land or be secured before this assignment can start, forfeiting the positioning time described in the spec. An `Idle` candidate can meanwhile retain its unrelated intercept target.

**Executed scenario:** Existing live-test fixture, runner on second, ball at `(0,0.5,1)` at contact `0.4 s`, velocity `(-4,28,22)` m/s; call normal `assign()` at `t=1`. The primary's believed fielding point is `(-5.792227,94.763893,0)`, so this is a deep left/center relay case. The shortstop is `Idle`; its own believed first bounce is **5.263085 s**, but its intercept is the finite fallback **11.291867 s**. `relay_job` returns `None`, despite the shortstop having no airborne play. Additional deep-fly probes give the same result.

**Suggested fix:** Represent whether the fielder believes it has a playable airborne interception separately from its eventual recovery time. Use that belief to admit the early relay, while preserving the protection against pulling an infielder off a catchable fly. Add a test before the first bounce; the current relay tests start with the ball already held and cannot exercise this branch.

### 4. P3 — Interpolation loses the finite maximum-speed endpoint through infinity arithmetic

**Location:** `sim/crates/bb_physics/src/throw.rs:145–157` (`ThrowTable::time`).

At the maximum speed the selected cell has weight `a == 1`. If its lower-speed row is unreachable but the maximum-speed row is reachable, `r0 + (r1-r0)*a` evaluates as `infinity + (-infinity)`, yielding NaN, which is converted to infinity. This is not the documented conservative treatment of a genuinely mixed-speed request: the unreachable row has zero weight. It can incorrectly force a strong arm into a relay or an out-of-reach fallback.

**Executed scenario:** Default table, `time(48,130)` returns **infinity**, although `solve_throw(...,130,48)` returns a flight time of **4.699492 s** and `reach(48)` reports **130 m**. `time(48,129.999)` also returns infinity; `time(48,125)` is finite. Invalid-input probing additionally finds `time(30,NaN) == 0`, because `d.max(0)` turns the invalid distance into zero; no natural-game NaN input was established.

**Suggested fix:** Select exact interpolation endpoints before doing arithmetic involving potentially infinite neighbors, for both axes. Reject non-finite invalid inputs explicitly instead of allowing NaN distance to masquerade as a zero-length throw. Test the maximum-speed edge, partially unreachable cells, and the intended invalid-input contract.

## Verification

- Ran `cd sim && cargo test --workspace --release`: **117 tests passed**, zero failures; doc tests passed. The first invocation failed during linking because Zig tried to write its default cache outside the writable sandbox. Re-ran successfully with `ZIG_GLOBAL_CACHE_DIR` and `ZIG_LOCAL_CACHE_DIR` under `/tmp`; no privileges or source changes were needed.
- Built the release CLI and ran `BB_DEBUG_LIVE=1 sim/target/release/sim-cli games --n 512 --seed 2`: **5.05 runs/team-game**, **no rules warnings**, and no timeout diagnostics. The command was run from `sim` using its corresponding relative binary path.
- Created an isolated copy of the sim under `/tmp`, preserving the reviewed implementation, and added five temporary private live-module probes. They exercised receiver targeting, solver/table boundaries, deep-fly early assignment, and **96 held-ball scenarios** varying left/center/right holder, fielding location, and runner on first/second. All probes completed. The 96-case sweep found **zero stalls, zero duplicate Relay roles, and zero uncovered-base ticks**. This is targeted coverage, not proof for all possible plays. The existing successful relay-chain and backup/receiver precedence tests also passed in the unmodified workspace suite.
- Independently checked the nonmonotonic throw with `analysis/oracle/bbphys.py`, using mirrored Y/spin signs to use its decreasing-plane event, default air density, 22 s spin decay, and a 1 ms integration step. Oracle target heights for 29°, 31°, 33°, 35° were respectively **1.031854, 1.571899, 1.592095, 0.995909 m**; a separate angle bisection located the chest-height root quoted above.
- Ran matched **512-game** batches for each of seed sets 2 and 5 in the isolated copy, enabling/disabling only relay assignment via a temporary guard. All **2,048 games completed**, with zero warnings. Results reproduce the author's measurements:

| Seed set | Relays enabled, runs/team-game | Relays disabled | Paired difference | Estimated standard error of difference |
| --- | ---: | ---: | ---: | ---: |
| 2 | 5.054688 | 4.988281 | +0.066406 | 0.053120 |
| 5 | 5.004883 | 4.883789 | +0.121094 | 0.049920 |

  The increase alone does **not** identify a defect: another catch/transfer and execution error can cost the defense even when a belief predicts a faster route. It should not be dismissed confidently as noise either: set 5 is about **2.43 standard errors** above zero on this paired diagnostic. The established defects above should be fixed and the same batches repeated before attributing the difference to a particular mechanism or changing calibration. No reproduced timeout explains these numbers. The pre-drag 4.32 figure was not independently re-run.
- Repeated full games for seeds **1, 2, 7, 14, 128**, comparing parallel first runs with subsequent serial runs in the isolated copy: complete event-log bytes and digests matched exactly; all completed without warnings. The `OnceLock` initializer depends only on the fixed reference environment and ordered vectors. Inspection found no new production transcendental calls outside `bb_math::m`, unordered iteration, outcome rolls, RNG-purpose changes, or reused throw sub-indices. `throw_to` preserves speed-then-direction draw order and increments the throwing fielder's own `throws` once per release.
- Inspected ground conversion and capture deferral: throws stop at the physics ground event, pass the final velocity/spin to the free-ball model, and do not defer a capture across that endpoint. Relay throws admit only the designated receiver; base backups require passage beyond the receiver's receiving reach. The previous fallback-receiver/cutoff conflict is excluded by `receiver_for`. No additional demonstrated stall or wrong-fielder relay interception was found.
- Compiled and executed the **actual extracted C++ `StartReset` and `ResetFrame` bodies**, with small stand-ins for Unreal math/containers and synthetic ABI frames, in `/tmp`. Five cases covered a distant returning holder, side change, catcher supplying a new ball, pitcher already holding it, and a pitcher returning from far off the mound. Checked actor ordering and arrival at the next pitch's positions, the 16-actor cap, finite frame values, return-throw possession/release/hiding, and no return throw at a side change. All passed. Reviewed pause/speed progression, pending-play ownership, staging handoff, and the shadow's placement/visibility/scale by inspection. **Unreal was not rebuilt or rendered**; this does not establish animation or visual quality. No additional client finding was established.
- Ran `bash -n` on both launcher scripts. Installed copied scripts into an isolated `/tmp` directory with **a space in the repository path**, generated the desktop entry, and ran `desktop-file-validate` successfully. Mocked process/notification/dialog/client boundaries verified seed forwarding, successful client launch, client failure propagation, invalid-seed rejection, and missing-engine refusal without consent. No desktop menu entry or user configuration was installed outside the temporary directory; actual GUI launch and Rerun rendering were not exercised.
- `docs/agent-models.md §5.5` broadly describes the implemented drag/backspin, full effort, reference table, and receiver preference, but does not authorize the world-future leak in finding 1 and overstates the solver guarantee in finding 2. Section 5.7 matches the normal cutoff choices/spacing and relay route comparison, but its early-assignment claim fails as in finding 3. No additional spec discrepancy is promoted to a finding.
- Ran `git diff --check 1f7ca4c..HEAD`. Removed the isolated source copies, temporary probes, harnesses, binaries, and logs after recording the results. The pre-existing untracked `sim/log.txt` was left untouched. The only review-authored repository file is this document.

## Resolution

Resolved 2026-11-12: all four findings fixed. `cargo test --workspace --release` passes.

| # | Status | What was done |
|---|---|---|
| 1 | Fixed | The receiver no longer sees the realized path. He starts on the spot the throw was aimed at; after his reaction time he reads the throw every 0.25 s with the batted-ball read model (a noisy estimate of its current position and velocity, error growing with distance and shrinking with viewing time, fresh error draws per throw from `FielderPerception` sub-index 1 + throws so far), flies that estimate with his own throw physics and typical backspin, and goes to the catchable point nearest the aim. The realized path is used only to move the ball and for captures. Test: `receiver_plans_from_his_read_not_the_throws_future` (the review's 35 / 37 / 39 m/s case: on the bag at release whatever the executed speed; a throw dying short draws him in after he reads it) |
| 2 | Fixed | `solve_throw` now searches upward in 5° steps for the first angle that reaches the target, starting 6° below the vacuum answer, and refines that bracket; the steepest angle falling short no longer means out of reach. Test: `a_long_throw_whose_height_peaks_below_the_steepest_angle_is_solved`, against the review's oracle root (29.784°, 5.1688 s) |
| 3 | Fixed | "No play on it" is now the man's own read: his intercept is later than his believed first bounce. Test: `relay_man_goes_out_early_on_a_deep_fly` (the review's scenario), plus a short fly that keeps everyone on the ball |
| 4 | Fixed | `ThrowTable::time` never lets a neighbour with no weight into the arithmetic, and non-finite input is out of reach. Test: `the_table_edges_are_finite_where_the_throw_carries` |

The relay cost the review asked about, re-measured after these fixes (512 games each): seed set 2, 5.14 runs/team-game with relays against 4.99 without; seed set 5, 5.05 against 4.91. Experiments, each on both seed sets: throwing direct instead of through the relay man changes nothing, so the route choice is not the cause; with cutoffs but no deep relay man, runs equal no relays, so the cutoffs are neutral and the deep relay man (beyond 80 m) is the whole cost; letting him go out only after the ball is down changes nothing. Three behaviours found on the way were fixed because they looked wrong, though none moved the number: an infielder with the ball (the relay man himself) set off a second cutoff that took the first baseman off his base; a relay man stayed on his line after a throw went past him instead of becoming free to back it up; and a middle infielder could be sent out while his own read said he could cut the ball off first. **Still open:** why the deep relay man costs about 0.1–0.15 runs. Per the owner (2026-11-12), players should not play perfectly but should look believable, so this is a diagnosis item, not a calibration target.

Per-game time after the fixes: 218 ms (205 before them, 199 before the session); the solver's search and the receiver's reads cost the difference.
