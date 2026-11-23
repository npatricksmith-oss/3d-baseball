# The double cut (2026-11-22)

Branch `work/double-cut`, from main `ba40063`. Owner decision Q35, option (c), as put in the [relay hand-overs note](2026-11-22-relay-handovers.md) §4: on a deep ball with a play at home a middle infielder goes out as the relay man and the first baseman lines up behind him as the cutoff man home, whichever field the ball is in; the third baseman stays at third, the other middle infielder covers second, first is left; the relay man chooses between throwing home and throwing through the cutoff man; a single that rolls past an outfielder adds a man instead of trading one. With it, the [Opus relay review](../reviews/2026-11-22-opus-relay-handovers-review.md) P3-3: the deep call on a ground ball flipped when the lead outfielder changed. The spec is agent-models §5.7 ("The double cut", and "A deep ball") and §5.5 (the route).

## 1. What was built

| Change | What was wrong (`ba40063`) | Now | Commit |
|---|---|---|---|
| The double cut (`double_cut_job`) | One man went out: the relay man on a deep ball, or the single's cutoff man home; the corner men stayed at their bases on a deep ball | On a ball that came down and got past the outfielders (a deep ball, not a catch) whose relay throw goes home, the first baseman lines up behind the relay man, 45 ft (`CUTOFF_M`) from the plate on the line from the relay man's spot; he stays there while the outfielder's throw goes to the relay man and while the relay man holds it | `e9dae69` |
| The hold home on a deep ball (`relay_base`) | The corner man's hold of the throw home (a runner from base still on the bases) applied on a single only | Also on a deep ball: the run from third scoring does not send the first baseman back while the runner from first is still coming | `e9dae69` |
| The convention (`deep_cut_by_convention`, `deep_jobs`) | Teammates learned the relay man's job `CALL_HEARD_S` (0.3 s) after he took it; his base waited for them | Everyone works the deep ball's cutoff men out from what all have seen, at once, as on a single through the infield; each man re-works his own cover when it changes. A corner man out on a single's cutoff who is not the double cut's man is back at his base | `e9dae69`, `f7d728e` |
| The route (`throw_race`) | The relay route went to the first-numbered man with a cutoff job | To the nearer of the men lined up between the holder and the base: the outfielder throws to the relay man, the relay man to the first baseman | `e9dae69` |
| The deep call on a ground ball (`latch_deep`) | Judged against the spot and position of the outfielder known to be going for it, it flipped back when another outfielder took the ball over (118/162) | Once judged deep, a ground ball stays deep through the outfielder's pickup to his throw | `e9dae69` |

No new parameter. The two fixes in `f7d728e` came from the first trace: on 39/56 the once-a-second re-call's table still put the shortstop, last known on second, on second although the convention counted him out as the relay man, and the second baseman had no job for 0.26 s; on 107/40 the pitcher, covering third while the third baseman was out on the single's cutoff, kept third in the third baseman's own table for three ticks.

## 2. The named plays (game trace, 120 games, seeds 0–119)

- **Seed 23 play 83** (runners on first and third, two out; a ball into left-centre falls in at 61 m and rolls past the left fielder). Before: at 4.19 s the first baseman came back to first, the shortstop went out as the relay man from second, and the second baseman, on first, went to second only at 4.49 s. Now at 4.19 s the shortstop goes out, the first baseman stays out and moves onto the relay line, and the second baseman leaves first for second at once.
- **Seed 97 play 119** (runners on first and second, two out; a ball to left first down 74 m, fielded 102 m out). Before: at 4.20 s the third baseman left the single's cutoff with no job and the shortstop, on third, went out; third had nobody until 4.50 s. Now at 4.20 s the third baseman takes third, the shortstop goes out, the first baseman goes out. He has 30 m to run from first to the line on the left of the mound and is still 16 m off it at the left fielder's throw (6.50 s): the cost of the first baseman on every ball, as ruled.
- **Seed 118 play 162** (runner on first, two out; a grounder past the right fielder, taken over by the centre fielder and fielded 114 m out). Before: the second baseman out at 4.11 s, back at 4.26 s, out again at 5.29 s. Now the second baseman and the first baseman go out at 4.11 s and stay out; the shortstop takes second at once.

