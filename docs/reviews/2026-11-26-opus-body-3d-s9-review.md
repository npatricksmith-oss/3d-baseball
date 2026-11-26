# Review: phase 1, session 9 of the force-driven body (the hands, a muscle that holds, the feet)

- **Reviewer:** rev-body-3d-9, Claude Opus 5.5, high effort. I wrote none of the change.
- **Range:** `main..HEAD` on `work/body-3d`, eleven commits, `5933e15` to `1bd190f`.
- **Edited:** nothing but this file. No source, test, spec, note or TASKS file. The session's `body3d_probe` binary was not rebuilt: after touching the sources I set `body3d_probe.rs`'s time back to its commit time so `probe3d.py --check` would use the existing binary, and the binary's time is still 01:01:22.
- **Probes and their output:** `sim/target/codex-probes/review-body-3d-9/`. Every run went through the job queue (jobs 378–388), one at a time.

Terms used below:

- An **end** or **stop** is the spring that holds a joint at the end of its range.
- The **score** is what the practice search lowers: the sum of the squared terms in `practice3d.residuals`.
- A **one-number difference** is a run with one of the plan's 205 free numbers moved by 2% of its range. The search builds its slopes from these runs.
- A **damped step** is the Levenberg–Marquardt step the search tries from those slopes.
- The **clock term** is the score's charge for an arm muscle firing earlier than the batter could have decided.
- **S** is start S's result on part 3's body (`analysis/research/body/practice3d_s9_cleats_S.json`, the "nearest motion"), unless I say otherwise.

## Verdict

**As far as my runs show, the search is the first cause of "no swing", partly through a defect in its score; the start is the second cause. Nothing shows the body is the cause, and I found no defect in what was built.**

- **The score has a defect.** The clock term charges only the earliest arm onset. Every session-9 result has five to eight arm muscles tied at the very first instant of the run. Moving any one of them later delays contact, so the clock term mostly rises (by up to 37). The search therefore reads every onset as uphill. Moving them all 10 ms later together lowers S's score from 153.4 to 139.5 at once (−9%, against the 40 fallback iterations' 178.1 → 153.4). At the 0.20 s viewing time it lowers the score by 205. The search cannot see that direction.
- **The score is too rough for the search's differences.** Along each of the eight damped steps the search would try from S, the score rises within 1/1000 to 1/100 of the step, while the slope model predicts a fall. The slope of one number measured forward and backward differs in sign or by up to 10 times. Part of this roughness comes from the 0.5 ms integration step: the same 0.03 ms change of one onset costs nine times less at 0.125 ms.
- **The start is jammed.** With no muscle working at all, the lead arm reaches its turned-in end 27 ms into the run. Under S's plan four ends carry load within 46 ms. The first of them is the lead hip's sideways end (at 28 ms, 142 N·m), which the account does not list. The rear wrist starts in the corner of its box, bent back and tilted away from the thumb together, a combination a real wrist does not reach.
- **What was built holds up.**
  - The new stops, the ankles' ends and the raise's end make no energy. I checked this on the whole body with every stop elastic and every parent moving, a case the worker's tests do not cover.
  - The two engines agree.
  - The new readings are a clinician's.
  - Session 8's motion does leave the new ranges. Its lead upper arm turns in to 148° against a 69° end.

**"Whether this body can swing at all is unknown" is the right statement.** Its reasons should name the clock defect and the search's step first. The cheapest decisive test is in "What is left" below.

## Findings

### P1-1. The clock term cannot be seen by the search, and it is what stopped the 0.20 s runs

- **Where:** `practice3d.py`, `residuals`: `r.append(max(0.0, (tx - self.t_avail) - min(on)) / CLOCK_UNIT ...)`, and `arm_onsets`.
- **What is wrong.** The residual is the overrun of the *earliest* arm onset. With two or more arm muscles tied at the earliest onset, moving any one of them leaves `min(on)` where it was, and the only effect is a later contact (`tx`). So every one-number difference sees the clock term rise. The search's linear model then predicts that moving the onsets together raises the term by the sum of those rises. In fact moving them together lowers it, because `min(on)` moves. Every session-9 plan starts at that tie: the box's lower bound for an onset is the run's first instant, and plan S has its arm muscles there.
- **How I know.** `clock.py` on the 0.20 s result and on S (`clock.out`):

