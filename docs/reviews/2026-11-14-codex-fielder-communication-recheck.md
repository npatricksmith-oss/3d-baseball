# Independent re-review: fielder communication

Reviewed `c0976f3..e02ade8` on `work/live-comm`, principally `48bcb9d`; excluded the main merge `e675bbb`. Read `CLAUDE.md` first, the previous communication review and its Resolution, the Opus infield review, agent-models §5.7, the calibration note and ledger. Also followed the holder decision path into `live/holder.rs`.

**P1-1 is not fully closed.** The two reported coverage/backup leaks are fixed, with meaningful regression tests. Other private-state paths remain, and sampled games contradict the claimed one-hearing-interval bound on coverage gaps. The revised numerical attribution reproduces. The ordinary same-tick deflection and corner-caller tie tests pass, but the ball cache still fails on two replacements in one tick.

## Findings

### P1-1 — Private jobs still affect other fielders through relay routing and scheduling

**Locations:** `sim/crates/bb_game/src/live/holder.rs:43`; `sim/crates/bb_game/src/live.rs:2184`, `2208`, `2080`, `2089`. The prior review's Resolution overstates closure at `docs/reviews/2026-11-14-codex-fielder-communication-review.md:97`.

The holder's `throw_race()` still discovers the relay by scanning **actual** `role`, rather than the holder's delayed knowledge. Updating `receiver_for()` did not update this separate receiver/route selection. It feeds both out-attempt evaluation and the return throw.

**Isolation probe:** CF holds the ball at `(0,125)`, 2B stands at `(0,60)`, with a runner starting on second. Keep the ball, all bodies, attributes and publications identical; change only 2B's private role Idle → Relay(4). `known_job(2B)` is `(Idle,None)` in both cases. CF's race to home changes from `(relay=None, through=false, ball_at=∞)` to `(relay=Some(2B), through=true, ball_at=5.639239)`. The assertion that the holder's choice stays unchanged fails.

Two scheduling paths also fail isolation:

- `update_relay()` first finds the actual relay globally. With CF holding at `(45,55)` and identical public state, changing only SS's unheard role Idle → Relay(4) changes 1B's result from Relay(4), target `(16.5043,20.1719)`, to Cover(1). The existing relay is retired first; on a held ball the replacement is not independently assigned in that branch. This is not merely SS consulting his own job.
- `expire_peels()` invokes the shared `recall()` only if **someone's private `peel` exists**. Starting from the existing called-off-infielder pop-up fixture, let the calls be heard and advance to just after landing. Keep every publication and body fixed; remove only 1B's private peel marker, retaining his Backup role. With the marker, other men reread and change targets/jobs; without it they do not. The probe excludes 1B from its comparison. A public landing can justify rereading, but whether teammates receive that opportunity cannot depend on an unheard private marker.

These are constructed isolation cases, not estimates of their game frequency. The broad diagnostic matrices found no additional outsider changes in the tested cover tables, backup tables, `recall`, `relay_job`, secured-ball response, return home, throw backup and base-receiver cases when varying another man's role or private plan. Those passes do not cover the failing scheduling contexts above.

**Suggested fix:** use the holder's `seen(by,i)` for relay eligibility under the current spec. Evaluate each prospective cutoff man's decision independently, without another man's current role deciding whether he runs this tick. Trigger teammates' rereads from public events/their own schedules, rather than another man's private peel. Retain the three failing isolation probes as regression tests and audit scheduling gates as well as table inputs.

### P1-2 — Recovery can send all nine, then repeatedly abandon the ball and base coverage

**Locations:** `sim/crates/bb_game/src/live.rs:1793`, `1921`, `1935`, `2371`; first-base fall-through at `1675`. Contradicted descriptions: `docs/agent-models.md:495`, `497`; `docs/calibration/2026-11-14-fielder-communication.md:54`, `102`.

When nobody is known to be chasing, `hear()` lets all recent readers reconsider. `first_by_own_estimate()` excludes every teammate known not to be chasing this ball. After an initial mutual deferral, that can exclude **everyone**, so all nine independently choose Go, including men covering bases. Once those chases become known, their differing estimates can make them all stop again. Their own stopped chase continues reserving them out of the cover tables until its publication is heard. This produces a feedback cycle, rather than the note's “one goes” recovery.

**Real-game evidence, raw game seeds 0–49:** on seed **25/play 94**, all nine start at `t=0.967317`, all stop at `1.267317`, and all nine start again at `1.567317`. The cycle repeats at 0.6-second spacing; there are eight all-nine starts. First, second and third have **no assigned cover for 4.800 s**, `0.967317..5.767317`. Across the sample, **26 plays** have repeated all-nine starts.

The following counts measure contiguous **loose-ball intervals after initial assignment with no `Role::Cover(base)`**, ending at a cover or a non-loose phase. They measure missing jobs, not the normal travel time to a covered bag. Identical seeds produce different later plays, so this is a distribution comparison, not a paired-play causal decomposition.

