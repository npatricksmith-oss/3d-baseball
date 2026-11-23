# Review: cutoff and relay hand-overs (`work/relay-handovers`)

Reviewer: Claude Opus 5.5, independent (wrote none of the change), 2026-11-22. Job: `REVIEW-relay-handovers.md`.

Change set `7622093..ea66e2b` (sim) and the docs commits after it to `c5eb654`: `sim/crates/bb_game/src/live.rs` (`relay_job`, `relay_base`, `deep_ball`, `cover_table`, `cover_after_chasers`), `docs/agent-models.md` §5.7, `sim/crates/bb_game/tests/throw_choices.rs` and 11 frozen fixtures in `tests/fixtures/a4/`, `docs/metrics-ledger.md` (row of 2026-11-22), `docs/calibration/2026-11-22-relay-handovers.md`.

**Verdict: mergeable as is.** No P1 or P2. Three P3s: two are text that says something the code does not do, and one is a pre-existing flip that this change does not touch. Invariants: nothing new reads world state that the fielders could not see (2), nothing touches `bb_agents` (3), and nothing adds a source of nondeterminism (5). A trace from my own build of the head's `live.rs` matches the worker's `tr-i1h.log` byte for byte.

## Findings

### P3-1. `held_home` holds while a runner is on the bases, not "while he is still running" as every text says

`sim/crates/bb_game/src/live.rs:5090` (`relay_base`): `held_home` is true while any runner who was on base at the pitch is not out and has not scored, whether he is moving or standing on a bag. The doc comment (`live.rs:5076-5081`), agent-models §5.7 (line 638, "keeps the throw home on a single while a runner who was on base at the pitch is still running"), the note's §1 row "1 (cost)" and the ledger row ("while a runner from base still runs") all say *still running*.

*How checked.* A probe copy of the head (`count` variant in `sim/target/review-probes/relay/variant.py`) printed every call of `relay_base` in which `held_home` alone made the base home, with the runners' state, over game seeds 0–119:
- 44 plays: 42 with runners on first and third at the pitch, 2 with runners on first and second, and none with a runner on first only;
- on 20 of the 44 the only runner left from base was standing on second (s = 54.9 m, speed 0) while the base was held home, for up to 2.1 s (seed 13 play 200: 3.95–5.63 s; seed 105 play 136: 3.88–5.56 s).

A second variant that does what the text says (`heldmoving`: held only while a runner from base moves faster than `moving_mps`) passes all 34 tests in `throw_choices.rs`. On the trace it is worse than the code as built, by `hc.py` against the head's trace:

| Runner in scoring position | Head (as built) | Text's rule |
|---|---|---|
| hand-overs at the pickup: air in the dirt / air shallow / ground | 8 / 1 / 4 | 12 / 4 / 7 |
| hand-overs, air in the dirt | 25 | 28 |
| third open, air in the dirt | 6 (1.4 s) | 8 (2.3 s) |

With the run in and the runner from first held at second, a real infield does not realign: the cutoff man stays where he is and the ball comes in to him. The code is right; the text is wrong.

*Fix.* Change the text in the four places to say "while a runner who was on base at the pitch is still on the bases (not out, not scored), moving or not", and give the measured reason (this table). No code change.

*Q1, answered.* The rule does reach a runner standing short of third, as above, and it should. It never applies on a deep ball (`!deep`), and the case the question names, a corner man out with a runner on first only, does not occur. With the runner counted at his pitch base, the only way the base becomes home with a runner on first only is `visible_target` with him heading home. If he then holds at third, he counts on third and the base is home anyway.

### P3-2. The note says all six tests fail on `7622093`; one passes there

`docs/calibration/2026-11-22-relay-handovers.md:148`: "each was run failing on `7622093` by the first worker". `a_corner_man_out_on_the_cutoff_home_keeps_it_when_the_lead_runner_scores` (13/227) **passes** on `7622093`. On main, the runner from first counted at the base he had reached, so the base stayed home. The test guards a cost of item 1 (`7413eb8`), not a fault on main. It fails only with `ea66e2b`'s `held_home` taken out ("the first baseman off the cutoff home at 3.42 s"). The test's own doc ("Fails without the rule") and the fixtures README ("fails on `7622093` or without its change") are right.

*How checked.* `7622093`'s tree with the head's `throw_choices.rs` and fixtures: 29 passed, 5 failed, and 13/227 is not among the 5.

