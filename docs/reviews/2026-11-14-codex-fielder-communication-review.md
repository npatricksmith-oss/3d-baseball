# Codex review: fielder communication, 2026-11-14

Reviewed `c3a2714..1d1ec8b` (`8861a38`, `636366d`, `1d1ec8b`) on `work/live-comm`. Read `CLAUDE.md` first. Reviewed the complete `live.rs` decision paths, agent-models §5.7, the calibration note, metrics-ledger row, previous P1-1 Resolution, and the additional FFI test change.

**P1-1 is only partially fixed.** The old private-target arbitration is gone and the call delay works, but same-tick choices still reach teammates through coverage and backup assignment. Replay passes. There are also new cache and equal-priority coordination defects. The measurements recorded in the ledger do not establish its categorical attribution of the final run increase.

## Findings

### P1-1 — Coverage and backups still consume unpublished teammate choices

**Locations:** `sim/crates/bb_game/src/live.rs:1601,1608`, `2279–2291`, `2189–2197`; related current-role consumers at `1992`, `2259–2262`, `2301–2323`, `3525–3536`. The blanket closure claim is at `docs/reviews/2026-11-14-codex-live-play-review.md:228`.

`assign_among()` collects the candidates' private `Decision::CoverSecond` results into `second` and immediately supplies that vector to the shared coverage table. `cover_table()` reserves those men and overrides the normal second-base assignment. No publication or hearing occurs. These decisions depend on their owners' private reads/intercept times; they are not reconstructible from the public tables.

Separately, `assign_backups()` builds its available-player pool from **current** `role == Idle`, after the current decisions have been applied. Thus another man's newly chosen, unheard chase immediately changes the backup's choice. Protecting a man's own chase from an assigned table job is legitimate; allowing that private choice to alter another man's assignment is the remaining leak.

**Isolation probes, with public motion and publications held fixed:**

- Known CF chase, CF at `(0,85)`, LF at `(0,80)`, RF at `(10,80)`. Change only LF's current private role from Idle to Chase. `known(LF)` remains `(false,0,false)` in both cases, but RF's backup target changes immediately from `(0,25.794706)` to `(0,93)`: behind second versus behind the fielder.
- Grounder starting at `(0,0.5,0.5)` with velocity `(1.5,24,-3)`, SS at `(1,25)`. Preserve the second baseman's saved read and intercept point, varying only his private `t_int` between `0.8` and `20`. His decision changes Go → CoverSecond. With `known(2B) == (false,0,false)` in both cases, SS changes Cover(2) → Idle in the same call. This is a constructed information-isolation case, not a sampled game's frequency estimate.

The two new committed tests pass despite both leaks. The other current-role consumers listed above also mean the claim that all roles are public cannot be assumed: their derivation includes these private choices. The note's explicitly accepted immediate visibility of a **stopped** chase does not authorize immediate knowledge of a newly started one.

**Suggested fix:** derive each observer's table/availability view from published, delayed knowledge and public table inputs, with an override only for his own decision. A private CoverSecond choice needs communication or an independently computed expectation by the other observer. Apply an assigned backup job only if its recipient's own decision permits it; do not silently reallocate another backup using an unheard chase. Add isolation checks for decisions, cover reservations and backup targets, then revise the previous Resolution to reflect the actual boundary.

### P2-1 — A new loose ball can reuse the previous ball's same-tick read

**Locations:** `sim/crates/bb_game/src/live.rs:1573–1576`, `2702–2720`; production deflection path at `2670–2678` and close read at `2411–2416`.

The new read cache is keyed only by time: `read_at == self.t && belief.is_some()`. `set_free()` changes the ball/path and `ball_since`, then calls `assign()` without invalidating that cache. A chaser who just read the approaching ball can therefore reuse that prediction when a rim deflection changes its velocity in the same tick. His new call and destination describe the old flight. This can last until the next replan (normally 0.4 s); the call tier is only recomputed on an assignment.

**Probe:** make the initial read at contact + 0.35 s, then replace the ball at its **same current position**, retaining 30% of horizontal velocity and setting vertical velocity to 1 m/s, as in a slowed glove deflection. CF's pre-deflection intercept is `(-0.092649,99.365637)`. `set_free()` leaves that exact intercept cached; an uncached prediction of the new ball gives `(0,37.423046)`, about 62 m away. The assertion that it is a “new ball, a new call” is not implemented for these readers.

**Suggested fix:** include ball generation in the read cache key, or invalidate saved reads before assigning a new free ball. Test a same-tick deflection after a close read, rather than advancing the clock before checking the replacement prediction.

### P2-2 — Equal-priority corner outfielders never resolve their heard sure calls

**Locations:** `sim/crates/bb_game/src/live.rs:280`, `1628–1631`, `1818`.

