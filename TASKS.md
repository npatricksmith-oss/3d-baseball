# Task List

**Last updated:** 2026-11-12 (third session)
The live to-do list for the project. [PLAN.md](PLAN.md) holds decisions and design. This file holds what is next and what is done. Update it at the end of every work session.

Status: `[ ]` not started · `[~]` in progress · `[x]` done · `[?]` needs an owner decision

---

## Resume here (2026-11-12, third session: umpire calls)

**First:** the [Codex review of the Anticipation commit](docs/reviews/2026-11-12-codex-anticipation-review.md) is finished but uncommitted and unresolved (see the second-session handoff below). The [Codex review of the umpire calls](docs/reviews/2026-11-12-codex-umpire-calls-review.md) is resolved: two client scoreboard-timing bugs fixed (strike three showed before the call; the third out was skipped straight to the next half).

Done (owner request: "when an out is made the umpire should make a call right away; within 1 s for non-close plays, almost right away when close"):
- **Umpire out calls** (rules-spec §1.3, data-formats §5.4): every out gets an `UmpireCall` timeline marker at the moment the umpire signals it: 0.2 s after a bang-bang out (runner within 0.15 s of the bag, a diving or shoestring catch, strike three), rising to 0.7 s on routine plays. The client shows "OUT at first" / "OUT!, caught" (red and larger when close) as it happens and advances the outs on the scoreboard; the play's summary still appears when the clip ends. `bb_game::umpire`; `BbPlayInfo.calls`; `cargo run --release -p bb_game --example umpire_calls -- 20`. Observation only: game results are unchanged.
- Not done: no umpire figures on the field (the call is HUD text); safe calls on close plays are not signalled.

## Previous handoff (2026-11-12, second session)

