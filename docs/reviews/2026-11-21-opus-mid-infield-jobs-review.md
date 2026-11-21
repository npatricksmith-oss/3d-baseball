# Review: the middle infielders' jobs on a ball through the infield (`work/mid-infield-jobs`)

**Reviewer:** Opus (independent; wrote none of the change). **Date:** 2026-11-21.
**Range:** `904c1ea..d92dec2` (last sim commit `6028bed`; test tolerance `318d545`). Worktree `.claude/worktrees/runner-decision`.
**Read:** `sim/crates/bb_game/src/live.rs` (diff and the helpers it calls), `tests/throw_choices.rs`, the fixture `tests/fixtures/a4/51001-102.json`, agent-models §5.5/§5.7, the mid-infield note, the ledger row, the saved-plays diagnosis §3, the TASKS.md and observations.md edits.

## Findings

### P1-1. With a runner on first the shortstop leaves second for the cutoff to third while another infielder is going for the ball, and fields it

`live.rs:4625-4655` (`through_job`, the `short` test at 4648) and `live.rs:4658` (`cutoff_job`, which sends a loose ball with a known infielder chasing to `through_job`).

`through_job` sends its man out whenever *his estimate* (`seen_intercept` on his read) has no other infielder fielding the ball inside `INFIELD_BREAK_R`. It never looks at whether a teammate is **known to be going for it**. The estimate is often wrong in the pessimistic direction, so the man goes out on balls that an infielder then fields well inside the infield:

- **Seed-wide (probe `zz_review_mid` + `BB_COVER_TRACE`, game seeds 0–119, ground balls first down within `INFIELD_R`).** On the branch a middle infielder had a cutoff job while the first ball was loose on 154 ground balls with a runner on first. On **56** of them an infielder fielded the ball (2B 19, 1B 20, 3B 12, P 4, SS 1). On 41 of the 56 it was fielded 30–45 m from home, 12 under 30 m. On **39** of the 56 an infielder was already known to be going for it when the shortstop went out. With nobody on: 412 early cutoffs, **145** fielded by an infielder (3B 53, P 43, 1B 38, 2B 6, SS 5), 82 with an infielder already known to be going for it.
- **Second base uncovered while the ball is loose, runner on first, ball fielded by an infielder:** 4 plays on main, **40** on the branch (120 games each). Counting every ground ball with a runner on first it goes from 52 to 102. With nobody on it falls from 83 to 42.
- **Frozen play, main against the branch** (play 102's fixture turned 3° toward the right, runner on first, none out; `zz_review_main` with `BB_COVER_TRACE=1` on both builds). The ball and every catch are the same on both builds: the 2B chases it, fields it at 51.3 m at 4.59 s, nobody is out. On main the shortstop covers second throughout (`[.., 2B 1, .., SS 12, ..]`). On the branch the shortstop takes `Relay(3)` at 1.74 s, is back on second at 1.76 s, leaves again at 2.32 s and returns at 3.76 s. **Second is empty for 1.45 s** with the runner from first running and the second baseman chasing. The job also **flickers** as the estimate crosses the 51.6 m threshold.
- **Game examples (branch):** seed 1 play 188 (one out, runner on first). At the first call the shortstop goes out as cutoff to third while the second baseman goes for the ball. Second stays uncovered until the 2B fields it at (7.2, 42.5) at 1.76 s, and he takes the out at first only. Seed 8 play 89: the shortstop goes out at 2.58 s, 0.03 s before the 2B fields it at 39 m. Seed 3 play 169: a chopper the first baseman fields at 26 m. At the first call the shortstop is out toward third and the second baseman has no job (`[0, 14, 1, 0, 13, 23, …]`), so second is empty until 0.96 s.

**Would a real player do this?** No. With a runner on first and a ground ball the second baseman, the first baseman or the pitcher is going for, the shortstop goes to second for the force or the pivot. He goes out for a relay only once the ball has visibly got past the infielders. The spec agrees with the real player, not the code (P2-1: "an infielder known to be going for it, or holding it, ends the job").

**Outcome cost, measured:** small so far. Runner-on-first infield grounders gave force at second 199/267 on main and 199/265 on the branch, double plays 114 against 110. These are reshuffled games, within noise. The cost is in behaviour on the commonest double-play situation, and it would show in the client.

**Fix suggested.** At a minimum, an infielder **known** to be going for the ball ends or blocks `through_job`, as the spec says. Toward third (runner on first) that includes the pitcher. Better still, the man goes out only once the ball has visibly passed the chasing infielder or the infield (`ball_at` is already the convention for "as everyone sees it"), not on his estimate of a teammate's range. Add a frozen runner-on-first case like the +3° variant above to `throw_choices.rs`, and re-measure second-uncovered on **all** ground balls, not only those an outfielder fields (the note's 46 → 22 leaves these cases out by construction).

