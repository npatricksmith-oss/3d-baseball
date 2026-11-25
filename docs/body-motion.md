# Body motion: a force-driven batter and pitcher (PLAN D31, §14.1 phase 0)

**Status: design; phase 1 at its gate: with every designed freedom driven on measured limits (PLAN D34) the fitted swing meets every gated curve and angle on time; the bat's speed is outside the gate on every reading, and how the gate should read it is the owner's decision, changed by §4.8 (§4.4, §4.9).** Written 2026-11-25 by worker body-design on branch `work/body-motion`, revised the same day after three independent reviews. Phase 1's first three sessions (worker body-3d, 2026-11-25) built the three-dimensional checking engine, a compiled probe of it, the placed and moving body, the fit under the design's rules and a 3D inverse dynamics. The body does not reach the ball in time under the rules. The legs can turn the pelvis in time; what is short is the hips' and lead knee's strength at the speeds they turn at. Whether the trunk is also over its sourced bound is not established: session 4 has its need at 216–355 N·m against 288 (§4.4). This file is phase 0 of [PLAN §14.1](../PLAN.md#141-the-body-motion-rebuild-d31): it settles the seven items of the plan and the owner's question on consolidating muscle groups and ratings, as far as a flat probe can. Nothing here is in Rust; today's swing and release stay in games until phase 4.

## Status and what is left

- **Done:** the probes of §3 (stdlib Python, committed under `analysis/research/body/`), this spec, the owner's walkthrough as [research note 0](research/body/0-owner-walkthrough.md). Worker body-research's notes 1–7 are merged ([index](research/body/README.md)).
- **Reviewed and revised.** Three independent reviews (2026-11-25: [Codex Sol](reviews/2026-11-25-codex-sol-body-motion-review.md), [Gemini Carbon](reviews/2026-11-25-gemini-carbon-body-motion-review.md), [Gemini Neon](reviews/2026-11-25-gemini-neon-body-motion-review.md)) found errors in the probes and the spec. This version is rewritten from the corrected runs; "What the first version got wrong" below lists what changed.
- **Built in the probes:** a driven pelvis–trunk–arms–bat chain over planted feet, in the horizontal plane (§4.2). Not built: any three-dimensional body (§4.1 designs it; phase 1 builds it), the pitcher's body (§11, design depth only).
- **The main open finding.** Under the design's own rules (the arms start no earlier than the decision allows, drives rise no faster than 50 ms, strengths inside sourced bounds), the flat chain does not reach the ball in time (§4.3). The fit that does reproduce [4] breaks those rules. Phase 1's first job is to show that the three-dimensional body can swing under them; if it cannot, the work stops there (§13).
- **Phase 1, session 7 (2026-11-25):** (0) the 22.9 m/s of [4]'s own motion is a measuring effect: [3]'s 29–31 m/s is the bat's end (23.9–25.5 at the sweet spot), and [3] and [4] smoothed their marker paths through the impact; undone, [4]'s motion has 27.6–29.9 m/s on this body (§4.8). (1) The 19 undriven freedoms' limits, expectations and four held-out checks were written before any run (§4.5). (2) The engine got a drive about a moving axis and the channels, with their checks (§8.1). (3) **Refitted with 41 channels, contact is +0.8 ms and every gated curve is inside its band (the trunk's 0.57); the arms now move as [4]'s do; the bat is at 21.3 m/s, where the same limits allow 30.6 if asked only for speed. Three held-out checks on the muscles' use fail, and the trunk's second channel is not needed** (§4.9).
- **Phase 1, session 6 (2026-11-25):** the arms' limits were written down with their sources before any refit (§4.5: strength by direction, speed curves, tendon stretch at shoulders and elbows, a starting level, the clock restated as the bat's measured first motion at −170 ms with the arm muscles rising at most 70 ms earlier). **Refitted, contact is +4 ms, the bat first moves at −172 ms, and every gated curve but the trunk's is inside one band unit. The bat is at 20.1 m/s: the gate asks 29–34, [4]'s own motion on this body has 22.9.** Nothing was added; the gate is not passed (§4.7).
- **Phase 1, session 5 (2026-11-25), after the owner's ruling that limits come only from measurements (PLAN D34):** the limits were written down with their sources before any refit (§4.5); the engine got measured strength-at-speed curves, tendon springs at the leg joints and muscles tensed beforehand, with their checks (§8.1). **Refitted, the lower body is inside the gate (pelvis 573°/s at −93 ms, the feet's forces within a band) and the bat is still 21–28 ms late at 18.6 m/s. What is short is the arms, whose limits are estimates and were not touched. Nothing was added to close the gap** (§4.6).
- **Phase 1, session 4 (2026-11-25, diagnosis only):** session 3's 330–440 N·m for the trunk was the lumbar torque about the pelvis's long axis, which takes in part of a large bending moment. About the trunk's own long axis, the way strength is tested, the need is 321 N·m as placed and 216–355 over the things it rests on; worked up from [4]'s ground forces it is 131–172 before the feet's unreported free moments. **So the trunk's excess over 288 is not robust.** The hips' and lead knee's shortfall at speed stands (§4.4).
- **Phase 1, session 3 (2026-11-25):** the lower body alone turns the pelvis in time inside the bounds; the 3D inverse dynamics has [4]'s swing needing 330–440 N·m of trunk rotation against a bound of 288, and 70–95% of the hips' and lead knee's full strength at speeds where the force–velocity law leaves 25–60%. **Stopped for the owner under §13's rule; §4.4 has the question, the options and a recommendation.**
- **Phase 1, sessions 1 and 2 (2026-11-25):** the 3D engine in Python with its checks (§8.1); a compiled probe of it, agreeing to 2 × 10⁻¹² and measured at 50 µs a step (§8.2); the body placed on [4]'s measured state, moving, with both knees resolved; the fit under the rules, the rules relaxed one at a time, the small frame and the wrists (§4.4). **The 3D body as built comes round about 41 ms late at 17 m/s. The pelvis is turned too late, which is upstream of the arms' rules, so this is not yet the answer to whether a body can swing under them.**
- **Left for whoever continues:** the owner's decision in §4.4 (which bat speed the gate reads, and whether the fit's targets inside the last 40 ms are the unsmoothed ones); the hands' speed; the trunk's second channel taken out; then an independent review; dead ends are listed in the [research index](research/body/README.md) under "Tried and ruled out".

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

