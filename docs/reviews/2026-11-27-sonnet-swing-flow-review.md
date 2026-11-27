# Review: swing-flow (a real hitter's recorded swing on the figure), 2026-11-27, Sonnet (rev-swing-flow)

Range `main..work/swing-flow`, 10 commits (ce81363 at the tip). Nothing in it can change a game: a Python exporter, its tests, `bin/body-swing.sh`, the viewer's `ABbBodySwing` (a separate actor from the game's batter), docs. **No P1 found.** The mapping is what the note says, the checks are mostly independent of the exporter, and the force-driven path is unchanged.

## Findings

### P2

1. **`test_real_swing.py` is skipped silently where the data are absent, and that is everywhere but this machine.** `analysis/research/body/test_real_swing.py:33` (`HAVE`) and the `skipUnless` on the class: with `BB_OBP_DIR=/nonexistent` it prints `Ran 7 tests ... OK (skipped=7)`. On main (no data) the whole mapping check reports OK. This is by design (licence), so it is not a bug, but "tests pass" for this change means "passes on one laptop". It also needs `uv run --with numpy --with c3d`: plain `python3 -m unittest` (the command in the review job and in AGENTS.md style) fails with `ModuleNotFoundError: numpy`, an ERROR and not a skip, so anyone running the analysis tests with system Python sees a failure. Fix: guard the `import numpy` and `import real_swing` so a missing module also skips, and say so in the docstring.
2. **The independent checks cover the limbs, not the trunk, hips or head.** `test_joints_are_his_markers` (line 98) checks knees, ankles, elbows and wrists against midpoints of raw skin-marker pairs. The pieces the note flags as constructed, not his own, have no independent check: the **hip centres** (Harrington regression, `real_swing.py`), the **lumbar** joint, the **head's** correction (17° up, 7° round, from the calibration trial) and the feet's heading/pitch. The shoulders are checked only against the dataset's own shoulder centres (within 25 mm; the note says 22 mm). The only hint for the hips is `test_scale_is_the_model_leg`, which uses them in the leg length, so it restates part of the exporter's own chain. How I know: read the test's `test_*` list against the mapping in the docstring. A cheap independent check for the hips: pelvis-marker midpoint (the ASIS/PSIS markers read raw) against the hip midpoint within a few cm; and the head's face line against the headband's raw four markers plus the stated offset, rather than the exporter's own output.
3. **`test_feet_still_while_planted` reads the exporter's `contacts` columns, not the raw heel/toe markers.** It is a physical invariant (no skating), so it is not tautological, but a mistake in the exporter's sole placement (the "heel under the heel marker by its flat height") would pass it. The raw LHEL/LTOE-type markers would be the independent reference. Low risk: I saw no skating in the sheets.

### P3

