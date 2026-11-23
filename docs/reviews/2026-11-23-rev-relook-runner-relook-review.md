# Review: the base relook (`work/runner-relook`, `bda4b46..3fc8610`)

Reviewer: rev-relook (Claude Opus 5.5, high effort). I wrote none of the change. Scope: `runner_relook` / `base_relook`, the read of a throw at its release, the relay-man read, the quick return throw (`QUICK_RETURN_S`), the four new fixtures and the changed observation test in `sim/crates/bb_game/src/live.rs`, `live/runner_trace.rs`, agent-models §6.4, `examples/base_relook_probe.rs`, and the ledger row.

**Verdict.** The change keeps the invariants and replays deterministically. I found no path that lets a runner leave a base he owes a retouch, or that leaves two runners on one base. The probe counts reproduce to the digit. The six outs behind "6 of 14 put out" are model faults, not mistakes a real runner makes, and the ledger names the wrong cause for half of them. The three outs from first are a throw *to his next base* read as going to the base beyond it. The three from third are well-read throws to second that the man there relays home in 0.28–0.47 s, against the runner's 0.6 s belief. In four of the six, the out is completed by the old "trapped" turn, which sends a runner away from the nearer base. Separately, a pre-existing fault in `runner_decide` drops the tag from a forced runner's race past his forced base, and the base relook now runs that path in 62% of its looks.

## Findings

### P2-1. A throw to his next base read as going to the base beyond it sends the runner into the tag (all three outs from first)

`live.rs:7197` (`project_runner_throw`: `dest = near(q_down)`), `live.rs:497` (`pass_time`, `PASS_REACH_M` 2.5 m), reached from `base_relook` (`live.rs:7783`) at the instant of the release.

*Scenario.* A fly is caught, the runner on first retouches, and the outfielder throws to second. The base relook fires at the release, where the read's velocity error is at its largest (`shrink` = 1; the new release cap fixes only the position). His extrapolation carries the throw a little long or a few degrees wide. It comes down nearest third, and its track misses second's bag by more than 2.5 m, so he times second as third's arrival + `RELAY_READ_S` + a throw back: 4.1–4.9 s, against the true 1.8–3.4 s. He goes, and nothing makes him look again until the ball is caught at second. Belief dump (instrumented copy, below):

| Seed, play | Throw | Read dest | His ball at 2nd | True arrival at 2nd | His run | Read error |
|---|---|---|---|---|---|---|
| 24, 92 | RF (38.8, 69.7) → 2nd | 3 | 4.10 s | 1.96 s | 4.10 s | depth velocity −2.8σ |
| 1000003050, 173 | RF (33.3, 72.0) → 2nd | 3 | 3.84 s | 1.76 s | 4.10 s | depth velocity −2.9σ |
| 41, 194 | CF (13.4, 108.6) → man 5 m short of 2nd | 3 | 4.91 s | 3.38 s | 4.10 s | lateral/vertical 1.4σ |

The ball beat him by 0.9–2.5 s. A runner on first never misjudges by two seconds a throw coming straight at the bag in front of him. A throw from right field to second points toward the third-base side, so a modest overshoot names third. The flip is not a "3σ misread" (one case is 1.4σ), and `sigma_race_s` cannot absorb a 2-second jump in the believed time.

*Suggested fix* (one or more):
- From a base, take the throw look a read time into the flight, not at the release. A held runner has no reason to commit at the release, and the read improves quickly (`RUNNER_READ_TAU_S`).
- Or give the base runner a second look while the throw is in the air and he is still near his bag.
- Or count a throw as catchable at his next base when its read track passes within reach of the man visibly covering that base (`known_job` `Cover(b)`), not only within 2.5 m of the bag.

Add a fixture: runner on first, RF throw to second at average arm, decided across that test's error draws: he never goes.

### P2-2. A forced runner's race past his forced base drops the tag (pre-existing; the base relook now runs it in 62% of its looks)

