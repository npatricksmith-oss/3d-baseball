# Independent review: fielder momentum

Reviewed `1a0eaef..21a4535` against `CLAUDE.md`, `docs/agent-models.md` §§5.1–5.8, and `docs/calibration/2026-11-12-fielder-momentum.md`. Locations below refer to **21a4535**, not subsequent workspace changes. No P1 finding was established. The calibration endpoints reproduce, but the movement and receiving defects below remain.

## Findings

### 1. P2 — Arrival snapping bypasses the braking limit and erases momentum

**Location:** `sim/crates/bb_game/src/live.rs:430`, `sim/crates/bb_game/src/live.rs:457–458`.

The crossing condition sets velocity to zero regardless of its magnitude. The zero-distance branch does the same. Consequently a late chaser does not run through his intercept as documented: crossing the target gives him an instantaneous stop, and a subsequent replan can send him off from rest in any direction. The 5 cm / 0.5 m/s settling condition does not protect these other branches. This undermines the movement law precisely around intercepts, bags, and changing targets.

**Failing scenario and verification:** In a temporary private-module probe using the actual `fielder_step`, put a default infielder at `(0,0,0)`, target `(0.05,0,0)`, velocity `(7,0,0)` m/s, at `t=2`. Both `Idle` and late `Chase` return position `(0.05,0,0)` and **zero velocity after one 1/60 s tick**. That is an effective 420 m/s² stop instead of at most 6 m/s². At zero target distance, even a moving fielder immediately loses all velocity. The existing `stopped_fielders_have_no_speed_and_running_ones_keep_it` test actually accepts this behavior.

Only snap when the remaining motion is consistent with settling at low speed. Otherwise preserve velocity and integrate the crossing, including the overshoot and subsequent braking/turning.

### 2. P2 — `stop_point` omits a factor of speed

**Location:** `sim/crates/bb_game/src/live.rs:382`; capture uses at `sim/crates/bb_game/src/live.rs:1219` and `:1224`.

`vel * (v / (2 * BRAKE) / v)` simplifies to `vel / (2 * BRAKE)`. Its displacement magnitude is therefore `v / (2 * BRAKE)`, whereas constant-deceleration stopping distance is **v² / (2 * BRAKE)**. Equivalently, the vector displacement should be `vel * (v / (2 * BRAKE))`. This affects batted-ball catches and relay catches. A batted-ball holder also retains `Chase` and its old intercept time, so the claimed planting behavior need not even request braking toward this short target.

**Failing scenario and verification:** With velocity `(7,0,0)` m/s, the actual helper places the target **0.583333 m** ahead, instead of **4.083333 m**. Feeding that target into the actual movement step as a late chaser reaches it in **0.083333 s**, with a last pre-stop speed of **7.082701 m/s**; the crossing snap then stops him. Honest braking takes **1.166667 s**. These values came from the `/tmp` probe, not from a reimplementation of the helper.

Correct the displacement and explicitly put a planting holder into a braking movement state. Correcting the formula alone does not fix finding 1 or a holder who still requests full chase speed.

### 3. P2 — The sideways-drift reach estimate is substantially optimistic

**Location:** `sim/crates/bb_game/src/live.rs:410–413`; consumed by `:653–654` and `:799–800`.

Adding the sideways stopping distance at right angles accounts for extra ground, but gives the runner the ordinary straight-line forward acceleration throughout the turn. That is not how `fielder_step` accelerates: acceleration is resolved along the current velocity, whose initial direction may be perpendicular to the goal. The estimate therefore is not conservative and can admit intercepts the movement model misses even with a fixed, perfect target. It also feeds confidence/slack calculations, so this is more than visual drift.

