# Independent review: client v0 and realized timelines

Reviewed `2c1d4ef..4a94bc2` (`bdd1e43`, `4a94bc2`) against the binding invariants in `CLAUDE.md` and `PLAN.md`. Locations below refer to `4a94bc2`. No P1 defect found; recording preserved outcomes and event-log bytes in the executed comparisons. Unreal runtime behavior was inspected in code, not independently exercised in the engine.

## Findings

### F1 — P2: Short blobs abort the host instead of returning an error

**Location:** `sim/crates/bb_ffi/src/lib.rs:131–139` (`timeline`), shared by both new exports.

**Executed scenario:** Pass a readable seven-byte allocation and a valid output pointer to either `bb_timeline_info` or `bb_timeline_sample`. Both processes abort with exit status 134: FlatBuffers' `buffer_has_identifier` asserts that the buffer contains at least eight bytes, and the panic cannot unwind across `extern "C"`. Null pointers and the existing 64-byte junk test do not cover this. This is an input-validation crash, not demonstrated memory corruption. Rust's safety comments restrict callers to returned blobs, but the C header promises error returns for bad arguments/non-timelines without stating that restriction.

**Suggested fix:** Reject `len < 8` before checking the identifier; check null output pointers before parsing. Keep the verified `flatbuffers::root` path. Clarify that callers must supply a live, readable allocation of the stated length. Prefer a slice-taking parser whose returned timeline lifetime is tied to that slice, rather than the current safe helper with an unconstrained lifetime.

### F2 — P2: The realized timeline can end before the decisive catch or fence crossing

**Location:** `sim/crates/bb_game/src/game.rs:256–260`; `sim/crates/bb_game/src/timeline.rs:161–167`. Recording terminates before the final state in `live.rs:1363–1390`.

**Executed scenario:** Seed 7, play 22 ends with a fly catch at `4.150634 s`, but the encoded duration and final ball knot are `4.133968 s`. The last displayed ball is still airborne at `(7.189, 78.627, 0.264) m`; the client holds that frame, then advances the inning. Seed 0, play 37 similarly finishes before the receiving fielder secures the throw that produces the force out. Seed 0, play 10 stops at `5.169711 s`, before the home-run fence crossing at `5.177964 s`.

The live loop breaks on the terminating event before recording it. Further downsampling with `step_by(2)` can discard another final sample, and duration is then derived from those incomplete samples. This newly authoritative playback track omits the moment that explains the scoreboard change.

**Suggested fix:** Record the terminal ball and actor states on every exit, preserve first/last/event samples during downsampling, and derive duration from the complete play interval. Encode the possession/dead-ball transition at its actual timestamp so the final frame shows the completed play.

### F3 — P2: Awarded runners have no recorded journey to their awarded bases

**Location:** `sim/crates/bb_game/src/game.rs:200–206,239–265`; `sim/crates/bb_game/src/timeline.rs:84–96,161–167`.

**Executed scenario:** Seed 0, play 10 awards batter 258 a home run, but his last root sample is near first base at `(18.729, 20.066, 0) m`. He never rounds the remaining bases and disappears for the next batter. Seed 6, play 187 is a hit batter: there are no moving actor tracks, so batter 260 stays in the box for the entire `0.504187 s` clip, then appears on first in the next timeline. Walks use the same standing-only path. This is distinct from the missing terminal sample: retaining the final live tick cannot provide the missing awarded movement.

**Suggested fix:** Produce sim-authored movement tracks for dead-ball awards, including forced advances and home-run completion, and extend playback duration accordingly. Keep that continuation observational: it must not alter rules outcomes, event-log contents, or RNG consumption. Supply explicit actor completion/visibility semantics when a runner scores or leaves the play.

### F4 — P2: Moving batter tracks replace the batting-box position before contact

**Location:** `sim/crates/bb_game/src/timeline.rs:84–96`; `sim/crates/bb_ffi/src/lib.rs:220–225`.

**Executed scenario:** Seed 7, play 1 records standing position `(-0.75, 0.2, 0) m`. Its live root track starts at contact, `0.406973 s`, at `(0, 0, 0)`. Sampling at release returns that future root position because the encoder discards `standing` whenever movement exists and the sampler clamps earlier times to the first root knot. Play 8 repeats this. The batter therefore changes starting location depending on whether the upcoming pitch will be hit, with a roughly `0.78 m` snap relative to ordinary pitches.

**Suggested fix:** Preserve standing roots from release through contact, and make the physical transition from the batting box onto the running path continuous. Merely interpolating from a release knot to the current first live knot would make the batter drift before contact; the movement model and recording need a consistent starting position and time.

### F5 — P3: Central-difference Hermite tangents pull a ground ball underground before pickup

**Location:** `sim/crates/bb_game/src/timeline.rs:42–54`.

**Executed scenario:** Seed 0, play 37, sampled through the C ABI at approximately `1.965 s`, has ball-center height `-0.04994 m`. The surrounding recorded ground-ball positions at `1.944130 s` and `1.977463 s` both have height `0.036888 m`, the ball radius. The next sample, after pickup, jumps to hand height `1.2 m`; using it in the preceding central-difference tangent causes the earlier Hermite interval to dip below the turf. Thus accurate sampled positions alone do not ensure a physically valid playback curve.

