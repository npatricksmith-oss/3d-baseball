# Review: the relay man's exchange (`work/relay-exchange`)

Reviewer: Opus (high effort). Scope: `0f9850e..cc7703e` (`6241d1f`, `715295b`, `3b24ae4`, `cc7703e`). Files: `sim/crates/bb_agents/src/throwing.rs`, `sim/crates/bb_game/src/live.rs`, `sim/crates/bb_game/src/live/holder.rs`, agent-models §5.5/§6.3/§6.4, the throw-race note §8 and the ledger row. I wrote none of it.

**Summary.** The code change is correct and small. Every relay the world makes (a throw with `relay: true`, caught by its receiver) now takes `relay_s`. `relayed` is cleared at every securing, so it cannot carry over from an earlier reception. `relayed` and `pivot` are mutually exclusive (a relay is never a pivot), so their order in `set_up_for` does not matter. No agent belief reads `relayed` or the true relay man's grade, except the holder's estimate of his teammate's `Transfer`, as the pivot already does by the spec's rule. No invariant is broken: no outcome roll, no new RNG stream, no world read by `bb_agents`. Tests pass.

Two of the worker's conclusions go further than the data:
- **The headline gain from third reproduces in direction but is smaller on fresh seeds.** At 275–300 ft it is 57% → 65%, not 68% → 91%.
- **The direct throw still decides most sent runners' races at 250–275 ft.** So the remaining tag-up shortfall is not only the runner's caution.

The change also removes almost all relays from the game, which the write-up does not say plainly.

## Findings

### P2-1. The direct throw still decides the race from third at 250–275 ft; the "decision side only" conclusion does not hold

*Where:* the throw-race note §8, lines 218–222 and 253–254 ("The direct throw did not take over these races", "the race from third now matches", "So the limit from third is now the decision side"), and the ledger row's Open column (`docs/metrics-ledger.md:187`).

*What happens:* my rerun of the send-every-tagging-runner counterfactual, on seeds the worker did not use (2000–2199, before and after the change), gives this for runners on third at the catch:

| Catch depth | Before: safe / sent | After: safe / sent | After: ball reached the catcher by a direct throw (safe) | Worker, after (seeds 0–299, 1000–1299) |
|---|---:|---:|---:|---:|
| 250–275 ft | 6 / 21 (29%) | 10 / 23 (43%) | 14 (6) | 17 / 35 (49%); direct 27 (12) |
| 275–300 ft | 13 / 23 (57%) | 13 / 20 (65%) | 5 (1) | 42 / 46 (91%); direct 6 (5) |

- **Pooled, after the change:** 250–275 ft 27 of 58 safe (47%), 275–300 ft 55 of 66 safe (83%).
- **At 250–275 ft, 41 of the 58 sent runners faced a direct throw home, and 23 of those were out.** The throws reached the catcher 3.5–3.8 s after the catch, against the runner's 4.08 s.
- **At 275–300 ft, 5 of the 11 direct throws beat the runner.** In my set four of the four outs there were direct throws.
- **MLB's implied ≥ 42% at 250–275 ft is met. The ≥ 86% at 275–300 ft is borderline**: 83% pooled, 65% on the fresh seeds alone, 20 cases.

The relay was the main fault at 275–300 ft, and the fix is right. But on a sent runner from 250–275 ft the holder does make the direct throw, and it wins more often than not. MLB's send rates imply about 5 in 6 are safe there. So the runner's caution against a direct throw is partly earned. Fitting `caution_z` now would fit it to a race that is still too quick at 250–275 ft.

*Suggested fix (docs only):* restate §8 and the ledger's Open column. The relay fix closed the 275–300 ft gap within sample noise (pooled 83% against ≥ 86%). At 250–275 ft the direct throw (release about 0.85 s after the catch, ball at the plate in 3.5–3.8 s) still beats more than half of the runners sent, against MLB's roughly 5 in 6 safe. The next step there is the direct race: the outfielder's catch-to-release on a throw home, and the tag-up run time. It is not `caution_z` alone. Put the counts per bin and their small samples (20–46 runners) in the note.

