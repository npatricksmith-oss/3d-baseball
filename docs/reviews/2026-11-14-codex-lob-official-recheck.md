P1 findings: yes; do not merge.

Rechecked `61f9deb..c35f800` on `work/lob-official`, after reading `CLAUDE.md`, the first review and its Resolution, and PLAN’s invariants. The current head is `23f5729`; the later commits do not change the reviewed engine or scorer. All four original findings are fixed as described below. A newly discovered, pre-existing walk-off defect still violates the requested all-case LOB definition; it is retained by these fixes, not introduced by them.

## Earlier findings

### P1-1 — Confirmed fixed: ignored held-pass outs remain in LOB

`sim/crates/bb_rules/src/engine.rs:1101` excludes identities in `lp.outs`, rather than identities with the temporary `r.out` flag. `retire` appends to that list alongside `RunnerOut` (`:1054`); the third out ends the live play, stopping later pending retirements. All live-play third-out paths converge here. The separate caught-strike-three path emits its actual out and counts the remaining occupied bases. I found no other temporary flag excluding an identity from the engine’s third-out scalar. The base-position flags at `:1106` only populate the historical base slots after LOB has already been counted.

Reused the original `sim/target/lob-review-probe/` two-held-passes probe in the separately rebuilt recheck binary against the fixed crates: exactly one `RunnerOut` (A.4), then `HalfInningEnded { runs: 0, left_on_base: 3 }`. The three survivors are A.3, A.5 and A.6.

`lob/lob-004` is a valid regression expectation derived from OBR 9.02(g)/9.03(c): three existing runners plus the batter, less the one recorded out, less zero credited runs, equals three. I copied the unchanged committed scenario into an archived `cbbbebd` snapshot and ran its scenario harness: **133 scenarios, exactly one failure, `lob-004`, actual LOB 2 versus expected 3**. The current workspace’s scenario harness passes.

### P1-2 — Confirmed fixed: saved-log regression and independent scorer restored

`sim/crates/bb_stats/src/lib.rs:239` computes completed-half LOB from its own survivor set plus the stranded batter-runner (`:208`). `HalfInningEnded.left_on_base` is read only into the diagnostic `engine_lob_by_half` (`:243`), never used to calculate team LOB. Walk-offs use the scorer’s remaining identities at `GameEnded` (`:248`).

The three committed fixtures reproduce all six old-reader LOB values, and all six team lines balance:

| Fixture | LOB, away/home | I1, away/home: each side equals |
|---|---:|---:|
| `game_3.bblog` | 6 / 3 | 34 / 30 |
| `game_7.bblog` | 8 / 9 | 37 / 36 |
| `game_1001.bblog` | 7 / 7 | 39 / 37 |

The committed fixture regression test passes. The schema-version decision is reasonable: the field still claims official team LOB; its historical values were wrong, and readers now reconstruct rather than depend on those values.

The per-half cross-check is real. The engine uses its live-play identities, recorded outs and credited runs; the scorer reconstructs survivors from serialized events and its batter inclusion. My audit explicitly compared `lob_by_half` with `engine_lob_by_half`, in addition to the earlier independent event-based audit. Across **600 games, direct seeds 0–299 and 1000–1299, 10,728 completed halves**, there were zero cross-check failures, zero independent LOB mismatches and zero team I1 mismatches, including 67 walk-offs. As a mutation check, I encoded nine completed-half scenario logs with the engine scalar changed to **99**: scorer LOB stayed correct while the diagnostic engine value changed, so the equality would fail.

The earlier table’s constructed cases pass the scorer: nullified home touch (`lob-001`: 1), stranded batter (`lob-002`: 1), cancelled pending run (`lob-003`: 1), between bases (`passing-023`: 1), later ignored touch (`passing-024`: 1), held pass with an earlier credited run (`passing-025`: 2), two held passes (`lob-004`: 3), shared base (`twoonbase-008`: 2), and protected overrun (`overrun-007`: 1). Additional probes pass for a short-of-first walk-off single (`gameflow-007`: 2), walk-off walk (`gameflow-005`: 3), walk-off throw award (`awards-015`: 2), walk-off ground-rule award (`awards-014`: 2), ordinary walk (`awards-002`: 3), ground-rule award (`awards-020`: 2), and carried-catch award (`awards-019`: 1).

**The broader claim that the restored scorer implements the definition in every case is still false:** the additional pending-run walk-off below loses a runner without either `RunScored` or `RunnerOut`. The original saved-fixture regression is fixed; this newly discovered case is separate.

### P3-1 — Confirmed fixed: citations and stranded-batter wording

