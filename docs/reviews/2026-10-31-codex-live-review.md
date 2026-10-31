# Codex live-ball review — 2026-10-31

Reviewed `sim/crates/bb_game/src/live.rs` and its integration in `game.rs` against PLAN §4, agent-models §§5–6, rules-spec §§2, 5–6 (also its fact-ordering contract in §1.2), and the previous M4 review. No source files were edited by this review.

The checkout was being edited and committed by another process during the review. **All line references below use the final review snapshots**, copied to `/tmp/codex-live-review/`, with these SHA-256 hashes:

| File | SHA-256 |
| --- | --- |
| `sim/crates/bb_game/src/live.rs` | `bdfd4e96faabe2bc5a50e84f3c45e46dab9146ae3806436a5ed8ae7c33ffdd0d` |
| `sim/crates/bb_game/src/game.rs` | `c52677fe81ae29d39b0e22c922848067eafdddd5d93790ffc5bca2bec88e06b4` |

HEAD was `0981df7` when those snapshots were taken; they include working-tree changes. Later edits can move the referenced lines. Findings are numbered in severity order; scenarios are code-derived examples, not executed reproductions unless explicitly stated.

### 1. [P1] The initial runner decision still reads the actual future landing time

**Location:** `sim/crates/bb_game/src/live.rs:954` (`likely_catch`).

**Problem:** `sim.out.path.first_bounce()` is the original world's complete free-flight path, not any runner's observation or prediction. At contact plus 0.5 s, every runner learns the true future first-ground-contact time, then combines it with the primary fielder's private `t_int` to choose whether to tag up. This directly contradicts D1 and the previous review's claim that finding 5 was fixed in substance. After a deflection it can also refer to a path the ball is no longer following.

**Concrete scenario:** Two airborne balls look the same to the runner at 0.5 s but have different unobserved spin and hence different future landing times. With the fielder's believed intercept near the comparison threshold, one makes the runner hold and the other makes him run, solely because the simulator inspected the future landing. A rim deflection before 0.5 s leaves the comparison using the original landing time.

**Suggested fix:** Estimate catch likelihood from the runner's own noisy ball/fielder observations and internal trajectory model. Do not pass the world's `BallPath` or a fielder's private intercept belief into that policy.

### 2. [P1] Runner and defensive policies directly share private state and exact world measurements

**Location:** `sim/crates/bb_game/src/live.rs:604–640`, `:765–818`, `:306–343`, `:194–205`, `:650–668`.

**Problem:** There is still no observation boundary around several important decisions:

- `ball_time_to` reads the holder's exact position, arm strength and `ready_at`; for a loose ball it reads the chaser's private `t_int` and `target`; for a throw it reads the true release state and intended `to_base`. Adding one scalar noise draw to the resulting margin does not hide those private inputs.
- Runner blocking reads other runners' intended `goal`, even while they have not moved. The defense likewise selects a base from each runner's actual `goal`, exact `s`, and actual `vmax`, rather than estimating heading and speed from observation.
- Assignment compares everyone's private intercepts centrally, immediately updates everyone's roles/targets, and recall selects candidates using exact ball/fielder distances. A single minimum reaction time starts prediction for all nine fielders, including those whose own reaction time has not elapsed. The added secondary-chaser logic also reads the primary's private prediction.
- `emit` immediately marks runners out, and `caught()` immediately scans rules events. Other agents act on those judgments in the same tick, without the specified 0.2 s umpire-signal delay; calls likewise have no specified 0.3 s communication delay.

**Concrete scenario:** A runner privately changes his goal from third to home while still at third. The holder changes his throw choice immediately, before any motion or coach signal reveals the decision. Conversely, a chaser privately misreads a loose ball and revises his intercept time; every runner instantly changes his estimate of the defense's arrival time. With different reaction attributes, a slow-reacting outfielder can win the assignment using a prediction made before he has reacted.

**Suggested fix:** Give each agent an observation snapshot with noisy visible positions/velocities and timestamped calls/umpire signals. Keep goals, planned intercepts, transfer deadlines, execution attributes, and throw intent private. Infer intended destinations from observed movement or explicit communication. Run each agent only after its own reaction deadline.

### 3. [P1] Capture implicitly aims the glove at the true ball, bypassing a bad read

**Location:** `sim/crates/bb_game/src/live.rs:468–508`, `:973–977`.

