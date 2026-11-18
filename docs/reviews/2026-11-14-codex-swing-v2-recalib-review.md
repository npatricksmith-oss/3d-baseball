# Independent review: swing v2 and recalibration

> Superseded figures: current numbers are in the [metrics ledger](../metrics-ledger.md) (§1).

Reviewed HEAD `6bd5047b80c77a523f368552e31f7e8778c87bda`, using `CLAUDE.md`'s invariants and PLAN D19. Scope: `git log --oneline main..HEAD` and `git diff main...HEAD`. Current `main` is `dae1b7a`; its merge base with HEAD is `f5f15a7`. No source, tests, specs, notes, ledger, or TASKS were edited.

No P1 finding. The kinematics, population draw, sampled replay checks, and headline metrics pass. The main substantive finding concerns the report's attribution of carry loss to backspin.

## Findings

### P2 — The spin counterfactual does not isolate the claimed backspin penalty

**Locations:** `docs/calibration/2026-11-14-swing-recalibration.md:86`, `:89`, `:152`; measurement in `sim/crates/bb_game/examples/aim_offset_probe.rs:25` and `:28`.

The table labels `|ω|` as backspin, then interprets scaling the entire vector to 2,500 rpm as evidence that excessive backspin costs 15–30 ft. Total spin includes substantial sidespin and gyrospin here. This matters because the report recommends prioritizing spin in the next collision-sourcing investigation on the strength of that attribution.

I reproduced the middle-height four-seamer at −3 cm: EV 97.718 mph, LA 27.256°, total spin 5,890.10 rpm, **signed backspin 4,364.11 rpm**, gyrospin −1,346.47 rpm, and vertical-axis spin −3,923.18 rpm. Backspin was measured as `ω · normalize((v.y, -v.x, 0))`, converted to rpm. At zero offset the table's positive “1,190 backspin” is actually total spin with **−338 rpm backspin** (topspin).

The vector and the original carry are real outputs of the collision model, not a probe-generated spin artifact. However, the attribution is confounded: scaling all components to 2,500 rpm reproduces carry **346.38 → 375.19 ft**; changing only the backspin component to 2,500 rpm, keeping the orthogonal components and launch state fixed, gives **346.38 → 340.36 ft**. At −2.5 cm the analogous backspin-only change gives **357.19 → 349.62 ft**, rather than the report's whole-vector result of 373.02 ft. The probe's different flight start-time label does not explain the result: using the actual contact time reproduces the reported whole-vector carry.

**Suggested fix:** label the existing measurement and counterfactual as total spin / whole-vector scaling. Report signed backspin, sidespin and gyrospin separately, cite a comparator measuring the same quantity, and distinguish their carry counterfactuals before attributing the loss or recommending which collision mechanism to change. This finding does not justify a physics adjustment under D19.

### P3 — The attack-angle/direction test passes the v1 swing it claims to reject

**Location:** `sim/crates/bb_agents/tests/swing_statcast.rs:183` and `:197`.

The test substitutes a single middle-middle swing's ±12 ms timing sweep for the reported population regression, then says v1's population slope of 0.26 fails its 0.35 lower bound. Those are different measurements. Compiling `main`'s v1 `swing.rs` in a temporary probe with the same aim, bat, 25 sample times and regression calculation gives **slope 0.37753347**, inside `[0.35, 0.80)`, and AD end separation **41.6130°**, also passing the second assertion. Thus this particular test provides a sign/geometry check but not the advertised v1 regression protection. Other new tests still reject v1's major defects.

**Suggested fix:** correct the claim and retain this as a geometry check, or add a representative mixture of timing, location and height evaluated consistently for both models and the independent population expectation. Do not tighten the single-swing threshold solely around v2's current answer.

### P3 — The original swing note assigns the cage result to the wrong vertical offset

**Location:** `docs/calibration/2026-11-14-swing-model.md:127`.

The note says the on-time cage four-seamer leaves at 2–4° with the aim 2 cm under center. `Cage::plan_perfect` aims at the ball center; it does not apply the PA model's `template.aim_offset_z`. My cage sweep reproduces **2.801° at zero offset**, **18.977° at −2 cm**, and **27.256° at −3 cm**. The recalibration note's §2.2 table correctly reports these values, so the two notes disagree on a mechanism central to proposal C.

**Suggested fix:** identify the old 2–4° measurement as the zero-offset cage case and point to the explicit offset sweep. If another measurement was intended, supply its actual placement and reproduction command.

### P3 — The binding swing spec contradicts the executed speed normalization

**Location:** `docs/agent-models.md:330`; implementation `sim/crates/bb_agents/src/swing.rs:453` and `:459`.

