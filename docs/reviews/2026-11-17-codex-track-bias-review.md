# Independent review: persistent tracking error

Reviewed `f9e43ed..e9ff96c` on `work/track-bias`, after reading `CLAUDE.md` and agent-models §5.1. No P1 found. The principal fix works, including the owner's saved play, but one P2 filter-correctness issue remains. There is also a P3 correction to the reproduction metadata.

## Findings

### P2-1 — Include the learned bias's dependence on the ball state in the measurement Jacobian

**Locations:** `sim/crates/bb_game/src/live.rs:594`, `:597`, and `:2072`–`:2082`; the EKF claim in `docs/agent-models.md:422`.

`hp` depends on position, and `hv` depends on both position (the sight frame) and velocity (speed). Nevertheless, `see` always uses an identity derivative on the observed ball block, plus the two bias columns. Once the estimated bias is nonzero, this is not the Jacobian of the measurement mean the world generates. Choosing the prediction rather than the noisy look as the expansion point is sensible, but does not supply these missing derivatives.

There is an exact, simple counterexample away from the distance clamp and frame singularity. For an average outfielder on a later look, with only the along-sight position bias nonzero,

```
H_b0(r) = 3 × 0.6 × (3.5 / 30) × (r - eye) = 0.21 (r - eye)
h(r, b0) = r + 0.21 (r - eye) b0
∂h/∂r = (1 + 0.21 b0) I
```

For a learned `b0 = -2`, the derivative is **0.58 I**, whereas the update uses **I**. The same issue affects the across-sight term and the velocity measurement's geometry and speed derivatives. It changes both the gain and the reported covariance; it is not a reason to make the normalized bias states drift.

**Evidence:** an isolated 10,000-draw measurement probe used predicted position `(0, 10, 3)` m, eye `(0, 0, 1.7)` m, position prior `0.25 I` with true-position draws at σ 0.5 m per axis (four-sigma-truncated normals), known bias `b0 = -2` (zero bias uncertainty), and the specified fresh measurement noise drawn about the true geometry. There was no flight model or outcome decision. Mean position error²/reported variance was **2.406 lateral, 1.282 depth, 2.337 in the other sight-frame direction**. The identical zero-bias control gave **1.044 / 0.975 / 0.995**. At the old bias fraction 0.2, the negative-bias lateral result was 1.352: the refit magnifies the omission. Known bias is a valid limiting case of learning these states, and isolates the new measurement term from the pre-existing approximate flight transition.

**Failure scenario:** a fielder who has learned a substantial negative range bias still becomes too certain about his corrected ball position. Averaging across positive and negative biases on one routine fly can conceal this conditional miscalibration. This does not negate the large improvement from the old filter, reproduced below.

Use the full Jacobian of `x_block + H_b(r, v) b` at the chosen prediction, including the cross-block derivatives for velocity looks, with a consistent sequential-update linearisation. Add independent coverage checks conditioned on learned bias, as well as varied flight geometry. Recheck the fit after that correction rather than compensating through another covariance multiplier.

### P3-1 — Correct the diagnostic seed ranges and saved-play coordinates

**Locations:** `docs/metrics-ledger.md:169` and `:173`; `docs/calibration/2026-11-17-track-bias.md:56`.

The ledger identifies the second standard set as literal seeds **1000–1299**. However, `sim/crates/bb_cli/src/game_lab.rs:109` maps CLI seed arguments through `seed * 1_000_003 + i`. The prescribed `games --n 300 --seed 1000` therefore runs **1,000,003,000–1,000,003,299**. A reviewer using `Game::new(1000 + i)` or `fly_misses 1000 300` would examine a different population. The candidate's prescribed CLI command reproduced the bracketed numbers, so this is a labeling error, not evidence that those measurements were fabricated.

The saved-play note also says its ball lands near **(-12, 119)**. Fresh before/after builds of `read_track 40580 15 8` both report **(-13.2, 105.0)** at **6.31 s**. The baseline `fly_misses` reproduces the stated launch angle, hang time, 50 ft required coverage, 5.4 / 2.5 / 2.3 m read errors, and 1.6 m closest approach. Thus this is the intended play, but the recorded coordinate is wrong. Correct these reproduction identifiers while retaining the supported before/after conclusions.

