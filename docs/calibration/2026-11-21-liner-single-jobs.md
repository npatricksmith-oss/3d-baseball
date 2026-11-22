# The infielders' jobs while a line drive or blooper falls in (2026-11-21)

**Job.** TASKS.md §3, "Middle infielders with no job while a ground single is loose", cases (a) and (d): a liner or blooper that first comes down in the dirt got no single's jobs while it was up, and a line-drive or fly single with nobody on had no cutoff man for second. Branch `work/liner-single-jobs`, from main `445ef2c`. Continues the [cutoff-home note](2026-11-21-cutoff-home-jobs.md) §5 and the [mid-infield note](2026-11-21-mid-infield-jobs.md) §3. Spec: agent-models §5.7, "A ball in the air that will come down in front of an outfielder", "Cutoffs and relays" (the deep ball) and "The cutoff to second".

## 1. What was wrong (main `445ef2c`)

Game seeds 0–119 with `BB_COVER_TRACE` and the cutoff-home trace probe. The balls counted here were still in the air at the first call, came down before anyone caught them, and were first secured by an outfielder: 1,345 of them. They are grouped by where they first came down: on the infield dirt (`INFIELD_R`, 47.4 m), in the shallow outfield (short of `RELAY_DEEP_M`, 80 m) or deep. A stretch is a middle infielder with no job for more than 0.1 s while the first ball was loose.

| Base state, first down | Plays | Stretches begun while it was up | after it came down |
|---|---|---|---|
| Runner in scoring position, dirt | 73 | **57** (20.3 s) | 4 |
| Runner in scoring position, shallow | 130 | **144** (57.9 s) | 22 |
| Runner on first, dirt | 83 | 73 | 17 |
| Runner on first, shallow | 112 | 126 | 21 |
| Nobody on, dirt | 176 | 145 | 5 |
| Nobody on, shallow | 310 | **344** (160.1 s) | 54 |
| Deep (all base states) | 461 | 483 | 16 |

