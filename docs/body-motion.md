# Body motion: a force-driven batter and pitcher (PLAN D31, §14.1 phase 0)

**Status: design, not built.** Written 2026-11-25 by worker body-design on branch `work/body-motion`. This file is phase 0 of [PLAN §14.1](../PLAN.md#141-the-body-motion-rebuild-d31): it settles the seven items of the plan and the owner's question on consolidating muscle groups and ratings. Nothing here is in Rust; today's swing and release stay in games until phase 4.

## Status and what is left

- **Done:** the probes of §3 (stdlib Python, committed under `analysis/research/body/`), this spec, the owner's walkthrough as [research note 0](research/body/0-owner-walkthrough.md). Worker body-research's notes 1–7 are merged ([index](research/body/README.md)). Their three main findings are taken in: §5.1 and §11 (the last links are driven by inertia and elastic storage, not their own muscles), §6.1 (error sizes back-solved from published outcome spreads) and §14 (no published model makes a motion without fitting to a measured one).
- **Built in the probes:** the smallest model that answers the questions honestly, a driven pelvis–trunk–arms–bat chain over planted feet, in the horizontal plane (§4.2). Not built: any three-dimensional body (§4.1 designs it; phase 1 builds it), the pitcher's body (§11, design depth only).
- **Left for whoever continues:** the independent review; the owner's decisions of §14; then phase 1 (§13).

## 1. Summary in baseball terms

**The body.** The batter is a set of body segments (feet, shanks, thighs, pelvis, trunk, head, upper arms, forearms, hands) with masses and inertias from de Leva's tables scaled by his height and weight, joined at his joints. His feet hold the ground by friction, the two hands hold the bat, and muscle drive at the joints turns each segment against the one below it. The bat's path is the result. Nothing in the motion is fitted to where it is meant to end.

**How a swing happens.** Each joint group has a drive. It switches on at a planned instant, rises over a set time, is capped by the player's strength, and weakens as the joint turns faster (muscle's force–velocity law). The drives are timed proximal to distal: the legs turn the pelvis against the planted feet, the trunk is pulled after the pelvis and then driven itself, the arms follow, and the bat, cocked at the wrists, is pulled round by the hands until it whips through. In the probe, the average batter's drive was worked back from Fortenbaugh's 33 pro hitters [4] and reproduces them within about one between-batter SD:

- peak pelvis rate 575°/s at −66 ms (measured: 581 at −70);
- peak trunk rate 706°/s at −56 ms (766 at −57);
- peak arm rate 1,146°/s at −56 ms (1,160 at −65, [3]);
- bat speed at contact 30.6 m/s ([3]: 29–31).

The wrists' own drive switched itself off in the fit: the bat is whipped through by the hinge, as golf models also find. Peak hip–shoulder separation is 16° (18°). Where it falls short is §4.3.

**The breakdowns, from the body** (§6.2; the ball is fixed where the average swing meets it; the sweet spot is 0.634 m from the hands):

