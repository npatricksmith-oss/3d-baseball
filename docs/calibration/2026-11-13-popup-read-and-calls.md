# Pop-ups dropping in short right and center: the read, and who calls the ball (2026-11-13)

TASKS "Next" item 1, following docs/calibration/2026-11-13-idle-fielder-target.md and the
Codex idle-fielder review (finding #2 and its ranked causes). Two questions: why an
outfielder's read of a ball launched above 50° is so far off, and when outfield priority
should apply.

## 1. Why the read is off: not spin, the position-depth term

`cargo run --release -p bb_game --example read_error -- 48` reads every fair ball launched
at 25° or more (48 games) from the starting position of the outfielder nearest its landing
point, at several times after contact, through the same function the simulator uses
(`live::read_prediction`, split out of `Sim::predict_from` with no change in behavior: game
digests are identical). Each column is how far one σ of one error term alone moves his
predicted landing point; "spin" is the miss with no read error at all (typical-spin belief
and the 0.1 s agent flight step against the true path, + = predicted deeper). Medians, m.

| launch | t after contact | spin | position depth | position lateral/height | velocity depth | velocity lateral/vertical | all, 1 σ |
|---|---:|---:|---:|---:|---:|---:|---:|
| 25–35° (hang 5.0 s) | 0.4 s | −0.8 | 22.0 | 8.2 | 6.3 | 4.1 | 24.7 |
| | 2.0 s | +0.4 | 6.3 | 2.3 | 1.7 | 1.1 | 7.1 |
| | 4.0 s | +0.2 | 1.7 | 0.4 | 0.5 | 0.3 | 1.9 |
| 35–50° (5.6 s) | 0.4 s | −1.3 | 22.7 | 7.8 | 6.0 | 4.0 | 24.9 |
| | 2.0 s | −0.4 | 8.1 | 2.8 | 1.5 | 1.0 | 8.7 |
| | 4.0 s | 0.0 | 3.1 | 1.1 | 0.6 | 0.4 | 3.4 |
| 50–60° (5.7 s) | 0.4 s | −1.3 | 23.3 | 7.8 | 5.5 | 3.6 | 25.4 |
| | 2.0 s | −0.6 | 9.5 | 3.3 | 1.1 | 0.8 | 10.1 |
| | 4.0 s | −0.1 | 5.0 | 1.8 | 0.5 | 0.4 | 5.3 |
| 60°+ (5.4 s) | 0.4 s | −0.5 | 24.1 | 8.0 | 5.3 | 3.0 | 26.2 |
| | 2.0 s | −0.3 | 11.3 | 3.9 | 0.7 | 0.5 | 12.0 |
| | 4.0 s | −0.1 | 7.2 | 2.5 | 0.4 | 0.4 | 7.7 |

(The observer does not move here, so the late rows are upper bounds; a fielder running in
closes the distance and his error with it.)

- **Spin and drag are not the cause.** The typical-spin table stops at 55° and the launch
  angle is rebuilt without drag, but together they move the landing point by under 1.5 m
  at every launch angle. This agrees with the review's replay (true spin recovers 5 of 26).
- **The early error is the same for every fly: about 25 m, one σ**, nearly all of it the
  position-depth term (`read_sigma0` 3.0 m at 30 m × distance ≈ 90 m × `DEPTH_STRETCH` 3).
- **What is different about pop-ups is that the error does not fade.** The position term is
  proportional to the observer's distance from the ball. A fly hit toward an outfielder
  comes to him and his error goes with the distance (1.7 m with a second left). A shallow
  pop-up comes down 30–40 m in front of him, so with a second and a half left he is still
  5–7 m unsure of its depth, running at full speed, and a late correction costs a turn.

Whether a 25 m depth error at the first look is realistic was not settled here. The interim
read gives the observer no knowledge that the ball started at the plate (at 0.4 s the ball
is within 16 m of home, yet one σ of depth is 22 m). The launch-state estimate of
agent-models §5.1 has that knowledge built in and is where this belongs; the outfield
value was fitted to line-drive and fly-ball hit rates (docs/calibration/2026-11-12-fielder-momentum.md)
and changing the model means refitting them. Nothing in the read was changed.

## 2. Who calls the ball

The old rule: any fielder who believed he would catch the ball in the air with 0.3 s to
spare called it, outfielders over infielders, at the defense's first look. With a 25 m read
an outfielder often believed that about a ball he had misjudged; the infielder who really
had it was called off and (since the idle-fielder fix) stopped.

The new rule (agent-models §5.7, `Sim::call_value`, `assign_among`):

- A fielder **calls** a ball in the air only when he is sure: his time to spare must cover
  `CALL_SLACK_S` plus `CALL_SURE_SIGMAS` (1.0 [E]) σ of his own read's depth error, run off
  at his speed. He knows how good his read is; he does not know its error.
- Sure calls rank as before: outfielder over infielder, then the most time to spare.
- **Until someone is sure, everyone who believes he can catch it in the air goes for it.**
  The one with the most time to spare is the chaser the other jobs are built around. When
  a sure call comes (calls are remade once a second), the others peel off to cover and
  backup jobs, as before.

So on a pop-up to short right-center the second baseman (a 1 m read) calls it at once and
goes out, and the right fielder backs him up; on a fly at an outfielder he calls it at once
(his time to spare covers even a 25 m read); on a fly in the gap both outfielders go until
one is sure. Setting `CALL_SURE_SIGMAS` to 0 gives the old rule exactly (identical game
digests), which is how the "before" column was run.

## Measured

`sim-cli games --n 1024 --seed 5`, main at `454d341` against this change:

| | runs/team-game | GB hit | LD hit | FB hit | PU hit |
|---|---:|---:|---:|---:|---:|
| before | 5.55 | .411 | .723 | .134 | .047 |
| after (1 σ) | 5.47 | .411 | .723 | .122 | .018 |
| 2 σ (not adopted) | 5.41 | | .725 | .119 | .007 |
| 3 σ (not adopted) | 5.38 | | .726 | .118 | .004 |
| MLB BABIP (reference) | | ≈ .24 | ≈ .68 | ≈ .13 | ≈ .02 |

Seed 2, 1,024 games, after: 5.27 runs, FB .120, PU .018. Game time is unchanged (413 vs
423 ms per game per core on 16 loaded threads). `pitcher_roam 1 200`: 42 of 14,981 (0.3%),
as before.

`pu_hits 128` (fair pop-ups not caught, seeds 0–127): 28 → 12. The review's examples are
now caught: seed 41 play 93 by the second baseman, seed 35 play 74, seed 9 play 197.

One σ is kept because it lands on the reference rate and leaves the mistake a real team
makes: the margin covers one σ of the read, so a sure call is sometimes made on a ball
misjudged by more than that, and some of those drop. Two of the twelve, from their
recorded tracks (`trace_play`; the calls themselves were not logged, so who called what is
inferred from who ran and who stopped):

- Seed 82 play 192: a pop-up down the right-field line, landing at (43, 51) m, 26 m in
  front of the right fielder; the second baseman, 33 m away, never goes. The right fielder
  drifts back 2 m over the first two seconds, then comes in and is 3 m short at landing.
- Seed 29 play 135: short center, landing at (−7, 62) m. The shortstop starts for second
  base, pulls up 1.5 s after contact and stands 20 m from where the ball lands; the centre
  fielder does not move until then, comes in, and is 8 m short. That is what a wrong sure
  call by the outfielder looks like: the remaining form of the old bug, now only when his
  read is off by more than his margin instead of on every unsure ball.

## Not done

- **The called-off infielder stands still** (seed 29 play 135). §5.7's "peels off to back
  up" for a man called off under a ball is still not built: a real one drifts out behind
  the caller. With pop-ups at the reference rate this is about how plays look, not a number.
- **The read model itself** (section 1): how fast a depth read should firm up, with the
  ball's origin known, belongs to the launch-state perception of §5.1.
