# Review: swing v3 step 1, the oracle, goldens and spec changes (`work/swing-oracle`, `e1a6c5c..ce4f049`)

Reviewer: rev-swing-oracle (Claude Opus, high effort), 2026-11-24. Wrote none of the change. Job: `REVIEW-SWING-ORACLE.md`. Files read: `analysis/oracle/swing_v3.py`, `test_swing_v3.py`, `mutations_swing_v3.py`, `golden/swing_v3.json`, the `test_oracle.py` diff, `docs/swing-model-v3.md` (diffed against main), PLAN D26, [3]'s and [4]'s notes. Every number below was recomputed with stdlib probes in `sim/target/review-probes/` (listed in Verification). No source, test, spec, fixture, TASKS or observations file was edited.

**Summary.** The oracle implements the spec's text faithfully. The body, the hand anchors, χ, the jet, the joins and the brake all match §4–§8, and every join is C2 on random swings. The tolerances in §11.1 are achievable, and nothing needs a root find at run time. The worker's mutation claims reproduce. The main problem is a cost of the elevation fix that the spec never measures. Contacts 3–10 ms before the planned instant (a fifth of all contact) now meet a bat whose sweet spot is moving 2–10° more steeply downward than today's. The first version kept that under 1°. The owner chose Q45 without this number, so it should be measured, and either put to him or fixed, before step 2 builds the Rust. Below that: the brake's velocities are untested, the aim grid misses the realistic combinations of swing time and speed, and §5.3's stance table describes the old path.

## Findings

### P1

**P1-1. The elevation join changes the attack angle of late contact a lot, and §6.3, §6.5, §12 and §14 never say so.** (`swing_v3.py:419-420, 428`; spec §6.3 l.253 and l.258, §6.5 l.276-284, §12 l.416, §14 Q45)

[4]'s elevation is followed to −30 ms (of [4]'s time). Over the last 26 ms of the sim's swing, δ_E then turns the bat from falling at about 1,300°/s to rising at the jet's +180 to +510°/s. Most of that reversal is packed into the last 8 ms: peak |Ë| is 108,000–165,000°/s² at about −4 ms on the golden aims, against 32,000–54,000°/s² for the jet itself. Over the planned aim range it reaches 291,000°/s² (T 0.130 s, low inside). §6.3 quotes only "the correction's" 53,000–86,000°/s².

The sweet spot's velocity direction, which the collision reads, therefore differs from today's on exactly the contacts §6.5 calls nearly unchanged. Attack angle, v3 (step 1) minus v2, from `attack.py` and `variants_attack.py`:

| contact before t_c | −3 ms | −5 ms | −8 ms | −10 ms | −15 ms | −25 ms |
| :-- | --: | --: | --: | --: | --: | --: |
| step 1, middle / inside-low / outside-high | −1.8 / −2.2 / −1.5° | −4.8 / −5.6 / −3.9° | −7.6 / −9.2 / −5.9° | −8.1 / −10.2 / −5.9° | −11.5 / −14.3 / −8.5° | −16.2 / −19.6 / −12.1° |
| first version (the path the owner saw) | −0.1 / −0.1 / −0.0° | −0.2 / −0.2 / −0.1° | −0.4 / −0.6 / −0.1° | −0.7 / −1.0 / −0.2° | −1.6 / −2.6 / −0.5° | −5.2 / −8.4 / −2.0° |

At −5 ms the middle aim's bat is moving 1.3° *down* where today's moves 3.5° up. Over the same window the attack direction changes about a third less than v2's (at −10 ms: +9.5° against +14.5° toward the opposite field). §6.5 reports this band (8,885 contacts in 600 games, 20% of contact, 3–10 ms early) as "under 1° back, 0.5–3° higher and 0.3–4 mph faster". The largest per-contact change there is a bat 2–10° steeper downward. That will lower launch angles on a fifth of contact. It also undercuts §6.5's "pushed more to the opposite field" and the §12 expectation of "probably within a point".

