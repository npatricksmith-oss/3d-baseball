# Codex review: dive animation and lunge

Reviewed `f4d3167..009f31e`, with `HEAD` at `009f31eaad4dc7d728d6896cc0bc19461c14261c`, after reading `CLAUDE.md` and the preceding dive/two-out review. Locations refer to `009f31e`. No source, existing test, spec, or TASKS file was edited. `TASKS.md` was already modified when this review began. This document is the only authored repository file; diagnostic binaries and logs are under `sim/target/`.

## Findings

No P1 finding. Three P2 defects and two P3 presentation defects remain.

### 1. P2 — The lunge can carry the body through the modeled wall

**Locations:** `sim/crates/bb_game/src/live.rs:1873` (landing point), line 893 (movement); `sim/crates/bb_physics/src/ground.rs:52` (fence shape).

The landing point and every lunge step follow an unchecked straight segment toward the ball. Being short of an in-bounds ball does not guarantee an in-bounds segment: the reference park's quadratic fair-territory fence meets its constant-radius foul-territory fence at an inward corner. The new body movement can cut across that corner and stop outside the wall.

**Verification:** A fair ground-ball fixture put the fielder and ball 1 mm inside their respective fence radii, at angles `π/4 - 0.006` and `π/4 + 0.018`. The actual capture code committed a dive across the **2.439539 m** gap. Its movement took the body **0.232968 m outside** the modeled boundary and left it **0.125097 m outside**, at `(71.789630, 70.629818)`. Both initial endpoints were inside. The play remained live. This used the existing reference park, perfect hands, and the production `try_captures`/`move_fielders` code.

**Impact / suggested fix:** Constrain the swept lunge and its recorded landing point against the park boundary. A distance cap alone is insufficient. General fielder collision handling was already absent; this finding concerns the newly introduced movement during a dive and does not claim a measured frequency in ordinary games.

### 2. P2 — A second dive at the previous get-up time disappears from the FFI

**Location:** `sim/crates/bb_ffi/src/lib.rs:303`.

Pairing accepts `DiveUp.t >= Dive.t`. If the same fielder dives again exactly when his previous recovery ends, the second `Dive` pairs with the **previous** `DiveUp` at that same time. The subsequent `up.t > t` filter rejects it throughout the second dive. Recovery permits this boundary (`down` uses `t < ready_at`), and conversion to `f32` can also collapse almost equal times onto it.

**Verification:** A valid synthetic timeline contained one fielder's `Dive(1.0), DiveUp(2.5), Dive(2.5), DiveUp(4.0)`. Sampling at **2.5 s and 3.0 s returned `n_dives = 0`**, instead of one. The same test with the second dive at 4.0 s passed, including its changed direction and exact end boundary.

**Impact / suggested fix:** The client shows the second lunge/recovery as ordinary standing or movement. Pair with the strictly later get-up for these positive-duration dives, or establish explicit per-fielder pairs when decoding. Add the adjacent-dive case; a whole-game test with separated dives does not cover it.

### 3. P2 — A play ending during a dive freezes or truncates its presentation

**Locations:** `sim/crates/bb_game/src/timeline.rs:194` and line 288; `client/Source/BaseballClient/BbDirector.cpp:1365`, line 1473; `ResetFrame` at line 1005.

`DiveUp` is emitted even beyond `duration_s`, but the duration does not include it. The client clamps sampling to that duration, so the diver's `since_s` and `up_in_s` stop advancing during `HoldAfter`. Reset then discards dive state and treats the player as standing. If the play ends on the dive's capture tick, simulation exits before `move_fielders`; the actor track never even reaches the recorded landing point. Extending duration alone would therefore not complete those lunges.

