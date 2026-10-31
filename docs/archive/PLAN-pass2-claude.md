# 3D Physics-Based Baseball Management Game: Planning Pass 2

**Date:** 2026-10-30
**Inputs:** [brainstorming note (2026-10-26)](./2026-10-26--physics-based-baseball-game__note.docx), [PLAN.md (Pass 1, Gemini)](./PLAN.md)
**Status:** Planning only. Nothing here is implemented. Items marked **Proposed decision** still need the owner's sign-off.

Pass 1's broad architecture is sound and I keep most of it. This pass does five things:

1. Tightens the core invariant so it can actually be enforced (§1).
2. Corrects numbers and claims in Pass 1 that were wrong or unsupported (§2).
3. Answers the five review questions Pass 1 left for this pass (§3).
4. Specifies the riskiest subsystems (batter perception, swing model, bat-ball collision, fielding, AI beliefs, calibration) closely enough to build from (§4).
5. Reorders the roadmap so a crude but honest full game is watchable early, instead of perfecting each layer before the next one exists (§5).

---

## 1. Invariants, Restated

Pass 1's four invariants stay. Two of them need sharper wording.

### 1.1 The No-Cheat Invariant: beliefs versus world

The phrase "no outcome rolls" leaves a question open: may an agent think in probabilities? It must. A runner deciding whether to tag up has to estimate whether the catch will be made. A batter deciding whether to swing has to estimate the odds the pitch is a strike.

**Rule:** the code is split into two layers.

| Layer | May use probabilities? | Examples |
| :--- | :--- | :--- |
| **World** (physics, rules, umpire in ABS mode) | **Never** to decide an outcome. Randomness enters only as physical or physiological noise on *inputs*: release angles, swing-plane error, perceptual samples, bounce-surface normals. | Ball flight, bat-ball contact, glove capture, base-arrival timestamps, fair/foul |
| **Agent cognition** (players, managers) | **Yes**, freely. Beliefs, expected values, and learned tables are all fine. | "P(strike) = 0.62", "P(I beat the throw) = 0.4", run-expectancy tables |

The world never reads an agent's belief, and an agent never reads the world's true state. Agents receive only noisy observations. This is enforceable in code: agent modules get an `Observation` interface, never a `BallState`. A test can assert that `AI/` has no reference to the physics state types.

The official scorer's counterfactuals (hit or error?) are *bookkeeping* and sit outside both layers. They never change what happened.

### 1.2 Determinism: what is promised and what is not

- **Promised:** the same build, same machine, and same seed give the same game. This is required for debugging and regression tests.
- **Not promised:** bit-identical results across OS/CPU/.NET versions. `Math.Sin`/`Exp`/`Pow` can differ across platform math libraries. Pass 1's network idea (an authoritative host streams results) already avoids needing this. Keep it that way.
- **Consequence for replays and saves:** store the **recorded `PlayTimeline` and event log**, never "seed + inputs to re-simulate." Re-simulating breaks the moment SimCore code changes, which it will do constantly.
- **RNG:** use **counter-based keyed streams**, e.g. `Rng(gameSeed, pitchIndex, agentId, purpose)`, rather than long-lived sequential streams. Adding a new random draw in one subsystem then cannot shift every later number in the game. It also enables "re-run pitch 147 with only the swing noise changed" during debugging.
- **Parallel seasons:** each game's seed derives from `(seasonSeed, gameId)`. Games within one league day run in parallel. Cross-game state (fatigue, injuries, standings) merges at the day barrier.

### 1.3 New invariant: the event log is the source of truth for stats

Every statistic, including box scores, leaderboards, and Statcast metrics, is a **pure function of the per-game event log**. Nothing increments counters during simulation. This gives three benefits: new stats can be added later and back-filled; accounting identities become trivially testable; and the scorer can be fixed without re-simulating.

---

## 2. Corrections to Pass 1

