# Review: the swing family refitted at the measured stance (`work/swing-stance`, `4cec8af..e9e9cce`)

*Reviewer: rev-swing-stance (Claude Opus 5.5, high effort), 2026-11-24. Job: `REVIEW-SWING-STANCE.md`. Scope: `docs/swing-model-v3.md` §5.3.1, §6.3, §6.7, §11–§15; `analysis/oracle/swing_v3.py`, `test_swing_v3.py`, `mutations_swing_v3.py`, `golden/swing_v3.json`; PLAN D27–D29. No source, test, spec or fixture edited. Probes and outputs: `sim/target/review-probes/stance/`.*

## Summary

The refit does what it says. Recomputed: the oracle tests pass (77 tests, 8 expected failures). The hands and knob speed up once on every swing I tried. The stance reading is the better supported of the two. Every mutation, the worker's and my six, is caught by a test other than the golden file.

The arms at contact (Q1) are the one problem that should be fixed before the Rust. The worker read [4] Table 1 correctly, but not completely.

- **Table 1's only by-location body value makes the arms worse.** Table 1 gives the trunk's turn by location (pelvis plus upper trunk, 53–80°). Applied as the contact body, it raises the share of swings with an arm more than 2 cm past straight from 46% to 53%.
- **A reading built from [4]'s own numbers, plus Savant's distance off the plate, removes most of it.** The reading takes Savant's stance hips after [4]'s 6 cm drift back. With Savant's 0.918 m off the plate added, the share falls from 46% to 18%. It changes nothing that now passes. It also clears the zone's low-inside speed failure (1.099×), which settles Q4.
- **The rest needs a body that bends for low pitches, which no source gives.** It reaches every swing the game makes, and it moves only the clavicle path and the checks, not the bat or the hands.

The first part moves the hands' anchors, so it changes every golden path. It has to land before step 2.

Late contact (Q5) is a failure the oracle already predicts for step 4. The cause is [4]'s elevation curve, not its azimuth-rate peak. It is the same cause as the plane-tilt expected failure. The Rust should hold that curve as data, so a step-4 redesign does not rebuild it.

There is one gap in the build order. Step 3b's deeper contact shifts the fielders' swing-lead cue by 4.4°, and the spec leaves its refit to step 4.

## Findings

### P1-1. The arms at contact: the smallest fix from the sources is a different stance instant and Savant's distance off the plate; it moves the hands' anchors, so it must precede step 2

**Where.** Spec §5.3 (the body table; "every quantity is a function of P and H only"), §5.3.1, §6.7.4 "The arms at contact", §15 "The arms at contact". Oracle: `BODY`, `P_STANCE` (`swing_v3.py:48–50, 71`). Tests: `ArmsAtContact` (`test_swing_v3.py:693`).

**Recomputed.** Probe `reach.py` on the standard run's full swings (every fifth of the family review's `aims.csv`, 13,432 swings, H 1.85 m, contact hands from v2's jet at D29).

| | >2 cm | >5 cm | >10 cm |
| :-- | --: | --: | --: |
| The build | 46.2% | 27.7% | 9.6% |
| Contact depth 0.35 m | 11.9% | — | — |

These are the spec's numbers exactly. By zone cell, the mean overreach is 9–12 cm on low pitches and 4 cm on the outside third at mid-height; it is about zero on high pitches.

**What [4] Table 1 actually gives by location** (read from the thesis PDF, `sim/target/research-dl/4/s.txt` lines 3482–3496):

- the lead knee: the worker is right, 15–18° at every location;
- the pelvis rotation and the upper trunk's rotation relative to it, so the global trunk turn: HI 80°, HO 69°, LI 71°, LO 53°, MID 75°. The model holds 75° for every pitch;
- the lead shoulder's elevation and azimuth (LI 63°, HO 78°);
- the trail elbow;
- the lag, the bat's elevation and azimuth, and the head.

It does **not** give trunk flexion, lateral tilt, pelvis height or obliquity, the trail shoulder, or the lead elbow by location. The lead elbow is given for the middle pitch only (56°). §15's "its trunk's lean by location is not given" is right; §6.7.4's sentence should name what *is* given.

**Applying [4]'s by-location trunk turn makes the arms worse:** 46.2% → 53.3% over 2 cm. Turning less open for outside pitches carries the clavicle away from the plate. So no body taken from [4]'s location data fixes the arms.

**[4]'s own arms disagree with the contact hands even on the middle pitch.**

