# Owner-watch fixes: independent review

Reviewed `4ebe7fd..31d57bb` at `31d57bb`, after reading `CLAUDE.md`, PLAN's binding decisions, and the owner-watch notes. Focus: visible baseball and live-play correctness; no recommendation to tune percentages more closely. No P1 defect established. The findings below are ordered by practical impact.

## Findings

### 1. P2 — Home coverage never returns when the batter becomes a scoring threat

**Location:** `sim/crates/bb_game/src/live.rs:607–620` (`assign_backups`), `1279–1285` (`receiver_for`).

Removing the catcher's `Cover(4)` job with nobody initially on base leaves home without a cover man. The occupancy test uses `r.start`, so a batter reaching second or third never changes that classification. There is no subsequent coverage rotation while the ball is held or thrown. A fallback receiver starts toward home only when `holder_act` considers a throw there, potentially too late for an otherwise routine play.

**Executed scenario:** Recorded game, seed **14**, play **53**, starts with only the batter on offense. Runner 9 touches first at 4.821 s, second at 8.188, third at 11.538, and home at 14.888. The defense secures the ball near `(-24.67, 5.15)` at 13.271 s, but nobody covers home. At play end the catcher is still at `(32.40, 19.47)`, backing up first, and the pitcher is at `(-4.42, 48.57)`. This is a real scoring play, not just a role-table construction. A separate staged left-field-hit probe confirms there is no `Cover(4)` and the fallback pitcher needs another **3.095 s** to reach home.

**Suggested fix:** Rotate the catcher back to home, or explicitly give another available defender home coverage, as the batter advances into scoring position. Judge current visible threats rather than only starting occupancy. Preserve a receiver separately from the backup when assigning the throw's jobs.

### 2. P2 — A backup cannot catch an airborne overthrow

**Location:** `sim/crates/bb_game/src/live.rs:669–683` (`back_up_throw`) and `783–785` (`try_captures`).

The new backup assignments position a defender behind the receiver, but the capture filter permits only the designated receiver to attempt a thrown ball. That remains true after the ball has sailed over the receiver and directly into the backup's reach. It becomes available to the backup only after striking the ground or being deflected into `Ball::Free`.

**Executed scenario:** A high throw from `(0, 80, 1.8)` toward second passes above the receiver. The backup stands exactly 13 m beyond second at `(0, 25.7947, 0)`. At 2.206843 s the ball is directly over him at **1.30 m**, with perfect hands and no reaction/blocking restriction. `try_captures` leaves it `Thrown`. Giving the identical position and velocity a `Free` trajectory immediately yields `Held { by: 0 }`. The receiver is 13 m away; no interception in front of the base is involved.

**Suggested fix:** Permit the assigned backup to field a throw after it has passed the receiving area, using the existing physical reach and glove execution checks. Retain protection against an unrelated defender intercepting the intended throw in front of the receiver.

### 3. P2 — Every baserunner initially retreats from his new lead

**Location:** `sim/crates/bb_game/src/live.rs:1373–1376`, `1448–1482`, and `1115–1127`.

Runners initialize off base with `goal = start`. `move_runners` runs immediately, but the first runner decisions wait until contact +0.5 s. Only the batter gets a forward goal during that interval. Consequently even a forced runner on a grounder runs toward his old base, then reverses when told to advance. This consumes part of the lead's benefit and creates the opposite visible response from the intended shuffle/start.

**Executed scenario:** An average runner starts at path distance **32.932 m** from home and retreats to **31.892849 m** over the first 30 ticks: **1.039151 m backward** before a decision. The encoded timeline for seed **1**, play **7**, shows runner 1 moving from `(15.5083, 23.2864)` toward first before starting his advance. This is independent of the ball's eventual catch/drop decision.

**Suggested fix:** Give undecided runners an explicit hold-at-lead state, or make the appropriate contact decision immediately. A retreat should follow a catch/tag-up read; forced runners should receive their advance goal when their read permits it. Avoid representing “has not decided yet” as “run back to the base.”

