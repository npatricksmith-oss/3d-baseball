# Physics Specification

**Last updated:** 2026-10-30
**Status:** Draft v1. Gates milestones M1 (pitch flight) and M2 (bat and contact).
**Parent:** [PLAN.md](../PLAN.md) §6 and §13

This document defines the **world physics**: the parts of the simulation that decide what physically happens. Agent perception and decision models are in [agent-models.md](agent-models.md). Rules judgments that use this geometry (fair/foul, strike/ball) are in the rules spec.

**Parameter status tags** (used throughout):
- **[S] Sourced:** taken from a published source (§16).
- **[D] Derived:** computed from sourced values or from this spec's own formulas.
- **[E] Estimate:** a placeholder that must be calibrated or verified. Every [E] value is a named, configurable parameter, never a literal in code.

**D1 reminder:** physics never rolls for outcomes. The *only* randomness inside world physics is bounce-surface normal perturbation (§9.3). All other noise (release error, swing error, glove error) is applied by agents to their *motor commands* before the world receives them.

---

## 1. Units, Frames, Sign Conventions

### 1.1 Units
SI everywhere inside the sim: m, kg, s, rad, rad/s, N, kg/m³. Imperial units (mph, ft, in, rpm, oz) appear only in config files that label them explicitly, in UI, and in reports.

| Conversion | Value |
| :--- | :--- |
| 1 mph | 0.44704 m/s |
| 1 ft | 0.3048 m |
| 1 in | 0.0254 m |
| 1 rpm | π/30 rad/s ≈ 0.104720 rad/s |
| 1 oz | 0.028349523 kg |

### 1.2 World frame (right-handed; same axes as Statcast/Trajectory Calculator)
- **Origin:** the rear point (apex) of home plate, at ground level.
- **+Y:** from home plate toward the pitcher's rubber and center field.
- **+Z:** vertically up.
- **+X:** toward the first-base side (the catcher's right, the pitcher's left).
- Gravity: **g** = (0, 0, −9.80665) m/s² [S].

The client conversion is defined once in the client (PLAN.md §5). The sim never knows about it.

### 1.3 Spin sign convention (high bug risk)
Spin is an angular velocity vector **ω** (rad/s), right-hand rule. The surface velocity of a point at offset **r** from the ball's center is **ω** × **r**.

| Situation | Spin vector | Check |
| :--- | :--- | :--- |
| Batted ball moving **+Y** with **backspin** | **ω** ∥ **+X** | top of ball moves −Y (backward); Magnus **ω** × **v** points +Z (lift) |
| Pitch moving **−Y** with **backspin** (four-seam "rise") | **ω** ∥ **−X** | Magnus points +Z |
| Pitch moving −Y with **topspin** (curveball) | **ω** ∥ **+X** | Magnus points −Z |

A unit test must encode this table. During this spec's cross-check, a sign error here shortened a 406 ft fly ball to 270 ft, an easy bug to miss without the test.

---

## 2. Baseball Properties

| Quantity | Default | Allowed range | Status |
| :--- | :--- | :--- | :--- |
| Mass m | 0.14529 kg (5.125 oz) | 0.1417–0.1488 kg (5–5¼ oz) | [S] Official Baseball Rules |
| Circumference | 0.231775 m (9.125 in) | 9–9¼ in | [S] |
| Radius R | 0.036888 m | | [D] |
| Cross-section A = πR² | 4.2748 × 10⁻³ m² | | [D] |
| Moment of inertia I_b = α m R² | α = 0.4 | 0.35–0.42 | [E] uniform-sphere value (as used by Kensrud et al.); verify for a real baseball |

Pass 1's R = 0.0366 m is replaced by 0.036888 m.

---

## 3. Atmosphere

Air density in kg/m³ comes from temperature, pressure, humidity, and elevation. This is the formula used by Nathan's Trajectory Calculator, as reproduced in the open-source `pybbda` port [S, via third-party port; verify against Nathan's current spreadsheet]:

```
Tc      = temperature in °C
P_mmHg  = barometric pressure in mmHg (sea-level-referenced reading)
SVP     = 4.5841 · exp( (18.687 − Tc/234.5) · Tc / (257.14 + Tc) )     # saturation vapor pressure, mmHg
ρ       = 1.2929 · (273 / (Tc + 273)) · ( P_mmHg · exp(−β · elev_m) − 0.3783 · RH% · SVP / 100 ) / 760
β       = 1.217e-4 per m
```

- **Calculator default environment:** 70 °F, 29.92 inHg, 50% RH, 15 ft elevation → **ρ = 1.1938 kg/m³** [D].
- v1 uses one ρ per ballpark (elevation from the park config, standard weather). Weather variation and wind come later (PLAN §14), but the flight equations below already take air velocity, so adding wind needs no model change.

---

## 4. Flight Model

### 4.1 State
`(r, v, ω)`: position, velocity, spin. Spin changes only through decay (§4.4) and through impulses (§8, §9).

### 4.2 Forces
With air velocity **w** (zero in v1), relative velocity **u** = **v** − **w**, u = |**u**|, **û** = **u**/u:

