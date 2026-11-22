# Review: the infielders' jobs while a liner or blooper falls in (`work/liner-single-jobs`)

Reviewer: Opus (independent; wrote none of the change). Range `main..e16a457` (`main` = `445ef2c`), final sim code `b90e422`. Files: `sim/crates/bb_game/src/live.rs`, `sim/crates/bb_game/tests/throw_choices.rs` and fixtures `tests/fixtures/a4/{5-60,5-149,0-17,2-117,11-203}.json`, `docs/agent-models.md` §5.7, `docs/calibration/2026-11-22-liner-single-jobs.md`, `docs/metrics-ledger.md`, `TASKS.md`, `observations.md`.

## Summary and verdict

The change does what the note says on the cases it measures, and the figures I re-derived from the worker's traces match the note exactly. No invariant is broken: every input to `air_read`, `air_cutoff`, the new `deep_ball` branch and `known_first_out` is either the fielder's own read and estimates, a teammate's job or call as shown `CALL_HEARD_S` ago, or a visible fact (where the ball is now, where it first came down, the pre-pitch spots). There are no new random draws, no hash iteration and no direct transcendental calls, and the per-look caches are filled in a fixed order. But the change adds a failure the note and ledger do not report. One infielder judges a ball a single and, by the convention, counts the corner man as out on the cutoff home (or, toward third, the shortstop). Meanwhile the man he counts did not judge it a single and stays on his bag. The first man then covers that same bag and does not go out as the relay man, because his own single judgment blocks the deep ball's early relay. The convention never gives way to what he can see: the man he counts standing on the bag. In seeds 0–119, two infielders stood on one bag with nobody on a cutoff or relay for more than 0.5 s on **0 plays on main and 32 on the branch**, for up to 3.2 s, several of them on balls that came down past 80 m. **Mergeable after fixing P1-1.** The P2s can go with it or into TASKS.

## Findings

### P1-1. A convention counted against what the fielder can see: two men on one bag, nobody on the relay (new; not in the note or ledger)

