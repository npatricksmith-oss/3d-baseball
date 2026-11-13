# Foul sources review — 2026-11-13

Reviewed `706b6ce..818a0a923f919db20d953e37028e025495b1cf50` after reading `CLAUDE.md` and the resolved foul-measures review. This review addresses the new follow-up, especially calibration-note §§6–7, `miss_sources`, the added measurements, the timing × height cage grid, and TASKS. It does not reopen the earlier review's findings. The commit changes diagnostics and documentation only; no simulation behavior change was found. The printed follow-up tables reproduce, but several conclusions exceed or contradict the measurements.

## P1 — The height accounting does not isolate execution or the effect of swing shape

**Location:** `docs/calibration/2026-11-13-foul-measures.md:133`, `:144`–`:145`, `:171`; supporting code `sim/crates/bb_game/examples/foul_diag.rs:153`–`:179`, `:461`–`:465`; repeated in `TASKS.md:27`–`:30`.

The signs are correct: `aim_error.y` is aim **z** minus true ball **z**, and positive `h_nom_cm`/`h_int_cm` means bat above ball. However, these are different reference measurements. `pa.rs:473`–`:476` measures the final aim against the free pitch at the **aim's depth and crossing time**. `h_nom_cm` instead compares the executed sweet spot at `t_c` with the pitch at the **sweet spot's depth and crossing time**. `h_int_cm` uses the closest point on the **extended axis**, potentially beyond the physical tip, at closest approach. Subtracting them gives an accounting identity, not independent causal components.

For current v1, the nominal residual can be written, in meters, as:

```
nominal − aim_error.y − aim_offset_z
  = execution v_offset
    + axis_c.z × (reach noise + intended reach-clamp error)
    + true_z_at_aim_depth − true_z_at_sweet_spot_depth
```

At nominal `t_c`, plane-tilt noise does not move the sweet spot: rotation angle is zero. Its effect enters the subsequent trajectory and therefore the purported “path” term. That term also combines sweet-spot movement, a change of axial station, and the ball's movement between the nominal depth crossing and actual closest approach. It is not a measurement of the height effect of replacing v1 with another swing.

A scratch decomposition of the **504 AD 12–30° whiffs** reproduced “path adds” **−4.582 cm**, but its components were **+8.514 cm** of sweet-spot vertical movement, **−6.765 cm** from changing station along the bat, and **−6.331 cm** from the ball's vertical movement. The small net value hides substantial opposing terms. Across all 4,356 measured competitive swings, the “rest” mean was **−0.244 cm**, but RMS was **1.466 cm**, with p5/p95 **−2.650/+1.910 cm**. Execution vertical offset alone had RMS **1.185 cm**, despite a mean of **−0.007 cm**. Thus “execution noise adds almost nothing” is justified only as a statement about signed mean bias.

The related geometric exclusion is also too strong. Agent-models §3.6 specifies an in-plane hand arc and in-plane lag **plus a small out-of-plane tilt**; it does not establish an identical placed plane to v1. Even with exactly the same plane, different phase or hand motion can change the bat's height and interception time within that tilted plane. A concrete counterexample: PA index **43**, pitch **1**, originally whiffs at AD **18.804°**; delaying its executed swing by **10 ms**, with `n`, `pivot`, `axis_c` and all other geometry unchanged, recovers contact at **10.732 ms after the original `t_c`**, inside the original search window. This is a timing counterexample to the claimed geometric impossibility, not a validation of the full hand-path template.

The large positive aim errors do support prioritizing perception/adjustment experiments. They do **not** establish “a different swing shape would not recover these” or measure the effect of §3.6. R13's 74% MLB whiff rate at AD >30° is conditioned on AD, not on a measured 20–30 cm aim error, so it cannot supply the missing comparison.

**Suggested fix:** Label the residual and intercept-minus-nominal difference as descriptive accounting terms, expose the depth/station/ball-motion terms, and report spread alongside signed means. Preserve aim error as a strong measured contributor while treating the effect of alternative swing kinematics as untested. Correct the categorical exclusions in §§6–7 and TASKS; prioritizing correction can remain a proposed experiment.

