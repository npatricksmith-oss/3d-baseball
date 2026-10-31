# Task List

**Last updated:** 2026-10-31
The live to-do list for the project. [PLAN.md](PLAN.md) holds decisions and design. This file holds what is next and what is done. Update it at the end of every work session.

Status: `[ ]` not started · `[~]` in progress · `[x]` done · `[?]` needs an owner decision

---

## Done

- [x] Brainstorming note (2026-10-26)
- [x] Planning Pass 1, Gemini (archived: [docs/archive/PLAN-pass1-gemini.md](docs/archive/PLAN-pass1-gemini.md))
- [x] Planning Pass 2, Claude (archived: [docs/archive/PLAN-pass2-claude.md](docs/archive/PLAN-pass2-claude.md))
- [x] Owner decisions: stack (Rust sim + Unreal 5), universal DH, classic extra innings, shifts allowed, three-batter minimum, injuries later, live management is the primary experience, no RAM upgrade for now
- [x] Merged master plan: [PLAN.md](PLAN.md)
- [x] Physics spec draft v1, with formulas checked against Nathan's published reference case: [docs/physics-spec.md](docs/physics-spec.md)
- [x] P1. Data formats draft v1: [docs/data-formats.md](docs/data-formats.md) (2026-10-31). Open items in its §10
- [x] P2. Agent models draft v1: [docs/agent-models.md](docs/agent-models.md) (2026-10-31). Open items in its §13
- [x] P3. Calibration targets draft v1: [docs/calibration-targets.md](docs/calibration-targets.md) (2026-10-31). Open items in its §13
- [x] P4. Rules spec draft v1: [docs/rules-spec.md](docs/rules-spec.md) (2026-10-31). Open items in its §15
- [x] **M0 Foundations** (2026-10-31, except the Unreal spike M0.8/M0.9): git repo; toolchains (rustup, uv, zig linker shim, toolbx for `flatc`); Cargo workspace with the crate split; `bb_math` (Vec3/Mat3, units, `libm` wrappers, **Philox4x32-10** keyed RNG with KATs); six FlatBuffers schemas incl. the R12 additions; event-log container with BLAKE3 digest chain and determinism tests; `sim-cli`; Rerun output; [CLAUDE.md](CLAUDE.md); crate-graph test enforcing `bb_agents` ↛ `bb_world`
- [x] **M1 Pitch flight** (2026-10-31): RK4 flight (full-state, spin decay in the ODE), Hermite events, atmosphere, spin-axis/tilt conventions, ABS zone, release point, aiming solve, movement metrics, arsenal templates for 10 pitch types (in public ranges, lefty mirror), release execution noise; Tier A A1–A6 and plausibility bands pass; **independent Python oracle written by Codex** agrees to ~3e-8 m; tunneling gate test; `sim-cli pitch/tunnel/arsenal`
- [x] **M2 Bat and contact** (2026-10-31): bat profile with integrated mass properties (A12), §8.1 impulse solver with energy guard (A7–A9, A11, oracle fixtures), COR location profile, contact detection/resolution, analytic swing model (zero-lag template), swing execution noise, `bb_world` pitch resolution, cage harness; M2 gate sweeps pass (early→pull, late→oppo, undercut→backspin fly, overcut→topspin grounder, off-barrel→weak); contact map in [docs/m2-contact-map.md](docs/m2-contact-map.md); `sim-cli cage/cage-map`
- [x] Codex code review of M0–M2 code: 10 findings, all fixed ([docs/reviews/2026-10-31-codex-m1-review.md](docs/reviews/2026-10-31-codex-m1-review.md))

---

## Phase: M3 Batter vs pitcher (current)

Planning specs P1–P4 are drafted (owner review still open). M0 (minus the Unreal spike), M1, and M2 are done.

### Research follow-ups (from the specs' open-item lists)
Not blocking; resolve during M1–M2 at the latest.
- [ ] R1. Get Nathan's current Trajectory Calculator spreadsheet; confirm the drag and lift coefficients against the `pybbda` port. **Before M1 golden tests.**
- [ ] R2. Wood-bat COR versus impact location, and its speed slope. **Before M2.**
- [ ] R3. Spin-versus-undercut data for test A10. **Before M2.**
- [ ] R4. Ball bounce and rolling values for grass, dirt, turf. **Before M5.**
- [ ] R5. Baseball moment-of-inertia factor α.
- [ ] R6. Tune the default bat to ~31–32 oz; add 2–3 stock bat variants. **During M2.**
- [ ] R7. Exact ABS edge rule. Optional; the league is fictional.
- [ ] R8. Confirm Savant `spin_axis`, tilt, and `pfx_x` signs for RHP/LHP four-seamers against a real Savant sample (data-formats §9). **Before M1 golden tests.**
- [ ] R9. Literature pass on human interception and time-to-contact perception to set Vision's angular σ, Weber fraction, and tracking blackout distance (agent-models §3.2–3.3, §10). **Before M3.**
- [ ] R10. Refresh every [R] reference value in calibration-targets against FanGraphs, Savant, and Baseball-Reference for the latest full season; record sources (calibration-targets §13). **Before M3.**
- [ ] R11. Verify the OBR rule numbers and MLB interpretations cited in rules-spec (three-batter minimum at inning boundaries, mound-visit limit, walk-off base credit). **Before M4.**
- [x] R12. Rules-spec §15 item 2 additions are in the M0.5 schemas (`sim/schemas/events.fbs`, `commands.fbs`) (2026-10-31)

---

## Phase: M0 leftovers

- [ ] M0.1b Install Git LFS before any Unreal asset is committed (`.gitattributes` already routes `*.uasset`/`*.umap` to LFS). Needs `sudo dnf install git-lfs` or the toolbx
- [ ] M0.2b Optional: `sudo dnf install gcc glibc-devel` on the host to retire the zig linker shim (see CLAUDE.md)
- [ ] M0.8 **Unreal spike** on this machine (14 GiB RAM): precompiled UE5 for Linux, low-RAM build settings, link the Rust library through the C ABI, play a capsule timeline; record RAM use and compile times against the PLAN §3.3 pass criteria
- [?] M0.9 Owner review: spike result confirms or reopens D6 (Unreal)

---

## Later milestones (detail added when each one starts)

- [ ] **M1** Pitch flight: also build the Python physics oracle (`analysis/oracle/`)
- [ ] **M2** Bat and contact
- [ ] **M3** Batter vs pitcher
- [ ] **M4** Walking-skeleton full game + Unreal client v0. **Owner judges watchability.**
- [ ] **M5** Deepen the field
- [ ] **M6** Live management (primary experience)
- [ ] **M7** League and season
- [ ] **M8** Presentation polish. **Revisit the RAM upgrade before this.**

Exit gates for each are in [PLAN.md §14](PLAN.md).

---

## Parked (decided "later")

- Injuries, physically grounded (D18), after M8
- Seam-shifted wake, weather and wind, human-umpire mode, swing styles, scouting inferred from behavior, online play