```
k        = ρ A / (2 m)
ω⊥       = ω − (ω·û) û                    # transverse spin; the gyro component produces no Magnus force
S        = R |ω⊥| / u                     # spin factor
C_D      = cd0 + cdspin · (|ω| in rpm / 1000)
C_L      = cl2 · S / (cl0 + cl1 · S)
a_drag   = −k · C_D · u · u
a_magnus = k · C_L · u² · (ω⊥ × û) / |ω⊥ × û|      (zero if |ω⊥| = 0)
a        = g + a_drag + a_magnus
```

| Coefficient | Default | Status |
| :--- | :--- | :--- |
| cd0 | 0.3008 | [S] Trajectory Calculator defaults, via the `pybbda` port; verify |
| cdspin | 0.0292 (per 1000 rpm) | [S] same |
| cl0 | 0.583 | [S] same |
| cl1 | 2.333 | [S] same |
| cl2 | 1.120 | [S] same |

Notes:
- Whether the C_D spin term should use total spin |ω| or transverse spin |ω⊥| is **open**. The calculator applies it to total spin, and v1 follows the calculator.
- Seam-shifted wake effects (movement not explained by Magnus force, which matters for sinkers and some changeups) are **deferred** (PLAN §14). The model above explains movement entirely through Magnus force.

### 4.3 Drag crisis and low-speed behavior
The coefficients above are fitted to game-speed balls. Very slow balls (dribblers, bunts after a bounce, short throws) get drag from the same formula. That is acceptable because drag barely matters at those speeds. No special case.

### 4.4 Spin decay
Following Nathan, "The Spin Decay of a Baseball", which scales from golf-ball data [S]:

```
dω/dt = −(u / (τ₁₀₀ · 44.704 m/s)) · ω         # decay rate proportional to airspeed
```

- **τ₁₀₀** is the 1/e decay time at 100 mph. Nathan estimates **20–25 s** for a baseball [S]. Default **τ₁₀₀ = 22 s**.
- Golden tests use τ₁₀₀ = ∞ (no decay) to match the calculator's default.
- Nathan's paper and this spec's cross-check both find that decay changes a 100 mph fly ball's carry by **less than 1 ft**. Pass 1's "15–25% over 5 s" figure is superseded.

---

## 5. Numerical Integration

### 5.1 Integrator
Classic **RK4** on the full state `(r, v, ω)`, with spin decay (§4.4) integrated as part of the ODE. *(Changed in M1, 2026-10-31: the earlier draft held spin constant within a step and decayed it once at the step end. That split scheme drifted 0.7 mm over a 5.6 s fly ball against the oracle; the full-state form agrees with the oracle to ~3e-8 m at production step sizes and costs nothing measurable.)*

### 5.2 Step size
Cross-check result: on a 100 mph / 30° / 2500 rpm fly ball, RK4 at Δt = 10 ms and Δt = 1 ms agree to **0.0001 ft** in carry. A fixed 1 ms step everywhere (Pass 1) is unnecessary.

| Phase | Default Δt | Note |
| :--- | :--- | :--- |
| Pitch flight | 2 ms | ~200 steps per pitch |
| Batted ball / throw flight | 5 ms | |
| Bat-ball contact window | see §7 | Sampled from interpolated ball motion, not integrated more finely |

**M1 result:** confirmed. A3 gives |Δcarry| < 1e-6 ft between 10 ms and 1 ms, and the Rust sim at these steps matches the independent oracle (Δt = 0.1 ms) to ~3e-8 m on all 21 flight fixtures.

### 5.3 Dense output and event detection
Between steps, ball position is a **cubic Hermite interpolant** built from (r, v) at both step ends. All events are found on this interpolant, then refined by bisection to 1 µs:
- Plate-plane crossing (strike zone, §10)
- Ground contact (§9), wall/pole/base contact (§9)
- Fielder-glove interception, a query made by agents (`agent-models.md`)
- Bat contact (§7)

An event discards the rest of the step. The state is rebuilt exactly at the event time by re-integrating a partial step from the step start.

### 5.4 Determinism
All transcendental functions go through the `libm` crate. No fused multiply-add (`mul_add`) or platform SIMD intrinsics in physics code. Iteration order is always fixed (PLAN §4.2).

---

## 6. Bat Model

### 6.1 Geometry
The bat is a solid of revolution around its long axis. `u` is the distance from the knob end (m), `L` is total length, and `R_bat(u)` is the profile radius. The profile is piecewise: knob → handle → taper → barrel → end cap (rounded with radius = barrel radius; a cupped end is an option).

**Default 34-inch wood bat** [E, a plausible profile; tune in M2]:

| Segment | u (in) | Diameter (in) |
| :--- | :--- | :--- |
| Knob | 0 – 0.5 | 1.35 |
| Knob → handle | 0.5 – 1.0 | 1.35 → 0.93, linear |
| Handle | 1.0 – 11 | 0.93 |
| Taper | 11 – 22 | 0.93 → 2.50 with shape exponent 1.6: d = 0.93 + 1.57·((u−11)/11)^1.6 |
| Barrel | 22 – 34 | 2.50 |