- [4]'s lead elbow is 56° bent at contact and never straighter than 42° (at +43 ms). On these segments that puts the hands at most 0.543 m from the lead shoulder, against the model's 0.625 m.
- Probe `arm4.py` builds [4]'s lead arm by location from Table 1's shoulder angles, with the elbow at 56°, and takes the best forearm roll. It still lands 8 cm (middle), 15 cm (LI) and 22 cm (LO) from v2's contact hands.

The 2 cm-past-straight check is therefore far looser than [4]. The model's hands are about 0.15 m further from the chest than [4]'s ([4] about 0.4 m, the model 0.57 m). That is §5.3.1's points 4–5 restated: the ball is met 0.60 m in front of the contact hips, where [4]'s arms want 0.45–0.5 m.

**The smallest change from the sources** (probes `reach.py`, `reach_drift.py`, `combo_range.py`; rating range in queue jobs 284–287):

1. **Read Savant's stance hips at the end of [4]'s stance phase, after its 6 cm drift back, not at the set-up before it.**
   - The pelvis knots become fwd L 0.18 / 93 0.23 / c 0.24 m: [4]'s 7 + 11 + 5 cm by phase, plus Appendix E's 1 cm in the last phase. Today they are 0.12 / 0.17 / 0.17.
   - Savant does not say which instant it takes (§5.3.1). None of §5.3.1's six arguments tells 6 cm apart, and its points 4–5 (the arms) favour the nearer body.
   - [6]'s centre of mass moves 0.23 m (cross-check only, D21). Note 11's foot centroid moves 0.26 m (an upper bound).
2. **Use Savant's distance off the plate, 0.918 m.** The game's 0.95 was kept as "no change needed" (§5.3.1, note 11 §3). On outside pitches 3 cm decides the reach.

Results of (1) and (2) together:

- **Arms:** an arm more than 2 cm past straight on 17.6% of the game's swings, against 46.2% (5 cm: 9.4%; 10 cm: 2.0%).
- **Rating range in the zone:** no failure at all. The low-inside corner peaks at 1.099×, so §6.7.4's expected failure passes (P2-3). The hands and knob have no valley, and the arms at launch and −93 ms are within 2 cm (0.9 cm at launch, the template's own residual; 0.7 cm at −93 ms).
- **Golden aims:** the lag at contact is 63 / 60 / 66° (the build's 63 / 59 / 66°). The hands are 0.49–0.51 m ahead of the chest, nearer [4]'s 0.4 m. The uncocking rate passes. The lag maximum is 134–137° at −93 to −95 ms.
- **Long corner:** one-surge 1 and bat-through-body 1 (the build: speed 1, one-surge 2, body 1).

Because the knots move W_L and W_93, this changes every path golden and the template's checks. That is why it has to come before step 2.

3. **The rest is a body that bends for low pitches, [E].** No source gives it, but it is what a real hitter does.
   - The change: forward flexion at contact +15° per 0.185 m below 0.745 m, so 31° at the zone's bottom against [4]'s 15° on the middle pitch.
   - With (1) and (2), the overreach falls to **0.0% of the game's swings over 2 cm (largest 3.1 cm)** and to none in the rating range.
   - It moves only the contact body, so only S(τ) and the checks that read it. The bat and the hands do not read `bodies['c']`: I checked that no bat or hand quantity changes.
   - Cost: at LOW IN the uncocking rate (measured on S) falls to 1,258–1,327°/s, below [3]'s scaled bound of 1,348. That bound would become an expected failure.
   - What would settle it: a source of trunk flexion or pelvis height by pitch height. [6]'s per-swing trunk angles with HitTrax's point of impact could serve as a cross-check under D21.

**What goes wrong if step 2 is built on the present body.** The goldens, W_L, W_93 and the χ check would be rebuilt when the stance instant is revised. Step 3 exports an S(τ) whose arms cannot reach the hands on half the swings, and the client's `FBbReach` (up to 24 cm) has to hide 10–27 cm on low pitches.

**Under D27 I agree this is fixed before the Rust:** (1) and (2) because they move the hands. (3) could technically follow, since it touches only S and the checks, but it belongs in the same oracle pass and must precede step 3.

**Suggested fix.**

- Adopt (1) and (2) as [E] readings, giving their basis in §5.3.1.
- Add (3) as an [E] contact body that follows the aim's height, with §15's "what would settle it".
- Regenerate the goldens. Turn `ArmsAtContact.test_both_arms_reach_the_contact_hands` into a must-pass test over the rating range.
- Keep a reported check of the lead shoulder-to-hands distance against [4]'s 56° (0.543 m at 1.85 m). The middle pitch comes to 0.563 m under (1) and (2).

### P2-1. Late contact is a failure the oracle already predicts for step 4; its cause is [4]'s elevation fall, not the rate peak; the Rust must hold that curve as data

**Where.** Spec §6.3 (E(u)), §6.7.6, §12 "Bat speed by contact depth", §15 "Late contact's speed".

**What the sources say.**

- [4]'s angular speed ratio is confirmed: from §5.4's table, √((Ȧ cos E)² + Ė²) is 2,276°/s at −25 ms against 1,927 at contact, which is 1.18×.
- Its 1.18× comes mostly from the elevation: Ė about −1,000°/s at −25 ms, −120°/s at contact. That is a level batting-practice swing bottoming at the ball.
- The jet's bat is rising at contact (+331°/s, MLB's 8° attack angle).
- Both lab sources filtered through the impact: [3] 13.3 Hz, [4] 12 Hz Butterworth (`s.txt` line ~2808). That can manufacture a speed peak before impact.
- [6] (cross-check only) has the bat at 98% of its maximum at contact, peaking within about 2 ms.
- Savant's speed rises from deep contact to 6–12 in out front: 68.4 / 70.7 / 71.8 / 72.9 / 73.1 / 73.4 / 72.6 mph (note 9).

