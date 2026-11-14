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
- **The platoon advantage is real in the sim, and too big.** Against an opposite-handed pitcher the average batter strikes out 14.4% of the time, against a same-handed one 19.8%: 5.4 points. Nothing sets it. The batter sees a pitch by the angles from his eyes (agent-models §3.2), which are 0.75 m to his side of the plate; a same-handed pitcher lets go on that same side, so the ball comes nearly down his line of sight and its sideways movement shows late, and his breaking balls move away from the batter. In MLB the gap is smaller: batters hit about .030 to .045 of wOBA better against opposite-handed pitchers (FanGraphs community research; the gap has been shrinking), and the strikeout gap is about 3 points (my recollection; I found no published league table to cite). So the direction is right and the size is roughly double. It is recorded here, not tuned: the candidates are the eye offset (0.75 m [E]) and how much of a pitcher's release side the batter's scouting model already expects.

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

About 45% of plate appearances are now opposite-handed (they were none), and those batters strike out less, put more balls in the air and hit more home runs. **Runs rose by about 0.6 a team-game and strikeouts fell by 2.6 points.** The run environment is back near where it was before the dive-rate work (5.48). The fix belongs to the next calibration of the plate-appearance knobs, which should be done over the real mix of pairings, and to the size of the platoon gap above; it is listed open in TASKS.

## Dives (300 games from seed 1000, `examples/dive_probe.rs`)

3.8 dives a game (3.3–3.7 before), 43% caught. The shortstop dives 0.98 times a game and the second baseman 0.90; the dive-rate note blamed part of the shortstop's excess on every batter pulling the ball to his side. With left-handed batters the second baseman now gets as many, so the infield total did not fall: the near-miss dives from the earliest-point fielding plan are the larger cause.

## Fielders' throws

A throw now leaves the thrower's hand, 0.35 m [E] to his arm side (agent-models §5.5), instead of the middle of his body. This changes every throw by a fraction of a degree and is not separable from the batters' change in the numbers above. What the glove hand does to fielding is not built (same section).
