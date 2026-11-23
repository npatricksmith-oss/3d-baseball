# Codex independent review: the double cut

**Not mergeable as is.** P1-1 violates the observation boundary. P2-1 leaves the first baseman without a useful job after the double cut has ended; P2-2 leaves its new throwing behavior insufficiently checked.

Reviewed `ba40063..2366ed4` against AGENTS.md, Q35 (c), agent-models §5.5/§5.7, the worker's note, the metrics ledger and the relay hand-overs brief. The concurrent review file was neither read nor modified. This review changes only this document, is uncommitted, and leaves resolution to the Coordinator.

## Findings

### P1-1 — The first baseman's continuation reads the thrower's private route

**Location:** `sim/crates/bb_game/src/live.rs:5178–5179` (`double_cut_job`).

While a throw is airborne, the first baseman keeps his job only when `Ball::Thrown` says `relay: true`, `to_base: 4`, and `receiver` is a middle infielder. These are saved intended-route fields. The branch neither observes the released ball's motion nor waits for a call. The repository already identifies incoming route information as private in `the_holders_visible_reception_hides_the_incoming_throws_private_route`.

**Verified:** in a fresh HEAD copy, establish the double cut and launch a physically solved throw from the centre fielder to its relay man. Keep the complete physical trajectory, aim, receiver, positions and publications fixed; change only `relay`. `double_cut_job` changes from `Some((0, 14, 0))` to `None`. After `update_relay`, the first baseman is `Relay(4)` in one copy and `Idle` in the other. There is no visible difference that could support that decision, even under the current exact-public-motion convention.

**Suggested fix:** infer continuation from the first baseman's observation of the throw and previously shown assignments, or an appropriately heard call/visible reception. Add an isolation case varying intended-route metadata while preserving observable motion. Describing the intended receiver as a visible fact in the spec does not establish the required observation boundary.

### P2-1 — Once the double cut ends, the first baseman waits through the flight before returning

**Location:** `sim/crates/bb_game/src/live.rs:5225–5227`, with the retirement paths at `5968–5970` and `8242–8250`.

A throw to second retires the first baseman's cutoff job and gives him a braking target. `deep_jobs` returns throughout the airborne throw, and retirement does not give him his next cover job. He consequently follows that stopping target until the next possession instead of beginning his return to an uncovered first base. Leaving first *during* the double cut follows Q35; this finding concerns what happens after that job has ended.

**Verified:** a fresh seed-17 replay reproduces play 103. The throw to second retires him at **6.04696 s**, when he is 6.30 m from first. He targets a stop farther away and receives the first-base job only at the reception, **8.13030 s**, now 11.08 m from first. The throw back to first is at **9.41363 s**, versus 8.60 s in the stored baseline trace; both builds retire the batter. In 242 stored double-cut plays with an idle first baseman at release and a subsequent possession, the median flight until that possession is 2.73 s. This is a systematic job gap, not just the necessary cost of running back or an occasional player mistake.

**Suggested fix:** when he observes his cutoff job ending, have the first baseman take his own next job during the flight, including returning to an uncovered first base. Preserve his double-cut job while the throw actually comes to its relay man. Add a regression for his return during 17/103's flight; the eventual unchanged out does not check that behavior. No change to Q35's coverage during the double cut is needed.

### P2-2 — The constructed test does not test the relay man's route choice or execute the double cut

**Location:** `sim/crates/bb_game/src/live.rs:11380–11395`.

Both route assertions check only `ThrowRace.relay`, the candidate receiver. They never assert `through`, check the selected route, release the ball, or exercise reception. The test directly assigns `Ball::Held` to the middle infielder. It verifies alignment and candidate ordering, but cannot substantiate the note's claim that it checks the relay man's choice with the first baseman behind him. No executed relay occurs in the supplied 120-game trace, so that trace does not fill this gap.

**Verified:** the HEAD diagnostic has `relay == Some(2)` with the first baseman on his spot, yet `through == false`: with the relay man's arm at 40 m/s, direct arrival is 6.6848 s versus 7.3636 s via first. The same candidate assertion also passes with the first baseman 16 m off the line. Those are legitimate diagnostics of what the assertion does, not evidence that these direct choices are wrong. The existing end-to-end relay tests start without a ground touch, so they do not establish the new first-baseman double cut either.

