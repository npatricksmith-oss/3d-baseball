# Body research 5: strength limits and how fast muscle drive rises (2026-11-25, worker body-research)

For [PLAN D31 / §14.1](../../../PLAN.md) item 2: the cap on each joint's drive and its fall with joint speed. Reading status: **FT** full text, **TAB** a table read from an open full text, **AB** abstract only, **2nd** a model paper's description of someone else's value. SI units; N·m and rad/s.

**The headline.** Measured maximum voluntary torques exist for the legs (fully modelled), and for shoulder rotation, wrist and trunk rotation as isolated numbers. **None of them reaches the joint speeds of a swing or a pitch** (isokinetic tests top out at about 4.7 rad/s for the shoulder and about 2 rad/s for the hip; the delivery's shoulder internal rotation reaches ~117 rad/s). Everything above the tested range is extrapolation. The late, fast joints (elbow, wrist, shoulder rotation at release) cannot be driven by a torque generator with a normal force–velocity curve, so their speed has to come from the motion of the segments before them and from elastic energy.

## 1. Lower limb: a full torque–angle–velocity model with parameters

Anderson, Madigan & Nussbaum 2007, J Biomech 40:3105–3113 (author manuscript PMC6820133). **FT/TAB.** 34 healthy active adults (14 aged 18–25, 14 aged 55–65, 6 over 65; both sexes), isometric, concentric and eccentric maximal voluntary contractions on an isokinetic dynamometer, six lower-limb exertions.

Model (their Eq. 9): `T(θ, ω) = C1 · cos(C2 (θ − C3)) · [2 C4 C5 + ω (C5 − 3 C4)] / [2 C4 C5 + ω (2 C5 − 4 C4)]` for concentric motion (ω ≥ 0); eccentric: the same curve with −ω times `(1 − C6 ω)`. `C1` is the maximum isometric torque, normalised by **body weight × height** (so in N·m divide by nothing and multiply by weight in N and height in m); `C2` sets the angle range, `C3` the angle of peak torque, `C4` and `C5` the angular velocities (rad/s) at which concentric torque has fallen to **75 % and 50 %** of isometric.

Male, age 18–25 means (SD in brackets) from Table 3:

| Exertion | C1 (× weight × height) | C4 = ω at 75 % (rad/s) | C5 = ω at 50 % (rad/s) | Example C1 for 95 kg, 1.88 m (my arithmetic) |
| :-- | --: | --: | --: | --: |
| Hip extension | 0.161 (0.049) | 1.578 (0.286) | 3.190 (0.586) | 282 N·m |
| Hip flexion | 0.113 (0.025) | 2.095 (0.489) | 4.267 (0.995) | 198 N·m |
| Knee extension | 0.163 (0.040) | 1.517 (0.593) | 3.952 (1.546) | 285 N·m |
| Knee flexion | 0.087 (0.015) | 2.008 (1.364) | 5.233 (3.554) | 152 N·m |
| Plantar flexion | 0.095 (0.022) | 0.987 (0.595) | 3.558 (2.144) | 166 N·m |
| Dorsiflexion | 0.033 (0.005) | 0.699 (0.108) | 1.940 (0.301) | 58 N·m |

Limits: **recreationally active young adults**, not athletes (athletes' C1 should be higher; the paper does not say by how much); the **model is fitted to isokinetic speeds** (the dynamometer's range was about ≤ 3–4 rad/s) and gives torque for higher speeds only by extrapolation; **no hip rotation, no hip abduction, no trunk or upper limb** in this paper. Torque at 50 % of isometric is reached at only about 3–5 rad/s; what joint speeds a stride-leg knee or hip reaches in a swing or pitch I did not source (Kageyama 2014 in [note 3](3-pitching-kinetics.md) gives stride-knee extension velocity only as a significance).

## 2. Hip and trunk rotation, the swing's engine

| Source | Subjects, task | Value | Read |
| :-- | :-- | :-- | :-- |
| "Asymmetry of the isokinetic trunk rotation strength of Korean male professional golf players", Ann Rehabil Med 2012;36:821–7 (PMC3546185; authors not recorded) | 51 male pro golfers (22.3 ± 4.3 y, 1.765 ± 0.053 m, 74.7 ± 7.9 kg) and 50 healthy controls, isokinetic trunk axial rotation | Peak torque, **aiming-side rotation 140.6 ± 30.9 N·m at 30°/s (0.52 rad/s), 127.4 ± 26.4 at 60°/s (1.05 rad/s), 117.0 ± 25.0 at 120°/s (2.09 rad/s)**; non-aiming side 131.8 ± 27.9, 117.7 ± 25.2, 111.0 ± 23.9. Torque falls **17 % from 30 to 120 °/s**. The authors note that golf swing speeds of about 200°/s cannot be tested reliably | **FT** (text and results) |
| Yanaka & Morishita, J Strength Cond Res 2026 (inverse dynamics in swings) | 27 baseball batters, MRI | Peak pivot-hip torque correlated with bat-head speed r = 0.59, trunk rotational torque r = 0.62 (bat-swing torques, not strength); the correlations **vanish after controlling for body mass**. Torque magnitudes were in tables I could not open | **AB** |
| Hip abduction/extension isometric, "Changes in hip range of motion and strength in collegiate baseball pitchers over a season", PMC4527197 | 14 D-I pitchers (19.4 ± 1.4 y, 1.89 ± 0.064 m, 96.1 ± 8.4 kg) | Pre-season isometric hip abduction about **107 ± 10.6 N·m (lead) and 109 ± 16.4 N·m (trail)**, lead-hip extension about **74.6 ± 10.2 N·m** (from a search excerpt of the table; the table was not opened) | **2nd** |

For comparison, the hitting inverse-dynamics numbers of [note 2](2-hitting-kinetics.md) are percentages and plots; the trunk rotation torque a batter exerts is not published as N·m in anything I could open.

## 3. Shoulder, elbow, forearm and wrist

| Joint action | Source | Value | Read |
| :-- | :-- | :-- | :-- |
| Shoulder internal / external rotation, arm abducted 90°, elbow 90°, in the scapular plane, **33 collegiate pitchers** (20.2 ± 1.4 y, 187.7 ± 6.5 cm, 90.7 ± 12.4 kg; fastball 37.5 m/s) | Trunt, Sturdevant, Adams, Skelley, MacFadden et al., "Clinical shoulder measurements related to joint loads in collegiate pitchers", PMC10426641 (Sanford Health; first-author order as listed on the page) (Biodex System 4 Pro; five maximal repetitions) | Dominant arm peak torque (N·m/kg): **IR concentric 0.73 ± 0.19 at 90°/s, 0.63 ± 0.16 at 180°/s, 0.57 ± 0.19 at 270°/s**; **IR eccentric 0.88 ± 0.22 at 90°/s**; ER concentric 0.46 ± 0.10, 0.39 ± 0.09, 0.33 ± 0.10; ER eccentric 0.48 ± 0.15 at 90°/s. For the mean 90.7 kg pitcher: IR concentric **66 N·m (90°/s = 1.57 rad/s), 57 N·m (3.14 rad/s), 52 N·m (4.71 rad/s)**; IR eccentric **80 N·m**; ER/IR ratio 0.65 | **FT** (**TAB**) |
| Same, professional pitchers | Ellenbecker & Mattalino 1997 (125 pitchers, 210 and 300°/s), Wilk 1993 (150 pitchers, 180 and 300°/s; ER/IR ratio **65 % at 180°/s and 61 % at 300°/s**), Brown 1988 (41 MLB players) | Throwing-arm internal rotation torque greater than non-throwing arm; the values are in the papers' tables | **AB** |
| Wrist flexion / extension / radial / ulnar | Delp, Grierson, Buchanan 1996 (J Biomech 29:1371), 10 healthy adult males, shoulder 90° abducted, elbow 90° flexed, isometric | Peak **flexion 12.2 ± 3.7 N·m (range 5.2–18.7), extension 7.1 ± 2.1 (3.4–9.4), radial deviation 11.0 ± 2.0 (7.9–15.3), ulnar deviation 9.5 ± 2.2 (5.9–11.9)**; flexion peaks near 40° flexion; passive moments ≤ 1.2 N·m in the mid-range | **AB** |
| Elbow flexion/extension, forearm pronation/supination | Hortobágyi & Katch 1990 (cited by Attias 2026) | The high-strength group had **25 % greater** peak net elbow flexion and extension torque than the low-strength group | **2nd** (the paper itself not opened); no N·m values found |
| Golf swing generators (a model's estimate) | MacKenzie & Sprigings 2009 | Maximum torso rotation torque in the literature **192 N·m**; shoulder abduction **~80 N·m**; both quoted from sources they cite (refs 34, 35, not opened) | **2nd** ([note 1](1-forward-dynamics-precedents.md)) |

**The mismatch that matters.** The pitching shoulder's internal-rotation torque in the delivery is **90–120 N·m** (ASMI, [note 3](3-pitching-kinetics.md)) at an angular velocity of 6,500–6,700 °/s (about **115 rad/s**). The strongest isokinetic internal-rotation values measured for the same kind of pitcher are **52–80 N·m**, at no more than **4.7 rad/s** (concentric) or 1.6 rad/s (eccentric 80 N·m). So (a) the torque the pitcher exerts at the instant of maximum external rotation is **greater than the pitcher's maximal voluntary concentric torque** and about the size of the eccentric one; (b) at 115 rad/s any Hill-type concentric generator is at or below zero: with a force–velocity factor `(ωmax − ω)/(ωmax + C ω)`, as in the golf models of note 1 (ωmax 30–60 rad/s, C = 3), the factor is **negative** for ω > ωmax, i.e. no drive at all (my arithmetic). Hence the arm's late speed comes from **elastic recoil and velocity-dependent (inertial) torques**, as Hirashima 2008 and Roach 2013/2014 found; torque generators are the early, slow part of the chain. Caveat: isokinetic strength is measured at a different arm position and without the stretch-shortening cycle, and the two torque definitions need not match.

## 4. How fast drive rises and falls (the time constants in the models of note 1)

| Value | Used by | Notes |
| :-- | :-- | :-- |
| Activation `τ = 40 ms` (`T = Tmax(1 − e^(−t/τ))`) | Sprigings & Neal 2001 (Sports Eng), citing Sprigings 1986 | FT |
| Activation **20 ms**, deactivation **40 ms** for all four generators | MacKenzie & Sprigings 2009 | FT; the pair appears chosen, not fitted |
| Activation time constant **10 ms → 5 ms** (a Hill-type muscle in OpenSim, chosen "to better represent the high speed of the pitching motion"); "0.005 s was the lower bound of commonly used values in the literature whereas 0.01 s was the upper bound" | Attias 2026 (from Buffi et al.) | FT |
| Activation **ramps with a minimum of 70 ms** (quintic function, zero velocity and acceleration at the ends) | Felton 2023 (from Yeadon et al. 2010) | FT |
| Model-time constants 10 ms and 40 ms as defaults, another source | a musculotendon model's documentation (search result, not opened) | 2nd |
| Activation **15 ms**, deactivation **50 ms** (the defaults of OpenSim's `Thelen2003Muscle`) | Thelen 2003, J Biomech Eng 125(1):70–77, doi 10.1115/1.1531112; the values as set in OpenSim's source, `OpenSim/Actuators/Thelen2003Muscle.cpp` (`constructProperty_activation_time_constant(0.015)`, `constructProperty_deactivation_time_constant(0.050)`) | **AB** for the paper, whose abstract states no time constants (it adjusts Hill-model parameters for older adults, including "prolonged muscular deactivation"); the numbers are read from the OpenSim source only (added 2026-11-25, body-design) |

What physiology says about the rise of *voluntary* torque: the review by Maffiuletti et al. 2016 (Eur J Appl Physiol 116:1091; **AB**) states that the rate of force development in an explosive contraction is "mainly determined by the capacity to produce maximal voluntary activation in the early phase (first **50–75 ms**)", through motor-unit discharge rate; it can be raised by explosive or heavy-resistance training. I did **not** open a measured value of electromechanical delay or of the time to a given fraction of peak torque in ballistic contractions in any joint relevant here; the searches returned none. So **the 20–40 ms used in golf models is a modelling constant, and the ≥ 70 ms ramp of the cricket models is a fitted minimum; the measured rise of a voluntary maximal drive to its peak is of the order of the 50–75 ms of the review**, but I have no number for time to 90 % of peak torque.

**Intrinsic stiffness, for the postural feedback** (added 2026-11-25, body-design, after the reviews). Loram & Lakie 2002, "Direct measurement of human ankle stiffness during quiet standing: the intrinsic mechanical stiffness is insufficient for stability", J Physiol 545(3):1041–1053, doi 10.1113/jphysiol.2002.025049 (PMC2290720). **AB** (the abstract, via NCBI's E-utilities; the full text not opened): small mechanical perturbations to the foot of freely standing subjects, with no stretch reflex in soleus or gastrocnemius; the intrinsic ankle stiffness is **91 ± 23 % (mean ± SD) of that necessary to provide minimal stabilisation**, about constant, rising only slightly with ankle torque, and attributed "to the foot, Achilles' tendon and aponeurosis rather than the activated calf muscle fibres". The abstract does not give the number of subjects. It is quiet-standing ankle stiffness, not a gain measured in a swing or a stride.

## 5. What this gives the design

1. **A leg generator can be sized from published parameters:** C1 (as a multiple of weight × height), ω75 % and ω50 %, for hip, knee and ankle, men 18–25, recreationally active; scale up for athletes (unsourced).
2. **Trunk rotation torque has a measured level (about 120–140 N·m for 74 kg professional male golfers)** and falls about **17 % from 0.5 to 2.1 rad/s**. The tested range (≤ 2.1 rad/s) is small beside a swing's trunk rotation speed (Koike's simulated lower trunk reached 9.4–11.8 rad/s, [note 1](1-forward-dynamics-precedents.md)), so the rest is extrapolation.
3. **Shoulder internal rotation of 50–80 N·m (collegiate pitchers) is smaller than the delivery's 90–120 N·m**: the delivery needs the elastic and velocity-dependent routes (notes 3, 1). A pure-torque shoulder cannot reproduce a pitch.
4. **Wrist torque is small** (7–12 N·m flexion/extension, ~10 N·m deviation): even a perfect wrist generator is a minor drive; in bat swings it is a hinge (note 1).
5. **The time constants in the literature span 5–40 ms (activation) and ≥ 70 ms (ramp)**; choose one with its source, and make it a named parameter (invariant: [E] values).

## 6. Not found

- **Maximum voluntary torques** for hip rotation, hip abduction (as a model), trunk rotation in baseball players, shoulder horizontal adduction, elbow flexion and extension, and forearm rotation as N·m in athletes or men; and their **fall with angular velocity** beyond the isokinetic range.
- **Any strength value at joint speeds above about 5 rad/s** (a single paper that measures torque during a fast ballistic motion).
- A measured **electromechanical delay** and a **time to 90 % of peak torque**.
- Athlete versus non-athlete **C1** ratios for the Anderson model.
- Licence: Anderson 2007, MacFadden-group 2023 and the golfers' paper are open access; only a table's few values are quoted. Delp 1996 and the other abstracts are quoted from PubMed.