## P1 — The `g` marker predicts fouls that are actually fair after the bounce

**Location:** `sim/crates/bb_cli/src/cage_lab.rs:80`, `:96`–`:97`; conclusion at `docs/calibration/2026-11-13-foul-measures.md:166`.

`LA < 0` and `|initial spray| > 45°`, after a fair first landing, does not establish that the ball will roll foul. The code neither follows its ground path nor obtains a rules decision. Spin, ground impulses and roughness change its direction; a fielder's touch or passage of first/third can establish fair before any later movement into foul ground.

I sent all **14 `g` cells** in the default FF grid through `live::simulate_live` and the rules engine, using average fielders, bases empty, actual pitch/bat-yaw/swing-lead cues, and seeds **0–15**. Of **224** plays, **121 were fair and 103 foul**, with no missing decisions. **Seven cells were fair in all 16 seeds**; ten cells had at least one fair result.

For example, **+1 cm / −25 ms** has LA **−41.317°**, spray **−59.391°**, and first landing **(−0.797, 1.280) m**. The grid prints `g`, but all 16 plays were fair. In seed 0 the ball was secured at **(−19.061, 21.038) m**, in fair territory. Even the more extreme **+3 cm / −20 ms** cell, spray **−84.672°**, was fair in all 16 seeds.

**Suggested fix:** Either label `g` only as “downward launch outside ±45°, first landing fair; subsequent call unmeasured,” or classify using the actual ground path and rules facts, explicitly specifying surface/seed and fielding assumptions. Remove the claim that these cells already reproduce MLB's early ground-ball fouls. Refresh the calibration conclusion before using this grid to choose the next model change.

## P1 — The joint table does not support the claimed two exhaustive error groups

**Location:** `docs/calibration/2026-11-13-foul-measures.md:160`, `:172`; repeated in `TASKS.md:29`–`:30`, informing `:35`.

The quarter of swings with large early/high errors is real. The assertion that the rest are close, with few intermediate errors, is not. Applying the note's ranges to the measured sample:

- **1,107/4,356 = 25.4%** have timing error <0 and nominal height >6 cm.
- Only **1,647/4,356 = 37.8%** are within ±15 ms and ±4 cm.
- **1,602/4,356 = 36.8%** are in neither group. That is nearly half of the swings outside the early/high group.

The printed table itself includes substantial additional populations: **5.2%** of all swings in the −4 to −6 cm / −5 to +5 ms cell, **6.7%** below −6 cm / −5 to +5 ms, and **4.1%** below −6 cm / +5 to +15 ms. Nor does every close cell have the claimed 0–2% whiffs and 1–20% fouls: −2 to 0 cm / +5 to +15 ms has **4% whiffs and 28% fouls**, and −4 to −2 cm in that timing bin has **56% fouls**.

“All-or-nothing” pitch-type reading is not established either. The scratch probe found **306 competitive swings with the correct type at commit but nominal height >6 cm**. Among the 1,107 early/high swings, **760 had the correct final dominant type** in the adjustment loop. `PitchRecord.read` records the commit type (`pa.rs:589`), while `aim_error` records the final adjusted aim (`:537`), so those stages must be distinguished. This is compatible with investigating correction limits; it is not evidence for a binary final read/error distribution.

The unbounded >6 cm bin groups mild and severe errors, and a coarse joint table does not by itself demonstrate two separated modes or show which density would reproduce MLB. More partially corrected swings remain a reasonable counterfactual, not a measured solution to the foul deficit.

**Suggested fix:** Report the three population shares, separate positive and negative height tails, and distinguish commit read, final read, commit aim and final aim. Use finer distributions and controlled changes to misreads/adjustability before claiming absent intermediate swings are the main lever. Update TASKS accordingly.

## P1 — About 8 cm is a typical correction budget, not its maximum

