# Review: session 10 of the force-driven body (work/body-3d, `d9e76ee..9d4aa6d`)

Reviewer: Opus 5.5 (high effort), 2026-11-26. I wrote none of this. Brief: `REVIEW-body-3d-s10.md`. Range: the brief names `638b9db..9d4aa6d`, but `638b9db` is a `main` commit made after the branch forked at `d9e76ee`. The TASKS.md change that range shows comes from `main`; no session 10 commit touches TASKS.md, observations.md or PLAN.md. Probes and their outputs are under `sim/target/codex-probes/review-body-3d-10/`.

## Summary

What was built holds up. The wrist's ellipse is the right ellipse, and its stop is passive in both engines. The cleared start is what the rule gives, and the full rule rerun returns it. The evolution strategy is a correct separable CMA-ES. The kept 0.10 s swing replays as the spec says in both engines and at all three steps: 0.2 cm, 17.69 m/s, −3.6°, legs then trunk then arms. Every stop's net work is negative. Part 2's "the test did not decide" is the honest reading.

The account has three problems for the owner, and all three bear on "why 40 mph":

1. **The hold-still torques supply over a quarter of the swing's energy** (P1-1). They are constant torques with no strength-at-speed law and no clock. The spec never shows their work apart; it is lumped with the joint damping.
2. **The joints' damping eats almost two thirds of the muscles' work** (P1-2). It is an [E] with no source and no stated range, and it is not among the account's suspects. In an indicative replay of the same plan, a third of the damping gives a fastest sweet spot 3.2 m/s higher.
3. **"No muscle group at its full strength" is read as "strength does not hold it"** (P1-3). In fact nine of 41 groups sit at 92–97% of their bounds.

## Findings

### P1-1. The hold-still torques give the best swing 157 J, 27% of its positive work. No muscle law limits them, and no result reports them apart.

`drive.solve_tone` sets one constant torque per joint, fixed in the parent's frame, held for the whole run. Its size is 69 N·m at the lead knee, 60 at the rear knee and 47–63 at the hips and lumbar. It has no strength-at-speed curve, no onset and no rise; it is not counted as effort or under the clock. The engine books its work together with the joint damping, and the result prints only the sum: "damping and hold-still torques −108.8 J".

Measured on the kept 0.10 s result (`energy.py`, the probe at 0.25 ms, to contact; the ground's figure is what is left of the kinetic energy's change and matches the engine's own −45.2 J):

| | S, 0.10 s (the best) | S, 0.20 s | C, 0.10 s |
| :-- | --: | --: | --: |
| the drives (muscles) | +416.5 J | +287.5 | +278.6 |
| **the hold-still torques** | **+156.8 J** | **+157.4** | **+125.0** |
| the joints' damping | −265.7 J | −187.4 | −191.4 |
| the stops | −50.8 J | −56.0 | −56.1 |
| gravity; the ground | +62.2; −45.3 J | +96.9; −39.2 | +59.3; −30.1 |
| kinetic energy gained | 273.7 J | 259.2 | 185.3 |
| the hold-still torques' share of the positive work | **27%** | **35%** | **31%** |

- By joint, in the best swing: lead knee +58.1 J, rear knee +49.9, rear hip +39.6, rear ankle +10.2. **At the lead knee the hold-still torque does about twice the knee extensor's work (+58.1 against +30.5 J).** It goes on pushing at 69 N·m while the knee straightens at 870°/s, where a real extensor gives almost nothing. So "the front knee straightens hard" (§4.12, "For the eye") comes mostly from this torque. At the rear knee the hold-still torque's +50 J is opposed by the knee flexor's −25 J.
- In C at 0.20 s and in D at both viewing times the lumped figure is positive (+44, +20 and +40 J in their files): the hold-still torques outweigh all the damping.
- It is not a passive element, so §13's passivity check passes on its own terms. But it stands for muscle, and none of the muscle rules hold it. Three statements need qualifying: "nothing in the program adds energy" (For the owner), "the legs and trunk lead" at 0.20 s (35% of that run's input is these torques), and any statement that the swing stays inside every muscle's strength at its speed.
- This element dates from session 2, but its work was never measured apart. Session 10's result is the first that rests on it this heavily.

