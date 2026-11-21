# Review: the covering man and the receiver's estimate (`work/dp-cover`, `667762a..37bec49`)

Reviewer: Opus, high effort. Scope: `git diff 667762a..37bec49`: `sim/crates/bb_game/src/live.rs`, `live/holder.rs`, `tests/throw_choices.rs`, the two `a4` fixtures, agent-models §5.3/§5.5, the worker's note [2026-11-21-dp-cover](../calibration/2026-11-21-dp-cover.md) and the ledger row. I wrote none of it.

**Summary.** The two saved plays are fixed. Both fixtures fail on `667762a` and pass on the branch, and all workspace tests pass. The new beliefs read only what a teammate can see. The note's standard-run figures reproduce exactly. Two things are wrong:

- **P1-1. A holder now lets a throw go before its receiver is in place.** The receiver estimate is now accurate, and that exposes an old release rule: the holder throws once the ball would get there no more than 0.05 s *before* the receiver is within 1.5 m of the bag. When the man comes in from the side, the ball passes the bag out of his reach and runs on into the outfield. On seeds 0–255 the pitcher's feeds to second that no fielder caught went from 0 of 66 (main) to 39 of 115. The same happens on his throws to third and at first, and once or twice per 100 games from the other infielders. No real pitcher does this. It must be fixed before merge.
- **P2-1. The note's explanation of the shortstop's double-play drop (question 1) is wrong.** The shortstop does not carry the ball to second more often. His own carries, as his first choice, went from 9 to 5 in seeds 0–99, and from 18 to 10 in seeds 0–255. The worker's "9 → 28" counts every carry record of the shortstop. On the branch most of those records are the *covering* shortstop: he catches a feed short of the bag and then steps onto it (31 of 37 are within 2 m of the bag). The note §4 and ledger §5 need correcting. The answer to question 1 is under "Question 1" below.

## Findings

### P1-1. The hold-for-the-receiver rule lets the throw arrive before the receiver can reach it; with the accurate estimate, throws go through the bag into the outfield

`live.rs:6674` (`holder_act`): `if race.direct < race.receiver_at - 0.05 { …wait… }`, with `receiver_at` from `receiver_ready` (`live.rs:6876`), the time the man is within `RECEIVE_REACH` (1.5 m) **of the bag**.

- **What happens.** The holder waits only until his throw would arrive no earlier than 0.05 s *before* the receiver is 1.5 m from the bag. The receiver catches a throw only within 1.5 m of the ball (`live.rs:5007`). A man coming at the bag from the side is about 1.5 m from the ball as it passes over the bag, and further from it a moment earlier. A throw that arrives on that boundary, or on the far side of the bag by its normal miss, passes him. On main the estimate was a run from rest at 8 m/s and was 0.5–0.8 s pessimistic for a man already running, so holders waited far longer than the rule needed, and the rule's slack never showed. Now the estimate is exact (the worker's test shows it to the tick), and the throw is released to arrive just as the man reaches 1.5 m.
- **Concrete case: seed 12, play 105 on the branch** (review probe `zz_trace`). One out, runner on first. The pitcher fields a comebacker at 1.02 s and waits for the second baseman, then throws at 2.38 s. His estimate has the receiver at 1.5 m at 3.05 s and the ball at the bag at about 3.0 s.
  - The ball passes over the bag at 3.02 s, 1.3 m high.
  - The second baseman, crossing from the first-base side at about 5 m/s, is 1.75 m from it at that moment (at 3.07 s he is 1.32 m from the bag).
  - No catch is attempted. The ball rolls to the centre fielder, who secures it at 5.1 s. The runner goes to third and the batter is safe at first.
