# The tracking read's persistent error, carried as a state (2026-11-17)

Branch `work/track-bias`, from main `f9e43ed`. Spec: agent-models §5.1, "The persistent error as a state". Owner's saved play: seed 40580, play 15 (observations.md).

## 1. Diagnosis (from TASKS.md, 2026-11-16)

The play is a routine fly to centre: launch angle 38°, 5.9 s of hang time, and the centre fielder needs 50 ft of the 122 he can cover. He called it at 0.7 s and was there in time. But from 1 s to 0.6 s before it came down, with the ball 8 m from him, his tracking read stayed 2.2–2.5 m off *laterally*. He closed too late, stopped 1.6 m from the ball, did not dive, and the ball went to the wall for an inside-the-park home run.

An A/B with `look_noise()` found the cause. Each look's persistent part (`TRACK_BIAS_FRAC` times the play's fixed draws) went into the Kalman update as if it were fresh noise on every look. The filter's covariance shrank as if N looks averaged the persistent part down by √N, which a repeated error cannot do. So the filter grew sure of a lateral velocity worked out from the early, far looks, and the good close looks could not move it.

**Measured** (`the_tracking_read_reports_a_covariance_that_covers_its_error`): an average centre fielder 12 m short of a routine fly (38°, 41 m/s), over 120 plays of draws. Two seconds before landing his lateral error was 0.92 m RMS, while his own σ was 0.18 m. The mean of error² / σ² was **26.7**; an honest filter gives 1.

## 2. What was built

The filter now carries the persistent error as four more states, b: position along his line of sight, position across it, velocity along it, velocity across it, each in units of its own σ. That makes thirteen states with a dense covariance.

- **Prior:** b ~ N(0, I), independent of the ball. His first velocity look carries b too, so the starting velocity estimate, and the position flown back from the plate, start correlated with it.
- **Between looks:** b does not change.
- **Each look:** it sees r + H_b·b + fresh noise. H_b is the two directions (along ×3, across level) times `TRACK_BIAS_FRAC`·σ0·d/30 m at that look, the same scaling the world uses when it draws the error. Velocity works the same way, with vel_frac·|v̂|.
- **Geometry:** the filter's distance, directions and speed now come from his own prediction before the look (on the first look, the plate cue's position and the velocity as seen), not from the ball's true position. That is an extended Kalman filter's linearisation. The world's draw still uses the true geometry.
- **His doubt** (the landing σ that calls weigh) loses its extra persistent-error term. That term is now inside the covariance, so keeping it would count it twice.
- **Draws:** no new draws, and the draw order is unchanged.

**Why a state rather than a wider covariance per look.** In metres, the persistent part is a misjudged angle times the range, plus a misjudged fraction of the range. It shrinks as the ball comes in, and it turns as his line of sight turns. A filter that carries it as a state learns it from that changing geometry: the looks up close correct a misjudgment made off the bat, as a real outfielder's eye does. Widening each look's covariance by the persistent part would still treat it as independent from look to look, so N looks would still average it down by √N. That is less wrong, but not honest.

**Built first, and rejected: the geometry from the look itself.** At `TRACK_BIAS_FRAC` 0.2 it was honest: mean error² / σ² 0.98 / 1.43 / 1.29 at 2 / 1 / 0.6 s before landing. At 0.6 it was 2.4 again 2 s out, and his doubt 1 s out was half his miss (0.40 m against 0.78 m). A look is up to several metres off in range, so the filter misjudged the scale of his own persistent error by up to a fifth. That slip is itself persistent and unmodelled, and it grows as the square of the persistent part.

**After the change** (`TRACK_BIAS_FRAC` 0.6, same test): lateral error 0.51 / 0.30 / 0.14 m RMS at 2 / 1 / 0.6 s before landing, against his own σ of 0.53 / 0.27 / 0.13 m. The mean error² / σ² is 0.92 / 1.21 / 1.16. The doubt test (balls from games, real spins) has RMS doubt 1.06 / 0.41 m against an RMS miss of 0.83 / 0.39 m at 2 / 1 s.

