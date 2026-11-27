# Review: body-3d session 13: the swing looked at, and the proposal for what the score should ask

**Reviewer:** Opus 5.5 (an independent Claude worker that wrote none of this), 2026-11-27. **Range:** `main..work/body-3d` (`d1f7bb4`..`146f077`). **Brief:** `REVIEW-body-3d-s13.md`. No search was run (the owner has cancelled them); only replays and single forward runs, with probes in `sim/target/review-probes/s13/`.

## Summary for the owner (one minute)

- **Is the account right? Yes, in substance.** I read all twelve frame sheets and reran the table. It reproduces to the digit. All seven single-muscle replays behind the causes reproduce to the digit with code I wrote myself. The viewer draws what the model computes (its check passes again at 4.2 mm and 1.5°, and I compared the viewer and the model's own lines at −60 ms and at contact by eye). It is a chop from hands at shoulder height, with the bat hanging about 59° down at the ball. He has almost no weight on his feet, the rear leg is locked straight and he pitches forward afterward. Every kept swing since session 10 has the barrel 45–60° below level and 0–24% of his weight on the front foot at contact. Some numbers are wrong or overstated:
  - His chest is 29° ahead of his hips at contact, not 45°: it started 16° behind them.
  - His arms are bent about 52° at contact, not "stretched straight".
  - A hitter's pelvis is at about 208°/s at the instant his front foot lands, not 566°/s. That figure belongs 90 ms later.
  - The barrel runs within 5 cm of the ball's line for 22 ms around contact, not "the last 10 ms only".
  - Several hitter figures come from smoothed data you ruled out (D35), or are arithmetic presented as measurements.
- **Is the proposal the right thing to ask of the batter? Right in direction, wrong in its order and in three items.**
  - **Item 1, ranked as the change the author "expects most from", would not change this swing.** The present chop already meets the ball within 2 cm of the sweet spot from 14 ms early to 14 ms late. That is in the session's own output, and I reran it.
  - **Item 2, as written, asks for the ball "up the middle" at all three timings.** That runs against your D44: early swings pull the ball and late ones push it. Both kept chops already send their 7 ms early and 7 ms late balls up the middle. Only a launch-angle band would bite, and the proposal leaves that band unstated.
  - **Item 7 proposes four gate rows from the lab study [4],** which D37 forbids. A fifth row asks the bat to peak 10 ms before contact, which D35 rules out and which contradicts the gate's own row "the bat at its fastest at contact".
  - **Missing: your own ruling D39** (a joint's end takes energy and the search avoids loading it). It is decided and still not built. The locked rear leg is how he stands on nothing: his rear knee leans 837 N·m on its end (ligaments fail at 13–34). Item 3's demand for weight on his feet would reward more of exactly that.
- **What I would change:**
  - Put D39 first.
  - Merge items 1 and 2 into one ask: the ball's exit speed and launch angle by the game's collision when he is on time, 7 ms early and 7 ms late. Leave the direction free, so that early can pull and late can push; D44's check stays a check.
  - Take every [4] number out of the gate rows and out of item 3's floor, and drop the 10 ms bat peak.
  - Say that item 5 (a quiet head) has no measurement behind it. The model has no head of its own: the head is the top of the trunk.
  - Item 6 (his back's muscles) is inside D34 only if the strength at speed above 2.1 rad/s is stated as an estimate. It is not needed to hold him up: the front hip's extensor could do that and is used at 1% of its strength.
- **What to build first:**
  1. D39.
  2. The merged items 1 and 2.
  3. Item 3.
  4. Item 6, as one change.
  5. The balance control of §5.3, before item 4: an open-loop "finish on his feet" would be met by a finish that falls under the first error in a game.

## Findings

### P1

**P1-1. Proposal item 1 would not change the present swing: the chop already meets the ball at 7 ms early and late.** `docs/body-motion.md:3442`, with the table row at `:3415`. Item 1 says: "A barrel that dives onto the ball's line for its last 10 ms cannot do it… This is the single change I expect most from." The session's own `s13/look/contact_pm7.out` already contradicts this. I reran it at more timings (`contact3d.py --swing practice3d_s12_posture.json --shifts 0,±3,±5,±7,±10,±14,±20`; `review-probes/s13/contact_s12.out`). By the game's collision, the 26.25 m/s chop meets the ball:
- 0.2 cm from the sweet spot when 7 ms early, 0.0 cm when 7 ms late;
- within 2 cm from 14 ms early to 14 ms late;
- within 4.1 cm at ±20 ms.

The session's own 25.63 m/s swing does the same: within 0.5 cm at ±7 ms (`contact_r4s1.out`). The reason is that the sweet spot runs along the ball's line after contact as well as before it. It is within 5 cm of the line from −10.4 to +11.1 ms (53 cm of the line) and within 8 cm from −13.9 to +14.1 ms (`geom_s12.out`). The table says "on the line for the last 10 ms only". Also, the score already charges this at first order: its third and fourth terms are the sweet spot's velocity off the ball's line times `TIMING_ERR` = 7 ms (`practice3d.py:535–537`). Item 1 is that term made exact, and the chop already satisfies it. The bat speed 7 ms early is 24.97 against 26.27 m/s, so "it also asks the bat to still be fast out in front" asks for 5%. **What does separate the chop from a hitter's swing at those timings is where the ball goes up and down:** +6.7° when 7 ms early, −14.8° when late (−18° at 10 ms late). That is item 2's launch band (P1-2), not item 1.

**P1-2. Item 2's direction ask would be met by the chop and runs against D44; its only teeth are a launch band it does not state.** `:3443`. It asks the ball at the three timings of item 1 to go "between the gaps", "neither into the ground nor straight up". By the collision:
- the 26.25 swing sends its 7 ms early ball +1.8° (up the middle) and its 7 ms late ball +12.4°;
- the 25.63 swing sends them +5.2° and +4.8°: up the middle both times.

So a steep bat already makes direction nearly independent of timing. A score asking for "up the middle" at all three timings rewards that, and it rewards most a bat that translates along the line without turning. D44 says the opposite: "An early or late swing then meets it further out or deeper, and pulls it or pushes it". Its check is "that early swings pull the ball and are faster". The item's force therefore rests entirely on the launch band [E]. The same 26.25 swing leaves −5.5° on time, +6.7° early and −14.8° late, so whether the chop is charged depends on a number not yet written. Recommendation: ask for the exit speed and the launch angle at the three timings, with the band written down before any run and tried at two widths. Leave direction free: "fair" at most. Keep D44's pull and push as a check never given to the search.

**P1-3. Four proposed gate rows come from [4], against D37; one row is the smoothed bat peak that D35 rules out.** `:3448`. D37: "nothing the body is fitted to and nothing it must pass comes from it… the study is a picture to look at beside the result, never a number to fit or to pass." Item 7 proposes:
- the barrel 30 ± 5° below the hands [4];
- the pelvis turned 77 ± 6° and the chest −2 ± 4° against it [4];
- the order of top speeds "75–70, 65–57 and 10 ms before contact; Welch, [4]";
- the front foot's load at contact, 64% [4].

As gate rows (things he "must pass") these are **against the ruling, plainly.** Citing them in the account's "what a hitter does" column is inside it.

The bat's top speed "10 ms before contact" is [4]'s bat azimuth-rate peak (`swing/4-miami-study.md:46`). It lies in the last 40 ms that D35 removes as smoothed ("the bat's loss at impact is spread backwards"). It also contradicts the gate's own row, "the bat at its fastest at contact" (D35, D37, D40).

Measured replacements exist:
- the swing's tilt, Statcast (32.0 ± 5.7° on middle fastballs; `savant.out:31`);
- the pelvis and separation at contact, OpenBiomechanics (`swing/6-openbiomechanics.md:62–64`);
- the order pelvis −75, torso −70, bat at contact (OB);
- the front foot's peak load, already in D40.

The front foot at contact has no non-[4] figure except Welch's 84%, also a tee study.

**P1-4. Your ruling D39 is missing from the proposal, and item 3 would reward the fault it covers.** `:3440–3449`; the kept swing's options carry no `strain_w`, so `practice3d.py:287` scores joint-end loads at 0. "Falling forward on locked legs" is half of your description of the swing. The locked rear leg (knee 1°, heel flat; table row at `:3416`) carries him on the knee's end: 837 N·m in the 25.63 swing, and 11–12 ends over 50 N·m in both swings (`gate` in `r4_s1_posture.json`; `ligaments_over_50` in `practice3d_s12_posture.json`). Ligaments fail at 13–34 N·m (note 11 B). D39 is decided ("the practice search counts a heavy load on a joint's end as something to avoid") and has been "not built" since session 8. The proposal does not list it. Item 3 asks for "a floor near his weight" on his feet through the last 100 ms. The cheapest way to carry weight without muscle is a straight leg standing on its stop. **The likeliest loophole of item 3 is a stiffer version of today's locked legs, unless D39 is in the score first.** Recommendation: D39 is item 0, built before any of items 1–7.

### P2

**P2-1. "Chest 45° ahead of the hips" is 29°; the pelvis's 38° is compared with an angle in another frame.** `:3416`. The account takes the difference of the two turns since the start (83° − 38°). At the start his chest was already 16.1° *behind* his pelvis (absolute `trunk_rot` − `pelvis_rot` = −24.0 − (−7.9)). At contact it is +28.8° ahead (`geom_s12.out`). Hitters at contact have −2 ± 4° ([4]) or about 2° (OB), so the conclusion stands, but the number is wrong. Likewise "buckle turned 38°" sits beside "pelvis turned 77 ± 6° [4]". The 77 is [4]'s absolute angle ("19 to 77 deg… to face roughly the pitcher"), in which a hitter's pelvis turns about 95° from foot plant. Compare turn with turn (38 against about 95, or OB's 76° from plant to contact), or angle with angle.

**P2-2. Hitter figures placed at the wrong instant.** `:3412–3414`.
- "His pelvis is already 25° open and turning at 566°/s as the foot plants [OB]" sits in the row of the model's front foot landing (−211 ms). OB's 566°/s is at full plant, fp_100, 122 ms before contact (`swing/6:26`, `:50`). At 200 ms before contact OB's hitters turn at 208°/s (`swing/6:62`).
- "480–640°/s through this stretch [200 to 110 ms]" holds only from −150 ms.
- "The rear upper arm comes in from 74° to 29°" spans −400 to −50 ms, not the 90–40 ms row (`swing/6:67`).

The model still lags hitters at every instant: −84 to +14°/s against 208, and 214 against 479–641. But the gap at foot-down is smaller than the account shows. That matters for P2-8: how much of the lag is the start.

**P2-3. Figures from smoothed data or arithmetic, given as hitters' facts.**
- "The bat is at 98% of its top speed at contact and peaks within 2 ms of it [OB]" (`:3417`). PLAN D35 itself says this figure "is itself filtered through contact and bears little weight"; note 10 says "almost no weight".
- "The measured spread is 7 ms" (`:3442`; also `practice3d.py:51`, "the measured spread"). Note 6 converts Statcast's within-batter contact depth (SD 27 cm) into time, "my arithmetic, assuming every depth change were timing" (`body/6-movement-variability.md:28`). That is an upper bound, not a measured timing spread. The measured figures in that note are a skilled thrower's 7 ms release window and a batter's 9.4 ± 6.3 ms sweet-spot tolerance (`:18`, `:20`).
- "Braking his forward move with half his weight at 83 ms [4]" is a horizontal force of 0.5 body weight (`swing/4:41`), not weight carried.
- "130% at 93 ms [4]" is [4]'s averaged curve (149 ± 12 in its swings' own peaks, note 10).
- "+8 to +10° for major leaguers" is a video analyst's figure (`swing/7:40`), not marked "coaching".
- "Hips before hands" is in quotation marks as coaching, but the sources say hips before torso (`swing/2:30`, `body/0:88`).

**P2-4. Item 6's reason ("without it he cannot… hold himself up, whatever the score asks") is not supported by the session's own replays.** `:3447`.
- With the back held as strong as Schultz's 482 N·m, he still leans 39° at the ball: the fold moves to the hips (`back_replays.out`; the account says so at `:3423`).
- The muscle that would hold him up at the hips is already the search's: the front hip's extensor, bound 430 N·m, is asked for **4.4 N·m** in the 26.25 swing (`practice3d_s12_posture.json` plan, `hip_ext_L`). Meanwhile the front hip's *flexor* does 94 J of folding.

So the fold is the search's choice, because nothing charges it. It is not a missing muscle. Item 6 may still be right for the side tilt that brings the hands down (cause 1(d)), and it is inside D34 for strength: Schultz 1987, isometric, mean + 2 SD, scaled like every other bound. But a fired side bend at hitters' rates (4° to 32° in about 120 ms, roughly 4 rad/s) would run on the part of the strength-at-speed line that is [E]. Only Thompson's extension is measured, to 2.09 rad/s, and it is then used for all four directions (`oracle/body3d/drive.py:510–524`). D34 asks for that to be written into the spec as an estimate before the run. §4.13 rung 2 deferred these muscles as "a different change (more muscle for the swing)", so under D42 it is one change with a bat speed before and after, not a fix of the score.

**P2-5. Items 3 and 5 put unmeasured or [4]-derived numbers in the score.** `:3444`, `:3446`.
- Item 3 justifies "a floor near his weight [E]" by "hitters' front foot alone has 130% at 93 ms and 64% at contact, [4]": a [4] number entering the score by another name.
- Item 5's allowance for head travel has no measurement (the notes have none: OB "Head: not covered"; the account admits it). The model has no head of its own: `swing_look.py` draws the head 0.20 m up the trunk's axis from the shoulders. So item 5 is a charge on the trunk's top. That is a shaping term chosen by eye, the kind of unmeasured allowance D34 and D37 keep out, and it overlaps item 3 (centre of mass) and item 4 (head not at his knees).

Recommendation: write item 3's floor from what a body standing still needs (his own weight, the physics) rather than from [4]. Hold item 5 until a measurement of head travel is found.

**P2-6. Cause 1(a) is overstated, and the replay method shows contributions, not causes.** `:3421`. "The hands are there because (a) his lead arm is held up… by the muscle that raises it." Without that muscle the hands are at 1.19 m instead of 1.28 m: 9 cm of the 50 cm by which they are above the ball. They start at 1.48 m, in [4]'s launch posture (`causes.out`, section 1). A single muscle removed from a searched plan, with nothing re-tuned, shows what that muscle does *in this plan*. It does not show why the search chose it: that is what nothing in the score charges (cause 1(c)). The account flags replays as "indicative", but reads them as causes at `:3421–3425`. The *what* of each replay is right (all seven reproduce: `review-probes/s13/replays.out`).

**P2-7. The account's figures for forward travel compare different baselines.** `:3416`, `:3426`, `:3444`. "His centre of mass has gone 21 cm toward the pitcher by contact (a hitter's: 6 cm from foot plant to contact)" compares from launch against from plant. From his own front foot landing (−211 ms) to contact, his is 17.5 cm (`geom_s12.out`). OB's 6 cm is 0.23 − 0.17 m, a difference of two means measured from the first frame of the take (`swing/6:46`). The point stands (17.5 against 6); the number does not.

