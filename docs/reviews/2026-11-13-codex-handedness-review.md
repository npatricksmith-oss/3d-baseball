# Handedness review — 2026-11-13

Reviewed `3de5572..bdf19e9` (`claude/a1-173226`) after reading `CLAUDE.md`, the relevant specifications and TASKS “Resume here.” No P1/P2 simulation or client-code regression found. The release suite passes, the published pairing tables and current 300-game calibration reproduce, and independent probes confirm matchup selection, roster compatibility and mirror geometry. Two P3 documentation findings remain.

## P3 — The documented matchup shares do not follow the hand tables or measured games

**Locations:** `docs/data-formats.md:100`; `docs/calibration/2026-11-13-handedness.md:32`; repeated in `TASKS.md:16`.

The specification says about **42%** of appearances against a right-handed pitcher are from the left side, but its own position-player tables imply **45.0875%**: 32.4% fixed left plus 12.6875% switch. Including the pitcher equally in a nine-player lineup gives **45.92%**, rather than explaining the discrepancy. These are lineup expectations; realized PA weighting can vary.

The calibration note separately says about **45% of all PAs** are opposite-handed. Counting actual matchups in its seeds 0–299 gives **11,439 / 22,974 = 49.7911%**. The PA counts by pitcher/batter are R/R **9,449**, R/L **7,474**, L/R **3,965**, L/L **2,086**. Switch hitters are classified by their resolved batting side. This matters when choosing the mixture for the proposed recalibration: left-side share against RHP and opposite-handed share against all pitchers are different quantities.

**Verification:** independently calculated the expectations from the documented position and conditional batting tables; ran a scratch `Game::new(seed, false)` probe over all 300 games, counted pitches by the fielding pitcher's roster hand and each batter's resolved side, and counted PAs on `PlateAppearanceEnded`. The total agrees with the CLI's PA population. Correct the percentages and label whether each is a table expectation or a measured PA share.

## P3 — The new ABI comment references a nonexistent field

**Location:** `sim/crates/bb_ffi/src/lib.rs:88`.

The `BbPlayer.bats` documentation tells clients that `BbPlayInfo.batter_side` identifies the side a switch hitter uses on a play. Neither the Rust structure nor `include/bb_ffi.h` has that field; `BbPlayInfo.batter` is an agent id, not a batting side. Following the comment produces an invalid client implementation.

**Verification:** read both structure definitions and searched the repository for `batter_side`; this comment is its only occurrence. Document the available route instead: resolve the batter and opposing pitcher's hands from the rosters, or inspect the batter's box in the sampled timeline. No ABI layout change is needed to correct the comment.

## Verification

Only this review is added to the tracked tree. No source, test, specification or TASKS file was edited. Scratch Rust sources were supplied to `rustc` through stdin; probe executables, logs and Zig caches are under `sim/target`. No Unreal build or execution was attempted.

### Workspace tests and C ABI

- Ran `export PATH=$HOME/.cargo/bin:$PATH; cd sim && cargo test --workspace --release`. The first attempt could not write Zig's default cache outside the sandbox. Re-ran with `ZIG_GLOBAL_CACHE_DIR=$PWD/target/zig-cache` and `ZIG_LOCAL_CACHE_DIR=$PWD/target/zig-local`: **179 passed, 0 failed**. Cargo's target directory remained `sim/target`.
- Ran `crates/bb_ffi/c/run_smoke.sh` with those cache settings: C client completed seed 3, **203 pitches, 54 out calls, final score 4–2**; roster lookup and timeline ABI checks passed.
- A Python `ctypes` probe independently checked 216 players from seeds 0–11 against Python Philox hand draws. All values match the documented encoding: bats **1 left / 2 right / 3 switch**, throws **1 left / 2 right**. The sample contains bats counts **68 / 117 / 31**, and throws counts **23 / 193**. `BbPlayer` is **80 bytes**, with offsets bats **12**, throws **13**, given **16**, family **48**. The Rust and C declarations agree.
- Read the event-header conversion and stats reader. `Bats` is explicitly converted to schema `Hand::{Left,Right,Switch}`; throwing hands cannot become switch. `bb_stats` reads these fields from the header and converts them to L/R/S, preserving event sourcing. The header and FFI round-trip tests pass.

### Game orchestration and invariants