(The 57 are the cutoff-home note's "57 of 85". Its count includes every ball first down in the dirt, grounders too: 85 stretches of more than 0.1 s with a runner in scoring position on balls an outfielder fielded. Here, by its `stretch.py`, it is 85 on `445ef2c`.)

The causes:

- **Nobody acted on a single while the ball was up.** The conventions of the ground single (the cutoff man out at once by his own read, the others counting him out) all waited for the ball to come down on the dirt (`!ball_still_up`).
- **The deep-ball rule sent a middle infielder out as the relay man on a ball falling in front of an outfielder, and back.** While a ball is up, where it is being fielded is the point of its ground track nearest the outfielder going for it: his own feet, 85–97 m out. So a liner falling in front of a deep centre fielder counted as a deep ball, and the relay man went out early. When it came down short, the corner man or nobody took over. With a runner in scoring position, 110 of the 144 shallow stretches begun while the ball was up were a middle infielder with no job at the first call who then went out as the relay man (`start → relay4`), and all 22 after it came down were that relay man coming back (`relay4 → cover1`, `cover3` or no job). After it came down, a ball first down beyond the dirt stayed a deep ball while it was fielded beyond 80 m: a single the centre fielder charged and took at 85 m kept its relay man; one he took inside 80 m had a relay man until he crossed 80 m, then a cutoff man.
- **Case (d).** With nobody on, the cutoff to second was given only on a ball first down on the dirt. A line-drive or fly single had no cutoff man on any of the 310 shallow ones: the relay man went out toward third while the ball was up, and once it came down short he had nothing (`relay3 → held`, 45 stretches).

Seed 5 play 60 (frozen, fixture `5-60.json`): second inning, none out, a runner on third. A 105 mph liner over second first comes down at 45 m, on the dirt behind the bag, at 1.56 s, and skips to the centre fielder, who fields it at (14, 92) m. On `445ef2c`: at 0.70 s the second baseman has no job; from 1.00 s he is out as a deep ball's relay man toward home; at 1.57 s, once it is down, the first baseman goes out on the cutoff home and the second baseman goes to first.

## 2. The rule (`sim/crates/bb_game/src/live.rs`; agent-models §5.7)

1. **The judgment** (`air_read`, each infielder's own but the catcher's; kept once made, `Fielder::air`). The ball is still up, and no outfielder is known to him to have made a sure call on it. His read is recent (`HEAR_READ_AGE_S`) and still has the ball up. It gives him no play: no catch, and nothing he fields inside `INFIELD_BREAK_R`. By his estimate (`seen_intercept` on his read), no other infielder the ball has not yet gone by catches it or fields it inside that range. The first outfielder to reach it does so only after it has come down, beyond `RELAY_FROM_M`, and not as a deep ball (item 4). His read must have it coming down short of `RELAY_DEEP_M`; this is asked first, since a ball coming down beyond is never a single and is the commonest ball in the air. He works it out at each look at the ball (`LOOK_S`, 0.1 s), not every tick (`Fielder::air_no`). Every tick, it cost 12% of the run time on three slow seeds; at each look, 10% in the standard run; with the first question, the figure in §6 (measured on the final build).
2. **The cutoff man goes out at once** (`air_cutoff`) on his own judgment, on the ground single's infield gate (`infield_lets_go`). He is the middle infielder on the ball's side toward second with nobody on, the shortstop toward third with a runner on first, and toward home the corner man for the outfielder known to be going for it, or else the `through_point`'s (`single_of`). He goes to the cutoff spot on the line from the base to where he estimates it will be fielded. While the ball is up the job is his by his own judgment, and nobody else's choice takes it from him. Once it is down the cutoff rule takes over; on a single in front of the outfielder that is the same man.
3. **The others by the conventions**, as on a ground single: the infielders who judge it a single count the cutoff man home as out (`home_cutoff_by_convention`, `through_jobs`, `through_backup`: the second baseman to first, the shortstop to second, the pitcher behind home), and with a runner on first the second baseman takes second (`second_takes_over`). A judgment made while the ball was up stands after it comes down beyond the dirt (it is not asked again; an infielder known to be going for the ball suspends the conventions while that is known). The deep ball's early relay man does not go out on a ball he judges a single (but see item 7), nor once he knows his partner is out on a cutoff for it (item 8).
4. **The deep ball** (`deep_ball`). A ball first down beyond `RELAY_DEEP_M`, or caught, is judged as before. A ball first down in the outfield short of 80 m is deep once it is more than `PAST_OF_M` = 5 m [E, unsourced] deeper than the pre-pitch spot of the outfielder whose spot is nearest where it came down, loose or in a fielder's hands. While it is loose, it is also deep if a middle infielder was known to be out as its relay man when it came down and no corner man was known out on the cutoff home then (item 8). Grounders keep their rule (deep once past the outfielder fielding it).
5. **Case (d).** With nobody on, the cutoff to second is given on any ball that came down before anyone caught it (`relay_job`), not only on one first down on the dirt. After a catch nobody goes out, as before.
6. **The cutoff to second keeps its man.** Of the two middle infielders, the one first known to have gone out on it on this ball keeps it (`second_cutoff_man`, `known_first_out`). On a blooper up the middle whose side changed (seed 6 play 233), each in turn saw the other out 0.3 s late and handed the job back, every 0.3 s for 1.9 s.
7. **A cutoff man who stays** (`single_cutoff_stays`, `Fielder::stays`, `Fielder::cut`; Opus review P1-1, added after the review). Two infielders' reads of one ball can differ. A man who judged it a single counts the single's cutoff man out only until he knows that man is staying: once the cutoff man is known to him, on this ball, to be on another job than a cutoff or a chase, as known `CALL_HEARD_S` after the moment he could first have gone (`could`: the later of when the counting man began counting him the cutoff man, and the moment every infielder was known on this ball, the first showing of each plus `CALL_HEARD_S`; the job may have been shown at any time on the ball, a cover is not shown again). The gate applies toward second as well as toward third and home, the conventions end for the counting man on this ball and he works the tables from what he knows. The deep ball's early relay man who judged it a single then goes out too: otherwise nobody is out for it. Before, the second baseman would stand on first with the first baseman for 3.2 s (seed 80 play 169); now two men stand on one bag at most the 0.6 s it takes to know the cutoff man is not going.
8. **The relay man and the corner man's cutoff** (Opus review P2-1). The deep ball's early relay man stands down for his partner's cutoff (to second or third, which takes his place), and for a corner man's cutoff home only on a ball his own read has coming down short of `RELAY_DEEP_M`; on one his read has coming down beyond it, both are out, as on any ball going deep. After it comes down, the jobs known at the touch decide `deep_ball`'s known relay man: with only the relay man out the ball stays deep while loose; with the corner man out too the call was split and the fixed mark decides.

**Tried and dropped** (each measured on game seeds 0–119 with the same scripts):

- *A ball first down short of the outfielder's spot is in front of him until it gets past him and the man chasing it* (the grounder's rule, first version). A gap hit landing at 50–70 m and rolling to the wall stayed a "single" while the centre fielder ran alongside it, and the corner man went out until the fielder had it at 110 m.
- *The fielding point deeper than his spot*: the point moves with the chaser and flipped the judgment.
- *The fixed mark alone* (no known relay man, no `PAST_OF_M`). A gap hit judged deep in the air, with the relay man out, became a single at the bounce (the corner man out, the relay man back) and deep again once past the spot. Out-back-out with a runner in scoring position, shallow: 2 on main, 24. The known relay man brought it to 8.
- *Without `PAST_OF_M`*: a hard liner skipping to the centre fielder at his spot (seed 5 play 60) was a deep ball by every infielder's estimate, by a metre (88.5–90.5 m against his 88.4 m spot).
- *Judging on any read*: a high fly read at the first call, 5 s earlier, still had its landing in the past, so it was judged to drop. With a runner in scoring position, 38 of 364 caught flies had a corner man on the cutoff home while the ball was up and first open on 19. Requiring a recent read that still has the ball up brought these to 14 and 10 (main: 0 and 7).
- *Handing the cutoff to second to whichever man is known to be out on it now*: it did not stop the trading of item 6.

