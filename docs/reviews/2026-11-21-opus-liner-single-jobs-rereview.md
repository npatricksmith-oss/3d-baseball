# Re-review: the liner-single review fixes (`work/liner-single-jobs`)

Reviewer: Opus (independent; wrote none of the change). A narrow re-review of the fixes to the [first review](2026-11-21-opus-liner-single-jobs-review.md) only. Range `941b4f2..0deb7f1`; sim code in `84e10f8`: `sim/crates/bb_game/src/live.rs` (`single_cutoff_stays`, `single_cutoff_man`, `track_single_cutoff`, `Fielder::stays`, `Fielder::cut`, the relay gate in `relay_job`, `first_down_t` and `relay_out` in `deep_ball`), `tests/throw_choices.rs` (`a_single_judged_in_the_air_gives_way_to_a_cutoff_man_who_stays`) and fixtures `61-187`, `21-86`, `77-114`. Also agent-models §5.7, the [note](../calibration/2026-11-21-liner-single-jobs.md), the ledger row, TASKS and observations.

## Summary and verdict

Both fixes do what the worker says. I reran the review's scripts on the final trace, and every figure in the worker's table matches:

- two men on one bag for more than 0.65 s: 30 → 0;
- the same for more than 0.5 s: 32 → 23, each exactly 0.6 s;
- more than 1 s with no cutoff or relay man: 15 → 0;
- out-back-out with a runner in scoring position, deep: 7 → 2.

I also checked where `tr-b8.log` came from. I rebuilt the trace probe from the committed `84e10f8` sources and reran it. The new trace is byte-identical to `tr-b8.log`.

The 0.6 s floor is real. It is two `CALL_HEARD_S`: the cutoff man's infield gate, then the time to see that he did not go. The new test's 0.61 s allowance is that floor plus one tick. It is not tuned: two of the three fixtures sit exactly on it (they fail at 0.57 s).

Each case of the new test fails on `e16a457`'s `live.rs` for the stated reason. Each of its two assertions also fails on its own.

No invariant is broken:

- Every new input is the fielder's own read, a job shown `CALL_HEARD_S` ago, or a visible fact.
- The latches belong to the fielder, are keyed by the ball, and are written and read only for that fielder.

The worker's departure from the review's "known to stay" test is sound. The latch cannot pin a wrong belief for more than `CALL_HEARD_S`: it only ends his anticipation, and the tables still count the cutoff man as soon as he is seen out.

What the review missed or the note glosses over is small:

- On a few shallow singles with a runner in scoring position, the cutoff man home now changes at the pickup (P2-1).
- Some of the deep "first open" time is not the standard alignment (P3-1).
- Some text is stale (P3-2, P3-3).

**Mergeable as is.** P2-1 can go to TASKS with the relay review, or be fixed first; the P3s are text.

## Findings

### P2-1. A relay man kept out by `relay_out` while the ball is loose is replaced at the pickup by the corner man, who leaves his bag as the outfielder gathers the ball (not reported)

`live.rs` 5144–5150 (`relay_out` holds only `while matches!(self.ball, Ball::Free { .. })`), together with the relay gate at 5031–5037.

Evidence comes from the first review's `pickup.py` (runner in scoring position, ball first down short of 80 m, a middle infielder on the relay home at the pickup and off it at the pickup). It gave 1 → 1 → **6** (main → reviewed → final). My `py2/pickup2.py` counts any hand-over of the cutoff or relay job at the pickup on the air balls. For runner in scoring position, shallow, it gave 8 → 2 → 7. So the final build is back to main's count, but on different plays.

Five of the six are new:

- **Seed 23 play 94.** First down at 69 m; the right fielder fields it at 85.7 m; the throw comes 2.5 s after the pickup.
  - Final build: the second baseman is the relay man from 0.95 s to the pickup (3.78 s). At the pickup the first baseman goes out from first as the cutoff home, and the second baseman has no job for 0.3 s.
  - Main: a clean relay by the second baseman through the throw.
  - Reviewed build: the first baseman was out from the bounce, 1.2 s before the pickup.
