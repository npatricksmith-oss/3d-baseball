# CLAUDE.md

Guidance for AI agents working in this repository. Load this every session. [PLAN.md](PLAN.md) holds the decisions and design; [TASKS.md](TASKS.md) holds what is next. Where this file and PLAN.md disagree, PLAN.md wins; fix this file.

## What this is

A 3D baseball management game where **every outcome comes from players' physical and mental attributes, never from dice rolls** (PLAN §0). A headless, deterministic Rust sim core resolves each pitch and play; an Unreal Engine 5 client (later) plays back the resulting timelines.

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

```text
PLAN.md, TASKS.md, CLAUDE.md
docs/                  specs (physics, agents, data formats, calibration, rules) and reviews/; metrics-ledger.md (what each task did to the game's numbers)
analysis/oracle/       independent Python physics oracle (stdlib only) + golden JSON fixtures
analysis/plots/        uv-run plotting scripts (PEP 723 headers)
sim/                   Cargo workspace
  schemas/*.fbs        FlatBuffers schemas; regenerate with sim/schemas/gen.sh
  crates/
    bb_math            Vec3, Mat3, units, deterministic math (m), Philox keyed RNG
    bb_physics         pure models: flight, atmosphere, spin axis, zone, pitch aim, bat, collision
    bb_observe         observation types and shared domain enums (PitchType, Hand)
    bb_agents          players/managers (no bb_world!): pitcher, throwing (set-up and hurry), motor (execution noise)
    bb_schema          generated FlatBuffers code + event-log container and BLAKE3 digest chain
    bb_world           true world: pitch resolution against the swing
    bb_rules           rules engine: pure step function over facts; scenarios/**/*.toml corpus
    bb_game            orchestrator: cage, plate appearances (pa.rs), causal live-ball plays (live.rs; play.rs is the older skeleton kept for diagnostics), games (game.rs), rosters (roster.rs: who the players are and their hands; no abilities yet), event log, timelines
    bb_stats           statistics as pure functions of the event log (box score, accounting identities)
    bb_ffi             C ABI for engine clients (include/bb_ffi.h; c/run_smoke.sh links a C client)
    bb_league          (later milestones)
    bb_cli             `sim-cli` lab binary (Rerun output behind the default `rerun` feature)
client/                Unreal Engine 5.8 project, C++ only, no binary assets (docs/unreal-spike.md); `client/run.sh`
bin/                   owner's desktop launcher: play.sh (Unreal client, or the Rerun viewer when the engine is absent), install_launcher.sh
```

## Toolchain on this machine

- Rust via rustup in `~/.cargo/bin` (not on PATH in non-login shells: `export PATH=$HOME/.cargo/bin:$PATH`).
- **No system gcc/glibc-devel and no sudo on the host.** Rust links through `~/.local/bin/zigcc` (zig from `uv tool install ziglang`), configured in `~/.cargo/config.toml`. For anything needing real system packages (e.g. `flatc`), use the toolbx container: `toolbox run -c fedora-toolbox-45 <cmd>` (passwordless `sudo dnf` works inside it).
- Python: `uv` in `~/.local/bin`. The oracle uses only the stdlib (`python3 -m unittest analysis/oracle/test_oracle.py`). Plot scripts: `uv run analysis/plots/<script>.py`.
- Rerun viewer 0.41.0: `uv tool install rerun-sdk==0.41.0`, then `rerun file.rrd`. The SDK crate is pinned to `=0.41.0` to match.
- Codex CLI (`codex exec`) is available for independent validation/review passes.
- Unreal Engine 5.8.4 (precompiled) belongs at `~/UnrealEngine/5.8.4`, outside the repository. (On 2026-11-12 it was found in the desktop Trash and restored; `bin/play.sh` says so and asks before falling back to Rerun.) Build the client with `Engine/Build/BatchFiles/Linux/Build.sh BaseballClientEditor Linux Development -project=<abs path>/client/BaseballClient.uproject` (6 s incremental). Check visual changes headlessly: `client/run.sh 7 -RenderOffscreen -BbShots=6`, then read the PNGs in `client/Saved/Shots`.

## Commands

