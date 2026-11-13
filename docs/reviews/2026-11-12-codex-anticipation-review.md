# Anticipation review

Reviewed `74679a6..cdd473c` against `CLAUDE.md`, `docs/agent-models.md` §§5.1–5.3, 5.10 and 10, and the RNG registry. Line references refer to `cdd473c`. The checkout is `61ff69a`, whose only subsequent change is `TASKS.md`; the reviewed code is identical. No source, test, spec, or TASKS file was edited.

No P1 finding was established. The published run-environment numbers reproduce. Two P2 defects and two P3 boundary/contract issues remain.

## Findings

### 1. P2 — The pre-reaction “swing” observation includes the collision result

**Location:** `sim/crates/bb_game/src/live.rs:465`; supplied by `sim/crates/bb_game/src/game.rs:264`; sensor described in `docs/agent-models.md:457`.

`seen_swing` is centered on the true outgoing ball velocity, rather than any property of the visible bat. It therefore includes effects of the incoming ball and collision that are not determined by the bat's angle. Adding independent Gaussian error does not make that observation a measurement of the bat. As `swing_sigma` approaches zero, it becomes a perfect read of exit spray, rather than a perfect read of swing geometry. This privileged information drives movement from contact, before the §5.1 ball-read reaction gate.

This is **not an outcome roll**: randomness still perturbs a perception input and the world resolves outcomes physically. Nor was a forbidden crate dependency introduced. A noisy observation of launch velocity is legitimate physical perception once the ball is being read, as in §5.1. The defect is using it earlier under the specified *swing* sensor contract; the crate-graph test cannot detect that distinction in orchestration code.

**Failing scenario and verification:** A `/tmp` probe called production `resolve_contact` twice with identical bat kinematics, ball position, incoming velocity `(0,-40,0)` m/s, and contact normal. Changing only ball spin about Z from −200 to +200 rad/s changed outgoing spray from **+4.071844° to −3.239788°**. The bat observation is identical, but this code's observation mean changes by 7.31°. Actual seed-5 records also show why bat angle and exit spray cannot simply be equated: at play 82 the bat's horizontal normal is **−0.112687°**, while exit spray is **+29.812506°**; at play 224 they are **−12.608575°** and **+43.794041°**. These normal angles are diagnostics, not a proposed complete swing-to-spray predictor.

The minimal honest alternative is to sample visible bat geometry from `rec.swing` at **actual contact time**, then pass a noisy bat-yaw/axis observation into anticipation. Use handedness and a learned mapping, with residual collision uncertainty, to turn that cue into a spray belief. Keep the executed swing itself on the world/orchestrator side of the observation boundary. If no cue is available, use the pitch prior alone. Alternatively, explicitly specify and authorize an early *ball-direction* sensor with its own latency; it should not be presented as reading the bat. Re-run calibration after changing the cue.

### 2. P2 — First assignment omits the jump remaining before a fielder reacts

**Location:** `sim/crates/bb_game/src/live.rs:790`, `sim/crates/bb_game/src/live.rs:794`; same assumption in fallback ETA at `:778` and call slack at `:941`.

The first assignment occurs at the **minimum** defensive reaction time. For a slower-reacting fielder, `start` is moved forward to his own reaction time, but `run_gap` still receives his position and velocity **now**. He continues accelerating toward `jump` during the intervening interval. Previously, freezing his state over that interval was correct because he stood still. Now the planner omits both displacement and velocity change, potentially changing standing/dive feasibility and calls. Later plans after his reaction time use the current state correctly.

**Failing scenario and verification:** The private-module probe used production initialization, movement, and intercept helpers: seed 0, play 3, fielder IDs `0x0181..0x0189`, normal positions, grade-80 Anticipation, pitch X = 0, RHB, contact at 0.4 s, launch velocity `(10,30,30)` m/s. At first assignment (0.65 s), CF is at X **0.253181 m**, moving at **1.827274 m/s**. At his reaction end (0.75 s), production stepping puts him at X **0.472067 m**, moving at **2.438576 m/s**. The first plan omits **0.218886 m** and **0.611302 m/s**.

A controlled predicted rolling path makes the feasibility difference explicit: from `(14.153181,93.536,0)` at 2.75 s, velocity `(0,20,0)` m/s, deceleration 0.001 m/s², ending at 3.15 s. The unmodified scan finds **no standing intercept**, only a dive at 2.92 s. Supplying the state propagated to reaction end, with the same prediction and planning time, finds a **standing intercept at 2.96 s**. This isolates the planner assumption; it is not a claim that this synthetic path was observed in the 2,048-game sample.

Project each fielder through his remaining jump interval before computing post-reaction reach, fallback ETA, and call slack, or defer his assignment until his reaction gate opens. Preserve the velocity as well as the position.

### 3. P3 — Linear angle fusion has an arbitrary discontinuity on backward fouls

