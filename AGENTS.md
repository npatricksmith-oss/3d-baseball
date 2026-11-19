# AGENTS.md

Rules for every agent in this repository: the Claude lead, Claude workers and Codex workers and reviewers. The lead's own rules are in [CLAUDE.md](CLAUDE.md). [PLAN.md](PLAN.md) holds the decisions and design; [TASKS.md](TASKS.md) holds what is next. Where this file and PLAN.md disagree, PLAN.md wins; fix this file.

## What this is

A 3D baseball management game where **every outcome comes from players' physical and mental attributes, never from dice rolls** (PLAN §0). A headless, deterministic Rust sim core resolves each pitch and play; an Unreal Engine 5 client plays back the resulting timelines.

## Invariants (never break these)

1. **No outcome rolls (D1).** The world never consults a probability to decide what happens. Randomness enters only as noise on physical inputs: release angles, swing placement, glove placement, perception samples, and bounce-normal tilt (the *only* world-side randomness, physics-spec §9.3). Agents may hold probabilistic beliefs; the world never reads them.
2. **Agents never see world state.** `bb_agents` must not depend on `bb_world` (PLAN §15). Agents get `bb_observe` types and may run `bb_physics` functions on their own noisy estimates. A test in `bb_game` enforces the crate graph.
3. **Execution noise lives in `bb_agents::motor`.** Its `Executed*` outputs go only to `bb_game`/the world. Decision code never reads them.
4. **Physics is authoritative (D4).** Animation conforms to physical trajectories, never the reverse.
5. **Determinism (PLAN §4.2).** Same build + seed + commands ⇒ same game, bit for bit.
   - All transcendental math goes through `bb_math::m` (the `libm` crate). Never call `f64::sin`, `exp`, `powf`, etc. directly in sim code. `sqrt` is fine (IEEE correctly rounded); `m::sqrt` wraps it anyway.
   - No `mul_add`, no SIMD intrinsics, no fast-math.
   - No `HashMap`/`HashSet` iteration in sim logic. Use `BTreeMap` or index vectors.
   - Randomness only through `bb_math::KeyedRng` with an `RngKey` (data-formats §2.3). **RNG: Philox4x32-10**, key = game seed, counter = `[play_index, agent_id<<16 | purpose, sub_index, block]`. The `Purpose` registry is append-only: never renumber or reuse a value. Within one stream, the draw order is part of the replay contract (see `motor::execute_release`).
6. **Event sourcing (PLAN §4.3).** Every statistic is a pure function of the event log. Nothing increments stat counters during simulation.
7. **Specs before code** for risky subsystems. Implement from `docs/*.md`; tests come from the specs' golden cases. If code must deviate, update the spec in the same change and say why.
8. **Decision log is binding.** Reopening a decided item in PLAN §1 needs the owner.

## Units and frames

