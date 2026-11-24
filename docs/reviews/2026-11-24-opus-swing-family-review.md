# Review: the swing family for PLAN D27 (`work/swing-family`, 2624c8c..9dddcd3)

Reviewer: Opus (independent; wrote none of the change), 2026-11-24. Scope: [swing-model-v3](../swing-model-v3.md) §4.3, §6.1, §6.3, §6.7, §7, §8, §10–§15; `analysis/oracle/swing_v3.py` (`Template`, `Swing`), `analysis/oracle/test_swing_v3.py`, `analysis/oracle/mutations_swing_v3.py`, `analysis/oracle/golden/swing_v3.json`. No sim source changed on the branch. Every number below was recomputed with my own probes under `sim/target/review-probes/family/` (stdlib Python on the oracle's `Swing` positions, my own finite differences and checks; one out-of-tree Rust probe for the game's swing aims). Nothing outside that directory and this file was edited.

**Verdict.** The family does what D27 asks for the bat in the zone: on my own grid (700 in-zone swings, heights 1.70/1.85/2.00 m, both hands, every rating and effort at ±3 sd alone and in combination, the long corner apart) the sweet spot peaks at 1.000–1.088× its contact speed, never earlier than the bound, the azimuth rate has exactly one surge, and the bat clears the body. The out-of-zone exceptions are the template's geometry, and almost no game swing goes there. But step 2 should not start yet, for two reasons. **The owner's question in §6.7.7 is mis-posed**: 0.170 s and 0.150 s measure the same swing from different starting instants, so option (a) would move every batter's decision 20 ms for a change of definition (P1-1). **The bat's handle speeds up twice in every swing**: the knob reaches 8 m/s 110 ms before contact, slows to under 3 m/s and speeds up again. This is the opposite of what D27 asks, and no check looks for it (P1-2). The long swings' launch hands also sit out of the lead arm's reach (P2-1). Several statements hold for the grade-50 batter at effort 0 only (P2-2, P2-3).

## Findings

### P1

