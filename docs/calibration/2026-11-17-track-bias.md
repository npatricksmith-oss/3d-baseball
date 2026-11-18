# The tracking read's persistent error, carried as a state (2026-11-17)

> Superseded figures: current numbers are in the [metrics ledger](../metrics-ledger.md) (§1).

Branch `work/track-bias`, from main `f9e43ed`. Spec: agent-models §5.1, "The persistent error as a state". Owner's saved play: seed 40580, play 15 (observations.md).

## 1. Diagnosis (from TASKS.md, 2026-11-16)

The play is a routine fly to centre: launch angle 38°, 5.9 s of hang time, and the centre fielder needs 50 ft of the 122 he can cover. He called it at 0.7 s and was there in time. But from 1 s to 0.6 s before it came down, with the ball 8 m from him, his tracking read stayed 2.2–2.5 m off *laterally*. He closed too late, stopped 1.6 m from the ball, did not dive, and the ball went to the wall for an inside-the-park home run.

An A/B with `look_noise()` found the cause. Each look's persistent part (`TRACK_BIAS_FRAC` times the play's fixed draws) went into the Kalman update as if it were fresh noise on every look. The filter's covariance shrank as if N looks averaged the persistent part down by √N, which a repeated error cannot do. So the filter grew sure of a lateral velocity worked out from the early, far looks, and the good close looks could not move it.

**Measured** (`the_tracking_read_reports_a_covariance_that_covers_its_error`): an average centre fielder 12 m short of a routine fly (38°, 41 m/s), over 120 plays of draws. Two seconds before landing his lateral error was 0.92 m RMS, while his own σ was 0.18 m. The mean of error² / σ² was **26.7**; an honest filter gives 1.

## 2. What was built

The filter now carries the persistent error as four more states, b: position along his line of sight, position across it, velocity along it, velocity across it, each in units of its own σ. That makes thirteen states with a dense covariance.

- **Prior:** b ~ N(0, I), independent of the ball. His first velocity look carries b too, so the starting velocity estimate, and the position flown back from the plate, start correlated with it.
- **Between looks:** b does not change.
- **Each look:** he expects to see h(r, v, b) = r + H_b(r)·b, plus fresh noise. H_b(r) is the two directions (along ×3, across level) times `TRACK_BIAS_FRAC`·σ0·d/30 m, the same scaling the world uses when it draws the error. Velocity works the same way, with vel_frac·|v| and the line of sight from r (`Look::expect`).
- **The update** is an extended Kalman update. h and the fresh covariance are evaluated at the estimate as it stands, and H is h's full Jacobian in r, v and b, by central differences. The velocity look is taken after the position look, so it is worked out from the corrected position (*after the Codex review's P2-1*, §2a). The world's draw still uses the ball's true geometry.
- **His doubt** (the landing σ that calls weigh) loses its extra persistent-error term. That term is now inside the covariance, so keeping it would count it twice.
- **Draws:** no new draws, and the draw order is unchanged.

**Why a state rather than a wider covariance per look.** In metres, the persistent part is a misjudged angle times the range, plus a misjudged fraction of the range. It shrinks as the ball comes in, and it turns as his line of sight turns. A filter that carries it as a state learns it from that changing geometry: the looks up close correct a misjudgment made off the bat, as a real outfielder's eye does. Widening each look's covariance by the persistent part would still treat it as independent from look to look, so N looks would still average it down by √N. That is less wrong, but not honest.

**Built first, and rejected: the geometry from the look itself.** At `TRACK_BIAS_FRAC` 0.2 it was honest: mean error² / σ² 0.98 / 1.43 / 1.29 at 2 / 1 / 0.6 s before landing. At 0.6 it was 2.4 again 2 s out, and his doubt 1 s out was half his miss (0.40 m against 0.78 m). A look is up to several metres off in range, so the filter misjudged the scale of his own persistent error by up to a fifth. That slip is itself persistent and unmodelled, and it grows as the square of the persistent part.

**The second build** (reviewed by Codex) linearised about his prediction, but with H = [I, H_b] taken at it. At `TRACK_BIAS_FRAC` 0.6 its mean error² / σ² was 0.92 / 1.21 / 1.16, and its doubt test had RMS doubt 1.06 / 0.41 m against an RMS miss of 0.83 / 0.39 m at 2 / 1 s.