| | 0.20 s result (score 2,231.8, clock 1,730.8) | S (score 153.4, clock 22.0) |
| :-- | :-- | :-- |
| arm muscles within 3 ms of the earliest onset (the run's first instant) | 8 | 5 |
| each moved 2% alone: change in the clock term | −0.5 to +37.0 (seven of eight up) | −0.1 to +1.8 (four of five up) |
| every arm onset moved to at least 5, 10, 20 ms after the start, together: change in the score | −105, −165, **−205.5** | −13.1, **−13.9**, +42.5 |

- **Consequences for what is reported:**
  - "At 0.20 s the search did not move off a plan that breaks the rule" (§4.11, §4.4) is a fact about this score, not about the viewing time or the body.
  - The column "Part 2: watching 0.20 s" says nothing about how long the batter watches. The account half-says this; it does not give the cause.
  - "The plan's arm muscles still fire at the first instant" (part 2, 0.10 s) has the same cause.
- **This is not new to session 9.** Session 8's clock had the same form, and there a tensed arm muscle counted as firing at the first instant, so ties were certain. Part 2 was built to make the clock mean something, and this form of the term undoes that.
- **Fix.**
  - Charge each arm muscle its own overrun: a sum over arm channels of `max(0, (tx − t_avail) − onset_n)`, weighted by its strength share. A soft minimum would also do.
  - Or let the search move all arm onsets as one extra number.
  - Rerun the 0.10 s and 0.20 s comparisons after the fix, before anything is said about the viewing time.

### P1-2. The search stalls because the score is rough at the scale of its differences, and part of that roughness is the 0.5 ms step

- **Where:** `practice3d.lm` (one-number differences at 0.02 of each range, which is 6 ms of onset; damped steps capped at 0.25), and `H_STEP` = 0.5 ms. Account: §4.11 "Not shown", item 1.
- **What is wrong.** The account says the search stalls on "a score made rough by stiff ends and a foot that sticks and slips". That is untested, and it is only part of the story.
- **How I know.** `line.py` and `jump.py` at S (`line_cleats.out`, `jump_cleats.out`):
  - **Along every damped step the search would try** (λ from 0.01 to 164), the score at 1/1000, 3/1000 and 1/100 of the step is higher than at S by +0.17 to +9.7. The slope model predicts falls of 0.01 to 1.3. At λ = 41 and 164 the first 1/1000 falls by 0.001–0.004, and the score then jumps by +0.7 by 1/100. **The slope model is wrong at every length tried, not only at long steps.**
  - **Measured forward and backward**, the slope of the core's onset is +3,568 and −14,090 at a 2% difference: the score rises both ways, so S sits in a kink. At 0.5% it is +918 and −1,176; at 0.125% it is +865 and −918. The lead knee extensor's onset gives +9,875 forward and −920 backward. None of these converges as the difference shrinks.
  - **The core's onset moved by 1e-4 of its range (0.03 ms)** raises the score by 0.73 at the 0.5 ms step and by 0.084 at 0.125 ms. The "off the ball's line" and "miss" terms carry the jump. Some of the roughness is therefore the integrator's step, not the body.
  - **Ten of the 205 one-number differences at S are lower than S** (the best 153.375 against 153.437). The fallback step takes these, which is why it gains a little each iteration and gets nowhere.
- **Consequence.** "The search stalls" is shown, and so is why: a Levenberg–Marquardt search on finite differences is the wrong tool for this score at this step. That is a fault in the search, not a finding about the body.
- **Fix (stated before any run, as the job's rules ask):**
  - Run the search's evaluations at 0.25 ms or 0.125 ms. At 0.125 ms the run costs four times as much and the jump is nine times smaller.
  - Use a search that does not need slopes, for example CMA-ES, or random-direction steps with the score evaluated at 0.125 ms.
  - At the least, test every accepted step at both step sizes.

### P1-3. The rear wrist starts in the corner of its box, where a wrist does not go; "both wrists start inside a wrist's range" holds only for the box

- **Where:** §4.11 part 1, "As built", the start table and "Both wrists start inside a wrist's range"; §4.4 "For the owner": "The back hand's impossible starting position is gone."
- **What is wrong.** At the start the rear wrist is bent back 73.8° of its 75° end and tilted away from the thumb 29.7° of its 32.4° end, both at once.
  - [Note 11](../research/body/11-hands-and-joint-ends.md) A1 (Crisco 2011; Gehrmann 2008) has a wrist's reach as an ellipse, not a box. The ellipse is tilted about 27° along the dart-thrower's line, so its narrowest reach is toward extension with ulnar tilt, exactly this corner.
  - Take the plainest reading: the ellipse that fits inside the box, with no tilt. The rear wrist's start is 1.33 times the ellipse's radius along its own direction, so 33% beyond the ellipse's edge. With the measured tilt it is further out.
  - §4.11 said the box would stay "until a run shows a wrist leaning on a corner of the box". The start itself does.
  - Hitters' rear wrist at foot plant (OpenBiomechanics, in the same table) is bent back 52 ± 14° and tilted 5 ± 12° *toward* the thumb, well inside.
- **How I know.** `start.py` (`start_cleats.out`, `start_hands.out`) reads the start's angles. The ellipse ratio is computed from the box's centre and half-ranges: bending −5.6 ± 69.4°, tilt −5.2 ± 27.2°.
- **What the other rear-arm posture does** (`start_alt.py`, `start_alt.out`). With the elbow clearance at 0.08 m or 0.06 m the rule picks the low rear arm. The rear wrist is then near neutral in bending (−0.9° and +8.2°) and still tilted 28–30° away from the thumb; that is inside the ellipse (0.90 and 0.85 of its radius). But the rear forearm sits 5.5° and 12° from its palm-down end, and the wrist is 50–60° less bent back than hitters'. **0.08 m is inside ARM_CLEAR's own stated range (0.08–0.12)**, so the start's rear arm flips between two quite different postures within its own [E].
- **Fix.**
  - Say in §4.11 and §4.4 that the rear wrist starts where only the box allows.
  - Build the oblique wrist range (a stop that reads both angles) or a corner test, before the next search uses this start.
  - Report the start's sensitivity to ARM_CLEAR across its stated range.

### P1-4. What the owner is told in §4.4 overstates or misplaces four things

Each sentence that needs it, corrected, is in "For the owner" below. In short:

1. **"The search is a careful step-by-step improver and it stalls on this body."** It stalls because of P1-1 and P1-2, which are the search's faults. As written, the sentence reads as a property of the body.
2. **"How long he must watch… With 0.20 s the search did not move off a plan that breaks the rule."** The cause is the clock term's form (P1-1). Nothing about the viewing time has been run in a way that could show anything.
3. **"Whether his back foot needs to come up on its toe. It never does in any run."** True, but the reason is missing, and the reason is the plan, not the foot.
   - S uses 0.8 N·m of the rear calf's 231 (0.3%); part 1's result uses 32.
   - With the rear calf switched on at its full strength from 100 or 150 ms (`feet.py`, `feet.out`), the rear heel rises 3.1–6.8 cm on part 3's body, 2.1–4.7 cm on the weld, and 2.2–10 cm on part 1's. From the first instant it rises 11–27 cm.
   - Nothing but the plan keeps the heel down, and the worker's argument that the rigid plate already "comes up onto its toe" is right.
   - When the heel does come up the rear foot turns further backward (to −11° to −48°), because the rear hip's rotators are at full strength and turn the leg backward.
4. **"On cleats it stops sliding backward (30° before, 4° now) and his hips open 58° instead of 35°."** The 30° and 35° are session 8's swing on the weld, a different body and plan. On this session's bodies the rear foot slid −28° (part 1), **−8° (part 2, same flat feet as part 1)** and −4° (part 3, cleats); the pelvis turned 34°, **46°** and 58°. Most of the change came with part 2's different plan, before the cleats. The cleats' own share is −8° → −4° and 46° → 58°, in searches that had not settled and moved more than the feet.

### P2-1. The start leans on more ends than the account lists; the lead arm's strain is [4]'s posture, not the grip's [E]s

- **Where:** §4.11 "Not shown", item 2; §4.4 "What is not known"; "Status and what is left", item 2.
- **What the runs show** (`start.py`, `start_alt.py`):
  - **The ends nearest at the start** (degrees from the end): the lead upper arm's turn in 1.1; the rear wrist's bend back 1.2; the lead arm's sweep across the chest 2.0 (a stop placed by session 8's rule 2° past the start; the clinical end is 0.4° away); the rear wrist's tilt 2.7; **the lead hip's sideways end 2.8**; the lead ankle's roll 4.1 (part 3's body).
  - **With no muscle working** (gravity and the hold-still torques only), the lead upper arm's turn end engages at 27 ms, the rear wrist's at 55 ms, the lead sweep at 62 ms and the lead hip's turn at 65 ms (116 N·m).
    - The hold-still torques do not hold the start still: after their solve, 7–10 m/s² and 25 rad/s² of acceleration remain somewhere in the body, and the centre of mass moves 67 mm in 100 ms.
    - This is **not new**: the weld's body has 7.5 and 24.6.
    - It does mean the start drifts into its ends before any muscle acts.
  - **Under S's plan**, the lead hip's sideways end engages at 28 ms (142 N·m), the rear wrist's tilt at 39 ms, the lead sweep at 44 ms (110 N·m) and the lead arm's turn at 46 ms (60 N·m). On part 1's body the lead hip's end engages at 15 ms with 302 N·m.
  - **What frees the lead arm.** Every grip variant leaves the lead arm where it is: 1.1° from the turn-in end and 134.6° across the chest, at obliqueness 10°, 15° and 20° and at clearance 0.06–0.12 m. The lead arm's strain comes from [4]'s trunk and elbow, not from the hands.
    - [4]'s trunk turned 10° the other way (`trunk_rot` −10) brings it to 5.9° inside, with the rear wrist 9.7° inside.
    - The lead elbow bent 15° more brings it to 5.6° inside.
    - Each of the opposite changes puts the lead arm 3.9° and 5.5° past its end.