**P1-1. The owner's choice in §6.7.7 compares two different start instants; under one definition the measured swing already takes about 0.150 s.** (§4.3, §6.7.7, §12 first row, §13 step 2.)
- *What the spec says.* §4.3 maps [4]'s −170 ms onto the swing's start, "[4]'s bat turns at 240°/s there, 10% of its peak, the same threshold v2 used to define its start". §6.7.7 then says today's swing takes 0.150 s where the measured one takes 0.170 s, and recommends (a): decide 20 ms earlier.
- *What v2's start actually is.* v2's start is **the hands** at 10% of their peak rate (agent-models §3.6, `BatterAttrs::t_swing`'s doc comment, `analysis/oracle/swing_oracle.py` `t_start`). That is the instant v2's 0.150 s was fitted to Blast's 0.149 s time to contact, whose start of downswing is not defined. v2's **bat** reaches 10% of its peak rate only 108 ms before contact.
- *What v3's start is.* The template starts **from rest** at −T, because §4.3 drops [4]'s 240°/s. Measured by the two thresholds (`start_defs.py`):

  | Threshold | v3 template, middle aim (T 0.170 s) | Inside-low / outside-high | Today's v2 |
  | :-- | --: | --: | --: |
  | bat's angular speed at 10% of its peak | −150.0 ms | −143 / −158 ms | −108 ms |
  | hands at 10% of their peak (v2's definition) | −160 ms | — | −150 ms |

  [4]'s own rate graph passes 10% at −170 ms because its bat is already turning there.
- *So the choice is mis-posed.* By the bat threshold §4.3 cites, the recommended swing takes 0.150 s. By v2's own definition it takes 0.160 s. Option (a)'s 20 ms comes from hanging the unsourced 0.05 s latency on a from-rest instant that does not exist in v2. Its real cost (every batter sees 20 ms less of the pitch, and the perception calibration is redone) is paid for a change of definition. Option (b), shortening the latency to 0.03 s, is not "adjusting a delay to keep an old calibration". It keeps the decision tied to the same event as today. Option (c) has no physical reason left: the template's wrap is right at T̄ = 0.170 s from rest.
- *If step 2 is built on it.* `BatterAttrs::t_swing` (0.150 s, which feeds `pa.rs`'s commit and v2) and v3's T_b (0.170 s at grade 50) are two attributes with no mapping between them in the spec. Behind the switch, the commit either moves for v2 too, which changes the digests, or uses an undefined quantity.
- *Fix.* Restate §6.7.7 before the owner decides, and state the threshold for every number:
  - Keep T̄ = 0.170 s from rest for the template. The measured wrap is right there.
  - Define the decision clock from the same event as today: the hands at 10%, 0.160 s for the grade-50 middle swing. The commit then moves about 10 ms, not 20. Or say plainly that the latency is measured to the first motion and is 0.04 s.
  - Name the attribute and its mapping (T_b(v3) = `t_swing` × 0.170/0.150, or a new field) in §7 and §13 step 2.
  - Correct §4.3's "the same threshold".

**P1-2. The hands and the knob speed up twice on every swing: fast at −110 ms, nearly stopped at −70 to −50 ms, fast again at contact.** (§6.3's hands' sextic, inherited from step 1 and scaled into every batter; §6.7.4 does not check it.)
- *Measured* (`hands_speed.out`, grade 50; the mid-wrists W, m/s, at −150 / −110 / −93 / −70 / −50 / −30 / −10 / 0 ms):

  | Pitch | W's speed | Knob's speed |
  | :-- | :-- | :-- |
  | middle | 3.1 / 7.4 / 5.7 / 3.3 / 2.3 / 4.4 / 8.8 / 9.4 | 8.1 at −110, 2.7 at −70, 6.5 at contact |
  | in-high | 2.1 / 7.8 / 5.4 / 0.9 / 2.7 / 4.5 / 7.9 / 8.4 | 8.5 at −110, 2.6 at −70, then 6.0, 4.75, 5.4: three surges |

  Before their contact peak the hands lose 40–90% of their early speed on every aim and rating (`early_phase.out`). Today's v2 hands rise once, to 7.7 m/s at −30 ms, and ease to 6.5 m/s at contact. Step 1's merged path has the same stall (8.6 → 2.3 → 9.4 m/s).
- *Why it matters.* D27 says no swing "eases and surges". The bat's handle does, and the client draws the hands at W: the figure's hands would visibly stop and then lunge. It is also the likely cause of an expected failure that §11 blames on the stance and the sources: the arm vector's turning rate peaks at −116 to −126 ms against [3]'s −65 ms. [3]'s arms turn fastest exactly where v3's hands stall. The sweet spot's speed hides the stall because the bat's rotation carries it.
- *If step 2 is built on it.* The hands' basis matrix and its goldens are ported to Rust and would have to be redone.
- *Fix.* Give the hands' curve a rate condition and test it:
  - Candidates: the hands' velocity at u_93 from [4]'s arm angles' rates, or a hands' speed shape as a share of their contact speed (as the azimuth's).
  - Add to §6.7.4 a one-surge check on the knob's speed (relative to its value at the time, see P3-1).
  - Re-run the arm-vector bound. If it then passes in time, move it out of the stance findings.

### P2

**P2-1. Long swings launch with the hands out of the lead arm's reach, and the reach check cannot see it.** (§6.7.3 scales W about the contact pose; `analysis/oracle/swing_v3.py:938` checks |W − S| ≤ (0.112 + 0.186 + 0.146)·H from the clavicle.)
- *Measured.* The lead shoulder to the hands at launch, against a straight lead arm (0.614 m at H 1.85), from `anatomy.out` and `range_summary.out`:

  | Batter | λ | Lead arm at launch |
  | :-- | --: | :-- |
  | template | 1.000 | 1.1 cm over (§6.1 reports it) |
  | fast bat +3 sd | 1.117 | 6.2 cm over |
  | slow hands +3 sd | 1.158 | 8.0 cm over (7.3 / 8.7 cm at 1.70 / 2.00 m) |
  | slow bat + slow hands | — | 2.0 cm over |
  | long corner | 1.293 | 14.5 cm over |

  280 of my 700 in-zone swings put the lead arm more than 2 cm past straight at launch, all with λ > 1.
- *Why the check misses it.* Measured from the clavicle the same hands are at most 1.013× full reach, and only on the corner. The clavicle check allows the half-shoulder toward the rear arm, but the lead hand is on the bat too.
- *Elbows on short swings.* Compact swings are fine at launch (lead elbow 39–57°, rear 132° against [4]'s 72° and 128°). On long ones the rear elbow opens to 117° (slow hands) and 103° (corner).
- *What it means.* Scaling the hands' displacement by λ is not anatomical: the hands are pushed straight back along the template's launch–contact chord, past the arms. Nothing in a game reads the arms, but the client's reach solver (`FBbReach`, up to 24 cm) would carry 8–15 cm for a legal 80-grade batter. That does not meet "a slow swing as realistic as a fast one".
- *Fix.* Add a two-arm reach check (each shoulder, at L and at 93; the shoulders are known exactly at the knots) to §6.7.4's must-pass list. Then do one of:
  - put λ's extra travel into A and E and scale W's displacement less (λ_W < λ), and re-check the speed bound (§6.7.2's ii-a shows that the scaling's split moves it);
  - build each batter's W_L by §6.1's misfit with his own launch bat (χ is then per batter, or tabulated in λ).

**P2-2. Quick and hard swings keep step 1's late-contact change; §6.7.6, §11 and §12 report the grade-50 batter at effort 0 only.**
- *Measured* (`late_vs_today.out`, `slope_family.out`; v3 against today's v2 at 0.150 s):

  | Swing (T) | Attack angle v3 − v2 at −10 ms (middle / inside-low / outside-high) | At −25 ms | Azimuth / elevation at −25 ms, middle |
  | :-- | :-- | :-- | :-- |
  | grade 50 (0.170 s) | −4.0 / −6.4 / −1.9° | — | −4.3° / +8.6° |
  | hard +3 sd (0.150 s) | −6.1 / −9.3 / −3.4° | −15.8 / −19.4 / −11.9° | −11.5° / +12.8° |
  | quick hands +3 sd (0.147 s) | −5.8 / −9.1 / −3.0° | −15.1 / −18.7 / −11.4° | — |

- *The attack-slope bound.* §6.7.6 says the middle pitch now passes the ±0.15 bound. That holds at grade 50 only: +0.139 there, +0.155 hard, +0.212 quick hands, and +0.37 on quick hands at inside-low.
- *Plane tilt.* Easy swings are 11–12° off β, against grade 50's 8°.
- *Why it matters.* Effort has σ 4.5%, so these swings are common, and §12's "late contact moves toward MLB's" holds only for the average swing.
- *Fix.* Report §6.7.6, §11's expected failures and §12's expectations over the family (at least ±2 sd of effort and the BatSpeed spread the games use), not one batter.

**P2-3. §12 omits the fielders' swing-lead cue, which was fitted on the fixed 0.150 s swing.**
- *What it is.* `game.rs` passes the lead L = t_plate − t_start into the play. The fielders' belief is face ≈ 184.10° − 1,213.3°/s × L (`sim/crates/bb_game/src/live.rs:935`, agent-models §5.10: "refit when the swing model changes").
- *What v3 does to it.*
  - At grade 50 the lead's mean moves by +20 ms, which is 24° of face if the constants are not refit.
  - T now varies with the aim: across the plate 0.157–0.184 s. Inside pitches, which are pulled, start 13 ms later, so their lead reads as late, toward the opposite field: the wrong sign.
  - T varies with effort, 7.6 ms per sd, which carries nothing about spray.
  - These components are of the order of the lead's whole present spread (sd 8.2 ms).
- *Fix.* Add a row to §12, and refit `LEAD_FACE_*` in step 4 with the cue's value re-measured. Consider redefining the lead against an instant that does not move with the aim and the effort, for example when the bat passes a fixed azimuth.

**P2-4. The re-plan's held T and the effort noise need one sentence each, or step 2 can break invariant 3.**
- *What the spec says.* §6.7.5 holds T across re-plans.
- *How `pa.rs` re-plans.* It calls the planner (`plan_swing`, decision code in `bb_agents`) and then `apply_placement` with the executed placement.
- *The trap.* If the held T is read back from the executed swing (t_c − t_start(), which carries the effort draw) and passed to `plan_swing`, decision code reads an executed quantity.
- *Fix.* Say that the plan carries the intended T, T_t·T_b/T̄ at effort 0 (fixed at the commit), and that `apply_placement` divides it by (1 + e) every time, as it multiplies the rates.
- *Also say plainly that the effort is execution noise.* It is drawn by `motor::execute_swing` on `SwingEffort`, σ 4.5%, truncated at 4σ; the batter does not choose it. The model therefore treats each draw as a whole-swing tempo the body times for itself. The alternative, the start at the intended T and the swing late, would make a swing 7.3 ms late per sd of effort, more than the game's whole contact-timing spread (sd 5.6 ms), and would change games a lot.
- *The MLB evidence.* §6.7.5's "at most a fifth" is an upper bound. The within-batter speed–depth regression also contains the reverse cause, deep contact meeting a slower bat, which note 10 itself describes.

**P2-5. The worker's "effort changes the travel, not the time" mutation does not put effort in the travel** (`analysis/oracle/mutations_swing_v3.py:128`).
- *What it does.* The patched `travel` is the original formula, and effort is not one of its arguments. The mutation only removes effort from the time and lets the contact correction absorb the rate mismatch.
- *Results.* It is caught (6 tests), but under the wrong label. My own version, effort multiplying λ at a fixed time with the launch moving with effort, is caught by 5 tests: the effort invariance, the within-batter travel, Appendix E's elevation, the launch-per-pitch test and the golden file.
- *Notable.* RatingRange does not catch it. A swing that is longer when harder is "realistic" by every bound. Only the within-batter definition (note 10 §2, the chain) rules it out, so those tests carry real weight.
- *Fix.* Relabel the worker's mutation and add the real one.

### P3

**P3-1. The "one surge" speed allowance has 0.1% headroom, and its measure hides early stalls.**
- *Headroom.* In my in-zone grid the largest sweet-spot dip is 1.89% (quick hands, the zone's bottom, 9 ms before contact; 1.5–1.7% on compact swings at 1.70 and 2.00 m). §6.7.4 justifies its 2% with "ripples by up to 1.5%".
- *Early stalls.* The dip is measured against the peak speed. A stall early in the swing therefore reads small. The long corner on high pitches loses 23–27% of its speed at the time, about 120 ms before contact, which §6.7.4 reports as a "4% dip".
- *Fix.* Measure the dip against the running maximum's own value, with an absolute floor (for example 0.1 m/s), and state the real in-zone maximum.

**P3-2. The out-of-zone exceptions are the template's, and almost no game swing reaches them; say so with the numbers.**
- *Where they fail.* My grid (1,015 out-of-zone swings) fails "peak > 1.10" only at the inside edge low (x_in ≥ +0.35 m at or below the zone's bottom, x_in +0.20 just below it) and at the far inside edge up to a quarter of the zone. Grade 50 alone fails 26 of 174, to 1.164× at 1.85 m and 1.173× at 2.00 m. Scaling adds up to 0.03 (slow hands 1.196×, fast bat 1.182×).
- *How often games go there.* The standard run's 67,158 full swings (`aims.csv`, an out-of-tree probe over both seed sets) aim inside of +0.25 m on 0.0%, below 0.40 m on 0.5%, and inside of +0.15 m and below the zone on 0.6%.
- *Verdict.* Acceptable under D27 as reported failures. §6.7.4 should cite these shares instead of "a batter swings at pitches out of the zone often".

**P3-3. The family tests that restate the construction should say so.**
- *Which ones.*
  - `test_travel_across_batters_as_MLBs`: its flat length ÷ speed is λ ∝ `BatSpeed` restated.
  - `test_within_a_batter_the_travel_holds_and_the_time_follows` and `test_an_easier_swing_is_the_same_swing_slower`: both are the definition of T ∝ 1/(1 + e).
  - `test_quick_hands_is_a_shorter_swing_and_a_slow_bat_too`: λ < 1 ⇒ less wrap.
- *They still earn their place.* They catch real slips: my "λ on the azimuth only" mutation is caught by two of them and by RatingRange.
- *The one number not imposed.* The slope of 0.054 ft/mph compares a from-rest length with Savant's swing length, which §15 itself equates with the last 65 ms. Over 65 ms the slope is 0.090 ft/mph, at the band's top.
- *Independent tests.* The rate share at u_93 (Appendix E) in the re-plan test, Appendix E's elevation and azimuth at every effort, the hand case, and RatingRange's bounds.
- *A gap.* `test_the_swing_starts_inside_the_decision_window` (`analysis/oracle/test_swing_v3.py:574`) tests T − T̄ for the template only. The relevant quantity is T − T_b for each batter. Slow hands with effort −3 sd reach 47 ms in the zone (`range_summary.out`); with an effort draw at the 4σ truncation, 52–63 ms between the zone's outside edge and 0.30 m out (see Verification), which starts a few ms late. Test T − T_b for each batter.

**P3-4. The mutation rule "each must fail tests other than the golden-file checks" is not met by all.**
- *Which ones.* S piecewise linear (golden only); the hands passing W_93 at the unscaled −93 ms (golden plus one expected failure turning into an unexpected success); s_T = 1 and the −93 ms anchor unscaled (the same, plus the re-plan test).
- *Why it matters.* Catching by "an expected failure starts to pass" is accidental. Each of these mutations makes the out-of-zone one-surge test pass (removes the outside-edge 2.2% dip). That says the dip is sensitive to where u_93 falls, worth knowing for P3-2.
- *Fix.* Either add an independent test for S's shape (for example the clavicle's speed or a body-knot check off the knots) or state the exceptions.

**P3-5. The driven-chain argument is a dimensional identity: right, but narrower than written.**
- *What makes it exact.* `driven_chain_probe.py` uses a constant torque and a wrist stop whose stiffness and damping scale with f and √f. Equations with only inertial and configuration-dependent forces are then exactly self-similar under t → t/√f. That is why T·v is identical to four figures. Gravity is the only thing that breaks it.
- *What it does not test.* Time-profiled activation, the force–velocity relation of muscle, or an active release, any of which would break the similarity. It also only supports the within-batter half.
- *The across-batter half.* λ ∝ `BatSpeed` at a common time is the opposite of what the chain gives a stronger batter with the same body (same path, shorter time). It rests on note 10 alone.
- *T ∝ 1/(contact speed) and T ∝ 1/√(drive) are the same statement* (v ∝ √f). Effort ±3 sd is drive × 0.75–1.29.
- *Fix.* Say all of this in §6.7.1.

**P3-6. T_t depending on the aim is an outcome of the geometry, not a source; it is planned from his own estimate, which is fine.**
- *Size.* Inside pitches take 13 ms less (higher contact rate at a shorter reach).
- *No source.* The common coaching view is the opposite: an inside pitch needs an earlier start. No source here measures it, so mark it [E-outcome] in §6.7.3 and §15.
- *Invariant 2.* The batter plans T from his own aim at the commit (`bank.dominant()`), so nothing is read from the world. A re-plan keeps T and lets λ follow: tested to 10 cm, adding at most 1.2% to a dip.

**P3-7. Check-swing timing with a variable T (§8.3, stale).**
- *The abort point.* t_s + 0.6·T is t_c − 0.4·T: from −49 ms (T 0.123 s) to −98 ms (0.244 s).
- *What binds when.* `pa.rs`'s observation loop stops 70 ms before the aim, so for T < 0.175 s the loop binds, and the last brake is about 20 ms before contact. For longer swings the 0.6 rule closes earlier, up to 48 ms before contact.
- *Stale lines.* §8.3's "never after t_c − 10 ms (the abort point, 60 ms before contact…)" and its "at −12 ms: … a swing" no longer apply to the average batter. Under v3 a yaw-ruled check swing needs a quick swing.
- *Fix.* Restate, and add it to §12's checked-swing row.

**P3-8. Smaller overstatements.**
- §6.7.3, "the azimuth is [4]'s shape exactly". δ_A still moves the rate by +0.06 of the contact rate at −60 ms and −0.05 to −0.08 at −10 ms. The peak is 1.02–1.07× at −9 ms, against [4]'s 1.09.
- §5's table checks the launch azimuth: "comes out at −225.26°". That agreement with [4]'s −225° is a coincidence of two offsets. The template's sweep is 230°, against [4]'s 217° (its rate graph) or 218° (its text); the jet meets the ball 12° further round than [4]'s −7° and turns 6% faster. Say the absolute wrap matches and the sweep is 6% longer, as λ's logic expects of a faster bat.
- §6.7.3's after-contact statement (within 0.2 mph to +10 ms) holds for grade 50 only. Over the family it is 0.55 mph at +10 ms and 3.0 mph at +25 ms (slow hands, the outside edge; `after_contact.out`). That is still small, since contact past +15 ms is rare.

**P3-9. Stale lines from the fixed-time design** (`python3 bin/check-docs.py` passes on the committed files; its only complaints are the untracked job file).
- §1 "Between" ("168° short of contact and turning at 56%"; now 175.9° and 0.565).
- §2 "By construction v3 reproduces … every contact after it" (now v2's at the swing's own T, P3-8) and its −10 / −25 ms figures (step 1's).
- §6.1 step 3 (bat at "−225°" and "A_ref − 168°"; now −225.26° and −171.10° per step 4).
- §6.3 "Why these conditions" ("δ_A adds about 13°") and "What it follows" ("the rate 0.10–0.16 above [4]'s … (the 13°)").
- §6.3 "What it costs" (step 1's numbers, including the tilt 39.2 / 48.9 / 28.9°, now 40.5 / 46.0 / 33.7°).
- §5's row "Peak bat azimuth rate … 9–22 ms … 1.05–1.07".
- §8.3 (P3-7).
- §14 Q48 ("the decision clock and the plate-appearance rates stay").
- §15 "Live MLB timing … s_T is an assumption".
- §13 lists step 1b above step 1.

## Answers to the job's questions, in short

1. **Range.**
   - *In the zone.* Yes, for the bat; my own grid confirms §6.7.4: peak 1.000–1.088×, never earlier than −25 ms × T/T̄, azimuth-rate dip 0, bat clear.
   - *The 2% allowance.* It has 0.1% headroom, and its measure is lenient early in the swing (P3-1).
   - *Out of the zone.* The exceptions are the template's; scaling adds at most 0.03 and the long corner (P3-2). Games aim there on 0.0–0.6% of swings, which is acceptable.
   - *Not checked by the spec.* The hands' double surge (P1-2) and the lead arm's reach (P2-1).
2. **Scaling.**
   - *(a)* A, E and W all scale about the batter's own contact jet, with the launch about his middle-middle jet via the fade. Short swings stay anatomical; long swings over-reach the lead arm (P2-1). The 52–62° launch elevations of long swings are an [E] consequence of putting the travel into E, unsourced; [4] shows 41–44°.
   - *(b)* λ reproduces note 10 by construction, not by fitting: the relation is built in and the tests restate it (P3-3).
   - *(c)* The chain is right as an identity, and T ∝ 1/v and T ∝ 1/√f are the same statement (P3-5).
   - *(d)* T_t follows the aim from the batter's own estimate, so invariant 2 holds, but it is unsourced (P3-6).
3. **Timing.**
   - *Effort.* It is execution noise. The start absorbs it, so an easier swing is not late. If it were, a swing would be 7.3 ms late per sd, more than the game's 5.6 ms timing sd (P2-4).
   - *Invariant 3* holds if the held T is the intended one (P2-4).
   - *The check-swing window* now depends on T (P3-7).
   - *The commit point* follows T_b, but in today's games every batter's `t_swing` is 0.150 s: only `BatSpeed` varies, so the QuickHands half of the family is dormant in games until that rating exists.
4. **Contact on time.**
   - *At the planned instant* it is unchanged. After contact it is v2's at the swing's own T: within 0.15 mph to +5 ms and 0.55 mph to +10 ms over the family.
   - *Late contact.* §6.7.6's grade-50 numbers reproduce: −3.9 / −6.4 / −1.9° at −10 ms; middle 71.3 against 68.6 mph at −15 ms against v2 at the same T, 68.0 against today's game.
   - *The attack slope and the tilt* are honest for grade 50 only (P2-2). The "tilt 8° against 16°" difference is the template's wrap (−225° against −198°), not the time: a hard swing at 0.150 s on the full wrap is +6.2°.
5. **The owner's choice** is mis-posed (P1-1).
6. **Tests and mutations.**
   - *Runs.* `test_oracle.py`: 73 tests OK, 11 expected failures. The worker's runner (1,260 s) reproduces the spec's counts.
   - *The runner's fix.* The static-method restore works: `travel` is a staticmethod again after a patch and restore, and an inherited `A_max` is deleted again.
   - *The gap the worker added* (the re-plan's rate share) catches "a re-plan holds the travel".
   - *Unlabelled exceptions* are listed in P3-4.
   - *My mutations.* λ on the azimuth only: 5 caught, including RatingRange. Effort multiplying the travel at a fixed time: 5 (not RatingRange, P2-5). T_t fixed per batter: 7 (AimRange, RatingRange).
7. **Stale lines** are in P3-9. `bin/check-docs.py` is clean on the committed files.

## Verification

All under `sim/target/review-probes/family/` (outputs beside the scripts), run from the repository root:
- `python3 -m unittest analysis/oracle/test_oracle.py`: job 267, 61 s: 73 tests, OK (expected failures = 11) (`oracle_tests.txt`).
- `python3 analysis/oracle/mutations_swing_v3.py`: job 266, 1,260 s (`mutations_worker.txt`). `my_mutations.py`: job 270, 206 s (`my_mutations.out`): none 0; λ on the azimuth only 5; effort scales the travel 5; T_t fixed per batter 7. A direct check that the runner's patch/restore leaves `Swing.travel` a staticmethod and removes an inherited override (printed in the session).
- `range_probe.py`: job 271, 439 s, 1,715 swings, 0 errors (`range_probe.json`, `range_summary.out`). The grid: x_in −0.40 … +0.50 m (7), z from 0.2 of the zone's height below it to 0.15 above it (7), H 1.70 / 1.85 / 2.00, both hands, 15 rating and effort combinations at H 1.85 (fewer at the other heights and for left-handers). Speeds by central differences of `Swing.point`, azimuth rate from the reported unwrapped A, the arms' distance from each shoulder at the body knots, and the bat against the trunk capsule and head on 21 bat points. (A first run, job 269, lost 659 swings to a bug in my own dip locator; rerun after the fix.)
- `anatomy.py` (P2-1), `hands_speed.out` / `early_phase.py` (P1-2, P3-1), `start_defs.py` (P1-1), `after_contact.py` and `late_vs_today.py` (P2-2, Q4), `slope_family.py` (the attack slope, tilt, lag and uncocking across the family), `driven_chain_probe.py` run and read (P3-5).
- The game's swing aims: an out-of-tree crate `aimdump/` (path dependencies on the workspace crates, own target dir; no source edited), job 268, 300 + 300 games at the standard seeds: 77,461 swings, 67,158 full; every `t_swing` 0.150 s (`aims.csv`).
- Start against the commit for slow swings: T − T_b at the plate's outside edge (x 0.25 m, middle height) = 38 ms grade 50 at −3 sd effort, 44 ms slow hands at −3 sd, 57 ms slow hands at the 4σ truncation; 51–73 ms at the shins 0.45 m outside (grade 50 at −3 sd to slow hands at the truncation).
- `python3 bin/check-docs.py`: no complaint about committed files (only the untracked `REVIEW-SWING-FAMILY.md`).

## Resolution

Open.