LF and RF both have priority 1. `heard_call()` uses an index tie-break when selecting a heard call, but `decide()` compares only `(tier, CALL_PRIORITY)`. Two sure corner-outfielder calls therefore never outrank each other. Both continue even after hearing the other; repeated decisions do not resolve it. `lead()` selecting one of them does not change either man's decision.

**Probe:** error-free reads of a center-field fly (`v=(0,33,22)`), LF/RF at `(-5,92)` and `(5,92)`, CF away at `(70,60)`, corner read sigma 0.01. After publication and the hearing interval, both calls are tier 3, each hears the other's tier-3 call, both roles remain Chase, and both decisions are Go. This is persistent convergence, beyond the intended overlap before communication arrives. Game frequency was not measured.

**Suggested fix:** define and consistently apply a complete deterministic order for equal-priority sure callers, using information both can know. Alternatively specify and implement a further audible tie-resolution exchange. Check that exactly one continues after that resolution and the other receives a cover/backup job.

### P2-3 — The ledger combines different counterfactuals and overstates causal attribution

**Locations:** `docs/metrics-ledger.md:151,293`; `docs/calibration/2026-11-14-fielder-communication.md:75–82`; `docs/reviews/2026-11-14-codex-live-play-review.md:233`. Relevant behavior: `sim/crates/bb_game/src/live.rs:1855–1863`, `1900–1904`, `2044–2048`, `2075–2080`, `2103`.

The ledger says “the same build with every knowledge delay set to zero gives 5.76 [6.01] and, on 1,000 games, 6.07.” Its supporting table identifies **5.76 [6.01] as `8861a38` with zero delay**, before the outfielder comparison fix, and **6.07 as `636366d` with every use instant except decisions**. Neither is the stated final-build/all-delay-zero comparison. The 1,000-game comparison still contains delayed decisions. These results suggest sensitivity to communication changes, but do not isolate the final delay from every structural change or establish that the relay explains most of the final increase. The recorded single-channel interventions also need not add independently after game histories diverge.

There are two concrete reasons to revisit the proposed mechanism:

- The assertion that the relay goes out only once the ball is down is false for deep balls: `relay_job()` explicitly sends him early when his own read rules out an air play. The existing `relay_man_goes_out_early_on_a_deep_fly` test passes.
- In an instrumented, behavior-preserving run of seeds 0–19, **32 loose-ball intervals lacked a known lead for 0.600 s** after assignment. For example seed 0/play 116: `t=0.657..1.257`. Initial mutual deferral lasts one hearing interval; the subsequently started chase needs another interval to become known. `fielding_point()` remains None, so relay assignment cannot start throughout that interval. This contradicts the note's §4 claim that this wait is “0.3 s at most.” No interval with **no actual chaser** exceeded 0.35 s in this sample; this is delayed knowledge, not evidence of a prolonged abandoned ball.

These observations identify timing mechanisms worth measuring, not an explanation of the reported +0.22/+0.07 runs or +5% doubles. Counting job transitions alone does not measure departure time, distance lost, or whether the relay was ready for a throw. I did not rerun the 300/1,000-game rate comparisons.

**Suggested fix:** label each counterfactual with its exact revision and intervention; soften the categorical attribution. Compare the final revision against its own zero-delay variant on identical seed sets. For the relay hypothesis, record initial assignment/chase/hearing/relay-departure times, ball-down time, relay position at release and reception, and the first differing play. Preserve the intervention patches or reproducible commands. Correct the early-relay and maximum-wait descriptions.

### P3-1 — A decision repeats identical teammate-intercept scans

**Locations:** `sim/crates/bb_game/src/live.rs:1638–1642`, `1722`, `1730–1742`, `1765`, `1777–1785`.

For a middle infielder yielding a grounder to his partner, `decide()` calls `grounder_break()`, which calls `partner_fields_it()`, then calls `partner_fields_it()` again after the break returns false. Both scans use the same observer prediction, teammate body, and tick. Other fall-through paths can scan the partner again in `first_by_own_estimate()`.

**Probe:** the CoverSecond isolation fixture above counted **two identical partner scans in one decision**. Across the instrumented 20-game sample, `seen_intercept()` ran 44,842 times and consumed approximately 0.678 s of 8.646 s total game time (about 7.8%, including instrumentation overhead). This is not a paired thermal benchmark and does not independently verify the claimed 28% slowdown or attribute all of it to this function.

**Suggested fix:** compute the partner result once per decision, or cache each observer/teammate intercept within the decision batch. Retain separate observer predictions and invalidate on body motion, time or ball/read changes; a cache retained merely “per read” across ticks would be stale. Share path samples within an observer's scan batch if further profiling justifies it. Verify identical digests after the optimization.

## Invariant and test assessment

