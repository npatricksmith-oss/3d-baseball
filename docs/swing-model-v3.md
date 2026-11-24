# The swing model, v3: the measured swing (proposed rewrite of agent-models §3.6)

**Status (2026-11-24): specified, not built.** PLAN D25 (owner Q44 = b): the sim's swing is rebuilt whole from the measurements. This file is the rewrite of [agent-models §3.6](agent-models.md#36-swing-model) and of the lines of [physics-spec §6.3](physics-spec.md#63-bat-pose) and [rules-spec §4.5](rules-spec.md#45-check-swing) it touches (§6.6 and §8.4 below hold their proposed text). Until build step 4 (§13) switches the game to it, agent-models §3.6 (v2) describes what the code does; at that step this text replaces §3.6 there and this file is reduced to a pointer. Written by worker swing-spec on branch `work/swing-rebuild` from main `aa4a31d`; revised the same day for [the Sonnet review](reviews/2026-11-24-sonnet-swing-v3-spec-review.md), every changed number measured with `sim/target/codex-probes/v3_review_probe.py`, `v3_variants.py` and `v3_checks.py` (stdlib Python on the v2 oracle; outputs beside them).

**Sources.** [4] Fortenbaugh 2011 (33 AA hitters, live batting practice at 25 m/s, 300 Hz; the base) and [3] Welch et al. 1995 (7 professionals off a tee, 200 Hz) give every number of the swing's shape (D21). Statcast (MLB 2025 bat tracking, as fitted in [docs/calibration/2026-11-14-swing-model.md](calibration/2026-11-14-swing-model.md)) gives the contact instant, as v2 does. OpenBiomechanics is not used (D21). Every value read from [4] was checked against its text (`sim/target/research-dl/4/s.txt`, §4.1 and Appendix C's analysis code); the research notes are [docs/research/swing/](research/swing/swing-synthesis.md), and the diagnosis that led here is [launch-position.md](research/swing/launch-position.md). Today's swing at contact is measured in [docs/calibration/2026-11-24-swing-contact-baseline.md](calibration/2026-11-24-swing-contact-baseline.md) (the baseline, §2).

Tags as elsewhere: [S] sourced, [E] engineering value (named, configurable), [C] convention. "τ" is time from the nominal contact t_c (negative before). Times written "[4] −93 ms" are [4]'s own; §4.3 says how they map onto a batter's swing.

## 1. Summary

Today's swing (v2) is one rigid plane laid backward from the contact pose. It meets the ball the way MLB hitters do (Statcast) but before about 40 ms ahead of contact it is a different swing: the bat flat and pointing at the catcher where a measured hitter's is 40° up and wrapped behind his head, in line with the arms where his is cocked 120–130° behind them (diagnosis §3).

v3 keeps what v2 gets right and rebuilds the rest:

