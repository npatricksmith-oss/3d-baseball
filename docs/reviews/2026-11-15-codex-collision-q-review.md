No P1; all three changes are sound to merge as physics/mechanism work, with g = 0.6 remaining a provisional [E] estimate and the runs/home-run acceptance gate left to the owner.

> Superseded figures: current numbers are in the [metrics ledger](../metrics-ledger.md) (§1).

## Reviewed

Reviewed `4fe04d7..5a8173c`: `c6acc53`, `ff7a227`, `e66f55d`, and `5a8173c`. Read `CLAUDE.md` first, the three requested calibration notes, physics-spec §8.2/A8, agent-models §3.6, and PLAN D19/D20. Inspected the complete diff, the decision/execution path, collision/oracle tests, diagnostic definitions, and wall handling. Checkout HEAD is `3ad6ba6`; its only addition beyond the requested range is the review prompt, so its simulation source is identical to `5a8173c`.

**Partial reach is a physical agent mechanism, not an exit-speed fudge.** It changes hand radius and therefore the actual station where a ball meets a finite bat; collision efficiency, effective mass, misses, and speed at that station follow from the existing models. The gain is named/configurable, introduces no outcome roll or new randomness, and is fitted to measured swing/contact behavior rather than runs or HR. Calibration-targets §2.3 permits agent [E] changes when mechanism evidence justifies them. The numerical value is not measured or uniquely identified by these data, and should retain its stated [E]/Proposed status.

I independently recomputed the twelve cached dates, without fetching Savant data. For 368 batters with at least 20 competitive tracked swings, outside-to-inside location bins reproduce:

| Pitch location | Fair-ball squared-up ratio | Share ≥ 0.8 | Swing length, ft |
|---|---:|---:|---:|
| >10 in outside | 0.770 | 43% | 7.654 |
| 4–10 in outside | 0.823 | 63% | 7.364 |
| Middle ±4 in | 0.847 | 72% | 7.247 |
| 4–10 in inside | 0.801 | 56% | 7.174 |
| >10 in inside | 0.699 | 27% | 7.247 |

The lateral-intercept and depth tables also reproduce. These support location-dependent poor contact and less hand adjustment than perfect sweet-spot coverage. They do not directly measure g, nor establish an inside/outside hand-adjustment asymmetry. The existing asymmetric collision response already makes equally displaced inboard contact more costly in an on-time cage swing.

**The decision boundary is intact.** `batter::decide` receives perceived hypotheses and an own-noise model, not `ExecutedSwing`, its placement draw, or world truth. `pa.rs` plans from the batter's estimate and sends motor execution to world resolution. Ignoring the deterministic coverage shortfall is an approximate self-model, not privileged information access; an eventual coverage-aware decision model should use intended geometry/attributes and uncertain beliefs, never the executed swing. Partial reach itself is deterministic geometry, so placing it in the planner does not move execution noise out of `motor`.

**Full-reach compatibility passes when isolated correctly.** I built unmodified `ff7a227` in an ignored target directory. Its full-game digests match the current implementation with g = 1 and the same old `f_taper = 0.55`, on seeds 0, 4, 7, and 1000. Merely setting g = 1 while retaining the new falloff is not a rollback of both changes and is not expected to reproduce those games.

**The inboard falloff is a defensible sourced surrogate.** The paper's §III C gives the quoted approximately 25% exit-speed loss at 60 cm and near-rigid behavior around 68–74 cm. The test compares flexible/rigid exit speeds at the same station, rather than treating a 25% exit-speed loss as a 25% COR loss. Its target comes from the paper, independently of Rust output. My separate Python calculation using oracle mass properties gives 0.74873 of rigid with `f_taper = 0.27`, versus 0.84371 with 0.55; the new profile gives 0.9815/0.9945 at ±1.2 in from its peak. Thus 0.27 follows approximately under the stated default-bat surrogate assumptions; it is not a coefficient Nathan directly measured or a uniquely precise fit. The paper used a different bat and rigid COR, which limits precision. [Nathan 2000, author-uploaded full text](https://www.researchgate.net/publication/228894251_Dynamics_of_the_Baseball-Bat_Collision).

The cage anchors, A10a–c, and A14 pass. This patch changes neither barrel geometry nor the normal-speed argument of the tangential restitution law. A14 deliberately holds normal COR fixed, so it verifies that plumbing rather than validating the new falloff. More severe jam losses are intended; the handle floor remains 0.35. There is no implemented bat-fracture model here to validate or regress.

