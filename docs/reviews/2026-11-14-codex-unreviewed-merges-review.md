# Independent review: four unreviewed merges

Reviewed main at `57f12b7`, after reading `CLAUDE.md`. Used `git log --oneline --grep` and `git show` to trace:

1. Set-throw effort: `4477892`, merge `0e10efd`, and its subsequent changes present on main; agent-models §5.5, `bb_agents::throwing`/`motor`, `bb_game::live` and `live/`.
2. Scorer fixes: `2a21ef5`, `73155fb`, `3c53a87`, `6310015`, merge `5317f84`; the Gemini review and Resolution, scorer, evidence producer/schema, captions, rules-spec and data-formats.
3. Box score: `0156347`, `d772600`, `1373784`, merge `27dff42`; Rust/C layouts, snapshots and client consumption.
4. A16 and side changes: `4cf8da0`, `3655784`, their implementation commits, and `1b2941d` (side-change cut and stepped speeds); timeline, telemetry, physics producer and client playback.

Swing v2 and the refit were present during current-main verification but were not reviewed. No source, tests, specs, ledger or TASKS were edited. No branches were switched or commits made. Temporary programs, archived historical sources and outputs are under `sim/target/`.

## Findings

### P1-1 — Change 2: a throw after first base can incorrectly turn a single into a double

**Location:** `sim/crates/bb_stats/src/scorer.rs:145` (introduced by `2a21ef5`).

The new `while_batter_short(t.t_release())` restriction is appropriate when deciding whether the defense chose another runner instead of retiring the batter at first. It is also applied to the separate extra-base credit calculation. A batter already safe at first can still take second on a throw at a preceding runner. OBR 9.06(b), including its right-fielder/throw-to-third example, requires distinguishing that advance from a double.

**Probe:** hand-serialized evidence has R1, batter touching first at 4.0 s, right fielder throwing at R1 at third at 5.0 s, `batter_goal = 1` at release, R1 reaching third, and batter subsequently touching second at 7.0 s. The scorer returns `Hit(2)`; the expected ruling is `Hit(1)`. The existing accounting identities all permit this wrong extra-base credit.

**Suggested fix:** retain the timing gate for the hit-versus-fielder's-choice decision, but remove it from consideration of advances on subsequent throws. Apply the logged goal/advance heuristic to those throws too. Add the rule's example with the throw occurring after the batter's first-base touch.

### P1-2 — Change 2: a later throw to first can erase a hit already earned

**Location:** `sim/crates/bb_stats/src/scorer.rs:101–106` (introduced by `2a21ef5`).

The replacement for the release-time comparison uses the initial reference margin for every last throw to first that goes out of play. It does not check whether the batter had already reached first before that throw. An ordinary-effort counterfactual at the original fielding time cannot establish that a later return throw caused his original safe arrival.

**Probe:** batter touches first at 4.0 s; the original reference margin is +0.30 s; a non-out-attempt return throw to first leaves at 6.0 s and goes out of play, awarding additional bases. Result: `ReachedOnThrowAward`, rather than retaining `Hit(1)` and excluding the subsequent award from its value. This is a controlled scorer-input reproduction, not a claimed occurrence in the 600 simulated games.

**Suggested fix:** preserve a hit earned before the errant throw, subject to the preceding-runner rules. Use the reference margin to decide whether the throw award supplied the batter's initial safe arrival only when that question remains open. Cover both a desperation throw before first and a return throw after first.

### P1-3 — Change 2: foul territory bypasses the sacrifice-fly fielder requirement

**Location:** `sim/crates/bb_stats/src/scorer.rs:62,175` (retained in the sacrifice-fly rewrite in `2a21ef5`).

OBR 9.08(d)'s outfielder/infielder-in-the-outfield qualification applies in foul territory too. `caught_foul || by_outfielder || caught_in_outfield` instead makes every caught foul eligible.

