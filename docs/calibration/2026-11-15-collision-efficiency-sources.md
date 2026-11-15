# What the sources say about how hard a squared-up ball leaves a wood bat (2026-11-15)

Branch `work/collision-q-research`, from main `6ef084a`. A sourcing job: no sim code, spec or run changed. It answers the condition D19 left open ("collision efficiency stays at q = 0.20; revisit only as a sourcing question if a tail gap remains", PLAN §1) after [power-mix](2026-11-15-power-mix.md) and [ev-la-bins](2026-11-15-ev-la-bins.md) found the gap in the contact itself. Arithmetic in this note is stdlib Python from the code's constants (script not kept; every figure is reproducible from §1).

**Labels.** SOURCED: I opened and read the document (text extracted from the PDF or page) and the number is in it. ESTIMATE: my derivation or judgment, with the inputs shown. Unverified: seen only in a search snippet or a fetch summary; carries no conclusion. Figures in the papers were images I could not read; numbers only from figures are marked.

## Summary for the lead

1. **The sim's sweet-spot q is 0.200 and its best q anywhere on the barrel is 0.219 (at 26.5 in from the knob).** Its highest possible exit speed for a 71.6 mph sweet-spot swing against an 89.5 mph pitch is **103.6 mph** (rigid-bat arithmetic, §1), not the 108.7 mph of Statcast's 1.23 × bat + 0.23 × pitch.
2. **Lab q for MLB-type wood bats is 0.21 to 0.23 at lab speeds** (SOURCED: 0.214 and 0.227 for two 34-in wood bats at 29 in, ball 110 mph, Nathan et al. 2010; 0.228 for the best MLB wood bat of 1999, NCAA/BESR). The mass term of those bats matches the sim's (m/M_eff 0.227 against the sim's 0.224 at 29 in); **the sim's ball-bat COR does not**: 0.44 at 29 in against a measured BBCOR of 0.49. Lab BBCOR falls with speed by an amount no source I could open gives, so the game-speed q is an ESTIMATE of about 0.21 to 0.23.
3. **The real data already say about q = 0.20 at the 90th percentile** (ESTIMATE, §3): the real p90 squared-up ratio 0.958 corresponds to q_eff = 0.201 and the sim's 0.934 to 0.186, in the same units. The real p90 exit speed (104.1 mph for 71.6/89.5) is the sim's *ceiling* (103.6). A 90th percentile at 0.20 means the real ceiling is higher; keeping the sim's p90-to-ceiling ratio (0.98), the real sweet-spot q is about 0.215.
4. **The 0.958 against 0.934 gap survives matching conventions** (§3): bat speed is the sweet-spot speed on both sides, and switching the pitch term from release to plate speed multiplies both ratios by about 1.015 and leaves the 0.024 difference.
5. **Recommendation: 0.20 is at the bottom edge of the sourced range, not centred in it. A sourced change is q ≈ 0.215 (range 0.21–0.23), by `BatBallParams::e_ref` 0.442 → about 0.46 (§5). It cannot be adopted alone**: the earlier counterfactual (swing-model §6) found q = 0.23 alone puts mean exit speed at 93.1 mph against 89.5, and real contact has a lower low end than the sim's. A change to q must be paired with a re-look at how contact is spread over the barrel and the rest of the exit-speed distribution. Confidence: moderate on the direction, low on the third digit.

## 1. What the sim does now

All from the code and physics-spec §8; q values computed with the code's constants (34-in bat, 660 kg/m³, ball 0.14529 kg).

