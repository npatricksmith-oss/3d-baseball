# Independent review: phase 1, session 7 of the force-driven body

Reviewer: rev-body-3d-7 (Claude Opus 5.5, high effort), 2026-11-25. Range `90ce8c8..6c5ae8d` on `work/body-3d` (merged to main as `b212f7b`). Job: `REVIEW-BODY-3D-7.md`. I edited no source, test, spec or note; my probes and their output are in `sim/target/codex-probes/review-body-3d-7/`.

**Verdict.** The engine work, the limits and the refit's bookkeeping are sound and reproduce exactly. §4.8's main point also stands: [4]'s 22.9 m/s is not what its batters swung. What does not hold is the *size* §4.8 gives and how firmly the owner is told it. The 27.6–29.9 m/s ("62–67 mph") rests on a fitted shape that jumps at the edge of its window. Part of it comes from an "unsmoothed" barrel-drop curve that has the bat chopping down at about 20°. So the second half of the recommended Q59 option, replacing the fit's targets inside the last 40 ms with unsmoothed ones, does not follow and would mislead the next fit. The refit's own swing also chops down (−7.8° at contact), and no check catches it.

## P1 findings

### P1-1. The "unsmoothed" elevation curve is not a hitter's swing, and option (b) would make it a fit target

**Where:** [body-motion §4.8](../body-motion.md#48-where-the-placed-motions-bat-speed-goes-session-7-step-0-a-diagnosis), docs/body-motion.md:749 (the elevation row), :752 (27.6–29.9 m/s, "in elevation 3.4–4.5"), :776–777 (options and recommendation), :262–266 (the owner's options).

**What is wrong.** §4.8 undoes the filter on the bat's elevation as well as its azimuth. The barrel is then "still falling at 616–755°/s" at contact where [4]'s curve has it nearly level (36°/s). That elevation turn supplies 3.4–4.5 m/s of the 27.6–29.9. I ran the placed motion on §4.8's own undone curves (`attack.py`, using `batspeed.unsmooth`). The sweet spot's velocity at contact then points **20–22° downward** (9.7–11.3 m/s down), against +5.6° for [4]'s motion as placed and OpenBiomechanics' +4.1 ± 5.9° ([note 6](../research/swing/6-openbiomechanics.md)). Major leaguers average about +8–10° ([note 7](../research/swing/7-video-xHktmnRAx1s.md)). A −20° attack angle is about 4 SD below the measured hitters. With only the azimuth undone (2nd-order, corrected variant), the placed motion has **27.8 m/s at an attack angle of −1.8°**. So the speed survives without the elevation part, but §4.8's breakdown of it does not.

**What follows.** Options (a) and (b) both say the fit's targets "inside the last 40 ms become the unsmoothed ones". Done as §4.8 computes them, the elevation target would teach the body to chop down on the ball. The same may hold for the lag target (65° read as 58–61°), which I did not check.

**Fix.** Before the owner rules, drop "replaced by unsmoothed ones" from (a) and (b). Instead, leave the filter's footprint (about ±40 ms around contact) out of the misfit, or weight it down. Keep the contact time and the gate's direct speed band. Add the attack angle at contact as a check (OpenBiomechanics +4.1 ± 5.9°). If unsmoothed targets are kept for the azimuth, state their shape as [E] beside a range (P1-2).

### P1-2. The size of the correction is stated more firmly than the evidence allows, and its basis is described wrongly

**Where:** docs/body-motion.md:753 ("rests on the bat's rate rising in a straight line over its last 70 ms"), :773 and :253 (the owner's "62–67 mph … as nearly as it can be undone"), `analysis/research/body/batspeed.py` `unsmooth()`.

**What is wrong.**

- **The straight pieces are not straight lines joined to [4]'s curve.** The basis functions inside ±70 ms are free and the outside is [4]'s curve, so the fitted "true" rate jumps at −70 ms. I printed the pieces (`edge.py`). Every straight variant starts **264–409°/s below [4]'s curve at −70 ms**: the bat would lose 300–400°/s instantly 70 ms before contact, then rise more steeply than any straight line from [4]'s value could. The curved variants jump the other way, by +276 to +1,361°/s. "A straight line" is therefore not the conservative case §4.8 calls it.
- **My own fit gives a different picture** (`filt.py`, `filt2.py`: scipy `butter` and `filtfilt`, not `batspeed.py`). There the rise is joined continuously to [4]'s curve at −70 ms and the step falls at contact.
  - A straight rise gives almost **no correction**: 2,380–2,440°/s just before contact, a step of 0.03–0.16 of a free bat's. It also fits worst (RMS 83–97°/s), and its filtered curve loses only 3% by contact against [4]'s 8.4%.
  - An accelerating rise (u^k) fits far better: RMS 20–36°/s for the 2nd-order corrected filter. It reproduces [4]'s shape: peak at −10 ms, 6–7% lost by contact, 1,757 at +15 ms (1,745) and 1,441 at +35 (1,451).
  - Capping the step at a free bat's 1,519°/s (an upper limit from the collision) puts the rate just before contact at **3,260–3,390°/s**. With k = 2 it is 2,950–3,020.
  - A bat that peaks 10–30 ms before contact and holds there gives 2,100–2,300°/s with no step, at RMS 86–110.
  - Filtering the marker positions (a rotating bat direction) in place of the rate curve moves these by about 30°/s. Not a concern.
- **What this means.** The direction is robust. The best-fitting continuous shapes give *more* than §4.8's 2,649–2,841, so §4.8's range is closer to the low side than the middle. But [4] alone cannot set the number. It spans from no correction (a shape that fits badly) to about 3,400°/s, and §4.8's figures come from one parametrisation with an unphysical jump. "62–67 mph, as nearly as it can be undone" reads as a measurement. It is one estimate within a wide range.
- **A held bat does not lose less in the impact** (:741). The collision lasts about a millisecond, and the hands' forces carry little impulse in that time, so the handle is effectively free. A held bat loses about a free bat's 1,519°/s at once and recovers some of it afterwards. Physics therefore favours the larger step and the larger correction. (The owner's "27 mph off the barrel at once" is that free-bat figure, while §4.8's own fit used 0.52–0.71 of it. The two should agree.)

**Fix.** In §4.8 and in "For the owner", state the size as a range across shapes, with the shape as [E]. Put the robust facts first: [4]'s own exit velocity needs at least 23.1–24.7 m/s at the sweet spot (below), so 22.9 cannot be the batters' true speed. In `unsmooth()`, join the inside pieces to [4]'s curve at ±70 ms (or report the jump). Then rerun the four filter variants with straight and accelerating rises.

## P2 findings

### P2-1. The fitted swing chops down on the ball, and no held-out check reads it

**Where:** §4.9's held-out checks (docs/body-motion.md:818–825); the owner's "it also swings more like a hitter" (:256).

On the saved fit (`fitsplit.py`), the sweet spot's attack angle is **−7.8° at contact, −16° at −10 ms and −24° at −20 ms**. The barrel is still dropping at 392°/s at contact, and its elevation adds 1.1 m/s to the 21.3. [4]'s motion as placed has +5.6°; OpenBiomechanics has +4.1 ± 5.9°. The fit matches [4]'s elevation curve to 0.28 band units yet comes down on the ball, because the elevation band (8°) cannot see the barrel's path. For a game this is the number that sets the launch angle.
**Fix:** add the attack angle at contact to the held-out checks (OpenBiomechanics, measured), and report it for this fit and session 6's. Say it to the owner beside the passes.

### P2-2. Opposing muscles pulling against each other are everywhere in the fit, and the search is not at its best fit; removing the trunk's second channel treats a symptom

**Where:** docs/body-motion.md:848 ("The body without the trunk's second channel is the cleaner one"), :266 ("the trunk's second muscle should come out"); `fit3d_d34_all.json`.

The saved plan has more opposing pairs on together than the two §4.9 names:
- lead shoulder: internal rotation 70 N·m (from −62 ms) against external rotation 33 (from −204 ms);
- rear shoulder: internal rotation 70 against external rotation 27;
- lead elbow: flexors 112 against extensors 24, held at 0.32 from the start;
- lead wrist: flexion 29 against extension 6;
- the trunk's two channels and the lead arm's raiser and lowerer, which §4.9 already names.

Each channel's excitation is one ramp, so two opposing ramps give the search net-torque shapes that one ramp cannot. With no cost on effort the search uses them.

The "without" table shows the search is not at its best: the trunk's second channel switched off gives **cost 2.98**, while all channels on gives 3.36. With that channel at zero strength the 41-channel search could have reached 2.98, so it stopped at a worse fit than the one available. The two failed cold starts (12.4, 15.5) say the same.

Removing a channel Bae measured (241 N·m) goes against "every designed freedom driven on measured limits". Under D34, the honest fix is in the fit's criterion, not the body.
**Fix:** keep the channel. Add a small effort term (activation² or torque/bound², its weight [E], stated before the run and swept). Or cap simultaneous activation of an opposing pair at the recorded EMG where one exists. Then refit from more than three starts, including starts that are not near one hand estimate. Report whether the opposing pairs, the early arm rises and the slow hands survive.

### P2-3. The owner is told the body "matches every curve the gate reads" without the caveats §4.9 gives elsewhere

**Where:** docs/body-motion.md:255 and :3, against :806 and §4.9 "How settled the searches were".

205 fitted numbers face 15 curves and 5 numbers at contact. The curves are means low-pass filtered at 12 Hz over 250 ms, so each carries roughly 2 × 12 × 0.25 ≈ 6 independent values: about 90–100 independent target values in all (my estimate). The fit has twice as many free numbers as independent target values. Two of three cold starts ended at 3.6× and 4.6× the best cost. §4.9 says "weak evidence by itself"; the owner's paragraph and the status line do not.
**Fix:** one sentence in "For the owner": the match is from the best of three starts, the other two did not reach it, and with 205 numbers matching the curves is expected, so the held-out checks are the real test (three of them fail, plus P2-1).

### P2-4. "29–34 m/s is at the edge of what this body can do" is too kind

**Where:** docs/body-motion.md:893–895, :259.

The 30.6 m/s strongest batter:
- swings with a front leg that does not block (70% of body weight against [4]'s 130%, knee 61° bent);
- turns the hips at 813°/s ([3]: 714 ± 76);
- presses the lead arm 16° past its range onto a soft spring [E] carrying **574 N·m, about 4× the strongest arm muscle's bound**;
- breaks the arm clock by 1 ms;
- has every channel at its sample's mean + 2 SD at once.

What this body reaches with a swing a hitter makes is not known. It is probably well under 30.6, since the stop's stiffness alone was left unrun. The search still gaining 0.3 m/s cuts the other way, but only for the unrealistic swing.
**Fix:** say "this body reaches 68 mph only with a swing no hitter makes; whether it can reach 60–76 mph with a hitter's swing is open". Before Q59 is acted on, run the strongest batter with the gate's ground-force curves and the elbows' and shoulders' stops held, or the stop stiffened.

## P3 findings

- **P3-1. Winter's correction is wrong for the 4th-order variants.** `analysis/research/body/batspeed.py:78` uses `2 ** (1/order)` where the correction for two passes is `2 ** (1/2)`. For order 2 these coincide; for order 4 the cut-off is 14.8 Hz instead of 13.4. The effect is small: 2,649 becomes 2,684°/s, and the angle variant 2,890 (`winter.py`). Fix the formula and rerun.
- **P3-2. Source statements firmer than the notes.**
  - The quote and context from [4] §3.4 check out word for word against the full text (`sim/target/research-dl/4/s.txt:2828–2836`), though note 4 holds only "markers low-pass filtered at 12 Hz". Add the quote to the note.
  - "The bat peaks 2 ± 5 ms before contact" (:736, :825): note 6 says "within ~2 ms", with no SD. Note 6's own sweet-spot difference gives 94.5% at contact (31.1 against 32.9), not 98%.
  - "OpenBiomechanics, the same point": the note does not say where its bat speed is measured.
  - That [3] filtered *through* the impact is an inference; note 3 gives only the filter.
- **P3-3. Stale status lines.**
  - docs/body-motion.md:3 still says "The body does not reach the ball in time under the rules … against 288", which contradicts the bold sentence before it, and the trunk bound is now 260.
  - :9 says "Not built: any three-dimensional body".
  - :10 still says "Phase 1's first job is to show…".
  - §13's gate (:1361) still reads "pelvis 77 ± 6°" while the fit and §4.9 use 64.5 ± 6 (the rate-curve reading). One of them should be stated as the gate.
- **P3-4.** Held-out check 2: 1,064°/s against 1,160 ± 96 sits exactly on one SD's edge, and its arm line is [E]. "Sweeps at the measured rate" (:256) should say "at the edge of the measured spread".
- **P3-5. Floating axis singular at the arm hanging straight.** `engine.float_axis` (engine.py:291) gives zero torque when the arm hangs exactly along the trunk and flips the torque's direction across that point; near it the direction is ill-conditioned while the size stays full. The fitted swing stays at 53–83° from hanging, so nothing here is affected. A fielder's or pitcher's arm at rest would be, so note it for phase 2.

## Answers to the job's questions, briefly

1. §4.8: the source quotes hold (P3-2 aside). The effect reproduces independently (P1-2). The size is neither a floor nor a central estimate: its basis is a parametrisation with a jump, and the best continuous fits give more. The elevation part is unphysical (P1-1). The cross-checks reproduce (Verification). Withdrawing the 22.9 m/s gate follows even at the weakest reading, from [4]'s own exit velocity. The 27–34 band is defensible on direct measurements; replacing the last-40 ms targets is not.
2. §4.5 is identical in `1b62fbf` and `6c5ae8d`, and the engine had no channels before `fa23e7c`, so the limits were fixed before any run.
   - Arithmetic checked: Holzbaur raising 112, lowering 136, elbow flexion 112, wrists 46 and 24; Dodds rotation 70 and 39; Bae's other direction 187.6 × 1.284 = 241, with speed points 0.893 / 0.842 / 0.648 and zero at 11.2 rad/s; Kentel's forearm 17.35 and 18.30, unscaled. `drive.py` equals the spec.
   - Evidence levels are stated honestly: one player for the forearm, five men for the raisers, a half-read wrist SD.
   - Bae's other-direction SD (27.87) is not in note 8 and I could not check it. It should go into the note.
   - The shoulders' reverse turn (Silva) and the wrists' reverse cock (Peleg) are measured and still undriven; §4.5 says so. That is outside this job's list, not a D34 breach.
3. The torque about the floating axis is a pure couple on the two bodies: the same unit axis, the same `wrel`, with power and work taken about it. The shortening speed equals dθ/dt for the angle from hanging (I checked the derivation). Every new channel is in `MODEST`, so the momentum and energy tests see all of them. The spherical-coordinate test and the anatomy test (directions from joint centres) are independent of the code, and the `float_swap` mutation fails the spherical test. Singularity: P3-5.
4. Verified from the JSON and a rerun: +0.81 ms, 21.27 m/s, no residual over 0.72, 56 of 205 on a bound, cost 3.401. The held-out quantities are not in `residuals()`.
   - The split holds: at contact 21.3 = hands 0.5 + azimuth 19.7 + elevation 1.1, with an azimuth rate of 2,151°/s, hands at 3.3 m/s and the bat's end at 26.2.
   - Against [4]'s motion as placed (22.9 = 3.2 + 19.7 + 0.0), the azimuth part is identical. The 1.6 m/s difference is the hands (−2.7) partly offset by the chopping elevation (+1.1).
   - The rest of the gap to the gate is in [4]'s curves themselves, so the explanation stands.
   - On the strongest batter, the opposing pairs and the three failures stated to the owner: P2-2 to P2-4.
5. The owner's numbers agree with §4.8 and §4.9 in both units (62–67 mph = 27.6–29.9, 48 = 21.3, 68 = 30.6, 60–76 = 27–34). Stale lines: P3-3. The index's "Tried and ruled out" covers what failed. `check-docs.py` passes apart from this file's own name in `REVIEW-BODY-3D-7.md`, which this commit resolves.
6. No sim crate is touched (only `analysis/` and `docs/`). The arm channels sit under the −240 ms clock in both the box and `check_rules`, and the trunk's second channel under the legs' rules, as §4.5 says. Nothing on the job's "not to be changed" list was changed. The relax row "the arms from the window's start" is session 6's existing diagnosis, worth 10 ms.

## What I would tell the owner about Q59

The study's hitters were not swinging at 51 mph. That part is settled: the balls they hit came off too hard for a 51 mph bat, and the study smoothed its camera data straight across the moment the ball was hit, which drags the bat's speed down just before contact. How fast they really swung cannot be read back out of the study. Depending on what you assume the bat did in its last few hundredths of a second, my reruns put it anywhere from the low 50s to about 70 mph, with the most likely shapes in the mid-to-high 60s. So "62–67 mph" is a reasonable middle, not a measurement.

I would take a gate of about 60–76 mph (27–34 m/s) at the sweet spot. It rests on hitters measured directly: college and minor-league hitters at 66–70 mph, major leaguers in games at 71.5.

I would not do the second half of option (b) as written, matching the body to "unsmoothed" curves over the last 40 ms. Undone that way, the barrel is chopping down on the ball at about 20°, which no hitter does. Instead, leave the last 40 ms out of what the body is matched to, and check its speed and its upward path through the ball against direct measurements.

Two more things you should know. The current fitted body also comes down on the ball, at about 8°. And when asked only for bat speed, it reaches 68 mph only with a swing no hitter makes (soft front leg, lead arm jammed across the chest). So a hitter-like 60–76 mph swing is not yet shown to be within this body's reach.

## Verification

All under `sim/target/codex-probes/review-body-3d-7/`, output beside each script.

- `bin/job.sh` job 350: `python3 -m unittest analysis/oracle/test_body3d.py`: 29 tests OK in 16 s, none skipped. `python3 analysis/research/body/probe3d.py --check`: worst 8.64e-12 (the fitted swing), PASS. `python3 bin/check-docs.py`: one error, the not-yet-written path of this review.
- `git diff` of §4.5 between `1b62fbf` and `6c5ae8d`: empty.
- `fitsplit.py`:
  - Session 6's plan (`fit3d_d34_arms.json`) on the new code with the new channels off: +4.01 ms, 20.10 m/s, identical to the saved values.
  - Session 7's plan: +0.81 ms, 21.27 m/s.
  - The split at 10 ms steps, and the attack angle (−7.8° at contact).
- `fit3d.py --show fit3d_d34_all.json` (`show.out`): residuals, 56 of 205 on a bound, the plan's opposing pairs.
- `filt.py`, `filt2.py` (uv, numpy and scipy offline): my own Butterworth and filtfilt.
  - Continuous straight, accelerating and plateau rises before contact, and a step at contact.
  - A profile over the rate just before contact.
  - The filter applied to the rotating bat direction instead of the rate.
  - The free-bat cap on the step.
- `edge.py`: `batspeed.unsmooth`'s pieces at ±70 ms (the jump). Elevation and lag misfit with a break against none: 0.48 against 1.30, 0.38 against 1.18.
- `attack.py`: the placed motion's attack angle as placed (+5.6°), with §4.8's three curves undone (−20.3° and −22.2° in two variants), with only the azimuth undone (−1.8°, 27.8 m/s). The 4th-order azimuth-only case failed to assemble a posture and was not used.
- `coll.py` and hand arithmetic:
  - Bat 0.9645 kg, I_cm 0.0383, sweet spot 0.116 m from the centre of mass, effective mass 0.720 kg.
  - For a 36.7 m/s exit at e = 0.50–0.55 the sweet spot needs 24.6–23.1 m/s; the impact takes 12.2 m/s and 1,528°/s off a free bat. Agrees with §4.8.
  - A pure turn about the lead grip gives the sweet spot 0.81 of the bat end's speed (0.82–0.83 with the hands' motion).
- `winter.py`: P3-1.
- Full text of [4] §3.4 and §3.4's contact definition (`sim/target/research-dl/4/s.txt:2828–2836, 2944–2946`): contact is "the instant when the incoming pitch deviated from its initial trajectory"; the text also says 300 Hz is "probably not fast enough to precisely capture the instant" of contact.

## Resolution

Open.