**Location:** `sim/crates/bb_game/src/live.rs:468`, `sim/crates/bb_game/src/live.rs:471`.

The posterior uses ordinary subtraction/averaging of `atan2` angles. Almost identical directions on opposite sides of ±π can therefore produce very different estimates and commitments. This is separate from the documented hard jump-zone boundary. Fair sprays around ±45° do not cross the `atan2` seam, but backward fouls do enter the live simulation.

**Failing scenario and verification:** With seed 1, play 3, third baseman ID `0x0185`, RHB, pitch X = 0, zero pitch-read error, and grade-20 swing sigma, change true spray from **+179.999° to −179.999°**, preserving the same RNG keys. The swing-noise draw is **1.513536** in both cases. The posterior changes from **+77.694192° to −52.111469°**. At his −36° home angle, 3B changes from no jump to target velocity **`(-2.926657,-2.126341,0)` m/s**, despite only a 0.002° physical direction change. The production-helper probe reproduced this directly.

Use circular/unwrap-aware fusion and shortest-angle offsets, with an explicit policy for highly conflicting/antipodal cues, or exclude back-half contacts from this forward-field approximation. Merely adjusting the ±45° foul-line thresholds does not address this seam.

### 4. P3 — The angular zone does not guarantee protection of cover jobs

**Location:** `docs/agent-models.md:458`; implementation at `sim/crates/bb_game/src/live.rs:473` and `sim/crates/bb_game/src/live.rs:548`.

The spec says the gate prevents pulling a man off a cover job, but it tests only angular proximity. Multiple fielders' zones overlap, cover roles are assigned later, and the pre-reaction movement branch ignores an assigned cover target. A non-chaser can lean away from his bag and continue accelerating that way after receiving a cover role. This can be an intentional cost of anticipation, but the claimed guarantee is false.

**Failing scenario and verification:** Production-helper probe: seed 2, play 3, the IDs/positions above, RHB, pitch X = 0, grade-80 Anticipation, ball velocity `(15,26,30)` m/s, contact at 0.4 s. At assignment (0.65 s), RF is primary and 1B is `Cover(1)`, but 1B has already displaced **`(-0.263204,+0.205638,0)` m**, with velocity **`(-1.869162,+1.460349,0)` m/s**. His displacement projects **−0.191527 m toward first base**: he moved away from it. Giving 1B a 0.35 s reaction time, with everyone else's defaults, also shows an existing cover role being ignored: the next production step adds **`(-0.032856,+0.025670,0)` m**, further away, while his target is the bag.

Either make known cover responsibilities constrain commitment/movement, or document this tradeoff and remove the guarantee. An angular zone alone cannot establish tactical availability.

## Other checks

- **Signs and frame:** Correct for the current model. RHB inside is negative X, pull is negative spray, and `atan2(vx,vy)` increases toward RF. The derivative of the radial direction `(sin φ,cos φ)` is `(cos φ,−sin φ)`, so the across-line vector has the correct sign. The 96-game location example gives mean spray **−14.3°, −11.9°, −7.4°, −4.5°, −3.0°, +0.5°, +2.1°** from the most inside to most outside RHB bin. This supports the table's direction; its exact fitted means need not recur after anticipation changes subsequent game histories.
- **Independent expectations and left-handed mirror:** The unit test's hard-coded signs agree with the published home angles, but its projection basis repeats the implementation formula. Its near-perfect swing read also suppresses the handedness-dependent prior; the blind RHB test uses X = 0. Thus it does not meaningfully test the nonzero-location handedness branch. An additional `/tmp` probe disabled pitch noise and made the swing read uninformative: RHB pitches at X = ±0.2 and mirrored LHB pitches gave mirrored SS/2B velocities within **0.000001 m/s**. No sign defect was found.
- **Fusion and commitment:** For the stated fixed 22° Gaussian prior, `k = sp²/(sp²+ss²)` and `σ_post = sp·ss/√(sp²+ss²)` are correct. This is a plug-in prior based on noisy pitch location, not full propagation of uncertainty through the nonlinear location table. The commitment formula implements §5.10. The hard 20°/25° zone cutoff can abruptly change a full jump to zero, including for a fielder near a foul line, but that discontinuity is explicitly specified. Finding 3 concerns an additional, unintended angular seam.
- **Pre-reaction movement:** The lack of braking/turn caps in this branch does not itself break §5.3 for reachable initialization: every fielder starts at rest, `jump` is constant, and pre-reaction velocity remains collinear with it. Only acceleration toward a fixed speed is needed. Once reacting, the ordinary braking/turning branch applies to the carried velocity. The Euler speed-law approximation is consistent with the existing movement integration. The remaining concerns are findings 2 and 4.
- **Determinism:** Purpose 11 is appended without renumbering existing values. Pitch and swing use distinct sub-indices 0 and 1, each keyed by the fielder ID; no shared stream's draw order changes. No new hash iteration was introduced. Runtime `atan2`, `cos`, `sin`, square root, and grade-mapping power use `bb_math::m`. `to_radians` is a constant multiplication and `signum` a sign operation; neither requires a transcendental wrapper. Direct trigonometric calls in the new unit test do not enter simulation execution. The data-format registry row mentions only sub-index 0; §5.10 and the implementation also specify/use 1 for the swing.
- **Lab/API mapping:** `Game::with_fielders` applies the supplied array to both teams; `simulate_game_fielders` uses that configuration throughout. CLI flags construct the same average array, then apply the requested grade and/or set `jump_gain = 0`. With both flags, no-jump takes precedence. Measured mappings match §10: grades 20/50/80 give pitch sigmas **0.29296875/0.15/0.0768 m** and swing sigmas **29.296875/15/7.68°**.
- **Bat-knob bound:** No defect established in the widened assertion at `sim/crates/bb_ffi/src/lib.rs:374`. Seed 5, play 249 starts its swing at 0.236632 s with knob **`(-1.614418,0.099701,1.262156)` m** and pivot **`(-0.95,0.30,1.101714)` m**. Their distance is **0.712259 m**, matching executed `r_knob` (the template allows 0.70 m before reach noise). Plate-centered X was the wrong arm-reach bound: 0.95 m pivot offset plus roughly 0.70 m reach already exceeds 1.6 m. A 1.75 m X allowance is justified for this case; a pivot-relative check would express the intended geometry better. The FFI frame test passes.