**Failing scenario and verification:** Default infielder, position `(0,0,0)`, velocity `(0,7.9,0)` m/s, goal `(5,0,0)`. `run_terms` followed by `run_time_from(d - 0.7, ...)` predicts planning reach in **1.358859 s**. Stepping the actual movement toward that unchanged goal as a late chaser gives a gap of about **3.82 m** at the nearest tick to that ETA. He first enters the actual **1 m** standing reach at **1.900000 s**, and the planner's **0.7 m** reach at **1.950000 s**. A ball passing the goal at the predicted time is missed. At a 10 m goal the corresponding prediction is **1.954385 s**, versus **2.283333 s** to actual standing reach.

These probes used the production helpers and production step, with no perception changes or replans. They isolate the turn estimate from read noise. Use a reach estimate that includes the initial turn's time/forward-acceleration cost, or a conservative bounded integration of the movement law.

### 4. P2 — Shrinking the receiving point toward the aim can pin a receiver to an unreachable throw

**Location:** `sim/crates/bb_game/src/live.rs:1356–1358`, `sim/crates/bb_game/src/live.rs:1379–1380`.

The prior pulls the read's catch point toward the bag, then the foot-on-bag test treats that posterior mean as a catch point guaranteed to lie within reach. A read outside the anchored envelope can thus become an inside-envelope target. With quarter-second reads, the receiver may remain planted until the next correction comes too late. The 10% reach reserve does not prevent this. This is a specific failure of the prior/anchoring combination, not a claim that every errant throw must be caught.

**Failing scenario and verification:** Receiver initially standing on a bag; release 20 m away along +Y, 37 m/s, solved chest-height throw passing **1.7 m to the side** of the bag. Use default receiving attributes, perfect hands, and zero perception-error draws. The unmodified receiving and movement code misses: minimum horizontal separation is **1.698866 m**, outside the 1.5 m receiving reach. He is still targeting the bag 0.5 s after release; a later read finally sends him sideways after the useful opportunity.

In paired `/tmp` probes with the same throw and observations, changing only the movement target to the already-computed weighted `catch_at` catches at **0.566667 s**, after moving about **0.218 m** sideways. Alternatively, retaining the bag rule but using the unweighted read catches at **0.583333 s**. The failure and both counterfactual catches reproduce at first, second, and home. The synthetic first-base case used the same default infielder attributes as the second-base receiver; home used catcher attributes. Relay throws correctly bypass the bag rule.

Evaluate anchored catchability using the receiving belief's uncertainty/reachable envelope, rather than treating a prior-shrunk point as certain. Preserve the ability to make a small necessary step before a clearly outside throw passes.

### 5. P2 — Grounder restoration still restores bag targets for noncandidate infielders

**Location:** `sim/crates/bb_game/src/live.rs:764`, `sim/crates/bb_game/src/live.rs:774–779`.

Only the re-call candidates get fresh intercept targets. `own`, however, copies every fielder's current movement target, including cover targets left from earlier assignments. The subsequent loop examines all three breaking infielders. A noncandidate's old ground-ball belief and finite intercept time can qualify him to break, while `own[i]` is a bag. This repeats the bug the commit intends to remove, and additionally removes that fielder from base coverage until a later update.

**Failing scenario and verification:** Instrumented only the isolated commit copy to log this exact transition during normal complete games: a noncandidate previously in `Cover(base)` becomes `Chase`, keeping the same bag target. There were **46 such fielder transitions in games with seeds 0–7**. For example, **seed 0, play 31, t=1.546204 s**, fielder index 3 changes from `Cover(2)` to `Chase` while still targeting second, `(0,38.794706,0)`; primary is index 4. The existing `trace_play -- 0 31` example reproduces the play, which is not secured until **1.812870 s**. Another example is seed 0, play 75, index 4 leaving `Cover(3)` for a chase whose target remains third.

A separate direct probe also reproduces the transition. Retain a dedicated own-intercept point, refresh every infielder whose break is reconsidered, or restrict restoration to fielders whose saved target is actually an intercept.

### 6. P3 — A wall carom is still classified as straight off the bat

