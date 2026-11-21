# Review: the base map in the score bug (`work/base-map`, `904c1ea..127f3e9`)

Reviewer: Sonnet (medium effort), who wrote none of the change. Date 2026-11-21.
Range: three commits (`7023f2c` WIP, `b808f4a`, `127f3e9`). Files: `client/Source/BaseballClient/BbBaseMap.h` (new), `BbDirector.cpp/.h`, `BbHud.cpp/.h`, `docs/hud.md`, `observations.md`, `TASKS.md`.

## Summary

The change does what the owner asked. Nothing under `sim/` changes and no ledger row is due. The map ends every play on the official result, and both seeds I ran pass the check. Two things can still make it wrong on screen: the 7 m lit zone, which darkens a base while its runner is still on it, and foul detection that trusts `pitch_call` 4 or 5. The check cannot catch either. The rest is minor.

## Findings

### P1

None.

### P2

**P2-1. A lit base goes dark while its runner is still on it (overrun, long lead), then lights again.** `BbBaseMap.h:19` and `:117-124`.
The light is `distance to the base <= LitZoneM` (7 m). In `-BbBaseMapShots=3`, play 24 (seed 7, a force at second; the batter reached first), the sequence of the four pictures (crops in `/tmp/rev-strip-01.png`) is:
- 60% of the play (`basemap_01_3.png`): the batter's dot is clearly past first base (an overrun of more than 7 m) and first is **unlit**, with outs at two.
- 85% (`basemap_01_4.png`): he is back on the bag and first is lit again.

The official state is "runner on first" for the whole stretch, so the light flickers off and on while the dot sits beside the base. This is the "dot and lit base disagree" case the ruling meant to prevent. The same will happen with a lead longer than 7 m (a runner taking a secondary lead, once leads and pickoffs, A6/A8, are built), and a runner rounding a base lights it for the moment he passes within 7 m.
The check cannot see it: it compares the lit set only at t=0 and at the end, and counts how often the set changes. It never asserts that the lit set is stable while nobody leaves a base.
Suggested fix: pick the lit base from the runner's path state, not from a radius alone. Options:
- Keep a runner "on" the base he last stood on until he is nearer the next one than 7 m; or
- Use hysteresis (light at 7 m, release at about 12 m or only once he heads toward the next base).

Also add a check assertion: for each play, a base lit at the end was lit continuously from the first sample at which a runner reached it.

**P2-2. Foul detection trusts `pitch_call` 4 and 5, which is a guess, not the ruling.** `BbBaseMap.h:44-48`.
`describe.rs:40-45` and the sim's own comment at `pa.rs:597-605` say the pitch call is a guess from where the ball first lands; the rules decide fair or foul on the field (`FairFoulDecided`). The summary is built from that ruling: it is exactly "Foul" whenever the sim ruled foul and the plate appearance goes on. So `IsFoul` is wrong in two directions:
- `pitch_call == 4` for a ball that first lands foul and then rolls fair (a fair ball in the rules). The map would freeze at the pre-pitch runners, show no batter or runner dots, and fail the end-of-play equality.
- `pitch_call == 5` or `4` on a caught foul fly (an out) or a foul tip that is strike three. The summary is not "Foul", but the map treats it as a foul, so a runner who tags up and scores on a caught foul fly stays on his base on the map until the end, then jumps. The plate-appearance-ending foul cases have no recorded test.

Neither occurred in 196 + 215 plays (seeds 7 and 12), so it is rare, not absent. Separately, the third `||` term (the string ends in "Foul") has no sim output behind it: `describe.rs` makes no summary other than the exact string "Foul" that ends in "Foul". It is dead code, and the exact-match term already covers the case.
Suggested fix: decide by the summary alone (`Strcmp(summary, "Foul") == 0`), the sim's own ruling in the only structured place the client has it today, and drop the `pitch_call` and suffix terms. If a structured field is wanted, `BbPlayInfo` needs a `foul` flag from `bb_ffi`; that is a sim-side change, so file it as a follow-up and note it in `hud.md`.
The director already does the same text match at `BbDirector.cpp:2393`; use one helper for both.

### P3

**P3-1. Stale comment: `BbDirector.h` (the `RunBaseMapCheck` doc) says it reads `BbBaseMap::Occupied`.** There is no such function; it is `BbBaseMap::Read`.