Rules limits [S]: maximum diameter 2.61 in; maximum length 42 in.

### 6.2 Mass properties
Mass properties are **computed by integrating the profile** with wood density ρ_w, not entered by hand. This keeps geometry and inertia consistent for every bat in the league:

```
M      = ∫ ρ_w π R(u)² du
u_cm   = (1/M) ∫ u dm
I_cm   = ∫ (u − u_cm)² dm  +  ∫ ¼ R(u)² dm      # transverse
I_ax   = ∫ ½ R(u)² dm                            # about the long axis
```

Cross-check values for the default profile with ρ_w = 660 kg/m³ [D]:

| Quantity | Value |
| :--- | :--- |
| M | 0.965 kg (34.0 oz). Slightly heavy for a 34-in MLB bat (typically ~31–32 oz). Tune by density, profile, or cupped end. |
| u_cm | 23.5 in (0.597 m) from knob |
| I_cm (transverse) | 0.0382 kg·m² |
| I_ax | 3.96 × 10⁻⁴ kg·m² |
| MOI about the 6-in point | 0.229 kg·m² (≈ 12,500 oz·in²) |

**Validation bands for any generated bat** [E; verify against published wood-bat data]: M 0.85–0.95 kg; MOI about the 6-in point 0.18–0.24 kg·m².

### 6.3 Bat pose
The swing model (`agent-models.md`) supplies the bat's world pose `T_bat(t) = (p_knob(t), q(t))` as an **analytic, differentiable function of time**, from which the physics gets:
- axis unit vector **â**(t), center of mass **c**(t) = **p**_knob + u_cm·**â**
- linear velocity **V**(t) of the center of mass, angular velocity **Ω**(t)
- the velocity of any bat point **p**: **V** + **Ω** × (**p** − **c**)

**Free-bat approximation** [S, standard in the literature]: during the ~1 ms collision, the hands' influence is ignored. After contact, the bat's own path is cosmetic; the swing model may apply the recoil impulse or ignore it.

---

## 7. Bat-Ball Contact Detection

1. **Window:** from the swing plan's nominal contact time t̂, check [t̂ − 25 ms, t̂ + 25 ms]. Skip if the swing never brings the bat within 0.5 m of the ball's path (cheap reject).
2. **Signed gap:** for time t, let **b**(t) be the ball center from §5.3's interpolant. Project it onto the bat axis to get u(t), clamped to [0, L]. The gap is:
   `d(t) = |b − (p_knob + u·â)| − R_bat(u) − R`
   This is exact for a cylinder and close for gentle tapers. At the end cap, use distance to the cap sphere.
3. **Sample** d(t) every **0.1 ms** (≤ 500 evaluations). At the first sign change from + to −, **bisect** to 1 µs → contact time t*, contact station u*.
4. **Contact normal n̂**, pointing from the bat into the ball: the radial direction from the axis point to the ball center, tilted by the taper slope R′(u*):
   `n̂ = normalize( radial − R′(u*) · â )`
5. **Contact point** on the ball: **r**_b = −R·n̂ relative to the ball center. On the bat: **r**_B = (**b** − R·n̂) − **c**.
6. **Miss:** no sign change in the window. Record the **closest approach** (minimum d, time, and u) for telemetry and play explanations.
7. **Second contact:** after the impulse (§8), if the bodies are still approaching at the contact point, apply one more impulse (at most 2 in total). This handles grazing foul tips.

---

## 8. Collision Impulse Solver

One **instantaneous rigid-body impulse model with normal and tangential restitution and a Coulomb friction limit**. It is used for bat-ball contact and, with an immovable second body, for ground, wall, base, and pole contacts.

### 8.1 Formulation
Body 1 is the ball (m, I_b, contact offset **r**₁ = **r**_b). Body 2 is the bat (M, world inverse inertia I₂⁻¹ built from I_cm, I_ax, and **â**, offset **r**₂ = **r**_B). For immovable surfaces, body 2's inverse mass and inertia are 0.

```
contact-point velocities:  c₁ = v + ω × r₁ ;  c₂ = V + Ω × r₂
relative velocity:         c  = c₁ − c₂ ;  c_n = c·n̂ ;  c_t = c − c_n n̂
approaching iff            c_n < 0

[x]× = skew-symmetric cross-product matrix of x
K = (1/m + 1/M)·𝟙  −  [r₁]× I₁⁻¹ [r₁]×  −  [r₂]× I₂⁻¹ [r₂]×     # Δc = K·J for impulse J on ball (−J on bat)
```

**Targets:** c_n′ = −e_n·c_n and c_t′ = −e_t·c_t.