**First:** the [Codex review of the Anticipation commit](docs/reviews/2026-11-12-codex-anticipation-review.md) (`74679a6..cdd473c`) was started at the end of the session. If it is in `docs/reviews/`, commit it, fix or defer each finding, and write its Resolution. Its first question: whether reading the swing from the true launch direction plus noise is honest perception (the alternative is reading the bat's yaw from the swing record).

Done this session (owner request: fielders "get a jump" by anticipating from pitch location and batter handedness, with a rating):
- **Fielder momentum** (`21a4535`, agent-models §5.3): velocity is a vector; braking at most 6 m/s², turning at most a_lat 7 m/s²; fielders slow into turns they cannot make. Until now they turned and reversed instantly at full speed, which hid six planner bugs, now fixed: infielders broke for pop-ups as if grounders; a breaking infielder ran at his base; fly balls went to the earliest intercept (a coin toss); the re-call looked near the ball's current position; calls were traded on noise; fielders assumed 1,800 rpm backspin where the contact model gives flies 3,300–6,500 (now a table by launch angle). Refit: velocity read 0.06 → 0.04, OF read 2.5 → 3.0. [Report](docs/calibration/2026-11-12-fielder-momentum.md).
- **[Codex momentum review](docs/reviews/2026-11-12-codex-momentum-review.md): resolved** (`74679a6`): arrivals no longer stop a fielder dead, planting distance, a turn-aware reach estimate (tested against the stepped movement), receivers not pinned to the bag, fresh-read breaks, wall caroms.
- **Anticipation** (`cdd473c`, agent-models §5.10): the fielder combines the pitch location and batter's side (a prior) with a read of the swing at contact, and starts toward the side he expects at contact instead of standing through his reaction time. Leaning on location alone made fielders *worse* (7° of signal against 22° of scatter); the swing read is what pays. 2,048 games: no jump 5.35 runs/team-game, grade 20 5.31, 50 5.22, 80 5.03. `sim-cli games --anticipation <grade>` / `--no-jump`.

Next, in the order a viewer would notice:
- **Watch it in the client** (`bin/play.sh`): fielders now carry momentum, plant after catches, curve into turns, and move at contact. Nothing in the Unreal client changed; check it still looks right.
- **Batters of both sides in games.** Every game batter is right-handed (`Matchup::average()`), so the handedness half of Anticipation only runs in unit tests. Needs per-slot batters and a check that pitcher targeting handles left-handers.
- Run environment 5.22 runs/team-game (MLB ≈ 4.5): ground balls still go for hits at .40 (MLB .24); infield hits are 20% of grounders, mostly high choppers from the contact model's launch-angle spread. Turning is still the largest movement cost (runs would be ~4.4 if turns were free); the atlas perception of §5.1 should reduce replan target jumps.
- One-hop throws; the motion stand-ins (no catching motion, bat not in the batter's hands).
- Still open from reviews: foul share 24% vs 50%; foul-territory walls. (Day-fixes #1, the runner who crossed home before a catch and never went back, is fixed: branch `worktree-runner-return-on-catch`, 2026-11-12; see that review's Resolution.)

Numbers (`sim-cli games --n 1024`), end of session: seed 2, 5.22 runs/team-game (before Anticipation), seed 5 2,048 games 5.22 with it; GB .399, LD .721, FB .116, PU .005. Per-game time unchanged (~260 ms single thread on the loaded laptop, same as HEAD before the session).

### Previous handoff (2026-11-12, first session)

**First:** nothing is waiting on a review. The [Codex review of `1f7ca4c..845a862`](docs/reviews/2026-11-12-codex-throws-relays-review.md) is resolved (receivers read throws instead of seeing their future; the throw solve finds long throws whose height peaks below 35°; the deep relay man goes out early on his own read; table edges). Its one open item: the deep relay man costs 0.1–0.15 runs per team-game and the cause is unknown (the experiments that ruled things out are in its Resolution). Owner direction (2026-11-12): players should not play perfectly, but play must be believable; treat this as a diagnosis item, not a number to tune away.

Done 2026-11-12:
- **Desktop launcher** (owner request): `bin/play.sh`, installed in the app menu as "3D Baseball" by `bin/install_launcher.sh`. It builds and opens the Unreal client with a new seed each launch (shown in the notification); right-click offers a chosen seed or the Rerun viewer. The engine had been moved to the desktop Trash on 2026-11-11 and was restored to `~/UnrealEngine/5.8.4`; the launcher now says so and asks before falling back to Rerun.
- **Ball shadow** (owner request): a dark disc straight under the ball, shrinking as it climbs (`845a862`). Owner found ball and shadow too big: ball now drawn 2× real (15 cm, was 3×), shadow the drawn ball's size (15 cm, was 50 cm).
- **Throws with drag and backspin** (`dbaae0a`, agent-models §5.5): `bb_physics::throw` (solve and `ThrowTable`), receivers take a throw at the bag when it is catchable there. Runs 4.32 → 4.99 per team-game: long throws take a few tenths longer, which the vacuum model's 0.92 × arm had hidden.
- **Cutoff and relay men** (`8c430e9`, agent-models §5.7): 5.05 runs (4.99 throwing direct; +0.06 to +0.12 on two seed sets, near noise). Open question for the review: why relays do not help the defense at all.

Next, in the order a viewer would notice:
- Run environment is now 5.14 runs/team-game (MLB ≈ 4.5) after the honest throw physics. Ground balls are the excess (GB hit .385 vs MLB .24). Before tuning abilities, look for planner bugs first (memory: three fielder-planning bugs once hid behind tuned values). One-hop throws (agent-models §5.5) are not chosen yet.
- Throwing, pitching and batting motion are stand-ins; no catching motion; the bat is not in the batter's hands.
- Still open from reviews: a runner who crosses home before a catch never returns (day-fixes review #1, P1, rules engine); foul share 24% vs 50%; foul-territory walls.

### Previous handoff (2026-11-11, after the owner's first watch)


The owner watched the Unreal client and asked for four things; all four are in ([notes and numbers](docs/calibration/2026-11-11-owner-watch-notes.md)):
1. **Human figures:** the engine's mannequin with idle / walk / jog, facing, team tint (`client/run.sh` copies the assets from the engine; they are not in the repository).
2. **Quit:** Esc or Q. Other keys: Space pause, `-` `=` speed, C camera.
3. **Double plays:** 3.6 → 2.0 a game (MLB ≈ 1.7): runner leads, going halfway on flies, a 4.45 s batter to first, fewer grounders, plus `K_perception` 2.7 and Discipline's grade-50 values (0.5, +0.9) to keep scoring at 4.48 runs per team.
4. **Backups:** idle fielders back up the fielder and the likely throw, one per base, per the coaching guides the owner linked.

Owner direction: favor what is visible over calibration; get numbers close and move on.

The [Codex review of that change set](docs/reviews/2026-11-11-codex-watch-fixes-review.md) is resolved: the catcher returns home when a runner rounds first, a backup can catch an overthrow in the air, runners hold their leads, nobody jumps or runs backward in the timeline, and the batter runs out of his box.

**Next, in the order a viewer would notice:**
- [~] Throwing, pitching and batting motion: stand-ins from the engine's stock clips are in (`914a187`: wind-up and delivery before each pitch, fielders' throws, the catcher's crouch, the batter's stance and turn). Still missing: a catching motion, the bat in the batter's hands, real baseball animation. The [Codex review of `31d57bb..914a187`](docs/reviews/2026-11-11-codex-motions-review.md) is resolved (backups wait until a throw is past the receiver; the batter's run out of his box is in the live simulator, not the encoder; throw motions follow recorded possession, `BbFrame.holder` / `release_in_s`).
- Cutoff and relay men on outfield throws (in both of the owner's references; not built).
- [x] The ball back to the pitcher and players walking to position between pitches: staged by the client (`StartReset` / `ResetFrame` in `BbDirector.cpp`; [docs/unreal-spike.md](docs/unreal-spike.md)). **Not yet reviewed by Codex**: include it in the next review. The owner has not watched it yet.
- Ground balls go for hits too often (.377; infield hits ~16% of grounders) since leads and the faster batter.
- Still open from reviews: a runner who crosses home before a catch never returns; foul share 24% vs 50%; foul-territory walls.

Numbers (`sim-cli games --n 512 --seed 2`), 2026-11-12 after the review fixes: 5.14 runs/team-game, K 17.4%, BB 7.4%, HR 4.3%; GB .385 / LD .716 / FB .141 / PU .017. Per-game time 218 ms on one thread (`simulate_game` timed over 24 games; 199 at the start of the session). (2026-11-11: 4.32 runs, GB .374.) Build `sim-cli` with `cargo build --release -p bb_cli` on its own.

1. [x] **Per-game budget (200 ms): met, ~155–170 ms on one thread** (the laptop's timings swing widely with load; compare builds interleaved, best of several). Two-stage aim solve and seeded scouting builds (−28 ms); batter filter with an analytic Jacobian, direct plate covariance, and aim points from the nominal's depth crossing (−61 ms; `sim-cli pa` 641 → 421 µs per pitch, identical B1 rates); live plays with a 0.1 s agent flight step, the recall reusing the chaser's same-tick prediction, and a single-pass intercept scan using the forward running curve (−51 ms; 512 games unchanged within noise). Breakdown in agent-models §12. Note: run totals differ by seed set (5.6–5.8 runs/team-game between 128- and 512-game samples); compare calibration runs on the same seeds.
2. [~] **Run environment: 4.29 runs/team-game now** (512 games, seed 2; MLB ≈ 4.5). (a0) is done: replans keep the fielder's speed, the intercept scan tests between its samples, chasers arrive at speed; the "everyone breaks" rule now uses each infielder's own read and has lost its extra 0.6 s condition (no measurable effect). Still open: (a) foul-territory walls and backstop (physics-spec §9.4; foul ground reaches the outfield-fence radius, so overthrows roll ~100 m), which needs the rules engine's thrown-ball-out-of-play award; (b) infield positioning: most remaining through-balls are in the 5–6 hole, the 3–4 hole, and up the middle on the first-base side, and shifting toward the pull side made things worse last time; (c) the contact model makes too many grounders and topped balls (BIP launch-angle quartiles −15/5/25° vs MLB ≈ −2/13/28°).
3. [ ] The rest of "M4 next steps" below.

---

## Owner decisions (from the 2026-10-31 overnight session)

- [?] **O1. Chase rate.** Average hitters chase 11.6% (MLB 29%). The owner approved trying perceptual overconfidence (2026-11-11); it was implemented, measured, and backed out: even total overconfidence gives 18% and wrecks contact. Diagnosis in [docs/calibration/2026-11-11-chase-diagnosis.md](docs/calibration/2026-11-11-chase-diagnosis.md): pitch locations are fine and the batter's strike belief is already uncertain near the edge; the swing decision is what is too sharp, and the low foul share (21% vs 50%) is what keeps K% and pitches/PA low. The owner chose to fix fouls first (2026-11-11). **That is not done:** [docs/calibration/2026-11-11-foul-share-diagnosis.md](docs/calibration/2026-11-11-foul-share-diagnosis.md) shows no noise setting gets the foul share past ~30% (timing errors turn into whiffs, glancing contact stays forward), so the gap is structural. Next: get the MLB foul-direction split and oblique-collision data (new research item R13), then check the bat's angle at contact and its time through the zone in the swing model. Discipline's grade-50 values (about c_D 0.5, b_D 0.9; currently 1.0, 0.0) stay parked behind that. `cargo run --release -p bb_game --example foul_diag -- 3000 [timing ms] [bat-control cm] [K_perception]`. Tools: `sim-cli pa` prints swing rate by miss distance; `cargo run --release -p bb_game --example chase_diag -- 3000 [c_D] [b_D]`.
- [x] **O2. Interim knobs: accepted by the owner (2026-11-11)** as the working point: `K_perception` 2.25, `K_command` 0.8, `K_swingplane` 1.0 (`Knobs::calibrated_m3()`).
- [x] **O3. Spec deviations made overnight: accepted by the owner (2026-11-11).** Full-state RK4 with spin decay inside the ODE (physics-spec §5.1); frictionless-impulse energy-guard fallback (physics-spec §8.1); event-log frame identifiers `BBEH`/`BBEV`/`BBEF` (data-formats §4.1); rolling resistance 0.22/0.28 (`bb_physics::ground`; physics-spec §9.1 now lists them); the batter's 10-state filter (agent-models §3.3); the pursuit-limit and persistent-bias perception terms; the scoreboard as a public channel for pitch-type learning (agent-models §3.8 [C]).
- [?] M0.9 Owner review: **the Unreal spike passes every unattended check (2026-11-11, [docs/unreal-spike.md](docs/unreal-spike.md))**: ~300 fps offscreen, 2.4 GB, 6 s rebuilds. Client v0 plays full games with capsules, a bat, two cameras and play-by-play. Still needed from the owner: run `client/run.sh`, confirm the editor window opens, and judge watchability; then D6 can be marked confirmed.

---

## Done

- [x] Brainstorming note (2026-10-26); Planning Passes 1–2 (archived); owner decisions; merged [PLAN.md](PLAN.md)
- [x] Specs P1–P4 drafted (data formats, agent models, calibration targets, rules) and the physics spec (2026-10-30/31)
- [x] **M0 Foundations** (2026-10-31, except the Unreal spike M0.8/M0.9): git repo; toolchains (rustup, uv, zig linker shim, toolbx for `flatc`); Cargo workspace with the crate split; `bb_math` (Vec3/Mat3, units, `libm` wrappers, **Philox4x32-10** keyed RNG with KATs); six FlatBuffers schemas incl. the R12 additions; event-log container with BLAKE3 digest chain and determinism tests; `sim-cli`; Rerun output; [CLAUDE.md](CLAUDE.md); crate-graph test enforcing `bb_agents` ↛ `bb_world`
- [x] **M1 Pitch flight** (2026-10-31): RK4 flight, Hermite events, atmosphere, spin-axis/tilt conventions, ABS zone, release point, seeded aiming solve with explicit failure, movement metrics, arsenal templates for 10 pitch types (in public ranges, lefty mirror), release execution noise; Tier A A1–A6 and plausibility bands pass; **independent Python oracle written by Codex** agrees to ~3e-8 m; tunneling gate test; `sim-cli pitch/tunnel/arsenal`
- [x] **M2 Bat and contact** (2026-10-31): bat profile with integrated mass properties (A12), §8.1 impulse solver with energy guard (A7–A9, A11, oracle fixtures), COR location profile, contact detection and resolution (end caps, adaptive scan), analytic swing model (zero-lag template, braking, plane tilt), swing execution noise, `bb_world` pitch resolution, cage harness; M2 gate sweeps pass; contact map in [docs/m2-contact-map.md](docs/m2-contact-map.md); `sim-cli cage/cage-map`
- [x] **M3 Batter vs pitcher, mechanisms** (2026-10-31): 20–80 grade mappings (§10); batter hypothesis bank (10-state EKF per pitch type: plate point, arrival, break deviation, release point, persistent perceptual biases), pursuit-limited angular noise, spin cue, sequence prior; swing/take with Discipline, full-covariance P(strike), own-execution beliefs, anticipated-correction credit; causal mid-swing adjustment (Adjustability); real check swings judged by yaw; bootstrap pitcher selection with count-based target maps; in-game adaptation (`PitcherMemory`) with a times-through-the-order test; archetypes; `sim-cli pa/sensitivity/archetypes`; monotonicity tests; performance pass (~395 µs per pitch)
- [x] **M4 started** (2026-10-31): `bb_rules` pure step function with a TOML scenario corpus (39 scenarios); ground bounces/rolls, fence, home runs, wall caroms; walking-skeleton play simulator (fielders, dives, glove capture, throws, runners, tag-ups); full AI-vs-AI games in `sim-cli game/games`
- [x] Codex reviews: M0–M2 (10 findings, all fixed), M2–M3 (19 findings: 15 fixed, 4 deferred with reasons), in [docs/reviews/](docs/reviews/)

---

## Phase: M3 calibration (open) and M4 (current)

### M3 gate items still open
- [~] B1 plate-appearance rates in rough Tier B bands: 13/20 in band ([report](docs/calibration/2026-10-31-m3-interim.md)). Open: O-Swing% (11 vs 29), foul share (18 vs 50), called-K share (45 vs 25), pitches/PA (3.0), Zone% (56), K% (13), HBP% (no evasion). Blocked on O1 for the chase mechanism
- [x] Velocity anchoring (agent-models §3.1): each type's expected speed anchors 20% toward the speed perceived on the previous pitch; test `bb_game/tests/anchoring.rs` (changeups after fastballs are timed earlier)
- [ ] Composure (leverage-scaled noise, agent-models §1.5): needs game state; wire up in M4 now that games exist
- [ ] Hit-by-pitch evasion and the posed capsule body (agent-models §3.7)
- [ ] SpinCapacity has no effect (the batter's scouting knows this pitcher's true shapes; a league-prior blend would fix it); SpinRecognition is weak; QuickHands flat between 30 and 50. See the report's §7.2 notes
- [ ] `Executed*` visibility isolation (module or crate split, plus a compile-fail test)
- [ ] `criterion` benchmarks for the per-pitch budget (the timing harness is `bb_game/examples/perf.rs`)

### M4 next steps
- [x] Fix the findings of the Codex M4 review ([docs/reviews/2026-10-31-codex-m4-review.md](docs/reviews/2026-10-31-codex-m4-review.md), with Resolution)
- [x] Codex live-simulator review ([docs/reviews/2026-10-31-codex-live-review.md](docs/reviews/2026-10-31-codex-live-review.md)): 8 fixed, 3 partly, 8 deferred to M5, with Resolution
- [x] Event-log output from real games (`bb_game::eventlog`, one `PlayRecord` per pitch) and the **M0 exit test on real games**: same seed ⇒ same digest across threads; the log reads back verified (`bb_game/tests/determinism.rs`)
- [~] Event-sourced box score in `bb_stats`: runs, linescore, PA by result, LOB, outs per half-inning from the log alone, with accounting identities over 80 games (`bb_stats/tests/accounting.rs`). Hits vs errors still need the scorer (rules-spec §10); `bb_game::game::classify_bip` is a provisional lab ruling that reads facts not in the log (day-fixes review findings 4–5), to be replaced by persisted `ScorerEvidence` and a pure scorer
- [~] Rules engine, from the Codex day-fixes review: [x] a runner returning from home after a catch (un-scoring a pending run; finding 1, P1; fixed 2026-11-12, rules-spec §5.3, `--example home_before_catch`); [ ] a fielder-touch fact so the retouch reference is first touch, not secure possession (finding 2)
- [ ] Rules engine: uncaught third strike and caught foul tips (catcher capture), overrunning first, two runners on a base, passing runners, interference (§5.7); grow the corpus toward 150 scenarios (§12.3)
- [x] Causal live-ball simulator (`bb_game/src/live.rs`): tick loop, fielders predict from noisy observations with their own physics, calls and re-calls, coverage, real captures and deflections, wall robberies, thrown balls with tracking receivers, observation-based runners, rules stepped live (2026-10-31)
- [ ] Live-ball simulator next steps (relays, cutoffs, and throws with drag done 2026-11-12; no double cut, no one-hop throws): atlas-based perception (agent-models §5.1), runner leads, rounding and slides, `BallPassedBase`, first-touch facts, the deferred live-review findings (3–6, 11–13, 16), runners reconsidering on bobbles and throw releases
- [ ] Run environment: runs/team-game is ~5.6 with the causal simulator (target ~4.5). Ground balls reach base .55 of the time (MLB ≈ .24) because too many get through the fixed-table infield positions; LD/FB/PU are near MLB rates. Measure first: by batted-ball type and fielder (`examples/dump_plays.rs` plus `analysis/plots/plays_topdown.py` give visual checks)
- [x] Play timelines for the client (data-formats §5, `bb_game::timeline`: ball knots, markers, telemetry, view state) and Rerun output of whole games (`sim-cli game --rrd`)
- [x] C ABI for the Unreal spike (`bb_ffi`: create/advance/score/free, header `sim/crates/bb_ffi/include/bb_ffi.h`); a C client plays a full game (`sim/crates/bb_ffi/c/run_smoke.sh`)
- [~] M0.8 Unreal spike: done and passed. Unreal client v0 (capsules, a camera, a scoreboard) plays full games: `client/run.sh`. Also in: stands as plain blocks, a runners-on diamond in the scoreboard, keys (Space pause, - and = speed, C camera: broadcast / behind home / overhead; `-BbCamera=` on the command line). The window mode was run on the desktop: 328 fps at 1600×900, worst frame 7.5 ms. **Not yet reviewed by Codex:** commits after `4ebe7fd` (relay timing, stands, HUD, keys). Done since: play-by-play text, the bat and swing, a center-field pitch camera cutting to a follow camera, award jogs (walks, home-run trots), clips that end on the deciding catch. Next, roughly in order of what a viewer notices: the pitcher's delivery and fielders' throwing and catching motion (everyone still slides), the ball returning to the pitcher between pitches, facing direction, runners-on and line score in the HUD, a ball trail on deep flies. **Owner judges watchability**

### Research follow-ups (from the specs' open-item lists)
- [ ] R1. Nathan's current Trajectory Calculator spreadsheet; confirm C_D/C_L against the `pybbda` port
- [ ] R2. Wood-bat COR versus impact location, and its speed slope
- [ ] R3. Spin-versus-undercut data for test A10. The sim gives ~4,300 rpm at a 2 cm undercut, which may be high
- [ ] R4. Ball bounce and rolling values for grass, dirt, turf (rolling resistance was raised to 0.22/0.28 so grounders stop in ~110 m; needs real roll-out data)
- [ ] R5. Baseball moment-of-inertia factor α
- [ ] R6. Tune the default bat to ~31–32 oz (it is 34.0 oz, outside its own §6.2 band, as the oracle noted); add 2–3 stock bats
- [ ] R7. Exact ABS edge rule (optional)
- [~] R8. Savant's CSV documentation (checked 2026-10-31) confirms `spin_axis` 180 = pure-backspin fastball and that `pfx_x`, `release_pos_x`, `plate_x` are from the catcher's perspective; also, from 2026 `plate_x` is at mid-plate (data-formats §9.4 updated). Still to do: confirm numeric signs on a real Savant sample. Also settle which Savant movement window matches the sim's full-flight IVB (the arsenal templates assume full-flight ≈ Savant scale)
- [ ] R9. Literature pass on interception and time-to-contact perception (σ_ang, Weber fraction, blackout, the 1.2 rad/s pursuit limit, the 10% persistent share)
- [ ] R10. Refresh every [R] reference value in calibration-targets against the latest full season
- [ ] R11. Verify the OBR rule numbers and MLB interpretations cited in rules-spec
- [ ] R13. MLB fouls by direction (pull side, opposite side, backward) and a published oblique bat-ball collision data set (does glancing contact grip or slide?), for the foul-share gap (calibration report 2026-11-11)
- [x] R12. Rules-spec §15 item 2 additions are in the M0.5 schemas (2026-10-31)

### M0 leftovers
- [ ] M0.1b Install Git LFS before any Unreal asset is committed (`.gitattributes` already routes `*.uasset`/`*.umap` to LFS). Needs `sudo dnf install git-lfs` or the toolbx
- [ ] M0.2b Optional: `sudo dnf install gcc glibc-devel` on the host to retire the zig linker shim (see CLAUDE.md)
- [x] M0.8 **Unreal spike** on this machine: passed 2026-11-11 ([docs/unreal-spike.md](docs/unreal-spike.md)). Engine at `~/UnrealEngine/5.8.4` (outside the repository; the 40 GB zip in the project folder can be deleted)

---

## Owner wishlist: eventual improvements (2026-11-12)

The owner asked for these to be recorded, **not worked on yet**. They are in the owner's order, not in priority order. When one starts, move it into its milestone and add detail.

- [ ] W1. **Stadium and field aesthetics**: a real ballpark look for the field and stands (the stands are plain blocks now). Presentation, M8
- [ ] W2. **Pitcher windup vs. stretch**: a full windup with the bases empty, the stretch with a runner on. The stretch should also shorten the delivery time, which matters to stolen bases (W13)
- [ ] W3. **Batter stances and swings**: each player has their own stance and swing, with a wide variety. The owner's examples of styles: Pete Rose, Darryl Strawberry, Gary Carter. The owner may send a longer list of styles to model later. Overlaps the parked "swing styles" item
- [ ] W4. **HUD**: make it look more like a TV broadcast
- [ ] W5. **More teams**: team names, mascots, and full rosters. Ties to M7 (league)
- [ ] W6. **Team selection** before a game
- [ ] W7. **Player aging**: a career arc for both skill and experience. M7
- [ ] W8. **Fielders' throwing animation** (also under the M0.8 client next steps)
- [ ] W9. **Player names**
- [ ] W10. **Fielding errors**. Errors must come from physical causes (a bobble, a bad throw), never from a die roll (D1). Ties to the scorer's hit-or-error ruling (rules-spec §10)
- [ ] W11. **Player and team statistics**, built from the event log (`bb_stats`)
- [ ] W12. **Umpires with animations** (an umpire-calls module, `bb_game/src/umpire.rs`, was in progress in the main checkout's uncommitted changes on 2026-11-12)
- [ ] W13. **Stolen bases** (needs runner leads, from the live-ball next steps, and the stretch delivery, W2)

---

## Later milestones (detail added when each one starts)

- [ ] **M5** Deepen the field
- [ ] **M6** Live management (primary experience)
- [ ] **M7** League and season
- [ ] **M8** Presentation polish. **Revisit the RAM upgrade before this.**

Exit gates for each are in [PLAN.md §14](PLAN.md).

---

## Parked (decided "later")

- Injuries, physically grounded (D18), after M8
- Seam-shifted wake, weather and wind, human-umpire mode, swing styles, scouting inferred from behavior, online play
