# Review: body-motion design (`work/body-motion`, `c8f5e04..d0def11`)

*Reviewer: `gemini-neon` (Gemini 4 Neon, independent review), 2026-11-25. Scope: `docs/body-motion.md`, `analysis/research/body/*`, `analysis/research/fortenbaugh_appendix_e.py` diff, `docs/research/body/0-owner-walkthrough.md` through `docs/research/body/7-datasets-and-licences.md`, and `docs/research/body/README.md`. No source, test, spec, probe, note, PLAN.md, TASKS.md or observations.md edited. Scratch probes and outputs: `sim/target/review-probes/gemini-neon/`.*

## Summary and verdict

**Verdict: sound with named fixes — fix P1-1 through P1-3 before putting §14 to the owner or starting phase 1.**

The core architecture (forward multibody dynamics in `bb_physics::body`, proximal torque generators with passive/elastic distal whip, postural feedback in balance phases and pure feedforward through the swing, roster-time learned swings blended per pitch, and a two-strength-group rating split) is a sound, D31-faithful foundation for phase 1. All six fast probes reproduce bit for bit, and every male segment row in `common.py` matches de Leva 1996 Table 4 exactly.

Three P1 problems block signing off as written: (1) a bug in `breakdowns.py` (`shifted()` shifts `t_on` but leaves `t_off` fixed) manufactured the lunging, late-foot-wait, hips-early, and late-swing figures, including numbers put directly to the owner in §14; (2) the fitted drive (`fit_params.json`) breaks §5.4's decision clock (`on_sh = -231 ms`, before `on_legs = -228 ms`), hides 4–5× physiological strength caps behind slow ramps, and fails to fit Fortenbaugh [4] once those rules are enforced; and (3) §9's 100-byte drive-program replay silently reopens PLAN D9, while §5.3/§5.5/§10 leave AGENTS.md invariants 1–3 broken or unspecified for proprioceptive noise, mid-swing updates, and moving eyes.

---

## Findings

### P1-1. `breakdowns.py` forgets to shift `t_off`, manufacturing the lunging, late-foot-wait, hips-early, and late-swing results (including numbers in §14)

**Where.** `analysis/research/body/breakdowns.py:68–72, 89–108, 124`; `analysis/research/body/ratings.py:167`; `docs/body-motion.md` §1 (lines 29–33), §6.1 (line 367), §6.2 (lines 380–391, 398, 401, 404), §14 (line 638).

**What is wrong.** `breakdowns.shifted(plan, names, dt)` shifts `p.ch[n][1]` (`t_on`) by `dt` but leaves `p.ch[n][2]` (`t_off`) untouched (contrast `ratings.plan_for` line 39, which correctly shifts `t_off` whenever `t_off < 0.9`). In `fit_params.json`, `legs` runs from `-228 ms` to `-52 ms` (176 ms duration, 138 ms ramp) and `core` from `-238 ms` to `+35 ms`. Delaying `t_on` without shifting `t_off` truncates the drive mid-ramp; advancing `t_on` lengthens it. In addition, `breakdowns.py:124` prints `sim.cap` after `sim.run(t_end=0.12)` finishes — reading the friction cap 120 ms *after* contact, not during hip rotation.

**Evidence** (recomputed in `sim/target/review-probes/gemini-neon/diag_probes.py`, Probe 3):

| Case | Spec / `breakdowns.py` (`t_off` unshifted) | Fixed (`t_off` shifted with `t_on`) | Peak legs / cap during swing |
| :-- | :-- | :-- | :-- |
| `lunging: hips held 100 ms` | `+114.9 ms`, `0.435 m` (−20 cm), `17.6 m/s`, pel `−1°` / trk `12°` | **`+97.9 ms`, `0.617 m` (−1.7 cm), `29.3 m/s`, pel `61°` / trk `54°`** | `210 / 217 N·m` (unclipped) |
| `lunging: hips held 60 ms` | `+63.2 ms`, `0.560 m`, `25.1 m/s`, pel `36°` / trk `44°` | `+58.7 ms`, `0.623 m`, `29.9 m/s`, pel `62°` / trk `58°` | `206 / 241 N·m` |
| `late foot +160, hips wait 77` | `+84.7 ms`, `0.511 m` (−12 cm), `22.0 m/s`, pel `22°` / trk `33°` | **`+76.1 ms`, `0.621 m` (−1.3 cm), `29.7 m/s`, pel `62°` / trk `56°`** | `207 / 233 N·m` |
| `late foot +120, hips wait 37` | `+38.4 ms`, `0.599 m`, `27.9 m/s`, pel `50°` / trk `54°` | `+36.8 ms`, `0.627 m`, `30.2 m/s`, pel `63°` / trk `60°` | `205 / 301 N·m` |
| `hips early 40 ms` | `−7.0 ms`, `0.643 m`, **`31.8 m/s`** (+1.1 m/s), pel `101°` | `−5.8 ms`, `0.625 m`, **`30.1 m/s`** (−0.6 m/s), pel `90°` | `231 / 301 N·m` |
| `late swing 15 ms` | `+15.0 ms`, `0.623 m`, `29.8 m/s` (−0.9 m/s) | `+14.7 ms`, `0.632 m`, `30.6 m/s` (−0.1 m/s) | `204 / 288 N·m` |

1. **Lunging ("the pelvis never turns, arms only") was an artifact of cutting `legs` down to 76 ms** (barely half its 138 ms ramp). With duration preserved, the pelvis still turns to `61°` (vs `64°` normal) and bat speed is `29.3 m/s`. Even at `s = 0.53 m` on a `0.77 m` stride, the lever rule leaves 31% BW on the rear foot, giving `cap = 217 N·m` — still *above* the peak legs torque (`204–210 N·m`). The spec's `"cap 212 N·m against 251"` (§6.2 line 404) compared two post-contact `t = +120 ms` values (normal swing peak cap is `296 N·m`, and `legs` clips `0/540` steps).
2. **`late foot +160, hips wait 77` inverts the speed trade-off in §14.** §14 tells the owner option (b) yields `22 m/s` against option (a)'s `25 m/s`; with `t_off` shifted, option (b) keeps **`29.7 m/s`** and `0.621 m` (truly "intact", just `+76 ms` late).
3. **`hips early 40 ms` only gained speed (`31.8 m/s`) because leaving `off_legs` fixed added 40 ms of extra leg drive.** With duration preserved it already *loses* `0.6 m/s` without needing trunk force–length (though force–length is still physically worthwhile in 3D).
4. **§6.1's sensitivity baseline** ("15 ms late start ... at −0.9 m/s") drops to `−0.1 m/s`. `ratings.py:167` also calls `breakdowns.shifted` for its `10 ms` late-start column.