**Algorithm:**
1. **Sticking solution:** J* = K⁻¹ (c′ − c), with c′ = −e_n c_n n̂ − e_t c_t.
2. If |J*_t| ≤ μ · (J*·n̂), accept J = J*.
3. **Otherwise sliding:** t̂ = c_t / |c_t|, J = j (n̂ − μ t̂), with `j = −(1 + e_n) c_n / ( n̂ᵀ K (n̂ − μ t̂) )`.
4. **Apply:** v += J/m; ω += I₁⁻¹ (r₁ × J); V −= J/M; Ω −= I₂⁻¹ (r₂ × J).
5. **Guard:** total kinetic energy must not increase. If it does (possible with some e_t > 0 geometries), retry with e_t = 0 and log the case. If the e_t = 0 solution still gains energy (possible through normal–tangential coupling in K; found by the M1 code review), apply a **frictionless normal impulse** `J = −(1 + e_n) c_n / (n̂ᵀK n̂) · n̂`, which cannot gain energy for e_n ≤ 1, and flag it. *(Added 2026-10-31.)*

**Analytic checks** (these become test cases):
- Head-on normal impact on the bat's axis-normal line, ball with no spin. The model reduces to 1/M_eff = 1/M + b²/I_cm (b = distance from the bat's center of mass) and the classic collision efficiency `q = (e_n − r)/(1 + r)`, r = m/M_eff, with outgoing ball speed `q·v_pitch + (1 + q)·v_bat`.
- Gripping-regime oblique impact on a stationary ball: reproduces the Kensrud–Nathan–Smith relations [S], e.g. ω = (1 + e_x)·v_bat·sinψ / (R(1 + r_x)(1 + α)).

### 8.2 Bat-ball parameters

| Parameter | Default | Status / note |
| :--- | :--- | :--- |
| e_n(u*, c_n) | `e_peak(c_n) · f(u*)` | Normal COR: profile × speed dependence |
| f(u) | 1 at the sweet spot u_ss; quadratic falloff to f_tip = 0.75 at u = L and f_taper = 0.55 at the taper start; 0.35 in the handle | [E] Models vibrational energy loss off the sweet spot. Calibrate against published COR-vs-impact-location data in M2 |
| u_ss | L − 0.15 m (about 6 in from the tip) | [E] Statcast also defines bat speed 6 in from the tip |
| e_peak | **Tuned so q = 0.20 ± 0.01 at u_ss** for the default bat at game-speed relative velocity (~150–165 mph). The cross-check needs **e_peak ≈ 0.44–0.47** depending on exact u_ss. | [S] q ≈ 0.2 for wood at the sweet spot (Nathan); e derived [D] |
| Speed dependence | e_peak(c_n) = e_ref − s_e·(|c_n| − v_ref), s_e > 0 | [E] COR falls with impact speed (sign [S], magnitude to calibrate) |
| e_t (tangential restitution, e_x in KNS) | 0.40 | [S] KNS 2017 fit: 0.405 ± 0.010 average, wood bat 0.464. Cross & Nathan 2006 found 0.17 in a different setup. Range 0.15–0.46; calibrate against spin-vs-undercut |
| μ (ball-bat friction) | 0.50 | [S] value used by KNS and earlier work; KNS lower bound only ≥ 0.15 |

**Collision-efficiency cross-check** [D] (default bat, 85 mph pitch at the plate, 75 mph bat at the contact point):

| Impact u (from knob) | M_eff | r | e_n needed for q = 0.20 |
| :--- | :--- | :--- | :--- |
| 26 in | 0.877 kg | 0.166 | 0.399 |
| 28 in | 0.728 kg | 0.200 | 0.440 |
| 30 in | 0.573 kg | 0.253 | 0.504 |

At q = 0.20 the sanity anchor gives **≈ 107 mph exit velocity**. Off-sweet-spot impacts lose q quickly: near the tip (32.5 in, e = 0.40) the model gives q ≈ 0.04, about 81 mph. That matches the expected "end of the bat" weak contact.

### 8.3 Scale of contact sensitivity
For offset D between the ball's center and the bat axis (perpendicular to the swing direction), the normal tilts by asin(D / (R + R_bat)). With R + R_bat ≈ 0.0687 m for a 2.5-in barrel, that is **≈ 0.83° per mm** near center. Launch angle also depends on the bat's attack angle, the pitch's descent angle, and friction. **M2 deliverable:** the measured map from (vertical offset, horizontal offset, timing) to (exit velocity, launch angle, spray angle, spin), used to size swing-noise parameters in `agent-models.md`.

### 8.4 Bunts (M5)
A bunt uses the same solver. "Deadening" the ball with soft hands is modeled as a bunt-grip multiplier on e_n (< 1) [E]. It is a physical parameter of the bunt technique, not an outcome adjustment.

---

## 9. Ground, Field Objects, and Bounces

### 9.1 Surfaces
The ground is a **height function** h(x, y): zero everywhere except the pitcher's mound (§11). The surface type at (x, y) comes from 2D region polygons. Each surface defines:

| Surface | e_n at reference speed | μ | e_t | Rolling resistance μ_r | Roughness σ_n | Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| Infield grass | 0.45 | 0.45 | 0.0 | 0.28 | 0.6° | [E] all |
| Infield dirt | 0.50 | 0.55 | 0.0 | 0.22 | 1.5° | [E] |
| Outfield grass | 0.42 | 0.45 | 0.0 | 0.14 | 0.8° | [E] |
| Warning track | 0.48 | 0.60 | 0.0 | 0.10 | 1.5° | [E] |
| Artificial turf (park option) | 0.55 | 0.40 | 0.0 | 0.06 | 0.3° | [E] |

