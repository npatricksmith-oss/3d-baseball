# Body research 2: hitting, joint torques, powers and ground forces (2026-11-25, worker body-research)

For [PLAN D31 / §14.1](../../../PLAN.md) item 2 (the drive). Reading status on every source: **FT** = full text read, **FIG** = a number read off a figure in a PDF I rendered (accuracy about ±10 % of the axis, said where used), **AB** = abstract only (PubMed or publisher), **2nd** = a later paper's description.

**The headline:** the open literature on the swing gives **almost no peak joint torques in N·m as numbers in a table or abstract**. The Tsukuba group (Koike, Ae and co-workers) and the Japan Institute of Sports Sciences (Horiuchi, Nakashima, Morishita, Yanaka) computed them by inverse dynamics, but their papers state torques as figures, as r-values, or as percentage contributions, and I could open the full text of only the conference versions. What numbers could be read are in §2, marked.

## 1. Ground reaction forces under each foot

Already in hand and not repeated: Fortenbaugh 2011 ([4-miami-study.md](../swing/4-miami-study.md): 33 AA pros, **live** pitching, 300 Hz mocap, force plates; lead foot vertical force rises 50 → **130 % BW at −93 ms**, lead braking force (Fx) peaks **50 % BW at −83 ms**, lateral (Fy) peak 28 % BW at −123 ms, trail Fy peak 24 % BW at −127 ms; trail vertical falls 47 → 34 % BW; ball-location effects of 143–149 vs 138 % BW) and Welch 1995 ([3-welch-1995.md](../swing/3-welch-1995.md): 7 pros off a tee; lead foot 123 % BW (1007 ± 472 N) at foot down, 84 % BW (709 ± 270 N) at contact; back foot 102 % BW at foot off, 58 % at foot down, 16 % at contact).

What else exists:

