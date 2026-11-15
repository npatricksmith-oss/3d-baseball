Found two P1 issues: an unpublished relay still schedules teammates’ cover decisions, and the public recovery order can send the wrong man and turn a catch into a hit. Do not merge yet.

Reviewed `66196ba..f3d9e7a` on `work/live-comm`, principally `faf2737` and its integration with main `57f12b7`. Read `CLAUDE.md` first, the preceding review and recheck with their Resolutions, agent-models §5.7, the calibration note and metrics ledger. Main’s swing/refit work was examined only for merge preservation. No source, test, spec, ledger or TASKS file was edited; no commit was made. Scratch revisions and probes are under `sim/target/third/`.

## Findings

### P1-1 — Relay retirement still exposes private state through a scheduling gate

**Locations:** `sim/crates/bb_game/src/live.rs:2304`, `2319`, `2324`; related cover-trigger gates at `2286` and `2332`.

The replacement relay selection fixes the previous probe, but `update_relay()` still asks whether **any actual relay retired**, then calls `cover_roles_after_chasers()` for everyone. The cover tables read knowledge correctly; whether another man gets to consult his table still depends on an unpublished job. This is the same class of invariant failure as the recheck’s private-peel scheduling leak.

**Targeted isolation probe:** 3B holds the ball, 1B is known to be chasing, P is idle. Keep ball, bodies, attributes, all publications and P’s private state fixed. Change only SS’s unheard role Idle → Relay(4); SS remains known Idle. Calling `update_relay()` leaves P Idle in the first case and makes him Cover(1) in the second. `third_relay_retirement_schedule_isolation` fails.

**Probe using a real game snapshot:** immediately before `update_relay()` on raw seed **0/play 0**, `t=1.998330571`, clone the simulation’s relevant state and run the function with/without changing only 2B’s current role Idle → Relay(4). His known job stays Idle. C changes from Backup, target **(22.062929, 6.673568)**, to Cover(4), target **(0,0)**. The original game is left untouched. This demonstrates the missed scheduling context without relying on an arbitrarily stale cover-table fixture. All 50 instrumented game digests still match the original branch.

The original holder-route, relay-selection and peel-expiry probes now pass, as do the stable-state role and private-plan matrices. Those passes do not close this gate.

**Suggested fix:** give each man a public or personal schedule for refreshing his cover choice. A man retiring his relay may take his own next job; his private retirement must not refresh everyone else. Audit both `was_cover` triggers for the same issue. Retain the failing retirement probe and a real-snapshot variant.

### P1-2 — The shared order ranks proximity to a ray, not ability to field the ball

**Locations:** `sim/crates/bb_game/src/live.rs:1731`, `1745`, `2008`, `2053`; corresponding convention in `docs/agent-models.md:538`.

The order is computable from public knowledge: `eta()` and its helpers use visible position/velocity, known physical attributes, contact time and field geometry. No private target/read enters it. Its strict rank and index tie-break also prevent a cycle of yielding through that order within one decision batch. The sampled all-nine feedback cycle is fixed.

However, `track_point()` projects onto an **unbounded forward ground ray**, and `public_rank()` counts only the man’s travel to that ray. It ignores when, whether and at what height the ball reaches that point. Being already on the ray gives rank zero even dozens of metres beyond where a slow ball stops; being near its beginning can nominate an infielder for an outfield fly.

**Slow-ball probe:** a loose ball starts at **(0,30,0.05)**, velocity **(0,2,0)**; all men are known not to be chasing and first is covered. After the hearing interval it stops at approximately **(−0.002,30.637)**. Recovery selects **CF at (0,97.536)** with public rank **0**, although his estimated run to the stopped ball is **9.073 s**, versus P’s **2.142 s**. `third_recovery_slow_ball_uses_reachable_track` fails. This is a deterministic geometric error, not a noisy disagreement between fielders.

**Actual fly, replayed with identical inputs:** raw seed **2/play 140**, EV **34.753 m/s**, LA **34.008°**, spray **−9.505°**. Main’s defense catches it with CF at **t=4.953134**, just before its first bounce at **4.965227**, near **(−22.28,84.27)**. On the branch everyone initially defers; at **0.953134** recovery sends **P**, whose own intercept is about **12.13 s**. At 1.403 he is running toward the outfield while CF remains idle. SS subsequently chases; the outfielders are not recalled into the chase until about **3.37 s**. LF finally fields it at **8.019801**, and it is a hit. This replay uses the branch’s exact contact, players, runners, rules state and RNG keys for both defenses, rather than comparing different later pitches in divergent games.