**P2-8. Item 4 rests on the balance control of §5.3, and item 5 on a stride; neither is said.** `:3445–3446`. A standing finish 150 ms after contact must be reached by an open-loop plan of about 425 ms with no feedback. The search can find one: it is deterministic. But D32 says that in a game "his body only runs a learned motion forward with its errors". A finish balanced only by practice falls at the first error, and nothing will look. Build §5.3's postural feedback before item 4, or say that item 4 tests practice only.

On the start, item by item (the brief's question 4):
- Items 1–3 can be asked of a body that starts at [4]'s foot plant, moving at 0.6 m/s: hitters move at up to 1.06 m/s at plant (OB). The start does not force a chop. Session 12's "front leg pushing" run carried 123% of his weight at contact.
- The start's near-zero pelvis rate does cost turn. Hitters are at about 208°/s at touchdown (P2-2), so it is a real part of the hips' lag, which cause 4 attributes to the feet alone.
- Items 4 and 5 need what is not built.

### P3

- **P3-1. "Both arms stretched straight out" (`:3416`):** the model's elbows are bent 52° (lead) and 51° (rear) at contact (the table; the line drawings show both bent). The account's own hitter column gives "his are 51° and 52°".
- **P3-2. Not mentioned:** his front foot turns 40° open by contact (`foot_L` −39° in the table) while carrying 0.00–0.07 of his weight. A hitter's front foot is planted. This is the feet "spinning" of §4.14, still there.
- **P3-3. The replays behind the causes have no committed script.** `causes.out` and `back_replays.out` exist only as outputs. I reproduced the muscle removals with my own code. The two "back held" replays cannot be reproduced without knowing how the back was held (a stiff hold up to 482 or 150 N·m?).
- **P3-4. contact3d's agreement with the game's collision is by reading, not by test.** Its impulse is the oracle's `bbphys` (held to Rust by `collision_golden`, which passes: 12 tests). Its restitution `e_n` and detection are re-typed from the spec. I checked the constants against `bb_physics/src/bat.rs:200–219` and `:229–260` (identical), and the gap and normal against `contact.rs:43–148` (identical on the barrel's side; the bat's end caps are not handled, which does not matter at the sweet spot). The `--check`'s head-on case works its expected value from contact3d's own `e_n`, so it does not test `e_n` against the game. A golden case run through both would.
- **P3-5. Sources disagree where one is cited, and the gate bands would follow one.** Rear knee at contact: OB 71°, Welch 45°, [4] 54°. Side tilt: OB 32 ± 8°, Welch 20°. Front foot at contact: [4] 64%, Welch 84%. The proposed trunk-lean row "upright to 15° (Welch, [OB])" is ambiguous in sign: Welch has 9° *back*, OB's "torso x +15" has no stated direction, and [4]'s 16° is 30 ms after contact.
- **P3-6.** Statcast's swing tilt on middle fastballs: 10th–90th percentile 25–39° (`savant.out:31`), not 25–38°. The 32 ± 6 is its SD (5.7) and is right.

## The rest of the session's code (question 6)

- **Part 0a (the feet's load at every step):** right. `body3d_probe.rs` fills the diagnostics from each step's first evaluation, which is of the state the step starts from. `probe3d.py --check` passes (worst 3.6 × 10⁻¹¹), so the states are unchanged. The test (`SessionThirteenChecks.test_the_feets_least_load…`) works the ground's force from the contact law in the test itself, not from the probe's force code. It is independent and it passes.
- **D44's ball place, `widened`, `rebodied`:** the tests are independent:
  - the place along the line is worked by hand from the pitch's angles;
  - `widened` is tested on a bowl with a known least;
  - `rebodied` is tested in N·m against the plan.

  The arms' clock moving with the place is consistent with how the clock is defined: a ball met further out has flown less, so the decision is earlier on the same look. Unsearched, as the account says.
- **`contact3d.py`:** see P3-4. Its `--check` passes. The ±7 ms replays reproduce exactly.
- **Savant script:** `--check` passes, and a rerun on the twelve files is identical to the session's output. The doc's table matches it (spot-checked: the up-the-middle row, the fastball row, the within-batter slopes).
- **Viewer `-BbBodySwingShots=<ms>`:** the change is only in `BbBodySwing.cpp/.h` and a comment in `BbDirector.h`. Nothing in a game changes. The old form still takes its 16 pictures. `-BbBodySwingCheck` passes (4.2 mm, 1.5°). `ViewTags` already has "above", so the third view is safe.

## The account's other claims (question 7)

- "Passes twelve of the gate's eighteen rows": **true** (`gate` in `r4_s1_posture.json`: 18 rows, 12 passed). Two of the twelve pass in the chop's own way:
  - the lead knee at 4° on a leg carrying 0.00 of his weight ("the front leg firming");
  - the front foot's greatest load as a spike 58 ms after the start.
- "Every bat speed since session 10 is the speed of that chop": **consistent** on the two measures I could read for all eight kept swings: barrel 45–60° below level, front foot 0.00–0.24 of his weight at contact. I did not watch the older swings move.
- 25.63 m/s at 2,681 generations: **true** of the kept result. The collision replay gives 25.61 at the on-time touch.

## Verification

Every command was run from the worktree root unless noted; outputs are in `sim/target/review-probes/s13/`.

| Command | Result |
| :-- | :-- |
| `python3 analysis/research/body/swing_look.py analysis/research/body/practice3d_s12_posture.json` | `table.out`: identical to `s13/look/s12_posture_table.out` (diff: only the session's "drew" lines) |
| all 12 sheets in `docs/img/body-s13/` | read by eye: 6 viewer sheets and 6 line sheets; −60 ms and contact compared between the viewer and the lines: same pose |
| `replays.py` (mine: one burst's strength set to 0, nothing re-tuned) | `replays.out`: the kept swing and all six removals match `causes.out` and `back_replays.out` to the printed digit |
| `geom.py` (mine) | `geom_s12.out`: separation −16.1° at the start, +28.8° at contact; centre of mass moving at 0.595 m/s toward the pitcher at the start; head travel 37.8 cm (34.6 toward the pitcher), 114.6 cm by +170 ms (59.3 lower); centre of mass +21.4 cm by contact, +17.5 cm from front foot down; sweet spot within 3/5/8 cm of the ball's line for 16/22/28 ms |
| `contact3d.py --check` | PASS (4 cases) |
| `contact3d.py --swing practice3d_s12_posture.json --shifts 0,3,-3,5,-5,7,-7,10,-10,14,-14,20,-20` | `contact_s12.out`: ±7 identical to `contact_pm7.out`; the sweet spot is ≤2 cm off at ±14 ms |
| `contact3d.py --swing s13/r4_s1_posture.json --shifts 0,7,-7` | `contact_r4s1.out`: on time +1.6°, 7 early +5.2° and +10° up, 7 late +4.8° and −16.6° up |
| `cargo test -p bb_physics --test collision_golden` | 12 passed |
| `python3 -I analysis/statcast/savant_contact_by_spray.py --check` | PASS |
| `python3 -I analysis/statcast/savant_contact_by_spray.py sim/target/research-dl/savant` | `savant.out`: identical to `s13/savant_contact_by_spray.out` |
| `python3 -m unittest test_body3d.SessionThirteenChecks -v` (in `analysis/oracle`) | 5 tests OK |
| `python3 analysis/research/body/probe3d.py --check` | PASS, worst 3.6 × 10⁻¹¹ |
| `bin/body-swing.sh analysis/research/body/practice3d_s12_posture.json -RenderOffscreen -BbBodySwingCheck` | `BB_BODYCHECK result PASS largest_mm 4.2068 … largest_turn_deg 1.539` |
| `bin/body-swing.sh … -RenderOffscreen -BbBodySwingShots` | `BB_BODYSWING pictures taken: 16` |
| the kept swings' JSON (`bat_elev_deg`, `feet_at_contact`, `gate`, `bounds`, plan `hip_ext_L`) | as quoted above |
| hitter figures against `docs/research/` | checked figure by figure by a Sonnet helper; I re-read the lines quoted in P2-2, P2-3 and P1-3 myself (`swing/6:26, 46–50, 62, 64`; `body/6:18–28`; `swing/4:46`; PLAN D35, D37, D39, D40, D44) |

Not done: the two "back held" replays (no script; P3-3); no search of any kind.

## Resolution

Open.