`live.rs:7530`: `let tag_s = if forced { 0.0 } else { typical_tag_s }`, with `forced = runner_forced(k, k)`. That flag is true for a batter-runner always (`force_with`: `start == 0`), and for any runner whose bases behind are filled, whatever base he is deciding from.

*Scenario.* With runners on first and second, the batter-runner stands on first and the runner from first stands on second, both on bases they held. A throw is released and each takes a base look (`runner_decide(k, b)` with `b > start`). Each races the ball to the next base with no tag, although the force ended when he reached the base he was forced to (OBR 5.09(b)(6)). His belief is optimistic by `typical_tag_s` (0.25 s), more than one `sigma_race_s`. `must` already checks `at_base == r.start`, but `tag_s` does not.

The arrival decision (`live.rs:8095`) has had this fault all along. Instrumented copy, seed set 0: **3,045 of the 4,879 base looks** were by such runners (1,169 batter-runners on second, 1,137 runners from first on second, 280 on third, 229 runners from second on third, 106 batter-runners on first, 124 on third), and **12 of the 36 sends** came from them.

*Fix:* `forced && at_base == self.r[k].start`, which is a rules-alignment fix with no owner decision needed. Test: the batter-runner standing on first and a runner from first standing on second each race with the tag. With the fix (copy, `REV_FIX=1`): seed 0 sent 35, put out 4 (before 36, 4); seed 1000 sent 36, put out 6 (before 38, 6). The games diverge, so the counts move by noise. The fix is about the belief being wrong, not the counts.

### P2-3. The trapped runner turns toward the base the ball takes longer to reach, ignoring his own distance to each (pre-existing rule; it completes 4 of the 6 outs)

`live.rs:7748–7757`: `else if ball_back >= ball_on { from } else { next }` when he believes he is safe at neither base.

*Scenario, seed 41 play 194.* The runner from first is 6.5 m short of second at 7.7 m/s when the shortstop secures the throw 5.8 m from the bag. Second is a coin flip (ball ≈ 0.6 s + tag against his 0.85 s). First is hopeless (ball ≈ 2 s + tag, against ~3.8 s with the stop). The rule picks first because the ball takes longer to get there. He brakes, drifts on, turns, and the shortstop walks over and tags him 1.3 m from second.

*Seeds 225/150, 1000003051/211 and 1000003123/11.* The runner from third is 11–14 m from home when the throw home is released. Home is a bang-bang play: in 1000003051 the ball reached the plate 0.1 s before he would have, with the tag still to come. He turns back toward third, 16–19 m away, drifts to within 6–8 m of the plate under `BRAKE`, and is tagged going back to third 4 s later.

No real runner turns from a near base he might make to a far one he cannot. Without a rundown model, the least-bad choice is the base with the larger margin (ball + tag − his time), or the larger `p_safe`, not the larger ball time.

*Fix:* choose by margin in the trapped branch, and update the §6.4 "Looking again" sentence that states the old rule. Fixtures: the seed-41 geometry (he goes on to second) and the runner 11 m from home (he goes on).

### P2-4. The ledger row and the TASKS lead misdiagnose the failed sends

`docs/metrics-ledger.md` (newest §3 row, "Open"), `TASKS.md:40`, and the spec's line on the release read: "From first and third most sends that fail are 3σ misreads of a throw at its release (the race's `sigma_race_s` 0.2 s does not carry the read's own spread)."

*What the traces show.*
- **From first:** destination flips (P2-1) at 1.4–2.9σ that move the believed time by 1.5–2.5 s. That is a structural fault of the read, not a spread `sigma_race_s` should carry.
- **From third:** the reads were good. Read destination 2, correct every time, and believed arrival at second within 0.08–0.28 s. The believed total was 0.16–0.46 s slow, mostly from `RELAY_READ_S` 0.6 s against the world's 0.28, 0.47 and 0.47 s exchanges at second, with a +2.1σ timing draw in one. These are the "`RELAY_READ_S` 0.6 s vs the 0.27 s hurried return for a throw to another base" lead the row lists last. The sends were close calls a real runner can lose. The outs at third come from P2-3.

