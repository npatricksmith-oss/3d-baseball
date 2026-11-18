P1 found; do not merge.

> Superseded figures: current numbers are in the [metrics ledger](../metrics-ledger.md) (§1).

Reviewed the four commits `dc1ce82..f38bed2`, against `CLAUDE.md`, agent-models §6.4, the metrics ledger, and the owner's 24583/81 report. The actual merge base is `ecd2bf7`; `dc1ce82` adds documentation on main, with identical sim sources. Its documentation-only differences are not branch regressions. No tracked implementation, test, spec, ledger, or TASKS file was changed during this review.

## Findings

### P1-1 — A trailing runner arriving after the look can still erase a runner

**Location:** `sim/crates/bb_game/src/live.rs:3531`, with the unchanged-event early return at `:3491`.

The crowded-base test checks only whether the trailing runner has reached the base **at the decision instant**. A runner already returning does not look again when that trailing runner subsequently arrives. Both can stop on the same base. The rules engine then warns and overwrites one runner in `st.bases`, without recording an out (`bb_rules/src/engine.rs:1143`).

**Probe:** `target/runner-review-probe`, `review_late_crowding`. Start with runners originally on first and second, batter retired, third baseman holding the grounded ball on third. The lead runner is 8 m past second at 7 m/s; the trailing runner is 0.2 m short of second at 8.2 m/s. Send a secured event and step the ordinary runner movement. The trailing runner has not reached second when the lead runner looks, so the lead runner returns. At 7.067 s both have `s = 54.864`, goal second; the trailing runner is waiting. `PlayEndCheck` emits `two_runners_on_base` and retains only the original lead runner at second. The probe's no-warning assertion fails. All setup facts are sent through the rules engine, including leaving the original bases and retiring the batter.

**Suggested fix:** maintain the return-base traffic constraint while bodies move, including an approaching trailing runner, and give the trailing runner a legal yielding response. A secured/released counter alone cannot enforce it. Add this late-arrival regression and assert that every live runner remains represented unless a rules event retires him or scores him.

### P2-1 — Ignoring throw aerodynamics changes the nearest base and the catchable-height bound

**Location:** `sim/crates/bb_game/src/live.rs:182` and `:207`; the inaccurate error claim is also in `docs/agent-models.md:655`.

The claimed one-or-two-metre error is not a valid bound. Backspin lift matters as well as drag. Using the actual throw solver to construct throws, then supplying only their current position and velocity to the new reader, gives these counterexamples:

- From valid shallow right field `(20, 55)`, a chest-high throw to third at the **default RF arm speed, 40.546528 m/s**, projects to `(-9.114, 28.690)`: 13.860 m short of third, nearest **second**. `throw_passes_base(..., 3)` returns `None`, so that bound does not repair the destination error. A valid-field chest-height sweep found 1 wrong read in 495 samples at positional average arms, and 24 in 510 at 45 m/s. These are release-time errors, exactly when the new relook acts.
- From `(-40, 45)` to home at 40 m/s, the passing-base helper calls third catchable in 0.828 s, but the aerodynamic trajectory passes its closest point to third **4.155 m high**, above the helper's own 3 m ceiling. Nine such false passing-base bounds appeared in the focused valid-field sweep.
- A 40 m/s throw from `(0, 80)` that hops 4.986 m before home reaches home at about 2.71 s, 0.50 m high after one bounce. At release it is read as going to second. Low throws and elevated catches were also probed; the existing tests' three origins at one speed do not establish the claimed bound.

**Evidence:** `runner-review-probes-final.txt` (`review_realistic_projection`, `review_pass_bound`) and `runner-review-hop.txt`, under `sim/target/`.

**Suggested fix:** predict with a runner-owned throw model using observed current state, typical spin, and reference drag/bounce physics; never inspect the actual destination, receiver, or future path. Use that same prediction for catchable passage. Add the default-arm right-field case, the over-third case, and the one-hop case as regressions, and correct the spec's error claim.

### P2-2 — The return race pays for a turn that the runner's legs never make

**Location:** `sim/crates/bb_game/src/live.rs:3514`–`:3515`, and the new goal assignment at `:3541` entering movement at `:3660`.