**Problem:** Once the true ball enters the reach envelope, the pocket test is just `hands_sigma * hypot(ex, ez)`. It never measures the ball against a glove position commanded from the fielder's belief; the nominal pocket is implicitly centered on the true ball. A completely wrong prediction therefore cannot make the glove aim at the wrong point. Dive initiation also branches on true current speed, distance, and height, and the slow-ball shortcut overwrites the target with exact ball coordinates. Exact geometry is appropriate for resolving a collision, but here it also supplies the motor target/decision.

**Concrete scenario:** A fielder's prediction puts a short hop on his left, while roughness sends the actual hop to his right. If both are inside the generous reach envelope, the routine pocket test can secure the actual ball immediately without moving the glove across or paying the specified hand-speed limit. An actual ball just beyond standing reach can also trigger an instant dive even when the agent's observed ball position is elsewhere.

**Suggested fix:** Represent a glove target and glove motion driven by observations, including hand-speed and dive commitment limits. Resolve contact against that realized glove volume. Keep the Gaussian placement error as physical input noise around the commanded placement; the norm test itself need not be an outcome roll.

### 4. [P1] Tick processing does not implement chronological facts or the promised tie priority

**Location:** `sim/crates/bb_game/src/live.rs:689–715`, `:1010–1024`.

**Problem:** Each tick processes ball-dead events, captures, tags and defensive base touches before moving runners. `move_runners` then advances a full `DT` but timestamps the resulting touches with the tick's starting `self.t`. Fielders also move a full tick, while their base contacts are generally tested on the next iteration. These are different effective clocks. There is no merge by actual crossing time or by the required equal-time priority. A third out or dead ball sets `done` before a same-tick home/base touch can even be produced.

**Concrete scenario:** A runner and a ball-holding defender enter the base's contact volume during the same 1/60 s interval. The runner's crossing is backdated to the interval start; the defender's is checked on the following tick. Conversely, a defensive contact already eligible at the tick boundary is processed before any runner touch that the simulator would assign that same timestamp, so an exact-time force can win despite the specified runner-touch priority. A third tag out can similarly discard a same-window home touch.

**Suggested fix:** Integrate actors over a shared interval, detect contact times within that interval, and merge ball, glove, base and tag events by `(time, explicit priority, stable source order)` before stepping rules. Do not send future actor movement with the start-of-interval timestamp.

### 5. [P1] Rim contacts still do not report the first fielder touch

**Location:** `sim/crates/bb_game/src/live.rs:510–518`, `:507–508`; supporting interface: `sim/crates/bb_rules/src/state.rs:153–183`.

**Problem:** A rim deflection changes the world's ball path but emits no glove-touch fact. The only fielder contact the rules engine receives is eventual `GloveSecured`. Fair/foul is therefore decided too late, and tag-up eligibility starts at the wrong contact. The current input vocabulary lacks a separate glove-touch fact, so physically simulating the deflection alone does not fix the lifecycle. This is the remaining part of previous finding 10, with effects beyond visualization.

**Concrete scenario:** A grounder is touched by a fielder on fair ground in front of third, deflects foul, and is secured or settles there. It should become fair at the first fielder touch; the supplied facts can instead make it foul. On a bobbled fly, a runner who legally leaves after the first rim touch is treated as having left before the eventual catch.

**Suggested fix:** Add and emit an explicit first/contact glove fact at the deflection or initial capture time, with ball coordinates. Let rules preserve first-touch time separately from continuous possession and catch confirmation.

### 6. [P2] The receiver gets the exact executed throw, and the ball observation has noiseless channels

**Location:** `sim/crates/bb_game/src/live.rs:235–252`, `:579–590`, `:973–977`.

**Problem:** `update_thrown` analytically solves the landing/catch-height point from the true noisy `r0` and `v0` and sets the receiver's target to that point immediately after release. This reveals the thrower's execution error without any read error or observation latency. The general fielder prediction perturbs only horizontal components: both `p.z` and `v.z` remain exact. The slow-ball pursuit shortcut also uses exact horizontal coordinates instead of the distance-scaled observation already available.

**Concrete scenario:** A throw is misdirected several degrees. On the next tick the receiver knows the exact displaced chest-height intersection and runs straight to it. A distant fielder also knows the exact vertical speed at the apex despite having substantial horizontal read uncertainty.

**Suggested fix:** Track a noisy perceived throw state for the receiver and predict from that estimate. Include uncertainty in observed elevation/vertical velocity (or observe angular measurements and derive a state estimate). Retain small close-range error rather than switching to truth.