Rolling resistance for infield grass and dirt was raised from 0.12 / 0.09 to 0.28 / 0.22 in M4 so that grounders stop in ~110 m (owner accepted 2026-11-11; real roll-out data is research item R4). The code has only these two surfaces so far (`bb_physics::ground`: dirt inside the infield radius, grass elsewhere); the other rows keep their first-draft values until they are implemented.

e_n decreases with impact speed: e_n(c_n) = e_ref / (1 + k_s·|c_n|) [E form]. **All values in this table are unsourced placeholders.** *(M4 skeleton, 2026-10-31: the implementation uses rolling resistance μ_r = 0.22 on dirt and 0.28 on grass, because the values above let a 25 m/s grounder roll ~250 m; 0.22/0.28 gives ~110 m. R4 replaces both with measured values.)* M5 calibrates them against observed behavior: ground-ball speeds reaching infielders, the hop height of typical grounders, roll-out distances of outfield singles. Literature on ball bounce from Cross would make good starting points if it can be found.

### 9.2 Bounce and roll
- A **bounce** is §8's solver against an immovable ground with normal n̂ (perturbed per §9.3).
- **Rolling mode:** if post-bounce |c_n| < 0.25 m/s [E], or the bounce apex is below 1 cm, switch to rolling. The ball is constrained to the surface and rolls without slipping, with deceleration μ_r·g along the velocity plus slope components (the mound). Spin is locked to rolling (ω = n̂ × v / R, the no-slip condition).
- **Rest:** speed < 0.05 m/s means the ball is at rest.
- **Surface transitions** while rolling (dirt to grass) switch parameters at the region boundary.

### 9.3 Bad hops: the one source of physics randomness
At each bounce, the surface normal is tilted by a random angle drawn from a 2D Gaussian with σ_n (by surface; it scales with field condition, which a park or weather parameter can widen). Drawn from the keyed RNG `(gameSeed, playIndex, bounceIndex, "surface")`. This is the **only** stochastic element of world physics, and it is a physical one: a surface imperfection.

### 9.4 Field objects

| Object | Geometry | Contact | Notes |
| :--- | :--- | :--- | :--- |
| Outfield wall segments | Vertical rectangles along a polyline with per-segment height; **top edge as a horizontal cylinder** (radius 5 cm [E]) | §8 against immovable; padding e_n 0.35 [E], μ 0.5 [E] | The top-edge cylinder produces "off the top of the wall" bounces. A segment's material (padding, brick, ivy, chain-link) sets its parameters per park, giving emergent park character |
| Foul poles | Vertical cylinders on the foul lines | Rules: contact = home run | Radius per park config |
| Bases | 18 in × 18 in boxes [S], 3 in high [E]; the plate and rubber are flush or near-flush | §8 | "Ball hits the bag" bounces emerge |
| Backstop, foul-territory walls, dugouts | Vertical walls along the playing-field boundary (§9.5) | A wall below its top: carom like the outfield wall (stands-wall padding e_n 0.35 [E]). Over its top, or into a dugout: out of play | Geometry and the reference park's values in §9.5 |
| Batter, catcher, umpire bodies | Capsule sets from the agent's current pose | Contact recorded for rules (hit by pitch, ball hits umpire); the ball is damped (e_n 0.2 [E]) | Pose supplied by agents |
| Gloves | Defined in `agent-models.md` (capture geometry) | A failed capture deflects via §8 with glove e_n ≈ 0.15 [E] | |


### 9.5 The playing-field boundary (added 2026-11-14, A10)

The playing field is the region the ball and the players stay inside. Its boundary is the outfield fence (§9.4, `fence_at(θ)` through the line and center distances) plus, in foul territory, the **backstop** behind home and a **foul-territory wall** (the front of the stands) along each foul line. A ball that crosses the boundary above the wall top there, or crosses it at a dugout, has **left the playing field** (rules-spec §2 `BallTouched{OutOfPlay}`); below the top it caroms. Before this section the fence's line distance simply continued around into foul ground, so foul territory reached 330 ft from home in every direction and a foul pop could be chased 100 m from the plate.

**Geometry** (reference park; every value a park parameter). Coordinates along and off a foul line: for the first-base side (x ≥ 0), s = (x + y)/√2 is the distance along the line from the rear apex of home, c = (x − y)/√2 the distance off it into foul ground (c > 0 is foul). The third-base side mirrors it (x → −x). The playing field is the intersection of:

| Boundary piece | Inside when | Reference value | Status / source |
| :--- | :--- | :--- | :--- |
| Outfield fence | ρ ≤ fence_at(θ) (ρ = horizontal distance from home) | 330 / 400 / 330 ft, 8 ft high | existing (§9.4) |
| Backstop | y ≥ −B | **B = 60 ft** (18.29 m) | [S] OBR 2.01 recommends 60 ft or more from home to the backstop |
| Side wall (each side) | c ≤ C | **C = 50 ft** (15.24 m) | [E] OBR 2.01 recommends 60 ft from the base lines to the nearest fence or stand; most major-league parks have less (below) |
| Taper to the foul pole | c ≤ C·(s_pole − s)/(s_pole − s_T) | **s_T = 100 ft** (30.48 m) along the line; s_pole = the fence's line distance | [E] the stands close in to meet the fence at the pole, as in most parks |

