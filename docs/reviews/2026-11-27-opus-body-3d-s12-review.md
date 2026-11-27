# Review: session 12 of the force-driven body (work/body-3d, `main..47c74f8`, 29 commits)

Reviewer: Opus 5.5 (high effort), 2026-11-27. I wrote none of this. Brief: `REVIEW-body-3d-s12.md`. Every figure of mine below comes from a replay or a single forward run whose script and output are under `sim/target/review-probes/s12/` (Verification). I ran **no search**: no question I reached needed one to settle a P1. I edited no source, test, spec, PLAN or TASKS file.

## Summary: did the session gain honest bat speed?

**Yes. 21.1 → 26.25 m/s (47.2 → 58.7 mph) is a real gain, and every part of it comes from a corrected fault or from the batter's own muscles. Nothing was added to the body, and I found no loophole in the score that pays for it.** What is overstated is how sure the figure is. The 26.25 is the better of two posture searches. The other reached 24.43, and neither search was run to its end.

Part by part, in baseball terms:

1. **The search was being stopped too early (+1.53 m/s, 3.4 mph). Honest.** The same swing on the same body was simply allowed to practise longer. Nothing about the batter changed.
2. **The score measured how square the bat was up to 5 ms before contact (+0.47 m/s, 1.1 mph). Honest: this was a real fault, and the fix asks no less of him.** Read the new way, the bat's angle at contact agrees with a fine-grained read to within 0.1–0.2°. Session 11's kept swing scores 26.20 the new way, against 26.18 the old way, so the old figures were not cheapened.
3. **The search lost what it had learned at every restart (24.24 m/s when run to its end). Honest.** This is the same body and the same swing, practised to the end of a search that keeps its state.
4. **He chose his own stance at launch, inside the same joint limits (+3.14 m/s in one search, +1.70 in the other). Honest, but its size is not pinned down.** I checked the obvious ways this could be a loophole, and none is it:
   - The new start is 0.9 cm *lower* and has *less* motion (22.4 J against 25.3).
   - No joint end is loaded at the start, and every joint is at least 5.1° inside its range.
   - The gain survives, in replays, with the lab study's launch velocities removed (22.1 against 18.4 m/s). It also survives with his starting loads set by his own balance, and at both ends of the unmeasured trunk and hip friction.
   - The two searches still disagree by 1.8 m/s. The second one is *slower* than the same start practised to its end without the posture change (24.43 against 24.67), though the comparison is unequal because the posture search was cut off earlier. So the posture is worth something between about 0 and 3 m/s at the searches' ends. "+3.14" is the top of that range.

