# Review: the fixes to the saved-plays review (`work/saved-plays`, `c6d522c..ebcd6ad`)

Reviewer: Claude Sonnet 5.5 (wrote none of the change). Client-only change; it cannot alter a game. Reviewed against AGENTS.md, PLAN §1 D9 (Q30 b) and the Codex review's "Worker follow-up".

## Summary

The three P1 fixes and the P2 fixes close their findings. I found **no P1** (no wrong play shown, crash, hang, silent re-simulation or broken build), and one P2 in the build stamp. The rest are P3.

- The play saved is the play last drawn. `Presented` is captured after the frame is drawn and before any handoff in that tick, and the bytes are copied when the dialog opens.
- Nothing in saved mode asks the sim for a pitch. I ran an empty argument, a quoted path with a space, and a legacy file: each logged `BB_RESULT plays=0`.
- Loading a recorded timeline is memory-safe because Rust verifies the FlatBuffer. The MD5 binds exactly the bytes written to the bytes read.
- The 39-case check is not vacuous: it compares two different buffers (the live sim's blob and the reloaded file), and every refusal case asserts its own banner except one.

Findings:

| # | Sev | Where | One line |
|---|-----|-------|----------|
| 1 | P2 | `client/Build/stamp-commit.sh:23-28` | `-dirty` is set by any untracked or modified file in the whole repo, so most saves record a commit that cannot be checked out |
| 2 | P3 | `BbDirector.cpp:4429-4465` | Handoffs are held only by the pause; three branches ignore it |
| 3 | P3 | `BbSavePlay.cpp:660-675` | A save with a park but no full roster ignores its park and the on-screen note does not say so |
| 4 | P3 | `BbSavePlay.cpp:67-76`, `208-262` | Names are rewritten when they hold `"`, `;`, `=` or `\`; park and player values are not range-checked |
| 5 | P3 | `BbSim.cpp:107-137`, `BbDirector.cpp:1144` | Some sanity checks are unreachable, and the tests do not prove `LoadRecorded` calls them |
| 6 | P3 | `BbDirector.cpp:377` | `-BbSavedPlayCheckFile=` with an empty value swallows the next option |
| 7 | P3 | `BbDirector.cpp:1104`, `BbSavePlay.h` | The check leaves some round-trips unasserted |
| 8 | P3 | `docs/hud.md`, `docs/data-formats.md` | Small wording errors |

## Findings

### P2-1. The `-dirty` stamp is set by files that do not affect the build

`stamp-commit.sh:23-28` runs `git status --porcelain` over the whole worktree. Any change anywhere counts: an edited `TASKS.md`, a review file, and above all a new `saved-plays/*.toml`. Saving plays is the owner's normal loop.

Failure scenario: the owner saves a play. The new `.toml` and `.timeline` are untracked, so the next build is stamped `<sha>-dirty`, and every save made with that build records `commit = "<sha>-dirty"`. `<sha>-dirty` names no state anyone can check out. D9 keeps the commit "so the client shows it exactly as seen", and the commit is also what a refused save tells the owner to build.

Evidence: my own run of the check printed `commit=ebcd6ad-dirty`. The only things untracked here are the two `REVIEW-*.md` job files.

Suggested fix: restrict the dirtiness test to the paths that decide the binary, for example `git status --porcelain -- client sim ':!client/Saved'`. Optionally record the `-dirty` marker only for tracked-file changes.

Checked and fine: a throwaway repository gave `sha`, then `sha-dirty` after an untracked file, and the header's mtime did not change between two runs with the same text (no rebuild storm). With git missing from `PATH`, or outside a repository, the script exits 0 and writes `unknown`. It quotes the project path, and this repository's path has a space. The documented gap (the first build after a `*.Target.cs` change keeps the old stamp) is real but harmless. I did not test from the main checkout.

### P3-2. Handoffs are held by the pause alone, not by the dialog

The fix text and `data-formats.md` say handoffs hold while the dialog is open. The only guard is `bPaused = true`, which freezes `PlayTime` and `ResetTime`. The branch at `BbDirector.cpp:4429` (`PlayTime >= duration + HoldAfter && NextPlay.bValid`) and the one at `:4443` (`... && bPending && Pending.IsReady()`) are not guarded by `bSaveOpen` or `bPaused`.

Failure scenario: the dialog opens after the hold time has already passed, while the worker's next pitch is not yet ready (rare: the sim runs one pitch ahead). When the worker finishes, `BeginPlayback` runs under the open dialog, the screen moves to the next pitch, and `Presented` follows it. The dialog's title and its copied bytes still say the earlier play, so the save is correct. Only the screen and the dialog disagree, and `History[0]` can be released (the copy is why this is safe).

Everything else in Q1 holds:

- `Presented` is captured at `:4372`, after the frame is drawn and before the handoff chain.
- S, the unattended hook and the `R` key all read the previous tick's `Presented`.
- `HoldsBlob` guards against a freed pointer, and no release of `Presented`'s blob can fall between the capture and the next tick's key handling.
- Cancel and Save both restore `bPaused` and the cursor.
- Opening a second dialog is refused.

Suggested fix: add `&& !bSaveOpen` to the two branches, or make the dialog's pause explicit in the chain.

### P3-3. A park without a full roster is ignored silently

`FinishView` (`BbSavePlay.cpp:660-675`) calls `UseRecorded` only when `HasRoster()` (exactly 18). If a hand-edited or damaged save has a valid `park` but fewer or more than 18 player lines, the park is dropped. The note then says only "names, numbers and throwing hands" come from the current build, and does not mention the park. A normal save always writes both, so this is not reachable from the game itself.

### P3-4. Writer and reader differ on special characters, and values are unchecked

- `Plain()` (`:67`) replaces `"`, `;`, `=`, `\` and control characters in a name with a space. The file is always safe to read, but a name such as `O"Neil` comes back as `O Neil`, which is not "as saved".
- `ParsePlayer` and `ParsePark` do not range-check. `(uint8)W[3]` and `(int32)H[0]` are casts from a double that may be out of range (undefined behaviour in C++). `nan` and huge values go straight into `ParkDesc`, which `Park->Build` then uses.
- `timeline_md5` is optional. Deleting the line disables the binding (the play-number check still runs).

All of this needs a hand-edited file. The number formats are safe: `%.9g` reads back to the same float, and UE's `Printf` and `Atod` are locale-independent here. I did not test them under a non-C locale.

### P3-5. Some "sampled" and "summary" checks cannot fail, and the wiring is untested

- `InfoSane` rejects `n_calls > BB_MAX_CALLS`, and `LoadRecorded` rejects `n_actors > BB_MAX_ACTORS` and `n_dives > BB_MAX_DIVES`. `bb_timeline_info` and `bb_timeline_sample` clamp these themselves (`bb_ffi/src/lib.rs:422-424`, fixed-size arrays), so those branches never run on real data. The "too many calls" test (`BbDirector.cpp:1144`) tests dead code.
- The summary tests call `InfoSane` directly. If `LoadRecorded` stopped calling it, every check would still pass. The worker said this ("a hand-crafted blob cannot be built"); the gap is that the wiring has no test.
- 49 frames cannot rule out a non-finite value between samples. A valid FlatBuffer with two knots at the same time would give a non-finite Hermite value at that instant only. The MD5 means the file cannot be damaged after saving, so this needs a buggy saving build whose own MD5 matches. I consider the hang and out-of-range-actor risks closed: the duration is capped at 120 s, the actor and dive arrays are fixed-size, and Rust verifies the buffer before reading it.
- I did check that `InfoSane` does not refuse real plays. A throwaway ctypes probe read 744 plays (seeds 7, 3647 and 11) from `libbb_ffi.so` and applied the same conditions: 0 refused. The longest play is 22.7 s, the call time is at most 0.2 s past the end, the dead time is never past the end, and the contact time is never past the end.

### P3-6. `-BbSavedPlayCheckFile=` with nothing after it takes the next option

This is the bug the fix removed for `-BbSavedPlay=`. The check file option still uses `FParse::Value`. Run with `-BbSavedPlayCheckFile= -game` it reports "Could not read -game" and exits 1. It fails closed, so only the message is wrong.

### P3-7. What the check does not assert

The check is sound in the ways that matter: the frame comparison is between the live blob and a reloaded copy; the round trip changes a name, a hand and the fence height; the runners come from a play with runners on; the swap case is refused by the MD5 and not by chance. Not asserted:

- the "empty timeline file" case passes any title (`nullptr`, `BbDirector.cpp:1104`); I saw it refused as `TIMELINE UNUSABLE`;
- `BoardAfter`'s round trip, the other 17 players and the park's walls, poles, bases, rubber and dirt are not compared (only `fence_height_m` and `n_wall`);
- the play is chosen with runners on, but if no play fits in 60 pitches the loop falls back (`I == 59`) to any play, and then `bases == 0` is compared with `0`. It did not fall back on seed 7 (`runners before: 1`).

### P3-8. Docs

- `docs/hud.md` says "no game is started" for a refusal. A sim is created from the seed (`Started->Start`) and then not asked for any pitch. "No pitch is requested" is accurate.
- `docs/data-formats.md` has a stray line of two spaces before the `client/run.sh -BbSavedPlay=` paragraph.
- AGENTS.md is accurate, except that `-BbSavedPlayCheckFile=` empty-value behaviour is as in P3-6.
- The 27 legacy headers are accurate: each names its own seed and its own inferred commit, none holds a `timeline =` key (all 27 are refused, with the commit named, which I confirmed for one), and the diff touches only line 2 of each.
- Nothing exercises the "older save without roster" path on a real file, because no save in `saved-plays/` has a timeline. Only the check covers it.

## Q-by-Q answers

1. **P1-1:** closed (P3-2 above for the one hole in the stated hold).
2. **P1-2:** bounds are right and no real play is refused. A bad blob is read only after Rust's verification, so nothing reads a bad blob before the checks. See P3-5 for what is unreachable.
3. **P1-3:** an empty value, an empty value followed by an option, and a quoted path with a space were each tried; `plays=0` in all. A value followed by an option (`-BbSavedPlay=-foo`) is taken as a path and refused as "could not open", still in saved mode. `=` inside a path is fine (the value runs to whitespace, or to the closing quote).
4. **P2-3/4:** `FinishView` runs before `Park->Build` and the figures and crew (`BbDirector.cpp:418` before `:421`). See P3-3 and P3-4 for the writer and reader.
5. **P2-1:** see P2-1.
6. **P2-2:** see P3-5 and P3-7; nothing passes vacuously on the paths that matter.
7. **Live play:** no change except `SavedWarning` being reset every tick and the live board branch being re-indented. `SavedBoard` runs only in saved mode: a `BbBoxScore` zero-fill and a small loop per frame. I measured saved mode headless at 104-122 fps with the board repainted only on change. I did not measure a live game.
8. **Docs:** P3-8.

## Verification

Ran:

- Read all of `git diff c6d522c..ebcd6ad` for `client/` and the docs, and `BbDirector.cpp`, `BbSavePlay.cpp`, `BbSim.cpp` and `bb_ffi/src/lib.rs` (the timeline reading) in full where they touch saved plays.
- `taskset -c 0-7 nice` Build.sh of `BaseballClientEditor` at `ebcd6ad`: up to date, stamp `ebcd6ad-dirty`.
- `client/run.sh 7 -RenderOffscreen -BbSavedPlayCheck`: `BB_SAVED_PLAY_CHECK PASS commit=ebcd6ad-dirty cases=39`, exit 0, 20 s, 652 frames equal.
- Four headless runs of the editor binary (scratchpad logs only): `-BbSavedPlay= -BbBenchSeconds=4` (empty argument: banner path, `plays=0`, exit 0); `-BbSavedPlayCheckFile=` on a legacy file (refused, commit 30634cf named, exit 1); `-BbSavedPlayCheckFile=` empty (exit 1, "Could not read -game"); `-BbSavedPlay="<path with a space>"` on a copy of a legacy file (refused for its missing timeline, `plays=0`, exit 0).
- `stamp-commit.sh` in a throwaway repository (clean, dirty, mtime unchanged on repeat, git missing from `PATH`, outside a repository).
- A Python ctypes probe over the three games above (744 plays) to apply `InfoSane`'s conditions to real summaries (scratchpad only; no `sim-cli games`).
- `ls` and `grep` over the 27 legacy TOMLs.

Not checked:

- The dialog itself with a person at the keyboard (typing, drag, Esc), and `-BbSaveAtHandoff` or `-BbSaveAt` (they write into `saved-plays/`, which I was told not to touch). I read the code only; the Coordinator's `BB_SAVE open seed=7 play=1 (current play now 2)` line is the only evidence for the handoff case.
- A build from the main checkout, from a clean worktree, or with `git` missing during a real UBT build (the script was tested alone).
- A live game's frame cost, a non-C locale, or a save of a play from a game in progress through a replay or the walk back.
- Hand-edited files with `nan` or huge park values: I read the parser but did not feed them in.

## Worker follow-up (2)

Commit 94133ea (Sonnet 5.5):

- **P2-1:** `client/Build/stamp-commit.sh` now runs `git status --porcelain -- client sim` with excludes for `client/Saved`, `Intermediate`, `Binaries`, `DerivedDataCache` and `sim/target` (gitignored files never show). `-dirty` on status failure, `unknown`, and rewrite-only-on-change are kept.
- **P3-2:** the two handoff branches in `BbDirector.cpp` (`NextPlay.bValid` and `Pending.IsReady()`) now also need `!bSaveOpen`.
- **P3-6:** `-BbSavedPlayCheckFile=` is parsed by `SavedPlayArgument` (now taking the option name); an empty value logs `BB_SAVED_PLAY_CHECK_FILE REFUSED no file given` and exits 1.
- **P3-8:** `docs/hud.md` says "no pitch is requested"; the stray line in `docs/data-formats.md` is gone; AGENTS.md says the check-file option also exits 1 when no file is given.

Ran: the stamp script in a throwaway repository (clean → `ab94a9c`; untracked `REVIEW-X.md`, `TASKS.md`, `saved-plays/x.toml` at the top → clean; file under `client/Saved` → clean; edit under `client/Source` → `-dirty`, reverted → clean; untracked file under `sim/crates` → `-dirty`; file under `sim/target` → clean). Editor target build (succeeded). `-BbSavedPlayCheck`: PASS commit=94133ea (no `-dirty`, with the untracked `REVIEW-*.md` files present), cases=39, exit 0. `-BbSavedPlayCheckFile= -game`: the "no file given" message, exit 1. Not run: the dialog with a person at the keyboard, and `-BbSaveAtHandoff` (it writes into `saved-plays/`), so P3-2 is checked by reading only.

## Resolution

Coordinator, 2026-11-18.

- **P2-1, P3-2, P3-6, P3-8: fixed** in `94133ea` (see "Worker follow-up (2)"). Not re-reviewed: small and covered by the check; the Coordinator rebuilt from touched sources and ran `-BbSavedPlayCheck` at `2aaa3ca` (PASS, 39 cases, exit 0, stamp `2aaa3ca` with no `-dirty`).
- **P3-3, P3-4, P3-5, P3-7: deferred**, with a line in TASKS.md. Each needs a hand-edited save file or is a gap in the check's coverage, not a fault a normal save can reach.
