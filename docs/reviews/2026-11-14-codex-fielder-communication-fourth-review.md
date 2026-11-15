One P1 remains: an unpublished holder decision moves a teammate. Do not merge.

Reviewed `ae440b9..5cfb10e`, code `74ea51b`, against main `57f12b7`. Read `CLAUDE.md` first, the third review and its Resolution, agent-models §5.7, the calibration note and metrics ledger. Audited the live tick, coordination consumers, `live/holder.rs`, isolation harness and rank cache. The checkout actually starts at documentation-only `847b694`; its production `live.rs` and `live/holder.rs` are byte-identical to `5cfb10e`. The intervening Opus review was read as context, not used as verification. All instrumentation and counterfactual builds are under `sim/target/codex4/`. No production source, committed test, spec, ledger or TASKS file was edited, and no commit was made.

## Findings

### P1-1 — The holder still commands another man's target before any observable throw or publication

**Locations:** `sim/crates/bb_game/src/live.rs:3718–3720`; harness exemption at `6721–6722`; receiver selection at `3816`. The holder's timing/option calculation is in `sim/crates/bb_game/src/live/holder.rs:41–79` and `137–199`.

After privately choosing a destination and receiver, `holder_act` checks whether the receiver can arrive in time. If he cannot, it writes that teammate's target to the bag and returns **without throwing**. The receiver does not make this decision from his own cover table or a heard request. His published job can remain Backup or Relay. The harness skips perturbing the holder altogether, so its passing result cannot detect this path.

**Paired-state probe:** before each `holder_act` in literal seeds 0–9, fork the real state and change only the holder's internal preparation duration, `set_base += 1.0`. Keep ball, bodies, attributes, receiver state and every publication identical. Compare other men's targets only when **both executions leave the ball Held**, excluding differences caused by a real release. `fourth_holder_pre_release_isolation` fails:

- **1/195, t=7.219582563:** CF holds the ball. LF is publicly Backup of second. One execution leaves his target at **(−5.244504, 26.899532)**; the other sends him to second, **(0, 38.794706)**, a 13 m redirect. Neither execution releases the ball or publishes a request.
- **8/121, t=4.164621129 onward:** CF's preparation changes whether 2B's Relay(3) target is overwritten with second base. This repeats across eight ticks, then reaches the idle SS at `4.297954463`.

This code also exists on main (`57f12b7:live.rs:3126–3128`); it is an inherited defect, not a newly introduced regression. Nevertheless, the requested isolation guarantee and binding invariant have no exception for it. I therefore rank it P1 rather than treating its age as permission to merge with the guarantee still false. No changed out is claimed from this probe; the established failure is information/control isolation.

**Suggested fix:** let the receiver take a job from his own public inputs, or publish the holder's request and let the receiver respond after the communication delay. Keep the holder's choice local while he waits. Permit physical releases to affect observers, but remove the blanket holder exemption: assert that unchanged public observations cannot produce different non-holder decisions before release. Retain the real-snapshot counterexample.

### P2-1 — The harness is a useful negative control, but does not establish isolation as a class

**Locations:** `sim/crates/bb_game/src/live.rs:6639–6650`, `6657–6677`, `6707–6722`, `3010`; tick decisions at `4148–4160`, `2695–2706`, and throw backups at `3754`.

The eight hooked coordination steps cover the repaired cutoff scheduling gates, and the committed default run passes. However:

- `set_free` calls `assign` outside the initial-assignment hook. Assignments after deflections and grounded throws are not checked there.
- Replans and close-range reads can change decisions/targets outside these hooks. Excluding all of `try_captures` and `update_thrown` because they also contain physical ball events leaves their separable decision portions unchecked. P2-2 below illustrates why those reads matter.
- Role flips omit Cover(3/4) and Relay(1/2/3); private `backs`, error draws and planning clocks are not flipped. Clearing a belief does not exercise different valid beliefs on opposite sides of a decision threshold.
- Outputs omit `next_plan`, belief, publications and other decision state. `iso_ball` compares a free-ball label or thrown destination/receiver, not the throw trajectory or read/scheduling fields. Holder option estimates can change without a final action changing in the sampled step.
- Every 200th invocation undersamples first assignments and rare transitions. The holder exemption hides P1-1.

**Independent checks:** a fresh archive of `f3d9e7a` with the committed harness transplanted fails: **903 violating perturbations at 107 sampled step snapshots**, all `update_relay`, over seeds `0..3`, stride 200, **218,933 checks**. I uncapped diagnostics to count failures. This confirms the negative control, but does not reproduce the exact claimed count of 40 under these settings.

