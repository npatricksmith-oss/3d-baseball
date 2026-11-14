# Codex review: dive rate (owner backlog A5)

Reviewed `328e51b..56dd244` on `claude/a15-de1824`, with HEAD at `56dd244`, after reading `CLAUDE.md`, the relevant PLAN invariants, agent-models, and calibration requirements. Locations below refer to `56dd244`. No source, existing test, spec, or TASKS file was edited. This is the only authored repository file; build products, an archived parent checkout, and diagnostic binaries/logs are under `sim/target/`. Diagnostic source variants were passed to `rustc` on stdin.

## Findings

### 1. P1 — The outfielder's decision bypasses his noisy velocity read

**Location:** `sim/crates/bb_game/src/live.rs:1932–1937`.

The threshold is an agent decision, but its input is the world's execution sigma: `sg` uses `bv.norm()` from `self.ball_at(self.t)`, rather than velocity from `fd.belief`. Consequently an outfielder knows the exact speed that will affect his execution before deciding whether to attempt the catch. Calling the resulting number his belief does not establish the required observation boundary. His own hands are legitimate self-knowledge; the ball's exact velocity is supposed to go through perception. Current proximity used to schedule a close-range observation is a different use of world state.

**Verification:** A production-code fixture held the belief fixed at a smooth predicted ground ball moving at 20 m/s, with the fielder at `(0,80,0)`, ball at `(0,78.4,0.04)`, average hands `.018`, and an already-earned dive extension at time 2.0. Close/dive re-reads were frozen to isolate the decision. `dive_gains` returned true in both cases. Changing only world velocity from `(5,0,0)` to `(40,0,0)` changed the result from `dives=1, attempts=1` to `dives=0, attempts=0`. Both balls were already moving away, so the closest-approach wait did not cause the difference.

**Required correction:** Calculate the willingness to dive from his perceived state and self-knowledge, then independently calculate physical execution difficulty from truth. Keep the physical catch test unchanged. This is an invariant-2 information leak, **not an outcome roll**: the eventual catch still depends on glove-placement geometry, with no Bernoulli draw.

### 2. P2 — The closest-approach wait does not preserve next-tick eligibility

**Location:** `sim/crates/bb_game/src/live.rs:1916–1917`, in combination with `1936–1937`.

The wait checks only `how(...)` for the next tick. It can discard a presently acceptable dive even when the next tick will fail the outfielder's probability gate. A descending ball can get horizontally closer while gaining enough speed to make its execution sigma worse. There is no saved commitment or fallback attempt when that happens.

**Verification:** At time 2.0, put a stationary CF at `(0,80,0)` and a free ball at `(-0.02,78.4,1.0)` with velocity `(1,0,-10)`. Give him full earned extension and freeze re-reads/target movement. Hands `0.017846870982` (within 1% of the average `.018`) put the two ticks on opposite sides of the documented threshold. The production calculations yielded:

```text
tick  time       P(pocket)    height     dives  attempts
0     2.000000   .500100028   1.000000   0      0
1     2.016667   .499899996    .832046   0      0
2     2.033333   .499653347    .661520   0      0
3     2.050000   .499360304    .488426   0      0
4     2.066667   .499021116    .312770   0      0
```

At tick 0 the ball is within the dive envelope and above the willingness threshold, but he waits. At tick 1 it is closer and still physically reachable, but he refuses. Thus the claimed guarantee that it is still his to attempt next tick is false. This is a boundary counterexample, not a measured frequency in normal games. Check the complete next-tick attempt conditions, or commit to the attempt before waiting. The same wait also omits the next stretch's gain check and possible intervening dive-read reset; those additional paths need coverage rather than an assumption that `how` is sufficient.

### 3. P2 — The stretch's large fielding-efficiency change remains unvalidated

**Location:** `docs/calibration/2026-11-13-dive-rate.md:57`, alongside `sim/crates/bb_game/src/live.rs:31–37` and `1975`.

The ablation supports attributing the run reduction to the stretch as a whole. It does not validate the new reach, execution-error multiplier, or recovery cost, nor isolate the proposed explanation that the shorter recovery produces the extra outs. Setting `STRETCH=0` changes which balls are attempted, the chosen dive/stretch mode, and downstream possessions as well as recovery times. The successful synthetic stretch test checks the promised timer, not whether this model produces plausible fielding opportunities and throws.