- **How much of "no swing" the start alone explains is not shown by my runs.** No search ran from a freed start.
- **Fix.**
  - List the lead hip's end in the start's strain.
  - Say that the lead arm's strain is [4]'s posture, unaffected by the grip's [E]s.
  - Before the next search, move the start off the ends by a stated rule (for example the least change to [4]'s trunk turn and lead elbow that puts every end at least 5° away). That is a modelling choice, not the owner's.

### P2-2. The guard on the upper arm's turn can fail again exactly when a swing gets fast

- **Where:** `body.py`, the `shoulder_up_` stop; §4.11 "A fault in part 1's new stop".
- **What is wrong.** The raise's end (168.8°, Soucie's forward raise) is an anatomical end, not a patch. But it is the only thing between a follow-through and a failed run.
  - The turn's reading fails within 5° of 180°, which leaves 6.2° past the end.
  - With the large joints' stiffness (20,000 N·m/rad) the end takes about 2,200 N·m to cross. S already puts 899 N·m into it (2.6° deep).
  - Loads in a follow-through grow roughly with the bat's speed squared, so at 29 m/s, the gate's speed, about 2,100 N·m is to be expected (an estimate).
  - The failure happens after the 50 ms the score reads, so a swing approaching the gate's speed would turn into refused steps again, from a part of the run nobody looks at.
  - The same end is used in every direction, although the sideways raise is 184° (Boone & Azen); sideways it is 15° tighter than anatomy.