4. **Licence: nothing committed is dataset rows, but the note carries one hitter's identifying facts.** `git log -p main..HEAD`: no `.swing`, `.csv`, `.c3d`, picture or table of joint values. The only numeric row my search found is plate geometry in `real_swing_draw.py`. The note (§1) does quote one hitter's metadata row (height, weight, level, bat, bat speed, exit velocity, launch, pitch zone, the C3D's file path with the participant ids) and event times (−620, −178, −131 ms). Facts from an NC-SA dataset, attributed, a few numbers; I judge it acceptable but it is the owner's call under Q68. `analysis/research/body/swings/real/478_5.swing` and `swings/swing-research/` exist locally and are untracked/ignored (`git status` clean apart from `JOB.md`, `REVIEW-swing-flow.md`).
5. **Stale text in `docs/player-visuals.md:498`:** still says `-BbBodySwingShots=<ms>` runs "to 60 ms after contact"; §14.1 (line 517) says a recorded file runs to its last sample. Add "(a recorded swing: to its last sample, §14.1)" there.
6. **TASKS.md "Done, to review, merge: ... Then a Sonnet review"** is stale once this review is committed; the Coordinator will rewrite it. TASKS.md is 30,689 bytes (the diff replaced a line, no growth).
7. **The viewer's HUD still labels the key help "Left/Right step one sample"** for the force-driven file too (it steps 1 ms there). Cosmetic.
8. **`bin/body-swing.sh --real`** depends on `uv` being on `$HOME/.local/bin` and on the data being fetched; when the data are absent the error comes from `real_swing.py`. Fine, noted so the owner is not surprised. Unquoted `${EXTRA[@]}` with an empty array is fine under `set -u` only on bash 4.4+ (this machine's is).

## Answers to the review's questions

1. **Is it his motion?** Yes, as far as I can check. The docstring's mapping (lab +x → sim +Y, lab +y → sim −X, +z up, no mirror; one uniform scale s = 1.035 about the floor from the model's leg over his; one x,y translation by HitTrax; his own joint centres each sample; bat unscaled) is what `real_swing.py` does. `grep` for smooth/filter/clamp/interp/fit finds none applied to the motion (line 55 of the docstring says "Nothing is smoothed, fitted or made up beyond this"); `np.max/min` hits are checks and event picks. The constructed pieces are said in the note: the lumbar joint, the hips (Harrington), the shoulders squared to the trunk (22 mm at most), the feet's roll dropped, the ball of the foot by the shoe's proportions, the head corrected by the headband offset. Those are disclosed. One thing the note says only in passing: the figure's segment lengths are **his, varying each sample** (the viewer stretches the pieces), so it is "his joints on a stretchy mannequin", not "the model's lengths" as the job asked; the note says why (hands unreachable by 11 cm otherwise). I agree with that choice.
2. **Tests.** `uv run -q --with numpy --with c3d python3 -m unittest analysis/research/body/test_real_swing.py analysis/research/body/test_swing_export.py` → `Ran 13 tests ... OK` (7 new, 6 old). Mostly independent: raw C3D markers, HitTrax's contact height (within 4 cm; 1.8 cm), the README's frame written again, de Leva legs, the right-hander's box. Gaps in P2-2 and P2-3. Absent data: 7 skipped, OK (P2-1).
3. **Viewer.** No Unreal process was running (`ps` showed none), so I ran it (touched `BbBodySwing.cpp`, one run at a time, `-RenderOffscreen`, `taskset`/`nice`):

   | file | check | result |
   |---|---|---|
   | force-driven `practice3d_s12_posture.swing` | none | PASS 4.21 mm, 1.54°, right, exit 0 |
   | same | mirror / nostretch / twist | FAIL exit 1 (handedness WRONG 2.7 m; 36.5 mm; 90.0°) |
   | recorded `478_5.swing` | none | PASS 3.68 mm, 0.535°, right, exit 0 |
   | same | mirror / nostretch / twist | FAIL exit 1 (handedness WRONG 2.9 m; 82.6 mm; 90.0°) |

   The force-driven numbers equal the note's old figures (shoulders 4.2 mm, feet 1.54°). C++ diff: the force-driven file has no `head` body (`BODY_ORDER` in `swing_export.py:36`), so `HeadBody == INDEX_NONE` and the head path is skipped; new fields default to the old behaviour (`MissCm = -1` is only a "no miss" marker for the recorded file and the force-driven file always writes one); nothing touches the game's own batter, the sim or the FFI.
4. **Licence.** See P3-4.
5. **Look at it.** `strip_open.png` and `full-v3/sheet_20.png` (−260 to −200 ms, stance and the start of the stride, the figure beside his stick figures from all three sides): the figure's pose is his stick figure's in every row, the feet stay put, the bat is where his is. The strip (foot plant to the follow-through) reads as a leg-lift hitter's swing: the lead leg lifts and strides closed, the hands stay back, the hips open, the barrel comes under the hands, a low pitch met in front, the bat wraps over the lead shoulder. The note's §3 matches what I see. Things a baseball person would add that §3 does not say: the figure's torso is thicker and more upright than his stick figure suggests (a mannequin stretched to his joints); the hands and wrists, being blobs, hide the grip, so the top hand's roll over at contact cannot be judged; the lead leg in the strip looks very stiff and straight at foot plant for a stride this long (his is, by his markers). None of it is a fault of the mapping. The note already owns the real limits: met 0.42 m in front, a 61 mph machine pitch's timing, the finish cut at +500 ms.
6. **Docs.** `python3 bin/check-docs.py` exits 1, but only on the two untracked job files (`REVIEW-swing-flow.md:7` and `:9`: this file's path did not exist yet and `BbBodySwing.cpp/.h` is not a path). After this file exists only the `.cpp/.h` line of the untracked `REVIEW-swing-flow.md` can fail; it is not part of the commit range. Stale lines: P3-5, P3-6.

## Verification (what I ran)

- `git log --oneline main..HEAD` (10 commits), `git diff --stat` (10 files, no data files); `git log -p main..HEAD | grep` for `.swing`/`.csv`/`.c3d` additions (only text in the docs and the code that names them) and for rows of decimals (one: the plate polygon).
- `uv run -q --with numpy --with c3d python3 -m unittest ...test_real_swing.py ...test_swing_export.py` → 13 tests OK; plain `python3` → `ModuleNotFoundError: numpy`; `BB_OBP_DIR=/nonexistent` → 7 skipped.
- Unreal: the eight check runs in the table above, each `exit` code as listed.
- Read `strip_open.png`, `full-v3/sheet_20.png`, note §1–§4, the C++ and shell diffs.
- Not done: I did not re-run the 630-picture set (11 minutes) nor re-fetch the dataset's README to confirm the lab frame word for word; the frame is instead confirmed by `test_right_hander_in_the_box` (right-hander on the third-base side facing the plate, lead foot to the pitcher).

## Resolution

Resolved by the Coordinator, 2026-11-27. Reviewer: Sonnet at medium effort (the change cannot alter a game).

- **P2-1: fixed** (b6bc97f): a missing numpy or c3d skips `test_real_swing.py` with its reason; the docstring says how to run it and that a skip says nothing about the mapping.
- **P2-2: fixed** (66543a1): the hips against a second regression from the raw pelvis markers, the head's turn against its raw markers; note 12 lists what stays unchecked (the lumbar joint, the shoulders beyond the dataset's own centres, the level calibration stance, the feet's roll and ball point).
- **P2-3: fixed** (520b439): the drawn feet against his raw heel and toe markers.
- **P3-5: fixed** (5a8c549). **P3-6: fixed** by the Coordinator's handoff.
- **P3-4: with the owner as Q68** (the note quotes one hitter's metadata from a non-commercial dataset).
- **P3-7, P3-8: deferred** (a help label and a note on `uv`; cosmetic).
- Found at the pre-merge check: the docs check failed on a checkout with no export (a backticked git-ignored path); fixed (90f28ff).
- No re-review of the fixes: small and test-covered. Pre-merge on main: 16 tests (10 new pass, 6 old skip as before), the viewer's check passes on the recorded swing (3.68 mm, 0.54 deg) and on the force-driven one (4.21 mm, 1.54 deg).
