# Independent review of the 25–50° double-offs (`work/fly-read`)

Reviewer: Claude Opus 5.5 (wrote none of the change). Reviewed `48f36a9..925f8ea` on `work/fly-read`. Sim code last changed in `b52e848`, in `sim/crates/bb_game/src/live.rs` and `sim/crates/bb_physics/src/flight.rs`. Job file: REVIEW-FLYREAD.md.

## Summary

The change does what the [fly-read note](../calibration/2026-11-19-fly-read.md) says, and I found no P1.

- **World flight is unchanged.** On 300 games, the first batted ball's world flight is bit-identical between main and the branch: every knot's position, velocity and spin, the apex, and the bounces. The workspace tests pass, including the oracle flight test.
- **The belief and both relooks use only what the observer can see.** That is his noisy look at the ball, the public contact clock and rulings, the fielders' visible positions and motion, his own place and speed, and keyed draws.
- **The tables are fitted from simulated population data** (seeds 0–95) and tested on seeds outside the fit.
- **Both new relook fixtures fail when their relook is switched off.**
- **Same-seed digests are identical.**

**The run drop is not a bug.** Almost all of it comes from step 1 alone, the belief refit. The relooks and the 50% cut-off give a little back. On a third seed set the drop is −0.07. Liner hits fall in all three sets, and a probe shows why: on main, a zero-error read carried 10–25° liners 2.5–5.7 m long. That bias is now gone.

The one substantive finding is **P2-1**. The 50% tag cut-off from second also applies with one out. With one out, a tag-up advance is worth almost nothing (about 0.03 runs), but going back to the bag on a ball that drops costs the runner his chance to score. In the branch's own games, runners on second with one out who went back at 50–80% scored on 8% of the balls that dropped in the park. Runners who held partway at 20–50% scored on 38%. That fits the wider gap in second-to-home on singles (53.8% against MLB's 61.7%).

The rest are P3 records and stale text. **The branch is safe to merge with P2-1 fixed, or deferred as an owner question.**

## Findings

### P2-1. The 50% tag cut-off from second applies with one out, where tagging gains almost nothing and costs runs on balls that drop

