# Review: the depth read of a ball seen on the ground (`work/depth-read`, `aa4a31d..476fef6`)

Reviewer: Opus (independent; wrote none of the change), 2026-11-24. Job: `REVIEW-DEPTH-READ.md`. Files: `sim/crates/bb_game/src/live.rs`, `src/play.rs`, `tests/throw_choices.rs`, `examples/depth_read_probe.rs`, `docs/agent-models.md` §5.1 *The depth of a ball seen on the ground*, the note `docs/calibration/2026-11-24-depth-read.md`, the newest ledger row.

**Verdict.** The change is small, honest in its geometry, keeps the replay contract (no new draws, no change of draw order, the one persistent depth draw now applied at a fused σ) and does what it says on the play it was built for. One P2: a ground ball knocked off a glove's rim is always popped up, so for its first ~0.2 s every fielder loses the cue. That is the instant the new-ball call round re-reads everyone, so the outfielders' reads fall back to the old error of tens of metres when they are reassigned. The rest are P3: the hop's σ is worked at the wrong range, the tests have no lower bound, the docs leave out some roles that change, and the extra bases are attributed beyond the noise.

## Findings

### P2-1. A ground ball off a glove's rim loses the cue for its first hop, exactly when every fielder re-reads it

- **Where.** `live.rs:7202-7208` (rim deflection: `nv = … + (0, 0, 1.0)` and `r.z ≥ radius + 0.01`, so the new free ball always starts in the air going up at 1 m/s), `live.rs:3575-3582` `on_ground_since` (`None` until the new path's first bounce: no bounce yet, not `seen_rolling`), `live.rs:7357-7369` `set_free` → `assign()` (a new ball starts a call round, so every candidate calls `read` → `predict` now). After a touch every fielder has no track (agent-models §5.1, "a touch … ends the track"), so all of them read it by the snapshot read with no cue.
- **Failure scenario (reproduced).** In a copy with the cue made 10× sharper (my mutation, below), seed 65147 play 10 changes course: the second baseman knocks the ball down at about 2.23 s (the ball goes from about 20 m/s to about 3 m/s, z 0.15 → 0.20 m, next ground touch at 2.528 s). The right fielder, 44.6 m away, had been reading it to 0.2–0.3 m in depth. From 2.24 s his read is **9.4 m** off along his line of sight. It stays 8.7–9.5 m off through 2.54 s, because the read made at the call round is kept until his next replan. That is the old snapshot error (`DEPTH_STRETCH`·3.5 m·44.6/30 m, times his draw and the viewing-time shrink) coming back for one hop. In the unmutated branch the same happens on every rim deflection of a ground ball. I did not count how often (dives alone are 2.8 a game).
- **Why it is wrong.** The spec says the cue is absent on a ball that "came off … a glove". But a grounder knocked down by an infielder leaves the glove a few centimetres above the grass, at a point the outfielder sees on the ground. Its angle below his eyes gives its range almost as well as a rolling ball's. A real outfielder does not lose 10 m of depth because a ball popped 5 cm off a glove. The note's headline (chasers more than 20 m off: 11.8 → 0.4 a game) is measured only up to the first secure and does not see this window.
- **Suggested fix.** Treat a new free ball that starts low as seen on the ground at its start: in `on_ground_since`, when the path has no bounce yet and its start height is under a named [E] height (e.g. 0.3 m), return the time since the path's start. Add the height doubt to the σ if wanted (a ball up to z₀ moves the range by about d·z₀/h, which is 2.7 m at 45 m for 0.1 m, still far better than 15 m). Change the spec line "Not on a ball … came off … a glove" to match, and add a test: a rim deflection of a grounder, where an untracked outfielder's read right after `set_free` is within the cue's bound. Balls off the wall in the air (a known plane) can stay open.

### P3-1. On a hop, the cue's σ is taken at the range now, not at the range of the bounce it came from

- **Where.** `live.rs:2725-2727` (`ground_range_sigma(dist, …)` with `dist` the ball's level range now), the spec's "d the range now" (agent-models §5.1, *When he has it*).
- **Failure scenario.** For a hop coming toward him, the angle was measured at the bounce point, farther away, so the model is too sharp. Probe over six grounders/choppers from the unit fixture, all nine fielders, every 0.05 s on a hop: the worst honest/model ratio is **1.50**. That is the shortstop, ball now 30.1 m away, bounced at 43.0 m, 0.53 s ago: model 0.76 m, honest 1.14 m. On a hop going away from him, the model is too loose (that is harmless). The run-on term (1.5·σ_v·s) is also folded into the single persistent depth draw as if independent. Physically it is his own velocity misread along the line of sight (the `err[2]` draw already applied to `ve`), so its sign should follow that draw.
- **Fix.** Pass the last bounce's level range in `ground` and use it in the declination term. Keep the run-on term (or tie it to `err[2]`, the honest version). Spec: "the cue at the bounce point's range".

### P3-2. No test fails when the cue is too sharp; the frozen fixture test is a replay bound, not a bound on the read

- **Where.** `live.rs` `a_ball_on_the_ground_is_read_where_his_eye_puts_it` (upper bound only: `off ≤ |e|·σ_δ·1.15`), `tests/throw_choices.rs` `seed_65147_play_10_the_right_fielder_reads_the_chopper…` (upper bound 8 m on a re-simulated play).
- **Failure scenario.** Mutation `fd.a.ground_decl_sigma * 0.1` at the call site (`live.rs:3561`): `a_ball_on_the_ground_is_read_where_his_eye_puts_it` **passes** (187 reads of the rolling ball, 4 on a hop). The fixture test fails, but only because the play changed (the second baseman now fields it, after the P2-1 deflection) and the deflection window put the read 9.5 m off. A better read failed a test that is meant to bound how bad the read is. So the fixture test guards "this play goes this way", not the read's quality. `seed_65147_play_10_an_outfielder_takes_a_grounder…` failed too, for the same reason.
- **Fix.** Make the rolling-ball test two-sided where the cue dominates. For an outfielder whose snapshot depth σ is at least 5× the cue's, the fused σ is at least 0.98 of the cue's whatever the weighting, so `off ≥ 0.85·|e|·σ_δ` is independent of the formula. That fails under the 10× mutation and under "perfect depth". Say in the fixture test's comment that it bounds this play only while its course holds.

### P3-3. "Who else changes" is incomplete in the note and the spec

- **Where.** Note §3 "*Infielders*", spec *Infielders, knowingly changed*, ledger "Cause".
- `predict` serves every untracked `read`: the call round (`assign_among`, every candidate), `retake` (an idle fielder choosing a cover or backup job), the fallback chaser and `close_reads`. So the cue also changes:
  - the **pitcher and catcher** (infield σ0 0.3, e.g. a comebacker bouncing inside the pitcher's reaction);
  - **idle fielders' choice of job** (the note's own seed 1 play 66 centre fielder now takes a backup);
  - every fielder's read of a ball **rolling after a wall carom, a glove touch or a throw that came down**. Tracks end at those, so after them everyone reads by the snapshot read, and now with the cue once the ball is on the ground.
- All of these are physically right. The docs should name them, because the counterfactual (outfielders only, before any wall) did not include them (see P3-4).
- Unchanged, as claimed: tracked reads, the first flight (`off_bat` is `Some` only before the first bounce, and the cue only after it), runners (`runner_ball_errors`, their own path) and receivers (`Ball::Thrown` → `on_ground_since` is `None`). `landing_depth_sigma` (the call's doubt) still takes no cue, so an untracked infielder about to glove a hop off the ground is sure of his read and doubts it as before. The spec lists this as open; it is a small inconsistency, not a leak.

### P3-4. The extra bases claimed go beyond the perfect-depth bound and are within noise

- **Where.** Ledger row "Cause" ("a few extra bases go (triples −0.06 [−0.01] a game)"), note §5.
- **Numbers (probe, 300 games; jobs 256, 258).** Ground-ball doubles 362 → 355 [371 → 334], triples 108 → 89 [93 → 89]. The counterfactual with no outfield depth error at all gives doubles 357 [360] and triples 96 [83]. Doubles plus triples over both sets: main 934, this branch 867, the counterfactual 896. The branch moves more than the bound it is supposed to sit inside. Each set's games diverge after the first changed play, so the counts are close to independent: the doubles move at seed 1000 is −37 against a spread of √(371+334) ≈ 27 (1.4σ), and the triples move at seed 0 is −19 against √197 ≈ 14 (1.4σ). The 10% doubles move is not explained by the outfielders' read. It is noise, or the parts the counterfactual left out (P3-3: infielders, pitcher, catcher, balls after a wall or a glove).
- **Fix (wording only).** Say "within noise; beyond the outfield-only perfect-depth bound, so not attributable to the outfielders' faster pickups". Ground-ball hits did not rise (.307 → .309 [.312 → .308]; the counterfactual .309 [.313]), which is fair, and runs 4.09 → 4.10 [4.37 → 4.32] is within the sets' spread.

### P3-5. The 3′ figure: defensible, but anchor it, and eye height is a second error the model leaves out

- **Where.** `live.rs:777-781` `GROUND_DECL_SIGMA_RAD`, agent-models §5.1 and §10.
- **Comparison.** The model's range σ as a share of range is (d² + h²)/(h·d)·σ_δ: **1.6% at 30 m, 3.2% at 60 m, 4.2% at 80 m, 5.2% at 100 m**. That matches "a few percent of range at tens of metres" for action-based distance judgments on a ground plane (Ooi, Wu & He 2001 show the cue and its use, not a σ). At 30 m it is at the sharp end.
- **Anchor.** The sim's own `Vision` σ_ang at grade 50 is 1.0 mrad = 3.4′ (agent-models §10), within 15% of 3′, and is a better in-house anchor than "the ball's angular size at 80 m".
- **Missing error.** His eye height bobs while he runs. An error σ_h in h moves the range by d·σ_h/h, which is 1.8–3% of the range for σ_h = 3–5 cm. That is as large as the angle term inside about 60 m. It would make the cue a little less sharp for infielders and is not modelled.
- **Checks.** It is named, [E], on `FielderAttrs` (`ground_decl_sigma`), and scaled by `ReadSkill` with `READ_RHO` in the same direction as the other read σs (`play.rs:136`; 1.97′ at 80, 4.56′ at 20, consistent with the §10 table).
- **Fix.** Cite the Vision σ_ang as the anchor, keep [E], and note the eye-height term as open.

### P3-6. Small doc nits

- `observations.md` line for seed 65147 play 10 says "(branch `work/depth-read`)". That is stale once merged; say "2026-11-24".
- Spec: "for an outfielder under 600 m" — the crossover is 667 m. The statement is true but the figure looks computed, so say ~670 m or "anywhere on a field".
- `python3 bin/check-docs.py` is clean apart from the untracked `REVIEW-DEPTH-READ.md`, which names this file.

## Answers to the job's questions, briefly

1. **What he sees.** "Where it last touched the ground" and "the time since" are the perfect phase events the spec already grants (bounce times and `seen_rolling` are read from the world, as the tracking read already does). The true level range for σ follows the snapshot read's existing convention. Rolling or not is the same observation boundary as before. No new leak. On a hop, using the bounce point is physically right: the ball in the air is not on the plane. But the σ is taken at the range now (P3-1). The σ growth (1.5·σ_v·s in quadrature) is the right size; its correlation with the velocity draw is ignored.
2. **Fusion.** It is precision weighting of two independent σs. `fd.err[0]` is the same persistent draw, now applied at `sd`. No RNG call is added and the draw order is unchanged (`read_prediction_g` takes `err` from `fd.err` as before). Jumps:
   - at the first bounce, `sd` drops from `DEPTH_STRETCH`·σ_p (28 m at 80 m) to about σ_g. That is a one-off jump toward the truth, as a real fielder's first look at the bounce would be;
   - hop → roll: the run-on term goes from √(…+(1.5σ_v s)²) to the bare cue, a few tenths of a metre;
   - at a glove: P2-1, a real jump and back;
   - off the wall in the air: the same flicker, off a known plane, left open.
   Edge cases:
   - d → 0: `dist.max(1.0)` gives σ_g ≈ 2 mm. With σ_δ = 0 and since = 0, 1/0 → ∞ → `sd` = 0, no NaN;
   - above eye level: h is fixed and the cue is taken from the bounce point, so the hop's height does not enter (right);
   - warning track, ball at rest against the wall: `seen_rolling` is true and d is small, so the cue is sharp and correct.
3. **Who else.** P3-3.
4. **The number.** P3-5.
5. **Tests.** The geometry test is independent (h/tan(atan2(h,d)±σ)) and fails with d²/h. The rolling test has no lower bound (P3-2, mutation run). The fixture test is a replay regression bound (P3-2).
6. **Measurements.** Fair for runs and ground-ball hits. The doubles and triples are over-attributed (P3-4). Job 258's standard runs were from a stale `sim-cli` (identical to main to the digit); the worker saw it and reran in job 260, which the ledger uses. Good.
7. **Docs.** P3-6. The spec's numbers match the code: 0.47/1.89/3.36/5.25 m at 30/60/80/100 m; the infield 0.33 m against 0.75 m fused to 0.30 m; the depth factor 1.5 = `0.5·DEPTH_STRETCH`; and the range-rate σ√(12·`LOOK_S`/T³) is the least-squares slope σ (3.7 m/s at 80 m, 1 s).

## Verification

- Read the diff `aa4a31d..476fef6` (`live.rs` `read_prediction_g`, `ground_range_sigma`, `on_ground_since`, `predict`; `play.rs`; the three tests), the callers of `predict`/`read` (`live.rs:4395, 4411, 4767, 5218, 5394, 6927`), `set_free` and the rim deflection (`live.rs:7202-7208, 7349-7369`), `update_thrown` (`7537-7568`), `off_bat`, `landing_depth_sigma`, `call_value`.
- Job logs 255 (main and counterfactual standard runs), 256 (probe, main against the counterfactual), 258 (workspace tests: all pass; the standard runs in it equal main's, a stale `sim-cli`; the probe after), 260 (fresh `sim-cli` after: runs 4.10 [4.32], GB .309 [.308]). The ledger's figures match them.
- **Mutation and probe in a copy** (`sim/target/review-probes/ws`, `CARGO_TARGET_DIR=sim/target/review-probes/target`; queue jobs 261, 262):
  - (a) `ground_decl_sigma * 0.1` at `live.rs:3561`: `the_range_of_a_ball…` ok; `a_ball_on_the_ground_is_read_where_his_eye_puts_it` **ok** (should fail: P3-2); both seed 65147 play 10 tests in `throw_choices` fail, because the play changes (second baseman fields it at (15.4, 46.4)). The trace in job 262 shows the right fielder's depth error going 0.3 m → 9.4 m at 2.24 s, the instant the ball came off the rim (P2-1);
  - (b) a throwaway test `review_probe_hop_range` (six grounders, every fielder, every 0.05 s on a hop): honest/model σ up to 1.50 (P3-1).
- Arithmetic (python): the cue's σ at 30/60/80/100 m is 0.47/1.89/3.36/5.25 m = 1.6/3.2/4.2/5.2% of range; `Vision` 1 mrad = 3.44′; the crossover with the outfield snapshot is 667 m and with the infield snapshot 57 m.
- `python3 bin/check-docs.py`: clean apart from the untracked job file's reference to this review.
- No source, test, spec, TASKS or observations file was edited. The copy and its target are under `sim/target/review-probes/` (gitignored).

## Resolution

Resolved 2026-11-24 by the change's author (Opus, `68db91e`..`1efeade`; per finding in the [depth-read note](../calibration/2026-11-24-depth-read.md) §8). No second review: the fixes are small and each has a test; the Coordinator's pre-merge run (job 265) passed the workspace tests and repeated the worker's standard-run figures.

- **P2-1** fixed: a new free ball starting within `GROUND_START_Z_M` (0.3 m [E]) of the ground is read as on the ground where it started, with its height as a doubt; test `a_grounder_off_a_gloves_rim_is_read_on_the_ground_where_it_left_the_glove`. Two ground-dive fixtures were moved back into their stated windows (their assertions unchanged).
- **P3-1** fixed: the hop's declination σ at the bounce point's range. The run-on term stays independent of `err[2]` (decided, note §8).
- **P3-2** fixed: the rolling-ball test is two-sided and fails with the cue ten times sharper.
- **P3-3, P3-4, P3-6** fixed in the note, spec, ledger and observations.
- **P3-5** fixed: 3′ anchored to `Vision` σ_ang, kept [E]. **Deferred:** the eye-height term (note §7).
- **Not covered by a test:** dropping the height doubt (z/d) itself.