**How a swing happens.** Each joint group has a drive. It switches on at a planned instant, rises over a set time, is capped by the player's strength, and weakens as the joint turns faster (muscle's force–velocity law). The legs turn the pelvis against the planted feet, the trunk is pulled after the pelvis and then driven itself, the arms follow, and the bat, cocked at the wrists, is pulled round by the hands until it whips through. In the flat probe, a drive fitted to Fortenbaugh's 33 pro hitters [4] reproduces their mean swing within about one between-batter band at most points:

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
  - The measurements of [docs/research/swing/](research/swing/) (D21) as facts the body must reproduce, and where the batter stands and where the ball is met (D28, D29).
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

**In one paragraph.** The engine, its checks, a compiled probe of it (§8.2) and the placed, moving body are built. Session 2's whole-body fit under the design's rules did not reach the ball (41–52 ms late at 17–18 m/s). Sessions 3 and 4 found why: the first limits were too tight for the legs at speed, and the trunk's apparent excess was not robust. The owner ruled that limits come only from measurements (PLAN D34); §4.5 writes them down. With measured limits at the legs and trunk the lower body came inside the gate (§4.6); with them at the arms too the body reached the ball, the trunk's curve and the bat's speed still outside (§4.7). **Session 7 found that the bat's "missing" speed in the measured motion is mostly a measuring effect (§4.8), and gave every designed freedom its drive on measured limits: the fitted swing now meets every gated curve and angle, contact +0.8 ms, the arms moving as batters' do; the bat is at 21.3 m/s, and the same limits allow 30.6 (§4.9).** The history below stands as it was found; its numbers are from the limits of the time.

**For the owner** (rewritten after session 7; the numbers are §4.8's and §4.9's). In baseball terms:

- *Where the "51 mph" came from.* The study the body is fitted to did not have slow hitters. It smoothed its camera data straight through the moment the bat hits the ball, and the ball takes about 27 mph off the barrel at once; smoothing spreads that loss backwards, so the bat seems to slow before contact. Undone as nearly as it can be, those pros swung at **62–67 mph at the sweet spot**. And the older tee study's 65–69 mph, which gave the gate its lower figure, was measured at the very end of the bat, not the sweet spot. Major leaguers in games average 71.5.
- *What was done to the body.* Every part the design gives it now has muscle on it, each from a measured strength: raising and lowering each arm, turning each upper arm, bending the elbows, rolling the forearms, flexing the wrists, and turning the trunk back. The limits and what each was expected to do were written down before anything was run (§4.5).
- *What the body does now:* **it meets the ball on time (0.8 ms late) and matches every curve the gate reads,** the shoulders' included (they peak on time and slow into contact as the pros' do). The hips, the cock of the bat and the barrel's drop at contact are each on the measured value.
- *It also swings more like a hitter where nobody told it to.* Four measured things were kept away from the fit and checked afterwards. Before this session the body met its curves with both elbows flying up over the shoulders; now the back elbow drops into the slot (27° against the pros' 41°), the lead arm sweeps at the measured rate, and the barrel is at its fastest at contact.
- *Where it does not swing like a hitter, said plainly:* it fires six arm muscles before the trunk, where recordings of hitters' muscles show legs, trunk, arms (a rerun shows the body does not need this: held to the recorded order the swing is the same); it barely uses the lead triceps, which is recorded working hard early; and it holds the lead arm up with a muscle that is recorded quiet, pulling against its opposite. The new trunk muscle is also used only to pull against the first; the swing is as good without it.
- *The bat's speed:* **21.3 m/s, 48 mph.** The gate asks for 65–76 mph; the smoothed curves the body is matched to hold 51; the same curves with the smoothing undone, 62–67. Nothing holds it down: twice the measured arm strength adds 1 mph. It is slow because it copies curves that are slow near contact, and because its hands move at half the speed the same study's hands do (7 mph at contact against 15), which the fit is not asked to match since the study does not report the hands.
- *What this body could do if asked only for bat speed:* **68 mph, on time,** with every muscle at the top of its measured range at once. So the gate's 65–76 mph is at the edge of what it can do, and it gets there in a way no hitter swings (the front leg soft, the lead arm jammed across the chest).
- *The small-framed hitter* (1.75 m, 77 kg) meets the ball on time at 92% of the average's bat speed.
- *The decision (Q59, restated because the first finding changes it):*
  - (a) the gate reads 29–34 m/s (65–76 mph) at the sweet spot at contact, as written, and the curves the body is matched to inside the last 40 ms are replaced by the study's curves with the smoothing undone, the way that was done being stated as an assumption;
  - (b) the gate's band becomes 27–34 m/s (60–76 mph), the study's own hitters at the bottom and the major-league figure at the top, with the same change to the curves;
  - (c) nothing changes; the bat's speed is listed as open, knowing the curves near contact are smoothed.
  - The earlier option of taking 22.9 m/s (51 mph) as the gate is withdrawn: it would write the smoothing into the gate.
- *Recommendation:* (b), then one more fit against the unsmoothed curves with the hands' speed reported beside it, then the independent review. Whichever is chosen, the trunk's second muscle should come out.
- *Waiting on you:* Q59 only.

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

*The axes.* A shoulder is driven the way its motion is described clinically: the existing channel turns the arm about the trunk's long axis; **raising and lowering** act about the axis square to the trunk's long axis and the upper arm (it moves with the arm; the angle it changes is the arm's angle from hanging at the side, which is what [4] reports as the shoulder's elevation); **rotation** acts about the upper arm's own long axis. The elbow's flexors act about the hinge the extensors use; forearm rotation about the forearm's long axis; wrist flexion and extension about the wrist's second axis, which in a hand closed round a handle lies along the handle (the motion of a throttle grip). Both hands hold the bat thumb toward the barrel, which fixes which way is flexion.

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

- **The hands are not what is short.** They reach 11.0 m/s at −50 ms and are at 6.5 m/s at contact (7.0 measured from the mid-point between the hands, §4.7's figure). Blast's database has major-league peak hand speed at 23–29 mph, 10.3–13.0 m/s (a knob sensor's own measure; EXC, blastmotion.com and Prep Baseball Report boards). At contact they move 60° off the sweet spot's direction, so 6.5 m/s of hand gives it 3.2.
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
- *Still open (Q59):* the band the gate reads, §4.4.

### 4.9 The refit with every designed freedom driven (session 7)

`fit3d.py` on §4.5's limits with its last block, written and committed before the engine had the channels: 41 channels, 205 numbers; three cold starts staged as before (25, 25 and 40 iterations) and one more from session 6's fitted plan with the new channels switched off; one run of the whole body for the reported numbers (the same at a 0.5 ms step to 0.1 ms and 0.1 m/s). 46,400 swings, 38 minutes. The compiled probe agrees with the Python engine with all 41 channels on and on the fitted swing itself (9 × 10⁻¹² at worst), and with the new channels off the new code reproduces session 6's fit exactly (+4.0 ms, 20.1 m/s). This is a laboratory fit of the model's drive numbers to a recorded average swing, as before.

| | Start 0 | Start 1 (best) | Start 2 | From session 6's fit | The gate |
| :-- | --: | --: | --: | --: | :-- |
| contact (bat at −8°) | +4.6 ms | **+0.8** | +3.6 | +1.9 | 0 ± 5 ms |
| the bat's first motion | −170 ms | **−170** | −171 | −170 | −170 ± 15 ms |
| the first arm drive's rise before that | 59 ms | **70** | 62 | 65 | at most 70 ms |
| sweet spot at contact (its fastest, when) | 21.0 m/s (21.5, −35 ms) | **21.3 (21.3, at contact)** | 18.7 (18.9, −5 ms) | 21.9 (21.9, +5 ms) | 29–34 m/s; [4]'s own motion on this body 22.9 (25.0); the same with the impact's smoothing undone 27.6–29.9 (§4.8) |
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
- **Outside: the bat's speed, on every reading.** 21.3 m/s at contact against the gate's 29–34; against [4]'s own motion on this body, 22.9 at contact and 25.0 at its fastest (93% and 85%); against that motion with the impact's smoothing undone, 27.6–29.9 (§4.8).
- **Not evaluated:** the breakdowns of §6.2.
- **How settled the searches were.** The best start was still falling by 0.004 an iteration (cost 3.40) and the start from session 6's fit by 0.005 (4.09); start 0 by 0.03 (12.4); start 2 had stopped (15.5, the lead foot's forces 1.4–2.2 band units off). So two of the three cold starts did not reach the best start's neighbourhood in 40 iterations of the last stage: with 205 numbers the search finds different swings from different starts. The best and the warm start, which began in different places, agree on contact (0.8 and 1.9 ms), the bat's speed (21.3 and 21.9 m/s), the trunk's peak (760 and 769°/s at −65 ms) and the three angles at contact.

**The count of fitted numbers** went from 110 to 205 against the same 15 curves and 5 numbers at contact. That every curve is now inside its band is therefore weak evidence by itself; the held-out checks below are what tests the body.

**On a bound (best start): 56 of 205 numbers.** At full strength: the lead hip's extensors, the rear hip's abductors and rotators, the rear knee's extensors, the lead ankle, the trunk's forward rotators (260 N·m), the lead shoulder's sweep (92), the rear elbow's extensors, both shoulders' internal rotation (70), the lead elbow's flexors (112) and the rear wrist's flexion (46). Both wrists' uncocking channels are switched off, as is the rear forearm's pronation. Nineteen rises are at the 50 ms floor. The earliest arm drive (the rear elbow's flexors, 10 of 112 N·m) is at −240 ms, on the limit.

**The trunk.** It now peaks at 760°/s at −65 ms and is at 517°/s at contact ([4]: 766 at −57, 429 at contact), 75° round at contact ([4]: 62° on its rate curve, 75° on its angle curve). **How it slows is not how the job expected.** The fit runs both trunk channels together from before the window opens (both tensed at 0.3): the forward rotators at their full 260 N·m (250–285 applied) and the rotators the other way rising from 52 to 164 N·m by contact. The two muscles' net is still 100–145 N·m *forward* through the whole swing, 122 at contact. So the brake does not reverse the trunk's drive; it halves it, and the trunk slows because the arms and the bat take its motion: the rear shoulder's end-of-range stop carries up to 228 N·m at −60 ms and the new shoulder channels push the arms off the trunk. Whether the brake is needed at all is the "without" run below.

**The bat's speed, split as §4.8 splits it.** At contact 21.3 m/s = the hands 0.5 + the bat's turn in azimuth 19.7 + in elevation 1.1; the bat's azimuth rate is 2,151°/s ([4]'s curve: 2,217; the placed motion: 2,129), the bat's end at 26.2 m/s ([3]'s end marker: 29). **The body turns the bat as [4]'s curve does and is slow in the hands: 3.3 m/s at contact against the placed motion's 6.5, and 7.6 m/s at their fastest against 11.0** (major leaguers' peak hand speed: 10.3–13.0, §4.8). Session 6 had the same (3.6 m/s). The new drives did not give the hands their speed; step 1 expected 1–2 m/s from the shoulders' rotation and got about 1 in total (20.1 to 21.3).

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
| 4. the bat at contact against its fastest before | 98%, the peak 2 ± 5 ms before | 100%, the peak at contact | 100% | 92%, the peak at −10 ms |

- **Passed:** the arms now move as batters' do where session 6's did not. Session 6's body met its curves with both elbows flying up past shoulder height (the rear arm 121° from hanging at contact where [4]'s is 37°); with the arm's raising and lowering driven the rear elbow drops 27° through the swing ([4]: 41°) and both shoulders' elevations are nearer [4]'s than [4]'s own motion placed on this body is (the placement's elbows are [E], which is why it is 1.6–1.7 units off). The lead arm's turning rate came from 847 to 1,064°/s, inside one SD of [3]'s. The lead shoulder's sweep is on before the swing and through it. The bat is at its fastest at contact (expected to fail; it does not).
- **Failed, and said so:**
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
- **The trunk's brake is not what slows the trunk.** Without that channel the trunk's curve is still at 0.60, the fit is a little better and the bat a little faster. With it, the fit only sets two trunk muscles against each other. What slows the trunk is the arms being driven off it; no single arm drive carries that either. **The body without the trunk's second channel is the cleaner one, and nothing measured asks for it in a swing that is not checked.**
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

- **29–34 m/s is at the edge of what this body can do.** With every muscle group at its sample's mean + 2 SD at once and every timing chosen for bat speed alone, the body reaches 30.6 m/s (68 mph) on time. The search was still gaining 0.3 m/s in its last 10 iterations, so the ceiling is somewhat above that; it is not near 34.
- **It gets there with a swing no hitter makes:** the front leg does not block (the knee 61° bent at contact, the foot never over 70% of body weight), the hips turn at 813°/s, and the lead arm is pressed 16° past the end of its range across the chest, where the model's stop is a soft spring [E: 2,000 N·m/rad] carrying 574 N·m, more than any shoulder muscle's bound. A stiffer stop would take some of that speed away; how much was not run.
- **So the average batter's 21.3 m/s is not a strength limit:** the same limits allow 30.6. It is what the fitted curves hold (§4.8).

**The strongest batter held to a hitter's swing** (session 7 fix, review P2-4; this paragraph was written and committed before the run). `fit3d.py --strongest --held-swing`: the same search as the run above (every bound available, every timing free, asked only for the sweet spot's speed at contact, the timing held hard: contact at 0, the bat's first motion at −170 ms, no arm drive rising more than 70 ms before it), started from the average batter's fitted plan, 80 iterations, all 205 numbers free. Added, each as a one-sided penalty that is zero inside its limit and weighs 30 per unit past it [E] (`HELD_W`):

- the gate's ground-force curves (both feet, three directions) and its lower-body curves (the pelvis's rate and travel, the lead knee) each within 1 band unit RMS of [4]'s, from −250 ms to contact;
- no joint pressed past its range more than in the average batter's fitted swing: each end-of-range stop's largest torque up to contact at most the average fit's plus 5 N·m (the lead shoulder's 99, the rear's 228; the knees' and elbows' 0), per 10 N·m past it;
- the attack angle at contact inside OpenBiomechanics' 10th–90th percentiles, −3.3° to +10.7° ([note 6](research/swing/6-openbiomechanics.md)), per 2° past it. The average fit itself is outside this (−7.8°), so the run starts with that penalty on.