| # | Pass 1 claim | Problem | Pass 2 position |
| :--- | :--- | :--- | :--- |
| C1 | **~18–30 ms per full game**; a 2,430-game season in 6–10 s | The table counts only world physics. The expensive part is *agent cognition*. Batters re-predict trajectories under several pitch hypotheses at each perceptual update. Pitchers score candidate locations. Fielders re-plan intercepts repeatedly. Plausibly 5–20× the physics cost. | Set a **budget**, not a promise: **≤ 200 ms per headless game on one core**, measured in M3. Even at 200 ms, a 12-team, 162-game season (972 games) takes about 25 s on 8 cores. That is fine. No design decision should depend on the 20 ms figure. |
| C2 | "A **6 mm** vertical offset separates a grounder, a 28° home run, and a pop-up" | Too tight by roughly 5×. The contact-normal tilt is about asin(D / (R_ball + R_bat)) with R_ball + R_bat ≈ 7 cm. That gives about 0.8° of normal tilt per mm near center, and the launch angle follows a blend of normal, attack angle, and friction. On the order of **2 cm separates a line drive from a lazy fly**, and about 4–5 cm spans grounder to pop-up. | Matters because it sets the **scale of batter spatial noise σ_plane**. Compute the real offset→launch-angle map in M2 and derive the noise scales from it. Do not copy numbers from either plan. |
| C3 | Spin decay "~15–25% over 5 s" | Unsourced. Published estimates vary widely. | Make it a named parameter with a cited baseline. Validate it indirectly through carry distance and hang-time reference cases (§4.6). |
| C4 | "3D pentagonal prism… **any graze** of ball against prism" for ABS mode | MLB's ABS system (in use from 2026) evaluates a **2D zone at the plate's midpoint depth**, with top and bottom set as fixed percentages of batter height. The rulebook's 3D prism is the traditional human-umpire concept. *(Verify current exact percentages before relying on them.)* | Fictional league, so either is allowed. **Proposed decision:** v1 uses the 2D midpoint-plane, height-scaled zone. It is simpler, testable, and familiar to modern fans. The 3D prism can be a league-rules option later. |
| C5 | Target `.NET 9` | .NET 9 is an STS release near end of support. Godot's .NET build also pins a supported TFM. | Target the **LTS .NET version that the current Godot .NET build supports** (likely .NET 10 by build time). Check this in M0. SimCore must load inside Godot, so Godot's TFM wins. |
| C6 | Phase 0 exit gate: "identical state hash across CLI runs **and live Godot playback**" | Godot only *plays back* a timeline that SimCore produced, so it cannot diverge by construction. The gate tests nothing. | M0 gate: same seed gives the same event-log hash across repeated CLI runs **and** across the in-process call from Godot. That catches accidental use of Godot's RNG or time. |
| C7 | "~50 fair/foul plays per game" | Real games have roughly 70+ balls in play plus about 30–40 fouls. Minor, but it feeds the cost estimate. | Use measured counts from M3 onward. |
| C8 | Coordinate frame | The frame is correct and right-handed, and matches Statcast's convention. It omits the **Godot conversion**: Godot is Y-up with −Z forward. | Boundary transform: `godot = (x_sim, z_sim, −y_sim)`. This is a proper rotation (det = +1). It lives in **one** function in the client and nowhere else. |
| C9 | Tone of certainty ("100% physically emergent", "zero stuck states") | Not errors, but the plan reads as more settled than it is. The note says no decisions are final. | This pass labels items **Proposed decision** or **Open**. |

---

## 3. Answers to Pass 1's Review Questions

### Q1. Custom domain physics versus a physics library (Jolt/Bepu)?
**Agree with Pass 1, and go further: SimCore needs no general rigid-body library at all.**