It also puts a Statcast match that v2 passes at risk: attack angle against attack direction (agent-models §3.6: MLB +0.41° per degree, v2's games +0.44). Within one swing, across ±10 ms of timing, the slope of attack angle on attack direction is 0.47–0.76 for v2, 0.48–0.77 for the first version and 0.74–1.31 for step 1 (2.2–4.1 on the late side alone; `aa_ad.py`). The game-level slope will rise with it.

*Still the owner's choice?* In kind, yes. Step 1 follows [4]'s measured elevation, which the first version did not, and PLAN D26 already notes the fix. In effect, no: for contacts off the planned instant it is a different game from the one Q45 described. The bat's elevation at −25 ms went from +2–3° to +11–14° (§14 does now say this), and the bat's direction of travel on the common 3–10 ms-late contacts went from unchanged to 2–10° steeper (§14 does not say this).

*A cheaper construction that keeps [4]'s shape* (`variants.py`, `variants_attack.py`): over the same window, from −30 ms of [4]'s time, blend [4]'s elevation into **v2's own elevation curve** with a C2 smoothstep, instead of adding a quintic correction. v2's curve meets the jet to all orders at t_c, so the contact join is automatic. The elevation still follows [4] within 0.8° to −30 ms (middle aim), and nothing else changes. The attack-angle change falls to −0.1 to −0.5° at −3 ms, −0.6 to −1.8° at −5 ms and −2.4 to −5.5° at −8 ms, about a quarter to a half of step 1's. Peak sweet-spot speed is 1.03–1.07× (step 1: 1.03–1.05). The cost is about 2° more plane-tilt error (+7.2 / +11.0 / +3.1° against +5.0 / +8.8 / +1.1°, an expected failure either way). Starting the window earlier did not help: moving u_w to −50 or −70 ms only trims the attack-angle change by 1–4° and adds 3–5° of tilt error, and blending from −50 ms gives up [4]'s elevation (5–9° off).

*Fix.* Before step 2, add the sweet spot's velocity direction (attack angle and direction) to `v2_difference`, to §6.5's table and to §12. Add a bound on the within-swing slope of attack angle on attack direction. Then either adopt the blend, or report the attack-angle cost to the owner as a numbered question and have him confirm Q45 on the new numbers.

### P2

**P2-1. Nothing tests the braked bat's velocity.** (`test_swing_v3.py:289-303`; `swing_v3.py:879-885`; §11 case 6, §11.1 "Brakes")

`Braking` checks the stop pose, the zero velocity after the stop, the velocity at t_b and the half-way pose. Patching `sigma` so that velocities are not scaled by σ̇ during the brake (full speed until the stop) passes the whole suite: 0 of 36 tests fail (`my_mutations.py`). The golden `brakes` section stores only stop azimuth, ruling, stop knob and stop axis. A Rust brake that forgets σ̇ would therefore pass step 2. A braked bat that touches the ball (310 such touches in 600 games today; fewer in v3) would then hit with its unbraked speed. Fix: add braked path samples (pose, Ω and V_cm every 5 ms from t_b to t_b + B) to the golden and to §11.1, plus an oracle test that the reported braked velocity equals the finite difference of braked positions (as `test_reported_velocity_is_the_derivative_of_position` does unbraked).

**P2-2. The aim-range grid never combines swing time with bat speed or effort, so §6.3's results and §14 Q48 understate the failures; the fast side (a mid-swing slowdown) is not checked at all.** (`swing_v3.py:681-707`; spec §6.3 l.266, §14 Q48 l.466; `AimRange` l.342-351)

The grid varies T_swing only at effort 0 and bat speed 32 m/s. It varies effort only at T 0.150 s on three aims, and never varies `BatSpeed`. Re-running 900 swings (`grid.py`, `speedfactor.py`) with T 0.130 / 0.150 / 0.174 s, bat speed × effort factor 0.81–1.21 and α ±4°, both hands:
- **Slow swings at today's T 0.150 s** (the case step 4 ships): low-outside reaches 1.11× at factor 1.00, 1.18× at 0.94 (one sd of `BatSpeed` × effort) and 1.25× at 0.90; the middle aim 1.11× at 0.90 and 1.17× at 0.85. The spec says "effort −10%: 1.10–1.18", but low-outside at −10% was never run. At 0.81 the peak is 1.48×. At T 0.130 s the peak is 1.31× at grade 50 and 1.78× at factor 0.81.
- **Fast swings or long swing times** (factor ≥ 1.10 at T 0.150 s on inside aims; factor ≥ 1.0 at T 0.174 s): δ_A must take away up to 100° of [4]'s rate shape. The azimuth rate then falls from 0.565 of the contact rate at −93 ms to 0.36–0.54 at about −50 ms and surges again. These are the "two surges" §6.3 gives as its reason for rejecting other constructions. The check is "A non-decreasing", so it passes.
- **Body and reach**: at 3 sd slow, T 0.130 s, high-inside, the bat passes 1.1 cm into the trunk capsule (0.2 mm clear at T 0.150 s). The hands reach up to 7.9 cm beyond full arm reach (the spec says 5.6 cm). The sampling is fine: 0.2 ms sampling agrees within 2 mm.

Fix: cross T × (`BatSpeed` × effort) × aims in `range_cases`, at least on the zone's corners and the low-outside edge. Add a check that the azimuth rate does not fall below its −93 ms value before contact (one surge). Restate Q48 in plain numbers: at today's single swing time a one-sd slow swing at the low-outside edge peaks 18% above its contact speed, and a fast swing slows in mid-swing. Give that point its own Q-number, since Q48 is decided (CLAUDE.md: every owner decision gets a number).

**P2-3. §5.3's stance table, its tip-speed sentence and §1's "20–26°" describe the first-version path, and on step 1's path the tip-speed argument runs the other way.** (spec l.22, l.166-173; affects the step-5 decision under Q46)

The same four stances on the step-1 path (`stance.py`), middle aim:
- **Today's stance:** the lag is 133 / 57 / 29 / 51° (the table says 50 / 22 / 51) and bottoms out at 8–43° at −17 to −22 ms (the table says 20–26°). Uncocking peaks at 2,124–2,777°/s (the table says 2,550–2,900).
- **P_y −0.10:** uncocking 1,375–1,671°/s (the table says about 1,770).
- **The tip-speed sentence** ("at P_y = −0.25 m the bat tip's speed peaks about 24 ms before contact at about 1.08 times its contact value, which is [3]'s shape … where today's stance gives a peak at contact") is now backwards. Today's stance gives 1.05–1.07× at −15 ms, exactly [3]'s shape. The deeper stance gives 1.14–1.16× at −23 ms, which fails the 1.15 bound at inside-low, and at P_y −0.40 it gives 1.17–1.19× on all three aims.

At 0.55 m deeper the inside-low lag maximum also falls at launch (−150 ms), which fails its time bound. §11's table shows these numbers, but the prose of §5.3, §11 (l.382, "with the batter 0.55 m deeper they pass") and §1 says the opposite. Fix: regenerate §5.3's table and §1's numbers from `swing_v3.py`. State that the deeper stance passes the lag, uncocking and hands-ahead checks but breaks the tip-speed shape (from 0.55 m) and the lag-maximum time at inside-low. Step 5 should know this.

### P3

**P3-1. The arm-vector rate is measured about the world vertical, but [3] measures it about the trunk axis.** (`swing_v3.py:608-615`; spec §5 l.113 says "about the trunk axis", §11 l.370 says "horizontal, about S")

[3] (`3-welch-1995.md` l.15): "Segment rotation is measured about the trunk axis (mid-hips to mid-shoulders)". About t̂(τ), interpolated through the knots (`armrate.py`):
- **Today's stance:** the peak is 1,185 / 1,357 / 1,150°/s at −1 / −1 / −105 ms (the vertical gives about 1,255 at −104 ms).
- **0.55 m deeper:** the peak is 1,545 / 1,686 / 1,427°/s at −31 / −28 / −38 ms (the vertical gives 1,434–2,698).

Both stances still fail, but the expected failure's docstring ("size inside the bound, at −104 ms") is partly an artefact of the axis. Fix: measure about t̂ as [3] does, then rewrite the docstring and §11's row.

**P3-2. The tolerance for χ in §11.1 (1e-6°) conflicts with the 1e-12 m tolerance on the hand anchors.** A χ rounded to 1e-6° moves W by about 5e-9 m. Fix: say that χ_L and χ_93 are stored at full double precision from the golden (308.2709274105184°, 309.4259720198007°), and fix §6.1's "χ_L = 308.2°" (308.27 rounds to 308.3).

**P3-3. The unwrap branch of the contact azimuth is code only.** `swing_v3.py:356-359` puts A_c in (A_L + 180°, A_L + 540°], i.e. (−45°, 315°]. §4.2 says only "continuous from L". The Rust must pick the same branch. A jet at −45° or less (outside even the stress range) would get a 540° sweep. Fix: write the rule into §4.2 or §6.3.

**P3-4. The aim range's "ranges to 1e-6" (§11.1 l.399) assumes the Rust copies the oracle's diagnostic sampling exactly.** The sampling is 1 ms (every 4th sample for clearance, 41 bat points), 41 points over 40 ms for the tilt, 130 polyline steps for the head travel and a Jacobi eigen-solve, and none of it is in the spec. Fix: specify the sampling, or loosen the tolerance and compare only the failure lists.

**P3-5. Some tests restate the construction.** `test_c2_at_every_joint`'s comparisons of `c1A` against `_seg2A` and of `dE` at u_w (l.182-185) are the conditions `poly_fit` imposes. A chain-rule slip in `_baseA`'s second derivative passes them and is caught only by the one-sided contact acceleration and the zone speed test. `GoldenFile` checks only `paths[:3]` and χ, so a stale `brakes`, `jets`, `hands` or `aim_range` section would pass. Several worker mutations are caught only by the speed bound's expected failure flipping to success, and that protection disappears when Q48 is settled. κ = 1 (the elevation fall not scaled to the aim) is caught only by the zone speed test. None of this hides a defect I could find (my own join and brake mutations were all caught except P2-1's), but the mutation list in §11 overstates how independent the checks are. Fix: test the joins by finite differences of `curves()`, not by coefficients (as `joins.py` does). Extend `GoldenFile` to every section.