- **Bat.** `BatProfile::default_34in` (`bb_physics/src/bat.rs:56`): length 34 in, integrated mass 0.9645 kg (34.0 oz), centre of mass 23.53 in from the knob, I_cm 0.03834 kg·m² (12,550 oz·in² about a point 6 in from the knob).
- **Contact point.** Sweet spot u_ss = L − 0.15 m = 28.09 in from the knob (`bat.rs:141`, `ss_from_tip` `bat.rs:197`; physics-spec §8.2 "Statcast also defines bat speed 6 in from the tip").
- **Normal COR.** `BatBallParams::e_n` (`bat.rs:249`): e_n = e_peak · f(u), e_peak = e_ref − s_e · (|c_n| − v_ref), with e_ref = 0.442, v_ref = 70 m/s, s_e = 0.001 per m/s (`bat.rs:203-205`). |c_n| is the contact-point normal approach speed at the first impulse (`contact.rs:196`).
- **Location factor f(u)** (`bat.rs:219`): 1 at u_ss; quadratic to f_tip = 0.75 at the tip; quadratic to f_taper = 0.55 at u_taper = 22 in; floor f_handle = 0.35.
- **Effective mass.** Rigid body: 1/M_eff = 1/M + b²/I_cm, b = u − u_cm (`bat.rs:49`; spec §8.1). The bat does not vibrate; f(u) stands in for the vibration loss. M_eff is 0.721 kg at u_ss (r = m/M_eff = 0.2015).
- **q along the barrel** (q = (e − r)/(1 + r), at |c_n| = 70 m/s):

| u (in from knob) | 24 | 25 | 26 | 26.5 | 27 | 28.1 (u_ss) | 29 | 30 | 31 | 32 | 33 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| e_n | 0.352 | 0.391 | 0.419 | 0.426 | 0.436 | 0.442 | 0.439 | 0.431 | 0.415 | 0.394 | 0.366 |
| M_eff (kg) | 0.961 | 0.932 | 0.878 | ≈0.84 | 0.807 | 0.721 | 0.649 | 0.575 | 0.506 | 0.446 | 0.393 |
| q | 0.175 | 0.203 | 0.217 | 0.219 | 0.217 | **0.200** | 0.176 | 0.142 | 0.100 | 0.051 | −0.003 |

  **Peak q = 0.2186 at 26.5 in, 1.6 in inboard of the sweet spot**, because the rigid M_eff grows toward the centre of mass faster than f(u) falls. The region within 0.01 of the peak is 25.3–27.7 in (2.4 in wide), within 0.02 is 24.8–28.2 in (3.4 in). The sweet spot sits at the outer edge of the plateau, not its middle. (The value 0.200 at u_ss is the A8 target, spec §8.2/§10.)
