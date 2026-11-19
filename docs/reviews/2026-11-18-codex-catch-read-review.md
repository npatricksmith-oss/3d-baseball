# Codex review of the runner’s catch and throw reads

Reviewed `bc69e96..9871b05` on `work/runner-catch-read`.

## Findings

### P1-1. The catch uncertainty still depends on the exact ball state

[`live.rs:2343`](../../sim/crates/bb_game/src/live.rs#L2343) · [`live.rs:2290`](../../sim/crates/bb_game/src/live.rs#L2290) · [`live.rs:1503`](../../sim/crates/bb_game/src/live.rs#L1503)

The first read gets four keyed errors. The eight alternative trajectories each go back through `ball_at` with an altered error vector. Their directions, scales and plate weighting depend on the true range, speed and descent of the ball. In the geometry probe, true depths of 20 and 21 m and valid observation errors produced the same noisy position and velocity to floating-point precision. Against the same visible fielder the catch odds were 0.199816 and 0.200806, on opposite sides of the advance cutoff. The hold point comes from the central projection with typical spin. Whether the runner uses it depends on those truth-conditioned alternate paths. The central projection also takes the exact contact location and uses the true path’s origin to decide whether the ball is still off the bat. A glove-rim deflection replaces that path without a recorded ground or wall fact. [Deflection and replacement](../../sim/crates/bb_game/src/live.rs#L4041)

Generate one contact observation at the world boundary. Build every alternative from that observation and its uncertainty estimated from the read. Give it an observed contact location or a plate prior and public touch information. Add an isolation test that fixes the observation and changes the underlying flight while requiring the same belief, action and hold point.

### P1-2. The holder forecast still reads the actual throw’s destination after reception

[`live.rs:3994`](../../sim/crates/bb_game/src/live.rs#L3994) · [`live.rs:4022`](../../sim/crates/bb_game/src/live.rs#L4022) · [`live.rs:4364`](../../sim/crates/bb_game/src/live.rs#L4364)

The runner reads `fd.face` both as an orientation and as a switch between ordinary and relay transfer. The capture step sets it from the true incoming velocity or clears it based on the actual throw type. On a feed, whether it is populated also depends on the actual `to_base` and the force. In the probe a stationary second baseman caught the same trajectory at the same point with the same noisy runner look and the same catch velocity. Changing only the throw’s intended base from second to third changed the time the runner gave the ball at first from 6.463 s to 6.328 s. A 0.135 s difference can flip a marginal advance. The visible stretch and lunge cues themselves were independent of `up_at` and `ready_at`.

Publish a holder observation with a visible heading, motion and posture. Let the runner infer the feed or relay from what he saw, then use a typical transfer for that posture. Extend the timer isolation test to vary the incoming throw’s private aim and route while holding the observation fixed.

### P1-3. Traffic can react to a catch before the runner sees it

[`live.rs:2071`](../../sim/crates/bb_game/src/live.rs#L2071) · [`live.rs:4441`](../../sim/crates/bb_game/src/live.rs#L4441) · [`live.rs:4948`](../../sim/crates/bb_game/src/live.rs#L4948)

`awaiting_catch_call` gates the relook and the base-arrival decision. The traffic checks still use `forced`, which reads the batter’s same-tick out flag. In a fixture with a runner heading from first to second and a stationary lead runner at second, emitting the catch switched the trailing runner’s goal back to first and started braking him on the catch tick. With the same runner motion and no catch, he accelerated toward second. `catch_seen` was false in both cases. A waiting runner can also reach `runner_decide` during the window. The rules should record the out and remove the legal force immediately. Base traffic and advance decisions need the runner’s view of the force and catch during the reaction interval. Use the same perceived ruling for these decisions and preserve the set-start exception for an anticipated tag-up.

### P2-1. The hold budget starts from rest even when the runner is still moving

[`live.rs:2417`](../../sim/crates/bb_game/src/live.rs#L2417) · [`live.rs:2445`](../../sim/crates/bb_game/src/live.rs#L2445) · [`live.rs:4988`](../../sim/crates/bb_game/src/live.rs#L4988)

The bisection bounds the return time from a stationary hold. At the first look a runner from first is already 8.0 m off at 3.0 m/s. In a close-catch fixture with a 30 m throw to first, the calculation gave him a 2.71 m hold. He drifted out to 8.92 m and retouched 2.82 s after the catch, against 1.64 s for the modeled transfer and throw. A catch while he approaches a farther hold can likewise cost forward braking and drift. When the central projection supplies no catch point, `read_contact` falls back to the full halfway limit. The trap test exercises this bypass by passing a close belief with no catch point.

Use a catch time and an appeal origin from the same observed trajectories that supply the belief. Evaluate the hold with the runner’s signed speed at that time, including the reaction and braking drift. When the observed scenarios give no recoverable partway position, start braking back immediately. Make the movement step respect the available braking distance. With a target 0.13 m ahead of the first-look position, the clamp planted the runner from 2.84 m/s in one tick.

### P2-2. A runner who retouches during the reaction window can lose the next decision

[`live.rs:4878`](../../sim/crates/bb_game/src/live.rs#L4878) · [`live.rs:5040`](../../sim/crates/bb_game/src/live.rs#L5040)

A runner already returning for reasons other than `tag_up` can legally touch his original bag between the catch and his reaction. The touch consumes no advance decision while the call is pending. When the reaction window ends, the catch handler sees that he has already retouched and skips the branch that would give him a new decision on arrival. On a catch 120 m from home, a fast runner who retouched third 0.067 s after the catch stayed there without a decision. Moving him a little farther from the bag at the catch delayed his return to 0.333 s, and he decided to go home. Run the normal post-retouch decision once the call is seen if the runner has already legally returned and is still planted. Preserve the physical return and the legal time of the touch.

### P2-3. The height adjustment creates a discontinuity in the catch belief

[`live.rs:2372`](../../sim/crates/bb_game/src/live.rs#L2372) · [`live.rs:2387`](../../sim/crates/bb_game/src/live.rs#L2387) · [`agent-models.md:676`](../agent-models.md#L676)

In an isolated projection with a fielder underneath the ball, moving a single sample from 1 mm below the 2.4 m ceiling to 1 mm above it changed the catch score from 0.9996 to 0.4985. The low side gives full height credit and the high side starts at about half. That jump enters each of the eight paths and can change a return into a partway hold. The branch’s stated concern about double-counting uncertainty is valid. The current join creates new sensitivity at the boundary. Apply spin perturbations to a path through time, or use a continuous joint reach-and-height model with an explicit dependence assumption. The scan also uses the ordinary ceiling near a wall, where physical capture uses 3.0 m. Include a fence catch when checking the replacement model. [World catch ceiling](../../sim/crates/bb_game/src/live.rs#L1842)

### P3-1. The catch thresholds have only a limited run-value justification

[`live.rs:117`](../../sim/crates/bb_game/src/live.rs#L117) · [`live.rs:2442`](../../sim/crates/bb_game/src/live.rs#L2442) · [`agent-models.md:676`](../agent-models.md#L676)

All four values are named [E] tuning constants. The run-value illustration gives 12.4% with nobody out and 17.9% with one out from first. It assumes a full start turns one fifth of singles into first-to-third gains. The 20% cutoff already exceeds both break-evens in that scenario, and the example gives no independent basis for the 80% return threshold, other base states, or the cost of a half-start. The 0.30 m glove width is an explicit unsourced proxy. The 0.45 m/s prior corresponds to 1.48 ft/s and is in scale for player sprint speeds. Savant’s published metric measures selected baserunning sprints and supplies a league mean and range. It leaves the spread of a runner’s uncertainty about a particular fielder’s reach open. The ±2σ points with weights of 1/8 reproduce the mean and unit variance of four independent unit-variance axes. Their average of the strongest catch score along each flight is a heuristic, especially near a discontinuity. Record that distinction and assess calibration across catch types and base/out states before treating the thresholds as probability cutoffs. [Savant sprint-speed definition](https://baseballsavant.mlb.com/leaderboard/sprint_speed)

### P3-2. The new tests miss the observation and motion gaps

[`live.rs:10173`](../../sim/crates/bb_game/src/live.rs#L10173) · [`live.rs:10324`](../../sim/crates/bb_game/src/live.rs#L10324) · [`live.rs:10455`](../../sim/crates/bb_game/src/live.rs#L10455)

The low-liner fixture checks real glove possession, a rules catch and a slowdown before it. The visible-momentum fixture has a geometric oracle, and the two outfield liners check legal appeal outcomes over fixed keys. The trap fixture injects `None` for the catch point, exercising the fallback hold and no-catch clock. The return-distance fixture puts the runner at the calculated hold with zero speed. It passes even when a moving runner on the same kind of play exceeds the budget. The throw geometry sweep supplies the true instantaneous motion to the projection helper. The separate noisy-read test checks key repeatability and a late-flight majority. Its helper launches the throw without recording a release, so the test leaves the viewing-time shrink unexercised. Add matched-observation tests for P1-1 and P1-2, a traffic test on the catch tick, a close-catch fixture starting at the standard moving lead, and a throw fixture with a recorded release that checks both base races from the same observation.

## Verification

- Ran `taskset -c 0-7 nice cargo test --offline --workspace --release` with `CARGO_BUILD_JOBS=2` in `sim`. All 450 executed tests passed and 4 were ignored. This included the crate boundary, cross-thread digest equality, runner relook replay and the focused catch and holder fixtures.
- Ran release probes of matched noisy contact observations, a hidden throw destination with fixed trajectory and reception, a catch tick with two runners, early and late retouches, the moving hold, and the height boundary. These produced the measurements in P1-1 through P2-3. The probes used source copies under `sim/target/codex-probes/`.
- Ran mutation controls in a separate release test binary built under `sim/target/codex-probes/`. The respective fixtures failed when I delayed the first look to 1 s, removed visible fielder momentum, allowed catch credit after the bounce, restored exact throw motion, restored the private holder timer, or restored the unconditional halfway target. The two-liner test still passed without the eight read-error points alone. With those points and the spin and speed spreads disabled, it failed on 19 appeals in 23 catches for the first liner. It guards the combined outcome, with room for other belief models to pass.

The changed live paths use `KeyedRng`, keeping the registry unchanged. Contact uses sub-index 1 with four truncated-normal draws in the original order. Decision `n` starts at 1 and uses `2n` for its timing draw and `2n + 1` for four ball draws. Both estimates of a thrown race rebuild the same keyed look, which gives the same observation. The eight alternate paths add deterministic offsets and consume no RNG. The arm stream at `2^20 + fielder index` is outside the reachable decision range in a 40 s play. The older, separately invoked `play::simulate_play` has its own pre-existing use of purpose 8 and is outside the live game path. I found no new execution-noise draw, outcome roll, direct transcendental call outside `bb_math::m`, unordered iteration or `mul_add` in the change. The thrown projection uses typical spin, and the catch scan stops at its projected first ground or wall touch. [RNG allocation](../data-formats.md#L66) · [Live draw order](../../sim/crates/bb_game/src/live.rs#L2307) · [Throw forecast](../../sim/crates/bb_game/src/live.rs#L4285)

A close hold is set once and blocks the normal between-base relook until it clears on a ruling. `MAX_TURNS` caps trapped-runner relooks, outside that hold, so it does not repeatedly switch the hold on and off. `settle_guard` uses base goals, and motion uses the partway target while the hold is active. The hold’s target remains short of the next base with the standard leads and speed. That protects the bag geometry, while P2-1 shows why it does not guarantee enough time to return. The planted tag-up uses its existing anticipated-start clock. The client’s out-signal clock is presentation-only, so it provides no independent timing gate for this review. [Motion and traffic](../../sim/crates/bb_game/src/live.rs#L4934) · [Settle guard](../../sim/crates/bb_game/src/live.rs#L4614) · [Out signal](../../sim/crates/bb_game/src/umpire.rs#L1)

### Paired game and event-log probe

I built the two revisions in `sim/target` and ran 300 paired games for each standard cohort sequentially under `taskset -c 0-7 nice`. The cohorts used direct game seeds starting at 0 and 1,000,003,000. I compared persisted logs and the state at the end of each pitch until inning, half, count, bases, outs, score or the next batter first diverged. Both runs used the CLI’s average fielders. The game logs diverged in every pair, and neither build reported a rules warning.

| CLI seed | Runs per team-game, base → tip | Paired change (SE) | Games with a state change | Median first changed pitch index |
| --- | ---: | ---: | ---: | ---: |
| 0 | 4.330 → 4.383 | +0.053 (0.080) | 265/300 | 70 |
| 1000 | 4.308 → 4.533 | +0.225 (0.086) | 272/300 | 60 |

The difference between the two paired effects is 0.172 runs per team-game, about 1.46 standard errors at the observed spread. At first state changes that were also the first changed runner or scoring events, the immediate run changes were +7 and +8. Full games ended +32 and +135. The mechanisms changed real plays, but this comparison does not attribute the wider cohort gap to any one bug.

- Seed `1000003000`, play `82` had the same contact, catch and first throw. The fly stayed up for 5.683 s. On the base revision the runner from third was planted at the catch and scored. On the tip he was still holding 9.50 m off, retouched 2.67 s after the catch and stayed on third. The single contact belief kept him in the hold throughout the flight. The base revision had one more run on that pitch and two fewer total. Seed `190`, play `11` also lost a run on the catch at the tip, which finished with 11 more runs across both teams.
- Seed `67`, play `0` and seed `1000003002`, play `3` had matching contact, first-base touch and outfield throw and reception. The tip turned the first case’s single into a tag out trying for second. In the second case the batter reached third after stopping at second on the base. Both had empty bases at contact, isolating a path other than the existing runners’ catch belief.

## Resolution

Lead, 2026-11-18. My fresh run after touching the changed sources: `cargo test --workspace --release` passed 450, failed 0. The standard runs match the worker's figures (runs 4.38 [4.53], HR/PA 2.73% [2.93%], K% 22.2 [22.1]). Merged into `main` with the findings open: the branch passes its tests, fixes the contact-read review's P2-1 to P3-1 and adds the owner's close-catch rule, and the fix job touches the same `live.rs` either way, so holding the merge unblocks nothing. Seed 1000's +0.22 runs is 1.46 standard errors from seed 0's paired change, with real play changes and no traced bug; it is left as noise unless the fixes below do not bring it back.

- **P1-1, P1-2, P1-3: open, next job** (one Codex worker, branch from `main`). The runner's catch belief must come from one contact observation and its own uncertainty, not truth-scaled alternative paths; the holder forecast must come from a published holder observation, not `fd.face`; base traffic must use the runner's perceived ruling during the reaction window. Matched-observation tests as the reviewer describes.
- **P2-1: open, next job.** Hold budget with the runner's signed speed and braking drift; brake back at once when no recoverable partway point exists; no one-tick plant from speed. Includes the runner from third holding 9.5 m off on a 5.7 s fly (seed 1000003000 play 82), which a real runner would not do.
- **P2-2: open, next job.** A runner who legally retouches during the reaction window gets his post-retouch decision when the call is seen.
- **P2-3: open, next job.** Remove the step at the 2.4 m catch-height ceiling; use the 3.0 m wall ceiling near a fence.
- **P3-1: deferred** (TASKS line). Calibrate the 20%/80% cut-offs across base/out states and catch types after the P1 fixes change the belief.
- **P3-2: folded into the next job's tests**; the moving-lead close-catch fixture and the recorded-release throw fixture go with P2-1 and P1-2.