**Suggested fix:** construct independently justified direct and via-first-baseman cases, assert the selected route, and execute the throw/reception with both men present. Include airborne continuation and a first baseman still moving toward the line. Keep the independent geometry assertions already present. Describe the present test as an alignment/candidate test until execution is covered.

### P3-1 — The convention can count a relay man whose cached intercept vetoes the job

**Location:** `sim/crates/bb_game/src/live.rs:5154–5161`, interacting with the existing loose-ball gate at `5062–5064`; the real-game test's conditional premise is at `tests/throw_choices.rs:1141–1142`.

The convention counts the selected middle infielder out unless he is known to be chasing. His own `relay_job`, however, can still return `None` because its cached infield intercept predates the current deep ball position. The new spec's statement that private gates do not hold him on a ball down beyond `RELAY_DEEP_M` is therefore too strong.

**Verified only in a constructed stale-cache state:** at t = 5 s, a ground ball publicly 110 m out, the centre fielder at 100 m, and a runner on second, retain a middle infielder's old infield intercept. `deep_cut_men` names fielder index 3 for home, while `relay_job` returns `None`. After `update_relay` and `deep_jobs`, the first baseman is `Relay(4)`, index 3 is idle, and his partner covers second. The real-game test skips this state because it first requires both cutoff men already to be out. The additional scan of seed 17 through play 103 found no natural occurrence; frequency and game cost are unmeasured.

**Suggested fix:** make the intercept gate reject expired predictions, or refresh the man's own read before using it to veto a publicly established deep-ball assignment. Do not expose that private read to teammates. Add an eligibility test from the public deep-ball situation, rather than conditioning every alignment assertion on both men already having taken the jobs. This may be deferred with its constructed-only limitation recorded.

### P3-2 — The module's one-cutoff description is stale

**Location:** `sim/crates/bb_game/src/live.rs:5–6`.

The module still says the defense uses “at most one cutoff or relay man.” Q35 and the new implementation deliberately use two.

**Verified:** compare the module comment with `double_cut_job` and the passing alignment test. **Suggested fix:** replace it with the current single/double-cut description.

## Verification

All builds and sim runs below used `bin/job.sh` and the isolated target `sim/target/review-probes-codex`. Copies and diagnostic sources remain beneath that directory: `head`, `base`, `no-convention`, and `probes`. Snapshot switches in the final controls used `cargo clean -p bb_game`; the logs show compilation from the intended snapshot. Earlier executions that reused cross-snapshot Cargo artifacts were discarded.

| Check | Result |
|---|---|
| HEAD: `cargo test -p bb_game --lib the_double_cut -- --nocapture` | Passed. |
| HEAD: `cargo test -p bb_game --test throw_choices -- --nocapture` | All 36 passed, including the amended pickup test and the seeds 0–7 coverage test. |
| `ba40063`, new fixture test run separately for 23/83, 97/119, 118/162 | Each fails as claimed: first baseman retires at 4.19 s on 23/83; he is not out with the relay man at 4.20/4.11 s on the other two. |
| `ba40063`, transplanted constructed alignment unit test | Fails: first baseman remains `Cover(1)`, not `Relay(4)`. |
| HEAD with the two cover-table convention lookups disabled | Seeds 0–7 test fails on seed 2/play 77 at 4.52 s: the partner is idle instead of covering second. |
| Baseline fixture provenance test, generated from real `ba40063` game prefixes | All three match contact position/velocity/spin (within 1e-12), contact time, scoreboard/count/bases, batting order, bat yaw, swing lead and pitch x. These are original inputs, not frozen new outcomes. |
| Five HEAD diagnostic unit probes | Establish the metadata leak, the fresh 17/103 return, candidate-versus-choice distinction, constructed intercept veto, and latch lifetime. All executed; their printed observations are described above and below. |
| Fresh HEAD CLI: `cargo build --release -p bb_cli --no-default-features`, then `sim-cli games --n 300 --seed 0` | Statistics match the note's final seed-0 output: runs 4.03; K/BB 22.7/8.0%; GB/LD/FB/PU hits .309/.665/.104/.006; HR/PA 2.70%; reached on BIP 33.6%; throws out of play .103/game; warnings `{}`. 5,234 ms/game/core. |
| `git diff --check ba40063..HEAD` | Passed. |

