# Codex review: first-base overruns, runner samples, and swing starts

Reviewed `4132571..a36034c` (`5c82bda`, `a36034c`). Read `CLAUDE.md` first and checked the binding invariants, changed specifications, calibration note, and downstream rules, game, statistics, and timeline code. This report is the only file written outside ignored scratch work under `sim/target/`. No source, existing test, spec, or TASKS file was edited.

## Findings

No P1 or P2 defect was established. One P3 clarification remains.

### 1. P3 — Narrow the statement that rounding first and returning is unimplemented

**Locations:** `docs/agent-models.md:517`; `docs/calibration/2026-11-13-overrun-and-samples.md:70`; `sim/crates/bb_game/src/live.rs:2195`.

The new running specification says a batter-runner either continues or runs through first, and lists rounding toward second and coming back as unmodeled. The existing blocked-base branch can already reverse an ordinary advance: a batter-runner whose goal is second retreats to first when a preceding runner stops at or retreats toward second. This uses ordinary path movement, without entering `OverrunPhase`.

**Evidence:** Scratch test `review_normal_advance_can_become_round_and_retreat` starts the batter-runner 3 m beyond first toward second, with the preceding runner now holding second. Unmodified movement code changes `goal` from 2 to 1 and retouches first at **4.767 s**, with `OverrunPhase::None`. Separate sample/tag probes confirm that a 1 m excursion remains protected, while a 3 m excursion ends protection permanently, including after retreat. Those judgments agree with rules-spec §5.5's geometric approximation; this is a documentation finding, not a request to change them. Recorded tracks from seeds 0–511 contained **zero** such round-and-return cases among the 8,421 reported returns, so the calibration count itself reproduces.

**Suggested clarification:** Distinguish the deferred voluntary rounding/return decision and curved movement from the existing retreat caused by a blocked destination. Mention that the latter already receives the sample-based attempt judgment. The new diagnostic's touch/leave/retouch sequence alone cannot distinguish these paths; the recorded geometry did distinguish them in this review.

## Rules and integration checks