**Fix.** Shift `p.ch[n][2]` alongside `p.ch[n][1]` whenever `p.ch[n][2] < 0.9` in `breakdowns.shifted()`, record peak `tau["legs"]` vs `cap` during the hip-turn window rather than at `t_end`, re-run `breakdowns.py` and `ratings.py`, and update §1, §6.1, §6.2, and §14 before sending §14 to the owner. Note honestly in §6.2 that reproducing "arms-only" lunging requires 3D front-knee collapse / CoM-over-foot unloading that the planar lever rule cannot produce.

---

### P1-2. The fitted drive breaks §5.4's decision clock, §1's proximal-to-distal order, and §4.3's strength cross-checks — and fails to fit [4] under those rules

**Where.** `analysis/research/body/fit_params.json`; `analysis/research/body/fit.py`; `docs/body-motion.md` §1 (line 15), §4.3 (lines 181–197), §5.1 (line 219), §5.4 (lines 303–320).

**What is wrong.**
1. **Backwards onset order and broken decision clock.** §1 says drives fire proximal to distal and §5.4 says the batter's decision starts the arms after `t_commit + T_latency` (`[4]`'s `-170 ms` first move), with `T_swing` replaced by the learned swing's time from arms onset to contact. In `fit_params.json`, **`on_core` (`-238 ms`) fires first, `on_sh` (`-231 ms`, `rise_sh = 23.7 ms`) fires second, and `on_legs` (`-228 ms`) fires third.** A `231 ms` arm duration leaves only ~119 ms of ball viewing on a 400 ms fastball at 50 ms latency (or 69 ms at the walkthrough's 100 ms latency).
2. **Hidden speedup in §5.4's "arms committed" table.** Because the shoulder fired 61 ms too early in the fit (fighting the coiling trunk at `-190 ms`, where trunk velocity dips from `-48` to `-66°/s`), delaying the arms by 30 or 60 ms *raises* bat speed from `30.7` to `33.3` and **`34.0 m/s`** while spinning the trunk to `128°` (and `164°` at +90 ms). §5.4's table replaced `Speed` and `Trunk` at 60 and 90 ms with `—`, obscuring that the probe's arms are wildly too early.
3. **Unphysiological strength parameters and apples-to-oranges cross-checks.** `fit_params.json` sets isometric `T_legs = 1,202 N·m` (`1,065 N·m` at `0.5 rad/s` — ~4× Anderson 2007's hip extension), `T_core = 352 N·m`, `T_sh = 253 N·m`, and `T_lead = 960 N`, hiding the oversized torques behind slow linear ramps (`rise_legs = 138 ms`, `rise_core = 192 ms`) and steep Hill decay (`WMAX = 20–30 rad/s`). §4.3 compares the model's *applied* peaks (`219`, `157 N·m`) against MacKenzie & Sprigings's *isometric `Tm`* (`200`, `160 N·m`; MacKenzie's applied peaks were `136` and `85 N·m`, so the model is 1.6–1.85× higher on both `Tm` and applied torque) and against mass-scaled pro golfers' *maximal voluntary* trunk strength (`173 N·m` at `0.52 rad/s`), where the model's actual strength `T_core · fv(0.52/25)` is **`318 N·m` (1.84×)**, not `1.3×`. §4.3 line 196 also miscomputes the Hill drop to `2.1 rad/s` as `31%` (comparing `fv(2.1/25)` against `fv(0) = 1.0`, instead of against `fv(0.52/25) = 0.904`, which is **`24%`**).
4. **Enforcing the spec's own constraints breaks the fit.** In `diag_probes3.py`, constraining only `on_sh >= -0.17` and `rise_sh >= 0.05` (§5.1's own floor) quadruples the cost (`0.52 -> 2.063`), drives `T_sh` to `613 N·m`, sends bat speed to `35.5 m/s` (`2,858°/s`), and inverts the kinematic sequence (trunk peaks at `-64 ms` before pelvis at `-62 ms`). Adding physiological torque caps (`T_legs <= 400`, `T_core <= 220`, `T_sh <= 180`, `T_wr <= 20`) — even with doubled `WMAX` (`40/50/60 rad/s`) — fails to reach contact by `+40 ms` (cost `4,275` at default `WMAX`, `1,789` at doubled `WMAX`).
5. **Unidentifiable and non-reproducible fit.** Four of 17 parameters (`T_wr`, `on_wr`, `rise_wr`, `off_core`) have exact zero cost sensitivity (`dCost = 0.000000`, Probe 5), and `T_lead` (`960 N`) is friction-clipped (`217/540` steps, `dCost = 1.9e-5` for ±10%). Because `fit.py` warm-starts from `fit_params.json` rather than `X0` (whose cold-start cost is `13.91`), rerunning `fit.py 400` (job 307, 143 s) changed `fit_params.json` (`0.5203 -> 0.5150`), drifting dead `T_wr` (`59.95 -> 69.48 N·m`, which directly changes `breakdowns.py`'s casting drive!), `T_rear` (+25%), `T_lead` (+17%), and `on_lead` (−18 ms).

**Fix.** In §4.3, report `T` alongside applied peaks, compare isometric-to-isometric and applied-to-applied against [N1] and [N5], and fix `31% -> 24%` and `1.3× -> 1.8×`. Show the full 60/90 ms rows in §5.4 and flag the `-231 ms` arm onset honestly. Make phase 1's 3D fit gate enforce proximal-to-distal onsets (`on_arms >= t_commit + T_latency`), `rise >= 50 ms`, and physiological torque bounds (dropping dead channels from the vector and fitting from a clean, documented starting point).

---

### P1-3. Timeline replay (§9) silently reopens PLAN D9, and §5.3/§5.5/§10 leave AGENTS.md invariants 1–3 broken or unspecified

**Where.** `docs/body-motion.md` §4.1 (line 116), §5.3 (line 292), §5.4 (lines 304, 308, 322), §5.5 (lines 338–343), §6.1 (line 369), §9 (line 547), §10 (lines 553–568).

**What is wrong.**
1. **Direct conflict with PLAN D9.** §9 recommends storing only the ~100-byte executed drive program in `PlayTimeline` and having the client re-integrate the body through `bb_ffi`. PLAN D9 explicitly mandates: *"Replays store recorded timelines, not seeds. ... A saved play stores the build's commit and the recorded play itself, so the client shows it exactly as seen on any later build. Rationale: Re-simulation drifts as code changes."* Re-integrating a drive program on a future build's `bb_physics::body` will silently drift saved plays whenever segment data, contacts, stops, or the integrator change. Worse, a single static `(t_on, t_rise, t_off, T)` program cannot even replay mid-swing re-blends (§5.5), check-swing antagonist braking (§5.4), or per-step proprioceptive sensing noise (§5.3). §9 rejects pose recording because `16 segments × 7 floats × 240 Hz` is ~200 kB/pitch, overlooking that storing the 12-DOF core angles at 60 Hz for 0.6 s is only ~1.7 kB/pitch (~500 kB/game) — 100× smaller and fully D9-compliant.
2. **Invariants 1 and 3 conflict on proprioceptive feedback and sensory triggers.** §5.3 says the postural reflex reads *"his true body state with his own sensing noise, an input of `bb_agents::motor`, not of his decisions,"* and §5.4 lets the heel-plant event (lead-foot load crossing 50% BW during world integration) trigger the hips. Yet §6.1 says every error is drawn from `KeyedRng` *before* the motion starts, §10 hands a single `ExecutedDrive` to `bb_world`, and AGENTS.md invariant 1 restricts world-side randomness to bounce-normal tilt alone (`physics-spec §9.3`). As written, there is no legal place for per-step proprioceptive sensing noise: `bb_world` cannot draw it (invariant 1), `bb_agents::motor` cannot read true body state during integration (invariant 2/3), and pre-drawing a per-step noise buffer is neither specified in §6.1/§10 nor compatible with §9's 100-byte replay.
3. **Unspecified world↔agent loop for moving eyes and mid-swing commands.** §4.1 says pulling off moves the eye point into perception, and §5.4–§5.5 allow mid-swing re-blends and check-swing braking after commit. That requires `bb_game` to step `bb_world`'s true body in 10 ms perception ticks, build `bb_observe` angles from the true eye without leaking true body/eye state to `bb_agents`, and route updated intended commands back through `bb_agents::motor` (with their own keyed draw sub-indices) into `bb_world`. §6.1's pre-motion draw and §10's "pitcher's motion first, then batter's" omit this coupling entirely.

**Fix.** (1) Store sampled poses (e.g. 60–120 Hz core coordinates + contact/release events, ~2–5 kB/pitch) in `PlayTimeline` so saved plays never drift across builds (or put reopening D9 explicitly to the owner in §14); (2) define a tick-indexed proprioceptive noise input pre-drawn (or drawn per 10 ms tick) by `bb_agents::motor` under a new appended `Purpose` and passed as read-only data into `bb_world` (zero RNG inside `bb_world`); (3) specify the 10 ms `bb_game` interleaving between `bb_world` body stepping, `bb_observe` eye-point projection, `bb_agents` mid-swing/check decisions, and `bb_agents::motor`.

---

### P2-1. `chain.py` hides an uncapped 737 N lateral ground force, decoupled friction caps, and `−71.5 N·m` sticky wrist-stop damping

**Where.** `analysis/research/body/chain.py:126, 175–190`; `docs/body-motion.md` §4.2, §4.3 (lines 194, 201–208), §5.3 (lines 284–290).

**What is wrong.**
1. **Uncapped `0.82 BW` lateral constraint force.** Because `chain.py` fixes `x_base = 0` (no lateral DOF), the ground silently applies whatever lateral force `Fx` is required to keep the trunk axis on `x = 0` against the whipping arms (9.1 kg) and bat (0.965 kg). Probe 2 measures **`|Fx| = 736.5 N` (0.82 BW)** near contact (`t = +19 ms`), pushing total horizontal ground force `√(Fx² + Fy²)` to **`1.16 × μW` even before adding the yaw couple**. Moreover, `chain.py:175–180` caps `rear`, `lead`, and `legs` independently instead of enforcing one Coulomb friction cone `√(Fx² + Fy²) ≤ μ Fz` per foot. In 3D with a floating pelvis and real foot friction cones, this lateral pull will shift the CoM toward the plate, load the feet asymmetrically, and interact with friction limits — so the planar probe has *not* established that a 3D swing needs no feedback or stays inside ground friction.
2. **Sticky wrist-stop damper.** Unlike `SEP_STOP` (line 185, which gates damping to penetrating velocity only), line 189 writes `2.0e4 * (lag - lag_stop) + 40.0 * (qd[3] - qd[4])` with no one-sided clamp. While the bat begins uncocking with `lag > lag_stop`, the damper pulls the bat *backward* with up to **`−71.45 N·m`** at `−118.5 ms` (Probe 2). Replacing the stop with a proper one-sided `20 N·m` hold (Probe B) shifts nominal contact by `+4.1 ms` (`30.82 m/s`, `lag_c = 62.7°`) and sends `arms committed +60 ms` to **`38.9 m/s` at `u = 0.809 m` (past the end of the bat)**.
3. **Inaccurate cap reporting in §4.3.** §4.3 line 194 says the friction cap ran at `230–260 N·m`; in Probe 2 it reaches **`296 N·m`** at peak legs torque (`−117.5 ms`), and `legs` clips `0/540` steps in the base swing.

**Fix.** In phase 1's 3D body, enforce a single coupled friction cone per foot on the floating pelvis, test whether open-loop swings stay balanced under real lateral centrifugal loads, and ensure every passive joint stop is strictly one-sided (never pulling back on rebound).

---

### P2-2. Pitcher release at `3.3°/ms` cannot meet MLB command (`0.57°`), and `balance.py` tested single-leg lift with two feet down

**Where.** `docs/body-motion.md` §5.3 (lines 271–282), §8.1 (line 499), §11 (lines 580–591); `analysis/research/body/balance.py:36–47, 74–83`.

**What is wrong.**
1. **Release angular rate vs MLB command.** §11 line 585 estimates the hand's angular rate at release as `40 m/s / 0.7 m ≈ 3.27°/ms` (`57.1 mrad/ms`). At `16.49 m` to home plate (`18.44 − 1.95 m`), **`1 ms` of finger timing error (Hore & Watts's best precision) scatters pitches by `94.3 cm` — more than two strike-zone widths (`43.2 cm`)**, and a `7 ms` window scatters by `6.97 m` (`diag_probes3.py`). Meanwhile, `docs/calibration-targets.md` §4.2 puts grade-50 MLB `Command` at **`σ_cmd = 10 mrad` (`0.57°`, `16.5 cm` per axis)** — which at `3.27°/ms` demands **`0.175 ms` timing SD**, 6× tighter than human finger precision and 6× finer than §8.1's `1 ms` RK4 step (linear interpolation over a `1 ms` step spanning `3.3°` cannot even resolve release direction cleanly). Real pitchers achieve command by straightening the hand path near release (coupled elbow extension + wrist flexion + trunk flexion, enlarging radius of curvature well past `0.7 m`), which §11 does not design.
2. **MER torque vs peak IR velocity conflation.** §5.1 line 229 and §11 line 580 say the shoulder delivers *"90–120 N·m at about 115 rad/s"*. In ASMI data ([N3] §1, §3), `92–101 N·m` IR torque occurs near maximal external rotation (when IR velocity is near zero, reversing eccentric-to-concentric), whereas `117 rad/s` peaks later during arm acceleration. Elastic storage is still needed, but conflating the two instants misstates the elastic-element design requirement.
3. **`balance.py` gave the 1.5 s pitcher a two-foot stance (`±30 cm`) during leg lift.** `balance.py:38` sets `cop_b, cop_f = (0.30, 0.30)` for `t < T2 = 1.22 s`, even though its own comment calls this `"pitcher: one foot, leg lift"`. Under single-foot CoP limits `(0.05, 0.12)` (Probe 7), `0.5°` still holds (`0.97 cm`), but a `3.0°` forward lean **falls at `1.44 s` (`33 cm`)**, and in the unmodelled frontal plane (foot width `±0.04 m`), a `1.5°` lean drifts `6.4 cm` and `2.5°` drifts `15.4 cm`. `balance.py` also omits proprioceptive sensing noise, uses true `mgh` in the reflex gain, and its `"mass"` error looks tiny (`0.05 cm`) only because the plan sits motionless at `θ = 0, τ = 0` for the first `1.07 s` (contrary to its stale docstring claiming a quintic inverse-dynamics lean).