- Ball flight: an ODE with gravity, drag, Magnus, and spin decay, integrated with RK4. About 50 lines.
- Bat-ball contact: an analytic swept test of a sphere against a varying-radius bat axis, plus a 1D impulse model. A few hundred lines.
- Stadium: static geometry only. Ground is a height function (flat, plus a mound cone and slight grading). Walls and fences are vertical polyline segments with heights. Foul poles are cylinders. Bases are boxes. A **specialized query set** (ray/sphere against plane, polyline wall, cylinder) is simpler and faster than a BVH. A BVH only becomes worthwhile for irregular stands geometry, which only affects home-run and foul-out-of-play determination and can be approximated by a "stands volume" plane.
- Players are kinematic agents, not rigid bodies.
- Jolt's real advantage is stacking, joints, and dynamic-dynamic contacts. Baseball's world physics has none of those. Godot's built-in physics can still be used **in presentation only** for cosmetic effects: a dropped bat bouncing, a helmet flying off.

### Q2. Swing parameterization?
**Use a kinematic swing template in a contact-relative frame, not a double pendulum.** Full design in §4.2. In short, the swing is an authored curve, scaled per batter, placed in the world by four parameters (plane tilt, vertical offset, reach, start time). That makes bat pose and velocity analytic functions of time, which the swept collision test needs anyway. A biomechanical double-pendulum model can replace the template later behind the same interface if individual swing styles become a goal. The note lists those as future.

### Q3. Is the glove-aperture fielding model too binary or too chaotic?
Both risks are real. The fix is to keep the **outcome geometric** and put the **noise in motor execution**, exactly as batting does. Design in §4.4. Key points:
- **Too binary:** caused by deterministic hands. Add Gaussian glove-placement error whose σ grows with ball speed, late reads, and difficult approach angles, and shrinks with skill. The Gaussian tail then *is* the error rate. It can be tuned so routine chances succeed well above 98%, without any roll.
- **Too chaotic:** caused by bounce-normal perturbation and deflections that are too large. Keep surface perturbation small, scaled by surface quality. Damp deflections heavily, since a ball off a glove loses most of its speed. Validate with a "routine grounder" test suite: an average fielder on a manicured infield should almost never misplay a 70 mph two-hopper.
- **First base stretch and scoop:** model 1B as a fielder with a foot-on-bag constraint. Reach is a volume anchored at the bag, extended by stretching, which costs time but shortens the throw's effective arrival. A scoop is a catch attempt at a short-hop point whose glove-error σ is inflated. If the throw falls outside the anchored reach, he must choose: come off the bag (then attempt a tag or concede the base) or let it go. These are decisions plus geometry, with no rolls.

### Q4. Pre-solved `PlayTimeline` versus live tick-stepping?
**Agree: pre-solve.** Managers act only between pitches. Mid-play decisions (send or hold the runner at third, cutoff choices) belong to *in-world agents* (base coach, fielders) acting under the manager's pre-set aggressiveness policy. A pitch plus its play solves in milliseconds, which is imperceptible.

Downsides to accept knowingly:
- A future "interactive" mode (human swings or fields) would need live stepping. The note doesn't call for one. If it comes later, SimCore's stepper already exists; the timeline is just its recorded output.
- Pausing mid-play and changing something is impossible. That is fine for a management game.
- The timeline must contain *agent intent* (target points, belief snapshots), not just trajectories, so presentation can animate anticipation (a fielder drifting before the catch) and so the explanation feature (§4.7) works.

### Q5. Stack: C#/.NET + Godot versus Rust + gdext versus Unity?
**Proposed decision: C# SimCore + Godot 4 (.NET).** Reasons, beyond Pass 1's:
- Most of the code by volume will be rules, AI, league, and UI, where iteration speed matters more than raw speed. Performance-critical code is small and isolated: the integrator, the collision test, and the predictors. C# structs plus `Span<T>` are fast enough at the §2 C1 budget.
- If the inner loop ever becomes a bottleneck, it can move to a native library later without touching the architecture.
- Known caveat: Godot's C# projects historically **could not export to web**. Treat this as a desktop-only game unless that changes.
- Unity's animation tooling is better. Since animation is physics-driven and mostly procedural here, that advantage is smaller than usual.

