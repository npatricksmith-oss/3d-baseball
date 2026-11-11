# Fielding and diagnosis review — 2026-11-11

Reviewed **`c31b227..411af1a`**, the four commits named in the request, after reading `CLAUDE.md`, the applicable specs, both diagnoses, and the day-fixes review's Resolution. Source locations below refer to `411af1a`. During this review, **`72dc6a1`** appeared in the shared checkout; it changes documentation only, records owner acceptance of the outfielder defaults, and partially addresses the spec discussion below. It is outside the requested range.

The aggregate claims reproduce, and the new regression tests detect their principal intended regressions. There are still movement-state and intercept-planning inconsistencies. No new P1 finding was established; the earlier review's pending-home-run issue remains deferred, not repaired.

## Findings

### 1. [P2] Left field does not receive the outfielder defaults being calibrated

**Location:** `sim/crates/bb_game/src/play.rs:47–61`; `game.rs:156`; `live.rs:1180` and `:579` (position indexing).

Every discovered constructor call supplies a zero-based index through `array::from_fn`, but `average` uses `pos >= 7`. Left field is index **6**, so only center and right receive the outfielder values. This indexing defect predates the range, but it directly invalidates the new change's interpretation as an outfield-wide calibration and increases the perception disparity.

**Executed scenario:** Reading the attributes in the live fixture gives LF **7.9 m/s, τ = 0.7 s, reaction 0.25 s, read σ = 0.3 m**; CF/RF get **8.0 m/s, τ = 1.0 s, reaction 0.40 s, read σ = 3.0 m**. LF also retains the infielder arm and transfer defaults. The reported LD/FB rates therefore fit a defense with a tenfold LF-versus-CF/RF read-noise difference, rather than three identically calibrated outfielders.

**Suggested fix:** Define the argument explicitly as a zero-based index and use `>= 6`, or use a position enum. Check all nine positions, then repeat the calibration before retaining these fitted values and aggregate claims.

### 2. [P2] A chaser can stop physically while retaining and increasing full running speed

**Location:** `sim/crates/bb_game/src/live.rs:238–245`, consumed by `:445`.

The position clamps to the target, but the returned speed does not account for that clamp. For a finite intercept time at or behind the current time, `early` stays false even at zero target distance, so the stationary fielder accelerates his stored speed toward maximum indefinitely. The new replanning formula then treats that stored speed as current movement.

**Executed scenario:** At `t = 2`, SS starts at `(0,30)`, speed **7 m/s**, target `(0.05,30)`, `Role::Chase`, `t_int = 2`. After 30 movement ticks he has moved only **0.05 m**, but reports **7.463203 m/s**. Moving the target 3 m in the opposite direction produces **0.124560 m** of movement on the first tick, at **7.473603 m/s**, despite having stood still for almost half a second. A missed or misread intercept can leave exactly this expired-plan state until the next read; a stale timestamp is not required to demonstrate the clamp inconsistency.

**Suggested fix:** Keep speed consistent with actual displacement when reaching a target. If catching on the run is intended to preserve momentum, represent continued motion/heading through the intercept and apply braking or steering physically; otherwise clear the stopped speed before the next reach calculation. Include reaching a target, waiting, and replanning in the regression, rather than checking only first arrival time.

### 3. [P2] Closest approach is not the minimum of the reachability gap

**Location:** `sim/crates/bb_game/src/live.rs:429–456`, with standing-before-dive selection at `:400–402`.

The projection correctly minimizes horizontal distance to the fielder's **current position** along the chord. Reachability instead depends on `distance(t) − covered(t) − reach`. Its minimum can occur after geometric closest approach, as the fielder gains running distance. Testing only the projection and the endpoint can still skip the entire standing window and select a much later standing intercept over an earlier dive candidate.

**Executed scenario:** At `t = 2`, SS is stationary at `(1.1,30)`, with normal infielder attributes. Use a rolling predicted ball starting at `(0,27.4,ball_radius)`, velocity `(0,10,0)` m/s, deceleration **2.2 m/s²**, stopping at `2 + 10/2.2`. A 1 ms feasibility scan using the production `run_dist_from` and **0.7 m** planning reach finds a standing point at **2.296 s**, `(0,30.2636224)`. Production `intercept_scan` instead returns standing at **4.690 s**, `(0,46.34029)`, and a dive candidate at **2.170 s**. `intercept_inner` selects the late standing point. This is a wrong initial plan; close reads may subsequently rescue it, so this probe does not claim a guaranteed missed capture in the full loop.

