# Handedness: what left-handers did to the game (owner backlog A1, 2026-11-13)

**Nothing was tuned.** Games now have left-handed pitchers, left-handed batters and switch hitters (data-formats §2.4). A platoon advantage came out of the geometry with no setting for it, and it moved the run environment up, because the interim knobs were calibrated with every batter right-handed against a right-handed pitcher.

## The four pairings (`examples/platoon.rs`, 4,000 plate appearances each, calibrated knobs, average players)

| Pitcher / batter | K% | BB% | HBP% | in play % | whiffs per swing % | mean spray (+ = right field) |
|---|---|---|---|---|---|---|
| R / R | 19.7, 19.9 | 8.0, 7.5 | 0.38, 0.23 | 72.0, 72.4 | 35.7, 35.8 | −5.6°, −5.2° |
| L / L | 20.0, 19.1 | 7.4, 7.2 | 0.35, 0.35 | 72.2, 73.3 | 35.2, 34.3 | +6.1°, +5.7° |
| R / L | 14.8, 14.2 | 7.3, 6.6 | 0.35, 0.40 | 77.5, 78.7 | 31.5, 31.3 | +5.5°, +5.4° |
| L / R | 14.3, 14.1 | 6.9, 6.8 | 0.30, 0.23 | 78.5, 78.9 | 31.4, 30.1 | −5.7°, −5.3° |

(Two values per cell: seeds 1 and 2.)

- **A left-hander is a right-hander in a mirror.** The two same-side rows agree and the two opposite-side rows agree, within sampling error (one sd of a K% here is 0.6 points), and every batter pulls the ball by the same 5° to 6°, to his own side.
- **Pitchers hit left-handed batters no more often** than right-handed ones (the "check that pitcher targeting handles left-handed batters" part of A1). Targets are symmetric across the plate and the body is on the batter's own side.
- **The platoon advantage is real in the sim, and too big.** Against an opposite-handed pitcher the average batter strikes out 14.4% of the time, against a same-handed one 19.8%: 5.4 points. Nothing sets it. In MLB the gap is smaller: batters hit about .030 to .045 of wOBA better against opposite-handed pitchers (FanGraphs community research; the gap has been shrinking), and the strikeout gap is about 3 points (my recollection; I found no published league table to cite). So the direction is right and the size is roughly double. It is recorded here, not tuned.
- **Where it comes from: sideways movement, seen from the batter's side.** My first account (the ball comes down a same-handed batter's line of sight, so the eye offset is the cause) was a guess, and the Codex review tested it with scratch interventions (4,000 PA per pairing, seed 3; K% for R/R, L/L, R/L, L/R):

  | Intervention | R/R | L/L | R/L | L/R |
  |---|---|---|---|---|
  | none | 20.0 | 19.3 | 13.8 | 14.7 |
  | eyes at x = 0 | 26.3 | 26.0 | 21.9 | 21.8 |
  | release at x = 0 | 19.4 | 18.6 | 13.9 | 15.4 |
  | eyes and release at x = 0 | 25.6 | 24.6 | 23.0 | 22.7 |
  | no sideways spin on any pitch | 17.1 | 16.3 | 15.1 | 16.3 |
  | eyes and release at x = 0, no sideways spin | 24.8 | 24.6 | 24.4 | 24.9 |
  | exact scouting | 12.2 | 11.9 | 8.0 | 8.3 |

  Neither the eye offset nor the release side alone explains the gap. It shrinks to about 2 points when both are centred, to 1 or 2 points when pitches have no sideways spin, and vanishes only with all three. So it is the handed movement of the pitches (arm-side run, glove-side break) meeting a batter who sees and swings from one side, not one parameter. No sign error was found. These are counterfactual populations, not a decomposition, and the eye offset is not shown to be the right thing to adjust.

## Games (300 games each; before = every player right-handed, main at `3de5572`; MLB column approximate)

| | seeds 0–299 before | after | seeds 1000–1299 before | after | MLB |
|---|---|---|---|---|---|
| runs / team-game | 4.73 | 5.35 | 4.89 | 5.43 | about 4.4–4.6 |
| K% | 17.8 | 15.3 | 17.8 | 15.1 | 22 |
| BB% | 7.2 | 7.0 | 7.7 | 7.4 | 8.5 |
| HR / PA | 4.09% | 5.21% | 4.22% | 5.26% | 3% |
| GB share | 53.0% | 50.9% | 52.8% | 49.6% | |
| GB hits | .333 | .346 | .343 | .349 | .24 |
| FB hits | .162 | .124 | .138 | .133 | .13 |

Half of plate appearances are now opposite-handed (49.8% measured over seeds 0–299: R/R 9,449, R/L 7,474, L/R 3,965, L/L 2,086, counted by the Codex review; they were none), and those batters strike out less, put more balls in the air and hit more home runs. **Runs rose by about 0.6 a team-game and strikeouts fell by 2.6 points.** The run environment is back near where it was before the dive-rate work (5.48). The fix belongs to the next calibration of the plate-appearance knobs, which should be done over the real mix of pairings, and to the size of the platoon gap above; it is listed open in TASKS.

## Dives (300 games from seed 1000, `examples/dive_probe.rs`)

3.8 dives a game (3.3–3.7 before), 43% caught. The shortstop dives 0.98 times a game and the second baseman 0.90; the dive-rate note blamed part of the shortstop's excess on every batter pulling the ball to his side. With left-handed batters the second baseman now gets as many, so the infield total did not fall: the near-miss dives from the earliest-point fielding plan are the larger cause.

## Fielders' throws

A throw now leaves the thrower's hand, 0.35 m [E] to his arm side (agent-models §5.5), instead of the middle of his body. This changes every throw by a fraction of a degree and is not separable from the batters' change in the numbers above. What the glove hand does to fielding is not built (same section).
