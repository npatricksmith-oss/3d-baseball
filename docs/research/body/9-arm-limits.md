# Body research 9: arm strength limits, strength at speed, tendon stretch and when the arm muscles switch on (2026-11-25, worker arm-research)

For the owner's question in [body-motion §4.4](../../body-motion.md) and [§4.6](../../body-motion.md): the arms' limits (100 N·m a shoulder, 70 N·m an elbow, one generic strength-at-speed law, no tendon stretch, no arm drive before −170 ms) are estimates. This note extends [note 5](5-strength-limits.md) and [note 8](8-strength-at-speed.md) and reports what the sources support; it argues for no number. Reading status: **FT** full text read; **TAB** a table read from an open full text or abstract page with a table; **AB** abstract only; **EXC** the passage a search tool returned, page not opened further (tables may be garbled); **2nd** quoted by another paper. SI beside the original units. "My arithmetic" marks anything I computed. Scaling to the mean hitter (92 kg, 1.84 m) is by weight × height ratio (92 × 1.84 = 169.3 kg·m over the sample's mass × height), as in §4.5; where a sample's height was not read, I say so and do not scale.

## 1. Kentel, King & Mitchell 2011 (J Appl Biomech 27:345–354)

**How I read it: FT** (the Loughborough repository copy, `pdfs.semanticscholar.org/8fbe/39f8e5a911cd5d005d4c42b1b5073691f515.pdf`; Table 2 is complete but its text layer breaks some numbers across spaces, e.g. "56.2 9"; I rejoined them, and every row's numbers are in the order the column heads give).

**Subject and task.** One 22-year-old male elite tennis player (ranked UK top 20), one-handed topspin backhands against balls fired at 30 ± 1 m/s. **His height and mass are not stated anywhere in the paper** (Table 1 gives only segment masses: upper arm 2.16 kg, forearm 1.52 kg, hand 0.40 kg), so these torques cannot be scaled by size; I report them as measured.

**Measurement.** Cybex Norm dynamometer. Isometric torque at eight joint angles across each range; concentric–eccentric isovelocity trials at crank speeds 50, 100, 150, 200, 250, 300, 350°/s (0.87–6.1 rad/s), two repetitions (King & Yeadon 2002 protocol). The curve is the 7-parameter function of Yeadon, King & Wilson 2006 times a 2-parameter quadratic angle term, fitted by simulated annealing. **Series elastic component: "stiffness parameter determined assuming 5% stretch for a maximal isometric contraction (de Zee & Voigt 2001)"**, i.e. K was assumed, not measured at these joints.

**Table 2, as printed** (T₀ isometric N·m; T_max maximum eccentric N·m; ω_max rad/s where concentric torque reaches zero; ω_c rad/s, the Hill hyperbola's asymptote; K N·m/rad):

| Joint movement | T₀ | T_max | ω_max | ω_c | K | θ_opt (rad) |
| :-- | --: | --: | --: | --: | --: | --: |
| Shoulder flexion | 56.29 | 78.81 | 20.74 | 6.70 | 1386.9 | 4.96 |
| Shoulder extension | 119.13 | 166.78 | 31.93 | 15.02 | 2366.9 | 0.83 |
| Shoulder horizontal adduction | 97.48 | 136.48 | 18.05 | 6.43 | 1968.5 | 2.41 |
| Shoulder horizontal abduction | 71.90 | 100.66 | 18.00 | 6.93 | 785.9 | 4.99 |
| Shoulder internal rotation | 42.67 | 59.73 | 19.47 | 9.71 | 603.1 | 5.90 |
| Shoulder external rotation | 33.75 | 47.25 | 18.00 | 7.20 | 113.7 | 0.90 |
| Elbow flexion | 75.82 | 106.14 | 18.10 | 5.43 | 651.5 | 0.33 |
| Elbow extension | 51.95 | 72.73 | 18.45 | 5.55 | 280.3 | 4.64 |
| Pronation | 17.35 | 24.29 | 27.88 | 9.71 | 18.0 | 4.50 |
| Supination | 18.30 | 25.62 | 35.45 | 16.90 | 36.2 | 1.78 |
| Wrist flexion | 45.78 | 64.10 | 21.40 | 10.70 | 31.2 | 4.26 |
| Wrist extension | 24.90 | 34.86 | 18.08 | 7.78 | 29.2 | 1.27 |
| Ulnar deviation | 41.44 | 58.02 | 18.02 | 5.83 | 23.3 | 0.43 |
| Radial deviation | 31.63 | 44.28 | 21.77 | 7.72 | 30.4 | 5.14 |

The paper also lists a_min, m, ω₁ and q (the differential-activation and angle-width parameters; read, not reproduced here) and states T_max = 1.4 × T₀ in every row (my arithmetic: 78.81/56.29 = 1.40; Yeadon 2006 sets T_max = 1.5 T₀ for the knee). Shoulder flexion, extension, horizontal adduction and abduction, internal and external rotation, elbow flexion and extension, pronation, supination, wrist flexion and extension, ulnar and radial deviation are all there; nothing is missing from the direction list. **Note 8 read only part of this table; this is all of it.**

**What the printed K implies (my arithmetic, T₀ / K):** an elastic angle of 2.3° (shoulder flexion), 2.9° (extension), 2.8° (horizontal adduction), 5.2° (abduction), 4.1° (internal rotation), 17° (external rotation), 6.7° (elbow flexion), 10.6° (elbow extension), 55° (pronation), 29° (supination), 84° (wrist flexion), 49° (wrist extension), 102° (ulnar), 60° (radial). The first five are plausible for a 5% stretch at a moment arm of 2–5 cm; the last nine are not, so **either K is in another unit for those rows or my reading of them is off. I would not use K for the elbow extensor, forearm or wrist rows without opening the original or King et al. 2006.** The shoulder rows (K ≈ 790–2370 N·m/rad) are the usable ones.

**Strength against speed from this table (my arithmetic).** Concentric branch, Yeadon 2006 eq. 2–3 (read in the paper, `cs.cmu.edu/~cga/d/app3.pdf`, FT): T/T₀ = (ω_c/ω_max)(ω_max − ω)/(ω_c + ω), the fraction of isometric torque at the contractile component's velocity ω, before the differential-activation factor (which for concentric speeds above ω₁ is near 1) and before series stretch (the joint moves faster than the contractile part by the tendon's share):

| Channel | 3 rad/s | 5 | 7 | 9 |
| :-- | --: | --: | --: | --: |
| Shoulder flexion | 0.59 | 0.43 | 0.32 | 0.24 |
| Shoulder extension | 0.76 | 0.63 | 0.53 | 0.45 |
| Horizontal adduction | 0.57 | 0.41 | 0.29 | 0.21 |
| Horizontal abduction | 0.58 | 0.42 | 0.30 | 0.22 |
| Internal rotation | 0.65 | 0.49 | 0.37 | 0.28 |
| External rotation | 0.59 | 0.43 | 0.31 | 0.22 |
| Elbow flexion | 0.54 | 0.38 | 0.27 | 0.19 |
| Elbow extension | 0.54 | 0.38 | 0.27 | 0.20 |
| Pronation | 0.68 | 0.54 | 0.44 | 0.35 |
| Supination | 0.78 | 0.66 | 0.57 | 0.49 |
| Wrist flexion | 0.67 | 0.52 | 0.41 | 0.31 |
| Wrist extension | 0.60 | 0.44 | 0.32 | 0.23 |
| Ulnar deviation | 0.55 | 0.39 | 0.28 | 0.20 |
| Radial deviation | 0.62 | 0.47 | 0.36 | 0.27 |

The subject's own data stop at 6.1 rad/s (350°/s crank); the 7 and 9 rad/s columns are the fitted hyperbola's extrapolation. This is one man's curve and fitted by one method; compare with the other athletes' fractions in §3.

## 2. Samples with means and SDs, N·m

Mean + 2 SD, and its scaled value to 92 kg × 1.84 m where the sample's size was read. SD from n ≤ 10 is very uncertain (Holzbaur: "standard deviations for a given joint averaged 57% of the average moment" across their mixed-sex ten).

### 2.1 Healthy adults, isometric

| Source (read) | Subjects, posture, machine | Result N·m (mean ± SD) | Mean + 2 SD (N·m) | ×1.167 → 92 kg, 1.84 m (my arithmetic) |
| :-- | :-- | :-- | :-- | :-- |
| Holzbaur, Delp, Gold & Murray, J Biomech 2007;40:742 (**TAB**, Stanford copy, text layer garbled; means and SDs of the male row read) | 5 men, 29.2 y, 177.8 cm, 81.6 kg; Biodex 3, dominant arm; elbow at 90° flexion, forearm supinated, wrist braced; shoulder adduction/abduction at neutral abduction (posture details read in part) | shoulder adduction **93.7 ± 11.3**, abduction **74.4 ± 10.8**; elbow flexion **79.5 ± 8.1**, extension **60.5 ± 6.2**; wrist flexion 25.6 (SD printed "7", cut), extension **14.0 ± 3.4** | add 116.3, abd 96.0, elb flex 95.7, elb ext 72.9, wrist flex ~39.6, wrist ext 20.8 | **136, 112, 112, 85, ~46, 24** |
| Danneskiold-Samsøe et al., Acta Physiol 2009;197:225 (**EXC**, Lido, men aged 20–29, isometric; n and body size not in my excerpt) | healthy Danish men, 20–29 y | shoulder flexion 63.0 ± 15.8, extension 91.9 ± 19.7, abduction 60.2 ± 14.0, adduction 89.6 ± 26.9; elbow flexion 70.3 ± 14.6, extension 53.2 ± 11.3; wrist flexion 23.9 ± 7.4, extension 13.1 ± 3.0 (wrist n = 10) | 94.6; 131.3; 88.2; 143.4; 99.5; 75.8; 38.7; 19.1 | not scaled (size unread) |
| "Maximal isokinetic and isometric muscle strength … 178 healthy subjects" (Danish; **TAB**, Table 3; authors not shown on the page I read; Biodex 3 Pro, dominant arm) | 93 men aged 15–83 (n per cell 79–91); sizes not in my excerpt | isometric: shoulder abduction 61.1 ± 14.1; elbow flexion 50.9 ± 13.6; wrist flexion 24.7 ± 7.45. Isokinetic at 60°/s (1.05 rad/s): abduction 59.5 ± 12.0, **adduction 83.1 ± 17.7**, elbow extension 46.7 ± 10.0, flexion 51.0 ± 12.2, wrist extension 11.4 ± 3.1, flexion 22.1 ± 5.5 | 89.3; 78.1; 39.6; 83.5; 118.5; 66.7; 75.4; 17.6; 33.1 | not scaled; the age range makes it a low anchor |
| Kaneko et al., MicroFET norms (**TAB**, hdl.handle.net/20.500.14094/00188176) | 110 men, 21.5 y, 171.0 cm, 64.4 kg, hand-held dynamometer, **force in N, not torque** | shoulder flexion 189 ± 50 N, extension 163 ± 49, abduction 178 ± 47, adduction 184 ± 63, internal rotation 177 ± 57, external rotation 121 ± 29; elbow flexion 225 ± 62, extension 177 ± 45; wrist flexion 125 ± 34, extension 114 ± 42 | in N; no lever arm given | not converted |

### 2.2 Athletes

| Source (read) | Subjects, speed, machine | Result | Scaled (my arithmetic) |
| :-- | :-- | :-- | :-- |
| Dodds et al., Int J Exerc Sci 2020;13:123 (**TAB**, PMC7039469) | 18 NCAA Division II baseball players (9 pitchers, 9 position players), 183.4 ± 4.7 cm, 89.6 ± 10.8 kg; Humac Norm, supine, 90°/90° abduction/elbow flexion, dominant arm, 5 repetitions at 180 and 300°/s (3.14, 5.24 rad/s) | pitchers: ER 20.7 ± 7.1 (180°/s), 16.4 ± 7.1 (300); IR 27.4 ± 12.3, 22.2 ± 10.2. Position players: ER 23.2 ± 7.4, 19.9 ± 4.0; IR **37.2 ± 15.4, 31.0 ± 13.4** | ×1.031. Position players' IR at 180°/s: mean 38.4, mean + 2 SD **70.1**; ER mean + 2 SD 39.2. These are low compared with Wilk's per-body-weight values below; the testing range was stopped at 65° IR |
| Wilkin & Haddock, J Strength Cond Res 2006;20:829 (**AB**; values from the abstract; "mean ± SEM") | 9 NCAA Division II pitchers, 23 y, 91.2 ± 3.1 kg; Biodex, 90°/90°, concentric 300 and 450°/s (5.24, 7.85 rad/s) | IR **50.7 ± 2.3 (SEM) at 300°/s, 37.1 ± 2.5 at 450°/s**; ER 30.2 ± 1.7 and 17.7 ± 2.2 (preseason). Treating SEM × √9 as SD (my arithmetic): SD 6.8, 7.6, 5.1, 6.5 | IR mean + 2 SD at 300°/s 64.3 N·m (mass ratio 1.01, height unread) |
| Wilk et al. 1993 (Am J Sports Med 21:61; **AB**), summary table in Wilk et al., JOSPT 2009;39:38 (**EXC**, table of N = 83 professional baseball players, "peak torque measured in ft·lb and body weight in lb") | 150 professional pitchers, 23.4 y, 199 lb (90.3 kg), Biodex, 180 and 300°/s | peak torque / body weight (%, i.e. ft·lb per lb × 100): ER 18–23 (180°/s), 15–20 (300); IR 27–33 and 25–30; abduction 26–32 and 20–26; adduction 32–36 and 28–33 | 1 ft·lb/lb = 2.989 N·m/kg: IR at 180°/s 0.81–0.99 N·m/kg → **74–91 N·m at 92 kg**; at 300°/s 0.75–0.90 → 69–83; adduction (shoulder, 180°/s) 0.96–1.08 → 88–99. These are the **ranges of means in the table, not an SD**; no SD was read |
| Andrade et al., shoulder isokinetic profile of handball players (PMC4207149, **TAB**) | 20 male national-team handball players, dominant arm, concentric 60 and 300°/s (1.05, 5.24 rad/s) | IR 0.71 N·m/kg (range 0.47–1.05) at 60°/s; 0.60 (0.41–0.88) at 300°/s; ER 0.51 (0.27–0.82), 0.44 (0.27–0.70); eccentric IR at 300°/s 0.84 (0.54–1.24). The ± printed with the means are small (0.02–0.05), so they are SEs, not SDs | IR at 60°/s 65 N·m at 92 kg; the sample's largest man (1.05) 97 N·m; eccentric IR at 300°/s 77 (max 114) |
| Silva et al., Br J Sports Med 2006;40:513 (**TAB**, PMC2465104) | 23 elite junior male tennis players, 14 y, 170 ± 14 cm, 59.8 ± 14.5 kg; Cybex 6000, supine, 60 and 180°/s | **horizontal adduction** 56.0 ± 22.7 N·m (60°/s), 56.2 ± 23.5 (180); **horizontal abduction** 42.5 ± 15.2, 39.6 ± 16.5; per body mass (printed per 100 kg) adduction 90.9 ± 21.3 and 90.4 ± 22.9, abduction 69.8 ± 15.0 and 64.5 ± 17.3 | per-mass mean + 2 SD × 92 kg: adduction **123 (60°/s), 125 (180°/s)**; abduction **92, 91**. Boys of 14 years; the per-mass scaling is the weaker of the two scalings |
| Laudner et al., Phys Ther Sport 2012;13:97 (**AB**/TAB headers; tables not reachable) | 30 NCAA Division I baseball players, 185.0 ± 5.0 cm, 88.9 ± 9.7 kg; Cybex Norm, 90 and 180°/s | pronation, supination, wrist flexion, extension peak torque in N·m exist in the paper's tables; **I could not read them**. Throwing arm: more pronation (90, 180°/s), more wrist extension (180°/s), less supination and wrist flexion (180°/s) | not read |
| Peleg et al., J Funct Morphol Kinesiol 2025;10:377 (**TAB**, PMC12551099; open) | 20 healthy men, 25.5 y, 176.3 ± 7.3 cm, 74.3 ± 9.9 kg, Biodex 4 Pro, dominant side, 90°/s (1.57 rad/s), range 45° | wrist concentric: **ulnar deviation 26.5 ± 6.6**, radial deviation **14.9 ± 3.4**, extension **8.8 ± 1.7** (flexion row cut in my excerpt); eccentric ulnar 30.3 ± 7.1, radial 18.9 ± 3.9 | ×1.292: ulnar mean + 2 SD 39.7 → **51.3**; radial 21.7 → **28.0**; extension 12.2 → 15.8. Healthy students, not athletes |
| Ellenbecker et al., Br J Sports Med 2002 (PMC2577484, **TAB**) | 32 elite female junior tennis players, 13.7 y, Cybex 6000, 90 and 210°/s | forearm pronation, supination, wrist flexion, extension, as peak torque/body weight (ft·lb per lb × 100): pronation 3.78 ± 1.05 (90°/s), 3.26 ± 0.90 (210); supination 2.15 ± 0.60, 1.85 ± 0.66; wrist flexion 5.00 ± 1.74, 3.59 ± 1.29; extension 3.81 ± 1.20, 2.94 ± 1.13 | girls: not scaled to a man; used only for fractions (§3) |
| Studia Sportiva (Masaryk), boys 12–14 (**EXC**) | 10 junior tennis players, Humac Norm | wrist extension 7.30 ± 2.79, flexion 12.80 ± 4.19 N·m (90°/s); 7.20 ± 2.56, 9.70 ± 2.28 (300°/s) | children; fractions only |

Also found, not used for numbers: Ellenbecker & Mattalino 1997 (125 pro pitchers, Cybex 300, 210 and 300°/s, IR/ER; **AB**), Newsham et al. 1998 (16 college pitchers, 180/300/450°/s, ER/IR ≈ 0.65; **AB**), Brown et al. 1988 (41 pro players; **AB**), Ellenbecker 1991 (22 male tennis players, shoulder/forearm/wrist at 90, 210, 300°/s; **AB**; no numbers in the abstract), Alderink & Kuck 1986 (24 baseball players' horizontal adduction/abduction, cited by Silva; **2nd**, not opened).

### 2.3 What the samples say against 100 and 70 N·m

- **Shoulder.** Healthy men's shoulder isometric maxima at their own sizes are 60–94 N·m for the groups a swing uses, with mean + 2 SD 88–143 N·m (Danneskiold-Samsøe 20–29 y: adduction 89.6 ± 26.9, extension 91.9 ± 19.7); scaled to the mean hitter the one size-given sample (Holzbaur, 5 men) gives 136 N·m (adduction) and 112 N·m (abduction) at mean + 2 SD. In athletes the shoulder's isokinetic rotation is 37–91 N·m at 3–5 rad/s (Dodds, Wilk). **100 N·m a shoulder lies inside the 92 kg mean + 2 SD of these healthy-adult samples for adduction (136) and abduction (112), and is about the mean for adduction in the Danneskiold-Samsøe sample.** Kentel's elite player: 97.5 N·m horizontal adduction, 71.9 abduction, 119.1 extension, 56.3 flexion, 42.7 internal rotation, 33.8 external rotation. No sample measured horizontal adduction in adult athletes in N·m (Silva's adolescents: mean + 2 SD ≈ 123–125 N·m at 92 kg by body mass).
- **Elbow.** Isometric flexion 70–80 N·m, extension 53–60 N·m at the mean; mean + 2 SD **flexion 96–112, extension 73–85 N·m** (Holzbaur scaled 112 and 85; Danneskiold-Samsøe unscaled 99.5 and 75.8). Kentel's player: flexion 75.8, extension 52.0. **70 N·m an elbow is at or below the mean for flexion and at about the mean + 1 SD for extension in these samples; it is below the mean + 2 SD bounds above for both.**
- **Wrist and forearm.** Wrist: isometric flexion 24–26 N·m, extension 13–14 (mean), mean + 2 SD about 39–46 and 19–24; ulnar deviation 26.5 (mean + 2 SD 51 scaled), radial 14.9 (28). Kentel's player has wrist flexion 45.8, ulnar 41.4, radial 31.6, extension 24.9. (§4.4's casting run used 20 N·m for the wrists.)
- **Forearm pronation and supination:** no adult-male sample in N·m was found; Kentel's one player (17.4, 18.3 N·m) is the only number.

## 3. Strength against speed in the arm directions

Fraction of the slow torque kept (concentric isokinetic; my arithmetic from the printed means):

| Direction, sample (read) | Slow → fast | Kept |
| :-- | :-- | :-- |
| Shoulder IR, pitchers, Wilkin 2006 (**AB**) | 300 → 450°/s (5.24 → 7.85 rad/s) | **73%** (37.14/50.66); ER 59% (17.68/30.16) |
| Shoulder IR, ER, Dodds 2020 (**TAB**) | 180 → 300°/s (3.14 → 5.24 rad/s) | pitchers IR 81%, ER 80%; position players IR 83%, ER 86% |
| Shoulder IR, ER, handball, Andrade (**TAB**) | 60 → 300°/s (1.05 → 5.24) | IR 85% (0.60/0.71), ER 86% (0.44/0.51) |
| Shoulder IR, ER, Wilk 2009 summary (**EXC**) | 180 → 300°/s | IR 91% (25–30 vs 27–33), ER 83% (15–20 vs 18–23); the table is ranges of means |
| Shoulder horizontal adduction, abduction, junior tennis, Silva (**TAB**) | 60 → 180°/s (1.05 → 3.14) | adduction 100% (56.2/56.0), abduction 93% (39.6/42.5) |
| Forearm pronation, supination, junior female tennis, Ellenbecker 2002 (**TAB**) | 90 → 210°/s (1.57 → 3.67) | pronation 86%, supination 86%; wrist extension 77%, flexion 72% |
| Wrist, junior tennis boys (**EXC**) | 90 → 300°/s (1.57 → 5.24) | extension 99%, flexion 76% |
| Danneskiold-Samsøe 2009 (**EXC**): shoulder, elbow, wrist force–velocity curves at 30–120°/s | | the paper says "the strengths at different velocities barely changed" for shoulder flexion/extension and abduction/adduction; no fractions read |
| All channels, Kentel's player (§1) | fitted hyperbola, 3–9 rad/s | 0.54 (elbow) to 0.78 (supination) at 3 rad/s; 0.19–0.49 at 9 rad/s |

**The highest speed any arm sample reached is 7.85 rad/s (450°/s, shoulder IR/ER), and only for rotation.** I found **no isokinetic measurement of horizontal adduction, elbow extension or flexion, wrist or forearm above about 5.2 rad/s (300°/s)** except Kentel's single player to 6.1 rad/s. The fractions above are from machines at 60–450°/s and from different people and techniques. They do not agree with each other: at 5.2 rad/s the pitchers' shoulder IR keeps 81–85% of the 1–3 rad/s value, while Kentel's hyperbola leaves 0.38–0.66 of isometric at 5 rad/s (the isometric value is higher than the slowest isokinetic value, which accounts for part of the difference, but not by a factor of two at the same speed).

## 4. When the arm muscles switch on in a swing

### 4.1 Electromyography of batting

| Source (read) | Subjects, task | What it found |
| :-- | :-- | :-- |
| Shaffer, Jobe, Pink & Perry, Clin Orthop 1993;292:285 (**AB** + secondary descriptions: Fortenbaugh's 2011 thesis, and Stewart's 2017 thesis, which reproduces numbers from the paper's table; **2nd**) | 18 professional batters (17 minor league, 1 major league), live swings; fine-wire electrodes in the lead arm's supraspinatus, triceps, posterior deltoid, middle serratus anterior; surface on the trunk and back leg; levels as % of a maximum muscle test (MMT) in six phases (wind-up, pre-swing, early/middle/late swing, follow-through) | Sequence: hips, then trunk, then arms. **Posterior deltoid, lead arm: 101 ± 91, 88 ± 37, 82 ± 45, 76 ± 40% MMT across pre-swing and the three swing phases, i.e. already active in pre-swing and highest then.** Triceps: 92 ± 50% (early swing), 73 ± 35% (middle); "over 90%" only early. Supraspinatus and serratus anterior: below 40% MMT throughout. Trunk: erector spinae 85–185%, obliques > 100% from pre-swing. Authors' reading: the arm muscles position the arms and hands, not generate the power. **No timing in ms relative to bat or hand motion is given; the phases are defined by the film, not by a clock I could read** |
| Kitzman, Res Q 1964 (**AB**) | 2 professional, 2 novice batters; pectoralis major, triceps, latissimus dorsi, both sides | these were active early in the swing and "took over" by others later (Fortenbaugh's wording, 2nd) |
| Reyes, Dickin, Crusat & Dolny, Sports Biomech 2011 (**AB**) | 16 recreational men, tee; gastrocnemius, biceps femoris, gluteus maximus, pectoralis major, latissimus dorsi, triceps | "a specific muscle recruitment order … starting with the lower body followed by the upper body muscles"; no ms in the abstract |
| Lim, Park & Kwon, ISBS 2015 (**AB**) | 3 recreational batters | peak (not onset) times in seconds on the recording's own clock: right pectoralis major 1.03, right external oblique 1.11, latissimus dorsi 1.15–1.16, left pectoralis major 1.25; not tied to hand motion in the abstract |
| Lim & Kwon, Korean J Appl Biomech 2016;26:205 (**AB** + methods) | 10 elite college and 10 amateur batters, 16 muscles (Delsys), "premotor time" | PMT = first sample above resting mean + 3 SD (1 s of rest); the abstract states only that tibialis anterior, gastrocnemius, external oblique and erector spinae differed between groups; no arm muscle numbers read |
| Ohta & Nakamoto, Int J Sport Health Sci 2015;13:84 (**FT** of the first pages) | 9 college batters, simulator, 40 swings; **front-arm triceps brachii** and leg muscles | the front triceps is "greatly activated during the swing" (peak EMG, mean of all muscles 74% of a maximum voluntary contraction); the time to peak is measured from the target's start, not from hand motion; no onset in ms from bat motion |
| Stewart et al. (Stewart, Simpson, Knight, Chander, Shapiro), swing-phase initiation (**TAB**; uky.edu `khp_facpub` 1021) | 12 collegiate batters, tee | phases timed before contact: shifting (trail rectus femoris EMG onset) −766 ± 181 ms, stepping −544 ± 91, landing −297 ± 60, **swinging (the lead hand starts to move toward the ball) −215 ± 44 ms**. The arms' own EMG was not timed |

**What is established:** the only quantitative arm EMG of professional batters is Shaffer's, and it gives levels by phase, not an onset before visible hand motion. Its phase table has the posterior deltoid (a lead-arm shoulder muscle) highest in pre-swing, and the triceps highest in early swing, i.e. **not silent before the swing proper**. The hands' visible start in a tee swing sits at about −215 ms in Stewart's college sample (against −170 ms for the professionals at 56 mph batting practice in the design). **I found no paper that gives an arm muscle's EMG onset in ms before the hands or bat visibly start forward.**

### 4.2 Electromechanical delay

| Source (read) | Subjects, muscle, method | Delay |
| :-- | :-- | :-- |
| Dick et al., J Electromyogr Kinesiol 2024 (consensus paper, **EXC**) | review | "a ~10 ms (Corcos 1992; Nordez 2009) to 120 ms (Blackburn 2009) delay between the onset of muscle activation and the onset of muscle force and torque"; varies with muscle, individual, length, method |
| Del Vecchio et al., J Appl Physiol 2018 (**EXC**) | gastrocnemius with electrical stimulation ~15 ms; vastus lateralis voluntary from rest ~38 ms (~17 ms stimulated); **biceps brachii, voluntary fast contractions from a baseline level, ~70 ms** (citing another study) | the one upper-limb figure I found, second hand |
| Chinese-language review of the literature 1979–2017 (cjter.com) (**EXC**, Chinese; in my translation) | | electrical onset to first mechanical signal 10–25 ms, mechanical signal to torque 29–45 ms, total 39–70 ms; passive series elastic parts (tendon and aponeurosis) account for 47.5 ± 6.0% of the total (Nordez 2009) |
| Jöllenbeck (ISBS 1998 and 1999; **TAB**, 140 men, explosive isometric contractions of elbow extensors, knee extensors, knee flexors) | | EMD from 12.3 ms to 112.7 ms for the same data depending on the detection threshold and filter (about 20 ms with the visual method) |
| Go et al., J Biomech 2018 (**TAB**, PMC6062445) | 8 young adults, tibialis anterior, ramped isometric | EMG-to-force 36 ± 31 ms (mean), median 170 ms in their setup; the paper blames slack in the set-up |

The delay is **a tendon-and-muscle property, partly series stretch**: with 3.3–5% tendon strain the stretch takes time to build (Nordez: tendon plus aponeurosis ~48% of the EMD). So electrical onset precedes visible joint motion by at least the EMD (about 10–70 ms for a voluntary start from rest in the studies read) and by more when the joint must overcome the bat's inertia. **No source gave this delay for the shoulder muscles or in a swing.**

## 5. Tendon stretch and series stiffness

| Source (read) | Model | Numbers |
| :-- | :-- | :-- |
| Kentel et al. 2011 (**FT**; §1) | torque generators at the arm joints, each a contractile component in series with an elastic component | K (N·m/rad) per channel as in the table; assumed 5% stretch at maximal isometric force (de Zee & Voigt 2001, ankle plantar flexors) |
| Chadwick et al., IEEE TNSRE 2009 (PMC2971671, **TAB**) | Hill-type 3D arm model | series element force F = k (L − L_slack)²; k set so a **4% elongation** at maximum isometric force |
| Thelen 2003 as implemented in OpenSim, John 2011 (**TAB**, `unusualinsights.com`) | Hill-type, all muscles | tendon strain at maximum isometric force **0.033**, linear-region stiffness 1.712/ε₀ᵀ (normalised), toe region to ε = 0.609 ε₀ᵀ at a force of 1/3 |
| Holzbaur, Murray & Delp, Ann Biomed Eng 2005 (**EXC**) | 3D upper-extremity model, 15 shoulder, 9 elbow and forearm muscles | tendon slack lengths by muscle (e.g. biceps long head 27.2 cm; supraspinatus 4.0 cm; pectoralis major sternal 8.9 cm), peak forces from PCSA × specific tension 140 N/cm² (shoulder/elbow), 45 N/cm² (forearm/hand); the slack lengths were chosen to match measured moments and "are not directly measurable" |
| Hu, Murray & Perreault 2012 (**EXC**) | arm endpoint stiffness | tendon stiffness from Zajac's generic normalised force–strain curve (1989) |

So the published values are **3.3–5% tendon strain at maximum isometric force, generic**; the only joint-level stiffness numbers are Kentel's (shoulder rows 790–2370 N·m/rad; the rest doubtful as read). I found no measurement of series stiffness of the shoulder, elbow or wrist muscle–tendon units in vivo in the sources opened.

## What this gives the limits

The sources do not agree on a single number; the table says what each supports. All N·m, mean + 2 SD scaled to 92 kg, 1.84 m only where a size was read.

| Channel | Mean + 2 SD with source | The single subject's value (Kentel) | Against the 100 / 70 in use |
| :-- | :-- | :-- | :-- |
| Shoulder horizontal adduction (and extension) | **136** (Holzbaur, 5 men, isometric adduction); 143 unscaled (Danneskiold-Samsøe, 20–29 y); 123–125 (Silva, boys, per mass); isokinetic 118 unscaled (Danish 178, 60°/s) | 97.5 (adduction); extension 119.1 | the bound is above 100 for adduction; the mean (84–94) is below it |
| Shoulder horizontal abduction | **112** (Holzbaur); 88 (Danneskiold-Samsøe); 91–92 (Silva); 84 (Danish, isokinetic, unscaled) | 71.9 | above 100 for Holzbaur only |
| Shoulder flexion, extension | flexion 95 (Danneskiold-Samsøe, unscaled); extension 131 | 56.3; 119.1 | no scaled sample |
| Shoulder internal / external rotation | IR **64–70** (Wilkin pitchers 300°/s: 64; Dodds position players 180°/s: 70); Wilk's body-weight means 74–91 (ranges of means, not 2 SD) | 42.7 / 33.8 | below 100 in every athlete sample |
| Elbow flexion | **112** (Holzbaur, scaled); 99.5 (Danneskiold-Samsøe, unscaled); 75–78 (Danish 178, unscaled) | 75.8 | above 70 in each |
| Elbow extension | **85** (Holzbaur, scaled); 75.8 (Danneskiold-Samsøe); 66.7 (Danish 178, isokinetic) | 52.0 | above 70 in the first two; 70 is inside the mean + 2 SD |
| Forearm pronation, supination | no adult-male sample found | 17.4, 18.3 | — |
| Wrist flexion / extension | flexion **~46** (Holzbaur, scaled; SD cut), 39–40 unscaled; extension **24** (Holzbaur), 16–19 (others) | 45.8 / 24.9 | — |
| Wrist ulnar / radial deviation | **51** / **28** (Peleg, 20 healthy men, 90°/s concentric, scaled) | 41.4 / 31.6 | — |

**The speed curve's points:** Kentel's hyperbola, §1 table (0.54–0.78 of isometric at 3 rad/s, 0.38–0.66 at 5, 0.27–0.57 at 7, 0.19–0.49 at 9). Athletes' isokinetic fractions: shoulder IR/ER keeps 59–91% between 3 and 8 rad/s (§3), but the denominators are slow isokinetic values, not isometric, and **no sample measured elbow, forearm, wrist or horizontal adduction above 5.2 rad/s**, so the curve above that is extrapolated from one player.

**A stiffness:** Kentel's K for the shoulder generators, 786–2367 N·m/rad (about 2–5° of elastic stretch at full isometric torque), set by an assumed 5% stretch; other models use 3.3–4% tendon strain at maximum force. Nothing measured at the arm joints.

**What the electromyography says about the −170 ms rule.** The rule is taken from visible hand motion. The one quantitative arm EMG of professional batters (Shaffer 1993) shows the lead arm's posterior deltoid at 76–101% of a maximum test throughout pre-swing and swing and the triceps at 92% in early swing: **these shoulder and elbow muscles are active before and at the start of the swing, not first switched on at the hands' first visible motion.** I found no onset time in ms against the bat's or hands' start, so the amount of lead is not measured. The electromechanical delay (about 10–70 ms for a voluntary start; ~70 ms for the biceps in one second-hand citation) means a muscle's electrical onset precedes its joint's first visible motion by at least that, and the arms hold the bat before the swing begins, which requires some activation (not measured in the sources read).

## Not found

- Kentel's subject's height and mass (not in the paper); the validity of K for the elbow extensor, forearm and wrist rows as printed.
- Adult-male pronation and supination in N·m; wrist deviation in athletes; horizontal adduction and abduction in adult athletes in N·m (Alderink & Kuck 1986 not opened); Laudner 2012's table values.
- Any isokinetic or isometric arm value above 7.85 rad/s; any non-rotation arm value above 5.2 rad/s other than Kentel's player.
- The Shaffer 1993 full text (phase boundaries, electrode details, the swing's clock); the table of posterior deltoid, triceps, supraspinatus levels is second hand through a thesis.
- Any arm or shoulder muscle's EMG onset in ms before the hands' or bat's first visible motion in batting; any EMG of the forearm flexors, extensors and pronators in batting with timing (Ohta's extensor carpi ulnaris, 2014, concerns inhibition under correction, not onset).
- An electromechanical delay for any shoulder muscle, or in a swing; measured series stiffness of shoulder, elbow and wrist muscle–tendon units.
- The Danneskiold-Samsøe sample's sizes; the authors of the 178-subject Biodex paper as printed on the page I read.
- Ellenbecker & Roetert's 1997–2004 tennis shoulder, elbow and forearm tables and Alderink & Kuck's horizontal values, which would have given more athletes' N·m.
