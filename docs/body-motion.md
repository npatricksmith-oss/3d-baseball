# Body motion: a force-driven batter and pitcher (PLAN D31, §14.1 phase 0)

**Status: design; phase 1 not passed. The lab study [4] is dropped as target and gate (owner, PLAN D37); the swing is found by practice on the body and checked against hitters measured directly, the bat's speed against 29–34 m/s (PLAN D38); the gate is §13's (D40). Session 8's review found that the engine's joint stops had been creating energy since session 2; fixed, and with the joints held to their ranges, the practice search finds the first swing on stops that make no energy: it meets a mid-zone fastball within 2 cm at 17.8 m/s (40 mph), found from the plan fitted to the lab study and its launch posture (the two starts that owe it nothing did not become swings). It leans on joint ends far harder than a joint can take, so it is not yet a swing a body could make (§4.10, §4.4). Session 9 gave the body a hitter's hands, a muscle that only holds and cleats' grip (§4.11): with real hands that swing cannot be made, and the search finds none; the nearest motion passes 7.9 cm from the ball at 19.0 m/s (43 mph). Session 10 (§4.12) moved the start off the ends of its joints and replaced the search with one that does not stall: that body swings. It meets a mid-zone fastball within 0.2 cm at 17.7 m/s (40 mph), legs then trunk then arms. Whether it can reach the gate's 29 m/s is not shown. Its independent review found two parts of the model that carry much of that swing and have no measured value: the joints' damping takes two thirds of the muscles' work, and the constant hold-still torques give a quarter of the swing's energy under no muscle rule.** Written 2026-11-25 by worker body-design on branch `work/body-motion`, revised the same day after three independent reviews. Phase 1's first three sessions (worker body-3d, 2026-11-25) built the three-dimensional checking engine, a compiled probe of it, the placed and moving body, the fit under the design's rules and a 3D inverse dynamics. Sessions 5–7 refitted it to [4] on measured limits (PLAN D34; dated history, §4.6–§4.9). Session 8 is §4.10. This file is phase 0 of [PLAN §14.1](../PLAN.md#141-the-body-motion-rebuild-d31): it settles the seven items of the plan and the owner's question on consolidating muscle groups and ratings, as far as a flat probe can. Nothing here is in Rust; today's swing and release stay in games until phase 4.

## Status and what is left

- **Done:** the probes of §3 (stdlib Python, committed under `analysis/research/body/`), this spec, the owner's walkthrough as [research note 0](research/body/0-owner-walkthrough.md). Worker body-research's notes 1–7 are merged ([index](research/body/README.md)).
- **Reviewed and revised.** Three independent reviews (2026-11-25: [Codex Sol](reviews/2026-11-25-codex-sol-body-motion-review.md), [Gemini Carbon](reviews/2026-11-25-gemini-carbon-body-motion-review.md), [Gemini Neon](reviews/2026-11-25-gemini-neon-body-motion-review.md)) found errors in the probes and the spec. This version is rewritten from the corrected runs; "What the first version got wrong" below lists what changed.
- **Built in the probes:** a driven pelvis–trunk–arms–bat chain over planted feet, in the horizontal plane (§4.2). The three-dimensional body is built in the checking model (phase 1, §4.4–§4.9); not built: the pitcher's body (§11, design depth only).
- **The main open finding.** Under the design's own rules (the arms start no earlier than the decision allows, drives rise no faster than 50 ms, strengths inside sourced bounds), the flat chain does not reach the ball in time (§4.3). The fit that does reproduce [4] breaks those rules. Phase 1's three-dimensional body does swing under them (§4.9).
- **Phase 1, session 10 (2026-11-26; PLAN D41's second of three), §4.12. The session's question: can this body make a swing that meets the ball at the gate's speed? As far as the runs show: it makes one that meets the ball, at 17.7 m/s (40 mph); whether it can at 29 m/s (65 mph) is not shown either way.** Three parts, each written before it was built or run; then [the independent review](reviews/2026-11-26-opus-body-3d-s10-review.md), which confirmed the build and corrected the account (the last sub-item).
  - *(1) A start off the ends.* [4]'s launch posture moved by a stated rule: a least change (by a compass search) to the 17 numbers it is built from that puts every joint 5° inside its end (thirteen change, none by more than 2.5°; session 9's start had five joints within 5° of an end, the rear wrist outside a wrist's reach among them). The wrist's reach is now an ellipse, not a box, held by a stop that reads both of its angles, passive in both engines; the rear wrist starts inside it. No stop is moved to clear the start. The start drifts as [4]'s moving hitters did (the feet's forces are 78% of the weight at that instant) and no joint end carries load in the first 50 ms. The rear arm's posture still flips inside the elbow clearance's own range.
  - *(2) The test without a search: it did not decide.* The solve is built and checked (the joint torques a motion asks, held against each muscle's strength at its speed, the free loads chosen by a linear programme; its 29 axes are exactly the engine's 41 muscle channels). But no motion I built by hand is one a body could make: of 64, none keeps the arms inside their ranges between the start and contact. **A failure of my construction, not a finding about the body.** Found on the way: a hitter's contact posture cannot be taken for the aim's ball; with the front foot planted where it stands this body's hips cannot face the pitcher (the lead hip's range of turn); a balanced contact posture inside every range exists by 1°, with the front knee bent 38–48°. The muscles' summed best power over 200 ms is 3.4 times the 690 J a 29 m/s swing holds, before what the joints' damping takes: power does not rule it out, by a margin that is not wide.
  - *(3) The search, made to work.* An evolution strategy that uses only the order of whole runs' scores, every run at 0.25 ms, its own checks printed with every result. Ten searches, no failed run, every result the same at 0.5, 0.25 and 0.125 ms. **From session 9's plan, watching 0.10 s: the ball met within 0.2 cm at 17.7 m/s, coming through it nearly level (3.6° down), settled.** It fails the gate on speed (11.3 m/s short), the hands (7.6 m/s), the hips and trunk (361 and 414°/s), the front leg (nothing on the front foot at contact), and leans on 15 joint ends at 92–1,204 N·m; nine of its 41 muscle groups are at 92–97% of their strength. Watching 0.20 s, searches that had not finished turned the trunk faster in all three pairs and the hips faster in one (544°/s); the arms still fired 12 ms early; whether the viewing time causes it is not shown. A plan written by hand, owing [4] nothing, became a swing for the first time (0.4 cm, 10.1 m/s); three of the four hand-written runs parked the bat at the ball (the score's basin, not remedied).
  - *(4) What the review measured (§4.12, "After the independent review").* **The hold-still torques** (one constant torque a joint, standing for the muscles that hold the launch posture; no strength-at-speed law, no onset, no effort, no clock) **give the best swing 157 J, 27% of what they and the muscles put in**, most of it at the knees: at the front knee twice the knee extensor's own work. **The joints' damping** (1 N·m·s/rad on every joint, an [E] with no source) **takes 266 J, 64% of the muscles' work**; the same plan replayed at a third of it has the bat 3 m/s faster (indicative; the review's replay). Every result now reports both apart, by joint. A source for the damping was looked for and not secured; it is flagged unsourced. Withdrawn: that the score's ridge explains session 9's stall. Nothing is waiting on the owner.
- **Phase 1, session 9 (2026-11-26; PLAN D41's first of three), §4.11:** the three parts session 8's reviews found missing, each designed in the spec before it was built and tested as anatomy or against a measurement. (1) **Each hand's own grip:** palms facing, the handle 15° oblique in the hand [E], the wrist's, forearm's and upper arm's angles read as a clinician reads them and held at their ends; both wrists start inside a wrist's range and a hitter's contact posture fits; the old "−168° and −134°" forearms were a zero on the wrong end. The start's free choices now follow a rule and leave the lead shoulder within 1° of two ends. (2) **A muscle tensed before the pitch only holds:** no net torque until its onset, then its hold lets go; the clock reads onsets. (3) **The feet on dirt:** the measured grip of a baseball shoe (1.15 for an unsourced 0.8; the old forefoot was half as hard to turn as a cleat), the ankles' ends; the rear foot was never "planted": in session 8's swing it slid 30° the wrong way. The toe piece first designed was withdrawn (a free-jointed toe carries no load). **A fault of mine was found and fixed on the way:** the upper arm's new reading is undefined with the arm overhead, the follow-through went there, and the first runs after parts 1 and 2 were stopped by failed runs; the shoulder now has its end of raise and every result counts its failed differences. **Result: no start becomes a swing on the new body.** The nearest (session 8's plan carried on, part 3's body) passes 7.9 cm from the ball at 19.0 m/s, chopping down at 18°, the arms against four or five joint ends, and fails the gate everywhere. Not shown either way whether the body can swing: the search stalls with no failed runs, the start from [4] sits at joint ends, and no plan was made for this body. **[The independent review](reviews/2026-11-26-opus-body-3d-s9-review.md) found nothing at fault in what was built and put the search first among the causes** (its clock term could not see arm onsets moving together; its score is too rough for its steps, partly from the 0.5 ms step), the jammed start second (the rear wrist starts outside a real wrist's reach), and nothing pointing at the body. The clock term and the run rule are fixed with tests; no search is rerun. Nothing is waiting on the owner.
- **Phase 1, session 8 and its two reviews' fixes (2026-11-25/26), after PLAN D37:** (0) every place [4] enters the code and the gate is listed (§4.10). (1) The practice aim, the joints' ranges from measured samples, the checks from direct measurements and the expected result were written before any search. (2) Session 8's first practice runs found no swing; [the first review](reviews/2026-11-25-opus-body-3d-s8-review.md) found why they could not be trusted: **the engine's stops on ball joints created energy** (the 30.6 m/s "strongest batter" was +414 J of it; the fit to [4] is 17.8 m/s, not 21.3), and a wrist's cock was misread past 90° of flexion. (3) Fixed, each with a test that fails on the old code; the stop law is conservative in both engines and "no passive element does net positive work" is an engine check. **The wrists, both halves** ([the second review](reviews/2026-11-25-opus-body-3d-s8fix-review.md), P1-1): the model's wrists uncock the right way and the lead wrist was placed right (the first review's finding that the travel was on the wrong side was wrong); but the rear hand, welded to the handle at the lead hand's roll, starts 60° more opened out than the lead, outside any wrist's range, and can reach a hitter's top-hand posture only through an impossible one: a missing part of the body. (4) **Rerun with a search that has no cliff, the search found the first swing on stops that make no energy, from the plan fitted to the lab study and its launch posture (the two starts that owe it nothing did not become swings): it meets a mid-zone fastball within 2 cm (the gate asks 1) at 17.8 m/s (40 mph), coming up through it at 7°.** It is not yet a swing a body could make: nine joint ends carry 75–906 N·m where ligaments fail at 13–34, the rear wrist is outside a wrist's range, his balance is 3 cm outside its test and his feet carry under half his weight at contact. It fails the gate on speed and on how it is made (the hips turn 35° at 330°/s, the weight stays back, the hands reach 7.7 m/s). What holds the speed is not established. Nothing is waiting on the owner (§4.4).
- **Phase 1, session 7 (2026-11-25):** (0) the 22.9 m/s of [4]'s own motion is a measuring effect: [3]'s 29–31 m/s is the bat's end (23.9–25.5 at the sweet spot), and [4] smoothed its marker paths through the impact ([3] probably too); undone, [4]'s motion has 26–33 m/s on this body over the bat paths that reproduce its curve, an estimate resting on the path's shape (§4.8, redone after the review). (1) The 19 undriven freedoms' limits, expectations and four held-out checks were written before any run (§4.5). (2) The engine got a drive about a moving axis and the channels, with their checks (§8.1). (3) **Refitted with 41 channels, contact is +0.8 ms and every gated curve is inside its band (the trunk's 0.57); the arms now move as [4]'s do; the bat is at 21.3 m/s, where the same limits allow 30.6 if asked only for speed. Three held-out checks on the muscles' use fail and the bat comes down on the ball at 7°; held to a hitter's swing the strongest batter reaches 22.5 m/s** (§4.9).
- **Phase 1, session 6 (2026-11-25):** the arms' limits were written down with their sources before any refit (§4.5: strength by direction, speed curves, tendon stretch at shoulders and elbows, a starting level, the clock restated as the bat's measured first motion at −170 ms with the arm muscles rising at most 70 ms earlier). **Refitted, contact is +4 ms, the bat first moves at −172 ms, and every gated curve but the trunk's is inside one band unit. The bat is at 20.1 m/s: the gate asks 29–34, [4]'s own motion on this body has 22.9.** Nothing was added; the gate is not passed (§4.7).
- **Phase 1, session 5 (2026-11-25), after the owner's ruling that limits come only from measurements (PLAN D34):** the limits were written down with their sources before any refit (§4.5); the engine got measured strength-at-speed curves, tendon springs at the leg joints and muscles tensed beforehand, with their checks (§8.1). **Refitted, the lower body is inside the gate (pelvis 573°/s at −93 ms, the feet's forces within a band) and the bat is still 21–28 ms late at 18.6 m/s. What is short is the arms, whose limits are estimates and were not touched. Nothing was added to close the gap** (§4.6).
- **Phase 1, session 4 (2026-11-25, diagnosis only):** session 3's 330–440 N·m for the trunk was the lumbar torque about the pelvis's long axis, which takes in part of a large bending moment. About the trunk's own long axis, the way strength is tested, the need is 321 N·m as placed and 216–355 over the things it rests on; worked up from [4]'s ground forces it is 131–172 before the feet's unreported free moments. **So the trunk's excess over 288 is not robust.** The hips' and lead knee's shortfall at speed stands (§4.4).
- **Phase 1, session 3 (2026-11-25):** the lower body alone turns the pelvis in time inside the bounds; the 3D inverse dynamics has [4]'s swing needing 330–440 N·m of trunk rotation against a bound of 288, and 70–95% of the hips' and lead knee's full strength at speeds where the force–velocity law leaves 25–60%. **Stopped for the owner under §13's rule; §4.4 has the question, the options and a recommendation.**
- **Phase 1, sessions 1 and 2 (2026-11-25):** the 3D engine in Python with its checks (§8.1); a compiled probe of it, agreeing to 2 × 10⁻¹² and measured at 50 µs a step (§8.2); the body placed on [4]'s measured state, moving, with both knees resolved; the fit under the rules, the rules relaxed one at a time, the small frame and the wrists (§4.4). **The 3D body as built comes round about 41 ms late at 17 m/s. The pelvis is turned too late, which is upstream of the arms' rules, so this is not yet the answer to whether a body can swing under them.**
- **Left for whoever continues** (PLAN D41: **one more session**; after it the work stops if there is still no swing that passes §13's gate). The best swing is 11 m/s short of the gate. I do not expect one session to close that (an expectation, not something a run showed), but two parts of the model with no measured value carry much of this swing and are tested first. **The last session should be spent on why it sits at 40 mph,** in the review's order. Each numbered change is one change from the kept swing, with a search run until it has settled (about 1,000 generations, ten minutes a run), written with what is expected before it runs:
  1. **The joints' damping at a read value.** First read a source (pointers, none opened: the relaxed wrist at 0.02–0.06 N·m·s/rad; the elbow and shoulder at 0.2–0.7 while a posture is held; nothing found for the hips, knees and trunk) and write the values by joint into §4.5 with their range, in place of 1.0 everywhere. Then one settled search at those values. The review's replay of the kept plan at 0.3 had the bat 3.2 m/s faster before any search adapted.
  2. **A hold that lets go, or follows the muscles' law.** The hold-still torques either fall away as the muscles start (as a tensed muscle's hold does, §4.11) or become muscle: held by the strength-at-speed law, counted as effort, their own channel's level beforehand. One settled search. Expected to cost speed: 157 J of the kept swing is theirs, 58 J of it at a front knee straightening at 870°/s.
  3. **The two together,** then both viewing times from S and from C, settled, with **the score's parked basin remedied** first (§4.12's last block of part 3; worked by hand before any run) so that starts which owe [4] nothing count. Only settled results on the corrected body say anything about the viewing time.
  4. **One change at a time from the settled swing on that body:** the strongest batter and the small frame (PLAN D32); the hips' range of turn at Boone & Azen's 44°; the ball met 11 cm further back (PLAN D29's [E]); the wrist's ellipse tilted 8° and the elbows 8 cm clear (the start's two [E]s); the start's margin at 3° and 8°.
  5. **PLAN D39 built** (a joint's end takes the motion's energy and the search avoids load on it; its two sizes written first). The best swing throws the front knee into its end with 1,005 N·m, mostly on the hold-still torque; item 2 may change that before D39 does.
  6. **The test without a search, made to decide** (§4.12, part 2's last item): the torque solve inside a search over the posture numbers' paths, with the damping in its equations at the read value.
  7. **Smaller, in no order:** a start that owes [4] nothing (it must look again at the grip's obliqueness, and would end the start's two flips); the knee's twist and the lead foot's turn under load; the hold-still solve's lead foot; the toe flexors' moment; each hand's roll about the handle; the raise's end by direction; the upper arm's turn by the arm's height; the held drive's missing stiffness; muscles for the trunk's bending ([note 12](research/body/12-trunk-bending-and-feet.md) C).
  8. Then the gate, the breakdowns of §6.2 and an independent review. Dead ends are in the [research index](research/body/README.md) under "Tried and ruled out".

**What the first version got wrong** (and what the reviews found):

- The breakdown probe moved a channel's onset and left its switch-off where it was, so a "held" or "waiting" drive was cut short and an "early" one lengthened. Lunging's arms-only 17.6 m/s swing, the heel-plant wait's 22 m/s and early hips' *gain* in bat speed came from that, not from the fault. With the burst moved whole, lunging and the wait are late but intact, and early hips cost a little speed (§6.2).
- "The wrists switched themselves off" rested on a wrist channel whose onset the fit had put after the end of the fitted window, where nothing could see it. Refitted with the channel visible, the wrists carry 0.3–13 N·m in three starts, effectively nothing, so the statement survives in a weaker form (§4.3). But the swing does depend on what holds the wrists' cock: without the cock stop the bat passes the ball 14 cm beyond its end (§4.3).
- The fit started the arms at −231 ms, 61 ms before the decision allows, and used strengths 1.5–2.5 × the sourced ones behind slow ramps. Enforcing the design's rules, the flat chain cannot swing (above).
- The friction cap was reported at the end of the run. It never limits the legs in the average swing; it does only when the front foot is late on a fixed clock (§6.2).
- The effort table moved each easier or harder swing earlier so it still met the ball at 0 ms, against the design's own rule. Not moved, a swing at 80% of its drive arrives 19 ms late (§7.2).
- The replay design re-ran a drive program on the client, which breaks PLAN D9 (§9 now stores the recorded motion).
- The sign of [4]'s lateral ground forces is settled by [4]'s Appendix C, and the ground moment quoted (222 N·m) was the wrong-sign column; under the right sign it peaks at 156 N·m (§5.2).

## 1. Summary in baseball terms

**The body.** The batter is a set of body segments (feet, shanks, thighs, pelvis, trunk, head, upper arms, forearms, hands) with masses and inertias from de Leva's tables scaled by his height and weight, joined at his joints. His feet hold the ground by friction, the two hands hold the bat, and muscle drive at the joints turns each segment against the one below it. The bat's path is the result. Nothing in the motion is fitted to where it is meant to end.

**How a swing happens** (*what follows in this section, to the breakdowns' table, is phase 0's flat probe fitted to the lab study [4]; under PLAN D37 that study is no longer a target, and the breakdowns and ratings are to be redone on a practised swing, §4.10*)**.** Each joint group has a drive. It switches on at a planned instant, rises over a set time, is capped by the player's strength, and weakens as the joint turns faster (muscle's force–velocity law). The legs turn the pelvis against the planted feet, the trunk is pulled after the pelvis and then driven itself, the arms follow, and the bat, cocked at the wrists, is pulled round by the hands until it whips through. In the flat probe, a drive fitted to Fortenbaugh's 33 pro hitters [4] reproduces their mean swing within about one between-batter band at most points:

- peak pelvis rate 582°/s at −58 ms (measured: 581 at −70);
- peak trunk rate 703°/s at −59 ms (766 at −57);
- peak arm rate 1,129°/s at −58 ms (1,160 at −65, [3]);
- bat speed at contact 30.2 m/s ([3]: 29–31).

Given a visible wrist channel, the fit puts almost no drive on it: the bat is whipped through by the hinge, as golf models also find. Where the fit falls short is §4.3: the pelvis and trunk end 13° short, and the bat's cock at contact is 49° against 65°.

**But that fit breaks the design's own rules.** It starts the arms 67 ms before the batter could have decided to swing, and its trunk is 1.5 × as strong as professional golfers'. With the rules enforced, the flat chain does not get the bat round in time (§4.3). What is missing points at the three-dimensional body: the elbows (which let the hands accelerate the bat without the whole arm turning), the second arm and the front leg's block. Phase 1 must show that the real body swings under the rules, or the work stops.

**The breakdowns, from the body** (§6.2; the corrected runs; the ball fixed where the average swing meets it; the sweet spot is 0.634 m from the hands, the bat's end 0.784 m):

| What went wrong | What the flat body did | Where it meets the ball |
| :-- | :-- | :-- |
| Lead foot down 160 ms late, the hips' drive on its own clock | the hips push against a foot that carries little weight: the ground's grip caps their turn (the only case where it binds), the pelvis is 6° open at the meeting (64° normally) | 28 ms late, 7 cm toward the handle, 24.5 m/s against 30.3: jammed, weak |
| The same, the hips waiting for the foot (the walkthrough's heel-plant trigger) | the whole swing starts 122 ms later, intact | 120 ms late at 28.9 m/s: against a real pitch, far behind it |
| Hips 40 ms early | the pelvis leads by 46° (16° normally) | 5 ms early, 1 cm toward the handle, 29.6 m/s, pulled 2.3° |
| Casting at the wrists' measured strength (20 N·m from −150 ms) | the bat uncocks early | 1 ms early, 4 cm toward the handle, 29.0 m/s, 4.5° further round |
| Pulling off: the trunk's drive 40 ms early | the trunk opens with no separation left | 15 ms early, 4 cm toward the end, 32.1 m/s, pulled 4.4° |
| Lunging: the rotation held 100 ms | the hold delays the whole swing | 98 ms late, intact (29.1 m/s): **the power loss coaches describe does not come out of the flat body** |

Lunging's loss of power, casting's hands drifting off the back shoulder, pulling off's barrel leaving the zone and the collapsed front leg need parts the flat body lacks (the front knee, the body's weight passing over the front foot, the elbows, the vertical). §6.2 names which.

**The ratings** (§7). The game needs, as causes: body size (`Height`, `Mass`), lower-body strength and upper-body strength, and how precisely the player times his drives. Two strength groups make visibly different hitters at the same bat speed: a lower-body hitter turns his hips at 724°/s, an upper-body hitter at 427°/s, and a late foot costs the second more. Bat speed, swing time, fastball speed and command become results the manager is shown. Whether timing to the pitch and timing within the body are one rating or two is for the owner (§14). The first version's "load depth correlated with strength" is withdrawn until phase 1 measures what really makes a stronger hitter's swing longer.

**Holding the body on its path** (§5.3):

- A standing body played forward on recorded forces falls: a 0.5° lean error grows to 41 cm in 1.5 s on one foot.
- Muscle's own stiffness plus a reflex 100 ms late holds it within 1 cm over the whole size range, for small errors. On one foot, a 2° error still grows to 14 cm and a 3° error falls. The pitcher's leg lift is the hard case.
- In the flat probe the swing needs no feedback: a 15 ms late start stays a 15 ms late meeting. That does not yet show a three-dimensional swing runs open loop: the flat chain cannot fall and has no swing plane to tilt (§5.3).

**Cost** (§8.2). The flat chain costs 1.7 µs per step in Rust, 0.46 ms per swing. Every three-dimensional figure is an estimate until phase 1 benchmarks a real step: about 2–75 s a game for a body integrated on every pitch, against PLAN §13.4's 200 ms. The budget question waits for that measurement (§14).

## 2. What is replaced and what stays

- **Replaced:**
  - The swing: an analytic bat path fixed at the planned contact (`sim/crates/bb_agents/src/swing.rs`, `plan_swing`, `apply_placement`), with five independent errors laid on it (`motor::execute_swing`: height, plane tilt, reach, timing, effort).
  - The release: the planned release plus seven independent errors (`motor::execute_release`).
  - The client's body fitted to the bat ([player-visuals §4](player-visuals.md), §13).
  - Swing-model v3, which was never put into Rust.
- **Stays:**
  - The world's reading of a bat pose in time (`bb_physics::contact::BatPose`, `detect`, `resolve_contact`): the body produces that pose.
  - Ball flight, the collision and the bat's mass properties (physics-spec §6–§8).
  - The batter's perception and decision: the hypothesis bank, the commit time and the check swing as a decision (agent-models §3.2–§3.5). Their timing meets the body in §5.4.
  - Where the batter stands and where the ball is met (D28, D29), and direct measurements of hitters as checks on the body (§4.10). **Changed by PLAN D37:** the lab study [4] of [docs/research/swing/](research/swing/) (D21) is no longer something the body must reproduce; it may be looked at as a picture of a real swing, and none of its numbers is a target or a gate.
  - D22: the coil and the early hip turn on every pitch, held up on a take.

## 3. Sources, evidence levels and the probes

Sources by number: **[3]** Welch et al. 1995 ([note](research/swing/3-welch-1995.md)); **[4]** Fortenbaugh 2011 ([note](research/swing/4-miami-study.md)); **[dL]** de Leva 1996, J Biomech 29(9):1223–1230, Table 4 (males), read from the paper (https://robotics.unibs.it/Antropometria/deLeva-ParametriDinamiciAntropometria.pdf); **[N1]**–**[N7]** the body-research notes: [1 forward-dynamics precedents](research/body/1-forward-dynamics-precedents.md), [2 hitting kinetics](research/body/2-hitting-kinetics.md), [3 pitching kinetics](research/body/3-pitching-kinetics.md), [4 segment parameters](research/body/4-segment-parameters.md), [5 strength limits](research/body/5-strength-limits.md), [6 movement variability](research/body/6-movement-variability.md), [7 datasets and licences](research/body/7-datasets-and-licences.md); **[S10]** MLB's swing length by bat speed ([note 10](research/swing/10-swing-length-by-bat-speed.md)).

Every number below is tagged **measured** (a source gives it), **derived** (computed from measured numbers by a stated method, the probe named), or **[E]** (assumed, with what would settle it). The notes record how each source was read: FT full text, TAB a table from it, FIG read off a figure, AB abstract only, 2nd quoted by another paper. Where a number below rests on an abstract or a figure, it says so.

**The probes.** Stdlib Python 3, run from the repository root. Each needs the digitised curves of [4] Appendix E, made by `python3 analysis/research/fortenbaugh_appendix_e.py` (which needs the dissertation PDF in `sim/target/research-dl/4/`, Pillow and poppler's `pdfimages`). The digitised curves were checked against [4]'s text at 16 points, every one within its reading error; the reviews regenerated all 23 curves identically. Outputs are kept in `sim/target/codex-probes/body/*.out`.

| Probe | Command | What it shows |
| :-- | :-- | :-- |
| inverse dynamics | `python3 analysis/research/body/inverse.py` | the yaw torques [4]'s motion needs, and what [4]'s ground forces give (§5.2) |
| the forward chain | `analysis/research/body/chain.py` (module) | the driven body (§4.2) |
| the fit, free | `python3 analysis/research/body/fit.py --rules free --starts 3` (3 min; through the job queue) | the drive that reproduces [4] (§4.3); cold start, so a rerun reproduces `fit_params.json` |
| the fit, the design's rules | `python3 analysis/research/body/fit.py --rules design --starts 4` (4 min; job queue) | the same under the design's own rules (§4.3); writes `fit_design.json` |
| which rule binds | `python3 analysis/research/body/limits.py` (2 min; job queue) | the design fit run on, and one rule relaxed at a time (§4.3) |
| the fitted swing | `python3 analysis/research/body/swing.py` | the free fit against [4] and [3], strengths, the friction cap, energy (§4.3) |
| checks | `python3 analysis/research/body/checks.py` | energy, momentum, angular momentum, the Euler–Lagrange identity, mutations, step size, the wrist's hold, the segment table (§8.1) |
| balance | `python3 analysis/research/body/balance.py` | what keeps a standing body up (§5.3) |
| breakdowns | `python3 analysis/research/body/breakdowns.py` | the owner's named failures and the decision clock (§5.4, §6.2) |
| ratings | `python3 analysis/research/body/ratings.py` (25 s); `BB_BODY_SCALING=MH` for torque by weight × height | the consolidation comparison (§7) |
| the 3D engine's checks | `python3 -m unittest analysis/oracle/test_body3d.py` (13 s; also run by `test_oracle.py`) | closed forms, independent derivations, conservation, mutations (§8.1) |
| the 3D engine's compiled probe | `python3 analysis/research/body/probe3d.py --check` (50 s; builds `body3d_probe.rs` with `rustc` into `sim/target/codex-probes/body3d/`); `--bench` | the probe state for state against the Python engine on four runs; its time per step (§8.2) |
| the 3D placed body | `python3 analysis/research/body/placed3d.py` (20 s; job queue), output `sim/target/codex-probes/body3d/placed3d.out` | the two knees, the starting velocities, undriven runs (§4.4) |
| the 3D fit on the measured limits | `python3 analysis/research/body/fit3d.py --starts 3 --iters 25 --iters-all 40 --warm` (38 min; job queue); writes `fit3d_d34_all.json`; `--held s6` or `--held all` for the held-out checks; `--without`, `--strongest` (then `--strongest --on-time --from <its json>`) | the 41 channels fitted to [4] under the rules with §4.5's limits (§4.9). Session 6's fit of 22 channels is `fit3d_d34_arms.json` (from `ec25578`, §4.7). Session 5's fit with the arms on estimates is `fit3d_d34.json` (from `76d3293`, §4.6); session 2's (`fit3d_design.json`, `fit3d_design_nostop.json`) from `987e92d` |
| which rule binds; what the answer rests on | `python3 analysis/research/body/fit3d.py --relax`; `--ranges` (9 and 13 min; job queue) | that fit continued with one rule relaxed, or one of §4.5's range ends, at a time (§4.6) |
| the legs alone, 3D | `python3 analysis/research/body/fit3d.py --lower forces` (also `pelvis`, `rate`; 3–4 min each; job queue) | whether the legs turn the pelvis in time inside the bounds, under three stated weightings (§4.4's numbers are from `876b3aa`, before §4.5's limits) |
| the trunk's torque, three ways | `python3 analysis/research/body/trunk3d.py` (2 min), output `sim/target/codex-probes/body3d/trunk3d.out` | the lumbar torque from above, from below and from the angular momentum, by axis, with its sensitivities (§4.4) |
| where the placed motion's bat speed goes | `python3 analysis/research/body/batspeed.py` (10 s), output `sim/target/codex-probes/body3d/batspeed.out` | the sweet spot's speed in [4]'s motion split into hands and turning; the 12 Hz filter through the impact undone; where each source measures (§4.8) |
| inverse dynamics, 3D | `python3 analysis/research/body/inverse3d.py` (20 s; job queue), output `sim/target/codex-probes/body3d/inverse3d.out` | the joint torques [4]'s motion needs against the bounds, and the turn about the vertical against [4]'s ground forces (§4.4) |
| the small frame; the wrists | `python3 analysis/research/body/fit3d.py --small`, `--wrists` (2–3 min each; job queue) | PLAN D32's small frame; with and without a stop on the wrists' cock (§4.4) |
| the practice search (session 8 and its fix) | `python3 analysis/research/body/practice3d.py --staged main --reach 15 --iters 25` (16 min; job queue; four processes inside the one job), then `--continue <a start's json> --iters 30`; `--vary <names>` (15 iterations each from `practice3d_main.json`); `--show <json>`; `--main` is the search as first written, which parks the bat at the run's end (§4.10) | the swing asked for by the aim of §4.10 on the body with its joints' ranges, from four starts, against the direct-measurement checks, with the work of the muscles, stops and ground; `practice3d_main.json` is the settled swing |
| which stops change the earlier swings | `python3 analysis/research/body/which_stops3d.py` (20 s), output `sim/target/codex-probes/body3d/which_stops3d.out` | each saved plan of §4.9 replayed once with one group of the joints' stops on; one body, one measure; its header says how little a replay shows (§4.10) |
| the joints against their ranges | `python3 analysis/research/body/ranges3d.py [plan.json ...]` (1 s) | every joint's angle through a saved swing against `body.ROM` (§4.10) |
| Rust cost, 3D kernel | `rustc -C opt-level=3 -C linker=$HOME/.local/bin/zigcc -o sim/target/codex-probes/body3d/bench_body3d analysis/research/body/bench_body3d.rs`, then run it | the 49-row dense solve alone (§8.2; superseded by the probe's measured step) |
| Rust cost | `rustc -C opt-level=3 -C linker=$HOME/.local/bin/zigcc -o sim/target/codex-probes/body/bench_chain analysis/research/body/bench_chain.rs`, then run it | time per step of the flat chain in Rust (§8.2) |

## 4. The body (§14.1 item 1)

### 4.1 The three-dimensional body for phase 1

| Segment | Mass, inertia | Joint to its parent | Degrees of freedom |
| :-- | :-- | :-- | :-- |
| pelvis (lower trunk, LPT) | [dL] LPT, by `Mass` and `Height` | the root, floating (6) | 6 |
| trunk (middle + upper trunk, MPT + UPT, with head and neck) | [dL] MPT + UPT + head | lumbar–thoracic, one ball joint | 3 |
| thigh ×2 | [dL] | hip, ball joint | 3 each |
| shank ×2 | [dL] | knee, hinge | 1 each |
| foot ×2 | [dL] | ankle, two axes (flexion, inversion) | 2 each |
| upper arm ×2 | [dL] | shoulder (the shoulder girdle folded in), ball joint | 3 each |
| forearm ×2 | [dL] | elbow hinge plus forearm rotation | 2 each |
| hand ×2 | [dL] | wrist, two axes (the cock is radial–ulnar deviation; flexion–extension) | 2 each |
| bat | physics-spec §6.2 (integrated from the profile) | welded to both hands: a closed loop | 0 (6 loop constraints) |

That is 35 degrees of freedom before constraints.

**As built** (`analysis/oracle/body3d/body.py`): the two hands and the bat are one rigid body. A hand welded to the handle moves as the handle does, so the weld is built into the body's mass and inertia instead of being twelve more constraint rows; the loop of the two arms closes through the two wrists (each two axes). That leaves 13 bodies and 49 joint rows, 29 freedoms, which is the table's 35 less the loop's 6. The force along the bat at each hand is then read from the wrist's joint force. What de Leva's table does not give is a named [E] list at the top of that file: the hip joints' spacing, the shoulders' position on the trunk, the ankle's height and place on the foot, the sole's contact points, the wrist's distance from the handle, the hands' roll about the handle (taken as the same for both), and the contact's stiffness.

- **Segment numbers.** [dL] gives each segment's mass as a share of body mass, its centre of mass as a share of its length, and three radii of gyration as shares of its length; `analysis/research/body/common.py` uses its male rows, with the alternative head (vertex–cervicale) and upper-trunk (cervicale–xiphion) rows (checked digit for digit against Table 4 by the reviews). This is **measured** for de Leva's subjects (Zatsiorsky's, mostly physical-education students; mean 73.0 kg, 1.741 m). Scaling the lengths by stature is **[E]**: de Leva gives no scaling rule and suggests measuring the individual's segment lengths. Applying the table to a 92 kg professional is **[E]**: no table measured on baseball players exists ([N4] §5), and [N4] expects errors of 10% or more on some segment. Composite segments in 3D need their inertia tensors rotated and moved by the parallel-axis rule. The probe's sensitivity (`checks.py` item 6, the drive unchanged, against the baseline's contact): ±10% on the trunk's inertia moves contact ±1.3 ms, on the pelvis's 0.1–0.2 ms, on the arms' mass ±3.1–3.3 ms; bat speed stays within 30.18–30.30 m/s against 30.25.
- **The feet.** A loaded foot is held to the ground by a contact constraint:
  - no sinking;
  - friction within a cone, μ [E] (shoe on clay, no source yet; the probe used 0.8);
  - the centre of pressure inside the sole, so the ankle torque is bounded by the load times the sole's reach ahead of and behind the ankle;
  - the foot's horizontal force and its twisting moment drawn from the same contact, resolved together as one wrench, not capped separately as the probe does (§4.2).

  The rear foot stays down through the swing and pivots onto its ball late. That is [5]'s picture and OBP-only data ([swing-synthesis §3](research/swing/swing-synthesis.md)), so it is [E] for its timing. The lead foot is a free limb during the stride. It touches toe first and then heel ([4] "stepping on thin ice"; the owner's walkthrough), and its load builds over about 170 ms ([4]: 10% BW at foot down, −340 ms, to 50% at the weight commitment, −170 ms; **measured**). [4] defines those two events by force thresholds; neither is heel contact, which the 3D foot can report directly.
- **The block.** The front leg's knee and hip extensors, driven in their own channel. After foot down they brake the body's forward motion and extend the knee: [4]'s lead knee goes from 48° at the commitment to 18° at contact, **measured**. The lateral forces on the two feet form the couple that turns the pelvis ([4]'s lead and trail GRFy, 28% and 24% BW at −123 and −127 ms; Ae 2017 in [N2]). A front knee that bends instead of extending is the "collapse" of §6. A body whose weight passes over the front foot unloads the rear foot, and the couple goes with it (lunging).
- **The hands and the bat.** Each hand is welded to the handle at its grip point [E: 0.08 m and 0.18 m from the knob]. The loop through both arms is closed by constraint forces. The grip holds as long as the force along the bat stays under a limit [E; Koike 2003's bottom hand carried about 750 N along the bat at impact ([N2] §2.1: read off a figure, five hitters, off a tee), so the limit must be above that]. A slip is not modelled in phase 1.
- **The fingers and the ball** (the pitcher, §11).
- **The head and eyes** ride on the trunk. Where the eyes are is the true body's; what the batter sees from there is the perception's business (agent-models §3.2). How a moving eye point reaches perception without the agent reading the true body is part of the clock of §5.4.

### 4.2 The probe's reduced model, and what it leaves out

`chain.py`: five degrees of freedom in the horizontal plane.

- **s**: the body's forward position along the line to the pitcher, over feet fixed 0.42·H apart. [4]'s toe-to-toe stride at foot down is 42% of height (**measured**; [4] Appendix C's is the inter-ankle distance).
- **φ1**: the pelvis's yaw, a rotor about the vertical through the body. Its inertia is de Leva's LPT plus half the thighs' [E].
- **φ2**: the trunk's yaw (MPT + UPT + head, [dL]). It carries the lead shoulder at r_sh = 0.112·H on its lead side (swing-model v3 §5.2) [E].
- **φ3**: the arms, one link from the lead shoulder to the hands, 0.30·H long [E]. It carries both arms' masses at their [dL] centres.
- **φ4**: the bat (physics-spec §6.2's default 34-in bat, its mass properties rounded to 3 figures), hinged at the hands 0.08 m from the knob [E]. Its cock is lag = φ3 − φ4. A passive stop holds the cock at most at 133° [E: [4]'s largest 3D angle between the bat and the clavicle-to-wrist line, used as a planar hinge limit; not an anatomical range]. The stop is a stiff spring–damper [E: 2·10⁴ N·m/rad, 40 N·m·s/rad] that acts only past the limit and never pulls (the first version's damper pulled the bat back by up to 71 N·m as it uncocked; fixed).

Six drive channels (§5.1):

| Channel | Acts on |
| :-- | :-- |
| legs | ground → pelvis yaw |
| core | pelvis → trunk |
| shoulder | trunk → arms |
| wrist | arms → bat |
| rear | the rear leg's push along s |
| lead | the lead leg's brake along s |

The ground:

- the vertical load is split between the feet by the lever rule on s, and always sums to one body weight (the probe has no vertical dynamics: [4]'s lead foot alone reaches 1.3 BW);
- the lead foot's share builds over 170 ms after it touches [E: the vertical dynamics left out];
- the legs' yaw torque is capped by friction: μ times the smaller foot load times the stride (a lateral couple), plus each foot's free moment, μ·0.05 m·load [E];
- the push and the brake are capped by friction on their own foot, separately from the yaw cap;
- across the line of the stride the base is held on its line by whatever sideways force that takes, uncapped. Near contact that is about 740 N (0.8 BW, Neon's measurement), so with the forward force the feet would need more than friction allows.

The trunk-on-pelvis rotation has a passive range of ±45° [E: no source read].

**What the reduction leaves out** (phase 1's job):

- Everything vertical: the bat's elevation and attack angle, the trunk's tilt and side bend, the knees' bend and the pelvis's rise. The swing plane is taken as horizontal, so the planar chain cannot express "barrel under" or "over". The bat's speed is also flattered: [4]'s bat is 30° below level at contact, and the same curves read in 3D give 27.0 m/s at the sweet spot (`inverse.py`) against the flat fit's 30.2.
- The real vertical dynamics of weight transfer, for which the lever rule with a ramp stands in. The flat body cannot fall or lose a foot's support.
- The elbows: the arm link has a fixed length, so the hands' path cannot widen or narrow except by the wrist. Casting enters at the wrist only.
- Two arms in a loop: one arm link carries both.
- The muscles' force–length relation.
- One coupled friction wrench per foot (above).

The model answers questions about the drive, control, errors, ratings, replay and cost only as far as the yaw chain carries them; §4.3 shows where it stops.

### 4.3 How well the reduced model reproduces the measured swing

**The free fit** (`fit.py --rules free`): 17 drive numbers, by Nelder–Mead, against [4]'s pelvis, trunk and bat-azimuth rate curves from −250 ms to contact. Each misfit is scaled by a band read off [4]'s graphs [E: pelvis 80, trunk 100, bat 250°/s, about one between-batter SD], plus contact at 0 ms, the lag at contact (65 ± 10°, [4] Table 1), the separation's peak (17.1 ± 4° [E: Table 1's ±4° is at contact]) and the body's forward travel. The cost is a sum of squared scaled residuals per sampled time, a fitting criterion, not a statistic. Every onset and switch-off is held inside the scored window, so no channel can drift unseen; the fit starts cold from a documented guess (`X0`, from `inverse.py`'s torques) and from two perturbed starts, so a rerun reproduces the tracked file. The three starts end at costs 0.82, 0.55 and 0.62; the best is kept. Output `fit_free.out`, `swing.out`.

| Quantity | Model (best start) | Measured |
| :-- | :-- | :-- |
| pelvis peak rate | 582°/s at −58 ms | 581 at −70 [4]; 714 (SD 76) at −75 [3] |
| trunk peak rate | 703°/s at −59 ms | 766 at −57 [4]; 937 (SD 102) at −65 [3] |
| arm peak rate | 1,129°/s at −58 ms | 1,160 (SD 96) at −65 [3] (a different observable: [3]'s arm angle) |
| bat azimuth peak rate | 2,452°/s at 0 ms | 2,435 at −10 [4] |
| hip–shoulder separation peak | 16.0° at −122 ms | 18 at −103 [4] |
| bat lag, largest / at contact | 134° / 49° | 133 / 65 (SD 10) [4] |
| pelvis / trunk at contact | 64° / 62° | 77 (SD 6) / 75 [4] |
| sweet-spot speed at contact | 30.2 m/s | 29 at contact, 31 peak (barrel end, tee) [3]; [4] none; MLB average 32.1 (Statcast, [S10]) |
| residuals, RMS in band units | pelvis 0.35, trunk 0.39, bat 0.38, travel 0.06 | — |

- **Where it falls short.** The pelvis and trunk end 13° and 12° short of [4]'s turn at contact (2 SD; session 3 of phase 1 found that [4]'s own rate curves integrate to 12° and 13° less than its angle curves over this window, so this shortfall is the data's, §4.4); the bat peaks at contact, not 10 ms before it; its cock at contact is 49°, not 65°. The arm link has no elbows, so nothing slows the hands before contact as [4]'s lead arm does, and nothing carries the lead hip back as the front knee's extension does ([3]).
- **What the fit cannot tell apart.** Between the three starts the brake runs 12–682 N and the push 5–382 N with almost the same cost: the fit does not identify them. Strength, rise time, switch-off and the force–velocity curve trade against each other.
- **The wrists.** With the wrist channel visible, the three starts put 1.1, 0.3 and 12.6 N·m on it (the last switching on 11 ms before contact): effectively no drive. The fit's statement is that the bat needs no wrist drive in this model; it is not evidence that real wrists are inactive (the flat arm already over-uncocks, to 49°).
- **The strengths it used.** Isometric T (applied peak in the swing): legs 1,234 N·m (201), core 292 (196), shoulder 253 (150), push 124 N (61), brake 243 N (263 with the eccentric gain). Against sources: the core at 0.52 rad/s is 264 N·m, against professional golfers' 141 ± 31 N·m at 74.7 kg, 1.765 m ([N5] §2, FT), which is 180 ± 40 scaled by weight × height to 92 kg and 1.84 m: 1.5 × the mean. The legs' 1,234 N·m is about four times one hip's extension strength (Anderson 2007, [N5] §1), but the legs' channel is not one hip (below). The model's Hill curve falls 24% from 0.52 to 2.1 rad/s against the golfers' 17%. No measurement exists above about 5 rad/s ([N5] §6): every drive's fall at swing speeds is extrapolation.
- **The arms start too early.** The best start begins the shoulder's drive at −237 ms, the core's at −238 and the legs' at −229: the arms 67 ms before [4]'s first move (−170 ms), which the design takes as the earliest the decision allows (§5.4).

**The fit under the design's own rules** (`fit.py --rules design`, 4 starts, output `fit_design.out`). The rules, set before the fit was run:

- the shoulder's and wrist's onsets no earlier than −170 ms;
- every rotational rise at least 50 ms [E: Maffiuletti et al. 2016 (abstract only) describe the early force development over the first 50–75 ms; that is not a measured rise time, so the floor is an assumption];
- strengths inside bounds argued from sources:
  - **legs.** The pelvis's yaw drive is the couple of the two legs' horizontal foot forces across the stance, plus the hips' rotators through each foot's free moment, not one hip's torque. [4]'s lateral forces push the rear foot toward the plate and the lead foot away, which is the rear hip's extension moment and the lead hip's flexion moment acting through legs about 0.85 m long [E]. Anderson 2007 (recreational adults, FT): 267 and 188 N·m at 92 kg and 1.84 m, so foot forces of 314 and 221 N, a moment of about 206 N·m about the stance's middle. The rotators add 2 × 40–60 N·m [E, no source], and athletes perhaps × 1.3 [E]: 380–430 N·m. The bound is 0.30 × weight × height = 498 N·m. The ground caps it again by friction, about 300 N·m with both feet loaded: that, not the hips, is the binding limit;
  - **core** 288 N·m: the golfers' mean + 2 SD, scaled by weight × height, divided by the model's force–velocity factor at the tested speed;
  - **shoulders** 200 N·m [E: two shoulders at about 100 N·m each; the nearest sourced values are pitchers' internal rotation, 66 concentric and 80 eccentric, and a golf model's 80 N·m shoulder abduction, 2nd hand];
  - **wrists** 20 N·m (two wrists, 11.0 and 9.5 N·m in radial and ulnar deviation, Delp 1996, abstract only), and the cock held by at most 20 N·m instead of the stiff stop;
  - **push and brake** 1.5 × [4]'s peak mean trail and lead GRFx (21% and 50% BW) [E margin].

**Result: the flat chain does not reach the ball.** In all four starts the search ends on the bounds (every strength at its upper bound but the brake, every rise at 50 ms, the arms at −170 ms, the legs at the window's start), at a cost near 1,128, with the bat 11.3° short of contact at +30 ms. Run on (`limits.py`), the bat reaches contact at +35 ms at 26.4 m/s, with the trunk spun 146° open while the arms catch up.

**Which rule binds** (`limits.py`, one rule relaxed at a time and re-fitted; a diagnosis of the reduced model, not a fit the design adopts):

| Relaxed | Result |
| :-- | :-- |
| the arms' onset free | contact +5.8 ms, residuals about 1 band unit, 28.1 m/s; the shoulder then starts at −249 ms |
| shoulder bound doubled (400 N·m) | contact +15 ms, 31.1 m/s |
| no 50 ms rise floor | contact +21 ms |
| legs' and trunk's bounds × 1.5 | contact +30 ms |
| the stiff cock stop instead of 20 N·m | no contact |

The decision clock binds: in the flat chain, a rigid arm turned by the shoulder alone needs about 240 ms of drive to bring the bat round, and the clock gives it 170. That points at what the flat chain lacks: the elbows (the rear arm's extension and the lead arm's straightening move the hands with far less inertia than turning the whole arm), the second arm's own drive, and the front leg's block (which stops the body and speeds the trunk; the flat chain's brake is weak and unidentified). It also leaves one question about the clock itself: [4]'s "first move" at −170 ms is the hands' visible motion at 56 mph, and the arms' drive may start before the hands visibly move (electromechanical delay, and a pre-commit load of the hands on every pitch). Phase 3 settles the clock at game speeds; phase 1 must show the 3D body swings within it. **If the 3D body also cannot, the work stops there** (§13).

**The wrists' hold** (`checks.py` item 5, on the free fit):

| The cock held by | Contact (−8°) | Bat speed | Lag, largest / at contact | Stop's peak torque | The fixed ball point: when, where on the bat |
| :-- | :-- | :-- | :-- | :-- | :-- |
| the stiff one-sided stop | +1.4 ms | 30.25 m/s | 134° / 49° | 261 N·m | 0 ms, 0.634 m |
| the stop capped at 100 N·m | +0.2 | 30.31 | 135 / 45 | 100 | −0.6, 0.615 |
| capped at 20 N·m (two wrists, sourced) | +5.2 | 30.15 | 139 / 63 | 20 | +1.6, 0.737 (10 cm toward the end) |
| nothing | +13.1 | 28.47 | 143 / 84 | 0 | +8.4, a miss: the bat passes the ball 0.92 m out, beyond its 0.784 m end |

So what holds the cock does decide where the ball meets the bat. The stiff stop is a device that carries far more than hands can. In 3D the cock is held by the two hands' couple, each wrist bounded by its sourced strength, and a passive limit (if any) must have an anatomical value, never pull, and have its work and reaction recorded. Phase 1 runs with and without it.

### 4.4 The three-dimensional body: where phase 1 stands (sessions 1 to 5)

**In one paragraph.** The engine, its checks, a compiled probe of it (§8.2) and the placed, moving body are built. Session 2's whole-body fit under the design's rules did not reach the ball (41–52 ms late at 17–18 m/s). Sessions 3 and 4 found why: the first limits were too tight for the legs at speed, and the trunk's apparent excess was not robust. The owner ruled that limits come only from measurements (PLAN D34); §4.5 writes them down. With measured limits at the legs and trunk the lower body came inside the gate (§4.6); with them at the arms too the body reached the ball, the trunk's curve and the bat's speed still outside (§4.7). **Session 7 found that the bat's "missing" speed in the measured motion is mostly a measuring effect (§4.8: settled that it is one, an estimate how large), and gave every designed freedom its drive on measured limits: the fitted swing now meets every gated curve and angle, contact +0.8 ms; the bat is at 21.3 m/s and comes down on the ball at 7°. The same limits allow 30.6 m/s only with a swing no hitter makes; held to a hitter's swing, one search found 22.5 (§4.9). The owner has ruled that the measured curves' last 40 ms are not a target (PLAN D35).** **Corrections to this paragraph (session 8's review): the "30.6 m/s" was energy the engine's joint stop created (15.6 with the stop fixed), and the fitted swing's 21.3 is 17.8. Since then the owner has dropped [4] as target and gate (PLAN D37) and answered Q59 = a; session 8's practice runs, the review's findings and the reruns are §4.10. With the stops fixed and the joints held to their ranges the practice search finds the first swing on stops that make no energy: it meets a mid-zone fastball within 2 cm at 17.8 m/s (40 mph), an arms-and-shoulders swing whose hips barely turn. It was found from the plan fitted to the lab study and its launch posture; the two starts that owe it nothing did not become swings. It leans on joint ends far beyond what a joint can take and its rear wrist is outside a wrist's range, so it is not yet a swing a body could make; the "For the owner" block below is rewritten from that and from the second review.** **Session 9 (§4.11) built the three parts those reviews found missing: each hand's own grip with a wrist's, a forearm's and an upper arm's real ranges; a muscle tensed before the pitch that only holds; the feet's measured grip on dirt and the ankles' ends. With real hands session 8's swing cannot be made, and the search finds none from the same three starts: the nearest motion passes 7.9 cm from the ball at 19.0 m/s with the arms against four or five joint ends. Whether the body can swing is not shown either way: the search stalls, the start taken from [4] sits at two joint ends, and no plan has been made for this body.** **Session 10 (§4.12) took both away: the start is [4]'s launch posture moved by a rule until every joint is 5° inside its end, and the search uses only the order of whole runs' scores at a 0.25 ms step. That body then swings: the ball met within 0.2 cm at 17.7 m/s (40 mph), legs then trunk then arms, settled, the same at three step sizes; and a plan written by hand, owing [4] nothing, meets the ball at 10 m/s. Nothing reaches the gate's 29 m/s; whether the body can is not shown. A test built to settle that without a search (a hand-built motion solved for its joint torques) did not decide: no motion I built was one a body could make. The session's review then measured two parts of the model nobody had looked at: the joints' damping, an unsourced number, takes 64% of the muscles' work in that swing, and the constant hold-still torques, which no muscle rule limits, give 27% of its energy; and nine of its 41 muscle groups are within 8% of their strength. So what holds it at 40 mph is not known, and those two come first.** The history further down stands as it was found; its numbers are from the limits of the time, and every swing in it was matched to [4].

**For the owner** (rewritten after session 10 and corrected after [its independent review](reviews/2026-11-26-opus-body-3d-s10-review.md), 2026-11-26; the numbers are §4.12's; session 9's version of this block is in this file at commit `d9e76ee`, and this block before the review at `9d4aa6d`). In baseball terms:

- **Can this body make a swing that meets the ball at 65–76 mph? As far as the runs show: it makes a swing that meets the ball, at 40 mph. Whether it can at 65 is not known. What holds it at 40 is not known either, and the review found two parts of the body model, neither with a measured value, that have a large hand in it.**
- **What the batter does now.** He has a swing again, with real hands. Against a middle-of-the-zone 85 mph fastball he starts his legs first, his trunk 28 ms later and his arms 51 ms later, and **puts the barrel through the ball within a tenth of an inch at 40 mph (17.7 m/s)**, the bat travelling nearly level (3.6° downward; hitters come up through it at 8–10°), its barrel hanging steeply, 55° below his hands. It is the same speed session 8's swing had, but that swing needed a back wrist no man has; this one keeps both wrists inside a real wrist's reach up to contact (on the untilted wrist range we chose); after contact the front wrist goes 3–4° past.
- **What was built to get there.**
  - *His starting posture was moved off the ends of his joints.* It is still the lab study's launch posture, changed by a rule and by no more than 2.5° anywhere, so that every joint starts at least 5° short of its limit (before, five joints started within 5° of a limit, his back wrist beyond a real wrist's reach among them).
  - *His wrists got a real wrist's reach:* a wrist bent right back cannot also be tilted right over, and now his cannot.
  - *The search that finds his swing was replaced.* The old one felt its way by small differences and stalled. The new one tries 24 whole swings at a time and keeps moving toward the better ones. It did not stall once in ten runs, and every result is the same at half and at a quarter of the time step up to contact (what his arms do in the follow-through still changes with the step).
- **What passed** (the best swing): he meets the ball (0.2 cm; the gate asks 1); legs, then trunk, then arms; he does not jump; no stop, damper or the ground adds energy; the constant holding torques do (157 J of the swing's 574 J), and they follow none of the muscles' rules.
- **What failed, with margins** (the gate's figure in brackets): the bat's speed, 40 mph (65–76: 25 mph short); the bat is past its fastest at contact, at 92% (98%); his hands, 17 mph (23–29); the bat's angle through the ball, −3.6° (−3 to +11: by 0.6°); his hips' turning, 361°/s (555–855) and his shoulders', 414 (755–1,075); his arms start 3 ms earlier than he could have decided; his front leg: the knee locks dead straight (2–26° bent) and **his front foot carries nothing at contact** (it should take 1.5–2.6 times his weight on the way); his balance, 0.4 cm outside; his front forearm ends 6° past a forearm's range after contact, and his front upper arm is turned 2.6° past a shoulder's before it; and he leans on the ends of fifteen joints far harder than a joint can take (the front knee thrown straight at 870°/s into 1,005 N·m, mostly by the holding torque there and not by his thigh; your ruling D39, that a swing may not do this, is not built yet). Only one starting plan reaches this speed; the gate asks for two that agree.
- **Is he short of muscle? Not known.** Nine of his 41 muscle groups work at 92–97% of their strength in the best swing, and the search is charged for effort, so strength may well be part of what holds him. Two parts of the body model also matter, and neither has a measured value: the joints' built-in damping, which soaks up about two thirds of what his muscles put in (a test replay with a third of it swung the bat about 3 m/s faster), and the constant holding torques that keep him standing at the start, which never let go and give the swing about a quarter of its energy, most of it at the knees. *(I had written "Not in any one place the runs can see … what holds him at 40 mph is where the search came to rest, not a muscle on its limit". The review measured otherwise, and my own re-reading gives its figures.)* Taken all together, his muscles' best power over a fifth of a second is 3.4 times the energy a 65 mph swing holds, before what the damping takes: enough on paper, by a margin that is narrower than that reads.
- **Two things about how his body is built that came out of this session,** each resting on choices in the model that are named with it.
  - *With his front foot where the lab study's launch posture puts it, his front hip runs out of turn at about 70° open* (the lab study's hitters are at about 77° at contact). That rests on the foot's place, the average resting range of a hip's turn (an active range would give 11° more) and a knee and foot that do not twist.
  - *He can only just reach this pitch in balance.* A posture that puts the barrel on a middle fastball with his weight over his feet and every joint inside its range exists, by one degree, and it has him sitting into it with his front knee bent 38–48° where a hitter's is nearly straight. That rests on the same front foot and hip, on where the ball is met, and on a search for the posture that may be a degree or two short of the best.
- **How long he watches the pitch: nothing shown yet.** Made to watch 0.20 s (the game's own figure) instead of 0.10 s, his arms have to wait. In one search that had not finished, his trunk turned faster at 0.20 s than at 0.10 s, and in its continuation his hips did too (544°/s); his arms still started 12 ms too early, and a third of that swing's energy came from the holding torques. Whether the viewing time causes it is not shown. Nothing is put to you on it.
- **What is not known.**
  - *Whether he can swing at 65 mph.* The search is local: it improves the plan it is given. The fast swing came from session 9's plan, which came from the lab study's; a plan written from nothing got to the ball at 23 mph.
  - *What his swing is with the damping at a measured value, and with holding torques that let go or obey a muscle's limits.* Neither has been run. The first should make him faster and the second slower; by how much is the next session's first business.
  - *What a swing that does not lean on joint ends looks like* (D39), faster or slower.
  - *Whether his back foot's pivot matters.* Given a working back calf he hardly used it.
- **Something that did not work.** I built swings by hand, meaning to check each joint's effort against its muscle without any search, as the review asked. None of the 64 I built was a motion a body could make (the arms passed through impossible postures on the way from the start to contact), so that test told us nothing about his strength. That is my construction's failure, and it is said as that.
- **Where this leaves your ruling (PLAN D41).** You allowed up to three more sessions; this was the second, and one is left. He now swings, 25 mph short of the gate. I do not expect one session to close 25 mph, but two unmeasured parts of the model (the damping and the holding torques) are worth testing first. So it should be spent finding out why he sits at 40, in this order: the damping set from a measured source; holding torques that let go or follow the muscles' limits; then the same search run to a finish at both viewing times; a stronger batter, a smaller one, more turn at the hips and the ball met further back, one at a time; your ruling on joint ends built; and the hand-built test done properly, with the motion's path searched for.
- **Your rulings, written in, unchanged:** the lab study is dropped as target and gate (D37); the bat's speed is read against 65–76 mph (D38); a joint's end is to take the motion's energy and the search is to avoid load on it (D39, not built); the gate is §13's (D40); up to three more sessions, now one (D41).
- *Waiting on you:* nothing.

**Can the legs turn the pelvis in time? Yes** (`fit3d.py --lower`: the 14 leg and trunk channels, the arms undriven, the design's rules, three cold starts each, 40 iterations). Hip abduction and adduction were added first, each bounded at 132 N·m: abduction from college pitchers' isometric 109 ± 16 N·m (mean + 2 SD scaled by weight × height; [N5] §2, second hand, hand-held dynamometer), adduction set equal [E: range 0.7–1.4 ×; the one comparison found has non-athletes' adductors at 70 N·m against abductors at 87, read from a search summary only]. The weightings were stated before each run:

| Fitted against | Pelvis rate at −93 ms, three starts ([4]: 558°/s) | Lead foot's peak load ([4]: 130% at −93 ms) | Note |
| :-- | :-- | :-- | :-- |
| the pelvis's rate only | 588, 482, 533 | 170% at +55 ms, 83%, 100% | what the legs can do for the turn if nothing else is asked |
| pelvis rate, travel, lead knee | 519, 575, 511 | — | |
| those and both feet's three forces and the trunk's rate (session 2's stage A) | **569, 567, 497** | **128% at −115 ms, 125%, 136%** | residuals of the best start in band units: pelvis rate 0.68, travel 0.19, the feet's forces 0.67–2.09, lead knee 0.72 |

- **The weighting was not the fault.** With the ground forces in the fit, the pelvis is on time and the lead foot is loaded as measured. The knees still reach their stops (278 and 348 N·m).
- **Why it was late in the whole-body fit.** Without the arms the pelvis then slows early (449°/s at −70 ms; turned 62° at contact against 77°), and the ground's turning moment about the pelvis's vertical peaks at 149 N·m at −140 ms (from [4]'s forces: 164 N·m at −140 ms) but sums to only 2–4 N·m·s from −170 ms to contact. Turning the pelvis is cheap. Turning the trunk, the arms and the bat behind it is what costs, and that is what the next table measures.

**What [4]'s swing asks of each joint** (`inverse3d.py`; no search). The body is placed on [4]'s curves every 10 ms, velocities and accelerations are differences of those postures (made exactly consistent with the joints), and the engine's equations are solved for the joint torques. The legs' torques come up from [4]'s measured ground forces, so they do not depend on the upper body's reconstruction; the trunk's and arms' come down from the reconstructed upper body. Hip torques are along the thigh's own three axes, the way hip strength is tested.

| Joint and direction | Needed at its peak (when) | Range over the cases below | Bound (full strength) | What the force–velocity law allows at the joint's speed then |
| :-- | --: | --: | --: | --: |
| trunk rotation toward the pitcher, about the trunk's own long axis (session 4) | 321 N·m (−90 ms) | **216–355** | **288** | 239 (1.0 rad/s) |
| the same about the pelvis's long axis (session 3's row; not the axis strength is tested about) | 438 (−90) | 329–439 | — | — |
| rear hip extension | 256 (−120) | 237–256 | 348 | **175** (3.3 rad/s) |
| lead hip adduction | 123 (−140) | 123–134 | 132 | **35** (7.1 rad/s) |
| lead knee extension | 250 (−110) | 220–250 | 352 | **204** (2.5 rad/s) |
| lead hip abduction | 180 (−10) | 87–180 | 132 | 166 (being stretched) |
| rear hip rotators | 68 (−90) | 18–68 | 78 [E] | 26 (5.6 rad/s) |
| lead hip rotators | 236 (−90) | 37–236 | 78 [E] | 23 (6.4 rad/s) |
| lead hip flexion | 114 (−140) | 92–114 | 244 | 269 |
| rear knee flexion | 114 (−140) | 102–114 | no channel in the body (Anderson's value × 1.3 would be 188) | — |
| ankles' plantar flexion, rear and lead | 88, 70 | 63–90 | 205 | 106, 250 |
| each shoulder about the trunk's axis | 47, 57 | 34–57 | 100 | 56, 111 |
| rear elbow extension | 61 (−80) | 48–61 | 70 | 39 (4.8 rad/s) |

- **The cases in the range:** differencing over 10 and 15 ms; the pelvis's and trunk's rotation read from [4]'s rate curves instead of its angle curves; the turning moment [4]'s forces do not explain put on the pelvis directly instead of under the feet. Differencing over 5 ms doubles most peaks (trunk 756 N·m) and is noise.
- **Over its full strength: nothing for certain.** The trunk's need is 0.75 to 1.23 times its bound (session 4, below). Everything in the legs is inside its bound, with the lead hip's adduction at it. The lead hip's rotators are over their [E] bound only if the unexplained turning moment (below) reaches the ground through that leg.
- **Over what the force–velocity law leaves at the joint's speed:** the trunk if its need is at the upper end (0.9–1.5 ×), the rear hip's extension (1.5 ×), the lead knee (1.2 ×), the rear elbow (1.6 ×), the lead hip's adduction and both hips' rotators (2.6 × and more). The law gives 75% of full strength at 1.25 rad/s and 50% at 3.3; Anderson 2007 measured 75% at 1.6 and 50% at 3.2 rad/s for hip extension in recreational adults ([N5] §1). So the law is the measured one where there is a measurement, and the swing needs more than it.
- **Not built:** the rear knee needs 114 N·m of flexion and the body has no knee flexors.
- **How far to trust it.** The legs: well, because they are worked up from [4]'s measured forces. The trunk: not well (next paragraph). [4]'s forces leave up to 1,700 N unexplained on the body, vertical and horizontal alike. The hands' path is [E] ([4] does not report it); between −85 and −30 ms the two elbows and the lag cannot all be met from these shoulders, by up to 7°, and the nearest smoothed path is used.

**How solid is the trunk's torque?** (`trunk3d.py`, session 4; Newton's and Euler's equations summed over the segments, no solve, no fit; nothing in the model changed.) Three ways to the torque on the trunk at the lumbar joint, about the trunk's own long axis:

| Estimate | Axial torque at its peak | When | What it rests on |
| :-- | --: | --: | :-- |
| from above, as placed | 321 N·m | −90 ms | the reconstructed upper body, [4]'s angle curves, differencing over 10 ms |
| from above, differenced over 15 ms | 235 | −100 ms | the same, features under 60 ms smoothed away |
| from above, pelvis and trunk from [4]'s rate curves | 247 | −100 ms | [4]'s rate curves, which turn 12–13° less than its angles |
| from above, both of those | 216 | −100 ms | |
| from above, the hands' path varied (both elbows ± 8°, lag ± 10°, smoothing twice as wide) | 283–329 | −90 ms | [4]'s between-batter SDs as the range of the unreported path |
| from above, the trunk's bend varied (upright to 30° forward; 10° to either side; the lean kept toward the plate) | 294–326 | −90 ms | the [E] bend |
| from above, arms' and bat's masses and inertias ± 10% | 296–345 | −90 ms | de Leva on a professional |
| from above, the joint 3 cm forward or back, to either side, up or down | 286–355; 314–327; 321 | −90 ms | where the one lumbar joint is put |
| **from below**, [4]'s ground forces through the legs and pelvis, no free moments | **153** (131–172 over the bend and where on the foot the force acts) | −150 ms | the measured forces, the feet's places, the pelvis's and legs' motion |
| from below, with the largest free moments friction allows (58–117 N·m) | about 190–290 | | the friction's reach [E] |
| the upper body's angular momentum about the vertical | a mean of 198 N·m from −170 to −50 ms (it gains 23.7 N·m·s in 120 ms, then 1.8 in the last 50) | | the reconstruction's upper body; a mean is a floor on the peak |
| [4]'s trunk rate curve alone | 65 for the trunk and head; 176 if the arms and bat were carried rigidly | −120 ms | the curve's steepest rise, 160 rad/s² |

- **Which axis.** Session 3's 330–440 N·m was the lumbar torque about the pelvis's long axis. The torque there is mostly bending, not twisting: at −90 ms it is 321 about the trunk's long axis, 521 in side bend and 189 forward and back, 721 N·m at its largest near contact. With the trunk leaning 15° the pelvis's axis takes in part of the side-bending moment, which made 445 of 321. Strength is tested about the trunk's own axis with the pelvis held: 321 is the number to set against 288.
- **From above and from below do not agree,** and they must. At −90 ms they differ by 346 N·m about the vertical, where the feet's free moments could explain at most 58–117. So at least 230 N·m of that is error, in the upper body's reconstruction, in the reading of [4]'s forces, or both.
- **What the unexplained force does to "from below", component by component.** It reaches 1,155 N toward the plate, 1,027 N toward the pitcher and 1,728 N vertically. A vertical force has no moment about the vertical, so it does not touch the axial part; it leaves the two bending components undetermined (from above and from below differ by up to 568 and 758 N·m there, so neither bending number should be used). The horizontal part, up to 1,226 N between −150 and −60 ms, moves the axial estimate from below by 123 N·m for every 0.10 m between the joint and wherever that force really acts. So the axial part survives only loosely: 153 N·m, give or take about 120, plus the free moments.
- **What reaches the upper body other than through the joint's torque.** As placed, the upper body's centre of mass is 0.12 m to the side of the joint (the 15° bend and the arms), and the joint's force acting at that distance takes away 73 N·m on average from −170 to −50 ms, so the joint's torque about the vertical has to average 261 N·m to give the 198. With the trunk upright there is no such term and the peak about the vertical is 306 N·m. That offset is [E].
- **What moves the number most:** how the curves are differentiated (10 against 15 ms: 321 against 235) and angle curves against rate curves (321 against 247). The hands' path, which was the suspect, moves it by 40 N·m; the bend by 30; the joint's fore-and-aft place by 35 each way; its height not at all.
- **Is the need above 288 N·m robust? No.** From above it is 216 to 355 N·m; from below, 131–172 before free moments and at most about 290 with them. The bound sits inside the range.
- **By how much, at the least and at the most:** at the least it is not over at all (216 from above is 0.75 of the bound, and the measured forces alone give about half of it); at the most it is 355 N·m, 1.23 times the bound. Against note 8's 245–275 N·m for the trunk at this speed the same range is 0.8 to 1.45 times.

**The turn about the vertical** (the same run; N·m·s about the vertical through the centre of mass; the range is differencing over 5, 10 and 15 ms, then the rate-curve reading):

| | The motion needs | [4]'s ground forces give | Left for the feet's free moments, which [4] does not report |
| :-- | --: | --: | --: |
| −250 ms to contact | 33.1–35.0 (31.0) | 23.5–23.8 | 9.6–11.3 (7.2) |
| −170 ms to contact | 28.7–31.0 (27.3) | 16.8–18.1 | 11.9–13.0 (9.2) |
| −250 to −93 ms | 18.6–22.6 (20.3) | 18.9–19.2 | −0.6 to 3.4 (1.4) |

In three dimensions [4]'s forces give two thirds of the turn (the flat analysis of §5.2 had 13 of 36). Up to −93 ms they give all of it. The shortfall is in the last 93 ms, 6–9 N·m·s, an average free moment of 60–100 N·m, where friction under the feet allows about 58–117 N·m at −90 ms and half that at contact (0.8 × the load × a reach of 0.05–0.10 m [E]). So the ground can plausibly supply the turn; it is not what is short.

**[4]'s angle curves and rate curves disagree.** From −250 ms to contact [4]'s pelvis angle turns 97.5° and its own pelvis rate curve integrates to 85.2°; the trunk's 94.7° and 81.8°; 10 of each 12–13° is in the last 93 ms. (The bat's agree: 221.6° and 223.7°.) [4]'s text has the same gap (19° to 77° in 93 ms is a mean of 624°/s; the stated peak is 581°/s). The flat fit of §4.3, fitted to the rate curves, ended 13° and 12° short of the angles at contact: that was this, not a fault of the flat chain. The gate of §13 asks for both the rate curves and 77° at contact, which cannot both be met. The placement uses the angle curves; the fits' misfit uses the rate curves.

**Not run this session:** the whole-body fit again, the rules one at a time, the small frame and the wrists (the job's step 3), because of the stop above. Session 2's results for them follow and stand as they were, from a body with 16 channels (no hip abduction or adduction).

Everything below is from `placed3d.py`, `fit3d.py` and `probe3d.py` (§3), for [4]'s mean batter (1.84 m, 92 kg), from −250 ms.

**The two knees are resolved by the heels and [4]'s own pelvis obliquity** (`body3d.body.pose`). With both feet flat and the pelvis level, the lead knee's measured bend asks for a pelvis 9.5 cm lower than the rear knee's does at −340 ms, 6.2 cm at −250 ms, 1.3 cm at −170 ms, and 12.9 cm *higher* at contact: a steady drift of 22 cm. Three things [4] reports or implies close it:

- [4]'s stride is the distance between the ankle joints (its Appendix C code), and its pelvis position is measured from the centre of home plate, so where the pelvis sits between the feet is not given. The flat probe's reading (the zero at the lead ankle) is kept: it puts the centre of mass within 4–6 cm of where [4]'s own ground forces say it must be from −340 to −250 ms. Moving the pelvis forward until both knees fit with flat feet would need 8–12 cm and would then make the pelvis travel *backward* after −170 ms, against [4].
- **Early, the lead heel is up** (toe touch, "stepping on thin ice"): raised on the ball of the foot by 34° at −250 ms, 18° at −200, 7° at −170, down by −120 ms, both knees then have their measured bends. At −340 ms even a 60° heel [E limit] leaves the lead knee 4° short.
- **Late, [4]'s pelvis obliquity** (trail hip toward the ground: 1° at −170 ms, 14° at −93, 27° at contact) takes the 12.9 cm to 4.6 cm, and **the rear heel rising** (7° at −120 ms, 16° at −93, 22° at contact) takes the rest.

How that changes the weight on the feet: not the starting loads, which are [4]'s (below), but what the legs can do with them. Session 1's flat placement had the pelvis 6 cm too low with the rear knee at 62°, and undriven the lead foot never took more than 18% of body weight; with the heel placement the pelvis is at 0.93 m and the lead foot is on its ball with the heel coming down as the weight arrives, which is the walkthrough's heel plant.

**The start is moving, and every velocity is a slope of [4]** (`body3d.body.place`). The body is placed 10 ms before and after the start as well, and each segment's velocity is the difference, so it is the slope of the same curves the posture is built on:

| At −250 ms | Value | From |
| :-- | :-- | :-- |
| the centre of mass | 0.62 m/s toward the pitcher (58 kg·m/s), 0.05 m/s away from the plate | the result of all the rest |
| the pelvis's forward speed | 0.69 m/s | [4]'s pelvis travel (slope 0.69 over ±10 ms, 0.71 over ±20) |
| pelvis, trunk, bat azimuth rates | −1, −36, 0°/s | slopes of [4]'s angle curves ([4]'s own rate curves, not used, say −4, −49, −11) |
| bat elevation, lag, lead and rear elbow | −22, −31, −31, 0°/s | slopes of [4]'s curves |
| lead and rear knee | 0, −50°/s | [4]'s curve; [4]'s text values interpolated [E] |
| the lead heel coming down | 308°/s | the heel angles placed either side [E: follows from the knees] |
| each foot's ground force | lead 14% of body weight up, 1% back; rear 65% up, 18% toward the pitcher | [4]'s force curves: each sole point starts as deep, and its friction spring as stretched, as that takes |
| the trunk's forward bend, the pelvis's tilt, the feet's places | no rate | [E]: held fixed in the posture |

The one [E] choice in the velocities that matters is the differencing interval: over ±3 ms the digitised pelvis curve's steps give 0.41 m/s instead of 0.62 (±5 and ±10 ms agree).

**Was the missing forward momentum why the lead foot never took the weight? Partly.** Undriven from −250 ms (the starting muscle level and joint damping only):

| Start | Pelvis travel at −200, −150, −100 ms ([4]: −0.029, +0.008, +0.038 m) | Lead foot's mean load, −250 to 0 ms ([4]: 78% of body weight) |
| :-- | :-- | --: |
| moving, as placed | −0.029, +0.013, +0.066 | 24% |
| at rest | the pelvis stays back (−0.06 to −0.04) | 20% |
| velocities over ±5 ms / ±3 ms | as placed / slower | 25% / 22% |
| session 1's flat placement, moving | −0.029, +0.014, +0.067 | 16% |

The moving body follows [4]'s pelvis travel for 100 ms with no drive at all, which the body at rest does not. But the weight still does not come onto the front foot: the lead knee gives (48° to 52°) and nothing brakes the body, so the pelvis runs on to +0.20 m by contact where [4]'s stops at +0.05. The momentum was missing and is now there; the block is a drive and has to be found by the fit.

**The muscles' level at the start** (`drive.solve_tone`, with [4]'s ground forces on the feet): at most 57 N·m (rear knee), the rest 2–49. Held constant [E], with joint damping of 1 N·m·s/rad [E], as in session 1.

**The fit under the design's rules** (`fit3d.py`; a laboratory fit of the model's drive numbers to a recorded average swing, nothing more).

- **The channels (16):** rear hip extension, lead hip flexion and extension, each hip's rotators, both knees' extension, both ankles' plantar flexion, trunk rotation, each shoulder (about the trunk's axis), each elbow's extension, each wrist (uncocking). Each has a strength, an onset, a rise time and a switch-off: 64 numbers.
- **The rules, as the search's box, set before any 3D fit:** strengths inside the sourced bounds (rear and lead hip extension 347 N·m and lead hip flexion 244, Anderson 2007 × weight × height × 1.3 [E]; hip rotators 78 [E]; knees 352 and ankles 205, Anderson; trunk 288; shoulders 100 [E]; elbows 70 [E: no N·m value found, range 50–100]; wrists 9.5, Delp 1996); rises of at least 50 ms; the arms' channels no earlier than −170 ms; every onset inside the window.
- **The misfit:** [4]'s pelvis, trunk and bat rates, the pelvis's travel, both feet's three ground forces, the lead knee, both elbows, the bat's elevation and lag, each from −250 ms to 0 in band units (`fit3d.BAND`: the flat fit's bands for the rates and travel; 15% and 8% of body weight for the vertical and horizontal forces, 8–10° for the angles, all [E], set before the fit), plus contact at 0 ± 5 ms and [4]'s Table 1 at contact.
- **The search:** Levenberg–Marquardt with a finite-difference Jacobian, projected onto the box, through the compiled probe at a 1 ms step. Staged: the lower body and trunk against the ground forces, pelvis, trunk and lead knee with the arms undriven; then the arms with those held; then all 64 numbers against everything. The numbers reported are from one run of the whole body with the last stage's result (the same at a 0.5 ms step to 0.1 ms and 0.1 m/s). Start 0 is a hand estimate from [4]'s event times at half strength; starts 1 and 2 perturb it (fixed seed). 11,400 swings, 6 minutes.

| | Start 0 | Start 1 | Start 2 (best cost) | Start 2 continued 30 iterations | [4] and the gate |
| :-- | --: | --: | --: | --: | :-- |
| contact (bat at −8°) | +51.6 ms | +41.5 | +42.9 | +40.8 | 0 ± 5 ms |
| sweet spot at contact (at 0 ms) | 17.6 m/s (12.3) | 17.2 (12.8) | 17.2 (12.9) | 17.2 (13.1) | 29–34 m/s |
| bat azimuth at 0 ms | −95° | −80° | −81° | −78° | −8° |
| pelvis peak rate | 484°/s at +25 ms | 420 at +60 | 563 at +60 | 566 at +60 | 581 at −70 |
| trunk peak rate | 846°/s at +35 ms | 794 at +10 | 770 at +5 | 776 at +5 | 766 at −57 |
| lead foot's peak load | 116% BW at −90 ms | 106 at −110 | 103 at −105 | 103 at −105 | 130 at −93 |
| at contact: pelvis, trunk | 83°, 124° | 82°, 134° | 82°, 130° | 80°, 133° | 77 ± 6°, 75° |
| at contact: lag, bat elevation | 83°, −11° | 92°, −11° | 89°, −13° | 91°, −13° | 65 ± 10°, −30 ± 5° |
| at contact: lead knee, lead and rear elbow | 11°, 9°, 60° | 9°, 10°, 62° | 9°, 9°, 64° | 9°, 9°, 65° | 18°, 56°, 81° |
| cost | 186 | 148 | 147 | 141 | |

- **Residuals of the best start, RMS in band units** (the gate asks for 1; beyond 2 is "stop"): pelvis rate 2.75, trunk rate 1.64, bat rate 1.62, pelvis travel 1.79, lead foot vertical 3.24 and fore-aft 2.67, rear foot vertical 2.14, bat elevation 2.16, lag 1.16, lead elbow 1.18, lead knee 0.91, rear elbow 0.41.
- **On a bound: 32 of the 64 numbers.** Every arm channel (both shoulders, both elbows, both wrists) is at its strength bound, at the earliest onset (−170 ms) and at the shortest rise (50 ms). So are the trunk (288 N·m from the window's start), both hips' rotators, the rear hip's extension and the lead hip's flexion. The lead hip's extension and the rear ankle are switched off; the lead knee is near its bound (349 of 352 N·m, applied 327).
- **What the drives deliver** is well under their strengths, because the force–velocity law cuts them at speed: the rear hip's extension applies 179 of 348 N·m, the lead hip's flexion 188 of 244, the trunk 137 of 288, the shoulders 60–75 of 100.
- **The stops:** both knees are driven straight into their stops (peak 407 and 374 N·m), the lead knee reaching 9° where [4]'s is 18°. The lead shoulder's range stop carries at most 63 N·m.
- **Not converged to a tolerance:** the search stopped at its iteration limit, and 30 more iterations moved contact from +42.9 to +40.8 ms. The three starts agree on the picture, not on the numbers (contact +41 to +52 ms).

**Which rule binds** (`fit3d.py --limits`: the best start continued with one rule relaxed, re-fitted with every number free; a diagnosis, not a fit the design adopts):

| Relaxed | Contact | Sweet spot at contact | Pelvis peak | Note |
| :-- | --: | --: | --: | :-- |
| nothing (continued the same 30 iterations) | +40.8 ms | 17.2 m/s | 566°/s at +60 ms | |
| the arms' onset free in the window | **+15.7 ms** | 17.8 m/s | 685 at +60 | lag 76°, bat elevation −26° at contact |
| shoulders' and elbows' bounds doubled | +28.1 ms | 21.0 m/s | 362 at +60 | the lead elbow fully straight (1°) |
| no 50 ms rise floor | +30.2 ms | 17.8 m/s | 422 at +60 | |
| legs' and trunk's bounds × 1.5 | +29.4 ms | 17.7 m/s | 657 at +60 | the trunk 144° open at contact |
| wrists' bound × 3 | +32.0 ms | 18.0 m/s | 620 at +60 | |

The decision clock binds most, as in the flat chain (§4.3: +5.8 ms there). But here no single rule is enough, the bat never passes 21 m/s, and **in every row the pelvis peaks after contact.** Stronger legs do not fix that. So the arms' rules are not the first thing wrong.

**What the body lacks** was answered by session 3, above: not hip channels, not the weighting, not the force–velocity law's agreement with the one measured, but strength at speed and at the trunk. Session 2's list of candidates is in the research index's "Tried and ruled out" where each was tested.

**The wrists' cock, with and without a stop** (`fit3d.py --wrists`; a one-sided stop on each wrist's cock a margin past its angle at the start [E: the starting angle is not known to be neutral], re-fitted from the design fit):

| | Contact | Sweet spot at contact | Lag at contact | Stops' peak torque |
| :-- | --: | --: | --: | :-- |
| no stop (the design fit, continued) | +40.8 ms | 17.2 m/s | 91° | — |
| stop 5° past the start | +35.9 ms | 17.5 m/s | 84° | lead 42 N·m, rear 38 |
| stop 20° past the start | +37.8 ms | 17.3 m/s | 91° | lead 96 N·m, rear 0 |

A stop moves contact 3–5 ms and the bat's speed not at all. In the flat chain the stop decided whether the bat met the ball (§4.3); in this body the wrists are not what is short.

**The small frame of PLAN D32** (`fit3d.py --small`: 1.75 m, 77 kg [E], the same sourced bounds at his weight × height, which makes each 0.80 of the average's; re-fitted under the rules from the average's plan, 30 iterations): contact +49.3 ms, **14.1 m/s at contact against the average's 17.2 (82%)**, 10.7 against 12.9 m/s at 0 ms. Like the average he has every arm number on its bound, so "strengths at the top of the bounds" is what the fit gives him. Since neither body swings yet, this is a ratio between two bodies that do not reach the ball, not his bat speed.

**Where the fits without the shoulders' range stops went** (the first run of this session, kept as `fit3d_design_nostop.json`): contact +36 to +44 ms at 18–19 m/s, the same picture. The stops were added because the lead arm is placed at its anatomical limit across the chest; they changed nothing that matters.

**Next:** the owner's decision above. If it is (a), a research job: trunk rotation strength in hitters or other rotational athletes; hip and trunk torque at 3–7 rad/s in athletes; knee flexion strength; hip adduction strength from a dynamometer. If it is (b) or (c), then in this order: the knee flexors; the hip channels moved to the thigh's own axes (the engine needs a drive about an axis fixed in the child, which it has, and the probe's copy); the whole-body fit with all 20 channels, the rules one at a time, the small frame and the wrists; the hands' path by a continuation that keeps both elbows inside their measured bands; then the breakdowns of §6.2.

### 4.5 The limits after the owner's ruling (PLAN D34), written before the refit

The owner ruled on 2026-11-25 (Q58 = a): the body's strength limits come only from measurements. The machine-test limits that were too tight for the measured swing may be replaced by what a source supports and by nothing else. **This subsection was written and committed before any refit**, so that no limit is chosen by what the swing then does. It replaces §4.4's per-joint bounds, the single force–velocity law of §5.1 for the legs and trunk, and the gate's reading of [4]. Reading status as in [N8]: FT full text, TAB a table, AB abstract only, EXC a search excerpt, 2nd second hand; "arith." marks arithmetic done here or in [N8]. For [4]'s mean batter, weight × height is 1,660.6 N·m.

**Full strength.** One rule for every joint that has a sample: the source's mean plus two SDs, scaled by weight × height. That is how the trunk's bound was already made. It replaces "the mean × 1.3 [E]" at the legs: the 1.3 had no source.

| Channel | Bound, N·m | From | Read | Range a rerun tests |
| :-- | --: | :-- | :-- | :-- |
| hip extension, each side | 430 | Anderson 2007: C1 0.161 (SD 0.049) × weight × height, mean + 2 SD | TAB ([N5] §1) | 267 (the mean); 524 (one elite jumper's 480 N·m, Wilson 2006, TAB, scaled from 82 kg and 1.89 m; the subject's size is from Yeadon 2006 and was not confirmed for this table) |
| hip flexion | 271 | Anderson: 0.113 (0.025) | TAB | 188 |
| knee extension | 404 | Anderson: 0.163 (0.040) | TAB | 271 (the jumper's 328 N·m scales to 358, inside the bound) |
| knee flexion (new channel) | 194 | Anderson: 0.087 (0.015) | TAB | 144 |
| ankle plantar flexion | 231 | Anderson: 0.095 (0.022) | TAB | 158 |
| hip abduction | 132 | college pitchers' 109 ± 16.4 N·m at 96.1 kg, 1.89 m, hand-held dynamometer | 2nd ([N5] §2) | unchanged from session 3 |
| hip adduction | 132 | **[E]: set equal to abduction; no dynamometer value found** | — | 92–185 (0.7–1.4 ×); amateurs' swings are reported to use 221 scaled ([N8] §4, EXC), a check on what a swing asks, not a bound |
| hip rotators | 78 | **[E]: no source** (§4.3) | — | 50–110 |
| trunk rotation | 260 at 0.52 rad/s and slower | Bae 2012, professional golfers: 140.6 ± 30.9 N·m at 30°/s, mean + 2 SD, scaled | FT ([N8] §1.1) | 250–290 ([N8] §5). The old 288 was this 260 divided by the old law's factor; no higher value is used |
| each shoulder, each elbow | 100, 70 | [E], unchanged (§4.3) | — | not varied this session |
| each wrist | 9.5 | Delp 1996 | AB | not varied |

**Strength at speed.** Each joint group gets its own measured curve: the fraction of full strength at the joint's speed, straight lines between the points. Above the last measured speed every curve is an extrapolation and is marked.

| Group | Points (rad/s: fraction) | From | Read | The alternative a rerun tests |
| :-- | :-- | :-- | :-- | :-- |
| hip extension | 0: 1.00; 1.05: 0.83; 3.14: 0.70; 4.19: 0.66; 14.2: 0 | 0.83 at 1.05 rad/s is Anderson's fitted law (arith. in [N8] §2.1); athletes keep 84% of that at 3.14 rad/s (Silva 2018, runners) and 79–80% at 4.19 (Rodrigues 2023, CrossFit); zero at the elite jumper's 14.2 rad/s (Wilson 2006, set from the task, not measured as zero torque) | TAB, TAB (EXC), TAB | Anderson's own curve: 1.578: 0.75; 3.19: 0.50; 6.4: 0. **Above 4.19 rad/s: [E]** |
| hip abduction, adduction, rotators | the hip extension curve | **[E]: no torque–speed data exist for these at any speed above 2 rad/s** ([N8] §6); assumed to fall as the same joint's extensors do | — | the first law (top speed 20 rad/s, curvature 0.25) |
| hip flexion | 0: 1; 2.095: 0.75; 4.267: 0.50; 8.6: 0 | Anderson's C4 and C5; the line through them continued to zero | TAB | — |
| knee extension | 0: 1; 1.517: 0.75; 3.952: 0.50; 5.24: 0.43; 13.4: 0 | Anderson's C4 and C5; athletes keep 52–55% of the 1.05 rad/s value at 5.24 (Ning & Faro 1996; pro soccer 2011), times Anderson's 0.82; zero at the jumper's 13.4 rad/s (Wilson) | TAB; AB, EXC; TAB | **above 5.24 rad/s: [E]** |
| knee flexion | 0: 1; 2.008: 0.75; 5.233: 0.50; 11.7: 0 | Anderson | TAB | — |
| ankle plantar flexion | 0: 1; 0.987: 0.75; 3.558: 0.50; 8.7: 0 | Anderson | TAB | — |
| trunk rotation | 0: 1; 0.52: 1; 1.05: 0.906; 2.09: 0.832; 4.19: 0.671; 12.9: 0 | Bae 2012: 140.6, 127.4, 117.0 N·m at 0.52, 1.05, 2.09 rad/s; Gifford's golfers keep 81% of the 2.09 rad/s value at 4.19; the last line continued to zero | FT; EXC | **above 4.19 rad/s: [E]** |
| shoulders, elbows, wrists | the first law (top speeds 30, 35, 40 rad/s, curvature 0.25) | [E], unchanged | — | — |
| a muscle being stretched, every group | rises to 1.5 × full strength | Wilson 2006: T_max = 1.5 T0 in every generator | TAB | the approach to 1.5 keeps the first law's slope [E] |

**Tendon stretch at the leg joints,** as Yeadon, King and Wilson's generators have it: the muscle acts through a spring in series. The spring's stretch at full strength is taken from Wilson 2006's tabulated stiffness and full strength for one elite jumper (TAB): hip extension 480 N·m over 5,092 N·m/rad, 5.4°; knee extension 328 over 857, 21.9°; hip flexion 289 over 496, 33.4°; plantar flexion 283 over 767, 21.1°. The stiffness here is the bound above divided by that angle (the models' own rule: the same stretch at full strength): 4,560, 1,055, 465 and 626 N·m/rad. No value is tabulated for knee flexion or for hip abduction, adduction or rotation, so they get none. **None at the trunk** ([N8] §3.3: no value exists).

**Muscles tensed before they fire.** The legs' and trunk's drives may already be on at the start of the fitted window, each at a level between 0 and its full activation that the fit finds, and so loaded while they are being stretched ([N8] §3.1: most of the measured gain after a stretch is this active state, Walshe 1998, FT). The load and stride are on every pitch (§5.4), so this is not a decision made early. A channel holds its starting level until its onset, then rises to full. The arms' channels start at zero and no earlier than −170 ms, as before. The constant starting joint torques of §4.4 are kept: they also carry directions that have no drive (the trunk's bending, the ankles' roll).

**Parts added.** Knee flexors (above). The nine hip channels act about the thigh's own three axes (extension and flexion about its side-to-side axis, abduction and adduction about its front-to-back axis, the rotators about its long axis), the way hip strength is tested and the way §4.4's table reports the need.

**The gate reads [4]'s rate curves.** [4]'s pelvis and trunk angle curves turn 12–13° further than its rate curves integrate to (§4.4). The gate's curves are the rate curves, because the peaks and their times that [4] and [3] report are rates and because the bat's rate and angle agree. At contact the pelvis is therefore asked for what its rate curve gives from the start of the window: −20.7° at −250 ms plus 85.2°, **65 ± 6°** in place of 77 ± 6°. The placement at −250 ms uses the angle curves; the two readings agree there.

**What rests on an assumption, for the owner:** hip adduction's and the hip rotators' full strength; the speed curve of hip abduction, adduction and rotation; every curve above 4–5 rad/s; the shoulders' and elbows' strength and speed curve; the slope toward the 1.5 × of a stretched muscle. Everything else in the three tables is a measured number.

**The arms' limits (session 6), written before the arms were refitted.** The owner's ruling is applied to the arms the way the tables above apply it to the legs and trunk. This block was written and committed before any run with it. It replaces the arm rows of the three tables above (100 and 70 N·m, the first law, no tendon stretch, no starting level, onsets from −170 ms). Sources are [N9]'s, with its reading statuses. Which way each channel acts decides which measurement applies: the lead shoulder's channel sweeps the lead arm out from across the chest about the trunk's axis (horizontal abduction); the rear shoulder's brings the rear arm forward and round (horizontal adduction); the elbows' extend; the wrists' uncock the bat (ulnar deviation, both hands).

| Channel | Bound, N·m | From | Read | Range a rerun tests |
| :-- | --: | :-- | :-- | :-- |
| rear shoulder (horizontal adduction) | 123 at 1.05 rad/s and slower | Silva 2006: 23 elite junior tennis players, 90.9 ± 21.3 N·m per 100 kg at 60°/s, mean + 2 SD × 92 kg. The only sample with SDs in this direction; **boys of 14, scaled by mass**, which is its weakness | TAB | 97.5 (one elite adult, Kentel 2011, FT, his size not given, so unscaled); 136 (Holzbaur 2007, 5 men, adduction in the frontal plane, another direction, mean + 2 SD scaled) |
| lead shoulder (horizontal abduction) | 92 at 1.05 rad/s and slower | Silva: 69.8 ± 15.0 per 100 kg, the same way | TAB | 71.9 (Kentel); 112 (Holzbaur, abduction) |
| each elbow's extension | 85 | Holzbaur 2007: 5 men, isometric 60.5 ± 6.2 N·m at 81.6 kg, 1.778 m, mean + 2 SD scaled. An SD from five men is very uncertain | TAB (text layer garbled) | 52 (Kentel); 76 (Danneskiold-Samsøe, men of 20–29, mean + 2 SD, sizes unread, unscaled, EXC) |
| each wrist's uncocking (ulnar deviation) | **two readings, both run** | see below | | |

**The wrists: why the sources differ, and which the channel is.** Delp 1996 (AB): isometric, ten men, ulnar deviation 9.5 ± 2.2 N·m, so 13.9 at mean + 2 SD (sizes unread). Peleg 2025 (TAB): twenty men on a Biodex at 90°/s, 26.5 ± 6.6 N·m, 51 at mean + 2 SD scaled; Kentel's one player 41.4. All are one wrist, in N·m, in the same direction; units and counting are not the difference. The gap is not uniform across directions: against the other samples Delp's extension is 1.2–1.9 times lower, radial deviation 1.35, flexion 2, ulnar deviation 2.8. That pattern is what a hand gripping a handle would give against a hand held open, since the finger flexors, recruited by a grip, add most to flexion and ulnar deviation. **That is an inference here: Delp's fixture was not read** (abstract only), so it is not settled. A batter grips a handle, so if the inference holds the channel is Peleg's. Both ends are run as whole fits: **51 N·m** (Peleg) and **13.9 N·m** (Delp), and reported side by side.

**The arms' strength at speed.** The only arm curves to 6.1 rad/s are one elite tennis player's fitted hyperbolas (Kentel 2011, FT); samples reach 3.14 rad/s in these directions (Silva) and 5.2–7.9 rad/s only for shoulder rotation, which no channel here uses. Each curve is 1 at and below the speed its bound was measured at, then follows the sample where there is one and the player's hyperbola beyond it, as a ratio to its own value at the join. Above 6.1 rad/s the player's curve is itself an extrapolation.

| Group | Points (rad/s: fraction) | From | The alternative a rerun tests |
| :-- | :-- | :-- | :-- |
| rear shoulder, horizontal adduction | 0–3.14: 1.00; 4: 0.865; 5.24: 0.705; 6.1: 0.612; 9: 0.376; 13: 0.167; 18.05: 0 | Silva: 100% kept from 1.05 to 3.14 rad/s (TAB); beyond, Kentel's hyperbola (top speed 18.05, ω_c 6.43 rad/s) relative to its value at 3.14 | Kentel's hyperbola alone from zero speed (0.57 at 3 rad/s, 0.41 at 5), on his 97.5 N·m |
| lead shoulder, horizontal abduction | 0–1.05: 1; 3.14: 0.93; 4: 0.807; 5.24: 0.661; 6.1: 0.576; 9: 0.356; 13: 0.158; 18.0: 0 | Silva: 93% kept; Kentel (18.0, 6.93) beyond | the same, on his 71.9 |
| elbow extension | 0: 1; 1: 0.801; 2: 0.655; 3: 0.544; 4: 0.455; 5: 0.384; 6.1: 0.319; 9: 0.195; 13: 0.088; 18.45: 0 | Kentel's hyperbola (18.45, 5.55) from zero speed: Holzbaur's bound is isometric. One player; no sample measured elbow extension against speed | — |
| wrist, ulnar deviation, on Peleg's 51 | 0–1.57: 1; 2: 0.920; 3: 0.765; 4: 0.642; 5: 0.541; 6.1: 0.449; 9: 0.274; 13: 0.120; 18.02: 0 | Kentel (18.02, 5.83) relative to its value at 1.57 rad/s, where Peleg measured | — |
| wrist, on Delp's 13.9 | Kentel's hyperbola from zero speed: 1: 0.80; 3: 0.55; 5: 0.39; 6.1: 0.32 | Delp's value is isometric | — |
| an arm muscle being stretched | rises to 1.4 × | Kentel: T_max = 1.4 T₀ in every arm row (FT) | (the legs and trunk keep 1.5 ×) |

**Tendon stretch at the arm joints,** from Kentel's printed stiffness and full strength, as the stretch at full strength: rear shoulder (horizontal adduction) 97.48 over 1,968.5 N·m/rad, 2.8°; lead shoulder (horizontal abduction) 71.90 over 785.9, 5.2°; elbow extension 51.95 over 280.3, 10.6°. The stiffness here is the bound divided by that angle. Kentel set his stiffness from an assumed 5% tendon stretch; nothing is measured at these joints. **The wrists get none:** the printed wrist rows give 49–102° of stretch at full strength, which [N9] doubts as read and no wrist can turn through. A rerun tests the elbows without theirs and the wrists with their printed value (102°).

**The arms may be tensed beforehand.** Each arm channel has a starting level between 0 and 1 that the fit finds, as the legs and trunk have: the hands hold the bat and load it on every pitch, and the one quantitative recording of professional batters has the lead arm's shoulder and elbow muscles strongly active before the swing ([N9] §4.1, second hand).

**The clock, restated as what was measured.** [4] has no event for the hands' first move. Its event at −170 ms is the lead foot passing half of body weight, which begins the phase it calls swing initiation: "the batter's arms finally began to move significantly" and "the bat finally began to rotate forward". What [4] measured there is its bat curve: the bat turns round at −245 ms, is 6.7° past that at −170 ms and moving at 236°/s (its text) to 239°/s (its curve), on its way up. The spec's "[4]'s first move at −170 ms" was this read as the decision. So the measured quantity becomes the rule:

- **the bat's first visible motion:** the model's bat azimuth rate must rise through 236°/s at −170 ms **± 15 ms** [E for the tolerance: [4]'s curve is a mean, and the one sample that reports the spread of the hands' start between batters has 44 ms (Stewart, tee, [N9] §4.1)];
- **the arms' drive may rise above its holding level no more than 70 ms before that,** the top of the measured electromechanical delay's range (10–70 ms, [N9] §4.2; about 70 ms for the one upper-limb figure, second hand; none is measured for a shoulder or in a swing). So no arm channel's onset is earlier than −240 ms. The delay the fit ends with (the bat's first motion minus the earliest arm onset) is reported against 10–70 ms, and a rerun tests 10 ms.
- **Not allowed, and not tried:** the arms' forward drive starting before the batter could have decided. The holding level is a level, not a start: it is constant until the onset.

**What rests on an assumption in the arms, for the owner:** the rear and lead shoulders' bounds rest on one sample of boys scaled by mass; the elbows' on five men; every arm speed curve beyond 3 rad/s, and the elbows' and wrists' at any speed, on one tennis player; all tendon stretch in the arms on an assumed 5%; which wrist reading applies; the ± 15 ms; and the shoulders act about one axis only (the trunk's), with no channel for raising or lowering the arm, bending the elbow or turning the forearm.

**The undriven freedoms' limits (session 7), written before any run with them.** §4.7 lists what the arms and trunk lack. Each gets a drive under the same rule, and the direction decides which measurement applies. This block was written and committed before the engine had the channels and before any refit. Sources are [N9]'s and [N8]'s with their reading statuses; one value was read this session and is marked. 19 channels are added to the 22, each with a strength, an onset, a rise, a switch-off and a starting level: 205 fitted numbers in place of 110.

*The axes.* A shoulder is driven the way its motion is described clinically: the existing channel turns the arm about the trunk's long axis; **raising and lowering** act about the axis square to the trunk's long axis and the upper arm (it moves with the arm; the angle it changes is the arm's angle from hanging at the side, which is what [4] reports as the shoulder's elevation); *(For phase 2: that axis is undefined when the arm hangs exactly along the trunk; `engine.float_axis` then gives no torque and flips its direction across that point, its size staying full. The fitted swing stays 53–83° from hanging, so nothing here is affected; a fielder's or pitcher's arm at rest would be. Review of session 7, P3-5.)* **rotation** acts about the upper arm's own long axis. The elbow's flexors act about the hinge the extensors use; forearm rotation about the forearm's long axis; wrist flexion and extension about the wrist's second axis, which in a hand closed round a handle lies along the handle (the motion of a throttle grip). Both hands hold the bat thumb toward the barrel, which fixes which way is flexion.

| Channel (each side unless said) | Bound, N·m | From | Read | Range a rerun tests |
| :-- | --: | :-- | :-- | :-- |
| shoulder, raising the arm | 112 | Holzbaur 2007: 5 men, isometric abduction 74.4 ± 10.8 N·m at 81.6 kg, 1.778 m, mean + 2 SD scaled | TAB (text layer garbled) | 56 (Kentel's one player, flexion; with the next three as one rerun) |
| shoulder, lowering the arm | 136 | Holzbaur: adduction 93.7 ± 11.3, the same way | TAB | 119 (Kentel, extension) |
| shoulder, internal rotation | 70 at 3.14 rad/s and slower | Dodds 2020: 9 college position players, 37.2 ± 15.4 N·m at 180°/s, 89.6 kg, 1.834 m, mean + 2 SD scaled. Wilkin's pitchers give 64 at 5.24 rad/s (AB, SD from an SEM) | TAB | 43 (Kentel) |
| shoulder, external rotation | 39 at 3.14 rad/s and slower | Dodds: 23.2 ± 7.4, the same way | TAB | 34 (Kentel) |
| elbow flexion | 112 | Holzbaur: 79.5 ± 8.1, mean + 2 SD scaled | TAB | 76 (Kentel) |
| forearm, pronation | 17.4 | **Kentel's one player, as measured: no adult-male sample exists in N·m** ([N9]); his size is not given, so unscaled | FT | none measured; × 2 run as a diagnosis only |
| forearm, supination | 18.3 | the same | FT | the same |
| wrist flexion | 46 | Holzbaur: 25.6 N·m, its SD printed "7" with the rest cut (Danneskiold-Samsøe's is 7.4), mean + 2 SD scaled | TAB | not varied |
| wrist extension | 24 | Holzbaur: 14.0 ± 3.4, mean + 2 SD scaled | TAB | 16 (Peleg 2025, on a handle at 1.57 rad/s: 8.8 ± 1.7, scaled) |
| trunk rotation away from the pitcher (one channel) | 241 at 0.52 rad/s and slower | Bae 2012, the same 51 golfers turning the other way: 131.83 ± 27.87 N·m at 30°/s, mean + 2 SD scaled | **FT, the paper's Results text, read this session** ([N8] has the means only) | 260 (the first direction's bound) |

**Their strength at speed,** in §4.5's form: 1 at and below the speed the bound was measured at, the sample's points where there are any, then Kentel's one player's hyperbola (top speed, ω_c) as a ratio to its own value at the join. A stretched arm muscle rises to 1.4 ×, the trunk to 1.5 ×, as before.

| Group | Points (rad/s: fraction) | From |
| :-- | :-- | :-- |
| raising the arm | Kentel's shoulder flexion from zero speed (20.74, 6.70): 3: 0.59; 5: 0.43; 9: 0.24; 20.74: 0 | one player; Holzbaur's bound is isometric; his frontal-plane rows do not exist, so the forward-plane rows stand in |
| lowering the arm | Kentel's shoulder extension from zero speed (31.93, 15.02): 3: 0.76; 5: 0.63; 9: 0.45; 31.93: 0 | the same |
| internal rotation | 0–3.14: 1; 5.24: 0.833; 7.85: 0.611; then Kentel (19.47, 9.71): 9: 0.516; 13: 0.263; 19.47: 0 | Dodds's position players keep 83.3% from 3.14 to 5.24 rad/s (TAB); Wilkin's pitchers 73.3% from 5.24 to 7.85 (AB). **The one arm curve with samples to 7.85 rad/s** |
| external rotation | 0–3.14: 1; 5.24: 0.858; 7.85: 0.503; then Kentel (18.00, 7.20): 9: 0.414; 13: 0.185; 18.00: 0 | Dodds 85.8%; Wilkin 58.6% |
| elbow flexion | Kentel from zero speed (18.10, 5.43) | one player |
| pronation; supination | Kentel from zero speed (27.88, 9.71); (35.45, 16.90) | one player; junior players keep 86% from 1.57 to 3.67 rad/s where his curves give 78% and 84% (a check, not used) |
| wrist flexion; extension | Kentel from zero speed (21.40, 10.70); (18.08, 7.78) | one player |
| trunk rotation away from the pitcher | 0–0.52: 1; 1.05: 0.893; 2.09: 0.842; 4.19: 0.648; the last line continued to zero at 11.2 | Bae: 131.83, 117.67, 111.04 N·m at 0.52, 1.05, 2.09 rad/s (FT); Gifford's golfers' other row keeps 76.9% from 2.09 to 4.19 (EXC). **Above 4.19 rad/s: [E]** |

**Tendon stretch** at full strength, from Kentel's printed stiffness as §4.5 does for the neighbouring channels: raising 56.29 over 1,386.9 N·m/rad, 2.3°; lowering 119.13 over 2,366.9, 2.9°; internal rotation 42.67 over 603.1, 4.1°; elbow flexion 75.82 over 651.5, 6.7°. **None** for external rotation, the forearm or the wrists (the printed rows give 17–84°, which [N9] doubts as read), and none at the trunk (no value exists). A rerun tests the new channels with no tendon springs.

**Starting levels and the clock.** Each new channel has a starting level between 0 and 1 that the fit finds. The 18 arm channels are under the arms' clock as it stands: no onset before −240 ms, the bat's first motion at −170 ± 15 ms. The trunk's second channel is under the legs' and trunk's rules. Every rise is at least 50 ms.

**What stays undriven, because nothing in the notes measures it:**

- **the trunk's bending, sideways and forward-and-back.** [N5] and [N8] hold rotation only. It keeps the constant starting torques and the joint damping of §4.4.
- **the hands' roll about the handle.** In a real grip the skin's friction under the fingers' squeeze holds the bat from turning in the hand, and the hand can only turn the handle through the wrist. The one measurement is a torque the grip carried, not a limit: Koike 2003's bottom hand put about 20 N·m about the bat's long axis at impact (FIG, off the panel's edge, five hitters, tee; [N2] §2.1). No source bounds a slip, so the bat stays fixed in both hands, and where each wrist sits round the handle stays [E] and the same for both. What the model lacked here was the wrists' flexion and extension, which is how a fixed hand turns a handle; that is in the table.
- **the shoulder girdle** (the shoulder joint moving on the trunk): no body part and no measurement.
- **Still one-way, and not in this job's list:** each shoulder's turn about the trunk's axis the other way (measured: 123 and 92 N·m, Silva) and the wrists' cocking the other way (28, Peleg).

**What each new drive is expected to do, written before the run:**

- *lowering the rear arm:* the rear elbow drops to the side as [4]'s does (its arm 78° from hanging at −250 ms, 37° at contact); the barrel's drop and the rear elbow's bend fit better; little speed.
- *raising and lowering the lead arm:* holds the lead arm up against the bat's pull; the lead elbow's fit; no speed.
- *internal rotation of the rear shoulder, external of the lead:* with the rear elbow bent, turning the upper arm about its own axis swings the forearm and the hands forward. Expected to be the one new drive that adds hand speed: 1–2 m/s at the sweet spot.
- *elbow flexors:* the lead elbow is no longer driven straight (36° at contact against 56°); no speed.
- *forearm rotation:* the top hand rolls over the bottom one through contact. At 17–18 N·m, under 0.5 m/s; it moves the bat's elevation.
- *wrist flexion and extension:* turn the bat about its own handle; nothing at the sweet spot.
- *the trunk's brake:* the trunk's rate falls after −57 ms as [4]'s does; its curve comes inside one band unit and its angle at contact from 99° toward [4]'s 62–75°. A trunk that stops lets the arms run on: up to 1 m/s.
- *The bat's speed overall:* the curves the fit is matched to hold 22.9 m/s at contact (§4.8). A body matched to them is expected to come up from 20.1 toward 22–23 m/s, and not to 29.

**Held out: measured things the fit is not given, checked afterwards.** More drives match any curve more easily, so these say whether the body swings as batters do. Each is also computed for session 6's fit, for comparison.

1. **[4]'s shoulder elevation curves, both arms** (digitised, never in the misfit): the upper arm's angle from hanging at the side, in the trunk's frame. Rear: 78° at −250 ms, 71 at −170, 48 at −93, 37 at contact. Lead: 83, 88, 80, 70. Reported as RMS from −250 ms to contact in units of 10° [E: [4]'s between-batter SD at contact is 4–11°], beside what the placed motion gives, since [4]'s trunk frame is not the model's.
2. **[3]'s arm rate:** 1,160 ± 96°/s at −65 ms. Read here as the turning rate about the vertical of the line from the lead shoulder to the lead wrist [E: [3] does not define it beyond "arms"].
3. **The batting electromyography of [N9] §4.1:** (a) the order of first rises is legs, then trunk, then arms; (b) the lead shoulder's muscle that sweeps the arm out (posterior deltoid: 76–101% of a maximum test from pre-swing through the swing) is on before the swing and stays on; (c) the lead elbow's extensor is highest early (triceps: 92% in early swing, 73% in the middle); (d) the muscle that raises the lead arm stays low (supraspinatus: under 40% throughout). **A new drive the fit runs in a way none of these recordings shows is said so.**
4. **The bat does not slow before contact** (OpenBiomechanics: at 98% of its peak at contact, §4.8). The fit is matched to [4]'s smoothed curve, which does slow, so this is expected to fail; it is reported.

**What rests on an assumption in this block, for the owner:** the forearm's strength is one tennis player's; every speed curve here except the shoulder's rotation is that player's; the raising and lowering bounds are five men's and measured in the frontal plane; the wrist's flexion SD is half-read; tendon stretch as before; Gifford's row for the trunk's fall above 2 rad/s; the two [E] readings in the held-out checks.

### 4.6 The refit on the measured limits (session 5)

*Dated history (PLAN D37, 2026-11-25): this section's fits were matched to the lab study [4], which is no longer a target, a gate or a check. **Its bat speeds include energy that the engine's joint stop created (found by session 8's review, §4.10): with the stop corrected the fit to [4] has 17.8 m/s at its contact, not 21.3, and the swing asked only for speed 15.6, not 30.6.** The rest stands as what was found then; the body's swing is now found by practice (§4.10).*

`fit3d.py` on §4.5's limits, written before it: 22 channels (16 of the legs and trunk with a starting level each, 6 of the arms), 104 numbers, three cold starts, staged as before, the reported numbers from one run of the whole body (the same at a 0.5 ms step). 19,450 swings, 13 minutes. The compiled probe agrees with the Python engine on this fitted swing (`probe3d.py --check`).

| | Start 0 | Start 1 | Start 2 (best) | Start 2 continued 20 iterations | The gate (§13, on [4]'s rate curves, §4.5) |
| :-- | --: | --: | --: | --: | :-- |
| contact (bat at −8°) | +26.4 ms | +27.6 | **+22.4** | +21.0 | 0 ± 5 ms |
| sweet spot at contact (at 0 ms; its fastest) | 18.6 m/s (14.6; 21.8) | 18.2 (14.3; 21.0) | **18.6** (15.9; 20.8) | 18.8 (16.3; 20.6) | 29–34 m/s |
| pelvis rate at −93 ms; its peak | 442°/s; 499 at −70 ms | 556; 566 at −85 | **573; 573 at −90** | 579 | 558 at −93; 581 at −70 |
| trunk peak rate | 961°/s at 0 ms | 1,070 at +50 | 966 at −25 | 980 at −25 | 766 at −57 |
| bat azimuth peak rate | 2,204°/s at +55 ms | 2,076 at +60 | 2,148 at +50 | 2,125 at +50 | 2,435 at −10 |
| lead foot's peak load | 117% BW at −35 ms | 125 at −85 | **120 at −95** | 117 at −95 | 130 at −93 |
| at contact: pelvis; trunk | 65°; 145° | 72°; 156° | 69°; 155° | 67°; 158° | 65 ± 6°; 62° |
| at contact: lag; bat elevation | 104°; −10° | 108°; −10° | 106°; −15° | 107°; −16° | 65 ± 10°; −30 ± 5° |
| at contact: lead knee; lead and rear elbow | 4°; 15°, 81° | 1°; 11°, 86° | 5°; 20°, 89° | 7°; 21°, 92° | 18°; 56°, 81° |
| cost | 84.6 | 88.5 | 66.8 | 64.2 | (session 2: 147–186) |

**Residuals of the best start, RMS in band units** (the gate asks for 1; beyond 2 is "stop"):

- **inside 1:** pelvis rate 0.69, pelvis travel 0.31, the lead foot's three forces 0.85, 0.77, 0.69, the rear foot's 0.67, 0.59, 0.50, lead knee 0.42, rear elbow 0.68;
- **between 1 and 2:** bat rate 1.13, lead elbow 1.28, lag 1.54, bat elevation 1.64;
- **beyond 2:** trunk rate 2.90.

**What changed against session 2:** the lower body. With measured limits the pelvis's rate, both feet's forces, the lead knee and the pelvis at contact are inside the gate in the best start, where session 2 had the pelvis peaking after contact and the lead foot's forces 2.7–3.2 band units off. Contact came from +41–52 ms to +21–28 ms.

**What is still outside the gate:** contact (21–28 ms late), the bat's speed (18.6 m/s against 29–34), the trunk's rate (it runs ahead: 966–1,070°/s, 145–158° at contact), the lag (104–108° against 65 ± 10: the bat is not released), the bat's elevation (−10 to −16° against −30 ± 5) and the lead elbow (driven nearly straight).

**On a bound (best start): 35 of 104 numbers.** Every arm channel is at its strength bound, at the earliest onset (−170 ms) and at the shortest rise (50 ms). In the legs: both hips' rotators and the rear knee's flexors at their strength bounds, the lead knee's extensors at 402 of 404; six rises at the 50 ms floor. The trunk is **not** at its bound (198 of 260 N·m, 162 applied): it turns too fast as it is. The tendon springs hold up to 44 J (lead hip flexion) and 35 J (lead knee extension); the lead knee still reaches its stop (234 N·m).

**The inverse dynamics against the new limits** (`inverse3d.py`, §4.4's table recomputed with §4.5's bounds and curves; needs unchanged): rear hip extension needs 256 N·m where 298 is now allowed at its speed (0.86); lead knee extension 250 of 260 (0.96); rear knee flexion 114 of 156. Still over: the lead hip's adduction at 7.1 rad/s (123 against 62; both its bound and its speed curve are [E]), the lead hip's abduction late (180 against 172) and the hips' rotators (68 against 44 at the rear; [E]). So the two ways agree: with measured limits the legs' main muscles can do what [4]'s motion asks.

**One rule relaxed at a time** (`fit3d.py --relax`: the best start continued 20 iterations with every number free; a diagnosis, not the design):

| Relaxed | Contact | Sweet spot at contact | Note |
| :-- | --: | --: | :-- |
| nothing | +21.0 ms | 18.8 m/s | |
| the arms' onset free in the window | **+8.3 ms** | 20.2 m/s | bat elevation −27°, lag 80° at contact; trunk rate 1.72 band units |
| shoulders' and elbows' bounds doubled | +14.4 ms | 22.1 m/s | the lead elbow fully straight |
| no 50 ms rise floor | +15.5 ms | 19.4 m/s | |
| wrists' bound × 3 | +15.3 ms | 19.8 m/s | |
| legs' and trunk's bounds × 1.5 | +20.8 ms | 18.9 m/s | no longer matters |

**The wrists' cock with a stop** (`--wrists`; a one-sided stop a margin past the cock's angle at the start [E], re-fitted 15 iterations): 5° past, contact +13.6 ms at 19.4 m/s (the stop carries 41 N·m); 20° past, +13.4 ms at 18.0 m/s (111 N·m). Without a stop +21.0 ms. A stop is worth about 7 ms here, more than in session 2's body (3–5 ms).

**What the answer rests on** (`--ranges`: §4.5's range ends one at a time, re-fitted 15 iterations from the best start; the baseline is +21.0 ms, 18.8 m/s):

| Changed | Contact | Sweet spot at contact | Pelvis at −93 ms |
| :-- | --: | --: | --: |
| **nothing tensed beforehand** | **+40.4 ms** | 21.2 m/s | 591°/s |
| hip extension on Anderson's own speed curve | +26.1 ms | 17.9 | 521 |
| hip abduction, adduction and rotators on the first law | +23.6 ms | 18.8 | 568 |
| the legs' bounds at their sources' means (267, 188, 271, 144, 158 N·m) | +23.2 ms | 18.0 | 526 |
| hip rotators 50 N·m | +22.7 ms | 18.8 | 563 |
| no tendon springs | +21.6 ms | 18.8 | 572 (lead foot 107% BW) |
| hip extension 524 N·m; trunk 290; trunk 250; hip adduction 185; adduction 92; rotators 110 | +21.0 to +21.2 ms | 18.8–18.9 | 578–584 |

- **Most sensitive to:** the arms' earliest onset (13 ms), and the legs and trunk being tensed beforehand (19 ms the other way). Then the wrists' stop, the arms' strength and the rise floor (5–8 ms each).
- **Not sensitive to:** any range end of the legs' or trunk's strength (within 2 ms), the tendon springs (0.6 ms) or the three [E] hip items (hip adduction's and the rotators' strength, their speed curve: within 3 ms). So the unmeasured hip numbers the owner was to be told about do not carry the answer.
- **The bat's speed** is 18–22 m/s in every run of this session. No rule and no range end moves it to the gate.

**The small frame of PLAN D32** (`--small`: 1.75 m, 77 kg [E], the same sources' bounds at his weight × height, 0.80 of the average's; his own timings, re-fitted 20 iterations from the average's plan): contact +23.7 ms, **16.3 m/s at contact against the average's 18.6–18.8 (87%)**, 17.9 against 20.6–20.8 at its fastest. 40 of his 104 numbers are on a bound. As in session 2 this is a ratio between two bodies that do not yet reach the ball on time.

**Not converged to a tolerance:** each search stopped at its iteration limit; continuing the best start 20 iterations moved contact from +22.4 to +21.0 ms. The three starts agree on the picture, and differ by 5 ms in contact.

**Next, if the owner extends the rule to the arms** (§4.4, "For the owner"): read Kentel 2011's table in full (shoulder, elbow, wrist full strength, speed curves and series stiffness of one elite tennis player) and look for a shoulder and elbow sample with SDs; write the arms' limits into §4.5 before any run; give the arms' channels their curves, tendon springs and starting levels; refit; then the breakdowns of §6.2. An independent review comes before anyone calls the gate passed.

### 4.7 The refit with the arms on measured limits (session 6)

*Dated history (PLAN D37, 2026-11-25): this section's fits were matched to the lab study [4], which is no longer a target, a gate or a check. **Its bat speeds include energy that the engine's joint stop created (found by session 8's review, §4.10): with the stop corrected the fit to [4] has 17.8 m/s at its contact, not 21.3, and the swing asked only for speed 15.6, not 30.6.** The rest stands as what was found then; the body's swing is now found by practice (§4.10).*

`fit3d.py` on §4.5's limits with its arm block, written before it: 22 channels, each with a strength, an onset, a rise, a switch-off and a starting level, 110 numbers; three cold starts, staged as before, the last stage 45 iterations; one run of the whole body for the reported numbers (the same at a 0.5 ms step). 21,300 swings, 14 minutes. The compiled probe agrees with the Python engine on the fitted swing. The wrists' two readings were each fitted whole.

| | Start 0 | Start 1 | Start 2 (best) | Wrists on Delp's 13.9 N·m (two starts) | The gate |
| :-- | --: | --: | --: | --: | :-- |
| contact (bat at −8°) | +6.1 ms | +3.9 | **+4.0** | +7.0, +5.9 | 0 ± 5 ms |
| the bat's first motion (rate through 236°/s) | −174 ms | −173 | **−172** | −176, −174 | −170 ± 15 ms (§4.5) |
| the first arm drive's rise before that | 66 ms | 67 | **65** | 64, 66 | at most 70 ms (measured 10–70) |
| sweet spot at contact (its fastest) | 19.7 m/s (20.2) | 20.6 (22.2) | **20.1 (21.5)** | 19.6, 20.1 | 29–34 m/s; [4]'s own motion on this body: 22.9 (25.0) |
| pelvis rate at −93 ms | 539°/s | 528 | **562** | 554, 575 | 558 |
| trunk peak rate | 925°/s at +60 ms | 896 at +60 | 836 at +60 | 877, 840 | 766 at −57 |
| bat azimuth peak rate | 2,149°/s at +10 ms | 2,113 at +10 | 2,060 at +10 | 2,223, 2,082 | 2,435 at −10 |
| lead foot's peak load | 116% BW | 125 | **130 at −95 ms** | 115, 120 | 130 at −93 |
| at contact: pelvis; lag; bat elevation | 64°; 72°; −28° | 66°; 73°; −27° | **66°; 72°; −27°** | 66°; 83°, 79°; −29°, −27° | 65 ± 6°; 65 ± 10°; −30 ± 5° |
| at contact: trunk; lead knee; elbows | 100°; 29°; 39°, 86° | 99°; 5°; 38°, 94° | 99°; 12°; 36°, 93° | 108°, 104° | 62°; 18°; 56°, 81° |
| cost | 15.9 | 11.0 | 10.1 | 19.3, 14.6 | (session 5: 66.8) |

**Residuals of the best start, RMS in band units** (the gate asks for 1): pelvis rate 0.54, bat rate 0.89, pelvis travel 0.35, the lead foot's forces 0.40, 0.76, 0.53, the rear foot's 0.59, 0.54, 0.58, lead knee 0.24, lead elbow 0.94, rear elbow 0.69, bat elevation 0.27; **over 1: trunk rate 1.42, lag 1.30.**

**Against the gate of §13:**

- **Inside:** contact (best start and one other; the third is at +6.1 ms); the pelvis's and the bat's rate curves; the pelvis, the lag and the bat's elevation at contact; all six ground-force curves; the bat's first motion and the delay before it; no rule broken; the engine's checks.
- **Outside:** the bat's speed against 29–34 m/s; the trunk's rate curve (1.42 band units). Not evaluated: the arm's rate curve of [3] (a different observable) and the breakdowns of §6.2.
- **The searches settled:** continuing the best start 20 iterations moved contact by 0.1 ms and the cost from 10.15 to 10.08.

**On a bound (best start): 30 of 110 numbers.** Both shoulders, both elbows and the lead wrist are at their strength bounds and at the 50 ms rise; the rear wrist is switched off. In the legs only the lead knee's flexors are at their strength bound (the rear knee's flexors, the rear hip's extensors and rotators within 5% of theirs). The trunk is at 125 of 260 N·m. The earliest arm drive is the lead elbow's, at −237 ms, 3 ms inside its limit. Starting levels: the lead shoulder 0.55 and lead elbow 0.09; the others 0.

**The bat's speed, and what the gate should read.** [4] did not measure bat speed; the gate's 29–34 m/s is [3]'s tee study and Statcast's game swings. The body moved exactly along [4]'s curves (`inverse3d.py`'s placed motion) has the sweet spot at **22.9 m/s at contact and 25.0 at its fastest, 10 ms earlier**, on either reading of the trunk's curves and with the elbows ± 8°; its bat turns at 1,850°/s at contact (2,212 at its peak; these are the bat's whole turning rate, not the azimuth rate [4] reports, §4.8) and its hands move at 7.0 m/s (4.9 on the rate-curve reading). The fitted swing: 20.1 m/s at contact, the bat turning at 1,824°/s and the hands at 3.6 m/s. So the body turns the bat as fast as [4]'s motion does and is slow in the hands. Against [4]'s own motion it is 12% slow at contact and 14% at its fastest; against the gate's band it is 9 m/s short. A gate of 29–34 m/s and [4]'s curves cannot both be met by one body: that is the second inconsistency in the gate, after the angle and rate curves (§4.5). *(Session 7: the reason is not two sets of hitters. [3]'s figure is the bat's end and both studies' curves are smoothed through the impact, §4.8.)*

**The trunk.** It reaches the right speed at the right time (786°/s at −80 ms against [4]'s 696 at −80 and 766 at −57) and then does not slow: 797°/s at contact where [4]'s is at 429. It ends 99° round at contact, against 62° on [4]'s rate curve and 75° on its angle curve. The body has one trunk channel, toward the pitcher, and nothing that brakes the trunk.

**One rule relaxed at a time** (`--relax`, the best start continued 20 iterations):

| Relaxed | Contact | Sweet spot at contact |
| :-- | --: | --: |
| nothing | +3.9 ms | 20.1 m/s |
| no first-motion rule, the arms' drive from the window's start | +2.8 ms | 20.3 (the bat then starts at −180 ms) |
| shoulders' and elbows' bounds doubled | +1.5 ms | **22.3** |
| no 50 ms rise floor | +3.5 ms | 20.2 |
| wrists' bound × 3 | +3.3 ms | 20.3 |
| legs' and trunk's bounds × 1.5 | +3.8 ms | 20.1 |

No rule binds contact any more. Only the shoulders' and elbows' strength moves the bat's speed, to 22.3 m/s with twice the measured bounds.

**What the answer rests on** (`--ranges`, re-fitted 12 iterations from the best start; the baseline is +3.9 ms, 20.1 m/s):

| Changed | Contact | Sweet spot at contact |
| :-- | --: | --: |
| **the legs and trunk not tensed beforehand** | **+29.9 ms** | 19.6 m/s |
| **the arms' rise at most 10 ms before the bat's first motion** (the bottom of the measured delay) | **+10.6 ms** | 19.9 |
| the shoulders on Kentel's one player (97.5 and 71.9 N·m, his curve alone) | +7.4 ms | 18.5 |
| the wrists on Delp's reading (whole fits, above) | +5.9 ms | 20.1 |
| elbows 52 N·m | +5.5 ms | 19.3 |
| the wrists with their printed tendon stretch | +5.5 ms | 20.4 |
| elbows 76 N·m; the arms not tensed beforehand | +4.3 ms | 19.8–19.9 |
| no tendon springs at the arms; none at the elbows; shoulders 136 and 112 N·m | +3.6 to +4.0 ms | 20.1–20.6 |
| a stop on the wrists' cock, 5° or 20° past the start (`--wrists`) | +4.3, +3.7 ms | 20.2 |

- **Most sensitive to:** the legs and trunk being tensed beforehand, and how long before the bat moves the arms' drive may rise. On the delay the fit sits at 65 ms, near the top of the measured 10–70 ms; no delay is measured for a shoulder muscle or in a swing ([N9] §4.2).
- **Not sensitive to:** which wrist reading (2 ms), the arms' tendon stretch (under 2 ms), the arms being tensed (0.4 ms), the elbows' and shoulders' range ends except Kentel's player (under 2 ms), a wrist stop.

**The small frame of PLAN D32** (`--small`: 1.75 m, 77 kg [E], his own timings, re-fitted 20 iterations from the average's plan): contact **+5.6 ms**, the bat's first motion at −175 ms, **18.1 m/s at contact against the average's 20.1 (90%)**, 19.9 against 21.5 at its fastest, the lead foot at 131% of his body weight. He reaches the ball as the average does, 10% slower in the bat.

**What the body still lacks** (each has measured strengths in [N8] or [N9]; none was added in session 6; session 7 gave each its drive, §4.5 and §4.9, except the hands' roll, which nothing measured bounds):

- the shoulders act about the trunk's axis only: nothing raises or lowers the arm ([4]'s rear shoulder drops 22° in the swing);
- no forearm rotation, no elbow flexors, no wrist flexion;
- nothing that brakes the trunk (Bae's golfers were tested turning both ways);
- the hands' roll about the handle is fixed and the same for both hands.

**Next:** the owner's decision in §4.4. Before any review: the breakdowns of §6.2 are not run, and the gate's two inconsistencies (angle against rate curves; 29–34 m/s against [4]'s own 23–25) need one reading each.

### 4.8 Where the placed motion's bat speed goes (session 7, step 0: a diagnosis; redone after its review)

*Dated history (PLAN D37, 2026-11-25): this section's fits were matched to the lab study [4], which is no longer a target, a gate or a check. **Its bat speeds include energy that the engine's joint stop created (found by session 8's review, §4.10): with the stop corrected the fit to [4] has 17.8 m/s at its contact, not 21.3, and the swing asked only for speed 15.6, not 30.6.** The rest stands as what was found then; the body's swing is now found by practice (§4.10).*

`batspeed.py` (§3); no fit, no drive, nothing in the model changed. §4.7 read the 6–11 m/s between the placed motion's 22.9 m/s and the gate's 29–34 as two different sets of hitters. It is mostly two things about how the sources measured. Session 7 put a size on the second that its [review](reviews/2026-11-25-opus-body-3d-s7-review.md) found unsupported (P1-1, P1-2): its "27.6–29.9 m/s" rested on pieces that jumped at the edge of their window, and part of it came from an undone barrel-drop curve that chops down on the ball. This version is redone from `batspeed.py` with the pieces joined to [4]'s curve; the review's own probes (scipy, written separately) agree with it where the two set the same problem (below). **[4]'s batters were not slow; how fast they were is an estimate with a wide range, and the owner has ruled that the smoothed part of the curves is not a target (PLAN D35).**

**What is settled, each with its evidence:**

1. **[3]'s speed is the bat's end, not the sweet spot.** Its markers were on the "bat handle just above the hands, bat barrel end" ([note 3](research/swing/3-welch-1995.md), FT). In the placed motion the sweet spot moves at 0.82–0.83 of the end's speed, so [3]'s 31 m/s (fastest, −15 ms) and 29 (contact) are **25.5 and 23.9 m/s at the sweet spot** (the ratio derived). The gate's "29" was never a sweet-spot speed.
2. **[4] smoothed its marker paths through the impact.** Its own text: "the raw XYZ positions of the markers were passed through a 12 Hz Butterworth filter", on data "cropped from a few frames before the hitter's initial movement until a few frames past follow-through" ([4] §3.4, FT, quoted in [note 4](research/swing/4-miami-study.md)). [3] filtered at 13.3 Hz; that its filter also ran through the impact is an inference from a filter on the whole recorded swing, not stated.
3. **[4]'s own batted balls need more than 22.9 m/s.** Its regression (Eq. 14, middle location, mean pitch 25.0 m/s, mean peak bat rate) gives a mean exit velocity of 36.7 m/s, 82 mph. With the oracle's collision (physics-spec §8), a ball met squarely at the sweet spot leaves that fast only if the sweet spot moves at **23.1–24.7 m/s** (normal restitution 0.55–0.50); every ball met less than squarely needs more. So 22.9 is below a floor, and is not what [4]'s batters swung.
4. **Hitters measured directly.** Statcast: 71.5 mph, 32.0 m/s, 6 inches from the head at contact, the mean of major leaguers' competitive swings in games ([note 9](research/swing/9-step0-data.md)). OpenBiomechanics, 677 swings of 98 mostly college hitters against a 65 mph machine: 66.2 mph (29.6 m/s) at contact, 10th and 90th percentiles 60.0 and 72.0 mph (26.8 and 32.2 m/s), its minor leaguers 69.5 mph (31.1 m/s) at their fastest; where on the bat it measures is not stated ([note 6](research/swing/6-openbiomechanics.md)). **There the bat is at 98% of its fastest at contact** (the source's 66.2 against 67.1 mph; the note-writer's own finite difference of the markers gives 94.5%, 31.1 against 32.9 m/s, with the peak at −2 ms, SD 5; the note says to use the source's). A real bat does not lose 9% before it meets the ball.

**The split.** The sweet spot is 0.634 m from the lead hand's grip. Its speed in the placed motion, as the hands' translation plus the bat's turning about the hands (each part taken along the sweet spot's own direction, so they add up):

| | Sweet spot | = hands | + turn in azimuth | + turn in elevation | Hands' own speed | Bat's azimuth rate (its whole turning rate) |
| :-- | --: | --: | --: | --: | --: | --: |
| at its fastest, −10 ms | 25.0 m/s | 2.6 | 21.4 | 0.9 | 7.4 m/s | 2,339°/s (2,118) |
| at contact | 22.9 | 3.2 | 19.7 | 0.0 | 6.5 | 2,129 (1,850) |
| [4]'s curves alone, no body, at contact | 21.3 from turning | | | | | 2,217 on its rate curve (1,926) |

- **The hands, measured directly first.** Blast's database has major-league hitters' peak hand speed at 23–29 mph, 10.3–13.0 m/s: the fastest the hands go in the swing, as Blast's sensor on the knob of the bat computes it (its own measure; the time of the peak is not given; EXC, blastmotion.com and Prep Baseball Report boards, not a paper). Nothing in the notes measures the hands' speed *at contact* directly: OpenBiomechanics has a "hand speed" whose unit and definition are unclear ([note 6](research/swing/6-openbiomechanics.md); unusable, not even as a cross-check), [3] reports no hand path and [4] none either.
- **The placed motion's hands are not a measurement.** Their 11.0 m/s at −50 ms and 6.5 at contact (7.0 from the mid-point between the hands, §4.7's figure) come from [4]'s smoothed curves (both elbows, the lag, the bat's angles, the trunk) through the placement, along a hand path that is [E] because [4] does not report one. The 11.0 at their fastest sits inside Blast's range. **How much of the slowing to 6.5 could be the filter** (`batspeed.py` §6): the impact takes speed off the hands too, between nothing and the 3.3 m/s that a free bat's handle is kicked back [E]; shared over the bat, both hands and both forearms (5.1 kg) the ball's impulse is 1.7 m/s. The 12 Hz filter shows half of such a loss at contact, a quarter at −10 ms, under a tenth at −20 and nothing at −50. So the filter could account for 0.9–1.7 m/s of the hands' speed at contact and 0.4–0.9 at −10 ms; the slowing from 11.0 to about 8 m/s between −50 and −10 ms is in [4]'s curves before the filter reaches them. The hands are also placed on the bat's smoothed angles near contact, so the smoothing reaches them a second way: with the bat's azimuth undone (below) the placement's hands give the sweet spot −0.9 to −1.8 m/s where they gave +3.2. Whether real hitters' hands slow before contact as [4]'s placed ones do is not measured directly in any source in the notes.
- **The speed is not mostly the hands.** At contact the placed hands move 60° off the sweet spot's direction, so 6.5 m/s of hand gives it 3.2.
- **The speed is the bat's turning**, and the turning is [4]'s own curve: no freedom the placement leaves out can change it. The placement is a posture at every instant, so every joint takes whatever angle closes the arms on the bat (the shoulders' three axes, the forearms, both wrist axes); the only thing [4] does not give is the hands' path, worth 3 m/s here. A shoulder joint moving 5 cm on the trunk in the last 100 ms [E: the shoulder girdle, which the body does not have] would add 0.5 m/s to the hands and about half of that to the sweet spot.
- **2,435 against 2,212°/s are two different quantities.** 2,435 is [4]'s rate of the bat's azimuth (the direction of its shadow on the ground). §4.7's 2,212 (2,204 here) and 1,850 are the placed bat's whole turning rate: with the barrel 21–30° below level, the azimuth rate counts for its cosine. Like for like the placed motion's azimuth rate peaks at 2,339°/s at −10 ms against [4]'s 2,435 (2,419 as digitised): 4% low, because the placement differentiates [4]'s angle curve over ±10 ms across a peak 20 ms wide. At contact that is 1.0 m/s of the sweet spot's speed.
- **What does not matter:** making the velocities consistent with the joints (13°/s); smoothing the hands' path (22.9 either way); the pelvis's and trunk's rate-curve reading (22.9); shoulders at Winter's 0.129 of height from the trunk's axis in place of 0.112 (22.7). Differencing over ±5 ms is noise (29.2 m/s at −10 ms, 18.8 at contact: the digitised points are 5–9 ms apart).

**The filter through the impact.** The ball takes speed off the bat within a millisecond; a 12 Hz low-pass spreads that loss over about 40 ms either side, so the curve starts to fall before contact and reads, at contact, part of the way down. [4]'s rate curve has that shape: 2,419°/s at −10 ms, 2,217 at contact (8.4% below its peak), 1,745 at +15, 1,451 at +35.

- **How large the loss at impact is** (the oracle's collision): the ball that leaves at 36.7 m/s takes 12.2 m/s off the sweet spot of a free bat and 1,519°/s off its turning rate, which is **1,749°/s of [4]'s azimuth rate** with the bat 30° below level (the rate of the shadow is the turning rate over the cosine of the elevation).
- **A held bat loses about as much as a free one** (session 7 said it loses less; the review said not; the review is right). In the contact (0.6–1.0 ms [E]) the ball's impulse is 8.8 N·s and the handle of a free bat is kicked 3.3 m/s; a grip modelled as a spring of 10⁴–10⁶ N/m [E: soft palm to nearly rigid] adds at most 0.6 N·s, so the step is **94–100% of a free bat's**. The forces the hands were already applying act the same with or without the ball. Only a handle held rigidly still would cut the turning rate's step, to 77% (the sweet spot's own loss hardly changes, 2%, because the sweet spot is near the centre of percussion about the hands). The body as built has the hands lumped rigidly into the bat and rigid wrists, which is that stiff case; a real grip is not, which is the free-bat approximation the physics-spec already uses (physics-spec §6.3 [S]). So the step's ceiling is the free bat's.

**How much the filter took: the smoothing undone on stated shapes.** [4]'s true azimuth rate is taken as [4]'s curve outside ±70 ms; inside, a rise of a stated shape [E] from [4]'s value at −70 ms to an unknown rate just before contact, a step at contact, and a piece that rejoins [4]'s curve at +70 ms. Every shape joins [4]'s curve in value at both edges (session 7's pieces did not: they started 264–409°/s below it, or 276–1,361 above, at −70 ms); one also joins it in slope. Filtered and fitted to [4]'s curve over ±100 ms, for each of four filters [E] ([4] does not give the filter's order or whether its cut-off was corrected for two passes; Winter's correction is now the two-pass one, review P3-1). A shape **fits** if its misfit is under 50°/s [E: a fifth of the fit's band on this curve] **and** its step is no more than a free bat's 1,749°/s: a bigger step would take more off the bat than the ball can.

| Rise before contact [E] | Rate just before contact, four filters | Step (of a free bat's) | Misfit | Filtered: lost by contact ([4]: 8.4%) | Fits |
| :-- | --: | --: | --: | --: | :-- |
| straight, from [4]'s value at −70 ms | 2,378–2,436°/s | 44–237 (0.03–0.14) | 83–97°/s | 2.8–3.6% | no filter |
| a rise, then flat over the last 10, 20 or 30 ms | 2,060–2,306 | −506 to +76 (the bat speeding up at impact) | 86–114 | 1.5–3.5% | none |
| accelerating as u² (u the fraction of the window gone) | 2,948–3,022 | 1,058–1,096 (0.6) | 36–73 | 4.1–6.0% | 2nd order, corrected only |
| accelerating as u³, or u^k with k fitted (2.9–4.9), step free | 3,456–5,027 | 1.0–2.2 of a free bat's | 20–60 | 4.8–6.8% | none: the step is more than the ball can take |
| the same with the step at a free bat's (k 2.8–3.0) | **3,402–3,544** | 1,749 (1.00) | 20–65 | 3.8–6.7% | three filters of four (not 4th order uncorrected, 65) |
| accelerating, joined in slope too (k 4.6–4.7), step at a free bat's | 3,502–3,603 | 1,749 (1.00) | 23–69 | 3.8–5.7% | 2nd order corrected (23) and 4th order corrected (50, at the limit) |
| session 7's straight pieces (with their jump) | 2,684–2,841 | 852–1,073 | 63–83 | | — |

- **The shapes that fit put the bat's azimuth rate just before contact at 2,970–3,540°/s**, where the filtered curve reads 2,217: 34–60% above it. The straight rise that session 7 called the conservative case does not reproduce [4]'s shape: filtered, it loses only 3% by contact against [4]'s 8.4%, and it fits worst. Rises that level off before contact fit worse still and need the bat to speed up at the impact.
- **The top of the range is set by a physical ceiling, not by the data.** Every accelerating shape, left free, asks for a step of 1.0–2.2 times what the ball can take off a free bat; held to that, it settles on the ceiling. Something the family of shapes does not contain may be at work too (the mean of swings each timed to one 3.3 ms frame is smoothed a little more than any one swing). So the shapes bound the answer; they do not measure it.
- **The bat at contact is 4–6° further round** than [4]'s smoothed angle curve (−9.7° as digitised) on the shapes that fit: it reaches [4]'s −8° 1–2 ms earlier. The fit's contact criterion is read off the smoothed curve.
- **Against the review's own probes** (`filt.py`, `filt2.py`, scipy's Butterworth and `filtfilt`): where the problem is the same, the figures agree to a few °/s (straight 2,436 / 237 / 83 in both; u² 2,972 / 1,080 / 36; u³ 3,510 / 1,833 / 20; k fitted 2.9, 3,456 / 1,760 / 20; the plateaus 2,102–2,306 in both). The review's capped figures (3,260–3,390) are lower than these (3,402–3,544) because it capped the step at a free bat's 1,519°/s of turning rate, where [4]'s curve is the azimuth rate and the same loss is 1,749°/s of it.

**The placed motion on those shapes** (the bat's azimuth undone by adding each shape's rate correction to [4]'s angle curve, the rate before contact continued past it, so nothing is differenced across the impact):

| Undone | Sweet spot at contact | Attack angle at contact (OpenBiomechanics: +4.1 ± 5.9°, 10th–90th percentiles −3.3 to +10.7°) |
| :-- | --: | --: |
| nothing ([4]'s motion as placed) | 22.9 m/s (51 mph) | +5.6° |
| the azimuth, the straight rise (does not fit) | 24.5–25.3 (55–57 mph) | +0.1 to +0.5° |
| **the azimuth, the shapes that fit** | **26.0–33.3 m/s (58–74 mph); 30.1–31.1 (67–70 mph) on seven of nine** | **−2.4 to −5.8°** |
| the azimuth and the lag | 26.1–32.0 (58–72 mph) | −4.2 to −5.6° |
| the azimuth, the elevation and the lag (session 7's) | 27.3–32.3 (61–72 mph) | **−19.2 to −21.7°** |

- **The speed is in the bat's turning:** the turn in azimuth alone gives 26.9–32.5 m/s at contact. The placement's hands then go slightly backward along the sweet spot's path (−0.9 to −1.8 m/s, where they gave +3.2 as placed), because the arms are placed on [4]'s smoothed elbow and lag curves and must reach a bat that is further round. That mixture of undone and smoothed curves is also why the attack angle drops to −2 to −6°: the placed motion is no longer one consistent swing near contact, and its speed carries that uncertainty.
- **The elevation undone is not a hitter's swing** (review P1-1, confirmed). It has the barrel still dropping at 616–755°/s at contact and the sweet spot coming down on the ball at 19–22°, three to four SD below OpenBiomechanics' hitters; major leaguers average about +8–10° ([note 7](research/swing/7-video-xHktmnRAx1s.md)). It is ruled out as a target.
- **The lag undone, checked the same way.** At contact it reads 57.9–60.5° (as filtered 64.5; [4]'s table 65 ± 10), so inside the measured spread, and undoing it does not make the bat chop (the attack angle stays at −4 to −6°, as with the azimuth alone). But it closes at 1,827–2,030°/s at contact, 1.5–2.7 SD above the fastest uncocking [3] measured (1,588 ± 162°/s, at −20 ms; [3]'s arm line runs from the shoulders' mid-point to the wrists' and its curve is also smoothed), and its pieces jump 4–7° at +70 ms. It is not a motion that is shown to be wrong, and not one that is shown to be right; it is not a target either.

**The account.** Of the 6.1 m/s to the gate's lower end and the 11.1 to its upper:

| Candidate | Accounts for | Evidence |
| :-- | :-- | :-- |
| the gate's lower end, 29 m/s, is [3]'s bat end, not its sweet spot | all of the gap to [3]: at the sweet spot [3] has 23.9 at contact and 25.5 at its fastest, the placed motion 22.9 and 25.0 | measured (the marker's place, FT); the ratio derived |
| the 12 Hz filter run through the impact | **3.1–10.4 m/s at contact over the shapes that fit** (22.9 becomes 26.0–33.3; 7.2–8.2 on seven of nine); 1.6–2.4 for the straight rise, which does not fit | the filter measured ([4] §3.4, FT); the size derived; **the shape [E]** |
| the mean curve blunter than single swings | 0.5 m/s: the mean of the swings' own peaks is 2,486°/s, the mean curve's peak 2,435, 2.1% ([4] Table 3 against its §4.1.5) | measured (TAB) |
| freedoms the placement does not use | none through the bat's turning; at most about 0.3 m/s through the hands | derived; the 5 cm [E] |
| the body's dimensions | −0.2 m/s (shoulder width); height and weight are [4]'s own means | derived |
| the instant | none in the speed: the model, Statcast and OpenBiomechanics all read at contact. The bat's angle at contact is smoothed too (4–6°, 1–2 ms) | derived |
| **left over** | **to 29 m/s: nothing (the range straddles it, −4.3 to +3.0). To Statcast's mean of 32.0: −1.3 to +6.0. To 34: +0.7 to +8.0** | different hitters and pitches: AA players against 56 mph batting practice, major leaguers' fastest 90% of swings in games |

**What would settle the size:** [4]'s marker paths unfiltered, or filtered separately before and after contact; or the bat's turning rate of professional hitters from any set processed that way. OpenBiomechanics' bat at 98% of its fastest at contact already settles the shape near contact for its hitters: the bat does not slow before it meets the ball.

**What this changes (with Q59 and PLAN D35).**

- *What was found:* the "51 mph" swing was a measuring effect. That much is settled by [4]'s own batted balls (at least 52–55 mph needed at the sweet spot if every ball were met squarely) and by its own description of its filter. How fast its hitters really swung cannot be read back out of the study: depending on the bat's last 70 ms before contact, 55–74 mph, with the shapes that reproduce the study's curve at 58–74 and most of them at 67–70.
- *Decided (owner, PLAN D35):* the study's curves inside the last 40 ms before contact are not a target for the body. There it is checked against hitters measured directly: bat speed at the sweet spot at contact, the attack angle, the bat at its fastest at contact. "Matched to unsmoothed curves" (session 7's recommendation) is withdrawn: the unsmoothed elevation chops down on the ball, and the size of the unsmoothing rests on an assumed shape.
- *The gate's two angles at contact read from the same curves:* the cock (lag, 65 ± 10°) and the barrel's drop (elevation, −30 ± 5°) are smoothed through the impact too. Undone they read 58–61° and −34 to −36°; the elevation undone is not a hitter's swing, and the lag undone is unproven. Their bands are wider than either correction, so as checks they stand; as exact targets at contact they are worth no more than the curves they come from.
- *Answered since (Q59 = a, PLAN D38, 2026-11-25):* the bat's speed is read against 29–34 m/s (65–76 mph).

### 4.9 The refit with every designed freedom driven (session 7)

*Dated history (PLAN D37, 2026-11-25): this section's fits were matched to the lab study [4], which is no longer a target, a gate or a check. **Its bat speeds include energy that the engine's joint stop created (found by session 8's review, §4.10): with the stop corrected the fit to [4] has 17.8 m/s at its contact, not 21.3, and the swing asked only for speed 15.6, not 30.6.** The rest stands as what was found then; the body's swing is now found by practice (§4.10).*

`fit3d.py` on §4.5's limits with its last block, written and committed before the engine had the channels: 41 channels, 205 numbers; three cold starts staged as before (25, 25 and 40 iterations) and one more from session 6's fitted plan with the new channels switched off; one run of the whole body for the reported numbers (the same at a 0.5 ms step to 0.1 ms and 0.1 m/s). 46,400 swings, 38 minutes. The compiled probe agrees with the Python engine with all 41 channels on and on the fitted swing itself (9 × 10⁻¹² at worst), and with the new channels off the new code reproduces session 6's fit exactly (+4.0 ms, 20.1 m/s). This is a laboratory fit of the model's drive numbers to a recorded average swing, as before.

| | Start 0 | Start 1 (best) | Start 2 | From session 6's fit | The gate |
| :-- | --: | --: | --: | --: | :-- |
| contact (bat at −8°) | +4.6 ms | **+0.8** | +3.6 | +1.9 | 0 ± 5 ms |
| the bat's first motion | −170 ms | **−170** | −171 | −170 | −170 ± 15 ms |
| the first arm drive's rise before that | 59 ms | **70** | 62 | 65 | at most 70 ms |
| sweet spot at contact (its fastest, when) | 21.0 m/s (21.5, −35 ms) | **21.3 (21.3, at contact)** | 18.7 (18.9, −5 ms) | 21.9 (21.9, +5 ms) | 29–34 m/s; [4]'s own motion on this body 22.9 (25.0); the same with the impact's smoothing undone 26–33, an estimate (§4.8) |
| pelvis rate at −93 ms | 469°/s | **549** | 517 | 555 | 558 |
| trunk peak rate | 1,024°/s at −45 ms | **760 at −65** | 732 at −55 | 769 at −65 | 766 at −57 |
| bat azimuth peak rate | 2,373°/s at +25 ms | **2,151 at contact** | 1,944 at −5 | 2,136 at +5 | 2,435 at −10 |
| lead foot's peak load | 144% BW at +60 ms | **127 at −85** | 142 at −55 | 132 at −90 | 130 at −93 |
| at contact: pelvis; lag; bat elevation | 62°; 57°; −26° | **65°; 65°; −30°** | 65°; 66°; −28° | 65°; 66°; −30° | 65 ± 6°; 65 ± 10°; −30 ± 5° |
| at contact: trunk; lead knee; elbows | 83°; 3°; 64°, 43° | **75°; 11°; 72°, 79°** | 75°; 5°; 80°, 91° | 79°; 13°; 58°, 91° | 62° (75° on the angle curve); 18°; 56°, 81° |
| cost | 12.4 | **3.40** | 15.5 | 4.09 | (session 6: 10.1) |

**Residuals of the best start, RMS in band units** (the gate asks for 1): pelvis rate 0.49, **trunk rate 0.57**, bat rate 0.62, pelvis travel 0.13, the lead foot's forces 0.49, 0.72, 0.43, the rear foot's 0.39, 0.37, 0.55, lead knee 0.35, lead elbow 0.44, rear elbow 0.23, bat elevation 0.28, lag 0.62. **Nothing is over 1.**

**Against the gate of §13:**

- **Inside:** contact; every gated rate curve, the trunk's now with them; the pelvis, the lag and the bat's elevation at contact, each on [4]'s value; all six ground-force curves; the bat's first motion and the delay before it; no rule broken; the engine's checks.
- **Outside: the bat's speed, on every reading.** 21.3 m/s at contact against the gate's 29–34; against [4]'s own motion on this body, 22.9 at contact and 25.0 at its fastest (93% and 85%); against that motion with the impact's smoothing undone, 26–33 (an estimate, §4.8); against directly measured hitters (OpenBiomechanics' 10th percentile at contact 26.8).
- **Not evaluated:** the breakdowns of §6.2.
- **How settled the searches were.** The best start was still falling by 0.004 an iteration (cost 3.40) and the start from session 6's fit by 0.005 (4.09); start 0 by 0.03 (12.4); start 2 had stopped (15.5, the lead foot's forces 1.4–2.2 band units off). So two of the three cold starts did not reach the best start's neighbourhood in 40 iterations of the last stage: with 205 numbers the search finds different swings from different starts. The best and the warm start, which began in different places, agree on contact (0.8 and 1.9 ms), the bat's speed (21.3 and 21.9 m/s), the trunk's peak (760 and 769°/s at −65 ms) and the three angles at contact.

**The count of fitted numbers** went from 110 to 205 against the same 15 curves and 5 numbers at contact. That every curve is now inside its band is therefore weak evidence by itself: the curves are means low-pass filtered at 12 Hz over 250 ms, about six independent values each, so roughly 90–100 independent target values against 205 fitted numbers (the review's estimate), and the best of three cold starts was the only one to get there. The held-out checks below are what tests the body.

**On a bound (best start): 56 of 205 numbers.** At full strength: the lead hip's extensors, the rear hip's abductors and rotators, the rear knee's extensors, the lead ankle, the trunk's forward rotators (260 N·m), the lead shoulder's sweep (92), the rear elbow's extensors, both shoulders' internal rotation (70), the lead elbow's flexors (112) and the rear wrist's flexion (46). Both wrists' uncocking channels are switched off, as is the rear forearm's pronation. Nineteen rises are at the 50 ms floor. The earliest arm drive (the rear elbow's flexors, 10 of 112 N·m) is at −240 ms, on the limit.

**The trunk.** It now peaks at 760°/s at −65 ms and is at 517°/s at contact ([4]: 766 at −57, 429 at contact), 75° round at contact ([4]: 62° on its rate curve, 75° on its angle curve). **How it slows is not how the job expected.** The fit runs both trunk channels together from before the window opens (both tensed at 0.3): the forward rotators at their full 260 N·m (250–285 applied) and the rotators the other way rising from 52 to 164 N·m by contact. The two muscles' net is still 100–145 N·m *forward* through the whole swing, 122 at contact. So the brake does not reverse the trunk's drive; it halves it, and the trunk slows because the arms and the bat take its motion: the rear shoulder's end-of-range stop carries up to 228 N·m at −60 ms and the new shoulder channels push the arms off the trunk. Whether the brake is needed at all is the "without" run below.

**The bat's speed, split as §4.8 splits it.** At contact 21.3 m/s = the hands 0.5 + the bat's turn in azimuth 19.7 + in elevation 1.1; the bat's azimuth rate is 2,151°/s ([4]'s curve: 2,217; the placed motion: 2,129), the bat's end at 26.2 m/s ([3]'s end marker: 29). **The body turns the bat as [4]'s curve does and is slow in the hands.** Against the direct measurement first: **its hands' fastest is 7.6 m/s, against major leaguers' peak hand speed of 10.3–13.0 m/s** (Blast's knob sensor, EXC, §4.8), 2.7–5.4 m/s short whatever [4]'s smoothing did. Against [4]'s motion placed on this body (smoothed curves, an [E] hand path; the filter could account for up to 1–2 m/s at contact, none at the peak, §4.8): 3.3 m/s at contact against 6.5, and 7.6 at their fastest against 11.0. Session 6 had the same (3.6 m/s). The new drives did not give the hands their speed; step 1 expected 1–2 m/s from the shoulders' rotation and got about 1 in total (20.1 to 21.3).

**The held-out checks of §4.5** (never in a misfit; session 6's fit and the placed motion beside them):

| Check | Measured | This fit | Session 6's fit | [4]'s motion placed on this body |
| :-- | :-- | :-- | :-- | :-- |
| 1. rear shoulder's elevation at −250, −170, −95 ms, contact | 78, 71, 49, 37° | 83, 79, 60, 56°; RMS 1.19 units of 10° | 83, 83, 92, 121°; 4.48 | 1.72 |
| 1. lead shoulder's elevation | 83, 88, 81, 70° | 74, 77, 62, 64°; 1.29 | 74, 83, 99, 143°; 3.07 | 1.64 |
| 2. the lead arm's turning rate | 1,160 ± 96°/s at −65 ms ([3]) | 1,064 at −70 ms | 847 at −75 | — |
| 3a. first rise above its holding level: legs, trunk, arms | in that order | legs −248 ms, **arms −240, trunk −187** | — | — |
| 3b. the lead shoulder's sweep (posterior deltoid) on before and through the swing | 76–101% of a maximum test | 0.62 of its bound before the swing, 0.95–1.00 from −50 ms | 0.55, then 1.00 | — |
| 3c. the lead elbow's extensor highest early | 92% early, 73% in the middle | **0.27–0.29 throughout** | 0.98–1.00 throughout | — |
| 3d. the muscle that raises the lead arm stays low | supraspinatus under 40% | **0.95** (107 of 112 N·m from −125 ms) | no such drive | — |
| 4. the bat at contact against its fastest before | 98% (OpenBiomechanics' 66.2 against 67.1 mph; the note's own finite difference: the peak at −2 ms, SD 5) | 100%, the peak at contact | 100% | 92%, the peak at −10 ms |
| 5. the attack angle at contact (added after the review, P2-1; a check, never fitted) | +4.1 ± 5.9°, 10th–90th percentiles −3.3 to +10.7° (OpenBiomechanics, 677 swings of mostly college hitters against a 65 mph machine, TAB of a public set) | **−7.2°** (−16 at −10 ms, −24 at −20) | +0.9° (−9 at −10 ms) | +5.6° |

- **Passed:** the arms now move as batters' do where session 6's did not. Session 6's body met its curves with both elbows flying up past shoulder height (the rear arm 121° from hanging at contact where [4]'s is 37°); with the arm's raising and lowering driven the rear elbow drops 27° through the swing ([4]: 41°) and both shoulders' elevations are nearer [4]'s than [4]'s own motion placed on this body is (the placement's elbows are [E], which is why it is 1.6–1.7 units off). The lead arm's turning rate came from 847 to 1,064°/s, at the edge of [3]'s measured spread (1,160 ± 96: one SD below, on an arm line that is [E]). The lead shoulder's sweep is on before the swing and through it. The bat is at its fastest at contact (expected to fail; it does not).
- **Failed, and said so:**
  - **the attack angle** (check 5, added after the review): the sweet spot comes down on the ball at 7.2° at contact, 1.9 SD below OpenBiomechanics' hitters and under their 10th percentile; the barrel is still dropping at 392°/s at contact. The elevation curve's band (8°) cannot see the barrel's path. Session 6's fit had +0.9°, inside the spread. For a game this is the number that sets the launch angle;
  - **the order of first rises.** Six arm drives rise 44–70 ms before the bat moves and before the trunk's own rise at −187 ms (the rear elbow's flexors at −240, the rear arm's lowering at −227, the rear forearm's supination at −225, the lead elbow's extensors at −216, both wrists' flexion at −214 and −211). The trunk and legs are tensed beforehand, so they are not silent; but the recorded order is legs, trunk, arms, and the fit's arms rise early. The rule allows it (70 ms before the bat's first motion); the electromyography does not show it.
  - **the lead elbow's extensor** works at under a third of its strength throughout, where the recorded triceps is at 92% early in the swing.
  - **raising the lead arm** runs at 0.95 of its bound from −125 ms, against the lead arm's lowering at 0.67 (97 against 99 N·m: the two cancel). The one muscle recorded that raises the arm (supraspinatus) is under 40% throughout; the deltoid's middle part, which also raises it, was not recorded. **This is a drive used in a way no batter's muscle is recorded working.** The same holds for the trunk's two channels run against each other (above): the recordings have the obliques and the back's muscles all active from pre-swing, so co-contraction is recorded, but not this.
- **Not recorded in any batting electromyography, so neither confirmed nor contradicted:** the shoulders' rotation (both internal rotators at 56–61 N·m from −81 and −62 ms), forearm rotation (the rear forearm's supination at 17 N·m from before the swing), wrist flexion (the rear wrist's at 45 N·m from the start: it rolls the handle), the elbows' flexors (the lead's at 90 N·m from −116 ms; the lead elbow is 72° bent at contact against [4]'s 56°).

**Which drive does what** (`fit3d.py --without`: the best start re-fitted 12 iterations with one of session 7's groups switched off; "nothing" is the same 12 iterations with everything on, which also says how far the best start was still moving: 0.1 ms and 0.04 of cost):

| Switched off | Cost | Contact | Sweet spot at contact | Trunk rate curve | Lag curve |
| :-- | --: | --: | --: | --: | --: |
| nothing | 3.36 | +0.7 ms | 21.3 m/s | 0.57 | 0.61 |
| **all 19 (session 6's body, re-fitted)** | **9.95** | +3.9 | 21.5 | **1.36** | **1.47** |
| the trunk's rotators away from the pitcher | **2.98** | +2.3 | 22.2 | 0.60 | 0.41 |
| the shoulders' rotation, both ways | 3.44 | +1.7 | 21.1 | 0.60 | 0.52 |
| raising and lowering the arms | 5.09 | +3.4 | 20.6 | 0.51 | 0.88 |
| the elbows' flexors | 3.94 | +2.5 | 20.9 | 0.54 | 0.78 |
| forearm rotation | 3.52 | +2.8 | 21.3 | 0.51 | 0.46 |
| wrist flexion and extension | 3.54 | +3.1 | 21.4 | 0.60 | 0.32 |

- **No one drive is needed; together they are.** Any one group can be taken out and the swing still meets every curve. With all of them out the trunk's curve and the lag go back outside (session 6's result, reproduced).
- **The trunk's brake is not what slows the trunk.** Without that channel the trunk's curve is still at 0.60 and the bat a little faster. What slows the trunk is the arms being driven off it; no single arm drive carries that either.
- **Opposing muscles pull against each other all over the fit, and the search is not at its best** (review P2-2, which session 7 understated as the trunk alone). On together in the saved plan: the trunk's two channels; the lead arm's raiser and lowerer; the lead shoulder's internal rotation (70 N·m from −62 ms) against its external (33 from −204 ms); the rear shoulder's internal (70) against external (27); the lead elbow's flexors (112) against its extensors (24, held at 0.32 from the start); the lead wrist's flexion (29) against extension (6). Each channel is one ramp, so two opposed ramps give the search net torques one ramp cannot, and with no cost on effort it uses them. With the trunk's second channel switched off the refit reaches cost 2.98 against 3.36 with it on, a fit the 41-channel search could have found with that channel at zero: it stopped short. **So the channel stays (Bae measured it, PLAN D34); what the next fit has to deal with is its criterion:** a small cost on effort [E, weight stated before the run and swept], or opposing pairs held to recorded co-activation where there is a recording, and more than three cold starts, some far from the hand estimate. The owner has allowed the cost on effort (Q60 = a, PLAN D36). That is the next session's, after the owner has ruled on Q59.
- **Step 1's expectations against this:** right about the rear arm's lowering (the elbow drops), the elbows' flexors (the lead elbow's curve from 0.94 to 0.44; it now ends 72° bent, past [4]'s 56°), forearm rotation and the wrists (nothing at the sweet spot). **Wrong about the shoulders' rotation** (expected 1–2 m/s; it is worth 0.2) **and about how the trunk would slow** (expected the brake; it is the arms). The bat's speed was expected at 22–23 m/s and is 21.3–22.2.

**One rule relaxed at a time** (`--relax`, 12 iterations from the best start):

| Relaxed | Contact | Sweet spot at contact |
| :-- | --: | --: |
| nothing | +0.7 ms | 21.3 m/s |
| no first-motion rule, the arms' drive from the window's start | +0.7 | 21.3 |
| every shoulder and elbow bound doubled | +0.8 | **21.7** |
| no 50 ms rise floor | +0.8 | 21.3 |
| legs' and trunk's bounds × 1.5 | +0.7 | 21.3 |
| wrists' bounds × 3 | +0.7 | 21.3 |

No rule binds anything. **Twice the measured shoulder and elbow strength adds 0.4 m/s.** The bat's speed is not held down by a limit.

**What the answer rests on** (`--ranges`, 12 iterations from the best start; the baseline is +0.7 ms, 21.3 m/s):

| Changed | Contact | Sweet spot at contact | Note |
| :-- | --: | --: | :-- |
| **the legs and trunk not tensed beforehand** | **+4.9 ms** | 22.4 | the lead foot at 243% of body weight; three of its force curves outside their bands (session 6: +29.9 ms) |
| **the arms' rise at most 10 ms before the bat's first motion** | +1.7 | 21.4 | cost 3.37 against 3.36: **it no longer costs anything** (session 6: +10.6 ms). The arms then first rise at −180 ms, after the trunk's −185: the recorded order |
| session 7's shoulder channels on Kentel's one player (56, 119, 43, 34 N·m, his curves) | +1.0 | 21.0 | |
| the elbows' flexors at 76 N·m | +0.9 | 21.0 | |
| the forearm's strength doubled (a diagnosis, not a range end) | +0.7 | 21.3 | |
| wrist extension 16 N·m; the trunk's brake 260 | +0.7 | 21.3 | |
| session 7's channels without tendon springs | +0.7 | 21.1 | |

- **The two things §4.7's answer leaned on, looked at again.** *The legs and trunk tensed beforehand:* the bat still arrives inside the gate without it, but the front foot then takes nearly twice its measured load; it is still needed, for the ground forces. *The arms rising 65 ms before the bat moves:* no longer needed. Held to 10 ms the swing is the same, and the arms then rise after the trunk as the electromyography has it. **So the held-out failure on the order of first rises is the search using room it does not need, not something the body requires.**
- **Not rerun:** the wrists' other reading (Delp's): both uncocking channels are switched off in this fit, so their bound cannot matter. A stop on the wrists' cock (under 1 ms in session 6).

**The small frame of PLAN D32** (`--small`: 1.75 m, 77 kg [E], his own timings, 20 iterations from the average's plan; the forearm's one-player strength is not scaled): contact **+1.1 ms**, the bat's first motion at −171 ms, every curve inside its band (trunk 0.66, bat rate 0.80), the lead foot at 125% of his body weight, **19.7 m/s at contact against the average's 21.3 (92%)**.

**The strongest batter the measured limits allow** (`--strongest`: from the best start, every number free inside the rules, no curve asked for, only the sweet spot's speed at contact with the ball met on time and the bat first moving at −170 ms):

| | Asked for speed, timing loosely held (30 iterations) | Continued with the timing held hard (40 iterations) | The fitted average batter |
| :-- | --: | --: | --: |
| sweet spot at contact | 29.7 m/s | **30.6 m/s** | 21.3 |
| contact | 17 ms early | −0.3 ms | +0.8 |
| the bat's first motion; the arms' rise before it | −164 ms; 76 ms (6 over the rule) | −168 ms; 71 ms (1 over) | −170; 70 |
| pelvis peak rate | 909°/s | 813°/s at −45 ms | 551 ([4]: 581; [3]: 714 ± 76) |
| trunk peak rate | 990°/s after contact | 919 at −55 ms | 760 |
| lead foot's peak load; lead knee at contact | 131% BW; 14° | **70% BW; 61°** | 127%; 11° ([4]: 130%; 18°) |
| the lead shoulder's end-of-range stop | 433 N·m | **574 N·m** | 119 |

- **This body reaches 68 mph only with a swing no hitter makes** (review P2-4; session 7 called 29–34 m/s "the edge of what it can do"). With every muscle group at its sample's mean + 2 SD at once and every timing chosen for bat speed alone, it reaches 30.6 m/s (68 mph) on time, breaking the arms' clock by 1 ms. The search was still gaining 0.3 m/s in its last 10 iterations, so the ceiling is somewhat above that; it is not near 34.
- **It gets there with a swing no hitter makes:** the front leg does not block (the knee 61° bent at contact, the foot never over 70% of body weight), the hips turn at 813°/s, and the lead arm is pressed 16° past the end of its range across the chest, where the model's stop is a soft spring [E: 2,000 N·m/rad] carrying 574 N·m, about four times the strongest arm muscle's bound. A stiffer stop would take some of that speed away; how much was not run.
- **So the average batter's 21.3 m/s is not a strength limit alone:** the same limits allow 30.6, but only with a swing no hitter makes; held to a hitter's swing, the search below finds 22.5. Part of the 21.3 is what the fitted curves hold near contact (§4.8).

**The strongest batter held to a hitter's swing** (session 7 fix, review P2-4; this paragraph was written and committed before the run). `fit3d.py --strongest --held-swing`: the same search as the run above (every bound available, every timing free, asked only for the sweet spot's speed at contact, the timing held hard: contact at 0, the bat's first motion at −170 ms, no arm drive rising more than 70 ms before it), started from the average batter's fitted plan, 80 iterations, all 205 numbers free. Added, each as a one-sided penalty that is zero inside its limit and weighs 30 per unit past it [E] (`HELD_W`):

- the gate's ground-force curves (both feet, three directions) and its lower-body curves (the pelvis's rate and travel, the lead knee) each within 1 band unit RMS of [4]'s, from −250 ms to contact;
- no joint pressed past its range more than in the average batter's fitted swing: each end-of-range stop's largest torque up to contact at most the average fit's plus 5 N·m (the lead shoulder's 99, the rear's 228; the knees' and elbows' 0), per 10 N·m past it;
- the attack angle at contact inside OpenBiomechanics' 10th–90th percentiles, −3.3° to +10.7° ([note 6](research/swing/6-openbiomechanics.md)), per 2° past it. The average fit itself is outside this (−7.8°), so the run starts with that penalty on.

The trunk's and arms' curves, the bat's own curves and the held-out checks are not asked for. What is reported: the bat's speed at contact, every penalty's state at the end (a limit the search did not keep is said as broken, not hidden), what the speed sits on (strengths on their bounds, the pelvis's and trunk's peaks, the lead foot's load and knee), and how fast the speed was still rising over the last 10 iterations.

**What it found** (job 354, 336 s; `sim/target/codex-probes/body3d/fit3d_all_strongest_held.out`):

| | Held to a hitter's swing | Asked for speed alone, timing held (above) | The fitted average batter |
| :-- | --: | --: | --: |
| sweet spot at contact | **22.5 m/s (50 mph)** | 30.6 (68 mph) | 21.3 (48 mph) |
| contact; the bat's first motion; the arms' rise before it | −1.8 ms; −170 ms; 67 ms | −0.3; −168; 71 | +0.8; −170; 70 |
| attack angle at contact | **−3.4°** (the limit −3.3) | — | −7.2° |
| lead foot's peak load; lead knee at contact | 132% BW at −60 ms (its vertical curve 1.03 band units, the limit 1); 2° | 70%; 61° | 127%; 11° |
| pelvis's peak; trunk's peak | 615°/s at −90 ms; 764 at −65 | 813; 919 | 551; 760 |
| the lead shoulder's end-of-range stop up to contact | 105 N·m (the limit 104) | 574 | 99 |
| bat azimuth peak | 2,359°/s at contact | | 2,151 at contact |

- **The search stopped at 22.5 m/s and was not moving:** 21.3 to 22.5 in the first 25 iterations, then 22.54 m/s over the last 10 with its steps being refused, and it ended itself at iteration 35 of 80 (the cost no longer falling by 10⁻⁵ of itself). It ends with three limits just touched (the attack angle, the lead foot's vertical force, the lead shoulder's stop), each by less than a tenth of its unit; none is broken by more.
- **What it sits on:** the trunk's forward rotators (259 of 260 N·m), the lead knee's extensors (404 of 404), the rear hip's rotators and abductors, the lead shoulder's sweep (92), the rear elbow's extensors (85), the lead elbow's flexors (112), the rear shoulder's internal rotation (70) and the rear wrist's flexion (46) at full strength; 41 of 205 numbers on a bound. The opposing pairs of the average fit are still on (the trunk's brake at 172 N·m at contact, the lead arm's raiser at 0.98 of its bound against its lowerer).
- **What this says, and what it does not.** From the average batter's swing, a local search held to a hitter's ground forces, legs, joint ranges, clock and attack angle finds only 1.2 m/s more. It is one start, its penalties' weight is [E], and a search that ends where its steps are refused may be in a corner of the penalties rather than at the body's ceiling: it does not show that 27–34 m/s is out of this body's reach. Nor does anything show it is within reach. The run asked for speed alone reached 30.6 m/s only through a soft front leg, a pressed lead shoulder and fast hips, each of which this run forbids. **So whether this body can swing a bat at 60–76 mph the way a hitter does is open, and the evidence leans against it as the body and its search stand:** its hands are slow against direct measurement (7.6 m/s at their fastest against major leaguers' 10.3–13.0, above), and holding the swing to a hitter's leaves the bat near 50 mph. The next test is the same held search started from the fast swing (pulled into the limits from above) and from more starts, stated before it runs.

**What is left:**

- the band the gate's bat speed reads: answered since, Q59 = a, 29–34 m/s (PLAN D38). The owner has ruled that [4]'s curves inside the last 40 ms are not a target (PLAN D35): the next fit ends its curves at −40 ms and is checked there on the sweet spot's speed at contact, the attack angle and the bat at its fastest at contact;
- the hands: 7.6 m/s at their fastest against major leaguers' measured 10.3–13.0 (Blast, EXC); 3.3 m/s at contact against 6.5 in [4]'s motion as placed (smoothed curves, an [E] hand path). Not a drive that is missing (every designed freedom is now driven) and not a strength bound. The hands' path is not in the misfit, because [4] does not report it; a target for it would have to come from the placed motion, whose hand path is [E];
- the attack angle: −7.2° at contact, below the measured spread (check 5);
- the fit's criterion: opposing pairs on together, no cost on effort, two of three cold starts unsettled (above); the trunk's second channel stays;
- the breakdowns of §6.2 and an independent review, as before.

### 4.10 Practice in place of the lab study (session 8, PLAN D37)

The owner ruled on 2026-11-25 (PLAN D37): nothing the body is fitted to and nothing it must pass comes from the lab study [4]; no other recorded swing takes its place; the swing is found by practice on the batter's own body (D32), effort counted (D36), inside measured strength (D34) and the joints' real ranges, and is then checked against hitters measured directly. He added: the swing may roughly look like the study's, since that looks real, but none of its measurements is used. This section is written in the order the work was done; each dated block was committed before the next began.

**Step 0: every place [4] enters, and what happens to it** (written before anything else; no run). "Kind" is one of: target (a search is asked to match it), gate (the spec asks the body to pass it), check (compared afterwards), penalty, timing rule, starting state, body dimension, strength or limit, definition, history.

| Where | What comes from [4] | Kind | Now |
| :-- | :-- | :-- | :-- |
| `fit3d.py` `BAND`, `Ctx.target`, `residuals()` | 15 curves from −250 ms to contact: the pelvis's, trunk's and bat's rates, the pelvis's travel, six ground-force curves, the lead knee, both elbows, the bat's elevation, the lag | target | **Removed.** The practice search (`practice3d.py`, step 2) reads none of them. `fit3d.py` stays as the dated record of §4.6–§4.9 |
| `fit3d.py` `AT_CONTACT` | pelvis 64.5 ± 6°, lag 65 ± 10°, bat elevation −30 ± 5° at contact | target, gate | **Removed** |
| `fit3d.py` `CONTACT_AZ`, `measures()` | contact defined as the bat's azimuth reaching [4]'s −8° | definition, target | **Removed.** Contact is the sweet spot reaching the ball (step 1) |
| `fit3d.py` `tc / CONTACT_BAND`; §13's "within ±5 ms of plan" | the ball met at [4]'s contact instant, 250 ms after the start | target, gate, timing rule | **Removed.** How long the swing takes from its start is a result |
| `drive.py` `FIRST_MOTION_T`, `FIRST_MOTION_RATE`, `FIRST_MOTION_TOL`; `fit3d.py` `first_motion_time()` | the bat's rate through 236°/s at −170 ± 15 ms | timing rule, target | **Removed as a rule.** When the bat first moves is reported and checked against direct measurements of swing time |
| `drive.py` `ARMS_EARLIEST` (−240 ms), `fit3d.py`'s box on the arms' onsets | no arm drive more than 70 ms before [4]'s −170 ms | timing rule | **Removed.** The 70 ms is the measured electromechanical delay ([N9] §4.2), which the engine's activation and tendon springs already carry. The clock that stands is "no arm drive before the batter could have decided", and its source is the pitch, not [4]: step 1 |
| `fit3d.py` `HELD_CURVES`, `STOP_REF` (`--held-swing`) | [4]'s ground-force and leg curves within a band; no stop past what the fit to [4] made it carry | penalty | **Removed.** What defines a hitter's swing is the body: strengths, rise floor, joint ranges (step 1) |
| `fit3d.py` `held_out()` check 1 | [4]'s shoulder elevation curves | check | **Dropped** (only [4] has them) |
| `fit3d.py` `ATTACK_PLACED` | the attack angle of [4]'s motion placed on this body (+5.6°) | check | **Dropped** |
| `fit3d.py` `START_ON` | start 0's onsets guessed from [4]'s event times | a search's starting guess | **Not a rule, but [4]'s.** At most one start of several; the main run has starts that owe it nothing (step 2) |
| the saved fits (`fit3d_d34*.json`, `fit3d_all*.json`) | plans fitted to [4], or searched from one | a search's starting guess | The same: one start among several, flagged, never the reference |
| `body.py` `pose()`, `place()` at −250 ms | **the starting posture and every starting velocity:** the pelvis's rotation and travel, the trunk's rotation, both knees (the lead from its curve, the rear from the text), the pelvis's side tilt (text), the lead foot's angle (text), the bat's azimuth and elevation, the lag, both elbows; each velocity a slope of those curves over ±10 ms; each foot's starting load and friction from [4]'s ground forces | starting state | **Kept this session, flagged in every result as [4]'s** (the owner: a starting stance that roughly looks like the study's hitters at launch is acceptable). It is an input, not a target; step 2 measures how much the answer leans on it. What a start that owes [4] nothing would be: below |
| `body.py` `STRIDE` (0.42 × height between the ankles), `STRIDE_CLOSED` (0.11 m) | where the feet are | starting state | Kept with the start, flagged. Another source for the same thing: OpenBiomechanics' ankles 0.89 ± 0.09 m apart at foot plant ([note 6](research/swing/6-openbiomechanics.md)), 0.07 m apart across; [3]'s 0.85 ± 0.12 m toe to toe. [4]'s 0.42 × 1.84 = 0.77 m is inside both spreads |
| `body.py` `curve_obliquity`, `curve_trail_knee`, `curve_lead_foot` | text values of [4] | starting state | Kept with the start |
| `drive.solve_tone` | the constant joint torques that hold the placed posture still against [4]'s ground forces at −250 ms | starting state | Kept with the start, flagged: it is worked out from [4]'s posture and forces. It also carries the directions that have no drive (the trunk's bending, the ankles' roll) |
| `Model(H=1.84, M=92.0)`; `drive.REF_WH` | [4]'s mean batter | body dimension | **Kept as a size, not as [4]'s.** A professional of 1.84 m and 92 kg is an ordinary one: MLB's listed averages are about 1.87 m and 94 kg [E: common figures, not sourced in this repository; to be read from a roster table before phase 4]. On that size every scaled strength bound would be 3.8% higher (weight × height). The sources' strengths are scaled to the body's own weight × height whatever it is, so no strength here is [4]'s |
| `body.observe()` | the angles are defined "the way [4] defines them" (bat azimuth from +X, the lag from the lead arm's line) | definition | Kept: geometry, not a measurement |
| §13's numeric gate; PLAN §14.1's "reproduces the measured swing" | every item but the engine's checks and the bat's speed | gate | **Superseded** (PLAN §14.1 as amended); the proposed gate is step 3's |
| `test_body3d.py` (the placed posture and its slopes against [4]'s curves at −250 ms) | | a test of the placement code | Kept while the start is [4]'s: it tests that `place()` builds what it says. It goes when the start does |
| `probe3d.py --check` (states placed at −340 and −250 ms; the fitted swing) | | engine check | Kept: any state would do; it compares two implementations of the engine |
| `inverse3d.py`, `trunk3d.py`, `batspeed.py`, `placed3d.py`; the flat probe (`inverse.py`, `fit.py`, `limits.py`, `swing.py`, `breakdowns.py`, `ratings.py`) | diagnoses of [4]'s motion; the flat chain fitted to [4] | history | Not used by the practice search. §4.2–§4.9, §5.2, §6.2 and §7.2 stand as dated history; the breakdowns and the ratings comparison are to be redone on the practised swing |

**§4.5's limits: which were sized or chosen against [4], not taken from a strength measurement.** D34's limits stay; this is the list the job asked for.

- *The hip rotators' 78 N·m* has no source (§4.5 says so). Its range (50–110) was set while [4]'s inverse dynamics was asking 49–68 N·m of them (§4.4), so the range, though not the number, was looked at against [4]. It stays [E]; step 2 does not lean on it unless the rotators end on their bound, which is reported.
- *Hip adduction set equal to abduction* [E], with "amateurs' swings are reported to use 221 N·m" beside it as a check: not [4]'s, but a swing's demand, not a strength. It stays [E].
- *Mean + 2 SD as the average professional's strength.* The rule is §4.5's own (it is how the trunk's bound was first made), but it was extended to every joint in the session where [4]'s swing needed more than the first limits gave. It is a statement about who a professional hitter is among the sources' samples (mostly men who are not athletes), not a number from [4]. It stays under D34; the strongest-batter run of earlier sessions used the same bounds, so "average" and "strongest" differ here only in what the search asks for, which step 2 says again.
- *Which freedoms have a drive.* The knee flexors, the hips' nine channels about the thigh's own axes and session 7's 19 channels were added in the order [4]'s swing showed a need for them. Each is an anatomical muscle group with its own measured strength, so none is [4]'s; but freedoms [4]'s swing did not ask for (the trunk's bending, the shoulder girdle, the ankles' roll) have no drive for that reason alone. Said so in the results.
- *Muscles tensed before they fire* (a starting level per channel) and *the 50 ms rise floor*: sourced from Walshe 1998 ([N8]) and from Maffiuletti's description of early force development (§4.3: an assumption, not a measured rise time); neither is [4]'s. They stay.
- *"The gate reads [4]'s rate curves"* and *the clock paragraph* of §4.5: gate and timing rule, removed above.
- *The end-of-range stops* (`SH_ADD_MAX` 135°, the knees' and elbows' 0–150°) are clinical ranges; their stiffness (2,000 N·m/rad) is [E] with no source. Not [4]'s. Step 1 replaces them.

**A start that owes [4] nothing, and what it costs.** It would be the batter standing still in his stance, held in balance by his own drives, then loading, striding and landing by his own programme, so that the posture and velocities at launch are results. That needs what §5.3 already says a standing body needs and this body does not have: postural feedback through stance, load and stride (intrinsic stiffness and a delayed reflex), a stride programme, the lead foot's landing, and drives for the trunk's bending. That is §13's order item 3 ("the stride, the stance and the postural feedback"), sized there at a session or more; it cannot be built inside this one. A cheaper start that still owes [4] nothing in its *motion*: a still, loaded posture at foot plant built from clinical-looking round numbers (feet 0.8 m apart, knees bent 45°, the pelvis and trunk turned back 20° and 30° from square, the hands at the rear shoulder, the bat cocked), at rest, with the same hold-still torques. It can be built in an hour, but its numbers would be chosen by eye, which is what the owner has allowed of [4]'s launch posture anyway. **So this session keeps the placed start, flagged, and tests how much rests on it by moving it** (step 2: the hands, the trunk's coil, the weight shift and the starting velocities changed by stated amounts, one run from rest).

**Step 1: the aim, the joints' ranges, the checks and what is expected, written before any search ran.** One diagnosis was run first, on swings already saved (no search): `ranges3d.py`, which reads every joint's angle through a saved swing against the ranges below. The owner has since answered Q59 = a (PLAN D38): the bat's speed is read against 29–34 m/s (65–76 mph) only.

*The pitch.* The game's own average four-seam fastball from a right-hander, aimed at the middle of the zone (`sim-cli pitch --type FF --n 200 --seed 1`, the release binary of 2026-11-24; the mean of the 200 at the plane where the ball is met): **38.0 m/s (85 mph) there, coming down at 5.3°, moving 1.6° toward first base, 0.403 s after release, 0.78 m off the ground.** Where it is met, from PLAN D28 and D29 (Savant's stance and intercept figures, [note 11](research/swing/11-stance-depth.md)): 0.92 m toward the plate from the midpoint of the hips and **0.72 m toward the pitcher from where the hips are at the start** [E: 0.61–0.84. Savant's 0.78 m is from the hips at an instant it does not state; if at the stance, the hips have since come forward about 0.17 m; if at contact, they have 0.06 m still to go (OpenBiomechanics' travel, [note 6](research/swing/6-openbiomechanics.md))]. Nothing here is [4]'s except the start it is measured from.

*The aim: what the batter is trying to do, in numbers.* Contact is the instant the sweet spot crosses the plane square to the ball's line at that point. When that happens is a result: the batter times his whole motion to the pitch, so the search does not ask for a duration. *(Not true of the swing found in session 8's fix: with its arm muscles tensed from the start counted as started, item 7's clock caps the time from the start to contact at 253 ms, and that swing sits on the cap; see the fix's results.)* The search minimises the sum of these, each a squared term:

1. **Meet the ball.** The sweet spot's distance from the ball's line at contact, up-and-down and along the bat, each per 1 cm [E].
2. **Stay on the ball's line through contact**, so that a swing a little early or late still meets it: the sweet spot's speed across the ball's line (up-and-down, and along the bat), times a timing error of 7 ms (the measured spread of where hitters meet the ball, 0.27 m, [N6]), times the share of that error the bat's own travel takes up, per 1 cm. A bat coming up through the ball at the angle the ball comes down scores zero here. This comes from the pitch's geometry; no swing was consulted.
3. **Hit it back up the middle:** the bat's axis square to the ball's line seen from above, per 10° [E].
4. **As fast as the body allows:** the sweet spot's speed along the ball's line at contact, rewarded linearly: 1 m/s is worth 1 cm of miss [E]. (In the least-squares search this is the square root of 45 m/s less that speed; 45 is out of reach on purpose.)
5. **Effort (PLAN D36).** For each of the 41 muscle groups, the mean over the swing, from the start to contact, of the square of the fraction of its full strength it is asked for (its level times its strength over its bound); effort is the mean of that over the 41, between 0 and 1. **Weight: one unit of effort costs as much as 4 m/s of bat speed** [E], so one muscle group held at full strength all swing costs 0.1 m/s. Tried at 0, 1, 4 and 16.
6. **On his feet, in balance** (each one-sided, zero when met): (a) from contact to 50 ms after it, the point his centre of mass is heading for, its position plus its velocity divided by √(g / its height) (Hof's extrapolated centre of mass, the standard test of standing balance; not re-read this session), stays over the area his two feet span, per 2 cm outside; (b) his feet carry at least a tenth of his weight between them at every instant (he does not jump), per 5% of body weight; (c) 50 ms after contact his centre of mass is no lower than 0.9 of its height at the start (his legs have not given way), per 2 cm.
7. **The clock: no arm drive before he could have decided.** Its source is the pitch and the game's own latency, not [4]: the ball is in the air 0.403 s; he cannot decide before he has watched it for 0.10 s [E: 0.05–0.15; the game's batter watches a 95 mph pitch for about 0.2 s, agent-models §3.4] and his arms do not answer for 0.05 s after he decides (agent-models §3.4, [E]). **So no arm muscle rises above its holding level more than 0.253 s before contact**, per 2 ms earlier. The legs and trunk start on every pitch (PLAN D22) and are not under this clock.

What the search may change: for each muscle group its strength (0 to its bound of §4.5), when it rises, how long it takes to rise (at least 50 ms), when it switches off, and the level it is held at beforehand: 205 numbers, as before. Nothing asks for a curve, an angle, a ground force or a time of any event.

*What keeps it a human swing: the body, and so its joints' ranges.* `ranges3d.py` on two saved swings, the fit to [4] (§4.9's best start) and the swing asked only for speed, shows that the lead shoulder was the smallest part of it. **With nothing holding the joints, both swings do things no body can:**

| Joint (as the body has it) | Real range | The fit to [4] | Asked only for speed |
| :-- | :-- | :-- | :-- |
| each wrist's cock (the bat turning toward and away from the forearm) | 58° in all (22° toward the thumb, 36° away) | ~~the lead wrist travels 178°, the rear 178°~~ **93° and 127°** | ~~185°, 80°~~ **115°, 79°** (corrected: the first reading turned over past 90° of flexion; session 8's fix, below) |
| each wrist's flexion and extension | 151° in all | 96°, 131° | 114°, 75° |
| the rear thigh behind the pelvis (hip extension) | 17° | 70° | 38° |
| each thigh's turn in its hip | 33° in, 34° out | to 126° and 135° as measured here | 115°, 116° |
| the lead arm across the chest (as then read, a projection; the speed-only swing got there on energy the stop made) | 135° [E] | 141° | 159° |
| the trunk's side bend | 35° [E] | 37° | 39° |
| the trunk's twist on the pelvis | 55° [E] | 9° | 15° |

(Angles up to 50 ms after contact. **Corrected in session 8's fix, below:** it was the thigh's *sideways* angle that over-read when the hip is bent, not its turn, which is right as read; and the wrists' first figures were a reading artefact.) So the earlier swings, the ones matched to [4] among them, got the bat round partly by letting the wrists travel 1.4–2.2 times their range, folding the barrel back toward the forearms, and the back leg trail 70° behind the pelvis. That was not visible in any curve the fits were given.

*The ranges, and the rule for them.* The end of each range is the mean of a sample of adult men measured passively with a goniometer. Which sample: Soucie 2011 (CDC; 114 men of 20–44) where it measured the motion; otherwise Roaas & Andersson 1982 (105 men of 30–40); otherwise Boone & Azen 1979 (109 males, active). Reading status as in the notes; all were read this session by a search worker, whose report says how.

| Range | Ends | From | Read |
| :-- | :-- | :-- | :-- |
| hip, thigh forward and back | 130.4° forward, 17.4° back | Soucie 2011, Haemophilia 17:500, men 20–44 (SD 7.5, 5.9) | TAB (the CDC's page) |
| hip, thigh out and in | 38.8° out, 30.5° in | Roaas & Andersson 1982, Acta Orthop Scand 53:205 (SD 7.0, 7.3) | TAB |
| hip, turning in and out | 32.6° in, 33.6° out | Roaas (SD 8.2, 6.8); Boone & Azen's active 44° and 44° is the range end a rerun tests | TAB; EXC |
| knee | 137.7° bent, 1.0° past straight | Soucie | TAB |
| elbow | 144.6° bent, 0.8° past straight | Soucie | TAB |
| wrist, the cock | 22° toward the thumb, 36° away: 58° of travel | Boone & Azen, through Norkin & White's table A.1 (AAOS: 20 and 30) | TAB (a compilation; means only) |
| wrist, flexion and extension | 76° and 75°: 151° of travel | the same | TAB |
| lead and rear arm across the chest | 135° from straight out to the side | [E], as before: AAOS's clinical figure, not read. Wilk 2012's 369 professional pitchers: 42 ± 8° of horizontal adduction (TAB), which is 132° if its zero is the arm pointing forward; the zero was not read | [E] |
| the same arm drawn back behind the shoulders' line | 45° | [E]: no figure read | [E] |
| trunk's twist on the pelvis | 55° either way | the most that professional batters and pitchers were measured to reach in their own motions, 46 ± 9° and 55 ± 6° (Fleisig 2013, Sports Biomech 12:324); a joint goes at least as far as it has been seen to go | AB. [E: 46–64] |
| trunk's forward, back and side bend | 80°, 25°, 35° | [E]: AAOS's thoracolumbar figures, not read this session | [E] |

- **Where the model has no anatomical zero, the travel is held, not the ends** (the placing was challenged by the review and is kept, with two other placings as reruns: session 8's fix, item 3). Both hands are welded to the handle at the same roll [E, §4.1], so a wrist's angle to the handle has no clinical zero in this body. Each wrist's cock is held to 58° of travel placed so that the start is 5° inside the end toward the thumb [E: a hitter at launch has his wrists cocked; the other placing a rerun tests is the start in the middle]; each wrist's flexion to 151° of travel with the start in the middle [E].
- **Where the start is already at a range's end, the stop begins 2° beyond the start's own angle.** The start is a posture a group of hitters were measured in (flagged as [4]'s), so it is inside a hitter's range. This applies to the lead arm across the chest (135.8° at the start: the stop is at 137.8°).
- **Not held, and said so:**
  - *the ankles*: the measured range (12.7° of the shin coming forward over the foot) is taken with the knee straight and no weight on the foot; the rear ankle is at 20° at the start, as any bent-knee stance is. No weight-bearing figure was read. Their angles are reported;
  - *each forearm's turn and each upper arm's turn about its own length*: the engine's stop holds an angle about one fixed axis and these two are not that; the forearm's also has no zero in this body (the hands' roll). Their travel is reported against the real 162° (pronation 76.9 + supination 85.0, Soucie) and 173° (Boone & Azen's 69 + 104), and a swing that travels further is said to;
  - *how far the arm is raised*: the range (about 170–184°) is never near in a swing.
- **How hard a joint resists past its end: nothing measured was found, for any joint here.** What exists is resistance inside the range, and it is small: the trunk's twist 20 N·m/rad over 4–22 N·m (Kosmopoulos 2012, 18 people, EXC); a hip's turn 10 N·m/rad over its first 20° (Leite 2012, 26 people, EXC); the wrist 0.6–1.9 N·m/rad (Formica 2012, TAB); the hips', knees' and ankles' fitted curves of Riener & Edrich 1999 only second hand, a few tens of N·m at the range's end. Engin & Chen 1986 measured the shoulder's resistance beyond its range; only the abstract could be opened, no numbers. McGill 1994 (the trunk): abstract only. Nobody loads a living joint far past its end, because that is an injury. **So the stiffness is [E]: 20,000 N·m/rad at the trunk, hips, knees and the shoulders' sweep** (350 N·m for each degree past: the 574 N·m the fast swing put on the lead shoulder would get 1.6° past, where the old 2,000 let it go 16°), **2,000 at the elbows and wrists** (their muscles are 17–112 N·m, and a stiffer spring on so little inertia would want a smaller time step). Step 2 tests 2,000 and 6,000 at the big joints. The resistance inside the range is left out (it is small beside the muscles).
- **A joint held at its end is not the same as a joint that can carry any load there.** The stop on the lead arm across the chest stands for the arm lying against the chest, which can take a push. The others stand for ligaments. Every result reports what each stop carried and how far past it went. Stated now, before the run: **if a stop other than the lead arm's carries more than 50 N·m** [E: about what an examiner's push at the end of a large joint's range amounts to], that is reported as the swing leaning on a ligament, and one more run counts each such stop's largest load over 50 N·m as effort (per 50 N·m, weight 1), to see what the swing is without it. That run is a diagnosis; whether the practice search should count it is then the owner's.

*The checks: never given to the search; the practised swing is compared with them afterwards.* Only direct measurements of hitters. "Processing" is note 10's finding on how the source made the number.

| Check | Measured | Source, evidence | Processing |
| :-- | :-- | :-- | :-- |
| the sweet spot's speed at contact | **29–34 m/s (65–76 mph)** (owner, Q59 = a) | Statcast: major leaguers 71.5 mph on average | not published; OpenBiomechanics' 66.2 mph (mostly college) is filtered at 20 Hz and 4–6% low |
| the hands' fastest speed | 10.3–13.0 m/s (23–29 mph), major leaguers | Blast's sensor on the knob, the company's boards (EXC) | not stated. Read here on the handle 0.15 m from the knob [E: where Blast is said to compute it; not re-read] |
| the attack angle at contact | major leaguers about +8 to +10° (Statcast, as §4.4 has it); +4.1 ± 5.9°, 10th–90th percentiles −3.3 to +10.7° (OpenBiomechanics, 677 swings, TAB) | | Statcast's not published; OpenBiomechanics filtered at 20 Hz. The aim asks for +5.3°, the ball's own line, so this checks the aim was met more than it checks the body |
| the bat at its fastest at or just before contact | still speeding up at the usual contact point | the collision's physics; Statcast's bat speed by contact depth ([N10] §1a.5) | read here as: speed at contact at least 98% of its fastest before contact |
| the order of the muscles' first rises | legs, then trunk, then arms | Shaffer 1993, 18 professional batters, fine-wire and surface electrodes ([N9] §4.1, second hand) | phase averages of rectified recordings |
| the lead shoulder's sweep muscle on before and through the swing; the lead triceps highest early; the muscle that raises the lead arm low | 76–101%, 92% then 73%, under 40% of a maximum test | the same | the same |
| the time from the bat's first motion to contact | 0.149 s, major leaguers 0.13–0.17 (Blast's "time to contact", vendor; the range is the game's own table, [E]); college hitters' lead hand starts toward the ball 215 ± 44 ms before contact off a tee (Stewart, TAB, [N9] §4.1) | | Blast's not stated. Read here as the game reads it (agent-models §3.6): from when the hands last rise through a tenth of their fastest speed |
| the front leg braced | the lead knee 46° bent when the foot is planted, 14 ± 12° at contact, straightening at up to 350°/s 50 ms before (OpenBiomechanics, TAB); 15° at contact ([3], 7 professionals off a tee) | sources other than [4], so the check stands | angles: hardly touched by either filter |
| the front foot's greatest load | 2.0 ± 0.4 times body weight, 10th–90th percentiles 1.5–2.6 (OpenBiomechanics, each swing's own peak, 636 swings, TAB) | | force plates filtered at 40 Hz; single swings, so not blunted by averaging |
| the hips' and the trunk's fastest turning | 705 ± 75 and 915 ± 80°/s (OpenBiomechanics, each swing's own); hips 714 ± 76°/s at −75 ms ([3]) | TAB | 20 Hz and 13.3 Hz on peaks more than 100 ms wide: under 2% |
| the most the trunk is twisted back against the hips | 21 ± 6°, about 120 ms before contact (OpenBiomechanics) | TAB | an angle |
| the elbows at contact | rear 79 ± 13°, lead 53 ± 15° bent (OpenBiomechanics) | TAB | angles |
| the lead arm's turning rate | 1,160 ± 96°/s at −65 ms ([3]; read as §4.5 reads it, [E]) | TAB | 13.3 Hz, 65 ms from the impact: not affected |

- **Dropped, because only [4] has them:** the shoulders' elevation curves; the bat's cock and the barrel's drop at contact; the pelvis's angle at contact; the six ground-force curves and their times; every time of an event before contact.
- **For the eye only** (the owner, after D37): each result says in baseball terms whether it looks like a hitter's swing as [4] shows one (the weight coming forward, the front leg firming up, hips then shoulders then hands, the bat held back and then whipping through), with no band and no pass or fail.

*The runs* (step 2), stated before any: the main run is the average batter (1.84 m, 92 kg, §4.5's bounds), this aim, these stops, from **four starts**: (A) session 7's fit to [4]; (B) the swing asked only for speed (§4.9), which the stops pull inside the ranges; (C) a plan written by hand that owes nothing to either: every leg and trunk muscle at 0.6 of its bound from the start, every arm muscle that swings the bat forward at 0.6 from 150 ms after the start, their opposites off, every rise 80 ms; (D) the same with every muscle at 0.3 from the start and every timing shifted at random by up to 50 ms (a fixed seed). Each 30 iterations of the same Levenberg–Marquardt search as before on all 205 numbers; the best is continued 20 more. How settled each was is reported as the change over its last 5 iterations. Then, each 15 iterations from the best: effort's weight at 0, 1 and 16; the big joints' stops at 2,000 and 6,000 N·m/rad; the hips' turn at 44°; the wrists' travel placed with the start in the middle; the start moved (the hands 5 cm further back; the trunk coiled 10° more; the starting velocities halved; the body at rest); the strongest batter (each bound at the top of its tested range in §4.5: hip extension 524, the trunk 290, the shoulders' sweep 136 and 112, hip adduction 185, the hip rotators 110); the small frame (1.75 m, 77 kg). If the session runs short the order is: the main run, the strongest and small frames, the start, effort, the stops.

*What I expect, so the result can be held against it:*

- **Bat speed 24–28 m/s (54–63 mph), below the owner's band.** The speed-only swing's 30.6 came with wrists folding through 185° and the lead arm 24° past its range; with those held I expect it to lose several m/s. It should beat the 21–22 of the fits to [4], which were matching smoothed curves.
- **The hands' fastest speed 8–10 m/s**, still short of the measured 10.3–13.0.
- **The attack angle within 3° of the ball's line** (+5.3°), since the aim asks for it and the fits' −7° came from [4]'s barrel curve.
- **The front leg will not brace as a hitter's does:** nothing in the aim asks for it, and the speed-only swing left it soft. I expect the lead knee over 30° bent at contact and the front foot under 1.5 times body weight; the balance test may firm it somewhat.
- **The hips 650–850°/s, the trunk 850–1,000°/s**; the trunk's twist against the hips more than hitters' 21°, toward the stop.
- **From the bat's first motion to contact about 0.17–0.22 s**, longer than the measured 0.15.
- **The wrists' stops will carry load** (the bat's whip pulls the wrists to the end of their travel in a real swing too), and the hips' turning stops may: this body's knees and feet cannot turn, so the hips' turn has to take all of the pelvis's rotation over planted feet, where a hitter's back foot pivots.
- Effort's weight: the opposing pairs of §4.9 switch off at any weight above zero, at a cost of under 1 m/s.

**Step 2: the practice runs. The search did not find a swing; the session ended inside this step.** *(These runs were made on stops that created energy and with a cock reading that could turn over; their numbers are kept as the record and are superseded by the runs of session 8's fix, below.)* `practice3d.py` (the aim, the stops and the checks above, in code), `ranges3d.py`, and the outputs `practice3d_main.out`, `practice3d_vary1.out` and `practice3d_which_stops.out` under `sim/target/codex-probes/body3d/`. Every run starts from the state placed on [4] at −250 ms (an input, flagged). Jobs 357 and 358: 6 and 5 minutes.

*Before the search: two things the stops showed, each from single swings.*

- **WITHDRAWN (session 8's fix, item 1): this bullet's mechanism is wrong and its table mixes two bodies and two measures.** The earlier fast swing was not a bounce: the engine's stop created the energy. The corrected table is in the fix's results. As first written: ~~A spring at the end of a joint's range is a trampoline, and the earlier "fast" swing was bouncing on one.~~ Each earlier swing run once with one group of stops at a time (`practice3d_which_stops.out`; the bat's fastest speed in the run, at a 0.125 ms step):

| Stops on the body | The fit to [4] (§4.9) | The swing asked only for speed (§4.9) |
| :-- | --: | --: |
| sessions 1–7's body (soft stops at the shoulders, knees and elbows) | 21.3 m/s | 30.6 m/s |
| none at all | 18.7 | **17.5** |
| the shoulders' only, at 20,000 N·m/rad | 20.2 | **48.0**, the lead shoulder's stop carrying 6,570 N·m |
| the wrists' only (58° of travel) | **7.5** | 8.7 |
| the hips' only | 14.1 | 14.2 |
| the trunk's only; the knees' and elbows' only | 18.7; 18.7 | 17.3; 19.6 |
| all | 11.6 | 49.5 (3,775 N·m on the lead shoulder) |

  The swing asked only for speed has 17.5 m/s with no stop, 30.6 with the old soft stop and 48 with a stiff one: ~~**its speed was the lead arm thrown against the end of its range and flung back by the stop's spring, not muscle.**~~ *(withdrawn: the stop created the energy)* So §4.9's "the same limits allow 30.6 m/s" was an artefact of the stop, and "the strongest batter" of sessions 7 and 7-fix says nothing about strength. A one-sided spring with little loss gives back what it is given; a real joint's end does not, and nothing measured says how much it loses (above). **With stiff stops a violent swing also cannot be integrated at 1 ms** (the run fails, or reads 37 m/s at 0.5 ms and 46 at 0.125), so the search was run at 0.5 ms and every result rerun at 0.125 ms.
- **WITHDRAWN (session 8's fix, item 3): not shown.** A single replay of a plan tuned without stops cannot show what the hands can do; what it shows is that the earlier plans folded the wrists the wrong way. As first written: ~~Held to a wrist's real travel, this body's hands cannot turn the bat.~~ With only the wrists' stops on, the fit to [4] has the bat at 7.5 m/s. Both hands are welded to the handle at one roll, the wrist's flexion axis lies along the handle (so its 151° of travel only rolls the bat about itself), and the cock's 58° is all that is left to swing the bat round each forearm; the rear forearm sweeps through far more than that against the bat as the rear elbow straightens. A hitter's top hand takes that up in the fingers, the palm and the forearm's turn. **The hips are second:** held to their ranges the same swing has 14.1 m/s; the rear thigh can no longer trail 70° behind the pelvis, and this body's rear foot does not come up onto its toe and turn as a hitter's does.
- **CORRECTED (review P2-1): the two earlier swings passed at 1.0–1.2 m, the top of the zone and above, with the hands at the shoulders; "24–45 cm lower" is where those swings went, not a limit of this body's reach.** As first written: ~~The earlier swings also passed nowhere near a strike.~~ When its bat is square the fit to [4] has the sweet spot 1.23 m off the ground, 0.73 m from the hips toward the plate and 0.31 m ahead of them; the speed-only swing 1.02 m, 0.70 and 0.39. The ball of the aim is at 0.78 m, 0.92 and 0.72: 24–45 cm lower, about 20 cm further out and 33–41 cm further out front. No curve of [4] says where the bat is, so seven sessions did not see that the body was swinging at shoulder height with its hands at its shoulders. A hitter gets the barrel down by tilting and bending his trunk (4° of side tilt when the front foot plants, 32° at contact, OpenBiomechanics); this body's trunk has no muscles for bending (§4.5: nothing in the notes measures them).

*The main run* (the average batter, the aim and stops as written; 25 iterations a start, the best continued 15; 205 numbers):

| Start | Iterations run | How it ended | The bat at the end |
| :-- | --: | :-- | :-- |
| A, the fit to [4] | 25 | still falling slowly (0.7 in the last 5, cost 837) | crosses the ball's plane 283 ms after the start, 9 cm above the ball and 21 cm short along the bat, at 15.8 m/s, coming down at 20°. Not a swing: the trunk's side stop carries 10,600 N·m, the hips read 2,500°/s, both feet leave the ground |
| B, asked only for speed | 20 | stopped, its steps refused (cost 3,290) | 51 cm above the ball; 37 m/s at a 0.5 ms step and 46 at 0.125: not a result (~~a bounce off the lead shoulder~~: energy made by the lead shoulder's stop, 3,700 N·m on it) |
| C, by hand, 0.6 of the forward muscles | 22 | stopped (cost 571) | never reaches the ball's plane in 350 ms; ends 6 cm from the ball moving down at 12 m/s |
| D, by hand, 0.3 of everything, random timings | 7 | stopped, its steps refused (cost 2,704) | never reaches it; the rear shoulder's stop at 42,000 N·m |
| C continued | 8 | stopped (cost 523) | the same as C |

**No start became a swing that meets the ball.** The lowest cost belongs to a motion that lowers the bat to within 5 cm of the ball in 350 ms without ever swinging through it. None of step 1's checks can be read against that (for the record, C continued: the sweet spot 12 m/s, the hands' fastest 5.0 m/s, the hips 278°/s, the front foot never over 0.63 of body weight; ten stops over 50 N·m). **Every expectation written in step 1 was wrong:** I expected a swing of 24–28 m/s.

*The follow-up runs* (20 iterations each from C continued; since that is not a swing, most could not become one):

| Run | Result |
| :-- | :-- |
| load on a ligament counted (the run stated in step 1) | stopped after 5 iterations; the same non-swing, ten stops still over 50 N·m |
| the strongest batter (each bound at the top of its tested range) | never reaches the ball's plane; 9.8 m/s |
| the small frame (1.75 m, 77 kg) | never reaches it; 11.2 m/s |
| *a diagnosis added after the main run:* the ball at the top of the zone (1.05 m) | never reaches it; ends 1 cm above and 4 cm short |
| *a diagnosis added after the main run:* a ball high and close, about where the earlier swings' bats went (1.20 m up, 0.75 m across, 0.40 m ahead; no pitch a hitter is thrown) | **the only run that swung: meets the ball (0.1 cm), 297 ms after the start, at 15.5 m/s, coming down at 18°;** hands' fastest 7.2 m/s; hips 444°/s, trunk 572°/s; the lead knee 15° bent at contact, the front foot at 1.0 times body weight; the lead shoulder's stop at 3,750 N·m and the lead wrist's at 3,160. Still falling a little (1.4 in the last 5 iterations). Step-checked: 15.5 m/s at both steps |

Not run: effort's weight at other sizes, the stops' other stiffnesses, the hips' turn at 44°, the wrists placed in the middle, the moved starts. Each would have started from a motion that is not a swing, so none would have said what it was meant to. **So nothing can be said yet about how much rests on the start, on effort's weight or on the stops' stiffness, nor about the strongest batter and the small frame.**

*What this says, and what it does not.*

- **It does not show that a body cannot swing by practice.** It shows that this body, with its joints held to real ranges by plain springs, and this search, did not get there in one session. Three separate things stand in the way, and the runs do not say how much each weighs:
  1. *The body's hands* (**withdrawn as a cause**; see session 8's fix, item 3)*.* §4.1's weld of both hands to the handle at one roll cannot work inside a wrist's real travel (7.5 m/s above). This is a missing part of the body, and the first thing to build: each hand's own grip (its own roll about the handle, and how the handle lies across the palm), sized from anatomy, with the forearm's turn held to its range.
  2. *The body's reach to a strike* (**not a cause shown; a thing to test**, and tested in the fix's runs)*.* ~~The ball is 45 cm below and 20 cm beyond where this body's bat goes.~~ *(That was where the two earlier swings went, not a limit of this body: the fix's search meets the aim's ball.)* The trunk's bending has no muscles; the rear foot cannot come up and turn; the knees and feet cannot turn, so the hips' turn has to take all of the pelvis's rotation. Each has a measurable strength or range; none is in the notes.
  3. *The search.* With stiff springs at the joints' ends the motion changes abruptly with small changes of a timing, and a search that feels its way by small differences (2% of each number's range) gets refused steps: three of four starts stopped within 7–22 iterations. A bat that never reaches the ball's plane is also scored by a different rule from one that does, which makes a cliff. The search needs stops that take energy out (so the motion is smooth in the timings), a score with no cliff, and probably a start built in stages (first get the bat to the ball, then ask for speed), as `fit3d.py` staged the legs before the arms.
- **What is established, as corrected by session 8's fix:** the earlier fast swing's speed was energy the engine's stop created (not a bounce); the fits to [4] took the wrists through 1.4–2.2 times their travel, folding the bat back toward the forearms where a hitter's wrists open it out, and trailed the back leg 70° behind the pelvis; and they swung at 1.0–1.2 m with the hands at the shoulders. None of the bat speeds of §4.6–§4.9 was a swing a body could make.
- **For the eye:** nothing here looks like a hitter's swing yet. The one run that met a ball (high and close) has the weight coming onto the front foot and the front knee straightening from 48° to 15°, and the hips ahead of the trunk by 19°, which is the right picture; but the bat chops down at 18° at 35 mph.

**Session 8's fix (2026-11-25): what the independent review found, what was changed, and the runs again.** [The review](reviews/2026-11-25-opus-body-3d-s8-review.md) found three defects that change step 2's conclusions. **Where a block above states something that no longer stands it is marked there and points here.** Each fix has a test that fails on the code as it was; `probe3d.py --check` passes with the same law in the compiled probe.

*1. The joints' stops created energy (review P1-1; an engine defect from the first shoulder stop, session 2).* A stop reads an angle and pushed back about a fixed axis. On a hinge that axis is the direction the angle changes in; on a ball joint (shoulder, hip, the joint between pelvis and trunk) it is not, and the stop did work that nothing stored paid for. **The 30.6 m/s of §4.9's "strongest batter" was that energy (+414 J from the lead shoulder's stop), not a bounce, and not the batter's.** Fixed: the stop's torque and damping act along the read angle's own gradient, and its work is booked with it (§8.1). Tests: a rod alone on a ball joint never gains energy from a stop, tilted or not; the old law fails that test.

- *What it changes in numbers* (each earlier plan run once on the body it was found on, the bat's speed when its azimuth passes −8°, 0.125 ms step): **the fit to [4] 21.3 → 17.8 m/s; the swing asked only for speed 30.6 → 15.6 m/s.** The stops' work is now −5 and −8 J. So every bat speed in §4.6–§4.9 included energy the stop made.
- *Where each stop's angle is undefined, by design.* A stop that reads a direction projected about an axis has no angle when the direction lies along the axis; one that reads a direction's angle out of a plane has no gradient at ±90°. The engine fails the run loudly within 5° of either (no clamp). So that no swing comes near one:
  - the arm across the chest and the arm drawn back are now read as the upper arm's angle out of the trunk's fore-and-aft plane and out of its side-to-side plane. (The old reading, "seen down the trunk's axis", has no value with the arm hanging, and its gradient is already 2.4 times too large 25° from hanging, which a hitter's back arm reaches.) These are undefined only with the arm pointing straight across the body or straight back, 45° beyond their own ends, and a stop of this kind asks for its gradient only while it is engaged;
  - the thigh's sideways angle is read the same way, out of the pelvis's fore-and-aft plane (item 4): undefined only with the thigh straight out to the side, 51° past its end;
  - the thigh's forward-and-back angle and its turn: undefined with the thigh straight out to the side, which the sideways stop keeps 51° away;
  - the trunk's twist, forward bend and side bend: undefined with the trunk bent 90° forward or sideways; their own stops end at 80° and 35°;
  - the knees, elbows and wrists are hinges or two-axis joints whose stops read the joint's own axes: never undefined.

*2. The wrist's cock was read wrongly past 90° of flexion (P1-3).* The reading projected the forearm's long axis, which turns over by 180° when the wrist's flexion passes 90°. It is now the angle of the forearm's own axis about the bat's, which is the joint's cock exactly. Test: flexion carried through 130° leaves the cock's reading where it was. **Step 1's "178°" for the wrists was that artefact:** on the joint's own axis the earlier swings' cock travelled 93° and 127° (the fit to [4]) and 115° and 79° (the speed-only swing) to 50 ms after contact (the review's figures, its probe 7): 1.4–2.2 times a wrist's 58°, not three times.

*3. The wrists (the first review's P1-2 and the second review's P1-1): the direction and the lead wrist were right; the rear hand is outside a wrist's range, because of the weld.* **Both halves have to be said together.** The first half, as written for the fix: The review found that through the fit to [4] the cock's reading rises while the bat's lag falls, concluded that uncocking raises the reading, and so that the 58° had been put on the wrong side, leaving 5° to uncock. Its measurements are right. Its reading of them is not, and the evidence is this:

| | Lead wrist | Rear wrist |
| :-- | :-- | :-- |
| what the reading is | with the wrist unflexed, 90° less the angle between the forearm (wrist toward elbow) and the barrel (in general the cosine of that angle is sin(cock) × cos(flexion)): 0 with the forearm square to the handle; **positive is the barrel folded back toward the elbow; negative is the bat opened out toward a straight line with the forearm** | the same |
| at the start | −6° (the bat 96° from the forearm) | −66° (156°: this forearm already nearly in line with the bat, a product of how §4.1 welds both hands at one roll) |
| the bat turned about the wrists' own axis so that the lag falls | the reading **falls** | **falls** |
| the body driven by its two uncocking wrist muscles alone, 40 ms (a test) | the lag falls; the reading **falls** | **falls** |
| through the fit to [4], start to its contact | **rises to +59°** (the bat 64° from the forearm), read on the body of sessions 1–7 at 0 ms on its clock; on the practice body with no stops, to the bat's azimuth −8°, the second review reads +24° (74°) | **rises to +58°** (64°); the second review's: +62° (60°) |
| hitters measured directly ([note 11](research/body/11-hands-and-joint-ends.md) A5; for checking, not targets) | tilts from 10° toward the thumb at foot plant to **16° away from it at contact** (OpenBiomechanics, 650 swings); the bat **138–157° from the lead forearm at contact, "increasing from a cocked start"** (Gélinas 1988, EXC); the knob-side hand's excursion 40 ± 6°, fastest away from the thumb at contact (Flynn 2021, TAB) | from 5° toward the thumb to 8° away (OpenBiomechanics); held 37–77° extended throughout |

So a hitter's wrists open the bat out from the forearm through the swing, which in this body is the reading falling, and that is where step 1 put the travel: right for the lead wrist, and the right direction for both. **The swings fitted to [4] did the opposite: they got the bat round with both wrists folding the barrel back toward the forearms, by 65° and 124°, to 64° from each forearm at contact, where a hitter's lead forearm and bat are 138–157° apart.** The lag fell in those swings only because the lead arm's line swung, not because the wrists released. That is also why §4.9's fit switched both uncocking wrist muscles off. The review's placing (the travel toward the barrel folding back) lets those plans run on (15.1 and 16.3 m/s in its probe 11 against 7.5 and 8.7) because it permits what they do, not because it is where a wrist's travel lies.

- *What was done.* The direction is now taken from the model, not written by hand (`body.uncock_sign`: the bat turned about the wrists' axis so that the lag falls), as the job asked; it comes out on the side step 1 had. The test drives the uncocking wrist muscles and fails if the cock moves toward the end with less than half the travel; the review's placing fails it.
- *Where the 58° is placed, an [E]:* each wrist starts 5° inside its cocked end, the other 53° toward the bat opening out [E: a hitter at launch has his wrists cocked]. **Two other placings are reruns** (below): the review's, and the clinical 22° and 36° either side of the forearm square to the handle, which for the rear wrist has to be widened by 30° to take in the start (its forearm starts where no wrist could put it; that is the weld's fault and stays flagged).
- **The second half (the second review, P1-1, which the fix's account missed).** Both hands grip thumb toward the barrel, so the same reading means the same wrist posture on either hand, and hitters' two wrists start within about 5° of each other. This body's rear wrist starts 60° more opened out than its lead wrist (the forearm 156° from the barrel against 96°), 7° past the opened-out end the lead wrist is given. Its 58° of travel, placed from that start as if it were a cocked wrist, runs from 151° through the line of the bat (180°) to 29° past it on the other side of the handle, where no wrist goes. The cause is §4.1's weld: both wrist joints sit on the same side of the handle, so both forearms come at it from that side, where a hitter's palms face opposite ways. In this weld the rear forearm can reach a hitter's top-hand side of the handle only by going through the line of the bat, with the travel placed either way. **So neither "the travel was on the wrong side" nor "the wrists are set up right" is the thing to say: the wrists uncock the right way, the lead wrist is a hitter's, and the rear hand is a missing part of the body.** `test_the_rear_wrists_range_is_one_a_wrist_reaches` states it and is an expected failure until each hand has its own grip. The stop is not moved in this job.
- **Does the swing's speed rest on it? Not much, by the reviewer's replay** (his probe `q1_rear_on_lead_interval`, under `sim/target/codex-probes/review-body-3d-8fix/`; not rerun here): the practised plan replayed with the rear cock held to the lead's own interval of the reading, widened only to clear its start (−67.8 to −1.2), has 17.9 m/s at contact against 17.8, with the miss moved to −1.1 cm up and +4.8 cm along the bat.
- *What stands of step 2's "the hands".* Not "this body's hands cannot turn the bat": a single replay of a plan tuned without stops cannot show that (the review is right there). What the replays show is that the earlier plans needed the wrists to fold the wrong way. Whether the welded hands can swing a bat inside a wrist's travel is for the search to say (below). The rear wrist's start, and [note 11](research/body/11-hands-and-joint-ends.md) A1's finding that a wrist's range while gripping is not a box (flexion falls by 13–27% with the hand closed, and the two directions trade against each other), are the next session's.

*4. The hip's sideways angle (review P3).* It was read as the thigh projected about the pelvis's fore-and-aft axis, which over-reads when the hip is also bent (30° reads 39° at 45° of flexion), so that stop bit early; the lead hip's start read 37.9° and is 36.0°. It is now the joint's own angle. Test: 30° reads 30° at 0, 45 and 70° of flexion. **Step 1's caveat pointed at the wrong angle:** the thigh's turn and its forward-and-back angle were right as read (they are the standard joint coordinates), and so was the rear thigh's 70° behind the pelvis.

*5. The arms' clock (P2-3, P2-4).* A bat that never comes round is no longer charged the clock against the run's end. **An arm muscle tensed beforehand counts under the clock:** a starting level over 1% of the muscle's bound is drive from the first instant, so it is an arm drive that rose at the start. The other way was tried and is kept as an option: re-solving the hold-still torques for each plan's starting levels, so that a tensed muscle moves nothing at the start (test: it then does not). It is not used because the hold-still torque that balanced the muscle then stays on for the whole swing and takes that share of the muscle's strength away: the fit to [4]'s plan replayed that way has the bat at 2.4 m/s. A muscle that is tensed, holds, and then fires with all its strength needs a drive that carries no net torque before its onset; the engine has none. So for now a hitter's arms hold the bat with the hold-still torques alone, and the recorded pre-swing activity of the lead shoulder (76–101%) cannot be met: said in the checks. **What this rule does in practice (the second review, P2-2):** a plan whose arm muscles start tensed has an arm drive "started" at the run's first instant, so the clock reads "contact no later than 253 ms after the start". It is then a cap on the whole motion from the starting posture, not a limit on the arms' swing, and the swing found below sits on it (258 ms). Under this rule a hitter, whose lead shoulder is recorded working before the swing, would fail any viewing time; so how a tensed muscle counts has to be settled before the viewing time can be asked about.

*6. The run's length.* On the corrected body the first rerun of the main run (job 359, the run 350 ms long as in session 8) ended every start with the bat arriving at the ball exactly at the run's last instant, where nothing after contact is seen (start A: 20.6 m/s along the ball's line at 350.0 ms). The run is now 450 ms and a contact counts only if the 50 ms after it lie inside the run. Job 359's output is kept (`sim/target/codex-probes/body3d/s8fix_350ms/`); it is not a result.

*Stated before the further runs* (written while the 450 ms main run, job 360, was in the queue and before its result was read):

- **The main run** is §4.10's as written, on the corrected body: the same aim, the same four starts, 25 iterations each and the best continued 15, the stops plain springs at 20,000 and 2,000 N·m/rad with the damping they had (60 and 20 N·m·s/rad), the wrists' travel toward uncocking, a 0.5 ms step with every result rerun at 0.125 ms.
- **If a start becomes a swing:** every check, then the follow-ups in §4.10's order, each 15 iterations from the best: the strongest batter and the small frame; the start moved; effort's weight; the stops' stiffness; and the wrists' two other placings.
- **If none does: a staged search**, from the same four starts. Stage one, 15 iterations: the score is how near the sweet spot comes to the ball's place at any instant of the run (three directions, per 1 cm), with balance, effort and the clock read at that instant; no speed, no line, no squareness, and so no cliff between a bat that reaches the ball's plane and one that does not. Stage two, 25 iterations from where stage one ended: the whole aim as written. Then **the reach one coordinate at a time** (review P2-1), each a staged search of 15 and 20 iterations from the four starts: a ball high and close (1.20 m up, 0.75 m across, 0.40 m ahead: where the earlier swings went); the same with its height alone at the aim's 0.78 m; with its distance ahead alone at 0.72 m; with its distance across alone at 0.92 m.
- **An amendment, written after the 450 ms main run's first start was read and before any staged run** (job 360, stopped after start A): with the longer run the search again parked the bat at the ball at the run's last instant (5.8 m/s, 0.5 cm away, never crossing the ball's plane). That is the score's fallback for a bat that never comes round, which reads the run's end: a fault of the search's rule, the "cliff" of step 2, not of the body. So in the staged search's second stage the search reads contact as the instant the sweet spot is nearest the ball's place, anywhere up to 50 ms before the run's end, and scores the miss in all three directions there. When the bat passes through the ball that instant is the plane's crossing, so the aim is unchanged; a bat that has not reached the plane is scored by the same rule, so there is no cliff and nothing to gain by arriving at the run's end. **The report still judges every result by the plane's crossing as the aim defines it.** The main run as §4.10 wrote it is therefore reported as what it is, a search that parks, and the staged search is the one whose results are read.
- **PLAN D39 (a joint's end takes the motion's energy and the search avoids heavy load on it) is not built in this job.** It is the next session's first item, with [note 11](research/body/11-hands-and-joint-ends.md) B: nothing is measured past a joint's end; inside the range 78–87% of the energy put in comes back; cadaver ligaments yield or fail at 13–34 N·m, so step 1's 50 N·m line for "a heavy load" has nothing behind it and its size is for that session to state.

*The runs again* (all on the corrected body; outputs under `sim/target/codex-probes/body3d/`: `which_stops3d.out`, `s8fix_350ms/`, `s8fix_450ms_parked/`, `practice3d_staged_main.out`, `practice3d_staged_main_BCD_retry.out`, `practice3d_main.out`, `practice3d_vary1.out`, `practice3d_vary2.out`; jobs 359–365; every result at a 0.5 ms step and again at 0.125 ms; the start is still the state placed on [4], an input, flagged).

**The which-stops table, redone** (`which_stops3d.py`, committed; one body, one measure: the sweet spot's speed when the bat's azimuth passes −8°, the instant those plans were timed to; 0.125 ms step; the stops' net work to that instant beside it):

| Stops on | The fit to [4] | The swing asked only for speed |
| :-- | :-- | :-- |
| none | 16.5 m/s; stops 0 J | 16.3; 0 |
| the wrists', travel toward uncocking | 6.9; −6 J | 4.9; −13 |
| the wrists', travel the other way (the review's) | 12.5; −3 | 13.1; −16 |
| the wrists', the clinical range about the forearm square to the handle | 15.2; −3 | 15.4; −2 |
| the hips' | 14.1; −20 | 14.4; −32 |
| the trunk's; the shoulders'; the knees' and elbows' | 16.5; 16.2; 16.5 (0, −1, 0 J) | 16.4; 15.9; 17.8 (−12, 0, −2 J) |
| all | 1.6 at +69 ms; −30 | 9.0; −64 |
| all, the wrists' travel the other way | 10.5; −26 | 11.4; −70 |

No stop does positive work anywhere. With the shoulders' stops alone the speed-only plan has 15.9 m/s where session 8 read 48: that was the old law. **How little this table shows:** each plan was tuned on a body with other stops under a law that made energy, so a replay with a stop added says what that plan does, not what the body can do; the searches below are what count.

**The main run as §4.10 wrote it parks the bat.** Four starts, 25 iterations each. In a 350 ms run (job 359) every start ended with the sweet spot arriving at the ball at 350.0 ms, the run's last instant (A within 0.1 cm at 20.6 m/s; B and C within 1–2 cm at 12–13 m/s; D 3–5 cm off at 6 m/s); in a 450 ms run (job 360, stopped after start A) at 450 ms, at 5.8 m/s. The searches themselves now run their full 25 iterations with the score falling smoothly, where session 8's stopped in 7–22 with steps refused; but what they find is the hole in the rule for a bat that never comes round. Not a result about the body.

**The staged search** (as stated above; jobs 361 and 363; the same four starts; stage one 15 iterations, stage two 25; starts B, C and D run a second time after the search was made to retry a refused step with closer differences):

| Start | Stage one (reach) | Stage two (the whole aim) | The bat at the end, by the aim's own reading |
| :-- | :-- | :-- | :-- |
| A, the fit to [4] | 15 iterations, still falling fast; nearest the ball 2.9, 4.4 and 1.4 cm | 25 iterations, score 50.3, fell 1.7 over the last 5; continued 30 more (job 362): **46.7, fell 0.013: settled** | **a swing: crosses the ball's plane 258 ms after the start, 1.0 cm above the ball and 1.6 cm along the bat, at 17.8 m/s** (the same at 0.125 ms) |
| B, asked only for speed | 15; 3.3, 5.9, 1.6 cm | first time stopped after 1 iteration; second time 25, score 169, falling 1.5: not settled | crosses at 262 ms, 2.0 and 6.8 cm off, 12.8 m/s, coming down at 22°, both feet off the ground at some instant |
| C, by hand | stopped after 2 iterations, then after 13: 20 cm from the ball | stopped after 3 | never reaches the ball's plane |
| D, by hand, random timings | 15; 6.0, 13.4, 0.3 cm | stopped after 16, score 301 | crosses at 280 ms, 6 and 18 cm off, at 6.9 m/s |

**One start of four becomes a settled swing; the two that owe nothing to an earlier fit do not.** The search is still what decides whether a start gets there, so "from at least two starts that owe each other nothing" (§13) is not met, and the swing below is the best found, not shown to be the best there is.

**The practised swing against the checks** (start A, settled; `practice3d_main.json`; never given to the search):

| Check | Measured | The swing | |
| :-- | :-- | :-- | :-- |
| meets the ball; on his feet | within 1 cm (§13); the aim's three balance tests | 1.0 cm up, 1.6 cm along the bat; his centre of mass heads 3.0 cm outside his feet's area at worst; his feet carry 0.43 of his weight between them at contact and 0.33 at +30 ms (never under 0.14), and his centre of mass ends 5% lower than it started: he is dropping at about 0.6 g as he meets the ball; 258 ms after the start | **fails** the 1 cm (by 0.6 cm) and balance test (a) (by 3 cm); passes "does not jump" only because its floor is a tenth of his weight |
| sweet spot at contact | 29–34 m/s | **17.8 m/s (40 mph)**: the hands' travel 2.1 + the bat's turn about them 15.8 | **fails** |
| the hands' fastest | 10.3–13.0 m/s | 7.7 m/s, 50 ms before contact; 4.3 at contact | fails |
| attack angle | −3 to +11° (major leaguers +8 to +10) | +7.3° (the ball's line asks +5.3) | passes |
| the bat at its fastest at contact | 98% or more | 91%: fastest 19.7 m/s 32 ms before | fails |
| the hands' first motion to contact | 130–220 ms | 204 ms | passes |
| first rises: legs, trunk, arms; the clock | in that order; no arm drive more than 253 ms before contact | legs at the start, trunk 21 ms later, **15 arm muscles tensed from the start**, which count as started; so the first arm drive is 258 ms before contact: 5 ms more than the rule he was searched under allows, and the swing's length from its start is that cap, not a free result | fails both |
| the front leg braced | lead knee 2–26° at contact, straightening into it | 48° at the start, 55° at −50 ms, 42° at contact | fails |
| the front foot's greatest load | 1.5–2.6 × body weight | 0.69, 80 ms before contact; at contact front 0.17, rear 0.26 | fails |
| the hips' and trunk's fastest turning | 555–855; 755–1,075°/s | 331 and 540°/s, both 131 ms before contact; the pelvis turns 35° in all, the trunk 70° | fail |
| no passive element does net positive work | at most 0 | to contact the muscles did 361 J, the stops −39 J, the ground −31 J | passes |
| no joint more than 2° past its range | | the most is 1.9° (the rear elbow, after contact) | passes |
| no unheld joint further than a body goes | forearm 162°, upper arm's turn 173° | the rear upper arm turns 176° (148° to contact); the rear forearm 106° (99° to contact), about twice a hitter's 50° and inside a body's range. *(First reported as 285°: the forearm's reading turned over when the rear wrist's cock passed the line of the bat, 45 ms after contact; the second review's P1-2; fixed, with a test.)* | fails, by the upper arm's 3° |
| a ligament's load (PLAN D39, not yet built) | cadaver ligaments yield or fail at 13–34 N·m; a knee bent sideways to injury at 107 ± 64 ([note 11](research/body/11-hands-and-joint-ends.md) B3; no figure for a hip) | nine joint ends at 75–906 N·m: the rear hip's turn 906 (11 times its own rotators' bound), the trunk's twist 636, the rear hip back 578, the rear knee 509, the rear elbow 384 (after contact), the lead hip's turn 323, the lead hip sideways 259, the trunk's side bend 207, the lead wrist 75 | **the swing leans on what no joint can carry** |
| *reported, not gated:* lead shoulder's sweep muscle; lead triceps; the lead arm's raiser | 76–101%; 92% then 73%; under 40% | 0.35–0.41; 0.57 throughout; 0.57 | not as recorded |
| the trunk twisted back against the hips | 21 ± 6° | 3°; by contact the trunk is 56° *ahead* of the hips, against its stop | not as measured |
| the elbows at contact; the lead arm's rate | 79 ± 13°, 53 ± 15°; 1,160 ± 96°/s | 26°, 18°; 542°/s | not as measured |

- *What it sits on.* At full strength, 5 of 41: the rear hip's rotators and adductors, the lead forearm's pronators, the rear shoulder's external rotators, the rear forearm's supinators. Eleven rises are at the 50 ms floor. Effort 0.19; two opposing pairs on together. *(Corrected, the second review's P1-3: a count of muscles at their bounds does not say whether strength limits the speed. The rear hip's rotators, which turn the pelvis, are among the five, on a bound with no source, §4.5.)*
- *Where it came from:* start A, the plan fitted to [4], from [4]'s placed launch state. The two starts that owe [4] nothing (C and D) did not become swings.
- *For the eye:* it does not look like a hitter's swing. The weight stays on the back leg, whose knee straightens; the front knee stays bent and never takes the weight; the hips open 35° and stop; the shoulders turn on past them to the end of the trunk's twist; the arms reach out nearly straight; the barrel hangs 48° below the hands and is 30° past square at the ball. The bat does come up through the ball along its line.
- *Against what step 1 expected.* Bat speed 24–28 m/s: wrong, 17.8. Hands 8–10 m/s: 7.7. Attack angle within 3° of the ball's line: yes. The front leg not braced, the knee over 30°, the front foot under 1.5 × body weight: yes. Hips 650–850 and trunk 850–1,000°/s: wrong, half that. The trunk twisted back toward its stop: wrong way round. First motion to contact 0.17–0.22 s: yes. The wrists' stops carrying load (75 N·m) and the hips' turning stops (906): yes. Opposing pairs off when effort costs: nearly (two left), but they are off at zero weight too.

**How much rests on what** (each 15 iterations from the settled swing with one thing changed, the search reading contact at the nearest approach; "fell" is the score's fall over the last 5 iterations). **Read against 18.0 m/s, not 17.8:** the "settled" swing (it had fallen 0.013 over its last 5 iterations) moved to 18.0 when carried on the same 15 iterations, as the three effort rows show, still falling 0.5–0.9 per 5. **So a difference under about 1 m/s is inside what an unsettled search moves**, and most rows below were still falling by more than that.

| Changed | Sweet spot at contact | Against 18.0 | Miss | Both feet left the ground? | Notes; fell |
| :-- | --: | --: | :-- | :-- | :-- |
| effort's weight 0; 1; 16 (the main run: 4) | 18.0; 18.0; 18.0 m/s | 0 | 0.7–0.8, 1.4 cm | no | effort 0.195, 0.194, 0.193; two opposing pairs at each. Effort costs 0.8 of the score's 46.7 here, so this shows only that nothing rests on its weight for changes near this plan; 0.5–0.9 |
| the big joints' stops at 6,000 N·m/rad (20,000) | 18.2 | +0.2 | 0.4, 1.4 | no | 1.1 |
| the same at 2,000 | 20.4 | **+2.4** | 0.2, 0.6 | **yes** | the joints run further past their ends; the stops take −100 J; 4.8, not settled |
| the hips' turn 44° each way (33°) | 18.2 | +0.2 | 0.6, 1.2 | no | **the rear hip went to its new end (764 N·m on its turning stop) and the pelvis turned 3° more (18° at contact against 15°); the hips still 330°/s.** The extra range was used at the hip. With the feet flat and the knees hinges, the hips' turn is most of what the pelvis has: the planted feet are the first thing to test; 1.1 |
| the wrists' travel the other way (the first review's placing) | 15.0 | −3.0 | 9.1, 14.5 | no | from this plan the swing has to be found again: still falling 14 after 15 iterations, not a result |
| the wrists' clinical range about the forearm square to the handle | 16.9 | −1.1 | 2.1, 3.9 | no | settled (0.01) |
| the start: the trunk coiled 10° more | 18.3 | +0.3 | 1.2, 2.2 | no | attack +10.7°; stopped after 11 |
| the start: the rear elbow 15° more bent (the hands nearer the rear shoulder; step 1's "5 cm further back" was not buildable as written, since the placement solves the hands from the elbows) | 19.1 | +1.1 | 0.7, 1.3 | **yes** | 2.6 |
| the start: the pelvis 5 cm further back | 18.2 | +0.2 | 2.3, 2.8 | no | attack +16.8°; 2.3 |
| the start: every starting velocity halved | 18.5 | +0.5 | 0.8, 1.4 | **yes** | 2.9 |
| the start: at rest | 18.8 | +0.8 | 0.4, 0.8 | no | 2.5 |
| **the strongest batter** (nine bounds raised 11–41% to the top of their tested ranges) | **21.6 (48 mph)** | **+3.6** | 0.1, 0.3 | **yes**: the front foot carries nothing at contact, the rear 0.09 of his weight, both off the ground by +30 ms | hands 9.2 m/s, attack +8.0°, hips 383°/s, the pelvis to 38°, the lead knee straight; the rear hip's sideways stop at 1,018 N·m; 0.3. **Not "the same way", and no run kept the strongest batter on the ground** |
| **the small frame** (1.75 m, 77 kg) | **16.9** (17.2 at the fine step) | −1.1 | 1.3, 1.0 | no | 0.94–0.96 of the average's; his balance 11 cm outside his feet; 0.0 |

- **The start:** moved five ways the bat's speed is +0.2 to +1.1 m/s, within what an unsettled search moves, and two of those swings leave the ground. Whether a start that is not [4]'s would give a different kind of swing is not tested by moving this one.
- **The search's start** is what the answer leans on (the table of starts above): the one swing came from the plan fitted to [4].
- **Unheld joints, reread with the corrected forearm reading** (no run repeated): the rear forearm's turn is 85–130° in every one of these, inside a forearm's 162°. The upper arms' turn about their own length overruns a body's 173° in the main swing (176°) and by more in the swings that leave the ground (the strongest batter 261°, the soft stops 251°, at rest 209–214°).

*What this says, and what it does not* (rewritten after the second review).

- **This is the first swing found on stops that make no energy, and it passes a bat through a strike at 40 mph. It is not yet a swing a body could make.** It keeps every joint within 2° of its range only by leaning on the ranges' ends: nine of them carry 75–906 N·m, where ligaments fail at 13–34 N·m; the rear hip's stop carries 11 times its own rotators' bound, turning the pelvis against the rear thigh through a wall no hip has; and the rear wrist is outside a wrist's range throughout (item 3). PLAN D39, which the owner has ruled and which is not built, is the rule that counts such loads: **it is likely to change this swing a great deal.** The speed is 11 m/s short of the gate's band, and the strongest batter's 21.6, off the ground, is 7 short.
- **Whether it is short for want of muscle is not established.** Nine stronger muscle groups gave 3.6 m/s more (+20%), in a swing that left the ground; no run kept that batter on the ground. The rear hip's rotators, which turn the pelvis, are on a bound that has no source (§4.5).
- **What holds it is not established. The planted feet are the first thing to test.** The pelvis turns 35° and the rear hip is hard against the end of its turn; given 11° more, the hip went to its new end and the pelvis turned 3° more. This body's feet cannot pivot and its knees cannot twist ([note 12](research/body/12-trunk-bending-and-feet.md) D has the measurements), so the hips' turn is most of what the pelvis has; the cheapest test is a rear foot that can pivot or a knee that can twist. Other candidates, none tested: the clock's cap on the whole motion (item 5); the arms tensed from the start; the search staying near the plan it was given; the score's weights [E] for the line and squareness (5.5 of the score's 46.7 is the bat 30° past square); and the score's parked basin (next).
- **The score leaves a parked bat about as good as this swing** (the second review, P2-4). With contact read at the nearest approach, a sweet spot brought to the ball's place and stopped there scores the speed term's 45 and little else; this swing scores 46.7 (speed 27.5, and 19.2 for the miss, the line, squareness, balance and the clock). The cliff is gone, but a basin about as deep sits beside the swing, and the reach stage starts every search near it. No result was seen to use it; it may be part of why three starts of four do not become swings. **Not changed in this job.** The reviewer's remedy, for the next session, to be stated before any run: read the line and squareness at the plane's crossing when there is one, and add to the nearest-approach reading a term for the sweet spot not moving toward the ball along its line.
- **Not run:** the one-coordinate reach tests (the aim's own ball was met, which answers the question they were for); PLAN D39's absorbing joint ends and counted load; any start that is not placed on [4].

### 4.11 The hands, a muscle that holds, a back foot that pivots (session 9)

Session 9 (2026-11-26, PLAN D41: the first of up to three more sessions) builds the three parts of the body that session 8's two reviews found missing or wrong, in the reviews' order, and runs the practice search after each so that what each part does can be seen apart. Written in the order the work was done; each part's design was committed before it was built, and what was expected of each run before it ran. PLAN D39 (a joint's end takes the motion's energy and the search avoids load on it) is not built here; it is the next session's.

**What session 8 taught, held to in every part:** a passive part gets the passivity check (alone, tilted, driven into) in both engines before a search uses it, and every result reports each passive element's net work; every angle read or stopped has a test that carries the joint's other freedoms through their range; a range is tested as an angle a clinician would recognise, each side separately; no cause is drawn from one replay; margins are numbers against the gate's own tests; where a swing came from is said.

#### Part 1: each hand's own grip (the design, written before it was built)

**What the weld was, measured on the placed start** (a geometry probe, no run; the anatomical readings below are new and are tested in the build):

| | Lead (bottom) hand | Rear (top) hand |
| :-- | :-- | :-- |
| wrist tilted toward the thumb (+) or away (−) | −6° | **−66°** (a wrist has 36°, less with the hand closed) |
| wrist bent toward the palm (+) or back (−) | −19° | −3° |
| forearm turned palm-down (+) | **+12°** | **+46°** |
| upper arm turned in (+) about its own length | **+69°** (a shoulder has 69°) | +20° |
| angle between forearm and barrel | 96° | 156° |

- **The forearms' "−168° and −134°" were a zero on the wrong end.** The reading's reference (the elbow's hinge crossed with the forearm) points to the little finger's side with the forearm in neutral, not the thumb's, so neutral read ±180°. Read from the thumb's side the forearms start 12° and 46° turned palm-down, inside a forearm's range.
- **Both wrists on one side of the handle is what two facing palms give; that part of the weld was a real grip.** Call a hand's own directions d (wrist toward knuckles), r (toward the thumb) and p (out of the palm). A right hand has r = d × p and a left hand r = p × d. Both thumbs toward the barrel and the palms facing each other across the handle (p opposite) give the same d: both wrists lie to the same side of the handle. What takes a forearm round to another side is the wrist's bend back or forward, which turns the forearm about the handle: a hitter's top wrist is bent back 47–77° ([note 11](research/body/11-hands-and-joint-ends.md) A5). The second review's reading (its P1-1, "my reading, not a measurement"), that the two forearms come at the handle from opposite sides in the plane of the cock, is not what facing palms give; its finding stands: the rear wrist started and travelled where no wrist goes.
- **What was wrong is three other things.**
  1. *The placement's free choices.* [4] gives both elbows' bends, the bat's direction and the lag. It does not give where each elbow points (each arm can swivel about the line from its shoulder to its wrist) nor how the bat is rolled about its own length. The placement pointed both elbows down and a little out and faced the bat's wrist side at the shoulders, [E] choices made in session 1 before any joint had a range. That is what put the rear forearm 156° from the barrel.
  2. *No anatomical zero.* Session 8 held each wrist's travel from wherever it started, so nothing said that the rear wrist's start was 66° away from the thumb, and the lead upper arm's start on the end of a shoulder's turn was not seen at all.
  3. *The handle was square to the hand.* A closed hand does not hold a handle square to the forearm: the line of the knuckles is oblique, and the barrel's end of a gripped handle tilts away from the wrist. With the handle square, the most a wrist's 32° lets the bat open out from the forearm is 122°; hitters' lead forearm and bat are measured 138–157° apart at contact (Gélinas 1988, [note 11](research/body/11-hands-and-joint-ends.md) A5).

**The design.**

- **Each hand has its own frame on the handle** (`body.hand_frame`): d, r and p as above, fixed in the bat.
  - *The palms face each other:* the rear hand's palm faces along the bat's +y and the lead's along −y; both thumbs toward the barrel. *Each hand's roll about the handle* [E: 0°, the palms squarely opposed; ±20° covers the grips coaches name, from the knocking knuckles in line to the big knuckles in line; no measurement of a hitter's hands on the handle exists, [note 11](research/body/11-hands-and-joint-ends.md) A4] is a named number for each hand, `GRIP_ROLL`.
  - *The handle lies obliquely across the palm:* the barrel's end tilted `GRIP_OBLIQUE` = **15°** away from the wrist, so that with the wrist straight the forearm and the barrel are 105° apart. [E: 10–20°. Kapandji's anatomy text gives 100–110° between a gripped tool's handle and the forearm, from memory and not re-read this session; ergonomics texts treat the power grip as square (0°), which is the range's other end; a "finger" grip across the base of the fingers is more oblique.] What each end of the range does was computed before the choice (a posture search, no run; `sim/target/codex-probes/body3d/s9/explore2.out`, `explore3.out`): at 0° no hitter-like contact posture for the aim's ball (the bat within 10° of square, the elbows inside OpenBiomechanics' bands) keeps every arm joint inside its range; at 30° the lead wrist cannot take [4]'s launch posture (31° toward the thumb of a wrist's 22°); at 15° both exist. **So the 15° is chosen inside its anatomical range by what makes a hitter's two postures possible, and is flagged as that.**
  - *The wrist's centre* is 7 cm from the handle's axis along the hand's own d (`WRIST_TO_GRIP`, [E] as before), so it now sits 1.8 cm toward the knob from its grip point.
- **The wrist is the same two-axis joint, now with anatomical axes and zeros.** Bending toward the palm and back is about the forearm's own radial line (its x, toward the thumb); the tilt toward and away from the thumb is about the hand's p. Zero is the hand in line with the forearm (d along it, palm facing as the forearm's palm side faces). The placement builds each forearm's x toward the thumb; nothing is signed from the start any more (`cock_reference` and `uncock_sign` stay only for the old weld).
- **The ranges**, each side separately, as stops at their anatomical ends (no travel placed from the start, no stop moved to clear the start):

| Joint | Ends | From |
| :-- | :-- | :-- |
| wrist, toward the thumb and away | 22° and **32.4°** | Boone & Azen's 22° and 36° (through Norkin & White, as §4.10), the second cut by 10% for a hand closed on a 25 mm cylinder (Gehrmann 2008: −10, −10, −11% across three closed-hand conditions; toward the thumb not reduced) |
| wrist, toward the palm and back | **63.8°** and 75° | 76° and 75°, the first cut by 16% (Gehrmann, the 25 mm cylinder; −13% on 50 mm, −27% a closed fist: the range's ends) |
| forearm's turn, palm down and palm up | 76.9° and 85.0° | Soucie 2011, as §4.10 |
| upper arm's turn about its own length, in and out | 69° and 104° | Boone & Azen through Norkin & White, as §4.10 |

  - **The wrist's range is kept a box, and is one.** [Note 11](research/body/11-hands-and-joint-ends.md) A1 has the reachable region an ellipse tilted 27° (more tilt away from the thumb is available with the wrist bent toward the palm: 80° of side-to-side travel at the edge of a circle against the box's 54°). An oblique edge needs a stop that reads two angles at once, which the engine does not have; it is left until a run shows a wrist leaning on a corner of the box. The closed hand's loss is in.
  - Stiffness as §4.10 ("small": 2,000 N·m/rad), [E], nothing measured.
- **How each is read, and where each is undefined.**
  - *The wrist's two angles and the forearm's turn* are read on the joints' own axes: each is the angle about one axis of a two-axis joint between a direction fixed in each body, both held square to that axis by the joint itself. They are never undefined and their gradients are the joints' axes. (§4.10 said the forearm's turn was not an angle about one fixed axis. It is, about the forearm's own length, read from the forearm.)
  - *The upper arm's turn* cannot be read that way on a ball joint (any direction fixed in the trunk lies along the arm somewhere a hitter's arm goes). It is read as clinicians define it at the side and at 90° out: the upper arm is carried from hanging to where it is by the shortest turn, and the turn about its own length left over is the reading (a new stop, kind 2; zero is the elbow bending straight forward with the arm hanging, and the forearm level and forward with the arm straight out to the side). Its gradient is (u + v) / (1 + u·v), u the hanging direction and v the arm's. It is undefined only with the arm pointing straight up, 180° from hanging; the run fails loudly within 5° of that. The gradient grows as the arm rises (1.4 times at 90°, 2.9 at 140°).
- **The start is still [4]'s launch posture, flagged; its three free choices get a rule.** The bat's roll and the two elbows' swivels are set so that the arm joint nearest an end of its range (both wrists' two angles, both forearms' turns, both upper arms' turns) is as far from it as can be, each distance counted in half-ranges from the middle [E: a hitter stands where nothing is strained; the rule uses only the ranges above]. Found once at the start's instant and held for the two neighbouring postures the starting velocities are slopes of. No measured wrist angle is given to it; OpenBiomechanics' wrists at foot plant are compared afterwards.
- **The drives are the same 41.** The two "wrist" channels are each wrist's tilt away from the thumb (they were signed from the start by which way the lag fell; the test now asks both).

**Tests, written with the build:** each new reading against a posture built from the clinical definition (a forearm turned 40° palm-down reads 40°, each side; an upper arm turned out 60° reads 60° hanging, raised forward and raised sideways; a wrist tilted 20° toward the thumb closes the forearm-to-barrel angle by 20°); each reading carried through the joint's other freedoms without a jump; the kind 2 stop's gradient against finite differences; the passivity check for kind 2 (the rod on a ball joint, tilted, damped and not) in both engines; each wrist's stops as angles between forearm and barrel inside what a wrist reaches (the expected failure of session 8 removed); the start inside every arm range (by how much is in "As built"; the design first said "by at least 2°", which [4]'s posture does not allow this body's lead shoulder); a hitter's contact posture inside every arm range (built from the aim's ball and OpenBiomechanics' elbows; for checking); the old weld's body unchanged, state for state.

**What I expect** (before the build): the start's worst arm joint about 0.85–0.9 of a half-range from the middle (the posture search's coarse grid: the lead wrist 19° toward the thumb, the rear 25° away, the rear forearm 65° palm-down, the lead upper arm 55° turned in); the rear elbow higher and further back than the old start's. Session 8's plan replayed on the new body will not be a swing (its arms were tuned to the old start and to wrists with no anatomical ends).

**As built** (`body.py`: `hand_frame`, `arm_range`, `arm_angles`, `strain`, `clinical`, `Model(grip="hands")`, `pose`'s rule; `engine.py` and `body3d_probe.rs`: the kind 2 stop; `drive.py`: the wrists' drives signed by anatomy; `test_body3d.py`: `HandsChecks` and four engine tests; the weld stays as `grip="weld"` so that the dated results of §4.4–§4.10 and the probe's check still run, and session 8's saved swing replays on it at 17.8 m/s as before).

- **One thing was added to the design before any test: an elbow keeps clear of the other arm.** The rule as first written put the rear elbow 6 cm from the lead elbow, the two forearms through each other (the body has no flesh). Each elbow's centre now stays `ARM_CLEAR` = 0.10 m from the other arm's bones [E: 0.08–0.12, about an upper arm's thickness]. **The start leans on this number:** the rear arm has two postures that take [4]'s elbow bend, one with the elbow low in front beside the lead elbow (the one the 6 cm came from, the less strained) and one with it up and back; the clearance decides between them.
- **The start, as the rule sets it** (still [4]'s launch posture: the pelvis, trunk, knees, feet, the bat's direction, the lag and both elbows' bends are [4]'s and are met to 0.0°; flagged). The bat is rolled 26° from the weld's; the lead elbow hardly moves (2°); **the rear elbow swivels 94° up and back: it is now 29 cm higher, 15 cm further toward the catcher and 21 cm further from the plate than the weld's, above the shoulder, the arm raised 132° from hanging (83° before).**

| At the start | Lead | Rear | Range | Hitters at foot plant (OpenBiomechanics; for checking) |
| :-- | --: | --: | :-- | :-- |
| wrist toward the thumb (+) | +6° | **−30°** | −32 to +22 | +10 ± 10; +5 ± 12 |
| wrist toward the palm (+), back (−) | −50° | **−74°** | −75 to +64 | −19 ± 18; −52 ± 14 |
| forearm palm-down (+) | +18° | −45° | −85 to +77 | |
| upper arm turned in (+) | **+68°** | −23° | −104 to +69 | |
| upper arm across the chest, from straight out to the side | **134.6°** | 46° | to 135 [E] | |
| forearm to barrel | 96° | 101° | | |

  - **Both wrists start inside the box this body gives a wrist, and the start is strained.** *(Corrected after the review, P1-3: "inside a wrist's range" holds only for the box. The rear wrist starts in the box's corner, bent back 73.8° of 75° and tilted away from the thumb 29.7° of 32.4° together, which a real wrist does not reach: "After the review", below.)* Its tightest margins: the lead arm across the chest 0.4° inside its [E] end and turned in 1.1° inside its end ([4]'s lead arm lies across the chest, as session 8 found; the swivel trades one for the other and cannot clear both); the rear wrist bent back 1.2° inside its end and tilted away from the thumb 2.7° inside. No stop is moved for the wrists, forearms or upper arms, and none carries anything at the start. (The lead arm's sweep stop keeps session 8's rule, 2° beyond the start: it stands for the arm against the chest.)
  - **Against hitters:** the lead wrist's tilt is as measured; both wrists are bent back more than hitters' (1.7 and 1.6 SD); **the rear wrist is tilted 30° away from the thumb where hitters' is 5° toward it (2.9 SD).** That tilt is there in every placement tried (−25 to −31°): it comes from where [4]'s bat lies against where this body's rear forearm can be. Not fitted, and not right.
  - *What the grip's [E]s would change* (the same rule, computed, no run): each hand rolled 20° about the handle so that the wrists bend back less (lead −20°, rear +20°): strain 0.95 and 0.93, the lead wrist bent back 4°, the rear 70°; rolled the other way: the lead arm outside two ranges (1.08). The rolls stay at 0.
- **The readings are a clinician's, each side** (`HandsChecks`, postures built from rotations alone): thumb up and palm to the midline is the forearm's zero with the elbow bent at the side, and 90° palm-down faces the palm at the ground; the upper arm's zero has the forearm forward both with the arm hanging and with it straight out to the side, where 90° turned out points the forearm up; a wrist tilted 20° toward the thumb closes the forearm-to-barrel angle from 105° to 85°. Each reading is carried through its joint's other freedoms (the elbow 10–140°, the arm raised to 150° in four directions, the wrist's other angle through its range) and does not move, to 10⁻⁷°.
- **Each wrist's range is a wrist's:** as angles between forearm and barrel, posed at each end, 83–137° unbent (nearer 90° when bent), inside the 68–157° of session 8's test. `test_the_rear_wrists_range_is_one_a_wrist_reaches` was an expected failure on the weld; `test_each_wrists_range_is_one_a_wrist_reaches` passes for both hands. (Gélinas's 138–157° between bat and lead forearm at contact is at and past this wrist's 137°: the 15° of obliqueness is the low end of what those hitters show.)
- **A hitter's contact posture fits** (`test_a_hitters_contact_posture_lies_inside_every_arm_range`): the sweet spot at the aim's ball, the bat within 10° of square, the elbows inside OpenBiomechanics' bands, the trunk turned to the pitcher and leaning over the plate; at least one such posture has every arm joint more than a tenth of a half-range inside and the elbows clear.
- **The new stop is passive, in both engines.** Alone on a rod: swung 0, 40, 90 and 130° from its rest direction, damped and not, and with a 30 N·m torque driving it in, the rod never has more energy than it started with plus what the drive gave, and the engine's reading equals the quaternion's own swing-twist split. A mutation (the torque about the limb's own axis) gains 24% at 130° and fails the test. On the whole body, eight arm muscles at their bounds drive four of the new stops; each only pushes back, the stops' net work is negative and the energy account closes to 3%. **That run has to be made at 0.25 ms: at 1 ms the account is 450 J out** (stiff stops against full-strength muscles; the search runs at 0.5 ms and every result is rerun at 0.125 ms, as §4.10). `probe3d.py --check` has the hands' body driven into its stops: the two engines agree to 1.2 × 10⁻¹¹.
- **Session 8's plan on the new body** (one replay; it says only that the plan does not carry over, as expected): the bat passes 49 cm above the ball at 16.7 m/s, coming down at 11°.

**The runs after part 1, stated before any** (the same protocol after each part, so that the parts can be compared): the average batter on the hands' body, the aim and the stops as §4.10, the clock's rule unchanged for this part (a tensed arm muscle counts as started: part 2 changes that). **Three starts: S, session 8's practised plan (found from the fit to [4], on the weld); C and D, §4.10's two plans written by hand, which owe [4] nothing.** Each a staged search: 15 iterations of the reach stage, then 30 of the whole aim with contact read at the nearest approach; a 0.5 ms step, each result rerun at 0.125 ms; every result judged by the plane's crossing. All from [4]'s launch posture as placed above (an input, flagged).

*What I expect:* S becomes a swing that meets the ball within 3 cm at 14–18 m/s, no faster than session 8's 17.8: nothing was added that makes speed, and the rear forearm (which turned 106° on the weld) and the rear upper arm (176°) are now held. The arms' new stops carry load (the rear wrist starts 1.2° from one). The hips still turn under 350°/s and the hip and trunk ends still carry hundreds of N·m: part 1 does not touch them. C and D do not become swings (they did not in session 8, and this start is more strained).

**WITHDRAWN as a result about the body (see "A fault in part 1's new stop" below): these first runs were made with a stop whose reading failed in the follow-through, and most of the search's trial runs failed with it. Kept as the record of what was first found and said.** As first written: **The runs after part 1: no start became a swing on the hands' body** (job 368, 9 minutes; the output is now `p1_staged_first_undefined_turn.out`; `sim/target/codex-probes/body3d/s9/p1_staged.out`, `practice3d_staged_main_p1_{S,C,D}.json`; every figure at the plane's crossing, at a 0.5 ms step and again at 0.125 ms; from [4]'s launch posture as placed above, an input, flagged).

| Start | Reach stage (15) | Whole aim (30 asked) | The bat at the plane's crossing | Not a swing because |
| :-- | :-- | :-- | :-- | :-- |
| S, session 8's practised plan (from the fit to [4]) | nearest the ball 1.8, 10.0 and 7.2 cm | **stopped after 9: no step accepted** even with the differences taken four times closer | 281 ms after the start, 1.0 cm up and **12.0 cm along the bat**, at 15.1 m/s (14.9 at the fine step), coming down at 14° | misses by 12 cm; his centre of mass heads 26 cm outside his feet; the first arm drive 281 ms before contact (253 allowed) |
| C, by hand | 6.8, 12.4, 2.0 cm | stopped after 3 | 386 ms, 24.5 cm up, 6.0 cm along, 14.2 m/s of which 1.9 along the ball's line | both feet leave the ground |
| D, by hand, random timings | 0.0, 6.0, 5.8 cm | 30, still falling (3.2 over the last 5) | 271 ms, 0.3 cm up, 9.9 cm along, at 7.6 m/s, coming down at 27° | misses by 10 cm at 17 mph; the front foot at 3.5 times body weight |

- **Against the gate, start S (the nearest thing to a swing):** the bat 15.1 m/s (29–34: 13.9 short); the miss 12.0 cm (1 cm asked); balance 26.2 cm outside (0 asked); the hands 7.5 m/s (10.3–13.0); the attack angle −13.8° (−3 to +11); the bat at 85% of its fastest (98%); the hips 272°/s (555–855) and the trunk 345 (755–1,075); the front foot's greatest load 0.37 of body weight (1.5–2.6); the lead knee 65° bent at contact (2–26); the arms' clock 28 ms over. It passes nothing but "no joint more than 2° past its range" (2.0° at the most) and the passivity check.
- **Passive elements' net work to contact** (stops; ground): S −24 J (−39 by 50 ms after) and −16 J; D −72 J (−122) and −32 J; C's are in its file; none positive. Damping and the hold-still torques: −160 J (S).
- **Joint ends' largest loads, S** (N·m): the rear hip bent back 456, the trunk's twist 437, the lead hip sideways 324, the lead wrist's bend 191, the lead hip's turn 157, the rear elbow 135, the lead forearm's turn 87, the rear wrist's tilt 86, the lead wrist's tilt 75, the lead upper arm's turn 70, the rear forearm's turn 43. **The rear hip's turning end, 906 N·m in session 8's swing, carries nothing here** (a different motion, not a remedy). Five of the new arm stops carry load; ligaments fail at 13–34.
- **The arms in S:** the lead forearm goes to its palm-down end (77°) and stays there; the rear forearm to its palm-up end (85°); the lead upper arm sits on its turned-in end all swing (it started 1° from it); both wrists end on their away-from-the-thumb ends (33° and 32°). The lead forearm and barrel are 115° apart at contact (hitters 138–157°).
- **Feet:** C leaves the ground; S and D do not (never under 0.51 and 0.30 of body weight between the feet).
- **Against what I expected:** wrong on the main point. I expected S to become a swing within 3 cm at 14–18 m/s; its speed is in that band but it is 12 cm from the ball and the search stopped. Right that C and D would not become swings, that the new arm stops would carry load, and that the hips would stay under 350°/s.
- **What this shows, and what it does not.** With each hand given a wrist's, a forearm's and an upper arm's real ranges, and the start that [4]'s posture then allows (the lead shoulder and the rear wrist each within about 1° of an end), this search under this protocol found no swing from three starts. It does not show that the body cannot swing with a hitter's hands: S's search stopped on refused steps, not on a settled score, and one follow-up is stated next. **Session 8's 17.8 m/s swing was made with a rear wrist outside any wrist's range and with forearms and upper arms nothing held; held, that swing is gone, and nothing has replaced it yet.**
- *A follow-up, stated before it ran:* S's result carried on 30 more iterations (the search starts again with its first difference size; job queued with part 2's runs). I expect it to stop again within 10 iterations no nearer than 8 cm: the refused steps look like the stops' roughness near a start that sits on two ends, not the size of the differences.

#### Part 2: a muscle tensed before the pitch only holds (the design, written before it was built)

**What is wrong now** (session 8's second review, P2-2; §4.10 the fix, item 5). A plan gives each muscle a level it is held at before it rises. In the engine that level is net torque on its joint from the first instant. So a tensed arm muscle is an arm drive that has started, the clock ("no arm drive before he could have decided") reads it at the run's first instant, and the rule becomes a cap on the whole motion, 253 ms from the starting posture. A hitter's lead shoulder is recorded working at 76–101% before the swing, so a hitter would fail it.

**The dead end, and why** (`hold="resolve"`, session 8; kept in the code as a record). It re-solved the constant hold-still torques so that they cancelled each plan's starting levels. The body then starts still, but the cancelling torque is constant and stays on all swing: when the muscle fires, that share of its strength is still being cancelled (the fit to [4]'s plan replayed that way: the bat at 2.4 m/s).

**The design: what a tensed muscle is held against lets go when it fires.**

- A muscle tensed before its onset is held by its opposite number. *(Corrected after the review, P2-5: this is not what co-contraction does mechanically. The opposite number here cancels the muscle exactly, which leaves out the stiffness a co-contracted pair gives a joint [E]: "After the review", below.)* **Until its onset the pair puts no net torque on the joint, whatever the joint does:** the opposing hold is equal to the muscle's own torque at every instant (through its strength-at-speed curve and its tendon spring, as the muscle itself is). So a tensed muscle moves nothing and does no work before it fires: there is nothing in it that could add energy.
- **At its onset the hold lets go:** it falls away as a muscle switching off does, with the engine's deactivation time constant (50 ms; `TA_REL` [E: 15–50 ms, the activation and deactivation constants; no measurement of an antagonist's release before a swing is in the notes]). The net torque on the joint is the muscle's own torque times 1 − (its level beforehand × the hold's remaining share) / (its activation now): zero at the onset, rising without a step as the activation rises and the hold decays, and the muscle's whole torque once the hold is gone.
- What being tensed still buys: the muscle's activation starts from its held level instead of from zero and its tendon spring is already stretched, so its torque arrives sooner than a slack muscle's; against that, the hold takes 50 ms to clear. Which wins is a result.
- **Every muscle, not only the arms'.** A leg or trunk muscle tensed beforehand is held the same way until its own onset (it may fire at the first instant; its torque then rises from zero instead of stepping in).
- **Effort (PLAN D36):** a tensed muscle is working, and so is what holds it. The effort of a channel is counted as before, plus the hold's share (its level beforehand × the hold's remaining share, squared) while the hold lasts.
- **The clock then means what §4.10 says:** no arm muscle *fires* (its onset) more than the allowed time before contact. A tensed arm muscle no longer counts. The allowed time is the pitch's 0.403 s less how long he watches before he can decide, less 0.05 s for the arms to answer. **How long he watches is not put to the owner yet; both are run:** 0.10 s (as §4.10; the arms may fire 253 ms before contact) and 0.20 s (the game's own figure for its batter, agent-models §3.4; 153 ms).
- The constant hold-still torques (`solve_tone`) are unchanged: they stand for the postural drives of the launch posture, [E], and do not let go.

**Tests, written with the build:** a body with every muscle tensed at 0.7 of its bound, onsets late, moves state for state as the body with none tensed, while it moves under gravity and its hold-still torques, to the first onset (not only at the first instant); the drives' booked work is zero to then; the net torque at an onset is zero and rises without a step to the muscle's whole torque; the energy account closes with held muscles firing; a mutation (the hold never lets go) fails the "whole torque" test, which is the dead end above; the two engines agree with held muscles (`probe3d.py --check`).

**What I expect of the runs after part 2** (the same three starts and iterations as after part 1; two runs, one for each viewing time; each start's part 1 result is not reused, so that the runs differ from part 1's only by the hold and the clock):

- *Watching 0.10 s (253 ms):* the swing is no longer capped at 253 ms from the starting posture, so start to contact gets longer than part 1's (which I expect on the cap) by 10–60 ms, with the legs and trunk leading; the bat 0–2 m/s faster than part 1's; "legs, then trunk, then arms" met or nearer; the lead shoulder's sweep muscle may be tensed (it is reported).
- *Watching 0.20 s (153 ms):* session 8's hands first moved 204 ms before contact, so this binds: the bat 1–3 m/s slower than at 0.10 s, or the same speed with more of the turn made before the arms fire.
- The joint ends' loads and the hips' turning rate do not change in kind: part 2 does not touch them.

**As built** (`engine.py`: `actuator(held=...)`, `hold_share`, `TA_REL`; `body3d_probe.rs` the same; `drive.add_drives(held=...)`; `practice3d.py`: `hold="let_go"`, `t_view`, the clock reading onsets, the hold's share of effort; `test_body3d.py`: `HeldChecks`).

- **A held muscle gives nothing before its onset, whatever the body does.** The hands' body with all 41 muscles at their bounds, tensed at 0.7 and held, moves entry for entry (to 10⁻¹²) as the body with no muscles for the 55 ms before the first onset, while gravity and the hold-still torques move it; the drives have booked exactly 0 J. Tensed and not held, the same body is elsewhere within 5 ms. (The comparison is with a body of no muscles: a muscle at its bound with activation 0 is not quite nothing, because its tendon spring is worked as if a thousandth active, up to 0.15% of its strength. That was there before and is not changed.)
- **The hold lets go without a step:** on a rod, the torque at the onset is zero, never changes by more than a twentieth of the muscle's strength in a millisecond, and a quarter of a second later is the un-held muscle's own (strength × activation × its speed curve, computed in the test). **The mutation in which the hold never lets go fails that test: it is session 8's dead end.**
- The energy account closes with held muscles firing (3% of the gross work, at a 0.25 ms step). The two engines agree on the hands' body with every muscle tensed and held, firing at staggered onsets: 2.9 × 10⁻¹³.
- Nothing passive was added: before its onset the pair's net torque is zero, so it can do no work; after it, the torque is the muscle's own times a share between 0 and 1.

#### Part 3: a back foot that can pivot (the design, written before it was built)

**What the feet did in session 8's swing, read before designing** (one replay in the Python engine, `sim/target/codex-probes/body3d/s9/feet_s8.py`; a reading of that one swing, and no more is drawn from it than it shows):

- The rear foot stayed flat on its four points to contact. Its heel never left the ground.
- **It did turn on the ground, the wrong way:** 30° with the toe going back toward the catcher, half of it in the 20 ms when the foot carried 0.03 of body weight, while the pelvis turned 37° toward the pitcher. The rear hip's own turn went from 26° turned out to its end at 34° turned in: 60° of turn at the hip for 37° of pelvis.
- The ground's twisting moment on the loaded, flat rear foot was 30–75 N·m at 0.5–1.4 times body weight.

So this body's rear foot was never "planted and unable to pivot" (§4.10 and both reviews said it was; none of us had looked). It slides round against about what a cleat gives. In that swing it was turned backward: the rear hip's rotators, at full strength all swing, turn the pelvis one way and the leg the other, and the leg went. **What it cannot do is what a hitter's does: come up over its toes, so that the leg can turn with the pelvis.** And two things under it have no source or no end:

1. *No toes.* The foot is one rigid plate with contact points at the heel and at the ball. Raised on the ball it stands on two points 8 cm apart and its front quarter passes through the ground.
2. *The ankle has no end on either axis* (§4.10 left the bend unheld because the measured range was a straight, unloaded knee's; the roll never had a stop, nor a drive).
3. *The ground's grip, 0.8, has no source* (§4.1, §5.1).

**The design, from [note 12](research/body/12-trunk-bending-and-feet.md) D.**

- **Toes: designed, then withdrawn before the build, for two reasons found while sizing them.** As first written the rear plate was to be cut across the ball of the foot into a rear foot and a toe piece on a free hinge (toes bending back to 58°, the heel-rise test's figure, Nawoczenski 1999, AB).
  1. *A toe piece on a free joint carries nothing.* A load on the toes' tips needs a moment at the toes' joint to hold it, which is the toe flexors' (60 N·m in running and 112 in sprinting, [note 12](research/body/12-trunk-bending-and-feet.md) D2); with no muscle there the whole load stays on the ball's two points. **That is exactly what the rigid plate already does when it comes up on its ball:** it turns over the ball's line as a foot does over toes that take no load. The plate could always "come up onto its toe"; in session 8's swing it did not.
  2. *A 0.19 kg toe piece on the engine's ground cannot be integrated at the search's step* (four contact points' damping on that mass is past the explicit integrator's limit at 0.5 ms).

  So the foot stays one plate. **What is left out is the toe flexors' moment** [E]: it would let the toes' tips carry load, 6.5 cm of support ahead of the ball. Nothing holds the heel's rise to the toes' 58°; the heel's height is reported.
- **Both feet are treated alike** (`Model(feet="cleats")`); nothing is built for the front foot that the rear does not have. No source measures a hitter's front foot coming up or turning ([note 12](research/body/12-trunk-bending-and-feet.md) D4: 6° between forefoot and heel contact), and nothing here makes it.
- **Both ankles get their ends, as measured with weight on:** the shin forward over the foot 39° (Baumbach 2014, 20 adults, the knee bent 20° or more; **25° with the knee straight, which the stop cannot know: [E]**), the foot pointed down 50° (Hallaceli 2014, passive, 987 young adults); the sole rolled inward 34° and outward 19° (the same, men). Stiffness as the other large joints' ([E], §4.10).
- **The ground's grip is the measured one for a baseball shoe on basepath dirt:** `MU` = **1.15** for every contact point (Brosnan, McNitt & Serensits 2009: static 1.04–1.27, dynamic 1.02–1.20, FT; it replaces 0.8, which had no source). This is both feet's, so part 3's runs change the front foot's grip too; said in the results.
- **The foot's resistance to turning is not a separate number.** It comes from where the sole's points are and the grip, and is checked against the same paper's rig: a baseball shoe's forefoot under 1,190 N turned through 40° gave 68–100 N·m. The ball's two points are moved from 4 cm to **5 cm** either side of the foot's line (`BALL_HALF_W` [E: 4.5–5.5; a forefoot is about 10 cm wide and its studs sit at its edges]) and tested in that rig.
- **The knee's twist is not built.** [Note 12](research/body/12-trunk-bending-and-feet.md) D3: with weight on, the knee's turn about the shin falls by 58–80% and is 5–9° in loaded movements. Whether a run shows a need for it is said after the runs.
- The start is unchanged: the rear foot flat, its toes in line.

**Tests, written with the build:** the forefoot alone under 1,190 N, kept level and turned 40°: its peak twisting moment against Brosnan's 68–100 N·m and against Coulomb's law worked by hand, the ground's work on it never positive (the contact law is the old one; this is its check about the vertical), and what the old grip and stud spacing gave; each ankle's two readings as a clinician defines them, each side, each unmoved by the other; the start inside all four ankle ends with no stop moved and none engaged; the two engines agree on the body with cleats; the plate's body unchanged.

**What I expect of the runs after part 3** (the same three starts and iterations; the hold of part 2, watching 0.10 s; from [4]'s launch posture):

- **I do not expect the search to find the heel coming up from these starts.** It moves each number 2% at a time from plans that keep the foot flat, and nothing in the aim asks for it. If the heel stays down in all three, that is the result, and what would test the foot is a start with the heel already rising or a term for it, the next session's. *(Written before part 1's and part 2's runs were read. Since no start became a swing after part 1, the comparisons below are against part 2's results from the same start, whatever those are.)*
- The firmer grip (1.15 for 0.8) should hold the rear foot against the rotators' reaction: the backward turn under 15° (it was 30°), the pelvis turning 40–50° (35°), the hips 350–450°/s (331), still under hitters' 555–855.
- The rear hip's turning end still carries hundreds of N·m. The bat 0–2 m/s faster than part 2's.

**As built** (`body.py`: `Model(feet="cleats")`, `MU_CLEAT`, `BALL_HALF_W`, the ankles' two stops in `range_specs`; `practice3d.py`: `--feet cleats`, `--mu`, and a line in every report for what each foot did; `test_body3d.py`: `FeetChecks`; nothing in either engine changed).

- **The stud spacing is set inside its range so that the forefoot gives the low end of Brosnan's measurement** *(reworded after the review, P2-3; first written as "the forefoot in Brosnan's rig gives 68.4 N·m", as if that were agreement with the measurement)*: 68.4 N·m in that rig (1,190 N, kept level, turned 40°; the measured 68.5–100.1), which is Coulomb's law for two points (1.15 × 1,190 N × 0.05 m). Across the spacing's own [E] (4.5–5.5 cm) the same law gives 61.6–75.3 N·m, and 5 cm is the value that lands on the measurement's low end. **What the test checks independently is the contact law about the vertical:** the engine gives Coulomb's value, and the ground took 46 J in that turn and gave none. The grip, 1.15, is inside both of the paper's measured ranges. **The plate of sessions 1–8 gives 38.1 N·m in the same rig, 56% of the least measured:** the old forefoot was about half as hard to turn as a cleat on dirt.
- **The ankles' readings are a clinician's, each side:** a foot pointed down 30° reads +30, the shin 25° forward over the foot −25; a sole rolled 20° toward the midline reads 20° toward its inversion end and its normal does face the midline; neither moves with the other (10⁻⁷°). The ends are 39° and 50°, 34.6° in and 19.1° out, no stop moved.
- **The start is inside all four:** the lead foot, on its ball at toe-touch, is pointed down 25.5° and rolled in 30.5° (4° inside); the rear shin is 20.3° forward and the rear sole rolled in 16.5°. No stop carries anything at the start.
- The two engines' agreement on the body with cleats and the whole test run are in "The checks" below.

#### A fault in part 1's new stop, found when part 2's first runs stopped the way part 1's had

**What happened.** Part 2's first runs (job 371; `p2_staged_first_undefined_turn.out`) ended like part 1's: the whole-aim search stopped after 3–9 iterations with no step accepted, from every start but one. That is not how a search settles, so I probed it before reading anything into it (`sim/target/codex-probes/body3d/s9/stuck.py`, at start S's two results):

| At start S's result | Part 1's | Part 2's (watching 0.10 s) |
| :-- | --: | --: |
| of the search's 205 one-number differences, how many fail to run | **124** | **59** |
| of 60 plans with every number moved at random by up to 2% | 24 fail | 21 fail |
| the lowest score among those that ran (the result's own score) | 409.6 (481.0) | 311.1 (327.3) |
| every failure's message | "a limb's own axis is turned right over" | the same |
| the lead upper arm's greatest angle from hanging in that motion | 175° | 175° |

**The cause is mine, and it is session 8's second lesson again.** The upper arm's turn (the kind 2 stop of part 1) has no value with the arm pointing straight overhead, and the engine fails the run within 5° of that, loudly, as designed. I wrote in part 1's design that this posture "is never near in a swing" (§4.10 had said the same of the arm's raise) and did not test it. In these motions the lead arm goes overhead in the follow-through, after contact, where nothing in the aim looks; the run fails; a failed run is a refused step; and with 59–124 of 205 differences failing the search has no slope to follow. Lower scores were there to be found beside both results. **So "no start became a swing" after part 1 was the search stopped by a reading fault, not a finding about a body with a hitter's hands, and my reading of it above ("held, that swing is gone") went further than a stopped search can carry.** The follow-up's result (stopped again after 3 iterations, as I had "expected") was the same fault and tested nothing.

**The fix is a shoulder's own end, which the body lacked:** the arm is raised no further than **168.8°** from hanging (Soucie 2011, the forward raise, men of 20–44, as §4.10's table; 184° sideways in Boone & Azen's sample is the range's other end [E]). A kind 1 stop on the upper arm's angle out of the trunk's level plane, at the large joints' stiffness; it reads 90° less the raise and is never asked for its gradient with the arm hanging. It keeps the turn's reading 6° clear of where the run fails; to get there a swing would have to push 2,200 N·m into the raise's end, which is reported like every other end's load.

- *Tests:* the raise reads 0, 90, 150 and 165° for arms so raised in four directions and however turned (`HandsChecks`); an upper arm alone, driven up by a 5 N·m muscle from 30°, is stopped within 3° of 168.8°, the run never fails and the two stops' work is not positive; **without the raise's stop the same run fails, which is the fault** (`RaiseChecks`).
- *At start S's part 1 result, with the stop in:* 0 of 205 differences and 0 of 60 random plans fail; the motion to contact is unchanged (the same score, 481.04; the arm now stops at 169°).
- *What I should have done:* tested "no swing posture is near it" on a swing before saying it. The check that would have caught it at once is now made after every search: the report counts how many of the plan's one-number differences fail to run (below).

**The runs again, stated before any** (one job, the four bodies in the order they were built, so that each differs from the one before by one part): (1) part 1's body, the clock counting a tensed arm muscle; (2) part 2's hold, watching 0.10 s; (3) the same watching 0.20 s; (4) part 3's feet, with the hold, watching 0.10 s. Each from the three starts S, C and D, 15 iterations of the reach stage and 30 of the whole aim, as stated for part 1. Each result is followed by the count of failed differences at its end.

*What I expect now:* what I first wrote for each part stands as the expectation (part 1: S a swing within 3 cm at 14–18 m/s; C and D not; part 2 and part 3 as written under each), with one change: **I no longer expect the searches to stop on refused steps; if one still does with no failed differences, the roughness is the body's (stiff ends near a strained start) and is said so.**

**The reruns stopped too, with no failed runs; a second follow-up, stated before it ran** (the reruns' table is below, written after this). With the raise's end in, no result has a failed difference (0 of 205 at all twelve), yet the whole-aim search still stopped after 3–16 iterations from start S on every body, with no damped step accepted even with the differences four times closer. Lower scores lie beside those points (60 random plans within 2% of part 1's S result: the lowest 409.6 against 481.0). So the search's step, not a failed run, is what stops: the slope it builds from one-number differences does not predict the score across stiff joint ends and a foot that sticks and slips.

- *The change, to the search and not to the body or the aim* (`practice3d.py`, `--fallback`; off, it is session 8's search): when no damped step is accepted, the 205 differences are plans already run, each one number away; if the best of them scores lower it is taken, and the search goes on from there.
- *The run:* start S's rerun result on each of the four bodies, carried on 40 iterations with the fallback, contact read at the nearest approach as before. Each result judged by the plane's crossing, at both steps, with its failed differences counted.
- *What I expect:* every score falls. On part 1's body, part 2's at 0.10 s and part 3's, S comes within 3 cm of the ball at 14–19 m/s; whether any is within the gate's 1 cm I do not know. At 0.20 s the clock's term (most of that score) falls and the bat slows below 20.9 m/s. None is settled after 40 iterations of single steps; differences between the bodies under about 1 m/s will mean nothing.

#### The runs after each part: the results (jobs 374 and 375; the first runs of jobs 368 and 371 are withdrawn above)

Outputs under `sim/target/codex-probes/body3d/`: `s9/rerun_staged.out`, `practice3d_staged_{main,view_010,view_020,cleats}_r_{S,C,D}.json` (the reruns), `s9/fb_*.out` and `practice3d_r_*_S_fb.json` (start S carried on with the fallback); the two ends kept in the repository are `analysis/research/body/practice3d_s9_hands_S.json` (part 1's body) and `practice3d_s9_cleats_S.json` (part 3's). Every figure is at the plane's crossing at a 0.5 ms step, the bat's speed again at 0.125 ms. **Every one starts from [4]'s launch posture as part 1 placed it (an input, flagged), and start S is session 8's practised plan, which came from the fit to [4].** No result has a failed difference (0 of 205, all sixteen).

**No start became a swing on any of the four bodies.** The nearest, start S, passes 8–21 cm from the ball.

| Start S, carried on with the fallback | Part 1: the hands | Part 2: the hold, watching 0.10 s | Part 2: watching 0.20 s | Part 3: the feet on dirt (hold, 0.10 s) |
| :-- | --: | --: | --: | --: |
| the rerun stopped after (score) | 11 iterations (149.5) | 16 (297.0) | 3 (2,377) | 3 (178.1) |
| with the fallback: iterations, of them single steps; score | 23, 10; 139.6 | 25, 25; 285.4 | 28, 1; 2,232 | 40, 37; 153.4 |
| score still falling per 5 iterations | 1.1 | 0.0 | 0.7 | 0.6 |
| start to contact | 263 ms | 268 | 240 | 263 |
| **the miss: up; along the bat** (gate: 1 cm) | 3.4; **7.6 cm** | 3.2; **13.2** | 7.6; **20.6** | 3.1; **7.9** |
| **sweet spot at contact** (29–34 m/s) | **15.3** (15.3 fine) | **16.9** (16.9) | **21.8** (21.8) | **19.0** (19.1) |
| attack angle (−3 to +11°) | −17.0° | −26.3° | −17.3° | −17.7° |
| bat's speed at contact of its fastest before (98%) | 81% | 79% | 92% | 89% |
| hands' fastest (10.3–13.0 m/s) | 8.7 | 9.9 | 10.3 | 9.7 |
| hips'; trunk's fastest turning (555–855; 755–1,075°/s) | 284; 405 | 316; 506 | 263; 510 | 353; 511 |
| front foot's greatest load (1.5–2.6 × body weight) | 0.58 | 1.06 | 1.47 | 1.13 |
| lead knee at contact (2–26°) | 50° | 1° | 58° | 54° |
| balance: heading outside the feet's area (0) | 3.2 cm | 9.5 | 11.7 | 5.1 |
| least the feet carry (0.10 of body weight) | 0.36 | 0.41 | 0.27 | 0.22 |
| the first arm muscle to fire, before contact (allowed) | 263 ms (253; a tensed one counts) | 268 (253) | 240 (**153**) | 263 (253) |
| order of first rises (legs, trunk, arms) | not met | not met | not met | not met |
| stops' net work to contact (to +50 ms); the ground's | −48 (−92); −30 J | −72 (−139); −28 | −59 (−303); −49 | −35 (−166); −16 |
| both feet left the ground | no | no | no | no |
| the rear heel's greatest height; the rear foot's turn on the ground (+ toward the pitcher); the pelvis's turn | 0.0 cm; −28°; +34° | 0.0; −8°; +46° | 0.0; −40°; +20° | 0.0; **−4°**; **+58°** |

- **The other two starts** (the reruns; they owe [4] nothing): on part 1's body C stopped after 3 iterations 25 cm from the ball with both feet off the ground, and D passed 10 cm from it at 7.6 m/s; with the hold neither reached the ball's plane at either viewing time; **on the feet of part 3 both cross it**, 2–6 cm from the ball at 9.0 and 8.5 m/s, coming down at 54–58°. None is a swing.
- **The arms at contact, the same on all four bodies:** the lead forearm on its palm-down end (78–79° of 77), the rear forearm on its palm-up end (78–86° of 85), the lead upper arm on its turned-in end (70–71° of 69, where it started 1° short), the lead wrist 2–5° past its away-from-the-thumb end (34–37° of 32.4) and bent 38–51° toward the palm (hitters 8 ± 10), the rear wrist 20–31° away from the thumb and 17–25° toward the palm (hitters bent back 47 ± 17). The lead forearm and barrel are 118–128° apart (hitters 138–157°). The rear elbow is nearly straight (2–11°; hitters 79 ± 13); the lead 51–64° (53 ± 15). **These are not a hitter's arms at contact; they are arms jammed against four or five ends.** The lead wrist is 1.6–4.6° past its end at contact, so "no joint more than 2° past its range" fails or nearly fails on every body.
- **Joint ends' largest loads** (N·m, to 50 ms after contact; ligaments fail at 13–34): part 1: the trunk's side bend 787, the rear hip back 539, the lead hip sideways 485, the rear elbow 469, the trunk's twist 334, the lead hip's turn 307, the rear hip's turn 187. Part 2 at 0.10 s: the lead knee straight 1,051, the trunk's side bend 624, the lead hip's turn 560. Part 3: **the lead arm's new end of raise 899** (the follow-through runs into it), the lead hip sideways 517, the rear hip back 495, the rear hip's turn 447, the lead hip's turn 438.
- **Passive elements:** every stop group's and the ground's net work is negative in all sixteen results. Before its onset a held muscle's pair does none by construction.

**What each part changed, as far as searches that have not settled can say** (start S with the fallback; nothing under about 1 m/s means anything, and every one of these is a motion that misses, so a "faster" bat is a faster miss):

| | The bat | The miss along the bat | Other |
| :-- | :-- | :-- | :-- |
| the weld to **part 1** (session 8's swing: 17.8 m/s within 2 cm) | 15.3 | 7.6 cm | the swing that met the ball is gone: the rear wrist's old travel, the forearms' and upper arms' unheld turn and the old launch posture are what it was made with |
| part 1 to **part 2**, watching 0.10 s | 16.9 (+1.6) | 13.2 cm | contact 5 ms later; the hips and trunk 30–100°/s faster, the front foot to 1.06 of body weight and the front knee straight; the plan's arm muscles still fire at the first instant (its own onsets), 15 ms before the clock allows |
| watching **0.20 s** instead | 21.8 | 20.6 cm | not a test of the viewing time: the arms fire 87 ms too early and that is 78% of the score; the search took one single step in 28 iterations. *(The review, P1-1: the old clock term is why. With eight arm onsets tied at the first instant the search read every one of them as uphill.)* |
| part 2 to **part 3** | 19.0 (+2.1) | 7.9 cm | the rear foot's backward slide 8° to 4°, the pelvis's turn 46° to 58°, the hips 316 to 353°/s; C and D reach the ball's plane, which they did not before |

**Against what I expected.**

- *Part 1:* wrong. I expected S to become a swing within 3 cm at 14–18 m/s. The speed is in that band; the bat passes 7.6 cm from the ball. C and D not becoming swings, the new arm ends carrying load and the hips under 350°/s: as expected.
- *Part 2:* the bat 0–2 m/s faster at 0.10 s: yes (+1.6, within what means nothing). Start to contact 10–60 ms longer: no (5 ms). "Legs, trunk, arms": not met; the plan's arms still fire at the start. At 0.20 s I expected a slower bat or more turn before the arms; neither can be read from a search that did not move.
- *Part 3:* the heel staying down: yes, on every body and every start but the two that flew. The backward turn under 15°: yes (4°). The pelvis 40–50°: more (58°). The hips 350–450°/s: just (353). The bat 0–2 m/s faster: +2.1.
- *The follow-ups:* I expected the fallback to bring S within 3 cm on three bodies. It did not: the scores fell 4–14% and the misses hardly moved.

**What this session shows, and what it does not.**

- **Shown: the body now has a hitter's hands, a muscle that only holds, and feet that grip like cleats, each tested as anatomy or against a measurement, and nothing passive in any of them adds energy.**
- **Shown: session 8's exact motion cannot be made with real ranges** *(narrowed after the review, P2-4; first written as "the one swing phase 1 had does not exist")*. Read with session 9's readings (the reviewer's `s8arms.py`), that motion's lead upper arm turns in to 148° (the end is 69°), its rear upper arm turns out to 129° (104°) and its lead forearm turns palm-down to 80° (77°); the rear forearm stays inside its range, and the rear wrist started outside any wrist's. So the upper arms overrun by 25–79° and the lead forearm by 3°. **That no 40 mph swing exists on this body is not shown:** the search that looked for one had the two faults below.
- **Shown: this search, from these three starts and from [4]'s launch posture, finds no swing on the new body.** The nearest motions pass 8–21 cm from the ball at 15–22 m/s with the arms against four or five ends.
- **Not shown: that the body cannot swing.** Three things stand between these runs and that statement, and none was tested apart:
  1. *The search stalls, and why is now shown (the review, P1-1 and P1-2; "After the review", below).* Its clock term could not see the arm onsets moving together, and its score rises within a hundredth of every damped step it would try, with slopes that disagree forward and backward; part of that roughness is the 0.5 ms step. *(First written: "a local search on a score made rough by stiff ends and a foot that sticks and slips", which was a guess.)* These are faults of the search, not findings about the body.
  2. *The start is at the ends of joints (more of them than first listed: the lead hip's sideways end is 2.8° away and is the first to carry load; the review, P2-1).* [4]'s launch posture puts this body's lead arm 0.4° from the end of its sweep across the chest and 1.1° from the end of its turn in, and the rear wrist 1.2° from bent right back. Every motion from it begins by leaning on an end.
  3. *The plans were made for another body.* S was tuned on the weld; C and D have never been swings on any body.
- **Not shown: anything about the viewing time.** At 0.20 s the search did not move from a plan whose arms fire at the first instant, and the old clock term is why (the review, P1-1). **The two columns of part 2 show nothing about how long he watches until both are run again on the new clock term; they are not rerun in this job.**
- **The knee's twist: no run shows a need for it.** On cleats the rear foot holds (4° of slide) and the pelvis turns 58°; the rear heel never comes up, so the leg is never asked to turn over a pivoting foot. It stays out.
- **The parked basin's remedy was not done** (the job allowed it only with time left), and **PLAN D39 is not built**.

**The checks, after touching every changed source** (jobs 376 and 377; outputs `sim/target/codex-probes/body3d/s9/final_*.out`):

- `python3 -m unittest analysis/oracle/test_body3d.py`: 60 tests, OK (42 before the session; the 18 new are `HandsChecks` 8, `HeldChecks` 5, `FeetChecks` 2, `RaiseChecks` 1 and three for the kind 2 stop, less the two wrist-reach tests folded into one; the expected failure is gone). `test_oracle.py`: 112, OK (8 expected failures, as before).
- `probe3d.py --check`: PASS, worst 3.6 × 10⁻¹¹ against 10⁻⁹ over eight cases, three of them new (the hands' body driven into its stops; every muscle tensed and held; cleats). **The cleats case is 100 ms long, not 200:** with every leg muscle at its bound the body is on two of its sole's points by 80 ms, and from 100 ms that motion multiplies rounding differences (first written as "about twenty times every 10 ms", which holds for 100–120 ms only; over 100–200 ms it averages about three times per 10 ms in bursts of 10–40: the review, P3-1) (the two engines, and the probe's own two elimination orders, agree to 10⁻¹³ until then and are 2 × 10⁻⁷ and 7 × 10⁻⁷ apart at 200 ms). The first run of the check failed on it; it is the motion's sensitivity, not a difference of laws, and part 3 changed no law in either engine.
- `bin/check-docs.py`: clean. Session 8's saved swing replays on the weld at 17.8 m/s, as before; part 3's end result replays at 19.0 m/s, 3.1 and 7.9 cm from the ball.

#### After the independent review of session 9 (2026-11-26): two fixes to the search, stated before they were built

[The review](reviews/2026-11-26-opus-body-3d-s9-review.md) found nothing at fault in what parts 1–3 built and two defects in the search, one of them in its score. Both are fixed here with tests. **No search is rerun in this job:** the results above stand as what the old score and the old run rule found, and what the review shows about them is written into them below.

**1. The clock charges each arm muscle its own overrun (review P1-1).**

- *The old form:* one term, the overrun of the earliest arm onset alone: max(0, (contact − allowed time) − the earliest onset), per 2 ms.
- *What was wrong with it:* with two or more arm muscles tied at the earliest onset, moving any one of them later leaves the earliest where it was; all the search sees is contact arriving a little later, so the term rises. Every plan of sessions 8 and 9 starts at such a tie (the box's lower bound for an onset is the run's first instant).
- *The new form:* one term for each arm muscle that is used (strength over 1% of its bound): its own overrun, max(0, (contact − allowed time) − its onset), per 2 ms, **weighted by its share of the arms' asked strength** (its strength over its bound, divided by the sum of that over the arm muscles used; the weights sum to 1).
- *Why that weight:* with every arm muscle tied at one onset the sum of squares is exactly the old term, so the clock's size against the rest of the score is unchanged; a muscle asked for 2% of its strength that fires early is charged 2% of what a full-strength one is, in place of the old cliff at 1%; and every onset has its own slope, so a search can see both one muscle moving and all moving together.
- *What it leaves:* the term is still zero for a plan whose arm muscles all fire inside the allowed time, so the rule itself is unchanged: no arm muscle fires before he could have decided.
- The old form is kept as `clock="earliest"` for the test that shows it fails.

**2. A run is not failed by what happens after the last instant the score reads (review P2-2).**

- *What was wrong:* a run that fails anywhere in its 450 ms is a refused step, though the score reads nothing later than 50 ms after contact. The raise's end keeps the upper arm's turn defined up to about 2,200 N·m; a follow-through at the gate's bat speed is estimated to put about that much into it (the review's estimate, from 899 N·m at 19 m/s). A swing nearing the gate would become refused steps again, from a part of the run nobody reads.
- *The choice:* the run is not ended at contact (the probe does not know where the ball is); instead **a failure later than 50 ms after the plane's crossing does not fail the run.** The compiled probe now hands back every record it made before a failure. If the sweet spot crossed the ball's plane at least 50 ms before the failure, those records hold everything the score and the report read, and they are used; the result says that the run failed and when. A failure before that is a failed run, as before, and is still counted among the failed differences.
- **The raise's end stays** (168.8°, a shoulder's own end). One value is used in every direction; raised sideways a shoulder goes to 184° (Boone & Azen), so sideways this end is 15° tighter than anatomy [E].

**The two fixes as built** (`practice3d.py`: `clock_residuals`, `Ctx.arm_fired`, `Ctx.clock`, `Ctx.run`; `probe3d.py`: `RunFailed`; `body3d_probe.rs`: its records written a line at a time; `test_body3d.py`: `SearchChecks`).

- **The clock.** The test works the term by hand for five arm muscles tied at the first instant, 30 ms over, and a sixth inside the clock: moving all five 10 ms later lowers the charge to (20/30)² of itself; moving any one alone lowers it, and still does for a muscle asked for 0.6 of its strength or more when that delays contact by a tenth of the move; all tied, the charge equals the old form's; inside the clock it is zero. **The old form fails it:** one muscle moved alone changes nothing, and with the later contact its charge rises.
  - *At two of session 9's results, old form and new* (`sim/target/codex-probes/body3d/s9/clock_new.py`; single runs of saved plans, no search). The old form reproduces the reviewer's `clock.py`: at the nearest motion (part 3's S) the five tied onsets moved 2% alone change the charge by +1.8, +1.7, +1.5, +0.2 and −0.1; at the 0.20 s result seven of eight raise it, by up to +37.0. **On the new form every one of the five lowers it (−0.3 to −1.6), and seven of the eight at 0.20 s (−5.2 to −19.9; one +15.4).**
  - *The scores are on a new scale where the arm muscles are not all tied:* the nearest motion's is 139.6 (it was 153.4; its clock's charge 8.1 for 22.0), the 0.20 s result's 1,746 (2,232). The figures at the plane's crossing are unchanged. No score in the tables above is recomputed; they are the old form's.
- **A late failure.** The probe on a run that really fails (the lone upper arm without its raise's end) hands back every record it made, each equal to the Python engine's to 10⁻⁹, the last within one record's interval of where the Python engine fails. On records built in the test (a sweet spot crossing the ball's plane 100 ms into a run), a failure 60 or 200 ms after the crossing leaves the run usable and says so; one 40 ms after, or before the crossing, is raised as before. Every report now says if its run failed after contact and when.
- **The two-engine check's sensitive case is now judged against itself** (the review's P3-1 suggestion, done): the cleats case is 200 ms long again, and each record's difference between the engines is held to ten times what the Python engine differs from itself with one starting velocity moved by 10⁻¹³ (and to 10⁻⁹ where that is larger). The other seven cases keep the plain 10⁻⁹.

**What the review shows about the session's results** (its probes are under `sim/target/codex-probes/review-body-3d-9/` and are not rerun here except where said; each item is the reviewer's measurement).

- **Why the search stalls (P1-2; `line.py`, `jump.py`, at the nearest motion).**
  - Along every one of the eight damped steps the search would try, the score at 1/1000, 3/1000 and 1/100 of the step is higher than where it stands, by +0.17 to +9.7, where the slope model predicts falls of 0.01–1.3. The slope model is wrong at every length tried, not only at long steps.
  - Measured forward and backward, the slope of the trunk muscle's onset is +3,568 and −14,090 at the search's 2% difference (the score rises both ways: a kink), +918 and −1,176 at 0.5%, +865 and −918 at 0.125%; the lead knee extensor's onset gives +9,875 and −920. None converges as the difference shrinks.
  - **Part of the roughness is the integrator's step, not the body:** the trunk muscle's onset moved by 0.03 ms raises the score by 0.73 at the 0.5 ms step and by 0.084 at 0.125 ms.
  - Ten of the 205 one-number differences are lower than the point itself (the best 153.375 against 153.437): what the fallback step took, which is why it gained a little each iteration and got nowhere.
  - So a damped least-squares search on one-number differences at a 0.5 ms step is the wrong tool for this score. That is the search's fault.
- **The rear wrist starts outside a real wrist's reach (P1-3; `start.py`).** Bent back 73.8° of 75° and tilted away from the thumb 29.7° of 32.4° at once: the corner of the box. [Note 11](research/body/11-hands-and-joint-ends.md) A1 has a wrist's reach as an ellipse, narrowest toward that corner. Against the plainest ellipse (the one that fits inside the box, untilted) the start is 1.33 times its radius, 33% outside; with the measured tilt, further. Part 1's design kept the box "until a run shows a wrist leaning on a corner"; the start itself does. Hitters' rear wrist at foot plant is well inside (bent back 52 ± 14°, 5 ± 12° toward the thumb).
  - **The start's rear arm flips inside the clearance's own stated range** (`start_alt.py`): with `ARM_CLEAR` at 0.08 m (its range is 0.08–0.12) the rule picks the low rear arm; the rear wrist is then near neutral in its bend (−0.9°), still tilted 28–30° away from the thumb and inside the ellipse (0.90 of its radius), with the rear forearm 5.5° from its palm-down end and the wrist 50–60° less bent back than hitters'.
- **The start leans on more ends than part 1 listed, and drifts into them (P2-1; `start.py`, `start_alt.py`).**
  - Nearest ends at the start: the lead upper arm's turn in 1.1°; the rear wrist's bend back 1.2°; the lead arm's sweep 2.0° to its stop (0.4° to the clinical end); the rear wrist's tilt 2.7°; **the lead hip's sideways end 2.8°**; the lead ankle's roll 4.1° (part 3's body).
  - Under the nearest motion's plan the lead hip's sideways end is the first to carry load (142 N·m at 28 ms; 302 N·m at 15 ms on part 1's body), then the rear wrist's tilt at 39 ms, the lead sweep at 44 ms and the lead arm's turn at 46 ms.
  - **The hold-still torques do not hold the start still.** After their solve 7–10 m/s² and 25 rad/s² of acceleration remain somewhere in the body and the centre of mass moves 67 mm in 100 ms; with no muscle at all the lead upper arm's end engages at 27 ms. Not new (the weld has 7.5 and 24.6), and never said.
  - **The lead arm's strain is [4]'s trunk and elbow, not the grip's [E]s:** at obliqueness 10°, 15° and 20° and clearance 0.06–0.12 m the lead arm stays 1.1° from its turn-in end and 134.6° across the chest. [4]'s trunk turned 10° the other way brings it 5.9° inside (the rear wrist 9.7° inside); the lead elbow bent 15° more, 5.6° inside.
- **What the feet did, part by part on this session's own bodies (P1-4, item 4).** Part 3's design compared part 3 with session 8's swing on the weld (30° of backward slide, a 35° pelvis), another body and plan. On this session's bodies, start S with the fallback: the rear foot slid −28° (part 1), **−8° (part 2, the same flat feet)** and −4° (part 3, cleats); the pelvis turned 34°, **46°** and 58°. Most of the change came with part 2's different plan. **The cleats' own share is 8° to 4° and 46° to 58°**, in searches that had not settled and that moved more than the feet.
- **The heel can come up; the kept plans do not use the rear calf (P1-4, item 3; `feet.py`).** The nearest motion asks 0.8 N·m of the rear calf's 231 (0.3%); part 1's result 32. With the rear calf at full strength from 100 or 150 ms the rear heel rises 3.1–6.8 cm on part 3's body (2.1–4.7 on the weld, 2.2–10 on part 1's), and 11–27 cm from the first instant. So nothing but the plan keeps the heel down, as part 3's withdrawal of the toe piece argued. When the heel does come up the rear foot turns further backward (−11° to −48°), the rear hip's rotators being at full strength.
- **What the held drive is (P2-5).** A tensed muscle whose opposite number cancels it exactly. A real co-contracted pair stiffens its joint: moved, the shortening muscle's force falls and the lengthening one's rises, so the pair resists. Here the hold mirrors the muscle's own force at its own speed, so a tensed limb is limp to outside forces until it fires [E]. It made no difference to this session's results (the plans' arm muscles fire at the first instant). *What a run would need to show for it to matter:* a plan that keeps muscles tensed and held for tens of milliseconds while other parts move their joints (the arms held while the legs and trunk start), with those joints carried more than a few degrees before their muscles fire; the hold's missing stiffness would then be what should have resisted.
- **The obliqueness leans on [4]'s launch posture (P3-3).** 15° was kept because at 30° the lead wrist cannot take that posture. [4]'s posture is the start, flagged, not a target, so PLAN D37 stands; but a start that owes [4] nothing must look at this [E] again. If Kapandji's 100–110° holds when re-read, 15° is its middle whatever the posture. It does not affect the lead arm's strain (above).
- **The upper arm's turn has one range at every height of the arm (P3-4) [E].** 69° in and 104° out are measured with the arm 90° out to the side. Across the chest, where the lead arm starts, a shoulder turns in less, so the lead arm's real end is, if anything, tighter than the stop.
- **Passivity, on a case the session's tests did not cover** (the reviewer's `passive.py`): the whole body with cleats, no ground, gravity, muscle or damping, every stop elastic and every body flung, 16–22 stops engaged a run, every kind session 9 added among them. The stops return what they took to within an error that takes either sign and shrinks with the step. No energy made.

**The checks after the review's fixes, every changed source touched first** (job 389; `sim/target/codex-probes/body3d/s9/fix_*.out`): `test_body3d.py` 62 tests, OK (the two new are `SearchChecks`); `test_oracle.py` 112, OK (8 expected failures); `probe3d.py --check` PASS, worst 3.6 × 10⁻¹¹ over the seven plain cases, and the sensitive cleats case within ten times the Python engine's own difference from itself at every record of its 200 ms; `bin/check-docs.py` clean. Session 8's saved swing still replays on the weld at 17.8 m/s; its score now reads 45.6 where it read 46.7, the clock's charge 4.0 for 5.1 (the new term; not every arm muscle of that plan was at the earliest onset).

### 4.12 Can this body swing? A start off the ends, a test without a search, a search made to work (session 10)

Session 10 (2026-11-26, PLAN D41: the second of three) has one question: **can this body make a swing that meets the ball at the gate's speed, yes or no, and if no, what named part stops it.** Three parts in order, each written here before it is built or run: a start that does not lean on joint ends; a motion built by hand and solved for its joint torques, with no search; then the search, by a method that does not need slopes, at a finer step. Written in the order the work was done. PLAN D39 is not built here.

#### Part 1: a start off the ends (the design, written before it was built)

**Read first, on the start as session 9 left it** (a geometry probe, no run: `sim/target/codex-probes/body3d/s10/margins.py`, `sens.py`, `ellipse.py`, `rule.py`). Every joint's distance from its nearer anatomical end at the start, on the body with hands and cleats:

| Joint | Inside its end by |
| :-- | --: |
| the lead arm across the chest (the clinical end, 135°; the stop itself was set 2° past the start by session 8's rule) | 0.4° |
| the lead upper arm's turn in | 1.1° |
| the rear wrist bent back | 1.2° |
| the rear wrist tilted away from the thumb | 2.7° |
| the lead hip, the thigh out to the side | 2.8° |
| the lead ankle's roll inward | 4.1° |
| the rear hip's turn | 6.4° |
| every other joint | 15° or more |

- The arms' margins move with the trunk's turn, forward bend and side bend, the bat's direction, the lag and the two elbows, and not with anything below the trunk. The legs' margins move with the pelvis's turn and travel, the lead knee, the stride and the pelvis's tilt, and not with anything above. The lead hip's sideways margin and the lead ankle's roll trade against each other under most of those (the lead knee 8° more bent: the hip 18° inside, the ankle past its end).

**The wrist's oblique range** (the review's P1-3; [note 11](research/body/11-hands-and-joint-ends.md) A1).

- *What is measured.* (a) Each direction alone, with a goniometer: 76° toward the palm, 75° back, 22° toward the thumb, 36° away (Boone & Azen), cut for a closed hand to 63.8° and 32.4° (§4.11). (b) The edge of the reach traced as a circle with a 25 mm cylinder in the hand: 140.2 ± 10.2° from front to back and 74.3 ± 11.9° from side to side in all (Gehrmann 2008). (c) Six cadaver wrists under ±2 N·m: an ellipse whose long axis is tilted 26.6 ± 4.4° from pure bending, toward bent-forward-and-away-from-the-thumb (Crisco 2011).
- *They do not make one ellipse.* The ellipse through the four ends of (a) with its long axis tilted as (c) does not exist (above 21° of tilt the four points lie on a hyperbola), and by 10° of tilt its front-to-back arc is already 154° against (b)'s 140°. An ellipse with (b)'s two arcs and (c)'s tilt is 31° wide where the wrist is unbent, against (a)'s 54°. They are three samples measured three ways, and a wrist's reach is not an exact ellipse.
- *The choice:* **the ellipse through the four ends of (a), not tilted** [E: tilt 0–8°]. Its centre is 5.6° bent back and 5.2° away from the thumb, its half-axes 70.7° and 27.3° *(first written as 69.4° and 27.2°, the box's half-ranges; the four ends lie on the two axes, not on lines through the ellipse's centre, so the ellipse through them is 1.3° and 0.1° larger)*. It keeps every single-direction range as measured and takes away the box's four corners. Its front-to-back arc is 141° ((b): 140°); its side-to-side arc is 54°, 20° less than (b)'s, so it is tight sideways where the wrist is bent: [E], on the side that allows a wrist less. Tilted, the corner session 9's start sat in (bent back and away from the thumb) is further outside (1.40 times the ellipse's radius untilted, 1.57 at 5°, 1.66 at 8°), and the other two corners open; so untilted is the kindest of the range to that corner and the result is reported at 8° too. Hitters' measured wrists are well inside at any of these (0.3–0.8 of the radius at foot plant and at contact, OpenBiomechanics' means).
- *Session 9's start against it:* the rear wrist 1.32 times the radius, 17.5° outside along its own direction (the review's 1.33); the lead wrist 0.75.
- *How it is held: a stop that reads both of the wrist's angles* (`engine.py`, a stop of kind 3; the same in `body3d_probe.rs`). The two angles are the ones the wrist's two stops read now, each on its own axis of the two-axis joint (§4.11: never undefined; each already tested through the joint's other freedoms). The stop's reading is **how far outside the ellipse the pair lies, measured along the line from the ellipse's centre**, in radians: negative inside. Past zero it pushes back with the small joints' spring and damper (2,000 N·m/rad, 20 N·m·s/rad, as the box's stops did) along the reading's own gradient, which is the two angles' gradients combined by the reading's two partial derivatives. The reading is a function of the joint's posture alone and the torque acts along its gradient, so by the argument of §8.1 it cannot give back more than it took. It replaces the wrist's two box stops (the ellipse passes through the box's four ends and, between them, lies inside the box but for a sliver: bent back, it reaches 76.3° at 5° of tilt away from the thumb, 1.3° past the box). *Where it is undefined:* nowhere a wrist goes; each underlying angle wraps at ±180°, 105° or more past the ellipse.
- *Tests, written with the build:* the reading against the ellipse worked by hand at the four ends, at the centre and in a corner; its gradient against finite differences of the reading, with the wrist posed across both angles and the forearm's turn and the elbow carried through their ranges (the reading must not move with them); the passivity check in both engines (a hand on a two-axis joint thrown into the ellipse from several directions, elastic: the energy comes back to within the integrator's error, which must shrink with the step; damped and driven in: never more than it started with plus the drive's work); a mutation (the torque along one angle's gradient only) fails it; the two engines agree on the whole body driven into both wrists' ellipses (`probe3d.py --check`).

**The rule for the start.**

- *What may move:* the numbers the launch posture is built from, each of them an input and none a target. [4]'s measured ones: the pelvis's turn and travel, the trunk's turn, the bat's direction seen from above and its tilt, the lag, both elbows' bends, both knees' bends, the pelvis's side tilt, the lead foot's angle, the stride's length and how closed it is. And the three the placement set by hand because [4] gives none: the trunk's forward bend (15°), its side bend (0°) and the rear foot's angle (square).
- *The rule:* **the least change to those 17 numbers** *(corrected after the review, P3-5: what the compass search below returns is a least change number by number, not shown to be the least of all)***, as the sum of the squares of the changes with a degree and a centimetre counted alike [E], that puts every joint at least 5° inside its anatomical end and each wrist at least 5° inside its ellipse.** "Every joint" is every range the body holds (§4.10, §4.11): the trunk's three, each hip's three, the knees, both of each ankle's, each shoulder's sweep across and back and its raise, the elbows, the forearms' and upper arms' turns, the wrists' ellipses. The 5° is [E: 3–8°]; the reason for it is the next block's test.
- *The three free choices stay under session 9's rule* (the bat's roll and the two elbows' swivels set so that the arm joint nearest an end is as far from it as can be, the elbows 10 cm clear of the other arm), with one change: a wrist's strain is its place in the ellipse (1 on its edge), not its two angles apart.
- *No stop is moved to clear the start any more.* Session 8's rule set the lead arm's sweep stop 2° beyond the start; with the start 5° inside, every stop stands at its anatomical end and the build fails if one would have to move.
- *How it is solved:* from no change, a search that tries each number a step up and a step down in turn and keeps what lowers the sum of squares plus a penalty on every margin short of 5°, the steps halved from 2° to 1/8° and the penalty raised in four stages until no margin is short by more than a hundredth of a degree. Deterministic; the result is stored with the body's constants and a test reruns the rule.
- *What the probe gave* (with session 9's box strain inside the free choices; the build's figures follow in "As built"): changes of 3.75° at most (the bat 3.8° further round and 2° flatter, the pelvis 2.9° more open, the trunk 2.4° more closed, the lag 2° larger, the trunk 2° bent to the other side), the square root of their summed squares 6.8; four margins at 5.0° (the rear wrist's ellipse, the lead arm across the chest, the lead hip sideways, the lead ankle's roll), the lead upper arm's turn 8.9° inside.
- **The posture is still [4]'s launch posture, moved by this rule, and is flagged as that in every result.** The starting velocities are still the slopes of [4]'s curves (a constant shift does not change a slope).

**Does the start hold still?** The hold-still torques (`drive.solve_tone`) are one constant torque at each joint, solved so that the placed body's accelerations are as small as the ground forces allow. The review found 7–10 m/s² and 25 rad/s² left after the solve and the lead arm at its end in 27 ms with no muscle working. Measured before anything is changed, on the new start:

- from rest and from the starting velocities, no drive, 50 ms and 100 ms: how far each joint has moved, how far the centre of mass has moved less what its starting velocity alone would carry it, which ends carry load and when;
- what the leftover acceleration is made of: the whole body's (its centre of mass's acceleration and the rate of change of its angular momentum, which no joint torque can alter: they are the ground's forces against the weight) and what is left at the joints once that is taken out.

*What I expect:* no end carries load within 50 ms from either start (the nearest is 5° away; at the review's 25 rad/s² a joint needs about 80 ms to drift 5°), and some do by 100 ms. Most of the leftover is the whole body's: the feet's forces are [4]'s at −250 ms, when its hitters were not standing still, so they do not balance the weight, and a joint torque cannot change that. If so the drift is the start's own motion, as [4] measured it, and not a fault of the hold; if a large share is at the joints instead, the solve is at fault and is fixed before part 3.

**The start's sensitivity to the elbow clearance** [E: 0.08–0.12 m] is reported with the result: the rule rerun at 0.08 and 0.12 m, the rear arm's posture, the changes and the margins at each.

**What a start that owes [4] nothing would add** (not built here; §4.10 sized it). The launch posture and its velocities would be results of the batter's own stance, load and stride, so nothing of the lab study would be left in the swing at all; and the two things this rule cannot do would be done: the weight would be where his own motion put it (so the feet's forces would balance his body, and "hold still" would mean something), and the grip's obliqueness would be chosen without [4]'s posture (§4.11, the review's P3-3). It needs postural feedback through the stance and stride (§5.3) and is not inside this session.

**Part 1 as built** (`engine.py`: a stop of kind 3, `oval_reading`, `oval_angle`; `body3d_probe.rs` the same; `body.py`: `wrist_oval`, `oval_place`, `oval_spec`, `Model(wrist="oval", shift=...)`, `POSTURE_NUMBERS`, `clear_start`, `START_CLEAR`, `start_margins`; `practice3d.py`: `--wrist oval`, `--clear`; `test_body3d.py`: `OvalChecks`, `StartChecks`; outputs under `sim/target/codex-probes/body3d/s10/`: `run_rule_main_and_clearance.out`, `still.out`).

- **The ellipse is the one worked by hand** (`OvalChecks`): each of the four measured ends lies on it; the box's bent-back-and-away-from-the-thumb corner is 1.40 times its radius; tilted 8° it still passes through the four ends; tilted 25° it does not exist. Its reading for a wrist posed from the clinical definitions is the distance worked from the posed angles, inside and out, on either hand, and does not move with the forearm's turn, the elbow or the upper arm (10⁻⁷°).
- **The stop is passive.** A body on the wrist's two-axis joint thrown into the ellipse in eight directions: elastic, its energy never exceeds the start's and comes back to within an error that takes either sign and shrinks with the step (over the eight directions 8.0 × 10⁻⁴ of the energy at a 0.1 ms step, 2.6 × 10⁻⁵ at 0.025 ms); damped, it loses energy; driven in by 15 N·m it never has more than the start plus the drive's work. Its gradient equals the reading's rate by finite differences. **The mutation (the torque along one angle's gradient alone) ends with 800–1,500 times the starting energy.**
- **The rule's result** (job 391, 264 s): thirteen of the seventeen numbers change, none by more than 2.5, the square root of the summed squares 5.1:

| The launch posture's number | Change |
| :-- | --: |
| the lag (the bat against the lead arm) | +2.5° |
| the bat's tilt; its direction seen from above | +2.1°; +2.0° |
| the pelvis's turn; the trunk's | +1.75° (more open); −1.75° (more closed) |
| the pelvis's side tilt | +1.5° |
| the rear knee; the lead knee | +1.0°; −0.75° |
| the lead foot's angle; the stride closed; the stride's length | +0.6°; −0.6 cm; −0.4 cm |
| the rear elbow; the trunk's side bend | −0.6°; −0.25° |
| the lead elbow, the pelvis's travel, the trunk's forward bend, the rear foot's angle | unchanged |

  - *Margins:* the lead arm across the chest, the lead hip sideways and the lead ankle's roll 5.0°; the rear wrist's ellipse 5.1°, the lead's 5.3°; the lead upper arm's turn 6.0°; the rear hip's turn 7.2°; every other joint 15° or more. `StartChecks` holds the stored change to the rule: every joint 5° inside, every stop at its anatomical end with none moved and none carrying anything, and each changed number moved an eighth back toward [4]'s leaves some joint short (the whole rule is rerun with `BB_SLOW=1`).
  - *The arms at the start:* the bat rolled 31° and the rear elbow swivelled 93° up and back, as session 9's (26° and 94°): the rear elbow is 15 cm above its shoulder. **The rear wrist is bent back 71° and tilted 8° away from the thumb (session 9: 74° and 30°)**, 0.93 of the way to its ellipse's edge; the lead wrist bent back 57° and 10° toward the thumb, 0.91. Against hitters at foot plant (OpenBiomechanics; for checking): the rear wrist's tilt is now inside one SD (−8° against +5 ± 12); both wrists are still bent back more than hitters' (rear −71 against −52 ± 14; lead −57 against −19 ± 18, 2.1 SD). On an ellipse tilted 8° the rear wrist would be at 1.02 of the radius, just outside: the start leans on the tilt's [E].
- **The start's sensitivity to the elbow clearance** (the rule rerun; same job):

| Elbows clear by | Rear arm | The change (root of the summed squares; largest) | Rear wrist: bend, tilt, place in its ellipse | Lead wrist |
| :-- | :-- | :-- | :-- | :-- |
| 8 cm | **low: the rear elbow 17 cm below its shoulder**, the arm raised 68° | 5.9; 2.5° | +3°, −26°; 0.75 | −15°, +11°; 0.62 |
| **10 cm (as built)** | high: 15 cm above, raised 128° | 5.1; 2.5° | −71°, −8°; 0.93 | −57°, +10°; 0.91 |
| 12 cm | high: 15 cm above | 5.6; 2.5° | −69°, −8°; 0.90 | −58°, +9°; 0.91 |

  **The rear arm still flips inside the clearance's own range** (the review's P1-3): at 8 cm the rule takes the low rear arm, which leaves both wrists far from their edges and the rear wrist unbent, where a hitter's is bent back 52°. The start keeps 10 cm and the high elbow, which is how a hitter stands, and says that this rests on an [E]. Not run: the rule at 3° and 8° of margin and on the tilted ellipse (queued and stopped to free the queue).
- **Does the start hold still? It drifts as [4]'s moving hitters did, and no end carries load in the first 50 ms** (`still.py`; no drive at all; a 0.25 ms step):

| | Session 9's start | The cleared start |
| :-- | :-- | :-- |
| left after the hold-still solve | 10.3 m/s², 24.9 rad/s² | 10.2 m/s², 21.7 rad/s² |
| the ground's force against the weight (912 N) | 714 N up, 81 and 85 N sideways | the same |
| so the centre of mass accelerates | 2.1 m/s² downward, 0.9 each way level | the same |
| the ground's moment about the centre of mass | 26, 101, 59 N·m | 23, 102, 59 N·m |
| from rest, by 50 ms; by 100 ms | the lead ankle 5°, 17°; nothing else over 2°, 5° | the same (5°, 16°) |
| from rest, ends carrying load within 100 ms | the lead upper arm's turn at 75 ms | **none** |
| from the starting velocities, ends within 100 ms | the lead upper arm's turn at 30 ms, the rear wrist 55, the lead hip's turn 65, the lead arm's sweep 65 | **the lead hip's turn at 75 ms; none before** |
| the centre of mass by 100 ms, of which its starting velocity alone | 67 mm, 57 | 66 mm, 56 |

  - *What the leftover is made of.* The feet's forces are [4]'s at −250 ms: 78% of the weight, and not under the centre of mass. No joint torque can change the whole body's motion, so the body starts falling at a fifth of gravity onto its front foot, as [4]'s hitters were doing at that instant. The hold-still solve is a least-squares fit over every body's acceleration counted alike, and it puts most of that imbalance into one light body: **the lead foot, on its toe, turning its heel down at 30 m/s²** (5° in 50 ms). That is the weight coming onto the front foot, not a joint let go.
  - *The review's "67 mm in 100 ms" was 57 mm of starting velocity and 10 mm of drift.*
  - *Against what I expected:* right that no end carries load in 50 ms from either start, and that some do by 100 ms (one, from the starting velocities; none from rest). Right that the leftover is the ground's imbalance; I had not foreseen that the solve would put it into the lead foot. The solve is left as it is: a hold that kept the feet still would have to change the feet's forces, which are the start's own.

#### Part 2: the test without a search (the design, written before it was built or run)

**The question this part can answer:** is there a motion from the cleared start to a hitter's contact posture, putting the sweet spot through the aim's ball along the ball's line at the gate's speed, that asks no muscle group for more than its strength at the speed it is moving, no foot for more than the ground can give, and no joint to go past its range? **This is a diagnosis.** The motion is no target, nothing is fitted to it and no score reads it (PLAN D37). It is built from joint ranges, the aim's ball and stated time laws, and is not shaped on [4]'s curves; the one thing of [4]'s in it is the start, as everywhere.

**How a motion is built** (`analysis/research/body/built3d.py`).

- *A posture is the 17 numbers of part 1 and three more:* the pelvis's turn, travel and side tilt; both knees' bends (the pelvis's height and the raised heel follow from them, as the placement has always worked); both feet's angles, the stride; the trunk's turn, forward bend and side bend; the bat's direction seen from above and its tilt; where the hands are (the lead hand's place, carried in the trunk's frame, in place of [4]'s two elbows and lag); the bat's roll and the two elbows' swivels.
- *The start:* part 1's, with its starting velocities.
- *The contact posture,* built as `test_a_hitters_contact_posture_lies_inside_every_arm_range` builds one, for the whole body:
  - the sweet spot at the aim's ball; the bat square to the ball's line seen from above and its barrel 20° below the hands [E: 10–30];
  - the pelvis turned 75° toward the pitcher from facing the plate [E: 60–90; hitters' hips face the pitcher or nearly at contact] and carried forward 10 cm [E: 0–17, OpenBiomechanics' travel];
  - the trunk turned 15° further than the pelvis, bent forward 30° and to the side 20° [E, inside the test's 25–40° and 15–30°; hitters' side tilt at contact is 32°, OpenBiomechanics];
  - the lead knee 14° bent, the rear 70° [E: 55–90], the rear foot turned 60° toward the pitcher on its ball (its heel's height follows from the knees);
  - the bat's roll and the two swivels by part 1's rule of least strain with the elbows clear; the elbows' bends are then results, reported against OpenBiomechanics' 53 ± 15° and 79 ± 13°.
- *What moves at contact:* the sweet spot's velocity is the gate's speed along the ball's line, coming up through it (+5.3°). The hands carry a fifth of it [E: 0.15–0.30; measured hands are at 4–7 m/s at contact] and the bat's turning about the hands the rest, about the axis square to the bat and to that line. The pelvis is still turning at half its average rate and the trunk at its average rate [E].
- *The time law:* each number runs from its start to its contact value on a cubic in time that meets its starting rate and its rate at contact; before its own onset it carries on at its starting rate. Three onsets: the legs and pelvis, the trunk, the arms and bat.
- *After contact* the same laws run on for 20 ms, so that the accelerations at contact are differences of real postures.
- **The base motion:** 200 ms from the start to contact; the trunk starts 30 ms after the legs and the arms 60 ms after; the pelvis carried forward 10 cm; 29 m/s.

**The motions, varied in a stated way** (one thing at a time from the base, then every combination of the first three):

| Varied | Values |
| :-- | :-- |
| how long it takes, start to contact | 160, **200**, 250 ms |
| the order: the trunk's and the arms' onsets after the legs' | 0 and 0 (all together); **30 and 60**; 50 and 100 ms |
| the weight shift: the pelvis carried forward | 0, **10**, 17 cm |
| the sweet spot's speed | 17, 21, 25, **29**, 32 m/s |
| the hands' share of that speed | 0.15, **0.20**, 0.30 |
| the barrel below the hands | 10, **20**, 30° |
| the pelvis's turn at contact | 60, **75**, 90° |

27 combinations of the first three at 29 m/s and at 21 m/s, and the single changes: about 70 motions. Each is first checked as geometry: every joint inside its range at every instant (how far inside is reported), the elbows clear, the feet where they were. A motion that leaves a range is reported as that and is not solved.

**How the torques are solved** (the engine's own equations, as `inverse3d.py` does; the Python engine, no ground model).

- Every 5 ms the body is placed on the motion; velocities and accelerations are differences of postures 2 ms apart, made exactly consistent with the joints.
- The unknowns are the loads that can act on this body: **a torque about each of the 29 joint axes the muscles act about** (the trunk's three about the pelvis's axes; each hip's three about the thigh's own axes, the way hip strength is tested; each knee's hinge; each ankle's bend and roll; each shoulder's three, about the trunk's long axis, about the floating axis that raises and lowers the arm, and about the upper arm's own length; each elbow's bend and the forearm's turn; each wrist's bend and tilt), and **a force at each sole point that is on the ground**, which can only push, inside the friction cone of the measured grip (1.15).
- The joints' own damping [E] is in the equations, so the torques are what the muscles must give. The hold-still torques are not: they stand for muscle.
- **Each torque is held against the muscle group that makes it, at the speed that joint axis is turning** (strength × its measured strength-at-speed curve, §4.5; a muscle being stretched has its eccentric plateau). The two arms and the bat are a closed ring and the two feet another, so the loads are not determined by the motion: twelve numbers are free. They are chosen, at each instant, **to make the hardest-worked muscle group's share of its strength at that speed as small as it can be** (a linear programme; `scipy`'s solver, run through `uv`). So the answer at each instant is the best this body could do for that motion, not one arbitrary sharing.
- **Directions no muscle of this body acts in** (§4.5 lists them: the trunk's bending forward, back and sideways; the lead arm drawn across and the rear arm drawn back about the trunk's axis; the rear hip's flexion; each hip's rotators the other way; each ankle's pull-up and roll; each wrist's tilt toward the thumb) are left free in that solve and then made as small as they can be without raising the worst share; what they are asked for is reported in N·m beside a measured strength where one exists (the trunk's bending: Schultz 1987's standing men, mean + 2 SD scaled as §4.5 scales, 370–480 N·m; for scale only, the body has no such muscle).
- **A joint's end carries nothing by construction:** the motion is inside every range, so no stop is engaged; the margins are the report.
- *Checks of the solve itself:* a forward run of a known plan (a saved practice result) is fed back as a motion and its drives' torques, read from the run, must come back as a feasible sharing with a worst share no larger than the run's own; the solved loads put back into the engine's equations must reproduce the motion's accelerations (to 10⁻⁶ of their size); the result at a 2.5 ms sampling and with the differences taken over 1 ms and 4 ms.

**What a pass and a fail would and would not show** (written before any run).

- *A pass* (some motion whose worst share stays at or under 1 at every instant, on its feet): that one motion is inside this body's strengths, ranges and grip, **so the body is strong enough to swing at that speed** and the search's job is to find and control such a motion. It would not show that muscles can drive it: a muscle's torque here may jump from one 5 ms to the next, where the body's muscles need at least 50 ms to rise and 50 ms to let go and act through tendon springs; nor that it balances after contact; nor that it is a hitter's swing; nor that the practice search can find it.
- *A fail at 29 m/s with a pass at a lower speed:* this family of motions is short of the gate, by a named muscle group at a named instant. The speed at which the worst share crosses 1, motion by motion, is the body's limit **for this family**.
- *When "this body cannot" may be said:* only if every motion of the table fails at the same named joint with a wide margin (a worst share over 1.5), and making that joint's work easier in the ways the table allows (longer, a different order, more or less weight shift, more from the hands) does not bring it under. About 70 motions in seven directions is a family, not every motion; a fail is "no motion of this family", and the write-up says which freedoms the family did not vary (the path between start and contact; the contact posture's other numbers).
- *A muscle that is asked for something in a direction it does not have* is a statement about the body's list of muscles (§4.5), not about strength; it is reported apart.

**What I expect:** the base motion at 29 m/s fails. The arms' muscles are small (the forearms' 17–18 N·m, the wrists' 24–51, the shoulders' turn 39–70) and the bat's last 60 ms asks most of them; I expect the worst share between 1.5 and 4 there, at the wrists or the forearms' turn, and the trunk's side bend asked for 200–400 N·m with no muscle to give it. The legs and the trunk's turn under 1. The speed at which the base motion's worst share crosses 1: 17–23 m/s. Longer and more staggered motions do better than short ones. I do not know whether any of the 70 passes at 29.

**Part 2 as built and run: the test did not decide. No motion I built by hand is one a body could make, so none could be held against the muscles; what was found on the way is about the contact posture.** (`analysis/research/body/built3d.py`; run with `uv run --with numpy --with scipy`; outputs under `sim/target/codex-probes/body3d/s10/`: `contact_grid`, `contact_search_first.out`, `contact_search_balanced.out`, `built_base.out`, `built_table.out`, `built3d_table.json`, `audit.out`; jobs 392–394.)

- **The solve is built and checks out** (`built3d.py --check`):
  - each of the engine's 41 drive channels, alone, gives exactly the accelerations of one of the solve's 29 joint axes with the stated sign (largest difference 3 × 10⁻¹⁵ of an acceleration): the solve's muscles are the engine's;
  - the solved loads put back into the engine's equations give the motion's accelerations to 10⁻¹²;
  - holding the start's posture still asks at most 0.126 of any muscle group (the lead arm's raiser), the ground carries the whole weight, 912.0 N, and nothing is asked that the feet cannot give; under twice the gravity the share is exactly twice;
  - the start's 20 numbers placed again give the placed start to 0.13 mm.
  - *Changed from the design:* the check "a forward run fed back as a motion" was replaced by the channel-by-channel one, which is sharper (a wrong sign on one channel cannot hide in a sharing). Added: what the feet cannot give is measured (a force on the pelvis from nowhere, made least before anything else), so that a motion the ground cannot hold up is reported as that and not as "no solution".
- **A hitter's contact posture, as the design wrote it, cannot be taken for the aim's ball.** With the pelvis turned 75° and carried 10 cm forward, the lead knee 14° bent and the trunk bent 30° forward and 20° sideways, the hands are 0.70 m from the lead shoulder and the lead arm is 0.58 m long. Over the design's own [E]s (144 postures: the trunk's two bends, the weight shift, the lead knee 14° or 26°, the barrel's tilt, the pelvis at 75° or 90°): 68 can be reached, **none with every arm joint inside its range** (the nearest is 7% past an end), and the lead hip is 10–25° past the end of its turn in all of them. So the contact posture was found by a rule instead: the posture whose joint nearest an end is furthest from it (a compass search over the pelvis, the trunk, the rear foot and the bat's tilt; the bat's roll and the elbows' swivels by part 1's rule inside it).

| Contact posture for the aim's ball | Tightest joint, inside its end by | The hips open | Lead knee; rear elbow; lead elbow | Other |
| :-- | :-- | :-- | :-- | :-- |
| a hitter's, by the design's numbers | cannot be taken | 75–90° | 14° | the lead hip's turn 10–25° past its end |
| everything free, the weight anywhere | 7.5° (both wrists' ellipses; the rear hip's turn 8.5°) | **36°** | 49°; 13°; 46° | the trunk twisted 50° on the pelvis; the body's weight toward the plate, off its feet |
| the same, the pelvis held at 60°; at 75°; at 90° | 5.3°; **−0.2°** (the lead hip sideways and its turn, both at their ends); cannot be placed | 60°; 75°; 90° | 27°; 7°; 59° at 60° | |
| **the weight over the feet** (the centre of mass 0.6 of the way from the rear forefoot to the lead foot [E: 0.5–0.7]), the pelvis free | **0.9°** (the lead wrist's ellipse, the rear hip's turn) | 53° | 48°; 30°; 48° | the pelvis 9 cm lower than at the start, the trunk bent 38° to the side |
| the same, the pelvis held at 70° | **1.0°** (the lead wrist's ellipse; the lead hip sideways 2.5°) | 70° | 38°; 30°; 50° | the pelvis 8.5 cm lower |
| hitters measured (OpenBiomechanics; for checking) | | about square to the pitcher | 14 ± 12°; 79 ± 13°; 53 ± 15° | the trunk tilted 32° to the side |

  - **With the front foot planted where it stands, this body's hips cannot face the pitcher inside its joints' ranges** *(for scale, added after the review, P2-3: [4]'s own hitters have the pelvis at 77 ± 6° at contact, §4.4's table, so "about 70°" is one SD short of them, not 20° short of square)***.** The lead hip turns in 32.6° (Roaas & Andersson's mean), the knee is a hinge and the cleated foot is planted 30° open; past about 70° of pelvis turn the lead hip is at two ends at once. The body does have a contact posture for this ball inside every range and on its feet, **by one degree**, and it gets there by sitting: the front knee bent 38–48° where a hitter's is nearly straight (14 ± 12°), the pelvis 8.5–9 cm lower than at the start, the rear elbow at 30° where a hitter's is at 79°.
  - *What this rests on:* the lead foot's place and angle ([4]'s stride, flagged); the hip's turning range (the mean of 105 men of 30–40, passive; Boone & Azen's active 44° would give 11° more); a knee with no twist and an ankle that cannot turn about the shin (§4.11: 5–9° under load in life); the ball's place (0.92 m toward the plate from the hips and 0.72 m ahead, PLAN D28–D29, the second [E: 0.61–0.84]); and the search (a compass search from four starts; the margin could be a degree or two better, and one start of the held-at-60° case ended 13° outside).
- **The motions: 64 specified, 44 could be built, none stays inside the joints' ranges, none was solved** (the design's rule: a motion that leaves a range is reported as that). Three faults of the construction, in the order they showed:
  1. *Both heels off the ground mid-swing.* The first base motion brought the lead heel down on the same cubic as everything else, so from −150 to −20 ms the body stood on its two forefeet. Fixed: the lead heel is down 60 ms after the start [E] and stays.
  2. *The body thrown off its feet.* The contact posture first used had the weight toward the plate, and its trunk's lean arrives in 170 ms: the feet would have had to give 1,500–4,000 N sideways. Fixed by the weight-over-the-feet rule above. That posture is 8.5 cm lower than the start, and a cubic that takes the pelvis there in 200 ms asks it to drop at 12.8 m/s² at first: faster than a body falls. Not fixed.
  3. **The arms between the start and contact.** The start has the rear elbow above the shoulder and both wrists bent back; contact has the rear arm reaching forward and down. Carried between them on cubics of the two elbows' swivels and the bat's roll, the wrists pass 140–150° outside their ellipses and the forearms 70–100° past their ends. Carried instead by the shortest turn of each upper arm and forearm in the trunk's frame, with the joints then closed on the bat, the rear elbow bends 118° backwards on the way. Over the 44 motions the worst joint is 105–191° outside its range (both wrists and both forearms in every one). **The bat comes round 231° between the start and contact and the hands turn with it; how the forearms and wrists take that turn up is not something two end postures and a time law decide.** A hitter's top hand rolls under and over through it.
  - The torques of such motions were looked at once and are not results: hundreds of times the muscles' strengths, at joints passing through postures no joint has.
- **An audit that needs no motion** (`audit.py`; a necessary condition and no more). On this body, in the contact posture above with the sweet spot at 29 m/s, the hands at 8.4 m/s, the trunk turning 470°/s and the pelvis 220°/s, the body holds **690 J** *(the audit leaves out what the joints' damping takes on the way, 266 J in the 17.7 m/s swing and more at higher speeds: the review, P1-2; the margin below is narrower than it reads)*: the bat and hands 357, the arms 209, the trunk 119, the pelvis and legs 6 (at 21 m/s: 434 J; at 17: 341). The most power each muscle group can give, at its own best speed (strength × its strength-at-speed curve × speed, §4.5): the two hip extensors 1,430 W each, the knee extensors 960, the trunk's rotators 830, the shoulders' 260–570, the elbows' 170–215, the wrists' 55–140, the forearms' 56–86. **All together, the stronger of each opposing pair, every one at its best speed at the same instant: 11.6 kW (legs 7.0, trunk 0.8, arms 3.8); over 200 ms that is 2,300 J, 3.4 times what the swing holds.** So the muscles' power does not rule a 29 m/s swing out, and the margin is not wide: no motion has every muscle at its best speed for 200 ms, and the arms alone (750 J at that bound) could not give the bat, hands and arms their 566 J unless they worked at three quarters of it throughout. Most of the bat's energy has to come up from the legs through the trunk, as it does in a hitter.
- **Against what I expected:** wrong on the main point. I expected the base motion to be solved and to fail on the arms' small muscles by 1.5–4 times, with a speed of 17–23 m/s at which it passed. No motion got as far as being solved. I did not foresee that a hitter's contact posture cannot be taken, nor that the arms' path would be the obstacle.
- **What this part shows, and what it does not.**
  - *Shown:* this body has a balanced contact posture for the aim's ball inside every joint range, by 1°, and it is not a hitter's (the front knee bent 38–48°, the hips at most 70° open). A hitter's contact posture with the hips facing the pitcher does not exist on it with the front foot planted as it stands. The muscles' summed power is 3.4 times a 29 m/s swing's energy at the loosest bound.
  - *Not shown:* whether the body is strong enough to swing at 29 m/s, or at any speed. The test needed one valid motion and I did not build one. **This is a failure of the construction, not a finding about the body**, and "this body cannot" may not be said from it: by the rule written above, a fail counts only for motions that could be solved.
  - *Not turned into a plan:* nothing of part 2 enters part 3.
  - *What would make the test decide* (for the last session): the motion's shape between the two ends has to be found, not written: the same solve inside a search over the posture numbers' paths (a few knots each), asking for the least worst share with every range held and the feet's limits kept. That is a search, but over smooth kinematics with no stiff ends, no ground that sticks and slips and no time step, which are what make the practice search's score rough. It is still a diagnosis: no recorded motion enters it and nothing of it need enter the practice search.

#### Part 3: the search, made to work, then run (the method, written before any run)

**What is searched and scored is unchanged:** the plan's 205 numbers (each muscle group's strength, onset, rise, switch-off and level beforehand), the aim of §4.10 with contact read at the sweet spot's nearest approach, the clock charging each arm muscle its own overrun (§4.11). The body is session 9's with part 1's two changes: hands, a muscle that holds, cleats, **the wrists' ellipses, the cleared start** (still [4]'s launch posture moved by part 1's rule, flagged in every result). Every muscle is free, the rear calf among them (nothing is switched off or held at its old level).

**The method: an evolution strategy that needs no slopes** (`practice3d.py`, `cma()`; the separable form of CMA-ES, Ros & Hansen 2008, written out in the standard library).

- *What it does.* It keeps a centre plan and, for each of the 205 numbers, a spread. Each generation it draws 24 plans about the centre, runs them all, ranks them by score, moves the centre to the weighted mean of the better half, and widens or narrows each number's spread by how the better plans were spread. Only the order of the scores is used.
- *Why this one:* the review measured the score's slopes as unusable (they disagree forward and backward and do not settle as the difference shrinks) and its value as jumping by fractions of a unit under a 0.03 ms change of one onset. A method that uses ranks of whole runs over a spread of plans is not misled by a jump smaller than the spread's own effect, needs no difference to be small, and cannot be refused a step. The separable form (one spread per number, no cross terms) is used because with 205 numbers and a few thousand runs a full covariance could not be learned. *What it gives up:* it cannot learn that two numbers must move together except through the centre's own drift, and it is a local method from where it starts.
- *Its settings, all [E] and stated:* 24 plans a generation (the standard 4 + 3 ln 205 is 19; 24 is four workers' six each), the better 12 weighted by rank; the first spread 0.03 of each number's range (the old search's difference was 0.02); the standard learning rates for the separable form; a plan outside the box is run at the box's edge and charged 10⁴ times its squared distance outside; a failed run ranks last and is counted; the random draws are a fixed seed's. The result is the best plan ever run, not the last centre.
- *The step:* **every run at 0.25 ms** (0.18 s a run; session 9's searches ran at 0.5 ms). On the nearest plan the bat's speed at 0.25 ms and at 0.125 ms agree to 0.01 m/s, where 0.5 ms differs by 0.2.
- *Two stages, as before:* 100 generations of the reach stage (only bring the sweet spot to the ball, in balance, inside the clock), then 250 of the whole aim; 8,400 runs a start, about ten minutes.

**The search's own checks, built in** (rule 6; printed with every result):

1. *The result at three steps:* the best plan run at 0.5, 0.25 and 0.125 ms: its score, its miss and its speed at each. A result that holds only at the search's own step is not a result.
2. *The line from where it started to where it ended:* the score at eleven points between the starting plan and the result and one beyond it, at 0.25 and at 0.125 ms. It should fall along the line; how rough it is on the way says what the search was working against.
3. *Slopes forward and backward,* for the eight numbers the search moved furthest, at differences of 0.02 and 0.005: reported, not used, as the measure of how rough the score is at the result.
4. *The spread at the end* (the largest and the median of the 205) and the score's fall over the last 50 generations: whether it had settled.
5. *Failed runs,* counted every generation; and the old count of failed one-number differences at the result.

**The runs** (one job each, one at a time):

| Start | What it is | Viewing times |
| :-- | :-- | :-- |
| S | session 9's nearest plan (it came from the fit to [4], through sessions 8 and 9: flagged) | 0.10 s and 0.20 s |
| K | S with the rear calf at half its strength from 100 ms after the start, rising in 80 ms [E]: the review showed the heel comes up when the calf works and that no kept plan used it | 0.10 s and 0.20 s |
| C | written by hand, 0.6 of every forward muscle (owes [4] nothing) | 0.10 s and 0.20 s |
| D | written by hand, 0.3 of every muscle, random timings (owes [4] nothing) | 0.10 s and 0.20 s |

Part 2 made no motion that could be turned into a plan. Each result is read against every gate check with its margin, with each passive element's net work, the joint ends' loads, and both wrists' place in their ellipses.

**What I expect:**

- *The search moves.* From S the score (315 on the cleared start: the miss 57, balance 156, the line 34, speed 32, the clock 15) falls below 150 at 0.10 s within the 350 generations, mostly by balance and the miss; the line check shows a fall from start to result at both steps.
- *It does not reach the gate.* The best of the eight meets the ball within 3 cm at 17–22 m/s; none is inside 29–34. The hips stay under 450°/s (the lead hip's range, part 2). Joint ends still carry hundreds of N·m (D39 is not built).
- *The viewing time:* at 0.20 s the arms may fire only 153 ms before contact; I expect contact speeds 1–3 m/s lower than at 0.10 s from the same start, with the legs and trunk starting earlier relative to the arms. This is the first run that can show it.
- *C and D:* they cross the ball's plane (they did on cleats in session 9) and come within 10 cm; I do not expect either to beat S.
- *K:* the rear heel comes up; whether that buys hip speed I do not know.
- If the search does not move from S (the score falling less than 5% in 350 generations), the method is at fault again and that is the finding.

**Part 3 as built and run: the search moves from every start, and the body with real hands has a swing again. From session 9's plan it meets the ball within 0.2 cm at 17.7 m/s (40 mph) and has settled there; a plan written by hand, which owes the lab study nothing, meets it within 0.4 cm at 10.1 m/s. Nothing reaches the gate's speed.** (`practice3d.py`: `cma`, `search_checks`, `--cma`; `test_body3d.py`: `SearchChecks.test_the_search_without_slopes_finds_a_known_minimum_on_a_score_its_slopes_would_miss`; jobs 395–404, 7.3 minutes a run and 9.5 a continuation; outputs `sim/target/codex-probes/body3d/s10/cma_*.out` and `.json`; the two results kept in the repository are `analysis/research/body/practice3d_s10_S_view100.json` and `practice3d_s10_S_view200.json`. Every result is from [4]'s launch posture moved by part 1's rule, an input, flagged; S and K are session 9's plan, which came from the fit to [4].)

- **The method on a score with a known answer** (the test): 30 numbers, some of the answer on the box's edge, the score stepped by 0.05 at every 0.003 of each number, one plan in fifty failing to run: from 0.2 away the search ends within 0.02 of the known point in every number, the same twice from one seed, and never runs a plan outside the box.
- **No run failed in any of the ten searches** (84,000 runs of the eight and 24,000 of the two continuations), and no result has a failed one-number difference.

| Start, viewing time | Score: start → end (fall over the last 50 generations) | By the plane's crossing: the miss up, along the bat | Sweet spot | Attack angle | Hips; trunk (°/s) | The first arm muscle fires (allowed) | What it is |
| :-- | :-- | :-- | --: | --: | :-- | :-- | :-- |
| **S, 0.10 s** | 303 → 45.2 (4.8) | 0.3; 1.0 cm | 16.3 m/s | −11.8° | 375; 469 | 261 ms (253) | a swing; not settled |
| **S, 0.10 s, carried on 500 generations** | 45.2 → **32.5 (0.65: settled)** | **0.2; −0.1 cm** | **17.7 m/s** | **−3.6°** | 361; 414 | 256 ms (253) | **the best found** |
| S, 0.20 s | 1,996 → 137.5 (11.7) | 3.0; 2.3 cm | 19.2 | −30.8° | 392; 598 | 198 ms (153) | passes 3.8 cm away, the arms 45 ms early |
| **S, 0.20 s, carried on 500** | 137.5 → 73.2 (7.0: not settled) | 1.7; 0.9 cm | 17.5 | −25.9° | **544; 699** | 165 ms (153) | passes 1.9 cm away, chopping down; the arms 12 ms early |
| K (the rear calf working), 0.10 s | 427 → 45.9 (2.0) | 0.7; 1.5 cm | 14.0 | −9.4° | 272; 377 | 263 (253) | a swing, slower than S |
| K, 0.20 s | 2,154 → 132.5 (14.1) | 2.2; 2.4 cm | 18.7 | −29.0° | 371; 615 | 201 (153) | as S at 0.20 s |
| **C (by hand; owes [4] nothing), 0.10 s** | 1,641 → 66.7 (6.1) | **0.1; 0.4 cm** | **10.1** | −34.4° | 199; 305 | 262 (253) | **a swing: slow, chopping down** |
| C, 0.20 s | 3,206 → 212.3 (26.3) | 77; 36 cm | 14.2 | −45° | 408; 228 | 140 (153) | parked: the bat brought to the ball and stopped (−2.3 m/s along the line there) |
| D (by hand, random timings), 0.10 s | 11,965 → 62.0 (5.2) | 54; 2.8 cm | 14.3 | −60° | 263; 304 | 195 (253) | parked (−0.2 m/s) |
| D, 0.20 s | 14,379 → 213.1 (37.6) | 62; 18 cm | 15.2 | −54° | 334; 270 | 166 (153) | parked (−2.3 m/s) |

- **The search's own checks** (rule 6; every result):
  1. *At three steps.* Every result's score is the same at 0.5, 0.25 and 0.125 ms to 0.2 (the best: 32.5, 32.5, 32.5; its speed 17.69 m/s and its miss within 0.1 cm at all three). The roughness the review measured at 0.5 ms is not what these results rest on.
  2. *Along the line from start to result* (at 0.25 ms; at 0.125 ms the same within 3):
     - S, 0.10 s: 303, 313, 325, **353**, 315, 109, 97, 80, 63, 50, 45; beyond the result 75. **The score rises by 50 before it falls.** *(WITHDRAWN after the review, P2-1: I wrote here that this ridge, with the score's roughness, is why session 9 stalled. The review measured the line term by term: the ridge is the balance term alone, a cliff from 228 to 20 between 0.4 and 0.5 of the way, every other term falling all along; the whole-aim stage never crossed it, the reach stage having already carried the plan past by another path; and the line is on this session's body and start, not session 9's. It shows that the straight path from S to this result loses its balance before regaining it, and nothing about session 9, whose review's causes stand.)*
     - its continuation: 45, 51, 62, 68, **72**, 59, 48, 45, 40, 34, 32; beyond 34: a second ridge.
     - S, 0.20 s: 1,996 falling at every point to 137, and 131 beyond the result: it had not arrived. Its continuation: 137, 137, 139, 144, 158, **170**, 158, 125, 104, 87, 73; beyond 83.
     - C and D: falling at every point (C at 0.10 s: 1,641 to 67, then 108 beyond).
  3. *Slopes at the result, forward and backward* (the eight numbers each search moved furthest). At S's first result, of the five slopes that are not zero three disagree in sign forward and backward at a difference of 0.02 and two at 0.005 (the lead shoulder's sweep muscle's onset: +17 and −18). At the settled result two of eight disagree at 0.02 (the rear arm's lowering muscle's strength: +14 and −37) and none at 0.005, with sizes that differ up to eightfold. The score is still not smooth at a result; the search did not need it to be.
  4. *Settled?* Only the continued S at 0.10 s (0.65 over its last 50 generations; its spreads 0.007 of a number's range at the median, 0.020 at the largest). Every other result was still falling when it stopped.
- **The best found (S at 0.10 s, carried on) against the gate** (never given to the search; the gate's figure in brackets):

| Check | The swing | |
| :-- | :-- | :-- |
| meets the ball (1 cm) | 0.2 cm up, 0.1 cm along the bat; the same at a quarter of the step | **passes** |
| the sweet spot (29–34 m/s) | **17.7 m/s (40 mph)**: 11.3 short | fails |
| the bat at its fastest at contact (98%) | 92%: fastest 19.2 m/s 31 ms before | fails |
| the hands' fastest (10.3–13.0 m/s) | 7.6 | fails |
| the attack angle (−3 to +11°) | −3.6° | fails by 0.6° |
| the hips'; the trunk's fastest turning (555–855; 755–1,075°/s) | 361; 414 | fail |
| first rises: legs, trunk, arms | legs at the start, trunk 28 ms later, arms 51 ms later | **passes** |
| no arm muscle fires before he could have decided (253 ms) | 256 ms: 3 ms early | fails by 3 ms |
| the hands' first motion (130–220 ms before contact) | 243 ms | fails |
| the lead knee (2–26° at contact, straightening into it) | 47° at the start, 8° at −50 ms, 1° at contact: locked straight against its end (1,005 N·m, read at every step; 899 from 1 ms records). **Its hold-still torque does twice its extensor's work in that straightening (the review, below)** | fails |
| the front foot's greatest load (1.5–2.6 × body weight) | 0.51, 134 ms before contact; **nothing on it at contact** (the rear foot 0.20) | fails |
| in balance; does not jump; legs have not given way | his centre of mass heads 0.4 cm outside his feet; the feet never carry under 0.12 of his weight; ends at 0.89 of its height | fails by 0.4 cm; passes; fails by 1% |
| no joint more than 2° past its range | up to contact: the lead upper arm's turn 2.6° past, the lead knee 1.3°, the trunk's twist 1.1°. In the 50 ms after: the lead forearm 5.9° past its palm-down end, the lead wrist 3.1° past its ellipse, the lead arm's raise 2.2°; those three figures change with the step (the wrist 0.8, 3.1 and 3.9° at 0.5, 0.25 and 0.125 ms: the review, P3-2) | fails |
| a joint's end carrying a heavy load (D39, not built) | 15 ends over 50 N·m, read at every step *(corrected after the review, P3-1: 1 ms records understated the peaks)*: the trunk's side bend 1,204 (after contact; 327 before), the lead knee straight 1,005, the lead arm's raise 929 (after contact), the rear hip's turn 663, the trunk's twist 607, the rear hip bent back 528, the rear knee 482, the lead hip's turn 419; ligaments fail at 13–34. *What follows contact does not settle with the step (P3-2): the lead wrist's stop carries 675, 263 and 196 N·m at 0.5, 0.25 and 0.125 ms* | fails |
| no passive element does net positive work | muscles 417 J; the stops −51 J (−165 by 50 ms after), the ground −45 J; the joints' damping −266 J. **Not a passive element, and not held by any muscle rule: the hold-still torques, +157 J** (the review, P1-1; below) | **passes**, on its own terms |
| two starts that owe each other nothing, within 1 m/s | S 17.7 and C 10.1 m/s | fails |
| *reported:* the elbows at contact (79 ± 13, 53 ± 15°) | rear 22°, lead 54° | |
| *reported:* each wrist's place in its ellipse (the untilted one, [E]) | 0.96 and 0.96 at most to contact, read every millisecond *(corrected, P3-3)*: inside. In the 50 ms after: the lead 1.05, the rear 1.03 | |
| *reported:* muscles at full strength; rises at the 50 ms floor | none exactly on its bound, **nine of 41 at 92–97% of it**; no rise exactly on the floor, seven within 10 ms of it *(corrected after the review, P1-3)* | |
| *reported:* the rear foot | flat (heel 0.2 cm), turned 15° backward; the pelvis turned 42° | |

  - *For the eye:* closer to a swing than anything since session 8. The legs go first, then the trunk, then the arms; the bat comes through the ball nearly level (3.6° down); the front knee straightens hard. But it is still an arms-and-trunk swing off the back foot: the front foot carries nothing at contact, the hips open 42° at half a hitter's rate, the back elbow is nearly straight, and the barrel hangs 55° below the hands, 9° short of square.
- **What the viewing time does: not shown** *(rewritten after the review, P2-2; I had written that the search "makes the legs and trunk lead" and "uses the lower body in a hitter's way", which is more than unsettled runs carry)*. Watching 0.20 s the arms may fire only 153 ms before contact. In searches that had not finished, the trunk turned faster at 0.20 s than at 0.10 s in all three pairs (S 598 against 469°/s, K 615 against 377, S carried on 699 against 414); the hips did so in one, the continuation (544 against 361°/s; in the first runs 392 against 375). In that continuation the pelvis turned 72° (42°), the rear foot came up 1.3 cm and turned 18° toward the pitcher, the front foot took 1.39 times his weight (0.51), and the bat arrived at 17.5 m/s chopping down at 26°, 1.9 cm from the ball. **Three things stand between that and a finding:** it had not settled (7.0 over its last 50 generations); its arms still fire 12 ms before the clock allows, so the condition it is meant to test is not met; and 35% of its positive work is the hold-still torques', mostly at the knees and the rear hip (below). Whether the viewing time causes any of it is not shown. Nothing is put to the owner on it.
- **K: the rear calf.** Given half the calf's strength from 100 ms, the search kept 26% of it tensed and ended with the heel 0.3 cm up at 0.10 s and 1.4 cm at 0.20 s; its speeds at the same stage were 14.0 and 18.7 m/s against S's 16.3 and 19.2, in searches that had not settled, so the comparison is not a finding *(P3-7)*. The heel does not come up in any swing that meets the ball. No run shows the calf buying anything, and that is all that can be said.
- **C and D: the score's parked basin, three times of four.** With contact read at the nearest approach a bat brought to the ball and stopped scores the speed term's 45 and little else, and three of the four runs from the hand-written plans ended there (their speed along the line at the nearest approach −0.2 to −2.3 m/s; the plane's crossing is then 54–77 cm above the ball, as the bat falls). **C at 0.10 s is the exception and is a swing.** The second review of session 8 named this basin (its P2-4); it is not remedied here (below).
- **Against what I expected:**
  - *The search moves:* yes, further than I wrote. From S I expected under 150 by 350 generations; it reached 45 and then 32.5.
  - *Within 3 cm at 17–22 m/s, none inside the gate:* yes: 0.2 cm at 17.7 m/s.
  - *The hips under 450°/s:* at 0.10 s yes (361); **at 0.20 s no: 544°/s.** I had tied the hips' rate to the lead hip's range (part 2) and that was wrong as a limit on rate: the pelvis turned 72° there, with the rear foot turning.
  - *0.20 s 1–3 m/s slower than 0.10 s:* not shown (17.5 against 17.7, the one not settled).
  - *C and D come within 10 cm and neither beats S:* C meets the ball, which I did not expect; D and C at 0.20 s parked, which I did not foresee.
  - *K's heel comes up:* barely.
- **What part 3 shows, and what it does not.**
  - *Shown:* the body session 9 built, started off its joints' ends, swings: it meets the aim's ball within 0.2 cm at 40 mph, legs then trunk then arms, the same at a quarter of the step, with both wrists inside a wrist's reach and no passive part adding energy. The search was what stood between session 9 and this, as its review said.
  - *Shown:* the search no longer stalls. Ten searches, no failed run, every result the same at three steps.
  - *Shown:* a plan that owes [4] nothing can become a swing that meets the ball (C, 10 m/s). The start's posture is still [4]'s.
  - *Not shown: whether this body can reach the gate's 29 m/s.* *(Corrected after the review, P1-2 and P1-3. I wrote that the best swing has "no muscle group at its full strength and no rise at its floor", that "what holds it is not strength in any single muscle's number", and that "nothing found names the part that would stop it". The first is true only of "exactly on its bound": nine groups are at 92–97%, in a search that is charged for effort and does not settle on a box's edge. The second does not follow. The third is true only of the parts I looked at: the review looked at two I had not, the joints' damping and the hold-still torques, and both carry more of this swing than any of my three suspects has been shown to; "After the review", below.)* The search around the best swing has settled; what holds it at 17.7 m/s is not known. Three things I listed, none tested apart, and after them the review's two:
    1. *It leans on joint ends.* Fifteen carry 92–1,204 N·m (the lead knee's 1,005). The lead knee is thrown straight into its end at 870°/s and the trunk twists to its end; a swing that may not do that (D39) is a different swing, and nothing says whether it is faster or slower.
    2. *The lower body is hardly used at 0.10 s and is used at 0.20 s.* The one run that makes the hips and trunk turn near a hitter's rates is the one whose arms were made to wait; it had not settled.
    3. *The plan it came from.* S is session 8's arms-and-trunk swing, found from the fit to [4]; the search is local. C, from nothing, reached 10 m/s.
  - *Not shown:* anything settled about the viewing time; that the swing is a hitter's; that it balances (it fails the test by 0.4 cm and the front foot is off the ground at contact).
- **The score's parked basin: not remedied, and what the remedy is** (stated, as the job allowed only with time left). Three of four hand-written starts ended in it. The remedy the second review of session 8 asked for, unchanged: read contact at the nearest approach only until the sweet spot has crossed the ball's plane once in the run, and score a bat that reaches the ball without crossing its plane as having the speed it has there *along the line*, charged the full speed term plus what it is short of crossing; or, simpler with this search, start the whole-aim stage only from a reach result that crosses the plane, and reject a plan that does not. Before any run of it: what a parked bat scores under the old rule and the new, worked by hand.

#### After the independent review of session 10 (2026-11-26): what it measured, and what is changed

[The review](reviews/2026-11-26-opus-body-3d-s10-review.md) confirms what was built (the ellipse and its stop, the cleared start, the search, the kept swing at all three steps and in both engines, every stop's net work negative, part 2's "did not decide") and corrects the account on three things that bear on why the swing sits at 40 mph. I disagree with none of its findings: each figure of its P1-1, P1-2 and P1-3 that I re-measured came out the same. **No search is rerun here.** The two kept results are re-read with the report as it now is (`practice3d.py --show … --update`; outputs `sim/target/codex-probes/body3d/s10/reread_*.out`); corrections to sentences above are marked where they stand.

**1. The hold-still torques give the swing over a quarter of its energy, and no muscle rule holds them (P1-1).**

- *What they are:* `drive.solve_tone`'s one constant torque at each joint, fixed in the parent's frame for the whole run: 69 N·m at the lead knee, 60 at the rear knee, 46–63 at the hips and the trunk, 26 at the rear ankle, 2–8 elsewhere. They stand for the muscles that hold the launch posture ([E], session 2). They have no strength-at-speed law, no onset and no rise, are not counted as effort and are not under the clock. The engine books their work in one integral with the joints' damping, and every result until now printed only the sum.
- *Now reported apart, by joint, in every result* (`practice3d.hold_work`, read at every step; the test works it on a rod with a constant torque and a capped damper, against the torque times the angle turned). To contact:

| | S, 0.10 s (the best) | S, 0.20 s | C, 0.10 s |
| :-- | --: | --: | --: |
| the muscles | +416.5 J | +287.5 | +278.6 |
| **the hold-still torques** | **+156.8 J** | **+157.4** | **+125.0** |
| their share of what they and the muscles put in | **27%** | **35%** | **31%** |
| the joints' damping | −265.7 J | −187.4 | −191.4 |
| the two together, and the engine's own integral of both | −108.8; −108.8 | −30.0; −30.1 | −66.4; −66.1 |
| the stops; the ground (the engine's integrals) | −51; −45 J | −55; −41 | −56; −32 |

  (My re-read gives the review's figures to the tenth of a joule in all three. The review's own table closes the account with gravity, +62 J in the best swing, and a kinetic energy gain of 274 J.)
  - *By joint, the best swing:* lead knee +58.1 J, rear knee +49.9, rear hip +39.6, rear ankle +10.2, the trunk −9.2, the rest under 8 each. At 0.20 s: lead knee +58.0, rear knee +49.7, rear hip +30.5.
  - **At the lead knee the hold-still torque does about twice the knee extensor's own work** (+58.1 against +30.5 J, the review's figure for the muscle): it goes on pushing at 69 N·m while the knee straightens at 870°/s, where a real extensor gives almost nothing. "The front knee straightens hard" is mostly this torque.
  - In three of the other runs (C at 0.20 s, D at both viewing times) the lumped figure is positive (+44, +20, +40 J): there the hold-still torques outweigh all the damping.
- *What it changes in what was said:* "no passive element adds energy" holds, and these are not passive; but they stand for muscle and escape every rule the muscles are under, so **no statement that this swing stays inside its muscles' strength at their speeds, or inside the clock, covers them**. The swing's legs-then-trunk-then-arms order and the lower body's part at 0.20 s are read with a quarter to a third of the energy coming from torques that were on from the first instant.

**2. The joints' damping takes two thirds of the muscles' work, and its size has no source (P1-2).**

- *What it is:* `drive.JOINT_DAMP`, 1.0 N·m·s/rad about every axis of every joint, its torque capped at 30 N·m: an [E] from session 1 with no source and, until now, no stated range.
- *What it takes, the best swing to contact:* 265.7 J, 64% of the muscles' 416.5 J and about as much as the body's whole gain in kinetic energy (274 J). The arms lose 196 J of it: rear shoulder −69.6, rear elbow −39.5, rear wrist −32.1, lead shoulder −26.0, lead wrist −23.6. Its torque reaches 29 N·m at the rear elbow and 16 N·m at each wrist, where the forearms' muscles have 17–18 N·m and the wrists' 24–51.
- *The review's replay of the same plan at lower damping* (its `damp.py`, not rerun here; **indicative only**, rule 4: the plan was tuned at 1.0 and no longer meets the ball): the sweet spot's fastest speed before contact 19.25 m/s at 1.0, **22.46 at 0.3**, 23.00 at 0.1, 23.26 at none. So this one unsourced number is worth 3–4 m/s on this plan before any search adapts to it, more than anything I listed has been shown to move.
- *A source, looked for and not secured* (2026-11-26; search summaries only, no paper opened: the two fetches tried were refused): a study of the relaxed wrist under perturbation reports damping of 0.02–0.06 N·m·s/rad; arm studies give the elbow about 0.2–0.4 and the shoulder about 0.7 N·m/(rad/s) while a posture is held (Gomi & Osu 1998 and the earlier single-joint work it cites); ankle figures of 0.2–2.4 are during walking, with the muscles working. **None is read, so the value stays unsourced and is flagged so in `drive.py`.** They point the way the review did: at the wrists and elbows 1.0 is ten to fifty times what a relaxed joint is reported to have; at the hips, knees and trunk nothing was found. *Its range until a source is read:* [E: 0.02–1.0, by joint].
- *What it changes in what was said:* "nothing found names the part that would stop it" was true only of the parts I looked at. Part 2's audit ("3.4 times") leaves damping out: the 17.7 m/s swing already loses 266 J to it, and the loss grows with the square of the joints' speeds.

**3. Nine muscle groups work at 92–97% of their strength (P1-3).** In the best plan: the rear shoulder's sweep 0.974 of its bound, the rear forearm's supinators 0.972, the trunk's rotators 0.968, the lead forearm's pronators 0.968, the rear hip's adductors 0.957, the lead shoulder's internal rotators 0.948, the rear shoulder's external rotators 0.945, the lead arm's raiser 0.944, the rear wrist 0.918; seven of the nine reach full activation. Seven rises are within 10 ms of the 50 ms floor (the rear shoulder's 52 ms). Two things keep a result just off its bounds: the score charges effort (0.83 of the best's 32.5), and a search that ranks plans drawn with a spread of 0.0065 of each number's range does not come to rest exactly on a box's edge. **So "no muscle at its full strength" was a statement about a test for exact equality, and "strength does not hold it" does not follow. Whether he is short of muscle is not known.** Every report now lists the groups at 90% or more of their bounds with their largest activations, and the rises within 10 ms of the floor.

**4. Smaller corrections, each made where it stands.**

- *P2-1:* the "ridge" explanation of session 9's stall is withdrawn (above).
- *P2-2:* the 0.20 s run is described as what unsettled runs show (above).
- *P2-3:* "the hips cannot face the pitcher" rests on the lead foot's place and angle at [4]'s −250 ms, the passive mean of a hip's turn (an active range gives 11° more), a knee and ankle that do not twist, and a compass search one of whose starts ended 13° outside; and the comparison is with [4]'s 77 ± 6°, one SD away. §4.12 said the first four; "For the owner" did not, and does now.
- *P3-1, P3-2, P3-3:* joint ends' loads are read at every step (the lead knee 1,005 N·m, not 899; the lead hip's turn 419, not 376); what follows contact does not settle with the step and is said so; the wrists' place is read every millisecond and at contact itself.
- *P3-4:* the two-engine check's ellipse case is 200 ms again, judged as the cleats case is (within ten times the Python engine's own difference from itself, and to 10⁻⁹ where that is larger, which still holds the first 100 ms to the plain tolerance); the cut to 100 ms had dropped the lead wrist's longest engagement (165–184 ms). The explanation is corrected: the 1.3 × 10⁻⁸ is in the stops' booked-work integral, a state entry, not in the motion (the bodies' own entries differ by at most 2 × 10⁻⁹ and fall back to 2 × 10⁻¹⁰). The check now prints the bodies' own entries apart.
- *P3-5:* the start's rule gives a least change number by number, by a compass search; "the least" is corrected in the code, the test and above.
- *P3-6:* that both wrists start inside a real wrist's reach rests on the untilted ellipse ([E]; on one tilted 8° the rear wrist starts at 1.02); "For the owner" says so.
- *P3-7:* the research index's line on the rear calf is reworded.
- *P3-8:* the search's test is renamed for what it checks (a known minimum on a stepped score with failing runs); nothing in it shows a slope method would miss that minimum.
- *P3-9:* "one session will not close 11 m/s" was a judgement and is stated as an expectation.
- *P3-10:* session 9's start had five joints within 5° of an end (six readings, the rear wrist's two angles among them), not "six joints".

**What is left for the last session is reordered by this** ("Status and what is left"): the damping at a read value and a hold that lets go or follows the muscles' law come first, each as one change with a search run until it has settled; no such search is run in this job.

**The checks after the review's changes, every changed source touched first** (job 410; `sim/target/codex-probes/body3d/s10/fix_*.out`): `test_body3d.py` 72 tests, OK (the one new is the hold-still split's); `test_oracle.py` 112, OK (8 expected failures); `built3d.py --check` PASS; `bin/check-docs.py` clean; `probe3d.py --check` PASS, worst 3.6 × 10⁻¹¹ over the seven plain cases, and the two sensitive cases each within ten times the Python engine's own difference from itself at every record of their 200 ms. On the ellipse case the engines are 1.4 × 10⁻⁸ apart at most in the whole state and **2.2 × 10⁻⁹ in the bodies' own entries**, against the Python engine's 1.5 × 10⁻⁸ from itself: the review's reading, reproduced. The two kept results re-read give the figures above (17.69 m/s, 0.2 and −0.1 cm; 17.5 m/s, 1.7 and 0.9 cm).

**The checks, after touching every changed source** (jobs 405 and 406; outputs `sim/target/codex-probes/body3d/s10/final_*.out`):

- `python3 -m unittest analysis/oracle/test_body3d.py`: 71 tests, OK (62 before the session; the nine new are `OvalChecks` 5, `StartChecks` 3 and the search's own). `test_oracle.py`: 112, OK (8 expected failures, as before).
- `probe3d.py --check`: PASS, worst 3.6 × 10⁻¹¹ against 10⁻⁹ over eight plain cases, the new one among them (the wrists' ellipses with every drive at its bound: 2.5 × 10⁻¹¹), and the sensitive cleats case within ten times the Python engine's own difference from itself. **The first run of the check failed on the new case** (1.4 × 10⁻⁸ over 200 ms). It is the motion's sensitivity and not a difference of laws (`s10/oval_diverge.py`): with the rear wrist's ellipse engaged at 30 ms and again at 90 ms (1,275 N·m) the engines agree to 6 × 10⁻¹²; between 100 and 120 ms the difference grows five-hundredfold and from then on tracks the Python engine's difference from itself under a 10⁻¹³ nudge (1.29 × 10⁻⁸ against 1.50 × 10⁻⁸ at every record). The case was cut to 100 ms. *(Changed after the review, P3-4: it is 200 ms again under the cleats case's rule, and the plateau is in the work integral, not the motion; "After the review", above, and the checks at its end.)*
- `built3d.py --check`: PASS. `bin/check-docs.py`: clean.
- The two kept results replay: `practice3d.py --show analysis/research/body/practice3d_s10_S_view100.json` gives 17.69 m/s, 0.2 and −0.1 cm.

### 4.13 What each unmeasured part is worth in bat speed: a ladder of single changes (session 11)

Session 11 (2026-11-26) works under the owner's rulings of that day ([PLAN D42](../PLAN.md)): the cap of three sessions is lifted and the work goes on while each session gains honest bat speed ("honest": speed that comes from a measured part of the body, not from an unmeasured allowance, a bug or a loophole in the score); joint friction takes sourced values where a paper can be read and a stated range, run at several values, where none can; what is left of the lab study goes from the stance; no strength without a measurement (D34), and if he is still short the owner is told which muscles and by how much; **the fixes are made one at a time, with a measured bat speed after each.** The gate is §13's, unchanged. Written in the order the work was done; each rung is written here before it is built or run.

**The ladder's table** (kept up to date; speeds are the sweet spot's, in settled searches from the same plan by the protocol below, two searches a rung with different random draws; the rung's figure is the better-scoring search's; "gain" is at contact, over the rung before, and counts only if it is larger than the 1.95 m/s (4.4 mph) by which rung 0's own two searches differ, or if both of the rung's searches beat both of the rung before's):

| Rung | The one change | At contact: the rung's figure; the other search | The bat's fastest before contact (the same two) | Gain at contact over the rung before | What it rests on |
| :-- | :-- | --: | --: | :-- | :-- |
| 0 | none: session 10's body | **17.7 m/s (39.6 mph)**; 15.7 (35.2) | 19.2; 17.8 m/s | — (the two differ by 1.95 m/s: the yardstick) | unsourced friction (1.0 N·m·s/rad everywhere), constant hold-still torques, [4]'s stance and launch state |
| 1 | joint friction at the values read; middles of stated ranges at the hips and trunk | **14.7 m/s (32.9 mph)**; 14.9 (33.2) | **20.4; 19.7 m/s** | **−3.0 m/s (−6.6 mph): a loss at contact, both searches under both of rung 0's.** The bat is faster before contact (+1.2 and +1.9 m/s) and slows by a quarter in the last 20–30 ms as an arm reaches its elbow's end | read: wrist's bend, shoulder, knee, ankle; the elbow with the arm holding its place; **not measured: hip, trunk** (150 J of the swing's losses), forearm's turn, wrist's tilt |
| 2 | on rung 1's body: the posture held by his muscles, under their rules, in place of the constant hold-still torques | **16.9 m/s (37.7 mph)**; 16.6 (37.1) | 18.6; 19.3 m/s | **+2.1 m/s (+4.8 mph) over rung 1: a gain by the rule** (both searches over both of rung 1's); against rung 0: −0.8, no change shown | measured strengths and strength-at-speed curves; Schultz 1987's trunk bending strength; 12 N·m at the ankles' roll left as constant torque; the hips' and trunk's friction not measured |
| 3 | the feet where major leaguers' are (1.03 m apart, level, the front foot 44° open), the ball level with the front ankle; the rest of the start by its rule | **21.5 m/s (48.2 mph)**; 20.7 (46.2) | 22.4; 21.3 m/s | **+4.7 m/s (+10.5 mph) over rung 2; +3.8 m/s (+8.6 mph) over rung 0: a gain,** both searches over both of every rung before | Statcast's foot points at the intercept (my reading of the page's data, checked against its published column); the posture's other numbers still [4]'s, bent by the rule to within 4.0° of four joint ends |
| 4 | the hips' ends of turn by their own muscles, measured with the hip straight (in 32°, out 44°; they were 32.6° and 33.6°) | 21.1 m/s (47.2 mph); 20.9 (46.7) | 22.0; 21.4 m/s | −0.4 m/s: **no change shown** (both searches inside rung 3's pair) | Simoneau 1998's 21 college men; Boone & Azen's 44° both ways was measured seated and is not used |
| (2 alone, on rung 0's body) | the same change without rung 1 | 16.7 m/s (37.3 mph); 15.7 (35.2) | 18.8; 17.7 m/s | −1.0 m/s against rung 0: **no loss shown** (inside the yardstick; both searches inside rung 0's pair) | measured strengths and strength-at-speed curves; Schultz 1987's trunk bending strength; 12 N·m at the ankles' roll left as constant torque |

**Where the session stands while this is written** (for whoever picks it up if it is cut short): every rung through 6 is designed below and committed before its runs; the runs are in the job queue (`bin/job.sh status`, labels `body-3d-11 …`) and write to `sim/target/codex-probes/body3d/s11/<rung>.out` and `.json`; `ladder3d.py --table` and `--block <rung>` print what has finished.

#### The protocol, the same for every rung (written before any run)

- **The body of rung *n* is the body of rung *n* − 1 with one thing changed.** Rung 0 is session 10's: hands, cleats, a tensed muscle that holds, the wrists' ellipses, the cleared start (still [4]'s launch posture moved by §4.12's rule, flagged), the 0.10 s viewing time.
- **The search is session 10's, from the same plan every time:** the evolution strategy of §4.12, every run at 0.25 ms, from start S (session 9's nearest plan, which came from the fit to [4]: flagged), 100 generations of the reach stage, 250 of the whole aim, then 500 more from that result. That is exactly how session 10's 17.7 m/s was found. If the best score has fallen 1.0 or more over its last 50 generations it is carried on in blocks of 250 (two at most) and reported as not settled if it still has.
  - *Why from S every time and not from the rung before's result:* so that every rung gets the same search from the same plan. A chain of searches each starting where the last ended would give later rungs more searching, and a gain could be that.
  - *What it gives up:* S was tuned on the body of sessions 8 and 9. On a body far from that one it may be a poor place to start, and the search is local. Where a rung's search from S ends clearly worse than the rung before, the rung before's result is also carried on for 500 generations on the new body, and both are printed.
- **The spread.** Rung 0 is run twice from S with different random draws (session 10's own run, seeds 1 and 11; the same again with seeds 2 and 12). The difference of the two is the spread of two searches from the same start. Two runs make a poor estimate of a spread; it is the yardstick the job asks for, and where a gain is within twice it I say so.
- **Dependence runs** (a value moved across its range; one muscle group scaled) are settled searches too, not replays: the rung's result carried on for 500 generations on the changed body, **beside the same result carried on for the same 500 generations on the unchanged body** (so the comparison is between equal amounts of searching).
- **Rule 6's checks with every result** (§4.12's, printed by the search): the result at 0.5, 0.25 and 0.125 ms; the score along the line from the starting plan to the result; slopes forward and backward; whether it had settled; failed runs.
- **After every rung:** the sweet spot's speed at contact in m/s and mph and the gain; the distance from the ball; every gate check with its margin; the work of every element to contact, by joint: muscles, hold-still torques, the joints' damping, the stops, gravity, the ground, against the body's gain in kinetic energy (`ladder3d.py`, `energy`: each read at every step; on session 10's kept swing it gives the review's figures: muscles +416.5 J, hold-still +156.8, damping −265.7, stops −51.4, ground −45.2, gravity +62.2, kinetic energy gained 273.7, the sum closing to 0.5 J).
- **Before each run** I write what I expect and what the result would and would not show.

Code: `analysis/research/body/ladder3d.py` (the protocol, the energy account, the gate's rows; results under `sim/target/codex-probes/body3d/s11/`).

#### Rung 0: session 10's swing, searched for again (the baseline and the spread)

- *What is run:* nothing is changed. The same search from the same plan S with other random draws (job 412: `practice3d.py --cma S --seed 2`, then 500 generations on with seed 12; started before this section was written, as the job's first act, since it changes nothing).
- *What I expect:* a swing that meets the ball within 1 cm at 16–19 m/s. The first search fell 0.65 over its last 50 generations, so it was still creeping; two searches of 205 numbers from one plan with different draws need not end at the same plan, and I expect them 0.3–1.5 m/s apart.
- *What it would and would not show:* the difference is the yardstick for every gain below. It would not show the body's best at this rung: both searches are local, from one plan.

**Rung 0 as run** (job 412, 1,061 s; `s11/r0b.out`, `r0b_cont.out`, re-read with the ladder's report in `s11/r0_b_show.json`). **The same search from the same plan with other random draws ends 1.95 m/s (4.4 mph) slower, at a different swing.**

| | Session 10's search (seeds 1, 11) | The same again (seeds 2, 12) |
| :-- | --: | --: |
| the sweet spot at contact | **17.69 m/s (39.6 mph)** | **15.74 m/s (35.2 mph)** |
| the miss | 0.2 cm | 0.9 cm |
| the score (lower is better) | 32.5 | 39.6 |
| settled: the fall over the last 50 generations | 0.65 | 0.07 |
| the same at 0.5, 0.25, 0.125 ms | 17.69 at all three | 15.74 at all three |
| the attack angle | −3.6° | −8.5° |
| the hips'; the trunk's fastest turning | 361; 414°/s | 258; 421°/s |
| the lead knee at contact; the front foot's greatest load | 1°; 0.51 × body weight | 40°; 1.08 |
| muscles; hold-still torques; damping; stops; ground; gravity (J) | +417; +157; −266; −51; −45; +62 | +439; +82; −269; −37; −22; +99 |
| kinetic energy gained | 274 J | 293 J |
| groups at 90% or more of their bounds | 9 | 7 |

- **The spread of two searches from the same start is 1.95 m/s.** Wider than the 0.3–1.5 I expected. Both have settled by the rule, both are the same at three steps, neither has a failed run; they are two different swings (the second keeps its front knee bent 40° and takes half as much from the hold-still torques, mostly at the rear hip). The search is local and the score has many basins; "settled" says a search has stopped, not that it found the best.
- **What this does to the protocol (changed here, before rung 1 is run): every rung is searched twice from S, with the same two sets of draws (seeds 1, 11 and seeds 2, 12), and the rung's figure is the speed of the search with the better (lower) score**, since the score is what the search works on; choosing by speed would pick the luckier draw. Both are printed. For rung 0 that is 17.69 m/s. **A gain is called a gain only if it is larger than 1.95 m/s, or if both of a rung's searches are faster than both of the rung before's;** anything less is reported as no gain shown. Two searches a rung are still few: the yardstick is rough and is said so wherever a figure is near it.
- *The line check* (rule 6) for the second search's last 500 generations: 51, 46, 45, 44, 45, 45, 44, 43, 42, 40, 40; beyond 40: flat at the end. Its slopes and the first 350 generations' checks are in the output.

#### Rung 1: joint friction from what was measured (the values, written before any search)

**What was read** is [research note 14](research/body/14-joint-damping.md): 30-odd sources, with how each was read. Two search workers opened the papers; I checked one table against the saved text (Amankwah's). Two things in them decide how the values below are used:

- **A joint's damping grows with how hard its muscles work, and that part is already in this body.** Every source that measured both has relaxed damping the least, rising with effort (the ankle two to four times by 30% of full effort; the knee from 1.0 to 2.9 with co-contraction; the elbow by 0.18–0.19 N·m·s/rad for each N·m held). That added damping is a working muscle's own loss of force with speed, seen from outside, and the drives carry it: the elbow extensor's strength-at-speed curve of §4.5 loses 0.23 N·m per rad/s for each N·m it gives at low speed, against Gomi & Osu's measured 0.18–0.19. **So the joints' own damping takes the relaxed joint's value** (a modelling choice: a value that followed activation would count the muscles' part twice; a relaxed value with the muscles' curves counts it once).
- **The linear coefficient overstates a relaxed joint at a swing's speeds.** Measured with movements of a few degrees, it falls as the movement gets larger or faster (the wrist's by 73% from 0.4° to 6°; relaxed legs moved through their range resist with a torque that grows as speed to the power 0.08–0.2). No source measured a joint's damping at 500–1,500°/s. The body keeps a linear damper, the form the values come in; they are upper readings on that side, and the range's low ends say what less would be worth.
- **The cap is dropped.** Sessions 1–10 capped the damper at 30 N·m, with no source. At the values below no read joint's damper comes within a factor of five of it, so it did nothing but hide the law; a relaxed joint's resistance probably does level off with speed (above), but nothing read gives the level for the arms or trunk.

**The values** (`drive.DAMPING`; N·m·s/rad; where a joint's two movements can be told apart in its parent segment's frame they have their own value):

| Joint, movement | Was | Now | Source and condition | Read? Range if not |
| :-- | --: | --: | :-- | :-- |
| wrist, bent forward and back | 1.0 | **0.04** | relaxed: the mean of four studies as Salmond 2017 tabulates them (0.014–0.09); Lakie 1984's relaxed wrists 0.07–0.14 off its figure | read |
| wrist, tilted toward or away from the thumb | 1.0 | 0.09 | **never measured** (Peaden & Charles 2014: "unknown"); their assumption, scaled from stiffness | **not measured:** 0.03–0.20 [E: from the bend's value to twice the assumption] |
| forearm's turn | 1.0 | 0.03 | **never measured**; the same authors' assumption 0.024 | **not measured:** 0.02–0.10 [E: as the wrist's range] |
| elbow's bend | 1.0 | 0.43 | Tsuji 1995, four men "asked to relax" the arm but holding its place, reflexes in: 0.22–0.63; an upper reading | the truly relaxed elbow **not read** (two papers closed): 0.22–0.97 (Tsuji's least; Gomi & Osu's largest zero-torque value) |
| shoulder | 1.0 | **0.47** | Lipps 2020, at rest: 0.61 raised, 0.33 turned about the upper arm, 0.46 swept level; their mean | read |
| knee | 1.0 | **0.25** | the relaxed lower leg left to swing (Casabona 2012: 0.18 swinging down, 0.31 up) | read; small perturbations about a held angle give 1.0–2 (a fivefold disagreement between methods; the free swing is the nearer to a swing) |
| ankle, bent; rolled | 1.0 | **0.40; 0.165** | Lee, Krebs & Hogan 2014, relaxed: 0.34 and 0.45; 0.15 and 0.18 | read |
| hip | 1.0 | 2.0 | relaxed subjects 2–21 from 1° pulls (Huang 2020, a poor fit by its authors' account); the swinging leg in walking 1.9–4.6; a relaxed hip moved through its range resists with 14–17 N·m at 300–700°/s (Amankwah's fit carried up from 90°/s), 1.4–2.7 as a coefficient. Nothing for the hip sideways or about the thigh | **not read as a relaxed mean:** 0.5–4.6 |
| trunk on the pelvis | 1.0 | 5.0 | relaxed sitting, forward and back: 4–7 (second hand, my lever arm); with the trunk's muscles holding a load: 27.5 ± 9.0 (Moorhouse & Granata 2005); the least quick-release mean 1.2. **Nothing for the twist relaxed** | **not read:** 1.0–27.5 (the least mean; a trunk holding a load, which a relaxed one cannot exceed) |

- **So the read values are far below 1.0 in the arms and legs and the unread ones are above it at the hips and trunk.** Of the 266 J the old damping took in session 10's swing, 196 J was in the arms, where the read values are a half (shoulder, elbow's bend) to a twenty-fifth (wrist) of it; 30 J was at the hips and 13 J at the trunk, where the middle of the range is two and five times it.
- *Not from any source, and said so:* the middle of an unread range is not a measurement. The hips' 2.0 and the trunk's 5.0 are where I put the middle (Huang's least relaxed subject and the bottom of the walking leg's range; relaxed sitting); D34's rule applies to them: flagged, never passed off as measured, and the dependence runs below say what hangs on them.

**The integrator at the lower damping, before any search** (`s11/damp_integ.py`, `damp_integ.out`): on session 10's kept plan and on a plan with every muscle at its bound, at the old damping and at the central, low and high sets: **the two engines agree over 200 ms at 0.25 ms to 1.4 × 10⁻¹¹ or better in every case** (the old damping: 4.6 × 10⁻¹²), and **the kept plan's replay is the same at 0.25 and 0.125 ms to 0.03 m/s and 0.1 cm at every set** (at 0.5 ms it differs by 0.2–0.3 m/s, as §4.12 found at the old damping). Tests: a rod's speed against a damper falls as the worked exponential and the damper's booked work is the kinetic energy lost; the table is read as written; an elbow turning about its hinge alone meets the bend's coefficient and about the forearm's length alone the turn's (`LadderChecks`).

**The replay of session 10's kept plan, no search** (indicative only, rule 4; the plan was tuned at 1.0): at the central set the sweet spot crosses the ball's plane 6.5 cm high at 19.7 m/s; at the low set 7.3 cm high at 20.8; at the high set 5.4 cm off at 15.6 (the old damping: 17.7).

**What I expect** (rung 1: the central set, from S by the protocol, two searches):

- *The search:* it meets the ball within 1 cm. **Faster than rung 0 by 1–3 m/s** (18.5–21 m/s for the better-scoring search): the replay gains 2.0 m/s before any search adapts, the review's replay at 0.3 everywhere gained 3.2, and here the hips and trunk are damped more than before, not less. With the spread of two searches at 1.95 m/s, a gain of 1–3 may not clear the yardstick; whether both of rung 1's searches beat both of rung 0's (17.7 and 15.7) is the stronger test.
- *The damping's work:* well under the old 266 J; I expect 80–140 J, most of it now at the trunk and hips.
- *What it would show:* what the unsourced 1.0 was costing, at values read where they could be read. *What it would not show:* the body's speed with the true friction: the hips' and trunk's values are middles of ranges nobody has measured, and the dependence runs are what say how much rests on them.

**The dependence, as the owner asked** (after the rung's searches): rung 1's better result carried on for 500 generations on the body with the unread joints' values at the low ends, at the middle (unchanged) and at the high ends, all together; then, if the bat's speed moves by more than the yardstick, one group at a time (the arms' unread values, the trunk's, the hips'), each at its low and high end with the others in the middle.

**Rung 1 as run** (jobs 415 and 416; `s11/r1.out`, `r1_b.out`, `.json`; the first search needed one extra block of 250 generations to settle). **With the joints' friction at the values read, both searches meet the ball slower than both of rung 0's: 14.72 and 14.85 m/s (33 mph) against 17.69 and 15.74. But the body is faster, not slower: the bat reaches 20.4 and 19.7 m/s before contact (rung 0: 19.2 and 17.8), the hands 11.2 and 11.0 m/s (7.6 and 7.5; the gate's band is 10.3–13.0), and it then loses a quarter of the bat's speed in the last 20–30 ms, when an arm runs straight into its elbow's end.**

| | seeds 1, 11 | seeds 2, 12 |
| :-- | --: | --: |
| the sweet spot at contact | **14.72 m/s (32.9 mph)** | **14.85 m/s (33.2 mph)** |
| the miss | 1.0 cm | 0.8 cm |
| the score (lower is better) | 37.6 | 37.7 |
| settled: the fall over the last 50 generations | 0.39 | 0.15 |
| the speed at 0.5, 0.25, 0.125 ms | 14.72, 14.72, 14.74 | 14.60, 14.85, 15.01 |
| failed runs among the plan's 205 one-number changes | 0 | 0 |
| the line from the starting plan to the result (11 points, then one beyond) | 343 290 264 248 225 170 140 111 81 51 38 83 | 343 299 255 225 188 155 111 81 59 44 38 45 |
| the bat at its fastest at contact | 72% | 75% |
| the hands' fastest | 11.2 m/s | 11.0 m/s |
| the attack angle | -2.6° | -2.4° |
| the hips'; the trunk's fastest turning | 511; 598°/s | 437; 345°/s |
| the pelvis's turn from the start to contact | 48° | 34° |
| first rises: legs, trunk, arms (ms before contact) | 306, 263, 255 | 299, 257, 259 |
| the first arm muscle before contact (253 allowed) | 255 ms | 259 ms |
| the lead knee at contact; the front foot's greatest load | 1°; 1.08 x body weight | 1°; 0.35 x body weight |
| balance: outside his feet; least on the feet; height kept | 2.0 cm; 0.12; 0.88 | 2.0 cm; 0.03; 0.88 |
| joints past their range: to contact; to 50 ms after | 4.2° (the lead upper arm's turn); 4.2 (the lead upper arm's turn) | 3.9° (the lead upper arm's turn); 3.9 (the lead upper arm's turn) |
| joint ends over 50 N·m; the heaviest | 15; 772 N·m (elbow R) | 16; 1229 N·m (shoulder up L) |
| groups at 90% or more of their bounds | 12 | 9 |
| WORK TO CONTACT, J: muscles | +489 | +656 |
| holding muscles | +0 | +0 |
| hold-still torques (constant, under no muscle rule) | +185 | +177 |
| the joints' damping | -234 | -274 |
| stops; ground; gravity | -63; -38; +83 | -84; -37; +73 |
| kinetic energy gained | 426 | 525 |
| muscles (with holding) by joint, the five largest | hip R +233, hip L +79, shoulder R +77, lumbar +68, shoulder L +38 | hip R +264, hip L +136, lumbar +111, shoulder L +82, shoulder R +74 |
| damping by joint, the five largest | lumbar -80, hip L -33, hip R -30, shoulder R -29, elbow R -25 | lumbar -86, hip L -47, shoulder R -46, hip R -31, elbow R -27 |
| hold-still torques by joint, the three largest | hip L +84, knee L +58, knee R +29 | hip L +105, knee L +58, lumbar -30 |
| GATE: meets the ball | passes (-0.0 cm) | passes (-0.2 cm) |
| GATE: the sweet spot at contact | fails (short by 14.3 m/s) | fails (short by 14.1 m/s) |
| GATE: the bat at its fastest at contact | fails (-26 points) | fails (-23 points) |
| GATE: the hands' fastest | passes (inside by 0.9 m/s) | passes (inside by 0.7 m/s) |
| GATE: the attack angle | passes (inside by 0.4°) | passes (inside by 0.6°) |
| GATE: the hips' fastest turning | fails (short by 44°/s) | fails (short by 118°/s) |
| GATE: the trunk's fastest turning | fails (short by 157°/s) | fails (short by 410°/s) |
| GATE: first rises legs, trunk, arms | passes (-306, -263, -255 ms) | fails (-299, -257, -259 ms) |
| GATE: no arm muscle before he could decide | fails (+2 ms) | fails (+6 ms) |
| GATE: the hands' first motion before contact | fails (over by 11 ms) | fails (over by 12 ms) |
| GATE: the lead knee at contact | fails (short by 1°) | fails (short by 1°) |
| GATE: the front foot's greatest load | fails (short by 0.42 x body weight) | fails (short by 1.15 x body weight) |
| GATE: in balance | fails (+2.0 cm) | fails (+2.0 cm) |
| GATE: does not jump | passes (+0.02) | fails (-0.07) |
| GATE: legs have not given way | fails (-0.02) | fails (-0.02) |
| GATE: no joint more than 2° past its range | fails (+2.2°) | fails (+1.9°) |
| GATE: no joint end heavily loaded (D39, not built) | fails (15 ends over 50 N·m; the most 772 N·m (elbow R)) | fails (16 ends over 50 N·m; the most 1229 N·m (shoulder up L)) |
| GATE: no passive element does net positive work | passes (stops -63 J, ground -38 J, damping -234 J) | passes (stops -84 J, ground -37 J, damping -274 J) |

- **By the ladder's rule: 14.72 m/s, 2.97 m/s (6.6 mph) less than rung 0, with both searches under both of rung 0's. A loss at contact, larger than the yardstick.**
- **What happened, read from the two swings.** In the first the sweet spot is at 18.6 m/s 50 ms before contact, 20.4 at −20 ms and 14.9 at contact; the lead elbow is dead straight at contact (2°) and the heaviest joint end is the rear elbow's (772 N·m). In the second the rear elbow is the straight one (2°). With the arms' friction at a half to a twenty-fifth of what it was, the arms fly out, reach the end of the elbow's range before the ball, and the end takes the speed (the stops' work is −63 and −84 J to contact where rung 0's was −51 and −37). The bat is at 72% and 75% of its fastest at contact (rung 0: 92% and 89%; the gate asks 98%).
- **This is a swing the search could not time, not a slower body.** The body gains 426 and 525 J of kinetic energy by contact (rung 0: 274 and 293), the hips turn at 511 and 437°/s (361 and 258) and the trunk at 598 and 345 (414 and 421). The score reads the bat's speed along the ball's line at contact and nothing before it; from S, both searches settled on swings that are past their peak when the ball arrives. Meeting the ball 20 ms earlier in the same motion would be worth about 5 m/s; neither search found that.
- **The friction's work did not fall as I expected** (−234 and −274 J; I expected 80–140): it moved. The arms' friction took 196 J in rung 0 and takes 75–95 J now; the trunk's takes 80 and 86 J (13 before) and the hips' 64 and 78 J (30). **So about 150 J of this swing's losses now sit at the two joints whose values nobody has measured** (the trunk at 5.0 and the hips at 2.0, the middles of their ranges). The dependence runs below are for that.
- **Against what I expected:** wrong on the figure that counts. I expected 18.5–21 m/s at contact; the bat does reach 19.7–20.4 m/s, before contact, and arrives at 14.7–14.9. I did not foresee that lighter arms would outrun the plan's timing into the elbows' ends. Wrong too on the friction's total.
- *What it shows:* at read friction this search, from this plan, does not turn a faster body into a faster bat at the ball. *What it does not show:* that the read friction costs bat speed in a swing that is timed. The chain below, which carries rung 0's own swing onto this body, is the other reading.

#### A second measure of every rung: one swing carried through the changes (added after rung 1's first search, before any of it was run)

**Why.** Rung 0's two searches from S differ by 1.95 m/s, rung 2 alone's by 0.91, and rung 1's first search (below) ended 3.0 m/s *under* rung 0's with the same draws, at a different kind of swing. A search from S draws a basin as much as it measures a body. If every rung's effect is smaller than that, the protocol above cannot show any of them, however many rungs are run. So each rung is also measured a second way, which holds the swing fixed and changes only the body:

- **The chain.** It starts from rung 0's better swing (session 10's kept 17.69 m/s). Rung 1's body is searched on *from that swing* for 500 generations (the whole aim, the same draws every time, no extra blocks); rung 2's body from where that ended; and so on to rung 5. Each step is a settled search, not a replay: the plan adapts to the new body.
- **The reference chain.** The same swing carried on for the same 500 generations a step **on rung 0's unchanged body**, five steps. A chain gets 500 more generations at every step, and the reference says what that alone is worth.
- **The reading.** A rung's gain by continuation is (the chain's speed after the rung − before it) less (the reference's change over the same step). The first reference step is run twice with different draws; their difference is this measure's own yardstick.
- **What it gives up.** It is local: it says what each change is worth *to this swing and its neighbours*, a swing that grew up on the old body. A change that only pays with a different kind of swing (rung 3's wider base may be one) shows here as less than it is worth, or as a loss. The searches from S are the other view. Both are reported for every rung; where they disagree, both are said.
- *What I expect:* the reference creeps up by 0.1–0.5 m/s a step, its two draws within 0.5 m/s of each other. Rung 1 gains 1–3 m/s by continuation (the replay of this same plan gained 2.0 before any search), rung 2 loses 0–2, rung 3 is the least certain (the replay of the plan on MLB's feet is fast, 22 m/s, and misses by 14 cm), rung 4 nothing, rung 5 loses 0–2.

#### Rung 2: a hold that obeys the muscles' rules (the design, written before any search; built and checked at the start instant first)

*(Written before rung 1's values were settled, because it does not wait on the reading; it is run "alone", on rung 0's body, while the damping's sources are read, and again on rung 1's body as the ladder's rung 2.)*

**What a standing hitter's muscles do.** Nothing but his muscles holds him in his launch posture: the thighs' front muscles hold the bent knees, the hips' extensors and side muscles hold the pelvis, the calves hold the ankles, the trunk's muscles hold its lean, the shoulders hold the bat up. Each of those muscles is working at a low level before the swing, and it is the same muscle that then fires. So a holding torque is a muscle's torque and is under everything a muscle is under.

**The choices.**

1. *The hold lets go as each muscle's drive starts* (as §4.11's tensed muscle does). Rejected: until it lets go the torque is still under no strength-at-speed law and no effort charge, and a search can keep a useful hold for nothing by never firing a muscle at that joint. At the lead knee, session 10's swing got twice its extensor's work from the hold; a plan that left the knee's muscles unfired would keep all of it.
2. *The hold is carried by the muscles themselves.* **Taken.** It obeys every rule the muscles are under, and it is what the body does.

**The rule, as built** (`drive.solve_hold_muscles`, `add_hold_muscles`; `practice3d.Ctx(stand="muscle")`).

- The hold-still torques are solved as before (`solve_tone`: the torques that make the placed body's accelerations least). Each joint's torque is then split along the axes its muscle groups act about (exactly, at a joint with three such axes; at a hinge or a two-axis joint the part about an axis the joint itself holds does nothing and is dropped), and each share is handed to the group that pulls that way.
- **A group's holding level is a drive of the engine like any other:** about the group's own axis, on the group's own measured strength-at-speed curve (§4.5), fully on from the start, its strength set so that at the start's own joint speed it gives exactly the hold-still torque. So the body's accelerations at the first instant are what they were (checked: the largest difference of any body's acceleration is 3 × 10⁻¹³ of 1,668). From then on:
  - *strength at speed:* it gives less as the joint moves its way and nothing at its curve's top speed (the knee extensor's holding 69 N·m is half at 3.95 rad/s and gone at 13.4 rad/s, 770°/s); it gives more, to the stretched muscle's plateau, when the joint is forced back, as an active muscle does;
  - *the bound:* the group's burst may ask at most its bound less its holding strength, so holding and burst together never exceed the measured strength;
  - *effort:* the group's level in the score is its burst's and its holding level's together;
  - *letting go:* it goes when the plan switches that muscle off (the plan's own switch-off time, 100 ms after the start at the earliest), with a muscle's 50 ms fall;
  - *the clock:* a holding level is not a fired arm muscle; the clock reads onsets, as since §4.11;
  - *no tendon spring* [E]: at 69 N·m the knee extensor's spring holds about 2 J.
- **The trunk's bending has no muscle in the plan** (§4.5), and the largest hold with nowhere to go is there: 43 N·m bending the trunk toward his front side. It is carried by **holding-only groups with a measured strength**: Schultz et al. 1987's standing men ([note 12](research/body/12-trunk-bending-and-feet.md) C1: 10 men of 19–25, isometric, about L3; flexion 191 ± 70, extension 204 ± 78, side bend 180 ± 65 and 164 ± 56 N·m), each at mean + 2 SD scaled by weight × height as every bound of §4.5 is (443, 482, 415 and 370 N·m). Their strength at speed is Thompson 1985's standing extension over 0.52–2.09 rad/s (9% lost), then a straight line to zero at the twist's top speed, 12.9 rad/s [E: nothing is measured above 2.1 rad/s for a trunk's bending]. **The search cannot fire them:** they hold, under the curve, and that is all. Giving the trunk's bending muscles to the search is a different change (more muscle for the swing) and is not made here.
- **What is left as a constant torque,** because no muscle of this body acts that way and no strength for one has been read (D34): the rear ankle's roll 11.8 N·m, the lead ankle's pull-up 7.7, the rear hip's turn the other way 4.5. Reported apart in every result, with their work.

**At the start** (`s11/hold_check.py`, `s11/tones.py`): 25 groups hold. The front knee's extensor 69.1 N·m (0.17 of its bound), the rear knee's 59.6 (0.17), the front hip's flexor 58.5 (0.21), the rear hip's extensor 44.5 (0.11), the trunk's side benders 43.1 (0.10), the rear calf 22.8, the front hip's side muscles 22.3, the trunk's rotators 19.2, the rest under 12 each (the arms' 1–6 N·m: the shoulders' raisers holding the arms and bat up).

**What I expect** (rung 2 alone, on rung 0's body, from S by the protocol):

- *The replay of session 10's kept plan on this body, before any search* (indicative only, rule 4): it passes 10.6 cm under the ball at 15.2 m/s; the holding muscles do +20 J net where the constant torques did +157 J, and the muscles themselves 456 J where they did 417.
- *The search:* it meets the ball within 1 cm again. **Slower than rung 0 by 1–3 m/s** (14.5–17 m/s): 157 J of the kept swing came from the constant torques, the replay keeps 20 J of it, and the bursts at the knees and hips have 11–21% less of their bound to ask. The front knee no longer snaps straight at 870°/s into its end; its end's load falls well under 1,005 N·m.
- *What it would show:* what the constant torques were worth in bat speed on this body, by this search. *What it would not show:* whether a real hitter's legs give more; the holding levels are worked from [4]'s launch posture and its ground forces (rung 5's list), and the trunk's bending still cannot be driven.

**Rung 2 alone, as run** (rung 0's body with the posture held by muscle and nothing else changed; jobs 413 and 414, 19 minutes each; `s11/r2a.out`, `r2a_b.out`, `.json`). **Its two searches give 16.65 and 15.74 m/s. Both lie inside rung 0's own pair (17.69 and 15.74): taking the constant hold-still torques away and making his muscles hold him shows no loss of bat speed beyond the yardstick.**

| | seeds 1, 11 | seeds 2, 12 |
| :-- | --: | --: |
| the sweet spot at contact | **15.74 m/s (35.2 mph)** | **16.65 m/s (37.3 mph)** |
| the miss | 0.2 cm | 0.3 cm |
| the score (lower is better) | 35.0 | 33.8 |
| settled: the fall over the last 50 generations | 0.50 | 0.19 |
| the speed at 0.5, 0.25, 0.125 ms | 15.74, 15.74, 15.74 | 16.67, 16.65, 16.65 |
| failed runs among the plan's 205 one-number changes | 0 | 0 |
| the line from the starting plan to the result (11 points, then one beyond) | 425 457 449 463 464 488 458 230 94 49 35 60 | 425 434 428 505 569 469 122 67 46 38 34 44 |
| the bat at its fastest at contact | 89% | 89% |
| the hands' fastest | 6.9 m/s | 7.5 m/s |
| the attack angle | -5.7° | -4.7° |
| the hips'; the trunk's fastest turning | 346; 391°/s | 306; 444°/s |
| the pelvis's turn from the start to contact | 34° | 47° |
| first rises: legs, trunk, arms (ms before contact) | 312, 309, 254 | 319, 290, 257 |
| the first arm muscle before contact (253 allowed) | 254 ms | 257 ms |
| the lead knee at contact; the front foot's greatest load | 41°; 0.41 x body weight | 28°; 1.29 x body weight |
| balance: outside his feet; least on the feet; height kept | 0.0 cm; 0.13; 0.89 | 0.4 cm; 0.05; 0.90 |
| joints past their range: to contact; to 50 ms after | 2.3° (the lead upper arm's turn); 4.7 (the lead forearm) | 2.6° (the lead upper arm's turn); 4.8 (the lead forearm) |
| joint ends over 50 N·m; the heaviest | 14; 962 N·m (trunk side) | 14; 1026 N·m (shoulder up L) |
| groups at 90% or more of their bounds | 7 | 10 |
| WORK TO CONTACT, J: muscles | +443 | +506 |
| holding muscles | +20 | +15 |
| hold-still torques (constant, under no muscle rule) | -1 | +5 |
| the joints' damping | -248 | -269 |
| stops; ground; gravity | -23; -14; +63 | -41; -40; +66 |
| kinetic energy gained | 241 | 243 |
| muscles (with holding) by joint, the five largest | hip R +137, shoulder R +90, shoulder L +83, lumbar +82, elbow L +17 | hip R +193, shoulder L +94, shoulder R +93, lumbar +34, knee R +27 |
| damping by joint, the five largest | shoulder R -68, elbow R -37, wrist R -29, shoulder L -25, wrist L -24 | shoulder R -70, elbow R -45, wrist R -32, shoulder L -28, wrist L -26 |
| hold-still torques by joint, the three largest | hip R -3, ankle R +2, ankle L +0 | ankle R +6, hip R -4, ankle L +3 |
| GATE: meets the ball | passes (-0.8 cm) | passes (-0.7 cm) |
| GATE: the sweet spot at contact | fails (short by 13.3 m/s) | fails (short by 12.3 m/s) |
| GATE: the bat at its fastest at contact | fails (-9 points) | fails (-9 points) |
| GATE: the hands' fastest | fails (short by 3.4 m/s) | fails (short by 2.8 m/s) |
| GATE: the attack angle | fails (short by 2.7°) | fails (short by 1.7°) |
| GATE: the hips' fastest turning | fails (short by 209°/s) | fails (short by 249°/s) |
| GATE: the trunk's fastest turning | fails (short by 364°/s) | fails (short by 311°/s) |
| GATE: first rises legs, trunk, arms | passes (-312, -309, -254 ms) | passes (-319, -290, -257 ms) |
| GATE: no arm muscle before he could decide | fails (+1 ms) | fails (+4 ms) |
| GATE: the hands' first motion before contact | fails (over by 33 ms) | fails (over by 25 ms) |
| GATE: the lead knee at contact | fails (over by 15°) | fails (over by 2°) |
| GATE: the front foot's greatest load | fails (short by 1.09 x body weight) | fails (short by 0.21 x body weight) |
| GATE: in balance | passes (+0.0 cm) | fails (+0.4 cm) |
| GATE: does not jump | passes (+0.03) | fails (-0.05) |
| GATE: legs have not given way | fails (-0.01) | fails (-0.00) |
| GATE: no joint more than 2° past its range | fails (+2.7°) | fails (+2.8°) |
| GATE: no joint end heavily loaded (D39, not built) | fails (14 ends over 50 N·m; the most 962 N·m (trunk side)) | fails (14 ends over 50 N·m; the most 1026 N·m (shoulder up L)) |
| GATE: no passive element does net positive work | passes (stops -23 J, ground -14 J, damping -248 J) | passes (stops -41 J, ground -40 J, damping -269 J) |

- **By the ladder's rule** (the better-scoring search): 16.65 m/s (37.3 mph) against rung 0's 17.69: 1.04 m/s less, inside the 1.95 m/s yardstick, so **no loss is shown**. Seed for seed: 15.74 against 17.69 (−1.95) and 16.65 against 15.74 (+0.91): the sign is not even the same.
- **The energy moved as expected even where the speed did not:** the constant torques' +157 J and +82 J (rung 0's two swings) are gone; the holding muscles give +20 and +15 J, and the muscles themselves 443 and 506 J where they gave 417 and 439. What is left as constant torque (the ankles' roll, the rear hip's turn the other way) does −1 and +5 J.
- **Against what I expected:** I expected 1–3 m/s slower. By the rule's figure it is 1.0 slower, by the pair it is no different; I had not weighed that the search would simply ask more of the muscles (their work is up 26–67 J). Right that the front knee no longer snaps straight into its end: it is 41° and 28° bent at contact (rung 0: 1° and 40°), and the heaviest joint end is no longer the knee.
- *What it shows:* on this body, by this search, the 27% of the swing's energy that the review found coming from torques under no muscle rule was not what held the speed up: the muscles can carry it. *What it does not show:* anything finer than about 2 m/s.

**Rung 2 as run, on rung 1's body** (the read friction and the posture held by muscle; jobs 417 and 418; `s11/r2.out`, `r2_b.out`, `.json`). **16.85 and 16.58 m/s (37.7 and 37.1 mph): both searches faster at contact than both of rung 1's (14.72, 14.85). By the ladder's rule a gain of 2.1 m/s (4.8 mph) over rung 1, the first the ladder shows; against rung 0 it is 0.8 m/s less, inside the yardstick.**

| | seeds 1, 11 | seeds 2, 12 |
| :-- | --: | --: |
| the sweet spot at contact | **16.85 m/s (37.7 mph)** | **16.58 m/s (37.1 mph)** |
| the miss | 0.3 cm | 0.6 cm |
| the score (lower is better) | 33.1 | 33.8 |
| settled: the fall over the last 50 generations | 0.66 | 0.56 |
| the speed at 0.5, 0.25, 0.125 ms | 16.84, 16.85, 16.85 | 16.53, 16.58, 16.59 |
| failed runs among the plan's 205 one-number changes | 0 | 0 |
| the line from the starting plan to the result (11 points, then one beyond) | 369 378 405 444 479 365 211 158 72 38 33 51 | 369 337 346 154 123 98 86 78 54 40 34 46 |
| the bat at its fastest at contact | 91% | 86% |
| the hands' fastest | 7.4 m/s | 7.6 m/s |
| the attack angle | -3.1° | -3.8° |
| the hips'; the trunk's fastest turning | 208; 337°/s | 385; 399°/s |
| the pelvis's turn from the start to contact | 24° | 46° |
| first rises: legs, trunk, arms (ms before contact) | 320, 326, 253 | 339, 317, 255 |
| the first arm muscle before contact (253 allowed) | 253 ms | 255 ms |
| the lead knee at contact; the front foot's greatest load | 34°; 1.04 x body weight | 39°; 1.53 x body weight |
| balance: outside his feet; least on the feet; height kept | 2.0 cm; 0.12; 0.88 | 0.0 cm; 0.09; 0.89 |
| joints past their range: to contact; to 50 ms after | 2.1° (the lead upper arm's turn); 3.7 (the lead forearm) | 2.5° (forearm R); 5.0 (the lead forearm) |
| joint ends over 50 N·m; the heaviest | 14; 1010 N·m (shoulder up L) | 16; 890 N·m (shoulder up L) |
| groups at 90% or more of their bounds | 6 | 9 |
| WORK TO CONTACT, J: muscles | +381 | +459 |
| holding muscles | +25 | +28 |
| hold-still torques (constant, under no muscle rule) | +1 | +6 |
| the joints' damping | -168 | -215 |
| stops; ground; gravity | -38; -25; +74 | -77; -40; +71 |
| kinetic energy gained | 250 | 247 |
| muscles (with holding) by joint, the five largest | hip R +152, lumbar +84, shoulder L +83, shoulder R +50, knee R +24 | hip R +207, shoulder L +81, lumbar +77, shoulder R +59, hip L +41 |
| damping by joint, the five largest | lumbar -42, shoulder R -39, hip R -24, elbow R -22, hip L -15 | lumbar -83, shoulder R -34, hip R -28, hip L -23, elbow R -18 |
| hold-still torques by joint, the three largest | ankle L +4, hip R -4, ankle R +0 | ankle R +6, hip R -4, ankle L +4 |
| GATE: meets the ball | passes (-0.7 cm) | passes (-0.4 cm) |
| GATE: the sweet spot at contact | fails (short by 12.1 m/s) | fails (short by 12.4 m/s) |
| GATE: the bat at its fastest at contact | fails (-7 points) | fails (-12 points) |
| GATE: the hands' fastest | fails (short by 2.9 m/s) | fails (short by 2.7 m/s) |
| GATE: the attack angle | fails (short by 0.1°) | fails (short by 0.8°) |
| GATE: the hips' fastest turning | fails (short by 347°/s) | fails (short by 170°/s) |
| GATE: the trunk's fastest turning | fails (short by 418°/s) | fails (short by 356°/s) |
| GATE: first rises legs, trunk, arms | fails (-320, -326, -253 ms) | passes (-339, -317, -255 ms) |
| GATE: no arm muscle before he could decide | passes (+0 ms) | fails (+2 ms) |
| GATE: the hands' first motion before contact | fails (over by 39 ms) | fails (over by 45 ms) |
| GATE: the lead knee at contact | fails (over by 8°) | fails (over by 13°) |
| GATE: the front foot's greatest load | fails (short by 0.46 x body weight) | passes (inside by 0.03 x body weight) |
| GATE: in balance | fails (+2.0 cm) | passes (+0.0 cm) |
| GATE: does not jump | passes (+0.02) | fails (-0.01) |
| GATE: legs have not given way | fails (-0.02) | fails (-0.01) |
| GATE: no joint more than 2° past its range | fails (+1.7°) | fails (+3.0°) |
| GATE: no joint end heavily loaded (D39, not built) | fails (14 ends over 50 N·m; the most 1010 N·m (shoulder up L)) | fails (16 ends over 50 N·m; the most 890 N·m (shoulder up L)) |
| GATE: no passive element does net positive work | passes (stops -38 J, ground -25 J, damping -168 J) | passes (stops -78 J, ground -40 J, damping -215 J) |

- **I expected this rung to cost speed, and on rung 1's body it gains.** Why, as far as the swings show: rung 1's swings had the constant torques putting in 185 and 177 J (84–105 J at the front hip, 58 J at the front knee, both of which they threw straight) on a body with light arms, and the bat was past its peak at contact (72–75% of its fastest). With the hold under the muscles' rules the body gains 250 and 247 J of kinetic energy instead of 426 and 525, the hands are back at 7.4–7.6 m/s, the bat's fastest before contact is lower (18.6 and 19.3 m/s against 20.4 and 19.7) and **it arrives at 91% and 86% of that fastest instead of 72–75%.** A slower body that the search can time meets the ball faster than a faster one it cannot.
- **Seen with rung 2 alone** (on rung 0's body: no loss shown), the hold under the muscles' rules has not cost bat speed on either body. The honest figure the job expected to be a cost is, by these searches, between −1.0 (alone) and +2.1 m/s (on rung 1).
- **Rungs 1 and 2 do not add.** Alone from rung 0: rung 1 −3.0, rung 2 −1.0; together −0.8. The two changes work on each other through what the search can time, and through the noise of the searches themselves.
- *The energy:* muscles 381 and 459 J, holding muscles +25 and +28 J, what is left as constant torque +1 and +6 J; the friction takes 168 and 215 J, 42 and 83 J of it at the trunk; the stops 38 and 77 J.
- *Against the gate,* the first search: meets the ball (0.3 cm); the attack angle −3.1° (fails by 0.1°); first rises in order with the arms exactly on the clock (253 ms); the front leg 34° bent and loaded to 1.04 times his weight; hips 208°/s and trunk 337°/s, slower than rung 0's.
- *What it shows:* with friction at the read values and the hold carried by muscle, both parts the review found unmeasured are out of the swing, and it meets the ball at 16.6–16.9 m/s: about what rung 0's two searches gave (15.7–17.7) with them in. *What it does not show:* a gain or a loss against rung 0.

#### Rung 3: the front foot's place and angle from MLB's data (the design, written before any search)

**What was read** is [research note 15](research/body/15-stance-and-hip-turn.md) A and B. Statcast's published stance columns are the feet *before the stride* (one second before release) and are no use for a body whose feet are already planted. But the leaderboard page carries each hitter's foot points at the bat's intercept, and a planted front foot is where it landed. Recomputed by me from those points (2026, 231 hitter-sides; the units checked against the published stance column, correlation 0.986):

| | Major leaguers at the intercept | The body until now ([4]'s) |
| :-- | --: | --: |
| the ankle joints apart, along the line to the pitcher | **102.6 ± 7.9 cm** (10th–90th percentile 92–113) | 77 cm |
| the lead ankle toward the plate from the rear one | **1 ± 10 cm** | 10.4 cm |
| the front foot turned toward the pitcher from square to the plate | **43.8 ± 10.2°** (31–58) | 30° |
| the ball met, toward the pitcher from the lead ankle; from the rear ankle | **−0.4 ± 9.4 cm** (level with it); 102.2 ± 8.8 cm | +27 cm; 103.7 cm |

- **He has been standing on a base 26 cm narrower than a major leaguer's, meeting the ball 27 cm in front of his front foot where they meet it level with theirs.** [4]'s hitters (0.42 of height) and Welch's seven off a tee (85 cm toe to toe) stride shorter than major leaguers do in games; §4.10 took the ball's place from Savant's figure measured from the hips *in the stance* and applied it from the hips at launch, which are further forward.
- **The choice: MLB's averages, not practice.** The job allows either and prefers practice inside MLB's range. Practice over the feet would mean a search per candidate stance (each with the start's rule rerun, 7 minutes, and two searches, 40 minutes), and two searches of the *same* body differ by 1.95 m/s (rung 0): a handful of candidates could not be told apart. So the feet take the league's means, which are real figures: **1.03 m apart, level across, the front foot 44° open.** The angle is the intercept's: a major leaguer's front foot may land less open and turn (Welch's seven landed at 29°); this body's cleated foot cannot pivot, so it stands at its contact angle from the start [E: 29–44°].
- **The rest of the start, by the start's own rule** (§4.12's, `clear_start`, with the three numbers given and not counted): the least change to the other fourteen numbers that puts every joint 5° inside its end on those feet (job 419, 444 s). On [4]'s other numbers unchanged the wide base cannot be stood on at all (both knees' bends cannot be met, a heel lifts 60°); the rule moves the pelvis 9.9° more open, 6.1 cm back from the lead ankle, tilted 8.3° more, the rear knee 6.9° straighter and the rear toe 7.8° back; the arms' numbers stay as §4.12's. **The rule stops with the rear ankle's roll 4.0° inside its end and both hips' sideways margins and the lead ankle's roll at 4.7°, not the 5° it asks** (its [E] is 3–8°): on a base this wide the hips' sideways range and the ankles' roll are what give out. The pelvis is 5.5 cm lower and the centre of mass 0.47 of the way from the rear ankle to the lead (it was 0.38).
- **The ball's place follows the feet:** 1.022 m toward the pitcher from the rear ankle, which is level with the lead ankle, as Statcast has it; its height and its distance across are unchanged. PLAN D28 and D29 are in the plate's frame and are not touched: this fixes where the model's feet are under them. Without this the old rule (0.72 m ahead of the hips at launch) would put the ball 21 cm in front of his front foot, 2.3 SD from where major leaguers meet it.
- *Still [4]'s after this rung:* the pelvis's and trunk's turn, the knees, the tilt, the arms and bat, every starting velocity, the feet's starting loads (rung 5's list). The start is flagged as that in every result.
- *Held by muscle on this base* (rung 2's solve): the trunk's side benders 91 N·m (43 before), **the rear hip's adductors 78 N·m, 0.54 of their bound** (the wide base hangs on them, and their burst keeps 46%), the front knee's extensor 63, the rear hip's rotators 28 (0.35 of their bound).
- *Tests:* the placed ankles, foot angle and margins against the figures above; the ball level with the lead ankle within 1.2 cm; with `BB_SLOW=1` the rule rerun returns the stored numbers (`StanceChecks`).

**What I expect** (rung 3, from S by the protocol, two searches):

- *The replay of session 10's plan on this body, no search* (indicative): it crosses the ball's plane 14 cm high at 22.0 m/s, the pelvis turned 30° past facing the plate.
- *The search:* meets the ball within 1 cm; **I do not know which way the speed goes.** For it: the front foot is 14° more open, so the front hip has 14° more turn before its end; the ball is level with the front foot instead of 27 cm ahead of it, so he need not reach; the replay is fast. Against it: S was tuned on the narrow base and is a worse place to start from; the rear hip's adductors and rotators have a half and a third of their strength taken by holding the base; the stance sits within a degree of four joint ends. If both searches from S end clearly worse than rung 2's, rung 2's result is carried on for 500 generations on this body too (the protocol's rule).
- *What it would show:* what [4]'s feet were worth, at MLB's mean feet. *What it would not show:* the best feet for this body (no practice over them), or a hitter's launch posture on a wide base (the pelvis's turn, the knees and the velocities are still [4]'s, bent to fit by a rule).

#### Rung 3 as run: on major leaguers' feet both searches find the fastest swings this body has made (jobs 451 and 452; `s11/r3.out`, `r3_b.out`, `.json`; the better kept as `analysis/research/body/practice3d_s11_rung3.json`)

**20.66 and 21.53 m/s (46.2 and 48.2 mph), 0.7 and 0.1 cm from the ball, settled, the same at three steps, with the two best scores of any search (28.1 and 25.2). Both are faster than both searches of every rung before. By the ladder's rule: 21.5 m/s, a gain of 4.7 m/s (10.5 mph) over rung 2 and of 3.8 m/s (8.6 mph) over rung 0's 17.7.**

| | seeds 1, 11 | seeds 2, 12 |
| :-- | --: | --: |
| the sweet spot at contact | **20.66 m/s (46.2 mph)** | **21.53 m/s (48.2 mph)** |
| the miss | 0.7 cm | 0.1 cm |
| the score (lower is better) | 28.1 | 25.2 |
| settled: the fall over the last 50 generations | 0.11 | 0.25 |
| the speed at 0.5, 0.25, 0.125 ms | 20.67, 20.66, 20.66 | 21.56, 21.53, 21.53 |
| failed runs among the plan's 205 one-number changes | 0 | 0 |
| the line from the starting plan to the result (11 points, then one beyond) | 143 112 99 92 87 78 63 52 39 33 28 35 | 143 106 92 77 59 49 36 33 29 28 25 29 |
| the bat at its fastest at contact | 97% | 96% |
| the hands' fastest | 9.6 m/s | 7.5 m/s |
| the attack angle | -2.7° | -0.0° |
| the hips'; the trunk's fastest turning | 199; 372°/s | 225; 409°/s |
| the pelvis's turn from the start to contact | -1° | 16° |
| first rises: legs, trunk, arms (ms before contact) | 254, 253, 255 | 268, 223, 253 |
| the first arm muscle before contact (253 allowed) | 255 ms | 253 ms |
| the lead knee at contact; the front foot's greatest load | 58°; 1.97 x body weight | 50°; 2.10 x body weight |
| balance: outside his feet; least on the feet; height kept | 0.0 cm; 0.21; 0.89 | 0.0 cm; 0.21; 0.91 |
| joints past their range: to contact; to 50 ms after | 3.7° (the lead upper arm's turn); 3.7 (the lead upper arm's turn) | 2.7° (the lead upper arm's turn); 4.3 (wrist oval L) |
| joint ends over 50 N·m; the heaviest | 10; 956 N·m (knee R) | 13; 1272 N·m (shoulder up L) |
| groups at 90% or more of their bounds | 10 | 7 |
| WORK TO CONTACT, J: muscles | +478 | +513 |
| holding muscles | +20 | +13 |
| hold-still torques (constant, under no muscle rule) | +1 | +4 |
| the joints' damping | -172 | -177 |
| stops; ground; gravity | -30; -55; +68 | -29; -70; +52 |
| kinetic energy gained | 314 | 307 |
| muscles (with holding) by joint, the five largest | lumbar +127, hip R +116, shoulder L +105, shoulder R +60, hip L +47 | hip R +143, shoulder L +113, hip L +86, lumbar +75, shoulder R +61 |
| damping by joint, the five largest | lumbar -54, shoulder R -37, hip R -25, hip L -20, elbow R -19 | lumbar -65, shoulder R -36, hip R -25, hip L -20, elbow R -12 |
| hold-still torques by joint, the three largest | hip L -6, ankle R +5, ankle L +2 | ankle R +6, hip L -3, ankle L +2 |
| GATE: meets the ball | passes (-0.3 cm) | passes (-0.9 cm) |
| GATE: the sweet spot at contact | fails (short by 8.3 m/s) | fails (short by 7.5 m/s) |
| GATE: the bat at its fastest at contact | fails (-1 points) | fails (-2 points) |
| GATE: the hands' fastest | fails (short by 0.7 m/s) | fails (short by 2.8 m/s) |
| GATE: the attack angle | passes (inside by 0.3°) | passes (inside by 3.0°) |
| GATE: the hips' fastest turning | fails (short by 356°/s) | fails (short by 330°/s) |
| GATE: the trunk's fastest turning | fails (short by 383°/s) | fails (short by 346°/s) |
| GATE: first rises legs, trunk, arms | fails (-254, -253, -255 ms) | fails (-268, -223, -253 ms) |
| GATE: no arm muscle before he could decide | fails (+2 ms) | passes (-0 ms) |
| GATE: the hands' first motion before contact | passes (inside by 4 ms) | passes (inside by 2 ms) |
| GATE: the lead knee at contact | fails (over by 32°) | fails (over by 24°) |
| GATE: the front foot's greatest load | passes (inside by 0.47 x body weight) | passes (inside by 0.50 x body weight) |
| GATE: in balance | passes (+0.0 cm) | passes (+0.0 cm) |
| GATE: does not jump | passes (+0.11) | passes (+0.11) |
| GATE: legs have not given way | fails (-0.01) | passes (+0.01) |
| GATE: no joint more than 2° past its range | fails (+1.7°) | fails (+2.3°) |
| GATE: no joint end heavily loaded (D39, not built) | fails (10 ends over 50 N·m; the most 956 N·m (knee R)) | fails (13 ends over 50 N·m; the most 1272 N·m (shoulder up L)) |
| GATE: no passive element does net positive work | passes (stops -30 J, ground -55 J, damping -172 J) | passes (stops -29 J, ground -70 J, damping -177 J) |

- **What changed in the swing.** He lands on his front foot and loads it: 1.97 and 2.10 times his weight at its most (the gate asks 1.5–2.6; no swing before reached 1.6, and rung 0's kept swing had 0.51). The ball is level with his front foot and he no longer has to reach for it: the bat arrives at 97% and 96% of its fastest (the gate asks 98%), neither elbow is at its end at contact (12° and 51° in the first), and the stops take 30 J where they took 38–84 J on the narrow base. Contact comes 260 and 271 ms after the start, 40–50 ms sooner than on [4]'s feet. The bat comes through level (−2.7° and 0.0°: inside the gate's band).
- **What did not change, and is now the plainest fault:** his hips hardly turn. The pelvis turns −1° and 16° from the start to contact, at 199 and 225°/s at its fastest (the gate asks 555–855), and the front knee is still bent 58° and 50° at contact (2–26° asked): he swings with his trunk (372 and 409°/s; 755 asked) and arms from a planted, bent front leg. The front foot being 14° more open did not make the hips turn, and **it does not stay open: both feet turn on the dirt, toe back, the front foot by 53° and 30° and the rear by 44° and 23° by contact** (in rung 0's swing the front foot turned 25° and the rear 15° the same way). So the front foot is no longer 44° open when the ball is met; in the first search it is past square. The grip is the measured one (1.15); what turns the feet is the swing's own twist against a base whose hips do not turn.
- **Against the gate** (the better-scoring search): passes nine rows: meets the ball; the attack angle; no arm muscle before he could decide (253 ms: on the clock); the hands' first motion (218 ms); the front foot's load; in balance; does not jump; the legs have not given way; no passive element adds energy. Fails: the bat's speed, 7.5 m/s (17 mph) short; the bat at its fastest at contact by 2 points; the hands, 7.5 m/s (10.3 asked); hips and trunk; the order of first rises (the trunk's rotators first rise 30 ms *after* the arms'); the front knee; a joint 2.7° past its range before contact (the lead upper arm's turn) and 4.3° after (the lead wrist's ellipse); 13 joint ends loaded (D39 is not built).
- **The energy:** muscles 478 and 513 J, holding muscles +20 and +13 J, constant torques +1 and +4 J; friction −172 and −177 J (the trunk 54 and 65, the hips 45, the rear shoulder 37); the stops −30 and −29 J; the body gains 314 and 307 J.
- **Against what I expected:** I wrote that I did not know which way the speed would go. It went up by more than any change so far. The reasons I gave for a gain are the ones the swings show (no reach, a foot to land on); the reason against (S being a poor start on this base) was wrong: both searches from S went straight down the line to their results with no ridge (143, 112, 99, … 28).
- **The chain's step 3 said the opposite (−3.6 m/s), and it was the measure at fault,** as I warned before running it: it carried a swing grown on the narrow base onto the wide one and ended with the pelvis turning backward. On a change this large the searches from S are the reading to keep.
- *What it shows:* [4]'s feet and the ball's place in front of them were costing this body about 4 m/s. The speed gained comes from where major leaguers' feet are measured to be. *What it does not show:* that these are the best feet for him (no practice over them); and the posture standing on them is still [4]'s other numbers bent by a rule, within a degree of four joint ends.

#### Rung 4: the hips' turn by their own muscles (the design, written before any search)

**What was read** ([note 15](research/body/15-stance-and-hip-turn.md) C) **is not what session 10's review and §4.12 assumed.** They took "an active range gives about 11° more" from Boone & Azen's 44.4° and 44.2°. Those are active, but **measured seated, with the hip bent to a right angle** (so say Roaas & Andersson, Simoneau and a NASA report; Boone & Azen itself was not opened). A hitter's front hip at contact is near straight. With the hip straight:

| Source | Men | Turning in | Turning out |
| :-- | :-- | --: | --: |
| the body until now: Roaas & Andersson 1982, passive, prone | 105, aged 30–40 | 32.6 ± 8.2° | 33.6 ± 6.8° |
| **Simoneau 1998, active, prone** (the same men seated: 30 ± 7° and 35 ± 8°) | 21 college men | **32 ± 9°** | **44 ± 7°** |
| Ellenbecker 2007, active, prone | 101 professional pitchers | 23 ± 8° | 35 ± 9° |
| the same | 64 elite male tennis players | 27 ± 10° | 37 ± 9° |

- **The change:** each hip's end of turn moves to Simoneau's active, hip-straight means: **turning in 32° (it was 32.6°), turning out 44° (it was 33.6°).** The stop stays what it is. It is the one sample of ordinary young men measured actively with the hip straight; the two athlete samples have *less* of both, and taking the pitchers' would cut the turn in by 10° (said, not done: the owner's ruling is the active range, and the ladder changes one thing).
- **So the active range gives the front hip no more turn in.** The front hip's limit at contact is its turn in (the pelvis turning toward the pitcher over the planted front thigh), and that end does not move. What gains 10° is the turn out: the rear hip as the pelvis turns away from that thigh, and either hip in the load.
- *Test:* the two ends as stops on each side (`StanceChecks`).

**What I expect** (rung 4, from S, two searches): **no gain that clears the yardstick.** The end that limits the front hip has not moved; the rear hip's turn out carried 663 N·m at its end in session 10's swing, so its 10° more may let the pelvis turn a little further (a few degrees more at contact) and take load off that end. *What it would show:* whether the rear hip's end was holding the pelvis. *What it would not:* anything about the front hip, whose range by its own muscles is what it already had.

#### Rung 4 as run: the hips' turn by their own muscles changes nothing that can be seen (jobs 453 and 454; `s11/r4.out`, `r4_b.out`, `.json`)

**20.89 and 21.11 m/s (46.7 and 47.2 mph), both inside rung 3's pair (20.66 and 21.53). By the ladder's rule 21.1 against 21.5: 0.4 m/s less, far inside the yardstick. No gain and no loss shown,** as expected; the chain's step said the same (+0.06 m/s).

| | seeds 1, 11 | seeds 2, 12 |
| :-- | --: | --: |
| the sweet spot at contact | **20.89 m/s (46.7 mph)** | **21.11 m/s (47.2 mph)** |
| the miss | 0.2 cm | 0.0 cm |
| the score (lower is better) | 26.6 | 26.2 |
| settled: the fall over the last 50 generations | 0.55 | 0.33 |
| the speed at 0.5, 0.25, 0.125 ms | 20.88, 20.89, 20.89 | 21.12, 21.11, 21.11 |
| failed runs among the plan's 205 one-number changes | 0 | 0 |
| the line from the starting plan to the result (11 points, then one beyond) | 153 115 90 75 67 52 44 38 35 29 27 36 | 153 119 109 99 80 61 51 40 32 29 26 30 |
| the bat at its fastest at contact | 98% | 96% |
| the hands' fastest | 8.1 m/s | 8.1 m/s |
| the attack angle | -0.6° | -0.7° |
| the hips'; the trunk's fastest turning | 207; 379°/s | 188; 331°/s |
| the pelvis's turn from the start to contact | 4° | 11° |
| first rises: legs, trunk, arms (ms before contact) | 258, 239, 253 | 266, 233, 253 |
| the first arm muscle before contact (253 allowed) | 253 ms | 253 ms |
| the lead knee at contact; the front foot's greatest load | 61°; 2.38 x body weight | 41°; 2.35 x body weight |
| balance: outside his feet; least on the feet; height kept | 0.0 cm; 0.21; 0.90 | 0.0 cm; 0.13; 0.90 |
| joints past their range: to contact; to 50 ms after | 3.0° (the lead upper arm's turn); 3.0 (the lead upper arm's turn) | 2.8° (the lead upper arm's turn); 4.7 (wrist oval L) |
| joint ends over 50 N·m; the heaviest | 12; 1177 N·m (shoulder up L) | 13; 1330 N·m (shoulder up L) |
| groups at 90% or more of their bounds | 8 | 12 |
| WORK TO CONTACT, J: muscles | +463 | +483 |
| holding muscles | +22 | +32 |
| hold-still torques (constant, under no muscle rule) | +3 | +1 |
| the joints' damping | -166 | -192 |
| stops; ground; gravity | -38; -47; +62 | -29; -62; +59 |
| kinetic energy gained | 303 | 292 |
| muscles (with holding) by joint, the five largest | shoulder L +121, hip R +113, lumbar +87, hip L +72, shoulder R +61 | hip R +136, shoulder L +114, lumbar +85, hip L +70, shoulder R +65 |
| damping by joint, the five largest | lumbar -44, shoulder R -32, hip R -26, hip L -24, elbow R -19 | lumbar -65, shoulder R -37, hip L -29, hip R -27, elbow R -15 |
| hold-still torques by joint, the three largest | ankle R +6, hip L -4, ankle L +2 | hip L -5, ankle R +4, ankle L +2 |
| GATE: meets the ball | passes (-0.8 cm) | passes (-1.0 cm) |
| GATE: the sweet spot at contact | fails (short by 8.1 m/s) | fails (short by 7.9 m/s) |
| GATE: the bat at its fastest at contact | fails (-0 points) | fails (-2 points) |
| GATE: the hands' fastest | fails (short by 2.2 m/s) | fails (short by 2.2 m/s) |
| GATE: the attack angle | passes (inside by 2.4°) | passes (inside by 2.3°) |
| GATE: the hips' fastest turning | fails (short by 348°/s) | fails (short by 367°/s) |
| GATE: the trunk's fastest turning | fails (short by 376°/s) | fails (short by 424°/s) |
| GATE: first rises legs, trunk, arms | fails (-258, -239, -253 ms) | fails (-266, -233, -253 ms) |
| GATE: no arm muscle before he could decide | passes (+0 ms) | passes (+0 ms) |
| GATE: the hands' first motion before contact | fails (over by 9 ms) | passes (inside by 9 ms) |
| GATE: the lead knee at contact | fails (over by 35°) | fails (over by 15°) |
| GATE: the front foot's greatest load | passes (inside by 0.22 x body weight) | passes (inside by 0.25 x body weight) |
| GATE: in balance | passes (+0.0 cm) | passes (+0.0 cm) |
| GATE: does not jump | passes (+0.11) | passes (+0.03) |
| GATE: legs have not given way | fails (-0.00) | passes (+0.00) |
| GATE: no joint more than 2° past its range | fails (+1.0°) | fails (+2.7°) |
| GATE: no joint end heavily loaded (D39, not built) | fails (12 ends over 50 N·m; the most 1177 N·m (shoulder up L)) | fails (13 ends over 50 N·m; the most 1330 N·m (shoulder up L)) |
| GATE: no passive element does net positive work | passes (stops -38 J, ground -47 J, damping -166 J) | passes (stops -29 J, ground -62 J, damping -192 J) |

- **The front hip's limiting end did not move, and the swings still lean on both hips' ends of turn:** the front hip's carries 601 and 570 N·m and the rear hip's 641 and 856 N·m (rung 3: 443 and 326; 665 and 822). Ten degrees more turn out at the rear hip did not free the pelvis: it turns 4° and 11° from the start to contact, at 207 and 188°/s.
- *Small things that did move, none beyond what two searches of one body differ by:* the bat at 98% and 96% of its fastest at contact (the first search is the first swing to meet the gate's 98%); the front foot's load 2.38 and 2.35 times his weight (inside the gate's 1.5–2.6, near its top); both first arm muscles exactly on the clock.
- *What it shows:* the owner's ruling is applied (the hips at their active range, read with the hip straight) and is worth nothing measurable to this swing. *What it does not show:* that hip range is unimportant. His hips are not turning for some other reason, and a swing that did turn them would meet these ends.

#### Rung 5: what else of the lab study is still in the start (the list, and the one part a rule can carry; written before any search)

**Every number of the start that is still [4]'s after rungs 3 and 4** (from §4.10's table of where [4] enters):

| What | Where it comes from now | Can practice or a direct measurement replace it, and at what cost |
| :-- | :-- | :-- |
| **the posture's numbers:** the pelvis's turn and its place between the feet, the trunk's turn, both knees, the pelvis's side tilt, the bat's direction and tilt, the lag, both elbows (and three set by hand: the trunk's forward bend, its side bend, the rear foot's angle) | [4]'s curves and text at −250 ms, each moved by the start's rule (on MLB's feet by up to 9.9°) | *A direct measurement:* none read. Statcast has feet and the hips' place only; OpenBiomechanics has every angle at foot plant and is not to be used here. *Practice:* the search sets the fourteen numbers with the plan, the joints' margins charged in the score in place of the compass rule. Cost: every run then rebuilds its start (0.6 s beside a run's 0.18 s, so a search takes about four times as long), the arms' free choices flip (§4.12) and want a continuous rule first, and the hold and the feet's loads must be re-solved for each posture (built below). **Not done here: the next session's first item** |
| **every starting velocity** | the slopes of [4]'s curves over ±10 ms | *By a rule:* he stands still at launch. **Done below** (rung 5). What that leaves out is real: a hitter at foot plant is moving forward at about 0.6 m/s and his load is unwinding; his own stride would give him that, and a stride needs the postural control of §5.3 (not built) |
| **each foot's starting load and its level force** | [4]'s force plates at −250 ms (0.79 of his weight in all, 14% of it on the front foot) | *By a rule:* his own weight, shared by balance. **Done below** |
| the hold | worked from the posture and those forces (`solve_tone`), since rung 2 carried by his muscles | it follows whatever the posture and loads are; after rung 2 only the ankles' roll (10–14 N·m) and 4 N·m at the rear shoulder are left as constant torques with no measured muscle |
| the plan the search starts from (S) | session 9's, which came from the fit to [4] | not part of the body; flagged in every result. A plan written by hand reached 10 m/s in session 10 |
| *gone:* the feet's place and angle (rung 3), the ball's place relative to the feet (rung 3), the hips' ends of turn (never [4]'s) | | |

**The one change of rung 5: the launch state by a rule. He stands still in balance on his own muscles.**

- *No starting velocity* (`place(moving=False)`, as §4.10's `start_at_rest` run had it).
- *The feet's loads by balance* (`body.balance_loads`, `Model(loads="balance")`): his weight, shared between the feet by the lever rule along the line of the feet (each foot's load at the middle of its sole points on the ground). On MLB's feet: 401 N on the front foot and 502 N on the rear, all of his 903 N where [4]'s loads carried 713 N.
- *The feet's push* (`drive.solve_spread`). A body standing still on a wide base pushes its feet apart against the ground's grip, and statics does not say how hard (the two level forces are equal and opposite). **The rule [E]: the push that leaves the hold-still torques smallest** (the sum of their squares), from none to what the less loaded foot's grip allows. On MLB's feet it is 198 N; with none, the rear hip's adductors would be asked for more than their bound to hold the base, and the hold would be left as 178 N·m of constant torque. *First tried without the push, and with [4]'s velocities kept:* the loaded front foot then slides at its starting velocity and the ground drags on it with 300 N; the loads and the velocities are one state and have to go together.
- *What the start is then:* the centre of mass does not start to fall (0.1 m/s² against 2.1 on [4]'s loads) and the hold-still solve leaves 2.2 m/s² and 2.6 rad/s² (it left 8.8 and 30.9): "holding still" now means something. **The hold is a standing athlete's:** both knees' extensors 50 N·m each (0.12 of their bounds), the trunk's extensors 37, the rear hip's extensor 29, the calves 28 and 16, the rear hip's adductors 26 (0.20). On [4]'s loads and velocities the same posture asked the rear hip's adductors for 0.54 of their bound and the trunk's side benders for 91 N·m.
- *Tests* (`StanceChecks`): the ground's force is his weight to a part in a thousand and the lead foot's share is the lever rule's, worked in the test; the two feet's level forces are equal, opposite, the size solved and inside the grip; the hold's sum of squares is smaller at the solved push than at 30 N either side.
- **This replaces a measured moving state by an assumed still one. It is a rule, flagged as that, and not a measurement of how a hitter launches.**

**What I expect** (rung 5, from S, two searches): **slower than rung 4 by 0–2 m/s, and it may not clear the yardstick either way.** He no longer starts with [4]'s forward travel (0.63 m/s, 18 J) or its turning, and S's timings were tuned to a body already moving; against that, his legs now hold him properly and their bursts keep more of their bounds (the rear hip's adductors 0.80 of theirs against 0.46). Contact will come later after the start (it has to build everything from rest). *What it would show:* what [4]'s launch velocities and forces were worth to this search. *What it would not show:* what a hitter's own stride would give him.

#### Rung 6: which muscles, and by how much (the design, written before any run; queued behind rung 5 on the expectation that he is still short)

**No strength is added to the body (PLAN D34).** This is a measurement of the gap for the owner: what-if bodies, each labelled as that, searched to see what moves the bat.

- *Where the bounds already stand.* Every muscle group's bound in §4.5 is its source's mean + 2 SD, scaled to this batter's size: already the strength of about the strongest man in forty of each sample. So any scaling above 1.0 asks for more than that.
- *The reference:* the last rung's better-scoring result carried on for 500 generations on the unchanged body, **twice with different draws** (the difference is the yardstick for these carried-on searches, as rung 0's was for the ladder).
- *One group at a time:* the same result carried on for 500 generations on a body with one group's bounds at 1.5 times (hips: extensors, the front flexor, rotators and the side muscles; knees and calves; the trunk's rotators; shoulders; elbows; forearms and wrists: the six make up all 41 groups). The plan starts with the same torques, so the search begins from the same swing and may use the extra. Read as m/s per 10% of that group's strength (a fifth of the gain at 1.5), only where the gain is larger than the reference's own spread.
- *All together:* every bound at 1.5 and at 2.0 times, the same way: with the reference, three points for the smallest scaling of everything that would reach 29 m/s (a straight line through them, or "beyond 2.0" if it does not reach). **A what-if:** nobody has muscles 1.5 times the strongest measured.
- *The strongest batter the measured limits allow:* each bound at the top of its tested range in §4.5 (the hips' extensors 1.22 times, the trunk's rotators 1.12, the shoulders' sweep 1.11–1.22, the hips' adductors 1.4 and rotators 1.41), one carried-on search.
- *On the last rung's settled swing itself:* the groups at 90% or more of their bounds, with the speed each is shortening at when it works hardest.
- **What it would show:** which groups, given more, this search turns into bat speed, and roughly how much. **What it would not show:** that strength is what he lacks. A search that cannot use a muscle well gains nothing from more of it; a body whose start or joints hold the swing back would stay slow at any strength; and the carried-on searches are local, from one swing.

*What I expect:* the shoulders and the trunk's rotators move the bat most (they do the most work in every swing so far and sit at 91–99% of their bounds); the knees and calves least. Everything at 1.5 times gains 3–6 m/s; 29 m/s is not reached below 2.0 times, if at all. The strongest measured batter gains 1–2 m/s.

#### The chain as run: one swing carried through rungs 1 to 5 (jobs 440–450; `s11/r0_c1.out`, `r1_c.out` … `r5_c.out`)

| Step | Body | The sweet spot at contact | Its fastest before contact | Miss | Score | Settled (the fall over the last 50 generations) | Change by continuation |
| :-- | :-- | --: | --: | --: | --: | :-- | --: |
| start | rung 0: session 10's kept swing | 17.69 m/s (39.6 mph) | 19.25 | 0.2 cm | 32.5 | | |
| reference | rung 0, carried on 500 generations, two sets of draws | 17.69; 17.69 | 19.25 | 0.2 cm | 32.5; 32.5 | 0.00: **neither search found anything better than the swing it started from** | 0 |
| 1 | read friction | 17.24 (38.6) | 19.47 | **2.2 cm** | 44.5 | **not settled** (2.48) | −0.45 |
| 2 | and the posture held by muscle | **19.66 m/s (44.0 mph)** | 20.45 | 0.4 cm | **29.1** | 0.24 | **+2.42** |
| 3 | and major leaguers' feet | 16.07 (36.0) | 18.12 | 0.2 cm | 34.8 | 1.00, at the rule's edge | **−3.59** |
| 4 | and the hips' active turn | 16.13 (36.1) | 17.59 | 0.2 cm | 33.3 | 0.71 | +0.06 |
| 5 | and the still start | 12.97 (29.0) | 16.84 | **2.0 cm** | 44.1 | **not settled** (3.00) | −3.16 |

- **The reference is flat, exactly.** On its own body session 10's swing is a place the search cannot leave: 12,000 runs around it with each set of draws found nothing lower. So the reference chain is one number (its later steps would repeat the first and were stopped), and the chain's changes are read as they stand. It also means this measure's own noise cannot be read from the reference; the first step is run again with other draws (queued last).
- **Step 2 is the fastest swing this body has made: 19.66 m/s (44.0 mph), 0.4 cm from the ball, the best score of any swing so far (29.1; rung 0's kept swing 32.5), settled, the same at three steps (19.56, 19.66, 19.64 m/s).** It is on the body with neither of the two parts the review found unmeasured: the friction is at read values (the hips' and trunk's at the middles of their ranges) and the hold is carried by muscle. Against the gate it passes six rows (meets the ball; the attack angle, −1.6°, for the first time; first rises in order; in balance; does not jump; no passive element adds energy) and comes nearest yet on two more: **the bat at 96% of its fastest at contact** (98% asked; rung 0: 92%) and the hips at 531°/s (555 asked; 361). It still fails on speed by 9.3 m/s, the hands (6.6 m/s), the trunk (438°/s), the front leg (locked straight, 0.68 of his weight on the front foot at most), the clock by 2 ms, and it leans on 17 joint ends (D39 is not built). Muscles 459 J, holding muscles +24 J, constant torques −1 J, friction −203 J (the trunk 47, the hips 75, the rear shoulder 32), stops −64 J.
- **By this measure the read friction alone is worth nothing to the swing (−0.45 m/s, and it no longer meets the ball inside 1 cm after 500 generations), and the hold under the muscles' rules on top of it is worth +2.4 m/s.** The searches from S said the same in sign for both (rung 1 lost, rung 2 gained on it). What the two changes do together is the firmer figure: **+2.0 m/s over rung 0 by the chain** (19.66 against 17.69), −0.8 by the searches from S (16.85 against 17.69). The body is the same; the swings found on it differ by 2.8 m/s.
- **Steps 3 and 5 lose 3 m/s each, and this measure cannot say whether the body or the carried swing is why.** On major leaguers' feet the carried swing ends with the pelvis turning 26° *backward* from start to contact, the hips at 139°/s and the front knee bent 47°: an arms swing on a base it was not grown on. On the still start it had not settled and was 2 cm off the ball. This is the weakness I wrote down before running it: a change that needs a different kind of swing shows here as a loss. The searches from S on those bodies (running next) are the other view.
- **Step 4, the hips' turn by their own muscles: +0.06 m/s. Nothing,** as expected: the front hip's limiting end did not move.
- *Against what I expected:* the reference does not creep (I expected 0.1–0.5 a step); rung 1 by continuation loses a little where I expected +1–3; rung 2 gains 2.4 where I expected a loss; rung 4 nothing (right); rung 5 loses more than I expected, unsettled.

#### The dependence on friction nobody has measured: the design as run (changed twice before any of it ran)

Rung 1's design put these runs on rung 1's body. After the chain they were moved to rung 2's body and the chain's swing there; after rungs 3 and 4 they are moved once more, **onto rung 4's body and its better-scoring swing (21.11 m/s)**: the body after all four of the owner's changes, where the session's fastest swings stand and the figure the session's answer rests on. None of the earlier copies ran (they are stopped as they come up in the queue).

- *Each run:* rung 4's better swing carried on for 500 generations (the same draws, no extra blocks) on rung 4's body with some joints' friction moved to an end of its range. The middle is that swing itself (a settled swing carried on its own body returns unchanged: the chain's reference showed it, and rung 6's reference run checks it here).
- *All together:* every unread value at its low end (hips 0.5, trunk 1.0, elbow's bend 0.22, forearm's turn 0.02, wrist's tilt 0.03); at its high end (4.6, 27.5, 0.97, 0.10, 0.20).
- *One group at a time, the others in the middle:* the trunk at 1.0 and at 27.5; the hips at 0.5 and at 4.6. (The arms' three unread values are not run apart: in rung 4's swings the elbows' and wrists' friction takes under 25 J in all.) These are queued without waiting on the all-together runs: the queue is long and it is the owner's question.
- *What I expect:* low ends together: +0.5 to +2 m/s (about 110 J of the swing's 166–177 J of friction is at the trunk and hips). High ends together: a large loss, 3 m/s or more, nearly all of it the trunk's 27.5 (a value measured with the trunk's muscles holding a load; at this swing's 330–380°/s of trunk turn a damper that size asks for 160–180 N·m, two thirds of what the trunk's rotators have). The hips apart: under 1 m/s either way; they hardly turn in these swings.

#### Rung 6 stands on rung 4's body (decided before rung 5's searches finished, and before any of rung 6 ran)

Rung 6's design above says "the last rung's result". **It is run on rung 4's body, from rung 4's better-scoring swing (21.11 m/s), whatever rung 5 gives:** rung 4 is the body after the four changes the owner ruled (friction, the hold, the feet, the hips); rung 5's still start is a rule of mine that replaces a measured moving state with an assumed one, and "which muscles and by how much" should not rest on it. One reference run, not two (the chain's two references on rung 0's body were identical to the last digit).

## 5. The drive and its control (§14.1 items 2 and 3)

### 5.1 What a drive is

A drive channel is one muscle group's net torque at a joint, or force along a leg:

**τ(t) = T · a(t) · f_v(ω/W)** [form: the golf and bowling generators of [N1] §1.1–§1.2]

- **The excitation u(t)** is the motor command: 0 before the onset t_on, a linear ramp over the rise time t_rise, 1 until the switch-off t_off, then 0. A mid-motion change can alter u only from now plus the latency on (§5.4).
- **The activation a(t)** follows u through first-order dynamics: 15 ms rising, 50 ms falling [E: the defaults of OpenSim's Thelen 2003 muscle, [N5] §4, read from OpenSim's source; the paper's abstract states none]. [N1] lists 20 / 40 ms in golf, a minimum 70 ms ramp in bowling, and 5 ms in Attias's pitching model. The design's rule is a rise of at least 50 ms [E]; Maffiuletti et al. 2016 ([N5] §4, abstract only) describe early force development over 50–75 ms, not a time to peak, so the floor is an assumption for phase 1 to test.
- **The force–velocity factor f_v** is Hill's: f_v = (1 − w)/(1 + w/0.25) while shortening at w = ω/W of the channel's top speed W [E: curvature 0.25]. While lengthening it rises to 1.5 × [E, slope 4]. Golf's generators use the same form with C = 3, i.e. 0.33 ([N1]), which falls less steeply.
- **T** is the channel's strength: the player's strength rating for that group times his size factor (§7.1).
- **W** is the channel's top speed [E]: legs 20, core 25, shoulder 30, wrist 40 rad/s, legs along s 3 m/s. [N1]'s golf values are 30–60 rad/s, also estimated.

A learned swing is one set of (t_on, t_rise, t_off) per channel, made for the player's own body (§5.5). His ratings set T, and the body does the rest.

**Which links get a drive** (the research's second finding, [N1], [N3], [N5]): torque generators drive the proximal links; the last links are driven mainly by the segments' dynamics and elastic recoil.

- **For the batter:** legs, push, brake and block, trunk, both shoulders, and the elbows (the flat fit's failure under the decision clock, §4.3, says the elbows must be driven, not only held). The wrists hold the cock with their small measured strengths.
- **For the pitcher:** the same, plus a series elastic element at the shoulder. The two shoulder numbers that motivate it are not simultaneous: the internal-rotation torque of 90–100 N·m ([N3], ASMI inverse dynamics; horizontal adduction 90–120) peaks near maximum external rotation, where the internal-rotation speed is near zero, and the 115 rad/s peak speed comes later. What they show together is that the arm reaches a speed at which no Hill curve has drive left (52–80 N·m measured at ≤ 4.7 rad/s, [N5]), so its speed must come from energy stored earlier and from the segments' dynamics (Hirashima 2008; Roach 2013/14, abstract). Whether the shoulder also drives actively through the acceleration is open.

**What no source gives** (all [E], to be sourced or chosen in phase 1): the channels' top speeds W; the force–velocity curvature for lumped joint groups; the eccentric plateau; the friction coefficient on clay; each foot's free-moment reach; any anatomical limit on the wrists' cock; the trunk's axial range; strength in N·m by group for professional hitters ([N2]: "almost no peak joint torques in N·m as numbers in a table or abstract", and §5: the trunk's torque about the vertical "not found as a number"; the nearest are [N5]'s, all at slow isokinetic speeds); the lead foot's loading in the 3D body.

### 5.2 Where the drive's numbers come from

*Dated history (PLAN D37): the drive is no longer worked back from [4] or fitted to it. Each player's drive timings are found by practice on his own body (§4.10, §5.5); the strengths come from strength measurements (§4.5). What follows is how the flat probe's numbers were found, kept for the record.*

**Worked back from [4] (inverse dynamics).** `inverse.py` takes [4]'s mean pelvis and trunk rate curves, its bat azimuth, elevation and lag, and its pelvis travel, and computes the yaw torques a pelvis → trunk → arms → bat chain needs (de Leva's segments at [4]'s mean batter, 1.84 m and 92 kg). Corrected after the reviews: the pelvis's forward acceleration was added to the bat twice; the hands' velocity lacked the pelvis's forward speed; the arms' own inertia was missing from the shoulder's torque; the rear foot is now 0.77 m behind [4]'s pelvis zero, as in the fit. Not corrected: the inverse model hinges the arm on the trunk's axis (no shoulder offset), so its shoulder and core split differs from `chain.py`'s.

- **The legs' yaw torque** rises from about 200 N·m at −180 ms to a plateau of 480–490 N·m from −140 to −110 ms, then falls through zero. A spike of 680 N·m at −18 ms is near contact, where the twice-differentiated, averaged curves are noise (below).
- **The yaw moment of [4]'s measured ground forces** peaks at about 156 N·m at −150 ms. (The first version quoted 222 N·m at −120 ms; that was the other sign's column.) It is computed about the pelvis from the four horizontal force curves, the feet [E] 0.77 m apart and the lead foot 0.11 m closed ([4]).
- **The sign of [4]'s lateral forces is settled.** [4]'s Appendix C (the export script `ForcePlateInfo.sky`, printed pp. 202–204) negates both plates' Y for right-handed hitters and keeps it for left-handed ones, so the plotted +GRFy points toward home plate for both. That is the sign the probe used.
- **Over −170 ms to contact,** the chain needs 36 N·m·s of angular impulse; [4]'s lateral couple gives 19, less 6 from the closed lead foot's braking force, so 13. The remaining 23 N·m·s is open: [4] does not report the feet's free moments, and nothing yet shows they close it; the planar reading of the arms (below) also carries part.

**What filtering and averaging do.** [4]'s markers were low-pass filtered at 12 Hz, and its curves are means over hitters, each hitter's middle fastballs averaged and then aligned on ball contact ([4] §3.5); the five event lines label the phases. The inverse dynamics differentiates them twice after a further 16 ms smoothing [E]. Torque features shorter than about 40 ms are lost, the torques in the last 30 ms are noise, and the averaged curve is no one's swing. The arm's direction is read as the bat's azimuth plus [4]'s 3D lag, a planar reading of a 3D angle, which makes the arm's rate 1,418°/s at −90 ms against Welch's 1,160 at −65. The inverse torques are rough first guesses, with no independent bound on their accuracy; they set the fit's starting point, nothing else. Phase 1 redoes this in 3D with consistent marker geometry, a differentiation sensitivity sweep, and separate external-force and angular-impulse checks.

**Fitted forward.** The forward fit (§4.3) makes the measured motion come out of the drive and needs no second derivatives, but it identifies a motion in one model, not the drives: several of its numbers trade against each other.

**From published pitching forces** (§11): Fleisig's and Attias's joint kinetics ([N1] §1.4) and the bowling generators, for phase 5.

### 5.3 What keeps a driven body on its path

A body driven by a fixed program does not stay on its path by itself. Two cases differ.

**A standing or striding body falls without feedback** (`balance.py`). The model is a body over its support as an inverted pendulum along the stride: centre of mass at 0.53·H [E], the ankle torque bounded by where the centre of pressure can go (one foot: 0.12 m ahead of the ankle, 0.05 m behind [E]; a two-foot stance ±0.30 m [E]). The plan: stand, push the body into motion (the rear leg, [4]'s trail GRFx about 20% BW), then fall onto the front foot for 0.28 s ([4]: foot off to foot down). The planned ankle torque is worked back on the player's own model of his body. Errors: a lean 0.5° off at the start, the torque off by 1% of m·g·h, or his model's mass 2% off. "Falls" means leaning past 15° [E].

| Duration and support | Played back (open loop) | Muscles' own stiffness and damping (0.9·m·g·h, no delay) | Stiffness plus a reflex 100 ms late |
| :-- | :-- | :-- | :-- |
| 0.28 s on one foot (the batter's stride) | 1.2 cm off | 0.9 cm | 0.9 cm |
| 0.6 s, two feet then one (load and stride) | 2.7 cm | 1.0 cm | 0.9 cm |
| 1.5 s on one foot (a pitcher's leg lift and stride) | **falls at 1.33 s** (41 cm) | 1.6 cm (14 cm with the torque error) | 1.0 cm (1.4 cm) |

- **Larger errors on one foot** (1.5 s, the reflex case): a 1° lean error ends 2.0 cm off, 2° ends 14 cm off, 3° falls at 1.44 s. The centre of pressure reaches the edge of the foot and the reflex has nothing left to push with. The pitcher's leg lift is the hard case; the batter's stride is not.
- **Over the size range** (stature 1.70–2.00 m, mass 75–120 kg; 1.5 s, 0.5° lean): open loop drifts 37–45 cm, with the reflex 0.91–1.05 cm. Mass drops out because every term scales with it.
- **The reflex's delay** can be 50–200 ms (0.85–2.4 cm) and its gain 0.3–1.6 × m·g·h (0.86–1.27 cm); at 2.5 × it rings (1.5 cm).
- **What is real:** muscle has intrinsic stiffness (Loram and Lakie 2002, [N5] §4, abstract only: the ankle's quiet-standing stiffness is 91 ± 23% of what minimal stabilisation needs, attributed to the foot, Achilles tendon and aponeurosis, too weak alone). Balance is held by feedback with a delay between the long-latency reflex (50–100 ms) and voluntary correction (150 ms and more).
- **What is a device:** the feedback acts on the difference from the *planned* posture, so the plan is the reference. That is equilibrium-point control (a real theory, not settled). The gains are [E]. A posture reference reached through bounded, delayed forces is a control model; tracking the planned pose hard would replace the dynamics and is not allowed.
- **What the probe does not test:** the frontal plane (a foot is about ±0.04 m wide; Neon's run: a 1.5° sideways lean drifts 6 cm, 2.5° drifts 15 cm), the lifted leg's own motion, the landing's impulse, sensing noise in the reflex, and three rotational axes at once. Phase 1 tests all of these with the 3D feet.

**The swing in the flat probe needs no feedback.** It is 0.25 s from the first drive, shorter than a voluntary correction loop, the feet are planted, and a 15 ms late start of the whole program gives a 14.7 ms late meeting at 0.1 m/s less (§6.2). What holds it: the planted feet, the force–velocity law (a segment that runs ahead loses drive), and the joints' range limits.

**That does not yet show a 3D swing runs open loop.** The flat chain cannot fall: its feet are fixed, its vertical loads always sum to one body weight, and its sideways force is unbounded (§4.2). It has no swing plane to tilt, and in 3D 2 cm of barrel height separates a line drive from a pop-up (physics-spec §8.3). So in 3D the design keeps the muscles' intrinsic stiffness and damping (no delay, about the learned motion) through the swing as well, and phase 1 tests the swing with perturbed stance, foot friction and loss of support on all three axes, open loop first, then with bounded delayed proprioception. A new voluntary command cannot arrive in time; a reflex can.

**So the design:** postural feedback (intrinsic stiffness plus a delayed reflex, its reference the learned motion) through stance, load, stride, leg lift and balance; intrinsic stiffness through the swing and the arm's whip; sensed events may trigger the next part of the program (§5.4). Nothing reads the world (invariant 2). The reflex reads the player's own body as he senses it: the true body state, delayed, plus sensing noise drawn by `bb_agents::motor` (§5.4).

### 5.4 The batter's timing when his body is real

The events of the motion, for the average batter: [4]'s times at 56 mph batting practice, scaled to the pitch's flight as in [swing-synthesis §3](research/swing/swing-synthesis.md); in the game they run on the batter's clock, timed to the pitcher.

| Event | When ([4]) | In the body |
| :-- | :-- | :-- |
| the load, the negative move | stance to −621 ms | postural program: weight to the rear leg, the body coils (the load's depth is a trait, §7) |
| lead foot off | −621 ms | the stride's program starts on the batter's clock, set by what he sees of the pitcher's motion |
| toe touch | −340 ms (10% BW) | the lead foot's contact switches on; its load starts building |
| heel plant (the trigger) | not measured in [4] | a sensed event: below |
| the hips begin (on every pitch, D22) | pelvis rate rising from −245 ms | the hips' drive, gated on the heel plant |
| weight commitment | −170 ms (50% BW) | an event of the motion, not a trigger |
| the arms' commitment | [4]'s "first move" at −170 ms | the decision: the shoulders' and arms' channels may start only after t_commit + T_latency (agent-models §3.4) |
| contact | 0 | the bat meets the ball wherever the motion took it |

**One causal clock.** On every pitch:

1. **Started on every pitch, on the batter's own clock:** the load, the stride's drive and (D22) the hips' drive. The stride is timed to the batter's expectation of the pitch, built from what he has seen of the pitcher; nothing later in the pitch can move it back.
2. **Gated on a sensed event:** the hips' drive may not start before the heel plant is sensed (the walkthrough: "Heel-Plant is the trigger"). The event is the lead foot's sensed state crossing a threshold: in the 3D foot, heel contact; in the flat probe, its load reaching 10% BW ([4]'s foot-down threshold) [E]. The latency is 30 ms [E, range 20–50: a proprioceptive loop]. If the event never comes (the foot is held up, the stride is checked), the hips start at a deadline [E: the planned onset plus 100 ms], or not at all on a take. On a fastball the heel comes almost at once after toe touch and the gate does not bind (in the probe the average swing's trigger, the event plus its latency, comes about 30 ms before the hips' own onset).
3. **D22's early hip turn** is that gated drive at its pre-commit level: the same on every pitch, held up on a take (its drive stopped and an opposite drive started after the latency). A separately gated increase in the hips' drive may follow the decision; whether it is needed is phase 1's to measure.
4. **The arms commit** only after t_commit + T_latency. t_commit comes from the batter's own decision on his observations (agent-models §3.4), never from the true pitch. The latest commit that still meets the ball is a property of his learned swing *given the prefix already in progress*, not a constant T_swing: in the probe, delaying the arms 30, 60 and 90 ms delays the meeting 7, 22 and 51 ms while the trunk spins open without them (§6.2).
5. **A mid-motion change** (a re-blend toward a new aim, a check swing, a stronger or weaker effort decided late) may change only future excitation: u(t) for t ≥ t_now + T_latency, on any channel, already active ones included, filtered by activation. Nothing already integrated changes, and no earlier stride or onset is re-chosen from the true pitch.

**Required test (the prefix property).** Two pitches that give the batter the same cues up to time t produce the same nominal plan, the same motor draws and the same executed motion up to t. Phase 3 tests it through a take, a changed read, a late heel plant and a weak swing.

**The probe and the clock.** The free fit breaks rule 4: its arms start at −237 ms. Under the rule, the flat chain does not reach the ball (§4.3). Its timings must not be read as the clock's.

**How `bb_game` steps the pitch.** The pitcher's motion does not react to the batter, so it may be integrated first, provided the batter's cues are handed to him in simulated time. Then, per perception tick (10 ms, agent-models §3.2):

1. `bb_world` integrates the true body (and the ball) over the tick with the executed drive in force. It has no RNG.
2. The batter's observation is built from the true eye point and the true ball, with perception noise drawn agent-side as today (`bb_observe` types; the agent never gets the eye point or the body).
3. `bb_agents` decides on the observation: commit, hold, check, re-blend.
4. `bb_agents::motor` turns a decision into an excitation change effective from t_now + T_latency, with its keyed draws, and passes it to the world.
5. The body's own sensing for the reflexes and the heel trigger: the true body state, delayed by the loop's latency, plus sensing noise. That noise is drawn by `bb_agents::motor` from `KeyedRng` under a new `Purpose` (appended), one sample per tick with the tick's index as the sub-index, and passed to the world as read-only data with the executed drive. So a branch in one pitch does not shift any other draw, and `bb_world` stays free of randomness.

**Open, with the phase that must settle it:** how a body that pulls off moves the eye point and with it the batter's observations (phase 3); the heel trigger's threshold and latency (phase 1, from the 3D foot); whether a separate post-commit increase in the hips' drive exists (phase 1); the clock at game speeds: how the stride and hips compress at 90+ mph (phase 3; swing-synthesis §6); the arms' earliest onset relative to [4]'s first move (phase 1 and 3, §4.3).

**The check swing** is a decision to stop: the arms' and hips' channels switch off and an opposite drive (the antagonists) starts after the latency. Whatever the bat has, it keeps until the braking drive takes it away. Whether it crosses the plate is the ruling (rules-spec §4.5).

### 5.5 How a player produces a motion without reading the world

- **His learned swings.** For each player, a set of drive programs is built on his own body:
  - one per aim on a grid over the zone and its edges [E: for example 5 × 5 heights and sides];
  - one per pitch-speed class, for the load and stride's timing (grouped by the batter's *expected* speed, never the true one);
  - one per approach he can choose (for example the two-strike swing with a shorter stride, walkthrough §1): a planned approach is its own program with its own commit clock.

  Each is found by a bounded search that a real hitter's practice stands in for, on nominal inputs and his own body: meet the aim's point at its time with the most bat speed, from his launch posture, within the decision clock of §5.4. The search may fail for an aim; a failed aim stays failed (the player cannot hit that pitch well), and the library is frozen before any game draw. **This is pending the owner's ruling** (§14), not settled: whether a one-time practice search is acceptable under "no motion fitted to its planned end". [N1]'s golf, cricket and pitching models all find their timings by a search, but against a recorded motion; none produces a motion without one. Searching against recorded motions to identify the model's parameters (as `fit.py` does) is a separate laboratory operation.
- **Between aims.** For a pitch between grid points, his program is a blend of the neighbouring programs' timings. The body's response to the blend is whatever it is, and a blend can miss what a dedicated swing would have hit. Blending works only between programs from the same family of solutions: in Neon's probe, averaging the timings of two different fits moved contact by 9 ms and the trunk by 29°. So the grid's programs must be built by continuation from their neighbours, and phase 3 tests that a blended swing's contact moves smoothly between grid points (a test, not a fit).
- **What he can change in mid-motion:** future excitation only (§5.4, rule 5). Through the late adjustment of agent-models §3.6 (`Adjustability`, the speed of a re-blend) and the check swing.
- **Invariant 2.** The agent builds and blends programs on *his own* body model with his own beliefs about the pitch. `bb_agents::motor` draws the execution errors (§6.1) and hands the executed program to the world, which integrates the true body (§10). The agent never sees the executed motion.

## 6. Errors (§14.1 item 4)

### 6.1 Where each error enters

**A planned approach is not an error.** An easier swing, a two-strike swing with a shorter stride, or a swing sitting on one pitch is a program the batter chose (§5.5), with its own commit clock, so it may legitimately start earlier or later than his normal swing. Execution noise is what the body does differently from the program he chose.

| Error | Link it enters | Size [E, with range] | How it carries |
| :-- | :-- | :-- | :-- |
| onset jitter, per channel | the channel's burst (t_on and t_off move together) | a per-link baseline σ₀,k times the player's timing ratings (§7.4): proximal links (stride, hips) 10–30 ms, trunk and arms 3–10 ms, fingers 0.5–2 ms ([N6] §5: Koike's ±50 ms re-timing of the legs; Hore and Watts' 1 ms fingers, abstract only) | everything after the onset, through the dynamics |
| drive on the day | each channel's T: a common factor plus a small per-channel factor, multiplicative (σ ∝ drive, [N6]) | a fresh, calm player's baseline 3–5% CV [E: the reviews' estimate; [N6] §1 supports noise proportional to the drive but gives no coefficient], widened by fatigue and composure (PLAN §8.6) | the whole motion from that channel's onset; a weaker swing starts when planned and arrives late |
| perception error | the plan itself: aim, arrival time, pitch type (agent-models §3.2) | `Vision`, `SpinRecognition` | the program chosen; the body executes it faithfully |
| stride error | the stride's drive: its strength and onset (the stride is a motion, not a schedule) | as onset jitter and drive above | touch-down, and with it the heel trigger and the block, follow from the body. In the flat probe the touch-down time is an input: a stand-in |
| sensing noise | the reflexes' and triggers' view of his own body (§5.4) | [E] | the postural corrections and the heel trigger's timing |
| grip and finger slip (pitcher) | the hand–ball contact's parameters (§11) | `SpinCapacity` or a feel rating [E] | the ball's spin and direction through the contact |
| release timing (pitcher) | the fingers' drive | the timing ratings | the ball's direction, through the hand's path (§11) |

Draws keep every program physical: windows stay positive and ordered, a burst cannot start before its trigger, and correlated draws (the common drive factor, a shared timing offset) are drawn as such. A swing with jitter must still have a plausible physical cause.

**How big the errors are.** No study reports the trial-to-trial SD of a muscle group's onset or peak torque in hitting or pitching ([N6]). So the sizes are [E], and phase 3 (batter) and phase 5 (pitcher) set a few population scales (a common timing factor, a common drive factor, the per-link baselines' ratio) so that the body's own outcome spreads match measured ones. That is Tier B calibration of noise scales (PLAN §13), done on the body's result over many pitches, never on a pitch's own outcome. The targets, and what each really measures:

| Target | Value | What it includes | How to use it |
| :-- | :-- | :-- | :-- |
| a batter's contact depth, within-batter SD ([S10] §2) | 0.27 m (about 7 ms at 40 m/s) | pitch location, pitch types, perception and approach as well as execution | match in the cage with perception, locations and pitch mix active, never as motor noise alone |
| a batter's bat speed, within-batter SD ([S10] §2) | 1.9 m/s (4.3 mph) | depth, location, protective and two-strike swings | the same; the depth- and location-controlled SD, 3.2 mph = 1.4 m/s (Statcast 2025 sample, agent-models' swing-error table), is the nearer target for execution plus approach |
| a pitcher's release point SD (Wakamiya 2024) | 3–6 cm | three seasons: rubber position, fatigue, changes in mechanics, about 1 cm measurement error | an upper bound for within-outing execution |
| a skilled thrower's release (Jegede 2005, abstract only) | about 7 ms | a measured spread of release time in one throwing task | a check, not an onset σ |
| the fingers' release precision (Hore and Watts 2011, abstract only) | about 1 ms | timing with the ball's back-force on the fingers | the fingers' baseline, not a generic jitter |
| a batter's timing tolerance (Nakashima 2025) | 9.4 ± 6.3 ms | a sweet-spot window from separately measured bat and pitch paths | a check on the body's result, not a spread |

**What the probe says about the budget** (`ratings.py` B, `breakdowns.py`). Unplanned drive noise costs time as well as speed: a swing at 0.8 of its drive, started when planned, meets the ball 17 ms late at 27.9 m/s against 30.3. So drive noise large enough to give a 1.9 m/s (or even 1.4 m/s) spread would add 10–16 ms of timing spread, more than the whole 7 ms depth spread allows. The bat-speed spread must come mostly from planned approach and location, with motor drive noise at the 3–5% baseline. (Neon's probe: 10 ms of independent onset jitter gives 7.8 ms of timing and 1.5 m/s of speed spread in the flat chain.) Late links need the tightest timing; early links bear tens of ms.

**No error moves anything earlier in the motion.** Each is drawn from `KeyedRng` in `bb_agents::motor`, the pitch's program draws before the motion starts and the per-tick sensing draws by tick index (§5.4); the stream's draw order is part of the replay contract, as in `motor::execute_release`. Each changes one link at its time, and the integration carries it forward. The five independent errors of today's swing (height, plane tilt, reach, timing, effort) disappear: height, tilt and reach become results of the onsets and strengths.

### 6.2 The named breakdowns, from the body

`breakdowns.py` (output `breakdowns.out`), on the free fit of §4.3, whose arms start too early (§5.4): these are sensitivities of that swing, not of a swing that obeys the clock. Each case changes one thing (a case that needs two says so). The ball is tested two ways, both with the real bat (from the hands to its end at 0.784 m; past the end is a miss):

- **fixed point**: where the average swing's sweet spot is at contact. When the bat line passes it ("meet", + = late), where on the bat ("u" from the hands; sweet spot 0.634), the bat's speed there, and its azimuth against the average's (+ = further round, toward the pull side). A geometric sensitivity: a real ball is not at that point when the bat is early or late.
- **moving ball**: a ball on the pitch line through that point at 40 m/s [E], timed to reach it at the average swing's contact. Still no height (the probe is planar), no collision and no batted ball; in the probe's geometry a late bat tends to pass the deeper ball beyond its end. Neither test gives a spray angle or a rollover: those need the collision and the 3D bat.

**Which shift each fault means.** A timing fault moves the whole burst (onset and switch-off together): the hitter runs his practised drive, earlier or later. Moving the onset alone instead lengthens or shortens the burst, which is a different fault (a drive cut off on a fixed clock). The table gives the burst moved whole; the onset-only rows follow it for comparison.

| Case | Meet ms | u m | Speed m/s | Angle ° | Moving ball: meet, u | Pelvis / trunk at meeting ° | Legs' drive peak / friction cap then, N·m |
| :-- | --: | --: | --: | --: | :-- | :-- | :-- |
| average swing | 0 | 0.634 | 30.3 | 0 | 0, 0.634 | 64 / 62 | 201 / 297, never capped |
| late foot +60 ms (drive on its own clock) | 0 | 0.634 | 30.3 | 0 | 0, 0.634 | 64 / 62 | never capped |
| late foot +120 ms (own clock) | +3.9 | 0.633 | 29.9 | −1.4 | +1.7, 0.640 | 51 / 58 | 305 / 163, capped 209 steps |
| late foot +160 ms (own clock) | +28.0 | 0.562 | 24.5 | −8.8 | +11.9, 0.731 | 6 / 31 | 890 / 165, capped 331 steps |
| late foot +60 ms, the program waits for the foot (28 ms) | +28.1 | 0.629 | 30.0 | −1.3 | +13.0, miss | 63 / 60 | never capped |
| late foot +120 ms, waits 84 ms | +83.2 | 0.621 | 29.3 | −5.9 | +55, miss | 62 / 55 | never capped |
| late foot +160 ms, waits 122 ms | +120.1 | 0.616 | 28.9 | −8.6 | +91, miss | 61 / 52 | capped 14 steps |
| hips 20 ms early | −2.7 | 0.634 | 30.2 | +1.2 | −1.2, 0.637 | 79 / 64 | never capped |
| hips 40 ms early | −5.2 | 0.624 | 29.6 | +2.3 | −2.2, 0.632 | 90 / 65 | never capped |
| casting: 20 N·m wrist drive from −150 ms (two wrists' measured strength) | −1.3 | 0.591 | 29.0 | +4.5 | −0.5, 0.592 | 64 / 62 | never capped |
| casting: 10 N·m from −150 ms | −0.7 | 0.611 | 29.6 | +2.4 | −0.3, 0.613 | 64 / 62 | never capped |
| casting: 20 N·m from −100 ms | −0.6 | 0.620 | 30.0 | +1.4 | −0.3, 0.622 | 64 / 62 | never capped |
| pulling off: trunk 20 ms early | −10.0 | 0.661 | 31.7 | +3.0 | −4.5, 0.691 | 56 / 76 | never capped |
| pulling off: trunk 40 ms early | −14.5 | 0.671 | 32.1 | +4.4 | −6.7, 0.730 | 52 / 82 | never capped |
| lunging: rotation and brake held 60 ms | +58.9 | 0.623 | 29.5 | −4.3 | +34, miss | 62 / 57 | never capped |
| lunging: held 100 ms | +98.2 | 0.618 | 29.1 | −7.1 | +70, miss | 62 / 54 | never capped |
| a plain 15 ms late start (for scale) | +14.7 | 0.631 | 30.2 | −0.3 | +6.4, 0.669 | 64 / 61 | never capped |
| arms committed 30 ms later | +6.6 | 0.682 | 32.5 | +5.2 | +2.9, 0.681 | 75 / 91 | never capped |
| arms committed 60 ms later | +22.0 | 0.712 | 33.2 | +9.0 | +10.0, 0.752 | 91 / 125 | never capped |
| arms committed 90 ms later | +50.6 | 0.656 | 28.8 | +12.8 | +24, miss | 113 / 160 | never capped |

The same timing faults with the onset alone moved (the first version's construction):

| Case, onset only | Meet ms | u m | Speed m/s |
| :-- | --: | --: | --: |
| late foot +120, waits 84 | +98.2 | 0.444 | 17.9 |
| late foot +160, waits 122 | never reaches the ball point | | |
| hips 40 ms early | −6.6 | 0.648 | 31.6 |
| lunging: held 100 ms | +122.0 | 0.372 | 14.4 |
| late swing 15 ms | +15.2 | 0.622 | 29.4 |

Lunging with the stride's push also run on for the hold (a second change) is the same to 0.1 m/s: the push is small and friction-capped. The trigger in the wait rows is the lead foot's load reaching 10% BW plus 30 ms [E], found causally (the body run with the program held until the event).

What each shows, real and not yet:

- **Late foot.**
  - Up to about 60 ms late, on the hips' own clock, nothing happens: the foot is down well before the hips need it.
  - Later than that, the hips push against a foot that carries little weight. This is the one case where the friction cap binds (the legs' drive is held to about 165 N·m against 890 asked): the pelvis barely turns, and the ball is met late, toward the handle, weak. That is the walkthrough's "Too Late: slow load or late foot-plant → near the handle, weak", from the body.
  - If the program waits for the foot, the swing is intact but late by the wait plus a little. Against a real pitch that lateness is the fault: at 120 ms late the ball is long past (the moving ball passes beyond the bat's end).
  - The walkthrough says which: "Heel-Plant is the trigger" (§14).
- **Hips early.** Early and slightly pulled, the ball 1 cm toward the handle, 0.7 m/s slower. The walkthrough's "off the end of the bat; rollover to the pull side" is not reproduced: the meeting is only 5 ms early and the rollover needs the 3D bat and the collision. The force–length relation of the trunk (over-separation weakening the trunk's drive) is still a gap to fill in 3D, but it is not needed to make early hips cost speed, and it must be added only with a defensible law, not because the fault should lose power.
- **Casting.** The coach's casting is the hands drifting away from the back shoulder (the elbows and shoulders). The flat arm cannot do that, so the probe tests only the wrists uncocking early, at their measured strength: 1.3 m/s slower, 4 cm toward the handle, 4.5° further round. The real fault needs the elbows (phase 1).
- **Pulling off.** Early, toward the end of the bat, faster, pulled 4.4°. The walkthrough's barrel yanked out of the zone and head pulled away are vertical and perceptual: phase 1 (the body) and phase 3 (the eye point, §5.4).
- **Lunging.** Holding the rotation while the stride drifts on makes the flat body late, not weak: its weight cannot pass over the front foot (the lever rule keeps a third of it on the rear foot at the meeting, Neon's measurement), it has no front knee to collapse, and the drift's push is tiny. **The flat body does not produce the power loss.** It needs the 3D legs and vertical dynamics: the centre of mass passing over the front foot, the rear foot unloading, the knee bending instead of blocking.
- **The decision clock.** The hips and trunk on their own clock and the arms delayed: the body already turning buys the arms some time, but the trunk spins open without them (125° at 60 ms), the signature of pulling off. Because this fit's arms start too early, the numbers say only that the arms' delay does not carry one for one.

Not tested in the probe (each needs the 3D body or the pitcher): the collapsed front leg, flying open, early and late release, finger slip (§11).

## 7. Ratings, and the owner's consolidation question (§14.1 item 5)

### 7.1 Two separate questions

The first version mixed two things that are separate:

- **What the sim needs as causes.** The body integrates one channel per joint group that its physics and the breakdowns need (about twelve in 3D: legs' yaw, rear-leg push, lead-leg brake and block, trunk rotation, trunk tilt, both shoulders, both elbows, wrists, and the fingers for a pitcher). Each channel needs a strength T and a timing spread; the ratings feed those.
- **What a manager is shown.** Results he can read and scouts can measure: bat speed, swing time, exit velocity, fastball speed, command, and the body ratings that are causes and have a plain meaning (height, weight, strength, timing). Bat speed and command become results of the body, reported as before; they stop being inputs.

**Strength scales with size** [E]. At a fixed build, torque goes as muscle cross-section times moment arm, i.e. as mass; Anderson 2007 normalises leg torques by weight × height instead ([N5] §1). The two differ for a tall lean player against a short thick one (§7.2). Yanaka and Morishita 2026 ([N2] §2.4, abstract only) found the bat-speed correlation of hip and trunk torque to vanish once body mass was controlled.

### 7.2 What the probe shows

`ratings.py`, output `ratings.out` (and `ratings_MH.out` with torque scaled by weight × height). No hitter's swing is re-learned here: every hitter uses the average's program with his own strengths. Where a table says "re-timed", the whole program is moved so the bat reaches contact at 0 ms: a description of where his practised swing would start, not something the game runs.

**The same hitter swinging easier or harder, the program not moved** (every drive × 0.8 to × 1.2; D31: a weaker swing starts when planned and arrives late):

| Effort | Bat at contact angle | Bat speed | Fixed ball point: when, where | Path |
| :-- | :-- | :-- | :-- | :-- |
| × 0.8 | 19 ms late | 27.9 m/s | 17 ms late, 0.635 m | 2.00 m |
| × 0.9 | 9 ms late | 29.1 | 8 ms late, 0.634 | 2.00 |
| × 1.0 | on time | 30.3 | 0, 0.634 | 2.00 |
| × 1.1 | 6 ms early | 31.3 | 7 ms early, 0.633 | 2.00 |
| × 1.2 | 11 ms early | 32.2 | 13 ms early, 0.632 | 1.99 |

- The path is the same length at every effort. A slower swing by the same hitter is as long and so slower, as MLB's within-batter data say ([S10] §2: 0.7% shorter for a 4.5% slower swing at the same depth). This is the driven chain's similarity, and it comes out by itself.
- If he *plans* the easier swing, it is a different program, started 19 ms earlier for × 0.8 (the re-timed table in `ratings.out`, kept as a description).

**Across hitters, one strength rating with the average's program** gives the same path at every strength and a time ∝ 1/speed. MLB's hitters do not: length rises with bat speed, and length/speed is flat at 68–70 ms ([S10] §1).

**A construction, not a finding: the launch coil.** Giving each strength level one more freedom, the launch coil (pelvis, trunk, arms and bat all started further back or less far, together; no change in lag or separation), and scanning it to keep the time from the first drive at the average's 240 ms:

| Strength | Bat m/s | Coil, + = more than the average | Path m | Length / speed |
| :-- | --: | --: | --: | --: |
| × 0.70 | 22.9 | −45° (the scan's end; time not held: 246 ms) | 1.54 | 67.2 ms |
| × 0.85 | 26.3 | −27° | 1.72 | 65.2 ms |
| × 1.00 | 30.3 | 0 | 2.00 | 66.1 ms |
| × 1.15 | 33.4 | +25° | 2.29 | 68.6 ms |
| × 1.30 | 35.7 | +45° (the scan's end) | 2.56 | 71.6 ms |

Length/speed comes out flat at 65–72 ms and the slope is 0.12 ft per mph (MLB 0.05–0.09), but only with about ±25° of coil for ±15% strength, twice the between-hitter spread of coil in [6] (a different measure and cohort; a cross-check only, D21). That says this construction is not how real hitters get there; it does not show that coil should follow strength. The candidates are 3D: the hands' own load and path, the bat's vertical wrap (+44° to −30° in [4]), the stride, and stature and limb length, which go with bat speed across hitters. Phase 1 compares them with the same definitions as Savant's. (Savant's swing length is the bat head's 3D path and its length/speed is not a swing time; the probe's path is the sweet spot's in 2D from a speed threshold.)

**Two hitters with the same bat speed under the recommended split** (lower body: legs, push, brake; upper body: trunk, shoulders, wrists; strengths scaled together until the bat speed matches the average's; the program the average's, re-timed as a whole):

| Hitter | Lower, upper | Pelvis peak | Trunk peak | Arm peak | Pelvis at contact | 10 ms late start | Foot 150 ms late |
| :-- | :-- | --: | --: | --: | --: | :-- | :-- |
| strong lower, weak upper | 1.48, 0.86 | 724°/s | 811 | 1,189 | 95° | +10.0 ms, 0.0 m/s | +22 ms, −1.4 m/s |
| weak lower, strong upper | 0.69, 1.20 | 427°/s | 586 | 1,080 | 35° | +10.1, 0.0 | +26, −1.8 |
| tall lean, 1.95 m 85 kg (one rating) | 1.06 (by mass), 1.00 (by weight × height) | 578 | 697 | 1,108 | 64 | +10.2, 0.0 | +28, −1.9 |
| short thick, 1.75 m 100 kg (one rating) | 0.94 (by mass), 0.99 (by weight × height) | 583 | 705 | 1,148 | 64 | +10.2, 0.0 | +22, −1.4 |

- Two strength groups make clearly different hitters at the same bat speed: their hips turn at very different rates (visible on screen), and a late foot costs the upper-body hitter more. A plain timing error costs both the same.
- Neither hitter's timing was re-learned for his body, so the weak-lower hitter's pelvis is only 35° open at contact: a hitter of that build would learn a different sequence. The comparison is fair only as far as the program allows.
- Scaled by mass, the tall lean hitter needs a 13% higher rating than the short thick one for the same bat speed; scaled by weight × height, the same rating. Phase 1 chooses, with the 3D segments.

### 7.3 The levels compared

| | One strength rating per channel (6 in the probe, about 12 in 3D) | Two groups: lower body (legs, push, brake, block), upper body (trunk, shoulders, elbows, wrists) | One strength |
| :-- | :-- | :-- | :-- |
| Breakdowns it can express | all, each with its own proneness | all, as errors; proneness through the timing ratings | all, as errors, every hitter equally prone |
| Different hitters at the same bat speed | many ways, most invisible to a manager | yes: lower-body against upper-body hitter (§7.2), visible in hip speed and in lower-body breakdowns | only through `Height` and `Mass` |
| MLB's spreads | each channel needs a distribution and correlations, none published ([N2] §5) | two distributions and one correlation | one distribution |
| Unsourced numbers per player | about 12 strengths | 2 strengths | 1 strength |
| PLAN §7's rule (one meaning, one unit, one mechanism) | each a torque scale on its channel | each a torque scale on its group's channels (dimensionless; a group mixes torques and leg forces) | a scale on all channels |

No per-channel alternative was fitted under the same controls, so the comparison is of what each level *can* express, not of measured fits.

### 7.4 Recommendation

The model integrates every channel the 3D body needs. As causes, a player has:

| Rating | Meaning | Replaces |
| :-- | :-- | :-- |
| `Height`, `Mass` (kept) | segment sizes, masses and inertias ([dL]); strength scales with size (by mass or by weight × height: phase 1) | — |
| `LowerStrength` | a dimensionless scale on the legs' channels: hip rotation, push, brake and block | part of `BatSpeed` and `ArmSpeed` |
| `UpperStrength` | a dimensionless scale on the trunk's and arms' channels (for a pitcher, the throwing arm's) | the rest of `BatSpeed` and `ArmSpeed` |
| `Coordination` | a dimensionless multiplier on every link's baseline timing spread σ₀,k (§6.1): how precisely he sequences his body | `BatControl` and `Command`, as their cause |
| `Timing` (the owner's question, §14) | a multiplier on the spread of the stride and trigger against the pitch: how well he syncs to the pitcher | `Timing`, as its cause |

Load depth is a style trait of his learned swing, drawn in an observed range [E], independent of strength until phase 1 says otherwise. `Adjustability` stays as the speed of a mid-swing re-blend (§5.5). `SpinCapacity` and a pitch's own feel stay for the fingers (§11). `Stamina` stays: fatigue is less drive.

The manager is shown the results (bat speed, swing time, exit velocity, fastball speed, command, contact quality) and the body ratings that have a plain meaning.

Why two strength groups and not one: one rating cannot make a lower-body hitter and an upper-body hitter (§7.2), and the owner's breakdowns split along that line (the late foot, the block and lunging are lower body; casting and the release are upper body). Why not more: no source gives a per-channel distribution. A third group (trunk) can be split off later without changing the model, if phase 4 finds residual differences that the two cannot carry.

## 8. Replay and speed (§14.1 item 6)

### 8.1 The integrator

**Classical fourth-order Runge–Kutta at a fixed step** [E: 1 ms through the swing, the stride's landing and the arm's whip; 2–4 ms in the quiet phases (stance, load, leg lift); both to be confirmed by phase 1's convergence tests].

Activation is advanced once per step by its exact exponential (`m::exp`) and held through the step's stages (phase 1 can integrate it as a state instead, which removes the tenth-of-a-millisecond wobble below). The non-smooth parts (contacts switching, range stops, friction) are evaluated at each stage's state in a fixed order. Every transcendental goes through `bb_math::m`, the linear solves use partial pivoting in a fixed order, no FMA, no fast-math, no hash iteration, so the same build, seed and commands give the same motion bit for bit (invariant 5).

**The 3D engine as built** (`analysis/oracle/body3d/engine.py`, phase 1 session 1):

- **Formulation: maximal coordinates.** Every segment is a free rigid body (centre of mass, unit quaternion, linear and angular velocity). Every joint is a set of constraint rows made from two primitives: a point of one body on a point of the other (3 rows), and an axis of one body square to an axis of the other (1 row). A ball joint is a point; a two-axis joint a point and one axis pair; a hinge a point and two; a weld a point and three. Each derivative evaluation solves (J M⁻¹ Jᵀ) λ = γ − J M⁻¹ F by Cholesky in a fixed order, 49 rows for the batter. Chosen over minimal (joint-angle) coordinates because the hands' closed loop is then two ordinary joints, every joint type is the same few lines, and the equations can be checked against joint-angle derivations that share no code with them. Its cost is the larger solve (§8.2).
- **Constraint drift:** Baumgarte's stabilisation, C̈ + 2aĊ + b²C = 0 with a = b = 50 s⁻¹ [E], evaluated at every stage. Measured on the driven body over 80 ms: 0.33 µm at a 1 ms step, 0.01 µm at 0.5 ms; 19 µm without stabilisation. No projection per step. In the violent hand-set runs of §4.4 (stops and landings) the drift reaches 10 µm at 1 ms, so the fit should run at 0.5 ms or with stronger gains until the event tests settle it.
- **Contact:** points on each sole (heel and ball, inside and outside) against the ground plane. The normal force is a one-sided spring and damper that is never negative; friction is a stick spring to an anchor, capped by the cone μN, the anchor dragged when the cap is reached. Anchors move once per step and are held through the stages, like the activation's corner. Four points per foot give §4.1's requirements without more machinery: no sinking beyond the spring's fraction of a millimetre, the centre of pressure inside the sole (each point's force is ≥ 0), and the foot's horizontal force and twisting moment from one set of contact forces.
- **Joint limits:** one-sided spring-damper stops on an angle: the knees' and elbows' straight end, and (session 2) each shoulder's end of range across the chest, 135° from the arm straight out to its side [E: the clinical normal range is 130–135°]. A stop's torque is clamped so it can push out of the limit and never pull. The lead arm is placed at that limit at −250 ms, so the turning trunk pushes it; in the fitted swing the stop carries at most 63 N·m (§4.4). The wrists have no stop unless a run says so.
- **A drive about a moving axis** (session 7, §4.5): a drive may act about the axis square to a direction fixed in its parent and one fixed in its child (the unit vector along their cross product), which is how raising and lowering the arm are driven: square to the trunk's long axis and to the upper arm. Torque and reaction are equal and opposite about that axis, and its speed is the two bodies' relative angular velocity about it, so the momentum and work accounts hold as for a fixed axis. Where the two directions are parallel (the arm straight down along the trunk) there is no such axis and the drive gives nothing [E: within 10⁻⁹ of parallel]. Rotation about a segment's own long axis (the upper arm, the forearm) is a fixed axis of that segment, driven as the hips are.
- **Strength at speed** (session 5, §4.5): a drive either follows the first law of §5.1 or a measured curve, the fraction of full strength at the joint's speed as straight lines between points; a muscle being stretched rises toward the curve's own plateau (1.5 × for the legs and trunk, 1.4 × for the arms) on the curve's own top speed.
- **Muscles tensed beforehand** (§4.5): a drive's excitation holds a starting level until its onset and then rises to full; its activation starts at that level.
- **Tendon stretch** (§4.5): a drive may act through a spring in series. The joint gets the spring's stiffness times its stretch. The stretch follows stretch′ = (the muscle's shortening speed) − (the joint's), where the muscle's speed is the one at which its strength-at-speed curve gives the spring's torque. It is advanced once per step by a backward Euler step solved with 40 bisections (fixed, for replay) and held through the step's stages, like the friction anchors, so any stiffness is stable; it is first order in the step.
- **A failed solve** (a pivot that is not safely positive: a singular posture or redundant joints) raises an error. It is never regularised away.
- **Activation** is integrated as a state (15 ms up, 50 ms down), not held through the step.
- **Work** of the drives, stops, damping and contacts is integrated beside the motion as four more states.

**Its checks** (`analysis/oracle/test_body3d.py`, 29 tests, each against something the engine did not produce):

- a free symmetric body against Euler's closed form (the body-frame precession, the cone about the angular momentum, the straight-line centre): 2 × 10⁻⁹;
- a hinged rod released at 2.0 rad against the pendulum's exact (elliptic) period: 5 × 10⁻⁹ rad;
- a double pendulum against Lagrange's equations in its two angles, integrated in the test: 2 × 10⁻⁸ rad after 1.5 s;
- the Euler–Lagrange identity for a 3D chain (ball, two-axis joint, hinge) at three moving states: the engine's accelerations against those from the chain's own joint-angle kinematics, differentiated by the complex step: 2 × 10⁻⁶ (the differences' level);
- a closed loop with an exact answer (a parallelogram linkage is a pendulum): back at its start after one exact period to 2 × 10⁻⁸ rad, the loop shut to 10⁻¹⁰ m;
- the whole body off the ground with every drive on: linear momentum constant to rounding on three axes, angular momentum to 7 × 10⁻⁸ kg·m²/s (the size the joints' drift explains);
- the placed body on [4]'s curves at the start, both knees included, and moving at [4]'s rates (read from [4]'s own rate curves, which the placement does not use);
- the whole body on the ground, driven: energy gained equals work booked to 2 mJ in 16 J at 1 ms, 0.07 mJ at 0.5 ms and 0.004 mJ at 0.25 ms (fourth order);
- a block on four contact points: it stays under μmg, accelerates at F/m − μg above it, and no contact force is ever negative, also when dropped;
- a pendulum swung into a stop: the stop never pulls;
- a measured speed curve passes through its points, falls with speed, and its inverse undoes it on both sides, also with the arms' plateau; the arms' bounds, curve points and tendon angles are §4.5's, typed into the test from its tables;
- a tendon spring against a body too heavy to move ends carrying exactly the muscle's full strength at its activation;
- **a drive about a moving axis** (session 7): a rod on a ball joint, turning about the vertical, raised by a constant torque about the axis square to the vertical and to itself, against the same motion written in spherical coordinates and integrated in the test: 2 × 10⁻⁷ rad after 0.3 s at a 0.5 ms step, 10⁻⁸ at half of it;
- **every one of session 7's 19 channels turns its joint the way its name says** (and the elbows' extensors and both trunk channels with them): each alone at its bound on the placed body, the change it makes to its joint's angular acceleration is taken along a direction the test builds from the joint centres and the anatomical definition (raising takes the arm away from hanging along the trunk; internal rotation of the right arm is a positive turn about the line from elbow to shoulder, the left arm's the other way; and so on). A channel with its sign turned fails it (tried: −765 rad/s² for the rear shoulder's rotation);
- session 7's bounds, curve points and tendon angles are §4.5's, typed into the test from its tables; the one player's forearm strength does not scale with the batter's size;
- the whole body's momentum and work checks (above) now run with all 41 channels on;
- **the tendon's energy:** for a rod driven through a spring, the muscle's own work (its curve at its own speed) equals the work the rod receives plus the energy in the spring, to 0.06 J in 73 J at a 0.25 ms step, halving with the step (0.11, 0.22 J at 0.5 and 1 ms);
- **mutations:** the moving axis taken the wrong way round (child's direction first) lowers the rod it should raise and fails the spherical check by 0.3 rad or more; the spring advanced with the joint's speed reversed breaks that account by 1,400 J; the gyroscopic term removed fails the free body; the centripetal term's sign flipped fails the double pendulum and the identity; a drive's reaction on its parent removed fails angular momentum.

**No passive element does net positive work (session 8's review, P1-1).** A stop reads one angle between two bodies. Its torque and its damping now act along that angle's own gradient (the vector g with d(angle)/dt = ω_rel · g), and its work is booked with the same g; until this fix the torque acted about the stop's fixed axis, which is the gradient only on a hinge, and on a ball joint the stop did work that nothing stored paid for. Two kinds of stop: the angle of a direction projected about an axis (g = n − (n·f) f_p / |f_p|²), and the angle of a direction out of the plane square to an axis (g = (f × n) / |f × n|). Each is undefined where the direction lies along the axis; within 5° of that the run fails loudly (`STOP_SINGULAR`) instead of clamping, and §4.10 says where that lies for each stop. Tests: a rod alone on a ball joint, tilted 0, 30 and 60° out of the stop's plane, never has more kinetic plus stored energy than it started with, loses with a damper, and gives all of it back without one, for both kinds; the old law fails that check (the rod's energy grows more than tenfold); a stop at its singular posture raises. The compiled probe has the same law (`probe3d.py --check`). **The joints' ranges (session 8 and its fix, §4.10):** after the review, tests that look where the defects were (the cock's reading with flexion carried through 130°; the wrists' travel against the engine's own uncocking; the hip's sideways angle at 0, 45 and 70° of flexion and the arm hanging; an arm muscle tensed beforehand held by re-solved hold-still torques); after the second review, the forearm's turn with the cock carried through the line of the bat, and each wrist's range as an angle between forearm and barrel inside what a wrist reaches (the rear hand's is an expected failure until each hand has its own grip); and the first two: the ranges' angles against the knees', elbows' and trunk's angles read another way, every stop clear of the start and carrying nothing there, the old body's stops unchanged; and a stop pushing back by its stiffness times the excess, only past its end. **Still blind:** the segment table itself beyond its mass sum; the ground model's realism; a drive axis wrong alike in the model and in what the fit compares; event-time location (not built). **What the 3D body still needs specified:** how the hands' loop and the feet's contacts are solved (a deterministic constrained solve, its iteration count and tie order fixed), how constraint drift is held (projection or stabilisation, and whether it is applied per step), how a contact's making or breaking and an impact are handled, and what happens when a solve fails. Events (toe touch, heel contact, the bat passing a point, release) are located within the step by cubic Hermite interpolation of the state, or by sub-stepping at 0.1 ms around release and contact [E], with a convergence test against finer location (§11 says why release needs it).

**Checks** (`checks.py`, output `checks.out`, on the free fit):

- **Energy.** The kinetic energy gained equals the work of every force, integrated with the same stages, to 0.4 mJ in 424 J (1 × 10⁻⁶).
- **No forces** from −100 ms: kinetic energy constant to 1 × 10⁻¹¹; the stops checked inactive.
- **Internal drives only:** forward momentum constant to 8 × 10⁻¹⁰ N·s.
- **Angular momentum** about the moving base point: its change equals the impulse of the external yaw loads (the ground's couple and the base's motion), to 2 × 10⁻¹⁰ relative.
- **The Euler–Lagrange identity** at 20 random states: M·q̈ + h against the Lagrangian's derivatives by finite differences, to 1 × 10⁻⁴ (the finite differences' own level).
- **Step size:** at 2, 1, 0.5 and 0.25 ms, contact at +1.445, +1.463, +1.433 and +1.420 ms, the sweet-spot speed 30.244–30.252 m/s.
- **Mutations** show what each check sees. A velocity term that does no work (skew-symmetric) passes the energy check (4 × 10⁻⁸) and fails the Euler–Lagrange identity (1.1) and angular momentum (1.6 × 10⁻³ against 7 × 10⁻¹⁰). A dropped reaction on the pelvis, booked consistently in the power, passes energy and fails angular momentum (0.76). A sign flipped alike in the equations and the power passes every check.

**Blind spots:** the segment masses and lengths themselves; a force or torque direction wrong alike in the equations and the power; whether the ground model is realistic (only its bookkeeping is checked); everything vertical. Phase 1 adds 3-axis linear and angular momentum for the free-floating body, the loop's drift, the joint limits' energy, work from measured external forces against [4]'s ground forces, analytic free-body and constrained cases, and event-time convergence.

### 8.2 The cost

**Measured:** the flat chain in Python, 0.15 s per 270 ms swing at a 0.5 ms step; in Rust (`bench_chain.rs`, opt-level 3, one core), **1.7 µs per RK4 step**, 0.46 ms per swing at 1 ms. The Rust loop uses native math, a constant yaw cap and no load split, trunk stop, recording or contact solve: a rough cost for a 5-degree-of-freedom loop of arithmetic, nothing more.

**Measured for the 3D body (phase 1 session 1):**

- **Python** (the checking engine): 7.9 ms per RK4 step for the full batter (13 bodies, 49 joint rows, 8 contact points), 2.5 s for a swing from −250 ms to +60 ms at 1 ms. Too slow to fit with: a fit needs thousands of swings.
- **Rust, the whole step** (`body3d_probe.rs`, a copy of the engine operation for operation; opt-level 3, one core, scalar arithmetic in a fixed order, no SIMD intrinsics, no `mul_add`; `probe3d.py --bench`, session 2): **116 µs per RK4 step** with the dense solve in the Python engine's row order, **50 µs** with the joint rows ordered along the body's tree so that the Cholesky factor skips each row's leading zeros. The reordering changes nothing but rounding: both agree with the Python engine to 2 × 10⁻¹² on every state entry over four runs of 150–340 steps (undriven, every drive on its bound with stops and landings, off the ground, and the fitted swing; `probe3d.py --check`, tolerance 1 × 10⁻⁹). The probe allocates its work arrays every evaluation and reads no tables, so 50 µs is not a floor.
- **So a swing costs 16 ms** (310 steps), one fit evaluation 34 ms with the Python side's bookkeeping, and a three-start fit of 64 numbers 6–7 minutes (11,400 swings).
- **A game:** 290 pitches at 800–1,600 steps each (the table below) at 50 µs is **about 12–23 s**, for the full body with nothing tabulated, against PLAN §13.4's 200 ms. That is the measured version of the table's first row and sits at its low end.

**The estimates made before that measurement** (kept for the reduced core and the per-pitch step counts):

| | Per RK4 step [E] | Per pitch [E] | Per game, 290 pitches [E] |
| :-- | :-- | :-- | :-- |
| full body (35 joints, two feet in contact, the hands' loop, `libm`) | 40–160 µs (Featherstone's articulated-body algorithm, about 450 flops per joint per evaluation, times 2–5 for the loop and contact solves) | 800–1,600 steps (the batter from the stride to the finish, the pitcher from the leg lift to release, 1–2 ms) | 9–75 s |
| a reduced core keeping the breakdowns' parts (about 20 joints, below) | 15–60 µs | 500–1,600 steps | 2–30 s |

Against PLAN §13.4's 200 ms per game, every 3D option integrated on each pitch is 10–400 × over.

**What a reduced core would give up.** The first version listed a "12-degree-of-freedom core"; it counted 11, and it dropped:

- the legs (hips, knees, ankles): no block, no collapsed front leg, no real weight transfer, no feet's contact wrench;
- the rear arm: no two-handed loop, no grip couple, no rear elbow's push;
- forearm rotation: with an elbow hinge, the hands cannot turn the barrel onto different pitch planes (the bat's roll about its own axis does not move its axis);
- the pelvis's vertical and sideways motion: no balance, no lunge;
- for the pitcher, the throwing arm and fingers.

Those are the parts the breakdowns and the 3D fit need. A core that keeps them is about 20 joints, and what it saves over the full body is modest. Which poses and breakdowns a production body must preserve before it may replace the full body is for phase 1 to measure and the owner to accept.

**Tabulating what no error reaches buys little.** The batter's load and stride are timed to the pitcher and vary every pitch. The pitcher's stride, block and trunk carry his lower-body faults (flying open, a collapsed front leg) and fatigue, so his delivery must be integrated from the leg lift on every pitch. Only the set position before the first draw can be tabulated. (The first version tabulated the delivery up to the arm's acceleration, which would have removed the pitcher's lower-body faults.)

**Building the learned swings** (§5.5) has a cost of its own [E]: 25–75 programs a batter, 300–3,000 evaluations each, 1–10 ms an evaluation of a 0.3 s swing in 3D, so roughly 10 s to 40 min a player. That cannot run per game: the library is built once per player and stored with him (the league's file), deterministically (fixed start, fixed iteration order), and `sim-cli games` with fresh rosters needs a pre-built player pool or a cache. Phase 3 measures it.

**So the budget question waits** for phase 1's benchmark of a real constrained 3D step and phase 3's library cost (§14). What can be said now: a flat body would fit 200 ms but cannot swing up or down at a pitch, so it cannot carry games; any 3D body will cost seconds a game.

## 9. What the timeline carries, and what the client draws (§14.1 item 7)

PLAN D9 is decided: a saved play stores the build's commit and the recorded play itself, so the client shows it as seen on any later build. The body complies:

- **The timeline carries the body's sampled joint coordinates** for each moving player: the pelvis's position and orientation and every joint angle, q(t), with the events at their exact times (toe touch, heel contact, release, contact) and the state at each event. The client poses the skeleton from q(t) by forward kinematics; it never re-runs the physics.
- **Size** [E, to be set by measured pose error in phase 2]: the 3D body has 41 coordinates (6 for the pelvis, 35 joints). At 120 Hz, stored as 16-bit values, that is about 10 kB per second of motion: a batter's 1 s (load to finish) and a pitcher's 1.8 s come to about 27 kB a pitch, 8 MB a game, before delta coding. Around contact and release the bat and the hand move a centimetre in a fraction of a millisecond, so the samples there are denser, or each sample carries the velocities for Hermite interpolation (twice the size). Phase 2 sets the rate so that the bat's tip and the hands, interpolated, are within 1 cm [E] of the integrated body.
- **Only watched and saved plays record poses.** Headless games (`sim-cli games`, seasons) record none.
- **The executed drive program** (onsets, rises, offs, strengths, the per-tick sensing draws and the decisions' excitation changes) may be stored as a same-build diagnostic: with the seed, it reproduces the motion on the same build only. It is never the replay.
- **Hand IK is not needed.** The hands are on the bat because the body put them there.
- **Contact and the follow-through.** At contact the body receives the reaction of the bat–ball collision (physics-spec §8) as an impulse on the bat, equal and opposite to the ball's; it changes only the continuation of the motion. The follow-through is integrated past contact with the drives switching to braking (the eccentric absorption of [N2] §2.4) until the finish. PLAN D24 (the sim's bat track ends 0.03 s after contact) then applies only to what the world reads; the timeline carries the body's own finish.

## 10. Where the model lives in the crates

- **`bb_physics::body`** (new, pure):
  - the segments, joints, contacts and loop;
  - the drive law of §5.1;
  - the postural feedback law of §5.3, as a function of a sensed state passed in;
  - the integrator of §8.1.

  No randomness, no world types. Both the agent, on his own body and beliefs, and the world, on the true body, call it.
- **`bb_agents`:** the player's learned swings and deliveries (built once per player, §8.2), the blend between aims, the decisions that start, hold or stop channels.
- **`bb_agents::motor`:** the execution errors of §6.1 and the per-tick sensing noise of §5.4, drawn from `KeyedRng` with new `Purpose` values (appended, never reused) and stable sub-indices. It returns an `ExecutedDrive` (the program, the sensing draws, and later excitation changes) that only `bb_game` and the world read (invariant 3).
- **`bb_world`:** integrates the true body with the executed drive, tick by tick, and samples the bat's pose in time into `BatPose`. `detect` and `resolve_contact` are unchanged; the collision's reaction goes back to the body (§9). The pitcher's release (the ball's state as it leaves the fingers) becomes the world's input to `FlightEnv`, in place of today's planned-plus-noise release.
- **`bb_game`:**
  - orchestrates the pitch tick by tick (§5.4): the world's body and ball, the batter's observations, his decisions, the motor's changes;
  - builds or loads the players' learned libraries;
  - records the sampled poses of watched plays (§9).

  The crate graph test stays: `bb_agents` uses `bb_physics`, not `bb_world`.

## 11. The pitcher, at design depth

- **The body.** The same engine and the same segment table, with:
  - the pivot (rear) foot planted on the rubber;
  - the stride foot free through the leg lift, which is held on one foot by the postural feedback (§5.3: on one foot, errors of 2–3° are already more than the reflex can hold, so the leg lift is phase 5's hardest balance case);
  - the stride landing as a contact with the mound's slope (the mound's geometry is already in the game);
  - the throwing arm's shoulder (3, its internal rotation the fastest joint in sport), elbow (flexion and forearm rotation) and wrist (2);
  - a **series elastic element at the shoulder** [E: its deformation, stiffness, damping and geometry to be fitted, and checked on stored and returned energy as well as speed];
  - the ball held in the hand (below).

  The generators drive the legs, the trunk and the shoulder's early motion. The elbow's and wrist's late speed comes from the segments' dynamics and the elastic recoil (§5.1, with why the two shoulder numbers are not simultaneous). The published shoulder and elbow torques (ASMI: internal rotation 90–100 N·m near maximum external rotation, horizontal adduction 90–120, elbow flexion 49–73, [N3]) are checks on the model, not drive profiles. The delivery's timing pattern is similar across levels ([N3]: pelvis peak at about a quarter, trunk at about half the foot-contact-to-release interval); whether one delivery shape per pitcher, scaled by his strengths, carries every pitch type and arm slot is to be tested, not assumed.
- **The ball while held.** The ball is a body of its own, pressed into the fingers and palm by contact forces with normal compliance, friction and slip [E]. The fingers' drive changes the grip; the ball separates when the contact forces can no longer hold it against its acceleration, not at a scheduled instant. Its spin comes from the torques of the contact forces through the grip and the roll-off, integrated; its position, velocity and angular velocity at separation go to `FlightEnv`. A grip is a learned parameter set per pitch type; a premature or failed release is whatever the contact does. Kinoshita's finger forces **peak** 4–7 ms before release ([N3] §4); the paper does not say they end there.
- **Release sensitivity against measured command: a requirement to verify.** At the hand's path speed over its radius (40 m/s over about 0.7 m [E]), the ball's direction turns about 3.3° per ms, so 1 ms of release timing would move the pitch about 0.9 m at the plate. Grade-50 command is σ 10 mrad (0.57°, about 17 cm per axis at the plate; calibration-targets §4.2), which at that rate would need 0.2 ms of timing, finer than the fingers' measured 1 ms (Hore and Watts 2011, abstract only). Real pitchers command the ball because the hand's path straightens near release (elbow extension, wrist flexion and trunk flexion together lengthen its radius). Phase 5 must show that its delivery's hand path is straight enough near release (|dv̂/dt| of roughly 0.3–0.6°/ms [E]) to give MLB's command from finger timing at its measured precision. If it is not, the delivery model is wrong, and no release-noise scale may be shrunk to hide it.
- **Release in the integrator.** Release is located by sub-stepping or Hermite interpolation (§8.1), never by linear interpolation in a 1 ms step: at 6.7°/ms of shoulder rotation that alone gives direction errors the size of elite command. A convergence test against finer location is part of phase 5's gate.
- **Blending between aims** must not bias the aim: the grid's interpolation error is tested to well under the command σ [E: 2 mrad].
- **Directions are tests, not rules.** "Early release: up and arm-side", "late: down and glove-side", a backed-up slider from a finger slip, a collapsed block, flying open, fatigue reducing speed: each is a mechanism to check in the body, with possible exceptions, never a direction coded in.
- **Flying open** is the trunk's drive early against the stride foot's plant (as pulling off, §6.2); the arm drags. **A collapsed front leg** is the lead leg's block too weak or late: the knee bends after foot strike, the body's forward motion is not stopped, and the arm's whip loses what the block would have given it (Felton et al. 2023's bowling model, [N1] §1.2: a more extended front knee is common to every optimised delivery). **Fatigue** is less drive and slower activation; the onsets' spread grows.
- **The numbers:** Attias et al. 2026's OpenSim model of one MLB starter (34 DOF, [N1] §1.4), the bowling generators, and the ASMI joint kinetics of [N3]. The pitcher's equivalent of [4] (a live-pitching set of professionals with means and bands over time) is not among the notes; Fleisig's ASMI professional series (n = 60) exists but was not opened as numbers ([N3] §7). Getting it is phase 5's first task. OpenBiomechanics' pitching data stay a cross-check only (D21, [N7]).

## 12. Not established, and what would settle it

- **Whether a 3D body can swing under the design's rules** (§4.3): the decision clock, the 50 ms rise floor and sourced strengths. Phase 1's first question; the work stops if it cannot.
- **What holds the wrists' cock** in 3D, and whether a passive limit exists (§4.3).
- **The arms' clock at game speeds** (§5.4, §4.10): how long a batter must watch a pitch before he can decide [E: 0.10 s here; the game's own batter watches about 0.2 s].
- **Strength by joint group for professionals in N·m**, its spread, and any value above about 5 rad/s ([N2] §5, [N5] §6).
- **Movement variability**: no σ of onsets or strengths is published ([N6]); the sizes are [E] and set by outcome spreads with perception and location active (§6.1).
- **What makes a stronger hitter's swing longer** (§7.2): not the coil alone.
- **The trunk's force–length relation** and its axial range.
- **Whether a 3D swing needs feedback** (§5.3), and balance on one foot in 3D.
- **The cost of a constrained 3D step** and of building the learned libraries (§8.2).
- **What a joint does at and past the end of its range** (§4.10; PLAN D39): nothing is measured past the end for any joint ([note 11](research/body/11-hands-and-joint-ends.md) and [note 13](research/body/13-joint-ranges.md)); how much energy the end takes and what load counts as heavy are [E].
- **The hands on the handle** (§4.1, §4.10): each hand's own roll, the wrist's range while gripping, and a drive that holds without net torque before it fires.
- *No longer open questions for the gate (PLAN D37; history in §4.4–§4.9, §5.2):* the 3D body's fit to [4] and [3]; the arms' onset relative to [4]'s first move; the angular impulse [4]'s lateral couple does not give; how much the impact's smoothing took off [4]'s bat curves; the hands' path in [4]'s swing.
- **Licences for shipped numbers** ([N7]): no open, commercially licensed adult dataset with ground forces exists, so shipped values come from published means and SDs, cited. The research notes do not settle reuse of digitised curves or Statcast data; D21's cross-check restriction stays.

## 13. Phases 1 to 5, sized

**PLAN §14.1's rule, copied:** phase 1's gate is that "driven by the drive derived from the measurements, it reproduces the measured swing within the measurements' own error; momentum and energy checks pass; each named breakdown gives its consequence. **If it cannot, the work stops here and the owner is told what blocks it.** No Rust before this." Today's swing and release stay in games, unchanged, through phases 1–3.

**Phase 1's gate (decided: owner, Q62 = a, PLAN D40, 2026-11-25).** The gate written here until 2026-11-25 (the fitted swing within bands of [4]'s rate curves, its angles at contact, its ground forces and its contact instant) was superseded that day by PLAN D37; its text is in this file at commit `5488127`. PLAN §14.1's "reproduces the measured swing" went with it.

- **Pass, all of:**
  - *the practice search of §4.10* (the aim's pitch; strengths inside §4.5's bounds; rises of at least 50 ms; the joints' ranges; no arm drive before the batter could have decided) finds, for the average batter, a swing that meets the ball within 1 cm in balance, from at least two starts that owe each other nothing and agree within 1 m/s, the same within 0.5 m/s at a quarter of the time step;
  - *against hitters measured directly* (never given to the search; sources and their processing in §4.10's table): the sweet spot at 29–34 m/s at contact (owner, Q59 = a); the hands' fastest speed 10.3–13.0 m/s; the attack angle between −3° and +11°; the bat at 98% or more of its fastest at contact; the muscles' first rises in the order legs, trunk, arms; the lead knee 2–26° bent at contact and straightening into it; the front foot's greatest load 1.5–2.6 times body weight; the hips' fastest turning 555–855°/s and the trunk's 755–1,075°/s (each sample's mean ± 2 SD); the hands' first motion 130–220 ms before contact. *Which rule made each band* (review P3; no band is changed here): the bat's speed is the owner's (Q59 = a); the hands' speed is the vendor's own range for major leaguers; the attack angle and the front foot's load are OpenBiomechanics' 10th–90th percentiles, used because the source gives them and both are skewed (the foot's load cannot go below about 1, the attack angle's SD is as large as its mean); the hips' and trunk's turning are mean ± 2 SD because their percentiles (602–797 and 806–1,011°/s) are of mostly college hitters and the professionals in the same set sit at their upper end; the lead knee is mean ± 1 SD (2–26°) because ± 2 SD would run to −10°, past straight, and to 38°, which is not a braced leg; the swing time runs from the vendor's 130 ms to the top of college hitters' 215 ± 44 ms less one SD. They are not one rule; each is the tightest band its source supports, and none is changed here;
  - *the body:* no joint past its range by more than 2°; no joint without a stop travelling further than its real range; and no joint's end but the lead arm's against the chest carrying a heavy load (the owner has ruled this, Q61 = b, PLAN D39; the size of "heavy" is the next session's to state before it runs: nothing measured stands behind the 50 N·m first written here, and [note 11](research/body/11-hands-and-joint-ends.md) B3 has ligaments failing at 13–34 N·m);
  - *the engine's checks:* momentum, angular momentum, energy and the Euler–Lagrange identity to their tolerances, **and no passive element doing net positive work** (a stop, a damper or the ground never ends a run having given the body more energy than it took; added after session 8's review, §8.1); every practice result reports the stops' and the ground's net work;
  - *every named breakdown* run as one change (or a stated two) on the practised swing, its consequence reported as the body gives it.
- **Reported, not gated:** the recorded muscle levels of [N9] §4.1 (second hand); the elbows at contact; the trunk's twist against the hips; [3]'s lead arm rate; how much the result rests on the start, on effort's weight and on each [E] of the stops; the strongest batter and the small frame of PLAN D32; whether it looks like a hitter's swing as [4] shows one (for the eye only).
- **Stop:** if no practised swing meets the ball in balance inside the joints' ranges, or the one that does is outside the bat's speed band with every strength on its bound. The owner is told what the body lacks.

**Phase 1 after its tenth session (2026-11-26):** not passed. The start is off the ends of its joints by a stated rule, the wrists have an oblique range, and the practice search no longer stalls (§4.12). The body with real hands swings: the ball met within 0.2 cm at 17.7 m/s, legs then trunk then arms, settled, the same at a quarter of the step. Against the gate it passes "meets the ball", the order of first rises, "does not jump" and the passivity check, and fails the rest: the bat's speed by 11.3 m/s, the hands, the hips and trunk, the front leg, the attack angle by 0.6°, the clock by 3 ms, balance by 0.4 cm, fifteen joint ends loaded, one start only. Whether the body can reach 29 m/s is not shown; the test built to settle it without a search did not decide. [The independent review](reviews/2026-11-26-opus-body-3d-s10-review.md) confirmed the build and found that the joints' damping (unsourced) takes 64% of the muscles' work in that swing and the hold-still torques (under no muscle rule) give 27% of its energy, with nine muscle groups at 92–97% of their strength (§4.12, "After the independent review"). One of PLAN D41's three sessions remains. **The order from here** is "Status and what is left", items 1–8: the damping at a read value and a hold that lets go or follows the muscles' law first.

**Phase 1 after its ninth session (2026-11-26):** not passed, and further from it. The body has a hitter's hands, a muscle that only holds and cleats' grip, each tested (§4.11). With real hands session 8's swing cannot be made and the practice search finds none from its three starts: the nearest motion passes 7.9 cm from the ball at 19.0 m/s and fails every check but "does not jump" and the passivity check. Whether the body can swing is not shown either way (the search stalls; the start sits at joint ends; no plan was made for this body). Two of PLAN D41's three sessions remain. [The independent review](reviews/2026-11-26-opus-body-3d-s9-review.md) found nothing built at fault and put the search and the start first among the causes. **The order from here** is "Status and what is left", items 1–7: the clock term (done), a start off the ends with the rear wrist inside a real wrist's reach, a built motion solved by inverse dynamics as the test of whether the body can swing without a search (a diagnosis, no target), the search at a finer step by a method without slopes, then D39 and the rest.

**Phase 1 after its eighth session and its two reviews' fixes (2026-11-26):** not passed. The first practice search ran on a stop law that created energy; corrected and rerun, the search finds the first swing on stops that make no energy (17.8 m/s, from the plan fitted to [4]), which leans on joint ends beyond what a joint can carry and fails the gate on speed and on how it is made (§4.10's last blocks, §4.4). The earlier sessions' results below were all matched to [4] on a body whose joints nothing held and whose shoulder stops added energy.

**Phase 1's order as it stood before session 9** (both reviews'; items 1–3 were session 9's and are built; the order now is "Status and what is left"): (1) each hand's own grip, with the rear wrist inside a wrist's range and the forearm's and upper arm's turn held; (2) how a muscle tensed before the pitch holds, then the clock's question for the owner; (3) a rear foot that can pivot or a knee that can twist, as the cheapest test of what holds the speed; (4) PLAN D39 built, its sizes written first, with [note 11](research/body/11-hands-and-joint-ends.md) B's loads beside them; (5) the score's parked basin; (6) a start that owes [4] nothing; (7) the practice runs from independent starts and this gate.

**Phase 1 after its seventh session (2026-11-25):** with every freedom §4.1 designs driven on measured limits, the fitted swing meets the gate's contact time (+0.8 ms), every rate curve (pelvis 0.49, trunk 0.57, bat 0.62 band units), the three angles at contact and the ground forces, under the rules. Outside: the bat's speed, 21.3 m/s, on every reading (29–34; [4]'s own motion 22.9; that motion with the impact's smoothing undone 26–33 on the shapes that reproduce [4]'s curve, an estimate, §4.8), and the attack angle (−7.2°, the bat coming down on the ball; §4.9). The gate's lower figure was [3]'s bat end, and [4]'s curves inside the last 40 ms are smoothed through the impact (§4.8): the owner has ruled that part is not a target (PLAN D35); the band is Q59. Three of the held-out checks on how the muscles are used fail (§4.9). The breakdowns are not run. Not passed; an independent review comes after the owner's decision.

**Phase 1 after its sixth session (2026-11-25):** with the arms on measured limits too the fitted swing meets the gate's contact time, the pelvis's and bat's rate curves, the three angles at contact and the ground forces. Outside: the bat's speed (20.1 m/s against 29–34; [4]'s own motion gives 22.9, so this criterion and [4]'s curves cannot both be met and needs one reading) and the trunk's rate curve (1.42 band units; nothing in the body brakes the trunk). The breakdowns are not run. Not passed; an independent review comes after the owner's decision (§4.4, §4.7).

**Phase 1 after its fifth session (2026-11-25):** refitted on the measured limits of §4.5. Inside the gate: the pelvis's rate and angle at contact, both feet's forces, the lead knee, the pelvis's travel. Outside it: contact (21–28 ms late), the bat's speed (18.6 m/s), the trunk's rate, the lag and the bat's elevation at contact. The gap is at the arms (§4.6). The gate is not passed; the owner decides whether the arms' limits are to be sourced the same way.

**Phase 1 after its fourth session (2026-11-25, diagnosis only):** the trunk's need is 216–355 N·m about its own axis against 288, so it is not established that the trunk is over its bound; the hips' and lead knee's need at speed stands (§4.4). The swing work stays stopped.

**Phase 1 after its third session (2026-11-25): stopped at the gate.** The legs alone turn the pelvis in time; the inverse dynamics has [4]'s swing needing more than the trunk's bound and more than the force–velocity law leaves the hips at their speeds (§4.4). No bound was changed and no term added. The owner decides between looking for sources, stated assumptions and an elastic part. Note for the gate itself: [4]'s rate curves and its angles at contact cannot both be met (§4.4), so "the rate curves within 1 band unit" and "pelvis 77 ± 6° at contact" need one of them chosen.

**Phase 1 after its second session (2026-11-25):** the engine, its checks, the compiled probe, the placed and moving body and the cost are done (§4.4, §8.1, §8.2). The fit under the rules has run and **is on the gate's "stop" side as the body stands** (§4.4): the bat comes round about 41 ms late at 17 m/s, the pelvis's rate curve is 2.8 band units off, and no single relaxed rule brings contact inside ±5 ms. §4.4 says why that is not yet a finding about bodies: the pelvis is not turned in time by the hip channels this body has, which is upstream of the rules about the arms. Next: §4.4's list.

**Phase 1's order** (Codex's): (1) one placed body, feet down at the foot-down posture, with a two-grip fit of the hands and bounded forces, and its contact outcomes left free; (2) a representative constrained 3D step benchmarked early, with its matching oracle case (cost, §8.2); (3) the stride, the stance and the postural feedback; (4) the moving-ball breakdowns with the collision.

Sizes are in worker sessions of about this one's length; Opus for physics, Sonnet for porting and tests. Revised up after the reviews.

| Phase | What it needs, in order | Size | What could stop it |
| :-- | :-- | :-- | :-- |
| 1. The body in the checking model (Python oracle) | the order above; the drive of §5.1 with sourced bounds; the fit with several starts and every residual and bound reported; the breakdowns as goldens; the checks of §8.1 | 6–9 Opus sessions | the gate above; the closed loop of the two hands; [4] gives means only, so the per-batter spread can be checked, not fitted |
| 2. The engine in Rust | `bb_physics::body` bit for bit against the oracle; a `sim-cli` cage command; recorded poses in the timeline (§9); the client draws one swing | 3–5 sessions (Sonnet port, Opus review; Opus if any reduced body must be derived) | performance (§8.2) |
| 3. The batter in plate appearances, behind a switch | the clock of §5.4 and its prefix test; learned libraries and their cost; the motor errors and sensing noise; the check swing; cage rates and direction checks over the rating range | 4–6 sessions | building libraries robustly for every body; the blend between aims; the clock at game speeds |
| 4. The batter in games | the switch on; the ratings of §7.4 and the roster generator; recalibration through ability distributions only; ledger rows; Opus review; the client drawing the sim's body | 3–5 sessions | the cost budget (an owner decision once measured); run-environment shifts the distributions cannot reach (PLAN §13.2) |
| 5. The pitcher | a pitching dataset first (§11); the delivery's body, the held ball and the release; the command requirement; the same gates as 1–4 | 7–10 sessions | the release model has no lumped-model precedent; one-foot balance through the leg lift; command at the measured finger precision |

## 14. For the owner to decide

**Whether a player's swings may be built by practice when he is created.** *(Answered 2026-11-25: yes, PLAN D32, and players must not be bound to body-type archetypes: a small-framed shortstop with real power has to be possible. Phase 1 checks what a small frame can reach inside the strength bounds.)*

- *Now:* the swing is drawn to meet the planned contact on every pitch, which you have ruled out.
- *What is proposed:* when a player is created, a search over his drive's timings "practises" each swing on his own body until it meets its target as well as his body allows. Some aims fail and stay failed. The finished set is frozen before he plays. On every pitch his body then runs one of those swings (or a blend of neighbours) forward, with its errors, and meets the ball wherever the motion takes it.
- *The question:* is a swing found by practice before play acceptable under "no motion is fitted to its planned end", given that no motion in a game is ever fitted?
- *What the literature says* ([N1]): the published models of this kind (golf, cricket, pitching) find their drive timings by a search, but against a recorded motion. None produces a motion without a search.
- *Options:* (a) yes, practice once at creation, frozen, failures kept; (b) no: a player's swings would then have to be copied from recorded motions, which exist only as averages over many hitters.
- *Recommendation:* (a). It is how a hitter's swing comes to be, and nothing in a game is adjusted to its result.

**The ratings players have.** *(Answered 2026-11-25: two timing ratings, option (b), PLAN D33.)*

- *Now:* `BatSpeed`, `QuickHands`, `BatControl` and `Timing` for batters, `ArmSpeed` and `Command` for pitchers set results directly.
- *What is wrong:* under D31 bat speed, swing time and command are results of the body. They become what the manager is *shown* (as measured), and the ratings become the causes.
- *The causes proposed:* height and weight; lower-body strength and upper-body strength (one strength cannot make a legs hitter and an arms hitter, §7.2; a strength per joint group has no data to draw it from); and how precisely he times his drives.
- *The question:* is timing one rating or two? Two things are involved: syncing his stride and trigger to the pitcher (a hitter who is often late or early), and sequencing his own body (a hitter whose swing is loose: casting, early hips). They enter at different links (§6.1) and give different breakdowns.
- *Options:* (a) one `Coordination` rating for both; (b) two: `Timing` (to the pitch) and `Coordination` (within the body). For the pitcher the same question: his tempo against his release precision.
- *Recommendation:* (b). It keeps one meaning per rating (PLAN §7) and lets a free swinger who barrels the ball when he is on time be different from a disciplined but sloppy one. No source gives the two spreads separately, so both are [E] at first.

**What the hips do when the front foot is late.** *(Answered by your walkthrough; no decision needed unless you disagree.)*

- Your walkthrough says: "Heel-Plant is the trigger. Dropping the front heel locks the front side and initiates hip rotation." So the design waits for the heel (§5.4).
- The corrected probe: with the foot 160 ms late and the hips waiting, the swing is intact (28.9 m/s) but 120 ms late, far behind the pitch. With the hips firing on their own clock instead, the swing is 28 ms late, weak (24.5 m/s) and jammed, because the hips have no weight on the front foot to turn against. (The first version's numbers here, 22 against 25 m/s, came from the probe's error.) Either way a late foot makes the hitter late, as your walkthrough's "Too Late: … late foot-plant" says.

**How much computing a game may take.** *(Not yet a question.)* Every 3D body costs seconds a game against today's 200 ms (§8.2), but the range is wide, 2 to 75 s, because nothing 3D has been measured. The options and a recommendation come after phase 1 benchmarks a real step and phase 3 measures the cost of building the players' swings.

**Withdrawn:** the first version's question on drawing load depth correlated with strength. The probe's coil construction needed twice the real spread (§7.2), so the cause of a stronger hitter's longer swing is open until phase 1.