**P3-6. Small text slips in the spec.**
- §6.3 l.266: of the three grade-50 swings above 1.1×, one is low-*inside* (x −0.45). The text says "low and outside".
- §5 l.114: the azimuth rate peaks at 1.05–1.07× (2,473 / 2,349 = 1.05), not "1.06–1.07".
- §6.3 l.258: the elevation acceleration quoted is the correction's alone. The total is 108,000–165,000°/s² on the golden aims (P1-1).
- §11 l.371: the 0.55 m-deeper tip speed at inside-low is 1.16 and fails, not "at the edge".
- §5.4's "What it says about v3's interior" bullets describe the first version and should say so.
- PLAN D26 does not carry the step-1 cost figures. §14 does.

**P3-7. Two inherited expected failures are about v2, not about the sources or the stance.** The knob speed at inside-low (5.65 m/s) is v2's jet (q = 0.32 at a short reach), and the 6–9 m/s bound is an inference, not a measurement (§15). Labelling it "the jet is v2's" is right, but it is a v2 finding and belongs in v2's open items, not in v3's costs.

## Answers to the job's questions, in brief

1. **(a)** The joins are sound. Value, rate and acceleration are equal at u_93, at u_w and at τ = 0 against v2 for 60 random swings (T 0.125–0.18 s, speed factor 0.85–1.15, effort ±10%, both hands; differences at numerical noise, tip velocity continuous to 1e-5 m/s). The rate has no kink anywhere; the jerk is discontinuous at the joins, which nothing reads. **(b)** The costs recompute exactly: at −25 ms 70.3 / 71.5 / 70.8 mph against 62.8 / 62.2 / 63.5, the bat 12.7 / 14.4 / 11.2° higher and 5.6 / 4.2 / 7.0° back; tilt 37.0 / 46.7 / 27.0° against β 32.0 / 37.9 / 25.8°; the sweet spot peaks at 1.03–1.05×. The speed cost comes from the azimuth's rate shape, not the elevation: every elevation variant leaves the −25 ms speed at 69–77 mph. The attack-angle cost (P1-1) is new and material. **(c)** Appendix E's reading error (±1° of elevation, ±40°/s of rate) sits inside the 1° and 0.17 test tolerances, which is right for checks against [4]. The Rust tolerances do not involve it (both sides use the same table). κ runs 0.83–1.19 over the range and nothing breaks at the edges, apart from the size of the elevation acceleration (P1-1).
2. **The code matches the spec** in the trunk frame (flexion then lean, x_T = t̂ × r̂′), reading 1, W(χ), the misfit, s_T on every anchor (τ_93, u_w, the S knots and the bounds' times), δ_A, δ_E, the hands' sextic, Ω, V_cm, the brake and A_max. The differences are P3-3 (the unwrap branch) and P3-1 (the arm-rate axis). **χ_93 is genuinely the same for every batter.** M(χ) depends only on directions and on lengths proportional to H, both independent of P. A_ref, the template's middle-middle contact azimuth, depends only on α(h = 0.5), β and ψ(x = 0), not on H or `BatSpeed`. It would change only if the template's Statcast fits change, so the Rust should keep its test that recomputes A_ref.
3. **Tests.** Independent: the hand case, the reading signs, the jet against v2's older fixture with finite differences, velocity = derivative, the mirror, Appendix E (middle aim only), and [3]'s and [4]'s bounds. A wrong oracle would simply regenerate: the paths at the other aims and swing times between the knots, the hands' path between W_L, W_93 and the jet, S(τ) between its knots, the brake stops, the aim range and χ (self-consistency only). The worker's mutations reproduce exactly as §11 lists them. Mine: the join (δ_A acceleration not less the shape's, δ_E rate not less the shape's, the rate shape's chain factor, u_w at −20 ms) was caught by 1–4 tests each. The brake (ruling on the unbraked path; travel 3B/4) was caught. The brake's velocity not scaled was **not** caught (P2-1).
4. **The seven expected failures.**
   - Uncocking, hands ahead, lag at contact (high-outside): stance, as claimed. They pass at 0.55 m deeper, but that stance breaks the tip-speed shape and the inside-low lag-maximum time (P2-3).
   - Arm-vector rate: fails either way, but partly because of the axis (P3-1).
   - Plane tilt: a real cost. It also partly predates step 1: the first version was +1.2 / +7.5 / −3.9°.
   - Speed bound: the fixed 168° against a variable contact rate, worse than stated (P2-2).
   - Knob speed: v2's jet (P3-7).
   None of them excuses a construction defect, but two of the stated reasons are incomplete.