- **Passing geometry:** For the current straight, one-dimensional base paths, projection preserves path progress. A grid of **798 forward/retreat positions**, using the appropriate last touched base, had maximum error **4.441e-16 bases**. Initial leads plus the batter-box position produced neither a pass nor a warning. The box offset is confined to the batter's first 4 m, far behind the nearest preceding runner. Movement emits intervening base touches during retreats, keeping the projection's adjoining segments current. Protected overrunners compare at first, rather than at their distance down the foul line. No false pass from these mechanisms was established.
- **Pending home touches after a catch:** The catch-processing branch clears the simulator's `scored` flag before sampling a runner returning to retouch. A targeted catch/return probe placed that runner at progress 3.4 and the follower at 3.6; `sample_runners` correctly emitted the follower's passing out before the retouch. It did not exclude the returning runner or confuse his pending run with his physical position. This agrees with the passing rule and its lead-runner-action comment. [2026 OBR 5.09(b)(9)](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf#page=60)
- **Rounding protection and sampling phase:** With samples already present, the first `BaseLeft` grants provisional protection to every batter-runner, including one continuing toward second. Along first-to-second, the 1.5 m fair-side threshold corresponds to approximately **2.121 m of path travel**. A sampled 3 m excursion loses protection permanently; retreating afterward does not restore it. A 1 m excursion does not lose it. At the same 2.25 m position, a tag was safe without the newest sample and an out with it. This is the explicit §5.5 “Starts” trade-off, not phase independence. Tags are processed before that tick's samples, so the latest judgment evidence can lag movement. Moving sampling from even to odd ticks nevertheless left **all 512 game logs byte-identical** for seeds 0–511. That cohort result does not guarantee phase independence for every possible crossing or tag; passing also has finite sampling resolution.
- **A real tag during the straight overrun:** The existing integration test injects a tag on the return. Additional probes kept a ball holder within tag distance, invoked normal `holder_act`, and explicitly submitted physical tags during the return. The holder made no play on the overrunner and the rules rejected the tags. Without samples, the existing counterpart test retires him. Protection through an immediate return agrees with the first-base exception; attempt judgment and delayed-return appeals remain the documented approximations. [2026 OBR 5.09(b)(4), (11)](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf#page=58)

## Ordering, ending plays, and the overrun model

The production facts emitted before sampling are at or before `sim.t`; captures, tags, and base movement use that tick's time. `ball_facts` drains newly due contacts each tick in timestamp order, with its existing fact-time floor. A newly due contact cannot predate the preceding sample: contacts through that preceding tick were already drained. Although `last_fact_t` excludes samples, that omission did not establish a reversal. Both 512-game CLI batches and the 256-game diagnostic printed `rules warnings: {}`; the paired phase audit also found none.

At 4.0, 8.2, and 12.0 m/s crossing speeds, direct movement probes returned after **2.483, 6.233, and 11.017 s**, reaching **1.967, 8.337, and 17.900 m** past first. None timed out. During the overrun, `s`, `goal`, and `last_base` stayed at first, `settled()` stayed false, a waiting flag survived, and no new runner decision executed. The early overrun branch excludes ordinary waiting/retreat movement until return. Catch retouch logic applies to runners starting above base zero, whereas this phase is entered only by the batter-runner; a catch retires him through `absorb`.

The holder's exclusion is a reasonable visible-path approximation here: on the transition tick the runner still touches first; afterward his position and movement show the straight overrun/return. It adds no decision access to executed motor output or hidden ball state. It should be revisited when the model permits an overrunner to change his mind, which §6.2 explicitly defers.

A targeted third-out force elsewhere while the batter-runner was protected off first ended the play with **LOB = 1** and no warning. Across 512 games, **121** surviving off-first endings were third-out/game endings and **835** were dead-ball endings. Box-score score, PA, pitch, and LOB totals matched the event log in every game. Encoding and decoding **673 pitch timelines** from seeds 0, 13, and 22 found finite, ordered actor knots and valid `view_after` states. Seed 13/play 183 and seed 22/play 113 preserved their terminal overrun positions exactly to float storage precision; three off-first dead-ball awards continued to home. No client-facing encoding or box-score failure attributable to the protected overrun was established. This was headless verification; the Unreal client was not watched.

## Swing bound and invariants

Delaying `t_c` preserves width, peak angular speed, and swing geometry, giving later contact without inventing a faster swing. This matches §3.4's specified consequence. The first bound runs before `t_abort` is computed, so that abort boundary follows the delayed initial swing. The second bound applies after each adjustment. Setting `t_brake` immediately breaks the adjustment loop, so neither current call shifts an already-braked swing; copying `t_brake` afterward introduces no new check-swing rescheduling. The helper itself does not translate a pre-existing `t_brake`, so its current use on unbraked plans matters.

A **216-case** finite-input sweep preserved the translated speed profile and initial abort timing and always satisfied the bound. Cancellation stress cases needed up to **31 `next_up` iterations**; treating the correction as necessarily one ulp would be inaccurate. The loop converged in every tested case. There was no early start among 8,199 executed HEAD swings.

| 64 games, seeds 0–63 | Executed swing records | Starts before decision | Maximum lead |
| --- | ---: | ---: | ---: |
| Actual `4132571` baseline | 8,172 | 12 (0.146843%) | 17.932729 ms |
| HEAD live model, only the two PA bounds removed in scratch | 8,199 | 13 (0.158556%) | 17.932729 ms |
| Actual HEAD | 8,199 | 0 | 0 ms |

Thus **“0.16%, up to 18 ms” reproduces** for the changed live-play environment without the bound, rather than literally for the old commit's game population. That scratch counterfactual also reproduces seed 11/play 227's **7.346196 ms** early start. Counts include executed check attempts, including those ultimately judged takes, rather than only `record.swung`.

`PitchRecord` aim/commit errors remain errors of the intended aim; they are not executed timing errors and need not absorb this execution constraint. `foul_diag` computes executed timing from `sw.t_c` and re-resolves the recorded swing against the release. Its 3,000-PA run completed with those consistency assertions intact: **4,381 judged swings, 2,835 contacts, 23.7% foul/contact**, and **4,357 competitive swing measurements**. No new diagnostic mismatch was established.

The diff adds **no RNG calls, draws, or purpose values**, unordered collection iteration, native transcendental calls, or fused operations. Existing draw order within a call is unchanged; altered game paths can naturally change which existing keyed calls occur. The agent-to-world dependency test passes. Repeated seeds 0, 11, and 59, including recording on versus off, produced byte-identical logs and digests. Statistics continue to derive from logged events. `RunnerPassed` being serialized only through its `RunnerOut { kind: Tag }` remains the explicit §15 deferral. `OVERRUN_DECEL`, `OVERRUN_RETURN_MPS`, the reused `RUNNER_TAU`, sampling cadence, and the rules' attempt/passing margins are named parameters; no newly buried [E] literal was found. Apart from finding 1, §§3.4/6.2 and rules-spec §§5.5/15 describe the implemented behavior and trade-offs.

## Verification

Commands ran from `sim/` unless otherwise stated, with `PATH="$HOME/.cargo/bin:$PATH"`. Scratch builds used `CARGO_TARGET_DIR` under this checkout's `sim/target/`; Zig caches were also redirected there when linking required writable caches. Scratch Cargo commands used `--offline` after the initial index lookup failed. All final commands below exited 0.

| Command / probe | Output / result |
| --- | --- |
| `cargo test --workspace --release` | **151 test functions passed**, zero failed, including both overrun integration tests, swing-start, dependency, determinism, FFI, and scenario-corpus checks. Log: `target/review-workspace-tests.log`. |
| `cargo run --release -p bb_cli -- games --n 512 --seed 0` | **5.46 runs/team-game**; warnings `{}`. Log: `target/review-games-512.log`. |
| `cargo run --release -p bb_cli -- games --n 512 --seed 2` | **5.24 runs/team-game**; warnings `{}`. CLI seeds are `2 * 1_000_003 + i`, not consecutive seeds starting at 2. Log: `target/review-games-512-seed2.log`. |
| `cargo run --release -p bb_game --example overrun_probe -- 256 0` | **15,429** batted balls ending a PA; **10,036** first touches; **4,178** overrun returns; **1,423** continued advances; **4,407** off-first endings (3,909 own outs, 61 third-out/game endings, 437 dead balls); 20 never-left outs, 8 never-left survivors; anything else **0**; passing **0**; warnings `{}`. Log: `target/review-overrun-256.log`. |
| `cargo run --release -p bb_game --example foul_diag -- 3000` | **4,381 swings / 2,835 contacts / 23.7% foul/contact**; re-resolution assertions passed. Log: `target/review-foul-diag.log`. |
| `cargo test --offline --release --manifest-path target/review-probes/Cargo.toml review_ -- --nocapture` | **7 focused probes passed**: projection/leads/box, pending-home return/pass, rounding/retreat/tag phase, overrun speed/waiting/defense, blocked-base retreat, protected-overrunner LOB, swing-bound arithmetic. Log: `target/review-targeted.log`. |
| Scratch `audit` binary, same manifest | **512/512 complete**, no warnings, box-score/event agreement; even/odd sampling schedules produced **0 different logs**; repeat/recording comparisons passed. Log: `target/review-phase-audit.log`. |
| Scratch `classify_returns` binary, same manifest | Recorded 512 games: **8,421** first/leave/first sequences, **0** rounded returns, **8,421** straight overruns. Log: `target/review-return-classification.log`. |
| Scratch `timeline` binary, same manifest | **673** pitch timelines verified; **2** terminal off-first tracks preserved; **3** off-first dead-ball awards finished at home. Log: `target/review-timeline.log`. |
| Scratch swing measurement at `4132571` | Exported the baseline workspace into `target/review-baseline/` using `git archive`; ran its scratch `review_swing_measure` example with target `target/review-baseline-build/`. **12/8,172 early**, max **17.932729 ms**. Log: `target/review-baseline-swings.log`. |
| HEAD and unbounded scratch swing measurements | HEAD **0/8,199**; copied HEAD game with only the two PA bound calls removed **13/8,199**, max **17.932729 ms**. Logs: `target/review-head-swings.log`, `target/review-unbounded-swings.log`. The scratch sources were subsequently restored to bounded PA code for the phase comparison. |

Probe sources and intermediate copies remain ignored under `target/review-probes/`. During probe development, an initial third-out fixture had not yet established a fair ball, and an artificial 16-iteration rounding cap was too strict; correcting the fixture and measuring convergence produced the final seven passing probes above. These were scratch-harness assumptions, not product test failures. Initial linking attempts that used the read-only default Zig cache were rerun successfully with writable caches.

## Resolution

Resolved 2026-11-13. No P1 or P2.

1. **Fixed** (docs only). agent-models §6.2 now separates the voluntary turn toward second and back (not modelled) from the retreat to first that a blocked base already causes, which the samples judge for an attempt as rules-spec §5.5 describes. The calibration note says the same, and also that no such retreat happened in the 512 probed games.

Also from the review: the swing-start rate was measured on the game paths after the overrun change; at `4132571` it was 12 of 8,172 swings (0.15%), up to 17.9 ms. agent-models §3.4 and the calibration note give both. Codex measured up to 31 `next_up` steps in stress cases. The loop comment said "an ulp"; it now says the shortfall can be many ulps (comment only).