- **Seed 115 play 32.** The same pattern, with the throw 0.82 s after the pickup. The first baseman starts from first toward a cutoff spot 20-odd metres away less than a second before the ball is thrown.
- **Seed 21 play 86** (one of the new fixtures), **seed 47 play 142** and **seed 63 play 211** (third baseman for shortstop): the same pattern.

Mechanism:

1. With the P1-1 fix, the man who judged it a single sees the corner man staying. He is then not held in by his own single judgment, and goes out as the deep ball's relay man (while it is up, the fielding point at the outfielder's feet is beyond 80 m).
2. At the bounce, `relay_out` (he was known out at the touch, no corner man was) keeps the ball deep while it is loose.
3. Once the outfielder has it, `relay_out` no longer applies. The fixed mark (spot + `PAST_OF_M`) says single, and `relay_job` gives the cutoff home to the corner man.

A real infield does not change its cutoff man as the outfielder picks up the ball. The man already in line stays there, and the first baseman does not leave first at that moment.

Suggested fix: let the known relay man keep the job through the pickup. Either:

- let `relay_out` also hold once an outfielder holds the ball (`Ball::Held { by } if by >= 6`, until the throw), still judged on the jobs known at the touch, a fixed fact; or
- extend `home_cutoff_man`'s "a man known out on it keeps it" to a middle infielder known on `Relay(4)`.

Re-measure with `pickup.py` and `py2/pickup2.py`. If it is deferred, add it to TASKS' "For the relay review" line ("the cutoff man home changes at the pickup").

### P3-1. The deep "first open" (7 plays, 11.0 s) is mostly, not wholly, the standard alignment; second open and a pitcher flipping are not reported

The note §3 ("First open on a deep ball while both are out … the standard alignment on a ball over the outfielder; the batter is going for two") and the ledger row say this.

Checked play by play (`codex-probes/liner/unc_b.py tr-b8.log risp deep 0`):

- **Standard alignment (7.4 of the 11.0 s).**
  - Seed 18 play 108 (2.9 s, a triple): first baseman on the cutoff home, shortstop the relay man, second baseman on second.
  - Seed 75 play 109 (2.7 s, a double): first and second basemen out, shortstop on second.
  - Seed 47 play 138 (1.75 s, a double): the same.

  These are right: on an extra-base hit the batter-runner is past first and first is left open.
- **Seed 13 play 224.** The batter stopped at first, a single. With the first baseman and the shortstop both out (1.25 s), the second baseman stayed on first and **second** was open 1.25–2.42 s, with a runner on first at the pitch. Only then did he go to second and leave first open.
- **Seed 93 play 113.** The second baseman goes first → second → first; second is open 0.99–1.45 s and 2.47–3.39 s.

  `cover_table`'s own comment says that with the shortstop on a cutoff "the second baseman is needed at second". Here a cover already taken by the convention stands instead.
- **Seed 107 play 40.** The third baseman and shortstop are both out, and the pitcher alternates cover third / back up home at 1.25, 1.40, 1.55, 2.42, 2.72 and 3.22 s. That is not a mistake a real pitcher makes. Seed 109 play 209 (shallow) shows the same, briefly.

With a runner in scoring position on deep balls, second uncovered went 3 plays/1.45 s (main) → 7/2.5 s (reviewed) → 6/3.7 s (final). The note does not give this.

All of these happen while the ball is still up and change no outcome in the trace. Fix the note's wording, and add the 2B-to-second hand-over and the pitcher's flip to the TASKS relay-review line.

### P3-2. Stale doc comment and a dead field from the dropped variant