- **SI inside the sim:** m, kg, s, rad, rad/s. mph/ft/rpm/oz only at config, UI, and report boundaries (`bb_math::units`). Stored field names carry unit suffixes (`_m`, `_mps`, `_s`, `_rad`, `_radps`).
- **Sim frame (Statcast frame), right-handed:** origin at the rear apex of home plate at ground level; **+Y** toward the pitcher and center field; **+Z** up; **+X** toward first base (catcher's right).
- **Spin** is the angular-velocity vector ω (right-hand rule). Pitch moving −Y with backspin ⇒ ω ∥ −X; batted ball moving +Y with backspin ⇒ ω ∥ +X (physics-spec §1.3, tested).
- **Savant spin axis:** 180° = pure backspin, 0° = topspin; tilt is from the pitcher's perspective (12:00 = backspin, RHP four-seam ≈ 1:00). Derived for display only (data-formats §9).
- **Unreal frame** conversion `ue = 100 × (y, x, z)` lives only in the client plugin.

## Repository layout

Top level: `PLAN.md`, `TASKS.md`, `observations.md` (the owner's checklist), `docs/` (specs, `reviews/`, `calibration/`, `metrics-ledger.md`, `archive/`), `analysis/` (the independent Python physics oracle with golden fixtures, plot scripts), `sim/` (Cargo workspace; `schemas/*.fbs`), `client/` (Unreal Engine 5.8 project, C++ only, no binary assets; docs/unreal-spike.md), `bin/play.sh` (the owner's launcher).

Crates in `sim/crates/`:

```text
bb_math      Vec3, Mat3, units, deterministic math (m), Philox keyed RNG
bb_physics   pure models: flight, atmosphere, spin axis, zone, pitch aim, bat, collision
bb_observe   observation types and shared domain enums (PitchType, Hand)
bb_agents    players/managers (no bb_world!): pitcher, throwing, motor (execution noise)
bb_schema    generated FlatBuffers code, event-log container, BLAKE3 digest chain
bb_world     true world: pitch resolution against the swing
bb_rules     rules engine: pure step function over facts; scenarios/**/*.toml corpus
bb_game      orchestrator: cage, plate appearances (pa.rs), live-ball plays (live.rs; play.rs still holds shared types such as FielderAttrs, RunnerIn, base_pos), games, rosters, event log, timelines
bb_stats     statistics as pure functions of the event log
bb_ffi       C ABI for engine clients (include/bb_ffi.h)
bb_cli       `sim-cli` lab binary
```

## Toolchain on this machine

- Rust via rustup in `~/.cargo/bin` (not on PATH in non-login shells: `export PATH=$HOME/.cargo/bin:$PATH`).
- **No system gcc/glibc-devel and no sudo on the host.** Rust links through `~/.local/bin/zigcc` (zig from `uv tool install ziglang`), configured in `~/.cargo/config.toml`; if a link or `cc`-crate build fails oddly, suspect that shim first. For anything needing real system packages (e.g. `flatc`), use the toolbx container: `toolbox run -c fedora-toolbox-45 <cmd>` (passwordless `sudo dnf` works inside it).
- **`/tmp` is a 7.5 GB RAM-backed tmpfs.** Build in the checkout's gitignored `sim/target` (probes and baseline exports under `sim/target/codex-probes/`), never in a new target dir under `/tmp` or a scratchpad.
- **The laptop crashes when hot.** `sim-cli games` starts 16 threads. Prefix sim runs and full test runs with `taskset -c 0-7 nice`, keep `CARGO_BUILD_JOBS=2`, and never run two measurement runs at once.
- Python: `uv` in `~/.local/bin`. The oracle uses only the stdlib (`python3 -m unittest analysis/oracle/test_oracle.py`). Plot scripts: `uv run analysis/plots/<script>.py`.
- Rerun viewer 0.41.0: `uv tool install rerun-sdk==0.41.0`, then `rerun file.rrd`. The SDK crate is pinned to `=0.41.0` to match.
- Codex CLI (`codex exec`) is available for independent validation/review passes.
- Unreal Engine 5.8.4 (precompiled) belongs at `~/UnrealEngine/5.8.4`, outside the repository. `bin/play.sh` asks before falling back to Rerun when it is missing.
- Build the client: `Engine/Build/BatchFiles/Linux/Build.sh BaseballClientEditor Linux Development -project=<abs path>/client/BaseballClient.uproject` (6 s incremental).
- Check visual changes headlessly: `client/run.sh 7 -RenderOffscreen -BbShots=6`, then read the PNGs in `client/Saved/Shots` (made by the run). Fixed picture sets without a game (`-BbParkShots`, `-BbFigureShots`, `-BbMotionShots`, `-BbSlideShots=<n>`, `-BbDiveShots=<n>`, `-BbCrewShots`) are described in park-visuals §4 and player-visuals §6–§12. Open a saved play with `client/run.sh -BbSavedPlay="<path to .toml>"` (quote the path; it may have spaces). The headless check, which saves and reopens plays through the viewer's own loader and exits 1 on any failure, is `client/run.sh 7 -RenderOffscreen -BbSavedPlayCheck`; `-BbSavedPlayCheckFile="<toml>"` only tries to open one file (exit 1 if the viewer would refuse it or no file is given).

## Commands

```bash
cd sim && cargo test --workspace          # all tests (unit, golden, oracle cross-checks)
cargo run --release -p bb_cli -- arsenal  # pitch-type table for an average pitcher
cargo run --release -p bb_cli -- pitch --type SL --n 20 --rrd out/sl.rrd
cargo run --release -p bb_cli -- tunnel --with CU --json out/t.json
cargo run --release -p bb_cli -- pa --n 3000 --k-perception 2.9 --k-command 0.8 --k-miss 1.4   # plate-appearance rates at the calibrated knobs
cargo run --release -p bb_cli -- sensitivity          # one-attribute sweeps, direction checks
cargo run --release -p bb_cli -- archetypes           # batter/pitcher archetype profiles
cargo run --release -p bb_cli -- game --seed 7        # one full AI-vs-AI game
cargo run --release -p bb_cli -- games --n 200        # run environment over many games
cargo test -p bb_rules --test scenarios -- --nocapture   # rules scenario corpus (TOML)
./schemas/gen.sh                          # after editing any .fbs
```

## Testing expectations

- Every change must be checkable headlessly: unit, golden, property, and snapshot tests, plus CLI distribution reports.
- **Tier A physics** (must match reality) is tested against published cases and the independent oracle (`analysis/oracle/golden/*.json`). Rust ↔ oracle flight agreement is ~3e-8 m; keep the tolerances tight so a real model change shows up.
- **Tier B playability** (runs/game, K%, ...) is tuned only through ability distributions and noise scales, never through fudge factors (PLAN §13). Close is enough (owner, 2026-11-11): a measure within a point or two of its target or its old value is not worth bisecting; check its run-to-run noise first, report it and move on. Before tuning an ability value, look for a planner or movement bug.
- **Players make mistakes, but only ones a real player could make** (owner, 2026-11-12). Judge a behaviour change by "would a real player do this?", not only by its run cost.
- **No tautological tests** (owner, 2026-10-31). A test checks against something independent (a spec golden, the oracle, a physical invariant, an old-versus-new bound); it does not restate the implementation.
- **Stale binaries lie.** `cargo build --example X` leaves `sim-cli` stale, and a worktree's test binary can be older than its source. Before reporting done, `touch` the changed sources and rerun the tests and the standard run.
- Parameters tagged [E] in the specs are named, configurable parameters, never literals buried in logic.
- When you change physics, regenerate or extend the oracle fixtures from the spec, not from the Rust output.
- Calibration changes go in a report under `docs/calibration/` naming the band and mechanism that justified each change (calibration-targets §2.3–§2.4). The calibrated global knobs live in `bb_game::pa::Knobs::calibrated_m3()` (the name is historical; games use it); `Knobs::default()` stays at 1.0.
- Independent reviews go in `docs/reviews/`, each with a Resolution section saying what was fixed or deferred and why. A reviewer edits nothing else.

## Unreal rules (PLAN §3.4)

Game logic in C++; Blueprints only for thin visual wiring. Script asset setup with the editor Python API. Automated screenshot capture for visual checks. The sim runs off the game thread; Unreal never steps gameplay physics. Low-RAM rules (14 GiB): precompiled engine only, capped build parallelism, no starter content.

## Workflow (every agent)

- Work on your own `work/<topic>` branch in your own worktree. Commit early and in small, described steps (WIP commits are fine), so a usage cutoff never loses work. Workers never merge into `main` and never push; the lead does that.
- Update [TASKS.md](TASKS.md) at the end of every work session, following the rules at its top (reviewers excepted: they write only their review file).
- **Stale text misleads the next agent.** When a change makes a comment, a doc line or a figure untrue, fix or delete it in the same commit. Notes under `docs/calibration/`, `docs/reviews/` and `docs/archive/` are dated records: read them for how something was found, and take current figures from the ledger and current tasks from TASKS.md.
- **[observations.md](observations.md) is the owner's checklist (owner, 2026-11-16):** his own words, each with a box (`[x]` done, `[~]` partly, with a one-line plain note on what is left). When work finishes or advances one of his items, tick it there in the same commit. New owner observations (including saved plays' comments) go there too, then into TASKS.md's backlog for the detail. He follows progress there instead of in TASKS.md.
- **Metrics ledger (owner, 2026-11-14):** every task that changes sim code adds a row to [docs/metrics-ledger.md](docs/metrics-ledger.md): the standard run (`sim-cli games --n 300 --seed 0` and `--seed 1000`) on the commit before and the commit after, the cause of any movement, and what is unexplained. Identical digests get one line under "No effect on games". An unintended or unexplained shift also goes in ledger §4 or §5. Client-only, tooling and docs work gets no row.
- Record owner-relevant choices in the relevant spec or PLAN §1 (as "Proposed" if the owner has not ruled).