**q and its oracle check pass.** Defaults give q = 0.215146 at the reference speed and 0.213876 at 85 + 75 mph closing speed. The lab cross-check gives e_n = 0.477999. The source really reports 0.490/0.494 for the two wood bats and 0.489 with SD 0.009 over 54 impacts; ±0.02 is a model validation allowance, not the measured SD. The game-speed central value remains a sourced estimate with an unmeasured speed slope. [Nathan et al., Table I and §III C](https://arxiv.org/pdf/1009.2549).

Regenerating A8 in memory with `gen_golden.collision_inputs()` and `bbphys.collision()` exactly reproduces the committed fixture and 109.4 mph. Rust agrees with the fixture at the existing 1e-9 relative vector tolerance. The fixture supplies the analytic target's derived restitution; the separate Rust default-parameter test checks the actual rounded/speed-dependent defaults. These are complementary checks, not a claim that the default production case equals 109.4 mph to oracle tolerance. The swing fixture also regenerates exactly from `swing_oracle.golden_cases()`.

## Findings

### P1

None in the reviewed changes.

### P2-1 — The location/O-Contact residual is real; its exclusive attribution to timing is not established

**Locations:** `docs/calibration/2026-11-15-collision-q-and-poor-contact.md:97`, `sim/crates/bb_agents/src/swing.rs:398`.

The reproduced lateral table has outside >6 in ratio 0.746 / 34% squared up, against 0.795 / 52% real; slightly inside −6..−3 in is 0.865 / 81%, against 0.834 / 68%. The exact 300-game seed-0 discipline check gives O-Contact 39.720%, below the 45–68 band, with Contact 65.320%, Z-Contact 72.973%, and SwStr 16.260%. Aggregate exit-speed agreement does not close these mechanism gaps.

Timing is a supported contributor. In the perfect-perception, noise-free cage at x = −0.20 m (inside), advancing the swing 20 ms moves contact from −3.25 in to +0.70 in relative to the sweet spot and raises the ratio from 0.842 to 0.960. At x = +0.20 m (outside), it moves contact from +2.79 to +4.60 in and lowers the ratio from 0.880 to 0.712. This demonstrates the claimed direction of the effect without introducing asymmetric gains.

It does not prove timing is the entire game residual. In a separate 30-game probe, fair competitive contacts within ±5 ms of nominal timing still have middle-plate ratio 0.877 versus real 0.847, and outside 4–10 in ratio 0.829; other perception, placement, height, and selection effects remain. Extreme-bin on-time counts are too small to diagnose an asymmetric reach law. A paired 30-game reach comparison lowers O-Contact from 44.65% at g = 1 to 38.49% at g = 0.6, with the new collision profile held fixed. Losing contact past the finite tip is mechanically expected, not evidence of a collision bug, but the resulting calibration deficit should remain open.

**Suggested fix:** retain the disclosed gap and qualify the claim that its cause is exclusively upstream timing. Diagnose timing/height/lateral aim errors jointly by actual pitch location, correct supported perception errors, then recheck contact and swing-length shapes. Only introduce asymmetric reach if a residual remains with independent support. This is a nonblocking mechanism-calibration finding, not grounds to compensate q or the sourced falloff toward game totals.

### P2-2 — Pre-existing wall-reach contracts disagree; the final games merely avoid the witness

**Locations:** `sim/crates/bb_game/src/live.rs:2219`, `sim/crates/bb_game/src/live.rs:1142`, `sim/crates/bb_game/tests/play_area.rs:55`.

The unmodified pre-reach `ff7a227` build reproduces the reported test failure at seed 4, play 191: the dead-ball point is (37.4026, 17.0162, 1.4178), with clearance −0.93053 m. The current implementation with the old contact settings reproduces the same game digest and witness. On final defaults, `play_area` passes.

Capture permits a standing stretch out to `REACH + STRETCH = 1.4 m`, while the planner's over-wall limiter and the test assume a 1.0 m reach from a body 0.3 m inside. The recorded first baseman remains inside the wall; his last body sample is approximately 1.21 m horizontally from that contact point, and no dive occurs. This is within the implemented stretch but outside the planner/test contract. It is not evidence of an impossible six-metre-high catch: the deflection/dead point is 1.42 m high. The original ball crosses the wall earlier at about 11.34 m height.

**Suggested fix for the fielding lead:** decide whether the stretch is allowed over the rail, then make planning, actual capture, and tests use the same geometric limit. If prohibited, enforce the limit at capture, including stretch/dive attempts; if allowed, update planning and the derived test bound together. Preserve this contact state as a targeted regression independent of batter defaults. This conflict predates the branch's production changes and is not a reason to reject the three collision/contact mechanisms.

### P3-1 — Real/sim intercept tables use different centering populations

**Locations:** `analysis/statcast/poor_contact.py:92`, `analysis/statcast/poor_contact.py:117`.

Real intercepts are centered over each batter's competitive tracked swings, including whiffs. Sim intercepts are centered over contacts only, including sub-60 mph swings, grouped by hand. Consequently identical bin labels do not have precisely identical reference points. Re-centering real data over competitive contacts moves some lateral means by about 0.004 and squared-up shares by about two points. It does not remove the much larger outside gap, and does not affect the direct pitch-location evidence above, but weakens precise depth/lateral shape comparisons.

**Suggested fix:** center both samples over competitive contacts, or record simulated whiff intercepts and use competitive swings on both sides. State the centering population explicitly.

### P3-2 — New diagnostic bypasses the deterministic math facade

**Location:** `sim/crates/bb_game/examples/poor_contact.rs:65`.

`cosang.acos()` is a direct transcendental call. It only formats a read-only diagnostic and cannot alter game digests, but bypasses the repository's math rule and can make diagnostic columns platform-dependent.

**Suggested fix:** use `bb_math::m::acos(cosang)`. No new production transcendental calls, hash iteration, RNG purposes, or draw-order changes were found.

## Verification

- `cargo test --workspace --release --no-fail-fast`: **353 passed**, including `play_area`, cage/exit-speed anchors, crate-boundary checks, A8, A10a–c, A14, collision and swing oracle comparisons.
- `python3 -m unittest analysis/oracle/test_oracle.py analysis/oracle/test_swing_oracle.py`: **28 passed** (23 + 5). A8 and swing fixtures independently regenerated in memory, with exact equality; committed fixtures were not rewritten.
- One standard pair, `sim-cli games --n 300 --seed 0` and `--seed 1000`: runs **4.62 / 4.92**, HR/PA **2.51% / 2.79%**, K **22.4% / 22.2%**, BB **8.2% / 8.2%**.
- One `poor_contact` run of 300 games per seed set, pooled through `poor_contact.py`: **31,171 fair balls**, <80 mph **23.4%**, mean EV **89.1 mph**, hard-hit **40.1%**, 105+ **8.3%**, squared-up ≥0.8 / ≥0.9 **66% / 33%**, HR/fair ball **3.96%**. The note's full exit-speed table reproduces. Cached real sample: **8,758 tracked fair balls**; all twelve dates only, no Savant downloads.
- Standalone probes under `sim/target/`: four-seed full-reach equivalence against an unmodified archived build; the same four final seeds run twice with identical digests; controlled cage timing/location sweeps; paired 30-game reach diagnostics; exact 300-game plate discipline; and the legacy `play_area` test, which fails at the reported witness as expected.
- All cargo builds/tests and simulation runs used `taskset -c 0-7 nice`, `CARGO_BUILD_JOBS=2`, the requested PATH/Zig cache settings, and one heavy job at a time. Logs/probes use the `sim/target/collision-q-review-*` prefix. No source, test, spec, ledger, or TASKS file was edited; no commit was made.

## Resolution

No P1. Resolved by the lead (2026-11-15); the owner's separate question, whether the run and home-run levels are "in range" for the merge (Q7), is still open.

- **P2-1 (the location and O-Contact residual; attribution to timing not established): the claim is qualified, the gap stays open.** The note §4 now says timing is a shown contributor, not the whole residual, with the review's cage and ±5 ms figures. Deferred, recorded in TASKS: diagnose timing, height and lateral aim errors jointly by pitch location (the batters' 17 ms early timing on outside pitches is already an open perception item), then recheck contact and swing-length shapes; an asymmetric reach only if a residual remains with independent support. O-Contact 39.7% (band 45–68) is recorded as a known gap this change widened (44.7% at g = 1 in the review's paired probe).
- **P2-2 (wall-reach contracts disagree; older than the branch): deferred to the fielding work, recorded in TASKS.** Capture allows a standing stretch to 1.4 m while the planner's over-wall limit and `play_area`'s bound assume 1.0 m from a body 0.3 m inside. It goes with the wall-braking job (Q5), which touches the same code: one geometric limit for planning, capture and the test, and the seed 4 play 191 contact state (on `ff7a227`'s contact settings) as a regression independent of batter defaults.
- **P3-1 (real and sim intercept tables centred over different populations): deferred.** It moves shares by about two points and does not touch the pitch-location evidence the mechanism rests on. To do when the location shape is next worked (P2-1): centre both over competitive contacts and say so.
- **P3-2 (a direct `acos` in the new diagnostic): fixed** (`bb_math::m::acos`).
- g = 0.6 stays [E], Proposed, as the review says.