**Location:** `sim/crates/bb_game/src/live.rs:565–575`.

The condition excludes ground bounces and replacement free paths, but does not exclude wall contacts already recorded inside the original path. Between a wall carom and the next ground bounce, it therefore reconstructs a launch angle using elapsed time since bat contact and applies the contact-spin table. This contradicts the documented post-deflection topspin belief and can misdirect a fielder recovering a carom.

**Failing scenario and verification:** Default park/environment, contact at 0.4 s, initial ball position `(0,110,1)` m and velocity `(0,25,3)` m/s. The actual path hits the center-field wall at **0.892173 s**, before its first ground bounce at **1.239551 s**. With zero read error, a read at **0.912173 s** predicts **+2283.901 rpm** backspin instead of the documented **−600 rpm** post-deflection assumption. The probe inspected the first spin state of the actual `predict_from` result. Include observed wall contact in the history predicate.

## Verification

- Read the requested invariants, specification sections, calibration report, the complete relevant commit diff, and movement/planning/capture/receiving/holder call paths. The initial checkout was clean at `21a4535`. Other work subsequently changed source files in the shared workspace; those changes were left untouched, and subsequent verification used `git archive` snapshots under `/tmp`. All findings and line numbers concern the requested commit. This review document is the only repository file authored by this review.
- Ran `cd sim && cargo test -p bb_game`: **47 tests passed**, no failures; doc tests passed. Zig initially could not write its home-directory cache. Re-ran with `ZIG_GLOBAL_CACHE_DIR=/tmp/codex-momentum-zig-global` and `ZIG_LOCAL_CACHE_DIR=/tmp/codex-momentum-zig-local`; no privileged operation was needed.
- Ran the requested `cargo run --release -p bb_cli -- games --n 512 --seed 2`. Repeated the final-commit batch in an isolated snapshot with `--no-default-features` to omit the optional Rerun viewer. Also ran the baseline snapshot, with a separate Cargo target directory to avoid sharing cached artifacts between revisions. Both endpoints reproduce the report's displayed precision:

  | Revision | Runs/team-game | GB hit | LD hit | FB hit | PU hit | Rules warnings |
  | --- | ---: | ---: | ---: | ---: | ---: | --- |
  | `1a0eaef` | 5.14 | .385 | .716 | .141 | .017 | None |
  | `21a4535` | 5.29 | .400 | .727 | .110 | .005 | None |

  The final batch also reports 33 extra-inning games, 46 walk-offs, K 17.4%, BB 7.4%, HBP 0.3%, HR/PA 4.10%. Intermediate experimental rows were not reconstructed; they are not separate retained revisions in this range. The report's approximately 260 ms single-thread timing was not validated: the calibration runs used 16 threads amid concurrent work, so their wall times are not a controlled comparison.
