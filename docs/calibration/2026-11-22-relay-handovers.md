# Cutoff and relay hand-overs (2026-11-21 to 2026-11-22)

Branch `work/relay-handovers`, from main `7622093` (the liner-single merge). TASKS §3 "For the relay review": what three reviews left about who is the cutoff or relay man and who covers which base while an outfield hit is loose ([liner-single note](2026-11-21-liner-single-jobs.md) §5; [liner-single re-review](../reviews/2026-11-21-opus-liner-single-jobs-rereview.md) P2-1, P3-1; [liner-single review](../reviews/2026-11-21-opus-liner-single-jobs-review.md) P2-2; [cutoff-home review](../reviews/2026-11-21-opus-cutoff-home-jobs-review.md) P3-2, P3-3). The spec is agent-models §5.7. One worker did items 1 and 2 and was stopped at the owner's request; a second finished the job.

The standard: what a real infield does. Fielders act only on what they know (a teammate's job `CALL_HEARD_S` = 0.3 s after he takes it, what they see).

## 1. What changed, item by item

| Item | What was wrong (main `7622093`) | Now | Commit |
|---|---|---|---|
| 1 | A runner from first who passed second counted as a runner on second: the throw home was the likeliest, and the first baseman went out on the cutoff home with first open (liner review P2-2, a convention bug) | On a single (not a deep ball) each runner counts at the base he was on at the pitch, unless he stands on third, which threatens home (Gemini A4 P2-3). The throw on the runner from first goes to third with the shortstop cutting; the first baseman stays at first | `7413eb8`, `40102f7`, `ea66e2b` |
| 1 (cost) | Counted at the pitch's base, the runner from first behind a runner from third made the throw third the moment the run scored: the job went from the corner man to the shortstop 3.4–3.9 s after contact | A corner man known out on the cutoff home keeps the throw home on a single while a runner who was on base at the pitch is still running | `ea66e2b` |
| 2 | The relay man held the job by `relay_out` only while the ball was loose: at the outfielder's pickup the corner man replaced him and left his bag 0.8–2.5 s before the throw (re-review P2-1) | The known relay man keeps it through the pickup to the throw; a ball in an outfielder's hands is judged where he secured it, not where he steps in to throw | `40102f7` |
| 3 | Deep ball, first baseman on the cutoff home and shortstop out: the second baseman, put on first while the shortstop was free, stayed there with second open about 1.2 s (re-review P3-1) | He leaves first for second as soon as he knows both are out, as the table already says (`home_order`). While the shortstop goes for a ball still in the infield, he stays | `e99e28d` |
| 4 | Cutoff man and relay man swap when a ball that fell in front of an outfielder rolls past him; second open ≤ 0.3 s while the partner learns the relay man has gone | Not changed: the part that is a design choice is the question in §4 | — |
| 5 | The once-a-second re-call's cover table had only the third baseman and the shortstop for third; the one used between calls also had the pitcher. With both out, the pitcher went to third between calls and back to backing up home at each re-call | Both tables: third baseman, shortstop, pitcher | `6efe905` |
| 6 | `relay_job` sent a corner man out on the cutoff home from a loose ground ball without the infield gate once an outfielder was the known lead (cutoff-home P3-3) | The same gate as at the first call (`infield_lets_go` toward home) | `ba27f63` |

## 2. Counts

120 games, seeds 0–119, the trace probe (`BB_COVER_TRACE=1 zz_cut_home 120 0`). Balls an outfielder first secured after they came down, the first loose ball only; *air* means still up at the first call, *gb* down before it; base state at the pitch; zone by where it first came down (*dirt* inside `INFIELD_R`, *shallow* inside 80 m, *deep* beyond). Plays (seconds in total). "Hand-overs": the cutoff or relay man changed while loose; "pickup": changed within 0.5 s of the outfielder's pickup; "out-back-out": a man off a cutoff and back on it; "two on a bag": two infielders on one base with nobody on a cutoff or relay for more than 0.5 s; "ping-pong": one infielder's job A, B, A, B. Main is `7622093`; the middle column is `40102f7` (items 1 and 2, as the first worker left them, measured on 2026-11-22); final is `ea66e2b`.

**Runner on first only**

| Zone | Measure | Main | Items 1–2 | Final |
|---|---|---|---|---|
| air, dirt | first open | 29 (8.9 s) | 0 | 0 |
| air, shallow | first open | 24 (5.6 s) | 0 | 0 |
| gb, dirt | first open | 24 (6.5 s) | 0 | 0 |
| air dirt / shallow / gb | first baseman on the cutoff home | 29 / 24 / 24 | 0 / 0 / 0 | 0 / 0 / 0 |
| air dirt / shallow / gb | hand-overs | 45 / 50 / 29 | 13 / 14 / 9 | 13 / 13 / 8 |
| air dirt / shallow / gb | at the pickup | 25 / 31 / 13 | 2 / 1 / 1 | 2 / 1 / 1 |
| air dirt / shallow / gb | out-back-out | 10 / 13 / 5 | 3 / 6 / 2 | 3 / 6 / 2 |
| air, dirt | second open | 13 (3.0 s) | 14 (4.0 s) | 14 (4.0 s) |
| air, shallow | second open | 22 (6.9 s) | 20 (7.1 s) | 19 (6.8 s) |
| gb, dirt | second open | 6 (1.1 s) | 13 (2.9 s) | 12 (2.8 s) |
| air dirt / shallow / gb | third open | 5 / 5 / 3 | 1 / 0 / 1 | 1 / 0 / 1 |

**Runner in scoring position**

| Zone | Measure | Main | Items 1–2 | Final |
|---|---|---|---|---|
| air dirt / shallow / deep | hand-overs | 24 / 52 / 11 | 30 / 58 / 11 | 25 / 54 / 11 |
| gb dirt | hand-overs | 14 | 16 | 14 |
| air dirt / shallow, gb dirt | at the pickup | 9 / 7, 3 | 13 / 5, 8 | 8 / 1, 4 |
| air dirt / shallow / deep, gb | out-back-out | 6 / 10 / 2, 2 | 7 / 10 / 2, 4 | 6 / 10 / 2, 2 |
| air dirt / shallow / deep, gb | two on a bag | 2 / 10 / 5, 3 | 3 / 10 / 5, 4 | 4 / 10 / 5, 5 |
| air dirt / shallow / deep, gb | ping-pong | 6 / 5 / 2, 7 | 6 / 5 / 2, 8 | 6 / 4 / 1, 8 |
| air, dirt | first / second / third open | 12 (3.6 s) / 8 (1.9 s) / 6 (1.4 s) | 13 (3.9 s) / 10 (2.3 s) / 9 (2.4 s) | 12 (3.7 s) / 8 (1.9 s) / 6 (1.4 s) |
| air, shallow | first / second / third open | 27 (8.6 s) / 20 (5.5 s) / 14 (4.8 s) | 27 (8.6 s) / 24 (6.9 s) / 17 (5.3 s) | 27 (8.9 s) / 21 (5.6 s) / 15 (4.8 s) |
| air, deep | first / second / third open | 7 (11.0 s) / 6 (3.7 s) / 2 (0.8 s) | 7 (10.7 s) / 6 (3.9 s) / 2 (0.8 s) | 7 (12.3 s) / 6 (2.1 s) / 2 (0.4 s) |
| gb, dirt | first / second / third open | 6 (1.0 s) / 5 (1.1 s) / 9 (2.7 s) | 5 (1.0 s) / 7 (1.4 s) / 10 (2.7 s) | 5 (1.0 s) / 4 (0.9 s) / 8 (2.4 s) |

Nobody on: every measure the same as main but two on shallow balls in the air: pickup hand-overs 2 → 0 (item 2) and third open 1 play (0.1 s) → 0.

The review's own probes on the final trace: `pickup.py` (a middle infielder on the relay home at the pickup and off it after), 6 → 0; `pingpong.py`, the pitcher between third and backing up home with a runner in scoring position, 3 → 1 (93/183 left). This job's `pp.py`, which also counts a flip at a re-call, gives 3 → 2 (93/183 and the new 9/9; item 5 below).

**By item**, on the trace before and after each step:

- *Item 1* (main → `7413eb8`): the runner-on-first rows above. The middle column's runner-in-scoring-position costs (hand-overs 24 → 30 on the dirt, pickups 3 → 8 on the ground, third open 6 → 9) were nearly all runners on first and third: the first-and-third hand-over at the run (11 plays: seeds 2/112, 13/200, 13/227, 58/48, 60/157, 64/241, 67/66, 77/180, 105/136, 111/152, 115/44). `ea66e2b` takes them back to main's figures (table) without changing the runner-on-first rows.
- *Item 2*: the first worker's figures (STOPPED report): `pickup.py` 6 → 0 and its three named plays (23/94, 115/32, 63/211) fixed; pickups on shallow air balls with a runner in scoring position 7 → 5, now 1.
- *Item 3* (`40102f7` → `e99e28d`): second open on deep air balls with a runner in scoring position 6 plays 3.9 s → 6 plays 2.2 s; first open there 10.7 → 12.1 s (by the table nobody holds first then; the batter-runner is going for second). Both named plays fixed.
- *Item 5* (`e99e28d` → `6efe905`): the pitcher's third/backup ping-pong (`pp.py`) 3 → 2 plays (107/40 and 109/209 fixed; 93/183 is left: he follows the third baseman's own flip between the cutoff home and third; 9/9 is new, 0.2 s at a re-call where a teammate's cover was not yet known); third open with a runner in scoring position, deep 0.8 → 0.4 s, shallow 5.3 → 5.0 s; ping-pong in all: air deep 2 → 1, shallow 5 → 4, ground with a runner on first 7 → 6, ground with a runner in scoring position 8 → 9 (9/9).
- *Item 6* (`6efe905` → `ba27f63`): a corner man out on the cutoff home before an infielder's pickup 2 → 1 play (12/9 fixed; 117/38 left: the first baseman went out before the shortstop was going for it, at 0.94 s, and the gate let him). Bases no longer open on 12/9 (first, 0.4 s, the batter thrown out at first), 69/259 (second 0.15 s, third 0.3 s), 95/176 (first 0.15 s) and 69/282 (second 0.3 s). The cost: two men on one bag for more than 0.5 s on 3 more plays (39/145 0.82 s, 95/176 0.55 s, 111/74 0.52 s): his teammates count him out by the convention (`home_cutoff_by_convention`, on their own read) while the gate still holds him. An open base costs more than a doubled one, so this stays.

**What item 1 costs and leaves.** With a runner on first, the shortstop now goes out on the cutoff to third and the second baseman covers second. When the ball then gets past the outfielder to the right side, the second baseman goes out as the deep ball's relay man and the shortstop comes back to second: second open for the 0.3 s it takes the shortstop to know it (a middle infielder leaving second for a cutoff or relay with the ball down and second then empty, runner on first: second baseman 13 → 30 plays, 3.1 → 7.6 s; shortstop 14 → 6). This is item 4's hand-over (§4).

## 3. Tests (`tests/throw_choices.rs`; fixtures frozen from `7622093`)

Each frozen play is listed in the fixtures' README. Each test checks who holds which bag when, or who is on a cutoff, at every tick, and fails without its change:

- `a_runner_from_first_passing_second_keeps_the_first_baseman_at_first` (6/6, 3/63): nobody on the cutoff home before the throw, the first baseman on first throughout, the shortstop on the cutoff to third at the pickup.
- `a_corner_man_out_on_the_cutoff_home_keeps_it_when_the_lead_runner_scores` (13/227): the first baseman stays on the cutoff home from going out to the throw, the run from third scoring before it.
- `the_relay_man_keeps_the_job_through_the_outfielders_pickup` (23/94, 115/32, 63/211).
- `a_deep_ball_with_the_first_baseman_and_shortstop_out_has_second_covered` (13/224, 93/113): second covered at every tick once both have been out longer than 0.3 s and a tick.
- `the_pitcher_keeps_third_while_the_third_baseman_and_shortstop_are_out` (107/40, 109/209): third covered while both are out (107/40); the pitcher's jobs never go A, B, A, B before the throw.
- `seed_12_play_9_the_first_baseman_holds_first_while_the_shortstop_goes_for_the_chopper` (12/9): first covered at every tick to the pickup, the first baseman never on the cutoff while the shortstop goes for the ball.

The unit test `the_cutoff_lines_up_for_home_with_a_runner_standing_on_third` (Gemini A4 P2-3) still passes: a runner standing on third counts there.

## 4. Item 4, a question for the owner

**The double cut on a ball past the outfielders.** On a ball in the gap or down a line that gets past the outfielders with a runner who may score, the textbook alignment is a double cut: a middle infielder goes out as the relay man, a corner man (in most systems the first baseman, so that the third baseman stays at third) lines up as the cutoff man for the throw home between the relay man and the plate, the other middle infielder trails or covers second, and the pitcher backs up home or third.

*What the game does now.* One man only: the relay man (the middle infielder whose pre-pitch spot is nearer the relay spot) or, on a single, the cutoff man home (the first baseman on a ball the centre or right fielder fields, the third baseman on one the left fielder fields). The corner men stay at their bases on a deep ball. When a ball that fell in front of an outfielder rolls past him, the single's cutoff man comes back and the relay man goes out (in 120 games with a runner in scoring position: hand-overs 25 / 54 on balls in the air first down on the dirt / in the shallow outfield; a man out, back and out again on 6 / 10). On a fly over the outfielders, a corner man out on the single's cutoff and the relay man out on his own read both stay out while it is up, and when it comes down one goes back. The partner covering second learns the relay man has gone 0.3 s late: second is empty for up to 0.3 s on about 30 such plays with a runner on first (§2), with the ball 80–120 m out, so no throw can arrive in that time (cutoff-home review P3-2).

*Two plays from the final build's trace* (`show.py tr-i1h.log <seed> <play>`):

- *Seed 23, play 83*: runners on first and third, two out; a ball into left-centre comes down 61 m out at 2.3 s and rolls on; the left fielder picks it up 97 m out at 4.5 s. At 0.95 s the first baseman goes out as the cutoff man home (the centre fielder is the lead) and the shortstop starts out as the relay man; at 1.25 s the shortstop, seeing the first baseman out and the ball coming down short, goes back to cover second. At 4.19 s, the ball past the outfielders, the first baseman comes back to first and the shortstop goes out again as the relay man. A real infield would leave the first baseman out, lined up behind the shortstop.
- *Seed 97, play 119*: runners on first and second, two out; a ball to left comes down 74 m out and rolls to the left fielder 102 m out at 5.7 s. The third baseman goes out at 1.40 s as the cutoff man home (a ball the left fielder fields) and the shortstop, out as the relay man, goes back to cover third; at 4.20 s the third baseman comes back to third and the shortstop goes out again: third is empty for 0.3 s. Under a double cut with the third baseman as the cutoff man, third would be left to the pitcher; with the first baseman, the third baseman stays home.

*What is wrong.* Two infielders swap jobs where a real infield would add a man, not trade one; and on a ball to left the third baseman, the single's cutoff man home, would leave third uncovered under a double cut, so the game would need a rule for which corner man takes it.

*Options.*
- (a) Leave it: one relay or cutoff man, the swap and its 0.3 s.
- (b) Keep the man already out: when a single becomes a deep ball, the cutoff man already out goes on as the relay man (a corner man playing relay man 40 m out, which a real infield does not do), or the middle infielder already out on a cutoff stays out as the relay man whichever side the ball is on.
- (c) The double cut: on a deep ball with a play at home, the relay man goes out and the first baseman lines up as the cutoff man home behind him, whichever field the ball is in; the third baseman stays at third, the second baseman (or shortstop) covers second, first is left (the batter-runner is past it). The relay man then chooses, as the outfielder does now, between throwing home himself and throwing through the cutoff man. A bigger change: the relay's throw choice gets a second cutoff, the coverage table a new row, and the backups a new case.

*Recommendation:* (c), the standard alignment, as its own job after this merges; the second-base gap would go with it, because who goes out decides who covers second. Until the owner rules, the game keeps (a).

The rest of item 4 that is not a design choice: no bug found. The out-back-out plays are this swap (seeds 23/83, 63/210, 97/119: the shortstop out on his own read while the ball is up, back when a corner man is seen out on the cutoff home and the ball will come down short, out again as it rolls past the outfielder).

## 5. Still open

- Item 4 (§4), waiting on the owner.
- **A single with a runner on third** (seen while measuring item 1; standard alignment, not an owner question). On a single the runner from third scores; the standard throw goes where the next runner is going: third with a runner on first (the shortstop cutting), second with nobody else on. The game lines up for home while he is on the bases (he counts on third at the pitch's base until he scores), so with runners on first and third the corner man is out on the cutoff home and third is the shortstop's base only after the run. Not changed here: it reaches every single with a runner on third, and a runner from third who holds (a shallow blooper) must still count.
- Item 6's doubled bag (§2) and 117/38 (a corner man out before any infielder goes for the ball, then the shortstop goes).
- Seed 93 play 183: the pitcher follows the third baseman between the cutoff home and third.
- The others of the liner-single note's §5 not named here are unchanged.

## 6. Standard run

`sim-cli games --n 300`, seeds 0 [1000], main `7622093` against the final build, run back to back (main's binary is the one built from `7622093`, whose seed 0 output repeats the liner-single final's line for line).

| | Main `7622093` | Final `ea66e2b` |
|---|---|---|
| Runs/team-game | 4.05 [4.39] | FINAL_RUNS |
| GB / LD / FB / PU hit | .308/.666/.104/.005 [.313/.657/.097/.007] | FINAL_HITS |
| HR/PA | 2.74% [2.95%] | FINAL_HR |
| K%, BB% | 22.6, 8.0 [22.0, 8.5] | FINAL_KBB |
| Reached on BIP | 33.5% [33.7%] | FINAL_BIP |
| PA/team-game, pitches/game | 37.8, 239 [37.9, 237] | FINAL_PA |
| Out of play after a throw, a game | 0.123 [0.127] | FINAL_OOP |
| Rules warnings | {} | {} |
| Time, ms per game per core (back to back) | MAIN_TIME | FINAL_TIME |

FINAL_DISCUSSION

## Reproduce

- The probe: `zz_cut_home.rs` (the cutoff-home probe, copied into `crates/bb_game/examples/`, not committed), built on each commit; `BB_COVER_TRACE=1 zz_cut_home 120 0 2> tr.log`. The traces are in `sim/target/codex-probes/handover/`: `tr-base.log` (main), `tr-i1.log` (`7413eb8`), `tr-i2e.log` (`40102f7`), `tr-i3.log`, `tr-i5.log`, `tr-i6.log` (`e99e28d`, `6efe905`, `ba27f63`), `tr-i1h.log` (final).
- The tables: `hc.py a.log b.log …` (§2; `-v` lists plays); `diffplays.py a.log b.log [empty|1st|risp]` (plays new or gone per measure; plays after the first that differs in a game are not the same plays); `seq.py tr.log kind cat zone` (hand-over sequences); `show.py tr.log seed play` (one play's jobs by tick); `pp.py` (ping-pong by fielder and jobs); `p32.py` (a middle infielder leaving second for a cutoff or relay); `cornerif.py` (a corner man on the cutoff home before an infielder's pickup). The review's `pickup.py` and `pingpong.py` are in `sim/target/review-probes/py*/`.
- The fixtures: `sim/target/codex-probes/handover/fxsrc` is `7622093` with the fixture dump block in `game.rs` (`liner/mkdump.py`), built in `handover/tgt`; `ZZ_DUMP=seed:play tgt/release/examples/zz_cut_home 1 <seed>` prints the fixture.

## For the relay review

Commit range `7622093..ea66e2b` (sim); the docs commits after it change no code. Files: `sim/crates/bb_game/src/live.rs` (`relay_job`, `relay_base`, `deep_ball`, `cover_table`, `cover_after_chasers`), `docs/agent-models.md` §5.7, `sim/crates/bb_game/tests/throw_choices.rs` and the 11 frozen fixtures in `tests/fixtures/a4/`. Invariants to hold it to: AGENTS.md 2 (agents never see world state) and 5 (determinism). Questions, most risk first:

1. **`relay_base`'s `held_home` is self-reinforcing.** On a single, a corner man known out on the cutoff home keeps the throw home while any runner from base is still running; he went out because the base was home. Can it hold home where a real infield would turn to third, e.g. a runner on first only after the corner man went out on a misread (the runner was not yet counted at his pitch base), or the runner from second held at third with the runner from first going to second? Does it reach deep balls at all (`!deep`), and should it end when the only runner still moving is going to a base short of third?
2. **`relay_base` reads the runners from the world** (`self.r`: `s`, `v`, `direction`, `start`) as it did before; the new rule adds the runner's speed against `moving_mps` to decide "stands on third". Is that something the viewer can see (it is the same visible-motion threshold `visible_target` uses), or should it come from his read of the runner?
3. **Counting a runner at his pitch base on a single** (item 1). A runner from second who reaches third and keeps going is counted on third only once he stops; while he runs through third he counts on second, which still makes it home, so no change. A runner from first who stops on second counts on first: the throw goes to third. Is there a case where the pitch's base sends the throw to a base behind the play (a runner from first standing on third after a misplay)?
4. **`deep_ball` judged where the outfielder secured it** takes the last `GloveSecured` in the play's facts. With an outfielder holding the ball after a throw from another outfielder (a relay between outfielders), that is where he caught the throw, not where the ball was fielded. Harmless, or does it flip the deep call?
5. **`relay_out` held through `Ball::Held { by: 6..=8 }`.** Any case where the outfielder holds long (a run-down, a fake) and the relay man should give way?
6. **Item 6's gate costs a doubled bag** on 3 plays (§2): the corner man held by the gate while his teammates count him out by the convention. Acceptable, as argued, or should the convention ask the same gate?
7. **`two_to_second`** uses the second baseman's own current role (`self.f[3].role`) while the rest of the table uses what `o` has seen: deliberate (it is his own decision), but check it cannot fire for a viewer other than the second baseman (`o == 3`).
8. **The re-call cover table gives the pitcher third** after the third baseman and shortstop: does that ever take him off backing up home with a play at the plate coming?

Tests to read against the spec: the six in §3; each was run failing on `7622093` by the first worker (fixtures frozen from it).