**Fix.** Add hand-path straightening, sub-step ball-on-finger roll-off mechanics, and grip-to-spin-axis design to §11 before phase 5; re-run `balance.py` with single-foot sagittal and frontal CoP bounds, nonzero motion from `t = 0`, and proprioceptive noise.

---

### P2-3. §6.1's back-solve targets double-count perception, location, depth, and multi-season drift

**Where.** `docs/body-motion.md` §6.1 (lines 358–367); `docs/research/body/6-movement-variability.md` §3, §5.

**What is wrong.** §6.1 plans to back-solve average motor execution noise (`σ_onset`, `σ_strength`) from four targets: `0.27 m` (`~7 ms`) contact-depth SD, `1.9 m/s` bat-speed SD, `3–6 cm` release-point SD, and `7 ms` release window. Three of the four are *total* game/multi-year spreads, not pure motor execution noise:
- `0.27 m` contact depth (`[S10]` §2) includes pitch-location spread (inside vs outside), pitch-type surprises, and **perception error** (`docs/agent-models.md` §3.6 states perception should dominate execution). Sizing pure onset σ to hit `7 ms` before adding perception and location will double-count timing spread.
- `1.9 m/s` (`4.3 mph`, `[S10]` §2) likewise mixes depth/location and protective swings; `docs/agent-models.md` §3.6 already measured the depth- and location-controlled competitive SD as **`3.2 mph` (`1.43 m/s`)**. Probe 6 shows that in `chain.py`, `5%` common strength noise gives only `0.46 m/s` bat-speed SD, and even an unphysiological `15%` strength CV gives only `1.28 m/s` while creating `10.1 ms` of timing SD — whereas `10 ms` independent onset jitter already produces `7.8 ms` timing SD and `1.47 m/s` bat-speed SD.
- Wakamiya 2024's `3–6 cm` release-point SD spans **three seasons** (2021–2023, including fatigue, rubber shifts, mechanics changes, and `1 cm` measurement error), not within-outing execution noise; and Jegede's `7 ms` is a throw tolerance window, not an input timing SD (P2-2).

