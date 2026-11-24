# Review: the ground-ball dive, fix round (rev-of-dive-2, 2026-11-24)

Reviewer: `rev-of-dive-2` (Claude Opus, high effort, Code Mode). I wrote none of this change. Range `ff6ea48..work/of-ground-dive` (`ef1e83b`, `4d4ff4b`, `7715575`, `0da2212`). Spec: agent-models §5.3, "What a dive at a ball on the ground costs him". Note: `docs/calibration/2026-11-23-of-ground-dive.md` §7–§8. First review: `2026-11-23-rev-of-dive-of-ground-dive-review.md`.

**Verdict.** `dive_pays` keeps invariants 2 and 3, and its three outcome times and probabilities are right. Tests pass (569/0/4), and the ledger's seed-0 and seed-1000 figures reproduce exactly. Against main's rule the round is a net gain for the defence: in the paired probe, 146 new outfield ground dives make the play 90 times better and 5 times worse. Ground-ball hits do not rise.

Two new defects cost real outs, though. **Rule (3), the short-hop rule, drops diving catches in the air (P2-1):** pop-ups and liners that main's fielders caught diving now fall, about 10 plays made worse in 600 games. **As a ball reaches a fielder, `dive_pays` stops weighing the dive he is about to make (P2-2):** for example, a pitcher who used to lay out at a chopper up the middle and get the out at first now stays up. Both are fixable inside `dive_pays` without a new parameter. I would fix both before merging, because P2-1 turns outs into hits on the plainest play in the sport, a catchable pop-up.

## Findings

### P2-1. Rule (3) lets catchable pop-ups and liners drop: its air/ground split misfires and flickers (`live.rs` `dive_pays`, the `ball_still_up() && dive.map_or(…)` test; agent-models §5.3 last sentences)

Rule (3) weighs a ball still in the air as a ground ball when his plan's first dive point comes after the first bounce by his read. On the ground, `dive_pays` compares only times to have the ball, so it gives no value to the out that a catch in the air makes. The test misfires in two ways.

- **The plan's dive point is not the capture's envelope.** The plan schedules dives only to `REACH + 0.6·DIVE` = 1.9 m, but the capture lays out to 2.5 m. A liner or pop-up passing 1.9–2.5 m from him, still in the air, has a plan dive point only after it has come down, when it is nearer. So rule (3) calls it a short hop. This is the same gap rule (2) closes on the ground. Of the 28 lost air catches below, 22 were taken in main at more than 1.9 m.
- **The split flickers between reads, and the must-dive flag must hold without a break.** Seed 106 play 191 (third baseman, a 51° pop-up, foul): his reads go `ground → false` (2.31–3.48 s), `in the air → true` (3.59–3.94 s), then `ground → false` at 4.04 s, 0.02 s before it lands 1.54 m from him. The flag never holds long enough for the lunge, and the ball drops. In main he dives and catches it at 4.06 s. Seed 1043 play 201 (second baseman, a 33 m/s liner): `ground` at 1.17 s breaks the hold. He has only 0.2 s of lunge at 1.49 s, the ball passes 2.20 m away, and it is a double with a run instead of an out.

Measured (paired probe, seeds 0–299 and 1000–1299; "lost" = the other rule made a diving catch in the air on the first differing play and this branch made no air dive):
- Against the branch without rule (3) (mode 2): 28 air catches lost, 13 of those plays worse.
- Against main's rule: 24 lost, 10 worse. Six of the ten are pop-ups or fly balls (seeds 8/75, 26/199, 106/191, 130/59, 175/20 at 43–67°; four are foul pops that drop with nobody near), two are hard liners at the middle infielders (78/74, 1043/201), and two are outfield liners (224/189, 1227/25).

Rule (3) is still a net gain (against mode 2: 116 changed plays, 30 better, 19 worse, 12 more outs). It removes many fruitless infield dives at balls still up, for example the pitcher's (+5/−0, 5 more outs) and the first baseman's (+5/−2). Would a real player do this? A first baseman lets a foul pop drop 2 m from him rather than lay out: no.

**Suggested:** keep a ball still in the air on the in-air rule whenever his read has it pass within the capture's envelope (`REACH + DIVE`, at or below `DIVE_MAX_Z`) before its first bounce. Only a dive his read can make nowhere but after the bounce is a ground dive. That uses existing constants, removes the flicker in most cases, and keeps the short-hop gain. Re-run the pair against mode 2.

### P2-2. As the ball arrives, the dive weighed is a later one, and the hold breaks just before the lunge (`dive_pays` → `intercept_scan`)