### 7. [P2] Runners never retreat after an unexpected catch, and stopped runners do not reconsider

**Location:** `sim/crates/bb_game/src/live.rs:623–648`, `:654–724`, `:949–964`.

**Problem:** The one initial fly decision is the only place that sets `tag_up`. Thereafter a catch is handled only for runners already waiting with that flag. A runner who committed on an apparent hit keeps his forward goal after an unexpected catch and never returns to retouch. There is no policy path that assigns a retreat goal. A runner who holds a base simply takes the early `continue` at lines 682–684 forever; ball fielded, throw released, missed reception, and a coach/leading runner moving do not trigger reconsideration, contrary to agent-models §6.3.

**Concrete scenario:** A runner leaves first on a sinking liner that an infielder unexpectedly catches. He continues toward second and can be appealed out instead of reacting to the catch and returning. Separately, a runner holds second while the ball is controlled, then watches a throw get away into the outfield; he remains frozen on second for the rest of the play.

**Suggested fix:** Reconsider on observed catch/fielding/release/loose-ball signals and relevant occupancy changes. Implement a return-to-start/retouch state with ordered reverse touches, and allow a stationary runner to resume advancing when his observed margin changes.

### 8. [P2] Goal blocking does not enforce runner order or resolve two runners on one base

**Location:** `sim/crates/bb_game/src/live.rs:638–641`, `:654–724`, `:740–743`; downstream consequence: `sim/crates/bb_rules/src/engine.rs:687–693`.

**Problem:** Blocking checks only whether a preceding runner's *goal equals the next base*. Forced movement bypasses that check, and the batter's initial run to first has no occupancy check at all. There is no world crossing/passing fact, spacing constraint, or treatment of a non-entitled runner sharing a base. The holder skips tags whenever a runner is geometrically near any base, even if another runner is entitled to it. At settlement, rules warn about two occupants and overwrite one, so a runner can disappear without an out or run.

**Concrete scenario:** On a high fly judged likely to be caught, the original first-base runner waits on first while a fast batter reaches and rounds first. The batter can pass the waiting runner without a passing out. If the fly then drops, the original runner's unconditional forced advance can take him toward a base already occupied by the batter, who may now be holding. Neither the movement loop nor base-contact tagging resolves that occupancy.

**Suggested fix:** Detect passing from continuous runner positions and report it through an appropriate fact. Resolve entitlement when runners share a base; base proximity alone must not grant immunity. Treat goal blocking as policy advice, not the physical/rules invariant. The 32-game check observed five `two_runners_on_base` warnings; those aggregate warnings do not establish which exact play caused each one.

### 9. [P2] Ground and wall facts are grouped rather than merged by time

**Location:** `sim/crates/bb_game/src/live.rs:543–572`, `:194–198`.

**Problem:** `ball_facts` appends every due ground touch, then every due wall touch, then the endpoint. It does not sort the collected facts. `floor` is captured once before collection, and `emit` only updates a maximum; neither prevents a later item from going backward relative to an earlier item in the same batch. Clamping timestamps to an old fact time also hides timing errors instead of preserving physical contact times.

**Concrete scenario:** In one tick interval, a low ball hits the wall at 2.001 s and the ground at 2.009 s. The function sends ground at 2.009 before wall at 2.001. This violates the rules contract and can also choose the wrong first fair/foul contact near a line. The 32-game run reported one `time_reversed` warning, although the aggregate output does not identify its originating play.

**Suggested fix:** Merge all physical event streams chronologically, preserving their original times and the explicit tie order. Assert ordering at the producer; do not repair it with `max(floor)`.

### 10. [P2] A throw can be secured after it has already gone underground

**Location:** `sim/crates/bb_game/src/live.rs:468–470`, `:593–597`, `:1014–1018`.

**Problem:** Throws are sampled analytically without a ground stop. Capture runs before `update_thrown`, and the standing/dive tests impose only an upper height bound. A negative `b.z` therefore still passes. If the throw is secured, the ground transition is skipped entirely. If it is missed, the ball is moved back up to ground level at the end-of-tick horizontal position and restarted, delaying/moving its actual ground contact.

**Concrete scenario:** A low throw crosses the ground just after one tick. At the next tick it is a few centimeters below ground and within 1.5 m of its receiver. The simulator can emit `GloveSecured` and make a subsequent force/tag without reporting the ground hit or applying a bounce. This is not a physical scoop.