**Wanted:** every result reports the hold-still torques' work apart from the damping's, by joint. The last session's list should include a run where the hold lets go when the muscles start (or follows the strength-at-speed law), as one of the single changes.

### P1-2. The joints' damping takes 266 J, 64% of the muscles' work. It is an [E] with no source, and an indicative replay says it is worth several m/s.

`drive.JOINT_DAMP = 1.0` N·m·s/rad on every axis of every joint, capped at 30 N·m. It has no source and no range (`drive.py:120`, §4.4: "[E], as in session 1"). In the best swing, to contact, it dissipates 265.7 J against the muscles' 416.5 J and a kinetic-energy gain of 273.7 J. The arms take 196 J of it: rear shoulder −69.6 J, rear elbow −39.5, rear wrist −32.1, lead shoulder −26.0, lead wrist −23.6. The damping torque reaches 29 N·m at the rear elbow and 16 N·m at each wrist, where the forearms' muscles have 17–18 N·m and the wrists' 24–51. Passive joint damping published for wrists and elbows is roughly one to two orders of magnitude below 1 N·m·s/rad (not checked against a source here, so this is a pointer, not a figure).

The same plan, replayed with no search at a lower damping (`damp.py`, 0.25 ms). **Indicative only (rule 4): the plan was tuned to damping of 1 and no longer meets the ball.**

| Damping (N·m·s/rad) | Fastest sweet spot before contact | Where it crosses the ball's plane |
| :-- | --: | :-- |
| 1.0 (as built) | 19.25 m/s | 0.2 cm, 17.69 m/s |
| 0.3 | 22.46 | 8.5 cm high, 21.46 m/s |
| 0.1 | 23.00 | 6.0 cm high, 20.97 m/s |
| 0 | 23.26 | 2.4 cm high, 20.36 m/s |

So an unsourced constant is worth about 3–4 m/s on this plan before any search adapts to it. That is more than any of the three suspects in "What part 3 shows" (joint ends, the lower body, the plan) has been shown to move. The statements "nothing found names the part that would stop it" and "no part that stops it was found" are true only of the parts the account looked at.

Part 2's audit also leaves damping out. Its 690 J for a 29 m/s swing against 2,300 J of summed best power is "3.4 times". At this damping the 17.7 m/s swing alone already loses 266 J. Damping power grows with the square of the joints' speeds, so at 29 m/s the loss would be several hundred joules more, and the margin is much narrower than 3.4 times. The audit itself states that it is a necessary condition and no more.

**Wanted:** the damping gets a source and a range, and the last session's "one change at a time" starts with it (a settled search at the sourced value), before the stronger and smaller batters.

### P1-3. Nine muscle groups at 92–97% of their bounds, read to the owner as "not short of muscle".

`at_full_strength` counts a strength exactly on its bound. In the best plan: shoulder_R 0.974, sup_R 0.972, core 0.968, pron_L 0.968, hip_add_R 0.957, sh_irot_L 0.948, sh_erot_R 0.945, sh_raise_L 0.944, wrist_R 0.918 of the bound. Activations reach 1.000 (hip_rot_R, hip_rot_L, shoulder_R, elbow_ext_L, core). Four rises sit within 6 ms of the 50 ms floor (shoulder_R 52 ms, sh_lower_R 54, hip_add_R 56, hip_rot_R 56), so "no rise at its floor" means "none exactly on it".

Two things keep a result just off its bounds:

- The score charges effort: 0.83 of the best's 32.5.
- A rank-based search with a final spread of 0.0065 of each number's range does not settle exactly on a box edge.