- **What moves.** The bat (the only physical body), held at the **hands** (mid-wrists, a point on the handle), whose path is built round a **shoulder point S** (the clavicle, the root of [4]'s arm vector) that follows the body. Three elements; the pelvis, trunk turn, legs and head stay presentation keyed to the sim's clocks (§3).
- **Where it starts.** A fixed **launch pose L** per batter, the same on every pitch, at rest: the bat 41° up and wrapped (azimuth −225°), cocked well behind the arms ([4]: 121°; 129° as built), the hands just in front of the back shoulder at its height ([4] at weight commitment, −170 ms) (§6.1).
- **How it meets the ball.** The **contact jet**, the pose, velocity and acceleration at t_c, is v2's plan exactly: the sweet spot at the aim point at `BatSpeed`, with Statcast's attack angle, attack direction, path tilt and vertical bat angle, and v2's reach rule (§6.2). After t_c the bat is v2's (§6.4). Before t_c the measured shape brings the bat into the contact window differently from today (§6.5).
- **Between.** Smooth curves in the bat's azimuth and elevation and in the hands' position, from L at rest through [4]'s mid-swing anchor (−93 ms: azimuth 168° short of contact and turning at 56% of its contact rate, elevation 39°; the hands at a per-batter point built from [4]'s arm angles) into the contact jet (§6.3). The lag through the swing is an outcome checked against [4] (121° → 133° max at −80 ms → 65° at contact).
- **Checked swings** brake by slowing the swing's own clock, so the bat stays on its path; the ruling measures the bat's turn from L (rules-spec §4.5's wording), so a wrapped bat is no longer "a swing" before it moves (§8).
- **Determinism and noise** are as v2: the same five execution draws in the same order on the same two streams; no new RNG purpose (§9).
- **What changes in games**: balls met before the planned instant (late swings; 23% of contact is 3 ms or more before it): 25 ms before t_c the bat is 6–9° further back, 2–3° higher and its sweet spot 2–7 mph faster than today's; 10 ms before, under 1° and within about 2 mph; at and after t_c it is today's bat. Checked swings change a lot (the braked bat is far back: about 0.6 fewer strikes a game). The recalibration should be small to moderate (§12).
- **A finding that needs data before the build** (§5.3, §14): the measured arms at contact ([4]'s shoulder angles and its 65° bat-to-arm angle) put the hands about 0.4 m in front of the batter's chest, toward the pitcher. With today's contact depth and stance spot the sim's batter would have to stand about half a metre further back in the box than he does (at about the same distance from the plate), or meet the ball that much further out in front. Either the sim's batter stands in the wrong place or its contact point is too deep (its depth level, 0.35 m, is unsourced); Statcast's batter-relative intercept settles it. The probe backs the finding: [4]'s lag curve (falling steadily from 133° to about 60° at contact) appears only with the body 0.55–0.70 m deeper; with today's stance the lag bottoms out at 20–26° before contact and rises again. Where the body stands also moves the hands' anchors and so the late path, so it is a game question, not only a look.

## 2. What v3 must keep: today's contact (the baseline)

The collision reads one thing from the swing: the bat's pose and velocity field at the contact instant, inside t_c ± 25 ms (physics-spec §7). Nothing else that decides a game reads the bat before t_c − 25 ms except a checked swing (diagnosis §2: the brake, the braked path, the ruling). The fielders read the bat's horizontal angle at t_c and the swing's lead t_plate − t_s (`game.rs:409-415`).

Today, over the standard run's 600 games (77,434 swings; [baseline](calibration/2026-11-24-swing-contact-baseline.md)):

| At contact (full swings, n 43,590) | mean | sd |
| :-- | --: | --: |
| contact − t_c | +1.2 ms | 5.6 |
| sweet-spot speed | 71.2 mph | 4.3 |
| attack angle / direction | 9.1° / +2.2° | 5.9 / 9.2 |
| bat elevation / azimuth ([4]'s convention) | −29.5° / +7.7° | 3.5 / 13.4 |
| bat against its path (hooked +) | +0.27° | 1.52 |
| knob speed / bat angular speed | 6.4 m/s / 2,046°/s | 1.0 / 142 |
| EV, LA, spray (pull +) of all contact | 84.1 mph, 16.4°, −14.2° | 15.9, 30.6, 47.2 |

Slopes: spray +3.2° and bat-against-path +0.27° per ms of contact − t_c; launch angle +10.1° per cm of undercut; EV 1.01 mph per mph of sweet-spot speed. Checked swings: 13.2% of swings, the brake at −82 ± 32 ms, 3.4% ruled swings (310 touches, 36 on the yaw).

**By construction v3 reproduces** every quantity at the planned contact instant (the jet is v2's) and every contact after it (§6.4: the bat is v2's there; 57% of contacts). **It moves** contacts before t_c, which meet a bat further back and faster than today's, by an amount that grows quickly with the timing error (§6.5: under 1° at −10 ms, 6–9° at −25 ms), and checked swings a lot (§12). It is not "v2 to second order" over the window: the first version of this spec said so, and the review measured otherwise.

## 3. The state: what moves, and why these three

| Element | What it is | Degrees of freedom | Source of its motion |
| :-- | :-- | :-- | :-- |
| **Bat** | rigid body of revolution (physics-spec §6), the only thing the world reads | 5: direction â (azimuth A, elevation E) and the hands' point W on its axis | [4] angles; the contact jet |
| **Hands** W | the mid-wrists, taken as the point of the bat's axis at u_g = 0.08 m from the knob [E] (between the two hands; the real mid-wrists sit 5–8 cm off the axis, which moves the lag by a few degrees) | (part of the bat's 5) | [4] arm angles at two instants; the contact jet |
| **Shoulder point** S | [4]'s "clavicle", the root of the arm vector a = W − S | 3, prescribed (no dynamics, no decision) | [4] pelvis travel and trunk angles; the stance spot |

Lag λ = ∠(a, â), [4]'s `BatLag()`: 0° = the bat in line with the arms, 180° = pointing back along them ([3] reports the supplement). The arm vector's turn is [3]'s "arms" segment.

**Why not fewer.** Without S there is no arm vector, so no lag: the measured relation of bat to hands, which D25 names, could not be stated or checked. Without a hands path of its own (hands fixed on a circle about one centre, as v2), the hands cannot go from beside the back shoulder at launch to 0.4 m in front of the chest at contact while the bat stays cocked. One plane for the bat (v2) cannot hold the bat 39–44° up while it turns 50° in azimuth ([4] −170 to −93 ms) and then bring it to −30°: no single plane contains that path (diagnosis §1).

**Why not more.** Pelvis, trunk turn, legs and head do not touch the bat once S's path and the hands' path are given, and nothing in the sim reads them. They stay presentation, keyed on the sim's clocks as [player-visuals §13.2](player-visuals.md#132-the-timing-model) designs (the hips and chest peak before the sim's arm and bat). The proximal half of the measured sequence (pelvis 581°/s at −70 ms, trunk 766°/s at −57 ms [4]) is therefore the client's, checked there; the distal half (arm, then bat) is the sim's, checked here (§11). Modelling them in the sim would add state that no outcome reads. If a later model drives the bat from the body (a torque-driven chain), it can replace this one behind the same interface (`BatPose`).

**Elbows** are not sim state. The distance |a| follows from the hands' path; the drawn elbows' angles follow from |a| and the figure's arm lengths and are a check (rear 81°, lead 56° at contact [4]).

## 4. Frames, definitions and time

### 4.1 Frames

The batter's frame, for a right-handed batter (a left-hander is mirrored in x, as now): x̂ toward the plate from the batter, ŷ toward the pitcher, ẑ up. It is the sim frame for a right-hander (AGENTS.md "Units and frames").

### 4.2 Angles ([4]'s definitions, from its Appendix C code)

- **Bat azimuth A and elevation E** (`BatLag()`: `BatPhi − 270`, `90 − BatTheta`): â = (cos E cos A, cos E sin A, sin E). A = 0° the bat across the plate (tip toward the plate), −90° at the catcher, −180° pointing away from the plate behind him, −225° wrapped behind his head toward the pitcher's side. A increases through the swing (counter-clockwise seen from above for a right-hander) and is **unwrapped**: continuous from L, never reduced modulo 360°.
- **Lag λ** (`BatLag()`): the angle between â (knob → cap) and the vector from the clavicle to the mid-wrists, as above.
- **Shoulder elevation θ and azimuth φ** (`PolarShoulder()`): [4]'s code takes the polar angles of the vector **from the elbow to the shoulder** joint centre in the upper trunk's frame, the same formula for both arms: θ = its angle from the trunk's long axis (so 0° = arm hanging, 90° = horizontal), φ = atan2(y, x) − 90° in axes that Appendix B's diagram defines (not yet read). This spec never uses that vector or [4]'s axes directly. It places the **shoulder-to-elbow** unit vector d in its own trunk frame (x_T forward from the chest, y_T toward the lead side, t̂ up the trunk; §5.3):

  **d = −cos θ·t̂ + sin θ·(cos φ·x_T + sin φ·y_T)** (reading 1, provisional [E]),

  so φ = 0° is the upper arm pointing forward from the chest and negative φ turns it toward the batter's rear side (the rear arm out to the side and back; the lead arm across the chest). Reading 2, the only other one the code allows, reverses the horizontal part (φ + 180°). **The check that tells them apart is [4]'s own prose**: during swing initiation "the trail upper arm dropped and moved forward" while its azimuth rose −78° → −42°, and during acceleration the lead arm went "down and forward" as its azimuth rose −24° → −1°. Under reading 1 the rear elbow's forward coordinate (along x_T, from the rear shoulder) rises +0.07 → +0.19 m from launch to −93 ms and the lead elbow's +0.30 → +0.32 → +0.32 m; under reading 2 both are reversed (the elbows move back). The oracle asserts the sign (§11). Reading 2 would also put the launch hands only 0.15–0.28 m from the clavicle, against the swing-start hands near the back shoulder the sources describe. Appendix B (step 0) settles it for good.
- **Elbow and knee**: flexion, 0° straight.

### 4.3 The swing's clock and [4]'s times

The swing starts at **t_s = t_c − T_swing** (`QuickHands`, 0.150 s at grade 50; as v2) and maps onto [4]'s **weight commitment, −170 ms** ([4] §4.1.3–4.1.4: "swing initiation", when the bat "finally began to rotate forward"). [4]'s times are scaled into the batter's swing by

**s_T = T_swing / T₄**, T₄ = 0.170 s [E: [4]'s commitment-to-contact time],

so an event [4] puts at τ₄ comes at τ = s_T·τ₄ (−93 ms → −82 ms at grade 50), and [4]'s rates are used only as ratios to the contact rate (§5), never in absolute °/s. At grade 50 the sim's swing is 12% quicker than [4]'s AA batting practice; T_swing stays the decision clock it is now (agent-models §3.4), which keeps the commit time and so the plate-appearance rates where they are. The alternative (keep [4]'s times and start the sim's bat at [4]'s −150 ms, already turning at about 500°/s) is an owner choice (§14).

The bat is **at rest at t_s** ([4]'s bat turns at 236°/s at −170 ms: it has been unwinding slowly since its minimum azimuth at −243 ms, 7° in 73 ms). v3 drops that 236°/s and starts from rest, which costs about 3° of azimuth over the first 20 ms against [4]; the gain is a launch pose the client's load can end on exactly, the same on every pitch.

## 5. The anchors

Every measured value the path must pass through (**constraint**) or is tested against (**check**). Times are [4]'s, scaled by s_T; "c" marks a value of the contact jet (§6.2). Middle pitches unless stated.

| Quantity | Value | Source (section, table) | Use |
| :-- | :-- | :-- | :-- |
| Bat azimuth at launch (−170 ms) | −225° | [4] §4.1.4 ("uncoiling from an azimuth of −225º to −175º") | constraint, L |
| Bat elevation at launch | 41° [E] | [4] §4.1.2: 44° max at the end of the stride (−340 ms); §4.1.5: 39° at −93 ms. Not stated at −170 | constraint, L |
| Lag at launch | 121° | [4] §4.1.4 ("from 121º to 131º") | target of the hands' fit (§6.1); 129° as built, a check ±10° |
| Rear elbow, rear shoulder elevation / azimuth, lead elbow at launch | 128°, 71° / −78°, 72° | [4] §4.1.4 (start-of-phase values) | constraint, builds W_L (§6.1) |
| Lead shoulder elevation / azimuth at launch, −93 ms, contact | 88° / −30°, 80° / −23°, 70° / −1° | [4] §4.1.3 (end of coiling), §4.1.4 (end of initiation), §4.1.5 | the reading check (§4.2) and the arms-at-contact check; not used to place the hands |
| Trunk (upper trunk, global) turn at launch | −22° | [4] §4.1.4: "from −22º to 2º" over −170 to −93 ms (also its maximum counter-rotation, −186 ms) | constraint, S and W_L (§5.3) |
| Trunk forward flexion, lean toward the rear side | see §5.3's table | [4] §4.1.4–4.1.5 | constraint, S |
| Bat azimuth at −93 ms | A_c − 168° | [4] §4.1.4–4.1.5: −175° at −93 ms, −7° at contact (sweep 168°) | constraint |
| Bat azimuth rate at −93 ms | 0.558·Ȧ_c | [4]: 1,247°/s at −93 ms against 2,233°/s at contact | constraint |
| Bat elevation at −93 ms | 39° | [4] §4.1.5 | constraint |
| Lag at −93 ms | 131° | [4] §4.1.4 | target of the hands' fit for the middle-middle aim (§6.1); 133° as built; an outcome for other aims (153° to 103° for contact azimuths −20° to +45°) |
| Rear elbow, rear shoulder elevation / azimuth, lead elbow at −93 ms | 126°, 49° / −42°, 68° | [4] §4.1.4 (end-of-phase values) | constraint, builds W_93 |
| Trunk turn at −93 ms, at contact | +2°, +75° | [4] §4.1.4, §4.1.5 | constraint, S |
| Pelvis forward of the stance at −170, −93 ms | +12, +17 cm | [4] §4.1.1–4.1.4: back 6, then forward 7 (stride), 11 (coiling), 5 (initiation) cm | constraint, S (§5.3) |
| Pelvis at contact | +17 cm forward [E], 5 cm higher | [4] §4.1.5 states the 5 cm rise and no forward travel in the last phase; "no travel" is this spec's assumption | constraint, S |
| Lag maximum | 133° at −80 ms (−71 ms at T_swing 0.150) | [4] §4.1.5 | check ±8°, ±20 ms (§11) |
| Lag at contact, by location | middle 65 ± 10°; HI 79, HO 63, LI 58, LO 51 | [4] §4.1.5; Table 1 (means by location) | check (§5.3), with the next row |
| Arms at contact | lead shoulder elevation 70°, azimuth −1°; rear 35° (min, at −23 ms), −9°; lead elbow 56°, rear 81° | [4] §4.1.5, Table 1 | check: the hands at least 0.25 m in front of the clavicle along x_T [E] (§5.3) |
| Peak lag (uncocking) rate | 1,588°/s at −20 ms | [3] Table 2 | check: \|λ̇\| peak within ±25% of 1,588 × Ȧ_c / 2,233°/s (scaled by the ratio of v3's contact azimuth rate to [4]'s, since [3] gives no contact rate), at −20 ± 15 ms scaled |
| Peak arm-vector turning rate | 1,160°/s at −65 ms | [3] Table 2 (arms about the trunk axis) | check ±25% of 1,160 × Ȧ_c / 2,233, at −65 ± 20 ms scaled |
| Peak bat azimuth rate | 2,435°/s at −10 ms; 2,233°/s at contact | [4] §4.1.5–4.1.6 | reported (v3's peaks 12–26 ms before contact, §6.5) |
| Bat speed (barrel end) | peak 31 m/s at −15 ms, 29 m/s at contact | [3] "Ball Contact" | shape only: v3's tip speed peaks before contact at 1.0–1.15 × its contact value, within −30 to 0 ms. Absolute values are not comparable: the sim's sweet spot is set to MLB's 71.5 mph and [3]'s tip marker is filtered at 13 Hz |
| Bat tip's speed toward the plate and downward | peaks of 19 and 16 m/s near −40 ms | [3] | check of the times (−40 ± 15 ms scaled); magnitudes reported |
| Elbows at contact | rear 81 ± 8°, lead 56° | [4] Table 1 | client check (player-visuals §13.7) |
| Bat elevation, azimuth at contact | −30 ± 5°; −8 ± 12° | [4] Table 1 | check (the jet gives −29.5°, inside one sd, and +7.7°, 1.3 sd above [4]'s mean: within two sd) |
| Contact-instant values | sweet-spot speed, attack angle α, direction ψ, path tilt β over the last 40 ms, vertical bat angle, depth y_c, reach gain g | Statcast via agent-models §3.6 v2 | constraint, the jet (§6.2) |
| Swing length | 7.2–7.3 ft | Statcast | check, with Statcast's own start-of-swing definition (§15) |

### 5.1 What the sources do not give, and what v3 assumes

Plainly, so no one mistakes an assumption for a measurement:

- **The curves between the stated instants.** [4] states azimuth, elevation and lag at −170, −93, −80 (lag only) and contact; its Appendix E graphs have the full curves and were not read (they are figures). v3 joins the stated values with the lowest-order smooth curves (§6.3) [E]. Their shape is not neutral: it is what brings the bat into the contact window faster and further back than today's (§6.5), and it lifts the elevation to 45–46° between launch and −93 ms, 1–2° above [4]'s stated 44° maximum. Reading Appendix E (step 0) would replace the interior shape with the measured one or confirm it.
- **The hands' position at any instant.** Neither [3] nor [4] gives it. v3 builds it at −170 and −93 ms from [4]'s arm angles on the figure's segment lengths, choosing the forearm's roll by a least-misfit rule (§6.1), and takes it from the contact jet at t_c. Between, it is a smooth curve [E]. The arm angles, the lag and the lead arm's length do not fit exactly together on these proportions (§6.1's residuals).
- **The elevation at launch** (41° [E], between [4]'s 44° and 39°; the probe found 39° and 44° change nothing that matters).
- **The trunk frame of [4]'s shoulder azimuth** (§4.2): reading 1 assumed, backed by [4]'s prose, to be confirmed.
- **The trunk's lean at launch and its flexion at contact**, and the pelvis's travel in the last phase (§5.3's table, [E] marks).
- **Where the body is relative to the contact point** (§5.3).
- **The hand speed at contact.** [3] and [4] do not give it; v3 keeps v2's (q = 0.32, hands at about 6.4 m/s; [3]'s arm and uncocking rates suggest 6–9 m/s, a check, §11).
- **Anything after contact.** v3's track ends at t_c + 0.03 s (D24); the client draws the follow-through.

### 5.2 Segment lengths

Upper arm 0.186·H, forearm with hand to the mid-wrists 0.146·H, half the shoulder width (clavicle to shoulder joint centre) 0.112·H, clavicle above the mid-hips 0.28·H [E: standard anthropometric fractions of height; the client's figure must use the same fractions, so the drawn arms reach the sim's hands exactly]. At H = 1.85 m: 0.344, 0.270, 0.207 and 0.518 m.

### 5.3 The shoulder point S, the trunk frame, and the body at contact

The body is built at three knots, **L** (τ = −T_swing), **93** (τ_93 = −0.093·s_T) and **c** (τ = 0), for a right-hander in the batter's frame (a left-hander is built the same and mirrored in x). Every quantity is a function of the stance reference P and the height H only: it does not depend on the aim or on whether he swings (D22).

| Knot | pelvis forward of P (fwd) | pelvis rise | trunk turn ψ | forward flexion φ_f | lean toward the rear side φ_l |
| :-- | --: | --: | --: | --: | --: |
| L | 0.12 m | 0 | −22° | 8° | 10° [E: pelvis obliquity 1° + trunk lateral flexion 9°, the −93 ms value; [4] gives none at −170] |
| 93 | 0.17 m | 0 | +2° | 13° | 23° (obliquity 14° + lateral flexion 9°) |
| c | 0.17 m [E] | 0.05 m | +75° | 15° [E: 13° at −93, 16° at +30 ms] | 31° (obliquity 27° + lateral flexion 4°) |

Sources: [4] §4.1.1–4.1.5 (pelvis travel, rise and obliquity, upper trunk global turn, flexion and lateral flexion). Lateral flexion is [4]'s relative to the pelvis and obliquity the pelvis's own tilt; adding them for the trunk's lean is an approximation [E].

At each knot:

- mid-hips **O** = (P_x, P_y + fwd, 0.50·H + rise) [E: 0.50·H is the hip-joint height with the knees at [4]'s 40–50° of flexion];
- facing **f** = (cos ψ, sin ψ, 0) (ψ = 0°: the chest faces the plate, +90°: the pitcher); rear side **r** = (sin ψ, −cos ψ, 0) (the batter's right);
- trunk axis **t̂** = cos φ_l·(cos φ_f·ẑ + sin φ_f·f) + sin φ_l·r (flexion first, then the lean; t̂ is a unit vector because f, r and ẑ are orthonormal);
- **S** = O + 0.28·H·t̂ (the clavicle);
- **r̂′** = normalize(r − (r·t̂)·t̂); trunk frame **x_T** = t̂ × r̂′ (forward from the chest), **y_T** = t̂ × x_T (toward the lead side);
- rear shoulder = S + 0.112·H·r̂′, lead shoulder = S − 0.112·H·r̂′.

At H = 1.85 m and today's P = (−0.95, 0.30, 0): S_L = (−0.918, 0.310, 1.430), S_93 = (−0.836, 0.271, 1.390), S_c = (−0.663, 0.512, 1.404) m. (At contact the rear side faces the plate, so the 31° lean moves the clavicle 0.27 m toward it.)

**S(τ)** is the quadratic in τ through the three knots (Lagrange form, each coordinate), on [−T_swing, +0.03 s]; the knots scale with s_T like every other anchor. It is exported so the client puts the figure's clavicle there (§10). Only S(τ) is needed at run time; the shoulders and the trunk frame are needed at L and 93 to build the hands (§6.1) and at c for the checks.

**The finding.** With the jet's bat for a middle pitch (elevation −31°, azimuth +5°, the sweet spot at the aim) the hands (mid-wrists) are at about (−0.55, 0.30, 1.06) m. [4]'s arms at contact (both arms in front of the chest, reading 1) and its 65° lag need the clavicle about **0.4 m behind the hands toward the catcher and 0.3 m above them**, about level with them across: the hands in front of the belly, the arms pointing down and toward the pitcher, the bat across the plate. S_c above is about right across but **0.5–0.6 m too far forward**: the hands sit 0.06–0.08 m *behind* the clavicle along x_T, which no hitter does. The probe shows what that does to the whole curve (`v3_checks.py`, `v3_stance_sweep.txt`):

| Stance P_y (today 0.30 m) | lag at −82 / −40 / −20 / 0 ms (middle) | uncocking peak, against [3] scaled | hands ahead of the clavicle at contact |
| :-- | :-- | :-- | --: |
| +0.30 | 133° / 50° / 22° / 51° (minimum 20–26° about 25 ms before contact, then rising) | 2,550–2,900°/s at −39 to −50 ms: 1.7 times too fast | −0.06 to −0.08 m |
| −0.10 | 133° / 85° / 58° / 45° | 1,760–1,780°/s | +0.30 m |
| −0.25 | 133° / 99° / 75° / 53° | 1,350–1,480°/s, inside the bound | +0.44 m |
| −0.40 | 133° / 112° / 87° / 59° | 1,180–1,670°/s | +0.58 m |

So [4]'s lag curve appears when the batter stands 0.55–0.70 m deeper than today. Moving him also moves the hands' anchors W_L and W_93 with his body, which changes the late path itself: at P_y = −0.25 m the bat tip's speed peaks about 24 ms before contact at about 1.08 times its contact value, which is [3]'s shape (1.07 at −15 ms), where today's stance gives a peak at contact. **Where the batter stands is therefore a question about contact, not only about looks.**

So either the sim's batter stands in the wrong place (the stance spot, 0.30 m from the plate's back tip toward the pitcher, is set by eye) or the sim's contact is too deep (contact depth y₀ = 0.35 m is [E]; only its slope across the plate came from Statcast). Statcast's bat tracking records the intercept relative to the batter (`intercept_ball_minus_batter_pos_x_inches`, `_y_inches`, used for the slopes in agent-models §3.6 and `analysis/statcast/savant_bat.py`); the sample's CSVs are not on this machine. **Step 0 (§13) re-pulls them, reads Savant's definition of the batter's position, and computes the mean intercept relative to the batter**; then the owner chooses how to place the batter (§14). Until then the lag and arms-at-contact checks are expected to fail with today's stance, and the rest of this spec does not depend on the choice.

## 6. The path

### 6.1 The launch pose L and the hands' anchors

The hands are built at L and at 93 from [4]'s arm angles on the segment lengths of §5.2, in the body of §5.3. Both are **per batter**: neither depends on the aim, so every swing has them and they are continuous across aims. The construction has no root to find at run time.

1. **Rear elbow** = rear shoulder + 0.186·H·d, with d from §4.2 (reading 1) at the knot's rear-shoulder elevation and azimuth (L: 71°, −78°; 93: 49°, −42°).
2. **Forearm**: with p̂ = normalize(t̂ − (t̂·d)·d) and q̂ = d × p̂, the hands are **W(χ)** = elbow + 0.146·H·[cos κ·d + sin κ·(cos χ·p̂ + sin χ·q̂)], κ the rear elbow's flexion (L: 128°, 93: 126°) and χ the forearm's roll about the upper arm, which [4] does not give.
3. **χ by least misfit** [E]: χ minimises

   M(χ) = ((λ(χ) − λ*)/10°)² + ((|W(χ) − lead shoulder| − R)/(0.03·H))²,

   λ(χ) = ∠(W(χ) − S, â) with the knot's bat (L: A = −225°, E = 41°; 93: A = A_ref − 168°, E = 39°, where A_ref is the contact azimuth of the batter's own **middle-middle** aim, +4.8° for the template), λ* = [4]'s lag (121°, 131°), and R = the lead arm's shoulder-to-wrist distance at [4]'s lead elbow flexion (72°, 68°): R = √(l_u² + l_f² + 2·l_u·l_f·cos κ_lead), l_u = 0.186·H, l_f = 0.146·H. Among the local minima keep those with the hands in front of the rear shoulder ((W − rear shoulder)·x_T ≥ 0: "hands just in front of the back shoulder"), and take the lowest. The 10° is [4]'s spread of the lag at contact; the 0.03·H is about 5 cm.
4. **Constants.** Every length is a multiple of H, so χ does not depend on H: χ_L and χ_93 are template constants, computed once by the oracle and stored with their derivation in a test. The probe gives **χ_L = 308.2°, χ_93 = 309.4°** (the forearm keeps nearly the same roll, so the hands' path is smooth), and at H = 1.85 m, P as today: W_L = (−0.872, −0.124, 1.401), W_93 = (−0.576, 0.069, 1.221) m.
5. The knob K_L = W_L − u_g·â_L.

**Residuals, reported by the oracle and not hidden:** at L the lag comes out 129° (target 121°) and the lead shoulder is 0.625 m from the hands, longer than the lead arm at 72° (0.499 m) and 1 cm beyond even a straight lead arm (0.614 m): on these proportions the lead arm cannot quite reach the hands the rear arm and the lag put by the back shoulder. At 93 the lag is 133° (131°) and the lead arm 1.6 cm long. The cause is in the inputs that are not measured (the proportions, the clavicle taken on the shoulder line rather than at the front of the chest, the trunk frame); step 0 and the client's figure settle them. The client's reach solver (`FBbReach`, up to 24 cm) covers it in the picture meanwhile.

**Why not the lag alone, or both arms alone** (the first version solved the cone for the lag per swing): solving the lag at 93 with the swing's own bat makes the hands depend on the aim, and the solution can vanish (the review found none for contact azimuths of 27° and more without the trunk's lean; with the lean included the probe finds roots up to +45°, but the margin is thin and the baseline's azimuths reach past it); there can be up to four roots, and none. Solving the two arms alone (the rear arm's cone meeting the lead arm's sphere) has no solution at L on these proportions (6.5 cm short) and gives a lag of 147°. The misfit is always defined, chooses continuously, and leaves the lag where it belongs, as an outcome checked against [4].

*Picture:* at L the hands are at shoulder height, 0.21 m in front of the rear shoulder and 0.43 m from the clavicle, the rear elbow out and back, the bat 41° up and wrapped toward the pitcher's side. Today's swing starts with the hands 0.57 m behind his spot at chest height and the bat flat at the catcher (player-visuals §13.10).

### 6.2 The contact jet

At the executed nominal contact t_c the bat's state and its first two time derivatives are **v2's** (agent-models §3.6 v2, steps 1–8 and Placement), computed exactly as today by `plan_swing` and `apply_placement`:

- pose: knob K_c, axis â_c;
- velocity: knob velocity K̇_c, angular velocity ω_c = θ̇(0)·n̂;
- acceleration: K̈_c and ω̇_c = θ̈(0)·n̂, from v2's two rate bumps at τ = 0.

From these, at τ = 0: W, Ẇ, Ẅ (the point u_g along the bat); A_c, E_c and their first two derivatives (from â, â̇ = ω × â, â̈ = ω̇ × â + ω × (ω × â)).

This is how v3 keeps "match by construction" (§2) **at the planned instant**: there the sweet spot is at the aim, at `BatSpeed`·(1 + effort), with α, ψ, the plane tilt β and the vertical bat angle; the hands at q's share; the bat square to its path (v2's λ_c = 0, which is a statement about v2's hand-arc centre, not [4]'s lag); and the sweet spot's speed still rising (v2's τ_b = +10 ms: Statcast's bat speed rises with contact depth out to 6–12 in out front). v2's template keeps its role as the **contact planner**: its widths and peak times now set only the jet and the path after t_c (§6.4). Before t_c the curves of §6.3 decide, and they do not reproduce v2's rates there (§6.5).

### 6.3 Between: launch to contact

On τ ∈ [−T_swing, 0], three groups of curves, each a polynomial in τ:

| Curve | Conditions | Degree |
| :-- | :-- | :-- |
| A(τ) | at −T_swing: A_L, Ȧ = 0, Ä = 0; at τ_93 = −0.093·s_T: A_c − 168°, Ȧ = 0.558·Ȧ_c; at 0: A_c, Ȧ_c, Ä_c | 7 |
| E(τ) | at −T_swing: E_L, 0, 0; at τ_93: 39°; at 0: E_c, Ė_c, Ë_c | 6 |
| W(τ), each of x, y, z | at −T_swing: W_L, 0, 0; at τ_93: W_93 (per batter, §6.1); at 0: W_c, Ẇ_c, Ẅ_c | 6 |

- **Closed form.** With u = τ/T_swing the interior knot sits at the fixed fraction u_93 = −0.093/0.170, the same for every batter, so each curve is a fixed linear combination of the conditions: the basis polynomials are computed once (in the oracle, and as constants in Rust with their derivation in a test), and each swing's coefficients are a dot product. No per-swing linear solve.
- **Pose at τ:** â = (cos E cos A, cos E sin A, sin E) (mirrored for a left-hander), W from its curve, knob K = W − u_g·â. **Velocity:** â̇ from Ȧ, Ė; ω = â × â̇ (the bat has no roll about its own axis, as in v2); the centre of mass moves at Ẇ + ω × ((u_cm − u_g)·â). All transcendental functions through `bb_math::m`.
- **Why these conditions.** At rest at the start, so the client's load ends on L without a jump; [4]'s mid-swing values where [4] states them; the jet at contact, so the planned contact is v2's. Ȧ at −93 ms is a ratio to the contact rate (not 1,247°/s) so that a faster swing is faster all through. The elevation passes through 39° at −93 ms; between launch and there it rises to 45–46° (1–2° above [4]'s 44° maximum). Ė = 0 at −93 ms would cap it at 41°, but it pushes the elevation's fall later and makes the sweet spot reach 74 mph 25 ms before contact, faster than at contact, so it is not used (probe `v3_variants.py`). The lag between the anchors is left to fall out and is checked (§11).
- **Why not join today's path before the window** (the review's other remedy): the measured mid-swing (168° of azimuth still to go 82 ms before contact, turning at 56% of the contact rate) cannot be joined to v2's state 30 ms before contact (only 63° to go, turning at 1,616°/s) without the bat speeding up, slowing to 1,450–1,600°/s, and speeding up again: two surges in one swing (probe: rate peaks at −72 ms and again at contact). Dropping the −93 ms anchor does not cure it (the wrapped launch is 110° behind v2's own start, so the deficit must be made up before the window). A condition on the third derivative at contact does not help either (the difference at −25 ms stays 7–9.5°). So v3 keeps the measured shape up to the planned instant, and §6.5 states the difference; the owner can choose otherwise (§14).
- **What must be verified** (the oracle does it before any Rust, over the whole aim range of §11): A non-decreasing on [−T_swing, 0] (true for the three golden aims; the rate has a shallow plateau near −90 ms); E within [−45°, 50°] (probe: −38° to 46°); the hands between 0.15 m and the arms' full reach from S; the sweet spot's speed never above 1.1 times its contact value before contact; the lag checks of §11. If a curve misbehaves somewhere in the range, the fix is an interior condition from [4]'s stated values, not a different shape family.

### 6.4 After contact

For τ ∈ [0, +0.03 s] (D24) the bat follows v2's analytic swing (the bumps continued past t_c), which joins the curves of §6.3 with matching position, velocity and acceleration at τ = 0. Contacts found after t_c (57% of contacts today, all within +25 ms) therefore meet exactly v2's bat; the after-contact motion is otherwise cosmetic (physics-spec §6.3) and the follow-through is the client's.

### 6.5 Where v3 differs from v2 inside the contact window

At τ = 0 the curves and v2 share position, velocity and acceleration; after it the bat is v2's (§6.4). Before it they differ, and not by a small third-order term, as the first version of this section estimated (0.1° at −10 ms, 2° at −25 ms). Measured on the oracle's v2 jet (`v3_review_probe.py p21`, the three golden aims, today's stance):

| aim | v3 − v2 azimuth at −10 / −25 ms | v3 − v2 elevation at −25 ms | sweet-spot speed v3 / v2 at −25, −15, −10 ms (mph; 71.6 at contact) | v3's azimuth rate peak |
| :-- | :-- | :-- | :-- | :-- |
| middle | −0.7° / −7.6° | +2.7° | 66.9 / 62.8, 69.9 / 68.0, 70.8 / 69.9 | 2,546°/s at −20 ms (contact 2,349) |
| inside-low | −0.4° / −5.8° | +1.9° | 64.3 / 62.2, 67.4 / 67.6, 69.1 / 69.6 | 2,679°/s at −12 ms (contact 2,527) |
| outside-high | −0.9° / −9.2° | +3.3° | 70.5 / 63.5, 72.5 / 68.5, 72.4 / 70.1 | 2,535°/s at −26 ms (contact 2,172) |

The reviewer's numbers are reproduced exactly. The reason is the measured shape itself: [4]'s bat still has 168° to turn 82 ms before contact, and v2's has 111°, so v3's bat must turn faster than today's in the last 80 ms, and its azimuth rate peaks 12–26 ms before contact, as [4]'s does (−10 ms). **In v3 the lab's peak is built in**, whatever the owner chooses about speed at contact.

**What it does to contact.** Contacts after t_c (57%, the early swings) are unchanged. Contacts before it (late swings) meet a bat further back, higher and faster: from the baseline's timing table, 1,003 contacts in 600 games (2.3%) come 10–26 ms before t_c, where the bat is 3–9° further back and 2–7 mph faster; 8,885 (20%) come 3–10 ms before, where it is under 1° back and up to 2 mph faster. Expected: late balls hit a little harder and pushed more to the opposite field, some swings at the window's early edge turning misses into fouls or the reverse; foul share and exit speed of late contact move, the totals probably within a point (to be measured, §12). Against Statcast's speed by contact depth (bat speed about 3.4 mph under the batter's mean for balls met 12 in or more deep, about −15 ms), v2 gives −3.6 mph at −15 ms and v3 −1.7 (middle aim): **v3 keeps about half of MLB's loss of speed on late contact**. That is a Tier M check (calibration-targets §4.3) for build step 4; if it fails, the fix is the interior shape from [4]'s Appendix E, not a speed knob.

### 6.6 physics-spec §6.3, as proposed

Replace the v2 paragraph with: *"The swing model's v3 (agent-models §3.6, from docs/swing-model-v3.md) gives the bat's direction by its azimuth and elevation and the hands' point W on its axis at u_g from the knob, each a smooth function of time: **V** = **Ẇ** + **Ω** × ((u_cm − u_g)·**â**), with **Ω** = **â** × **â̇** (no roll about the bat's axis). Near contact the motion is the planned contact state of v2 (hands on an arc in the swing plane, the bat turning in it)."* The free-bat approximation is unchanged.

## 7. Attributes, decisions and noise

| Input | Enters as | Changes from v2 |
| :-- | :-- | :-- |
| `BatSpeed` | the jet's sweet-spot speed (k) | none |
| `QuickHands` → T_swing | t_s = t_c − T_swing; s_T scales [4]'s times | the launch is still at rest at t_s and the anchors scale |
| Height H | L, W_93, S (segment fractions, §5.2) | new |
| `BatControl`, `Timing` | σ of δ_v, α, δ_r, t₀ (motor) | none |
| `Adjustability` | the bounded mid-swing move of the aim | none |
| Effort (4.5%) | multiplies the jet's rates (v2: both bumps) | the launch no longer moves with effort (in v2 the rates' integrals moved the start pose) |
| The batter's decisions | swing or take, aim point and time, mid-swing corrections, hold-up (check) | none: the decision code (`batter::decide`, pa.rs's loop) is unchanged and never reads the swing's path |

