# Source 4: Fortenbaugh 2011, "The Biomechanics of the Baseball Swing" (Univ. of Miami PhD dissertation)

Fetched: https://hittingperformancelab.com/wp-content/uploads/2015/01/hitting-baseball-miami-study.pdf (249-page PDF, text extracted with pdftotext; stored in sim/target/research-dl/4/). Cited by section number (section numbers are the dissertation's own). Appendix B diagrams and Appendix E graphs were read later (2026-11-24, [9-step0-data.md](9-step0-data.md)); all values below come from the text and tables.

## What it is and how far to trust it
- Dissertation, David M. Fortenbaugh, University of Miami, May 2011; lab at the American Sports Medicine Institute (Birmingham, AL). (Abstract, 3.1)
- Subjects: 43 AA-level (Southern League) position players recruited; data for 33 usable. Age 25 +/- 2 y, height 184 +/- 6 cm, mass 92 +/- 9 kg. About 40 swings each, about 1300 trials. (3.1, 5.5)
- Method: 8 Eagle cameras plus 2 force plates, both at 300 Hz; markers on body, bat and ball; markers low-pass filtered at 12 Hz; force data unfiltered. Live batting-practice pitcher at about 13 m, in an L-screen, throwing about 4 fastballs to 1 changeup. Fastball 25.0 +/- 0.6 m/s (about 56 mph), changeup 21.3 +/- 0.6 m/s; flight time about 520 ms. Players wore indoor turf shoes, a lab hat, and swung their own wood bats. (3.2, 3.3, 5.1)
- The "typical batter" description (4.1) is the average of successful swings against fastballs "down the middle" only. Success means a measurable ball exit velocity (foul tips and misses were failures). (3.4, 3.5)
- Trust: high for timing and sequence in live-pitch swings at the professional level; the best single source of a full phase by phase composite. Limits: indoor lab, batting practice rather than game pitching, self-selected bats, a small number of players on one level (AA), a single lab. It reports no foot spacing or stance width, no hand position, no bat speed in m/s and no attack angle (see "Not covered"). Angles are in the study's own conventions (3.4), so check signs before use.

## Events and phases (timing relative to bat-ball contact, BC = 0; 3.4, 4.1)
Five events (force-plate based):
- Lead foot off: lead foot vertical force (GRFz) falls below 10% body weight (BW). Mean -621 ms.
- Lead foot down: GRFz rises back above 10% BW. About -340 ms.
- Weight shift commitment: lead foot GRFz passes 50% BW. About -170 ms.
- Maximum lead foot GRFz. -93 ms (130% BW).
- BC: instant the pitch leaves its trajectory.

Six phases: stance (from -1000 ms to lead foot off), stride (to lead foot down), coiling (to 50% BW), swing initiation (to max lead GRFz), swing acceleration (to BC), follow-through (BC to +200 ms; the author assumes the batter turns into a runner after that).
Because lead foot off is at -621 ms and the ball takes about 520 ms, the stance and the start of the stride happen before the pitcher releases (5.1; the author calls this "likely").

## Stance (4.1.1; -1000 to -621 ms)
- Upper body almost still (5.1). Weight drifts back: lead foot GRFz falls from 37 to 17% BW, trail foot GRFz rises from 57 to 86% BW. (Study notes the lead value is misleading from averaging.)
- Lead knee flexes 36 to 48 deg, trail knee 51 to a max of 56 deg (180 = fully flexed, 0 = straight per 3.4). Pelvis drifts back about 6 cm (0.06 m).
- Foot spacing, stance width, trunk tilt at setup, hand height, bat angle at setup: not given. Lead foot angle to X axis is 69 deg at the start of the stride.
- Head: 0 deg = looking at the pitcher, 90 deg = looking at home plate (3.4); head rotation is 28 deg at the start of coiling and 47 deg at BC.