| Base | Main `5ba6750`: intervals >0.3 s | Final: intervals >0.3 s | Longest final gap |
|---|---:|---:|---:|
| First | 0 | 152 | 4.800 s |
| Second | 7 | 220 | 4.800 s |
| Third | 43 | 171 | 4.800 s |

Home is omitted because the bases-empty catcher deliberately backs up first. Final double-cover intervals were bounded: 32 at first, 40 at second, eight at third, all at most 0.300 s; none at home. The asserted bound holds for those overlaps, not for uncovered bases.

**First-base protection is not general.** There are 34 grounder first-base gaps longer than 0.3 s. Restricting to gaps before the first possession, on grounders first secured by an infielder, four remain: seeds/plays **11/32, 19/158, 27/111, 35/108**, each **0.600 s**. For example 19/158 has first uncovered at `0.958229..1.558229`, before 2B fields it at `1.791563`. The dedicated pitcher/first-baseman regression tests pass, but do not cover this recovery path. A rejected `grounder_break()` can still fall through to Go after all presumed competitors have been excluded.

**The lead-gap claim needs a distinction.** Seeds 0–19 reproduce **32 intervals of 0.600 s**, about 2.1% of the measured no-known-lead intervals; none exceeded 0.600 s. However, there are **50 actual-no-chaser intervals in 34 plays**, not only the initial 32: eight plays repeat the all-nine cycle. Across 0–49 there are 67 long knowledge gaps and 129 actual-no-chaser intervals. Each individual actual gap is at most 0.300 s, but repeated gaps and delayed releases of cover reservations leave bases open much longer. The note's precise denominator (1,490) differs slightly from this end-of-tick instrument's 1,495; the 32 long cases and approximate percentage reproduce.

**Suggested fix:** give no-chaser recovery a publicly computable, stable fallback convention, and ensure a man does not abandon his commitment merely because the delayed eligibility set changes. Preserve first-base protection through the fallback path. Test repeated hearing cycles and retained base coverage, not just one initial assignment. Correct the claimed coverage bound and “one goes/only those 32” descriptions. The proposed motion-onset observation may shorten hearing, but has not been shown to cure this feedback cycle.

### P2-1 — Ball timestamps are not unique cache generations

**Locations:** `sim/crates/bb_game/src/live.rs:1601`, `2843`; reachable production sequence at `3016` and `2803`, called in order at `3952` and `3953`.

`read_ball` records `ball_since`, which is just `self.t`. Two `set_free()` calls in one tick therefore have the same cache key. This is reachable: `update_thrown()` turns a grounded throw into a free ball and reads it, then `try_captures()` can rim that ball and call `set_free()` again before time advances.

**Production-path probe:** launch a downward throw, advance to its ground endpoint, call `update_thrown()`, then place the pitcher at that public ball position and choose a glove-error scale that puts the deterministic keyed placement on the rim. Call the real `try_captures()`. At `t=1.267851594`, the first free ball stops at approximately `(-0.6493,92.9603)`, and the rimmed ball stops at `(1.1849,30.9202)`. CF retains exactly his old intercept `(5.0183,72.2986)`. The assertion that the changed ball changes his plan fails. This uses physical glove placement, not an outcome roll.

A simpler two-replacement probe retains `(5.1401,78.4208)` for a replacement stopping at `y=35.3101`. The committed one-deflection test passes now and fails on `c0976f3`; it covers the first replacement, not this second replacement.

**Suggested fix:** invalidate saved reads on every `set_free()`, or use an incrementing ball generation distinct from time. Add the grounded-throw/rim sequence to regression coverage.

### P3-1 — A small allocation cleanup avoids the deferred cache's invalidation problem

**Location:** `sim/crates/bb_game/src/live.rs:1540`.

The duplicate partner scan is removed as claimed. Deferring a broader observer/teammate cache while correctness is open is reasonable. The roughly 29% slowdown also reproduces: the 1,000-game runs took 63.9 s on main and 82.1 s on final, under the same CPU restriction.

There is a cheaper option: `scan_from()` allocates a `Vec` for each path sample although it holds at most `SUB` (six) points. A scratch-only fixed six-element array preserved sample order/arithmetic and **all 50 full-game digests**. Alternating 300-game timings were baseline **24.9, 24.5 s**, array **24.5, 24.4 s**. That is too small a gain to distinguish confidently from heat noise; it does not resolve the 29% cost.

**Suggested fix:** consider the fixed local buffer as an optional allocation cleanup, with digest checks. Keep the broader cache deferred until profiling the corrected behavior; do not treat the older scan-cost percentage as a measured bound on this revision's optimization opportunities.

## Other review conclusions