**Suggested fix:** retain a shared convention, but rank a physically reachable, time-dependent ball opportunity using an agreed model of the current public observation. Account for descent and finite ground travel. Do not substitute the world’s realized future or another man’s private prediction. Keep first-base protection and test both the slow-ball and outfield-fly counterexamples; revise the specified convention with the fix.

### P2-1 — Some second-base vacancies are forced by the partner exclusion, even with perfect reads

**Locations:** `sim/crates/bb_game/src/live.rs:1840`, `1690`; interpretation and claimed return time at `docs/calibration/2026-11-14-fielder-communication.md:148`.

A partner known not to be chasing is excluded outright, including a man covering second who is about to reconsider the same grounder. That is stronger than uncertainty about who will win a close race.

**Probe:** use the existing pivot-test grounder, velocity **(−1.5,24,−3)**, zero all perception-error draws, and make 2B known Idle and SS known Cover(2) on this ball. Fresh reads give fielding times **2.690 s** and **2.930 s**. Nevertheless both `partner_fields_it()` results are false and both decisions are **Go**. `third_perfect_middle_reads_keep_one_at_second` fails. SS ignores the objectively earlier partner because that partner was not previously chasing.

The natural sample also does not support “in every case” returning 0.3–0.6 s after hearing the other. On **38/play 18**, SS leaves second at **1.413260**; his chase is known by **1.713260**, but both continue until 2B returns to Cover(2) at **2.746594**: **1.333 s** uncovered, over **1 s after hearing**. Their subsequent reads continue to favor chasing until the grounder passes them.

The first-read and uncertain-fly cases can represent believable independent mistakes. The perfect-read grounder case is a tactical rule defect; I would not classify all 18 vacancies as merely believable noisy mix-ups. This is materially smaller than the old all-nine/base-abandonment P1.

**Suggested fix:** distinguish a partner committed to an incompatible job from one idle or covering the pivot bag who may field this grounder on the same reread. Preserve one middle infielder’s coverage when their opportunity estimates clearly agree. Retain legitimate close-read disagreements, add the perfect-read regression, and correct the note’s return-time claim.

### P3-1 — The calibration note retains contradictory pre-merge coverage conclusions

**Location:** `docs/calibration/2026-11-14-fielder-communication.md:148`.

After correctly giving the new main/branch table and explaining two first-base gaps, the paragraph still says first “never” lacks a cover for more than 0.3 s and quotes second **26 against 7**, third **33 against 43**. The reproduced merged counts are first **2 against 0**, second **18 against 11**, third **29 against 68**.

**Suggested fix:** remove the stale closing sentences or explicitly label them as pre-merge results. Keep the measured durations separate from judgments about whether a live play was actually endangered.

### P3-2 — The allocation fix is sound, but the broader slowdown remains

**Locations:** `sim/crates/bb_game/src/live.rs:1546`, `1719`, `1966`; cost claim at `docs/calibration/2026-11-14-fielder-communication.md:152`.

The six-slot buffer is correctly bounded: at most five subdivision samples plus the endpoint, consumed in the original order. A scratch reversal to the previous `Vec` sampling produces **all 50 identical full-game digests**.

With both CLIs built without default features, alternating **main, branch, main, branch**, 300 games from CLI seed 0 took **20.7, 27.9, 18.6, 23.9 s**. That is about **35% and 28%** slower per pair, consistent with the note’s approximate remaining cost within this machine’s variability. The fixed buffer has not removed it.

**Suggested fix:** keep this as performance debt and profile the corrected revision. Public ranks are pure for a fixed body/ball snapshot and are repeatedly recalculated inside comparisons; measure caching those per decision batch before attempting the broader observer/intercept cache. Preserve digest and information-isolation checks.

## Other review conclusions

**Recovery and coverage measurement, literal seeds 0–49:**

| Measurement | Main `57f12b7` | Branch `f3d9e7a` |
|---|---:|---:|
| Live plays | 3,781 | 3,790 |
| All-nine chase starts | 0 | 0 |
| Plays with repeated all-nine starts | 0 | 0 |
| Actual no-chaser intervals | 0 | 59, all 0.300 s |
| No-known-lead intervals >0.3 s | 0 | 59, maximum 0.600 s |
| First uncovered >0.3 s | 0 | 2, maximum 1.017 s |
| Second uncovered >0.3 s | 11 | 18, maximum 1.333 s |
| Third uncovered >0.3 s | 68 | 29, maximum 2.417 s |
| Double-cover intervals >0.3 s, any base | 0 | 0 |