**Suggested fix:** Find the throw's first ground intersection within the interval and process it before any later capture, continuing through the bounce model. Require realized contact geometry above the surface; do not clamp an already-penetrating ball into a new trajectory after capture tests.

### 11. [P2] First capture is immediately reported as continuous secured possession

**Location:** `sim/crates/bb_game/src/live.rs:499–508`, `:731–778`.

**Problem:** A successful pocket test switches directly to `Held` and emits `GloveSecured` at first contact. There is no 0.3 s secure interval, provisional possession, or possible drop during that interval. `ready_at` delays a throw but does not delay the catch/out or prevent immediate tags and defensive base touches. Transfer time is a different quantity from continuous control.

**Concrete scenario:** A fielder pockets a fly while landing from a dive, and the batter is declared out instantly; a runner at his glove can be tagged in that same tick. The model cannot represent a ball coming loose before the required secure interval, nor can rules distinguish first touch from a confirmed catch.

**Suggested fix:** Track provisional capture/control and a secure deadline, preserve first-touch time, and emit secure/drop/release facts according to the specified possession lifecycle. Resolve tags/catches using the appropriate control interval rather than treating transfer readiness as confirmation.

### 12. [P2] Ground balls still never produce base-passage facts

**Location:** `sim/crates/bb_game/src/live.rs:543–577`; contract/interface: `sim/crates/bb_rules/src/state.rs:175`, `sim/crates/bb_rules/src/engine.rs:117–122`.

**Problem:** `BallPassedBase` appears in timestamp matchers but is never constructed by the live simulator. `ball_facts` reports only ground, wall, rest and out-of-play events. A pending infield ground ball can pass first/third fair, then receive its first decisive fact somewhere else. This is the acknowledged remaining portion of previous finding 21, not a completed fix.

**Concrete scenario:** A rolling ball crosses the third-base decision plane fair, then a later bounce carries it foul before it is touched. Rules never receive the fair passage; a subsequent foul-side ground contact or rest can incorrectly make it foul.

**Suggested fix:** Detect the first relevant near-ground crossing of the first/third-base plane on each realized free-ball interval and emit `BallPassedBase` at the crossing time with its measured side. Preserve contact/possession truncation when merging that event.

### 13. [P2] Wall catches do not check whether the fielder is inside the park

**Location:** `sim/crates/bb_game/src/live.rs:409–425`, `:454–470`, `:507–508`.

**Problem:** Movement has no field-boundary constraint and capture never checks the fielder's ground-contact point. A near-wall ball merely raises maximum catch height to 3 m. Belief targets and backup targets can lie beyond the fence, so a fielder can stand outside the park and still emit a valid-looking catch. `GloveSecured` supplies ball coordinates, not fielder foot coordinates, so the rules engine cannot detect this.

**Concrete scenario:** A noisy depth read takes a left fielder slightly beyond the fence. A ball still just inside the wall and below 3 m comes within his reach; the simulator records a catch before the ball's boundary event even though his feet are outside the playing field. That is not the permitted reach-over robbery described in rules-spec §5.3.

**Suggested fix:** Constrain actor movement against park geometry and include/check fielder ground-contact position when validating a capture. Permit the glove, rather than the feet, to extend beyond the wall.

### 14. [P2] The runner's force calculation omits the batter being out

**Location:** `sim/crates/bb_game/src/live.rs:623–638`.

**Problem:** For a runner starting at first, `(1..r.start).all(...)` is always true. For runners farther ahead it checks original occupants of bases 1 upward but never the batter (`start == 0`). A ground-ball out of the batter therefore does not remove the agent's `must` condition, although it removes all forces in the rules engine. This is an incorrect decision even after allowing perfect observation of the out.

**Concrete scenario:** A popup expected to be caught drops beside first. The first baseman fields the bounce and forces out the batter in `holder_act`, before the waiting first-base runner reacts to the drop in `move_runners`. That runner's force is gone, but `runner_decide` still orders him to leave first regardless of the ball's arrival margin.

**Suggested fix:** Compute the runner's believed force chain including the batter's alive status and following-runner outs, updating it on observed rulings. Keep that agent belief separate from the rules engine's authoritative force map.

### 15. [P2] Settlement can be declared outside the infield; the return target also creates a latent stall

**Location:** `sim/crates/bb_game/src/live.rs:800–815`, `:833–838`, `:928–942`.