### 4. P2 — Forced walks interpolate runners backward from their leads before advancing

**Location:** Changed standing positions at `sim/crates/bb_game/src/game.rs:204–205`, integrated with `sim/crates/bb_game/src/timeline.rs:133–146`.

On a dead-ball advance without live actor samples, the runner's track has one knot, now located at his lead. The award-jog builder projects the current position onto the base path only when `track.len() > 1`; otherwise it starts the jog at the old base. Linear playback therefore moves him backward throughout the pitch and pre-jog interval, then forward for the award.

**Executed scenario:** Seed **1**, play **23**, a forced walk advances runner 259 from first to second. His encoded root knots begin at **t=0**, `(15.508265, 23.286440)` (the 5.5 m lead), then **t=0.901150**, `(19.397354, 19.397354)` (first base), and finish at second at t=5.501150. Thus the viewer sees him retreat the full **5.5 m** before accepting the walk.

**Suggested fix:** Project the existing position for one-knot tracks too, and insert a stationary knot at the actual current position until the award jog starts. Keep this presentation correction outside rules/stat decisions.

### 5. P3 — The main acceleration specification omits the new runner override

**Location:** `docs/agent-models.md:366`, `540`; `sim/crates/bb_game/src/live.rs:48`, `1125`.

The calibration report correctly records runner tau **0.85 s**, but the main spec still describes baserunning through the grade-based `Acceleration` parameter and lists grade 50 as **1.00 s**. Live runners actually use a fixed 0.85 s, independent of that table. A reader implementing runners from the binding spec will reproduce the previous movement model.

**Suggested fix:** Document the interim 0.85 s runner override and 0.25 s batter start in the main movement specification, distinguishing it from the attribute table and fielder acceleration. No additional calibration is needed.

## Verification

- `export PATH=$HOME/.cargo/bin:$PATH; cd sim && cargo test --workspace --release`: **108 tests passed**, zero failed; doc tests also passed.
- Built the default-feature release CLI and ran `./target/release/sim-cli games --n 512 --seed 2`: **4.48 runs/team-game**, **17.4% K**, **7.4% BB**, **4.02% HR/PA**, **53.3% ground balls**, and `rules warnings: {}`. The initial link attempt could not write Zig's cache outside the sandbox; retrying with `ZIG_GLOBAL_CACHE_DIR` and `ZIG_LOCAL_CACHE_DIR` under `/tmp` succeeded.
- Built the release `calls` example and executed its binary for seeds **1–40**: **79 double plays**, **zero triple plays**. This uses the same example executable as `cargo run --release -p bb_game --example calls -- <seed>`.
- An independent external driver ran recorded games for seeds **1–256**: **18,406 balls in play**, **zero incomplete games**, **zero warnings** (including no unsettled-play/timeout warnings), and **21 appeal outs**. These successful batches do not disprove the geometric failures above.
- Six targeted external probe tests passed. They used a copy of the reviewed `live.rs` under `/tmp`, with probe-only access to private state; repository code remained untouched. A caught liner at first with a runner off his lead produces a fly out plus an **appeal** out, not a force. Existing legal-tag-up and retouch-once tests passed in the workspace suite. No stuck halfway runner, missed mandatory retouch, or illegal lead-related out was established in this focused review.
- **Backup recovery measurement:** Controlled A/B, seeds **1–40**, same current physics/recovery code, comparing old initial backup targets with the new assignments. Staged shortstop throw toward first aimed 3 m off in both horizontal axes; the receiver was deliberately unavailable for two seconds to ensure an overthrow. After four seconds of defensive positioning, mean release-to-recovery was **12.777 s old vs 11.414 s new**, an improvement of **1.363 s (10.7%)**. New assignments were faster in **39/40** cases; the worst new case was **16.600 s**, versus old maximum **12.967 s**. This demonstrates a controlled recovery benefit, not an end-to-end claim about natural overthrow frequency or all field situations.
- The named backup assignment tests pass. Covers are excluded from the free-candidate list, including a pitcher covering first. `backup_spot` is geometrically beyond the base on the actual throw line; I found no sign reversal placing its target in front. The receiver fallback and airborne-capture integration need the corrections above. Existing tests establish their named sample layouts, not every spray angle or later rotation.
- Calibration inspection/probe: `K_perception=2.7`, `aim_offset_z=-0.020`, `RUNNER_TAU=0.85`, `BATTER_START=0.25`. Discipline maps grades **20/50/80** to **(c,b)=(0.05,1.65)/(0.5,0.9)/(0.95,0.15)**, matching §10's grade columns. `Discipline::default()` exactly matches grade-50 `BatterGrades::apply`. Apart from finding 5, no inconsistency established for the requested calibration moves.
- `holder_act`: squared chance is a deterministic decision score, not an outcome roll. `runner_eta` uses observed speed and a typical maximum; outcomes still require physical catches/touches. No separate new makeable-play hold or throw-behind defect was established from this change. Finding 1 does prevent timely receiver positioning.
- **Client, inspection only:** Position deltas are divided by sim time; animation play rate multiplies by sim/wall time; pause sets rate to zero. Gait thresholds and the 14 m/s placement guard are internally consistent. The −90° yaw is consistent with the stated +Y mesh-forward convention. Missing any required mannequin/animation asset selects the cylinder fallback. Esc/Q reach `RequestExit(false)` independently of valid sim playback. No client build, asset-axis verification, or graphical animation test was performed.

