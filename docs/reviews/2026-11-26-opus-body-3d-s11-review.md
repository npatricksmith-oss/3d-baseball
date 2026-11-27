# Review: session 11 of the force-driven body (work/body-3d, `4948b07..7024f5c`)

Reviewer: Opus 5.5 (high effort), 2026-11-26. I wrote none of this. Brief: `REVIEW-body-3d-s11.md`. Every figure of mine below comes from a run whose output is under `sim/target/review-probes/s11/` (Verification). I edited no source, test, spec, PLAN or TASKS file.

## Summary

What was built holds up. The read friction and the hold carried by muscle are what the spec says. The energy account closes on a kept swing. The five checks pass. The friction values read from papers are what the papers print (I had five of them opened). The hips' correction to the owner is right. The friction dependence holds up when searched from S at the high end (P3-6).

Three of the account's claims are not supported by the runs, and all three bear on what the owner decides next:

1. **"All of it came from his feet" (P1-1).** Rung 3 changed two things: the feet, and where the ball is met, which moved 21.5 cm nearer his hips. That puts it outside the aim's own stated range and away from PLAN D28/D29's figure. I ran each change without the other, twice, by the ladder's protocol. **Either change alone gives the whole gain.** The wide base with the ball where it was: 21.8 and 21.2 m/s. The ball moved deeper on [4]'s narrow base: 19.7 and 21.3 m/s, with the best score any search has reached. Both changes together, rung 3 itself: 20.7 and 21.5. Every body with both [4]'s feet and the old ball place: 14.7–17.7. The gain is real. It cannot be credited to the feet. Half of the rung changed the task (where the ball is met relative to the body), not the body, and that place is an owner's decision.
2. **"His hips do not turn" (P1-2).** In every swing on the wide base, both hip joints turn through their whole range of turn by their own muscles. They reach their ends 30–60 ms before contact and stay there, loaded at 256–920 N·m. The pelvis barely turns in the world for a different reason: in the same 120 ms both thighs and both feet turn the other way, 30–47°, the feet spinning toe-back on the dirt. Meanwhile the ground carries only 0.35–0.65 of his weight. The account's "three suspects, none tested" leaves out what the runs show.
3. **"He is not short of muscle; the gap is not a strength gap at all" (P1-3).** The what-ifs show that this body cannot turn more strength into bat speed while its hips sit on their ends and its feet spin. With 1.5 times the strength, the extra goes into the front hip's end (764–920 N·m against 570–601). That answers D34's question for this base and this launch state. It does not support "no strength gap at all", and "three to five times" disagrees with the session's own numbers.

The rest is P2 and P3: the "both over both" rule is a weak test; the front foot's 2.35 times his weight is an early spike, not a block; and smaller wording.

**Waiting on the owner (for the Coordinator to frame):** where the ball is met relative to the batter's hips. Rung 3 moved it from D28/D29's footing to Statcast's foot-relative intercept. Half of the session's gain is reached by that move alone.

## Findings

### P1-1. Rung 3's gain cannot be credited to the feet: on [4]'s narrow base, moving only where the ball is met gives the same gain

*Claimed* (§4.4: "All of it came from his feet"; "the gain comes from standing where Statcast measures major leaguers standing"; Status: "every bit of it from standing where major leaguers stand"; §4.13 rung 3: "PLAN D28 and D29 are in the plate's frame and are not touched"; "the ball's place follows the feet"; "[4]'s feet and the ball's place in front of them were costing this body about 4 m/s").

*Found: the rung is two changes.* Measured on the two bodies' starts (`geom.py`): on rung 2's body the ball is met 0.720 m toward the pitcher from the pelvis and 0.152 m below it. On rung 3's it is 0.505 m and 0.097 m below: **21.5 cm deeper and 5.5 cm higher relative to the hips**. Relative to the rear ankle it hardly moved (1.037 → 1.022 m). `MEET_AHEAD` is 0.72 m with a stated range of [E: 0.61–0.84]; 0.505 is outside it. Nothing else in the aim changed. The pitch's line, its speed, the score's terms and the arms' clock (253 ms before contact, from the pitch) are the same. In this model the ball has no time of arrival, so a deeper ball does not change the time allowed. (In the world, a ball met 14.5 cm further on arrives about 4 ms later.)

*Found: each change alone gives the whole gain.* I ran each half of the rung alone, by the ladder's protocol, from S with both of its seed sets (`q1.py`; jobs 500–503; every result settled, within 1 cm of the ball, the same at three steps to 0.12 m/s):