*Fix.* Change the note's line to: five fail on `7622093`; 13/227 fails without `ea66e2b`; each fails with only its own change taken out (Verification, below).

### P3-3 (pre-existing, not from this change). A ground ball past the outfielders flips deep and back when the lead outfielder changes

Seed 118 play 162: runner on first, two out, a grounder through the right side that rolls past the right fielder. The centre fielder fields it 114 m out. At 4.11 s the second baseman goes out as the relay man home. At 4.26 s the lead changes from the right fielder to the centre fielder, the ball is no longer deep, and the shortstop goes back out on the cutoff to third while the second baseman returns to second. At 5.29 s the ball is deep again and the second baseman goes out again. The ground-ball branch of `deep_ball` (`live.rs`, the `Ball::Free` arm after the in-the-air branches) compares the ball with the *lead's* spot and position, so a change of lead can flip the call. Main does the same on this play, with the first baseman and second baseman each going out, back and out again (`tr-base.log`). It is the only such play in `gb 1st dirt` on the head's trace. It belongs with item 4 (who goes out when a single becomes a ball past the outfielders).

*Fix.* Defer to TASKS with item 4: a fixed mark for a ground ball, as already done for a ball first down in the shallow outfield (the spot of the outfielder nearest where it is first fielded or first passes one), or keep the first deep call once it is known.

## The note's eight questions