## Load, stride and foot plant (4.1.2, 4.1.3, 5.1)
- Stride is a "controlled fall or drift" forward, not a violent push: trail foot GRFx holds about 21% BW; trail GRFz peaks 92% BW near -500 ms, then falls to 83%. (5.1)
- Stride length 42% of body height (study gives only the percentage, no metres), lead foot about 11 cm (0.11 m) closed. Lead foot angle opens 69 to 63 deg during the stride and to 56 deg during coiling.
- Lead knee peaks at 52 deg flexion at about -483 ms. Trail knee extends 56 to 50 deg during the stride, to 44 deg (max extension) at about -220 ms.
- Pelvis moves forward 7 cm during the stride and a further 11 cm during coiling, 5 cm more during initiation. Pelvis forward travel by phase: 7, 11, 5 cm (the study does not give a total).
- Countermovements (coil): pelvis transverse rotation from -10 to -18 deg, max counter-rotation -21 deg at about -250 ms; upper trunk -9 to -16 deg, max -22 deg at -186 ms; pelvis posterior tilt -15 to -26 deg; trail shoulder azimuth min -84 deg at -200 ms; lead shoulder -30 deg; bat to a minimum azimuth of -232 deg at -243 ms (bat wrapped behind head; bat elevation max 44 deg at end of stride). Lead elbow 89 to 72 deg flexion.
- Foot plant is soft: lead foot GRFx and GRFz stay very low right after lead foot down and rise towards the end of coiling, with a faster rise from about -200 ms ("stepping on thin ice" cue). (5.1)
- Weight transfer: during coiling trail GRFz falls 83 to 47% BW while the lead takes over (50% BW at -170 ms). Lead braking forces rise (GRFx 2 to 11% BW, GRFy 1 to 23% BW); trail GRFy propulsive 0 to 20% BW. Lateral stabilising forces (lead and trail GRFy) reach over 80% of their final peak during coiling.
- Hip-shoulder separation (upper trunk minus pelvis): only 2 deg to -9 deg during coiling, peaking at 18 deg at -103 ms, much smaller than in pitching, golf or discus; other batting studies cited report 6-15 deg. (4.1.3, 5.1)

## Rotation and swing (4.1.4, 4.1.5, 5.1)
Swing initiation (-170 to -93 ms):
- Lead foot GRFz rises 50 to 130% BW; lead foot GRFx peak 50% BW at -83 ms; GRFy peak 28% BW at about -123 ms; trail GRFy peak 24% at -127 ms; trail GRFz falls 47 to 34% BW.
- Lead knee extends 48 to 40 deg; trail knee flexes 47 to 54 deg. Pelvis rotates -13 to 20 deg, reaching 558 deg/s (over 95% of its eventual peak); upper trunk lags (-22 to 2 deg), reaching 610 deg/s.
- Trail elbow stays flexed (128 to 126 deg), trail upper arm drops and moves forward (shoulder elevation 71 to 49 deg), lead elbow 72 to 68 deg. Hands stay close to the body ("casting" is the fault).
- Bat unwinds -225 to -175 deg azimuth, velocity 236 to 1247 deg/s; bat lag 121 to 131 deg. Head tilts down and toward trail shoulder.
Swing acceleration (-93 ms to BC):
- Peak order: pelvis rotation velocity 581 deg/s at -70 ms (slows to 287 deg/s at BC); upper trunk 766 deg/s at -57 ms (430 deg/s at BC); lead shoulder azimuth velocity 342 deg/s at -43 ms; lead knee extension 263 deg/s at -40 ms; trail elbow extension has a first local peak 476 deg/s at -47 ms and keeps rising to 752 deg/s at BC (overall max 868 deg/s at +17 ms); bat azimuth velocity max 2435 deg/s at -10 ms. So proximal-to-distal order: pelvis, trunk, lead arm, trail elbow, bat; pelvis to trunk gap is about 13 ms, pelvis to bat about 60 ms.
- Pelvis rotates 19 to 77 deg (about 60 deg) to face roughly the pitcher; trunk global 2 to 75 deg; pelvis obliquity 14 to 27 deg; pelvis anterior tilt -20 to 8 deg and rises 5 cm. Separation closes from -18 to -2 deg. Lead knee extends 40 to 18 deg.
- Bat: elevation falls 39 to -30 deg (barrel below hands at BC); bat lag max 133 deg at -80 ms, down to 66 deg at BC; bat sweeps about 170 deg (-175 to -7 deg azimuth). The author attributes the bat speed to keeping the hands close to the body (angular momentum), with two distinct bat accelerations (about 1250 deg/s in initiation, then to 2435).
- Ground forces dissipate in this phase (lead GRFz 130 to 64% BW; lead GRFx -50 to -13% BW).
- Head: rotates 41 to a max 49 deg at -33 ms then back toward the pitcher (47 deg at BC), tilting toward the trail shoulder (-15 to -31 deg).

