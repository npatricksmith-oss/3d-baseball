# Plate-cue review — 2026-11-13

Reviewed `5da8633..2caaa70686af0384dd777a6c458a7d06eb55cb65`, with `CLAUDE.md` read first. No P1 finding. Two P2 findings concern the uncertainty math; one P3 concerns the calibration diagnostic.

## P2 — Carry the position/velocity covariance into the call's uncertainty

**Location:** `sim/crates/bb_game/src/live.rs:561–565`, with the shared velocity introduced at `live.rs:593–604`.

The final `hypot` treats the fused position error and the future velocity error as independent. They are correlated: the plate position is calculated from the same `ve` subsequently flown forward. A depth velocity overestimate moves both the inferred current position and its future position in the same direction. The position posterior's variance already includes the first contribution, but adding variances alone omits their cross term.

An independent scalar case using the implementation's own linear error model makes this explicit. Set `dist=90`, `sigma0=3.5`, `tau_read=0.6`, `view=0.65`, `speed=30`, `s=1`, and `t_left=2`, with a horizontal unit LOS. The depth gain is 0.992074 and the current posterior sigma is 1.942946 m. The function returns **3.161620 m**. Including the shared velocity covariance gives **4.020862 m**, even retaining precisely the same spin floor and linear propagation approximation. In this scalar model, the missing variance is `2 * t_left * gain * s * sigma_v_depth²`. In the plate-dominated limit, the velocity contribution grows like `(s + t_left) * sigma_v_depth`, rather than the quadrature of the two time intervals.

A probe of the actual private planner, using an unmodified copy of `live.rs`, also finds a call boundary affected by this omission. Use the call-test contact point `(0, 0.5, 1)`, launch velocity `(7, 30, 24)`, launch spin `believed_spin(v, Some(0))`, average attributes, zero read errors, and no anticipation jump. At 0.7 s after contact, CF at his home position has a believed intercept at absolute time 5.85 s and 1.139508 s to spare. `call_value` returns tier 3: its required margin is 1.105997 s. Adding just the position/velocity cross term to the existing propagation approximation raises that margin to **1.230701 s**, leaving him tier 1. This is a comparison within the call's existing approximation, not a claim that linear propagation is an exact flight covariance.

**Suggested fix:** retain the fused state's position/velocity cross covariance and propagate the joint uncertainty to the intercept. At minimum, include the cross term consistently with the existing linear model; a Jacobian of the observer's backward/forward prediction can also account for drag and Magnus. Add an analytic constant-acceleration regression with known correlated errors and a call-tier boundary regression. Update §5.7's description when correcting the formula.

## P2 — Build the fusion covariance from the noise that is actually sampled

**Location:** `sim/crates/bb_game/src/live.rs:522–526`; compare the samples at `live.rs:591–593`.

`read_cov` describes independent lateral and height errors. The sampler uses the **same** `err[1]` for lateral position and height, and the same `err[3]` for lateral velocity and vertical velocity. Consequently, both input covariances have a lateral/height cross term that the new fusion discards. This makes the claimed precision-weighted 3D update use different covariances from its observations. The sign and order of the Kalman update itself are correct; its inputs are not.

For `los=X`, `sigma=1`, `depth=3`, enumerating four equally weighted latent draws `(±1, ±1)` gives the position covariance

```text
[9  0    0   ]
[0  1    0.5 ]
[0  0.5  0.25]
```

whereas `read_cov` returns `diag(9, 1, 0.25)`. This is exact enumeration of the existing sampler, not a statistical tolerance issue. For `sp=sv=s=1`, using the existing plate covariance approximation and spin floor, the correct lateral/height gain block is `[[0.210526, 0.105263], [0.105263, 0.052632]]`; the implementation uses `diag(0.235294, 0.090909)`. Thus a plate-minus-direct residual of one metre laterally and zero vertically should also adjust height by 0.105263 m under these covariances, but the implementation adjusts height by zero. Height errors affect the predicted landing time and point.

