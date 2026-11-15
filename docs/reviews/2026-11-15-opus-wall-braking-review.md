# Review: wall braking, the wall-contact speed, one reach over a wall (Opus 5.5)

**No P1. Merge.** Two P2s are not caused by this branch, but it makes them worse, and they should be the next work: the pop-up hit rise (P2-1) and throws wide of a bag (P2-2).

Reviewer: Claude Opus 5.5 (not the author; Codex near its limit). Branch `work/wall-braking`, range `3f8ee60..8772075` (`a9630cf` code and tests, `a6332a9` spec, `8772075` ledger). Read: the `live.rs` diff in full, the uses of `BRAKE` in `live.rs` and `live/holder.rs`, `try_captures`, `move_fielders`, `fielder_step`, `run_gap`, `wall_delay`, `receiver_reads`, the spec paragraphs (agent-models §5.3, §5.5, §5.7, §6.4; physics-spec §9.5), the ledger row and the research note. Probes ran in copies of the source under `sim/target/review-probe` (branch) and `sim/target/review-main` (main `3f8ee60`); this branch's source was not touched.

## Findings

### P2-1. The pop-up hit rise is real extra hits, lost at the edge of range by fielders who overshoot or hand the ball to a worse read. Weaker braking makes recovery cost more (answers Q1)

`sim/crates/bb_game/src/live.rs:1365` (a late chaser runs flat out to his intercept), `fielder_step` braking at `BRAKE`, and the call hand-off.

Evidence (seed set 1000, `games --n 300`, the branch and main both rebuilt; the figures reproduce the ledger row exactly):
- **Not a denominator effect.** Fewer caught foul pops would raise the pop-up hit rate without extra hits, since a caught foul fly counts as a ball in play. But foul outs barely moved (1.61 → 1.58 a game), and pop-ups' share of balls in play went 8.7% → 8.4%. Pop-up hits counted directly: **44 on main, 78 on the branch.**
- **What they are.** An instrumented copy logged every pop-up (launch over 50°) whose first touch is the ground. The fair ones land 40–80 m from home after 5.5–7 s of hang time, untouched (one `GloveSecured` after landing and one on the throw). Of 88 fair drops, the nearest fielder at the end of the play was within 4 m in 55. These are not balls nobody went for. They are lost at the edge of range.
- **Two traced plays** (positions every 0.5 s):
  - `seed 1000003040 play 5`: the shortstop is the caller with intercept `(-37.6, 39.9)`, about 4 m long of where the ball lands, `(-33.5, 41.5)`, at 6.20 s. He runs there at 7.8 m/s. His read corrects to `(-35.0, 41.0)` at 5.0 s and `(-34.3, 41.7)` at 5.5 s, but he is still moving away. At 6.0 s he is 2.6 m from the landing point at 1.9 m/s, and it drops.
  - `seed 1000003096 play 88`: the first baseman is the caller, at 7.8 m/s with an intercept 1.2 m from where the ball lands (`ti` 5.42 s; it lands at 5.52 s). At 3.5 s he stops chasing, and the right fielder takes the ball with a read 6–7 m short (`(41.0, 40.2)`, `ti` 5.72 s by his own estimate, later than the ball). He runs in at 8 m/s, his read swings behind him at 5.0 s, and at 5.5 s he is 4.6 m away, still moving toward home.