The negative controls are expected test failures, not build failures. Queue jobs 46, 47 and 48 contain the final control/probe, standard-run and baseline-unit logs respectively. This review's fresh test scope is the focused `bb_game` unit/integration suite above; its fresh distribution rerun is seed 0. Seed 1000 and the full-workspace total were checked against the supplied records, rather than independently rerun here.

### Other questions from the brief

- **Pre-throw observation boundary:** `deep_cut_point`/`deep_cut_men` use public ball motion, pre-pitch alignment, runners' visible motion and already heard publications. `lead` still uses delayed calls; `deep_cut_as_known_by` treats each man's own chase separately. The convention does not need another hearing delay merely to infer a standard assignment from those common inputs. P1-1 is the separate post-release leak.
- **Ruling and geometry:** the first baseman takes the second cutoff job in every field; the third baseman and other middle infielder have the specified cover jobs; catches and the constructed nobody-on case retain their guards. `through_spot` uses the existing named `CUTOFF_M` = 14 m, about 45 ft, with its existing halfway limit for short throws. No new physical parameter or buried [E] value was introduced. The note openly distinguishes assigned cover from actually being back on a bag, and records the first baseman's late arrival on 97/119.
- **Route selection:** the radial `closer` condition excludes a man behind the holder along the direction to the base, but `lined` means a known cutoff job, not actual alignment or readiness. It can select an off-line or moving man. A diagnostic with two known candidates, one 40 m off line, selects that nearer man with an infinite via time instead of the usable farther candidate. This establishes a limit of the selector, not a demonstrated misroute on 97/119; no through throw occurs in the supplied game sample. A normal single with only one known cutoff candidate retains the old candidate selection.
- **Latch and determinism:** `deep_gen` starts at zero for each new `Sim`; `set_free` increments `ball_gen`, invalidating the previous latch. It cannot leak into the next play. Throws themselves do not clear it: it is suspended while thrown or held by an infielder, and can reactivate if the same generation returns to an outfielder. A constructed later reception at 60 m is still deep with the latch and not deep without it; no natural occurrence was established. New state and tie-breaking use fixed-order iteration; no new RNG draw, unordered collection iteration or direct transcendental call was introduced.
- **Counts and measurements:** recomputing the stored trace tables gives 130 runner-on-first and 149 runner-in-scoring-position double cuts, with the reported assigned-cover results. Trades are 97 → 64 with runners in scoring position and 47/66 unchanged for runner-on-first/nobody-on. The latter are the same play sets, not merely equal totals. Adding a first baseman is not counted as a trade; a runner from first can still produce the double cut once he reaches second. The note, ledger and stored standard outputs agree on both seed sets. The unexplained out-of-play fall is 2 [1] throws per 300 games. Keeping that explicitly unexplained is appropriate under AGENTS' Tier B tolerance; it is not evidence of a throwing-accuracy improvement.
- **Test independence:** the alignment/cover expectations come from Q35 and geometry, not the new output. Fixture provenance and the negative controls check the claimed regression cases. The amended pickup test permits only a first baseman already out before pickup. Those are useful independent checks; P2-2 and P3-1 describe the coverage they do not provide.

## Resolution

Resolved 2026-11-23 (Coordinator), fixes by the Opus worker on `work/double-cut` (note §7); no re-review, each fix has a test that fails without it.

- **P1-1: fixed** (`193a941`). The first baseman stays out only while the ball, as he sees it fly, is coming to the man he knows to be the relay man (`throw_comes_to`); no read of `relay`, `to_base` or the receiver. Test `the_first_baseman_reads_the_throw_not_its_stored_route` varies only the stored route. The receiver of a throw is still treated as known to the receiver himself, as on main: not this branch's.
- **P2-1: fixed** (`193a941`): he takes his own job at the release; 241 → 0 idle spells; regression test on 17/103.
- **P2-2: fixed** (`193a941`): `the_double_cut_played_through`, every throw thrown and caught, with a case each way for the relay man's choice.
- **P3-1: deferred**, TASKS §3 (constructed only).
- **P3-2: fixed** (`a85c36a`).
- Coordinator's check on the merged tree: 530 tests pass; standard run 4.04 [4.34] runs per team-game, rules warnings {}.
