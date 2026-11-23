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

- `the_double_cut_lines_up_the_first_baseman_behind_the_relay_man` (unit, constructed): balls in an outfielder's hands 100 m out to centre, left-centre and down the right-field line with a runner on second: the relay man on the line from the ball to home, the first baseman between him and the plate about 45 ft from it, the third baseman on third, the other middle infielder on second, the catcher home, nobody on first; the outfielder's relay route goes to the relay man, and with the relay man holding the ball the first baseman stays and is his man in line. Not on a caught ball, nor with nobody on. Fails with the double cut taken out, and with the old route rule.
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

- **For the owner, not decided here (the Coordinator numbers it).** *When the first baseman goes out.* Now: once the ball has come down beyond the outfielders (a hit everyone has seen). The relay man goes out earlier, while a ball going deep is still in the air, on his own read. On a ball clearly over an outfielder's head a real first baseman also starts for the line before it lands; on one that may yet be caught he must stay, since a runner on first who went too far is doubled off there. *Options:* (a) as built, at the bounce; (b) also while the ball is up once the relay man is known out and no outfielder has a sure call. *Measured* (the 279 double cuts of §3): the first baseman goes out a median 0.12 s after the relay man (quartiles 0.00–2.43 s; the long ones are balls the relay man went out on while they were up) and 2.9 s before the outfielder's throw (1.9–3.9 s). *Recommendation:* (a): he is mostly in line before the throw; (b) needs the runner-on-first tag-up case worked out.
- On a ball to left the first baseman has about 30 m to run to the line and is often not there by the outfielder's throw (97/119). As ruled; worth watching in the owner's saved plays.
- The relay man flips between the middle infielders on balls up the middle as the fielding point moves (10/38, 71/185; pre-existing). The man first known out as the deep ball's relay man could keep it, as `second_cutoff_man` does for the single's cutoff to second; not the middle infielder out on a single's cutoff to third, who is not the relay man on a ball past the right fielder (118/162).
- After a throw goes by the first baseman he stops where he is and takes first back only by the tables' "and back" rule while the ball is in the outfield. A trail man and the catcher's "Cut!" call are not built (§5.5).
- The relay man's throw through the first baseman is untested in games (no relay throws home in 120 games).

## Reproduce

- Probe: `zz_dcut.rs` (in `sim/crates/bb_game/examples/`, not committed: the relay hand-overs probe with a `C` line per throw choice); `BB_COVER_TRACE=1 zz_dcut 120 0 2> tr.log`. Main's probe was built from `git archive ba40063` in `sim/target/codex-probes/dcut/fxsrc` (with the fixture dump block in `game.rs`, which prints only under `ZZ_DUMP=seed:play`). Traces: `sim/target/codex-probes/dcut/tr2-base.log`, `tr2-v2.log`.
- Tables: `dcut/dc.py a.log b.log` (§3; `-v` lists plays), `handover/hc.py a.log b.log`, `handover/show.py tr.log seed play`.
- Standard run: `dcut/std.sh`; outputs `dcut/std-{main,head}-{0,1000}.txt`, timing pair `dcut/t-*-0.txt`.
