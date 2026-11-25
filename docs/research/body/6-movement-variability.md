# Body research 6: movement variability, how big an error in a muscle group's onset or strength can be (2026-11-25, worker body-research)

For [PLAN D31 / §14.1](../../../PLAN.md) item 4 (errors: which link each error enters, which rating sets its size). Reading status: **FT** full text, **TAB** a table in an open full text, **AB** abstract only, **2nd** from another paper. SI units.

**The headline.** Published numbers are for the **outcomes** of skilled movements (release windows, release points, contact-depth spread) and for **windows of tolerance** the movement can bear, but **no study I found reports the trial-to-trial standard deviation of a muscle group's onset time or of its peak torque in hitting or pitching**. The nearest are timing windows from simulations of throwing and the timing precision of the release itself. An error model for onset or strength therefore has to be **sized by working back from the outcome spreads**, with the sensitivities of note 1's models.

## 1. Signal-dependent noise (strength errors)

- Harris & Wolpert 1998, Nature 394:780 (**AB**): neural control signals carry noise whose **variance grows with the size of the control signal**; trajectories are chosen to minimise the variance of the end position. This explains smooth, symmetric velocity profiles and Fitts's law. The abstract gives no coefficient.
- Jones, Hamilton & Wolpert 2002, J Neurophysiol 88:1533 (**AB**): in voluntary isometric contractions of the thumb extensor, **force SD scales linearly with mean force**; electrically stimulated contractions to the same forces showed constant variability. The authors conclude the linear scaling is a by-product of the motor-unit pool's organisation (orderly recruitment by twitch size), **not** noise in the motor command; the *magnitude* of the variability at a given force comes from synaptic noise and common drive. No coefficient of variation is in the abstract (so none is available to me for a muscle group in a throw).
- For the design: a strength error as **multiplicative noise** (SD proportional to the drive) is the supported form; the proportionality constant is **not found**.

## 2. Timing windows and timing precision

| Source | Task, subjects | Number | Read |
| :-- | :-- | :-- | :-- |
| Chowdhary & Challis 1999, J Theor Biol 201:219 | Simulated overarm throws in the sagittal plane, a **muscle-actuated two-segment model** (forearm; hand plus projectile) with parameters specific to two male subjects; target 20 cm across at 6 or 8 m | At the optimal solution for the 8 m target: proximal-distal delay between wrist-flexor and elbow-extensor onset **49 ms**, release **83.4 ms** after onset; **launch window 1.2 ms** (the time in which release can occur and still hit); the window for the **wrist-flexor activation** **10.4 ms** with release at the planned time. A control scheme that needed the least precision (delay 56 ms, release 89.4 ms) had windows of **4 ms in the delay and 2.4 ms in release** when both vary together | **AB** |
| Jegede, Watts, Stitt, Hore 2005, J Sports Sci 23:805 | 9 unskilled + 6 skilled throwers, 30 fast accurate overarm throws each, search coils 1000 Hz, radar | Mean **timing windows for ball release**: **28 ms (unskilled), 7 ms (skilled)**. Release timing explained ball-speed differences in the unskilled only; in skilled throwers ball speed followed peak forearm angular velocity | **AB** |
| Hore & Watts 2011, J Neurophysiol 106:2024 | skilled throwers, finger and hand angles by search coil | Release timing precision "as low as **1 ms**"; without a ball in the hand, **they lose the 1–2 ms precision** (the ball's back-force on the fingers does the timing) | **AB** |
| Nakashima, Horiuchi, Kimura, Sakurai 2025, Front Sports Act Living 7:1557145 (CC BY) | 18 pitched-ball trajectories of 10 collegiate pitchers + 145 bat-swing trajectories of 29 collegiate batters, collected separately | **Acceptable range of timing error** for contact at the bat's sweet spot: **9.36 ± 6.25 ms** (0.227 ± 0.163 m along the ball's path); across swings **2.48–30.40 ms** (0.056–0.614 m), depending on how well the bat's path matches the ball's path seen from the side and above, and on bat angle at impact | **AB** (the full text is open; I read the abstract) |

How to read them together: the **skilled thrower's release window of 7 ms and a batter's sweet-spot window of about 9 ms (SD 6; the range over swings 2.5–30 ms)** are the tolerances; the athletes have to be **more precise than their tolerance** to succeed, and Hore's 1 ms release timing says the *fingers* achieve that, not a central clock. Chowdhary & Challis's windows (1–10 ms) are for a two-segment model without feedback, so they are **smaller than skilled throwers' real precision** and set the order of magnitude: an **onset error of a few ms in the late links** matters; the early, slow links tolerate tens of ms.

## 3. Variability of outcomes (what an error model must reproduce)

### 3.1 Hitting

