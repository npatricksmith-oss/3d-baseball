# Foul correction review — 2026-11-13

Reviewed `fb4af8d..e3c6022ad62d84e83916d2160b4c0731ea3e286e` after reading `CLAUDE.md` and both resolved foul reviews. The default behavior-preservation claim passes: all four game digests match the parent, and all 148 release tests pass. The counterfactual table reproduces. No P1 regression found; the findings concern the new measurement's reference frame, interpretation of the experiment, and parameter/spec precision. Earlier resolved findings are not reopened.

## P2 — Foul rate conditional on AD is not invariant under correction

**Location:** `docs/calibration/2026-11-13-foul-measures.md:231`–`:232`, also `:197`, `:235`; repeated in `TASKS.md:41`.

The experiment supports “these settings do not close the foul deficit,” but not the categorical conclusion that correction is not a foul lever or that foul rate within an AD bin is independent of which swings reach it. AD alone does not specify the contact map's inputs: height, pitch state, barrel station and swing geometry also matter. Correction changes those inputs and their distribution within each bin.

The assertion that wider phase settings “do not change the foul rate in any bin” is numerically false. For default / `phase=0.2` / `phase=0.3`, foul/contact at AD below −20° is **78.6 / 58.8 / 59.1%**, and at AD 20–30° it is **29.7 / 23.3 / 25.9%**. Tail samples are small, so these are not precise estimates of a population effect, but neither do they establish invariance. Even the highlighted AD 12–20° rate moves **8.1 → 11.7%** with `adj=100`, while remaining far below MLB's reference.

The decision confound **is explicitly acknowledged at line 230**; this is not a finding that the acknowledgement is missing. Nevertheless, the independent PA runs cannot identify what happened to the same swings. A scratch paired experiment held the **4,357 baseline competitive pitches**, count, scouting, previous read, previous perceived speed and RNG keys fixed, forced the original swing decision, and disabled check cancellation in both arms. Default replay matched the recorded executed swings and batted results exactly. With `adj=100 phase=1 cut=0`, **925 baseline whiffs became contact: 623 fair and 302 foul (32.6% foul)**. Additionally, **250 fair contacts became foul**, **190 fouls became fair**, and **185 original contacts became whiffs**. At AD 20–30°, paired foul/contact was **27/91 = 29.7% → 6/41 = 14.6%**. These are a controlled execution counterfactual, not new PA/K% estimates; conditioning on resulting AD still changes bin membership.

**Suggested fix:** Retain the reproducible, narrower conclusion: the tested settings improve contact substantially while leaving the principal foul deficits, especially |AD| <6° and AD 12–20°. Say recovered contact is *mostly* fair, report transitions and bin denominators, and remove the claim of conditional invariance. To distinguish population effects from a deficient contact response, compare fixed pitch/aim/execution sweeps or standardize the joint inputs, rather than conditioning on AD alone. Carry the qualified conclusion into TASKS.

## P2 — “Estimate error” uses truth at the aim's depth, not the estimate's depth

**Location:** `sim/crates/bb_game/examples/foul_diag.rs:192`; inputs at `sim/crates/bb_game/src/pa.rs:534`–`:536`; interpretation at `docs/calibration/2026-11-13-foul-measures.md:204`–`:213`.

`known_gap` has the advertised raw sign: `(want.x − cur.x, want.z − cur.z, t_want − t_aim)`. However, `aim_error.y` uses the true pitch at **cur.y**, whereas `want` was predicted at another depth. Adding the two gives `want.z − true_z(cur.y)`, not the estimate's error `want.z − true_z(want.y)`. The omitted correction is `true_z(cur.y) − true_z(want.y)` when converting the printed value to an estimate error at its own depth. The same reference issue would apply to a reconstructed timing estimate error.

The depth difference comes from bounded x correction and the subsequent `cur.y = depth(nx)`; it is not recorded in `known_gap`. Moreover, `aim_of` evaluates depth using the component's plate-plane mean x, while its returned path x can differ, so even reaching `want.x` need not make the two depths identical.

A scratch instrumented replay of the default 3,000 PAs gives:

| Population | Printed estimate error | Estimate error at its own depth |
|---|---:|---:|
| Early and over >6 cm, n=1,107 | +9.325 cm | **+10.041 cm** |
| Close, n=1,647 | +1.343 cm | +1.388 cm |
| Bat under >4 cm, n=1,099 | −3.557 cm | −3.577 cm |
| AD 12–30° whiffs, n=504 | +8.729 cm | **+9.323 cm** |