**Fix.** Specify in §6.1 that phase 3/5 back-solving runs **with perception and location active in the cage** (matching total game spreads to `[S10]`/`Wakamiya`), or targets controlled execution-only subsets (`1.43 m/s` depth-controlled bat speed; `calibration-targets.md §4.2`'s `10 mrad` fresh command) rather than raw pooled totals.

---

### P2-4. §7's ratings probe tested three groups instead of §7.4's two, froze timings across builds, and misdiagnoses the `2×` coil spread

**Where.** `analysis/research/body/ratings.py:37, 43–44, 143–175`; `docs/body-motion.md` §7.1–§7.4 (lines 416–491), §14 (lines 648–652).

**What is wrong.**
1. **Mismatch between probe and recommendation.** §7.4 puts trunk (`core`) inside `UpperStrength`, yet `ratings.py:155–156` held `core = 1.0` while varying `legs` (`1.3 / 0.75`) and `shoulder/wrist` (`0.75 / 1.3`) — testing three groups, not two. Probe 6 re-ran the actual two-group split (`Lower` = legs/rear/lead, `Upper` = core/shoulder/wrist) and confirms two groups still separate hitters cleanly (`pelvis 714` vs `424°/s`, `trunk 815` vs `590°/s`, late-foot loss `−1.15` vs `−1.70 m/s` at `A_c` and `−2.87` vs `−4.27 m/s` at each hitter's own ball point `P`).
2. **Timings were not re-learned for different builds.** `ratings.py:144`'s docstring claims `"launch re-learned"`, but line 149 hardcodes `w = 0.0` and keeps all 17 timings frozen to `fit_params.json`. Consequently the weak-legs hitter finishes at an unrealistic `pelvis = 40.7°` at contact (vs `[4]`'s `77°`), and the counterintuitive result that the *arms* hitter loses twice as much speed on a late foot (`−1.9` vs `−1.1 m/s`) happens because the frozen `-231 ms` strong shoulder fires 40 ms before the late foot lands at `-191 ms`, spinning his unbraced pelvis *backward* to `−4.9°` while the strong-legs hitter (`1,647 N·m`) slams the rising cap immediately after landing (Probe A).
3. **Rigid `±45°` whole-body coil in Section A.** `ratings.py:43–44` rotates pelvis, trunk, arms, and bat rigidly by `±45°` (pelvis starting `+24°` open at `0.70×` strength and `−66°` at `1.30×`) to hold `T_first = 239 ms` constant. Needing twice `[6]`'s coil spread (`±15°/SD` vs `±7°/SD`) reflects missing 3D levers (hand load, elbow/arm radius, stride length, contact depth, and re-learned timings), not proof that a correlated `LoadDepth` rating should carry MLB's swing-length relation.
4. **Minor scaling slip.** `ratings.py:37` has an identical-branch ternary scaling linear leg forces (`rear`, `lead`) by `M`, whereas under `CSA ∝ M/H`, torque scales as `M` while linear force scales as `M/H`.

**Fix.** Re-run §7.2 with §7.4's actual two-group split and re-learned timings per archetype; do not commit to a strength-correlated `LoadDepth` rating until phase 1 measures how much of MLB's swing-length slope comes from 3D hand load, stride, and elbows.

---

### P2-5. §8.2's speed budget and 12-DOF core omit the hardest 3D costs, drop legs and rear arm, and ignore roster-learning time and blend failure

**Where.** `docs/body-motion.md` §5.5 (lines 326–337), §8.2 (lines 513–542), §13 (lines 610–616); `analysis/research/body/bench_chain.rs`.

**What is wrong.**
1. **Optimistic cost extrapolation.** `bench_chain.rs` hardcodes `cap = 280.0`, omits `loads()`/`TAU_LOAD`, `SEP_STOP`, `rear`/`lead` friction caps, `rise` ramps, and `t_off`, and calls `std::f64::sin_cos`/`exp` rather than deterministic `bb_math::m` (`libm`). §8.2's 35-DOF (`10–20 µs/eval`) and 12-DOF (`2 µs/eval`) estimates cite Featherstone's open-tree ABA (~450 flops/DOF) while omitting: (a) the **6-DOF closed loop** of two hands welded to the bat (loop Jacobian, $J_c M^{-1} J_c^T$ solves, and constraint stabilization at every RK4 stage); (b) **unilateral two-foot ground contacts with 3D friction cones** (LCP/iterative solver, or stiff compliant springs forcing smaller `dt`); (c) `libm` transcendental calls per joint; and (d) the fact that pitcher breakdowns entering at stride/foot-plant (flying open, collapsed front leg, §11) prevent tabulating the delivery up to arm acceleration — requiring 0.5–0.7 s of integration per pitch, not 0.2 s. Real 35-DOF cost could easily be 2–5× higher than §8.2's `6 s/game`.
2. **The 12-DOF core drops the legs and rear arm.** Counting §8.2 lines 534–539 yields only **11 DOFs** (pelvis yaw+tilt 2, trunk 3, lead shoulder 2 + elbow 1, wrist cock 1, bat roll 1, forward travel 1) and eliminates **both legs** (hips, knees, ankles, feet), the **rear arm** (closed loop, trail elbow, two-hand couple), pelvis lateral/vertical motion, and the pitcher's arm/fingers. Without knees and feet, front-leg blocking, knee collapse, real weight transfer, and coupled friction cones all revert to hand-crafted planar proxies.
3. **Unbudgeted roster-time learning and non-smooth grid blends.** §5.5 proposes optimizing a `5 × 5` aim grid × pitch-speed classes (~75 optimizations per batter) when the roster is made. Games in `sim-cli games --n 300` generate rosters from seeds on the fly; 75 nonlinear optimizations × 18 players per game would dominate runtime unless precomputed/parameterized or cached per league. Moreover, linearly blending timing parameters between two distinct local optima fails: in `diag_probes3.py`, averaging the timings of the base fit (`tc = +1.0 ms`, `pel/trk 64/63°`) and clock-constrained fit (`+3.8 ms`, `63/67°`) shifts contact to **`+10.1 ms`** and trunk to **`92°`**.

**Fix.** Benchmark a minimal 3D closed-loop + contact prototype early in phase 1 (including `libm`) before finalizing the §8.2 budget; clarify whether the 12-DOF core keeps a leg/knee DOF or explicitly accept what it gives up; require smooth continuation across the aim grid and specify how roster-time learning is amortized/cached and kept deterministic across `sim-cli games`.

---

### P2-6. §14's owner questions quote buggy numbers, offer an unphysical stand-in "for now", and omit two owner decisions

**Where.** `docs/body-motion.md` §14 (lines 618–652).

**What is wrong.**
1. **Late-foot question** quotes `22 m/s` (option b) against `25 m/s` (option a) from buggy `breakdowns.py`; the real figures are `29.7 m/s` vs `25.2 m/s` (P1-1).
2. **Computing-budget question** lists option (a) ("keep 200 ms with the flat body") only to say it is not offered, and omits what the 12-DOF core loses (legs/knees/rear arm, P2-5) and roster-learning cost.
3. **Load-depth question** recommends `(a)` — correlating coil with strength at twice real hitters' spread "until phase 1 finds the other part" — offering an unrealistic stand-in "for now" when phase 1 has not yet checked 3D hand load, stride, and elbows.
4. **Two missing owner decisions:** (a) **Reopening PLAN D9** (re-integrating 100-byte drive programs vs storing recorded core poses in `PlayTimeline`, P1-3 — reopening any PLAN §1 decision requires the owner); (b) **Phase 1's numeric pass/stop gate** (PLAN §14.1 says work stops if phase 1 fails to reproduce the measured swing and breakdowns, yet §13/§14 define no numeric gate tolerances and leave unclear whether phase 1 validates the 35-DOF body or the 12-DOF core).

**Fix.** Correct the late-foot numbers, remove the non-option (a) in Q1 while stating what the 12-DOF core gives up, recommend deferring the coil correlation until phase 1 measures 3D swing length, and add the D9 replay choice (or adopt recorded core poses) and phase-1 pass/stop criteria.

---

### P3-1. `inverse.py` double-counts pelvis acceleration and omits arm inertia; Fortenbaugh Appendix C already settles the `GRFy` sign

**Where.** `analysis/research/body/inverse.py:70, 80–83, 96, 108–109, 117`; `docs/body-motion.md` §5.2 (lines 250–253), §12 (line 597).

**What is wrong.**
1. `inverse.py:70` adds `ypa[i]` into `ha[i][1]`, line 80 copies `ha[i][1]` into the bat's acceleration `ab`, and **line 96 adds `ypa[i]` a second time**. `vh[1]` (line 82) omits `ypd[i]`, lines 108–109 omit the arms' own rotational inertia `I3 · a3[i]`, and line 117 puts the rear foot at `-0.80 m` while `fit.py:39` puts it at `-0.77 m`. Numerical impact is small (Probe 1: legs plateau `526.6 -> 525.9 -> 543.6 N·m`; impulse `35.34 -> 35.38 -> 36.04 N·m·s`; max `|wrist|` `178.2 -> 180.6 N·m`; `ypd` at contact is `-0.007 m/s`), and the ground impulse split (`18.93 − 6.02 = 12.91 N·m·s`) reproduces §5.2's `19 − 6 = 13`.
2. **The `GRFy` sign is not open.** Fortenbaugh Appendix C (`ForcePlateInfo.sky`, p. 203 of `sim/target/research-dl/4/s.txt`) explicitly negates `GRFy` (`-1 * tfGRF(1)` and `-1 * lfGRF(1)`) for right-handed batters (`LHeelMarkerX > RHeelMarkerX`) while leaving left-handers unnegated. Since `+Y = Z × X` points away from the plate for a RHB and toward it for a LHB, the plotted `+GRFy` points **toward home plate** for both hands — confirming `inverse.py`'s `sgn = +1`.

**Fix.** Remove the duplicate `ypa` addition on line 96, add `I3 · a3` and `ypd`, align rear-foot offset with `fit.py`, and remove the "unresolved `GRFy` sign" from §5.2 and §12 (citing Fortenbaugh Appendix C p. 203).

---

### P3-2. Source misattributions, wrong section citations, dropped reading-status caveats, and untagged `[E]` numbers

**Where.** `docs/body-motion.md` §3–§5, §8–§9, §11; `analysis/research/body/common.py:31`.

**What is wrong.**
1. **de Leva length scaling misattributed.** `docs/body-motion.md` line 101 and `common.py:31` state *"Lengths scale with stature, which is de Leva's own assumption."* Direct check of de Leva 1996 (p. 1229: *"l is the segment length, which can be measured from a filmed or videotaped performance"*) and Note 4 (`4-segment-parameters.md:27, 68`) shows de Leva gives no stature-scaling rule; scaling lengths by `H / 1.741` is an `[E]` assumption. (All 10 male segment rows in `common.py` — using alternative `VERT→CERV` head and `CERV→XYPH` UPT rows — were checked digit-for-digit against de Leva 1996 Table 4 and match exactly.)
2. **Wrong section citations in §4.3.** Line 196 cites `[N5] §3` for golfers' trunk rotation strength (it is `[N5] §2`); line 199 cites `[N5] §5` for Delp 1996 wrist strength (it is `[N5] §3`).
3. **Reading-status caveats dropped.** §4.1 cites Koike 2003's `750 N` without noting `FIG` (±10% read from plot, n = 5, tee); §5.1 cites Maffiuletti 2016 (`50–75 ms`) without noting `AB` only (and it describes the early RFD phase, not time to peak torque); §6.1 cites Jegede 2005 (`7 ms`) and Hore & Watts 2011 (`1 ms`) without noting `AB` only; §11 cites `0.15 s` ballistic arm time from `[N3]`'s `2nd`-only `0.145 s`; §5.3 cites Loram & Lakie 2002 (`0.9 mgh`) and §5.1 cites Thelen 2003 (`15 / 50 ms`) without entries in `[N1]–[N7]` or reading status.
4. **Untagged `[E]` constants.** `fit.py`'s misfit weights (`80 / 100 / 250°/s`, `3 ms`, `4°` peak separation SD — `[4]` Table 1's `±4°` is at contact, not peak — `0.03 m`, `0.77 m` vs `0.80 m` rear foot); `chain.py`'s `0.054·H` thigh offset, eccentric slope `4.0`, `m_tr` excluding shanks/feet, stop gains (`2e3 / 20`, `2e4 / 40`); `breakdowns.py`'s `30 ms` heel-plant trigger delay and `20 ms` casting rise; `balance.py`'s `I = 1.08 m h²`, CoP bounds (`0.12 / 0.05 / 0.30 m`), damping (`0.025 / 0.105 mgh`), push (`0.08 mgh`, `0.15 s`); §5.5's `5 × 5` grid; §8.1's `1 ms / 2–4 ms` steps.

