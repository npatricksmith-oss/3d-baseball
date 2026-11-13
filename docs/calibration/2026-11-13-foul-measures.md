# Foul share: Statcast-style measures and the bat-angle counterfactual (2026-11-13)

TASKS "Next" item 2 (O1/R13), done as the owner directed: measure first, change no calibration. This note changes no simulation code or values. The sim gives 23.7% of contact foul (B1.15 target 50%, band 40–60%; MLB 51.2% with foul tips, [R13](../research/r13-foul-direction.md) F1).

Tools:
- `cargo run --release -p bb_game --example foul_diag -- 3000 [timing ms] [bat-control cm] [K_perception] [bat angle °]` prints R13's tables F1–F6 from the sim's swings, with the MLB value beside each. It uses an average RHP against an average RHB, seed 1, `Knobs::calibrated_m3()`. The last argument is the counterfactual of §4; `-` keeps an earlier argument's default.
- `cargo run --release -p bb_game --example foul_in_games -- 128` gives the foul share by launch angle in full games, where the rules engine judges each ball from the live play. Caught fouls count as fouls, so its MLB total is 52.5% (R13 F1). Play timeouts, which force a fair decision, are left out.
- `sim-cli cage-map --type FF` gains a "Bat-angle counterfactual" section.
- `SwingTemplate::bat_angle` / `with_bat_angle` (bb_agents) give the counterfactual geometry: the pivot swung about an aim point at the same distance.

Definitions follow R13: attack direction (AD) is pull-positive; for a whiff the intercept is the instant of closest approach; check swings are left out (4,357 competitive swings in 3,000 PAs).

## 1. Well-timed steep contact passes; the larger deficit is low-angle contact

R13's leading hypothesis was that the sim's well-timed steep contact stays fair. It does not. P(foul | contact):

| LA (°) | sim, all | MLB, all | sim, \|AD\| < 6° | MLB, \|AD\| < 6° |
|---|---|---|---|---|
| below −30 | 11 | 57 | 7 | 62 |
| −30 to −10 | 5 | 41 | 1 | 20 |
| −10 to 10 | 3 | 22 | 2 | 5 |
| 10 to 25 | 11 | 37 | 4 | 20 |
| 25 to 40 | 30 | 47 | 16 | 24 |
| 40 to 50 | 46 | 66 | 44 | 46 |
| 50 to 60 | 65 | 74 | 71 | 69 |
| 60 to 70 | 75 | 78 | 86 | 81 |
| 70 and up | 86 | 87 | 90 | 88 |

- **Well-timed steep contact matches MLB.** Above 40° with |AD| < 6°, 63.7% of contact goes foul against a bin-weighted MLB 62.9% (Codex review probe).
- **Over all contact, 40–60° still falls short:** 46% against 66% at 40–50°, and 65% against 74% at 50–60°. Above 40°, 60.5% of contact goes foul against a bin-weighted MLB 72.4%. The shortfall there comes from mistimed contact.
- **The largest deficit is low contact:** grounders, chopped balls and liners. Below 25° the sim fouls 3–11% against MLB's 22–57%. In MLB a fifth of fouls leave below −10°; in the sim it is 8%. Sim fouls are steeper than MLB's: LA p50 46° against 30°, and 41% of them leave above 50° against 26%.

Full games give the same answer (`foul_in_games`, 128 games, 9,333 contacts, no play timeouts): 22.9% foul under the rules engine's §5.2 judgment, against 22.6% from `pa.rs`'s call where the ball first comes down. By bin the two agree within 3.4 points. The deficits are in the contact itself, not in how fair or foul is called.

## 2. Where in the timing window the fouls are missing

