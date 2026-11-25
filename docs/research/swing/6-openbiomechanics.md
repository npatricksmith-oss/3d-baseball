# Source 6: OpenBiomechanics Project (Driveline), baseball_hitting

> **LICENCE FLAG FOR THE OWNER (read first).** Data and the biomechanics documentation are **CC BY-NC-SA 4.0** (Attribution, **NonCommercial**, ShareAlike), copied exactly from `LICENSE-DATA.md`:
> - "All data and any derived works from The OpenBiomechanics Project are provided, unless otherwise noted, under an Attribution-NonCommercial-ShareAlike 4.0 International Creative Commons license."
> - NonCommercial: "You may not use the material for commercial purposes." ShareAlike: "If you remix, transform, or build upon the material, you must distribute your contributions under the same license as the original." Attribution required, with link to licence and note of changes.
> - Extra exclusion: "Any employee or contractor employed by, associated with, or a significant shareholder of a professional sports organization or financial analysis firm is forbidden to use The OpenBiomechanics Project data for any use whatsoever." Commercial use needs a paid written licence (see openbiomechanics.org, Contact Us). "Commercial purposes" is defined as including for-profit organizations, corporations and "sole proprietorships with the intent to profit now or in the future".
> - Code in the repo is MIT (`LICENSE-CODE.md`); that does not cover the data or the README/data dictionary.
> - **Decided (owner Q41, 2026-11-23): this dataset is used only as a cross-check.** The base swing's numbers come from sources 4 and 3; nothing here goes into shipped parameters, so no licence is sought. The synthesis shows these figures beside the chosen ones as "check", and lists what only this source gives ("OBP only").
> - Consequence: do not ship this data, or tables/curves derived from it, in a game that is or may become commercial without a licence. The summary figures below are my own statistics from the data and are derivative works under this licence. Treat them as research guidance only, and re-derive or license before baking any into shipped parameters. No dataset file or per-swing row is committed; the downloads sit in git-ignored `sim/target/research-dl/6/`.

## What it is
- Driveline Baseball R&D, OpenBiomechanics Project, GitHub `drivelineresearch/openbiomechanics`, release tag `dataset-v1` (data files dated Dec 2022). Fetched the shallow sparse clone (README, data dictionary, `DATASHEET.md`, POI + metadata CSVs) and release zips `hitting_joint_angles/joint_velos/landmarks/force_plate` (all fetched fine; C3D not downloaded).
- **Who/how:** 677 swings (672 have an exit velocity), **98 hitters**, one session each, ~7 swings each. Levels (swings/hitters): college 517/74, high school 83/12, MiLB 47/8, independent 30/4. 496 right-handed swings, 181 left-handed. Age 17-31 (median 20). Marker-based mocap (360 Hz) with ten-marker instrumented bat, 4 embedded force plates (1,080 Hz, rear/lead), hitters hit **off a pitching machine at about 65 mph from about 40 ft** (README). Exit velocity from HitTrax (median 91.0 mph = 40.7 m/s).
- Processing: Visual3D hybrid model; joint angles "adjusted to better match coach/player intuition (righty/lefty symmetry, sign negations, adding or subtracting 90 degrees)". So signs are conventions, not raw anatomy; see below.
- Trust: high for kinematic magnitudes and timing of a lab swing against a slow machine pitch. Caveats: machine pitch at about 65 mph with no pitcher cue (timing and stride behave differently from live MLB pitching), lab setting, mostly college hitters (76% of swings), few MiLB (8 hitters). Not MLB-only. Statistics below are over swings (hitters pooled), with a hitter-mean check; swings are not independent (about 7 per hitter), so the SD of swings is not between-hitter SD (hitter-mean SD given where it differs).
- 27 swings have no event times (650 usable for timing); POI has all 677.

## Events (the source's definitions)
- **first move (fm)**; **loaded position**; **fp_10 / "front foot contact"** = lead-foot vertical force reaches 10% bodyweight; **fp_100 / "front foot plant"** = 100% bodyweight on the lead foot; **contact** (bat-ball); "down swing" event (bat-torso angle); "max hip-shoulder separation (hss)"; "heel strike (hs)". Angle POIs are labelled "launchpos" = at foot plant.
- Time series window: starts about 1.5 s (SD 0.4) before contact (the take starts well before the stride) and runs about 0.43 s (SD 0.14) after contact; there is no end-of-finish event, no head marker, no bat-release event.

