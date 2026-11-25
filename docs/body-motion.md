# Body motion: a force-driven batter and pitcher (PLAN D31, §14.1 phase 0)

**Status: design; phase 1 stopped at its gate for the owner's decision (§4.4).** Written 2026-11-25 by worker body-design on branch `work/body-motion`, revised the same day after three independent reviews. Phase 1's first three sessions (worker body-3d, 2026-11-25) built the three-dimensional checking engine, a compiled probe of it, the placed and moving body, the fit under the design's rules and a 3D inverse dynamics. The body does not reach the ball in time under the rules. The legs can turn the pelvis in time; what is short is the trunk's strength against its sourced bound and the hips' and trunk's strength at the speeds they turn at (§4.4). This file is phase 0 of [PLAN §14.1](../PLAN.md#141-the-body-motion-rebuild-d31): it settles the seven items of the plan and the owner's question on consolidating muscle groups and ratings, as far as a flat probe can. Nothing here is in Rust; today's swing and release stay in games until phase 4.

## Status and what is left

- **Done:** the probes of §3 (stdlib Python, committed under `analysis/research/body/`), this spec, the owner's walkthrough as [research note 0](research/body/0-owner-walkthrough.md). Worker body-research's notes 1–7 are merged ([index](research/body/README.md)).
- **Reviewed and revised.** Three independent reviews (2026-11-25: [Codex Sol](reviews/2026-11-25-codex-sol-body-motion-review.md), [Gemini Carbon](reviews/2026-11-25-gemini-carbon-body-motion-review.md), [Gemini Neon](reviews/2026-11-25-gemini-neon-body-motion-review.md)) found errors in the probes and the spec. This version is rewritten from the corrected runs; "What the first version got wrong" below lists what changed.
- **Built in the probes:** a driven pelvis–trunk–arms–bat chain over planted feet, in the horizontal plane (§4.2). Not built: any three-dimensional body (§4.1 designs it; phase 1 builds it), the pitcher's body (§11, design depth only).
- **The main open finding.** Under the design's own rules (the arms start no earlier than the decision allows, drives rise no faster than 50 ms, strengths inside sourced bounds), the flat chain does not reach the ball in time (§4.3). The fit that does reproduce [4] breaks those rules. Phase 1's first job is to show that the three-dimensional body can swing under them; if it cannot, the work stops there (§13).
- **Phase 1, session 3 (2026-11-25):** the lower body alone turns the pelvis in time inside the bounds; the 3D inverse dynamics has [4]'s swing needing 330–440 N·m of trunk rotation against a bound of 288, and 70–95% of the hips' and lead knee's full strength at speeds where the force–velocity law leaves 25–60%. **Stopped for the owner under §13's rule; §4.4 has the question, the options and a recommendation.**
- **Phase 1, sessions 1 and 2 (2026-11-25):** the 3D engine in Python with its checks (§8.1); a compiled probe of it, agreeing to 2 × 10⁻¹² and measured at 50 µs a step (§8.2); the body placed on [4]'s measured state, moving, with both knees resolved; the fit under the rules, the rules relaxed one at a time, the small frame and the wrists (§4.4). **The 3D body as built comes round about 41 ms late at 17 m/s. The pelvis is turned too late, which is upstream of the arms' rules, so this is not yet the answer to whether a body can swing under them.**
- **Left for whoever continues:** the owner's decision in §4.4; dead ends are listed in the [research index](research/body/README.md) under "Tried and ruled out".

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
| the 3D fit, the design's rules | `python3 analysis/research/body/fit3d.py --rules design --starts 3` (7 min; job queue); writes `fit3d_design.json`; `--no-shoulder-stops` writes `fit3d_design_nostop.json` | the 16 channels fitted to [4] under the rules (§4.4) |
| which rule binds, 3D | `python3 analysis/research/body/fit3d.py --limits` (7 min; job queue) | the design fit continued, and one rule relaxed at a time (§4.4) |
| the legs alone, 3D | `python3 analysis/research/body/fit3d.py --lower forces` (also `pelvis`, `rate`; 3–4 min each; job queue) | whether the legs turn the pelvis in time inside the bounds, under three stated weightings (§4.4) |
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

### 4.4 The three-dimensional body: where phase 1 stands after its third session

**In one paragraph.** The engine, its checks, a compiled probe of it (§8.2) and the placed, moving body are built. Session 2's whole-body fit under the design's rules does not reach the ball (41–52 ms late at 17–18 m/s), and its pelvis peaks after contact. Session 3 asked whether that is a missing part or a real limit, two ways. **The legs alone can turn the pelvis in time:** fitted without the arms, inside the bounds, the pelvis is at 497–588°/s at −93 ms ([4]: 558) with the lead foot at 125–136% of body weight ([4]: 130). **But worked back from [4]'s motion, with no search, the swing asks for more than the body is allowed:** the trunk's rotation needs 330–440 N·m against a sourced bound of 288, and the hips and the lead knee must give 70–95% of their full strength while turning at 2.5–7 rad/s, where the design's force–velocity law leaves them 25–60% of it. That is why the fitted drives deliver about half their strengths and the swing is late. Under §13's rule the work on the swing stops here for the owner's decision, below.

**For the owner.** In baseball terms:

- *What the body does now:* a hitter with the strongest legs, trunk and arms the sources allow, starting his arms when the decision allows, gets the bat to the ball about 40 ms late at about 60% of a professional's bat speed.
- *What is short:* not the timing of the arms first, and not the hips' ability to turn. It is how hard the trunk and hips can pull **while they are already turning fast**. The pros' average swing, worked back through this body, needs the trunk to turn the shoulders with 330–440 N·m; the strongest value a source gives (professional golfers, their mean plus two SDs, scaled to a 92 kg hitter) is 288. It needs the rear hip to drive at 256 N·m while turning at 3.3 rad/s; the hip's sourced full strength is 348, but the force–velocity law leaves 175 at that speed. The lead hip and lead knee are in the same position (table below).
- *What could explain it,* without changing anything yet:
  1. **The trunk's bound is too low for professional hitters.** It comes from golfers of 75 kg, tested slowly on a machine. A source would have to show hitters' trunk rotation strength of 330–440 N·m, 1.15–1.5 times the golfers' strongest. None of the research notes has one ([N5] §2: "not published as N·m in anything I could open").
  2. **Strength at speed is under-rated.** The law used matches the one measured on recreational adults up to 4 rad/s (below) and is extrapolation above it. A source would have to show athletes' hips or trunk holding 70% or more of full strength at 3–7 rad/s. [N5] found no measurement at swing speeds.
  3. **The body lacks a part that carries load without muscle drive:** the stretch of the trunk and hips as the hips lead the shoulders (elastic recoil, as §11 already proposes for the pitcher's shoulder). The force–velocity law has an eccentric gain of 1.5 for a muscle being stretched, but no stored energy.
  4. **The reading of [4].** Its pelvis and trunk angle curves turn 12–13° further than its own rate curves integrate to (below). Read from the rate curves, the trunk needs 344 N·m instead of 438, still over 288. The curves are also averages over 33 hitters, and no one of them swings the average.
- *The decision:* (a) look for a source for 1 or 2 before anything is changed, and keep the swing work stopped until then; (b) accept, as a stated assumption with a range, a higher trunk bound or a flatter force–velocity law for professionals, and run the whole-body fit again; (c) design the elastic part of 3 with its own evidence and checks. Nothing in this session changed a bound or added a term.
- *Recommendation:* (a) first, a short research job on trunk rotation strength and on strength at speed in rotational athletes, because it decides between (b) and (c).

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
| trunk rotation toward the pitcher | **438 N·m (−90 ms)** | 329–439 | **288** | 239 (1.0 rad/s) |
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
- **Over its full strength: the trunk only,** by 15–50%. Everything in the legs is inside its bound, with the lead hip's adduction at it. The lead hip's rotators are over their [E] bound only if the unexplained turning moment (below) reaches the ground through that leg.
- **Over what the force–velocity law leaves at the joint's speed:** the trunk (1.4–1.8 ×), the rear hip's extension (1.5 ×), the lead knee (1.2 ×), the rear elbow (1.6 ×), the lead hip's adduction and both hips' rotators (2.6 × and more). The law gives 75% of full strength at 1.25 rad/s and 50% at 3.3; Anderson 2007 measured 75% at 1.6 and 50% at 3.2 rad/s for hip extension in recreational adults ([N5] §1). So the law is the measured one where there is a measurement, and the swing needs more than it.
- **Not built:** the rear knee needs 114 N·m of flexion and the body has no knee flexors.
- **How far to trust it.** [4]'s forces leave up to 1,580 N unexplained on the pelvis (twice-differenced knee and heel angles set its height), which is why the legs are worked up from the forces and not down from the pelvis. The hands' path is [E] ([4] does not report it); between −85 and −30 ms the two elbows and the lag cannot all be met from these shoulders, by up to 7°, and the nearest smoothed path is used. The trunk's number rests on that reconstruction.

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
- **A failed solve** (a pivot that is not safely positive: a singular posture or redundant joints) raises an error. It is never regularised away.
- **Activation** is integrated as a state (15 ms up, 50 ms down), not held through the step.
- **Work** of the drives, stops, damping and contacts is integrated beside the motion as four more states.

**Its checks** (`analysis/oracle/test_body3d.py`, 20 tests, each against something the engine did not produce):

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
- **mutations:** the gyroscopic term removed fails the free body; the centripetal term's sign flipped fails the double pendulum and the identity; a drive's reaction on its parent removed fails angular momentum.

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
- **Licences for shipped numbers** ([N7]): no open, commercially licensed adult dataset with ground forces exists, so shipped values come from published means and SDs, cited. The research notes do not settle reuse of digitised curves or Statcast data; D21's cross-check restriction stays.

## 13. Phases 1 to 5, sized

**PLAN §14.1's rule, copied:** phase 1's gate is that "driven by the drive derived from the measurements, it reproduces the measured swing within the measurements' own error; momentum and energy checks pass; each named breakdown gives its consequence. **If it cannot, the work stops here and the owner is told what blocks it.** No Rust before this." Today's swing and release stay in games, unchanged, through phases 1–3.

**Phase 1's numeric gate** [E: the tolerances are proposals for the owner's review of phase 1, set before it runs]:

- **Pass:** under the design's rules (the arms after the decision, rises of at least 50 ms, strengths inside sourced bounds, the wrists bounded by their measured strength), the average batter's fitted swing reaches contact within ±5 ms of plan; the pelvis, trunk, arm and bat rate curves within 1 band unit RMS each from −250 ms to contact; at contact pelvis 77 ± 6°, lag 65 ± 10°, bat elevation −30 ± 5° ([4] Table 1); the bat's speed within [3]'s and Statcast's range (29–34 m/s at the sweet spot); [4]'s vertical and lateral ground forces within their bands; momentum, angular momentum, energy and the Euler–Lagrange identity to their tolerances; every named breakdown run as one change (or a stated two), its consequence reported as the body gives it.
- **Stop:** if no fit within the rules reaches contact, or the rate curves stay beyond 2 band units, or contact needs a rule broken. The owner is told which rule binds and what the body lacks, as §4.3 does for the flat chain.

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