The interpolated time and position also lie on a constant-speed chord, not necessarily on `pred.at(time)`: rolling motion accelerates, flight curves, and an interval may cross a bounce or wall. The arithmetic is correct for a straight constant-speed segment, but is not exact trajectory interpolation.

**Suggested fix:** Search/refine the reachability gap within each interval, evaluating the predicted path at the candidate time, and split intervals at surface contacts. A deterministic bounded refinement or the spec's finer ground-ball sampling would reduce the remaining phase dependence. Preserve the current chronological ordering of interior point before endpoint; that ordering is correct.

### 4. [P2] The resting-ball fallback still discards current speed and can reverse fielder priority

**Location:** `sim/crates/bb_game/src/live.rs:404–409`; assignment comparison at `:504–508`.

The scan now counts current speed, but its fallback still calls `run_time` from rest. Consequently the same fielder changes movement models when no pre-settle sample is reachable. This is an inherited fallback that the speed-preserving repair leaves inconsistent.

**Executed scenario:** At `t = 2`, a predicted ball at `(0,30,ball_radius)` rolls at **0.1 m/s**, decelerates at **2 m/s²**, and stops at **2.05 s**, at `y = 30.0025`. A fielder 6 m away, already running at **6 m/s**, gets fallback time **3.359052041 s**. A stationary fielder 5 m away gets **3.208339892 s** and therefore appears earlier to assignment. Inverting the production running-distance law from 6 m/s gives the first fielder arrival about **2.879954 s**, well before the stationary fielder. Both scans lack a reachable sample before the ball settles.

**Suggested fix:** Add/use the inverse of `run_dist_from` in the fallback, with the same reaction/recovery assumptions as movement and scan. Check a moving-versus-stationary assignment, not just equality of the distance formulas.

### 5. [P3] The MLB chase reference columns are not a reproducible matched comparison

**Location:** `sim/crates/bb_cli/src/pa_lab.rs:28–52`; `examples/chase_diag.rs:18–35`; `docs/calibration/2026-11-11-chase-diagnosis.md:9–45`.

