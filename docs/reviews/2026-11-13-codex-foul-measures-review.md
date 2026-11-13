# Foul measures review — 2026-11-13

Reviewed `bcd7305..77aba6e` at `77aba6e1a03c179da32dcea1e9ef1b41c7a62d7d`, after reading `CLAUDE.md`. The default simulation behavior appears unchanged. The principal findings concern conclusions drawn from otherwise reproducible measurements.

## P1 — The all-contact results do not support “above 40° matches MLB”

**Location:** `docs/calibration/2026-11-13-foul-measures.md:29`, also lines 13 and 102; `TASKS.md:18`.

The note correctly rejects the specific hypothesis about **well-timed** steep contact, but extends that result to all steep contact. Its own table contradicts the extension: at LA 40–50°, the default run gives 133/287 = **46.3%** foul versus MLB **66%**; at 50–60°, it gives 135/208 = **64.9%** versus **74%**. The full-game measurement independently gives 49.4% versus 66%, and 63.3% versus 74%, in these bins. These are substantial remaining steep-contact deficits.

A scratch probe aggregated the default competitive contacts above 40°: **401/663 = 60.5%** foul. Applying the R13 MLB bin rates to those same bin counts gives **72.4%**, a gap of 11.9 points. For the well-timed subset, the corresponding comparison is **226/355 = 63.7%** versus a bin-standardized MLB **62.9%**, which does support the narrower conclusion.

**Suggested fix:** Say that well-timed steep contact passes this check, while all-contact fouls still fall short at 40–60°. Retain low-angle contact as a major deficit without declaring it the entire gap. Correct the matching claims in “What this means” and TASKS.

## P1 — A clamped tip station does not establish the cause of the early whiff

**Location:** `docs/calibration/2026-11-13-foul-measures.md:52`, also lines 102–104 and `TASKS.md:19`; supporting classification at `sim/crates/bb_game/examples/foul_diag.rs:132`.

The **78.2% endpoint classification is reproducible**, but the note treats it as evidence that insufficient longitudinal reach caused those misses and that moving the hands would turn them into fouls. `contact::gap` clamps the ball's projection onto the bat axis to `[0, length]`. Thus `ClosestApproach.u == length` says that the closest axis point is the tip; the ball can simultaneously be far above or below the extended barrel. This is not a mutually exclusive physical diagnosis of “reach miss” versus “height miss.”

Of the **394/504** whiffs at AD 12–30° labeled past tip, a scratch probe found only **32/394** within ball-plus-barrel radius of the extended axis at their recorded closest approach. Extending the final cylindrical profile segment to 3 m, keeping the pitch, swing and search window fixed and running detection only, recovered exactly **32** contacts. The remaining **362** still missed. The median transverse surface clearance of these 394 was **12.9 cm**. For example, PA index 2 has unclamped axial station 1.1320 m versus bat length 0.8636 m, but also an offset from the tip of `(0.0643, 0.1703, −0.3126)` m and a surface gap of 0.2931 m.

This probe isolates longitudinal reach; it neither implements nor disproves the proposed hand-path model. It does show why the reported 78% cannot be attributed wholesale to running out of bat.

**Suggested fix:** Preserve “closest point is the tip” as a geometric label, report axial excess and transverse clearance separately, and describe the fixed-pivot explanation as a hypothesis. Test timing and alternative kinematics while holding pitch and aim errors fixed before claiming that those whiffs become hooked/topped fouls. Update the corresponding TASKS diagnosis.

## P1 — The −30° recommendation omits an existing overcorrection and understates what changes

**Location:** `docs/calibration/2026-11-13-foul-measures.md:98`, also lines 105 and 109; `TASKS.md:25`.

The gain from **23.7% to 27.5%**, the mean tilt of **31.9°**, and the median vertical bat angle of **−27.5°** all reproduce. However, the note singles out −35° as overshooting without reporting that **−30° already substantially overshoots the well-timed steep-contact bins that passed at baseline**:

| LA, with \|AD\| < 6° | Default | −30° | MLB R13 |
|---|---:|---:|---:|
| 40–50° | 43.9% (n=148) | 66.9% (n=172) | 46% |
| 50–60° | 70.8% (n=120) | 83.0% (n=94) | 69% |
| 60–70° | 86.0% (n=57) | 100.0% (n=35) | 81% |

The same paragraph's assertion that the low-angle fouls “all stay where they were” is also too strong. All-contact foul share at LA below −30° goes **11.3 → 15.8%**, at −10–10° **2.7 → 4.2%**, and at 10–25° **10.8 → 16.5%**. Early AD 12–20° foul/contact doubles **8.1 → 16.5%**, while whiff/swing worsens **42.7 → 51.7%**. Important deficits remain, but they do move.

