# Independent test-gaps and sources review

Reviewed `a69dd50..7104cd5` (`work/test-gaps`, merged at `3697520`) and `docs/calibration/2026-11-14-sources.md`, including `e6d9c3b` and `0900352`. Read `CLAUDE.md` first and checked agent-models §5.3–§5.5 and data-formats §2.4. Medium depth. No P1 findings; one P2 and two P3 findings below. No production behavior change was found. Locations in findings refer to **7104cd5**, not the later main-branch line numbers.

## Findings

### P2 — The new fly test misses the positive airborne comparison

**Location:** `sim/crates/bb_game/src/live.rs:5462–5469`, `going_for_a_fly_gains_only_when_he_cannot_catch_it_on_his_feet`; implementation `Sim::dive_gains`.

The near case exercises a standing catch before the bounce. The far case does **not** exercise a standing intercept after the bounce: `intercept_scan(...).0` is `None`, so `dive_gains` returns `true` before testing `ball_still_up` or comparing the intercept with the first bounce. Thus a regression that disables going for every airborne ball with a reachable post-bounce standing play escapes this test.

**Verified mutation:** replace `p.first_bounce().is_some_and(|b| ts >= b.0)` with `false`. The new fly test still passes. Scratch probes of the unmodified implementation report:

| Initial distance | Standing intercept (absolute s) | Believed first bounce (absolute s) | Gain |
| --- | ---: | ---: | --- |
| 3 m, existing near case | 4.010 | 4.218525 | false |
| 15 m, additional probe | 4.730 | 4.237014 | true |
| 40 m, existing far case | None | 4.274711 | true |

Keep the existing no-intercept case, but add a case such as 15 m that asserts a standing intercept exists **after** the believed bounce, then requires gain for both costs. This closes the airborne branch gap named in the dive-rate review's Resolution; the current test closes only part of it.

### P3 — The change-of-mind test accepts a missing stretch recovery cost

**Location:** `sim/crates/bb_game/src/live.rs:5408–5410`, `a_stretch_is_dropped_when_his_read_changes_during_the_wait`.

`ready(&kept).abs() < 1e-9 + 0.3` checks proximity to **zero**, with a 0.3 s tolerance. It accepts a control that owes no stretch recovery, despite the message saying “control owes a stretch.” The independent expectation is `abs(ready(&kept) - 0.3) < tolerance`.

**Verified mutation:** replace the capture branch's `self.t + STRETCH_COST_S` with `self.t`. This new test still passes. The unmodified fixture secures at 1.650 s with 0.300 s residual recovery; its changed-read case secures at 1.700 s with 1.500 s recovery and one dive. The action comparison is useful: removing the stretch gain gate makes the test fail. This is a weak assertion in the new test, **not** a total suite gap: the older `a_ball_just_past_reach_is_taken_with_a_stretch_not_a_dive` already asserts the exact 0.3 s cost.

### P3 — “Believing the truth” is actually a noisy read; the roller bracket does not bracket that read

**Location:** `sim/crates/bb_game/src/live.rs:5348–5391` (`crossing_with_mind_change`), `5440–5453` (`believing_the_truth`), `5477–5493` (roller test).

Both helpers initialize belief with `sim.predict(CF)`. `new_sim` supplies keyed perception errors, and `predict` applies position/velocity noise, estimated spin and smooth-ground prediction. Perfect hands do not remove perception noise. Consequently the descriptions “His belief is the true path” and “believed to be the world's own” are inaccurate.

The roller test brackets the **world** path's wait, while `dive_gains` judges the **belief** path. That is a useful geometric sanity check, but it does not establish the asserted inequalities for the input actually used by the decision. This is not a tautology, and reversing the ground comparison makes the test fail; it is a weaker independence argument than the comment claims.

Use an explicitly supplied, controlled belief for these fixtures, or document the noisy read and bracket its path independently too. Keep the expected decisions as literals. The fly probe's different believed bounce times for the same initial vertical motion at different distances illustrate why the distinction matters.