- **How often** (seeds 0–255, every first commitment with a force on at second and fewer than two out; "uncaught" = the next fielder to secure the ball is not the throw's receiver):

  | | main `667762a` | branch | branch, braking covers only (variant A) | branch, no touch timing (variant B) | branch + release guard (variant C, below) |
  |---|---|---|---|---|---|
  | pitcher's throws to second uncaught | 0 / 66 | **39 / 115** | 34 / 110 | 41 / 117 | 4 / 113 |
  | all throws to second uncaught | 19 / 559 | 60 / 646 | 57 / 646 | 62 / 651 | 24 / 627 |

  `throw_choice_probe` over seeds 0–99 (every throw, every base): throws not secured next by their receiver went from 22 of 1,764 to 54 of 1,843. On the branch these include the pitcher to second (17/53), to third (8/24) and to first (3/257).
  - Variants A and B show the misses do not come from the crossing or from the touch timing. They come from the receiver estimate meeting this release rule.
  - The pitcher's feeds to second rose from 66 to 115 because the estimate now has the cover there in time. A third of those feeds then go into centre field.
  - This probably accounts for part of the GB hit rise the note calls noise (.309 → .317). It may account for part of the runs and extra bases too: in the case above the runner took third.
- **Fix (tested as a probe, variant C).** Release only when the ball will reach the bag after the receiver can take it there. The probe used `race.receiver_at > self.t && race.direct < race.receiver_at + 0.1`. The first condition stops a holder waiting forever on a short toss to a man already in reach.
  - With that guard: pitcher's uncaught feeds 39/115 → 4/113; all feeds to second 60 → 24 (main 19).
  - Double plays by first fielder: P 47% → 62%, 1B 40 → 43%, 2B 48 → 47%, 3B 48 → 50%, SS 50 → 58% (main 60%).
  - A cleaner form of the same fix: time the receiver's readiness at a radius from the bag that leaves room for the throw's typical miss (`BAG_REACH_FRAC × RECEIVE_REACH` less `THROW_MISS_MIN_M`, or the touch radius for a force), and make the tolerance a named [E] margin *after* that time, not before it.
  - Add a test: a cover coming at second from the first-base side, a holder waiting on him; the throw the rule releases must pass within the receiver's reach of him.
  - The pitcher's double-play share then rises further above MLB (19.6%). That excess already has its own cause on record (ledger §5); re-measure it after the fix.

### P2-1. The note and the ledger give the wrong cause for the shortstop's double-play drop

`docs/calibration/2026-11-21-dp-cover.md:73` (§4, first bullet), `docs/metrics-ledger.md:406` (§5 entry), and the ledger row's Open/Cause text.

- **Claim:** "he carries the ball to second himself three times as often (9 → 28)".
- **Measured** (review probe `zz_review_dp`, the first commitment of each play by the man who fielded the ball, fewer than two out, a force at second available):
  - Shortstop carries to second: seeds 0–99, 9 → 5 (main's choice counts match the worker's "before" exactly: 65 to second, 9 carries, 6 to first, 5 to third). Seeds 0–255: 18 → 10.
  - Counting every carry record by the shortstop (`throw_choice_probe`, seeds 0–99) gives 18 → 37, the worker's kind of count. Of the 37, only 5 are the shortstop's first choice. 31 are within 2 m of the bag: the covering shortstop who caught a feed short of second and steps onto it (the touch-after-catch that crossing makes common).
- **Fix:** correct the note §4 and the ledger §5 entry and row with the cause below, and drop the 6U-3 lead.

### P2-2. The carry's double-play estimate disagrees with what the world does after a 6U-3 or 4U-3 (pre-existing, asked in question 1)

`live/holder.rs:119–134` (`force_continuation`, carry branch); the world: `live.rs:6531–6542` (a carrier is an ordinary holder on the bag) and `set_up_for` (`live.rs:6742`).

- **Estimate.** The carrier is timed as a pivot man: the touch at `option.ball_at`, then `pivot_s` (0.80 s at grade 50), plus a turn and plant from braking onto the bag at 2.39 m/s.
- **World.** The carrier keeps the set-up he spent carrying. His turn and footwork come from his *catch* (`fd.face`, `fd.catch_vel`), not from how he is moving at the bag. He lets the ball go on the tick after the touch.
- **Measured, the ball at first, world minus estimate** (seeds 0–255):
  - Shortstop carries: main −0.51 s, branch −0.36 s (world sooner). DPs 12 of 18 against an estimated 8.4; branch 7 of 10 against 7.5.
  - Second-baseman carries: −0.30 and −0.09 s.
  - The force at second itself comes 0.20–0.23 s later in the world than `carry_ball_at` has it, on both builds.
- **Which side is wrong.** Neither side is right.
  - A second baseman running to second away from first at 5 m/s and throwing to first on the touch with no plant is not something a real player can do. The world should charge the carrier a turn and plant from his motion at the touch (`exchange_footwork_s(fd.vel, to_first)`, the turn from his running direction).
  - The estimate should use that same model.
- **Not caused by this change, and not merge-blocking.** Hand it to the throw-choice backlog with a frozen 4U-3.

### P3-1. `BAG_TOUCH_M` is named but the world still uses the literal

`live.rs:771` names `BAG_TOUCH_M` = 0.6 for the estimate. The world's force-touch, pivot-start and pivot-wait tests still use `0.6`: `live.rs:5196`, `6531`, `6571`. If the constant is changed, the belief and the world part. Use the constant everywhere, and name it in agent-models §5.5 as the other three are.

### P3-2. The receiver-estimate test is mostly a restatement; the case that matters in games is untested

`live.rs:15689` (`the_receiver_estimate_follows_the_covering_mans_movement`).

- **Why it is a restatement.** The estimate (`cover_reach`) and the world (`fielder_step`) both call `cover_step` and `step_toward`. With the feed fixed on both sides (the test pins `read_t` and turns off re-reads), agreement to the tick is guaranteed by construction. The only independent assertion is that the old estimate was more than 0.3 s off.
- **What it does not cover.** The case it leaves out is the one in games: the ball held, the holder's estimate timed to *his* throw (`arrives`: his hurry, effort and cut), the cover moving on *his* belief (`feed_seen`: `THROW_EFFORT`, the holder's set time from the catch). These differ, so the cover can brake when the holder expects him to cross, or the reverse.
- **Fix.** Keep test 2 (closed-form kinematics, independent). Replace test 1's assertion with a Held-ball case that bounds belief against world, and add the P1-1 test.