- **`single_cutoff_stays`'s doc comment** (`live.rs` 5449–5450) says "as shown at or after the moment `o` made his judgment (so `c`, seeing the same ball, had as long to judge it)". That is the variant the worker measured and dropped (out-back-out 34). The code asks for `c`'s job as known at `t`, with `t − CALL_HEARD_S` at or after `could`: the later of when `o` began counting `c` (`Fielder::cut`) and the moment every infielder was known on this ball.
- **`Fielder::air`'s new third element** (1547; written at 5401: "and when he made it") is never read (`let (made, fp, _)`). Remove it, or say what it is for.
- **The spec's and the note's wording** ("as shown at or after the moment that man could have gone … and every infielder known on this ball plus `CALL_HEARD_S`") reads as if his shown entry must be dated after that moment, and as if two delays are added before it. The code checks:
  - the moment every infielder is first known (their first showing plus `CALL_HEARD_S`) is `could`;
  - his job as known `CALL_HEARD_S` after `could` decides, whenever it was shown (a cover is not re-shown).

  The spec also calls it the gate "a cutoff man toward third or home waits for", but `single_cutoff_stays` applies it toward second too. That is harmless (it only delays the latch), but say so.

### P3-3. Small things

- **`observations.md` line 121** still says "Since 2026-11-22 the same jobs are taken while a line drive or blooper is still in the air" (P3-4 of the first review; the link on that line was fixed, the date was not).
- **`single_cutoff_stays` reads past the horizon** (`live.rs` 5471). It reads each infielder's first `shown` entry on this ball without the `CALL_HEARD_S` horizon. The result equals a horizon-limited read: an entry not yet public gives `could > t − CALL_HEARD_S`, so the answer is `false`, the same as "not yet known". So nothing leaks. But the code depends on that arithmetic; add `e.at <= t − CALL_HEARD_S` to the `find` so that it reads only public entries.
- **The `stays` latch is not keyed by the man.** If the cutoff man changes after the latch (a runner seen going on), the counting man does not anticipate the new one and waits `CALL_HEARD_S` to see him. This is documented ("for good on this ball"), and the cost is one knowledge lag. No change needed.

## Questions answered

**1. Invariants 2 and 5.**

The inputs to `single_cutoff_stays(o, c)`:

- `o`'s own latches (`stays`, `air`, `cut`);
- `c` from `single_cutoff_man(o)`:
  - `relay_base(false, o)`: `o`'s reading of visible runners;
  - `second_cutoff_man`: shown jobs;
  - `home_cutoff_man(single_of())`: `lead()` is known calls plus visible positions, and `seen(side, other)` is a shown job;
- the infielders' first showings on this ball (see P3-3: past the horizon, but equivalent);
- `shown_then(c)`: `c`'s job shown `CALL_HEARD_S` ago, on this ball.

The relay gate:

- `own_single`: his own `air_read`, plus the above;
- `corner_out`: `seen(who, 2|4)`, shown jobs;
- `lands_short`: `self.f[who].belief.first_bounce()`. "His own read has the ball landing short of 80 m" is his own read (`belief`), not the world's path.

`deep_ball`'s `relay_out`:

- `first_down_t` is a visible event in `out.facts`;
- `at_touch(j)` reads `shown` entries up to touch − `CALL_HEARD_S`, filtered to this ball, so it is public once the ball is down (it is only asked after the touch);
- `relay_base(false, 5)`, as before.

Nothing reads a teammate's private read, the true landing point or the outcome. The isolation harness flips `stays` and `cut`, and `isolation_no_private_state_reaches_a_teammate_in_real_games` passes on `84e10f8` (Verification 4).

Determinism:

- No new RNG, hash iteration, transcendental or `mul_add` (`m::hypot` only).
- `track_single_cutoff` writes only `f[o].cut`, with `self.t`, whether it is first called from `update_relay`'s loop or from a decision in the same tick, so the call order does not matter.
- `stays` is written only from calls about `o`.
- The latches are keyed by `ball_gen + 1`, which `set_free` bumps on every new ball (a deflection, or a throw on the ground). `new_sim` builds the fielders fresh for each play, so no latch survives a new ball or the next play.
- A latch survives the outfielder's pickup on the same ball, but every reader of it (`home_cutoff_by_convention`, `second_takes_over`, the relay gate behind `!down`) is gated on a loose ball or a ball still up.

