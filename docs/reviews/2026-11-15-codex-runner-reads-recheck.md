P1 remains; do not merge.

> Superseded figures: current numbers are in the [metrics ledger](../metrics-ledger.md) (§1).

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

All five findings fixed on `work/runner-reads`; measured once (ledger row "Runner reads, after the Codex recheck").

- **P1-1, fixed as an invariant where bodies stop** (`settle_guard`, `settles_on`, `refuge_free`). It runs on the final goals of every tick, so the held-base branch, `turned_back`, the cap and the look can no longer bypass it; of two runners settling on a base the trailing one gives way if he is standing on it with a free base behind him and no force, otherwise the lead runner goes on. The decision side agrees (`return_crowded`, and the held-base branch no longer sends a runner back to a crowded base). Evidence: Codex's `review_three_runner_traffic` as `three_runners_do_not_settle_two_on_second` (fails with the guard off), `review_late_crowding` still passes, and `random_runner_layouts_never_leave_two_on_a_base` (3,000 seeded layouts: no two live runners on a base, every live runner on a base, out or scored, zero rules warnings; with the guard off 72 fail). Both standard sets: no warnings, no `play_timeout`. The guard's last resort when no base behind a standing runner is free is that the lead runner goes on; the property test found no layout needing another.
- **P2-3, fixed.** Only a trailing runner who is on or past the base, or heading for it as the next base and unable to stop short of it or with no way back, crowds a return. `a_distant_batter_does_not_crowd_the_return_to_second` (Codex's `review_distant_batter_crowding`).
- **P2-4, fixed.** The one-turn cap applies only to a runner whose look finds no base he believes he reaches (and no crowding); a base he believes safe may be taken. `the_turn_cap_does_not_refuse_a_base_he_believes_safe` (Codex's `review_cap_refuses_believed_safe_escape`). No play reaches the time limit in either standard set, so no total-turns cap was needed.
- **P2-2 (retouch plant), fixed in the smaller honest form the lead allowed, applied to every return to a base:** the approach speed is limited by `sqrt(2·BRAKE·distance)` (1 m/s floor), he brakes at most `BRAKE` a tick when already too fast, he is planted on arrival before his next leg is judged (so that leg starts from rest in the estimate and in the legs), and the return estimate steps the same law (`run_time_to_stop`). `a_runner_retouching_a_base_brakes_into_it_and_leaves_from_rest` (no tick drops more than `BRAKE·DT`, arrival at no more than the approach speed, v = 0 on the bag, trip time within three ticks of the estimate). The tag-up tests pass unchanged. **Not covered:** `review_turn_pair_crossing` (a braking runner crossing third keeps a turn spent between second and third): the counter resets when he stands on a base, as §6.4 states; not keyed by base pair. Deferred with the rundown.
- **P2-5, fixed in the rules engine** (the lead authorised this one change to the official record): `RunnerState.sample_prev` holds the sample before the latest; an off-base tag records the base he returns to when his path progress fell between the two, else the base ahead. Scenarios `tag-003` (tagged returning to second: out at second) and `tag-004` (tagged advancing, with samples: out at third); `tag-003` fails on the old engine. The scenario corpus (136) passes. `umpire_calls` is back to one expected base (seed 0 play 17, the batter caught returning to first, is now recorded at first). rules-spec §5.5 says so.
- **Not raised, noted:** the recheck's 10/2,395 destination misreads for elevated catches in the mixed-height sweep stand (the chest-height assumption is not a universal destination guarantee); no change.