### 2a. After the Codex review (P2-1): the full Jacobian

The persistent error's size depends on the range and its directions on the line of sight; the velocity term also depends on the speed. So once his estimate of b is not zero, ∂h/∂r is not I. For a range error of b0 = −2σ at 0.6 it is 0.58 I. The second build used I.

Codex's isolated probe (position known to 0.5 m, b0 = −2 known exactly, one look) gave mean error² / variance 2.4 across the line of sight, against 1.0 at zero bias.

**The fix.** H is now the Jacobian of h in r, v and b at the estimate as it stands. The velocity look's dependence on position (through the line of sight) and on speed is included. Each look's linearisation point is the estimate it updates.

**The test** is `a_look_with_a_learned_persistent_error_reports_a_covariance_that_covers_its_error`, isolated from flight:
- the ball's position is known to 0.5 m and its velocity to 1 m/s;
- b is known exactly and not zero;
- the ball is drawn from that belief, and a position look then a velocity look are drawn as the world draws them;
- three geometries (near and low, far and high, a liner to his side) and three biases, among them b0 = −2.

Over 2,000 draws, the mean error² / σ² along, across and up his line of sight, for position and velocity, is 0.92–1.11 in all 54 cells. With the identity Jacobian put back, the first case gives 1.39 / 2.36 / 2.38, and the test fails.

**In flight** (`the_tracking_read_reports_a_covariance_that_covers_its_error`): lateral error 0.50 / 0.29 / 0.12 m RMS at 2 / 1 / 0.6 s before landing, against his own σ of 0.55 / 0.28 / 0.11 m. The mean error² / σ² is 0.83 / 1.14 / 1.36. The doubt test (75 balls from games, real spins) has RMS doubt 1.05 / 0.41 m against an RMS miss of 0.77 / 0.32 m at 2 / 1 s.