## Audit of every new test

The range adds six `live.rs` tests, two `throwing.rs` tests, and one rosterless integration test. Helpers are not counted as tests. Mutations below were made only in `/tmp` archive copies and were restored; none was committed.

| New test | Recalculation and independence assessment | Bug sensitivity |
| --- | --- | --- |
| `a_left_handed_fielder_releases_from_his_left_and_a_right_handed_one_from_his_right` | Independently, `d = 27.432 / sqrt(2) = 19.39735322`; target vector `(27.39735322, -18.60264678)` has length `33.11606001`, unit `(0.82731319, -0.56174094)`. Right vector is `(-0.56174094, -0.82731319)`. Adding/subtracting 0.35 m gives R `(-8.19660933, 37.71044038, 1.8)`, L `(-7.80339067, 38.28955962, 1.8)`. Both literals are within `2e-5`. The comment's `d=19.3978` and length `33.115` are slightly inaccurate intermediate approximations, without invalidating the assertions. It inspects an actual live throw at release, not just the helper. | Changing production arm offset from 0.35 to 0.30 m fails this test. Swapping hands or ignoring the left hand would also violate the independent coordinates/mirror assertion. |
| `a_stretch_is_dropped_when_his_read_changes_during_the_wait` | Nominal closest distance 1.3 m is outside 1.0 m reach and inside 1.4 m stretch. Balance costs are independently 0.3 s versus 1.5 s. The unchanged-read control and changed-read action differ, with the observed times/recovery above. Neither result is generated by calling the asserted decision function to calculate its expected answer. The fixture includes running, slowing/bouncing ball physics and noisy rereads, so straight-line arithmetic alone cannot predict its exact tick. | Removing the stretch gain gate fails it. Removing stretch recovery passes it (finding P3). It demonstrates delayed acquisition, not permanent loss of the ball. |
| `a_dive_read_that_changes_resets_the_lunge` | Nominal extension is `2.2 - 1.0 = 1.2 m`; `1.2 / 1.5 * 0.3 = 0.24 s` to lay out. An initial 0.4 s lead allows it; a flip around 0.2 s leaves roughly 0.2 s, insufficient after restart. These are approximate fixture-design calculations: the ball slows and the body moves. The control requires an actual dive, preventing an always-no-dive implementation from passing. | Keeping the old timer when a read says no (`else { fd.must_dive_since }` instead of infinity) fails the flipped case. This directly checks the intended reset regression. It does not assert the exact 0.1 s cadence or 15 m distance boundary. |
| `a_missed_stretch_leaves_the_ball_loose_and_the_fielder_on_his_feet` | Nominal 1.3 m selects stretch geometry. A one-metre base hands sigma does **not** mathematically guarantee a miss on every draw: a Gaussian can lie arbitrarily near zero. For this fixed keyed fixture, the actual attempt is a stretch at 1.650 s, horizontal distance 1.212065 m; effective sigma 2.478481 m, standardized draws `(-0.35202021, -0.04246928)`, giving radial error `2.478481 * hypot(...) = 0.878802 m`, well outside the independently specified 0.12 m rim. Thus this particular miss is real, and `attempts >= 1` prevents a no-attempt pass. | Making a clean miss set `Ball::Held` fails it. Forcing all attempts to miss passes, as expected for a miss-only test; the existing successful-stretch test covers the other side. “Every time” should mean this fixed fixture, not every seed. |
| `going_for_a_fly_gains_only_when_he_cannot_catch_it_on_his_feet` | Vacuum flight to ground solves `2 + 10t - 4.905t² = 0`, giving 2.22222 s. Even at instantaneous 8.5 m/s, 40 m takes 4.706 s, whereas 3 m is readily reachable before landing. Those are independent, generous action expectations, but the actual belief is noisy and the far case exits before the airborne comparison. | The near case would reject an always-true result. Forcing only the airborne comparison false survives: P2 above. A reachable post-bounce case is still needed. |
| `going_for_a_roller_gains_only_when_the_cost_is_less_than_the_wait` | The world-path probe at t=2 gives distance to planning reach `d=5.36147457 m`, ball speed `5.84957085 m/s`, so the optimistic running bound is `5.36147457 / (5.84957085 + 8.5) = 0.37363310 s`. A stationary fielder's sampled upper bound is 1.335 s. Thus `0.3 < wait < 1.5` for that path, independently of `intercept_scan`. These wide bounds are useful; they reuse ball physics, not the decision being asserted. They do not establish the same bracket for the noisy belief (P3 above). | Reversing production `ts - self.t > cost_s` to `< cost_s` fails it. Always-true or always-false gain also cannot satisfy both assertions. |
| `set_up_adds_transfer_turn_and_footwork` | From §5.5: A set `0.65+0.12+0.2=0.97`, earliest `0.6*(0.65+0.12)=0.462`, hurry at 0.6 `(0.97-0.6)/(0.97-0.462)=0.72834646`. B set `0.65-0.12+0.3=0.83`, earliest `0.6*0.53=0.318`, midpoint `0.574`, hurry 0.5, speed `1-0.5*(0.10+0.08)=0.91`, sigma `1+0.5*(1+1)=2`. Relay input `0.7*0.65=0.455`, earliest `0.273`. All arithmetic agrees; these are independent spec equations, not values sampled from Rust output. | Changing `RUSH_MIN_FRAC` to 0.5 fails it. Dropping turn/footwork or including footwork in earliest time would also fail. Scope limitation: the test **supplies** ±0.12 turn and the 0.7 relay factor; it does not prove live code selects them. Existing turn and relay/lifecycle tests remain necessary. |
| `moving_away_is_owed_only_above_two_metres_a_second` | Projection of `(0,-2,0)` away from target `(0,10,0)` is exactly 2 m/s, so no cost; `(0,-2.01,0)` projects to 2.01 and owes 0.2 s. Independent threshold boundary, with literal answers. Its comment mentions incoming-ball thresholds, but it tests catch velocity only. | Changing `>` to `>=` at `AWAY_MPS` fails it. It does not test the separate 3 m/s incoming-ball facing cutoff. |
| `a_log_without_rosters_still_reads_and_counts` | The hand-built header omits config in one iteration and omits roster fields on both named teams in the other. Two PA-ended events give PA=2, K=1, HR=1, ordinary batted=0 (HR has its own category); one `RunScored` gives runs=1 and top-first linescore `[1]`; `GameEnded` supplies 1–0, and half-start supplies inning=1. Empty batting lists follow missing roster fields, independently of the sim. No stat expectation comes from running a game. The tiny stream is a reader fixture, not a legally complete one-inning baseball game. | Requiring the optional team roster with `unwrap()` fails the teams-present case. Ignoring all events cannot pass the literal counts. It uses current schema builders with fields omitted; it is useful optional-field compatibility coverage, not a frozen historical binary/schema-version corpus. |

