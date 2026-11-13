# Idle fielders stop chasing the ball: effect on the run environment (2026-11-13)

Change: Part A of docs/calibration/2026-11-12-pitcher-outfield-diagnosis.md. In
`Sim::cover_roles` (`sim/crates/bb_game/src/live.rs`), a fielder left `Role::Idle` now gets
`target = stop_point(..)` (he pulls up where braking stops him) instead of keeping the
intercept that `assign_among` wrote for every call candidate. Nothing was tuned.

## Measured

`sim-cli games --n 1024 --seed 5`, base `760afe8` (coord/review-fixes) against this change:

| | runs/team-game | PA/team-game | GB hit | LD hit | FB hit | PU hit |
|---|---:|---:|---:|---:|---:|---:|
| before | 5.27 | 39.7 | .409 | .722 | .122 | .005 |
| after | 5.48 | 40.1 | .410 | .722 | .131 | .044 |
| MLB BABIP (reference) | | | ≈ .24 | ≈ .68 | ≈ .13 | ≈ .02 |

Runs rose 0.21 per team-game. Ground balls and line drives did not move. All of the change
is in fly balls and, mostly, pop-ups (launch > 50°): pop-up hits went from 0.5% to 4.4%.

`pitcher_roam 1 200` (the symptom): the pitcher ended more than 30 m from the mound and
every base on 2,030 of 14,777 balls in play (13.7%) before, 39 of 14,856 (0.3%) after. Of
the 39, 35 are balls hit behind home (spray > 100°) that he chases as the primary, and 4 are
loose throws behind home or in foul ground that he chases. All 39 are `Chase` plays, which
the diagnosis expected the fix to leave.

## Why pop-ups drop now

A throwaway probe (128 games, seeds 0–127, not committed) listed every fair ball launched
above 25° that was not caught and came down within 75 m of home, with the nearest fielder
when it landed:

| | uncaught shallow FB (25–50°) | uncaught shallow PU (> 50°) |
|---|---:|---:|
| before | 25 | 3 |
| after | 32 | 26 |

Corrected after the Codex idle-fielder review (finding #3, P3; numbers re-measured with the
review's landing snapshot at `fix/idle-fielder-review`, seeds 0–127, the 26 uncaught fair
pop-ups landing within 75 m, each snapshot at the first tick at or after the landing time):

- On 25 of the 26 the nearest fielder at landing is an outfielder (right or center fielder,
  two left fielders), median 4.93 m from the landing point. The landing points are 51–71 m
  from home, mostly in short right and center.
- These fielders are **not standing on a spot**: 24 of the 26 nearest fielders are moving
  faster than 4 m/s (median 8.14 m/s), and none slower than 0.5 m/s. The pattern is an
  outfielder who called the ball (tier 2, his own read giving at least `CALL_SLACK_S` to
  spare), misread its depth early (outfield `read_sigma0` is 3.0 m against 0.3 m in the
  infield, stretched in depth), corrected his route all the way in, and was still running
  when it came down. Seed 9 play 197 is an example: at 0.65 s after contact the left
  fielder's read puts the landing near (−29, 92) m (review trace); it lands at (−8, 59) m and
  he is 11.8 m away, still running at 8.2 m/s. A late route after an early depth error is a perception question, not a
  stationary fielder refusing to adjust.

Before the change, the infielders who heard that call were left `Idle` but still aimed at
their own intercepts, so they kept running out. Replaying each of the 26 from the same
contact input and rules state with the old target behavior, 23 are caught. Most of those are
not catches by an idle man: at the first successful glove contact **19 of the 23 catchers
have regained `Chase`** (their drift put them close enough to win a later re-call) and only
**4 are still `Idle`** (a capture does not check who is supposed to catch the ball). Seed 35
play 74 is a direct idle rescue: with the old behavior the idle second baseman pockets the
ball 0.81 m from him, 1.2 m up, 5.48 s after contact and 0.06 s before it would land; now he pulls up, and the right fielder, who called it, is 3.85 m away
at landing, running at 8.1 m/s. (The earlier example here, seed 6 play 84, was wrong: the
second baseman is `Chase` throughout, never called off; he is 1.30 m away and moving
1.09 m/s at landing after a dive that rims the ball, and the old behavior does not catch it
either.)

## Assessment

The change is correct as a model of the rule: a fielder with no job should not chase the
ball. It exposes two gaps that the drift had hidden:

1. **Outfielders misjudge shallow pop-ups and still win the call.** The call rule gives the
   ball to an outfielder on his own read, and a re-call never takes it from him for an
   infielder with a better read, because tier 2 beats tier 1. Whether the depth error is
   realistic for a ball hit that high has not been checked.
2. **"A fielder who hears a higher-priority call peels off to back up"** (§5.7) is not built.
   A called-off infielder now stops. A real one keeps drifting under the ball behind the
   caller, which is the legitimate form of the old safety net.

The fly-ball rate (.131) is now at the MLB reference. The pop-up rate (.044) is about twice
it, and runs are 0.21 higher. Following the close-enough rule, nothing is retuned here. The
two gaps above are the mechanisms to fix if pop-ups need to come down. Neither is a knob.