**Location:** `docs/calibration/2026-11-13-foul-measures.md:172`; relevant implementation `sim/crates/bb_game/src/pa.rs:444`, `:497`, `:503`–`:531`.

The correction loop does not have a fixed 0.14 s duration. Commit uses `bank.arrival()`, the original aim time comes from the dominant hypothesis at contact depth, and `t_aim` can subsequently move within `dt_lim`. The cutoff is recomputed as **updated `t_aim − 60 ms` every tick**. Later timing correction can extend the time during which spatial correction is applied. Also, `dx` and `dz` are each clamped to `lim`; the code bounds each coordinate's speed, not the Euclidean speed of the aim.

A scratch copy of `pa.rs`, instrumented without changing its decisions, recorded all **4,982 planned/executed swings** in the same 3,000 PAs. The accumulated per-coordinate budget had median **7.8 cm**, p95 **9.0 cm**, maximum **10.2 cm**. Maximum actual net vertical aim change was **9.6 cm**.

A competitive, unbraked example is play index **52004** (PA 52, pitch 4): commit **0.270 s**, final adjustment **0.430 s**, initial aim time **0.47902 s**, final aim time **0.49492 s**. Its vertical aim moves **−9.6 cm**. The late phase correction keeps the updated cutoff beyond the last adjustment; this exceeds the claimed maximum in the actual default sample.

**Suggested fix:** Call 8 cm an approximate typical budget, report the measured distribution and actual displacement separately, and explain the moving cutoff and per-coordinate limits. Measure saturation before attributing the remaining aim errors to an 8 cm hard cap.

## P3 — A missing height crossing now silently removes an otherwise competitive swing

**Location:** `sim/crates/bb_game/examples/foul_diag.rs:174`; stale count at `docs/calibration/2026-11-13-foul-measures.md:11`.

The new `free.find(...)?` exits `measure`, dropping a swing from **all existing Statcast tables** if the free pitch never reaches the nominal sweet-spot depth. The old diagnostic kept it. The default output now says **4,356 competitive swings**, while the note still says **4,357**.

The omitted swing is PA index **717**, pitch **3**, a swinging strike. The pitch hits the ground at y **0.3698675 m**, just before the nominal sweet-spot depth **0.3698039 m**; the new crossing therefore does not exist. Its closest approach and original kinematic measurements are still available.

**Suggested fix:** Keep such swings in the original measures and make the new nominal-error fields optional, reporting missing measurements explicitly. If intentionally restricting the cohort, document that selection and refresh the count and dependent tables.

## P3 — The new topped-ball example includes a cell that already lands foul

**Location:** `docs/calibration/2026-11-13-foul-measures.md:173`.

The stated “5–7 cm overcut at −50° to −67°, spraying −18° to −37°, and it stays fair” only describes the **+5 and +6 cm** on-time cells. At **+7 cm**, default `cage-map` reports LA **−55°**, spray **−159°**, explicitly marked **F**. The ball already lands foul under the PA convention.

**Suggested fix:** Change the range to **5–6 cm**, or describe the +7 cm transition separately. The physical question about the remaining fair topped balls can remain open.

## Verification

All builds and probe executables used `sim/target`. Scratch source, the instrumented copy of `pa.rs`, and logs were under `/tmp`. No source, test, spec or TASKS file was edited.