## Gaps from the November 13 Resolution sections

The new tests close rosterless reading, a left-handed live release, a deterministic stretch miss, and the two change-of-mind examples. They add combined setup arithmetic and a ground-cost distinction. They do not close every earlier omission:

- **Dive-rate review:** the positive airborne comparison remains uncovered as described in P2. The stretch `low_catch` boundary remains untested: neither the new tests nor another assertion in `live.rs` pins an upright stretch above the 0.4 m low-catch threshold versus one below it. Charted-play validation of stretch frequency, secure rate and throw delay remains deferred. The diagnostic's air/ground classification after a deflection is also unchanged and untested by this range.
- **Hurried-throws review:** the live race fixtures still build runner timing with production `throw_time`; an independent throw-flight/time oracle remains absent. The new unit setup cases do not replace that. The previously fixed dive acquisition/recovery and carry lifecycle tests remain present.
- **Handedness review:** a left-handed throw now passes through sim release geometry, but the explicitly unperformed **Unreal fielder-motion screenshot check** remains unperformed; this Rust test cannot validate the client animation.
- **Rosters review:** still no committed independent golden seed specifically exercising rejected **family name and uniform number** draws (seed 7 exercises rejected given names; broad legal-roster tests and the earlier independent probe cover more seeds). The new rosterless case is now committed, but a frozen old-schema corpus and the noted client rebuild are not supplied here.
- **Popup-calls review:** no added regression fixtures for both middle infielders leaving second uncovered or a re-call dropping an active chaser outside the candidate set. These remain owner-deferred behavior issues, not changes requested by this review. The old idle-fielder review's unconditional-priority issue was subsequently addressed by the existing sure-caller/pop-up tests; it should not be listed as entirely untouched. The popup review's marginal gap-fly fixture was subsequently widened to 0.4 s in the plate-cue resolution, so the older 30 ms warning is stale.
- **Rules-passing review:** no new coverage/decision for strict I1/I10 accounting of runners between bases or touching home after the third out, or for scoring a batter retired on a held pass before a home run. Existing passing scenarios exercise related events, but these unresolved accounting/scoring expectations remain unpinned.

