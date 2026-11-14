# Independent review: speed-dependent tangential restitution

Reviewed 2026-11-14 on `work/collision`, HEAD `be66dab`. Read `CLAUDE.md` first and applied its invariants, PLAN D20 and §13, and physics-spec §8.1–§8.2.1. Scope is the requested `git log f710d9f^..be66dab` / `git diff f710d9f^..be66dab`: the decision, specification, Python oracle and fixtures, Rust collision/contact changes, A14 and A10a–c, diagnostic examples, collision note §8, and the metrics-ledger row. The five merged target-note commits were checked only for the observations this change relies on. No source, test, specification, ledger, or TASKS file was changed; no commit was made. All temporary probes and new outputs are under `sim/target/`.

The implementation matches the owner's law and the impulse formulation. All 338 release workspace tests and 22 Python tests pass. The requested cage anchors, both standard runs, and all seven refit rows reproduce. There are two findings below: a regression-test coverage gap and an incorrect auxiliary table value. The empirical evidence supports fitting the spin anchors; it does not establish that actual restitution falls linearly with speed.

## Findings

### P1

None found: no demonstrated wrong production result or broken invariant in the reviewed change.

### P2-1 — A14 bypasses the contact integration that selects the impact speed

**Location:** `sim/crates/bb_physics/tests/collision_golden.rs:237–249`, particularly line 249; production integration is `sim/crates/bb_physics/src/contact.rs:189–193`.

**Evidence:** A14 reconstructs `Body` values, computes the normal speed and `params.e_t(cn)` inside the test, then passes that material directly to `collision::resolve`. It never calls `resolve_contact`. This verifies the standalone law and impulse solver, but not the new production selection of restitution from the pre-impact contact-point velocity.

A temporary probe using the actual A10a and A10b/c geometries, with a deliberately incorrect **34–60 m/s** contact ramp, passes all their checks: tee slope 184.152 rpm/degree with Eq. 8 agreement, backspin 2,377.952 rpm and sidespin slope −74.549 rpm/degree. A14 would catch this error if the standalone `BatBallParams::e_t` changed, but not if the error lived only in `resolve_contact` while the standalone function remained correct. Thus the endpoint integration tests do not close this gap. A 30–60 ramp also passes A10b/c, although A10a's tight Eq. 8 check rejects it.

This is a coverage finding, not evidence that the present wiring is wrong. The additional review-only `spin-review-contact-probe.rs` calls the real `resolve_contact` on all five oracle geometries, holding normal restitution at the fixture's independently specified 0.45. It agrees with Python to at most 2.62e−10 m/s and 9.56e−10 rad/s.

**Suggested fix:** Extend A14 to reconstruct `ContactPoint`/`BatKinematics` and exercise `resolve_contact` against independently generated ball velocity and spin, including a mid-ramp rotating-bat case. Keep the standalone law checks. Hold e_n explicitly at the fixture value or generate a spec-derived fixture with the normal-COR law included; do not obtain expected outputs from Rust. Include a counterfactual check that an incorrect contact-only ramp fails.

### P3-1 — Constant-zero tee slope is about 116, not 131 rpm/degree

**Location:** `docs/calibration/2026-11-14-collision-sloped-barrel.md:229`; the derivation repeated from line 74 is the source of the error.

**Evidence:** The table's `(131)` comes from dividing the old approximately 183 rpm/degree by 1.4. Eq. 8 scales spin at fixed contact angle ψ, but changing e_t also changes launch angle θ, the regression's independent variable. Repeating the six-point ψ = 5, 10, 15, 20, 25, 30 degree tee sweep gives **115.618 rpm/degree** at constant e_t = 0, versus 184.039 for the new law. The five-point A10a sweep gives 115.160 versus 184.152. The main 184-against-179 anchor is correct; this error affects the auxiliary zero-restitution comparison.

**Suggested fix:** Replace the 131 estimate in both tables with the measured approximately 116, identify the sweep, and remove the `×1/1.4` derivation for spin versus launch angle.

## Review assessment

### Physics, source, and invariants

