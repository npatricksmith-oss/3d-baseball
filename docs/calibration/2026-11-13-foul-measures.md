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

Why the bats pass over the ball is measured in §6. Most of it is where the batter aimed, not the swing's geometry.

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

The R13 picture shifts. Well-timed steep contact is not the gap. What the sim lacks most is fouls on swings that are moderately off: hooked and topped balls early, sliced ones late, and topped and undercut balls on time. Low-angle fouls on well-timed swings are short as well. The collision model stays untouched for now (§8, as R13 F7 advised). §6 measures where the misses come from, and §7 says what that changes.

Changing the swing would move the launch-angle mix and the spray, and so the run environment, the fielders' spray reads (`live.rs` fits to `spray_by_bat_angle`, `LEAD_FACE_*`) and the client's batter-box position (taken from the pivot). Any change needs those refits.

## 6. Where the misses come from (follow-up, same day)

The owner asked Fable to choose between measuring first and building §3.6's hand path. Fable recommended measuring, with a change of aim. Its check: with perception almost removed (`foul_diag -- 3000 15 - 0.05`: execution timing σ 15 ms), early swings almost never whiff (1 whiff in about 580 swings at AD 12–30°; well-timed contact also fouled only 4.7%). So the over-the-ball misses had to come mostly from where the batter aimed. That run changes perception and timing noise together, so it supports the aim-error reading but does not isolate the swing's shape (Codex foul-sources review). The new measures below are in `foul_diag` and `cage-map`. The first version of this section overstated four conclusions; the [Codex foul-sources review](../reviews/2026-11-13-codex-foul-sources-review.md) corrected them.

**Each whiff's height miss, as an accounting** (bat above the ball's center, cm, means, 3,000 PAs, default knobs).
- "Nominal" is the executed swing's sweet spot at its own contact time, against the ball when it reaches that depth.
- It is written as the batter's final aim error (his perceived point minus the true one, at the aim's own depth), the template's aim offset (−2 cm), and a rest. The rest holds execution offset and reach noise, plus the ball's drop between the aim's depth and the sweet spot's.
- "Path" is the change from nominal to the closest approach. It sums the sweet spot's movement, the change of station along the bat, and the ball's movement. For AD 12–30° whiffs these are +8.5, −6.8 and −6.3 cm (review probe), so a small net hides large opposing terms.
- The terms are measured at different depths and times, so they are an accounting identity, not independent causes.

| AD bin (°) | whiffs | at intercept | nominal | path (RMS) | aim | offset | rest (RMS) | read right at commit |
|---|---|---|---|---|---|---|---|---|
| < −12 (late) | 78 | −12.5 | −14.9 | +2.4 (5.8) | −12.5 | −2.0 | −0.4 (1.2) | 68% |
| −12 to 12 | 497 | −1.5 | −1.0 | −0.4 (4.6) | +1.3 | −2.0 | −0.4 (1.4) | 72% |
| 12 to 30 | 504 | +15.9 | +20.4 | −4.6 (7.3) | **+22.5** | −2.0 | −0.1 (1.4) | 28% |
| > 30 (early) | 320 | +23.7 | +24.4 | −0.7 (5.1) | **+27.0** | −2.0 | −0.6 (1.7) | 24% |

By pitch type, at AD 12–30°, the mean aim error and the share read right at commit are: FF +7.7 cm (99%), SL +29.9 (13%), CH +23.8 (3%), CU +29.1 (3%).

- **The batter's final aim is the largest term in early whiffs.** On average it exceeds the nominal miss. The rest averages near zero but has an RMS of about 1.4 cm, about the size of execution's vertical noise. The swing's path term averages −4.6 cm on these whiffs but varies by 7 cm.
- **Most early whiffs on SL/CU/CH were misread at commit** as fastballs. Across all swings, height error falls 0.59 cm per ms of timing error (corr −0.78): early goes with aimed high. The commit read is not the whole story. Of the 1,107 early-and-over swings, 760 ended the adjustment loop with the right dominant type (review probe, which instrumented `pa.rs`). The batter found the pitch type and still could not move his aim far enough.
- **What another swing shape would change is untested.** §3.6's hand path and lag are in-plane, but the spec also has a small out-of-plane tilt. Within the same plane, a different phase or hand motion changes where and when the bat meets the ball's height. The review found a whiff (PA 43, pitch 1, AD 18.8°) that a 10 ms later swing with unchanged geometry turns into contact. The measurements here neither support nor exclude a swing change.