- **Fix.** Stop each run, or stop failing it, 50 ms after contact, because nothing later is read: run to `tx + AFTER`, or have the probe return its records up to the failure. Keep the raise's end; give it its direction-dependent value or state the 15° as [E].

### P2-3. The "rig" check of the cleats is set by the stud spacing's [E], so it is not an independent check

- **Where:** §4.11 part 3, "The forefoot in Brosnan's rig gives 68.4 N·m"; `FeetChecks.test_the_forefoots_twisting_resistance_is_a_cleats_on_dirt`; the commit message "Tested in Brosnan's rig (68.4 N m against the measured 68.5–100.1)".
- **What is wrong.** The test asserts the engine's peak equals Coulomb's two-point value, 1.15 × 1,190 N × `BALL_HALF_W`, and that it is at least 0.97 × 68.5.
  - `BALL_HALF_W` is an [E] (4.5–5.5 cm). Across that range the same formula gives 61.6–75.3 N·m, and 68.5 needs 5.006 cm. So 5 cm is the number that lands at the low end of the measurement.
  - What the test does check independently is that the contact law gives Coulomb's value and takes energy. That is worth having, but it is not agreement with Brosnan.
  - The source's numbers check out: [note 12](../research/body/12-trunk-bending-and-feet.md) D has Brosnan's 68.5–100.1 N·m on a basepath mix at 1,190 N through 40°, and traction coefficients of 1.04–1.27 static and 1.02–1.20 dynamic, so the grip of 1.15 is inside both.
  - The old plate's "38.1, 56% of the least measured" is right.