**Probe `mono_probe.py`.** I replaced [4]'s azimuth rate after −30 ms with a monotone rise into its contact value, treating the −10 ms peak as a filter artifact. The late-contact gain did **not** go away. The sweet spot's peak rose from 1.086× to 1.097× (middle) and from 1.108 to 1.117× (inside-low), and the speed at −15 ms moved by under 1.2 mph. The sweep is fixed, so a lower rate near contact must be made up earlier.

The cause is therefore the elevation's steep fall up to −30 ms plus the hands. It is the same cause as §11's plane-tilt expected failure (the sweet spot 5.5–8.8° steeper than Statcast's β over the last 40 ms). Statcast's tilt and Savant's speed by depth agree with each other and against [4]'s batting-practice curve. No D21 source gives an uppercut swing's elevation curve.

**What goes wrong.** Step 4 will fail the Tier M check: the oracle gives +4.8 mph at −15 ms where MLB has −3.4. If the elevation's last 60 ms were code (u_w at −30 ms, the blend into v2's elevation, [4]'s table), the redesign would rebuild the Rust.

**Suggested fix.**

1. Step 2 holds [4]'s two curves (knots, moments), the window start u_w and the blend's form as data, with the basis rederived by test as §9 already says for the matrices. A step-4 change of the interior is then a table and golden change.
2. The spec states step 4's measurement and its trigger now, so the Rust is not built twice:
   - **Measure** the sweet spot's speed by plate-relative contact depth relative to the batter's mean, in Savant's bins: ≤ −12, −12…−6, −6…−2, ±2, 2…6, 6…12, ≥ 12 in. Do not bin by milliseconds before t_c: 12 in deep is about 8–10 ms of ball travel, not 15.
   - **Measure** the path tilt by depth.
   - **Trigger:** if the −12…−6 in and ≤ −12 in bins are at or above the ±2 in bin (MLB: −2.2 and −4.5 mph), redesign the elevation interior. Its candidate is a fall that bottoms before contact and is laid toward Statcast's plane over the last 40 ms.
   - §6.7.5's allowance (up to a fifth of an easier swing's extra time shown as lateness) moves the deep bins by only about 2 mph. It cannot close the gap alone.
3. The Coordinator should tell the owner that Q45's check is now a predicted failure, not an open one. Settling the curve before step 4 from [6]'s bat path as a cross-check is an option to offer.

### P2-2. Step 3b moves the fielders' swing-lead cue by 4.4°, and its refit is assigned only to step 4

**Where.** Spec §12, rows "Where the batter stands" and "The fielders' swing-lead cue"; §13 step 3b. Code: `game.rs:415` `swing_lead = t_plate − t_start`; `live.rs:935–937` `LEAD_FACE_*`.

**What goes wrong.**

- In step 3b `contact_y0` moves from 0.35 to 0.49 m with v2 still the swing. Every planned contact, and so every swing's start, comes about 3.6 ms earlier against a fixed `t_plate`.
- The lead therefore rises 3.6 ms, and the fielders' belief moves by −1,213.3°/s × 0.0036 s ≈ −4.4°. That exceeds the fit's own residual of 3.1°.
- Step 3b's ledger row would then mix a fielding bias into the batter's move, and step 4 would refit a cue that was already off.