## Timing (my computation from the time series, event-aligned; ms; SI = s/1000)
| Quantity | mean | SD | p10 | p50 | p90 |
|---|---|---|---|---|---|
| fp_10 (lead foot touches, 10% BW) to contact | 200 | 104 | 147 | 175 | 300 |
| fp_100 (foot plant) to contact | 122 | 33 | 100 | 128 | 147 |
| fp_10 to fp_100 (heel strike to full weight) | 77 | 97 | 28 | 44 | 178 |
| Peak pelvis axial angular velocity, relative to contact | -75 | 23 | -98 | -75 | -50 |
| Peak torso axial angular velocity | -70 | 10 | -83 | -72 | -58 |
| Peak lead-shoulder (global) angular speed magnitude | -49 | 27 | -67 | -56 | -30 |
| Peak sweet-spot (barrel) speed (finite difference of marker) | -2 | 5 | -8 | -1 | 0 |
| Maximum hip-shoulder separation | -123 | 28 | -142 | -125 | -106 |
- Peak lead-hand and rear-shoulder (global) angular speeds sit at or after contact (median +42 and -64 ms, window censored at +50 ms): not reliably timed. Sample: n=649-650.
- Order: **pelvis peak, torso peak (mean 4 ms later; the pelvis speed is a plateau, within 1% between -100 and -50 ms), lead shoulder (about 20 ms later again), hands/bat at contact.** Peak separation comes about 50 ms before the pelvis/torso speed peaks.