**Swing populations by nominal error** (4,356 swings with a nominal error; one pitch came down before the sweet spot's depth):

| Population | share of swings | whiff | foul of contact | read right at commit |
|---|---|---|---|---|
| early, bat over by > 6 cm | 25.4% | 92% | 18% | 27% |
| close: within ±15 ms and ±4 cm | 37.8% | 1% | **10%** | 99% |
| other, bat under by > 4 cm | 25.2% | 37% | 61% | 94% |
| other, the rest | 11.5% | 21% | 15% | 98% |

The joint table (share of swings % / whiff % / foul % of contact; timing negative = early, height positive = bat over):

| height \ timing | < −25 ms | −25..−15 | −15..−5 | −5..5 | 5..15 |
|---|---|---|---|---|---|
| > 6 cm | 14.4 / 100 / 33 | 6.2 / 93 / 17 | 3.7 / 72 / 20 | 1.4 / 52 / 21 | 0.0 |
| 4..6 | 0.5 / 85 / 0 | 1.1 / 39 / 14 | 1.9 / 10 / 9 | 1.3 / 0 / 10 | 0.2 |
| 2..4 | 0.6 / 64 / 40 | 1.3 / 9 / 12 | 3.3 / 3 / 3 | 2.7 / 1 / 1 | 0.4 |
| 0..2 | 0.4 | 1.1 / 4 / 9 | 4.3 / 0 / 1 | 4.1 / 1 / 2 | 0.8 / 0 / 9 |
| −2..0 | 0.3 | 1.1 / 4 / 6 | 4.8 / 0 / 2 | 5.3 / 0 / 6 | 1.9 / 4 / 28 |
| −4..−2 | 0.3 | 0.8 / 9 / 16 | 3.0 / 0 / 9 | 5.4 / 2 / 20 | 1.9 / 0 / 56 |
| −6..−4 | 0.1 | 0.4 | 2.5 / 7 / 22 | 5.2 / 6 / 50 | 2.0 / 14 / 84 |
| < −6 | 0.1 | 0.3 | 2.0 / 42 / 58 | 6.7 / 52 / 84 | 4.1 / 70 / 93 |

- The first version called the errors bimodal. That was wrong: 37% of swings are in neither the early-and-over group nor the close group.
- **The sim's fouls come mostly from the bat under the ball** (61% of that group's contact). The early-and-over group is mostly whiffs.
- **The close group is the largest, and it fouls only 10% of its contact.** MLB's well-timed contact fouls about 40% (R13 F3: |AD| < 6°). The groups are not the same cut, so this is a pointer, not a comparison.

**The contact map at fixed pitch and aim** (`cage-map --type FF`, "Timing × height grid"):
- On time (−5 to +5 ms), contact is fair from about −4 to +5 cm. The foul band is the last 1–2 cm before a miss, under the ball.
- Late (+10 to +30 ms), it fouls across most of the band, as sliced flies and balls going back.
- Early (−20 to −35 ms) with the bat over, it makes pulled grounders launched outside the ±45° lines (`g`). They first land fair, and in live plays they are often still fair: the review ran the 14 `g` cells through the live play and rules engine over 16 seeds each, and got 121 fair, 103 foul. So the grid does not show that these become MLB's early fouls.
- SL and CH maps have the same broad shape.

## 7. What this changes

- **No swing change is indicated or excluded yet.** The hand path's own claims (swing length 9.5 ft against 7.2, the AD spread) remain a separate question. Its effect on the misses is untested.
- **Mid-swing correction is the strongest candidate lever, as an experiment.** On early whiffs the final aim is 20–30 cm high, and most of those batters had the pitch type right by the end of the loop.
  - The correction moves each coordinate at `Adjustability` (0.6 m/s per axis). It stops 60 ms before the aim time, which itself moves within ±10% of the time left.
  - In the default sample the vertical budget has a median of 7.8 cm, a p95 of 9.0 and a maximum of 10.2. The largest actual vertical correction was 9.6 cm (review probe).
  - Whether batters hit that budget (saturation), and what a larger or earlier correction does to whiffs and fouls by AD, is the next measurement. This is Tier B (perception and adjustment noise, PLAN §13) and overlaps coordinator item 6.
- **The close group's low foul rate is a second question:** the on-time map's narrow foul band. MLB fouls 62% of well-timed contact below −30° (R13 F3). In the sim's on-time map, a ball overcut by 5–6 cm leaves at −50° to −67°, sprays −18° to −37° and stays fair, while +7 cm already lands foul backward. Whether real topped balls go foul off the plate, the dirt or the batter needs evidence before any Tier A change.

**Proposed next step (owner):** measure how often the mid-swing correction saturates. Then, as counterfactuals, raise and lengthen it and watch whiffs, fouls by AD and the joint table, before choosing any change. This lands with coordinator item 6. The hand path and the bat angle wait. Until then, Discipline's grade-50 values (chase, O1) stay parked as before.

## 8. The correction experiment (owner: "yes", same day)

§7 proposed the batter's mid-swing correction as the strongest candidate lever, to be tested as an experiment. It is not the foul lever.

Changes made for the experiment:
- `t_cut` and the phase-rate limit were bare literals in `pa.rs`. They are now the named [E] parameters `SwingTemplate::adj_cutoff_s` (60 ms) and `adj_phase_frac` (0.10), with the same values. The game digests for seeds 1, 2, 3 and 42 are bit-identical before and after.
- `PitchRecord::known_gap` (diagnostics only) records, at the last adjustment tick, the batter's latest estimate minus where he had moved his aim.
- `foul_diag` takes `adj=<m/s>`, `cut=<ms>` and `phase=<fraction>` as counterfactuals, and prints the saturation measures below.

