# Foul share of contact: diagnosis, 2026-11-11

Target B1.15: about 50% of contact goes foul (envelope 40–60%). The sim gives 21%. The owner asked for this to be fixed in the contact model ahead of the chase rate ([chase diagnosis](2026-11-11-chase-diagnosis.md)). **It is not fixed.** None of the noise settings swept here gets past about 30%, which points at the swing or contact geometry, though the sweeps do not prove a ceiling. No calibration values changed.

Runs: `cargo run --release -p bb_game --example foul_diag -- 3000 [timing σ ms] [bat-control σ cm] [K_perception]` (average RHP vs average RHB, seed 1, otherwise `Knobs::calibrated_m3()`), and `sim-cli cage-map --type FF` with a wider timing axis.

## 1. Where the sim's contact goes now

| | Share of contact |
|---|---|
| Fair | 78.8% |
| Foul, opposite-field side | 14.7% |
| Foul, backward | 6.1% |
| Foul, pull side | 0.4% |

Among swings that made contact, the batter's aim was off in time by −13 to +6 ms (5th to 95th percentile; execution adds σ 6 ms) and in height by −5.0 to +6.4 cm. The contact band is ±7 cm, so height error is near the limit of what still makes contact, while timing error is small. Pulled fouls barely exist.

## 2. More timing error mostly makes whiffs

| Timing σ (ms) | K_perception | Contact% | Foul share | Pull | Opposite | Backward | K% | LA SD (°) |
|---|---|---|---|---|---|---|---|---|
| 6 (now) | 2.25 | 75.2 | 21.2 | 0.4 | 14.7 | 6.1 | 14.1 | 31.5 |
| 12 | 2.25 | 73.1 | 24.5 | 1.7 | 16.3 | 6.5 | 16.5 | 31.9 |
| 18 | 2.25 | 68.2 | 27.2 | 2.2 | 17.9 | 7.1 | 21.1 | 32.1 |
| 24 | 2.25 | 62.4 | 30.2 | 3.1 | 18.5 | 8.6 | 26.6 | 32.1 |
| 6 | 1.5 | 89.2 | 20.1 | 0.0 | 15.8 | 4.2 | 4.4 | 28.3 |
| 22 | 1.5 | 77.5 | 28.2 | 2.3 | 19.4 | 6.5 | 10.8 | 28.7 |
| 22 | 1.2 | 81.6 | 29.5 | 2.4 | 21.0 | 6.1 | 7.5 | 26.6 |
| 28 | 1.2 | 73.3 | 29.6 | 2.8 | 20.4 | 6.4 | 11.3 | 27.3 |

Quadrupling timing noise, with or without less perception noise, moves the foul share from 21% to about 30% and no further. Bat-control σ (0.9 vs 1.2 cm) changes nothing, because the batter's height error is mostly perceptual.

## 3. Why: the contact map has a narrow foul window

From the cage map (four-seam, middle-middle, perfect perception, only the listed error applied):

- On time, the ball is fair for vertical offsets from −5 to +5 cm.
- Timing from about −23 to +18 ms is fair. The foul windows are roughly −25 to −33 ms (pull) and +20 to +30 ms (opposite). Beyond ±35 ms the bat no longer reaches the ball's path and the swing misses.
- So a timing error is fair for 41 ms, foul for about 10 ms on each side, then a whiff. Timing noise wide enough to fill the foul windows spills mostly into whiffs.
- Undercutting shifts the ball to the opposite field (+30° of spray at −5 cm), which is where the sim's fouls come from. Overcutting shifts it to the pull side by less.
- A ball clipped 5 cm under still leaves forward (56° launch, 83 mph): the ball grips the bat, so glancing contact is a pop-up, not a foul back. Backward fouls need 6–7 cm of offset, the last centimetre before a miss.

## 3a. Corrections from the Codex fielding review (#6)

- The timing window above is the slice at zero height error. The contact boundary depends strongly on height: 5 cm under and 35 ms early still makes fair contact, 5 cm over and 35 ms late makes contact at +119° of spray.
- The collision model already slides at steep angles: stick at −5 cm, slide at −7 cm (backward, spray 157°). What needs checking is where that transition falls, not whether sliding exists.
- The bat is already tilted about 20° below horizontal at contact in the on-time case.
- Early and late contact at the edge of the window is on the bat's rounded end cap. Where on the bat contact happens is a cheap thing to measure next.

## 4. What this means

Something in the collision model or the swing kinematics is likely missing. Candidates, none verified and none shown to be the most likely (read 1 and 2 with the corrections above):

1. **Glancing contact.** The impulse model keeps the ball gripping the bat at steep contact angles. If real glancing contact slides, more of it goes backward. This is Tier A physics (physics-spec §8.1, research items R2–R3: the sim's spin at a 2 cm undercut already looked high). It needs published oblique-collision data, not tuning.
2. **Bat angle at contact.** A real bat points down from the hands (about 30° at contact for a low pitch). With a tilted bat, a vertical miss throws the ball sideways as well as up or down, which makes fouls out of vertical error on well-timed swings. The swing template's handling of this should be checked against the map's weak overcut-to-pull shift.
3. **Time through the zone.** A real swing moves the hands forward through contact, which keeps the barrel on the ball's path longer than a rotation about a fixed point. That would widen the ±30 ms contact window and turn early and late swings into fouls where the sim has whiffs.
4. **The target's composition.** The MLB split of fouls by direction (pull, opposite, back) would say which of 1–3 matters. It is not in calibration-targets and was not looked up here.

## 5. Proposed

Find the MLB foul-direction split and a published oblique bat-ball collision data set (research), then check candidates 2 and 3 in the swing model against the contact map. The chase-rate change (Discipline grade-50 values) stays parked behind this, as the chase report explains.
