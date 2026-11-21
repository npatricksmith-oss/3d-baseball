# Re-review: the review fixes on the middle infielders' jobs (`work/mid-infield-jobs`)

**Reviewer:** Opus (independent; wrote none of the change). **Date:** 2026-11-21.
**Range:** `dc96641..792c3d1`. Sim code is in `a767730` and `ded08b0` (`sim/crates/bb_game/src/live.rs`, `tests/throw_choices.rs`); the rest is docs. Worktree `.claude/worktrees/runner-decision`.
**Against:** the first review, [2026-11-21-opus-mid-infield-jobs-review.md](2026-11-21-opus-mid-infield-jobs-review.md): P1-1, P2-1, P2-2, P2-3, P3-1, P3-2 and its `### Worker follow-up`.
**Not re-reviewed:** the first build.

## Summary

**P1-1 is fixed.** On my own build of `792c3d1`, the first reviewer's probe over the same 120 games gives exactly the worker's figures. The trace log is byte-identical to the worker's `tr-fix2.log`.

- Runner on first: second uncovered while the ball is loose on infielder-fielded balls 4 → 40 → **5**. Early cutoffs fielded by an infielder 56 → **0**.
- Forces at second 199 → **200**; double plays 114 → **114**.
- The frozen +3° runner-on-first play now keeps the shortstop on second throughout.
- The extra play against main's 4 is not a regression of the old kind (Q2).
- No new input reads the world.
- The gate on `relay_job` changed no play other than a loose ground ball.

The fix brings new job flip-flops of its own, smaller and away from the force:

- **With nobody on, a middle infielder goes out at the first call, is pulled back 0.3 s later, then goes out again.** This happens on 22 plays in 120 games (main 8, first build 11). The follow-up's "no leave-return-leave flicker" is not true at game level, and the flicker test does not catch it (P2-1).
- **With a runner on first, the second baseman starts for second at the first call on 43 plays and drops to no job 0.3 s later**, 32 of them on balls an infielder fields; main has none. Once, second was left empty for 0.15 s with the shortstop out (P3-1).
- **Smaller points:**
  - a second baseman covering second by his own choice is taken off it for the cutoff on comebackers (P3-2);
  - two of the three tests are weaker than claimed (P3-3);
  - the comebacker figure and four position labels in the spec, note and ledger are off (P3-4).

## Findings

### P2-1. With nobody on, the cutoff man for second goes out, comes back and goes out again (22 plays in 120 games, against 8 on main)

**Where:** `live.rs:4718-4741` (`infield_lets_go`: the known-chaser test at 4725 applies to a man already out), with `through_job` at 4684.

**What happens.** At the first call nobody has been heard yet. `known(j)` only reports what a man showed 0.3 s ago, so no chase is known. Toward second, therefore, only the estimate holds the man back, and that estimate leaves out the pitcher. He goes out. 0.3 s later the chasers become known. If one of them is still in front of the ball, the "once out" rule pulls him off, and he goes idle (braking) or to his table job. When the ball has passed that chaser he goes out again.

**Shown.** Probe `rr_flick2.py` / `rr_flick3.py` on the 120-game trace counted a middle infielder whose cutoff ends and restarts toward the same base within 1 s while the first ball is loose:

| | Main | First build | Final |
|---|---|---|---|
| Nobody on | 8 | 11 | **22** |
| Runner on first | 1 | 13 | 3 |
| Runner in scoring position | 10 | 3 | 3 |

Of the 22 with nobody on, the man was taken off by a known chaser in front of the ball:

- the pitcher, 5 (the estimate leaves him out, the known-chase test does not);
- the first baseman, 5;
- the third baseman, 4.

Of the rest, 4 were the man's own chase and 3 his partner's.

**Examples (final build):**

- Seed 33 play 150: the shortstop is out at 0.65 s. At 0.95 s, with the third baseman's chase now known, he goes to cover third. At 1.38 s he is out again, and the left fielder fields it.
- Seed 18 play 91: the second baseman is out at 0.65 s, idle at 0.95 s (the first baseman is known to be going), and out again at 1.60 s.
- Seed 10 play 52: the shortstop is out at 0.68 s, idle at 0.95 s (the pitcher), and out at 1.02 s.

