P1 remains; do not merge.

Rechecked `9ab85f5..ea1218d`, the first review and its Resolution, agent-models §6.4, and ledger row “Runner reads, after the Codex review.”

## Findings

### P1-1 — Not fixed: the held-base retreat can still erase a runner

**Locations:** `sim/crates/bb_game/src/live.rs:3745`, `:3531`; settlement at `sim/crates/bb_rules/src/engine.rs:1145`.

The original `review_late_crowding` now settles safely. But the held-base branch unconditionally sends a runner back, even when `no_way_back` correctly says that base is occupied. It sets `turned_back`, which then exempts that retreat from `return_crowded`.

**Targeted probe `review_three_runner_traffic`:** batter retired; three live runners originally from first, second and third. Runner 9 now holds second; runner 8 is 7 m past second at 8 m/s, heading for third; runner 7 holds third. The middle runner returns onto occupied second. Both stop at `s = 54.864`; `PlayEndCheck` emits `two_runners_on_base`, leaves bases `[None, Some(8), Some(7)]`, and loses runner 9 without an out or run. Setup departures, arrivals and the batter's retirement go through the rules engine. Zero warnings in standard games does not establish this invariant.

### P2-2 — Partly fixed: the retouch plant still supplies an instantaneous stop and an inconsistent race

**Locations:** `sim/crates/bb_game/src/live.rs:3825`–`:3830`, race at `:3491`.

`review_reverse_motion` now moves **0.115 m forward**, at **+6.9 m/s**, on its first tick. The ordinary reversal and second-look regression tests pass, including arrival within three ticks of the signed-velocity estimate.

The retouch exception remains unphysical. `review_retouch_plant` puts a returning runner 0.08 m from second at −7 m/s: he touches it, chooses third, and becomes stationary in one tick. At `BRAKE = 6`, that stop requires about 1.17 s and 4.08 m. Calling it a plant does not supply the missing braking. Moreover, `runner_decide` runs **before** the zeroing and grants its unsigned-speed running-start discount: about **3.018 s** for the next leg versus **4.024 s** from the rest state the plant actually creates. The three older tests passing does not validate this physical rule.

### P2-3 — New: a distant batter can falsely block a return to second

**Location:** `sim/crates/bb_game/src/live.rs:3571`.

`return_crowded` applies `no_way_back` to every trailing runner, without requiring him to be approaching this base's stretch. In `review_distant_batter_crowding`, the batter is only 5 m from home, moving at 3 m/s; his reaction-plus-braking endpoint is 6.35 m. He can stop at first. Nevertheless, his inability to return **home** makes second “crowded,” 49.864 m ahead of him. An unforced runner 8 m past second consequently chooses third, where the ball is already held. Scope the constraint to runners who actually threaten that return base; this is an unnecessary commitment to an out.

### P2-4 — New: the rundown cap also forbids an escape the runner judges safe

**Location:** `sim/crates/bb_game/src/live.rs:3645`.

A bounded commitment is defensible as a temporary policy for a runner trapped at both ends. This unconditional cap also applies when a fresh race identifies a safe alternative. In `review_cap_refuses_believed_safe_escape`, a runner has used his turn and is retreating toward second, where the ball is held. He is 0.5 m short of third at −1 m/s: turning toward third costs **0.503 s**, against his ball estimate of **1.123 s**. The actual decision judges that escape safe, but the cap retains second; he returns into a physical tag.

Read-only counters in the single 300-game outs probe found **212 refused looks**, **20** with a believed-safe alternative, and **166 subsequent tag outs**, including **7** after a refused believed-safe alternative. These are cohorts, not measured counterfactual outs caused by the cap. Restrict the stand-in to trapped situations, or explicitly accept the broader tactical sacrifice.

### P2-5 — New: the rewritten umpire test accepts a wrong-base event

**Locations:** `sim/crates/bb_rules/src/engine.rs:402`; acceptance at `sim/crates/bb_game/tests/umpire_calls.rs:84`.

Seed 0 play 17 reproduces the concern. Runner 260 is tagged returning to first at **8.978003 s**, **1.120 m from first** and **26.312 m from second**. Both `RunnerOut` and `OutCall` record base 2, while the call's 0.136562 s margin measures the return to first. The engine unconditionally uses `r.at + 1` for an off-base tag.

Under the repository's destination-base event contract (data-formats §4.3 and `OutCall.base`), this return play belongs at **first**. The retirement itself is valid; the event/call base is defective. Accepting either neighboring bag masks that mismatch rather than investigating it.

## Confirmed fixes and scope

**P2-1: confirmed fixed for the reported failures.** The shallow-RF/default-arm throw reads third; the over-third throw supplies no false catchable passage; the 5 m-short hop reads home. The earlier chest-height sweep now has **0/1,005** destination errors and the passage sweep **zero false high passes**. The broader mixed-height sweep still has 10/2,395 destination misreads for elevated catches: the chest-height assumption is not a universal destination guarantee. The implementation uses current position/velocity, reference air, typical backspin and known surface physics; it never reads true spin, destination, receiver or future samples.

**Repinned tests:** both seed-1/seed-8 holder situations are found and checked; the rewritten seed-17/play-203 test guards the visible-motion condition and passes. The umpire rewrite has the defect above.

**Turn state/replay:** `turns` belongs to each runner, initializes to zero and resets at a goal-base touch. There is no hidden mutable replay state; the existing cross-thread digest/full-log replay test passes. It is not an explicit base-pair key: a braking runner crossing third can retain a turn spent between second and third (`review_turn_pair_crossing`). Traffic/held-base corrections bypass the counter, so the bound applies to fresh-look reversals, as §6.4 qualifies it. The forced-trailing-runner probe keeps the forced advance and sends the lead runner onward. Neither standard set reaches the play time limit.

## Verification

| Run, once each | Result |
| --- | --- |
| `cargo test --workspace --release` | **395 passed, 4 ignored, zero failures** |
| `sim-cli games --n 300 --seed 0` / `--seed 1000` | **4.85 [4.88]** runs/team-game; `rules warnings: {}` in both, including zero crowding warnings and timeouts |
| `runner_outs_probe 300 0` | **6,318** hit plays: **107** batter tags, **39** other-runner tags; **943** reached-otherwise plays: **310** other-runner tags, **639** forces; runs/team-game **4.852** |

Earlier probes and focused probes of the changed paths used a current-source copy under `sim/target/runner-recheck-probe`; the outs example was copied there unchanged apart from crate references and diagnostic counters. Evidence: `sim/target/runner-recheck-{tests,probes,focused,cap-escape,umpire,outs-300,standard-0,standard-1000}.txt`. No further game sweeps or baseline runs were added. Every Cargo build/test and sim execution ran from `sim/`, prefixed with `taskset -c 0-7 nice`, with two Cargo build jobs and the requested PATH/Zig caches; one heavy job at a time. Only this review was added outside ignored probe/build artifacts; no commits were made.

## Resolution

Open.