`intercept_scan` scans from `self.t + 0.05` and from the planning start. As the ball comes level with him, the imminent pass drops out of the scan, and the plan's "first dive point" jumps to a much later one, somewhere he could run to and dive seconds later. `dive_pays` then prices that far dive, finds it later than the standing take, and returns false. Rule (2)'s nearest-pass fallback is used only when the plan has no dive point at all. The old `dive_gains` looked only at the standing take and did not have this failure.

- Seed 290 play 242 (a −64° chopper, first bounce at the plate, at 0.50 s): the pitcher's reads at 1.13–1.37 s give a standing take 4.3–5.0 s off and a dive expected at 2.4–2.7 s, so true. At 1.48 s the dive point jumps from 0.27 s ahead to 4.07 s ahead, and the expected time goes to 5.69 s against a 4.33 s take, so false. The flag breaks as the ball passes 2.46 m from him. In main he dives, catches it and gets the out at first; on the branch the shortstop fields it at 3.5 s and it is a single.
- Seed 1210 play 26: the same play shape, the same jump (td 0.31 → 4.17 s at 1.28 s), the same result.

Against main's rule, 28 infield ground dives disappear on first differing plays: 2 better, 5 worse. Two of the five are these.

**Suggested:** weigh the earlier of the plan's first dive point and the nearest pass within `REACH + DIVE` while the ball is faster than `LUNGE_MIN_MPS` (rule (2)'s point). Then the dive priced is the one the capture is about to make.

### P3-1. With no standing take and no pass within reach, `dive_pays` returns true, and he lunges at a nearly stopped ball (the `near(tc) > REACH + DIVE` branch)

`return stand.is_none()` sets the must-dive flag whenever his read has no standing take. Rule (1) does not apply on this branch. When the real ball then comes within the envelope, the capture lets him lay out at any ball faster than 3 m/s:
- seed 64 play 57: the right fielder lays out at a 4 m/s ball 2.48 m away, the ball comes off his glove, and the double becomes a triple. In main he took it on his feet 0.40 s later.
- seed 137 play 129: a 3 m/s ball, a miss, a triple instead of a double.

Fifteen new outfield ground dives came from this branch (6 better, 2 worse). Both worse plays are these slow balls, and nobody lays out at a ball rolling at walking pace. **Suggested:** use rule (1)'s resting-point take here too, or return false once his read has the ball near rest.

### P3-2. The new cost test does not pin the knock-down and miss costs (`a_dive_at_a_ground_ball_pays_only_when_its_expected_cost_beats_his_standing_take`)

In the near case his standing take is 1.79 s away and the plan's dive point 0.53 s away. A sure catch therefore has him up at 2.03 s, already later than the take. The test passes with a get-up-only charge counted from the dive (mutation: expected time = `td + DIVE_GETUP_S`, which ignores the knock-down and the miss). The note's "it fails with the get-up-only charge" is true only of the old charge counted from now (`dive_gains`). That charge's change is what the test pins. The mixture, the actual content of review P2-1, is untested.

**Suggested:** add a case where the take falls between the catch and the expected time. At about 11 m/s in the same fixture, the take is about 2.3 s, a catch about 2.0 s and the expected time about 2.35 s. The poor-hands test does pin the hands gate: with the gate always passing on the ground (mutation), it fails as it should. The lay-out test does not touch the cost rule.

### P3-3. Seed 1000's −0.12 runs is not plain noise around the note's 0.015 (ledger row, note §7)

- Reproduced on sim-cli's own seed-1000 set (`1000·1,000,003 + i`): main's rule 4.450, branch 4.333, HR/PA 2.88% → 2.79%, the ledger's figures exactly.
- Played in lockstep, the mean difference is −0.117 a team-game with a standard error of 0.043 (2.7 SE). 247 of the 300 games end with identical runs, home runs and PA.
- The note's "18 runs in 600 games" counts only each game's first differing play (here −8 runs in 300 games). Later plays in the same game also feel the rule, and the games that diverge then drift.
- Pooled over my three 300-game sets (seeds 0–299: −0.025 ± 0.026; 1000–1299: −0.038 ± 0.027; sim-cli 1000: −0.117 ± 0.043), the rule's effect against main is about **−0.06 ± 0.02 runs a team-game**. Seed 1000's −0.12 sits at the high end of the noise around that.
- The home-run move is downstream noise: −18, +1 and −10 home runs in the three sets, none on a changed play.

**Suggested:** the ledger row should give the pooled figure and drop "the dive's own effect … 0.015". Runs are already an open item (ledger §5).

### P3-4. The early-dive cell: the diagnosis is half right (note §7–§8, job question 4)