| AD bin (°) | sim share of swings | MLB | sim whiff/swing | MLB | sim foul/contact | MLB |
|---|---|---|---|---|---|---|
| < −20 (late) | 0.8 | 13.7 | 60 | 33 | 79 | 76 |
| −20 to −12 | 4.5 | 12.3 | 30 | 18 | 63 | 57 |
| −12 to −6 | 10.6 | 11.7 | 23 | 13 | 43 | 49 |
| −6 to 0 | 18.2 | 12.8 | 15 | 11 | 27 | 43 |
| 0 to 6 | 20.2 | 12.4 | 17 | 12 | 14 | 39 |
| 6 to 12 | 13.9 | 11.0 | 21 | 14 | 12 | 42 |
| 12 to 20 | 12.9 | 11.2 | 43 | 23 | 8 | 54 |
| 20 to 30 | 8.2 | 8.5 | 74 | 40 | 30 | 77 |
| > 30 (early) | 10.8 | 6.2 | 98 | 74 | 60 | 86 |

Three things differ from MLB:
- **Late contact is at the right foul rate but rare.** It is 5% of swings beyond −12°, against 26% in MLB.
- **Early contact almost never goes foul, and early swings mostly miss.** At AD 12–20° the sim fouls 8% of contact against 54%. Early fouls should be hooked or topped grounders (R13 F4, LA p50 0°). The sim has the right kind (its 75 early fouls have LA p50 7°, and 41% are below −10°) but few of them. Only 34% of its fouls have a pull-side AD (MLB 45%).
- **Well-timed contact fouls half as often** (|AD| < 6°: 20% against 40%), and the shortfall is the low-angle bins of §1.

