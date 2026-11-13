# Codex review: dive recovery and two-out throws

Reviewed `2355c34..f4d3167` (`7a3c5b0`, `f4d3167`), after reading `CLAUDE.md`. Locations below refer to **`f4d3167`**, rather than concurrent, uncommitted work in this checkout. No source, existing test, spec, or TASKS file was edited. This review is the only authored repository file; build products and diagnostic logs are under `sim/target/`.

## Findings

No P1 finding. Two P2 defects and one P3 specification clarification remain.

### 1. P2 — A retreat to an already-reached base receives the force bonus

**Location:** `sim/crates/bb_game/src/live.rs:2393` (`forced` at line 2081; `visible_target` at line 414).

`forced(k)` describes the surviving chain of runners from their starting bases. It does not establish that the runner can currently be forced out at his visible destination. In particular, a runner who started on first, reached second, and is now returning toward second from the second-to-third segment satisfies both `forced(k)` and `goal == r.start + 1`. The new branch awards weight **1.0**, although retiring him there requires a tag and deserves **0.9** under the stated policy. The force would become actionable again if he retreated below second; approaching second from above has not done that.

**Verification:** An exact-revision probe established a fair ground ball, delivered the runner's touch and departure from second to the rules engine, and placed him 4 m beyond second with goal 2 and speed 7 m/s. The holder's predicate was true. Rules state had `forced_to = Some(2)` but `at = 2`; a subsequent defensive touch of second produced **no out**. Thus checking the retained force destination alone would not fix this either. Normal movement can create a retreat when a preceding runner blocks the next base; a voluntary rounding decision is not required.

**Impact / suggested fix:** This newly promotes a tag play over legitimate alternatives and incorrectly assigns it the third-out force's run-suppression advantage. Require the visibly approaching runner to be short of the force destination as well as satisfying the force-chain and destination checks. Preserve the existing reinstated-force behavior for a runner who has actually returned below that base. Use visible progress/touch history, without giving decision code the rules engine's private live state.

### 2. P2 — Catch deferral ignores the new height limit for a downed fielder

**Location:** `sim/crates/bb_game/src/live.rs:1797`, interacting with line 1815.

The current-tick eligibility test correctly limits a downed fielder to `DIVE_MAX_Z`, but the next-tick test still uses only `zmax_at(bn)`, the standing/leaping height. A ball within reach and below 1.2 m can therefore be passed up because its next position is horizontally closer, even though that next position is too high for this fielder. The next tick rejects the catch, and the ball can escape without any glove attempt.

**Verification:** With perfect hands, the stationary CF down until 2.5 s, and a free ball at time 1.5 s at `(-0.3, 80, 1.19)` moving `(6, 0, 2)` m/s, the next tick's height is **1.2219623 m**. The code defers the currently legal catch. Over the next 20 ticks it makes **zero attempts and zero captures**, although the first position is only 0.3 m away and meets the new height limit. No ground/wall/end boundary prevents the deferral.

**Suggested fix:** Apply the fielder's applicable height limit to the prospective capture too, accounting for whether he will still be down then. Add a rising-ball boundary case alongside the two rolling-ball regressions. The existing tests do not exercise the added height restriction.

### 3. P3 — Qualify the new recovery description's receiver and retry exceptions

**Locations:** `docs/agent-models.md:380`; `sim/crates/bb_game/src/live.rs:1867`, lines 1881–1890.

The new prose says that whichever attempt secures the ball, throwing waits for the original get-up plus transfer. That is true for the intended free-ball re-fielding path, but the receiver branch immediately overwrites the newly calculated deadline with `t + 0.7 * transfer`. A fielder still down from a dive who receives a low throw can consequently recover before his original get-up ends.

**Verification:** A downed CF receiving a relay throw at **1.5 s**, with his original recovery ending at **2.5 s**, secures the low ball and gets `ready_at = 2.095 s`. Calling normal movement at that deadline clears `dove`, 0.405 s before the original recovery ends. The ordinary base receiver uses the same override. This override **predates the reviewed range**; the new formula does not resolve that interaction. Thrown balls also use `RECEIVE_REACH = 1.5 m`, rather than the free ball's 1 m reach.

The availability of a low grab is additionally subject to `blocked_until`: a rim deflection permits another attempt after 0.5 s, whereas a complete missed dive blocks attempts until the get-up ends. Those existing restrictions are absent from the general wording.

**Suggested clarification:** Scope the implemented guarantee to an eligible free-ball re-fielding after the retry block. Either document the existing receiver exception explicitly or preserve the original get-up before applying the shorter receiver transfer. A separate recovery deadline would make this distinction clearer if that behavior is changed later.

## State lifecycle and integration

Apart from finding 3, no stale-flag failure was established in the requested catch-and-throw lifecycle. Every new play initializes `dove = false` and `ready_at = 0`. Only committing a dive sets the flag; movement clears it when the deadline passes. A standing fielder's future transfer deadline alone cannot make him `down`. A probe with `dove = false` and an old deadline of 9 s still caught a 1.8 m-high ball at 2 s and correctly replaced that deadline with 2.85 s.