**Verification:** Across **1,482 pitches** at seeds `0, 1, 5, 7, 11, 7475`, **7 of 140 dives** were still active at timeline end. For seed **5 / play 4**, both dive and play end are **7.333155 s**: the actor track ends at `(-62.045188, 37.493815)`, although `DiveUp` promises landing at `(-63.410074, 37.965357)` 1.5 s later. At the final sample the FFI reports `since_s = 0`, `up_in_s = 1.5`. Seeds **5 / 164**, **7 / 168**, and **7475 / 234** likewise end at dive initiation. Other cases freeze a landed fielder before his rise.

**Impact / suggested fix:** Complete the recorded dive's post-play presentation, including the remaining lunge and recovery, before reset staging takes over. Keep this separate from resuming game adjudication after the deciding fact. The documented Dive/DiveUp bracket currently describes a completion the viewer may never see. `-BbDiveShots` also cannot reach all three phases of these truncated dives.

### 4. P3 — Starting the rise abruptly resets the dive's layout transform

**Locations:** `client/Source/BaseballClient/BbDirector.cpp:1130`, line 1185, and `PlaceDiver` at line 1058.

At `up_in_s = 0.6`, the figure still uses `AnimDive` with a 45° tilt, a 40 cm downward offset, and a `90·sin(45°) = 63.64 cm` backward offset. Immediately below 0.6 s, the branch switches to `AnimCrouch`; because the layout code applies only to `AnimDive`, tilt and both offsets reset to zero in one frame. `PlayAnimation`/`SetPosition` also switch the held clip directly. There is no rise interpolation for this actor transform.

**Impact / suggested fix:** The layout itself jumps **63.64 cm forward and 40 cm upward**, with a 45° rotation change, at the start of every rise. Preserve continuity across the clip handoff and ease the layout offsets through the rise interval. This is established by reading the transforms; Unreal and the asset poses were not run, so the exact rendered appearance remains unverified.

### 5. P3 — The pitch pose overrides an early pitcher dive

**Location:** `client/Source/BaseballClient/BbDirector.cpp:1116`.

The pitch branch precedes the dive branch and remains active through play time 0.8 s, regardless of `DiveSince`. A pitcher diving on an early comebacker therefore keeps the delivery pose and plate-facing yaw while his sim body starts lunging, then switches partway into the dive.

**Verification:** The six-game timeline scan found five actual pitcher dives before 0.8 s, including seed **0 / play 12 at 0.744100 s**, seed **11 / play 20 at 0.785181 s**, and seed **7475 / play 25 at 0.728028 s**. These satisfy the conflicting branch when the pitch asset is loaded.

**Suggested fix:** Give an active dive priority over delivery, or end the delivery pose when a live defensive action begins. This affects presentation only.

## Previous review resolutions

All three specific fixes claimed in the preceding review's Resolution are implemented and check out:

1. `Sim::out_weight` requires `r.s < goal * BASE_L` as well as the surviving force chain and destination. The regression test passes: short of second gets weight 1.0; returning from beyond second gets the tag weight. No rules-private runner state was added to this decision.
2. The next-tick capture deferral applies `DIVE_MAX_Z` when the fielder will still be down. The supplied rising-ball regression passes, alongside the existing recovery/re-fielding tests.
3. A downed receiver uses `up_at + RELAY_TRANSFER_FRAC * transfer`, rather than shortening his original get-up. An additional thrown-ball probe with get-up at **2.5 s** and transfer **0.85 s** produced readiness at **3.095 s**, as required. The spec now also states both retry blocks.

These fixes do not resolve the separate findings above.

## State, rules, ABI, and specification checks