My run of the cell (standing take under 2.7 s) gave 21 dives, 3 better and 4 worse (the note: 21, 1 and 6; the outfield total, 146 against 150, also differs slightly). The four worse:
- two are the slow-ball lunges of P3-1;
- one is a dive the in-air rule allowed while the ball was up (no plan dive point, so rule (3) did not apply) and that came after the bounce (195/76). This is the mirror of P2-1;
- one is a misread: his read put the standing take 3.77 s away, and in main he took it on his feet 0.07 s later at 0.95 m (1031/293).

In main, the same fielder took the ball standing at 0.95–1.0 m in 9 of the 21, by a stretch in 1, and dove too in 5 (at nearly the same moment, the same outcome). So the planner's 0.7 reach mostly hides takes inside the standing reach of 1.0 m, not the stretch, and it accounts for one bad play in 600 games. That is not worth its own task beyond the existing planner-reach line. P3-1 is the part that is a defect.

### P3-5. Rule (2)'s fallback point is measured from where he stands now (`dive_pays`, `near`)

The nearest pass and its distance `h` are taken from `fd.pos` now, not from where he will be. The point ignores the lunge time (`DIVE_LUNGE_S` from the flag), and the landing point is not kept off the wall as the capture's lunge is. So it can price a pass he cannot reach in the lunge, or one he will have run past. The capture still checks the geometry, so only the cost estimate is off. Fold this into P2-2's fix.

## Answers to the job's questions

1. **Rule (3) and the infielders.** The expected-cost test does not keep an infielder up at a grounder through the hole: with no standing take, his take is the ball's resting point in the outfield, seconds away, and the dive pays. The exception is P2-2 at the moment of the pass. Against main's rule, 102 other first changed plays: 25 better, 18 worse, 10 more outs, 9 fewer bases for the batter.
   - Worse, by position: first baseman 1 (a foul pop), third baseman 3 in the air (pop-ups) and 2 on the ground (bounced throws at 7.8 s and 11.4 s that his read said he would take standing soon: misreads), second baseman 3 (a liner, a pop to shallow centre, one more), shortstop 1 (a 33 m/s liner), pitcher 3 (the two choppers of P2-2 and one slow roller), outfielders 5.
   - The air-catch losses are P2-1; the pitcher losses are P2-2.
   - Rule (3) alone (against mode 2): 116 plays, 30 better, 19 worse, 12 more outs.
   - **Ground-ball hits do not rise:** on first changed ground-ball plays, 90 → 78 hits (108 plays); on the 102 others, 38 → 26. Standard run GB hit .308 on both main and the branch.
