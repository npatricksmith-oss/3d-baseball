# Re-review: the fixes to the catch-throw review (`work/catch-throw`, `dc0a539..826120a`)

Reviewer: Opus. Scope: `dc0a539..826120a` (sim commits `a65ad33`, `409d16e`, `55402ee`; the frozen fixture `46afacc`; docs `ade3604`, `826120a`), judged against the first review's findings ([2026-11-19 review](2026-11-19-opus-catch-throw-review.md)). I wrote none of it. Q32 and Q33 are decided and not reopened. What the first review passed is not re-reviewed.

**Summary.** Most of the findings are fixed:
- **P1-1 (the runners' cut belief): fixed.** A runner now believes a cuttable throw only where a man with the relay job for that base stands on its line, the same predicate the holder's throw uses. Tags from second on seed 1000 are back to 15.4%, the first review's ablation figure exactly (main 16.9%).
- **P2-1 (the rolling throw): fixed in the physics.** Through a cutoff man 14 m out the throw lands 2–10 m short and reaches the plate on one hop at 37–40 m/s, out to 90 m.
- **P3-2 (the miss cost): fixed.** It is deterministic and matches a 20,000-point Monte Carlo to within 0.001 s.
- **P3-3, P3-4, P3-5 (tests, the plant's knob, drift accuracy): fixed**, apart from the two test gaps below.

The fix for P3-1 (the runners' read of a low throw) has two faults:
- **It misnames the base.** When a runner's noisy read of an ordinary throw comes down short, the new read follows the throw on and names the first bag it passes within 2.5 m. A read that passes the target bag a little wide is followed on to the next bag along the line. Read 0.3 s after release, a plain throw from centre field to second is read as a throw home 11% of the time (the old read: 0.7%). On average a batter rounding second then believes the ball reaches third 0.18 s later than it does (old read: 0.03 s). Switched off in 300 games on each seed set, it takes away 0.07 triples and 0.05 outfield assists a team-game on both sets. That is the part of the rise the ledger leaves unexplained (P2-1).
- **It misses the throw it was written for.** On the throw through a cutoff man, the man stands 2.4–3.6 m from where the ball comes down through catching height. That is inside `LOW_THROW_MAN_M` (4 m), so the runner reads the ball at the plate 0.5–0.75 s before it gets there. The test for the read leaves the cutoff man off the field.

## Findings

### P2-1. The runners' new read of a low throw names a base beyond the target when a noisy read passes the bag wide, and makes them bolder on ordinary throws

`live.rs:4796–4815` (`project_runner_throw`, the follow-on branch). Constants: `LOW_THROW_READ_M` (`live.rs:218`) and the use of `PASS_REACH_M` in `reached`.

- **What happens.** The branch fires whenever a runner's read of a throw comes down through catching height more than 8 m from every bag with no fielder within 4 m. For the throw it was written for, it almost never fires (P3-1). It does fire on ordinary plain throws, because a runner's read of a throw early in its flight is noisy (`READ_VEL_FRAC` 4% of the speed, ×1.5 along his line of sight, and a vertical velocity error of about 0.7 m/s at release). A flat 30–50 m throw to second read with that error often comes down 8 m or more short in the runner's projection.
  - The read then follows the projected ball along the ground. It takes the first bag the ball passes within `PASS_REACH_M` (2.5 m) as the destination.
  - If the projection passes the target bag 2.6 m wide or more than 3 m up, it goes on to the next bag along the line: home for a throw to second from centre, first from left-centre, third from right-centre.
  - The old read named the bag nearest where the projection comes down, which for these reads is nearly always the target.
- **What it does to a runner's read** (review probe `zz_probe_d2`). Plain throws at 37 m/s, 1,000 read-error draws each, sized from the runner's own position, compared with the noise-free read of the same throw.
  - "Wrong base" is how many of the 1,000 reads name a base other than the target. "Late" is the mean error of the runner's believed time to the next base, and in brackets how many reads are more than 0.5 s late there.
  - Each cell reads new / old.

  | Throw, runner | Read at | Wrong base (of 1,000) | Late at the next base |
  |---|---|---:|---:|
  | CF (0, 70) → second, batter rounding second | release | 217 / 135 | +0.18 / +0.05 s (162 / 73) |
  | | +0.3 s | 130 / 23 (home 114 / 7) | +0.18 / +0.03 s (119 / 12) |
  | | +0.6 s | 45 / 0 | +0.07 / +0.01 s (45 / 0) |
  | CF (0, 90) → second | +0.3 s | 161 / 61 (home 121 / 21) | +0.18 / −0.04 s (125 / 10) |
  | | +0.6 s | 90 / 6 | +0.16 / −0.01 s (87 / 1) |
  | LCF (−50, 70) → second | +0.3 s | 184 / 129 (first) | +0.18 / +0.04 s (182 / 103) |
  | RCF (50, 70) → second | +0.3 s | 144 / 70 (third) | time to second +0.37 / +0.09 s |
  | RF (45, 60) → third, runner from first reads home | +0.3 s | 41 / 52 | +0.06 / +0.02 s (42 / 5) |
  | CF (0, 75) → home | +0.3 s | 75 / 76 | +0.14 / +0.11 s (109 / 80) |

  - On throws to second the new read is 0.1–0.2 s later on average than the old one, and puts 10–15% of reads more than half a second late.
  - On throws to third and home the effect is smaller, 0.02–0.05 s.
  - It also leans late on throws that really are short. At 6% slow, 2σ of the execution noise, it is 0.04–0.16 s later than the old read. It follows them to the bag on the hop, while the receiver steps in up to 3 m (`STEP_IN_M`) to take them.
- **Would a real runner do this?** No. A runner who sees a throw coming to second, with a man covering the bag, does not decide it is going home because it looks a couple of metres wide. The base the throw is going to is the one it is heading for. A 2.5 m miss is within the receiver's reach and step.
- **What it moves in games.** Ablation with the branch switched off (`ZZ_NO_LOWREAD`; everything else is the final build), against the worker's run of the final build, 300 games each:

  | Per team-game (zz_ct_probe) | Main | Final build | Final, read switched off |
  |---|---:|---:|---:|
  | Triples, seeds 0–299 | 0.890 | 1.098 | 1.028 |
  | Triples, seeds 1000–1299 | 0.837 | 1.088 | 1.015 |
  | Outfield assists, seeds 0–299 | 0.240 | 0.300 | 0.250 |
  | Outfield assists, seeds 1000–1299 | 0.263 | 0.372 | 0.320 |
  | Runner on third, outfield catch, 0–1 out: scored (0 / 1000) | 62.8% / 61.5% | 60.5% / 62.2% | 63.2% / 64.6% |
  | Runs (probe; 0 / 1000) | 4.24 / 4.36 | 4.28 / 4.42 | 4.29 / 4.45 |

  - **The read adds the same on both seed sets:** +0.07 triples and +0.05 outfield assists a team-game. Pooled, that is 1,312 against 1,226 triples (+7%) and 403 against 342 assists (+18%).
  - That is the part of the rise over the Q33 build that the ledger row leaves "not explained", within noise.
  - With the read off, seed 0's assists are back at main's level. Scoring from third is 2.4–2.7 points higher on both sets with the read off.
  - Final and main are the worker's `fin2_*.out` and `base_*.out`. "Read off" on seed 0 is my run. On seed 1000 it is the worker's `abl_ZZ_NO_P31.out`: built at `409d16e`, whose only sim difference from `55402ee` is inside the branch the switch turns off.

  The worker's note found this on seed 1000 for the first, 3 m version. At 8 m it called the gap "within 1.5σ"; the second seed set shows the same gap.
- **Suggested fix.** Name the destination from the direction of the throw, not from the 2.5 m catch radius. The first bag the projected track passes nearest after it comes down (the first local minimum of distance to a bag) is the one it is going to, at whatever distance, with the time of that closest pass. Treat a read that comes down short of its nearest bag on the throw's line as a throw to that bag, arriving on the hop.
  - Then cover the cutoff man (P3-1) by letting a man standing short of where it comes down, on its line, not stop the follow-on. Only a man the ball comes down at (a relay reception) should.
  - Re-measure triples and outfield assists on both seed sets. If the misnaming goes, the "not explained" part of the ledger row should go with it.

### P3-1. The P3-1 read misses its own case: with the cutoff man at his spot, a runner reads the throw through him at the plate 0.5–0.75 s early

`live.rs:4796` (`to_a_man`), `live.rs:218–221`, test `a_runner_reads_a_low_throw_to_the_base_it_is_going_to` (`live.rs:8859`).

- **What happens.** The throw through a cutoff man 14 m out passes him at 2.0 m and comes down through catching height 2.4–3.6 m beyond him (10–12 m short of the base). That is inside `LOW_THROW_MAN_M` = 4 m of the man, so `to_a_man` is true and the follow-on never runs. The read falls back to the nearest bag (the right one, home or third) at the time it comes down through 1.3 m. That is 0.5–0.75 s before the ball reaches the bag on its hop, and 0.36–0.57 s before the read with nobody near.
- **Probe** (`zz_probe_c`, runner's noise-free read against the world's throw along the line; times from release):

  | Throw | World, at the base | Read, nobody near | Read, cutoff man at his spot |
  |---|---:|---:|---:|
  | Home from (0, 75), 37 m/s | 2.74 s | 2.60 s | 2.16 s |
  | Home from (±45, 60), 34 m/s | 3.10 | 2.93 | 2.36 |
  | Home from (0, 66), 34 m/s | 2.56 | 2.42 | 1.96 |
  | Third from (45, 60), 37 m/s | 2.80 | 2.66 | 2.20 |
  | Third from (0, 75), 34 m/s | 2.17 | 2.05 | 1.68 |

  In a few cases (40 m/s from 66 m, the shorter throws to third) the ball comes down 7–12 m from him and the follow-on runs. With nobody near the read is 0.12–0.17 s early: the 2.5 m catch radius plus `PASS_CATCH_S`, about where the catcher takes it.
- **The test** puts the throw through an empty field: nobody stands at the cutoff spot, so it passes. In a game the man is always there, since he is the reason for the throw.
- **Impact** is small today: 6 throws home through a cutoff man in 600 traced games, and runners err toward caution. The spec's line on `LOW_THROW_READ_M` ("the throw through a cutoff man 45 ft out comes down 10–12 m short") gives this throw as the rule's reason.
- **Suggested fix.** With P2-1's fix the man short of the landing point no longer stops the follow-on. Put the cutoff man in the test.

### P3-2. The relay man goes out at a fixed 80 m whatever the thrower's arm: up to 0.2 s lost against the faster route for arms away from grade 50

`live.rs:694` (`RELAY_DEEP_M`), `live.rs:3785` (`relay_spot`), `relay_job`.

- **The physics crossover** (review probe `zz_probe_a`, A2, with the miss cost at σ 1.2°; it matches note §4) is about 74–75 m (243–246 ft) at 34 m/s, 82–84 m (269–276 ft) at 37 m/s and 92 m (302 ft) at 40 m/s.
- **What the game does.**
  - Up to 80 m the cutoff man stands 14 m out. A relay through him is slow, so the holder throws through him.
  - Past 80 m the relay man stands at the relay spot. A throw past him rolls in, so the holder relays.
- **Time lost against the better route** (time plus miss cost, throw home from centre):

  | Depth | 34 m/s | 37 m/s | 40 m/s |
  |---|---:|---:|---:|
  | 75–80 m (246–262 ft) | 0.03–0.23 s | 0 | 0 |
  | 82–88 m (269–289 ft) | 0 | 0 | 0.11–0.20 s |
  | 90 m (295 ft) | 0 | 0 | 0.07 s |

- **Verdict.** For a grade-50 arm the 80 m switch is within about 0.05 s of the crossover, so it is consistent with the spec's 276 ft. For a strong arm between 269 and 295 ft, the depth of the tag-ups the throw-race note is about, the game relays where a throw through the cutoff man is 0.1–0.2 s quicker. Runners' beliefs follow the world, so nobody is misled. The defence is slower than it would be.
- **Suggested fix.** Positioning, agent-models §5.7. Decide who goes out from the outfielder's arm, which his teammates know as well as runners do (scouting), using the crossover tables already built. Or record in §5.7 that the depth is fixed at grade 50's crossover and costs strong and weak arms up to 0.2 s.

### P3-3. The throw is kept low at the cutoff man's position at the release, usually before he reaches his spot; then it rolls in

`live.rs:6046` (`throw_to`: `cutoff_out` at the release), with `relay_job`'s "once the ball is down" start (§5.7, unchanged).

- **What happens.** The world's throw must pass ≤2.0 m where the man stands when the throw is released. In the three traced throws through him (final build, `trace_play`) he was still on his way in:
  - 13/157: about 25 m from home, 4.7 m off the line;
  - 32/54: about 22 m;
  - 1094/100: about 18 m.

  His spot is 14 m out. The job starts when the ball is down, he shows it `CALL_HEARD_S` later, and the outfielder releases about 0.9–1.1 s after the pickup.
- **What the throw does then.**
  - On 13/157 it landed 15.1 m short and reached the catcher after two bounces.
  - By the physics (`zz_probe_a`, A1), passing a man 25 m out gives 2–4 bounces, reaching the plate rolling (ball centre 0.04–0.06 m up) at 11–22 m/s. That is the rolling throw P2-1 was about.
  - `Cut::of` puts a man 25 m out on a 70–80 m throw nearer the relay spot, so the holder's and runners' belief takes the slow table. Belief and world agree; the throw is not the owner's one-hop throw.
- **Suggested fix.** Two options:
  - Keep the throw low where he will be: his shown target (`relay_spot`), or the nearer of that and where he is.
  - Send the cutoff man when the ball is through the infield, not when it is down. That is §5.7's.

  Either way, add the in-game share of throws through a cutoff man that reach the plate on the fly or one hop to note §5's route table.

### P3-4. Tests: the cut-table test is loose and compares the physics with itself; P3-1's test leaves out the cutoff man

`live.rs:8791` (`the_cut_tables_match_the_worlds_cuttable_throw`), `live.rs:8859`.

- **What the test compares.** Both sides are `bb_physics` (the table is `cut_throw_time` from straight-away centre, now `cut_throw_time_along` on that line). At 0° it is the table's interpolation against itself; off centre it adds the dirt and grass crossing. It does not fly the Sim's own throw (`throw_to`: the release point offset by handedness, the man's real position along and off the line, effort, the receiver).
- **The tolerances cannot tell the tables apart.**
  - The real agreement is 0.009 s home and 0.047 s to third (`zz_probe_e`), against tolerances of 0.06 and 0.12 s.
  - The throw through costs only 0.02–0.06 s over the plain aim, so the home test would pass with the plain table at most points.
  - To third the plain table is closer to the world's throw through (0.015 s) than the through table is (0.047 s). The belief built from throws home makes a throw to third through a cutoff man about 0.05 s slower than it is. That is minor: almost no such throws are made.
- **Suggested fix.**
  - Tighten home to about 0.02 s, and to third use a bound that rejects the plain table, or build the through table for third separately.
  - Add one case through `throw_to` with the cutoff man placed off the spot, flown with zero bounce tilt, so the test is against the world's launch.
  - P3-1's test: put the man at `through_spot`.
- **The rest of the test changes are good.**
  - `a_fielder_moving_away_needs_longer_to_get_set` is anchored on the movement law's own stop (`law_stop_s`), which is independent.
  - `the_plant_is_the_stop_under_the_braking_limit` now uses §5.3's 1.47 s and the 2 m/s point. That is acceptable, though algebraically it is still (v − 2)/`BRAKE`.
  - `a_runners_cut_belief_follows_the_throw_the_world_would_make` checks the relay-job convention, and the man not yet on the line, against `cutoff_out`. It would have failed on P1-1.

### P3-5. Records: the ledger credits the outfield-assist rise to the plant; the hop read's share is now measured

`docs/metrics-ledger.md` (the catch-throw row, Cause and Open columns), note §5 "What moved and why", agent-models §5.5 (the `LOW_THROW_READ_M` bullet).

- **The ledger's cause for outfield assists.** It reads "more batters stretch hits into triples, more runners go first to third, and more are thrown out trying". P1-1's ablation table showed the plant moved assists little on seed 1000 (0.328 → 0.312 with the old footwork). The hop read moved them 0.320 → 0.372 there, and 0.250 → 0.300 on seed 0 (P2-1).
- **What to change in the records.**
  - Once P2-1 is fixed, re-measure and drop the "not explained" line, or move it to ledger §5 with the ablation if it is kept.
  - The spec bullet should say the read's motivating case is not covered (P3-1) until it is fixed.
- **Credited correctly.** The ledger and note now credit the tags-from-second rise to P1-1, give P2-2's pickup table and set triples against MLB.

## Question by question

1. **Invariant 2 in the runners' cut belief.**
   - **What `man_in_throw(None, …)` reads.** It reads each fielder's job as he showed it `CALL_HEARD_S` ago (`known_job`) and where the fielders stand. It reads nothing of the holder's route choice, target, `through`/`relay` flags or `Ball::Thrown.to_base`.
   - **Who calls it.** `observer: None` means "what anyone on the field can see". Only runner code calls it so: `catch_hold_s` (twice), `ball_time_to` for a held ball, and `pickup_time_to`. The holder (`throw_race`, the effort choice) and the world (`throw_to` through `cutoff_out`) use `Some(by)`, his own knowledge through `seen`. That is the same predicate with the same lag, so belief and world name the same man.
   - **The outcome.** The worker's final-build probe on seed 1000 gives tags from second at 46 of 298 = 15.4%, the first review's no-cut-belief ablation exactly (main 16.9%). Fixed.
2. **The one-hop throw.**
   - **The physics.** Through a man 14 m out, 60–90 m throws home land 2.4–10.7 m short. They reach the plate after one bounce at 37 and 40 m/s, 14–23 m/s and 0.07–0.41 m up. At 34 m/s from 80 m and more it is two bounces at 12–13 m/s.
   - **In games.** Two of the three traced throws were one-hop. The third rolled in because the cutoff man had not reached his spot (P3-3).
   - **The table test** checks the physics against itself, loosely (P3-4).
   - **The 80 m switch** is consistent with grade 50's crossover and costs other arms up to 0.2 s (P3-2).
3. **The runners' read of a low throw.**
   - `LOW_THROW_READ_M` and `LOW_THROW_MAN_M` are reasoned, not sourced, and marked [E].
   - The rule never fires on the throw through a cutoff man it was written for (P3-1).
   - Where it does fire, on noisy reads of plain throws, it names the base beyond the target on throws to second from all three fields: home from centre, first from left-centre, third from right-centre.
   - That is not a believable runner behaviour, and it moves triples and outfield assists (P2-1).
4. **The miss-cost quadrature.**
   - **Deterministic.** It uses `m::exp`/`m::sqrt`, a fixed 48 nodes, an in-order sum and no map. Two calls are bit-identical.
   - **Against Monte Carlo** (20,000 quantiles, `run_time` of each excess):

     | Throw | Quadrature | Monte Carlo |
     |---|---:|---:|
     | 80 m direct | 0.1693 s | 0.1684 s |
     | 40 + 40 m relay | 0.0387 s | 0.0383 s |
     | 60 m | 0.0852 s | 0.0847 s |
     | 100 m | 0.2552 s | 0.2541 s |

     That matches the first review's 0.17 s and 0.04 s.
5. **Tests.**
   - P3-4 covers the cut-table test and the missing cutoff man.
   - **The frozen saved play `65147-10`.**
     - *Why it was frozen.* The old test replayed seed 65147 from the first pitch. The review fixes changed earlier plays, so play 10 was no longer the same ball.
     - *What it checks.* The fixture holds the contact from `4f18c47`, where the whole-game replay still reached this ball. It shows a chopper that touches down 1.7 m in front of the plate, skips through the right side and is still bouncing 50 m out. On this build the right fielder secures it at (29.4, 89.3), 94 m out, 4.67 s after contact. The assertion (right fielder first, inside 100 m and 6 s) checks what the owner's play was about.
     - *Two caveats.* It is reconstructed with average fielders and 8.2 m/s runners, as the fixture README says. Nothing shows the freeze would fail on the pre-fix behaviour: I did not build the pre-fix commit as a negative control.
   - All seven `throw_choices` tests pass on this source (five frozen infield plays, the frozen saved play, and my throwaway print of it).
6. **The records.** P1-1 is now credited correctly in the ledger row, note §5 and the spec. The assists cause and the "not explained" triples are P3-5 and P2-1. Nothing else needs a ledger §4 entry. Ledger §5 already carries the hop read as a suspect; it should carry the ablation once the read is fixed.

## Verification

All in a copy of the branch's `sim/` at `826120a` under `sim/target/review-probes/ctfix/tree`, target `sim/target/review-probes/ctfix/target`, `CARGO_BUILD_JOBS=2` and `taskset -c 0-7 nice`, one measurement at a time.

- **Throwaway switches in the copy only:**
  - `ZZ_LOW` (thread-local) and `ZZ_NO_LOWREAD` (env): the follow-on threshold, off = the old nearest-bag read;
  - `ZZ_NOMAN`: ignore `to_a_man`.
- **Review probes** (`#[test] zz_probe_*` appended to the copy's `live.rs` tests, release profile):
  - **A.** The world's cuttable throw home from 60–90 m at 34/37/40 m/s with the man at 14, 25 and 35 m: landing short of the plate, bounces, speed and height at the plate. Then the three routes with miss cost, the best against what the game's 80 m switch picks.
  - **B.** `miss_cost_s` against a 20,000-quantile Monte Carlo of E[`run_time`((|X| − 1.5)⁺)], and a bit-identity check.
  - **C.** A runner's noise-free read of the through throw home and to third: nobody near, the cutoff man at `through_spot`, the man ignored, and home positions. Against `cut_throw_time_along`.
  - **D, D2.** Runners' noisy reads of plain throws to second, third and home from six spots. 400 and 1,000 draws through `noisy_ball_motion`, sized as `runner_throw_observation`, at 0, 0.3 and 0.6 s after release. Compared: the base named and the believed time to the target and the next base, new read against the old, and the noise-free read of the same throw. (Probe D's absolute biases carry a 0.6 s offset from a stale `sim.t`; only its new-minus-old differences are used. D2 is correct.)
  - **E.** The cut table against `cut_throw_time_along` over the test's grid, and the plain table against the same.
- **`trace_play`** (release build of the copy) on the final build's three throws home through a cutoff man found in the worker's `fin2_*.err` traces: 13/157, 32/54, 1094/100.
- **`cargo test --release -p bb_game --test throw_choices`** in the copy: 7 passed, with a throwaway print of the frozen `65147-10` replay.
- **Ablation** with `zz_ct_probe` (the worker's probe, copied) built from the copy, seeds 0–299: `ZZ_NO_LOWREAD=1`, then with no switch. The no-switch run is byte-identical to the worker's `fin2_0.out`, so the copy and the switch's off state are the final build. Seed 1000 with the read off is the worker's `abl_ZZ_NO_P31.out`: built at `409d16e`, whose only sim difference from `55402ee` is inside the switched-off branch.
- **Read in full:** the sim diff `dc0a539..826120a`, `man_in_throw`, `cut_seen`, `Cut::of`, `relay_job`, `relay_spot`, `update_relay`, `throw_race`, `throw_to`, `project_runner_throw`, `ThrownRead::time_to`, the new tests, the frozen fixture and its README entry, and the spec, note and ledger changes.
- I did not rerun the workspace tests or the standard runs, as the job says.

## Resolution

Open.
