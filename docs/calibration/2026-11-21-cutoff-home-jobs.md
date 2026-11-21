# The infielders' jobs on a ground single with a runner in scoring position (2026-11-21)

**Job.** TASKS.md §3, "Middle infielders with no job while a ground single is loose", case (a): with a runner on second or third and a ground ball through the infield, a middle infielder often had no job for 0.6–0.9 s while the ball was loose, and the tables could leave the shortstop with nothing, the second baseman at second and the pitcher at first (seed 9 play 9). Branch `work/cutoff-home-jobs`, from main `312789e`. Continues the [mid-infield note](2026-11-21-mid-infield-jobs.md) (§3, "Runner on second or third"). Spec: agent-models §5.7, "With a runner in scoring position" and "Cutoffs and relays"; §5.5, "Where the man in the throw stands".

## 1. What was wrong (main `312789e`)

Game seeds 0–119, with `BB_COVER_TRACE` and a throwaway probe that adds each play's base state, first bounce, first secure and runners' outcomes. Ground balls (first down within the infield dirt, `INFIELD_R`) with a runner in scoring position that an outfielder fielded first: 129. On them a middle infielder had 166 stretches of more than 0.1 s with no job while the ball was loose (118 s in all); the second baseman 0.36 s a play, the shortstop 0.56 s. (The mid-infield note's count, game seeds 0–59 with its `jobs.py`, reproduces on `312789e` as 93 of 147 stretches on plays with a cutoff home, against its 95 of 157 on the reshuffled games of `d92dec2`.)

By the job before and after each stretch:

| Cause | Stretches | Seconds |
|---|---|---|
| **No job from the first call until 0.3 s after the cutoff man home was known** (second baseman then to first, shortstop to third or second) | 70 | 46 |
| A middle infielder out as a deep ball's relay man home and back: mostly a line drive or blooper that first comes down in the dirt, which the deep-ball rule treats as deep while it is up (no job before it, `start → relay4`; and when it came down in front of the outfielder, no job again, `relay4 → cover`) | 54 | 27 |
| A middle infielder who went for the ball and stopped, until the cutoff home was known | 26 | 17 |
| **The shortstop with no job until the ball was fielded**: the first baseman out as the cutoff home on a ball to the left of centre, the second baseman kept on second, the pitcher given first (seed 9 play 9 and 6 others) | 7 | 22 |
| Other | 9 | 6 |

Why: the cutoff man home (the first baseman; the third baseman on a ball fielded left of x = −15 m) went out only once an outfielder was known to be going for the ball (`relay_job`), which is 0.3 s after the first call at the soonest and often 0.6 s, and the others' tables changed only when his job became known, 0.3 s after that. Nobody acted on the standard assignment, which every real infielder carries out as he sees the ball go through. On a ball left of centre the coverage table, with the first baseman out, dealt first to the pitcher and second to the second baseman, so the shortstop had nothing and the pitcher was not behind home: on 13 of the 129 plays that was still the arrangement when the ball was fielded. The pitcher had no job a mean 0.36 s a play (on 119 of 129). First base stood uncovered on 86 plays (median 0.30 s, the hand-over from the first baseman to the second baseman), third on 47.

Seed 9 play 9 (bottom of the first, none out, a runner on second, a high chopper off the plate over the middle, fielded by the centre fielder at (−12.5, 81) m), `312789e`:

| t (s) | P | C | 1B | 2B | 3B | SS |
|---|---|---|---|---|---|---|
| 0.66, the first call | – | home | first | second | third | **–** |
| 0.96 (the left fielder known going) | backup | home | cutoff home | second | third | – |
| 1.26 | **first** | home | cutoff home | second | third | – |
| 1.41–1.71 | first | home | cutoff home, off, on | second | third | goes for it, stops |
| 2.72–4.64 (fielded) | backup | home | cutoff home | second | third | **–** (3.7 s in all) |

On the final build (§2): 0.66 s the shortstop has no job (by his own estimate the pitcher or the second baseman may still reach the chopper: case (b)); 0.96 s the third baseman out as the cutoff home (the left fielder is going for it), the pitcher behind home; 1.26 s the shortstop at third; 1.41–2.01 s he goes for the ball and stops; 2.42–2.61 s the second baseman off second for 0.19 s at a re-call (he still knew the shortstop as on second); from 2.72 s to the catch 3B cutoff, 1B first, 2B second, SS third, P behind home.

## 2. What changed (`sim/crates/bb_game/src/live.rs`)

1. **Who will field it, as everyone works it out** (`through_point`): before anyone is known to be going for the ball, the outfielder whose pre-pitch spot is nearest the ball's ground track as all see it now, at the track's point nearest that spot (the fielding point's construction on a known spot instead of a known chaser).
2. **The cutoff man home by the outfielder** (`home_cutoff_man`): the third baseman on a ball the left fielder fields, the first baseman on one the centre or right fielder fields (the usual convention); a man known to be out on it keeps it when another outfielder takes the ball over. The old side test (x < −15 m of where it was being fielded) handed the job back and forth between the first and third basemen on a centre-field ball near that line (seed 86 play 73; with the early cutoff of item 3 it changed hands every 0.3 s). No parameter is left: the −15 m was a literal.
3. **The cutoff man home goes out at the first call** (`through_job` toward home), on the same gate as the shortstop toward third (`infield_lets_go`): his own read has the ball through with no play for him, no infielder known to be going for it in front of it, by his estimate no other infielder (the pitcher included) can field it in the infield, and every infielder has shown what he is doing on this ball. Toward home, any infielder known to be going for the ball holds him at first, wherever the ball is relative to that man (after the Opus review, P2-1, §6). A chase that becomes known sends him back to his base.
4. **The others by the convention** (`home_cutoff_by_convention`). Each infielder but the catcher who judges for himself that the ball is through (the same facts and his own read and estimate as `second_takes_over` with a runner on first) counts the cutoff man as out in his own working of the tables at once, instead of waiting to see him go. The judgment is kept for the ball once made (`Fielder::through`, so it does not come and go as the estimate moves); a chase known to him ends it (after the review: wherever the ball is); once anyone is known to be on a cutoff or relay, the tables use what is known. It is asked at the first call, at each hearing, and every tick for a man with no job (`through_jobs`: he watches the ball go through). Teammates learn the job he takes 0.3 s later, as always.
5. **The coverage table with the first baseman cutting** (`home_order`): the second baseman covers first and the shortstop second; the pitcher covers no base and backs up home. With the shortstop going for the ball or on a cutoff, the second baseman covers second and first waits for the shortstop to be free. Hand-overs in each man's own table: a free shortstop takes second from a second baseman on it, who goes to first once he knows the shortstop is coming; the second baseman takes first from a pitcher who covered it while the first baseman went for the ball; a first baseman who was on the cutoff this play and has been replaced by a deep ball's relay man, while the ball is out of the infield (loose beyond the dirt or in an outfielder's hands, not thrown), takes first back from the second baseman, who goes to second (narrowed after the review, P3-1, §6).
6. **The pitcher's backup of home before anyone is known to be fielding it** (`through_backup`): by his own judgment that the ball is through, behind home on the line from the `through_point`.

Not changed: the deep ball's relay man; a ball in the air (the conventions wait for it to come down); `relay_job`'s cutoff home once an outfielder is the known lead (no infield gate for the corner men, as on main: review P3-3, TASKS).

Tried and dropped: making the in-air early relay need the man's own read to have the ball coming down beyond the infield dirt. It took the relay man off line drives that drop in front of the outfielder, but left him with no job while the ball was up, and with nobody on it raised the middle infielders' no-job time (0.118 → 0.155 s a play); that is case (d)'s ground, not this one.

## 3. Measurements

Game seeds 0–119, main `312789e` → the first final `e56ddc5` → the build after the review's fixes (§6); ground balls with a runner in scoring position (first down within the infield dirt). The games reshuffle after the first changed play. Two counts are given where they differ (Opus review P3-5): over the **first loose ball** only (from the first call to the first secure, deflections excluded: the scripts `an.py`, `unc.py`, `stretch.py`) and over **every loose ball** of the play (after a deflection or a throw too: `an_all.py`). The first-ball rows are those of the first version of this note, which did not say so; its "0 early cutoffs fielded by an infielder" was a first-ball count and was not true over every loose ball (P2-1).

Ground singles an outfielder fielded first: 129 → 125 → 125.

| | Main | `e56ddc5` | After review |
|---|---|---|---|
| Second baseman, no job while loose, mean a play: first ball; every loose ball | 0.360; 0.421 s | 0.109; 0.174 s | **0.116; 0.181 s** |
| Shortstop, the same | 0.557; 0.643 s | 0.102; 0.191 s | **0.120; 0.209 s** |
| Middle infielders' stretches > 0.1 s with no job, first ball (seconds) | 166 (118 s) | 77 (26 s) | **85 (29 s)** |
| of them begun with the ball on the ground | 98 (76 s) | 20 (6 s) | 28 (8 s) |
| of them begun while it was still up (a liner or blooper coming down in the dirt) | 68 (43 s) | 57 (20 s) | 57 (20 s) |
| the same, every loose ball (2B + SS) | 193 | 105 | 113 |
| Pitcher, no job while loose, mean a play (first ball) | 0.357 s | 0.245 s | 0.252 s |
| First uncovered while loose, first ball (plays; seconds; longest) | 86; 28.6 s; 0.75 s | 27; 7.8 s; 0.73 s | **25; 6.3 s; 0.45 s** |
| the same, every loose ball | 86; 28.5 s; 0.75 s | 24; 7.1 s; 0.73 s | 22; 5.5 s; 0.45 s |
| Second uncovered while loose, first ball | 10; 3.2 s; 0.60 s | 22; 5.5 s; 0.43 s | **23; 5.7 s; 0.43 s** (median 0.30 s) |
| the same, every loose ball | 21; 6.2 s; 0.60 s | 32; 8.6 s; 0.53 s | 33; 8.7 s; 0.53 s |
| Third uncovered while loose, first ball | 47; 17.1 s; 0.60 s | 14; 3.9 s; 0.90 s | **14; 3.9 s; 0.90 s** |
| the same, every loose ball | 51; 18.1 s; 0.60 s | 19; 5.7 s; 0.90 s | 19; 5.0 s; 0.90 s |
| The seed 9 play 9 arrangement at the catch (pitcher at first, shortstop with nothing) | 13 | 0 | **0** |
| The standard arrangement at the catch (1B cutting: 2B first, SS second, 3B third, P behind home; 3B cutting: 1B first, 2B second, SS third) | 82 | 92 | 92 |
| Out-back-out within 1 s while loose, first ball (runner in scoring position / nobody on / runner on first) | 8 / 5 / 1 | 6 / 5 / 1 | 6 / 5 / 1 |

Every ground ball with a runner in scoring position (763 → 765 → 765, every loose ball):

| | Main | `e56ddc5` | After review |
|---|---|---|---|
| Early cutoffs (a man out on a cutoff with no outfielder the known lead) | 1 | 42 | 14 |
| **of them toward home, on a ball an infielder fielded** | 0 | **12** (first ball only: 0) | **1** |
| First uncovered while loose, ball fielded by an infielder | 18 | 28 | 22 |
| Second uncovered, the same | 18 | 19 | 19 |
| Third uncovered, the same | 18 | 20 | 18 |
| Runner on first (every loose ball): early cutoffs home on a ball an infielder fielded | 0 | 8 | 0 |

The one left (seed 5 play 89) comes after the third baseman had the ball and threw it (a loose ball after a throw, with every base covered); the play is the same as on main. First uncovered on infielder-fielded balls, 18 → 22: after a throw gets away, a first baseman whose own cutoff job is over (§6) and the reshuffled games; not traced one by one.

**Second uncovered, 10 → 22 plays (first ball, `e56ddc5`; 23 after the review).** Nine of the 22 are the same plays as on main (both middle infielders going for one ball; a liner in the air with the second baseman out as the deep relay). Ten are new and of one kind: a ground single that gets past the centre fielder becomes a deep ball, the relay job passes from the first baseman to the shortstop, who leaves second, and the second baseman at first goes to second once he knows it, 0.3 s later (before, the second baseman was usually still on second because the shortstop had nothing). Each lasts at most 0.30 s, deep in the play, while the ball is still in the outfield. The other three (0.15–0.22 s) are hand-overs at a re-call or after a chase (seed 9 play 9 at 2.42 s, §1). The longest of all 22 is 0.43 s, against 0.60 s on main. Not fixed: the second baseman could count the relay man from the deep-ball facts himself, as he does the cutoff man; it touches the deep relay's assignment, which a measured relay review should own (agent-models §5.7, Opus infield-fixes P3-4).

**Second's cover changing hands** (the mid-infield note's count, game seeds 0–59, all ground balls an outfielder fielded): 59 → 95 → 97 (main → `e56ddc5` → after review), the hand-overs of item 5.

**Nobody on and a runner on first** (the cases the mid-infield work fixed): middle infielders' no-job time on ground balls an outfielder fields, 2B 0.118 → 0.120 s and SS 0.107 → 0.105 s with nobody on; 0.187 → 0.181 and 0.399 → **0.137** with a runner on first (when the runner from first is seen going home, the cutoff goes home and the new table gives the shortstop second instead of nothing). Early cutoffs fielded by an infielder with nobody on 64 → 64; second uncovered while loose, nobody on 35 → 36, runner on first 45 → 39 (first ball; the same after the review). The first version's take-back of first (item 5) also reached nobody-on plays (Opus review P3-1: seed 32 play 33 and seed 77 play 10 were the first plays to differ from main in their games); narrowed, their job traces are identical to main's, and the take-back's signature with nobody on is back to main's count.

**The mid-infield note's figure** (game seeds 0–59, every ground ball an outfielder fields, its `jobs.py`): 2B 0.164 → 0.112 → 0.115 s, SS 0.306 → 0.099 → 0.102 s a play; stretches over 0.1 s on plays with a cutoff home 93 → 57 → 58 (main → `e56ddc5` → after review).

**Outcomes** (game seeds 0–119, balls first secured by an outfielder): ground singles with a runner on second, the runner scoring 40 of 62 → 39 of 59 (the same after the review); outs at home 1 → 1; outs on the bases 20 → 18 (outfield assists on these plays); on balls in the air 50/91 → 51/91, 1 → 1, 35 → 36. Within a few counts: the jobs change where men stand, not the throw home.

**Standard run** (`sim-cli games --n 300`, seeds 0 [1000]): main `312789e` (the bounce-track final's figures; its sim code is the same) → `e56ddc5` → final `1c06fdb` (after the review). Runs 4.08 → 4.07 → **4.08** [4.44 → 4.46 → **4.46**]; GB/LD/FB/PU hits .310/.671/.103/.005 → .309/.670/.103/.005 → .310/.671/.102/.005 [.316/.665/.100/.007 → .315/.663/.099/.007 → .315/.663/.099/.007]; HR/PA 2.73% → 2.72% → 2.73% [2.95% → 2.98% → 2.98%]; K% 22.6 [22.2 → 22.1 → 22.1]; BB% 8.0 [8.4 → 8.5 → 8.5]; reached on BIP 33.8% → 33.7% → 33.7% [34.0%]; PA/team-game 37.7 [38.1]; out of play after a throw 0.133 → 0.153 → 0.153 [0.113 → 0.120 → 0.120] a game (six balls more in 300 games, within its noise); rules warnings {}; time 4,540 → 4,618 → 4,523 [4,485 → 4,567 → 4,505] ms per game per core (run-to-run timing). Every move is inside the seed sets' noise; every digest changed (fielders' paths change, and the games after).

## 4. Tests (`tests/throw_choices.rs`)

- `seed_9_play_9_a_single_to_left_centre_with_a_runner_on_second_gets_the_standard_jobs` (fixture `9-9.json`, frozen from `312789e`): at the catch the corner man on the cutoff home and the standard arrangement for him (the left fielder went first, the centre fielder took it over, so the third baseman is the cutoff); the pitcher behind home; second never open longer than the 0.3 s knowledge lag. Fails on `312789e` (the second baseman on second, not first).
- `a_ground_single_with_a_runner_in_scoring_position_gives_every_infielder_his_job`: play 102's grounder with a runner on second, turned every degree from −30° to +30° and mirrored, and two game plays frozen from `312789e` (seed 65 play 43, a runner on second, through the right side to the right fielder; seed 30 play 35, a runner on third, through the left side to the left fielder), each turned up to 6° either way. On each variant fielded in front of an outfielder: the standard jobs at the catch, and once everyone can know no infielder is going for the ball, no middle infielder or corner man without a job for more than 0.3 s at a stretch. Fails on `312789e` (play 102 mirrored, turned −17°: the shortstop with no job 0.66–0.97 s).
- `seed_8_play_208_the_first_baseman_holds_first_while_the_second_baseman_runs_the_ball_down` (fixture `8-208.json`, frozen from `e56ddc5`; added after the review, §6): one out, a runner on second, the second baseman runs down a ball off his glove and fields it 49 m out; first covered at every tick to the catch, and the first baseman never on the cutoff home while the second baseman is going for it. Fails on `e56ddc5` (first open at 2.28 s, the first baseman on the cutoff).
- Play 102, its mirror, the runner-on-first, comebacker and flicker tests pass unchanged. The isolation harness checks the new step (`through_jobs`) and the new private state (`Fielder::through`).

`cargo test --workspace --release` on the final build, sources touched: 512 passed, 0 failed, 4 ignored (511 on `e56ddc5`, before the seed 8 play 208 test). One existing test, `a_retired_cutoff_man_with_no_job_stops`, failed on the first version of the first baseman's taking first back (with an infielder holding the ball he took first from the second baseman); that hand-over now applies only while the ball is out of the infield.

## 5. Still open

- **A liner or blooper that comes down in the dirt** (57 of the 85 remaining stretches after the review; 57 of 77 on `e56ddc5`): while it is up nobody acts on a single, and the deep-ball rule can send a middle infielder out as the relay man and back. Shares its ground with case (d) (line-drive and fly singles).
- **While an infielder is still going for the ball** the others wait on him (case (b)): the test counts from the moment everyone can know no infielder is going.
- **Second for 0.3 s when a single gets past the centre fielder** (§3): for the relay review (TASKS).
- **`relay_job`'s cutoff home has no infield gate** once an outfielder is the known lead (as on main; review P3-3, TASKS): seed 191 play 125, third empty 0.6 s while the shortstop fields a chopper.
- The pitcher still has no job until he judges the ball through (0.245 s a play), and first base waits for the shortstop when he is the one going for the ball.
- Cases (b)–(f) of the TASKS item do not share this cause; (b) and (d) border it as above.

## Reproduce

- The fixtures: `tests/throw_choices.rs` (the job trace from `replay_jobs`).
- The trace probe (kept outside the tree, `sim/target/codex-probes/cuthome/`): copy `zz_cut_home.rs` into `crates/bb_game/examples/`, build it on each commit, `BB_COVER_TRACE=1 zz_cut_home 120 0 2> tr.log`, then `an.py before.log after.log` (counts by base state), `stretch.py tr.log` (stretches by the job before and after), `unc.py before.log after.log` (uncovered bases), `outcomes.py`, and the mid-infield `../mid/jobs.py` on the seeds 0–59 lines.

## 6. The Opus review's fixes (2026-11-21, [review](../reviews/2026-11-21-opus-cutoff-home-jobs-review.md))

- **P2-1, early cutoff home on a ball an infielder runs down after it came off him.** The gate toward home let the first baseman go once the ball had passed the infielder known to be going for it; after a deflection that infielder chases it from behind and fields it 50–57 m out, and first stood empty 0.5–2.75 s (20 plays in game seeds 0–119, 9 in 120–239). Fixed by the reviewer's first option: toward home any infielder known to be going for the ball holds the cutoff man (and the others' convention) wherever the ball is. I kept it because the trace offers nothing better that a teammate can know: whether the chaser will still reach the ball in the infield is his own private read. Over every loose ball, early cutoffs home on infielder-fielded balls 12 → 1 (runner in scoring position) and 8 → 0 (runner on first); the one left is after a throw, with every base covered. Cost on clean singles: the middle infielders' no-job time rises 0.109 → 0.116 s (2B) and 0.102 → 0.120 s (SS) a play, stretches 77 → 85, as the corner man and his cover wait for a man who went for the ball and stops. New test from seed 8 play 208 (§4).
- **P3-1, the first baseman's take-back of first.** Narrowed to what the spec said: only a first baseman who was on the cutoff this play (his shown jobs) and has been replaced, with the ball loose beyond the infield dirt or in an outfielder's hands, not thrown. Seed 32 play 33 and seed 77 play 10 (nobody on) now trace identically to main; with nobody on the take-back's signature is back to main's count. `a_retired_cutoff_man_with_no_job_stops` still passes.
- **P3-5.** §3 now gives the first-ball rows and the every-loose-ball rows side by side.
- **P3-4.** The note's date is 2026-11-21 (renamed from `2026-11-22-…`); links, the ledger, the spec and two code comments follow.
- **P3-2, P3-3.** Deferred to TASKS (§5).

