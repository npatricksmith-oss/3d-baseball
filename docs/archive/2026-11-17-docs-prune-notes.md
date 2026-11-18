# Documentation prune, step 1 (2026-11-17)

Audited the working docs against the sim and client on the branch base, `9d6bd62`. The counts cover 14 retired items, 38 rewritten items or status blocks, and 33 TASKS entries that gained open work (including additions to existing bullets). The reference pass repaired or retired 89 inline link, path and section references, counted separately from content rewrites.

## Changes by file

- [CLAUDE.md](../../CLAUDE.md) records the existing roster BatSpeed draw in `Population::bat_speed` and fixes the ledger and screenshot references.
- [PLAN.md](../../PLAN.md) records the shipped foundations, pitch flight, contact, PA and spectator milestones, and the field work already in `bb_game::live`. It sends pending work to TASKS and lists the specifications in §19.
- [TASKS.md](../../TASKS.md) closes pruning step 1, moves remaining research and visual follow-ups into the backlog, and narrows the older live-play items to double cuts, deliberate one-hop throws and distinct first-fielder-touch facts.
- [agent-models.md](../agent-models.md) retires the swing-authoring and atlas placeholders. `plan_swing`, `Sim::track_now` and `read_prediction_k` are already in use. It distinguishes current pitch selection and runner defaults from the manager and catcher designs.
- [calibration-targets.md](../calibration-targets.md) points to the current CLI and ledger. Sensitivity intervals, full-league checks and machine-readable reporting remain gate work.
- [data-formats.md](../data-formats.md) records the shipped FFI, instant-replay retention, Philox and spin tests. The snapshot field catalog is in agent-models §9, and implementing the snapshots and durable retention is M6 work.
- [hud.md](../hud.md) replaces the old strip and planned panel list with the shipped draw functions and screenshot flag. Remaining exports and display polish are in TASKS.
- [m2-contact-map.md](../m2-contact-map.md) labels the original sweep as a historical baseline and gives the regeneration command. The table remains the record from `7ff078f`.
- [park-visuals.md](../park-visuals.md) records the boundary export and geometry built by `ABbPark::Build` and moves remaining polish to TASKS. Q13 and Q14 already cover the drawn mound and surface differences.
- [physics-spec.md](../physics-spec.md) moves sourcing and bat-profile work to TASKS. The A10 shape sweep exists in `spin_rises_with_undercut_and_overcut`. The remaining comparison is against sourced measurements across the sweep.
- [player-visuals.md](../player-visuals.md) removes the old mannequin starting point, skinned-mesh fallback and completed staging TODOs. It keeps the built figure, pose and crew records and moves the remaining gaps to TASKS.
- [rules-spec.md](../rules-spec.md) records what the schemas and writer already do, keeps the section and item number cited by the schemas, and moves remaining rules and logging work to TASKS. `Sim::step_overrun` and the rules engine already handle the first-base overrun.
- [unreal-spike.md](../unreal-spike.md) preserves the spike measurements and desktop spectator evidence. It replaces superseded v0 gaps and controls with the current client behavior.

## Needs the lead/owner

- **D1 wording.** `choose_pitch` samples intended pitch type and target. The agent spec treats policy sampling as legitimate, while the literal decision-log wording says randomness enters only as physical or physiological noise. The rules still resolve each outcome from physical facts. The owner can clarify the wording when reopening the log. [Pitch policy](../../sim/crates/bb_game/src/pa.rs) · [Agent model](../agent-models.md#24-pitch-selection)
- **D6 and watchability.** The offscreen spike passed and the spectator build ran on the desktop. Confirm the editor play loop under the hardware gate, formal D6 sign-off and any remaining watchability targets with the owner. [Spike measurements](../unreal-spike.md#measurements) · [Watchability targets](../calibration-targets.md#63-watchability-b3-owner-facing)
- **D4 and park alignment.** The client stands actors and draws ground balls on its visual mound while the physics ground is level. It also paints a more detailed surface than the sim uses. Q13 and Q14 already authorize aligning them with future fielding work. [Park rulings](../park-visuals.md#6-step-1-as-built-2026-11-15)
- **D9 replay retention.** Instant replay uses recorded timelines in memory, and the on-disk saved-play file is a seed/play bookmark. Durable recorded replay data remains to be built. [Director history](../../client/Source/BaseballClient/BbDirector.h) · [Bookmark writer](../../client/Source/BaseballClient/BbSavePlay.cpp)
- **D17 substitution enforcement.** The command schema carries a pitching change, but the spectator game uses one pitcher per team. Three-batter-minimum enforcement is still an M6 implementation requirement. [Game](../../sim/crates/bb_game/src/game.rs) · [Rules design](../rules-spec.md#93-pitching-changes-and-the-three-batter-minimum-d17)
- **D19 and D20 merge wording.** Swing v2 and the tangential restitution ramp are in use. The history includes the collision-q integration at `fe69159`, and the integration report records 3.88/3.92 runs per team-game against 4.39 in MLB. Confirm whether an owner ruling superseded the old hold language and which run/HR gate applies. [Integration report](../calibration/2026-11-15-integration-of-the-held-branches.md) · [Bat-ball parameters](../../sim/crates/bb_physics/src/bat.rs)
- **Research checks.** The code verifies the implemented conventions and tests. R8–R11, direct physics measurements, and the current Trajectory Calculator workbook still require external evidence. The lead can also confirm whether the original 1,000-game accounting gate was recorded on the current lineage. [Research backlog](../../TASKS.md#7-older-open-items-from-2026-10-31-to-11-12-spot-checked-against-the-code-2026-11-17)

PLAN §1 is unchanged. Verification was a source and documentation audit. Desktop play, fresh distribution runs and external research were outside this step.
