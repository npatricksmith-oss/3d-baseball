# Independent review: swing-noise split and refit

Reviewed `7ace606..375e3f9` on `work/collision`: `08b167c`, `f452845`, `dfeb54a`, and `375e3f9`. Read `CLAUDE.md` first and applied its invariants, PLAN §13, and calibration-targets §2.3–§2.4. Compared replay behavior with an archived, unmodified `be66dab` source tree. No source, tests, specifications, ledger, or TASKS were edited; temporary sources, builds, probes, and logs are under `sim/target/refit-review-*`. No commits made.

No P1 finding. The motor split, sampled replay contract, adopted standard-run numbers, and spin-review fixes pass verification. The findings concern the interpretation and completeness of the calibration record.

## Findings

### P2-1 — Matching hard-hit rate does not establish that the remaining HR deficit is in angle and direction

**Location:** `docs/calibration/2026-11-14-collision-sloped-barrel.md:348`; the diagnosis is also summarized in `docs/metrics-ledger.md:296`.

The note concludes that because hard-hit rate is now right, the remaining deficit is in the angle and direction of the hardest-hit balls. The reproduced diagnostics leave an exit-speed-tail deficit open:

| Fair batted-ball measure | Before | Adopted | Cached twelve-date Savant 2025 sample, recounted |
|---|---:|---:|---:|
| EV ≥95 mph | 41.7% | 38.2% | 40.74% |
| EV ≥105 mph | 6.3% | 5.5% | 10.23% |
| EV ≥110 mph | 1.1% | 0.9% | 1.89% |

The real sample contains 9,165 tracked fair batted balls, including HRs. It is a sample rather than a season-matched causal experiment, but it directly prevents ruling out the speed tail from agreement at the 95 mph threshold. Agreement in maximum EV likewise does not establish agreement in the frequency of fast contact. The preceding swing-recalibration note already identifies this tail as relevant to HR production.

The measured fall in barrels is supported: 1,021/18,143 → 723/16,340, or 5.63% → 4.42%, with HR/barrel 47.8% → 47.0%. That describes the loss in joint EV/launch-angle quality. It does not separate the marginal EV distribution from launch angle; the barrel classifier does not use direction at all. The LA spread and known direction gap remain plausible contributors, with their relative contributions unresolved.

**Suggested fix:** retain the measured barrel explanation, record the deficient upper EV tail, and label the angle/direction attribution as a hypothesis. Compare joint EV/LA bins and conditional HR conversion against a matched real sample before assigning the residual deficit to a particular mechanism. This finding supplies no justification to change Tier A physics or widen the sourced effort spread.

### P3-1 — The 7.5% barrel comparator is an approximate reference, not a sourced 2024 measurement

**Location:** `docs/calibration/2026-11-14-collision-sloped-barrel.md:344`; `docs/metrics-ledger.md:296`.

The only repository basis found for 7.5% is `docs/calibration-targets.md:163`. That table uses `[R]` references, explicitly defined at line 10 as approximate values from general knowledge awaiting verification. The refit gives no season, sample, or primary citation for this comparator.

