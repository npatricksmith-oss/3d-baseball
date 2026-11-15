P1 found; do not merge.

## Reviewed

Independent review of `b532a67..0c66ad6` on `work/runner-accel`, with `CLAUDE.md` read first. Reviewed the owner's Q1, agent-models §5.7 runners, §6.2 and §6.4, the acceleration calibration note and ledger row, the tag-up review's P2-3, and the complete diff. Traced `running.rs`, `live.rs` (including `live/holder.rs`), `play.rs::RunnerIn`, game inputs, changed tests/examples, and the receiving rules-engine code.

## Findings

### P1-1 — The new zero crossing creates a false safe zone against tags

**Locations:** `sim/crates/bb_game/src/live.rs:4992` (new initial coordinate), `live.rs:4613–4615` (tag gate).

The tag gate still treats every coordinate within 0.3 m of a multiple of `BASE_L`, including zero, as being on a base. A right-handed batter now reaches that interval while running out of his box. This coordinate zero is neither a base he is entitled to nor the world position of home plate.

**Probe:** With the default right-handed box, speed 8.2, τ 0.76, and a catcher holding a grounded ball on the plate, at 0.533333 s after contact the batter has `s = 0.003623152`, world position `(-0.492910, 0.605873)`, and distance **0.781052 m** from the catcher. The catcher's 0.25 s tag preparation has elapsed and his tag reach is 1 m. `holder_act()` emits no tag and leaves the batter alive. Delivering `TagContact` in that same state immediately puts him out. The probe asserts both results. The existing near-zero exemption predates this branch, but the shifted coordinate moves it farther into the batter's run and makes this newly crossed zero another false base.

**Suggested fix:** Exclude the batter's pre-first segment from base protection. Determine entitlement/contact at real bases rather than applying the nearest-coordinate test to zero. Add a near-plate tag regression through the actual holder step, for both boxes and during the start delay.

### P2-1 — Crossing zero emits a fictitious home touch and freezes rules samples

**Locations:** `sim/crates/bb_game/src/live.rs:4452–4470`, `sim/crates/bb_rules/src/engine.rs:998`, `engine.rs:121–124`.

The crossing loop includes `b = 0`. The negative right-handed start therefore emits `BaseTouched { base: 0 }` on the way to first, despite the batter never touching the plate. The rules engine accepts this as `on_base = Some(0)`. No corresponding `BaseLeft(0)` follows: the movement code explicitly excludes zero from departures. Subsequent position samples are consequently replaced with progress zero until first is touched.

**Probe:** The P1 probe records the spurious touch at absolute `t = 0.916666667`. At `t = 0.933333333`, the rules runner has `on_base = Some(0)` and sample progress `0.0` despite his off-plate world position. This produces no rules warning. Ordinary first-base forces still work because `forced_to` remains first; this finding does not claim blanket immunity from rules-engine tags.

**Suggested fix:** Emit runner base crossings only for actual destinations 1 through 4; zero is the batter's initial indexing origin. Assert that his first base-touch fact is first base, and that his pre-first samples continue to reflect his geometric position without acquiring base entitlement.

### P2-2 — The claimed source for the speed adjustment is absent

**Locations:** `sim/crates/bb_agents/src/running.rs:9–12`, `docs/calibration/2026-11-15-runner-acceleration.md:20`, `docs/agent-models.md:544`.

