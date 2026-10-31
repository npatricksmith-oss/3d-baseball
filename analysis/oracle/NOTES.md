# Independent baseball physics oracle

This implementation was derived only from `docs/physics-spec.md` and
`docs/data-formats.md`, following `analysis/oracle/TASK.md`. No files under
`sim/` were read. All created files are under `analysis/oracle/`. It uses
Python 3's standard library and performs no network access.

Run the checks from the repository root:

```
python3 -m unittest analysis/oracle/test_oracle.py
python3 analysis/oracle/gen_golden.py
```

The tests also run directly with `python3 analysis/oracle/test_oracle.py`.
Both invocations passed all 20 tests. The generated JSON was also checked for
exact record keys, finite values, and numeric spin-axis fields.
The generator resolves its output directory relative to its own file and
writes JSON with Python's full round-trip float representation (`json.dumps`,
indentation 2, no NaN or Infinity). Its five output files follow the task's
specified shapes exactly. The computed-results section below is regenerated.

## Interpretation choices

- The task takes precedence over §5.1: RK4 advances all nine components
  `(r, v, omega)`, including the continuous spin-decay ODE, at dt = 1e-4 s.
  Spin is not held constant within a step. The public integrator can take a
  different dt for A3. `tau100=None` means no decay; positive values are seconds.
  Airspeed drives decay even when density is zero, literally following §4.4.
- Ground is the ball-center plane z = 0, as requested, rather than sphere-surface
  contact at z = R. Ground and plate events require decreasing coordinates.
  Cubic Hermite interpolation locates events with a time bracket at most
  1e-10 s wide. The returned state comes from a partial RK4 step and is not
  snapped to the target plane. Residual coordinates of order 1e-9 m are expected.
  Apex is evaluated by a partial step at the Hermite vertical-velocity root,
  and includes both the initial position and the terminal position.
- The flat-ended bat uses every listed profile segment, including the complete
  22–34 in barrel. **End-cap rounding is ignored.** Quadrature splits at all
  segment boundaries. An adaptive Simpson integral computes mass and inertias;
  tests independently check the result against closed-form power integrals.
- The A1/A2 and §14.3 batted-ball fixtures start at `(0, 0, 3 ft)`, with velocity
  in the Y–Z plane and spin along +X. All §14.3 pitch fixtures release at
  `(0, 54 ft, 6 ft)` aimed level, purely along −Y, and stop at the plate
  midpoint y = 0.2159 m. Both sets use calculator-default density and **no spin
  decay**, except A2. Other named fixtures explicitly label their decay time.
- The 90%-efficient four-seamer retains 2300 rpm total spin: 90% is transverse
  along −X and the remaining gyro magnitude is `sqrt(1 - 0.9²)` along −Y.
  No spin-rate reduction is used for drag. Other cases include mixed spin,
  side spin, gyro spin, lateral releases and lateral velocities.
- No-Magnus twins retain the actual trajectory's total spin, drag law, and
  spin-decay setting; only the Magnus term is disabled. Each trajectory stops
  at its own crossing time of the same plate plane. Reported full-flight break
  is actual minus twin position. This is not a fixed-time comparison.
- §12 mentions a Savant 40-ft variant without specifying its exact window or
  fitting procedure. These fixtures report the explicitly defined internal
  full-flight movement only. No public Savant sample or spreadsheet was fetched,
  because this task requires no network access. External calibration remains
  unverified; these outputs establish the stated equations' predictions.
- Projected distance follows §12 literally: horizontal distance from the world
  origin. `flight_telemetry` also supplies a separately named travel distance
  from the initial position. Launch and spray angles use `atan2` and degrees.
  `plate_strike` expects a position already evaluated at the midpoint plane
  and implements inclusive circle/rectangle overlap, including rounded corners.
- The mirror operation is X reflection for polar vectors (position/velocity),
  but `(wx, wy, wz) -> (wx, -wy, -wz)` for axial spin vectors. Merely flipping
  wx would not mirror the Magnus force correctly.