### P2-2. Relays almost vanish from the game, and the write-up does not say so

*Where:* `sim/crates/bb_game/src/live/holder.rs:59–66` (the route is chosen by time alone, `through = v < d`). The note §8 line 217 says only that "he throws direct or not at all".

*What happens:*
- The worker's own probe counts infielders who catch an outfielder's throw away from the bags and throw on. Before the change: 48 + 47 = 95 in 600 games. After: 4 + 5 = 9 (`sim/target/codex-probes/relay/{before,after}_*.txt`).
- In my trace of seeds 0–99 after the change, one throw in 100 games went to the relay man (seed 11 play 64, held 0.750 s).
- Outfielders made 658 throws of 75–90 m to a base in those 100 games, and one of them went through the cutoff.

Real outfielders at the wall throw to the cutoff man. One reason is time, and it is close with drag. The others are accuracy and a long throw's bounce, and the cutoff man's chance to redirect to another base or hold. The holder weighs only time, so a sourced (slower) exchange removes the relay almost entirely. This is not a bug in this change, which makes the time right. But the sim now plays a whole class of plays (long throws home or to third from the wall) differently from MLB, and the ledger row does not record it. Outfield assists at home fell 10 → 4, consistent with this.

*Suggested fix:* record "relays 95 → 9 per 600 games" in the ledger row and note §8, and add a backlog line: measure how often the sim's long direct throws bounce or go wide, against how often MLB throws through the cutoff, before deciding whether the route choice should weigh a long throw's accuracy. Changing the holder's route choice is a design call, possibly the owner's (it is not an Official Baseball Rules fix).

### P3-1. A runner reads a base receiver pulled off the bag as a relay man

*Where:* `sim/crates/bb_game/src/live.rs:313–322` (`HolderLook::release_time`: `received_throw` and not within `RECEIVE_REACH` = 1.5 m of a bag ⇒ `relay_s`).

*What happens:*
- In 100 games after the change (seeds 0–99, `BB_THR` trace), 17 receptions were by base receivers (not a relay, not a pivot) who took the throw 1.5–6 m from the nearest bag and then threw. In the same games there was one true relay.
- The world releases those receivers at 0.7 × `Transfer` with the usual hurry: catch-to-release mean 0.59 s, minimum 0.283 s. The runner now reads 0.75 s plus footwork.
- Before this change he read 0.455 s for them, which was his read of a relay too.
- The error changes sign (belief about 0.13 s too quick → about 0.16 s too slow) but not much in size. It is now about 17 times more common than the case the branch was written for.

A real runner would not take a third baseman who stepped 3 m off the bag to glove a throw for a cutoff man in shallow left.

*Suggested fix:* read a relay from what the runner can see of the job. The cutoff man going out is shown (`shown`, agent-models §5.7), or use a reception distance a base receiver does not reach, such as 6 m. Add a test with a receiver 3 m off third and a cutoff man at the relay spot.

### P3-2. The runner's read of a throw in flight to the cutoff man is still on the old relay

*Where:* `sim/crates/bb_game/src/live.rs:352–358` (`ThrownRead::time_to`: the throw is taken to the nearest base, plus `RELAY_READ_S` = 0.6 s plus a typical throw from that bag). Agent-models §5.5 "What the players know" does not list this path.

*What happens:* a throw from the wall to a relay man 55 m out is read as a throw to second (about 17 m nearer home), then 0.6 s and a 39 m throw home. The world gives 0.75 s and a 55 m throw. The runner's estimate is about 0.6 s early, against about 0.2 s before this change. With relays now about one per 100 games, no runner decision fell inside a relay throw in my 100 traced games (`BB_RT`: 0 lines), so the effect on games is negligible today.

