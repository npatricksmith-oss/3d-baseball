# Fielding: planner fixes and outfielder values, 2026-11-11

Run-environment item 2(a0): grounders getting through the infield. Measured with `cargo run --release -p bb_game --example gb_through -- 128` and `sim-cli games --n 512 --seed 2`.

## 1. Three planner and movement bugs (mechanisms, not knobs)

A new diagnostic in `gb_through` asks, for every grounder that reached the outfield, whether an average infielder could have got to it from his starting position. Before the fixes, of 964 through-balls, 288 were reachable standing and 269 with a dive: over half were fielder mistakes, not gaps. A tick-by-tick trace of a shortstop showed the cause, and two more turned up on the way:

1. **Replans assumed a standing start.** `intercept_scan` computed the distance a fielder could cover with `run_dist` from rest at every replan (every 0.4 s), although he was already running. A fielder 0.7 s from a ball he was about to field would conclude he could not reach it and turn for the ball's resting point in the outfield. Fixed: `run_dist_from` uses his current speed, with the same speed law the movement step uses.
2. **The scan stepped over the answer.** It sampled the predicted path every 0.12 s, in which a hard grounder travels 3–4 m; the stretch a fielder can reach is often under 2 m. Fixed: each step also tests the point of the segment between samples that passes closest to the fielder.
3. **Chasers braked to a stop on the intercept point**, while the plan assumed arriving at speed, so they arrived late. Fixed: a chaser slows for the spot only if he will be there before the ball.

| | Before | After fixes 1–3 |
|---|---|---|
| Routine grounders converted (target ≥ 96%, envelope ≥ 93%) | 88.6% | 94.1% |
| Grounders through to the outfield | 1,108 of 4,315 | 634 of 3,960 |
| … of which reachable standing | 288 (after fix 2 alone) | 110 |
| 80–90 mph grounders through | 29.7% | 18.3% |

## 2. Outfielders were tuned against those bugs

The same fixes apply to outfielders, and with their old values they then caught too much: 3.42 runs/team-game, LD hit rate .593 (MLB ≈ .68), FB .065 (MLB ≈ .13). Their interim values in `FielderAttrs::average` had been set while the planner was losing them time. Re-set, all [E]:

| Outfielder value | Was | Now | Reading |
|---|---|---|---|
| Reaction before the first step | 0.25 s | 0.40 s | Reading a fly off the bat takes longer than reacting to a grounder |
| Top speed while chasing | 8.4 m/s | 8.0 m/s | Tracking the ball costs some sprint speed |
| Read error at 30 m (lateral; depth is 3×) | 1.5 m | 3.0 m | The atlas perception of agent-models §5.1 is not built yet; this stands in for early misreads |

Sweeps that led there (512 games, seed 2; LD / FB hit rate, runs):

| Reaction, speed, read | LD | FB | Runs/team-game |
|---|---|---|---|
| 0.25, 8.4, 1.5 (old values, fixed planner) | .593 | .065 | 3.42 |
| 0.45, 8.4, 1.5 | .633 | .067 | 3.58 |
| 0.60, 7.4, 1.5 | .724 | .121 | 4.14 |
| 0.25, 8.4, 3.0 | .639 | .105 | 3.70 |
| 0.25, 8.4, 5.0 | .704 | .213 | 4.27 |
| 0.40, 8.0, 4.0 | .721 | .181 | 4.15 |
| **0.40, 8.0, 3.0** | **.691** | **.121** | **3.87** |

These three values are a fit to two hit rates, not measurements; public outfield jump and catch-probability data should replace them (research). **Proposed, owner to confirm.**

## 3. Where the run environment stands

512 games, seed 2: **3.87 runs/team-game** (MLB ≈ 4.5; 4.98 before). Hit rates by type (interim ruling): GB .303 (MLB .24), LD .691 (.68), FB .121 (.13), PU .018 (.02). Routine grounders 94.2%.

Runs are now below MLB although ground balls still fall for hits too often. The rest of the gap is on the batting side and is not a fielding question: 56% of balls in play are grounders (MLB ≈ 43%), the contact model's topped-ball excess (TASKS 2(c)), and the plate-appearance mix is off (K 12%, BB 9.5%).

Left for grounders: 364 of 681 through-balls are out of any infielder's reach from the fixed starting positions (the 5–6 hole, the 3–4 hole, and up the middle on the first-base side), 207 need a dive, and infield hits are 8.4% of grounders.