[live.rs:144](../../sim/crates/bb_game/src/live.rs#L144) (`RUNNER_TAG_FROM_SECOND`), used through `tag_confidence` ([live.rs:2665](../../sim/crates/bb_game/src/live.rs#L2665)) at the first look ([live.rs:2672](../../sim/crates/bb_game/src/live.rs#L2672)) and on both relooks ([live.rs:2709](../../sim/crates/bb_game/src/live.rs#L2709), [live.rs:2725](../../sim/crates/bb_game/src/live.rs#L2725)).

**In baseball terms.** With one out, a runner on second goes back to tag on any fly he reads as at least a coin flip to be caught. Suppose it is caught and he takes third. That moves him from (second, two out) to (third, two out), which is worth about 0.03 runs (RE24: 0.32 → 0.35; with none out, (second, one out) → (third, one out) is 0.66 → 0.95, worth +0.29). Suppose instead it drops. He is standing on second and has to wait for the ball to come down, where a runner halfway would score. Real runners with one out go halfway on a fly in doubt and tag only when they are sure. The note's run-value argument (§3, "Sweeps") covers none out only. It then keeps one value for both because MLB's pooled advance rate with one out is still 26%. Those advances come mostly on deep flies that are sure catches, which the 80% cut-off would also send back.

**Measured** (instrumented copy of the branch, default behaviour unchanged, standard sets `games --n 300 --seed 0` and `--seed 1000`; runner on second, 0–1 out, ball dropped in the park):

| Outs | First look | Runners | Scored | Ended on third | Stayed on second | Out |
|---|---|---:|---:|---:|---:|---:|
| 0 | 50–80%, back to tag | 52 | 8 (15%) | 33 | 9 | 2 |
| 0 | 20–50%, held partway | 60 | 21 (35%) | 21 | 15 | 3 |
| 1 | 50–80%, back to tag | 88 | **7 (8%)** | 56 | 18 | 7 |
| 1 | 20–50%, held partway | 97 | **37 (38%)** | 29 | 23 | 8 |

On caught balls from the same 50–80% group, the tag-up reached third on 40 of 329 with none out and 57 of 540 with one out. The two score-read groups have different balls (a 50–80% read is nearer a fielder), so this is not a clean counterfactual. Still, with one out the 57 advances are worth about 2 runs in 600 games, and the drops cost several times that. Plays to look at: game seed 133 play 46 (34.8°, drops at 68 m, one out, the runner on second ends on second) and seed 96 play 39 (50.4°, drops at 66 m, same).

**Fix.** Split the return cut-off from second by outs. Keep 50% with none out, where the note's run-value argument holds. With one out use 80%, or a value from the same run-value break-even as the advance cut-off (agent-models §6.3 "What he does"). Then remeasure the tag-advance rate by outs against Retrosheet (37.3% / 25.9%) and second-to-home on singles. If the owner prefers one value, record that as a decision.

### P3-1. The tables' fit stopped before it settled

[batted_spin.rs:96](../../sim/crates/bb_game/examples/batted_spin.rs#L96) runs four sweeps, and the last one still moved a cell by 700 rpm (note §1). The tables in live.rs are therefore one sweep's snapshot, not a fixed point. That is harmless at the current precision: the out-of-sample test's cell misses are +1.7 m at most and +0.22 m pooled. **Fix:** sweep until the largest change is under about 50 rpm, then refit. It is cheap.

### P3-2. Test comment and bound disagree; the liner floor has one catch of margin

The typical-spin test's comment says the backspin matches "within 15%" ([live.rs:8900](../../sim/crates/bb_game/src/live.rs#L8900)), but the assertion allows 20% ([live.rs:8945](../../sim/crates/bb_game/src/live.rs#L8945)). The worst cell measured 7.5% (1,876 rpm against 2,017), so restore 0.15.

The liner fixture's catch floor fell from 18 to 16 ([live.rs:11259](../../sim/crates/bb_game/src/live.rs#L11259)) against a measured 17 of 24. That is within binomial noise of the old ≥18, so it is not evidence of a regression, but the floor now guards little. Its real assertion, no double-off, is unchanged.

The test also checks none of the 15° row's carry, where the fitted unseen spin is largest. My liner probe below covers it out of sample and it passes. Adding the 10–25° row with a looser per-cell bound would keep it covered when the contact model changes.

### P3-3. A throwaway environment switch was left in a test

The read-skill test's fly set reads `ZLO`/`ZHI` from the environment ([live.rs:9332](../../sim/crates/bb_game/src/live.rs#L9332)), a sweep left behind. Restore the literal 18.0..28.0.

### P3-4. Docs do not match the code on the relooks

- **data-formats §2.3** ([data-formats.md:66](../data-formats.md)): sub-index 2^21 + k is described as a look "after going at his first look". The holding relook draws from the same sub-indices.
- **agent-models §6.3**, "He keeps watching while it is up": both the going and the holding relook are described as going back at `RUNNER_CATCH_CONFIDENCE`. From second the code uses `RUNNER_TAG_FROM_SECOND` (50%) on both.
- **The note's §1 test figure is stale.** `a_zero_error_first_look_reads_deep_flies_without_bias` now prints a median of −0.10 m and 47% deeper on 205 flies, in debug and in release alike; the note says −0.33 m and 46%. Step 3 changed the games, and so the sample.

### P3-5. Why the relook leaves double-offs below MLB (a record for the open item, not a defect)

On the question of a too-perfect relook: each look draws fresh, independent noise (sub-index 2^21 + k). Its σ shrinks with watching time and, through `read_shrink`'s closing term ([live.rs:1597](../../sim/crates/bb_game/src/live.rs#L1597), `READ_CLOSE_S` = 3 s, fitted for fielders), goes linearly to zero as the ball comes down. Nothing of a wrong read carries over from one look to the next.

So a runner who went on a fly that will be caught always sees the catch coming one or two looks before it happens. Allocated on the standard sets (600 games, 0–1 out), runners put out on 25–50° catches, all causes, were:

- step 1 alone: 28;
- steps 1–2: 12;
- full build: 9, of which 2 are double-offs and the rest are runners from third thrown out tagging.

Both double-offs are runners from first who held partway and were caught off the bag on 95–116 m catches. That is a mistake a real runner makes. The full build's double-offs are about 1 per 300 games against MLB's 4. If the owner wants MLB's misjudgments back, the levers are: a runner's own closing time instead of the fielders' `READ_CLOSE_S`, a read error that carries over from one look to the next, or the per-catch-type cut-off the note proposes. This is not needed for the merge.

## Answers to the job's questions

1. **World physics.** `unseen_spin = 0.0` keeps the old arithmetic exactly (`w.norm()` branch, [flight.rs:71](../../sim/crates/bb_physics/src/flight.rs#L71)). Every world env comes from `FlightEnv::default_env()` (game.rs, pa.rs, cage.rs, play.rs, the throw table). Every `..` spread that carries a non-zero `unseen_spin` is in agent code: `Track::run_to`, `fly_estimate`, `ContactLook::project`, `read_prediction_k`, the first-look plate cue, `public_path`. Probe: identical (see Verification).
2. **Observations only.** Yes. Each check named in the job passes. The belief is looked up from the observer's own read velocity and the public contact clock. The relook goes through `observe_ball` (noisy motion, public `contact_since`), `runner_catch_paths` (visible fielder position and velocity, typical speed) and `catch_hold_s` (his own state, typical arm and tag). The holding-relook time to the bag uses his own signed speed. The stop conditions are public rulings and `Ball::Free`. The tables come from world launch states of seeds 0–95, as population physics a player could have learned. The bias tests use seeds 100–123.
3. **Would a real runner do this.** The relook itself, yes (P3-5). The 50% cut-off with one out, no (P2-1). Runners on third who go back on a holding relook score on 1 of 89 caught balls. That looks like a bug at first, but those are short flies (median landing 47 m, 10th–90th percentile 20–68 m), where the runner's first look already judged the tag-up race lost. Holders who tagged at their first look are on deep flies (median 97 m) and score on 229 of 400.
4. **The run drop.** Step 1 alone gives runs 4.26 [4.50] against main's 4.48 [4.55]. Steps 1–2 give 4.28 [4.50], and the full build 4.29 [4.52]. So the refit is essentially all of it.
   - **Noise.** A third set (`--seed 2000`) gives main 4.43 against 4.36. Over the three sets the change is −0.19, −0.03 and −0.07. One 300-game run's runs/team-game has a standard error of about 0.12 (σ ≈ 3 runs per team-game over 600), so the gap between seed 0 and seed 1000 is noise.
   - **HR/PA** moves +0.01, −0.09 and +0.04 points: noise. No home-run path changed.
   - **LD hits** fall in all three sets (−.013, −.003, −.015). The liner probe shows main's belief read liners long and the new one does not (Verification). This is fielders playing liners better, not a misplay.
   - **Cells with 1–2 balls** (55° at 100+ mph) are not reached by any measured rate.
5. **Tests.**
   - The two relook fixtures fail with their relook off and pass with the other one off.
   - The typical-spin test checks the belief's carry against the world's own flight out of sample, so it is not tautological. Its backspin bound is looser than its comment (P3-2).
   - `typical_ball` builds fixture balls from the belief. That is acceptable where read error is not under test, and it is said in each comment.
   - The plate-cue test now stops looking once the ball is on the ground. That is a correction, not a weakening.
6. **Determinism.** Sub-index 2^21 + k with k ≤ 80 (a 40 s play limit at 0.5 s per look) cannot meet 2n / 2n + 1 (n is the decision count) or 2^20 + 0..8. No `HashMap`, `mul_add` or direct transcendental in the sim diff; only the `batted_spin` fitting tool uses `f64::atan2`/`hypot`, and its output is pasted as constants. Same-seed digests are identical (Verification).
7. **The 50% cut-off.** It is a named [E] constant with a band (Retrosheet 2024 tag-ups from second, 30.2%) and a mechanism (runners on second never tagged at 80%). First and third stay at 80% (`tag_confidence`). The outs split is P2-1.
8. **Long liners in the plate cue.** No harm found, though I did not measure the plate's gain on a relook directly.
   - **Fielders:** they use the plate cue only at their first look, about 0.35 s after contact.
   - **Runners:** a runner's relook at 3–4 s weighs the plate against a direct read whose σ has closed with the ball's descent, so the plate's share is small. The routine-fly rates did not move.
   - **Liners overall:** the liner probe's zero-error reads from the true state are unbiased (Verification).
   - **Docs:** see P3-4.

## Verification

All builds used `taskset -c 0-7 nice`, `CARGO_BUILD_JOBS=2`, and ran one at a time. The throwaway trees and probes are in `sim/target/review-probes/`: `main-tree` from `48f36a9` and `br-tree` from `925f8ea`, each built in its own target directory there.

- **Workspace tests.** After touching `live.rs` and `flight.rs`, `cargo test --workspace` passed (exit 0), including `flight_matches_oracle`, both relook fixtures, the typical-spin and zero-error bias tests, and the liner double-off fixture.
- **World flight** (`zz_world_flight.rs`). The first contact of games 0–299 was hashed bit for bit over every flight knot (t, r, v, ω), the apex and the bounces, in both trees. The combined hash is `1f8c05d9cf2fa4c4` in both, and the per-game lines are identical; 256 of the 300 are flights of more than 200 knots.
- **Standard run.** The branch copy's `games --n 300 --seed 0 / 1000` gives 4.29 [4.52], LD .670 [.683], HR/PA 2.79% [2.77%], matching the ledger row.
- **Allocation.** The same build with `ZZ_RELOOK=off ZZ_TAG2=0.8` (step 1) gives 4.26 [4.50], LD .667 [.681]. With `ZZ_RELOOK=nohold ZZ_TAG2=0.8` (steps 1–2) it gives 4.28 [4.50]. On `--seed 2000`, main gives 4.43, LD .691, HR/PA 2.37%; the branch gives 4.36, LD .676, HR/PA 2.41%.
- **Runner trace.** An `eprintln` at the end of each live play, gated on `ZZ_TRACE`, logs each runner's first-look score, any relook tag-up, launch angle, outs at the start, catch, end base, off the bag at the catch, and landing distance. It is the source of P2-1, P3-5 and answer 3.
- **The switches are inert when unset.** The instrumented copy's `examples/digest` gives the same digests as the worktree's build.
- **Liner probe** (`zz_liner_read.rs`, seeds 100–139, outside the fit). For fair balls, the belief was flown from the true state at 0.35 s, new against main's 1-D backspin. Median landing miss, new against main:

  | Launch | New | Main |
  |---|---:|---:|
  | 10–15° | +0.58 m | +2.53 m |
  | 15–20° | −0.34 m | +4.47 m |
  | 20–25° | +0.41 m | +5.68 m |
  | 25–35° | −0.38 m | +4.38 m |

  At 60 m from home the new belief's height is +0.09 to +0.33 m and its time +0.01 to +0.04 s; main's height is +0.30 to +0.89 m and its time −0.01 to −0.03 s.
- **Relook fixtures, release, by `ZZ_RELOOK`:**

  | `ZZ_RELOOK` | Home-run read fixture | Holding fixture |
  |---|---|---|
  | unset | passes | passes |
  | `off` | fails ([live.rs:11036](../../sim/crates/bb_game/src/live.rs#L11036) in the copy) | fails ([live.rs:11085](../../sim/crates/bb_game/src/live.rs#L11085) in the copy) |
  | `nohold` | passes | fails |
  | `nogo` | fails | passes |

- **Test margins** (`--nocapture`). The typical-spin cells miss by −0.85 to +1.69 m (pooled +0.22 m), with backspin within 7.5% of the table. The zero-error test gives −0.10 m and 47% deeper on 205 flies in debug and release alike.
- **Digests.** `examples/digest` (seeds 1, 2, 3, 42) run twice on a fresh release build of the worktree gives identical output (seed 1 `9867a9dd…`).

## Resolution

Open.