- So the mechanism is not "runs through his spot" (the ledger's wording). It is a hard read on a high ball, corrected late, met by a chaser at full speed. At 4.75 m/s² a reversal from 8 m/s takes 1.68 s and 6.7 m, against 1.33 s and 5.3 m at 6. Main drops 44 balls the same way; this branch drops 34 more.

Verdict on Q1: the braking figure is the right one; the cost lands on a defect in arriving under the ball. A real chaser with 6 s of hang time gets under the ball early and adjusts, or comes in under control. He does not sprint to a read 4 m off and overshoot. And a caller who is on time is not displaced by a teammate whose own estimate says he is late. Pop-up hit rates of .046/.057 against MLB's about .02 are beyond "imperfect but believable".

**Suggested fix (follow-up, not on this branch):**
- (a) On a high ball with time to spare, a chaser plans to arrive early with a margin that covers how far his read can still move. He brakes onto the spot (the `early` path) instead of running through it at full speed.
- (b) Find why the first baseman left the ball at 3.5 s in play 88. A caller who is on time should not hand the ball to a fielder whose own `t_int` is past the ball's landing.
- (c) Rewrite the ledger row's mechanism sentence with this evidence. The two plays above make good regression seeds; build them as constructed states.

### P2-2. A throw 1.7 m or more wide of second is not caught by a receiver standing on the bag, even with a perfect read and hands. Not caused by this branch (main misses at 1.8 m); the test move hid it (Q2c)

`sim/crates/bb_game/src/live.rs:2951` (the `extending` stretch and dive are for chasers only, `receiver.is_none()`) and `receiver_reads` (`live.rs:3327`). Test: `a_throw_passing_beyond_reach_of_the_bag_draws_the_receiver_off_it` (`live.rs:6466`).

Probe: the test's own setup (20 m throw at 37 m/s, `read_err` 0), swept 1.3–2.6 m off the bag:
- Branch: caught to 1.6 m, **missed from 1.7 m on**.
- Main: caught to 1.7 m, **missed from 1.8 m on**.

Trace at 2.0 m off: the first read lands 0.3 s into a 0.58 s flight. The second baseman has moved 0.25 m when the ball passes 1.76 m from him, past the fixed `RECEIVE_REACH` of 1.5 m, and he has no stretch. The test sat at main's limit, and moving it to 1.6 m is a legitimate re-anchor. But the comment ("slower braking lost it") hides that every throw beyond about 1.7 m is now a miss. A middle infielder takes a throw 2 m wide with a step and a reach. This may contribute to the rise in throws out of play (0.080 → 0.097); not measured.

**Fix (follow-up):** give a receiver the stretch (`STRETCH`, with its balance cost) for a throw outside `RECEIVE_REACH`. Then add a test that sweeps the throw's offset rather than pinning one value.

### P3-1. The reach estimate's looser bound comes from a model mismatch, not from braking alone (Q2a)

`run_gap` (`live.rs:1212`), test `reach_estimate_tracks_the_movement_step` (`live.rs:5834`).

The worst case (2 m away, moving off at 6 m/s with 4 m/s sideways) is estimated 0.42 s early. `run_gap` brakes the sideways speed at `BRAKE` and grows the speed toward the goal at `a_lat`, in the goal's frame. `fielder_step` brakes along the velocity at `BRAKE` and rotates it at `a_lat`. The gap between the two scales with 1/`BRAKE`, so loosening the bound from −0.35 to −0.45 s is a consequence of the change, not a bent test. But the case it covers matters: a fielder close to the ball and moving away from it is exactly the overshoot state in P2-1. There his plan believes he gets back 0.4 s sooner than he can, which bears on dive decisions and calls (`CALL_SLACK_S` is 0.3 s).

**Fix:** under about 5 m, have `run_time_to` step `fielder_step` itself (at most about 100 ticks), then restore the −0.35 s bound or tighten it.

### P3-2. The over-wall regression only bites in its stretch case (Q3)

`nobody_catches_a_ball_farther_past_a_wall_than_his_arm_reaches_over_it` (`live.rs:9834`).

The guard sits before every capture path. `try_captures` is the only production place a ball becomes held (`live.rs:3151`), so it covers a catch on his feet, a stretch, a dive, and a throw's receiver near a wall. Plans use the same `beyond_reach_over_wall`. One limit, defined once.

A mutation probe (the guard switched off) makes the test fail: it catches the bug. But only through the "stretch" case at 0.72 m. The "standing" case at 0.72 m (0.5 m/s ball) is not caught with or without the guard: the ball passes 1.02 m from the body, beyond `REACH`, and extension needs a ball faster than 3 m/s. Geometry alone limits a catch on his feet to 0.7 m, so that half proves nothing. There is no dive case (a dive's 1.2 m ceiling equals the stands wall height, as the comment says).

**Fix:** relabel or drop the standing half; optionally add a dive case at a lower wall if one exists (the dugout roof).

### P3-3. 0.7 m over a wall is reasonable for the outfield fence, conservative for the waist-high stands rail (Q3)

`OVER_WALL_REACH_M` (`live.rs:63`).

0.7 m is right for a fielder leaping at a 2.4 m fence. Over a 1.2 m rail he can bend at the hips: torso, arm and glove reach about 1.0–1.3 m beyond it. That is an estimate, not sourced. It is the brief's "arm's length" and is acceptable. When the wall-awareness rating comes, consider a limit that depends on wall height.

### P3-4. One braking figure for small corrections (Q1)

Harper et al. measured the mean deceleration from a sprint (about 7.5 m/s) to a stop, and the note itself says deceleration ability depends on approach speed. The figure fits every use listed in the ledger:
- a fielder's stop, stop point, and braking onto a spot;
- the braking half of a turn (the along-velocity component; the rotation uses `a_lat`);
- the runner's reversal and return to a base;
- the pivot man onto the bag.

It is pessimistic for small corrections from low speed (a 2 m/s lean, small adjustments while settling), which a player takes in one step. Not worth a separate figure now. Note it for the wall-awareness and agility work. The runner's return to a base (`approach_speed`, used only when going back) brakes on foot where a real runner often dives back. At 4.75 a return from 8 m/s costs about 0.17 s more than at 6. Acceptable for now.

### P3-5. Stale text

- agent-models §6.4, "Turning round": "8 m past second at 7 m/s he drifts 4.1 m on" should be 5.16 m.
- `stopped_fielders_have_no_speed_and_running_ones_keep_it` (`live.rs:6264`) comment: "braking at 6 m/s²".
- The pivot test docstring (`live.rs:10308`): "visibly moving at 2.68" should be 2.39.
- `a_carrier_to_second_keeps_the_set_up_he_spent_running_with_the_ball` (`live.rs:10167`): it emits `GloveSecured` at (−5.0, 36.0) but `held_at` puts him at (−4.5, 36.8). Move the fact with him.

### Q2 and Q4, no finding

- **Q2(b), the dive test.** The ball's move inside the corner is a consequence of the new limit: on the fence curve it is 1.8 m past the taper wall, so no catch is attempted. A probe confirms the lunge clamp still fires (once), and the assertion is now stronger (`body_clearance(.., BODY_R)` against every wall, not the fence curve).
- **Q2(d), the carrier test.** Moving where the ball is fielded keeps the test's subject (keeping the set-up time on the touch) on the carry side of a decision that slower braking moved. A legitimate re-anchor, apart from the fact location in P3-5.
- **Q4, the wall cap.** `wall_speed_cap` = √(v_c² + 2·BRAKE·d) on the body's clearance, and `wall_delay`'s closed form (time braking from v to the cap, minus the unbraked time over the same distance) both check by hand. 0.47 s at the wall and 0.38 s at 0.3 m match the spec. A fielder never ends a tick inside a wall: `keep_inside` runs after the cap. A fielder pressing at the wall is clamped and his normal velocity zeroed every tick, so he cannot oscillate. The cap at the wall is v_c, and the tick's overshoot is removed by the clamp. `a_fielder_running_at_the_wall_arrives_at_the_contact_speed` passes (arrival about 2 m/s).
- **Q5, invariants.** `BRAKE` is sourced and `WALL_CONTACT_SPEED_MPS` is a named [E] parameter; there is no new randomness and no new transcendental (`m::sqrt` only). Replayable: my rebuilt seed-1000 run reproduces the author's ledger figures to the digit (runs 5.24, FB .236, PU .057), and `tests/determinism.rs` passes.

## Verification

- `cd sim && taskset -c 0-7 nice cargo test --workspace --release` passed: **402 passed, 0 failed, 4 ignored** (the new tests included). CPU 44–49 °C throughout.
- Mutation probe (guard switched off by a flag in the probe copy): the over-wall test fails, so it catches the bug.
- Dive probe: the lunge clamp fired once in `a_dive_never_carries_the_body_through_the_wall`.
- Throw sweep, 1.3–2.6 m off second, on the branch and on main (P2-2), plus one traced flight at 2.0 m.
- `sim-cli games --n 300 --seed 1000` on main and on the branch: the ledger's figures reproduced; foul outs 1.61 → 1.58 a game.
- Instrumented runs on main and the branch: pop-up hits 44 → 78. A trace of every pop-up that drops untouched on the branch, two plays read in full (P2-1).

## Resolution

Open.