## Sources note: five linked-number checks and definitions

Spot-checked five numbers against five linked publications. All five match:

| Claim in sources note | Linked evidence checked | Result |
| --- | --- | --- |
| 2026 RF arm mean 90.7 mph | [Savant arm-strength leaderboard](https://baseballsavant.mlb.com/leaderboard/arm-strength), 2026 league positional row | Matches. Savant uses the top 10% of OF throws; IF and 1B use different top fractions. This is not an all-throw average. |
| LHP/LHB K 24.0% | [MLB Data Warehouse, Examining Pitcher Splits](https://www.mlbdatawarehouse.com/p/examining-pitcher-splits), opening four matchups | Matches, and the note correctly avoids assigning an unverified sample year. |
| IF glove-to-release 0.640 ± 0.068 s | [Kita et al. PDF](https://www.jstage.jst.go.jp/article/ijshs/advpub/0/advpub_201316/_pdf/-char/ja), Table 1 and Methods | Matches; Methods select each player's shortest of five trials. The four phase means sum to 0.640 s. |
| Selected 2016 2B exchange mean 1.2 s | [MLB's 2016 Statcast team](https://www.mlb.com/news/verlander-and-buxton-lead-2016-statcast-team-c209343802), Pedroia section | Matches. Selection requires batter out and a 2B assist; not all grounders or all pivots. |
| 2B outs on dives 24% | [SIS, The All-Scrapes Team](https://www.sportsinfosolutions.com/2025/04/16/stat-of-the-week-the-all-scrapes-team/), second-base section | Matches the positional MLB average; not Giménez's individual 25%. |

I found no additional material, unqualified definition-mixing error in the final note. It explicitly warns against the main misleading comparisons: outs after infield dives versus balls secured; fly-ball averages including HR versus BABIP; positional max-effort arm statistics versus uniform sim ratings; player shares versus innings/PA/game shares; total target error versus directional sigma; and deliberate velocity changes versus rushed preparation. Those warnings should travel with any extracted table or headline.

The ranked ground-ball discrepancy remains a **qualified investigation signal**, not a measured .106 fielding deficit: pitcher-averaged batting averages and the sim's per-contact hit rates differ in weighting, scoring and classification. Likewise the 1.2 s exchange minus 0.65/0.455 s calculations compare a selected game-play mixture with base setup inputs, not matched full simulated receipt-to-release intervals. The note says so and forbids directly replacing parameters; stripping those qualifications would be misleading. The 36 SS dives/team-season conversion is arithmetically `2*36/162 = 0.4444` per two-team game, but its original SIS link did not open in this review; it was not counted among the five verified numbers.

## Verification

- In the working repository: `export PATH=$HOME/.cargo/bin:$PATH; export CARGO_BUILD_JOBS=2; cd sim && cargo test --workspace --release` completed successfully: **205 tests passed**, zero failed, ignored or filtered; doc tests passed too. This is the later main checkout, including the subsequent throw-choice diagnostic test.
- Archived both requested revisions with `git archive` under `/tmp/baseball-review-{before,after}`. A scratch `bb_game` example called `simulate_game(seed)` directly and printed its 32-byte digest. The final after run used the exact restored `7104cd5` source, without mutations/instrumentation. Both builds used the same toolchain/release configuration.

| Seed | `a69dd50` digest | `7104cd5` digest |
| --- | --- | --- |
| 1 | `6aadb72523b773e702738b7db051e23fcc9f17ae9e21af5719508d4f83f1f915` | identical |
| 2 | `bc279a49173fdf662b569522b8c65b04b5daf2493124258ab342fd3e2387a38a` | identical |
| 3 | `f876ea3a8cac41bf289c9184578cdc62e776be2c602428b807569d14d5002afe` | identical |
| 42 | `358da6cd0425ae14ca7a2bc457a11725f16f840830270fd1de2aa9bf5d612ff7` | identical |

- Diff inspection confirms all executable additions in the requested range are test-only. The remaining changes are the `flatbuffers` dev-dependency and its lockfile edge. The sources commits change only their document. Identical digests for these four seeds corroborate unchanged behavior; they are not exhaustive equivalence proof, and the digest excludes the header.
- The full workspace release suite also passed on the exact restored **`7104cd5`: 204 tests, zero failures**, plus passing doc tests (`/tmp/baseball-review-after-tests.log`). Scratch live tests separately passed **75/75** with diagnostic printing only. Targeted mutations and their results are recorded in the audit above; logs/scripts are `/tmp/baseball-mutation-*.log`, `/tmp/baseball-mutations.py`, `/tmp/baseball-other-mutations.py`, and `/tmp/baseball-final-probes.py`. The supplementary air probe is `/tmp/baseball-air-scans.log`. An attempted relay-factor mutation was skipped because its search target did not match; no mutation result is claimed for it.
- Fresh linking initially failed because Zig's default cache is read-only. Retried using `ZIG_GLOBAL_CACHE_DIR=/tmp/baseball-review-zig`, without administrator access. Cargo reused the workspace target cache; no source, tests, specs, TASKS or Git metadata in the working repository were edited.
- Read all November 13 review Resolution sections, inspected the affected implementations and existing tests, and fetched the five linked sources above. Only this review document was written in the working tree.

## Resolution

Resolved 2026-11-14 (lead). All three findings fixed in the tests; no sim code changed.

- **P2, fixed.** `going_for_a_fly_gains_only_when_he_cannot_catch_it_on_his_feet` has a third case at 15 m: the fixture asserts a standing intercept exists, and going for it must gain for both costs. The review's mutation (the airborne comparison forced to `false`) now fails this case by construction, since the result there comes only from that comparison.
- **P3, fixed.** The control in `a_stretch_is_dropped_when_his_read_changes_during_the_wait` must owe 0.3 s (`|ready − 0.3| < 1e-9`), not anything within 0.3 s of zero.
- **P3, fixed.** The helpers' comments say the belief is the fielder's own noisy read (`Sim::predict`), and the roller test brackets the wait on his read as well as on the world's path.

Deferred (still untested; listed in TASKS.md): the stretch's low-catch boundary at 0.4 m; an independent throw-flight time oracle for the live race fixtures; a golden roster seed that exercises rejected family names and uniform numbers; fixtures for the two owner-deferred pop-up cases; the rules-passing accounting cases (runners between bases after the third out, a batter retired on a held pass before a home run); a left-handed fielder's throw looked at in the client.
