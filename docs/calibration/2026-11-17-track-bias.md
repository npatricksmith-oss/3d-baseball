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
- **Geometry:** the filter's distance and directions now come from the look itself, not from the ball's true position. The world's draw still uses the true geometry.
- **His doubt** (the landing σ that calls weigh) loses its extra persistent-error term. That term is now inside the covariance, so keeping it would count it twice.
- **Draws:** no new draws, and the draw order is unchanged.

**Why a state rather than a wider covariance per look.** In metres, the persistent part is a misjudged angle times the range, plus a misjudged fraction of the range. It shrinks as the ball comes in, and it turns as his line of sight turns. A filter that carries it as a state learns it from that changing geometry: the looks up close correct a misjudgment made off the bat, as a real outfielder's eye does. Widening each look's covariance by the persistent part would still treat it as independent from look to look, so N looks would still average it down by √N. That is less wrong, but not honest.

**After the change** (same test): lateral error 0.43 / 0.30 / 0.14 m RMS at 2 / 1 / 0.6 s before landing, against his own σ of 0.44 / 0.26 / 0.13 m. The mean error² / σ² is 0.98 / 1.43 / 1.29.

## 3. Refit to the owner's band

The honest filter reads better, so the old values (`LOOK_SCALE` 0.2, `TRACK_BIAS_FRAC` 0.2) caught too many routine flies. The look scale is a property of the eye (1.3° per look at grade 50) and was left alone. The persistent part was refitted. Band: a routine fly caught 90–99% of the time by ability, and grade 50 was .967 on seeds 0–99 when the old values were fitted. Measured with `fly_misses <first> 100 - <grade>` on seeds 0–99:

FIT_TABLE

## 4. The owner's play and the misses

PLAY15

## 5. Measurements (standard run)

MEAS

## 6. What is still off

OPEN