**Saturation** (default; at the last tick; the aim error is the estimate's error minus the gap he knew of and had not yet moved):

| Population | n | final aim error, z | his estimate's error | known gap left | \|gap\| > 2 cm | known timing gap | \|gap\| > 3 ms |
|---|---|---|---|---|---|---|---|
| early, bat over by > 6 cm | 1,107 | +25.6 cm | +9.3 | −16.3 | 81% | +22.3 ms | 77% |
| close (±15 ms, ±4 cm) | 1,647 | +1.7 | +1.3 | −0.3 | 21% | +1.6 | 20% |
| bat under by > 4 cm | 1,099 | −5.3 | −3.6 | +1.7 | 29% | +1.0 | 24% |
| AD 12–30° whiffs | 504 | +22.5 | +8.7 | −13.8 | 77% | +16.7 | 77% |

The limits do bind on fooled swings. By the last tick the batter knew the ball was about 16 cm lower and 22 ms later than his swing, and could not get there. About two-thirds of the final aim error on those swings is correction he knew he needed.

**Counterfactuals** (3,000 PAs each; MLB in brackets):

| Variant | foul / contact | \|AD\| < 6° foul | AD 12–20° foul / whiff | early-and-over share | K% | Contact% | Z- / O-Contact% |
|---|---|---|---|---|---|---|---|
| default (0.6 m/s, 60 ms, ±10%) | 23.7 | 20.4 | 8.1 / 42.7 | 25.4 | 19.8 | 64.7 | 71.8 / 40.2 |
| Adjustability 1.2 | 23.6 | 20.6 | 9.3 / 39.8 | 23.2 | 18.1 | 66.9 | 74.1 / 43.2 |
| Adjustability 2.4 | 23.0 | 19.6 | 10.8 / 35.3 | 20.9 | 16.0 | 69.2 | 76.2 / 46.3 |
| Adjustability 100 (no speed limit) | 22.5 | 19.2 | 11.7 / 26.8 | 14.3 | 12.0 | 73.8 | 78.6 / 58.7 |
| phase ±20% | 24.5 | 20.6 | 9.4 / 45.9 | 24.6 | 19.8 | 65.2 | 72.6 / 39.7 |
| phase ±30% | 25.2 | 20.7 | 8.1 / 52.0 | 22.6 | 19.9 | 65.4 | 72.8 / 40.3 |
| cutoff 30 ms | 23.6 | 21.0 | 8.3 / 40.2 | 23.8 | 18.8 | 66.0 | 73.0 / 41.8 |
| 1.2 m/s and ±20% | 24.8 | 20.7 | 10.3 / 44.6 | 22.3 | 18.0 | 67.6 | 74.7 / 44.1 |
| no limits at all (100 m/s, ±100%, no cutoff) | 26.1 | 20.5 | 10.6 / 28.4 | 8.3 | 8.4 | 81.1 | 82.8 / 76.0 |
| MLB | [51] | [40] | [54 / 23] | | [22.5] | [76] | [85 / 57] |

- **More correction saves swings, but as fair contact.** Even with no limits at all, the foul share moves only from 23.7% to 26.1%. Fooled swings fall from 25% to 8%, whiffs fall and K% halves. Foul per contact holds at about 20% for well-timed contact and 8–12% at AD 12–20°, in every variant. The foul rate at a given timing is a property of the contact map, not of how many swings reach it.
- **The phase limit's wider settings move the timing distribution** (more swings in AD 0–6° and fewer early) but raise whiffs there. They do not change the foul rate in any bin.
- **What the correction does move is contact.** Contact%, Z-Contact% and O-Contact% are all below MLB's band now (64.7, 71.8, 40.2 against 76, 85, 57). They rise toward it with a larger Adjustability: at 2.4 m/s O-Contact% enters its band. But K% falls to 16 while the foul share stays low. With MLB's foul share, two-strike fouls would keep more at-bats alive and push K% the other way. So Adjustability cannot be set from these numbers until the foul share is fixed. This is B1 calibration (O1), noted here, not changed.

**What this changes.** The question is now the contact map: why contact at a given timing and height is fair so much more often than MLB's. At AD 12–20° MLB fouls 54% of contact and the sim 8–12%, whatever the correction. In the cage grid, contact 10–15 ms early is fair across −8 to +4 cm. Two candidates remain, neither measured yet:
1. **Spray tied to AD.** v1's radial bat about a fixed pivot points the bat's face exactly along the sweet spot's velocity, so a ball leaves close to the swing's direction. No public MLB data give spray minus AD (R13 F5: Statcast has no foul coordinates), so this rests on v1's construction and on MLB's fouls at a given AD, not on a measured spread. A bat whose face and path differ, as with §3.6's hand path and lag, would spread spray around AD and send more early and late contact past the lines. That is a cage counterfactual to build.
2. **The on-time foul band** (§6, §7): topped and undercut balls on time stay fair in the sim. The −30° bat angle widens it a little (§4).

**Proposed next step (owner):** a cage counterfactual of a swing whose bat face and path can differ: §3.6's hand path and lag, or the simplest version of it. Measure spray minus AD and the foul share by AD before building anything into the PA model. Adjustability stays at 0.6 m/s.
