# Umpire calls review — 2026-11-12

Reviewed commit `0d81821c0c7324703ab62a709e81514464fc9899` against its parent `469d25c`, after reading `CLAUDE.md` and the binding plan/spec guidance. HEAD remained at the reviewed commit during verification. No implementation, test, spec, or TASKS files were edited.

## Findings

### P2 — Strikeout scoreboard updates bypass the scheduled call

**Location:** `client/Source/BaseballClient/BbDirector.cpp:903`, `:935–939`.

For pitches without contact, `bKnown` becomes true at the fixed time 0.55 s. That branch immediately displays the final outs and replaces `Shown`, even when `Called == 0`. The strikeout signal instead occurs at the deciding pitch fact plus 0.2 s. Thus the scoreboard announces an out before the umpire, and the scheduled call does not advance it.

**Reproduction:** seed 0, play 2: the strikeout fact is at 0.488051 s and the call is at 0.688051 s. At both 0.55 and 0.60 s, the HUD logic already displays Top 1, 1 out, with zero calls delivered. All 283 strikeouts in the 20-seed probe had call times later than 0.55 s. With two outs, the same branch can reveal the next half before the third-out signal.

Derive the displayed outs from a stable pre-play snapshot and the delivered calls even after the pitch summary becomes known. The summary/count reveal and out reveal need separate timing.

### P2 — The third-out call never advances the board to three outs

**Location:** `client/Source/BaseballClient/BbDirector.cpp:935–944`.

On an inning-ending ball in play, the recorded motion usually ends at the deciding out and `timeline::encode` extends the clip through its delayed signal. Consequently the third-out call and `duration_s` coincide. At that instant `bKnown` takes precedence over the call-driven branch, installing the next half's final state (zero outs) immediately. The board skips the third out while the outgoing play and its OUT signal remain on screen, including the post-play hold before reset.

**Reproduction:** seed 0, play 9, a double play starting with one out: the first call at 3.890907 s displays Top 1, 2 outs. Immediately before the second call it still displays that state. At the second call, 4.878013 s, it switches directly to Bottom 1, 0 outs. The third-out call equaled clip duration in 269 inning-ending batted-ball clips across seeds 0–19.

Keep the outgoing half and its three-out display through the call/post-play phase; move to the next half at the reset or next-play transition. Preserve the pre-play state separately so rendering the final summary cannot overwrite the baseline used to count calls.

## Other review results

- **Out instants and completeness:** Replaying each pitch's facts from its saved pre-pitch rules state matched all 1,079 calls across seeds 0–19 to the exact fact that emitted that runner's `RunnerOut`, including its base and timestamp: 381 forces, 377 fly outs, 283 strikeouts, 37 tags, and one appeal. No missing calls or reversed signal times were observed. The game's batter and occupied-base runners populate both the rules state and `self.r`; no normal Game path was found that retires an unrepresented runner. The public live simulator still assumes consistent runner inputs.
- **Force/catch/third-out paths:** In the current rules implementation, `GloveSecured` emits a fly out; a force or base appeal is emitted by `BaseTouched { holding_ball: true }`, which `holder_act` can issue in the same tick as securing the ball on the bag. Tags and tag appeals use `TagContact`. `emit` reads the returned events even if the third out has already ended the live play. Neither `PlayEndCheck` nor either timeout route creates `RunnerOut`, so the direct `step` at `game.rs:327` currently loses no out calls. Live facts are resolved at the existing 1/60 s tick resolution; calls do not add another simulation tick of latency.
- **Margins:** The formulas match rules-spec §1.3's distance/top-speed proxy. Force/appeal distance uses the event base; a tag uses the runner's current goal, which can be behind him. This measures proximity, not a full acceleration-aware arrival-time advantage. Capture/tag/holder actions precede the next runner movement update, so the margin uses the runner position available at that deciding fact. `low_catch` is assigned before every successful `GloveSecured`, and each play starts with a fresh Sim; a low throw reception cannot leak into a later fly out. A 50-seed probe found 115 tags and one appeal, but no retreating-tag example. Overrun/retreat edge cases therefore remain inspection-based rather than targeted behavioral coverage.
- **ABI:** C and Rust agree on this x86-64 build: `sizeof(BbPlayInfo) = 176`, alignment 4, `n_calls` offset 124, `calls` offset 128; `sizeof(BbCall) = 12`, alignment 4, runner offset 8, base offset 10. `repr(C)`, field types, and explicit padding agree. The FFI sorts calls and bounds the copy to four; the Unreal code includes that same header. Appending is a binary ABI change: an old 124-byte caller must be rebuilt before loading the new library. `client/run.sh` builds both sides. Mixed-version dynamic loading has no size/version handshake.
- **HUD lifetime/reset:** The latest delivered signal replaces the previous one and is eligible for display for two seconds of play time. Playback speed scales that wall-clock duration; pause preserves it. Reset continues advancing `PlayTime` while retaining the outgoing play, so it does not restart calls or repeatedly increment outs. The first-play initialization sets `Shown` to Top 1, but the HUD scoreboard remains blank until one of its update branches runs; that blank-opening behavior predates this change.
- **Tests:** Event counts, sampled runner tracks, and scoreboard deltas supply useful independent evidence. However, the timestamp assertion at `tests/umpire_calls.rs:60` only finds *some* potentially deciding fact at that time, rather than proving that fact retired this runner; the replay probe above checks that stronger association. Margin checks use approximate track interpolation, assume 8.2 m/s, and skip several edge cases. No added test exercises the HUD's reveal precedence, explaining why the findings pass the suite.
- **Invariants:** The added call data does not feed back into rules, agents, movement, or RNG. No direct transcendental calls, unordered collection iteration, or outcome rolls were added. Parent/current game-log digests matched for seeds 1, 2, 3, and 42; the existing determinism and recording-on/off tests also passed.

