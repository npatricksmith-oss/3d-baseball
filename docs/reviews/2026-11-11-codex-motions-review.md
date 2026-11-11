# Independent review: watch fixes and stand-in motions

Reviewed `31d57bb..914a187`, after reading `CLAUDE.md`, PLAN §1, the owner-watch notes, and the previous review's Resolution. Priority is visible baseball and the binding invariants, without further percentage tuning. Locations below refer to `914a187`. No P1 finding established.

## Findings

### 1. P2 — A backup can intercept a catchable throw in front of its receiver

**Location:** `sim/crates/bb_game/src/live.rs:799–803`, capture loop starting at `792`.

The new permission checks whether the **ball** has crossed the base's plane, rather than whether it has passed the receiver's receiving area. Receivers move beyond the bag to meet actual throws (`update_thrown`), and both defenders can be within the 1.5 m receiving reach. Attempts still run in fielder-index order, so an earlier-index backup can secure the ball before the designated receiver gets his opportunity.

**Executed scenario:** A controlled throw toward second travels along −Y. At the test instant the ball and pitcher-as-backup are 0.20 m past second, at ball height 1.30 m; the intended second-base receiver stands 0.50 m past second. Thus the backup is **0.30 m in front of the receiver**. With perfect hands, enabling the backup's `backs = Some(2)` produces `Held { by: 0 }`. Disabling only that permission produces `Held { by: 3 }` within the next tick. The receiver could catch this throw; it is not an unreachable overthrow. This establishes the overlap failure, not its natural-game frequency.

**Suggested fix:** Preserve the intended receiver's catch opportunity, including his existing closest-approach wait, and admit a backup only after the throw has passed that receiver's physical receiving region. Merely testing passage beyond the nominal bag does not provide either protection.

### 2. P2 — The encoder changes the batter's realized positions to hide the jump

**Location:** `sim/crates/bb_game/src/timeline.rs:108–115`.

The live sim still records the batter on its home-origin path. Encoding adds a box offset that decays over the first four metres. These are changed root positions at the recorded sample times, not interpolation between physical samples. The client therefore reports a different location from the one used by the live sim's movement and tag-distance checks. This violates invariant 4/D4 even though it removes the visible jump and passes the speed ceiling test.

**Executed scenario:** Seed **1**, play **7**, at **0.405163 s**, the realized batter sample is `(0, 0, 0)`; sampling the encoded timeline at that knot gives `(-0.95, 0.30, 0)`, a **0.996243 m** discrepancy. The offset persists into the beginning of the run. A 16-game scan also detects discrepancies at subsequent recorded knots.

**Suggested fix:** Give the batter a physical route from his box into the base path in the live model. Use that same route for movement, tags, recording, and playback; encode its realized positions directly. The dead-ball award jog is a separate presentation case and does not justify changing live actor samples.

### 3. P2 — Proximity to a ball starts throws by players who do not possess it

**Location:** `client/Source/BaseballClient/BbDirector.cpp:433–454` (`Holds` and throw trigger).

Being within 0.5 m of the nominal carrying point does not establish possession. A passing throw, failed reception, or another defender carrying the ball nearby can satisfy `Holds(Frame, A)` and fail it 0.3 s later. The client then commits to a throw animation and turns the wrong defender toward the future ball position.

**Executed scenarios:** A scratch driver replayed the exact trigger and its `bThrowing` timing at 60 Hz against real FFI frames. In seed **1**, play **70**, at **3.3333 s**, shortstop `0x86` triggers although the only `GloveSecured` in the play belongs to third baseman `0x85`; the shortstop never possesses the ball. In seed **2**, play **43**, first baseman `0x183` triggers at **3.1833 s** as the airborne throw passes him. The ball then hits the ground at **3.807317 s**, and he actually secures it only at **6.639924 s**. These are data-level reproductions of the C++ condition; Unreal rendering was not run.

**Suggested fix:** Export realized possession and throw-release transitions, keyed by agent, and drive the anticipation clip from the next actual throw by that holder. Ball proximity alone cannot reliably distinguish receiving, missing, carrying, and releasing.

### 4. P3 — Clip preview can index an empty figure array

**Location:** `client/Source/BaseballClient/BbDirector.cpp:250–256`, `312`, `327`.

**Reasoning, client inspection only:** Figures are constructed only when the mesh **and all three gait clips** load. Preview admission checks only `FigureMesh`. With the mesh present but any idle/walk/jog asset missing, `bFigures` is false and `Figures` is empty, yet `-BbAnimTest` accesses `Figures[0]`. This yields an array assertion or invalid access instead of previewing the available clips. Missing preview clips themselves are handled, but the prerequisite figure allocation is not.

**Suggested fix:** Create the preview's figures independently of gait assets, or require and validate `Figures.Num() >= 8` before entering the loop and report the missing prerequisite.