```bash
cd sim && cargo test --workspace          # all tests (unit, golden, oracle cross-checks)
cargo run --release -p bb_cli -- arsenal  # pitch-type table for an average pitcher
cargo run --release -p bb_cli -- pitch --type SL --n 20 --rrd out/sl.rrd
cargo run --release -p bb_cli -- tunnel --with CU --json out/t.json
cargo run --release -p bb_cli -- pa --n 3000 --k-perception 2.25 --k-command 0.8   # B1 rates (M3)
cargo run --release -p bb_cli -- sensitivity          # one-attribute sweeps, direction checks
cargo run --release -p bb_cli -- archetypes           # batter/pitcher archetype profiles
cargo run --release -p bb_cli -- game --seed 7        # one full AI-vs-AI game (M4 skeleton)
cargo run --release -p bb_cli -- games --n 200        # run environment over many games
cargo test -p bb_rules --test scenarios -- --nocapture   # rules scenario corpus (TOML)
./schemas/gen.sh                          # after editing any .fbs
```

## Testing expectations

- Every change must be checkable headlessly: unit, golden, property, and snapshot tests, plus CLI distribution reports.
- **Tier A physics** (must match reality) is tested against published cases and the independent oracle (`analysis/oracle/golden/*.json`). Rust ↔ oracle flight agreement is ~3e-8 m; keep the tolerances tight so a real model change shows up.
- **Tier B playability** (runs/game, K%, ...) is tuned only through ability distributions and noise scales, never through fudge factors (PLAN §13).
- Parameters tagged [E] in the specs are named, configurable parameters, never literals buried in logic.
- When you change physics, regenerate or extend the oracle fixtures from the spec, not from the Rust output.
- Calibration changes go in a report under `docs/calibration/` naming the band and mechanism that justified each change (calibration-targets §2.3–§2.4). Interim global knobs live in `bb_game::pa::Knobs::calibrated_m3()`; `Knobs::default()` stays at 1.0.
- Independent reviews by Codex go in `docs/reviews/`, each with a Resolution section saying what was fixed or deferred and why.

## Unreal rules (from M0.8 on; PLAN §3.4)

Game logic in C++; Blueprints only for thin visual wiring. Script asset setup with the editor Python API. Automated screenshot capture for visual checks. The sim runs off the game thread; Unreal never steps gameplay physics. Low-RAM rules (14 GiB): precompiled engine only, capped build parallelism, no starter content.

## Workflow

- Update [TASKS.md](TASKS.md) at the end of every work session.
- **Metrics ledger (owner, 2026-11-14):** every task that changes sim code adds a row to [docs/metrics-ledger.md](docs/metrics-ledger.md): the standard run (`sim-cli games --n 300 --seed 0` and `--seed 1000`) on the commit before and the commit after, the cause of any movement, and what is unexplained. Identical digests get one line under "No effect on games". An unintended or unexplained shift also goes in its §4 or §5. Client-only, tooling and docs work gets no row.
- Commit in small, described steps. The remote is `origin` on GitHub; the session's lead agent may merge tested work into `main` and push it on its own judgment (owner, 2026-11-14), and says what it pushed, **but only when that lead is Claude Fable 5.2 or Claude Opus 5.5 at high effort**. Any other model or effort level asks the owner first. Worker agents do not merge into `main` or push. Never force-push without the owner's OK.
- Record owner-relevant choices you make in the relevant spec or PLAN §1 (as "Proposed" if the owner has not ruled).
- **Standard review protocol (owner, 2026-10-31):** every substantive change set (a milestone step, a risky subsystem, or the end of a work session with code changes) gets an independent review by Codex Sol before it is considered done. Run it in the background and do not poll:
  ```bash
  codex exec -m gpt-6.1-sol -c model_reasoning_effort="high" -s workspace-write -C "<repo root>" - < prompt.md
  ```
  The prompt names the commit range, the files and questions ranked by risk, and the invariants above. It tells Codex to verify claims by running tests or probes, edit no source, test, spec, or TASKS file, and write only `docs/reviews/<date>-codex-<topic>-review.md`, with findings ranked P1–P3, a Verification section, and `## Resolution` set to "Open." Afterwards: commit the review, fix or defer every finding, write the Resolution, and list unresolved findings first in TASKS.md "Resume here".