`live.rs` 5200–5236 (`home_cutoff_by_convention`, the `air` branch and the `on` latch), 5413–5437 (`second_takes_over`, the same), 5016–5020 (the deep relay gate's `self.air_read(who).is_none()`).

Evidence (worker's traces `tr-before.log` → `tr-a12.log`, seeds 0–119; my probe `review-probes/py/dup.py`): on balls still up at the first call that an outfielder fielded after they came down, stretches of more than 0.5 s with two infielders covering the same base and no infielder on a cutoff or relay:

| | main | branch |
|---|---|---|
| Runner in scoring position, dirt; shallow; deep | 0; 0; 0 | 2; 16; 6 |
| Runner on first, shallow; deep | 0; 0 | 6; 2 |
| More than 1 s (all base states) | 0 | 19 |

Over the same balls with runners on, the plays with more than 1 s with no cutoff or relay man after the outfielder's chase is known (plus 0.3 s): 4 → 15 (`norelay.py`).

Examples (roles as the trace codes them: 11/12/13 cover first/second/third, 24 relay home):

- Seed 80 play 169, runner in scoring position, the right fielder fields it. From 0.74 s to 3.94 s the first and second basemen are **both on first** (`[.., 11, 11, ..]`) and nobody is on a cutoff. On main the second baseman was the relay man from 1.05 s.
- Seed 61 play 187, first down beyond 80 m, the left fielder fields it. From 0.70 s to 3.43 s the third baseman and shortstop are **both on third**; the shortstop goes out only once it is down. On main he was on the relay home from 1.00 s.
- Seed 77 play 114, deep: first and second basemen both on first from 0.65 s to 3.42 s. On main the second baseman was the relay man from 1.25 s.

Mechanism. The second baseman's `air_read` says single, so `home_cutoff_by_convention(3)` returns the first baseman. `cover_table` and `cover_after_chasers` count him as the cutoff, and the second baseman takes first. The first baseman does not judge it a single, so `air_cutoff(2)` is `None` and he stays on first. In seed 80 play 169 (`BB_DEBUG_COMM`, re-run with the worker's final probe), at the first call (0.745 s) the first baseman's read has the ball reaching the outfield at (24, 115) m, over the right fielder, and the second baseman's has it at (18, 81) m. It came down at 78.6 m and the right fielder took it at 80.0 m. Such a gap between two reads is a mistake a real player can make; standing on first together for 3.2 s afterwards is not. After 0.3 s the second baseman knows the first baseman's job (cover first, `known_job`). But the `on` latch (`fd.through == gen1`), and after a reset the cached `f[o].air`, return the convention again, and nothing in it asks whether the man counted out is known to be doing something else. Then nobody goes out. If the fielding point is beyond 80 m, the deep relay man would be the second baseman, but `relay_job`'s gate keeps him in while the ball is up because his own `air_read` is `Some`; that is seeds 61/187 and 77/114, where the relay man goes out only at the bounce. If it is short of 80 m, nobody goes out while the ball is up unless the corner man judges it a single himself. Seed 80 play 169 sits on that line (taken at 80.0 m); either way nobody was out until it came down, when the first baseman went. A real second baseman who sees the first baseman holding first does not stand on first with him. On a ball over the outfielder he is the relay man.

Before this change the same convention existed only for ground balls, where the infielders' reads converge, and on main the traces show no such stretch on these balls in the air.

Suggested fix: end the convention (not only suspend it) once the man it counts out is known to `o`, on this ball, to be on another job, shown after he had time to react: `known_job(c)` on this ball, shown no earlier than `o`'s judgment, and not `Relay(_)`/`Chase`. Then `o` works the tables from what is known. In `relay_job`'s early gate, let the relay man's own single judgment hold him in only while the single's cutoff man is not known to be staying put (the same test). Add the play as a fixture (80-169 or 61-187) that asserts no bag is held by two men for more than 0.3 s and that someone is on a cutoff or relay from the outfielder's known chase plus 0.3 s.

### P2-1. The relay man stands down for the corner man's cutoff home on a ball that is going deep (out-back-out 1 → 7 deep, 2 → 9 shallow)

`live.rs` 5016–5020: `!(0..6).any(|j| j != who && matches!(self.seen(who, j).0, Role::Relay(_)))`.

Of the 7 out-back-outs on deep balls with a runner in scoring position and the 9 on shallow ones, 15 have the same shape (my `obo_list.py`). The middle infielder goes out as the deep relay man at or just after the first call (0.95–1.26 s). He goes back to cover a base 0.15–0.7 s later, mostly 0.3 s later, when the corner man, on his own single judgment, is known out on the cutoff home. He goes out again at the bounce (deep) or when the ball passes the spot plus `PAST_OF_M`. Seed 47 play 138: down at 86.7 m, fielded at 120 m by the centre fielder; the second baseman is on relay 0.95, back to first 1.25, out again 2.70. On main it was a clean relay from 0.95. The relay man read the ball right, and the gate pulled him in. On a deep ball the relay man and a cutoff man home are both out in standard play, so a corner man on `Relay(4)` is no reason for the relay man to stand down. The note's §5 explains the shallow 2 → 9 as "a gap hit landing short hands the job … once it passes the outfielder's spot". That is the second half; the first half, the relay man pulled in, comes from this gate. The deep 1 → 7 (and relay changing hands 2 → 11) is listed in the note and ledger without a cause, and the ledger says "Nothing unexplained".

Suggested fix: make the gate count only a man known out on a cutoff that replaces the relay man (a middle infielder on `Relay(2)`/`Relay(3)`), not a corner man on `Relay(4)`. Or drop the clause and let the deep relay gate rest on the relay man's own judgment. Re-measure out-back-out with `hands.py`.

### P2-2. A runner from first passing second sends the first baseman to the cutoff home on a single (first open 6 → 24 plays): pre-existing, but now common

Confirmed in the traces (runner on first, shallow: first uncovered 6 plays/1.6 s → 24/6.0 s; runner in scoring position, deep: 0 → 5). The cause the note gives fits the code: `relay_base`'s `on(2)` counts a runner who started at first at the base he is at now. In real play the throw on that runner goes to third, with the shortstop as cutoff, and first stays covered behind the batter-runner. The note and TASKS send this to the relay review as an owner question. I agree it is not this change's to fix. It probably needs no owner decision: the standard alignment (the first baseman stays with the batter-runner, the shortstop cuts the throw to third) is textbook, so it can be ordered as a fix to the conventions.

### P3-1. Spec: "No new parameter" is stale

`docs/agent-models.md` line 662 ends "No new parameter: `INFIELD_BREAK_R`, `RELAY_FROM_M`, `RELAY_DEEP_M` and `REACH` exist." The change adds `PAST_OF_M` [E, unsourced], named two paragraphs up. Fix the sentence.

### P3-2. The run-time cost is given three ways

The spec (line 658) says the judgment keeps its cost "to about 1–2% of a game's run time". Commit `b90e422` says "about 2%", and the note §2 and §6 and the ledger say +3–5% (measured back to back: 4,699 → 4,928 [5,044 → 5,193] ms). Use the measured figure in the spec. I see no obviously cheaper form that gives the same games: the landing-depth test is already asked first, the judgment is cached per look, and the outfielders' `seen_intercept` scans are what it costs.

### P3-3. "Ends it" vs "suspends it"

The spec (line 660) says a judgment made while the ball was up stands after it comes down beyond the dirt "until a chase known to him ends it, as on a ground single". In `home_cutoff_by_convention` (5224–5235) a known infielder's chase only returns `None` while it is known. Once it is no longer known, the cached `f[o].air` re-arms `through` without asking his read again, whereas on a ground single the read is asked anew. Either say so in the spec or clear `f[o].air` where `through` is cleared. (P1-1's fix touches the same lines.)