## Verification

All builds, tests, and simulations ran sequentially with `CARGO_BUILD_JOBS=2`, under `taskset -c 0-7 nice`, with `$HOME/.cargo/bin` on `PATH`. Cargo output stayed within this worktree's `sim/target`; the baseline binaries used its `track-review-before` subdirectory to avoid shared-target cache ambiguity. Zig caches were redirected to `/tmp` after the initial default-cache write was rejected by the filesystem sandbox. No administrator privileges were needed.

The existing main checkout was verified at `f9e43ed`, with no sim-source differences, and used read-only for the baseline build; no scratch worktree was necessary. Probe source copies and logs are under `/tmp/track-bias-review*` and `/tmp/track-review-*`. No repository source, test, spec, TASKS, observations, or ledger file was edited. No commit was made.

### Tests and determinism

- `cargo test --release -p bb_game --lib tracking -- --nocapture`: all four selected tests passed. This includes the prediction-isolation check and both uncertainty checks.
- `cargo test --workspace --release --no-default-features`: **435 passed, 0 failed, 4 ignored** across the workspace. This includes the first-look grounder regression, ReadSkill direction test, communication isolation, FFI timeline sampling, and same-seed digest/log equality across threads (seeds 42 and 24583). Recording and diagnostic tracing also preserve the log/digest.
- Diff inspection found **no new RNG calls or changed draw order**: the four fixed perception draws and six per-look fresh draws retain their streams and ordering. No new direct transcendental, `mul_add`, or unordered collection iteration was added. No outcome roll or new hidden calibration multiplier was found.

### Filter algebra, uncertainty, and short flights

The four states are standardized fixed draws, so **constant bias dynamics are correct**. Range, speed, and changing sight directions belong in their measurement coefficients. The along/across position and velocity coefficients match the generator, including velocity's 1.5 depth stretch and the first look's shrink. A unit bias prior is appropriate for the standardized, four-sigma-truncated draws. The negative initial velocity/bias cross-covariance has the correct sign, and its matching contribution is included in the ball prior.

The dense left/right propagation implements `F P Fᵀ`, including ball/bias cross-covariances, and the integrated acceleration process-noise coefficients are correct. The update implements `P - P Hᵀ S⁻¹ H P` for its supplied, frozen-coefficient H. The remaining concern is P2-1, not a matrix indexing error. The ball covariance already includes bias uncertainty; removing the old extra term from landing doubt is appropriate.

Reusing the new independent lateral-coverage measurement against a temporary copy of the old tracker reproduced the diagnosis:

| Time before landing | Baseline RMS error / σ (m) | Baseline mean error²/variance | Candidate RMS error / σ (m) | Candidate mean error²/variance |
|---|---:|---:|---:|---:|
| 2 s | .921 / .178 | 26.65 | .508 / .529 | .92 |
| 1 s | .756 / .130 | 33.95 | .295 / .271 | 1.21 |
| .6 s | .417 / .103 | 16.39 | .137 / .128 | 1.16 |

The real-spin landing-doubt test also reproduced the note: RMS miss/doubt **.83/1.06 m** two seconds out and **.39/.41 m** one second out, over 70 balls.

A separate probe used **500 draw sets per geometry**, stationary CF and SS observers, grounders, 12° and 20° liners, a 3.62 s short fly, routine flies from either side of the landing point, and doubled-spin variants. At three epochs it checked all sight-frame position/velocity components and Cholesky-factored the full covariance. **All 18,000 sampled covariances were exactly symmetric and positive definite**; the smallest Cholesky pivot was about `4.7e-5`.