## 3. Refit to the owner's band

The honest filter reads better, so the old values (`LOOK_SCALE` 0.2, `TRACK_BIAS_FRAC` 0.2) caught too many routine flies. The look scale is a property of the eye (1.3° per look at grade 50) and was left alone; the persistent part was refitted. Band: a routine fly caught 90–99% of the time by ability. When the old values were fitted, grade 20 / 50 / 80 were .924 / .967 / .979 (2026-11-15 note).

Measured with `fly_misses 0 100 - <grade>` (seeds 0–99), routine flies caught. The first build is the one with the look's own geometry (§2); the rest are the final filter.

| Build | `TRACK_BIAS_FRAC` | `READ_RHO` | Grade 20 | Grade 50 | Grade 80 |
|---|---|---|---|---|---|
| main (`f9e43ed`) | 0.2 | 1.06 | | .967 (300 games) | |
| first build | 0.2 | 1.06 | | .986 | |
| first build | 0.5 | 1.06 | .961 | .973 | .980 |
| first build | 0.6 | 1.06 | .926 | .964 | .974 |
| first build | 1.0 | 1.06 | | .855 | |
| final | 0.2 | 1.06 | | .986 | |
| final | **0.6** | 1.06 | .971 | **.967** | .982 |
| final | 1.0 | 1.06 | | .946 | |
| final | 0.6 | 1.12 | .944 | | |
| final | **0.6** | **1.15** | **.915** | **.967** | **.984** |
| final | 0.6 | 1.25 | .815 | | .993 |

**`TRACK_BIAS_FRAC` 0.2 → 0.6.** Mechanism: an honest filter learns the persistent error as the geometry changes, so the same persistent error now costs fewer catches. The catch rate is gentle in it (.986 / .967 / .946 at 0.2 / 0.6 / 1.0). 0.6 puts grade 50 where it was. In plain terms, an average outfielder's fixed misjudgment of a ball is 0.6 × 3.5 m = 2.1 m across at 30 m (4°), and three times that in range. That is three times one look's fresh error, but now he can correct it as the ball comes in.

