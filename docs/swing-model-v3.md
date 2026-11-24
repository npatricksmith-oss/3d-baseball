# The swing model, v3: the measured swing (proposed rewrite of agent-models §3.6)

**Status (2026-11-24): specified, not built; build step 0 (the data) and step 1 (the oracle and goldens: `analysis/oracle/swing_v3.py`, `golden/swing_v3.json`) done; the owner's four choices decided (PLAN D26, §14).** Step 1 revised §6.3 (the elevation fix) and with it the numbers of §6.5, §8.3, §11 and §12; §11.1 is what step 2 must reproduce. PLAN D25 (owner Q44 = b): the sim's swing is rebuilt whole from the measurements. This file is the rewrite of [agent-models §3.6](agent-models.md#36-swing-model) and of the lines of [physics-spec §6.3](physics-spec.md#63-bat-pose) and [rules-spec §4.5](rules-spec.md#45-check-swing) it touches (§6.6 and §8.4 below hold their proposed text). Until build step 4 (§13) switches the game to it, agent-models §3.6 (v2) describes what the code does; at that step this text replaces §3.6 there and this file is reduced to a pointer. Written by worker swing-spec on branch `work/swing-rebuild` from main `aa4a31d`; revised the same day for [the Sonnet review](reviews/2026-11-24-sonnet-swing-v3-spec-review.md), every changed number measured with `sim/target/codex-probes/v3_review_probe.py`, `v3_variants.py` and `v3_checks.py` (stdlib Python on the v2 oracle; outputs beside them).

**Sources.** [4] Fortenbaugh 2011 (33 AA hitters, live batting practice at 25 m/s, 300 Hz; the base) and [3] Welch et al. 1995 (7 professionals off a tee, 200 Hz) give every number of the swing's shape (D21). Statcast (MLB 2025 bat tracking, as fitted in [docs/calibration/2026-11-14-swing-model.md](calibration/2026-11-14-swing-model.md)) gives the contact instant, as v2 does. OpenBiomechanics is not used (D21). Every value read from [4] was checked against its text (`sim/target/research-dl/4/s.txt`, §4.1 and Appendix C's analysis code); the research notes are [docs/research/swing/](research/swing/swing-synthesis.md), and the diagnosis that led here is [launch-position.md](research/swing/launch-position.md). Today's swing at contact is measured in [docs/calibration/2026-11-24-swing-contact-baseline.md](calibration/2026-11-24-swing-contact-baseline.md) (the baseline, §2).

Tags as elsewhere: [S] sourced, [E] engineering value (named, configurable), [C] convention. "τ" is time from the nominal contact t_c (negative before). Times written "[4] −93 ms" are [4]'s own; §4.3 says how they map onto a batter's swing.

## 1. Summary

Today's swing (v2) is one rigid plane laid backward from the contact pose. It meets the ball the way MLB hitters do (Statcast) but before about 40 ms ahead of contact it is a different swing: the bat flat and pointing at the catcher where a measured hitter's is 40° up and wrapped behind his head, in line with the arms where his is cocked 120–130° behind them (diagnosis §3).

v3 keeps what v2 gets right and rebuilds the rest:

- **What moves.** The bat (the only physical body), held at the **hands** (mid-wrists, a point on the handle), whose path is built round a **shoulder point S** (the clavicle, the root of [4]'s arm vector) that follows the body. Three elements; the pelvis, trunk turn, legs and head stay presentation keyed to the sim's clocks (§3).
- **Where it starts.** A fixed **launch pose L** per batter, the same on every pitch, at rest: the bat 41° up and wrapped (azimuth −225°), cocked well behind the arms ([4]: 121°; 129° as built), the hands just in front of the back shoulder at its height ([4] at weight commitment, −170 ms) (§6.1).
- **How it meets the ball.** The **contact jet**, the pose, velocity and acceleration at t_c, is v2's plan exactly: the sweet spot at the aim point at `BatSpeed`, with Statcast's attack angle, attack direction, path tilt and vertical bat angle, and v2's reach rule (§6.2). After t_c the bat is v2's (§6.4). Before t_c the measured shape brings the bat into the contact window differently from today (§6.5).
- **Between.** From L at rest to [4]'s mid-swing anchor (−93 ms: azimuth 168° short of contact and turning at 56% of its contact rate, elevation 39°; the hands at a per-batter point built from [4]'s arm angles), then [4]'s own measured azimuth rate and elevation (Appendix E, scaled to the jet's contact rate and elevation) into the contact jet; the hands on one smooth curve throughout (§6.3). The lag through the swing is an outcome checked against [4] (121° → 133° max at −80 ms → 65° at contact).
- **Checked swings** brake by slowing the swing's own clock, so the bat stays on its path; the ruling measures the bat's turn from L (rules-spec §4.5's wording), so a wrapped bat is no longer "a swing" before it moves (§8).
- **Determinism and noise** are as v2: the same five execution draws in the same order on the same two streams; no new RNG purpose (§9).
- **What changes in games**: balls met before the planned instant (late swings; 23% of contact is 3 ms or more before it): 25 ms before t_c the bat is 4–7° further back, 11–14° higher and its sweet spot 7–9 mph faster than today's; 10 ms before, under 1° back, 2–3° higher and 3–4 mph faster; at and after t_c it is today's bat. Checked swings change a lot (the braked bat is far back: about 0.6 fewer strikes a game). The recalibration should be small to moderate (§12).
- **A finding about where the batter stands** (§5.3; decided D26, Q46 = a: he is not moved in this rebuild, a later step if a plate-relative contact depth confirms it): the measured arms at contact ([4]'s shoulder angles and its 65° bat-to-arm angle) put the hands about 0.4 m in front of the batter's chest, toward the pitcher. With today's contact depth and stance spot the sim's batter would have to stand about half a metre further back in the box than he does (at about the same distance from the plate), or meet the ball that much further out in front. Either the sim's batter stands in the wrong place or its contact point is too deep (its depth level, 0.35 m, is unsourced); Statcast's batter-relative intercept settles it. The probe backs the finding: [4]'s lag curve (falling steadily from 133° to about 60° at contact) appears only with the body 0.55–0.70 m deeper; with today's stance the lag bottoms out at 20–26° before contact and rises again. Where the body stands also moves the hands' anchors and so the late path, so it is a game question, not only a look.

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

**By construction v3 reproduces** every quantity at the planned contact instant (the jet is v2's) and every contact after it (§6.4: the bat is v2's there; 57% of contacts). **It moves** contacts before t_c, which meet a bat further back and faster than today's, by an amount that grows quickly with the timing error (§6.5: under 1° back and 2–3° higher at −10 ms, 4–7° back and 11–14° higher at −25 ms), and checked swings a lot (§12). It is not "v2 to second order" over the window: the first version of this spec said so, and the review measured otherwise.

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
- **Shoulder elevation θ and azimuth φ** (`PolarShoulder()`): [4]'s code takes the polar angles of the vector **from the elbow to the shoulder** joint centre in the upper trunk's frame, the same formula for both arms: θ = its angle from the trunk's long axis (so 0° = arm hanging, 90° = horizontal), φ = atan2(y, x) − 90° in the upper trunk's own frame (step 0, below). This spec never uses that vector or [4]'s axes directly. It places the **shoulder-to-elbow** unit vector d in its own trunk frame (x_T forward from the chest, y_T toward the lead side, t̂ up the trunk; §5.3):

  **d = −cos θ·t̂ + sin θ·(cos φ·x_T + sin φ·y_T)** (reading 1, **settled by step 0**),

  so φ = 0° is the upper arm pointing forward from the chest and negative φ turns it toward the batter's rear side (the rear arm out to the side and back; the lead arm across the chest). Reading 2, the only other one the code allows, reverses the horizontal part (φ + 180°) and is **ruled out**. *Step 0 (Appendix B, p. 185–186 of the PDF, printed 171–172; Appendix C `PolarShoulder()`, printed 185–188):* the code reads the shoulder and elbow joint centres `V_*_Shoulder_JC_Trunk` and `V_*_Elbow_JC_Trunk`, markers expressed in the upper trunk's own frame, takes shoulder − elbow, θ = acos(Z/|r|) (so Z is the trunk's long axis, up) and φ = atan2(Y, X) − 90°. Appendix B's diagrams draw the angles but label no axes. The axes follow from [4]'s data: the trail arm at launch (elevation 71°, azimuth −78°, elbow out to the side) puts elbow → shoulder along trunk +X with a small +Y, so +X is toward the lead side and, for a right-handed frame with Z up, +Y is **backward**; then azimuth 0° at contact (both arms, −1° and −9°) is the elbow → shoulder vector pointing straight back at the chest, the arms reaching forward, as they must. In the spec's terms this is exactly reading 1 (φ = 0° the upper arm forward, negative φ toward the rear side; forward coordinate of the elbows rising +0.07 → +0.19 m, §11 case 2). Residual doubt: the frame's handedness is inferred, not labelled; the contact check (arms forward, not back) would fail under the mirror image. **No spec number moves** (W_L, W_93, §5.3 unchanged). **The check that told them apart before was [4]'s own prose**: during swing initiation "the trail upper arm dropped and moved forward" while its azimuth rose −78° → −42°, and during acceleration the lead arm went "down and forward" as its azimuth rose −24° → −1°. Under reading 1 the rear elbow's forward coordinate (along x_T, from the rear shoulder) rises +0.07 → +0.19 m from launch to −93 ms and the lead elbow's +0.30 → +0.32 → +0.32 m; under reading 2 both are reversed (the elbows move back). The oracle asserts the sign (§11). Reading 2 would also put the launch hands only 0.15–0.28 m from the clavicle, against the swing-start hands near the back shoulder the sources describe.
- **Elbow and knee**: flexion, 0° straight.

### 4.3 The swing's clock and [4]'s times

The swing starts at **t_s = t_c − T_swing** (`QuickHands`, 0.150 s at grade 50; as v2) and maps onto [4]'s **weight commitment, −170 ms** ([4] §4.1.3–4.1.4: "swing initiation", when the bat "finally began to rotate forward"). [4]'s times are scaled into the batter's swing by

**s_T = T_swing / T₄**, T₄ = 0.170 s [E: [4]'s commitment-to-contact time],

so an event [4] puts at τ₄ comes at τ = s_T·τ₄ (−93 ms → −82 ms at grade 50), and [4]'s rates are used only as ratios to the contact rate (§5), never in absolute °/s. At grade 50 the sim's swing is 12% quicker than [4]'s AA batting practice; T_swing stays the decision clock it is now (agent-models §3.4), which keeps the commit time and so the plate-appearance rates where they are. The alternative (keep [4]'s times and start the sim's bat at [4]'s −150 ms, already turning at about 500°/s) was not taken (D26, Q48 = a).

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
| Bat azimuth rate at −93 ms | 0.558·Ȧ_c | [4]: 1,247°/s at −93 ms against 2,233°/s at contact | constraint (since step 1 met as Appendix E's 0.565, through the rate curve below) |
| Azimuth rate (as a ratio to the contact rate) and elevation, −93 ms to contact | §5.4's table | [4] Appendix E (digitised in step 0) | constraint since step 1 (§6.3): the rate's shape all the way, the elevation to −30 ms |
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
| Peak bat azimuth rate | 2,435°/s at −10 ms; 2,233°/s at contact | [4] §4.1.5–4.1.6 | reported (v3's peaks 9–22 ms before contact at 1.06–1.07 × its contact rate, §6.5) |
| Bat speed (barrel end) | peak 31 m/s at −15 ms, 29 m/s at contact | [3] "Ball Contact" | shape only: v3's tip speed peaks before contact at 1.0–1.15 × its contact value, within −30 to 0 ms. Absolute values are not comparable: the sim's sweet spot is set to MLB's 71.5 mph and [3]'s tip marker is filtered at 13 Hz |
| Bat tip's speed toward the plate and downward | peaks of 19 and 16 m/s near −40 ms | [3] | check of the times (−40 ± 15 ms scaled); magnitudes reported |
| Elbows at contact | rear 81 ± 8°, lead 56° | [4] Table 1 | client check (player-visuals §13.7) |
| Bat elevation, azimuth at contact | −30 ± 5°; −8 ± 12° | [4] Table 1 | check (the jet gives −29.5°, inside one sd, and +7.7°, 1.3 sd above [4]'s mean: within two sd) |
| Contact-instant values | sweet-spot speed, attack angle α, direction ψ, path tilt β over the last 40 ms, vertical bat angle, depth y_c, reach gain g | Statcast via agent-models §3.6 v2 | constraint, the jet (§6.2) |
| Swing length | 7.2–7.3 ft (the head of the bat, from an undefined start of tracking) | Statcast | check: head travel in the last 65 ms (§15) |

### 5.1 What the sources do not give, and what v3 assumes

Plainly, so no one mistakes an assumption for a measurement:

- **The curves between the stated instants.** [4] states azimuth, elevation and lag at −170, −93, −80 (lag only) and contact; its Appendix E graphs have the full curves, which step 0 digitised (§5.4). Since step 1, v3 uses two of them as data from −93 ms to contact (the azimuth rate as a ratio to its contact value, and the elevation, §6.3) and joins launch to −93 ms with the lowest-order smooth curves [E]. The first version joined the stated values with single polynomials; step 0 found its azimuth confirmed (within 5–8°) and its elevation 8–13° too early between −80 and −25 ms (§5.4), and step 1 replaced it. The shape is not neutral: it is what brings the bat into the contact window faster, further back and higher than today's (§6.5).
- **The hands' position at any instant.** Neither [3] nor [4] gives it. v3 builds it at −170 and −93 ms from [4]'s arm angles on the figure's segment lengths, choosing the forearm's roll by a least-misfit rule (§6.1), and takes it from the contact jet at t_c. Between, it is a smooth curve [E]. The arm angles, the lag and the lead arm's length do not fit exactly together on these proportions (§6.1's residuals).
- **The elevation at launch** (41° [E], between [4]'s 44° and 39°; the probe found 39° and 44° change nothing that matters).
- (Settled in step 0: the trunk frame of [4]'s shoulder azimuth, §4.2, reading 1.)
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

So either the sim's batter stands in the wrong place (the stance spot, 0.30 m from the plate's back tip toward the pitcher, is set by eye) or the sim's contact is too deep (contact depth y₀ = 0.35 m is [E]; only its slope across the plate came from Statcast). Statcast's bat tracking records the intercept relative to the batter (`intercept_ball_minus_batter_pos_x_inches`, `_y_inches`; `analysis/statcast/savant_bat.py`). **Step 0 re-pulled the six-date sample (11,185 competitive swings, `analysis/statcast/savant_intercept.py`):**

- Savant's definition (https://baseballsavant.mlb.com/csv-docs): "Distance, in inches, between the intercept point of the bat/ball and the batter's center of mass", in "the X (horizontal) direction" and "the Y (mound-to-plate) direction". It names no origin, sign or instant for the centre of mass and does not define "intercept point" further (the MLB glossary page was not readable, HTTP 406).
- **Y: the ball is met 30.5 in (0.775 m) in front of the batter's centre of mass** (sd 11.3 in; between batters' means sd 3.9 in; 29.8 in outside, 30.4 middle, 32.1 inside; right- and left-handers alike). **X: 36.9 in (0.94 m) to the side** (|x|; sd 6.9 in; the column carries no sign by handedness: right 37.2, left 36.6).
- The sim's batter: 0.95 m from the plate's middle line (P_x = −0.95) and the ball at the middle line, so **across, the sim agrees with Savant (0.95 m against 0.94 m)**. Along, the sim meets the middle pitch 0.05 m in front of the stance spot (y₀ 0.35 m, P_y 0.30 m), and about 0.12 m *behind* the pelvis of §5.3's contact knot (P_y + 0.17 m): **the sim's ball is 0.7–0.9 m nearer the body than Savant's.** That is the gap the lag and arms findings above needed (the body 0.55–0.70 m deeper), found independently in the tracking data: at contact the bat lies across the plate (azimuth near 0°), so the hands are at about the ball's depth, 0.7 m ahead of the centre of mass; [4]'s arms need them about 0.4 m ahead of the clavicle, and the clavicle sits 0.1–0.3 m ahead of the mass centre in the contact posture (15° of flexion, 31° of lean). The two agree in direction and within about 0.2 m, which the unstated frame of Savant's centre of mass could account for; the sim's present 0.05 m does not.
- **What the data cannot say:** whether the body is further back or the ball further out front *relative to the plate*. Savant's CSV gives the intercept only relative to the batter. [4]'s pelvis X (Appendix E, a plate-relative lateral position) reads −57 cm at −620 ms and −34 cm at contact (sd about 10 cm), with the reference point of "home plate" unstated, so it cannot be compared with Savant's 0.94 m across. Contact depth relative to the plate (a plate-relative bat-ball position, e.g. Hawk-Eye's) would settle it; nothing downloaded here has it.

The owner decided (D26, Q46 = a): the batter is not moved in this rebuild; he is moved deeper in the box as its own step after it (§13 step 5), and only once a plate-relative contact depth confirms that he stands deeper rather than meeting the ball further out front. Until then the lag-at-contact and arms-at-contact checks are expected failures with today's stance (§11).

### 5.4 Appendix E's curves (step 0)

[4] Appendix E (PDF pages 234–249, printed 220–235) draws 32 mean curves of the "typical" batter, ±1 sd bands, with the five force-plate events as vertical lines. Digitised with `analysis/research/fortenbaugh_appendix_e.py` (time axis from the event lines, value axis from the frame; method and errors in [the step 0 note](research/swing/9-step0-data.md)). **Legible: bat azimuth, elevation, lag, azimuth rate, both shoulders' elevation and azimuth, both elbows, pelvis X.** The curves reproduce [4]'s own text values (azimuth −173° at −93 ms against −175°, lag 132.7° at −80 ms against 133°, elevation 39.1° and −29.7°, rate 2,419°/s at −10 ms against 2,435°/s). Reading error: 1 px = 0.87° (azimuth), 0.29° (elevation), 0.31° (lag), 4.8°/s (rate), 1.35 ms; allow ±2° azimuth, ±1° elevation and lag, ±40°/s, and ±3 ms on steep parts. **Not in Appendix E:** the hands' position, the clavicle → wrist (arm) vector and its turning rate, the bat tip's speed (only [3] has them). Its only shoulder rate is the lead shoulder's azimuth rate.

[4]'s bat in its own time t₄ (middle fastballs; azimuth measured from its contact value, −9.7° on the graph, −7° in the text):

| t₄ (ms) | −170 | −130 | −110 | −93 | −80 | −70 | −60 | −50 | −40 | −30 | −25 | −20 | −15 | −10 | −5 | 0 |
| :-- | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: |
| azimuth − contact (°) | −215 | −199 | −183 | −163 | −147 | −132 | −117 | −102 | −86 | −65 | −57 | −47 | −34 | −22 | −7 | 0 |
| elevation (°) | 41.5 | 42.2 | 41.7 | 39.1 | 33.9 | 27.6 | 20.4 | 10.5 | 0.0 | −11.1 | −16.8 | −21.2 | −24.8 | −27.3 | −29.1 | −29.7 |
| azimuth rate (°/s) | 240 | 625 | 970 | 1,250 | 1,406 | 1,482 | 1,536 | 1,604 | 1,740 | 1,967 | 2,131 | 2,253 | 2,364 | 2,419 | 2,350 | 2,214 |
| lag (°) | 120.7 | 123.7 | 127.4 | 131.4 | 132.7 | 131.9 | 129.2 | 123.6 | 114.9 | 104.9 | 97.7 | 91.0 | 84.0 | 77.1 | 70.5 | 64.4 |

Arms (±2°): trail shoulder elevation 70.9° at −170 → 48.2° at −93 → 37.0° at contact (minimum 35.0° at −25 ms); trail shoulder azimuth −75° → −42° → −9°; lead shoulder elevation 87.9° → 80.4° → 69.7°, azimuth −30° → −23° → −1°; trail elbow 127.5° → 125.8° → 79.9° (falling fastest after −50 ms); lead elbow 72.2° → 68.2° → 55.9°. All agree with the stated values of §5.

**What it says about v3's interior** (`analysis/research/swing_v3_vs_appendix_e.py`, middle aim, v3 at τ = s_T·t₄, today's stance):

- **Azimuth: confirmed.** v3 is within 8° of [4] from −170 to contact (−4.6° at −93 ms by construction, +2° at −40, −2° at −25, +1° at −15; the 15° at launch is the 236°/s that §4.3 drops).
- **Azimuth rate: a second surge in [4] itself.** [4]'s rate climbs 1,250 → 1,536°/s from −93 to −60 ms (a plateau, 0.56 → 0.69 of the contact rate), then to 2,419°/s at −10 ms. v3's reaches the contact rate by −40 ms (1.00), peaks 1.08 at −25 to −20 ms. [4]: 0.79 at −40, 0.96 at −25, 1.02 at −20, 1.07 at −15, 1.09 at −10, 1.06 at −5, 1.00 at contact. So v3 is early by about 15 ms and slightly high (0.10 at −40, 0.12 at −25) in the middle of the swing.
- **Elevation: not confirmed.** v3's falls too early: at −70 / −60 / −40 / −25 ms v3 is 20° / 9° / −13° / −26° against [4]'s 28° / 20° / 0° / −17°, 8–13° low through the window. It leaves 45° at −110 ms (against 42°). The lag then falls early too (v3 85.7° at −60 ms, 38° at −40 against [4]'s 129° and 115°): part of that is elevation, part the body (§5.3).
- **Tried: adding the measured elevation as conditions** (20.4° at −60 ms and −11.1° at −30 ms, a degree-8 polynomial; probe in the note). The elevation then follows [4] to ±1° from −80 to −25 ms, but the sweet spot reaches 75.8–77.7 mph at −30 to −20 ms (middle) and 80.6–83.2 mph (inside-low), 6–11 mph above its contact speed, and for inside-low the elevation swings to 57–65° at −130 to −110 ms: it fails §11's "never above 1.1 times" bound. A shape from [4]'s elevation needs the hands' path (and the azimuth's rate) refit with it, not one more condition. This is the first thing for step 1 (the oracle) to settle.

**Step 1's answer** (§6.3, `analysis/oracle/swing_v3.py`): from −93 ms on, v3 uses [4]'s rate curve (as a ratio to the contact rate) and its elevation curve (scaled to the aim's contact elevation) as data, joined to the jet by short corrections, and a quintic from launch to −93 ms. The elevation then follows [4] within 0.8° from launch to −30 ms (middle aim), the azimuth stays within 4.2° of [4]'s graph from −93 ms on, and the sweet spot peaks at 1.03–1.05 × its contact speed 15 ms before contact. The costs (late contact 11–14° higher and 7–9 mph faster at −25 ms than today's; the path tilt over the last 40 ms 1–9° steeper than β) are in §6.3 and §6.5.

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

*Revised in build step 1 (2026-11-24): the elevation fix that step 0 asked for (§5.4). The oracle (`analysis/oracle/swing_v3.py`) implements this text; the first version (one polynomial per curve through the anchors) is kept there as `path='spec'` for comparison only.*

On τ ∈ [−T_swing, 0], written in u = τ/T_swing ∈ [−1, 0]. Since T_swing = s_T·T₄, [4]'s own time is t₄ = 170·u ms on every swing, so the joints sit at fixed fractions: u_93 = −93/170 and u_w = −30/170.

**[4]'s measured curves are data** [S]: two rows of §5.4's Appendix E table, each the natural cubic spline through its 16 points from −170 to 0 ms (second derivative zero at both ends): the azimuth rate over its contact value, r(t₄) = rate(t₄)/2,214°/s, with R(t₄) = ∫₀^t₄ r (the spline integrated exactly, in ms), and the elevation e(t₄) in degrees.

| Curve | From −93 ms of [4]'s time to contact (u_93 ≤ u ≤ 0) | Launch to −93 ms (−1 ≤ u ≤ u_93) |
| :-- | :-- | :-- |
| A(u) | A_c + Ȧ_c·s_T·R(t₄)·10⁻³ s/ms + δ_A(u): [4]'s rate shape scaled to the jet's contact rate. δ_A is the quintic with δ_A(u_93) = (A_c − 168°) minus the rest, zero rate and acceleration at u_93, and at 0 zero, zero and the jet's Ä_c less the shape's | the quintic from A_L at rest (Ȧ = Ä = 0) to the right-hand curve's value, rate and acceleration at u_93 |
| E(u) | 39° + κ·(e(t₄) − e(−93)) + δ_E(u), κ = (E_c − 39°)/(e(0) − e(−93)): [4]'s fall from −93 ms scaled to the aim's contact elevation. δ_E = 0 up to u_w = −30 ms [E]; from there the quintic that is zero to second order at u_w and adds the jet's Ė_c and Ë_c less the shape's at 0 | the quintic from E_L at rest to the right-hand curve's value, rate and acceleration at u_93 |
| W(u), each of x, y, z | the sextic on [−1, 0]: W_L at rest (3 conditions), W_93 at u_93 (per batter, §6.1), the jet's W_c, Ẇ_c, Ẅ_c at 0 (unchanged from the first version) | (the same curve) |

- **Closed form.** Each curve is a fixed linear combination of its conditions: the four basis matrices (the launch quintic on [−1, u_93], δ_A on [u_93, 0], δ_E on [u_w, 0], the hands' sextic) and the splines' moments and integrals at the knots are constants, exported in `golden/swing_v3.json` (`meta.basis`, `meta.appendix_e`) for the Rust to rederive in a test. Per swing: κ, the residuals at u_93 and 0, and dot products. No per-swing linear solve. The pieces join with value, rate and acceleration equal at u_93 and u_w, and at 0 with v2's swing (§6.4); the oracle tests all three.
- **Pose at τ:** â = (cos E cos A, cos E sin A, sin E) (mirrored for a left-hander), W from its curve, knob K = W − u_g·â. **Velocity:** â̇ from Ȧ, Ė; ω = â × â̇ (the bat has no roll about its own axis, as in v2); the centre of mass moves at Ẇ + ω × ((u_cm − u_g)·â). All transcendental functions through `bb_math::m`.
- **Why these conditions.** At rest at the start, so the client's load ends on L without a jump; [4]'s measured curves between −93 ms and contact, where step 0 read them; the jet at contact, so the planned contact is v2's. Every rate enters as a ratio to the jet's contact rate, so a faster swing is faster all through. The 168° still to turn at −93 ms is [4]'s text value (its graph's rate integrates to 166°, its azimuth graph reads 163°); with v2's contact rate over the scaled time [4]'s rate shape covers only 155° of it (the jet turns at 2,349°/s, [4]'s 2,214°/s over 0.88 of the time), so δ_A adds about 13° between −93 ms and contact. The elevation keeps [4]'s fall to −30 ms and then turns to the jet, whose bat is already rising at contact (Ė_c = +331°/s at the middle aim against [4]'s −120°/s: MLB's attack angle of about 8° against a lab batter's level swing at 56 mph batting practice); the window's start, −30 ms [E], is a trade: a later start follows [4] further (to within 1.0° at −10 ms) and moves the path tilt a little nearer β, but squeezes the turn into fewer milliseconds, and the correction's angular acceleration grows from 53,000–86,000°/s² (start −30 ms; the jet's own Ë_c is 32,000–54,000°/s²) to 81,000–133,000 (−20 ms) and 169,000–273,000°/s² (−10 ms); the sweet spot's speed hardly changes (1.03–1.10 × over the zone for any start from −40 to −10 ms).
- **What it follows** (middle aim, [4]'s Appendix E against v3 at τ = s_T·t₄; oracle `appendix_e_compare`): the elevation within 0.8° of [4] from launch to −30 ms (41.0 / 41.7 / 41.6 / 39.0 / 33.7 / 27.3 / 20.0 / 10.0 / −0.6 / −11.9° at −170 … −30 ms against 41.5 / 42.2 / 41.7 / 39.1 / 33.9 / 27.6 / 20.4 / 10.5 / 0.0 / −11.1°), then 1.0–2.6° under it to contact (the jet); the azimuth within 4.2° of [4]'s own azimuth graph from −93 ms on (which the construction does not use) and 4–15° behind it before −110 ms (the dropped 236°/s, §4.3); the rate 0.10–0.16 of the contact rate above [4]'s from −70 to −40 ms (the 13°), within 0.04 from −25 ms on, peaking at 1.05 at −10 ms ([4]: 1.09 at −10 ms). The first version was 8–13° low in elevation from −80 to −25 ms and rose to 45–46° before −93 ms.
- **What it costs** (the three golden aims, today's stance):
  - *Late contact moves more than the first version did.* 25 ms before t_c the bat is 11–14° higher than today's (first version 2–3°), 4–7° further back (6–9°) and its sweet spot 7–9 mph faster (63 → 70–72 mph; first version 2–7); 10 ms before, 2–3° higher, under 1° back, 3–4 mph faster. §6.5 has the table.
  - *The path tilt over the last 40 ms is not the jet's β:* 37.0° / 46.7° / 27.0° against 32.0° / 37.9° / 25.8° (middle, inside-low, outside-high; the first version 33.2° / 45.4° / 22.0°). Two causes: the hands' sextic leaves the jet's plane by 3–11 cm in the last 40 ms (the first version did the same), and the bat, following [4]'s elevation, is up to 12–13° off the plane at −40 ms. The two sources disagree here: [4]'s bat over its last 40 ms lies in a plane tilted 30°, close to Statcast's tilt, but level 86° of azimuth before contact, the jet's level 108° before it (the attack angle again). Tried and not kept: bringing the hands into the jet's plane between −93 and −40 ms (error over the zone −5.5° to +6.3°, from −2.6° to +13.6°: halved, not removed; one more mechanism), and laying the bat in the jet's plane over the last 40 ms (the elevation then 12–15° under [4]'s at −40 to −30 ms, and up to 1.44 × the contact speed when the blend is short). An expected failure in §11, a Tier M finding for step 4.
  - *Speed is sane:* the sweet spot peaks at 1.03–1.05 × its contact value 15 ms before contact, the tip at 1.05–1.07 × ([3]: 1.07 at −15 ms).
- **Tried for the elevation and not kept** (step 1, scratch probes): two measured points added as polynomial conditions (step 0's: the sweet spot at 1.09–1.23 × its contact speed, inside-low elevation up to 65–73° before −93 ms); [4]'s elevation plus one degree-6 correction over the whole swing (2.5–3.4° low from −60 to −30 ms, the correction spread everywhere); [4]'s azimuth graph as the shape (its azimuth near contact disagrees with its own rate graph, and the correction swings the bat backwards); the rate shape scaled to the 168° instead of the contact rate, or not anchored at −93 ms (the launch segment's rate 2–3.7 × its −93 ms value over the attribute range: two surges, and backwards for fast swings).
- **Why not join today's path before the window** (the review's other remedy): the measured mid-swing (168° of azimuth still to go 82 ms before contact, turning at 56% of the contact rate) cannot be joined to v2's state 30 ms before contact (only 63° to go, turning at 1,616°/s) without the bat speeding up, slowing to 1,450–1,600°/s, and speeding up again: two surges in one swing (probe: rate peaks at −72 ms and again at contact). Dropping the −93 ms anchor does not cure it (the wrapped launch is 110° behind v2's own start, so the deficit must be made up before the window). A condition on the third derivative at contact does not help either (the difference at −25 ms stays 7–9.5°). So v3 keeps the measured shape up to the planned instant, and §6.5 states the difference; the owner chose this (D26, Q45 = a, §14).
- **What the oracle verified** (§11 case 8; 372 swings: aims 0.45 m either side of the plate's centre and 0.15 m below to 0.15 m above the zone, the planned attack angle ±4°, T_swing 0.130 / 0.150 / 0.174 s, effort ±10%, left-handers, and the jet turned to contact azimuths −25° to +50° as a stress test): every path defined and continuous; A non-decreasing on every planned swing (the planner's contact azimuths span −8.5° to +23.5°; one stressed jet reverses at 5°/s); E within −44.4° to +42.3°; the bat clear of the trunk and the head (the trunk a 0.10 m capsule from the mid-hips to the clavicle, the head a 0.10 m sphere 0.13·H above the clavicle [E]: closest 6 mm and 1 mm). Two misses, reported: (1) **the sweet spot above 1.1 × its contact speed** at 3 of the 105 grade-50 swings (low and outside, at most 1.12), on slow swings (effort −10%: 1.10–1.18) and on quick ones (T_swing 0.130 s: 1.11–1.36), because the 168° still to turn at −93 ms is fixed while the contact rate follows `BatSpeed` and effort and the time follows s_T. Today every batter's T_swing is 0.150 s and `BatSpeed` (σ 3.9%) with effort (σ 4.5%) puts a two-sd slow swing near 1.13 at the middle aim; when `QuickHands` starts to vary T_swing this needs a decision (§14, Q48). The first version did the same (T 0.130: 1.10–1.35). (2) **The hands beyond the arms' full reach** (0.821 m from the clavicle at H 1.85 m) by up to 5.6 cm, 20 ms before contact, on pitches at the knees and outside swung with 4° more uppercut: the sextic bulges outward on its way to a long reach. The client's reach solver covers 24 cm.

### 6.4 After contact

For τ ∈ [0, +0.03 s] (D24) the bat follows v2's analytic swing (the bumps continued past t_c), which joins the curves of §6.3 with matching position, velocity and acceleration at τ = 0. Contacts found after t_c (57% of contacts today, all within +25 ms) therefore meet exactly v2's bat; the after-contact motion is otherwise cosmetic (physics-spec §6.3) and the follow-through is the client's.

### 6.5 Where v3 differs from v2 inside the contact window

At τ = 0 the curves and v2 share position, velocity and acceleration; after it the bat is v2's (§6.4). Before it they differ, and not by a small third-order term, as the first version of this section estimated (0.1° at −10 ms, 2° at −25 ms). Measured by the oracle (`v2_difference`, `golden/swing_v3.json` `report`; the three golden aims, today's stance), with the path of §6.3 as revised in step 1; the first version's numbers in brackets:

| aim | v3 − v2 azimuth at −10 / −25 ms | v3 − v2 elevation at −10 / −25 ms | sweet-spot speed v3 / v2 at −25, −15, −10, −5 ms (mph; 71.6 at contact) | v3's azimuth rate peak |
| :-- | :-- | :-- | :-- | :-- |
| middle | −0.8° / −5.6° [−0.7 / −7.6] | +2.6° / +12.7° [− / +2.7] | 70.3 / 62.8, 73.8 / 68.0, 73.6 / 69.9, 71.9 / 71.1 [66.9, 69.9, 70.8 at −25, −15, −10] | 2,473°/s at −10 ms (contact 2,349) [2,546 at −20] |
| inside-low | −0.6° / −4.2° [−0.4 / −5.8] | +3.1° / +14.4° [− / +1.9] | 71.5 / 62.2, 75.2 / 67.6, 73.7 / 69.6, 71.2 / 70.9 [64.3, 67.4, 69.1] | 2,692°/s at −9 ms (contact 2,527) [2,679 at −12] |
| outside-high | −0.9° / −7.0° [−0.9 / −9.2] | +2.1° / +11.2° [− / +3.3] | 70.8 / 63.5, 73.5 / 68.5, 73.8 / 70.1, 72.5 / 71.2 [70.5, 72.5, 72.4] | 2,309°/s at −22 ms (contact 2,172) [2,535 at −26] |

The reason is the measured shape itself: [4]'s bat still has 168° to turn 82 ms before contact, and v2's has 111°, so v3's bat must turn faster than today's in the last 80 ms, and its azimuth rate peaks 9–22 ms before contact, as [4]'s does (−10 ms). Since step 1 it also keeps [4]'s elevation to −30 ms (of [4]'s time, −26 ms here), where v2's bat, laid in the jet's plane, is already 11–14° lower. **In v3 the lab's peak is built in**, as the owner chose (D26, Q45 = a).

**What it does to contact.** Contacts after t_c (57%, the early swings) are unchanged. Contacts before it (late swings) meet a bat further back, higher and faster: from the baseline's timing table, 1,003 contacts in 600 games (2.3%) come 10–26 ms before t_c, where the bat is 1–7° further back, 2–14° higher and 3–9 mph faster; 8,885 (20%) come 3–10 ms before, where it is under 1° back, 0.5–3° higher and 0.3–4 mph faster. Expected: late balls hit harder, pushed more to the opposite field and met with a flatter or rising bat (launch angle of late contact moves), some swings at the window's early edge turning misses into fouls or the reverse; the totals probably within a point or two (to be measured, §12). Against Statcast's speed by contact depth (bat speed about 3.4 mph under the batter's mean for balls met 12 in or more deep, about −15 ms), v2 gives −3.6 mph at −15 ms and v3 **+2.2** (middle aim; the first version −1.7): **v3 keeps none of MLB's loss of speed on late contact**, as the lab's bat (1.07 × its contact speed at −15 ms, [3]) does not. That is the Tier M check (calibration-targets §4.3) for build step 4 (D26: checked there, not forced now); if it fails, the fix is the interior shape, not a speed knob.

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

The brake comes at −82 ± 32 ms (baseline), and never after t_c − 10 ms (the abort point, 60 ms before contact, plus the 50 ms latency). At −82 ms v3's bat is about 168° short of its contact azimuth (pointing away from the plate and up), and 20 ms more of path brings it to −124° to −139° (oracle, three golden aims): well back. A brake at −40 ms stops it at −41° to −47°, at −25 ms at −12° to −1°; only brakes in the last 15 ms before the cut-off (the baseline's p95 is −29 ms) carry it past 0° (at −12 ms: +16° to +31°, a swing on all three aims). A braked v3 bat almost never touches the ball and almost never passes 0°. Today 3.4% of checked swings are strikes (310 touches, 36 on the yaw), about 0.6 a game; v3 should make nearly all of them balls. No source here measures how often a real check swing is ruled a swing; this is reported and counted (D26, Q47 = a: no change to the brake or the ruling without a source).

### 8.4 rules-spec §4.5, as proposed

Replace the first sentence with: *"A batter has **swung** if the bat's horizontal direction, turned from its pose at the start of the swing in the swing's own direction, reaches the point where the bat is perpendicular to the pitcher–home line **with its barrel toward the plate** (the barrel head level with the front of the batter) [C]. The swing model reports the furthest azimuth reached, A_max (agent-models §3.6: unwrapped from the launch pose, 0° = across the plate), and the batter swung if A_max ≥ θ_swing = 0° [C, configurable]. A bat that starts wrapped behind the head passes the other perpendicular (pointing away from the plate, −180°) early in every swing; that one does not count."* The rest of §4.5 (any contact is a swing) stands.

In code: `Swing::judged_swing` becomes A(σ(t_b + B)) ≥ 0 on the unwrapped azimuth, and `max_angle` reports A_max; today's `a.y ≥ 0` test is only right for a bat that starts pointing at the catcher, and would call a wrapped bat a swing before it moves (diagnosis §2).

## 9. Determinism

- Every function is closed form in τ: polynomials, the natural cubic splines of [4]'s two curves and the exact integral of one (piecewise polynomials with constant knots and moments, §6.3), `m::sin`/`m::cos` for â, one `m::atan2`/`m::asin` for A and E of the jet. W_L and W_93 are built from the constants χ_L, χ_93 (§6.1) by closed-form vector arithmetic; nothing is solved at run time. No `mul_add`, no iteration that depends on a tolerance reached.
- No new RNG purpose. The draw order of `SwingExecution` (δ_v, α, δ_r, t₀) and `SwingEffort` is unchanged; nothing new is drawn.
- The basis matrices of §6.3 and the splines' moments are constants with a test that rederives them from their conditions and [4]'s table (the oracle exports both, `golden/swing_v3.json` `meta`).
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

*Built in step 1 (2026-11-24).* The oracle is `analysis/oracle/swing_v3.py` (stdlib only, beside v2's `swing_oracle.py`, which it imports for the contact jet); it computes everything below **from this spec**, independently of the Rust. `python3 analysis/oracle/swing_v3.py` writes `analysis/oracle/golden/swing_v3.json`, which `sim/crates/bb_agents/tests/` will read in step 2. Its checks are `analysis/oracle/test_swing_v3.py`, run with the other oracle tests by `python3 -m unittest analysis/oracle/test_oracle.py`. v2's oracle and golden stay as the contact planner's (the jet is v2's), so the jet is tested against v2's existing fixture. `analysis/oracle/mutations_swing_v3.py` reruns the checks under deliberate mistakes (below).

**Golden cases** (a right-hander, H = 1.85 m, grade 50: T_swing 0.150 s, `BatSpeed` 32.0 m/s, effort 0; the 34-in bat; and a left-hander mirrored). Each is computed by the oracle from this spec and compared with the Rust; the conditions the construction imposes are not themselves tests (AGENTS.md: no tautological tests), so each case below names what it can catch.

1. **A hand-worked case** (catches frames, signs, the reading of §4.2 and the χ convention; worked on paper, not by the oracle): S = (0, 0, 0); an upright trunk facing +x (ψ = φ_f = φ_l = 0, so t̂ = ẑ, x_T = +x, y_T = +y, r̂′ = −y); H = 1.85 m (l_u = 0.3441, l_f = 0.2701, half-shoulder 0.2072 m). Rear shoulder (0, −0.2072, 0). Rear arm θ = 90°, φ = −90°: d = −y, elbow (0, −0.5513, 0). Forearm κ = 90°: p̂ = ẑ, q̂ = d × p̂ = −x, and χ = 270° gives the forearm +x, so **W = (0.2701, −0.5513, 0)**, |W − S| = 0.6139 m, **hands at shoulder height**. With the bat at A = −90°, E = 45° (â = (0, −0.7071, 0.7071)): **λ = acos(0.5513 × 0.7071 / 0.6139) = 50.58°**. Under reading 2 the same inputs give the elbow at (0, +0.1369, 0), W = (−0.2701, 0.1369, 0) and λ = 108.6°, so the case tells the readings apart. The oracle agrees to 1e-12 (`hand_case`).
2. **The reading check** (§4.2, [4]'s prose): from L to 93 the rear elbow's forward coordinate (along x_T, from the rear shoulder) rises +0.068 → +0.193 m, and the lead elbow's +0.312 → +0.323 m from 93 to c; at contact both elbows are forward of the chest. All reversed under reading 2 (`reading_check`).
3. **The jet** at t_c for v2's golden cases `rhb_middle`, `rhb_inside_low`, `rhb_outside_high` and `lhb_middle_quick` (`golden/swing.json`): v3's knob and axis at and after t_c equal v2's fixture to 1e-9 and the sweet spot's velocity to 1e-5 (the fixture's own finite difference); the jet's analytic first and second derivatives equal five-point differences of v2's positions (`jets`).
4. **Path samples** every 5 ms from t_s to t_c + 0.03 for the three aims, the left-hander and the middle aim at T_swing 0.130 and 0.174 s: knob, axis, Ω, the centre of mass's velocity, the hands and S (`paths`). Rust against the oracle within §11.1's tolerances (two implementations of the same spec: catches implementation errors, not spec errors).
5. **Continuity and rest**: value, rate and acceleration equal from both sides at u_93, u_w and τ = 0 (the last against v2's swing); all rates zero at t_s; the reported velocity is the derivative of the position (Ω and V); the left-hander is the mirror (Ω a pseudovector); the launch pose is the same for every aim and effort.
6. **Braking** (`brakes`): brakes starting at −120, −82, −40, −25 and −12 ms: the stop pose (the path's pose at t_b + B/2), the stopped azimuth and the ruling, which agrees with the bat's direction at the stop (toward the plate and the pitcher). Middle / inside-low / outside-high: −82 ms stops at −132° / −124° / −139°, −40 ms at −44° / −41° / −47°, −25 ms at −7° / −1° / −12° (no swing); −12 ms at +23° / +31° / +16° (a swing).
7. **T_swing** 0.130 and 0.174 s: the elevation follows Appendix E at the scaled instants (within 1°) and the azimuth within 10° of its graph; the speed bound fails at 0.130 s (case 8).
8. **Over the whole aim range** (`aim_range`; §6.3's last item has the result): 324 planned swings (aims 0.45 m either side of the plate's centre and from 0.15 m below to 0.15 m above the zone, the planned attack angle ±4°, T_swing 0.130 / 0.150 / 0.174 s, effort ±10%, left-handers) and 48 stressed ones (the jet turned to contact azimuths −25° to +50°). The lag at 93 runs from 156° (contact azimuth −25°) to 100° (+50°).

**Bounds from the sources** (outcomes the construction does not impose; tolerances [E], wide enough for a model not fitted to them). Oracle values for the three golden aims (middle / inside-low / outside-high), today's stance, and the stance 0.55 m deeper (P_y = −0.25 m) for comparison; "expected failure" is an `@expectedFailure` test whose docstring carries the number, which turns into a failed run when it starts to pass:

| Check | Bound | Source | Today's stance | 0.55 m deeper |
| :-- | :-- | :-- | :-- | :-- |
| Lag maximum | 133 ± 8° at −71 ± 20 ms (−80 ms scaled) | [4] §4.1.5 | 135° / 130° / 140° at −89 to −90 ms: passes (narrowly on time) | 134° / 129° / 139° at −76 to −78 ms (inside-low: at launch) |
| Peak uncocking rate \|λ̇\| | ±25% of 1,588 × Ȧ_c / 2,233°/s, at −18 ± 15 ms scaled | [3] Table 2, scaled by [4]'s contact rate (§5) | 2,513 / 2,124 / 2,777°/s at −42 to −49 ms: **expected failure** (size at middle and outside-high, time at all three) | 1,518 / 1,455 / 1,581°/s at −15 to −23 ms: passes |
| Peak arm-vector turning rate (horizontal, about S) | ±25% of 1,160 × Ȧ_c / 2,233°/s, at −57 ± 20 ms scaled | [3] Table 2 | 1,257 / 1,253 / 1,265°/s (inside the size bound) at −102 to −106 ms: **expected failure** (time) | 1,880 / 2,698 / 1,434°/s at −38 to −46 ms (size fails) |
| Bat tip speed shape | peak before contact, 1.0–1.15 × the contact value | [3] (1.07 at −15 ms) | 1.050 / 1.070 / 1.047 at −15 ms: passes | 1.14–1.16 at −23 ms (at the edge) |
| Tip toward the plate and downward | peaks at −35 ± 15 ms scaled | [3] (−40 ms) | toward the plate −37 to −38 ms; downward −26 / −45 / −26 ms: passes | the same |
| Lag at contact, by location | middle 65 ± 15°; HI 79, HO 63, LI 58, LO 51, each ± 15° | [4] Table 1 | 51 / 72 / 30 / 62 / 39° (middle, HI, HO, LI, LO): **expected failure** at HO only | 53 / 71 / 53 / 54 / 41°: passes |
| Hands ahead of the clavicle at contact | ≥ 0.25 m along x_T | [4] arms at contact (§5) | −0.08 / −0.08 / −0.06 m: **expected failure** | +0.44 / +0.43 / +0.46 m |
| Plane tilt over the last 40 ms | within 1.5° of β | Statcast (the jet) | +5.0 / +8.8 / +1.2° (37.0 / 46.7 / 27.0° against 32.0 / 37.9 / 25.8°): **expected failure** (§6.3) | +4.5 / +7.7 / +0.9° |
| Swing length | head travel in the last 65 ms within 7.3 ± 0.7 ft (Statcast's mean; start undefined, §15) | Statcast | 7.34 / 7.37 / 7.41 ft: passes | 7.42 / 7.56 / 7.40 ft |
| Knob (hands) speed at contact | 6–9 m/s (§15: [3]'s rates, [4]'s elbow) | [3], [4] | 6.52 / 5.65 / 7.45 m/s: **expected failure** at inside-low (the jet is v2's, unchanged) | the same |
| Elevation against Appendix E (step 1) | within 1° from launch to −30 ms, middle aim, T_swing 0.130 / 0.150 / 0.174 s | [4] Appendix E (digitising error, §5.4) | within 0.8°: passes | — |
| Azimuth against Appendix E's own graph | within 8° from −93 ms to contact (10° at 0.130 / 0.174 s) | [4] Appendix E | within 4.2°: passes | — |
| Sweet-spot speed before contact | ≤ 1.10 × its contact value over the whole range | §6.3 [E] | in the zone 1.01–1.07: passes; whole range up to 1.12 (grade 50), 1.18 (effort −10%), 1.36 (T 0.130 s): **expected failure** (§6.3) | — |

The lag, uncocking and hands-ahead failures with today's stance are §5.3's finding, not defects of the construction: with the batter 0.55 m deeper they pass (Q46: a later step). The arm-vector rate fails either way: Appendix E has no arm-vector (clavicle → wrist) curve (step 0, §5.4), so [4] cannot say whether the bound or the model is off; the bound stays [3]'s alone, its failure a finding to read with that in mind. The plane tilt and the speed bound are §6.3's costs.

**Mutations tried** (`mutations_swing_v3.py`; each must fail tests other than the golden-file check): the shoulder azimuth's reading 2 (fails the hand case, the elbows' forward motion, the lag maximum, swing length, and the aim range's body clearance); the trunk frame's x_T and y_T swapped (25 tests); the azimuth's sign (the hand case, the jet against v2's fixture, the ruling against the bat's direction, velocity = derivative, the launch hands, the tip speed); the anchor at −80 ms instead of −93 (Appendix E elevation and azimuth, the rate shape, the lag maximum, swing length, tip speed); the −93 ms anchor not scaled by s_T (the azimuth against Appendix E, continuity, the lag maximum, and the speed bound's expected failure passing); s_T = T_swing/0.150 or s_T = 1 (Appendix E elevation, the lag maximum, swing length, tip speed or continuity, and the speed bound passing); Ω mirrored as a polar vector (the jet against v2's left-hander, the mirror, velocity = derivative).

### 11.1 What step 2's Rust must reproduce

From `golden/swing_v3.json`; tolerances are for two implementations of one closed-form spec in double precision (the oracle integrates v2's angles after contact by adaptive Simpson to 1e-13 and differentiates v2's jet analytically).

| Quantity (fixture key) | Tolerance | Note |
| :-- | :-- | :-- |
| Constants: χ_L, χ_93 (`meta.chi_L_deg` 308.271°, `chi_93_deg` 309.426°), A_ref (`A_ref_deg` 4.797°) | 1e-6° | the Rust stores χ as constants with a test that the misfit's minimum is there (its derivative zero, lower than the other minima listed in `hands.*.other_minima`) |
| Basis matrices (`meta.basis`: launch quintic, δ_A, δ_E, hands' sextic), spline moments and the rate integral at the knots (`meta.appendix_e`) | 1e-12 relative | rederived in a test from their conditions and [4]'s table |
| Body at the knots (`body`: O, S, t̂, x_T, y_T, both shoulders) and the hands' anchors (`hands`: W, elbow, knob, axis, lag, lead-arm distances) | 1e-12 m, 1e-9° | closed form |
| The hand-worked case (`hand_case`) | 1e-12 m, 1e-9° | |
| The jet (`jets`: hands and axis with first and second derivatives, unwrapped A and E with theirs) | 1e-9 (pose), 1e-7 relative (derivatives) | the Rust has v2's analytic rates; the oracle's are analytic too |
| Path samples (`paths`: knob, axis, Ω, V_cm, hands, S every 5 ms, t_s to t_c + 0.03) | 1e-9 m, 1e-9 rad, 1e-7 m/s and rad/s | after t_c the oracle integrates v2 numerically (1e-13) |
| Brakes (`brakes`: stop azimuth, ruling, stop pose) | 1e-9°, exact ruling | |
| Aim range (`aim_range.ranges`, `failures`) | the same failures, ranges to 1e-6 | a smoke test over the same 372 cases, optional |

Spec changes step 1 made to be implementable (each in this file): §6.3 rewritten (the elevation fix: [4]'s curves as data from −93 ms, the joints at u_93 and u_w = −30/170, the quintic and sextic bases); the misfit's A_ref is the middle-middle aim's contact azimuth, 4.797°, which v2's planner gives for every height (checked at 1.70, 1.85 and 2.00 m), so χ_93 is one constant, as §6.1 step 4 says; the body-clearance check's sizes ([E], case 8); the swing length check uses the bat's head (Savant's definition) over a fixed 65 ms.

**Property tests** (Rust): |â| = 1; the velocity the pose reports equals the finite difference of positions (as v2's test); A non-decreasing on every unbraked swing; a braked swing never moves after t_b + B; mirror symmetry for a left-hander; determinism (the same plan twice gives identical bits).

**Game-level tests**: a test that every swing's contact quantities at the planned instant (sweet-spot speed, α, ψ, bat elevation) equal v2's for the same plan, and that the bat after t_c is v2's; the `bb_game` crate-graph test unchanged.

## 12. The recalibration plan

**Expected to move, and why:**

| Measure | Expected | Why |
| :-- | :-- | :-- |
| Checked swings ruled swings | 3.4% → near 0 | the braked bat is far back (§8.3) |
| K% | −0.1 to −0.3 points | about 0.6 strikes a game become balls |
| BB%, pitches per PA | +0.1 to +0.2 points; up slightly | the same pitches |
| EV, LA, spray, foul share of late contact (3 or more ms before t_c, 23% of contact) | late balls harder (+1 to +4 mph at −10 ms, +7 to +9 at −25), pushed, and met by a bat 2–3° (−10 ms) to 11–14° (−25 ms) higher, so their launch angle moves; foul share there moves | the measured shape brings the bat in faster, further back and higher (§6.5) |
| EV, LA, spray of all contact; foul share | probably within a point; measure | early contact (57%) is unchanged; late contact is the minority |
| Contact%, whiff% | small changes at the window's early edge | the sweet spot is up to about 15 cm elsewhere at −25 ms (mostly higher) |
| Bat speed by contact depth (Tier M) | deep contact gains about 2 mph where MLB's loses 3.4: expected to miss the band | §6.5; the fix, if wanted, is the interior shape, not a knob (D26, Q45) |
| HR/PA, runs | within noise, follow K% and BB% | |
| HBP% and everything perceptual | unchanged (the batter is not moved in this rebuild, D26, Q46) | the eye point and body capsules follow the stance spot |

**Match by construction** (§2): every quantity at the planned instant, and every contact after it.

**Bands** (calibration-targets): §4.3 (bat speed 71.5–72 mph, swing length about 7.3 ft, attack angle, commit-to-contact 0.15 s), §4.4 (EV 88.5, LA 12°, hard-hit, GB/LD/FB/PU), §5 (K% 22.5, BB% 8.3, Swing%, Contact%, foul share), and §6.2 (runs 4.39).

**Order of work**, bottom-up (calibration-targets §2.1):

1. **Tier A/M first**: the oracle and goldens pass; then the cage (`swing_cage_probe`, and `swing_contact_baseline` against [the baseline](calibration/2026-11-24-swing-contact-baseline.md)): the timing table's rows from −3 ms on identical within noise, the earlier rows reported with §6.5's expectation beside them, and bat speed by contact depth against Statcast.
2. **The standard run**, before and after (ledger row). If K%, BB% and runs are within noise or move as the table says, stop: no retuning.
3. **If something moves more than expected**, look for a bug first (a planner or path error, AGENTS.md), then the checked-swing mechanism (§8): the brake and the ruling are the only new physics on the game path.
4. **Only then retune**, and only what calibration-targets §2.3 allows: `K_swing` (the miss scale) and the `BatSpeed` population mean, in that order. **Never** the anchors, the jet's Statcast fits, or a checked-swing threshold toward a stat. If late contact is off a Tier M band (speed by depth, spray by timing), the fix is the interior shape from [4]'s curves (step 0), not a knob. Swing length, if it leaves 6–8.5 ft, is a Tier M finding about the path's shape (§15), fixed in the curves' conditions from [4], never through speed.
5. When the batter or the contact depth is moved (D26, Q46: a later step, only once a plate-relative contact depth confirms it), that is its own calibration step after this one: perception, HBP and the late path (the hands' anchors move with the body, §5.3) change with it, and K%, BB%, HBP%, late contact and the timing of contact are re-measured; a calibration report under docs/calibration/ names each change and its band.

## 13. Build order

Each step testable and reviewable on its own. Sizes are rough Opus working time.

| Step | What | Game change | Size | Review |
| :-- | :-- | :-- | :-- | :-- |
| **0. Data** (done 2026-11-24, [note](research/swing/9-step0-data.md)) | Read [4] Appendix B (the shoulder angles' trunk frame) and, if legible, Appendix E's curves (azimuth, elevation, lag between the anchors). Re-pull the Statcast sample (calibration note 2026-11-14 §8 has the query) and compute the intercept relative to the batter (x, y), with Savant's definition of the batter's position. Write the numbers into §4.2, §5 and §5.3 of this spec, and the owner's choices of §14 are taken with them. | none | half a day | none (data; the Coordinator reads it) |
| **1. Oracle** (done 2026-11-24, §11) | `analysis/oracle/swing_v3.py` (beside v2's `swing_oracle.py`): the body (§5.3), χ_L and χ_93 by the misfit (§6.1), the jet (v2's), the curves, braking; `golden/swing_v3.json` with §11's cases 1–8 and the bounds; the elevation fix (§6.3) and the aim range settled (§6.3's last item); §11.1 the table for step 2. | none | 1 day | Opus, with the spec |
| **2. Rust, behind a switch** | `bb_agents::swing`: v3 path as a `BatPose` beside v2 (`SwingTemplate::model`, default v2); the clock-based brake, A_max and the new ruling apply to v3 only; tests from the goldens. Digests unchanged (v2 default). | none | 1 day | Opus (sim) |
| **3. Export E2 and D24** | L in `BbPlayInfo`, S samples in `BatTrack`, u_g, the `CheckSwingBrake` marker, the track's end at t_c + 0.03 s; schema, FFI, tests. Empty while v2 is the game's swing (§10). Ledger: "No effect on games". | none | half a day | Sonnet |
| **4. Switch the game to v3** | default v3; the ledger row (standard run before/after); the baseline example against the baseline; the checked-swing counts; the recalibration of §12 if needed; agent-models §3.6 replaced by this text, physics-spec §6.3 and rules-spec §4.5 updated. | yes | 1 day + runs | Opus (sim) |
| **5. Where the batter stands** (D26, Q46 = a: only once a plate-relative contact depth confirms he stands deeper) | stance spot and/or contact depth, eye point, body capsules; its own ledger row and calibration report. | yes | 1 day + runs | Opus |
| **6. Remove v2's path** | v2 kept only as the contact planner; the switch deleted. | none (digests identical) | an hour | Sonnet |

**The client** (player-visuals §13 steps, and `-BbBatSim`):

- **Step 1 (built)**: the load ends on L from E2 instead of on the sim's first swing pose; the handover (`BatHandoverS`, up to 48 cm) goes away, and with it the 3 of 406 swings that ended it more than 2 cm off. `-BbBatSim` becomes the default once step 4 lands: the sim's bat is now the one to draw from L on (the reason it was off, Q44, is answered). §13.10's note that L "is not what §13.1 describes" is resolved: L is now [4]'s.
- **Steps 2–3** (legs, coil, hip turn): the body's keys are re-checked so the figure's clavicle passes through the sim's S at launch, −93 ms and contact (a new check: within 2 cm); the chest's correction toward the hands (§13.4) should become small, and a large one is a defect. The elbows at contact become a real test of the sim's hands (rear 81°, lead 56°).
- **Step 4** (follow-through): starts from the sim's bat at t_c + 0.03 s (D24) as planned.
- **Step 5** (early, late, checked): the checked swing stops the body at the `CheckSwingBrake` marker; the braked bat is now well back, so strip 9's picture ("bat stopped in front of him with the barrel behind the plate") becomes "bat stopped behind him, the barrel still up".

## 14. The owner's choices, decided (PLAN D26)

The four choices this section put to the owner (Q45–Q48) were decided on 2026-11-24, all as recommended (option a): "go with your recommendations for all". What each means, and what it leaves to a later step:

**Q45. How the bat comes into the ball on a late swing: the measured swing all the way to the ball.** The bat turns fastest 9–22 thousandths of a second before contact, as in the lab, and is faster, further back and (since step 1) higher on a late swing than today's (§6.5): 25 thousandths early its sweet spot is at 70–72 mph against today's 63, 11–14° higher and 4–7° further back; balls met on time or out front are exactly as now. Not forced to MLB's bat tracking now. *Left to step 4 (recalibration):* the check against MLB's bat speed by contact depth (MLB: 3.4 mph slower for balls met a foot or more deep; v3 now 2 mph *faster* at −15 ms, today's game 3.6 slower). If it fails, the fix is the measured curve's interior (§6.3), not a speed setting.

**Q46. Where the batter stands: not moved in this rebuild.** MLB's bat tracking puts the ball 0.775 m in front of the batter's centre of mass; the game meets it 0.05 m in front of his stance spot, so [4]'s arms and bat-to-arm angle at contact cannot be reached from where he stands (§5.3). He is moved deeper in the box as its own step after the swing (§13 step 5), and only once a plate-relative contact depth confirms that he stands deeper rather than meeting the ball further out front. *Left until then:* the lag at contact (high-outside), the uncocking rate and the hands-ahead-of-the-chest checks are expected failures with today's stance; 0.55 m deeper they pass (§11).

**Q47. Checked swings: accepted as the measured swing gives them, and counted.** The braked bat stops far back (§8.3), so nearly no checked swing will be a strike. The brake's timing (before 0.6 of the swing, 0.04 s) and the ruling's threshold (the bat turned to across the plate) stay; changing either needs a source (video of check swings and the calls on them). *Left to step 4:* count them.

**Q48. The measured swing is slower than the game's: squeezed into each batter's own swing time.** [4]'s times are scaled by s_T = T_swing / 0.170 s, so the decision clock and the plate-appearance rates stay; the bat starts from rest at L. *Left for later (found in step 1, §6.3):* the measured shape has 168° still to turn at [4]'s −93 ms, whatever the bat speed, while the bat speed at contact comes from `BatSpeed` and effort. A slow swing or a short swing time therefore turns faster in mid-swing than at contact: the sweet spot reaches 1.10–1.18 × its contact speed on a 10% slow swing and up to 1.36 × at T_swing 0.130 s (grade 50: at most 1.12, at the edges of the zone). Today every batter's T_swing is 0.150 s, so only the slow swings (`BatSpeed` and effort, about 6% between them) show it. When `QuickHands` starts to vary T_swing, the coupling of swing time and bat speed is the owner's to decide (a quicker swing with an average bat is not what [4] measured).

## 15. Not established, and what would settle it

- **[4]'s curves between its stated instants** (§5.1): read in step 0 (§5.4) and, since step 1, used as data from −93 ms to contact (§6.3): the elevation follows [4] within 0.8° to −30 ms, the azimuth within 4.2° of its graph. **Not established:** how a pitch's location changes the elevation curve (Appendix E draws only middle fastballs; v3 scales [4]'s fall to the aim's contact elevation, which [4]'s Table 1 gives by location: −24° to −39°), and how the attack angle of MLB's swings (about 8°, the jet's) changes the last 30 ms, where [4]'s level batting-practice swing and the jet disagree (§6.3, the path tilt).
- (Settled by step 0: the trunk frame of [4]'s shoulder azimuth is reading 1, §4.2. Remaining doubt: Appendix B labels no axes; the handedness is inferred from [4]'s own values.)
- **The body's position relative to the contact point** (§5.3): step 0 found the ball 0.775 m ahead of the batter's centre of mass (Savant), 0.7–0.9 m more than the sim; **not established:** how much of that is a deeper stance and how much a ball further out front of the plate, and the instant and frame of Savant's centre of mass. A plate-relative contact depth (Hawk-Eye or a similar source) would settle it.
- **Swing length.** Savant (https://baseballsavant.mlb.com/csv-docs, and the bat-tracking leaderboard): "The total (sum) distance in feet traveled of the head of the bat in X/Y/Z space, from start of tracking data until impact point." The start is **not defined** ("start of tracking data"; no speed threshold or time is given), and it is the *head* of the bat (v2's comparison used the sweet spot, 6 in nearer the knob). Step 0, from v3's path (probe, three golden aims): the head travels 10.0–10.6 ft from t_s, 9.3–9.7 ft from −100 ms, 8.2 ft from −75 ms and 5.8–6.1 ft from −50 ms; **Savant's mean 7.3 ft is what v3's head travels from about 65 ms before contact** (−65 ms on all three aims; the sweet spot's 7.3 ft starts near −72 ms). So v3 agrees with Savant if its tracking starts 65 ms before contact (bat 124° of azimuth short of contact, 0.7 of its contact rate), a plausible start for a sensor that detects the swing; it cannot be settled further without Savant's definition. Savant's swing length also grows with contact depth (6.4 ft for balls 12 in or more deep, 8.4 ft for 12 in or more out front: `savant_bat.py`), the signature of a start fixed in time or in the bat's state rather than in the path. Step 1's oracle, on §6.3 as revised: the head travels 7.34 / 7.37 / 7.41 ft in the last 65 ms (middle, inside-low, outside-high), a check that passes (§11).
- **The hand speed at contact** (v2's q, unsourced): [3]'s uncocking (1,588°/s) and arm (1,160°/s) rates and [4]'s rear elbow extending at 752°/s at contact suggest 6–9 m/s against v2's 6.4; a check, not a constraint.
- **Live MLB timing**: [4]'s pitches were 56 mph. The swing's own 0.150–0.170 s agrees across sources; the scale s_T is an assumption (§4.3).
- **Real check swings**: no source here (§8.3).
- **The lead arm's reach at launch** (§6.1 residual: 12 cm long at 72° of lead elbow flexion): the proportions, the clavicle's position and the trunk frame; step 0 and the figure.
- **The arm-vector turning rate** fails its [3] bound in time or size whichever stance is used (§11). Step 0: Appendix E does not show it (no wrist or clavicle curve; the lead shoulder's azimuth rate peaks at 342°/s, a different quantity). What would settle it: [3]'s supplement or markers from OpenBiomechanics as a cross-check only (D21).
- **The path tilt over the last 40 ms** (§6.3): v3's sweet spot leaves the jet's plane (5° steeper at the middle aim, 9° inside-low), because the hands' curve and [4]'s elevation are not in the plane Statcast's tilt and attack angle give the jet. What would settle it: a lab or tracking source with both the bat's elevation in the last 40 ms and the attack angle (MLB's, not batting practice), or Statcast's tilt by contact depth, which would show whether real late contact keeps the plane.
- **Swing time against bat speed** (§14, Q48): [4] gives one swing time for its batters; how a quicker or slower bat changes the mid-swing (whether a slow swing also starts earlier) is not measured here.
- **What v3 does to games** has not been measured: nothing is built. §12's table is an expectation from the baseline, the construction and the oracle's aims.