| Body | Seeds 1, 11 (score) | Seeds 2, 12 (score) | Contact after the start |
| :-- | --: | --: | --: |
| rung 2: [4]'s feet, the ball 0.72 m ahead of the hips | 16.85 m/s (33.1) | 16.58 (33.8) | 330, 340 ms |
| **A: MLB's feet, the ball where it was relative to the hips (0.72 m)** | **21.80** (29.9) | **21.15** (26.6) | 290, 298 ms |
| **B: [4]'s feet, the ball where rung 3 puts it (0.505 m)** | **19.69** (29.0) | **21.26** (25.6) | 285, 288 ms |
| rung 3: both | 20.66 (28.1) | 21.53 (25.2) | 260, 271 ms |

- By the ladder's own reading (the better-scoring search): A 21.15, B 21.26, rung 3 21.53, against rung 2's 16.85. **Each half alone is worth what the whole rung is worth.** They do not add; whatever the two share, the gain does not need both. Every search of a body with [4]'s feet *and* the ball at 0.72 m (rungs 0, 1, 2, 2-alone: eight searches) lies at 14.7–17.7 m/s. Every search with either change lies at 19.7–21.8.
- B's second search has the lowest score of any search in the session (25.6; rung 3's best is 25.2 with both changes).
- So the session's honest gain is real: it is far outside the spread, and every one of these eight searches beats every search without the changes. But it is **not shown to come from his feet**. Moving the contact point 21.5 cm nearer his hips on the lab study's feet gives the same 21 m/s. The contact point is part of the task, not of the body.

*Is the deeper ball right?* Possibly. PLAN D28 and D29 put the hips 0.71 m behind the plate's front edge and the ball 0.07 m in front of it, 0.78 m apart. D28 itself says it is not established whether Savant's hip figure is at the stance or at contact. Statcast's own column has the intercept 31.1 in (79 cm) ahead of the hips *in the stance*, one second before release (note 15 A; the helper confirmed the definition). The front toe moves a median of about 20 in from stance to intercept (my helper's count from the page's data), and the hips move with the stride. So 0.505 m from the hips at launch may well be consistent with Statcast once the hips' travel is counted. Nobody has shown it. D28/D29's "not touched" is true only in the plate's frame; relative to the batter, where the ball is met moved outside the range written for it.

*Change.* In §4.4, Status, §4.13's table and §13: rung 3 is two changes, and either alone gives the gain. Stop crediting the gain to the feet ("all of it from his feet", "standing where major leaguers stand"). Write the split table. Say the ball is now met 21.5 cm nearer his hips, outside `MEET_AHEAD`'s [E 0.61–0.84]. Where the ball is met relative to the batter is the subject of the owner's D28/D29. The Coordinator should put it to him: the hips-to-ball distance from D28/D29's 0.78 m (stance) against Statcast's foot-relative intercept (0.505 m from the hips at launch), with this result. Until he rules, the 21 m/s rests on either the wide base or the deeper contact point, and neither alone is established as the cause.

### P1-2. The hips turn fully and jam at their ends; the pelvis stays still because the legs and feet turn backward on a lightly loaded base. "His hips do not turn" misreads the runs.

*Claimed* (§4.4 "What the batter does now" and "What is not known"; §4.13 rung 3 and rung 4; Status): "his hips do not turn"; "His hips are not turning for some other reason, and a swing that did turn them would meet these ends" (rung 4); three suspects, none tested: the launch posture, the search, D39.

*Found.* I replayed `practice3d_s11_rung4.json` and read the hips' turn stops every 10 ms (`hipstops.py`, `hipstops_practice3d_s11_rung4.out`). The front hip goes from −28.6° at the start to −39° in the load at −211 ms. It then turns in through 71° in 150 ms, about 700°/s relative to the thigh. It reaches its end of turn in (32°) at −61 ms, carrying 196 N·m there. The rear hip reaches its end of turn out (44°) at −31 ms. Both stay on their ends to contact, at 27–119 N·m, and with peaks of 570 and 856 N·m before contact (the stored result). In the world the pelvis turns +11°. Both thighs turn −30° and −44°, and both feet −46° and −42°: they turn the other way, toe back. The same holds in every swing I read on the wide base: rung 3's kept swing, the from-S searches at 1.5 times strength (`r6_all15_b`), and from the stored results of r3, r3_b, r4, r4_b, r6_all15 and r6_all20_w. In each, both hips' turn stops carry 256–920 N·m before contact, and both feet turn 23–67° toe back. Rung 2's swings, on [4]'s feet, show the same pattern, less of it.

