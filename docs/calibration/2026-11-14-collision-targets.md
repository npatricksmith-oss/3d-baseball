# Statcast targets for the collision work (2026-11-14)

Branch `work/collision-targets`, from `work/swing-recalib`. Python only; no sim code changed, so no ledger row. The owner has opened the collision question as a sourced physics change (Opus, `work/collision`); this note supplies the real-game numbers that change has to meet. Context: [recalibration report §3.1](2026-11-14-swing-recalibration.md), [bat angle at contact](2026-11-14-bat-angle-at-contact.md) (the sim's spray falls 28° from grounders to fly balls, MLB's 17–18°; nearly all of the offset comes from the collision).

**Summary of what the collision has to meet** (pull-positive degrees; twelve 2025 dates; numbers in §2):

1. **Spray falls with launch angle by about 17° from ground balls to fly balls** (Savant types: ground ball +14.7°, line drive +5.3, fly ball −2.6, popup −23.3; by band: 0–10° +11.8, 10–25° +5.9, 25–35° +0.3, 35–50° −5.0). The slope of spray on launch angle is −0.34° per degree over all balls in play (−0.46 over 0–50°); left-handed batters −0.40, right-handed −0.30. The sim's fall is 28°.
2. **The fall is the same at every pitch location and height**: slopes −0.34 to −0.42 across the five horizontal bins and −0.20 to −0.35 across the five height bins (the two outer ones, below and above the zone, are the flattest and the smallest: 794 and 350 balls). It is not a by-location effect; it is in the contact.
3. **The attack direction barely changes with launch angle** (mean −3.1° below 0°, +0.8° at 25–35°, +5.2° at 50°+: 8° over the whole range), so spray − AD falls almost as far as spray does: +13.9° at 0–10° to −7.7° at 35–50°, and −0.37° per degree of launch angle. Averaged over all non-popup balls in play spray − AD is +8.3° (SD 26.6°), the +7.2° (R/R) to +8.9° (all) the earlier notes quote.
4. **Within a band the spread is large and flat**: spray SD 24–27° at 0–50°, 29° below 0°, 49° for popups. A collision change should not shrink it.

## 1. Data