**P3-2. `-BbBaseMapCheck` is only partly independent.** `BbDirector.cpp` `RunBaseMapCheck`:
- Good: it uses the timeline's own runner positions (`F.actors[].pos`) to check the dots to scale, and the sim's recorded `Info.bases` for the end of every play; the no-jump bound is a physical speed limit (12 m/s).
- Too weak:
  - The "dot at the right distance from home" test recomputes the same affine map from the same actor position, so it mostly restates `ToDiamond`. The fixed-point asserts on the base corners are the only independent part.
  - `Pop(M.Lit) <= M.Dots.Num()` and the end-state equality pass for every foul play by construction (the foul branch just replays `BasesBefore`).
  - It never asserts when the batter appears or hides (strikeout, walk, hit by pitch, home run), that an out runner vanishes at the call time, that no dot shows for a runner who has scored, or that the lit set is stable (see P2-1).
  - It does not exercise the director's integration: `Before.bases`, `Before.outs`, the playback clock `T`, replay or a reopened saved play. A wrong `Before` or the saved path would pass.
- The checks run on the plays it generates; I saw 45,745 and 68,656 cases with no failures.

**P3-3. The dot hides the base diamond it sits on.** `BbHud.cpp:129-135`. The 8-unit white dot with a dark outline covers a 12 px diamond at the 1x bug size (`basemap_00_2.png`). With a lit diamond the gold ring shows round the dot, which reads fine; on an unlit diamond (`basemap_01_3.png`) the dot nearly covers the base and the "is this base on" cue is only the gold. Legible enough; a 6-unit dot, or drawing it hollow, would leave the base visible. Not a defect.

**P3-4. `hud.md:38` leaves an orphan sentence.** The base-map paragraph ends "Under it, slim lines for the pitch's speed..." which is about the bug as a whole (and was placed before the paragraph in the old text), so it now reads as if the slim lines were part of the base-map description. Move it back to the bug bullet. Also the description "a runner leaving for the next base puts the light out" is the behaviour behind P2-1; update it when P2-1 is fixed.

**P3-5. `FMap` allocates a `TArray` each tick** (`BbDirector.cpp` Tick: a local `BbBaseMap::FMap`, then `Bug.Dots` copied). At most three dots, one small allocation a frame: negligible, but `Read` could write into a reused member array.

**P3-6. Comment layout in `BbBaseMap.h` `Read`:** a bullet list is interrupted by the foul paragraph and then resumes ("- everyone once the third out..."), so the third-out bullet reads as part of the foul paragraph. Cosmetic.

## Answers to the ranked questions

1. **Does anything under `sim/` change?** No. `git diff main..work/base-map -- sim` is empty (0 lines). Client-only, so no timeline digest can change, and no ledger row is due.
2. **Is the map right at every moment?**
   - End of play: yes, the check compares the end of every play against the sim's `bases` for 196 and 215 plays.
   - Between plays: the director passes `Before.bases` and `Before.outs`, and `Before` is the previous `Info`, which has the new half's bases and outs already (0 and 0 after a third out), so the new half starts clear. The check's `bFresh` handling is redundant with this but harmless.
   - Third out: empty from the call; correct.
   - Batter: hidden until he is more than 1.8 m from the plate; so not shown on a strikeout (he never leaves), shown on a walk, hit by pitch and home run only after he leaves; the end-state count confirms it.
   - Out and scored runners: the out call hides the dot at `t_s`; a scorer is hidden by the home radius and the timeline keeps him there, since the end counts match.
   - Foul: P2-2.
   - Replay, pause, speed: the map reads the same `T` and `Frame` the figures use (`Tick`, `SampleDrawn`), so it is on the playback clock. Saved play: the `Sim` exists in saved mode, and `-BbSavedPlayCheck` passes (39 cases), though it does not draw the bug.