All four are half-planes or (the fence) a star-shaped curve, so "inside" is the minimum of four signed distances being ≥ 0, and the wall a ball meets is the one whose signed distance is the smallest.

With these values the backstop runs between x = ±3.26 m (the side walls meet it there, at 45°), and the foul territory inside the walls, behind home included, is **≈ 23,900 sq ft** (2,220 m²). That matches the major-league average of about 23,300 sq ft [S: Beyond the Box Score, 2019, "Marco Estrada and the Oakland Athletics' foul pop-ups"; the Oakland Coliseum, the largest, about 40,700]. OBR's 60 ft side clearance with the same taper would give about 31,800 sq ft, an Oakland-sized park, so the reference park uses 50 ft. **Geometry test (from this table, not the code):** the shoelace area of the polygon (pole, home, pole, taper start, backstop corner, backstop corner, taper start) is 23,885 sq ft ± 1.

**Heights and openings** [E], by where the wall is:

| Part | Height | Why |
| :--- | :--- | :--- |
| Backstop screen: the backstop and the side walls behind home (s < 0 on either side) | **9.0 m** (≈ 30 ft) | A backstop is a low wall under a tall net that the ball rebounds from; a pop-up that clears the net is in the seats |
| Dugouts: the side wall for **s from 20 ft to 80 ft** (6.10–24.38 m) | **0** (an opening) | The dugout is sunk below field level; any ball that reaches its front, rolling or in the air, has left the field. Placement and length are unsourced placeholders |
| Foul-territory wall elsewhere | **4 ft** (1.22 m) | The front wall of the stands along the lines; a fielder can reach over it for a catch (rules-spec §5.3: his feet must be in the field) |

Protective netting above the side walls is not modeled (deferred): a ball that would strike it goes into the seats here.

**Contact.** A ball meets a backstop, side or taper wall when its **center** comes within the ball radius R of the wall plane (the outfield fence skeleton still uses the center on its curve; that 3.7 cm is left as is so the outfield stays unchanged). Below the top (center height < wall height + R), the wall is §8's immovable body with the normal of the plane: the normal velocity reverses with the stands-wall restitution e_n = 0.35 [E] (the same padding value as the outfield wall), the tangential velocity is kept, and spin is halved, as for the outfield wall. At or above the top, or at a dugout, the ball has left the field. A ball **rolling** into a wall stops at it, as at the outfield fence (skeleton); rolling into a dugout it leaves the field.

**Throws** fly inside the same boundary: a thrown ball that reaches a wall in the air caroms or leaves the field by the same rules, and one that reaches the ground continues with the bounce model (§9.2) inside the boundary.

**Players** stay inside it: a fielder's body center stays at least its radius (0.3 m, the body radius used for dives) inside every wall. A fielder who runs at a wall stops at it, losing the velocity component into it; one at the wall can still reach above its top for a catch (the wall leap height applies within 3 m of any wall, as at the outfield fence).

---

## 10. Plate Crossing and Strike Zone

Per D11, using the ABS-style 2D zone:
- **Judgment plane:** y = y_mid = **0.2159 m**, the midpoint of the plate's 17-in depth (front edge at y = 0.4318 m) [S: the ABS zone is judged at plate midpoint].
- **Horizontal extent:** |x| ≤ 0.2159 m (17-in plate width) [S].
- **Vertical extent:** z_bot = 0.27·H, z_top = 0.535·H, where H is the batter's standing height [S: 2026 ABS uses 27% and 53.5% of height, measured without shoes].
- **Strike** if the ball's cross-section circle (radius R), centered where the ball center crosses y_mid, **overlaps the rectangle**. That is a circle-rectangle overlap test: rounded corners. [Choice. Treat "any part of the ball" as the rule; verify the exact ABS edge rule if it matters.]
- The crossing time comes from §5.3 event detection.
- A pitch that bounces before the plane is judged by the same geometry using the post-bounce trajectory. Rules decide whether a bounced pitch can be a strike (it cannot in standard rules).

---

## 11. Field Geometry (fixed elements)

All values [S] from the Official Baseball Rules unless marked. Exact base coordinates live in the field config file, derived from these:

| Element | Value |
| :--- | :--- |
| Home plate | 17 in wide front edge; 8.5 in sides; two 12-in edges meeting at the rear apex (origin) |
| Foul lines | From origin along ±45° from +Y |
| Base paths | 90 ft (27.432 m) from the rear apex of home to the outer corner of 1B/3B along the foul lines; 2B center 127 ft 3⅜ in from the apex |
| Bases | 18 in square (since 2023) |
| Pitching rubber | 24 × 6 in; front edge 60 ft 6 in (18.440 m) from the rear apex |
| Mound | 18 ft diameter; center 59 ft from the rear apex; rubber 10 in above home-plate level; slope 1 in per ft over 6 ft, starting 6 in in front of the rubber; flat table around the rubber |
| Batter's boxes | 4 × 6 ft, 6 in from the plate edge |
| Fences, foul territory, wall heights, park elevation | Per-park parameters (PLAN §11) |