**Fix.** Correct the de Leva attribution and `[N5]` section numbers, carry reading-status caveats for `AB`/`FIG`/`2nd` numbers into §4–§6 and §11, and tag every remaining model constant `[E]`.

---

## Answers to the ten questions (beyond the findings above)

1. **Owner's rule (D31) and invariants:**
   - *Roster-time search (§5.5):* **Honest device** (models practice; strictly causal per pitch, zero per-pitch fitting), provided optimization is deterministic (invariant 5) and smooth across the grid.
   - *Blend between learned swings (§5.5):* **Honest device** (forward-only, misses naturally between grid points), though vulnerable to basin-jumping without continuation (P2-5).
   - *Decision clock (§5.4):* **Real mechanism** in design, broken against the probe's `-231 ms` arm onset (P1-2); also unresolved how off-speed recognition can hold hips (§5.4 line 308) when hips begin at `-245 ms` before commit.
   - *Hips waiting for heel plant (§5.4):* **Real mechanism** (proprioceptive trigger), needs legal invariant 1/3 plumbing (P1-3) and corrected numbers (P1-1).
   - *Wrist cock stop (§4.2–§4.3):* **Artificial device** (257 N·m + sticky damper, P2-1); spec rightly plans to replace it with sourced `20 N·m` wrist strength in 3D.
   - *Friction caps (§4.2):* **Real mechanism in principle, incomplete device in probe** (decoupled caps, 170 ms kinematic ramp, uncapped 737 N lateral constraint, P2-1).
   - *Drive switch-off (`t_off`):* **Real mechanism** (agonist relaxation), corrupted in `breakdowns.py` by `shifted()` (P1-1).
   - *Back-solved error sizes (§6.1):* **Legitimate Tier B noise-scale calibration** under PLAN §13, not the old per-pitch fault — once perception/location spreads are separated from execution (P2-3).
   - *Invariants 1–3, 5:* See P1-3 (D9, proprioceptive noise, moving eyes/mid-swing loop) and P2-5 (deterministic roster search, `libm`).
