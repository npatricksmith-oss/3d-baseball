# Pop-up calls review — 2026-11-13

Reviewed `454d341..bdd7f03` on `fix/popup-calls`, with `CLAUDE.md` read first. No P1 finding.

## P2 — Extra chasers can remove every candidate to cover second

**Location:** `sim/crates/bb_game/src/live.rs:1127–1149`, interacting with `cover_roles_after_chasers` at lines 1427–1443.

**Defect:** The new air-ball branch can make both middle infielders chase. Coverage then considers only those same two fielders for second base. It leaves the base unassigned even when another fielder is available. `assign_relay` can also exhaust that two-man list by combining a relay assignment with a secondary chase; `assign_backups` does not repair coverage.

**Verified scenario:** Game seed **55, play 159**, with a runner on second. At contact +0.250 s, CF, 2B and SS are all tier 1, with respectively **2.9683, 0.4056 and 0.3490 s** to spare. All three receive `Role::Chase`. The final roles, in P/C/1B/2B/3B/SS/LF/CF/RF order, are:

```text
Backup, Cover(4), Cover(1), Chase, Cover(3), Chase, Backup, Chase, Backup
```

Second has no cover until the next recall at +1.067 s. Under the previous call rule on these same reads, CF wins outfield priority, SS covers second, and the air-play guard prevents a premature relay assignment. This is a coverage regression on an air ball, distinct from the existing ground-ball coverage limitations. The ball is eventually caught; this example establishes missing coverage, not an observed extra run. The calibration corpus also produced **17 tier-1 assignment events** without second-base coverage.

**Suggested fix:** Add a fallback covering assignment when both middle infielders are busy, before allocating backups. Preserve active chasers, relay men and existing covers. Add a scenario with both middle infielders chasing and a runner on second, plus the chase/relay combination.

## P2 — Recall can call off an excluded chaser before anyone is sure

**Location:** `sim/crates/bb_game/src/live.rs:1066–1070`, together with the global reset at line 1115 and candidate-only restoration at lines 1127–1134.

**Defect:** Recall includes the primary and four nearby fielders, but does not necessarily include other active chasers. `cover_roles` resets everyone's role, and the new branch restores only tier-1 fielders present in `calls`. Consequently an excluded secondary can be sent to backup while still believing he catches the ball and while nobody has made a sure call. This contradicts the new rule in agent-models §5.7 and the calibration note.

**Verified scenario:** Game seed **897, play 163**. Initially LF and CF are tier 1 and both chase. At +1.067 s the recall candidates are LF, SS, 3B and 2B. LF remains tier 1 with **1.6676 s** to spare; all three infielders are tier 0. A diagnostic fresh read for the omitted CF still gives **tier 1, 0.3215 s** to spare. Nevertheless CF changes from `Chase` to `Backup`. LF does not become sure until the +2.317 s recall. LF eventually catches this ball, so this is a verified premature role change rather than a demonstrated lost catch.

**Suggested fix:** Include every active chaser in recall, in addition to the nearby candidates. Recompute their calls and let the ordinary sure-call rule release them. Test an active secondary outside the nearest four both before and after a sure call.

## P3 — Correct the calibration baseline and identify the diagnostic sample

**Location:** `docs/calibration/2026-11-13-popup-read-and-calls.md:10–31,84`.

**Defect and verified reproduction:** The stated baseline GB and LD hit rates do not reproduce. Using the CLI's exact 1,024 seeds, `5 * 1_000_003 + i`, the parent implementation produces **12,687/30,908 = .410476 GB hits** and **6,970/9,695 = .718927 LD hits**, which print as **.410 and .719**, rather than the documented .411 and .723. The current implementation with `CALL_SURE_SIGMAS = 0` reproduces those same baseline counts.

The §1 read-error table reproduces with **zero sigma**, not with the committed one-sigma gameplay. For example, its first row is `−0.8 / 22.0 / 8.2 / 6.3 / 4.1 / 24.7`; the documented command on the committed code returns `−0.6 / 21.9 / 8.2 / 6.3 / 4.1 / 24.6`. The read function is unchanged, but changed game histories change the sampled contacts.

**Suggested fix:** Correct the baseline GB/LD entries and label the diagnostic table as measured with the baseline call rule, or regenerate it on the committed gameplay. The principal pop-up conclusion survives these corrections.

## Other checks