### P3-3. The covering man's belief of the feed (`feed_seen`, `live.rs:6910`) has three gaps

1. **A pivot man still going to the bag** (`pivot_wait_bag`). His pivot clock is charged from `secured_at`; the world starts it at his touch (`live.rs:6540`). The cover at first believes the throw 0.1–0.2 s early. He is more likely to judge himself late and cross first before it comes. Rare: no batter was safe on such a throw in the probes.
2. **Effort.** The throw's flight is always at `THROW_EFFORT`. With two out (no continuation) a force throw goes at the least effort that keeps the margin (`live.rs:6690–6697`). It arrives later than the cover believes.
3. **A holder carrying the ball to the cover's own bag.** The cover still times a throw from him and may cross the bag ahead of a carrier (the worker's own open item).

None of the three showed as a lost out in the probes. Covers that were within the touch radius before the release and more than 1 m off the bag at it: P at first 0 → 0, 1B 0 → 4, 2B at second 2 → 6, SS 15 → 12 (seeds 0–255). So the seed 23 play 268 fix generalises well enough. Fix the first two for consistency; give the third its own frozen case if one appears.

### P3-4. Stale comment

`live.rs:186–188` (`PIVOT_FACE_MIN_MPS`) still says a pivot man "brakes onto it (… 2.39 m/s at the touch radius)". Only an early one does now; a late one crosses at about 4.7 m/s.

## Question 1: the shortstop-started double plays

**Is the carry the cause?** No (P2-1). His own carries fell. His feeds to second are as many as before: 186 of 230 first choices on main, 177 of 212 on the branch (seeds 0–255). Fewer of them turn two:

| SS feeds to second (seeds 0–255) | main | branch |
|---|---|---|
| feeds | 186 | 177 |
| pivot man threw on to first | 130 | 102 |
| double plays | 122 | 95 |
| holder's estimated second-out p ≥ 0.3 | 132 | 105 |
| sum of estimated second-out p | 127.4 | 103.1 |