## Three most jarring things in the timeline data

1. **The defense leaves home empty while a batter circles the bases.** Seed 14/play 53 above makes a throw-and-tag opportunity look like defenders have forgotten the scoring base.
2. **Runners move the wrong way before ordinary advances.** The contact retreat and the full-lead walk retreat above are both encoded motion, so locomotion animation faithfully shows the wrong baseball action.
3. **The batter slides from his box to the plate at contact.** Seed 1/play 7's encoded root moves from `(-0.95, 0.30)` at t=0.385163 to `(0,0)` at t=0.405163: **0.996 m in 20 ms, about 49.8 m/s**, then waits for the running start. This predates this range, so it is not a new-change finding. The join in `timeline.rs:100–110` connects the box stance to `live`'s home-origin runner. Start the physical running path from the batter's actual location and preserve his follow-through/start delay; suppressing gait on the jump cannot remove the position jump.

## Resolution

Resolved 2026-11-11. All five findings fixed, plus the third "jarring" item. `cargo test --workspace --release` passes.

| # | Status | What changed |
|---|---|---|
| 1 | Fixed | `cover_home_again` runs every tick: if nobody is covering home and any runner has rounded first, the catcher drops his backup job and goes home. Tested |
| 2 | Fixed | A base's backup may catch a throw once it has gone past the base (ball beyond the base along its direction of travel); defenders in front of the receiver still may not. Tested with the review's sailing throw to second |
| 3 | Fixed | Runners start in a hold at their lead and keep it until their first read of the ball; they no longer drift back toward the base for half a second. Tested |
| 4 | Fixed | The award jog starts from where the runner is (his lead) for runners who never moved too. A whole-game test now checks no batter or runner ever moves faster than 13 m/s between samples |
| 5 | Fixed | agent-models §5.3 notes the interim runner values (acceleration time constant 0.85 s, batter's start delay 0.25 s) |
| Viewer 3 | Fixed | The batter's track eases from his box to the base path over his first four metres, so he runs out of the box. The same no-jump test covers it |

Numbers after the fixes (`sim-cli games --n 512 --seed 2`): 4.32 runs/team-game, no rules warnings; 83 double plays and no triple plays in 40 games.
