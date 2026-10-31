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

---

## Phase: Planning (current)

All four planning specs (P1–P4) are drafted. Planning closes when the owner has reviewed them; the follow-ups below run alongside M0–M2.

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
- [ ] R12. Apply the rules-spec §15 item 2 additions to data-formats (new events, `LineupIncomplete`, `Award` cause, pitching-change reason). **Before M0.5 schemas.**

---

## Phase: M0 Foundations (next after planning)

- [ ] M0.1 `git init`, Git LFS, `.gitignore`; first commit of the planning docs
- [ ] M0.2 Install toolchains: Rust (rustup, stable), Python with `uv` (oracle and analysis), Rerun viewer
- [ ] M0.3 Cargo workspace with empty crates per PLAN §15, including the `bb_agents` ↛ `bb_world` dependency rule
- [ ] M0.4 `bb_math`: SI vector types, unit conversions, keyed counter-based RNG, `libm` wrappers
- [ ] M0.5 FlatBuffers schemas from P1; event-log writer and reader; determinism hash test
- [ ] M0.6 `sim-cli` skeleton; Rerun hello-world (log a hand-built trajectory)
- [ ] M0.7 `CLAUDE.md`: invariants, units/frames, Unreal rules, testing expectations
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