1. **`held_home` self-reinforcing.** See P3-1. It holds home on 44 plays in 120 games, never on a deep ball, never with a runner on first only. Ending it once the remaining runners stop costs more than it saves (P3-1's table). Keep it; fix the text.
2. **Runners from the world, `moving_mps`.** Acceptable. `relay_base` is orchestrator code in `bb_game`, and it reads only what any fielder sees: where a runner is and how he is moving, which the spec already treats as seen exactly for now (§5.7, "what a fielder knows of his teammates"). `visible_target` reads the same fields with the same parameter. One difference is deliberate and right: "stands on third" uses the signed forward speed (`direction * v <= moving`), so a runner going back to third after rounding it counts as on third, while `runner_target` uses |v|.
3. **Pitch base on a single.** I found no case where the base ends up behind the play. A runner from first standing on third after a misplay counts on third, so the base is home. A runner from first who passes third heading home raises it to home through `visible_target`. A runner from first stopped between second and third counts on first, so the base is third, and a runner from second always makes it home. The open item a real infield would do differently is the note's §5: with a runner on third, the base is home until he scores.
4. **`deep_ball` at the last `GloveSecured`.** Harmless. A `GloveSecured` after the first secure is a caught throw (a throw, `Ball::Thrown`, has already ended the relay job) or a ball taken again. Either way the holder is judged where he took it, which is where he stands at that moment: the same answer the old code gave then, with no flip as he steps in. `first_down_pt` stops at the first `GloveSecured` as before.
5. **`relay_out` held while an outfielder has the ball.** Harmless. While he holds it, the relay man is the man his throw goes through. If he runs it in, `relay_job` ends at `RELAY_FROM_M` from where he is, as before. I found no case where the relay man ought to give way to anyone while the outfielder holds the ball.
6. **Item 6's doubled bag.** Acceptable as argued. The gate asks the corner man's own estimate ("no other infielder can field it in the infield"), which his teammates cannot know, so making the convention ask the same gate would hand them information they do not have. A bag held by two men for at most 0.82 s on 3 plays costs less than the open base it replaces (12/9, 69/259, 95/176, 69/282).
7. **`two_to_second`.** Correct. `o == 3` is explicit, and `self.f[3].role` is `seen(3, 3)` for him. It cannot fire together with `two_claims_first` (which needs `!ss_busy`, while `two_to_second` needs `ss_busy` or `relay(5)`, and `relay(5)` implies `ss_busy`) or with `one_back` (which needs `!relay(2)`).
8. **The pitcher third in the re-call table.** It does not take him off backing up home at the throw. Over game seeds 0–119 with a runner in scoring position, the pitcher's job at the first throw on `e99e28d` and on `6efe905` (item 5 alone) is the same: 3 plays on third and 859 backing up. Home's order (catcher, pitcher) is unchanged, and the change only makes the re-call's table match the one used between calls.

## Questions 9–12

9. **Tests.** All six check against the standard alignment as the spec states it: the first baseman stays at first with a runner on first, the man already in line keeps the job, second is covered once both are out, third is covered, and nobody leaves the line before the throw. None restates the implementation. Each fails with only its own change taken out (Verification), and five of the six fail on `7622093` (P3-2).
10. **Spec against code.** Agent-models §5.7 matches the code except for P3-1. No new [E] literal: the only threshold `relay_base` adds is `ChoiceParams::moving_mps`, an existing named parameter. The tests' `0.3 + 0.02` is `CALL_HEARD_S` plus a tick, documented in the test.
11. **Figures.** Every cell of §2's main, "items 1–2" and final columns that I compared (all three tables and the nobody-on line) matches `hc-i1h.txt` / `hc-i2e.txt`. §2 item 3's figures (deep first open 10.7 → 12.1 s, second open 3.9 → 2.2 s) match `hc-i3.txt`. Every figure in §6's table and the ledger row matches `std3-{main,head}-{0,1000}.txt`. `std3.log` confirms the head binary is byte-identical to `sim-cli-final2`. The head's own `std3` binary was built at `68bfb23`, a docs commit after `ea66e2b` with no sim change.
12. **§5 leftovers.** None blocks the merge. Item 4 (the double cut) is the owner's question. The single with a runner on third is pre-existing standard-alignment work. Item 6's doubled bag is the cheaper error (Q6). On 93/183 the pitcher follows the third baseman. Item 1's cost, second open with a runner on first on ground balls (6 → 12 plays, 1.1 → 2.8 s): I looked at 2/77 and 118/162. Each is a ball rolling past the outfielders (fielded 114–119 m out at 7.8–8.6 s) and the 0.3 s swap the note describes, with no play on any runner at second in that time. A real infield would not swap, but this is item 4, not a new fault.

## Verification

All builds were in `sim/target/review-probes/relay/` (sources `main-src`, `var-src`; targets `tgt-main`, `tgt-var`), run through the job queue (jobs 13, 16, 17).

- **`7622093` with the head's `throw_choices.rs` and fixtures** (`t-main.log`): 29 passed, 5 failed: 12/9 (first open at 0.95 s), 6/6 (a cutoff home at 3.74 s), 13/224 (second open at 1.59 s), 23/94 (the second baseman off the relay at 3.78 s), 107/40 (third open at 1.40 s). 13/227 passes (P3-2).
- **Head with one change taken out at a time** (`variant.py`, `t-<variant>.log`). Each fails exactly its own test and passes the other 33:
  - `pitchbase` (count each runner at the base he has reached): 6/6.
  - `noheld` (drop `held_home`): 13/227, the first baseman off the cutoff home at 3.42 s.
  - `noglove` (judge a held ball where the outfielder stands now): 63/211, the shortstop off the relay at 2.85 s.
  - `noheldrelay` (`relay_out` only while the ball is loose): 23/94.
  - `notwo` (drop `two_to_second`): 13/224.
  - `nothird` (third's re-call order without the pitcher): 107/40.
  - `nogate` (drop item 6's gate): 12/9.
  - The head itself: 34 passed. The `heldmoving` variant: 34 passed.
- **Trace probe** (`zz_cut_home 120 0`, `BB_COVER_TRACE=1`). My build of the head gives `tr-head.log`, byte-identical to the worker's `tr-i1h.log`. `tr-heldmoving.log` and the `count` probe (`count.log`, `an_held.py`) give P3-1's figures. Item 5's effect on the pitcher at the throw: `q8.py` (in `review-probes/relay/`), run over `tr-i3.log`, `tr-i5.log` and `tr-i1h.log`.
- **Figures** (Q11): `hc.py` on the worker's traces, read against note §2. The `std3-*.txt` files were read against note §6 and the ledger row.
- **`cargo test --workspace`** on a copy of the head tree (`var-src`, with `analysis/`): 523 passed, 0 failed (`ws-test.log`), the same count as the worker's `test-head.log`.

## Resolution

Resolved 2026-11-22 (Coordinator). P3-1 and P3-2 (text only) fixed by the relay worker in `e910b28`: the `relay_base` doc comment, agent-models §5.7, the note and the ledger row now say the throw is held home while a runner from base is still on the bases, moving or not, with this review's measured reason; the note says five tests fail on `7622093` and 13/227 fails without `ea66e2b`. No re-review (comment and docs only). P3-3 (pre-existing: the deep call on a ground ball flips when the lead outfielder changes, 118/162) is deferred to the double-cut job (owner Q35, option (c)), in TASKS.md §3.