## Verification

1. Inspected `git show 0d81821`, the rules engine's complete out-emission paths, live tick ordering, timeline encoding/generated schema, C/Rust layouts, and Unreal playback/reset/HUD code.
2. Ran `cd sim && cargo test --release --workspace`: **130 passed, 0 failed, 0 ignored**. The first attempt failed because Zig's default cache was outside the writable sandbox. The successful rerun set `ZIG_GLOBAL_CACHE_DIR=/tmp/umpire-zig-cache` and `ZIG_LOCAL_CACHE_DIR=/tmp/umpire-zig-local`.
3. Ran `bash sim/crates/bb_ffi/c/run_smoke.sh` with those cache variables: **passed**, `241 pitches, 54 out calls, final=1, score 6-3`.
4. Ran `cd sim && cargo run --release -p bb_game --example umpire_calls -- 20`: **passed**. Margin histogram in 0.5 s bins: `[493, 125, 109, 29, 23, 0, 0, 300]`.
5. Compiled temporary probes from stdin into `/tmp`: exact fact-to-out replay over seeds 0–19; signal ordering and third-out duration checks; tag/appeal inspection over seeds 0–49; independent C and Rust size/alignment/offset checks. All completed successfully after correcting probe dependency selection.
6. Used the built C ABI through Python `ctypes` to obtain seed 0 timelines and evaluate the HUD's branch conditions at the times reported in both findings. This verified the state arithmetic without running Unreal.
7. Compiled the parent's unchanged behavioral modules plus its `game.rs`/`live.rs` obtained with `git show`, using the same dependency builds, and compared the existing digest example with the reviewed build for seeds 1, 2, 3, and 42: **all four digests identical**. The parent probe omitted the unused timeline encoder to avoid mixing its old marker initializer with the new generated schema.

Unreal was not built or launched; rendered text layout and frame-by-frame visual presentation were not verified. Temporary binaries/logs stayed under `/tmp`. The pre-existing untracked anticipation review was left untouched.

## Resolution

Both findings fixed in the client (`BbDirector.cpp`, HUD update), checked with offscreen screenshots of seed 7 around the third out of the top of the first (a strikeout, called 0.82 s into the pitch):

- **P2, strikeout outs before the call: fixed.** On a pitch not put in play, the result (summary, count, bases) now appears at the later of 0.55 s and the umpire's last call. Screenshot at 0.4 s: "Top 1, 2 out, 2-2", no summary.
- **P2, third out skipped: fixed.** The board is now built from `Before` (the previous play's result, set in `BeginPlayback`, top of the first before the first pitch) plus the calls delivered so far. It never reads the final state's outs. A play that ends the half leaves that half on the board at three outs, through the hold and the reset, until the next pitch begins. Screenshots: "Top 1, 3 out" with "Strikeout swinging / OUT" after the call, "Bot 1, 0 out" on the next pitch. This also fills the board from the first frame (the blank opening the review noted).
- While checking, I found and fixed a bug of my own in the fix: on an inning-ending pitch the count was zeroed before the play was known (it showed 0-0 instead of 2-2).

Not changed, with reasons:
- Test strength (`tests/umpire_calls.rs`): the timestamp check matches *a* deciding fact rather than the one that retired this runner. The review's replay probe confirmed the exact association for all 1,079 calls, so this is left as is. The HUD's reveal logic has no automated test: the client has no unit-test harness, and offscreen screenshots are the project's check for it (CLAUDE.md).
- ABI handshake: there is no size or version check for a mismatched library. `client/run.sh` and `bin/play.sh` always rebuild both sides, so this is deferred until the client loads libraries it did not build.