**Problem:** `settled` accepts a holder anywhere in the park once all surviving runners have `v == 0`, bypassing rules-spec §5.8's mandatory infield possession. Separately, the no-play return target is second base, which is about 38.8 m from home and therefore outside this implementation's 30 m infield criterion. Once there, the holder repeatedly chooses to take the ball to the same base. The current invalid settlement escape masks that lack of a legitimate continuation; removing only the escape would leave this state cycling until timeout. The simulator also uses 40 s while the spec's operational limit is 60 s.

**Concrete scenario:** An outfielder finishes his transfer with all runners holding. He can end the play while still in the outfield, without returning the ball. If he returns it to the second baseman, that holder still does not meet the stated infield definition and has no policy to move inside 30 m.

**Suggested fix:** Require infield possession for normal settlement, and explicitly move/return a no-play ball to a valid settling location. Handle any stationary runner between bases using the specified inactivity interval. Use the agreed timeout configuration rather than a separate hard-coded limit. A timeout must remain a reported model failure, not the normal completion route.

### 16. [P2] Tag immunity, force eligibility and runner base touches use incompatible base geometry

**Location:** `sim/crates/bb_game/src/live.rs:692–715`, `:740–756`.

**Problem:** A runner is immune to tags whenever within 0.3 m of any base center, but `BaseTouched` is emitted only when his scalar path coordinate crosses the exact center. Defensive force reporting separately stops considering an approaching runner once he is within 0.1 m of the center. These conditions disagree with each other and with rules' open base-touch interval. The runner can be classified as off base by rules but protected from both a tag and a force by the world.

**Concrete scenario:** A receiver secures a throw while standing on first and the batter is still 0.08 m short of the runner's touch threshold. The batter has no `BaseTouched` yet, but `pending` is false and the tag test also calls him on base. The fielder never reports the out; the batter then reaches first on the runner update. A runner leaving a base gets similar tag immunity during the first 0.3 m after `BaseLeft`.

**Suggested fix:** Define shared base/body contact volumes and derive touch, leave, on-base immunity and defensive base contact from their entry/exit times. Do not add unrelated 0.1/0.3 m policy exclusions around those physical facts. Handle entitlement separately when two runners touch the same base.

### 17. [P2] Throw RNG keys depend on unrealized future bounces and can be reused

**Location:** `sim/crates/bb_game/src/live.rs:530–537`, `:820`.

**Problem:** `set_free` advances `bounce_k` by the number of bounces in the entire newly precomputed future path. A capture can truncate that path before most of those bounces happen, but a subsequent throw still uses the resulting counter as its `ThrowExecution` sub-index. Thus unrealized future path details choose current execution noise. Conversely, consecutive throws by the same fielder without another `set_free` reuse the identical throw key and therefore identical speed/direction deviates. Separate noise purposes are coupled through a counter with the wrong lifetime.

**Concrete scenario:** A rim-deflected ball is secured promptly. Changing only how the never-reached tail of its free path would bounce changes `path.bounces.len()`, selecting a different noise draw for the immediate throw. During a multi-throw play, the same infielder receiving and throwing again can repeat exactly his previous angular error because no loose-ball path incremented the counter.

**Suggested fix:** Give each fielder a monotonically increasing throw-attempt index and each realized free-ball episode/bounce its own causal identifiers. Reserve keys independently by purpose, without counting future events that may never occur. This is a causality/noise-isolation hazard, not evidence that an unchanged build and seed produce different runs.

### 18. [P2] A diving rim contact bypasses the get-up interval

**Location:** `sim/crates/bb_game/src/live.rs:496–525`, `:409–412`.

**Problem:** A dive sets `dove = true`, but the rim branch sets only a 0.5 s capture block and returns before the missed-dive get-up assignment. `move_fielders` gates recovery using `ready_at`, which can still be zero, so the diver immediately runs again. The `dove` flag is also never cleared after recovery, causing later ordinary captures to inherit a fresh dive get-up penalty.

**Concrete scenario:** A fielder dives and tips a liner. Unlike a clean miss, he gets no 1.5 s recovery pause and can chase the deflected ball immediately, then re-field it after 0.5 s. Much later in that play, a standing pickup can still add 1.5 s because the historical `dove` flag remains set.

**Suggested fix:** Start a recovery state/deadline for every committed dive before branching on contact result. Gate movement and further actions consistently, then clear the active dive state when recovery completes. Track historical visualization information separately if needed.