**Is the 26.25 m/s swing powered by something a hitter does not have? Not in energy.** His muscles do 599 J of work, and his launch motion holds 22 J. His body drops 5.1 cm, which gives 46 J. Joint friction takes 134 J, the joint ends 34 J and the ground 53 J. The joint ends give back only 6 J. Replayed with ends that give back nothing (D39's first half), he is 0.12 m/s slower.

He *is* falling through contact: the centre of his body is coming down at 0.35 m/s with only 0.13 of his weight on his feet. But every swing this body has made falls the same way (38–58 J of fall), and the fall adds only 7.5 J of the posture's 94 J gain.

**What a hitter does not do, all reported plainly by the session:**
- He leans 1,008 N·m on the end of his rear hip's turn, which only the unbuilt half of D39 would charge.
- One shoulder is 4° past its range.
- He is nearly off the ground at contact, where hitters have about their whole weight on the front foot.

**Waiting on the owner:** nothing new from this review. Q64 has since been answered (D44 on main), so the account's "Q64 is open" lines need updating at the merge, as the brief says.

## Findings

### P2-1. The 0.56 m/s yardstick is the old, short search's. The posture searches' own pair differs by 1.82, so "nine times the spread" and "both gains, shown" overstate how sure the figure is

*Claimed* (`docs/body-motion.md:11`, `:3294`, `:3316`): "+5.14 m/s, nine times the 0.56 m/s by which five searches of this body differ". Also "+3.14 and +1.70 m/s, both gains". Also "a gain is called one above twice that, 1.1 m/s".

*Found.* The 0.56 is measured correctly. I recomputed the five results: 20.894, 21.113, 20.556, 20.624 and 20.908 m/s, a range of 0.556 and a standard deviation of 0.227. But it is the spread of §4.13's 750-generation protocol from one start. The session's later figures come from other protocols:

| Protocol | The two searches | Their difference |
| :-- | :-- | --: |
| 750 generations (the yardstick) | five, 20.56–21.11 | 0.56 (range) |
| 1,500 generations, read at contact | 23.11, 22.73 | 0.38 |
| carried to its end | 24.24, 24.67 | 0.43 |
| **posture, 1,500 generations** | **26.25, 24.43** | **1.82** |

The posture protocol's own pair is three times the yardstick. The +1.70 is less than that pair's own difference. So "nine times the spread" uses the wrong ruler. Against the posture pair, +5.14 is about 2.8 times.

*Found: the second pair does not show a gain at the searches' ends.*
- Same seeds (1, 11), no posture, carried to its end: **24.67 m/s**.
- With the posture, stopped at 1,500 generations: **24.43 m/s**.

The lengths are unequal, and the posture search was still falling (0.39 per 200 generations), so this does not show a loss either. The first pair is clear: 26.25 beats both searches carried to their ends (24.24 and 24.67) by 1.6–2.0 m/s.

*How strong the posture gain is* (my replays, indicative):
- The gain holds with no launch velocities and with loads by balance (finding P2-2).
- It holds across the friction range (Q1 below).
- With a 15 N·m twisting hold on the feet, the first pair still differs by 3.0 (26.08 against 23.12). The second pair narrows to 0.6 (23.69 against 23.12) (`twist.out`).

*What would settle it:*
- Run the second posture search to its end with `posture3d.py --resume` (its state is kept), and compare it with 24.67.
- Rerun the first posture search with its state kept and carry it to its end.
- Report the posture's worth as a range (0–3.1 m/s at 1,500 generations, search for search) until the two ends are in.

The headline 26.25 m/s can stand as "the fastest swing found". The account's own rule takes the better-scoring search, and nothing found here makes that swing dishonest.

### P2-2. The posture start was checked for the main ways a start can give free speed. It passes all of them; the account should say so, and where the remaining lab-study input sits

*Brief, question 3: velocities consistent with the moved posture, the feet's loads, every joint inside its end, no stored stretch in a stop, no height gained for free.*

Checked on the kept start (`energy.out`, `launch.out`, `margins.out`):

| Check | Posture start | Rung 4's start |
| :-- | --: | --: |
| centre of mass height | **0.9558 m** | 0.9648 m |
| kinetic energy | **22.4 J** | 25.3 J |
| least margin to a joint end | 5.10° (`ankle_roll_R`) | 4.04° |
| stops loaded at the start | none | none |
| feet's loads at the start | 0.14 front, 0.65 rear (the lab study's, unchanged) | the same |

- The launch velocities are the lab study's joint rates carried through the new posture. So they are consistent in the body's own terms, and the two starts' pelvis velocities agree to 0.03 m/s.
- The front foot carries a leftover of the lab study's stride: at the start it is still moving down at 0.41 m/s and turning at 6.3 rad/s while carrying 0.14 of his weight. The same is true on rung 4's start, so it is inherited, not new.
- Replays with the launch motion changed (indicative; the plan is not re-tuned):