The claimed 0.9–1.2 is real for the documented routine-fly measurement, but is not universal. At roughly .04 s before the first bounce, lateral position error²/variance was **1.60** for the short fly at CF, **.79** for that fly at SS, **.90** for the 12° liner at SS, and **.10 / .31 / 1.03** for flat/down/up grounders at SS. The brief grounders were conservative after their first look, rather than overconfident. The SS liner's other across-sight velocity component was overconfident (**3.87**, .234 m/s RMS error versus .119 m/s σ); it was also 3.85 with the new filter at bias 0.2. These are bounds on the honesty claim, not proof of a new infield regression. This geometry probe does not establish coverage for every moving observer or every game trajectory.

### Calibration and changed tests

Fresh runs of `fly_misses 0 100 - <grade>` reproduced the final fit:

| ReadSkill | Routine catches | Rate |
|---|---:|---:|
| 20 | 535 / 585 | .915 |
| 50 | 563 / 582 | .967 |
| 80 | 563 / 572 | .984 |

These are within the named 90–99% overall routine-play band. `TRACK_BIAS_FRAC` is a noise scale, and `READ_RHO` is an ability spread on the two read sigmas: both are permitted calibration mechanisms. Grade 50 is unchanged by the rho adjustment. There is no catch-probability branch or run-total fudge. These checks support the stated fit, subject to correcting P2-1; they do not independently establish that 0.6 is a physiological measurement of human bias.

The harder ReadSkill test remains an independent direction check: selection uses flight geometry, not whether either grade catches the ball, and the assertion is better readers catching more, not matching fitted counts. Using real spins is useful. Moving away from a ceiling-saturated fixture is justified. Likewise, the FFI seed change preserves its substantive sampling assertions and exercises dives in game 6; the full suite passed. There is no evidence here that it conceals a timeline regression.

The initial anticipation jump is unchanged for preserved inputs: `jump_at_contact` uses separate pitch/swing/lead cues and anticipation attributes, before tracking starts. The larger early tracking error can change subsequent routes and erase some advantage on marginal plays; the improved routine catch results do not establish that every route improves.

### Liners, ledger, saved play, and performance

For a mechanism check independent of diverging game prefixes, a fixed catalogue of **369 fair 10–25° balls from games 0–35** was replayed with four draw sets per ball, bases empty and perfect hands. Both runs used the new filter; only the temporary copy's bias fraction changed. There were 1,424 eligible first-bounce trajectories with identical bin counts:

| Flight time | Trials | Catches at bias .2 | Catches at bias .6 |
|---|---:|---:|---:|
| 1–2 s | 272 | 61 | 61 |
| 2–3 s | 556 | 134 | 123 |
| 3 s or longer | 596 | 327 | 304 |

The larger persistent component really does cost liner catches, supporting the direction of the ledger attribution. The explanation should be qualified: these liners are not all airborne only 1–3 s, and the largest loss in this probe is on longer liners. The probe does not assign the entire aggregate BABIP movement to a single learning-timescale mechanism.

Standard CLI outputs reproduced:

| Metric | Baseline `--seed 0` | Candidate `--seed 0` | Candidate `--seed 1000` |
|---|---:|---:|---:|
| Runs/team-game | 4.13 | 4.06 | 4.19 |
| GB hit rate | .310 | .309 | .313 |
| LD hit rate | .668 | .681 | .681 |
| FB hit rate | .127 | .113 | .098 |
| PU hit rate | .010 | .007 | .002 |
| Rules warnings | none | none | none |

Other printed K/BB/HR and PA/pitch figures also match the note. These hit rates are the CLI's provisional in-play classification, not official scorer BABIP.

Fresh baseline `fly_misses 40580 1 list` reproduced the saved miss; the candidate caught all **7/7** eligible outfield flies. The before/after read traces agree on the ball's displayed flight and landing, with the candidate's late lateral miss much smaller. See P3-1 for the coordinate correction.

Timing the requested `sim-cli games --n 300 --seed 0`, excluding compilation, gave **80.7 s before and 82.7 s after**, approximately **+2.5% wall time**. Both were release builds without the optional Rerun feature, under the same affinity and niceness. This single before/after pass includes differing play durations and counts, and is not an isolated matrix microbenchmark; it shows no large end-to-end performance regression.