### 19. [P3] Expensive predictions are repeated immediately, and profiling is always active

**Location:** `sim/crates/bb_game/src/live.rs:228–259`, `:321–332`, `:430–443`, `:982–1007`, `:1039`.

**Problem:** Assignment computes a full path and intercept but does not update `next_plan`. The main loop can immediately compute them again for the same chaser; a close-range read in `try_captures` can do so again in the same tick. The regular primary replan can be followed by recall, which predicts that primary again. All of these construct full bounce/roll paths, and the standing/dive intercept fallback can scan a path twice. `Instant::now`, elapsed-time collection and thread-local profiling updates also execute unconditionally despite the profiling module's `BB_PROF=1` description.

**Concrete scenario:** On the initial assignment tick, a nearby grounder can trigger assignment prediction, the main-loop replan, and the close-read prediction without intervening world movement. This work also happens in non-recording batch games. The concurrent change gating actor/ball recording on `record` removes the earlier unconditional-track allocation waste, so that fixed issue is not reported here.

**Suggested fix:** Cache predictions/intercepts by agent and observation timestamp, set deadlines whenever a prediction is refreshed, and let recall reuse current candidates' predictions. Short-circuit prediction once the useful horizon is covered. Gate profiling once per run/build rather than measuring every call by default. Measure before changing model cadence or selecting candidates from truth to save work.

## Verification and remaining limits

Executed the permitted commands with `PATH=$HOME/.cargo/bin:$PATH`, `ZIG_GLOBAL_CACHE_DIR=/tmp/zg`, and `ZIG_LOCAL_CACHE_DIR=/tmp/zl`:

- `cd sim; cargo test -q -p bb_game` — passed: 21 tests across the reported test binaries, plus zero doctests. This includes same-seed/cross-thread log identity and recording-independent digest checks.
- `cd sim; cargo run -q --release -p bb_cli -- games --n 32` — completed; 7.16 runs/team-game, 45.0 PA/team-game, 261 pitches/game, four extra-inning games, five walk-offs. Warnings: `{"time_reversed": 1, "two_runners_on_base": 5}`. No `play_timeout` warning was reported. Reported time was 4.9 s wall on the CLI's 16-worker batch; its derived per-core number is not a direct single-game benchmark.

Both commands started before the final snapshots and before several concurrent source edits. Their results characterize the build Cargo compiled during those invocations, **not a verified build of the final snapshot hashes above**. The changing checkout is also why different batch statistics here and in the earlier review are not evidence of same-build nondeterminism. No new tests, probes, or source changes were introduced; the concrete scenarios above remain proposed regression cases.

The fixed-step loop advances `t` unconditionally and has a finite play limit, so no unbounded loop was established for finite valid inputs. That guard does not prove normal settlement: finding 15 identifies an invalid settlement shortcut and a stall that must be addressed together. The batch exercised no timeout. No wall-clock/RNG/hash-iteration source of same-build run-to-run nondeterminism was found; profiling times do not feed decisions. Finding 17 concerns causal key isolation and repeated noise instead.

Defensive `BaseTouched { holding_ball: true }` and `TagContact` are currently emitted only from `Ball::Held`; I did not find a direct path that emits those while the ball is already `Thrown` or `Free`. The outstanding problems are capture/control timing, geometry and event ordering, rather than the old blanket assumption that a covering fielder has the ball. Free-ball ground/wall contacts are normally reported before that tick's capture, so the specific false post-ground capture found here concerns the separately sampled throw path (finding 10).

Relative to the prior review: finding 5 remains open (1–3 and 6 here); finding 6's unconditional over-fence bypass is removed, but legal robbery geometry remains incomplete (13); finding 9's frozen throw-arrival shortcut is removed, but reception timing still has physical errors (4, 6, 10, 16); finding 10 remains incomplete (5, 11); finding 21's abandoned-free-path emissions are improved, but required passage facts and ordering remain incomplete (9, 12); finding 24's fielder position teleportation is removed, while boundary and recovery constraints remain incorrect (13, 18).

## Resolution (Claude, 2026-10-31)

Fixes landed in two commits: `0c2585d` (most findings) and `3fd7757` (the runner-order follow-up, which also covers 7 and most of 8). After both: all workspace tests pass; `sim-cli games --n 128` gives 5.64 runs/team-game with **no rules warnings** and no play timeouts.

