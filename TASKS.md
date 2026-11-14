# Task List

**Last updated:** 2026-11-13 (hurried throws, A3; before that, handedness, A1; before that, the dive rate judged and fixed, A5, and rosters, A2, merged; before that, dives shown in the client, the get-up after a rimmed dive, two-out throws; before that, the overrun of first and runner position samples; before that, foul measures, sources and the correction experiment; pushed; the spray experiment parked for later; before that, the read knows the ball started at the plate; before that, called-off infielders back up the catch; before that, pop-up calls on `fix/popup-calls`; before that, owner's observations merged into the "Owner backlog" section; before that, the coordinator run on `coord/review-fixes`)
The live to-do list for the project. [PLAN.md](PLAN.md) holds decisions and design. This file holds what is next and what is done. Update it at the end of every work session.

Status: `[ ]` not started · `[~]` in progress · `[x]` done · `[?]` needs an owner decision

---

## Resume here (2026-11-14, overnight lead session, 00:00 to 06:00)

Main is pushed and passes `cargo test --workspace --release`: 305 pass, 3 ignored (two owner-deferred pop-up fixtures, one diagnostic table). Where the numbers stand, measured together on main `a321e55`, and how the night's changes add up: [docs/metrics-ledger.md](docs/metrics-ledger.md) §1. Workers ran in herdr panes (tabs "Workers", "Workers 2"), each in a worktree under `.claude/worktrees/` on a `work/*` branch.

**Waiting for the owner:**
- **The swing, v2** (`work/swing-model`, not merged; [note on that branch](docs/calibration/2026-11-14-swing-model.md), Proposed A to D). The old swing turned at a fixed angular speed: 64 mph inside, 84 outside (Statcast 71 to 72 at both), and the outer third was a home-run launchpad. v2 matches Statcast's bat speed by location, swing length, tilt and bat angle; with it home runs fall from 5.1% of PA to 0.56% (real 3.0%) and runs from 5.30 to 3.74. It does not move the foul share (21.9 to 24.2%, real 52.5%). Branched from `bcbb91f`: merge main before judging.
- **Runner acceleration.** Merged at 0.68 s (fits home to first, 4.25 s against 4.26 to 4.30). The Gemini tag-up review says the 90-ft sample was of faster-than-average runners and league speed gives about 0.75 s. Left- and right-handed batters start 1 m apart along the line and get the same time.
- **The tag-up decision rule** asks for a 99% chance of being safe (0.35 s margin, 0.15 s noise); flies caught at 225 to 275 ft are never sent (real about 40%). Replacing it with agent-models §6.3's run-expectancy rule is a model change.
- **Wall braking** costs 0.08 to 0.18 runs and has no sourced value (play-area recheck, Judgment Point 3).
- **Left-on-base counting:** two `known_gap` rules scenarios (runners between bases, a held pass).
- **Recalibration.** Runs are 5.83 [6.21] against 4.39, K 15.9% against 22.6%, HR/PA 5.2% against 3.0%, ground-ball hits .327 [.333] against .245; no knob has been touched since 2026-11-11. Sacrifice flies (0.253 a team-game), double-play conversion (33%), dives and running times are now near real. [Diagnosis and ranked causes](docs/calibration/2026-11-14-run-environment-diagnosis.md).
- The speed keys (0.25 to 16x) were never pressed: headless runs cannot.

**Reviews:**
- **[Codex Sol review of the live-play code as it stands](docs/reviews/2026-11-14-codex-live-play-review.md): Resolution open, nothing fixed yet. Start here.** One P1, eight P2s, two P3s. It replayed seeds 0 to 49 on two independent builds (all digests match) and ran two counterfactuals; neither showed a defect behind the high run total.
  - **P1-1 (older, still there):** fielders' coordination reads a teammate's private target and uncommunicated estimate (`fielding_point`, cutoff and backup placement, who covers second, `recall`, `relay_job`, `assign_among`), and the spec's 0.3 s communication delay is not implemented. Its probe: move only the center fielder's target and second base's cover changes at once.
  - **P2s:** a bounced or caromed throw can stay live after crossing into dead territory; a dive clipped at the wall still catches beyond it; the carry-to-second continuation still charges the pivot that carriers no longer pay; two wall contacts can reuse `SurfaceBounce` RNG streams; "first play" is reduced to "first throw with no recorded out"; the runner's tag allowance uses the force at the start, not at his destination; a pivot can choose 80% effort against the full-arm pivot contract; stopping walk-off awards drops the batter's required minimum advance.
  - **P3s:** pivot and set-up gates gain a tick from the floating-point clock origin; the continuation regression test no longer shows the continuation changing a choice.
- Gemini reviews with a Resolution written: play area and its recheck, throw choices, scorer, double-play pivot, tag-ups. Codex: test gaps, arm strength by position.
- **Open:** [Gemini infield-alignment review](docs/reviews/2026-11-14-gemini-infield-alignment-review.md) (three P2s). Fixes are in progress on `work/infield-review-fixes` (a Gemini agent, one commit, unmeasured, unmerged). A Gemini recheck of the scorer fixes was started and died at the reviewer's weekly quota.
- No review at all yet: A16 home-run display, the box-score screen and `bb_game_box_score`, the side-change cut, speed for accuracy on throws (`4477892`).

**Deferred by the reviews, each recorded in its Resolution:** tag geometry (the world tags 1 m short of the bag, a carry adds 0.25 s twice); the restored counterfactual in the 3/140 fixture; a runner oversliding a base (scorer P1-5 part 1); captions from the log; third basemen converting 48% of double-play grounders against 38%; the client draws the dugout roof at 2.4 m against the sim's 4 ft; six older test gaps listed in the test-gaps review.

**Not started or stopped part way:** the pitcher as a fielder (he takes 20% of grounders against about 8%; `work/pitcher-fielding`, a stash in worktree `calib-diagnosis`); pitchers' zone targets and the tracking limit (diagnosis ranks 4 and 5); a 133 mph exit speed (the v1 swing's tail; v2 removes it).

**Machine notes:** the laptop ran at 93 C under load all night with a guard that never had to pause anything (memory: laptop-heat-guard). `sim/target` once treated stale library builds as fresh; the fix is `cargo clean --release -p` the workspace crates. Gemini 4 Argon's quota is used up for about a week; Hydrogen and Codex are available.

Done and merged 2026-11-14: A10 play area, sim and client, with review fixes; A4 throw choices with review fixes, speed for accuracy, the double-play pivot with review fixes; tag-ups and runner τ; A12 scorer, batting lines, review fixes, and the box score on screen; A16 home-run camera and card; infield alignment from Savant 2024; arm strength by position; the side-change cut and stepped speed keys; two rounds of deferred tests; the sources, tag-up, diagnosis and throw-choice notes.

## Previous (2026-11-13: hurried throws, A3)

Branch `claude/handedness-complete-6e0bc7` (handedness is merged into main at `a4c8219`; A3 is on top). `cargo test --workspace --release`: 195 pass. Spec: agent-models §5.5 (set-up, the turn, hurrying), §10 (ratings note); measurements in [the throwing note](docs/calibration/2026-11-13-throwing.md).