After a free-ball diving catch, `ready_at` represents **recovery plus transfer**, and `dove` remains true until that combined deadline. `fielder_step` therefore remains frozen through transfer, as it already did for an immediate diving catch before this change. `try_captures` returns while the ball is held, and `holder_act` cannot throw before the deadline. On the release tick, captures have already run; movement then clears the flag. The short interval in which the flag remains true after the deadline cannot classify the fielder as down because `down` also checks time. The new `max(t)` handles an expired flag before movement clears it.

The rim branch preserves the dive deadline and returns after setting its 0.5 s retry block, before the complete-miss branch can replace that block with the full recovery deadline. `set_free` and reassignment do not reset either recovery field. A permitted low re-field adds transfer to the original recovery deadline, without charging another dive. A complete miss continues to block all attempts until recovery. The holder's close tags and base touches still precede its transfer gate, while running to a bag remains subject to the movement freeze. `settled()` still requires the holder's deadline to have passed; no premature free-ball settlement attributable to the new calculation was found.

For the force-chain questions, retired/scored candidate runners and protected overrunners are excluded from throw options. An out of the batter or a trailing runner is absorbed synchronously and removes the chain used by `forced(k)` for runners ahead. The batter reaching/overrunning first does not by itself eliminate a preceding runner's obligation to advance. A retreat toward a start base gets the tag weight because its destination differs from `start + 1`; finding 1 concerns a retreat toward the already-reached advance base instead.

`self.st.outs` includes outs made earlier in this play, including contacts processed earlier in the same `holder_act` call. `emit` steps the rules engine synchronously; retirement increments `st.outs`, and `absorb` marks the runner out before returning. A fair-ball probe starting with one out made a force at second and observed **two outs immediately**. The production loop stops on an inning-ending out before making another throw decision.

## Two-out policy and captions

**A sure tag can lose to a 0.95 force, as the question suggests.** A certain tag scores `1² * 0.9 = 0.9`; the force scores `0.95² = 0.9025`. The force needs only `p_force > sqrt(0.9) * p_tag`, with `sqrt(0.9) = 0.9486832981`. Squaring does not make the policy always select the highest estimated out probability.

A physical holder fixture with the pitcher at `(0.9, 18.2)`, receivers on first/home, two outs, an advancing batter and a runner from third produced believed probabilities **0.9500000000000003 at first** and **0.9999999994677791 at home**. `2355c34` threw home; `f4d3167` threw to first. Thus this can choose a less certain out than before, even in a constructed case where neither third-out choice lets a preceding run score. This is a consequence of the explicitly specified 10% score preference, rather than a formula/implementation discrepancy. The prose's “sure out” language should be read with that numerical tradeoff, not as a guarantee. These are agent beliefs, not world outcome rolls.

The caption change correctly distinguishes the specified lone third-out tag from a force when the batter has not reached first. Both added cases pass. Other out kinds do not establish an additional regression in that branch: a caught fly retires the batter and follows the earlier batter-out branch; normal retouch appeals also follow that catch. For multiple outs, the force predicate means “some other runner was forced out,” not necessarily “the final out was a force”; the double/triple-play prefix still describes the whole play. No additional caption finding was established.

The additions to agent-models accurately reproduce the score formula, base weights, cutoff, and historical seed examples, subject to findings 1–3 and the probability tradeoff above.

## Scoring attribution

I rebuilt the three exact Git revisions and ran the unchanged CLI `games` implementation for **200 games each with seed arguments 1 (default), 1000, and 2000**. Its actual game seeds are `argument * 1_000_003 + i`. A second diagnostic summed integer totals to avoid averaging rounded CLI numbers and confirmed these results:

| Revision | Total runs / 1,200 team-games | Runs/team-game | GB hits / GB | Pooled GB hit rate |
| --- | ---: | ---: | ---: | ---: |
| `2355c34` | 6,574 | 5.478333 | 7,326 / 17,745 | .412849 |
| `7a3c5b0` — dive only | 7,007 | 5.839167 | 7,895 / 18,081 | .436646 |
| `f4d3167` — both changes | 7,019 | 5.849167 | 7,898 / 18,117 | .435944 |

The dive commit accounts for **433 of the net 445 extra runs (97.3%)** in this matched cohort. The attribution to the dive fix is supported. The exact published endpoint numbers do not reproduce: these runs round to **5.48 → 5.85** and pooled GB hits **.413 → .436**, rather than 5.86 and .440. The final default-seed batch alone has GB hit rate .439787, which rounds to .440; that does not establish how the commit message obtained its figure. These cohort results do not imply a population effect of exactly this size.

The mechanism includes losing the illegal second-dive capture, rather than simply releasing every re-fielded ball sooner. On seed **7475/play 25**, the baseline CF secures at **3.978028 s** and releases at **6.344694 s** (2.366667 s holding). At both later commits he secures at **6.161361 s** and releases at **7.028028 s** (0.866667 s holding). His holding time shrinks, but his actual release is **0.683333 s later**. On play 26, the baseline and dive-only versions release toward home at **3.253520 s** and subsequently tag the runner; the final version releases at that same time toward first and makes the base-touch play there. Both historical examples therefore reproduce in the relevant revisions.

