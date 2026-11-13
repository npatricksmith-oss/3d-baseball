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

On 25 of the 26 pop-ups that drop after the change, the nearest fielder at landing is the
outfielder who had the ball: right or center fielder, median 4.7 m away, usually standing on
the spot he ran to. The landing points cluster 55–65 m out, in short right and center. These
are shallow pop-ups an outfielder called (tier 2, his own read giving at least
`CALL_SLACK_S` to spare) and then misjudged in depth. The infielders are 20–35 m away when
the ball lands.

Before the change, the infielders who heard that call were left `Idle` but still aimed at
their own intercepts, so they kept running out. When the outfielder misread the ball, one of
them was under it and caught it, because a capture does not check who is supposed to catch
the ball. That accidental safety net is what held pop-up hits at .005. One example of the
other pattern: seed 6 play 84, where the second baseman ran out, was called off by the right
fielder at a re-call, and now pulls up 1.3 m from where the ball lands. The right fielder
ends 8 m short.

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
