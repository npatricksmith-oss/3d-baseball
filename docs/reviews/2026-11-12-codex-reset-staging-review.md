# Reset staging review

Reviewed `dc870d72b8d266a6aabe0312302226a443b8f533`, including the previously unreviewed staging introduced in `1f7ca4c`, after reading `CLAUDE.md`. Read the reviewed files with `git show dc870d7:<path>` and the change with `git diff dc870d7~1 dc870d7`; the checkout's source was not used as the reviewed implementation. Locations below refer to `dc870d7`. Intent comes from its commit message and `docs/unreal-spike.md:50`. No P1 finding established.

## Findings

### 1. P2 — Side changes erase outgoing players instead of completing their exit

**Location:** `client/Source/BaseballClient/BbDirector.cpp:436–474`, `482–500`, `965–971` (render visibility at `901–917`).

The reset allocates the next pitch's actors first, then retains outgoing actors only until the shared 16-slot limit. A normal side change needs at least 20 actors. Some outgoing players therefore vanish at the beginning of the reset. The outgoing movers that fit are excluded from `Longest`, so others disappear at the next pitch while still well short of their dugout. This is separate from the documented shortcut of showing incoming fielders already partway out.

**Executed scenario:** In seed **7**, transition **play 6 → 7** (zero-based timeline indices), ten incoming actors leave room for only six outgoing fielders. Outfielders `0x187`, `0x188`, `0x189` and batter `0x003` are omitted immediately. The reset lasts **7.6667 s**. At its end, retained outgoing fielders `0x184`, `0x185`, `0x186` are respectively **14.08 m, 18.66 m, 22.97 m** from their dugout, then disappear. All **55 side changes** across seeds 7–9 omit outgoing actors. These are frame/scheduling reproductions, not rendered observations.

**Suggested fix:** Give outgoing staging actors their own rendering capacity, retain them through their exit, and either wait for them or let their exit continue during the next pitch. If a shortened side change is desired, make an explicit camera transition that conceals the removal rather than dropping visible actors in the field.

### 2. P2 — The first pitch still runs the sim on the game thread

**Location:** `client/Source/BaseballClient/BbDirector.cpp:350–359`; `client/Source/BaseballClient/BbSim.cpp:42–47`.

This is a pre-existing exception found during the requested Unreal-rule audit. `BeginPlay` calls `BeginPlayback(Sim->Next())` directly. `Next()` calls `bb_game_advance`, which resolves the entire pitch and any resulting live play before returning. Only subsequent requests use the thread-pool path at `BbDirector.cpp:362–371`. This violates the instruction that the sim runs off the game thread, even though startup has no previous clip to play.

**Concrete scenario:** Launching any game resolves its first pitch inside `BeginPlay`. With the reviewed FFI library, seed 7's first advance took about **72 ms** in this probe; this is an illustrative measurement, not a performance bound. A more expensive opening play would also block the game thread.

**Suggested fix:** Request the initial pitch through the same worker mechanism, show a loading/empty playback state until it is ready, and invoke `BeginPlayback` on the game thread after completion.

### 3. P3 — Short triangular returns can remain idle or face away from their movement

**Location:** `client/Source/BaseballClient/BbDirector.cpp:600–607`, `652–668`, `730`, `751–760`.

The new profile accelerates correctly, but animation still uses smoothed measured speed: idle below **0.4 m/s**, movement-facing only above **0.9 m/s**. Short triangular moves can finish without crossing either threshold. The root moves while the figure remains idle, or it plays a forward walk while continuing to face the plate. Avoiding the 14 m/s placement cutoff does not address these lower thresholds.

**Executed scenarios:** Seed **7**, transition **168 → 169**, moves left fielder `0x087` **0.05054 m** in **0.25958 s**. Its physical peak is only **0.389 m/s**, so the whole return is idle even before smoothing. In the same reset, runner `0x103` returns **0.13905 m**, directly away from home, in **0.43057 s**. Its peak is **0.646 m/s**, below the facing threshold. A 60 Hz reproduction of the smoothing reaches about **0.497 m/s**: the walk activates, but facing remains toward home, approximately opposite the path. A separate 0.5 m outward-return probe spends about its first **0.26 m** facing backward while the turn catches up.