5. **Aim range**: not dense enough in the dimensions that matter (P2-2). Location density is fine (7 × 5), and the stress jets cover the baseline's azimuths.
6. **§11.1** is complete except for the braked velocities (P2-1), the unwrap rule (P3-3), χ's precision (P3-2) and the diagnostic sampling (P3-4). The basis matrices from the oracle's floats agree with exact rational arithmetic to 2e-13 relative and the spline moments to 5e-14, so 1e-12 is achievable without being loose. At run time nothing needs a root find or an iteration. The spline needs an interval search (it is continuous, so ties are harmless). The jet needs one `m::atan2`/`m::asin`. χ is stored.
7. **§14 matches PLAN D26.** The new point under Q48 is real and stated plainly, but understated, and it should carry its own number (P2-2). `python3 bin/check-docs.py` is clean on the spec (its only complaints are about the untracked `REVIEW-SWING-ORACLE.md`). The stale lines are in P2-3 and P3-6.

## Verification

All probes are in `sim/target/review-probes/`, stdlib Python, run from the worktree root against `ce4f049`:
- `python3 -m unittest analysis/oracle/test_oracle.py`: 59 tests, OK, 7 expected failures (11.3 s).
- `python3 analysis/oracle/mutations_swing_v3.py`: every mutation is caught as §11 lists (`mutations.out`).
- `my_mutations.py`, `my_mutations2.py`: seven join and brake mutations plus κ = 1, W_93 at the unscaled knot and a linear S. All are caught except the brake's σ̇ (`*.out`).
- `costs.py`: §6.5's table and §6.3's tilt figures reproduced, plus the elevation rate and acceleration over the last 30 ms.
- `attack.py`, `variants_attack.py`, `aa_ad.py`: the attack angle and direction against v2 and the first version, and the within-swing AA–AD slope (P1-1).
- `variants.py`: u_w at −40 / −50 / −60 ms, and blends of E (and of E and W) into v2's from −30 to −60 ms.
- `eacc.py`: |Ë| over the 324 planned swings.
- `grid.py` (900 swings, 54 s), `speedfactor.py`, `clearance.py` (0.2 ms resampling): P2-2.
- `deeper.py`, `stance.py`: every bound at P_y 0.30 and −0.25, and §5.3's table on both paths (P2-3).
- `armrate.py`: the arm rate about the trunk axis (P3-1).
- `joins.py`: the C2 joins on 60 random swings.
- `exact.py`: the basis matrices and spline moments in exact rationals (Q6).
- The golden's `body`, `hands`, `brakes` and `aim_range` sections were checked by hand against §5.3, §6.1, §8.3 and §6.3's figures; all agree except the roundings in P3-2 and P3-6.