**`READ_RHO` 1.06 → 1.15** (`play.rs`, per 10 grades on the read's two σs; τ_read keeps 1.06). Mechanism: the ρ was fitted small (on the snapshot read, 2026-11-15, and kept by the tracking read) because the catch rate was steep in the read's error (ρ 1.4 had put grade 20 at .54). Both reads had that steepness for the same reason: neither could correct a persistent error. The snapshot read repeated its fixed draws on every read, and the tracking read took them as fresh and stopped listening. On the honest read, ρ 1.06 left grade 20 at .971, no worse than grade 50, and the band is *by ability*. With 1.15, grade 20's read σ is 1.52× grade 50's and grade 80's 0.66×: .915 / .967 / .984, the old fit's spread. Rostered players are all grade 50, so the standard run does not see this change.

## 4. The owner's play and the misses

**Play 15** (`BB_DEBUG_COMM=1 fly_misses 40580 1 list`; the same ball on both builds, landing near (−12, 119)):

- **Before:** "dive range, no dive." His landing-point error 2 / 1 / 0.5 s out was 5.4 / 2.5 / 2.3 m, and he came within 1.6 m of the ball.
- **After:** caught. The centre fielder holds his "Go" call all the way down, with his intercept steady at 6.11–6.17 s. Every fly of that game is caught (7 of 7).

**The 300-game routine set** (`fly_misses 0 300`, seeds 0–299, grade 50):

| | main `f9e43ed` | this branch |
|---|---|---|
| routine flies caught | .967 (1716 / 1775) | **.976** (1700 / 1742) |
| 25–35° / 35–50° / 50°+ | .964 / .966 / .977 | .974 / .975 / .986 |
| flies in the outfield that fall | .101 | .090 |
| lateral read sd, 2 s out, by bin (m) | 2.39 / 1.98 / 2.28 | 2.83 / 2.21 / 2.53 |
| lateral read sd, 1 s out | 0.97 / 1.02 / 1.36 | 0.79 / 0.67 / 0.76 |
| lateral read sd, 0.5 s out | 0.59 / 0.69 / 1.10 | **0.28 / 0.32 / 0.37** |
| depth read sd, 0.5 s out | 0.30 / 0.30 / 1.02 | 0.26 / 0.22 / 0.29 |

**The routine misses, by cause:**

| Cause | main | this branch |
|---|---|---|
| In dive range, no dive | 26 | **21** |
| Read good late, too far to make up | 19 | 10 |
| Reached, not caught | 10 | 8 |
| Left to a teammate, who missed | 4 | 2 |
| Late read error | 0 | 1 |
| **Total** | 59 | 42 |

Early in the flight the read is a little worse (the persistent part is three times as large), and late it is about twice as good: the far looks' misjudgment is corrected by the near ones. The 300-game rate is .976, against .967 on seeds 0–99 where it was fitted, and is still inside the band. Following the close-enough rule, it was not refitted to the third decimal.

## 5. Measurements (standard run)

300 games per set, seeds 0–299 [the `--seed 1000` set], main `f9e43ed` → this branch:

| | main | this branch | real MLB |
|---|---|---|---|
| runs / team-game | 4.13 [4.24] | **4.06 [4.19]** | 4.39 |
| K%, BB% | 22.0, 8.1 [22.0, 8.4] | 21.9, 8.2 [22.2, 8.4] | 22.58, 8.18 |
| HR / PA | 2.92% [2.78%] | 2.85% [2.73%] | 2.99% |
| reached on BIP | 35.3% [35.4%] | 35.4% [35.2%] | |
| GB hits | .310 [.316] | .309 [.313] | about .245 |
| LD hits | .668 [.664] | **.681 [.681]** | about .68 |
| FB hits | .127 [.112] | **.113 [.098]** | about .13 |
| PU hits | .010 [.010] | .007 [.002] | about .02 |
| PA / team-game, pitches / game | 38.3, 241 [38.7, 240] | 38.1, 239 [38.7, 242] | |
| rules warnings | none | none | |
| dives a game (`dive_probe 150 1000`), caught | 2.3, 45% | 2.3, 44% | |
| outfield dives in the air a game | 0.11 | 0.04 | |

**Separating the two changes** (the new filter at the old `TRACK_BIAS_FRAC` 0.2, seeds 0–299, by a temporary override): runs 3.87, LD .662, FB .104, PU .004. So the honest filter on its own takes fly hits from .127 to .104 and runs from 4.13 to 3.87. The refit's larger persistent part gives part of that back on flies (.113) and puts liners up (.662 → .681): a liner is in the air 1–3 s, too short for its misjudgment to be corrected.

## 6. What is still off

- **Fly-ball hits on the `--seed 1000` set** are .098 against about .13 real (.113 on seeds 0–299). Pop-up hits are .002–.007 against about .02.
- **The early read is a little worse.** Lateral sd 2 s out is 2.2–2.8 m, against 2.0–2.4 on main: the persistent part is three times as large, and early on nothing corrects it. Routine plays absorb it.
- **The ability spread is fitted on 100 games per grade.** At ρ 1.15, grade 20's read σ is 1.52× grade 50's. The rate is steep beyond that: ρ 1.25 had grade 20 at .815.
- **`a_poor_reader_misses_more_flies_than_a_good_one`** now plays batted balls from games 0–23 that land 18–28 m from the nearest outfielder's spot and are at least 3.5 s in the air: grade 20 catches 266 of 296, grade 80 280. On the old set (constructed balls with exactly the believed spin, 12–20 m, at least 4 s) both grades caught every ball, and on that geometry from games too, as they should on an easy fly.
- **`bb_ffi`'s timeline test** samples game 6, not 5: game 5 no longer has a dive.
- **Infield reads** use the same filter (a grounder before its first bounce, a liner, a pop-up). Grounder hits did not move; the infield was not examined separately.
