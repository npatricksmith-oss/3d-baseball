# The tracking read, and arriving under control (2026-11-15)

> Superseded figures: current numbers are in the [metrics ledger](../metrics-ledger.md) (§1).

Branch `work/tracking-read`, from `integrate/held` `2657f65` (the three held branches together; [integration note](2026-11-15-integration-of-the-held-branches.md)). Spec: agent-models §5.1 ("The tracking read") and §5.2. Commits:

| Commit | What |
|---|---|
| `45fc8b2` | Spec |
| `9d80e98` | Code and tests |
| `0db39d9` | The first look keeps the old first read's shrink |
| `c93288e` | Fixes from the Codex review (§3a) |

## 1. Diagnosis (seed 23080 play 8)

On the integration the owner's play is 98.5 mph at 45°, with 6.5 s of hang time, and the left fielder 64 ft away. It fell. Decomposing his read draw by draw:

- His fixed depth draw is −2.81σ, close to its ±4σ truncation.
- While the plate cue carried his depth estimate, he was right. Its depth gain was 1.0 to 1.5 s, then 0.8 at 2.4 s and 0.3 at 3.6 s.
- As that gain fell, the same −2.81σ, ×3 in depth, took over on every look. His landing point went +11 m deep at mid-flight.
- With no draws, the read stays within 0.3 m throughout.