### P2-1. The spec does not state the rule the code implements, and in places says the opposite

`docs/agent-models.md:629` (the "At the first call" bullet) against `live.rs:4648`.

- The spec says: "His own read may still have the pitcher or another infielder reaching it; going out on the ball's line is then also where a ball they knock down comes to, and he leaves the job when one of them is known to be going for it." The code (`8a444e4`) does not send him out when his estimate has another infielder fielding it in range. It does **not** end the job when a teammate is known to be going for it (P1-1). The pitcher is left out of the estimate toward second.
- Neither the "no other infielder by his estimate" rule nor the pitcher asymmetry (left out toward second, counted toward third) is in the spec. Both appear only in the code comment and the note.
- Related fix 6 (`live.rs:4536`: a man whose intercept lies beyond `INFIELD_BREAK_R` keeps the cutoff) is in the note §1 item 6 but not in agent-models.

Invariant 7 asks for the spec to change in the same commit. **Fix:** rewrite the bullet to the rule that is kept after P1-1, including the pitcher's treatment and fix 6.

### P2-2. With nobody on, a middle infielder runs out toward the outfield on balls the pitcher or the third baseman fields

`live.rs:4648` (`!(b == 2 && j == 0)`), plus the same estimate problem as P1-1.

With nobody on there is no force at stake, so this only shows on screen. Still, 145 of 412 early cutoffs (35%) went to balls an infielder fielded. 43 of those were the pitcher's (comebackers fielded about 18 m from home: the ball-side man broke out 0.1–0.3 s before the catch). 53 were the third baseman's, 30–45 m out, while the shortstop ran out as cutoff for second. On two plays (seed 19 play 71, seed 40 play 225) the second baseman went out as cutoff while the shortstop chased, and second stayed empty for 0.3–0.6 s.

The note's reason for leaving the pitcher out ("a ball he knocks down comes on toward the man going out on its line") holds only for a ball the pitcher deflects straight. Here he fields it cleanly. A real middle infielder breaks toward the ball or the bag on a comebacker; he does not run into the outfield.

**Fix:** the same gate as P1-1. Treat the pitcher like the other infielders toward second too, or, if the asymmetry stays, at least end the job when the pitcher is known to be going for it.

### P2-3. The note and the ledger measure only the cases the change improves

`docs/calibration/2026-11-21-mid-infield-jobs.md` §4 and the ledger row.

Every job figure counts only ground balls **first fielded by an outfielder**. That selection drops exactly the plays where the early cutoff goes wrong: infielder-fielded ground balls, where second-uncovered with a runner on first rose from 4 to 40 (P1-1). The note's §3 says "Toward third the pitcher counts, so the shortstop stays on second". That holds only for the pitcher. "The shortstop is the cutoff to third as soon as the ball is through" is decided by an estimate that is wrong about a third of the time.

The row's "Nothing unexplained" holds, but it is incomplete. The 22 second-uncovered plays include new kinds: three where one middle infielder was on a cutoff while the other chased (seeds 17/129, 28/233, 42/43 with a runner on first; 19/71 and 40/225 with nobody on).

**Fix:** after P1-1, add the infielder-fielded counts to the note and the ledger row.

### P3-1. `through_job` works out the base with the shortstop's view even when the second baseman is the man

`live.rs:4629`: `relay_base(false, 5)` reads `visible_target` with the shortstop's `throw_choice` parameters even when `second_cutoff_man` returns the second baseman. Today every fielder has the same `ChoiceParams`, so there is no effect. Once they become ratings it would be the wrong man's eyes. **Fix:** work out `who` from the base the side man would see, or pass the candidate.