- **Lunge bounds other than walls:** the displacement is `max(horizontal_gap - 1 m, 0)`, at most 1.5 m under the dive eligibility test. Its interpolation clamps at the landing point; it does not overshoot the ball's position at commitment. It can cross a foul line, which is legal playable territory. The modeled fence boundary is the issue in finding 1; there is no additional separate foul-territory limit in this field model.
- **Prone tags and base touches:** `holder_act` deliberately performs these before its transfer gate and reads the moving `fd.pos`. A probe made a force at first while the holder was still down; a separate geometry probe emitted a close tag while down. A fielder need not stand to make either contact under the existing rules model. Prohibiting all such contacts until get-up would be an incorrect blanket restriction. Reach is still the existing point-body approximation, without limb/contact pose geometry.
- **Other position consumers:** lunge movement changes visible distances for backups, relays, coverage, receiver estimates, and the holder's run-to-base choice. Those changes are consistent with moving the authoritative body. `receiver_ready` still estimates travel from visible distance at typical standing-start speed, without a recovery delay; the holder's run-to-base estimate likewise omits remaining recovery. Those approximations predate this commit. Movement itself remains gated by `ready_at`, so assigning a run target does not let a prone fielder run early. The existing freeze through transfer remains after a successful diving capture, while the new visible get-up marker is at dive + 1.5 s.
- **Held-ball height:** the 0.3 m value does not drive a new physical or rules decision in the current call paths. Held-ball runner timing uses horizontal distance; captures return for a held ball; throws launch separately at `THROW_RELEASE_Z`; tags and base touches use fielder/runner positions. Its changed vertical coordinate affects the recorded ball track. Recording on/off comparisons passed.
- **ABI:** the C header and Rust `#[repr(C)]` layouts agree on this x86-64 target: `BbActor` 20 bytes, `BbDive` 20 bytes, `BbFrame` 456 bytes, alignment 4; `holder`, `release_in_s`, `n_dives`, and `dives` offsets are 364, 368, 372, and 376. Both sides use four inline dives. `DiveUp = 20` is appended without renumbering prior markers, and generated Rust matches the schema.
- **Overflow:** five simultaneous synthetic dives return the first four with `n_dives = 4`; no out-of-bounds write occurs. The fifth is intentionally omitted by the fixed-capacity API. The six natural games reached only one simultaneous dive, which does not prove a production maximum of four. The limit is named in the header; documenting truncation in the prose would help consumers, but no memory-safety defect was found.
- **Client isolation:** the changed client code samples completed sim data and modifies poses, camera placement, and screenshots. It does not adjudicate outcomes. Dive takes priority over throw and catcher poses; the batter branch applies only to non-fielders. Reset zeroes the expanded frame and does not accidentally reuse its dive array. The missing recovery handoff and pitch precedence are findings 3 and 5. `-BbAnimTest=dive` is a presentation-only preview that returns before starting a game.
- **Specs and invariants:** agent-models §5.3 matches the lunge distance/time, held height, transfer rules, and retry blocks; data-formats §5.4 matches marker values and payloads, subject to the completion/pairing failures above. No new outcome roll, RNG purpose/draw, native sim transcendental call, unordered iteration, animation-to-sim feedback, or non-event-derived statistic was introduced. The dependency test passes.

## Verification