*Suggested fix:* list the path in §5.5's "What the players know" as known and deferred. If relays come back (P2-2), give `ThrownRead` the relay man's spot and `RELAY_S_50` when the throw comes down well short of every bag.

### P3-3. Base receivers keep the 0.27 s hurried release this change calls impossible

*Where:* `sim/crates/bb_game/src/live.rs:4344` and `:435` (`RELAY_TRANSFER_FRAC`, unsourced, with the general hurry).

*What happens:* in 100 games, 27 of the 71 receivers at a bag were hurried, with a minimum catch-to-release of 0.283 s. Examples are the shortstop at second throwing home in 0.283 s (seed 2031 play 72 and seed 2070 play 156, `go_after.out`). The spec keeps this ("no source"). Statcast's catcher exchange (league mean 0.73 s, elite 0.64 s, poor 0.85 s; MLB.com "Statcast adds pop time, exchange, arm strength") is a measured catch-to-release, and it is more than twice this floor.

*Suggested fix:* a backlog line in TASKS.md: source a base receiver's catch-and-throw. Catcher exchange is the nearest measured analogue. Use it to replace 0.7 × `Transfer` and its hurry.

### P3-4. Tests restate the constant; no test pins belief to world on a relay

*Where:* `sim/crates/bb_agents/src/throwing.rs:236–243` and `sim/crates/bb_game/src/live.rs:8490–8527`.

- `relay_s(0.65) - 0.75 < 1e-12` and `hold - RELAY_S_50 <= DT` restate the constant. The independent checks are `g70` within 0.03 s of the measured 0.62 s and `hold >= 0.62`. Those are the parts that would catch a regression; the old code fails `hold >= 0.62` (0.27–0.455 s). This is acceptable, but the assertion messages could say which check is the sourced one.
- **Missing:** a "the holder predicts the relay the world makes" test, the twin of `the_holder_predicts_the_pivot_the_world_makes`. It would compare `throw_race().via` with the world's arrival through the relay man, so a future change on one side cannot drift from the other.
- **Missing:** a test of the runner's `HolderLook` read for a reception away from the bags (see P3-1).

*Suggested fix:* add the two tests.

### P3-5. TASKS.md's "first to home on a double is 53%" is stale

*Where:* `TASKS.md:42`.

*What happens:* the repository's own `extra_base_rates` (scorer's ruling, seeds 0–99, this branch) gives 65 of 84 (77.4%). The worker's `zz_relay_probe` gives the identical 65 of 84 on the same seeds, so the two counts agree. The 53–54% dates from the 2026-11-15 runner-reads build (ledger: 54.1%). The current figure is about 77%, against MLB's 38%. The gap is twice what TASKS.md says.

*Suggested fix:* update the line to 77% (seeds 0–99, `extra_base_rates`) and drop the note §8 caveat "may be another build".

## Question by question

1. **Belief and world.** The world, the holder's route estimate (`holder.rs:62`, using `relay_s` of the teammate's `Transfer`), the runners' held-ball reads (`read_throw_time`, `typical_throw_time`) and `CUT_EXTRA_S` all use the new exchange.
   - **Cut and throw:** the sim has no cutoff man who intercepts a throw aimed past him (only the receiver may catch a throw; `try_captures`, `live.rs:4149–4162`), so no cut path exists.
   - **Relay man throwing to another base:** `SetUp::relay` applies whatever the target.
   - **`relayed` left set:** no; it is reset at every securing (`live.rs:4330`).
   - **Pivot precedence:** a relay is never a pivot (`!relay && …`).
   - **Paths still on the old relay:** the in-flight read (P3-2) and the off-bag receiver read (P3-1).
   - **Invariant 2:** no agent reads `relayed`.