**Noise stays where it is** (invariants 1–3): `motor::execute_swing` draws δ_v, α, δ_r, t₀ from `SwingExecution` in that order and the effort from `SwingEffort`, and `apply_placement` applies them to the plan, exactly as now. v3 changes only what a planned and placed swing looks like between L and t_c. L, S and the anchors are noise-free: the load is the same on every pitch (D22, D23), and its small real variation is a look for the client. No outcome is rolled.

**Mid-swing adjustment** (pa.rs) re-plans the jet every 10 ms until the cutoff, as now, and v3 rebuilds the curves from L to the new jet each time, with t_s = t_c − T_swing and `start_no_earlier_than(t_commit)` exactly as v2 (so the contact time is unchanged). A re-plan also changes the path already travelled, as v2's does; nothing reads the bat's past (a brake reads the current swing at the brake instant), and the timeline encodes only the final swing, so nothing is seen.

## 8. Early, late, checked swings and the ruling

### 8.1 Early and late swings

Timing is the jet's t_c against the ball, as now: an early swing meets the ball after t_c with the bat further round (pulled, hooked), a late one before it (pushed, sliced), with the slopes of §2. A swing made late by a late decision starts later (`start_no_earlier_than`) and is the same swing shifted, as now. The client's body timing against Δ = t_c − T_a (player-visuals §13.2) is unchanged.