## 3. Counts

The trace probe (`zz_dcut`, the relay hand-overs note's probe with the holders' throw choices added), 120 games, seeds 0–119, main `ba40063` against `f7d728e` (the games of `816a15e` are the same). Balls an outfielder first secured after they came down. "Unassigned" means no man had the base as his job at some tick before the outfielder's throw.

**During the double cut** (from the first tick the first baseman and a middle infielder are both out on the cutoff home with the ball down, to the throw): 130 plays with a runner on first at the pitch, 149 with a runner in scoring position. Second unassigned on none, two men on second on none, the third baseman off third on none.

| Measure (plays) | Main | Now |
|---|---|---|
| A man off a cutoff and another on at the same tick before the throw (a trade), runner in scoring position | 97 | 64 |
| the same, runner on first / nobody on | 47 / 66 | 47 / 66 |
| Second unassigned before the throw, runner on first / in scoring position | 181 / 104 | 157 / 88 |
| Third unassigned before the throw, runner on first / in scoring position | 15 / 62 | 15 / 51 |
| Out-back-out while loose, risp, ball in the air first down on the dirt; ground ball, runner on first / risp | 6; 2 / 2 | 2; 0 / 1 |
| A hand-over at the outfielder's pickup, risp, air on the dirt / ground ball | 8 / 4 | 5 / 3 |
| First open while loose, risp, air deep (plays, seconds) | 7 (12.3 s) | 80 (160.9 s) |
| First open while loose, runner on first, air deep | 0 | 58 (118.7 s) |
| Doubles with a runner on first; the runner from first scored | 115; 87 | 114; 87 |
| Throws home by an outfielder, direct / through a cutoff or relay man | 5 / 0 | 5 / 0 |
| Throws through a cutoff or relay man to any base | 0 | 0 |

The relay hand-overs note's table (`hc.py`), the rows that moved, main → now: second open while loose, runner on first, air in the dirt / shallow / deep 14 (4.0 s) / 19 (6.8 s) / 19 (8.7 s) → 4 (1.2 s) / 17 (5.1 s) / 19 (6.6 s), ground 12 (2.8 s) → 2 (0.4 s); runner in scoring position, air dirt / shallow 8 (1.9 s) / 21 (5.6 s) → 5 (0.9 s) / 13 (3.3 s); third open, risp, ground 8 (2.4 s) → 3 (1.5 s), air shallow 15 (4.8 s) → 12 (3.3 s). Its "hands" count (the set of men on cutoffs changed) rises on every deep zone (risp air deep 11 → 78): adding the first baseman changes the set, which that measure counts as a hand-over; the trade row above is the one that separates the two.

First is left open on the double cut by design. In the 120 games one throw went to first after a double cut began: seed 17 play 103 (a runner on first, a grounder past the centre fielder; he returned the ball to second, and the second baseman threw to first behind the batter-runner, who had rounded the bag), at 9.41 s against 8.60 s on main: he waited for the first baseman to come back from the cutoff. The batter-runner was tagged out at first on both builds.

No throw went through a cutoff or relay man to any base in 120 games on either build: relays are rare ([throw-race note](2026-11-19-throw-race.md) §8, about one in 100 games), so no throw in these games went to the relay man or through the first baseman. The relay man's choice with the first baseman behind him is checked in the constructed test only.

