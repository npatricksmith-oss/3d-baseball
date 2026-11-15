No P1; sound to merge as an explicitly interim read model, with acceptance of the 3.32 [3.42] run level left to the owner.

## Reviewed

`dc1ce82..cb42714`: diagnosis, spec, implementation, and measurements. Read `CLAUDE.md`, the owner's TASKS guidance, agent-models §5.1/§5.2/§10, and both calibration notes. Reviewed the changed examples, call fixtures, holder regression, rating plumbing, RNG, and diagnostic tracing. Review checkout `5598b7f` adds only the review prompt after the requested range. Current main `24b9f17` differs from `dc1ce82` only in TASKS documentation; its simulation code is the correct baseline for reproducing the fit.

## Findings

### P2-1 — The closing read retains the mid-flight loss of track information

**Location:** `sim/crates/bb_game/src/live.rs:1012`, `:1027`; `docs/calibration/2026-11-15-outfield-read.md:219`.

Each prediction reconstructs the estimate from the instantaneous observation, four persistent error draws, and the plate cue. There is no retained observed track; the plate cue loses weight as its spin allowance grows (`live.rs:919`). The closing multiplier repairs late accuracy without repairing this earlier behavior.

**Evidence:** `read_track 23080 8 7` reproduces an initial landing miss of 4.6 m, increasing to 12.9 m at t=3.59 s. At t=4.19 s the body is at (−43.3, 89.2), about 11 m from the eventual landing point (−32.7, 86.3), with 2.36 s left. The catch regression passes, but the visible excursion remains. The trace also includes an early backup assignment, so the whole route should not be attributed solely to read error.

**Suggested fix:** retain and assimilate successive noisy observations, allowing the observer to learn curvature with his own physics. Validate the read and route on this existing trace, rather than tightening the closing window to rescue the catch. This is a confirmed, disclosed limitation inherited from the previous read, suitable for a separate follow-up; it does not block merging the present bias correction.

### P2-2 — The evidence does not establish that the remaining fly-hit deficit is the legs

**Location:** `docs/calibration/2026-11-15-outfield-read.md:215`, `:238`.

“The last two points ... are not the read” is stronger than the evidence. The reported 2.5 s experiment moves fly hits from .099 to .109 and still catches .949 of routine flies, inside the owner's 90–99% band. The band does not uniquely establish 3.0 s or exclude remaining read overprecision. The routine boundary itself is an approximation anchored on one published example, not a matched Statcast probability table.

**Probe:** over branch seeds 0–47, fair first-flight flies landing beyond the dirt, away from walls, with landing after 3.05 s: select the nearest starting outfielder and require at least 60 ft to cover and a trace through release +3 s. Across 233 plays his mean displacement is **33.6 ft**, its projection toward landing **32.2 ft**, and distance traveled **34.1 ft**. His speed cap is **26.9 ft/s**; mean per-play maximum recorded non-dive speed is 24.9 ft/s. These measurements do not demonstrate elite legs.

Statcast's Jump window starts at release and its leaderboard excludes chances easier than 90%. [MLB definition](https://www.mlb.com/glossary/statcast/jump). Its Inciarte example gives 43 ft and +8.2 ft, implying a 34.8 ft baseline for that play. [MLB worked example](https://www.mlb.com/news/new-outfield-jump-burst-route-metrics-at-baseball-savant). Published average sprint speed is 27 ft/s. [MLB sprint-speed reference](https://www.mlb.com/news/the-fastest-player-on-each-mlb-team-c277321114).

**Suggested fix:** qualify the attribution as unresolved and replace the “average equals elite” estimate with measured movement and its selection limits. These probe plays are not matched to Statcast catch-probability bins, so they cannot establish the precise remaining cause either. Diagnosing movement next is reasonable; slowing fielders or ruling out the read is not yet justified. No compensating run knob is warranted.

### P3-1 — The spin guard catches broad drift, but not changes hidden within exit-speed groups

**Location:** `sim/crates/bb_game/src/live.rs:836`, `:6688`.

The prior is a **launch-angle-bin median**, with no exit-speed conditioning; it is not the conditional population mean asked about in the review question. This agrees with the actual spec and is a legitimate population belief. The existing test checks only four pooled fly-angle medians within 15%, leaving speed-dependent changes, grounders, and liners outside its coverage.

**Evidence:** the baseline's 48-game `batted_spin` run reproduces all nine coefficients exactly. In the separate 48-game branch probe, 40–50° spin medians are 2,802/4,102/4,291 rpm for <80/80–95/≥95 mph (n=139/207/40); the prior at 45° is 3,819 rpm. Pooled agreement can conceal changes in those subpopulations. This does not invalidate the small measured routine-play bias below.

**Suggested fix:** refit the offline belief when `work/collision-q` integrates, using the resulting contact population; do not substitute the particular ball's true collision inputs or spin at decision time. Prefer a reproducible model-derived offline calibration procedure, retaining uncertainty, over another manually maintained table. Extend the existing small staleness check with a few populated exit-speed/angle groups or landing-bias checks. The current test would catch another large pooled shift, but cannot promise to catch the next contact change.

## Model and invariant assessment

