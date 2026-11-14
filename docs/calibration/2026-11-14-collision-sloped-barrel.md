# The collision off a sloped barrel: sources and diagnosis (2026-11-14, Stage 1)

Branch `work/collision` (from `work/swing-recalib` at `f46ea5a`). **Stage 2 (owner ruled Option A, 2026-11-14): the speed-dependent e_t is built; §8.** Owner's ruling D20 (PLAN §1): the bat–ball collision off a sloped barrel is reopened as a sourced physics question, spec and oracle first, with no tuning toward a run total. **Stage 1 only: no physics code is changed here.** The one counterfactual run in games used a temporary edit, reverted (the tree is clean).

**The question** (recalibration report §3.1; bat-angle note §3; [collision targets](2026-11-14-collision-targets.md), Sonnet, `work/collision-targets` `a7a6fe9`). Off a barrel sloped about 30° down and met under the ball's center, the sim's ball goes too far toward the tip side and spins too much. Fly balls land at −19.8° (pull-positive) against MLB's −2.6°, and sidespin is 3,720 rpm at a 27° launch.

**Answer, in short.** Two separate things are wrong, and they have separate causes.

1. **Spin magnitude is a parameter question.** The code reproduces the Kensrud–Nathan–Smith (KNS) model and its measurement at tee speeds to within 2%. At game speeds the same tangential restitution (e_t 0.40) gives about 1.8 times Statcast's backspin at fly-ball launch angles. Nathan's own collision model shows the same excess against the same Statcast data. With e_t = 0 the sim matches both published Statcast spin measures (backspin at 25–30°, and sidespin per degree of spray), but then it no longer matches KNS at the tee. No constant e_t fits both.
2. **Direction (spray against launch angle) is not a parameter question.**
   - In a rigid-body collision with a cylinder, the ball leaves the bat (in the bat's frame) in the plane perpendicular to the bat's axis. On a barrel sloped 31°, lift above the swing's path comes with a sideways push toward the tip of about tan 31° per unit of lift. e_t and μ do not change that ratio.
   - Matching MLB's slope would need a bat sloped only about 8–15°. Statcast's own tilt and attack angle put it at about 30°.
   - MLB's slope also does not steepen for low pitches, where the bat is steepest; the sim's does.
   - Something real is missing from the standard rigid-body picture, or the inputs differ in a way no published source pins down. This part stays open; no sourced spec change for it is proposed.

## 1. What the model does (physics-spec §7–§8, `bb_physics::contact`, `collision`)

- **Normal.** At the first touch (bisection to 1 µs), n̂ is the radial direction from the nearest axis point to the ball's center, tilted by the profile's slope R′(u) along the axis (`contact_at`; zero on the barrel). For a bat sloped down by s, the "up" direction perpendicular to the axis is û = (sin s·ĥ_a, cos s), where ĥ_a is the axis's horizontal direction toward the tip. A ball met D_v under its center (vertically) is offset D = D_v·cos s along û. The normal is then tilted from the face by ψ with sin ψ ≈ D / (R + R_bat), and its azimuth turns toward the tip by atan(tan ψ·sin s).
  - Check (cage, middle, on time, s = 30.8°): 2 cm under predicts 7.1° (measured 7.1° from the path); 3 cm predicts 10.9° (measured 11.5°).
- **Impulse.** One rigid-body impulse with a sticking target (c_n′ = −e_n c_n, **c_t′ = −e_t c_t on the whole tangential vector**) and one Coulomb cone |J_t| ≤ μ J_n. Otherwise it slides along the initial slip direction. The energy guard follows. The tangential direction is isotropic: along and across the barrel are treated alike.
  - In the game contact the along-axis part is small: the relative velocity along the axis is 1.4–3.9 m/s against 3.7–38 m/s across it. The impulse along the axis is 0.11–0.26 N·s against 10.6–12.2 N·s normal.
  - **The ball sticks in every case** (μ 0.50 down to 0.15), so μ does nothing at game contact. It matters only in the limit μ → 0.
- **Bat.** A free rigid body (§6.3) with the full inverse inertia, including the small moment about its long axis. The tangential recoil factor is r_x = (mα/(1+α))(1/M + b²/I_cm + R²/I_ax) = 0.181 at KNS's impact point (Cross & Nathan's bat: 0.159).
- **Spin** (KNS Eq. 8, gripping): ω = (1+e_t)·v_t / (R(1+r_x)(1+α)), with v_t = v_rel·sin ψ the tangential relative speed. Its axis is perpendicular to n̂ and to the tangential impulse, which is close to **parallel to the bat's axis**: 16.6° off it at 3 cm under, 22° at 2 cm.
  - A spin axis along a sloped bat is backspin plus a slicing sidespin, Nathan's mechanism (2020, §IV).
  - By hand, with v_rel ≈ 70 m/s and ψ 24.8° at 3 cm under: ω ≈ 1.4·70·0.42 / (0.0366·1.18·1.4) ≈ 680 rad/s ≈ 6,500 rpm. The cage gives 5,890 rpm (the hand value ignores the pitch's spin and the exact v_rel).
- **Direction.** In the bat's frame the ball keeps a fraction of its tangential velocity, 1 − (1+e_t)α/((1+α)(1+r_x)) (0.66 at e_t 0.40, 0.76 at e_t 0). That tangential velocity points up the barrel's face (along û). The normal rebound is small (q ≈ 0.2), so this retained velocity carries both the launch angle above the path and a sideways part toward the tip, about tan s of the lift. The lab direction adds the bat's velocity, which dilutes the angle but not the ratio.
  - In the cage at 3 cm under, the normal impulse alone would send the ball 26.7° toward the tip of the path. The tangential impulse (1.49 N·s, mostly downward along the face) brings it back to 18.4°. In flight the sidespin curves it on to 33.1° at landing (`collision_anatomy` Part B).

The code is the spec, and the spec is the standard model: KNS 2017 plus the rigid-body impulse of Cross & Nathan 2006. Part A of `collision_anatomy` rebuilds KNS's tee experiment: a level bat at 77 mph at the impact point (b = 6.8 in past the center of mass), a ball at rest, ψ 5–30°. It gives:

| | spin, rpm per ° of θ | spin vs KNS Eq. (8) |
|---|---|---|
| code, e_t 0.40 | **183** | within 1% (689 vs 658 rpm at ψ 5° is the worst) |
| code, e_t 0.464 (KNS wood bat) | 195 | within 1% |
| KNS Fig. 3, measured (all bats, ⟨e_x⟩ 0.405) | **179** | |

## 2. The published physics

- **Kensrud, Nathan & Smith, "Oblique Collisions of Baseballs and Softballs with a Bat"** (Am. J. Phys. 2017; [arXiv:1610.03464](https://arxiv.org/abs/1610.03464)):
  - Setup: a machine swings the bat (pivoted, rubber-padded clamp) into a ball at rest on a tee, 63–88 mph at the impact point, normalized to 77 mph.
  - The scattering plane is perpendicular to the bat's axis (implied by r_x's R²/I_z term); no along-axis tangential component was studied.
  - Values: ⟨e_x⟩ = 0.405 ± 0.010 for baseballs; wood bat 0.464 ± 0.014 (Table II). μ is not measured: lower bound μ_k ≥ 0.15; the model uses 0.5. α = 0.4.
  - Spin against launch angle θ: 179 rpm/° (Fig. 3); exit speed −0.65 mph/° (Fig. 4).
  - Sec. V.2 applies the same e_x at 158 mph and says "testing that assumption will have to await experiments at higher speed". No tilted bat, no sidespin, no comparison with game spin.
- **Cross & Nathan, "Scattering of a Baseball by a Bat"** (Am. J. Phys. 74, 896 (2006); [arXiv:physics/0605040](https://arxiv.org/abs/physics/0605040)):
  - Setup: a ball dropped at about 4 m/s onto a hand-held level bat, offset perpendicular to the axis.
  - Values: e_T (apparent) = 0 ± 0.02; e_x = 0.16 ± 0.02; μ_k ≥ 0.50.
  - The authors warn the values may not carry to game speeds. No tilted bat, no sidespin.
- **Nathan, "Why Does a Fly Ball Carry Better to Centerfield?"** (preprint, June 25, 2020; [PDF](https://baseball.physics.illinois.edu/carry-v2.pdf)). Statcast/Trackman fly balls at Tropicana Field. Trackman measures total spin directly and infers the axis, so backspin and sidespin are derived (footnote 2). Figure values below are my reading of the plots:
  - Fig. 2 (EV about 100 mph, LA 25–30°, backspin 2,000–3,000 selected): sidespin crosses zero near **−10°** of adjusted spray (pulled) for both hands, and reaches about −2,000 rpm at +20° (opposite field): about **−65 rpm per degree** (Sonnet reads −80, ±30%).
  - Fig. 4 (EV 95–105, by 5° launch bucket, all balls): over the middle of the field, backspin is about **2,300 rpm at 25–30°**, about 1,800 pulled to 3,000 opposite. It runs from about −1,500 to +500 rpm at 0–5° to about 4,000 at 40–45°.
  - Fig. 5, **Nathan's own 3D collision model** (unpublished; 2D version in his [ModelingBallBatCollision slides](http://baseball.physics.illinois.edu/ppt/ModelingBallBatCollision.pptx), not read here): about **4,000–5,000 rpm at 25–30°**, which he calls "qualitative agreement" with Fig. 4.
  - §IV, in words: two mechanisms make sidespin, the bat's horizontal angle to the pitch and its downward tilt. "When the bat is tilted downward but still perpendicular to the pitch, the spin axis gets rotated, reducing the backspin and resulting in a slicing sidespin." His detailed models "are in agreement with" the zero near −10°.
- **Kagan, "The Physics of a Foul Down the Line"** (Hardball Times, 2014; [link](https://tht.fangraphs.com/tht-live/the-physics-of-a-foul-down-the-line)): qualitative only. Sidespin comes from the bat's angle to the pitch and from contact above or below the midline on a tilted bat, and is larger for opposite-field balls. No numbers.
- **Nathan et al. 2012, "Spin of a batted baseball"** (Procedia Engineering): **not found** online in this session. The spec's open item 3 (pull spin against undercut for A10) stays open in that form.
- **Not found:**
  - a measured horizontal launch direction against undercut on a tilted bat;
  - a measured e_x at game relative speeds;
  - any treatment of along-axis tangential motion;
  - a published batted-spin distribution beyond Nathan's figures.
- **MLB side** (Sonnet's note, twelve 2025 dates, 9,165 balls in play):
  - Spray falls 17° from grounders to fly balls; spray − AD falls 0.37° per degree of launch angle (+13.9° at 0–10°, −0.6° at 25–35°).
  - The slope is the same at every location, and by height it is −0.30 (bottom third), −0.35 (middle), −0.35 (top): no steeper for low pitches.
  - Attack direction is read in the single video frame nearest the ball. It is an instant, not a window, which is what the sim reports.
- **MLB, attack angle by launch** (my check on the six-date sample, `savant_*` CSVs, 4,513 balls in play with bat tracking): the attack angle hardly changes with launch angle (8.8° at 20–30°, 10.6° at 30–40°, SD about 8°). MLB fly balls get their lift from undercut, as the sim's do.

## 3. Comparison, like with like

Everything below is on-time contact on a middle four-seamer from the average RHP (`collision_anatomy`) unless it says games.

| Check | Published | Code (e_t 0.40) | e_t 0 (counterfactual) |
|---|---|---|---|
| Tee: spin per ° of θ (KNS Fig. 3) | 179 rpm/° | **183** | about 131 (×1/1.4, by Eq. 8) |
| Tee: spin vs KNS Eq. (8) | analytic | within 1% | within 1% (same formula) |
| Backspin at 25–30° launch, middle (Nathan Fig. 4) | about 2,300 rpm | **4,249** (27°) | **2,293** |
| Sidespin per ° of adjusted spray, LA 25–30°, EV 90+ (Fig. 2) | about −65 (−80 ± 30%) | −110 | **−73** |
| Spray where sidespin is zero (Fig. 2) | about −10° (pulled) | −1.9° | −1.8° |
| Nathan's own model, backspin at 25–30° (Fig. 5) | about 4,000–5,000 | 4,249 | |
| Landing spray − AD per ° of launch, cage, on time | (MLB, all contact: −0.37) | **−1.17** | −1.21 |
| … launch direction only (before the curve) | | −0.67 | −0.72 |
| … by height: low / middle / high pitch | MLB −0.30 / −0.35 / −0.35 | −1.35 / −1.17 / −1.01 | |
| Spray − AD at launch 10° | MLB +6.2 (10–25° band), +13.9 (0–10°) | −12.9 | −11.6 |
| Games: spray by type GB / LD / FB (`foul_diag`, R/R) | MLB +14.7 / +5.3 / −2.6 | +8.5 / −8.2 / −19.8 | not run |
| Games: HR/PA, mean EV, runs (`run_env_diag` 100) | 3.0%, 89.5, 4.39 | 0.92%, 89.5, 4.51 | **2.78%, 91.3, 5.85** |

Read across the rows:
- At tee speeds the code is right.
- At game speeds the code with KNS's e_t spins the ball about 1.8 times as fast as Statcast's fly balls, as Nathan's own model does. Dropping e_t to 0 makes both Statcast spin measures fit, and in games it nearly brings home runs to MLB's rate. It also raises mean exit speed 1.8 mph above MLB's: less energy goes into spin.
- No e_t touches the direction rows.

## 4. Diagnosis, by the four candidates

1. **(i) A modelling error in the geometry or the impulse: not found.**
   - The normal and the azimuth of its tilt follow from the cylinder by hand (§1, within 0.6°).
   - The impulse reproduces KNS's model and measurement at the tee.
   - The along-axis treatment is immaterial here: 1–2% of the impulse.
   - The direction result is a property of any rigid-body impulse model with a round barrel. The impulse has almost no component along the axis, so the bat-frame outgoing velocity lies in the plane perpendicular to the axis. Nothing in the code departs from the published model.
2. **(ii) A parameter whose sourced value differs from the game's: yes, for the spin, with a caveat.**
   - e_t 0.40 is KNS's tee value (a resting ball, 77 mph, a pivoted bat). At game relative speeds it gives about 1.8 times Statcast's derived backspin at fly-ball launch angles.
   - The only game-speed evidence is Trackman's: total spin measured, axis inferred.
   - e_t = 0 reproduces Statcast's backspin and its sidespin per degree of spray. But measured e_x rises from Cross & Nathan's 0.16 at 4 m/s to KNS's 0.40 at 34 m/s, so a game value near 0 has **no laboratory support**, and no constant e_t fits both the tee and the game.
   - μ is not involved: every game contact sticks for μ ≥ 0.15.
3. **(iii) A correct collision meeting wrong inputs: possible for the direction, and not by the bat's slope alone.**
   - The spray slope scales with the bat's slope at contact (Part E): matching MLB's −0.37 needs a slope of about 8–15° against the swing's 31°.
   - The swing's 31° is sourced. Statcast's swing-path tilt (32°) and attack angle (9°) give a vertical bat angle of about −30° (sin² = sin²tilt − sin²AA), and Blast measures −25 to −35° (vendor).
   - MLB shows none of the slope's height dependence that a sloped-bat mechanism must have (the sim: −1.35 low, −1.01 high). MLB's spray − AD is also strongly pulled at squared-up contact (+6 to +14° at 0–25° launch), where the sim is −2 to −13° at any slope.
   - So the inputs that would be needed are not the published ones. If the inputs are the cause, it is something the swing model does not represent and no source measures: for example, the face's horizontal angle at contact correlated with undercut, or the hands' velocity along the bat. The bat-angle note ruled out λ_c as the sim builds it.
4. **(iv) Statcast's measurement: small.**
   - Attack direction is read in the single video frame nearest the ball (Sonnet's note, §4), an instant like the sim's, so it carries no systematic lag. Its frame rate and derivative are unpublished.
   - The meaning of `hc_x`/`hc_y` (landing, fielded, end of play) is unpublished. It could bend ground balls toward fielders but not fly balls.
   - It cannot account for 20–30° at fly-ball launch angles.

**The open direction question, as one sentence.** A rigid barrel sloped as Statcast and Blast measure it, met under the ball, must send a lofted ball toward the bat's tip by about 1.2° per degree of launch (on time, landing). MLB's balls show 0.37, with no dependence on pitch height and a pull offset at squared-up contact. No published collision model or measurement found here resolves that, and Nathan's 3D model, which he says reproduces the sidespin's zero at −10°, is unpublished.

## 5. Proposed changes (for the lead and the owner; nothing applied)

### 5.1 New golden cases for test A10 (from the published values, not from Rust output)

| ID | Case | Expected | Source | Code today |
|---|---|---|---|---|
| A10a | Tee: level default bat, 77 mph at b = 6.8 in past the center of mass, ball at rest, offset ψ 5–30° across the axis | Spin within 1% of KNS Eq. (8) with the code's e_t and r_x; fitted spin per ° of θ 179 ± 18 rpm/° at e_t 0.405 | KNS 2017 Eq. (8), Fig. 3 | passes (183 at 0.40) |
| A10b | Game: average RHP four-seamer at the middle, on time, met under the ball to launch 25–30°, exit speed 90+ | Backspin 2,300 ± 600 rpm | Nathan 2020 Fig. 4 (read from the plot; derived spin) | **fails** (4,249) |
| A10c | Same contacts swept over timing ±15 ms and 1.5–4.5 cm under | Sidespin −65 ± 30 rpm per ° of adjusted spray | Nathan 2020 Fig. 2 | **fails** (−110) |
| A10d | Same sweep | Sidespin zero at −10 ± 5° of adjusted spray | Nathan 2020 Fig. 2 | fails (−1.9°); **report only** until the direction question is settled |
| A10e | Games: spray − AD against launch angle, slope over 0–50° | −0.37 ± 0.10° per ° | collision-targets note §2 | **report only** (a game-level check, not a physics golden) |

The oracle (`analysis/oracle/bbphys.py`, `gen_golden.py`) already has an independent rigid-body impulse; A10a's analytic part goes there as a fixture case written from KNS Eq. (8). A10b–A10d are published-value checks on cage sweeps, so they live in Rust tests with the published bands, not in the oracle.

### 5.2 A physics change for the spin (owner rules)

Spec §8.2's e_t row becomes:

> e_t (tangential restitution, e_x in KNS): **0.405 at tee speeds** (KNS 2017, resting ball, 77 mph; [S]). **At game relative speeds no measured value exists.** Statcast's batted backspin and sidespin at fly-ball launch angles (Nathan 2020 Figs. 2, 4) imply a value near 0. Proposed: e_t as a function of the normal impact speed |c_n|, 0.405 at |c_n| ≤ 34 m/s (KNS) falling linearly to 0 at 65 m/s (game sweet-spot contact) [E, anchored on the two physical observations of the same quantity, calibration-targets §2.2; the form between them is not sourced].

- The counterfactual with e_t = 0 throughout (§3, last row) shows the size of what this does: home runs about triple, exit speed rises about 1.8 mph, and runs rise about 1.3. So it cannot go in alone without re-checking A8 (q at the sweet spot is head-on and unchanged, but oblique contact loses less energy) and the run environment.
- Alternatives the owner may prefer:
  - (a) Keep e_t 0.40 and treat Trackman's inferred backspin as the open measurement question. A10b and A10c then stay report-only.
  - (b) A constant game value (e_t ≈ 0) with A10a moved to report-only.
- I recommend the speed-dependent form: it is the only one of the three that keeps the measured tee value and meets the game's spin.

### 5.3 The direction: no spec change proposed

The rigid-body model is the published one, and no source gives a direction measurement to fit. What would close it, in order:
1. Nathan's 3D collision model (unpublished; ask him, or read the 2D slides) for its horizontal launch direction against undercut and tilt.
2. A Statcast query, by Sonnet's script: spray − AD against launch angle split by the swing's tilt or by the batter's mean attack angle. A sloped-bat mechanism predicts a steeper slope for steeper swings.
3. Any lab measurement of scattering off a tilted bat.

Until then A10d and A10e are reported, and the spray offset stays a known gap (it is partly offset in games by the batters' early timing, recalibration report §2.4).

## 7. Two more diagnostics (2026-11-14, after the lead's read; defaults unchanged)

### 7.1 Like with like: the PA model's contact by Sonnet's launch bands

`foul_diag 10000` (all swings of the average R/R matchup, with perception and execution noise; fair contact only). New this section: Sonnet's bands, the launch direction (exit velocity) beside the landing spray, and an `et=` flag (counterfactual; the default stays 0.40). MLB is right-handed batters from the collision-targets note (twelve 2025 dates). Pull-positive degrees.

| Launch band | n | spray, sim (MLB) | spray − AD, sim (MLB) | gap in spray − AD | launch dir − AD | in-flight curve |
|---|---|---|---|---|---|---|
| < 0° | 2,704 | +12.0 (+13.3) | +9.5 (+17.3) | −7.8 | +16.0 | −6.5 |
| 0–10° | 1,096 | +1.1 (+9.2) | −0.5 (+11.3) | **−11.8** | +0.1 | −0.6 |
| 10–25° | 1,610 | −7.7 (+5.3) | −10.2 (+5.1) | **−15.3** | −6.2 | −4.0 |
| 25–35° | 817 | −18.1 (−0.5) | −21.8 (−1.4) | −20.4 | −13.1 | −8.6 |
| 35–50° | 694 | −21.9 (−4.7) | −28.6 (−7.7) | −20.9 | −18.7 | −10.0 |
| 50°+ | 208 | −26.1 (−18.8) | −38.0 (−17.0) | −21.0 | −26.7 | −11.3 |
| same, e_t 0: 0–10° / 10–25° / 25–35° / 35–50° | | +2.1 / −7.1 / −16.3 / −19.9 | +0.5 / −9.3 / −19.9 / −26.5 | −10.8 / −14.4 / −18.5 / −18.8 | | −0.4 / −3.6 / −7.8 / −9.6 |

**Level and slope, separated.**
- **Slope** of spray − AD across the four bands from 0° to 50°, from band means: sim −0.75° per degree of launch angle (−0.72 at e_t 0); MLB −0.51. The regressions over individual balls (0–50°) are sim −0.81, of which the launch direction contributes −0.53 and the in-flight curve −0.28. The excess slope is about −0.24° per degree, so roughly −9° from 0–10° to 35–50°.
- **Level**: the curve sits **11.8° to the opposite field at 0–10° and 15.3° at 10–25°** (spray − AD), and 7.8° for grounders below 0°.
- **The lead's suspicion holds: the level is the larger part** at every band. At fly-ball launch (25–35°, gap −20.4) it is about −12 of level plus about −6 of excess slope, with about −2 left over.
- In raw spray (not minus AD) the level is hidden at the low end by the batters' early timing, which pulls the sim's AD 2–10° (recalibration report §2.4): ground balls match MLB (+12.0 against +13.3), and the gap grows to −13 at 10–25° and −17 at 25–50°.
- **e_t's part is small.** e_t 0 lowers the slope by 0.03–0.05° per degree, and the curve's share by 0.014 (−0.277 → −0.263). The in-flight curve does carry about a third of the slope, but at a given launch angle it barely depends on e_t: less sidespin comes with less backspin and a flatter, shorter flight. e_t also moves the level by about 1°.
- **So e_t is the spin question only.** The spray gap is a level offset of about 12° (§7.2) plus the sloped barrel's geometric slope (§4).

### 7.2 The face ahead of the path, with faster hands (`face_path_probe`)

The hands' speed at contact is set through q (the hands' rate over the bat's at t_c, an unsourced [E] value, swing note Open 7), solved per case so the knob moves at the target speed at t_c. The default (λ_c 0, q 0.32) is 14.4 mph. Bat speed stays at `BatSpeed` by construction (71.5–71.9 mph). Cage, middle four-seamer, on time; "squared up" means the ball's center was the aim (launch 1–3°), "2 cm under" means launch 16–19°. The planned AD is +0.1°.

| λ_c | hands mph | q | swing length ft | actual AD | **face − AD** | AA actual | EV squared / 2 cm | spray − AD, squared: launch / land | spray − AD, 2 cm under: launch / land |
|---|---|---|---|---|---|---|---|---|---|
| 0° | 14 | 0.31 | 7.00 | −1.4 | −0.2 | 6.8 | 103.2 / 101.1 | −2.6 / −3.5 | −12.4 / −24.4 |
| 0° | 24 | 0.64 | 8.19 | −1.2 | −0.2 | 6.9 | 103.4 / 101.4 | −2.5 / −3.2 | −12.3 / −24.0 |
| 15° | 14 | 0.30 | 7.22 | −4.0 | +2.3 | 5.3 | 103.2 / 101.1 | −0.8 / −1.0 | −10.6 / −21.3 |
| 15° | 24 | 0.61 | 8.22 | −5.5 | +4.1 | 4.3 | 103.2 / 101.3 | +0.6 / +1.1 | −9.2 / −18.6 |
| 30° | 14 | 0.26 | 7.39 | −6.4 | +4.6 | 3.8 | 103.1 / 101.0 | +0.9 / +1.3 | −8.9 / −18.3 |
| 30° | 21 | 0.44 | 8.11 | −8.6 | +7.1 | 2.4 | 102.8 / 100.8 | +2.7 / +3.8 | −7.1 / −14.8 |
| 30° | 24 | 0.53 | 8.31 | −9.6 | +8.1 | 1.8 | 102.5 / 100.7 | +3.5 / +4.8 | −6.3 / −13.3 |
| 45° | 14 | 0.21 | 7.55 | −8.5 | +6.7 | 2.5 | 102.9 / 100.8 | +2.4 / +3.2 | −7.5 / −15.7 |
| 45° | 18 | 0.28 | 8.00 | −10.3 | +8.6 | 1.4 | 102.5 / 100.5 | +3.8 / +5.1 | −6.1 / −12.9 |
| 45° | 21 | 0.34 | 8.27 | −11.7 | +10.1 | 0.5 | 102.1 / 100.2 | +4.9 / +6.4 | −5.0 / −10.8 |
| 45° | 24 | 0.41 | 8.49 | −13.1 | **+11.6** | −0.3 | 101.6 / 99.8 | +6.0 / **+7.7** | −4.0 / −8.7 |
| 60° | 14–24 | 0.19–0.36 | 7.76–8.69 | −10.0 to −15.9 | +8.3 to +14.3 | 1.6 to −2.0 | **74–79** / 76–79 | +2.4 to +6.3 (land) | −13.6 to −7.2 (land) |

All twenty combinations are in `sim/out/collision/face_path.txt`.
- **Hand speed alone does nothing** (λ_c 0: face − AD −0.2 at 14 and at 24 mph): the hands must move along the bat to turn its face against its path.
- **Together the two do.** Face − AD reaches +4 to +8 at λ_c 30° and +7 to +12 at λ_c 45° for hands at 14–24 mph. Squared-up spray − AD moves from −3.5 to as much as +7.7, and 2 cm under from −24.4 to −8.7.
- **At λ_c 60° the planned contact falls off the sweet spot** (exit speed 74–79 mph): outside what the contact-relative plan handles without a refit, so not a candidate.

**What closing the level offset would need.** The gap at 0–10° (−11.8) and 10–25° (−15.3) is about the shift the cage shows for **λ_c ≈ 45° with the hands at about 21–24 mph** (+10 to +11 squared up, +14 to +16 at 2 cm under). λ_c 30° with 24 mph gives about two-thirds of it. Sonnet is looking for measured values (hand speed at contact, the bat's angle to its path); none is adopted here. The lead quotes Blast's peak pro hand speed in the low-to-mid 20s mph; I have not read that source, and it is the peak, not the speed at contact. Taking it would require refitting:
1. **Swing length.** It grows from 7.0 to 8.3–8.5 ft (Statcast 7.2–7.3), so the time constants (w_h, w_b, τ_b) need refitting, as for v2's first fit.
2. **Attack angle.** The actual AA falls by 6–7° (6.8 → 0.5 or −0.3), so α₀ needs refitting to keep Statcast's attack angle by height.
3. **Attack direction.** The actual AD moves 10–13° to the opposite field of the plan, so ψ₀ (and ψ_x, by location) needs refitting to keep Statcast's AD. Spray − AD, which is what the table reports, is what survives that refit.
4. **The bat speed normalization** needs nothing: k is solved from the full sweet-spot velocity.
5. **Hand speed itself** becomes a sourced Statcast/Blast-style check, if a source gives it at contact.
6. **Downstream refits.** The fielders' spray beliefs and `BAT_YAW_RATE_DPS` (`live.rs`/`play.rs`, refit mechanically as for v2), and the swing oracle's fixture (it uses λ_c 0).

It would not touch the slope (the sloped barrel's geometry, §4) or the spin (e_t, §5.2).

## 8. Stage 2: tangential restitution against impact speed (owner, 2026-11-14: Option A)

### 8.1 What was built

| Step | Commit | What |
|---|---|---|
| PLAN | `f710d9f` | D20 records the ruling: Option A for the spin; the direction gap stays known and unexplained |
| Spec | `217aebb` | physics-spec §8.2.1: e_t(|c_n|) = 0.405 up to 34 m/s, linear to 0 at 65 m/s, constant beyond. Four named [E] parameters (`e_t_lab`, `v_t_lab`, `e_t_game`, `v_t_game`) with their sources; the ramp's shape is stated as unsourced. Tests A10a–A10d and A14 in §14.2; open items 8 (direction) and 9 (e_x at game speed) |
| Oracle | `972fc3f` | `bbphys.tangential_restitution`, `bat_ball_collision`; fixture `golden/tangential_restitution.json`: the law at 13 speeds and five bat–ball contacts (two tee cases below the ramp, one at 45.8 m/s inside it, two at game speed, 64.3 and 70.5 m/s). The oracle checks the tee cases against KNS Eq. (8) to 1e-9. The existing fixtures regenerate byte-identical |
| Code | `cbb80c8` | `BatBallParams::e_t(cn_abs)`, used by `resolve_contact` at the first impulse's normal speed (a second impulse reuses it, as for e_n). Linear interpolation only: no transcendental math. `with_constant_e_t` for counterfactuals (`foul_diag et=`, `collision_anatomy`) |
| Tests | `cbb80c8` | A14 against the oracle (law to 1e-15, contacts to 1e-9); A10a, the KNS tee: spin about the bat's axis equals Eq. (8) at the lab e_t to 1e-9, and 184 rpm per degree of launch against KNS's measured 179 ± 18; A10b/A10c in the cage (`bb_game/tests/batted_spin.rs`). With the old constant 0.40, A10b and A10c fail (4,205 rpm, −110 rpm/°). Workspace: 338 tests pass |

**Both anchors hold, so no stop was needed.** At KNS's 77 mph the tee's normal speed is 34.4·cos ψ, at most 34.3 m/s, so e_t stays at 0.405 (≥ 0.401 at ψ 5°). The game contact that makes a 27–30° fly ball (3 cm under) meets the bat at 64.3 m/s, so e_t is 0.009 there and 0 for anything squarer.

### 8.2 What it does

**Cage** (`collision_anatomy`, `sim/out/collision/anatomy_law.txt`):

| | e_t 0.40 (before) | e_t(|c_n|) | e_t 0 | Published |
|---|---|---|---|---|
| Tee, rpm per ° of θ | 183 | 184 | (131) | 179 (KNS Fig. 3) |
| Backspin, LA 25–30°, EV 90+ (A10b) | 4,205 | **2,465** | 2,364 | about 2,300 (Nathan Fig. 4) |
| Sidespin per ° of adjusted spray (A10c) | −110 | **−80** | −73 | about −65 (Fig. 2) |
| Sidespin zero (A10d, report only) | −1.9° | −1.0° | −1.8° | about −10° |
| 3 cm under: LA, EV | 27.3°, 97.7 mph | 30.4°, 98.6 mph | | |
| Landing spray − AD per ° of LA, on time | −1.17 | −1.21 | −1.21 | MLB −0.37 (all contact) |

**Games** (standard run, `cbb80c8` against `4254e73`, seeds 0–299 [1000–1299]; ledger §3): runs 4.82 → 5.76 [4.50 → 5.78]; HR/PA 1.04% → 2.36% [0.98% → 2.24%]; K% 15.6 → 16.8 [15.8 → 16.7]; BB% 7.3 → 7.7 [7.6 → 7.7]; LD hits .676 → .763; FB hits .172 → .221; fouls a game 17.8 → 21.1 [17.5 → 21.4].
- **Why the fouls rose.** In the PA model (`foul_diag 10000`, R/R) foul share of contact goes 26.0% → 29.2%, nearly all opposite-side (18.5 → 21.3). An undercut ball keeps more of its tangential velocity up the face and launches about 3° higher, so more contact lands at 40–60°, where the sim's foul rate is highest. Longer counts follow: +10 pitches a game, K% +1.
- **What it does not move: the direction.** Spray − AD per degree of launch changes by 0.03–0.05 (§7.1). The known gap stays: about 12° level offset toward the opposite field plus about 0.24° per degree of excess slope. **It is recorded as known and unexplained** (physics-spec §17 item 8, ledger §5), and nothing was changed for it.

### 8.3 The refit: two sweep rounds, nothing adopted

Targets (calibration-targets; 2024 MLB): runs 4.39, K% 22.6, BB% 8.2, HR/PA 3.0%; exit speed none over 125 mph (§4.4); LA SD about 27° [20–35]. Each row is `run_env_diag 100 0` with the e_t law (`sim/out/et/env_*.txt`). Over 100 games (seeds 0–99) the base runs 0.4 above the 300-game standard run (6.18 against 5.76), so read the rows against the base row.

| Noise scales | runs | K% | BB% | HR/PA | mean EV | max EV | LA mean ± SD | foul/contact |
|---|---|---|---|---|---|---|---|---|
| base: perception 2.7, swing 1.0 | 6.18 | 16.5 | 7.7 | 2.64% | 90.5 | 118.1 | 12.0 ± 26.3 | 26.2 |
| swing 1.4 | 5.46 | 18.2 | 7.8 | 2.68% | 90.0 | 120.3 | 10.7 ± 28.2 | 27.7 |
| swing 1.8 | 5.21 | 20.4 | 8.3 | 2.95% | 89.4 | **128.0** | 9.6 ± 30.0 | 29.9 |
| swing 2.0 | 5.24 | 21.2 | 8.8 | 2.59% | 89.0 | **131.6** | 8.4 ± 30.5 | 31.1 |
| perception 3.5 | 3.62 | 31.1 | 8.9 | 1.66% | 87.8 | 116.2 | 9.8 ± 29.5 | 26.3 |
| swing 1.8, perception 2.9 | 4.65 | 24.1 | 8.4 | 2.31% | 88.8 | **128.0** | 9.0 ± 30.2 | 30.3 |
| swing 2.0, perception 2.9 | 4.39 | 25.4 | 8.9 | 2.56% | 88.2 | **131.6** | 8.0 ± 31.2 | 30.7 |

- **The swing-noise knob can reach runs, K and HR together, but it breaks the exit-speed tail.** `Knobs::swing` scales `BatControl` and timing σ and also the effort σ, so at 1.8 the fastest ball is 128 mph against "none over 125". The recalibration report (§2.3) turned it down for the same reason at 1.5. It is not adopted.
- **Perception alone is clean but costs home runs.** At 3.5 it overshoots K (31%) and runs (3.6). By interpolation about 3.0 gives K about 21–22% and runs near the target in the standard run, but HR/PA falls to about 2.2%, against 3.0%.
- **Why it needs more than a sweep.** The runs surplus is mostly hits on balls in play (GB .34, LD .76, FB .22 against .245, .68, .13: fielding's, `live.rs`, which this job may not touch). The HR deficit is the exit-speed tail and the opposite-field carry of the direction gap. No single noise scale lowers the first and raises the second.

**Refit plan (for the lead; nothing applied):**
1. **Split the swing knob** (a code change to `bb_agents::motor` and `pa.rs`, specs first): scale the vertical miss and timing (`BatControl`, timing σ) separately from the effort σ. That way the miss can widen without lengthening the exit-speed tail. Then sweep the miss scale alone, 1.4–1.8. Expected: K +2–4 points, runs −0.6 to −0.9, max EV about 120.
2. **Perception 2.7 → about 2.9–3.0** to carry K% to about 22. Check chase rate (sim about 24% against 29%) and BB% (target 8.2).
3. **Leave HR/PA where it lands (about 2.3–2.6%)** and record the deficit against the direction gap: fly balls sliced 20° toward the opposite field lose carry (recalibration report §2.2: about 29 ft from the sidespin at 3,720 rpm, now about 2,500). Do not raise bat speed: its mean (32.0 m/s) and spread are sourced.
4. **Then the standard run and a ledger row**, with the calibration report naming each change's band and mechanism (calibration-targets §2.3–§2.4).
5. The runs surplus on balls in play belongs to the fielding work (ledger §5, ground-ball hits).

### 8.4 State at handoff

- **Done and committed on `work/collision`:**
  - the ruling (PLAN D20);
  - the spec (physics-spec §8.2.1, A10a–d, A14, open items 8–9);
  - the oracle and its fixture;
  - the code (`BatBallParams::e_t`);
  - the tests (338 pass);
  - the measured ledger row with causes, and the direction gap in ledger §5;
  - two refit sweep rounds (§8.3).
- **Measured:**
  - Standard run: runs 5.76 [5.78], K% 16.8 [16.7], BB% 7.7 [7.7], HR/PA 2.36% [2.24%], fouls 21.1 [21.4] a game.
  - Cage: backspin 2,465 rpm at 25–30°, sidespin −80 rpm/°, tee 184 rpm/°.
- **Remains:**
  - The refit (plan in §8.3; it needs the swing knob split, a code change, before any sweep can be adopted).
  - A Codex review of `f710d9f..` (standard protocol; not run).
  - The direction gap (known, unexplained).
  - Nothing is merged; `work/collision` is not on main.

## 6. Reproduction

`cargo run --release -p bb_game --example collision_anatomy` (Parts A–F; output in `sim/out/collision/anatomy.txt`, untracked). §7: `foul_diag -- 10000 et=0.40` and `et=0` (`sim/out/collision/foul_et*.txt`); `face_path_probe` (`sim/out/collision/face_path.txt`). The games counterfactual is `bb_physics::bat` `e_t` 0.40 → 0.0, then `run_env_diag -- 100 0`, then reverted (`sim/out/collision/et0_env.txt`, `base_env.txt`). §8: `collision_anatomy` (`sim/out/collision/anatomy_law.txt`); `cargo test -p bb_game --test batted_spin -- --nocapture`; standard runs `sim/out/et/law-seed*.txt`; `foul_diag 10000` and `et=0.40` (`sim/out/et/foul_*.txt`); refit sweeps `BB_K_SWING=… BB_K_PERCEPTION=… run_env_diag 100 0` (`sim/out/et/env_*.txt`). Nathan 2020's figures were read from the PDF rendered at 110 dpi. The six-date Savant CSVs (attack angle by launch) are the swing note's; Sonnet's twelve-date tables are in `work/collision-targets`.