**Suggested fix:** Preserve mode/possession transitions and one-sided physical velocities at those boundaries instead of fitting one smooth flight spline across ground, pickup, holding, and throwing. Use additional event knots where necessary; a client-side height clamp would conceal the incorrect trajectory rather than preserve authoritative physics.

### F6 — P3: The report confirms the full spike gate beyond what its measurements establish

**Location:** `docs/unreal-spike.md:3,12–15`; binding criterion in `PLAN.md §3.3`.

**Code/document-derived scenario:** The opening says the spike passes and D6 is confirmed against all criteria, while the table explicitly says the editor's desktop window was not checked. The binding criterion includes the editor opening and playing a capsule timeline smoothly. Offscreen game throughput and headless editor initialization establish useful parts of that gate, but do not establish the interactive editor experience. An average of 290–310 fps also does not establish uniformly smooth playback when measured post-warm-up frames reach 110–130 ms; the table appropriately discloses those hitches.

**Suggested fix:** Qualify the conclusion as the measured offscreen/runtime and build checks passing, with interactive editor playback and owner watchability review pending. Retain the disclosed hitches and resolution. After the desktop check, record its conditions and result before claiming the complete gate is satisfied.

## Viewer priorities

The three sim-side fixes I would prioritize for watchability are **F2** (show the catch/force/fence crossing that explains the result), **F3** (show awarded runners reaching their destinations), and **F4** (keep the batter in a consistent place until contact). These are timeline/model issues; camera, mesh, or animation polish cannot recover the missing or contradictory data.

## Verification

- Read `CLAUDE.md` first and inspected the commit diff, relevant rules/live-play code, schemas, C header, client lifecycle, and `PLAN.md §3.3`. Concurrent working-tree edits appeared during review; authoritative verification was rerun from `git archive 4a94bc2` extracted to `/tmp/bb-review-4a94bc2`. Those edits were left untouched. All temporary probe sources and logs are under `/tmp`.
- Ran `cargo test --workspace --release` against that snapshot: **105 passed, 0 failed, 0 ignored**. This includes determinism, recording invariance, and both FFI tests. The initial attempt hit a read-only Zig cache; reruns set `ZIG_GLOBAL_CACHE_DIR` and `ZIG_LOCAL_CACHE_DIR` under `/tmp`. The existing Cargo target cache was reused.
- A separate release probe compared `Game::new(seed, false)` with `Game::new(seed, true)` for seeds **0–9**, completing **2,239 pitches**. Scores, entire event-log byte vectors, and digests matched for every seed. The same probe encoded timelines and inspected/sample-checked the scenarios above.
- C compilation with static layout assertions and Rust size/alignment checks agreed: `BbActor` **20 bytes**, `BbFrame` **336 bytes**, `BbPlayInfo` **28 bytes**, all aligned to four bytes. Checked actor position/yaw offsets **4/16**, frame count/actors offsets **12/16**, and play-info telemetry offset **16**. No layout mismatch found in this commit.
- A constructed valid timeline with two contiguous linear Hermite segments sampled correctly before, exactly at, and after their shared boundary. Negative times and infinities selected the appropriate endpoints. NaN returned success and the first position, without propagating NaNs in this finite fixture; that policy is undocumented, but no UB was observed. Twenty actor tracks returned exactly sixteen actors and preserved an adjacent output canary. Eight-byte bad-identifier and invalid-root buffers returned `-1`. Separate short-buffer subprocesses reproduced F1 in both exports.
- Real recorded contact joins for seeds 0–9 had a maximum position discontinuity of **0.07635 m**: seed 3, play 183, across a **1.993 ms** gap. The pitch encoder drops knots after contact without inserting the exact pitch contact endpoint, so the sampler briefly freezes before switching to the realized contact knot. This is smaller than the viewer priorities above; adding the exact endpoint would remove it. The first realized ball sample itself was at contact in these probes, rather than a large post-contact gap.
- Four ten-game C ABI runs advanced, freed every blob, checked final scores/status, called advance again after game over, and destroyed each game successfully. Warm runs took **1.798–1.821 s per ten games** (about **180–182 ms/game**), emitting **33,572,628 bytes** total, about **3.36 MB/game**. Equivalent nonrecording `Game` + timeline-encode runs took **1.750–2.071 s per ten games** and emitted **31,004,876 bytes**. These local timings do not establish a systematic recording slowdown. A separate probe of the retained `GameResult.viz` buffers measured **1.66–2.43 MB/game** of pitch/ball/actor sample capacity, excluding container/string overhead; `res.viz.push(viz.clone())` retains history the streaming C client does not consume until game destruction. No growth across games was inferred or leak claimed.
- Client code inspection found no race in the current call graph: only one `Next` mutates the game; `Sample` verifies/reads a separate immutable blob and does not access the game. Current-blob release follows sampling on the game thread. `EndPlay` joins/consumes the pending future, frees its blob and the current blob, then destroys the game and unloads the library. The invalid end-of-game play stops further requests. `FBbPlay` is copyable despite its “move-only” comment, but the present manual ownership paths do not double-free it.
- Frame conversion is correct: swapping X/Y changes handedness, and sim heading `(sin(yaw), cos(yaw), 0)` maps to Unreal heading `(cos(yaw), sin(yaw), 0)`. The positive Unreal yaw sign in `BbFrame.h` is therefore correct. No Unreal build, screenshots, GUI playback, sanitizer, or Miri run was performed; engine/runtime conclusions are limited accordingly.

## Resolution

Open.
