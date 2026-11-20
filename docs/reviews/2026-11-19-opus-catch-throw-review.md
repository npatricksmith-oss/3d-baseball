# Review: the catch sets up the throw (`work/catch-throw`, owner's Q32 with Q33)

Reviewer: Opus (high effort). Scope: `ab5c374..work/catch-throw` (`93399e8` … `82a0436`; final sim commit `4f18c47`). Files: `sim/crates/bb_agents/src/throwing.rs`, `sim/crates/bb_game/src/live.rs`, `sim/crates/bb_game/src/live/holder.rs`, `sim/crates/bb_physics/src/throw.rs`, agent-models §5.5 ("The catch sets up the throw" and "…the rules"), the [catch-throw note](../calibration/2026-11-19-catch-throw.md) and the ledger row. I wrote none of it. Q32 and Q33 are not reopened here; where the build departs from them I say so.

**Summary.** The code is clean and the invariants hold: no new RNG draw, no `f64` transcendental outside `bb_math::m`, no map iteration, `BRAKE` is the same 4.75 bit for bit, and no runner reads the holder's route or target (his read of a throw is geometry and his own tables). All 472 workspace tests pass, including the five frozen plays in `throw_choices.rs`. The seed-1000 run rise is the plant, as the note says: with the old flat footwork switched back on, the final build gives 4.49 against main's 4.47. The plant is also the whole of the triples rise and most of first to third.

The problems are in the new throw and the beliefs built on it:
- **The low throw is not the one the owner described.** It passes the cutoff man where the sim puts him, the relay spot 45 m from the thrower, so on a throw home it lands 15–35 m short of the plate and reaches it rolling, after two to four bounces, at 10–20 m/s. Q32 says "low, at his head or chest, so the ball carries home on the fly or one hop." The route crossover in note §4 and the runners' and fielders' belief table are built from this rolling throw.
- **The runners' belief is applied where the world almost never throws it, and it, not the plant, is what raised tags from second.** Every runner reads every outfield throw to third or home from 50 m or more as the slower cuttable throw (`cut_on`). In the worker's 600 traced games the world threw it 7 times, all home, and never to third. At 70–80 m that belief is 0.12–0.28 s slower than the throw that actually comes. With it switched off, seed 1000's tags from second fall back from 22.3% to 15.4% (main 16.9%) and scoring from third from 64.0% to 59.9% (main 61.5%). With the plant switched off they do not (23.8%, 66.7%). The ledger and note credit the rise in tags from second to the plant.
- **Outfielders running down a ball going away from the infield now pay a full stop from full speed.** Pickup-to-release at 5–7 m/s away went from 1.09 s to 1.69 s, and triples rose about 20% on a sim already about 7× MLB's triple rate. It is the same movement gap Q33 deferred the across term for, and it applies to away motion too.

## Findings

### P1-1. Runners believe a cuttable throw on nearly every outfield throw to third or home; the world almost never makes one, and that belief is what raised tags from second

`live.rs:263` (`cut_on`), used at `live.rs:2713`, `2720`, `4788`, `4826`.

- **What happens.** `cut_on(from, b)` is true for any ball 50 m or more out thrown to third or home. The world throws the cuttable throw only when a cutoff man with the job for that base stands within 5 m of the line (`cutoff_out`, `live.rs:5919`). That requires:
  - `relay_job` (`live.rs:3710`) lines up for one base only, the lead runner's. With a runner on second or third that is home, so nobody is ever lined up for a throw to third.
  - the man must have reached the line by the throw, and on a fly ball he is usually still playing the ball when it is caught.