**Verification:** Both parent/current seed batches reproduce a reduction of `.71` or `.77` runs/team-game, roughly 13%. Independently compiling the current engine with only `STRETCH=0` reproduces `5.63167` runs/team-game, ground-ball hit `.39687`, and fly-ball hit `.16945` for seeds 0–299. The current ground-ball hit rates `.332/.345` remain above the B2.2 envelope `.200–.280`; improved runs do not establish that the lower-level mechanism is correct. The note identifies the parameters as estimates but supplies no independent stretch-opportunity, success, or recovery observations and no sensitivity result.

**Required follow-up:** Treat the stretch's efficiency as provisional, with explicit mechanism validation and sensitivity/ablation separating reach and recovery effects. A physically justified stretch is a legitimate model correction; this finding does not allege intentional tuning toward runs or require restoring the old run total. The issue is presenting A5 as fixed while a mechanism large enough to move the run environment this much has only count/timer checks. Calibration-targets §2.1–§2.3 requires validating the mechanism before accepting favorable game totals as evidence.

### 4. P3 — The real-world success-rate citation changes its population and metric

**Location:** `docs/calibration/2026-11-13-dive-rate.md:10–11`, and comparison with `17`/`43`.

The cited [SIS All-Scrapes article](https://www.sportsinfosolutions.com/2025/04/16/stat-of-the-week-the-all-scrapes-team/) assigns the 24% out rate to **second basemen**, not all MLB divers. It separately reports different rates for first basemen and shortstops. It also lists a second baseman with 140 dives over three seasons; it does not support the accompanying assertion that the busiest second baseman had 19 in a season. That assertion needs its own season/source or correction.

Moreover, the probe's “caught” means immediate possession on the dive tick. SIS's infield success figures measure obtaining an **out**. Catching a ground ball and then losing the race to first is a probe success and a SIS failure. Those percentages cannot validate each other without matching the metric.

**Verification:** Read the cited article and the [same author's Correa article on ACTA Sports](https://actasports.com/statoftheweekcarlos-correas-amazing-slides-dives-and-jumps/). The latter supports 36 SS dives per 162 team games and the 22% SS out rate, so the note's `.44` dives per game for both shortstops is fairly quoted. The RF counts and 54% catch rate are also fairly quoted. The league-total “2 to 3” is explicitly the note author's estimate, not a misattributed published total; it should remain an estimate rather than a precise acceptance band. In particular, “about ten times” is approximate and not a separately established league measurement.

### 5. P3 — One reported ablation count does not reproduce

**Location:** `docs/calibration/2026-11-13-dive-rate.md:57`.

With only `STRETCH=0`, I count **2,229 dives in 300 games, 7.43/game**, which rounds to **7.4**, rather than the reported 7.3. The ablation's run/GB/FB outputs reproduce, so this does not overturn its main conclusion. Correct the count or identify the different run/configuration that produced it.

**Verification:** An atomic diagnostic counter at the production dive-emission site, in the stdin-compiled ablation, reported `dives/game 7.43000`; the other ablation outputs are listed below. The counter did not enter any gameplay decision.

## Other review conclusions

### Invariants and determinism

- **D1:** No new outcome roll. The analytic Gaussian-pocket probability chooses whether an OF attempts a dive. After an attempt, the same two truncated-normal glove-placement errors determine `d`; `d <= POCKET` secures the ball and the rim interval deflects it. Finding 1 concerns information used for the choice, not replacing physical catch resolution with probability.
- **`dive_gains`:** Its intercept scan uses the fielder's own predicted path, attributes, position and velocity. The branch on `ball_still_up()` asks whether a first bounce has happened by now, not when a future true bounce will happen. That is an observable fact. Comparing the predicted intercept to the predicted first bounce likewise uses belief. I found no future-world leak in this helper.
- **`must_dive_since`:** The near-ball read calls that helper; it does not scan the world's future flight. The actual current distance schedules the observation, as the existing close-range re-read already did. The timer represents a sampled belief held between reads, not proof that a real physical dive has already begun.
- **Closest approach:** The dive now shares the standing catch's true one-tick lookahead, including true bounce/wall/end boundaries and the next movement step. This extends an existing world-side capture convention to dives, without a longer or richer future forecast. It is not a clean agent-only observation interface, but the new correctness defect is its incomplete eligibility guarantee (finding 2).
- **RNG/math/order:** No new RNG purpose or draw, no within-stream draw reordering, and no unordered collection iteration in simulation were introduced. The glove stream still uses `(seed, play, fielder, GlovePlacement, attempts)` and two normal draws in the same order. Changed attempt schedules intentionally change which keyed attempts occur and subsequent play histories; identical replays of the new build remain deterministic. New exponential math uses `bb_math::m::exp`. The probe's `sqrt` is explicitly allowed by CLAUDE.md; integer `powi(2)` in a reporting-only calculation is not a new simulation transcendental call or replay risk.

### Capture mechanics

- `ext_at` gives zero while `must_dive_since` is infinite, rises at 5 m/s after the later of the qualifying read and reaction/recovery gate, and caps at 1.5 m. For the pitcher, 0.7 s after contact is the **start** of available extension, so full extension requires another 0.3 s. The arithmetic matches the stated rule; the tests establish a playable/late distinction, not an independent measurement of human dive timing.
- `how` prefers a stretch within 1.4 m at ordinary catchable height; otherwise it allows a low dive within the earned envelope. A failed stretch gain check need not fall back to a dive: the longer dive cost cannot improve the ground-ball time comparison, and the airborne comparison is the same for either cost.
- `set_free` resets both dive timers and calls `assign`, which refreshes all nine beliefs through `assign_among`; it does not leave a chaser predicting the old ball. The existing close-read timestamp is not cleared, but the assignment already supplies a fresh belief.
- A down fielder cannot stretch or dive again. Standing capture remains constrained to arm's reach and dive height until he is up, including its next-tick height check. A recovered bobble retains the original get-up deadline plus transfer.
- No new same-fielder double attempt occurs within one tick: each fielder appears once in the loop; a secure/rim returns; a clean miss sets a block. Later attempts after the block are intentional recovery attempts. Finding 2 establishes a skipped opportunity instead.
- Thrown balls retain `RECEIVE_REACH=1.5 m` and their existing receive/relay transfer path. The old code could additionally dive for a throw only when the receiver happened to retain `Role::Chase`; ordinary cover/relay receivers could not. That inconsistent exceptional behavior is removed by the free-ball restriction. There was no old general stretch-on-throws mechanism to preserve. A thrown catch cannot enter the new `extending` branch, so the stretch cost is neither spuriously applied to, nor accidentally overwritten on, a caught throw/relay.
- A stretch gets the 0.3 s balance cost before transfer when secured. Stretch misses use the existing rim/miss blocking and running behavior, with no separate balance-recovery state. That is untested; the documented cost is specifically before a throw, so I did not treat continued running after a miss as an independently established defect.
- `low_catch` remains true below 0.4 m; a stretch above that is an upright catch, unlike a dive. I found no reason to mark every high stretch as a shoestring catch. The geometry/threshold boundary itself lacks a test.

### Tests and believability

The five added behavior tests use independently stated scenarios: a near-reach ball should be taken upright, insufficient lunge time should lose the play, OF/SS risk differs, a pitcher has a delivery-recovery gate, and a pursuer runs down a slow roller. Their expected actions are not computed by calling `dive_gains` or duplicating the implementation formula. Explicit transfer-plus-0.3 timing is a specification expectation, not a tautological formula test. All five fail against the parent implementation. The sixth changed test is the pre-existing wall regression with its timer fixture adjusted; it passes on the parent as expected and still requires an actual dive, so it is not vacuous.

Missing coverage includes airborne `dive_gains` both with and without a standing catch before the predicted bounce, the no-predicted-bounce case, both sides/equality of the OF .5 threshold, finding 2's wait boundary, changed beliefs during the wait, successful/missed stretches followed by another attempt, a fresh free-ball transition's timers, and stretch height around `LOW_CATCH_Z`.

The aggregate data refute “an infielder never dives” and “an outfielder never dives”: the 300-game batch has 1.19 SS dives/game and .31 OF dives/game. The short-lead unit scenario also demonstrates that a sufficiently early read permits the play. This does not prove hot-shot behavior across speeds and reaction gates; a sweep near the 15 m read range would be useful. The fixed OF .5 cutoff also ignores inning, outs, score and backup position: for example, it can refuse the only chance to catch a would-be walk-off ball when playing it on the bounce necessarily loses. That is a limitation of the newly specified heuristic, beyond the implementation failures above. It should not be described as complete situational risk judgment.

### Diagnostic interpretation

The probe counts dive events correctly, and its same-fielder/same-time test correctly labels an **immediate** secure. It is not an out rate, nor eventual recovery: in seeds 1000–1299, 13 otherwise unsuccessful dives were followed by the same diver securing the ball within 1.5 s. Neither eventual possession nor a ground-ball pickup implies an out.

Air/ground is approximate. `step.batted` is the original uninterrupted path, whereas `set_free` can replace that path after a rim. `first_touch` is first **secure**, not first glove contact. A dive after an unsecured deflection can therefore be classified using an obsolete bounce time; an airborne deflection is not reliably excluded by the “before ground or glove” comment. The instrumented 300-game batch found **zero** air-labelled dives after an earlier `set_free`, so I did not find evidence that this limitation changes the reported air totals in that batch. A separate comparison against emitted ground/wall/secure facts had 13 disagreements, but those facts omit unsecured rim contacts and a wall rebound may still be airborne; those 13 are **not** 13 proven misclassifications. Do not use them as a correction to the note's totals.

The speed/gap histograms extrapolate from a downsampled track, so values near bounces, glove contacts and the 1.2 m/2.5 m limits are estimates. They are appropriate diagnostics, not authoritative physical boundary checks. Direct `powi`/`sqrt` here do not affect the game.

## Verification

All commands ran from this worktree. Cargo commands used `export PATH=$HOME/.cargo/bin:$PATH; cd sim`. Initial new links failed because Zig tried to write its global cache outside the writable sandbox. Retried successfully with `ZIG_GLOBAL_CACHE_DIR="$PWD/target/zig-review-cache"`; no administrator privileges or approval were required.

1. `cargo test --workspace --release`: **exit 0; 164 passed, 0 failed**, including 64 bb_game unit tests, integration tests and doc-test targets. Output: `sim/target/dive-review-tests.log`.
2. `cargo run --release -p bb_game --example dive_probe -- 300 1000`:

   ```text
   20559 balls in play, 929 dives (3.1/game)
   926 plays with a dive (4.5% of BIP), 42% immediately caught
   P .09, C .10, 1B .39, 2B .42, 3B .59, SS 1.19, OF .31 dives/game
   ```

   Output: `sim/target/dive-review-probe-1000.log`. These claims reproduce. Also ran `cargo run --release -p bb_game --example dive_probe -- 100 0`: 6,978 BIP, 345 dives (3.5/game), 344 plays with a dive (4.9%), 44% immediately caught. Output: `sim/target/dive-review-probe-0.log`; the note's second dive batch reproduces too.
3. `cargo run --release -p bb_cli -- games --n 300 --seed 1000`, and again with `--seed 0`:

   | Seeds | Runs/team-game | GB hit | LD hit | FB hit | Reached on BIP | Rules warnings |
   |---|---:|---:|---:|---:|---:|---|
   | 1000–1299 | 4.94 | .345 | .729 | .141 | 34.1% | `{}` |
   | 0–299 | 4.77 | .332 | .720 | .160 | 33.7% | `{}` |

   Outputs: `sim/target/dive-review-games-1000.log` and `dive-review-games-0.log`.
4. Archived `328e51b` with `git archive 328e51b sim` into `sim/target/dive-review-before`. Ran `cargo run --manifest-path target/dive-review-before/sim/Cargo.toml --release -p bb_cli --no-default-features -- games --n 300 --seed 1000`, and again with seed 0, using the separate `CARGO_TARGET_DIR="$PWD/target/dive-review-before-build"`. Only the optional Rerun visualizer feature was disabled; the game simulation was unchanged.

   | Parent seeds | Runs/team-game | GB hit | LD hit | FB hit | Reached on BIP | Rules warnings |
   |---|---:|---:|---:|---:|---:|---|
   | 1000–1299 | 5.71 | .433 | .716 | .143 | 39.9% | `{}` |
   | 0–299 | 5.48 | .425 | .725 | .131 | 39.4% | `{}` |

   Outputs: `sim/target/dive-review-before-1000.log` and `dive-review-before-0.log`. All before/after table entries reproduce.
5. Compiled the unchanged new `dive_probe.rs` on stdin against the parent build's `bb_game` and `bb_math` libraries; ran `target/dive-review-before-probe 100 0`:

   ```text
   7268 BIP, 2208 dives (22.1/game), 2123 plays with a dive (29.2%), 61% caught
   P 3.97, SS 4.38, OF 8.22 dives/game
   dives below 10 m/s: 4.31/game; gap bin 2.25–2.5 m: 14.35/game
   ```

   Output: `sim/target/dive-review-before-probe.log`. The main pre-change dive claims reproduce.
6. Stdin-compiled current `live.rs` diagnostic fixtures, with existing `play`/`umpire` dependencies, using `rustc --edition=2024 --test -O -C linker=/home/natex/.local/bin/zigcc -L dependency=target/release/deps` and explicit `--extern` library paths. `target/dive-review-fixed-belief review_fixed --nocapture` demonstrated finding 1; `target/dive-review-wait review_ --nocapture` demonstrated finding 2. Their fixture inputs and outputs are given with the findings. A separate `target/dive-review-boundary review_ --nocapture` smoke probe exercised airborne gain cases with vertical speeds -1, +1, +4 m/s; all returned true where the standing scan found no play. This is not full airborne-branch coverage.
7. Transplanted only the five added behavior tests/helpers into an in-memory copy of the parent `live.rs`; compiled on stdin as `target/dive-review-old-tests`. Each added behavior test failed on the old code, for its intended action difference. The parent's wall regression passed. No tracked test was edited.
8. Stdin-compiled current `live.rs` and `game.rs`, reusing unchanged dependencies, with only `STRETCH` set to zero and a diagnostic atomic counter at dive emission. Ran 300 seeds 0–299 in 16 worker threads (`target/dive-review-no-stretch`):

   ```text
   runs/team 5.63167, GB .39687, LD .73694, FB .16945, dives/game 7.43000
   ```

   This verifies the main stretch ablation and finding 5. Parallel work was ordinary simulation threads, not delegated agents.
9. `target/dive-review-classification` compared the original probe labels with emitted contact facts and counted delayed possession over 300 seeds 1000–1299: 929 dives, 13 contact-fact discrepancies, 13 non-immediate same-diver recoveries within 1.5 s. `target/dive-review-facts` inspected seeds 1070/play 160 and 1097/play 34; these demonstrate why contact facts alone cannot correct the air classification. `target/dive-review-air-deflections`, with diagnostic logging of `set_free` transitions, found zero air-labelled dives after an earlier transition in that batch. These probes support the qualified diagnostic assessment above, not a claim that 13 classifications are wrong.
10. Read both real-world articles linked above. The original SIS `?p=1965` URL did not fetch; the same author/title on ACTA supplies the SS figures. Final `git diff --check` was clean, and `git status --short` showed only this new review file.

## Resolution

Resolved 2026-11-13.

1. **P1, fixed.** The outfielder's chance is now computed from his belief: where his own prediction has the ball and how fast (`fd.belief.at(t)`), with his own hands, through the same σ form (`glove_sigma_mult`). The world's ball state no longer enters the choice. Test `an_outfielder_weighs_a_dive_by_his_own_read_of_the_ball`: with the belief fixed he decides the same whether the ball is really doing 5, 20 or 40 m/s, and the decision flips with the believed speed.
2. **P2, fixed for the dive; the stretch deferred.** The outfielder decides once, when a dive first offers, and before the wait for the closest tick; having decided to go he goes (`Fielder::will_dive`, cleared when the ball changes). Test `an_outfielder_who_has_decided_to_dive_goes` uses the review's descending ball, with hands set so the first tick is over an even chance and the next under. Deferred: a stretch's gain check and the dive read can still change his mind during the wait. Both answers come from his read saying he now has the ball on his feet, which is a change of mind a fielder can have, and the standing catch follows; no case of a lost ball was found. No test covers it.
3. **P2, accepted; partly done, the rest deferred.** The note now gives a sensitivity table (stretch 0, 0.2, 0.4, 0.6 m) and separates reach from recovery (0.4 m with a dive's 1.5 s cost): runs fall about 0.2 per 0.1 m, and about 0.4 of the 0.87-run drop is the quicker recovery. It says the stretch's size and cost are placeholders and the lower run total unconfirmed. Deferred: validating the stretch against charted plays (how often, how often held, how late the throw). Listed in TASKS.md.
4. **P3, fixed.** The 24% is attributed to second basemen, the unsupported "19 in a season" is gone (replaced by the article's 140 dives in three seasons), the note says SIS counts outs while the probe counts the ball in the glove, and "2 to 3 a game" is marked an estimate, not a band.
5. **P3, fixed.** The 7.3 was from 100 games. The table now gives 7.5 a game over seeds 0–299 on the fixed build (the review counted 7.43 on the build it read).

Also from the review's notes: the outfielder rule's blindness to the game situation is stated in the note. The probe's air/ground label after a deflection, and the missing tests it lists (airborne `dive_gains`, stretch misses, the low-catch height for a stretch), are deferred as low risk.