The holder's estimate tracks the world on both builds: the sum of p against the double plays made, and the ball at first a mean +0.07 to +0.09 s against his estimate. The fall is in the plays themselves. By counterfactual builds (same seeds; shortstop-started double plays per chance; the sampling error of one run is about ±3.4 points):

| Build | Shortstop-started double plays |
|---|---|
| main | 60% |
| branch | 50% |
| A: no crossing (every cover brakes onto the bag; estimate and touch timing kept) | 54% |
| B: no touch timing in `throw_race` | 52% |
| C: branch + the P1-1 release guard | **58%** |

So most of the drop goes with P1-1: feeds released to arrive as the second baseman reaches 1.5 m, caught off the bag, the pivot then later. The second baseman's crossing, with the plant he owes moving away from first, takes a few points. The touch timing takes about two.

**Is the carry's estimate consistent with the world's 6U-3?** No, in the other direction: the world throws 0.36–0.51 s sooner than the estimate (P2-2). That makes carries rarer than they should be by the holder's own rule, not more common.

**Is timing the feed to the touch right when the receiver catches at reach and touches after?** Yes, for this world. The force is the touch with the ball, and the estimate matches the world's touch to within 0.01 s (median 0.000, seeds 0–255). A real pivot man catches with his foot already on the bag. The sim's catches off the bag come from P1-1's early releases, not from the touch timing.

**Would a real shortstop make these carries?** His own (5 per 100 games, all within the 8 m `carry_max_m`, 7 of 10 turned) are plausible for a ball fielded near the bag. A real one would not then throw to first with no plant (P2-2). The "carries" that rose are a receiver stepping onto the bag after catching a feed short of it. With P1-1 fixed, most of those catches are made at the bag.

**Before merge:** fix P1-1, then re-run `dp_probe` by first fielder and correct the note and ledger (P2-1). P2-2 can wait.

## Question 2: first-baseman-started double plays

The crossing does it: 19% (main) → 40% (branch) → 24% with every cover braking (variant A). The 3-6-3's shortstop crosses second moving toward first and owes no plant. The pitcher covering first crosses too: his catches are now a mean 0.74 m from the bag (main 0.25 m; over 1 m on 117 of 239), touching 0.09 s later. No more batters are safe on them (10 → 9).

The holder's estimate is consistent with the world here: summed second-out p 23.5 against 28 double plays (branch), the ball at first 0.06 s sooner than estimated. So nothing in this change is wrong in itself. The first baseman now feeds second on 65 of 73 chances (main 44 of 67), because the covering shortstop is believed in time, and he is.

What is missing is world-side and already a seam. The runner from first does nothing to the pivot man: no slide, no break-up (`PIVOT_SLIDE_DELAY_S` = 0, agent-models §5.5). On a 3-6-3 that runner is coming straight at the shortstop as he crosses toward first, which is the play's main real cost. Neither does catching a throw while crossing at 4–5 m/s cost anything.

No change is needed in this branch. Record the 1B and P excess with that cause in ledger §5.

## Other questions

- **Invariants 1–3.** No outcome roll was added. The new code reads no `Executed*` output.
  - `cover_reach` reads a teammate's position, velocity and movement attributes (speed, τ, `a_lat`: known abilities, as before). It reads the runners' visible positions and the force state, and it applies the published movement rule.
  - `feed_seen` reads the holder's position, catch facts (heading, velocity at the catch, dive, stretch, whether it was a throw), the public secure time, his known `Transfer` and arm, and his pivot flag (derived from public facts: a feed caught at second with a force on). It reads none of his private set-up clocks (`up_at`, `set_base`) and not his choice.
  - The cover's `feed_at` is read only by himself.
  - The isolation harness (`iso_flips`) does not flip `pivot`/`pivot_wait_bag` or the `catch_*` fields that `feed_seen` reads. Those are observable here, but if any of them later gains a private component, add it to the flips.