The new explanation attributes a 27.62 ft/s sample mean and a −0.115 s-per-ft/s slope to the cited Statcast/FanGraphs sample. Neither number appears in the cited local tag-ups note §1 or the [original Olszewski analysis](https://community.fangraphs.com/are-third-base-coaches-too-hesitant-in-sacrifice-fly-situations/). The original describes a peak home-to-first split proxy, rather than measured catch-to-home times. Its published bins do yield 4.030071 s by weighted arithmetic, but do not supply the asserted conversion to a 27.0 ft/s runner. The background review's assertion is not independent source evidence.

**Suggested fix:** Supply the underlying data and calculation for those two numbers, or describe 4.10 s as the owner's chosen calibration target and mark the claimed empirical adjustment unverified. Keep the distinction between the proxy and a measured tag-up clock. The implemented τ is mathematically appropriate for the authorized target: the exact fit at 8.2 m/s is **0.758028 s**, so 0.76 s gives about 4.1019 s. This is a provenance issue, not evidence that Q1's chosen target was implemented incorrectly.

### P2-3 — Fitting the combined mean leaves a material handedness error

**Locations:** `sim/crates/bb_game/src/live.rs:84–89`, `live.rs:951–953`, `docs/calibration/2026-11-15-runner-acceleration.md:26–30`.

Independent calculation and the geometry probe reproduce **4.368844 s right-handed** and **4.197739 s left-handed**, a **0.171105 s** difference. A common start delay cannot change this difference. The note acknowledges the residual, but population differences do not fully defend it: the [cited home-to-first study](https://fantasy.fangraphs.com/minors-to-the-majors-home-to-first-time-analysis/) also compares the same switch hitters from both sides, reporting a 0.09 s mean advantage and 0.06 s median advantage (31 players). That is a more relevant control than the aggregate 0.04 s difference.

**Suggested fix:** Validate the stance/path and first-step model against paired handedness measurements, or explicitly retain this as an accepted calibration limitation before declaring the handedness fit complete. Fit both sides, not only their combined mean; do not introduce an unsupported hand-specific delay. `BATTER_START = 0.20` is a disclosed fit of a named physical parameter to one mean, rather than an outcome roll, but it is not independent evidence for either hand's timing or the modeled gap.

## Other checks

- Negative `floor() as u8` casts saturate to zero in Rust. Occupancy callers ask about bases 1–3; no wraparound occupancy was found. Forces use starting-base chains, and the batter's forced-first and `no_way_back` guards prevent the retreat/nearest-base logic from sending him home before first. Out runners are skipped by movement. His initial world position remains his box until the 0.20 s start.
- `runner_pos` handles the negative segment before calling the clamped ordinary path. Both routes are continuous at 4 m: a 1 nm coordinate change gives a 1 nm position change. Summed world distance agrees with charged distance within 0.000003 m: **27.978367260 m RH**, **26.581830501 m LH**. Existing first-base overrun tests pass; that phase still starts at the bag and uses its separate overrun distance.
- No missed league-τ use was found in live runner execution or self-judgment. Ordinary acceleration, retreat, relooks, return from an overrun and the scoring ETA use `r.tau`; a tag start uses `r.tau * tau_gain`. Holder races and double-play continuation use `runner_eta` with league speed/τ and visible motion. Changing private τ/top speed while keeping visible position/speed fixed leaves holder ETA estimates identical. Custom tag-start probes at τ 0.6 and 1.2, gain 0.8, verify the first velocity increment and arrival within two ticks of the independent continuous times.
- The new box/τ test has independent numerical expectations. Its 0.11 s tolerance follows the 0.1 s recorded-track sampling, though it does not test absolute arrival times or either zero-boundary defect. Edited tag-up tests preserve their old effective τ 0.6 case and tolerances. The seed-23 test legitimately drops the completed-double-play expectation while retaining the lead force, full-effort feed and onward throw; it could additionally assert the documented batter survival. No unexplained tolerance loosening was found.

## Verification

All cargo builds/tests/simulation runs were executed from `sim/`, prefixed with `taskset -c 0-7 nice`, with `CARGO_BUILD_JOBS=2`, the requested Cargo/local-bin PATH, and both Zig caches under `sim/target/`. Heavy commands ran sequentially. Optional Rerun support was disabled; the initial default-feature compilation was stopped before tests ran.

- `cargo test --workspace --no-default-features`: **419 passed, 0 failed, 4 ignored**. Log: `sim/target/runner-review-tests-minimal.log`.
- `cargo test --offline --manifest-path target/runner-review-probe/Cargo.toml --target-dir target --lib review_probe_ -- --nocapture`: **4 passed**. The probe crate copies HEAD's game modules under `sim/target/` and adds review-only tests; repository source/tests were not edited. Evidence: `sim/target/runner-review-probes.log`.
- Same build, seed 7 twice: matching digest `234b098a87ee7a41d5bbf546ee3e6fd81abc40b1a1c10eccd2dafd92fa658e8d`; no warnings.
- `cargo run --release --no-default-features -p bb_cli -- games --n 300 --seed 0`: **3.85 runs/team-game**, K **22.3%**, BB **8.2%**, HR/PA **2.72%**, reached/BIP **35.3%**, GB/LD/FB/PU hits **.310/.667/.116/.011**, PA/team-game **38.0**, pitches/game **238**, **rules warnings `{}`**. All reported post-change ledger metrics match at printed precision. Log: `sim/target/runner-review-games-0.log`. The bracketed `--seed 1000` set and the base commit were not rerun in this bounded review.

## Resolution

Resolved 2026-11-15 by the author (lead, Claude Fable 5.2), one test pass and one standard run after the fixes; no second review (owner: do not overtest).

- **P1-1: fixed.** The tag gate counts a runner as on a base only at first, second or third; coordinate zero protects nobody. Test `the_batter_passing_coordinate_zero_is_on_no_base` is the review's probe run through `holder_act`: the catcher on the plate tags the batter as he passes zero.
- **P2-1: fixed.** Base crossings are recorded for bases one to home only; the same test asserts no `BaseTouched` on the way out of the box.
- **P2-2: accepted, documents corrected.** The 27.62 ft/s and −0.115 s per ft/s figures came from the earlier tag-up review and are in neither the tag-ups note nor the source. The code comment, agent-models §5.7 and the calibration note §2 now call 4.10 s the owner's Q1 target, say what is and is not verified, and note that the published 4.03 s is a proxy. τ is unchanged (0.76 s is the fit to that target).
- **P2-3: deferred, recorded.** The 0.17 s left/right gap against the switch hitters' 0.09 s is kept as a known limitation (note §3, with the reviewer's source). No per-hand delay without a source; the missing piece is the first steps out of the box, which belongs with batter animation and ratings. Listed in TASKS.md.
- Not taken: asserting the batter's survival in the seed-23 test, and absolute arrival times in the box test (small).

After the fixes: 420 pass, 0 fail, 4 ignored. Standard run unchanged at printed precision on both seed sets (3.85 [3.99]); no rules warnings.