The changed passing/shared-base scenarios now cite 9.02(g), and `lob-002` correctly describes the batter as short of first and included by the explicit batter-runner clause. The spec cites 9.03(c) for the proof. Checked against the [2026 Official Baseball Rules, 9.02(g) and 9.03(c)](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf#page=124).

### P3-2 — Confirmed fixed: seed labels

`docs/metrics-ledger.md:250` now distinguishes direct game seeds 1000–1299 from the CLI argument `--seed 1000`, gives the standard CLI commands and direct-seed audit commands, and correctly identifies the CLI’s mapped seeds as 1,000,003,000–1,000,003,299. This agrees with `sim/crates/bb_cli/src/game_lab.rs:105`.

## New finding

### P1-3 — An uncredited pending walk-off home advance still removes a runner from LOB

**Location:** `sim/crates/bb_stats/src/lib.rs:177`, interacting with `sim/crates/bb_rules/src/engine.rs:1029` and the scorer’s `GameEnded` settlement at `sim/crates/bb_stats/src/lib.rs:249`.

The survivor reconstruction removes a runner on any `RunnerAdvanced` whose destination is home, even if no run is credited. `try_walk_off` iterates every pending home touch and emits that advance before calling `score_run`. Once the first touch supplies the winner, `score_run` suppresses later runs, but the later home advances are still emitted. The scorer consequently drops a runner who neither scored nor was put out.

**Reproduction:** bottom ninth, tied, runners on second and third. Both touch home while the ball is still in doubt; the batter reaches first. When the ball drops fair, only the lead runner’s winning run counts. The trailing runner and batter must give **LOB 2**, by 9.02(g). Actual events contain `RunnerAdvanced(lead, home)`, `RunScored(lead)`, `RunnerAdvanced(trailer, home)`, and `GameEnded`, with no `RunScored` or `RunnerOut` for the trailer. Actual scorer LOB is **1**.

I reproduced this first from a scenario fixture, then in a **complete scripted game from `StartGame`, with no manual state initialization**, using the actual rules engine’s events. Its 59-play serialized log passes the log reader and ends 2–3. The away line balances; the home line is **AB 28 + BB 1 = 29**, versus **R 3 + LOB 1 + opposing putouts 24 = 28**. Correct LOB 2 restores I1. The failing assertion and event trace are in `sim/target/lob-recheck-full-walkoff.txt`; the verified log is `sim/target/lob-recheck-full-walkoff.bblog`. Run the probe from `sim/` as `taskset -c 0-7 nice target/release/lob-recheck-probe fullwalkoff` (expected assertion failure).

This code path is also present in `6ef084a`; it is a newly discovered retained defect, not a gameplay regression caused by `c35f800`. The 600 sampled games do not encounter it. No `HalfInningEnded` is emitted for this final half, so its diagnostic engine entry is `None`; the valid completed-half cross-check cannot catch this walk-off error.

**Required correction:** a home advance alone must not remove an uncredited runner from scorer LOB. Preserve survivors until an actual `RunScored` or `RunnerOut`, and cover multiple pending home touches at a walk-off with an independent LOB/I1 regression. The engine should also avoid reporting advances beyond the winning run as though they completed.

## Verification

- `taskset -c 0-7 nice cargo test --workspace --release`: **348 passed, 0 failed, 3 existing ignored tests**; includes the scenario corpus and fixture/accounting tests. Output: `sim/target/lob-recheck-workspace-tests.txt`.
- Reused the original probe against current crates, ran its held-pass and fixture modes, and extended a separate probe under `sim/target/lob-recheck-probe/` for per-half cross-checks, scenario-to-log checks, scalar mutation, and the complete scripted walk-off. Sixteen existing scenario cases pass; the new pending walk-off intentionally fails. Outputs: `lob-recheck-edge.txt`, `lob-recheck-fixtures.txt`, `lob-recheck-cases2.txt`, and `lob-recheck-full-walkoff.txt`, all under `sim/target/`.
- Independent 600-game audit: 5403 completed halves in seeds 0–299 and 5325 in seeds 1000–1299; zero engine/scorer/independent mismatches and zero I1 failures. Outputs: `sim/target/lob-recheck-audit-{0,1000}-summary.txt` and the corresponding per-game/per-half files.
- Archived `cbbbebd` scenario test, with only the committed `lob-004` added in the temporary snapshot: expected failure, actual LOB 2. Output: `sim/target/lob-recheck-cbbbebd-scenarios.txt`.
- Built CLI binaries for the current source and archived **`6ef084a`**, using a separate baseline target directory and `--no-default-features` for both headless builds. Ran **`sim-cli games --n 300 --seed 0`** on each: all **12 report lines are byte-identical**, excluding only the final timing line. Normalized SHA-256: `65c5e03f619e96f877cc8a4e1d5f58fa5f35fcbe2b4bb0455d8d02ecccb8ca5f`. Outputs: `sim/target/lob-recheck-cli-{current,main}-0.txt`; `lob-recheck-cli-diff.txt` is empty. No additional gameplay/report regression found.
- Every Cargo build/test and simulation run used `taskset -c 0-7 nice`, `CARGO_BUILD_JOBS=2`, the requested PATH and Zig caches, and one heavy job at a time. Temporary snapshots/probes/build products are under `sim/target/`. No source, committed tests, specs, ledger, TASKS, or first-review files were changed; no commits were made. This report is the only added repository document.

## Resolution

P1-3 fixed; the four earlier findings were confirmed fixed by this recheck.

- **Scorer:** a `RunnerAdvanced` to home no longer removes a runner from the survivors; only `RunScored` or `RunnerOut` does (`bb_stats/src/lib.rs`). I checked the other uses of `RunnerAdvanced` in `scorer.rs` (batter award and base reached); they read the batter's bases, not survivors, and do not infer scoring or outs.
- **Engine:** `try_walk_off` stops after the winning run; later pending touches emit no advance (`bb_rules/src/engine.rs`). Spec §4.1 says so (OBR 5.08(b), 9.02(g)). Award walk-offs were already correct.
- **Tests:** scenario `lob/lob-005` (two pending touches, expects no `RunnerAdvanced` for the trailer; failed before); `bb_stats/tests/walkoff_lob.rs`: Codex's complete scripted game asserts LOB 2 and I1 on the home line (failed before: 29 against 28), and a second test asserts LOB for awards-014 (2), awards-015 (2), gameflow-005 (3) and gameflow-007 (2).
- **Measured:** `cargo test --workspace --release` passes; `scorer_audit 300 0` and `300 1000` pass; game digests of seeds 0–299 and 1000–1299 are identical before and after the fix, and `sim-cli games --n 300 --seed 0` is unchanged apart from the timing line.
