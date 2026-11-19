# Task List

**Last updated:** 2026-11-18 23:20 (two Claude workers fixing review findings).

What is next and what is open. Other files hold the rest:

- [PLAN.md](PLAN.md): decisions and design.
- [observations.md](observations.md): the owner's checklist, in his words. Tick his items there.
- [docs/metrics-ledger.md](docs/metrics-ledger.md): the game's numbers and what moved them.
- [docs/archive/TASKS-history-2026-11.md](docs/archive/TASKS-history-2026-11.md): every earlier handoff, verbatim. History only; nothing in it is a task.

**Keeping this file short (2026-11-17):** a handoff *replaces* "Resume here"; it is not added above the old one. What a session finished goes in the commit message, the ledger and observations.md, not here. An item that is done is deleted, not ticked and kept. Keep the file under about 30 KB.

Status: `[ ]` not started · `[~]` partly done · `[?]` needs an owner decision

---

## Resume here (2026-11-18, 23:20)

**State (23:20).** Both branches were reviewed and both reviews are committed on their branches; a Claude worker is fixing each. Tab "Workers" is a 2x2 grid: left column the old Codex (`t2d`) and agy (`t2h`) workers, idle and no longer used (Claude workers only, owner 2026-11-18; close them at the merge); right column the two Claude workers. Read a pane with `hc read <terminal>`.

**1. `work/catch-read-fixes`** (worktree `.claude/worktrees/runner-decision`). The [catch-read review](docs/reviews/2026-11-18-codex-catch-read-review.md)'s six findings were fixed (`43bd29e..0b18e54`); the review of those fixes (committed `2ec56a8` on the branch) found 4 P1 (relooks still read the true contact point; a private pivot, an unpublished catch call and another runner's private goal still steer runners), 1 P2 (fast retouch duplicates the decision) and 1 P3 (send-rate cohort). **Opus worker `catch-fixes` in `t2q`** fixes them from untracked WORKER-FIXES2.md, and checks the adverse movement the review recorded (double-offs on 25–50° catches 16.5 → 38.0 per 300 games). The same review found the branch's main gain: sacrifice flies from third 3 → 204 over 600 games. When done: write the review's Resolution, skip a re-review only if the fixes are small and test-covered (else one Opus review in a pane), touch `live.rs`, run the tests and both standard runs yourself, merge.

**2. `work/saved-plays`** (Q30, worktree `.claude/worktrees/saved-plays`, client only). Review committed `c6d522c` on the branch: 3 P1 (save can store a different play than shown; a bad `.timeline` can play wrongly or hang; empty `-BbSavedPlay=` starts a live game), 4 P2 (stale build stamp; check bypasses the viewer's load path; today's roster and park can contradict the recording; board hit totals and outs). **Sonnet worker `saved-plays-fixes` in `t2r`** fixes P1, P2 and P3-1 from untracked WORKER-SAVED-PLAYS-FIXES.md. P3-2 (startup camera aimed at the plate, `36007e7`): **kept, owner Q32 (a), 2026-11-18**; say so in the review's Resolution. When done: write the Resolution, build the client and run `-BbSavedPlayCheck` yourself, merge.

**Docs check merged (2026-11-18):** run `python3 bin/check-docs.py` before committing doc changes; it fails on dead links, anchors and paths in the working docs and on this file passing 30 KB (`--all` also checks the dated records, which have 135 old dead references, left as they are).

**Owner's rulings, 2026-11-18** (recorded in PLAN §1): **Q29 (a)** a player may mix his own choices (pitch, spot); D1 reworded. **Q30 (b)** a saved play keeps the build's commit and the recorded play itself (D9). **Q31** Unreal decided outright (D6). D19/D20 now record the Q7 merge; D4 is covered by Q13/Q14; D17 waits for pitching changes in M6.

**After that:** item 1 of "Open sim work" below.

---

## Open sim work, in order

One `sim/crates/bb_game/src/live.rs` job at a time. Each job gets a ledger row and a review.

### 1. Runners (agent-models §6.3–§6.4; [decision-model note](docs/calibration/2026-11-15-runner-decision-model.md))

