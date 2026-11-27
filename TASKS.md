# Task List

**Last updated:** 2026-11-27 (body-motion session 12 running).

What is next and what is open. Other files hold the rest:

- [PLAN.md](PLAN.md): decisions and design.
- [observations.md](observations.md): the owner's checklist, in his words. Tick his items there.
- [docs/metrics-ledger.md](docs/metrics-ledger.md): the game's numbers and what moved them.
- [docs/archive/TASKS-history-2026-11.md](docs/archive/TASKS-history-2026-11.md): every earlier handoff, verbatim. History only; nothing in it is a task.

**Keeping this file short (2026-11-17):** a handoff *replaces* "Resume here"; it is not added above the old one. What a session finished goes in the commit message, the ledger and observations.md, not here. An item that is done is deleted, not ticked and kept. Keep the file under about 30 KB.

Status: `[ ]` not started · `[~]` partly done · `[?]` needs an owner decision

---

## Resume here (2026-11-27)

**State.** PLAN D31–D44: a force-driven body; PLAN §14.1, spec [docs/body-motion.md](docs/body-motion.md).

**Sessions 8–12 are merged** (spec §4.10–§4.14): 26.25 m/s (58.7 mph; the gate asks 29). **D42:** on while each session gains honest bat speed. Session 12 gained.

**Owner, 2026-11-27: not a baseball swing; searches cancelled** until he rules: Q66 (s13 review), Q67.

**Running** (restart if stopped): session 13, `body-3d-13` (Fable), worktree `swing-research`, `work/body-3d`, brief WORKER-BODY-3D-13, changed by message: it only looks at the swing and proposes. Then: the owner. **The swing in the viewer (D43):** `bin/body-swing.sh`, player-visuals §14. Deferred: its tests skip on main (no [4] curves here).

**Next:** §2: the ground-dive leftovers ([note](docs/calibration/2026-11-23-of-ground-dive.md) §9).

**Open (ledger §5):** runs down 0.21 [0.18] since dp-cover (0.04 explained; check noise); double plays 12.3% (MLB 9.8%).

---

## Open sim work, in order

One `sim/crates/bb_game/src/live.rs` job at a time. Each job gets a ledger row and a review.

### 1. Runners (agent-models §6.3–§6.4; [decision-model note](docs/calibration/2026-11-15-runner-decision-model.md))