- Ran `cargo test --release -p bb_game` in the isolated final-commit copy with seven temporary probe tests added: **54 tests passed** (47 existing plus seven probes). Later ran two additional focused probes separately: eight complete games for the restoration diagnostic, and call-tier replacement. These passed. Probe code, diagnostic variants, logs, and source snapshots were confined to `/tmp`.
- The movement sweep covered **10,368** fixed-target cases: 12 distances from zero to 100 m, six initial speeds from zero to 7.9 m/s, headings every 5°, and both settling and late-chase behavior, for up to 40 simulated seconds each. **No NaNs or persistent nonarrival/orbit were observed** with default infielder attributes. This is bounded coverage, not proof for every configurable attribute or moving target. It did expose the arrival snap and optimistic turning ETA above.
- Checked the straight-line formulas and inverse bracket. For positive finite speed/time-constant attributes and `v0 >= 0` within top speed, the exponential distance law is correct; the discrete step is its first-order integration. For `v0 < 0`, the brake distance, stop time, subsequent run, and bisection lower/upper bounds are correct. An independent Python scalar solve gives **2.9626493868 s** for the unit test's 7 m/s reversal, 5 m goal, 7.9 m/s top speed, and 0.7 s acceleration constant; the outward stopping distance is **4.0833333333 m**.
- The reversal test is not wholly tautological: its `7/6` stop time and `49/12` distance are independent physical expectations, and it checks actual stepped motion. Its comparison of `run_time_from` against an expression using production `run_time` shares a helper, so it is not an independent validation of that helper. The curve test likewise checks actual motion against independent broad inequalities, but is a sanity test, not a closed-form derivation or a test of the reach planner. Neither test covers findings 1–3 adequately.
- Exercised confident OF versus confident IF calls, then moved the incumbent OF beyond reach and refreshed its belief: the confident IF replaced it despite hysteresis. Equal-score calls in candidate orders `[5,3]` and `[3,5]` both chose index 3. Inspection confirms higher tiers bypass the same-tier margin; confidence can still be wrong because of finding 3. The grounder time cutoff excludes late pop-up landings as intended; the restoration defect is finding 5.
- Inspected `try_captures` deferral: its next-position calculation calls the same `fielder_step` used by movement, at the same tick time, and excludes intervening ground/wall/end boundaries. No additional inconsistency caused by replacing scalar speed with velocity was established. It inherits the movement defects above.
- Ran receiving probes over **144** throws (three bases, six distances, eight lateral offsets) with exact observation draws and ordinary modeled uncertainty, plus the paired cases in finding 4. Not every miss is a defect; short reaction windows and large offsets can make a throw uncatchable. Relay targeting is exempt from bag anchoring, and home maps to base 4 correctly. `holder_act` checks taking a nearby runner's base before transfer readiness and can override the planting target in the capture tick; no separate forced-base planting regression was established.
- Ran `batted_spin -- 48` and `trace_play -- 0 31` in the isolated final-commit copy. The spin medians from −25° through 55° bins were **−7040, −4708, −2690, −763, 1301, 3232, 4818, 5808, 6707 rpm**. These broadly support the learned table's scale, but do not exactly reproduce its earlier fitted sample; changing fielding changes later game sequences. Interpolation probes at −100°, −25°, −20°, 0°, 55°, 100° returned **−7060, −7060, −5905, −1765, 6470, 6470 rpm**, respectively. Endpoint clamping/interpolation is sound for finite input. The initial workspace `pu_hits -- 16` run produced no uncaught-pop-up lines, but concurrent workspace changes make that run supporting diagnostics only, not commit-specific evidence.
- The straight-off-bat test reads path metadata, but its first-bounce comparison answers whether a bounce has **already happened**; it does not expose future bounce time/location to the agent's decision. The launch-angle estimate uses noisy current velocity plus elapsed gravity, not true launch velocity/spin. It is an approximation that ignores drag/lift in reconstructing launch angle. Finding 6 concerns missing observed wall history, not a future-state leak. The receiver prior uses the throw's past release position to determine throw length, not its future endpoint or realized motor error.
- Inspection found no new unordered collection iteration, outcome rolls, RNG-purpose renumbering, or non-libm transcendental calls in production changes. `atan2` and `hypot` go through `bb_math::m`; `to_degrees` is a unit conversion, and the new `powi(2)` is an integer square, not a transcendental. The inherent `hypot` in the existing fine-search unit test is test-only. Existing same-seed cross-thread log/digest and recording-independence tests passed. This establishes tested same-build determinism, not a cross-platform replay proof. The new main empirical knobs are named (`BRAKE`, `READ_VEL_FRAC`, `CALL_SLACK_S`, `RECALL_MARGIN_S`, `GROUNDER_BOUNCE_S`, `THROW_MISS_FRAC`); the 0.5 m prior floor and 0.9 receiving reserve remain literals and should be named when revising that logic.
- Ran `git diff --check 1a0eaef..21a4535`: passed.

## Resolution

Open.