[Note 10 of the swing set](../swing/10-swing-length-by-bat-speed.md) (8,269 swings of 255+ batters, Statcast bat tracking, deviations from each batter's own means): **within-batter SD of bat speed 4.3 mph (1.9 m/s), swing length 0.75 ft (0.23 m), contact depth 10.5 in (0.27 m)**. In time, 0.27 m of contact depth at a 40 m/s pitch (my arithmetic, assuming every depth change were timing) is about **6.7 ms**: of the same size as the batter's tolerance of the previous section, and the same size as the skilled thrower's release window. Fortenbaugh 2011 ([4-miami-study.md](../swing/4-miami-study.md)) gives SDs in its Tables 1–3 for the middle-fastball swings of 33 batters; the note does not say whether they are within- or between-batter, so they are not used here.

Koike's lower-body simulation ([note 1](1-forward-dynamics-precedents.md)): moving 175 torque-timing parameters by up to **±50 ms** changed peak lower-trunk speed by **+25 %**. That is the sensitivity to a large, *deliberate* timing change, not a variability measurement.

### 3.2 Pitching

Wakamiya et al. 2024, Front Sports Act Living 6:1447665 (CC BY; **TAB**): **344 MLB and 64 MiLB starters, 2021–2023, Statcast release-point data, 300,884 MLB four-seam fastballs**; variability = the **width of the 95 % confidence ellipse** of the release points of each pitcher (so the width of a 95 % region, not an SD; if the points are bivariate normal the width is about 4.9 σ, my arithmetic). Four-seam, MLB against MiLB:

| Measure | MLB | MiLB | Approximate σ (MLB, my conversion) |
| :-- | --: | --: | --: |
| Horizontal release-point width (RPX) | 30.6 ± 12.3 cm | 35.2 ± 16.2 cm | **6.3 cm** |
| Vertical release-point width, coronal plane (RPZ) | 15.2 ± 2.5 cm | 17.5 ± 3.4 cm | **3.1 cm** |
| Release extension width (RPEx) | 25.4 ± 4.6 cm | 24.6 ± 4.6 cm | 5.2 cm |

Breaking balls are wider (RPX 35.4 vs 39.5 cm). **Lower horizontal release-point variability goes with more strikeouts and fewer home runs**; xFIP improved by about 0.16 for each 1 cm decrease in RPX width. MLB spread is **13 % smaller horizontally and 13 % smaller vertically** than MiLB. The measurement error of the release point is about 1 cm (cited).

Kusafuka et al. 2023, PLoS One 18:e0290042 (**AB**): reducing the variability of the **ball's velocity angle at release** reduces the spread of pitch location; pitchers with low release-angle variability had a more consistent lower-middle-finger position relative to the ball before the acceleration peak. No angle SDs in the abstract.

Hore's and Jegede's release timing: **7 ms (skilled) release window; 1 ms precision of the fingers** (above). A release-time error moves the release point by (hand speed × error): at an assumed 30 m/s (my assumption; no hand speed is in these sources) 1 ms is 3 cm along the hand's path, of the order of Wakamiya's σ (3–6 cm). No source makes that link; it is my arithmetic.

## 4. Models that put the noise in the drive

- **Felton 2023** ([note 1](1-forward-dynamics-precedents.md)): the optimiser's movement in 88–98 activation timings is **not** an error model, but the sensitivity is: **+5 % strength gave only about +1 % ball speed** for one bowler (**2nd**, ISBS version), while timings alone gave +10 % (AB/2nd).
- **Chowdhary & Challis** above is the only sensitivity of the *outcome to onset times* of individual muscle groups (49 ms optimal proximal-distal delay; windows of 1–10 ms) that I found for an overarm throw.
- Attias et al. 2026, Kageyama 2014 and the others compare groups; none tests noise in drive.

## 5. What this gives the design

1. **Error sizes cannot be taken from a published onset-time SD.** They must be **back-solved**: choose onset and strength noise for each link so that the **release-point σ (3–6 cm), the release window (~7 ms) and the batter's contact-depth σ (0.27 m, ≈ 7 ms) and bat-speed σ (1.9 m/s)** come out; each of these four is sourced.
2. **Multiplicative strength noise** (SD ∝ drive) is the supported form; its constant is **unfound**.
3. **Late links need the tightest timing:** a thrower's last links have 1–10 ms windows (Chowdhary & Challis) and the fingers do the 1 ms part through the ball's back-force; early links (stride, trunk) tolerate tens of ms (Koike: ±50 ms changes in leg torque timing moved trunk speed 25 %).
4. **A batter's tolerance is 9 ± 6 ms (2–30 ms) and depends on the swing path**: a model that has the bat's path as a *result* gets this from its path; this is a check for the batter's body model, not an input.
5. **MLB pitchers are about 13 % tighter than minor leaguers** in release point (horizontal and vertical): a sourced scale for how command might differ between levels.

## 6. Not found

- **Trial-to-trial SD of onset time, duration or peak torque of any joint's muscle group** in hitting or pitching; within-pitcher joint-angle SDs at release.
- The **coefficient of variation of force** for a throwing or swinging muscle group (only the form, linear in force).
- **Timing variability of sequential movements** in a swing or pitch (the interval between pelvis and trunk peaks, or foot-down to swing start) as an SD in ms; Fortenbaugh 2011 has event times with between-subject SDs in the thesis tables, which I did not extract.
- A source linking the stride-foot timing of a batter to pitch speed and swing onset variability (a "timing of the swing against the pitch" error); the swing notes' own data ([4-miami-study.md](../swing/4-miami-study.md)) are the closest.
- Chowdhary & Challis's full text (windows for a second subject).
- Licence: Wakamiya 2024 and Nakashima 2025 are CC BY; Chowdhary & Challis and the others are cited from abstracts.