So two seconds of good looks counted for nothing (the outfield-read review's P2-1).

## 2. What was built

**A running estimate.** For a ball on its first flight off the bat, each fielder holds an estimate of its position and velocity, plus the extra acceleration his flight model is missing (the curvature of the spin he cannot see). It is a Kalman filter in 3×3 blocks:

- He runs the estimate on with his own physics.
- He looks every 0.1 s. A look is good across his three-dimensional line of sight and three times worse along it.
- Each look has fresh noise (the new `Purpose::FielderLook` = 16) plus a persistent part: `TRACK_BIAS_FRAC` of the play's fixed draws.
- His read is the estimate flown on with the learned acceleration (`FlightEnv::extra`, zero in the world).
- His doubt is the estimate's covariance carried to the landing point.

**Arriving under control.** A chaser who is early for the ball stops one doubt short of his spot.

**The reach estimate over a short way.** Within 5 m of the goal it steps the movement itself until he is heading for the goal.

**What was tried and rejected:**

| Tried | What happened |
|---|---|
| A velocity random walk standing in for the unseen spin | The velocity wandered 2.6 m/s per √s, and the read never settled. |
| Each look √(τ_read/`LOOK_S`) = 2.45× the old σ (the old viewing-time law for a still ball) | 16° of angular error per look; the read was off by 10–20 m at 1–3 s. |
| The first look at the plain σ | Infielders read a hard grounder before its first bounce 10× worse than before; grounder hits rose .291 → .308. With the old first read's shrink, .298. |

**Fit.** `LOOK_SCALE` and `TRACK_BIAS_FRAC` were fitted together to the owner's band for a routine fly, on seeds 0–99:

| Look scale | Bias fraction | Routine flies caught |
|---|---|---|
| 0.5 | 0.2 | .906 |
| 0.3 | 0.2 | .941 |
| **0.2** | **0.2** | **.967** |
| 0.3 | 0.1 | .986 |

The arrival margin was worth about +1.4 points (.892 without it at 0.5 / 0.2). The chosen look scale is 1.3° across the line of sight at grade 50. The bias fraction is 0.2 of the old snapshot read's σ at one look.

## 3. Measurements (300 games, seeds 0–299 [the `--seed 1000` set])

| | integration (`2657f65`) | tracking read (`0db39d9`) | real MLB |
|---|---|---|---|
| runs / team-game | 3.88 [3.92] | **3.93 [4.09]** | 4.39 |
| K% | 22.3 [22.3] | 22.3 [22.2] | 22.58 |
| BB% | 8.3 [8.8] | 8.4 [8.6] | 8.18 |
| HR / PA | 2.57% [2.65%] | 2.64% [2.88%] | 2.99% |
| reached on BIP | 34.7% [34.2%] | 34.6% [34.2%] | |
| GB hits | .291 [.295] | .298 [.303] | about .245 |
| LD hits | .668 [.665] | .658 [.649] | about .68 |
| FB hits | .112 [.116] | **.125 [.121]** | about .13 |
| PU hits | .028 [.018] | **.016 [.014]** | about .02 |
| PA / team-game, pitches / game | 38.4, 243 [38.8, 244] | 38.1, 240 [38.3, 240] | |
| rules warnings | none | none | |
| routine flies caught (`fly_misses 0 300`) | .956 (.926 / .962 / .979) | .958 (.954 / .956 / .973) | 90–99% by ability |
| grade 20 / grade 80 (seeds 0–119) | not run | .924 / .979 | |
| flies in the outfield that fall | .106 | .112 | |

**Read error of the responsible outfielder on routine flies.** Depth is along the line from home to the landing point; each cell is the sd in metres, integration → tracking read.

| | 2 s before landing | 1 s before | 0.5 s before |
|---|---|---|---|
| Depth | 2.0–2.9 → **0.9–1.1** | 0.6–0.9 → 0.5–0.6 | 0.2–0.4 → 0.3–0.4 |
| Lateral | 2.0–2.5 → 2.1–2.6 | 0.7–0.8 → 1.0–1.2 | 0.22–0.26 → **0.6–0.9** |

The mid-flight depth error is cut by more than half, and it no longer drifts. The late lateral error is worse (§5).

**The 75 routine flies not caught:**

| Cause | Flies |
|---|---|
| In dive range, no dive | 32 |
| Read good late, too far to make up | 22 |
| Reached, not caught | 13 |
| Left to a teammate, who missed | 5 |
| Late read error | 3 |

## 3a. After the Codex review fixes (`c93288e`; one test pass, one measurement)

The review fixed three things:

- the doubt now carries the whole nine-state covariance;
- the first look's persistent part is shrunk as well;
- within 5 m the reach estimate steps the movement into reach, walls included.

| | tracking read (`0db39d9`) | after the fixes (`c93288e`) |
|---|---|---|
| runs / team-game | 3.93 [4.09] | **3.83 [4.23]** |
| K% | 22.3 [22.2] | 22.4 [22.1] |
| BB% | 8.4 [8.6] | 8.3 [8.5] |
| HR / PA | 2.64% [2.88%] | 2.58% [2.86%] |
| reached on BIP | 34.6% [34.2%] | 34.6% [34.8%] |
| GB hits | .298 [.303] | .300 [.316] |
| LD hits | .658 [.649] | .666 [.657] |
| FB hits | .125 [.121] | .120 [.112] |
| PU hits | .016 [.014] | .014 [.008] |
| PA / team-game, pitches / game | 38.1, 240 [38.3, 240] | 37.9, 239 [38.8, 243] |
| rules warnings | none | none |
| routine flies caught (25–35° / 35–50° / 50°+) | .958 (.954 / .956 / .973) | **.965** (.967 / .966 / .953) |
| flies in the outfield that fall | .112 | .104 |

Routine flies stayed in the owner's band, so `LOOK_SCALE` and `TRACK_BIAS_FRAC` were not refitted. Runs moved −0.10 [+0.14], with opposite signs on the two sets, the size of their spread. The grounder hit rate on the `--seed 1000` set rose .013 and is not explained.

Routine high flies (50°+) are now caught slightly less often than low ones: .953 on 215 against .967 on 367. That is within about one standard error, so it is a direction, not a finding.

**The 61 routine flies not caught:**

| Cause | Flies |
|---|---|
| In dive range, no dive | 27 |
| Reached, not caught | 16 |
| Read good late, too far to make up | 13 |
| Left to a teammate, who missed | 4 |
| Late read error | 1 |

## 4. The review findings this closes

- **Outfield-read review P2-1** (the read keeps no track of what it has seen): **closed.** The read assimilates looks and learns the curvature with his own physics. The owner's play is now a test of its preserved contact (`the_owners_fly_seed_23080_play_8_is_caught`) and is caught: the centre fielder, with priority, takes it 1 m from where it lands. The ignored pinned-seed test is removed.
- **Wall-braking review P2-1, first half** (a chaser at full speed on a moving read overshoots): **addressed.** He arrives with a margin of his doubt. The review's pop-up, as a constructed ball, is caught on at least 18 of 20 plays of draws. Pop-up hits are .028 [.018] → .016 [.014].
- **Wall-braking review P2-1, second half** (play 88: an on-time caller displaced): **traced, no rule change.** On `work/wall-braking` the first baseman's own snapshot read drifted him from a sure call (0.66 s) to unsure (2.42 s) and to "no catch in the air" (3.44 s). Only then, at 3.74 s, did the general rule hand the ball to the right fielder. The cause is the read, which this job replaces. On this branch play 88 is another ball.
- **Wall-braking review P3-1** (`run_gap` optimistic under 5 m): **closed.** The reach estimate steps the movement. The bound is back to −0.35 s; the worst case is now −0.29 s. After the Codex review's P2-3 it steps all the way into reach, walls included, and is checked below 2 m and into a wall too (worst 0.18 s).
- **Not closed (now borderline after the review fixes, §3a): the owner's point that on routine plays high flies are not harder than low ones.** The read is harder on a high fly early and at the top of its flight (`the_tracking_read_is_harder_on_a_high_fly_early_and_converges_late`: RMS landing miss 13.2 m against 8.1 m one second after contact, and 2.3 against 2.0 at the top). On routine plays, though, high flies are still caught at least as often as low ones (.973 at 50°+ against .954 at 25–35°). A high fly gives a routine play more time, and the read has converged by then. See Q1 in the report.

## 5. What is still off

- **Runs** are 3.93 [4.09] against 4.39, +0.05 [+0.17] on the integration. Pop-up and fly-ball hits rose back toward MLB's, grounder hits rose .007 [.008], and liner hits fell slightly. Nothing was tuned toward the run total.
- **The late lateral read is worse** (sd 0.6–0.9 m half a second out, against 0.2–0.3). The old read's closing made its last half second nearly exact. The tracking read's looks are limited by distance, and its persistent part does not average out. A real fielder close to the ball has binocular depth, which this read does not give him; that is a candidate follow-up. Routine catches held, but the 13 "reached, not caught" and the 32 "dive range, no dive" may include it.
- **Grounder hits** are .298 [.303] against .291 [.295] on the integration. The residual is about the size of the seed sets' spread, and its cause is not separated.
- **The spin belief ignores exit speed.** The staleness test now bounds soft contact at 50% (a fourth sample showed 39%) and hard contact at 25%.
- **Dives not taken:** 32 of the 75 misses. Recorded here; `OF_DIVE_MIN_P` was not changed.
