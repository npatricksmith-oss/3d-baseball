# Review: the swing written by hand and made by muscles (PLAN D48; body-motion §4.17)

Reviewer: Opus, independent (wrote none of it). Branch `work/swing-look`, commits dfd9332..c22e484 against `main`. Brief: `REVIEW-swing-look.md`. Nothing here changes a game.

## Summary

The main claims hold. The swing is really made by force. In the forward run nothing moves the body except muscle torques within their stated limits, the joints' own friction, the joint ends and the ground. The ground only pushes, only under the soles, and never grips harder than its friction allows. The energy accounts close to the joule. Nothing later in a run changes anything earlier. No code reads the recorded swing. 52.1 mph, 5 ms late, and the finish within 0.3 cm of the written one all reproduce from the committed plan, and all 13 tests pass. In the pictures it reads as a hitter from foot down through the wrap, and §4.17's own account of it is honest.

Three things in the write-up are wrong or missing.

- **The claim about strength is not shown (P2-1).** §4.17 says the 1.5× strength "buys the standing, not the speed", and the owner's next question rests on that. At the measured strength (1.0×), the kept plan keeps him on his feet but swings at 47 mph, 15 ms late. The 52-mph, falls-off-his-front-foot result comes from a second plan that was practised separately and not committed. A third run at 1.0× gives yet another answer. So what the extra strength buys is not yet known.
- **He does not start still (P2-2).** The text says he starts standing still. In fact his body is already drifting toward the catcher at 0.32 m/s at the first instant.
- **The finish is partly propped on joint ends (P2-3).** After contact the lead wrist hits the end of its range, which was already moved 73° out, with 261 N·m. A wrist cannot take that. Two more joint ends carry load in the finish. §4.17 does not say so.

## Findings

### P2-1. What 1.5× strength buys is not shown; the 1.0× figures come from an uncommitted plan, and runs at 1.0× disagree