- [~] **The runner's start (Q27).** Moving secondary leads are merged (2026-11-18): +1.0 m at 3 m/s with 0–1 out, +1.5 m at 5 m/s with two out. All three bases use the contact state, and the live-ball timeline carries the shuffle from release. Early bounces respect the read clock. Runners brake for partway holds and returns, and the rules judge the retouch.
  - The contact-read review fixes are merged (2026-11-18); their own review's findings are the job in "Resume here". Runners use visible fielder motion, react to the public ruling, and judge thrown balls and holder readiness from observation. Their catch belief at the first look allows for their own read error, spin they cannot see and fielder speed. On a close catch they hold only as far off as they can get back from. On all singles, second to home is 56.7% (MLB 61.7%) and first to third is 33.3% (MLB 31.7%).
  - The 10–25° double-off jump on that branch (13.5 → 59.0 per 300 games) is fixed: 19.0 now, against Retrosheet's 25.7 by its line-drive label. Below 10° it is 59.5, mostly catches before the 0.5 s look (the fixed lead and coast, not the read). [Read error and return distance](docs/calibration/2026-11-17-runner-timing.md#read-error-and-return-distance).
  - Open: two-out sends to center are 117/118 (99.2%; MLB 75–91%), with a longer throw benchmark than at the corners. Fly balls with two or more outs are .006 [.005], about twice the base. A runner's return is a run that brakes onto the bag, with no dive back, so close holds lose races a real runner might win. The runner's catch-point estimate for the hold distance comes from one look; it can put the throw deeper than it is.
  - Deferred review P3s: the third-base route blend turns the lateral velocity in one tick (P3-1); the first decision sees one tick less coasting than its clock (P3-2). [Q27 timing and checks](docs/calibration/2026-11-17-runner-timing.md#final-measurements).
- [ ] Calibrate the runner's catch cut-offs (`RUNNER_CATCH_CONFIDENCE` 80%, advance below 20%) across base/out states and catch types, after the catch-read fixes (deferred catch-read review P3-1).
- [ ] Fit `AdvanceParams::caution_z` to MLB's attempt and success rates (now 1.0, fitted only to outs on the bases).
- [ ] The coach's read and send for a ball behind the runner (Q10 a). Model early-departure decisions on caught flies, slides around tags, a turn decision before the bag, and an overrun redirected after an overthrow.
- [ ] The rundown (Q11), on its own branch. A stand-in exists. Two small points from the runner-decision review wait for it.
- [ ] A runner stops dead standing at a base he holds, at 8 m/s (the sim brakes him only going back). A runner put out short of the bag leaves the picture, so there is no slide into a tag.
- [ ] Unexplained: doubles with a runner on first and two out nearly doubled after the decision model (88 → 167 in 300 games). Suspect: the scorer's single-or-double ruling when the batter takes second on the throw home. First to home on a double is 53% against 38%.
- [ ] Owner's saved plays 65, 67, 68, 156 and "round first on a throw to the cutoff" (observations.md): going halfway on a fly, running through first, rounding first.
- Waiting for ratings: per-runner caution and speed; τ refit to Statcast 90-ft splits (league top speed is 8.2 m/s; 27.0 ft/s is 8.23); the left/right home-to-first gap (0.17 s against 0.09 s).

### 2. Fielding

- [ ] Ground-ball hits .308 against .245.
- [ ] Revisit the geometric routine-play gate using catch-probability bins (calibration-targets §4.6).
- [ ] **Depth read after the first bounce.** A distant outfielder's snapshot depth along the line of sight has σ = `3.5 m × d/30 m × DEPTH_STRETCH` (3), about 27 m at 77 m on a first read. `off_bat()` ends the plate cue and tracking after the first bounce. On a rolling ball, the angle below the horizon constrains the distance. Specify that perception change in agent-models §5.1 before implementing it. See seed 65147, play 10.
- [ ] **The misplay on a ball still in the air** ([Fable review](docs/reviews/2026-11-17-fable-of-grounder-backup-review.md) P2-1). The outfield grounder rule (`outfielder_to_reconsider`) is shut while the ball is up and is looked at only when a call changes. In seed 43 play 74, all three outfielders leave a low fly that lands at (18, 57) and the second baseman goes; right field goes only at the 2.47 s recall and fields it at the wall. Suggested: open the gate for a ball in the air whose public first bounce is beyond `INFIELD_BREAK_R`, and look at it again when the ball comes down. Own branch, own ledger row.
- [ ] A fixture test for that rule's positive case (review P3-3): a hard grounder through the hole with a known infield chaser; exactly one outfielder, the first by `publicly_before`, chases through a second of hearings, and the other two never do.
- [ ] **Throw choice between near-certain options.** Seed 5 play 201: the first baseman, one metre from the bag with the ball, tosses to the pitcher. Two near-certain outs differ by more than the 1e-9 tie threshold (the carry preference in `sim/crates/bb_game/src/live.rs`), so tiny differences decide. Owner's saved play seed 47638 play 0 (no throw from first to the pitcher covering) goes with it.
- [ ] **"Seen him break"** (owner approved 2026-11-14; not built). A fielder sees a teammate start to move sooner than he hears his call. Move it in agent-models §5.7 from Proposed to decided; the time to notice is a named [E] parameter sourced from motion-onset data. Test case: seed 91689 play 5, where the centre and right fielders each pull up and the ball drops between them (`BB_DEBUG_COMM=1`, `cargo run -p bb_game --example trace_play -- 91689 5`).
- [ ] Dives not taken on routine fly misses (`OF_DIVE_MIN_P`): 27 of 61 before the track-bias fix; count again first.
- [ ] From the track-bias review: the shortstop's liner velocity is still overconfident (error²/var 3.9); very short grounders are read conservatively ([tracking note §6](docs/calibration/2026-11-17-track-bias.md#6-what-is-still-off)).
- [ ] No lost-ball state. The fly-hit deficit against matched catch-probability bins. An on-time caller displaced by a later teammate.
- [ ] A receiver on a bag has no stretch for a wide throw (owner's saved play 86).
- [ ] Owner's saved plays on throws and covering (observations.md): 56, 107, 142, 32 (set before throwing when there is time; trace how hurried each throw was before changing anything), 19, 52, 63, 159, 26, 68, 147.
- [ ] Not traced: infield hits with no throw rose 257 → 338 over 300 games after the infield alignment fixes; probably holders at 45–47.4 m who no longer make a return throw.

### 3. Throws and judgment (owner, 2026-11-15)

- [ ] **Outfield throws to the wrong base or past the cutoff.** Measure by play: throws to a base with no play while a trailing runner advances; cutoff man in line and missed. Then the model change. Includes the owner's "after an out from deep outfield, hit a cutoff".
- [ ] **Infield judgment on the double play:** when to go for two, by score, inning, outs, the runners' speed and the fielder's set-up (agent-models §5). Third basemen convert 48% of double-play grounders against a real 38%.

### 4. Batting and scoring

- Runs are about a quarter of a run low. Errors (A17), steals and wild pitches are not in the game.
- [ ] **Plate discipline, last measured 2026-11-14: measure again before acting.** Then: O-Contact 42.8% (band 45–68), swinging strikes 16.1% (8–15), Contact% 65.9% (70–83), Z-Contact 72.9% (79–91), pitches/PA 3.11 (3.5–4.3), chase 24.5% against 29%. The old owner decision O1 (chase rate) is this item; perceptual overconfidence was tried and backed out ([diagnosis](docs/calibration/2026-11-11-chase-diagnosis.md)).
- [ ] **The direction gap (known, unexplained;** physics-spec §8.2.1, ledger §5): spray minus attack direction sits about 12° toward the opposite field at every launch angle. Not yet compared band by band with the real figures in the [collision-targets note](docs/calibration/2026-11-14-collision-targets.md).
- [ ] Batters are 17 ms early on outside pitches and 2 ms on inside ones (perception). Swing v2's attack-angle slope against direction is 0.53 (Statcast 0.41).
- [ ] Not started: pitchers' zone targets and the tracking limit ([run-environment diagnosis](docs/calibration/2026-11-14-run-environment-diagnosis.md), ranks 4 and 5).
- Waiting for batter ratings (owner's idea, 2026-11-15): swing timing as part of the sweet spot, and per-batter timing, extension, plane and stance (agent-models §3.6).
- **Driveline OpenBiomechanics data is not to be used** (licence excludes commercial use).

### 5. Review findings still open

- [ ] [Codex live-play review](docs/reviews/2026-11-14-codex-live-play-review.md), P2s: a bounced or caromed throw can stay live after crossing into dead territory; a dive clipped at the wall still catches beyond it; the carry-to-second continuation still charges the pivot that carriers no longer pay; two wall contacts can reuse `SurfaceBounce` RNG streams; "first play" is reduced to "first throw with no recorded out"; the runner's tag allowance uses the force at the start, not at his destination; a pivot can choose 80% effort against the full-arm pivot contract; stopping walk-off awards drops the batter's required minimum advance. P3s: pivot and set-up gates gain a tick from the floating-point clock origin; the continuation regression test no longer shows the continuation changing a choice.
- Deferred in their Resolutions: tag geometry (the world tags 1 m short of the bag; a carry adds 0.25 s twice); the restored counterfactual in the 3/140 fixture; a runner oversliding a base (scorer P1-5 part 1); captions from the log; the client's dugout roof at 2.4 m against the sim's 4 ft; six older test gaps ([test-gaps review](docs/reviews/2026-11-14-codex-test-gaps-review.md)); the infield review's P3-4; three P3s of the [foul-at-the-bag review](docs/reviews/2026-11-16-agy-hydrogen-foul-at-bag-review.md).

### 6. Queued by owner rulings

- [ ] **Q13:** the sim's ground gets the mound's height, with the fielding work (it changes comebackers).
- [ ] **Q14:** the sim's dirt takes the drawn layout (skin, infield grass, warning track).
- [ ] **Q5 c,** with fielder ratings: each fielder judges the wall from his own sense of where he is, with a wall-awareness trait. Write it into agent-models as planned.
- **Held by the owner: the pitcher change** (`work/pitcher-fielding`). The pitcher starts from his follow-through and fields 8% of infield grounders, as in MLB, in place of 20%. It raised ground-ball hits sharply, so it stays off main until the middle infield is recalibrated. No review yet.

### 7. Older open items (from 2026-10-31 to 11-12; spot-checked against the code 2026-11-17)

Check the code again before starting one.

- [ ] Composure (leverage-scaled noise, agent-models §1.5): `sim/crates/bb_agents/src/motor.rs` takes the factor; nothing feeds it game state.
- [ ] Hit-by-pitch evasion (agent-models §3.7): let the batter move the body capsule away from the pitch.
- [ ] SpinCapacity has no effect; SpinRecognition is weak; QuickHands is flat between 30 and 50 ([M3 report, adaptation and sensitivity addendum](docs/calibration/2026-10-31-m3-interim.md#addendum-in-game-adaptation-and-sensitivity-later-the-same-night)).
- [ ] `Executed*` visibility isolation with a compile-fail test (none exists).
- [ ] `criterion` benchmarks for the per-pitch budget (not in any `Cargo.toml`; the timing harness is `sim/crates/bb_game/examples/perf.rs`).
- [ ] Rules corpus: 136 scenarios, toward 150 (rules-spec §12.3). Implement interference (rules-spec §5.7) and a separate first-fielder-touch fact for retouch timing on a deflected fly. Missed-base appeals and passing during awards are open (rules-spec §5.5–§5.6).
- [ ] Live play: double cuts and deliberate one-hop throws.
- [ ] **R8:** Check Savant `spin_axis` and `pfx_x` signs on real right- and left-handed four-seamers (data-formats §9.2–§9.3).
- [ ] **R9:** Source angular perception error, velocity Weber fraction and the near-plate blanking distance from interception and time-to-contact studies (agent-models §3.2–§3.3).
- [ ] **R10:** Refresh the calibration spec’s league reference values against the latest complete season and record the sources.
- [ ] **R11:** Audit OBR citations and interpretations, including three-batter minimum at inning boundaries, mound visits, the infielder definition on throw awards and walk-off base credit.
- [ ] Compare the flight coefficients to the current Trajectory Calculator spreadsheet, including total vs transverse spin in drag (physics-spec §4.2).
- [ ] Source the wood-bat COR by contact station and impact speed, baseball inertia factor, and turf/dirt bounce and rolling parameters (physics-spec §2, §8.2, §9.1).
- [ ] Validate the existing A10 undercut spin sweep against published measurements over its offset range (physics-spec §14.2).
- [ ] Add lighter/longer and heavier/end-loaded stock bats and revisit the typical 34-inch bat mass (physics-spec §6).
- [ ] Revisit tangential restitution between 34 and 65 m/s if direct measurements become available (physics-spec §8.2.1).
- [ ] Fit correlated release and swing execution noise if supported by measurements (release elevation with speed and swing timing with plane).
- [ ] Refit provisional agent estimates and elasticity gates as mechanisms and player ratings land (agent-models §10 and calibration-targets §7.2, §11).
- [ ] When fielding errors land (A17), audit ordinary-effort thresholds and the reference-fielder model against watched plays (rules-spec §10.1).
- [ ] Check first-base attempt and passed-infielder geometric thresholds against replay cases when their rulings are available (rules-spec §5.5, §5.7).
- [ ] Log runner passing with its own event/out kind so the scorer can show the actual ruling (rules-spec §5.5). Complete the schema-backed log producers for timeout detail, liveness and umpire signals, and deciding evidence for batted-ball rulings (rules-spec §15).
- [ ] Add infield fly and check the declaration time against runners’ hold behavior (rules-spec §13).
- [ ] Finish catcher receiving/blocking, uncaught third strikes, and caught foul tips (agent-models §4, rules-spec §4.4), plus bunts and balks (rules-spec §8, §13).
- [ ] Validate the dead-ball cut on a throw into the seats and a ground-rule double (physics-spec §9.5).
- [ ] Fill in depth anticipation and cut-off throws to hold a trailing runner when those models are specified (agent-models §5.10, §6.4).
- [ ] Add the real-sample Savant export and schema compatibility fixtures; wire up the machine-readable calibration bands, regression reports and gate runs (data-formats §8, §9.4 and calibration-targets §12).

---

## Owner backlog

[observations.md](observations.md) is the list and its boxes are the status. Only detail that is not there is kept here. His priorities are A, B, C; `[W..]` are the old wishlist numbers.

**Not seen moving by the owner yet** (built and checked in stills only): the delivery and the catcher, the swing, catches and throws in the field, the slide and the dive, the umpires, coaches and benches, and the HUD's panels. His first watch is the check on how they flow.

**His to overrule if he likes** (each is recorded in [park-visuals](docs/park-visuals.md), [player-visuals](docs/player-visuals.md) or [hud](docs/hud.md)): the clubs and park names, uniform cuts and colours, wall padding and seat colours, attendance, the delivery from the stretch on every pitch, the stride at every pitch, slides only with a play on the runner and always feet-first, the umpires' pale blue. Ball size is settled at 11.7 cm (`BallDrawnScale`); do not add an on-screen size floor back without asking.

### Priority A, not done

- [~] **A4. Throw choices from the outs and the runners.** Built with review fixes; more cases are items 2 and 3 above.
- [~] **A5. Dives.** Rate and motion are in. Left: every dive is head-first straight at the ball.
- [~] **A6. Slides.** Client only: a runner slides feet-first when a play is being made on him. The sim has no slide (agent-models §6.2); see item 1 above for the dead stop.
- [ ] **A7. Windup and stretch** [W2]: a windup with the bases empty, the stretch with runners on, which shortens the delivery and matters to steals. One delivery time today (agent-models §2.6). References are in observations.md.
- [ ] **A8. Leads only once the pitcher is set; pickoffs.** Fixed leads today (`LEAD_M` in `sim/crates/bb_game/src/live.rs`). Missing: lead timing tied to the pitcher coming set, leads from the runner's belief of pickoff risk, the pickoff (agent-models §2.6, §6.1).
- [ ] **A9. Stolen bases** [W13]. Needs A7, A8, A6 and the catcher's throw (agent-models §4).
- [ ] **A11. Wind and weather.** The flight model takes a wind vector (always zero) and an air density. Rain must act through physical causes (D1): a wet ball, wet ground; never a mistake rate.
- [ ] **A17. Fielding errors** [W10], from physical causes only (a bobble, a bad throw; D1). Ties to the scorer's hit-or-error ruling (rules-spec §10). Do not use the read's noise as a stand-in for errors (owner, 2026-11-15).
- [ ] The base map in the score bug follows the real runners during a play.
- [ ] Owner's throwing notes A–D in observations.md: plant before a throw on the run, a penalty across the body, how far outfield throws scatter (unchecked against real throws).

**Agreed order for what is left** (owner, 2026-11-13): A17 errors; then A8 → A7 → A6 → A9 (leads and pickoffs, the stretch, slides, steals); then A11, wind first.

### Priority B and C

- [ ] B1. Leagues, divisions, teams with rosters [W5]. M7. (Two invented clubs exist for the look only.) Use roster height and build for the client figures when ratings arrive (Q18).
- [ ] B2. Lineups by pitcher hand (a manager lever, M6).
- [ ] B3. Game settings screen, with team selection [W6].
- [~] B4. Player and team statistics [W11]: box score and batting lines exist; no statistics screens.
- [ ] B5. "Fielder's choice" is used too freely (`classify_bip` in `sim/crates/bb_game/src/game.rs`, after OBR 9.05). Audit a few hundred such plays, fix with the errors work, and make the text say what happened.
- [ ] C1. Better crowds: today's sits still. C2. Player development and aging [W7], M7. C3. Post-game highlights (disk retention of timelines, D9).

### Looks, later

- [ ] L3. Batter stances and swings per player [W3]. The owner's examples: Pete Rose, Darryl Strawberry, Gary Carter.
- [ ] Names that follow nationality; the catcher tossing his mask on a pop foul (saved play 40); a tag shown being made; the first baseman's "I got it" (159); a runner's athletic stance (65).
- [ ] Park polish: night lighting, per-team colours, moving flags, aisle/front railings, pole-end gaps, lattice towers, turf grain and sharper distant shadows, more detailed trees/sign lettering, and crowd standing, concourse motion and reactions.
- [ ] Figure and kit polish: shirt hem, finger and glove shapes, catcher’s cage mask, first-base mitt, batting gloves and shin guard.
- [ ] Pitch/batting staging: pitcher’s target and signs, catcher’s return from the crouch, foot placement after motions, carrying and dropping the bat, grip, swing recovery, and distinct check-swing and bunt poses.
- [ ] Fielding and running visuals: one visible jump/catch height matched to the sim’s track, throwing on the run, catch transitions and footwork, receiving foot on the bag, relay-ready poses, head-first and return slides, slide recovery, and dive variants and head turn. A receiver stretch and tags have tasks above.
- [ ] Crew polish: coach and bench transitions and reactions, umpire movement, fair/time/infield-fly/home-run and third-strike signals, safe-call coverage and matching the result card’s timing. Recheck crew cost with frame-rate measurements if the figure budget changes.
- [ ] Score displays: an "X" for the unplayed home half, errors in the line score, and full pitching lines via read-only exports.
- [ ] Packaged client materials need cooking and validation (park-visuals §2).

---

## Later milestones

- [ ] **M5** Deepen the field (much of it is under way above)
- [ ] **M6** Live management (the primary experience)
- [ ] M6 policy and calibration: manager personalities, full tactic set, fatigue and bullpen use bands; define elapsed-time gaps for warm-up and fatigue (data-formats §2.2).
- [ ] M6 explanations and replay: implement belief snapshots and explanation facts, then settle highlight retention and fact importance (data-formats §5.1, §5.5–§5.6).
- [ ] **M7** League and season. Generate more parks and add a run-factor check (0.85–1.15) for each.
- [ ] **M8** Presentation polish. Revisit the RAM upgrade before this.


Exit gates are in [PLAN.md §14](PLAN.md#14-roadmap).

---

## Parked (decided "later")

- [ ] **Foul share: the spray experiment** (owner, 2026-11-13). **Written before swing v2 replaced the swing (2026-11-14): measure the foul share again before anything else.** Then it was 23.7% of contact against MLB's about 51%. Read [the foul-share note, sections 5–8](docs/calibration/2026-11-13-foul-measures.md#5-what-this-means) and its three Codex reviews first.
  - Ruled out or weak then: well-timed steep contact; the fair/foul call; mid-swing correction (23.7 → 26.1% even with no limits); bat angle alone.
  - The open hypothesis then: the swing holds the ball's spray too close to its attack direction (MLB contact at attack direction 12–20° fouls 54%, the sim's 8–12%). This is close to the direction gap in item 4 above.
  - Second question: on-time topped balls stay fair (MLB fouls 62% of well-timed contact below −30°); needs evidence before any Tier A change.
  - Tools: `foul_diag`, `foul_in_games`, `sim-cli cage-map`.
- Injuries, physically grounded (D18), after M8.
- Optional later models: perceived leverage errors, strict ABS parity if requested, and DH-loss substitutions and two-way players (rules-spec §9.4, §13).