- **Hysteresis (question 4).** `cover_step` decides early or late on every tick with no hysteresis. While the holder is set and waiting, `feed_at` moves forward with time (`t + flight`), so a man near the threshold can switch between braking and crossing tick to tick. I did not measure chatter (the tracks are sampled every 0.1 s); it changes no outcome I could find. Check it in the client, or add a small hysteresis margin.
- **Determinism and [E] parameters.** `COVER_CROSS_MPS`, `EARLY_MARGIN_S` and `COVER_STEP_CAP_S` are named, tagged and in the spec. `BAG_TOUCH_M` is named but not shared with the world (P3-1). The literals `0.3` (`holder.rs:133`) and `along.max(0.5)` (`cover_step`) were there before. No map iteration was added; loops are over index ranges; the math uses `m::sqrt` and `norm`. The traced and untraced digests are identical over 100 games (`throw_choice_probe`'s own check).
- **Fixtures.** Both fail on `667762a` (the branch's `throw_choices.rs` and fixtures run in a copy of main: `seed_51001_play_66_…` and `seed_51001_play_168_…` FAILED, the other 12 pass), and both pass on the branch.

## Verification

All builds in this worktree's `sim/target` or, for copies, `sim/target/review-probes/t-*` (each copy its own target dir).

- `cargo test --workspace` on `37bec49` (sources touched first): all pass (bb_game lib 263 tests, including the three new ones; `throw_choices` 14).
- `git archive 667762a` into `review-probes/main-src`, with the branch's `throw_choices.rs` and the two fixtures added: the two new tests fail, the rest pass.
- `sim-cli games --n 300 --seed 0` on the branch: identical to the note's figures (runs 4.36, K 22.4, BB 8.1, HR 2.74%, GB .317/.056/.068, out of play after a throw 0.123). Seed 1000 not re-run.
- Review probe `zz_review_dp` (not kept; source in `sim/target/review-probes/zz_review_dp.rs`, analysis `an*.py` there). Seeds 0–255 on main, the branch and three counterfactual builds of the branch:
  - A: `cover_step` never crosses.
  - B: no `touch_at` in `throw_race`.
  - C: the P1-1 guard.
  - Per play it records the first commitment with a force at second (its carry, feed and first-base options with their estimates) and the world's touches, releases, secures and calls. Per throw caught near first or second it records the receiver's distance from the bag at the catch, the catch-to-touch time, and whether he had left the bag before the release.
- `zz_review_dp` with two-out choices included (seeds 0–99), to match the worker's "before" counts.
- `throw_choice_probe 100 0` on main and the branch (all commitments; carry records; throws not taken by their receiver).
- `zz_trace` (not kept): seed 12 play 105 on the branch, the ball and the pitcher, second baseman and shortstop tracks at 0.1 s.
- **Not checked:**
  - Tick-level chatter in `cover_step`.
  - The seed 1000 standard run.
  - `dp_probe`'s own figures (my chance definition differs from it, so my per-fielder rates are not the note's, but main and the branch are measured the same way).
  - Variant C's effect on the standard run, and whether it disturbs ordinary relay or tag-play timing.
  - Whether the uncaught feeds are what moved the GB hit rate.

## Resolution

Resolved for merge (Coordinator, 2026-11-21). The fixes are on `work/dp-cover`, final sim commit `acc17a9`; details in the Worker follow-up below.

- **P1-1 fixed:** a held throw reaches the bag `RECEIVE_MARGIN_S` (0.1 s) after the receiver can take it. The pitcher's feeds to second not taken: 39 of 115 → 4 of 114 (main 0 of 66); every throw, seeds 0–99: 54 → 24 (main 22).
- **P2-1 fixed:** the note and the ledger no longer blame the shortstop's carries.
- **P2-2 deferred** (predates the branch; TASKS "Open sim work" §3): the carry's double-play estimate against the world, and the throw with no plant after a carry.
- **P3-1, P3-2, P3-4 fixed. P3-3:** the visible carry fixed; the pivot clock from the catch and the full effort assumed are deferred in the same TASKS item.
- **Question 2 deferred** (same TASKS item): no slide or break-up on the pivot man, behind the first-baseman-started and pitcher-started excess.
- **No re-review:** the P1 fix is the change the reviewer tried in a test build, and it is covered by a fixture (seed 12 play 105) and a held-feed test. The Coordinator touched the sources and ran `cargo test --workspace` (497 passed) and the standard run (4.16 [4.59], rules warnings empty), matching the worker's figures.
- **Open after merge:** runs −0.21 [−0.18] against main, about 0.04 explained (ledger §5).

### Worker follow-up

On `work/dp-cover`, final sim commit `acc17a9` (fixture `2d1c8dc`; spec and TASKS `7f2f985`; docs after).

- **P1-1, fixed.** A man not yet within receiving reach counts as ready `RECEIVE_MARGIN_S` = 0.1 s [E, unsourced; agent-models §5.5] after he gets there, and the holder waits while his throw would reach the bag before that (a man already within reach is ready now): the review's variant C in named form. Tests: `seed_12_play_105_the_pitchers_feed_is_taken_by_the_covering_second_baseman` (frozen from `37bec49`, where the feed passed him; now he takes it at second, the force by 0.62 s) and `a_feed_held_for_a_cover_coming_across_is_taken_by_him` (below). Throws not taken by their receiver (`zz_uncaught`, throwaway; main matches the review's counts exactly): the pitcher's feeds to second, seeds 0–255, 0 of 66 (main) → 39 of 115 (review, `37bec49`) → **4 of 114**; every throw, seeds 0–99, 22 of 1,764 → 54 of 1,843 → **24 of 1,794**.
- **P2-1, corrected.** Note §4 and §5, the ledger row and the ledger §5 entry now give the review's cause (his own carries 9 → 5 and 18 → 10; feeds turning two less often, mostly through P1-1); the 6U-3 lead is deleted. With the hold fixed, shortstop-started double plays 51.9% (main 54.3%).
- **P2-2, not fixed (pre-existing; the Coordinator's call).** A TASKS "Open sim work" §3 item with the review's figures.
- **P3-1, fixed.** The world's three literal 0.6s are `BAG_TOUCH_M`, named in agent-models §5.5.
- **P3-2, fixed.** The holder now times the cover's crossing to *the cover's* belief of the throw (`feed_seen`, worked out from what the cover sees of him), not to his own hurried release: in the new held-ball test (starts 4–10 m out at 0–7 m/s, the cover moving on his own belief) the holder's belief of the man's reach is within 0.03 s of the world's (0.12 s with the holder's release), and every feed is taken by him at the bag. Test 1 now checks `cover_reach` and says it is a consistency check of the shared stepping; test 2 (kinematics) is kept.
- **P3-3.** Third gap fixed: a holder visibly running the ball to the cover's bag (faster than 3 m/s, within about 25° of the line, not on it) gives the cover no throw to time; he brakes onto the bag. The first two gaps (a pivot man's clock from the catch; throws assumed at full arm) are deferred to the same TASKS item.
- **P3-4, fixed.** The `PIVOT_FACE_MIN_MPS` comment names both arrivals.
- **Question 2.** Recorded in the note §5, ledger §5 and the TASKS item (no slide or break-up on the pivot man; catching while crossing costs nothing).
- **Re-measured on `acc17a9`** (sources touched): `cargo test --workspace` 497 passed, 0 failed; fixtures 51001-66 (force at second, 0.65 s) and 51001-168 (4-6-3, 0.20 s at first) unchanged. Standard run, main → final: runs 4.37 → 4.16 [4.77 → 4.59], GB .309 → .309 [.324 → .322], GB 2+ outs .065 → .072 [.063 → .072], out of play after a throw 0.103 → 0.103 [0.140 → 0.137]. `dp_probe 256 0`: 11.2% → 12.3% per opportunity; P 51.9 → 63.6%, 1B 19.3 → 35.4%, 2B 34.7 → 41.4%, 3B 44.0 → 46.6%, SS 54.3 → 51.9%. **Unexplained:** runs −0.21 [−0.18], of which the extra double plays and forces explain about 0.04 (ledger §5).
