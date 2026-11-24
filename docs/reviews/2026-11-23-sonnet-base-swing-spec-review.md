# Review: the base stance and swing design (player-visuals §13, commits 0b70855 and 0d44e99)

Reviewer: rev-swing-spec (Sonnet 5.5). Branch `work/base-swing-spec`. Scope: `docs/player-visuals.md` §13 against `docs/research/swing/swing-synthesis.md`, notes 3 and 4, the downloaded texts in `sim/target/research-dl/3` and `/4`, `client/Source/BaseballClient/BbDirector.cpp`, `BbPose.{h,cpp}`, `sim/crates/bb_game/src/timeline.rs`, `sim/crates/bb_agents/src/swing.rs`, `sim/crates/bb_ffi/src/lib.rs`, agent-models §3.6 and physics-spec §6.3. Nothing was built or run; this is a design, so every finding is about the text.

## Summary

The numbers are in good shape. Every base value I traced matches Fortenbaugh [4] or Welch [3], no value rests on OpenBiomechanics [6], and the by-eye values are marked. The scaling arithmetic (s = 0.714, foot off −0.07 s, foot down +0.13 s) is right. Every code claim I checked holds, including the 95/67/27/14% slowdown figures, `LOAD_S`, the 0.04 s brake and "the timeline is outside the digests".

The problems are in the timing model (§13.2), where the clock warp does not do what its text says, and in what Proposal 1 touches beyond one constant. D22 holds as designed: nothing before t_L can differ between a swing and a take.

- **P1 (1):** the warp from the pitch's clock to the swing's has wrong bounds and the wrong behaviour on late, early and very early swings.
- **P2 (4):** "late all over" contradicts [4] §4.3. The contact checks are tuned to the middle pitch only. Proposal 1 has client, FFI, saved-play and ordering consequences the text calls "none". The handover check cannot pass as written.
- **P3 (12):** table values and labels, inference precision, a build-order note and a few look questions.

No finding reopens a decision (D21–D23).

## Findings

### P1-1. The clock warp: bounds wrong, "only during the coil held" false (§13.2, §13.3, §13.4)

§13.2 says Δ = t_c − T_a is applied smoothly over W = max(0.10 s, |Δ|) from t_L, so "the body clock runs at most twice as fast or half as slow, and only during the coil held". §13.3 checks the body clock rate at 0.5–2 and the §13.7 list repeats it. With the body time b(t) = t − Δ·u(t) and u rising 0 → 1 over W, the rate is 1 − Δ/W. In the worked cases below, t_L = t_c − 0.20 (the usual case, t_s − 0.05; it is later if the load is cut). T_ref is 0.42 s.

1. **Late swings stall or reverse the body.**
   - Δ = +0.05 gives rate 0.5. Δ ≥ +0.10 gives W = Δ and rate 0, so the body freezes for 0.10 s or more.
   - Any smooth u (smoothstep, say) has a peak slope of 1.5, so the rate is 1 − 1.5Δ/W. At Δ = +0.10 it is −0.5: the hips un-turn.
   - The 0.5–2 check fails for every late swing over 50 ms.
