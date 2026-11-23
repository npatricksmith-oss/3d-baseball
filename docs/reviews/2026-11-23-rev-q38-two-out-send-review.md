# Review: Q38 (a), the two-out send from second (rev-q38, 2026-11-23)

Reviewer: rev-q38 (Claude Opus), independent of the change. Range `main..work/two-out-caution` (`f24debe` … `9b742ba`). Files: `bb_agents::running` (`AdvanceParams::send_two_out_z`, `for_decision`, test), `bb_game` `live.rs` (`runner_decide`, `runner_relook`, fixture), `game.rs` (`with_advance`), examples `runner_timing`/`runner_rates`; agent-models §6.3, the [two-out send note](../calibration/2026-11-23-two-out-send.md), the ledger row.

**Verdict.** The code is small, correct for what it says, and keeps the invariants. One parameter, a named band (Retrosheet 2024 two-out sends from second, by field) and a sourced behaviour (the coach's "send unless clearly beaten"). The pooled fit holds: 93.1% sent and 4.3% of sends out, against 92.1% and 3.9%. But the note's account of *why* runners were held, and so why the value is below zero, rests on a throw benchmark that is about 0.28 s slower than the throws the sim actually makes. The right-field split also says more than "noise". Neither blocks a merge. Both should be corrected in the note and carried as open items.

## Findings

### P2-1. The mechanism rests on a throw benchmark about 0.28 s slower than the sim's real throws, and right field's extra sends below z = 0.3 score no runs

*Where:* note §1 "Band and mechanism" and "Why the value is below zero", §2 field-split bullet; agent-models §6.3 "*Why below zero*"; `sim/crates/bb_game/examples/runner_timing.rs:147-180` (`throw_benchmark`).

*What:* The note says the corner runners held at the old caution "were all 0–0.27 s behind the ball alone, and the catcher's 0.25 s tag would have made most of them safe". "The ball" here is `runner_timing`'s benchmark: a clean `solve_throw` at the fielder's arm from the pickup point, with the receiver at the bag. It has no carry from the outfielder's charge and does not count a catch in front of the plate. On every two-out-relevant play with a direct throw home, the real throw beat the benchmark by about 0.28 s, all of it in the flight leg (the exchange agrees within 0.01 s):

| Field (start 2, direct throw home, `timing_z-0.2.csv`) | n | benchmark − actual ball, median (mean) | actual flight − benchmark flight, median |
|---|---|---|---|
| LF | 6 | 0.255 s (0.288) | −0.271 s |
| RF | 8 | 0.288 s (0.299) | −0.289 s |

The three close outs at right are seed 9 play 181, seed 1182 play 143 and seed 266 play 221. Against the benchmark they were 0.07, 0.05 and 0.02 s behind. Against the actual throw they were 0.34, 0.37 and 0.31 s behind, so they were beaten by the tag. The runners held at −0.2 had benchmark margins of −0.02 to 0.18 s behind, and the old caution's held runners −0.13 to 0.27 s. Against the throws the sim makes, most of them would also have been out at home. So "safe on most of them by the catcher's tag" and "every other part of the race was already in line" are not shown. That premise is what makes the caution, and not the race, the part found to be off.

The sweep's per-field breakdown points the same way (recounted from the worker's CSVs):

| z | RF sent | RF scored | RF out | LF sent | LF scored | LF out |
|---|---|---|---|---|---|---|
| 1.0 | 74 | 71 | 2 | 72 | 70 | 0 |
| 0.3 | 82 | 77 | 4 | 72 | 70 | 0 |
| 0.1 | 82 | 77 | 4 | 71 | 69 | 0 |
| −0.2 | 86 | 77 | 7 | 77 | 74 | 0 |
| −0.6 | 88 | 77 | 9 | 81 | 75 | 2 |

Below 0.3, every runner added at right ends as an out or a turn-back, and none scores. The run gain at −0.2 comes from left. All 7 corner outs are at right and none at left. With a shared out rate over 163 sends (86 at right) that split has p ≈ 0.01 one-sided, about 0.02 two-sided (hypergeometric). The note's "on 7 outs that is not distinguishable from noise" is not right. MLB has it the other way round (left 5.6%, right 2.2%).

*Why it matters:* The outcome fit stands: one parameter, matched to the pooled share sent and share out. But a future change that fixes the race (the benchmark, the outfielder's throw carry, or the runner's read of it) would be judged against a mechanism claim that is not true now. The flips described in Q3 below suggest the runner's own read at third may share part of this gap.

*Suggested fix (docs, plus a probe for the follow-up):*
1. Restate note §1 and the spec's "*Why below zero*" as an outcome fit of the share sent and the share out, not as "held runners would have been safe by the tag".
2. Correct the §2 field-split bullet with the p ≈ 0.02 split and the right-field table above.
3. Put the benchmark gap in TASKS as open. It also undercuts the runner-start note's "each leg" check, which used the same benchmark.
4. Probe, for the seven right-field outs, the runner's believed margin at third (`runner_decide`) against his margin at the throw's release (`runner_relook`). That shows whether his read of a charging outfielder's throw home is slow.

