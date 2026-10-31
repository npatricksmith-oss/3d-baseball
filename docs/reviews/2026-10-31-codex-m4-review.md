# Codex M4 review — 2026-10-31

Reviewed PLAN §4.1–4.2, rules-spec §§1–7 and 12, agent-models §§5–6, all `bb_rules/src/*.rs`, the scenario harness and corpus, and `bb_game/src/play.rs` / `game.rs`. Line references below use checkout commit `a1ac979`. The checkout changed during this review; the wall-contact fix in `d2f4cd7` was re-read and is recorded under Resolution. An unrelated edit to `examples/bip_sweep.rs` was present during the final inspection and was left alone.

Severity: **P1** changes outs/runs or violates the authoritative-physics boundary; **P2** is another correctness, accounting, or reproducibility-contract defect. Findings are ordered by severity. These are static findings, with predicted results traced through the code; the proposed scenarios were **not** installed or executed. TOML with `id`, `initial`, `step`, and `expect` uses the current corpus syntax, including its supported inline-table form. Blocks marked **descriptive fixture** document simulator inputs or missing fact vocabulary and are not runnable by the current rules harness.

Rule numbers were checked against the [2026 Official Baseball Rules](https://mktg.mlbstatic.com/mlb/official-information/2026-official-baseball-rules.pdf). Relevant provisions are 5.08(a), 5.09(b)(5)–(6), 5.09(c)(1), 5.09(d), and the definitions of Catch and Fair Ball. The current game-ending provision is **7.01(e)(3)**; the spec/corpus citation to 7.01(g) is stale. The project's classic extra innings and other explicit design choices remain the review baseline.

The permitted checks passed after redirecting Zig's otherwise unwritable caches to `/tmp`: `cargo test -q -p bb_rules` and `cargo run -q --release -p bb_cli -- games --n 32`. The batch reported 9.47 runs/team-game, 43.0 PA/team-game, 254 pitches/game, four extra-inning games, three walk-offs, and `rules warnings: {}`. This is a smoke check, not proof of physical consistency or the 1,000-game accounting gate. The final corpus has 40 TOMLs versus the spec's 150-case M4 target. The documented deferrals in `bb_rules/src/lib.rs:13`—including uncaught third strikes, foul tips, interference, and first-base overruns—are not individually reported as newly discovered implementation bugs.

### 1. [P1] A tag on a still-forced runner is incorrectly treated as a time play

**Location:** `sim/crates/bb_rules/src/engine.rs:119`, `:128`, `:524`.

Every runner-body tag becomes `OutKind::Tag`, without examining the active force. Consequently a third-out tag on R1 before second allows an earlier home touch to score. The existing force-removal loop correctly removes forces on runners ahead when a following runner is retired; it does not repair the classification of this out. Rules-spec §§5.4 and 6.4 require the force status at the out to govern run cancellation.

**Scenario:** R3 crosses home before R1 is tagged while still forced to second. Expected zero runs; current code counts one.

```toml
id = "review-force-tag-third"
initial = { inning = 7, outs = 2, batter = "A.4", bases = { first = "A.1", third = "A.3" } }
step = [
  { fact = "PitchDelivered", t = 0.0 },
  { fact = "BatContact", t = 0.42 },
  { fact = "BaseLeft", t = 0.5, agent = "A.1", base = 1 },
  { fact = "BaseLeft", t = 0.5, agent = "A.3", base = 3 },
  { fact = "BallTouched", t = 0.8, what = "Ground", pos = [-6.0, 25.0] },
  { fact = "GloveSecured", t = 1.3, fielder = "H.SS", pos = [-8.0, 30.0] },
  { fact = "BaseTouched", t = 4.0, agent = "A.3", base = 4 },
  { fact = "TagContact", t = 4.1, fielder = "H.2B", runner = "A.1", holding_ball = true }
]
expect = { score = [0, 0], half = "bottom", outs = 0 }
```

**Suggested fix:** determine whether the tagged runner is still forced and has not satisfied that obligation before recording the out. Preserve both the physical method (tag) and the rules classification (force) if both are needed in the log.

### 2. [P1] A third-out fly catch can count a runner who crossed home before the catch

**Location:** `sim/crates/bb_rules/src/engine.rs:417`, `:523`–`:527`.

`third_out` explicitly exempts `FlyOut` from its batter-before-first cancellation. A runner can accumulate a pending home touch while the ball is airborne, then have that run committed when the batter's catch is the third out. The catch's late `retouch_required` update does not remove the pending run. Rules-spec §6.4 has no such fly-out exemption.

**Scenario:** two outs, R3 runs on contact and crosses before a long fly is caught. Expected no run; current code scores it.

```toml
id = "review-third-out-catch"
initial = { inning = 7, outs = 2, batter = "A.4", bases = { third = "A.3" } }
step = [
  { fact = "PitchDelivered", t = 0.0 },
  { fact = "BatContact", t = 0.42 },
  { fact = "BaseLeft", t = 0.5, agent = "A.3", base = 3 },
  { fact = "BaseTouched", t = 4.5, agent = "A.3", base = 4 },
  { fact = "GloveSecured", t = 5.0, fielder = "H.CF", pos = [0.0, 95.0] }
]
expect = { score = [0, 0], half = "bottom" }
```

**Suggested fix:** a batter's catch for the third out must cancel runs on that play. Do not infer eligibility solely from a batter's high-water base index; a batter could physically reach first before a very high pop is caught.

### 3. [P1] The pending-run ledger survives a successful appeal against the scoring runner

**Location:** `sim/crates/bb_rules/src/engine.rs:469`, `:478`, `:500`, `:527`, `:552`, `:567`.

`record_out` marks a runner out but leaves that runner's `runs_pending` entry intact. Both settlement and third-out processing can subsequently score the retired runner. This is reachable when the runner crosses home before the fly is secured and the defense then appeals at the original base. In the opposite ordering, a home touch after the catch is silently omitted by the early return at `:478`, even if no appeal occurs. Run eligibility is being decided by when the catch is processed, rather than by the home touch and appeal resolution.

**Scenario:** no outs initially; R3 runs home early, the fly is caught, and third base is appealed for the second out. Expected two outs and no run; current settlement counts R3.

```toml
id = "review-appealed-run-still-pending"
initial = { inning = 6, outs = 0, batter = "A.4", bases = { third = "A.3" } }
step = [
  { fact = "PitchDelivered", t = 0.0 },
  { fact = "BatContact", t = 0.42 },
  { fact = "BaseLeft", t = 0.5, agent = "A.3", base = 3 },
  { fact = "BaseTouched", t = 4.5, agent = "A.3", base = 4 },
  { fact = "GloveSecured", t = 5.0, fielder = "H.CF", pos = [0.0, 95.0] },
  { fact = "BaseTouched", t = 6.0, agent = "H.3B", base = 3, holding_ball = true },
  { fact = "PlayEndCheck", t = 8.0 }
]
expect = { outs = 2, score = [0, 0], bases = {}, play = "Dead" }
```

There is another ledger boundary to cover: if a preceding runner's retouch appeal is the third out, a following runner's earlier home touch cannot automatically be counted just because its timestamp precedes the appeal. That case is covered by OBR 5.09(d), while rules-spec §6.4 currently describes fly appeals too broadly as time plays.

**Suggested fix:** retain home touches independently of unresolved retouch obligations; resolve each run's eligibility when an appeal succeeds or the appeal opportunity closes. Exclude the appealed runner's run and apply preceding-runner cancellation before committing the ledger. Clarify the spec's third-out appeal wording as well.

### 4. [P1] A runner who touches home before a home-run boundary event disappears from scoring

**Location:** `sim/crates/bb_rules/src/engine.rs:373`, `:381`.

`home_run` clears the live play, then awards runs only to runners with `scored_at.is_none()`. A runner with an earlier home touch is only pending, not yet included in the linescore. Filtering that runner out loses the run. This can turn a two-run homer into a one-run homer and change the winner.

**Scenario:** R3 crosses home while a high drive is still airborne; the ball then leaves the park. Expected two runs; current code counts only the batter.

```toml
id = "review-homer-with-pending-run"
initial = { inning = 4, batter = "A.4", bases = { third = "A.3" } }
step = [
  { fact = "PitchDelivered", t = 0.0 },
  { fact = "BatContact", t = 0.42 },
  { fact = "BaseLeft", t = 0.5, agent = "A.3", base = 3 },
  { fact = "BaseTouched", t = 4.5, agent = "A.3", base = 4 },
  { fact = "BallTouched", t = 6.0, what = "OutOfPlay", pos = [10.0, 125.0], in_flight = true }
]
expect = { score = [2, 0], bases = {}, batter = "A.5", play = "Dead" }
```

**Suggested fix:** reconcile pending home touches and home-run awards by runner identity, scoring every eligible runner exactly once before destroying the play state.

### 5. [P1] Decisions depend on future world outcomes that the players cannot observe

**Location:** `sim/crates/bb_game/src/play.rs:142`, `:177`, `:257`, `:358`, `:374`, `:404`, `:435`.

Fielding estimates read the fully executed future `BallPath`, including future roughness and wall effects; the fallback target is the exact future rest point. Dive selection looks directly at the next 0.3 seconds of the actual trajectory. More seriously, runner plans are built after the entire fielding simulation and branch on the **actual eventual** `caught_fly`, exact `t_sec`, and exact securing position. Those plans are then backdated to contact. The defense in turn chooses from the runners' exact future plans. This bypasses the belief/observation boundary in D1 and agent-models §§5.1, 5.6, and 6.3. The file's documented truth-plus-error approximation does not justify knowing a future glove failure or surface-noise realization.

**Concrete scenario:** two physically identical observations through contact/reaction, differing only in a later glove-placement sample. In one execution a liner is caught at 3 s; in the other it gets through and is secured at 8 s. The batter has no running touches in the first case, while existing runners can leave at contact in the second. Thus a future execution sample changes an earlier action without an intervening observation.

**Descriptive fixture:**

```toml
id = "review-future-catch-leaks-backward"
same_observations_through_s = 0.7
change_only = "GlovePlacement at first capture attempt"
case_a_capture_s = 3.0
case_b_capture_s = 8.0
compare = "runner motion and fielder decisions before 0.7 s"
expected = "identical action prefixes"
```

**Suggested fix:** advance the world and decisions together. Predict from observed launch state and an internal smooth-surface model, update on observations/signals with latency, and choose throws from observed runner motion. Keep `p_out` as an agent valuation; it is not itself an outcome roll.

### 6. [P1] An unfielded over-fence path disables every possible interception

**Location:** `sim/crates/bb_game/src/play.rs:198`–`:203`.

If the precomputed free-ball trajectory eventually clears the fence, the function returns before simulating any fielder. A reachable liner on its way over a short wall, a robbed home run, and an interceptable bouncing drive are all automatically allowed to leave. This deterministically substitutes a free-flight endpoint for the outcome of the live play.

**Concrete scenario:** a short-wall field and a descending drive that passes within standing reach of the positioned outfielder before crossing the wall. Even zero perception/glove-placement error cannot produce a catch in this branch.

**Descriptive fixture:**

```toml
id = "review-over-fence-still-interceptable"
wall_height_m = 1.0
free_path_end = "OverFence"
reachable_before_boundary = true
fielder_read_sigma0 = 0.0
fielder_hands_sigma_m = 0.0
expected = "resolve the interception before the boundary event"
actual = "only free-path ball facts; no fielding attempt"
```

**Suggested fix:** make boundary crossings compete in time with captures. Emit out-of-play only if no earlier physical interaction changes the path.

### 7. [P1] Failure to secure the ball is converted into invented possession

**Location:** `sim/crates/bb_game/src/play.rs:348`–`:358`, `:369`, `:449`.

When all capture attempts fail, `None` is replaced by `(t_limit, ball_position)`. That fabricated pair is then used as a fielder-held ball for runner decisions, throws, and possible `holding_ball = true` putout facts. No `GloveSecured` exists. `caught_fly` is also computed without requiring `secured.is_some()`. A time limit is neither possession nor a catch.

**Concrete scenario:** every pocket attempt misses until the chase deadline, with a slow runner still advancing and a receiver already covering the target base. The downstream throw logic can produce possession-dependent facts despite zero successful captures.

**Descriptive fixture:**

```toml
id = "review-no-capture-no-throw"
capture_success_times_s = []
chase_deadline_s = 22.0
runner_target_touch_s = 30.0
receiver_cover_s = 5.0
expected = "no throw, catch, or holding-ball out without possession"
```

**Suggested fix:** keep possession optional throughout. Continue the live loose-ball play or surface an explicit simulation failure/timeout through the rules interface. Never populate held-ball calculations with a timeout position.

### 8. [P1] The game loop erases unresolved live plays outside the rules engine

**Location:** `sim/crates/bb_game/src/game.rs:168`–`:178`.

On failure to settle, the orchestrator directly assigns `st.play = Dead`. It discards pending runs, outs' supporting play state, and runner positions while retaining stale pre-play bases and the current PA/count. The next pitch can therefore replay the same batter after a fair contact. This violates rules-spec §1.1's single mutation boundary and §5.8's explicit timeout placement/event behavior. A diagnostic warning does not make the resulting baseball state valid.

**Concrete scenario:** a ground contact and capture share a timestamp. Ball facts are inserted first, so the rules see a ground ball; `caught_fly = first_touch >= t_sec` classifies it as a fly in the simulator, which produces no batter-runner base touches. Settlement fails, then the loop erases the play. This also shows an inconsistent equality policy between simulator and rules.

The following corpus fragment reproduces the unsettled rules state which the game loop then overwrites:

```toml
id = "review-ground-and-glove-same-time"
initial = { inning = 3, batter = "A.4", count = [1, 1] }
step = [
  { fact = "PitchDelivered", t = 0.0 },
  { fact = "BatContact", t = 0.42 },
  { fact = "BallTouched", t = 1.0, what = "Ground", pos = [5.0, 30.0] },
  { fact = "GloveSecured", t = 1.0, fielder = "H.2B", pos = [5.0, 30.0] },
  { fact = "PlayEndCheck", t = 5.0, expect_events = [{ event = "RulesWarning", code = "runner_between_bases" }] }
]
expect = { play = "Live", outs = 0, batter = "A.4" }
```

**Suggested fix:** remove direct rules-state repair. Correct the inconsistent capture classification, continue physical play, and route any genuine timeout through an explicit rules input with placement and a logged failure. An unsuccessful simulation must not masquerade as another ordinary pitch.

### 9. [P2] Throw reception can wait for a receiver, and a tag can predate possession

**Location:** `sim/crates/bb_game/src/play.rs:461`–`:476`, `:483`–`:488`.

`t_catch = max(t_arr, t_cover)` makes an early throw wait at the bag until a late covering fielder arrives. It does not simulate where the ball travels meanwhile or require a receiver capture. For non-force plays, any positive arrival margin then creates a tag at `t_runner - 0.03`, even if the fielder only gets there 0.01 s before the runner. The tag is then 0.02 s **before** the fielder can possess/reach it. The self-fielded branch has the same backdating. A double-play relay likewise emits a first-base holding-ball fact without checking the receiving first baseman's arrival or capture.

**Descriptive fixture:**

```toml
id = "review-tag-before-possession"
throw_arrival_s = 4.0
receiver_cover_s = 4.99
runner_base_touch_s = 5.0
force = false
lateral_miss_m = 0.0
actual_generated_tag_s = 4.97
expected = "no tag before receiver possession and actual runner contact"
```

**Suggested fix:** intersect a moving throw with a receiver's reachable volume at the same time; otherwise continue the loose ball. Generate tags from ball-holding hand/glove versus runner geometry, after possession. A time estimate may choose an attempt but cannot establish that contact occurred.

### 10. [P2] First fielder touch and secure time are conflated

**Location:** `sim/crates/bb_rules/src/state.rs:146`, `:172`; `sim/crates/bb_rules/src/engine.rs:422`–`:432`; `sim/crates/bb_game/src/play.rs:272`–`:278`, `:350`.

The engine sets `first_touch = t` only on `GloveSecured`; no glove-touch or release/drop fact exists. A runner legally leaving after first contact but before secure time will be marked early. Fair/foul is also judged at secure position rather than first fielder touch. The simulator emits secure immediately on entering the pocket and treats a rim deflection as a delay without changing the ball trajectory, so it cannot provide the missing contact/retention history. Field-boundary foot eligibility is absent from the fact interface too. These omissions contradict rules-spec §§2 and 5.3, beyond the declared simpler movement/throw model.

**Concrete scenario:** the fielder first touches at 4.0 s in fair territory, R3 leaves at 4.1, and the fielder secures at 4.3 after moving into foul territory. The runner's departure and the fair ruling must be based on the earlier contact.

**Descriptive fixture — first-touch vocabulary needs implementing:**

```toml
id = "review-first-touch-before-secure"
first_fielder_touch = { t = 4.0, fielder = "H.RF", pos = [20.0, 20.1] }
runner_leave = { t = 4.1, agent = "A.3", base = 3 }
secure = { t = 4.3, fielder = "H.RF", pos = [20.2, 20.0] }
expected_fair = true
expected_retouch_required = false
```

**Suggested fix:** report first contact, actual deflections, possession retention/loss, and the capture's field-eligibility geometry separately. Start the configured secure interval at contact, preserve first-touch time/location through deflections, and let the rules judge the catch.

### 11. [P2] Swinging HBP cannot be judged from the current fact lifecycle

**Location:** `sim/crates/bb_rules/src/engine.rs:54`–`:84`; `sim/crates/bb_game/src/game.rs:100`–`:101`, `:136`; `sim/crates/bb_rules/scenarios/pitch/pitch-002.toml:15`.

`SwingMeasured` briefly sets `p.swung`, immediately discards the pitch state, and applies a call. Therefore the HBP branch can never observe `p.swung == Some(true)` from a prior swing fact. A body touch before the end-of-swing fact awards first; a swing fact first makes a later body touch silently ignored. The game emits only the body touch whenever `rec.hit_batter`, even when `rec.swung` is true. The scenario titled “Pitch hits a swinging batter” contains **no body touch** and tests an ordinary swing instead. A swing takes precedence over an HBP award. [MLB HBP definition](https://www.mlb.com/glossary/standard-stats/hit-by-pitch)

**Scenario:** body contact followed by the measurement confirming the offer. Expected one strike, no award; current code awards first and advances the lineup.

```toml
id = "review-swinging-body-hit"
initial = { inning = 2, batter = "A.4" }
step = [
  { fact = "PitchDelivered", t = 0.0 },
  { fact = "PitchCrossed", t = 0.39, in_zone = false },
  { fact = "BallTouched", t = 0.4, what = "Batter" },
  { fact = "SwingMeasured", t = 0.45, swung = true }
]
expect = { strikes = 1, balls = 0, bases = {}, batter = "A.4", outs = 0 }
```

**Suggested fix:** accumulate pitch evidence until judgment is complete, including the swing measurement and body-contact zone geometry; then apply exactly one result. Send the measurement on HBP paths as well. Add an actual body-contact step to `pitch-002`.

### 12. [P2] A runner who left early gains appeal immunity by standing on another base

**Location:** `sim/crates/bb_rules/src/engine.rs:119`–`:129`, `:478`.

`TagContact` returns for **any** `on_base.is_some()` before testing retouch obligations. A runner who left first early, reached second, and is tagged there is protected as though the second-base touch satisfied the appeal. Rules-spec §5.6 explicitly permits appealing by tagging that runner before legal retouch of the original base. An off-base tag on the same runner is mislabeled `Tag` instead of `Appeal` as well.

```toml
id = "review-tag-appeal-on-next-base"
initial = { inning = 6, batter = "A.4", bases = { first = "A.1" } }
step = [
  { fact = "PitchDelivered", t = 0.0 },
  { fact = "BatContact", t = 0.42 },
  { fact = "BaseLeft", t = 0.5, agent = "A.1", base = 1 },
  { fact = "GloveSecured", t = 4.0, fielder = "H.CF", pos = [0.0, 95.0] },
  { fact = "BaseTouched", t = 4.5, agent = "A.1", base = 2 },
  { fact = "TagContact", t = 5.0, fielder = "H.2B", runner = "A.1", holding_ball = true, expect_events = [{ event = "RunnerOut", runner = "A.1", kind = "Appeal" }] },
  { fact = "PlayEndCheck", t = 7.0 }
]
expect = { outs = 2, bases = {}, score = [0, 0] }
```

**Suggested fix:** evaluate unresolved leave-early appeals before ordinary safe-on-base protection and retain the appeal classification for run resolution.