2. **The window is not the coil hold.** The pitch clock puts the weight commitment at T_a − 0.17, so t_L = T_a + Δ − 0.20 is already at or past it for Δ ≥ +0.03. At Δ = +0.03 the window runs T_a − 0.17 to T_a − 0.07, which is the hips opening, not "the coil held". At Δ = +0.10 the body sits at hips +20° (peak separation) and waits there. That is not wrong to look at, because a hitter fooled by a changeup does fire early. It is not "waits longer coiled", and §13.1 and §13.4 should say what it is.
3. **The hips' first move lands late on early swings.** The swing's clock puts the commitment (hips' first move) at t_c − 0.17, 30 ms after t_L. The body reaches the swing's clock only at t_L + W, so it is behind by about 0.7·|Δ| at the commitment.
   - Worked case, a changeup swung 40 ms early (T_a = 0.50, t_c = 0.46, t_L = 0.26): the body runs at 1.4×. The commitment (body time 0.33) comes at real time 0.31, but the swing clock says 0.29.
   - The sim's hands start at t_s = t_c − 0.15 = 0.31, so the hips start with the hands, not 20 ms before them. The "hips go before the hands" look (Q42) is lost on early swings, the ones where the hitter is fooled.
4. **W can end after contact.** t_L + W = t_c − 0.20 + W, so for |Δ| > 0.20 the body is still off the swing's clock at contact. A slow curve (T_a = 0.55) swung 0.25 s early has t_L = 0.10, before foot down (0.13), and the whole swing-clock schedule asks the body for events before the foot is down. Event order survives (one time axis), but the soft landing is compressed to half.
5. **No distribution of Δ is measured.** Step 1 measures T_ref and the spread of start poses but not the distribution of Δ and of t_L − t_decide on the sim's own swings. These decide how much of 1–4 matters.

*What the text should say.*
- Define b(t) = t − s(t) with s linear (or any s whose slope stays in [0.5, 2] − 1). That gives W ≥ 2Δ for late swings and W ≥ |Δ| for early ones.
- State the cap where t_L + W would pass t_c − 0.03. Beyond that cap the body stays off by the residual and the chest correction (±15°) takes the rest.
- Drop "only during the coil held".
- Say what a late swing shows (hips already open, held mid-turn) and what an early swing shows (the hips' first move at the hands' start, not before).
- Add Δ to the step-1 measurements.
- Move the swing clock and anchor window out of step 1 into step 5, where late and early swings are checked (see P3-12).

### P2-1. "A late swing is late all over and an early one early all over" contradicts [4] §4.3 (§13.1 last paragraph, §13.4, §13.2 item 3)

The synthesis cites [4] for "stance, stride and coil the same against fastballs and changeups", which is right. [4] §4.3, Tables 40 and 41 (`sim/target/research-dl/4/s.txt` lines 6198–6245), shows the later phases differ the other way. Timing is relative to contact, fast hit / changeup hit / changeup missed:

| Event | Fast hit | Changeup hit | Changeup miss |
| :-- | :-- | :-- | :-- |
| Pelvis peak | −71 ms | −88 ms | −109 ms |
| Lead-knee extension peak | −30 ms | −47 ms | −67 ms |
| Bat azimuth peak | −10 ms | −16 ms | −24 ms |
| Trail elbow extension peak | +16 ms | +10 ms | −8 ms |
| Trail elbow at contact | 81° | 67° | 51° |

The discussion of the same text (`s.txt` ~7462) says the hitters "prematurely initiate the events of the kinetic chain since hitters are taught to anticipate a fastball". A fooled hitter is early in the legs and hips and late in the hands.

The design instead times the whole body from the swing's bat, "late all over". It matches [4] only if a changeup counts as "late" on the arms alone. This decides what the late and early swings look like, so it needs the owner's attention only in the sense of "does the base follow [4] here". The text should either state the departure and why, or keep the hips and legs on the pitch's clock plus a fraction κ of Δ ([E], set from Table 41) and let the chest correction and the elbows take the rest. The second option would also shrink P1-1, since the warp would then move only the arms.

The same table breaks the §13.7 elbow range for fooled swings (see P2-2).

### P2-2. The contact checks and the ±15° cap are tuned to the middle fastball only (§13.7, §13.4, §13.3)

§13.7 fails a run when, at contact, the back elbow is outside 66–95°, the front outside 45–70°, the hips outside 60–90°, or the chest's correction exceeds 15° on more than 5% of frames. Those are [4]'s values for middle fastballs.

- The sim's contact varies much more than that. Agent-models §3.6 gives ρ from 0.15·H to 0.46·H, with partial reach g = 0.6, so far or inside aims are met with hands well off the middle-middle radius. That is a hand-to-chest distance varying by a factor of about 3 (0.28–0.85 m on a 1.84 m figure).
- An elbow's flexion at contact is the shoulder-to-hand distance. On a far aim it is nowhere near 81°, and the body reaches the rest of the way with `FBbReach` (24 cm).
- [4] shows the spread even among its own swings: back elbow 66 to 89° by location (Tables 1–3) and 51 to 81° by pitch type (Table 40).
- The ±15° chest correction cannot supply the extra reach, so on far and inside pitches the 95% rule and the elbow ranges will fail in correct output.

*What the text should say.* Make the contact numeric checks conditional (middle-ish aims only, say ρ within ±15% of ρ_ref), and give the expected elbow flexion as a function of ρ for the rest. Alternatively say that the base shows [4]'s location table (hips 61–79°, elbows 66–89°) and let the chest correction follow, with the cap at 15° only for the middle band. §13.9 already defers per-location values to a later step; the checks should not include them before then.

### P2-3. Proposal 1: more than "one constant", and an ordering rule (§13.5, §13.6)

"Digests unchanged" is right. The event-log digest does not cover `timeline::encode`, `bb_game/examples/digest.rs` hashes the game, and the ledger's 11-16 row for `stance_pose` is the precedent. Invariants 4 and 5 are untouched: the bat after contact is cosmetic (physics-spec §6.3), and a shorter deterministic track is still deterministic. The text calls the rest "None" (schema, FFI) and "client must hand over". What else reads the track after +0.03:

1. **The FFI hold.** `bb_ffi/src/lib.rs:560-571` returns the last sample until `last.t + 0.25` and then `has_bat = 0`. Today the bat shows to +0.37; after Proposal 1 it is gone at +0.28. A finish held to +0.45 (§13.1) therefore needs the client to draw the bat on its own from +0.28 on. The §13.5 table does not mention it.
2. **`EndT` has many users.** `NoteBat` sets `EndT = LastMove` (`BbDirector.cpp:1724`), the last time the bat moved. It drives `bBatSim` and `bSwinging` (4110, 4119), the return to the shoulder (3370–3378, `BatReturnS` 0.8 s), the ease at `EndT + 0.3` (4123) and the motion strips' "finish" and "back" marks (3945). All of them move 0.09 s earlier.
3. **Ordering.** If Proposal 1 is merged before step 3, the present client starts `BatReturnS` at +0.03 from a bat at 95% of its speed and snaps it back toward his shoulder. Proposal 1 must land with step 3, or after it. §13.5's "(B) can be built first and switched to (C) with one constant" is true for the client side, but the sim change alone breaks the game's look.
4. **Saved plays keep their old tracks.** `BbSavePlay.cpp:306-336` writes a `.timeline` file with its md5, so the owner's saved plays (and `check-fair-signals.sh` and `-BbSavedPlayCheck`) carry the +0.12 track forever. `BatHandoffS` in §13.3 is written as "+0.03 (Proposal 1); +0.12 as now": the client must read the end of the track it has, not a constant. State that, and test step 3 on an old saved play and a new play.
5. **Stale text in the tree.** `swing_pose_probe.rs` (doc comment and the `sw.t_c + 0.12` sample), the `timeline.rs` comment ("through the follow-through") and data-formats §5.3 (line 350) all name +0.12.
6. **Contact or nominal contact.** The actual contact can be up to 25 ms after the nominal one. "Contact (or nominal contact) + 0.03 s" should say which; if actual contact is used, the end is always ≥ 0.03 s after it. The slowdown figures (95/67/27/14%) are at T_swing 0.150 only; they scale with the batter's T_swing, and a 0.130 batter's bat is down to about 7% at +0.12.

I checked the slowdown figures by hand from the bump law (`swing.rs:237-239`, width 0.055 s, peak at +0.010 s): ratios to the contact rate are 0.95, 0.67, 0.27, 0.14 at +0.03, +0.06, +0.10, +0.12. The "about 160°" past contact at +0.12 is 168–185° depending on k.

### P2-4. The handover continuity check cannot pass as written (§13.7)

"At the follow-through handover the bat's position and direction continuous (step under 2 cm and 2° a frame)". At +0.03 the bat turns at about 0.95 × 2233 = 2100°/s and the knob moves at about 6 m/s. That is about 35° and 10 cm per frame at 60 fps, so no continuous motion meets 2° and 2 cm a frame. The same applies to the swing's own samples: `bb_ffi` returns the previous 5 ms sample without interpolating, so the drawn bat jumps up to 10° at a time on its own.

*What the text should say.* Measure continuity against the expected motion: position within 2 cm of the extrapolation of the last 10 ms of the sim's track, direction within 2°, and the turning rate within 5% of the last sim rate, then non-increasing. The client should also interpolate the sim's 5 ms samples at least from t_L on, or the check will see the sample steps.

### P3-1. Head tilt at the commitment is mistimed (§13.3, angles)

The table has "Head tilt toward the back shoulder: commitment / contact 15 / 31". In [4] §4.1.4 (`s.txt` ~3298) the head is laterally flexed −9° to −15° *during* swing initiation (commitment to max force, −170 to −93 ms), and −15° to −31° during the acceleration phase. So 15° is the value at τ −0.093, and the commitment value is about 9°. The row should read: commitment 9, τ −0.093 15, contact 31.

### P3-2. Back knee at contact is a midpoint labelled as a source (§13.3)

"Back knee ... contact 50, source [4,3]". [3] gives 45°, [4] about 54°. 50 is neither; it should be marked "midpoint, by eye".

### P3-3. "+0.17 from the stance" for the hips' travel matches no sum (§13.3, distances)

Hips' travel: −0.06 (settle), +0.07 (stride), +0.11 (to commitment), +0.05 (to τ −0.093). From the stance start the net at the commitment is −0.06 + 0.07 + 0.11 = 0.12; from the settled position it is 0.18; and [4]'s total from the start of the stride to contact is 0.07 + 0.11 + 0.05 = 0.23 m. Neither 0.17 nor the way the row mixes them is explained. Pick one and state its origin.

### P3-4. Several "[4]" labels are derived or by-eye values (§13.3)

- The front foot's travel (0.12 H, 0.22 m) is the stride 0.42 H minus the by-eye stance width 0.30 H, not a [4] value. It changes if the width does.
- "Trunk bent forward: foot down 21" cites [3]'s foot-off value (the synthesis also has ~25° at landing from [7]).
- "Stride direction 8°" is atan(0.11/0.77), derived; the source ([3]) says 12°.

Mark each "derived" or "by eye" in the Source column, as D21 asks.

### P3-5. The slow-pitch load drifts instead of loading and holding (§13.2, §13.3)

The hands' load (L at C − 0.052) and the deepest coils are tied to the pitch's clock, so on a 0.55 s curve L is reached at R + 0.33 s, 0.26 s after foot down (0.07 s on the reference). The synthesis §4 says the load is the same for a high fastball and a slow curve ([7]'s clip) and only the stride then differs. Consider anchoring L on the pitcher's clock with a hold, and keeping the pitch's clock for the commitment and after.

### P3-6. Inference precision (§13.4, §13.6)

- **`t_s`.** The timeline already has a `SwingStart` marker at exactly `sw.t_start()` (`timeline.rs`, markers block; data-formats lines 350 and 383). The FFI does not expose it. A read-only export (like `contact_s`) is exact and cheaper than inferring the end of the load's straight line. §13.6 names a Q21 export as the fallback; I would make it the first choice.
- **`t_c`.** "Fastest turning less 0.010 s" has two errors. The 5 ms grid gives ±5 ms if read as an argmax, so it needs a fit over the samples to reach 2 ms (the peak is flat: at ±15 ms the rate is 96% of the peak). And the offset is `bat_peak × T_swing/0.150`, 8.7–11.6 ms across the batter range, an error of up to 1.6 ms from the fixed 0.010.
- **Checked swings.** `t_s + 0.150` is off by up to 24 ms for T_swing 0.130 or 0.174, enough to shift the body's clock visibly. The brake time and the bat's rate at the brake give T_swing exactly.
- **`t0 <= 0`.** `timeline.rs` clears the whole bat track when a swing starts at or before the release, so no t_L or t_s exists. This probably never happens (the swing waits on the decision), but the text should say the body then stays on the pitch's clock.

### P3-7. T_a on a ball hit first (§13.2)

For a ball hit before the plane y = 0.35 m (contact depth is 0.35 + 0.125·x_in, so up to ~3 cm in front of it), T_a is extrapolated from a track that ends at contact. The error is about 1 ms, which is nothing visible, but the D22 test in §13.7 ("swing withheld") reuses the same timeline and cannot see it. Step 1 should compare T_a from the truncated track with T_a from the full track on swings that miss, expecting ≤ 1 ms.

### P3-8. A take's foot: stays or eases back? (§13.1, §13.4)

§13.1 says the front foot stays where it landed "until he resets". §13.4 and `TakeRelaxS` ease "back to the stance pose over 0.6 s". If the stance pose includes the back foot position and the front foot, the planted foot slides. Say which parts relax and that the feet do not move until the reset.

### P3-9. The leg lift is specified twice (§13.1, §13.3)

The front knee at the top of the lift is 52° ([4], a floor) and the ankle height is 0.08 m (by eye). With the back knee at 56° and level hips, a 52° front knee leaves the foot about 1.4 cm below the back foot's level. The 8 cm needs hip flexion that is not in any key. §13.4 says the knees are solved from hips' height and tilt; say which of the two wins and what the hip does.

### P3-10. The body-to-bat gap differs from [4] by 20 ms (§13.3)

[4]'s bat azimuth rate peaks at −10 ms; the sim's bat rate peaks at +10 ms (agent-models §3.6 τ_b). With the body keyed to nominal contact like [4]'s, the hips-to-bat gap is 80 ms against [4]'s 61 ms. The spec calls timing differences of this size invisible; I agree, but the "Order of peaks" check hides it, so say so in the text. Also, for T_swing = 0.174 the sim's hands start 4 ms before the commitment, so for the slowest hitters the hips no longer lead.

### P3-11. Would a real hitter do this? (question 5)

- **The take that starts the hips and stops** (D22, owner Q42). Hips go from −13° to about 0° with the chest held at −22°, a separation of about 22°, inside the 18–22° of [4], [6] and [7]. That is what a hitter who "holds up" does, so it passes. Without a decision input the hold never depends on pitch location; that is what D22 gives up on purpose.
- **Leg lift 0.08 m and stance width 0.30 H (0.55 m).** Both are by eye and per-player knobs. 0.08 m is a small lift (a toe tap to a short kick) and 0.55 m ankle to ankle is "a little wider than the shoulders" for a 1.84 m man. Neither looks wrong, and both are looks-only (D23).
- **The ±15° chest correction.** Plausible for a hitter's trunk; see P2-2 for where it runs out.
- **The finish** (bat high over the front shoulder, two hands, held 0.15 s) is by eye, flagged as such, and is the least certain part. No source measures it.

### P3-12. Build steps and independence of the checks (question 6)

**§13.7 independence.** The checks that test something outside the keys are:
- the elbows at contact (an outcome of the hands and arm lengths);
- the speed peaks and their order (the keys are angles, so the peaks come out of the curve);
- "inferred times against the sim's records" (an independent comparison against `sw.t_start()` and `sw.t_c`);
- the D22 comparison (a property of the pose function);
- hands on the handle (a reach test, not a key).

The checks that restate the keys are the contact ranges for the front knee (10–25° around the key 18°), the back knee, the hips (60–90° around 77°) and the chest minus hips (±8° around −2°), plus "foot down ≥ 0.20 s before contact" on the reference pitch (the key is 0.29 s). Their value is as an integration test of the clock and the solver, not as a test of the research. I would keep them, say so, and add the independent ones: [6]'s check values (allowed under D21 as checks only: front knee 14° at contact, separation 21°, back-elbow 79°) with a wide tolerance, and the T_a extrapolation check of P3-7. `-BbSwingPairShots` compares two different pitches within 0.01 s of flight by eye; add the numeric comparison of the pose at identical play time, which D22 allows.

**§13.8 build steps.**
- **Step 1 is not checkable alone.** It adds the body clock, legs and stride while the sim's hands still jump from the stance to the swing's start in 0.05 s (3.4 m/s, `LOAD_S`). Until step 4, the arms must reach from a body that has strided 0.22 m forward to hands 0.4 m back, and the "hands ≤ 1.5 m/s" and "chest correction under 15°" checks cannot pass. Either move step 4 (the hands' load and handover) ahead of 2 and 3, or say these checks begin at step 4.
- **The three clocks and the anchor window** (step 1) are exercised only by late and early swings, which are step 5's pictures. Split: step 1 gets the pitcher's and the pitch's clock; step 5 gets the swing's clock and the warp.
- **Step 3 and Proposal 1** need each other (P2-3.3). Step 3 should be able to build against the old track first (read the track's end, do not hard-code +0.12 or +0.03).
- **D22 check** can run from step 2 (before t_L the bat is the stance), but it needs the t_L inference from step 1.

## Verification

**1. Numbers against the research.**
- Read §13.1, the three §13.3 tables and §13.9 against `swing-synthesis.md` line by line, then the ten that matter most against note 4 and its source text (`sim/target/research-dl/4/s.txt`).
  - Event times: foot off −621, foot down −340, commitment −170, max force −93 (4.1; grep counts 2 each).
  - Coil: hips −21° at −250 ms, chest −22° at −186 ms.
  - Knees: lead 36→48→52 (−483 ms), back 51→56→50→44 (−220 ms) (lines 3131–3204).
  - Peak speeds 581 and 766°/s (3 hits each), separation 18° at −103, bat 2435°/s at −10.
  - Contact: pelvis 77°, trunk 75°, back elbow 81°, front 56°, lead knee 18°.
  - Foot angle 69/63/56/49/36, head 47°.
- Welch [3] abstract and text: 714 and 937°/s, 31 m/s, 123% BW, 85 cm stride, centre of pressure 20 cm behind the centre of mass, knee 32° at foot off (`welch-full.txt`).
- All base values match their sources except the six labelling issues in P3-1 to P3-4. No value in the base tables is [6]-only: every [6] number appears in a "check" or "OBP only" column. The values chosen by eye (stance width, lift height, finish and its angles, stance hands, `TakeShowS`, `TakeRelaxS`, trunk stance and contact angles) are marked as such.
- [4]'s own inconsistency (hips turn 57° in the last 93 ms, 613°/s on average, against a stated peak of 581°/s) is real and §13.9 already notes it.

**2. The timing model.**
- Recomputed s = (0.42 − 0.17)/(0.52 − 0.17) = 0.714.
- Scaled events from the commitment at 0.25 s: foot off 0.25 − 0.451 × 0.714 = −0.072; top of lift +0.026; foot down +0.129; deepest hips C − 0.057; hands C − 0.052; chest C − 0.011; back knee C − 0.036. All match §13.3. Foot down is 0.29 s before contact on the reference pitch; [6] and [2] put it at 0.17–0.20 s on slower machine pitches, so it is earlier, not too late.
- D22: t_L = max(t_s − 0.05, t_decide) (`timeline.rs:278`), so the warp starts at or after the decision and nothing before t_L can differ between a swing and a take. The pitch's clock depends on the ball track alone.
- Walked through the seven cases by hand, with P1-1's formulas:
  - fastball on time (Δ = 0);
  - changeup 40 ms early, which loses the hips' lead (P1-1.3);
  - swing 30 ms late, where t_L is at the commitment and the window is the hips' opening, not the coil hold (P1-1.2);
  - take, where the hips stop at about 0° (plausible);
  - checked swing, with t_c off by up to 24 ms (P3-6);
  - 0.55 s curve, where nothing breaks but the load drifts (P3-5) and 0.25 s early puts t_L before foot down (P1-1.4);
  - 100 mph, where order holds, the clamp matters only below T_a ≈ 0.34 s, and a 40 ms early swing compresses the landing.

**3. Claims about the code.**
- `LOAD_S` = 0.05 s and `stance_pose` (knob 0.40 m back, 0.10 m toward the plate, 1.30 m up): `timeline.rs:31, 40-42` ✓.
- The track ends at `sw.t_c + 0.12` (line 269); `t_load = (t0 − LOAD_S).max(rec.t_decide)` (278); samples every 5 ms; the first load sample equals the stance, so the bat's first movement is at t_load ✓.
- `T_swing`: `ratio_from_grade(grade, 0.150, 1.05)` gives 0.130–0.174 s at grades 80–20 (1.05³) ✓. `BRAKE_S` = 0.040 s ✓. A check swing starts only before 0.45 × T_swing of the swing (`abort_time_frac`) ✓.
- `PlateCross` is written at a fixed 0.42 s (`timeline.rs:307`) ✓. `CheckSwingBrake` is in `timeline.fbs` and never written ✓. A `SwingStart` marker is written; the spec does not mention it (P3-6).
- Slowdown figures 95/67/27/14%: recomputed from the bump law, all four ✓ (see P2-3.6).
- "Timeline outside the event log and digests": `digest.rs` hashes `simulate_game`; `encode` is called only by `bb_ffi` and tests; ledger 11-16 is the precedent ✓.
- Reading `BbDirector.cpp:1659-1727` (`NoteBat`): t_L = `StartT` is exact to the sampling (the load's first sample is the stance itself), `EndT = LastMove`. The checked-swing brake is the last movement less 0.04 s ✓. t_c and t_s: see P3-6.

**4. Proposal 1.** Readers of the track after +0.03: `bb_ffi` sampler (hold and `has_bat`), `NoteBat` / `EndT` users, saved-play `.timeline` files, `swing_pose_probe.rs` (reads `sw.at`, not the track, but names +0.12), and the doc/comment text listed in P2-3. The `bb_ffi` tests sample the bat only at 0 s, up to the decision, and 0.30–0.60 s (contact). "Digests unchanged" holds; invariants 4 and 5 are untouched.

**5. Would it look like a real hitter?** Judged from the numbers and the keys, not from pictures (nothing is built): P3-11.

**6. Checks and steps:** P3-12.

**Not done:** no build, test or run (per the brief). Figures and Appendix E of [4] were not inspected. I did not check the side of the hips' tilt or the signs of [4]'s conventions beyond the foot angle and the head tilt.

## Resolution

Resolved 2026-11-23 by the spec's author (worker swing-spec) on `work/base-swing-spec`, under the Coordinator's rulings. All in `docs/player-visuals.md` §13; the TASKS batting-stances boxes follow the new build order. No code, no ledger row.

- **P1-1 — fixed (§13.2, §13.3, §13.7, §13.8), together with P2-1.** The warp now moves only the body (legs, hips, weight, chest) from A = T_a to a target A\* and is defined by its rate: r ramps over 0.02 s from its value at t_L to a constant r\* and back to 1, with r\* = [t_L + W − A\* − τ_b(t_L) − (r₀ + 1)R/2]/(W − R). W is the shortest window ≥ 0.06 s giving 0.5 ≤ r\* ≤ 2 and never longer than W_max = t_c − 0.03 − t_L (≥ 0.10 s since t_s ≤ t_c − 0.130). Bound: r moves only between r₀, r\* and 1, so 0 ≤ r ≤ 2 always and 0.5 ≤ r ≤ 2 unless restarting from a hold-up; never negative (nothing un-turns); starts at t_L (D22); ends by t_c − 0.03. Where capped (late Δ above ~0.075 s, early targets above ~0.15 s) the residual and what is seen are stated. "Only during the coil held" is gone; §13.1 now says what a little late, late (a hitch where a take's hold-up had begun) and very late swings show. The early-swing loss of the hips' lead (1.3) is answered by P2-1's κ: the hips now lead more on early swings, not less. The swing's clock moved to step 5; Δ and t_s − t_decide are step-1 measurements.
- **P2-1 — fixed as ruled ([4] §4.3).** Early swings: A\* = T_a + κΔ with κ = 2.7 from Table 41 (pelvis 17/38 ms early against bat 6/14 ms: 2.8 and 2.7; lead knee 2.8 and 2.6), the body's lead over the bat capped at 0.025 s ([4]'s largest group). Note against the ruling's wording: the measured factor is larger than one, not a fraction, because the hips are early by nearly three times the bat; that is what [4] shows and the spec says so. Late swings: κ = 1 (body late with the bat), by reasoning, as no source measures them. Arms follow the bat; the chest's correction and the elbows take the difference. §13.1 has plain descriptions of a hitter out in front of a changeup and one late on a fastball.
- **P2-2 — fixed (§13.7, §13.3, §13.4).** Contact checks split: middle fastballs on time (the old ranges, the 95% correction rule); other locations against [4]'s location means (hips ±12°, back elbow ±15°); off-speed and early swings with Table 40's elbow range, reported not failed. The correction cap is ±15° on middle aims, ±25° elsewhere. I used the pitch's location from `BbPlayInfo` rather than ρ (the client does not know ρ; location is what [4] tabulates).
- **P2-3 — fixed (§13.5, §13.6, §13.8); Proposal 1 is now Q43.** Listed: the FFI hold and `has_bat` (the follow-through draws the bat regardless), every `EndT` user, the order rule (the sim change with or after step 4, never before), saved plays (the client reads the track's end, never a constant; step 4 tested on an old saved play and a new game), the stale text to fix in the same commit, actual vs nominal contact (0.03 s after the later of the two), and the slowdown's dependence on `T_swing`. "About 160°" corrected to 170–185°.
- **P2-4 — fixed (§13.4, §13.7).** Continuity is measured against the motion extrapolated from the last 10 ms (2 cm, 2°, rate within 5% and then non-increasing); the client draws the sim's bat interpolated between its 5 ms samples (in the FFI under E1, or by the client).
- **P3-1 — fixed.** Head tilt 9 / 15 / 31 at commitment / τ −0.093 / contact (checked against [4] §4.1.4 text).
- **P3-2 — fixed.** Back knee at contact labelled "midpoint, by eye".
- **P3-3 — fixed.** The row now gives positions from the stance (−0.06 / +0.01 / +0.12 / +0.17) and states [4]'s increments and the 0.23 m total from the start of the stride. The old +0.17 was right but unexplained.
- **P3-4 — fixed.** Front foot's travel "derived (changes with the width)"; trunk at foot down 23° by eye between [3]'s 21° (foot off) and [7]'s ~25°; stride direction "derived", with [3]'s 12° beside it.
- **P3-5 — fixed (§13.2, §13.3).** The deepest coils and L are on the pitcher's clock with a hold (no later than their commitment-relative times); the commitment and after stay on the pitch's clock.
- **P3-6 — fixed (§13.6, §13.2).** Export E1 is the first choice: `swing_start_s` from the existing `SwingStart` marker, `swing_contact_s` as one appended telemetry field (−1 in old saved plays), and the bat interpolated in `bb_timeline_sample`; cost stated (header, ABI 2 → 3, schema field, FFI test, digests unchanged, one ledger line) and why Q21 allows it (t_c is not in the timeline). Raised for the Coordinator, no owner question. The fallback inference is corrected (a fit for t_c with a `T_swing`-scaled offset; a checked swing's t_c from its brake time and rate). The `t0 <= 0` case: the body stays on the pitch's clock and the bat the client's, logged.
- **P3-7 — fixed.** Step 1 compares T_a from a track cut at contact with T_a from the full track on misses (≤ 1 ms).
- **P3-8 — fixed (§13.1, §13.4).** On a take the hips, trunk, knees and hands relax; the feet do not move until the reset.
- **P3-9 — fixed (§13.4, §13.3).** The foot's height wins; the front thigh is raised 15–25° (by eye) and the knee bends as much as that leaves (about 60–75°); [4]'s 52° is a floor, not a key. Named as the first thing to look at in the leg step's pictures (step 2, since the order changed).
- **P3-10 — fixed (§13.2).** The 20 ms body-to-bat gap is stated and accepted. The slowest hitters' lost hip lead is fixed rather than noted: the body's keys between the commitment and contact are spread over `T_swing` + 0.02 s, so the hips move 0.02 s before the hands for every batter.
- **P3-11 — no change needed** beyond P3-9 (the reviewer finds the take, the lift, the width and the correction plausible; the finish stays by eye and is called the least certain part in §13.9).
- **P3-12 — fixed (§13.7, §13.8, TASKS).** New order: 1 the bat before the swing (load, handover, interpolation, E1) with the measurements; 2 legs and stride; 3 coil, hip turn and take; 4 follow-through (Q43 with or after it, built against today's track first); 5 the swing's clock with early, late and checked swings (strips 9–11, pair shots). §13.7 now says which checks test the design and which are integration tests round the keys, and adds [6]'s check values with wide tolerances. The numeric D22 comparison at identical play time was already in §13.7; kept.