The new decision estimates the return from rest, but changing `goal` leaves `r.v` intact. Movement flips `direction` immediately and accelerates that speed in the opposite direction. In `review_reverse_motion`, a runner 8 m past second changes from **+7.0 to −7.029 m/s in one 1/60 s tick**, moving 0.117 m backward immediately. His estimated return is 1.590 s; the same movement in the crowded-base fixture completes in 1.067 s. The movement helper predates this branch, but the new routine retreat path inherits its free reversal.

Also, on a later secured/released event while he is already returning, `t_on` still receives the running-start discount solely because `r.v > 1`; `t_back` still starts from rest. Neither uses his direction. Thus the two races do not consistently represent his own current motion.

**Suggested fix:** use signed velocity for both race estimates and a matching stop/reversal rule in movement. Cover a forward-to-return decision and a second look while returning. The disappearance of tagged runners should not be attributed entirely to better reads until this mismatch is measured or explicitly deferred.

## Other review conclusions

**Observation and replay invariants:** the changed throw reader consumes current `p, v`, not `to_base`, `receiver`, aim, or future trajectory samples. The new runner logic reads visible possession/release events and runner bodies. No new `Purpose`, hash iteration, outcome roll, or direct transcendental call was introduced. New decisions increment `decisions` and use their own `2n` timing stream and `2n+1` perception stream; sharing one perception between the two candidate bases is consistent. Later decision indices intentionally change, as the ledger acknowledges. The same-seed, cross-thread digest and full-log test for seed 24583 passes.

**Decision structure:** both distances start at the runner's current position, and forced runners are excluded. An unchanged event counter prevents tick-by-tick re-decisions. Another public event can reverse the goal again; there is no hysteresis, but no demonstrated infinite oscillation. This function always chooses an adjacent base, so it does not itself create a stopping point between bases. P1-1 remains a separate settlement failure.

**Changed tests:** the relocated seed-8 holder snapshot is reached and its isolation assertion passes, alongside the broader isolation tests and negative control. The FFI geometry relaxation has a legitimate trigger: seed 3 play 234 deflects a ball from a glove at `(-48.4102, 35.0775, 2.0246)`, 0.4231 m outside the wall. The pre-existing `batted_path`/`over_the_wall` handling explicitly permits that origin. The location tolerance therefore is not evidence of a new wall defect. These passing tests do not cover P1-1.

**Numbers:** both after standard sets reproduce the ledger: runs **5.12 [5.08]**, reached on BIP **42.4% [42.7%]**, GB hits **.338 [.340]**, PA/team-game **40.8 [40.4]**, pitches/game **255 [252]**, and **zero rules warnings**. The full seed-0 `runner_outs_probe` also reproduces **6,662 hit plays, 0 batter tags, 0 other-runner tags**, and **5 tags / 727 forces on reached-otherwise plays**. I did not repeat the full before standard sets or the bracketed tag-out count. A 100-game baseline probe independently found 139 batter tags and 37 other-runner tags on hit plays, versus 0 and 0 after.

Extra bases remain available. On literal seeds 0–99, average fielders as in the CLI, using the persisted official scorer's singles/doubles and the runner's target-base touches:

| Advance | Before | After | Approximate MLB comparison supplied in the request |
|---|---:|---:|---:|
| First to third or farther on a single | 125/453 = **27.6%** | 156/572 = **27.3%** | 28% |
| Second to home on a single | 94/254 = **37.0%** | 116/309 = **37.5%** | 60% |
| First to home on a double | 40/71 = **56.3%** | 45/66 = **68.2%** | 40% |

These are small, unpaired-opportunity samples after games diverge, not precision calibration estimates. They do not support a general refusal to advance. Concrete after examples: seed 0 play 56 goes first-to-third on a single while the defense returns to second; seed 3 play 20 scores the runner from second on a single while the return also goes to second. Second-to-home is low both before and after; doubles are aggressive. Zero tags merits investigation, particularly P2-2, rather than an assumption that all runners became timid.

**Owner's play:** replaying the actual base-build 24583/81 `LiveInput` and pre-play rules state through the revised simulator preserves the original contact and defense. The batter touches first at **4.706758 s**, overruns it, returns at **10.740091 s**, and finishes safe at first, instead of being tagged at second at 7.940091 s. The current full game's play 81 is a different contact, also ending safe at first. The reconstructed unit fixture is therefore supported by an actual preserved-input replay.

## Verification

