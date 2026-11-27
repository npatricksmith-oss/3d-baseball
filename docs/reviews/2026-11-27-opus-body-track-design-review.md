# Review: the body-track design (body-motion §4.16, PLAN D47) and its strength finding

**Reviewer:** rev-body-track (Claude Opus 5.5, high effort), 2026-11-27. I wrote none of this work.
**Range:** `main..work/body-track`: e08f3d8 (the probe `analysis/research/body/real_inverse.py`) and 1e95316 (the design, body-motion §4.16). No sim crate changes, so nothing here can change a game.
**Method:** I read the probe line by line and ran it. Then I ran it again with changes, and wrote checks of my own on the same data (listed under Verification). Every figure below is from those runs. Their outputs are under `sim/target/review-probes/body-track/` and are not committed, because the data is CC BY-NC-SA.

## Does the strength finding stand?

**Not as written.** The design says his swing needs two to three times D34's strength at five places: the front hip's block, the trunk's turn, both shoulders and the front elbow. Two of those five come from faults in the probe, and a third rests on one arbitrary choice:

- **The legs' figures rest on a sign error.** The probe reads the force plates' sideways force (toward first base) with the wrong sign (P1-1). With that fixed, the front hip's block needs 320–360 N·m. That is under its 432 N·m limit when the hip is held still, not 617–668. Two other leg rows (the front hip turning the pelvis, the front hip pushing out to the side) disappear. A different one appears: the front hip pulling in (adduction).
- **The arms' figures rest on one choice of how the two hands share the bat's load.** Over every possible sharing, the hardest-worked arm muscle needs 1.3–1.5 times its limit at its speed, not 2.3–2.9 (P1-2).
- **The trunk's turn stands, smaller.** It needs 1.3–1.4 times its limit even before speed is counted. That holds at every filter setting and is unaffected by where the filter ends. The plates confirm it once their sign is fixed. At his turning speed it is about 2 times, but that figure depends on a part of the strength-at-speed curve that nobody has measured.

**The smallest figure I can defend:** his trunk's turn toward the pitcher needs about 1.3 times our batter's limit, for about 35 ms, 110 to 78 ms before contact. Every other group is within about 1.5 times, or goes over only on a part of a speed curve or a limit that has no source. Q71 should not go to the owner with "two to three times at five places".

## Findings

### P1-1. The force plates' sideways force has the wrong sign (`real_inverse.py`, `plates()`)

**What is wrong.** `plates()` turns the dataset's plate forces into the sim's frame with `RS.LAB_TO_SIM`, the same turn the markers get. For the plates' lab-y component (the sim's X, toward first base), that gives the wrong sign. The fore-and-aft and vertical components are right.

**How I know.** Three checks, each independent of the others:
1. **The centre of mass's sideways speed.** I integrated the plates' force (mass × acceleration) forward from 1.45 s before contact. It gives the centre of mass's sideways speed as the mirror image of the speed the markers show: +0.05 against −0.05 m/s at −0.9 s, +0.06 against −0.05 at −0.6 s, −0.08 against +0.07 at −0.3 s. The fore-and-aft and vertical speeds agree.
2. **Correlation.** The plates' sideways force correlates negatively with what his motion needs in every stretch of the recording: −0.33 in the stance, −0.36 in the leg lift and stride, −0.60 in the swing. The other two components correlate at +0.81 to +0.97.
3. **The turning moment about the vertical.** I took the whole body's turning moment about the vertical through its centre of mass, as his motion needs it. Against the plates with zero twist at the feet, the unexplained part is 319 N·m rms with the sign as coded, and 73 N·m rms with it flipped. With the flip, the plates give 200–300 N·m where his motion needs 250–330 (−155 to −78 ms). On the pelvis, the probe's own balance (its §2) drops from 328 to 104 N·m rms about the vertical, and from 258 to 168 about the horizontal.

**What it changes.** I reran the probe with only that one sign flipped (`ri_copy.py --flipx`), at 10, 15 and 20 Hz. Strength at speed is shown as in the design: need divided by strength at that speed.