- `bat.rs:233–241` implements 0.405 through 34 m/s, linear interpolation to zero at 65, and zero above. The two values at the joins agree exactly; ±1e−9 m/s boundary probes show continuity. Parameters are named public fields; the empirical speed anchors are not buried literals.
- `contact.rs:189–193` uses `abs((v + ω×r₁ − V − Ω×r₂)·n)` before any impulse. It uses contact-point relative velocity, not ball speed, bat speed, or the relative speed's magnitude. Units are m/s. With n from bat to ball, approaching c_n is negative; the solver targets c_n′ = −e_n c_n and c_t′ = −e_t c_t. A second impulse reuses the original restitutions as specified.
- The scalar function expects an already absolute speed. Passing a negative scalar directly clamps to the lab value; the physical caller takes `abs` first. Zero and separating physical contacts produce no impulse. The contact probe verified zero and +10 m/s separating contacts for every fixture. There is no sign reversal or unintended impulse at zero speed.
- Making e_t depend on the initial speed does not change impulse bookkeeping: the same J and −J act at the common contact point, with the corresponding torques. Across the five independent oracle cases, linear-momentum residuals are below 3.7e−15 kg·m/s, angular-momentum residuals below 9.2e−16 kg·m²/s, and total kinetic energy decreases by 47.7–229.7 J. Rust matches the oracle and retains its existing energy retry/frictionless fallback. Reduced spin is not generally a conserved-energy transfer into ball translation: total kinetic energy also changes, and the exit-speed response depends on contact geometry.
- [Kensrud, Nathan and Smith](https://arxiv.org/pdf/1610.03464), Eq. 8, Fig. 3 and Table II support the tee equation, measured 179 rpm/degree, and the quoted all-bat 0.405; wood alone is 0.464. The owner explicitly selected the all-bat value. The paper does not supply the falling game-speed ramp. Its raw speed range extends into the ramp; the spec acknowledges this. The 34/65 anchors and interpolation remain an accepted modeling hypothesis, not a laboratory speed law.
- [Nathan 2020](https://baseball.physics.illinois.edu/carry-v2.pdf), Figs. 2 and 4 and footnote 2 support the approximate spin targets and the warning that the spin components depend on an inferred axis. Fig. 2 selects 2,000–3,000 rpm backspin; Fig. 4 is the separate backspin observation. The test's broader EV ≥90 mph cage grid is a calibration check against these bands, not a held-out population validation of e_t(speed).
- No obsolete production bat restitution constant remains. Explicit constant e_t values in generic collision fixtures, ground materials, and counterfactual diagnostics have different purposes. No world/agent dependency or outcome-roll code changed. The crate-graph tests pass. Production changes contain no transcendental call, `mul_add`, SIMD, hash iteration, or new RNG use. Native transcendental calls added in tests/diagnostics do not enter game execution.
- Same-build replay was checked twice for seeds 0, 7 and 1,000,003,000, using the fielders/game path used by the standard CLI. All digests match:

| Seed | Digest, identical in both runs |
|---|---|
| 0 | `7eade59825ccd854695f0c72525f7d391b1f494562001a2d68ce26eae7f37fc2` |
| 7 | `05522110fa4878e42e0a268fcbfb943bc27bf336588ef4ae16819d3dde968b2b` |
| 1,000,003,000 | `0b6e49f36cdcb89754c1992db3e40e26930bac4e38a072cbf2334a5baad87821` |

### Oracle independence and test sensitivity

`analysis/oracle/bbphys.py:403–422` independently implements the spec's interpolation and contact-point speed in standard-library Python. The generator imports that implementation, not Rust, and uses no Rust executable or results. Python tests independently assert the anchors, midpoint, continuity, impulse restitution and tee Eq. 8. Rebuilding `build_corpus()` and serializing with the generator's exact format makes all six fixtures byte-identical. SHA-256 of `tangential_restitution.json`: `3f1db16c1483b893e7f0370b857538f1dbb3e65b121b5b61e7cf6436cf4b0669`.

A14's 1e−15 law tolerance and 1e−9·(1+norm) vector tolerances readily distinguish model changes. A10a combines an independent analytic relation with the independent 179 ±18 lab number. Its Eq. 8 calculation appropriately shares physical input parameters; that portion alone does not independently establish their numerical values. A10b/c use external spin bands and reject the old high-spin model. None simply asserts the output equals a formula copied from the new ramp. Their sensitivity is deliberately different:

| e_t applied to contacts; tee reference held at spec's 0.405 | A10a Eq. 8 | A10a 179 ±18 | A10b backspin | A10c sidespin |
|---|---|---|---|---|
| Correct 34–65 ramp | pass | pass, 184.15 | pass, 2464.74 | pass, −79.55 |
| Wrong 30–60 ramp | fail | pass, 189.38 | pass, 2375.88 | pass, −74.35 |
| Wrong 34–60 ramp | pass | pass, 184.15 | pass, 2377.95 | pass, −74.55 |
| Constant zero | fail | fail, 115.16 | pass, 2364.41 | pass, −73.19 |
| Constant 0.40 | fail | pass, 183.19 | fail, 4205.48 | fail, −109.60 |
| Constant 0.405 | pass | pass, 184.15 | fail, 4227.01 | fail, −110.16 |

A14's law checks reject every incorrect law in this table if that law replaces the standalone function. The P2 finding concerns a regression confined to its production caller. These probes vary contact parameters without editing the crate. They are not claimed to be mutations of the checked-in implementation.

### Measurements and causal claims

Both after runs match the existing logs in every non-timing line. Replacing only contact restitution with constant 0.40 through `Game::map_matchups` reproduces both before logs in every non-timing line, without editing source or checking out another revision. The production-code comparison from `4254e73` to the pre-change base contains only diagnostic examples, supporting that baseline equivalence. The CLI's seed-1000 label actually starts at 1,000,003,000, as the ledger already explains.

| Metric | Before, seed 0 [1000] | After, seed 0 [1000] |
|---|---|---|
| Runs/team-game | 4.82 [4.50] | 5.76 [5.78] |
| HR/PA | 1.04% [0.98%] | 2.36% [2.24%] |
| K% | 15.6 [15.8] | 16.8 [16.7] |
| Fouls/game | 17.8 [17.5] | 21.1 [21.4] |

The cage returns 184 rpm/degree, 2,464.735 rpm backspin and −79.552 rpm/degree sidespin. The 3 cm undercut moves from 27.3 degrees/97.7 mph to 30.4/98.6, as stated. Constant-zero counterfactual carry there is 364 ft versus 346 at 0.40. Thus reduced tangential impulse, changed launch, and changed spin provide a supported qualitative mechanism for more home runs; the full-game HR delta was not decomposed into independent speed, angle, spin and game-sequence contributions.

Both `foul_diag 10000` outputs reproduce byte for byte. Foul/contact rises 26.0% →29.2%, opposite-side 18.5 →21.3; the 40–50 degree contact count rises 1,022 →1,133 and 50–60 rises 675 →792. Their conditional foul rates rise 62.7 →65.5% and 75.7 →77.7%. Together with the matched undercut contact, these support the stated foul mechanism. The additional pitches and K movement reproduce; attributing their entire numerical changes exclusively to fouls would require a controlled count/sequence decomposition that these runs do not provide.

All seven refit outputs reproduce **byte for byte**, including diagnostics beyond the note's table:

| Swing / perception | Runs | K% | HR/PA | Maximum EV, mph |
|---|---|---|---|---|
| 1.0 / 2.7 | 6.175 | 16.5 | 2.64% | 118.1 |
| 1.4 / 2.7 | 5.455 | 18.2 | 2.68% | 120.3 |
| 1.8 / 2.7 | 5.205 | 20.4 | 2.95% | 128.0 |
| 2.0 / 2.7 | 5.235 | 21.2 | 2.59% | 131.6 |
| 1.0 / 3.5 | 3.615 | 31.1 | 1.66% | 116.2 |
| 1.8 / 2.9 | 4.65 rounded | 24.1 | 2.31% | 128.0 |
| 2.0 / 2.9 | 4.39 rounded | 25.4 | 2.56% | 131.6 |

The tail conclusion is supported by more than a maximum: fair-BIP EV p99 rises 110.3 →113.9 →115.4 mph for swing 1.0/1.8/2.0, while mean EV falls. `motor.rs:81` scales effort σ with the same swing knob, and `swing.rs:459–460` multiplies both angular-rate bumps by `1+effort`. These sweeps demonstrate a real broadening, not that miss noise by itself produces that tail.

### Refit proposal and direction record

Splitting effort σ from placement/timing noise is physically meaningful and permitted by PLAN §13. Both are physical input-noise scales. Effort already has a separate RNG stream and a sourced 4.5% scale; vertical offset, reach, plane tilt and timing perturb different aspects of the swing. Preserve the effort-speed calibration and keyed draw contract. When implementing the split, also keep the decision-side `OwnExecution` model (`pa.rs:481–482`) consistent with the miss noise delivered to the motor. This requires a spec change and measurement, as the plan says; it requires no outcome fudge factor.

The proposed miss-only scale, maximum EV around 120, and perception 2.9–3.0 are hypotheses for the next sweep. None was actually measured in isolation here. The existing combined-knob rows must not be relabeled as results of the proposed split. No prediction that it will preserve the exit tail or attain a particular HR rate is established by the present sweeps.

The direction gap is accurately recorded as known and unexplained, at the approximate precision used. The twelve-date scripts reproduce 9,165 balls in play; square-contact spray +22.0, +9.3 and −1.5 degrees; and the batter-tilt tercile slope difference −0.183 ±0.058 over 8.6 degrees. The old/new PA fits for spray minus AD are −0.805/−0.768 per degree; their small difference supports the note's statement that changing e_t does little to the gap. A10d remains report-only: the current sidespin zero is −1.008 degrees versus the published approximately −10. No direction fix was attempted or proposed in this review.

## Verification

Commands ran from the repository root unless marked `sim/`. Repeated log-inspection commands are grouped below. All heavy builds, tests and simulation runs were sequential and used `taskset -c 0-7 nice`; `CARGO_BUILD_JOBS=2` was exported for cargo and simulation batches. The CLI internally reports 16 worker threads, but affinity limits them to the requested eight CPUs.

### Builds, tests, simulations, and probe commands

Environment used for successful cargo/linking commands in `sim/`:

```bash
export PATH=$HOME/.cargo/bin:$HOME/.local/bin:$PATH
export CARGO_BUILD_JOBS=2
export ZIG_GLOBAL_CACHE_DIR=$PWD/target/zig-global-cache
export ZIG_LOCAL_CACHE_DIR=$PWD/target/zig-local-cache
```

| Command | Result |
|---|---|
| `taskset -c 0-7 nice cargo test --workspace --release > target/spin-review-tests.log 2>&1` (`sim/`, initial attempt without redirected Zig caches) | Failed because Zig attempted to create a directory under read-only `/home/natex/.cache/zig`. No test failure. |
| Same test command with the environment above | Exit 0; 338 tests passed, zero failed. Existing duplicate example-output warnings. |
| `python3 -m unittest analysis/oracle/test_oracle.py` | Exit 0; 22 tests passed. |
| `taskset -c 0-7 nice cargo build --release -p bb_cli -p bb_game --examples > target/spin-review-build.log 2>&1` (`sim/`) | Exit 0. |
| `taskset -c 0-7 nice cargo build --release -p bb_cli > target/spin-review-cli-build.log 2>&1` (`sim/`) | Exit 0. |
| `taskset -c 0-7 nice target/release/sim-cli games --n 300 --seed 0 > target/spin-review-games-seed0.txt` (`sim/`) | Exit 0; reported after values reproduced. |
| `taskset -c 0-7 nice target/release/sim-cli games --n 300 --seed 1000 > target/spin-review-games-seed1000.txt` (`sim/`) | Exit 0; reported after values reproduced. |
| `taskset -c 0-7 nice target/release/examples/collision_anatomy > target/spin-review-anatomy.txt` (`sim/`) | Exit 0; cage anchors and contact counterfactuals reproduced. |
| `taskset -c 0-7 nice target/release/examples/foul_diag 10000 > target/spin-review-foul-law.txt` (`sim/`) | Exit 0; byte-identical to existing law output. |
| `taskset -c 0-7 nice target/release/examples/foul_diag 10000 et=0.40 > target/spin-review-foul-old.txt` (`sim/`) | Exit 0; byte-identical to existing constant output. |
| `taskset -c 0-7 nice target/release/examples/run_env_diag 100 0 > target/spin-review-env-base.txt` (`sim/`) | Exit 0; byte-identical baseline. |
| `BB_K_SWING=1.4 taskset -c 0-7 nice target/release/examples/run_env_diag 100 0 > target/spin-review-env-s14.txt` (`sim/`) | Exit 0; byte-identical. |
| `BB_K_SWING=1.8 taskset -c 0-7 nice target/release/examples/run_env_diag 100 0 > target/spin-review-env-s18.txt` (`sim/`) | Exit 0; byte-identical. |
| `BB_K_SWING=2.0 taskset -c 0-7 nice target/release/examples/run_env_diag 100 0 > target/spin-review-env-s20.txt` (`sim/`) | Exit 0; byte-identical. |
| `BB_K_PERCEPTION=3.5 taskset -c 0-7 nice target/release/examples/run_env_diag 100 0 > target/spin-review-env-p35.txt` (`sim/`) | Exit 0; byte-identical. |
| `BB_K_SWING=1.8 BB_K_PERCEPTION=2.9 taskset -c 0-7 nice target/release/examples/run_env_diag 100 0 > target/spin-review-env-s18-p29.txt` (`sim/`) | Exit 0; byte-identical. |
| `BB_K_SWING=2.0 BB_K_PERCEPTION=2.9 taskset -c 0-7 nice target/release/examples/run_env_diag 100 0 > target/spin-review-env-s20-p29.txt` (`sim/`) | Exit 0; byte-identical. |
| `taskset -c 0-7 nice target/spin-review-probe > target/spin-review-probe.txt` (`sim/`, three runs as the temporary probe was refined) | Exit 0 each time; final run uses fixed spec tee reference, sensitivity table and repeated digests above. |
| `taskset -c 0-7 nice target/spin-review-old-games 0 > target/spin-review-old-seed0.txt` (`sim/`) | Exit 0; every non-timing line matches previous before log. |
| `taskset -c 0-7 nice target/spin-review-old-games 1000 > target/spin-review-old-seed1000.txt` (`sim/`) | Exit 0; same for second before log. |
| `taskset -c 0-7 nice target/spin-review-contact-probe > target/spin-review-contact-probe.txt` (`sim/`) | Exit 0; five end-to-end oracle checks, no-impulse cases and law boundaries pass. |
| `taskset -c 0-7 nice target/spin-review-tee-zero > target/spin-review-tee-zero.txt` (`sim/`) | Exit 0; six-point slopes 184.038670 and 115.617539. The printed fixed-0.405 Eq. 8 check is intentionally not applicable to ψ=5, which enters the ramp. |

Temporary Rust probes were generated by `python3 -B - <<'PY'` heredocs: (1) copy the batted-spin sweep into `spin-review-probe.rs`, parameterize contact restitution, add tee checks and repeated game digests; (2) add the 34–60 case; (3) copy the CLI game report into `spin-review-old-games.rs`, replacing only the game wrapper with a constant-0.40 matchup map; (4) create `spin-review-contact-probe.rs` from the five independent JSON inputs; (5) hold the tee reference at spec 0.405 and create the six-point `spin-review-tee-zero.rs`. Each command exited 0 and wrote only under `sim/target/`. These retained `.rs` files are the complete probe definitions.

Python compile-driver heredocs executed the following rustc form, resolving each external library to the newest corresponding release `.rlib`:

```bash
taskset -c 0-7 nice rustc --edition=2024 target/spin-review-NAME.rs   -C linker=zigcc -L dependency=target/release/deps   -o target/spin-review-NAME --extern CRATE=target/release/deps/libCRATE-HASH.rlib ...
```

Compile invocations, in order: probe initially failed because `bb_world` was omitted; retry adding it failed because direct rustc defaulted to unavailable `cc` (the subsequent attempted run exited 127); retry with `linker=zigcc` and redirected caches succeeded; probe and old-games then compiled successfully; final probe, contact-probe and tee-zero compiled successfully. Probe/tee-zero externs: `bb_game bb_agents bb_math bb_observe bb_physics bb_world`; old-games: `bb_game bb_stats clap`; contact-probe: `bb_math bb_physics serde_json`. The six-point helper emitted only unused-code warnings. No two heavy jobs ran together.

Additional Python heredoc commands, all exit 0:

- `python3 -B - <<'PY'`: regenerate all six `build_corpus()` values in memory, serialize exactly as `gen_golden.py`, compare bytes and print SHA-256; write only the new regenerated fixture under `sim/target/`. All six byte-identical.
- `python3 -B - <<'PY'`: compare constant-0.40 versus law on the five Python inputs, printing normal speed, EV and total energy. Demonstrated that EV response is geometry-dependent and total energy changes.
- `python3 -B - <<'PY'`: independently calculate linear/angular momentum and energy before/after those five collisions. Residuals and energy losses reported above.
- `python3 -B - <<'PY'`: sum passed-test counts from `spin-review-tests.log` and search for failed results. 338, none.
- `python3 -B - <<'PY'`: symlink twelve cached `d_2025-*.csv` inputs as `sim/target/spin-review-savant/sc_*.csv` for the existing scripts. Twelve files, originals untouched.
- `python3 -B - <<'PY'`: compare after game logs excluding their last timing line and compare foul logs bytewise. All match.
- `python3 -B - <<'PY'`: progressively compare completed refit outputs against existing files and print run/EV summaries (four invocations while the sequential batch ran, plus final comparisons). Every completed file matched; final comparison matched all seven.
- `python3 -B - <<'PY'`: final comparison of both before logs excluding timing and all seven refit logs bytewise. All match.
- `python3 -B - <<'PY'`: write this review file. Final file/status checks are recorded below.

### Target data and source inspection commands

| Command | Result |
|---|---|
| `python3 -I analysis/statcast/savant_collision_targets.py /tmp/claude-1000/-home-natex-Projects-3D-Baseball--claude-worktrees-swing/9c3ddb4a-fd05-4c61-8eb1-40e9f64690a7/scratchpad/savant > sim/target/spin-review-statcast-six.txt` | Exit 0; exploratory six-date sample, not used as twelve-date reproduction. |
| `python3 -I analysis/statcast/savant_square_contact_spray.py` with that same six-date directory, redirected to `sim/target/spin-review-square-six.txt` | Exit 0; exploratory sample. |
| `python3 -I analysis/statcast/savant_collision_targets.py sim/target/spin-review-savant > sim/target/spin-review-statcast.txt` | Exit 0; twelve-date note reproduced. |
| `python3 -I analysis/statcast/savant_slope_by_tilt.py sim/target/spin-review-savant > sim/target/spin-review-tilt.txt` | Exit 0; −0.183 ±0.058 reproduced. |
| `python3 -I analysis/statcast/savant_square_contact_spray.py sim/target/spin-review-savant > sim/target/spin-review-square.txt` | Exit 0; square-contact figures reproduced. |
| `curl -L --fail --max-time 30 -o target/spin-review-carry.pdf https://baseball.physics.illinois.edu/carry-v2.pdf` (`sim/`) | Exit 6, sandbox DNS unavailable. Used web text and pre-existing rendered pages instead. |

Web tool calls: opened `https://arxiv.org/pdf/1610.03464` and `https://baseball.physics.illinois.edu/carry-v2.pdf`; found `0.405` and `179` in KNS; requested Nathan zero-index pages 7 and 2 twice, and KNS page 16 once. Text extraction succeeded; KNS screenshot returned cache miss and Nathan screenshot calls did not expose usable images through the wrapper. `view_image` then successfully inspected cached Nathan `carry/p-3.png`, `p-7.png`, `p-8.png` under `/tmp/claude-1000/-home-natex-Projects-3D-Baseball--claude-worktrees-swing/2403d156-9942-4747-84ea-1cd2900e3a2a/scratchpad/`. The PDF skill was read using `cat /home/natex/.codex/plugins/cache/openai-primary-runtime/pdf/26.1112.11217/skills/pdf/SKILL.md`; read-only workflow only.

### Repository and read-only command inventory

The following records all remaining inspection commands. To keep repeated file reads legible, `sed FILE: RANGES` denotes the executed `sed -n 'RANGEp' FILE` commands; multiple ranges were separate reads. Paths below use `P=sim/crates/bb_physics`, `G=sim/crates/bb_game`, `A=sim/crates/bb_agents`, `O=analysis/oracle`, `N=docs/calibration/2026-11-14-collision-sloped-barrel.md`, `L=docs/metrics-ledger.md`, and `T=sim/target`. These are notation, not environment changes. Inspections succeeded except the explicitly listed missing/inaccessible paths; `rg` with no matches returned 1 where appropriate.

Git commands:

```text
git status --short                      (initially clean; repeated during work)
git branch --show-current               (work/collision)
git log --oneline f710d9f^..be66dab       (12 commits, including requested target commits)
git diff f710d9f^..be66dab --stat         (20 files, 1667 insertions, 34 deletions)
git diff f710d9f^..be66dab -- P/src/bat.rs P/src/contact.rs O/bbphys.py P/tests/collision_golden.rs G/tests/batted_spin.rs
git diff f710d9f^..be66dab -- O/gen_golden.py O/test_oracle.py
git diff f710d9f^..be66dab --check         (clean)
git diff f710d9f^..be66dab -- sim/crates/bb_agents sim/crates/bb_world (empty)
git diff f710d9f^..be66dab -- sim/crates | rg '^\+.*(random|rng|HashMap|HashSet|mul_add|\.sin\(|\.cos\(|\.atan2\(|\.hypot\(|\.exp\(|\.powf\()'
git diff 4254e73 f710d9f^ -- sim/crates --stat (misplaced --stat printed the diff)
git diff --stat 4254e73 f710d9f^ -- sim/crates (only three diagnostic examples)
git diff --name-only f710d9f^..be66dab
git status --short --untracked-files=normal (clean before writing review)
```

Full-file/numbered reads:

```text
cat CLAUDE.md
cat P/src/collision.rs; cat P/src/contact.rs
cat G/examples/collision_anatomy.rs; cat G/examples/run_env_diag.rs; cat G/examples/foul_diag.rs
nl -ba N | tail -n 140; nl -ba L
nl -ba G/tests/batted_spin.rs | tail -n 25
nl -ba P/tests/collision_golden.rs | sed -n '224,300p'
nl -ba N | sed -n '225,261p'
```

Ranged reads:

```text
sed docs/physics-spec.md: 310,445; 268,291; 230,257
sed L: 39,135
sed O/gen_golden.py: 1,130; 125,180
sed O/bbphys.py: 300,430
sed O/test_oracle.py: 227,270
sed P/src/collision.rs: 1,225; 60,125; 138,190
sed P/src/contact.rs: 140,230; 145,228; 16,46
sed P/src/bat.rs: 198,230
sed P/tests/collision_golden.rs: 1,78
sed G/examples/run_env_diag.rs: 1,105; 1,100; 290,302; 410,437
sed G/examples/collision_anatomy.rs: 200,245; 80,123; 1,80
sed G/examples/foul_diag.rs: 112,150; 168,218; 265,320
sed G/src/pa.rs: 270,311; 43,65; 472,487; 507,523
sed G/src/game.rs: 620,672; 592,617
sed A/src/motor.rs: 65,96
sed A/src/swing.rs: 450,466
sed docs/agent-models.md: 348,359; 335,347
sed sim/crates/bb_cli/src/main.rs: 1,110
sed sim/crates/bb_cli/src/game_lab.rs: 1,128
sed analysis/statcast/README.md: 1,85
sed analysis/statcast/savant_square_contact_spray.py: 1,85
sed docs/calibration/2026-11-14-collision-targets.md: 1,80
sed sim/out/et/env_BB_K_SWING_1.8.txt: 1,80
sed sim/out/et/law-seed0.txt: 1,60
sed sim/out/et/foul_law.txt: 1,75
sed T/spin-review-square-six.txt: 1,24
sed T/spin-review-square.txt: 1,20
sed T/spin-review-statcast.txt: 1,55
sed T/spin-review-tilt.txt: 1,55
sed T/spin-review-foul-law.txt: 74,84
sed T/spin-review-foul-old.txt: 75,85
head -n 22 O/bbphys.py; head -n 22 O/test_oracle.py
```

Search commands (the patterns and scopes used):

```text
rg --files -g AGENTS.md -g '*physics-spec*'
rg -n '8.2.1|A10|A14|Kensrud|KNS|item 8|D20|Tier B|noise' docs/physics-spec.md PLAN.md
rg -n '8.2|e_t|A10|A14|Kensrud|Nathan|direction' docs/physics-spec.md
rg -n 'tangential|e_t|spin|direction gap' L
rg -n 'effort|k_swing|swing|timing' A/src/motor.rs G/src/pa.rs
rg -n 'http|179|65|2300|2,300|Trackman' N docs/calibration/2026-11-14-collision-targets.md
rg -n 'tangential|kns' O/test_oracle.py
rg -n 'D20|Tier B' PLAN.md
rg -n 'BB_K_|args|seed|digest' G/examples/run_env_diag.rs sim/crates/bb_cli/src/main.rs
rg -n '\be_t\b|0\.40|0\.405' sim/crates --glob '*.rs' | head -n 90
rg -n 'BB_K_|seed|digest|fn main' G/examples/run_env_diag.rs sim/crates/bb_cli/src/main.rs
rg -n 'digest|fn games|Command::Games|Games' sim/crates/bb_cli/src
rg -n 'digest|serialize' G/src/game.rs sim/crates/bb_schema/src/*.rs | head -n 35
rg -n 'mul_add|\.sin\(|\.cos\(|\.exp\(|\.powf\(|\.atan2\(|\.hypot\(' P/src/bat.rs P/src/contact.rs (none)
rg -n 'velocity.*effort|effort.*velocity|effort' A/src/swing.rs | head -n 15
rg -n 'energy|carry|spin|drag|counterfactual|HR' G/examples/run_env_diag.rs | tail -n 30
rg -n 'HR|carry|effort|swing' N | tail -n 25
rg -n '^\s*e_t:|\.contact\.e_t|params\.e_t' sim/crates --glob '*.rs'
rg -n 'simulate_game_fielders|with_population|pub fn finish|pub fn run|pub fn result|fn finish|fn into_result|pub fn new' G/src/game.rs
rg -n 'Part D|fitted slope|3 cm|^  3|^   3|^ 3' T/spin-review-anatomy.txt | head -n 15
rg -n 'swings .*contact|K%|40 to 50|50 to 60' T/spin-review-foul-law.txt T/spin-review-foul-old.txt
rg -n 'swings .*contact|40 to 50|50 to 60|fits over' T/spin-review-foul-old.txt T/spin-review-foul-law.txt
rg -n 'max|Exit|EV ' sim/out/et/env_base.txt | head -n 12
rg -n 'Q2|Exit|p99|max' G/examples/run_env_diag.rs | head -n 15
rg -n 'Tee|131|0\.40|spec|speed' N | head -n 20
rg -n 'Knobs|calibrated_m3|knobs.swing|SwingNoise' G/src/pa.rs | head -n 15
rg -n 'EFFORT_SIGMA|BatControl|effort|within-batter' docs/agent-models.md | head -n 20
rg -n 'spray minus|spray −|minus attack|0-10|35-50' T/spin-review-statcast.txt | tail -n 20
rg -n 'cargo|rust|subprocess|target/|tangential' O/bbphys.py O/gen_golden.py
rg -n 'effort|effort σ' docs/agent-models.md | head -n 6
rg -n '^## 13|Tier|ability distributions|noise scales|fudge' PLAN.md
rg -n '0.75|0.51|0.24' L N | tail -n 7
```

Discovery and monitoring commands:

```text
rg --files sim/target /tmp | rg '(carry.*pdf|1610|kns.*pdf|carry.*png|savant.*csv)$' | head -n 25
  (some unrelated private /tmp directories returned permission denied; cached inputs found)
rg --files /tmp/claude-1000 | rg '(carry|kns|1610).*(pdf|png)$' | head -n 35
rg --files sim/target/data | head -n 20 (directory absent)
rg --files /tmp/claude-1000/-home-natex-Projects-3D-Baseball--claude-worktrees-swing | rg '\.pdf$' (none)
rg --files /tmp/claude-1000 | rg '/sc_2025.*csv$' | tail -n 18
ls sim/out/et
ls sim/target/release/deps/libbb_game*
ls -l T/spin-review-env-*.txt (repeated as sequential runs completed)
cat T/spin-review-probe.txt (also with head -n 6)
cat T/spin-review-cli-build.log
cat T/spin-review-games-seed0.txt
cat T/spin-review-games-seed1000.txt | head -n 5
cat T/spin-review-contact-probe.txt
cat T/spin-review-tee-zero.txt
cat T/spin-review-tilt.txt | tail -n 18
head -n 4 T/spin-review-old-seed0.txt
head -n 5 T/spin-review-old-seed0.txt; head -n 5 T/spin-review-old-seed1000.txt
head -n 4 T/spin-review-old-seed0.txt
sed -n '1,24p' T/spin-review-square-six.txt
rg -n '^Runs/team-game|^ALL |^Fair BIP' T/spin-review-env-*.txt
rg -n '^Runs/team-game|^ALL |^Fair BIP' T/spin-review-env-s18.txt
rg -n '^Runs/team-game|^ALL |^Fair BIP' T/spin-review-env-p35.txt
rg -n '^Runs/team-game|^ALL |^Fair BIP' T/spin-review-env-s18-p29.txt T/spin-review-env-p35.txt
rg -n 'Fair BIP:|Scorer Hit Types' T/spin-review-env-s20-p29.txt (not created yet at that monitoring point; exit 2)
rg -n 'max' T/spin-review-env-base.txt | head -n 5
tail -n 4 T/spin-review-env-base.txt
tail -n 2 T/spin-review-foul-old.txt (twice)
tail -n 3 T/spin-review-games-seed1000.txt
```

The test log was monitored with `tail -n` values 4, 5, 8, 10 and 12 (some repeated), and running tool sessions were collected with `write_stdin` after yielding. One `ps -eo pid,pcpu,comm,args | rg 'run_env_diag|cargo test|rustc|spin-review-old' | head -n 5` monitoring attempt exposed only the tool shell/process wrapper rather than useful host-job information; no conclusion relied on it. No job was terminated. Final checks: `git diff --check`, `git status --short`, and a Python assertion of this report's final Resolution and absence of other tracked edits.

## Resolution

Both findings fixed (2026-11-14, Opus 5.5 worker on `work/collision`).

- **P2-1, fixed.** The oracle gained `bbphys.bat_ball_contact`, which takes a §7 contact state (ball center, velocity and spin; the bat's knob, axis, center-of-mass velocity and angular velocity; the normal) and builds r_b = −R·n̂ and r_B = (b − R·n̂) − c itself (§7 step 5), applies the §8.2.1 law at the first impulse's normal speed and allows the §7 step 7 second impulse. The fixture `tangential_restitution.json` gained `contacts`: the five existing geometries restated as contact states, plus two general contacts inside the ramp, a bat sloped 28° down, translating, rotating about a tilted axis and rolling about its own, met under the ball's center on the barrel (|c_n| 50.1 m/s) and on the taper with the normal tilted by the profile slope (38.8 m/s), the ball spinning about all three axes. The other six fixtures regenerate byte-identical. A Python test checks the contacts independently (the restated five equal the body-level results to 1e-12; normal restitution at the contact point; the ramp value). New Rust test `a14_resolve_contact_matches_oracle` rebuilds each `ContactPoint` and calls the production `resolve_contact` with e_n held at the fixture's 0.45 (no location falloff or speed slope): worst relative error 1.9e-11 in ball velocity and 4.1e-12 in spin, bat speed and attack angle to 1e-9, four contacts in the ramp. Its counterfactual resolves the same contacts with the 34–60 m/s ramp and requires a miss of more than 1e-6: it misses by 1.2e-2 (velocity) and 4.6e-2 (spin). A mutation inside `resolve_contact` alone (e_t taken from the relative speed's magnitude instead of |c_n|, the standalone law untouched) fails the new test (5.4e-2), then was reverted. The standalone law checks stay. physics-spec §14.2 A14 now names the contact-level check.
- **P3-1, fixed.** Both tables in the collision note now give about 116 rpm/° for constant e_t = 0, from this review's six-point sweep (115.6), and the ×1/1.4 derivation is gone; a sentence under the §8.2 table says why scaling by Eq. (8) at fixed contact angle was wrong.

Not acted on, by design: the review's remark that the evidence supports fitting the spin anchors but does not establish a linear fall with speed is already stated in physics-spec §8.2.1 ("the shape between the anchors has no source").