---

## 12. Derived Telemetry (definitions)

Every metric is computed from the event log and timeline (PLAN §4.3). Definitions are fixed here so stats stay comparable across versions.

| Metric | Definition |
| :--- | :--- |
| Release speed | |v| at release |
| Plate speed | |v| at the plate-plane crossing |
| Spin rate, spin efficiency | |ω|; |ω⊥| / |ω| at release |
| Spin axis | Direction of the Magnus force from ω⊥ at release, as a clock-face tilt **from the pitcher's perspective** (12:00 = pure backspin) and as Savant's 0–360° `spin_axis`. Exact formulas and checks: [data-formats.md §9](data-formats.md) |
| **Movement (internal definition)** | Displacement at the plate plane between the actual trajectory and a **no-Magnus twin** (same release state, drag and gravity only), over the **full flight**. Components: induced vertical break (z) and horizontal break (x). **Not identical** to Savant's pfx metrics, which use a 40-ft window. M1 also implements a Savant-comparable variant for sanity comparisons |
| Exit velocity | |v′| immediately after the bat impulse |
| Launch angle | Elevation of v′ above horizontal |
| Spray angle | Azimuth of v′ in the XY-plane from +Y; positive toward +X (first-base side) |
| Contact offset | Signed distance from the ball center to the bat axis at t*, decomposed into vertical-ish (in the plane of â and z) and along-axis (u* − u_ss) |
| Bat speed | Speed of the bat point at u_ss at t* (or at closest approach if missed). Matches Statcast's "sweet spot 6 in from the head" definition |
| Attack angle | Elevation of the sweet-spot velocity at t* |
| Projected distance | Horizontal distance from the origin to where the ball would reach z = 0 with no obstruction |
| Hang time | Contact → first ground or obstruction contact, or catch |
| Apex | Maximum z of the flight |

---

## 13. Randomness Inventory (physics-side)

| Source | Lives in | Notes |
| :--- | :--- | :--- |
| Bounce-normal perturbation | World physics (§9.3) | The only one |
| Pitch release error, swing execution error, glove placement error, throw error | Agents, applied to motor commands **before** the world acts | Specified in `agent-models.md` |
| Perceptual noise | Agents | Same |

Any new randomness in world physics requires a PLAN.md decision-log entry.

---

## 14. Validation (Tier A)

### 14.1 Oracle strategy
An **independent Python reference implementation** (`analysis/oracle/`, written in M1) implements §2–§5 and §8 directly from this document. Rust golden tests compare against oracle-generated tables over a grid of inputs. The oracle in turn is checked against published cases (below). Nathan's actual Trajectory Calculator spreadsheet should be obtained and used to cross-validate the `pybbda`-sourced coefficients.

### 14.2 Golden and analytic tests

| ID | Test | Expected | Source |
| :--- | :--- | :--- | :--- |
| A1 | Batted ball: 100 mph, 30°, 2500 rpm pure backspin, z₀ = 3 ft, calculator default atmosphere, no spin decay | Carry **406 ft ± 3**, flight **5.6 s ± 0.1** | [S] Nathan spin-decay paper. Cross-check: 407.7 ft / 5.62 s, apex 97 ft, with y₀ = 2 ft |
| A2 | Same with τ₁₀₀ = 20 s | Within 1 ft of A1 | [S] same paper (cross-check: +0.1 ft) |
| A3 | Step-size convergence for A1, Δt = 10 ms vs 1 ms | Δcarry < 0.01 ft | [D] cross-check: 0.0001 ft |
| A4 | Vacuum (ρ = 0) | Matches analytic projectile motion to 1e-9 relative | Analytic |
| A5 | Spin sign table §1.3 | Magnus directions as listed | Analytic |
| A6 | Mirror symmetry: sidespin ±, x₀ ± | Mirrored trajectories | Analytic |
| A7 | Head-on bat impact, no spin | Outgoing speed = q·v_pitch + (1+q)·v_bat with q = (e−r)/(1+r) to 1e-9 | Analytic (§8.1) |
| A8 | Default bat at u_ss, game speeds | q = 0.20 ± 0.01; EV 105–109 mph for an 85 mph plate speed and 75 mph bat | [S] q≈0.2 |
| A9 | Gripping oblique impact on a stationary ball | Spin and tangential speed match the KNS formulas (§8.1) | [S] KNS 2017 |
| A10 | Spin vs undercut sweep | Backspin rises with undercut, topspin with overcut; magnitudes within published ranges | [S] Nathan et al. batted-ball spin work (pull exact values in M2) |
| A11 | Energy guard | No collision increases total kinetic energy | Physical |
| A12 | Bat mass properties of default profile | Match §6.2 values; inside validation bands | [D] |
| A13 | Ball at rest on flat ground; rolling deceleration | Exactly μ_r·g; stops at the predicted distance | Analytic |

### 14.3 Plausibility bands (not goldens; flag if outside)
Cross-check values with this spec's model, 54 ft release distance, full-flight movement:

| Pitch | Flight time | Plate speed | Full-flight induced vertical movement |
| :--- | :--- | :--- | :--- |
| Four-seam 95 mph, 2300 rpm pure backspin | 0.397 s | 86.0 mph | +19.9 in |
| Same, 90% spin efficiency | 0.397 s | 86.0 mph | +18.8 in |
| Curveball 80 mph, 2600 rpm pure topspin | 0.475 s | 73.2 mph | −23.5 in |
| Changeup 85 mph, 1700 rpm backspin | 0.443 s | 77.5 mph | +17.9 in |

These use the internal full-flight definition (§12) and idealized pure spin axes. Real pitches have tilted axes and lower efficiency, so Savant-style numbers will be smaller. M1 compares Savant-definition outputs against public league-average ranges as a sanity check. Large disagreement means a model or definition error.

Additional batted-ball cross-check points (same model, calculator atmosphere):

| Exit velocity / launch angle / backspin | Carry | Hang time | Apex |
| :--- | :--- | :--- | :--- |
| 100 mph / 28° / 2200 rpm | 410 ft | 5.37 s | 88 ft |
| 105 mph / 28° / 2400 rpm | 433 ft | 5.67 s | 97 ft |
| 95 mph / 15° / 1500 rpm | 319 ft | 3.27 s | 31 ft |
| 90 mph / 45° / 3500 rpm | 308 ft | 6.13 s | 129 ft |

---

## 15. Parameter Summary (all configurable)

| Group | Parameters | Lives in |
| :--- | :--- | :--- |
| Ball | m, R, α | `physics.toml` |
| Air | ρ inputs per park; wind (zero in v1) | Park config |
| Aero | cd0, cdspin, cl0, cl1, cl2, τ₁₀₀ | `physics.toml` |
| Integration | Δt per phase, event tolerance | `physics.toml` |
| Bat | Profile table, ρ_w, cup option; per-player bat selection | Equipment config |
| Bat-ball | f(u) profile, u_ss, e_ref, v_ref, s_e, e_t, μ, second-contact limit | `physics.toml` |
| Surfaces | e_ref, k_s, μ, e_t, μ_r, σ_n per surface | Park config (with defaults) |
| Field objects | Wall segments (height, material), poles, bases, backstop | Park config |
| Zone | 0.27 / 0.535 height fractions, plane depth | League rules config |

---

## 16. References

- D. Kagan and A. M. Nathan, "Statcast and the Baseball Trajectory Calculator," *The Physics Teacher* 55, 134 (2017). Coordinate frame and calculator usage. https://physics.csuchico.edu/baseball/Pubs/Statcast.pdf
- A. M. Nathan, "The Spin Decay of a Baseball." Spin-decay model, τ ≈ 20–25 s at 100 mph, golden case A1. https://baseball.physics.illinois.edu/spindown-rev1.pdf
- A. M. Nathan, Trajectory Calculator. https://baseball.physics.illinois.edu/trajectory-calculator-new3D.html
- `pybbda` trajectory module, a third-party port of the calculator. Coefficient defaults and air-density formula. https://pybbda.readthedocs.io/en/latest/modules/analysis/trajectories.html
- J. Kensrud, A. M. Nathan, L. Smith, "Oblique Collisions of Baseballs and Softballs with a Bat," *Am. J. Phys.* (2017). e_x, μ, gripping relations. https://arxiv.org/abs/1610.03464
- R. Cross and A. M. Nathan, "Scattering of a Baseball by a Bat," *Am. J. Phys.* 74, 896 (2006). https://arxiv.org/abs/physics/0605040
- A. M. Nathan, "The effect of spin on the flight of a baseball," *Am. J. Phys.* 76, 119 (2008).
- A. M. Nathan, Physics of Baseball site (bat COR, collision efficiency q, aerodynamics). https://baseball.physics.illinois.edu/
- MLB, 2026 ABS challenge-system zone definition (27% / 53.5% of height, measured at plate midpoint). https://www.mlb.com/amp/news/ball-strike-challenge-system-2026.html
- Official Baseball Rules (field, ball, and bat specifications).

## 17. Open Items

1. Obtain Nathan's current Trajectory Calculator spreadsheet; confirm the C_D/C_L forms and coefficients match the `pybbda` port (§4.2), including whether the C_D spin term uses total or transverse spin.
2. Pull published COR-versus-impact-location data for wood bats to set f(u) (§8.2) and the speed slope s_e.
3. Pull spin-versus-undercut data from Nathan et al.'s batted-ball spin work for test A10.
4. Find literature values for ball-turf and ball-dirt bounce (COR, friction) and rolling resistance to replace the [E] surface table (§9.1).
5. Confirm the ball's moment-of-inertia factor α for a real baseball.
6. Confirm the exact ABS edge rule ("any part of the ball") if the league ever wants strict ABS parity. It doesn't need to; the league is fictional.
7. Tune the default bat profile to a typical MLB 34-in mass (about 31–32 oz) and add 2–3 more stock bats (lighter/longer, heavier/end-loaded) for player variety.