The replay also shows how little the feet are carrying while this happens. On rung 4's kept swing the ground carries 0.65 of his weight at −100 ms, 0.59 at −50, 0.43 at −20 and 0.35 at contact. The front foot carries 0.04 of his weight at contact (practice3d's own line, `show_r4.out`). A foot that light cannot hold a twist. I have not shown that the light load is *why* the feet spin. The runs show the two together, in every swing.

So the hips' joints are not idle: they use all of their range by their own muscles and then lock. The pelvis does not face the pitcher because the legs under it turn backward. Rung 4's range change was used: the rear hip runs to its new 44° end. It bought nothing because the legs, not the hip joints, give way. The account's suspects (the posture, the search, D39) may all play a part. But the runs already point at the feet's hold on the dirt, the base's load in the last 100 ms, and the hips' ends. Status item 6 lists the feet's turning under "Smaller, in no order".

*Change.* In §4.4, §4.13 (rungs 3, 4, 6) and Status: say that the hip joints turn through their full range and sit on their ends from about −60 ms. Say that the pelvis stays because both legs and feet turn 30–47° the other way, while the ground carries a third to two thirds of his weight. Move "the feet turn on the dirt" out of "Smaller" and into the first items. Next session's first single change could be a diagnosis, not a search: which of the feet's grip (the measured 1.15 is for sliding; the twisting moment a cleat holds was not read), the base's load, and the hips' ends holds the pelvis. Rung 3's "the front foot being 14° more open did not make the hips turn" should say the hips did turn and the foot turned back.

### P1-3. Rung 6 does not show "he is not short of muscle, the gap is not a strength gap at all"; and "three to five times" disagrees with its own numbers.

*Claimed* (§4.4: "Is he short of muscle? No … what he lacks is a swing that uses the strength he has"; §4.13 rung 6: "He is not short of muscle. As far as these runs show, the gap to the gate is not a strength gap at all"; "the 7.9 m/s he is short would take three to five times the strength").

*Found.*
- **The scaling reaches what it should.** `Dr.BOUND_SCALE` is in force while `bounds()` and `add_drives()` run (`practice3d.Ctx.__init__`). So each group's bound, its tendon spring (`ksec = bd/SEC_STRETCH`), the holding split (`self.bd` passed to `solve_hold_muscles`), the burst's cap (`bd − hold_T`) and the effort charge (normalised by the scaled bound) all scale. The strength-at-speed curves are fractions of the bound, so they scale with it. Only the hold-only trunk-bending groups (`HOLD_ONLY`) are not scaled; they cannot fire, so this is immaterial. The effort charge does not penalise the extra strength: the same torque costs less effort on a stronger group.
- **What the runs show.** On the wide base with [4]'s launch state, more strength from S finds the same kind of swing (pelvis −1° and −5°). The extra strength goes into the front hip's end of turn: 920 and 764 N·m with every muscle at 1.5 times, against 601 and 570 at 1.0 (stored results). With the hips on their ends and the feet turning back (P1-2), more strength cannot turn the pelvis. That is a fact about this base and launch state, not about his muscles. It answers D34's question ("which muscles and by how much") as: *on this body as it stands, no amount inside what a man has would close the gap, because the swing is held by the hips' ends and the feet*. That is worth telling the owner. "Not a strength gap at all" is not: a swing whose legs hold the ground and whose pelvis turns 60° may well need more of some muscle. §4.13's own "What this does not show" says so; §4.4 drops it.
- **"Three to five times" is not what the runs give.** A straight line through the session's own figures needs about 8 times (carried on: +1.12 at 2.0), about 4.8 times (from S: +1.05 at 1.5) or about 11 times (carried on: +0.40 at 1.5). Only the from-S rate stays under five, and nothing gives three. Any straight line through gains of 0.4–1.1 m/s, carried out to 7.9, means nothing anyway.
- *Smaller:* §4.4 says "None moved his bat by more than 1 mph (the forearms and wrists most, +1.1 mph)". 0.48 m/s is 1.07 mph, so the sentence contradicts itself.

*Change.* Replace the answer with what the runs show. With the hips on their ends and the feet turning back, no strength scaling up to 2.0 times moves the bat more than 1.1 m/s, so strength is not what holds *this* swing. Whether a swing that turns the pelvis would need more is not shown. Delete "three to five times". Fix "under 1 mph".

### P2-1. "Both searches over both of the rung before's" is a weak test, and two claimed gains rest on it alone.