- [~] **The runner's start (Q27).** Moving secondary leads are merged (2026-11-18): +1.0 m at 3 m/s with 0–1 out, +1.5 m at 5 m/s with two out. All three bases use the contact state, and the live-ball timeline carries the shuffle from release. Early bounces respect the read clock. Runners brake for partway holds and returns, and the rules judge the retouch.
  - After fly-read (2026-11-19): double-offs per 300 games 0 at 25–50° (MLB about 4: each in-flight relook's noise is fresh and closes as the ball comes down, so a runner always sees the catch coming; deferred review P3-5, levers in the [review](docs/reviews/2026-11-19-opus-fly-read-review.md)), 12.5 at 10–25°, 60.5 below 10°. On singles second to home is 55.1% (MLB 61.7%), first to third 31.2% (31.7%). Long liners still up after 3–4 s read 20–40 m off flown back from the plate cue (70 balls in 48 games; tracking corrects it). [Fly-read note](docs/calibration/2026-11-19-fly-read.md#open).
  - Open, diagnosed 2026-11-23 ([note](docs/calibration/2026-11-23-runner-start-open.md)): two-out sends: Q38 (merged). Dive back: Q39 (merged, below). Late-flight read and tag-up relook merged ([note](docs/calibration/2026-11-23-late-catch-read.md); open: its §6): second to home, 0–1 out, 43.1% → 46.0% (MLB 49.1%).
  - Deferred P3s left (runner-start note §5): runner-timing P3-1, P3-2.
- [ ] Base relook (§6.4; ledger row 2026-11-23): his look at a throw 0.5 s in misplaces it along his line of sight (σ capped only by the distance flown; the one out in 600 games). `RELAY_READ_S` 0.6 s vs the 0.28–0.47 s rushed return. Held runners start from rest.
- [ ] Batter-runner stops at first on 97/279 double cuts, some fielded 120 m out (64/196; main too; Opus review Q5).
- [ ] **The runner's read at third of a charging outfielder's throw home is ~0.2 s slow** ([note §2a](docs/calibration/2026-11-23-two-out-send.md#2a-the-runners-read-of-the-throw-home-review-p2-1)): he times it from the fielder's spot at his look (he lets go 2–3 m nearer) to the plate. If fixed, refit `send_two_out_z`. `runner_timing`'s benchmark is 0.28 s slow. All 7 corner outs at right.
- [ ] Deferred from Q39 ([note §6](docs/calibration/2026-11-23-dive-back.md)): a fielder stands in the lying runner.
- [ ] Retouch belief pessimistic: 8 retouches in 600 games he thinks lost are won diving (note §6).
- [ ] **Outs going back on grounders: 72 per 600 games, MLB about 3** (note §3–§4): 51 from second, where MLB's are out at third. Likely the missing rundown (Q11); count the sim's outs at third first.
- [ ] The coach's read and send for a ball behind the runner (Q10 a). Model early-departure decisions on caught flies, slides around tags, a turn decision before the bag, and an overrun redirected after an overthrow.
- [ ] The rundown (Q11), on its own branch. A stand-in exists. Two small points from the runner-decision review wait for it.
- [ ] A runner stops dead standing at a base he holds, at 8 m/s (the sim brakes him only going back). A runner put out short of the bag leaves the picture, so there is no slide into a tag.
- [ ] Unexplained: doubles with a runner on first and two out nearly doubled after the decision model (88 → 167 in 300 games). Suspect: the scorer's single-or-double ruling when the batter takes second on the throw home. First to home on a double is 77% against MLB 38% (65 of 84, seeds 0–99, `extra_base_rates`, 2026-11-19; the 53% quoted before dated from 2026-11-15). 2026-11-23: 61/67/92% at 0/1/2 out against Retrosheet 31/35/59% ([note §5](docs/calibration/2026-11-23-two-out-send.md#5-open)).
- [ ] Owner's saved plays 65, 67, 68, 156 and "round first on a throw to the cutoff" (observations.md): going halfway on a fly, running through first, rounding first.
- Waiting for ratings: per-runner caution and speed; τ refit to Statcast 90-ft splits (league top speed is 8.2 m/s; 27.0 ft/s is 8.23); the left/right home-to-first gap (0.17 s against 0.09 s).

### 2. Fielding

- [ ] Ground-ball hits .308 against .245.
- [ ] **Approach follow-ups** ([approach note](docs/calibration/2026-11-20-approach.md) §7; [review](docs/reviews/2026-11-20-opus-approach-review.md)): outfield pickup-to-release 1.05 → 1.12 s from the honest braked take (can he brake at full reach when the planned speed is well above a stop?); first to third on singles 36.4% against 31.7% (Q32's across charge; the crow hop in the plan and the angle from contact did not move it); the pop-up hit rise is untraced; `outfield_round_base` ignores the runner's direction and the tag time (review P3-2); two tests partly restate their formula (P3-5); the typical-spin test's comment quotes probe medians (875/919/901) where the test itself reads 962 at 15°/105 mph, the cell to refit first.
- [ ] A fielder who misses a catch stands still beside the dead ball for about a second before picking it up (approach review P3-7; seed 207 play 66; on main before the approach too; maybe the once-a-second recall).
- [ ] Revisit the geometric routine-play gate using catch-probability bins (calibration-targets §4.6).
- [ ] **Depth read follow-ups** ([note](docs/calibration/2026-11-24-depth-read.md) §7): the cue in the track's looks, the call's doubt, the runners' reads, a ball off a wall in the air; eye-height bob.
- [~] **Play 71 (seed 51001)** ([note](docs/calibration/2026-11-23-of-ground-dive.md)): he dives now and misses by 1.7 cm (a triple). Left: note §9.
- [ ] The rim's push is in fixed world axes (always toward home), not §5.4's collision; it slows outfield knock-downs ([review](docs/reviews/2026-11-23-rev-of-dive-of-ground-dive-review.md) P3-3).
- [ ] **Bounce-track review, deferred** ([review](docs/reviews/2026-11-21-opus-bounce-track-review.md)). P3-1: `BOUNCE_SPIN_RPM` 1,000 rpm is unsourced and small for mis-hits (topspin read as backspin); source it from the contact model's spin spread about the typical-spin table (`examples/batted_spin.rs`), then recount the hop let-gos. P3-3: outfielders' error²/σ² about 1.4–1.6 after the bounce (the in-flight read's own over-confidence), and over 40% of their errors grow just after the bounce though the RMS falls. P3-4: the roll's â drift sits on both level axes though only its part along the roll moves the ball (about 0.3 m of σ across over 2 s). Pre-existing: a snapshot read of a ball at rest against the wall can put it beyond the wall.
- [ ] **The misplay on a ball still in the air** ([Fable review](docs/reviews/2026-11-17-fable-of-grounder-backup-review.md) P2-1). The outfield grounder rule (`outfielder_to_reconsider`) is shut while the ball is up and is looked at only when a call changes. In seed 43 play 74, all three outfielders leave a low fly that lands at (18, 57) and the second baseman goes; right field goes only at the 2.47 s recall and fields it at the wall. Suggested: open the gate for a ball in the air whose public first bounce is beyond `INFIELD_BREAK_R`, and look at it again when the ball comes down. Own branch, own ledger row.
- [ ] A fixture test for that rule's positive case (review P3-3): a hard grounder through the hole with a known infield chaser; exactly one outfielder, the first by `publicly_before`, chases through a second of hearings, and the other two never do.
- [ ] **Throw choice between near-certain options.** Seed 5 play 201: the first baseman, one metre from the bag with the ball, tosses to the pitcher. Two near-certain outs differ by more than the 1e-9 tie threshold (the carry preference in `sim/crates/bb_game/src/live.rs`), so tiny differences decide. Owner's saved play seed 47638 play 0 (no throw from first to the pitcher covering) goes with it.
- [ ] **"Seen him break"** (owner approved 2026-11-14; not built). A fielder sees a teammate start to move sooner than he hears his call. Move it in agent-models §5.7 from Proposed to decided; the time to notice is a named [E] parameter sourced from motion-onset data. Test case: seed 91689 play 5, where the centre and right fielders each pull up and the ball drops between them (`BB_DEBUG_COMM=1`, `cargo run -p bb_game --example trace_play -- 91689 5`).
- [ ] Dives not taken on routine fly misses (`OF_DIVE_MIN_P`): 27 of 61 before the track-bias fix; count again first.
- [ ] From the track-bias review: the shortstop's liner velocity is still overconfident (error²/var 3.9); very short grounders are read conservatively ([tracking note §6](docs/calibration/2026-11-17-track-bias.md#6-what-is-still-off)).
- [ ] No lost-ball state. The fly-hit deficit against matched catch-probability bins. An on-time caller displaced by a later teammate.
- [ ] A receiver on a bag has no stretch for a wide throw (owner's saved plays 86, 5593/3).
- [ ] Owner's seed-5593 plays (observations.md): 52, fielders stop too quickly converging on a fly; 56, the third baseman neither tags nor throws to second, then throws to first. Client: 18, no ball shadow on a pitch.
- [ ] Owner's saved plays on throws and covering (observations.md): 56, 107, 142, 32 (set before throwing when there is time; trace how hurried each throw was before changing anything), 19, 52, 63, 159, 26, 68, 147.
- [ ] Not traced: infield hits with no throw rose 257 → 338 over 300 games after the infield alignment fixes; probably holders at 45–47.4 m who no longer make a return throw.

### 3. Throws and judgment (owner, 2026-11-15)

- [ ] **Who goes out as relay man depends on the outfielder's arm** (catch-throw re-review P3-2, agent-models §5.7): the relay man goes out past a fixed 80 m (grade 50's crossover); for a strong arm the throw through the cutoff man would be 0.1–0.2 s quicker. Decide from the arm (known by scouting) with the crossover tables. Also (re-review P3-3): the cutoff man, sent when the ball is down, is still on his way at the release on quick plays.
- [ ] **A base receiver's catch-and-throw is unsourced** (relay review P3-3): 0.7 × `Transfer` with the hurry, as quick as 0.28 s (27 of 71 receivers hurried per 100 games). Statcast's catcher exchange (mean 0.73 s, elite 0.64 s) is the nearest analogue; use it to replace `RELAY_TRANSFER_FRAC` and its hurry.
- [ ] **Outfield throws to the wrong base or past the cutoff.** Measure by play: throws to a base with no play while a trailing runner advances; cutoff man in line and missed. Includes the owner's "after an out from deep outfield, hit a cutoff" and the cut-or-let-it-go call (Q32, agent-models §5.5).
- [ ] **Double plays after `work/dp-cover`: what the review left** ([review](docs/reviews/2026-11-21-opus-dp-cover-review.md); [dp-cover note](docs/calibration/2026-11-21-dp-cover.md) §4). (a) P2-2, pre-existing: a carrier to second (6U-3, 4U-3) is estimated as a pivot man (touch, then `pivot_s` 0.80 s, a turn and plant), but the world lets him throw on the tick after the touch with the set-up he spent carrying. The ball at first comes 0.09–0.51 s sooner than estimated; the force at second 0.20–0.23 s later than `carry_ball_at`. Fix both to one model (a turn and plant from his motion at the touch, `exchange_footwork_s(fd.vel, to_first)`), with a frozen 4U-3. (b) P3-3: the cover's pivot clock starts at the catch (world: the touch, 0.1–0.2 s later) and assumes full arm. (c) First-baseman-started double plays 35.9% (MLB 21.5%): the 3-6-3 pivot costs nothing; no slide (`PIVOT_SLIDE_DELAY_S` = 0, A6).
- [ ] **Middle infielders with no job while a ground single is loose** ([note](docs/calibration/2026-11-21-mid-infield-jobs.md) §3). (a) A ball in the air not taken for a single (deep, or catchable by their estimate): no job the first call's 0.3 s ([liner note](docs/calibration/2026-11-21-liner-single-jobs.md) §5). (b) A ball another infielder could field: the ball-side man waits. (c) No job at the call on a chopper (51001/168). (f) Comebackers, nobody on: both middle infielders away 0.1–0.2 s (5 in 120 games; on 2 the 2B leaves second for the cutoff, 24/54, 32/107).
- [ ] **Double-cut leftovers** ([note §6–§7](docs/calibration/2026-11-22-double-cut.md#7-review-fixes-2026-11-22)): the relay man flips between middle infielders up the middle (10/38, 71/185); a man a throw passes heads for a base taken <0.3 s before the release, two on it till the catch (9 in 120 games). Deferred ([Opus](docs/reviews/2026-11-22-opus-double-cut-review.md), [Codex](docs/reviews/2026-11-22-codex-double-cut-review.md)): a relay throw landing short ends the first baseman's job (O P3-3); the convention counts a relay man his stale intercept vetoes (C P3-1, constructed); `relay_base` takes the outfielder's thresholds for all (matters once per-player).
- [ ] **A single with a runner on third** (relay hand-overs note §5): the runner from third scores, so the standard throw goes to third (runner on first, shortstop cutting) or second; the game lines up for home until he scores.
- [ ] **Relay leftovers** (relay hand-overs note §5): two men on one bag >0.5 s on 3 plays from item 6's gate (39/145, 95/176, 111/74); a corner man out before any infielder goes for the ball (117/38); the pitcher follows the third baseman between the cutoff home and third (93/183).
- [ ] **Do fielders or runners start moving before contact on screen?** The camera cuts after contact (saved play 112); nobody checked the timeline's fielders and runners against `contact_s` (sim side).
- [ ] **Infield judgment on the double play:** when to go for two, by score, inning, outs, runner speed and set-up (agent-models §5). Third basemen convert 48% of DP grounders (real 38%).

### 4. Batting and scoring

- Runs are about a quarter of a run low. Errors (A17), steals and wild pitches are not in the game.
- [ ] **Plate discipline, last measured 2026-11-14: measure again before acting.** Then: O-Contact 42.8% (band 45–68), swinging strikes 16.1% (8–15), Contact% 65.9% (70–83), Z-Contact 72.9% (79–91), pitches/PA 3.11 (3.5–4.3), chase 24.5% against 29%. The old owner decision O1 (chase rate) is this item; perceptual overconfidence was tried and backed out ([diagnosis](docs/calibration/2026-11-11-chase-diagnosis.md)).
- [ ] **The direction gap (known, unexplained;** physics-spec §8.2.1, ledger §5): spray minus attack direction sits about 12° toward the opposite field at every launch angle. Not yet compared band by band with the real figures in the [collision-targets note](docs/calibration/2026-11-14-collision-targets.md).
- [ ] Batters are 17 ms early on outside pitches and 2 ms on inside ones (perception). Swing v2's attack-angle slope against direction is 0.53 (Statcast 0.41).
- [ ] Not started: pitchers' zone targets and the tracking limit ([run-environment diagnosis](docs/calibration/2026-11-14-run-environment-diagnosis.md), ranks 4 and 5).
- Waiting for batter ratings (owner's idea, 2026-11-15): swing timing as part of the sweet spot, and per-batter timing, extension, plane and stance (agent-models §3.6).
- **Driveline OpenBiomechanics data is not to be used** (licence excludes commercial use).

### 5. Review findings still open

- [ ] A structured `foul` flag in `BbPlayInfo` (`bb_ffi`, sim side) to replace the client's match on the summary text "Foul" (`BbBaseMap::IsFoul`, used by the base map and the umpire's foul signal). Sonnet base-map review P2-2.
- [ ] [Codex live-play review](docs/reviews/2026-11-14-codex-live-play-review.md), P2s: a caromed throw can stay live past dead territory; a dive clipped at the wall catches beyond it; the carry continuation charges a pivot carriers no longer pay; two wall contacts can reuse `SurfaceBounce` streams; "first play" is "first throw with no recorded out"; the tag allowance uses the start's force; a pivot can choose 80% effort; walk-off awards drop the batter's minimum advance. P3s: gates gain a tick from the clock origin; the continuation test no longer shows a changed choice.
- Deferred in their Resolutions: tag geometry (the world tags 1 m short of the bag; a carry adds 0.25 s twice); the restored counterfactual in the 3/140 fixture; a runner oversliding a base (scorer P1-5 part 1); captions from the log; the client's dugout roof at 2.4 m against the sim's 4 ft; six older test gaps ([test-gaps review](docs/reviews/2026-11-14-codex-test-gaps-review.md)); the infield review's P3-4; three P3s of the [foul-at-the-bag review](docs/reviews/2026-11-16-agy-hydrogen-foul-at-bag-review.md).

### 6. Queued by owner rulings

- [ ] **Q13:** the sim's ground gets the mound's height, with the fielding work (it changes comebackers).
- [ ] **Q14:** the sim's dirt takes the drawn layout (skin, infield grass, warning track).
- [ ] **Q5 c,** with fielder ratings: each fielder judges the wall from his own sense of where he is, with a wall-awareness trait. Write it into agent-models as planned.
- **Held by the owner: the pitcher change** (`work/pitcher-fielding`). The pitcher starts from his follow-through and fields 8% of infield grounders, as in MLB, in place of 20%. It raised ground-ball hits sharply, so it stays off main until the middle infield is recalibrated. No review yet.

### 7. Older open items (from 2026-10-31 to 11-12; spot-checked against the code 2026-11-23)

Check the code again before starting one.

- [ ] Composure (leverage-scaled noise, agent-models §1.5): `sim/crates/bb_agents/src/motor.rs` takes the factor; nothing feeds it game state.
- [ ] Hit-by-pitch evasion (agent-models §3.7): let the batter move the body capsule away from the pitch.
- [ ] SpinCapacity has no effect; SpinRecognition is weak; QuickHands is flat between 30 and 50 ([M3 report, adaptation and sensitivity addendum](docs/calibration/2026-10-31-m3-interim.md#addendum-in-game-adaptation-and-sensitivity-later-the-same-night)).
- [ ] `criterion` benchmarks for the per-pitch budget (not in any `Cargo.toml`; the timing harness is `sim/crates/bb_game/examples/perf.rs`).
- [ ] Rules corpus: 136 scenarios, toward 150 (rules-spec §12.3). Implement interference (rules-spec §5.7) and a separate first-fielder-touch fact for retouch timing on a deflected fly. Missed-base appeals and passing during awards are open (rules-spec §5.5–§5.6).
- [ ] Live play: deliberate one-hop throws (the double cut is built, 2026-11-22).
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

- [~] **Batting stances.** Base swing: [player-visuals §13](docs/player-visuals.md#13-the-base-stance-and-swing-design-2026-11-23-step-1-of-5-built); per-player looks after (D23). [x] 1 bat/E1 (Q44: -BbBatSim) · [ ] 2 legs · [ ] 3 hips, take · [ ] 4 finish, Q43 · [ ] 5 late
- [~] **A4. Throw choices from the outs and the runners.** Built with review fixes; more cases are items 2 and 3 above.
- [~] **A5. Dives.** Rate and motion are in. Left: every dive is head-first straight at the ball.
- [~] **A6. Slides.** Client only: a runner slides feet-first when a play is being made on him. The sim has no slide (agent-models §6.2); see item 1 above for the dead stop.
- [ ] **A7. Windup and stretch** [W2]: a windup with the bases empty, the stretch with runners on, which shortens the delivery and matters to steals. One delivery time today (agent-models §2.6). References are in observations.md.
- [ ] **A8. Leads only once the pitcher is set; pickoffs.** Fixed leads today (`LEAD_M` in `sim/crates/bb_game/src/live.rs`). Missing: lead timing tied to the pitcher coming set, leads from the runner's belief of pickoff risk, the pickoff (agent-models §2.6, §6.1).
- [ ] **A9. Stolen bases** [W13]. Needs A7, A8, A6 and the catcher's throw (agent-models §4).
- [ ] **A11. Wind and weather.** The flight model takes a wind vector (always zero) and an air density. Rain must act through physical causes (D1): a wet ball, wet ground; never a mistake rate.
- [ ] **A17. Fielding errors** [W10], from physical causes only (a bobble, a bad throw; D1). Ties to the scorer's hit-or-error ruling (rules-spec §10). Do not use the read's noise as a stand-in for errors (owner, 2026-11-15).
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
- [ ] M6 learned tables: add a best-answer test ([note](docs/research/cogentic/README.md) §2).
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