### P3-4. Dates

The note's file name, the spec ("built 2026-11-22"), the ledger and TASKS all date the work 2026-11-22. The commits are dated 2026-11-21 (local), as is `date +%F` on this machine. The repository's dated records use the local date. Rename the note and fix the dates, or leave a line saying why.

### P3-5. Small stale or loose text

- `a_ground_single_fielded_deep_is_not_a_deep_ball` (`live.rs` ~10700): "The same ball in the same spot after a fly that came down in the outfield: deep". It is deep now only because the touch at (6, 80) is 80.2 m out, at or beyond `RELAY_DEEP_M`; a fly down at 70 m would not be. Say "came down beyond `RELAY_DEEP_M`". The rewritten assertion itself is not weakened: `is_empty()` became an exact `vec![(3, Role::Relay(2))]`, which is what case (d) requires, and the two new cases at 88 m and 110 m test both sides of `spot + PAST_OF_M`.
- `second_cutoff_man`'s doc comment (5131–5135) still says "unless his partner is already known to be out on that cutoff". The rule is now "the man first known out on it keeps it".
- The fixture README lists the five new fixtures but not that the guard test also reuses `155-23` and `264-109`.

## Questions answered

**1. Invariant 2.** Traced each input:

- `air_read`:
  - his own `belief`, `read_at`, `read_ball`, `t_int`/`int_pt`, `looks`, `role`;
  - `known(j)` for the outfielders' sure calls, as shown `CALL_HEARD_S` ago;
  - `seen_intercept` (visible positions and velocities, known speed and reaction, run on his read);
  - `ball_gone_by` and `ball_still_up` (where the ball is now, and whether it has touched down yet: visible);
  - the pre-pitch spots `home[]`;
  - the landing and fielding points: his read's `first_bounce` and his estimate's earliest outfielder.
