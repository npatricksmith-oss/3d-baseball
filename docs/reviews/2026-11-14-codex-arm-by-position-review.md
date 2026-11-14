# Arm strength by position: independent review

Reviewed `4937cb8` and `2f20f73`, comparing parent `9fdeec169a0ba522bf7b165da425e2d6274e1ea3` with `2f20f73a1a1ae5bc6567307111e67a803b81aadd`. Read `CLAUDE.md` first. Findings and line numbers refer to these revisions, not subsequent changes on main. No P1 findings; one P2 and two P3 findings.

## Findings

### P2 — Correct the source definition and leave the set-throw mapping explicitly estimated

**Locations:** `docs/agent-models.md:659–661`, `docs/calibration/2026-11-14-arm-by-position.md:6–8`, `sim/crates/bb_game/src/play.rs:63–69`.

The seven published mph values are transcribed correctly, but “top 10%” for every player is false. Savant selects the fastest **1% for 1B, 5% for 2B/3B/SS, and 10% for OF**, with qualifying minima of 100/75/50 throws respectively. [Savant leaderboard](https://baseballsavant.mlb.com/leaderboard/arm-strength). The [MLB glossary](https://www.mlb.com/glossary/statcast/arm-strength) agrees on these position-specific fractions.

Different fractions do **not** imply multiplying by 10, 5, or a percentile ratio. The methodology linked by Savant explains that these selections seek an average hard-throw speed roughly 5% below a player's fastest throw. They accommodate different frequencies of competitive throws. Accordingly, for a sim parameter meaning the **nominal speed of a full-effort release before motor noise**, a hard-throw mean is preferable to a season record: do not automatically add 5%. `execute_throw` already varies releases around that nominal value. [Tango's methodology](https://tangotiger.com/index.php/site/comments/statcast-arm-strength-for-fielders).

However, “therefore the conversion is units only” does not establish a measured **common set-throw capability**. The same methodology gives empirical position equivalencies, reflecting throwing context as well as personnel. Taking SS/3B as an approximate reference gives the following sensitivity check:

| Position | Raw mean × 0.44704, m/s | SS/3B-context adjustment, mph | Approximate equivalent, m/s |
| --- | ---: | ---: | ---: |
| 1B | 35.450272 | +6 | 38.132512 |
| 2B | 35.360864 | +3 | 36.701984 |
| 3B | 37.953696 | 0 | 37.953696 |
| SS | 38.490144 | 0 | 38.490144 |
| LF | 39.071296 | −3 | 37.730176 |
| CF | 40.054784 | −3 | 38.713664 |
| RF | 40.546528 | −4 | 38.758368 |

These adjustments follow the author's RF equivalencies, shifted to SS/3B; they are **not measurements of stationary throws** and should not be installed as exact replacements. [Published equivalencies](https://tangotiger.com/index.php/site/comments/statcast-arm-strength-for-fielders). The sim's “set” includes a balanced stride, but has no explicit crow-hop velocity contribution; its one arm value also follows a fielder onto relay and backup plays.

**Answer on materiality:** none of the raw unit conversions is numerically wrong. As position-context hard-throw proxies they are defensible [E]. As common set-throw limits, material context differences remain plausible: especially 1B, about **2.68 m/s**, and OF, about **1.34–1.79 m/s**, in the check above. It is not supportable to call these measured set-throw limits or to assert an exact stationary correction from these means alone. Correct the fractions everywhere and specify which interpretation is intended. If retaining the raw defaults, mark the context mapping as an assumption rather than “Definition settled”; a numerical recalibration needs matched throwing-context evidence.

### P3 — Replace the ledger's universal divergence and neutrality explanation with the measured limits

**Locations:** `docs/metrics-ledger.md:102`; calibration note, “Effect.”

The standard commands reproduce the rounded run totals, but the exact combined mean is **5.4641667 → 5.4666667**, not identical: **6,557 → 6,560 runs over 1,200 team-games**. More substantially, “Every digest changed” is false: **188/600 digests remained identical**, 92 in the first set and 96 in the second. Direct, uninstrumented simulations also reproduce unchanged digests for seeds 0 and 6. An unchanged event-log digest does not imply unchanged throw trajectories.

Nor does every throw change: P/C retain their old arms. LF increased **39.0 → 39.071296 m/s**; it did not become 0.1 m/s weaker. More SS throws cannot by themselves establish more successful SS plays or explain the run total.

The additional diagnostic below finds offsetting sample movements in outs after throws: **+40 on infield-origin plays, −40 on outfield-origin plays**. Neither movement is statistically clear, and changing only the infield arms does not reproduce the simple “stronger infield means fewer runs” explanation. The fair reading is **“no detectable aggregate run change in these 600 games; mechanism unresolved,”** not established neutrality or a demonstrated reshuffling explanation. Retain the note's uncertainty about LD/PU; extend that caution to the causal narrative.

### P3 — Label the second standard seed set correctly

**Locations:** `docs/metrics-ledger.md:98,102`; seed generation in `sim/crates/bb_cli/src/game_lab.rs:104`.

`games` maps its argument to `seed * 1_000_003 + i`. Thus `games --n 300 --seed 1000` uses **1,000,003,000–1,000,003,299**, not 1000–1299. Conversely, `throw_probe 300 1000` really would use 1000–1299. Label the ledger with the command argument or actual seeds so readers reproduce the intended experiment.

## Verification

Git archive copies and build/probe outputs were confined to ignored `sim/target/arm-position-review/`; no source, test, specification, or TASKS file was edited. The only tracked addition is this review. Used `PATH=$HOME/.cargo/bin:$PATH`, `CARGO_BUILD_JOBS=2`, and `ZIG_GLOBAL_CACHE_DIR` inside that directory. Final before/after binaries use separate Cargo target directories and have distinct hashes; this avoids reuse of stale artifacts across archived revisions. All builds were offline. CLI builds disabled the optional Rerun feature.

### Tests and standard runs

- At `2f20f73`: `cargo test --offline --release --workspace --no-default-features`: **206 tests passed**, no failures; doc tests passed. Includes the new positional-means test, throwing behavior, cutoff/relay cases, crate-graph isolation, and FFI tests.
- At both revisions: built `sim-cli` and ran `games --n 300 --seed 0` and `games --n 300 --seed 1000`. Every printed rate in the ledger row reproduced; all four runs reported no rules warnings.
- At both revisions: built and ran `cargo run --offline --release -p bb_game --example throw_probe -- 300 0` (executed the resulting binaries). Its printed aggregates reproduced the ledger.

| CLI seed argument | Before runs/team-game | After runs/team-game | Exact total runs, before → after |
| --- | ---: | ---: | ---: |
| 0 | 5.530000 | 5.401667 | 3,318 → 3,241 |
| 1000 | 5.398333 | 5.531667 | 3,239 → 3,319 |
| Combined | 5.464167 | 5.466667 | 6,557 → 6,560 |

| Throw probe, actual seeds 0–299 | Before | After |
| --- | ---: | ---: |
| Balls in play/game | 71.1 | 70.9 |
| Outs/BIP | .590 | .591 |
| Set throws/game; taken in air | 46.26; 96.3% | 45.99; 96.1% |
| Hurry >2/3/game; taken in air | 1.95; 92.5% | 1.97; 91.6% |
| SS throws/game; taken in air | 7.70; 96.4% | 7.91; 95.7% |
| 2B throws/game; taken in air | 8.16; 96.2% | 8.02; 96.2% |
| LF throws/game; taken in air | 7.20; 95.2% | 7.00; 95.7% |

### Offsetting effects and uncertainty

Compiled a diagnostic from stdin with `rustc`, linked separately to each archived `bb_game`, without adding or changing source files. It runs `Game::new(seed, false)` for the exact 600 standard seeds, captures the existing read-only `choice_trace`, and retains each finished game's run total and digest. These run totals agree with the CLI. For an out with `call.base != 0`, it counts whether a throw/return commitment preceded `call.t_out` and groups by the first `Input::GloveSecured` fielder: positions 1–6 versus 7–9. This is **outs after throws on IF/OF-origin plays**, including relays; it is not an official assist statistic or proof that a particular throw caused the out. Caught-ball outs are excluded.

| Diagnostic over 600 games | Before | After |
| --- | ---: | ---: |
| BIP | 42,820 | 42,886 |
| IF-origin outs after throws | 10,097 | 10,137 |
| OF-origin outs after throws | 1,690 | 1,650 |
| IF throws, including returns | 14,349 | 14,423 |
| OF throws, including returns | 14,881 | 14,916 |

Using a game as the paired sampling unit, the run difference is **+0.0025 runs/team-game**, approximate 95% interval **[−0.1373, +0.1423]** (`mean ± 1.96 * sample_sd / sqrt(600)`). IF-origin out changes are **+0.0667/game ±0.2177**; OF-origin changes **−0.0667/game ±0.1070**. The observed cancellation is real arithmetic, but these data cannot distinguish a persistent offset from sampling variation.

As a bounded check, reused the after binary and public `Game::with_fielders` to restore old arms for one group at a time, preserving each player's hand. No other attribute changed:

| Arms, same 600 seeds | Runs/team-game | Paired change from before; approximate 95% interval |
| --- | ---: | ---: |
| Before, all old | 5.464167 | — |
| New IF only, old OF | 5.520000 | +0.05583; [−0.05411, +0.16578] |
| Old IF, new OF only | 5.444167 | −0.02000; [−0.13805, +0.09805] |
| After, both new | 5.466667 | +0.00250; [−0.13727, +0.14227] |

All intervals include zero. These whole-game counterfactuals include subsequent sequence changes and interactions; they do not isolate a same-play physical effect. They provide no basis for claiming a resolved infield-versus-outfield mechanism.

### Wiring, invariant 2, and test independence

No additional live path retaining the old **36/39 positional split** was found. `Game::new` and `with_fielders` preserve roster handedness and obtain the per-position defaults. Holder timing and execution use the holder's `a.arm`; the relay estimate at `live.rs:2658` uses the relay man's `a.arm` and transfer, so it automatically incorporates the positional change. Cutoff placement thresholds are geometric coaching estimates, not hidden 36/39-speed assumptions. The 20–48 m/s throw table covers the new grade-50 values.

The teammate-attribute access predates this change. Treating a static teammate's nominal capability as familiar practice/scouting knowledge is consistent with the current simplified model; it does not expose the realized noisy `ExecutedThrow` or a future trajectory. Do not replace this estimate with the actual upcoming release speed. Runners still use the pre-existing **37 m/s, 0.75 s generic scouting prior** in `typical_throw_time`/`ball_time_to`; this is not the old positional split, though the stronger OF defaults widen that prior's mismatch. Position-aware, uncertain scouting could improve it without giving runners hidden execution state. No new invariant-2 breach was identified in the diff.

FFI constructs `Game::new(seed, true)` and exports the resulting timelines; the client plays those back. No independent arm-strength constant or schema field needs updating. The shortstop fixture helper was updated to 38.490 m/s; remaining explicit 39 m/s diagnostic throws specify test inputs rather than asserting the average outfielder's speed. No fixture failure appeared.

The new test **is independent of the production constant and unit-conversion helper for its stated arithmetic purpose**: its expected array contains literal results rather than reading `ARM_MEAN_MPH` or calling `mph_to_mps`. Independent Python `Decimal(mph) * Decimal('0.44704')` checks put every expected literal within the test's 0.0005 m/s tolerance (largest error 0.000472, RF). It checks position ordering and grade-50 preservation too. It cannot validate the mistaken source definition or the set-throw interpretation: those assumptions are shared with the specification. The live timing helper still calls production `throw_time`, so it is a consistency fixture rather than an independent flight-time oracle.

## Resolution

Resolved 2026-11-14 (lead).

- **P2, fixed in the text; the values stand as [E].** The spec, the note and the code comment now give Savant's fractions by position group (1% at first base, 5% for the other infielders, 10% in the outfield) and say the mapping from a position's hard-throw mean to a common set-throw limit is an estimate. The review's context adjustments (first base about 2.7 m/s low, outfield 1.3 to 1.8 m/s high) are recorded in the note and not applied, as it advises: they are not measurements of stationary throws. Since this review was written, set throws no longer always leave at the arm value (speed for accuracy, agent-models §5.5, `4477892`), which makes "near maximum" the right reading of the parameter.
- **P3, fixed.** The ledger says 412 of 600 digests changed and "no detectable aggregate change, mechanism unresolved" where it claimed every digest changed and an unchanged mean.
- **P3, already covered.** The ledger's "How to add a row" states that `--seed 1000` means game seeds from 1,000,003,000 and that the notes' label "seeds 1000-1299" is the command's argument; rows keep that label.