| # | Status | Change |
| :--- | :--- | :--- |
| 1 | Fixed | The initial tag-up decision uses the runner's own read (`runner_predict`: a noisy estimate of the ball's current state flown with the agent's physics) and `believed_field_time` (when some visible fielder could get there at league-typical speed). No world path, no fielder `t_int` |
| 2 | Partly fixed | Runners judge throws and fielding from league-typical arm, transfer and speed (`TYPICAL_*`, scouting-level knowledge) and the visible ball; a throw's destination is read from its heading. Blocking and the defense's choice of base use each runner's visible position and direction of motion (direction comes from the runner's goal with no observation latency). **Deferred to M5:** per-agent observation snapshots, umpire-signal and call delays, per-fielder reaction gating of the first prediction, and decentralized assignment |
| 3 | **Deferred (M5)** | A glove target driven by the fielder's belief needs the capture model rewrite (agent-models §5.1 atlas perception) |
| 4 | **Deferred (M5)** | Shared-interval integration with sub-tick contact times and a priority merge. The 60 Hz tick bounds the timing error at 17 ms |
| 5 | **Deferred (M5)** | Needs a new first-touch rules input (rules-spec §15) |
| 6 | **Deferred (M5)** | A noisy perceived throw state for the receiver and vertical read noise come with the atlas perception model |
| 7 | Fixed (`3fd7757`) | A runner who left his base before a catch goes back to retouch it, through intermediate bases in reverse order, then decides again from there. A runner waiting behind a runner ahead decides again once the runner ahead clears the next base. Reconsidering on other signals (a bobble, a throw release) is M5 |
| 8 | Mostly fixed (`3fd7757`) | Policy: a runner takes a base only after every runner ahead has passed it (he waits otherwise); a runner heading for a base the runner ahead has stopped at or is going back to returns to the base behind (only the advance from the start base can be forced, so the batter can retreat from second). The 86 `two_runners_on_base` warnings over 128 games are now 0. **Deferred to the rules engine and M5:** a passing fact, entitlement when two runners share a base, and the holder's tag exemption near any base (see 16) |
| 9 | Partly fixed | Ground, wall and settle facts within one tick are sorted by time before they go to the rules engine. The `max(floor)` clamp remains as a guard; producer-side assertions are M5 with finding 4 |
| 10 | Fixed | `update_thrown` runs before captures: a throw reaching the ground is found at its analytic ground time and continues through the bounce model before any capture test |
| 11 | **Deferred (M5)** | Provisional capture and a secure interval need new rules facts (with 5) |
| 12 | **Deferred (M5)** | `BallPassedBase` emission (also listed in TASKS) |
| 13 | **Deferred (M5)** | Park-boundary constraints on fielder movement and foot position in capture |
| 14 | Fixed | The runner's force belief requires the batter to be alive (`Sim::forced`) |
| 15 | Fixed | Settlement requires the holder inside the infield, whose radius is now 45 m (rules-spec §5.8 updated: the 30 m value excluded second base, the no-play return target) |
| 16 | **Deferred (M5)** | Shared base and body contact volumes |
| 17 | Fixed | Throw noise is keyed by a per-fielder throw counter, never by future bounce counts |
| 18 | Fixed | Every committed dive starts the get-up interval, rim contact included |
| 19 | Fixed | Predictions refresh `next_plan`; profiling is gated once per thread by `BB_PROF`. In `3fd7757` the agents' internal flight step went from 0.02 s to 0.04 s, which halves prediction cost with unchanged outcomes over 128 games |

**Performance, corrected.** The ~846 ms "per game per core" quoted after `0c2585d` was the CLI's wall time × 16 workers on an 8-core, 16-thread CPU, so it double-counted SMT. Measured on one thread (`BB_PROF=1 cargo run --release -p bb_game --example prof_game`): one game takes ~270 ms, of which live plays are ~117 ms (fielder predictions 64 ms, intercepts 20 ms, runner reads 6 ms). Pitches take the rest: `examples/perf.rs` puts a whole pitch at 540 µs, of which the pitcher's aim solve is 258 µs. The 200 ms budget now depends mainly on the pitch side.

**Run environment.** Runs/team-game went from 5.08 (after `0c2585d`) back to 5.64. The lower figure came from the runner bugs: trailing runners kept running into outs at bases the runner ahead held. 5.64 matches the level before the review (~5.6); the excess over MLB's ~4.5 is the open ground-ball problem (GB reached .55 vs MLB ≈ .24).