**The ping-pong count with nobody on, air deep, 5 → 7.** Both new plays (10/38, 71/185) are balls up the middle on which the relay man changes between the shortstop and the second baseman as the fielding point moves (whichever middle infielder's pre-pitch spot is nearer the relay spot): the same flips as on main. On main the partner stood with no job for 0.3 s each time; now he takes second at once, so the measure sees his jobs alternate. Not changed here (§5).

## 4. Tests

- `the_double_cut_lines_up_the_first_baseman_behind_the_relay_man` (unit, constructed; an alignment and candidate-route test: it checks the relay route each holder would take, not his choice of route or a throw, for which see §7): balls in an outfielder's hands 100 m out to centre, left-centre and down the right-field line with a runner on second: the relay man on the line from the ball to home, the first baseman between him and the plate about 45 ft from it, the third baseman on third, the other middle infielder on second, the catcher home, nobody on first; the outfielder's relay route goes to the relay man, and with the relay man holding the ball the first baseman stays and is his man in line. Not on a caught ball, nor with nobody on. Fails with the double cut taken out, and with the old route rule.
- `a_ball_past_the_outfielders_adds_the_relay_man_and_keeps_the_first_baseman` (fixtures 23/83, 97/119, 118/162 frozen from `ba40063`): from the deep call to the throw the relay man and the first baseman stay out, nobody else is on a cutoff, second and third have their men at every tick, the third baseman on third; a first baseman out before the call never leaves; his spot is between home and the relay man's spot, and he is nearer the line at the throw than at the call (on 23/83 on it). Each play fails on `ba40063`; without the latch 118/162 fails, without the convention 23/83 fails.
- `in_real_games_the_double_cut_keeps_third_second_and_home_covered` (seeds 0–7): at every tick of every double cut the third baseman covers third, the other middle infielder second, the catcher home. Fails without the convention (seed 2 play 77).
- `the_relay_man_keeps_the_job_through_the_outfielders_pickup` (relay hand-overs): now allows a first baseman already out on the double cut at the pickup; still nobody leaves a base at it.

`cargo test --workspace`: 526 passed, 0 failed (523 on main and the three new tests).

## 5. Standard run

`sim-cli games --n 300`, seeds 0 [1000], main `ba40063` against `f7baee3`, back to back on 2026-11-22, both built fresh (main from `git archive`; its output repeats the relay hand-overs ledger row's final figures line for line). The final build (`563bfa9`, sim as `816a15e`: the convention worked out once per tick), touched and rebuilt, gives the same figures on both seed sets, at 4,930 [5,058] ms per game per core; `cargo test --workspace` on it: 526 passed, 0 failed.

| | Main `ba40063` | `f7baee3` |
|---|---|---|
| Runs/team-game | 4.05 [4.33] | 4.03 [4.33] |
| GB / LD / FB / PU hit | .309/.666/.104/.006 [.312/.659/.096/.007] | .309/.665/.104/.006 [.312/.658/.096/.008] |
| HR/PA | 2.70% [2.94%] | 2.70% [2.95%] |
| K%, BB% | 22.7, 8.0 [22.1, 8.5] | 22.7, 8.0 [22.1, 8.5] |
| Reached on BIP | 33.6% [33.6%] | 33.6% [33.6%] |
| PA/team-game, pitches/game | 37.9, 239 [37.8, 237] | 37.9, 239 [37.8, 237] |
| Out of play after a throw, a game | 0.110 [0.113] | 0.103 [0.110] |
| Rules warnings | {} | {} |
| ms per game per core | 4,809 [5,061] | 5,034 [5,122] |

Within noise: the jobs change where the infielders stand, and a throw's timing on a play where a holder waits for his man at a bag (23/83: the left fielder threw to second at 6.70 s against 7.32 s on main, the second baseman reaching the bag sooner). The out-of-play-after-a-throw fall is 2 [1] throws in 300 games, not traced. The time is within one binary's back-to-back spread: main alone gave 4,809 and 5,105 ms on seed 0 a few minutes apart, `816a15e` 5,233.

## 6. Still open

- **Decided: Q36 (a)** (owner, 2026-11-22): the first baseman goes out at the bounce, as built; recorded in agent-models §5.7. The question as it was put: *when the first baseman goes out.* Now: once the ball has come down beyond the outfielders (a hit everyone has seen). The relay man goes out earlier, while a ball going deep is still in the air, on his own read. On a ball clearly over an outfielder's head a real first baseman also starts for the line before it lands; on one that may yet be caught he must stay, since a runner on first who went too far is doubled off there. *Options:* (a) as built, at the bounce; (b) also while the ball is up once the relay man is known out and no outfielder has a sure call. *Measured* (the 279 double cuts of §3): the first baseman goes out a median 0.12 s after the relay man (quartiles 0.00–2.43 s; the long ones are balls the relay man went out on while they were up) and 2.9 s before the outfielder's throw (1.9–3.9 s). *Recommendation:* (a): he is mostly in line before the throw; (b) needs the runner-on-first tag-up case worked out.
- On a ball to left the first baseman has about 30 m to run to the line and is often not there by the outfielder's throw (97/119). As ruled; worth watching in the owner's saved plays.
- The relay man flips between the middle infielders on balls up the middle as the fielding point moves (10/38, 71/185; pre-existing). The man first known out as the deep ball's relay man could keep it, as `second_cutoff_man` does for the single's cutoff to second; not the middle infielder out on a single's cutoff to third, who is not the relay man on a ball past the right fielder (118/162).
- ~~After a throw goes by the first baseman he stops where he is~~: fixed in §7 (he takes first at once). A trail man and the catcher's "Cut!" call are not built (§5.5).
- The relay man's throw through the first baseman is still not seen in games (no relay throws home in 120 games); §7 executes it in a constructed test and says why it is rare.

## 7. Review fixes (2026-11-22)

Two reviews: [Opus](../reviews/2026-11-22-opus-double-cut-review.md) (mergeable; P2-1, P3-1 to P3-4) and [Codex](../reviews/2026-11-22-codex-double-cut-review.md) (not mergeable: P1-1; P2-1, P2-2, P3-1, P3-2). Fixed here: Codex P1-1, P2-1 (both), Opus P3-1, Opus P3-2 with Codex P2-2, Opus P3-4, Codex P3-2. Deferred with a line in TASKS: Opus P3-3, Codex P3-1, the batter-runner who stops at first on a ball 120 m out, `relay_base`'s thresholds.

| Finding | What was wrong | Now | Commit |
|---|---|---|---|
| Codex P1-1 (observation boundary) | Through a throw in the air, the first baseman kept his job from the throw's stored route (`relay: true`, `to_base: 4`, the receiver): the thrower's intention, which he cannot see. Same physical throw, only the flag changed: `Relay(4)` in one copy, idle in the other | He stays out only while the ball, as he sees it fly, is coming to the man he knows to be the relay man: that man is the fielder (other than the thrower) nearest where it comes down through catching height (`throw_comes_to`). No read of `relay`, `to_base` or the receiver. (A first try used the runners' `LOW_THROW_MAN_M` = 4 m "thrown to a man": a relay throw executed with speed noise came down 4.5 m short of the relay man and the first baseman left.) The receiver of a throw is still treated as known to the receiver himself (`update_relay`, `try_captures`), as before: not this branch's | `193a941` |
| P2-1 (both) | A throw that went by a cutoff man stopped him where he was with no job until the next possession (`go_idle` in `holder_act` and `update_relay`; covers are not re-worked while a throw is in the air). On the double cut nobody else has first: 241 spells in 120 games, 2.6 s on average | He sees the release and takes his own next job at once (`take_own_job`): the double cut's first baseman goes to first | `193a941` |
| Opus P3-1 (the route) | `throw_race` took any man known on any cutoff job (any base, off the line) as the relay route; the cut height took only a man lined up for that base | One predicate for both (`men_in_throw`: the job for that base, between, within `CUT_LINE_M` of the line); with two, the nearer. **Not quite as the fix job put it** ("only men with `Role::Relay(base)`"): with nobody lined up for the base, the route may still go through a man on its line, by the same test, with a cutoff job for another base, by time alone as before. Built strictly, three relay tests failed (`a_throw_beyond_the_arm_goes_through_the_relay_man`, `the_relay_man_holds_the_ball_for_the_sourced_exchange`, `the_holder_predicts_the_relay_the_world_makes`): a centre fielder 112 m out with no play at home gets the ball back to second "the quicker way" (§5.7) through the relay man lined up toward home, who stands 1.3 m off the line to second, and strictly he had no route there at all and threw a long bouncing ball to second. No throw went through any cutoff man in 120 games on either build, so games do not decide it; this is the conservative reading, and the Coordinator may rule otherwise | `193a941`, `b351a2a` |
| Opus P3-2, Codex P2-2 (executed) | The constructed test checked the alignment and the candidate route, not the choice or a throw | `the_double_cut_played_through` (below) | `193a941` |
| Opus P3-4, Codex P3-2 (text) | §5.7's "and back", the hold's "first baseman known out", "cover or backup job"; the module's "at most one cutoff or relay man" | Reworded; Q36 recorded as decided | `a85c36a` |

**Tests** (each new one fails without its fix: a run with each fix reverted in turn, `dcut/fix/mut.sh`, job 69):
- `the_first_baseman_reads_the_throw_not_its_stored_route` (unit, the isolation case Codex asked for): a physically solved throw from the centre fielder to the relay man; the same throw with only its stored route changed (to home, receiver the catcher, not a relay; to second, receiver the second baseman; to second as a relay): the first baseman's job and spot are the same, out behind the relay man. A throw that physically goes to second but is stored as a relay to the relay man: he goes to first at once. Fails with the stored-route read (the second variant: `Cover(1)`), and without the own-job fix (`Idle`).
- `a_throw_that_goes_by_the_double_cut_sends_the_first_baseman_back_to_first` (fixture 17/103, frozen from `ba40063` like the others): from the centre fielder's release to second until the ball is taken there, the first baseman covers first at every tick and is nearer first at the catch. Fails without the own-job fix (at 6.05 s, `Idle`).
- `the_route_goes_only_through_a_man_lined_up_for_that_base` (unit): two men nearer the centre fielder than the relay man, one known on the cutoff home but 40 m off the line, one on the line but lined up for third: the route and the cut height both take the relay man. With nobody lined up for home, the man on the line for third is the route (the throw not kept low for him), never the man off it. Fails with the old route rule (the third baseman taken first).
- `the_double_cut_played_through` (unit, every throw thrown and caught in the world; expectations from `solve_throw`'s flight times and `RELAY_S_50`): (1) a relay man with a 36 m/s arm on his spot 56 m out, whose straight throw home on the fly beats the relay through the first baseman by more than 0.2 s; the centre fielder 112 m out with a 30 m/s arm, beyond the plate on the fly and within reach of the relay man, throws to him (the release is made for him: from 112 m his own route reckoning has no play on a runner just off third); the first baseman, still running in from first at the release, keeps his job through the flight; the relay man throws home himself; the catcher has it. (2) A relay man who has taken the throw 45 m out with an arm that cannot reach the plate on the fly but reaches the first baseman with 1.5 m/s to spare: he throws through the first baseman, who takes it and throws home. It passes on the head as committed; it is a check of the execution, so it does not single out one of the fixes.

**Why the relay through the first baseman is rare** (from the same flight times): from the relay man's spot 56 m out, for every arm that reaches the plate on the fly the straight throw beats the relay through the first baseman, whose exchange (0.75 s at grade 50) is more than the flight it saves. Only an arm that cannot reach the plate on the fly (22–25 m/s from 56 m) makes the relay quicker, and then the throw to the first baseman is a high one of 2.6 s or more. From his spot no such relay man had a play on a runner just off third, hence the 45 m case. The outfielder's route reckons one relay man, so to him a relay man who cannot reach the plate on the fly is no route home at all. Neither is a fault of the double cut; both are in agent-models §5.7 ("In practice").

**Re-measured** (120 traced games, seeds 0–119; `dcut/idle.py`, which reproduces the Opus review's table on its traces; probes built from `193a941` and `b351a2a`, traces `tr2-v3.log` and `tr2-v4.log`, the same job for job):

| | Main `ba40063` | `816a15e` (the reviews) | `b351a2a` |
|---|---|---|---|
| First baseman idle with first empty, after being out on the cutoff home | 0 | 241 (616.6 s) | 0 |
| of them with the batter-runner at first | 0 | 91 (209.8 s) | 0 |
| longest | — | 3.1 s (46/114) | — |

The §3 table is the same on `193a941` as on `816a15e`, row for row (double cuts 130/149, trades 47/66/64, second unassigned 157/88, third 15/51, the runner from first scoring on 87 of 114 doubles): the fixes act only once a throw is in the air. **No play's outcome changed** in the 120 games (outs, the batter's base, every runner's base, the first catch time: compared per seed up to the first difference, and there was none). The jobs changed on 261 plays:
- 248 double cuts: the first baseman to first at the release instead of standing.
- 13 single-cutoff plays (any cutoff man a throw goes by now takes his own job). On 4 he covers a base that was open through the throw: first on 38/57 and 83/235, third on 47/78 and 53/141. On 9 he heads for a base a teammate took less than 0.3 s before the release, which he does not yet know (2/73, 17/188, 23/32, 64/240, 74/166, 76/230, 80/168, 81/18, 109/157). Covers are not re-worked while a throw is in the air (agent-models §5.7), so both men stay on that base until the catch; before, he stood with no job. It is the hearing rule's known 0.3 s, lengthened by the flight: left as is, with a line in TASKS.

**Standard run** (`dcut/fix/final.sh`, job 74, on `b351a2a` after touching the changed sources; `cargo test --workspace` in the same job: 530 passed, 0 failed, the 526 before and the four new tests). Seeds 0 [1000]:

| | Main `ba40063` | Before the reviews (`563bfa9`) | Review fixes (`b351a2a`) |
|---|---|---|---|
| Runs/team-game | 4.05 [4.33] | 4.03 [4.33] | 4.04 [4.34] |
| GB/LD/FB/PU hit | .309/.666/.104/.006 [.312/.659/.096/.007] | .309/.665/.104/.006 [.312/.658/.096/.008] | .308/.665/.104/.006 [.312/.658/.096/.007] |
| HR/PA | 2.70% [2.94%] | 2.70% [2.95%] | 2.72% [2.94%] |
| K%, BB% | 22.7, 8.0 [22.1, 8.5] | the same | the same |
| Reached on BIP | 33.6% [33.6%] | 33.6% [33.6%] | 33.6% [33.6%] |
| PA/team-game, pitches/game | 37.9, 239 [37.8, 237] | the same | the same |
| Out of play after a throw, a game | 0.110 [0.113] | 0.103 [0.110] | 0.107 [0.107] |
| ms per game per core | 4,809 [5,061] | 4,930 [5,058] | 4,964 [4,937] |

Within noise of both: runs ±0.01, a hit rate by .001, HR/PA by a home run or so in 300 games. Rules warnings {}. Seeds 0–119 had no play's outcome changed; traced on to seed 299 (`tr3-v2.log`, `tr3-v4.log`, before against after the review fixes), each set of games first differs at one play, both the own-job fix doing what it is for: 192/17, the right fielder's return to second goes by the shortstop on the cutoff, who now heads for second and takes it where before it got by, and the batter stops at third instead of coming round to score; 231/111, a double cut, the first baseman back on first at the centre fielder's release, and the shortstop throws behind the batter-runner who had rounded first and gets him. Later plays in those games follow from a different game.

## Reproduce

- Probe: `zz_dcut.rs` (in `sim/crates/bb_game/examples/`, not committed: the relay hand-overs probe with a `C` line per throw choice); `BB_COVER_TRACE=1 zz_dcut 120 0 2> tr.log`. Main's probe was built from `git archive ba40063` in `sim/target/codex-probes/dcut/fxsrc` (with the fixture dump block in `game.rs`, which prints only under `ZZ_DUMP=seed:play`). Traces: `sim/target/codex-probes/dcut/tr2-base.log`, `tr2-v2.log`.
- Tables: `dcut/dc.py a.log b.log` (§3; `-v` lists plays), `handover/hc.py a.log b.log`, `handover/show.py tr.log seed play`.
- Standard run: `dcut/std.sh`; outputs `dcut/std-{main,head}-{0,1000}.txt`, timing pair `dcut/t-*-0.txt`. Review fixes (§7): `dcut/fix/final.sh` (tests and the standard run, `std-fix-{0,1000}.txt`), `dcut/fix/mut.sh` (each fix reverted in turn), `dcut/idle.py base.log v2.log v3.log` (the P2-1 table and where the builds first differ), fixture 17/103 dumped with `ZZ_DUMP=17:103 dprobe-base 1 17`.