**Probe:** a foul caught by the catcher 10 m behind home, fewer than two outs, followed by a runner tagging and scoring, returns `sacrifice_fly: true, rbi: 1`. It should be an ordinary out with the RBI and an at-bat, without a sacrifice fly. This directly affects AB and SF in the box score.

**Suggested fix:** require the eligible fielder/location in either territory; remove the unconditional `caught_foul` alternative. Add an infield foul-catch case alongside the shallow-outfielder test.

### P2-1 — Change 2: the walk-off cap is confined to one hit-classification branch

**Location:** `sim/crates/bb_stats/src/scorer.rs:81–83,95–97,148–152`.

The 9.06(f) cap executes only inside the `Unknown` → `judge_hit` → `Hit` branch. Award-derived hits and already classified `Single`/`Double`/`Triple` records return before it. OBR 9.06(f)'s comment explicitly includes automatic extra-base awards.

**Probe:** an awarded double with a winning runner starting on third and a walk-off event returns `Hit(2)`, rather than `Hit(1)`. **Reachability limitation:** today's rules engine stops awards at the winning runner (`bb_rules/src/engine.rs:763`), before awarding the trailing batter; that producer does not emit this particular counterexample. This is a latent log-reader/alternate-producer edge, not evidence that today's generated ground-rule walk-offs are all overvalued. The ordinary walk-off and over-the-fence HR tests pass.

**Suggested fix:** apply the non-HR walk-off limit once after determining the provisional hit value, covering all relevant input representations. Preserve the over-the-fence HR exception. Also document that ending simulation immediately at the winning run does not model every subsequent natural-course base touch discussed in 9.06(f).

### P2-2 — Change 2: the new appeal exception omits the force requirement

**Location:** `sim/crates/bb_stats/src/scorer.rs:115–117`; corresponding description in `docs/rules-spec.md:438`.

OBR 9.05(b)(2) concerns a runner forced to advance by the batter becoming a runner. The code tests only `Appeal` and `out.base == start + 1`.

**Probe:** R2 alone, an outfielder handling the hit, batter safely reaching first, and R2 appealed out for missing third produces `FieldersChoice`; that unforced appeal does not invoke 9.05(b)(2). Expected: `Hit(1)`. The existing R1 appeal test covers only the forced case. Missed-base appeals are currently deferred by the live rules engine, so this is a latent scorer-input defect, not a seeded-game reproduction.

**Suggested fix:** establish the applicable force and appeal reason from logged evidence before invoking this exception; add forced and unforced variants. Narrow the spec's unconditional appeal wording as well.

### P3-1 — Change 1: §5.5 still specifies two different effort/error laws

**Location:** `docs/agent-models.md:472,477`; `sim/crates/bb_agents/src/throwing.rs:41–54`.

The earlier bullet specifies directional sigma proportional to squared relative speed; the Built paragraph and implementation use the linear interpolation between 46.9/61.1 and 1. At 80% effort these give 0.64 and 0.767594 respectively. The implementation matches the Built paragraph, with named constants and a stated source, but the governing section remains contradictory. The sources note at `docs/calibration/2026-11-14-sources.md:72–74` also correctly says that total target error is not directional sigma.

**Suggested fix:** replace the obsolete squared-law bullet with the implemented law, and explicitly tag transfer of the experimental target-error ratio to directional sigma, and interpolation between the two speeds, as estimates [E]. Keep the distinction already stated in the sources note.

## Recheck of the Gemini scorer Resolution

Checked against the [2026 Official Baseball Rules](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf), especially 9.04, 9.05, 9.06 and 9.08(d), and the actual current code and executed cases.

