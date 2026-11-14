# The bat's angle at contact (λ_c) and the opposite-field offset (2026-11-14)

Branch `work/swing-lambda`, from `work/swing-recalib` at `6bd5047`. Diagnostic only: **no game change is adopted**, and the default (λ_c = 0) is untouched. Question (owner's ruling, through the lead): with the bat head turned past the hands' line at contact (λ_c > 0, the face toward the pull side of its path), what angle makes on-time contact show Statcast's spray − attack direction (AD) of +7.2° (R/R) to +8.9° (all)? v2 gives −5 to −11° on time ([recalibration report §2.4](2026-11-14-swing-recalibration.md)).

**Answer.** No believable angle does it, and no angle at all. λ_c turns the face against the path only through the hands' velocity along the bat, and the hands are slow (14 mph against the sweet spot's 71.5). The face therefore leads the path by about 0.2° per degree of λ_c, and could never lead it by more than about 11° even at 90°. In the PA model, λ_c = 15° moves spray − AD by +2.8°, mostly by moving the measured AD rather than where the ball goes. Nearly all of the offset comes from the collision: an undercut ball off a barrel sloped 28° down leaves toward the tip's side, and its sidespin carries it further the same way. A swing parameter cannot fix it.

Directions are pull-positive, degrees; spray is by first landing (Statcast's hit coordinates stand in the same way), with the launch direction in brackets where useful. Tools: `examples/lag_cage_probe.rs` (cage) and `foul_diag lag=<deg>` (PA model). Outputs: `sim/out/lag/` (untracked).

## 1. The cage: λ_c swept, on time

Perfect perception, an average RHP four-seamer at middle height against an average RHB, met on time. λ_c was already a template parameter (`SwingTemplate::lag_c`, agent-models §3.6, [E] 0); the contact pose handles λ_c ≠ 0 (ρ solved with the hands off the axis line, speed scale k from the full sweet-spot velocity), so bat speed at contact stays at `BatSpeed`.

**Met 2 cm under the ball's center (the game's aim; launch about 19°):**

| λ_c | | outside | mid-away | middle | mid-in | inside |
|---|---|---|---|---|---|---|
| 0° | spray (launch) | −31.2 (−18.0) | −28.4 (−15.8) | −25.7 (−13.7) | −23.0 (−11.7) | −20.3 (−9.6) |
| | AD; face − AD | −4.5; −0.3 | −2.9; −0.3 | −1.3; −0.2 | +0.2; −0.2 | +1.8; −0.2 |
| | spray − AD | −26.6 | −25.5 | −24.4 | −23.2 | −22.1 |
| 4° | spray − AD | −25.6 | −24.6 | −23.5 | −22.5 | −21.5 |
| 8° | spray − AD | −24.5 | −23.6 | −22.7 | −21.7 | −20.8 |
| 10° | spray − AD (face − AD) | −24.0 (+2.0) | −23.1 (+1.8) | −22.2 (+1.5) | −21.4 (+1.3) | −20.5 (+1.0) |
| 15° | spray − AD (face − AD) | −22.6 (+3.2) | −21.8 (+2.8) | −21.1 (+2.5) | −20.4 (+2.1) | −19.7 (+1.6) |

Across the sweep (0 → 15°, all five locations): exit speed 101.6–100.6 mph, unchanged within 0.1; launch angle 18.8–19.2° → 18.2–18.8°; bat speed at contact 71.5–71.7 mph, unchanged; swing length +0.24 to +0.32 ft at 15° (middle 6.97 → 7.24 ft; +0.14 at 8°, +0.18 at 10°).

**Met at the ball's center (launch about 3°):** spray − AD at λ_c = 0 is −5.8 / −4.6 / −3.5 / −2.3 / −1.1 outside to inside; at 15° it is −2.7 / −1.7 / −0.8 / 0.0 / +0.9. Every λ_c in between moves it in proportion (full table in `sim/out/lag/cage.txt`).