3. **The 7 m and 1.8 m radii.** Both are named constants (`LitZoneM`, `HomeHideM`), documented. The 7 m misfires as in P2-1; I did not find a case of two runners at one base. The 1.8 m radius did not misfire in 411 plays (box position is about 1 m from the apex); a batter stepping back more than 1.8 m for a hold-up swing would show for an instant (not seen).
4. **Is the check honest?** Partly; see P3-2.
5. **Pictures.** The dots and lit bases are legible at the bug's size (see P3-3). Play 22 (`basemap_00_2.png`) shows the runner on second, lit with a dot; play 24 shows the force at second and the flicker.
6. **Rules and code quality.** Game logic is in C++; the map is read once per tick on the game thread, as the figures are, with no stepping of anything new. Findings P3-1, P3-4, P3-5, P3-6. The `TASKS.md` edit (deleting the finished line) follows the rule at its top ("done is deleted"); the `observations.md` tick (`[x]` with a dated plain note) is in the owner's words and consistent with the code except that "no jump" has the P2-2 exception.

## Verification

All commands run in the worktree `/home/natex/Projects/3D Baseball/.claude/worktrees/saved-plays` at `127f3e9`. No `sim-cli games` was running (`pgrep -x sim-cli` empty) before each render.

| Command | Exit |
| --- | --- |
| `git diff main..work/base-map -- sim \| wc -l` (0 lines) | 0 |
| `Build.sh BaseballClientEditor Linux Development -project=... -MaxParallelActions=4` (up to date, 8 s) | 0 |
| `client/run.sh 7 -RenderOffscreen -BbBaseMapCheck`: PASS plays=196 with_runners=75 lit_set_changed_in=28 dot_samples=10821 cases=45745 failures=0 | 0 |
| `client/run.sh 12 -RenderOffscreen -BbBaseMapCheck`: PASS plays=215 with_runners=106 lit_set_changed_in=35 dot_samples=16706 cases=68656 failures=0 | 0 |
| `client/run.sh 7 -RenderOffscreen -BbSavedPlayCheck`: PASS commit=127f3e9 cases=39 | 0 |
| `client/run.sh 7 -RenderOffscreen -BbSpeed=10 -BbBaseMapShots=3` (12 PNGs, `basemap_00..02_1..4.png`) | 0 |
| `python3 bin/check-docs.py` | 0 |

Looked at: `basemap_00_2.png`, `basemap_01_3.png`, `basemap_02_4.png` in full, and a 6x crop of the bug on `basemap_00_2`, `basemap_01_3` and a four-frame strip of play 24 (the P2-1 evidence), via `/tmp/rev-crop-*.png` and `/tmp/rev-strip-*.png` (scratch, not in the repo). I did not run a sim probe, so P2-2's fair-ball case is derived from the code and the sim's own comments, not reproduced. No background shell was left running. I wrote nothing but this file.

## Resolution

Open.

### Worker follow-up

All client-only; no sim file changed.

- **P2-1 fixed.** The lit base is now decided from the runner's progress along the base path (`BbBaseMap::PathProgress`: his position projected onto the nearest side of the diamond, clamped to the side's ends), not a radius. A base is lit from when he reaches it (1.5 m short of it) until he has committed to the next (half way along the side). This is stateless, so it is right when paused, scrubbed or reopened, which ruled out hysteresis (it needs history). It holds through a long lead, an overrun (it clamps to the bag), a return to the base, and a runner rounding a base without stopping (lit as he passes, then dark half way to the next). New check assertion: for every runner still on the field at the end of a non-foul, non-half-ending play, a base he started at and ended at was lit at every sample, and a base he arrived at (within 1 m) was lit at every sample from then on, judged by his own track in metres, not the path rule. With the hold shortened to 0.1 the assertion fails at once (tested), and with the final rule seeds 7 and 51001 pass. New pictures `client/Saved/Shots/basemap_01_1..4.png` and `basemap_01_strip.png` (seed 7 play 24): first stays lit through the overrun, no flicker.
- **P2-2 fixed.** `BbBaseMap::IsFoul` is exactly the summary "Foul"; the `pitch_call` and suffix terms are gone; the director's umpire foul signal uses the same helper. `docs/hud.md` notes that a structured `foul` flag from `bb_ffi` would replace the match; open item added to TASKS.md section 5 (sim not changed).
- **P3-1, P3-4, P3-6 fixed.** The `RunBaseMapCheck` doc names `Read`; the orphan sentence is back on the bug bullet and the lit-base wording in hud.md is updated; the `Read` comment is one list plus the foul exception.
- **P3-2, in part.** Added: an out runner has no dot from his call time on, and no dot shows for a runner at the plate. The rest is left.
- **P3-3 and P3-5**: left alone, as ordered.