- `air_cutoff`:
  - `relay_base(false, c)` (c's reading of the runners);
  - `second_cutoff_man` (public `shown`);
  - `home_cutoff_man(single_of())` (`lead()` from known chases and visible positions; `through_point` from the visible ground track; `seen(side, other)`);
  - `infield_lets_go` (unchanged).
- `deep_ball`'s new branch:
  - `first_down_pt`, the first ground or wall touch in `out.facts`, a visible event, public at the touch, as `first_down_m` already was;
  - the outfielder spot nearest it (pre-pitch, public);
  - the ball's position now;
  - `known_job` of the middle infielders (public).

  Its one per-viewer term is `relay_base(false, 5)` (the shortstop's runner-reading threshold), which is an attribute, not private state.
- `known_first_out`: `shown` entries up to `CALL_HEARD_S` ago.

Nothing reads the true landing point, the true outcome, or another man's private read or job before it is shown. `relay_job`'s gate reads `air_read(who)` for `who`'s own job, as it already read his `no_air_play()`. The isolation harness now flips `Fielder::air` and passes. Not flipped: `air_no`, a negative cache keyed on his own look and read, which cannot carry anything between fielders.

**2. Invariants 5 and 1.** No new RNG key or draw, no `HashMap`/`HashSet`, no direct `f64` transcendentals (`m::hypot`, `norm`), no `mul_add`. `min_by` with `total_cmp` is deterministic. The caches (`air`, `air_no`) belong to the fielder, and their result for a look is fixed by the first call in the tick, in a fixed call order. In `update_relay`'s new loop over `c in 2..6`, `air_cutoff(c)` reads only `c`'s own role, the shown jobs and positions, not the roles set earlier in the loop, so the loop order does not matter. The determinism tests pass.

**3. Would a real player do this?**

- (a) Corner man on the cutoff home while a fly that is then caught is up (13 of 357 with a runner in scoring position). Real and correct: it is where the cutoff man stands for a throw home on the tag-up, and the 2B/SS/P alignment covers every bag. Doubled off and runs on those catches are unchanged (2 → 2, 118 → 118). First open while up 7 → 10: three more short gaps, acceptable.
- (b) See P2-2.
- (c) See P2-1. Partly real (a misread by the corner man), mostly a gate that pulls in a relay man who read it right.
- (d) A ball that truly gets past the outfielder short of 80 m: when the fielding point everyone sees while it is up (the chaser's ground track) is beyond 80 m and the relay man does not judge it a single himself, he goes out early and `relay_out` keeps it deep after the bounce. When one reads it as a single, the relay job comes only when the ball passes the spot plus 5 m, 1–2 s late on the gap hits listed (secured 97–118 m). `PAST_OF_M` is measured from a pre-pitch spot and a visible touch point, which everyone can know. I checked the other risk: a relay man out at the pickup and replaced there because the ball was fielded in front of the spot. 1 play on main, 1 on the branch (the same one), so `relay_out` does not cause late hand-overs at the pickup.
- (e) The pinned cutoff to second never ended on the wrong side of the ball by more than 10 m (of 471 cutoffs to second at the pickup, 4 on the wrong side by less than 10 m: balls up the middle). The pin does not hold the wrong man in practice.

**4. Correctness.**

- Gate edge cases:
  - A ball an infielder catches after being judged to have no play: `Held` by an infielder ends every air job, and the relay men go idle in `update_relay`.
  - An outfielder's sure call after the infielders left: the judgment is cached and stands. Intended and specified, and harmless for the jobs, since they suit a catch.
  - Walls: a first touch on a wall is at least 80 m out, so the ball is deep.
  - Deflections: `ball_gen` is the cache key.
  - Re-calls: no change.
- Jobs that never end: none found. `air_cutoff` is held only while the ball is up, and afterwards `relay_job` and `decided` govern as before.
- Pre-existing behaviour changed: `through_jobs` and `second_takes_over` now run while the ball is up (specified); case (d) (specified); the pin in `second_cutoff_man` applies to ground balls too (specified); and the relay gate's "another man out on a cutoff" clause, whose effect on deep balls is not described (P2-1).
- The rewritten unit test: see P3-5. It is not weakened.

**5. Tests.**

- The three fixtures fail on `445ef2c` for the stated reasons (Verification).
- The guard test passes on both and checks things this change could have broken: an infielder's own catch, the deep relay while the ball is up, and covered bags on caught flies.
- Missing:
  - a play where the infielders' judgments differ (P1-1);
  - seed 6 play 233's ping-pong, which `known_first_out` was written for and no test freezes;
  - a case with the ball between the spot and the spot plus 5 m (not deep) in the unit test;
  - a deep ball where a corner man misjudges a single (P2-1).

**6. Spec, note and ledger.**

- I re-ran `air.py`, `catches.py` and `hands.py` on the worker's `tr-before.log` and `tr-a12.log`. Every figure the note's §1 and §3 tables give matches: stretches, no cutoff man for second 310/310 → 37/306, first uncovered 6 → 24, out-back-out, changed hands, and the caught flies.
- `PAST_OF_M` is named, tagged [E, unsourced] in the code and spec, and not a literal.
- Stale text: P3-1 to P3-5.
- The ledger's "Nothing unexplained" misses P1-1's stretches and the deep out-back-outs (P2-1).
- Cost: P3-2.

## Verification

All runs under `taskset -c 0-7 nice` with `CARGO_BUILD_JOBS=2`, one at a time. The worktree was clean at `e16a457` throughout, apart from this file.

1. **Workspace tests at `e16a457`**: `cargo test --workspace --release` in the worktree's `sim/target` (cargo relinked `bb_game`; no source touched). **516 passed, 0 failed, 4 ignored**, 10 min 18 s. These include `determinism.rs`, `crate_graph.rs`, `isolation_no_private_state_reaches_a_teammate_in_real_games` (now flipping `Fielder::air`), the rewritten `a_ground_single_fielded_deep_is_not_a_deep_ball`, and the four new `throw_choices` tests. Log: `sim/target/review-probes/ws-tests.log`.
2. **The new tests on main**: `git archive 445ef2c sim` into `sim/target/review-probes/main-src`, with the branch's `tests/throw_choices.rs` and the five new fixtures copied over it. Built with `CARGO_TARGET_DIR=sim/target/review-probes/liner-tgt` (every workspace crate compiled from that copy) and ran `cargo test --release -p bb_game --test throw_choices`: 24 passed, **3 failed**, each for the stated reason:
   - `seed_5_play_60…`: "a middle infielder on a relay at 1.00 s, while it is up: [30, 14, 11, 24, 13, 12, …]";
   - `seed_5_play_149…`: "a middle infielder on a relay at 1.25 s, while it is up";
   - `seed_0_play_17…`: "at 2.80 s, after it came down at 2.78 s: [30, 30, 11, 0, 13, 12, …]" (the second baseman with no job);
   - `a_ball_an_infielder_catches_a_deep_ball_and_a_caught_fly_keep_their_jobs`: passes on main, as claimed.

   Log: `review-probes/main-tc.log`.
3. **The note's figures**: re-ran the worker's `air.py`, `catches.py` and `hands.py` (in `sim/target/codex-probes/liner/`) on `tr-before.log` and `tr-a12.log`. Every number in the note's §1 and §3 tables I compared matches, including 310/310 → 37/306, first uncovered with a runner on first 6/1.6 s → 24/6.0 s, out-back-out 10; 2; 1 → 5; 9; 7, changed hands 41; 38; 2 → 19; 51; 11, and the caught flies (13 of 357, first open 7 → 10, extra outs 2 → 2, runs 118 → 118). Not in the note: with a runner in scoring position on deep balls, first uncovered 0 → 5 plays; with nobody on, deep, no cutoff man for second 265 → 251.
4. **My probes**, all on the same two traces, in `sim/target/review-probes/py/`:
   - `dup.py`: two men on one base with nobody on a cutoff or relay, for more than 0.5 s: main 0 → branch 32; more than 1 s: 0 → 19.
   - `norelay.py`: more than 1 s with no cutoff or relay man after the outfielder's chase is known, runners on: 4 → 15.
   - `obo_list.py`: of the 16 out-back-outs with a runner in scoring position, shallow and deep, 15 run relay → cover → relay, the cover beginning 0.15–0.7 s after he went out (one is relay → no job → relay to third).
   - `side.py`: the cutoff to second at the pickup, on the wrong side of the ball by more than 10 m: 0 of 471.
   - `pickup.py`: the relay man replaced at the pickup on a ball first down short of 80 m: 1 → 1 (the same play).
5. **Seed 80 with debug output**: `BB_DEBUG_COMM=1 BB_COVER_TRACE=1 codex-probes/liner/probe-a12 1 80`, the worker's final probe binary (the same traces as `tr-a12.log`). Play 169's rows reproduce the trace, and the first call's reads show the first and second basemen's different estimates (P1-1). Log: `review-probes/s80.log`.
6. **Read**: the whole `live.rs` diff, and the callers and callees named in Q1. I did not run the standard run; I took the worker's `std-*` files for the figures quoted, and nothing here depends on them.

Housekeeping: `sim/target/review-probes/main-src` held an earlier review's probe copy of main (built 2026-11-21 02:07). I replaced it with the `445ef2c` copy above. `liner-tgt` is a reflink copy of `t-main/release` plus this build. Both are throwaway.

## Worker follow-up

By the fix worker (not the reviewer), finding by finding. Code, spec, fixtures and test in `84e10f8`; the note's figures, ledger row, TASKS and this section in the commit after it. Figures on game seeds 0–119 with the same trace probe and the review's scripts, final build against `e16a457`.

- **P1-1, fixed** (`single_cutoff_stays`, `Fielder::stays`, `Fielder::cut`; agent-models §5.7, "A cutoff man who stays"). The conventions end, for good on this ball, once the man they count out on the single's cutoff is known to the counting man to be on another job than a cutoff or a chase, as shown at or after the moment that man could have gone: once he was the cutoff man as the counting man counts him (a runner going on, or another outfielder taking the ball over, changes who it is), and after the infield gate he waits for (every infielder known on this ball plus `CALL_HEARD_S`). The deep relay gate lets the relay man's own single judgment hold him in only while the single's cutoff man is not so known. Not exactly the suggested test ("shown no earlier than `o`'s judgment"): a covering man's shown entry is not renewed while he keeps the job, so it is judged on his job as known at that moment; and measured on the counting man's own judgment the cutoff man waiting for the gate was counted as staying (out-back-out 9 → 34), while measured after the gate but no earlier than that judgment, a relay man who judged it late came back to the bag for 0.3 s (17). `dup.py`: 32 → 23 plays over 0.5 s, each exactly 0.6 s (the gate's 0.3 s and 0.3 s to see him), 0 over 0.65 s (main 0). `norelay.py` (more than 1 s, runners on): 15 → 0 (main 4). Test `a_single_judged_in_the_air_gives_way_to_a_cutoff_man_who_stays` (seed 61 play 187, seed 21 play 86, seed 77 play 114, frozen from `445ef2c`, the same balls on `e16a457`): no bag held by two infielders more than 0.6 s (not 0.3 s, for the reason above), and an infielder on a cutoff or relay at every tick from the outfielder's chase known plus 0.3 s. Each case fails alone on `e16a457`'s `live.rs`. Costs, in the note §3: first open 0.3 s on 15 more shallow balls when the cutoff man goes late, and a relay man who went out because the cutoff man stayed coming back when the cutoff man went after all (runner in scoring position, dirt, stretches after it came down 3 → 9).
- **P2-1, fixed** (agent-models §5.7, "The deep ball's early relay man"). The relay man stands down for his partner's cutoff (to second or third), and for a corner man's cutoff home only on a ball his own read has coming down short of `RELAY_DEEP_M`; on one he reads coming down beyond it, both are out. Dropping the corner man's clause outright left the relay man out on balls that fell in short, with first open while both were out (first uncovered while loose, runner in scoring position, shallow, 21.7 s against 8.6 s). Making it hold also needed `deep_ball`'s known relay man judged on the jobs known when the ball came down (with a corner man out too the call is split and the fixed mark decides): judged on the relay man's job now, the two traded the job every 0.3 s after the bounce (seed 47 play 142). Seed 47 play 138 is a clean relay from 0.95 s. Out-back-out, runner in scoring position (dirt; shallow; deep): 5; 9; 7 → 6; 10; 2 (main 10; 2; 1). The note's §3 and §5 and the ledger now give the cause of the rise (8 of the shallow 10 are gap hits fielded beyond 97 m: the relay man pulled in when the corner man was seen out on a ball he read landing short, out again once it rolled past the outfielder's spot); the deep 7 were this gate.
- **P2-2, not fixed (as instructed)**: reworded in TASKS §3 and the note §5 as a cutoff-convention bug for the relay review, not an owner question.
- **P3-1, fixed**: "One new parameter, `PAST_OF_M`".
- **P3-2, fixed**: one measured figure, from the final build's standard run back to back with main, in the spec, the note (§2 item 1 points to §6) and the ledger.
- **P3-3, fixed in the spec text**: a known infielder's chase suspends the conventions while it is known (the judgment in the air is not asked again); a cutoff man known to be staying ends them (P1-1).
- **P3-4, fixed**: the note is `docs/calibration/2026-11-21-liner-single-jobs.md`; the spec, ledger, TASKS, observations, fixture README and code comments follow. `bin/check-docs.py` clean.
- **P3-5, fixed**: the unit test's comment (down beyond `RELAY_DEEP_M`, 80.2 m), `second_cutoff_man`'s doc comment, and the fixture README (the guard test reuses `155-23` and `264-109`).

## Resolution

Open.
