# Review: the approach, rounding and the angle from contact (`work/approach`, `dce9769..af5825d`)

Reviewer: Opus (high effort). Scope: `dce9769..af5825d` (sim), plus the docs-only commits up to `01f6856`. Code: `live.rs` (approach, rounding, angle, set-up), `live/holder.rs`, `bb_agents/src/throwing.rs`; tests in `live.rs`, `throwing.rs`, `tests/throw_choices.rs`; spec `agent-models.md` §5.2, §5.3, §5.5; note `docs/calibration/2026-11-20-approach.md`; ledger row. I wrote none of it. Q32 and Q33 are decided and not reopened.

**Summary.**
- **No P1.** No decision reads world state (invariant 2). Every movement branch goes through `step_toward`, so no fielder can exceed his acceleration, braking or turning limits. Nothing new draws a random number, and all transcendental math goes through `m::`. The workspace tests pass. The two new frozen plays fail on `8621f54`, pass on main and pass on the branch.
- **The pop-up and fly-ball rises are noise, not lost catches.** I ran main and the branch on 600 direct games (seeds 0–299 and 1000–1299) and paired identical balls in play.
  - On seeds 0–299 the rises reproduce: FB .1026 → .1106, PU .0045 → .0095.
  - On seeds 1000–1299 both rates fall: FB .1031 → .0987, PU .0102 → .0087.
  - On identical balls the branch loses 2 catches main makes and makes 4 that main loses (flies, liners and pop-ups together).
  - Both lost catches are the set-up behind a fly. Each is caught with the set-up switched off, and each matches the note's own open case (seed 11 play 64): a set-up chosen on an early read takes the man off his plain path, and he cannot get back when it is dropped (P2-2). One of the two is a pop-up (seed 207 play 66), so the untraced pop-up case now has a trace.
- **The angle from contact is weighed against the wrong alternative while the ball is in the air** (P2-1). Against going straight at full speed it looks good. Once the ball is down it is weighed against braked straight takes, and it loses. That is why it is chosen about 2,700 times in 300 games and is still his way at the take once. Switched off over 300 games, it changes 6 of 15,000 balls in play, 3 each way.
- **The approach to a ball on the ground spends all the time in hand, with no margin and no plan-against-movement test** (P2-3). Against a throwaway approach-off build with the same rules, infielders give up 99 balls they field without it (89 of them become hits) and field 77 they otherwise miss. Example: a pitcher backs away from a comebacker to take it deeper and it gets by him (seed 1048 play 193). On average it costs nothing (the GB hit rate is level), but the losses are plays a real infielder makes.
- **The ground-ball hit rise against main is real and comes from the across term, not the approach** (P3-1). The ledger lists it without a cause.
- The other findings are minor: duplicated race constants, stale spec and comment text, a few literals, and partly restated tests (P3-2 to P3-7).

## Findings

### P2-1. The angle from contact is chosen in the air against an unbraked straight take, then given up on the ground against a braked one

`live.rs:3912` (`plan_angle`: `let r_straight = tp + at_base(p, along * v);`) against `live.rs:3730` (`plan_approach`: `let vc = if first { v } else { (v - m::sqrt(2.0 * BRAKE * v * slack)).max(0.0) };`).

- **What is wrong.** While the ball is up, `plan_angle` compares each angled way with going straight at full speed and paying the whole plant for the motion across the line to his base. Once the ball is down, `plan_approach` compares the same angle with straight takes braked by his time in hand (`vc`). Braking under `PLANT_FREE_MPS` owes no plant, so the braked straight take usually matches the angle or beats it.
  - The in-air plan therefore credits the angle with a saving that the on-ground plan, from the same state, does not count.
  - "Never slow for a ball in the air" does not require the unbraked comparison. The angle is planned only for a take on the ground beyond his reach in the air, so the braking a straight take would need also happens on the ground.
- **What it does.** The note's own figures: the angle is chosen in about 2,700 outfield plays per 300 games and is still his way at the take in 1 of 600 games. Of those that chose it, 123 of 127 had given it up 0.4 s or more before the take (note §6).
  - So in about nine plays a game an outfielder runs a curve and then straightens, which the client will show.
  - It costs planner time (the standard run's time per game rose 4,735 → 5,042 ms across the branch; the angle's share is not split out).
  - It cannot do the job it was built for, which is bringing first to third down.
  - Review probe, angle switched off by a throwaway switch, seeds 0–299: 6 of 14,916 identical balls in play change, 3 each way, and every hit rate is within 0.002.
