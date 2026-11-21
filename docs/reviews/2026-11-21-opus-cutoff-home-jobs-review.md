# Review: the infielders' jobs on a ground single with a runner in scoring position (2026-11-21, Opus)

Branch `work/cutoff-home-jobs`, range `312789e..687102d` (77b0431, 08d8cf6, e56ddc5, 687102d). Reviewer: Claude Opus, wrote none of the change. Reviewed against AGENTS.md's invariants, agent-models §5.7 and §5.5, and the worker's note `docs/calibration/2026-11-22-cutoff-home-jobs.md`.

**Summary.** No P1. Invariants 2, 5 and 8 hold; the isolation harness really covers the new private state (a planted leak is caught). The two new tests are independent of the code and fail on `312789e`'s `live.rs` as claimed. The note's headline figures reproduce, and they hold on fresh seeds 120–239 (2B no-job time 0.336 → 0.112 s a play, SS 0.473 → 0.107). One P2: the claim that no early cutoff home happens on a ball an infielder fields is not true. The worker counted only the first loose ball. Counting every loose ball, 29 plays in 240 games have one, all after the ball came off an infielder who then ran it down. On 16 of them the first baseman leaves first with nobody on it for 0.5–2.75 s. Five P3s.

## Findings

### P2-1. Early cutoff home on a ball an infielder runs down after it comes off him; first left empty for up to 2.75 s. The note and spec say this never happens