`docs/body-motion.md:3889` (the strength row), `:3901` ("at 1.0 and at 1.5 times the bounds it is 52.1 mph both times"), `:3920` (the owner's strength question, "1.5 times the measured bounds is what keeps him on his feet today").

What I ran: `sim/target/review-probes/probe_strength.py` (the kept plan, unchanged, run forward at other strengths) and `probe_m.py`. I also read the worker's plans in `sim/target/codex-probes/swing-look/`.

| Run | Bat at contact | Late | Front / rear foot at contact (× his weight) | Off the written finish |
| :-- | :-- | :-- | :-- | :-- |
| Kept plan, 1.5× and hips 1.6× (as committed) | 52.1 mph | 5 ms | 1.55 / 0.48 | 0.3 cm |
| Kept plan, 1.0× and hips 1.0× | 47.3 mph | 15 ms | 1.74 / 0.35 | 2.8 cm |
| Kept plan, 1.0× and hips 1.6× | 48.5 mph | 13 ms | 0.93 / 0.45 | 3.9 cm |
| `plan_m.json` (practised at 1.0×; not committed) | 52.1 mph | 10 ms | 0.00 / 1.08 | 14.5 cm |
| `plan_k_1.0_1.0.json` (another practice at 1.0×) | 47.1 mph | 21 ms | 0.23 / 0.59 | 8.4 cm |

§4.17's 1.0× figures match `plan_m.json` exactly. That plan lives only in `sim/target`, so nobody can rerun it from the repository. Another practice at 1.0× (`plan_k_1.0_1.0`) came out 5 mph slower. Run with the kept plan, 1.0× strength keeps him on his feet but costs 5 mph and 10 ms. So whether the extra strength buys standing or speed depends on which practice run you look at. Both "it is not strength" and "the loosening buys the standing" are overstated. This matters because §4.17's next step 5 sends exactly this question to the owner.

Fix: commit the plans being compared, or give the command that remakes them. Report the spread across practices. Reword the strength row and next step 5 to say what the extra strength buys is not yet settled.

### P2-2. He does not start still: his centre of mass is already moving toward the catcher at 0.32 m/s

`docs/body-motion.md:3895` ("He starts standing still in a stance written by hand") and `:3852` ("He stands still on both feet"). Code: `swing_author.py:507` (`two_sided`) and `swing_track.py:248` (`start_state` takes the written motion's velocities at its first sample).

What I ran: `sim/target/review-probes/probe_com.py`. In the written motion and in the forward run alike, at −1400 ms his centre of mass moves toward the catcher at 0.32 m/s, checked against its change of position. It is still at 0.23 m/s at −1050 ms. Through the "stance" he drifts 9 cm toward the catcher. The pelvis starts at 0.31 m/s. The table (`swing_track.py <plan> --table`) shows this from its first row. The ground's vertical push also swings 0.77 → 1.19 → 0.91 → 1.26 times his weight over the first 40 ms while the start settles.

The cause: the written path of his centre of mass is solved with a two-sided filter. In plain terms, the plan leans him ahead of time for every push written later. The load toward the rear foot starts at −900 ms, and its effect reaches back to the first sample. That is a plan anticipating, and it does not break the owner's rule inside a run. But the start is not at rest, and the tests already know it: `test_swing_look.py` says twice that "the stance is not quite still (he is already easing back)". It cannot be seen at the pictures' size.

Fix: correct the two lines. In "How it was made" step 2, say that the written path of his centre of mass is worked out from the whole written push, earlier and later alike. Either start the written motion earlier, so that he really is at rest, or say plainly that he starts already easing back.

### P2-3. After contact the follow-through leans on joint ends: the lead wrist takes 261 N·m against an end already moved 73° out

`docs/body-motion.md:3892` (the joint-ends row) and `:3859` (the wrap and finish). Code: `swing_track.py:149` (`add_stops`).

What I ran: `sim/target/review-probes/probe_forces.py`, which reads each joint end's torque in every record of the kept run.

- **Lead wrist's tilt:** this end was moved out 73° to 105.5°, because the written motion passes it after contact. The run hits even that moved end at +39 ms, with **261 N·m**. It touches it again on and off until +693 ms, 54 ms in all.
- **Rear upper arm's turn:** 55 N·m, +241 to +293 ms.
- **Lead hip's turn-in:** this end was moved 22°. It carries up to 27 N·m for 288 ms of the finish (+423 to +710 ms).
- No joint end carries anything before contact.

PLAN D39 says joint ends absorb and a load on them is avoided, and 261 N·m is far beyond anything a wrist could bear. It does not touch bat speed. But the extension and the wrap are partly held up by joint ends, not muscles, and §4.17 does not say so. The 82° gap between the arms and their written path after contact (§4.17's own figure) is part of the same story.

Fix: add this to "Where the body cannot do the written motion" and to the joint-ends row.

### P3-1. The fault at a 0.5 ms step: the figure is off, and the cause is this change's own foot-roll muscles

`docs/body-motion.md:3883` ("At the earlier sessions' step of 0.5 ms … by contact 190 J had appeared from nowhere").

What I ran: `probe_forces.py 0.0005` and `probe_h.py <roll strength> <step>`.

| Step | Foot-roll muscles | Unexplained energy at contact | Bat |
| :-- | :-- | :-- | :-- |
| 0.5 ms | 50 N·m (as committed) | **554 J** | 51.5 mph |
| 0.5 ms | 5 N·m | 0.0 J | 51.9 mph |
| 0.25 ms | 50 N·m | 0.0 J | 52.1 mph |
| 0.1 ms | 50 N·m | 0.00 J | 52.1 mph |

- With the kept plan the gap is 554 J, not 190.
- The motion itself barely changes: the energy he actually has at contact is within 5 J of the 0.1 ms run. What goes wrong is the bookkeeping. The foot's step-to-step flip feeds in energy that its own braking then takes out.
- The cause is the foot-roll muscles this change added (`swing_track.py` `ADDED["ankle_roll"]`). Weaken them and the 0.5 ms run closes too.

**It has no bearing on earlier work.** Sessions 8 and 9 ran at 0.5 ms, but sessions 10 to 13 ran at 0.25 ms (`ladder3d.py:37`, `practice3d.py:1261`). No earlier body had foot-roll muscles. At 0.25 ms this body's accounts close too.

Fix: say it is the added foot-roll muscles, give the kept plan's figure, and say which sessions used 0.5 ms.

### P3-2. The wrist's range shape is a loosening the list does not name, and the lead wrist uses it at contact

`swing_track.py:149` (`range_specs(..., "box")`) against `docs/body-motion.md:3892`.

The forward run holds each wrist's two angles with separate box limits. §4.12's ellipse, which sessions 10 to 13 used, stops a wrist from being at both ends at once. The change is stated in a code comment but is not in §4.17's list of loosened rules.

Also, the wrists' ends were moved out for after contact, but they stay moved out before contact too. The run uses that room. In `sim/target/review-probes/probe_oval.py`, at contact the lead wrist is **12.7° past its measured tilt end** and 15° outside the ellipse. §4.17 names only the rear wrist (23° past) and the lead forearm (6° past in the written motion; 4.6° in the run) before contact.

Fix: add both to the list.

### P3-3. Two tests claim more than they check

All 13 pass. Most check against something independent: the engine's own joint rows, body.py's joint readings, the compiled run against the Python one, the solved plan held by the forward engine, the published order of hips then chest then bat. Three are weaker.

- `test_swing_look.py:179` `test_a_steady_torque_is_held_by_one_muscle_of_the_pair` retypes the level rule from `swing_track.py` inside the test and runs its own copy. If the code's rule changed, this test would still pass. That is the restating kind of test AGENTS.md rules out. It should call the code's own rule.
- `test_swing_look.py:219` `test_nothing_but_muscles_and_the_ground` checks that energy and booked work agree. It does not check that nothing but muscles and the ground acts. A hold or a constant torque added to the model would be booked in the same place as joint friction, and the test would still pass. My probe found none (`probe_com.py`: 0 drives, holds, constant torques and outside forces), and no muscle level above 1. Add asserts for exactly those to the test.
- `test_swing_look.py:119` `test_no_recorded_swing_is_read` searches for the lower-case words only, so it would miss "OBP" or "OpenBiomechanics". It is still right today: my own search with any case finds nothing.

### P3-4. Smaller points on the pictures and the launcher

- **The ball always meets the bat.** The drawn ball is put through wherever the sweet spot is at the run's own contact, so every picture shows a hit. §4.17 says "the pitch is a picture", which is fair. An owner watching should know he is seeing a hit drawn in, not a bat timed to a pitch.
- **The ball hides the body at foot down.** From −180 to −150 ms (sheet 14, top row) the ball passes the camera and covers the body in one of the three views, just as the front foot lands.
- **`--look` ignores `--tree`.** That only matters before merge: observations.md tells the owner to use `--tree` for unmerged work, and main has no `--look` yet.

## Looking at it (all 25 sheets read; 13 to 18 closely, and contact zoomed)

- **Stance (sheets 1–4).** Knees bent, bent well over at the waist, hands by the rear shoulder, bat up. It reads as a batter's stance, a stiff, crouched one. It is motionless; the 9 cm drift (P2-2) does not show at this size.
- **Load and leg lift (5–11).** Little to see. The lead heel and knee come up a little and the foot barely leaves the dirt. The hands hardly move. §4.17 calls the lift "a plain one", which is fair. It is smaller than most leg-lift hitters'.
- **Stride (11–14).** The front foot goes out while the chest stays closed. It reads as a stride.
- **Foot down to launch (14–15).** The best part. The front foot lands, and the hips open before the shoulders. The bat tips back and lies flat behind him, the rear elbow drops, and the rear heel comes up.
- **Contact (16).** Head down, barrel below the hands, rear heel up, belt buckle at the pitcher. From the front view the bat points straight at the camera, which is right for a bat square to the pitch. The front knee is visibly bent, as §4.17 says.
- **Extension (17).** The arms go out toward the pitcher and the bat comes up over the lead shoulder. The arms look shorter and more bent than a hitter's. The lead wrist is on its end here (P2-3).
- **Wrap and finish (18–25).** The bat wraps behind his head with both hands on it, the front leg is bent and the rear foot is on its toe. From about +300 ms to +790 ms he does not move at all: a statue, where a hitter would rise and relax. §4.17 says "held", which is honest. A taller finish is already its next step.

§4.17's own account matches what I see. I would add three things: the frozen finish, how small the leg lift is, and that the follow-through is partly held up by joint ends.

## Causality and recorded data

- **Inside a run.** The forward run reads only the fixed plan at the present instant: the written motion and the planned torques, read between neighbouring millisecond samples. It also reads the body's present state. Practice changes the plan only between runs, and keeping the best round is a choice between runs. Time is never stretched: contact is simply when the barrel comes square, and the file's clock is shifted to that instant for display.
- **In the plan.** The plan itself looks ahead. The written path of his centre of mass and every written curve are smoothed or solved using times on both sides. That is intent, not a run looking ahead, but it explains P2-2 and should be stated.
- **Recorded data.** No code reads the OpenBiomechanics files. A search of every new file for obp, c3d, openbiomech, 478, real_swing and Driveline, in any case, finds nothing. The authored timings are not the recording's. Toe touch is −215 ms and heel plant −165 ms here; the recording has −178 and −131. The top of the leg lift is −500 ms here, at 12 cm; the recording has −620 ms, at 17 cm. The 0.89 m between the ankles after the stride equals OpenBiomechanics' average over all 98 hitters (research note 6), not this one recording. It also sits inside Welch's 0.85 ± 0.12 m. So it does not break D48 ("no coordinate of the recording").

## Verification

All runs were in `sim/target` (probes in `sim/target/review-probes/`). The compiled forward run builds into `sim/target/codex-probes/swing-look/` as the code has it.

- `bin/job.sh` job 549 ran three checks:
  - `uv run --with numpy python3 -m unittest test_swing_look.py`: 13 tests, OK, 20 s.
  - `swing_track.py --check`: the compiled run matches the Python one to 3.6e-15.
  - `swing_track.py swing_look_plan.json --table`: 52 mph, 5 ms after the written contact, rising 8°, barrel 32° below level.
- `probe_com.py`:
  - The model has no drives, holds, constant torques or outside forces: 0 of each. It has 31 joint ends and 8 sole contacts.
  - The plan's options are strength 1.5 and hips 1.6.
  - His centre of mass's velocity agrees with its change of position, in the written motion and in the run.
- `probe_forces.py`, the kept run at 0.1 ms:
  - Energy at −200, −100, 0, +100 and +795 ms against booked work: unexplained 0.00 J every time. At contact: muscles +334 J, joint friction −113 J, ground −23 J, motion +198 J.
  - The ground's vertical force is never below 0. Its sideways force never exceeds friction (largest ratio 1.000). No sole point pushes from above the ground.
  - The largest muscle level is 0.99999.
  - Muscle torques reach at most 1.67× the measured held-still strength (the rear shoulder sweeping back, at +38 ms, while lengthening, where a muscle is stronger), inside the 1.5× bound and the lengthening allowance.
  - Joint-end torques over time, and the joint-end moves; these match §4.17's list.
- `probe_forces.py 0.0005` and `probe_h.py`: the step and foot-roll table in P3-1.
- `probe_strength.py` and `probe_m.py`: the strength table in P2-1.
- `probe_oval.py`: how far each joint goes past its measured end before contact, and the wrist ellipse readings.
- The viewer's own check, read from `client/Saved/Logs/BaseballClient-backup-2026.11.28-00.04.06.log`: `swing_look_plan.swing`, 2201 samples, PASS, 4.2069 mm and 1.538°. These equal the older rung 4 file's figures, apparently a fixed property of the figure rather than of this swing.
- I read every picture sheet `docs/img/swing-look/sheet_01.jpg` to `sheet_25.jpg`, and a 2× crop of sheet 16 around contact. I did not start the viewer.
- I read the code: the control and force loops of `swing_track.py` and `swing_track.rs`, the engine's force loop (`analysis/oracle/body3d/engine.py`), and `swing_author.py`'s keys and centre-of-mass solve. I also read the tests, the diffs of `bin/body-swing.sh`, the client, `docs/body-motion.md`, `docs/player-visuals.md`, `docs/research/body/README.md`, observations.md, TASKS.md and AGENTS.md.
- Not checked: "sixteen rounds brought the barrel from 17 ms late to 5 ms late"; the inverse solve's first plan beyond its test; the line drawings (`--draw`).

## Resolution

Resolved 2026-11-27 (Coordinator). Every finding was fixed by a Sonnet worker in c9089e6 and 329c519, as each "Fix:" line asked; no re-review, the fixes being text, three tests and the launcher (the Coordinator reran the 13 tests, the compiled-run check and the docs check before merging).

- P2-1, P2-2, P2-3, P3-1, P3-2, P3-4 (text): body-motion §4.17 corrected. What the extra strength buys is stated as not settled; no practice was rerun, and the two uncommitted plans are named with where they live.
- P3-3: the tests now call the code's own rule, assert no drives, holds, constant torques, outside forces or muscle level above 1, and search the recorded-data words in any case. Left: the compiled run keeps its own copy of the muscle-pair rule, covered only by the compiled-against-Python check.
- P3-4 (launcher): `--look` honours `--tree`; syntax-checked only, not run in the viewer.
- Not fixed here, carried to the next swing job (TASKS "Resume here"): the follow-through leaning on joint ends, the frozen finish, the small leg lift, the start not at rest, and what the extra strength buys.
