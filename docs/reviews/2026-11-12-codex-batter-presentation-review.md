# Batter presentation review — 2026-11-12

Reviewed `61ff69a..bb1e248` (one commit), against `CLAUDE.md`, the relevant PLAN/specification sections, and the Unreal playback path. **The original release-time bat pose and early award-position drift are fixed with visualization recording enabled. The broader “nothing before commitment/the call” claim is not established.** No P1 finding.

## Findings

### 1. P2 — Starting the load 50 ms before the final swing can reveal the decision before commitment

**Location:** `sim/crates/bb_game/src/timeline.rs:220–230`; causal timing in `sim/crates/bb_game/src/pa.rs:437–448, 477–495, 527–531`.

`LOAD_S <= perception.latency` does not imply `t_start - LOAD_S >= actual_commit`. Commit uses the posterior's weighted plate-arrival estimate, and occurs on an observation tick. The swing instead targets the dominant component at contact depth, adds executed timing noise, and can have its entire contact time shifted again by subsequent observations. None of those operations preserves that inequality. The new load exposes this difference as pre-decision motion, including the body turn the owner complained about.

**Reproduction:** Average matchup with `Knobs::calibrated_m3()`, seed 11, play index 2, count 0–0, scouting from `scouting_for_pa(&mu, 11, 2)`, using `simulate_pitch`:

| Quantity | Seconds after release |
| --- | ---: |
| Actual decision/commit tick | 0.280000000 |
| Weighted arrival at commitment | 0.475077814 |
| Initial contact-depth aim time | 0.467762167 |
| Initial executed swing start | 0.310562309 |
| Final executed swing start | 0.300212757 |
| Encoded load begins | 0.250212757 |

Executed timing error is **−7.200 ms**; in-flight adjustment moves the start another **−10.350 ms**. At encoded time **0.26521274 s**, still before commitment, the sampled bat's horizontal reach is **0.455229 m**. That exceeds the client's 45 cm threshold in `BbDirector.cpp:669` and begins turning the stationary batter's body. His knob has also moved from the stance.

A diagnostic sweep of 2,000 independent 0–0 pitches with that setup produced 1,114 swings. Of those, **690** final starts precede commit + 50 ms; **543** have a non-stance load sample before commitment; **229** cross the client's body-turn threshold before commitment. The largest load-start lead over commitment was **45.407 ms**. This is observable behavior with ordinary attributes, not only a configurable edge case.

Carry the actual decision timestamp into presentation and constrain the load using it, shortening the presentation transition when necessary while preserving authoritative swing/contact poses. Do not infer commitment from the final motor trajectory. Any underlying execution that itself predates its permitted motor start needs a separate causal-model correction, not a visual displacement of contact.

The new test misses this because its cutoff is **the encoder's own `SwingStart` minus the encoder's 50 ms load duration**, rather than the decision time. Its upright-bat geometry is an independent visual assertion; its claimed causal boundary is implementation-derived. The spec and TASKS statement that the load always lies within the decision-to-motor window should be corrected with the fix.

### 2. P2 — The unchanged HUD still announces future contact during the wind-up

**Location:** `client/Source/BaseballClient/BbDirector.cpp:896–908`; metadata emitted by `sim/crates/bb_game/src/timeline.rs:285–355`.

This is a **pre-existing consumer defect left unresolved by this commit**, included because the requested review explicitly covers the owner's end-to-end complaint and this client path.

`Hud->PitchLine` displays exit velocity and launch angle whenever the complete timeline has contact telemetry. It has no playback-time guard. Playback starts at **−1 s**, so the viewer learns that the batter will swing and make contact during the wind-up, even with a perfectly neutral stance. For example, seed 0, play 3 is `Single to short`, exit velocity **46.195915 m/s**, launch angle **−0.1638543 rad**, contact at **0.404443288 s**. Those measurements are already available to this unconditional HUD branch before release.

The call and post-play scoreboard/base state have a separate guard, but noncontact results use the fixed **0.55 s** threshold rather than a pitch-specific call time. Seed 0, play 68 is a walk whose pitch ends at **0.443005633 s**: under this change's own call convention the signal arrives at **0.643005633 s**, yet “Walk” and the resulting view state appear at 0.55 s. Thus the runner-position fix does not make the displayed decision wait for the call.

Gate contact measurements on actual contact, and judgments/resulting view state on their corresponding observable times. Storing future markers, telemetry, summaries, and `view_after` in a complete offline replay is expected; showing those future values immediately is the leak. `SwingStart` and `Contact` markers must likewise be consumed according to their timestamps, not their mere presence.

