# Runner acceleration refit and the batter's own route to first (owner's Q1)

2026-11-15, branch `work/runner-accel`, on main `b532a67`. Source of the task: the tag-up review's P2-3 ([review](../reviews/2026-11-14-gemini-tag-up-review.md)) and the owner's ruling Q1. Spec: agent-models, the runners paragraph at the end of §5.7, §6.2, §6.4.

## 1. What changed

| | before | now |
|---|---|---|
| Acceleration time constant τ, v = v_max(1 − e^(−t/τ)) | 0.68 s, one constant | **0.76 s** [E], the league average; each runner carries his own (`RunnerIn::tau_s`) |
| Standing 90 ft at 8.2 m/s | 4.02 s | **4.10 s** |
| Tag-up set start | its own `tau_s` 0.68 s | the runner's own τ × `tau_gain` 1 [E] |
| Batter's distance to first | 27.43 m for both hands | his route: **27.98 m right-handed, 26.58 m left-handed** |
| Batter's first step after contact (`BATTER_START`) | 0.25 s | **0.20 s** [E] |
| Home to first, average runner (computed) | 4.27 s both | 4.37 s right-handed, 4.20 s left-handed, mean 4.29 s |

A fielder judging a runner still assumes the league values (8.2 m/s, 0.76 s); he does not know the runner's own.

## 2. Why these values

**τ.** Statcast 2017–2020, runners on third in sacrifice-fly situations (n = 848): mean standing 90 ft 4.03 s at a mean sprint speed of 27.62 ft/s, slope −0.115 s per ft/s ([tag-ups note](2026-11-14-tag-ups.md) §1). The league-average 27.0 ft/s runner is 0.62 ft/s slower: 4.03 + 0.62 × 0.115 = 4.10 s. The earlier 0.68 s fitted 4.03 s to an 8.2 m/s runner, as if the sample were average runners. With 8.2 m/s, t − τ(1 − e^(−t/τ)) = 27.432 / 8.2 = 3.345 s gives τ = 0.76 s for t = 4.10 s.

Left as it was: the league top speed stays 8.2 m/s (26.9 ft/s; 27.0 ft/s is 8.23 m/s, and Statcast's sprint speed is a best one-second window, so the asymptote would be nearer 8.25). The difference is 0.02 s over 90 ft.

**The route.** The batter stands in his box, about 0.95 m off the plate's centre line. He runs straight to the point 4 m up the first-base line (`BOX_EXIT_M`, unchanged) and then along it. From the right-handed box that first stretch is 4.55 m, from the left-handed box 3.15 m. His path coordinate now starts at 4 m minus that length (−0.55 m, +0.85 m), so every time and distance computed from it is his real one. Before, both were charged 4.00 m for it.

**`BATTER_START`.** With the routes and the new τ, 0.25 s gave 4.42 s right-handed and 4.25 s left-handed, mean 4.34 s. The 2016 means are 4.30 s and 4.26 s (mean 4.28 s); the scouting scale's average is 4.3 and 4.2. At 0.20 s the mean is 4.29 s. This is a fit of one number to one mean.

## 3. What does not fit

The sim's left/right gap is 0.17 s, all of it geometry (1.40 m at 8.2 m/s). The real gap is 0.04 s in the 2016 means and 0.1 s on the scouting scale. The populations differ (the means are not at equal speed), and nothing here models the swing's follow-through: a right-handed batter's carries him toward third, which would widen the gap, not close it. Not resolved. A per-hand start delay would be a fudge without a source, so there is none.

## 4. Measured

Standard run, main `b532a67` (its sim code is `385038d`'s, the tracking-read row) → `work/runner-accel`, seeds 0–299 [1000–1299]:

| | before | after |
|---|---|---|
| runs / team-game | 3.83 [4.23] | 3.85 [3.99] |
| K% | 22.4 [22.1] | 22.3 [22.2] |
| BB% | 8.3 [8.5] | 8.2 [8.5] |
| HR / PA | 2.58% [2.86%] | 2.72% [2.75%] |
| reached on BIP | 34.6% [34.8%] | 35.3% [35.2%] |
| GB hits | .300 [.316] | .310 [.310] |
| LD hits | .666 [.657] | .667 [.650] |
| FB hits | .120 [.112] | .116 [.113] |
| PU hits | .014 [.008] | .011 [.009] |
| PA / team-game, pitches / game | 37.9, 239 [38.8, 243] | 38.0, 238 [38.5, 241] |
| rules warnings | none | none |

Nothing moved by more than the two sets' spread, and the sets disagree in sign on runs and on grounder hits. Expected by mechanism: right-handed batters about 0.10 s later to first, left-handed batters about 0.07 s sooner, every runner 0.08 s later over a base from rest. No per-hand split of infield hits was measured.

Tests: 419 pass, 0 fail, 4 ignored. One fixed play changed: seed 23 play 109, a 3-2-3 double play, is now one out at home; the batter is left-handed and beats the catcher's throw to first by about 0.05 s. The test keeps what it was for (the lead force is taken, at full effort, and the catcher goes on to first).