**2. The departure from "shown no earlier than the judgment".** It is sound. A covering man's `shown` entry is not renewed while he keeps the job, so the review's literal test could never pass for a man who simply stays. The worker's version asks for `c`'s job as known `CALL_HEARD_S` after the first moment he could have gone.

Can the latch pin a wrong belief? It ends only `o`'s anticipation (`home_cutoff_by_convention`, `second_takes_over`) and his own single judgment as a reason to stay in (`own_single`). `cover_table` and `cover_after_chasers` still count `c` out from his known job, and the relay gate still stands down for:

- a partner known on a cutoff;
- a corner man known on the cutoff home, on a ball `o` reads landing short.

So when the cutoff man goes out late, the wrong belief lasts one `CALL_HEARD_S`, not the rest of the play.

The traces bear this out. With a runner in scoring position, on the final build:

| Zone | Base uncovered | Plays, total | Longest |
|---|---|---|---|
| Dirt | first | 12, 3.6 s | 0.5 s |
| Dirt | second | 8 | 0.3 s |
| Dirt | third | 6 | 0.3 s |
| Shallow | first | 27, 8.6 s; 24 of them ≤ 0.3 s | 0.75 s |
| Shallow | second | 20 | 0.6 s |
| Shallow | third | 14 | 0.6 s |

Two men out on one relay line for more than 0.5 s: 0 on main, 8 on the final build. All 8 are a corner man on the cutoff home with a middle infielder as the relay man, the deep alignment P2-1 asked for. The costs the worker reports are a covering man leaving a bag 0.3 s before the cutoff man is seen to go late, and the relay man coming back once he is seen. Seed 26 play 162, seed 86 play 48 and seed 77 play 167 are such plays. Those are mistakes a real infield makes when one man is slow to decide.

"Standard alignment" for the deep first-open: partly (P3-1). Who should be at first: on an extra-base hit nobody needs to be, because the batter-runner is past it. The second baseman (relay man to the left side) or the shortstop (to the right side) should be on second. That is so in 3 of the 7 plays, not in 13/224 and 93/113.

**3. P1-1 and P2-1 fixed.** Confirmed (Verification 1–3). The 0.6 s floor: everyone's first showing on a ball is at the first call's tick, so `could` is the first call plus `CALL_HEARD_S`. The latch then needs `t ≥ could + CALL_HEARD_S`. All 23 stretches over 0.5 s begin at the first call (0.65–0.74 s) and last exactly 0.6 s.

**4. The new test.** It is not tautological. It checks two independent things:

- no bag is held by two men beyond the communication delay;
- someone is in the throw's line from the known chase on.

Neither restates the code's latch. On `e16a457`'s `live.rs`, each case fails alone on the two-men assertion:

- seed 61 play 187: third, 0.70–1.32 s;
- seed 21 play 86: first, 1.40–2.02 s;
- seed 77 play 114: first, 0.65–1.27 s.

With that assertion relaxed, each also fails the relay assertion: nobody on a cutoff at 1.30 s, 1.40 s and 1.55 s. The fixtures' premises (base state, first-down distance, the fielder) match the traces. The literals 0.61 and 0.6 stand for 2 × `CALL_HEARD_S`, which is private to `live.rs`; other integration tests use literals the same way.

**5. Spec, note, ledger.** Figures I checked against `air.py`, `hands.py`, `unc_b.py` and the `std-*` files:

- note §3: 25 (10.4 s); 9, 38 of 306, 6; 10; 2, 27/24 ≤ 0.3 s, 7 plays/11.0 s;
- ledger: runs 4.05 [4.39], 4,476 → 4,773 [4,702 → 4,764] ms, +4% over both seed sets.

All match. The ledger row is under 2026-11-21 and the old 2026-11-22 row is gone. No "2026-11-22" is left in the spec, note, ledger, TASKS, code or tests; one is left in `observations.md` (P3-3). Stale text: P3-2.

## Verification