| Row | The design (as coded) | Sign fixed |
| :-- | --: | --: |
| front hip straightening (the block): peak need, N·m (limit 432) | 617–668 | **320–362** (255–446 with the pressure moved 4 cm) |
| the same, need ÷ strength at speed | 2.2–2.9 | 1.9–2.0, at 8.4–8.7 rad/s of hip extension, where the curve is [E] past 4.19 rad/s (body-motion §4.5's table) |
| front hip turning the pelvis | 1.7–2.1 | 0.6–0.9 |
| front hip pushing out (abduction) | 1.2–1.5 | 0.5 |
| **front hip pulling in (adduction)**: held still / at speed | not in the table | **1.3–1.4 / 2.2–2.6**, against a limit that is [E] (set equal to abduction's, which came from a hand-held dynamometer on college pitchers) |
| rear hip adduction / rear hip turning (no source) | inside | 1.3–1.5 / 1.3 |
| front ankle pushing down, at speed | 1.6–1.75 | 1.4–1.55 (0.85 held still; 115–281 N·m with the pressure moved 4 cm, so not robust) |

So the plates paragraph is wrong in part. It said the 0.2–0.4 body-weight disagreement through the swing is "the mass model and the markers at swing speed, not the plates", and that the 330 N·m about the vertical "is where" the feet's twist would be. The sideways disagreement drops from 0.36 to 0.24–0.29 body weights rms, and the turning residual drops by two thirds. The dead-end entry in `docs/research/body/README.md` repeats the claim. The plates' forces, sign corrected, are good evidence: they confirm the whole body's turning effort to about 100 N·m.

**Fix.** Flip the sim-X component in `plates()`, saying why (the evidence above), and rerun at the three filters. Rewrite the table, the plates paragraph, the README entry and Q71 from the new output. Any later code that reads the plates (part 6's checks, the front foot's load in the gate) must use the corrected sign. The vertical rows of the gate are not affected.

### P1-2. The headline is not robust joint by joint, and Q71 states it to the owner

The design calls the shortfall robust "at every filter and with the pressure moved", and says its own error "could be off by a third, though not by a half". Joint by joint:

- **Trunk turning toward the pitcher: it stands, smaller.**
  - **Held still:** the need about the pelvis's axis (the model's `core` channel acts about the pelvis's z, not the trunk's) is 320–356 N·m for about 35 ms, from −111 to −78 ms. That is **1.28–1.36 times** the 261 N·m limit at 10, 15 and 20 Hz.
  - **The filter's end does not matter:** filtering through 100 ms after contact changes these by under 1%.
  - **What makes the need:** the bat and hands (164–245 N·m of it) and the arms. The trunk's own mass model is a small part (36–80 N·m), so de Leva's shares on a heavy man barely matter here.
  - **At speed it is soft.** The ratio is 1.9–2.15 using the trunk's turn from his four thorax markers. It is 3.1–4.1 using the line between the dataset's shoulder centres, which turns up to 1119°/s against the thorax's 906. The probe's 2.0–2.6 lies between the two. Either way the separation speed (270–520°/s) is past the curve's last measured point (240°/s, [E] above it).
  - The design's top figure, 452 N·m, falls at −25 ms, inside the stretch it says it does not read.
- **The arms: an artifact of one choice.** The probe gives the bat's load to the hands as two point forces at the wrist centres, the least forces that make the bat's moment, with no wrist torque. That choice also drops up to 114 N·m (the moment along the line between the wrists), so it is neither the best case nor the worst. I solved, at every sample, for the sharing (forces between the hands and each wrist's moment within the wrist's strength) that makes the hardest-worked arm group as easy as possible. The groups were: elbow straightening and bending, forearm turn, the shoulder's three groups decomposed on the model's own axes, and the wrists. Before 40 ms ahead of contact, **the worst arm group needs 1.27–1.54 times** its strength at speed (the rear elbow straightening at −50 ms; at −75 ms, the rear shoulder's sweep and the rear elbow), against the probe's 2.3–2.9 for the shoulders and front elbow. That solve is generous in one way: it lets the front arm sweep back with the rear arm's strength, where the model has no such group. So 1.3–1.5 is a lower bound, and this is exactly the check the design defers to session 14. The arm speed curves are one tennis player's (Kentel), so these at-speed figures carry that too.
- **The front hip's block:** see P1-1. It is inside its limit held still, and only the [E] part of its speed curve puts it over.
- **The rear elbow** (3.6–6.3): it is straightening at 450–710°/s while the need at −50 ms is a *bending* torque (−186 N·m). That is the motion straightening it while its muscles brake, which is research index finding 2 (late speed is passed along the body, not made at that joint). The large figures sit at the window's end. This row should not be read as a shortfall.
- **Two-joint muscles.** The latissimus dorsi runs from the pelvis and lower back to the upper arm. It can add to the turn at the lumbar joint and at the shoulder at once, which single-joint channels with separately measured limits cannot show. The trunk-rotation tests behind the limit (Bae's golfers, seated, arms crossed) leave that path out. I did not size it. It is a reason the trunk's 1.3 times may overstate the gap, not evidence that it does.

**What Q71 should say instead** (for the Coordinator to put to the owner after session 14's solve): his trunk's turn needs about 1.3 times our limit when it is held still, and more at his speed, by an amount we cannot measure. A few other places come near 1.5 times, but only on unmeasured speed curves or on limits that have no source.

### P2-1. The Koike comparison does not support the probe as claimed

The design says Koike 2003's bottom-hand shoulder torque, about 190 N·m at contact, "agrees with the probe and not with the bound" (92 N·m; the probe's 212–219). Note 2 §2.1 records it as **shoulder abduction**: holding the bat in against its pull, read off a plot, from five college hitters off a tee, **at impact**. The 92 is the limit for a different direction: sweeping the arm round (horizontal abduction). The probe's 212–219 is a third quantity, the shoulder's moment about the trunk's long axis, and it peaks 50–60 ms before contact, not at impact. The model's own limit for raising the arm is 112 N·m (`SHOULDER_RAISE`). Koike's mean is about 1.7 times that, which is a fair hint that the arm limits may be low. It does not confirm the probe's figure or its direction. Q71's "found about twice our limit too" should be restated or dropped.

### P2-2. A rigid trunk cannot follow the dataset's shoulder centres, and the gate holds every point to 3 cm

The dataset's shoulder joint centres (Visual3D's) move a lot relative to his thorax. Before contact, his right shoulder centre's distance to his C7 marker ranges from 0.138 to 0.221 m. The distance between the two shoulder centres ranges from 0.282 to 0.347 m (median 0.303). The line between them turns up to 1119°/s, where his thorax markers turn at 906.

The model has a rigid trunk with fixed shoulders. Part 2 tracks both shoulders as targets, and part 6's gate holds every compared point within 3 cm rms and 8 cm at worst. The knees also ride on skin (his thigh reads 0.44 to 0.49 m long; note 12). This has three consequences:
- the gate can fail at points where the model's build cannot follow his markers, which is no fault of its muscles;
- tracking the shoulder line would over-rotate the model's trunk against his thorax;
- an over-rotated trunk raises its turning speed and so its need at speed (P1-2).

**Recommendation:**
- Track the trunk's turn by his thorax markers (C7, T10, CLAV, STRN).
- Weight the shoulder centres lightly, or give them their own looseness.
- State the gate's tracking thresholds against session 14's best pose fit, point by point (the best a rigid body can do), not as an absolute 3 cm.

On the proportions themselves (question 4), the arithmetic checks:
- The model's shoulder half-width at his height (0.112 × 1.753 = 0.196 m) is 4.4 cm wider than half the dataset's median spacing (0.152 m).
- His upper arm is 5.2 cm longer than de Leva's at his height.
- The reach from the trunk's axis to the elbow agrees within 1 cm.
- A spacing of 0.224 × H is about a man's width across the acromions, so it is too wide for joint centres in general, not only for him.
- Winter's 0.129 × H is shoulder breadth, wider still.

Part 1 takes "his shoulder centres", though, and says the lengths come from his standing calibration trial. That trial has acromion markers (0.282 m apart) and no Visual3D joint centres. The design should say which shoulder centres the model takes and how they are found in that trial.

### P2-3. D46 is not planned for sessions 14–20

D46: the data "and anything made from it stay out of the repository and out of the game". The design does not say how sessions 14–20 keep to it:
- The practice results of sessions 10–13 are committed by habit (`analysis/research/body/practice3d_*.json` are tracked). The commands learned toward his recording are made from it.
- Part 1's `Model(H, M, lengths=…)` with his lengths, and its test ("within 1 cm of his markers' joint centres"), need his data.
- The spec itself now quotes his segment lengths, joint torques and plate figures. Whether summary figures in a spec count as "made from it" is for the Coordinator or the owner to say. The probe's own docstring treats its printed output as derived and keeps it out of git.

**The design should state:**
- his body's constants, every practice output and every export live under `sim/target` (or the ignored `swings/real/`);
- tests that need his data skip when it is absent.

**It should also tell the owner plainly** (a buried decision, which belongs with Q71 or Q74): under D46, the base swing practised toward this recording cannot go into the game without a licence from Driveline or motion capture of our own. So sessions 14–20 make a base swing to study and look at, not one the game can use. The owner may want to start the licence or the capture now rather than after session 19.

### P2-4. Q73's option (b) is dismissed on a claim that can be checked cheaply

Q73 says (b) (change only when the swing starts) "cannot let him meet a fastball up the middle". By D44, when the swing starts sets where the ball is met and which way it goes. His recorded ball went 17° to the pull side. A later start meets a ball deeper in the same arc, and so straighter. Whether his unchanged swing, started later, hits a game fastball hard up the middle is exactly what session 13's what-if (`contact3d.py --swing`, the same path run against a ball arriving earlier or later) answers. It takes minutes on his recorded bat path, with no body run. Check it before Q73 goes to the owner. If (b) works, it is the option closest to D45 ("mimic it"), and it changes nothing about his swing.

### P3-1. Session 16 already stands on one foot

Part 3 step 1 says both feet "stay there to the finish: no balance control is needed". That is true to +150 ms. Session 16 runs to +500 ms and asks the owner to watch "the front foot's step". The plates show the front foot carrying nothing from about +375 ms to the record's end: he stands on his rear foot alone for the last 125 ms. That is the open-loop one-foot balance that session 17's stop row is written for. Session 16 should carry the same stop row.

A forward run with no errors can hold an unstable posture open-loop, but a game run with errors (session 20) cannot. From step 3 on, §5.3's balance control is likely needed for every game swing, not "wherever errors make him fall". The session count should allow for it.

The handed-over moving start (Q72) is gone by session 17 (a nearly still posture at the leg lift's top) and the given posture by 18, as the design says.

### P3-2. Session 14's stop has no threshold

"Stops for the owner (Q71) if the solve confirms the probes' shortfall" does not say what confirms it. Given P1-2, the trunk's turn will almost surely exceed 1 in the solve, so the plan will stop at 14. Say which groups count, by how much, and whether held still or at speeds past the measured curves. Plan to put Q71 to the owner then, with the solve's figures and P1-1's corrected legs.

### P3-3. The probe's axes differ slightly from the model's channels

The probe compares the lumbar joint's twist about the trunk's long axis, but the model's `core` channel acts about the pelvis's (`drive.CHANNELS`, axis in the parent). The difference is −9% to +7% at the peaks (pelvis axis 355 N·m against trunk axis 334 at −89 ms). It compares the shoulder's whole moment about the trunk's axis with the sweep group alone, though the model's raise and turn groups share that axis. Session 14's solve decomposes onto the channel axes (`solve_hold_muscles`), so this matters only for the probe's table.

Also, the probe treats any shortening speed below zero as fully stretched (×1.5 legs and trunk, ×1.4 arms) at once. That is generous near zero speed and does not change any row above.

### P3-4. Say the trial-to-trial learning to the owner in one line

Part 2's learning moves the next trial's command 20–40 ms earlier than where the last trial's error showed. That is practice before any game, the kind D32 rules for. Inside a run, nothing later changes anything earlier, and in a game the command is frozen (part 4). It is consistent with D31 and his causality rule. Given how he took "the body starts it earlier" before (2026-11-24), one plain sentence to him that this is how practice works, and never happens inside a game swing, would prevent a misreading.

## Answers to the brief's other questions

- **Can a chop pass the score? Is it honest about D31? (question 2)** No chop can pass. Every joint, both feet's heel and toe and three points of the bat are held to his path at every 5 ms record, in his own time, with a worst-moment term. Only the stated windows round contact are left out, and the bat's raw markers are kept to contact there. Nothing inside a run steers the body toward the recording: feedback tracking is considered and rejected (part 2), and the first try from the inverse solve is a feedforward command, run open-loop. The ball's push on the bat comes from the game's collision. Early and late swings come from the same frozen command: at steps 1–2 as the pitch arriving earlier or later against the same run, and from step 3 by moving the whole command. I found no tracking spring and nothing later in a run that changes anything earlier.
- **The start (question 3):** see P3-1.
- **The proportions (question 4):** see P2-2.
- **The gate and the sessions (question 5):** each session ends in pictures the owner can watch in the viewer, as written. Session 14's posed skeleton needs a posed sequence exported to the viewer's file, which is a small addition to `swing_export.py` that the design should name. The gate's absolute 3 cm is the problem (P2-2). On D46, see P2-3. Nothing from the dataset is committed on this branch: the probe is code only and reads the data from `sim/target`.
- **The owner's questions (question 6):**
  - Q71 is a real choice, but its figures are wrong (P1-1, P1-2, P2-1). Its recommendation (c) reopens D34 by treating an inverse-dynamics estimate as a measurement of strength. That estimate has just been shown to move by a factor of two with one sign and one assumption, so (c) should wait for session 14's solve at least.
  - Q72 is a real choice and plainly put.
  - Q73's case against (b) needs the check in P2-4.
  - Q74 is a real choice. Its thresholds need P2-2's change.
  - The buried decision is P2-3: whether this base swing can ever be used in a game.

## Verification

All runs were on this worktree at 1e95316 with the data at `sim/target/codex-probes/obp/` (swing 478_5). The scripts and outputs are in `sim/target/review-probes/body-track/`, not committed.

- `real_inverse.py --cut 15`, unchanged: reproduced the design's figures (for example front hip extension 637 N·m, ratio 2.64 at −75 ms; trunk 452 N·m, 2.48 at −64 ms; pelvis vertical residual 328 N·m rms).
- `ri_copy.py`: the probe copied with four switches added and nothing else changed:
  - `--flipx` flips the plates' sim-X;
  - `EXTRA=<samples>` runs the filter past contact;
  - quiet mode;
  - it returns its local variables.
  Run at 10, 15 and 20 Hz with and without `--flipx`: the P1-1 table.
- `rev_fp.py`: the plates against the motion from −1.45 s, component by component, and the centre of mass's speed integrated from the plates against the markers. This is P1-1's first two checks.
- `rev.py --flipx`: elbow angle rates against the probe's extension speed (correlation 0.94 and 0.96, so the probe's elbow sign is right); the lumbar twist about the trunk's and the pelvis's axes; each body's share of it; the trunk's turn from the shoulder line and from the thorax markers; the whole body's moment about the vertical against the plates (P1-1's third check).
- `rev_arms.py --flipx` at 10, 15 and 20 Hz: the best sharing of the bat's load (a linear programme at each sample, `scipy.optimize.linprog`), checked against the probe's own elbow and shoulder moments at the probe's split (equal to rounding). It also gives the trunk's twist ratio with the three readings of its speed. Running it with `EXTRA=36` showed that the filter's end changes nothing before −50 ms.
- `rev_legs.py --flipx`: the front hip extension, hip adduction and ankle needs with their joint speeds and curve values.
- `rev_sh.py`, `rev_sh2.py`: the shoulder centres' spacing and their movement against C7, CLAV and STRN in the swing; his acromion markers in the standing calibration trial.
- The plates after contact (the csv read directly): the front foot carries 0.00 body weights from +375 ms to the record's end (P3-1).
- The limits as coded, checked by arithmetic against the sources' figures quoted in `drive.py`:
  - trunk 260 N·m (Bae, mean + 2 SD, scaled);
  - sweep of the arm round 92 and 123 N·m (Silva);
  - elbow straightening 85 N·m (Holzbaur);
  - k = 1.0055.
  I did not reopen the papers themselves. That the hip and trunk curves are [E] above 4.19 rad/s is from body-motion §4.5's table.
- Not done: the engine's own solve (session 14's job), any practice search (out of scope), and any viewer run.

## Resolution

Open.