**Invariants and extraction.** The sure-call decision introduces no new world-truth input. Its distance comes from `pred.at(self.t)`, its movement state from `planning_state`, and its remaining inputs are the fielder's own attributes and elapsed viewing time. The projected pre-reaction movement is his own anticipated movement. Estimated distance makes this an approximate uncertainty estimate, not an oracle for the actual error or catch outcome. `read_prediction` still constructs the noisy observation from truth in the orchestrator; the decision reads the resulting belief. The extraction preserves the arithmetic and draw order. Parent and zero-sigma current builds have identical digests for seeds 1, 2, 3 and 42, and identical aggregate results over the calibration's 1,024 seeds. No new outcome roll, forbidden math operation, RNG stream or agent-to-world dependency was found; the added parameter is named and tagged [E].

**Role lifetime and coordination.** A sure call resets all fielders through `cover_roles`, including secondaries excluded from recall. Thus exclusion does not leave a tier-1 secondary running after someone else becomes sure; the converse failure is finding 2. The chosen primary always gets `Chase`. `cover_roles_after_chasers` protects chasers and relay men, `relay_job` rejects chasers, and backups consume idle fielders. I found no new path where a sure call leaves both former chasers without a chaser, or leaves an excluded secondary permanently chasing. Secondary replans refresh their own targets but do not themselves announce a call.

**Handoffs and flip-flops, measured in games.** In the calibration corpus (actual seeds **5,000,015–5,001,038**), there were **15,207 fair contacts launched at least 25°**. Of these:

| Situation | Plays | Caught |
|---|---:|---:|
| A selected tier-2 infielder had a tier-1 outfielder among that assignment's candidates | 2,091 | 2,021 |
| A recall changed the primary from tier-1 OF to tier-2 IF | 181 | 180 |
| Those handoffs on 25–50° fly balls | 27 | 27 |
| Those handoffs later returned to a tier-3 outfielder | 17 | 17 |

The 181 handoffs are about **0.18/game**, or **1.19%** of those fair air contacts. Tier-driven switches back do happen; they need not indicate unstable same-tier margin comparisons. The one missed handoff, seed **5,000,595 play 78**, also drops in the counterfactual below.

A sure infielder can take a routine fly from an outfielder who could catch it comfortably. Example: seed **5,000,925 play 5**, landing near **(21.6, 80.2) m**. At +1.067 s, RF is tier 1 with 1.3946 s believed spare and 2B is tier 2 with 0.5951 s. 2B takes it and catches it. A diagnostic reach calculation against the true descending path gives RF about **1.82 s** spare; a separate physical replay retaining OF priority catches it with RF. Seven of the 27 fair fly-ball handoffs had at least one second of such modeled true-path spare. This is an effect of the specified tier priority, not an additional invariant violation.

For the counterfactual, the temporary build preferred a tier-1 OF whenever a tier-2 IF would otherwise win, replayed the same live input and initial rules state, and returned only the unmodified result to the game. Across the 2,091 qualifying fair plays, this alternative gained **13** catches but lost **282**; none of the 181 recall handoffs gained a catch. These are controlled play replays, not alternate full-season hit rates. An additional corpus of seeds 5–1,028 had 195 fair OF1→IF2 handoffs, 193 caught, with all 19 subsequent returns to a sure OF caught.

**Tests.** The changed jump assertion still tests its original purpose: equality of time to spare before and at the reaction gate. Suppressing the projected remaining jump in a temporary mutation fails that equality (**3.799 versus 3.025 s**). Allowing either tier 1 or 3 does not weaken this check.

The new tests have geometrically argued expectations, zero perception draws but nonzero uncertainty attributes, and do not compute their expected tiers by repeating the implementation. Restoring the parent's assignment/call functions makes exactly these two tests fail (43 other library tests pass). The popup fixture has substantial margins: 2B spare **1.469 s** versus sure threshold **0.433 s**, RF spare **0.926 s** versus **3.254 s**. The routine CF fixture has **4.930 s** versus **3.458 s**. The gap fixture is less robust: RF has only **0.3297 s** spare, about **30 ms** above the chase threshold. Scaling launch velocity by ±1% preserves its initial two-chaser assignment, but a more comfortable RF margin would make the regression test less sensitive to planner changes. Neither new test covers findings 1 or 2.

