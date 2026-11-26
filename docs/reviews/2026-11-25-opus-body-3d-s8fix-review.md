# Independent review: session 8's fixes, the practice runs again and the corrected account

Reviewer: rev-body-3d-8fix (Claude Opus 5.5, high effort), 2026-11-25/26. Range `56f7252..d59d64e` on `work/body-3d` (not merged). Job: `REVIEW-BODY-3D-8FIX.md`. I wrote none of the change and not the first review. I edited no source, test, spec, note or TASKS file; this file is the only one I wrote in the repository, and it is not committed. My probes and their output are in `sim/target/codex-probes/review-body-3d-8fix/` (each `.py` beside its `.out`). Nothing was compiled: the probes use the session's own `body3d_probe` binary, which is newer than its source, and the Python engine. No probe needed the job queue (the longest ran about two minutes).

**Terms used below.** The *cock reading* is the number the engine's wrist stop reads (`wrist_dev_*`). The *forearm-to-barrel angle* is the angle between the forearm, pointing from the wrist to the elbow, and the bat, pointing from the knob to the barrel. *Radial deviation* tilts the hand toward the thumb; in a bat grip the barrel is on the thumb's side, so radial deviation folds the barrel back toward the forearm (the angle closes) and *ulnar deviation*, the uncocking, opens the bat out (the angle opens). A *stop* is the engine's end-of-range spring. A stop is *passive* if it can never give the body more energy than it took.

## Verdict

**Question 1 in plain sentences.** Both wrists uncock the way the worker says: the cock reading falls as the bat opens out from the forearm, for each hand, by the model's geometry and by anatomy. But for the rear (top) hand, neither "the travel was on the wrong side" nor "the wrists are not set up backwards" is the right thing to tell the owner. The weld puts the rear wrist at the start 60° more opened out than the lead wrist, beyond the end of the lead wrist's own range. Its 58° of travel, placed from there, runs through the line of the forearm and 29° past it, where no wrist goes.

The rest, in short:

- **What is right.** The stop law is fixed correctly in both engines, for both kinds of stop, damping included. My own passivity checks pass, and the old law fails them by orders of magnitude. The new scoring has no cliff. The headline figures reproduce exactly at both step sizes. The worker's lead-wrist anatomy is right and the first review's P1-2 was wrong about the lead wrist.
- **What the owner would be told wrong:**
  - "The back forearm turns 285°" is a reading that turns over. The forearm really turns about 100°. This is the same kind of fault as the first review's P1-3, in another reading.
  - "It is not for want of muscle" is not supported. The only strength test added 3.6 m/s, and the swing it found leaves the ground.
  - "He has a swing … the first bat speed a body could have made" goes further than the evidence. The swing rests on joint ends carrying up to 906 N·m, and the rear wrist starts and stays outside any wrist's range.
  - "In balance" and "meets the ball" fail the aim's own tests by small margins.
- **What the reported results lean on and do not say.** The found swing sits on the arms' clock. The clock now works as a 253 ms cap on the time from the start to contact, so Q64 is not put fairly. The swing comes from the plan fitted to the lab study, and the owner's section does not say so.

## P1 findings

### P1-1. The rear wrist: its start and its whole range lie outside a wrist's, because of the weld; the direction is right for both hands

**Where:** `analysis/oracle/body3d/body.py:139–150` and `add_ranges`; docs/body-motion.md §4.10 "Session 8's fix", item 3 (its table and "So a hitter's wrists open the bat out … that is where step 1 put the travel"); "Status and what is left" (:11, "The review's third finding … is not upheld"); §4.4 "For the owner" (:262, "the review read this as the model's wrists being set up backwards, which they are not"); the review's Resolution, row P1-2.

**What is right.**

- The direction is right for both hands.
  - The reading is 90° less the forearm-to-barrel angle while the wrist's flexion is near zero. At the start the lead wrist reads −6.2 against an angle of 95.8°, and the rear −65.8 against 155.6° (`q1_wrist_geometry`).
  - Turning the bat about the wrists' shared axis so that the lag falls lowers both readings (`uncock_sign` = −1).
  - In a bat grip the barrel is on the thumb's side. So radial deviation closes the forearm-to-barrel angle and ulnar deviation opens it. That holds whatever the model's frames are.
  - Hitters go from radial at foot plant to ulnar at contact with both wrists: lead +10 → −16°, rear +5 → −8° (note 11 A5, OpenBiomechanics).
  - The first review's "uncocking raises the reading" was wrong. It read the lag falling as the wrists releasing.