These are loose-ball missing-job intervals after initial assignment, not travel time to a covered base. The two first-base gaps reproduce at **35/75, 7.629911..8.629911**, and **37/34, 4.397049..5.413715**. The pitcher takes home as the catcher’s chase becomes known, then keeps that cover while first becomes vacant. The preservation of existing covers at `live.rs:2583` explains why first does not immediately take precedence. But in 35/75 the sole batter-runner is **already out**; in 37/34 he reaches first and heads for second while the loose ball is away to the right. Neither trace establishes a lost out caused by the missing first-base job. I would not make these two intervals another P1. Revisit priority among threatened bases with a live-runner regression, rather than treating every unassigned bag as an equally urgent defect.

**Standard numbers reproduce:**

| Build | Runs/team-game, seed 0 [1000] | Fly-ball hit rate, seed 0 [1000] |
|---|---:|---:|
| Main `57f12b7` | 4.31 [4.68] | .212 [.215] |
| Branch | 4.70 [4.72] | .234 [.242] |
| Branch, only `CALL_HEARD_S` changed to zero | 4.46 [4.45] | .226 [.227] |

All runs had zero rules warnings. These reproduce the two 300-game sets; this review did not rerun the ledger’s 1,000-game attribution.

**Fly-ball mechanisms:** the paired replay ran both defenses against each live input from 50 branch games. Across 1,137 contact trajectories with 25–50° launch, including fouls and home runs, there were **17 catches lost and 16 gained**. That sample does not quantitatively explain the full standard-run rise. Besides P1-2, detailed examples are:

- **11/1:** main CF catches at **4.948730**, first bounce **4.986504**. Branch RF initially chases and CF backs him up; CF starts chasing only around **3.3 s**, then fields the bounce at **5.348730**. This is delayed pursuit/selection between outfielders, not a failed glove roll.
- **4/280:** main **LF, while backing up CF**, catches at **6.285334**, first bounce **6.305548**. At 6.185 main LF stands near **(−15.11,85.30)**; branch LF is near **(−15.12,87.30)**, still backing up the higher-priority CF call despite his own sure read. The deeper backup geometry leaves the descending ball beyond standing reach. Branch CF fields it at **7.268667**. This is a changed backup position, not evidence that a peel marker forbids LF’s glove attempt: LF has no peel marker.
- **17/47, opposite direction:** branch CF catches at **5.607161**; main CF misses and fields at **7.590494**. The branch starts him earlier and changes his approach.

Thus there are concrete recovery, pursuit and backup mechanisms worth tracing over a larger paired sample. The entire .022 [.027] aggregate increase is not established as the cost of realistic hearing delay, and these examples do not establish a universal relay or called-off-man explanation.

**Cache and invariants:** the three original isolation probes and both same-tick cache probes pass. On the real grounded-throw/rim sequence, CF’s intercept changes from **(5.0183,72.2986)** to **(5.9134,22.8918)** after the second free ball; the two physical resting points differ by over 60 m. The generation counter fixes the reported cache defect. No new production hash iteration, direct transcendental math, non-keyed RNG, outcome roll or `bb_agents → bb_world` dependency was found. Exact current ball/body motion remains the documented simplification. Information isolation still fails at P1-1; repeatable digests do not excuse it.

**Merge preservation:** `git diff 57f12b7 f3d9e7a --stat` touches only this change’s source/example, coverage analyzer, review/spec/note/ledger and TASKS files. The swing, physics, population/refit, PA and FFI sources have no diff from main. Both main’s ledger rows and the communication rows survive. The FFI timeline test equals main’s version, including the already-caught short-throw exception and assertion; the merged comment preserves main’s refit example. No main swing work was lost.

## Verification

All cargo builds/tests and simulator executions ran from `sim/`, prefixed `taskset -c 0-7 nice`, with `CARGO_BUILD_JOBS=2`, the requested PATH and Zig caches under `sim/target/`. Heavy jobs were serialized. Scratch-only probe adaptations accommodated the existing two-argument `receiver_for`; initial probe compilation errors were corrected before the reported runs.