### P3-2. TASKS.md keeps a done item inside an open one

`TASKS.md`: the saved-plays item now reads "(Play 102, Q-C (a), done on `work/mid-infield-jobs`: …)". TASKS.md's own rule is "An item that is done is deleted, not ticked and kept". "Resume here" still lists play 102 as next; that is the Coordinator's to replace at merge. **Fix:** drop the parenthesis and leave only item (3).

### P3-3. The deep-ball rule widens the chosen option (accepted, noted)

The diagnosis's Q-C (a) said "the deep relay rule only for balls in the air past an outfielder". The branch also makes a ground ball deep once it has got past its chaser (`deep_ball`, `live.rs:4579`). That is what a real defense does on a ball rolling to the wall, the spec states it, and a test covers it. No change needed; recorded so the Coordinator knows the rule is wider than the option text.

## Answers to the questions

**1. Invariant 2 and D1.** No P1 world read found. Inputs of each new decision:

- **`through_job`:**
  - `self.ball` (loose or held) and `ball_still_up` (whether the first bounce has *happened*): visible facts.
  - `first_down_m` (the first `BallTouched` fact's position): a seen fact. It uses the true bounce point against the `INFIELD_R` threshold, as `is_grounder`-style facts already do.
  - `relay_base`: runners' `start` and path position, visible.
  - `second_cutoff_man`: `ball_side_x` (the shared "as everyone sees it" convention, built on `fielding_point`, `lead` and `ball_at`, unchanged), plus `seen(side, partner)`, which is heard.
  - The man's own `belief`, `t_int`, `int_pt` and `read_ball`: his read.
  - `seen_intercept(j, his read)`: teammates' visible position and velocity, known speed and reaction. That is his estimate.
- **`deep_ball`:**
  - `RELAY_DEEP_M` on the fielding point: holder position or the lead's track point, both visible.
  - `first_down_m`: a fact.
  - Held: the holder's pre-pitch spot (`home`, known).
  - Loose: `lead()` (heard), `ball_at(self.t)` (the ball's *current* position, the existing "as everyone sees it" convention, not its future path) and the chaser's position (visible).
  - Otherwise: the nearest outfielder's spot.
- **"Batter on first":** `relay_base` counts runners with `start > 0`, i.e. on base at the pitch, a known fact. It never reads the batter's true base beyond his visible position.
- **The world:** no new read of the true future path, the true fielding point or who truly fields it.
- **`job_trace`:** writes only.

**2. Determinism and the trace.** `job_trace` copies `role`, `pos`, `vel`, `target` and `t` after the tick's decisions and mutates nothing. Probe `zz_review_det`: digests for seeds 0, 1, 7 and 51001 were identical with the trace off, on (17,000–26,000 ticks recorded) and off again in one process. The worker's `jobs.py` counts on seeds 0–59 reproduce exactly from my own build (206/203 plays, 0.444/0.189 s, 46/22, 83/65, 171/85, 0/121), so the branch is reproducible across builds and processes. The diff adds no `HashMap`/`HashSet`, no RNG draw and no transcendental: it uses `m::hypot`, `norm` and `sqrt`, `floor`, `total_cmp` (`min_by` over a fixed `6..9` range).

**3. Would a real player do this?** By trace (fixture variants of play 102: turns −28° to +22°, speeds 0.4–1.0, bases empty, 1st, 2nd, 3rd, 1st+3rd, loaded, 0 and 1 out; game traces over 120 seeds):

- **Up the middle and both mirrors:** correct. Play 102 as frozen: 2B out as cutoff for second from 0.66 s, SS on second. Through the left side (−28°, the left fielder fields it at 118 m after it gets past him): SS goes out at 0.66 s, 2B covers, the relay turns toward third at 4.96 s once the ball is past the LF. Through the right side (+22°): the right fielder fields it and the 2B goes out once the first baseman can no longer field it.
- **Runner on first:** the SS is the cutoff to third from 0.66 s. The 2B idles 0.3 s and then covers second (`CALL_HEARD_S`, as the spec says). **Wrong when an infielder fields it**: P1-1.
- **Runner on second, third, 1st+3rd, loaded:** `through_job` returns nothing (the base is home). Unchanged from main apart from the cover-table fix. The 0.6–0.9 s gaps are pre-existing and listed in TASKS.
- **Double-play depth:** the gate ignores depth and uses `INFIELD_BREAK_R`. The wrong-ball cases of P1-1 include 1-out plays (seed 1/188, 8/89).
- **A ball knocked down after the man has left:** on a new loose ball everyone re-reads (`set_free` → `assign`). The cutoff man's job ends once someone is known to be going for it. Of the 56 runner-on-first cases, 11 involved a second loose ball (a deflection). Over all runner-on-first ground balls fielded by an infielder, second was empty at the moment of the secure on 3 plays on the branch (seeds 25/104, 38/38, 67/226, each fielded by the pitcher) against 4 on main. No force at second was measurably lost.
- **An infielder fields it after all:** the man is sent back to his base when the ball is secured, or 0.3 s after the chaser is known. In the +3° variant he flickered between second and the cutoff for 2 s. P1-1.
- **Misplayed or to the wall:** deep once past the chaser. Correct in the −28° variant and in the unit test (fielded at 110 m).
- **Bunt or slow roller** (speed 0.4–0.7): no early cutoff. The pitcher or the 2B fields; in one variant the 2B idles 1.6 s, which is pre-existing table behaviour.
- **The pitcher asymmetry:** with nobody on, toward second, it sends a middle infielder out on 43 comebackers the pitcher fields (P2-2). With a runner on first, counting the pitcher works for him, but the other infielders' counting fails (P1-1).
- **The 22 uncovered plays:** 14 are with a runner on first: mostly the 0.3 s hand-over at the first call on balls right of second (it happened later on main), plus three new cases of one middle infielder on a cutoff while the other chased. 6 are with nobody on: two new 2B-cutoff-while-SS-chases cases, the rest after the ball was fielded deep. Counting infielder-fielded balls as well, second-uncovered with a runner on first doubles (P1-1).

**4. The loosened test (`318d545`).** Sound; it does not hide a drift. The branch changes no batted-ball physics: only fielders' jobs in `live.rs`. Changed fielding changes outs and baserunners, hence later counts and batters, hence which balls are hit. That is the reshuffle.

The independent check was probe `zz_review_spin` over 192 games (seeds 100–291). The 25°/85 mph cell's median is **1,813 rpm against the table's 1,840 (1.5% off)**. Its 24-seed blocks run 1,519 (35 balls, the test's block, matching the worker's figure), 2,044, 2,050, 1,563, 1,754, 1,744, 1,782. So two of seven blocks are more than 15% off on 31–35 balls. The old rule was a 1-in-3 flake on a ~35-ball cell, not a detector. The new rule ("fail only if more than 15% off **and** outside the median's 99% interval") keeps 15% binding on the large cells, where the interval is narrower, and stops it firing on sampling noise in the small ones. Every other 25–45° cell is within 5% over 192 games.

A fixed input set (seeds frozen against the contact model alone) would be the cleaner fix, but it is not needed for honesty. Raising the minimum cell size from 30 to about 60 would remove the small cells from the 15% test altogether. Optional.

**5. The new tests.**

- **`check_ground_single_jobs` (play 102 and the turned mirror):** largely independent. It checks the spec's standard assignment (jobs 22 and 12 at every tick before the catch, no relay toward third) and the geometry against the world's own fielding point (within `CUT_LINE_M` of the line from second to the ball, out from where he started). The fixture is the owner's play, frozen from main.
- **The 3° turn on the mirror:** honest and documented. The exact mirror changes the world's bounce noise and the SS fields it, so it is a different play. The true mirror is then not covered.
- **The three `live.rs` unit tests:** scenario tests of the spec's table, with `on_segment` and `CUTOFF_M`/`RELAY_OF_THROW_M` geometry. Acceptable, not tautological.
- **Not covered:**
  - a runner on first with a ground ball an infielder fields (P1-1);
  - a comebacker with nobody on (P2-2);
  - the flicker;
  - a ball an outfielder lets by (only the held-at-110 m case, not the loose `deep_ball` branch);
  - re-calls with nobody on.

**6. The measurements.** I touched `live.rs` and `throw_choices.rs`, rebuilt `sim-cli` and ran the standard run on the branch:

- seed 0: runs 4.19, PA 37.7, pitches 237, K 22.5, BB 8.1, HR 2.72%, reached 33.8%, GB .309 (FC .057), LD .673, FB .102, PU .005, out of play after a throw 0.100;
- seed 1000: 4.59, 38.6, 242, 22.3, 8.5, 2.82%, 34.5%, .321 (.057), .670, .099, .007, 0.130.

Every figure matches the ledger row. The main figures, taken from the dp-cover final row, are valid: `git diff acc17a98 904c1ea -- sim/` is empty. The note's outfielder-fielded job counts reproduce exactly. The row is true but incomplete (P2-3).

**7. Spec and docs.**

- **§5.5:** right.
- **§5.7:**
  - "A deep ball", "the runners on base at the pitch" and the coverage-table revision match the code.
  - The "At the first call" bullet contradicts the code (P2-1).
  - Fix 6 is missing from the spec.
- **observations.md:** the 102 tick follows the rules (his words kept, plain note); its claim holds for play 102.
- **TASKS.md:** the new open item is useful; the done parenthesis breaks its rule (P3-2).
- **check-docs:** `python3 bin/check-docs.py` passes (rc 0).

## Verification

**Ran:**

1. Branch build (`CARGO_TARGET_DIR=sim/target/review-probes/mid-target`, seeded by a reflink copy of the worktree's target; changed sources touched): `sim-cli games --n 300 --seed 0` (87 s) and `--seed 1000` (90 s), one after the other. Figures in Q6.
2. `cargo test --workspace` on the branch after touching the changed sources: all pass (502 passed, 0 failed, 4 ignored; 12 min), including the two play-102 tests, the three new unit tests and the spin-belief test.
3. `zz_review_mid` (throwaway example, untracked, deleted): 120 games, seeds 0–119, on main (`904c1ea` source, identical to the worker's main copy) and on the branch, with `BB_COVER_TRACE=1`. Per-play bases, outs, first bounce, first secure and outs, joined to the job lines (`an.py`, `an2.py`, `an3.py` under `sim/target/review-probes/mid/`). The worker's `jobs.py` on seeds 0–59 of both logs reproduces the note's figures exactly.
4. `zz_review_mid_fx` (throwaway integration test, deleted): 132 variants of the play-102 fixture (turns, speeds, bases, outs) with `job_trace`. `zz_review_main` on both builds with the cover trace: the +3° runner-on-first comparison in P1-1.
5. `zz_review_det`: digests with the job trace off, on and off for seeds 0, 1, 7, 51001; all identical.
6. `zz_review_spin`: spin cells over seeds 100–291 (Q4).
7. `python3 bin/check-docs.py`: rc 0.

**Read only:**

- the cover-table change's effect at a deflection (traced 66 cases, all outfielder bobbles; no problem seen);
- the `relay_spot`, `throw_race` and `deep_ball` interplay in `throw_race` (`live.rs` ~6976);
- observations.md and TASKS.md wording.

All probe files were removed from the tree; outputs are kept in `sim/target/review-probes/mid/`.

## Resolution

Resolved (Coordinator, 2026-11-21). P1-1, P2-1, P2-2, P2-3, P3-1 and P3-2 were fixed on `work/mid-infield-jobs` (`a767730`, `ded08b0`, `792c3d1`; the worker follow-up below); P3-3 needed nothing. Because the P1 fix changed about 140 lines of `live.rs`, it had a focused [re-review](2026-11-21-opus-mid-infield-jobs-rereview.md), which confirmed P1-1 fixed on the same 120 games (second empty on infielder-fielded balls with a runner on first 4 → 40 → 5). Left open in TASKS.md "Open sim work" §3: false cutoffs with nobody on that an infielder then fields (mostly the pitcher's comebackers, kept by choice and explained in agent-models §5.7).

### Worker follow-up

Commits after `dc96641` on `work/mid-infield-jobs`; measurements in the [mid-infield note](../calibration/2026-11-21-mid-infield-jobs.md) §5.

- **P1-1, fixed.** New gate `infield_lets_go` (agent-models §5.7, "At the first call"): a middle infielder goes out on the cutoff to second or third from a loose ground ball only while no infielder known to him to be going for it is still in front of the ball (the ball, as everyone sees it now, not yet gone by him along its path by more than `REACH`), his partner is not known to be going for it, by his own estimate no other infielder the ball has not yet gone by can field it inside `INFIELD_BREAK_R`, and toward third (a runner on first) every infielder has already shown himself on this ball. Once out, only a known chaser still in front of the ball or the partner's chase takes him off; the estimate is not asked again, so there is no leave-return-leave flicker. The gate applies to `through_job` and to `relay_job`'s cutoff to second or third while a ground ball is loose. With a runner on first the second baseman's own judgment that the ball is through gives him second at once by the convention (`second_takes_over`), so the shortstop's later departure leaves nothing open. In the review's case (play 102 turned 3° toward first, runner on first) the shortstop now covers second at every tick until the second baseman fields the ball at 4.59 s. Second uncovered while the ball is loose on infielder-fielded ground balls with a runner on first, 120 games: main 4, first build 40, final 5; all runner-on-first ground balls 52 → 102 → 42; early cutoffs fielded by an infielder 56 → 0; force at second 199 → 199 → 200, double plays 114 → 110 → 114. Tests: `runner_on_first_the_shortstop_holds_second_while_the_second_baseman_fields_it`, `a_comebacker_sends_nobody_out_and_leaves_second_covered` (pitcher fields it, nobody on and runner on first), `the_through_cutoff_does_not_flicker` (play 102 turned −14° to +14°, both base states, also checks second is never open with a man on a cutoff with a runner on first). Two of the three fail on `d92dec2` (checked by running them against its `live.rs`).
- **P2-2, fixed except the pitcher toward second, kept on purpose.** The same gate applies with nobody on: early cutoffs fielded by an infielder 145 → 64, none now with an infielder already known to be going for it. The 64 are 44 the pitcher's, 20 other infielders' before their chase was known. I kept the pitcher out of the estimate toward second (not out of the known-chase rule): counting him holds the second baseman on play 102 still until the ball passes the mound (about 0.3 s, the owner's complaint), while leaving him out costs a median 0.02 s (at most 0.18 s; corrected by the re-review, P3-4: a median 0.18 s, at most 0.30 s, on the cutoff) of running out before the pitcher's catch on 44 comebackers in 120 games. The spec says why. The pair: a man whose partner he knows to be going for the ball stays at, or goes to, second; only the 0.3 s before the partner's chase is known can leave both away (5 of the comebackers, second empty 0.1–0.2 s before the pitcher's catch, nobody on). Second uncovered while loose with nobody on: main 83, first build 42, final 43.
- **P2-1, fixed.** agent-models §5.7 rewritten to the code: the "At the first call" bullet lists every condition (including the pitcher's treatment and why), the pair, `second_takes_over`, and the rule of related fix 6 (a man the ball has gone past keeps the cutoff). The header's tail of the old "Cutoffs and relays" paragraph, which my first edit had split off into the coverage-table bullet, is back in its paragraph.
- **P2-3, fixed.** The note's §5 and the ledger row now give every ground ball (runner on first, nobody on, runner in scoring position; early cutoffs and how many an infielder fielded; second uncovered while loose, and at the secure; forces at second and double plays), main against the first build against the final. The note's §4 figures are labelled as the first build's.
- **P3-1, fixed.** `through_job` reads the runners with the candidate's own `throw_choice` (the side man for second; the shortstop's view must agree for third).
- **P3-2, fixed.** The done parenthesis is gone from TASKS.md.
- **P3-3.** Nothing to do.

Tests and standard run on the final build (sources touched): `cargo test --workspace --release`, 505 passed, 0 failed, 4 ignored, including play 102 and its mirror (still fixed: the second baseman is the cutoff for second from the first call, the shortstop on second throughout). Standard run, main → first build → final: runs 4.16 → 4.19 → 4.20 [4.59 → 4.59 → 4.61], GB hits .309 → .309 → .309 [.322 → .321 → .323], reached on BIP 33.6% → 33.8% → 33.8% [34.5% → 34.5% → 34.6%]; all within the seed sets' noise (ledger row of 2026-11-21).