- **Failing case.** The note's quick singles with a runner on first: the angle is chosen in the air, and once the ball is down the braked straight take is within `APPROACH_GAIN_S` of it or better, so it is dropped.
- **Fix.** In `plan_angle`, weigh the angle against the best straight take with the braking the ground approach would give it (the `vc` rule, for the part of the run after the first bounce), so that a way chosen in the air is still preferred once the ball is down. If that leaves the angle never chosen, take it out: about 500 lines and seven [E] parameters for no change in games. Either way, re-measure first to third. The angle is the only mechanism on the branch aimed at it (P3-1 and Q4 below).

### P2-2. The set-up behind a fly, chosen on an early read, still costs catches that main and the branch without it make

`live.rs:3528–3582` (`fly_setup`, the gate `left >= settle + come_back + SETUP_SPARE_S` and the `fd.setup == 1 || at_back` branch). The movement to the spot is the ordinary `early` path of `fielder_step`.

- **What happens.** Paired balls in play, main against the branch, 600 direct games: 2 catches main makes are lost on the branch. Both are caught when a throwaway build switches only `fly_setup` off (the angle and the approach switched off do not change them).
  - *Seed 207 play 66* (a pop-up, 57°, to shallow left; runner on third, fewer than two out). The left fielder accelerates more slowly from 0.8 s. At 1.0 s he is moving at 1.24 m/s against main's 1.66, because he is heading for a spot behind the catch point that he is "early" for, with his speed held to stop on it. He is 0.57 m behind main's track from then on. Once the set-up is dropped he sprints at 8.1 m/s, as on main, and comes up 1.0–1.4 m short as the ball comes down. Main and the set-up-off build catch it at 5.97 s. The batter takes second.
  - *Seed 1042 play 28* (a fly, 45°, to right-centre; runner on second, none out). From 1.7 s the right fielder drifts deep, to y = 80.2 m against main's 78.9 m. He stays 1.3–1.7 m behind main's track and the ball lands 1.3 m from him. Main catches it at 5.72 s.