All runs under `taskset -c 0-7 nice`, `CARGO_BUILD_JOBS=2`, one at a time. The worktree stayed clean at `0deb7f1` apart from this file. My throwaway copies are in `sim/target/review-probes/rr/`, built with `CARGO_TARGET_DIR=sim/target/review-probes/liner-tgt`, and my scripts are in `sim/target/review-probes/py2/`.

1. **The review's scripts** on `tr-before.log` (main), `tr-a12.log` (reviewed) and `tr-b8.log` (final), in `review-probes/py/`:
   - `dup.py`, more than 0.5 s: 0 → 32 → **23** (each 0.6 s, starting 0.65–0.74 s);
   - `dup.py`, more than 0.65 s: 0 → 30 → **0**;
   - `norelay.py`: 4 → 15 → **0**;
   - `codex-probes/liner/hands.py`, out-back-out with a runner in scoring position (dirt; shallow; deep): 10; 2; 1 → 5; 9; 7 → **6; 10; 2**.
2. **The final trace's source.** `git archive 84e10f8 sim`, with the worker's `cuthome/zz_cut_home.rs` added as an example, built and run with `BB_COVER_TRACE=1 probe 120 0`. The output `rr/tr-rr.log` is byte-identical to `codex-probes/liner/tr-b8.log` (md5 `5f50b6fe1d6b7c45f0c5e0e4e835dd40`), 3 min 31 s.
3. **The new test, per case.** I added an `ONLY` filter, and a `DUP` override of the 0.61, to the test in two throwaway copies: `84e10f8`, and `84e10f8` with `e16a457`'s `live.rs`.
   - `e16a457`, default: all three fail on the two-men assertion with the messages in Q4 (`rr/e16-cases.log`).
   - `e16a457`, `DUP=99`: all three fail on the relay assertion (`rr/e16-cases-relay.log`).
   - `84e10f8`: the whole `throw_choices` passes, 28 tests. With `DUP=0.57`, seed 61 play 187 and seed 77 play 114 fail (two men 0.70–1.28 s and 0.65–1.23 s, i.e. on the floor); seed 21 play 86 passes (`rr/b8-cases.log`).
4. **Unit tests on `84e10f8`**: `cargo test --release -p bb_game --lib -- isolation ground_single_fielded_deep`. 5 passed, 1 ignored (`isolation_many_seeds`), including `isolation_no_private_state_reaches_a_teammate_in_real_games` (346 s) and `a_ground_single_fielded_deep_is_not_a_deep_ball` (`rr/lib-tests.log`).
5. **My probes** on the three traces (`review-probes/py2/`):
   - `pickup2.py`: hand-overs at the pickup, runner in scoring position, shallow: 8 → 2 → 7, with the plays (P2-1). The first review's `pickup.py`: 1 → 1 → 6.
   - `twoout.py`: two infielders on the same relay line for more than 0.5 s: 0 → 0 → 8, all corner man plus relay man.
   - `pingpong.py`: an infielder going A→B→A→B: 33 → 46 → 32 in all; with a runner in scoring position, 6 → 8 → 13 (seed 107 play 40, the pitcher, P3-1).
   - `flicker.py`: one-tick job flickers: 7 → 1 → 3 (seed 47 play 142: the third baseman out on the cutoff 8 ms before the bounce and taken off the next tick, echoed by the pitcher 0.3 s later; negligible).
   - The worker's `unc_b.py`, all zones and bases, runner in scoring position (P3-1 and Q2's table).
6. **Read**: the whole `941b4f2..0deb7f1` diff of `live.rs` and `throw_choices.rs`. In `live.rs`, the callers and callees of the new functions (`update_relay`, `cover_table`, `cover_after_chasers`, `home_cutoff_by_convention`, `second_takes_over`, `air_cutoff`, `infield_lets_go`, `lead`, `set_free`, `publish`). Also the spec §5.7 word diff, the note, the ledger row and the `std-final-*`/`std-main2-*` outputs.

I did not run the standard run or the whole workspace suite (the worker's `test-final.txt` reports 517 passed). No background job of mine is left running.

## Resolution

Open.