### P3-1. In the between-bases relook the two-out value also makes the way back to third look safer

*Where:* `sim/crates/bb_game/src/live.rs:8032-8034` (`adv = for_decision(start, next, …)`, then `adv.chooses_on(m_on, m_back, …)`), `sim/crates/bb_agents/src/running.rs` `chooses_on` (`caution_z` subtracted from both `p_on` and `p_back`).

*What:* The spec says the two-out value replaces `caution_z` when he "decides on home". In `runner_relook` between third and home, `chooses_on` takes the caution off the way on and the way back alike, so −0.2 also makes his chance of getting back to third look better. At third (`goes`, the way back certain) this cannot happen. Between bases it can make either choice more likely. It matters only when at least one base is better than even (otherwise the trapped branch decides on `p_safe`, with no caution), so it is rare.

*Fix:* Either let `chooses_on` take separate cautions for on and back, with the two-out value only on the way home, or say in §6.3 that between bases the value applies to both bases.

### P3-2. Keying the value to the starting base fits the data but is not the source's rule

*Where:* `running.rs` `for_decision` (`start == 2`); agent-models §6.3 "*Why only from second going home*".

*What:* The coach's two-out send is about whoever rounds third: the runner from first on a double with two out is the textbook case too. Keying on "started on second" leaves him out because the sim already sends him far too often (92.3% against 58.9%). That is a separate defect, so the key depends on it. "Started on second" also covers a runner from second who held at third and goes home on a later look from the bag (`base_relook` → `runner_decide`, for example on a throw to another base). The coach's pre-pitch decision does not describe that case, though going on it with two out is reasonable.

*Fix:* Mark the scope in §6.3 as interim and tie it to the open first-to-home-on-a-double gap, so that once that gap is fixed the value is tried for every runner rounding third with two out. No code change now.

### P3-3. The sweep supports a range, not −0.2 in particular

*Where:* note §2; spec "*Fit*".

*What:* The corners give 89.5% (z = 0.1), 93.1% (−0.2) and 93.7% (−0.3) sent, against 92.1% with a standard error of about ±2.0 points. The share out is 2.6%, 4.3% and 4.9%, on 4–8 outs, against 3.9%. Right field is identical at −0.2 and −0.3. Values from about 0.1 to −0.3 fit within noise, and −0.2 is a fair pick inside that range. It is not a fudge factor: the behaviour is sourced, and the value is an [E] parameter fitted to a named band (calibration-targets §2.3–§2.4).

*Fix:* Give the supported range in the note and in the spec line.

### P3-4. One test assertion sits on the edge of the break-even

*Where:* `running.rs` `the_two_out_send_from_second_takes_the_close_race`.

*What:* "The general caution holds him" holds by 1.4 points: Φ(0.75 − 1.0) = 0.401 against the 0.415 break-even. A refresh of the run table could flip it with no change in the rule. The `for (start, next)` loop restates `for_decision`'s condition. That is cheap, but it is not an independent check.

*Fix:* Use a race whose general-caution chance is clearly under 0.415, for example 0.05 s behind the ball alone, or say in the comment that the margin is thin.

## The questions

1. **Invariants: clean.**
   - The outs come from `runner_outs(k)`. That is the board (`st.outs`), less a catch whose call he has not yet seen: the same source `advance_values` already uses for the run values, so it is not world state hidden from him.
   - `for_decision` is a pure function of `(start, next, outs)`. `bb_agents/Cargo.toml` is unchanged, and the crate-graph test passes in the workspace run the worker logged.
   - There are no draws and no new `Purpose`. In `runner_relook` the changed line replaces an alias in place, after the draw, so the draw order is unchanged.
   - `Game` passes `self.advance`, which defaults to `AdvanceParams::default()`. Every other `LiveInput` builder uses `Default::default()`, so the standard game and the FFI run the same value.
2. **Scope: correct as specified.**
   - The two-out value is used only when `start == 2 && next == 4 && outs >= 2`: in `runner_decide` at third, in `base_relook` from third (through `runner_decide`), and in `runner_relook` between third and home.
   - First to third, first to home, the batter-runner, second to third and a runner from third keep `caution_z`. The unit test checks four of these.
   - The outs are the outs *now*. A second out made earlier in the play switches him to the two-out value, which matches the run values he weighs. A third out made elsewhere ends the play.
   - "Started on second" against "on third now": see P3-2.
3. **Would a real runner do this? Yes. The turn-back is not a fault this change makes.** I re-traced seeds 1268/139, 1026/13 and 1018/190 with the rebuilt `runner_timing` (raw 60 Hz traces).
   - In all three he goes at third at 2.77 s. He turns back 0.6–0.7 s later, at or just after the throw's release (3.35–3.48 s), 5–6 m past the bag.
   - On 1268/139, going on would have had him at home at about 6.15 s, against the ball at 5.63 s plus the 0.25 s tag: out by about 0.27 s (the note says about 0.2).
   - On 1018/190 the throw was cut off. He turned back, then went home again at 5.58 s from 12 m past third, and was out at home at 8.83 s.
   - A runner sent with two out who sees he is beaten by a quarter-second stopping and getting caught in a rundown is something real runners do. The outcome is the same out, so a "committed, never turns back" rule would change nothing here, and I would not add one.
   - The pattern itself, sent and then reversed as soon as he sees the throw, is the evidence for P2-1's probe: his read at third was more hopeful than the throw he then saw.