2. **Invariants 2 and 3: kept.**
   - `dive_pays` reads only his belief `p` (the standing take, the resting point `p.at(p.t_end())`, the nearest pass, `p.at(td)` for the ball's speed, `p.first_bounce()`), his planning state (`intercept_scan`, `scan_from` on a copy of his own body), his own `hands`, and the public fact `ball_still_up()`.
   - The knock-down path is `batted_path` on his estimate with no tilt draw (`|_, _| (0.0, 0.0)`), the field and the flight environment. Those are the same public inputs his own read uses (`believed_flight` changes only the spin he believes).
   - No `Executed*` value and no world ball path is read. The glove draw comes after the decision.
   - His knowing his own hands is §5.3's existing premise for the gate.
   - Determinism: `m::exp`, `m::hypot`, fixed-step loops, no hash iteration.
3. **Correctness.**
   - The three outcome times and the weights (pocket, rim minus pocket, one minus rim; Rayleigh CDFs of the capture's own σ, units in metres and seconds) are right.
   - A ball at rest is handled: the capture refuses a lunge below 3 m/s, and the resting-point fallbacks hold.
   - A ball moving toward him: the knock-down path runs on past him and he scans from where he lies. Correct.
   - No standing take: rule (1), except in the out-of-reach branch (P3-1).
   - The wall: the knock-down path sees it; the landing point does not (P3-5).
   - Rule (1)'s resting point is his read's (`p.at(p.t_end())`), reached at top speed with no acceleration, a small bias toward staying up.
   - Rule (2) can pick a pass he cannot reach in the lunge (P3-5). The larger fault is that rule (2) is not used when it matters (P2-2).
4. **A believable player?** Mostly. The early cell is small and half the bad plays in it are P3-1; see P3-4. P2-1 and P2-2 are the unbelievable cases this round adds.
5. **Tests.**
   - The three new or changed tests are independent of the implementation (geometry, the §5.4 radii, the hands' σ).
   - The cost test does not pin the mixture (P3-2). The poor-hands test pins the gate (the mutation fails it). The lay-out test checks where the ball ends up, as asked.
   - Seed 36 play 158 passes the double-cut test's premise checks (the outfielder fields it, the ball is down first, a relay man goes out). That the right fielder was first on the ball is the README's account, not something the test asserts. I did not check it separately.
6. **Numbers.** Seed 0 reproduced exactly (below). For seed 1000 see P3-3: reproduced exactly, and more than noise around 0.015.
7. **Docs.**
   - agent-models §5.3 matches the code, including the rule-(3) sentence, which describes what the code does; that rule is what P2-1 is about.
   - The first review's Resolution is accurate, except that "it fails with the get-up-only charge" holds only for the old charge from now (P3-2).
   - The note §7's "30 better, 18 worse, 12 more outs" matches rule (3) alone (my run: 30, 19, 12). It does not mention the lost air catches.

## Verification

All builds were in `sim/target/review-probes` (the worktree's own tests in `…/wt`, the probe copy in `…/t2`). Runs went through the job queue, one at a time, jobs 213–224.

- **Workspace tests** on `0da2212` (`live.rs` and `throw_choices.rs` touched first): `cargo test --workspace --no-fail-fast`: 569 passed, 0 failed, 4 ignored.
- **Standard run, seed 0** (`sim-cli` rebuilt): runs/team-game 4.11; PA 37.8, pitches/game 238; K% 22.6, BB% 8.0, HR/PA 2.74%, reached on BIP 33.8%; GB/LD/FB/PU hits .308/.668/.100/.005; out of play after a throw 0.123; rules warnings {}; 5,078 ms per game per core. All match the ledger row.
- **Probe copy.** `sim/target/review-probes/copy2` is `git archive 0da2212`, gitignored and outside the worktree's sources. Its `live.rs` has a thread-local `REVIEW_MODE`:
  - 0: the branch;
  - 1: the reviewed rule (`dive_gains` with the get-up);
  - 2: the branch without rule (3);
  - 3: a get-up-only charge inside `dive_pays` (expected time = `td + DIVE_GETUP_S`);
  - 4: main's rule (the pocket gate on the ground and `dive_gains`);
  - 5: the outfield hands gate always passing on the ground.

  It also logs every `dive_pays` call (its branch, take, dive point, expected time, chances, knock-down and miss times), every capture attempt (standing, stretch or dive, distance, glove error) and every dive. Nothing in the worktree was edited.
- **Paired probe** (`zz_rev2`): each game stepped in lockstep under two modes until the first pitch whose events, dives or securings differ; both versions of that play recorded; optionally both games played to the end.
  - Modes 0 and 4, full games: seeds 0–299 [1000–1299] 4.112 vs 4.137 [4.088 vs 4.127]; dives 2.59 vs 2.46; infield dives 2.05 vs 2.21 [1.95 vs 2.16].
  - First changed plays: 248 pairs. New outfield ground dives: 146, 90 better, 5 worse, batter's bases −111, runs −26. By the standing take in main's game: under 1.5 s, 12 (0/4); 1.5–2.7 s, 9 (3/0); 2.7 s or more, 63 (48/1); another fielder, 62 (39/0).
  - Modes 0 and 1: 196 pairs, 43 better, 38 worse, 14 more outs, bases −8, runs −3. Of the outfield ground dives the fix removes (96), 17 removals helped and 22 hurt.
  - Modes 0 and 2: 116 pairs, 30 better, 19 worse, 12 more outs.
  - Modes 0 and 4 on sim-cli's seed-1000 set: P3-3.
- **Single plays** (`zz_play`, every `dive_pays` call for one fielder): 290/242, 1210/26 (P2-2); 106/191, 1043/201 (P2-1); 64/57 (P3-1); 1035/12 and 9/47 (third baseman on bounced throws: misreads of the standing take, not a defect).
- **Mutations** (`cargo test -p bb_game --lib`, the cost, poor-hands, lay-out and play-71 tests): modes 0, 1, 2 and 3 all pass; mode 5 fails the poor-hands test only. The cost test's numbers (a printing test in the copy): 10 m/s, take 1.79 s, dive point 0.53 s, expected 2.28 s (mode 3: 2.03 s); 12 m/s, take 2.93 s; 14 m/s, take 4.15 s, expected 2.58 s.
- Every job I queued has finished; I left no background shells.

## Code Mode

One persistent context again suited a review: the probe's pair lines were parsed once and sliced by cell, by position, by rule branch and by outcome without re-running anything. Two queue rounds were lost to paths with spaces (a one-argument shell line needs its own quotes) and one to a shared target dir that served the old probe copy's stale library. Giving each copy its own `CARGO_TARGET_DIR` fixed it. The HTML-escaped output still makes Rust harder to read.

## Resolution

Open.