**Suggested fix:** Add the steep well-timed bins and the early-contact/whiff tradeoff to the counterfactual assessment. Say that −30° matches the two kinematic summaries while overproducing some steep fouls; treat its use with v2 as a candidate to remeasure rather than a setting validated by the foul distributions. Replace “none ... move” with the actual residual deficits and changes.

## P1 — Lower angular speed does not explain the measured depth slope as claimed

**Location:** `docs/calibration/2026-11-13-foul-measures.md:70`, with the causal prediction repeated at line 104.

The regression **+1.04°/inch** is correct, but the explanation that its difference from the derived **1.45°/inch** is caused by contact away from peak speed is unsupported and quantitatively inadequate. All 4,357 measured intercepts in the default sample occur at **at least 93.8042% of peak angular speed**. This follows from the ±25 ms window and the Gaussian width `0.150 / 2.145966...`, and was also checked from every recorded swing. At the reported contact p1/p99 times, speed is about 98.5–99.1% of peak. These reductions do not explain a roughly **28%** slope deficit under the note's speed-based argument.

The measured slope is a population regression over different pitch locations, aims, swing planes, radii and misses. It is not a controlled derivative of attack direction with respect to depth on one nominal arc. The measurement does not establish either this explanation or the promise that late lag release will steepen that regression toward MLB's value.

**Suggested fix:** Keep the regression as an observation, remove the asserted speed explanation, and measure a fixed-pitch/fixed-aim timing sweep separately from the pooled regression. Describe the effect of v2 on this slope as something to test.

## P2 — Fair pull/opposite cohorts use a different spray definition from R13

**Location:** `sim/crates/bb_game/examples/foul_diag.rs:146`–152; resulting comparison in `docs/calibration/2026-11-13-foul-measures.md:66`.

R13 constructs fair-ball spray from `hc_x, hc_y`, i.e. a ball-location coordinate. The diagnostic instead assigns fair pull/opposite from the **initial velocity's** `b.spray_angle`, despite already having the terminal flight position available. Spin can carry a ball across the center line, so these are different cohorts.

In the default competitive sample, **88/2,160 fair contacts** switch sides between initial spray and terminal flight position. The printed cohorts are 1,196 pulled / 964 opposite, with AD SD 6.932 / 6.930°. Splitting by terminal position gives 1,128 / 1,032 and SD **6.884 / 6.845°**, printed as **6.9 / 6.8**, rather than the note's **6.9 / 6.9**. The larger conclusion that fair-contact AD is narrower remains valid. Unobstructed landing position is itself an approximation to Statcast's recorded hit coordinate, so that limitation should be explicit.

**Suggested fix:** Use a consistently defined ball-location spray for the R13 fair-outcome comparison, or explicitly label the current split as initial spray and avoid presenting it as the identical cohort definition. Refresh the affected numbers.

## P3 — The cage grid note confuses the current template with exactly −20°

**Location:** `docs/calibration/2026-11-13-foul-measures.md:83`; rounded output headings originate at `sim/crates/bb_cli/src/cage_lab.rs:80`–81.

The cage output has two columns both displayed as “−20°”: the actual current-template angle and exactly −20°. Their grid results differ. **33.0%** is the current-template result; the explicit **−20°** result is **32.8%**. The −30° and −35° values, 34.4% and 37.3%, reproduce.

**Suggested fix:** State **32.8 → 34.4 → 37.3%** for exactly −20/−30/−35°, or label the first value “current.” Print another decimal of angle in the cage headers to distinguish the first two columns.

## P3 — The games diagnostic prints the wrong aggregate MLB reference for its numerator

**Location:** `sim/crates/bb_game/examples/foul_in_games.rs:60`.

The tool correctly counts caught foul flies as foul, as its module comment explains, but prints **51** in the aggregate MLB column. R13 F1 gives **51.2% without caught fouls** and **52.5% including them**. These are different reference numerators. The probe counted 478 caught fouls in this run, so the inclusion is real.

**Suggested fix:** Print **52.5%** (or 53 at the current integer precision) for the caught-inclusive aggregate, and label the reference convention. This does not change the measured PA/game agreement.

## P3 — “Merged into main” is not the checked repository state

**Location:** `TASKS.md:12`.

At review time the feature branch points to `77aba6e`, while local `main` still points to **bcd7305**. `git merge-base --is-ancestor 77aba6e main` exits **1**. The new commit is therefore not merged into the repository's current main branch, contrary to “Resume here.”

**Suggested fix:** Correct the merge status when updating TASKS, or perform the merge through the normal owner-authorized workflow before marking it merged.

## Verification

All builds used `sim/target`; scratch source and logs were under `/tmp`. No source, test, spec or TASKS file was edited.