*Claimed* (§4.13's table header and the protocol): a gain counts if it is larger than 1.95 m/s *or* if both of a rung's searches beat both of the rung before's.

*Found.* With two searches a side and no real effect, the chance that both new ones beat both old ones is 1 in 6. Rung 6's "+1.05 m/s for half as much again of every muscle" rests on that rule alone (22.01, 22.16 against 20.89, 21.11). Rung 2 on rung 1's body also passes it, and clears 1.95 by only 0.18. Rung 3 (+3.8 to +4.7) and rungs 1 and 5 (losses of 2.5–3.0) are well outside either test and stand. The within-rung differences across the session are 1.95, 0.91, 0.13, 0.27, 0.87, 0.22, 0.45 and 0.15 m/s. 1.95 is the largest, so the threshold is cautious, but the "or" lets weaker evidence through.

*Change.* Call a "both over both" result *suggestive*, and keep "gain" for results that clear the yardstick. §4.4's "+2.3 mph searching from scratch" for every muscle at 1.5 times should read "suggestive".

### P2-2. The front foot's 2.35 times his weight is a spike 70 ms after launch, not a block; at contact the front foot carries 0.04 of his weight.

*Claimed* (§4.4: "He lands on his front foot and puts 2.3 times his weight through it"; the gate row "the front foot's greatest load" passes; rung 3: "He lands on his front foot and loads it … no swing before reached 1.6").

*Found.* On rung 4's kept swing the peak is at −201 ms, 70 ms after the start (practice3d's own line). In the stored results the peak falls at −178 to −201 ms in r3, r3_b, r4 and r4_b. Only `r6_all15_b` peaks late (−62 ms). At contact the front foot carries 0.04–0.23 of his weight and the whole ground 0.35–0.67. The start carries [4]'s velocities (moving forward and down) onto a base 26 cm wider. The early spike looks like the body landing on the front leg, not a hitter's brace, which loads the front leg late in the swing. The gate row reads only the peak's size (`ladder3d.gate`).

*Change.* Report the peak's time and the front foot's load at contact beside the size, in the gate row and in §4.4. Do not count the row as passed on an early spike without saying so.

### P3-1. Source readings: right, with three small corrections (question 3)

A helper opened the full texts. My brief to it was to quote, not to judge.
- **Lipps 2020** (PMC7670493, Table 2's β0, at rest): 0.61, 0.33, 0.46 N·m·s/rad, glenohumeral, passive. Right. The ± figures are 95% confidence intervals, not SDs.
- **Lee, Krebs & Hogan 2014** (PMC4696764, Table IV): 0.34/0.45 bend and 0.15/0.18 roll, relaxed. Right. The brackets are SEs.
- **Casabona 2012** (J Appl Physiol 113:1747): 0.18 ± 0.08 and 0.31 ± 0.09 for controls, derived as 2ζωJ from the pendulum's decay. The numbers are right. The paper quotes them as the half-swings where the group difference was smallest and largest, not as the controls' range.
- **Salmond 2017**: the quoted range and mean are right, and they are a summary of four earlier studies.
- **Tsuji 1995** (Table 3): elbow 0.223–0.626, shoulder 0.421–1.054, joint viscosity from hand viscosity, the arm "asked to relax" while holding the hand's place, reflexes in. Right, as note 14 says.
- **Moorhouse & Granata 2005**: 27.5 ± 9.0 is trunk *flexion–extension* (not twist), holding load, reflexes in. The paper gives no lever for that conversion. Its own S1–T10 distance (0.203 m) would give about 12.9. So the trunk range's top (27.5) is a sagittal, active value that may be about twice what the paper's own lever gives. The range's reason ("a trunk holding a load, which a relaxed one cannot exceed") is still defensible as a ceiling. Say "flexion–extension, lever not stated".
- Every unread value is flagged as not measured in `drive.py`, §4.13 and note 14. The relaxed-not-active choice and dropping the 30 N·m cap are each stated with a reason, and the integrator was checked at the low, central and high sets (`damp_integ.out`). The account of rung 1 is right: the bat peaks at 20.4 and 19.7 m/s 20–30 ms before contact, and in each search one elbow reaches its end before contact, the lead in one (321 N·m) and the rear in the other (665 N·m), per the stored results.

### P3-2. The hips' correction to the owner is right (question 6)

Simoneau 1998 (JOSPT full text, Table 3, men, n = 21, active, "maximum effort"): prone (hip near 0° flexion) turning in 32 ± 9°, out 44 ± 7°; seated 30 ± 7° and 35 ± 8°. Boone & Azen's 44.4/44.2 are active and seated. Roaas & Andersson, Simoneau and a NASA report all say so; the paper itself is paywalled. Ellenbecker 2007's 23/35° are the pitchers' throwing-side hips, prone, active. Claim (d) stands as written. One nuance: rung 4 changed both ends of each hip, and the swings use the new turn-out end. The front hip also uses it in the load (−39° on rung 4's swing, past the old −33.6°).