After the review (each on the same seeds; out-back-out with a runner in scoring position, shallow, against 9 on `e16a457`):

- *The cutoff man's staying counted from the counting man's own judgment alone* (as the review suggested): a cutoff man who was only waiting for the infield gate was counted as staying, and both he and the relay man went out (34).
- *The same, after the gate*: a man who judged it late, already out as the relay man, came back to the bag for 0.3 s (17).
- *No wait for the moment the man became the cutoff man*: when a runner from first rounding second made the throw home the likeliest, the shortstop, off his cutoff to third, saw the first baseman on first and took no job for 0.3 s (runner on first, shallow: middle infielders' stretches after it came down 10 → 37).
- *The corner man's cutoff not counted at all in the relay gate*: the relay man stayed out on balls that then fell in short, with first open while both were out (first uncovered while loose, runner in scoring position, shallow: 21.7 s; with item 8, 8.6 s).
- *`deep_ball`'s known relay man judged on what he is known to be doing now*: with a corner man also out, the ball was deep while the relay man was known out, so he stayed out, and not while he was known back, so he came back: the two traded the job every 0.3 s (seed 47 play 142, seed 111 play 271).

## 3. Measurements

Game seeds 0–119, main `445ef2c`, the reviewed build (`e16a457`, sim code `b90e422`) and the final build (after the review fixes, sim code `84e10f8`). The games reshuffle after the first changed play, so play counts differ a little.

Balls in the air at the first call, first secured by an outfielder after coming down (`air.py`):

| | Main | Reviewed | Final |
|---|---|---|---|
| **Runner in scoring position, dirt**: stretches begun while up (seconds); after it came down | **57 (20.3 s); 4** | 25 (10.1 s); 3 | **25 (10.4 s); 9** |
| Runner in scoring position, shallow: the same | 144 (57.9 s); 22 | 43 (16.5 s); 0 | 47 (15.9 s); 1 |
| Runner on first, dirt | 73 (25.0 s); 17 | 40 (12.8 s); 11 | 39 (12.6 s); 11 |
| Runner on first, shallow | 126 (48.9 s); 21 | 94 (31.0 s); 10 | 84 (28.1 s); 12 |
| Nobody on, dirt | 145 (46.3 s); 5 | 44 (14.3 s); 1 | 44 (14.3 s); 1 |
| **Nobody on, shallow** | **344 (160.1 s); 54** | 115 (48.0 s); 20 | **114 (44.7 s); 18** |
| Deep, all base states | 483; 16 | 469; 21 | 464; 20 |
| **Nobody on, shallow: no cutoff man for second (plays)** | **310 of 310** | 37 of 306 | **38 of 306** (fielded deep, a deep relay man instead) |
| Pitcher with no job, runner in scoring position (dirt; shallow) | 21.6 s; 43.4 s | 15.7 s; 16.7 s | 15.7 s; 16.7 s |
| **Two infielders on one bag, nobody on a cutoff or relay, more than 0.5 s; more than 0.65 s** (`dup.py`, the review's) | **0; 0** | 32; 30 (up to 3.2 s) | **23; 0** (each 0.6 s) |
| **More than 1 s with no cutoff or relay man from the outfielder's known chase plus 0.3 s, runners on** (`norelay.py`) | **4** | 15 | **0** |
| Out-back-out of one man on a relay, runner in scoring position (dirt; shallow; deep) | 10; 2; 1 | 5; 9; 7 | 6; 10; 2 |
| The same, runner on first | 11; 11; 4 | 9; 14; 8 | 10; 13; 8 |
| Relay or cutoff changing hands, runner in scoring position | 41; 38; 2 | 19; 51; 11 | 24; 52; 11 |
| Uncovered while loose, runner in scoring position, shallow: first; second; third (plays) | 19; 11; 17 | 12; 19; 10 | 27; 20; 14 |
| The same, deep: first (plays; seconds) | 0 | 5; 1.6 s | 7; 11.0 s |
| Runner on first, shallow: first uncovered (plays; seconds) | 6; 1.6 s | 24; 6.0 s | 24; 5.6 s |

What is left of the stretches with a runner in scoring position is mostly the first call's 0.3 s:

- a middle infielder who did not judge it a single and went out as the deep relay man once the outfielder's chase was known (`start → relay4`, 30 shallow);
- the second baseman or shortstop with no job until the first baseman or the man fielding it is known (`start → cover1`, `start → cover3`, 17 in the dirt): a blooper an infielder might still reach on a hop by his estimate (case (b)).

What the review fixes cost (final against reviewed):

- **The 0.6 s on one bag** (23 plays). The man who judged it a single goes to the cutoff man's bag at the first call by the convention; the cutoff man cannot go before the infield gate (0.3 s after the first call), and it takes 0.3 s more to see he did not. Main had none: nobody acted on the single while the ball was up.
- **A base open 0.3 s when the cutoff man goes late.** Once the convention has ended, the man who would have covered for the cutoff man no longer counts him out; when the cutoff man then judges it a single on a later look and goes, his base is open the 0.3 s it takes to see him go. Runner in scoring position, shallow: first uncovered 12 → 27 plays, 24 of them 0.3 s or less. Before the fix the second baseman stood on first beside him and was still there when he left.
- **The same, the other way** (runner in scoring position, dirt: stretches after it came down 3 → 9, third uncovered 2 → 6). A man who judged it a single saw the cutoff man still on his bag once the gate had passed, ended the convention and went out as the relay man; the cutoff man judged it a single 0.15–0.45 s later and went out too; at the bounce the relay man came back (seed 26 play 162). A misreading a real player could make: the cutoff man was slow to decide.
- **First open on a deep ball while both are out** (runner in scoring position, deep: 5 plays 1.6 s → 7 plays 11.0 s; seed 18 play 108, seed 47 play 138). The relay man and the first baseman on the cutoff home are both out. That is the standard alignment on 3 of the 7 plays (7.4 of the 11.0 s; seeds 18/108, 75/109, 47/138: an extra-base hit, the batter-runner is past first). On 13/224 and 93/113 the second baseman stays on first with second open about 1.2 s; on 107/40 the pitcher alternates between covering third and backing up home six times. Second uncovered on deep balls with a runner in scoring position went 3 plays/1.45 s (main), 7/2.5 s (reviewed), 6/3.7 s (final). All while the ball is up; no outcome changes in the trace.
- **Out-back-out** is back near the reviewed build's (runner in scoring position, deep, 7 → 2, as on main; shallow 9 → 10). The deep 7 were the relay man pulled in by the corner man's cutoff (P2-1). Of the shallow 10, 8 are gap hits fielded beyond 97 m: the relay man out while it was up, back when the corner man was seen out on the cutoff home (his own read had it coming down short of 80 m, item 8), out again once it rolled past the outfielder's spot plus `PAST_OF_M`. That is a ball that changed its nature, and a real relay man would go back out too. The other 2: the first baseman's cutoff given to the relay man at the bounce and back (seed 85 play 15), and a shortstop out as the relay man, back, then the cutoff to second (seed 115 play 85).

**The first ball on the dirt, grounders included** (the cutoff-home note's tables; `an.py`, `stretch.py`, `unc.py`), runner in scoring position, balls an outfielder fielded (main 125, final 128): middle infielders' stretches over 0.1 s **85 (28.8 s) → 58 (20.0 s)** (reviewed: 52, 17.9 s). The second baseman with no job went from 0.116 s to 0.073 s a play, the shortstop from 0.120 s to 0.084 s (reviewed: 0.068, 0.075). First uncovered 25 → 23 plays, second 23 → 18, third 14 → 18 (the longest 0.50, 0.43, 0.90 s; reviewed 17, 16, 14). Over every loose ball (`an_all.py`), runner in scoring position: early cutoffs home on a ball an infielder fielded 2 → 2; first uncovered while loose 46 → 40 plays, second 52 → 45, third 43 → 47. With nobody on, early cutoffs on balls an infielder fielded rose 66 → 72 (every loose ball), and first stood open on 5 → 7 balls an outfielder fielded; not traced one by one.

**Flies an outfielder caught** (`catches.py`), runner in scoring position (354 → 357 caught; reviewed and final alike but where said):

- the corner man was out on the cutoff home while the ball was up on 0 → 13: balls judged to drop that the outfielder caught. The single's jobs cover every base for a catch too, and the cutoff man home is where a throw home on the tag-up would pass.
- An occupied first stood open while the ball was up on 7 → 12 (reviewed 10).
- Second 50 → 48, third 4 → 5 (reviewed 3).
- Extra outs (doubled off, thrown out tagging) 2 → 2; runs scored on the catch 118 → 118.

**The three fixtures**, `445ef2c` → final (the reviewed and final builds play them alike):

- *Seed 5 play 60*, the liner into the dirt, a runner on third. Main: the second baseman had no job at the first call (0.70 s), went out as the relay man home at 1.00 s and went to first at 1.57 s, once the ball was down and the first baseman was out. Now, at the first call the second baseman covers first, the shortstop second and the pitcher backs up home (the first baseman is counted out by convention), and at 1.00 s the first baseman is on the cutoff home.
- *Seed 5 play 149*, a fly falling in front of the right fielder with runners on first and second. Main: the second baseman had no job 0.65–1.25 s, was out as the relay man home 1.25–3.25 s, had no job again until 3.89 s, and first stood open 3.59–3.89 s. Now the standard arrangement holds from the first call; the first baseman is on the cutoff home at 0.95 s.
- *Seed 0 play 17*, a fly single to right with nobody on. Main: the second baseman went out toward third as the relay man while it was up, and had no job from the bounce (2.80 s) to the catch (3.73 s). Now he goes out toward third at 1.03 s; by his estimate the right fielder could still catch it. At 1.48 s, when he judges it will drop, he takes the cutoff to second, on the line from the ball to second at the catch. The shortstop is on second throughout.

**Standard run**: §6.

## 4. Tests (`tests/throw_choices.rs`; fixtures frozen from `445ef2c`)

- `seed_5_play_60_…` and `seed_5_play_149_…` (`check_risp_air_single`). While the ball is up no middle infielder is on a relay, and the corner man is on the cutoff home before it comes down. At the catch, the standard arrangement (`check_risp_arrangement`, split out of `check_risp_single_jobs`). Once everyone can know no infielder is going for it, no middle infielder or corner man is without a job for more than 0.3 s. Both fail on `445ef2c`.
- `seed_0_play_17_a_fly_single_with_nobody_on_gets_a_cutoff_man_for_second`. From the bounce to the catch, the second baseman is on the cutoff to second and the shortstop on second; at the catch, the cutoff man is on the line from the ball to second. Fails on `445ef2c`.
- `a_ball_an_infielder_catches_a_deep_ball_and_a_caught_fly_keep_their_jobs` (regression guard; it passes on `445ef2c` too):
  - seed 2 play 117, a pop-up the third baseman catches 51 m out: no cutoff while it is up;
  - seed 11 play 203, a ball over the centre fielder: a middle infielder out as the relay man while it is up, no corner man on the cutoff;
  - seed 155 play 23 and seed 264 play 109, flies caught with a runner on second: second and third covered at every tick to the catch.
- `live::tests::a_ground_single_fielded_deep_is_not_a_deep_ball`: the line drive fielded at 70 m with nobody on now has the second baseman on the cutoff to second (the old assertion, no cutoff man, was case (d)). Two new cases: fielded at 88 m in front of the centre fielder's 97.5 m spot, a single; at 110 m, deep.
- `a_single_judged_in_the_air_gives_way_to_a_cutoff_man_who_stays` (Opus review P1-1; seed 61 play 187, seed 21 play 86, seed 77 play 114, frozen from `445ef2c`, the same balls on the reviewed build): no bag held by two infielders for more than 0.6 s (the cutoff man's infield gate and 0.3 s to see him stay), and from the outfielder's chase known plus 0.3 s to the pickup an infielder on a cutoff or relay at every tick. Each of the three fails alone on `e16a457` (two men on third 0.70–1.32 s and beyond; on first 1.40–2.02 s; on first 0.65–1.27 s).
- The isolation harness's private-state flips include `Fielder::air`, `Fielder::stays` and `Fielder::cut` and compare them.

`cargo test --workspace --release`, sources touched: §6.

## 5. Still open

- **The first call's 0.3 s on a ball the infielders do not take for a single**: a deep ball (its relay man goes out once the outfielder's chase is known), or one an outfielder may still catch by their estimate. This is most of what is left (`start → relay3`, 76 nobody-on shallow; `start → relay4`, 30 with a runner in scoring position). A real relay man reads a ball over the outfielder's head as early as a single; that belongs with the relay review.
- **A runner on first who passes second sends the first baseman to the cutoff home** (`relay_base` counts runners who were on base at the pitch at the base they are at now). On a single the first baseman then goes out on the cutoff home and first is open 0.3 s until the second baseman sees it. Before this change these balls were deep, with a middle infielder as the relay man. Runner on first, shallow: first uncovered 6 → 24 plays (5.6 s on the final build). It also happens on ground singles (on the dirt, 30 → 29). This is a bug in the cutoff convention, not an owner question (Opus review P2-2): in standard play the throw on that runner goes to third, with the shortstop as the cutoff man, and the first baseman stays at first behind the batter-runner. Left for the relay review (TASKS §3).
- **A gap hit that lands short and rolls to the wall** hands the job from the cutoff man to the relay man once it passes the outfielder's spot. With a runner in scoring position, shallow, it changed hands on 38 → 52 plays, with out-back-out 2 → 10 (8 of them such gap hits, §3). Second open 0.3 s while the shortstop goes out as the relay man: 11 → 20 plays. This is the cutoff-home note's "second for 0.3 s when a single gets past the centre fielder" (TASKS, for the relay review).
- **The 0.6 s on one bag and the 0.3 s gaps of item 7** (§3, what the review fixes cost): the price of anticipating the cutoff man by convention when two reads differ.
- **The cutoff man home changes at the outfielder's pickup** (Opus re-review P2-1): on a shallow single with a runner in scoring position the middle infielder kept out by `relay_out` while the ball is loose is replaced at the pickup by the corner man, who leaves his bag 0.8-2.5 s before the throw (seeds 23/94, 115/32; 6 plays in 120 games, main 1). Suggested: the known relay man keeps the job through the pickup. TASKS §3, for the relay review. The deep-ball second-base and pitcher items are in §3 above.
- **A ball judged to drop that the outfielder catches** (13 of 357 caught with a runner in scoring position): the corner man was out on the cutoff home; after a catch beyond 80 m a middle infielder takes over as the relay man (the after-catch rule, not changed).
- **On a blooper up the middle the side can still change once while it is up** (seed 6 play 233: the shortstop out at 1.45 s, off 1.52–1.73 s, then his for good).
- `PAST_OF_M` = 5 m is unsourced.

## Reproduce

- The fixtures: `tests/throw_choices.rs` (`replay_jobs`). They were frozen with the cutoff-home trace probe built on `445ef2c`, plus the fixture dump block (`sim/target/codex-probes/rfix/dump_block.rs`, inserted into `game.rs` by `liner/mkdump.py`) and `ZZ_DUMP=seed:play`.
- The counts: the cutoff-home probe `zz_cut_home.rs` (copy into `crates/bb_game/examples/`), built on each commit; `BB_COVER_TRACE=1 zz_cut_home 120 0 2> tr.log`. Then, in `sim/target/codex-probes/liner/`:
  - `air.py before.log after.log` (balls in the air, by base state and first-down zone);
  - `hands.py` (relay changes of hands, out-back-out);
  - `catches.py` (flies caught);
  - `unc_air.py`, `obo.py`, `nocut.py` (play lists).
  The cutoff-home scripts `an.py`, `an_all.py`, `unc.py` and `stretch.py` in `../cuthome/` cover the balls first down on the dirt.
- After the review: the traces `tr-before.log`, `tr-a12.log` (reviewed) and `tr-b8.log` (final) in `sim/target/codex-probes/liner/`; the review's scripts `dup.py`, `norelay.py`, `obo_list.py` in `sim/target/review-probes/py/`; `unc_b.py` (plays with a base uncovered, by base state and zone).

## 6. Final build and standard run

`sim-cli games --n 300`, seeds 0 [1000], main `445ef2c` → the reviewed build (`e16a457`, sim code `b90e422`) → final (after the review fixes, `84e10f8`). Main's figures are the cutoff-home final's (`1c06fdb`, the same sim code but two comment lines); main's runs, repeated on both seed sets before the review and again back to back with the final build, give the same figures.

| | Main | Reviewed | Final |
|---|---|---|---|
| Runs/team-game | 4.08 [4.46] | 4.04 [4.39] | **4.05 [4.39]** |
| GB / LD / FB / PU hit | .310/.671/.102/.005 [.315/.663/.099/.007] | .308/.666/.103/.005 [.313/.659/.097/.007] | .308/.666/.104/.005 [.313/.657/.097/.007] |
| HR/PA | 2.73% [2.98%] | 2.73% [2.95%] | 2.74% [2.95%] |
| K%, BB% | 22.6, 8.0 [22.1, 8.5] | 22.6, 8.0 [22.0, 8.5] | 22.6, 8.0 [22.0, 8.5] |
| Reached on BIP | 33.7% [34.0%] | 33.5% [33.7%] | 33.5% [33.7%] |
| PA/team-game, pitches/game | 37.7, 238 [38.1, 239] | 37.8, 239 [37.9, 238] | 37.8, 239 [37.9, 237] |
| Out of play after a throw, a game | 0.153 [0.120] | 0.123 [0.127] | 0.123 [0.127] |
| Rules warnings | {} | {} | {} |
| Time, ms per game per core (final back to back with main) | 4,476 [4,702] | | 4,773 [4,764] |

Every digest changed against main, and against the reviewed build (its figures differ: the jobs on balls in the air change where fielders stand, and the games after reshuffle).

- **Runs: −0.03 [−0.07].** The mean of 600 team-games moves by about 0.1 when the games reshuffle, so this is within the noise. In the trace's games (seeds 0–119, reviewed build) the plays it touches show no change in outcome: a runner from second scored on an air single 51/91 → 54/96, outs on the bases on air singles 36 → 41, outs at home 1 → 0, runs on caught flies with a runner in scoring position 118 → 118 (final: 118). Its direction fits the open lead in ledger §5 that the deep relay man adds runs (fewer liner singles now get one); not checked.
- **Time +4%** (the two seed sets together; +7% [+1%] on each, run back to back with main): the air judgment (§2 item 1). The reviewed build measured +5% [+3%] the same way; the review fixes add no search, only lookups of shown jobs.

`cargo test --workspace --release` on the final build, sources touched: 517 passed, 0 failed, 4 ignored (512 on main; five new tests). `python3 bin/check-docs.py`: clean.