Not rerun: baseline `--seed 1000`, the 300-game `fly_misses` population, the dive-population probe, or the note's full 300-game bias-.2 counterfactual. Their exact reported counts remain author measurements; the reproduced standard outputs, 100-game grade fits, saved play, and matched liner probe are the independent checks here.

## Resolution

Both findings are fixed on `work/track-bias`. The code is in `dead272`, and the documents and measurements are in the commit that adds this section.

**P2-1: fixed.** A look's expected value is now one function of the ball's state and the persistent error, `Look::expect(r, v, b)`: the ball's position or velocity, moved by b at the σ and in the sight directions of a look from his eyes at r (with |v| for velocity), together with the fresh covariance there.

- `Track::see` evaluates it at the estimate as it stands. A velocity look therefore uses the position the position look has just corrected, which makes the sequential linearisation consistent.
- H is the full Jacobian in r, v and b, by central differences. It includes the velocity look's dependence on position through the line of sight, and on speed.
- No covariance multiplier was added.

**New independent test:** `a_look_with_a_learned_persistent_error_reports_a_covariance_that_covers_its_error`, built like the review's probe.
- The ball's position is known to 0.5 m and its velocity to 1 m/s; b is known exactly and not zero.
- A position look and then a velocity look are drawn as the world draws them, about the true ball.
- It covers three geometries (near and low; far and high; a liner to his side) and three biases (b0 = −2; b1 = +2 with b3 = −2; all four at ±2).

Over 2,000 draws each, the mean error²/variance along, across and up his line of sight, for position and velocity, is 0.92–1.11 in all 54 cells; the test requires 0.85–1.15. With the identity Jacobian put back temporarily, case 0 with b0 = −2 gives 1.39 / 2.36 / 2.38 and the test fails, which reproduces the review's 2.4.

**In flight:** `the_tracking_read_reports_a_covariance_that_covers_its_error` gives 0.83 / 1.14 / 1.36 at 2 / 1 / 0.6 s. The real-spin doubt test has RMS doubt 1.05 / 0.41 m against an RMS miss of 0.77 / 0.32 m (75 balls).

**The fit was rechecked, not refitted** (it stays inside the band). Grades 20 / 50 / 80 on seeds 0–99 are .934 / .979 / .986 (were .915 / .967 / .984). `fly_misses 0 300` gives .980 (was .976; main .967), with 34 misses (14 "dive range, no dive"). Play 15 is still caught.

**Standard run on the final build**, `--seed 0` [`--seed 1000`]:
- runs 4.14 [4.16] (reviewed build 4.06 [4.19]; main 4.13 [4.24]);
- hit rates GB / LD / FB / PU .308 / .683 / .109 / .006 [.308 / .682 / .098 / .004];
- dives 2.3 a game, 45% caught.

The ledger row, the note (§2a, §3–§6) and agent-models §5.1 and §10 are updated.

**Not addressed** (outside P2-1, recorded in the note §2a and §6): the review's geometry probe found the shortstop's liner velocity component at 3.87, the same at bias 0.2, and very short grounders conservative. Coverage is shown for routine flies and for one look in isolation, not for every trajectory or moving observer.

**P3-1: fixed.**
- The ledger heading and row, and the note's §5, now say that `--seed 1000` runs game seeds 1,000,003,000–1,000,003,299, and that `dive_probe 150 1000` takes game seeds 1000–1149 directly. The older ledger rows keep the file's stated label convention, which its "Standard run" paragraph explains.
- The saved play's landing point is now (−13.2, 105.0) m at 6.31 s, from `read_track 40580 15 8`. The (−12, 119) was the call's spot from the communication log.
- The liner attribution is qualified with the review's matched-probe table (1–2 s 61 = 61, 2–3 s 134 → 123, ≥ 3 s 327 → 304). The loss is mostly on liners up 3 s or longer, where the now-worse early read sets a long route. The first version's "too short to correct" is withdrawn as the explanation, and the aggregate is not assigned to one mechanism.

**Full workspace tests** (`cargo test --workspace --release --no-fail-fast`): 436 passed, 0 failed, 4 ignored. The ReadSkill direction test has grade 20 catching 279 of 308 and grade 80 292.
