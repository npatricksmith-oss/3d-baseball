# Agent Models Specification

**Last updated:** 2026-10-31
**Status:** Draft v1. Gates milestone M3 (batter vs pitcher); the fielding and running sections gate M4–M5.
**Parent:** [PLAN.md](../PLAN.md) §7, §8, §10.4 · **Siblings:** [physics-spec.md](physics-spec.md), [data-formats.md](data-formats.md), `calibration-targets.md` (P3), `rules-spec.md` (P4)

This document defines how every autonomous agent **perceives, believes, decides, and executes**: pitchers, batters, catchers, fielders, runners, base coaches, and the AI manager. It also defines the attribute-to-parameter mappings, the learned belief tables, and the belief snapshots each model must emit for play explanations.

Status tags: **[S]** sourced, **[D]** derived, **[E]** estimate (a named, configurable parameter, never a literal in code), **[C]** design choice the owner may revisit.

**D1 reminder.** Agents may hold probabilities, expected values, and learned tables. Nothing in this document decides an outcome. Agents produce **motor commands**; the world resolves them with physics. Every number below either shapes a belief, shapes a decision, or sets the size of physical noise on a motor command.

---

## 1. Common Framework

### 1.1 The agent loop

Every agent runs the same four stages, at its own rate:

```
Observation (bb_observe)  →  Belief update  →  Decision  →  Intended motor command
                                                                  │
                                              Execution (noise)   ▼
                                                           Executed command → world (bb_world)
```

- **Observations** are the only input. They are types in `bb_observe`, built by `bb_game` from world state plus perceptual noise. Agents never see a `bb_world` type (PLAN §4.1, §15).
- **Execution** is a separate module, `bb_agents::motor`. It takes an *intended* command and applies execution noise with the keyed RNG ([data-formats §2.3](data-formats.md)). Its output type, `Executed*`, is consumed only by `bb_game`, which passes it to the world. **Decision code never receives an `Executed*` value**, so an agent cannot know its own error except by observing the result like anyone else. This is enforced by module visibility and checked in review.
- **Rates [E]:** batter perception 100 Hz from release to contact; fielder and runner re-planning 30 Hz; pitcher and manager decisions once per play.
- **Determinism:** agents run in a fixed order (by `AgentId`) within each tick. No agent reads another agent's state directly; shared information goes through observations or explicit communication messages (§5.7).

### 1.2 Attributes: internal units and the 20–80 scale

Attributes are stored in **physical units** (PLAN §7). The 20–80 scouting grade is a display transform over the league population for that attribute [C]:

- **"More is better" quantities** (speed, arm strength): `value(g) = μ + σ_pop · (g − 50) / 10`.
- **Noise and time quantities** (command σ, reaction time), where less is better and the scale is multiplicative: `value(g) = value_50 · ρ^(−(g − 50) / 10)`, with a per-attribute ratio ρ.
- Grades clamp to [20, 80] for display only; the stored physical value is not clamped.

μ, σ_pop, value_50, and ρ for every attribute are in §10. The population distributions themselves (correlations, position differences) belong to the generator and `calibration-targets.md`.

### 1.3 Noise conventions

- All execution and perception noise is **Gaussian, truncated at ±4σ** [C]. Truncation removes absurd outliers (a 6σ wild pitch from an elite pitcher) without affecting calibration.
- Effective noise is a product of independent factors:
  `σ_eff = σ_attr · K_global · F_fatigue · F_composure · F_situation`
  - `σ_attr` from the attribute (§10).
  - `K_global` is one of PLAN §13.2's global calibration knobs (perception, command, swing plane, fielder reaction). Default 1.
  - `F_fatigue` and `F_composure` from §1.5 and §2.5.
  - `F_situation` from the specific mechanism (a rushed throw, a short hop, a reach).
- Every factor is a named function in one module, so the sensitivity tool (PLAN §13.3) can sweep each one.

### 1.4 Agents' internal physics

Agents predict with the **same physics functions** as the world (`bb_physics`), fed with their own noisy estimates (PLAN §4.1). Two cost and realism rules:
- **Precompute per game, not per pitch.** Batter trajectory predictions use per-pitch-type tables and linear sensitivities built at game start (§3.3). Fielders use a **batted-ball atlas**: landing point, hang time, and descent path as functions of (exit speed, launch angle, spray, spin), built once per park and air density.
- **Agents do not model what they cannot perceive.** A fielder's prediction assumes the *typical* spin for the launch angle, not the true spin (§5.1). A fielder never predicts a bad hop (surface roughness, physics spec §9.3). These mismatches are how real misreads happen.

### 1.5 Composure and perceived leverage

Each agent perceives game leverage from its own beliefs: `L̂` is the leverage index looked up from the learned win-expectancy table (§8) for the agent's own reading of the situation. A rookie and a veteran see the same scoreboard, so v1 uses the true base-out-score state for `L̂` [C]; misreading leverage is a later option.

```
F_composure = 1 + κ_c · (1 − C) · clamp(L̂ − 1, 0, L_max)
```

- `C ∈ [0, 1]` is `Composure`. κ_c = 0.15 [E], L_max = 4 [E]. An average player (C = 0.5) in a leverage-3 spot gets 15% more noise; a C = 0.9 player gets 3%.
- Applies to **execution noise** (release, swing placement, glove, throw) and to **perceptual noise** (batter observations, fielder reads).
- It never shifts a mean. Pressure makes players less precise, not systematically worse in one direction. This is physiological stress, not a "clutch" modifier (PLAN §8.6).

---

## 2. Pitcher

### 2.1 Arsenal

Each pitch type `k` in a pitcher's arsenal is defined physically:

| Field | Unit | Meaning |
| :--- | :--- | :--- |
| `type` | enum | FF, SI, FC, SL, ST (sweeper), CU, KC, CH, FS, SV (slurve). Savant codes ([data-formats §9.4](data-formats.md)) |
| `speed_ratio` | — | Release speed / `ArmSpeed` (FF = 1.00, CH ≈ 0.88, CU ≈ 0.82) |
| `spin_rate_factor` | — | Spin rate / (fastball spin from `SpinCapacity`) |
| `tilt` | clock hours | Magnus direction at release, pitcher's perspective ([data-formats §9.3](data-formats.md)) |
| `efficiency` | 0–1 | Transverse fraction of spin |
| `gyro_sign` | ±1 | Direction of the gyro component |
| `command_mult` | — | Multiplier on the pitcher's base command σ for this pitch |
| `release_offset` | m (x, z) | Small per-pitch shift of the release point (a "tell" when large) |
| `usage` | table | Base usage by count group (ahead, even, behind, two strikes) and batter handedness |

The spin vector for a release velocity **v** is constructed from rate, tilt, efficiency, and gyro sign by inverting [data-formats §9.2](data-formats.md). That construction has a round-trip unit test.

### 2.2 Release point

Release position from body and delivery attributes [E, chosen to land in public ranges; check bands in §11]:

```
y_rel = 18.440 − Extension
z_rel = h_front + 0.65·H + ℓ·sin(ArmSlot)
x_rel = s_arm · (0.10·H + ℓ·cos(ArmSlot))        ℓ = 0.40·H
```

- H is height (m); `ArmSlot` is the angle above horizontal of the shoulder-to-hand line at release; `h_front` = 0.10 m [E] is the front-of-mound height at the landing foot.
- `s_arm` = −1 for right-handers (arm side is third base, −X), +1 for left-handers.
- Check: H = 1.88 m, slot 45°, extension 1.95 m → release (−0.72, 16.49, 1.85) m, i.e. Savant `release_pos_x` −2.4 ft, `release_pos_z` 6.1 ft, extension 6.4 ft. Typical.
- Release-point noise is part of execution (§2.3).

### 2.3 Aiming and execution

**Intent:** a pitch type `k` and a target `(x_t, z_t)` on the strike-zone plane (y = y_mid, physics spec §10).