## Stance / setup (time-series frame 0 = start of the take window, not a defined "stance"; weak)
- Stance ankle separation at first frame (joint centres, m): 0.68 SD 0.11 (p10 0.55, p90 0.82) = 0.68 m = 2.2 ft; almost all of it along the line to the pitcher (x 0.66 m, lateral 0.12 m): a nearly square-to-pitcher parallel stance as the lab frame sees it. Hitters' foot placements are measured in the lab frame, not relative to the plate.
- Centre of mass height 1.02 m SD 0.06 (hitter height not analysed).
- Knee flexion at frame 0: lead 38 deg SD 12, rear 46 SD 12; hip flexion lead 31, rear 45 (deg, source sign: +flexion). Elbow flexion at frame 0: lead 91, rear 129. These are the take posture, not a set stance.
- Pelvis/torso axial at frame 0 about -93/-94 deg (the source's 0 = pelvis/torso facing the pitcher; negative = turned away toward the catcher); torso-pelvis about -3. Pelvis tilt x: -17, torso x: -12 (source sign).
- Weight split at start: rear plate 0.67 BW (SD 0.25; p10 0.38, p50 0.60, p90 1.01), lead plate 0.33 BW (total 1.00 BW). **Not interpretable as a rear/lead weight split**: README says hitters start with one or both feet on the back plate; the lead foot often sits on the same plate.
- Bat angle/head/eyes/hand position at setup: not covered.

## Load, stride, foot plant
- Load = rotation back toward catcher: loaded-position pelvis axial about -101 deg (SD 8), torso -100 (SD 8) in the source frame; torso-pelvis at load -3.7 deg (SD 5): load turns the pelvis and torso together (about 10 deg past the take-start rotation), little separation at load. At -300 ms: pelvis -114, torso -111 (SD 8, 7).
- Centre of mass first moves **back** (min x displacement -0.12 m, SD 0.05) before fp_10, then forward: **+0.17 m (SD 0.09) from start to foot plant, +0.23 m (SD 0.10) from start to contact**; peak COG velocity (POI, whole take) 1.06 m/s SD 0.20 (p10 0.81, p90 1.31). COM drops 0.07 m (SD 0.05) from start to contact.
- Stride: lead ankle-joint-centre horizontal travel from first frame to plant: **0.34 m** SD 0.13 (p10 0.17, p90 0.52); component toward the mound 0.27 m SD 0.14 (p10 0.08, p90 0.45). Includes any pre-load foot movement, so it overstates the stride from a fixed set stance; ankle separation grows from 0.68 m to **0.89 m** at plant (SD 0.09) = a stride of about 0.23 m beyond stance width. At plant the feet are about 0.07 m (SD 0.05) apart laterally: **a straight stride, lead foot toward the pitcher, not open**. Rear ankle moves 0.05 m (SD 0.03). Lead ankle lifts 0.22 m (SD 0.09, p90 0.34) max before plant. Stride length as % height: not given.
- Lead knee: flexed 83 deg (SD 16; p10 63, p90 104) max during stride (the lead knee is raised high), then **46 deg flexed at plant** (SD 10; POI 45, SD 9). Rear knee flexion 62 deg at plant (SD 11); lead hip flexion 62 (SD 9); rear hip 50 (SD 15). Lead ankle dorsiflexion-ish 86 deg at plant (source sign).
- At plant (source frame): pelvis axial -74 deg (SD 19), torso -94 (SD 21): **the pelvis has already opened about 25 deg from -98 at heel contact; torso about 14 deg (-108 to -94)**. Torso-pelvis separation at plant **-19 deg** (SD 7.5; p10 -28, p90 -9) in POI (x_factor_fp_z). Hitters with larger separation at plant: bat speed r = -0.20 (more negative = more separation, slightly faster). Torso tilt (lateral, toward rear leg) 4 deg at plant, 32 deg at contact (SD 8).
- Pelvis angular velocity at plant 566 deg/s (9.9 rad/s; SD 158), torso 643 (11.2 rad/s; SD 215), hand speed magnitude at plant 833 (SD 317; unit mislabelled in the dictionary as deg/s; the hand-speed definition unclear). So **rotation is well under way at plant, and the torso already moving at about 70% of its peak.**
- Rear/lead arms during stride: rear elbow 126 deg (SD 8) flexion at first move to 121 (SD 10) at plant; bat/hand details: not covered. Weight: rear share of vertical force at fp_10 0.80 (SD 0.08), i.e. 80/20 rear/lead as the lead foot touches; lead leg peak vertical force fp_10 to contact **2.0 BW (SD 0.4; p10 1.5, p90 2.6)** (bodyweight from metadata mass).

## Rotation and swing
- Peak axial angular speed (swing, load to contact), deg/s, rad/s beside:
  - **pelvis 705 deg/s (12.3 rad/s) SD 75**, p10 602, p50 714, p90 797
  - **torso 915 deg/s (16.0 rad/s) SD 80**, p10 806, p50 920, p90 1011 (hitter-mean SD 72)
  - **upper arm resultant 1205 deg/s (21.0 rad/s) SD 137**, p10 1018, p90 1398
  - lead-shoulder (global) resultant 1220 deg/s (21.3 rad/s) SD 143 (time series)
  - hand (global) resultant 2081 deg/s (36.3 rad/s) SD 332 (time series; peak at or after contact)
  - **Progression pelvis 705 < torso 915 < arm 1205 < hand 2081 deg/s**: a classic proximal-to-distal speed-up (each segment faster than the one before, ratios about 1.3, 1.3, 1.7).
- Pelvis axial angle (source frame) at -400/-200/-150/-100/-50/0/+50 ms from contact: -110 / -109 / -89 / -60 / -26 / +2 / +10 deg (mean). Torso: -107 / -113 / -106 / -82 / -38 / +3 / +20. So pelvis completes about 112 deg rotation in about 300 ms and about 60 deg in the last 100 ms; torso about 85 deg in the last 100 ms and keeps rotating after contact (+45 deg at +150 ms) while the pelvis slows (+14).
- Pelvis axial angular velocity (mean, deg/s) at -200 / -150 / -100 / -50 / 0 / +50 ms: 208 / 479 / 641 / 646 / 368 / -13. Torso: 58 / 345 / 788 / 853 / 484 / 168. Both are decelerating at contact (pelvis to about 50% of peak by contact, near zero 50 ms after).
- Separation trace (torso minus pelvis, source sign; magnitude): about 3 deg at load, **peak 21 deg (SD 6; p10 13, p90 29; POI max 21 SD 6) at -123 ms**, closes to 2 deg at contact and reverses (torso 5 deg ahead at +50 ms, 17 at +100).
- Pelvis tilt at contact +7 deg (SD 6) vs torso x +15 (SD 8) vs load -21/-26: both pitch from forward flexion toward upright, torso more (about 41 deg change load to contact); torso lateral tilt toward the rear leg goes -10 deg at -400 ms to 28 at -50 to 32 at contact.
- Lead leg: knee flexion 46 (plant) to 44 (-100 ms) to 32 (-50) to **14 at contact (SD 12)** to 8 at +50 ms; lead hip flexion 62 deg at plant to 31 at contact, 22 at +50; lead knee extension speed peaks about 350 deg/s (6.2 rad/s) at -50 ms. **The front leg firms up/extends into contact (blocking leg), about 30 deg of knee extension in the last 50 ms.**
- Rear leg: rear knee flexion 62 at plant to 72 at -50 ms (more bent, sits into the back leg) to 71 at contact; rear hip goes from 50 deg flexion at plant to -6 at contact (extension; hip drive).
- Arms: rear elbow flexion 127 at -400 ms, 109 at -50, **79 at contact (SD 13)**, 41 at +50, 35 at +100 (extends after contact); lead elbow flexion 76 (-400) to 58 (-50) to **53 at contact (SD 15)**, 37 at +50 (min) then flexes to 58 at +100, 73 at +150 (lead arm pulls in after extension). Rear shoulder abduction (y) 74 deg at -400 to 29 at -50/0 (arm adducts to the body); shoulder detail beyond this: not covered in source terms.
- Bat: bat speed at contact 66.2 mph (29.6 m/s) SD 4.8; max bat speed 67.1 mph (30.0 m/s) SD 4.9 (p10 60.8 = 27.2; p50 67.5 = 30.2; p90 73.0 = 32.6 m/s); Blast sensor 68.2 mph (30.5 m/s) SD 6.0. **Bat speed at contact 98% of max (the source's 66.2 against 67.1 mph); the bat peaks within ~2 ms of contact** (the timing table above: the note-writer's finite difference, −2 ms, SD 5). The source does not say where on the bat its bat speed is measured (added 2026-11-25, review P3-2) (my sweet-spot finite difference gives peak 32.9 m/s SD 2.6 = 73.7 mph and 31.1 m/s at contact; it exceeds the source's number by about 5-10% and is computed differently, so use the source's). Barrel speed at -100/-50/0/+25 ms (sweet spot, mine) 7.0 / 18.9 / 31.1 / 18.5 m/s: barrel speeds up about 12 m/s in the last 50 ms, drops sharply after contact (the ball).
- Attack angle at contact **+4.1 deg** SD 5.9 (p10 -3.3, p50 4.4, p90 10.7; hitter-mean SD 4.3). Barrel vertical velocity at contact about +4.8 mph (2.2 m/s) SD 6.7. Bat-torso angle at "down swing" ("early connection") 109 deg SD 14. Barrel height at contact 0.79 m SD 0.13 (pitch location varies). Bat lag/on-plane time: not given (no metric); path across the zone: not covered.
- Head: not covered (no head markers).

## Contact
- Lead knee 14 deg flexed (SD 12), front hip flexion 31, rear hip -6 (extended), pelvis rotated to about square (+2 deg, i.e. facing the pitcher in the source frame, SD 11), torso +3 (SD 12) i.e. also about square or slightly past, lateral torso tilt 32 deg toward the rear leg, rear elbow 79 deg and lead elbow 53 deg flexed (**neither arm is fully extended at contact; full extension comes about 50-100 ms later**), rear shoulder adducted. COM forward of its start by 0.23 m, lower by 0.07 m. Contact point vs plate/body: not given in these tables.
- Bat speed peak 66-67 mph with hitter spread 4.5; relation to exit velocity r = 0.71 (swing level).
- Bat speed by level (swings/means; max bat speed mph): high school 65.3 (SD 4.3, n 83), college 66.9 (5.0, 517), independent 70.5 (2.2, 30), MiLB 69.5 (2.7, 47) [29.2, 29.9, 31.5, 31.1 m/s]. Exit velocity 87.8 / 89.7 / 94.0 / 94.3 mph (39.3 / 40.1 / 42.0 / 42.2 m/s). Attack angle 3.5 / 3.9 / 7.6 / 5.1 deg. Peak pelvis 701 / 704 / 733 / 707 deg/s; torso 901 / 915 / 975 / 900 deg/s; upper arm 1148 / 1205 / 1350 / 1214 deg/s. Separation at plant -22 / -17 / -19 / -19 deg. **Pros (independent, MiLB) are faster by 3-4 mph in the bat but have nearly the same pelvis and torso peak speeds; the difference is mostly in the arms/hands and the bat** (n = 4 independent and 8 MiLB hitters, so weak).
- Correlation with max bat speed (swings, across all): hand speed 0.40, torso peak 0.26, upper arm 0.25, pelvis at plant 0.28, pelvis peak 0.17, separation at plant -0.20, attack angle 0.17, lead knee at plant 0.05. No single rotational number explains it (all |r| <= 0.4).

## Follow-through and finish
- Only the first about 0.43 s after contact is recorded. Torso keeps turning to +45 deg at +150 ms (pelvis +14, so the torso rotates about 30 deg past the pelvis), lead knee straightens to 8 deg at +50 ms then re-bends to 16 at +150; rear elbow extends to 35 deg by +100 ms; lead elbow flexes back to 73 by +150 (wrap). One-hand release, wrist roll, balance, and leaving the box: **not covered** (wrist angles exist in the full data, not analysed here; POI: lead wrist at first move 30 deg SD 17, swing extreme -11 deg SD 10).

## Numbers table (swing-level; SI beside original)
| Quantity | mean | SD | p10 / p50 / p90 | n swings (hitters) | SI |
|---|---|---|---|---|---|
| Bat speed at contact | 66.2 mph | 4.8 | 60.0 / 66.6 / 72.0 | 677 (98) | 29.6 m/s |
| Max bat speed | 67.1 mph | 4.9 | 60.8 / 67.5 / 73.0 | 677 (98) | 30.0 m/s |
| Exit velocity (machine 65 mph pitch) | 89.9 mph | 7.6 | 80.2 / 91.0 / 98.2 | 672 (97) | 40.2 m/s |
| Attack angle at contact | 4.1 deg | 5.9 | -3.3 / 4.4 / 10.7 | 677 (98) | 0.072 rad |
| Peak pelvis axial speed | 705 deg/s | 75 | 602 / 714 / 797 | 677 (98) | 12.3 rad/s |
| Peak torso axial speed | 915 deg/s | 80 | 806 / 920 / 1011 | 677 (98) | 16.0 rad/s |
| Peak upper-arm speed | 1205 deg/s | 137 | 1018 / 1208 / 1398 | 677 (98) | 21.0 rad/s |
| Pelvis / torso speed at plant | 566 / 643 deg/s | 158 / 215 | 431 / 592 / 711; 387 / 690 / 849 | 677 (98) | 9.9 / 11.2 rad/s |
| Hip-shoulder separation at plant | 18.1 deg (torso behind pelvis) | 7.5 | 9.3 / 18.4 / 27.6 | 677 (98) | 0.32 rad |
| Peak separation | 21 deg | 6 | 13 / 21 / 29 | 649 | 0.37 rad |
| Lead knee flexion at plant | 45 deg | 9 | 34 / 45 / 57 | 677 (98) | 0.78 rad |
| Lead knee flexion at contact | 14 deg | 12 | not given | 650 | 0.25 rad |
| Stride (ankle travel) | 0.34 m | 0.13 | 0.17 / 0.33 / 0.52 | 650 | 1.1 ft |
| Foot plant to contact | 122 ms | 33 | 100 / 128 / 147 | 650 | 0.122 s |
| Heel strike (10% BW) to contact | 200 ms | 104 | 147 / 175 / 300 | 650 | 0.20 s |
| Peak COG speed | 1.06 m/s | 0.20 | 0.81 / 1.06 / 1.31 | 677 (98) | 3.5 ft/s |
| Peak lead-leg vertical force after plant | 2.0 BW | 0.4 | 1.5 / 2.0 / 2.6 | 636 | 2.0 x body weight |
Hitter-mean SDs (98 hitters) are about 5-10% smaller for most items (bat speed 4.5 mph, torso 72, pelvis 70 deg/s, attack angle 4.3 deg).

## Where it disagrees with coaching / expectation
- **The pelvis does not wait for the foot to plant**: pelvis is already at 566 deg/s (80% of peak) and 25 deg open at full plant; most rotation (about 110 deg pelvis) happens within about 300 ms of contact, plant to contact is only 122 ms.
- **Separation is small and early**: only 18-21 deg of lag (torso behind pelvis), peaking about 120 ms before contact, i.e. before the speed peaks; not 40-60 deg. Correlation with bat speed is weak (-0.20).
- **Pelvis and torso peak nearly together** (4 ms apart on average), not a long staggered gap; the sequence is mostly in speed amplitude (705 < 915 < 1205 < 2081 deg/s), with the peak-time gap pelvis-to-shoulder about 25 ms and to the barrel about 70 ms.
- Arms are **bent at contact** (rear elbow 79, lead 53 deg flexion); extension arrives after contact. Lead knee extends (about 14 deg flexed) into contact. Stride is **straight (lateral 0.07 m)** and small relative to the stance (about 0.23 m beyond width).
- Pros differ from college mostly in bat speed and arm speed, not in pelvis/torso speed.
- Slow machine pitch (65 mph): timing from foot plant to contact (122 ms) is likely longer than against 90+ mph live pitching; do not use as an MLB number without checking.

## Not covered / not given
Stance width relative to shoulders or plate, set-up foot angle, weight split (see caveat), hand and bat set position, head/eyes, lead-arm and wrist roll at finish, one/two-hand release, finish balance, bat lag, on-plane time, contact position relative to plate (HitTrax point-of-impact exists in hittrax.csv; not analysed), foot rotation/pivot angle, hand-speed unit (dictionary says deg/s; values look unreliable as a unit).