**Suggested fix:** Supply staging movement direction and motion phase to the figure independently of smoothed speed. Begin a controlled turn before departure and use a suitable slow step/shuffle or locomotion blend for short moves; choose idle when the move is complete. Keep the physical rest-to-rest profile.

### 4. P3 — Fast playback can skip the return-throw animation trigger

**Location:** `client/Source/BaseballClient/BbDirector.cpp:617–629`, `687–704`, `836–842`, `865–868`.

The ball samples directly from reset time, but starting the throw clip requires observing a frame during the final **0.3 s before release**. A speed-key change or a long frame can step over that window. After release, `holder` is already `-1`, so the clip cannot start even while its follow-through should still be visible. Pausing on that frame preserves an airborne ball with an idle thrower.

**Executed scenario:** Seed **7**, transition **0 → 1**, throws at reset time **5.116815 s**, with **0.883185 s** flight. At **30 Hz, 16×**, samples at **4.8 s** and **5.333333 s** straddle the entire trigger window. The first sample is **0.316815 s** before release; the second has the ball about **24.5%** through its flight and should show clip time **0.666519 s**, but `bThrowing` was never set. Replaying the trigger at those rates misses **94 of 637** staged throws across seeds 7–9; at 60 Hz, 1× and 16×, none were missed in this sample.

**Suggested fix:** Derive the reset throw's animation phase from its known release time on every frame, including the post-release follow-through. It must not depend on having visited an anticipation frame. For timeline throws, retain/query the relevant release across the elapsed sampling interval as well.

## Verification

- Exported the reviewed snapshot into `/tmp/bb-reset-review-dc870d7` with `git archive`. Built its unmodified `bb_ffi` offline in release mode with two build jobs and target/Zig caches under that scratch directory. Ran `cargo test --offline --release -p bb_ffi -j 2` against the scratch manifest: **2 tests passed**, zero failed; doc tests passed. No other worktree was checked out or modified.
- Reimplemented `TravelSeconds`, `TravelGone`, `SpeedFor`, and reset scheduling in scratch Python probes. The real-number profile sweep covered **10,026 cases**, including zero, tiny distances, triangle/trapezoid boundaries, and a distance grid through 100 m. Positions were finite, bounded, monotone, and reached the endpoint; the chosen cruise never exceeded 5 m/s. A separate emulation rounding each profile arithmetic operation and square root to float32 checked **30,051 distance/speed combinations**, each with 1,001 time samples: no backward sample steps, finite bounded positions (within float tolerance), and exact distance return when called at total time. This is an arithmetic emulation, not a compiled Unreal test. For positive configured speeds/acceleration, the zero-distance guard precedes division, and the triangular branch has no cruise-time division. `SpeedFor(0, ...)` may return zero, but callers retain a positive base speed with `Max`.

  | Distance | Selected cruise | Rest-to-rest time | Actual peak |
  | --- | --- | --- | --- |
  | 0 m | 1.6 m/s | 0 s | 0 m/s |
  | 0.5 m | 1.6 m/s | 0.816497 s | 1.224745 m/s |
  | 6 m | 3.6 m/s | 2.866667 s | 3.6 m/s |
  | 20 m | 4.417424 m/s | 6 s | 4.417424 m/s |
  | 70 m | 5 m/s | 15.666667 s | 5 m/s |