## Verification

Read `CLAUDE.md` first, the requested spec sections, and the complete commit diff. All probe code and probe outputs were placed under `/tmp`.

The initial `cd sim && cargo test -p bb_game` failed at linking because Zig tried to write its cache outside the writable sandbox. Subsequent build commands used `ZIG_GLOBAL_CACHE_DIR=/tmp/anticipation-zig-cache` and `ZIG_LOCAL_CACHE_DIR=/tmp/anticipation-zig-local`.

Commands run from `sim`:

| Command | Result |
| :--- | :--- |
| `cargo test -p bb_game` with the cache overrides | **52 passed**, 0 failed: 32 unit tests and 20 integration tests; doc tests 0. Includes both new anticipation tests, crate isolation, recording equivalence, and same-seed/thread determinism. |
| `cargo test -p bb_ffi timelines_sample_to_plausible_frames` | **1 passed**, 0 failed; 1 unrelated test filtered out. |
| `cargo run --release -p bb_game --example spray_by_location -- 96` | Completed; 5,386 fair batted balls, bin means reported above. |
| `cargo run --release -p bb_cli -- games --n 2048 --seed 5 --no-jump` | Completed; no rules warnings. |
| `cargo run --release -p bb_cli -- games --n 2048 --seed 5 --anticipation 20` | Completed; no rules warnings. |
| `cargo run --release -p bb_cli -- games --n 2048 --seed 5` | Completed at default grade 50; no rules warnings. |
| `cargo run --release -p bb_cli -- games --n 2048 --seed 5 --anticipation 80` | Completed; no rules warnings. |

Reproduced calibration (CLI seed 5 expands to game seeds `5,000,015..5,002,062`):

| Mode | Runs/team-game | GB hit | LD hit | FB hit | PU hit |
| :--- | ---: | ---: | ---: | ---: | ---: |
| No jump | 5.35 | .408 | .722 | .120 | .005 |
| Grade 20 | 5.31 | .406 | .719 | .119 | .004 |
| Grade 50 | 5.22 | .399 | .721 | .116 | .005 |
| Grade 80 | 5.03 | .384 | .715 | .115 | .003 |

The published figures reproduce to every printed digit. Most of the hit-rate change is in ground balls, which account for about 53% of batted balls. Full commitment produces roughly 0.3–0.5 m of displacement and 2.4 m/s of velocity by reaction end, enough to change marginal ground-ball plays. The scale is mechanically plausible **for the implemented sensor**. It does not validate the swing-cue interpretation in finding 1. These are whole-game comparisons with diverging subsequent pitches and opportunities, rather than a fixed corpus of identical batted balls.

Additional commands: `python3 /tmp/build_anticipation_probe.py` (with the same cache overrides), `/tmp/anticipation_probe`, `/tmp/anticipation_probe --planner`, and `/tmp/anticipation_probe --cover`. The builder copied the unchanged production `live.rs` into `/tmp`, appended access to its private helpers, and linked a standalone Rust executable against the built workspace libraries. The probes called production movement, initialization, assignment, interception, RNG, and contact resolution; the counterfactual planner comparison changed only the supplied fielder state. Outputs are `/tmp/anticipation-probe.log`, `/tmp/anticipation-planner-probe.log`, and `/tmp/anticipation-cover-probe.log`. Findings above provide the inputs and observed failures.

## Resolution

Open.