The trunk's and arms' curves, the bat's own curves and the held-out checks are not asked for. What is reported: the bat's speed at contact, every penalty's state at the end (a limit the search did not keep is said as broken, not hidden), what the speed sits on (strengths on their bounds, the pelvis's and trunk's peaks, the lead foot's load and knee), and how fast the speed was still rising over the last 10 iterations.

**What is left:**

- the owner's reading of the gate's bat speed (Q59, with §4.8's finding that [4]'s curves near contact are smoothed through the impact). If the fit's targets inside the last 40 ms change, this fit is redone against them;
- the hands: 3.3 m/s at contact against 6.5 in [4]'s own motion. Not a drive that is missing (every designed freedom is now driven) and not a strength bound. The hands' path is not in the misfit, because [4] does not report it; a target for it would have to come from the placed motion, whose hand path is [E];
- the trunk's second channel: the fit is cleaner without it;
- cold starts: two of three did not settle in 40 iterations with 205 numbers; a review should ask for more starts or a search that is not started three times from near one hand estimate;
- the breakdowns of §6.2 and an independent review, as before.

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

**Still blind:** the segment table itself beyond its mass sum; the ground model's realism; a drive axis wrong alike in the model and in what the fit compares; event-time location (not built). **What the 3D body still needs specified:** how the hands' loop and the feet's contacts are solved (a deterministic constrained solve, its iteration count and tie order fixed), how constraint drift is held (projection or stabilisation, and whether it is applied per step), how a contact's making or breaking and an impact are handled, and what happens when a solve fails. Events (toe touch, heel contact, the bat passing a point, release) are located within the step by cubic Hermite interpolation of the state, or by sub-stepping at 0.1 ms around release and contact [E], with a convergence test against finer location (§11 says why release needs it).

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
- **The 3D body's fit to [4] and [3]**, including the bat's elevation (+44° to −30° from −340 ms to contact), the pelvis and trunk at contact (77° and 75°), the lag at contact (65°) and the bat's speed at its true elevation.
- **What holds the wrists' cock** in 3D, and whether a passive limit exists (§4.3).
- **The arms' earliest onset** relative to [4]'s first move, and the clock at game speeds (§5.4).
- **Strength by joint group for professionals in N·m**, its spread, and any value above about 5 rad/s ([N2] §5, [N5] §6).
- **Movement variability**: no σ of onsets or strengths is published ([N6]); the sizes are [E] and set by outcome spreads with perception and location active (§6.1).
- **The remaining angular impulse** (§5.2): 23 of 36 N·m·s that [4]'s lateral couple does not give.
- **What makes a stronger hitter's swing longer** (§7.2): not the coil alone.
- **The trunk's force–length relation** and its axial range.
- **Whether a 3D swing needs feedback** (§5.3), and balance on one foot in 3D.
- **The cost of a constrained 3D step** and of building the learned libraries (§8.2).
- **How much the impact's smoothing took off [4]'s bat curves** (§4.8): 4.7–7.0 m/s on one stated form; unfiltered marker paths, or any professional set filtered separately before and after contact, would settle it.
- **The hands' path and speed in [4]'s swing**: not reported; the placed motion's is [E], and the fitted body's hands are half as fast (§4.9).
- **Licences for shipped numbers** ([N7]): no open, commercially licensed adult dataset with ground forces exists, so shipped values come from published means and SDs, cited. The research notes do not settle reuse of digitised curves or Statcast data; D21's cross-check restriction stays.