| Original finding | Recheck |
| --- | --- |
| P1-1: RBI zeroed for any two outs | Fixed for the represented cases. Sacrifice fly plus a later tag, hit plus two baserunning outs, and an unforced second out retain RBI; force and reverse-force double plays withhold it. Rule-text tests pass. |
| P1-2: throw out of play strips RBI | The event-order bug is fixed: home-touch evidence distinguishes runs already across; the credited-run filter also controls SF. The three intended tests pass. Full error/ordinary-effort adjudication remains the stated A17 seam. |
| P1-3: wild throw misrules infield hits | Initial-arrival cases are fixed and pass; the broader replacement introduces P1-2 above for an already safe batter. |
| P1-4: walk-offs | Before-first singles, the ordinary hit-value cap and full over-the-fence HR credit pass. The cap does not cover all accepted record branches: P2-1 above. Natural-course continuation after the winning run remains unmodeled. |
| P1-5: preceding-runner putouts | Oversliding/return-direction gap remains explicitly deferred; runners currently stop on the base. The forced appeal case passes, but the implementation and Resolution overgeneralize the clause to unforced runners: P2-2 above. |
| P1-6: whom the defense played on | Appended runner IDs are populated from the chosen option; batter-directed throws/carries and late throws no longer incorrectly deny the initial hit. Reusing the before-first gate for extra-base value introduces P1-1 above. |
| P2-1: reference throw setup | Producer includes get-up, grade-50 transfer, turn and footwork from the catch; first baseman's carry alternative is included. Round-trip evidence test passes. Covering-pitcher/second-baseman geometry remains the disclosed approximation. Counterfactual is calculated after live simulation, without agent feedback. |
| P2-2: audit's inside-the-park HR mismatch | Fixed. Executed audit includes 14 inside-the-park HR in its 20-game sample and excludes them from BIP: 412 non-HR hits equals H 451 minus HR 39; BIP 1102 equals AB 1493 minus K 360 minus HR 39 plus SF 8. |
| P2-3: legacy evidence and shallow SF | Absent/early evidence remains explicitly undetermined; shallow OF versus IF tests pass. Foul territory still bypasses eligibility: P1-3 above. |
| P2-4: accounting tests | Improved independent checks and all 54 pinned hand-scored batter lines pass. All requested 600-game identities pass. The proposed engine-LOB-plus-stranded-batter oracle is not sufficient: the engine's strict LOB excludes runners between bases. Corrected seed-0 diagnostic gives strict LOB [6,6], extra short-of-first batters [0,0], versus box LOB [7,8]. This is a definition difference, not a demonstrated box-score identity failure. A full separate LOB reconstruction remains outstanding. |
| P3-1: captions | SF, ground-rule double and multiple outs on a hit are now represented; caption tests pass. Fielder attribution still uses live facts, as explicitly deferred. Captions are not official statistic counters. |

## Other review conclusions

**Change 1:** effort selection uses the holder's modeled race, reference-atmosphere `ThrowTable`, known setup and current publicly visible runner motion. It does not consult a realized throw trajectory or an `ExecutedThrow` to choose effort. The live orchestrator represents that public motion directly in its runner structures; it does not introduce a new noisy runner-observation layer. Current main forces full effort for moving-runner returns, relays and any available force continuation, including low-probability continuations. Other throws fall back to full effort if the modeled 0.25 s margin cannot be retained. The grid includes 1.0. Noise remains in `motor::execute_throw` with the unchanged keyed speed-then-direction draws; the new multiplier itself consumes no RNG.

The measured-endpoint/midpoint tests have external numerical expectations. The live easy/close policy test is less independent: its close-race ETA comes from `eta_for_margin`, sharing throw/setup models with the implementation. It proves the branch behavior, not independent physical timing accuracy. The separate motor probe verifies stream consumption and numeric endpoints; an independent marginal-policy sweep would improve coverage.

**Invariant 6:** official rulings and box scores consume serialized records only. The live presentation ruling is obtained from `writer.latest_play()` after serialization. Neither scorer nor box-score aggregation uses RNG or running simulation stat counters. Snapshot footer scores can reflect the later current game (`game.rs:525–529`), but the box-score reader derives score/finality from retained events, not that footer; the C prefix comparison confirms no future-result leak in `bb_game_box_score`.