| Launch | Posture swing | The 23.11 swing (no posture) |
| :-- | --: | --: |
| as kept | 26.25 m/s | 23.11 m/s |
| velocities halved | 25.71 | 22.76 |
| at rest | 22.09 | 18.37 |
| loads by balance (rung 5's rule) | 21.77 | 17.55 |

So the lab study's launch motion is worth about 4–5 m/s *to both* swings, and the posture's gain does not come from it.

*What would settle it:* nothing more is needed. Add the energy, height and margin rows to §4.14 part 4. The account says the start's kinetic energy is lower; it should also say its height is lower.

### P3-1. The every-step "does not jump" reading touches five kept figures, not two; the choice of 24.24 over 24.67 is right

*Claimed* (`docs/body-motion.md:46`): the 5 ms reading "missed a 3 ms dip in two of this session's swings".

*Found* (`jump.out`): the least load on his feet from the start to 50 ms after contact, read at every 0.25 ms step:

| Swing | At 5 ms records | At every step | Time under 0.10 |
| :-- | --: | --: | --: |
| 26.25 (posture) | 0.135 | **0.129** | none |
| 24.43 (posture, seeds 1, 11) | 0.195 | 0.099, at −6 ms | 0.25 ms |
| 24.24 (carried) | 0.146 | 0.068 | 3.0 ms |
| 24.67 (carried, seeds 1, 11) | 0.229 | 0.000 | 3.0 ms |
| r4_s3 (20.56) | 0.142 | 0.089 | 0.75 ms |
| r4_s4 (20.62) | 0.232 | 0.097 | 0.25 ms |

- The gate's "does not jump" row reads at 1 ms. Read at every step, it would fail for r4_s3 and r4_s4, two of the five yardstick searches, which the account says all pass. It would also fail for the second posture swing, by 0.001.
- The score term read at every step: 24.24 gets 0.41, 24.67 gets 4.00, r4_s3 gets 0.05, and all others 0. So by the corrected score 24.24 is the better swing, and the account's choice stands.
- **The headline 26.25 m/s is untouched.**

*What would settle it:* the fix already listed as next session's item 0. Add r4_s3, r4_s4 and the second posture swing to the account's list.

### P3-2. "First rises in order legs, trunk, arms" passes by the gate's own rule, by 4 ms, and the account should give that margin

*Claimed* (`docs/body-motion.md:3298`): "his muscles fire in a hitter's order for the first time since rung 2 (legs 274, trunk 254, arms 250 ms before contact)".

*Found* (`show_posture.out`; the plan's onsets). The rule (`practice3d.report`, `USED` = 0.05 of a bound) reads the earliest onset in each group. On that reading:
- **Legs:** the front hip's flexor, tensed at 0.45 beforehand, with its onset at −273.7 ms. That is 1 ms after launch.
- **Trunk:** the core, at −254.1 ms.
- **Arms:** the rear shoulder's lowering muscle, at −250.1 ms, which is 3 ms inside the clock's 253.

So it passes, by 4 ms between trunk and arms. The legs' main drivers come much later: the rear hip's extensor at −178.5 ms, the front knee's extensor at −171.0 and the front hip's turner at −156.8. Twelve arm groups have their onsets before the first of those.

By the gate's rule (D40) the row passes; I am not asking to change the rule. But "a hitter's order" in the owner's paragraph says more than a 4 ms pass on the earliest onsets shows.

*What would settle it:* print the margin in the row ("trunk to arms 4 ms") and the onsets of the largest leg, trunk and arm groups beside it.

### P3-3. Part 1's diagnosis: the arithmetic checks; three of its plain-language sentences are looser than the runs

I reran replays 0, FR, B and C15 on rung 4's kept swing, and 0, FR and B on the 26.25 swing, from my own build (`pelvis.out`). Every figure I compared matches the account's table:
- rung 4 kept: +11.2° pelvis turn; FR +30.2°; B gives 0.81 of his weight on the front foot, +19.4° and 40.34 kg·m²/s at 21.34 m/s; C15 +24.5° at 22.00 m/s.
- The turning moment: level forces 63.1 + 158.0 = 221.1 N·m, and the feet's own twist 13.4 + 0.6 = 14 N·m.
- The upper body's and legs' turning momentum, 28.96 + 2.68 = 31.64, matches what the ground gave, 31.67.

Looser than the runs:

1. **"His feet are at the edge of their grip making the level forces that turn him"** (`:2894`). I measured each foot's net level force against grip × load over the last 100 ms on rung 4's kept swing. The rear foot is at 0.90 of its grip (at or above 0.9 for 79% of the time). The front foot is at **0.66** (21%). Its individual sole points are at their limit 81% of the time, but they slide in different directions as the foot spins. So the front foot is spinning at its points' limit; it is not making as much level force as its load allows. On the 26.25 swing the front foot *is* at 0.92 of its grip, on 0.18 of his weight.
2. **"A quarter to a half more turning"** (`:12`, `:2898`). The eight replays give +27, +26, +47, +22, +10, +16, +51 and +56%. Better: "a tenth to a half more; a quarter or more in five of eight".
3. **"Which is how hitters turn"** (`:12`, `:2893`). The baseball source, Ae 2017, was read as an abstract only (note 12 D4: "AB"), and it gives no free moment. The "negligible" per-foot twist comes from golf abstracts and an excerpt, against a vendor's 20–25%. Say "as far as one baseball abstract and golf studies show".

### P3-4. Part 2: the what-if ground was kept out of the batter. Wannop's loads and linear trend match the published abstract

- `Ctx(foot_twist)` and `Ctx(sole_wide)` are off by default. `r4_twist15` and `r4_grip_top` exist only in `ladder3d.RUNGS`. None of the three kept swings' options carries them.
- The `body.py` change (2 lines, `:1031–1032`) only lets the start's three arm choices be given from outside. With no `arms` option it does nothing. **"Nothing was added to the body" is true.**
- Wannop's thesis exists (Calgary, 2012, doi 10.11575/PRISM/26204). The journal form's abstract (Wannop & Stefanyshyn 2012, Footwear Science 4:37) confirms the loads 335, 433, 580, 678 and 776 N and that "normal load had a linear effect on traction". I did not open the thesis's chapter 4. The plotted values and the printed fits (14.55 + 0.0215 × load and 15.23 + 0.0188 × load) stay as the reader agent read them, as the account says. They enter nothing in the body.
- One more replay: with the 15 N·m hold, the 26.25 swing is 26.08 m/s and the 24.43 swing is 23.69 (`twist.out`). So the posture swings lean little on the model letting a lightly loaded foot spin freely.

### P3-5. Tests (question 8)

The ten new tests in `SessionTwelveChecks` are mostly against independent cases:
- the turning momentum of a body worked by hand;
- the ground's twist split, worked by hand;
- the momentum balance in the Python engine, a law neither reading is built from;
- the end rule and the leaps on scores whose least is known;
- a kept state that resumes to the last digit of an unbroken search.

Weaker:
- `through_charge` restates `max(0, 5 − v)` with worked numbers.
- The margins test checks the charge on rung 4's start against rounded figures, to ±0.15.
- **Nothing tests the squareness reading where the score uses it** (`Ctx.residuals`: the index `i`, the share `q`, the guard `i + 1 < len(recs)`). Only `bat_between` is tested. I checked it by hand (`square.out`): on four swings the read-at-contact angle is within 0.16° of the angle read from a 0.25 ms record at contact (−1.80 against −1.96; −1.07 against −1.17; −1.42 against −1.31; −0.14 against −0.14). The old reading is off by 15–17° on the session's new swings, which sit away from a record.

*What would settle it:* a test that the score's squareness at a kept result agrees with a fine-step read, and that it does not jump as contact crosses a record.

### P3-6. Smaller wording

- `:11`: "+2.00 m/s by three corrections". The 23.11 figure uses two of them (the longer search and the reading at contact). The third, keeping the search's state, is in the 24.24 figure.
- §4.14 part 4: give the gravity term and the downward speed at contact (46 J; −0.35 m/s, with 0.14–0.49 of his weight on the ground over the last 40 ms) beside "nearer leaving the ground at contact than any swing before". The owner asks whether he is falling into the ball. He is, about as much as every earlier swing, and it is not where the gain came from.
- The first line of `practice3d.report`'s output for a posture result still says "THE REST MOVED BY THE RULE OF 4.12". For `practice3d_s12_posture.json` the rest was moved by practice.
- The spec's checks paragraph says "`body.py` gained one line"; the diff has two.

## Questions as asked

1. **Something a hitter does not have?** Not in energy (the Summary's account; `energy.out`). The body's kinetic energy rises 427.5 J from the start to contact, and the sources sum to it within 6.9 J:

| Source | Work to contact (J) |
| :-- | --: |
| muscles | +573 |
| holding muscles | +26 |
| constant hold-still torques | −3 |
| joint friction | −134 |
| joint ends (stops) | −34 |
| ground | −53 |
| gravity (the fall of his centre of mass) | +46 |

   - The launch state held 22.4 J. At contact the bat holds 308.5 J.
   - **Falling through contact:** yes, his centre of mass is moving down at 0.35 m/s. But the fall is 46 J here against 38–58 J in every earlier swing, and only 7.5 J of the posture's extra 94 J.
   - Of that extra 94 J: about 51 J is less joint friction (the trunk's 28 J against 63, because the pelvis now turns with the trunk). Another 40 J is more muscle work. Ground, gravity and stops nearly cancel.
   - **The friction value is unmeasured, but the gain does not hang on it** (`damp.out`, replays):

| Trunk/hip friction | Posture swing | 23.11 swing | Gain |
| :-- | --: | --: | --: |
| trunk at the low end | 26.51 | 23.16 | 3.35 |
| trunk at the high end | 21.08 | 16.98 | 4.10 |
| every unread value low | 26.33 | 22.79 | 3.54 |
| every unread value high | 20.44 | 16.52 | 3.92 |

   - **The stops:** they give back 6.0 J and take 36.8 J (`energy.out`). With stops that give nothing back (a probe copy, `probe_d39.rs`), the swing is 26.11 m/s against 26.23 (`d39.out`).
   - **No passive element does net positive work.**
   - **It holds at a quarter of the step:** 26.14, 26.23, 26.24 and 26.24 m/s at 0.5, 0.25, 0.125 and 0.0625 ms (`steps_posture.out`).
   - **It holds in both engines:** the Python engine and the probe agree state for state to 2.1 × 10⁻¹² through 60 ms after contact, and the speed at contact is identical (`py_posture.out`).
   - What D39's second half (charging load on a joint's end) would cost needs a search, not a replay. Not settled.
2. **The squareness read at contact:** a corrected fault, not a lighter demand (P3-5's figures).
   - The score still asks for the bat square to the ball's line at contact and moving along it; the term for the bat's path off the ball's line is unchanged.
   - Session 11's kept swings score 26.20 and 26.93 the new way (26.18 and 26.61 the old way).
   - The speeds are unchanged, because the speed was always read at contact.
   - "21.1 → 23.11" is like for like in bat speed. It is two corrections together: +1.53 from length, shown by the 22.64 run, and +0.47 from the reading.
3. **The posture search:** P2-1 and P2-2.
   - The margins' charge does what the spec says. I checked it in the code and against its test. The kept posture's charge is 0, with the least margin 5.10°. Rung 4's start's charge is 1.166.
4. **The spread:** P2-1 and P3-1.
5. **Part 1:** P3-3.
6. **Part 2:** P3-4.
7. **The gate table for 26.25:** replayed. Every row and margin the account gives is reproduced: ten pass outright and the front foot's load passes on size only; seven fail. The fails, with margins: the bat 2.8 m/s short; the hands 0.7 m/s short; the hips 145°/s short; the trunk 215°/s short; the front knee 1° over; the lead upper arm 4.0° past its range (2.0° over the allowance); 11 ends loaded, the most 1,008 N·m at the rear hip's turn. The order row: P3-2.
8. **Tests:** P3-5. **The `body.py` lines:** P3-4.
9. **The account:**
   - The lab study is used nowhere as a target or a gate. It stays only in the launch velocities and loads, flagged throughout, and the posture's gain does not rest on them (P2-2).
   - Claims beyond the runs: P2-1, P3-2 and P3-3.
   - The owner's paragraph is plain. It should carry the range for the posture's worth (P2-1).

## Verification

All scripts and outputs are in `sim/target/review-probes/s12/`. The probe was built there from `analysis/research/body/body3d_probe.rs` (`rv_common.py` points `probe3d.OUT` and `BIN` there). Every run used `taskset -c 0-7 nice`. No search was run and no job was queued.

| Command | Result |
| :-- | :-- |
| `python3 rv_show.py practice3d_s12_posture.json show_posture.json` → `show_posture.out` | 26.25 m/s; gate 10 outright, 1 on size, 7 fail, as the account says; 26.15, 26.25, 26.26 at 0.5, 0.25, 0.125 ms |
| `python3 rv_engines.py practice3d_s12_posture.json steps` → `steps_posture.out` | 26.140, 26.231, 26.241, 26.242 m/s at 0.5, 0.25, 0.125, 0.0625 ms (the 5 ms-record reading) |
| `python3 rv_engines.py practice3d_s12_posture.json py` → `py_posture.out` | the Python engine against the probe, to contact + 60 ms: largest state difference 2.09 × 10⁻¹²; speed at contact 26.2308 in both |
| `python3 rv_energy.py practice3d_s12_posture.json cp:s12/r4_s2_sq.json practice3d_s12_carried.json practice3d_s11_rung4.json` → `energy.out` | start heights 0.9558 and 0.9648 m; start kinetic energy 22.4 and 25.3 J; fall 5.1, 4.2, 3.7, 6.4 cm (46, 38, 33, 58 J); downward speed at contact −0.35 to −0.56 m/s; stops give back 6.0, 6.7, 9.0, 8.1 J; no stop loaded at the start |
| `rustc … probe_d39.rs`; `python3 rv_d39.py …` → `d39.out` | stops that give nothing back: 26.23 → 26.11; 24.24 → 24.12; 23.10 → 22.86; 21.04 → 21.06 m/s |
| `python3 rv_launch.py practice3d_s12_posture.json cp:s12/r4_s2_sq.json` → `launch.out` | front foot moving at (0.21, 0.24, −0.41) m/s and turning at 6.3 rad/s at launch; velocities halved, at rest, loads by balance: 25.71, 22.09, 21.77 against 22.76, 18.37, 17.55 |
| `python3 rv_damp.py …` → `damp.out` | the posture's gain at the ends of the unread friction: +3.35 to +4.10 (seeds 2, 12); +1.7 to +3.5 (seeds 1, 11) |
| `python3 rv_twist.py …` → `twist.out` | with a 15 N·m twisting hold: 26.08, 23.69, 23.12, 24.20 m/s |
| `python3 rv_jump.py` on 14 swings → `jump.out` | P3-1's table |
| `python3 rv_square.py` → `square.out` | the reading at contact within 0.16° of a fine-step read; session 11's kept swing scores 26.182 (old reading) and 26.202 (new) |
| `python3 rv_pelvis.py practice3d_s11_rung4.json 0 FR B C15`; `… practice3d_s12_posture.json 0 FR B` → `pelvis.out` | the account's replay figures reproduced; front foot's net level force at 0.66 of its grip (rung 4 kept) and 0.92 (26.25) |
| `python3 rv_margins.py` → `margins.out` | posture start: least margin 5.10°, charge 0.000; rung 4's start: charge 1.166 |
| The spread recomputed from the five stored results | 20.894, 21.113, 20.556, 20.624, 20.908: range 0.556, standard deviation 0.227, mean 20.819 |
| Wannop & Stefanyshyn 2012 abstract (web) | loads 335–776 N; the effect of load linear |

## Not checked

- The thesis's chapter 4 figures and fits (P3-4).
- What D39's charge on loaded ends would cost (needs a search).
- The searches' own histories beyond their headers and end lines.
- The five checks the session reports (`final_*.out` under `sim/target/codex-probes/body3d/s12/`). I did not rerun them.

## Resolution

Resolved 2026-11-27 (Coordinator). Fixes by a Sonnet worker that ran no search (`16bdf87`, `e4d28c2` on `work/body-3d`, after merging main); merged as `e6ee5ef`. No second review: the fixes are wording taken from this review's own figures and one test.

- **P2-1: fixed in the account; the runs deferred to session 13, first item.** The posture's worth is stated as 0 to 3.1 m/s, with each protocol's own pair beside it; 26.25 m/s stands as the fastest swing found. Both posture searches run to their ends are session 13's item 0.
- **P2-2, P3-1, P3-2, P3-3, P3-6: fixed** in §4.14, the status section, §4.4 and §13 (the tables, the margins, the reworded sentences, the five figures the every-step reading touches).
- **P3-5: fixed.** A test that the score's squareness at a kept result is the bat's angle at contact from a fine-step run. `through_charge`'s and the margins' weaker tests are left as they are.
- **P3-4: nothing to fix.** Wannop's chapter 4 was not opened by the reviewer; its figures enter nothing in the body.
- **Not settled by this review, carried to session 13:** what charging the load on a joint's end (D39's second half) costs this swing; it needs a search.
- **Q64:** answered by the owner the same day (PLAN D44); the account's lines are brought up to it.

Checks run by the Coordinator from `main` after the merge (job 532): 94 body tests, 112 oracle tests (8 expected failures), `built3d.py --check`, `probe3d.py --check` (worst 3.6e-11), `pelvis3d.py --check`, the swing exporter's 6 tests, `bin/check-docs.py`: all pass. Nothing under `sim/` changed.