---

## 4. Detailed Designs for the Risky Subsystems

### 4.1 Batter perception: a hypothesis bank

Pass 1 proposed a Kalman filter over raw ball samples. That is workable but undersells the mechanism that matters. **Batters are mostly classifying the pitch type, then estimating where it goes.** Model it that way:

1. **Prior over pitch types** `π_k` comes from the pitcher's arsenal and usage by count, the previous pitches in the at-bat, and (later) scouting knowledge. Batters can also "sit on" a pitch, which sharpens the prior on purpose. The manager's "ambush fastball" sign maps here.
2. **Per hypothesis `k`:** from the batter's noisy estimate of release position and velocity, predict the trajectory using type `k`'s expected spin. This prediction runs on the **batter's internal model**, which may itself be slightly wrong, and the error can be per-batter. Within-type variability (one slider breaks more than another) is a Gaussian around each prediction.
3. **Observations** every ~10 ms: the ball's *angular* position as seen from the batter's eyes, with noise σ_angle set by `Vision`; time-to-arrival from optical expansion, with Weber-fraction noise; and a **spin cue** (axis/seam pattern) with noise that shrinks with `SpinRecognition` and viewing time.
4. **Posterior** = a Gaussian mixture over (plate x, plate z, arrival t). Weights update by likelihood.
5. **Commit point:** at about (estimated arrival − swing duration − decision latency), decide swing or take from the mixture: P(strike), expected contact value at the chosen aim point, count, and manager directive. The **aim point is usually the dominant mode**, not the mean. That is what makes tunneling work: if the slider's trajectory is indistinguishable from the fastball's until after commit, the batter swings where the fastball would have been.
6. **After commit:** observations keep updating; the swing can be adjusted within limits (§4.2) or checked.

What emerges with no special-casing: tunneling, sitting on a pitch, sequencing effects, the value of spin recognition, and fooled check swings. Optional later extra: **perceptual anchoring**, where the previous pitch's velocity biases the next velocity prior. That is the changeup-after-fastball effect, and it falls out of the prior.

Cost: about 3–5 hypotheses × one trajectory prediction each, plus about 40 cheap likelihood updates per pitch. Tens of microseconds, inside the budget.

### 4.2 Swing model: template in a contact-relative frame

- A **swing template** is an authored curve parameterized by phase `s ∈ [0, 1]`: hand (knob) path `H(s)` and bat direction `D(s)`, where `D(s)` is mostly a wrist-uncocking angle in the swing plane plus a small out-of-plane tilt. Scaled per batter by body size and bat length. v1 has **one template** per handedness; styles come later.
- **Timing profile:** `s(t)` is set by swing duration (from `QuickHands`) and a velocity profile, so peak barrel speed at contact equals the batter's `BatSpeed`.
- **Placement parameters** `θ = (planeTilt, verticalOffset, reach, tStart)`. Horizontal location is mostly handled by *timing*: inside pitches are met earlier in the arc (out front), outside pitches later and with more reach. That is roughly what real hitters do.
- **Inverse solve:** given the aim point and the predicted arrival time, find `θ`. Precompute a lookup table per template at load (aim point → θ), refined with one or two Newton steps. Microseconds.
- **Execution noise:** Gaussian perturbation of `θ` with σ values from `BatControl` (spatial) and `Timing` (temporal). Their scale comes from C2's offset→launch-angle map.
- **Mid-swing adjustment:** after commit, θ can move toward the updated target at a bounded rate (`Adjustability`) until a cutoff about 50–80 ms before contact. The cutoff is a parameter.
- **Check swing:** brake the phase rate if the abort decision comes before `s_abort`. Whether it counts as a swing is judged geometrically (bat angle relative to the front of the plate), which is a rules question.
- **Bunt:** a separate, nearly static template. Not in the first skeleton.

Bat pose and velocity are analytic in `t`, so the swept collision test (§4.3) can query exact point velocities `v + Ω × r`.