**Change 3:** no additional actionable ABI/client defect found. Rust `repr(C)` and the header agree: `BbBatLine` 88 bytes/alignment 2, totals 32/2, team 1152/2, box 2308/2, play info 212/4. Verified representative field offsets too. Output is caller-owned and contains fixed arrays, not borrowed Rust allocations. Counts and innings are bounded; oversized pitch limits clamp to available plays. Zero pitches preserves lineups with zero statistics and no final flag. Client `GameLock` serializes `Next()` and `BoxScore()`; immutable timeline sampling is separate. `EndPlay()` joins the pending worker and releases current, queued and pending blobs before destroying the library.

**Change 4:** projected distance is copied from the world's unobstructed true-contact flight (`bb_world/src/pitch.rs:104–118`) into display telemetry and the C ABI; no agent consumer was found. The trot appends dead-ball presentation motion along the awarded base paths at 4.8 m/s; the rules result is already resolved. The executed trot test independently checks scored runner IDs, ordered base geometry, final home position, pace and duration. Separate historical builds confirm unchanged game digests for four seeds. Client side-cut code advances both staging clocks together, and speed keys select the stated eight playback rates. Camera, HUD and actor changes occur in game-thread playback; the worker lambda calls only `FBbSim::Next`. No Unreal gameplay-physics stepping was found.

## Verification

All heavy jobs were sequential. Commands ran from `sim/` with `CARGO_BUILD_JOBS=2`, the requested PATH, and Zig caches at `$PWD/target/zig-{global,local}-cache`; cargo commands and probe runs used `taskset -c 0-7 nice`.

- `cargo test --release -p bb_stats -p bb_ffi -p bb_agents -p bb_game --no-default-features`: **267 passed, 3 ignored, 0 failed**. Includes scorer rule cases, hand-scored fixtures, evidence round trip, FFI prefix/final cases, effort policy and home-run trot. Output: `target/unreviewed-tests.log`.
- `cargo test --release -p bb_rules`: scenario corpus passed. Output: `target/unreviewed-rules.log`.
- Standalone `rustc --edition=2024 -O -C linker=$HOME/.local/bin/zigcc`, linking current release rlibs through `-L dependency=target/release/deps`, built `target/unreviewed_{scorer_probe,motor,accounting,layout,lob}.rs`. The scorer builder reuses test serialization scaffolding, with independently specified adversarial inputs and rule expectations. `unreviewed_scorer_probe` reproduces the five scoring cases above; `unreviewed_motor` passes 300 keyed throws, checking the two normal draws, next RNG word and numeric effort multipliers. Outputs: `target/unreviewed-scorer.log`, `target/unreviewed-probes.log`.
- `target/unreviewed_accounting`: **300 games each**, seeds **0–299** and **1000–1299**. Respectively **72,397 / 72,575 pitches**, **23,259 / 23,215 PA**, **38 / 29 walk-offs**. **No accounting-identity assertion broke; zero undetermined rulings.** Checked PA decomposition, AB accounting, R + LOB + outs, hit decomposition, inning runs/hits, player/team sums, final-score agreement, pitch counts, outs, and RBI/SF/hit bounds. Its exploratory strict-engine-LOB comparison prints differences; the separate corrected `unreviewed_lob` diagnostic and source inspection establish the incompatible definition described above. Outputs: `target/unreviewed-accounting.log`, `target/unreviewed-lob.log`.
- `cargo build --release -p bb_ffi`, then `zigcc -O2 target/unreviewed_ffi.c target/release/libbb_ffi.a -lunwind -lm -lpthread -ldl -o target/unreviewed_ffi` and the executable: passed with assertions explicitly enabled (`#undef NDEBUG`; Zig defines it at `-O2`). Seed 7 ends **6–7 after 258 pitches**. Prefixes **0/1/37/120/200** remain byte-identical after completion; final/oversized limits, null arguments, output canaries and zero-pitch statistics pass. C and Rust sizes/alignments/offsets match. Output: `target/unreviewed-ffi.log`.
- `cargo run --release -p bb_stats --example scorer_audit -- 20 0`: passed; 14 inside-the-park HR excluded consistently, no unresolved rulings. Output: `target/unreviewed-audit.log`.
- Archived `4cf8da0^1` and `4cf8da0` under `target/a16-review/`; no checkout changes. Ran `cargo run --offline --release --manifest-path target/a16-review/{before,after}/sim/Cargo.toml -p bb_game --example unreviewed_digest`, with **separate build directories** for the final comparison. Seeds **0,1,7,42** have **identical full game digests and scores before/after**. Outputs: `target/unreviewed-a16-{before,after}.log`.
- **Unreal was neither built nor launched.** On-screen layout, camera transitions, frame pacing and engine-runtime thread/lifetime behavior could not be executed here; client conclusions are source inspection, not runtime validation. No sanitizer or alternate-architecture ABI run was performed.