### P3-3. The Statcast reading cannot be rechecked from the repository

Note 15 says the foot points were recomputed by "`s11/` scratch script". No such script or data file is under `sim/target/codex-probes/body3d/s11/` (the `stance_*` files there are the start's rule). The page does carry the points the note describes. A helper read the page's embedded data: big toe, small toe and heel of each foot, in feet, at the stance, at release and at the intercept (fields suffixed 0, 1, 2). It found that the stance's foot separation is big toe to big toe. The figures the body stands on (1.03 m, 44°, 1.022 m) rest on the worker's construction of an ankle joint from those points. That construction is stated in note 15 but not kept as code. *Change:* commit the script (or keep it under `s11/`) with the date the page was read.

### P3-4. Tests (question 8)

- `test_a_damper_takes_a_spinning_rods_speed_as_the_worked_decay`: independent, a worked exponential and energy. Good.
- `test_a_joints_two_dampings_act_on_its_two_movements`: independent, geometric. Good.
- `test_the_posture_held_by_muscle_…`: the knee curve's points come from §4.5's printed curve, and the starting accelerations are compared across two code paths. Good. Part (5), the effort formula, restates the code.
- `test_the_damping_table_is_read_as_written`: mostly restates `DAMPING`'s literals. The structural checks (read joints the same in every set, low ≤ high) are worth keeping.
- `StanceChecks`: the placement test checks the placed ankles and foot angle against the given figures, which is a test of `pose()`, not of Statcast. `Pc − ankle_R = 1.022` restates the constant; the "within 1.2 cm of the lead ankle" is the real check. The balance and spread tests are independent (statics, a worked lever rule, a solve compared at ±30 N). The hips' test checks `range_specs` with `ROM` patched. The runs use `add_ranges(rom=…)`, which I checked separately: the kept rung 4 result's limits are (−44, 32) and (−32, 44).
- `probe3d --check`'s two new cases do exercise the read damping (by axis, no cap) and the holding drives, at the search's 0.25 ms step, over 200 ms. They do not exercise a holding drive letting go, because `add_hold_muscles` sets no switch-off. The switch-off is set by `practice3d.Ctx.prepare`, and the compiled engine's switch-off is an existing feature.
- No test covers rung 6's `bound_scale` reaching the tendon springs and caps; I checked that by reading the code (P1-3).

### P3-5. Smaller wording

- §4.12 (lines 1818 and 1982) still says "for the last session" and "What is left for the last session". These sit in session 10's dated section, so they are history, but D42 lifted the limit. A one-word "(then)" would do.
- `drive.py`'s and note 14's "0.61 ± 0.22" (Lipps) are 95% CIs, and Lee's are SEs. Say so where a ± is quoted.
- Rung 5's list of what is still [4]'s (question 7) is complete against §4.10's table: posture numbers, velocities, the feet's loads and level forces, the hold and the plan S. The only thing not on it is the launch instant itself, [4]'s −250 ms foot plant, which is a definition, not a number of the body.

### P3-6. The friction dependence stands, checked from S (question 5)

The dependence runs were single searches carried on from rung 4's swing, which lean toward "no change" (the rung 6 reference returned exactly 21.11). So I searched rung 4's body with every unread value at its high end (hips 4.6, trunk 27.5, elbow 0.97, forearm 0.10, wrist's tilt 0.20) from S, by the ladder's protocol with rung 4's better seeds (2, 12; `q5.py`, job 505). It gives **20.73 m/s (46.4 mph)**, 0.1 cm from the ball, settled (0.20), the same at three steps (20.78, 20.73, 20.72), score 26.7. Rung 4's own search with the same draws gave 21.11 at the middle values, and the carried-on run at the high end gave 20.28. So the high end costs 0.4 m/s from S, inside the carried-on reading's −0.83. "His bat speed does not hinge on these numbers" stands for the high end. The low end was not rechecked. One wording fix: "2.8 mph" is from single carried-on searches; say so.

## Questions as asked

1. *Rung 3:* two changes, and either alone gives the whole gain (P1-1). Nothing else in the pitch, aim or score changed. Statcast's figures exist as the note describes them (the intercept-instant foot points are in the page's data). The worker's derived figures cannot be recomputed from the repository (P3-3). Handedness and frame in `body.py` are right: a right-handed batter on the −X side, the lead foot's 44° measured from +X toward +Y, the lead ankle level across. The rest of the start is §4.12's rule run on the given feet. No stop moved. The rule stopped at 4.0–4.7° margins (it asks 5) at the ankles' roll and the hips' sideways range, as stated.
2. *Rung 6:* P1-3.
3. *Friction:* P3-1.
4. *The hold:* no constant torque remains beyond what is reported: ankles' roll, the rear hip's turn the other way and 4 N·m at the rear shoulder (`tone_left`), +0.5 J on rung 4's kept swing. The holding drives follow the strength-at-speed law (tested against §4.5's printed points), are on from the start (a pre-tensed level, no rise), are charged as effort with the burst, take their share of the bound, let go at the plan's switch-off, and are not read by the clock. At the first instant the accelerations are identical to the constant torques' (test, 1e-9 relative). Neither was in equilibrium; [4]'s moving start never was. Energy on rung 4's kept swing (my replay): muscles +483.3, holding +32.2, constant +0.5, damping −192.3, stops −28.7, ground −62.4, gravity +58.7, kinetic energy +292.1; the sum closes to −0.87 J. This matches §4.13's table to the joule.
5. *The noise:* P2-1, P3-6. Rung 3's gain is outside the spread on every fair comparison: both its searches beat all four of rungs 0 and 2, by 3.0–5.0 m/s (its cause is P1-1's question).
6. *The hips' range:* P3-2. Right.
7. *Rung 5's list:* complete (P3-5).
8. *Tests:* P3-4.
9. *The account:* P1-1, P1-2, P1-3, P2-2. §4.4 does list the failing gate rows, the joints past range and the 13 joint ends. Stale lines: P3-5. "Hold-still torques" are no longer described as present, and the old damping appears only as history.

