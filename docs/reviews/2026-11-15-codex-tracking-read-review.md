No P1 found; do not merge.

## What was reviewed

Reviewed `2657f65..801c7c1`, agent-models §5.1–5.2, the calibration note, and the metrics-ledger row. HEAD `1ae47d3` adds only the review prompt; its simulation code equals the candidate. Read `CLAUDE.md` first. No source, tests, specs, ledger, or TASKS edits; no commit. Diagnostic harnesses and logs are under `sim/target/`.

The main invariants hold in the reviewed paths. Tracking receives noisy current position/velocity observations and the contact cue, then uses its own estimated state and believed spin. It does not consume true spin or the true landing point. World environments have zero `extra`; the two nonzero constructions are local observer copies. `FielderLook = 16` is appended, with draws keyed by seed/play/fielder/look; existing draw sequences are unchanged. No new hash iteration, direct transcendental calls in simulation logic, or outcome roll was found. Capture remains physical glove geometry. Replay and isolation tests pass.

The constant-acceleration transition, integrated process-noise coefficients, and sequential Kalman updates are algebraically consistent with the specified approximation. The sight-frame axes and covariance anisotropy are correct. A five-second tracking probe retained symmetric, positive-definite covariance and finite states. The fresh-noise covariance excludes persistent bias, so adding a separate bias allowance is not inherently double counting; the incomplete propagation below is a separate problem.

## Findings

### P2-1 — Landing doubt omits uncertainty in the acceleration used to fly the estimate

**Location:** `sim/crates/bb_game/src/live.rs:1980`–`1987`; matching assertion in `docs/agent-models.md` §5.1, “His doubt.”

The landing function depends on all nine state components, but its variance uses only the position/velocity submatrix. Covariance with those components does not substitute for the missing acceleration Jacobian: the omitted terms are `Ja Paa Jaᵀ`, `2 Jr Pra Jaᵀ`, and `2 Jv Pva Jaᵀ`. The comment claiming otherwise is mathematically incorrect.

**Evidence:** `review_probe_covariance` gives a valid belief with uncertainty only in acceleration. Reported doubt is exactly zero, although ±1 m/s² along Y changes its predicted landing by **2.631 m**. On a normally initialized 45° tracking trace, using the same finite-difference scheme and persistent allowance, reported versus full nine-state landing σ is **3.47 versus 11.85 m** at one second after contact, **1.74 versus 4.24 m** at two seconds, and **0.43 versus 1.26 m** at four seconds. This feeds both sure calls and the arrival margin. These are errors relative to the filter's own covariance, not a claim that those larger values equal measured real error.

**Suggested fix:** Differentiate landing with respect to acceleration too and propagate the complete covariance, including cross terms. Correct the spec's explanation and add an acceleration-only uncertainty case plus an independent tracking-error calibration test. The existing `a_fielders_doubt_matches_the_spread_of_his_reads` at line 7786 exercises the old snapshot read, not `track_doubt`.

### P2-2 — The first-look shrink leaves an unshrunk grounder error floor

**Location:** `sim/crates/bb_game/src/live.rs:1900`–`1912`.

The first-look exception shrinks fresh errors, while the persistent position and velocity errors still use plain `base_p`/`base_v`. For a grounder close to its first bounce, the persistent component can therefore greatly exceed the entire old first-read error. This follows the spec's general “plain σ” bias sentence, but conflicts with its first-look exception and the note's claim that the old grounder precision was preserved. The unexplained grounder increase has at least this concrete mechanism to investigate.

**Evidence:** `review_probe_first_look_and_lifecycle`, shortstop at his reaction gate, contact velocity `(5,36,-1)` m/s: shrink **0.0241**, so the persistent contribution alone is **8.31 times** a fresh one-σ contribution. With lateral position/velocity draws of one σ and fresh noise disabled, the new position error is **0.0741 m**, versus **0.0100 m** for the base snapshot read. At vertical velocity zero the corresponding errors are **0.0741 versus 0.0204 m**. This does not quantify how much of the aggregate +.007 GB-hit change it explains.

**Suggested fix:** Apply the first-look shrink to its persistent contribution as well, retaining the intended later-look bias model; reconcile the conflicting spec statements. Add a grounder comparison against the base read with controlled independent draws, rather than checking only fly convergence.

### P2-3 — A reach achieved during stepping is discarded before the arrival-time search

**Location:** `sim/crates/bb_game/src/live.rs:1506`–`1512`; coverage gap at line 6331.

When stepping gets within reach before the queried horizon, `left <= reach` enters the outer condition but does not return. The closed form then predicts another turn from that already-reached state. `run_gap` can become positive again, and the binary search in `run_time_to` selects a later crossing. Calls and teammate arrival estimates consume that result.

**Evidence:** Average outfielder at `(0,80)`, velocity `(4,-4)` m/s, goal `(1,80)`, reach 0.7 m: the actual late-chaser movement first reaches it in **0.100 s**; the candidate estimates **1.327 s**. Its gap is negative at 0.10 s but positive at 0.50 and 1.00 s. The base already had a related nonmonotonic closed-form problem (**0.962 s**), so this is a worsened existing limitation, not an entirely new failure. A 1 m wall-approach case estimates **1.634 s** versus **0.117 s** with the movement and wall rules applied. The tightened test bound is honest for its grid, but that grid starts at 2 m and misses these cases.

