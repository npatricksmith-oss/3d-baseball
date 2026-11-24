# The swing model, v3: the measured swing (proposed rewrite of agent-models §3.6)

**Status (2026-11-24): specified, not built.** PLAN D25 (owner Q44 = b): the sim's swing is rebuilt whole from the measurements. This file is the rewrite of [agent-models §3.6](agent-models.md#36-swing-model) and of the lines of [physics-spec §6.3](physics-spec.md#63-bat-pose) and [rules-spec §4.5](rules-spec.md#45-check-swing) it touches (§6.6 and §8.4 below hold their proposed text). Until build step 4 (§13) switches the game to it, agent-models §3.6 (v2) describes what the code does; at that step this text replaces §3.6 there and this file is reduced to a pointer. Written by worker swing-spec on branch `work/swing-rebuild` from main `aa4a31d`.

**Sources.** [4] Fortenbaugh 2011 (33 AA hitters, live batting practice at 25 m/s, 300 Hz; the base) and [3] Welch et al. 1995 (7 professionals off a tee, 200 Hz) give every number of the swing's shape (D21). Statcast (MLB 2025 bat tracking, as fitted in [docs/calibration/2026-11-14-swing-model.md](calibration/2026-11-14-swing-model.md)) gives the contact instant, as v2 does. OpenBiomechanics is not used (D21). Every value read from [4] was checked against its text (`sim/target/research-dl/4/s.txt`, §4.1 and Appendix C's analysis code); the research notes are [docs/research/swing/](research/swing/swing-synthesis.md), and the diagnosis that led here is [launch-position.md](research/swing/launch-position.md). Today's swing at contact is measured in [docs/calibration/2026-11-24-swing-contact-baseline.md](calibration/2026-11-24-swing-contact-baseline.md) (the baseline, §2).

Tags as elsewhere: [S] sourced, [E] engineering value (named, configurable), [C] convention. "τ" is time from the nominal contact t_c (negative before). Times written "[4] −93 ms" are [4]'s own; §4.3 says how they map onto a batter's swing.

## 1. Summary

Today's swing (v2) is one rigid plane laid backward from the contact pose. It meets the ball the way MLB hitters do (Statcast) but before about 40 ms ahead of contact it is a different swing: the bat flat and pointing at the catcher where a measured hitter's is 40° up and wrapped behind his head, in line with the arms where his is cocked 120–130° behind them (diagnosis §3).

v3 keeps what v2 gets right and rebuilds the rest:

- **What moves.** The bat (the only physical body), held at the **hands** (mid-wrists, a point on the handle), whose path is built round a **shoulder point S** (the clavicle, the root of [4]'s arm vector) that follows the body. Three elements; the pelvis, trunk turn, legs and head stay presentation keyed to the sim's clocks (§3).
- **Where it starts.** A fixed **launch pose L** per batter, the same on every pitch, at rest: the bat 41° up and wrapped (azimuth −225°), cocked 121° behind the arms, the hands just in front of the back shoulder ([4] at weight commitment, −170 ms) (§6.1).
- **How it meets the ball.** The **contact jet**, the pose, velocity and acceleration at t_c, is v2's plan exactly: the sweet spot at the aim point at `BatSpeed`, with Statcast's attack angle, attack direction, path tilt and vertical bat angle, and v2's reach rule (§6.2). Statcast is the better source for the instant that decides the game, and [3] and [4] agree with it within their spread at contact, except on when the bat is fastest (an owner choice, §14).
- **Between.** Smooth curves in the bat's azimuth and elevation and in the hands' position, from L at rest through [4]'s mid-swing anchor (−93 ms: azimuth 168° short of contact and turning at 56% of its contact rate, elevation still 39°, lag 131°) into the contact jet (§6.3). The lag through the swing is an outcome checked against [4] (121° → 133° max at −80 ms → 65° at contact).
- **Checked swings** brake by slowing the swing's own clock, so the bat stays on its path; the ruling measures the bat's turn from L (rules-spec §4.5's wording), so a wrapped bat is no longer "a swing" before it moves (§8).
- **Determinism and noise** are as v2: the same five execution draws in the same order on the same two streams; no new RNG purpose (§9).
- **What changes in games**: contact more than about 10 ms off the planned instant a little (third-order terms of the path), checked swings a lot (the braked bat is far back: about 0.6 fewer strikes a game). The recalibration should be small (§12).
- **A finding that needs data before the build** (§5.3, §14): the measured arms at contact ([4]'s shoulder angles and its 65° bat-to-arm angle) put the hands about 0.4 m in front of the batter's chest, toward the pitcher. With today's contact depth and stance spot the sim's batter would have to stand about half a metre further back in the box than he does (at about the same distance from the plate), or meet the ball that much further out in front. Either the sim's batter stands in the wrong place or its contact point is too deep (its depth level, 0.35 m, is unsourced); Statcast's batter-relative intercept settles it.

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

**By construction v3 reproduces** every quantity at the planned contact instant (the jet is v2's) and, to first and second order in the timing error, the timing table and the slopes. **It moves** contact far from t_c a little and checked swings a lot (§12).

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
- **Shoulder elevation and azimuth** (`PolarShoulder()`): the polar angles of the vector from the elbow to the shoulder joint centre, in the upper trunk's frame, the same formula for both arms. Elevation 0° = arm hanging, 90° = horizontal. Azimuth = atan2(y, x) − 90° in the trunk frame, whose axes Appendix B's diagram defines (not yet read). **Provisional reading** [E]: azimuth 0° = the upper arm pointing forward from the chest, negative = turned toward the batter's rear side (for the rear arm, out to the side and back; for the lead arm, across the chest). It is the only reading in which [4]'s values describe a loaded hitter (rear arm −84°: the back elbow out and back; lead arm −30°: across the chest) and a hitter at contact with both arms in front of him (lead −1°, rear −9°) that fits [4]'s 65° lag; the other readings put the hands beside or behind the chest at contact. **Step 0 confirms it (§13).**
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
| Lag at launch | 121° | [4] §4.1.4 ("from 121º to 131º") | constraint, L |
| Rear elbow, rear shoulder elevation / azimuth, lead elbow at launch | 128°, 71° / −78°, 72° | [4] §4.1.4 (start-of-phase values) | constraint, builds W_L (§6.1) |
| Trunk (upper trunk, global) turn at launch | −21° [E] | [4] §4.1.3: −22° at −186 ms; §4.1.4: −22° → 2° over −170 to −93 | constraint, builds W_L |
| Bat azimuth at −93 ms | A_c − 168° | [4] §4.1.4–4.1.5: −175° at −93 ms, −7° at contact (sweep 168°) | constraint |
| Bat azimuth rate at −93 ms | 0.558·Ȧ_c | [4]: 1,247°/s at −93 ms against 2,233°/s at contact | constraint |
| Bat elevation at −93 ms | 39° | [4] §4.1.5 | constraint |
| Lag at −93 ms | 131° | [4] §4.1.4 | constraint, builds W_93 |
| Rear elbow, rear shoulder elevation / azimuth, lead elbow at −93 ms | 126°, 49° / −42°, 68° | [4] §4.1.4 (end-of-phase values) | constraint, builds W_93 |
| Trunk turn at −93 ms | +2° | [4] §4.1.4 | constraint, builds W_93 |
| Pelvis forward of the stance at −170, −93 ms, contact | +12, +17, +17 cm (and 5 cm rise over the last 93 ms) | [4] §4.1.1–4.1.5: back 6, then forward 7 (stride), 11 (coiling), 5 (initiation) cm | constraint, S (§5.3) |
| Lag maximum | 133° at −80 ms | [4] §4.1.5 | check ±5°, ±10 ms |
| Lag at contact, by location | middle 65 ± 10°; HI 79, HO 63, LI 58, LO 51 | [4] §4.1.5; Table 1 (means by location) | check (§5.3), with the next row |
| Arms at contact | lead shoulder elevation 70°, azimuth −1°; rear 35° (min, at −23 ms), −9°; lead elbow 56°, rear 81° | [4] §4.1.5, Table 1 | check: the hands in front of the clavicle toward the pitcher (§5.3) |
| Peak lag (uncocking) rate | 1,588°/s at −20 ms | [3] Table 2 | check: the peak magnitude within ±25% when scaled to the contact rate; at −20 ± 10 ms |
| Peak arm-vector turning rate | 1,160°/s at −65 ms | [3] Table 2 (arms about the trunk axis) | check ±25%, ±15 ms |
| Peak bat azimuth rate | 2,435°/s at −10 ms; 2,233°/s at contact | [4] §4.1.5–4.1.6 | check of the shape **only if the owner takes the lab's speed peak** (§14); otherwise reported |
| Bat speed peak (barrel end) | 31 m/s at −15 ms; 29 m/s at contact | [3] "Ball Contact" | as the row above |
| Bat's speed toward the plate and downward | peaks of 19 and 16 m/s near −40 ms | [3] | check ±25% (today's sim: 20.1 and 10.1) |
| Elbows at contact | rear 81 ± 8°, lead 56° | [4] Table 1 | client check (player-visuals §13.7) |
| Bat elevation, azimuth at contact | −30 ± 5°; −8 ± 12° | [4] Table 1 | check (the jet gives −29.5°, +7.7° today: inside both) |
| Contact-instant values | sweet-spot speed, attack angle α, direction ψ, path tilt β over the last 40 ms, vertical bat angle, depth y_c, reach gain g | Statcast via agent-models §3.6 v2 | constraint, the jet (§6.2) |
| Swing length | 7.2–7.3 ft | Statcast | check, with Statcast's own start-of-swing definition (§15) |

### 5.1 What the sources do not give, and what v3 assumes

Plainly, so no one mistakes an assumption for a measurement:

- **The curves between the stated instants.** [4] states azimuth, elevation and lag at −170, −93, −80 (lag only) and contact; its Appendix E graphs have the full curves and were not read (they are figures). v3 joins the stated values with the lowest-order smooth curves (§6.3) [E]. Reading Appendix E (step 0) would replace the interior shape with the measured one or confirm it.
- **The hands' position at any instant.** Neither [3] nor [4] gives it. v3 builds it at −170 and −93 ms from [4]'s arm angles on the figure's segment lengths (§6.1) and takes it from the contact jet at t_c. Between, it is a smooth curve [E].
- **The elevation at launch** (41° [E], between [4]'s 44° and 39°).
- **The trunk frame of [4]'s shoulder azimuth** (§4.2): assumed, to be confirmed.
- **Where the body is relative to the contact point** (§5.3).
- **The hand speed at contact.** [3] and [4] do not give it; v3 keeps v2's (q = 0.32, hands at about 6.4 m/s; [3]'s arm and uncocking rates suggest 6–9 m/s, a check, §11).
- **Anything after contact.** v3's track ends at t_c + 0.03 s (D24); the client draws the follow-through.

### 5.2 Segment lengths

Upper arm 0.186·H, forearm with hand to the mid-wrists 0.146·H, half the shoulder width (clavicle to shoulder joint centre) 0.112·H, clavicle above the mid-hips 0.28·H [E: standard anthropometric fractions of height; the client's figure must use the same fractions, so the drawn arms reach the sim's hands exactly]. At H = 1.85 m: 0.344, 0.270, 0.207 and 0.518 m.

### 5.3 The shoulder point S, and the body at contact

S(τ) = P + Δ(τ), P the batter's stance reference (today `SwingTemplate::pivot`: x = −0.95 m, y = 0.30 m for a right-hander). Δ is built from [4]'s pelvis travel (table above), the clavicle 0.28·H above the mid-hips, and the trunk's lean: forward flexion over the plate 8° at launch and 13° at −93 ms ([4] §4.1.4); toward the rear side at contact by the pelvis's obliquity plus the trunk's lateral flexion, 27° + 4° ([4] §4.1.5; [3]'s 20° is global). The three positions (launch, −93 ms, contact) are joined by the same kind of curve as the hands (§6.3), with zero velocity at none of them [E: velocity from the neighbouring anchors]. S is the same on every pitch: it does not depend on the aim or on whether he swings (D22). It is exported so the client puts the figure's clavicle there (§10).

**The finding.** The lag at contact is fixed by three things: the bat's line at contact (the jet), the hands' point on it, and S. With the jet's bat (elevation −30°, azimuth +5°, the sweet spot at the aim) and [4]'s arms at contact (both arms in front of the chest, §4.2), a 65° lag needs the clavicle about **0.4 m behind the hands toward the catcher and 0.3 m above them**, nearly level with them across: the hands in front of the belly, the arms pointing down and toward the pitcher, the bat across the plate. That is how a hitter at contact looks. The jet puts a middle pitch's hands (mid-wrists) at about x = −0.55 m, y = +0.30 m, z = 1.06 m, so the clavicle would be near x = −0.6 m, y = −0.1 m. S built from today's stance spot, with [4]'s pelvis travel (+17 cm) and the trunk's lean at contact (about 31° toward the rear side, which at contact is toward the plate, and 13° forward), sits near x = −0.66 m, y = +0.5 m, z = 1.38 m: about right across, but **about 0.5–0.6 m too far forward**. With that S the lag even comes out near 50°, but with the hands behind the chest, which no hitter does; so the check is the arms' direction (the next anchor row), not the lag alone.

So either the sim's batter stands in the wrong place (the stance spot, 0.30 m from the plate's back tip toward the pitcher, is set by eye) or the sim's contact is too deep (contact depth y₀ = 0.35 m is [E]; only its slope across the plate came from Statcast). Statcast's bat tracking records the intercept relative to the batter (`intercept_ball_minus_batter_pos_x_inches`, `_y_inches`, used for the slopes in agent-models §3.6 and `analysis/statcast/savant_bat.py`); the sample's CSVs are not on this machine. **Step 0 (§13) re-pulls them, reads Savant's definition of the batter's position, and computes the mean intercept relative to the batter**; then the owner chooses how to place the batter (§14). Until then the anchor stays a check that is expected to fail at contact; everything else in this spec does not depend on it.

## 6. The path

### 6.1 The launch pose L

One pose per batter (his height and handedness), the same on every pitch, at rest. Built once:

1. S_L from §5.3; the trunk turned −21° (coiled toward the catcher); the rear and lead shoulder joint centres 0.112·H either side of S_L along the trunk's shoulder line.
2. The rear elbow: 0.186·H from the rear shoulder at elevation 71°, azimuth −78° (§4.2, provisional reading): about 19° below shoulder height, out to the side of the coiled trunk, which at launch is toward the catcher and a little away from the plate.
3. The hands W_L: on the cone of points 0.146·H from the rear elbow at 128° of flexion, **at the point where the clavicle-to-hands vector makes λ = 121° with the bat** (A = −225°, E = 41°). The cone and the lag condition give at most two points; take the one in front of the rear shoulder (toward the plate). That is a one-dimensional root find on the cone's angle, done once per batter.
4. Check the lead arm: the lead shoulder to W_L must lie within the lead arm's reach at [4]'s 72° of lead elbow flexion (distance 0.10·H either side of that arm's length at 72°) [E]. If it fails, the trunk frame of step 2 is wrong (§4.2) or the figure's proportions are; fix those, not the lag.
5. The knob K_L = W_L − u_g·â_L.

W_93 is built the same way at −93 ms (trunk +2°, rear shoulder 49° / −42°, rear elbow 126°, lead elbow 68°, λ = 131°), with the bat's direction at that instant, which depends on the aim through A(τ_93) = A_c − 168°. So W_93 is solved per swing (the same one-dimensional root; a few µs), not per batter.

*Expected* (rough, from the figure in step 1 of the oracle): the hands about 0.1–0.2 m in front of and slightly above the rear shoulder, the bat pointing up and back over the head toward the pitcher's side. Today's swing starts with the hands 0.57 m behind his spot at chest height and the bat flat at the catcher (player-visuals §13.10).

### 6.2 The contact jet

At the executed nominal contact t_c the bat's state and its first two time derivatives are **v2's** (agent-models §3.6 v2, steps 1–8 and Placement), computed exactly as today by `plan_swing` and `apply_placement`:

- pose: knob K_c, axis â_c;
- velocity: knob velocity K̇_c, angular velocity ω_c = θ̇(0)·n̂;
- acceleration: K̈_c and ω̇_c = θ̈(0)·n̂, from v2's two rate bumps at τ = 0.

From these, at τ = 0: W, Ẇ, Ẅ (the point u_g along the bat); A_c, E_c and their first two derivatives (from â, â̇ = ω × â, â̈ = ω̇ × â + ω × (ω × â)).

This is how v3 keeps "match by construction" (§2): at t_c the sweet spot is at the aim, at `BatSpeed`·(1 + effort), with α, ψ, the plane tilt β and the vertical bat angle; the hands at q's share; the bat square to its path (v2's λ_c = 0, which is a statement about v2's hand-arc centre, not [4]'s lag); and the bat still speeding up at contact (v2's τ_b = +10 ms: Statcast's bat speed rises with contact depth out to 6–12 in out front). v2's template keeps its role as the **contact planner**: its widths and peak times now set only the jet's accelerations. The owner may instead take [3]'s and [4]'s speed history, which peaks 10–15 ms before contact (§14).

### 6.3 Between: launch to contact

On τ ∈ [−T_swing, 0], three groups of curves, each a polynomial in τ:

| Curve | Conditions | Degree |
| :-- | :-- | :-- |
| A(τ) | at −T_swing: A_L, Ȧ = 0, Ä = 0; at τ_93 = −0.093·s_T: A_c − 168°, Ȧ = 0.558·Ȧ_c; at 0: A_c, Ȧ_c, Ä_c | 7 |
| E(τ) | at −T_swing: E_L, 0, 0; at τ_93: 39°; at 0: E_c, Ė_c, Ë_c | 6 |
| W(τ), each of x, y, z | at −T_swing: W_L, 0, 0; at τ_93: W_93; at 0: W_c, Ẇ_c, Ẅ_c | 6 |

- **Closed form.** With u = τ/T_swing the interior knot sits at the fixed fraction u_93 = −0.093/0.170, the same for every batter, so each curve is a fixed linear combination of the conditions: the basis polynomials are computed once (in the oracle, and as constants in Rust with their derivation in a test), and each swing's coefficients are a dot product. No per-swing linear solve.
- **Pose at τ:** â = (cos E cos A, cos E sin A, sin E) (mirrored for a left-hander), W from its curve, knob K = W − u_g·â. **Velocity:** â̇ from Ȧ, Ė; ω = â × â̇ (the bat has no roll about its own axis, as in v2); the centre of mass moves at Ẇ + ω × ((u_cm − u_g)·â). All transcendental functions through `bb_math::m`.
- **Why these conditions.** At rest at the start, so the client's load ends on L without a jump; [4]'s mid-swing values where [4] states them; the jet at contact, so every contact quantity is v2's to second order. Ȧ at −93 ms is a ratio to the contact rate (not 1,247°/s) so that a faster swing is faster all through, and the elevation is held at 39° until −93 ms as [4] has it. The lag between the anchors is left to fall out and is checked (§11).
- **What must be verified** (the oracle does it before any Rust, §11): A increases monotonically on [−T_swing, 0]; E stays within [−40°, 50°]; the hands stay between 0.15 m and the arms' full reach from S; the hands' and the bat's speeds rise without a dip after −93 ms; the lag peaks near −80 ms. If a polynomial overshoots, the fix is one more interior condition from [4]'s stated values (for instance Ė = 0 at τ_93), not a different shape family.

### 6.4 After contact

For τ ∈ [0, +0.03 s] (D24) the bat follows v2's analytic swing (the bumps continued past t_c), which joins the curves of §6.3 with matching position, velocity and acceleration at τ = 0. Contacts found after t_c (57% of contacts today, all within +25 ms) therefore meet a bat that agrees with v2's to second order at t_c and exactly after it; the after-contact motion is otherwise cosmetic (physics-spec §6.3) and the follow-through is the client's.

### 6.5 Where v3 differs from v2 inside the contact window

On [−25 ms, 0] the curves and v2 share position, velocity and acceleration at 0 and differ by their third derivative. v2's bat angle has a jerk of about −320·k rad/s³ at contact (k ≈ 36 rad/s); a difference of that size moves the bat by about 0.1° at −10 ms and 2° at −25 ms. That bounds what v3 does to contact before t_c: about 1,000 contacts in 600 games fall more than 10 ms before it (baseline, timing table). The oracle reports the actual difference (§11).

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

The brake comes at −82 ± 32 ms (baseline). At −82 ms v3's bat is about 168° short of its contact azimuth (pointing away from the plate and up), and 20 ms more of path brings it to about −140°: well back. Only a brake started after about −25 ms (p95 is −29 ms) gets the bat to the plate. A braked v3 bat almost never touches the ball and almost never passes 0°. Today 3.4% of checked swings are strikes (310 touches, 36 on the yaw), about 0.6 a game; v3 should make nearly all of them balls. No source here measures how often a real check swing is ruled a swing; this is reported, and the owner may want it looked at (§14).

### 8.4 rules-spec §4.5, as proposed

Replace the first sentence with: *"A batter has **swung** if the bat's horizontal direction, turned from its pose at the start of the swing in the swing's own direction, reaches the point where the bat is perpendicular to the pitcher–home line **with its barrel toward the plate** (the barrel head level with the front of the batter) [C]. The swing model reports the furthest azimuth reached, A_max (agent-models §3.6: unwrapped from the launch pose, 0° = across the plate), and the batter swung if A_max ≥ θ_swing = 0° [C, configurable]. A bat that starts wrapped behind the head passes the other perpendicular (pointing away from the plate, −180°) early in every swing; that one does not count."* The rest of §4.5 (any contact is a swing) stands.

In code: `Swing::judged_swing` becomes A(σ(t_b + B)) ≥ 0 on the unwrapped azimuth, and `max_angle` reports A_max; today's `a.y ≥ 0` test is only right for a bat that starts pointing at the catcher, and would call a wrapped bat a swing before it moves (diagnosis §2).

## 9. Determinism

- Every function is closed form in τ: polynomials, `m::sin`/`m::cos` for â, one `m::atan2`/`m::acos` family for A and E of the jet, and the one-dimensional root for W_L and W_93 (a fixed number of bisection steps on a fixed bracket, so the same operations in the same order every time). No `mul_add`, no iteration that depends on a tolerance reached.
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
- The `CheckSwingBrake` marker (in the schema, never written) is written at t_b, so the client stops the body with the bat.

## 11. Goldens and the oracle

The Python oracle (`analysis/oracle/swing_oracle.py`, stdlib only) gains a v3 module that computes everything below **from this spec**, independently of the Rust; its fixtures go in a new `swing_v3.json` beside `analysis/oracle/golden/swing.json` and `sim/crates/bb_agents/tests/` reads them. v2's oracle and golden stay as the contact planner's (the jet is v2's), so the jet is tested against v2's existing fixtures.

**Golden cases** (a right-hander, H = 1.85 m, grade 50: T_swing 0.150 s, `BatSpeed` 32.0 m/s, effort 0; the 34-in bat; and a left-hander mirrored):

1. **L**: S_L, both shoulders, the rear elbow, W_L, K_L, â_L; with λ_L = 121° and A_L, E_L to 1e-9 (they are conditions), and the lead-arm distance check.
2. **The jet** at t_c for v2's golden cases `rhb_middle`, `rhb_inside_low`, `rhb_outside_high` and `lhb_middle_quick` (`golden/swing.json`): equal to v2's pose and velocity there, and to the acceleration of v2's bumps, to 1e-9.
3. **Interior anchors**: at τ_93, A = A_c − 168°, Ȧ = 0.558·Ȧ_c, E = 39°, λ = 131° (to 1e-9).
4. **Path samples** every 5 ms from t_s to t_c + 0.03 for the three aims: knob, axis, Ω, the centre of mass's velocity (Rust within 1e-9 m and rad).
5. **Continuity**: position, velocity and acceleration equal from both sides at τ = 0 (curves against v2's continuation) to 1e-9; at rest at t_s.
6. **Braking**: brakes starting at −120, −82, −40 and −25 ms: the stopped pose, A_max, and the ruling (expected: no swing for the first three; the −25 ms case states its own expected answer from the oracle).
7. **T_swing** 0.130 and 0.174: the anchors land at s_T·(−93 ms).

**Checks the oracle reports and the tests bound** (outcomes, not conditions; tolerances [E], wide enough for a model that is not fitted to them): lag maximum 133 ± 5° at −80 ± 10 ms (scaled); peak uncocking rate within ±25% of [3]'s 1,588°/s scaled to the contact rate, at −20 ± 10 ms; peak arm-vector turning rate (about S, horizontal) within ±25% of 1,160°/s scaled, at −65 ± 15 ms; the bat's speed toward the plate and downward peaking near −40 ms; swing length against Statcast (with its definition, §15); the plane tilt over the last 40 ms within 1.5° of β; the difference from v2 at −10 and −25 ms (§6.5); and the lag at contact by location against [4]'s Table 1 (reported, failing only after §5.3 is settled). Monotone A, bounded E, hands within reach (§6.3).

**Property tests** (Rust): |â| = 1; the velocity the pose reports equals the finite difference of positions (as v2's test); A non-decreasing on every unbraked swing; a braked swing never moves after t_b + B; mirror symmetry for a left-hander; determinism (the same plan twice gives identical bits).

**Game-level tests**: a test that every swing's contact quantities at the planned instant (sweet-spot speed, α, ψ, bat elevation) equal v2's for the same plan; the `bb_game` crate-graph test unchanged.

## 12. The recalibration plan

**Expected to move, and why:**

| Measure | Expected | Why |
| :-- | :-- | :-- |
| Checked swings ruled swings | 3.4% → near 0 | the braked bat is far back (§8.3) |
| K% | −0.1 to −0.3 points | about 0.6 strikes a game become balls |
| BB%, pitches per PA | +0.1 to +0.2 points; up slightly | the same pitches |
| EV, LA, spray, foul share of contact | within noise; the far timing rows may move a little | the jet is v2's; only third-order terms differ (§6.5) |
| Contact%, whiff% | within noise | as above, and misses are decided in the window |
| HR/PA, runs | within noise, follow K% and BB% | |
| HBP% and everything perceptual | unchanged **unless** the owner moves the batter (§14) | the eye point and body capsules follow the stance spot |

**Match by construction** (§2): every contact-instant quantity and the slopes.

**Bands** (calibration-targets): §4.3 (bat speed 71.5–72 mph, swing length about 7.3 ft, attack angle, commit-to-contact 0.15 s), §4.4 (EV 88.5, LA 12°, hard-hit, GB/LD/FB/PU), §5 (K% 22.5, BB% 8.3, Swing%, Contact%, foul share), and §6.2 (runs 4.39).

**Order of work**, bottom-up (calibration-targets §2.1):

1. **Tier A/M first**: the oracle and goldens pass; then the cage (`swing_cage_probe`, and `swing_contact_baseline` against [the baseline](calibration/2026-11-24-swing-contact-baseline.md)): every contact-instant row identical, the timing table's middle rows identical within noise, the far rows reported.
2. **The standard run**, before and after (ledger row). If K%, BB% and runs are within noise or move as the table says, stop: no retuning.
3. **If something moves more than expected**, look for a bug first (a planner or path error, AGENTS.md), then the checked-swing mechanism (§8): the brake and the ruling are the only new physics on the game path.
4. **Only then retune**, and only what calibration-targets §2.3 allows: `K_swing` (the miss scale) and the `BatSpeed` population mean, in that order. **Never** the anchors, the jet's Statcast fits, or a checked-swing threshold toward a stat. Swing length, if it leaves 6–8.5 ft, is a Tier M finding about the path's shape (§15), fixed in the curves' conditions from [4], never through speed.
5. If the owner moves the batter or the contact depth (§14), that is its own calibration step after this one: perception and HBP change with it, and K%, BB%, HBP% and the timing of contact are re-measured; a calibration report under docs/calibration/ names each change and its band.

## 13. Build order

Each step testable and reviewable on its own. Sizes are rough Opus working time.

| Step | What | Game change | Size | Review |
| :-- | :-- | :-- | :-- | :-- |
| **0. Data** | Read [4] Appendix B (the shoulder angles' trunk frame) and, if legible, Appendix E's curves (azimuth, elevation, lag between the anchors). Re-pull the Statcast sample (calibration note 2026-11-14 §8 has the query) and compute the intercept relative to the batter (x, y), with Savant's definition of the batter's position. Write the numbers into §4.2, §5 and §5.3 of this spec. | none | half a day | none (data; the Coordinator reads it) |
| **1. Oracle** | `swing_oracle.py` v3: L, W_93, the jet (v2's), the curves, braking; goldens §11 cases 1–7; the checks report. Settle overshoots here (§6.3). | none | 1 day | Opus, with the spec |
| **2. Rust, behind a switch** | `bb_agents::swing`: v3 path as a `BatPose` beside v2 (`SwingTemplate::model`, default v2); the clock-based brake, A_max and the new ruling apply to v3 only; tests from the goldens. Digests unchanged (v2 default). | none | 1 day | Opus (sim) |
| **3. Export E2 and D24** | L in `BbPlayInfo`, S samples in `BatTrack`, u_g, the `CheckSwingBrake` marker, the track's end at t_c + 0.03 s; schema, FFI, tests. Ledger: "No effect on games". | none | half a day | Sonnet |
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

**When is the bat fastest: before the ball or just after it?** *Now:* the game's bat is still speeding up when it meets the ball and is fastest about a hundredth of a second later, as MLB's bat tracking implies (bat speed keeps rising for balls met 6–12 inches further out front). *The lab studies* ([3] off a tee, [4] in batting practice) have the bat fastest 10–15 thousandths of a second before contact and slightly slower at the ball. *Options:* (a) keep MLB's timing at contact and the lab's shape everywhere else: batted balls barely change; (b) take the lab's: balls met late are hit harder than now and balls met out front softer, pull and fly-ball rates by timing shift, and the hitting calibration is redone in earnest. *Recommended:* (a). MLB game data decides the moment that decides the game, the lab measures a different quantity (the bat's turning rate in [4], a barrel marker smoothed at 13 Hz in [3]) on slower pitches, and the two agree everywhere else.

**Where does the batter stand?** *Now:* the game's batter stands about 0.95 m off the middle of the plate, roughly even with it, and meets a middle pitch about 5 cm in front of where he stands. *The measurements:* at contact a hitter's hands are about 0.4 m in front of his chest, with the bat cocked about 65° to his arms. For the game's bat to have that relation to his arms, he would have to stand about half a metre further back in the box than the game's batter does (about as far from the plate), or meet the ball that much further out in front. MLB's bat tracking records where the ball is met relative to the batter and can settle which (step 0). *Options:* (a) once the data are in, move the batter (deeper in the box) and with him the point he watches from and the body the pitch can hit: hit-by-pitch and pitch recognition change a little and are re-measured; (b) meet the ball further out in front instead (if the data say so): the ball is met earlier and higher, which changes timing and contact and needs a fuller recalibration; (c) leave him where he is: the swing is measured everywhere except that at contact his arms and bat are nearly in line (the look today). *Recommended:* decide after step 0's numbers; (a) if they confirm the gap, done as its own step after the swing.

**How far a checked swing goes.** *Now:* about one held-up swing in thirty is a strike (touching the ball or the bat passing the front of the plate). *With the measured swing* the bat is still far back when the batter brakes, so nearly none will be. No source here says how often a real one is called a swing. *Options:* (a) accept and measure it; (b) let the batter brake later or more slowly (a later decision point than 60% of the swing, or a longer brake than 0.04 s) so some checked swings do go. *Recommended:* (a) now; (b) only with a source (video of check swings) to set it.

**The measured swing is slower than the game's.** *Now:* the game's swing takes 0.150 s from start to contact for an average batter (MLB's bat sensors), and the hitter's decision clock is built on it. [4]'s hitters took 0.170 s from their weight shift to contact against 56 mph batting practice. *Options:* (a) squeeze the measured shape into each batter's own swing time (the average one 12% quicker): the decision clock and the plate-appearance rates stay; (b) keep the measured times and start the game's bat 0.02 s into [4]'s swing, already turning: the start no longer comes from rest and the client's load has to hand over to a moving bat. *Recommended:* (a).

## 15. Not established, and what would settle it

- **[4]'s curves between its stated instants** (§5.1): Appendix E's graphs (figures in the PDF). Settles the interior shape; until then the curves are the lowest-order ones through the stated values.
- **The trunk frame of [4]'s shoulder azimuth** (§4.2): Appendix B's diagrams. The provisional reading is the only one that fits [4]'s own coil and contact descriptions; if Appendix B says otherwise, W_L and W_93 move and §5.3's numbers are redone. The lead-arm check (§6.1 step 4) also catches a wrong choice.
- **The body's position relative to the contact point** (§5.3): Statcast's batter-relative intercept and Savant's definition of the batter's position (step 0).
- **Swing length.** v3's sweet spot starts wrapped behind the head, so over its whole 0.150 s it travels further than v2's (an estimate: 8–10 ft against Statcast's 7.2–7.3 ft). Statcast's swing length runs from its own "start of swing", which must be read before comparing (v2 compared the last 150 ms). If v3 is long by Statcast's definition, it is a Tier M finding about the path's early part.
- **The hand speed at contact** (v2's q, unsourced): [3]'s uncocking (1,588°/s) and arm (1,160°/s) rates and [4]'s rear elbow extending at 752°/s at contact suggest 6–9 m/s against v2's 6.4; a check, not a constraint.
- **Live MLB timing**: [4]'s pitches were 56 mph. The swing's own 0.150–0.170 s agrees across sources; the scale s_T is an assumption (§4.3).
- **Real check swings**: no source here (§8.3).
- **What v3 does to games** has not been measured: nothing is built. §12's table is an expectation from the baseline and the construction.