**Would a real player do this?** No. A real second baseman or shortstop does not start out, stop dead and start out again inside a second. The force is not at stake (nobody on), and I found no outcome cost. It is plain on screen, though, and the follow-up says the flicker is gone.

`the_through_cutoff_does_not_flicker` does not see it. Its turns of play 102 (−14° to +14° in 2° steps) never produce it. Its flicker assertion also passes on the first build (P3-3).

**Fix.** Make the rule that takes a man off the same as the rule that sends him out. Toward second with nobody on, options:

- (a) Once out, only his partner's chase (or a secure) takes him off. Known non-partner chasers do not, as the estimate already treats the pitcher. Cost: he stays on the cutoff until the catch on some balls a corner man fields (as P2-2 accepted for the pitcher).
- (b) Toward second too, wait until every infielder is known, as toward third. This costs about 0.3 s on play 102, which the owner complained about.

I recommend (a). Add a frozen nobody-on case from one of the games above (e.g. seed 33 play 150) to the flicker test.

### P3-1. `second_takes_over` fires at the first call on balls an infielder then fields, and is judged afresh every tick

**Where:** `live.rs:4634-4651`; used in `cover_table` (5017) and `cover_after_chasers` (5079).

**The gap.** The convention uses the same facts as the shortstop's departure, minus two of them: the estimate of the other infielders, and waiting until every infielder is known. At the first call nothing is known. A second baseman whose own read has the ball through therefore takes second at once. 0.3 s later the first baseman (usually) is known to be going, the convention lapses, and he drops to no job. That is `go_idle`: he brakes to a stop, which is main's behaviour for him on those balls.

**Shown (`rr_two.py` and an inline count).** Runner on first, ground balls, both middle infielders on second at once while the ball is loose: main 14, final 73. On 43 of them the second baseman's next job is none (main 0). 32 of those are balls an infielder fields: the first baseman 30 times in total, the pitcher 11. Examples: seed 1 play 7, seed 3 play 169, seed 4 play 129. In each the role array goes `[.., 1, 12, 13, 12, ..]` → `[.., 1, 0, 13, 12, ..]` at +0.3 s.

**The flip.** The convention also re-reads his own read every tick, without the "once out" stickiness the shortstop has. Seed 52 play 234 (runner on first):

- 1.35 s: the second baseman goes to second.
- 1.50 s: he drops to no job while the shortstop is already out toward third (23), so **second is empty for 0.15 s**.
- 1.65 s: he is back on second.

Overall second is still uncovered less often than on main (42 against 52 plays; median 0.20 s against 0.30 s), so this is a P3.

**Fix.** Give `second_takes_over` the shortstop's estimate step (the pitcher counted, as toward third). Once he has taken second this way, keep it unless a known chaser in front of the ball or the shortstop's chase ends it, mirroring `infield_lets_go`'s `out` branch. State both in §5.7.

### P3-2. On a comebacker, a second baseman covering second by his own choice is taken off it for the cutoff

**Where:** `live.rs:4684-4708` (`through_job` does not look at why the man is where he is) and the pitcher exclusion at 4737.

**Shown.** Two of the five nobody-on comebackers that leave second empty (spec §5.7: "On 5 of those…"):

- Seed 24 play 54: the second baseman is on `Cover(2)` (12) at 0.66 s. At 0.79 s `through_job` makes him the cutoff (22). The pitcher fields at 0.93 s, and second is empty for 0.13 s.
- Seed 32 play 107: the same pattern, 12 → 22 at 0.81 s.

In both, his own read already had his partner fielding the ball; that is why he was covering second. The estimate inside `through_job` then says no infielder other than the pitcher fields it inside 51.6 m. One man is acting on two judgments that contradict each other.

The spec blames the 0.3 s before the partner's chase is known. In these two cases the man already acted on his own judgment that the partner has it.

**Cost:** nobody on, 0.1–0.25 s, no force.

**Fix.** A man covering second on his own `CoverSecond` decision on this ball is not a through-job candidate. Or his partner is never left out of his estimate when he made that decision. Either removes these without counting the pitcher.

### P3-3. Two of the three new tests are weaker than the follow-up implies

`tests/throw_choices.rs`. I ran the final test file against the first build `d92dec2`, whose sim is identical to `6028bed` apart from the spin test's tolerance (`git diff 6028bed d92dec2 -- sim/` touches only that unit test).