- **What.** `through_job` toward home (live.rs:5274–5301) lets the cutoff man go out while an infielder is the known lead, as long as the ball has gone by that man (`infield_lets_go`, live.rs:5312–5346: a known chaser counts only while `!ball_gone_by(j, margin)`). After a deflection or knock-down, the infielder chases the ball from behind. The ball has "gone by" him, and he leaves the estimate, yet he is the one who fields it. In these 29 plays he fields it at 50–57 m, just past `INFIELD_BREAK_R`. The 1B then leaves first while the 2B, the only man `home_order(true, false, _)` gives first to, is the one chasing. Nobody covers first.
- **Where.** live.rs:5274–5301 (`through_job`, `4 if … => (home, 4)`), live.rs:5326 (the `ball_gone_by` exemption), live.rs:5621–5627 (`home_order`: first = `[3]` only). Note §3 ("Early cutoffs … of them fielded by an infielder: 26; **0**"). agent-models §5.7 "*Not changed:* … balls an infielder fields (none of these jobs: the gate and the judgment both hold while an infielder can field it)".
- **Evidence.** The note's 0 comes from `an.py`, which looks only at rows where `since == rows[0]['since']`, the first ball state. My `early_all.py` counts every loose ball and finds early cutoffs home (Relay(4), no outfielder lead) on balls an infielder first secured:
  - **Seeds 0–119** (worker's `tr-final.log`): 20 plays. Base states 1-- 8, -2- 5, 12- 4, --3 2, 1-3 1.
  - **Seeds 120–239** (my run): 9 plays.

  `eif.py` lists them all. Every one is on a second loose ball, after the ball came off the infielder. First stood empty with the 2B chasing:
  - seed 8 play 208: 1.80 s
  - seed 6 play 84: 2.40 s
  - seed 72 play 169: 2.30 s
  - seed 86 play 122: 1.83 s
  - seed 96 play 152: 2.57 s
  - seed 104 play 136: 1.70 s
  - seed 143 play 228: 2.75 s
  - seed 178 play 226 (runners on second and third, none out): 2.67 s
  - seed 212 play 46: 1.42 s
  - plus 0.5–1.2 s on seven runner-on-first plays

  On the 3B's side, third was empty 0.3–0.6 s, with the pitcher taking third. On every one of the 29 the outcome (P line: outs, bases reached) is identical to main. The ball is secured at 4–6 s, too late for a play on the batter, so nothing was lost here. Trace of seed 8 play 208 (runner on second, one out), final: the 2B chases from 0.70 s. A new ball state at 1.98 s. The 1B goes out on Relay(4) at 2.28 and returns at 4.08, first empty all that time. The 2B secures at (16, 49) at 5.25 s. On main the 1B stays on first throughout.
- **Would a real player do this?** Leaving for the cutoff once the ball is plainly rolling into the outfield grass with a runner scoring is defensible. But the man who knocked it down is still going after it and could still make a play at first, and a real first baseman holds the bag until that is settled. The spec's claim, the note's 0 and the "Early cutoffs" row are wrong as written.
- **Suggested fix.**
  - *Either* toward home, count an infielder known to be chasing as able to field the ball whatever the ball's position relative to him. Concretely, in `infield_lets_go` for `b == 4`, block on any known infield chaser of this ball rather than only those in front. The cutoff man then goes out once an outfielder is the known lead or the infielder stops. Measure the cost on clean singles: an infielder who dives and misses stops quickly.
  - *Or* keep the behaviour, and correct the note's row and the spec's "Not changed" sentence to say it happens on a ball an infielder runs down beyond `INFIELD_BREAK_R` after it came off him, with the count above.

  Either way, the note's counts should use every loose ball.

### P3-1. The first baseman's take-back of first (`one_back`) is broader than documented, and it reaches nobody-on plays

- **What.** `one_back` (live.rs:5750) applies to *any* first baseman who is Idle or Backup while the 2B is known on Cover(1), and no infielder holds the ball. It does not require that he was on the cutoff. "No infielder holds it" also includes a thrown ball and a ball loose in the infield. The code comment, the note §2 item 5 and agent-models §5.7 describe something narrower: "a first baseman whose cutoff job has gone to a deep ball's relay man while the ball is out of the infield".
- **Evidence.** `firstdiff.py` compares main and final per game, up to the first differing play. Of 120 games (seeds 0–119), two first diverge on a nobody-on play: seed 32 play 33 and seed 77 play 10. Both differ by this take-back after a chase with no batted-ball touch: on final the 1B takes first from the 2B and the 2B drops to no job. Of the 54 first divergences on nobody-on or runner-on-first plays, `firstdiff2.py` explains every one by the cutoff home (runner seen going home: 51) or this take-back (3). None is unexplained. With nobody on, the take-back signature (`oneback.py`) occurs on 8 plays on final against 4 on main. These changes are mild and arguably better (on main, seed 32 play 33 had first empty while the 1B stood idle). The take-back also pulls the 1B into the pre-existing deep-ball relay flicker: seed 20 play 85, seed 53 play 8 and seed 118 play 162 swap the cutoff between the 1B and 2B every 0.05–0.5 s. Main had the same flicker but left first empty instead.
- **Suggested fix.** Say what the code does: any 1B with no job takes first back while no infielder holds the ball. Say so in the comment, §5.7 and the note, and add the nobody-on effect to the note's §3. Or narrow the code to a 1B who was on the cutoff this play. Narrowing it to keep `a_retired_cutoff_man_with_no_job_stops` passing is acceptable ("once an infielder has the ball the bases stand" is a fair convention). But the stated reason, "out of the infield", is not what the code tests. Leave the flicker with the deep-relay item.

### P3-2. Second uncovered 10 → 22 (seeds 0–119), 10 → 19 (seeds 120–239): the worker's breakdown checks out; leave it with the relay review

The worker's breakdown is right (`unc.py -l` on both logs). 9 of the 22 are old plays: both middle infielders going for the ball, or a liner with a middle infielder out as the deep relay man. 10 are the one new kind: a single past the centre fielder makes the ball deep, the SS leaves second as the relay man, and the 2B, at first after the hand-over, learns 0.3 s later. These are seed 26 play 11, 32/126, 39/56, 46/87, 47/240, 56/191, 61/181, 77/185, 86/73 and 117/22. Each lasts at most 0.30 s, after 3.4 s, with the ball 80–120 m out. 3 are hand-overs: seed 9 play 9, seed 69 play 259, seed 117 play 38. In 7 of the 10 the batter ran through second while it was empty. No throw could arrive in that window, so no play is lost, and a real infield would also be moving the 2B over at that moment. No cheap fix fits this change: the 2B cannot rebuild `relay_job`'s deep-ball man from public facts, because it uses the relay man's own read (`no_air_play`, `int_pt`). Deferral as the note says is right. Put a line in TASKS so it reaches the measured relay review.

### P3-3. The cutoff man home given by `relay_job` still leaves his base while a ground ball is loose in the infield (pre-existing; this change moves it from first to third on balls the LF is going for)

`relay_job` applies `infield_lets_go` only to middle infielders (live.rs:4991, `(who == 3 || who == 5)`). Once an outfielder is the known lead, the corner man goes out on the cutoff home while an infielder may still field the ball:
- **Seed 191 play 125** (bases loaded, two out): the 3B is off third 1.00–1.75 s and third is empty 0.6 s while the SS fields the chopper at 43 m. Main had the 1B off first there instead.
- **Seed 217 play 104:** the 1B is off first 0.96–1.71 s; the SS fields at 43 m and gets the batter at first.

The counts are the same on main (`cornerif.py`: 4 plays in seeds 120–239). Not caused by this change, but now that the cutoff home is a convention, the corner men toward home should get the same gate toward home that `through_job` has. Defer with a TASKS line (TASKS §3 case (b) territory).

### P3-4. Dates: the note's filename and other text are dated 2026-11-22; today is 2026-11-21

`docs/calibration/2026-11-22-cutoff-home-jobs.md` (title "(2026-11-22)"), the ledger header "### 2026-11-22" and its update line, agent-models §5.7/§5.5 ("revised 2026-11-22", "built 2026-11-22", "2026-11-21, 2026-11-22"), and live.rs:5090 and 5619 ("Before 2026-11-22"). Rename to `2026-11-21-…` and fix the links in TASKS.md, the ledger and agent-models (`python3 bin/check-docs.py` will catch the links).

### P3-5. The note's measurements count only the first loose ball

`an.py`, `unc.py` and `stretch.py` keep `since == rows[0]['since']`. That causes P2-1's 0. It also leaves the uncovered-base and early-cutoff counts after a deflection or overthrow out of every table. The note should say so where it gives those counts (§3), or recount.

## Answers to the questions

1. **Invariant 2. No leak found.** Every new decision reads only:
   - the viewer's own read, role and `through`;
   - teammates through `seen`/`known`/`known_job` (lagged) and their visible `pos` (`ball_gone_by`, `seen_intercept`);
   - public facts: the ball's present position and velocity (the existing "as everyone sees it" convention, as in `fielding_point`), `first_down_m`, `ball_still_up`, `ball_gen`, the runners on base, the pre-pitch spots in `through_point`.

   `home_cutoff_man` reads `seen(side, other)`, which is the lagged `known_job`. The `Cell` writes in `home_cutoff_by_convention` touch only the viewer's own `through`. Every caller passes the man deciding (`cover_table`, `cover_after_chasers`, `through_backup(0)`, `through_jobs`). The harness's coverage is real: a copy in which `home_cutoff_by_convention(o)` also reads a teammate's `through` failed `isolation_no_private_state_reaches_a_teammate_in_real_games` (23 of 951,804 checks leaked, e.g. "seed 0 play 33 … f2 through"). The real build passes it.
2. **Second base open.** See P3-2.
3. **Infielder-fielded balls.** See P2-1 and P3-3. On seeds 120–239, first-ball tables of RISP ground balls an infielder fielded: second open 7 → 9 plays and third 4 → 6 (`uncif.py`). The new ones are seed 152 play 77 (0.03 s), seed 185 play 199 (0.07 s), seed 191 play 125 (P3-3) and seed 217 play 104 (first and second, the 1B's relay_job cutoff with the SS fielding). I found no lost force or double play. Of the four first-differing plays in seeds 120–239 whose outcome changed, two are better defence, made possible by the new standard cover:
   - seed 179 play 34: the runner is thrown out at third with the SS on third at once;
   - seed 217 play 104: the SS, heading to second instead of third, fields a chopper behind second and gets the batter, where main gave a run-scoring single;
   - seed 191 play 125: a different third out, at first instead of second;
   - seed 140 play 71: the batter is thrown out at second.

   In seeds 0–119, three runner-on-first plays changed: the runner scored instead of stopping at third.
4. **Flicker and stuck state.** `through` holds `ball_gen + 1`, so a deflection, a throw or an overthrow (each a new `ball_gen`) makes the stored value inert, and every play starts at 0 (`new_sim`). A wrong judgment on a ball that dies in the infield is cleared as soon as any infielder is known to be going for it: a stopped ball has gone by nobody, and the margin is 0 once judged. Fast A–B–A flips (B shorter than 0.6 s, `aba2.py`), main → final:
   - seeds 0–119: RISP grounders 11 → 14, runner-on-first grounders 7 → 10, nobody on 15 → 16, balls in the air unchanged;
   - seeds 120–239: RISP 7 → 9, runner on first 6 → 5.

   The extra ones are the P3-1 deep-relay swaps and seed 9 play 9's SS (third, second for 0.3 s, third), both brief. The slower A–B–A increase with a runner on first (39 → 67) is the SS going back to second when the cutoff turns home, 2–3 s later: a change of play, not a flicker. Cover of second changing hands: the 2B's first job second and his job at the catch first rises 2 → 16 (`wrongway.py`): the documented hand-over. No man runs back and forth between two bases repeatedly, except in the pre-existing deep-ball flicker.
5. **Cutoff man by the fielding outfielder.** Every viewer computes the same `through_point` from public facts, so they agree with each other. Both corner men are never out at once (`cut.py`: 0 plays in 240 games). The cutoff changed hands between corner men on 1 play in seeds 0–119 (main 2) and on 0 in seeds 120–239 (main 2). On an outfielder-fielded RISP grounder there is a cutoff home at the catch on all but 14 / 10 plays; those are deep balls, where a middle infielder is the relay man. In the left-centre gap the convention's outfielder can differ from the one who becomes the known lead. The SS then starts for one base and switches to the other within 0.3–0.6 s, on 1–2 plays per 120 games (`gap.py`: seed 69 play 259, 121/195, 187/229; seed 26 play 11 goes third, none, second). Acceptable.
6. **Scope.** See P3-1. All non-RISP first divergences are explained.
7. **Tests.**
   - **Not tautological.** Both check the coaching-guide standard arrangement and a knowledge-lag bound, not the code's own tables.
   - **They fail on main.** With `312789e`'s live.rs and the new test file, both fail as the note says. seed 9 play 9: "at 4.62 s fielder 4 has job 12 (expected 11)". The turned-play test: "play 102 mirrored, turn −17: fielder 6 with no job from 0.66 to 0.97 s". The other 20 tests pass.
   - **The 0.3 s allowances** equal `CALL_HEARD_S`, the knowledge lag every hand-over must take, so they are justified. They do hide the hand-over gaps on second (P3-2), and the no-job check starts only after the last infield chase is known (case (b) left out on purpose).
   - **The `one_back` narrowing:** see P3-1.
8. **Determinism.** No HashMap or HashSet, no direct transcendental calls in sim code (`m::hypot` throughout; `f64::hypot` only in tests), no RNG draws. `through_point`'s `min_by` with `total_cmp` returns the first minimum. `through_jobs` runs fixed 0..6, and each man's step changes only his own state; others read it lagged. My probe builds of both commits reproduce the worker's traces byte for byte on seeds 0–2.
9. **Spec, note, ledger, TASKS.**
   - §5.7 matches the code except where P2-1 and P3-1 say otherwise.
   - No stale −15 m rule is left: the only mention, live.rs:5090, is historical.
   - Spot checks, rerunning the worker's `an.py`, `unc.py` and `stretch.py` on his logs: 2B 0.360 → 0.109 and SS 0.557 → 0.102 s a play; stretches 77 (25.7 s); first uncovered 86 → 27, second 10 → 22, third 47 → 14; early cutoffs 0 → 26. Those logs reproduce from the probe.
   - The ledger row matches the worker's `std-0.txt` (runs 4.07, K% 22.6, 4,618 ms per game per core).
   - Date: see P3-4.
10. **Would a real player do this?**
    - Seed 9 play 9: at the catch 3B cutting, 1B first, 2B second, SS third, P behind home, the standard. On the way, the SS goes for the chopper and stops (1.41–2.01 s), and at 1.71 s the pitcher, the 3B and the SS are all on their way to third. That is case (b), not this change.
    - Seed 217 play 104: better than main. The SS covers second, toward the ball, and fields a chopper he would have let through.
    - Seed 8 play 208: P2-1.
    - Seed 26 play 11: the left-centre gap. The SS starts for third, has no job for 0.15 s, then takes second; the 1B and 2B both stand on first for 0.3 s at 5.27 s. Brief, and real infielders also adjust.

## Verification

- **Workspace tests.** `touch sim/crates/bb_game/src/live.rs`, then `taskset -c 0-7 nice cargo test --workspace --release` in `sim/`: **511 passed, 0 failed, 4 ignored** (log `sim/target/codex-probes/review-cuthome/test-ws.log`).
- **Tests on main.** A copy of `687102d` with `312789e`'s live.rs (`review-cuthome/oldlive`), `cargo test --release -p bb_game --test throw_choices`: 20 passed, 2 failed (the two new tests, messages above; `test-tc-old.log`).
- **Isolation harness control.** A copy with a planted leak (`review-cuthome/leak`: `on` also true when another infielder's `through` is set): `isolation_no_private_state_reaches_a_teammate_in_real_games` failed, 23 leaks (`test-leak.log`).
- **Trace probe.** The worker's `zz_cut_home.rs`, built from both copies with `CARGO_TARGET_DIR=sim/target/review-probes`. Both reproduce the worker's `tr-final.log` and `tr-before.log` byte for byte on seeds 0–2. Run on seeds 120–239 for both (`tr-new-120.log`, `tr-old-120.log`, `BB_COVER_TRACE=1`).
  - Note that a second workspace copy built into the same target dir was treated as fresh (cargo keys path packages relative to the workspace root, and tar gave old mtimes). I touched the sources and checked that the binaries differ before using them.
- **Analysis scripts** in `sim/target/codex-probes/review-cuthome/`: `aba.py`, `aba2.py` (flips), `cut.py` (cutoff ownership), `early.py`, `early_all.py`, `eif.py` (early cutoffs, every loose ball), `oneback.py`, `firstdiff.py`, `firstdiff2.py`, `outdiff.py` (scope and outcome changes), `uncif.py`, `cornerif.py`, `wrongway.py`, `gap.py`, plus the worker's `an.py`/`unc.py`/`stretch.py`. Run over the worker's seeds 0–119 logs and my seeds 120–239 logs.
- **Not done:** the 300-game standard runs (the Coordinator's), and `isolation_many_seeds`.

## Resolution

Open.