The official [Savant 2024 league Statcast table](https://baseballsavant.mlb.com/league?season=2024&view=statcast) reports 9,716 barrels in 124,203 batted balls, 7.8%. The cached twelve-date 2025 sample has 818 Statcast-labelled barrels in 9,165 tracked fair batted balls, 8.93%; that is a different sample and should remain identified as such. Both support a barrel deficit, but neither sources the specific 7.5% figure.

**Suggested fix:** identify 7.5% as the existing approximate band reference, or replace the asserted real comparator with a cited season/sample and explicit denominator. Keep the reference envelope separate from the measured MLB comparison.

### P3-2 — Record the additional plate-discipline envelope crossings exposed by the adopted setting

**Location:** `docs/calibration/2026-11-14-collision-sloped-barrel.md:344` and `:349`; `docs/metrics-ledger.md:150`.

The record calls out overall Contact% below its envelope, but omits two additional crossings visible in the same reproduced 300-game diagnostic:

| Measure | Before → adopted | calibration-targets envelope |
|---|---:|---:|
| O-Contact% | 47.4 → 42.8 | 45–68 (§5, B1.13) |
| Swinging-strike% | 14.8 → 16.1 | 8–15 (§5, B1.14) |

Other omitted context: Z-Contact falls 76.1 → 72.9 (already below 79–91); pitches/PA rises 2.97 → 3.11 (already below 3.5–4.3); called-strike share rises 17.1 → 19.0% of pitches. Swinging-strike movement is numerically present in the ledger, but its envelope crossing is not identified. The B1 bands originate in the PA lab; the report already applies their overall-contact band to these game diagnostics, so it should apply or qualify the submetric comparisons consistently.

**Suggested fix:** add these movements and envelope statuses to the calibration note and ledger's open issues, distinguishing newly crossed bands from existing shortfalls. This makes the adopted K% improvement and its remaining mechanism gaps reviewable without implying that the whole plate-discipline envelope has been reached.

## Other review results

**The relaxed timeline assertion is justified for this play.** `sim/crates/bb_ffi/src/lib.rs:1008–1016` originally required the ball to be more than 3 m from the original thrower 0.3 s after every announced release. That rejects legitimate short throws. The replacement accepts either spatial separation or possession by another fielder, preserving the stated departure/possession check. It does not verify the complete flight between those samples; the previous assertion did not establish flight continuity either.

The seed-5/play-201 trace shows the actual oddity. Fielder 131 (1B) initially chooses to carry to first. At t=4.024976 s, 0.9993 m from the bag, he switches to throwing to pitcher 129. Estimated arrival is 4.127142 s by carry versus 4.103705 s by throw; the corresponding option values are 0.9999927125 and 0.9999971371. Their difference exceeds the `1e-9` tie threshold at `live.rs:3042`, so the carry preference does not apply. Release occurs at 4.024976 s and the pitcher secures the ball and touches first at 4.058310 s. An intervening ball sample is in flight above glove height. This is an existing throw-choice model's sensitivity to a tiny difference between nearly certain outs, not a phantom timeline release or a newly introduced motor defect. A fielding follow-up can examine carry/throw timing and treatment of nearly equivalent options; rejecting all short transfers in the FFI test would not fix that choice.

**The split preserves the contract tested.** At shared scales 1.0 and 1.8, digests match `be66dab` for seeds 0, 1, 2, 7, 42, 299, and 1,000,003,000: 14/14 comparisons. The probe explicitly restores perception 2.7 and command 0.8 when comparing the split with the old knob. Adopted-setting repeats match for seeds 1, 2, 3, and 42. `motor::execute_swing` retains the same four placement/timing draws in the same order, followed by effort from its separate stream. The registry and RNG keys are unchanged.

`pa.rs:489–491` supplies attribute × `swing_miss` to `OwnExecution`, matching the vertical/timing sigmas constructed by `SwingNoise::new` at `motor.rs:82–89`. Effort does not enter the decision model. Decision functions receive beliefs and `OwnExecution`, not `Executed*`; retaining the executed placement while applying an adjusted plan remains orchestrator/motor application. The changed agent-models specification accompanies the code in `f452845` and matches this behavior. All four `Knobs::default()` fields remain 1.0. The diff introduces no outcome rolls, direct transcendental math in production logic, unordered-map iteration, RNG-source changes, or unnamed new [E] parameters.

**The adoption is Tier B noise tuning.** The production behavioral change is perception 2.7 → 2.9 and miss 1.0 → 1.4; command remains 0.8 and effort 1.0. The note names K/BB, fouls, hard-hit, and bat-speed-spread references and explains the mechanisms. Both changed scales increase noise, so this is not a compensating pair under §2.4. No collision parameter, population distribution, or outcome correction moved in this range.

Both standard seed sets reproduce the ledger. Selected additional context is shown below; brackets mean the CLI's `--seed 1000` set, whose actual game seeds start at 1,000,003,000.

| Metric | Before → adopted |
|---|---|
| Runs/team-game | 5.76 → 4.31 [5.78 → 4.68] |
| K%; BB% | 16.8 → 21.5 [16.7 → 21.5]; 7.7 → 7.9 [7.7 → 8.4] |
| HR/PA | 2.36% → 1.78% [2.24% → 1.77%] |
| Reached on BIP | 40.6% → 38.6% [40.5% → 39.1%] |
| GB/LD/FB/PU shares, seed-0 set | 46.3/23.0/24.1/6.6 → 49.5/20.1/22.4/8.1% |
| GB/LD/FB/PU hit rates, seed-0 set | .345/.763/.221/.035 → .322/.761/.212/.036 |
| Fouls/game; foul outs/game | 21.1 → 20.5 [21.4 → 21.2]; 1.79 → 1.57 [1.57 → 1.63] |
| PA/team-game; pitches/game | 40.2 → 38.8 [40.1 → 39.4]; 239 → 241 [237 → 245] |

All listed hit rates and both sets' type shares match the ledger. Foul share of contact rises even while fouls/game falls, because contact volume falls. Besides the plate-discipline measures in P3-2, foul-outs and provisional grounder FC share move: seed-0 GB FC .066 → .071. HBP remains 0.3% at printed precision; standard runs report no rules warnings. No unexplained aggregate mismatch was found.

**The effort explanation is supported within the tested counterfactual.** Re-running 100 games at perception 2.7 and miss 1.8, effort 1.0 → 1.8 gives HR/PA 1.78% → 2.95%, p99 EV 109.9 → 113.9 mph, maximum 117.4 → 128.0 mph, and HR mean bat speed 76.7 → 79.0 mph. The motor-level test measures within-batter SD 3.21 mph with widened misses alone versus 5.77 with both scales widened. Thus the old combined sweep obtained additional HRs by widening effort beyond the cited within-batter spread. This supports rejecting that calibration route; it does not establish the exclusive cause of the remaining deficit (P2-1).

**The spin review is resolved technically.** A14 constructs `ContactPoint`s from the Python-generated contact fixtures and calls production `resolve_contact`, with normal restitution held at the oracle input. The seven fixtures include sloped rotating barrel/taper contacts; four contacts lie inside the ramp. Python generates expected velocities/spins from the specified contact-state geometry and impulse equations, without reading Rust output. All six JSON fixtures regenerate byte-identically. Production errors are 1.91e-11 in relative ball velocity and 4.11e-12 in spin. A temporary contact-only mutation selecting a 34–60 ramp, leaving the standalone law unchanged, fails the new production-oracle assertion with errors 1.21e-2 and 4.56e-2. The six-point tee slope is 115.617539 rpm/degree at constant zero restitution, confirming the rounded 116 correction; the law gives 184.038670 over the same sweep.

## Verification

All cargo builds, tests, rustc probes, and simulations ran serially with `taskset -c 0-7 nice`. From `sim/`, the environment was:

```bash
export PATH=$HOME/.cargo/bin:$HOME/.local/bin:$PATH
export CARGO_BUILD_JOBS=2
export ZIG_GLOBAL_CACHE_DIR=$PWD/target/zig-global-cache
export ZIG_LOCAL_CACHE_DIR=$PWD/target/zig-local-cache
```

| Command / probe | Result |
|---|---|
| `taskset -c 0-7 nice cargo test --workspace --release` | 340 passed, 3 ignored, 0 failed; `target/refit-review-tests.log`. |
| `python3 -B -m unittest analysis/oracle/test_oracle.py` (repository root) | 23 passed. In-memory `gen_golden.build_corpus()` serialization compared against all six committed JSON files: all byte-identical. |
| `taskset -c 0-7 nice cargo build --release -p bb_cli -p bb_game --example run_env_diag`, then `taskset -c 0-7 nice cargo build --release -p bb_cli` | Both successful. |
| `taskset -c 0-7 nice target/release/sim-cli games --n 300 --seed 0`, repeated with `--seed 1000` | Adopted standard numbers above; `target/refit-review-std{0,1000}.txt`. |
| `taskset -c 0-7 nice target/release/examples/run_env_diag 300 0`, then `BB_K_MISS=1.0 BB_K_PERCEPTION=2.7` with the same command | After and before outputs byte-identical to `out/refit/env300_{after,before}.txt`; new evidence in `target/refit-review-env-{after,before}.txt`. |
| `python3 -B target/refit-review-batch2.py` | Preserved serial compile/run driver. Builds archived `be66dab` with `cargo build --offline --release --manifest-path target/refit-review-old/sim/Cargo.toml --target-dir target -p bb_game --lib`; runs old digest and unchanged CLI-report wrapper, A14/spread tests, contact-only mutation, tee probe, and the two 100-game effort counterfactuals. All expected results obtained; mutation intentionally exits 101. |
| `target/refit-review-digest-{new,old}` and `target/refit-review-toss` (each CPU-limited as above) | 14 shared-scale digest matches, four adopted repeat pairs; seed-5 throw-choice/trajectory evidence in `target/refit-review-toss.txt`. Probe sources retained beside binaries. |
| `python3 -I analysis/statcast/savant_ev.py sim/target/spin-review-savant` (repository root), plus direct CSV label/threshold count | Recounts twelve cached Savant dates; output `target/refit-review-real-ev.txt`, counts/dates in `target/refit-review-real-bbe.json`. Official 2024 comparator checked on the linked Savant league page. |

Archived standard-run logs are `target/refit-review-old-std{0,1000}.txt`. The effort counterfactual logs are `target/refit-review-env-{miss18,combined18}.txt`; the miss-only output also matches its recorded sweep byte-for-byte. Git inspection included the requested log, full diff, invariant-related searches, and `git diff --check`. Final status contains only this review outside ignored temporary artifacts.

## Resolution

Open.