So "no muscle group at its full strength … so what holds it is not strength in any single muscle's number; it is where this search came to rest" (§4.12) and "Is he short of muscle? Not in any one place the runs can see" (For the owner) are not supported. What the runs show is that several groups are within a few per cent of their bounds and the score is trading speed against effort.

### P2-1. "Why session 9 stalled: the score rises from 303 to 353 before falling to 45" does not hold as an explanation.

Measured by term along the same line (`line.py`):

- **The ridge is entirely the balance term:** 158 at the start, 198, 221, 246, 257 at α = 0.3, 228 at 0.4, then **20 at 0.5**, a cliff between 0.4 and 0.5.
- Every other term falls monotonically: the miss 53→35, the line 30→18, the clock 15→0, "never came round" 15→2.
- The reach stage's own score has the same ridge (242 → 302 → 61).
- **The whole-aim stage never crossed it on this line.** Its history starts at 76.6 (generation 0 of `history`), below the ridge: the reach stage had already moved the plan there, and not along a straight line.
- The line is measured on session 10's body and start (cleared start, ellipse wrists). Session 9 ran on another start, with box wrists, using a different method.

So the line shows that the straight path from S to the result loses balance before it regains it. It does not show what a slope-following search from S would do on session 9's body, nor why session 9 stalled. The session 9 review's causes (the clock term, the roughness at 0.5 ms) stand. The sentence "That, with the score's roughness, is why session 9 stalled" should be withdrawn, or narrowed to the balance cliff on this line.

### P2-2. The 0.20 s run is described as more than one unsettled run shows.

The spec says the search "makes the legs and trunk lead" and "What it does show is direction … the practice search uses the lower body in a hitter's way". For the owner says "the search then does what hitters do". This rests on:

- **The hips:** 375 against 392°/s in the first, unsettled runs of both viewing times; the gap to 544 appears only in the continuation, which itself had not settled (7.0 over its last 50 generations). The trunk is faster at 0.20 s in all three pairs (S 469/598, K 377/615, S carried on 414/699), so that part is a consistent pattern, though still from unsettled searches.
- **The clock:** the arms still fire 12 ms before the clock allows, so the "made to wait" condition is not met.
- **The energy:** 35% of that run's positive work is the hold-still torques (P1-1), mostly at the knees and the rear hip.

Rule 5 ("a difference between unsettled searches is not a finding") applies. Suggested wording: "one run that had not finished turned the trunk faster at 0.20 s in all three starts, and the hips faster in one; whether that is the viewing time's doing is not shown".

### P2-3. "His hips cannot face the pitcher" is told to the owner as a fact about his body; it rests on the start's foot and on a search.

Part 2's finding depends on several things:

- the lead foot fixed where [4]'s launch posture puts it at −250 ms, 30° open;
- the lead hip's passive mean turn (Roaas & Andersson 32.6°; the active Boone & Azen figure of 44° would give 11° more, as the spec says);
- a knee with no twist and an ankle that cannot turn about the shin;
- a compass search from four starts, one of which ended 13° outside.

Each of these is disclosed in §4.12, but "Two things about how his body is built" in For the owner leaves out all but the knee and foot. The comparison is also overstated. "Hitters get square to the pitcher" (90°) sits beside the spec's own [4] figure for the pelvis at contact, 77 ± 6° (§4.4's table, line 417), so the gap from "about 70°" is about one SD, not 20°. Suggested wording below.

### P3