4. **Is the fit honest? Yes as an outcome fit; the stated mechanism is not shown.** See P2-1 and P3-3. The value below zero has a source (the coach's send), is an [E] parameter, and comes from a named band and a single-parameter sweep with no compensating pair.
5. **Tests: they pass and are mostly independent.**
   - The live fixture's race is independent of the runner's read. The fixture times it with `transfer + solve_throw(...)` (integrated flight). The runner reads it through `look.release_time` + `read_throw_time` (`direct_time`, his own approximation) and his arm read, which carries its own error. They share only the inputs (arm, transfer).
   - The thresholds were fitted to the measured counts with margins (≥170 against 187, ≥ old + 50, ≤60 against 41, ≤6 against 2). The old-against-new bound is a real check.
   - In the fixture the fielder is standing, so P2-1's charge carry does not apply. The fixture's "safe by the catcher's tag" refers to the benchmark race.
   - The unit test: see P3-4.

## Verification

- `touch` on `running.rs` and `live.rs`, then job 118 (`bin/job.sh`): `cargo test -p bb_game --lib two_out` gave 6 passed, including `with_two_out_the_runner_from_second_is_sent_unless_clearly_beaten`. `cargo build --release --example runner_timing` succeeded.
- `cargo test -p bb_agents --lib two_out`: `the_two_out_send_from_second_takes_the_close_race` passed.
- Raw traces: `runner_timing 1268 139 265`, `1026 13 1` and `1018 190 2` on the branch build (default −0.2), saved in the reviewer's scratchpad. They give the turn times and positions quoted in Q3.
- Recounts from the worker's `sim/target/codex-probes/q38/timing_*.csv`:
  - The per-field sent, scored and out table and the turn-back counts in P2-1.
  - The benchmark-against-actual gap by field: direct throws home, start 2, no cutoff in either.
  - The held runners' benchmark margins at 1.0 and −0.2.
  - The hypergeometric p for 7 of 7 outs at right (86 of 163 sends): 0.0101 one-sided.
- Read: the full diff `main..work/two-out-caution`, `runner_decide`, `runner_relook`, `base_relook`, `ball_time_to`, `read_throw_time`, `advance_values_after`, `runner_outs`, `AdvanceParams::{p_safe, goes, chooses_on}`, and `runner_timing`'s `throw_benchmark` and `actual_delivery`.
- Not run: the full workspace test run and the standard run. The worker's logged run (549 passed) and the ledger row were taken as given. Their figures are consistent with the note's sweep.

## Worker follow-up (worker-1, 2026-11-23)

- **P2-1.** Note §1 and the spec's "*Why below zero*" now call it an outcome fit, with the benchmark gap stated. The §2 field-split bullet now carries the right-field table and p ≈ 0.02. The probe (temporary hooks in `runner_decide` and `runner_relook`, which only print, on the same 600 games; note §2a) found that his read at third of the throw home is about 0.21 s hopeful (median; 10 direct throws), all of it in the flight; his read of the release is right. At the release he reads it right (−0.05 s). About 0.1 s is distance: he times the throw from where the fielder stands at his look, the fielder lets go 1.9–3.3 m nearer, and the catcher takes it 0.4–1.8 m out. The benchmark is 0.28 s slow. Both gaps are open in TASKS §1, and there is a correction line in the runner-start note under "each leg". The race model is not changed.
- **P3-1.** Fixed: `AdvanceParams::chooses_between`, with the two-out caution on the way home only, and the test `the_two_out_send_does_not_make_the_way_back_look_safer` (hand-worked from the run table). No measured rate moved: the CSV and both standard-run reports are identical.
- **P3-2.** §6.3 marks the scope as interim, tied to first to home on a double, and names the runner who goes home on a later look from the bag.
- **P3-3.** The range 0.1 to −0.3 is in the note, the spec line and the parameter's doc comment.
- **P3-4.** The case is now 0.15 s behind the ball alone (a 31% general-caution chance against the 41.5% break-even). The suggested 0.05 s behind would be 0.20 s in hand, a 50% chance, which goes even at the general caution. The scope loop is labelled as documentation.


## Resolution

Coordinator, 2026-11-23. All findings handled by worker-1 (`4514ac7`, `b621fe4`; Worker follow-up). P2-1: the note and spec now call −0.2 an outcome fit of the share sent and share out; the field split (all 7 outs at right, p ≈ 0.02) is stated; the probe found the runner's read at third of the throw home about 0.21 s hopeful, all in the flight, and that gap and the benchmark's 0.28 s are open in TASKS §1 (the race model is not changed here). P3-1 fixed (`chooses_between`; no rate moved, the reports are identical). P3-2, P3-3 documented; P3-4 test case moved clearly under break-even. No re-review: small, test-covered and with identical game output for the code fix; the Coordinator reran the tests and the standard run before merging.
