# Independent review of the catch-read fixes follow-up

Reviewer: Claude Opus 5.5 (wrote none of the change). Reviewed `2ec56a8..155a874` on `work/catch-read-fixes`. Sim code last changed in `9fdbf8d`, and all of it is in `sim/crates/bb_game/src/live.rs`.

## Summary

Each of the five fixes does what the worker's follow-up says it does. I found no runner decision that still reads hidden state:

- The loose-ball relook now goes through the same observation as the first look.
- The holder's reception look is fixed when he takes the ball.
- The hold for the call keys on a visible possession.
- The settle guard reads other runners' positions and signed speeds, never their goals.
- A fast retouch reports the runner leaving the bag on the same tick as the touch, and he makes his one decision when he plants.

Every new fixture fails on the commit before its fix, at the assertion it names. The same seed gives the same digest across runs, and the change adds no unordered iteration, no direct transcendental and no `mul_add`.

There are no P1 findings. The one substantive finding (P2-1) concerns question 8. The worker's trace of the 76 runners doubled off at 25–50° is right about the mechanism. Its caveat that the sample "does not show a population bias" no longer holds. Even with no read error, the runner's first look reads 25–50° flies long across the whole population, by a median 2.5–2.8 m on deep flies. About one in eleven in-park flies landing 100 m or more is read as a home run. That bias belongs in the follow-up job alongside the missing in-flight relook. **The branch is safe to merge with that gap left for a follow-up**: the bias predates this follow-up, and none of the five fixes makes it worse.

## Findings

### P2-1. The runner's first look reads 25–50° flies long across the population, not only on the doubled-off plays

The first look flies the runner's noisy motion with the reference atmosphere, typical spin and the plate prior ([live.rs:243](../../sim/crates/bb_game/src/live.rs#L243), `ContactLook::project`; [live.rs:1464](../../sim/crates/bb_game/src/live.rs#L1464), `believed_spin`). I measured it with **no read error at all** on every fair 25–50° ball in 80 games (seeds 0–79). The look came at 0.5 s after contact, with the same model, through the public `read_prediction` with zero error draws. I compared the signed carry miss along the spray line against the true landing point (+ means read deeper):

| True landing | In-park flies | Read clears the fence | Median miss | 10th pct | 90th pct |
|---|---:|---:|---:|---:|---:|
| < 60 m | 24 | 0 | +1.1 m | +0.6 | +2.2 |
| 60–80 m | 111 | 0 | +2.2 m | +0.6 | +5.5 |
| 80–100 m | 361 | 1 | +2.5 m | +0.8 | +5.8 |
| ≥ 100 m | 342 | **32** | +2.7 m | +0.8 | +5.5 |

Of 125 true home runs, 5 were read in the park.

The bias is one-sided: even the 10th percentile is read deeper than the truth. Two causes are ruled out:

- **The plate prior.** Flying with the true contact point gives the same table: +2.5 and +2.8 m on the two deep bins, and 32 false home runs.
- **A stale spin table.** `batted_spin` (48 games) gives current medians of 1,902, 2,914 and 3,773 rpm for the 20–30°, 30–40° and 40–50° bins. The table in [live.rs:1444](../../sim/crates/bb_game/src/live.rs#L1444) holds 1,909, 2,949 and 3,770.

What remains is the believed flight itself: a pure-backspin axis at the median rpm, decayed through `fly_back`, against the true spin. My guess is the true axis's tilt and gyro component (pure backspin gives the most lift) or the nonlinearity of carry in spin. I did not isolate which.

**Consequence.** A runner whose zero-error read already clears the fence gets a catch score near 0 and advances at once ([live.rs:2608](../../sim/crates/bb_game/src/live.rs#L2608)). The doubled-off runners on deep flies are partly a systematic misread that every runner shares, not only the random misjudgment of carry a real runner makes. The worker's own numbers fit this: 16 of the 76 clear the fence even with the true motion, and the rest land a median 4.2 m long.

**Scope.** This predates this follow-up. `believed_spin`, `typical_batted_spin_rpm`, `read_prediction` and `position_from_plate` are unchanged since the merge base `43bd29e`. Fielders use the same snapshot read, but they correct it with repeated tracking looks. The 2026-11-15 outfield-read note says it fixed "read every fly too deep", so its first-look bias is worth rechecking with the same probe.

**Recommendation.** Make the read bias the first step of the follow-up job for the 25–50° double-offs, before any in-flight relook or cutoff calibration. AGENTS.md says to look for a planner or model bug before tuning. Calibrating the 20% cutoff against a read that runs 2.5 m long would bake the bias into the cutoff. Correct the related sentence in the runner-timing note ("does not show a population bias").

### P3-1. The no-catch clause of `awaiting_call` cannot happen in a game today, and it would strand a planted tag-up runner if it could

`awaiting_call` also holds runners through the reaction window for a glove possession that the rules did not rule a catch ([live.rs:4642](../../sim/crates/bb_game/src/live.rs#L4642)). Two facts make that combination impossible in a game:

- The rules rule a catch for any `GloveSecured` before a ground touch ([engine.rs:875](../../sim/crates/bb_rules/src/engine.rs#L875)).
- `emit` copies every ground or wall fact into `no_catch_at` while no catch stands ([live.rs:2158](../../sim/crates/bb_game/src/live.rs#L2158)), and once that is set `flight_secured_at` stays empty.

So a possession in flight that is not a catch cannot occur. The fixture reaches it only by passing a ground touch to the rules with `step`, bypassing `emit`.

The change is still right: it ties the hold to what the runner sees rather than to the ruling, as P1-3 asked. On real catches the protection comes from `awaiting_catch_call`, which is applied once: the second clause requires `!caught()`. A trap or a ball off the wall sets `no_catch_at` first, so it never starts this hold. A catch's hold ends at `catch_seen`.

The latent problem: if the rules ever gain an umpire's no-catch call on a glove possession, a runner planted on his base for a tag-up would never leave it. The tag-up branch waits for `catch_seen` or `no_catch_read()`, and neither would fire ([live.rs:5168](../../sim/crates/bb_game/src/live.rs#L5168)). A comment at the clause, saying the rules cannot produce this case today, would keep the next reader from relying on it.

### P3-2. The new departure on crossing covers every runner, not only fast retouches, and nothing tests the wider case

`9fdbf8d` reports a departure for any runner whose step crosses a bag and ends the tick off it ([live.rs:5308](../../sim/crates/bb_game/src/live.rs#L5308)). Before this, a runner crossing a bag that was not his goal got a `BaseTouched` and no `BaseLeft`, so the rules kept `on_base = Some(b)` until his next touch ([engine.rs:342](../../sim/crates/bb_rules/src/engine.rs#L342)). That covered:

- a runner going back to his start who passes an intermediate bag;
- a lead runner whom the settle guard pushes on past his goal.

Such a runner counted as standing on that bag for tags and for passing. The fix closes a real, pre-existing hole in the rules, and agent-models §6.4 states the general rule. The ledger row, the follow-up's text and the fixture describe only the retouch case, and no test tags a runner between bases after such a crossing. Ordinary advances are unaffected, because a runner's goal moves one base at a time and he lands exactly on it. That fits the near-identical standard runs.

Touch and departure are both stamped at the tick's end, `self.t`, like every touch in the sim. The appeal check `t >= first_touch` sees the touch first. A crossing in the middle of the same tick as the catch can be stamped up to one `DT` (1/60 s) after the true crossing. That convention predates this change and is not made worse by it.

### P3-3. The 3,000-layout property test never has a catch call pending

`random_traffic_case` builds grounder states only: a ground touch at 0.5 s and no catch ([live.rs:12069](../../sim/crates/bb_game/src/live.rs#L12069)). So the only per-runner inputs to the guard, `runner_forced` and `runner_sees_out` through `awaiting_catch_call`, are the same for every observer in it. By reading the code I checked that the two runners' views cannot disagree in a game:

- Only a planted tag-up runner sees a catch before the others.
- That runner stands on his own start base, where a forced runner coming up behind cannot be standing.
- Standing is required for giving way.

So I found no deadlock (both holding one base) and no double move. With views equal, the trailing runner gives way exactly when the lead runner, by his own check, does not go on. A tick-order check also holds: a trailing runner who gives way is moving away from the bag by the time the lead runner looks, so he no longer counts as settling there. This is a coverage gap, not a defect; a pending-catch variant of the property test would turn this argument into a test.

### Other answers to the brief

- **Q1 (P1-1).** `runner_predict` is now `observe_ball(k, sub).project(..)` ([live.rs:2441](../../sim/crates/bb_game/src/live.rs#L2441)). The plate cue is `public_contact_since`: the time since contact while no ground or wall fact has been emitted, no catch has been ruled and no glove has touched the ball ([live.rs:2448](../../sim/crates/bb_game/src/live.rs#L2448)). Every bounce and wall touch is emitted as a fact on its tick ([live.rs:4287](../../sim/crates/bb_game/src/live.rs#L4287)), so the plate cue goes off exactly when a visible touch happens.
  - The draws are the same keyed `RunnerPerception` stream and sub-index as before (`runner_ball_errors(k, sub)`, four truncated normals in a fixed order). The first look keeps sub 1, and relooks keep `2n + 1`. No purpose is added and no stream is reordered.
  - `off_bat()` remains only in fielder code (lines 2244, 2256, 2277, 2961, 3375) and the read trace. No runner path reads `inp.ball.r`.
  - One small behaviour change in the first look: `view` is now always the time since contact. Before, it was 0 once a ground fact existed. The first look runs only while `no_catch_at` is empty, so this matters only for a glove deflection in flight inside 0.5 s, which now drops the plate cue.
- **Q2 (P1-3).** Covered under P3-1.
- **Q3 (P1-4).** Covered under P3-3. "Settling" for another runner is judged from his visible position and signed speed ([live.rs:4844](../../sim/crates/bb_game/src/live.rs#L4844)). "Refuge free" reads only visible occupancy and motion toward the base ([live.rs:4900](../../sim/crates/bb_game/src/live.rs#L4900)). Each runner changes only his own goal.
- **Q4 (P2-1 retouch).** The departure is stamped on the same tick as the touch (see P3-2). The new rule ([live.rs:4981](../../sim/crates/bb_game/src/live.rs#L4981), [live.rs:5225](../../sim/crates/bb_game/src/live.rs#L5225)) agrees with the rules and with baseball sense:
  - A runner is entitled to the base he holds (OBR 5.06(a)(1)), and the plate is no refuge for a runner from first.
  - A runner carried behind his own bag goes back to it.
  - A runner who overran and must return now gets no relook until he is back. On the way back his arrival is a running arrival, so he decides at the bag ([live.rs:5323](../../sim/crates/bb_game/src/live.rs#L5323)). The cost is a few tenths of a second on a ball that drops, which a real runner would also lose.
  - The planted-at-the-catch branch ([live.rs:5141](../../sim/crates/bb_game/src/live.rs#L5141)) decides once, at `catch_seen`.
- **Q5 (P1-2).** `catch_seen_vel` is written only at the reception ([live.rs:4163](../../sim/crates/bb_game/src/live.rs#L4163)). `catch_vel`'s other readers are the world's own set-up (`set_up_for` and the reception set-up), not runner code. `catch_heading`, `catch_received_throw` and `catch_stretched` are also written only at the reception.
- **Q6 (tests).** All five fail on their parents, at the claimed assertions (Verification). None restates the implementation: four hold everything visible fixed and vary only hidden state, and the retouch fixture checks the physical fact that a touch happens only on a tick whose movement reaches the bag.
  - The P1-1 fixture failed on its first arm (moving the contact point 15 cm changed the predicted landing from (6.647, −1.130, 108.526) to (6.644, −1.249, 108.494) and the pickup time at third from 10.785 to 10.784 s). That stopped the test, so its replaced-path arm was not checked on the parent here.
  - The retouch fixture failed on the parent's missing departure. Its one-decision assertions come later in the test and were not reached there.
- **Q7 (determinism).** The diff adds no `HashMap` or `HashSet`, no `mul_add` and no direct `f64` transcendental. The new vectors in `settle_guard` are filled in index order. Same-seed digests are identical (Verification).
- **Q8.** The worker's mechanism is confirmed in code. Relooks fire only on a change in the counts of securings and releases ([live.rs:4972](../../sim/crates/bb_game/src/live.rs#L4972)), so nothing happens while the ball is in the air, and a score below the 20% cutoff advances the runner at the first look ([live.rs:2608](../../sim/crates/bb_game/src/live.rs#L2608)). The follow-up leaves the bin at 38.0, as the ledger shows. Added to the planner gap is the read bias in P2-1. Merging is safe as long as the follow-up job takes both.
- **Q9 (docs).** The agent-models §6.3–§6.4 edits match the code: the relook's observation boundary, the possession hold, the settle guard reading motion, the retouch at speed, the departure on crossing and the behind-own-base rule. The runner-timing note is accurate except for the "population bias" sentence (P2-1).

## Verification

All runs used `CARGO_BUILD_JOBS=2 taskset -c 0-7 nice`, one at a time, built in `sim/target`.

- Touched `live.rs` and ran `cargo test --offline --workspace --release` at `155a874`: 462 passed, 0 failed, 4 ignored.
- **Fixtures on their parents.** For each fix commit I took the parent's `live.rs`, applied only that commit's test hunks, built in a copy of the tree under `sim/target/codex-probes/opus-fix2/` (sharing `sim/target`), and ran the one test. For `f5d3a14` I removed the one line that adds `view:` to an older test's literal, a field that does not exist on the parent. All five failed:
  - `f5d3a14`: "the true contact point moved the relook".
  - `2cdc419`: the feed's holder look differs from the other routes after the bag touch.
  - `bc61b7c`: movement through the window differs, with a decision made in the no-catch arm.
  - `ace95f0`: "the lead runner's private goal moved the guard" (3 against 2).
  - `9fdbf8d`: "the drift past the bag is a departure".
- **Carry probe.** A throwaway example ran the zero-error first-look read on all fair 25–50° balls in seeds 0–79, once with the plate prior and once with the true contact point (table in P2-1). It is kept at `sim/target/codex-probes/opus-fix2/zz_opus_carry_probe.rs` and is not in the crate.
- `batted_spin` (48 games): the median spin by launch angle against the believed-spin table.
- **Determinism.** `examples/digest` (seeds 1, 2, 3, 42) twice gave identical digests. `sim-cli game --seed 7` twice gave identical output apart from the wall-clock line.
- Read the diffs of all five commits, the touched paths in `live.rs` (`move_runners`, the movement step, `runner_relook`, `ball_time_to`, `observe_holder`, `read_contact`, `runner_catch_read`, `emit`, `ball_facts`) and `bb_rules` (`base_touched`, `BaseLeft`, the tag judgement, the catch ruling).

**Not checked:**

- I did not rerun the 300-game standard runs or `runner_timing`; I took those figures from the ledger.
- I did not isolate the source of the carry bias between spin axis and spin-drag nonlinearity.
- I did not check the fielders' first-look bias.
- I ran no in-game probe of a tag on a runner after a non-goal crossing (P3-2), and no property test with a catch pending (P3-3).
- On the parents, the P1-1 fixture's replaced-path arm and the retouch fixture's one-decision assertions were not reached.

## Resolution

Open.
