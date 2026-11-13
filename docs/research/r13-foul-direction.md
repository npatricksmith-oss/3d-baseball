# R13: Foul direction and glancing contact (research note, 2026-11-13)

Research item R13 (TASKS research list), opened by the [foul-share diagnosis](../calibration/2026-11-11-foul-share-diagnosis.md). This note changes no code and no calibration values.

## Question

The sim sends 21% of contact foul; MLB sends about 50% (target B1.15). The diagnosis found that no noise setting gets past ~30%: in the sim, timing errors turn into whiffs and glancing contact stays fair. Three questions:

1. In MLB, which contact goes foul? Is it mistimed swings (balls spraying past the lines) or vertical misses (pop-ups and topped balls), and what share goes back, down the lines, or up?
2. What do published oblique bat-ball collision data say: does glancing contact grip or slide?
3. What do bat-tracking data say about the bat at contact (attack direction, attack angle, vertical bat angle, swing length) that the sim's swing template can be checked against?

## Sources and how the numbers were made

- **Statcast pitch-level sample (primary).** Baseball Savant CSV export, regular season 2025, all pitches on 12 dates spread over the season (Apr 8, Apr 22, May 6, May 20, Jun 3, Jun 17, Jul 1, Jul 22, Aug 5, Aug 19, Sep 2, Sep 16): 51,723 pitches, 24,407 swings. Query template: `https://baseballsavant.mlb.com/statcast_search/csv?all=true&hfGT=R%7C&hfSea=2025%7C&player_type=batter&game_date_gt=<date>&game_date_lt=<date>&type=details`. Retrieved 2026-11-12. It is a sample, not a full season: treat the percentages as about ±1 point (sampling only). Field meanings are from the [CSV documentation](https://baseballsavant.mlb.com/csv-docs). Definitions used:
  - swing = `description` in {foul, foul_tip, hit_into_play, swinging_strike, swinging_strike_blocked} (bunts excluded); contact = foul + foul_tip + hit_into_play;
  - caught foul = in play with "in foul territory" in `des`;
  - spray from `hc_x, hc_y` (center 125.42, 198.27);
  - **sign of `attack_direction`:** in the CSV, positive means *opposite field* for both hands. This is inferred, not documented: on fair balls the raw value correlates +0.42 with spray toward right field for right-handed batters and −0.32 for left-handed batters. Below, attack direction (AD) is written **pull-positive**.
- **Statcast definitions:** MLB glossary pages for [attack direction](https://www.mlb.com/glossary/statcast/attack-direction), [attack angle](https://www.mlb.com/glossary/statcast/attack-angle), [swing path tilt](https://www.mlb.com/glossary/statcast/swing-path-tilt), [swing length](https://www.mlb.com/glossary/statcast/swing-length), and [intercept point](https://www.mlb.com/glossary/statcast/intercept-point), all read 2026-11-12. Also M. Petriello, ["4 new swing metrics…"](https://www.mlb.com/news/new-statcast-swing-metrics-2025), MLB.com, 2025-05-20.
- **Foul landing zones:** A. Choi, ["We Watched 906 Foul Balls To Find Out Where The Most Dangerous Ones Land"](https://fivethirtyeight.com/features/we-watched-906-foul-balls-to-find-out-where-the-most-dangerous-ones-land/), FiveThirtyEight, 2019-07-15 (now redirects; read from the Wayback copy of 2026-03-01). Data: [fivethirtyeight/data/foul-balls](https://github.com/fivethirtyeight/data/tree/master/foul-balls).
- **Collision physics:**
  - Kensrud, Nathan & Smith, "Oblique Collisions of Baseballs and Softballs with a Bat," *Am. J. Phys.* (2017), doi:10.1119/1.4982793, [arXiv:1610.03464](https://arxiv.org/abs/1610.03464) (KNS);
  - Cross & Nathan, "Scattering of a Baseball by a Bat," *Am. J. Phys.* 74, 896 (2006), [arXiv:physics/0605040](https://arxiv.org/abs/physics/0605040);
  - Sawicki, Hubbard & Stronge, "How to hit home runs," *Am. J. Phys.* 71, 1152 (2003), [PDF](https://baseball.physics.illinois.edu/AJP-Nov03.pdf) (SHS).
- **Vertical bat angle and time to contact:** Blast Motion, [FAQ](https://blastmotion.com/faq/) and ["What Is Vertical Bat Angle"](https://blastmotion.com/blog/what-is-vertical-bat-angle-in-a-baseball-swing/) (2026-07-24). Vendor figures from a proprietary sensor database; weaker than Statcast.

## Findings

### F1. The target holds: about half of contact goes foul, and the share is steady across counts

2025 sample:
- Per swing: foul 37.5%, foul tip 2.1%, in play 37.7%, whiff 22.7%.
- Foul share of contact: 48.6% for `foul` only, **51.2% with foul tips**, and 52.5% if caught fouls are counted too.
- By count, foul share of contact is 50.5% with 0 strikes, 51.8% with 1, and 51.3% with 2. It is not a two-strike "protect" effect.

| Split | Foul share of contact |
|---|---|
| Four-seam (FF) | 57.2% |
| Cutter (FC) | 53.7% |
| Splitter (FS) | 51.0% |
| Slider, sweeper, curve, change, sinker | 46–48% |
| In zone (Gameday zones 1–9) | 49.6% |
| Out of zone (11–14) | 57.2% |
| Height: below the zone | 52.2% |
| Height: bottom third | 46.7% |
| Height: middle third | 47.3% |
| Height: top third | 57.8% |
| Height: above the zone | 69.1% |

Height is normalized by `sz_bot` and `sz_top`. High fastballs are where fouls concentrate.

### F2. Fouls are weak, and most are steep: up or down, rarely on a line

| | n | Launch angle tracked | LA p5 / p25 / p50 / p75 / p95 (°) | EV p25 / p50 / p75 (mph) |
|---|---|---|---|---|
| Foul (not tip) | 9,161 | 88% | −43 / 1 / 30 / 50 / 71 | 70 / 77 / 84 |
| Caught foul | 245 | 100% | 45 / 61 / 71 / 79 / 86 | 71 / 79 / 87 |
| Fair in play | 8,951 | 100% | −39 / −7 / 12 / 29 / 54 | 81 / 92 / 100 |

Launch-angle mix of tracked fouls versus fair balls:

| LA band | Fouls | Fair balls |
|---|---|---|
| Below −10° | 21.6% | 21.1% |
| −10° to 10° | 7.8% | 25.6% |
| 10° to 25° | 14.2% | 22.1% |
| 25° to 50° | 30.4% | 24.0% |
| Above 50° | 26.0% | 7.2% |

The 12% of fouls with no launch angle are probably mostly balls going straight back that the system did not track (inference, not verified).

### F3. Even well-timed contact is about 40% foul, and it goes foul by launch angle

P(foul | contact) by pull-positive attack direction. Attack direction is measured at the bat-ball intercept, so per the glossary it is largely a timing measure:

| AD bin (°) | Share of swings | Whiff per swing | Foul per contact |
|---|---|---|---|
| < −20 (late) | 13.7% | 32.8% | 75.8% |
| −20 to −12 | 12.3% | 17.5% | 56.7% |
| −12 to −6 | 11.7% | 13.4% | 49.2% |
| −6 to 0 | 12.8% | 10.9% | 42.7% |
| 0 to 6 | 12.4% | 11.8% | 39.4% |
| 6 to 12 | 11.0% | 13.6% | 42.4% |
| 12 to 20 | 11.2% | 22.6% | 54.0% |
| 20 to 30 | 8.5% | 39.9% | 76.7% |
| > 30 (early) | 6.2% | 73.6% | 85.9% |

P(foul | contact) by launch angle, for all contact and for well-timed contact (|AD| < 6°):

| LA (°) | All contact | |AD| < 6° |
|---|---|---|
| below −30 | 57% | 62% |
| −30 to −10 | 41% | 20% |
| −10 to 10 | 22% | **5%** |
| 10 to 25 | 37% | 20% |
| 25 to 40 | 47% | 24% |
| 40 to 50 | 66% | 46% |
| 50 to 60 | 74% | **69%** |
| 60 to 70 | 78% | **81%** |
| 70 and up | 87% | **88%** |

Timing matters: past |AD| ≈ 12° the foul share climbs to 55–86%. But many fouls come from swings that were close to on time: 22.5% of fouls had |AD| < 6° and 44.6% had |AD| < 12°. Among those well-timed fouls, about 57% left above 40° and about 15% below −30°. In MLB, a well-timed ball that leaves at 50–70° is foul 70–80% of the time.

### F4. Fouls come from both sides of the timing window, and the two sides differ in launch

- 45% of fouls have a pull-side attack direction (early), 55% an opposite-field one (late).
- Early fouls (AD > +10°) are mostly hooked or topped grounders: LA p50 0°, 42% below −10°.
- Late fouls (AD < −10°) are mostly sliced flies: LA p50 33°, 6% below −10°.
- Near-center fouls are split between pop-ups (≈40% above 50°) and balls chopped down (14–31% below −10°).

The sim has almost no pull-side fouls (0.4% of contact, 2% of its fouls).

### F5. Landing area of uncaught fouls (FiveThirtyEight 2019, 906 balls, 10 parks, foul tips and caught fouls excluded)

- Zones 1–3, defined in the article as the netted seats "behind home plate and both dugouts" plus the matching foul ground: 454 balls (50.1%).
- Zones 4–5, from the dugouts to the poles: 441 (48.7%).
- Zones 6–7, past the poles: 11 (1.2%).
- Zone 1 alone holds 278 balls (30.7%): 80 flies, 73 grounders, 54 liners, 54 pop-ups, and 17 that hit the batter. That zone 1 is the area behind the plate is my reading of its contents. The article text does not say so.
- Statcast measured EV on only 580 of the 906 fouls. The fouls reaching zones 4–5 are mostly flies (317 of 441) and are hit slightly harder (median EV 77–79 mph against 71–74 mph in zones 1–3).
- The data have no batter hand, so pull side versus opposite side cannot be split.
- **Public Statcast does not give foul direction:** `hc_x` and `hc_y` are empty for all 9,664 fouls and foul tips in the 2025 sample. No published source found gives the MLB pull / opposite / back split. That part of R13 stays open.

### F6. Bat kinematics at contact (Statcast 2025 sample unless noted)

| Quantity | MLB value | Source |
|---|---|---|
| Bat speed (all swings with tracking) | mean 69.9, p50 71.2, p10 62.3, p90 77.5 mph | sample |
| Attack angle | mean 9.0°, p10 −5°, p90 23°; glossary average ≈10° | sample; glossary |
| Attack direction | p10 25° oppo, p50 ≈0°, p90 23° pull; glossary average ≈2° pull, seasonal player means 15° pull to 10° oppo | sample; glossary |
| Swing path tilt (last 40 ms) | mean 32°, p10 23°, p90 41°; by pitch height 37° low, 32° middle, 25° high | sample; Petriello 2025 |
| Swing length (barrel path from the start of tracking, ~150 ms, to impact) | mean 7.2 ft, p10 6.0, p90 8.4; glossary 7.3 ft | sample; glossary |
| Intercept point vs batter center of mass | mean 29.8 in out front; glossary ≈30 in | sample; glossary |
| AD versus intercept depth | **−1.48° per inch deeper** (corr −0.89 RHB, −0.85 LHB) | sample |
| Attack angle versus AD | **+0.41° per degree toward pull** (corr 0.71). Early swings meet the ball rising and late swings meet it level or falling: median AA −4° at AD < −20°, 22° at AD > +20° | sample; glossary says the same in words |
| Time to contact (start of downswing to impact) | 0.149 s | Blast FAQ (vendor) |
| Vertical bat angle at impact (barrel below hands) | "−25° to −35°" for professionals; steeper on low pitches | Blast blog (proprietary; unverified) |

By outcome, with depth measured as the intercept point **relative to that batter's own average** (per batter and side, at least 20 swings), positive out front:

| Outcome | AD p50 (°) | AD SD (°) | Depth p50 (in) | Depth SD (in) |
|---|---|---|---|---|
| Fair, pulled | +5.7 | 14 | +3.4 | 6.5 |
| Fair, opposite field | −10.3 | 19 | −5.4 | 7.5 |
| Foul | −2.6 | 18 | −2.2 | 10.2 |
| Whiff | +8.0 | 26 | +3.3 | 15.6 |

### F7. Oblique collision data stop at moderate obliquity; what exists says the ball grips

- **KNS 2017:**
  - Setup: a swinging bat (63–88 mph) struck a stationary ball. Bat-path angle ψ went up to 30° and offset up to 1.35 in (baseballs), with launch 0–30° and spin to 3500 rpm.
  - All collisions were "stick-slip", with slip only at the very end. The ball gripped, giving up to 40% more spin than rolling contact.
  - Fit: e_x = 0.405 ± 0.010 (wood bat 0.464).
  - Their model puts gross slip at tan ψ ≥ 3.5 μ(1 + e_y), "near ψ ≈ 60°" for μ = 0.5, outside the tested range.
  - They do not treat glancing, pop-up, or foul collisions.
- **Cross & Nathan 2006:**
  - Setup: balls dropped at ~4 m/s onto a hand-held bat, scattering angles up to ~120°.
  - "No collision in the data set showed continuous sliding" (μ ≥ 0.50); e_x ≈ 0.16.
  - The authors warn against extrapolating to game speed.
- **SHS 2003:** μ ≈ 0.50 for wood. Within 0.35–0.50, μ "does not affect batted ball spin, velocity, or launch angle". Optimum fastball undercut is 2.65 cm with an upward swing of 0.159 rad.
- **Consequence for the sim:** the physics-spec §8.1 solver with its defaults sticks at least as far as the measured cases. As a derived check for an infinitely heavy bat, the spec's own stick condition gives tan ψ ≤ μ(1 + e_n)(1 + 1/α)/(1 + e_t) ≈ 0.5 · 1.45 · 3.5 / 1.4 ≈ 1.8, so ψ ≈ 61° with α = 0.4. That matches KNS's 60°. **No published game-speed data say that glancing contact slides more than the sim lets it.** Making the ball slide to create fouls would be a Tier A change with no evidence behind it, so this note does not propose one.

## What this implies for our model

The MLB data move the diagnosis. Timing noise alone cannot be the gap: MLB contact made within ±6° of on-time attack direction is still ~40% foul, while the sim's on-time slice is fair from −5 to +5 cm of vertical offset. **The sim's biggest miss is likely the well-timed, steep contact** (F3): in MLB a well-timed ball that leaves at 50–70° is foul 70–80% of the time. The cage map's 5-cm undercut ball (56°, 83 mph, about 30° toward the opposite field, fair) is the archetypal MLB foul. Second, the sim lacks early-side fouls (F4). Concrete checks, each with an MLB reference:

1. **P(foul | contact, LA ≥ 50°) ≥ 0.70**, also for well-timed contact. The sim's value is unknown; the pop-ups in the cage map suggest it is low. This is the main check.
2. **P(foul | contact, |AD| < 6°) ≈ 0.40.** At |AD| 12–20° it should be ≈0.55; at 20–30° ≈0.77. Whiff per swing should be ≈0.11 at |AD| < 6°, 0.23–0.40 at 12–30°, and ≈0.74 beyond 30°.
3. **Fouls split ≈45/55 early/late** by attack direction. Early fouls are low (LA p50 ≈0°); late fouls are flies (LA p50 ≈33°). The sim's 2% pull-side fouls fails this.
4. **Bat geometry at contact:**
   - The sim's bat tilts about 18–20° below horizontal at contact for a middle pitch (diagnosis §3a; my estimate from the `swing.rs` template: pivot (−0.95, 0.30, 0.60·h), aim (0, 0.35, 0.8) m). Blast's professional range is −25° to −35°, steeper for low pitches.
   - A steeper bat turns a vertical miss into a sideways one: the contact normal of an undercut tilts toward the opposite field by about the bat's vertical angle. This is candidate 2 in the diagnosis and the cheapest geometric lever, since the template authors it ([E] values) and it is not Tier A physics.
   - Check: vertical bat angle at contact should be about −30° ± 5° and should get steeper for low pitches.
5. **Swing time-scale checks (these probably pass; measure, don't assume):**
   - Bat speed 32 m/s = 71.6 mph: matches.
   - T_swing 0.150 s against Blast's 0.149: matches.
   - Rotation radius: the sim rotates at ω = 32 rad/s about a fixed pivot with a 1.0 m sweet-spot radius. Derived, not measured: for v_ball ≈ 38 m/s, a timing error Δ moves AD by ≈ ω·v_b/(v_s + v_b) ≈ 1.0° per ms and the contact depth by ≈ 0.69 in per ms. That gives ≈ 1.45° per inch, against **1.48° per inch** measured in MLB (F6). So a fixed 1 m arc reproduces the MLB depth-direction relation, and "time through the zone" (candidate 3) is not supported as the main cause. On that conversion, the sim's fair timing window (−23 to +18 ms) is about ±20° of AD. That is roughly where MLB's foul share passes 55–75%, so the sim's window edges are in about the right place. What is wrong is the inside of the window.
   - Swing length: the sim's sweet spot travels about 2.7 rad × 1.0 m ≈ 8.9 ft over the last 150 ms (derived from the speed profile). MLB averages 7.2–7.3 ft. A ~20% longer swing is worth confirming by measurement.
   - Attack angle should move with AD at **≈ +0.4° per degree toward pull** (F6). With a fixed swing plane, the sim's slope is set by the plane's tilt.
6. **Foul quality:** sim fouls should have EV p50 ≈ 77 mph (p25 70, p75 84). Their LA should be bimodal, with < 10% between −10° and 10°.
7. **Location and pitch type:** the foul share should be highest on high pitches (top third ≈58%, above the zone ≈69%) and on four-seamers (≈57%), and ≈47% on breaking balls and changeups.
8. **Collision model:** leave §8 alone for this problem (F7). The question for §8 is narrower: at the obliquities where the sim switches from stick to slide (between −5 and −7 cm), print ψ and confirm it is near the ≈60° stick limit, so that no other path makes glancing balls grip artificially.

## Proposed measurements (sim side)

Extend `cargo run --release -p bb_game --example foul_diag` to print the same tables as F1–F6 from the sim's contacts and misses, using the Statcast definitions:

- **Attack direction and attack angle** of the sweet-spot velocity at contact (at closest approach for a miss), pull-positive. Print quantiles by outcome (fair pulled, center, opposite; foul; whiff), and P(foul | contact) and P(whiff | swing) in the F3 AD bins.
- **P(foul | contact) by launch-angle bin**, for all contact and for |AD| < 6°: the second F3 table.
- **Intercept depth** (contact point y minus the batter's mean contact y) by outcome, and the AD-against-depth slope (target −1.48° per inch).
- **Slope of attack angle against AD** (target ≈ +0.4).
- **Vertical bat angle at contact** by pitch-height third (target about −25° to −35°), and **swing length** of the sweet spot over the last 150 ms (target 7.2 ft).
- **Foul EV and LA quantiles**; the foul share by pitch type and by normalized pitch height (F1).
- **Foul landing**: share of fouls with |spray| 45–60°, 60–90°, and > 90° (backward). Compare only loosely with F5: about half of uncaught non-tip fouls land in the netted zones behind the plate and dugouts, and about half beyond the dugouts.

Add a counterfactual pass to `sim-cli cage-map`: re-run the on-time vertical-offset slice with the template's bat tilt at −20°, −30°, and −35°. Print LA, spray, and fair or foul for offsets from −7 to +7 cm. This tests candidate 2 directly, before any calibration change.

## Open questions

1. **The MLB pull / opposite / back split of fouls** has no public source: Statcast leaves foul hit coordinates empty, and the FiveThirtyEight zones have no batter hand. Getting it would mean hand-charting video or using a licensed Hawk-Eye feed.
2. **Statcast launch angle for balls that go backward:** whether it is reported as the plain elevation angle (as assumed here) is not documented. If it is, F2–F3 mix balls going back with balls going up and forward.
3. **The 12% of fouls with no launch angle** are assumed to be balls going straight back, which is unverified. If they are, the backward share is underrepresented in F2–F3.
4. **Vertical bat angle** has only vendor numbers (Blast) and one college study, none from Statcast. Horizontal bat angle at contact (the bat axis against the plate line) is not public anywhere I found.
5. **Oblique collisions above ψ ≈ 30° at game speed** have not been measured, as far as this search found. The sim's ≈60° stick limit is model extrapolation (KNS and our §8.1 agree), not data.
6. **Sign and mean of `attack_direction`:** the CSV sign is inferred (F6). The sample's mean (≈0°) differs from the glossary's 2° pull, perhaps because of competitive-swing filtering or player-season weighting. This does not affect the bins above beyond about 2°.
