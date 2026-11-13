# Codex rules review: first-base overruns, shared bases, and passing

Reviewed `638871c..worktree-agent-aed39d3914e6c38bd` (`ac6e4d5`, `25a56c1`, `0026dd0`) in `coord/review-fixes`, at HEAD `b2d102392bfd6d1d44a227743a6512a6681b05b4`. Read `CLAUDE.md` first. The scoped rules sources, harness, scenarios, and spec are unchanged between the range endpoint and HEAD; later simulator changes are present. Probes and baseline builds were written under `/tmp/codex-rules-review/`; this report is the only repository file written. Locations below refer to HEAD. Findings 1–4 concern the new implementation; finding 5 is explicitly a pre-existing integration gap.

## Findings

### 1. P2 — Preserve passing that occurs before fair/foul resolution

**Location:** `sim/crates/bb_rules/src/engine.rs:156`, `:164`, `:559`; `docs/rules-spec.md:221`.

Both detection paths discard passing while `fair == Pending`; neither `decide` nor `glove` subsequently adjudicates the discarded history. Checking later positions cannot recover an earlier crossing once runners restore their order or touch home. Passing must survive eventual fair/caught resolution, while an uncaught foul cancels it. [OBR 5.09(b)(9), 5.06(c)(5)](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf#page=60)

**Verified:** A touch-only probe starts with A.4 on second, A.5 on third, no outs. While a high fly is pending, A.4 touches third at `4.0`, leaves at `4.1`, and touches home at `7.5`; A.5 remains on third. The batter touches first at `4.2`. The ball lands fair beyond the infield at `8.0`; A.5 then scores at `11.5`. Settlement produces **zero outs and two runs**, instead of a passing out on A.4 and one run. No `RunnerPosition` integration is needed to expose this. A separate position probe crosses the runners at `5.0`, restores their order at `6.0`, and decides fair at `6.5`; that passing out also disappears.

**Suggested fix:** Retain provisional passing decisions and their original times while the ball is unresolved. Resolve the history on fair/caught determination and discard it on an uncaught foul, preserving third-out/run timing. Add fair and caught counterparts to `passing-013`, including a crossing that has ended before resolution.

### 2. P2 — A runner returning from a pending home touch must remain passable

**Location:** `sim/crates/bb_rules/src/engine.rs:108`, `:134`, `:168`, `:725`; `docs/rules-spec.md:220`; `sim/crates/bb_rules/scenarios/passing/passing-009.toml:16`.

`scored_at.is_some()` excludes the returning runner from both passing paths until he retouches his original base. A pending home touch does not exempt a returning runner from passing rules. [OBR 5.09(b)(9) and Comment](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf#page=60) The spec expressly describes this incorrect exemption; it is not listed as a deferral.

**Verified:** A.5 leaves third and touches home at `4.5`, before a catch at `5.0`. A.4 stays on second through the catch, then legally tags up, touches third at `8.5`, and leaves it. At `10.0`, simultaneous samples place returning A.5 at progress `3.4` and A.4 at `3.6`: A.4 has passed him, but there is no out. A.4 touches home at `11.0`; A.5 retouches third at `12.0` and scores again at `15.5`. Settlement produces **one out and two runs**, instead of two outs and one run. A.4 owes no retouch in this probe, so the result does not depend on deferred appeals.

`passing-009` also encodes an incorrect timing assumption: its comment excludes the returning runner, and its expected passing out waits until the post-retouch `7.6` samples. The crossing occurs during the return; the final out/run totals conceal the timing defect.

**Suggested fix:** Separate a pending run from the runner's physical participation in the play. Track returning positions and passing before retouch, while retaining the existing run-cancellation and appeal bookkeeping. Correct §5.5 and extend `passing-009` with samples during the crossing and a third-out variant.

### 3. P2 — Tags between first-base exit and the next sample bypass overrun protection

**Location:** `sim/crates/bb_rules/src/engine.rs:110`, `:114`, `:320`.

Protection starts only on an off-bag position sample. A chronological tag arriving after `BaseLeft` but before the next roughly 30 Hz sample sees `NotSeen` and immediately retires the runner. Immediate-return overrun protection applies during that interval too. [OBR 5.09(b)(4), Exception](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf#page=58)

**Verified:** On a fair grounder, the fielder secures the ball at `1.3`. The batter touches first and is sampled there at `4.200`, leaves at `4.210`, and is tagged at `4.220`, before the next regular sample at about `4.233`. The engine emits `RunnerOut { base: 2, kind: Tag }`. Inserting a straight-down-the-line sample at `4.215`, `(19.55, 19.55)`, makes the same tag safe. These represent the same immediate-return overrun with different sampling phases. Existing overrun scenarios all supply a protective sample before the tag.

**Suggested fix:** Define how contacts are adjudicated between samples. For example, require a current position sample at first-base exit/tag contacts and defer an ambiguous tag until that evidence is available, or introduce provisional protection with explicit attempt evidence. Add a tag-before-next-tick case. The documented absence of samples from today's orchestrator is a separate limitation; periodic samples alone do not close this gap.

### 4. P2 — One retreat can produce multiple passing outs, but only one is recorded

**Location:** `sim/crates/bb_rules/src/engine.rs:167`, `:178`, `:756`; the position loop also stops at `:145`.

`passed_on_touch` returns only the first pair, and its caller never searches again after recording that out. Each following runner who passes the leader is independently liable. [OBR 5.09(b)(9) and Comment](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf#page=60)

**Verified:** Start loaded, no outs: A.4 on first, A.5 on second, A.3 on third. On a fair ball, A.5 reaches third at `5.0`, A.4 reaches it at `5.1`, and A.3 returns there at `6.0`. Nobody has passed yet. A.3 leaves third and retreats to second, touching it at `9.0`, behind both followers still on third. That touch emits only `RunnerPassed { runner: A.4, passed: A.3 }`; A.5 survives and the out count is **one instead of two**. No further runner facts are required to establish the second pass. With one initial out, the missing second out would prevent the half from ending.

**Suggested fix:** Enumerate all established passing relationships in a stable order and process them until none remain or the inning ends. Reacquire live state after each out. Extend the lead-runner-action corpus with two followers, including a third-out case.

### 5. P2 — Shared-base occupants collapse to one left-on-base runner (pre-existing)

**Location:** `sim/crates/bb_rules/src/engine.rs:880`, `:881`, `:402`.

Third-out settlement writes each survivor into a single slot per base, then counts occupied slots. Two survivors sharing a base overwrite one another. Neither is automatically retired merely by sharing the base, and both stranded runners count toward LOB. [OBR 5.06(a)(2), 9.02(g)](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf#page=124)

**Verified:** With two outs, A.4 starts on second and A.5 on third. On a fair grounder, A.5 returns to third at `3.0`, A.4 arrives there at `3.4`, and the batter is tagged before first at `3.8` for the third out. Neither occupant is tagged or passes the other. The event is **`HalfInningEnded { left_on_base: 1 }` instead of 2**, losing a runner from the accounting identities. The same probe against rules at `638871c` reproduces it: this is not a regression introduced by the reviewed commits, but the newly supported shared-base behavior still exposes it.

**Suggested fix:** Calculate LOB from surviving runner identities before collapsing occupancy, and pass that count into half-inning settlement. Add this case beside `twoonbase-*`; an occupied-base count cannot represent it.

## Other checks

- `force-004` is correct: tagging a still-forced runner on his original base retires him by force. The forced/non-forced entitlement cases in `twoonbase-001` through `007` agree with the cited rules, including force removal and the caught-fly case. [OBR 5.09(b)(6), Force Play definition, 5.06(a)(2), 5.06(b)(2)](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf#page=58)
- The corpus already covers turning toward second and then being tagged (`overrun-002`, `006`), passing as a time play (`passing-003`, `004`), force removal (`011`), the comment's rundown (`005`), and an uncaught foul (`013`). Their expected results are consistent with their facts and documented geometric approximations. Finding 2 identifies the wrong timing assumption in `009`.
- An additional probe makes a passing out at home the third out: an earlier lead run counts, the passer's home touch scores nothing, and subsequent touches/position samples leave the new half unchanged. A separate probe confirms no overrun immunity before first has been reached.
- Equal-timestamp touches are order-sensitive exactly as §5.5 warns: delivering the preceding runner's third-base touch first yields no passing out; swapping the two equal-time touches yields one. The orchestrator must preserve that additional ordering requirement; ordinary “touches before tags” priority alone is insufficient.
- Event production uses runner vectors in stable order, no RNG, no unordered collection iteration, and no new transcendental math or fused operations. Projection arithmetic follows a fixed evaluation order. `RulesOptions` exposes the named [E] parameters with defaults `overrun_attempt_m = 1.5` and `pass_margin_m = 0.5` (`state.rs:252`). The probe replays are deterministic; sampling dependence in finding 3 is a judgment/input-contract gap.
- §5.5 largely describes the implemented mechanics, including the erroneous returning-runner exclusion. §15 explicitly defers serializing `RunnerPassed`: today's event log retains its `RunnerOut` as `Tag`, so it cannot distinguish the cause. It also states that live `bb_game` sends no `RunnerPosition` and currently stops runners on first. §5.5 explicitly defers immediate-return appeals under 5.09(c)(3) and passing during awards/home-run trots; the geometric attempt test is acknowledged as a stand-in in §15. These stated deferrals are not reported as implemented features.

## Verification

All commands ran in this checkout; Cargo commands below ran from `sim`. Zig caches were set to `/tmp/codex-rules-zig-global` and `/tmp/codex-rules-zig-local`.

| Command | Result |
| --- | --- |
| `cargo test --workspace --release -j 4` | Exit 0; 137 test functions passed, zero failed, including all workspace/doc-test groups. |
| `cargo test -p bb_rules --test scenarios --release -- --nocapture` | Exit 0; **94 scenarios, 0 failed**. |
| `cargo run --release -p bb_cli -- games --n 512 --seed 2` | Exit 0; **5.34 runs/team-game**, `rules warnings: {}`. The requested 5.23 calibration value is not reproduced at this later HEAD. |
| `bash /tmp/codex-rules-review/run.sh` | Compiles/runs the Rust fact probes against the release rules library, runs the baseline LOB probe, and repeats the 512-game comparison; all assertions pass, confirming the reported defective outputs and the successful checks. |

For isolation, exported `lib.rs`, `state.rs`, and `engine.rs` from `638871c` into `/tmp/codex-rules-review/baseline/`, compiled them as `bb_rules_baseline`, and compiled HEAD's unchanged `bb_game` sources against that library as `bb_game_baseline`. The comparison uses the CLI's exact seed sequence, `2 * 1_000_003 + i`, for `i = 0..511`, checking entire log byte streams as well as digests. **512/512 match**; all games complete, neither version warns, and HEAD emits zero `RunnerPassed` events. Both versions score 5,466 total runs, or **5.337890625 runs/team-game**. Thus the current calibration difference is not caused by this rules change.

Probe source: `/tmp/codex-rules-review/probes.rs`; traces: `probes.log`, `lob-baseline.log`, `compare-games.log`, and `verification.log` in that directory. Workspace/corpus/CLI logs are `/tmp/codex-rules-workspace-test.log`, `/tmp/codex-rules-scenarios.log`, and `/tmp/codex-rules-games.log`. No source, existing test/scenario, spec, or TASKS file was edited.

## Resolution

Open.
