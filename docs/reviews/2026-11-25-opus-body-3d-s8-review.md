# Independent review: phase 1, session 8 of the force-driven body (the practice search)

Reviewer: rev-body-3d-8 (Claude Opus 5.5, high effort), 2026-11-25. Range `5488127..d164b96` on `work/body-3d` (not merged). Job: `REVIEW-BODY-3D-8.md`. I edited no source, test, spec, note or TASKS file. My probes and their output are in `sim/target/codex-probes/review-body-3d-8/` (`p1`–`p13`, each `.py` with its `.out`). Nothing was compiled: the probes use the session's own `body3d_probe` binary and the Python engine. One probe (`p6`, `p13`) runs a patched copy of the Python engine, `engine_fixed.py`, which lives in that directory and is not part of the code.

**Terms used below.** A *stop* is the engine's end-of-range spring (`engine.System.stop`). It reads one angle and pushes back once the angle passes a limit. A *ball joint* turns freely in all three directions (the shoulders, the hips, the lumbar joint between pelvis and trunk). A stop is *conservative* when the energy it gives back is never more than the energy it took in. A *swing* below means a run whose sweet spot crosses the ball's plane.

**Verdict.** The session did a lot of good work. It wrote down the aim, the ranges and the checks before running anything. It found, and says plainly, that the earlier swings were not swings a body could make. The main result reproduces exactly, and the tests pass. **But two of the owner's three causes, and the mechanism given for the 30.6 m/s, rest on two defects in the code that the session did not see. Q61 and the order of work in Q63 are built on them.**

1. **The stops on ball joints create energy** (P1-1). This is an engine defect that dates from the first shoulder stop. A stop pushes about a fixed axis but reads an angle projected onto a plane, so it can do net positive work with no energy stored to account for it. The 30.6 m/s was not a bounce. In that swing the lead shoulder's stop put **+414 J** into the body, while the bat had 409 J at contact and the muscles did 346 J. With the stop made conservative (same stiffness, same damping) the same plan gives **15.6 m/s**. Every practice run of session 8 was searched on stops that create energy: start A +4,300 J, start B +1,900 J, start D +1,170 J, the main result +117 J, the one swing +139 J.
2. **The wrists' travel is placed on the wrong side** (P1-2). In this model the swing's uncocking *raises* the cock angle. `add_ranges` puts the start 5° from the *upper* end, so each wrist can uncock only about 5°. All 53° of its travel lies in the cocking direction, which the swing never uses. Placed the other way round, the fit to [4] held to the wrists' 58° reaches **15.1 m/s**, not 7.5, and the speed-only swing **16.3**, not 8.7. With no stops they reach 18.7 and 17.5. So "this body's hands cannot turn the bat" is not shown, and every practice run had wrists that could hardly uncock.
3. **"The wrists folded through about 180°" is a reading artefact** (P1-3). The cock is read from the forearm's projection, which flips by 180° when the wrist's flexion passes 90°. Read on the joint's own axis, the cock travelled 93° and 127° (the fit to [4]) and 115° and 79° (the speed-only swing). That is still past 58°, but not three times it. The stop uses the same reading, so it jumps by thousands of N·m (the "lead wrist's [stop] at 3,160" of the one swing, the 9,041 N·m of the which-stops run).

What stands: the main result is not a swing, and it is not one with conservative stops either (`p6`). The high-and-close swing meets its ball at about 15.6 m/s with either stop law. The earlier fits did swing at 1.0–1.2 m with the hands at 1.35–1.52 m. The rear thigh's 70° behind the pelvis is a correct reading. The ball's placement matches PLAN D28/D29. D37 is held to: nothing of [4] is a target, gate or penalty. What the owner is told needs rewriting before he rules. Q61 should be withdrawn until the stop law is fixed. The fix is a bug fix and needs no decision from him. Q63's first item should be the two fixes and a rerun, not the hands.

## P1 findings

### P1-1. The engine's stops on ball joints create energy; the 30.6 m/s was that, not a bounce

