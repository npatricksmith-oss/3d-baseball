# Calibration Targets Specification

**Last updated:** 2026-10-31
**Status:** Draft v1. Gates milestones M3 (plate-appearance bands), M4 (game-level accounting and first envelope check), and M5 (fielding and baserunning bands).
**Parent:** [PLAN.md](../PLAN.md) §13 · **Siblings:** [physics-spec.md](physics-spec.md), [agent-models.md](agent-models.md), [data-formats.md](data-formats.md), `rules-spec.md` (P4)

This document defines **what "correct" means** for every number the simulation produces, and **what may be changed** to get there. It collects the Tier A physics checks from the physics spec, defines the Tier B playability envelope with numeric bands, owns the player-population distributions that agent-models §10 defers here, and sets the calibration procedure, sample sizes, and report format.

Status tags (as in the sibling specs): **[S]** sourced, **[D]** derived, **[E]** estimate, **[C]** design choice the owner may revisit. One new tag:
- **[R] Reference:** an approximate recent-MLB value (2023–2025 seasons), written from general knowledge of FanGraphs, Baseball Savant, and Baseball-Reference league pages. It orients a band; it is **not** a target the sim must hit. Every [R] value is re-checked against the actual league pages in task **R10** before M3.

**D1 reminder.** Calibration never adds an outcome roll, a probability override, or a correction term that pushes a stat toward its band. If a stat cannot reach its band by moving the permitted knobs (§2.3), **a model is wrong** and gets fixed (PLAN §13.2).

---

## 1. The Four Kinds of Check

| Kind | Question | Must match reality? | Fails how | Where |
| :--- | :--- | :--- | :--- | :--- |
| **A. Physics validation** | Does the world obey physics? | **Yes** | Hard test failure | §3 (values owned by physics-spec §14) |
| **M. Mechanism plausibility** | Does each agent mechanism, in isolation, produce human-looking numbers (pitch shapes, bat speeds, exit velocities, sprint times)? | Roughly. Bands are wide | Gate failure at the milestone that owns the mechanism | §4 |
| **B. Playability envelope** | Does a league of these players produce watchable baseball? | **No.** A distinct league is allowed | Gate failure if outside the **envelope**; warning if outside the **target** | §5, §6 |
| **S. Sensitivity and direction** | Does every attribute and tactic push outcomes the way physics and baseball sense say, by a visible amount? | Direction only | Hard test failure on a wrong sign; gate failure on a dead attribute | §7, §8 |

Two additional hard invariants run alongside every calibration run but are not calibration: **accounting** (box-score identities, owned by the rules spec) and **liveness** (no stuck states, no timeouts). A calibration report with any accounting or liveness failure is invalid and is not read for bands.

### 1.1 Target versus envelope

Every Tier B and Tier M metric has two ranges:
- **Target band** [C]: where the default league should sit. Defaults to "MLB-like" because that is the easiest baseball for a viewer to read. The owner may move targets to give the league its own flavor (for example more contact, more steals) without reopening anything else.
- **Envelope**: outside this, the game stops looking like baseball (too many strikeouts to watch, scores that look like football). Leaving the envelope fails a gate even if the owner has moved the target.

A metric inside the envelope but outside the target is a **warning**: it is tuned next, but it does not block a milestone except where §10 says so.

---

## 2. Principles

### 2.1 Order of calibration

Calibrate bottom-up. A higher level is only read once every level below it passes, because a wrong lower level can be "fixed" at a higher level by compensating knobs, which hides a model bug.

```
A  physics goldens (M1–M2)
└─ M  mechanism bands: pitch shapes, release, swing, batted ball, running, throwing (M1–M5)
   └─ B1 plate-appearance rates in the cage (M3)
      └─ B2 ball-in-play outcomes (M4–M5)
         └─ B3 game totals and watchability (M4–M5)
            └─ B4 spreads: player and team distributions (M7)
```

### 2.2 What is never tuned toward stats

- **Any [S] or [D] physics value** (aerodynamics, bat-ball collision, ball mass). These are fixed by physical data.
- **[E] physics values** (surface bounce, f(u) bat profile) are calibrated **only against physical observations** of the same quantity (grounder speed at the infielder, roll-out distance, COR-versus-location data), never against BABIP or runs.
- **Rules geometry** (zone height fractions, base distances, fence positions of the reference park).
- **Learned belief tables** (agent-models §8) are re-fit, never hand-edited (§9.4).