- **The original two P1 probes are properly fixed.** Transplanted unchanged to `c0976f3`, the backup test fails with RF targets `(0,25.794706)` versus `(0,93)`, and the second-base test fails with SS Cover(2) versus Idle. Both pass now. Their expectation is information isolation with publications/public motion fixed, not a duplicate of the table implementation. The second test deliberately recalls only 2B; its premise explicitly checks Go versus CoverSecond. Opus P2-3's specific same-tick partner-choice leak is closed.
- **Corner sure-call ordering is sound.** The corner test fails on `c0976f3` with both still chasing, and passes now with LF continuing and RF backing up through further hearing cycles. `outranks`, heard-call selection and sure-call lead selection agree. Position numbers and the LF-over-RF convention are pre-pitch public knowledge, and §5.7 documents them.
- **The revised attribution reproduces.** The zero-delay workspace was extracted from the final revision; its only source difference is `CALL_HEARD_S: 0.3 → 0.0`. No earlier structural version was substituted. An additional final-source rebuild using the zero-delay build's same `--no-default-features` CLI configuration also reproduced 6.21 on 1,000 games, so that controlled comparison differs only in the delay constant. All three builds used identical seed commands:

  | Build | `--n 300 --seed 0` | `--n 300 --seed 1000` | `--n 1000 --seed 5` |
  |---|---:|---:|---:|
  | Main `5ba6750` | 5.86 | 6.18 | 6.03 |
  | Final | 6.17 | 6.22 | 6.21 |
  | Final, delay zero | 5.66 | 5.87 | 5.99 |

  Thus +0.22 and −0.04 are supported as the reported 1,000-game counterfactual split **for this implementation**, including its remaining defects. Zero delay still means next-tick publication, as the note says. The withdrawal of the specific relay attribution is appropriate; these runs do not identify a mechanism or estimate a universal real-defense cost. The early-deep-relay test passes, supporting that correction. The remaining false recovery/coverage statements are addressed in P1-2.
- **The motion-onset rule remains Proposed.** §5.7 explicitly marks it unbuilt, unsourced and awaiting the owner's ruling. `known()` still uses delayed `Shown` entries; no earlier chase knowledge is inferred from motion. Leaving the proposal unimplemented is correct.
- **Determinism and other invariants:** no new production hash iteration, direct transcendental call, non-keyed randomness or world outcome roll was found. The teammate scans use deterministic saved estimates; math/RNG boundaries are preserved. The crate-graph test passes. Exact present ball/body motion is the documented interim public-observation simplification; this does not close earlier, separate world capture/future-timing findings. The private-information invariant still fails as detailed in P1-1.

## Verification

All builds, tests and simulation invocations ran from `sim/`, prefixed with `taskset -c 0-7 nice`, with `PATH=$HOME/.cargo/bin:$HOME/.local/bin:$PATH`, `CARGO_BUILD_JOBS=2`, and both Zig caches under `sim/target/`. Heavy jobs were serialized. Revision copies, added probes, instruments, binaries and logs are under `sim/target/recheck/`; each scratch revision used its own target directory to avoid stale artifacts across revisions.

- `cargo test --workspace --release`: **326 passed, zero failed, three ignored**, including 146 `bb_game` library tests, crate-graph, recording isolation and cross-thread determinism checks.
- Scratch `cargo test --release -p bb_game --lib` with the four new committed regressions transplanted to `c0976f3`: both `unheard` tests, the same-tick deflection test and the corner-call test **fail as expected**. All four pass on final in the workspace run.
- Scratch final `cargo test --release -p bb_game --lib recheck_ -- --nocapture --test-threads=1`: **two diagnostic matrices passed, five assertions failed**, reproducing holder routing, relay scheduling, private-peel scheduling, and the two cache cases. Repeated with the allocation experiment removed from production code: same results. Details are in `target/recheck/final-restored-probes.log`. The matrices logged zero additional outsider variations; they are diagnostic coverage, not exhaustive proofs.
- Instrumented `recheck_games 50` on final and main, followed by `python3 target/recheck/analyze.py`: **3,736 / 3,706 live plays**, respectively, raw seeds 0–49; coverage and chase-interval results above. Original and instrumented final digests agree for seeds 1, 2, 3 and 42. Instrumentation changes no decisions.
- Fresh release CLI builds; `sim-cli games --n 300 --seed 0`, `--n 300 --seed 1000`, and `--n 1000 --seed 5`, separately on main, final and final-with-zero-delay: all nine results in the table reproduced, plus the identical-feature final 1,000-game repeat; no rules warnings. CLI seed arguments use the existing mapping `argument * 1_000_003 + i`; the 50-game interval probe uses literal seeds 0–49.
- Original release digest example (`target/release/examples/digest` after its release build), twice: seeds **1, 2, 3, 42 match**. Seed 42: `1ad343f315ae567fd582a908d86e2c0625e8d16534dedc6756f5e20094dbde5c`.
- Scratch fixed-buffer experiment: all **50 digests identical** to final, plus two alternating 300-game timing pairs, as reported under P3-1. The production scan was restored in the scratch tree before the final defect-probe rerun.

No tracked source, test, spec, ledger or TASKS file was edited. No commit was made. This report is the only tracked deliverable.

## Resolution

Open.