*Not universal* (Codex's geometry probe, on the second build): a liner seen by the shortstop had one across-sight velocity component at 3.9 (also at bias 0.2), and very short grounders were conservative (0.1–1.0). These bound the claim. They are not shown to be regressions, and this change was not aimed at them.

## 3. Refit to the owner's band

The honest filter reads better, so the old values (`LOOK_SCALE` 0.2, `TRACK_BIAS_FRAC` 0.2) caught too many routine flies. The look scale is a property of the eye (1.3° per look at grade 50) and was left alone; the persistent part was refitted. Band: a routine fly caught 90–99% of the time by ability. When the old values were fitted, grade 20 / 50 / 80 were .924 / .967 / .979 (2026-11-15 note).

Measured with `fly_misses 0 100 - <grade>` (seeds 0–99), routine flies caught. The first build took its geometry from the look itself (§2). The second build is the one Codex reviewed, with H = [I, H_b]. The final build has the full Jacobian (§2a). The values were fitted on the second build. The final build was checked against the band and was not refitted (the lead's instruction: refit only if it leaves the band).

| Build | `TRACK_BIAS_FRAC` | `READ_RHO` | Grade 20 | Grade 50 | Grade 80 |
|---|---|---|---|---|---|
| main (`f9e43ed`) | 0.2 | 1.06 | | .967 (300 games) | |
| first build | 0.2 | 1.06 | | .986 | |
| first build | 0.5 | 1.06 | .961 | .973 | .980 |
| first build | 0.6 | 1.06 | .926 | .964 | .974 |
| first build | 1.0 | 1.06 | | .855 | |
| second | 0.2 | 1.06 | | .986 | |
| second | **0.6** | 1.06 | .971 | **.967** | .982 |
| second | 1.0 | 1.06 | | .946 | |
| second | 0.6 | 1.12 | .944 | | |
| second | **0.6** | **1.15** | **.915** | **.967** | **.984** |
| second | 0.6 | 1.25 | .815 | | .993 |
| **final** | **0.6** | **1.15** | **.934** | **.979** | **.986** |

**`TRACK_BIAS_FRAC` 0.2 → 0.6.** Mechanism: an honest filter learns the persistent error as the geometry changes, so the same persistent error now costs fewer catches. The catch rate is gentle in it (.986 / .967 / .946 at 0.2 / 0.6 / 1.0). 0.6 puts grade 50 where it was. In plain terms, an average outfielder's fixed misjudgment of a ball is 0.6 × 3.5 m = 2.1 m across at 30 m (4°), and three times that in range. That is three times one look's fresh error, but now he can correct it as the ball comes in.

**`READ_RHO` 1.06 → 1.15** (`play.rs`, per 10 grades on the read's two σs; τ_read keeps 1.06). Mechanism: the ρ was fitted small (on the snapshot read, 2026-11-15, and kept by the tracking read) because the catch rate was steep in the read's error (ρ 1.4 had put grade 20 at .54). Both reads had that steepness for the same reason: neither could correct a persistent error. The snapshot read repeated its fixed draws on every read, and the tracking read took them as fresh and stopped listening. On the honest read, ρ 1.06 left grade 20 at .971, no worse than grade 50, and the band is *by ability*. With 1.15, grade 20's read σ is 1.52× grade 50's and grade 80's 0.66×. On the second build that gave .915 / .967 / .984, the old fit's spread. On the final build it gives .934 / .979 / .986: in the band, a little higher, as the full Jacobian makes a learned bias count for what it is. Rostered players are all grade 50, so the standard run does not see this change.

## 4. The owner's play and the misses

**Play 15** (`BB_DEBUG_COMM=1 fly_misses 40580 1 list`; the same ball on both builds; `read_track 40580 15 8` has it land at (−13.2, 105.0) m, 6.31 s after contact):

- **Before:** "dive range, no dive." His landing-point error 2 / 1 / 0.5 s out was 5.4 / 2.5 / 2.3 m, and he came within 1.6 m of the ball.
- **After:** caught. The centre fielder holds his "Go" call all the way down, with his intercept steady at 6.11–6.17 s. Every fly of that game is caught (7 of 7).

**The 300-game routine set** (`fly_misses 0 300`, seeds 0–299, grade 50; the second build in the middle column, the final build on the right):

| | main `f9e43ed` | second build | final |
|---|---|---|---|
| routine flies caught | .967 (1716 / 1775) | .976 (1700 / 1742) | **.980** (1702 / 1736) |
| 25–35° / 35–50° / 50°+ | .964 / .966 / .977 | .974 / .975 / .986 | .976 / .979 / .995 |
| flies in the outfield that fall | .101 | .090 | .086 |
| lateral read sd, 2 s out, by bin (m) | 2.39 / 1.98 / 2.28 | 2.83 / 2.21 / 2.53 | 2.79 / 2.22 / 2.52 |
| lateral read sd, 1 s out | 0.97 / 1.02 / 1.36 | 0.79 / 0.67 / 0.76 | 0.77 / 0.67 / 0.75 |
| lateral read sd, 0.5 s out | 0.59 / 0.69 / 1.10 | 0.28 / 0.32 / 0.37 | **0.26 / 0.32 / 0.39** |
| depth read sd, 0.5 s out | 0.30 / 0.30 / 1.02 | 0.26 / 0.22 / 0.29 | 0.25 / 0.22 / 0.26 |

**The routine misses, by cause:**

| Cause | main | second build | final |
|---|---|---|---|
| In dive range, no dive | 26 | 21 | **14** |
| Read good late, too far to make up | 19 | 10 | 12 |
| Reached, not caught | 10 | 8 | 6 |
| Left to a teammate, who missed | 4 | 2 | 1 |
| Late read error | 0 | 1 | 1 |
| **Total** | 59 | 42 | **34** |

Early in the flight the read is a little worse (the persistent part is three times as large), and late it is about twice as good: the far looks' misjudgment is corrected by the near ones. The final 300-game rate is .980 (.979 on seeds 0–99, where the second build was fitted at .967). That is still inside the band, so it was not refitted. Play 15 is caught on the final build as well, with the same calls.

## 5. Measurements (standard run)

300 games per set: `--seed 0` (game seeds 0–299) [`--seed 1000`, game seeds 1,000,003,000–1,000,003,299, as `sim-cli` maps its seed argument], main `f9e43ed` → this branch:

| | main | second build | final | real MLB |
|---|---|---|---|---|
| runs / team-game | 4.13 [4.24] | 4.06 [4.19] | **4.14 [4.16]** | 4.39 |
| K%, BB% | 22.0, 8.1 [22.0, 8.4] | 21.9, 8.2 [22.2, 8.4] | 21.9, 8.3 [22.2, 8.5] | 22.58, 8.18 |
| HR / PA | 2.92% [2.78%] | 2.85% [2.73%] | 2.87% [2.77%] | 2.99% |
| reached on BIP | 35.3% [35.4%] | 35.4% [35.2%] | 35.2% [34.9%] | |
| GB hits | .310 [.316] | .309 [.313] | .308 [.308] | about .245 |
| LD hits | .668 [.664] | .681 [.681] | **.683 [.682]** | about .68 |
| FB hits | .127 [.112] | .113 [.098] | **.109 [.098]** | about .13 |
| PU hits | .010 [.010] | .007 [.002] | .006 [.004] | about .02 |
| PA / team-game, pitches / game | 38.3, 241 [38.7, 240] | 38.1, 239 [38.7, 242] | 38.4, 241 [38.7, 242] | |
| rules warnings | none | none | none | |
| dives a game (`dive_probe 150 1000`: game seeds 1000–1149, taken directly), caught | 2.3, 45% | 2.3, 44% | 2.3, 45% | |
| outfield dives in the air a game | 0.11 | 0.04 | 0.05 | |

**Separating the two changes.** The final filter at the old `TRACK_BIAS_FRAC` 0.2 (seeds 0–299, by a temporary override) gives runs 3.83, LD .656, FB .104, PU .004. On the second build it was 3.87, LD .662, FB .104. So the honest filter on its own takes fly hits from .127 to .104, and runs from 4.13 to 3.83. The refit's larger persistent part gives some of that back on flies (.109) and puts liner hits up (.656 → .683).

*Which liners* (Codex's matched probe on the second build, qualified after its review's P3-1): 369 fair 10–25° balls from games 0–35, four draw sets each, bases empty, perfect hands, bias 0.2 against 0.6:

| Flight time | Trials | Catches at 0.2 | Catches at 0.6 |
|---|---|---|---|
| 1–2 s | 272 | 61 | 61 |
| 2–3 s | 556 | 134 | 123 |
| 3 s or longer | 596 | 327 | 304 |

So the larger persistent part costs liner catches, but not mainly on the shortest. The 1–2 s liners are unchanged (they are mostly out of reach either way). The biggest loss is on liners up 3 s or longer: long runs where the early read sets the route, and the early read is now worse (§6). The first version of this note said liners were "too short for the misjudgment to be corrected"; that was a guess, and the probe does not support it as the whole story. The probe does not assign the whole aggregate liner movement to one mechanism.

## 6. What is still off

- **Fly-ball hits** are .109 [.098] against about .13 real. Pop-up hits are .006 [.004] against about .02.
- **The early read is a little worse.** Lateral sd 2 s out is 2.2–2.8 m, against 2.0–2.4 on main: the persistent part is three times as large, and early on nothing corrects it. Routine plays absorb it; long liners pay for some of it (§5).
- **The ability spread is fitted on 100 games per grade.** At ρ 1.15, grade 20's read σ is 1.52× grade 50's. The rate is steep beyond that: ρ 1.25 had grade 20 at .815 (second build). The final build sits higher (.934 / .979 / .986) and was left there, inside the band.
- **Coverage is shown for routine flies and for one look in isolation**, not for every trajectory or moving observer (Codex's geometry probe: a shortstop's liner velocity component at 3.9).
- **`a_poor_reader_misses_more_flies_than_a_good_one`** now plays batted balls from games 0–23 that land 18–28 m from the nearest outfielder's spot and are at least 3.5 s in the air: grade 20 catches 279 of 308, grade 80 292 (266 and 280 of 296 on the second build; the set is drawn from games, so it moves with them). On the old set (constructed balls with exactly the believed spin, 12–20 m, at least 4 s) both grades caught every ball, and on that geometry from games too, as they should on an easy fly.
- **`bb_ffi`'s timeline test** samples game 6, not 5: game 5 no longer has a dive.
- **Infield reads** use the same filter (a grounder before its first bounce, a liner, a pop-up). Grounder hits did not move; the infield was not examined separately.