### 2.3 What may be tuned toward stats

Only these, in this order of preference:
1. **Population distributions** (§11): μ and σ_pop of each attribute in physical units, position adjustments, correlations.
2. **Global knobs** (PLAN §13.2, agent-models §1.3), each a single multiplier defaulting to 1:

| Knob | Multiplies | Sweep range | Expected primary effects (sign as the knob rises) |
| :--- | :--- | :--- | :--- |
| `K_perception` | batter σ_ang, Weber w, σ_s0; fielder read noise | 0.5–2.0 | K% ↑, chase% ↑, whiff% ↑, hard-hit% ↓, routine-play success ↓ |
| `K_command` | pitcher σ_cmd | 0.5–2.0 | BB% ↑, HBP% ↑, zone% ↓, mean miss ↑, hard-hit% ↑ (more middle-middle mistakes) |
| `K_swingplane` | batter σ of δ_v and t₀ | 0.5–2.0 | whiff% ↑, weak contact ↑, mean EV ↓, foul share ↑ |
| `K_fielder_reaction` | fielder T_r and τ_read | 0.5–2.0 | BABIP ↑, extra-base share ↑, routine-play success ↓ |
| `BatSpeed` μ shift | population mean of `BatSpeed` | ±3 m/s | mean EV ↑, HR/PA ↑, K% ↓ slightly |

3. **Agent [E] parameters** (agent-models §10 mappings, κ_c, decision thresholds), **only** when a Tier M band shows the mechanism itself is off. Changing one requires a note in the calibration report naming the band that justified it.

### 2.4 No compensating pairs

If two knobs move in opposite directions in one calibration round to hold a stat in place (for example `K_perception` up and `K_swingplane` down), the report must explain why. Repeated compensation is a sign that one mechanism is mis-specified and the other is masking it.

---

## 3. Tier A: Physics Validation

Owned by [physics-spec §14](physics-spec.md). Collected here so a calibration run can list every check in one place. **Tested, never tuned.**

| ID | Check | Pass |
| :--- | :--- | :--- |
| A1 | 100 mph / 30° / 2500 rpm backspin, calculator atmosphere | Carry 406 ± 3 ft, flight 5.6 ± 0.1 s |
| A2 | A1 with spin decay τ₁₀₀ = 20 s | Within 1 ft of A1 |
| A3 | Step-size convergence, 10 ms vs 1 ms | Δcarry < 0.01 ft |
| A4 | Vacuum | Analytic to 1e-9 relative |
| A5 | Magnus sign table | Directions as listed |
| A6 | Mirror symmetry | Mirrored trajectories |
| A7 | Head-on impact, no spin | q formula to 1e-9 |
| A8 | Default bat at u_ss | q = 0.20 ± 0.01; EV 105–109 mph for 85 mph pitch, 75 mph bat |
| A9 | Gripping oblique impact | KNS formulas |
| A10 | Spin vs undercut | Signs correct, magnitudes in published ranges (values pending R3) |
| A11 | Energy guard | No collision gains kinetic energy |
| A12 | Default bat mass properties | Match profile |
| A13 | Rolling deceleration | Exactly μ_r·g |