Baseball Savant pitch-level CSVs, regular season 2025, the twelve dates R13 used (Apr 8, Apr 22, May 6, May 20, Jun 3, Jun 17, Jul 1, Jul 22, Aug 5, Aug 19, Sep 2, Sep 16): 51,723 pitches, re-pulled 2026-11-14 into an untracked directory (`sim/target/data/savant/`, ignored by git; the earlier CSVs were in another session's scratch outside the worktree). Public data, a generic client name, no personal details.

```bash
mkdir -p sim/target/data/savant && cd sim/target/data/savant
for d in 2025-04-08 2025-04-22 2025-05-06 2025-05-20 2025-06-03 2025-06-17 2025-07-01 2025-07-22 2025-08-05 2025-08-19 2025-09-02 2025-09-16; do
  curl -sS -A "research-script/1.0" -o sc_$d.csv "https://baseballsavant.mlb.com/statcast_search/csv?all=true&hfGT=R%7C&hfSea=2025%7C&player_type=batter&game_date_gt=$d&game_date_lt=$d&type=details"; sleep 3; done
cd - && python3 -I analysis/statcast/savant_collision_targets.py sim/target/data/savant > out.md
```

Balls in play are rows whose description starts `hit_into_play` and that have hit coordinates and a launch angle: **9,165** (4,004 left-handed batters, 5,161 right-handed). Spray is `atan2(hc_x − 125.42, 198.27 − hc_y)` (the swing note's §8 formula), mirrored so that **positive = toward the pull side** for both hands. Horizontal location is inside-positive (−mirror × `plate_x`, feet from the center line); height is `(plate_z − sz_bot) / (sz_top − sz_bot)`, cut into thirds of the batter's own zone. Attack direction's CSV sign is negative = pulled, so pull-positive AD = −`attack_direction` (as in `savant_sprayad.py`).

**Caveats.**
- **Sampling:** a band mean's standard error is SD/√n, 0.5–1.0° for the bands with 1,000+ balls and 3–6° for cells under 100 (the location × launch-angle cells at mid-away, middle and mid-in have 66–217 balls each, so read those as ±2–4°). Twelve dates are about a tenth of a season. The earlier six-date figures (GB +15.3, LD +6.6, FB −2.8, PU −21.7) differ from these (+14.7, +5.3, −2.6, −23.3) by up to 1.3°, which is the size to expect.
- **Where `hc_x` and `hc_y` are measured.** Savant's documentation gives only "Hit coordinate X/Y of batted ball". I found no published definition of the point (landing, where fielded, where the play ended). The sim's spray in the earlier notes is by first landing, so a ground ball that rolls to a fielder off the line, or a fly ball that is caught, may differ between the two. Ground-ball spray is the likeliest to differ (bounces and rolls curve it toward the fielder).
- **Popups** (launch angle 50°+) have an SD of 48° and mean spray that depends on foul pops near the plate; I would not use them as a target beyond the sign.
- **Launch angle** is Statcast's measured launch angle at exit (`launch_angle`), the elevation of the exit velocity, as in the sim.
- Fair balls only (`hit_into_play`); foul-ball spray is not in Statcast's public data (R13).

## 2. Spray against launch angle, location and height

Each cell is n, mean pull-positive spray (°), SD (°). Sides pooled unless named; the left-handed rows are mirrored, so a pull is positive for both.

Files: 12 dates; balls in play with coordinates and launch angle: 9165 (R 5161, L 4004)

#### Pull-positive spray by launch-angle band, all (pull-positive, sides pooled)

| group | n | mean | SD |
|---|---|---|---|
| < 0 | 2898 | +15.7 | 29.4 |
| 0-10 | 1274 | +11.8 | 26.5 |
| 10-25 | 1972 | +5.9 | 26.0 |
| 25-35 | 1197 | +0.3 | 24.5 |
| 35-50 | 963 | -5.0 | 27.1 |
| 50+ | 861 | -19.8 | 48.6 |

Slope of spray on launch angle: -0.341 deg per deg (r -0.31, n 9165)
Same, launch angle 0-50 only: -0.455 deg per deg (r -0.22, n 5406)

#### Pull-positive spray by launch-angle band, right-handed

| group | n | mean | SD |
|---|---|---|---|
| < 0 | 1631 | +13.3 | 28.7 |
| 0-10 | 712 | +9.2 | 27.2 |
| 10-25 | 1126 | +5.3 | 26.3 |
| 25-35 | 621 | -0.5 | 24.2 |
| 35-50 | 566 | -4.7 | 27.2 |
| 50+ | 505 | -18.8 | 49.2 |

Slope of spray on launch angle: -0.295 deg per deg (r -0.27, n 5161)
Same, launch angle 0-50 only: -0.386 deg per deg (r -0.19, n 3025)

#### Pull-positive spray by launch-angle band, left-handed

| group | n | mean | SD |
|---|---|---|---|
| < 0 | 1267 | +18.8 | 29.8 |
| 0-10 | 562 | +15.1 | 25.2 |
| 10-25 | 846 | +6.8 | 25.4 |
| 25-35 | 576 | +1.2 | 24.9 |
| 35-50 | 397 | -5.5 | 27.1 |
| 50+ | 356 | -21.3 | 47.7 |

Slope of spray on launch angle: -0.404 deg per deg (r -0.36, n 4004)
Same, launch angle 0-50 only: -0.549 deg per deg (r -0.27, n 2381)

#### Pull-positive spray by Savant batted-ball type, sides pooled

| group | n | mean | SD |
|---|---|---|---|
| ground_ball | 4021 | +14.7 | 28.8 |
| line_drive | 2148 | +5.3 | 26.4 |
| fly_ball | 2418 | -2.6 | 26.2 |
| popup | 578 | -23.3 | 55.2 |

Fall from ground balls to fly balls: 17.3 deg (ground_ball +14.7, fly_ball -2.6); to popups 38.0

#### Pull-positive spray by horizontal location (ft from the plate center line, + inside), sides pooled

| group | n | mean | SD |
|---|---|---|---|
| outside | 3898 | +1.5 | 30.9 |
| mid-away | 745 | +5.5 | 30.7 |
| middle | 767 | +5.8 | 29.4 |
| mid-in | 708 | +8.4 | 30.3 |
| inside | 3047 | +10.0 | 33.2 |

#### Pull-positive spray by pitch-height third of the batter's zone, sides pooled

| group | n | mean | SD |
|---|---|---|---|
| below zone | 794 | +14.7 | 28.2 |
| bottom third | 2630 | +10.0 | 30.5 |
| middle third | 3587 | +4.3 | 31.0 |
| top third | 1804 | -0.9 | 33.5 |
| above zone | 350 | -3.7 | 35.5 |

#### Spray against launch angle by horizontal location: slope (deg per deg, r, n), sides pooled

| group | slope | r | n | mean spray |
|---|---|---|---|---|
| outside | -0.368 | -0.35 | 3898 | +1.5 |
| mid-away | -0.367 | -0.32 | 745 | +5.5 |
| middle | -0.419 | -0.40 | 767 | +5.8 |
| mid-in | -0.358 | -0.33 | 708 | +8.4 |
| inside | -0.342 | -0.30 | 3047 | +10.0 |

#### Mean pull-positive spray (n; SD) by horizontal location x launch-angle band, sides pooled

| | < 0 | 0-10 | 10-25 | 25-35 | 35-50 | 50+ |
|---|---|---|---|---|---|---|
| outside | +12.4 (1443; 30.2) | +5.3 (558; 26.8) | -1.6 (814; 25.5) | -5.5 (453; 23.3) | -11.7 (367; 24.9) | -26.8 (263; 44.5) |
| mid-away | +17.1 (217; 28.3) | +8.9 (118; 25.4) | +7.7 (147; 24.2) | -1.2 (116; 24.4) | -5.7 (81; 26.3) | -18.3 (66; 48.9) |
| middle | +18.5 (208; 24.5) | +14.6 (107; 23.8) | +7.8 (182; 25.3) | +0.3 (118; 24.4) | -8.1 (78; 26.7) | -23.9 (74; 38.6) |
| mid-in | +17.0 (189; 29.1) | +19.7 (107; 21.1) | +10.9 (163; 24.4) | +0.5 (110; 24.2) | -2.5 (72; 28.6) | -15.4 (67; 45.9) |
| inside | +20.0 (841; 28.6) | +19.3 (384; 25.8) | +13.0 (666; 24.9) | +7.3 (400; 24.3) | +2.0 (365; 27.5) | -15.4 (391; 52.5) |

#### Spray against launch angle by height third: slope (deg per deg, r, n), sides pooled

| group | slope | r | n | mean spray |
|---|---|---|---|---|
| below zone | -0.262 | -0.25 | 794 | +14.7 |
| bottom third | -0.295 | -0.27 | 2630 | +10.0 |
| middle third | -0.345 | -0.32 | 3587 | +4.3 |
| top third | -0.352 | -0.30 | 1804 | -0.9 |
| above zone | -0.201 | -0.17 | 350 | -3.7 |

#### Mean pull-positive spray (n; SD) by height third x launch-angle band, sides pooled

| | < 0 | 0-10 | 10-25 | 25-35 | 35-50 | 50+ |
|---|---|---|---|---|---|---|
| below zone | +20.2 (348; 26.7) | +22.2 (124; 22.7) | +11.2 (172; 27.2) | +6.2 (71; 21.1) | -5.0 (49; 25.8) | -6.8 (30; 48.3) |
| bottom third | +18.0 (994; 29.1) | +14.1 (403; 27.0) | +9.1 (567; 25.1) | +3.9 (288; 25.0) | -3.4 (213; 27.3) | -16.5 (165; 48.1) |
| middle third | +15.5 (1079; 29.3) | +10.7 (495; 26.2) | +3.7 (789; 26.4) | -1.3 (495; 24.5) | -5.7 (399; 27.0) | -19.9 (330; 43.9) |
| top third | +9.2 (414; 30.3) | +5.8 (217; 26.2) | +4.3 (381; 25.6) | -1.3 (287; 24.2) | -6.5 (242; 26.6) | -24.0 (263; 51.6) |
| above zone | -0.2 (63; 29.7) | +2.5 (35; 24.0) | +1.4 (63; 20.7) | -2.4 (56; 25.4) | -0.7 (60; 29.9) | -17.5 (73; 56.3) |

#### Spray minus attack direction (pull-positive, deg) by launch-angle band; popups left out as in savant_sprayad.py

| band | all: n | mean | SD | right-handed batters: n | mean | SD |
|---|---|---|---|---|---|---|
| < 0 | 2745 | +19.4 | 30.0 | 1553 | +17.3 | 27.2 |
| 0-10 | 1209 | +13.9 | 21.4 | 677 | +11.3 | 22.1 |
| 10-25 | 1880 | +6.2 | 20.8 | 1072 | +5.1 | 21.7 |
| 25-35 | 1154 | -0.6 | 18.4 | 594 | -1.4 | 16.5 |
| 35-50 | 904 | -7.7 | 22.7 | 532 | -7.7 | 21.9 |
| 50+ | 310 | -18.1 | 23.1 | 185 | -17.0 | 22.8 |

All non-popup balls in play with attack direction: n 8202, spray - AD mean +8.3 SD 26.6

#### Attack direction (pull-positive) by launch-angle band, same rows

| band | n | mean AD | SD |
|---|---|---|---|
| < 0 | 2745 | -3.1 | 19.8 |
| 0-10 | 1209 | -1.7 | 14.8 |
| 10-25 | 1880 | -0.5 | 13.7 |
| 25-35 | 1154 | +0.8 | 12.4 |
| 35-50 | 904 | +3.1 | 13.2 |
| 50+ | 310 | +5.2 | 9.2 |

Slope of (spray - AD) on launch angle: -0.374 deg per deg (r -0.36, n 8202)

## 3. Sidespin: what Statcast gives

**Statcast's public CSV has no batted-ball spin field.** The spin fields in the file are `spin_dir`, `spin_rate_deprecated` (both "Deprecated field from the old tracking system"; the latter "Replaced by release_spin"), `release_spin_rate` ("Spin rate of pitch tracked by Statcast"), and `spin_axis` ("The Spin Axis in the 2D X-Z plane in degrees from 0 to 360, such that 180 represents a pure backspin fastball"), all about the pitch ([csv-docs](https://baseballsavant.mlb.com/csv-docs)). The batted-ball fields are `launch_speed`, `launch_angle`, `hit_distance_sc`, `bb_type`, `hc_x`, `hc_y`, `launch_speed_angle` and the expected-stat columns; none is a spin. The 12-date CSV's header agrees (checked column by column). So sidespin on batted balls cannot be tabulated from the public file, and Statcast's own figure for it is not published there.

**What is published (Nathan).** Alan Nathan, "Why Does a Fly Ball Carry Better to Centerfield?" (preprint dated June 25, 2020, [PDF](https://baseball.physics.illinois.edu/carry-v2.pdf)), uses Statcast fly balls from Tropicana Field (no wind, constant air) tracked to at least 80% of their distance. Its footnote 2: Trackman "measures the total spin ω directly, it only infers the spin axis (and therefore ωb and ωs) from the trajectory, using a proprietary algorithm", so the sidespin is an inferred quantity, not a measured one. His figures, read from the plots (approximate, my reading), all for **exit speed near 100 mph, launch angle 25–30° and backspin 2,000–3,000 rpm**; the backspin window is a **selection**, not a distribution of batted backspin:
- Fig. 1 (distance against sidespin): sidespin spans about ±2,000 rpm for these balls; the parabolic fit falls from about 400 ft at zero sidespin to about 375 ft at ±2,000.
- Fig. 2 (sidespin against spray, batter-handedness-adjusted): for right-handed batters sidespin runs from about +1,500 at −30° (pulled) through zero near **−10°** (slightly pulled) to about **−1,500 to −2,500 rpm at +10° to +25° (opposite field)**; left-handed batters mirror. A slope of roughly −80 rpm per degree of spray (my reading, ±30%). Nathan's sign: positive sidespin breaks toward the left-field foul line.
- Text: sidespin is largest for opposite-field and pulled balls and zero near −10° because "the bat is not normally horizontal at impact but is usually tilted, with the barrel of the bat below the knob": a bat tilted downward but perpendicular to the pitch gives a slicing sidespin (§IV).
- His model run (100 mph, 27.5°, 2,500 rpm backspin): 1,500 rpm of sidespin cuts distance by 12 ft (about 8 ft per 1,000 rpm); about 40% of that from drag rising with total spin, 15% from the curved path, 45% from reduced lift (§III).
- Backspin is largest for the opposite field and smallest for the pull side (Fig. 4, selected EV 95–105 mph, by 5° launch-angle bucket); Nathan explains it with an attack angle that changes through the swing (§V). The figure's numbers are not in the text and I did not read them.

Also: Nathan, "Using Statcast Data To Study The Flight of a Baseball" (2016, [PDF](https://baseball.physics.illinois.edu/THT-Fly-Ball-Distances-Statcast.pdf)): model carry at 103 mph and 27°: 336 ft at 0 rpm backspin, 368 at 500, 386 at 1,000, 395 at 1,500, 400 at 2,000, 403 at 2,500 and 3,000. Carry saturates above about 1,500–2,500 rpm.

**Not found:** a published distribution of batted-ball backspin or sidespin by launch angle, or by launch angle and spray together, for all balls (not only the 2,000–3,000 rpm-backspin selection). The nearest comparator for the sim's sidespin is Fig. 2: about 2,000–2,500 rpm at +20° spray, against the sim's 3,720 at +19.6° (recalibration report §2.2).

## 4. How Statcast measures attack direction and attack angle

**An instant, not a window; the swing-path tilt is the window.** Quotes (the three MLB.com glossary and news pages returned HTTP 406 to my fetcher, so the MLB wording below is as quoted on Savant and on FanGraphs; the glossary pages `https://www.mlb.com/glossary/statcast/attack-angle` and `.../attack-direction` and the news piece `https://www.mlb.com/news/new-statcast-swing-metrics-2025` were not read directly):
- Savant csv-docs ([baseballsavant.mlb.com/csv-docs](https://baseballsavant.mlb.com/csv-docs)), glossary: attack angle is "The vertical angle at which the sweet spot of the bat is traveling at the point of impact with the ball"; attack direction "The horizontal angle at which the sweet spot of the bat is traveling at the point of impact with the ball", "expressed as an angle to the 'PULL' or 'OPPO' direction"; swing path tilt "The vertical angle of the arc traced by the swing path over the 40 ms prior to contact". Field entries: attack_angle "...as it impacts the ball, as compared to the ground"; attack_direction "...as compared to a line from home plate to straightaway center field"; swing_path_tilt "defined by the path of the bat in the 40 ms prior to contact, which approximates a slice of a flat disc in shape".
- FanGraphs, Davy Andrews, ["Test Driving Statcast's Newest Bat Tracking Metrics"](https://blogs.fangraphs.com/test-driving-statcasts-newest-bat-tracking-metrics/) (May 21, 2025): "As with all bat tracking metrics, the number used is from the video frame when the bat is closest to the ball."; attack angle "at impact (or when the bat is closest to the ball, in case of a whiff)"; swing path tilt "describes the angle that the head of the bat takes over the 40 milliseconds (or 0.04 seconds) prior to contact."
- FanGraphs, David Appelman, ["Statcast Bat Tracking Metrics Are Now on FanGraphs!"](https://blogs.fangraphs.com/instagraphs/statcast-bat-tracking-metrics-are-now-on-fangraphs/) (March 6, 2026), quoting the MLB glossary: attack direction "measures the horizontal direction that the sweet spot of the bat is moving at the point of contact (or the point the bat and ball cross paths on a swing-and-miss)"; bat speed "at the point of contact with the ball (or where the ball and bat would have met, in case of a swing-and-miss)".
- Baseball Prospectus, Stephen Sutton-Brown, ["Understanding Swing Processes Through Bat and Pitch Tracking"](https://www.baseballprospectus.com/news/article/103703/best-of-bp-2025-understanding-swing-processes-through-bat-and-pitch-tracking/) (Dec 26, 2025): intercept point is the "point at which their bat is nearest to the baseball on a swing (whether or not they make contact)".

So attack angle and direction are the velocity direction of the sweet spot **in the single video frame where the bat is nearest the ball** (the tracked frame, not an average over frames), which is what the sim reports (`foul_diag`: the sweet spot's velocity at the bat–ball intercept, closest approach for a whiff). Swing path tilt alone is a fit over the 40 ms before contact; the sim reads it from its plane normal (R13's "last 40 ms" definition). **Not found in any page I could read:** the tracking system's name, its frame rate, how many points on the bat are tracked, and how the velocity at a frame is derived from positions (a frame difference or a smoothed fit). One discrepancy: the MLB swing-path-tilt page says tilt is "the overall plane the bat follows over the course of the swing" while Savant says 40 ms. Tom Tango: no page I read carries his words on this; tangotiger.com appears only as a data source link (the launch-speed estimates for untracked balls) and in a Baseball Prospectus link to a comments thread titled "statcast-smoothness-of-a-swing-path" that I did not read. **No method beyond the above was found**; asking Savant or MLB for the frame rate and the derivative method is the way to close it.

## 5. Reproduction and files

`analysis/statcast/savant_collision_targets.py` (standard library; `python3 -I`), documented in `analysis/statcast/README.md`. The CSVs are not committed. The tables of §2 are the script's output, unedited except for heading levels.