- `cargo test --workspace --release`: **385 passed, 4 ignored, zero failures**, including determinism, rules scenarios, FFI, and isolation tests. The initial fresh default-profile build was stopped; the completed result is the release suite.
- `sim-cli games --n 300 --seed 0` and `--seed 1000`: run once each; both report `rules warnings: {}`. `runner_outs_probe 300 0` matches the counts above.
- Temporary copied-source harnesses, baseline source from `dc1ce82`, and all probe outputs are under `sim/target/runner-review-*`. Focused probes reproduce P1-1 with a failing assertion and print the physics/movement counterexamples. Baseline/after advance rates use 100 games each; preserved-input replay and the two advancing examples are recorded separately.
- Every Cargo build/test and sim execution used `taskset -c 0-7 nice`, `CARGO_BUILD_JOBS=2`, and the requested PATH/Zig cache setup, from `sim/`, one heavy job at a time. No commits were made.

## Resolution

All three findings fixed on `work/runner-reads`; one new fault found and fixed on the way. Measured once (ledger row "Runner reads, after the Codex review").

- **P2-2, the return race and the reversal: fixed (`852c44b`).** Movement now brakes a runner whose goal turns behind him at `BRAKE` (6 m/s², the fielders' figure) through zero and accelerates him the other way on his usual curve; a retouched runner who goes on again plants on the bag first. Both race estimates use his signed velocity (`run_time_from`). Tests: `a_runner_who_turns_back_stops_first_and_arrives_when_he_expected` (8 m past second at +7 m/s: no backward step in the first tick, v = 7 − BRAKE·DT after it, a 4.08 m drift on, arrival within three ticks of the estimate) and `a_second_look_while_returning_counts_the_stop`. Three older tests with runners at 12–16 m/s (the retouch and tag-up cases) needed the plant on the retouched bag. As the review suspected, the disappearance of tagged runners was partly the free reversal: with the legs physical the batter tags are 107 [130] in 300 games, against 0 [1] on the first revision and 438 [475] on main.
- **P1-1, two runners on one base: fixed (`2cda65d`).** `return_crowded` is checked every tick while the bodies move (a trailing runner on the base, or unable to stop short of it from his position and speed after `HOLD_READ_S` and then `BRAKE`, or with no way back); a lead runner who cannot return goes on (`runner_traffic`); a trailing runner who can stop gives way and goes back to his own base if it is free (`base_held_ahead` counts a runner visibly coming back to it as holding it). Tests: the review's `review_late_crowding` setup as `a_trailing_runner_arriving_after_the_look_does_not_leave_two_on_a_base`, `a_trailing_runner_gives_way_to_a_lead_runner_coming_back`, and `no_two_runners_end_up_on_one_base_in_real_games` (ten games including seed 1000003226, no `two_runners_on_base`). Both standard sets have no rules warnings.
- **P2-1, the throw read: fixed (`19e6027`).** `Sim::thrown_read` predicts the throw from the observed position and velocity with the reference atmosphere's drag, a typical throw's backspin and one bounce when it comes down short, never the true spin, destination, receiver or future path. The same predicted track bounds "can be caught on the way" (closest pass within 2.5 m, no higher than 3 m). The spec's claim that drag and spin cost a metre or two is withdrawn (§6.4). Regressions: `a_default_arm_throw_from_shallow_right_field_is_read_as_going_to_third`, `a_throw_over_third_on_its_way_home_is_not_catchable_at_third`, `a_throw_that_hops_before_home_is_read_as_going_home`, and a sweep (`a_runner_reads_throws_across_the_outfield_and_arms`: the review's origins, three bases, three arms, four points of the flight, no misread).
- **New: a rundown stand-in (`791c3df`).** With the reversal physical, 91 [104] plays in 300 games ran to the 40 s limit: a runner caught between third and home turned at every throw. A runner now turns round once between two bases (`MAX_TURNS`). The rundown itself is deferred (the fielders must close on the runner; the tags it produces, 310 on fielder's-choice plays against 184 on main, are the stand-in's and not a calibrated rate).
- **Other conclusions:** the three real-game tests the changes moved were repinned or found afresh (seed 1 and seed 8 holder snapshots, seed 17 play 203 in `throw_choices`, seed 0 play 17 in `umpire_calls`, whose runner is now tagged returning to first with the engine's base 2); none hides a defect. Open, deferred: the rundown model; second to home on a single (38% against about 60%); runs 4.85 [4.88] with no knob touched.