| What went wrong | What the body did | Where the ball meets the bat |
| :-- | :-- | :-- |
| Lead foot down 160 ms late, the hips' drive on its own clock | the hips fire with no front foot to turn against: the pelvis is only 4° open at the meeting (64° normally) | 28 ms late, 7 cm toward the handle, 25.2 m/s against 30.7: jammed, weak |
| The same, the hips waiting for the heel plant (the walkthrough's trigger) | the whole swing slides 77 ms | 85 ms late, 12 cm toward the handle, 22 m/s |
| Hips 40 ms early | the pelvis leads by 46° (normally 16°) | 7 ms early, 1 cm toward the end, the bat 2.4° further round: pulled |
| Casting: the hands throw the barrel from −150 ms | the bat swings out wide early | 3 ms early, 10 cm toward the handle, 27.7 m/s, the bat 10.5° further round: rolled over (at the wrists' measured strength, a smaller version: −1.3 m/s, 4.5 cm, 4.5°) |
| Pulling off: the trunk's drive 40 ms early | the trunk opens with no separation left | 15 ms early, 4 cm toward the end, pulled 4.5° |
| Lunging: fooled, the rotation held 100 ms while the stride drifts on | the weight goes forward (0.53 m against 0.44); the pelvis never turns (−1°) | 115 ms later than the fastball, 20 cm toward the handle, 17.6 m/s: arms only |

**The ratings.**

- Recommended: the model integrates every drive channel, fed by a few ratings (§7.4): body size (`Height`, `Mass`); lower-body strength and upper-body strength; a coordination rating that sets how precisely the drive's onsets are timed; and the load depth (how far he coils) as a trait of his swing.
- One "strength" rating alone makes every hitter's swing the same shape. Two strength groups give believably different hitters at the same bat speed (§7.3): a legs hitter turns his pelvis at 711°/s, an arms hitter at 438°/s, and a late foot costs the arms hitter twice the bat speed.
- MLB's relation between swing length and bat speed comes out only if a stronger hitter also coils more. The coil this needs is about twice the spread between real hitters (§7.2).

**Holding the body on its path** (§5.3):

- A standing body played forward on recorded forces falls: a 0.5° lean error grows to 41 cm in 1.5 s.
- The muscles' own stiffness plus a reflex with 100 ms of delay holds it within 1 cm over the whole size range.
- The swing itself (0.25 s, feet planted) needs no feedback. Muscle's force–velocity law, the joints' range limits and the planted feet keep it on its path, and a 15 ms late start stays a 15 ms late meeting.

**Cost** (§8.2). The probe's chain costs 1.7 µs per step in Rust, 0.46 ms per swing. A full 3D body integrated on every pitch would cost about 6 s per game against PLAN §13.4's 200 ms. A 3D core of about 12 degrees of freedom, integrated per pitch only over the phases an error can change, fits about 0.25–0.6 s. Either way the budget is the owner's (§14).

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

Every number below is tagged **measured** (a source gives it), **derived** (computed from measured numbers by a stated method, the probe named), or **[E]** (assumed, with what would settle it).

**The probes.** Stdlib Python 3, run from the repository root. Each needs the digitised curves of [4] Appendix E, made by `python3 analysis/research/fortenbaugh_appendix_e.py` (which needs the dissertation PDF in `sim/target/research-dl/4/`, Pillow and poppler's `pdfimages`). The digitised curves were checked against [4]'s text at 16 points, every one within its reading error. For example: pelvis 77° at contact (77), peak pelvis rate 580°/s at −73 ms (581 at −70), lead GRFz 130% BW at −96 ms (130 at −93), separation −17.1° at −104 ms (−18 at −103). Outputs are kept in `sim/target/codex-probes/body/*.out`.

| Probe | Command | What it shows |
| :-- | :-- | :-- |
| inverse dynamics | `python3 analysis/research/body/inverse.py` | the yaw torques [4]'s motion needs at each joint, and how [4]'s ground forces compare (§5.2) |
| the forward chain | `analysis/research/body/chain.py` (module) | the driven body (§4.2) |
| the fit | `python3 analysis/research/body/fit.py 400` (about 2 min; through the job queue) | the average batter's drive from [4] (§4.3) |
| the fitted swing | `python3 analysis/research/body/swing.py` | the swing against [4] and [3], with the energy check (§4.3) |
| integrator checks | `python3 analysis/research/body/checks.py` | energy, momentum, step size (§8.1) |
| balance | `python3 analysis/research/body/balance.py` | what keeps a standing body up (§5.3) |
| breakdowns | `python3 analysis/research/body/breakdowns.py` | the owner's named failures (§6.2) and the decision clock (§5.4) |
| ratings | `python3 analysis/research/body/ratings.py` (20 s) | the consolidation comparison (§7) |
| Rust cost | `rustc -C opt-level=3 -C linker=$HOME/.local/bin/zigcc -o sim/target/codex-probes/body/bench_chain analysis/research/body/bench_chain.rs`, then run it | time per step and per swing in Rust (§8.2) |

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

- **Segment numbers.** [dL] gives each segment's mass as a share of body mass, its centre of mass as a share of its length, and three radii of gyration as shares of its length. Lengths scale with stature, which is de Leva's own assumption. The table is in `analysis/research/body/common.py`. This is **measured** for de Leva's 100 young male athletes (mean 73.0 kg, 1.741 m); applying it to a 92 kg professional is **[E]**. No table measured on baseball players exists ([N4] §5), and [N4] expects errors of 10% or more on some segment for an athlete. The probe's sensitivity (`checks.py` item 6) is modest. With ±10% on the trunk's inertia, the pelvis's or the arms' mass, the fitted drive unchanged:

- contact moves −2.3 to +4.1 ms;
- bat speed moves 30.59–30.70 m/s against 30.65.

A heavier player's distribution of mass is a phase 1 choice to make with that sensitivity in view.
- **The feet.** A loaded foot is held to the ground by a contact constraint:
  - no sinking;
  - friction within a cone, μ [E] (shoe on clay, no source yet; the probe used 0.8);
  - the centre of pressure inside the sole, so the ankle torque is bounded by the load times the sole's reach ahead of and behind the ankle.

  The rear foot stays down through the swing and pivots onto its ball late. That is [5]'s picture and OBP-only data ([swing-synthesis §3](research/swing/swing-synthesis.md)), so it is [E] for its timing. The lead foot is a free limb during the stride. It touches toe first and then heel ([4] "stepping on thin ice"; the owner's walkthrough), and its load builds over about 170 ms ([4]: 10% BW at foot down, −340 ms, to 50% at the weight commitment, −170 ms; **measured**).
- **The block.** The front leg's knee and hip extensors, driven in their own channel. After foot down they brake the body's forward motion and extend the knee: [4]'s lead knee goes from 48° at the commitment to 18° at contact, **measured**. The lateral forces on the two feet form the couple that turns the pelvis ([4]'s lead and trail GRFy, 28% and 24% BW at −123 and −127 ms; Ae 2017 in [N2]: the lateral forces of both legs make the large whole-body moment about the vertical). A front knee that bends instead of extending is the "collapse" of §6. A body whose centre of mass passes over the front foot unloads the rear foot, and the couple goes with it (lunging).
- **The hands and the bat.** Each hand is welded to the handle at its grip point [E: grip points 0.08 m and 0.18 m from the knob]. The loop through both arms is closed by constraint forces. The grip holds as long as the force along the bat stays under a limit [E; Koike 2003 measured about 750 N along the bat at impact for the bottom hand ([N2] §2.1), so the limit must be above that]. A slip is not modelled in phase 1.
- **The fingers and the ball** (the pitcher, §11). The ball rides in the fingers. It is released when the fingers' drive opens them, and it rolls off the finger pads, whose last contact sets the spin axis.
- **The head and eyes** ride on the trunk. Where the eyes point is the perception's business (agent-models §3.2): a body that pulls off moves the eye point, and that is how pulling off reaches perception.

### 4.2 The probe's reduced model, and what it leaves out

`chain.py`: five degrees of freedom in the horizontal plane.

- **s**: the body's forward position along the line to the pitcher, over feet fixed 0.42·H apart. [4]'s toe-to-toe stride at foot down is 42% of height (**measured**).
- **φ1**: the pelvis's yaw, a rotor about the vertical through the body. Its inertia is de Leva's LPT plus half the thighs' [E].
- **φ2**: the trunk's yaw (MPT + UPT + head, [dL]). It carries the lead shoulder at r_sh = 0.112·H on its lead side (swing-model v3 §5.2) [E].
- **φ3**: the arms, one link from the lead shoulder to the hands, 0.30·H long [E]. It carries both arms' masses at their [dL] centres.
- **φ4**: the bat (physics-spec §6.2's default 34-in bat), hinged at the hands 0.08 m from the knob [E]. Its cock is lag = φ3 − φ4. A passive stop holds the cock at most at 133°, [4]'s largest lag (**measured** as a value; as a stop, [E]).

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

- the vertical load is split between the feet by the lever rule on s;
- the lead foot's share builds over 170 ms after it touches;
- the legs' yaw torque is capped by friction: μ times the smaller foot load times the stride (a lateral couple), plus each foot's free moment, μ·0.05 m·load [E];
- the push and the brake are capped by friction on their own foot.

The trunk-on-pelvis rotation has a passive range of ±45° [E: no source read].

**What the reduction leaves out** (phase 1's job):

- Everything vertical: the bat's elevation and attack angle, the trunk's tilt and side bend, the knees' bend and the pelvis's rise. The swing plane is taken as horizontal, so the planar chain cannot express "barrel under" or "over".
- The real vertical dynamics of weight transfer, for which the lever rule with a ramp stands in.
- The elbows: the arm link has a fixed length, so the hands' path cannot widen or narrow except by the wrist. Casting enters at the wrist only.
- Two arms in a loop: one arm link carries both.
- The muscles' force–length relation.

The model is enough to answer the design's questions about the drive, control, errors, ratings, replay and cost, because those turn on the yaw chain that carries most of the bat's energy. It is not enough to reproduce a batted ball, which needs the bat's elevation.

### 4.3 How well the reduced model reproduces the measured swing

The average batter's drive was fitted to [4] (§5.2): 17 drive numbers, by Nelder–Mead, against [4]'s pelvis, trunk and bat-azimuth rate curves from −250 ms to contact. Each misfit is scaled by [4]'s between-batter band as read off the graphs (pelvis 80, trunk 100, bat 250°/s), plus contact at 0 ms, the lag at contact (65 ± 10°), the separation's peak (17.1 ± 4°) and the body's forward travel. The final cost is 0.52 per point in those units, that is, within about one between-batter SD on average. Output `sim/target/codex-probes/body/swing.out`.

| Quantity | Model | Measured |
| :-- | :-- | :-- |
| pelvis peak rate | 575°/s at −66 ms | 581 at −70 [4]; 714 (SD 76) at −75 [3] |
| trunk peak rate | 706°/s at −56 ms | 766 at −57 [4]; 937 (SD 102) at −65 [3] |
| arm peak rate | 1,146°/s at −56 ms | 1,160 (SD 96) at −65 [3] |
| bat azimuth peak rate | 2,496°/s at 0 ms | 2,435 at −10 [4] |
| hip–shoulder separation peak | 16.3° at −120 ms | 18 at −103 [4] |
| bat lag, largest / at contact | 134° / 50° | 133 / 65 (SD 10) [4] |
| pelvis / trunk at contact | 64° / 63° | 77 (SD 6) / 75 [4] |
| sweet-spot speed at contact | 30.6 m/s | 29 at contact, 31 peak (barrel end, tee) [3]; [4] none; MLB average 32.1 (Statcast, [S10]) |
| body's forward travel at contact | 0.441 m from the rear foot | 0.436 m ([4]'s pelvis; the feet's position [E]) |

**Where it falls short, and why:**

- The pelvis and trunk end 13° and 12° short of [4]'s turn at contact (2 SD). The fitted pelvis decelerates late, and the model has no knee extension to carry the lead hip back ([3]: "the left knee extends and pushes the left hip back").
- The bat peaks at contact, not 10 ms before it, and its lag closes to 50°, not 65°. The arm link has no elbows, so nothing slows the hands before contact as [4]'s lead arm does (its peak is at −43 ms).

Both are expected to move in the three-dimensional body and are phase 1's gate. The fit is of [4]'s *mean* curves. They are averages over batters aligned on events, filtered at 12 Hz ([4] 3.3), so a single batter's peaks are higher and sharper. The fit matches the average shape, not any one swing.

**The drive it found** (applied, not capped):

- legs: up to 204 N·m;
- core: 219 N·m;
- shoulder: 157 N·m;
- the wrist's drive off throughout (the passive stop carries up to 257 N·m while the bat is cocked);
- the lead leg's brake: up to 412 N;
- the rear leg's push: 77 N.

Cross-checks:

- The golf generators of MacKenzie and Sprigings 2009 ([N1] §1.1): torso 200 N·m, shoulder 160 N·m.
- Koike 2003's bottom-arm shoulder torque at impact, about 190 N·m ([N2] §2.1).
- The friction cap on the legs' yaw torque ran at 230–260 N·m. [4]'s measured lateral couple peaks at about 220 N·m (§5.2).

- Trunk rotation strength measured isokinetically in 51 professional golfers (74.7 kg): 141 N·m at 0.5 rad/s, falling to 117 at 2.1 rad/s ([N5] §3). Scaled by mass to 92 kg, about 173 N·m at 0.5 rad/s. The model's core reaches 219 N·m, about 1.3 × that. Its Hill curve also falls faster at low speed than the measurement does (31% against 17% by 2.1 rad/s).
- No measurement exists above about 5 rad/s ([N5] §6), so the drive's fall at swing speeds (10–40 rad/s) is extrapolation for every source.

The wrist's switching off agrees with [N1]: Koike: "torques in the last part of the swing do not raise bat speed; they hold it"; Hirashima: the distal joints of a throw are accelerated by velocity-dependent effects, not their own torques. It also agrees with what the wrists can do: about 10 N·m each in radial–ulnar deviation ([N5] §5).

**The wrist's cock stop is a device, and the swing does not depend on it.** The stop briefly carries up to 257 N·m near −120 ms, more than hands can hold. With its torque capped at what two wrists hold, 20 N·m (`checks.py` item 5):

- the bat speed is 30.53 m/s against 30.65;
- contact comes 3.3 ms later;
- the largest lag is 138° against 134°;
- the lag at contact is 63° against 50°, nearer [4]'s 65°.

So the stop is not what makes the swing. In 3D, the cock is held by the two hands' couple and by the bat's own path; phase 1 should use the sourced wrist strength, not a stiff stop.

## 5. The drive and its control (§14.1 items 2 and 3)

### 5.1 What a drive is

A drive channel is one muscle group's net torque at a joint, or force along a leg:

**τ(t) = T · a(t) · f_v(ω/W)** [form: the golf and bowling generators of [N1] §1.1–§1.2]

- **The excitation u(t)** is the motor command: 0 before the onset t_on, a linear ramp over the rise time t_rise, 1 until the switch-off t_off, then 0.
- **The activation a(t)** follows u through first-order dynamics: 15 ms rising, 50 ms falling. These are [E], OpenSim's Thelen 2003 defaults; [N1] lists 20 / 40 ms in golf, a minimum 70 ms ramp in bowling, and 5 ms in Attias's pitching model. Physiology puts the rise of a maximal voluntary drive at about 50–75 ms (Maffiuletti et al. 2016, [N5] §4; no time to 90% of peak torque was found). The fitted rises (legs 138 ms, trunk 192 ms) are slower than that. The shoulder's, 24 ms plus activation, is faster, and phase 1 should bound it from below by the review's 50 ms.
- **The force–velocity factor f_v** is Hill's: f_v = (1 − w)/(1 + w/0.25) while shortening at w = ω/W of the channel's top speed W. While lengthening it rises to 1.5 × [E]. Golf's generators use the same form with C = 3, i.e. 0.33 ([N1]).
- **T** is the channel's strength: the player's strength rating for that group times his size factor (§7.1).
- **W** is the channel's top speed [E]: legs 20, core 25, shoulder 30, wrist 40 rad/s, legs along s 3 m/s. [N1]'s golf values are 30–60 rad/s, also estimated.

A learned swing is one set of (t_on, t_rise, t_off) per channel, made for the player's own body (§5.5). His ratings set T, and the body does the rest.

**Which links get a drive** (the research's second finding, [N1], [N3], [N5]): torque generators drive the proximal links only.

- **For the batter:** legs, push, brake, trunk and shoulders. The elbows and wrists hold position with their small measured strengths.
- **For the pitcher:** the same, plus a series elastic element at the shoulder. The throwing shoulder's internal-rotation torque in a pitch, 90–120 N·m at about 115 rad/s ([N3]), exceeds the strongest measured voluntary values (52–80 N·m at ≤ 4.7 rad/s, [N5]), and no Hill curve has drive left at 115 rad/s. The last links' speed is made by the segments' dynamics (velocity-dependent torques: Hirashima 2008, Koike) and elastic recoil (Roach 2013/14).

The probe's batter shows the first half: its wrist drive fitted to off.

**What no source gives** (all [E], to be sourced or chosen in phase 1):

- the channels' top speeds W;
- the force–velocity curvature for lumped joint groups;
- the eccentric plateau;
- the friction coefficient on clay;
- each foot's free-moment reach;
- the wrist's cock limit as a stop (its value is [4]'s);
- the trunk's axial range;
- strength in N·m by group for professional hitters (note 2 §5: "not found as numbers"). The nearest are trunk rotation in golfers, hip, knee and ankle generator parameters in recreational men, and pitchers' shoulders, all at slow isokinetic speeds ([N5]);
- the lead foot's loading time in the 3D body (here, [4]'s 170 ms).

### 5.2 Where the drive's numbers come from

**Worked back from [4] (inverse dynamics).** `inverse.py` takes [4]'s mean pelvis and trunk rate curves, its bat azimuth, elevation and lag, and its pelvis travel. It computes the yaw torques the chain needs (de Leva's segments at [4]'s mean batter, 1.84 m and 92 kg).

- **The legs' yaw torque** rises from about 200 N·m at −180 ms to a plateau of 470–480 N·m from −140 to −110 ms, then falls through zero at −85 ms.
- **The yaw moment of [4]'s measured ground forces** peaks at the same time, −120 ms, at 222 N·m. That is computed about the pelvis from the four horizontal force curves, with the feet [E] 0.77 m apart and the lead foot 0.11 m closed ([4]).
- **Over −170 ms to contact,** the chain needs 35 N·m·s of angular impulse. The lateral couple gives 19, less 6 from the closed lead foot's braking force, so 13. The feet's free moments, which [4] does not report, and the planar reading of the arms (below) carry the rest.

**A sign [4] leaves open.** [4] states its axes as X toward the pitcher, Z up and Y = Z × X. Read so, its lateral forces would turn the batter *away* from the pitcher. They can only turn him toward it if Y points toward the plate, so the probe uses that sign and flags it. Welch's prose ([3]: the rear foot's shear "in −X" at foot off, i.e. the foot pushing back on the plate) suggests that shear forces in these labs may be the feet's action, not the reaction.

**What filtering and averaging do.** [4]'s markers were low-pass filtered at 12 Hz and its curves are means over batters aligned on five events. The inverse dynamics differentiates them twice, after a further 16 ms triangular smoothing.

- Torque features shorter than about 40 ms are lost.
- The torques near contact (the wrist's, and the shoulder's sign reversals in the last 30 ms) are noise.
- The averaged curve is no one's swing: peaks are blunted and spread.

The arm's direction is read as the bat's azimuth plus [4]'s lag, a planar reading of a 3D angle. It makes the arm's rate 1,418°/s at −90 ms against Welch's 1,160 at −65. So the ID's absolute torques are good to perhaps a factor 1.5, and its timing better than that. **Measured:** the motion and the ground forces. **Derived:** the torques. **[E]:** the segment geometry of §4.2.

**Fitted forward.** The forward fit (§4.3) is the better source of the drive for the model. It makes the measured motion come out of the drive, it needs no second derivatives, and its torques land on the golf generators' and Koike's (§4.3).

**From published pitching forces** (§11): Fleisig's and Attias's joint kinetics ([N1] §1.4) and the bowling generators, for phase 5.

### 5.3 What keeps a driven body on its path

A body driven by a fixed program does not stay on its path by itself. Two cases differ.

**A standing or striding body falls without feedback** (`balance.py`, output `sim/target/codex-probes/body/balance.out`). The model is a body over its foot as an inverted pendulum along the stride: centre of mass at 0.53·H [E: an athletic stance], the ankle torque bounded by where the centre of pressure can go under the foot. The plan is to stand, push the body into motion (the rear leg, [4]'s trail GRFx about 20% BW), then fall onto the front foot for 0.28 s ([4]: foot off to foot down). The planned ankle torque is worked back on the player's own model of his body. Errors: a lean 0.5° off at the start, the torque off by 1% of m·g·h, or his model's mass 2% off.

| Duration | Played back (open loop) | Muscles' own stiffness and damping (0.9·m·g·h, no delay) | Stiffness plus a reflex 100 ms late |
| :-- | :-- | :-- | :-- |
| 0.28 s (the batter's stride) | 1.2 cm off | 0.9 cm | 0.9 cm |
| 0.6 s | 2.7 cm | 1.0 cm | 0.9 cm |
| 1.5 s (a pitcher's lift and stride) | **falls at 1.32 s** (41 cm) | 1.8 cm (11 cm with the torque error) | 1.0 cm (1.4 cm) |

- **Over the size range** (stature 1.70–2.00 m, mass 75–120 kg; 1.5 s, the lean error), open loop drifts 37–46 cm and with the reflex stays within 0.91–1.05 cm. Mass drops out, because every term scales with it.
- **The reflex's delay** can be 50–200 ms (0.85–1.78 cm) and its gain 0.3–1.6 × m·g·h (0.86–0.97 cm). At 2.5 × it starts to ring (1.8 cm).
- **What is real:** the intrinsic stiffness is a measured property of muscle (Loram and Lakie 2002 put the ankle's at about 0.9 of the toppling stiffness, too weak alone, as the table shows). A feedback with a delay of the long-latency reflex (50–100 ms) to voluntary correction (150 ms and more) is how people balance.
- **What is a device:** the feedback acts on the difference from the *planned* posture, so the plan is the reference. That is equilibrium-point control in the motor-control literature (a real theory, not settled). The gains are [E], chosen in the range that holds without oscillating.

**The swing needs no feedback.** It is 0.25 s from the first drive, shorter than a voluntary correction loop, and the feet are planted. What holds it are real mechanisms:

- the planted feet;
- the force–velocity law, which acts as damping: a segment that runs ahead loses drive;
- the joints' range limits: the wrist's cock and the trunk on the pelvis.

In the probe a 15 ms late start of the whole drive gives a 15.0 ms late meeting (§6.2, "late swing"). The error carries; it does not grow. The ballistic part of the pitcher's delivery (the last 0.15 s of the arm) is the same.

**So the design:** postural feedback (intrinsic stiffness plus a delayed reflex, its reference the learned motion) through stance, load, stride, leg lift and balance, and pure feedforward through the swing and the arm's whip. A sensed event may trigger the next part of the program: the heel plant may trigger the hips (§5.4), a decision triggers the arms. But nothing ever reads the world (invariant 2). The reflex reads the player's own body as he senses it (proprioception), which in the sim is his true body state with his own sensing noise, an input of `bb_agents::motor`, not of his decisions.

### 5.4 The batter's timing when his body is real

The events of the motion, for the average batter. These are [4]'s times at 56 mph batting practice, scaled to the pitch's flight as in [swing-synthesis §3](research/swing/swing-synthesis.md); in the game they run on the batter's clock, timed to the pitcher.

| Event | When ([4]) | In the body |
| :-- | :-- | :-- |
| the load, the negative move | stance to −621 ms | postural program: weight to the rear leg, the body coils (the coil depth is a trait, §7.2) |
| lead foot off | −621 ms | the stride's program starts on the batter's clock, set by the pitcher's motion |
| toe touch | −340 ms (10% BW) | the lead foot's contact constraint switches on; its load starts building |
| the hips begin (on every pitch, D22) | pelvis rate rising from −245 ms | the legs' and core's drives on the pre-commit clock; in the fit they start at −228 and −238 ms |
| heel plant and weight commitment | −170 ms (50% BW) | in the model an event of the motion (the lead foot's load passing half); it can be the trigger of the full hip drive |
| the arms' commitment | [4]'s "first move" at −170 ms | the decision: the shoulder's and wrist's channels may start only after t_commit + T_latency (agent-models §3.4) |
| contact | 0 | the bat meets the ball wherever the motion took it |

**What he has already begun when he decides** (the walkthrough's "yes, yes, no"): the load, the stride and the start of the hips are the same on every pitch (D22), so the body is in motion before the decision. The decision starts the arms' channels or holds them. A take stops the hips' drive and applies the opposite drive, so the hips hold up and come back. Holding the weight back on off-speed is an extended hold: the hips' drive is delayed while the stride's drift goes on. Done well, the block (the lead brake) is still applied on time, so the weight stays back. Done badly, it is the lunge of §6.2.

**What the probe says about the decision clock** (`breakdowns.py`, "arms committed"). The hips and trunk run on their own clock and the arms' drive is delayed:

| Arms delayed | Meeting | Speed | Bat | Trunk at the meeting |
| :-- | :-- | :-- | :-- | :-- |
| 30 ms | only 6 ms late | 33.3 m/s | 6° further round | 93° against 63° |
| 60 ms | 21 ms late | — | 9° further round | — |
| 90 ms | 49 ms late | — | 13° further round | — |

So the body already turning buys the arms about 24 ms of every 30 ms they wait, but the trunk spins open without them: the signature of pulling off.

**Phase 3 must therefore set the decision clock from the body, not from a constant T_swing.** The latest commit that still meets the ball is a property of each learned swing. The clock's form, t_commit = t̂_arrival − T_swing − T_latency, stays, with T_swing replaced by the learned swing's own time from the arms' onset to contact. [4]'s 56 mph timing is slower than a game's; at 90+ mph, how the stride and hips compress is an open question (swing-synthesis §6), for phase 3 to settle against the game's flight times.

**The check swing** is a decision to stop: the arms' and hips' channels switch off, and an opposite drive (the antagonists) starts after the latency. Whatever the bat has, it keeps until the braking drive takes it away. Whether it crosses the plate is the ruling (rules-spec §4.5).

### 5.5 How a player produces a motion without reading the world

- **His learned swings.** For each player, when the roster is made, a set of drive programs is built on his own body:
  - one per aim on a grid over the zone and its edges (for example 5 × 5 heights and sides);
  - one per pitch-speed class, for the load and stride's timing.

  Each is found by an optimisation that a real hitter's practice stands in for. The objective: meet the aim's contact point at its time, with the most bat speed, from his launch posture, within the decision clock. The search: onsets, rise times and switch-offs, as in [N1]'s golf and bowling models, which fit 6–107 timings by global search.

  This is learning, done once. Per pitch nothing is solved. It is the owner's reading, and it answers the three reasons swing-model v3 §6.7.2 gave against a driven chain:

  - the unsourced numbers are now the drive's of §5.1, sourced where the literature allows;
  - no root find runs per pitch;
  - the batter's model of his own dynamics is the learned set itself.
- **Between aims.** For a pitch between grid points, his program is the weighted blend of the neighbouring programs' timings (onsets, rises, offs: times blend; strengths are his own). The body's response to the blend is whatever it is. A blend can miss what a dedicated swing would have hit, and that is a real limitation of practice. How fine the grid must be is set in phase 3 by how far a blended swing's contact falls from its aim (a test, not a fit).
- **What he can change in mid-motion.** Only drive not yet spent: the onsets that have not happened, the strengths of channels not yet on, a switch-off. The mechanisms are:
  - the late adjustment of agent-models §3.6 (`Adjustability`), as a re-blend of the not-yet-started channels toward a new aim, limited by the latency;
  - the check swing (§5.4).

  Nothing already integrated changes. An effort error drawn at the start is part of the program and cannot move anything earlier (the owner's ruling of 2026-11-24).
- **Invariant 2.** The agent builds and blends programs on *his own* body model with his own beliefs about the pitch. `bb_agents::motor` draws the execution errors (§6.1) and hands the executed program to the world, which integrates the true body (§10). The agent never sees the executed motion.

## 6. Errors (§14.1 item 4)

### 6.1 Where each error enters

| Error | Link it enters | Size set by | How it carries |
| :-- | :-- | :-- | :-- |
| onset jitter, per channel | the channel's t_on (and t_off) | coordination (§7.4): σ per onset, back-solved (below) | everything after the onset, through the dynamics |
| strength on the day | each channel's T, a common factor plus a small per-channel factor, multiplicative (σ ∝ drive, the supported form, [N6]) | fatigue, composure (PLAN §8.6) | the whole motion from that channel's onset |
| perception error | the plan itself: aim, arrival time, pitch type (agent-models §3.2) | `Vision`, `SpinRecognition` | the program chosen; the body executes it faithfully |
| the foot's timing | the lead foot's touch-down time (the stride's program) | coordination; the pitcher's timing against his expectation | the block and the hips (§6.2) |
| finger slip (pitcher) | the fingers' last contact on the ball | `SpinCapacity` or a feel rating [E] | the spin axis and the ball's direction (§11) |
| release timing (pitcher) | the fingers' opening | coordination | the ball's direction (up and arm-side early, down and glove-side late) |

**How big the errors are.** No study reports the trial-to-trial SD of a muscle group's onset or peak torque in hitting or pitching ([N6]). The sizes are therefore back-solved, in phase 3 for the batter and phase 5 for the pitcher: the onset and strength σ of the average player are set so that the body's own outcome spreads match the published ones. This is Tier B calibration of noise scales (PLAN §13), done on the body's result, never on a per-pitch outcome. Targets ([N6]):

| Target | Value |
| :-- | :-- |
| a batter's contact depth, within-batter σ | 0.27 m, about 7 ms at a 40 m/s pitch |
| a batter's bat speed, within-batter σ | 1.9 m/s (MLB, [S10] §2) |
| a pitcher's release point σ | 3–6 cm (MLB, 344 starters) |
| a skilled thrower's release window | about 7 ms; the fingers' release precision 1 ms (Hore and Watts 2011) |

A batter's tolerance of about 9 ± 6 ms ([N6]) is a *check* on the batter's body, which has the bat's path as a result, not an input. The probe gives the sensitivity the back-solve starts from: a 15 ms late start of the whole swing meets the ball 15.0 ms late at −0.9 m/s (§6.2). Late links need the tightest timing (1–10 ms windows); early links bear tens of ms ([N6] §5: Koike's ±50 ms re-timing of the legs). The probe agrees: a foot up to 60 ms late changes nothing (§6.2).

**No error moves anything earlier in the motion.** Each is drawn from `KeyedRng` before the motion starts (the stream's draw order is part of the replay contract, as in `motor::execute_release`). It changes one link at its time, and the integration carries it forward. A slower or weaker swing starts when it was planned and arrives late. The five independent errors of today's swing (height, plane tilt, reach, timing, effort) disappear: height, tilt and reach become results of the onsets and strengths.

### 6.2 The named breakdowns, from the body

`breakdowns.py` (output `sim/target/codex-probes/body/breakdowns.out`). Each case changes one link of the fitted average swing; the ball is a fixed point where the average swing's sweet spot meets it. "Meet" is when the bat line passes that point (+ = late); "u" is where on the bat, from the hands (sweet spot 0.634 m, the end 0.784 m); "angle" is how much further round the bat is at the meeting (+ = pulled).

| Case | Meet ms | u m | Speed m/s | Angle ° | Separation peak ° | Pelvis / trunk at meeting ° |
| :-- | --: | --: | --: | --: | --: | :-- |
| average swing | 0 | 0.634 | 30.7 | 0 | 16.3 | 64 / 63 |
| late foot +120 ms (drive on its own clock) | +4.1 | 0.634 | 30.4 | −0.8 | 7.1 | 50 / 59 |
| late foot +160 ms (drive on its own clock) | +27.9 | 0.566 | 25.2 | −8.7 | 1.6 | 4 / 33 |
| late foot +120 ms, hips wait for the plant (+37) | +38.4 | 0.599 | 27.9 | −3.4 | 18.2 | 50 / 54 |
| late foot +160 ms, hips wait (+77) | +84.7 | 0.511 | 22.0 | −7.7 | 20.1 | 22 / 33 |
| hips 20 ms early | −3.1 | 0.642 | 31.3 | +1.3 | 32.7 | 84 / 67 |
| hips 40 ms early | −7.0 | 0.643 | 31.8 | +2.4 | 46.2 | 101 / 72 |
| casting: wrists from −150 ms | −2.6 | 0.539 | 27.7 | +10.5 | 16.3 | 62 / 61 |
| casting: wrists from −100 ms | −1.6 | 0.594 | 29.9 | +3.4 | 16.3 | 63 / 62 |
| casting from −150 ms at the wrists' measured strength, 20 N·m ([N5]) | −1.3 | 0.589 | 29.4 | +4.5 | 16.3 | 63 / 63 |
| pulling off: trunk 20 ms early | −10.2 | 0.660 | 32.2 | +3.8 | −0.7 | 56 / 78 |
| pulling off: trunk 40 ms early | −15.0 | 0.677 | 33.1 | +4.5 | −0.9 | 50 / 85 |
| lunging: rotation held 60 ms, the drift runs on | +63.2 | 0.560 | 25.1 | −5.9 | 19.3 | 36 / 44 |
| lunging: held 100 ms | +114.9 | 0.435 | 17.6 | −13.8 | 20.2 | −1 / 12 |
| a plain 15 ms late start (for scale) | +15.0 | 0.623 | 29.8 | −1.4 | 17.1 | 60 / 60 |

What each shows, with what is real and what is not yet:

- **Late foot.**
  - Up to about 60 ms nothing happens, because the foot is down well before the hips need it.
  - Later than that, the hips fire against a foot that carries no weight, so the friction couple is not there and the pelvis cannot turn: jammed and weak. This is the walkthrough's "Too Late: slow load or late foot-plant → near the handle, weak".
  - If instead the hips wait for the heel plant, the whole swing slides late by the wait: later still, but with the sequence intact.

  Which of the two a hitter does is a control rule (§14).
- **Hips early.** Early and pulled, with the ball nearer the end of the bat, as the walkthrough says ("off the end of the bat; rollover to the pull side"). The power loss that coaches also describe does *not* come out (31.8 m/s against 30.7). The probe's trunk has no force–length relation: a stretched trunk muscle beyond its best length loses force, and that is the mechanism by which over-separation costs power. Its range stop at 45° only just engages (46°). The 3D body must carry force–length for the trunk channel; this is a known gap.
- **Casting.** Slower (−3 m/s), the ball 10 cm toward the handle, the bat 10.5° further round: "drastically slows the swing … forces the batter to roll over". The 60 N·m wrist drive used is the fit's own, three times what two wrists hold. At the measured 20 N·m the same error costs 1.3 m/s and 4.5 cm. The arc widens through the wrist, the only path the reduced arm has. In the 3D body the hands also leave the back shoulder by the elbows and shoulders, whose strength is larger, so the 3D figure is expected to lie between the two rows.
- **Pulling off.** Early and pulled toward the end of the bat. The part the walkthrough stresses, the barrel yanked out of the zone and the head pulled away, is vertical and perceptual and outside the planar probe (§4.2). The 3D body carries both.
- **Lunging.** The drift runs on, the weight goes forward (0.53 m against 0.44), and the rear foot unloads, so the yaw couple shrinks (cap 212 N·m against 251). The pelvis never turns, and the swing is arms only: 17.6 m/s, off the handle. That is the walkthrough's "only weak arm strength". Against a changeup arriving about 80 ms later than the fastball ([4]: 21.3 against 25.0 m/s over 13 m), the 100 ms hold is roughly on time. The failure is the power, as it should be.

Not tested in the probe (each needs the 3D body or the pitcher): the collapsed front leg, flying open, early and late release, finger slip (§11).

## 7. Ratings, and the owner's consolidation question (§14.1 item 5)

### 7.1 Two separate counts

How many drive channels the *model* integrates and how many *ratings* a player shows are separate choices.

- **The model** needs one channel per joint group that the breakdowns and the body's physics distinguish. In the probe that is six; in the 3D body about twelve: legs' yaw, rear-leg push, lead-leg brake and extension, trunk rotation, trunk tilt, lead shoulder, rear shoulder, elbows, wrists, and the fingers for the pitcher.
- **The ratings** feed the channels' strengths T and the onsets' σ.
- **Strength scales with size.** A channel's T is the rating times (`Mass`/92 kg) [E]: at a fixed build, torque goes as muscle cross-section times moment arm, i.e. as mass. Yanaka and Morishita 2026 ([N2] §2.4) found hip and trunk torque to follow muscle and body size, the bat-speed correlations vanishing once mass was controlled.

### 7.2 What the probe shows for the strength levels

`ratings.py`, output `sim/target/codex-probes/body/ratings.out`.

**The same hitter swinging easier or harder** (every drive × 0.8 to × 1.2, nothing re-learned):

| Effort | Bat speed | Time from the first drive | Path |
| :-- | :-- | :-- | :-- |
| × 0.8 | 28.4 m/s | 256 ms | 1.97 m |
| × 1.0 | 30.6 m/s | 239 ms | 1.99 m |
| × 1.2 | 32.5 m/s | 226 ms | 2.00 m |

- Time × speed: 7.26–7.36 m.
- A slower swing by the same hitter is as long and so slower, as MLB's within-batter data say ([S10] §2: 0.7% shorter for a 4.5% slower swing at the same depth).
- This is the driven chain's similarity, and it comes out by itself.

**Across hitters, a single strength rating with the average's swing** gives exactly the row above: the same path at every strength, the time ∝ 1/speed. MLB's hitters do not do that: length rises with bat speed and length/speed is flat at 68–70 ms ([S10] §1).

**The lever.** The probe gave each hitter's learned swing one more freedom, the launch coil (pelvis, trunk, arms and bat starting further back or less far). It chose the coil that keeps the swing's time at the average's 239 ms from the first drive. The swing time moves about 0.43 ms per degree of coil, monotonically. (The bat's wrap alone barely moves it: 239–250 ms over −40° to +10°.)

| Strength | Bat m/s | Coil, + = more than the average | Path m | Length / speed |
| :-- | --: | --: | --: | --: |
| × 0.70 | 23.1 | −45° (the scan's end; the time not held: 245 ms) | 1.51 | 65.2 ms |
| × 0.85 | 26.6 | −27° | 1.71 | 64.2 ms |
| × 1.00 | 30.6 | 0 | 1.99 | 64.8 ms |
| × 1.15 | 34.0 | +26° | 2.29 | 67.4 ms |
| × 1.30 | 36.3 | +45° (the scan's end) | 2.52 | 69.4 ms |

- Length/speed comes out 64–69 ms, near MLB's 68–70 ms. Savant's start of tracking is undefined, so the absolute value is not a test; its flatness is.
- The slope is 0.11 ft per mph against MLB's 0.05–0.09.
- But the coil this needs, about ±26° for ±15% strength, is large. One SD of MLB bat speed (4.2%) is about 9% of strength in the chain (speed goes as √strength), so ±15° of coil per SD. The coil's spread between hitters in [6] (a cross-check only, D21) is 8–26° p10–p90, about ±7° per SD.

So strength plus coil reproduces MLB's relation with about twice the real spread of coil. Something else must carry the rest; candidates are the hands' load, the stride and the approach to contact depth. Where else it comes from is phase 1's question in 3D, and how the roster draws coil is the owner's (§14).

**Two hitters with the same bat speed, built differently** (strength shapes scaled to the average's 30.6 m/s, same launch, re-timed only):

| Hitter | Group strengths (legs, core, shoulder, wrist) | Pelvis peak | Trunk peak | Arm peak | 10 ms late start | Foot 150 ms late |
| :-- | :-- | --: | --: | --: | :-- | :-- |
| strong legs, weak arms | 1.37, 1.05, 0.79, 0.79 | 711°/s | 913 | 1,222 | +10.3 ms, −0.1 m/s | +26 ms, **−1.1 m/s** |
| weak legs, strong arms | 0.76, 1.01, 1.31, 1.31 | 438°/s | 527 | 1,086 | +10.4 ms, −0.2 m/s | +24 ms, **−1.9 m/s** |
| tall lean, 1.95 m 85 kg (one rating) | 1.06 all | 570 | 700 | 1,125 | +10.3, −0.2 | +27, −1.7 |
| short thick, 1.75 m 100 kg (one rating) | 0.94 all | 577 | 708 | 1,166 | +10.6, −0.2 | +22, −1.3 |

- The two strength shapes are clearly different hitters at the same bat speed: they turn their hips at very different rates (visible on screen), and a lower-body breakdown costs the arms hitter nearly twice the speed. Their response to a plain timing error is the same.
- The two body shapes at one rating differ much less once matched. With one rating, size alone separates players: the tall lean hitter needs a 13% higher rating than the short thick one for the same bat speed.

### 7.3 The levels compared

| | One drive and one strength rating per channel (6 in the probe, ~12 in 3D) | A few groups: lower body (legs, push, brake), and upper body (trunk, arms, wrists); trunk optionally its own | One strength plus coordination |
| :-- | :-- | :-- | :-- |
| Breakdowns it can express | all; each channel can have its own timing σ, so a hitter prone to casting differs from one prone to lunging | all, as errors; proneness per group (e.g. a lower-body timing σ and an upper-body one) | all, as errors, but every hitter is equally prone to each: one σ for all onsets |
| Different hitters at the same bat speed | yes, many ways, most of them invisible to a manager | yes: legs hitter against arms hitter (§7.2), visible in hip speed and in lower-body breakdowns | only through `Height` and `Mass` |
| MLB's spreads | each channel needs its own population distribution and correlations, none of which is published ([N2] §5) | two distributions and one correlation | one distribution; easy to match bat speed's 4.2% SD |
| Unsourced numbers per player | ~12 strengths + ~12 σ | 2–3 strengths + 2 σ | 1 strength + 1 σ |
| Roster drawing | needs a covariance no source gives | lower and upper strength drawn correlated [E], each scaled by mass | one draw |
| PLAN §7's rule (one meaning, one unit, one mechanism) | each a torque in N·m on its own channel | each a torque scale on its group's channels, in N·m at the average size | a torque scale on all channels |

### 7.4 Recommendation

The model integrates every channel the body needs (about twelve in 3D), and a player shows these ratings:

| Rating | Meaning | Replaces |
| :-- | :-- | :-- |
| `Height`, `Mass` (kept) | segment sizes, masses and inertias ([dL]); strength scales with mass | — |
| `LowerStrength` | torque scale of the legs' channels: hip rotation, push, brake and block | part of `BatSpeed` and `ArmSpeed` |
| `UpperStrength` | torque scale of the trunk's and arms' channels (for a pitcher, the throwing arm's) | the rest of `BatSpeed` and `ArmSpeed` |
| `Coordination` | σ of every onset, ms; how precisely the sequence is timed | `Timing`, `BatControl` and `Command`, as their cause |
| `LoadDepth` (batter) | how far his learned swing coils; drawn correlated with strength | part of `QuickHands` (with strength, it sets the swing's time) |

Bat speed, swing time, fastball speed and command become results of the body, reported as before.

`Adjustability` stays as the speed of a mid-swing re-blend (§5.5). `SpinCapacity` and a pitch's own feel stay for the fingers (§11). `Stamina` stays: fatigue is less drive.

Why two strength groups and not one: §7.2 shows that one rating cannot make a legs hitter and an arms hitter, and the owner's breakdowns split along that line (the late foot, the block and lunging are lower body; casting and the release are upper body). Why not more: no source gives a per-channel distribution, and the extra ratings would change little a manager can see. A third group (trunk) can be split off later without changing the model.

## 8. Replay and speed (§14.1 item 6)

### 8.1 The integrator

**Classical fourth-order Runge–Kutta at a fixed step:**

- 1 ms through the swing, the stride's landing and the arm's whip;
- 2–4 ms in the quiet phases (stance, load, leg lift).

Activation is advanced once per step by its exact exponential (`m::exp`) and held through the step's stages. The non-smooth parts (the contact constraints switching, the range stops, the friction caps) are evaluated at each stage's state in a fixed order. Events (toe touch, heel plant, the bat passing a point, release) are located by linear interpolation within the step. Every transcendental goes through `bb_math::m`, the linear solves use partial pivoting in a fixed order, and no hash iteration is involved, so the same build, seed and commands give the same motion bit for bit (invariant 5).

**Checks** (`checks.py`, output `sim/target/codex-probes/body/checks.out`):

- **Energy.** The kinetic energy gained equals the work of every force, integrated with the same stages, to 2.4 mJ in 450 J (5.4 × 10⁻⁶; the remainder is the non-smooth caps and stops, where RK4 loses its order).
- **No forces** from −100 ms: kinetic energy constant to 1 × 10⁻¹¹.
- **Internal drives only:** forward momentum constant to 7 × 10⁻¹⁰ N·s.
- **Step size:** at 2, 1, 0.5 and 0.25 ms, contact comes at +0.93, +1.11, +0.97 and +0.94 ms, and the sweet-spot speed at 30.663, 30.648, 30.648 and 30.647 m/s. A 2 ms step is within 0.2 ms and 0.02 m/s of the finest. The wobble of a tenth of a millisecond comes from the activation being held through each step, which phase 1 can remove by integrating activation as a state.

Phase 1 adds angular momentum about the vertical for the free-floating 3D body, and the loop constraints' drift.

### 8.2 The cost

**Measured.**

- The probe in Python: 0.13 s per 270 ms swing at a 0.5 ms step.
- The same chain in Rust (`bench_chain.rs`, opt-level 3, one core): **1.7 µs per RK4 step**, so 0.46 ms per swing at 1 ms and 0.23 ms at 2 ms (`sim/target/codex-probes/body/bench_chain.out`).

**Estimated for the 3D body** [E: from Featherstone's articulated-body algorithm, about 450 flops per revolute degree of freedom per evaluation, plus the feet and the hands' loop]:

- about 35 degrees of freedom is about 10–20 µs per evaluation, so 40–80 µs per RK4 step;
- per pitch: the batter's load, stride and swing or take (about 0.6 s) and the pitcher's delivery (about 1.5 s) at 1–4 ms steps is about 800 steps, about 50 ms;
- **at 290 pitches a game, about 15 s.** Integrating only the phases an error can change (below), about 250 steps a pitch: **about 6 s.** Against PLAN §13.4's 200 ms per game, 30× over.

**What would fit:**

1. **Tabulate what no error reaches.**
   - The stance, load and leg lift are the same on every pitch up to the first error's onset.
   - The pitcher's delivery up to the arm's acceleration is computed once per pitcher, pitch type and fatigue level.
   - The learned swings are built and tabulated when the roster is made (§5.5).
   - Per pitch, integrate only from the first link an execution error enters (about 0.25 s for the batter, 0.2 s for the pitcher's arm), at 1 ms near release and contact and 2 ms elsewhere.
2. **A 3D core for the per-pitch integration**, about 12 degrees of freedom:
   - pelvis yaw and tilt;
   - trunk rotation, side bend and flexion;
   - the lead arm's shoulder (2) and elbow;
   - the wrists' cock;
   - the bat's roll;
   - the body's forward travel.

   With the rest of the skeleton posed from it for drawing, and its parameters derived from the full body. At about 2 µs per evaluation, 8 µs per step: 250 steps is 2 ms a pitch, **about 0.6 s a game** (0.25 s at 2 ms steps throughout).
3. **The budget.** A planar core like the probe's would fit 200 ms (about 0.1 s a game) but cannot give a bat's elevation or attack angle, so it cannot carry games. A 3D core needs about 0.3–0.6 s per game. That is about 20–40 s for a 972-game season on 16 threads, against §13.4's 15 s. The choice is the owner's (§14).

## 9. What the timeline carries, and what the client draws (§14.1 item 7)

- **The client draws the sim's own body.** Per pitch it needs each segment's pose in time. Storing 16 segments' positions and rotations at 240 Hz is about 200 kB a pitch, 60 MB a game: too much.
- **Recommended:** the timeline carries each motion's *executed drive program* (onsets, rises, offs, strengths; about 100 bytes a player a pitch), its start state and the events (toe touch, heel plant, contact, release). The client calls the same body integrator through `bb_ffi` and re-runs the motion deterministically, as the sim did (invariant 5 makes the replay exact). For the 3D core of §8.2, the client poses the full skeleton from the core as the sim does for its own checks.
- **Hand IK is not needed.** The hands are on the bat because the body put them there.
- **The follow-through** is integrated past contact with the drives switching to braking (the eccentric absorption of [N2] §2.4) until the finish. PLAN D24 (the sim's bat track ends 0.03 s after contact) then applies only to what the world reads; the client draws the body's own finish.

## 10. Where the model lives in the crates

- **`bb_physics::body`** (new, pure):
  - the segments, joints, contacts and loop;
  - the drive law of §5.1;
  - the postural feedback law of §5.3, as a function of a sensed state passed in;
  - the integrator of §8.1.

  No randomness, no world types. Both the agent, on his own body and beliefs, and the world, on the true body, call it.
- **`bb_agents`:** the player's learned swings and deliveries (built at roster time by `bb_game`'s roster generator calling `bb_physics::body` with the player's nominal body), the blend between aims, the decisions that start, hold or stop channels.
- **`bb_agents::motor`:** the execution errors of §6.1, drawn from `KeyedRng` with new `Purpose` values (appended, never reused). It returns an `ExecutedDrive` that only `bb_game` and the world read (invariant 3).
- **`bb_world`:** integrates the true body with the executed drive and samples the bat's pose in time into `BatPose`. `detect` and `resolve_contact` are unchanged. The pitcher's release (the ball's state as the fingers let go) becomes the world's input to `FlightEnv`, in place of today's planned-plus-noise release.
- **`bb_game`:**
  - orchestrates the per-pitch order: the pitcher's motion first, then the batter's to the ball;
  - tabulates the per-player motions;
  - writes the timeline's drive programs.

  The crate graph test stays: `bb_agents` uses `bb_physics`, not `bb_world`.

## 11. The pitcher, at design depth

- **The body.** The same engine and the same segment table, with:
  - the pivot (rear) foot planted on the rubber;
  - the stride foot free through the leg lift;
  - the stride landing as a contact with the mound's slope (the mound's geometry is already in the game);
  - the throwing arm's shoulder (3, its internal rotation the fastest joint in sport), elbow (flexion and forearm rotation) and wrist (2);
  - a **series elastic element at the shoulder**, stretched in the cocking phase (maximum external rotation) and recoiling into internal rotation;
  - the ball held by the fingers.

  The generators drive the legs, the trunk and the shoulder's early motion. The elbow's and wrist's late speed comes from the segments' dynamics and the elastic recoil ([N3] §6, [N5] §5: the delivery's 90–120 N·m of internal-rotation torque at about 115 rad/s exceeds any measured voluntary strength). The published shoulder and elbow torques (ASMI: internal rotation 90–100 N·m, horizontal adduction 90–120, elbow flexion 49–73, [N3]) are checks on the model, not drive profiles. The delivery's timing pattern is the same at every level ([N3]: pelvis peak at about a quarter, trunk at about half the foot-contact-to-release interval), which supports one delivery shape per pitcher scaled by his strengths.
- **His release is a result:** the hand's position and velocity, and the fingers' last contact, at the instant the fingers' drive opens them and the ball rolls off.
  - Release speed, point, direction and spin are what the arm and fingers deliver.
  - An early opening lets the ball go while the hand is still rising along its arc and before it has turned in: up and arm-side.
  - A late opening, the opposite: down and glove-side.
  - Each millisecond of release timing turns the ball's direction by roughly the hand's path speed over its radius [E: 40 m/s over about 0.7 m, about 3° a ms]. That is why command is a matter of milliseconds. Skilled throwers release within about a 7 ms window, and their fingers time the release to about 1 ms through the ball's back-force on them (Hore and Watts 2011, [N6]); finger forces of 50–100 N end 4–7 ms before release ([N3] §6).
  - So the release is not a free-running timer. It is a hand-and-finger event that follows the hand's acceleration, and an error earlier in the arm becomes a release error through it.
  - A finger slip moves the last contact and with it the spin axis: the slider that backs up.
- **Flying open** is the trunk's drive early against the stride foot's plant, as in §6.2, and the arm drags: speed lost, the ball arm-side.
- **A collapsed front leg** is the lead leg's block channel too weak or late: the knee bends after foot strike, the body's forward momentum is not stopped, and the arm's whip loses what the block would have given it. Felton et al. 2023's bowling model ([N1] §1.2) found that a more extended front knee is common to every optimised delivery, the precedent for this mechanism.
- **Fatigue** is less drive (lower T, slower activation): speed falls, the onsets' σ grows.
- **Holding the delivery on its path.** The 1.5 s delivery includes the leg lift on one foot, where §5.3's table says a played-back body falls (1.32 s). The postural feedback carries the lift and stride, and the arm's last 0.15 s is ballistic.
- **The numbers:** Attias et al. 2026's OpenSim model of one MLB starter (34 DOF, Moco, [N1] §1.4), the bowling generators, and the ASMI joint kinetics of [N3]. The pitcher's equivalent of [4] (a live-pitching kinematic and kinetic set of professionals with means and bands over time) is not among the notes. Fleisig's ASMI professional series (n = 60) exists but was not opened as numbers ([N3] §7). Getting it is phase 5's first task. OpenBiomechanics' pitching data stay a cross-check only (D21, [N7]).

## 12. Not established, and what would settle it

- **The 3D body's fit to [4] and [3].** The probe is planar. Phase 1's gate is the 3D fit within the measurements' SD, including the bat's elevation (+44° to −30° from −340 ms to contact, [4]), the pelvis and trunk at contact (77° and 75°), and the lag at contact (65°).
- **The sign of [4]'s lateral forces** (§5.2): the dissertation's raw axes, or Fortenbaugh, would settle it.
- **Strength by joint group for professionals in N·m**, its spread, and any value above about 5 rad/s: not found ([N2] §5, [N5] §6). The nearest are the isokinetic values of [N5] and the 9-parameter torque functions of Yeadon and King ([N1] §1.2, coefficients not opened).
- **Movement variability**: no σ of onsets or strengths is published ([N6]). The sizes are back-solved from outcome spreads (§6.1).
- **Licences for shipped numbers** ([N7]): no open, commercially licensed adult dataset with ground forces exists, so shipped values come from published means and SDs, cited (as [4] and [3] here). The Statcast-derived constants already in the project rest on MLB's terms, which [N7] could not read; that needs settling before release.
- **What else shortens a weaker hitter's swing** besides the coil (§7.2).
- **The trunk's force–length relation** and its range of axial rotation (§6.2, hips early).
- **The decision clock at game speeds** (§5.4): how the stride and the hips compress at 90+ mph.
- **The lateral-force couple's other half** (§5.2): free moments under batters' feet.

## 13. Phases 1 to 5, sized

Sizes are in worker sessions of about the length of this one; Opus for physics, Sonnet for porting and tests.

| Phase | What it needs, in order | Size | What could stop it |
| :-- | :-- | :-- | :-- |
| 1. The body in the checking model (Python oracle) | 3D multibody of §4.1 (articulated-body algorithm or a Lagrangian with constraint forces for the feet and the hands' loop); the drive of §5.1; postural feedback; fit of the average batter to [4]'s curves and bands (pelvis, trunk, shoulders, elbows, bat azimuth, elevation and lag, ground forces); the breakdown tests of §6 as goldens; energy and momentum checks | 4–6 Opus sessions | the 3D fit not reaching [4]'s bands, above all the bat's elevation and the arms' shape; the closed loop of the two hands; [4] gives means only, so the per-batter spread cannot be fitted, only checked |
| 2. The engine in Rust | `bb_physics::body` bit for bit against the oracle; a `sim-cli` cage command; the client draws one swing through `bb_ffi` | 2–3 sessions (Sonnet port, Opus review) | performance (§8.2), if the core of §8.2 has to be derived from the full body first |
| 3. The batter in plate appearances, behind a switch | learned swings built at roster time (§5.5) and their grid; the decision clock from the body (§5.4); the motor errors (§6.1); check swing as drive stopped; cage rates and direction checks over the rating range | 3–4 sessions | building learned swings robustly for every body (the optimisation must not fail for extreme ratings); the blend between aims being too coarse; the decision clock at game speeds |
| 4. The batter in games | the switch on; the ratings of §7.4 and the roster generator; recalibration through ability distributions only; ledger rows; Opus review; the client drawing the sim's body | 3–4 sessions | the cost budget (§8.2, an owner decision); run environment shifts that the distributions cannot reach, which would mean a model is wrong (PLAN §13.2) |
| 5. The pitcher | a pitching dataset first (§11); the delivery's body, fingers and release; the same gates as 1–4 | 5–7 sessions | the release model (the fingers' opening and the spin) has no lumped-model precedent; a 1.5 s horizon held by feedback; command at millisecond precision |

## 14. For the owner to decide

**How much computing a game may take.**

- *Now:* PLAN §13.4 allows 200 ms per game; today's swing and release are formulas and fit easily.
- *What is wrong:* a body driven by forces on every pitch costs more. The probe's simple body fits, but it moves only in a flat plane, so it cannot swing up or down at a pitch. A full body costs about 6 s a game.
- *Options:* (a) keep 200 ms with the flat body, which cannot hit a ball in the air the way a real swing does, so it is not offered; (b) a three-dimensional core body of about 12 joints, computed on every pitch only from the moment an error can enter, with everything else computed once per player. That is about 0.3–0.6 s a game, a 972-game season in 20–40 s on 16 threads instead of 15 s; (c) the full body every pitch, about 6 s a game, a season in about 6 minutes.
- *Recommendation:* (b), with the budget raised to 0.6 s a game.

**Whether learned swings may be built by search.**

- *Now:* the swing is drawn to meet the planned contact on every pitch, which the owner has ruled out.
- *What is proposed:* each player's swings are found once, when he is created, by a search over the drive's timings that "practises" until the swing meets the target fast. On every pitch his body then runs one of those swings forward, with its errors, and meets the ball wherever it goes.
- *The question:* is a swing found by practice at roster time acceptable under "no motion is fitted to its planned end", given that the pitch-by-pitch motion is never fitted?
- *What the literature says* ([N1], the research's first finding): every published model of this kind, in golf, cricket, batting and pitching, finds its drive timings by a search against a measured motion. None produces a motion any other way. A one-time search is therefore the established method, here pointed at the player's own target instead of at a recording.
- *Recommendation:* yes; it is how a hitter's swing comes to be. No game motion is adjusted to its result.

**What the hips do when the front foot is late.**

- *Now:* there is no front foot in the sim.
- *Options:* (a) his hips fire on his own clock regardless: a late foot leaves them nothing to push against, and the swing is weak and jammed (25 m/s, 7 cm toward the handle at 160 ms late); (b) his hips wait for the heel plant, the trigger in your walkthrough: the swing is late but intact (85 ms late, 22 m/s at 160 ms); (c) a mix that differs by hitter, set by a trait.
- *Recommendation:* (b) for every hitter, as the walkthrough describes, with (c) later if the owner wants variety.

**The ratings players show.**

- *Now:* `BatSpeed`, `QuickHands`, `BatControl` and `Timing` for batters, `ArmSpeed` and `Command` for pitchers set results directly.
- *What is wrong:* under D31 those are results of the body.
- *Options:* one strength rating per joint group (about a dozen); two groups, lower and upper body; or one strength rating; each with `Height`, `Mass` and coordination.
- *Recommendation:* lower-body strength, upper-body strength, coordination, and for batters a load depth (how far he coils), with height and weight (§7.4). One strength rating cannot tell a legs hitter from an arms hitter (§7.2); a dozen have no data to draw them from.

**How deep a hitter's load is.**

- *What the probe found:* MLB's stronger hitters swing longer paths in about the same time. In the body, that comes out only if stronger hitters also coil further, and by about twice the spread real hitters show.
- *Options:* (a) draw load depth correlated with strength, accepting that the coil alone carries the relation until phase 1 finds the other part in the three-dimensional body; (b) draw load depth independently and let the relation be whatever the body gives.
- *Recommendation:* (a), and phase 1 measures whether the hands' load and the stride take up the rest.
