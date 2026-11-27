# Review: the body's practised swing in the viewer (PLAN D43), `work/body-viewer`, 5a3ed26..a97e510

Reviewer: Sonnet 5.5 (independent; wrote none of the change). Scope: the brief `REVIEW-body-viewer.md`.

## Verdict

Nothing under `sim/` or `analysis/oracle/` changed. With no `-BbBodySwing` switch the game paths are unchanged (every changed line read, saved-play check and the standard pictures run). The viewer draws the file as written: linear interpolation of positions, slerp of turns, nothing else is done to the data. The check passes on rung 4 (largest 4.2 mm). `--tree` wrote nothing into `../swing-research`. The findings below are about what the check can and cannot prove, one staleness hole in the launcher, and rough edges in its error messages. No P1.

## Findings

### P2-1. The swing is not re-exported when the body code changes, only when the plan or the exporter does
`bin/body-swing.sh:35`: `[ "$PLAN" -nt "$OUT" ] || [ "$EXPORTER" -nt "$OUT" ]`. The swing depends on `practice3d.py`, `probe3d.py`, `body3d/*` and the probe's Rust source of the tree it runs on (`--tree` makes this explicit: "run on that checkout's body code"). A change to the body model with the same kept plan leaves the old `.swing` in place and the owner watches the old model's motion, with a header that still names the old commit (I saw `commit da3c14b` in the viewer's top line while HEAD was a97e510). The owner judges "does the model swing like a hitter" by this picture, so a stale one is a wrong answer shown with confidence. Fix: re-export when any body-code file in the tree is newer than the output, or when the header's `commit`/`tree` differ from the tree's HEAD, or simply always (the export takes about 2 s; 1.4 s measured with a warm probe).