- **Fix.** Write it as "the stud spacing is set inside its range so that the forefoot gives the low end of Brosnan's measurement", and keep the test as a check of the contact law.

### P2-4. "Session 8's swing cannot be made with real hands": shown for the motion, not for a 40 mph swing; the forearms' part is weaker than written

- **Where:** §4.11 "Shown", second item; §4.4 "What the batter does now".
- **What the runs show** (`s8arms.py`, `s8arms.out`). Session 8's motion, read with session 9's readings, starts where §4.11 says (forearms 12° and 46° palm-down, upper arms 69° and 20° turned in) and then:
  - the lead upper arm turns in to 148° by contact (end 69°);
  - the rear upper arm turns out to 129° (end 104°);
  - the lead forearm turns palm-down to 80° (end 77°);
  - the rear forearm stays inside its range to contact.
- **So the motion is impossible with real ranges** (a kinematic fact, not one replay), with the upper arms by 25–79° and the lead forearm by 3°.
- **"Forearms… turning further than a body's do" holds only for the lead, by 3°.**
- **That no 40 mph hit exists on the new body is not shown,** because of P1-1 and P1-2.
- **Fix.** Use the corrected sentence below.

### P2-5. The held drive is not what co-contraction does mechanically; harmless today, but the spec claims more than it builds

- **Where:** §4.11 part 2: "held by its opposite number (co-contraction)… no net torque on the joint, whatever the joint does"; `engine.hold_share`.
- **What is right.**
  - Before its onset the pair gives exactly zero, so it can make no energy.
  - After the onset the torque is the muscle's own, times a share between 0 and 1, booked as the muscle's work. No energy is made when the hold lets go.
  - The share depends only on the present time, the plan's onset, the tensed level and the present activation, so nothing later changes anything earlier, which keeps the owner's causality rule.
  - The two engines agree (2.9e-13 in `probe3d.py --check`).
  - The clock reads onsets, as the spec says (`arm_onsets`).
- **What is not physical.** A real co-contracted pair is the way a joint is stiffened: when the joint moves, the shortening muscle's force falls and the lengthening one's rises, so the pair resists the motion. Here the opposing hold mirrors the muscle's own force at its own speed, so the pair has no stiffness and no damping. A tensed limb is limp to outside forces until it fires.
- **Effect.** None on this session's results: S's arm muscles fire at the first instant.
- **Fix.** Write the design as what it is: "a tensed muscle whose opposite number cancels it exactly, which leaves out the stiffness co-contraction gives" [E]. Then say what a run would need to show to make that matter.

### P3-1. The two-engine check's cleats case: the explanation is right; the rate is overstated

`diverge.py` (`diverge.out`) carries the cleats case to 200 ms and runs the Python engine against itself with one velocity moved by 1e-13. That self-perturbation grows like the engines' difference and faster:

| ms | 90 | 100 | 110 | 120 | 130 | 150 | 190 | 200 |
| :-- | --: | --: | --: | --: | --: | --: | --: | --: |
| Python against the probe (tree-ordered) | 7.7e-14 | 1.8e-12 | 6.1e-12 | 2.9e-10 | 3.1e-09 | 9.0e-09 | 7.0e-08 | 7.3e-07 |
| Python against itself, moved 1e-13 | 4.8e-13 | 1.8e-11 | 9.4e-11 | 1.1e-09 | 2.7e-09 | 1.3e-08 | 7.5e-08 | 1.9e-06 |

- From 80 ms the body stands on two sole points, so it is the motion's sensitivity, not a difference of laws, as the worker says.
- "About twenty times every 10 ms" holds for 100–120 ms only. From 100 to 200 ms the growth averages about three times per 10 ms, with bursts of 10–40 times.
- Suggestion: judge such a case against its own self-perturbation (difference ≤ 10 × the self-perturbed difference) and keep 200 ms.

### P3-2. Dead ends missing from the research index