- The collision function accepts the task's explicit world inverse-inertia
  matrices. The ball's generated inverse inertia uses `alpha=0.4` and the exact
  circumference-derived R. Matrices are row-major. World bat inertia uses the
  axis projection formula in the task; rotated-axis cases exercise its
  off-diagonal entries. A normal must be unit length. All supplied contacts
  are approaching (`c_n < 0`); separating contacts raise an error rather than
  inventing an extra output branch. Fully immovable bodies contribute zero
  energy; a moving infinite-mass surface is not used in the corpus.
- A7's analytic formula assumes its normal and tangential responses decouple.
  The A7 fixtures therefore use `r2 = (u - u_cm, 0, 0)` with bat axis +X and
  normal +Y (the slender-bat analytic check). A finite radial component of r2
  generally couples the normal/tangent constraints and changes that reduction.
  Separate oblique cases use finite barrel-radius offsets. A8 chooses the
  restitution derived from the computed mass properties to yield q = 0.20;
  it does not implement or tune the unspecified speed/location COR profile.
- For the A9 tangential check, the bat contact offset is along +X and the
  normal is +Y. With tangent +Z, define
  `B_t = inv_M2 + b²/I_cm` and `r_x = m*alpha*B_t/(1+alpha)`.
  This gives the stated KNS spin relation and
  `v_ball_t = alpha*R*abs(omega_x)` for an initially stationary ball.
- The sticking impulse, friction-cone test, and sliding direction use §8.1
  literally. Energy is evaluated from both bodies' translational and rotational
  states. The guard compares against a 64-ULP allowance at the pre-impact
  energy scale to ignore floating-point roundoff, then retries exactly once
  with e_t = 0. If even that retry gains energy, the solver raises an error;
  it does not rescale impulses or silently clamp restitution. Inertias must be
  positive definite or exactly zero for an immovable body. Degenerate sliding
  with zero c_t or a nonpositive denominator also raises an explicit error;
  the spec gives no fallback for either condition.
- `energy_guard_stress` deliberately exercises the general solver outside
  ordinary bat-contact geometry: r2 = (0.3, 0.03, 0.2) m, e_n = 0.8,
  e_t = 0.9, mu = 20. Its first sticking candidate falls outside the cone;
  the resulting sliding fallback gains energy. The e_t = 0 retry selects a
  dissipative sticking solution. This is a guard test, not a
  proposed baseball material or bat surface point.
- Spin-axis/tilt use the release velocity, not a fixed −Y axis. The inverse
  helper `spin_from_tilt` is explicitly for level −Y pitches. Tilt is numeric
  hours in [0,12), so pure backspin is 0 (displayed as 12:00) and pure topspin
  is 6. Axis/tilt are undefined for zero or pure gyro spin: the helper returns
  `None` and efficiency remains numeric. Those undefined axes are tested but
  omitted from the numeric JSON fixtures. A relative 1e-14 cross-product
  threshold recognizes numerical pure-gyro alignment. A zero X–Z projection
  is also undefined. Display rounding to 15-minute text is outside these numeric
  fixtures. The four §9.2 checks and inverse tilt/efficiency/gyro-sign round trips
  are included in the tests.

## Spec inconsistencies and limits

- The radius and area in §2 are rounded. R is derived from the exact requested
  circumference (0.03688813693512404 m), rather than the rounded 0.036888 m.
- The spec's split spin update and this task's continuous ODE differ. The task's
  full-state ODE is the reference used here.
- §6.2's default mass of about 0.965 kg lies outside its own 0.85–0.95 kg
  validation band. A12 cannot simultaneously match that default and meet the
  mass band. The profile is preserved rather than changed to satisfy the band.
  The tabulated transverse and 6-in inertias are approximate rather than exact
  reproductions of the specified flat-ended profile.
- The §8.1 A7 reduction needs a decoupled normal response. At a general finite
  surface radius, transverse sticking impulses can affect normal effective
  mass. The simple `1/M_eff = 1/M + b²/I_cm` formula does not account for this
  coupling; analytic fixtures make the required idealization explicit above.
- §8.1's guard does not specify a numeric tolerance, handling for a still-unsafe
  retry, degenerate sliding directions, or separating contacts. The explicit
  choices above avoid silently returning a physically invalid result.