- Read `CLAUDE.md`, the full commit diff, R13, and the relevant swing, contact, pitch-resolution, PA, game and rules paths. No new outcome rolls, agent dependency on `bb_world`, event-sourcing mutation, or direct transcendental calls outside `bb_math::m` were found in the changed code. Degree conversions are arithmetic conversions, not transcendental calls.
- `cd sim && export PATH=$HOME/.cargo/bin:$PATH && cargo test --workspace --release`: **148 passed, 0 failed** across unit/integration/doc-test results.
- `cargo run --release -p bb_game --example foul_diag -- 3000`: completed; **4,381 swings, 2,835 contacts, 4,357 competitive swings**, overall foul/contact **23.7%**. Checked the note's default tables, quantiles, regressions, height/type splits, early/late splits and landing shares against output. Numerical transcription generally agrees; findings above concern interpretation and the listed exceptions.
- The same command with `-- 3000 - - - -30`: completed; **27.5%** overall foul/contact, **31.9°** mean tilt, **−27.5°** median VBA. Also ran `-- 3000 - - - -35`: **30.3%**, **36.6°**, **−32.4°**. The note's counterfactual table reproduces, including K/contact and LA summaries.
- `cargo run --release -p bb_game --example foul_in_games -- 128`: completed; **9,333 contacts**, **2,106 PA-model fouls (22.5651%)**, **2,136 game fouls (22.8865%)**, **73** game-foul/PA-fair and **43** game-fair/PA-foul disagreements; no undecided contacts. The largest bin difference is **38/1,112 = 3.42 points** below −30°, supporting the reported rounded 3.4-point bound.
- Scratch game probe over the same 128 seeds: **478 caught foul flies, 392 home runs, 0 play timeouts**. Catches emit `FairFoulDecided` even for foul flies; home runs emit `FairFoulDecided { fair: true }`; both are correctly included by the example. A pending play timeout also emits a forced fair decision (`bb_rules/src/engine.rs:1025`), which this diagnostic would count as ordinary fair rather than “undecided.” That fallback did not contaminate this sample; future samples should report it separately.
- `cargo run --release -p bb_cli -- cage-map --type FF`: completed; the reported −4 cm undercut and +4 cm topped-ball examples reproduce, as do the counterfactual grid results except the current-versus-exactly−20° attribution above.
- Compiled `/tmp/foul_review_probe.rs` against the workspace release libraries with `rustc`, producing `sim/target/release/foul_review_probe`. The default PA probe re-resolved **all 4,357 competitive swings with zero exact mismatches**, checking contact/miss plus contact time, post-collision ball state, EV, LA and spray. `pa.rs` uses `pitch_flight` followed by `resolve_flight`; `resolve_pitch` uses that same flight constructor, so reconstruction is equivalent here. The diagnostic's own assertion checks only EV/LA for contact and presence/absence for misses, but the stronger probe confirms its claim for this sample.
- The same probe verified raw/clamped tip projection and transverse gaps, the extended-barrel detection counterfactual, and the LA aggregation reported above. Searching the two additional adjacent windows, covering nominal ±50 ms on the existing pitch trajectory, recovered **no additional contacts**; there were no unreported misses at a trajectory endpoint. Window-classified misses had minimum/median/maximum surface gaps **0.167 / 0.706 / 3.678 m**. Thus “only far misses” is supported for this sample by a direct probe, not merely by the detected-contact quantiles.
- Verified the measures' geometry: AD uses `atan2(-side * vx, vy)` and is pull-positive for either hand; AA is velocity elevation; +Y depth is out front; VBA is knob-to-tip axis elevation. For this fixed-plane swing, `acos(abs(n.z))` is the plane's angle to the ground. Integrating sweet-spot speed over the preceding 150 ms measures the requested path length; the fixed plane also makes the last-40-ms tilt identical to its instantaneous plane tilt. Hardcoded R13 references otherwise match, with the aggregate caught-foul exception above.
- Counterfactual geometry probe mirrored RHB/LHB aims and poses at −20/−30/−35° over four times: **zero mirror error**. The new test checks geometric properties and the resulting planned axis against the requested angle; it does not simply compare `bat_angle(with_bat_angle(...))` to itself. No tautological-test finding.
- Reconstructed the pre-refactor inline `plan_perfect` calculation in scratch code and compared complete `IntendedSwing` values for **72 cases** (both hands, FF/SL/CH/CU, three horizontal and three vertical targets): **zero unequal plans**. Inspection also confirms the extracted operations and evaluation order are unchanged.
- The first attempt to link `foul_in_games` failed because Zig tried to write its cache under read-only `~/.cache/zig`. Reran successfully with `ZIG_GLOBAL_CACHE_DIR` and `ZIG_LOCAL_CACHE_DIR` under `sim/target`; no administrator action or source change was needed.

## Resolution

Open.