## Contact (4.1.5, 4.1.6, Table 1 for MIDDLE)
- At BC (middle fastball): lead knee about 18 deg flexion (almost straight; prior studies 11-20 deg); pelvis rotation 77 +/- 6 deg; upper trunk relative to pelvis -2 +/- 4 deg; lead shoulder elevation 70 deg, azimuth -1 deg; trail elbow flexion 81 +/- 8 deg (still bent; trail elbow keeps extending after contact); lead elbow 56 deg; bat lag 65 +/- 10 deg; bat elevation -30 +/- 5 deg; bat azimuth -8 +/- 12 deg; head rotation 47 +/- 5 deg. Upper trunk global 75 deg (more open than the 48-70 deg of other studies).
- Contact position of the ball relative to body/plate: not covered (ball tracked, but no location relative to body is reported in the text read).
- Bat speed in m/s and attack angle: not given. Ball exit velocity (BEV) is regressed but a mean BEV for middle fastballs was not extracted by me.

## Pitch location effects at BC (4.2.1, Tables 1-3; mean values)
| Quantity | HIGH IN | HIGH OUT | LOW IN | LOW OUT | MIDDLE |
|---|---|---|---|---|---|
| Pelvis rotation at BC (deg) | 79 | 69 | 76 | 61 | 77 |
| Upper trunk rel. pelvis (deg) | 1 | 0 | -5 | -8 | -2 |
| Trail elbow flexion (deg) | 89 | 77 | 77 | 66 | 81 |
| Bat lag (deg) | 79 | 63 | 58 | 51 | 65 |
| Bat elevation (deg) | -24 | -21 | -39 | -33 | -30 |
| Head rotation (deg) | 38 | 48 | 48 | 55 | 47 |
| Peak pelvis velocity (deg/s) | 645 | 568 | 587 | 506 | 601 |
| Peak trunk velocity (deg/s) | 837 | 784 | 760 | 699 | 813 |
| Peak bat azimuth velocity (deg/s) | 2430 | 2259 | 2786 | 2370 | 2486 |
| Time of peak pelvis velocity (ms) | -64 | -75 | -73 | -82 | -72 |
- Inside pitches: pelvis and bat more open at BC, larger peak pelvis, lead knee, lead shoulder and bat velocities, peak lead GRFz larger (143-146 vs 138% BW). Outside pitches: pelvis less rotated, trail elbow extension velocity greater. Lower body (GRF, lead knee) was "fairly consistent for all trials" (5.5).
- High pitches: more trunk rotation relative to pelvis and a higher bat; low pitches: bat lower and arm velocities peak earlier.

## Pitch type (changeup vs fastball; 4.3, 5.5)
- Batters begin the lower-body chain early because they anticipate a fastball: a progressive delay in event timing from fastball success to changeup success to changeup failure; lead foot peak GRF times are earliest relative to BC for failed changeup swings. Fast-success swings used more lower body and trunk; changeup swings used more upper body, notably trail elbow extension, with greater peak bat azimuth velocity when the swing failed. Early phases (stance, stride, coiling) were similar across conditions. Peak GRF magnitudes were larger for fast-success.