**Calibration and traced examples.** The requested current CLI run gives **5.47 runs/team-game**, GB **.411**, LD **.723**, FB **.122**, PU **.018**, with no rules warnings. The parent gives 5.55 runs, FB .134 and PU .047; the baseline corrections are in finding 3. `pu_hits 128` reproduces **28 → 12** uncaught fair pop-ups. The diagnostic reproduces the note's mechanism: position-depth error dominates, with the sample-version differences described above. I did not rerun the two-/three-sigma sweeps, seed-2 calibration or pitcher-roam measurement.

Both traced misses support the note's main reading. Seed **82 play 192** lands at **(42.895, 51.179) m**. RF makes tier-3 calls at +0.250 and +1.067 s, with **2.256 and 2.575 s** believed spare; 2B is initially tier 2 but is called off. RF secures it just after the bounce. Seed **29 play 135** really is a wrong sure CF call: CF is tier 3 at those first two assignments, with **3.608 and 2.417 s** spare. His first target is **(3.27, 105.24) m**, whereas the ball lands at **(−7.259, 62.284) m**. His depth draw is **−1.473 sigma**. SS initially covers second and becomes idle on the next recall; CF subsequently loses certainty and fields the ball after it lands. The motion descriptions are approximate, but the sure calls are directly confirmed by instrumentation, not inferred solely from tracks. The called-off infielder's lack of a backup job is already explicitly deferred in the note.

## Verification

All temporary sources, copies, logs, linker cache and build output were kept under `sim/target/popup-review*`. No source, test, spec or TASKS content was edited in the working tree.

- Read `CLAUDE.md`; inspected the full commit diff, related role/replan/capture code and referenced documentation.
- `cd sim && cargo test --workspace --release`: **143 passed, 0 failed**.
- `cargo run --release -p bb_cli -- games --n 1024 --seed 5`: rates above; no rules warnings.
- `cargo run --release -p bb_game --example read_error -- 48`: completed; compared committed and zero-sigma samples.
- `cargo run --release -p bb_game --example pu_hits -- 128`: **12**; parent temporary build: **28**.
- `cargo run --release -p bb_game --example trace_play -- 82 192` and `-- 29 135`: completed; checked tracks and possession facts against assignment instrumentation.
- Temporary `review_games` probes over both 1,024-game corpora: logged calls, selected tiers, role resets, uncovered bases, omitted chasers and catches; performed independent live-play counterfactual replays. Replayed seeds 55, 897 and 5,000,256 with detailed assignment logging.
- Temporary true-path reach diagnostic over the calibration corpus: seven fair fly-ball handoffs with at least 1 s modeled OF spare; all 27 fair fly-ball handoffs caught.
- Parent versus current zero-sigma `digest` example: **4/4 identical**. Temporary aggregate report using the CLI's seed schedule and classification: identical baseline counts and 5.55 runs over **1,024 games**.
- Parent call/assignment mutation with current tests: **43 passed, exactly the two new tests failed**. Remaining-jump mutation: the changed gate test failed as expected. Temporary fixture probe recorded margins and ±1% launch-velocity checks.
- Initial fresh diagnostic links were rejected because Zig's default cache is read-only in this sandbox. Pointed `ZIG_GLOBAL_CACHE_DIR` to a copied cache under `sim/target/popup-review/zig-cache` and reran successfully. The temporary workspace used a separate Cargo target directory; committed diagnostics were rerun without instrumentation.

## Resolution

Resolved 2026-11-13: one finding fixed, two deferred by the owner.

**P2 #1, both middle infielders chasing leaves second uncovered: deferred (owner, 2026-11-13).** The gap lasts until the next re-call, at most about a second, and the review's example lost nothing. The owner's ruling on both P2 findings: mistakes like these happen in real games, so an occasional one is fine for now; keep them for a later fix. Listed in TASKS.md.

**P2 #2, a re-call sends an active chaser outside its candidate set to backup before anyone is sure: deferred (owner, 2026-11-13)**, same ruling. The fix when it is taken up: every active chaser is a re-call candidate.

**P3, calibration note: fixed.** The baseline ground-ball and line-drive rates now read .410 and .719, and the read-error table is labelled as measured on the old call rule's games, with the committed code's first row given.

Also noted for later, not a finding: the gap-fly test's right fielder has 30 ms of margin over the chase threshold; a more comfortable ball would make the test less sensitive to planner changes.