- The lead wrist, as placed, is a hitter's.
  - It starts at 96°, cocked.
  - Its stops allow 91–149°.
  - In the practised swing it opens to 147° at contact, on its stop at 75 N·m. Gélinas measured 138–157° at contact (note 11 A5).
  - The review's placing (38–96°) would have been wrong for this hand.

**What is wrong, for the rear wrist.**

1. **Its start is not a hitter's.** Both hands grip with the thumb toward the barrel (`cock_ref` +1 for each), so the same reading means the same wrist posture on either hand. Hitters' two wrists start within about 5° of each other (OpenBiomechanics at foot plant: +10.3 ± 10.4° and +5.4 ± 11.8° radial). This body's start at 156° is 60° more opened out than the lead wrist's 96°, and 7° past the opened-out end the lead wrist is given (149°). The worker says the start is "where no wrist could put it" (§4.10, item 3); that is right. But the travel is then placed from it as if it were a cocked wrist.
2. **Its range as placed is not a wrist's range.** The stops are −118.8 … −60.8 in the reading. In forearm-to-barrel angle that runs from 151°, through 180° (the forearm exactly in line with the bat), to 151° on the other side of the handle. No wrist reaches 180°.
   - In the practised swing the rear wrist presses its cocked-end stop, lightly (6 N·m), from −178 to −98 ms.
   - It then opens to 160° at contact (the reading −82.9) and passes the in-line posture 45 ms after contact (`q1_wrist_geometry`, `q1_pron_reading`).
3. **Why: the weld.** Both wrist joints sit 7 cm to the bat's −x side (`wrist_b`), so both forearms come at the handle from that side. At the start the forearm's component along the bat's x is −0.94 (lead) and −0.41 (rear). In a baseball grip the palms face opposite ways, and the two forearms come at the handle from opposite sides of it, within the plane square to the wrists' shared deviation axis. That is my reading of the grip, not a measurement.
   - In this model the rear forearm can reach the hitter's side of the handle only by going through the line of the bat. A cock reading near −150 to −165 would be a top hand at 105–120° on that side, 85–100° below the start, and the travel allows 53°.
   - A hitter's top-hand posture is beyond the rear wrist's reach in this weld with the travel placed either way. The search cannot give the rear arm a hitter's top hand, whatever the stops say.

**What a hitter's rear wrist needs, and whether the stop allows it.** A hitter's top hand starts near the lead's angle and opens about 14° by contact (OpenBiomechanics: +5 → −8°), with the rear elbow straightening from launch to 79 ± 13° at contact.

- If the rear forearm stays on its present side of the handle, its reading would have to rise about 50° from the start to reach the lead's angle. The stop as placed allows 5°.
- If it crosses to the hitter's side, it has to go through the line of the bat. The stop allows that, but no wrist can do it.

**Does the speed rest on it?** Not much. The practised plan replayed with the rear wrist held to the lead's own interval of the reading, widened only to clear its start (−67.8 … −1.2), gives 17.9 m/s at contact against 17.8 at both steps, with the miss moved to −1.1 cm up and +4.8 cm along the bat (`q1_rear_on_lead_interval`). So the reported speed does not depend on the rear wrist's travel past the lead's range. What is wrong is what the owner is told about the hands.

**Is `test_the_wrists_travel_lies_the_way_the_swing_uncocks` independent?** In part.

- Its check that the reading equals 90° less the forearm-to-barrel angle is independent geometry. With "the lag falls", it ties the direction to anatomy.
- Its criterion for uncocking, the lag falling, is the same one `uncock_sign` uses, so it cannot disagree with the code's direction.
- Its last block restates `add_ranges`' "mirrored" rule.
- It places the travel relative to the start, so it cannot see that the rear wrist's start, and with it the whole range, lies outside a wrist's. No test asks that.

**Fix.**

- Tell the owner both halves. The model's wrists uncock the right way. The rear hand, welded at the lead hand's roll, starts outside a wrist's range and can reach a hitter's top-hand posture only through an impossible one. That is a missing part of the body, the first item the worker already lists for the next session, and it is not about which side of the start the travel lies.
- Until each hand has its own grip, hold the rear cock to the same interval of the reading as the lead's (widened to clear the start, as `add_ranges` does). This replay shows it costs nothing at contact.
- Add a test that both wrists' ranges, as angles between forearm and barrel, lie inside what a wrist reaches. It fails today on the rear.
- Correct "which they are not" (§4.4), "Status" :11, §4.10 item 3's last sentence, and the Resolution's P1-2 row.

