P1 findings: yes; do not merge.

Reviewed `80b54c1..cbbbebd` (`9c51b9f`, `86b1ddf`, `cbbbebd`), including the engine, scorer, accounting tests, five changed scenarios, rules/data-format specifications, and metrics entry. Read `CLAUDE.md` first and checked PLAN §§1 and 4. The later prompt-only commit `d1f6b5f` does not change the reviewed implementation. No implementation, test, spec, ledger, or TASKS files were edited.

## Rule and implementation assessment

The governing clause is **2026 OBR 9.02(g)**, printed p. 112 (PDF p. 124), not 9.02(a)(3), which specifies safe hits. It explicitly includes “a batter-runner whose batted ball results in another runner being retired for the third out”. **9.03(c)**, printed p. 113, supplies the box-score proof. Both are in the [2026 Official Baseball Rules](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf#page=124).

A home touch nullified by a force third out or the batter-runner's retirement before first belongs in LOB: there is neither a credited run nor that runner's putout. This follows from 9.02(g) and is required by 9.03(c), rather than remaining an assumption. Position between bases, a cancelled pending run, or a home touch after the effective third out does not eliminate a surviving runner. The proof includes interference/obstruction/shift-violation awards; these generated games have none, so their contribution is zero.

The ordinary implementation is sound: the live-play vector registers the batter and existing runners once; `third_out` excludes recorded scoring identities through `credited` and ordinarily retired identities through `out`. Consequently:

| Requested case | Review result |
|---|---|
| Batter short of first; another runner makes the third out | Included once; `lob-002` correctly expects 1. A batter who is himself retired is excluded. |
| Nullified home touch | Included; `lob-001` correctly expects 1. |
| Between bases; cancelled run after retouch | Included; `passing-023` and `lob-003` each correctly expect 1. |
| Held pass in `passing-025` | Correct: the legal earlier run is excluded and the late home toucher plus batter give LOB 2. |
| Two runners sharing a base; protected overrunner | Identity-based counting preserves both occupants and the overrunner; existing `twoonbase-008` and `overrun-007` pass. |
| Legal run on a time play | `credited` excludes it from LOB; the independent audit found no R/LOB duplication. |
| Outs after the third out | **Fails for multiple held passes: P1-1.** |
| Walk-off/game end | `GameEnded` retains the scorer's survivor-set fallback when no `HalfInningEnded` was emitted; completed halves are not added twice. All 67 walk-offs in the two audit sets balance. |
| Awards in progress | Inspected `award_first`, ground-rule, throw, and carried-catch award paths. Forced first-base awards finish before game end; other walk-off awards stop after the winner. Existing walk-off walk and ground-rule award scenarios pass. No additional defect found in these paths. |

The five changed expectations can therefore be independently derived from the rule and accounting, without using the engine's answers. Missing regression cases are multiple held passes crossing the third-out boundary and re-scoring pre-change logs. Explicit box-score LOB assertions for walk-off awards would also strengthen coverage.

## Findings

### P1-1 — Pending outs after the third out disappear from LOB

**Location:** `sim/crates/bb_rules/src/engine.rs:1099`, interacting with `:218` and `:232`.

`r.out` also means temporarily removed for an unadjudicated pass. `judge_pending_passes` takes all pending passes, then stops when the first one ends the inning. Later passers retain `out = true` even though their outs are ignored under rules-spec §6.3 and no `RunnerOut` is emitted for them. The new filter excludes those identities, breaking I10. Because `bb_stats` now trusts this scalar, its previous independent survivor accounting no longer protects the box score.

**Probe:** `sim/target/lob-review-probe/main.rs`, `edge` mode. Start with two outs, batter A.6, bases A.4/A.5/A.3. Before the ball is ruled, A.6 touches first; A.5 and A.4 reach third alongside A.3. A.4 attempts home at t=6 and passes A.5; A.5 attempts home at t=7 and passes A.3. The ball drops fair at t=8. The output records only A.4's out, then `HalfInningEnded { runs: 0, left_on_base: 2 }`. Correct LOB is **3**: A.3, A.5, and A.6. A.5's later out is ignored. The earlier probe with two passes established by one retreat also returned 2 instead of 3.

**Suggested fix:** Determine effective retired identities from the outs actually recorded through the third out (for example `lp.outs`), distinguishing them from pending-removal flags. Preserve ignored later passers as LOB. Add a scenario with two differently timed held passes and an independent I10/box-score assertion. The existing accounting test at `sim/crates/bb_stats/tests/accounting.rs:160` checks terminal-event disjointness but never compares the remaining identities with `HalfInningEnded.left_on_base`.

### P1-2 — Existing saved logs now produce incorrect box scores

**Location:** `sim/crates/bb_stats/src/lib.rs:226`.

Unconditionally trusting `HalfInningEnded.left_on_base` reintroduces the old engine's undercount when reading existing logs. This is a regression on committed fixtures, not a hypothetical compatibility concern. Both readers accept exactly the same bytes; only the new scorer's LOB changes:

| Fixture/team | LOB at `80b54c1` | LOB after change | New I1: left side versus right side |
|---|---:|---:|---:|
| `game_3.bblog`, home | 3 | 2 | 30 ≠ 29 |
| `game_7.bblog`, away | 8 | 5 | 37 ≠ 34 |
| `game_1001.bblog`, away | 7 | 4 | 39 ≠ 36 |
| `game_1001.bblog`, home | 7 | 5 | 37 ≠ 35 |

**Evidence:** `lob-review-probe fixtures .`, linked separately against the baseline and reviewed crates; outputs are `sim/target/lob-review-fixtures-{before,after}.txt`. Every fixture team balanced with the baseline reader. The existing hand-scored fixture test passes because it does not assert LOB or I1. Old/new logs have no distinguishing rules-version change; `eventlog.rs:63` leaves that producer field at its default.

**Suggested fix:** Preserve log-based survivor reconstruction for older logs, including the stranded batter, and add fixture I1/LOB checks. If newer logs are to use the engine scalar, identify that behavior explicitly and retain an old-log fallback. Follow `docs/data-formats.md:536`/`:542` when changing field semantics and behavior versions. PLAN §4.3 promises that scorer corrections can be backfilled without re-simulation.

### P3-1 — Scenario citations name the safe-hit clause

**Location:** `sim/crates/bb_rules/scenarios/passing/passing-023.toml:6` and `passing-025.toml:7`; related wording in `lob/lob-002.toml:4`.

The passing scenarios cite 9.02(a)(3) for LOB; the current clause is 9.02(g). `lob-002` also says the batter reached first although its facts deliberately leave him short of first. Its expected count is right because of the explicit batter-runner inclusion, not a physical first-base touch.

**Suggested fix:** Cite 9.02(g), with 9.03(c) for the proof, and describe the batter's explicit scoring inclusion accurately.

### P3-2 — Metrics seed labels do not reproduce the second CLI run

**Location:** `docs/metrics-ledger.md:250`; seed transformation at `sim/crates/bb_cli/src/game_lab.rs:105`.

The entry calls seeds 1000–1299 a standard `sim-cli games --seed 1000` set. That CLI actually uses seeds **1,000,003,000–1,000,003,299**. The reported 877/5325 half-inning comparison is reproducible with direct seeds 1000–1299, which this review audited.

**Suggested fix:** Distinguish direct simulation/audit seeds from the CLI's seed argument, and give the exact command for each measurement.

No separate P2 findings.

## Verification

- Ran `cargo test --release -p bb_rules -p bb_stats`: all passed, including the 132-scenario corpus and strict accounting tests. `accounting.rs:38` has an exact I1 assertion; it already had one at the baseline, with no tolerance to remove.
- Wrote an independent audit under `sim/target/`. From decoded log events it registers each completed batter's surviving trip and removes only actual `RunScored`/`RunnerOut` identities. It does not use the engine's LOB value or scorer result to derive its expected count. It compares every completed half, final walk-off survivors, per-team LOB, and strict I1.
- Audited **600 games**, direct seeds **0–299 and 1000–1299**: **10,728 completed halves**, zero independent LOB mismatches and zero team I1 mismatches in newly generated logs. This sampling does not cover the constructed held-pass failure.
- Ran the same audit against an archived `80b54c1` snapshot. Normalized hashes of every decoded event payload and box score, excluding LOB and the digest chain, matched for all 600 games. Digest changes: **291/300 games in each set**. First-set LOB changes: **917/5403 (16.97%)**, comprising +1 in 656 halves, +2 in 213, +3 in 48. Second set: **877/5325**, comprising +1 in 647, +2 in 186, +3 in 44. These exactly reproduce the direct-seed ledger counts. The frequency is credible: the old predicate required base contact at the third out, so runners moving on contact were systematically omitted. No play/run/out/hit movement was found in these samples.
- Built and ran `sim-cli games --n 300 --seed 0` for the reviewed source and baseline; gameplay/report lines match after excluding the inherently variable final timing line. Baseline CLI verification used an isolated target directory to avoid shared-cache reuse.
- Repeated seeds 0, 7, and 1000: complete log bytes and digests match on repeat. The changed code uses vectors and ordered survivor sets, introduces no hash iteration, RNG, physics, or agent-state access. Statistics remain pure functions of the log; the correctness failures are P1-1 and P1-2.
- Every Cargo build/test and simulation invocation used `taskset -c 0-7 nice`, `CARGO_BUILD_JOBS=2`, the requested PATH and Zig caches, with one heavy job at a time. Temporary probes, snapshots, binaries, and output are confined to `sim/target/`. No commits were made.

## Resolution

All four findings fixed on `work/lob-official`.

- **P1-1 fixed.** `third_out` now treats a runner as put out only if he has an out recorded through the third out (`lp.outs`), not by the held-pass removal flag `r.out`. New scenario `lob/lob-004` (two held passes at 6.0 and 7.0 across the third out, LOB 3 derived from 9.02(g)/9.03(c)); it failed before the fix with `left_on_base: 2` and passes after.
- **P1-2 fixed, by the lead's design.** `bb_stats` computes LOB itself from the events again (reached, `RunScored`, `RunnerOut`, plus the stranded batter-runner of 9.02(g)) and no longer reads `HalfInningEnded.left_on_base`, which stays in the log as the engine's claim. `Box` gained `lob_by_half` and `engine_lob_by_half`; `accounting.rs` asserts they agree half by half in newly generated games, and `committed_fixture_logs_balance_with_their_old_lob` asserts I1 and the old LOB (8 for `game_7` away, 7/7 for `game_1001`, 3 for `game_3` home) on the three committed fixtures. `data-formats.md` says readers recompute and old logs carry the undercount; the field's meaning did not change, so no schema or behavior version moves (the table at lines 536-542 requires a bump only for a changed meaning).
- **P3-1 fixed.** Citations now read OBR 9.02(g), with 9.03(c) for the proof, in scenarios, spec, comments and ledger; `lob-002` describes the batter as short of first, counted by the explicit batter-runner clause.
- **P3-2 fixed.** The ledger row gives the exact commands and separates direct game seeds from the CLI's `--seed` mapping.

Verification: `cargo test --workspace --release` passes; `scorer_audit 300 0` and `300 1000` (600 games) complete with I1 asserted on every team.