**Why early swings miss: the bat passes over the ball.** Each whiff is judged against the bat's axis extended past the tip. The world's closest-approach station is clamped to the bat, so "closest at the tip" alone does not say that a longer bat would have hit; the first version of this note made that mistake (Codex review #2).

| AD bin (°) | whiffs | swung under | swung over | longer barrel hits | window edge | clearance p50 |
|---|---|---|---|---|---|---|
| < −12 (late) | 80 | 98% | 0% | 0% | 3% | 3.5 cm |
| −12 to 12 | 497 | 65% | 35% | 1% | 0% | 2.4 cm |
| 12 to 30 | 504 | 3% | **91%** | 6% | 0% | 11.0 cm |
| > 30 (early) | 461 | 0% | 67% | 2% | 31% | 18.7 cm |

Timing error becomes height error. Early bats pass over the ball and late bats under it, by more the further off the timing is. Only 6% of the early whiffs would have hit with a longer barrel. The search window (`contact.rs` `WINDOW_S` ±25 ms) clips only far misses: contact always happens within −9 to +12 ms of nominal (p1/p99), and the review's search of ±50 ms recovered no contact.

In the cage map the same thing shows on a perfectly perceived, square swing. 30 ms early makes contact at −38° LA, and each 5 ms early lowers the launch by 5–9° more (5° on time, −20° at 20 ms early). In MLB, early contact is still met (whiff 23–40% at AD 12–30° against the sim's 43–74%) and fouled (54–77%) as hooked or topped grounders (R13 F4). The sim's bat leaves the ball's height faster on either side of on-time than MLB's. Why is not measured here. Candidates: how the swing plane lines up with the pitch's path, and the shape of v1's swing (a radial bat about a fixed pivot, agent-models §3.6's "zero-lag special case") against the spec's hand path. A timing sweep with the pitch and aim held fixed, comparing swing shapes, would tell them apart.

## 3. The swing against Statcast

| Measure | sim | MLB (R13 F6) |
|---|---|---|
| Bat speed, mph (mean / p10 / p50 / p90) | 72.8 / 61.5 / 71.7 / 86.7 | 69.9 / 62.3 / 71.2 / 77.5 |
| Attack angle, ° (mean / p10 / p90) | 12.1 / 6.3 / 18.9 | 9.0 / −5 / 23 |
| Attack direction, ° (p10 / p50 / p90) | −8.7 / 4.7 / 31.1 | −25 / ≈0 / 23 |
| Swing path tilt, ° (mean / p10 / p90) | 23.7 / 14.3 / 33.4 | 32 / 23 / 41 |
| Swing length, ft (mean / p10 / p90) | 9.5 / 7.9 / 11.2 | 7.2 / 6.0 / 8.4 |
| Vertical bat angle, ° (mean / p10 / p90) | −18.5 / −30.5 / −6.6 | −25 to −35 (Blast, vendor) |
| AD against intercept depth, ° per inch out front | +1.04 (corr 0.94) | +1.48 (corr 0.87) |
| AA against AD, ° per ° toward pull | +0.26 (corr 0.83) | +0.41 (corr 0.71) |
| AD SD of fair contact, ° (pulled / opposite, by where the ball comes down) | 6.9 / 6.8 | 14 / 19 |

- **Bat angle and swing-plane tilt** steepen for low pitches as in MLB, but are 7–10° too flat at every height. Median vertical bat angle runs from −25° in the bottom third to −12° in the top third. Tilt runs 28° / 23° / 17° by third, against MLB's 37° / 32° / 25°.
- **The swing is about 30% too long.** Over the last 150 ms the sweet spot travels 9.5 ft against 7.2. The speed bump is a symmetric Gaussian, so the barrel reaches speed early. A real swing whips: the lag releases late (§3.6's λ), and the barrel gains most of its speed in the last few tens of ms. The long swing does not show in bat speed at contact, which matches. The p90 of 86.7 mph comes from outside pitches met at a longer radius (speed scales with r).
- **Attack direction spreads half as much.** Fair contact spans AD ±7° against MLB's ±14–19°. The pooled regression of AD on depth gives 70% of MLB's slope. That regression mixes pitch locations, aims and radii, so it is not the derivative along one arc. R13 §5 derived 1.45° per inch for a 1 m arc. Why the pooled value is lower is not established: every intercept is at ≥ 94% of peak angular speed, so slower contact away from peak does not explain it (Codex review #4). Fair pulled and opposite are split by where the ball first comes down. R13 used Statcast's hit coordinates; splitting by launch direction moves 88 of 2,160 balls.
- **Foul share by pitch height is inverted.** By height the sim gives 16% below the zone, 30% / 23% / 20% by third (bottom to top), and 19% above. MLB gives 52% / 47% / 47% / 58% / 69%. High pitches, where MLB's fouls concentrate, are where the sim's bat is flattest.
- **By pitch type:** FF 22.6%, SL 23.5%, CH 28.7%, CU 23.6%. MLB has FF highest (57%) and the others at 46–48%.
- **Foul EV** p25/p50/p75 is 63 / 73 / 88 mph, against 70 / 77 / 84.
- **Foul landing:** 89% land on the opposite side or behind it. Measured from straightaway: 45–60° holds 27% opposite against 5% pull, 60–90° holds 31% against 3%, and beyond 90° 31% against 3%. MLB has no public split (R13 F5).

## 4. Bat-angle counterfactual (R13 check 4)

The pivot is swung about a middle-middle aim point at the same distance. Reach and bat speed there stay the same, and the swing plane tilts with the bat (`with_bat_angle`).

**Cage map** (FF middle-middle, perfect perception):
- On time, a steeper bat sends undercut balls to the opposite field sooner and lower. At −4 cm, −20° gives 45° LA, +24° spray, fair; −30° gives 36°, +29°, foul (first landing); −35° gives 32°, +31°, foul.
- It sends topped balls further to the pull side: at +4 cm, spray −10° becomes −14°.
- Over a uniform grid of offsets (±7 cm) and timings (±40 ms), the foul share of contact goes 32.8% → 34.4% → 37.3% at exactly −20° / −30° / −35° (33.0% for the current −20.1°), with more of the fouls on the pull side (15% → 25%).

**PA model** (`foul_diag … - - - <angle>`, 3,000 PAs):

| Bat angle at a middle-middle aim | now (−21°) | −30° | −35° |
|---|---|---|---|
| Foul share of contact | 23.7% | 27.5% | 30.3% |
| \|AD\| < 6° foul per contact (MLB ≈ 40) | 20.4% | 23.1% | 26.2% |
| Swing path tilt, mean (MLB 32°) | 23.7° | **31.9°** | 36.6° |
| Vertical bat angle, p50 (MLB −25 to −35) | −18.6° | −27.5° | −32.4° |
| P(foul \| LA 25–40°, \|AD\| < 6°) (MLB 24) | 15.5% | 29.0% | 44.8% |
| P(foul \| LA 40–50°, \|AD\| < 6°) (MLB 46) | 43.9% | 66.9% | |
| P(foul \| LA 50–60°, \|AD\| < 6°) (MLB 69) | 70.8% | 83.0% | |
| P(foul \| LA 60–70°, \|AD\| < 6°) (MLB 81) | 86.0% | 100% (n = 35) | |
| AD 12–20°: foul per contact (MLB 54) / whiff per swing (MLB 23) | 8.1% / 42.7% | 16.5% / 51.7% | |
| P(foul \| LA < −30°) (MLB 57) | 11.3% | 15.8% | 19.3% |
| K% / Contact% | 19.8 / 64.7 | 20.6 / 65.9 | 21.0 / 66.9 |
| Mean LA / LA SD | 13.1° / 32.9° | 12.3° / 29.8° | 11.6° / 28.1° |

The −30° geometry matches two independent MLB measures at once: Statcast's swing path tilt (mean 31.9° against 32°, p10 22° against 23°, p90 42° against 41°) and Blast's vertical bat angle. It gains about 4 points of foul share. The foul distributions do not support it on their own:
- It **overshoots the well-timed steep bins that passed at −21°**: 67% against 46% at 40–50°, 83% against 69% at 50–60°.
- The low-angle fouls move but stay far short: 11 → 16% below −30° (MLB 57), 3 → 4% at −10–10° (22), 11 → 17% at 10–25° (37).
- Early contact (AD 12–20°) fouls twice as often, 8 → 17% (MLB 54), but whiffs more, 43 → 52% (MLB 23).
- The swing length and the AD spread do not move.

−35° overshoots tilt as well. **Bat angle is a real lever, but only about a seventh of the gap,** and at −30° it trades a steep-foul surplus for a small low-foul gain. It is a candidate to remeasure with the next swing model, not a setting the foul data validate.

## 5. What this means

The R13 picture shifts. Well-timed steep contact is not the gap. What the sim lacks most is contact on mistimed swings: hooked and topped balls early, sliced ones late. The sim's bat leaves the ball's height quickly on either side of on-time, so those swings miss over or under the ball where MLB batters still foul it off. Low-angle fouls on well-timed swings are short as well. The collision model stays untouched for now (§8, as R13 F7 advised).

What to measure next, before changing the swing:
1. **Timing against height, with the pitch and the aim held fixed.** How fast the bat's height at the ball's path changes per ms of timing error, in the sim and as implied by MLB's whiff and foul rates by AD. Then which part of the swing sets it: attack angle against the pitch's descent, plane tilt, or the fixed pivot.
2. **The swing's shape.** §3.6 specifies a hand path (an arc of radius ρ_h ≈ 0.30 m) with a lag λ that releases at contact. v1 is its zero-lag, fixed-pivot case. Whether the full template keeps the bat on the ball's path longer, shortens the swing toward 7.2 ft (v1's symmetric speed bump has no late whip), or changes the AD spread is a hypothesis to test. A counterfactual swing in the cage would test it, not an assumption.
3. **Bat angle:** about −30° matches two kinematic measures and gains about 4 points, but overproduces well-timed steep fouls. Remeasure it with whatever swing shape comes next.

Changing the swing moves the launch-angle mix and the spray, and so the run environment, the fielders' spray reads (`live.rs` fits to `spray_by_bat_angle`, `LEAD_FACE_*`) and the client's batter-box position (taken from the pivot). Any change needs those refits.

**Proposed next step (owner):** measure 1 and 2 as counterfactuals in the cage and with `foul_diag`, then choose the swing change, likely §3.6's hand path and lag with the bat angle remeasured. Until that is done, Discipline's grade-50 values (chase, O1) stay parked as before.