## Follow-through (4.1.6; BC to +200 ms)
- Bat keeps rotating (azimuth -7 to 239 deg), decelerating 2233 to 827 deg/s; bat elevation rises -30 to 28 deg by +150 ms; bat lag min 26 deg at +67 ms, 68 deg at end.
- Lead knee reaches max extension 11 deg at +73 ms, then flexes. Lead foot opens 49 to 36 deg by +60 ms then further. Pelvis keeps turning, 77 to 95 deg; trunk global 75 to 135 deg; trunk flexion max 16 deg at +30 ms, then 7 deg.
- Lead elbow max extension (42 deg flexion) at +43 ms; trail elbow max extension velocity 868 deg/s at +17 ms, minimum angle 37 deg at +83 ms. Trail shoulder rises (37 to 77 deg elevation); lead shoulder second adduction thrust, a "batter's shoulder" injury risk.
- Head returns toward the field (rotation 47 to 11 deg).
- Hands: study analyses a two-handed grip throughout, no one-hand finish or wrist roll reported. Balance and leaving the box: not covered (the study stops at +200 ms).
- Lead GRFz second local maximum 68% BW at about +110 ms; trail GRFz returns to 41% BW.

## Numbers (SI beside original; 1 deg = 0.01745 rad)
| Quantity | Value | SI | Source |
|---|---|---|---|
| Lead foot off | -621 ms | -0.621 s | 4.1.1 |
| Lead foot down | about -340 ms | -0.34 s | 4.1.2 |
| 50% BW weight commitment | about -170 ms | -0.17 s | 4.1.3 |
| Max lead GRFz | 130% BW at -93 ms (143-149% BW middle, Table 5) | -0.093 s | 4.1.4 |
| Stride length | 42% of height, 11 cm closed | 0.11 m | 4.1.2 |
| Pelvis forward travel | 6 cm back, then 7+11+5 cm forward | m | 4.1 |
| Peak pelvis velocity | 581 deg/s at -70 ms | 10.1 rad/s | 4.1.5 |
| Peak trunk velocity | 766 deg/s at -57 ms | 13.4 rad/s | 4.1.5 |
| Peak lead shoulder azimuth vel. | 342 deg/s at -43 ms | 6.0 rad/s | 4.1.5 |
| Peak lead knee extension vel. | 263 deg/s at -40 ms | 4.6 rad/s | 4.1.5 |
| Peak bat azimuth velocity | 2435 deg/s at -10 ms | 42.5 rad/s | 4.1.5 |
| Peak trail elbow ext. velocity | 868 deg/s at +17 ms | 15.1 rad/s | 4.1.6 |
| Max hip-shoulder separation | 18 deg at -103 ms | 0.31 rad | 4.1.4 |
| Pelvis at BC | 77 deg | 1.34 rad | 4.1.5 |
| Lead knee at BC | 18 deg flexion | 0.31 rad | 4.1.5 |
| Bat lag at BC | 66 deg (65 middle) | 1.15 rad | 4.1.5 |
| Bat elevation at BC | -30 deg | -0.52 rad | 4.1.5 |
| Pitch speed | 25.0 m/s fastball, 21.3 m/s changeup | same | 3.3 |
Standard deviations for the middle-fastball values are in Tables 1-3 for the location table; the phase description in 4.1 gives means only (graph standard deviation bands are in Appendix E, not read).

## Disagreements with common coaching or expectation
- Hip-shoulder separation is small (peak 18 deg), far less than in pitching or golf. (5.1)
- Counter-rotation and coil happen late and smaller in live pitching than in tee studies (pelvis -21 deg vs -28 deg tee; trunk -22 vs -52 deg). Foot plant is much earlier (-340 ms) than in tee studies (-240 to -175 ms). (5.1)
- The stride is a drift, not a push; the front foot lands soft and loads from about -200 ms; pelvis moves forward in every phase from stride on, so linear and rotational movements both matter (Lau and Williams ideas both supported). (5.1)
- The trail elbow is still bent (about 81 deg) at contact and extends after it. The bat barrel is below the hands at contact. (4.1.5)
- Lower body timing is committed before the pitcher releases; batters expect a fastball and are fooled by changeups by firing early. (5.1, 5.5)

## Quotes
- "the stride appears to be a rather 'controlled fall' or 'drift' forward rather than a violent, aggressive push" (5.1)

## Not covered
Foot width and angle in the stance, weight split at setup beyond force percentages, trunk tilt at setup, hand and bat setup, bat speed in m/s, attack angle, on-plane time, sweet-spot or ball location relative to plate and body, one-hand release, wrist roll, balance and leaving the box. Figures and Appendix E graphs not examined.