### 3. P3 — Award hold positions still expose future movement through root yaw and velocity

**Location:** `sim/crates/bb_game/src/timeline.rs:175–177, 193–196`; `sim/crates/bb_ffi/src/lib.rs:270–273`.

The added hold knot preserves position, but its centered-difference velocity looks ahead into the step onto the path. Its yaw consequently points toward that future step. The FFI changes to the next knot's yaw halfway through the preceding interval, including while position is held. The new regression test checks only position.

**Reproduction:** Seed 0, play 68, batter 262:

| Knot time | Position (m) | Velocity (m/s) | Yaw (rad) |
| --- | --- | --- | ---: |
| 0 | (−0.95, 0.30, 0) | (0, 0, 0) | 0 |
| 1.0430056 | (−0.95, 0.30, 0) | (0.7857434, −0.24812949, 0) | 1.8766752 |
| 1.2090461 | (0, 0, 0) | (5.16562, 0.46708706, 0) | 1.4806193 |

At **0.54 s**, before the **0.643006 s** call, the sampled root is still in the box but has already turned to **1.8766752 rad**. All **97 walks** in the 16-game probe exhibited a pre-call root-yaw change. The client's primitive-body fallback uses this yaw (`BbDirector.cpp:873`); the skeletal figure path computes its own facing and therefore does not inherit this particular turn.

There is also a format-contract mismatch: `docs/data-formats.md:287` specifies Hermite root interpolation. With the emitted endpoint velocities, that same supposedly stationary interval would already displace the batter **11.1 cm** at 0.54 s under Hermite interpolation. Today's FFI uses linear positions, so that displacement is not present in the current client.

Preserve a stationary root pose through the hold, including its facing, and give hold endpoints velocities consistent with the supported interpolation contract. This is a residual information leak; the old encoder also exposed award movement/facing early.

## Verification

### Tests and probe isolation

- Checkout was clean at `bb1e248140f85638f4c2252d50e67eee401420dc`. Read `CLAUDE.md` before reviewing the diff. No repository source, test, specification, or TASKS file was edited. This review is the only repository file authored.
- Ran `cd sim && cargo test -p bb_ffi`: **3 passed**, including the new regression; doc tests passed. The first attempt failed because Zig tried to write its read-only home cache. Retried successfully with `ZIG_GLOBAL_CACHE_DIR=/tmp/codex-batter-zig-global` and `ZIG_LOCAL_CACHE_DIR=/tmp/codex-batter-zig-local`; no authentication or privileged operation was needed.
- Ran `cargo test -p bb_game`: **32 unit tests and 20 integration tests passed**; doc tests passed. This includes the crate dependency checks, same-seed cross-thread digest/log equality, and recording-independence test.
- Built standalone Rust probes in `/tmp/codex-batter-probes` against the checkout's compiled crates. The timing diagnostic extracted `pa.rs` into a temporary module and added observation-only captures of the actual commit tick, initial aim/start, timing deviation, and final start. For all 2,000 pitches, its swing, release, and contact flag matched the unmodified public production function. Presentation checks used the production encoder and FFI sampler.
- Compiled the **exact new FFI test** with only its encoder call redirected to the `61ff69a` encoder extracted into `/tmp`. It **fails on play 0 at t=0**, with bat axis `(−0.80146766, −0.27662292, 0.16411662)`. Thus the new test really catches the old stance bug. A separate walk-only comparison against that old encoder found premature offensive position movement on all 97 walks in seeds 0–15, independently confirming the award regression it addresses. Seed 11 contains eight walks, so the new test's own seed exercises that path too; its normal old-code execution aborts on the stance failure before reaching them.
- `git diff --check 61ff69a..bb1e248` passed. No Unreal build or visual watch was performed; client-display conclusions come from its actual C++ branches plus sampled timeline data.

### Blend and bat-track edge cases