2. **Headline.** Reproduced in direction, smaller at 275–300 ft on fresh seeds. The conclusion is only partly right (P2-1).
3. **Sources.**
   - 0.75 s at grade 50 is a fair reading. The search confirms the MLB.com article exists (the Kiermaier–Adames relay, 2019 ALDS Game 4), though I could not read the figure itself.
   - It sits next to Statcast's catcher exchange: mean 0.73 s, elite 0.64 s. A catcher already faces his target, while a relay man turns half round, so 0.75 s for a relay is, if anything, quick. Grade 70 at 0.64 s against Tatis's 0.62 s is consistent.
   - "No early release" is justified as it was for the pivot: the figures are already relays in a race.
   - Worth adding the catcher exchange to §5.5's sources as a measured analogue.
4. **Side effects.** Relays nearly vanish (P2-2). Outfield assists at home 10 → 4 follows from that. Extra-base rates and trailing runners are within noise: singles first to third 31.5 → 31.9%, doubles first to home 76.4 → 77.7%. Runs move −0.03 [0.00]. I found nothing a real relay man would not do. The behaviour no real player would show is an outfielder at the wall bypassing his cutoff man on nearly every long throw (P2-2).
5. **Tests.** Partly independent (P3-4). Missing: a holder-versus-world relay test and a runner read test.
6. **Doubles, first to home.** 77% is right for this build; TASKS.md's 53% is stale (P3-5).

## Verification

- **Tests on the branch itself** (`CARGO_TARGET_DIR=sim/target/review-probes/target-tests`): `cargo test -p bb_game -p bb_agents --lib` passes, 247 passed and 4 ignored in bb_game, 29 in bb_agents. `relay_time_by_transfer_grade` and `the_relay_man_holds_the_ball_for_the_sourced_exchange` pass. `a_throw_beyond_the_arm_goes_through_the_relay_man` and `the_holder_predicts_the_pivot_the_world_makes` still pass.
- **Probe copies.** I exported `git archive 0f9850e` and `HEAD` into `sim/target/review-probes/{before,after}/` and built them into separate target dirs (`target-before`, `target-after`).
  - Both copies have the throw-race note's `BB_TAG_GO` override, which sends every tagging runner from his starting base.
  - The `after` copy also has two read-only stderr traces. `BB_THR` logs every throw: `relayed`, `pivot`, received-a-throw, `set_base`, earliest, hurry, time since up, base, relay. `BB_RT` logs runner decisions taken while a relay throw is in the air.
  - The worker's `zz_race_probe.rs` and `zz_relay_probe.rs` are copied as examples, plus my `zz_gs_probe.rs` (every glove securing, base touch, out and run).
  - A shared target dir silently served the `after` build for `before` (cargo hashes workspace members by relative path), so the copies need separate target dirs. Worth knowing for the next comparison probe.
- **Counterfactual** (`BB_TAG_GO=1 zz_race_probe 2000 200`, before and after; summarized with `sim/target/review-probes/go.py`):
  - The table in P2-1.
  - Run on the worker's own `go2`/`go3` outputs, `go.py` reproduces his figures exactly (21% → 49%, 68% → 91%), so the difference is the seeds, not the counting.
- **As-built trace** (`BB_RT=1 BB_THR=1 zz_gs_probe 0 100`, after; `recv.py`):
  - one relay throw (held 0.750 s, not hurried);
  - 0 runner decisions during a relay throw;
  - 88 base-receiver throws: 71 within 1.5 m of a bag (27 hurried, minimum 0.283 s) and 17 at 1.5–6 m (mean 0.590 s, minimum 0.283 s);
  - 658 outfield throws of 75–90 m to a base, 1 through the relay man.
- **`extra_base_rates 100` (after):** `[[346, 83], [156, 83], [84, 65]]`, so doubles first to home is 65 of 84. `zz_relay_probe 0 100` (after) gives the same 65 of 84 and 2 infield catch-and-throw "relays" (0.750 s and 0.900 s).
- **Sources:** a web search found the MLB.com article, and Statcast's exchange definition and catcher figures (MLB.com, "Statcast adds pop time, exchange, arm strength"). I did not read the 0.75 s or 0.62 s figures themselves.

## Resolution

Open.