- **The effect is about 0.2° of spray − AD per degree of λ_c.** It matches the kinematic bound: the face leads the path by atan(qρ·sin λ / (u_ss + qρ·cos λ)) with qρ ≈ 0.18 m and u_ss ≈ 0.71 m.
- **Reaching +8° would need a face about 30° ahead of the path** at the game's aim (about 11° at the ball's center). The planar double pendulum cannot produce that at any λ_c.
- **Most of the change is in AD, not in the ball.** At 15° the middle AD moves −1.3 → −4.0 while spray moves −25.7 → −25.1. A larger λ_c makes the sim's AD by location worse against Statcast's (its AD is already 2–10° too far toward the pull side in games, from the batters' timing).
- **Timing constants.** Swing length grows with λ_c, about 0.02 ft per degree. Up to about 5° it stays within the fit (7.2–7.4 ft against Statcast's 7.2–7.3). At 10–15° the widths w_b and τ_b would need refitting to bring the length back. Bat speed needs nothing, since k is solved from the full velocity.

## 2. The PA model (`foul_diag 3000 lag=…`, average R/R, perception and noise on)

| | λ_c 0 | λ_c 8° | λ_c 15° | MLB |
|---|---|---|---|---|
| All fair: spray; spray − AD | −2.5; −5.5 | −2.2; −4.0 | −2.2; −2.7 | +6.1 to +8.1; +7.2 (R/R) to +8.9 |
| On time (±5 ms), spray − AD, outside → inside | −11.4 / −11.3 / −8.8 / −9.2 / −5.2 | −9.8 / −9.9 / −7.1 / −7.8 / −4.3 | −8.4 / −8.5 / −5.9 / −6.8 / −3.5 | +7 to +9 |
| All fair spray, outside → inside | −5.7 / −5.7 / −1.3 / −0.5 / +1.3 | −5.8 / −5.2 / −1.0 / −0.2 / +1.6 | −6.0 / −5.2 / −1.0 / 0.0 / +1.5 | −1.3 / +4.5 / +6.9 / +8.4 / +14.3 |
| On-time spray, outside → inside | −15.5 / −13.6 / −9.8 / −8.5 / −3.2 | −15.7 / −13.9 / −9.4 / −8.3 / −3.2 | −15.9 / −13.9 / −9.5 / −8.4 / −3.3 | |
| Swing length, middle, ft | 7.38 | 7.51 | 7.65 | 7.23 |
| K%; foul share | 20.5; 27.3% | 20.3; 27.0% | 20.2; 27.0% | |

**Where balls go does not move** (all fair −2.5 → −2.2; on-time spray by location unchanged within 0.5°): λ_c only moves the measured AD. **Spray by batted-ball type** shows where the offset lives:

| | GB < 10° | LD 10–25° | FB 25–50° | PU 50°+ |
|---|---|---|---|---|
| Sim, all fair (λ_c 0) | +8.5 | −8.2 | −19.8 | −27.7 |
| Sim, on time | +3.9 | −19.9 | −34.9 | |
| MLB (Savant 2025, by Statcast type) | +15.3 | +6.6 | −2.8 | −21.7 |

Grounders are pulled in both (less in the sim). The sim's air balls go far more to the opposite field than MLB's. Spray falls by 28° from grounders to fly balls, against MLB's 18°, and by 39° on time. The offset grows with launch angle, which is what an undercut-on-a-sloped-bat cause predicts.

## 3. Where the offset at λ_c = 0 comes from

One contact (the cage's, on time) re-resolved with the collision code (`resolve_contact`), changing one thing at a time:
- **(a) the pitch's incoming lateral angle** removed;
- **(b) the pitch's spin** removed;
- **(c) the bat leveled:** its pose, velocity and spin rotated about the ball's center so the axis is horizontal, with its horizontal direction kept and the contact geometry intact. The rotation also turns the path by 2–4°, so spray − AD is the fair measure;
- **(d) as (c), with the contact normal given the face's azimuth.** This changed nothing: once the bat is level, the round barrel's up/down offset no longer tips the normal sideways.

Middle, met 2 cm under (launch 19°):

| | spray (launch) | AD | spray − AD | normal's direction |
|---|---|---|---|---|
| actual | −25.7 (−13.7) | −1.3 | −24.4 | −8.5 |
| (a) no lateral incoming | −24.0 (−12.8) | | −22.7 | |
| (b) no pitch spin | −24.7 (−12.4) | | −23.4 | |
| (c) level bat | +0.6 (+1.7) | +2.7 | −2.1 | +2.4 |

The other locations give the same split: the level bat removes 18–26° of spray − AD (18 outside, 22 middle, 26 inside), (a) 1.1–2.2°, (b) 0.8–1.4°. Met at the ball's center the offset is small (−3.5 at the middle) and splits about evenly: lateral 1.3, pitch spin 1.6, slope 1.6.

So, at the game's aim:
1. **The bat's downward slope with undercut contact: about 22° of the 24°.** It acts in two parts:
   - **Launch direction, about 12°.** The contact normal points up and toward the tip, so 2 cm under the center its azimuth is 7° opposite of the face; the ball also takes the tangential push along the sloped barrel.
   - **Sidespin carry, about 12° more.** Launch −13.7° becomes −25.7° at landing.

   This is a **collision cause**: the geometry is real, and the swing's slope (vertical bat angle −28° at the middle, Blast −25 to −35°, vendor) is in range. The collision's spin is about twice the published values at these launch angles (recalibration report §2.2), so the sidespin share is probably overstated too.
2. **The pitch's incoming angle: about 1.5–2°.** A RHP's four-seamer moves toward the first-base side; the ball keeps part of that.
3. **The pitch's spin: about 1°.**
4. **What is left with a level bat: about 2°.** The bat's own rotation during the collision and the contact's place along the barrel.

**What the swing contributes: nothing here.** The face is ⊥ to the path (λ_c = 0), and on time the face and the path agree to 0.3°.

**A measurement question as well.** The sim reads AD at the instant of contact, and the sweet spot's path turns about 1.4–1.5° per ms there. The same swing read 1.7 / 3.3 / 5.0 ms before contact gives AD 2.6 / 5.0 / 7.5° further to the opposite field (middle: −1.4 → −4.0 / −6.4 / −8.9). If Statcast's AD is effectively a direction over the last few frames before the intercept (300 fps is 3.3 ms a frame), it reads more opposite-field than the instant of contact, and MLB's spray − AD would look 3–7° more pulled than an instantaneous measure. Statcast's glossary says only that AD is the sweet spot's direction "at the point of contact" ([MLB glossary](https://www.mlb.com/glossary/statcast/attack-direction)). No method was found, so this is a hypothesis, not a correction.

## 4. Judgment

- **λ_c is not the lever.** A few degrees moves spray − AD by about 0.2° per degree and moves the ball not at all; reaching +8° would take a face 30° ahead of its path, which this kinematics cannot make at any λ_c. A large λ_c would also lengthen the swing past Statcast's and push the sim's AD further from Statcast's by location. **Recommendation: leave λ_c at 0.**
- **The cause is the collision's, plus possibly the measure.** About 90% of the on-time offset comes from undercut contact on a sloped barrel: its launch direction and its sidespin. The sim's spray falls with launch angle about 1.6 times as fast as MLB's (2.2 times on time).
  - The same Tier A collision that gives about twice the published backspin probably gives too much sidespin as well.
  - The tangential behavior (e_t 0.40, μ) decides how much of the barrel's slope the ball takes away.

  Both belong to the sourcing question the owner put under D (physics-spec §8; test A10's magnitudes still pending). Spray by launch angle (§2 table) is a check for any collision change.
- **Next sourcing steps (proposed):**
  1. A published spin-and-direction against undercut-and-bat-slope data set for test A10. Nathan, Cross and others measured batted-ball spin against undercut. Bat tilt's sidespin is described in the physics literature, but no table was pulled here.
  2. Statcast spray by launch angle and by pitch height, from the same 2025 sample: MLB's slope of spray against launch angle is the number a collision change has to meet.
  3. How Statcast measures AD (a frame or an instant).
- **The location trend** (§2: on time −15.5 → −3.2 against MLB −1.3 → +14.3, a 12° range against 15.6°) is close in range and offset in level: the same collision offset plus the batters' timing (recalibration report §2.4).

## 5. Published values for the bat's angle at contact

**None found.** A search for the bat's angle to the lead forearm, or for how far the barrel is ahead of or behind the hands at contact, turned up no measured values. Blast Motion defines vertical bat angle (barrel to ground) and "early connection" (lead arm to torso, 80–105°), not the bat against the hand path ([Blast: vertical bat angle](https://blastmotion.com/blog/what-is-vertical-bat-angle-in-a-baseball-swing/)). Coaching sources say the hands lead the barrel into contact and the bat head stays below the hands; none give a number. Statcast's 2025 swing metrics (attack angle, attack direction, swing path tilt) do not include it ([MLB, 2025](https://www.mlb.com/news/new-statcast-swing-metrics-2025)). λ_c stays an unsourced [E] value at 0; this note found no reason to move it.

## 6. Reproduction

`work/swing-lambda`: `cargo run --release -p bb_game --example lag_cage_probe`; `cargo run --release -p bb_game --example foul_diag -- 3000 lag=0` (and `lag=8`, `lag=15`). Default digests (λ_c = 0, the `digest` example, seeds 1, 2, 3, 42) are identical to `work/swing-recalib` (`51f69f5d…`, `20a7a0b6…`, `29a62d2f…`, `b37bbb5f…`): this branch changes no library code, only the two examples.