## Verification

All under `sim/target/review-probes/s11/`:

- **The five checks** (job 504, after touching every changed source): `chk_test_body3d.out`: 80 tests, OK. `chk_test_oracle.out`: 112, OK (8 expected failures). `chk_built3d.out`: PASS. `chk_probe3d.out`: PASS, worst 3.6e-11 on the plain cases; the two new cases 6.8e-11 and 4.6e-13. `chk_docs.out`: two findings, both in the untracked review brief `REVIEW-body-3d-s11.md` (a path to this file before it existed, and a bare filename). The committed docs are clean.
- **Rung 4's kept swing re-read** (`ladder3d.py --show … --verify`): `show_r4.out`, `show_r4.json`. 21.11 m/s at 0.5, 0.25 and 0.125 ms; energy as in question 4.
- **The geometry of the two starts**: `geom.py` (output in this review, P1-1).
- **The hips' stops and the segments' turn through a swing**: `hipstops.py`; `hipstops_practice3d_s11_rung4.out`, `hipstops_practice3d_s11_rung3.out`, `hipstops_r6_all15_b.out`, `hipstops_r2.out`. The stored results' hip-stop loads and feet are read from `sim/target/codex-probes/body3d/s11/*.json` (P1-2).
- **Rung 3 split** (`q1.py`, jobs 500–503, the ladder's protocol from S): `r3_oldball_s1.out/.json`, `r3_oldball_s2…`, `r2_deepball_s1…`, `r2_deepball_s2…`.
- **The high friction from S** (`q5.py`, job 505): `r4_high_fromS_s2.out/.json`: 20.73 m/s.
- **Sources** were opened by two helper agents: the five friction papers and Moorhouse (P3-1); Simoneau, Boone & Azen through Roaas, Ellenbecker, and the Savant page and its data (P3-2, P3-3).

## Not checked

- Note 14's other sources (Huang 2020, Amankwah, van der Kooij, Hodges via Karayannis, Cholewicki, the remaining wrist papers), and Welch 1995's foot-plant figures.
- The worker's Statcast numbers recomputed from the page's data: the helper confirmed the fields exist, not the means.
- The chain's steps (jobs 440–450, 471) and rung 5's searches, beyond their stored summaries.
- Whether the feet's twist on the dirt is a fault of the contact model (a cleat's twisting moment) or real for this load. P1-2 shows the turning; it does not show its cause.
- The rung 6 one-group runs beyond their stored summaries.

## Resolution

Open.