### 8.2 The brake

A check swing still starts braking at t_b = (the tick when EV_swing − EV_take < −θ_check) + latency, only before t_s + 0.6·T_swing (as now). v3 brakes by slowing **the swing's own clock**: the pose at t is the path at

σ(t) = t for t ≤ t_b; σ(t) = t_b + s − s²/(2·B) with s = min(t − t_b, B), B = `BRAKE_S` = 0.040 s,

so σ advances by B/2 = 20 ms of path while slowing to a stop, the bat and hands stay on their planned path, and every velocity is the path's times σ̇ = 1 − s/B. (v2 stops each rate bump from its value at t_b, which carries the bat off its path; v3's form is the simpler and the more physical: the same move, slowed.)

### 8.3 What a checked swing does now

The brake comes at −82 ± 32 ms (baseline), and never after t_c − 10 ms (the abort point, 60 ms before contact, plus the 50 ms latency). At −82 ms v3's bat is about 168° short of its contact azimuth (pointing away from the plate and up), and 20 ms more of path brings it to −128° to −142° (probe, three aims): well back. A brake at −25 ms still stops it at −12° to −1°; only brakes in the last 10–20 ms before the cut-off (the baseline's p95 is −29 ms) can carry it past 0°, and then only on inside pitches. A braked v3 bat almost never touches the ball and almost never passes 0°. Today 3.4% of checked swings are strikes (310 touches, 36 on the yaw), about 0.6 a game; v3 should make nearly all of them balls. No source here measures how often a real check swing is ruled a swing; this is reported, and the owner may want it looked at (§14).