- **Observation boundary:** the back-flight receives noisy velocity `ve`, assumed population spin, elapsed observed contact time, and the shared physics environment (`live.rs:856`). It does not read the ball's true spin or future. The plate cue additionally uses the observed contact point. Importantly, `read_shrink` (`:934`) computes its vacuum descent time from the **true instantaneous height and vertical velocity before noise is added**. This is a state-dependent observation-noise law, like the existing distance scaling, rather than an agent reading the world's landing time. The replan interval (`:1720`) uses the fielder's own predicted bounce/end time. Neither uses the true landing point or future trajectory to choose an action.
- **Noise and replay:** the change still perturbs physical perception inputs and resolves catches physically. No catch-probability roll, new production RNG draw, changed draw order, or new `Purpose` was introduced. The same four keyed perception draws are retained across reads. New transcendental operations use `bb_math::m`; estimated parameters are named and tagged [E]. Communication isolation passes.
- **Closing shape:** continuous visual correction has research support; the particular linear position scaling, effectively quadratic velocity contribution, and 3.0 s onset are an empirical surrogate, not an experimentally established uncertainty law. The note plainly identifies the fitted onset and rating ratio. [Fink, Foo & Warren](https://pmc.ncbi.nlm.nih.gov/articles/PMC3816735/) supports online correction, not these numerical choices. That distinction is adequate for an interim model. The .973 versus .950 aggregate catch rates do not alone prove an inversion of judgment difficulty: hang time and opportunity differ. The early high-fly spread test passes; whole-flight ordering is not established. Keep this separate from P2-1 and from later fielding errors.
- **Rating:** `play.rs:135` applies §10's less-is-better ratio to each fielder's own position noise, velocity noise, and read time constant. Grade 50 preserves the default; rostered fielders remain average. The same-flies poor-versus-good direction test passes. The reported .917/.983 game rates were not independently rerun.
- **Trace:** `read_trace` is thread-local and off by default. Simulation code only appends records (`live.rs:4483`); readers are diagnostic examples. It adds no decision input or RNG consumption. Seed 7's full event digest was identical with tracing on and off.

## Verification

All heavy commands ran serially from `sim/`, prefixed with `taskset -c 0-7 nice`, with `CARGO_BUILD_JOBS=2`, the requested PATH, and Zig caches under this worktree's `sim/target/`.

- **One full test run:** `cargo test --workspace`: **382 passed, 0 failed, 4 ignored**. Includes normal communication isolation, its negative control, the holder regression, true-spin isolation, closing-read bands, rating direction, and seed 23080 play 8. Did not run `isolation_many_seeds`.
- **One standard run:** `cargo run --release -p bb_cli -- games --n 300 --seed 0`: runs **3.32**, PA/team-game 37.1, K/BB 21.5/8.2%, HR/PA 1.88%, reached on BIP 32.4%; GB/LD/FB/PU hits **.293/.653/.099/.011**. No rules warnings. Matches the reported after column. Before and bracketed second-set figures were not rerun. The CLI maps a nonzero seed argument through `seed * 1_000_003 + i`; the note's “[1000–1299]” label should not be read literally as game seeds for `--seed 1000`.
- **One routine pass:** `cargo run --release -p bb_game --example fly_misses -- 0 300`: **1,739/1,799 = .967**. By angle: 402/423=.950; 1,160/1,194=.972; 177/182=.973. All eligible outfield flies: 3,235/3,527 caught, .083 fall. Zero-draw mean depth bias at 2/1/0.5 s before landing, metres:

  | Launch angle | 2 s | 1 s | 0.5 s |
  |---|---:|---:|---:|
  | 25–35° | +.21 | −.03 | −.02 |
  | 35–50° | +.34 | +.10 | +.03 |
  | 50°+ | +.33 | +.10 | +.03 |

  The remainder is small, not exactly zero. With perception noise included the mean depth error at 2 s is +.52/+.52/+.42 m, falling to +.01/+.05/+.01 m at 0.5 s.
- **Targeted probes:** baseline `batted_spin 48` via current main's manifest, with all build output redirected under this worktree's target; branch 48-game spin/exit-speed/movement probe plus seed-7 trace digest comparison; `read_track 23080 8 7`. Sources and logs are under `sim/target/outfield_review_probe.rs` and `sim/target/outfield-review-*.log`. No source, test, spec, ledger, or TASKS file was edited; no commit was made.

## Resolution

No P1. Resolved by the lead (2026-11-15); whether the run level is acceptable for the merge is the owner's (Q7, asked for the three overnight branches together).

- **P2-1 (the read keeps no track of what it has seen; the deep drift on seed 23080 play 8 remains, 12.9 m at 3.59 s): deferred to its own job, recorded in TASKS.** The fielder should assimilate successive noisy observations and learn the flight's curvature with his own physics, validated on that play's trace, not by tightening the closing window. This branch stays the interim model the review calls it.
- **P2-2 (the remaining fly-hit deficit is not shown to be the legs): the claim is withdrawn.** The note now says the cause of fly balls falling at .10 against .13 is unresolved: the review's probe has the nearest outfielder covering 33.6 ft in Statcast's 3 s window on plays needing 60 ft or more, against a 34.8 ft baseline in MLB's worked example, with a 26.9 ft/s cap against a 27 ft/s average sprint speed, which is not elite; and `READ_CLOSE_S` at 2.5 s is also inside the owner's band (.949 routine, flies .109). Next: movement and the read measured against matched catch-probability bins before either is changed. No compensating knob.
- **P3-1 (the spin belief is a launch-angle-bin median; its staleness test is coarse): deferred to the integration with `work/collision-q`,** which changes the contact population: refit the belief there by the offline procedure, and extend the staleness test with a few exit-speed and angle groups or landing-bias checks. Recorded in TASKS.
- The note's seed label: `--seed 1000` runs game seeds 1,000,003,000 onward, not 1000–1299; corrected where the note says otherwise.