**Suggested fix:** preserve the existing draws and construct this part of the covariance as `outer(lat + 0.5*z, lat + 0.5*z) * sigma²`, alongside the depth term. Use it for both position and velocity, with appropriate propagation into the plate estimate. Add an exact latent-draw covariance test and a small analytic fusion case with off-diagonal terms. If independent height noise is intended instead, changing the sampler requires an explicit replay/draw-order decision.

## P3 — Stop the plate-cue diagnostic at the end of the first trajectory

**Location:** `sim/crates/bb_game/examples/plate_cue.rs:29–39`; affected claim at `docs/calibration/2026-11-13-plate-cue.md:46`.

The sampling bound considers only ground and wall touches. A ball clearing the fence has neither, so the example continues sampling beyond `tr.last().t`. `BallPath::at` clamps to the terminal position and velocity, but `position_from_plate` still flies that velocity backward by the full requested `s`. These are no longer first-flight samples at the requested time, and the resulting error is incorrectly attributed to spin.

Reproduction: `cargo run --release -p bb_game --example plate_cue -- 16 true` prints **0.00 / 34.95 m** for 10–25° at 4 s. Seed 2, play 52 requests absolute time 4.447891 s, although the flight ended over the fence at 3.521482 s; its true-spin miss is **72.429484 m**. There are 19 such bad four-second samples in those 16 games. A temporary probe adding `tr.last().t` to the bound removes them: that cell becomes **0.00 / 0.00 (54)** instead of **0.00 / 34.95 (73)**. On 48 games, the typical-spin four-second liner median becomes **23.27 m**, rather than **26.01 m**. The note's displayed 25–40° and 55–90° table cells still reproduce.

**Suggested fix:** cap `t_end` at `tr.last().t` and regenerate affected diagnostic claims, including the 18–26 m liner statement. Add a terminal over-fence case to ensure a clamped path cannot enter the table as a later flight sample. This finding concerns the example, not backward integration.

## Verification

- Ran `cd sim && cargo test --workspace --release`: **145 passed, 0 failed**, including the inversion test, call tests, crate-graph check, and replay/thread determinism checks.
- Ran the requested `cargo run --release -p bb_cli -- games --n 1024 --seed 5` and `--seed 2` on HEAD. Also exported `5da8633` to `/tmp` and ran the same baseline batches with `--no-default-features` (the optional Rerun dependency disabled). Every §5 before/after number and the corresponding TASKS “Resume here” figures reproduced:

  | Seed / version | Runs/team-game | GB hit | LD hit | FB hit | PU hit |
  |---|---:|---:|---:|---:|---:|
  | 5 / baseline | 5.44 | .411 | .724 | .120 | .008 |
  | 5 / HEAD | 5.35 | .413 | .707 | .123 | .008 |
  | 2 / baseline | 5.26 | .406 | .715 | .119 | .008 |
  | 2 / HEAD | 5.23 | .409 | .694 | .131 | .008 |

  All four batches reported no rules warnings. These runs verify the reported distributions, not the correctness of the uncertainty model. Historical intermediate knob sweeps, serial timing comparisons, and the separate 512-game hit-length table were not rerun.