**Aiming solve** (pitcher's intent, no noise): find release direction angles (yaw ψ, elevation φ) so that the nominal pitch (`speed_ratio·ArmSpeed`, pitch-type spin, release point) reaches the target.
- Initial guess from a per-pitch-type offset learned at game start (the "aim correction" for that pitch's break).
- Fixed-point update: `ψ ← ψ − Δx / D`, `φ ← φ + Δz / D`, with D the release-to-plate distance. Small-angle geometry makes this Jacobian nearly exact, so 2–3 flights converge to < 1 mm. No finite differences needed.
- Two stages (implementation, 2026-10-31): the search iterates with a 0.02 s flight step (`AIM_SEARCH_DT`) to 1e-5 m, then the answer is confirmed, and corrected if needed, with the world's pitch step (`DT_PITCH`, 0.002 s) to the 1e-4 m acceptance tolerance. At 0.02 s RK4 lands within ~3 nm of `DT_PITCH` at the plate for every arsenal type, so a warm-started solve costs one full-step flight plus a few at a tenth of the cost. The reported residual is always the full-step one. A batter's scouting models aim each believed shape from the pitcher's cached zone-center solve rather than re-solving the true pitch.
- Pitchers know their own pitches. The solve is exact for the intent (they aim correctly); misses come only from execution.

**Execution noise** (purpose `PitchRelease`), each independent:

| Quantity | σ | Notes |
| :--- | :--- | :--- |
| Yaw and elevation of release direction | σ_cmd · command_mult · F_fatigue · F_composure · K_cmd | σ_cmd from `Command` (§10); ≈ 1.6 cm at the plate per mrad |
| Release speed | 0.45 m/s [E] (≈ 1 mph) · F_fatigue | |
| Spin rate | 2% [E] | |
| Spin tilt | 3° [E] | Moves break direction pitch to pitch |
| Release point x, z | 2 cm [E] | |

Elevation noise and release-speed noise are physically coupled in real deliveries; v1 treats them as independent [C].

### 2.4 Pitch selection

At each decision point the pitcher picks `(k, target)` from candidates: each arsenal pitch × a 7 × 7 target grid spanning x ∈ [−0.45, 0.45] m and z from 0.15·H_b to 0.70·H_b of the batter's height [E].

**Believed value of a candidate:**

```
V(k, target) = Σ_j  π̃_j · E_exec[ RV(k, loc, count, batter_class, surprised = (j ≠ k)) ]
```

- `π̃_j`: the pitcher's **model of what the batter expects** (theory of mind). It is computed by running the batter's own prior-construction function (§3.1) on *observable* history only: the pitch sequence, the count, and the batter's visible swings and takes. It never reads the batter's internal state.
  - `PitchCraft` (w ∈ [0, 1]) sets how much of that history the model uses: `π̃ = w · π̃_history + (1 − w) · π̃_count_only`. A crafty pitcher sequences against what the batter just saw; a thrower only knows "hitters expect fastballs when behind."
- `E_exec[·]`: expectation over the pitcher's **own believed** command ellipse (a 9-point Gaussian quadrature over yaw and elevation). Wild pitchers therefore avoid aiming at edges they cannot hit.
- `RV`: the learned pitch run-value table (§8). The `surprised` key is what makes deception valuable.
- Lower V is better for the pitcher (run value to the batting team).

**Mixed strategy:** sample from `P(k, target) ∝ exp(−V / τ_sel)` with purpose `PitchSelection`. τ_sel = 0.01 runs [E]. Sampling makes the pitcher unpredictable, which a pure argmin would not be. This is consistent with D1: randomness picks what the pitcher *tries*, never what happens.

**Manager directives** add terms to V before sampling [E weights]:

| Directive | Term |
| :--- | :--- |
| AttackZone | +λ · P_exec(ball) |
| WorkEdges | −λ · edge_proximity(target) for in-zone targets; heavy penalty on the zone's center |
| PitchAround | +λ · P_exec(strike in heart of zone); targets outside the zone favored |
| InduceGrounder | −λ · (expected launch angle < 10° share), from the learned table |
| IntentionalWalk | handled by rules (no pitches thrown) |

**Bootstrap heuristic (M3, before tables exist):** usage-by-count sampling for `k`; targets drawn from count-based zone maps (behind → middle-low, ahead → chase regions). Replaced in M6 by the table-driven rule above.

### 2.5 Fatigue

Per-pitch effort `e = (v_rel / v_max)²` [E] (a max-effort fastball costs 1; a changeup about 0.8). Pickoff throws cost 0.3 [E]; warm-up pitches cost 0.4 [E].

```
f = Σ e / Stamina                        # Stamina: effort budget, "pitch-equivalents"
F_fatigue = 1 + k_σ · max(0, f − f₀)      # f₀ = 0.75, k_σ = 1.2 [E]
v_max(f)  = ArmSpeed · (1 − k_v · max(0, f − f₀))      # k_v = 0.06 [E]
spin(f)   = spin · (1 − k_s · max(0, f − f₀))          # k_s = 0.04 [E]
```

- Between innings, f recovers by `r_inn = 0.04` [E] (short rest).
- Emits `PhysicalLoad` events ([data-formats §4.3](data-formats.md)) with the effort amount.
- Multi-day recovery (`Recovery` attribute) belongs to M7. The model stays effort-based so it can feed physically grounded injuries later (D18).

**Bullpen warm-up** (M6): states cold → warming → ready → overwarmed. Ready after `n_warm` = 20 [E] warm-up pitches and at least 6 min [E] of elapsed game time ([data-formats §2.2](data-formats.md)). Each warm-up pitch adds effort, so a pitcher warmed twice starts the game partially fatigued. Ready decays to cold after 15 min [E] idle.

### 2.6 Holding runners and pickoffs (M5)

The pitcher compares the runner's observed lead with a learned "pickoff value" belief and chooses: deliver, pickoff throw, step-off, or vary hold time. `DeliveryTime` (first move to release) sets steal windows. A slide step trades 0.15 s [E] of delivery time for +20% command σ [E]. Full design deferred to M5 with balks.

---

## 3. Batter

### 3.1 Prior over pitch types

Before each pitch, the batter forms `π_k` over the pitcher's arsenal:

```
π_k ∝ usage_k(count, hand) · seq_k(history) · sit_k · exposure_k
```

- `usage_k`: the batter's belief about the pitcher's usage. v1: the true arsenal usage table (scouting reports are public) [C]. Scouting uncertainty comes later.
- `seq_k`: sequence expectation from this at-bat. After a pitch of type j, `seq_j` = 0.85 [E] (repeats are slightly less expected). A learned sequence table replaces this in M6.
- `sit_k`: the Ambush directive (or a batter's own sit decision at 2-0 / 3-1 [E]) multiplies the chosen type by `s_amb` = 3 [E] and renormalizes. Sitting on a pitch is a gamble the batter knowingly takes.
- `exposure_k`: from in-game adaptation (§3.8).

**Velocity anchoring:** the batter's expected release speed for type k moves toward the last seen pitch's speed: `μ_v,k ← μ_v,k + a_anchor · (v_last − μ_v,k)`, a_anchor = 0.2 [E]. That is the changeup-after-fastball effect, produced by the prior and nothing else.

### 3.2 Perception geometry

- **Eye point:** midpoint between the eyes in the stance, at height 0.86·H [E] and 0.75 m [E] toward the batter's side of the plate center, about at y = 0.3 m [E]. Comes from the batter's box position and handedness.
- **Release estimate:** at release, a noisy estimate of the release point (σ 5 cm [E]) and release speed (Weber fraction 5% [E] on the batter's expectation) starts every hypothesis.
- **Tracking blackout:** batters cannot follow the ball over the last part of its flight. Observations stop when the ball is within `d_blank` = 1.5 m [E] of the eye plane. Anything learned after that is unusable for the swing.

### 3.3 Hypothesis bank

For each pitch type k in the arsenal with `π_k > 0.01`, the batter maintains a **linear-Gaussian filter in plate coordinates**:

- **State:** `s_k = (x_p, z_p, t_p)`, where the ball would cross the plane y = y_mid, and when.
- **Prior:** mean from the batter's predicted trajectory for type k (§1.4 tables: nominal path from the release estimate, plus the expected spin for type k); covariance `Σ_k⁰` = within-type spread (§3.8) plus release-estimate uncertainty propagated through the precomputed sensitivities.
- **Observation model:** each observation is linearized around hypothesis k's current estimate (an extended Kalman filter). The Jacobian `H_k(t)` maps a change in `s_k` to a change in the observation at time t. *Implementation (2026-10-31):* `H_k` is analytic and recomputed at every observation. The predicted position is linear in every state except the arrival time, which enters through the flight progress, so `H_k` costs one nominal-path lookup; a unit test checks it against central finite differences. This replaces the "precomputed once per (pitcher, type) at game start" of the first draft. An earlier version built a finite-difference `H_k` per observation at about three times the cost.
- **Weights:** `w_k ← w_k · N(o_t; ô_k,t, S_k,t)`, the predictive likelihood, then normalized. A batter fooled early stays fooled until the observations diverge enough to move the weights.

**Observations every 10 ms** (purpose `BatterPerception`, `sub_index` = observation number), each with noise from §10:

| Observation | Model | σ |
| :--- | :--- | :--- |
| Angular position (azimuth, elevation) from the eye point | true angles + noise | σ_ang from `Vision` · F_composure · K_perc |
| Time-to-arrival from optical expansion | τ̂ = τ · (1 + ε), ε ~ N(0, w²) | Weber fraction w from `Vision` |
| Spin cue | observed spin direction and rate, as a likelihood over types via each type's spin signature | σ_spin(t_view) = σ_s0 / √(1 + t_view / τ_s); σ_s0 from `SpinRecognition` |

The spin cue updates **only the weights** (it says which type, not where). Angular position and time-to-arrival update both the weights and the per-hypothesis states.

**Posterior:** a Gaussian mixture `Σ_k w_k · N(s_k, Σ_k)`.

**Cost:** about 6 hypotheses × 3-D Kalman updates × ~25 observations: a few microseconds per pitch. No flight integration per pitch.

### 3.4 Commit time

```
t_commit = t̂_arrival − T_swing − T_latency
```

- `t̂_arrival`: weighted mean of the hypotheses' `t_p`.
- `T_swing` from `QuickHands` (§10): start of swing to contact, ≈ 0.15 s average [E].
- `T_latency` = 0.05 s [E]: decision to first motor output.
- The batter re-evaluates `t_commit` each observation and commits when the current time reaches it. For a 95 mph fastball (0.40 s flight), this leaves about 0.2 s of viewing before commit.
- **No swing starts before the commit** (built 2026-11-13, `Swing::start_no_earlier_than`). The planned swing is timed from the believed arrival, and execution noise and the later timing adjustment (§3.6) move it; a start that would fall before the commit is moved to the commit, the whole swing with it, so he meets the ball that much later. A start inside `T_latency` after the commit is allowed (a shorter load). Before this, about 0.15% of swings started up to 18 ms before the batter had decided to swing (64 games: 12 of 8,172 at the commit before, 13 of 8,199 on the game paths of the overrun change; Codex overrun-samples review).

### 3.5 Swing or take

At commit, from the mixture:

**Aim point** `a = (x*, z*, t*)` is the **mean of the highest-weight component** (the dominant mode), not the mixture mean (PLAN §8.1). If two components are close in weight (within 0.1 [E]) and their means are within 3 cm and 5 ms [E], use their weighted average.

**Value of swinging:**

```
EV_swing = Σ_k w_k · W(s_k − a, a, count)
```

`W` is the learned **swing-value table** (§8): the expected run value of a swing aimed at `a` when the ball actually arrives with offset `s_k − a`, integrated over the batter's own believed execution noise. It captures "a 3 cm miss low still makes weak contact; a 10 cm miss is a whiff."

**Value of taking:**

```
EV_take = Σ_k w_k · [ P_k(strike) · RV_strike(count) + (1 − P_k(strike)) · RV_ball(count) ]
```

`P_k(strike)` from component k's Gaussian against the zone (circle-rectangle test of physics spec §10, integrated over the Gaussian).

**Discipline** acts on calibration, not on vision [C]. The batter evaluates `EV_take` with a distorted strike probability:

```
logit(P̃) = c_D · logit(P) + b_D
```

`c_D` (sharpness; 1 is perfectly calibrated) and `b_D` (bias; > 0 means "everything looks like a strike") come from `Discipline` (§10). Low discipline chases because it *believes* bad pitches are strikes and decides crisply on that wrong belief.

**Decision:** swing if `EV_swing − EV_take > θ_dir`, where θ_dir comes from the batting directive:

| Directive | Effect |
| :--- | :--- |
| Normal | θ = 0 |
| Take | no swing this pitch |
| Patient | θ = +0.02 runs [E] |
| Protect | θ = −0.02 runs [E]; swing-value table uses "avoid strikeout" weighting |
| Ambush | prior sharpening (§3.1); θ = −0.01 [E] when the dominant type is the sat-on type |
| Elevate | aim point shifted below the ball's predicted center by δ_elev = 1.0 cm [E] (undercut → lift) |
| Bunt, HitAndRun | separate behaviors: bunt template (M5); hit-and-run forces a swing and favors contact |

### 3.6 Swing model

A **template in a contact-relative frame** (PLAN §8.3), authored and tuned in M2.

**Template** (per batting side), phase `s ∈ [0, 1]`:
- Hand (knob) path `H(s)`: an arc in the **swing plane**, radius ρ_h ≈ 0.30·H_scale [E], sweeping angle φ_h(s).
- Bat direction `D(s)`: in the swing plane at angle `φ_h(s) + λ(s)`, where the lag λ(s) releases smoothly from λ₀ ≈ −90° [E] to 0 at the contact phase s_c ≈ 0.75 [E], plus a small out-of-plane tilt.
- Scaled per batter by height and bat length.

**Timing profile:** `s(t) = S((t − t₀) / T_swing)` with S a monotone quintic whose rate peaks near s_c. The phase-speed profile is scaled so the sweet-spot speed at s_c equals `BatSpeed`. Contact away from s_c (jammed or out in front) happens at lower bat speed and at a different point on the bat. That is how getting jammed physically weakens contact.

**Placement** `θ = (planeTilt α, verticalOffset δ_v, reach δ_r, startTime t₀)`.
- Contact depth `y_c(x*)` is a template function: inside pitches are met out front (larger y), outside pitches deeper. Horizontal location is handled mainly through timing (PLAN §8.3).
- **Inverse:** given the aim point and t*, find θ so that the sweet spot `u_ss` passes through `(x*, y_c(x*), z*)` at time t*. A per-template table over (x*, z*) gives the initial θ; one or two Newton steps refine it. Microseconds.

**Execution noise** (purpose `SwingExecution`) on θ, from §10:

| Component | σ source |
| :--- | :--- |
| δ_v (vertical offset) | `BatControl` |
| α (plane tilt) | `BatControl` (fixed ratio to δ_v [E]) |
| δ_r (reach) | `BatControl` × 0.7 [E] |
| t₀ (timing) | `Timing` |

All widen with F_composure and K_swing. **M2 measures** the offset → (exit velocity, launch angle) map (physics spec §8.3); these σ values are then set so that a perfectly perceived pitch produces a realistic spread of contact quality. Perception error should dominate execution error for typical pitches [C].

**Mid-swing adjustment:** after commit, observations continue (until the blackout). The aim point updates from the posterior; θ moves toward the new inverse at bounded rates:
- δ_v and δ_r: ≤ `v_adj` from `Adjustability` (§10).
- Timing: phase rate may change by ≤ ±10% [E] (`SwingTemplate::adj_phase_frac`).
- Adjustment stops at `t_cut` = 60 ms [E] before t* (PLAN range 50–80 ms; `SwingTemplate::adj_cutoff_s`).

**Check swing:** if, before phase `s_abort` = 0.45 [E], the updated `EV_swing − EV_take` falls below `−θ_check` = −0.03 runs [E] (`batter::CHECK_THETA_RUNS`), the batter brakes: phase rate decays to zero over 40 ms [E]. Whether it counts as a swing is judged by the rules engine from the bat's pose (`rules-spec.md`). The swing model only reports the maximum bat rotation reached. *v1 as built:* the abort point is a fraction of the swing's start-to-contact time, 0.6 (`SwingTemplate::abort_time_frac`), not the phase 0.45; under v1's Gaussian speed bump the two differ, and changing it is a behavior change to validate.

**Bunt** (M5): a separate near-static template; the batter positions the bat on the predicted path and applies the bunt-grip COR multiplier (physics spec §8.4).

### 3.7 Body and hit-by-pitch evasion

- Body is a capsule set (head, torso, upper and lower arms, thighs, shins) scaled by height, posed from the stance and the swing phase.
- **Evasion:** if the posterior predicts the ball passing within 0.25 m [E] of the head or torso, the batter starts an evasion after 0.15 s [E] of latency: torso rotation and a step back, limited to 2 m/s [E] capsule speed. A committed swing suppresses evasion.
- Hit-by-pitch is decided by the world (capsule contact). The batter's belief only decides whether to move.

### 3.8 In-game adaptation

Each batter keeps a memory per opposing pitcher and pitch type: `n_k`, the number of type-k pitches seen this game (including from the on-deck circle at reduced weight 0.3 [E]).

- **Within-type spread** shrinks with exposure: `σ_k(n) = σ_∞ + (σ_0 − σ_∞) · exp(−η · n_k)`, η from `Adaptation` (§10). σ_0 is the batter's spread before seeing the pitcher; σ_∞ = 0.5·σ_0 [E].
- **Model correction:** the batter's expected spin for type k moves toward the observed spin cue of each pitch seen: learning rate η / (1 + n_k).
- This is the mechanism for the times-through-the-order penalty (PLAN §8.6). Nothing scripts it.
- Memory is part of `SaveState` ([data-formats §7.1](data-formats.md)).

---

## 4. Catcher

- **Receiving:** the catcher sets the target glove position at the intended pitch location (signaled by the pitcher's intent; the catcher and pitcher share the target, a battery convention [C]). Catching is a standard capture (§5.4) with the catcher's `Hands`. With ABS, framing does not exist.
- **Blocking:** for a predicted pitch in the dirt, the catcher moves glove and body capsules toward the predicted bounce path. `Blocking` (§10) sets lateral reach speed and reaction. A blocked ball deflects physically off the body capsules (physics spec §9.4). Wild pitches and passed balls emerge.
- **Throwing on steals:** transfer from `Transfer` (catcher distribution), throw from `ArmStrength` and `ArmAccuracy`. **Pop time is a derived display value** (transfer + throw flight to second base), not a separate attribute [C].
- **Foul tips:** a foul tip is "caught" only if the catcher's capture test succeeds on the deflected ball.

---

## 5. Fielders

### 5.1 Perception after contact

After contact plus `Reaction` time T_r, each fielder holds an **intercept estimate** of the ball's future path.

- **Estimated launch state:** exit speed, launch angle, spray angle, each with error `e(t)`. The fielder looks up the predicted path in the batted-ball atlas (§1.4) using the **typical spin for that launch angle** [E: from the M2 contact map], not the true spin. A ball with unusual backspin carries farther than the fielder expects. That bias is deterministic and physical.
- **Error process** (purpose `FielderPerception`, `sub_index` = tick): `e(t)` is an Ornstein–Uhlenbeck process with correlation time τ_c = 0.3 s [E], so the estimate drifts smoothly instead of jittering. Its standard deviation shrinks with viewing time:
  `σ_e(t) = σ_e0 / √(1 + (t − t_contact − T_r) / τ_read)`, τ_read from `ReadSkill`.
- **Depth is harder than direction:** the error covariance is stretched along the fielder's line of sight by `k_depth` = 3 [E]. Balls hit straight at a fielder are the hardest to judge, as in real baseball.
- Composure and `K_perc` scale σ_e0.

**Typical spin, built 2026-11-12 (`live.rs`, `typical_batted_spin_rpm`):** a ball straight off the bat is predicted with the spin the contact model typically gives at the launch angle the fielder estimates (his read of the velocity with the gravity drop since contact added back): the medians of `examples/batted_spin.rs` by 10° bin, from −7,060 rpm (topspin) at −25° to +6,470 rpm at 55° (3,290 at 25°, 4,800 at 35°). After a bounce, a deflection, or a carom off the wall: 600 rpm topspin. The interim model had assumed 1,800 rpm backspin for every rising ball, while the contact model gives fly balls 3,300–6,500 rpm, so every fly carried past the fielder's prediction and his target crept back all flight; with free turning that cost little, with momentum it cost about 1.5 runs per team-game. The table is a belief and must be refit when the contact model changes (R3 asks whether its spin is high).

**Interim read values (refitted 2026-11-13 with the plate cue, docs/calibration/2026-11-13-plate-cue.md):** position error 3.5 m at 30 m for outfielders (was 3.0, refitted with momentum 2026-11-12; 0.3 m infield); velocity error `READ_VEL_FRAC` 0.04 of speed per axis (was an unnamed 0.06).

**Where the interim read's error came from (measured 2026-11-13, `examples/read_error.rs`, docs/calibration/2026-11-13-popup-read-and-calls.md):** before the plate cue below, for an outfielder at his first look, one σ of the position-depth term moved his predicted landing point 22–24 m at every launch angle; the velocity terms 3–6 m; the typical-spin belief and the coarser flight step under 1.5 m. The interim model gave the observer no knowledge that the ball started at the plate (22 m of depth error on a ball 16 m from home, 10 m of lateral error on the same ball).

**The plate cue, built 2026-11-13 (`position_from_plate`, `plate_weighing`; docs/calibration/2026-11-13-plate-cue.md):** while a batted ball is still on its first flight off the bat, s seconds after contact, the observer knows where it was hit and when. Drag, Magnus and gravity act on the ball's velocity alone, not on where it is, so he flies his read of its velocity back s seconds with his own physics and the typical spin above: a ball going that fast now, having left the contact point s seconds ago, must be *there*. He weighs that position against his direct read in three dimensions by how good each is (precision-weighted, a Kalman update): the direct read with the error above (σ_p per lateral axis, ×3 in depth; across his line of sight one draw moves it sideways by σ and up by σ/2 together, as the simulator samples it); the plate's with his velocity read's error carried over s (σ_v·s, ×1.5 in depth, the same coupled draw across) plus the spin he cannot see, `PLATE_SPIN_M` = 1.5 m per axis at 1 s, growing as s^1.5 [E] (`examples/plate_cue.rs`: flown back with the true spin the position is exact; with typical spin it misses by a median 0.6 m at 0.5 s, 2.3 m at 1 s, 4–9 m at 2 s; line drives that stay up 3–4 s, 18–23 m). Early the plate wins in every direction, so an outfielder's first-look landing error is about 7 m (one σ), nearly all of it his velocity read: how hard and at what angle the ball was hit. As the ball comes nearer his direct read takes over (the position terms move the landing point 2–5 m in the last two seconds of a pop-up 30–40 m in front of him). After a bounce, a deflection or a wall, only the direct read is used. The spec's launch-state form (errors on exit speed, launch angle and spray, the atlas) is this, expressed through the velocity read; the atlas is not built. *Not kept:* a learned prior of how far balls at each launch angle have typically gone pulled every hard-hit ball toward the typical one (liner triples .069 → .086 in 512 games), and a depth-only version left the lateral error in place.

### 5.2 Intercept planning

At 30 Hz, from its current estimate, each fielder computes candidate intercepts:
- **Fly and line drives:** the earliest point on the predicted descending path, at catchable height (0.3–2.2 m [E]), reachable before the ball. Prefer arriving early and settling; if unreachable, the point that minimizes the remaining gap (dive candidate).
- **Ground balls:** sample the predicted ground path (the agent's own bounce/roll model, with no roughness) every 20 ms; the intercept is the earliest point where the fielder's reach time is ≤ the ball's arrival time minus a set-up margin 0.15 s [E]. A charging preference [E] favors intercepts closer to the infield when the time saved matters for the throw.
- **Reach time** uses the movement model (§5.3) from the current state, not a straight-line distance. *Built 2026-11-12 (`run_gap`, `run_time_to`; revised after the Codex momentum review #3):* in the goal's frame his velocity is u toward it and w across. With no sideways speed: the speed law from u (moving away: a stop at 6 m/s² first). With sideways speed, the turn comes first: w brakes off at 6 m/s² (w/6 s, drifting w²/12 m) while his speed toward the goal grows at a_lat, up to top speed; then the speed law over what is left, drift included. The test `reach_estimate_tracks_the_movement_step` checks it against the stepped movement over 85 cases (distance 2–40 m, every mix of toward, away and across): within ±0.35 s, RMS under 0.15 s. (The first version, which added the drift but let him accelerate straight at the goal, was up to 1.1 s optimistic.) The 0.15 s set-up margin was tried as a hard requirement and made things worse (it rules out plays he can make); it is not built.

### 5.3 Movement

A kinematic point with heading:
- Speed along the path follows `v(t) = v_max · (1 − e^(−t/τ_a))` from rest, with v_max from `Speed` and τ_a from `Acceleration`. Check: v_max = 8.23 m/s, τ_a = 1.0 s gives 27.3 m in 4.3 s, a typical home-to-first time once the swing follow-through is added [D].
- Turning is limited by lateral acceleration `a_lat` from `Agility`; sharp direction changes force deceleration.
- Steering: pursuit toward the current intercept, re-planned at 30 Hz. **First steps follow the first estimate**, so an early depth error produces a visible wrong first step (a `FirstStep` explanation fact).
- *Built 2026-11-12 (`fielder_step`):* velocity is a vector. Speeding up along his direction of motion follows the speed law above; slowing takes at most 6 m/s² (`BRAKE` [E]) and turning at most `a_lat` (`Agility`, 7 m/s² at grade 50), each up to the change wanted. So a reversal is a full stop (from 7 m/s: 1.17 s and 4.1 m), and a sharp turn at speed a curve. Until then fielders turned and reversed instantly at full speed. A non-chaser, or a chaser early for the ball, wants the speed that stops him on his spot at 6 m/s²; a late chaser runs through. Anyone slows into a turn he cannot make at speed: a spot d away at θ off his heading needs a curve of radius d/(2 sin θ), so his wanted speed is at most √(a_lat·d/(2 sin θ)) (a spot behind him: θ = 90°). Without this a late chaser who narrowly missed his spot looped round it. After catching a batted ball, and for a relay man catching a throw, the target becomes where braking stops him, v²/(2·6) ahead, and he brakes onto it (he plants; he does not run back to the planned spot). He is on his spot only once within 5 cm and below 0.5 m/s (`SETTLE_V`); faster, he carries on through it (Codex momentum review #1: arriving used to stop him dead).
- Dive: if the gap at the predicted catch time is between arm reach and arm reach + 1.5 m [E], the fielder may dive; a dive adds reach, inflates glove σ (§5.4), and costs 1.5 s [E] to get up before a throw.
  *Built 2026-11-13 (`try_captures`, `fielder_step`):* a dive carries the body: over 0.3 s (`DIVE_LUNGE_S` [E]) he travels toward the ball until his glove is at it (arm's reach short of the ball, at most the 1.5 m of dive extension), then lies there, still, until he is up. The lunge never crosses the wall: the body stops 0.3 m short of it (`BODY_R` [E]), which matters where the fair and foul fences meet at an inward corner. He throws from where he landed. A ball he holds while down is by the turf (0.3 m), not at chest height. The get-up runs from the dive. Until he is up the fielder cannot dive again, but he can still grab a ball that comes within arm's reach at or below dive height (1.2 m). A ball rimmed off his glove that stays close is the usual case (§5.4: bobbles that stay close can be re-fielded). Whichever attempt secures the ball, he can throw only once he is up: at the end of the get-up plus his transfer (for a thrown ball he takes, the receiver's shorter transfer, also from when he is up). After a rimmed ball he may try again 0.5 s later; after a clean miss, not until he is up. Before this, a fielder who rimmed a dive could dive again from the ground half a second later, and that catch started a second full get-up. On seed 7475 play 25, the CF held a single 2.4 s while the runner went from first to third.

### 5.4 Capture

The glove moves toward the predicted ball position at capture time, limited by a hand speed of 6 m/s [E] within the reach envelope (arm length 0.42·H [E] plus lean and dive extension).

At the capture instant, the glove pocket center has a Gaussian error in the plane perpendicular to the ball's velocity (purpose `GlovePlacement`):

```
σ_g = σ_hands · (1 + k_v·v_ball/40 + k_late·lateness + k_reach·reach_frac + k_hop·short_hop + k_dive·diving)
      · F_composure · K_hands
```

| Factor | Meaning | Default [E] |
| :--- | :--- | :--- |
| `v_ball/40` | ball speed in m/s, normalized to 40 | k_v = 0.5 |
| `lateness` | how late the final estimate settled relative to capture, s, normalized to 0.3 s | k_late = 0.8 |
| `reach_frac` | fraction of maximum reach used | k_reach = 0.6 |
| `short_hop` | 1 if the ball bounced within 1 m before capture | k_hop = 1.0 |
| `diving` | 1 if diving | k_dive = 1.0 |

**Outcome (pure geometry):** with `d` the distance from ball center to pocket center in that plane:
- `d ≤ r_pocket` (0.06 m [E]): **catch**.
- `r_pocket < d ≤ r_rim` (0.12 m [E]): **deflection**, solved by the physics collision model with glove e_n ≈ 0.15 (physics spec §9.4). Bobbles that stay close can be re-fielded.
- `d > r_rim`: **miss**. The ball continues and may hit the body capsules (a block).

**Sanity check [D]:** for a 2-D Gaussian, P(d > r) = exp(−r²/2σ²). A routine chance with σ_g = 1.8 cm gives P(miss pocket) = exp(−5.6) ≈ 0.4%, and some of those deflections are recovered. That fits PLAN §8.7's "routine plays well above 98%" with room for harder chances to fail far more often.

### 5.5 Transfer and throw

- **Transfer time** from `Transfer`, plus footwork: +0.2 s [E] when moving away from the target, +0.3 s [E] after a dive (on top of the get-up time).
- **Throw speed:** the fielder chooses a speed up to `ArmStrength`. With a large believed time margin, a lower speed improves accuracy [E: σ scales with (v/v_max)²].
- **Trajectory:** ballistic solve with drag and typical throw backspin (1500 rpm [E]), aiming at the receiver's chest target. Long outfield throws may choose a one-hop trajectory when it arrives sooner (the bounce is physical).
- **Execution noise** (purpose `ThrowExecution`): direction σ from `ArmAccuracy` · (1 + k_rush·rush + k_off·off_balance) · F_composure; speed σ 3% [E]. `rush` = how much the transfer was shortened below the fielder's normal; `off_balance` = 1 when throwing while moving away or after a dive.
- The receiver runs the standard capture test. A first baseman uses §5.8.

**Built 2026-11-12 (`bb_physics::throw`, `live.rs`):** throws fly with drag and 1500 rpm backspin, aimed by the thrower's own solve (the flatter elevation that reaches chest height at the target, found by stepping up from below the vacuum answer, since with drag and backspin the height at the target peaks before 35°; none past 35°). Every throw is released at full arm strength (`THROW_EFFORT` 1.0) until speed-for-accuracy is modelled; the old vacuum throw at 0.92 of the arm stood in for drag. Speed noise (3%) now moves where the throw comes down. The receiver starts on the spot the throw was aimed at; after his reaction time he reads the throw every 0.25 s (the batted-ball read model of §5.1's interim version: a noisy estimate of where it is and how it moves, flown on with his own throw physics) and goes to where he believes it is catchable, between scoop height (0.15 m) and 2.4 m, nearest that spot. He never sees the realized path (Codex review 2026-11-12 #1). *Since 2026-11-12 (momentum):* he weighs his read against the knowledge that throws come near where they are aimed: his believed catch point is the aim point plus k times the read's offset, k = σ_p²/(σ_p² + σ_r²), with σ_p = 5% of the throw's length (`THROW_MISS_FRAC` [E], at least 0.5 m) and σ_r his read's depth error now plus its velocity error flown to the spot. From far off he barely moves; as the read firms up he goes where it says. Without this a catcher's first read of a 57 m relay throw sent him 17 m behind the plate. Taking a throw at a bag, he keeps his foot on it when both the weighed catch point and his read alone are within 0.9 × the 1.5 m receiving reach (`BAG_REACH_FRAC`; §5.8's stretch); otherwise he goes to the weighed point (Codex momentum review #4: the weighing alone could pin him to the bag while a throw passed 1.7 m to the side). Fielders and runners judge throw times from a table of the same physics in a reference atmosphere (`ThrowTable`, a belief). One-hop throws are not chosen yet: a throw beyond the arm's reach leaves at 35° and bounces.

### 5.6 Throw decision

At the end of the transfer, the fielder evaluates each option (each base with a runner heading to it, the cutoff, hold the ball):

```
P̂(out) = Φ( (t̂_runner − t̂_ball) / σ_FIQ )
value  = P̂(out) · ΔRE_out + (1 − P̂(out)) · ΔRE_safe  − throw-error risk term
```

- `t̂_runner`, `t̂_ball`: the fielder's own estimates (runner speed estimated from observation; ball time from the agent's throw model).
- σ_FIQ from `FieldingIQ` (§10). ΔRE from the run-expectancy table (§8).
- Pick the best option; ties broken by a fixed option order.

### 5.7 Defensive coordination

**Runners, interim values in the live simulator (2026-11-11):** every runner accelerates with a 0.85 s time constant (not yet the grade-based `Acceleration` of §10), the batter starts 0.25 s after contact, runners take leads of 5.5 / 6.5 / 4.5 m off first / second / third and hold them until their first read of the ball, and with fewer than two out they go 5 m further and wait on a fly that might drop.

**Backups (built 2026-11-11, `Sim::assign_backups`):** every fielder with no other job backs up the play: one behind the man fielding the ball and one behind each base a throw is likely to go to, on the line of that throw and about 13 m beyond the base, never two at one base. Who goes where follows standard assignments (the pitcher behind third or home, the far outfielder behind second, the catcher behind first with the bases empty); the list was first written up in docs/calibration/2026-11-11-owner-watch-notes.md, and the table below is the current one. Each base job lists its candidates in order of preference, and the first one still free takes it:

| Ball | Bases | Behind the fielder | Behind second | Behind third | Behind home | Behind first |
|---|---|---|---|---|---|---|
| Outfield (fielded > 45 m from home) | empty | nearest free outfielder | far-side outfielder, else CF | **pitcher** (owner, 2026-11-13) | — (the catcher has left home) | catcher, on the line from second |
| Outfield | runner on first | nearest free outfielder | far-side outfielder, else CF | pitcher (from foul ground), else LF | — (the catcher covers home) | — |
| Outfield | runner on second or third | nearest free outfielder | far-side outfielder, else CF | — | pitcher | — |
| Infield | any | nearest free outfielder | CF, else far-side outfielder (runner on first only) | LF, else pitcher (runner on second only) | — | catcher, else RF |

The far-side outfielder is the left fielder on a ball to the right side (x > 0) and the right fielder otherwise. Bases-empty outfield ball, pitcher behind third: **the owner's decision of 2026-11-13** (docs/calibration/2026-11-12-pitcher-outfield-diagnosis.md, Part B). On a ball in the gap or down a line the batter-runner may try for three, and third is the base the throw would go to after second; the pitcher's other standard place, behind home, has no runner to play. Before this the pitcher was a fallback for second and almost never got a job, so he stood near the mound (after Part A) or ran toward the ball (before it). When the throw is actually released, `back_up_throw` moves the backup of that base onto the real throw's line; if nobody has that job, the nearest fielder with nothing to do takes it. **A fielder with no job (`Role::Idle`) does not chase the ball (2026-11-13):** whenever the defense calls or re-calls the ball, a fielder left without a chase, cover, cutoff or backup job pulls up where braking stops him (`stop_point`: v²/(2·6 m/s²) along his motion) and stands there until he is given a job. The same holds for every other way a job ends (Codex idle-fielder review #1, `go_idle`): a cutoff or relay man whose job is over (a throw goes past him, or an infielder has the ball), a dropped backup, and the bases-empty catcher leaving home all brake to a stop; any real cover or backup job is then given explicitly. This happens once, when the job ends, not every tick. Before this, `cover_roles` left in his target the intercept every candidate computes during the call, so idle fielders, most visibly the pitcher on bases-empty fly balls, kept sprinting toward the ball (docs/calibration/2026-11-12-pitcher-outfield-diagnosis.md, Part A). That drift had also been an unplanned safety net: an infielder called off a blooper kept running and caught the ones the new chaser missed. Without it more shallow flies and pop-ups drop (docs/calibration/2026-11-13-idle-fielder-target.md). **Peeling off (built 2026-11-13, `PEEL_BACKUP_M` = 5 m [E]):** an infielder (pitcher and catcher included) who believed he could catch a ball in the air, and hears a sure call by someone else, does not stop: unless a cover job takes him, he backs up the catch from `PEEL_BACKUP_M` short of where *his own* read has it coming down, on the side he is coming from (out of the caller's way and out of reach, close enough to field a ball that drops or pops out). While the call stands he makes no attempt to catch the ball, however close it comes (his read may be off, or he may still be backing away). He keeps the job across re-calls he is not part of; it ends, checked every tick, when the ball comes down, is caught or is thrown, and a loose ball is then re-called at once; a ball off a glove is a new ball and a new call. His own read, not the caller's target, because he backs up the ball, not the man: on a shallow pop-up his read is the better one. Since he is now near the ball, a re-call can hand it back to him when the caller's read falls apart (seed 29 play 135: the centre fielder sure-calls a pop-up he has read 40 m too deep, the shortstop drifts out, and when the centre fielder is no longer sure the shortstop calls it and catches it; before, he stood 20 m away and it dropped). Outfielders called off keep the existing job behind the fielder (deeper, on the ball's line from home).

**Cutoffs and relays (built 2026-11-12, `Sim::relay_job`, `assign_relay`, `update_relay`):** once a ball is down in the outfield more than 50 m from home (or in an outfielder's hands), one infielder lines up between it and the base the throw will likely go to: home with a runner in scoring position, third with a runner on first, otherwise second, raised to any base a runner is visibly heading past that. Toward home the first baseman is the cutoff (the third baseman on balls to the left side); toward third, the shortstop; with no runner to play, nobody. On a ball beyond 80 m the shortstop (left and center) or second baseman (right) goes out as the relay man instead, early if his own read says he cannot get there before it lands. Only a ball an outfielder fields gets one; while the ball is loose, a man whose own read says he gets there before the chaser plays it himself; once a throw goes past him, his job ends and he is free to back it up. He stands short of the base by the throw's length less 45 m, at least 14 m (about 45 ft) and at most halfway, and keeps to the line as it moves. The fielder with the ball compares, from his throw-time belief, a direct throw with one through the relay man (his throw, the relay man's quick release, the relay man's throw) and takes the quicker; a throw beyond his reach on the fly always goes through. With no play he gets the ball to second the quicker way. Measured on 512 games (after the review fixes): relays add 0.14–0.15 runs per team-game against none (two seed sets), all of it from the deep relay man and none from routing throws through him; the cause is not yet known (docs/reviews/2026-11-12-codex-throws-relays-review.md, Resolution). The first version, which pulled infielders out while a catchable fly was in the air, doubled hits on pop-ups and was fixed.

**Calls, built 2026-11-12, revised 2026-11-13 (`Sim::call_value`, `assign_among`, `recall`):** each fielder judges from his own read. On a ball he believes he catches in the air with at least 0.3 s to spare (`CALL_SLACK_S` [E]) he goes for it, and he **calls it only when he is sure**: his time to spare must also cover `CALL_SURE_SIGMAS` = 1 [E] σ of his own read's depth error where the ball comes down, run off at his speed (`landing_depth_sigma`; a fielder knows how good his read is): from what he believes the ball is doing now he runs his own read (§5.1) once as it is and once with each of its four error draws at one σ, and adds up in quadrature how far each moves where he predicts it comes down, along his line of sight, with the spin he cannot see in the plate's share of his depth. Running his own physics carries drag, the plate cue's use of the same velocity he flies forward, and the coupled lateral and height draws. *Revised 2026-11-13 with the plate cue:* the first version used only the error of where the ball is now (σ₀ · d/30 m · 3 / √(1 + view/τ_read)), which was the whole story when that term was 22 m at first look; with the plate cue it is under a metre early, and what an outfielder is unsure of is the velocity. A straight-line sum of position and velocity terms (the cue's first version) missed that the plate position and the forward flight share one velocity error, and ignored drag (Codex plate-cue review #1); the per-draw version is within 15% of the sampled spread of his reads (test `a_fielders_doubt_matches_the_spread_of_his_reads`). Sure calls rank outfielders over infielders, then the most time to spare. Until someone is sure, the man with the most time to spare is the chaser and **everyone else who believes he can catch it in the air keeps going for it too**; when a call comes they peel off to their cover and backup jobs. Otherwise (grounders, balls nobody can catch in the air) the earliest believed intercept wins, outfielders taking near-ties (0.15 s). *Why (2026-11-13, docs/calibration/2026-11-13-popup-read-and-calls.md):* the first version let an outfielder call any ball he believed he could catch, at his first look, over every infielder. At that look his depth read is about 25 m (one σ), so on shallow pop-ups he often called a ball he had misjudged, the infielder who had it stopped, and it dropped (pop-up hits .047 against MLB ≈ .02). Outfield priority now applies as it does on a field: to the outfielder who is sure, which for a ball in between means late, coming in on it. Once a second the defense re-calls among the chaser and the four fielders nearest **where the chaser is going** (not the ball's position now, which for a pop-up is over the infield); the chaser keeps the ball unless someone calls it in a higher tier, or in his tier with 0.3 s more to spare (`RECALL_MARGIN_S` [E]). The earlier rule, earliest intercept for everything, was a coin toss on fly balls (everyone catches when it comes down) and handed shallow pop-ups to outfielders who had misjudged them. Infielders break for a ground ball without a call only if their read has it coming down within 25 m **in the first second** (`GROUNDER_BOUNCE_S` [E]; an infield pop-up also lands within 25 m, seconds later), and they run at their own intercept (the rule used to leave them aimed at the base the cover step had just given them).

Coordination emerges from **calls**, not a central oracle [C]:
- Each fielder, from its own estimate, computes whether it can make the play. When its believed margin is positive, it "calls" the ball. A call reaches teammates after `T_call` = 0.3 s [E].
- Conflicts resolve by a priority table (CF > corner OF; OF > IF on balls in between; SS > 3B on balls to the left, etc. [data file]). A fielder who hears a higher-priority call peels off to back up.
- Two fielders who both commit before a call arrives can converge, and the world resolves the result physically (both reaching, a collision later). Rare mix-ups emerge.
- **Where the holder throws, built 2026-11-12, revised 2026-11-13 (`holder_act`):** for each running runner he judges the chance of an out at the base the runner is heading for, p = Φ((runner's time − ball's time)/0.12 s), from what he can see. Plays with p > 0.3 are scored p² × w and the highest wins. Squaring p makes a sure out beat an uncertain play on a lead runner. With fewer than two out, w weights the lead runner: 0.9 / 1.0 / 1.1 / 1.3 for first / second / third / home. With two out every out ends the inning, so w is 1.0 for a force and 0.9 for a tag play (`TWO_OUT_TAG_W` [E]). A force needs no tag, and no run can score on a third-out force (OBR 5.08(a)), so an equally sure tag loses to the force. A sure tag still beats a doubtful force. Before this, on seed 7475 play 26 (runners on first and third, two out, a chopper to the pitcher), the pitcher threw home for a tag play instead of taking the force at first.
- **Coverage and cutoffs** come from situation tables (base-out state × ball direction → who covers which base, who cuts off). These are data, authored from standard baseball defensive assignments, and are beliefs the coaching staff holds.

### 5.8 First base

- **Foot on the bag:** reach is a volume anchored at the base, extended by stretching. Stretching takes 0.25 s [E] and gains up to 0.6 m [E] of reach, which shortens the throw's effective arrival time.
- **Scoop:** a capture of a short-hopped throw, using k_hop.
- **Off-bag decision:** if the predicted throw is outside the anchored reach, the first baseman leaves the bag to catch it and attempts a tag if the runner is close [E rule: when believed P(tag) > 0.3], otherwise concedes the base.

### 5.9 Positioning

Pre-pitch home positions come from alignment presets (normal, corners in, double-play depth, and so on; PLAN §10.1) stored as per-position offsets by batter handedness [data file], or from custom positions set by the manager. Fielders reach their positions between plays (no simulation of that walk).

### 5.10 Anticipation: the jump (owner request, built 2026-11-12)

The owner asked that fielders get a jump on the ball, anticipating where it will be hit from the pitch location and the batter's side, with a rating for it alongside range. `Anticipation` is that rating. Range is not a separate attribute: it comes out of `Speed`, `Acceleration`, `Agility`, `Reaction` and `Anticipation` together.

**Three reads, combined (`live.rs`, `jump_at_contact`; the third added 2026-11-13):**
- **Before contact: the pitch.** He reads where the pitch crosses the plate (horizontal, σ `antic_sigma`, purpose `FielderAnticipation` sub-index 0). With the batter's side it gives a prior on the ball's direction: the batter's typical pull (5° to his pull side, `MEAN_PULL_DEG`) plus the shift for that location (`anticipated_pull_deg`: inside pitches pulled up to 7.7° more, outside ones up to 6.5° less; means by 10 cm bin from `examples/spray_by_location.rs`, a belief about the contact model), scattered by 22° (`SPRAY_SCATTER_DEG`).
- **At contact: the bat.** He reads the bat's horizontal angle at the instant of contact (the barrel's yaw, knob to tip, `atan2(x, y)`), with error σ `swing_sigma` (degrees of bat angle, sub-index 1; derived below from how well a person can tell where a moving object was at the moment of an event). He reads the bat, not the ball: the orchestrator (`game.rs`) samples the executed swing at the actual contact time and passes the bat's true yaw into the play (`LiveInput::bat_yaw`); `jump_at_contact` adds each fielder's read error, and his belief (`jump_from_reads`) sees only that noisy angle, the noisy pitch location and the batter's side. The ball's launch direction is never an input before his reaction gate (§5.1). *Revised 2026-11-12 (Codex anticipation review #1):* the first version read the true outgoing spray plus noise, which carried the collision's result (spin and contact point move spray by several degrees at the same bat angle).
- **What the bat angle means (his belief).** The face angle is the barrel's yaw less 90° for a right-handed batter (mirrored for a left-hander; negative = his pull side): 0 when the bat is square to the pitch line. Over 5,386 fair balls in play (96 games, `examples/spray_by_bat_angle.rs`, 2026-11-12) spray ≈ 5.2° + 1.36 × face (`SWING_SPRAY_A_DEG`, `SWING_SPRAY_B` [E]), with 18.6° of scatter about the line (`SWING_SPRAY_RESID_DEG` [E]; r = 0.57; the face itself varies only 9.5° between swings). The scatter is the collision: where on the barrel and how far under or over the ball the bat met it. So the bat's cue for the spray has σ = √((1.36 σ_read)² + 18.6²): 56° at grade 80, 106° at grade 50, 204° at grade 20 (with the derived read σ below; the pitch prior is 22°, so the bat moves his estimate by 13%, 4% and 1% of the way to it). The fit is a belief and must be refit when the swing or contact model changes. It is a marginal fit, so fusing it with the location prior counts some of the location's information twice; the overlap is small (location moves spray about 7°, the face 9.5° × 1.36).
- **During the swing: the swing's lead (owner decision 2026-11-13, built the same day).** He times the bat's launch against the ball in flight: how long before the pitch reaches the plate the bat starts forward. The true quantity is the lead L = t_plate − t_start (s), with t_plate the pitch's free flight crossing the judgment plane and t_start the executed swing's start (`Swing::t_start`, 10% of peak angular speed); the orchestrator (`game.rs`) computes it and passes it into the play (`LiveInput::swing_lead`), and `jump_at_contact` adds each fielder's error, σ `lead_sigma` (purpose `FielderAnticipation` sub-index 2, truncated at 4σ). Over fair balls in play L has mean 159 ms and sd 8.2 ms, about twice the spread of the batter's timing at contact (4.1 ms), because it carries both halves of being early: starting early and meeting the ball out in front. 
  - *What it means (his belief).* L is a second read of the same thing the bat angle shows, the bat's face at contact: face ≈ 161.93° − 1,063.1°/s × L (`LEAD_FACE_A_DEG`, `LEAD_FACE_B_DPS` [E]), with 3.6° of scatter (`LEAD_FACE_RESID_DEG` [E]; r = 0.93; 11,588 fair balls in 200 games, `examples/spray_cues.rs`, 2026-11-13; refit when the swing model changes). So a lead read gives a face estimate with σ = √((1,063.1°/s × σ_lead)² + 3.6²), batter-relative (the lead is the same for either side; the face is mirrored for a left-hander as above). It is not a separate spray cue: the two face estimates (bat angle, lead) are fused precision-weighted on the circle into one, and only that is mapped to spray through the line above, once. Fusing them as two spray cues would count the batter's timing twice (the unit test `the_swing_lead_moves_the_jump_toward_the_side_it_reads` works both by hand: −22.06° against −25.7°).
  - *How well it can be read (error per grade).* The judgment is where a moving object (the ball, about 6 m out at the launch) was at the moment of an event on a different object (the bat starting forward): the task of Linares, Holcombe & White (2009) with the cue away from the moving object, their Exp. 1: 75 ms (CI 60–86). **Grade 50 = 75 ms** (`LEAD_READ_TIME_SIGMA_50_S` [E]); with the Anticipation ρ of 1.25, 38.4 ms at grade 80 and 146 ms at grade 20 (10 ms at the lab grade 160). The face error it implies: 41° at grade 80, 80° at 50, 156° at 20, about the same as the bat-angle read's (39°, 77°, 150°), so each roughly halves the variance the other leaves. The lead's twice-larger spread does not escape the limit: the Exp. 8 value (40 ms, a cue on the object's own path) does not apply, because the bat's launch is not on the ball's path. *Assumption, as for the bat angle:* a fielder's explicit timing is no better than a practised lab observer's.
  - *Honest information.* Both reads are timing judgments of the batter's timing, which in the sim varies by only 4–8 ms, against a fielder's 20–150 ms. With the lead added, a grade-80 fielder's best linear estimate of the spray from his noisy reads improves from 22.33° (pitch location alone) to 22.16° (with the bat angle) to 22.00° (with the lead too); at grade 50, 22.42° → 22.40° → 22.35° (`examples/spray_cues.rs`, 200 games). Every other honest cue measured carries less (below, "Cues assessed").
- **Cues assessed (2026-11-13; docs/calibration/2026-11-13-anticipation-cue.md).** Over 11,588 fair balls in play (spray sd 22.5°): the called pitch (type and intended location, from the catcher's signs) carries nothing beyond where the pitch actually crossed: type means differ by 1.3° at most and everything visible before contact together (location, called target, height, speed, count, type) leaves 22.02° against 22.12° for horizontal location alone; the speed change from the previous pitch, the count and the batter's earlier balls in play this game carry nothing measurable. Everything informative is the batter's timing (face, contact depth, timing offset, lead: r ≈ 0.51–0.58, each leaving about 18.3°, and jointly no better than one), and every honest read of it is a timing judgment: the bat angle (above), the contact depth relative to the plate (the ball moves 38 m/s, so 4.1 ms of spread against the same 20–80 ms reads), and the swing's lead. The lead carries the most per millisecond of reading error and is the one added.
- **Combining the reads.** He first fuses his two face reads (bat angle, lead) into one face estimate, then maps it to one spray cue, then combines that with the pitch prior. Each fusion is precision-weighted, on the circle: the estimate is the first moved by k = σ_1²/(σ_1² + σ_2²) of the **shortest** angle from it to the second, and offsets from his own line are shortest angles too, so nothing jumps at the ±180° seam (Codex anticipation review #3). **Back-half policy (revised 2026-11-13):** his model covers balls hit forward. A face read that by itself puts the ball more than 90° from straightaway (more than about 62° of face from square for the bat angle; a lead far outside the batter's range for the lead, whose face estimate is not wrapped, so a wild one cannot come back round into the forward half) is off his model: he **discards that read and falls back on the others**, the remaining face read if it is plausible and the pitch prior in any case. So an antipodal prior and cue never meet, and an implausible read never makes him stand still. (Until 2026-11-13 an implausible bat read made him stay set; with the derived read error that happened on about 39% of reads at grade 50, 10% at 80 and 66% at 20, so a poor reader leaned less often rather than leaning on what he did know. Coordinator decision 2026-11-13, following the owner's.) The discard rate per read is the same as before for the bat angle (normal approximation, face mean −7.4°, sd 9.3°) and about the same for the lead (its face error per grade is within 5% of the bat angle's); both reads are discarded together on about 1% of plays at grade 80, 16% at 50 and 44% at 20, and those fielders lean on the prior alone. If the estimate puts the ball within 20° (infield) or 25° (outfield) of his line from home but off to one side (`JUMP_ZONE_DEG`), he starts toward that side across his line, at top speed × min(1, offset / (2 σ_posterior)) (`JUMP_FULL_SIGMAS`) × `jump_gain` (1). A ball at him or well away from him, and he stays set. The pitcher and catcher do not jump. Without a bat read (lab set-ups with no swing) he leans on the pitch prior alone; without a pitch read nobody leans.
- **How well the bat can be read (derivation, owner decision 2026-11-13).** The bat's angle at contact cannot be seen in a frozen frame: it has to be paired with the moment the ball meets it, and the bat is turning fast. Three limits, from the sim and the vision literature:
  - *(a) Timing: when contact happened.* At contact the bat's horizontal angle sweeps at 1,915°/s (mean over 2,835 fair balls in 48 games, sd 143°/s, `examples/bat_read_geometry.rs`, 2026-11-13; `BAT_YAW_RATE_DPS` [E], a fixed property of the swing model, refit when it changes). An error of σ_t in pairing the bat's angle with the moment of contact is an angle error of 1,915°/s × σ_t. The task is judging where a single moving object was at the moment of a cue, against stationary references (the field), which is the task of Linares, Holcombe & White (2009, *J. Vision* 9(13):9, "Where is the moving object now? Judgments of instantaneous position show poor temporal precision (SD = 70 ms)"): the scatter of reported positions equals the distance the object moves in about 70 ms, nearly independent of the trajectory and the cue (Exp. 1, cue a colour change at fixation: mean 75 ms, CI 60–86); with the cue a transient on the object's own path (Exp. 8, rings flashed around the trajectory: closest to a ball striking the bat) it fell to 40 ms (CI 26–53). Brenner & Smeets (2010, *Vision Research* 50:1101–1108, "How well can people judge when something happened?") find the same order: synchrony and temporal-order SDs of 40–50 ms are typical, 20–30 ms at best; Brenner & Smeets (2015, *J. Vision* 15(3):8) summarise "temporal precision is 20 ms at best for simple visual tasks" (judging whether two stimuli were simultaneous or which came first). The much finer precision of interception (about 6–7 ms SD hitting a falling ball, Brenner, Driesen & Smeets 2014, *Front. Hum. Neurosci.* 8:342) and of aligning two bars moving together (about 5 ms, Chung, Levi & Bedell 1996, as analysed by Brenner & Smeets 2010) is not available here: the first is an action guided throughout, the second a relative position between objects moving together, and Linares et al. show the single-object judgment does not get that benefit. **Grade anchors:** grade 50 = 40 ms (`BAT_READ_TIME_SIGMA_50_S` [E]: Linares Exp. 8, a practised lab observer with the cue on the object). With the Anticipation ρ of 1.25 this gives 20.5 ms at grade 80 (the best published value for any explicit visual timing judgment, 20 ms) and 78 ms at grade 20 (Linares Exp. 1, the cue away from the object). The three published values sit on the ρ rule without adjustment. *Assumption:* a professional fielder's explicit timing is no better than the best lab values; there are no published measurements of fielders on this task. **Bat-angle read σ:** 39° at grade 80, 77° at grade 50, 150° at grade 20.
  - *(b) Space: distance and viewing angle.* A change δψ of the bat's angle moves the barrel by R·δψ sideways (R = 0.99 m, the contact point's mean horizontal distance from the swing's pivot), of which the fielder sees the part across his line of sight (fraction v; median 0.71 at first base, 0.49 third, 0.34 second, 0.14 short, 0.39 left, 0.14 centre, 0.60 right, at the standard positions; a square bat points its barrel's motion straight at the middle of the field). At the resolution limit of normal vision (1 arcmin, the 20/20 definition; *assumption*, generous for an object not tracked by the eye) the angle error from this is atan(1′ · D / (R · v)): 0.7° at first base, 5.4° at short, 11.5° in centre, at the median view. Added in quadrature to (a) it changes the grade-80 σ by at most 4% (centre field, 39.2° → 40.9°) and the grade-50 σ by at most 1%. Only in the edge-on tail (centre field's tenth percentile, v = 0.03: 44°) is it comparable. **So one σ per grade is the model; the viewing term is not built.** It becomes worth building only if the timing term is sharpened, e.g. by a cue that is read relative to something moving with the bat.
  - *(c) The pre-reaction window.* The read has to be made from the contact instant itself, before the reaction gate (0.25 s infield, 0.35 s outfield) opens the ball read of §5.1; nothing later can refine it. Light travel is negligible; the sound of contact reaches centre field after 0.28 s, too late to steer a jump that begins at contact, and it would only time the contact, not show the bat. The values in (a) are trial-to-trial SDs of single judgments, which is what a fielder has: one look per swing, no averaging over repeats.

  *Consequence:* the bat's angle at contact, read honestly, carries little: its spread between swings (sd 9.3°) is far smaller than even an elite read error (39°). A fielder's jump is driven mostly by the pitch prior. Before 2026-11-13 `swing_sigma` was 15° at grade 50 (7.7° / 29° at 80 / 20), set for the earlier read of the ball's spray and never re-derived for a bat-angle read.
- **Cover jobs are not protected (Codex anticipation review #4).** The angular zone does not know who will cover a base: zones overlap, cover roles are given only after the first read, and until his own reaction time ends a fielder keeps going with his jump even after he has been given a base to cover. A man who leans away from his bag pays for it with the stop and turn of §5.3, exactly like a man who leans the wrong way for a ball. That is accepted as part of the bet: anticipation trades a quicker first step on the balls he reads right against a later one on everything else, including his cover jobs. (The first version of this section claimed the zone kept a man with a cover job from being pulled off it; it never did.)

**Movement:** from contact until his reaction time ends he goes for that velocity with the speed law (§5.3); then he reads the ball (§5.1) and plans from where he is and how he is moving, so a misread costs the stop and turn of §5.3. The defense's first assignment is made at the quickest fielder's reaction time; a slower fielder is still carried by his jump then, so every plan made before his own gate opens (reach, the fallback time to the ball's resting point, the call's time to spare) starts from his position **and velocity** stepped through the rest of his jump with the movement step itself (`Sim::planning_state`; Codex anticipation review #2: the first version planned from where he was at the assignment, missing up to the last tenth of a second of his lean).

**Why the swing as well:** fielders leaning on the pitch location alone did worse than fielders who waited (2,048 games: 5.35 runs per team-game with no lean, 5.36 average, 5.45 at grade 80), and shading position before the pitch did too. Location moves the expected direction only about 7° against 22° of scatter, so for a ball in his area the side he leans to is right only a little more than half the time, and a wrong lean (carried through the reaction time, then braked at 6 m/s²) costs more than a right one gains. Most of the scatter comes from the swing's timing, which the bat shows. (Measured since the bat read replaced the spray read: see the anticipation-review calibration note, docs/calibration/2026-11-12-anticipation-review-fixes.md.)

**Measured with the swing's lead and the revised back-half policy (2026-11-13; seed 5, every fielder at one grade; docs/calibration/2026-11-13-anticipation-cue.md):** 1,024 games: no jump 5.57 runs per team-game (GB hit .412); Anticipation 20: 5.52 (.411); 50: 5.55 (.410); 80: 5.49 (.412); lab 160: 5.47 (.406). 4,096 games: no jump 5.55; 20: 5.56; 50: 5.58; 80: 5.51; 160: 5.49. The grades still do not rank on the 20–80 scale beyond whole-game noise; grade 80 may gain a few hundredths of a run. The lead is honest but, like the bat angle, is a timing judgment of a batter whose timing varies by 4–8 ms, so it adds little; the fallback to the prior gains nothing, because the prior alone is worth about what waiting is.

**Measured with the derived bat read (1,024 games, seed 5, base `2e715df`), every fielder at one grade, superseded:** no jump 5.57 runs per team-game (GB hit .412); Anticipation 20: 5.59 (.411); 50: 5.52 (.411); 80: 5.54 (.414); lab grade 160: 5.36 (.405). On the 20–80 scale the jump is now worth about 0.05 runs or less and the grades do not rank beyond whole-game noise: the bat's angle at contact cannot be read well enough to matter, and what remains is the pitch prior (docs/calibration/2026-11-13-bat-read-derivation.md).

**Measured with the bat read at the old σ (15° at grade 50), superseded:** no jump 5.34 runs per team-game (GB hit .410); Anticipation 20: 5.30 (.408); 50: 5.27 (.409); 80: 5.16 (.406). The honest cue carries about half the value the spray read did; nothing was retuned to recover it (docs/calibration/2026-11-12-anticipation-review-fixes.md).

**Measured with the first (spray) read (2,048 games, seed 5), superseded:** no jump 5.35 runs per team-game (GB hit .408); Anticipation 20: 5.31 (.406); 50: 5.22 (.399); 80: 5.03 (.384). `sim-cli games --anticipation <grade>` / `--no-jump`.

**Not built yet:** batters of both sides in games (every game batter is right-handed, so the handedness half is exercised only by the unit tests `fielders_jump_toward_the_side_they_read` and `the_swing_lead_moves_the_jump_toward_the_side_it_reads`); depth anticipation (the swing also hints at the launch angle). Pitch type (the called pitch) was assessed as a cue on 2026-11-13 and carries no spray information in the sim (type means within 1.3°; see "Cues assessed").

---

## 6. Runners and Base Coaches

### 6.1 Leads

- **Primary lead** distance chosen from the runner's belief of pickoff risk: `lead = base_lead(Speed) − k_pick · pickoff_threat_belief` [E], with an aggressiveness shift from the manager.
- **Secondary lead:** on the pitch, the runner shuffles out 1–1.5 m [E].
- **Jump:** the runner detects the pitcher's commitment to the plate after `T_jump` from `JumpRead` (purpose `RunnerPerception`). A runner who starts before the commit point on a pickoff move gets picked off physically (M5).

### 6.2 Running

The base path is a sequence of segments with a rounding arc at each base, radius set by speed and `Agility` [E]. Movement uses the §5.3 model. A slide decelerates at 6 m/s² [E] and lowers the body for tag geometry. A base is reached when the body touches the base volume (physics spec §9.4). Tag outs are geometric: glove-with-ball contact with the runner's capsules before the base touch.

*(M4, 2026-11-11: the batter starts in his box, not on the plate. His offset from the base path shrinks linearly to nothing over his first 4 m of running (`BOX_EXIT_M` [E]); tags, his view of the ball, and the recorded track all use that position. He is charged the base-path distance, a few centimetres short of the route's true length.)*

*(Built 2026-11-13, `Sim::step_overrun`: the overrun of first.)* A batter-runner who decides at first to stop there runs through the bag: he touches it, leaves it on the next tick, and carries on straight down the first-base line, braking from his crossing speed at `OVERRUN_DECEL` = 4 m/s² [E] (from 8.2 m/s he pulls up 8.4 m, 28 ft, past the bag), then walks back at `OVERRUN_RETURN_MPS` = 2.5 m/s [E] (reached with the running time constant) and touches it again. He makes no turn toward second, so the rules engine sees an overrun and he cannot be tagged out on the way back (rules-spec §5.5); the fielders see that too and make no play on him. The play does not settle until he is back on the bag. He makes no new decision while out there; one waiting on a runner ahead decides once he is back. Not yet modelled: a voluntary turn toward second on a hit and back (he decides at the bag to go on or run through; the turn is a right angle, not a curve), a slide into first, and a runner who sees an overthrow during his overrun and goes for second. A batter-runner who went on and is sent back because the runner ahead holds second already retreats to first along the path; the samples judge whether he went far enough to have made an attempt (1.5 m from the line, about 2.1 m of travel toward second), as rules-spec §5.5 describes. (Codex overrun-samples review #1.)

*Position samples (built 2026-11-13).* Every other 60 Hz tick (30 Hz, `SAMPLE_EVERY_TICKS`), after the runners move, the live sim sends each runner still on the base paths to the rules engine as a `RunnerPosition` at his recorded position, all at the same time (rules-spec §2). They are what lets the engine judge the overrun and passing between bases from positions (§5.5).

### 6.3 Advance decisions

At each decision point (contact, ball fielded, throw released, passing a base), the runner evaluates "advance" against "hold":

```
P̂(safe) = Φ( (t̂_ball − t̂_self) / σ_BRIQ )
advance if  P̂(safe) · ΔRE_safe + (1 − P̂(safe)) · ΔRE_out  >  ΔRE_hold + θ_aggr
```

- `t̂_self` uses the runner's own speed (known accurately). `t̂_ball` comes from observing the fielder and the ball, with noise from `BaserunningIQ`.
- `θ_aggr` comes from the team's running-aggressiveness setting.
- **Base coaches** make the send-or-hold call at third (and at second when the runner cannot see the ball), with a team coach perception σ [E, average] and the same rule. The runner follows the coach.
- **Fly balls:** the runner estimates P(catch) from the fielder's position and the ball (own estimate). High P(catch) → return and tag up; uncertain → go halfway; a likely hit → run.
- **Batter-runner** always runs; rounding first depends on the same rule.

---

## 7. AI Manager

Uses the identical command API (data-formats §6). v1 policy, detailed in M6:
- **Win expectancy and leverage** from the learned tables (§8).
- **Pitching changes:** uses only what a manager can observe: pitch count, the visible fatigue band, times through the order, matchups, the three-batter minimum (D17), and bullpen readiness.
- **Tactics** (steal, bunt, hit-and-run, intentional walk, alignment) chosen by comparing table-estimated run or win values, shifted by a **personality vector** [C]: analytics-driven, small-ball, quick-hook, and so on as weights on these terms.
- Emits `ManagerNote` snapshots with its believed win probability and reason (data-formats §5.5).

---

## 8. Learned Beliefs

Tables are agent beliefs (PLAN §8.5, D1). They are built offline from the sim's own Monte Carlo output, shipped as data files, and referenced by hash in `ProducerInfo.belief_tables_hash` (data-formats §3).

| Table | Key | Value | Used by |
| :--- | :--- | :--- | :--- |
| `RE` run expectancy | base-out state (24) | expected runs to end of inning | Everyone (ΔRE) |
| `CountRV` | count × base-out | run value of a ball, a strike | Batter take value, pitcher |
| `PitchRV` | pitch type × location bucket (13 × 13) × count group × batter class × surprised | expected run value | Pitcher selection |
| `SwingW` | aim error (Δx, Δz, Δt bins) × location bucket × count group × batter class | expected run value of the swing | Batter swing decision |
| `WE` win expectancy | inning × half × score diff × base-out | P(win) | Manager, leverage |
| `LI` leverage index | same as `WE` | leverage | Composure, manager |
| `ThrowOut` | throw distance × runner speed class × fielder arm class | P(out) | Fielder decisions (prior on σ_FIQ estimates) |

**Bootstrap (M3–M4):** hand-written heuristics and a public-style RE/WE table [E: standard run-environment values]. `SwingW` starts as a Gaussian contact-probability model over aim error with hand-set run values.

**Learning loop (M6):**
1. Run the cage and full games with the current tables.
2. Fit each table from the logged outcomes (event logs plus the per-pitch beliefs recorded in `SwingRecorded.batter_read`).
3. Replace the tables; repeat until the change in each table is below a tolerance (about 3–5 rounds expected).
4. Regenerate per league environment (M7).

Tables must be **smooth and monotone where physics says so** (more break never helps a hitter guessing the other type). Learning fits use shrinkage toward the previous round so AIs cannot lock onto an exploit of another AI's blind spot.

---

## 9. Belief Snapshots (fills data-formats §5.5)

Each model emits these only when a timeline is requested, and recording must not change behavior ([data-formats §5.5](data-formats.md)).

| Snapshot | Emitted by | Fields |
| :--- | :--- | :--- |
| `BatterBelief` | §3.3 every observation; §3.5 at commit | per hypothesis: type, w_k, s_k mean (x, z, t), Σ_k (6 floats). At commit: aim point, P̃(strike), EV_swing, EV_take, θ_dir, decision |
| `BatterAdjust` | §3.6 each tick from commit to cutoff | updated aim point, executed θ, adjustment rate saturation flags, check-swing state |
| `PitcherIntent` | §2.4 | chosen (k, target); top 3 alternatives with V; π̃ (the pitcher's model of the batter's expectations); directive |
| `FielderRead` | §5.1–5.3 each re-plan | estimated intercept point, σ_e along/across the line of sight, intercept time, role (primary/backup/cover/cutoff), call state |
| `ThrowDecision` | §5.6 | options with P̂(out) and value; chosen option |
| `RunnerDecision` | §6.3 | option, P̂(safe), ΔRE terms, coach signal |
| `ManagerNote` | §7 | believed WE, decision, reason code |

---

## 10. Attribute-to-Parameter Mappings

Defaults for an MLB-like population. **All values [E]** unless marked; `calibration-targets.md` (P3) owns the population distributions and their tuning. For "less is better" rows, the 80 and 20 columns use the ρ rule of §1.2.

| Attribute | Internal parameter | Unit | Grade 50 | σ_pop or ρ | Grade 80 | Grade 20 |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `ArmSpeed` | fastball release speed | m/s | 41.8 (93.5 mph) | σ 1.1 | 45.1 | 38.5 |
| `SpinCapacity` | fastball spin per unit speed | rpm per mph | 24.2 | σ 1.4 | 28.4 | 20.0 |
| `Command` | σ_cmd, release angle per axis | mrad | 10.0 | ρ 1.2 | 5.8 | 17.3 |
| `Extension` | release extension | m | 1.95 | σ 0.12 | 2.31 | 1.59 |
| `ArmSlot` | release arm angle | deg | 45 | σ 12 | (style, not graded) | |
| `DeliveryTime` | first move → release, from the stretch | s | 0.98 | ρ 1.06 | 0.82 | 1.17 |
| `Stamina` | effort budget | pitch-eq. | 95 (starters), 30 (relievers) | σ 12 / 6 | | |
| `BatSpeed` | sweet-spot speed at s_c | m/s | 32.0 (71.5 mph) | σ 1.1 | 35.3 | 28.7 |
| `QuickHands` | T_swing | s | 0.150 | ρ 1.05 | 0.130 | 0.174 |
| `BatControl` | σ of δ_v | cm | 1.2 | ρ 1.25 | 0.61 | 2.3 |
| `Timing` | σ of t₀ | ms | 6.0 | ρ 1.25 | 3.1 | 11.7 |
| `Adjustability` | v_adj | m/s | 0.6 | σ 0.15 | 1.05 | 0.15 |
| `Speed` | v_max (sprint speed) | m/s | 8.23 (27 ft/s) [S: Statcast average] | σ 0.46 | 9.61 | 6.85 |
| `Acceleration` | τ_a | s | 1.00 | ρ 1.08 | 0.79 | 1.26 |
| `Agility` | a_lat | m/s² | 7.0 (built 2026-11-12, §5.3) | σ 0.8 | 9.4 | 4.6 |
| `Reaction` | T_r | s | 0.25 (infield); 0.35 (outfield, interim) | ρ 1.12 | 0.18 | 0.35 |
| `Hands` | σ_hands | cm | 1.8 | ρ 1.2 | 1.04 | 3.1 |
| `ArmStrength` | max throw speed | m/s | 37.0 (83 mph) | σ 2.2 | 43.6 | 30.4 |
| `ArmAccuracy` | throw direction σ | deg | 1.2 | ρ 1.2 | 0.69 | 2.07 |
| `Transfer` | catch → release | s | 0.75 | ρ 1.08 | 0.60 | 0.94 |
| `Blocking` | catcher lateral reach speed | m/s | 3.0 | σ 0.4 | 4.2 | 1.8 |
| `Vision` | σ_ang; Weber w | mrad; — | 1.0; 0.10 | ρ 1.2 (both) | 0.58; 0.058 | 1.73; 0.173 |
| `SpinRecognition` | σ_s0; τ_s | (type-likelihood units); s | 1.0; 0.08 | ρ 1.25 (both) | 0.51; 0.041 | 1.95; 0.156 |
| `Discipline` | c_D; b_D | —; logits | 0.5; +0.9 (owner-directed calibration, 2026-11-11; were 1.0; 0.0) | σ 0.15; σ 0.25 | 0.95; +0.15 | 0.05; +1.65 |
| `ReadSkill` | τ_read | s | 0.6 | ρ 1.2 | 0.35 | 1.04 |
| `Anticipation` (§5.10) | σ of the pitch-location read; σ of the timing of the contact instant against the bat's sweep (× 1,915°/s = the bat-angle read σ); σ of the timing of the bat's launch against the ball in flight (the swing's lead; built 2026-11-13) | m; ms (deg); ms | 0.15; 40 (77); 75 | ρ 1.25 (all) | 0.077; 20.5 (39); 38.4 | 0.29; 78 (150); 146 |
| `JumpRead` | T_jump | s | 0.15 | ρ 1.12 | 0.11 | 0.21 |
| `BaserunningIQ` | σ_BRIQ | s | 0.15 | ρ 1.2 | 0.087 | 0.26 |
| `FieldingIQ` | σ_FIQ | s | 0.12 | ρ 1.2 | 0.069 | 0.21 |

Interim outfielder values (owner accepted 2026-11-11, `FielderAttrs::average`; docs/calibration/2026-11-11-fielding-planner.md): reaction 0.35 s, top speed while chasing 8.2 m/s, initial read error 2.5 m at 30 m (re-fitted after the left-field index fix; the owner had accepted 0.40 / 8.0 / 3.0). They were fitted to line-drive and fly-ball hit rates and stand in for the atlas perception of §5.1 until it is built; the grade-20 and grade-80 columns above are for the infield value.
| `PitchCraft` | w | 0–1 | 0.5 | σ 0.15 | 0.95 | 0.05 |
| `Composure` | C | 0–1 | 0.5 | σ 0.15 | 0.95 | 0.05 |
| `Adaptation` | η | per pitch seen | 0.15 | σ 0.04 | 0.27 | 0.03 |

Notes:
- Height, mass, and handedness are not graded.
- `Discipline` maps to two parameters with opposite signs: a high grade sharpens (c_D up) and removes the swing bias (b_D down).
- Bounded quantities (`PitchCraft`, `Composure`) clamp to [0, 1].
- PLAN §7.1 lists `PopTime` as a catcher attribute. Under §4 it is derived from `Transfer` and `ArmStrength`. PLAN §7.1 is updated to match.

---

## 11. Validation

Each mechanism gets tests that need no full game. Monotonicity tests feed `sim-cli sensitivity` (PLAN §13.2).

**Unit and analytic**
- Release point §2.2: the worked example; mirror symmetry between right and left-handers.
- Aiming §2.3: intended pitches hit their targets to < 1 mm for every arsenal type.
- Spin construction §2.1: round trip through data-formats §9.
- Batter filter §3.3: with zero observation noise and a single hypothesis, the posterior mean converges to the true crossing; with two identical hypotheses, weights stay equal.
- Swing inverse §3.6: noise-free swings put the sweet spot on the aim point to < 1 mm.
- Capture §5.4: the closed-form P(miss pocket) matches Monte Carlo for fixed σ_g.
- Movement §5.3: the home-to-first check.
- Execution isolation §1.1: a compile-fail test that decision code cannot name `Executed*` types.

**Monotonicity (property tests)**
- More `Command` → smaller mean miss distance. More `BatSpeed` → higher mean exit velocity. More `Vision` → smaller aim error. More `SpinRecognition` → fewer swings at the wrong hypothesis. More `Discipline` → lower chase rate. More `Hands` → fewer misplays. More `Speed` → shorter home-to-first. More `Composure` → smaller noise growth with leverage.
- Pitch tunneling: holding the release state fixed, a slider whose path matches the fastball's longer gets more whiffs from the same batter.
- Adaptation: whiff rate against the same pitcher falls with plate appearances (times through the order).

**Behavioral bands (checked in M3–M5, values in `calibration-targets.md`)**
- Pitcher release-point distributions in public ranges (release height 5.5–6.5 ft, side 1–3 ft).
- Zone rate, chase rate, whiff rate, contact rate, and pitches per plate appearance.
- Routine-play success > 98%; error rate in envelope.

---

## 12. Performance Budget

PLAN §13.4: ≤ 200 ms per headless game on one core, ~300 pitches and ~70 balls in play.

| Component | Budget per game [E] | Measured 2026-10-31 | Notes |
| :--- | :--- | :--- | :--- |
| Game-start precompute (batter tables, atlas lookups, aim corrections) | 10 ms | 8 ms | 18 scouting builds, seeded from the cached aim solves (was 23 ms) |
| Pitch selection | 15 ms | < 1 ms | |
| Aiming (2–3 flights per pitch) | 20 ms | 13 ms | One full-step flight plus coarse search flights, with release execution (was 21 ms) |
| Pitch flight (world) | 10 ms | 12 ms | |
| Batter perception and decision | 10 ms | ~25 ms | Filter updates with an analytic observation Jacobian (10 ms), decisions (7 ms), aim points from the nominal path's own depth crossing (3 ms); was 71 ms with a finite-difference Jacobian and a 1 ms path march |
| Swing and contact (world) | 15 ms | 8 ms | |
| Fielding, running, coordination (30 Hz) | 60 ms | ~70 ms | Live plays at 60 Hz, batted-ball flight and rules stepping included. Fielder predictions ~38 ms (2,563 per game; agents fly their predictions at 0.1 s, within a few µm of the world's step); intercept scans 4 ms. Was 114 ms |
| Batted-ball flight and bounces (world) | 20 ms | (above) | |
| Rules, scoring, event log | 10 ms | ~20 ms | Remainder: game loop, event log, timelines |
| **Total** | **≈ 170 ms** | **≈ 155–170 ms** | One thread, 259 pitches and 66 balls in play (seed 3); best of five runs, since timings vary widely with machine load. Within the 200 ms budget |

M3 measures the real numbers with `criterion`. If a component exceeds its budget, cache or table it first; never simplify the model in a way that changes behavior without a decision-log entry.

---

## 13. Open Items

1. All [E] values, especially §10, need calibration in M3–M5 against `calibration-targets.md`.
2. The swing template's exact shape (§3.6) is authored in M2 against published swing kinematics (attack angle, swing length, time to contact).
3. The perception noise values (σ_ang, Weber fraction, d_blank) need a literature pass on interception and time-to-contact studies. New task **R9**.
4. Execution noise correlations (release elevation with speed; swing timing with plane) are ignored in v1.
5. Pickoffs, balks, bunts, and steals are sketched here and designed fully in M5.
6. AI manager personalities and the full tactic set are designed at M6.
7. Leverage perception uses the true situation in v1 (§1.5). Misreading leverage is a later option.
