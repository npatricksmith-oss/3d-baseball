# Overnight Development Report: 2026-10-31

The owner asked for continuous autonomous development from 00:39 to 05:00, with Claude making the decisions and using Codex CLI (gpt-6.1-sol) for validation. This report says what now exists, how to run it, what was checked, and what needs the owner. The live to-do list is [TASKS.md](../TASKS.md); decisions waiting for you are at its top.

## Summary

The project went from planning documents to a working simulation core:

- **Physics (M1–M2):** pitch flight, bat–ball collisions, batted-ball flight, bounces, rolls, fence. Tier A golden tests pass. An **independent Python oracle written by Codex from the spec alone** agrees with the Rust sim to ~3×10⁻⁸ m on 21 flight cases and to 1e-9 on 18 collision cases.
- **Agents (M3):** pitchers aim and miss physically. Batters perceive each pitch through noisy eye-angle and time-to-arrival observations with a per-pitch-type Kalman filter, decide swing or take from beliefs, commit, adjust mid-swing within limits, check their swing, and learn a pitcher's arsenal through the game. Every attribute grade maps to a physical parameter.
- **Rules (M4 start):** a pure, deterministic rules engine, tested by 40 TOML scenarios with no physics involved.
- **Full games (M4):** AI vs AI, nine innings, pitch by pitch. Live-ball plays run in a **causal** tick simulation: fielders and runners act only on what they have observed, throws fly and must be caught, and the rules engine judges every fact as it happens. Games are deterministic and logged in the FlatBuffers event-log format with a BLAKE3 digest chain.
- **Client API (M0.8 prep):** a C ABI (`bb_ffi`) streams one `PlayTimeline` per pitch; a C program plays a full game through it, so the Unreal spike only needs the editor.

Nothing anywhere rolls dice for an outcome (D1). The only randomness is noise on physical and perceptual inputs, drawn from a keyed counter-based RNG (Philox4x32-10).

## Try it

```bash
cd "sim"
cargo run --release -p bb_cli -- game --seed 7 --pbp          # a full game, pitch by pitch
cargo run --release -p bb_cli -- game --seed 7 --rrd out/g.rrd  # same game as a Rerun recording (ball, fielders, runners)
rerun out/g.rrd                                                 # scrub the game_time timeline
cargo run --release -p bb_game --example dump_plays -- 7 > out/p.json && uv run ../analysis/plots/plays_topdown.py out/p.json out/p.png
./crates/bb_ffi/c/run_smoke.sh                                  # a C client plays a game through the ABI
cargo run --release -p bb_cli -- games --n 200                  # run environment over 200 games
cargo run --release -p bb_cli -- tunnel --with SL               # fastball/slider tunneling
cargo run --release -p bb_cli -- arsenal                        # 10 pitch types for an average RHP
cargo run --release -p bb_cli -- archetypes                     # power vs contact vs patient hitters
cargo run --release -p bb_cli -- sensitivity                    # each attribute's effect, 30 → 70 grade
cargo test --workspace                                          # all tests
```

The Rerun viewer is installed (`~/.local/bin/rerun`, version 0.41.0 to match the SDK).

## What was verified, and how