## 13. Phases 1 to 5, sized

**PLAN §14.1's rule, copied:** phase 1's gate is that "driven by the drive derived from the measurements, it reproduces the measured swing within the measurements' own error; momentum and energy checks pass; each named breakdown gives its consequence. **If it cannot, the work stops here and the owner is told what blocks it.** No Rust before this." Today's swing and release stay in games, unchanged, through phases 1–3.

**Phase 1's numeric gate** [E: the tolerances are proposals for the owner's review of phase 1, set before it runs]:

- **Pass:** under the design's rules (the arms after the decision, rises of at least 50 ms, strengths inside sourced bounds, the wrists bounded by their measured strength), the average batter's fitted swing reaches contact within ±5 ms of plan; the pelvis, trunk, arm and bat rate curves within 1 band unit RMS each from −250 ms to contact; at contact pelvis 77 ± 6°, lag 65 ± 10°, bat elevation −30 ± 5° ([4] Table 1); the bat's speed within [3]'s and Statcast's range (29–34 m/s at the sweet spot); [4]'s vertical and lateral ground forces within their bands; momentum, angular momentum, energy and the Euler–Lagrange identity to their tolerances; every named breakdown run as one change (or a stated two), its consequence reported as the body gives it.
- **Stop:** if no fit within the rules reaches contact, or the rate curves stay beyond 2 band units, or contact needs a rule broken. The owner is told which rule binds and what the body lacks, as §4.3 does for the flat chain.

**Phase 1 after its seventh session (2026-11-25):** with every freedom §4.1 designs driven on measured limits, the fitted swing meets the gate's contact time (+0.8 ms), every rate curve (pelvis 0.49, trunk 0.57, bat 0.62 band units), the three angles at contact and the ground forces, under the rules. Outside: the bat's speed, 21.3 m/s, on every reading (29–34; [4]'s own motion 22.9; that motion with the impact's smoothing undone 27.6–29.9). The gate's lower figure was [3]'s bat end, and [4]'s curves inside the last 40 ms are smoothed through the impact (§4.8): the criterion and the fit's targets near contact need the owner's reading (Q59). Three of the held-out checks on how the muscles are used fail (§4.9). The breakdowns are not run. Not passed; an independent review comes after the owner's decision.

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