Across competitive swings with both truth crossings, the absolute discrepancy has median **0.091 cm**, p95 **1.562 cm**, maximum **6.165 cm**. PA index **1096**, pitch index **0**, has aim depth **0.457699 m** and estimate depth **0.158537 m**: the printed estimate error is **−7.603 cm**, versus **−1.438 cm** at its own depth. Thus this can be material per swing, despite modest group-mean changes. The raw residuals still support the observation that correction limits bind; this does not invalidate their signs or counts.

**Suggested fix:** Record the final estimate and its depth, and compare it directly with truth there, reporting missing crossings. Alternatively label the sum explicitly as “estimate height minus truth at the final aim depth,” and expose the depth-transport term before calling the split a perception-versus-correction accounting. Apply the same convention to timing. This finding concerns the newly introduced estimate-error reconstruction, not the previously resolved nominal/intercept height accounting.

## P2 — The exact velocity identity does not establish the claimed face/spray identity

**Location:** `docs/calibration/2026-11-13-foul-measures.md:236`; resulting next-step rationale in `TASKS.md:49`.

For v1, the exact identity is

```text
v_ss = angular_speed × (r_knob + u_ss) × (n × axis).
```

Consequently the sweet spot moves perpendicular to the **3D bat axis**, in the swing plane. If “face normal” means the in-plane vector `n × axis`, alignment is indeed exact. But this is not the horizontal face normal used for face angle in agent-models §5.1, nor is it necessarily the collision surface normal. The cylindrical bat has no single surface normal independent of where the ball meets it. In particular, `v_xy · axis_xy = −v_z × axis_z`, which need not vanish for a tilted barrel and rising path.

In the default **4,357 competitive swings**, the absolute angle between horizontal sweet-spot velocity and the horizontal normal to the bat axis has median **3.50°**, p95 **9.10°**, maximum **19.51°**. Across **2,815 competitive contacts**, the acute 3D angle between sweet-spot velocity and the actual `ContactPoint.n` has median **22.95°**, p95 **65.22°**. The collision also includes incoming ball velocity, off-axis contact, friction and spin. An in-plane kinematic identity therefore does not imply that outgoing spray equals AD, or that adding a hand path/lag must broaden spray and increase early/late fouls.

The note correctly calls this an unmeasured candidate, and a cage counterfactual is a reasonable next experiment. The phrases “exactly along” and “would spread ... and send more” nevertheless give the proposed mechanism more certainty than the established geometry supports.

**Suggested fix:** Specify the in-plane perpendicular identity and distinguish it from horizontal face angle and collision normal. State that hand motion/lag *could* change spray relative to AD; measure the baseline spray-minus-AD distribution and compare controlled variants before predicting its spread or foul effect. Keep TASKS framed as testing this hypothesis rather than relying on it as an established explanation.

## P3 — Two other §3.6 adjustment controls remain unnamed literals

**Location:** `sim/crates/bb_game/src/pa.rs:503`, `:520`; named specification controls at `docs/agent-models.md:315`.

The cutoff and phase-rate extraction is complete, including `OwnExecution`, but the same loop still embeds the check-swing threshold **−0.03 runs**, specified as `−θ_check` [E], and the abort boundary as **0.6 of elapsed start-to-contact time**. The spec names an abort **phase** `s_abort = 0.45` [E]. The latter is not automatically the same as 0.45 of elapsed time under the Gaussian speed profile, so blindly replacing 0.6 with 0.45 would be an unjustified behavior change. `BRAKE_S = 0.040` is already named in `swing.rs`.

These literals predate this range; they are included because the requested review explicitly asks about other bare [E] controls in this loop. They do not contradict the verified default digest equality. They do leave the parameter cleanup incomplete under CLAUDE.md's requirement that [E] parameters be named and configurable, and leave the abort mapping undocumented.

**Suggested fix:** Expose the check threshold and the implemented abort control with unchanged defaults, and document how the time-based v1 cutoff corresponds to the spec's phase rule, or explicitly document the approximation. Treat any subsequent phase-rule correction as a separate behavior change with validation.

## Verification

Only this review file was added. Probe sources and logs were written under `/tmp`; all builds, executables and Zig caches used `sim/target`. No source, test, spec or TASKS file was edited. The parent source was extracted with `git archive fb4af8d sim` into `/tmp/foul-correction-parent`, with an isolated build directory at `sim/target/correction-review-parent`; no scratch Git worktree was registered.

- Read the complete commit range, agent-models §3.6, the new note §8, TASKS, both resolved reviews, and the relevant PA, swing, motor, contact, decision and reporting paths. No new outcome rolls, `bb_agents` dependency on `bb_world`, direct transcendental calls outside `bb_math::m`, or event-sourcing mutation were found. `known_gap` remains diagnostic only.
- `cd sim && export PATH=$HOME/.cargo/bin:$PATH && cargo test --workspace --release`: **148 passed, 0 failed**. Zig global/local caches were set beneath `sim/target`.
- Ran `cargo run --release -p bb_game --example digest` for both HEAD and the archived parent. All four hashes match exactly:

| Seed | Parent and HEAD digest |
|---|---|
| 1 | `ecd817095e948b5cf8089a513347ac8a058842e56e6050063eca85cdc75a84d6` |
| 2 | `b7905bb51d18619040d69db5b5c137f59380e73ce79a3522e569fd9a7d704e97` |
| 3 | `6d0f956104e421175ff2a071289067dfaf4edc9d3196ec7d38c3495946958764` |
| 42 | `f1a9f0d32c6a9581f68b1bc62be3c1a5daba8d20ac34e895dd582daabb9d9fd7` |

- Inspected every replacement: `adj_cutoff_s` reaches the decision's correction duration and the moving loop cutoff; `adj_phase_frac` reaches expected timing credit and the executed timing bound. Defaults and arithmetic order are preserved. The new gap computation does not mutate simulation inputs or consume RNG. Digest equality establishes the claimed sample equivalence, not a proof for every possible seed/configuration.
- Ran `cargo run --release -p bb_game --example foul_diag -- 3000`, then the built release example with each flag combination below. All §8 counterfactual rows reproduce at the printed precision:

| Flags | Foul/contact | Near-AD foul | AD 12–20° foul / whiff | Early/high share | K% | Contact% | Z- / O-Contact% |
|---|---:|---:|---:|---:|---:|---:|---:|
| Default | 23.7 | 20.4 | 8.1 / 42.7 | 25.4 | 19.8 | 64.7 | 71.8 / 40.2 |
| `adj=1.2` | 23.6 | 20.6 | 9.3 / 39.8 | 23.2 | 18.1 | 66.9 | 74.1 / 43.2 |
| `adj=2.4` | 23.0 | 19.6 | 10.8 / 35.3 | 20.9 | 16.0 | 69.2 | 76.2 / 46.3 |
| `adj=100` | 22.5 | 19.2 | 11.7 / 26.8 | 14.3 | 12.0 | 73.8 | 78.6 / 58.7 |
| `phase=0.2` | 24.5 | 20.6 | 9.4 / 45.9 | 24.6 | 19.8 | 65.2 | 72.6 / 39.7 |
| `phase=0.3` | 25.2 | 20.7 | 8.1 / 52.0 | 22.6 | 19.9 | 65.4 | 72.8 / 40.3 |
| `cut=30` | 23.6 | 21.0 | 8.3 / 40.2 | 23.8 | 18.8 | 66.0 | 73.0 / 41.8 |
| `adj=1.2 phase=0.2` | 24.8 | 20.7 | 10.3 / 44.6 | 22.3 | 18.0 | 67.6 | 74.7 / 44.1 |
| `adj=100 phase=1 cut=0` | 26.1 | 20.5 | 10.6 / 28.4 | 8.3 | 8.4 | 81.1 | 82.8 / 76.0 |

- The baseline saturation table also reproduces: group counts **1,107 / 1,647 / 1,099 / 504**, signed z gaps **−16.3 / −0.3 / +1.7 / −13.8 cm**, timing gaps **+22.3 / +1.6 / +1.0 / +16.7 ms**, and the reported threshold shares. These measure residual requests at the last completed update, not necessarily saturation on every earlier tick.
- Compiled `/tmp/foul_correction_probe.rs` with an instrumented scratch copy of `pa.rs`. Across **9,158 pitch records**, its executed swings, aim errors, known gaps, releases, batted tuples and calls matched the workspace library. It computed the depth-reference and face/normal measurements above. Every baseline competitive swing has `known_gap`; the one missing nominal height crossing remains in the existing measures and is excluded only from nominal-error populations, as previously fixed.
- **625 checked swings** can retain a gap from before their braking observation, because braking exits before the update assignment. The new saturation cohort excludes all check/braked swings, so that staleness does not bias this table. When no update executes, initialization leaves the field `None`; it is not carried between pitches. Non-default cohorts should continue reporting missing values rather than treating them as zero correction demand.
- Compiled `/tmp/foul_correction_forced.rs` for the paired experiment described in the first finding. The forced default arm exactly matched all **4,357 baseline competitive executed swings and batted results** before comparison. This intentionally fixes decisions and PA contexts; it does not claim to simulate the counterfactual PA distribution.
- TASKS correctly records the new diagnostics, tested values, unchanged defaults and pending review in its new “First” entry. Its new causal summary and face/path rationale need the qualifications above. The older unconditional “Nothing is waiting on a review” sentence at line 14 also needs reconciliation with that pending-review entry. Historical merge wording is not reopened as a duplicate of the earlier review.

## Resolution

Open.