- **How often.** With the set-up switched off, over 600 games it gains 4 catches and loses 2 on identical balls. Fly and pop-up rates are within 0.001. So it is not a net loss, and the fly-ball and pop-up rises are not this. But both losses are the "first read" case the note leaves open (§6, seed 11 play 64), and a real outfielder does not drift behind a fly in the first two seconds before he knows where it is going: "go to the ball first and set up once under it" (the note's own words).
- **Fix.**
  - Enter the set-up only once his read has settled. For example, require his doubt (`ARRIVE_SIGMAS · doubt`) to be within `SETUP_AT_M`, or require that he would already be at the catch point with `settle + come_back + SETUP_SPARE_S` in hand going there first.
  - Until then, run at the catch point as plain play does, and drift back to the spot only from under the ball.
  - Freeze 207/66 and 1042/28 as plays that must be caught. Check first that the fixture replay (average fielders, default positions) reproduces them.

### P2-3. The approach spends all of the time in hand on a later take, with no margin and no plan-against-movement test, and infielders give up balls they field without it

`live.rs:3730` (`vc` from the whole `slack`), `APPROACH_GAIN_S = 0.01` (`live.rs:653`). The movement is `step_toward`'s `arrive` cap.

- **What is wrong.** Braked to v_c = v − √(2·BRAKE·v·s), he reaches the take point exactly as the ball does by his plan. A later take must beat the intercept by only 0.01 s.
  - Rounding needed a 0.3 s margin and a 0.15 s gain over its plan-against-movement bound, and the angle a 0.05 s gain, both from the same failure: "paced to meet the ball exactly, he was a step late and the ball went past him".
  - The plain braked take has neither, and no test checks its time against the movement step. `a_shortstop_with_time_in_hand_slows_into_a_ground_ball` checks one ball.
- **What it does** (review probe, the branch against a throwaway build with only the ground approach switched off, same rules, 600 direct games, about 10,300 paired ground balls):
  - Ground balls an infielder fields without the approach but fields more than 1 s later with it (it got by him): **99**, of which 89 become hits.
  - The other way: 77, of which 72 are hits.
  - Net ground-ball hits on paired balls: 139 gained against 134 lost. The GB hit rate is .3191 → .3156 on seeds 0–299 and .3283 → .3290 on 1000–1299. So on balance it trades, but it churns about 175 balls in 600 games, and the excess loss (22, about 1.7σ) is of the wrong kind.
- **Failing case, seed 1048 play 193.** A chest-high comebacker to the pitcher, who has a throw to first.
  - Without the approach (and on main) he steps to his left, takes it at 1.07 s in front of him, and the batter is out.
  - With it he backs away toward a deeper point on the ball's path, the ball passes him 0.64 m away at 1.27 m height, and it rolls to a stop. He picks it up at 4.30 s: a hit.
  - Seed 136 play 13 (the second baseman; main against the branch) is the same thing on a grounder: on nearly the same path, the ball passes 1.1 m from him instead of into his glove, and he picks it up 2.4 s later.
  - A real infielder does not back off a ball he can take in front of him for a better throwing angle.
- **Fix.**
  - Keep a margin in the braked take as the angle does: `ROUND_MARGIN_S` scaled by the share of the ball's motion across his way. That leaves a ball coming straight at him unaffected.
  - Do not plan a take behind him on a ball coming at him (later point farther from home than he is, with the ball's velocity toward him).
  - Add a plan-against-movement test for braked takes like `the_plan_costs_a_rounded_take_as_the_movement_runs_it`.
  - Freeze 1048/193 (seed 1048 play 193).

### P3-1. The ground-ball hit rise is real and its cause is not in the ledger

Ledger row of 2026-11-20 ("Cause" column).

- **The rise.** GB hits rise against main: .3118 → .3156 on seeds 0–299 and .3189 → .3290 on 1000–1299 (direct seeds). On identical balls, 173 grounders became hits and 98 became outs.
- **What causes it.** Most are infield plays taken by the same fielder a median 0.11 s later, with the batter safe.
- **Not the approach.** The approach-off build with the same rules is .3191 / .3283, so the rise is the across term of Q32 on infield throws, with the approach slightly offsetting it.
- **Why it matters.** The ledger quotes GB .312 → .316 without a cause, and ground-ball BABIP moves further from MLB's ≈.24.
- **Fix.** One line in the Cause column (Q32's across term on infield throws, the approach about neutral) and a §5 lead.

### P3-2. The base rule's race duplicates the holder's literals and differs from them

`live.rs:714–715` (`RACE_SIGMA_S`, `RACE_MIN_P`), `live.rs:3617–3626` (`outfield_round_base`).

- **Duplicated values.** The holder's own race rule, which this one says it follows, uses bare `0.12` and `0.3` (`live/holder.rs:132,170,194`, `live.rs:5853–5854`). A change to one will not reach the other. Name them once and use the names everywhere.
- **Runner direction.** `outfield_round_base` takes `r.v` (a speed) without `r.direction`, and `heading = floor(s/BASE_L)+1`. A runner going back to his base (a line drive, a tag-up) is treated as advancing at that speed. The holder's options use `visible_target` (direction-aware).
- **Tag.** It also counts no tag time on a non-force base.
- **Fix.** Use `visible_target` and the holder's tag term.

### P3-3. Stale spec and comment text

- `agent-models.md:464` (§5.3): "Coming through a set-up behind a fly ... (distance over time left, at most his top speed ...)". As built (`ba1b109`) it is `paced_speed`, the movement step's speed law and braking (§5.5 was updated, §5.3 was not). The same paragraph has rounding paced "with the ball"; it is `ROUND_MARGIN_S` before the ball.
- `agent-models.md:447` (§5.2, rounding, *What it does*): "with the safeguards above no take is rounded in 600 games". That holds for `5c83ab4`. At `8621f54` and `af5825d` there are 5 in 600 (note §6 table, ledger §1).
- `agent-models.md:454`: rounding's *What the others know* bullet sits after the angle's section and its own *What others know* line. Move it up under rounding.
- `live.rs:2439`: the comment "On an approach (agent-models §5.2) he is timed against the ball at his take point, and arrives there no faster than he planned" sits above the set-up come-through block, not the approach. Move it down to the `(t_goal, arrive)` match.
- `live.rs:16522–16524` (`the_set_up_behind_a_fly_does_not_cost_a_catch`): "the two catches the set-up's come-through timing loses in games ... are made here, over this grid and eight read-noise draws". Those two are fixed and frozen, and the test runs one play per ball, with no eight draws.

### P3-4. Literals in the logic

- `live.rs:3755`: `+ 5.0` degrees, the slack on the widest way kept while coming through.
- `setup_depth`: the cap `2.0 * SETUP_BACK_M`. The spec says "at most 5 m".
- `angle_tangent`: `(1.0 + c).max(0.05)`.
- `fly_setup`: `to.norm() < 1.0`.

The first two change behaviour and want names. The 30° and 60° limits, the 2.5 m depth, the 0.3 s and 0.15 s safeguards, `ROUND_M`, `ANGLE_GAIN_S` and the rest are named and in the spec.

### P3-5. Tests that restate the implementation, and one that tests an inert feature

- `the_crow_hop_gain_goes_with_the_set_throw` (`live.rs:16467`) asserts `arm + 0.5·gain` at hurry 0.5, which is the formula `(1 − hurry)·gain` as coded. The 0 and 1 ends and the sourced `MOMENTUM_GAIN × 2` are the independent parts.
- `the_drift_term_follows_the_unbraked_speed` (`throwing.rs:506`) computes `w` with the implementation's expression. Its fixed figure 0.546 is the check; the rest restates it.
- `the_angle_from_contact_does_not_lose_a_ball_in_the_gap`: on this build the five balls are taken by the same man at the same spot within 0.02 s with the angle on and off. The angle is planned and dropped before the take each time (P2-1), so it guards a path that is never taken to the end. It needs at least one ball on which the angle is still his way at the take.

### P3-6. A stale `through` flag can carry into a later chase

`round_reached` runs on every fielder every tick (`live.rs:5065`), not only on chasers. `set_chase` ORs the old `through` into a new plan on the same way (`ROUND_SAME_COS`). A man who chased, stopped and chases again on the same line could start "coming through" at once. It is rare and harmless today (rounding is almost never chosen). Clear `approach` when the role leaves `Chase`, or run `round_reached` only for chasers.

### P3-7. Outside this branch: a fielder who misses a catch stands still beside the dead ball for about a second

Seen while tracing 207/66. After the ball came down, the left fielder stood 1.0–1.05 m from it, still, from 6.42 s until he picked it up at 7.67 s (the ball came to rest at 6.67 s). The approach-off build does the same (7.62 s), so it is not this branch. It looks like the once-a-second `recall`, and it gave the batter second. It belongs on the backlog.

## Answers to the questions

1. **Invariant 2: held.**
   - Every input traces to the fielder's own read (`belief`, `t_int`, `int_pt`, `doubt`, `call`), his own body (`planning_state`, his attributes) or what everyone sees (runners' `s`, `v` and `start`, the outs, the field, and `ball_still_up`, an existing "fact everyone can see").
   - Set-up: `fly_setup` (read, call, runners, outs, field). Angle: `plan_angle` (`first_bounce` of his belief, `run_gap` from his planning state). Base rule: `outfield_round_base` (runners as seen; his arm and transfer). Timing: `setup_depth` and `settle_time` (own body). Rounding's safeguards: the ball's velocity is `bv` from `pred.at`, his belief.
   - `fd.target` is read elsewhere only by the debug trace. Runners read the take as it happens: `observe_holder`, with `footwork_s` of the visible motion.
2. **Movement physics: no limit can be exceeded.**
   - Every branch of `fielder_step` (set-up come-through, angle, rounding, braked approach) ends in `step_toward`, which caps speeding up at the speed law, braking at `BRAKE` and turning at `a_lat`. Pacing only lowers the top speed.
   - Plan against movement, run here: rounding over 82 cases, −0.06 to +0.11 s and within 24°; the angle over 78 cases, −0.05 to +0.04 s, within 2°, slowed 0.75 m/s at most. Both as claimed.
   - The braked ground take has no such test (P2-3).
3. **Balls lost.**
   - The fly and pop-up rises are not robust (summary). On 600 paired games the branch loses 2 catches main makes, both to the set-up's early detour (P2-2). Neither is read error alone, and both are caught with only the set-up off.
   - The note's "hits both ways" attribution holds on my pairs: 6 flies became hits with no change in the catch, the same first touch at the same time.
   - The approach does lose some infield balls (P2-3).
4. **First to third.** A real consequence of Q32 as built, not a runner reading what he should not.
   - The holder's race (`set_up_for`) and the runner's read at the pickup (`HolderLook::release_time`) both charge the across plant through `footwork_s`, from the motion the runner sees. Before the pickup the runner's read is unchanged (intercept plus `TYPICAL_TRANSFER`, no plant: the cautious error).
   - Outfield assists fell (0.255 → 0.230), so runners who now go are safe in the world too, which a misread would not produce.
   - The one mechanism meant to bring it back, the angle, is undone by P2-1. I did not re-measure first to third myself.
5. **Determinism: held.**
   - No new RNG draw or `Purpose`; no `HashMap`/`HashSet`; no `mul_add`.
   - Transcendentals go through `m::` (`Vec3::rotate` uses `m::sin`/`m::cos`); `to_radians` and `signum` are exact.
   - The test switches are `cfg(test)` thread-locals. Two runs of the same build gave identical balls-in-play listings.
6. **Inert machinery.**
   - Rounding is correct where it runs (its test, above) and rarely runs (5 rounded takes in 600 games). I would keep it, behind its switch: it is the owner's Q33, small at run time, and tested.
   - The angle I would not keep as is. It runs a curve in about nine plays a game, almost never keeps it, and moves 6 of 15,000 balls in play, 3 each way. Fix P2-1 and re-measure, or remove it.
7. **[E] parameters.** Named and in the spec, apart from P3-2 (race constants) and P3-4.
8. **Tests.**
   - Workspace tests pass.
   - The frozen plays `seed_155_play_23_…` and `seed_264_play_109_…` fail on `8621f54` (the left fielder takes both off the ground, at 3.62 s and 6.65 s) and pass on main and on the branch.
   - The independence issues are P3-5.
9. **Spec, note and ledger.**
   - P3-1 (the ground-ball cause) and P3-3 (stale text).
   - The ledger's seed-1000 figures come from `sim-cli games --seed 1000`, which is not direct seeds 1000–1299. On direct seeds 1000–1299 the fly and pop-up rates fall, which is worth a line beside the "about 1.5σ" claim.

## Verification

All builds were in `sim/target/review-probes/approach/` (`tgt`, `tgt-main`, `tgt-br`, `tgt-nosetup`, `tgt-noangle`), from `git archive` trees with every file touched afterwards.

- **Separate target directories per tree.** Cargo hashes workspace members by their path relative to the workspace root. A tree built into the same target directory as another tree was taken as fresh and not rebuilt (my first main run reused `8621f54`'s binary). Every result below comes from a build whose log shows its own tree compiled.
- **Workspace tests.** `cargo test --workspace --release` on the worktree: all pass (bb_game lib 261 passed, 4 ignored; `throw_choices` 8 passed).
- **The six approach tests, `--nocapture`.**
  - Rounding: 82 cases, −0.06..+0.11 s, 24°.
  - Angle: 78 cases, late +0.04, early −0.05, 2°, slowed 0.75 m/s.
  - Shortstop: take 1.97 → 2.05 s, off the line 6.29 → 2.79 m/s, release 3.22 → 2.65 s.
  - Set-up: through at 6.02 m/s toward home, never past the point.
  - Guard: 22 of 22 caught, 9 through at more than 3 m/s.
  - Gap test: the same fielder, same spot, within 0.02 s, on all five balls.
- **Frozen plays.** `throw_choices.rs` and the two fixtures copied into `8621f54` and `dce9769` trees: both fail on `8621f54`, both pass on `dce9769`.
- **Balls in play** (`rv_bip`, a throwaway example: one line per fair ball in play, with the provisional hit, the catch and the first glove possession). Main against the branch on direct seeds 0–299 and 1000–1299, joined on (seed, play, EV, LA, spray): 9,709 and 9,575 identical balls.
  - Rates, seeds 0–299: GB .3118/.3156, FB .1026/.1106, PU .0045/.0095.
  - Rates, seeds 1000–1299: GB .3189/.3290, FB .1031/.0987, PU .0102/.0087.
- **Throwaway switches** (an environment variable in copied trees; unset, each is identical to the branch on seeds 0–4).
  - Set-up off: 4 catches gained and 2 lost by the set-up over 600 games.
  - Angle off: 6 balls in play changed of 14,916, 3 each way.
  - Ground approach off: 99 infield balls lost and 77 gained by the approach; GB hits 139 against 134 on paired balls.
- **Traces** (`trace_play` and a throwaway tick-level track `rv_track`): 207/66, 1042/28, 1048/193, 136/13, a sample of 14 gained infield hits (2/3, 30/193 and others), and the post-miss stand on 207/66 with the approach off.
- I did not re-run the standard `sim-cli games` runs or the note's trace probe. Probe runs used cores 0–5, one at a time; the laptop peaked at 92 °C once (8 processes), and after that runs used 6 cores.

## Resolution

Coordinator, 2026-11-20. Fixes by an Opus worker on `work/approach` (`0d50b02`–`657811f`, final sim commit `6ed53e4`; note §7), the spin test by a Sonnet worker (`7f46ef8`). No re-review: P2-1's fix is a deletion and P2-2/P2-3 are covered by a new plan-against-movement test and four new frozen plays.

- **P2-3, fixed.** The cause was the plan and the movement disagreeing: the movement braked him to his planned speed at his full reach while the plan timed the take at 0.7 m, so he stood a metre off the ball's path. He now brakes at the planning reach; `brake_floor` keeps the plan within the braking he can do; `braked_take` plans no take he would run past before the ball arrives; `ball_passed_him` forbids a take behind him on a ball coming at him. Test `the_plan_times_a_braked_take_as_the_movement_runs_it` (93 cases, −0.65 to +0.08 s, within 0.71 m). Frozen plays `1048-193` and `136-13` fail on `af5825d` and pass. The suggested margin was tried and not kept: it broke frozen play `seed_13_play_140` and raised triples and pickup-to-release; without it the excess of balls getting by is gone (64 lost against 75 gained on paired balls, against 99 and 77 before).
- **P2-2, fixed.** The set-up is entered only once his tracked doubt (`ARRIVE_SIGMAS × doubt`, from a read after his first look) is within `SETUP_BACK_M`. Frozen plays `207-66` and `1042-28` fail on `af5825d` and are caught; `155-23` and `264-109` still pass. The 1 m `SETUP_AT_M` gate was tried and failed the set-up test (coming through under the crow hop's speed).
- **P2-1, removed.** One try weighing the angle against the braked straight take: chosen about 3,300 times a set and his way at the take in 2–4 plays, first to third 36.0% [37.1%]. The angle from contact's code, parameters, fields and tests are gone; rounding stays.
- **P3-1, fixed** (ledger cause and a §5 lead). **P3-2, fixed in part:** `RACE_SIGMA_S` and `RACE_MIN_P` are the single source, `holder.rs` included; the runner's direction and the tag time in `outfield_round_base` are **deferred** (TASKS backlog). **P3-3, fixed. P3-4, fixed** (`SETUP_MAX_M`, `SETUP_BASE_NEAR_M`, `ROUND_KEEP_SLACK_DEG`, `SETUP_BEHIND_DEG`; the angle's literal went with the angle). **P3-6, fixed** (a fielder who stops chasing drops his approach).
- **P3-5, deferred:** the two tests that partly restate their formula stay as they are (their fixed figures are the independent part); the angle's no-ball-lost test went with the angle.
- **P3-7, deferred** (outside this branch): the fielder standing beside the dead ball after a missed catch goes to TASKS backlog.
- **Q4 (first to third), accepted as a consequence of Q32's across charge**, as the review found: 36.4% [36.3%] against MLB 31.7%, listed open in TASKS.
- **Also found while fixing:** `the_typical_spin_belief_follows_the_contact_model` failed by sampling (a 30–80-ball median in the 15° row moves by hundreds of rpm with the games' balls); its fixed 300 rpm bound is now the median's 99% order-statistic interval (`7f46ef8`). No cell excludes the table; 105 mph at 15° reads about 250 rpm above it and is the cell to refit first.