- For ordinary non-antipodal unit axes, knob lerp and normalized axis lerp correctly reach the swing's starting knob/axis, preserve bat length, and leave the executed swing unchanged. This is **nlerp**, so angular speed is not constant. The current FFI holds bat samples rather than interpolating them: the load is a series of 5 ms pose steps, followed by the exact start sample. Its first load knot is still the stance; the first differing sample is normally 5 ms later.
- Positive starts shorter than `LOAD_S` compress the blend: a synthetic **25 ms** start gives load samples at **1, 6, 11, … ms**, then the exact start pose. At **0.5 ms**, no load sample fits and the bat snaps from its t=0 stance to the start pose. A synthetic **−10 ms** start produces decreasing bat times (`0, −0.01, −0.005, …`), because the new stance sample precedes earlier swing samples in the vector. No decreasing bat time occurred in the 16 ordinary games. Supporting pre-release swings would require an explicit ordering/pre-roll policy; the clamp alone does not provide it.
- With no visualization stance recorded, swings retain their previous first sample at `t_start`, and takes have no bat track. A probe yielded first swing sample **0.31944847 s** and no actors. Thus “every pitch opens in a stance” is conditional on recording; `bb_game_create` enables recording, so the owner-facing FFI path has it.
- A synthetic exactly opposite stance/start-axis pair remains finite, but nlerp flips **180°** across its zero-vector fallback. It is not a smooth antipodal blend. This case was not established for ordinary generated swings. A robust general blend would choose a deterministic perpendicular interpolation plane near opposition.
- Quaternion construction correctly uses `(cross(+X, axis), 1 + axis.x)` with normalization. Its pre-existing near-`−X` fallback returns a finite unit quaternion, snapping the axis to exactly `−X` within roughly **0.001414 rad** (at most about **1.22 mm** at a 0.8636 m tip). Probes at lateral axis components 0, 0.0005, 0.001, and 0.0015 produced finite results and the expected threshold behavior. Arbitrary roll around a cylinder's axis does not change the sampled bat endpoints.

### Awards, chronology, and speed

- Examined full timelines for seeds **0–15**: **225 awarded runner tracks** (147 walk, 2 hit-by-pitch, 76 home-run awards). There were **no decreasing actor times**. The 69 equal-time adjacent knots all had identical positions, so none was a teleport. No non-home-run out-of-play award occurred in this sample; that case was checked through the common encoder branch and formula, not claimed as game-probe coverage.
- Before awards are appended, `t_end` is at least every existing track's final time. Therefore the common `t_jog >= t_end + 0.4` hold cannot precede an existing knot for the normal event producer, which emits one dead-ball advancement per awarded runner. Updating `t_end` while appending different runners does not invalidate that bound. On-play advancement events are skipped.
- **Home runs and balls out of play are not wholly unchanged in presentation timing.** Their nominal jog time remains the old `t_end + 0.4` when that is at least pitch end + 0.6; otherwise the new global maximum delays even those awards. Additionally, all awards now hold at the last position until `t_jog`, then spend `distance(here, path)/6` reaching the path, shifting every subsequent knot by that duration. Seed 1/play 67 retains nominal jog time **5.282005 s**, but reaches the path at **5.296883 s** and finishes at **18.996881 s**, versus **18.982004 s** before.
- **33 awarded runner tracks** in the sample were moving faster than 0.5 m/s at their last realized segment. They now stop at that last point, hold, and restart; continuity of position is preserved, continuity of velocity is not. This affects post-dead-ball presentation, not the authoritative rules or live trajectory.
- The sprinter assertion remains meaningful: the step onto the path takes its distance divided by 6 m/s, rather than teleporting. Every positive-duration offensive root segment in the probe was at most **8.200050 m/s** after f32 encoding; the additional award segments fit the jogging bound. Corners and the final shortened stride cannot increase chord speed above path speed. The existing 50 ms sampling test is a useful visual regression test, though it cannot exclude every shorter-than-sample excursion by itself.

### Information boundaries and invariants

- With recording enabled, the t=0 stance does not depend on swing/take, check-swing status, or contact. Live offensive position tracks begin around contact rather than at commitment; award positions now remain held through the call. Remaining visible leaks are findings 1–3.
- The complete replay intentionally contains future bat samples, marker presence/times, batted-ball segments, contact telemetry, summary, and post-play view state. It is not a time-filtered agent observation. There is no dedicated check-swing decision field in this timeline; its eventual motion/judgment can be inferred from the completed data. Consumers must gate future information before displaying it. Current HUD call/view gating avoids release-time result text, but its telemetry and fixed call threshold remain incorrect as described above.
- No new outcome rolls, RNG draws/purpose changes, unordered collection iteration, agent-to-world dependency, feedback from animation into physics, or non-`bb_math::m` production transcendental was found. The inherent `hypot` calls added here are test-only. Simulation outcomes/event sourcing are untouched; existing deterministic log and recording-independence checks pass. These checks establish same-build tested determinism, not cross-platform identity.

## Resolution

Open.