**Suggested fix:** Preserve the first stepped entry into reach for arrival-by-time queries, and make the arrival search find the first crossing. Extend the independent movement comparison below 2 m, including oblique velocities and wall approaches. The claimed short-range closure is premature.

## Verification

- One `cargo test --workspace` pass: **414 passed, zero failed, four ignored**. Includes deterministic digest comparisons across threads, future-path isolation, the owner-contact catch, arrival-margin behavior, physics/oracle checks, and rules scenarios. All builds/tests/simulations used `taskset -c 0-7 nice`, `CARGO_BUILD_JOBS=2`, and the requested PATH/Zig caches; heavy jobs ran sequentially.
- One standard candidate run, `cargo run --release -p bb_cli -- games --n 300 --seed 0`: **3.93 runs/team-game**, K 22.3%, BB 8.4%, HR/PA 2.64%, reached/BIP 34.6%; GB/LD/FB/PU hits **.298/.658/.125/.016**; **rules warnings `{}`**. This reproduces the unbracketed candidate measurements. The base aggregate, `--seed 1000`, and routine-fly population sweeps were not rerun.
- Isolated probes: `sim/target/tracking-review-probe/results.log`, `reach-results.log`, and `base-results.log`. Replaying base seed 23080 through play 8 confirms every preserved ball-state, contact-time, pitch, bat-yaw, swing-lead, handedness, situation, and fielder-ID value in the regression fixture. The base does not catch it; the candidate's regression passes. This is an honest preserved-contact test.
- Bounce and deflection probes reject the old track; a normal read clears its doubt. Walls also terminate tracking, and balls beyond wall reach remain excluded by planning/capture. There is no modeled visibility/lost-ball state: reacted fielders keep looking regardless of role; skipped looks predict forward rather than reacquiring from scratch. No NaN appeared in the exercised paths. The arrival margin is released when the fielder ceases to be early; its fixed-target test passes, but the reach defect above limits confidence in the combined behavior.
- The soft-contact spin bound now tolerates a documented, existing speed-dependent prior error; it does not establish tracking correctness. The reach bound was tightened, not loosened. Worse late lateral error alone is consistent with replacing the old closing factor with range-limited looks and persistent error, rather than evidence of an axis/frame defect. A 40-draw typical-spin control half a second out measured projected error SD **0.47 m**, versus reported RMS doubt **0.80 m**: the covariance omission does not imply universal overconfidence or by itself explain that late-error regression.

## Resolution

All three P2s fixed by the author (2026-11-15) in `c93288e`, with the spec (agent-models §5.1, §5.2) in the same commit. Then one test pass (all pass) and one measurement (note §3a).

- **P2-1 (landing doubt omits the acceleration): fixed.** `track_doubt` differentiates the landing point with respect to all nine state components, including the learned acceleration, and carries the full covariance with every cross term. The spec's incorrect sentence is replaced.
  - `a_tracked_doubt_carries_the_learned_acceleration`: a belief uncertain only in the acceleration, 1 m/s² per axis, has the doubt a ±1 m/s² change gives the landing point, within 15% on each axis.
  - `the_tracking_doubt_matches_the_spread_of_tracked_reads` is the independent calibration. It uses 86 balls from games 0–5 with their real spins, each tracked from 12 m short. RMS doubt is 1.10 m against an RMS miss of 0.78 m 2 s out, and 0.62 against 0.53 m 1 s out. The required band is a factor of 1.5 either way. The doubt is now on the cautious side; before the fix it was 3–4× too small early in flight, by the review's probe.
- **P2-2 (the first-look shrink leaves an unshrunk grounder floor): fixed.** The first look's persistent part takes the shrink with its fresh part; later looks keep the plain σ. The spec's two statements are reconciled.
  - `a_grounders_first_look_is_read_as_sharply_as_the_snapshot_read` reproduces the review's case: the shortstop at his gate, a grounder at 36 m/s with vz −1, 0 and +2, lateral draws at one σ, fresh noise off.
  - The tracked position is 0.002–0.009 m off, against the snapshot read's 0.010–0.051 m; it was 0.074 m.
  - Grounder hits afterwards are .300 [.316], against .298 [.303] before. So the mechanism is fixed, but the aggregate did not fall back: the `--seed 1000` set's +.013 is not explained.
- **P2-3 (a reach found while stepping is discarded): fixed, beyond the suggestion.**
  - Within 5 m, moving, the estimate steps the movement until he is within reach (at most 2.5 s), with the walls applied as in `move_fielders` (`wall_rule`, now shared); `wall_delay` adds nothing there.
  - `run_time_to` takes that first entry. Otherwise it scans the first 3 s at 0.05 s for the first crossing before bracketing.
  - `run_gap` and `run_time_to` now take the field.
  - The test `reach_estimate_tracks_the_movement_step_below_two_metres_and_at_a_wall` covers 0.75–1.9 m and every oblique motion up to 8 m/s, on open ground and into the centre-field wall, within ±0.35 s; the worst is 0.18 s.
  - The review's case (1 m, (4, −4) m/s) now matches the movement. Switching to the closed form when he was heading for the goal had also failed at the wall: 1.39 s against 0.68 s, because the stepped run went through the wall's line. That is why the walls are now in the step.
- **Verification.** One workspace test pass (all test groups pass), then one standard run on both seed sets and `fly_misses 0 300`.
  - Routine flies caught .965 (in the owner's band; `LOOK_SCALE` and `TRACK_BIAS_FRAC` not refitted).
  - Runs 3.83 [4.23]; no rules warnings.