The sim rows reproduce, but the MLB shares and swing rates are memory-based constants without a season, query, or matching bin definition. The code bins Euclidean distance from the physical zone rectangle minus ball radius, using fixed centimetre thresholds. Naming those bins Statcast attack zones does not establish that the MLB reference used the same geometry, treatment of corners, vertical normalization, or outside-only shadow subset. The official [Savant search](https://baseballsavant.mlb.com/statcast_search) supports attack-zone filtering, but I could not verify the exact **43 / 23.5 / 5.5%** league figures, or the **40 / 41 / 19%** shares, from a primary-source query during this review.

**Executed/code-derived scenario:** The baseline outside-shadow swing rate is **20.1%**, but the example itself reports **14.7 / 16.7 / 37.7%** with zero/one/two strikes. The sim also has unusually short PAs and few two-strike opportunities. Comparing this pooled rate to an undated league constant cannot isolate a swing-value defect from differences in count and location composition. The internal Discipline sweep does establish that its value model moves decisions; it does not independently prove that pitcher locations and perception need no further investigation.

**Suggested fix:** Save a dated MLB extraction with identical bin definitions, explicitly distinguish full shadow from outside shadow, and compare within count and finer location bands. Until then label the CLI references unverified estimates and soften the report's exclusion of alternative causes. The large internal chase deficit and the reproduced sweep remain useful observations.

### 6. [P3] The foul diagnosis generalizes a one-dimensional map slice into a structural ceiling

**Location:** `docs/calibration/2026-11-11-foul-share-diagnosis.md:35–49`.

The quoted zero-height-error timing window and the −5 cm, on-time result approximately reproduce. They do not define a rectangular contact/foul window over the joint timing-height distribution, or prove that no permitted noise/ability distribution can reach the target. The candidate list is appropriately marked unverified, but the preceding categorical claim goes beyond the sweeps.

**Executed scenario:** With the same perfect-perception middle-middle FF cage, zero vertical offset gives spray **−42.4° at −23 ms**, **+42.4° at +18 ms**, and **+47.3° at +20 ms**. However, **−5 cm / −35 ms** still contacts and is fair, with spray **−11.1°**, whereas **+5 cm / +35 ms** contacts with spray **+119.4°**. The contact boundary strongly depends on height; the 41 ms fair interval is only the zero-offset slice.

Reconstructing the production impulse inputs also confirms **Stick** at −5 cm/on time (82.9 mph, 56.1° launch), but **Slide** at −7 cm/on time (backward spray 156.6°). Sliding already exists; it is its transition and physical accuracy that need validation. The bat is already tilted: contact-axis `z ≈ −0.35`, about **20.5° below horizontal**, in the on-time case. Early/late contacts at the edge of the zero-offset window also reach **u = 0.8636 m**, the bat's rounded end cap. Contact station/end-cap geometry is an inexpensive additional diagnostic before attributing the gap to absent bat tilt or hand translation.

**Suggested fix:** Qualify the window as a slice and the ceiling as “not reached by these sweeps.” Inspect the joint timing-height-contact-station distribution, including end-cap contacts, impact branch, pitch location, and count-dependent swing approach. Validate the existing tilt and stick/slide transition before changing Tier A collision physics. No single most-likely physical cause is established by these tables.

## Other requested assessments

**Movement and determinism.** For a fixed target the clamp prevents position overshoot and position oscillation; it can also stop a chaser abruptly, as finding 2 demonstrates. I did not establish an ordinary ball being run through because of the new `early` rule alone. `t_int` is an absolute time, matching `self.t`; primary, secondary and close replans refresh it alongside the target. Cover roles always take the braking branch regardless of their stored timestamp, and nonfinite times also brake. The slow-ball direct-target override does not refresh the timestamp. The scalar distance law is the analytic counterpart of the step's first-order acceleration law; Euler stepping introduces a small discretization difference. Both currently permit instantaneous turns, so retaining speed across a turn is internally consistent with this skeleton, although it still differs from agent-models §5.3's heading/Agility model. Stopped-state speed and the fallback are the concrete inconsistencies here.

The new planner uses `bb_math::m`, adds no RNG draws or outcome rolls, and reads the fielder's noisy prediction rather than the true future path. Its interior candidate precedes its endpoint deterministically. Existing same-seed/thread/recording digest tests pass. Historical digests can change with intentional behavior changes; repeatability remains intact.

**Outfielder calibration.** Speed/reaction positional means and perception-noise scales are permitted classes of Tier B inputs under calibration-targets §2.3; fitting them is not itself an outcome roll or a compensating-pair violation. All three changes reduce fielding effectiveness, rather than moving opposing knobs to hide each other. However, fitting three defaults to two B2 hit rates does not identify their individual mechanisms. If treated as new agent-model [E] mappings rather than interim positional ability/noise defaults, §2.3 item 3 requires a justifying **Tier M** band; the report supplies B2 fits, not an outfield mechanism check. Retain the distinction and measure reaction/distance/route and perception behavior before presenting these as validated physical defaults.

Individually **8 m/s = 26.25 ft/s** is plausible as a chase speed near the public 27 ft/s sprint reference, and **0.40 s** is at the project's first-step envelope's upper bound. [MLB's Sprint Speed definition](https://baseballsavant.mlb.com/sprint-speed-scroller) measures a fastest one-second running window, not ball-tracking speed. [Statcast Jump](https://www.mlb.com/glossary/statcast/jump) measures early distance, rather than a stationary reaction delay. Neither directly validates these fitted numbers or the **3 m** lateral read error.

As an independently derived check, unobstructed straight running with the new CF/RF defaults covers **13.39 m / 43.94 ft in 3 s**, and **15.64 m / 51.31 ft in 3.3 s**, before adding reach. Those are useful conditional-distance probes, not measured average jumps: perception and routes can reduce progress. [MLB's catch-probability example](https://www.mlb.com/glossary/statcast/catch-probability) assigns only **10%** to **52 ft in 3.3 s**. Aggregate LD/FB agreement should therefore be supplemented with distance/time/direction catch curves and observed routes, rather than interpreted as independent validation of speed and reaction.

At `411af1a`, agent-models §10 still says reaction **0.25 s** without the new outfield exception; its sprint-speed mapping also does not explain the chase-speed distinction. The later **`72dc6a1`** adds an interim-default explanation, so the missing reaction/default documentation is subsequently addressed, subject to finding 1's actual indexing. The changed function also feeds `Game::new`, the live unit fixture, `bip_sweep`, `grounder`, and the reach diagnostic in `gb_through`. The old `play::simulate_play` skeleton consumes the supplied attributes too; these are shared defaults, not a live-planner-only configuration.

**Latest capture and scoring fixes.** `zmax_at(bn)` fixes the wall-strip loss, and refusing thrown-ball deferral when next height is at/below ball radius agrees with `update_thrown`'s conversion boundary. Both fixes survived targeted probes; the wall fixture fails with the old height guard. There is no committed regression specifically for the thrown-ball boundary; the external probe described below fails when that guard is removed.

For the cases it actually implements, `classify_bip` now correctly distinguishes a force elsewhere, an infielder's one-base putout choice, and an outfielder's baserunning putout independently of first-base arrival time. [2026 OBR 9.05(b)](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf#page=129) supports those distinctions. The acknowledged one-base/return qualifier and failed-choice judgment remain missing. Also, “first fielded” currently means **first secure possession**, so an earlier infielder deflection is invisible, and an appeal for missing a forced base, 9.05(b)(2), is not generally equivalent to `OutKind::Force`. These are limitations of the provisional diagnostic, not evidence that full official scoring has been repaired. Calling the new synthetic test OBR-based does not make it a complete scoring implementation.

**Day-fixes Resolution.** Deferrals 1 and 2 have real cross-layer dependencies: clearing pending runs needs rules-engine support, and first touch needs a fact independent of possession plus persisted representation. Deferral 1 leaves the earlier P1 scenario operationally unresolved. Its “defense can still appeal” mitigation is narrow: `holder_act` can emit an appeal when already at the original base, but throw-option selection skips `scored` runners; there is no general deliberate appeal to rescue the stranded runner. Deferral 5 is reasonable only while these hit/FC values remain explicitly provisional lab output: `bb_stats` does not use them, and they are not reconstructible from the persisted log. Their use as calibration evidence needs that caveat. Findings 3 and 6 are repaired in the claimed respects; finding 4 is correctly marked **fixed in part**. Nothing here establishes that all scoring or retouch cases are fixed.

**Diagnosis conclusions.** The reruns support the reports' measured low foul share, poor payoff from larger timing noise, and Discipline's ability to increase chasing while shortening PAs. They support deferring a chase-only retune. They do not establish a universal 30% foul ceiling, uniquely identify a collision defect, or validate the remembered MLB attack-zone columns. The overconfidence experiment was backed out; I did not recreate that uncommitted experimental implementation.

## Verification

- Ran `export PATH=$HOME/.cargo/bin:$PATH; cd sim && cargo test --workspace --release`: **101 tests passed**, no failures, including dependency-boundary, oracle, rules, and digest determinism checks.
- Ran `cargo build --release -p bb_cli`, then `./target/release/sim-cli games --n 512 --seed 2`: **3.87 runs/team-game**, no rules warnings, GB/LD/FB/PU provisional hit rates **.287/.691/.121/.018**, GB FC **.071**. The reported 55.6% GB share also reproduces.
- Ran `cargo run --release -p bb_game --example gb_through -- 128`: **1,183 routine grounders, 94.2% converted**; through/infield-hit/out totals **681/351/3,048**, and through-ball reach classes **110 standing / 207 dive / 364 unreachable**. TASKS “Resume here” matches these numbers. The ≥96% target remains unmet; the ≥93% envelope passes.
- Ran `chase_diag -- 3000` and `chase_diag -- 3000 0.4 0.8`: baseline **11.6% O-Swing, 14.1% K, 3.1 pitches/PA**; alternative **22.6%, 12.9%, 2.8**. Both tables reproduce their report rows.
- Ran `foul_diag -- 3000` and `foul_diag -- 3000 18`: baseline contact/foul share **75.2%/21.2%**, foul directions **0.4 pull / 14.7 opposite / 6.1 backward**; 18 ms row **68.2%/27.2%**, **2.2/17.9/7.1**, K **21.1%**, LA SD **32.1°**. These match the tables.
- All temporary probe source and mutations were under **`/tmp/fielding-review-probe`**; outputs are `/tmp/fielding-*.log`. The temporary workspace copied the reviewed source and reused build artifacts. Initial example linking hit the read-only default Zig cache; retrying with `ZIG_GLOBAL_CACHE_DIR` and `ZIG_LOCAL_CACHE_DIR` under `/tmp` succeeded. No administrator operation was needed.
- Executed the attribute, stopped-speed, rolling-path feasibility, and resting-ball priority probes described in findings 1–4. The thrown-ball probe placed a perfect-hands receiver 0.6 m from a throw at `z = .08 m`, velocity `(0,10,-4)` at `t = 2`; production secured it immediately. Removing only the new ground-boundary guard produced no immediate capture and failed the assertion.
- In the temporary copy, restored the pre-fielding `fielder_step` and `intercept_scan` while retaining current tests and the compatibility helper. **All three new fielding regressions failed**: between-samples at phase 5, retained-speed replanning, and chaser arrival. Restoring the pre-latest classification and capture guard makes the **new scoring test and wall-zone test fail**. The stronger runner tests pass on the immediate parent because that parent already has the underlying fixes; a constant timing-stream mutation fails the RNG test, and reinstating per-tick retouch debt fails the retouch-then-advance test. The RNG test observes timing draws only; as the Resolution acknowledges, it does not test the free-ball read stream or explicitly compare against the initial read.
- Executed the extended cage probe at timing −40…+40 ms and vertical offsets −7…+7 cm, reconstructing collision bodies/material from the returned contact to inspect Stick/Slide and energy-guard flags. The cited on-time −5/−7 cm cases used no energy fallback. The repository's normal cage-map CLI still prints only −15…+15 ms; the wider probe was external.
- Profiled **32 sequential games**, seeds `2 * 1_000_003 + i`, with `BB_PROF=1`: two current-code runs took **5.243/5.238 s** total and **4.368/4.352 ms of intercept work per game**. Disabling only the interior candidate in the temporary build took **5.295 s**, with **3.934 ms/game** of intercept work. Thus the observed scan cost is small; the roughly 0.42 ms/game intercept difference is indicative, not an isolated benchmark, since changed plays also change call counts. No whole-game slowdown was established by these timings.

## Resolution

Resolved 2026-11-11. All six findings acted on. `cargo test --workspace --release` passes.

| # | Status | What changed |
|---|---|---|
| 1 | Fixed | `FielderAttrs::average` takes the zero-based index; left field (6) now gets the outfield values. Test `all_three_outfielders_get_outfield_values`. The outfielder values were re-fitted with all three outfielders using them (below) |
| 2 | Fixed | Reaching the target sets the fielder's speed to zero. Test `stopped_fielders_have_no_speed_and_running_ones_keep_it` |
| 3 | Fixed | The scan works on the shortfall (distance minus reach minus ground covered). An interval is searched at a sixth of the step, on the predicted path itself, whenever the shortfall at its start is within what the ball and the fielder can close in one step. Test `intercept_finds_the_first_reachable_moment_on_a_slowing_ball` checks the review's scenario against an independent 1 ms search. Intervals are not split at bounces |
| 4 | Fixed | The resting-ball fallback uses `run_time_from`, the inverse of the distance law with the fielder's current speed (same test as #2) |
| 5 | Fixed (docs) | The chase report and `sim-cli pa` now say the MLB attack-zone figures are unverified estimates from memory, and the report no longer rules out pitch location or count mix. A dated extraction with matching bins is research item R13 |
| 6 | Fixed (docs) | The foul report now calls its timing window a zero-offset slice and its 30% a limit of the sweeps, and records the review's corrections: sliding already exists (stick at −5 cm, slide at −7 cm), the bat is already tilted ~20°, and edge-of-window contact is on the end cap |

Also changed while here: on a ground ball each infielder now decides to break from his own read (first bounce in the infield, his intercept in the infield), as agent-models §5.7 says, instead of from the primary chaser's read; the extra "within 0.6 s of the primary" condition, which the spec does not have, is gone. It made no measurable difference.

**Outfielder values, re-fitted with left field included:** reaction 0.35 s, chase speed 8.2 m/s, read error 2.5 m (the owner had accepted 0.40 / 8.0 / 3.0, which were fitted while left field was using infield values). The review's caution stands: three values fitted to two hit rates are not individually validated, and straight-line distance covered looks generous against Statcast's catch-probability examples. They are interim defaults to be checked against distance-and-time catch curves.

**Numbers** (`sim-cli games --n 512 --seed 2`): 4.29 runs/team-game, no rules warnings; provisional hit rates GB .311, LD .702, FB .121, PU .017. `gb_through -- 128`: routine grounders 92.3%, just under the 93% envelope (94.2% before these fixes; run-to-run noise on this measure is about ±0.7 points, and the owner's direction is to get close and move on). One game takes ~161 ms on one thread (budget 200 ms).

Not resolved here: the day-fixes review's deferred P1 (a runner who crossed home before a catch never returns), which this review notes is still open.