- **P3-1. Peak loads read from 1 ms records understate them.** Read every 0.25 ms step, the lead knee's end carries 1,005 N·m before contact (reported: 899); the lead hip's turn carries 419 (375). At 0.125 ms the figures are 1,015 and 418. The conclusion does not change; the figures do.
- **P3-2. What follows contact does not converge with the step.** The lead wrist's distance past its ellipse is 0.8, 3.1 and 3.9° at 0.5, 0.25 and 0.125 ms; that stop's load is 675, 263 and 196 N·m; the lead forearm's excess is 2.6, 5.9 and 5.8°. "The same at three steps" holds for contact (17.693, 17.691, 17.689 m/s; 0.14, 0.19, 0.20 cm), not for the follow-through figures in the gate table. "Keeps both wrists inside a real wrist's reach all the way" (For the owner) is true up to contact; afterwards the lead wrist goes 3–4° past.
- **P3-3.** The wrists' place in their ellipses is sampled every 5 ms. The lead wrist's "0.94 at most to contact" is below its value at contact itself, 0.96 (`wrist_oval` in the kept file: 0.936 max, 0.962 at contact).
- **P3-4. The probe check's shortening (question 5): the worker's conclusion is right, the explanation is imprecise, and the cut loses coverage.**
  - Over 200 ms (`diverge.py`), the 1.3 × 10⁻⁸ plateau is in **the stops' booked-work integral** (state entry 211), not in the motion. The bodies' states differ by at most 2.1 × 10⁻⁹ (at 110 ms) and fall back to 2 × 10⁻¹⁰.
  - The engines' difference lies inside the Python engine's own spread under three different 10⁻¹³ nudges (6 × 10⁻¹⁰, 1.5 × 10⁻⁸ and 2.9 × 10⁻⁸). No difference of law is hidden.
  - Both wrists' ellipses do engage inside the first 100 ms (lead at 89, 106 and 108 ms, up to 762 N·m; rear at 25–31, 90, 107–113 ms, up to 1,275 N·m), as the probe's comment says. But the cut drops the lead wrist's longest engagement (165–184 ms, 20 steps).
  - Keeping 200 ms under the "sensitive" rule the cleats case already uses, or comparing the motion's entries apart from the work integrals, would keep that engagement in the check.
- **P3-5.** `clear_start` is a compass search with a penalty, and its result is a local least change. The test's "each changed number moved an eighth back leaves a joint short" is a condition one number at a time. "The least change" should read "a least change, by a compass search". The stored change is what the rule returns: the `BB_SLOW=1` rerun passes in 265 s.
- **P3-6. The skipped runs (question 3).** The 3° and 8° margins and the tilted ellipse were not run. The only claim this leaves exposed is "both wrists inside a real wrist's reach" at the start: on the 8°-tilted ellipse the rear wrist starts at 1.02, just outside (§4.12 says so). For the owner does not, and should say it rests on the untilted [E]. No other claim depends on the skipped runs.
- **P3-7.** The research index lists the rear calf (start K) as tried and ruled out on "no faster than the same plan without it (14.0 and 18.7 against 16.3 and 19.2)". Those are unsettled searches (rule 5). "No run shows it buying anything" is the most that can be said.
- **P3-8.** The search's test (`test_the_search_without_slopes_finds_a_known_minimum_on_a_score_its_slopes_would_miss`) is independent: the minimum is known by construction, the failures are planted and the box edge is exercised. But nothing in it shows that a slope method would miss that minimum; the name claims more than the test checks.
- **P3-9.** "One session will not close 11 m/s" (Status, For the owner: "25 mph") is a judgement, not something a run showed. P1-2's indicative 3–4 m/s from one [E] means it should be stated as an expectation, and the damping and the hold-still torques should come first in item 2's single changes.
- **P3-10.** The count of joints near their ends at session 9's start is inconsistent. The Status says "six joints within 5° and one outside"; `StartChecks` lists five short (on the ellipse, the rear wrist's two box angles become one). The docstring says six.

## Answers to the brief's questions, in short