- Sampled complete games for seeds **7, 8, 9** through that snapshot's C ABI: **697 pitches, 694 transitions**. Reconstructed `StartReset` scheduling from actual end/start frames. Every retained next-pitch actor, including delayed throwers, finished by `ResetDuration`, with zero remaining distance in the reconstruction. The longest reset was **30.1329 s** (seed 7, 195 → 196), correctly including the thrower's delayed 62.66 m return after waiting for the pitcher. The reset is no longer capped at six seconds.
- **Ball timing:** All **637 staged throws** arrived at or after the pitcher's computed arrival. The pitcher is never the delayed thrower, so his arrival calculation does not omit a departure delay. No throw is staged across a side change. However, release can precede mound arrival: in seed **7**, **120 → 121**, release is at **3.5 s**, with the pitcher **7.20 m** short; the ball arrives at **6 s**, when he arrives. Thus the implementation synchronizes reception, whereas the documentation's “throws it ... once he is back” literally describes a later release. No absent-recipient catch was established.
- **Handoff and controls:** Returning actors use the next timeline's actor order and sample-at-zero targets. All sampled actor heights were zero, matching `ResetFrame`'s zeroed Z. `TravelGone` clamps after arrival; the completion tick displays the staged endpoint before `BeginPlayback` starts the next preroll at `-1 s`. No material positional pop was established for the next pitch's actors; outgoing actors have finding 1. Pause freezes both `PlayTime` and `ResetTime`; their constant offset makes the reset throw's `ThrowStart` calculation work despite using `PlayTime`. Manual clip phase and gait playback freeze. Facing interpolation still uses wall-clock `DeltaSeconds`, so a manual turn can continue easing while paused. Speed keys advance both clocks consistently and do not raise the profile's speed in simulation seconds. They can skip animation windows as in finding 4. There is no separate clip-skip key in this snapshot.
- **Facing/placement:** The coordinate swap and mannequin's −90° correction agree. Ordinary movement-facing uses an interpolated rotation rather than a direct snap. A continuous reset mover's sampled average speed is at most 5 m/s, so it cannot itself trigger the 14 m/s placement rejection, even when playback is accelerated; `SimDt` scales with playback speed. Finding 3 concerns the separate low-speed animation gates.
- **Authority:** The reset's ball arc and player interpolation are presentation of dead time and never feed back into the sim. Collision is disabled for the displayed shapes/figures. No gameplay physics stepping or Blueprint game logic was found in this path. Subsequent pitch resolution uses `Async(...ThreadPool...)`; startup has finding 2.
- Unreal **5.8.4 is present**, but neither this checkout nor the scratch export has client `Content`, `Binaries`, or `Intermediate`, so this would not be the cheap incremental build described in `CLAUDE.md`. No Unreal build, rendering, or screenshot verification was performed. Animation findings reproduce the C++ conditions and timing mathematically; they do not establish asset-specific visual quality. Source, tests, specifications, and `TASKS.md` were left untouched by this review; its sole repository write is this document. Concurrent unrelated source, scenario, and review changes appeared during verification and were left untouched.

## Resolution

2026-11-12: `dc870d7` merged into main as is; all four findings deferred. None is a regression of `dc870d7`: findings 1, 2 and 4 are in the staging and startup code it builds on (`1f7ca4c`, never reviewed until now), and finding 3's thresholds predate it (the old 4.5 s cap made short moves slower still). The commit fixes what the owner reported (the pitcher skating back at up to 16 m/s); the review's profile sweep and reset scheduling checks confirm it. Fixing these needs Unreal client work and a watch, so they are listed first in TASKS "Resume here".

| # | Status | Why it waits |
|---|---|---|
| 1 | Deferred (P2, client) | Outgoing players vanish at side changes (16-slot actor pool shared with the next pitch). Needs separate capacity for outgoing actors or a camera cut; visible at every side change, so first in line |
| 2 | Deferred (P2, client) | The first pitch resolves on the game thread (`BeginPlay`, ~72 ms). Route it through the same `Async` path with an empty first frame |
| 3 | Deferred (P3, client) | Very short returns (under ~1 m) can play idle or face backward: drive staging gait and facing from the planned motion, not smoothed speed |
| 4 | Deferred (P3, client) | At high playback speed the return-throw clip can be skipped: derive its phase from the known release time each frame |