## Invariants

The diff introduces no outcome roll, RNG draw/purpose change, native transcendental call, unordered iteration, or fused arithmetic. Capture remains a physical reach/glove-placement test. The new probability weight affects the holder's belief-based choice, not world adjudication; its inputs add no hidden trajectory or executed-noise access. The dependency check passed. No ability/noise calibration or attempt to fit scoring through a new global outcome parameter was made; the rate movement follows a physical correction and an explicitly documented decision preference. Statistics still derive from events.

For each of the three revisions, complete games at seeds **0, 11, and 7475** produced identical event-log bytes and digests with recording off versus on. No determinism regression was established.

## Verification

Commands ran with Cargo targets and Zig caches under this checkout's `sim/target/`. The initial workspace test invocation failed because Zig's default cache was read-only. Re-running from `sim/` with `ZIG_GLOBAL_CACHE_DIR="$PWD/target/zig-cache" ZIG_LOCAL_CACHE_DIR="$PWD/target/zig-local-cache" cargo test --workspace --release` passed.

Concurrent edits appeared during the review. All subsequent focused tests, comparisons, and traces used module contents read with `git show` from the named revisions, assembled in memory and passed to `rustc` on stdin. No scratch source/test files were written. Those builds used existing release dependencies, Rust edition 2024, optimization level 3, the configured Zig linker, and `CARGO_PKG_VERSION=0.1.0`. Artifacts are prefixed `dive-review-` or `libdive_review_` under `sim/target/`.

| Check | Result / log |
| --- | --- |
| `cargo test --workspace --release`, writable caches | **155 test functions passed**, zero failures, including scenario corpus, dependency, determinism, accounting, and FFI tests. `target/dive-review-workspace-tests.log`. |
| Exact `f4d3167` game tests plus seven diagnostic tests, `target/dive-review-probes --nocapture` | **63 passed**, zero failures. Probes cover rising-ball deferral, downed receiver, retreat classification, score arithmetic, actual .95-force choice, synchronous outs, and standing/recovery state. `target/dive-review-targeted.log`. |
| Same probability-choice fixture at `2355c34` | Passed: throws home instead of first. `target/dive-review-baseline-choice.log`. |
| Exact-revision CLI `games --n 200 --seed {1,1000,2000}`, each of three revisions | All nine batches completed, warnings `{}`. `target/dive-review-<revision>-seed<argument>.log`. |
| Integer-total diagnostic, same cohorts | All **1,800 games complete**, zero rules warnings; totals above. Recording comparisons also passed. `target/dive-review-exact-<revision>.log`. |
| Exact-revision `trace_play.rs`, seed 7475, plays 25/26 | Six traces reproduced the capture/release and destination differences above. `target/dive-review-trace-<revision>-play<play>.log`. |

During harness setup, direct `rustc` needed explicit Cargo version/linker settings and a consistent dependency-artifact set. Two initial fixture assertions also required correction: a bounce short of first had not yet established fair territory, and runner ETA includes acceleration rather than constant speed. The final fair-ball fixtures and inverted ETA produced the passing results above. These setup failures were not product test failures.

## Resolution

All three findings are fixed in the commit that adds the dive animation (2026-11-13).

1. **P2, retreat given the force weight:** fixed. The weight is now `Sim::out_weight`. With two out, a runner counts as forced only while he is still short of the force base (`r.s < goal · BASE_L`), which is his visible position. A runner coming back to a base he has already reached gets the tag weight. Test: `a_runner_back_to_a_base_he_reached_is_a_tag_play`.
2. **P2, catch deferred past a downed fielder's height:** fixed. The next-tick deferral uses the height limit for a fielder who will still be down then (`DIVE_MAX_Z`). Test: `a_fielder_down_from_a_dive_takes_a_rising_ball_while_it_is_low` (Codex's probe: it fails without the fix).
3. **P3, receiver exception:** fixed in code rather than documented. A fielder down from a dive who takes a thrown ball now starts the receiver's shorter transfer when he is up (`up_at + RELAY_TRANSFER_FRAC · transfer`). The FFI's new dive check also found this: a downed shortstop moved before his get-up ended. The spec (agent-models §5.3) now states that rule and the retry block: 0.5 s after a rimmed ball, until he is up after a clean miss.

**Probability trade-off (not a finding):** yes, a sure tag (0.9) can lose to a 0.95 force (0.9025). That is the specified preference. The spec says "an equally sure tag loses to the force; a sure tag still beats a doubtful force", and 0.95 is not doubtful at that weight.

**Commit message figures:** the `f4d3167` message gave 5.48 → 5.86 runs and GB hits .413 → .440. Codex's integer totals over the same cohorts give 5.48 → 5.85 and .413 → .436. The message averaged the three rounded batch figures and quoted one batch's GB rate. The attribution to the dive fix (97%) stands.