## Verification

- Ran `export PATH=$HOME/.cargo/bin:$PATH; cd sim && cargo test --workspace`: **109 tests passed**, zero failed; doc tests passed. This includes the new lead/backup/home-return test, the offensive speed-ceiling timeline test, legal tag-up/retouch tests, rules scenarios, crate boundaries, and cross-thread/recording determinism checks.
- Ran `cargo build --release -p bb_cli && ./target/release/sim-cli games --n 128 --seed 2`: **128 games**, **4.18 runs/team-game**, and **`rules warnings: {}`**. No recommendation to move those percentages closer to a reference league.
- Built an external scratch crate under `/tmp/bb-motions-review`. For baseline comparison it contains a copy of `bb_game` with `live.rs` and `timeline.rs` restored from `31d57bb`; other sim source files are unchanged across the range. Ran its release `compare` mode for seeds **1–128**, baseline and reviewed code: **zero warnings and zero incomplete games in both**; appeal outs **11 → 10**; balls in play launched above 10° with multiple outs **20 → 18**. This is not a pure caught-fly metric, but neither it nor appeal totals indicates increased double-offs in this batch. No new stuck-runner/timeout case was established. The hold clears on the first read, a ground touch, or a caught-ball retouch response.
- Ran two targeted scratch tests with a copy of the reviewed `live.rs`, changing only its import path and adding probe access/tests: **2 passed**. One reproduces finding 1. The other checks the catcher returns home, reassigns the same outfield jobs, and returns home again. Assignment can transiently restore his backup role during a recall, but `cover_home_again` restores home coverage before movement in that tick; this probe does **not** establish visible oscillation. First's backup is intentionally vacated at the return threshold, and `back_up_throw` can appoint another on a later release. No additional consequential first-base coverage regression was established.
- Ran the scratch release frame-replay mode on **16 recorded games**, seeds **1–16**. It compares encoded actors to realized knots, checks actor IDs/counts across present/+0.3/+0.7 s frames, and reproduces the C++ throw predicate and its cooldown at 60 Hz. All sampled look-ahead frames succeeded and retained actor order. There were **774 trigger starts**, including **45 before any recorded possession by that actor** (a screening count, not a claim that every one is an erroneous full throw). The two unambiguous examples are in finding 3. Finding 2 is measured at a recorded knot, independently of interpolation.
- Ran scratch release `digest` mode: seeds **1, 2, 7, 14, 128**, each simulated twice, yielded identical complete event-log bytes and digests; all had zero warnings. Diff inspection found no new production transcendental calls, unordered-map iteration, RNG purpose renumbering/reuse, or explicit reused RNG sub-indices. Changed control flow can naturally change which existing keyed decisions occur.
- Compiled and executed the repository's C smoke client against the reviewed release `bb_ffi` static library, placing the executable in `/tmp`: **205 pitches, final=1, score 1–2**. Separate C static assertions and a Rust scratch test agree on `BbActor` **20 bytes**, `BbFrame` **364 bytes**, `BbPlayInfo` **124 bytes**, and relevant offsets (`actors=16`, `has_bat=336`, `bat_knob=340`, `bat_tip=352`, `summary=28`). The Rust test passed null game/output/blob, short-blob, and null-free cases. Actor sampling caps writes at 16. No new FFI safety/layout defect established; the range changes only FFI tests. Ball encoding/sampling is unchanged by this range.
- **Client review is by reading `914a187`, not by building or running Unreal.** With the current valid immutable timeline, `Sample` failure is independent of requested time, so a successful current frame also has successful look-aheads; ignoring those return values is not a demonstrated failure here. Actor tracks are never removed as time advances, so index identity is stable within a play. Fielders are excluded from the batter test, preventing catcher selection; no umpire actor is supplied. Leaving a manual clip resets `Manual`, and `Gait=-1` forces locomotion selection; the clip objects also have owning `UPROPERTY` references, so the cached raw pointer alone is not a demonstrated GC defect. A tag retains possession and no separate handoff exists in this sim; the pitcher's initial release is excluded by the delivery branch. No additional pre-roll defect established in camera cuts, delayed call text, wall-time screenshots/benchmark, or pause/speed timing. Preview's fixed-rate loops are separate from game playback.
- Cargo used `ZIG_GLOBAL_CACHE_DIR=/tmp/bb-review-zig-global` and `ZIG_LOCAL_CACHE_DIR=/tmp/bb-review-zig-local`. The scratch crate's initial online dependency lookup failed DNS; rebuilding offline succeeded. All probes and their binaries stayed in `/tmp`. Only this review document was written by this review. Concurrent unrelated edits appeared in `BbDirector.cpp`/`.h`; they were left untouched and excluded from the review.

## Resolution

Open.