“Execution never changes the speed scale k” contradicts the new effort draw: both rate peaks are multiplied by `1 + effort`. Plane-tilt placement also rebuilds the geometry and normalization, potentially changing k through the solved radius. The effort row below does not resolve the categorical statement or explicitly specify how effort enters both angular-rate integrals. An implementation following that sentence would lose the modeled within-batter speed spread.

**Suggested fix:** specify that the rebuilt contact geometry is normalized to `BatSpeed`, then both angular-rate bumps are multiplied by `1 + effort`; timing shifts the contact time and translations move the swing. State the resulting sweet-spot speed at executed nominal contact as `BatSpeed × (1 + effort)`.

## Verification

### Commands and execution conditions

- Read `CLAUDE.md` first; inspected the complete scoped diff, commit log, PLAN D19, agent-models §3.6/§10, data-formats §2.4, calibration-targets §4.3, both notes and the ledger.
- Used `PATH=$HOME/.cargo/bin:$HOME/.local/bin:$PATH`, `CARGO_BUILD_JOBS=2`, and `taskset -c 0-7 nice` for builds, tests and simulation runs. Ran heavy jobs sequentially. Read the `k10temp` device's `temp1_input` before heavy runs; recorded starts were below 67°C, so no >90°C cooldown was required.
- `cd sim && taskset -c 0-7 nice cargo test --workspace --release`: **320 passed, 0 failed, 3 ignored**, across 53 successful test-result summaries. Includes crate-graph, roster/oracle, population, swing oracle, six Statcast swing tests, exit-speed, home-run-trot and determinism checks.
- `python3 -m unittest analysis/oracle/test_oracle.py analysis/oracle/test_swing_oracle.py`: **24 passed**, stdlib only.
- Built HEAD `bb_cli` with `--release --no-default-features --bin sim-cli`, and `bb_game` examples `run_env_diag`, `digest`, `aim_offset_probe`. The first uncached links encountered the sandbox's read-only default Zig cache; subsequent links used copied Zig caches under `sim/target/review/`. No privilege escalation or external build-output directory was used.
- Temporary Rust/Python probes, archived historical sources, logs and build trees are under `sim/target/review/`; workspace test log is `sim/target/review-tests.log`. Probes were compiled with `rustc --edition=2024 -O`, the workspace's Zig linker, and explicit release-library dependencies.
- `git diff --check main...HEAD`: clean.

### Invariants, RNG and population

No new outcome roll, world-state read by decision code, or decision dependence on `Executed*` was found. The PA orchestrator retains execution deviations while rebuilding the agent's adjusted intended plan; those deviations do not enter `batter::decide` or its belief update. `bb_agents` dependencies remain `bb_math`, `bb_physics`, `bb_observe`; both crate-graph tests pass. No new production transcendental outside `bb_math::m`, `mul_add`, SIMD, or hash-container iteration was found. Direct trigonometric methods present in fielding tests predate this change.

`SwingEffort = 14` and `RosterAbilities = 15` append to unchanged purposes 0–13. `SwingExecution` still draws vertical offset, plane tilt, reach, timing in that order with the same truncation; effort has its own keyed stream. `roster.rs` is unchanged in the scoped diff, as are its identity/hand streams; roster golden tests pass. Adding abilities therefore preserves existing seeds' names, numbers, order and hands.

The population uses one rejection-truncated standard normal at ±3 from `(seed, play=0, field_agent, RosterAbilities, sub=0)`, grade `50 + 10z`, then `32 + 1.25z` m/s. Each batting slot receives its player's field-agent draw, including the pitcher. This agrees with the spec; position shifts and correlations are explicitly deferred. No run-total target or outcome-specific multiplier is present.

Re-ran `python3 -I analysis/statcast/savant_sd.py <existing six-date Savant CSV directory>` against the retained source CSVs. It reports **216 batters / 7,697 competitive swings**, between-batter SD **2.90 mph**, mean of means **71.68 mph**, pooled within-batter SD **4.29 mph**, mean sample count **35.6**, sampling-error estimate **0.72 mph**, and fixed-depth/middle within-batter SD **3.21 mph**. The report's arithmetic is correct: `sqrt(2.90² − 0.72²) = 2.809199 mph = 1.255824 m/s`; 1.25 m/s is a reasonable rounding of the stated source/ruling. Removing sampling error using the average sample count is an approximation, not an exact heteroskedastic variance estimate, but does not undermine that rounding at the quoted precision.

### Independent kinematics probes

Used six new aims `(x,z)` in metres: `(-0.31,0.58)`, `(0.29,0.91)`, `(-0.13,1.04)`, `(0.17,0.47)`, `(-0.65,0.75)`, `(0.65,0.75)`, with template contact depths and nominal contact 0.437 s. These cover low/high, inside/outside and both reach clamps.