*Fix:* rewrite the Open cell and the TASKS item with the P2-1 to P2-3 causes, so the next job tunes the right thing (it is not `sigma_race_s`).

### P3-1. The quick-return fixture's "world" restates the runner's formula

`live.rs` test `a_runner_held_on_second_races_the_quick_return_throw` (line 16052): `hurried = RUSH_MIN_FRAC * RELAY_TRANSFER_FRAC * fd.a.transfer`. That is `QUICK_RETURN_S` computed from the same constants, and it leaves out the receiver's turn, which the world adds before the 0.6 (`SetUp::new`: `RUSH_MIN_FRAC × (base + turn)`). For the 32 m/s "he goes" case, a lower bound above his run is valid evidence. For the 37 m/s "he stays" case, `ball < mine` with a lower bound does not show the world beats him. The third-base counter test adds 0.5 s for this reason; this one does not.

*Fix:* take the receiver's real `SetUp` earliest (with its turn) as the world's quickest. For the stay case, assert against an upper bound, or run the play forward and observe the tag.

### P3-2. Smaller test and comment points

- `a_runner_held_on_third_scores_when_a_deep_throw_goes_to_second`: the comment says "even at the quickest exchange", but `quickest_home_with_tag` uses the set exchange `RELAY_TRANSFER_FRAC × transfer`, not the hurried release the world used on exactly this play in seed 1000003051 (0.28 s). Measured: his run 4.10 s; ball with the set exchange 4.72 s; with the hurried exchange (no turn) 4.54 s. The assertion still holds with 0.44 s to spare, so only the wording is wrong.
- `a_runner_reads_a_throw_to_the_relay_man_as_going_on_from_him`: the spec and the comment say "within 0.1 s of the world", but the test asserts `< 0.4`. Measured: −0.083, −0.073 and −0.073 s at 0, 0.5 and 1 s. Tighten it to 0.1.
- Missing cases:
  - a forced runner standing past his forced base (P2-2);
  - a base look while a catch call is pending (`awaiting_call`: no look, and `looked` left for later);
  - a runner who retouched after a caught fly and then takes a base look (all three outs from first were this path);
  - a throw to the runner's own next base from right field (P2-1).
- `runner_throw_observation` comment (`live.rs:7074`): "he cannot misplace it by more than it has gone since". The cap is on σ, and the draws are truncated at 4σ, so the depth error can reach about 4× the distance flown. The spec's wording ("depth σ no more than the distance") is right; make the comment say the same.
- `runner_trace::capture`'s guard clears `TICKS` but not `LOOKS`. A `capture` user that never calls `take_looks` (`runner_timing`, the tests) accumulates looks, and a probe that panics mid-play hands stale looks to the next `take_looks`. Clear both in the guard.

## Answers to the questions

1. **Invariants 1–3.** Held.
   - No outcome is rolled, and the base relook draws only through the runner's own keyed streams.
   - The release cap scales the σ of an observation by the true distance flown. That is the same pattern as the existing distance-scaled σ (the observation boundary builds a noisy observation from truth; the decision code never sees the cap or the truth). It is defensible as perception: he watched the release point, which `observe_holder` already gives him.
   - The relay read uses true fielder positions and the delayed `known_job`. That matches the existing convention for what a runner sees (`observe_holder`, `cutoff_in_it`).
   - `QUICK_RETURN_S` is a named constant derived from the fielders' model (`RUSH_MIN_FRAC`, `RELAY_TRANSFER_FRAC`, `INFIELD_TRANSFER_50_S`), so it is league knowledge with its source in its doc comment. It omits the receiver's turn, which makes it a slight underestimate and errs toward staying.
