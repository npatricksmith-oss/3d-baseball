# Swing length against bat speed (2026-11-24, worker swing-length)

Question (owner, via spec §4.3, §6.3, §14.1): v3 gives every batter the same bat travel and the same swing time (0.150 s) while contact speed varies with rating. Do slower-bat hitters swing shorter, longer in time, or both? Measurement only; no sim or spec edit.

Data: the 2025 Savant sample of [9-step0-data.md](9-step0-data.md) §3 (26,489 rows, 12,163 with bat tracking, 11,185 competitive swings = each batter's fastest 90%), script `analysis/statcast/savant_swing_length.py` (stdlib; output kept in `savant_swing_length.out.txt`). Only the dates were sent to Savant. **The sample is thin per batter** (578 batters, median 17 competitive swings, p90 39, max 66), so batter-level figures use batters with ≥ 20 swings (255 batters; a ≥ 40 cut leaves 51). I did not pull the season leaderboard (see "Not done").

## Definitions (Savant, quoted in 9-step0-data §3)

- Swing length: "The total (sum) distance in feet traveled of the head of the bat in X/Y/Z space, from start of tracking data until impact point."
- Bat speed: "measured at the point six inches from the head of the bat." Intercept columns: distance between the bat/ball intercept point and the batter's center of mass, Y mound-to-plate.
- **Savant gives no swing time, and no speed through the swing** in these columns (nor a "time to contact" I could find; the csv has `bat_speed`, `swing_length`, `attack_angle`, `attack_direction`, `swing_path_tilt`, intercepts). The start of tracking is not defined. So swing time cannot be derived; only length/speed at contact, which is not a time (a bat that speeds up covers its length in more time than that).

## 1. Across batters (per-batter means, ≥ 20 swings, 255 batters)

Bat speed mean 71.75 mph, sd 3.01 (4.2%); swing length mean 7.31 ft, sd 0.45. Fit: length = 1.037 + 0.0874 ft/mph × speed (2.7 cm per mph), **r = 0.59**. Holding the batter's mean contact depth fixed: 0.085 ft/mph. At ≥ 40 swings (51 batters, speed sd only 2.5): 0.052 ft/mph, r = 0.37, so the slope is not tight (noise in each batter's mean, and a narrower range, pull it down or around; take 0.05–0.09).

| per-batter bat speed | batters | speed | length | mean depth | length / speed |
| :-- | --: | --: | --: | --: | --: |
| < 67 | 19 | 65.3 mph | 6.57 ft | 28.4 in | 69 ms |
| 67–69 | 23 | 68.2 | 7.01 | 28.2 | 70 |
| 69–71 | 49 | 70.1 | 7.26 | 30.8 | 70 |
| 71–73 | 74 | 71.9 | 7.38 | 30.6 | 70 |
| 73–75 | 59 | 73.8 | 7.43 | 31.5 | 69 |
| ≥ 75 | 31 | 76.5 | 7.64 | 31.2 | 68 |

Slower-bat batters have shorter swings, roughly in proportion: length ÷ contact speed is flat at 68–70 ms. Slow hitters also meet the ball slightly deeper (28 vs 31 in), which explains only a little (depth slope 0.01 ft/in).

## 2. Within a batter (8,269 swings of 255+ batters, deviations from each batter's own means)

Within-batter sd: speed 4.3 mph, length 0.75 ft, contact depth 10.5 in.

- Uncontrolled: 0.060 ft/mph, r = 0.34 (as percentages, length moves 0.60 × the speed change).
- **Depth explains it.** Length against depth: r = 0.84 (0.058 ft per inch; 0.8% of length per inch); speed against depth r = 0.29. With depth in the fit the speed slope falls to 0.020 ft/mph (%length = 0.21 × %speed).
- At a fixed depth (within ±2 in of his own mean; 1,207 swings): **0.015 ft/mph, r = 0.16** (standard error about 0.003). A swing 4 mph slower at the same depth is about 0.06 ft (0.7%) shorter. The band table agrees: speed dev −4.3 mph → −0.27 ft over all depths, −0.06 ft at fixed depth; −8.7 mph → −0.54 ft, −0.04 ft (28 swings).

So within a batter a slow swing is mostly a *deep-contact* swing (the bat is still short because it was met early) and, at the same depth, nearly the same length at lower speed.

## 3. What it means for the model

Per sd of bat speed across batters (3.0 mph, −4.2%), using the across-batter slope 0.087 ft/mph:

| batter | contact speed | bat travel | travel ÷ speed, against the average batter | swing time if length/speed held |
| :-- | --: | --: | --: | --: |
| 1 sd slow | 68.7 mph (−4.2%) | −0.26 ft (−3.6%) | 0.964 / 0.958 = 1.006 | 0.151 s |
| 2 sd slow | 65.7 mph (−8.4%) | −0.53 ft (−7.2%) | 0.928 / 0.916 = 1.013 | 0.152 s |

- **Swing length: both-and, but mainly shorter.** MLB's slower hitters travel shorter by nearly the same percentage that they are slower, and their length/speed is about constant. If that relation held in the game, travel would shrink about 3.6% per sd and the swing time would stay at about 0.150 s (+1% per sd). The data show **no sign that slow hitters take longer**, within the limits above (no real swing time; length/speed is only a proxy).
- **Overspeed (§14.1).** The table there arises because travel and time are fixed while speed falls. With travel scaled as above, the mid-swing speed needed relative to contact speed barely changes from the average batter's (×1.006 and ×1.013): the middle-pitch figure would stay near the average swing's 1.04 instead of 1.09 and 1.16, and the grade-50 low-pitch 1.15 would be the floor for everyone rather than 1.20/1.29. Rough arithmetic only: it treats the travel to cover in the last 82 ms as scaling with the whole swing's length, which the data cannot show (below). So **between-batter differences (`BatSpeed`, the 3.9% part) could be removed by shortening the travel**; the swing-to-swing effort part cannot: at a fixed depth a slower swing by the same batter is not shorter (0.015 ft/mph, 0.7% for a 4.5% slower swing), so its overspeed stays (≈1.09 middle for a 1 sd slow effort).
- **Depth** is the other lever: deep contact is shorter by 0.058 ft/in, 6.4 ft against 8.4 ft out front, and slower. A swing that meets the ball deeper should have the bat travel shortened by that, and this needs no relation to speed.

## What the data cannot tell

- The launch position, the start of "tracking", and the shape of the path between: length is one total, speed one value at contact. Whether the shorter travel is a later start (shorter backswing) or a smaller path in the last 82 ms is unknown, so the "÷ whole length" step above is an assumption.
- Any swing time or speed profile; the "time" column is length ÷ contact speed, not a duration.
- Causes: batters differ in strength, bat, height and approach; across-batter slope is a correlation. Per-batter means from ~20–60 swings carry noise (sd of a mean ≈ 0.9 mph vs 3.0 between batters, so the slope is attenuated only a little, but the ≥ 40 cut gives a lower slope).
- Savant's own position of bat speed (6 in from the head, not the sweet spot) and one season (2025, six dates).

## Not done

The season leaderboard (https://baseballsavant.mlb.com/leaderboard/bat-tracking) was not pulled; the sample's batter level is usable for the slope but thin. Pulling it would tighten §1 (and might publish swing-time-like columns if any; I found none in the csv docs).