### 4.3 Bat-ball collision (concretizing Pass 1 §3.2)

- **Contact detection:** within about ±25 ms of nominal contact, sample the signed distance `d(t) = |ball(t) − closestPointOnBatAxis(t)| − (R_ball + R_bat(u))` at 0.1 ms, then bisect the first sign change. About 500 cheap evaluations per swing. Fine.
- **Bat as a free body during the ~1 ms collision** (standard approximation; the hands don't matter). Effective mass at impact distance `b` from the bat's center of mass: `1/M_eff = 1/M + b²/I_cm`.
- **Normal direction:** `e(u)` is a COR profile peaking near the sweet spot (wood/BBCOR-like ≈ 0.50), dropping toward handle and endcap, with mild dependence on relative speed. With `r = m_ball / M_eff`, the collision efficiency is `q = (e − r)/(1 + r)` ≈ 0.2 at the sweet spot. **Sanity anchor:** exit speed ≈ q·v_pitch + (1+q)·v_bat. An 85 mph pitch at the plate and 75 mph bat speed give about 107 mph.
- **Tangential direction:** a sliding/rolling friction model (μ ≈ 0.5, plus a tangential restitution term), following Nathan et al.'s batted-ball spin work. This produces backspin on undercut balls, topspin on overcut balls, and sidespin from horizontal offset and bat angle.
- **Foul tips and grazing contact** come out of large offsets naturally. A foul tip "caught" by the catcher is then a catcher-glove capture test like any other.
- **Validation oracle:** before any agents exist, reproduce published collision relationships: q versus impact location, launch angle and spin versus undercut. See §4.6.

### 4.4 Fielding: perception, routes, capture

- **Perception:** after contact plus a per-fielder reaction delay, a fielder's estimate of the landing or intercept point comes from noisy observations of launch direction and speed. It sharpens over time, faster with `ReadSkill`. The fielder never sees the true landing point.
- **Movement:** acceleration- and turn-limited kinematics toward the *current estimate*, re-planned at about 20–50 Hz. Bad first steps and imperfect routes emerge from early estimate error.
- **Capture:** the glove moves toward the predicted catch point under a hand-speed limit. At the catch instant, the glove pose has Gaussian error (σ from `Hands`, inflated by ball speed, late reads, reaching, short hops, sun/wall later). Outcome is geometric: ball center within the pocket radius gives a **catch**; within the rim radius gives a **deflection** with a heavily damped, physically redirected velocity; outside gives a **miss**.
- **Throws:** solve the throw ballistically to the receiver's target with the arm's velocity, then add angular and speed noise (σ from `ArmAccuracy`, inflated by rushed transfers and off-balance positions). The receiver runs the same capture test, using the 1B rules in §3 Q3.
- **Coordination:** one defensive coordinator assigns the primary fielder, backups, coverage, and cutoffs from *beliefs*, not truth. Miscommunications (two fielders converging) can emerge, and that is acceptable.

### 4.5 Agent beliefs learned from the simulation itself

Pitchers need "what is the run value of throwing a slider here in 1–2?" Batters need "what is a swing at this location worth?" Runners need "do I beat this throw?" Those **beliefs** must come from somewhere. Proposal:

1. **Bootstrap** with simple hand-written heuristics: count-based target zones, pitch mix by count, fixed swing thresholds.
2. **Learn tables from the sim's own Monte Carlo output**, offline. Examples: run value by (pitch type, location bucket, count, batter archetype); out probability by (throw distance, runner speed, fielder arm). Ship these tables as data.
3. **Iterate:** agents now use the tables, outcomes shift, so re-learn. A few rounds should approach a rough equilibrium.

This is allowed under §1.1 because these are agent beliefs. It also prevents the classic emergent-system failure: AI exploiting a blind spot in another AI forever (e.g., every pitcher throwing low-away sliders). The batter's sequence-aware prior (§4.1) is the in-game counter-pressure. Learned tables are the offline one.

### 4.6 Calibration: two separate targets

Pass 1 merges these. They must stay separate:

| Tier | Must match reality? | Examples | Method |
| :--- | :--- | :--- | :--- |
| **A. Physics validation** | **Yes.** It's physics. | Pitch movement (induced vertical/horizontal break) versus spin for known pitch types; carry, e.g. about 100 mph at about 28° reaching ~400 ft at sea level in calm air; collision efficiency q ≈ 0.2 at the sweet spot; spin versus undercut. | Golden-value unit tests against a reference trajectory calculator (Alan Nathan's published trajectory calculator is the natural oracle) and published collision results. These are tested, not tuned. |
| **B. Emergent playability envelope** | **No.** The note explicitly lets the league differ from MLB. It must still be *watchable baseball*. | Runs per game, K%, BB%, batting average on balls in play, home runs per game, steals, errors per game, game length in pitches. Bands broad enough to allow a distinct league but excluding nonsense (e.g. K% outside roughly 10–35%). | Tuned only through **ability distributions and noise scales**, never through outcome code. |

