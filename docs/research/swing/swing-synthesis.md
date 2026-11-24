# The basic MLB-level swing: synthesis of eight sources (research note, 2026-11-23)

Written for the rebuild of the batter's base stance and swing (owner, 2026-11-23: "decent start but incomplete and too unnatural"). It describes one **generalized, idealized swing**; per-player variations come later (TASKS "Batting stances"). It changes no code and no calibration values. Each source has its own note; this page cites them as **[n]**.

**Decision Q41 (owner, 2026-11-23): every number of the base swing rests on Fortenbaugh [4] and Welch [3]; OpenBiomechanics [6] is used only as a cross-check** and is shown beside the chosen value as "check". (Reason: [6]'s licence is non-commercial and share-alike, and its swings are machine-pitched.) A quantity that only [6] gives is listed in §3 under "OBP only". **Decision Q42** is in §5, gap 1. Both are in [PLAN §1](../../../PLAN.md) (D21, D22).

| n | Note | What it is | Weight |
| :-- | :-- | :-- | :-- |
| 4 | [Fortenbaugh 2011](4-miami-study.md) | PhD thesis, 33 AA pros, **live** pitching (25 m/s), 300 Hz mocap + force plates | **base**: timing, sequence, angles, forces |
| 6 | [OpenBiomechanics](6-openbiomechanics.md) | 677 swings, 98 hitters (college 76%, MiLB/independent 11%), machine pitch ~65 mph, mocap + force plates. **Licence CC BY-NC-SA 4.0 (see the top of that note).** | **check only (Q41)**; slow machine pitch |
| 3 | [Welch et al. 1995](3-welch-1995.md) | JOSPT paper, full text read (via a connector, 2026-11-23): 7 right-handed pros (6 minor league, 1 major league), **off a tee**, 6 cameras 200 Hz + 2 force plates; tables partly illegible | **base**: sequence, peak speeds, forces; a tee swing, so coil and timing are a ceiling, not live |
| 7 | [Ochart webinar](7-video-xHktmnRAx1s.md) | coach quoting Driveline lab means; sample sizes not given | medium |
| 2 | [OnBase manual](2-sportngin-pdf.md) | coaching manual built on 3D research; almost no numbers | qualitative |
| 8 | [HDR video](8-video-KGs4MIRy9Yk.md) | one coach's youth cues | opinion |
| 1 | [Theia3D promo](1-video-4oPeCSbi0m8.md) | 53 s vendor promo, no content | none |
| 5 | [Driveline "Energy Flow" short](5-video-hac-hMcfZYk.md) | 4.5 s skeleton clip, no numbers | visual check of order only |

Measurements outrank opinion below. Where a number comes from one source, it says so. In the tables, "check" marks a [6] or [7] value shown beside the chosen one.

**Time convention.** t = 0 is bat-ball contact; negative is before. The two measured sets differ in pitch speed: [4] swings at 25 m/s (about 56 mph, 520 ms flight); [6] at a machine's ~65 mph. A major-league fastball is faster, so the **times before contact of the stride and load are likely shorter in the game than in [4]**; the swing itself (last ~170 ms) matches across sources. Treat anything earlier than about −170 ms as scalable to the pitch's flight time.

## 1. The base swing as one timeline

Positions are in the batter's own terms (RH batter; "lead" = front side, "rear" = back side). Angles are flexion unless stated; the sources' sign conventions differ ([4] 3.4; [6] "adjusted to match coach intuition"), so signs are restated in words. Ranges are source-to-source or SD.

| Time (ms) | Event | Feet / legs | Pelvis / trunk | Arms / hands / bat | Head / weight | Sources |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| ≤ −1000 to −620 | **Stance** (starts before pitcher moves) | Lead knee 36→48°, rear 51→56° as he settles; both feet down; weight drifts back (rear plate 57→86% BW, lead 37→17%) | Pelvis drifts back ~6 cm; upper body nearly still; slight forward lean of the trunk toward the pitcher ("attacking posture") | Hands near the rear shoulder; bat tipped back; rear elbow even with or in front of mid-hands, not flying | Head level, eyes on the pitcher. Feet roughly the width of the bat (HDR, one coach); stance width is **OBP only** (ankles 0.68 m apart along the line to the pitcher, lab frame) | 4, 2, 8 (check 6) |
| ≈ −700 to −500 | **Load / negative move**: weight into the rear leg and hip | Pelvis goes back ~6 cm [4]; centre of pressure 20 cm behind the centre of mass at foot off [3] (check: centre of mass back 0.12 m [6]; 3–4 in for a compact hitter, 12 in+ for a big mover [7]). Rear knee must not slide outside the rear foot (OnBase: 1 in is a sway) | Pelvis and torso turn back together, ~10° past the stance, almost no separation (~3–5°); torso coil ~8° [7]. Off a tee [3]: at foot off arms 150°, shoulders 30°, hips 18° coiled, trunk 21° forward and 6° left, back knee 32°, centre of pressure 20 cm behind centre of mass | Hands go back with the torso and down, knob toward the catcher (HDR) | Balanced; "can hold the position" (HDR) | 4, 3, 2, 8 (check 6, 7) |
| **−621** | **Lead foot off** (10% BW) | Lead knee peaks at 52° at about −483 [4] (an event-aligned mean, so a floor for a single swing; check: 83° [6]); **lift height is OBP only** (0.22 m) | — | — | Rear leg carries ~85–90% BW [4] | 4 (check 6) |
| −620 to −340 | **Stride (positive move)**: a controlled forward drift, not a push | Lead foot travels toward the pitcher and lands **closed ~11 cm** (foot angle 69→63°); stride **42% of body height** toe-to-toe at foot down [4], 85 cm [3], 12° closed [3] (check: 0.23 m beyond stance width, 0.07 m lateral [6]). Rear foot stays; heel stays down [2] | Pelvis forward 7 cm. Trunk coils slightly: pelvis counter-rotates to −21° at −250, upper trunk −22° at −186 (live [4]); off a tee the coil is bigger and earlier: hips max 28° at **−350**, shoulders 52° at **−265**, arms 185° at foot down [3]. Forward bend ~25° at landing (flaw: 32–35°+) | **Hands separate from the nose/chest** (stretch); bat wraps behind head, minimum azimuth at **−243 ms**; hands inside the lead elbow | Head stops advancing; posture leans slightly forward until landing | 4, 3, 7, 2 (check 6) |
| **−340 to −200** | **Lead foot down** (toe / ball first, soft: "stepping on thin ice"); 10% BW at −340 [4], −200 [6], toe-touch to contact 167–200 ms [2] | Foot plants closed to straight (lead foot angle 61° closed, stride direction 12° closed [3]; toe-to-toe 0.77–0.89 m, see §3); lead knee ~48° [4] (check: 45° [6]); lead foot 123% BW at foot down off a tee [3], centre of pressure jumps from 20 cm behind to 20 cm ahead of the centre of mass | Pelvis still at its counter-turn (−13° to −21°) and trunk at −16° to −22° [4]: coiled, **separation about 2°**. Check: [6] and [7] show the pelvis already ~25° open and at 566°/s at full plant with 18° separation; [4] does not (see §3) | Hands high and back, load still coiled; first 3–4 in of hand travel not begun | Posture reverses: slight backward lean at heel plant so head stops and motion turns rotational | 4, 3, 2 (check 6, 7) |
| **−170** | **Weight commitment** (lead foot 50% BW) = swing initiation | Rear GRF falls 83→47% BW; lead 50%→130% BW by −93 | Pelvis leads: ~−13°→20° in the next 80 ms | **First move**: back elbow leads, drops and moves forward; bat unwinds (236→1247°/s) | — | 4, 2 |
| −125 to −100 | **Peak hip-shoulder separation** | — | **18°** at −103 ms [4] (check: 21° at −123 [6], ~22° [7]) | Hands in front of rear hip; bat lag (angle behind hands) ~130° at −80 | — | 4 (check 6, 7) |
| −93 | **Max lead-foot force** | 130% BW [4] (123% at foot down [3]; check: 2.0 BW [6]); lead leg blocks | Pelvis 558°/s (95% of peak) | Back elbow passes rear hip; connection (hands, rear elbow and rear hip line up) | — | 4, 3, 2 |
| −75 to −57 | **Peak pelvis**, then **peak torso** (torso 13 ms later [4]; check: 4 ms [6]) | Lead knee extending (−40 ms peak 263°/s [4]; check: 350°/s at −50 ms [6]) | Pelvis turns from −21° (at −250) to 77° at contact, ~98° in about 250 ms [4] | Lead arm peaks next (−50 to −43) | Head still, then turns with chest | 4, 3 (check 6, 7) |
| −50 to −10 | **Peak lead arm, hands, then bat** | Rear knee flexes 47→54° [4] (check: 72° [6]); rear-hip drive and heel-up pivot are **OBP only** / seen in [5] | Torso decelerating (braking) as arm and bat accelerate | Bat lag 133°→66° at contact [4]; bat azimuth speed peaks **−10 ms**; bat lag uncocks at 1588°/s at −20 ms, bat linear peak 31 m/s at −15 ms [3] | — | 4, 3 |
| **0** | **Contact** | **Lead knee nearly straight: 18° [4], 15° [3]** (check 14° [6]), lead leg carries 84% BW [3]; rear knee 45° [3], ~54° [4] (check: ~71° [6]) | Pelvis ≈ square to pitcher (77° turn, [4]); torso 75° global, **separation closed to ~2°**; torso side-bent 20° toward the rear leg [3] (check: 28–32° [6,7]), trunk 9° extended [3] (forward bend traded for side bend) | **Both elbows still bent: rear 81°, lead 56° [4]** (check 79°, 53° [6]); bat barrel *below* the hands (−30°); bat speed 29 m/s at contact, 31 m/s peak at −15 ms [3] (check: 29.6 m/s college mean, ~31 m/s pros [6]); attack angle is **OBP only** (+4°) or a coach's average [7] (8–10°) | Head still, 47° turned toward the ball [4]; pelvis forward 23 cm in all since the stride began (7+11+5 cm) [4] (check: centre of mass 0.23 m forward [6]; its 0.07 m drop is **OBP only**, and [4] has the pelvis rising 5 cm) | 4, 3 (check 6, 7) |
| +17 to +83 | **Extension** | Lead knee peaks extension 11° at +73 [4] | Pelvis slows, torso keeps going | **Rear elbow extends** (peak speed +17 ms, minimum angle 37° at +83); lead elbow max extension +43 ms (42°), both arms straight about 50–100 ms after contact (check [6]) | Head turns back toward the field | 4, 2 |
| +100 to +200 | **Through** | Lead foot opens (49→36° by +60 ms and on); lead knee re-bends | Pelvis 77→95°; trunk 75→135° by +200 ms (trunk ~60° past the pelvis at +200 [4]) | Bat decelerates 2233→827°/s, rises −30→+28° elevation by +150, lag minimum 26° at +67; lead elbow flexes again (wrap) | — | 4, 6 |
| +200 to ≥ +430 | **Finish** | Stiff lead leg, rear foot on its toe (visual, [5]) | Torso well past pelvis | Bat wraps around, ends high behind the head, two-handed in the skeleton clip [5]; **no source measures the end**: [4] stops at +200 ms, [6] at ~+430 ms | Balance and leaving the box: not covered anywhere | 4, 6, 5 |

## 2. Kinematic sequence

Peak axial speeds and when they occur. Same order in every source.

| Segment | [4] Miami (AA, live) | [6] OBP (check only) | [3] Welch | [7] Ochart targets | Order |
| :-- | :-- | :-- | :-- | :-- | :-- |
| Pelvis | 581°/s (10.1 rad/s) at −70 ms | 705°/s (12.3 rad/s), SD 75, at −75 ms (SD 23) | 714°/s (12.5 rad/s), SD 76, at −75 ms | at least mid-600s°/s | 1 |
| Upper trunk | 766°/s (13.4 rad/s) at −57 ms | 915°/s (16.0 rad/s), SD 80, at −70 ms (SD 10) | 937°/s (16.4 rad/s) at −65 ms | up to 1100–1200°/s | 2, 4–13 ms after pelvis |
| Lead shoulder / upper arm | 342°/s azimuth at −43 ms | 1205°/s (21.0 rad/s), SD 137, near −49 ms | arm vector 1160°/s (20.2 rad/s), SD 96, at −65 ms (same time as the shoulders) | arm 1.2–1.3 × torso | 3 |
| Hands | not given | ~2080°/s (36.3 rad/s), SD 332, at or just before contact | bat-lag (uncocking) speed 1588°/s (27.7 rad/s) at −20 ms | ~2000°/s near −50 ms | 4 |
| Bat | 2435°/s (42.5 rad/s) azimuth at −10 ms | peak barrel speed within 5 ms of contact | 31 m/s linear peak at −15 ms, 29 m/s at contact; rear-elbow extension 948°/s at −15 ms | — | 5 |

Speed ratios from the base sources: pelvis→torso 1.3 ([4] 766/581, [3] 937/714); torso→arm 1.2 ([3] 1160/937). Checks: [6] 1.3, 1.3 and arm→hand 1.7; [7]'s coach targets 1.6 and 1.3. The torso decelerates as the arm accelerates; the sequence of slowing is the same order (OnBase [2] calls it the best predictor of efficiency). Bat speed: 29 m/s at contact, 31 m/s peak [3]. Check [6]: 66 mph (29.6 m/s) college-heavy, 69.5–70.5 mph (31.1–31.5 m/s) for 12 independent and MiLB hitters, whose gain is in arm and bat speed, not pelvis and torso speed.

## 3. Where the sources agree and conflict

**Agree (follow these).**
- Order pelvis → torso → lead arm → hands/bat, each peak higher than the last, in all of [2,3,4,5,6,7]. Peak speeds from [4], [6] and [3] agree within 20% (pelvis 581–714°/s, trunk 766–937°/s); the pelvis peak time agrees exactly between [6] and [3] (−75 ms).
- Separation is **small**, 18–22°, and peaks before the speed peaks; more is not better ([4,6,7]). It runs against the common "40–60° coil" picture.
- The stride is a controlled forward drift for timing, not power; the foot lands soft; the front side then firms up and the lead knee extends toward contact ([4,6,2,7]).
- Centre of mass first goes back, then forward ([6,7,4]).
- Elbows are still bent at contact and extend after it ([4,6]); the barrel is below the hands at contact ([4]).
- Head quiet after the foot plant ([7,2,4]).

**Conflict, and the choice for the base swing.**

| Issue | Sources | Choice and why |
| :-- | :-- | :-- |
| When the lead foot touches | −340 ms [4] (force-plate 10% BW, live pitch, soft landing); −200 ms (median −175) [6] (machine); −167 to −200 ms [2] (toe touch) | Follow [4]: down at **−340 ms** for a 520 ms flight (65% of the flight), weight committed at **−170 ms**. Scale the touch time with the pitch's flight (about −260 ms for a 0.40 s flight, an assumption); keep the commitment at −150 to −170 ms, where the sim's swing starts (`T_swing` 0.150 s). [6] and [2] are later (−200 to −167 ms): a check that the touch is not later than −200 ms. |
| Lead-knee flexion | 52° peak, 36→48° in stance [4]; 83° peak, 45° at plant [6]; 15° at contact [3] | **The scale is not the cause**: [3] and [4] both use 0 = straight, 180 = fully flexed, and the three agree at contact (14–18°). The peak differs because [4] averages swings aligned on events, which blunts a peak that varies in time; [6] takes each swing's own maximum. Base is [4]: stance 36–48°, lift peak 52°, 48° at the commitment, 18° at contact; [3] 15° at contact. The height of the lift is only in [6] (83°, 0.22 m ankle): **OBP only**, choose by eye. |
| Rear-knee flexion at contact | 45° [3] (tee); ~71° [6] | Use [3] 45° and [4] ~54°: 45–55°. [6] disagrees (71°) and stays a check. |
| Stride length | 85 cm toe-to-toe [3]; 42% of height [4]; stance 0.68 m → 0.89 m ankle separation [6] | **Same quantity, in agreement**: [4]'s thesis converts [3]'s 85 cm to ~46% of height, so 42% of 1.84 m ≈ 0.77 m is also toe-to-toe at foot down, and [6]'s 0.89 m is ankle-to-ankle. It includes the stance width, so the foot's own travel is ~0.2–0.3 m, which [6] (0.23 m beyond stance) checks. The stance width itself is OBP only or HDR's "bat length". |
| Peak pelvis speed | 581 [4] vs 705 [6] vs 714 [3] | Base [4] and [3]: use 580–710°/s; [6]'s 705 sits inside it. |
| Torso peak | 766 [4], 915 [6], 937 [3], 1100–1200 target [7] | Base [4] and [3]: 770–940°/s; [6] 915 is inside it; [7]'s target is a coach's. |
| Separation at plant | 18° [6,7] vs max 18° at −103 ms [4] (≈2° in coiling) | Follow [4]: ~2° at foot down, peak 18° at −103 ms, as the pelvis opens from −170 ms. [6] and [7] have 18° already at plant, so the pelvis starts opening earlier in machine-pitch swings; flagged, not used. |
| Attack angle | +4.1° (SD 5.9) [6] vs 8–10° [7] | Neither [3] nor [4] gives it: **OBP only** (+4°) and a coach's pro average (8–10°). Needs another source. The sim's swing model sets its own plane (agent-models §3.6), and the body follows it. |
| Contact leg and arm | OnBase "front knee should not flex more after plant"; [6] knee 45°→14° | Consistent: extend, never collapse. |
| Coil size | hips 28° / shoulders 52° at −350 / −265 ms (tee) [3]; −21° / −22° at −250 / −186 ms (live) [4]; ~8° at end of load [7] | Live pitching has the smaller, later coil: follow [4] and [7]. Welch's tee coil is the large end of the range, for hitters who coil a lot. |
| Hitting is "50/50" | [7] says unilateral (rear then lead); force plates [4,6] agree | Unilateral. |
| Pitch type and location change the swing | [4]: inside pitches open pelvis 79° vs 61° low-away; lower body "fairly consistent"; [7]: lanes | Base = middle pitch; the arms and bat vary, the legs barely do. |
| Feet width | HDR: = bat length; [6]: 0.68 m between ankles | Neither is in [3] or [4]: a coach's cue and an OBP-only number in the lab's frame. Choose by eye. |

**OBP only: needs another source or a value chosen by eye.** Stance width and foot angle (0.68 m; HDR: bat length); height of the leg lift (0.22 m ankle, ~83° knee); the centre of mass dropping 0.07 m and going back 0.12 m ([4] has the pelvis back 6 cm and rising 5 cm); vertical attack angle at contact (+4°); the rear-hip extension and heel-up pivot of the back leg; the pelvis already open at plant; the follow-through from +200 to +430 ms (torso +45° at +150, lead elbow re-bending to 73°); peak barrel speed by level.

A claim only one coach makes (and not used for numbers): feet = bat length and "launch position" as a hold-able posture (HDR [8]); "stripe of the pants" connection line (OnBase [2]).

## 4. What varies and what nearly all share

**Shared (the base).**
- The sequence and its speed ratios; pelvis opening while the torso resists at the plant; small separation; stiff, extending lead leg at contact; rear-hip drive; head still after plant; bent elbows at contact; bat below the hands at contact; torso side-bend toward the rear leg at contact; front-foot strike closed or straight, soft.
- The load starts before the pitch can be read and is the same whether or not he swings ([7]'s Cabrera clip: same load for a high fastball and a slow curve; the stride then differs).

**Varies by hitter (left for per-player stances).**
- Stance: width, foot angle, knee bend, crouch, hand and bat height and angle, how much the bat wiggles ([2]: "styles can differ widely"; [7] "no one perfect pattern").
- Load size: centre-of-mass sway back 0.08 m to 0.3 m+ ([7]); how high the leg lifts (a leg-kick against a toe tap) ([6]: lift SD 0.09 m); hand load and bat wrap.
- Coil and separation: torso coil 8–26°, separation at plant 9–28° ([6] p10–p90).
- Bat path: vertical attack angle (4–15°); out-to-in or in-to-out ([7]).
- Finish: one or two hands, how high, how far round. **Not measured anywhere in these sources**: take it from pictures later.

## 5. Gaps against what the game does now

Read: [player-visuals §9](../player-visuals.md), [agent-models §3.6](../agent-models.md), [physics-spec §6.3](../physics-spec.md), `BbMotion::Swing` in `client/Source/BaseballClient/BbPose.cpp` and `stance_pose` / `LOAD_S` in `sim/crates/bb_game/src/timeline.rs`.

What matches: the sim's swing is 0.150 s from start to contact (`T_swing`), and the sources put swing initiation (weight commitment, back elbow starts) at −170 ms and bat acceleration at about −93 ms. The hands-and-bat part is therefore right; what is missing is the **body before it**, and after it.

Most visible first. "Client" = pose and animation only. "Sim" = needs something the sim does not export now.

1. **The body has no coil, and the hips start with the hands (client, from data it already has; decided, owner Q42).** Hips and shoulders are driven by how far the hands have turned since the swing began, so the pelvis starts turning about −150 ms and nothing coils back first. In [4] the pelvis counter-turns to −21° at −250 ms and the upper trunk to −22° at −186 ms during the stride, the hips then open from about −170 ms (weight commitment), the pelvis is at 558°/s by −93 ms, and separation peaks at 18° at −103 ms; the pelvis ends ~98° round at contact. **Q42: the coil and the early hip turn happen on every pitch, like the stride, and are held up on a take**, so they give nothing away. The timeline already holds each pitch's contact time, so no new export is needed. To settle in the spec: how a held-up turn looks (the hips stop part-way and come back, with no bat), and how the times scale with pitch flight.
2. **The hands jump from the stance to the swing's first pose (client).** The stance puts the knob 0.40 m back and the swing's first pose is ~0.57 m back, and the load is `LOAD_S` = 0.05 s. That is about 0.17 m of hand travel in 50 ms (3.4 m/s), and the hands then start the swing from rest. The sources show a smooth load: hands move back and **separate from the chest through the whole stride**, the bat wraps behind the head at −243 ms (minimum azimuth −232°), the back elbow stays level with the hands, and the bat then unwinds. The load should be one continuous move over ~300–400 ms, the same on every pitch to a generic load pose (swing or take), ending where the swing starts.
3. **The follow-through is cut at 0.12 s (sim for the bat, client for the body).** The sim's bat track ends 0.12 s after contact. The sources need the bat to decelerate from ~2200°/s to ~800°/s over 200 ms, rise from −30° to +28° elevation, lag the hands, and the torso to turn ~60° more; the rear elbow straightens 50–100 ms after contact; then a two-handed wrap to a high finish. Since the bat after contact is cosmetic (physics-spec §6.3), the client can extrapolate a decelerating arc itself; a sim track continued to about +0.4 s would be cleaner and would keep "body follows bat". No source measures the finish, balance or how he leaves the box.
4. **Legs are posed by direction only, with no weight and no knee extension (client).** Missing: the lead knee **extends from ~48° at the weight commitment to ~18° at contact** [4] (15° [3]), a blocking front leg; the rear knee stays bent at 45–54° [3,4]; and weight goes from ~85% rear to the front (the lead foot takes 130% BW [4]). Today the pelvis dips and returns a few centimetres and the front knee only turns. The pelvis should go back ~6 cm in the stance, then forward 23 cm in all by contact [4].
5. **The stride is the wrong size and shape (client).** Front foot up as the ball is let go, down 16 cm nearer 0.17 s later. The base sources show a stride of 42% of body height toe-to-toe (about 0.8 m with the stance), landing ~11 cm closed, toe first and soft, with the foot down at −340 ms (scaled with the pitch) and the weight following by −170 ms (a "controlled fall"). The height of the leg lift is OBP only.
6. **Trunk posture does not swap forward bend for side bend (client).** At the plant he leans ~25° forward (the head slightly ahead of the belt) with the lean reversing to slightly backward at heel plant; at contact the trunk is side-bent ~20° toward the rear leg and 9° extended [3] (check: 28–32° [6,7]). The current chest sets roll −0.18 × round (about 10°) and a fixed pitch.
7. **Elbows should be bent at contact and straighten after it (client).** Both hands stay on the sim's handle (the bat rules), so the elbow IK must show the rear elbow ~80° and lead ~55° at contact, extending to straight about 50–100 ms later; check it is not already straight.
8. **Stance details (client, low confidence).** Ankle spacing is 0.54 m along the pitcher line here against 0.68 m in [6] (OBP only; choose by eye). The hands sit in front of the rear shoulder, which is plausible; sources give no hand height, bat angle or knee bend for the stance.
9. **Head and eyes (client).** Eyes are on the ball, good. Missing: head still after the plant, a small tilt toward the rear shoulder (−15 to −31° [4]), and turning back toward the field after contact.
10. **Check a late or checked swing.** The body follows the bat, so a checked swing stops it; once the hips start before the hands (gap 1), a swing taken late still needs the hips to be in the right place. Test with late and early swings.

## 6. Open questions

- **MLB-scale timing.** The load and stride timings come from 56–65 mph pitches. At 90+ mph the stride lead time and plant-to-contact time are shorter. No source here measures live 90 mph swings. How should the stride scale with the pitch's flight?
- **Stride length (settled).** [4]'s 42% of height is toe-to-toe at foot down, the same measure as [3]'s 85 cm (the thesis itself equates [3] to ~46% of height). It is the stance width plus the foot's travel, so it does not conflict with [6]'s 0.23 m beyond stance width. What remains open is the stance width itself.
- **Knee scales (narrowed).** [3] and [4] share one scale (0 = straight); [6] states flexion positive. The three agree at contact. The peak during the lift (52° vs 83°) still differs by method (event-aligned mean against per-swing maximum); for the lift height use [6] and a picture.
- **Contact point relative to the body and plate** is not given by any source; [6]'s HitTrax contact data exist but were not analysed. The sim's own contact point comes from the pitch.
- **Finish, one-hand release, wrist roll, balance and leaving the box**: nothing measured in the eight. Needs video of MLB hitters or another study.
- **Welch 1995 gaps.** Its foot-off and foot-down times, Table 3 (angles at the three events, so bat lag at contact) and the vertical force cells are not legible in the text; [3] gives no follow-through (it stops at contact).
- **Licence (decided, Q41).** [6] is non-commercial and share-alike, so it is used only as a cross-check and none of its numbers goes into shipped parameters. The "OBP only" list in §3 is what that leaves without a base source.