| Check | Result |
| :--- | :--- |
| Tier A physics goldens (physics-spec §14: A1–A9, A11–A13) | Pass. A1 fly ball 405.7 ft / 5.62 s (spec 406 ± 3 ft, 5.6 ± 0.1 s) |
| Independent oracle (Codex, Python stdlib, written without seeing the Rust) | Flight within 3e-8 m; air density, spin axis, collisions, bat mass properties within 1e-9 |
| Pitch shapes | All 10 types land in public MLB movement ranges; left-handers mirror right-handers exactly |
| M1 gate: fastball and slider share early flight | A slider aimed through the fastball's tunnel point stays within 2–3 in for 30 ft, then separates to 11 in at the plate |
| M2 gate: contact behaviors | Early pulls, late goes the other way, undercut gives backspin fly balls, overcut gives topspin grounders, off-barrel is weak. Map in [m2-contact-map.md](m2-contact-map.md) |
| M3: attribute directions | 9 of 10 attributes move their main stat the physical way (common-random-number sweeps) |
| M3: times through the order | Emerges from in-game adaptation: contact +2.6 pp from the 1st to the 3rd time through; none with adaptation off |
| M3: archetypes | Power, contact, patient, free-swinger, elite, and weak hitters produce distinct, sensible profiles |
| Rules | 55 scenarios, 15 of them written by the Codex reviewer to expose bugs (counts, HBP vs swing, walks with forces, fair/foul incl. the 2026 ground-contact clarification, catches, tag-ups and appeals, forces and reinstated forces, tags, time plays, home runs with pending runs, ground-rule doubles, LOB, game end, extra innings, walk-offs at the winning touch) |
| Determinism (M0 exit gate) | Same seed gives the same game digest across runs and threads; logs read back and verify; recording visuals does not change the game |
| Crate rule (D1 at compile time) | `bb_agents` cannot reach `bb_world` (test) |
| Accounting (event sourcing) | Box scores rebuilt from the event logs alone match the sim over 80 games: runs, linescores, three outs per completed half-inning, PA totals |
| Code review | Four independent Codex reviews (M1, M3, M4 rules + skeleton, causal simulator): 10 + 19 + 24 findings fixed or explicitly deferred with reasons; all in [reviews/](reviews/) |

## Numbers to know

**Plate-appearance calibration (interim, 13 of 20 metrics in band):** K% 13 (target 22.5), BB% 10, Swing% 43, Z-Contact% 84, O-Swing% 11 (target 29), mean exit velocity 89 mph, mean launch angle 7°, hard-hit 38%. Full report: [calibration/2026-10-31-m3-interim.md](calibration/2026-10-31-m3-interim.md).

**Full games:** about **5.6 runs per team-game** (MLB ≈ 4.5). Line drives (.73 vs .68), fly balls (.12 vs .13), and popups (.02 vs .02) fall in at MLB rates. Ground balls are too productive (.58 including fielder's choices vs ≈ .24), mostly because they get through the infield; the infield positions are a fixed table for now. About 230–370 ms per game on one core (budget 200 ms); 128 games take ~6 s on 16 threads, so a 972-game season would take under a minute.

## Decisions for the owner

1. **Chase rate (O1).** Once perception was modeled honestly, average hitters chase far less than real ones (11% vs 29%). The model's batters are never confidently wrong, and real ones often are. The choice: make average hitters somewhat overconfident about what they see, or leave perception calibrated and let the learned swing-value tables (M6) set the threshold. This changes what Vision and Discipline mean in the game.
2. **Interim calibration knobs (O2):** perception noise ×2.25, command ×0.8, swing execution ×1.0.
3. **Spec deviations made overnight (O3)**, each written into its spec: full-state RK4 (spin decay inside the integration); a frictionless fallback in the collision energy guard; distinct event-log frame identifiers; higher rolling resistance; a 10-state batter filter; smooth-pursuit-limited vision; a scoreboard pitch-type channel for in-game learning.
4. **Unreal spike (M0.8):** not run, because it needs your Epic account sign-in.

## Machine setup done overnight

- Rust (rustup), uv, the Rerun viewer, and Codex were already present or installed per user.
- **The host has no C compiler and no passwordless sudo**, so Rust links through a zig-based shim (`~/.local/bin/zigcc`, zig installed with uv). For system packages (e.g. `flatc`) the existing toolbx container `fedora-toolbox-45` is used. To retire the shim: `sudo dnf install gcc glibc-devel`. Also install `git-lfs` before any Unreal assets.
- Git repository initialized on `main`; every step is a separate commit; no remote.

## Known gaps (in TASKS.md)

- Fielding: positions from a fixed table (infield holes), throws without drag, no relays/cutoffs or runner leads, perception through a simplified noisy-state model rather than the atlas of agent-models §5.1.
- Rules not yet covered: uncaught third strike and caught foul tips (need a catcher), interference, overrunning first, substitutions, the official scorer (hits vs errors).
- Batter: velocity anchoring, composure (needs leverage tables), hit-by-pitch evasion; SpinCapacity has no effect yet.
- Research items R1–R11 are unchanged, except R12, which is done.