1. **The headline replays.** The kept 0.10 s result in the probe at 0.5, 0.25 and 0.125 ms: contact 308.26, 308.24 and 308.24 ms after the start; 0.14, 0.19 and 0.20 cm up and −0.07, −0.09 and −0.09 cm along the bat; 17.693, 17.691 and 17.689 m/s; −3.59° and −3.57°. The Python engine at 0.25 ms gives 17.6897 m/s and 0.193 cm, with states within 3.5 × 10⁻¹⁰ of the probe over the whole 450 ms run.
   - **Order:** legs −307 ms (hip_rot_L), trunk −279 (core), arms −256 (sh_lower_R), as stated. Settled as defined (0.65 over 50 generations).
   - **Ranges:** before contact only the stops in §4.12's table carry load; every passive element's net work is negative at 0.25 and 0.125 ms. Each stop to contact at 0.25 ms: trunk twist −8.2 J, lead knee −14.5, rear hip flexion −6.1, rear knee −5.3, rear hip turn −4.4, lead hip turn −3.3, lead hip sideways −2.7, trunk side −2.2, rear shoulder −2.0, lead upper arm −1.6, lead shoulder −0.5; the ellipses 0 to contact and −8.2 (lead) and −2.6 J (rear) to the run's end.
   - **The feet** stay inside the cone (largest friction over load 1.150 at a grip of 1.15) and never pull.
   - **The muscles** are inside their bounds by construction, but the hold-still torques are not (P1-1).
   - **Gate rows:** I confirm every gate row's figure, apart from P3-1 to P3-3.
2. **The search** is separable CMA-ES as Ros & Hansen write it (weights, cc, cs, c1 and cμ scaled by (n+2)/3, hσ, the step-size path), with box repair and a penalty. Its checks are real and match its outputs. The "303 → 353" explanation does not hold (P2-1); the 0.20 s run is overstated (P2-2). The parked runs are described fairly: C at 0.20 s and D at both viewing times end with the nearest approach moving −0.2 to −2.3 m/s along the line, and the table labels its speed column as the plane crossing's.
3. **The ellipse:** centre (−5.58, −5.20)°, half-axes 70.72 and 27.29°. Its arcs are 141° front to back and 55° sideways. The corner session 9's rear wrist sat in is at 1.40, 1.57 and 1.66 times the radius at 0, 5 and 8° of tilt; past 21° of tilt no ellipse exists. Both readings take the same clinical sign on each hand, so the 180° turn argument holds.
   - **The reading** uses kind 0 angles about axes the universal joint holds square to the other direction, so it is never undefined.
   - **Passive in both engines:** the gradient is checked by finite differences, the Rust stop is identical in law, and it is engaged within the two-engine check.
   - **The start:** with no drive, the cleared start has no end carrying load from rest within 100 ms. From its starting velocities, the lead hip's turn carries load from 73 ms, up to 131 N·m. "No end carries load in 50 ms" is true.
4. **Part 2:** "did not decide" is the honest reading, and `built3d.py --check` tests independent things: the engine's channels against the solve's generators, the loads put back into the equations, the doubling under twice the gravity. Its findings on the way are disclosed with their dependencies in §4.12, but the owner's text strips them (P2-3), and the audit leaves out damping (P1-2).
5. **The probe change** hides no difference of law. The explanation should name the work integral, and the cut loses the lead wrist's long engagement (P3-4).
6. **The tests:**
   - `OvalChecks` is independent: the ellipse by hand, the reading from posed clinical angles, gradients by finite differences, passivity measured by the test's own reading. Its mutation fails as claimed.
   - `StartChecks` is partly self-referential (the margins come from the code under test), but the readings are tested elsewhere and the full rule rerun reproduces the stored change.
   - The search's test is independent (P3-8).
7. **The account:** see P1-1 to P1-3, P2-2, P2-3 and P3-9. The figures, test counts (71 and 112), NOTES.md and the research index's new lines are current, apart from P3-7 and P3-10.

## Corrected sentences for the owner (For the owner)

- In place of "**Is he short of muscle? Not in any one place the runs can see.** In the best swing no muscle group is working at its full strength … So what holds him at 40 mph is where the search came to rest, not a muscle on its limit":
  > **Is he short of muscle? Not known.** Nine of his 41 muscle groups work at 92–97% of their strength in the best swing, and the search is charged for effort, so strength may well be part of what holds him. Two parts of the body model also matter, and neither has a measured value: the joints' built-in damping, which soaks up about two thirds of what his muscles put in (a test replay with a third of it swung the bat about 3 m/s faster), and the constant holding torques that keep him standing at the start, which never let go and give the swing about a quarter of its energy, most of it at the knees.