- `cargo test --workspace --release`: **353 passed, zero failed, three ignored**, including **156 bb_game library tests**, crate-graph, recording-isolation, cross-thread determinism and FFI timeline tests.
- Recheck probes transplanted to scratch final: **six passed**, including the three previous isolation failures, two cache cases and role-consumer diagnostic matrix. The wider private-plan matrix also passes. Their stable contexts miss the newly failing retirement schedule.
- Targeted new probes: retirement isolation, slow-ball recovery and perfect-read middle-infielder coverage **fail** as described; these are review failures, not failures in the committed suite.
- Read-only coverage traces on branch and main, each literal seeds 0–49; independent interval/all-nine counts and `analysis/coverage_gaps.py` agree with the table.
- Fresh main/branch CLI builds and both 300-game standard seed sets; a pristine archived branch with only the hearing constant changed supplies both zero-delay sets. Separate scratch target directories prevent cross-revision library reuse.
- Original digest example twice: seeds **1, 2, 3, 42** identical. Seed 42: `6f8d2fbed1656a3114168eff1c93b9d273aa9743e6441383744c8c4ca47ffd71`.
- Dual-defense replay with cloned initial rules state and identical live inputs, 50 games; **all 50 resulting branch digests match** the original branch trace. Detailed replays also inspect the two first-base gaps and 38/18.
- Fixed buffer versus scratch `Vec` version: **all 50 digests match**. Two alternating timing pairs use matching CLI feature configurations.
- `git diff --check` clean before this report; merge/file comparisons as above. This report is the only tracked change.

## Resolution

Worker Claude Opus 5.5 for lead "Fable Lead", 2026-11-14. Code in `74ea51b`; docs, numbers and this Resolution after it.

- **P1-1 (scheduling leaks): fixed as a class, with a general harness.**
  - The gate: a man leaving the cutoff job (`update_relay`) or taking it from a base (`update_relay`, `assign_relay`, both `was_cover` triggers) re-worked every teammate's cover at once. Now he takes his own next job (`take_own_job`: his own cover or backup table) and nobody else is moved; his teammates re-work theirs when his change is known (`hear`, a public trigger). The schedules that remain are public or a man's own: the first read (the earliest known reaction), the once-a-second re-call (from the known lead's distance to the ball), a new loose ball, the ball coming down, a ball secured, a throw released, a change in what is known (`hear`), and each man's own replan and close read.
  - **Harness** (`isolation_no_private_state_reaches_a_teammate_in_real_games`, tests only): a hook before each coordination step of a live play (`assign`, `recall`, `expire_peels`, `chasers_see_the_ball_secured`, `holder_act`, `cover_home_again`, `hear`, `update_relay`) forks the real game, changes one man's private state (each other job, his target, his plan, his backup of a catch, his call, his read; never what he has shown), runs the step on both forks and requires every other man's job, target, backup base, peel, call, read time and plan, and the ball's state, to be the same. The holder's own state may decide his throw. Seeds 0–49, every 200th call of each step by default (`ISO_SEEDS`, `ISO_STRIDE` widen it). On `f3d9e7a` it found 40 leaks in the first three games, all in the cutoff step. After the fix a dense run (every 7th call, seeds 0–9, 766 s) found none, so no other gate turned up. The review's two probes are tests: `isolation_seed_0_play_0_a_private_cutoff_job_moves_nobody` (the real snapshot, t = 1.998 s) and `isolation_a_private_cutoff_retiring_moves_nobody` (constructed); both failed on `f3d9e7a`.
  - Not covered by the harness: `try_captures` and `update_thrown` (a man's own glove and catch change the ball itself, which everyone then sees), and the runners.
- **P1-2 (recovery order): fixed.** `public_rank` is now each man's earliest intercept (`seen_intercept`: standing reach, a dive, the ball's descent, where it stops) of `public_path`: the ball's present position and velocity, the typical spin for how it is moving (`believed_spin`), run on with the fielders' own physics and no bounce scatter, as an error-free read is. Every man can compute it from the present public observation; it uses no world future and no man's own read. First-base protection is unchanged. Tests: `the_man_sent_for_a_ball_nobody_took_can_get_to_it` (the review's slow ball: the pitcher is sent, not the centre fielder standing on its line) and `the_man_sent_for_a_fly_nobody_took_is_an_outfielder` (seed 2 play 140's contact: an outfielder, not the pitcher); both fail under the old order. agent-models §5.7 is revised.
- **P2-1 (partner exclusion): fixed.** `partner_fields_it` leaves out a partner only when he is known to be on another base or out as the cutoff; one idle, backing up or covering second is weighed. Test `error_free_middle_infielders_keep_one_at_second` (the review's probe: 2.690 against 2.930 s; both went before, now one goes and the other stays at second). The note's "0.3–0.6 s" claim is corrected (seed 38 play 18 was 1.333 s).
- **P3-1:** the note's stale pre-merge coverage sentences are removed or labelled.
- **P3-2:** public ranks are computed once a tick for all nine and kept while time, ball and positions are unchanged (`rank_cache`). PROGRESS_P32

PROGRESS_REST