Commands ran from `sim/`, with Cargo outputs in `sim/target/` and writable Zig caches:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
export ZIG_GLOBAL_CACHE_DIR="$PWD/target/zig-cache"
export ZIG_LOCAL_CACHE_DIR="$PWD/target/zig-local-cache"
cargo test --workspace --release
./crates/bb_ffi/c/run_smoke.sh
cargo build --release -p bb_cli
target/release/sim-cli games --n 200 --seed 1
target/release/sim-cli games --n 200 --seed 1000
target/release/sim-cli games --n 200 --seed 2000
```

| Check | Result / log under `sim/target/` |
| --- | --- |
| Release workspace tests | **157 passed**, zero failures; `dive-animation-review-workspace.log`. Includes dependency, determinism, rules corpus, accounting, and FFI coverage. |
| Linked C smoke client | Passed: **238 pitches, 54 out calls, final score 5–4**; `dive-animation-review-c-smoke.log`. |
| Independent C layout assertions and Rust offsets | Passed; `dive-animation-review-c-layout.log` and `dive-animation-review-timelines.log`. |
| Existing live/play/umpire tests plus three diagnostic probes | **58 passed**, zero failures; `dive-animation-review-probes.log`. Probes exercise prone contact geometry, a simple near-wall lunge, and receiver get-up. |
| Foul-corner wall and prone-force probes | **2 passed**, reproducing the wall defect and a legal prone force; `dive-animation-review-wall.log`. |
| Synthetic FFI marker cases | Separated repeated dives, exact start/end, a get-up after duration, and five-dive capacity check passed. Adjacent repeated dives reproduced finding 2; `dive-animation-review-timelines.log`. |
| Six natural games with timelines | **1,482 pitches, 140 dives**, seven truncated dives and five early pitcher dives; `dive-animation-review-timelines.log`. Independent shared-library sampling agreed on the seven active-at-end cases: `dive-animation-review-ended-dives.log`. |
| Recording off versus on, seeds 0, 11, 7475 | Event-log bytes and digests identical for each complete game; `dive-animation-review-timelines.log`. |
| Three 200-game CLI cohorts | **5.67, 5.76, 5.91 runs/team-game**, respectively; all warnings `{}`. Logs: `dive-animation-review-games-seed{1,1000,2000}.log`. Their rounded mean is **5.78**, reproducing the commit's reported figure. GB hit rates .429, .434, .425 support its approximate .429 claim. |

The preceding review independently measured the parent's pooled rate as **5.849167**; the present endpoint agrees with the claimed modest decrease. The parent cohorts were not rerun here, and no new population-level calibration conclusion is inferred from these matched seed arguments.

Diagnostic Rust and C source was assembled in memory and supplied to compilers on stdin; no scratch source or test file was written. Rust probes used the checked-out production modules, release dependencies, edition 2024, optimization level 3, and the configured Zig linker. Initial harness setup encountered mismatched FlatBuffers artifacts, an incorrect API name, a receive fixture sampled before closest approach, and a ground touch short of the fair-establishment boundary. Those setup issues were corrected before the final results above. Unreal was reviewed by reading only, as requested; it was neither built nor run.

## Resolution

All five findings are fixed (2026-11-13, the commit after `009f31e`).

1. **P2, the lunge through the wall:** fixed. The landing point is checked at 15 points along the lunge (`LUNGE_CHECKS`), and the body stops at the last one at least `BODY_R` = 0.3 m [E] inside the fence. Test: `a_dive_never_carries_the_body_through_the_wall` (Codex's fixture; it fails without the clamp, 0.030 m past the wall). agent-models §5.3 says so.
2. **P2, adjacent dives dropped by the FFI:** fixed. The pairing is now `active_dives`, a plain function: each `Dive` pairs with that fielder's first `DiveUp` strictly after it. Test: `a_dive_starting_as_the_last_one_ends_is_reported` (Dive 1.0, DiveUp 2.5, Dive 2.5, DiveUp 4.0: one dive with the second dive's direction at 2.5, 3.0 and 3.9 s, none at 4.0).
3. **P2, a play ending during a dive:** fixed in the timeline (presentation only). For a dive whose `DiveUp` is past the diver's track, the track gets a knot at the lunge's end (the landing point) and at the get-up, and `t_end` (so `duration_s`) covers the get-up. The game's adjudication is unchanged. The FFI whole-game test now asserts that no fielder is still down at the clip's last frame. data-formats §5.4 says so.
4. **P3, the rise jumps:** fixed. Over the first half of the rise the layout (tilt, drop, and the body offset that follows the tilt) eases off with a smoothstep from where he lay. A headless capture 0.58 s before he is up shows him still on the turf instead of 64 cm forward and 40 cm up.
5. **P3, the pitch pose overrides an early dive:** fixed. The delivery pose applies only while the pitcher is not diving (`In.DiveSince < 0`).

The overflow note (more than four dives at once are left out) is now in data-formats §5.4. 3 × 200 games after the fixes: 5.67 / 5.75 / 5.91 runs/team-game (the wall clamp moved one batch by 0.01).