- **`runner_on_first_the_shortstop_holds_second_while_the_second_baseman_fields_it` (431):** fails on the first build ("at 1.74 s the shortstop is not on second … 23"). It is independent and pins P1-1.
- **`the_through_cutoff_does_not_flicker` (460):** fails on the first build, but only through its second assertion (second open with a man on a cutoff, turn −2°). With that assertion disabled, its **flicker assertion passes on the first build**. The review's flicker was at +3°, and the test's turns are even degrees. So it does not test what its name says, and it misses P2-1's game-level cases.
- **`a_comebacker_sends_nobody_out_and_leaves_second_covered` (444):** **passes on the first build**. At −8° the first build already sends nobody out, so it does not pin P2-2, the comebacker early cutoff. The follow-up says only "two of the three fail", which is true; but this test guards nothing the fix changed.

**Fix:**

- add +3° (and ±1°) to the flicker test's turns, and a frozen nobody-on game case (P2-1);
- turn the comebacker to an angle where the first build sends the ball-side man out, or replace it with a frozen comebacker from the games (seed 24 play 54, after P3-2).

### P3-4. The comebacker figure and four position labels in the records do not match the trace

**The comebacker time.** agent-models §5.7 and the ledger row say the ball-side man is out "a median 0.02 s and at most 0.18 s before the pitcher had it" on the 44 comebackers. From the same trace (byte-identical to the worker's), his time on the cutoff while the first ball is loose is a median **0.18 s**, at most **0.30 s**: 37 clean catches, median 0.18, max 0.30; 7 after a deflection, median 0.17. The 0.30 s cases are the ones where the pitcher's chase became known and took him off. 0.02 s is the minimum, not the median. "A few centimetres of running" is still about right (0.3 s from a set start).

**The position labels.** The note's §5 table labels the 64 early cutoffs fielded by an infielder as "pitcher 44, 3B 5, 2B 5, SS 6, 1B 4". The probe's fielder numbers (1 P, 3 1B, 4 2B, 5 3B, 6 SS) give pitcher 44, **1B 5, 2B 5, 3B 6, SS 4**.

**Fix:** correct the figures in §5.7, the ledger row and the note.

## Answers to the questions

### 1. Invariant 2: what each new input reads

No world read. None of the new inputs reads another fielder's true intent, job or plan before that man could have shown it.

- **"An infielder he knows to be going for the ball"** (`known(j)` → `shown_then`): what `j` showed `CALL_HEARD_S` ago. That is heard or seen. `this_ball` compares the time it was shown with `ball_since`, a public fact.
- **"His partner is known to be going"** (`partner_going` → `known(8 - who)`): the same, heard.
- **"Every infielder has shown what he is doing"** (`known(j).2`): whether `j`'s newest show at least 0.3 s old is on this ball. `publish` pushes every fielder on the first tick after a new ball, so this is heard. In practice it means "0.3 s after the first call on this ball". A man who showed `Idle` counts as shown.
- **"The ball has passed that man"** (`ball_gone_by`):
  - `ball_at(self.t)`: the ball's current position and velocity, the existing "as everyone sees it" convention. The velocity's direction is used as well as the position, which is a visible motion, not its future path.
  - `self.f[j].pos`: the teammate's visible position.
- **The estimate** (`seen_intercept(j, pred)`): his own read and the teammate's visible state, known speed and reaction. Unchanged from the first build.
- **His own state:** `self.f[who].role` and `read_ball` (whether he is already out, whether his read is fresh).
- **P3-1's fix:** `relay_base(false, side)` and `relay_base(false, 5)` read the runners with each candidate's own `throw_choice`. Toward third both views must agree. Today every fielder has the same parameters, so there is no effect.
- **`second_takes_over`:** the second baseman's own belief, role and read freshness, plus `known` and `ball_at` as above. It is used only when `o == 3` works out his own tables.

### 2. Is P1-1 really fixed?

Yes.

**The 120-game probe.** `zz_review_mid` was built on `792c3d1` with sources touched, seeds 0–119, `BB_COVER_TRACE=1`, analysed with the first reviewer's `an.py`. Its log is byte-identical to the worker's `tr-fix2.log`, and every claimed figure reproduces:

- Runner on first:
  - ground balls 699;
  - early cutoffs 25, none fielded by an infielder;
  - second uncovered while loose 42, on infielder-fielded balls 5;
  - at the infielder's secure 1;
  - force at second 200 of 265 chances; double plays 114.
- Nobody on:
  - early cutoffs 308, fielded by an infielder 64;
  - second uncovered while loose 43 (infielder-fielded 14).
- Runner in scoring position: second uncovered 8.
- `an3.py`, ground balls an outfielder fields, seeds 0–59: 46 → 22 → 14.

**The 5 against main's 4.** Three are the same plays as on main, with identical traces: seed 27 play 111, seed 38 play 38, seed 85 play 133. Main's seed 1 play 79 is gone in the reshuffle. The two new ones:

- **Seed 1 play 230**, two out: a ball five fielders chase (`[1, 14, 11, 1, 1, 1, 1, ..]`); second is empty 0.40 s before anyone has a cutoff. That is the pre-existing everyone-chases kind, not the gate.
- **Seed 30 play 31**, two out:
  - At 0.96 s the right fielder is known to be going, and the shortstop takes the ordinary cutoff to third.
  - At 1.41 s the second baseman starts for the ball after a re-call.
  - The shortstop comes back to second at 1.71 s, when that chase is known, so second is empty for 0.30 s.
  - The second baseman fields it at 46.6 m.

  That is the 0.3 s knowledge lag; main would have kept the shortstop out until the lead changed. Not a regression.

**The frozen play.** Play 102 turned +3° with a runner on first is now `runner_on_first_…`. It passes on the final build: the second baseman fields at 51 m, and the shortstop is on `Cover(2)` at every tick from the first call to the catch. It fails on the first build at 1.74 s, as the first review saw.

**The 25 early cutoffs with a runner on first.** All went to balls an outfielder fielded: left fielder 11, centre fielder 12, right fielder 2. The shortstop goes out toward third 0.3–0.5 s after the first call, once every infielder is known and the ball is past them, with the second baseman already on second by the convention. Second was empty on 2 of them, for 0.15 and 0.28 s. That is what a real shortstop and second baseman do on a single through the infield.

Seed 17 play 189 is the only same-base off-on (1.40–1.55 s): his own read briefly put his intercept inside the infield, and then the centre fielder became the lead. That leaves the runner-on-first flicker at 3 plays in 120 games against 13 on the first build.

**The 42 runner-on-first plays with second empty, all ground balls:**

- fielded by an outfielder 35, by an infielder 5, never secured 2;
- median 0.30 s empty, max 0.60 s; 7 over 0.31 s;
- main: 52 plays, median 0.30 s, max 1.05 s; 16 over 0.31 s.

33 involve a middle infielder on a cutoff, median 0.20 s: mostly the 0.3 s hand-over on balls fielded deep, plus the P3-1 flips. Better than main on every measure; what a real pair does, apart from P3-1.

### 3. New faults from the gate

- **Deadlock:** none. `publish` runs every tick once the play is assigned, and on a new ball every fielder pushes a fresh show (`e.at < since`). So "every infielder known" becomes true 0.3 s after the first call on each ball, deflections included. It is a fixed delay, not a condition that can hang. On the 25 runner-on-first early cutoffs the shortstop went out on balls plainly in the outfield every time.
- **Stranding:** none. Once out, `infield_lets_go` keeps him out. But `through_job` needs the ball loose, and `relay_job` returns nothing once an infielder holds it, so a secure by any infielder, known chaser or not, ends the job on that tick (`update_relay`). The one residual is P2-1's pull-back, the opposite of stranding.
- **The gate on `relay_job`.** I instrumented an untracked copy of `792c3d1`: the gate in `relay_job` logs whenever it withholds a job the rest of the function would have given. Behaviour was unchanged; the per-play lines were identical to the trace run. Over the same 120 games it withheld a job on **16 plays, all loose ground balls**: 13 with nobody on, 3 with a runner on first. It never fired on a fly, a liner, a ball first down beyond the infield dirt, or a ball in an outfielder's hands, so its `partner_going` term had no effect outside ground balls.
- **The 5 comebackers** (second empty 0.1–0.25 s before the pitcher's catch, nobody on): acceptable, not a P2. There is no force and the play is at first. Two of them are P3-2's own-cover case, which is cheap to fix.

### 4. The three new tests

They are independent: frozen fixtures, the spec's job codes, and premises asserted (who fields it).

- **Fail on the first build, as claimed:** `runner_on_first_…` (pins P1-1) and `the_through_cutoff_does_not_flicker` (only through its second-open assertion; its flicker check passes there).
- **Passes on the first build:** `a_comebacker_…`.

Details in P3-3.

### 5. Spec and records

**agent-models §5.7** now states the rule the code implements:

- the four conditions, the pitcher's treatment and why, the pair, and "toward third, every infielder known";
- the once-out rule, `second_takes_over`, related fix 6, and the `relay_job` gate.

What it does not state:

- that "known" at the first call is empty, so toward second the man goes out on his estimate alone and can then be pulled back (P2-1);
- that `second_takes_over` is re-judged every tick and lapses into no job (P3-1).

Its comebacker figure is off (P3-4). P2-1 of the first review is otherwise fixed.

**The note §5 and the ledger row** match my rerun in every count, apart from P3-4's comebacker figure and the note's four position labels. The ledger's "Nothing unexplained" holds; its open list should add P2-1 if that is deferred.

**TASKS.md:** the done parenthesis is gone (P3-2 fixed).

## Verification

**Ran:**

1. **Probe on the final build.** `zz_review_mid` (untracked copy, deleted afterwards) built on `792c3d1` with `CARGO_TARGET_DIR=sim/target/review-probes/mid-target` after touching the changed sources. `BB_COVER_TRACE=1 zz_review_mid 120 0` ran in 3.5 min. Output is `sim/target/review-probes/mid/tr-final.log`, byte-identical to the worker's `sim/target/codex-probes/mid2/tr-fix2.log`.
2. **Analysis.** The first reviewer's `an.py`, `an2.py` and `an3.py` against `tr-main.log`, `tr-br.log` and `tr-final.log` (output `an-final.txt`). My scripts in `sim/target/review-probes/mid/`:
   - `rr_unc.py`, `rr_cut.py`: second uncovered and the early cutoffs, per play, with durations;
   - `rr_flick.py`, `rr_flick2.py`, `rr_flick3.py`: cutoff restarts (P2-1);
   - `rr_first.py`, `rr_two.py`: first-base cover and two men on second (P3-1);
   - inline counts for the comebacker times (P3-4).
3. **Gate instrumentation.** An untracked copy of `792c3d1` (`git archive` into `mid/fin-src`, deleted afterwards) with a log line in `relay_job`'s gate. `ZZ_GATE=1 zz_review_mid 120 0` gave per-play lines identical to run 1 (`gate.log`, summary `rr-gate.txt`).
4. **The new tests on the first build.** `d92dec2` exported to `mid/fb-src` (deleted afterwards) with the final `tests/throw_choices.rs`; sources touched, then `cargo test --release -p bb_game --test throw_choices`. 18 passed and 2 failed (`runner_on_first_…`, `the_through_cutoff_does_not_flicker`), with the comebacker test passing (`rr-fb-tests.txt`). The flicker test with its second-open assertion disabled passes (`rr-fb-flick.txt`).

   My first attempt shared the target directory with the instrumented copy, and cargo reused that copy's `bb_game`, because a `git archive` gives files old mtimes. Every test passed. I discarded that run and reran after touching every source.
5. **Workspace tests.** `cargo test --workspace --release` on the worktree after touching every source (`bb_game` recompiled from the worktree; `rr-test-ws.txt`): **505 passed, 0 failed, 4 ignored**, the worker's count.

All throwaway copies (the worktree example, `fin-src`, `fb-src`) were deleted. The outputs stay in `sim/target/review-probes/mid/`.

**Read only:**

- the diffs of `live.rs` and `throw_choices.rs`;
- `known`, `shown_then`, `publish`, `seen_intercept`, `ball_at`, `relay_base`, `cutoff_job`, `update_relay` and the tick loop;
- agent-models §5.7, the note §5, the ledger row, the TASKS.md edit, and the first review's follow-up.

**Not done:** the 300-game standard run, which is the Coordinator's before merging.

## Resolution

Resolved (Coordinator, 2026-11-21). Fixed on `work/mid-infield-jobs` (`1e3d045`, `2a72c11`; the worker follow-up below): P2-1 (toward second, a man out on the cutoff is taken off only by his partner's chase or a catch; out-back-out with nobody on 8 → 22 → 8 in 120 games), P3-1 (the second baseman's take of second uses the same estimate and holds), P3-3 (the flicker test fails on `792c3d1`; the comebacker test's comment says it is a regression guard) and P3-4 (the comebacker figure is a median 0.18 s, at most 0.30 s; position labels). Deferred to TASKS.md "Open sim work" §3: P3-2 (a second baseman covering second by his own choice taken off it for the cutoff on comebackers). No third review: the round is small and test-covered. The Coordinator merged main into the branch, touched the sources and ran `cargo test --workspace --release` (505 passed, 0 failed, 4 ignored) and the standard run (4.19 [4.59], ground-ball hits .308, matching the ledger row).

### Worker follow-up

Commits `294b2b2` (P2-1, P3-3) and `1e3d045` (P3-1, TASKS), then the docs; measurements in the [mid-infield note](../calibration/2026-11-21-mid-infield-jobs.md) §6, from the reviewer's probe and scripts over game seeds 0–119.

- **P2-1, fixed by option (a).** Toward second (nobody on), once a man is out only his partner's chase or a catch takes him off (`infield_lets_go`); a known chase in front of the ball no longer pulls him back. Toward third (runner on first) unchanged: a known chase in front of the ball still sends the shortstop back to second. Out-back-out within 1 s while loose (`rr_flick2.py`), main → `792c3d1` → now: nobody on 8 → 22 → **8**, runner on first 1 → 3 → 3, runner in scoring position 10 → 3 → 3. The P1 counts hold: runner on first, second uncovered while loose on infielder-fielded balls 4 → 5 → **5**, all ground balls 52 → 42 → 41; early cutoffs fielded by an infielder 0 → 0 → **0**; forces at second 199 → 200 → 198, double plays 114 → 114 → 113 (263–267 chances, reshuffled games).
- **P3-1, fixed.** `second_takes_over` now asks the shortstop's estimate of the other infielders (pitcher counted) and, once the second baseman is on second, only a known chase in front of the ball or the shortstop's chase ends it (his read and estimate are not asked again). Runner on first, second baseman from second to no job while loose: main 17 (0 fielded by an infielder) → `792c3d1` 56 (32) → now **16 (1)**. Spec §5.7 states both.
- **P3-2, deferred to TASKS.md** (the comebacker item (f), with seeds 24/54 and 32/107 and the suggested fix).
- **P3-3, fixed.** `the_through_cutoff_does_not_flicker` checks the out-back-out directly (a cutoff, a stretch with no job, a base or a backup in which neither he nor his partner went for the ball, a cutoff again), over turns of play 102 every degree from −4° to +4° (+3° included) and every 2° beyond, both base states, plus three nobody-on plays frozen from `792c3d1` (seeds 33/150, 10/52, 18/91). It fails on `792c3d1` (seed 33 play 150: the shortstop out, back, out again at 1.38 s) and passes now. Two adjustments, stated in its comment: a stretch in which the man or his partner went for the ball is not counted (turn +1: he chased, pulled up and went back on the line; turn +1 with a runner on first: the shortstop held second while the second baseman chased); and with a runner on first, second may be open for at most the 0.3 s knowledge lag (turn +1: the second baseman starts for the ball as the shortstop leaves). `a_comebacker_sends_nobody_out_and_leaves_second_covered` keeps its angle; its comment now says it is a regression guard only, passing on the first build too.
- **P3-4, fixed.** Re-measured on the final build: the pitcher's comebackers keep a man on the cutoff a median 0.18 s, at most 0.30 s, before the catch (44 in 120 games; same on `792c3d1`). Corrected in agent-models §5.7, the ledger row, the note §3, and my first follow-up. The spec now says in one sentence why leaving the pitcher out is still right at that cost (under half a metre of running, against about 0.3 s of standing on every ball up the middle if he is counted). The note §5 labels are corrected (P 44, 1B 5, 2B 5, 3B 6, SS 4).

Final build: `cargo test --workspace --release` 505 passed, 0 failed, 4 ignored, including play 102 and its mirror (the second baseman the cutoff for second from the first call, the shortstop on second). Standard run, main → final: runs 4.16 → 4.19 [4.59 → 4.59], GB hits .309 → .308 [.322 → .321], reached on BIP 33.6% → 33.7% [34.5% → 34.5%]; ledger row of 2026-11-21.