**Where:** `analysis/oracle/body3d/engine.py:515–533` (the stop law). It is read by §4.10 step 2, docs/body-motion.md:1109–1121 ("A spring at the end of a joint's range is a trampoline"), :1155 ("the earlier fast swing was a bounce off a spring"), :260 and :270–274 (the owner's Q61), the research index's new first dead-end line, and §13's proposed gate (:1620).

**What is wrong.** The stop reads θ, the angle of a direction fixed in body b (f) projected onto the plane square to an axis n fixed in body a. It applies a torque −k(θ − limit) about n. The power it delivers is τ·(ω_b − ω_a)·n. But the rate at which θ changes is not (ω_rel·n). Working through the projection gives dθ/dt = ω_rel · (n − (n·f) f_p / |f_p|²), where f_p is f's projection. The extra term is the out-of-plane tilt times any turning of f about its own projection. On a hinge (the knee), or on a universal joint whose axis is n (the elbow, and the wrist's cock), the extra term is zero and the stop is conservative. On a ball joint it is not. A limb tilted out of the stop's plane is spun about its own length by the stop's torque, and the stop is booked as doing that work. Nothing stored pays for it.

**How I know.**

- *A rod alone* (`p5_stop_toy.py`, the engine's own Python code, RK4 at 0.1 ms). A 2 kg rod on a ball joint, no gravity, no damping, one stop of this kind, turned into the stop. In the stop's plane its energy stays at 1.485 J. Tilted 30° below the plane it goes from 1.23 J to **269 J** in 45 ms. Tilted 60°, from 0.45 J to **860 J**. With the torque applied along the gradient above (`engine_fixed.py`, one change), both stay at 1.230 and 0.445 J to four figures.
- *The speed-only swing* (`p1`, `p2`, the body it was found on, 0.125 ms). At contact (bat azimuth −8°) the bat has 409 J. The drives did 346 J and the stops' booked work is **+283 J** (+474 J to +60 ms). The lead shoulder's stop alone did **+414 J**. As a spring on its own read angle it would have taken in 134 J and given back 49, a net *loss* of 85 J. So this was not a bounce: a bounce cannot return more than it took in.
- *The same plan with a conservative stop* (`p6`, Python engine, same k = 2,000 and c = 20, at 0.5 and 0.125 ms, identical): **15.6 m/s**, stops −8 J, drives 454 J. With the engine's own law and its damping raised 50× so that the stops' net work is −1 J: 23.0–23.5 m/s at contact. Either way, the 30.6 was made by the stop law.
- *The fit to [4]* (§4.9's 21.3 m/s) used +72 J from the shoulder stops (lead +46, rear +31). With a conservative stop it is **17.8 m/s at contact**. So §4.6–§4.9's bat speeds, all run with the shoulder stops, were raised by this too.
- *Session 8's practice runs* (`p4`, each saved result rerun at 0.5 and 0.125 ms, work to contact). Main result: drives 192 J, stops **+117 J** (the rear hip's flexion stop +186 J). The high-and-close swing: drives 174 J, stops **+139 J** (the trunk's side stop +75, its twist +59), and **+1,584 J by +100 ms**, where the sweet spot reaches 36.8 m/s. Start A: drives −1,113 J, stops **+4,178 J**. Start B: stops +1,910 J. Start D: stops +998 to +1,174 J. Start A's "trunk's side stop carries 10,600 N·m, hips at 2,500°/s, both feet leave the ground" and start D's "rear shoulder's stop at 42,000 N·m" are what an energy source looks like. The search was feeling its way across a score shaped partly by energy the stops created.
- *Why the engine's checks did not see it.* `test_work_against_energy` books each stop's τ·ω as work, so the energy balance closes whatever the stop does. No test asks that a passive element never do net positive work.

**Consequences for what the owner is told.**

- "Bouncing back like a ball off a wall" (:260) and Q61's premise that "a spring gives back everything it is given" (:270) describe the wrong mechanism. A conservative spring at the same stiffness gives 15.6 m/s. Making the ends absorb energy (Q61 b or c) without fixing the law still leaves the energy source in place: with damping at 10× the old value the stops still gave +38 J.
- "With nothing there to bounce off, that swing is 39 mph" (17.5 m/s) is not a property of the swing either. On the body the plan was found on (`fit3d.Ctx`), the same plan with no stops gives **11.0 m/s**. 17.5 comes only from `practice3d.Ctx`, whose hold-still torques at the arms differ by a few N·m because the old lead-shoulder stop was pressed at the start (`p3`). A plan tuned with a stop says nothing stable without it.
- The conclusion that §4.9's 30.6 says nothing about strength is *right*, and more firmly than stated.

**Fix.** In `System.stop`, apply the torque and its damping along g = n − (n·f) f_p / |f_p|² and book the power with the same g. That is the generalized force of the read angle, and it is conservative. Then add a check that can fail: a passive stop on a ball joint, with no other force, never ends with more energy than it started with (the rod above). g is singular where f lies along n. For the shoulders and hips that is far from any posture in a swing. For the wrist's cock it is exactly the 90° flexion flip of P1-3, so fix the reading first. Add "no passive element does net positive work" to §13's "engine's checks". Rerun the which-stops table and the main run. Add one line to §4.6–§4.9's history saying their bat speeds include stop-made energy (the fit to [4]: 21.3 → 17.8). **This is a bug fix, not an owner decision.** Q61 should be withdrawn until it is done. After the fix, how much a joint's end absorbs is a smaller question and may still be worth asking.

### P1-2. The wrist's travel is on the wrong side of the start: "the hands cannot turn the bat" is that sign

**Where:** `analysis/oracle/body3d/body.py:112–117` (the comment "positive is toward the barrel (radial deviation…)"), `:123` `WRIST_COCK_INSIDE`, `:147–150` (`hi = a0 + 5`, `lo = hi − 58`). docs/body-motion.md:1062 ("the start is 5° inside the end toward the thumb"), :1122 ("Held to a wrist's real travel, this body's hands cannot turn the bat"), :1152 (obstacle 1, "the first thing to build"), :265 (the owner's "his hands"), and the research index's "Both hands welded…" dead-end line.

**What is wrong.** Through the fit to [4], as the lag falls from 122° to 66° (the bat uncocking), the cock angle *rises*: lead wrist −6° → +71°, rear −66° → +53° (`p10`). The stops are lead −59.2…−1.2 and rear −118.8…−60.8 (`p10`, first block). So each wrist may uncock 5° (2° of that is the clearance) and has 53° of travel in the cocking direction, which the swing never uses. Whatever the comment's anatomy, the code puts the travel on the side opposite the swing.

**How I know** (`p11_wrist_mirror.py`, `practice3d.Ctx` as session 8 ran the table, 0.125 ms):

| Stops | The fit to [4]: fastest / at azimuth −8° | The speed-only swing |
| :-- | :-- | :-- |
| none | 18.7 / 16.5 | 17.5 / 16.4 |
| the wrists, as placed | **7.5** / 7.0 (the table's 7.5) | **8.7** / 4.5 |
| the wrists, travel mirrored (start 5° inside the lower end) | **15.1** / 12.5 | **16.3** / 14.5 |

Held to 58° with the travel where the swing needs it, the old plans lose 2–4 m/s, not two thirds of their speed. Those plans were tuned with no wrist stop at all.

**Consequences.** Obstacle 1 and its place at the head of the list are not supported. The weld may still need work: the cock still travels 79–127° in the old swings by the joint's own axis (P1-3), and the forearm's turn is unheld (P2-3). But the evidence offered for it is the sign error. Every practice run, the main run and the one swing included, ran with wrists that could barely uncock. That alone may be enough to explain why no start became a swing. The research index's dead-end line would stop the next worker from trying the welded hands with the travel placed correctly.

**Fix.** Place the travel from the direction the cock moves as the lag falls (`hi = a0 + 53`, `lo = a0 − 5` in this model's sign), or read the sign from the placed state as `drive.add_drives` does for the wrist drive. Correct the comment. Add a test that fails if the swing's uncocking direction has less than half the travel: for example, turn the bat about its y to reduce the lag and check the angle moves toward the end that has the travel. Rerun the which-stops table and the main run. Withdraw the dead-end line.

### P1-3. "The wrists folded through about 180°" is a reading artefact, and the same reading makes the cock stop jump

**Where:** `body.py` `range_specs()` `wrist_dev` (axis bat y, e = bat −x, f = forearm z) and `range_angles`. docs/body-motion.md:1036 (178°, 178°, 185°), :258 ("about 180°, where a wrist has 58°"), :1155 and the research index ("three times their travel").

**What is wrong.** The wrist is a universal joint. Its cock turns the forearm's x about the bat's y, and its flexion then turns the forearm about its own x, which in this weld lies along the handle. The reading projects the forearm's *long axis* onto the bat's x–z plane. Once flexion passes 90° from its zero, that projection points the other way and the reading jumps by 180°. The forearm's x is held square to the bat's y by the joint, so its angle about the bat's y is the cock itself, with no flip.

**How I know** (`p7_wrist_reading.py`, the old body, to +50 ms):

| | Cock as read: travel | Cock on the joint's own axis: travel | Flexion: start → most |
| :-- | --: | --: | :-- |
| fit to [4], lead | 179.6° | **92.7°** | 18.6° → 113.7° |
| fit to [4], rear | 180.6° | **126.9°** | −3.2° → 127.3° |
| speed-only, lead | 186.0° | **114.6°** | 18.6° → 133.7° |
| speed-only, rear | 79.3° | 79.3° | −3.2° → 72.0° (never past 90°) |

The one wrist whose flexion stayed under 90° reads the same both ways. The engine's cock stop uses this reading. With the lead wrist's flexion stop reaching 94.1° (`p10`), the flip happens in runs too. That is where "the lead wrist's at 3,160" (the one swing's flexion stop, after contact) and the which-stops run's 9,041 N·m at 0.5 ms (1,090 at 0.125) come from. The record-based energy of the cock stop shows 1,900–2,350 J "taken in" in those runs (`p1`), which is impossible for a wrist. With the conservative law of P1-1 the cock stop is also singular at the flip, and the all-ranges speed-only run then fails (`p13`).

**Fix.** Read the cock as the angle of the forearm's x about the bat's y (no flip). Keep the flexion reading, which is exact. Restate the old swings' wrist travel as about 80–130° by the joint's own axis, 1.4–2.2 times the range, not "about 180°" or "three times". The conclusion that the earlier swings overran the wrists still stands.

## P2 findings

### P2-1. "His reach" is not shown: this body brought the sweet spot to the ball; it did not swing through it

**Where:** docs/body-motion.md:1123, :1153 (obstacle 2), :261 and :266 (the owner's "every one of them swung at shoulder height… No pitch for a strike is there"; "his reach").

**What is wrong.**

- The main result's sweet spot ends 4.7 cm high, 5.1 cm along the bat and 11.8 cm short of the ball's plane, about 14 cm from the ball (`p9`; the spec's "within 5 cm" leaves out the 11.8). The top-of-the-zone run ended 0.9 cm and 4.1 cm off and just short. So the body can get the sweet spot to the ball's place. What it does not do is arrive there moving along the ball's line: the main result's attack angle is −59°, the bat coming down onto the ball.
- The "45 cm below and 20 cm beyond" figure is where the *old* swings went, not a limit of this body.
- The two diagnoses moved the ball's height, its distance across and its distance ahead all at once (1.20, 0.75, 0.40 against 1.05 or 0.78, 0.92, 0.72), so they cannot say which coordinate matters.
- "No pitch for a strike is there": the speed-only swing's sweet spot was at 1.02 m (confirmed, `p8`), and the session itself calls 1.05 m the top of the zone.
- "Every one of them" is two swings.

The checks themselves hold up: frames and signs are right (`p8`: the plate is +X of the hips, the ball's line points toward first base and down), and the ball's 0.92 m across and 0.72 m ahead match [note 11](../research/swing/11-stance-depth.md) and PLAN D28/D29, with the stride question flagged as [E]. The old swings' hands stayed at 1.52 m and 1.35 m from a start at 1.53–1.59 m. The trunk really has no bending drive.

**Fix.** Say the old swings went at about 1.0–1.2 m, the top of the zone and above. Move "reach" from a cause to a thing to test, after P1-1 and P1-2: one coordinate at a time (height alone at 0.78 m with the old across and ahead; ahead alone). Add a cheap check of whether the bat can pass through the ball's place while moving along its line.

### P2-2. The one swing that met a ball left the ground and overran unheld joints; this is not reported

**Where:** docs/body-motion.md:1145 and :1156.

**What is wrong** (`p9`, `p12_show_highclose.out`). The saved result's balance terms show the feet carrying **0.00 of body weight** at some instant (a jump; the aim's jump term is violated), and its heading 9.1 cm outside the feet. The unheld joints travel past what a body has: the rear forearm's turn 230° of 162°, the lead upper arm's turn about its own length 177° of 173°, the ankles 102° and 124° of 67°. At contact the hands travel at 2.0 m/s and the bat's turn about them gives 13.5. The table gives "the front foot at 1.0 times body weight" and the stops' loads, but not these. The lead upper arm's turn about its own length is exactly the motion P1-1's shoulder stop pumps.

**Fix.** Add these to the row. The speed figure itself is solid: it is 15.5 m/s at both steps, and 15.6 with a conservative stop (`p6`).

### P2-3. The follow-up runs stopped unsettled, and the score's largest term at the main result is the arms' clock read at a contact that never happened

**Where:** docs/body-motion.md:1139–1147; `practice3d.py:226–244`.

**What is wrong.**

- Settledness is reported for the main run but not for the follow-ups:
  - the top-of-the-zone run stopped after 9 iterations, still falling 96 over its last 5;
  - the small frame was still falling 46 at its 20-iteration limit;
  - the strain run stopped after 5 iterations, falling 143;
  - the strongest batter stopped after 11.
  
  "Never reaches it" in those rows is where a search stopped, not a property of the body.
- The score at the main result (`p9`, total 522.5): the arms' clock **158.6**, never coming round 139.6, balance 62.7, timing 66, speed 43.7, miss 47.9, effort 0.6. When the bat never comes round, `tx` is the run's end (+100 ms), so the clock (`tx − 0.253 − first arm onset`) is charged against a contact that did not happen. That is a second cliff beside the one the session names. At start D the clock is 755 of 2,704.

**Fix.** Report each follow-up's iterations and its fall over the last 5. When the bat does not come round, take the clock against the latest contact the aim allows, or leave it out. The session's own plan to stage the search (first reach the ball, then ask for speed) is the right next step, but only after P1-1 and P1-2.

### P2-4. The arms' clock does not hold an arm muscle that starts tensed

**Where:** `practice3d.py:243` and `:450` (the clock looks only at onsets); docs/body-motion.md:1028 ("no arm muscle rises above its holding level more than 0.253 s before contact").

**What is wrong.** Each arm channel's starting level is free from 0 to 1 (`Ctx.box` "start"). The hold-still torques are solved once with every drive at zero (`Ctx.__init__`) and not again for the plan (`fit3d.Ctx.prepare`). So a starting level is net drive from the first instant, outside the clock. In the main result the lead shoulder's sweep is at 0.24 of 81 N·m (about 19.5 N·m) and the lead wrist at 0.33 of 47 N·m (about 15.6 N·m) from −250 ms (`p12`, the JSON's plan). "Holding level" is therefore not a holding level.

**Fix.** Re-solve the hold-still torques with the plan's starting levels in place, so a tensed muscle only holds. Alternatively, count a starting level above what holding needs as an arm drive under the clock.

### P2-5. The two new tests do not look where the defects are; the which-stops table cannot be rerun from the repository

**Where:** `analysis/oracle/test_body3d.py:849–891`; docs/body-motion.md:1109–1119.

**What is wrong.**

- *The tests.* The first test compares the knees' and elbows' readings with `observe()`'s (independent, good) and the trunk's twist within 1.5°. The rest restates the code (19 stops, the clearance, the 58° span). The second test checks a stop's sign on the knee, a hinge, where nothing can go wrong. None would catch P1-1 (ball joints), P1-2 (the travel's side) or P1-3 (the flip).
- *The table.* The script that made `practice3d_which_stops.out` is not committed, so the table cannot be rerun. It also mixes two bodies and two measures. Its first row (21.3, 30.6) is `fit3d.Ctx`'s speed at contact. The other rows are `practice3d.Ctx`'s fastest speed anywhere to +60 ms, often 25–45 ms after contact ("no stops, 17.5 at +47 ms"; the bat is square at +23 ms, where it has 16.4). On `fit3d.Ctx` the "none" cells are 10.3 and 11.0, the "wrists" cell 14.8, and "hips" never comes round (`p1`).

**Fix.** Tests as in P1-1 to P1-3. Commit the table's script. Use one body and one measure (speed at contact) throughout, and say how fragile the single replays are.

### P2-6. The research index's new dead ends carry P1-1 to P1-3's conclusions

**Where:** `docs/research/body/README.md`, session 8's seven lines.

**What is wrong.** Three of the seven lines record P1-1 to P1-3's conclusions: "a bounce" (P1-1), "178°" (P1-3), and "both hands welded… 7.5 m/s" (P1-2). Dead-end lines exist so nobody retries the thing. These would steer the next worker away from the welded hands with correctly placed travel, and toward absorbing joint ends as the cure. The line on stiff springs (the motion jumps; 37 vs 46 m/s) is also partly P1-1.

**Fix.** Rewrite or withdraw them in the same change as the fixes.

## P3 findings

- **The hip readings, against the text's caveat** (docs/body-motion.md:1044). A thigh's turn (`hip_rot`) as the stop reads it is, worked through, the same angle as the ISB joint coordinate system's rotation. The flexion reading is the ISB flexion. So the caveat "a thigh's turn … over-read when the hip is bent a long way" points at the wrong angle. The one that over-reads is the *sideways* angle (`hip_abd`, a projection onto the pelvis's frontal plane), by about 1/cos(flexion) in tangent: 30° of true abduction reads 39° at 45° of flexion. The lead hip starts at 37.9° as read, past the 38.8° end once its stop is moved to clear the start. The rear thigh's 70° behind the pelvis is therefore a correct reading. The hips-only cell holds at 13.9–14.1 m/s with a conservative stop (`p13`), though it is still a single replay of a plan tuned without hip stops.
- **The arms' clock's viewing time.** 0.10 s [E: 0.05–0.15] excludes the game's own batter, who watches a 95 mph pitch for about 0.2 s before committing (agent-models §3.4, which §4.10 cites). The clock's arithmetic (0.403 − 0.10 − 0.05 = 0.253 s) is right, and it is the pitch's clock, not [4]'s. Measuring it from contact, a result, is the batter's intent and does not break the causality rule. But the aim has no arrival time for the ball, so the clock is the only link to the pitch. Say so, and say why the game's own 0.2 s is outside the range.
- **The proposed gate's bands** (docs/body-motion.md:1617). The hips and trunk use mean ± 2 SD, the lead knee mean ± 1 SD (14 ± 12 → 2–26°), and the attack angle and front foot use the 10th–90th percentiles. Use one rule or say why. The "engine's checks" need P1-1's passivity check.
- **Step size.** The which-stops "all" cell for the speed-only swing is 40.3 m/s at 0.5 ms and 49.5 at 0.125 ms. The table prints 49.5 with no note. The rest of what is reported as a result converges: the one swing 15.50 / 15.48, the main result's miss 4.7 / 4.5 cm.
- **The ranges' reading statuses** (docs/body-motion.md:1046–1060) come from a search worker's report that was not saved as a note, so they cannot be checked from the repository. I confirmed Soucie's men 20–44 for hip flexion 130.4°, hip extension 17.4° and knee flexion 137.7° from the CDC's archived table ([Joint Range of Motion Study](https://archive.cdc.gov/www_cdc_gov/ncbddd/jointrom/index.html), via a search summary). I did not reach the elbow's 144.6°, Roaas & Andersson, Boone & Azen or Fleisig 2013. Save the report as a research note, under the research-job habit of recording readings.
- **Stale text.** Four places still say things that are no longer true:
  - §12 (docs/body-motion.md:1593–1608) still lists "the 3D body's fit to [4] and [3]", "the arms' earliest onset relative to [4]'s first move", "the remaining angular impulse that [4]'s lateral couple does not give", "how much the impact's smoothing took off [4]'s bat curves" and "the hands' path and speed in [4]'s swing" as open questions. Under D37 they no longer bear on the gate. Mark them as history or drop them, and add P1-1's open item.
  - §1's "How a swing happens" (a drive fitted to [4] reproduces their mean swing) and its breakdowns table are the flat probe's fits to [4], with no D37 mark.
  - "Status and what is left" (:9 and :18) repeats P1-1 to P1-3's conclusions, and its work order starts with the hands.
  - §4.4's "In one paragraph" says "the same limits allow 30.6 m/s only with a swing no hitter makes".

## For the owner: what each sentence of "For the owner" (docs/body-motion.md:254–281) should become

- "the wrists folded through about 180°": *about 80–130°*, still past a wrist's 58° (P1-3).
- "the back leg trailed straight out 70° behind the hips": correct as a reading. "Straight out" goes further than the reading shows.
- "the 68 mph came from the front arm … bouncing back off it like a ball off a wall. With nothing there to bounce off, that swing is 39 mph": *the 68 mph was energy the program's joint stop created, not the batter's. With the stop working properly the same swing is about 35 mph* (15.6 m/s; P1-1).
- "every one of them swung at shoulder height … No pitch for a strike is there": *both swung at 1.0–1.2 m, the top of the zone and above, with the hands at the shoulders* (P2-1).
- "He does not yet have a swing … from any of four starting points": true of this code. Add that the search ran on two defects (P1-1, P1-2), so it was not yet a fair test.
- "Why … his hands": withdraw (P1-2). "His reach": make it something to test (P2-1). "The search": stands, after P1-1.
- **Q61**: withdraw. The stop law is a bug to fix and needs no ruling (P1-1). It can be re-asked afterwards if the absorbing end still matters.
- **Q62**: fair as put, with P1-1's check added to the engine's checks and the band rule made consistent.
- **Q63 (a)**: the size looks reasonable, but the order should start with fixing the stop law, the wrists' travel and the cock's reading (P1-1 to P1-3, one session with their tests) and then the practice runs again as written. The hands' grip, the trunk's bending drive and the turning foot come after that, if the reruns still point at them. He would also want to know that §4.6–§4.9's bat speeds were raised by the same stop defect (the fit to [4]: 21.3 → 17.8 m/s).

## Verification

What I ran, all under `sim/target/codex-probes/review-body-3d-8/` unless named:

- `python3 -m unittest analysis/oracle/test_body3d.py`: 32 tests, OK (16 s).
- `python3 -m unittest analysis/oracle/test_oracle.py`: 106 tests, OK (8 expected failures).
- `python3 analysis/research/body/probe3d.py --check`: PASS (8.6e-12).
- `python3 bin/check-docs.py`: its only complaints were the two references in `REVIEW-BODY-3D-8.md` to this file, before it existed.
- **Old behaviour unchanged** (`p1`, `p8`). `B.Model()` keeps sessions 1–7's six stops (the test checks this). The two old plans on the old body give 21.2 and 30.7 m/s at contact at 0.125 ms, against §4.9's 21.3 and 30.6.
- **The which-stops key cells** (`p2`, `p11`, `practice3d.Ctx`): no stops 18.7 and 17.5; the shoulders' stops only 20.2 and 48.0; the wrists only 7.5 and 8.7; all 11.6 and 49.5. These match the table. The hips-only cell was not rerun on that body with the engine's own law; on the old body it never comes round (`p1`). With a conservative stop it is 13.9 and 14.1 (`p13`).
- **The main result** (`p12_show_main.out`, `practice3d.py --show` with the probe's temporary files moved here): the bat never reaches the ball, the miss is +4.7 and +5.1 cm, the sweet spot 12.1 m/s. These match the JSON and §4.10. The high-and-close swing: 297 ms, miss +0.1 cm, 15.5 m/s.
- `p1`, `p2`: energy accounts of the old plans under each set of stops. `p3`: why `fit3d.Ctx` and `practice3d.Ctx` differ (hold-still torques at the arms). `p4`: energy accounts of session 8's saved results. `p5`: the lone rod. `p6`: the old plans and two practice results in the Python engine, with the stop law as is and conservative, at two steps. `p7`, `p10`: the wrist readings and the direction of uncocking. `p8`: the start's geometry and the old swings' sweet spot. `p9`: the score term by term. `p11`: the wrists' travel mirrored. `p13`: the ranges with a conservative stop.

**What I could not check, or checked only in part.**

- `engine_fixed.py` is a diagnosis, not a proposed implementation. It is singular at the wrist's flip, so its "all ranges" cells (`p13`: 3.7 m/s, and a failed run) are not results.
- I did not re-search anything. Whether a swing exists once P1-1 and P1-2 are fixed is unknown. All I show is that the reasons given for its absence do not stand.
- Of the ranges' sources, only three CDC figures were checked, through a search summary rather than the page itself.
- I did not check the hip-rotation and abduction stops' clinical zeros against the pelvis frame beyond the start's angles.
- I did not check the strongest-batter bounds against §4.5.

## Resolution

Open.