- §14.2's A1 note includes y0 = 2 ft for its 407.7-ft cross-check, while the task
  explicitly requires y0 = 0. This accounts for the approximately 2-ft offset.
  §14.3's batted-ball carry points appear consistent with that same offset;
  the generated results below keep the requested zero Y origin.
- §14.3 says “54 ft release distance,” but a release at y = 54 ft and a midpoint
  crossing at y = 0.2159 m travel only 53.2916667 ft in Y. Small differences
  from its rounded pitch cross-checks are reported without changing the inputs.
  In particular, its flight times do not reproduce exactly: the level-pitch
  midpoint-plane results are about 0.006 s longer. The spec does not fully
  identify the original aiming and termination choices, so no cause is assumed.
- Tangential restitution and the COR-vs-speed/station profile are calibration
  inputs, not uniquely determined by the spec. The solver accepts explicit
  e_n/e_t/mu; generated cases state all of them. The task does not request
  rolling, surface noise, bat contact search, or external empirical calibration.

<!-- GENERATED RESULTS -->

## Computed results

Generated by `python3 analysis/oracle/gen_golden.py` at dt = 0.0001 s.

Default air density: 1.19376483342215 kg/m³.

| Case | Carry (ft) | Time (s) | Apex (ft) |
| :--- | ---: | ---: | ---: |
| A1 | 405.727659184 | 5.618831547 | 97.295524933 |
| A2 | 405.807232216 | 5.587617642 | 96.923978595 |
| batted_100_28_2200 | 408.084558236 | 5.366441610 | 87.831723068 |
| batted_105_28_2400 | 431.092682061 | 5.669171241 | 97.099969318 |
| batted_95_15_1500 | 316.677656182 | 3.267908829 | 30.718697430 |
| batted_90_45_3500 | 306.418132455 | 6.134727140 | 129.405526387 |

Pitch results at y = 0.2159 m; movement is relative to a full-flight
no-Magnus twin, retaining the same total-spin drag and decay settings.

| Pitch | Time (s) | Plate speed (mph) | IVB (in) | Horizontal (in) |
| :--- | ---: | ---: | ---: | ---: |
| four_seam_95_2300 | 0.403140302 | 85.668191272 | +20.348038910 | +0.000000000 |
| four_seam_95_2300_eff90 | 0.403133192 | 85.683323366 | +19.207358430 | +0.104542255 |
| curve_80_2600 | 0.480852148 | 72.864356284 | -23.913197930 | +0.000000000 |
| change_85_1700 | 0.449315195 | 77.204013424 | +18.290948742 | +0.000000000 |

Computed flat-end bat:

| Quantity | SI value |
| :--- | ---: |
| M | 0.964473486464458 |
| u_cm | 0.597742530566319 |
| I_cm | 0.0383449386528473 |
| I_ax | 0.000396106371065318 |
| moi_6in | 0.229628935837006 |
| L | 0.8636 |

Corpus: 6 air-density, 21 flight, 18 collision (15 stick, 3 slide), and 8 spin-axis cases.

Energy guard fired for: energy_guard_stress.

A1 and A2 satisfy their stated bands. The §14.3 batted-ball points differ
by about 2 ft from the rounded listed carries; the spec’s A1 cross-check
explicitly used y0 = 2 ft, whereas these fixtures use the requested y0 = 0.
The four §14.3 pitch cross-checks are not reproduced exactly: flight times
are 0.00585–0.00632 s longer, plate speeds are 0.296–0.336 mph lower,
and vertical-movement magnitudes are 0.391–0.448 in larger. The original
aim and termination choices are not fully specified; these fixtures retain
the task’s level releases and midpoint crossing. Release-to-midpoint
travel is 53.2916667 ft rather than a full 54 ft.

The bat rounds close to §6.2 (the quoted I_cm and 6-in MOI differ by
about 0.38% and 0.27%). Its mass, 0.96447 kg, exceeds the stated 0.85–0.95 kg
validation band; this is already inconsistent with the spec’s own 0.965 kg default.
No coefficients, geometry, or golden outputs have been adjusted to force those checks.