- **Nothing is waiting on a review.** The [Codex hurried-throws review](docs/reviews/2026-11-13-codex-hurried-throws-review.md) is resolved: no P1; both P2s (dive footwork was read from a flag that clears mid-possession; carrying the ball to a bag waited for a throw's set-up) and both P3s (spec gaps) are fixed. It reproduced every number in the note and matched game digests with the new terms off.
- **Open, for the next calibration:** runs/team-game 5.35 → 5.53 (seeds 0–299). The footwork a fielder owes moving away (0.2 s) or after a dive (0.3 s) costs 0.37; hurrying and the turn give back 0.18. Recalibrate with the handedness mix (below). Check the outfield's 0.85 s transfer against measured exchange times: it may already contain some of that footwork.
- **Next in the agreed order:** A4 throw choices from the outs and the runners, then A17 errors.

Done (A3):
- **A fielder needs time to get set, and can throw before he is.** Set = transfer + turn + footwork. He can let go at 0.6 of transfer + turn; the earlier, the slower (down to 90% of his arm, 82% off balance) and the wilder (direction σ ×2, ×3 off balance). He hurries only when his own read of the race leaves him under 0.25 s from a full set-up. About 2.5 throws a game are hurried; a fully hurried one is not taken cleanly twice as often as a set one.
- **The glove hand turns the body.** A target off the glove shoulder is quick, off the throwing shoulder slow (±0.12 s): a left-handed first baseman is set for second 0.24 s sooner than a right-handed one.
- **Ratings:** `FielderAttrs::with_throwing(strength, accuracy, transfer)` (grades, §10); `sim-cli games --arm-strength/--arm-accuracy/--transfer`. Grade-80 arms make .606 outs per ball in play against .548 for grade 20.
- Throw noise is drawn in `bb_agents::motor::execute_throw`; what a fielder knows of his own set-up is `bb_agents::throwing`. With the new terms off, every game is unchanged. New diagnostic: `cargo run --release -p bb_game --example throw_probe -- 300 0`.

Not done:
- **No player has his own throwing ratings** (all grade 50): that needs the population generator.
- A backhand is no harder to field than a forehand (moved to A17). Speed-for-accuracy on set throws and composure are not built. A hurried throw is not shown differently in the client.
- All eleven new parameters are estimates [E] with no real-game source yet.

## Previous: 2026-11-13, handedness, A1

Branch `claude/a1-173226` (from main at `3de5572`). `cargo test --workspace --release`: 179 pass. Spec: data-formats §2.4 (hands), agent-models §5.5 (the throwing hand); measurements in [the handedness note](docs/calibration/2026-11-13-handedness.md).

**Nothing is waiting on a review.** The [Codex handedness review](docs/reviews/2026-11-13-codex-handedness-review.md) is resolved: no P1 or P2; its two P3s (wrong shares in the docs, an ABI comment naming a field that does not exist) are fixed. It rebuilt 54,000 players' hands with its own Python Philox, confirmed names and numbers are unchanged from before, matched 1,452 pitch records against its own matchup construction, and checked the mirror on all ten pitch types.

- **Open, for the next calibration:** left-handers raised the run environment. Runs/team-game 4.73 → 5.35 (seeds 0–299) and 4.89 → 5.43 (seeds 1000–1299); K% 17.8 → 15.2; HR/PA 4.1% → 5.2%. Nothing was tuned. The interim knobs were set with every plate appearance right against right; half are now opposite-handed (49.8% measured). Recalibrate the plate-appearance knobs over the real mix.
- **Open, a model question:** the platoon advantage comes out of the geometry and is about twice the real size: K 19.8% same-handed against 14.4% opposite, 5.4 points (MLB about 3 from memory, unsourced; wOBA gap .030–.045). Codex's interventions (table in the note) show it is the pitches' sideways movement meeting a batter who sees and swings from one side; neither the eye offset nor the release side explains it alone, so there is no single parameter to turn yet.
- **Next in the agreed order:** A3 throwing ratings and hurried throws (which is also where the glove hand should start to matter, below).

Done (A1):
- **Every player has hands.** The roster draws a throwing hand from the position (27% of pitchers, 35% of first basemen, 20–25% of outfielders throw left; nobody at catcher, second, third or short) and a batting side from the throwing hand (position players: 32% left, 13% switch, 55% right) [E]. New RNG purpose `RosterHands` = 13, so no seed's names or numbers changed. `sim-cli game` prints bats/throws on each batting line; the event log's header and `BbPlayer` carry them (`bats` 3 = switch).
- **Games play them.** Each batter stands on his own side against the other side's pitcher, a switch hitter on the side opposite the pitcher's hand; the pitcher throws with his own hand (`Matchup::for_hands`, one matchup per pairing in a game). Abilities are still average. Every game's digest changed.
- **A left-hander is a mirror image** (`cargo run --release -p bb_game --example platoon`): same-handed pairings agree with each other, opposite-handed ones with each other, pull is 5–6° to the batter's own side, and left-handed batters are hit by pitches no more often (the targeting check A1 asked for).
- **Fielders throw from their throwing hand:** the ball leaves 0.35 m [E] to the arm side (`throw_release_point`).
- **Client:** a left-hander's delivery, throw and batting stance are the right-handed stand-in clips in a mirror (the figure is flipped across his facing). Checked headlessly: seed 7 (left-handed batters in the first-base-side box) and seed 6 (a left-handed pitcher follows through with his left arm; `client/run.sh 6 -RenderOffscreen -BbShots=14 -BbShotEvery=0.45 -BbSpeed=0.25`). No throw by a left-handed fielder was looked at.

Not done:
- **The glove hand does nothing in the sim yet.** Reach is the same to both sides, a backhand costs nothing, and a throw needs no turn of the body. It belongs with set-up time in A3 (agent-models §5.5 says so). Gloves are not drawn (L2).
- A manager does not use hands: no lineups by pitcher hand (B2), no left-handed specialist.
- The fielders' spray beliefs (agent-models §5.10) were fitted on right-handed batters and are mirrored for left-handers; refit on both when next refit.
- Dives did not fall: 3.8 a game, the shortstop 0.98 and now the second baseman 0.90 (left-handed batters pull to his side).

## Previous: 2026-11-13, the dive rate, A5, and rosters, A2

Two sessions ran side by side; both are merged into main at `74cc5ca` and pushed. `cargo test --workspace --release` on the merge: 175 pass. The client was rebuilt against the new header and run headlessly on seed 7 (`client/run.sh 7 -RenderOffscreen -BbShots=6`): no errors.

- **Nothing is waiting on a review.** Both Codex reviews are resolved.
- **Open:** the stretch on his feet is unvalidated and moves the run environment (see the dive rate below).
- **Next:** A1 handedness (the roster already has `Player.bats`/`throws` to fill).

### The dive rate (A5)

The [Codex dive-rate review](docs/reviews/2026-11-13-codex-dive-rate-review.md) is resolved: its P1 (the outfielder weighed a dive with the world's ball speed, not his read) and the dive half of a P2 (he could change his mind while waiting for the closest tick) are fixed; two P3s in the note are fixed.

**Open from that review:** the stretch on his feet is unvalidated and moves the run environment (about 0.2 runs/team-game per 0.1 m of stretch; table in the note). Check it against charted plays before relying on the lower run total. Also untested: airborne `dive_gains`, stretch misses, a change of mind during a stretch's wait.

Done ([note](docs/calibration/2026-11-13-dive-rate.md), agent-models §5.3–§5.4):
- **Judged: the dive rate was not believable.** 22 dives a game against 2 to 3 in real games (Sports Info Solutions: a team's shortstops dive 36 times a season; the sim's dove 4.4 times a game for both teams, and its outfielders 8 times a game).
- **Fixed by when a fielder leaves his feet**, not by a rate: a dive must gain him something by his own read (no dive at a roller he is about to pick up); a dive needs its lunge time from when he knew he must dive (the pitcher cannot lay out for 0.7 s after contact); a ball just past reach is taken with a stretch on his feet (0.4 m, 0.3 s off balance); the further out, the harder the catch; an outfielder dives only when he gives himself an even chance. **3.3–3.7 dives a game, 41–45% caught.**
- **Run environment moved, not by tuning:** runs/team-game 5.48 → 4.73 (seeds 0–299) and 5.71 → 4.89 (seeds 1000–1299); GB hits .425 → .333 (MLB .24). The stretch does it (without it: 5.60 runs, 7.5 dives a game). Fly-ball hits rose on one seed set (.131 → .162) and not on the other.
- New diagnostic: `cargo run --release -p bb_game --example dive_probe -- 300 1000` (add `list` to print each dive).

Not done: real dive and get-up clips; the stretch has no animation; no dive was looked at in the client after this change. The shortstop still dives about 2.5 times the real rate (all batters are right-handed until A1, and near misses from the earliest-point fielding plan). The four new parameters are estimates [E].

### Rosters (A2)

Game digests are unchanged (seeds 1, 2, 3, 42 compared before and after): rosters are identity only and the log header is outside the digest.

The [Codex rosters review](docs/reviews/2026-11-13-codex-rosters-review.md) is resolved: no P1/P2, and its two P3s (no literal roster fixture; runs not checked per player) are fixed. It reproduced 18,000 generated players with its own Python generator, confirmed the unchanged digests, and checked the C layouts. Deferred: a committed test of a log without rosters.

Done (A2; spec in data-formats §2.4):
- **Every game has two named nines.** `bb_game::roster::Rosters::generate(seed)` gives each side nine players: given and family name, uniform number (1–99, unique on the team), position (each of 1–9 once), place in the batting order (a seeded shuffle of the eight position players; the pitcher bats ninth), and `bats`/`throws` (filled by A1, above). The same seed gives the same players. New RNG purpose `RosterGeneration` = 12.
- **Names [E]:** four pools (English .62, Spanish .28, Japanese .06, Korean .04), ASCII only; a player's pool is kept as `origin` so his look can match later (L2). Family names are unique in a game and given names on a team, so play-by-play uses the family name alone.
- **A player still has two agent ids** (batting `side<<8 | order`, fielding `side<<8 | 0x80 | position`); the roster resolves either. Merging them would change every game's RNG keys, so it waits for substitutions.
- **Where names show:** a caption that ends a plate appearance starts with the batter's family name ("Ramirez: Single to left"), so the client's HUD shows names with no client change. `sim-cli game` prints each side's batting lines by name (`bb_stats::BoxScore.batting`: PA, R, K, BB, HBP, HR, batted balls; at-bats and hits still need the scorer, A12). `--pbp` labels each pitch with the batter's position and name. The event log's header carries both rosters (schema 0.2).
- **C ABI for the client:** `BbPlayer`, `bb_game_roster(game, side, batting_slot)`, `bb_game_player(game, agent_id)`, and `BbPlayInfo.batter` (the batter of a play). That is what A16's home-run display and uniform numbers need.

Not done:
- `client/run.sh` rebuilds the client against the header every time it runs (done once after the merge, above). Nothing in the client shows numbers or a lineup yet.
- Choices the owner has not ruled on (data-formats §2.4): ASCII names without accents (marked Proposed); the pool weights [E]; a random batting order while every player is average.
- A1 (handedness) is next in the agreed order: fill `Player.bats`/`throws` in the generator, add switch hitters, and give each batting slot its own batter instead of `Matchup::average()`.

## Previous: 2026-11-13, seventh session: dives, from the owner watching seed 7475

Branch `claude/cf-misplay-analysis-98b93b` (from main at `2355c34`). `cargo test --workspace --release`: all pass.

**Nothing is waiting on a review.** The [Codex dive and two-out throw review](docs/reviews/2026-11-13-codex-dive-and-two-out-throw-review.md) and the [Codex dive animation review](docs/reviews/2026-11-13-codex-dive-animation-review.md) are resolved: all eight findings are fixed (five P2s, three P3s). The animation review's fixes: the lunge never crosses the wall, an adjacent second dive reaches the client, a play that ends during a dive lasts until the diver is up, the rise eases out of the layout, and an early pitcher dive beats the delivery pose.

Done (the owner watched seed 7475 and reported three plays; the traces are in the commits):
- **A rimmed dive no longer allows a second dive from the ground** (play 25). The get-up runs from the first dive. A downed fielder can only grab a low ball within arm's reach. Runs/team-game 5.48 → 5.85 and GB hits .413 → .436 over 3 × 200 games, almost all from this fix (Codex verified it). The double dive had been converting hits into outs.
- **With two out the holder takes the force over a tag on the lead runner** (play 26: the pitcher had thrown home instead of to first). `Sim::out_weight`, `TWO_OUT_TAG_W` 0.9 [E]. The caption for an inning-ending tag now reads "Fielder's choice …, runner tagged out" instead of "Force out".
- **Dives are shown in the client** (A5, play 58: the shortstop's diving stop looked like him standing with the ball for 2 s). In the sim a dive carries the body over a 0.3 s lunge (`DIVE_LUNGE_S` [E]), and the timeline gets `Dive`/`DiveUp` markers. `BbFrame.dives` reports dives under way. The client lays the figure out toward the ball (a stock leap clip, tipped 45° and dropped to the turf), holds him there, and stands him up through the landing clip's crouch. Check it headlessly with `client/run.sh 7 -RenderOffscreen -BbDiveShots=3`; `-BbAnimTest=dive` previews pose candidates.
- Measured: fielders dive about 23 times a game (both teams). Most dives start at the edge of dive range (2.25–2.5 m), and about 60% are caught. The owner accepts dives ("it's fine if he dives").

Not done: real dive and get-up clips (the stand-ins are the mannequin's leap and landing). A18 (outfielders block hard singles instead of diving) was done with the dive rate in the eighth session.

## Previous: 2026-11-13, sixth session: the overrun of first

Branch `claude/next-task-list-item-11a78e` (continuing from the fifth session below). `cargo test --workspace --release`: 151 passed, 0 failed.

**Nothing is waiting on a review.** The [Codex overrun-samples review](docs/reviews/2026-11-13-codex-overrun-samples-review.md) is resolved: no P1/P2, and its one P3 (the spec overstated what is unmodelled: a batter-runner sent back by a blocked base already retreats to first, judged from the samples) is fixed. It reproduced every count and verified determinism, box-score identities, and the timelines headlessly. Merged into main (not pushed).

Done (coordinator item 3; [note](docs/calibration/2026-11-13-overrun-and-samples.md), agent-models §6.2, rules-spec §5.5 and §15 item 7):
- **Position samples.** The live sim sends every runner on the base paths to the rules engine as a `RunnerPosition` at 30 Hz (`Sim::sample_runners`), so the engine's position-based overrun and passing rules now run in real games.
- **The overrun of first.** A batter-runner who stops at first runs through the bag, brakes down the line (`OVERRUN_DECEL` 4 m/s² [E], 8.4 m from 8.2 m/s), and walks back (`OVERRUN_RETURN_MPS` 2.5 m/s [E]); the rules engine sees him protected and the fielders make no play on him. In 512 games, 8,421 of 19,826 batter-runners who reached first overran it and came back; every one who was off the bag at the end of a play was put out, the inning or game ended, or the ball left the park. No passing outs and no rules warnings. Runs: seed 5 5.36 → 5.39, seed 2 5.21 → 5.23 (1,024 games; noise ±0.07). Not tuned.
- **Found on the way: swings could start before the batter decided** (0.16% of swings, up to 18 ms), exposed when seed 11's changed game path failed the FFI no-give-away test. A swing now starts no earlier than the commit, moved whole (`Swing::start_no_earlier_than`, agent-models §3.4).
- New diagnostic: `cargo run --release -p bb_game --example overrun_probe -- 512 0`.
- Not done: rounding first and coming back on a hit, sliding into first, going for second on an overthrow during the overrun. Plays that leave the batter on first now last until he is back (about 6 s after he crosses the bag). Not watched in the client.

Next: items 4–7 of the coordinator list below (item 2, fouls, is parked with the spray experiment; item 6 is related to it).

## Previous: 2026-11-13, fifth session: foul measures

Branch `claude/next-task-list-item-11a78e`, merged into main and pushed to origin (owner, 2026-11-13: "push and stop here"). `cargo test --workspace --release`: 148 passed, 0 failed. No simulation behavior or values changed (game digests bit-identical): the code changes are diagnostics, new template methods, named [E] parameters that were literals, and a refactor of the cage lab's aim.

The [Codex foul-measures review](docs/reviews/2026-11-13-codex-foul-measures-review.md) is resolved: all eight findings fixed. Four were P1s about the note's conclusions, and one of them changed the diagnosis of early whiffs.

Done (coordinator item 2, as the owner directed: measure before changing calibration; [note](docs/calibration/2026-11-13-foul-measures.md)):
- `examples/foul_diag.rs` now prints R13's tables F1–F6 from the sim's swings, with the MLB value beside each, plus how whiffs miss against the bat's axis extended past the tip. An optional fifth argument sets the bat angle as a counterfactual. New `examples/foul_in_games.rs` gives the foul share by launch angle in full games. `sim-cli cage-map` gains a bat-angle counterfactual section. `SwingTemplate::bat_angle` / `with_bat_angle` (tested) give the counterfactual geometry.
- **R13's main hypothesis is refuted:** well-timed steep contact is foul at MLB rates. Over all contact, 40–60° is still short (60.5% against 72.4% above 40°). The largest deficit is low: P(foul | LA < 25°) is 3–11% against MLB's 22–57%. Full games agree with the PA model's call (22.9% against 22.6%), so the problem is the contact, not the fair/foul judgment.
- **Mistimed swings miss over or under the ball:** at AD 12–30°, 91% of whiffs are the bat passing over it (median 11 cm clear). Only 6% would have hit with a longer barrel. Late whiffs pass under. MLB batters still foul these balls off (whiff 23–40% against the sim's 43–74%). Late contact is at the right foul rate but rare (5% of swings against 26%).
- **Swing measures against Statcast:**
  - Swing length is 9.5 ft against 7.2.
  - AD spread is half of MLB's; the pooled AD-on-depth slope is +1.04°/in against 1.48.
  - Swing path tilt is 23.7° against 32. Vertical bat angle is −18.6° against −25 to −35.
  - The foul share by pitch height is inverted (high pitches foul least).
- **Bat-angle counterfactual:** −30° matches Statcast tilt (31.9°) and Blast's bat angle. The foul share goes 23.7 → 27.5%, but well-timed steep fouls overshoot (67% against 46% at 40–50°). It is a lever worth about a seventh of the gap, to remeasure with the next swing shape.

- **Follow-up (owner: "ask Fable which to do"; Fable: measure first; note §6–§7, corrected by the [Codex foul-sources review](docs/reviews/2026-11-13-codex-foul-sources-review.md), resolved):**
  - **New measures.** `foul_diag` accounts for each whiff's height miss (aim, template offset, rest, swing path, with RMS and the commit read). It also prints swing populations and the joint timing × height table. `cage-map` prints a timing × height grid at a fixed pitch and aim.
  - **Early whiffs.** The batter's final aim is the largest term: 20–30 cm high on SL/CU/CH, mostly misread at commit, yet most found the type by the end of the adjustment loop. The correction budget is about 8 cm (max 10.2).
  - **Swing populations:**
    - 25% early and over (92% whiff);
    - 38% close (1% whiff, but only 10% foul);
    - 25% bat under (61% foul: where the sim's fouls come from);
    - 12% the rest.
  - **No swing change is indicated or excluded** by these measures.

- **The correction experiment (owner: "yes"; note §8):**
  - **Named parameters.** `t_cut`, the phase limit, the check-swing threshold and the abort point are now named [E] parameters (`SwingTemplate::adj_cutoff_s`, `adj_phase_frac`, `abort_time_frac`; `batter::CHECK_THETA_RUNS`), with game digests bit-identical.
  - **New diagnostics.** `PitchRecord::known_gap` and `est_error` record what the batter knew and had not yet corrected, and his estimate's own error; `foul_diag` takes `adj=`, `cut=` and `phase=` counterfactuals.
  - **The limits bind** on fooled swings (81% saturated, 16 cm and 22 ms of known correction left).
  - **The tested settings do not close the foul gap.** Even with no limits the foul share goes only 23.7 → 26.1%. In a paired run (Codex), 925 saved whiffs became 623 fair and 302 foul. Foul per contact by AD changes little in the main bins (K% 19.8 → 8.4).
  - **Contact% follows Adjustability.** At 2.4 m/s O-Contact% enters its band, but that waits on the foul share.

**Nothing is waiting on a review.** The [Codex foul-correction review](docs/reviews/2026-11-13-codex-foul-correction-review.md) is resolved: all four findings fixed.

The owner stopped the foul work here (2026-11-13) and parked its next step for later: see "Foul share: the spray experiment" under [Parked](#parked-decided-later).

Next: items 3–7 of the coordinator list below (item 2, fouls, is parked with the spray experiment; item 6 is related to it).

## Previous: 2026-11-13, fourth session: the plate cue

Branch `claude/next-task-list-item-11a78e` (continuing from the third session below). `cargo test --workspace --release`: 147 passed, 0 failed.

**Nothing is waiting on a review.** Merged into main (not pushed).

Done (owner item 2; [note](docs/calibration/2026-11-13-plate-cue.md), agent-models §5.1 "The plate cue"):
- **The read knows the ball started at the plate.** While a batted ball is on its first flight, the fielder flies his read of its velocity back to contact with his own physics and typical spin (drag and gravity act on the velocity alone), which says where a ball going that fast must be now, and weighs that against his direct read in 3D by how good each is. The cue's own error is all spin (`examples/plate_cue.rs`: exact with the true spin). An outfielder's first-look landing error is now about 7 m, nearly all of it his velocity read (how hard and at what angle), against 25 m before; a ball 15 m from home is no longer read 26 m deep or 9 m wide.
- **The call rule's self-knowledge** is now where the ball comes down (`landing_depth_sigma`: he runs his own read with each error draw at one σ and sees how far each moves his landing point); without it every outfielder was sure of every pop-up at first look.
- **Refit:** outfield read σ₀ 3.0 → 3.5 m at 30 m (FB band). 1,024 games, seed 5 / seed 2: runs 5.44 → 5.36 / 5.26 → 5.21; LD .724 → .718 / .715 → .710 (MLB ≈ .68); FB .120 → .128 / .119 → .131 (≈ .13); PU .008 → .008 / .009 (≈ .02). Uncaught pop-ups in 128 games 7 → 6. Game time +6%.
- The gap-fly call test now uses a ball both outfielders are unsure of with 0.4 s of margin (was 30 ms; deferred popup-calls review item).
- [Codex plate-cue review](docs/reviews/2026-11-13-codex-plate-cue-review.md) **resolved**: all three findings fixed (the call's doubt now runs the fielder's own read per error draw, which carries the plate/velocity coupling and drag; the weighing's covariance matches the sampler's coupled lateral/height draw; the plate-cue diagnostic stops at the fence).
- New diagnostic `examples/hit_bases.rs` (hits by bases per batted-ball type). It shows **triples far above MLB**: about .07 of liners and .02 of ground balls go for triples, before and after this change (MLB triples are well under 1% of balls in play; not checked by type): a run-environment lead for later.

Next: items 2–7 of the coordinator list below (fouls first).

## Previous: 2026-11-13, third session: the called-off infielder

Branch `claude/next-task-list-item-11a78e` (cut from main `a23f7c2`), merged into main (not pushed). `cargo test --workspace --release`: 144 passed, 0 failed.

**Nothing is waiting on a review.** The [Codex peel-off review](docs/reviews/2026-11-13-codex-peel-off-review.md) is resolved: both findings fixed (the called-off man now never makes a glove attempt while the call stands; his job ends every tick the ball is down, caught or thrown, and a loose ball is re-called at once).

Done (owner item 1 below; [note](docs/calibration/2026-11-13-peel-off-backup.md)):
- **A called-off infielder backs up the catch** (agent-models §5.7 "peels off to back up", `PEEL_BACKUP_M` 5 m [E]): an infielder who believed he could catch a ball in the air and hears a sure call goes, unless a base needs him, to 5 m short of where his own read has it coming down, on his side; he keeps the job across re-calls until the ball is down. Being near the ball, he can take it on a re-call when the caller's read falls apart: seed 29 play 135 is now caught by the shortstop. 1,024 games: pop-up hits .018 → .008 (MLB ≈ .02) on seeds 5 and 2; runs 5.47 → 5.44 and 5.27 → 5.26. Not tuned; owner item 2 (the read) will move it again.

Next: owner item 2 (the read knows the ball started at the plate), then items 2–7 of the coordinator list (fouls first).

## Previous: 2026-11-13, second session: pop-ups; merged into main and pushed

`fix/popup-calls` (cut from main `454d341`) is merged into main and pushed to origin. `cargo test --workspace --release`: 143 passed, 0 failed. The owner paused here and will continue in a new session. Owner (2026-11-13): merge into main often.

**Nothing is waiting on a review.** The [Codex popup-calls review](docs/reviews/2026-11-13-codex-popup-calls-review.md) is resolved: its P3 (two numbers in the calibration note) is fixed, and the owner deferred both P2 findings (2026-11-13: "errors such as you describe can take place" in real games; fine once in a while, keep for a later fix):
- Later: when both middle infielders go for an uncalled ball, nobody covers second until the next re-call (seed 55 play 159; 17 such assignments in 1,024 games). Wanted: a fallback cover before backups are handed out.
- Later: a re-call looks only at the chaser and the four fielders nearest his target, so another active chaser can be sent to backup before anyone is sure (seed 897 play 163). Wanted: every active chaser is a re-call candidate.
- ~~Later: the gap-fly test's right fielder has only 30 ms of margin over the chase threshold; pick a more comfortable ball.~~ Done 2026-11-13 (fourth session: 0.65 s).

**Then, the owner's two items (2026-11-13), in this order, merging each into main when done:**
1. ~~The called-off infielder drifts out behind the caller instead of standing still~~ **done 2026-11-13** (third session, above).
2. ~~The read knows the ball started at the plate (§5.1 launch-state perception), and the outfield read values are refitted~~ **done 2026-11-13** (fourth session, above).

Done (old "Next" item 1, pop-ups dropping in short right/center; [note](docs/calibration/2026-11-13-popup-read-and-calls.md)):
- **Why the outfielder's read is off above 50°: not spin or drag** (under 1.5 m at any launch angle). One σ of his read moves the predicted landing point about 25 m at first look on every fly, nearly all of it the position-depth term, which scales with his distance from the ball. A fly hit toward him brings that down to under 2 m; a shallow pop-up never comes near him, so he is still 5–7 m unsure with a second and a half left. New diagnostic: `cargo run --release -p bb_game --example read_error -- 48` (the read is now a pure function, `live::read_prediction`; digests unchanged).
- **Calls (agent-models §5.7):** a fielder calls a ball in the air only when he is sure: his time to spare covers one σ (`CALL_SURE_SIGMAS` [E]) of his own read's depth error. Until someone is sure, everyone who believes he can catch it goes for it; a sure call sends the others to their cover and backup jobs. Outfielders still outrank infielders among sure calls. 1,024 games, seed 5: pop-up hits .047 → .018 (MLB ≈ .02), fly balls .134 → .122, runs 5.55 → 5.47; seed 2: 5.27 runs. Uncaught pop-ups in 128 games: 28 → 12; seed 41 play 93 (the review's example) is caught by the second baseman.

Open from it:
- ~~A called-off infielder still stands still under the ball (seed 29 play 135)~~ done (third session).
- The interim read gives the observer no knowledge that the ball started at the plate (22 m of depth error when the ball is 16 m from home). How fast a depth read should firm up belongs to §5.1's launch-state perception; changing it means refitting the outfield read values.
- Part B of the pitcher fix was already done (the pitcher backs up third, owner 2026-11-13).

Next, in order: items 2–7 of the list below (fouls first).

## Previous: coordinator run (2026-11-13; branch `coord/review-fixes`, merged into main)

All of this session's work was done on `coord/review-fixes` (cut from `origin/main` 638871c, PR #1) and merged into main at the owner's request. `cargo test --workspace --release`: 141 passed, 0 failed, after every merge.

**First, unresolved:**
- [Codex idle-fielder review](docs/reviews/2026-11-13-codex-idle-fielder-review.md) **resolved**: every way a fielder becomes Idle now goes through one helper (`go_idle`) that stops him (relay/cutoff retirement had the same bug); the calibration note's explanation of the dropped pop-ups was corrected. **Deferred:** the unconditional outfield call priority (see Next 1).
- [Codex rules-passing review](docs/reviews/2026-11-13-codex-rules-passing-review.md) **resolved**: all five P2 fixed (passes made while the ball is in doubt are held and judged when it is ruled fair or caught; a runner going back from a pending home touch can be passed; overrun protection starts when the batter-runner leaves first in a sampled play; one retreat can produce several passing outs; left on base counts runners, not occupied bases, which adds one LOB in 10 of 512 real games). Scenarios 94 → 106. Open from it: runners between bases at the third out count as neither run nor LOB (breaks box-score identities I1/I10; fixing it moves LOB in ordinary games, so it needs its own change); a batter-runner put out on a held pass before a home run lands still gets a home-run PA result.
- Owner decisions of 2026-11-13, all applied (see Done): players jog off/on at side changes (a camera cut comes later); the pitcher backs up third on bases-empty outfield balls; the bat-read error was re-derived; a further honest anticipation cue was added.
- [Codex owner-items review](docs/reviews/2026-11-13-codex-owner-items-review.md) of those four changes **resolved**: no P1/P2; its one P3 (native math and an ad-hoc RNG in the new diagnostic examples) fixed, tables regenerated (changes ≤ 0.08°).

Done this session (coordinator with worker agents; merges in `coord/review-fixes`):
- **[Anticipation review](docs/reviews/2026-11-12-codex-anticipation-review.md) resolved:** fielders read the bat's horizontal angle at the actual contact instant (plus noise), not the true launch direction, and map it to spray with their own fixed line (`examples/spray_by_bat_angle.rs`); plans before a fielder's reaction gate step him through the rest of his jump; circular angle fusion, with cues more than 90° from straightaway ignored; the cover-job guarantee was dropped from §5.10. Grade 50 now saves 0.07 runs/team-game against no jump (was 0.16). [Report](docs/calibration/2026-11-12-anticipation-review-fixes.md).
- **[Reset staging review](docs/reviews/2026-11-12-codex-reset-staging-review.md) resolved (client):** outgoing players get their own 24 figures and walk off to the dugout; the first pitch is requested off the game thread ("Warming up" until ready); short returns sidestep or back-step from the planned move; the return-throw clip phase comes from its release time, so 16× playback no longer skips it. New `-BbStagingLog` flag. Built and checked headlessly; **not watched**.
- **[Codex coord review](docs/reviews/2026-11-13-codex-coord-review-fixes-review.md) resolved:** no P1/P2; both P3 fixed (the projected jump state is timed by the tick that produced it; the example uses `bb_math::m`).
- **Rules engine (rules-spec §5.4, §5.5):** overrunning first (OBR 5.09(b)(4)), two runners on a base (5.06(a)(2), (b)(2)), passing a preceding runner (5.09(b)(9)), a forced runner tagged on a base short of his forced base (force-004). Scenarios 64 → 94 (106 after the review fixes). Real games unchanged (identical digests over 512 games). **Simulator side not done:** the live sim does not emit `RunnerPosition` samples (so overrun protection never starts and passing is judged from base touches only) and the batter-runner never actually overruns first.
- **[R13 foul research](docs/research/r13-foul-direction.md):** MLB foul share of contact is ~51%, and ~40% even for well-timed contact; well-timed fouls are mostly steep undercut balls (69–88% foul above 50° launch). The gap is inside the timing window, not at its edges. Owner (2026-11-13): accepted; "ball/bat contact is two curves colliding at weird angles so foul balls should be fairly common."
- **[Pitcher in the outfield](docs/calibration/2026-11-12-pitcher-outfield-diagnosis.md): Part A fixed.** A fielder set Idle kept his own intercept as his target; now he pulls up where braking stops him (`cover_roles`). Flagged plays 13.7% → 0.3% of balls in play (the rest are balls hit behind home). Part B (the pitcher's job on bases-empty outfield balls) waits on owner decision 2; until then he stays near the mound. `cargo run --release -p bb_game --example pitcher_roam -- 1 200`.
- **It uncovered a hidden bug: pop-ups drop.** Runs 5.27 → 5.48 per team-game, PU hit .005 → .044 (MLB ≈ .02), FB .122 → .131 ([note](docs/calibration/2026-11-13-idle-fielder-target.md)). The called outfielder misjudges the depth of pop-ups landing 55–65 m out in short right and center and keeps the call (outfielders outrank infielders); the infielders who used to drift there by mistake caught them. Not tuned.
- **Side changes (owner, 2026-11-13):** outgoing players jog to their dugout and the incoming fielders jog out of theirs, at 4.8 m/s with the jog clip played at its own root speed (6.0 m/s, measured with `-BbClipSpeeds`), so in-play jogs/sprints now animate ~17% slower; side changes take 23–27 s of play time (median 23.5 s, seed 7). Walk clips still skate a little (root 3.0 m/s, played as 2.3). Not watched yet.
- **The pitcher backs up third on bases-empty outfield balls (owner, 2026-11-13):** median 0.4 m from his backup spot at the end of those plays (was 29.5 m, on the mound). Runs unchanged.
- **Bat-read error derived (owner, 2026-11-13; [note](docs/calibration/2026-11-13-bat-read-derivation.md)):** the fielder must pair the bat's angle with the moment of contact; human timing precision (40 ms at grade 50, Linares et al. 2009; Brenner & Smeets) × the bat's 1,915°/s yaw rate gives 77° at grade 50 (39° at 80), about 5× the old guess. The bat read is now nearly worthless.
- **A further honest anticipation cue (owner, 2026-11-13; [note](docs/calibration/2026-11-13-anticipation-cue.md)):** the swing's lead (when the swing started relative to the ball's arrival), fused with the bat read as a second view of the bat face; implausible reads are discarded and the fielder falls back on his other reads instead of standing still. **Finding: no honest cue makes Anticipation grades rank** (4,096 games, seed 5: no jump 5.55, grade 20 5.56, 50 5.58, 80 5.51 runs/team-game). The called pitch carries no spray information here, and every timing view is limited by the fielder's 20–80 ms timing error against the batter's 4–8 ms timing spread. For Anticipation to matter, the information must exist first: batters whose timing is really fooled by offspeed and sequence (pa.rs / bb_agents), which would make the called pitch an honest pre-pitch cue.
- CLAUDE.md no longer says the repo has no remote.

Next, in order:
1. ~~Pop-ups dropping in short right/center~~ **done 2026-11-13** (see "Resume here").
2. ~~**Fouls (O1/R13)**: extend `foul_diag` with Statcast-style measures and add a `cage-map` bat-tilt counterfactual before changing any calibration~~ **measured 2026-11-13** (fifth session, above). Its next step is parked: "Foul share: the spray experiment" under Parked.
3. ~~Live sim emits `RunnerPosition` at ~30 Hz and models an overrun of first.~~ **Done 2026-11-13** (sixth session).
4. Watch the client (`bin/play.sh`): reset staging, side-change jogs, sidesteps.
5. **Camera cut at side changes** (owner, 2026-11-13: later) to hide the 23–27 s jog off/on.
6. **Batter timing fooled by offspeed and sequence** (pa.rs/bb_agents), which is what would give fielders' Anticipation real information to read.
7. **The owner's priority A backlog** ([Owner backlog](#owner-backlog-observations-of-2026-11-13-merged-with-the-wishlist-of-2026-11-12) below, 17 items, in the order the owner agreed on 2026-11-13). It comes after items 1–6 (owner, 2026-11-13). Item 4 is partly answered by the observations themselves; "batters of both sides" (previous handoff) is now A1 and the foul-territory walls are A10.

Process (owner, 2026-11-13): at most two Claude worker agents at once, and every worker commits after each meaningful step; five at once hit the usage limit and one worker's uncommitted work had to be rescued.

## Previous handoff (2026-11-12, fourth session: runner return; all branches merged)

**First, unresolved review findings** (both resolved 2026-11-13, see above):
- [Reset staging review](docs/reviews/2026-11-12-codex-reset-staging-review.md) (client, all deferred): (P2) outgoing players vanish at side changes, because the 16-slot actor pool is shared with the next pitch; (P2) the first pitch resolves on the game thread in `BeginPlay`; (P3) very short returns play idle or face backward; (P3) at high playback speed the return-throw clip can be skipped.
- The Anticipation review below (uncommitted in the main checkout, Resolution open; another session owns it).

Merged into main this session: `worktree-runner-return-on-catch`, `worktree-owner-wishlist` (W1–W13 below), `worktree-batter-presentation` (one conflict in `BbDirector.cpp`: the HUD call now waits for max(`call_s`, the umpire's last out call)), `worktree-pitcher-reset-walk`. None of the client changes has been watched yet.

Done (day-fixes review #1, P1, open since 2026-11-11):
- **A runner who crossed home before a catch goes back** (rules-spec §5.3). The rules engine cancels his pending run when he retouches, and his next home touch scores. The runner agent sees the catch, runs back as a live runner (taggable, appealable, a throw target) and decides again. Rare in games: in 2,000 games, 79 runners crossed home before a third-out catch (no run, nothing owed) and 2 before a catch with the inning still going; both went back (`--example home_before_catch`).
- [Codex runner-return review](docs/reviews/2026-11-12-codex-runner-return-review.md) **resolved**: a walk-off no longer ends the game on a home touch made while the fly is still in the air (§4.1). A preceding runner retired on appeal for the third out cancels the following runners' runs (OBR 5.09(d), §6.4 corrected). Fourth-out appeals are still not modeled.

### Previous handoff (2026-11-12, third session: umpire calls)

**First:** the [Codex review of the Anticipation commit](docs/reviews/2026-11-12-codex-anticipation-review.md) is finished but uncommitted and unresolved (see the second-session handoff below). The [Codex review of the umpire calls](docs/reviews/2026-11-12-codex-umpire-calls-review.md) is resolved: two client scoreboard-timing bugs fixed (strike three showed before the call; the third out was skipped straight to the next half).

Done (owner request: "when an out is made the umpire should make a call right away; within 1 s for non-close plays, almost right away when close"):
- **Umpire out calls** (rules-spec §1.3, data-formats §5.4): every out gets an `UmpireCall` timeline marker at the moment the umpire signals it: 0.2 s after a bang-bang out (runner within 0.15 s of the bag, a diving or shoestring catch, strike three), rising to 0.7 s on routine plays. The client shows "OUT at first" / "OUT!, caught" (red and larger when close) as it happens and advances the outs on the scoreboard; the play's summary still appears when the clip ends. `bb_game::umpire`; `BbPlayInfo.calls`; `cargo run --release -p bb_game --example umpire_calls -- 20`. Observation only: game results are unchanged.
- Not done: no umpire figures on the field (the call is HUD text); safe calls on close plays are not signalled.

## Previous handoff (2026-11-12, second session)

**Owner watch fix (2026-11-12, third session, branch `worktree-batter-presentation`):** the owner saw the batter turn around before the pitch on swings, and a walked batter start walking during the pitch. The sim's decision was already causal; it was a timeline presentation problem (`bb_game::timeline`): the bat track began at the swing's laid-back start pose (the client holds the first sample, and turns the body after the bat), and award jogs had no hold knot (root tracks interpolate linearly, so the batter drifted from release). Now every pitch opens in the same stance and leaves it only in the 50 ms motor latency before the swing; on a walk or hit batter nobody moves until the umpire's call (pitch end + `T_signal` 0.2 s) plus 0.4 s. A `SwingStart` marker is emitted. Test `nothing_is_given_away_before_the_decision_or_the_call` (bb_ffi) fails on the old code for both. [Codex review](docs/reviews/2026-11-12-codex-batter-presentation-review.md) **resolved**: the load never starts before the batter's real decision (`PitchRecord::t_decide`); the HUD no longer shows exit velocity during the wind-up or the call before the umpire's (`BbPlayInfo::contact_s`/`call_s`); held runners do not turn early. The client builds; it has not been watched yet. **Open from that review:** the executed swing often starts sooner than the 50 ms motor latency after commit (690/1,114 swings), a causal-model question in `pa.rs` that would change outcomes; the antipodal bat-axis blend. Still a stand-in: the walked batter keeps the bat in his hands (no drop).

**First:** the [Codex review of the Anticipation commit](docs/reviews/2026-11-12-codex-anticipation-review.md) (`74679a6..cdd473c`) was started at the end of the session. If it is in `docs/reviews/`, commit it, fix or defer each finding, and write its Resolution. Its first question: whether reading the swing from the true launch direction plus noise is honest perception (the alternative is reading the bat's yaw from the swing record).

Done this session (owner request: fielders "get a jump" by anticipating from pitch location and batter handedness, with a rating):
- **Fielder momentum** (`21a4535`, agent-models §5.3): velocity is a vector; braking at most 6 m/s², turning at most a_lat 7 m/s²; fielders slow into turns they cannot make. Until now they turned and reversed instantly at full speed, which hid six planner bugs, now fixed: infielders broke for pop-ups as if grounders; a breaking infielder ran at his base; fly balls went to the earliest intercept (a coin toss); the re-call looked near the ball's current position; calls were traded on noise; fielders assumed 1,800 rpm backspin where the contact model gives flies 3,300–6,500 (now a table by launch angle). Refit: velocity read 0.06 → 0.04, OF read 2.5 → 3.0. [Report](docs/calibration/2026-11-12-fielder-momentum.md).
- **[Codex momentum review](docs/reviews/2026-11-12-codex-momentum-review.md): resolved** (`74679a6`): arrivals no longer stop a fielder dead, planting distance, a turn-aware reach estimate (tested against the stepped movement), receivers not pinned to the bag, fresh-read breaks, wall caroms.
- **Anticipation** (`cdd473c`, agent-models §5.10): the fielder combines the pitch location and batter's side (a prior) with a read of the swing at contact, and starts toward the side he expects at contact instead of standing through his reaction time. Leaning on location alone made fielders *worse* (7° of signal against 22° of scatter); the swing read is what pays. 2,048 games: no jump 5.35 runs/team-game, grade 20 5.31, 50 5.22, 80 5.03. `sim-cli games --anticipation <grade>` / `--no-jump`.

Owner report (2026-11-12, late): "the pitcher looks like he's skating or flying back to the mound after the play." Cause: the client's between-pitch staging held the reset to 4.5 s, so a pitcher far from the mound was moved at distance / 4.5 s: median 5.5 m/s, up to 16 m/s over three games (seeds 7–9), faster than the jog clip plays and, above 14 m/s, shown in the idle pose. **Fixed in the client** (`dc870d7`, merged): walk / easy jog, quickening to at most 5 m/s, with acceleration at both ends; the reset lasts as long as the slowest returner needs (6–7 s for a pitcher 20–25 m out, ~16 s from 70 m). Compiled; not yet watched. Reviewed by Codex together with the earlier staging work: findings deferred (see "Resume here").
- **Pitcher in the outfield (sim bug, open; diagnosed 2026-11-13, [note](docs/calibration/2026-11-12-pitcher-outfield-diagnosis.md)):** on fly balls and hits to the outfield the pitcher ends the play sprinting (7.9 m/s) as far as 50–70 m out, e.g. (54, 55) on "Fly out to right" (seed 7, play 1) and (21, 86) (seed 7, play 195). He is neither the called chaser nor at a base backup spot, so some other job (relay, re-call, or `back_up_throw`'s fallback) sends him there. A real pitcher backs up third or home. Find which role, then fix the planner.

Next, in the order a viewer would notice:
- **Watch it in the client** (`bin/play.sh`): fielders now carry momentum, plant after catches, curve into turns, and move at contact. Nothing in the Unreal client changed; check it still looks right.
- **Batters of both sides in games.** Every game batter is right-handed (`Matchup::average()`), so the handedness half of Anticipation only runs in unit tests. Needs per-slot batters and a check that pitcher targeting handles left-handers.
- Run environment 5.22 runs/team-game (MLB ≈ 4.5): ground balls still go for hits at .40 (MLB .24); infield hits are 20% of grounders, mostly high choppers from the contact model's launch-angle spread. Turning is still the largest movement cost (runs would be ~4.4 if turns were free); the atlas perception of §5.1 should reduce replan target jumps.
- One-hop throws; the motion stand-ins (no catching motion, bat not in the batter's hands).
- Still open from reviews: foul share 24% vs 50%; foul-territory walls. (Day-fixes #1, the runner who crossed home before a catch and never went back, is fixed and merged, 2026-11-12; see that review's Resolution.)

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

- [?] **O1. Chase rate.** Average hitters chase 11.6% (MLB 29%). The owner approved trying perceptual overconfidence (2026-11-11); it was implemented, measured, and backed out: even total overconfidence gives 18% and wrecks contact. Diagnosis in [docs/calibration/2026-11-11-chase-diagnosis.md](docs/calibration/2026-11-11-chase-diagnosis.md): pitch locations are fine and the batter's strike belief is already uncertain near the edge; the swing decision is what is too sharp, and the low foul share (21% vs 50%) is what keeps K% and pitches/PA low. The owner chose to fix fouls first (2026-11-11). **That is not done:** [docs/calibration/2026-11-11-foul-share-diagnosis.md](docs/calibration/2026-11-11-foul-share-diagnosis.md) shows no noise setting gets the foul share past ~30% (timing errors turn into whiffs, glancing contact stays forward), so the gap is structural. R13 is done ([note](docs/research/r13-foul-direction.md), 2026-11-13). The measurements ([note](docs/calibration/2026-11-13-foul-measures.md), 2026-11-13) found that well-timed steep contact is already foul at MLB rates. The gap is low-angle contact, and mistimed swings that miss over or under the ball where MLB batters foul it off. Next: a timing-against-height sweep and a cage counterfactual of §3.6's hand path, then the swing change, pending the owner. Discipline's grade-50 values (about c_D 0.5, b_D 0.9; currently 1.0, 0.0) stay parked behind that. `cargo run --release -p bb_game --example foul_diag -- 3000 [timing ms] [bat-control cm] [K_perception]`. Tools: `sim-cli pa` prints swing rate by miss distance; `cargo run --release -p bb_game --example chase_diag -- 3000 [c_D] [b_D]`.
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
- [x] R13 ([docs/research/r13-foul-direction.md](docs/research/r13-foul-direction.md), 2026-11-13; the pull/opposite split stays open: Statcast has no foul coordinates). MLB fouls by direction (pull side, opposite side, backward) and a published oblique bat-ball collision data set (does glancing contact grip or slide?), for the foul-share gap (calibration report 2026-11-11)
- [x] R12. Rules-spec §15 item 2 additions are in the M0.5 schemas (2026-10-31)

### M0 leftovers
- [ ] M0.1b Install Git LFS before any Unreal asset is committed (`.gitattributes` already routes `*.uasset`/`*.umap` to LFS). Needs `sudo dnf install git-lfs` or the toolbx
- [ ] M0.2b Optional: `sudo dnf install gcc glibc-devel` on the host to retire the zig linker shim (see CLAUDE.md)
- [x] M0.8 **Unreal spike** on this machine: passed 2026-11-11 ([docs/unreal-spike.md](docs/unreal-spike.md)). Engine at `~/UnrealEngine/5.8.4` (outside the repository; the 40 GB zip in the project folder can be deleted)

---

## Owner backlog (observations of 2026-11-13, merged with the wishlist of 2026-11-12)

Source: [observations.md](observations.md), written by the owner after watching games. Priorities A, B, C are the owner's. This list replaces the old wishlist; its numbers are kept in brackets (`[W2]`). "Have" says what already exists, checked against the code on 2026-11-13. Within a priority the order is by theme, not rank. Items were renumbered once on 2026-11-13 after the owner's rulings (looks moved to the polish stage, errors moved to A). When an item starts, move it into its milestone and add detail.

### Priority A

On the field (sim):
- [x] A1. **Handedness for every player** (2026-11-13; data-formats §2.4, agent-models §5.5, [note](docs/calibration/2026-11-13-handedness.md)). Hands in the roster, switch hitters, batters and pitchers of both sides in games, throws from the throwing hand, left-handers mirrored in the client. Left for A3: what the glove hand does to fielding and to the footwork of a throw.
- [x] A2. **Rosters: player names, numbers, positions** [W9, part of W5] (2026-11-13; data-formats §2.4, `bb_game::roster`). Nine named players a side from the game seed; names in captions, the event log, the box-score lines and the C ABI. Team names and benches wait for the league (B1); the client does not show numbers yet.
- [x] A3. **Throwing ratings and hurried throws** (2026-11-13; agent-models §5.5, [note](docs/calibration/2026-11-13-throwing.md)). Set-up time with the turn and footwork, hurried and off-balance throws, rating grades. Left: per-player ratings (population generator), the backhand (A17). Original entry: Each player has ratings for arm speed, accuracy, and set-up time. A hurried throw loses accuracy and speed; a fielder with time to get balanced gains both. Have: the spec's `ArmStrength`, `ArmAccuracy`, `Transfer` and its rush and off-balance noise terms (agent-models §5.5); in code each fielder has an arm speed and a fixed transfer time. Missing: no throw is ever hurried (the rush terms are not built), and the spec gives a rushed throw no speed loss, so §5.5 needs that added.
- [~] A4. **Smarter throw choices from the outs and the runners.** The owner's example: runner on third, two outs, the fielders should take the force at first or second, not throw home. Collect plays where the choice is wrong before changing the option evaluation (agent-models §5.5).
- [~] A5. **Dives that extend range, with a real cost to get up and throw.** Have: chasers dive (up to 1.5 m extra reach, 1.5 s to get up, harder the further out), the client shows the dive and the get-up with stand-in clips, and since 2026-11-13 the rate is believable: about 3 a game against 22 before and 2 to 3 in real games ([note](docs/calibration/2026-11-13-dive-rate.md)). A ball just past reach is taken with a stretch on his feet; outfielders dive only when they like their chances (this was "A18"). Missing: real dive and get-up animation, a stretch animation, and the owner watching it.
- [ ] A6. **Runners slide** when the play calls for it (agent-models §6.2; was in M5). Not built.
- [ ] A7. **Windup vs. stretch** [W2]: a full windup with the bases empty, the stretch with runners on; the stretch shortens the delivery, which matters to steals. Owner's references: [Am J Sports Med 2024 study](https://journals.sagepub.com/doi/abs/10.1177/03635465241247543), [Umpire Bible: pitching positions](https://umpirebible.com/pitching-positions/), and a search for "pitching stretch vs windup visualization". Not built (one delivery; agent-models §2.6 sketches the slide step).
- [ ] A8. **Runners lead off only once the pitcher is set; pickoffs.** The owner saw runners leading before the pitcher was on the mound, where a real pitcher would pick them off. Have: fixed leads of 5.5 / 6.5 / 4.5 m from first, second, third (`LEAD_M` in `live.rs`). Missing: lead timing tied to the pitcher coming set, leads chosen from the runner's belief of pickoff risk, and the pickoff itself (agent-models §2.6, §6.1).
- [ ] A9. **Stolen bases** [W13]. Needs A7 (delivery time), A8 (leads, jumps, pickoffs), A6 (slides), and the catcher's throw (agent-models §4).
- [~] A10. **Limits of the play area.** The owner saw a foul ball go out of play and the play carry on. Build the foul-territory walls and backstop (physics-spec §9.4; this is "Run environment (a)" above: foul ground now reaches the outfield-fence radius), make a ball that leaves the park dead (with the rules award for a thrown ball out of play), and have the client cut back to the field once the ball is gone.
- [ ] A11. **Wind and weather** that act on the ball and the players. Have: the flight model takes a wind vector (always zero) and an air density. Missing: weather per game, and rain. Rain must act through physical causes (D1): a wet ball (release and throw noise), wet ground (traction, bounces), not a mistake rate. **Was deferred past M8; the owner moved it to A on 2026-11-13** (PLAN §14 updated).
- [ ] A17. **Fielding errors** [W10; owner made it a priority, 2026-11-13], from physical causes only (a bobble, a bad throw; D1). Ties to the scorer's hit-or-error ruling (rules-spec §10), A3, A11, and B5
- [~] A12. **Post-game box score.** Have: `bb_stats` builds runs, the linescore, PA results, and LOB from the event log. Missing: hits vs. errors need the scorer (rules-spec §10; see "M4 next steps"), per-player lines now exist by name (`BoxScore.batting`: PA, R, K, BB, HBP, HR) but have no at-bats or hits until the scorer, and the client has no box-score screen.

On the screen (client). The owner (2026-11-13): hitting and throwing motions affect gameplay, so they stay here; the other looks wait for the polish stage (below).
- [ ] A13. **Fielders' throwing motion** does not look real [W8] (a stand-in from the engine's stock clips).
- [ ] A14. **Hitting animation:** stance, load, swing, follow-through (also a stand-in). Per-player stances and swings are L3 below.
- [x] A15. **Ball shadow after a hit cannot be seen.** Done 2026-11-13: the disc under the ball is sized by the camera (`ABbDirector::PlaceBallShadow`). With the ball on the ground it is at least 15 cm, and far away at least 0.7% of the view's width (`BallShadowViewShare`). It is up to 2.5× longer along the line of sight (`BallShadowMaxStretch`), since the follow camera sees a far disc nearly edge-on. It then shrinks to about half as the ball climbs, as before. On a 1600 px window a deep fly's shadow on the ground goes from 2 × 0.2 px to 11 × 3.4 px. Nowhere is it larger on screen than the 50 cm disc the owner rejected. Checked in headless shots of seed 7 (docs/unreal-spike.md) and by the [Codex review](docs/reviews/2026-11-13-codex-ball-shadow-review.md), resolved. Not yet watched by the owner in a window. The drawn ball itself (15 cm) is still about 3 px on a deep fly.
- [~] A16. **Home runs:** once the ball is over the wall, cut back to center to show the runners circling the bases; show a display with the batter's name (A2), the distance, and the exit speed in mph.

### Priority B

- [ ] B1. **Leagues, divisions, teams with rosters** [W5]: team names, mascots, full rosters. M7.
- [ ] B2. **Lineups by pitcher hand:** separate batting orders against left- and right-handed pitchers (needs A1; a manager lever, M6).
- [ ] B3. **Game settings screen** (includes team selection [W6]).
- [ ] B4. **Player and team statistics** from the event log [W11] (`bb_stats`).
- [ ] B5. **"Fielder's choice" is used too freely, and needs explaining.** Today the play-by-play says it whenever the batter reaches base on a ball in play that is not ruled a hit (`describe.rs`, `classify_bip`, after OBR 9.05). With no errors in the sim (W10) and a provisional scorer, it also covers plays a real scorer would call a hit or an error. Audit a few hundred such plays against 9.05, fix with the scorer work, and make the text say what happened ("reached on a fielder's choice, runner out at second").

### Priority C

- [ ] C1. **Better crowds** (after L1).
- [ ] C2. **Player development** (includes aging: a career arc for skill and experience [W7]). M7.
- [ ] C3. **Post-game highlights** (timelines are already recorded, D9).

### Polish stage (M8; owner, 2026-11-13: these wait, D13 stands)

- [ ] L1. **Stadium and diamond that look real** [W1]. Crowds can be something cheap for now (better crowds are C1).
- [ ] L2. **Players' look:** a variety of body shapes, skin colors, ethnicities and nationalities (names in A2 should match); team uniforms (teams are a tint now); gloves, on the hand given by A1 (the glove hand still matters to the sim under A1).
- [ ] L3. **Batter stances and swings per player** [W3], with a wide variety. The owner's examples: Pete Rose, Darryl Strawberry, Gary Carter; a longer list may follow. Follows A14
- [ ] L4. **HUD** [W4] that looks more like a TV broadcast
- [ ] L5. **Umpires with animations** [W12] (out calls exist as HUD text, `bb_game/src/umpire.rs`; no umpire figures)

### Order for priority A (owner agreed, 2026-11-13)

1. A2 rosters and A1 handedness: nearly everything else hangs off them. (Both done.)
2. Quick client fixes alongside: A15 shadow, A16 home-run camera and display.
3. A10 play-area limits (also a known run-environment item).
4. A3 and A4 throws, with A5 measured in the same pass, then A17 errors (they come out of rushed throws and bobbles).
5. A8 → A7 → A6 → A9: leads and pickoffs, the stretch, slides, then steals.
6. A12 box score (with the scorer, which also settles B5).
7. A11 wind first (the flight model is ready), rain after.
8. A13 throwing motion and A14 hitting animation.

---

## Later milestones (detail added when each one starts)

- [ ] **M5** Deepen the field
- [ ] **M6** Live management (primary experience)
- [ ] **M7** League and season
- [ ] **M8** Presentation polish. **Revisit the RAM upgrade before this.**

Exit gates for each are in [PLAN.md §14](PLAN.md).

---

## Parked (decided "later")

- [ ] **Foul share: the spray experiment** (owner, 2026-11-13: "add a task potentially for this at a later time"). The foul share is 23.7% of contact against MLB's ~51% (B1.15, O1/R13). This blocks the chase rate (Discipline's grade-50 values), and so contact rates and Adjustability as well. Read [the note](docs/calibration/2026-11-13-foul-measures.md) §5–§8 and its three Codex reviews first.
  - **Ruled out or weak so far:**
    - well-timed steep contact (already MLB-like);
    - the fair/foul call (games agree with the PA model);
    - mid-swing correction: saved swings come out mostly fair, 23.7 → 26.1% even with no limits;
    - bat angle alone: −30° matches Statcast tilt but gives only +4 points and overshoots steep fouls.
  - **The open hypothesis:** v1's radial bat about a fixed pivot may hold the ball's spray too close to the swing's attack direction. MLB contact at AD 12–20° fouls 54%, the sim's 8–12%.
  - **Steps:**
    1. Measure the sim's spray-minus-AD distribution, by AD and outcome, in `foul_diag`.
    2. In the cage, build a counterfactual swing whose bat face and path can differ (§3.6's hand path and lag, or its simplest version, with the bat angle remeasured), and compare spray minus AD and the foul share by AD at fixed pitch and aim.
    3. Only then change the PA model's swing, with refits of the run environment, the fielders' spray reads (`spray_by_bat_angle`, `LEAD_FACE_*`) and the client's batter-box position.
  - **Second question:** on-time topped balls stay fair (MLB fouls 62% of well-timed contact below −30°); needs evidence (plate, dirt, batter) before any Tier A change.
  - **Related:** coordinator item 6 (batter timing fooled by offspeed and sequence).
  - **Tools:** `foul_diag` (with `adj=`/`cut=`/`phase=` and a bat-angle argument), `foul_in_games`, `sim-cli cage-map` (timing × height grid, bat-angle counterfactual).

- Injuries, physically grounded (D18), after M8
- Seam-shifted wake, human-umpire mode, swing styles (W3), scouting inferred from behavior, online play
- Weather and wind left this list on 2026-11-13 (owner priority A, backlog A11)