2. **Determinism.** No new `Purpose` and no new stream. The base look uses the decision's existing sub-indices (2n timing, 2n + 1 read). Each base look increments `decisions`, which shifts that runner's later sub-indices, as every look does by the documented convention. No `HashMap`, no direct transcendental calls (`m::hypot`, `norm`), and the ties in `min_by(total_cmp)` resolve deterministically. Every digest changed, which is expected.
3. **The gate.**
   - It returns false for every flag listed.
   - A runner who owes a retouch has `retouch` set by the catch-call handling (`live.rs:7870–7900`) and gets no base look until he plants.
   - A pending call blocks the look at the call site and in `runner_decide`.
   - A forced runner short of his base is never standing on a base he holds.
   - Two runners: a trailing runner behind an occupied base waits (`base_uncleared_ahead`), and `settle_guard` still runs on every goal afterwards.
   - The overrun batter is excluded until he is back on first. He then looks at whatever happened meanwhile, which is legal (OBR 5.09(b)(4) protects only the return).
   - `turned_back` keeps its own rule.
   - The probe's rules-warning set is empty in the worker's run. I found no way to leave a base he must retouch or to end with two on one base.
   - The fault I did find on this path is P2-2: the tag belief.
4. **Real runner?** No, for all six (P2-1 for first; P2-3 for the end of the third-base plays; the third-base *sends* are close calls a real runner can lose, driven by `RELAY_READ_S`).
5. **Tests.** Mostly independent: the throw physics, `relay_s`, `tag_apply_s`, the sourced run law. The exceptions are P3-1 and P3-2, and four cases are missing.
6. **Ledger.**
   - The probe reproduces exactly on both full sets (the counts below).
   - "Runs moved by divergence" is fair. Seed 0 runs +0.09 a team-game, with HR/PA +0.08 points (≈ +0.03 HR, ≈ +0.045 runs a team-game, about half of it), and seed 1000 moved +0.02 runs with HR/PA −0.03 points. The worker's paired standard error of about 0.15 runs is the right order (team-game SD ≈ 3, 600 team-games a set).
   - The diagnosis in the Open cell is wrong (P2-4).

## Verification

- `touch sim/crates/bb_game/src/live.rs`; `cargo test --workspace` through the queue (job 85): **535 passed, 0 failed** (sum of every `test result` line), matching the ledger row.
- `base_relook_probe 300 0` and `300 1000003000` with `RELOOK_DUMP=1` (job 86, shared `sim/target`, unmodified tree). The output matches the ledger to the digit: 4,879 [5,003] looks; 36 [38] sent; 4 [5] put out; from second 57 of 60 safe; from first and third 6 of 14 out. Five of the six outs are "out elsewhere": tagged going back to the base he left (1 or 3). The sixth (seed 41) is tagged 1.3 m short of second while going back. Traces were read from the dumps (`sim/target/rev-relook/probe0.txt`, `probe1000.txt`).
- An instrumented copy of the tree (`sim/target/rev-relook/src`, built with `CARGO_TARGET_DIR=sim/target/review-probes`; not committed):
  - `REV_LOG` (job 87) logged every base look by a runner `runner_forced` holds while he decides from a base past his start: 3,045 of the seed-0 looks, 12 sends. The output was otherwise identical to the unmodified probe, so the instrumentation does not change games.
  - `REV_FIX=1` applied the P2-2 fix: seed 0 35 sent / 4 out, seed 1000 36 sent / 6 out.
  - `REV_LOG2` (job 88) printed each base decision's run time, believed ball time, tag, timing draw and margin, and each throw read's destination, arrival, relay and truth, for the six out plays (`sim/target/rev-relook/beliefs.txt`). The tables in P2-1 and P2-4 come from it. The sigma figures were computed by hand from the observed and true velocities (`READ_VEL_FRAC` 0.04, `DEPTH_STRETCH` 3, along the runner's line of sight).
  - A print-only test (jobs 89–90) reproduced the fixtures' numbers in P3-2.
- No background shells left running.

## Resolution

Open.