- In place of "nothing in the program adds energy" (What passed):
  > no stop, damper or the ground adds energy; the constant holding torques do (157 J of the swing's 574 J), and they follow none of the muscles' rules.
- In place of "this one keeps both wrists inside a real wrist's reach all the way":
  > this one keeps both wrists inside a real wrist's reach up to contact (on the untilted wrist range we chose); after contact the front wrist goes 3–4° past.
- In place of "the search then does what hitters do: his legs and trunk go at once and turn much harder (hips 544°/s …)":
  > in one search that had not finished, his trunk turned faster at 0.20 s than at 0.10 s, and in its continuation his hips did too (544°/s); his arms still started 12 ms too early, and a third of that swing's energy came from the holding torques. Whether the viewing time causes it is not shown.
- In place of "*His hips cannot face the pitcher at contact.* With his front foot planted where the lab study's hitters plant it, his front hip runs out of turn when his hips are about 70° open (hitters get square to the pitcher)":
  > *With his front foot where the lab study's launch posture puts it, his front hip runs out of turn at about 70° open* (the lab study's hitters are at about 77° at contact). That rests on the foot's place, the average resting range of a hip's turn (an active range would give 11° more) and a knee and foot that do not twist.
- In place of "One session will not close 25 mph":
  > I do not expect one session to close 25 mph, but two unmeasured parts of the model (the damping and the holding torques) are worth testing first.

## Verification

All run on `9d4aa6d` in this worktree. Jobs 407–409 through `bin/job.sh`; the short probes ran directly, one at a time.

| What | Result |
| :-- | :-- |
| `python3 -m unittest analysis/oracle/test_body3d.py` (job 407) | 71 tests, OK, 80 s |
| `python3 -m unittest analysis/oracle/test_oracle.py` (job 407) | 112 tests, OK (8 expected failures) |
| `python3 analysis/research/body/probe3d.py --check` (job 407) | PASS, worst 3.6 × 10⁻¹¹; ellipse case 2.7 × 10⁻¹¹ and 2.45 × 10⁻¹¹ |
| `uv run --with numpy --with scipy python3 analysis/research/body/built3d.py --check` (job 407) | PASS: 41 channels matched (4.6 × 10⁻¹⁵); start re-placed 1.3 × 10⁻⁴; loads put back 8.6 × 10⁻¹²; hold-still worst share 0.126, doubled exactly under twice gravity |
| `BB_SLOW=1 … test_body3d.py -k least_change` (job 409) | OK, 265 s: the rule returns the stored START_CLEAR |
| `replay.py` (job 408): the kept 0.10 s result, probe at 0.5, 0.25 and 0.125 ms, every step, plus the Python engine at 0.25 ms | Figures in answer 1; per-stop work, loads and excursions in `replay100.out`; Python against probe 3.5 × 10⁻¹⁰ |
| `energy.py`: energy by element and by joint, S at 0.10 and 0.20 s, C at 0.10 s | P1-1 and P1-2 tables (`energy.out`); closes on the engine's ground figure |
| `damp.py`: the kept plan with damping at 1, 0.3, 0.1 and 0 | P1-2 table (`damp.out`), indicative |
| `line.py`: the score by term along S → the first 0.10 s result, and the reach stage's score | P2-1 (`line.out`) |
| `diverge.py`: the ellipse case over 200 ms, each wrist's engagement per step, three self-nudges, the motion against the work integrals | P3-4 (`diverge.out`, `diverge2.out`) |
| `still.py`: no drive, 100 ms, the cleared start and session 9's, from the starting velocities and from rest | Cleared start: lead hip turn from 73 ms (from its velocities), none from rest |
| The ellipse's numbers (`body.wrist_oval`, `oval_place`, `clinical_sign`) | Answer 3 |

## Resolution

Open.
