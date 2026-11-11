# After the owner's first watch, 2026-11-11

The owner watched client v0 and named four things: players look like cylinders, the window cannot be quit, too many double plays, and fielders with no part in a play should back up throws (references: Pro Baseball Insider's outfield backup article and the Spiders "Cutoff & Backup Responsibilities Guide"). This records what changed in the sim and where its numbers moved. Client changes are in docs/unreal-spike.md.

## Double plays

40 games of play-by-play (`examples/calls`, seeds 1–40) had 146 double plays (3.6 a game; MLB is about 1.7) and, before an earlier fix, 5 triple plays. Causes found, each a missing piece of baseball, and what was done:

| Cause | Change |
|---|---|
| Runners stood on their bases until 0.5 s after contact | Runners start with a lead (first 5.5 m, second 6.5 m, third 4.5 m [E], lead plus the shuffle as the pitch is delivered); the rules engine is told they have left. A runner tagging up goes back to the base first |
| On any ball in the air a runner either tagged or ran, and was doubled off when a fly he ran on was caught (4% of fly balls ended in two outs) | With fewer than two out, on a fly or liner to the outfield that might drop, he goes 5 m past his lead and watches; he decides when it lands and goes back if it is caught |
| The batter took 4.63 s to reach first | Runner acceleration time constant 1.0 → 0.85 s and the batter's start delay 0.30 → 0.25 s: 4.45 s for an average runner (MLB averages about 4.4) |
| 56% of balls in play were grounders (MLB ≈ 43%) | The batter's aim under the ball's center goes from 1.3 to 2.0 cm (`SwingTemplate::aim_offset_z`): 53% grounders |
| Relay exchanges took a flat 0.25 s | 0.7 × the fielder's transfer time (earlier the same day) |

Those changes raised scoring to 5.7 runs per team (more balls in the air, more runners safe), so two calibration values moved with them, both owner-level knobs:

- `K_perception` 2.25 → 2.7 (`Knobs::calibrated_m3`): strikeouts 12% → 17–20%.
- Discipline's grade-50 values c_D 1.0 → 0.5 and b_D 0.0 → +0.9 (the proposal from the chase diagnosis): swing rate 36% → 48%, chase rate into its band.

## Where it landed

| | Before today's watch | Now | MLB |
|---|---|---|---|
| Double plays per game (both teams) | 3.6 | 2.0 | ≈ 1.7 |
| Triple plays in 40 games | 1 | 0 | ≈ 0 |
| Runs per team per game | 4.41 | 4.48 | ≈ 4.5 |
| K% / BB% (games) | 12.1 / 9.5 | 17.4 / 7.4 | 22.5 / 8.3 |
| HR per PA | 4.3% | 4.0% | 3.1% |
| Ground-ball share of balls in play | 56% | 53% | ≈ 43% |
| Fly balls ending in two outs | 3.1% | 0.6% | rare |
| Hit rate: GB / LD / FB (provisional ruling) | .311 / .701 / .121 | .377 / .708 / .135 | .24 / .68 / .13 |
| Routine grounders converted | 92.3% | 91.0% | ≥ 96% |

Plate-appearance report (`sim-cli pa --n 3000 --k-perception 2.7 --k-command 0.8`): 10 of 16 B1 metrics in band. Out: HBP, pitches/PA 3.1, Contact% 65 (target 76), Z-Contact, O-Contact, SwStr 17, foul share 24. Sensitivity: 10 of 10 attributes move their metric the right way.

**What got worse:** ground balls go for hits more often (.377). With leads, the lead runner is safe more often, and a faster batter beats more throws; infield hits are about 16% of grounders. The owner's direction is not to chase percentages, so this is recorded, not tuned.

## Backups

`Sim::assign_backups` (agent-models §5.7): fielders with no job back up the play, one per base, on the line of the likely throw and 13 m beyond the base.

- Behind the fielder: the nearest free outfielder gets into the ball's line beyond him.
- Outfield hit, nobody on: the far-side outfielder backs up second; the catcher backs up first.
- Outfield hit, runner on first: the pitcher backs up third from foul ground; the far-side outfielder backs up second; the catcher stays home.
- Outfield hit, runner in scoring position: the pitcher backs up home.
- Infield ball: the catcher (bases empty) or the right fielder backs up first; the center fielder backs up second with a runner on first; the left fielder backs up third with a runner on second.
- When a throw is released, the backup of that base lines up with the actual throw; if nobody has the job, the nearest fielder with nothing to do takes it.

Not done: cutoff and relay men (the shortstop or second baseman going out to take the outfielder's throw), which both references describe. Throws still go straight to a base.