| Source | Subjects, task | What it gives | Read |
| :-- | :-- | :-- | :-- |
| Fortenbaugh & Fleisig, "Ballpark forces", ISBS 2008 (Seoul) | 9 college players, **tee**, 9 strike-zone sub-zones, two Kistler plates at 1250 Hz | A timeline figure of the average swing's GRF (% BW) at initial rock-back, forward drive and contact (labels such as FxFL −1150…−18 are axis-end tick labels, not values). Ball location changed forces by **no more than 5 % BW (about 50 N)**; times did not differ with location | **FT** (one page; the figure's numbers are not legible in the extract) |
| Hashimoto, La Salle, Takata, Aguinaldo, Wasserberger, ISBS 2023 (Milwaukee) | **11 professionals** (22.7 ± 2.9 y, 1.82 ± 0.06 m, 88.3 ± 7.0 kg), **pitched balls at 65 mph from 40 ft**, 47 markers, force plates, Blast sensor on the knob | **Back-leg peak vertical GRF 886.5 ± 90.5 N** (that is 1.02 BW at the group's mean mass: my arithmetic, the paper does not say it); back-leg medial-lateral impulse −11.2 ± 22.7 (printed unit "N/s"; it must be N·s, the table's unit looks wrong); bat speed **32.3 ± 1.5 m/s** (72.2 ± 3.3 mph, sensor on the knob, so below the barrel's speed) | **FT** |
| Takata & Aguinaldo, ISBS 2022 (Liverpool) | 9 NCAA D-II players, **tee**, 33 swings, 240 Hz mocap, 1200 Hz plates | Bat speed 30.4 ± 1.3 m/s; predictors (back-leg vertical impulse, lead-leg propulsive impulse, L5S1 absorption and transfer in the acceleration phase, lead-hip absorption in follow-through; r² = 0.597) | **FT** |
| Ae, Koike et al., Sports Biomech 2017 (Kinetic analysis of the lower limbs in baseball tee batting) | **22 collegiate** males, tee, 250 Hz, 3 plates at 1000 Hz | The lateral/medial GRF of both legs produces the large whole-body moment about the vertical axis; **hip flexion/extension (both), stride-hip adduction and stride-knee extension** torques did the most mechanical work | **AB** |
| Ae et al., Hum Mov Sci 2018 (tee against a pitching machine) | **10 collegiate**, machine at **33.3 m/s** | Lower-trunk angular velocity was **larger off the tee**; swing time (stride foot down to impact) was **significantly longer against the machine**; angular impulses of bilateral hip adduction, pivot-hip external rotation and stride-hip and knee extension torques were **larger against the machine**, i.e. batters "exert these joint torques earlier" for a pitched ball | **AB** |
| Ae, Koike, Kawamura, Sports Biomech 2020 (hitting-point height) | 22 collegiate, low/middle/high tee | Hip abduction/adduction torques change with ball height; low ball: lower trunk rotates slower | **AB** |
| Horiuchi & Nakashima, Sports Biomech 2024 | 30 collegiate, tee, 2 plates | Horizontal GRF of the **stride foot** correlates with energy flowing into the lower torso from the stride hip; the **pivot** foot's does not | **AB** |
| Horiuchi, Sports Biomech 2024 (toss batting, and "torso dynamics during follow-through") | collegiate | Mechanical energy flow in the torso; follow-through is a braking phase for the torso | **AB** (titles and listings only for the follow-through paper) |

What these say together: the stride (lead) leg is a brake and a rotation axis (large horizontal and vertical force, hip adduction and knee extension torque), the back leg pushes and rotates the pelvis early (hip extension), and the **timing** of these torques moves with the pitch (earlier against a machine than off a tee).

## 2. Joint torques, powers and the hands' force on the bat

### 2.1 What the hands do to the bat (an instrumented bat)

Koike, Kimura, Kawamura, Fujii, Takahashi, Ae, "Kinetics of the upper extremities during baseball batting", ISBS/ISB 2003 long abstract (**FT**; numbers marked FIG are read off the rendered plots, ±10 % of the axis):

- **5 varsity players** (78/68/75/79/72 kg; 1.82/1.69/1.76/1.77/1.75 m), **tee at hip height**, hit "as strong as they could", 6–9 good trials each. VICON 120 Hz, ball by 250 Hz video, instrumented bat (strain-gauge bar between two grip handles, 500 Hz).
- The swing is split into three phases by normalised time: 1 stride (to 100 %), 2 down-swing (100–200 %: stride toe down to the bat parallel to the pitcher's line), 3 level swing (200–300 %: to impact). Forces in the bat's swing-plane axes.
- **Bottom (knob-side) hand: the force along the bat's axis (Fy) rises smoothly to about 750 N at impact** (FIG; SD band about ±70 N; from ~300 N at the start of the level swing). Other force components stay small (|Fx| ≈ 60 N, Fz ≈ −100 N, FIG). Its moment about the bat's long axis (Nz) reaches about **−20 N·m** at impact (FIG, off the panel's edge, so a lower bound).
- **Top hand:** forces are small in the same frame; the two hands' x and z forces are roughly equal and opposite, i.e. a **couple**, while only the bottom hand pulls along the bat. Authors: the bottom arm accelerates the bat, the top arm "modifies" the bat's motion.
- **Shoulder abduction torque of the bottom arm: mean about 190 N·m at impact** (FIG; SD band about 110–260 N·m), rising from ~30 N·m at the start of the down-swing; its peak **torque power about +240 W** (FIG; band 100–340 W) near 200 % time. The top arm's shoulder torques stay within ±~30 N·m until the last 25 % of the swing and rise to about +80 N·m at impact (FIG). The text says the abduction torque keeps the bat close to the body against centrifugal force; flexion/extension torques are small because that axis lies along the bat.
- Caveats: five players, tee, a 120 Hz capture; the figure's torque axes are −100 to 300 N·m.

Morishita & Yanai (JSME 2014; **AB**, Japanese): in two tee experiments a single collegiate batter's force-couple analysis showed that the **couple moment** on the bat produced the bat's horizontal angle change and the moment of the resultant force opposed it; in experiment 1 the mean couple and moment-of-force magnitudes were **2.7 and 3.2 N·m (peaks 4.4 and 4.8 N·m)** with head speeds of 15.4–19.9 m/s (a slow swing; the snippet I read does not say which task).

### 2.2 What each hand's force and torque contribute to bat-head speed

Miyanishi & Sakurai, ISBS 2009 (**FT**): 8 varsity batters (19 ± 1 y, 1.71 ± 0.06 m, 71 ± 12 kg), **machine at 120 km/h (33.3 m/s)**, 2 cameras 250 Hz, a force-and-torque model (Willmott & Dapena). Bat-head speed at impact **36.5 ± 2.9 m/s** measured, **36.6 ± 2.8** summed from the model. Contributions: grip force **along the bat 54 ± 4 %**, grip torque **normal to the bat's axis 49 ± 13 %**, the normal force plus its torque about the bat's centre −12 ± 9 %, the bat's speed at stride-foot landing 4 ± 1 %, gravity 4 ± 1 %, the bat's angular momentum at landing 1 ± 1 %. High-speed group (38.6 ± 2.8 m/s) against low (34.6 ± 0.5): contributions did not differ; the normal-torque share (52 vs 47 %) was close to significant. Authors: speed differences are "greater effort", not different motion patterns. (Kinematic estimate of hand forces, not instrumented.)

### 2.3 Whole-body contribution analyses (Koike group)

Koike & Mimura, ISBS (year not printed in the text) and the JSME 2015 abstract (**FT**/**AB**): 12 collegiate males, tee, 16-segment model; the **motion-dependent term** (centrifugal, Coriolis, gyro) is the largest contributor to bat-head speed; the largest positive joint torque is the knob-side shoulder ab/adduction (JSME 2015: with **concentric components** of knob-side shoulder abduction torque and torso rotation torque the major contributors); knob-side elbow axial torque contributes **negatively**. Forward swing 0.20 ± 0.02 s. See [note 1](1-forward-dynamics-precedents.md) §1.3 for peak timings (torso rotation torque's contribution peaks at ~30 % of the forward swing, knob-side shoulder adduction at ~55 %, barrel-side shoulder rotation at ~80 %). Their numbers are plotted as bars (m/s) I did not read.

### 2.4 Trunk and pelvis rotational torque and power

- **Yanaka & Morishita, J Strength Cond Res, Nov 2026 (AB, only the abstract):** 27 batters, MRI of hip and trunk muscles plus inverse dynamics. Peak **pivot-hip torque** correlated with bat-head speed (**r = 0.59**) and peak **trunk rotational torque** with it (**r = 0.62**). Pivot hip torque: gluteus maximus CSA **r = 0.73**, hamstrings **r = 0.73**. Lead hip torque: psoas major r = −0.50, rectus femoris r = −0.60 (a negative sign with a hip flexion torque defined as the lead hip's pull). Trunk torque: pivot-side external oblique r = 0.50, lead-side internal oblique r = 0.50. **After controlling for body mass all of these correlations disappeared**: torque follows muscle size and body size. The paper gives torque magnitudes in its tables; I could not open them.
- **Koike group lower-body simulation** ([note 1](1-forward-dynamics-precedents.md) §1.3): peak lower-trunk angular velocity about the vertical axis **9.4 rad/s measured, 11.8 rad/s with re-timed torques** for one 74 kg, 1.75 m batter; torque magnitudes were *not* changed.
- **Energy flow** (Hashimoto 2023, Takata 2022, above): the back hip generates most energy near maximum trunk rotation and then absorbs; energy passes **L5/S1 into the pelvis** (not up the chain) as the pelvis rotates toward the pitcher; the **lead hip and lumbosacral joint absorb** energy after impact (deceleration by eccentric torque). The joint-power plots in these papers were not extractable as numbers.
- **Timing, not magnitude (Ae et al., two papers, AB, 23 collegiate males, tee at waist height, 10-segment angle-driven upper-body simulation driven by the standard motion):** the model reproduced measured bat-head speed with **0.19 m/s RMS and 0.98° bat-angle RMS**; re-timing the joint motions raised bat-head speed **35.6 → 40.0 m/s** (Sports Biomech 2024: more barrel-side shoulder abduction, knob-side elbow flexion, torso right lateral flexion near impact) or **35.6 → 39.2 m/s** with hand forces estimated too (J Appl Biomech 2022: earlier barrel-side elbow supination, wrist radial flexion and torso lateral flexion, later barrel-side shoulder abduction).

### 2.5 Bat speed seen from tests (not torque)

Tee swings (Koike's collegiate groups): bat-head speed 35.6–37.9 m/s ([note 1](1-forward-dynamics-precedents.md)); Tsukuba machine-pitch group 36.5 ± 2.9 m/s (above); Hashimoto's professionals 32.3 ± 1.5 m/s at the knob (not comparable).

## 3. Source differences to carry

- **Tee against live ball:** the same laboratory found different lower-limb torques and a longer swing time against a 33.3 m/s machine (Ae 2018, AB). Fortenbaugh's live-pitch times and Welch's tee times already differ in coil timing ([swing-synthesis](../swing/swing-synthesis.md)).
- **Where the "bat speed" is measured:** Blast-type sensors on the knob (Hashimoto, Takata) read lower than barrel-head speed from markers (Koike, Miyanishi). Do not mix them.
- **Force plates vs torques:** the Tsukuba figures give hand forces with a bat instrumented by strain gauges, which is as close to a direct measurement of the "force and couple the hands put on the bat" as the literature has: five subjects.

## 4. What this gives the design

1. **Ground-force targets are available and consistent across studies** (lead ≈ 1.2–1.5 BW vertical near contact, braking ≈ 0.5 BW, back leg ≈ 1 BW and falling): use [4-miami-study.md](../swing/4-miami-study.md) for the live-pitch swing.
2. **The hands' force and couple on the bat have one primary measurement** (Koike 2003, 5 players): a single hand pulls along the bat's axis to **~750 N** and the shoulder holds **~190 N·m** at impact, with the other hand giving a small couple. That is a check on the grip, not a drive.
3. **Torque magnitudes in N·m for hip and trunk rotation in the swing are not published in anything I could open**; what is published is the *ranking* and the *r-values* (torque scales with muscle and body size, r ≈ 0.5–0.7 with bat speed before mass control). They have to come from strength data (note 5) and the model's own fit.
4. **Timing is the lever** (note 1): re-timing alone moved simulated bat speed 10–12 % and lower-trunk speed 25 %.
5. **A pitched ball changes the timing of the lower-limb torques** (earlier) relative to a tee: use live-pitch data for timing.

## 5. Not found

- Peak **hip, knee, trunk and shoulder torques in N·m** (and joint powers in W) for the swing as tabulated values; only the plots above and the paywalled full texts (Ae 2017/2018/2020, Horiuchi 2021–2024, Yanaka 2026).
- GRF under each foot in **pitched-ball** swings of **professionals** as a table: Hashimoto reports one value (back-leg peak vertical 886.5 ± 90.5 N); Fortenbaugh 2011 is the one full live-pitch dataset.
- **Trunk torque about the vertical axis** between pelvis and torso (L5/S1) as N·m: not found as a number.
- Hand force on the bat for **live-pitched** swings of **professionals**: the only instrumented-bat data are tee swings of five varsity players (Koike 2003) and a 28-gauge handle in 12 college players (Koike & Mimura; figures).
- Licence: nothing copied at length; short figure-read values quoted for citation.