## Resolution

Resolved 2026-11-24 by the oracle's author (Opus, `09b7b13`, `3e96af3`, `39d085a`), who reproduced each finding first. No second review: the Coordinator reran `test_oracle.py` (65 tests, 9 expected failures) and the docs check; step 2's Rust is reviewed against the same goldens.

- **P1-1** fixed by the reviewer's construction (the Coordinator's decision; the worker compared four joins and it was best or tied): [4]'s elevation is blended into v2's own elevation curve by a C2 smoothstep from −30 ms of [4]'s time. Attack angle v3 − v2 (middle / inside-low / outside-high): −0.3 / −0.5 / −0.1° at −3 ms, −1.2 / −1.8 / −0.6° at −5 ms, −3.9 / −5.5 / −2.4° at −8 ms, −6.2 / −8.3 / −4.0° at −10 ms. **Still open, stated in spec §6.5, §12 and §14 (Q45):** the change at 8 ms and later is large, the within-swing attack slope is an expected failure (0.80 / 1.03 / 0.59 against v2's 0.59 / 0.71 / 0.47), the path tilt is +7.2 / +11.0 / +3.1° off β, and below the zone the sweet spot reaches 1.10–1.15 × its contact speed on 21 of 105 locations. Step 4 measures these in games before the owner is asked to keep Q45 = a.
- **P2-1** fixed: braked samples in the golden and §11.1, a finite-difference test of the braked velocity; the σ̇ mutation fails.
- **P2-2** fixed: the grid crosses swing time with speed factor on eight aims (522 swings) and checks for a second surge. The finding itself is the owner's **Q49** (spec §14.1): open.
- **P2-3** fixed: §5.3, §1 and §11 regenerated; the deeper stance fixes three checks and breaks three (for step 5, Q46).
- **P3-1** fixed (about the trunk axis: 1,158–1,163°/s at −104 to −106 ms, still an expected failure in time; the worker's figures differ from the reviewer's and it says why in its report: the reviewer's included the frame's own rotation). **P3-2 to P3-6** fixed. **P3-7** relabelled as a v2 finding only.