Tier B method:
- Identify a **small set of global knobs**: league-wide perceptual noise scale, command noise scale, swing-plane noise scale, fielder reaction scale, bat-speed distribution mean.
- `sim-cli sensitivity`: sweep each knob and each individual attribute; assert **monotonicity** (Pass 1's idea, kept) and record elasticities (how much K% moves per unit of each knob).
- Use the elasticities to steer the envelope. If no knob combination reaches the envelope, a *model* is wrong. That is the signal to fix physics or perception, not to add a fudge factor.

### 4.7 New feature: causal play explanations

Every outcome has a physical cause, and the agents' belief snapshots are recorded. So the game can *explain* plays, which statistical sims cannot do:

> "Batter read fastball (78% confidence at commit). The slider broke 11 cm below his swing plane, so he topped it to short. Exit velocity 71 mph."

> "Center fielder's first step was in. The initial depth read was 9 m short, which cost about 0.4 s, and the ball landed 1.2 m past his reach."

This turns the simulation's depth into something the player can see. It also gives the manager feedback on whether their tactics worked, which emergent systems usually hide. It is a cheap add-on given §4.1's belief bank, provided timelines record belief snapshots (§3 Q4). **Proposed:** treat it as a core feature, not polish.

---

## 5. Revised Roadmap

Main change: a **walking-skeleton full game at M4**, using crude but honest (no-roll) versions of fielding, running, and rules. Pass 1 builds fielding, then baserunning, then rules, then the manager layer before a full game exists, so integration problems and "is this fun to watch?" both surface late. The note's PoC step 3 (check for plausible emergent behavior) needs whole games to judge.

| Milestone | Scope | Exit gate |
| :--- | :--- | :--- |
| **M0: Foundations** | Solution layout (SimCore / Cli / Tests / Godot client); SI vector math; keyed RNG; event-log and `PlayTimeline` formats; minimal Godot scene playing a hand-built timeline with time scrubbing; coordinate transform. | Deterministic event-log hash across CLI runs and the in-Godot call; timeline plays at 0.1×–4×. |
| **M1: Pitch flight** | RK4 ball flight (gravity, drag, Magnus, spin decay); pitch definitions; pitcher aiming inverse solve plus release noise; Godot pitch viewer with trails and a strike-zone overlay. | Tier A golden tests pass against the reference trajectory calculator for a set of pitch types; tunnel visualization shows fastball and slider sharing early flight. |
| **M2: Bat and contact** | Swing template plus inverse table; swept contact; impulse model; batted-ball flight on an infinite flat field. "Cage" viewer showing contact point, EV, LA, spin, and spray. | Tier A collision tests pass; scripted sweeps show early→pull, late→opposite field, undercut→backspin fly, overcut→topspin grounder, off-barrel→low EV; offset→LA map recorded to set noise scales (C2). |
| **M3: Batter vs pitcher** | Minimal v1 attributes (§6); hypothesis-bank perception; swing/take decision; check swings; heuristic pitcher AI; `sim-cli cage` and `sim-cli sensitivity`; per-pitch cost measured against the C1 budget. | Archetype matchups produce distinct, intuitive profiles; monotonicity suite passes; plate-appearance-level rates (K, BB, contact quality) land in rough Tier B bands. |
| **M4: Walking-skeleton game** | Flat field plus fence polyline and foul lines; **simple** fielders (reaction delay, noisy landing estimate, run, capture test); straight ballistic throws; runners that run on contact and advance by force or simple judgment; v1 rules core (§7); event-sourced box score; AI-vs-AI spectator mode in a basic 3D view. | A 9-inning AI-vs-AI game runs end to end in the CLI and watches start to finish in Godot; box-score accounting identities hold over 1,000 games; the owner judges whether it's watchable. |
| **M5: Deepen the field** | Ground bounce and roll by surface; wall caroms; mound; full fielding (§4.4) including 1B stretch and scoop; relays and cutoffs; catcher blocking (wild pitches and passed balls); tag-ups, rounding, sliding, steals; rules expansion. | 10,000 balls in play with no stuck states; routine-play success above threshold; error and extra-base rates in the Tier B envelope. |
| **M6: Manager layer** | Tactical command API; human manager HUD; AI manager on the same API; acute fatigue; bullpen warm-up; substitutions; pacing modes; causal play explanations (§4.7). | Human-vs-AI game is playable; tactics measurably change outcomes in A/B sims (e.g. a stolen-base attempt rate against slow deliveries). |
| **M7: League** | World generator; schedules; parametric ballparks (emergent park factors); multi-day recovery; SQLite saves; standings and leaderboards; parallel day-slate sim; learned AI tables (§4.5) regenerated per league environment. | Full fictional season playable; background games inside the per-game budget; park factors diverge in physically sensible directions. |
| **M8: Presentation** | Rigged players; IK onto physical bat and glove; broadcast camera director; replays; physically driven audio. | Subjective quality bar set by the owner. |

Deferred past M8, carried from the note and Pass 1: seam-shifted wake, weather and altitude air density (altitude can come earlier with parks), umpire perception and framing, swing styles, evolving mechanics, scouting inferred from behavior, online play.

---

## 6. Minimal v1 Attribute Set

Pass 1 lists about 30 traits. v1 should start with the smallest set that exercises each mechanism, each with a physical unit and a single owning subsystem. Add traits only when a mechanism needs one.

| Role | Attribute | Physical meaning | Feeds |
| :--- | :--- | :--- | :--- |
| All | `Height`, `Handedness` (bats/throws) | m; L/R/S | Zone bounds, reach, templates |
| Pitcher | `ArmSpeed` | Fastball release speed (m/s) | Release velocity |
| | `SpinCapacity` | Spin rate at a given velocity (rad/s) | Arsenal spin |
| | `Command` | Release angular σ (mrad) | Location noise |
| | `Arsenal[]` | Per pitch: speed ratio, spin axis, spin efficiency, usage | Pitch definitions, batter priors |
| | `Extension`, `ArmSlot` | m; rad | Release point, deception |
| | `Stamina` | Pitch budget before degradation | Fatigue (M6) |
| Batter | `BatSpeed` | Peak sweet-spot speed (m/s) | Swing timing profile |
| | `QuickHands` | Swing duration (ms) | Commit time |
| | `Vision` | Angular observation σ | Perception |
| | `SpinRecognition` | Spin-cue σ and its decay with time | Hypothesis posterior |
| | `BatControl` | Swing-plane σ (cm-equivalent) | Execution noise |
| | `Timing` | Temporal σ (ms) | Execution noise |
| | `Discipline` | Bias and sharpness of the swing/take threshold | Decision |
| Fielder/Runner | `Speed`, `Acceleration` | m/s; m/s² | Movement |
| | `Reaction`, `ReadSkill` | ms; estimate-noise decay | Fielding perception |
| | `Hands` | Glove-placement σ | Capture |
| | `ArmStrength`, `ArmAccuracy` | m/s; angular σ | Throws |

Display as a 20–80 scouting scale is a UI mapping over these values (M6+).

---

## 7. v1 Rules Scope (M4)

**In:** balls and strikes with the ABS-style zone (C4); fouls (including two-strike fouls and foul tips); walks; strikeouts; uncaught third strike; hit by pitch (batter body as capsules plus a simple evasion reaction); fair/foul determination; force and tag outs; caught fly balls; tag-up requirements; runs scoring, including time plays on third outs; inning and game flow; extra innings (pick the runner-on-second rule or not, see §9); universal DH; ground-rule doubles; home runs.

**Deferred:** infield fly (needs pop-up judgment; add in M5); appeals; balks; interference and obstruction; batting out of order; pitch clock (only relevant if live-paced presentation wants it); manager challenges.

Rules testing: a **scenario corpus** of data files ("runners on 1st and 3rd, 1 out, fly to RF caught, runner on 3rd leaves early → appeal-free v1 behavior is X"), each asserting the resulting state. The rules FSM is tested by scripted events alone, with no physics involved.

---

## 8. Risk Register (updated)

| Risk | Likelihood | Impact | Mitigation |
| :--- | :--- | :--- | :--- |
| Emergent stats land far outside the playability envelope and knobs can't reach it | **High** | High | Tier A first, so the physics is trusted; sensitivity elasticities from M3; treat unreachable envelopes as model bugs (§4.6) |
| Manager decisions feel inconsequential or opaque | Medium | **High** (it's a management game) | A/B sims per tactic in M6 gate; causal explanations (§4.7) |
| AI-vs-AI degenerate exploitation | Medium | Medium | Sequence-aware batter priors; learned tables iterated toward equilibrium (§4.5) |
| Scope: the project is very large for a small team | **High** | High | Walking skeleton at M4; minimal attributes; rules scope list; defer presentation polish |
| Statistical fallbacks creep in under deadline pressure | Medium | High | §1.1 layer split enforced by an architecture test (AI cannot reference world-state types; World cannot reference belief types) |
| Godot .NET constraints (TFM, no web export) | Low | Low–Med | Check in M0; desktop-first |
| Re-simulated replays drift as code changes | High if done naively | Medium | Store timelines, not seeds (§1.2) |

---

## 9. Open Questions for the Owner

1. **Team and skills:** solo developer plus AI assistants? How comfortable with C#, Godot, and the physics math? This sets how far to cut scope.
2. **Rule set flavor:** modern MLB-like (universal DH, ABS zone, runner on second in extras, pitch clock) or classic? The note says the league is fictional, so either fits.
3. **Early visuals:** is a plain geometric 3D view (capsules, lines, trails) acceptable through M5, with real character models only at M8?
4. **Primary experience** (the note's first open question, now sharper): is the main loop *managing your team's games live*, with background league games instant? Or is *season-level GM play* with mostly simulated games equally important? This decides whether M6 (manager layer) or M7 (league) comes first after M5.
5. **Pass 1 file:** keep `PLAN.md` (Pass 1) and this file side by side, or merge into one consolidated plan once these questions are answered?

## 10. Proposed Next Planning Artifacts

Before writing code, in this order:
1. `docs/physics-spec.md`: equations, parameter tables with citations, the coordinate frame, and Tier A golden test cases. Gates M1–M2.
2. `docs/agent-models.md`: §4.1, §4.2, §4.4, and §4.5 expanded with exact state variables, observation models, and parameter-to-attribute mappings. Gates M3.
3. `docs/data-formats.md`: event-log and `PlayTimeline` schemas (including belief snapshots). Gates M0.
4. `docs/calibration-targets.md`: Tier A values and Tier B envelopes. Gates M3–M5.