`0ae9ea3` wrote the index before the fallback step (`a1a51fe`) and its runs. "The best single difference taken when no damped step is accepted (the fallback): the score fell 4–14% and the misses hardly moved" is a dead end, and it has no line. The first follow-up (part 1's S carried on 30 iterations, stopped by the reading fault) is withdrawn in §4.11 but not in the index.

### P3-3. The handle's obliqueness is picked by [4]'s launch posture

- 15° was chosen because at 30° the lead wrist cannot take [4]'s launch posture. That is not a D37 breach: [4]'s posture is the start, flagged, not a target or a check.
- But the [E] now depends on [4], and a start that owes [4] nothing should re-examine it.
- Kapandji's 100–110° is "from memory, not re-read". 15° is the middle of that range, so if the source holds, the choice is anatomical whatever the launch posture.
- My variants show the obliqueness does not affect the lead arm's strain (P2-1).

### P3-4. The upper arm's turn has one range at every height of the arm

The ends (69° in, 104° out) are measured at 90° out to the side. With the arm across the chest, as the lead arm is at the start, a shoulder turns in less. If anything, the lead arm's real end is tighter than the stop. State it as [E].

## For the owner (§4.4): corrected sentences

- **Replace** "With real hands, the 40 mph swing of session 8 cannot be made: it depended on a back wrist bent where no wrist goes and on forearms and upper arms turning further than a body's do." **with:** "With real hands, session 8's exact 40 mph motion cannot be made. Its front upper arm turned in 79° past where a shoulder turns, its back upper arm turned out 25° past, and its back wrist started where no wrist goes. Whether some other 40 mph or faster swing can be made on this body is not known."
- **Replace** the first bullet under "What is not known" with: "*Whether this body can swing at all with real hands.* The search that looks for a swing has two faults of its own, found by the review. (1) The way it charges the arms for starting too early cannot see that starting several arm muscles a little later together is better: from the nearest motion, doing that improves the score 9% at once. (2) The score is too jumpy for its step-by-step method; along every step it tried, the score got worse within a hundredth of the step. Part of that jumpiness is the size of the time step the runs use. Also, the starting posture taken from the lab study puts his front arm within about 1° of two of its ends and his front hip within 3° of its sideways end, and his front arm reaches its end 27 ms after the start even with no muscle working. His back wrist starts bent back and tilted away from the thumb together, which a real wrist does not reach. None of this tells us about his body yet."
- **Replace** "*How long he must watch the pitch.* Both answers were run (0.10 s and the game's 0.20 s). With 0.20 s the search did not move off a plan that breaks the rule, so there is nothing yet to put to you." **with:** "*How long he must watch the pitch.* Both answers were run, but the search's fault (1) above is what stopped the 0.20 s runs: moving his arm muscles 20 ms later together improves that score by 9%, and the search could not see it. Nothing is known yet about the viewing time, and nothing is put to you."
- **Replace** "*Whether his back foot needs to come up on its toe.* It never does in any run. On cleats it stops sliding backward (30° before, 4° now) and his hips open 58° instead of 35°." **with:** "*Whether his back foot needs to come up on its toe.* It can: in a test with his back calf switched on, his back heel comes up 3–7 cm (more if the calf is on from the start). In the searched motions it never does, because none of the plans the search kept uses his back calf (the nearest motion uses under 1% of its strength). His back foot slid 28° backward with the hands alone, 8° after the second part (the same flat feet), and 4° on cleats; his hips turned 34°, 46° and 58°. The cleats' own share is the last step of each, in searches that had not settled."
- **In "What failed, with margins"**, add after "two starts that agree, none": "his back wrist starts outside a real wrist's reach (P1-3)."
- **In "Where this leaves your ruling"**, replace the last sentence with: "The next session's first jobs are the search's two faults and a start off the ends of his joints; then a starting plan made for this body, or a test that settles whether the body can make a swing without needing a search to find it."
- *Waiting on you:* nothing. I found nothing that is the owner's to decide. The start's change, the clock term's form, the search and the wrist's range are modelling choices or bugs.

## What is left, and its order

With two sessions left, I would put first what decides between "the search" and "the body" cheaply, before any new body part or D39:

1. **The clock term charges each arm muscle's own overrun** (P1-1). Small, testable: a test that moving all tied onsets together lowers the term, and that each onset alone has a slope.
2. **A start off the ends** by a stated rule (P2-1), with the rear wrist inside an oblique range or a corner test (P1-3), and the run ended 50 ms after contact (P2-2).
3. **The cheapest decisive test of whether the body can swing, without a search.** Build a motion by hand from that start to the hitter's contact posture the tests already construct (`test_a_hitters_contact_posture_lies_inside_every_arm_range`) at a gate bat speed, and solve its joint torques with the inverse dynamics already built (`inverse3d.py`). Compare each torque with its strength at that speed and each end's load with zero.
   - If one such motion fits, the body can swing, by construction, and the search's job is control.
   - If every motion tried fails at a named joint, that is the plain showing D41 asks for.
   - It needs no search and no score.
4. **Then the search again**, at 0.25 ms or 0.125 ms with a method that does not need slopes, from that start, with the rear calf free to work.
5. D39, the toe flexors, the grip's [E]s, as "Status and what is left" has them.

## Verification

What I ran (outputs in `sim/target/codex-probes/review-body-3d-9/`):

- `python3 -m unittest analysis/oracle/test_body3d.py`: **60 tests, OK** (`t_body3d.out`). `python3 -m unittest analysis/oracle/test_oracle.py`: **112, OK, 8 expected failures** (`t_oracle.out`). `python3 analysis/research/body/probe3d.py --check`: **PASS, worst 3.60e-11** over eight cases (`probe_check.out`). `python3 bin/check-docs.py`: clean except two lines in `REVIEW-BODY-3D-9.md` naming this file before it existed (`checkdocs.out`). Job 378, after touching every changed source except the probe's Rust source (see the header).
- **Headline figures at both steps** (`repro.py`, `repro.out`):
  - S at 0.5 ms: contact 263.4 ms after the start, miss +3.12 cm up and +7.91 cm along the bat, 19.02 m/s, attack −17.7°, hands 9.7 m/s, hips 353°/s, trunk 511°/s, front foot 1.13 × body weight, lead knee 54°. At 0.125 ms: +3.09 and +7.89 cm, 19.15 m/s.
  - Part 1's end: 15.25 m/s, +3.37 and +7.62 cm at 0.5 ms; 15.34, +3.47 and +7.54 at 0.125 ms.
  - All as §4.11 states.
- **Passivity, a case the tests do not cover** (`passive.py`, `passive.out`, `passive2.out`). The hands' body with cleats, no ground, no gravity, no muscle, no joint damping, every stop elastic (its damper zeroed), every body flung with random spins, so every joint's parent moves too. 16–22 stops engaged per run, among them every kind session 9 added: both wrists' angles, the forearms' and upper arms' turns, the raise's end, the ankles' bend and roll. A passive, elastic set of stops must return the kinetic energy it took whenever none is engaged:

| run | energy at the start | most taken into the stops at once | kinetic energy minus the start's, with no stop engaged, at its largest |
| :-- | --: | --: | --: |
| seed 4, 0.05 ms step | 506.5 J | 177.4 J | +8.9e-3 J |
| seed 4, 0.025 ms step | 506.5 J | 177.4 J | +2.8e-3 J |
| seed 1, 0.1 ms step | 156.3 J | 60.0 J | −0.91 J at the end |
| seed 1, 0.05 ms step | 156.3 J | 60.0 J | at most +0.12, −0.013 at the end |

  The error takes either sign and shrinks with the step. That is the integrator at the stops' kink, not a gradient fault, which would not shrink. I find no energy made.
- **The upper arm's turn:** a singular reading fenced by an anatomical end, and the only reading of its kind. The other new readings (both wrist angles, the forearms' turn, the ankles' roll) are kind 0 stops on two-axis joints whose axes the joint holds square, so they are never undefined. The raise's own kind 1 gradient fails at the same overhead posture (P2-2).
- **Not checked:**
  - I did not fit Gehrmann's envelope to the rear wrist; P1-3 uses the ellipse that fits inside the box, the plainest reading.
  - I ran no search, from a freed start or with a fixed clock term, so how much of "no swing" the start alone explains is not measured.
  - I did not re-read Kapandji, Soucie, Boone & Azen, Baumbach or Hallaceli; for Brosnan I read note 12 D's table, not the paper.
  - I did not check OpenBiomechanics' wrist sign conventions behind the "2.9 SD" comparison.
  - I did not check the Rust `hold_share` line by line; the two-engine check with held muscles at staggered onsets covers it numerically.
  - D37: I read every place session 9 touches [4]. The start stays [4]'s launch posture, flagged; nothing of [4] is a target, a penalty or a check. The obliqueness note is P3-3.

## Resolution

Open.