### P2-2. The output name is the plan's file name only, so two checkouts' same-named plans share one `.swing`
`bin/body-swing.sh:34`. `practice3d_s11_rung4.json` exists in both this tree and `../swing-research`. `bin/body-swing.sh --tree ../swing-research analysis/…/practice3d_s11_rung4.json` after a run on this tree finds `$OUT` newer than the plan and shows this tree's swing, labelled as the other tree's request. Same cure as P2-1 (check the header's `tree`), or put the tree's name in the file name (the probe directory already does).

### P2-3. `-BbBodySwingCheck` verifies the mapping's self-consistency, not the axes or the turns
`BbBodySwing.cpp:705–770`. The check compares each piece's transformed carried point against `ToCm(File.JointPos(...))`, the same `ToCm` that `Pose` uses to place the pieces, so a slip in `ToCm` (axis swap, mirror) passes by construction. The body's orientation (`AxisUE`, the `SquareTo` of each segment's forward axis, the foot's and the bat's roll) is not read at all: a wrong turn of a limb about its own axis, or a mirrored forward, leaves every joint where it was and passes. Also the many exactly 0.0000 mm results (knee, ankle, elbow, wrist, knob, grips) are zero because the piece's origin is mapped to that joint; they say nothing. What it does cover is real: that the pieces' lengths and widths are stretched to the model's, that the far joints land where the file's do, and that the 1 cm limit is not exceeded by the shoulders' non-perpendicular offset. So the claim "the owner's picture is as computed" rests on the pictures I looked at (below), not on the check, and §14 should not read as if the check settles axis and handedness. Suggested addition: an independent read of each body's turn (the drawn segment's forward axis against the file's quaternion rotated through the sim→Unreal map written out separately in the check) and a mirror test (a known asymmetric point, e.g. the lead foot's x in the file, against the drawn foot's side of the pelvis).

### P2-4. The planted fault in §14 cannot be reproduced
`docs/player-visuals.md` §14 (the check paragraph) reports a planted fault ("the limbs left at the mannequin's lengths": knees 1.3 cm, ankles 3.6 cm, shoulders 6.2 cm, exit 1) but there is no switch or test in the repository that plants it; it was a temporary source edit. I did not edit source, so I could not reproduce it. By reading, the numbers are plausible (the length ratios in the §14 list, thigh 44.6/43.3 etc., are of that size for the knees and ankles), but a fault that cannot be re-run protects nothing. A fault switch (`-BbBodySwingCheckFault=nostretch`) that must make the check exit 1 would turn this into a regression test.

### P3-1. The check's limit is 1 cm, 2.4 times the largest honest error and 10,000 times the typical one
`CheckLimitCm` = 1.0. Fine for "visibly wrong", but it would hide a 9 mm slip. After P2-3's additions, a tighter limit (say 6 mm, above the shoulders' 4.2 mm) is possible. Low priority.

### P3-2. Launcher messages
Tried each case:
- no kept plan in a tree: clear message, exit 1.
- `--tree` with no argument: `bin/body-swing.sh: line 19: $2: unbound variable` (set -u).
- `--tree` to a missing directory: `cd: …: No such file or directory`, exit 1 (acceptable, plain).
- plan file that does not exist: a Python traceback (`FileNotFoundError`), not a message.
- no Unreal engine (`UE_ROOT=/nonexistent`): the swing is exported, then `client/run.sh: line 17: /nonexistent/Engine/Build/BatchFiles/Linux/Build.sh: No such file or directory` after a cargo build; says what is missing but only through the shell's words. Better to check `$UE` first with a sentence (and "install it at ~/UnrealEngine/5.8.4").
- [4]'s curves missing: the exporter fails inside `common.py` with `FileNotFoundError: …/lead_grf_z.csv`, no sentence telling the owner to run `analysis/research/fortenbaugh_appendix_e.py`. (Tested with `BB_FORT_CSV=/nonexistent`; I did not remove the real directory for long.)
- viewer arguments (`-BbBodySwingShots`) pass through, and the repository path's space is handled: `-BbBodySwing="$OUT"` reached the engine and opened the file (run twice with the space in the path).
- a plan newer than its export: re-exports (the `-nt` test, working); but see P2-1/P2-2 for what it misses.

### P3-3. `newest_plan` prefers an older committed plan over a newer uncommitted one of the same session
`swing_export.py:45–52`: the key is `(session, last-commit time, mtime)`, and an uncommitted file has commit time 0, so within the highest session any committed file beats a newly kept, not yet committed one. §14 states the rule ("the one last committed"), so the code is true to it, but a research worker's fresh plan in the other tree is exactly the uncommitted case the `--tree` mode is for. Either say so in §14 or rank uncommitted files (by mtime) first.

### P3-4. Exporter tests that restate the exporter
`test_swing_export.py`: segment lengths, hands on the handle, plan-file figures and times are independent (the engine's joints and the practice report's own numbers) and are good. `test_ball_line` checks the ball's speed with the literal 37.98 and the meeting point from the file's own header against the file's rows: it checks the exporter's `ball_at` against the exporter's `ball_v`, which is a restatement. Harmless; the plan's pitch speed would be the independent source if the plan file records it. Also `test_feet_on_the_ground_at_the_start` has a 1 cm window around z = 0 for loaded points. The class is skipped silently when [4]'s curves are absent: say so in the skip reason (it does) and make sure the ledger/CI that runs it has them.

### P3-5. Wording in the header and doc
`swing_export.py` header says "the checking model"; §14 says the same. Fine. `AGENTS.md` carries the new switches in the "Check visual changes" bullet (the brief called it the toolchain line); OK. The `BB_BODYCHECK` header prints `limit_mm=10.0` while §14 and AGENTS say "1 cm": the same, but pick one unit.

## What I read and what I looked at

**Q1, game paths.** The `BbDirector.cpp` hunk is inside `if (SavedPlayArgument(Cmd, SwingFile, "-BbBodySwing="))`: with no switch it is skipped. `-BbBodySwing=` does not match `-BbBodySwingShots` (needs the `=`), and the include and header fields are inert. `Tick`'s `if (BodySwing) return;` is false for a game. `BbGameMode.cpp`: the HUD's new branch runs only when `BodyLines.Num() > 0`, which only the swing viewer fills. `BbFigure`: `RefCs.Add(name, Cs[I])` adds one map entry per bone while building; no behaviour changes. `-BbSavedPlayCheck` passes (`cases=39`, `commit=a97e510`); `-BbShots=6` made six pictures; `shot_00` (a ball, 79 mph curveball, bug, ball box) and `shot_03` (87 mph slider, strike zone panel) look as a game should.

**Q2, is the motion drawn as computed.** In `Sample` the time is clamped to the file's first and last instants (holding the first/last pose, not inventing a pose; the screen says so, I saw "The model's last computed pose (it runs here)" at +179.5 ms) and positions are lerped, turns slerped, bat and ball lerped; no smoothing, IK, clamp or pinned foot in `Pose`. What `Pose` does that is not interpolation, all disclosed in §14: each piece is stretched along and across its segment to the model's lengths (the mannequin is 1.83 m, the model's batter 1.84 m by different proportions), the feet are mapped to the soles' plane and ball-of-foot contact points, the head and neck ride the trunk, the bat is drawn from knob to tip scaled by `Z.Size()/BatCm`, with its roll from the body's x axis. The `placed` shift is one translation in x and y (`mv` in the exporter). Axes: `ToCm` = (100 Y, 100 X, 100 Z + ground), `AxisUE` swaps x and y: consistent with the `ue = 100 × (y, x, z)` rule; the pictures look like a right-handed batter in the third-base-side box facing the plate with the open-side view from first base (I judged handedness by eye from the stance and bat position only, not by measuring a marked point) (see P2-3 on what holds that, which is only the pictures). Interpolation error: tip within 0.24 mm of the unkept steps (the file's own measure, and the exporter refuses over 1 cm).

Pictures read (rung 4, `client/Saved/Shots/bodyswing_*`):
1. start, open side: wide stance, both feet on the dirt, bat in both hands behind the head, bat up; looks like a hitter's set-up; HUD text top left readable, controls line at the bottom is faint and small.
2. start, pitcher side: same, from the front; the bat angled up and back over the rear shoulder.
3. −50 ms, open side: bat nearly horizontal across the chest, hands together in front of the trunk, head down, hips turned; plausible.
4. contact, open side: the bat is vertical in front of the chest, pointing down, hands high near the chest. A real contact does not look like this (the barrel is nearly straight down; §14 records 57° below level and says so for the owner to judge). The ball is not visible (it is inside the barrel, as §14 says).
5. contact, pitcher side: the barrel points down and toward first base side from the hands at shoulder height; the trunk bent; the back foot up on the toe. Also the model's, not the viewer's, but the owner should see it before trusting any of it.
6. end (+179.5 ms), open side: folded forward at the waist, trunk near horizontal, arms and bat hanging; the sign at the top says it is the last computed pose and no batted ball is drawn.
The figure's pieces (helmet, uniform, shoes, white tape on the bat's handle, hands) are present and joined; I saw no gaps at the joints and no limb through the ground.