- **What the world did** (the worker's trace of the final build, 600 games, the route table in note §5): outfield first throws to third, 0 cuttable of 168; to home, 7 cuttable of 22.
- **How wrong the belief is.** At 70 m and 80 m the cut table is 0.12 and 0.28 s slower than the plain throw at 37 m/s, and 0.15 and 0.25 s at 34 m/s (`zz_cut` section C). Every runner deciding to take third or home on an outfield ball believes he has that much more time than the throw that will come.
- **What it moves.** Seed 1000, 300 games, the worker's `zz_ct_probe` on the review build with `cut_on` forced false for runners (`ZZ_CUTON_OFF`) and, separately, with the old flat footwork (`ZZ_OLD_FOOT`):

  | Seeds 1000–1299 | Main | Final | Final, old footwork | Final, no cut belief |
  |---|---:|---:|---:|---:|
  | Runner on second, third empty, outfield catch: advanced | 16.9% | 22.3% | 23.8% | 15.4% |
  | Runner on third, outfield catch, 0–1 out: scored | 61.5% | 64.0% | 66.7% | 59.9% |
  | Singles, first to third | 32.9% | 34.4% | 32.6% | 33.4% |
  | Triples per team-game | 0.837 | 1.020 | 0.842 | 0.988 |
  | Outfield assists per team-game | 0.263 | 0.328 | 0.312 | 0.303 |
  | Runs per team-game (probe) | 4.36 | 4.47 | 4.34 | 4.45 |

  Main and final are the worker's `base_1000.out` and `q33_1000.out`. My `sim-cli` run of the same build with every switch off reproduces the ledger's 4.60, and these are the same source.
  - So the rise in tags from second (and on this seed set, all of the rise in scoring from third) comes from the runners' belief, not the plant. With the plant gone they stay up; with the belief gone they fall to main's level or below. Runners put out going stay at 0–1 in every column, so the extra runners were safe. The belief happens to push the tag rates toward MLB (30% from second), but for a reason that is false in the world: the throw to third is the plain one.
  - The ledger row's cause column and note §5 ("Tags from second rise 14% → 21%: outfielders who catch drifting back toward the wall now owe the stop") have the cause wrong. Run totals: runs 4.60 → 4.55 with the belief off, within noise.
  - The extra outfield assists (0.263 → 0.328) are split between the two and within about 2σ of either column (counts 158–197).
- **Why it is P1.** It breaks the spec's own rule for this change ("each follows the truth where a player could know it"), and it produces a headline improvement the record credits to something else. Fixing it will take tags from second back to about 15% and scoring from third to about 60% unless P2-1's one-hop throw or another change lifts them.
- **A runner can know the convention.** The lead runner sets the relay job, and anyone can see whether a man is out on the line. So this is a belief that does not follow the truth where a player could know it, against the spec's own "What the players know" rule.
- **Suggested fix.** Make `cut_on` a `Sim` method that is true only when the base is the one `relay_job` lines up for and a man with that job is visible within `CUT_LINE_M` of the line. Better, judge it from what the runner sees of the men on the field. With P2-1 fixed as well, the cuttable throw is the one-hop throw, and the belief error shrinks to 0.02–0.06 s whichever way the runner's guess falls.
- **Also.** The appeal-throw read in `catch_hold_s` (`live.rs:2720`) applies `cut_on(read.at, r.start)` to the throw back to third with a runner on third. With that runner the relay job is toward home, never third.

### P2-1. The "low throw through the cutoff man" reaches home as a roller, not on the fly or one hop

`live.rs:242` (`cut_spot`), `live.rs:5947` (`throw_to`), `throw.rs:161` (`cut_elev`), `throw.rs:256` (`ThrowTable::build_cut`).

- **What happens.** The cuttable throw must pass the cutoff man at no more than 2.0 m where he stands. For any throw longer than 59 m that is `relay_spot`, 45 m from the thrower (`RELAY_OF_THROW_M`). With the throw and bounce physics (review probe `zz_cut`, section A), every throw home of 70–88 m at 34–40 m/s:
  - leaves at the same 7.2° (passing 2.0 m at 45 m out fixes the angle);
  - first lands 53–57 m from the thrower, so 13–35 m short of the plate;
  - bounces two to four times and reaches the plate rolling (ball centre 0.04 m up) at 10–20 m/s;
  - and arrives 0.07–0.27 s later than the plain aim.
- **In a game** (seed 175, play 179, `trace_play`): the centre fielder threw from (14, 85). The cutoff man, the second baseman, stood at (8, 37), 38 m from home. The ball touched down at 25.7 m, 17.6 m and 13.5 m from home. The catcher came out 3.7 m and fielded it as a loose ball. The runner from third scored 0.5 s later.
- **Why it matters beyond its frequency.** These throws are rare: 7 in the worker's 600 traced games. But this throw is the reason the relay wins from about 262–295 ft (note §4). It is also the table every runner and holder believes (P1-1).
- **Q32 says** "low, at his head or chest, so the ball carries home on the fly or one hop". The source is the owner's write-up, `docs/baseball-throw.md` line 47. With the cutoff man at the coaching spot instead, about 45 ft (`CUTOFF_M` = 14 m) from the plate, the same "no higher than 2.0 m at the cutoff man" rule gives that throw (`zz_cut` section E, from straight-away centre):

  | Throw | Lands short of the plate | At the plate | Plain aim | As built (relay spot) |
  |---|---:|---:|---:|---:|
  | 230 ft, 37 m/s | 6.6 m | 2.49 s, one bounce | 2.45 s | 2.57 s |
  | 260 ft, 37 m/s | 8.2 m | 2.98 s, one bounce | 2.93 s | 3.21 s |
  | 290 ft, 37 m/s | 9.4 m | 3.56 s, one bounce | 3.51 s | 3.76 s |
  | 290 ft, 34 m/s | 10.5 m | 4.07 s, two bounces | out of reach | 4.30 s |

- **Suggested fix.** Measure the cuttable throw's constraint at the cutoff spot of a throw through: `CUTOFF_M` from the base, or wherever the cutoff man of a non-deep ball stands once his position is fixed. Keep `relay_spot` for the relay man on deep balls. Then rebuild `build_cut` and redo note §4's crossover.
  - With the one-hop throw, the cuttable throw costs 0.02–0.06 s over the plain aim. The relay then wins only from about where the plain throw loses to it: about 88 m (290 ft) at 37 m/s by the same tables, deeper for stronger arms. That is still near the owner's "280+ ft".
  - Moving the cutoff man is agent-models §5.7 positioning. If the worker cannot do it under Q32's wording, it needs the owner (it is the bringing-into-line of his decided "one hop", so I would order it without a question).

### P2-2. The plant on balls run down going away: triples +20%, pickup-to-release 1.7 s at 5–7 m/s

`throwing.rs:152` (`footwork_s`); the effect, not the formula, which is Q33's option 3.

- **What happens.** The sim's outfielder reaches a ball rolling away from the infield at full speed, the same movement gap the note gives for deferring the across term (note §6). He then owes the whole stop. From the worker's own trace of the final build (`q33_*.err`, 600 games, outfield pickups, motion mainly away from the throw's target, all throws to second):

  | Away at the pickup | n | Plant | Pickup-to-release: main → final |
  |---|---:|---:|---|
  | 3–5 m/s | 1,694 | 0.35 s | 1.11 → 1.26 s |
  | 5–7 m/s | 500 | 0.80 s | 1.09 → 1.69 s |
  | 7+ m/s | 33 | 1.12 s | 1.11 → 1.99 s |

- **Triples** went from 0.89 / 0.84 to 1.04 / 1.02 a team-game, and doubles fell by about the same. MLB is roughly 0.14 a team-game (about 650–700 triples a season), so the sim is about 7× it and moves further away. The note's own §5 attributes it to "outfielders who pick up a ball running toward the wall now stop".
  - A real outfielder running down a ball in the gap slows into it as he gets to it and throws with a spin or a crow hop off the deceleration. He does not reach it at 6 m/s and then brake to a standstill.
  - The probe ablation on seed 1000 (P1-1's table): with the old flat footwork the final build's triples are 0.842 a team-game, main's 0.837. With the runners' cut belief off they stay up (0.988). So the plant is the whole of the triples rise, and most of first to third (32.9% → 34.4%, 32.6% with the old footwork).
- **Not a decision to reopen.** Q33 chose to charge motion away. But Q33 was framed as "motion across" being the movement gap. The away charge on these pickups comes from the same gap, and the ledger row does not set triples against MLB.
- **Suggested fix.**
  - Say this in the ledger row and note §5, with the MLB triple rate.
  - Make the approach job (next, Q33) cover balls run down going away (slowing into the pickup) as well as rounding the ball. Its check should include this table and the triple rate, not only Tango's 1.2 s.
  - If the owner wants an interim, the same switch that deferred the across term could cap the away plant on pickups (not fly catches) until then. That is his call: give it a Q number.

### P3-1. A runner reading a low throw in the air names the wrong base

`live.rs:4727` (`project_runner_throw`).

- **What happens.** `ThrownRead` takes the throw's arrival as the first point where it comes down through 1.3 m and names the nearest base to it. It allows one hop only for a throw that never rises above that height.
- **For the cuttable throw home** that point is about 49 m from the thrower (`zz_cut` section D). In all 15 sample lines (spray -40° to +40°, 230–290 ft) it is nearest second, first or third base, never home.
  - From straight-away centre at 290 ft it is 0.3 m from second base.
  - So a runner going home reads `time_to(4)` as a throw to second plus `RELAY_READ_S` plus second-to-home, about 1 s late.
  - A trailing runner reads the ball as coming to his own base.
- **Impact today** is small (7 such throws in 600 games). It grows if P2-1 is not fixed and the cut call makes these throws common.
- **Suggested fix.** Run the runner's read on through the bounce physics as the receiver's read now does (`receiver_read`'s hop branch). Or, with P2-1 fixed, take the dest from where the one-hop throw lands.

### P3-2. The miss cost overstates each throw's cost about twice; the crossover is shallower than the note says

`live.rs:270` (`miss_cost_s`), `holder.rs:73`.

- **The formula is right for what it says.** For a lateral miss X ~ N(0, s), E[(|X| − R)⁺] = 2s·φ(R/s) − 2R·(1 − Φ(R/s)).
- **But it then costs `run_time(E[excess])`, not E[`run_time(excess)`].** A run from a stand is concave in distance (about √(2τd/v) for short runs), so this overstates each term. At σ = 1.2° (`zz_cut` section B; the Monte Carlo is 20,000 quantiles of the same normal):

  | Throw | Formula | Monte Carlo |
  |---|---:|---:|
  | 80 m direct | 0.29 s | 0.17 s |
  | 40 + 40 m (relay) | 0.15 s | 0.04 s |

- **The difference between routes is about right**: 0.09–0.15 s either way. So the decision is not badly biased.
- **What the note and spec leave out.** The miss cost moves the crossover 2.5–7.5 m shallower than §4's time-only figures:
  - by the belief tables the relay wins from 77.5 m (254 ft) at 37 m/s and 85 m (279 ft) at 40 m/s, against 80 m and 92.5 m by time alone;
  - so the holder takes a relay up to about 0.15 s slower than the direct throw.
  - The "run from a stand" also ignores that the receiver reads the throw in flight and moves before it arrives.
- The test `the_route_is_the_relay_from_deep_and_low_through_the_cutoff_from_shallow` passes at 250 ft about a metre inside that crossover for a grade-50 arm.
- **Suggested fix.**
  - Cost E[`run_time`] (a short quadrature, or the closed form for the short-run approximation).
  - State the crossover with the cost in note §4 and agent-models §5.5.
  - Move the test's shallow cases off the knife edge (230 ft and 240 ft).

### P3-3. Tests: none pins belief to world for question 1; two restate the formula

- **Missing:**
  - (a) the cut table against the world's cuttable throw from off-centre lines and to third (my probe: home within 0.03 s at every angle; to third from centre and right of centre the belief is 0.03–0.10 s slow, so it is fine today, but nothing holds it);
  - (b) a runner's `cut_on` against whether the world would throw it (P1-1 would have failed it);
  - (c) a runner's `ThrownRead` of a low throw naming the right base (P3-1).
- **Restating the formula:**
  - `the_plant_is_the_stop_under_the_braking_limit` adds `PLANT_OVERLAP_S` back to its own output to recover 7/4.75;
  - `set_up_adds_transfer_turn_and_footwork` and `a_fielder_moving_away_needs_longer_to_get_set` recompute `(v − 2)/4.75`.
  - They check the constant, not an independent figure. The momentum test (Wagner's +1.6 m/s) and the two cutoff tests are good: a source and physical bounds.
- **Suggested fix:** add (a)–(c). Anchor the plant tests on §5.3's stop time and the 2 m/s no-charge figure without re-adding the overlap.

### P3-4. `PLANT_OVERLAP_S` decides most of what this change moves and is unsourced

- **It sets every plant.** The run change on seed 1000 is the plant (4.49 with the old rule against 4.60), and P2-2's table scales directly with it: at 5 m/s away, 0.42 s of overlap gives 0.63 s, and 0.2 s would give 0.85 s.
- **The other unsourced parameters decide few plays now:**
  - `HOP_MAX_MPS` caps the gain on the 20% of pickups charged in at 5 m/s or more. With the gain switched off, seed 1000 gives 4.67 against 4.60.
  - `CUT_LINE_M`, `CUT_HEIGHT_M` and `STEP_IN_M` touch only the rare cuttable throws and short hops.
- **Suggested fix:** tie `PLANT_OVERLAP_S` to the one empirical anchor there is, Tango's ≈1.2 s pickup-to-release plus tag (final build 1.08 + 0.25 ≈ 1.33 s). Say in the spec that it is the calibration knob of the plant, so the approach job re-checks it.

### P3-5. Q32's "drift off the line widens the direction error" is neither built nor marked deferred

agent-models §5.5, the decided paragraph's **Accuracy** bullet, against "the rules".

- The rules give no accuracy cost for drift beyond the hurried throw's existing off-balance σ. With the across term deferred, a set throw after drifting across has no accuracy cost either.
- **Suggested fix:** one line in "the rules" saying the drift accuracy term is deferred with the across term (Q33).

## Question by question

1. **Belief and world.**
   - The holder's race uses his own `catch_vel` (gain and plant), teammates' jobs through `seen`, and teammates' `ArmAccuracy`: allowed by the spec. A runner reads the plant and gain from `catch_seen_vel` and the dive from `lunge_at`, both what he sees. No runner path reads `through`, `relay`, `Ball::Thrown.to_base` or the holder's target. `cut_on` is geometry only and `read_throw_time` takes the quicker of the direct and halfway-relay routes by itself. The isolation test passes.
   - The cut table against the world's cuttable throw (radial lines home at -40°…+40°, 230–290 ft, 34–40 m/s): within 0.03 s. The park is one, the reference `Field::default()`, and the dirt circle is centred on home, so throws home are the same from any angle. Throws to third are not radial, so they cross the dirt differently: the belief is 0.03–0.10 s slow (too generous to the runner) from centre and right of centre at 260–290 ft, and 0.00–0.04 s from left field.
   - Where it biases decisions is not the table's error. It is `cut_on` applying it where no cut throw comes (P1-1) and the runner's in-flight read (P3-1).
2. **Seed 1000's +0.13.** It is the plant: the old flat footwork in the final build gives 4.49 (main 4.47). The momentum gain lowers runs (4.67 without it), the runners' cut belief adds a little (4.55 without it; 4.54 with no cut anywhere), and the receiver's hop read is neutral (4.58). All of these are single 300-game runs on one seed set, noise about 0.12. In the probe measures, triples and most of first to third come from the plant (P2-2's ablation). Tags from second and the rise in scoring from third come from the runners' cut belief (P1-1), not the plant as the ledger says. The outfield assists are split between the two and within noise of each. Two behaviours a real player would not show: P2-2's full stop from full speed, and runners counting on a throw that is not coming (P1-1).
3. **The low throw's physics.** The flight, bounce and roll are the sim's own and behave consistently: speeds after the first bounce 19–25 m/s and arrival 10–20 m/s rolling, never astray beyond the direction noise. But it is the wrong throw (P2-1). The route crossover by the sim's tables is 80 m (262 ft) at 37 m/s and 92.5 m (303 ft) at 40 m/s by time alone, and 254–279 ft with the miss cost (P3-2). With the one-hop throw it would be about 290 ft and deeper.
4. **The miss cost.** The formula is right for the expected excess. Costing the run of the mean excess overstates each term about twice, but the difference between the routes is about right. It makes the holder take a relay up to about 0.15 s slower, not clearly slower (P3-2).
5. **Determinism and invariants.** Confirmed:
   - no `Purpose`, `rng(` or `KeyedRng` in the added lines; the receiver's hop read and `cut_throw_time` use zero tilts;
   - no `.exp/.sin/.powf/mul_add` in sim code (`miss_cost_s` uses `m::exp`, `m::sqrt`; `phi` uses `m::erf`);
   - no hash maps;
   - `const BRAKE: f64 = throwing::BRAKE_MPS2` is the same literal 4.75, and at exactly 2 m/s away the plant is 2/4.75 − 2/4.75 = 0.0 exactly;
   - the tables are built once (`OnceLock`), deterministically;
   - my review build with every switch off reproduced seed 1000's 4.60.
6. **Tests.** The workspace passes (472 tests, 0 failed), including all five plays in `throw_choices.rs` (seeds 1/64, 23/109, 13/140, 3/140, 17/203) under Q33. Missing tests and two that restate the formula: P3-3.
7. **Unsourced parameters.** `PLANT_OVERLAP_S` deserves a source or an explicit calibration anchor first (P3-4). `CUT_HEIGHT_M` will matter once P2-1 is fixed and the cut call is built.

## Verification

All built in a copy of the branch's `sim/` at `82a0436` under `sim/target/review-probes/ct/tree`, target `sim/target/review-probes/ct/target`. Runs used `CARGO_BUILD_JOBS=2` and `taskset -c 0-7 nice`, one measurement at a time.

- **`cargo test --workspace --no-fail-fast`** in the worktree after touching the four changed sources: 472 passed, 0 failed (10 m 43 s). `throw_choices.rs`: 5 of 5.
- **Switches for ablation** (throwaway, in the copy only; every one off reproduces the branch). Env vars:
  - `ZZ_OLD_FOOT`: the old 0.2 s footwork;
  - `ZZ_NO_GAIN`;
  - `ZZ_CUTON_OFF`: runners never believe a cuttable throw;
  - `ZZ_CUTON_HOMEONLY`;
  - `ZZ_NO_CUTWORLD`: no cuttable throw in the world;
  - `ZZ_NO_HOP`: no hop read.
- **`sim-cli games --n 300 --seed 1000`**, runs per team-game:

  | Build | Runs | Reached on BIP |
  |---|---:|---:|
  | none (the branch) | 4.60 | 33.4% |
  | `ZZ_OLD_FOOT` | 4.49 | 33.4% |
  | `ZZ_NO_GAIN` | 4.67 | 33.5% |
  | `ZZ_CUTON_OFF` | 4.55 | 33.4% |
  | `ZZ_CUTON_HOMEONLY` | 4.59 | 33.3% |
  | no cut anywhere | 4.54 | 33.3% |
  | `ZZ_NO_HOP` | 4.58 | 33.5% |

  Main is 4.47 (ledger). K% 22.2–22.3 throughout.
- **The worker's `zz_ct_probe`**, seeds 1000–1299, single-threaded, run one after the other on the review build with `ZZ_OLD_FOOT` and with `ZZ_CUTON_OFF`, against the worker's own main and final outputs: P1-1's table.
- **`zz_cut`** (review probe, `bb_physics` only), sections A–E:
  - A: the world's cuttable throw against `build_cut` by spray angle, depth and speed, to home and to third, with arrival speed, height, landing point and bounces;
  - B: route crossover and the miss cost against a Monte Carlo;
  - C: cut belief against the plain throw;
  - D: where a runner's read puts a low throw's arrival;
  - E: the cuttable throw with the cutoff man 14 m from the plate.
- **`trace_play 175 179`**: the cuttable throw home in a game (P2-1). The worker's trace files (`sim/target/codex-probes/catchthrow/q33_*.err`, `base_*.err`) for P2-2's pickup table and the route counts.
- **The relevant code read in full:** the diff, `throw_race`, `throw_to`, `cutoff_out`, `receiver_read`, `update_thrown`, `project_runner_throw`, `relay_job`, `catch_hold_s` and `release_time`.

## Resolution

Open.
