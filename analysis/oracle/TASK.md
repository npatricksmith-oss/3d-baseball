# Oracle task (for Codex)

You are writing an **independent Python reference implementation** ("oracle") of the baseball physics in
`docs/physics-spec.md`. A separate Rust implementation is being written by another agent; your oracle is used
to cross-check it, so **do not read anything under `sim/`** and work only from the spec documents.

Constraints:
- Python 3 **standard library only** (no numpy/scipy; no network). Use `math`, `json`, `dataclasses`.
- All code in `analysis/oracle/`. Main module `analysis/oracle/bbphys.py`; generator script
  `analysis/oracle/gen_golden.py` that writes `analysis/oracle/golden/*.json`; self-tests in
  `analysis/oracle/test_oracle.py` runnable with `python3 -m unittest analysis/oracle/test_oracle.py` from the
  repo root (or `python3 analysis/oracle/test_oracle.py`).
- Follow the spec exactly (sections 1-5, 6.1-6.2, 8, 10, 12, 14). Where the spec is ambiguous, pick the most literal
  reading and write the choice down in `analysis/oracle/NOTES.md`. Also list in NOTES.md anything in the spec you
  believe is wrong or inconsistent (e.g. if golden values in §14 don't reproduce).

## What to implement
1. Constants and unit conversions (§1.1, §2). Ball m = 0.14529 kg, R = circumference/(2π) with circumference 0.231775 m.
2. Air density (§3), function of (temp_F, pressure_inHg, rh_percent, elev_ft). Check: default env -> 1.1938.
3. Flight acceleration (§4.2) with spin decay (§4.4) as a continuous ODE dω/dt = −(u/(τ100·44.704))·ω
   (τ100 = None means no decay).
4. A **high-accuracy reference integrator**: classic RK4 on the full state (r, v, ω) with a small step
   (dt = 1e-4 s), with events (ground z = 0 crossing with z decreasing; plane y = y_target crossing with y decreasing)
   located by cubic Hermite interpolation + bisection to 1e-10 s, and the state at the event obtained by
   re-integrating a partial step from the step start. Also track apex z.
5. Bat profile (§6.1) and mass properties by numerical integration (§6.2) for the default 34-in profile with
   ρ_w = 660 kg/m³ — report M, u_cm, I_cm, I_ax, MOI about the 6-in point. Treat the profile as described (knob,
   linear knob→handle, handle, power-law taper, barrel); **ignore the end-cap rounding** (flat end) and say so in NOTES.
6. The collision impulse solver of §8.1 exactly as written (sticking solution, friction-cone check, sliding
   fallback, energy guard with e_t = 0 retry). Inputs: ball (m, I_b = α m R², α = 0.4, v, ω, r1), body 2
   (inverse mass, 3x3 world inverse inertia, V, Ω, r2), n̂, e_n, e_t, μ. Outputs: impulse J, post v, ω, V, Ω,
   which branch was taken ("stick"/"slide"), and whether the energy guard fired. Inverse inertia of the bat in
   world frame from I_cm (transverse) and I_ax with axis â: I⁻¹ = (1/I_cm)(𝟙 − ââᵀ) + (1/I_ax) ââᵀ.
7. Spin axis / tilt per `docs/data-formats.md` §9 (spin_axis_deg from ω and v̂ at release; tilt hours).

## Golden output (exact JSON shapes — the Rust tests parse these)
All numbers SI unless a key says otherwise. Write pretty JSON with full float precision (repr).

`golden/air_density.json`: `[{"temp_f":..,"pressure_inhg":..,"rh_pct":..,"elev_ft":..,"rho":..}, ...]` — at least
the default env plus 4 other combinations (hot, cold, Denver elevation 5280 ft, humid).

`golden/flight.json`: list of cases:
```
{"name": str, "r0":[x,y,z], "v0":[..], "w0":[..], "rho": float, "tau100": float|null,
 "stop": {"kind":"ground"} | {"kind":"plane_y","y": float},
 "result": {"t": float, "r":[..], "v":[..], "w":[..], "apex_z": float}}
```
Include at least: A1 (100 mph, 30°, 2500 rpm pure backspin, z0 = 3 ft, y0 = 0, x0 = 0, calculator default rho,
no decay, stop at ground); A2 (A1 with τ100 = 20 s); vacuum A4 case (rho = 0, spin nonzero); the four §14.3
pitches (release at y = 54 ft... i.e. 16.4592 m, z = 6 ft, x = 0, aimed level i.e. v0 purely −Y unless you
choose otherwise — record your choice; stop at plane y = 0.2159 m (plate midpoint) ); the four §14.3 batted-ball
cross-check points (z0 = 3 ft, stop at ground); and at least 8 extra cases with mixed tilted/gyro spin, sidespin,
and nonzero x0/vx (pitches and batted balls), plus a mirror pair for A6.

`golden/bat_default.json`: `{"M":..,"u_cm":..,"I_cm":..,"I_ax":..,"moi_6in":..,"L":..}`

`golden/collision.json`: list of cases:
```
{"name": str, "m": float, "inv_I1": [[3x3]], "v":[..], "w":[..], "r1":[..],
 "inv_M2": float, "inv_I2": [[3x3]], "V":[..], "W":[..], "r2":[..],
 "n":[..], "e_n": float, "e_t": float, "mu": float,
 "result": {"J":[..], "v":[..], "w":[..], "V":[..], "W":[..], "branch": "stick"|"slide", "energy_guard": bool}}
```
At least 12 cases: head-on bat impact (A7 style) at several stations of the default bat, oblique stick cases,
oblique slide cases, ground-bounce cases (immovable: inv_M2 = 0, inv_I2 = 0), and one case that trips the
energy guard if you can construct one (say in NOTES if you cannot).

`golden/spin_axis.json`: list of `{"w":[..], "v":[..], "spin_axis_deg":.., "tilt_hours":.., "efficiency":..}`
including the four §9.2 checks.

## Self-tests
test_oracle.py must check: air density default ≈ 1.1938; A1 carry 406 ± 3 ft and 5.6 ± 0.1 s; A2 within 1 ft of A1;
vacuum case vs analytic to 1e-9 relative; §1.3 Magnus sign table; A7 analytic q formula; energy never increases
in all collision cases; spin_axis §9.2 checks. Run it and make sure it passes, then run gen_golden.py.

Finish with a short summary in NOTES.md of results, including the §14.3 plausibility-band numbers your oracle gets.