**Plausibility cross-checks** (physics-spec §14.3; flag, don't fail): the four pitch flight rows and four batted-ball rows listed there. A deviation > 3% in carry or flight time from those rows after an unrelated code change is a regression and fails CI.

**Oracle agreement:** the Rust implementation matches the Python oracle (`analysis/oracle/`) over the M1/M2 input grids to 1e-6 relative in position at every recorded sample.

---

## 4. Tier M: Mechanism Plausibility

Each table is checked by the milestone named in its heading, using `sim-cli cage` or a mechanism-specific lab command, on a **league-average population** (§11) unless the row says otherwise. Values are the population mean unless stated.

### 4.1 Pitch shapes and release (M1, re-checked M3)

Measured with Savant-definition movement through `sim-cli export --savant` (data-formats §9.4), right-handed pitchers; left-handers mirror horizontal break. Movement in inches; HB positive = arm side.

| Pitch | Velo [R] (mph) | Spin [R] (rpm) | IVB [R] (in) | HB [R] (in) | Envelope |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Four-seam | 94 | 2300 | +16 | +7 | velo 88–99, IVB +10 to +21 |
| Sinker | 93 | 2150 | +8 | +15 | IVB +2 to +13, HB +10 to +20 |
| Cutter | 89 | 2400 | +8 | −2 | HB −7 to +2 |
| Slider | 85 | 2450 | +1 | −5 | IVB −6 to +7, HB −12 to 0 |
| Sweeper | 82 | 2600 | 0 | −14 | HB −20 to −9 |
| Curveball | 79 | 2550 | −10 | −8 | IVB −18 to −3 |
| Changeup | 85 | 1750 | +6 | +14 | velo gap to own FB 6–12 mph |
| Splitter | 86 | 1400 | +3 | +10 | spin 900–1900 |

Targets: each pitch type's population mean within ±2 mph, ±250 rpm, ±3 in of the [R] value. Envelope as listed.

| Release metric | Target [R] | Envelope |
| :--- | :--- | :--- |
| Release height | 5.8–6.0 ft mean | 5.0–7.0 ft for 98% of pitchers; agent-models §11 public range 5.5–6.5 ft for the middle 80% |
| Release side (abs) | ~1.9 ft | 0.5–3.5 ft |
| Extension | 6.4 ft (1.95 m) | 5.5–7.5 ft |
| League pitch mix | FF ~32%, SI ~15%, SL+ST ~23%, CH ~11%, CU ~8%, FC ~8%, FS ~3% [R] | No type > 45% league-wide; every type ≥ 1% |
| Single pitcher mix | — | No pitcher throws one type > 80% over a season (degeneracy check) |

### 4.2 Command (M3)

| Metric | Target | Envelope | Note |
| :--- | :--- | :--- | :--- |
| Mean radial miss from intended target, grade-50 `Command`, fresh, leverage 1 | 8–9 in [D: σ_cmd 10 mrad × ~16.8 m ≈ 6.6 in per axis; mean radial = σ√(π/2)] | 6–12 in | Public catcher-target tracking studies report misses on the order of 10 in [R]; R10 confirms |
| Grade 80 vs grade 20 mean miss ratio | ≈ 0.33 [D: ρ 1.2 over 6 steps] | 0.25–0.45 | |

### 4.3 Swing (M2–M3)

| Metric | Target [R] | Envelope |
| :--- | :--- | :--- |
| Mean bat speed (competitive swings) | 71.5–72 mph | 67–76 mph |
| Bat speed spread across hitters (SD of player means) | ~2.5 mph | 1.5–4 mph |
| Swing length | ~7.3 ft | 6–8.5 ft |
| Attack angle at contact | ~10° | 5–16° |
| Commit-to-contact time | ~0.15 s | 0.12–0.19 s |

### 4.4 Batted balls (M2 cage sweeps, M3 live cage)

All balls in play plus fouls with tracked contact ("batted-ball events", BBE).

| Metric | Target [R] | Envelope |
| :--- | :--- | :--- |
| Mean exit velocity | 88.5 mph | 85–92 mph |
| Mean launch angle | 12° | 6–18° |
| Launch angle SD | ~27° | 20–35° |
| Hard-hit rate (EV ≥ 95 mph) | 38% | 30–47% |
| Barrel rate (Statcast definition) | 7.5% | 4–11% |
| Sweet-spot rate (LA 8–32°) | 33% | 26–40% |
| GB / LD / FB / PU share | 43 / 21 / 29 / 7 % | GB 35–50%, LD 16–26%, PU 4–11% |
| Player max EV (season, regulars) | 108–118 mph | none over 125 mph |
| Mean HR true distance | ~400 ft | 380–420 ft |
| Pull share of GB (same-side batter) | ~55% | 45–65% |

**Shape checks** (M2 gate, PLAN §14): early contact pulls, late goes the other way; undercut gives backspin flies; overcut gives topspin grounders; off-barrel contact is weak. EV must rise with bat speed at **1.0–1.3 mph per mph** of bat speed at the sweet spot [D: dEV/dv_bat = 1 + q with q ≈ 0.2].

### 4.5 Running and throwing (M4–M5)

| Metric | Target [R] | Envelope |
| :--- | :--- | :--- |
| Mean sprint speed | 27 ft/s [S: Statcast average; agent-models §10] | 26–28 ft/s population mean |
| Home-to-first, RHB, grade-50 `Speed` | ~4.4 s | 4.2–4.7 s |
| Home-to-first spread, grade 80 → 20 | ~4.0 → ~4.9 s | fastest > 3.85 s; slowest < 5.3 s |
| Catcher pop time, grade-50 | ~2.00 s | 1.85–2.15 s |
| Outfield max throw, mean | ~87 mph | 80–94 mph |
| Infield (SS/3B) max throw, mean | ~84 mph | 78–90 mph |
| Fielder first-step reaction after contact | 0.2–0.3 s | 0.15–0.40 s |

### 4.6 Fielding (M4 crude, M5 gate)

Define **opportunity time** as the time from contact to the ball reaching the intercept point, and **required distance** as the shortest distance from the fielder's pre-pitch position to that point. A ball is **routine** if a grade-50 fielder covers the required distance with ≥ 1.0 s to spare (fly balls) or if it is a grounder hit within 3 m of a fielder's starting position at < 95 mph.

| Metric | Target | Envelope |
| :--- | :--- | :--- |
| Routine fly success | ≥ 99% | ≥ 98% (agent-models §11) |
| Routine grounder success (fielded and converted to an out, given a grade-50 runner) | ≥ 96% | ≥ 93% |
| Errors per team-game | 0.55 [R] | 0.30–0.90 |
| Fielding percentage | .985 [R] | .975–.992 |
| Catch rate by Statcast-style star bucket | monotone, rising with catch probability; 5-star (< 25%) plays made ~10–20% | must be monotone |
| Double plays turned per team-game | 0.70 [R] | 0.40–1.10 |
| Wild pitches + passed balls per team-game | 0.35 [R] | 0.15–0.70 |

---

## 5. Tier B: Plate-Appearance Rates (M3)

Measured in `sim-cli cage` with league-average and population-sampled matchups, **all counts reached naturally**, bases empty (B1). Rates use FanGraphs definitions; zone uses the sim's own ABS zone (D11).

| ID | Metric | Target [R] | Envelope |
| :--- | :--- | :--- | :--- |
| B1.1 | K% (per PA) | 22.5% | 15–28% |
| B1.2 | BB% | 8.3% | 5–12% |
| B1.3 | HBP% | 1.1% | 0.4–2.0% |
| B1.4 | Ball-in-play% (incl. HR) | ~68% | ≥ 60% |
| B1.5 | Pitches per PA | 3.9 | 3.5–4.3 |
| B1.6 | Zone% (pitches in the ABS zone) | 48% | 40–55% |
| B1.7 | First-pitch strike% | 61% | 54–68% |
| B1.8 | Swing% | 47% | 40–54% |
| B1.9 | Z-Swing% | 67% | 58–76% |
| B1.10 | O-Swing% (chase) | 29% | 20–36% |
| B1.11 | Contact% (per swing) | 76% | 70–83% |
| B1.12 | Z-Contact% | 85% | 79–91% |
| B1.13 | O-Contact% | 57% | 45–68% |
| B1.14 | Swinging-strike% (per pitch) | 11% | 8–15% |
| B1.15 | Foul share of contact | ~50% | 40–60% |
| B1.16 | Called-strike share of strikeouts | ~25% | 12–38% |

### 5.1 Count shape checks (direction, hard)

- Zone% on 3-0 > zone% overall > zone% on 0-2.
- Swing% on two strikes > swing% on 0 strikes; O-Swing% on 0-2 > O-Swing% on 3-0.
- Batter wOBA after 1-0 > after 0-1 (count leverage exists).
- Pitches per PA rises with batter `Discipline` and falls with pitcher `Command` (§7).

### 5.2 Matchup shape checks (direction, hard)

- **Platoon:** same-handed pitcher holds a wOBA advantage of **+0.015 to +0.045** [R ≈ 0.025] over the opposite-handed pitcher, all else equal. This must come from release geometry and arm-side movement, not from a handedness term.
- **Times through the order:** batter wOBA rises from the 1st to the 3rd time facing a pitcher in the same game by **+0.010 to +0.040** [R ≈ 0.020], from `Adaptation` and pitcher fatigue together. Measured with fatigue off to separate the two: adaptation alone must give a positive, smaller rise.
- **Tunneling:** agent-models §11 tunneling test passes.

---

## 6. Tier B: Ball-in-Play and Game Totals (M4–M5)

Measured with `sim-cli season` on the reference park and a full generated league, **AI managers at default personality**, learned tables converged (§9.4). Per team-game unless stated.

### 6.1 Ball-in-play outcomes (B2)

| ID | Metric | Target [R] | Envelope |
| :--- | :--- | :--- | :--- |
| B2.1 | BABIP | .291 | .270–.320 |
| B2.2 | BABIP on grounders | ~.240 | .200–.280 |
| B2.3 | BABIP on line drives | ~.680 | .600–.760 |
| B2.4 | BABIP on fly balls (non-HR) | ~.130 | .080–.180 |
| B2.5 | HR / fly ball | 11–12% | 7–16% |
| B2.6 | Extra-base hits / hits | ~33% | 25–42% |
| B2.7 | Triples per team-game | ~0.13 | 0.05–0.30 (must be > 0) |
| B2.8 | Inside-the-park HR | rare, non-zero over a season | 0 < rate < 1 per 100 team-games |
| B2.9 | GIDP per GIDP opportunity | ~11% | 7–16% |
| B2.10 | Runner from 1st to 3rd on a single | ~28% | 18–40% |
| B2.11 | Runner from 2nd scores on a single | ~60% | 45–75% |
| B2.12 | Runner on 3rd, < 2 outs, scores on an outfield fly out | report only (M5) | — |

### 6.2 Game totals (B3)

| ID | Metric | Target [R] | Envelope |
| :--- | :--- | :--- | :--- |
| B3.1 | Runs per team-game | 4.4 | 3.5–5.5 |
| B3.2 | AVG / OBP / SLG | .243 / .312 / .400 | AVG .225–.275, OBP .295–.345, SLG .360–.450 |
| B3.3 | HR per team-game | 1.1 | 0.6–1.6 |
| B3.4 | Hits per team-game | 8.0 | 7.0–9.5 |
| B3.5 | Stolen-base attempts per team-game | 0.95 | 0.40–1.60 |
| B3.6 | Stolen-base success | 79% | 65–85% |
| B3.7 | Pitches per team-game | 147 | 130–165 |
| B3.8 | Left on base per team-game | 6.6 | 5.5–8.0 |
| B3.9 | Extra-inning games | ~9% | 5–14% |
| B3.10 | Shutouts per team-game | ~5% | 2–10% |

### 6.3 Watchability (B3, owner-facing)

These exist because "MLB-like" and "fun to watch" are not the same thing. They are **envelopes only**; the owner sets targets at M4 after watching games.

| ID | Metric | Envelope | Why |
| :--- | :--- | :--- | :--- |
| W1 | Three-true-outcome share (K + BB + HBP + HR) / PA | ≤ 40% | Above this, too few plays involve fielders |
| W2 | Balls in play per team-game (excluding HR) | ≥ 22 | The fielders are half the show |
| W3 | Plays with a runner moving on contact, per game | ≥ 10 | Baserunning drama |
| W4 | Margin of victory: share of one-run games | 22–35% | Close games exist |
| W5 | Blowouts (margin ≥ 8) | ≤ 12% | |
| W6 | Lead changes after the 6th inning, per game | report only | Owner reviews at M4 |
| W7 | Longest single play (sim time, contact to dead ball) | ≤ 40 s | No endless rundowns or loops |

---

## 7. Tier S: Sensitivity and Direction

### 7.1 Monotonicity (hard, property tests)

Owned by agent-models §11; the sensitivity tool asserts them. Each is run as a sweep of one attribute from grade 20 to 80 in steps of 10, all else at grade 50, with paired seeds (same RNG keys for every step) so noise cancels.

### 7.2 No dead attributes (gate)

For each attribute, the 20 → 80 sweep must move its **primary stat** by at least the threshold below, measured with the sample size in §9.2. An attribute that moves nothing is either unwired or swamped, and either way is a bug.

| Attribute | Primary stat | Minimum 20 → 80 effect [E] |
| :--- | :--- | :--- |
| `ArmSpeed` | K% | +4 pp |
| `SpinCapacity` | Whiff% on fastballs | +3 pp |
| `Command` | BB% | −4 pp |
| `Extension` | Whiff% on fastballs | +1 pp |
| `DeliveryTime` | SB success against | −8 pp |
| `Stamina` | Pitches before fatigue band 2 | +40 |
| `BatSpeed` | Mean EV | +10 mph |
| `QuickHands` | O-Swing% | −3 pp |
| `BatControl` | Contact% | +8 pp |
| `Timing` | Contact% | +6 pp |
| `Adjustability` | Contact% on off-speed | +3 pp |
| `Speed` | Home-to-first | −0.6 s |
| `Acceleration` | SB success | +6 pp |
| `Agility` | Range (outs above average per 150 games, IF) | +10 |
| `Reaction` | Range (OAA per 150, OF) | +10 |
| `Hands` | Errors per 100 chances | −2 |
| `ArmStrength` | Runner advance rate on OF singles | −10 pp |
| `ArmAccuracy` | Throwing errors per 100 throws | −1.5 |
| `Transfer` | Pop time | −0.25 s |
| `Blocking` | WP + PB per 100 IP | −4 |
| `Vision` | Contact% | +5 pp |
| `SpinRecognition` | Whiff% on breaking balls | −6 pp |
| `Discipline` | O-Swing% | −10 pp |
| `ReadSkill` | OAA per 150 (OF) | +6 |
| `JumpRead` | SB success | +5 pp |
| `BaserunningIQ` | Outs on the bases per 100 times on base | −1.5 |
| `FieldingIQ` | Wrong-base throws per 100 chances | −1 |
| `PitchCraft` | Batter wOBA against | −0.015 |
| `Composure` | wOBA-against difference, high vs low leverage | narrows by ≥ 0.010 |
| `Adaptation` | Times-through-order wOBA rise | +0.010 |

These thresholds are [E] first guesses. M3 measures the real elasticities and the owner reviews any attribute whose effect is much larger than the rest (it would dominate player value).

### 7.3 Elasticity record

`sim-cli sensitivity` writes, for each (attribute or knob, stat) pair, the slope per 10 grade points with a 95% interval. The record is versioned with the report (§12). A sign flip between two reports, or a slope change of more than 50% without a model change in the diff, is investigated before the gate.

---

## 8. Tactic Direction Checks (M6 gate)

PLAN §14 M6: "A/B sims show each tactic changes outcomes in the expected direction." Each check runs the same games twice with paired seeds, with the tactic forced on versus off at every eligible decision point. M6 will add exact situations; the expected signs are fixed here.

| Tactic | Situation | Must move | Direction |
| :--- | :--- | :--- | :--- |
| Infield in | Runner on 3rd, < 2 outs | Runs scoring from 3rd on grounders; BABIP on grounders | ↓; ↑ |
| Outfield deep | Late, leading | Extra-base hits; singles | ↓; ↑ |
| Shift toward pull side | Pull-heavy batter | BABIP on pulled grounders; on opposite-field grounders | ↓; ↑ |
| Steal (green light) | Fast runner on 1st | SB attempts; runs per inning vs slow runner | ↑; sign depends on runner (report) |
| Hit-and-run | Runner on 1st, contact batter | GIDP; strikeout–caught-stealing double plays | ↓; ↑ |
| Sacrifice bunt | Runner on 1st, 0 out | P(runner reaches 2nd); expected runs | ↑; ↓ for average hitters (report) |
| Intentional walk | 1st base open, strong batter | Runs this inning vs this batter's expected | situation-dependent (report) |
| Pull tired pitcher | Fatigue band ≥ 2 | wOBA against the next 3 batters | ↓ (if a fresh reliever is average) |
| Platoon pinch hitter | Same-handed matchup | wOBA of the PA | ↑ |
| Pitch around / attack | Count directive | Zone%, BB% | ↓/↑ as directed |

"Report" rows have no required sign; their measured effect is shown to the owner because the right answer depends on the situation, which is the point of managing.

---

## 9. Measurement

### 9.1 Run types

| Run | Command | Population | Use |
| :--- | :--- | :--- | :--- |
| Mechanism lab | `sim-cli cage --mode <pitch|swing|contact|run|throw>` | Single agent at grades 20/50/80 | Tier M |
| Cage | `sim-cli cage --pitches N` | Population-sampled batter × pitcher pairs, bases empty, fresh pitchers | B1 |
| Sensitivity | `sim-cli sensitivity --attr X` | Grade sweep, paired seeds | Tier S |
| League | `sim-cli season --teams 12 --games 162` | Generated league, reference park | B2, B3, W, B4 |
| CI smoke | `sim-cli season --teams 4 --games 20` | Small league | Regression only (§12.2) |

### 9.2 Sample sizes

A rate p from N independent trials has standard error √(p(1−p)/N). Sizes are set so the standard error is at most about **one tenth of the target band's half-width**.

| Level | Minimum sample | Example standard error |
| :--- | :--- | :--- |
| B1 rates | 50,000 PA (~195,000 pitches) | K% ±0.19 pp |
| B1 count splits | 5,000 PA per count | |
| Tier S per sweep step | 20,000 PA or 2,000 team-games, paired | |
| B2/B3 | One 972-game season (1,944 team-games), 3 seeds for gates | R/G ±0.07 per seed |
| B4 spreads | 10 seasons | team win% SD ±0.004 |

Per-game metrics are clustered (plays within a game are correlated), so their intervals are computed by game-level bootstrap, not by the binomial formula.

### 9.3 Seeds

Calibration uses a fixed, versioned seed list (`calibration/seeds-v1.txt`). Gate runs use the first three; development runs may use any. A band that passes on one seed and fails on another is reported as failing.

### 9.4 Learned tables and calibration

Learned belief tables (agent-models §8) depend on the run environment, and the run environment depends on the tables. Rules:
- After any knob or population change, re-fit the tables (agent-models §8 learning loop) **before** reading B1–B4.
- A calibration report records the table hash it ran with (`ProducerInfo.belief_tables_hash`, data-formats §3).
- If re-fitting moves a band metric by more than half its target band, the tables have not converged; run more learning rounds.

---

## 10. Milestone Gates

| Milestone | Must pass (envelope) | Warnings allowed |
| :--- | :--- | :--- |
| **M1** | A1–A6, oracle agreement, §4.1 pitch shapes (target, not just envelope) for the four core types (FF, SI, SL, CH), release envelope | Remaining pitch types inside envelope only |
| **M2** | A7–A12, §4.3 swing envelope, §4.4 shape checks and EV slope | §4.4 rate targets (cage is scripted at M2) |
| **M3** | §4.2, §4.4 envelope with live batters, all of §5 envelope, §5.1 and §5.2 direction checks, §7.1, §7.2 for pitcher and batter attributes | B1 targets ("rough Tier B bands", PLAN §14) |
| **M4** | Accounting and liveness over 1,000 games; B3.1, B3.2, B3.3 envelope; W1, W2, W7 | Everything else in §6; owner judges watchability |
| **M5** | A13; §4.5 and §4.6 envelope; all of §6 envelope; §7.2 for fielding and running attributes | §6 targets |
| **M6** | §8 signs | |
| **M7** | B4 spreads (§11.4) envelope over 10 seasons | |

---

## 11. Population Distributions

Agent-models §10 gives each attribute's grade-50 value and spread in physical units. This section owns how a league population is **drawn** from them, and the spreads it must produce.

### 11.1 Drawing attributes

- The generator draws a **latent grade** per attribute: `g = 50 + 10·z`, z standard normal, truncated at ±3 (grades 20–80) [C]. The physical value comes from the grade through agent-models §1.2. Drawing in grade space keeps "less is better" attributes log-normal in physical units, which matches how reaction times and noise levels are distributed.
- **Position and role adjustments** shift the mean grade before drawing (§11.2).
- **Correlations** are applied by drawing z jointly with the correlation matrix in §11.3 (Cholesky factor, fixed order of attributes, so the RNG key layout of data-formats §2.3 is stable).
- **Body:** height ~ N(1.88 m, 0.06 m) [R: MLB roster average ≈ 6 ft 2 in]; mass from height by BMI ~ N(27.5, 2.0) kg/m² [R]. Height and mass are not graded.
- **Handedness** [R]: pitchers throw left 28%. Batters: bat right 58%, left 33%, switch 9%; right-handed throwers bat left 20% of the time, left-handed throwers bat left 90%. Catchers and infielders other than 1B throw right.

### 11.2 Position and role mean shifts (grade points) [E]

| Group | Shifts |
| :--- | :--- |
| Starting pitcher | `Stamina` uses the starter distribution (agent-models §10); `Command` +3 |
| Reliever | `ArmSpeed` +4, `Command` −3, `Stamina` reliever distribution |
| C | `Speed` −10, `Acceleration` −5, `Transfer` +10, `ArmStrength` +5, `Blocking` +10 |
| 1B | `Speed` −8, `Agility` −5, `ArmStrength` −8, `BatSpeed` +5 |
| 2B | `Agility` +5, `Transfer` +5, `ArmStrength` −3 |
| SS | `Agility` +8, `Reaction` +5, `ArmStrength` +6, `Hands` +4, `BatSpeed` −3 |
| 3B | `Reaction` +6, `ArmStrength` +6 |
| CF | `Speed` +8, `ReadSkill` +6, `Reaction` +4 |
| LF / RF | RF `ArmStrength` +6; LF `ArmStrength` −3; both `BatSpeed` +3 |
| DH | `BatSpeed` +6, `Speed` −6, fielding attributes −8 |

Batting attributes are drawn for pitchers too but never used (universal DH, D10).

### 11.3 Correlations [E]

Only the correlations below; everything else independent. All values are first guesses for M7 to review.

| Pair | ρ | Reason |
| :--- | :--- | :--- |
| `Speed`–`Acceleration` | +0.6 | Same muscle |
| `Speed`–`Agility` | +0.4 | |
| `ArmSpeed`–`SpinCapacity` | +0.2 | |
| `ArmStrength`–`ArmAccuracy` | −0.1 | Hard throwers spray a little |
| `BatSpeed`–`BatControl` | −0.25 | Power-contact trade-off; keeps "power hitter" and "contact hitter" archetypes distinct |
| `BatSpeed`–`Mass` (via grade) | +0.3 | |
| `Vision`–`SpinRecognition` | +0.3 | |
| `Discipline`–`SpinRecognition` | +0.2 | |
| `Speed`–`Mass` (via grade) | −0.3 | |

### 11.4 Spreads the population must produce (B4, M7)

Season-level spreads are an output of attribute spread plus physical noise. They show whether talent matters (spreads too narrow) and whether luck matters (spreads too wide or results that are too predictable).

| ID | Metric | Target [R] | Envelope |
| :--- | :--- | :--- | :--- |
| B4.1 | SD of AVG, batters with ≥ 500 PA | .025 | .018–.035 |
| B4.2 | SD of K%, batters with ≥ 500 PA | 5.5 pp | 3.5–8 pp |
| B4.3 | League HR leader | 45–55 HR | 30–70 |
| B4.4 | SD of ERA, pitchers with ≥ 150 IP | 0.75 | 0.5–1.1 |
| B4.5 | SD of team win% | .070 | .050–.090 |
| B4.6 | Correlation of player wOBA, first half vs second half (≥ 250 PA each) | ~0.35 | 0.20–0.55 |
| B4.7 | Pythagorean exponent fit to runs scored/allowed | ~1.83 | 1.6–2.1 |

B4.6 is the luck check: if halves agree almost perfectly, noise is too small and outcomes feel scripted; if they barely agree, talent is swamped.

---

## 12. Reports and Regression

### 12.1 Calibration report

Every calibration run writes a directory `calibration/reports/<date>-<git-short-hash>/` with:
- `metrics.parquet`: one row per metric ID with value, interval, sample size, target band, envelope, status (`pass`, `warn`, `fail`).
- `summary.md`: failing and warning metrics first, then the full table, then knob values, population config hash, belief-table hash, seed list, and any §2.3 item-3 parameter changes with their justifying band.
- `sensitivity.parquet` when Tier S ran.

Bands are machine-readable in `calibration/bands.toml`, keyed by the metric IDs in this document (A1…, B1.1…, W1…). A test asserts that every ID in this document appears in `bands.toml` and the reverse, so the doc and the config cannot drift.

### 12.2 Continuous integration

- **Every commit:** Tier A goldens; monotonicity property tests at small N; the CI-smoke league with only accounting, liveness, and very loose sanity bounds (R/G between 1 and 12).
- **Nightly or on demand:** the full gate run for the current milestone.
- **Gates:** the full run on three seeds; the report goes to the owner with the milestone.

---

## 13. Open Items

1. **R10 (new):** refresh every [R] value against FanGraphs league pages, Baseball Savant leaderboards, and Baseball-Reference for the latest complete season, and record the sources here. **Before M3.**
2. All §7.2 thresholds, §11.2 shifts, and §11.3 correlations are [E] first guesses; M3 (batting and pitching) and M5 (fielding and running) revise them from measured elasticities.
3. Watchability targets (§6.3) are set by the owner at M4 after watching games.
4. Park effects: B3 bands assume the reference park. M7 adds a park-factor check (each parametric park's run factor between 0.85 and 1.15) once parks exist.
5. Fatigue and bullpen usage bands (innings per start, reliever appearances) depend on the AI manager and are added at M6.
6. The "routine play" definitions in §4.6 are geometric first drafts; M5 may replace them with a catch-probability model fitted from the sim's own output.