- Compared 18 unperturbed samples at relative times −37, 0, +16 ms with the independent Python oracle's numerical rate integration and position differentiation: maximum knob error **4.1e−16 m**, maximum velocity error **1.05e−8 m/s**. Existing fixture cross-checks also pass.
- Applied efforts −0.12/0/+0.16 together with nonzero tilt, reach, vertical and timing placement; differentiated positions centrally with `dt=1e−6 s` at six early/late times per swing. Maximum analytic-velocity error over 108 samples: **1.54e−8 m/s**.
- Executed nominal-contact sweet-spot speed agrees with `BatSpeed × (1 + effort)` to **1.1e−14 m/s**, including clamped swings. Existing unit tests also verify reachable aims are met at that speed without placement deviations.
- Corresponding left-handed positions and velocities are exact x-mirrors in those sampled operations (**zero measured difference**). Angular velocity is an axial vector and mirrors accordingly.
- Located radius clamp transitions at middle height `z=0.75`: **x=−0.290591207 m** (ρ=0.2775 m), **x=+0.288875082 m** (ρ=0.851 m). Sampling x ±1e−8 m gives position differences below **1.36e−8 m** and velocity differences below **1.84e−7 m/s**: continuous across both boundaries, with the expected derivative kink in the clamped inverse. A clamped swing retains C's lateral coordinate and misses along the bat axis rather than translating C to force the sweet spot onto the aim; speed remains normalized. The upper-clamp unit test independently checks that miss direction.

### Determinism and historical comparisons

Archived historical `sim/` trees with `git archive`; built `9f71441` separately and built **each** of `9c2e35e` and `90c8706` with its own initially empty Cargo target directory, using `cargo build --offline --release -p bb_game --example digest --example run_env_diag`. A preliminary shared-target historical build reused stale artifacts and was discarded; the clean builds are the basis of the comparisons below.

- HEAD games for seeds **0, 1, 2, 3, 42, 1000**, run twice, have identical event-log digests.
- HEAD with `Population::average()` matches `9f71441` digests for **1, 2, 3, 42**. `BB_BAT_SPEED_SIGMA=0 .../run_env_diag 100 0` matches the separately built `9f71441` diagnostic **byte for byte**, not just its rounded totals.
- Clean `9c2e35e` and `90c8706` builds produce identical digests for **1, 2, 3, 42**, also matching HEAD. This confirms the literal-naming change preserves sampled behavior, and supplies a separate-build comparison for HEAD (its sim sources match `90c8706`).
- Example HEAD digest, seed 1: `51f69f5d6984a0a117fb2b1e8fc8cd3b508c35d0a51dcc8a927c3ba3d7089543`. Its σ_pop=0 / `9f71441` counterpart: `73777a6e0174afbc7face02edad04eea6e6f757c5324c252b508082f24a16243`.

### Headline metrics and interpretation

`taskset -c 0-7 nice target/release/sim-cli games --n 300 --seed 0` reproduces the ledger/recalibration row:

| Measure | Measured HEAD |
|---|---:|
| Runs / team-game | 4.79 |
| HR / PA | 1.01% |
| K / BB | 15.5% / 7.4% |
| Reached on BIP | 39.3% |
| GB / LD / FB / PU share | 45.1 / 24.3 / 24.9 / 5.6% |
| Hit rate by those types | .367 / .677 / .168 / .026 |

The saved standard output agrees in its statistical lines; its introductory wording now mentions drawn BatSpeed, and elapsed timings differ.

`taskset -c 0-7 nice target/release/examples/run_env_diag 100 0` reproduces the recalibration diagnostic, including the saved `sig125_env.txt` output byte for byte:

| Measure | Measured HEAD |
|---|---:|
| Runs / team-game; K; HR/PA | 4.535; 15.8%; 0.96% |
| EV mean / SD | 89.56 / 12.62 mph |
| EV p90 / p99 / max | 103.2 / 109.8 / 119.6 mph |
| EV ≥105 / ≥110 | 6.5% / 1.0% |
| LA mean / SD | 11.82 / 25.10° |
| Barrels / fair BIP | 4.55% |
| Bat speed on fair BIP mean / p90; on HR | 71.4 / 77.1; 77.7 mph |

The exit-speed constraint is supported: there are 408 fair BIP at 25–30° launch but only 18 over-fence HRs there; the mean EV of those HRs is 105.9 mph. The on-time average cage swing at 27.3° produces only 97.7 mph. The offset sweep therefore does not show a simple launch-mean correction capable of supplying the missing power. This supports retaining the sourced population spread and the existing aim offset; it does **not** prove that launch distribution is irrelevant or establish the report's specific backspin explanation. I did not re-run all aim/noise/population counterfactuals, the full pre-v2 standard run, or the seed-1000 300-game metric row.