- **Private-state boundary:** own read/call/intercept use in `decide`, `grounder_break`, `partner_fields_it`, and `first_by_own_estimate` is appropriate. `seen_intercept`/`eta` use teammate present motion and known attributes, rather than his target/belief/intercept. `heard_call`/`lead` use delayed publications. The old shared private-target fielding point and private-intercept comparison in `relay_job` are removed. The remaining cross-decision paths are P1-1 above. Including an actual chaser in `recall` to refresh **his own** read is not itself a teammate information leak.
- **World-state boundary:** exact current ball/body motion is expressly permitted by the interim communication spec. I treated that as the documented observation simplification. This review does not close the earlier review's separate world-side capture/future-timing findings. The crate-graph test passes, but cannot establish information isolation inside `bb_game`.
- **Outcome randomness and determinism:** no new outcome roll, unordered hash iteration, direct transcendental call, or non-keyed RNG was found in the production change. Prediction and teammate scans are deterministic on saved perception errors. The added scans introduce no RNG draw; the existing stream constructors and within-stream draw sequences are unchanged. Different movement, calls and attempts deliberately change which later keyed executions occur, consistent with the documented changed digests. Old separately reported SurfaceBounce allocation issues are outside this change's resolution.
- **Parameters:** `CALL_HEARD_S` and `CALL_PRIORITY` are named and explicitly tagged [E] in code and agent-models §5.7. The former maps to the existing estimated `T_call=0.3 s`; the latter's estimated ordering is stated in the spec and calibration note. They are not represented as empirically sourced measurements. No missing [E] declaration was found for these two parameters.
- **Tests:** the private-target test is a useful independent mutation/isolation check for its named path; the delay test checks a literal 0.3-second expectation. In a scratch mutation changing `CALL_HEARD_S` to zero, the delay test failed with “gave it up 0.017 s after the call.” They are not mere restatements of the implementation, but their coverage is narrow: both pass with the P1-1 leaks present. The scratch library suite also passed with all four defect probes recording their reproductions. The FFI change admits an already-completed short flip and does not alter simulation behavior.
- **Believability:** bounded initial mutual deferral and temporary overlapping chases are represented. The observed longer knowledge gap, persistent sure-call tie, and stale post-deflection destination need the distinctions above. The documented temporary third/home coverage gaps remain explicit simplifications; I did not establish that every possible base-cover interaction is safe.

## Verification

Every cargo invocation ran from `sim/`, using `export PATH=$HOME/.cargo/bin:$PATH CARGO_BUILD_JOBS=2; taskset -c 0-7 nice cargo ...`. Scratch linker caches were redirected under `sim/target/scratch/` because the default Zig cache is outside the writable sandbox. No sudo or source-tree edits were used.

1. Original checkout: `cargo test --workspace --release`: **322 passed, 0 failed, 3 ignored**, including the cross-thread same-seed/full-log, recording-isolation and crate-graph tests.
2. Original checkout: `cargo run --release -p bb_game --example digest`, twice. Seeds **1, 2, 3 and 42 matched on both runs**. Seed 42: `b5c2680d798b6053599d4cde6c300868979ed48024534eca46249b75ca3f008a`.
3. Copied workspace under `sim/target/scratch/comm-review/`, with isolated scratch build output: four throwaway unit probes for backup isolation, CoverSecond isolation, a same-tick replacement ball, and equal-priority corner calls. Restored-delay scratch library run: **146 passed, 0 failed, 3 ignored** (142 existing passing tests plus four probes). Probes asserted/printed the defect behavior; they were diagnostic reproductions, not assertions that the defects were fixed.
4. Scratch zero-delay mutation: `a_call_is_not_heard_before_its_delay` **failed as expected**. Restoring 0.3 s restored its passing result.
5. Scratch diagnostic example: simulated seeds **0–19**, instrumenting actual-chaser/known-lead gaps and teammate-scan cost without changing decisions. Inspected the 32 long knowledge intervals and counted duplicate scans in the constructed CoverSecond case. No 300/1,000-game distribution or isolated relay-causality rerun was performed.
6. Final original-checkout library rebuild/check: `cargo test --release -p bb_game --lib`: **142 passed, 0 failed, 3 ignored**.

All throwaway probe sources, copied workspaces, diagnostic logs and scratch caches created for this review were removed. Only this review is a tracked deliverable; source, tests, specs, ledger, previous review and TASKS are unchanged.

## Resolution

**Open: nothing fixed yet, and `work/live-comm` is not merged** (lead "Fable Swing", 2026-11-14; the session paused before the fixes).

- **P1-1 (coverage and backups still read unpublished choices): open.** It blocks the merge. The earlier live-play review's Resolution now says "partially fixed".
- **P2-1 (stale read after a same-tick deflection): open.** Fix with P1-1.
- **P2-2 (two corner outfielders both keep going): open.** Fix with P1-1.
- **P2-3 (ledger attribution overstated): accepted, open.** The ledger row now carries a caution. The counterfactuals need redoing on the final revision once P1-1 is fixed; the 0.6 s wait for a known lead that Codex found is the first thing to measure.
- **P3-1 (repeated intercept scans): open.** Do after the above, checking digests are unchanged.