### 8.4 rules-spec §4.5, as proposed

Replace the first sentence with: *"A batter has **swung** if the bat's horizontal direction, turned from its pose at the start of the swing in the swing's own direction, reaches the point where the bat is perpendicular to the pitcher–home line **with its barrel toward the plate** (the barrel head level with the front of the batter) [C]. The swing model reports the furthest azimuth reached, A_max (agent-models §3.6: unwrapped from the launch pose, 0° = across the plate), and the batter swung if A_max ≥ θ_swing = 0° [C, configurable]. A bat that starts wrapped behind the head passes the other perpendicular (pointing away from the plate, −180°) early in every swing; that one does not count."* The rest of §4.5 (any contact is a swing) stands.

In code: `Swing::judged_swing` becomes A(σ(t_b + B)) ≥ 0 on the unwrapped azimuth, and `max_angle` reports A_max; today's `a.y ≥ 0` test is only right for a bat that starts pointing at the catcher, and would call a wrapped bat a swing before it moves (diagnosis §2).

## 9. Determinism

- Every function is closed form in τ: polynomials, `m::sin`/`m::cos` for â, one `m::atan2`/`m::asin` for A and E of the jet. W_L and W_93 are built from the constants χ_L, χ_93 (§6.1) by closed-form vector arithmetic; nothing is solved at run time. No `mul_add`, no iteration that depends on a tolerance reached.
- No new RNG purpose. The draw order of `SwingExecution` (δ_v, α, δ_r, t₀) and `SwingEffort` is unchanged; nothing new is drawn.
- The basis polynomials of §6.3 are constants with a test that rederives them from their conditions.
- Same build + seed ⇒ same game, as always; v3 changes games once (build step 4), with a ledger row.