- Traced `Game::new`, `step_pitch`, `with_fielders` and the live-play call. `mu_ix[batting_side][slot]` uses the **opposing** pitcher's hand and that slot's resolved batting side. Each matchup owns its zone, scouting template and aim seeds. Per-slot scouting is built from that matchup with the appropriate `9_000_000 + side*100 + slot` key.
- Compiled and ran `sim/target/handedness-matchup-probe`: independently constructed all 18 slot matchups and scouting models, maintained previous-read/perceived-speed context per PA, and compared **every `PitchRecord` debug field** against the game for 121 pitches on each of seeds 0–11. **All 1,452 records match exactly.**
- The shared `bs` vector is read-only in this game path. All current matchups have the same four arsenal types and identical `perception.break_sigma`, so sharing it is valid today. The slot-specific scouting is not shared. Previous-pitch context resets when the batter changes.
- The live play receives `self.fattrs[fside]`; its batter box comes from the active batter's mirrored pivot. The scratch matchup probe also supplied all-left throwing hands through otherwise-average `with_fielders` attributes. Full logs stayed identical to ordinary games for seeds **0, 3, 7, 42**, consistent with each player retaining his roster hand.
- Repeated complete games for seeds **1, 2, 3, 42** and compared full log bytes: exact equality each time. Changed digests relative to the all-right-handed parent are expected, not a determinism failure.
- No new outcome roll or agent dependency on `bb_world` was introduced. The orchestrator passes actual pitch/bat cues into the observation boundary; `jump_at_contact` adds each fielder's perception error before `jump_from_reads` uses them. The new throwing-hand offset uses the thrower's own position, hand and target, not executed throw noise. It does not add a new world-state or execution-noise input to decision code.

### Roster compatibility and independent hand distribution

Compiled the parent roster module obtained with `git show 3de5572:sim/crates/bb_game/src/roster.rs` alongside the current module, in memory, and compared **54,000 players** from seeds 0–2999. Player ids, given/family names, origins, numbers, positions and batting orders are identical. `RosterHands = 13` is appended; purposes 1–12 and the original roster stream are unchanged.

An independent Python implementation of Philox4x32-10 used counter `[0, field_agent<<16 | 13, 0, 0]`, key `[seed_low, seed_high]`, and the documented two uniform draws. It matched **all 54,000 hand pairs**, including the committed seed fixture. Left-throwing counts by position, each out of 6,000, were:

| P | C | 1B | 2B | 3B | SS | LF | CF | RF |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1,669 | 0 | 2,100 | 0 | 0 | 0 | 1,532 | 1,169 | 1,489 |

Position-player batting counts out of 48,000 were **15,628 left, 6,138 switch, 26,234 right**. Independently, the tables give left-throw share `(0 + .35 + 0 + 0 + 0 + .25 + .20 + .25)/8 = .13125`, hence batting shares **.324 / .126875 / .549125**. These explain the distribution test's rounded **.324 / .127 / .549** expectations without using its output as the oracle.

The added distribution test and literal throwing-geometry cases check independent expectations. The switch-hitter truth table checks the rule directly. The ABI/header mapping assertions repeat small conversion tables, but exercise real serialization boundaries; they are useful round-trip tests, not evidence by themselves for the generation distribution. The integration test checks actual release and box positions, rather than just inspecting the assigned enum values.

### Mirror geometry and platoon measurements

- Compiled and ran `sim/target/handedness-mirror-probe`. For **all ten pitch types**, independently solved targets `(x,z)=(.13,.8)` and `(-.13,.8)` for RHP/LHP. Release positions and velocities reflect as `(-x,y,z)`; spin, an axial vector, reflects as `(wx,-wy,-wz)`. Across **101 flight samples per type**, maximum reflected position discrepancy was **0 m** in this build. Eye point, swing pivot, contact depth and **101 swing poses/velocities** also mirror, with assertions below `1e-12` for the swing samples.
- Read pitch targeting, body capsule, motor noise, spin construction, `draw_offsets` and world contact resolution. Targets have a symmetric x distribution; the body and eyes change x sign with batting side. The league pitcher in `draw_offsets` inherits the actual pitcher's hand. Independent signed noise draws need not produce a sample-by-sample reflected game; the intended geometry and noise distributions mirror.
- Read `face_from_yaw`, `face_spray_cue` and `jump_from_reads`. Yaw is converted to batter-relative face before fitting; spray is converted back to field coordinates afterward. Pitch inside/outside and pull priors change sign consistently. The existing left/right jump and circular-boundary tests pass. The right-handed fit's unvalidated transfer to left-handers is already disclosed in the specification/TASKS.
- Ran `cargo run --release -p bb_game --example platoon -- 4000 3`, then the built example with `4000 1` and `4000 2`. **Every printed seed-1/seed-2 cell in the calibration table reproduces.** Seed 3 gives:

| Pitcher / batter | K% | BB% | HBP% | In play % | Whiffs/swing % | Mean spray |
|---|---:|---:|---:|---:|---:|---:|
| R/R | 20.0 | 8.0 | .45 | 71.6 | 36.2 | −5.6° |
| L/L | 19.3 | 7.2 | .40 | 73.2 | 34.6 | +4.9° |
| R/L | 13.8 | 7.0 | .35 | 78.9 | 30.0 | +5.6° |
| L/R | 14.7 | 7.2 | .30 | 77.8 | 31.1 | −5.7° |

Same-handed and opposite-handed mirror pairs agree within sampling variation. The roughly **19.8% versus 14.4%** K split reproduces; it is not evidence of a failure to mirror left-handers. HBP remains similarly low on both sides in these samples.