### Tests, specs and fielding scope

The numerical oracle uses integration and finite differences rather than Rust's erf/analytic velocity implementation. Statcast speed, spread, length and angle checks add useful external constraints; tilt means are fitted by construction and the tests acknowledge that. The identified attack-angle regression limitation is the exception to their blanket “v1 fails” claim. The merge retains the new Statcast test file byte for byte, and `home_run_trot` retains its original minimum coverage assertion while allowing more seeds to obtain homers; no weakened assertion was found.

Apart from the speed-scale sentence above, the checked default geometry/timing constants, population mapping, appended purposes, [E] labels and the attack-angle linear approximation agree with the specs. The newly named tilt guard, tilt-noise ratio and reach-noise ratio are present with the documented values. No additional buried empirical swing parameter requiring a name was identified.

In **`git diff main...HEAD`**, production changes in `live.rs` and `play.rs` are exclusively spray-belief/refit constants (including `BAT_YAW_RATE_DPS`), with accompanying comments and recalculated tests. There is no added fielding-policy change in this branch delta. A literal `git diff main HEAD` is broader because current main already contains subsequent infield fixes absent here; this review does not certify an integration with those later changes. The reproduced ledger numbers describe the reviewed branch, based on `f5f15a7`.

## Resolution

All four findings fixed on `work/swing-tests` (documentation, one probe, one test comment; no sim behaviour changed: `git diff work/swing-recalib` touches only `aim_offset_probe.rs` and `swing_statcast.rs` under `sim/`, and the `digest` example's seed 1 digest is still `51f69f5d…`, matching this review's HEAD).

- **P2, spin attribution: fixed** (`4fc09e7`, `6c58c23`). `aim_offset_probe` now prints total spin, signed backspin, sidespin, gyrospin and ω_z per offset, and the carry for the whole vector at 2,500 rpm, backspin alone at 2,500, sidespin and gyro zeroed, and each zeroed alone. It reproduces this review's −3 cm numbers exactly (backspin 4,364, gyro −1,346, ω_z −3,923 rpm; carry 346.4 → 375.2 whole-vector, → 340.4 backspin-only); the backspin axis `normalize(v.y, −v.x, 0)` was checked against physics-spec §1.3 (+X for a ball moving +Y). The recalibration report's §2.2 and §3 and the ledger row now say that the 5,890 rpm is total spin, that the carry loss follows the **sidespin** (zeroing it alone gains 29 ft; zeroing the gyro alone 3; setting the backspin alone to 2,500 rpm loses 6), and that the real Tier A question is the sidespin (3,720 rpm at the 27° barrel launch against about 2,000–2,500 in Statcast's opposite-field fly balls, Nathan 2020). The published comparator is cited with what it does and does not give: no batted-ball backspin or sidespin distribution by launch angle was found; Nathan's 2,000–3,000 rpm is a selection window. The sidespin and the opposite-field spray offset (§2.4) very likely share one cause (the bat's downward tilt at contact on an undercut ball, Nathan's §IV); the probe shows them rising together but does not isolate it (a λ_c or zero-tilt counterfactual would). No physics changed, per D19.
- **P3, attack-angle test: fixed as a claim, test kept** (`32504ce`). The comment now says it is a one-swing geometry and sign check that v1 also passes (0.378, reproduced by compiling `bcbb91f`'s `swing.rs`). The population version asked for (5 × 3 aims times 41 contact times at normal quantiles, timing SD set so the attack direction's SD is Statcast's 18.7°, not tuned to the slope) gives 0.533 for v2 and 0.355 for v1 against Statcast's 0.41, so it cannot separate them (v1 is nearer); it was not added, and the threshold was not tightened around v2.
- **P3, swing note line 127: fixed** (`c02b1bb`). The 2–4° cage launch is the zero-offset case (2.8°); a bat 2 cm under gives 19.0°, 3 cm 27.3°; the planned attack angle there is about 8°, not 7°. The note points to the recalibration report's §2.2 sweep.
- **P3, spec agent-models §3.6: fixed** (`05a157d`). The sentence now states what `apply_placement` does: a plane-tilt deviation rebuilds the geometry and is renormalised to `BatSpeed`; effort multiplies both rate bumps by `1 + effort`, so the sweet-spot speed at the executed nominal contact is `BatSpeed × (1 + effort)`; δ_v, δ_r and t₀ change no velocity.

Left alone: `docs/calibration/2026-11-12-fielder-momentum.md` and the run-environment diagnosis (§151) still describe the batted backspin as extreme; they are dated records of earlier measurements and were not rewritten.