## 10. What the timeline exports

- **E1 stays**: `swing_start_s` (t_s) and `swing_contact_s` (t_c).
- **The bat track** samples v3's bat every 5 ms from t_s to t_c + 0.03 s (D24), interpolated by the FFI as now. **Before t_s the bat is at L**, at rest: the sim's 50 ms straight-line load (`LOAD_S`) and its stance sample become presentation the client no longer needs, since its own load (player-visuals §13 step 1) ends exactly on L. The handover of step 1 (closing up to 48 cm in 50 ms) disappears.
- **New, E2 (needed so the client can put hands and body on the sim's segments):**
  - the launch pose L (knob, axis) per batter, in `BbPlayInfo` (so the load can end on it before the swing exists, and on a take);
  - the shoulder point S at each bat sample (a parallel vector in `BatTrack`, appended), so the figure's clavicle is put there;
  - the grip station u_g (one constant per bat).
  One schema change (`./schemas/gen.sh`), `BB_ABI_VERSION` up by one, an FFI test; no digest changes (the timeline is not in the event log): a "No effect on games" ledger line.
  **While v2 is the game's swing** (if step 3 lands before step 4): the new fields are written empty (`BbPlayInfo` flags L as absent, `BatTrack` has no S samples), since v2 has no launch pose or shoulder point; the client keeps its step 1 behaviour. They are filled from step 4 on.
- The `CheckSwingBrake` marker (in the schema, never written) is written at t_b, so the client stops the body with the bat.

## 11. Goldens and the oracle

The Python oracle (`analysis/oracle/swing_oracle.py`, stdlib only) gains a v3 module that computes everything below **from this spec**, independently of the Rust; its fixtures go in a new `swing_v3.json` beside `analysis/oracle/golden/swing.json` and `sim/crates/bb_agents/tests/` reads them. v2's oracle and golden stay as the contact planner's (the jet is v2's), so the jet is tested against v2's existing fixtures.

**Golden cases** (a right-hander, H = 1.85 m, grade 50: T_swing 0.150 s, `BatSpeed` 32.0 m/s, effort 0; the 34-in bat; and a left-hander mirrored). Each is computed by the oracle from this spec and compared with the Rust; the conditions the construction imposes are not themselves tests (AGENTS.md: no tautological tests), so each case below names what it can catch.

1. **A hand-worked case** (catches frames, signs, the reading of §4.2 and the χ convention; worked on paper, not by the oracle): S = (0, 0, 0); an upright trunk facing +x (ψ = φ_f = φ_l = 0, so t̂ = ẑ, x_T = +x, y_T = +y, r̂′ = −y); H = 1.85 m (l_u = 0.3441, l_f = 0.2701, half-shoulder 0.2072 m). Rear shoulder (0, −0.2072, 0). Rear arm θ = 90°, φ = −90°: d = −y, elbow (0, −0.5513, 0). Forearm κ = 90°: p̂ = ẑ, q̂ = d × p̂ = −x, and χ = 270° gives the forearm +x, so **W = (0.2701, −0.5513, 0)**, |W − S| = 0.6139 m, **hands at shoulder height**. With the bat at A = −90°, E = 45° (â = (0, −0.7071, 0.7071)): **λ = acos(0.5513 × 0.7071 / 0.6139) = 50.58°**. Under reading 2 the same inputs give the elbow at (0, +0.1369, 0), W = (−0.2701, 0.1369, 0) and λ = 108.6°, so the case tells the readings apart.
2. **The reading check** (§4.2): from L to 93 the rear elbow's forward coordinate (along x_T, from the rear shoulder) rises (probe: +0.07 → +0.19 m), and the lead elbow's rises from 93 to c. Fails under reading 2.
3. **The jet** at t_c for v2's golden cases `rhb_middle`, `rhb_inside_low`, `rhb_outside_high` and `lhb_middle_quick` (`golden/swing.json`): equal to v2's pose and velocity there to 1e-9 (an independent fixture: v2's oracle), and the acceleration to v2's bumps' to 1e-6 (finite differences).
4. **Path samples** every 5 ms from t_s to t_c + 0.03 for the three aims: knob, axis, Ω, the centre of mass's velocity, Rust against the oracle within 1e-9 m and rad (two implementations of the same spec: catches implementation errors, not spec errors).
5. **Continuity and rest**: position, velocity and acceleration equal from both sides at τ = 0 to 1e-9; all rates zero at t_s.
6. **Braking**: brakes starting at −120, −82, −40, −25 and −12 ms: the stopped azimuth and the ruling. From the probe: −82 ms stops at −135° (middle), −25 ms at −7° (middle), −1.4° (inside-low), −12° (outside-high); all no swing. The −12 ms case's answer comes from the oracle.
7. **T_swing** 0.130 and 0.174: the anchors land at s_T·(−93 ms) and the curves stay within the §6.3 bounds.
8. **Over the whole aim range** (the P1 lesson): a grid of aims across and up the zone and 0.15 m outside it, contact azimuths from −25° to +50°: every path defined, A non-decreasing, E within [−45°, 50°], the hands within reach, no sweet-spot speed above 1.1 times its contact value before contact, and the lag at 93 reported (probe: 153° at −20° to 103° at +45°).

**Bounds from the sources** (outcomes the construction does not impose; each test fails outside its bound; tolerances [E], wide enough for a model not fitted to them, and stated with what the probe gives today):

| Check | Bound | Source | Probe today (today's stance / stance 0.55 m deeper) |
| :-- | :-- | :-- | :-- |
| Lag maximum | 133 ± 8° at −71 ± 20 ms (−80 ms scaled) | [4] §4.1.5 | 130–139° at −88 to −90 ms (passes narrowly on time) / same |
| Peak uncocking rate \|λ̇\| | within ±25% of 1,588 × Ȧ_c / 2,233°/s, at −18 ± 15 ms scaled | [3] Table 2, scaled by [4]'s contact rate (§5) | 2,550–2,900°/s at −39 to −50 ms (fails) / 1,350–1,480°/s (passes in size; time −23 to −50 ms) |
| Peak arm-vector turning rate (horizontal, about S) | within ±25% of 1,160 × Ȧ_c / 2,233°/s, at −57 ± 20 ms scaled | [3] Table 2 | 1,250°/s at −104 ms (time fails) / 1,430–2,700°/s at −38 to −46 ms (size fails) |
| Bat tip speed shape | peak before contact, 1.0–1.15 × the contact value | [3] | at contact (fails) / 1.08 at −24 ms (passes) |
| Tip toward the plate and downward | peaks at −35 ± 15 ms scaled | [3] (−40 ms) | toward the plate −36 to −39 ms (passes); downward −49 to −51 ms (at the edge) |
| Lag at contact, by location | middle 65 ± 15°; HI 79, HO 63, LI 58, LO 51, each ± 15° | [4] Table 1 | 30–62° / 53–54° |
| Hands ahead of the clavicle at contact | ≥ 0.25 m along x_T | [4] arms at contact (§5) | −0.06 to −0.08 m (fails) / +0.43 to +0.46 m |
| Plane tilt over the last 40 ms | within 1.5° of β | Statcast (the jet) | to be measured |
| Swing length | reported against Statcast with its own start definition (§15) | Statcast | 8.1–9.1 ft from t_s |

The failures with today's stance are §5.3's finding, not defects of the construction; they are expected to pass once the body is placed (§14). The arm-vector rate fails either way: the oracle reports it and step 0 (Appendix E's curves) is the first place to look.

**Property tests** (Rust): |â| = 1; the velocity the pose reports equals the finite difference of positions (as v2's test); A non-decreasing on every unbraked swing; a braked swing never moves after t_b + B; mirror symmetry for a left-hander; determinism (the same plan twice gives identical bits).

**Game-level tests**: a test that every swing's contact quantities at the planned instant (sweet-spot speed, α, ψ, bat elevation) equal v2's for the same plan, and that the bat after t_c is v2's; the `bb_game` crate-graph test unchanged.

## 12. The recalibration plan

**Expected to move, and why:**

| Measure | Expected | Why |
| :-- | :-- | :-- |
| Checked swings ruled swings | 3.4% → near 0 | the braked bat is far back (§8.3) |
| K% | −0.1 to −0.3 points | about 0.6 strikes a game become balls |
| BB%, pitches per PA | +0.1 to +0.2 points; up slightly | the same pitches |
| EV, LA, spray, foul share of late contact (3 or more ms before t_c, 23% of contact) | late balls a little harder (up to +2 mph at −10 ms, +2–7 at −25) and pushed; foul share there moves | the measured shape brings the bat in faster and further back (§6.5) |
| EV, LA, spray of all contact; foul share | probably within a point; measure | early contact (57%) is unchanged; late contact is the minority |
| Contact%, whiff% | small changes at the window's early edge | the bat is up to 9 cm elsewhere at −25 ms |
| Bat speed by contact depth (Tier M) | deep contact loses about half of MLB's 3.4 mph | §6.5 |
| HR/PA, runs | within noise, follow K% and BB% | |
| HBP% and everything perceptual | unchanged **unless** the owner moves the batter (§14) | the eye point and body capsules follow the stance spot |

**Match by construction** (§2): every quantity at the planned instant, and every contact after it.

**Bands** (calibration-targets): §4.3 (bat speed 71.5–72 mph, swing length about 7.3 ft, attack angle, commit-to-contact 0.15 s), §4.4 (EV 88.5, LA 12°, hard-hit, GB/LD/FB/PU), §5 (K% 22.5, BB% 8.3, Swing%, Contact%, foul share), and §6.2 (runs 4.39).

**Order of work**, bottom-up (calibration-targets §2.1):

1. **Tier A/M first**: the oracle and goldens pass; then the cage (`swing_cage_probe`, and `swing_contact_baseline` against [the baseline](calibration/2026-11-24-swing-contact-baseline.md)): the timing table's rows from −3 ms on identical within noise, the earlier rows reported with §6.5's expectation beside them, and bat speed by contact depth against Statcast.
2. **The standard run**, before and after (ledger row). If K%, BB% and runs are within noise or move as the table says, stop: no retuning.
3. **If something moves more than expected**, look for a bug first (a planner or path error, AGENTS.md), then the checked-swing mechanism (§8): the brake and the ruling are the only new physics on the game path.
4. **Only then retune**, and only what calibration-targets §2.3 allows: `K_swing` (the miss scale) and the `BatSpeed` population mean, in that order. **Never** the anchors, the jet's Statcast fits, or a checked-swing threshold toward a stat. If late contact is off a Tier M band (speed by depth, spray by timing), the fix is the interior shape from [4]'s curves (step 0), not a knob. Swing length, if it leaves 6–8.5 ft, is a Tier M finding about the path's shape (§15), fixed in the curves' conditions from [4], never through speed.
5. If the owner moves the batter or the contact depth (§14), that is its own calibration step after this one: perception, HBP and the late path (the hands' anchors move with the body, §5.3) change with it, and K%, BB%, HBP%, late contact and the timing of contact are re-measured; a calibration report under docs/calibration/ names each change and its band.

## 13. Build order

Each step testable and reviewable on its own. Sizes are rough Opus working time.

| Step | What | Game change | Size | Review |
| :-- | :-- | :-- | :-- | :-- |
| **0. Data** | Read [4] Appendix B (the shoulder angles' trunk frame) and, if legible, Appendix E's curves (azimuth, elevation, lag between the anchors). Re-pull the Statcast sample (calibration note 2026-11-14 §8 has the query) and compute the intercept relative to the batter (x, y), with Savant's definition of the batter's position. Write the numbers into §4.2, §5 and §5.3 of this spec, and the owner's choices of §14 are taken with them. | none | half a day | none (data; the Coordinator reads it) |
| **1. Oracle** | `swing_oracle.py` v3: the body (§5.3), χ_L and χ_93 by the misfit (§6.1), the jet (v2's), the curves, braking; goldens §11 cases 1–8 and the bounds table; settle any misbehaviour over the aim range here (§6.3). Start from `sim/target/codex-probes/v3_review_probe.py`, which already builds most of it. | none | 1 day | Opus, with the spec |
| **2. Rust, behind a switch** | `bb_agents::swing`: v3 path as a `BatPose` beside v2 (`SwingTemplate::model`, default v2); the clock-based brake, A_max and the new ruling apply to v3 only; tests from the goldens. Digests unchanged (v2 default). | none | 1 day | Opus (sim) |
| **3. Export E2 and D24** | L in `BbPlayInfo`, S samples in `BatTrack`, u_g, the `CheckSwingBrake` marker, the track's end at t_c + 0.03 s; schema, FFI, tests. Empty while v2 is the game's swing (§10). Ledger: "No effect on games". | none | half a day | Sonnet |
| **4. Switch the game to v3** | default v3; the ledger row (standard run before/after); the baseline example against the baseline; the checked-swing counts; the recalibration of §12 if needed; agent-models §3.6 replaced by this text, physics-spec §6.3 and rules-spec §4.5 updated. | yes | 1 day + runs | Opus (sim) |
| **5. Where the batter stands** (if the owner chooses to move him, §14) | stance spot and/or contact depth, eye point, body capsules; its own ledger row and calibration report. | yes | 1 day + runs | Opus |
| **6. Remove v2's path** | v2 kept only as the contact planner; the switch deleted. | none (digests identical) | an hour | Sonnet |

**The client** (player-visuals §13 steps, and `-BbBatSim`):

- **Step 1 (built)**: the load ends on L from E2 instead of on the sim's first swing pose; the handover (`BatHandoverS`, up to 48 cm) goes away, and with it the 3 of 406 swings that ended it more than 2 cm off. `-BbBatSim` becomes the default once step 4 lands: the sim's bat is now the one to draw from L on (the reason it was off, Q44, is answered). §13.10's note that L "is not what §13.1 describes" is resolved: L is now [4]'s.
- **Steps 2–3** (legs, coil, hip turn): the body's keys are re-checked so the figure's clavicle passes through the sim's S at launch, −93 ms and contact (a new check: within 2 cm); the chest's correction toward the hands (§13.4) should become small, and a large one is a defect. The elbows at contact become a real test of the sim's hands (rear 81°, lead 56°).
- **Step 4** (follow-through): starts from the sim's bat at t_c + 0.03 s (D24) as planned.
- **Step 5** (early, late, checked): the checked swing stops the body at the `CheckSwingBrake` marker; the braked bat is now well back, so strip 9's picture ("bat stopped in front of him with the barrel behind the plate") becomes "bat stopped behind him, the barrel still up".

## 14. Open choices that are the owner's

Plain baseball terms; numbering is the Coordinator's.

**How the bat comes into the ball on a late swing.** *Now:* when a batter is late (the ball reaches the bat a little before he planned), the game's bat is still well short of full speed: 25 thousandths of a second before the planned moment its sweet spot is at about 63 mph against 72 at the planned moment, the way MLB's bat tracking shows late contact losing speed. *With the measured swing* the bat turns fastest 12–26 thousandths before contact, as in the lab studies, so on a late swing it is further back (6–9° at 25 thousandths), a little higher, and faster (64–71 mph): late balls are hit somewhat harder and pushed more to the opposite field, and about half of MLB's loss of speed on late contact is kept. Balls met on time or out front are exactly as now. *Options:* (a) the measured swing all the way to the ball, as specified: late contact changes as described, measured in the recalibration step against MLB's speed by contact depth; (b) keep today's bat for the last 30 thousandths: not possible as one smooth swing from the measured start (the bat would speed up, slow down and speed up again), so it means giving up the measured mid-swing position as well (the bat about 15° behind it 0.08 s before contact) and still a visible surge, or refitting today's last part of the swing to the measurements, which is several days' more work. *Recommended:* (a), with the check against MLB's late-contact speed; if it fails, the fix is the measured curve between −93 ms and contact from Fortenbaugh's graphs (step 0), not a speed setting.

**Where does the batter stand?** *Now:* the game's batter stands about 0.95 m off the middle of the plate, roughly even with it, and meets a middle pitch about 5 cm in front of where he stands. *The measurements:* at contact a hitter's hands are about 0.4 m in front of his chest, with the bat cocked about 65° to his arms. For the game's bat to have that relation to his arms, he would have to stand about half a metre further back in the box than the game's batter does (about as far from the plate), or meet the ball that much further out in front. MLB's bat tracking records where the ball is met relative to the batter and can settle which (step 0). It is not only a look: where his body is moves where his hands are in mid-swing, and so how the bat comes into the ball on a late swing (§5.3). *Options:* (a) once the data are in, move the batter (deeper in the box) and with him the point he watches from and the body the pitch can hit: hit-by-pitch, pitch recognition and late contact change a little and are re-measured; (b) meet the ball further out in front instead (if the data say so): the ball is met earlier and higher, which changes timing and contact and needs a fuller recalibration; (c) leave him where he is: the swing is measured everywhere except that at contact his arms and bat are nearly in line (the look today). *Recommended:* decide after step 0's numbers; (a) if they confirm the gap, done as its own step after the swing.

**How far a checked swing goes.** *Now:* about one held-up swing in thirty is a strike (touching the ball or the bat passing the front of the plate). *With the measured swing* the bat is still far back when the batter brakes, so nearly none will be. No source here says how often a real one is called a swing. *Options:* (a) accept and measure it; (b) let the batter brake later or more slowly (a later decision point than 60% of the swing, or a longer brake than 0.04 s) so some checked swings do go; (c) change what the umpire calls a swing: today it is the bat turned until it points straight across the plate; it could be the bat short of that by some angle, or the barrel's head crossing the front edge of the plate (closer to how umpires judge it in practice, though the rule book gives no definition). With the measured swing the hands start further back, so the same angle is a longer turn of the barrel than before. *Recommended:* (a) now; (b) or (c) only with a source (video of check swings and the calls on them) to set it.

**The measured swing is slower than the game's.** *Now:* the game's swing takes 0.150 s from start to contact for an average batter (MLB's bat sensors), and the hitter's decision clock is built on it. [4]'s hitters took 0.170 s from their weight shift to contact against 56 mph batting practice. *Options:* (a) squeeze the measured shape into each batter's own swing time (the average one 12% quicker): the decision clock and the plate-appearance rates stay; (b) keep the measured times and start the game's bat 0.02 s into [4]'s swing, already turning: the start no longer comes from rest and the client's load has to hand over to a moving bat. *Recommended:* (a).

## 15. Not established, and what would settle it

- **[4]'s curves between its stated instants** (§5.1): Appendix E's graphs (figures in the PDF). Settles the interior shape; until then the curves are the lowest-order ones through the stated values.
- **The trunk frame of [4]'s shoulder azimuth** (§4.2): Appendix B's diagrams. Reading 1 is the one [4]'s prose supports (the elbows move forward as the azimuths rise, §4.2); if Appendix B says otherwise, W_L and W_93 move and §5.3's and §6.1's numbers are redone. (The first version said the lead-arm check would catch a wrong reading; the review showed its tolerance could not, and it is gone.)
- **The body's position relative to the contact point** (§5.3): Statcast's batter-relative intercept and Savant's definition of the batter's position (step 0).
- **Swing length.** v3's sweet spot starts wrapped behind the head, so over its whole 0.150 s it travels further than v2's (probe: 8.1–9.1 ft from t_s against Statcast's 7.2–7.3 ft). Statcast's swing length runs from its own "start of swing", which must be read before comparing (v2 compared the last 150 ms). If v3 is long by Statcast's definition, it is a Tier M finding about the path's early part.
- **The hand speed at contact** (v2's q, unsourced): [3]'s uncocking (1,588°/s) and arm (1,160°/s) rates and [4]'s rear elbow extending at 752°/s at contact suggest 6–9 m/s against v2's 6.4; a check, not a constraint.
- **Live MLB timing**: [4]'s pitches were 56 mph. The swing's own 0.150–0.170 s agrees across sources; the scale s_T is an assumption (§4.3).
- **Real check swings**: no source here (§8.3).
- **The lead arm's reach at launch** (§6.1 residual: 12 cm long at 72° of lead elbow flexion): the proportions, the clavicle's position and the trunk frame; step 0 and the figure.
- **The arm-vector turning rate** fails its [3] bound in time or size whichever stance is used (§11): first look at [4]'s Appendix E curves.
- **What v3 does to games** has not been measured: nothing is built. §12's table is an expectation from the baseline, the construction and the probe's three aims.