A wider scratch harness adds the missing roles, backup bases, error/scheduling inputs, comparisons of all non-belief fielder fields plus belief samples, holder option comparisons, runner comparisons, new-loose assignments and explicit throw backups. Over seeds 0–2 it passes **575,760 perturbations**. New-loose assignments and throw backups run at stride 1, recalls at stride 10, other steps at stride 200. This includes 9 new-loose assignments and 145 throw-backup calls. It retains the holder exemption; the separate pre-release probe supplies that missing test and fails. These are finite sampled checks, not proof of the entire class.

**Suggested fix:** hook every decision entry point, including calls reached inside physical-event functions; compare decision schedules, beliefs/publications and relevant ball parameters as well as final roles/targets. Exercise valid alternative reads and all role/backup variants. Give rare transitions dense sampling, and use an observable-event-aware holder check instead of skipping him.

### P2-2 — A sure call survives fresh reads that say its caller cannot catch the ball

**Locations:** `sim/crates/bb_game/src/live.rs:1625–1627`, `2702–2706`, `1979`; confidence calculation at `2124–2140`.

`replan` and the close read update the caller's belief, intercept and target but leave `call` unchanged. `publish` continues using that stored tier. The periodic re-call need not repair it: the tick stops recalling once the known lead is close to the ball. Consequently a fielder can continue advertising certainty after his own current prediction says the ball bounces before he reaches it.

**Preserved-input replay, 4/280:** main LF catches at **6.285334**, before the first bounce at **6.305548**; the reviewed defense fields it with CF at **7.268667**. The previously recorded 2 m deeper LF backup position reproduces. An additional read-only diagnostic shows that at **6.185334**, CF still has tier **3**, but his latest read at **6.168667** predicts an intercept at **6.918667** and its own first bounce at **6.305311**. Applying `call_value` to that same belief returns **0**. Stored tier 3 also survives earlier fresh reads returning 0. The diagnostic preserves the original seed-4 digest.

Backing up too deep and initially misjudging a fly are plausible mistakes. A confidence signal that the implementation cannot withdraw on these fresh reads is a separate rule defect; the recorded backup-depth explanation is incomplete. This probe does not establish how much of the lost catch comes from the stale call, or that withdrawing it alone restores the catch.

**Suggested fix:** reconsider the man's call when his own replan/close read changes its confidence. Publish any change through the existing delay, without refreshing teammates from an unpublished change. Test a sure caller losing his air play, the delayed withdrawal, and a backup's subsequent reconsideration. Re-measure the paired case before attributing its outcome solely to backup geometry.

### P3-1 — Current coverage prose still contradicts the reproduced measurements

**Locations:** `docs/agent-models.md:538`; `docs/calibration/2026-11-14-fielder-communication.md:56`, `152`; stale final-revision figures also remain at `docs/metrics-ledger.md:305`.

The new §7 paragraph and ledger row correctly say first **3**, second **15**, third **31**. The spec still presents **2/18/29** as the measurement, and the note twice says first never lacks a cover beyond 0.3 s. Its closing paragraph also repeats **26 against 7** and **33 against 43** without marking those closing claims as historical. The ledger's later explanatory paragraph still quotes the preceding revision's +0.17 delay estimate and .234 [.242] fly rates.

**Suggested fix:** retain historical tables with their revisions, but replace or explicitly label stale present-tense conclusions. Use one current measurement summary. The longest current second-base gap is **2.067 s at 3/140**; the old 1.333 s example is not the current maximum.

## Other review conclusions

**Recovery order:** the third review's ranking mechanism is fixed. `public_path` starts from the current observed position/velocity, supplies typical believed spin and deterministic agent physics with zero bounce scatter. `seen_intercept` uses public bodies/motion, known attributes and reaction time, not private targets or the world's future path. Changing true spin/future while preserving the current public sample leaves ranks unchanged in the scratch probe. Strict `(rank, index)` comparisons prevent a cycle within a decision batch; this is not a claim of an invariant ranking across moving snapshots.

The geometry survey selects **P** for the slow roller, **CF** for the deep fly, **P** for the mound pop-up, **RF** down the right line, and **CF** for the sampled ball past the outfielders. All exclude the known Cover(1) man. The slow-ball regression and perfect-read middle-infielder regression pass in the committed suite.

On the **exact preserved 2/140 input**, recovery now sends CF rather than P. It still loses main's catch: main secures at **4.953134** before the bounce at **4.965227**; current CF fields at **5.103134**. The correction fixes the wrong-man mechanism, not every consequence of initial mutual deferral. The remaining delayed start is consistent with the specified hearing/recovery model, unlike the old pitcher running toward an outfield fly.