## Resolution

Fixed 2026-11-14 on `work/scorer-recheck-fixes` (scorer in `bb_stats/src/scorer.rs`, tests in `tests/scorer_rules.rs`, each written first and failing before the fix; expectations from the OBR clause, not the code). Where the review's probe needed a logged fact the harness lacked, the test builder gained it (`batter_goal`, a caught-foul ruling).

- **P1-1 fixed.** The before-first gate stays only on the hit-versus-fielder's-choice decision. The extra-base cap now takes the lowest logged `batter_goal` over every attempted throw at a preceding runner, before or after his first touch, so a batter safe at first who takes second on a throw at R1 has a single (9.06(b)). A goal of second already chosen at release stands (a double in progress is still a double). Tests: single case and double-in-progress case.
- **P1-2 fixed.** Only a last throw to first that left at or before his touch of first can have supplied his arrival (and take the margin decision); a return throw after his touch falls through to the hit path, where the award is already excluded from the value (`before_throw`, non-award advances). Tests: return throw after the touch keeps `Hit(1)`; desperation throw before the touch still `ReachedOnThrowAward`.
- **P1-3 fixed.** The `caught_foul` alternative is removed; a foul or fair catch is a sacrifice fly only by an outfielder (or an infielder who caught it in the outfield). Test: catcher's foul and infielder's foul are an out with an RBI and no SF; an outfielder's foul is an SF.
- **P2-1 fixed.** The 9.06(f) cap is applied once, after the provisional value, to every `Hit` however it was derived (ruled 1B/2B/3B, award double, evidence); `PaResult::HomeRun` keeps the over-the-fence exception. Test: award double, ruled double, ruled triple (cap 1 from third), triple from second (cap 2). The spec now notes the natural-course touches after the winning run are unmodelled (not deferred work in the scorer: it is a rules-engine property).
- **P2-2 fixed.** The appeal exception now requires the runner to be forced by the batter becoming a runner (all bases behind his start occupied at the pitch, from the logged runner list). Test: unforced R2 appealed at third leaves the hit; forced R2 behind R1 denies it. Rules-spec wording narrowed. Appeal reason is read from `OutKind::Appeal`; missed-base appeals stay deferred in the live engine.
- **P3-1 fixed (docs only).** agent-models §5.5's squared-law bullet is replaced by the implemented linear law, with the two transfers tagged [E].

Deferred, as the review lists them and unchanged: the oversliding/return-direction case of 9.05(b)(3) (runners stop on the base; needs direction at the tag); full error/ordinary-effort adjudication (A17); fielder attribution in captions from live facts; a full separate LOB reconstruction (the engine's strict LOB excludes runners between bases, a definition difference); covering-pitcher/second-baseman geometry in the reference throw; a marginal-policy sweep for the effort test; and the unrun Unreal, sanitizer and alternate-architecture checks. If I read the review wrong anywhere: I found no point where it misreads the rule text.

Verification: `cargo test -p bb_stats` all pass (30 in `scorer_rules`, including the 8 new). `scorer_audit` over 300 games on seeds 0–299 and 1000–1299: zero unresolved rulings, identities pass, and its output (AB, H, HR, SF, BABIP, ruling counts) is identical before and after the change: all five findings are latent in generated games, as the review said for P2-1 and P2-2 and the probes show for the rest.