**Q2, the check.** Output below. It reads the drawn components' transforms with `GetComponentTransform()` plus carried local points from the build, then subtracts the file's positions through `ToCm`: independent of `Pose`'s arithmetic, not of `ToCm` (P2-3).

**Q3, the exporter.** `Ctx(**options)`, `ctx.u_of(plan)`, one `ctx.run(u, every=1)` subsampled with `[::every]` (every = 4 at 0.25 ms), `ctx.contact(recs)`: the same calls as `practice3d.report` (which calls `run` with `every = 1 ms / h`). Its figures equal the plan file's: rung 4: 21.11 m/s (plan 21.112571643635142), miss 0.04 cm; session 12's posture plan 26.25 m/s against 26.245632 in its file, miss 0.02 cm. No state is altered (it only reads the recorded states; the ball and tip-error loops read `fine`). `ctx.failed_after` is carried to the `end` event.

**Q4, `--tree` read-only.** Before and after exporting `practice3d_s12_posture.json --tree ../swing-research`, I listed every file in `../swing-research` outside `.git` and `sim/target` with its mtime and size (`find -printf`): identical, and `git status --short` identical (empty). The `__pycache__` directories there already existed (they are gitignored and the other worker's own); none changed. The exporter sets `sys.dont_write_bytecode` for another tree; the probe is built into this tree's `sim/target/codex-probes/body3d-swing-research`. I did not run any of that tree's searches.

**Q5, §14's rule.** "the newest kept swing = of the highest session, the one last committed" is what `newest_plan` does (P3-3 for the uncommitted case).

**Q6, docs.** §14's controls, file layout, and figures match the code and my runs (451 samples, −271 to +179 ms, 16,236 joint readings, shoulders 4.2 mm, tip 0.24 mm). Stale items: none found beyond P2-4 and the unit mixing in P3-5. The new switches are in AGENTS.md.

## Verification

All commands from the repository root of this worktree (`/home/natex/Projects/3D Baseball/.claude/worktrees/body-research`); Unreal and Python runs under `taskset -c 8-15 nice`, `CARGO_BUILD_JOBS=2`, one at a time.

- `git diff --stat 5a3ed26..HEAD -- sim analysis/oracle`: empty (no sim or oracle change).
- `python3 -m unittest analysis/research/body/test_swing_export.py`: `Ran 6 tests ... OK` (1.8 s). I did not run the other body tests: the change adds no code they import; the exporter only imports them.
- `python3 swing_export.py practice3d_s12_posture.json --tree ../swing-research --out <scratch>.swing`: `wrote …: 451 samples, 275 ms before contact to 175 ms after; sweet spot 26.25 m/s (58.7 mph), the plan's file says 26.2456; missed by 0.02 cm; tip within 0.23 mm; joints' halves within 0.00012 m`; before/after listing of `../swing-research` identical; its `git status --short` identical.
- `client/run.sh 7 -RenderOffscreen -BbSavedPlayCheck`: exit 0, `BB_SAVED_PLAY_CHECK PASS commit=a97e510 cases=39` (the Error lines in the log are the check's own planted bad cases).
- `client/run.sh 7 -RenderOffscreen -BbShots=6`: exit 0, six pictures.
- `client/run.sh 7 -RenderOffscreen -BbBodySwing="<…/swings/practice3d_s11_rung4.swing>" -BbBodySwingShots`: exit 0, sixteen pictures, read above.
- `… -BbBodySwingCheck`: exit 0, `BB_BODYCHECK file=practice3d_s11_rung4.swing samples=451 joint_readings=16236 limit_mm=10.0`;
  joint lumbar 0.7317 mm (−19.5 ms), hip_L 0.0917, knee_L 0.0000, ankle_L 0.0000, hip_R 0.0988, knee_R 0.0000, ankle_R 0.0000, shoulder_L 4.2013 (−158.5 ms), elbow_L 0.0000, wrist_L 0.0000, shoulder_R 4.2074 (+30.5 ms), elbow_R 0.0000, wrist_R 0.0000; bat knob 0.0000, sweet_spot 0.0010; hand_on_handle L 0.0000 R 0.0000; sole_under_ball L 0.21 R 0.21 (information); `result PASS largest_mm 4.2074 limit_mm 10.0`.
- Planted fault: **not run** (needs a source edit; none allowed; see P2-4).
- `bin/body-swing.sh --tree <empty dir>`: exit 1 with message; `--tree <missing>`: `cd` error exit 1; `--tree` alone: unbound variable; `bin/body-swing.sh /nonexistent/p.json` and a missing plan in an existing directory: Python tracebacks; `UE_ROOT=/nonexistent bin/body-swing.sh …rung4.json`: re-exported (exporter newer than the file), then `Build.sh: No such file or directory`; `BB_FORT_CSV=/nonexistent` exporter run: `FileNotFoundError …/lead_grf_z.csv`.
- Caution I took and one slip to report: while testing the missing-curves case I moved `sim/target/research-dl` aside for a moment (without having first meant to) and moved it straight back; it is gitignored data and `ls sim/target/research-dl` shows `4` again. The exporter was not run in that window. The `.swing` files under `analysis/research/body/swings/` (gitignored) were re-made by my runs; nothing tracked changed (`git status --short`: only the untracked `REVIEW-body-viewer.md` brief and this file).
- Background shells: none left running.

## Resolution

Open.