### P1-2. "The back forearm turns 285°" is a reading that turns over; it turns about 100°

**Where:** `body.py` `range_angles`, the forearm's turn (`pron_*`: `g = fx if fx·bz > 0 else −fx`); `practice3d.py` `report()` "not held"; docs/body-motion.md §4.10's checks table ("the rear forearm turns 285°"), "What this says" (:1283, "the rear forearm turns 285° … so some of even this speed is not a body's"), §4.4 "What failed" (:269), the Resolution's P2-2 row.

**What is wrong.** The turn is read against the handle's direction "as the forearm carries it", chosen by the sign of the forearm's x along the bat. When the rear wrist's cock passes −90° (the forearm in line with the bat) that sign changes and the reading jumps by 180°. That happens at +44.3 to +46.3 ms in the practised swing, where the reading goes from 120.7° to −58.9° in 2 ms. The forearm's x measured directly about its own long axis, from the same reference, turns smoothly.

**How I know** (`q1_pron_reading.out`, 1 ms records to +50 ms):

| | Travel to +50 ms |
| :-- | --: |
| the forearm's turn as `practice3d` reads it | 285.0° |
| the same turn read directly (forearm x about forearm z), no turnover | **106.0°**; about 100° to contact |

The 106° is inside a forearm's 162°. It is about twice the 50° a hitter's rear forearm turns from foot plant to contact (note 11 A5, median), but that is not "further than a body can". The rear upper arm's 176° (of 173) reads smoothly, with the arm 83–123° from hanging, far from where its split is undefined; it stands (`q4_upper_arm_turn`).

The forearms' starting readings are −168° (lead) and −134° (rear). If the reading's zero is the thumb-up neutral its comment says, both would be beyond any forearm's supination. I did not check that zero further.

**Fix.** Read the forearm's turn from the elbow's own second axis, the forearm's x about its z, from a reference carried by the upper arm. That reading cannot turn over. Add a test like `test_the_cocks_reading_does_not_move_when_flexion_passes_90`: carry the cock through ±90° at a fixed turn and check the turn's reading does not move. Strike "285°" and "so some of even this speed is not a body's" everywhere; the overrun is the upper arm's 3°.

### P1-3. "It is not strength" is not supported

**Where:** docs/body-motion.md §4.4 "Why it is slow" (:270, "It is not for want of muscle: only 5 of 41 … and the strongest batter … reaches 48 mph the same way"); §4.10 "What it sits on" and "What this says" (:1282, "It is not short for want of muscle … mostly by throwing the body off the ground"); "Status" :11.

**What is wrong.**

- A count of muscle groups at their bounds does not say whether strength limits the speed. The test of that is the strongest-batter run, and it moved the speed: 9 bounds raised by 11–41% gave **21.6 m/s against 18.0, +20%** (`practice3d_vary1.out`, 15 iterations, the fall over the last 5 was 0.3).
- That swing is not "the same way":
  - the front foot carries 0.00 of body weight at contact and the rear 0.09;
  - both feet are off the ground by +30 ms;
  - the pelvis turns to 38° (against 15°) and the lead knee straightens to 1°;
  - the rear hip's sideways stop carries 1,018 N·m.
- "Mostly by throwing the body off the ground" was not tested. No run kept the strongest batter on the ground.
- Among the five groups at full strength in the practised swing are the rear hip's rotators, which turn the pelvis. Their bound, 78 N·m, "has no source" (§4.5, as §4.10 step 0 says).

**Fix.** Say: what holds the speed is not established. Nine stronger muscle groups gave 3.6 m/s more, in a swing that left the ground. The rear hip's rotators, which turn the pelvis, are on a bound with no source.

### P1-4. "He has a swing … the first bat speed a body could have made" goes beyond the evidence

**Where:** §4.4 "What the batter does now. He has a swing" (:263) and "What D39 was ruled on" (:275); §4.10 "What this says" (:1281, "That is the first bat speed in this project that a body could have made").

**What is wrong.** The swing meets every stop to within 2°, but it does so by leaning on the stops:

- Nine joint ends carry 75–906 N·m (`show_main.out`): the rear hip's turn 906, the trunk's twist 636, the rear hip's back stop 578, the rear knee 509, the rear elbow 384 (after contact), the lead hip's turn 323.
- The largest failure moments research note 11 B3 found for any joint are 13–34 N·m (the wrist's and elbow's ligaments, the knee's ACL in torsion) and 107 ± 64 N·m (a knee bent sideways to injury). Note 11 found no failure moment for a hip.
- The rear hip's stop carries 11 times its own rotators' 78 N·m bound. Over feet that cannot pivot, the pelvis is turned against the rear thigh through the end of the hip's range, a wall no hip has.
- The rear wrist starts and stays outside a wrist's range (P1-1).

PLAN D39, which the owner ruled, is the rule that would count this. It is not built. So the honest statement is "the first swing found on stops that make no energy", not "a swing a body could make".

**Fix.** Use the first wording, and say beside it what it leans on. In "What D39 was ruled on", the reason is sound and is understated: the 906 N·m is about 8 times the largest failure moment found and 27 times the largest ligament's. D39 is likely to change this swing a great deal, which the owner should expect.

## P2 findings

### P2-1. "Meets the ball … in balance" fails the aim's own tests by small margins, and "on his feet" hides a falling body

**Where:** §4.4 "What passed" (:268) and the swing's description; §4.10's checks table, first row ("met, the balance within 3 cm"); §13's gate (search item).

**What is wrong** (`show_main.out`, `q4_show_fine.out`, both steps):

- *The ball.* The miss is 1.0 cm up and 1.6 cm along the bat. The gate (§13, D40) asks for "within 1 cm".
- *Test (a): balance.* Hof's extrapolated centre of mass is 3.0 cm (2.9 at the fine step) outside the feet's area at worst. The test is zero only when met, so it is not met.
- *The clock.* The first arm drive is 258 ms before contact against the 253 allowed; the score charges 5.1 for it. The swing breaks a rule it was searched under. The table says so; the owner's section does not.
- *On his feet.* The feet carry 0.43 of body weight between them at contact and 0.33 at +30 ms, and the centre of mass ends 5% lower than it started. He passes test (b), "does not jump", only because its floor is 0.10. He is dropping at about 0.6 g as he meets the ball.

**Fix.** "Meets the ball within 2 cm (the gate asks 1); his balance just outside the test (3 cm); his feet carry under half his weight at contact; his arms start 5 ms before the rule allows."

### P2-2. The clock is now a 253 ms cap from the start to contact; the swing sits on it, and Q64 is not put fairly

**Where:** `practice3d.py` `arm_onsets` (a tensed arm muscle counts as started at the run's start) and `residuals` (the clock term); §4.10 step 1's "when that happens is a result"; §4.4 Q64.

**What is wrong.**

- Plan A, which became the swing, starts with 15 arm muscles tensed, among them the lead triceps at 1.00 and the forearms' turners at 0.98–1.00. Under the fix's rule each of those counts as an arm drive from the run's start.
- The clock then reads: contact no later than 253 ms after the start, which is placed on [4]'s state at its −250 ms. It is binding at the result (258 ms, charged 5.1), so the swing's duration is set by this cap, not found.
- The cap's source is the pitch, not [4], so D37 is not broken. But "how long the swing takes from its start is a result" is not true of this swing.

Q64 asks only how long he watches.

- Under (b), 0.2 s, any arm muscle tensed at this start would force contact within 153 ms of it. That is far from "a swing whose arms need longer fails": it would force the search to drop all arm tension before the swing.
- The recorded lead shoulder in hitters is active at 76–101% before the swing (§4.10's checks), so under the present counting rule a hitter would fail any viewing time.
- The viewing time and the counting rule cannot be decided apart. The second is a modelling question: the worker says the engine lacks a drive that holds without net torque until it fires.

**Fix.** Before Q64 is asked, build or choose how a muscle tensed before the pitch counts (the worker's own "a drive that holds without net torque"). Then put Q64 with what each option does to this start. Until then the 0.10 s is an [E], and the game's own 0.2 s is the consistent default to test. Say in §4.4 that the swing breaks the clock by 5 ms and that its 258 ms is the cap, not a free result. "His swing uses 258" should say from where: the placed start, not the hands' first move (204 ms).

### P2-3. The swing comes from the plan fitted to the lab study; the owner's section does not say so

**Where:** §4.4 "For the owner" and "In one paragraph"; "Status and what is left" :11.

**What is wrong.**

- The one settled swing came from start A, session 7's fit to [4], and from [4]'s placed launch state.
- §4.10's tables say so, and the step 0 table requires every such start to be "flagged, never the reference".
- §4.4 says only "only one of four starts", and "In one paragraph" and the status line say nothing.
- The two starts that owe nothing to [4] (C and D) never became swings.

**Fix.** One clause wherever the result is quoted: "found from the plan fitted to the lab study, from its launch posture; the two starts that owe it nothing did not get there."

### P2-4. The new scoring leaves a parked bat about as good as the swing found

**Where:** `practice3d.py` `residuals`, `contact="nearest"`.

**What is wrong.**

- With contact read at the nearest approach, a sweet spot brought to the ball's place and stopped there scores the speed term 45 (45 m/s asked, none given). It scores zero on the line, square if placed so, and little else.
- The practised swing scores 46.7: speed 27.5, and 19.2 for the miss, the line, squareness, balance and the clock.
- So an ideally parked bat would score about the same as the swing (45 plus its effort and balance). The cliff is gone, but a basin about as deep as the swing's sits beside it.
- A bat moving away from the ball is charged more than 45, which is right. There is no pitch arrival time, so contact can come at any instant; only the clock links it to the pitch, as the first review said.
- I did not see a result exploit this. It may be part of why three starts of four do not become swings, and the staged search's reach stage, which asks for no speed, starts every search near it.

**Fix.** Make arriving still worse than swinging. For example, score the line and squareness at the plane's crossing when there is one, and add to the nearest-approach reading a term for the sweet spot not moving toward the ball along its line. State it before any run, as the worker did for the amendment.

### P2-5. "How much rests on what": the variants are too unsettled for the finer statements, and the hips' run does not say "changes nothing"

**Where:** §4.10's last table and the bullets after it; §4.4 "How much rests on what" and "Why it is slow".

**What is wrong.**

- *Not settled enough for the finer statements.*
  - The baseline was "settled" (it fell 0.013 over its last 5 iterations), yet 15 more iterations moved it from 17.8 to 18.0 m/s, falling 0.5–0.9 per 5.
  - The start variants fell 2.3–2.9 over their last 5.
  - So differences under about 1 m/s are inside what an unsettled search moves.
  - "The start moves it 18.2–19.1" includes two swings that leave the ground (the rear elbow bent 15° more: 19.1; the velocities halved: 18.5).
  - "Nothing rests on effort's weight" is supported for moves near this plan, where effort costs 0.8 of 46.7; it does not show more than that.
- *The hips' run.* With 11° more turn in each hip the pelvis reached 18° at contact against 15. The rear hip's turning stop still carried 764 N·m, at its new end. So the extra range was used at the hip and not passed on to the pelvis. "Eleven degrees more turn changes nothing" and "so the sockets' range is not shown to be the limit" read the opposite way from what the run shows.
  - The rear hip is still against its end.
  - The feet are flat and the knees are hinges, so most of the pelvis's turn has to come from the hips' rotation.
  - Those two together point at the planted feet first, as the worker's own step 1 expected.
- *Other candidates the account does not name:* the clock's cap (P2-2), the parked basin (P2-4), the arms tensed from the start, and the score's [E] weights for the line and squareness (5.5 of the 46.7 is the bat 30° past square).

**Fix.** Report each variant's difference against the baseline carried on the same 15 iterations, with "both feet left the ground" beside it. Restate the hips' run as above. Add the candidates. The cheapest test is a rear foot that can pivot, or a knee that can twist.

### P2-6. Corrections on main still carry the first review's P1-2

**Where:** on main:

- PLAN D41's reason: "Corrected the same night by the review (P1-2, P2-1): … the code had placed the wrists' travel on the wrong side".
- Research note 11, line 5: "That came from a sign error in a test (the wrists' travel was placed on the wrong side)".
- D39's reason: "the batter bounced off his own joints … flung back (39 mph with nothing there, 107 with a stiffer spring)".

**What is wrong.** The lead wrist's travel was placed correctly (P1-1 above), and the bounce was energy the stop created. These are the Coordinator's files, not the worker's. They need the same correction, so that the owner does not read the withdrawn statements in PLAN.

**Fix.** The Coordinator amends the three: for D41 and note 11, "the wrists' travel was placed the right way; the rear wrist's start and range are outside a wrist's because of the weld"; for D39, "the energy came from a fault in the stop, not a bounce."

## P3 findings

- **"90° less the angle between forearm and barrel"** (§4.10, item 3's table; `body.py`'s comment) is exact only with no flexion. In general the cosine of the forearm-to-barrel angle is sin(cock) × cos(flexion). The table's "64°" uses the true angle. Say "with the wrist unflexed".
- **The fit to [4]'s wrist figures** (+59° and +58°, 64° from each forearm) do not say on which body they were read. On the practice body with no stops, to the bat's azimuth −8° at 0.125 ms, I get lead +24° (74°) and rear +62° (60°) (`q1_fit4_wrists`). Both fold back as the account says; the lead's figure differs.
- **Stale text under superseded headers:**
  - "a bounce off the lead shoulder (3,700 N·m)" (§4.10's first main-run table, start B);
  - "The ball is 45 cm below and 20 cm beyond where this body's bat goes" (obstacle 2), still stated as fact under its "not a cause shown" mark;
  - the withdrawn bullet's unstruck "its speed was the lead arm thrown against the end of its range and flung back by the stop's spring, not muscle".

  Strike or mark each.
- **§13** still says "if the owner rules Q61 = b, no joint's end … more than 50 N·m". He has ruled (D39), and note 11 B3 says nothing measured stands behind 50 N·m. Say it is ruled and that the size is the next session's.
- **The passivity checks' residuals.** In my two-body runs the new law keeps the stops' booked work plus their stored energy at or below 0.05 J, against 6–24 J in play. The residual does not shrink cleanly with the step, in motions that tumble chaotically and diverge between step sizes. That is numerical; the old law gains up to 120,000 J in the same runs. No action, but the engine's passivity check might state its tolerance.

## For the owner: what §4.4 should become

- "the review read this as the model's wrists being set up backwards, which they are not": *the model's wrists uncock the right way. But the back hand, welded to the bat at the front hand's angle, starts in a position no wrist can take: bent out 60° more than the front wrist, where a hitter's two wrists start nearly alike. It can reach a hitter's top-hand position only through an impossible one. That is the grip to build, the next session's first part.*
- "He has a swing … the barrel within 2 cm of the ball … on his feet": *the search found a swing that meets the ball within 2 cm (the gate asks 1), at 40 mph. But it leans on his joints' ends harder than any joint can take (900 N·m at the back hip's end of turn, where ligaments fail at 13–34), his balance is just outside the test, and he is dropping at contact with under half his weight on his feet. It is the first swing found without the program's made-up energy, not yet a swing a body could make.*
- "What passed: he meets the ball; he stays on his feet": *he meets it within 2 cm; he does not jump.* Add to "What failed": *the meeting within 1 cm; balance by 3 cm; the arms start 5 ms before the rule allows.*
- "two joints that nothing holds went further than a body can (the back forearm turns 285°…)": *the back upper arm turns 3° further than a body can. The back forearm turns about 100°, twice a hitter's, inside a body's range; the 285° was a reading fault.*
- "It is not for want of muscle … reaches 48 mph the same way": *not established. Making nine muscle groups as strong as the measurements allow added 8 mph, in a swing that leaves the ground. The back hip's turning muscles, which turn the pelvis, are at their limit, and that limit has no measurement behind it.*
- "when the hips were given 11° more turn the swing did not change, so that range is not shown to be the limit": *with 11° more the back hip went to its new end and the pelvis turned only 3° more. With the feet flat and unable to pivot, the hips' turn is most of what the pelvis has. The planted feet are the first thing to test.*
- "On where he starts from: little (18.2–19.1)": *little, within what a search that has not settled moves (about 1 m/s); two of those swings leave the ground.*
- Add, where the swing is first described: *found from the plan fitted to the lab study and its starting posture; the two starts that owe it nothing did not become swings.*
- "What D39 was ruled on": keep, and say the load is about 8 times the largest failure moment found for any joint, so the swing will change when D39 is built.
- **Q64:** withdraw as put. Under the present rule a muscle tensed before the pitch counts as started, so the clock caps the whole motion at 253 ms from the starting posture. The 258 ms is that cap, not the arms' swing, which takes 204. A hitter's front shoulder is measured active before the swing, so he would fail any viewing time under this rule. The rule for a tensed muscle is a modelling question to settle first. Then Q64 can be asked with what each option does, and the game's own 0.2 s is the consistent default to try.

## Verification

All under `sim/target/codex-probes/review-body-3d-8fix/` unless named. Every probe's `.out` is beside it.

- `python3 -m unittest analysis/oracle/test_body3d.py`: 39 tests, OK (19 s) (`v_test_body3d.out`).
- `python3 -m unittest analysis/oracle/test_oracle.py`: 109 tests, OK (8 expected failures) (`v_test_oracle.out`).
- `python3 analysis/research/body/probe3d.py --check`: PASS, worst 3.6e-11 against 1e-9, including the cases with the joints' ranges on (`v_probe_check.out`).
- `python3 bin/check-docs.py`: before this file existed, its only complaints were the two references in `REVIEW-BODY-3D-8FIX.md` to this file (`v_check_docs.out`). Rerun after writing it: see the end of this section.
- **The headline, reproduced** (`practice3d.py --show` → `show_main.out` at 0.5 ms; `q4_show_fine.py` at 0.125 ms). At both steps:
  - the miss +1.0 cm up, +1.6 cm along the bat; 17.8 m/s at contact (17.6 along the line); attack +7.3°;
  - the hands 7.7 m/s; the hips 331 and 330°/s; the trunk 540;
  - the front foot 0.69 and 0.71 × body weight;
  - the stops −39 and −38 J to contact, the ground −31 J;
  - contact 258 ms after the start; the bat 29.5° past square.
- **The gate's checks, as the result really stands:**
  - passes: the attack angle, the swing time (204 ms), no stop more than 2° past its end (1.9), the passivity check;
  - fails: the meeting within 1 cm, balance test (a) by 3 cm, the clock by 5 ms, the bat's speed, the hands' speed, the bat at its fastest at contact (91%), the order of first rises, the front leg, the front foot's load, the hips' and trunk's turning, the rear upper arm's turn (by 3°), two starts that owe each other nothing;
  - the rear forearm's 285° fails only as misread (P1-2).
- **The stop law** (read in `engine.py` `stop_angle` and the force loop, and in `body3d_probe.rs`).
  - Both apply the torque and the damping along the read angle's gradient and book the power with the same vector.
  - I checked the gradients by hand: kind 0 gives d(angle)/dt = ω_rel · (n − (n·f) f_p / |f_p|²); kind 1 gives ω_rel · (f × n)/|f × n|.
  - The one-sided clip only ever removes energy.
  - Every saved result (the main swing, the staged starts, the 15 variants) reports the stops and the ground at or below zero.
- **My own passivity check** (`q2_passivity`). Two free bodies, a "pelvis" on the world and a "thigh" on it, no gravity. One stop is a hip's turn: kind 0, its axis fixed in the moving thigh. The other is a sideways stop: kind 1. Tilts 0, 35 and 70°, two spins, damping 0 and 60, and with and without a 60 N·m muscle driving the thigh into its stop.
  - New law: the stops' booked work plus what they store never above 0.05 J, against 6–24 J of energy (residuals discussed in P3).
  - The law before the review: +0.06 J to +120,000 J in the same cases, including tilt 0 with the moving axis.
  - The kind 0 cases whose thigh swings out to the side (there is no sideways stop in that toy) fail loudly with the engine's "undefined" error, as designed.
- **Undefined postures** (`q2_singular_margin`, the practised swing, the soft-stops and the strongest-batter results, to +50 ms).
  - Kind 0 stops come no nearer than 38° to where their angle is undefined: the trunk's twist.
  - The drawn-back arm reads 5–6° from its undefined posture, but toward the side its stop never limits (its low end is −91°), so it never asks for its gradient. The account's statements on this hold.
- **The wrists** (`q1_wrist_geometry`, `q1_fit4_wrists`, `q1_pron_reading`, `q1_rear_on_lead_interval`): P1-1 and P1-2 above. `uncock_sign` is −1 on the placed start.
- **The variants** (read from the worker's `practice3d_vary1.out` and `practice3d_vary2.out`, not rerun): P1-3 and P2-5.

**What I did not check.**

- I did not rerun any search. The parked-bat basin (P2-4) is arithmetic on the score's terms, not a run.
- The absolute zero of the forearms' turn reading. The grip geometry of a real hitter's hands (opposite palms) is my reading, not a source.
- The PLAN and note 11 texts on main were read, not edited.

**check-docs, rerun after this file was written:** clean, exit 0 (`v_check_docs_after.out`).

## Resolution

Written 2026-11-26 by worker body-3d (job `WORKER-BODY-3D-8FIX2.md`). No search was rerun, no stop moved and no score changed; the swing in `practice3d_main.json` is as it was. Two readings are fixed with tests; the rest is the account ([body-motion](../body-motion.md) §4.4, "Status and what is left", §4.10's fix blocks, §13).

| Finding | What was done | Left |
| :-- | :-- | :-- |
| **P1-1** the rear wrist outside a wrist's range | **Accepted.** Both halves are now said together wherever the matter is stated (§4.4 "The hands, both halves"; "Status"; §4.10 the fix, item 3; the first review's Resolution, row P1-2; research note 11, line 5; the research index). `test_the_rear_wrists_range_is_one_a_wrist_reaches` asks that each wrist's range, as an angle between forearm and barrel, lie inside what a wrist reaches (68–157°); the lead's does (91–149°) and the rear's does not (151° through 180° to 29° past), so it is an expected failure with the reason and what removes it. The replay with the rear cock held to the lead's interval (17.9 m/s; the miss −1.1 and +4.8 cm) is recorded in §4.10 as the reviewer's, with the probe's name | **The stop is not moved** (the job's instruction); the suggested interim (hold the rear cock to the lead's interval) is not applied. Each hand's own grip is the next session's first item |
| **P1-2** the forearm's turn turns over | **Fixed.** The turn is read on the elbow's own axis from a reference fixed at the start. `test_the_forearms_turn_does_not_move_when_the_cock_passes_the_line_of_the_bat` (the old reading jumps in it). The saved swing and every variant were reread, no run repeated: **the rear forearm turns 106° to 50 ms after contact, 99° to contact**, as the review found; 85–130° across the variants. "285°" and "some of even this speed is not a body's" are struck; the overrun is the rear upper arm's 3° (176° of 173°) | The forearms' zero (−168° and −134° at the start) was not checked further; it comes with the hands |
| **P1-3** "not for want of muscle" | **Accepted.** §4.4, §4.10 and "Status" now say it is not established, what the strongest-batter run did (+3.6 m/s, +20%, off the ground, not "the same way"; no run kept that batter on the ground), and that the rear hip's rotators are on a bound with no source | — |
| **P1-4** "a swing a body could have made" | **Accepted.** Everywhere: "the first swing found on stops that make no energy", with what it leans on beside it (nine joint ends at 75–906 N·m against ligaments that fail at 13–34; the rear wrist) and that PLAN D39 is likely to change it a great deal | — |
| **P2-1** the margins | **Accepted.** §4.4 and §4.10's checks table: within 2 cm where the gate asks 1; balance 3 cm outside its test; 0.43 of his weight on his feet at contact and dropping; the arms 5 ms before the clock allows | — |
| **P2-2** the clock is a cap; Q64 | **Accepted. Q64 is withdrawn as put.** §4.4 and §4.10 (the fix, item 5; the aim's "a result") say the rule works as a 253 ms cap from the starting posture and that the swing sits on it. How a tensed muscle holds is item 2 of what is left, before the question returns to the owner. "Waiting on you" lists nothing | The rule itself is unchanged in this job |
| **P2-3** where the swing came from | **Accepted.** One clause in the status line, the "Status" bullet, "In one paragraph", §4.4 and §4.10's results | — |
| **P2-4** the parked basin | **Not changed, as the job says.** §4.10 states it with the review's arithmetic; the remedy is item 5 of what is left, to be stated before any run | The remedy |
| **P2-5** the variants | **Accepted.** §4.10's table gives each against 18.0 m/s (the baseline carried the same 15 iterations) with "both feet left the ground" beside it, says differences under about 1 m/s are inside what an unsettled search moves, and restates the hips' run (the rear hip went to its new end, the pelvis turned 3° more: the planted feet first). The review's other candidates are listed | No baseline run was added; 18.0 is the effort rows' |
| **P2-6** corrections on main | Research note 11, line 5 is corrected (the job gave it to this worker). **PLAN D41's and D39's reasons are the Coordinator's files and are not touched here** | The Coordinator |
| **P3** "with the wrist unflexed" | Said in §4.10's table and in `body.py`'s comment | — |
| **P3** on which body the fit's wrist figures were read | Said (the body of sessions 1–7 at 0 ms on its clock), with the review's figures on the practice body beside them | — |
| **P3** three stale lines | Struck or marked | — |
| **P3** §13's "if the owner rules Q61 = b" | Now says it is ruled (D39) and that nothing measured stands behind 50 N·m; the size is the next session's | — |
| **P3** the passivity check's tolerance | Not changed; noted here. The engine's test asks the booked work plus the stored energy to stay under a thousandth of the starting energy on the lone rod | A stated tolerance for multi-body runs, if wanted |

**The order of what is left** is the review's and the job's, unchanged: each hand's own grip; how a tensed muscle holds, then the clock's question; a rear foot that can pivot or a knee that can twist; PLAN D39; the score's parked basin; a start that owes the lab study nothing; then the practice runs and the gate.