2. **Planar 5-DOF evidence for 3D:**
   - *Likely to survive 3D:* open-loop instability of long balance phases (1.5 s); passive/inertial distal wrist whip; constant-length within-batter effort scaling; two strength groups distinguishing hitter styles better than one.
   - *Unlikely to survive or only argued, not shown:* "the swing needs no feedback" (untested against 737 N lateral pull, vertical balance, and closed-loop grip); lunging, pulling off, casting, and collapsed front leg; `2×` coil spread; 3D runtime cost and 12-DOF core adequacy; pitcher release/command.
3. **Reproduction, dynamics vs branches, integrator checks:**
   - *Reproduction:* All six fast probes matched worker outputs bit for bit (`breakdowns.json` 0 diff; `bench_chain` `1.70 µs/step` vs `1.69`); `fit.py 400` did **not** reproduce `fit_params.json` due to warm-starting (P1-2).
   - *Dynamics vs branches:* `chain.py` has no breakdown-specific outcome branches, **but** `breakdowns.py` introduced artificial effects via the `shifted()` `t_off` bug (P1-1), injected an unsourced `30 ms` wait instead of a load-crossing event inside `Sim`, injected a synthetic casting wrist drive from a dead fit parameter, and manually extended `rear` `t_off` for lunging.
   - *Energy and momentum checks (`checks.py`, Probe 4):* Real checks of EOM-vs-KE consistency — dropping `h` (`32.5%` / `22%` / `17.1 N·s`), flipping `h` (`21.8%` / `22.3%` / `62.9 N·s`), and dropping Newton's-3rd-law reaction `-tau["shoulder"]` (`101%` / `0.76%`) are all caught. **Blind spots:** flipping any force sign inside `forces()` (e.g. `tau["lead"]` → `9.56e-7`, better than baseline!) is completely missed because `power()` reads the same `tau` dict; vertical angular momentum is unchecked (and cannot be conserved with `x_base = 0`).