**Coverage, literal seeds 0–49:**

| Loose-ball measurement | Main `57f12b7` | Reviewed branch |
|---|---:|---:|
| Live plays | 3,781 | 3,780 |
| First uncovered >0.3 s | 0 | 3 |
| Second uncovered >0.3 s | 11 | 15 |
| Third uncovered >0.3 s | 68 | 31 |
| Double covers >0.3 s, any base | 0 | 0 |
| Nobody chasing >0.3 s | 0 | 0 |
| Repeated all-nine chase starts | 0 | 0 |

The branch's three first-base gaps are:

- **26/10, 3.802250–4.218917 (0.417 s):** 1B leaves as cutoff to home; 2B replaces him after the job becomes known. The earlier throw has escaped into right field. The batter touches first at **4.652250**; RF does not secure the ball until **6.252250**.
- **37/34, 4.397049–5.413715 (1.017 s):** the escaped throw draws 1B and C after it; P retains home before later taking first. The batter touches first at **4.697049**; C secures the loose ball at **7.080382**.
- **41/222, 0.654333–1.404333 (0.750 s):** P, 1B and others initially pursue a foul fly; covers return as their decisions change. 1B catches it at **5.754333** for the inning-ending out.

None establishes an out lost because first lacked an assigned cover during that interval. After each of the first two gaps starts, nobody secures the loose ball before the batter arrives; the third ends in the catch. These counts measure jobs, not physical arrival at a bag. Second's remaining excess is much smaller after the partner fix; its worst current gap, 3/140, is an airborne pursuit and ends in a catch.

**Numbers and paired flies:** the branch's two fresh standard runs reproduce **4.46 [4.58]** runs/team-game and fly-ball hit rates **.222 [.231]**, with zero rules warnings. A fresh dual-defense replay on the same inputs and cloned rules states reproduces **1,129 flies, 14 catches lost, 15 gained**, and all 50 branch digests match pristine games. This small paired sample neither explains the aggregate rate change nor proves the defenses equivalent.

The **11/1** miss reproduces: main CF catches at **4.948730**; branch CF fields at **5.348730** after initially backing RF. Selection between independently unsure outfielders and delayed takeover can reasonably remain a recorded model mistake. The **4/280** depth mechanism also reproduces, but P2-2 prevents endorsing its full explanation as merely a realistic outfield error. I did not rerun the zero-delay variants or the 1,000-game attribution sets.

**Determinism and cache:** seeds **1, 2, 3, 42** repeat exactly. Bypassing `rank_cache` in a fresh scratch build preserves all four digests; seed 42 is `c3f7a99c4f510d6131d8bc41d31144228765c3436c388ec9df3ab9b5998bfb4a`. Direct probes agree with uncached ranks after time advances, a body moves, and `set_free` changes the ball generation. The key omits velocity, although ranks depend on it; current tick ordering changes velocities in `move_fielders` after decisions and advances time before the next use, so I found no reachable stale-cache case. Including velocity would harden that assumption, not fix an established current failure.

No new production hash iteration, direct transcendental math, unkeyed RNG, outcome roll or agent-to-world crate dependency was found. Determinism does not close P1-1. The diff against the specified main preserves its swing/physics/PA/FFI work.

## Verification

All builds, tests and simulator runs were serialized, run from a `sim/` directory, prefixed `taskset -c 0-7 nice`, with `CARGO_BUILD_JOBS=2`, the requested PATH and Zig caches under the worktree's `sim/target/`.

- `cargo test --workspace --release`: **359 passed, zero failed, three ignored**, including 162 passing `bb_game` library tests and the default isolation harness, crate-graph, recording and determinism checks.
- Fresh CLI build without default features; both requested 300-game branch standard seed sets; fresh branch/main coverage traces over literal seeds 0–49 and analysis of their intervals.
- Fresh archived scratch revisions for the widened isolation check, failing holder probe and old-code negative control. Initial scratch test compilation needed the existing roster golden file made readable; reported results are subsequent completed runs.
- Recovery survey and public-path/cache probes pass. Four repeated and four cache-bypass digests match.
- Fresh paired replay over 50 games; separate replay of the old 2/140 and 4/280 inputs; first-base-gap facts and movement traces; additional digest-neutral confidence diagnostic on 4/280. Main's detailed trace uses a constant placeholder for the branch-only `call` field.
- Logs, scripts and scratch source are in `sim/target/codex4/`; the full workspace log is `sim/target/codex-fourth-tests.log`. `git diff --check` was clean before this report. This report is the only tracked change.

## Resolution

Open.