- Ran `read_error -- 48`: every displayed “after” cell in the note's §3 table matched, including first-look totals 8.6, 7.4, and 7.0 m. Ran `plate_cue -- 48`: the note's displayed typical-spin medians matched. Ran `plate_cue -- 16 true`, then temporary bounded variants and the concrete terminal-sample probe described above.
- Probed backward integration against independent fine forward steps with wind `(7, -3, 2)` m/s and spin decay enabled, at times from 1 microsecond to 3 s. Maximum position discrepancy was **0.000002408 m**. Negative RK4 steps correctly reverse spin decay as well as velocity evolution. The committed test already checks three launch shapes, several times, and the different forward/backward step sizes; the requested inversion unit test is present.
- Exercised the actual first-flight predicate in a temporary copy of the module: initial flight and an airborne foul retain the cue; the first bounce, exact wall-contact time, a fresh glove deflection, a held ball, and a throw disable it. `set_free` starts a new trajectory at its later time, so a dropped throw cannot regain the contact cue. A foul still in uninterrupted flight correctly retains knowledge of its origin. No new unintended access to true spin or hidden launch velocity was found. Exact contact position/time and the shared spatially uniform environment are the explicit modeling idealizations documented in §5.1 and the note's open items; the environment was already used for forward predictions.
- Checked the gain's residual sign, matrix order, coordinate frame, and posterior formula. They are correct for the covariances supplied and independent measurement errors. For a nonzero horizontal LOS, positive read scales, and positive `s`, the isotropic spin term makes the covariance sum invertible. Tiny `s`, zero horizontal LOS, and the singular zero-time fallback remained finite in probes. LOS is always constructed horizontally; a ball directly overhead produces a zero LOS, not a vertical unit vector. That case already collapses horizontal noise in the old sampler and still yields zero projected depth sigma; this change does not repair that existing degeneracy.
- `ti - self.t` is the remaining time from the believed state used to start prediction, and `bv` comes from that believed path at `self.t`. The old `read_depth_sigma` has no remaining callers. Existing tests exercise an infielder calling a short pop-up, an unsure outfielder leaving it to him, and an outfielder calling a fly near his own position. The re-chosen gap-fly test still asserts fixed behavioral expectations: two initial chasers, targets near the physical landing point, and a later CF call with RF peeling off. It does not calculate expected tiers from a duplicate formula, but also does not validate either covariance issue above. Dedicated covariance and lifetime-boundary regressions would add coverage those behavioral fixtures lack.
- Inspected added simulation math and diagnostics: transcendental operations use `bb_math::m`; no new RNG stream, unordered-map iteration, `mul_add`, or nondeterministic math was introduced. §5.1/§5.7 and the note otherwise describe the implemented cue, refit, time scaling, and call rule. Their precision-weighting/self-knowledge claims need the math corrections above.

The initial new CLI/example links were blocked by Zig attempting to write its cache outside the sandbox. Redirecting `ZIG_GLOBAL_CACHE_DIR` to `/tmp` resolved this without privileged operations. Temporary probe sources, executables, and the baseline export were removed. No source, test, spec, or TASKS file was changed.

## Resolution

All three findings fixed (2026-11-13).

- **P2, position/velocity covariance in the call's doubt: fixed, more broadly than suggested.** Adding the cross term to the linear model was tried first; against 600 reads of a gap fly drawn as the simulator draws them and flown forward, the linear model (with or without the cross term) was 1.1–1.4× the real spread (part of that the spin floor, which the sample does not contain), and it ignores drag. `landing_depth_sigma` now runs the fielder's own read (`read_prediction`) from his belief, once as it is and once with each of the four error draws at one σ, and adds up in quadrature how far each moves his landing point along his line of sight, plus the spin floor times the plate's depth gain. That carries the plate/velocity coupling, drag and the coupled lateral/height draws in one step. New test `a_fielders_doubt_matches_the_spread_of_his_reads` (within 15% of the sampled spread plus the spin term at 0.7, 1.5 and 2.5 s). The gap-fly call test was re-chosen again: with the smaller, more accurate doubt the centre fielder was sure of the first replacement ball at first look (now a ball landing at (27.5, 104.5); the right fielder clears the chase threshold by 0.4 s). Cost: +6% game time.
- **P2, covariance from the sampled noise: fixed.** `read_cov` now builds the across-the-line-of-sight part as one draw along `lat + 0.5 ẑ`, as the sampler draws it, for both position and velocity. The sampler is unchanged (no replay decision needed). New test `the_read_covariance_is_the_sampled_one` (exact enumeration of the ±1 draws).
- **P3, plate-cue diagnostic past the fence: fixed.** `t_end` is capped at `tr.last().t`. `plate_cue -- 16 true` now prints 0.00 / 0.00 in every cell; the liner figure in the note and spec is now 18–23 m.

After the fixes, `sim-cli games --n 1024`: seed 5 5.36 runs, LD .718, FB .128, PU .008; seed 2 5.21, LD .710, FB .131, PU .009. Close to the reviewed numbers; not refitted. `cargo test --workspace --release`: 147 passed.