To test the explanation, compiled a scratch copy of `pa.rs` in memory, preserving the pipeline while allowing its eye x coordinate to be overridden. A control run matches the ordinary seed-3 table. Each row below is **4,000 PAs per pairing, seed 3**, with all other settings unchanged except the stated intervention. Centering release uses `PitchDef.release_offset.0 = -pitcher.release_point().x / pitcher.throws.arm_side()` followed by `refresh()`. The no-lateral-spin intervention sets every pitch's `tilt_rhp=12` and `efficiency=1`, followed by `refresh()`; ordinary execution/model noise remains enabled.

| Intervention | R/R K% | L/L K% | R/L K% | L/R K% |
|---|---:|---:|---:|---:|
| Control | 20.0 | 19.3 | 13.8 | 14.7 |
| Eyes at x=0 | 26.3 | 26.0 | 21.9 | 21.8 |
| Release at x=0 | 19.4 | 18.6 | 13.9 | 15.4 |
| Eyes and release at x=0 | 25.6 | 24.6 | 23.0 | 22.7 |
| Eyes moved to the opposite side | 25.6 | 26.3 | 23.7 | 23.2 |
| No nominal lateral spin | 17.1 | 16.3 | 15.1 | 16.3 |
| Eyes/release centered, no nominal lateral spin | 24.8 | 24.6 | 24.4 | 24.9 |
| Exact scouting (`model_error=None`) | 12.2 | 11.9 | 8.0 | 8.3 |
| No pursuit penalty (`pursuit_limit=1e9`) | 15.2 | 14.3 | 11.1 | 11.3 |
| Fixed contact depth (`contact_k=0`) | 18.9 | 18.7 | 13.2 | 14.4 |

**Interpretation:** eye geometry affects the rates, but neither the eye offset nor release side alone explains the split. A substantial split remains with both centered, and becomes negligible in this sample when nominal lateral movement is also removed. Exact scouting does not remove it either. These interventions support a combination of viewing/perception geometry and handed pitch movement interacting with the swing; they do **not** isolate “movement shows late down the line of sight” as the sole mechanism, or establish the eye offset as the right calibration control. The note's broad geometry/movement explanation is plausible; its specific causal story remains a hypothesis. No separate handedness sign bug was demonstrated. These are counterfactual PA populations, not an additive decomposition of the original gap or a real-world validation.

### Throws and Unreal client inspection

`throw_release_point` uses the correct arm-side perpendicular in the sim frame: facing +Y, a right-hander releases toward +X; facing +X, toward −Y; facing home (−Y), toward −X. The added literal-coordinate tests cover these cases. `throw_to` solves horizontal aim and distance from the offset release point. Believed travel times still start at the body center; because the offset is perpendicular to the target direction, distance becomes `sqrt(d*d + .35*.35)`. At the direct-throw threshold of 6 m the difference is about **1.02 cm**, rather than 35 cm. This does not suggest a material planning mismatch in the current model. More detailed footwork/glove effects are explicitly deferred.

By reading `BbSim.{h,cpp}` and `BbDirector.{h,cpp}`:

- `Start` fills `LeftHanded` before `RequestNext` launches the first worker. The set is subsequently read-only, and the director creates a new `FBbSim` for this start. No concurrent roster read/mutation was found in the actual call path.
- Pitch/throw mirroring uses the player's throwing hand; batting mirroring uses the actual box x coordinate, so switch hitters and players whose batting/throwing hands differ are handled correctly.
- The mesh faces local +Y, so negative local X reflects left/right across its facing. Every **visible** figure, including staged and exiting players, goes through `PoseFigure` each rendered frame with a frame available. Hidden figures are not posed, but are posed before their next visible rendering; `bMirror` starts false on every call and the X scale is set from its absolute magnitude. I found no visible stale-mirror path. This is a code inspection, not visual verification of Unreal's rendered animation.

### Game calibration and final checks

Ran `cargo run --release -p bb_cli -- games --n 300 --seed 0`: **5.35 runs/team-game, 15.3% K, 7.0% BB, .3% HBP, 5.21% HR/PA**, GB share **50.9%**, GB hits **.346**, FB hits **.124**; **no rules warnings**. These match the note's current seeds-0–299 column at its printed precision. The parent calibration column, seeds-1000 batch, dive batch and external MLB comparisons were not independently rerun/validated.

`git diff --check 3de5572..HEAD` passed. The simulation's increased run environment is already disclosed as an open calibration issue; this review does not treat that disclosed tuning debt as a newly discovered implementation defect.

## Resolution

Resolved 2026-11-13.

- **P3, matchup shares:** fixed. data-formats §2.4 now says 45% of position players' turns against a right-hander are from the left side (the 42% was an arithmetic slip). The calibration note and TASKS give the measured opposite-handed share, 49.8%, with the review's counts.
- **P3, ABI comment:** fixed. The comment now points to the batter's box (sim x > 0 when he bats left), which is what the client uses; there is no `batter_side` field.
- **The cause of the platoon gap:** the review's interventions showed the note's story (the eye offset) was a guess that does not hold alone. The note now carries the intervention table and says what it supports: sideways pitch movement meeting a batter who sees and swings from one side. The gap's size stays an open model question in TASKS.
- Not done, as the review notes: the left-handed stand-in motions were checked in screenshots for a batter and a pitcher only, not for a fielder's throw.