- Read `CLAUDE.md`, the full range diff, the resolved earlier review, calibration note, R13, agent-models §3.6, and the relevant PA, motor, swing, pitch resolution, contact, ground, live-play and rules paths. Changed executable code is confined to examples/CLI diagnostics. No new outcome rolls, world-state dependency in agents, event-sourcing mutation, or transcendental calls outside `bb_math::m` were found in this commit.
- `cd sim && export PATH=$HOME/.cargo/bin:$PATH && cargo test --workspace --release`: **148 passed, 0 failed**. Zig global/local caches were placed under `sim/target`.
- `cargo run --release -p bb_game --example foul_diag -- 3000`: **4,381 counted swings, 2,835 contacts, 23.7% foul/contact; 4,356 measured competitive swings**. Checked the note's tables and new joint-cell values against output. The §6 decomposition, pitch-type means, **−0.594 cm/ms** regression and **−0.78** correlation reproduce. Existing rounded baseline summaries still reproduce, apart from the competitive-cohort count described above.
- `cargo run --release -p bb_game --example foul_diag -- 3000 - - - -30`: **4,481 swings, 2,951 contacts, 27.5% foul/contact**, mean tilt **31.9°**, median vertical bat angle **−27.5°**. The previously resolved steep-contact tradeoff remains reported in the note; no duplicate finding.
- `cargo run --release -p bb_game --example foul_in_games -- 128`: **9,333 judged contacts**, PA-model fouls **22.6%**, game fouls **22.9%**, **73** game-foul/PA-fair and **43** game-fair/PA-foul disagreements; **0** undecided/timeouts. Caught fouls and home runs emit fair/foul decisions; the diagnostic now explicitly excludes timeout decisions and uses the caught-inclusive **52.5%** MLB aggregate. Those earlier fixes remain intact.
- `cargo run --release -p bb_cli -- cage-map --type FF`: completed; reproduced the timing × height grid, counterfactuals, and topped-ball exceptions above. Also ran the built CLI for **SL** and **CH**: their broad grid patterns are similar, with different boundary cells.
- `cargo run --release -p bb_game --example foul_diag -- 3000 15 - 0.05`: reproduced the low-perception check: approximately **580 AD 12–30° swings with one whiff**, and **4.7%** foul/contact for |AD| <6°. This changes both perception and timing noise, producing a different swing/aim population; it supports an aim-error hypothesis but does not isolate alternative swing shapes.
- Compiled and ran `/tmp/foul_sources_probe.rs` against release libraries. It reproduced the new measurements, decomposed the residual and intercept terms, counted the joint populations, inspected commit/final type reads, traced correction budgets, and ran the **224 live-play grid checks** reported above. The instrumented PA copy matched the original library's **complete swing values, aim errors and batted tuples for every pitch in all 3,000 PAs**.
- Compiled and ran `/tmp/foul_plane_probe.rs`: verified the same-plane timing counterexample and that play **52004** is a counted, unbraked competitive swing. These are diagnostic counterexamples, not proposed simulation changes.

## Resolution

All six findings fixed (2026-11-13); diagnostics and the note only, no simulation code changed.

- **P1 (height accounting):** fixed. `foul_diag` labels the split as an accounting, says what "rest" and "path" contain, and prints the RMS of both beside the means. It also prints the share read right at commit. Note §6 quotes the review's three opposing path components and the rest's RMS of about 1.4 cm, and drops "execution adds almost nothing" and "a different swing shape would not recover these". §7 now says no swing change is indicated or excluded, and cites the review's same-plane timing counterexample.
- **P1 (`g` marker):** fixed. The cage legend now says `g` is a downward launch outside ±45° that first lands fair, with the later call not measured, and cites the live-play check (121 fair, 103 foul). The note no longer claims these cells are MLB's early fouls.
- **P1 (bimodal):** fixed. `foul_diag` prints four populations with shares, whiff, foul and commit read: 25.4% early and over, 37.8% close, 25.2% bat under, 11.5% the rest. The note drops "bimodal" and "all-or-nothing", quotes the review's 760 of 1,107 early-and-over swings that ended with the right type, and keeps the commit read and the final aim distinct. The proposal is now an experiment, not a measured solution.
- **P1 (8 cm budget):** fixed. The note gives the review's measured budget distribution (median 7.8 cm, p95 9.0, max 10.2; largest actual correction 9.6 cm). It explains the per-axis limit and the moving cutoff, and makes saturation the first measurement of the next step.
- **P3 (dropped swing):** fixed. A swing without a nominal crossing keeps every other measure, and its nominal fields are NaN and reported (1 of 4,357).
- **P3 (topped-ball range):** fixed. The range is now 5–6 cm, with +7 cm landing foul backward.
