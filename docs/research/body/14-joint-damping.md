# Body research 14: the joints' own damping (2026-11-26, worker body-3d, session 11)

For [PLAN D42](../../../PLAN.md) rule 1 and [body-motion §4.13](../../body-motion.md) rung 1: "Joint friction: use sourced values where a paper can be read. Where none can, use a stated range by joint, run at several values, and report how much the result depends on it." The body's joints have carried 1.0 N·m·s/rad on every axis since session 1, with no source. This note reports what sources measured: the damping B of a joint, in N·m·s/rad, the coefficient of the joint's speed in its resisting torque. It argues for no number; the choices are made in §4.13.

**Reading status:** **FT** full text read; **TAB** a table of an open paper read; **AB** abstract only; **2nd** quoted by another source; **mine** converted or inferred by the reader, not printed in the source. The pages were opened on 2026-11-26 by two search workers under those rules (each told to quote the paper's own words and to mark anything seen only second hand); what I checked myself is marked **checked**.

**Two things run through every joint below**, and both bear on what a swing's joints should carry:

1. **A joint's damping is not one number: it grows with how hard the muscles around it are working.** Wherever both were measured, relaxed damping is the smallest and it rises with activation, though less than stiffness does (the ankle 2–4 times by 30% of full effort; the knee from 1.0 to 2.9 with co-contraction; the trunk +25% for +70% load). That added damping is the muscles' own force–velocity behaviour seen from outside. **This body's muscles already carry their strength-at-speed curves**, so the active part is in the model once; only the relaxed (passive) value belongs in the joints' own damping, or it is counted twice.
2. **A relaxed joint's resistance to motion is nearly independent of speed in large movements.** Amankwah's fit to whole-range relaxed leg movements at 5–90°/s has the speed-dependent torque growing as speed to the power 0.08–0.2, and McFaull calls the small-oscillation damping values "upper limits". So a linear B identified from small, fast perturbations, carried to a swing's 500–1,500°/s, overstates what a relaxed joint takes. All the linear values below are from perturbations of a few degrees.

## Part A. The legs and trunk

### A1. Ankle

| Source | Subjects, method | Condition | B, N·m·s/rad | Read |
| :-- | :-- | :-- | :-- | :-- |
| Lee, Krebs & Hogan, IEEE TNSRE 2014;22:1104 | 10 (5 men), 21–37 y; ankle robot, random torque to 100 Hz, second-order fit 0.5–20 Hz | **relaxed** (EMG under 1.3% of maximum) | foot up and down: **0.34 (SE 0.02)** seated, **0.45 (0.02)** knee straight; rolled in and out: **0.15 (0.03)**, **0.18 (0.03)** | **FT** (its Table IV) |
| the same authors, 22:971 | the same subjects | one muscle held at 10, 15, 20, 25, 30% of maximum | up and down: 0.74, 0.86, 0.94, 1.04, 1.08 (shin muscle); 0.66 to 1.29 (calf); rolled: 0.19–0.30; 0.23–0.52 | **FT** (Table II) |
| Gottlieb & Agarwal, J Biomech 1978;11:177 | 4 men, seated; random torque to 50 Hz, excursions under 5° | relaxed; holding 1–2 N·m | relaxed 0.37–0.70 across −12° to +11° (one subject); holding: 0.53–0.86 | **FT** (OCR) |
| Mirbagheri, Barbeau & Kearney, Exp Brain Res 2000;135:423 | 13 healthy, supine; the intrinsic part separated from reflexes | rest to 50% of maximum | each subject's largest: 1.4 ± 0.16; "increased by 50% from rest to the lowest contraction level … but increased by only 15% thereafter" | **FT** (excerpt) |
| Rouse et al., IEEE TNSRE 2014;22:870; Lee, Rouse & Krebs 2016 | 10; ramps of 2° during walking | walking | not different from zero early in stance; about 0.03 N·m·s/rad per kg late; over the cycle 0.010–0.024 per kg (0.9–2.2 at 92 kg, **mine**) | **FT** |
| Weiss, Kearney & Hunter 1986 (two papers), Hunter & Kearney 1982 | | | not reached beyond abstracts; Lee 2014 says its relaxed values "were well matched" to Weiss's | **AB** |

### A2. Knee

| Source | Subjects, method | Condition | B, N·m·s/rad | Read |
| :-- | :-- | :-- | :-- | :-- |
| Koopman, van Asseldonk & van der Kooij, IEEE Trans Robotics 2016;32:920 | 8 elderly (67–82 y; 6 analysed relaxed); torque to 10 Hz on hip and knee, about 15° at the knee, standing on the other leg | relaxed (EMG 3%); holding position (EMG 22%) | "The passive knee damping found in our study (**1.0** Nms/rad)"; holding: 2.9 | **FT** |
| Tai & Robinson 1999; Zhang et al., J Biomech 1998;31:71 | | relaxed | "around 1 and 2, respectively" (Koopman's words); Zhang: about 2 (Huang 2020's words); from relaxed to 50% of maximum the damping ratio "remained at about 0.2" while the natural frequency more than doubled | **2nd**; **AB** |
| Casabona, Valle et al., J Appl Physiol 2012 | healthy controls; the lower leg dropped and left to swing (the pendulum test) | relaxed, a free swing through tens of degrees | **0.18 ± 0.08** and **0.31 ± 0.09**: the two half-swings the paper quotes for its controls, the ones where its two groups differed least and most (two of its figures, not the controls' whole range; corrected after session 11's review) | **FT** (excerpt) |
| Valle et al., BMC Musculoskelet Disord 2006;7:89 | 9 healthy women, 49 y; the pendulum test | relaxed | about 0.16 (0.11–0.60 over six half-swings), **mine** from a height-normalised table | **FT** (Table 2) |
| Sakaguchi et al. 2012 (a conference paper) | 2 male athletes, pendulum from 60° | relaxed; quadriceps stimulated | 0.05 and 0.35–0.37; stimulated 0.18–0.20 and 1.0–1.44 | **TAB** |
| McFaull & Lamontagne (Hum Mov Sci 1998; its conference abstract read) | 17 men, small oscillations at 10–130° | relaxed | least near 90° of bend; about 1.3 near straight (**mine** from "if the angular velocity is 10 rad/s the passive viscous moment at 10º would be about 13 N.m"); the authors call these upper limits | **FT** of the abstract; values in a figure |
| van der Kooij et al., IEEE TNSRE 2022 | 9 (26 y); pulses on the thigh in the swing phase of slow walking | walking, leg swinging | knee **0.02–0.14**; hip 1.9–4.6 | open text, excerpts |
| Zhang & Wang, J Biomech 2001;34:1107 | 9 men, knee straight, small random perturbation, **the knee bent sideways** (a direction this body's hinge does not have) | relaxed; 18 N·m | 3.6 ± 1.4; 4.0 | **FT** (excerpt) |

- **The knee's two kinds of measurement disagree by five times or more:** a leg left to swing freely (0.05–0.37, typically 0.16–0.31) against small perturbations about a held angle (1–2). A swinging leg is the nearer of the two to a swing.
- The knee's turn about the shin: no number found. Oatis 1993, Lin & Rymer 1991: not reached.

### A3. Hip

| Source | Subjects, method | Condition | B, N·m·s/rad | Read |
| :-- | :-- | :-- | :-- | :-- |
| Huang, Arami, Farkhatdinov, Formica & Burdet, IEEE TNSRE 2020 (arXiv 1908.07800) | 10 (6 women), 21–27 y; standing, knee braced, a 1.15° pull at the ankle | relaxed (EMG checked) | forward and back: "viscosity **2-21** Nm s/rad under relaxed condition"; the fit's R² 0.39–0.67 a subject; a rise with force that "only 42% cases passed the F-test" | **FT** |
| van der Kooij et al. 2022 (above) | | walking, leg swinging | 1.9–4.6 | excerpts |
| Koopman 2016 (above) | | relaxed | in a figure, not read; Huang quotes it as 0.5–10 | **2nd** |
| Amankwah, Triolo & Kirsch, JRRD 2004;41:15 (read in Amankwah's thesis, which reproduces it) | 5 able-bodied (23–29 y), the joint moved through its range at 5, 10, 60, 90°/s, relaxed | relaxed | **not a linear B:** a torque b × speed^n with n = 0.20 (hip forward and back), 0.12 (hip sideways), 0.089 (knee), 0.099 (ankle); "the influence of joint velocity on the passive joint moment at magnitudes above 10°/s was small" | **FT** (thesis, Table 2.2) |

- The hip sideways and the hip's turn about the thigh, as a linear B: **not found.** Davy & Audu 1987, Yoon & Mansour 1982, Vrahas 1990: not reached. The values 1.09 (hip), 3.17 (knee), 0.943 (ankle) used by gait models were seen third hand only; their origin (said to be Hatze's one subject) was not verified.

### A4. Trunk on the pelvis

| Source | Subjects, method | Condition | Damping | Read |
| :-- | :-- | :-- | :-- | :-- |
| Hodges et al., J Biomech 2009;42:61 (numbers through Karayannis 2013's Table 3) | 17 pain-free (9 men), half-seated, pelvis fixed, a weight released at about T9 | sitting relaxed | forward 55 (37) N·s/m, backward 91 (34) N·s/m: about **4–7 N·m·s/rad** on a 0.274 m lever (**mine**, the lever borrowed from Moorhouse) | **2nd** |
| van Drunen et al., J Biomech 2013 | 15, a force at T10 | a "relax" task | 503 N·s/m (38 N·m·s/rad on the same lever, **mine**) | excerpt |
| Moorhouse & Granata, J Biomech 2005;38:2000 (read in Moorhouse's dissertation) | 21 (10 men), standing, random force at T10 | **holding** 100–170 N of extension, never relaxed | 274–342 N·s/m; their own conversion: "27.5 ± 9.0 N-m-s/rad". **Trunk flexion–extension, not twist; the lever of the conversion is not stated** (the paper's own S1–T10 distance, 0.203 m, would give about 12.9: session 11's review), so the figure may be about twice what that lever gives | **FT** (Table 3.2) |
| Cholewicki et al. 2010 (two papers) | 20 and 14; quick release from 70–115 N | holding; reflexes included | "1.2(2.1) Ns/rad in extension, 4.8(8.2) Ns/rad in flexion, and 36.8(35.8) Ns/rad in lateral directions"; the other paper 33.4 (26.1) in extension to **47.8 (36.0) in axial rotation** (units as printed) | **FT** |
| Brown & McGill, J EMG Kinesiol 2009 | 9 men lying on a frictionless jig, quick release | graded effort | damping "predicted to be zero in 56% and 85% of flexion and lateral bend trials" | excerpt |

- **The trunk's twist, relaxed: not found.** The one number for the twist (47.8 ± 36.0) is with the trunk's muscles holding a load, reflexes included, its SD three quarters of its mean. Walsh 2003 (the lower spine's twist at resonance, 11 adults) probably holds a relaxed value and could not be opened.
- The trunk's forward and back values scatter from about 1 to 42 N·m·s/rad with SDs as large as the means.

## Part B. The arms

### B1. Shoulder

| Source | Subjects, method | Condition | B, N·m·s/rad | Read |
| :-- | :-- | :-- | :-- | :-- |
| Lipps, Baillargeon, Ludvig & Perreault, Ann Biomed Eng 2020;48:2354 | 11 (6 men), 29 y; the arm in a cast on a motor, raised 90° to the side, elbow at 90°; position steps of 0.06 rad | **at rest (0% of maximum)**; then a torque held at 5 and 10% | at rest: raising and lowering **0.61 ± 0.22**; turning about the upper arm **0.33 ± 0.10**; swept forward and back in the level plane **0.46 ± 0.16** ("at rest (0.46±0.16 Nm-sec/rad)"); **the ± are 95% confidence intervals, not SDs**; rising 0.09–0.21 per N·m of held torque | **FT** (its Table 2: the intercepts) |
| Gomi & Osu, J Neurosci 1998;18:8965 | 4 (2 men); the hand pushed 6–8 mm, forearm supported | a force held at the hand, no co-contraction; the fit's value at zero torque | shoulder 0.77 and 0.49 (two subjects); **elbow 0.97 and 0.55**; rising 0.09–0.10 per N·m at the shoulder and **0.18–0.19 per N·m at the elbow** | **FT** (Table 3) |
| Tsuji, Morasso, Goto & Ito, Biol Cybern 1995;72:475 | 4 men, 21–23; the hand moved 10 mm | "asked to relax his arm" and keep the hand in place | shoulder 0.42–1.05 (mean of 16 cells 0.71, **mine**); **elbow 0.22–0.63 (mean 0.43)** | **FT** (scan, Table 3) |
| Chang, Hunt, Atkins & Lee (an exoskeleton study) | 5; the arm held up unsupported | holding the arm against gravity | 2.5–4.0 | **FT**; a held arm with a cuff in series |
| Zhang et al., J Orthop Res 2000;18:94; Engin 1984 | | | not reached; Zhang: 0.08 per N·m (Lipps's words) | **AB**, **2nd** |

### B2. Elbow

- Gomi & Osu and Tsuji, above: **0.22–0.63 with the arm "relaxed" but holding its place** (reflexes included), 0.55–0.97 at zero torque by regression.
- Selen, Beek & van Dieën, Exp Brain Res 2006 (12 subjects, a fast elbow extension, perturbed late; **FT**): moving, about 0.30–0.38 (read off its Fig. 7, **mine**); "Stiffness and damping during maximal co-contraction ranged from 24.4 to 99.5 Nm/rad and 0.18 to 1.39 Nms/rad". It quotes Bennett 1992: 0 to 0.7 within a cycle; Popescu 2003: 0.4 to 1.2 between subjects.
- **A truly relaxed elbow: not found.** Hayes & Hatze 1977 and MacKay et al. 1986 measured it and neither could be opened (publisher fetches refused; no open copy).

### B3. Wrist, bent forward and back

| Source | Subjects, method | Condition | B, N·m·s/rad | Read |
| :-- | :-- | :-- | :-- | :-- |
| Salmond, Davidson & Charles, J Neurophysiol 2017;117:1239 (a paper that tabulates the others) | | relaxed; active | "prior measurements of passive damping in FE (range 0.014–0.09 Nms/rad, **mean 0.038** Nms/rad) (Gielen and Houk 1984; Klomp et al. 2014; Lakie et al. 1984; Park et al. 2011)"; "active damping in FE (range 0.01–0.11 Nms/rad, mean 0.069 Nms/rad)" | **FT** |
| Lakie, Walsh & Wright, J Physiol 1984;353:265 | 12 men, 11 women; the wrist driven at its resonance, about 2 Hz | **relaxed** (EMG silent) | "a damping ratio of about 0·25"; the coefficient "changes little over a wide range of torques"; read off its Fig. 7 (**mine**): men 0.10–0.14, women 0.065–0.09 | **FT** (scan) |
| Gielen & Houk, J Neurophysiol 1984;52:553 | | relaxed | 0.02–0.03 | **2nd** (Milner & Cloutier 1998; Peaden & Charles 2014); the paper **AB** |
| Halaki, O'Dwyer & Cathers, J Biomech 2006;39:2171 | 10; sinusoids of 0.4–6° | a flexor held at 15% of maximum | 0.11, 0.11, 0.11, 0.07, 0.04, 0.03 as the movement grows from 0.4° to 6°: **73% less at the largest** | **TAB** |
| Milner & Cloutier, Exp Brain Res 1998;122:309 | 10; a fast wrist flexion (500°/s) against an unstable load | moving, co-contracting | about 0.04–0.09 (its Fig. 5C, **mine**); it rose as the movement's size and speed fell | **FT** |
| Falzarano et al., Front Hum Neurosci 2021 | 13; a ramp of 0.34 rad at 100–150°/s | gripping a handle at up to 60% of maximum grip | 0.02 ± 0.01 to 0.06 ± 0.02; "did not differ significantly under the different velocity and grip conditions" | **FT** (Table 5) |

### B4. The wrist tilted toward and away from the thumb; the forearm's turn

**Never measured, as far as could be read.** Peaden & Charles, J Biomech 2014;47:2779 (**FT** of a copy): "The passive damping associated with coupled wrist and forearm rotations is unknown … damping of the wrist and forearm was assumed to be proportional to wrist and forearm stiffness (the constant of proportionality was chosen so that the damping in FE would be 0.03 Nm s/rad)". Their assumed values: the forearm's turn 0.024 (men), the wrist's tilt 0.088. Davidson & Charles 2017 assume the same way (0.021 and 0.082). Park, Chang & Kang 2017 claims a measured three-axis forearm and wrist impedance in 8 subjects; only its abstract was reached, with no numbers.

### B5. What an arm model assumes

Chadwick et al., IEEE TBME 2014 (**FT**): every arm joint "b is a small damping coefficient (1Nms/rad)", with no source. It is the value this body has carried.

**Across the arm sources:** every "relaxed" shoulder and elbow value except Lipps's is a posture held, reflexes included, so it is an upper reading of the passive value; and the wrist sources agree that the coefficient falls as the movement gets larger or faster. No source read measured a joint's damping at a swing's speeds.

## Not found

A relaxed linear damping for: the hip sideways and about the thigh; the knee about the shin; the trunk's twist and side bend; the elbow truly relaxed (Hayes & Hatze 1977, MacKay 1986: closed); the forearm's turn and the wrist's tilt (never measured); Gielen & Houk's own table. Kearney & Hunter 1990's review and Zatsiorsky 2002's table (neither reached). The passive damping a published whole-body model uses, with its source.