4. **Fit & physical sense:** Covered in P1-2 and P3-1.
5. **Consolidation (§7):** `LowerStrength`, `UpperStrength`, and `Coordination` (with `Height`, `Mass`) is the right core rating set; forcing `LoadDepth` as a strength-correlated rating before phase 1 checks 3D hand load, stride, and elbows is premature (P2-4).
6. **Errors (§6):** Entry points and back-solve concept are sound under PLAN §13; the four target values double-count perception, location, depth, and multi-season drift (P2-3).
7. **Sources:** de Leva Table 4 verified exact in `common.py`; Fortenbaugh Appendix C resolves `GRFy`; remaining citation/attribution/[E]-tag issues in P3-1 and P3-2.
8. **Cost & plan (§8, §13):** Cost and 12-DOF core issues in P2-5. Phase sizes: phase 1 (4–6 Opus sessions) is tight; phase 2 (2–3) is optimistic if 12-DOF core derivation is needed; phase 5 (5–7) is optimistic given zero lumped-model precedent for release/spin. **Most likely phase-1 failure point:** matching `[4]`'s 3D curves (especially bat elevation `+44° → −30°`, two-hand loop, pelvis/lag at contact) under physiological strength limits and the decision-clock constraint (`on_arms ≥ −170 ms`, `rise ≥ 50 ms`) — `docs/body-motion.md` does not yet specify numeric pass/stop gates or a fallback if phase 1 stalls.
9. **The pitcher (§11):** Release sensitivity (`3.3°/ms` vs `0.57°` MLB command) and single-leg 1.5 s balance are the primary blockers (P2-2); pitch-type grip/finger-roll-off mechanics (how FF/SL/CU/CH spin axes and velocities emerge) and pre-arm-acceleration errors vs tabulation (P2-5) are also undesigned.
10. **Questions for the owner (§14):** Covered in P2-6 (Q2 and Q4 are posed fairly; Q3 has buggy numbers; Q1 has a straw-man non-option and omits 12-DOF trade-offs; Q5 recommends an unphysical coil stand-in "for now"; D9 replay and phase-1 pass/stop gates are missing).

---

## Verification

All commands ran from `/home/natex/Projects/3D Baseball/.claude/worktrees/rev-body-neon` with `BB_FORT_CSV="/home/natex/Projects/3D Baseball/.claude/worktrees/swing-research/sim/target/research-dl/4/csv"`. Outputs saved under `sim/target/review-probes/gemini-neon/`.

- **Git scope:** `git log --oneline c8f5e04..d0def11` and `git diff c8f5e04 d0def11 --stat` — 18 commits, 23 files, 2,779 insertions.
- **Fast probes rerun & diffed against `swing-research/sim/target/codex-probes/body/*.out`:**
  - `python3 analysis/research/body/inverse.py`, `swing.py`, `checks.py`, `balance.py`, `breakdowns.py`, `ratings.py` — all six matched bit for bit (except `swing.py` wall time `0.13 s` vs `0.14 s`); `git diff analysis/research/body/breakdowns.json` was empty.
  - `rustc -C opt-level=3 -C linker=$HOME/.local/bin/zigcc -o sim/target/review-probes/gemini-neon/bench_chain analysis/research/body/bench_chain.rs` + `taskset -c 0-7 nice .../bench_chain` — `1.71 / 1.70 / 1.69 µs` per RK4 step at `0.5 / 1 / 2 ms` (worker: `1.70 / 1.69 / 1.68 µs`), check values identical (`2.777, 2.784, 2.783`).