**Suggested fix.** Add the cue to step 3b. Either refit `LEAD_FACE_*` there, or (§12's own candidate) define the lead against an instant that moves with neither the aim nor the contact depth. List it in step 3b's inventory and ledger note.

### P2-3. The must-pass speed bound has an expected failure inside the zone: decide it (P1-1's change removes it)

**Where.** Spec §6.7.4, §11's speed row; `RatingRange.test_speed_bound_at_the_zones_low_inside_corner` and the ≤ 1.11 allowance in `test_every_rating_and_effort_in_the_zone`.

**Assessment.**

- The 1.10 bound is [E]. It sits between [3]'s measured tip peak (1.07× at −15 ms) and [4]'s angular speed (1.18–1.19× at −25 ms, mostly the elevation; P2-1). [6] (cross-check) has about 1.02×.
- Failing it by 0.001–0.009 at one corner shows neither the bound nor the swing wrong. The test meanwhile allows 1.11 there, a bound set to the outcome.
- With P1-1's (1) and (2) the corner peaks at 1.099× and the whole zone passes 1.10. The decision then becomes "keep 1.10 must-pass everywhere in the zone".
- If P1-1 is not adopted, record a decision instead: a 1.11 in-zone bound with that reason, or the corner as an accepted failure. Do not leave a must-pass bound with a silent exception.
- The long corner (λ 1.29, +4.1 sd; `QuickHands` is not drawn in today's games) can stay reported and not required.

### P3-1. Shape by location the checks do not see: the hands' peak time, the lag on high-inside pitches, the arm vector on low pitches

**Where.** Spec §6.7.4 "The hands", §11 bounds (golden aims only). Probes `hands_grid.py` (280 swings, finite differences of positions) and `hi_lag.py`.

**Findings.**

- **The hands' peak time.** "A peak of 10–13.5 m/s 37–50 ms before contact" holds for the rating range's inner aims. Across the plate's whole width (x ±0.25 m, z 0.50–0.99 m) the hands peak from −60 ms (low-inside, easing 30–55% into contact) to −15 ms (low-outside, only 3–6% above contact). [4] Table 2 has the lead arm's turning peak at −41 to −49 ms at every location, earliest at LOW OUT (−49), where the model's hands peak latest. There is still no valley anywhere.
- **The lag on high-inside pitches.** It holds at 135° and rises again to 140–141° at −45 to −50 ms before uncocking: at HI 139° at −40 ms against [4]'s middle 115°. The lag-maximum check would fail there in size and time (141.0° at −45 ms), but it is run only on the golden aims.
- **The arm vector on low pitches.** On LO and on low-middle aims it peaks at −113 to −121 ms.

**Suggested fix.** Report the hands' peak time and the lag maximum on [4]'s five locations (`LOCATION_AIMS`) against Table 2's times, as reported comparisons. Note in §6.7.4 that the 37–50 ms is the inner zone's.

### P3-2. Step 2's golden tests must set `contact_y0` as well as `pivot_y`

**Where.** Spec §13 step 2 ("its golden tests set `pivot_y` … It reads the contact depth as the game does").

**What goes wrong.** The batter's reference contact pose X_ref (the fade, §6.7.3) is his middle-middle jet at `contact_y0` (`MIDDLE_MIDDLE`, `ref_jet`). Until step 3b the game's value is 0.35 while the goldens are at 0.49. A Rust test that sets only `pivot_y` gives every short swing (λ < 1) launch hands (1 − λ)·0.14 m off in y. The path goldens would catch it, but the spec should say so.

**Suggested fix.** "Its golden tests set `pivot_y` to −0.276 m and `contact_y0` to 0.49 m (both stored in `meta`)."

### P3-3. Step 3b: the eye point's instant, and the picture between 3b and 4

**Where.** Spec §12 "Where the batter stands", §13 step 3b.

**Findings.**

- **The eye point.** `eye_point` is placed "from the stance spot". The pelvis is 0.12 m forward of the stance by the swing's launch (0.18 m under P1-1), which is about when the decision is taken. Say which instant the eye point is for and why ([4] gives the head only as rotation).
- **The picture between 3b and 4.** v2's swing does not read the stance spot, so the client draws a figure 0.58 m deeper holding a bat placed relative to the ball. Before contact, v2's launch knob is near the hips' y, not behind the rear shoulder.

**Suggested fix.** Either keep the client's standing spot at today's place until step 4, or land the client's move with step 4. Say which in §13.

### P3-4. Small source readings in §5.3 and §15

**Findings.**

- **The hip height.** §5.3's 0.48·H at contact uses [3]'s toe-to-toe 0.85 m and an assumed ankle gap of 0.65 m. [4] gives its own stride: ankle centres 42% of height (`s.txt` 3152, Appendix C's code), 0.78 m at 1.85 m, which gives about 0.47·H. The difference is 1.5 cm, so it is not material, but cite [4]'s figure.
- **The stride in §15.** "Note 11's feet move 0.48 m further apart from the stance to stage 2" misreads note 11. The 0.48 m (19 in) is the front foot's own move; the back foot also moves 4 in, so the gap grows about 0.38 m. Note also that [4]'s ankle gap at foot-down (0.78 m) is about Savant's *stance* separation (0.75 m), so Savant's stage figures are not on [4]'s footing. Use them only with that caveat.

### P3-5. Tests whose bounds were set to the outcome, and what the oracle cannot tell apart

**Bounds set to the outcome:**

- `RatingRange`: the low-inside corner allowed ≤ 1.11 (P2-3).
- `ArmsAtContact.test_the_arms_reach_at_the_top_of_the_zone`: ≤ 0.03 m where `ARM_SLACK` is 0.02.
- `Family.test_travel_across_batters_as_MLBs`: length ÷ speed within 0.064–0.071 s. MLB's band is 68–70 ms; the model's low end is 64.1, just inside the test's 64.
- The arms at −93 ms on long swings largely restate `LAM_W_MAX` (chosen from the template's 7.4 cm slack so they pass).

Each should say so in its comment, or carry a source.

**What the oracle cannot tell apart.** It cannot distinguish "the ball met 0.14 m further out" from "the batter 0.14 m deeper": the swing is a rigid translation in y. My mutation "contact depth on the body" was caught only because the test module's golden aims are import-time constants at 0.49. The absolute stance and depth are guarded only by `meta` in the golden file, and their plate-relative consequences (the box, the eye, HBP, the decision clock) by nothing until step 3b. That is acceptable, but say it in §11.

### P3-6. Stale lines

**Tests** (`test_swing_v3.py`):

- `:427`, `:620`: "contact depth 0.35"; the goldens are at 0.49.
- `:430`: −94 / −94 / −95 ms; the build gives −89 / −89 / −90.
- `:478`: 0.46 / 0.46 / 0.48 m; the build gives 0.57 / 0.56 / 0.59.
- `:483`: "middle 54 …"; the build gives 63 / 79 / 66 / 59 / 50°.
- `:490`: the plane-tilt docstring's 9.6 / 7.2 / 10.9° are the 0.50·H values; the build gives 7.7 / 5.5 / 8.8°.
- `:541–542`: slope 0.067, 64.8 ms, 9.75–10.13 ft, 0.023 ft/mph; the spec has 0.060, 64.1, 9.70–9.90 and 0.012.

**Oracle** (`swing_v3.py`):

- `:294` `T_MEAN … [owner's choice pending]`: withdrawn (§14.1).
- `:1020`: `ARM_SLACK` comment "1.1 cm"; it is 0.9 cm.

**Spec:**

- §15, first bullet: "within 0.8° to −30 ms"; §6.3 now says 1.0°.
- §15: "13 ms shorter on the inside edge" against §6.7.3's 11 ms. The oracle gives 10.8 ms at x −0.45 m.
- §12, decision row: "inside ones 20 ms later". The oracle's earliest aim gives 18 ms (0.152 s at x −0.45, z 1.14).
- §12, cue row: "inside pitches … start 13 ms later" against 11 ms.
- §14 Q46: "§13 step 5"; it is now step 3b (in the superseded paragraph).

**`python3 bin/check-docs.py`** reports only the job file `REVIEW-SWING-STANCE.md`: a bare file name at line 3, and this review's path, which exists once this file is committed.

## Verification

- **Oracle tests:** `python3 -m unittest analysis/oracle/test_oracle.py` (queue job 280, 84 s): 77 tests, OK, 8 expected failures.
- **The worker's mutation logs:** jobs 275 and 279 (`bin/job.sh log`). Every mutation is caught by a non-golden test, except "δ_A acceleration not less the shape" (the equivalent one, as the spec says). Some make an expected failure pass, which is reported, not counted.
- **My mutations** (`my_mutations.py`, job 281). Each is caught by at least one non-golden test.

  | Mutation | Tests caught |
  | :-- | --: |
  | Stance depth from the rear apex (P_y −0.708) | 9 |
  | Stance depth from the plate's middle (−0.492) | 8 |
  | The y sign flipped (+0.276) | 13 |
  | The contact depth applied to the body (P_y −0.416, y0 0.35) | 4 (Family travel, lag maximum; see P3-5) |
  | The body at 0.50·H | 7 |
  | The reference pose at y0 0.35, aims at 0.49 | 3 (RatingRange) |

- **The arms at contact** (`reach.py`, `reach2.out`, `reach3.out`, `reach_drift.out`): 46.2 / 27.7 / 9.6% recomputed exactly. Also run: [4]'s by-location trunk turn, Savant's P_x, the pelvis travel at 0.17–0.26 m, flexion, lean and lowering variants. [4]'s lead arm by location is in `arm4.py` / `arm4.out`.
- **Body variants** on the golden, [4]-location and low or outside aims (`variants.py`, `var_*.out`). The rating range and the [4]-location bounds (`combo_range.py`, jobs 282–287, `range_drift.out`) were run for: the build, travel 0.26, travel with P_x and flexion, the drift reading, the drift reading with P_x, with flexion, and with both. Changing only the contact body leaves the bat and hands identical, as the construction says (`bodies['c']` feeds only S and the checks).
- **The hands** (`hands_grid.py`): 280 swings over x −0.25…0.25 and z 0.50…0.99 at H 1.85, with eight rating and effort cases. Speeds come from central differences of `Swing.hands` and the knob's position, not from the oracle's derivatives. No valley in the hands or the knob on any swing. Peak 9.5–16.5 m/s between −60 and −15 ms; contact 6.8–12.2 m/s.
- **Across contact** (`across_contact.py`): no second surge. The sweet spot falls monotonically after its pre-contact peak through contact and v2's after-contact bat.
- **Late contact** (`mono_probe.py`, `mono_orig.out`, `mono_mono.out`): the build's −25 / −15 / −10 ms speeds are reproduced (76.4 / 76.4 / 74.3 mph middle; v2 today 62.8 / 68.0 / 69.9). A monotone rate into contact does not lower them.
- **Spec numbers checked** (`numbers.py`): S at the three knots, W_L and W_93, T_t over the zone and the aim range, the high-inside lag curve (`hi_lag.py`).
- **The box geometry** of §5.3.1 point 3 rechecked by hand: the box runs y −0.698 to 1.130 m; the rear foot is at −0.643 (stance reading) or −0.813 (contact reading).
- **[4] Table 1–3** read from the thesis text (`sim/target/research-dl/4/s.txt`); its filter at line ~2808; its stride at 3152.
- `python3 bin/check-docs.py`: see P3-6.

## Resolution

Handled 2026-11-24/25 by the refit's author (Opus, `work/swing-stance` up to `0a16009`), who checked each finding first; the Coordinator reran `test_oracle.py` on the merged tree (77 tests, 8 expected failures). No second review of these fixes: step 2's Rust gets an Opus review against the same goldens, and that review is asked to look at the contact body's bend, which is new and [E].

- **P1-1** fixed, all three parts, with two further terms in the contact body the worker found necessary (the review's parts 1 and 2 alone left 18% of the game's swings with an arm more than 2 cm past straight): Savant's stance hips read after [4]'s drift back, the distance off the plate 0.918 m, and a contact body that bends for low pitches [E]. Both arms reaching the contact hands is a must-pass test.
- **P2-1**: the curves, the window start and the blend are data for step 2, and the spec states step 4's measurement in Savant's depth bins and its trigger. The owner then decided the last 40 ms should follow MLB's tracking (Q52 = a, PLAN D30). **Not built, open:** no join of the lab's swing into Statcast's passed the must-pass checks (the bat eases and surges on 99 of 108 swings in the rating range); the probe is `analysis/research/late_swing_d30.py`, and the spec (§6.7.6) says what blocks it. The worker also notes the review's "+4.8 mph at −15 ms" overstated the gap (Savant's deep bins sit 7–13 ms before contact: the present path is +1.1 and +3.4 mph in them, MLB −2.3 and −4.7). Back to the owner as Q53.
- **P2-2** fixed in the spec: the fielders' swing-lead cue is in step 3b's inventory.
- **P2-3** fixed: 1.10 is must-pass everywhere in the zone; the 1.11 allowance is gone.
- **P3-1 to P3-6** fixed.