- **Speed dependence.** Only through s_e: −0.001 per m/s. From a 56 m/s lab-like closing speed to 72 m/s (161 mph) e_peak falls 0.016 (0.456 → 0.440). The sim's q at u_ss is 0.2011 at a pitch of 82.5 mph and 0.1985 at 89.5 mph.
- **Where bat speed is measured.** The sim's swing is a pendulum; the speed that enters the collision is the speed of the contact point on the bat, `cp.bat.point_velocity`. The *reported* `bat_speed` (`contact.rs:208`, and `ev_bat.rs:16-30`) is the speed at u_ss at the contact instant, the same definition as Statcast's. So the sim's bat-speed axis in the power-mix tables is the Statcast convention, and contact inboard of u_ss meets the ball at less than that number.
- **Pitch speed.** The collision uses the ball's velocity at contact (after flight drag, so at the plate: about 8% below release is the brief's figure; ESTIMATE 82.3 mph for 89.5). The power-mix ratio used release speed.
- **Maximum exit speed, 71.6 mph sweet-spot swing, 89.5 mph release** (ESTIMATE: rigid rotation about a pivot 3.5 in from the knob, 9 cm per Nathan 2000's high-speed case, so the speed at u scales with u − pivot; `ev_bat`-style scan of u over 22–34 in):

| convention | max EV (mph) | at u (in) | q there |
|---|---|---|---|
| sim, pitch 89.5 mph (release) | **103.6** | 28.0 | 0.201 |
| sim, pitch 82.5 mph (plate) | 102.6 | 28.05 | 0.202 |
| sim, pivot 11.8 in (an alternative), pitch 89.5 | 104.2 | 28.9 | 0.175 |
| Statcast: 1.23 × 71.6 + 0.23 × 89.5 | **108.7** | – | 0.23 |
| Statcast, plate pitch 82.5 | 107.0 | – | 0.23 |
| q = 0.20 formula: 0.20 × 89.5 + 1.20 × 71.6 | 103.8 | – | 0.20 |

  The 103.6 matches the plain formula at q = 0.20 (103.8). Because the outermost 1.6 in of peak q is bought with a 6% lower bat speed, the best EV is at the sweet spot. **The sim's ceiling is 5.0 mph below Statcast's 1.23/0.23 line** at release speed (4.4 at plate speed). Each 0.01 of q is 1.6 mph at 71.6 + 89.5 mph (closing speed 161 mph).

## 2. What the literature says

**Formula.** Ball exit speed v_ball = q · v_pitch + (1 + q) · v_bat. SOURCED: Nathan, Smith, Faber & Russell, "Corked Bats, Juiced Balls, and Humidors: The Physics of Cheating in Baseball", arXiv:1009.2549 (2010), Eq. 2 (https://arxiv.org/pdf/1009.2549, PDF text read). The same paper defines the collision efficiency e_A = (e − m/M_eff)/(1 + m/M_eff) (Eq. 1), e the ball-bat COR (BBCOR), M_eff = I₆/z² for a bat pivoted six inches from the knob. The NCAA's BESR (Ashton-Miller, Carroll, Johnson, Nathan et al., "The BESR (Ball Exit Speed Ratio)", NCAA Baseball Research Panel, 2004, https://www.acs.psu.edu/drussell/bats/besr/BESRWhitePaper.pdf, read) gives the equivalent v_exit = (BESR + ½) v_bat + (BESR − ½) v_pitch, so **q = BESR − ½**.

**Measured values, wood bats, MLB ball.**

| What | Value | Source and conditions |
|---|---|---|
| e_A and BBCOR, "unmodified" Rawlings 34-in wood bat, 30.6 oz | e_A 0.214(2), BBCOR 0.490(2), m/M_eff 0.2269 | arXiv:1009.2549 Table I; impacts at 29 in from the knob, ball fired at ~110 mph onto a stationary bat pivoted 6 in from the knob, 72 °F, 50% RH. Each bat 5–6 impacts |
| same, "control" bat, 31.1 oz | e_A 0.227(1), BBCOR 0.494(2) | same table |
| BBCOR over all 54 impacts, four bats | mean 0.489, SD 0.009 | same, Fig. 3 |
| Best MLB wood bat tested in 1999 | BESR 0.728, i.e. q = 0.228 | BESR white paper: pitch 70 mph, bat speed 66 mph measured 6 in from the barrel end, "an initial lot of baseballs"; sample size not stated |
| Ball-bat BBCOR at the peak location, 125 mph | 0.487 (1970s balls) and 0.491 (2004 balls) | arXiv:1009.2549 Fig. 7 caption (mean of three impacts each, wood bat) |
| Ball COR on a rigid plate | 0.470 (1970s) and 0.468 (2004) at 120 mph | same, Fig. 6; the MLB specification is 0.514–0.578 at 58 mph |
| BBCOR vs impact location, two standard maple bats (S154/S155, ~34 oz, 35.1–35.2 in) | peak above 0.5; the range of impact locations where BBCOR exceeds 0.5 is 2.4 in wide; run-to-run SD 0.0038 (288 impacts); air cannon at 100 mph | Nathan, Smith & Russell, "Studies of the Torpedo Bat", ISEA 2026 v7 (https://baseball.physics.illinois.edu/ISEA2026-Torpedo-v7.pdf, read). The curve itself is a figure I could not read; ball type not stated |

**The sim's mass term matches, its COR does not.** At 29 in the lab bats have m/M_eff = 0.2269 (measured, flexible bat) and the sim's rigid bat 0.2239. With the same mass term, the lab BBCOR of 0.490 at 110 mph gives q = 0.214 and the sim's e = 0.439 at 70 m/s gives q = 0.176. The comparison is at a station and speed where both are defined, so it is a COR difference of about 0.05, not a mass difference. (The sim's e_n is the sweet-spot value 0.442 at 70 m/s, not a lab number; physics-spec §8.2 notes e_ref was derived to hit q = 0.20, not measured.)

**Speed dependence of the COR.** SOURCED qualitative: the ball COR falls with speed and balls with the same 58 mph COR can differ at game speed ("the spread in values approximately doubles at 120 mph", arXiv:1009.2549 §IV, citing Hendee et al. to 90 mph and Chauvin & Carlson to 150 mph; I did not open those). Nathan 2000 ("Dynamics of the baseball-bat collision", Am. J. Phys. 68, 979, read from PDF) finds the effective COR of the ball on a bat "only weakly dependent on the impact speed", slightly higher at half the speed (Fig. 12; it assumes e₀ = 0.50 at 160 mph and 0.53 at 130 mph for the ball alone). **No opened source gives a BBCOR number at 150–165 mph for a wood bat.** The torpedo paper says lab speed ordinarily correlates with a ~160 mph game closing speed and uses a model (Smith, Nathan & Duris 2010, not opened) to extrapolate to 150–165 mph. ESTIMATE of the game-speed peak COR: lab 0.49–0.50 at 100–125 mph, less an assumed drop of 0.02–0.03 to 160 mph (my guess, bracketed by Nathan 2000's "weak" dependence and the ball-alone COR falling from ~0.5 at 58 mph to 0.47 at 120 mph; the plots were not readable) gives **0.46–0.48**. With r = 0.20 that is q = (0.47 − 0.20)/1.20 = **0.225 (0.217–0.233)**.

**Variation along the barrel.** SOURCED: BBCOR is a parabola in impact location peaked "between the nodes of the lowest two modes", the sweet spot about 5–6 in from the tip (torpedo paper); in Nathan 2000 the exit speed at 60 cm (about 10 in inboard of the peak) is down 25% from the rigid-bat value, and falls off rapidly beyond about 80 cm (about 2 in outboard of 71 cm, the 84 cm test bat). The 2.4-in-wide region above 0.5 in a ~0.51 peak corresponds to a region of about 0.01–0.015 of BBCOR below the peak, 2.4 in wide; the sim's region within 0.01 of q's peak is also 2.4 in. ESTIMATE: the **widths agree; the peak value is the difference**.

**Flexible against rigid effective mass.** Nathan, "The Physics of the Torpedo Bat: An Update" (dated 2025-04-29, read as a PDF from baseball.physics.illinois.edu): M_eff of a real bat is always below the rigid-body value except at the sweet spot, where vibration is smallest. The sim's rigid M_eff is therefore right at u_ss and too large away from it; its f(u) is the stand-in.

**The ball.** SOURCED: Albert, Hosoi, Nathan & Smith, "Preliminary Report of the Committee Studying Home Run Rates in MLB" (2019, read from PDF text, https://baseball.physics.illinois.edu/HRReport2019.pdf): Rawlings's high-speed CCOR (cylindrical COR) rose about 0.0025 between 2018 and 2019 balls, "corresponds to an increase in exit velocity of about 0.3 mph" (so about 1.2 mph per 0.01 of ball CCOR at fixed bat), and the Fig. 13 axis spans 0.46–0.51 for the means of 6 or 12 balls. The committee's conclusion in both reports is no meaningful COR change; the changes were in drag and launch angles. **Not found or not opened:** the 2018 report (over the fetch limit), any 2021+ ball statement (humidor, "deadened ball") from Nathan or Smith, and MLB's published COR spec after 2019.

**Statcast and 1.23 / 0.23.** SOURCED: Powers & Yurko, "Swinging, Fast and Slow", arXiv:2507.01238 (2025, §2.2 Eq. 1, read): squared-up = batted-ball speed > 80% × {1.23 · (bat speed) + 0.23 · (pitch speed)}, "a rough approximation for 80% of the theoretical maximum ... given bat speed and pitch speed at contact, with additional assumptions about the mass of bat and ball (Nathan, 2000)". They report batted balls "as high as 110% of this rough theoretical maximum". Savant's "Statcast Metrics Context" (https://baseballsavant.mlb.com/statcast-metrics-context, read via fetch): squared-up is "at least 80% of the potential exit velocity (based on swing speed and pitch speed)"; "Bat speed is measured at the sweet-spot of the bat." (1.23, 0.23) = (1 + q, q) with q = 0.23 by the lab formula. **I found no source for Tom Tango having devised it; Powers & Yurko cite Nathan 2000.** I could not open the Nathan 2000 passage they mean (Nathan's text derives v_f for a pivoted or free bat but I did not find a numeric 0.23 in what I read; it is an ESTIMATE that the 0.23 is a typical wood-bat q with a rounded mass ratio).

## 3. Definitional traps

- **Where the bat speed is measured.** Lab formula: v_bat is the speed at the *point of impact* (BESR white paper: "This is the bat speed at the point of impact"). Certification convention: 66 mph measured 6 in from the barrel end whatever the impact point (same paper). Statcast: the speed of the sweet spot, about six inches from the end, at contact (Powers & Yurko; Savant). Nothing I read says what Statcast does for contact away from the sweet spot; that it reports the sweet-spot speed regardless is an inference from "measured at the sweet spot" and from the bat's 2D track (five 300 fps cameras, computer vision; no stated uncertainty).
- **Effect.** For a ball hit inboard of the sweet spot the bat meets it at less than the reported speed (ratio below the true q-equivalent); outboard, more. In both samples the ratio uses the sweet-spot speed, so the **bat-speed convention matches**: the sim's `ev_bat.rs` reports the speed at u_ss at the executed contact instant (§1). The real sample's contact-point mix is unknown; a real batter who tends to hit outboard of the sweet spot would raise the real ratio, one who hits inboard would lower it.
- **Pitch speed.** Powers & Yurko say "at contact"; Savant says nothing and the power-mix note used release speed. With the plate speed 8% down (89.5 → 82.3), the Statcast maximum falls from 108.7 to 106.9 mph and **both** ratios rise by 1.0 + 0.23·7.2/108.7 = 1.015: 0.958 → 0.972 and 0.934 → 0.948, a difference of 0.024 against 0.024. The 7–10% drop varies by pitch type (a changeup loses more than a four-seamer), and the real sample's pitch mix is not the sim's, so the exact shift is an ESTIMATE; **the gap survives**.
- **Ratio against q.** Because the 1.23/0.23 line is q = 0.23, convert a ratio ρ into an equivalent q via q_eff = (ρ·(1.23 b + 0.23 p) − b)/(b + p) (b = 71.6, p = pitch speed):

| | ratio | EV (89.5 release) | q_eff, p = 89.5 | q_eff, p = 82.5 |
|---|---|---|---|---|
| real p90 | 0.958 | 104.1 | **0.2017** | 0.2008 |
| sim p90 | 0.934 | 101.5 | **0.1855** | 0.1842 |
| sim ceiling (rigid arithmetic, §1) | 0.953 | 103.6 | 0.201 | 0.202 |

  The pitch convention moves q_eff by 0.001. **Real p90 and the sim's ceiling are the same number.** The real p90 is not the real ceiling (Powers & Yurko see 110% of 1.23/0.23 for some balls, which includes measurement error and contact away from the sweet spot). If the real p90 stands at the same fraction of its ceiling as the sim's does (101.5/103.6 = 0.98), the real ceiling is 104.1/0.98 = 106.2 mph, q = (106.2 − 71.6)/161.1 = **0.215** (ESTIMATE). That the sim's p90 sits just 2 mph under its ceiling comes from its contact distribution; a different real distribution would change this.
- **What must be re-measured to be sure:** the sim's contact-station distribution (the u of each contact, relative to u_ss) at the same squared-up ratios; the real sample's pitch speed at the plate (Savant's `effective_speed` or the sim's own), so both sides use the same term; and the real ratio's top tail by contact quality (Statcast's `launch_speed_angle` or `estimated_ba` do not give the contact point).

## 4. Other candidate causes for "same bat speed, 2 mph less"

Each with what the sources say and how to separate it using the data we have. None is excluded; q at the sweet spot is one of several.

1. **Ball-bat COR (this note's subject).** Lab BBCOR 0.49 against the sim's 0.44 at 29 in (§2). Test: run the sim's `ev_bat` with e_ref raised (a counterfactual, not done here) and see whether the p90 against bat speed table of power-mix §4 closes while the ratio's low end does not move. The sim's ceiling (103.6) is the check: the real p90 is the number to match.
2. **Ball COR and its drift.** 0.01 of CCOR is about 1.2 mph (2019 report). The ball in the real 2025 sample is the post-2021 ball; no opened source gives its COR against the 2018–2019 values. Only a ball COR change of ≈0.02 would explain 2.5 mph; the committee reports drifts of 0.0025 between seasons. ESTIMATE: ball COR drift is not the cause. Test: none from our data; ask Nathan's group or MLB/Rawlings numbers. Not resolved.
3. **Bat mass and moment of inertia.** Four 2024 MLB-pattern maple bats weighed 33.9–34.5 oz at 35.1–35.2 in with MOI 13,117–13,527 oz·in² about 6 in from the knob (torpedo paper Table 1; they are copies of a specific MLB player's bats, so n = 1 player). The sim bat is 34.0 oz, 34 in, 12,550 oz·in²: **mass and MOI are the same within 7%; length is 1 in short** (a longer bat has its sweet spot 1 in farther out; for a given swing speed measured at the sweet spot this is neutral). Cross & Nathan 2009 (not opened; the Illinois page's description: performance depends much more on MOI than on mass) and Nathan et al. 2011 ("A comparative study of baseball bat performance", Sports Eng. 13, 153; page description only) are the leads. Test: vary the bat's mass in the sim ±2 oz and see the p90 move; expected small (rigid r moves from 0.2015 by about 0.006 per 2 oz, q by 0.003).
4. **Where on the barrel contact lands.** The sim's q at 1 in off the sweet spot in either direction is about 0.02 (0.176 at 29 in, 0.217 at 26.5 in with the bat speed 6% lower inboard), so the contact spread matters as much as the peak. Test: histogram of u at contact in the sim (`cp.u`, available in the contact record) against the sourced width (the real region within 0.01 of the peak is 2.4 in, §2). The real distribution of contact location is not measured in anything I read (Statcast does not publish it).
5. **The bat's speed at the contact point against the sweet-spot speed.** A real batter whose contact falls at the sweet spot or beyond has a higher true bat speed than inboard contact. The sim's rigid-pivot scaling (§1) suggests the sim's contact inboard of u_ss loses 6% of bat speed per 1.6 in. Test: tabulate the sim's contact speed at u against its reported sweet-spot speed; the real equivalent is not available.
6. **Attack angle against the pitch's descent (off-centre loss).** Nothing in the lab sources addresses this (their impacts are normal to the bat axis). The sim's oblique collision is covered by the 2026-11-14 collision notes; a vertical misalignment of a few degrees costs energy to spin through e_t and friction. Test: split the sim's squared-up ratio by the angle between the ball's incoming velocity and the contact normal; compare with ev-la-bins §2 by launch angle (the real tail is flatter in launch angle).
7. **Air density and park altitude.** The collision does not depend on air density; the flight after it does (drag), by a fraction of a mph in the first 30 ft. Statcast reports exit speed at the radar's first track point a few feet from the bat (not opened; unverified). Real parks span sea level to 5,200 ft (Coors, where drag is about 18% lower, ESTIMATE); the effect on measured exit speed is below 0.2 mph. Test: split the real sample by park; one Coors date is enough. ESTIMATE: not the cause.
8. **Statcast measurement.** Exit speed is measured at contact, not after drag (the tracking radar's first-point reading, unverified); bat speed has no published uncertainty (§3). Both samples (§3 of power-mix) share the real measurement error on the *real* side only; a bias of 1 mph in `bat_speed` would move the real ratio by 0.012.

## 5. Conclusion for the lead

**Best-supported value.** The sweet-spot collision efficiency of an MLB wood bat with the current ball at game speed is **q ≈ 0.215, range 0.21–0.23**.
- SOURCED floor: lab q of 0.214 and 0.227 (110 mph, 30.6 and 31.1 oz Rawlings 34-in wood bats; arXiv:1009.2549) and 0.228 (best MLB wood bat of 1999; BESR). These are at lab speeds below game speed.
- ESTIMATE of the game-speed decline: 0.00 to −0.02 (the ball's COR falls with speed; no opened source quantifies the bat's BBCOR at 160 mph).
- ESTIMATE from the real data: p90 equivalent q 0.20, ceiling 0.215 if the real contact distribution looks like the sim's (§3).
- Statcast's own definition implies 0.23, a rough approximation by its authors.

**Is 0.20 inside it?** At its lower edge, below the lab values and below the real-data estimate; it is the number the spec took from "q ≈ 0.2 for wood at the sweet spot (Nathan)" without a measured game-speed BBCOR. The case for leaving it is only that its mass term is right and its COR then needs speed extrapolation nobody has measured.

**A sourced change** would be: raise the sweet-spot COR so q = 0.215 → `BatBallParams::e_ref` (`bat.rs:203`) from 0.442 to about **0.460** (e = q(1 + r) + r with r = 0.2015; 0.466 for q = 0.22, 0.478 for q = 0.23, the counterfactual of swing-model §6). It would also touch: A8 and the oracle goldens (spec §10, "q = 0.20 ± 0.01"; `analysis/oracle/golden`), spec §8.2's e_peak row, the PLAN D19 sentence, and the `s_e` and `v_ref` entries if the speed slope is re-sourced (a flatter slope would be consistent with Nathan 2000's "weak dependence" and would raise e at game speed too). `f_tip`, `f_taper` and the hit location model would not change. It does not touch the oblique-collision parameters (e_t, μ).

**Do not change q alone.** The 2026-11-14 swing-model §6 counterfactual (q = 0.23 via e_ref 0.478) gave HR 2.51% but mean exit speed 93.1 mph and hard-hit 54.7% against MLB 89.5 and 40.7%. Power-mix §4 shows the sim's mean exit speed (89.6) is already above the real sample's (88.8) because the real sample has **more poor contact** (63% of real balls at ratio ≥ 0.8 against 70% in the sim) and a better best contact. A sourced q raise therefore needs a matching look at the low end, i.e. contact location, the barrel's f(u) falloff (its tip and inboard sides), and the bat speed at contact: sourcing supports a higher *peak*, not a higher *average*. I would take the peak (q ≈ 0.215) and the spread as one decision.

**How sure.** Direction and size of the gap: fairly sure (two independent routes, lab and real p90, land between 0.20 and 0.23 against the sim's 0.200 at the sweet spot, and the real p90 equalling the sim's ceiling is a direct count). The value to the third digit: no. The game-speed BBCOR is the missing measurement.

**What I could not find or open.**
- Nathan 2003, "Characterizing the performance of baseball bats", Am. J. Phys. 71, 134 (cited in the corked-bat paper and on Nathan's page; the PDF was not opened: its q and the "prescription for bat speed" I know only from the corked-bat paper's Eq. 3, v_bat = 70 (I₀/I_knob)ⁿ mph).
- Nathan et al. 2011 (comparative bat study), Cross & Nathan 2009, Smith 2008 (BBCOR measurement), Smith, Nathan & Duris 2010 (the speed extrapolation), Hendee et al., Chauvin & Carlson: cited or described on pages I read, not opened.
- Any BBCOR measured at 150–165 mph for a wood bat; the post-2019 MLB ball COR; the 2018 committee report (over the size limit).
- The origin of 1.23/0.23 beyond Powers & Yurko's reference to Nathan 2000; no source connects it to Tom Tango. A Posnanski worked example (66.4 mph bat, 89.1 mph pitch, max "100.1 mph") does not equal 1.23 × 66.4 + 0.23 × 89.1 = 102.2 (the example appears to use another coefficient: 1.20 × 66.4 + 0.23 × 89.1 = 100.2, my fit, unverified); either the example or the coefficients as published differ from what the leaderboard uses, so the 80% test's exact line is **unverified**.
- Statcast's measurement point for exit speed and the uncertainty of `bat_speed`.
- Maple against ash: the sim's bat has ash-like density (660 kg/m³); the torpedo bats are maple, the Rawlings 34-in bats' wood is not stated.

**Licences.** Everything cited is a published paper, report or web page used for numbers only (reuse by citation); nothing was copied into the repo. The Illinois pages and arXiv papers carry no reuse licence I read; Baseball Savant data are the project's existing use. Driveline OpenBiomechanics data were not used.

## Resolution

Not reviewed by Codex: a sourcing note, no sim behaviour. Open until the owner rules on §5. Numbers for the lead's follow-up: (a) a sim counterfactual with `e_ref` 0.460 and with the contact-station histogram, (b) the real contact-point mix, which only a research partner could give.