- **`fit.py 400` rerun via job queue (`bin/job.sh`, job 307, 143 s):** warm-started from `fit_params.json`, descended `0.5203 -> 0.5150`, changed all 17 parameters (`T_wr 59.95 -> 69.48`, `T_rear 165.8 -> 207.4`, `T_lead 960.5 -> 1124.9`, `on_lead −95 -> −113 ms`, `off_core +35 -> +28 ms`). Reverted cleanly via `git checkout -- analysis/research/body/`.
- **Reviewer diagnostic probes:**
  - `python3 sim/target/review-probes/gemini-neon/diag_probes.py` — tested `inverse.py` double-`ypa` / `I3` / `ypd`, `chain.py` implicit `Fx` (`736.5 N`, `1.16 × μW`) and wrist-stop sticky damping (`−71.45 N·m`), `breakdowns.py` with `t_off` shifted, `checks.py` under 4 mutations, `fit.py` 17-parameter sensitivity (4 exact-zero parameters), `ratings.py` actual 2-group split and 120-trial Monte Carlo noise back-solve, and `balance.py` under single-foot sagittal/frontal CoP limits and sensing noise.
  - `python3 sim/target/review-probes/gemini-neon/diag_probes2.py` — diagnosed the weak-legs late-foot paradox (`pel_c = −4.9°`), 20 N·m one-sided wrist hold on breakdowns, and physiological-bounds fit failure (cost `4,275`, no contact).
  - `python3 sim/target/review-probes/gemini-neon/diag_probes3.py` — clock-only constrained fit (`on_sh >= −0.17`, `rise_sh >= 0.05`: cost `2.063`, `T_sh = 613 N·m`, `ss = 35.5 m/s`, inverted pelvis/trunk peaks), physio + doubled `WMAX` fit (`1,789`, no contact), midpoint timing blend (`+10.1 ms`, trunk `92°`), and pitcher release angular sensitivity (`3.27°/ms`, `94.3 cm/ms`).
  - Inline check on `inverse.py` ground-impulse decomposition (`18.93 − 6.02 = 12.91 N·m·s`) and `fit.py` cold-start `X0` cost (`13.9104`).
- **Source verification:**
  - Downloaded `https://robotics.unibs.it/Antropometria/deLeva-ParametriDinamiciAntropometria.pdf`, extracted text (`pdftotext`) and rendered Table 4 (`pdftoppm` page 6), verified all 10 segments of `common.py` and Table 4 values in `docs/research/body/4-segment-parameters.md` directly against the paper, and confirmed de Leva gives no stature-scaling rule (p. 1229).
  - Inspected `sim/target/research-dl/4/s.txt` (§3.3–§3.4, pp. 48–54, and Appendix C `ForcePlateInfo.sky`, pp. 202–204) directly, settling the `GRFy` sign.
  - Ran `python3 bin/check-docs.py` (no dead links inside `docs/body-motion.md` or `docs/research/body/*`; only expected missing review-file path in untracked `REVIEW-BODY-MOTION.md`).
- **Not checked:** did not open full texts of paywalled/external papers behind notes 1–6 beyond de Leva 1996, Fortenbaugh 2011 (`s.txt`), and web-search bibliographic checks on Anderson 2007, Bae et al. 2012 (PMC3546185), Wakamiya 2024, and Nakashima 2025; did not re-run `fortenbaugh_appendix_e.py` image extraction from `s.pdf` (used existing digitised CSVs); did not run `cargo test --workspace` (no Rust sim code changed in `c8f5e04..d0def11`).

## Resolution

Resolved 2026-11-25 by worker body-design (Opus) on `work/body-motion`, commits `8e25dda`, `3b3a598`, `17a3955`, `fc6e82c`, `f92b4c6`; checked by the Coordinator.

- **P1-1 (`shifted()`; cap read after the run): fixed.** As in the Carbon review's P1-1; the cap is now read at the legs' peak, and §6.2 says the flat body does not produce lunging's power loss and what it needs in 3D.
- **P1-2 (the fit breaks the clock and the strength bounds): confirmed and adopted as the main open finding.** `fit.py --rules design` (four starts) and `limits.py` reproduce the reviewer's result: under the arms' clock, a 50 ms rise floor and sourced bounds the flat chain does not reach the ball; the clock is the rule that binds. §4.3 reports isometric and applied strengths, the corrected comparisons (1.5 × the golfers' mean under weight × height scaling, 24% Hill drop) and the argument for the leg channel's bound. The fit cold-starts. Phase 1 must show a 3D body swings under the rules or stop (§13).
- **P1-3 (D9; invariants for sensing noise, eyes, mid-swing): fixed in the spec, one item deferred.** §9 records poses; §5.4 and §10 give the tick loop and where each draw is made. The moving eye point's effect on perception is a named phase 3 item.
- **P2-1 (lateral ground force, decoupled caps, sticky stop): stop fixed; the rest deferred to phase 1.** The stop is one-sided. A single friction cone per foot and the lateral load belong to the 3D contact (§4.1, §5.3, §12); the spec no longer claims the swing stays inside friction or needs no feedback in 3D.
- **P2-2 (release against command; one-foot balance): fixed at design depth.** §11 states the command requirement and that the two shoulder numbers are not simultaneous; `balance.py` now uses a one-foot base for the leg lift (a 2° error grows to 14 cm, 3° falls), reported in §1 and §5.3.
- **P2-3 (back-solve targets double-count): fixed** (§6.1's table of what each target includes and how to use it).
- **P2-4 (ratings probe): fixed.** The two-group split as recommended; the "re-learned" label corrected and the limit stated (timings not re-learned per build); the coil labelled a construction; the force-scaling slip noted with both scalings run.
- **P2-5 (cost, the core, library cost, blending): fixed** (§8.2; §5.5 requires continuation between grid programs and a smoothness test).
- **P2-6 (owner questions): fixed.** §14 now has two questions (the practice search; one timing rating or two), the late-foot item answered from the owner's walkthrough with corrected numbers, the budget deferred until measured, the coil question withdrawn. D9 is complied with; the phase 1 gate is numeric in §13.
- **P3-1, P3-2: fixed** (`inverse.py`; `f92b4c6`; attribution of length scaling to [E]; reading-status caveats; [E] tags).

Not re-reviewed: this is a design document and probes, nothing here runs in games, and PLAN §14.1's phase 1 gate (now numeric in the spec's §13) is the next check. The Coordinator reran `breakdowns.py`, `checks.py`, `ratings.py` and `swing.py` on the fixed branch (identical to the worker's outputs), the docs check and the oracle tests (76 pass, 8 expected failures).
