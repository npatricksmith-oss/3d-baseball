# Task List

**Last updated:** 2026-10-31 (day session)
The live to-do list for the project. [PLAN.md](PLAN.md) holds decisions and design. This file holds what is next and what is done. Update it at the end of every work session.

Status: `[ ]` not started · `[~]` in progress · `[x]` done · `[?]` needs an owner decision

---

## Resume here (end of the 2026-10-31 day session)

The overnight "Resume here" list is done (commit `3fd7757` plus the docs commit after it). Runners now retouch after catches, wait for the runner ahead to clear a base, and go back from a base the runner ahead holds: `sim-cli games --n 128` gives **0 rules warnings**, 5.64 runs/team-game. The "846 ms per game" was a measuring artifact (wall × 16 SMT threads); one game on one thread is ~270 ms, live plays ~117 ms. The live review's Resolution section is written. Next candidates, in order:

1. [~] **Per-game budget (200 ms), ~210 ms now** (best of six runs; the laptop's timings swing widely with load). Done: two-stage aim solve and seeded scouting builds (−28 ms per game); batter filter with an analytic Jacobian, direct plate covariance, and aim points from the nominal's depth crossing (−61 ms; `sim-cli pa` 641 → 421 µs per pitch with identical B1 rates). Breakdown in agent-models §12. Next: live plays (~125 ms vs 60 ms budget), where fielder predictions are about half (2,746 full-path predictions per game; cut the horizon or reuse across replans).
2. [ ] **Run environment:** ground balls reach base .55 of the time (MLB ≈ .24). Measure by fielder and spray before changing positioning.
3. [ ] The rest of "M4 next steps" below.

---

## Owner decisions waiting (from the 2026-10-31 overnight session)

- [?] **O1. Chase rate / batter overconfidence.** With perception biases modeled honestly, average hitters chase ~11% (MLB 29%) because their filter is never confidently wrong. Should average hitters be overconfident (believed bias σ < true σ, or Discipline `c_D` > 1 at grade 50), or should the learned `SwingW` table (M6) carry it? See [docs/calibration/2026-10-31-m3-interim.md](docs/calibration/2026-10-31-m3-interim.md).
- [?] **O2. Interim knobs.** `K_perception` 2.25, `K_command` 0.8, `K_swingplane` 1.0 (`Knobs::calibrated_m3()`), with 13/20 B1 metrics in band. Accept as the working point or ask for another pass.
- [?] **O3. Spec deviations made overnight** (each written into its spec): full-state RK4 with spin decay inside the ODE (physics-spec §5.1); frictionless-impulse energy-guard fallback (physics-spec §8.1); event-log frame identifiers `BBEH`/`BBEV`/`BBEF` (data-formats §4.1); rolling resistance 0.22/0.28 (`bb_physics::ground`, physics-spec §9.1 still lists 0.09/0.12); the batter's 10-state filter (agent-models §3.3); the pursuit-limit and persistent-bias perception terms; the scoreboard as a public channel for pitch-type learning (agent-models §3.8 [C]).
- [?] M0.9 Owner review: the Unreal spike result confirms or reopens D6 (Unreal). The spike has not been run (it needs an Epic account sign-in).

---

## Done

- [x] Brainstorming note (2026-10-26); Planning Passes 1–2 (archived); owner decisions; merged [PLAN.md](PLAN.md)
- [x] Specs P1–P4 drafted (data formats, agent models, calibration targets, rules) and the physics spec (2026-10-30/31)
- [x] **M0 Foundations** (2026-10-31, except the Unreal spike M0.8/M0.9): git repo; toolchains (rustup, uv, zig linker shim, toolbx for `flatc`); Cargo workspace with the crate split; `bb_math` (Vec3/Mat3, units, `libm` wrappers, **Philox4x32-10** keyed RNG with KATs); six FlatBuffers schemas incl. the R12 additions; event-log container with BLAKE3 digest chain and determinism tests; `sim-cli`; Rerun output; [CLAUDE.md](CLAUDE.md); crate-graph test enforcing `bb_agents` ↛ `bb_world`
- [x] **M1 Pitch flight** (2026-10-31): RK4 flight, Hermite events, atmosphere, spin-axis/tilt conventions, ABS zone, release point, seeded aiming solve with explicit failure, movement metrics, arsenal templates for 10 pitch types (in public ranges, lefty mirror), release execution noise; Tier A A1–A6 and plausibility bands pass; **independent Python oracle written by Codex** agrees to ~3e-8 m; tunneling gate test; `sim-cli pitch/tunnel/arsenal`
- [x] **M2 Bat and contact** (2026-10-31): bat profile with integrated mass properties (A12), §8.1 impulse solver with energy guard (A7–A9, A11, oracle fixtures), COR location profile, contact detection and resolution (end caps, adaptive scan), analytic swing model (zero-lag template, braking, plane tilt), swing execution noise, `bb_world` pitch resolution, cage harness; M2 gate sweeps pass; contact map in [docs/m2-contact-map.md](docs/m2-contact-map.md); `sim-cli cage/cage-map`
- [x] **M3 Batter vs pitcher, mechanisms** (2026-10-31): 20–80 grade mappings (§10); batter hypothesis bank (10-state EKF per pitch type: plate point, arrival, break deviation, release point, persistent perceptual biases), pursuit-limited angular noise, spin cue, sequence prior; swing/take with Discipline, full-covariance P(strike), own-execution beliefs, anticipated-correction credit; causal mid-swing adjustment (Adjustability); real check swings judged by yaw; bootstrap pitcher selection with count-based target maps; in-game adaptation (`PitcherMemory`) with a times-through-the-order test; archetypes; `sim-cli pa/sensitivity/archetypes`; monotonicity tests; performance pass (~395 µs per pitch)
- [x] **M4 started** (2026-10-31): `bb_rules` pure step function with a TOML scenario corpus (39 scenarios); ground bounces/rolls, fence, home runs, wall caroms; walking-skeleton play simulator (fielders, dives, glove capture, throws, runners, tag-ups); full AI-vs-AI games in `sim-cli game/games`
- [x] Codex reviews: M0–M2 (10 findings, all fixed), M2–M3 (19 findings: 15 fixed, 4 deferred with reasons), in [docs/reviews/](docs/reviews/)

---

## Phase: M3 calibration (open) and M4 (current)

### M3 gate items still open
- [~] B1 plate-appearance rates in rough Tier B bands: 13/20 in band ([report](docs/calibration/2026-10-31-m3-interim.md)). Open: O-Swing% (11 vs 29), foul share (18 vs 50), called-K share (45 vs 25), pitches/PA (3.0), Zone% (56), K% (13), HBP% (no evasion). Blocked on O1 for the chase mechanism
- [x] Velocity anchoring (agent-models §3.1): each type's expected speed anchors 20% toward the speed perceived on the previous pitch; test `bb_game/tests/anchoring.rs` (changeups after fastballs are timed earlier)
- [ ] Composure (leverage-scaled noise, agent-models §1.5): needs game state; wire up in M4 now that games exist
- [ ] Hit-by-pitch evasion and the posed capsule body (agent-models §3.7)
- [ ] SpinCapacity has no effect (the batter's scouting knows this pitcher's true shapes; a league-prior blend would fix it); SpinRecognition is weak; QuickHands flat between 30 and 50. See the report's §7.2 notes
- [ ] `Executed*` visibility isolation (module or crate split, plus a compile-fail test)
- [ ] `criterion` benchmarks for the per-pitch budget (the timing harness is `bb_game/examples/perf.rs`)

### M4 next steps
- [x] Fix the findings of the Codex M4 review ([docs/reviews/2026-10-31-codex-m4-review.md](docs/reviews/2026-10-31-codex-m4-review.md), with Resolution)
- [x] Codex live-simulator review ([docs/reviews/2026-10-31-codex-live-review.md](docs/reviews/2026-10-31-codex-live-review.md)): 8 fixed, 3 partly, 8 deferred to M5, with Resolution
- [x] Event-log output from real games (`bb_game::eventlog`, one `PlayRecord` per pitch) and the **M0 exit test on real games**: same seed ⇒ same digest across threads; the log reads back verified (`bb_game/tests/determinism.rs`)
- [~] Event-sourced box score in `bb_stats`: runs, linescore, PA by result, LOB, outs per half-inning from the log alone, with accounting identities over 80 games (`bb_stats/tests/accounting.rs`). Hits vs errors still need the scorer (rules-spec §10)
- [ ] Rules engine: uncaught third strike and caught foul tips (catcher capture), overrunning first, two runners on a base, passing runners, interference (§5.7); grow the corpus toward 150 scenarios (§12.3)
- [x] Causal live-ball simulator (`bb_game/src/live.rs`): tick loop, fielders predict from noisy observations with their own physics, calls and re-calls, coverage, real captures and deflections, wall robberies, thrown balls with tracking receivers, observation-based runners, rules stepped live (2026-10-31)
- [ ] Live-ball simulator next steps: atlas-based perception (agent-models §5.1), relays and cutoffs, throws with drag, runner leads, rounding and slides, `BallPassedBase`, first-touch facts, the deferred live-review findings (3–6, 11–13, 16), runners reconsidering on bobbles and throw releases
- [ ] Run environment: runs/team-game is ~5.6 with the causal simulator (target ~4.5). Ground balls reach base .55 of the time (MLB ≈ .24) because too many get through the fixed-table infield positions; LD/FB/PU are near MLB rates. Measure first: by batted-ball type and fielder (`examples/dump_plays.rs` plus `analysis/plots/plays_topdown.py` give visual checks)
- [x] Play timelines for the client (data-formats §5, `bb_game::timeline`: ball knots, markers, telemetry, view state) and Rerun output of whole games (`sim-cli game --rrd`)
- [x] C ABI for the Unreal spike (`bb_ffi`: create/advance/score/free, header `sim/crates/bb_ffi/include/bb_ffi.h`); a C client plays a full game (`sim/crates/bb_ffi/c/run_smoke.sh`)
- [ ] M0.8 Unreal spike, then the Unreal client v0 (capsules, broadcast camera, scoreboard). **Owner judges watchability**

### Research follow-ups (from the specs' open-item lists)
- [ ] R1. Nathan's current Trajectory Calculator spreadsheet; confirm C_D/C_L against the `pybbda` port
- [ ] R2. Wood-bat COR versus impact location, and its speed slope
- [ ] R3. Spin-versus-undercut data for test A10. The sim gives ~4,300 rpm at a 2 cm undercut, which may be high
- [ ] R4. Ball bounce and rolling values for grass, dirt, turf (rolling resistance was raised to 0.22/0.28 so grounders stop in ~110 m; needs real roll-out data)
- [ ] R5. Baseball moment-of-inertia factor α
- [ ] R6. Tune the default bat to ~31–32 oz (it is 34.0 oz, outside its own §6.2 band, as the oracle noted); add 2–3 stock bats
- [ ] R7. Exact ABS edge rule (optional)
- [~] R8. Savant's CSV documentation (checked 2026-10-31) confirms `spin_axis` 180 = pure-backspin fastball and that `pfx_x`, `release_pos_x`, `plate_x` are from the catcher's perspective; also, from 2026 `plate_x` is at mid-plate (data-formats §9.4 updated). Still to do: confirm numeric signs on a real Savant sample. Also settle which Savant movement window matches the sim's full-flight IVB (the arsenal templates assume full-flight ≈ Savant scale)
- [ ] R9. Literature pass on interception and time-to-contact perception (σ_ang, Weber fraction, blackout, the 1.2 rad/s pursuit limit, the 10% persistent share)
- [ ] R10. Refresh every [R] reference value in calibration-targets against the latest full season
- [ ] R11. Verify the OBR rule numbers and MLB interpretations cited in rules-spec
- [x] R12. Rules-spec §15 item 2 additions are in the M0.5 schemas (2026-10-31)

### M0 leftovers
- [ ] M0.1b Install Git LFS before any Unreal asset is committed (`.gitattributes` already routes `*.uasset`/`*.umap` to LFS). Needs `sudo dnf install git-lfs` or the toolbx
- [ ] M0.2b Optional: `sudo dnf install gcc glibc-devel` on the host to retire the zig linker shim (see CLAUDE.md)
- [ ] M0.8 **Unreal spike** on this machine (14 GiB RAM): precompiled UE5 for Linux, low-RAM build settings, link the Rust library through the C ABI, play a capsule timeline; record RAM use and compile times against the PLAN §3.3 pass criteria

---

## Later milestones (detail added when each one starts)

- [ ] **M5** Deepen the field
- [ ] **M6** Live management (primary experience)
- [ ] **M7** League and season
- [ ] **M8** Presentation polish. **Revisit the RAM upgrade before this.**

Exit gates for each are in [PLAN.md §14](PLAN.md).

---

## Parked (decided "later")

- Injuries, physically grounded (D18), after M8
- Seam-shifted wake, weather and wind, human-umpire mode, swing styles, scouting inferred from behavior, online play
