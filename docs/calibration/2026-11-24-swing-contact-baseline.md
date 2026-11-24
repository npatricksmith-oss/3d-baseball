# Today's swing at contact: the baseline for the rebuild (2026-11-24)

Worker swing-spec, branch `work/swing-rebuild`, on main `aa4a31d` (PLAN D25). Measurement only: no sim source changed, no game changed. The rebuilt swing is specified in [swing-model-v3](../swing-model-v3.md); this note records what the swing that it replaces actually puts into the collision, so the rebuild can reproduce each figure or move it knowingly.

**Reproduce:** `cargo run --release -p bb_game --example swing_contact_baseline -- 300 8` (the standard run's game seeds: `--seed 0` gives 0–299, `--seed 1000` gives 1,000,003,000–1,000,003,299; average fielders, exactly as `sim-cli games`). It plays the games unchanged and re-reads each executed swing (`PitchRecord::swing`) at the recorded contact time; nothing feeds back. Job 254, 191 s. Every batter is mirrored to a right-hander. Angles: bat elevation and azimuth in Fortenbaugh 2011's convention (azimuth 0° = bat pointing across the plate, −90° = at the catcher, −180° = away from the plate; read from his analysis code, Appendix C `BatLag()`); attack direction and spray pull-positive. "t_c" is the executed swing's nominal contact; T_a is when the ball's centre reaches the contact plane y = 0.35 m.

## 1. Counts (600 games)

| | all | set 0 | set 1000 |
| :-- | --: | --: | --: |
| swings started | 77,434 | 38,483 | 38,951 |
| full swings (not checked) | 67,181 | 33,384 | 33,797 |
| full swings with contact | 43,590 | 21,593 | 21,997 |
| of which fair / foul | 30,828 / 12,762 | 15,292 / 6,301 | 15,536 / 6,461 |
| full swings missed | 23,591 | 11,791 | 11,800 |
| checked swings | 10,253 | 5,099 | 5,154 |
| checked: touched the ball (a swing by rule) | 310 | 163 | 147 |
| checked: ruled a swing without contact | 36 | 14 | 22 |
| checked: ruled no swing | 9,907 | 4,922 | 4,985 |

Checked swings are 13.2% of swings started; contact on 64.9% of full swings; fouls 29.3% of full-swing contact. Of checked swings 3.4% are strikes by the swing rule (3.0% because they touched the ball, 0.4% on the bat's yaw).

## 2. Every swing

| Quantity | mean | sd | p5 | p50 | p95 |
| :-- | --: | --: | --: | --: | --: |
| T_swing = t_c − t_s (ms) | 150.0 | 0 | 150.0 | 150.0 | 150.0 |
| plane tilt β (°) | 32.0 | 3.4 | 26.5 | 32.0 | 37.7 |
| hand radius ρ (m) | 0.56 | 0.09 | 0.42 | 0.56 | 0.71 |
| Δ = t_c − T_a (ms; negative = early) | −13.0 | 20.9 | −52.4 | −9.1 | 13.5 |
| final aim time error, aim − truth (ms) | −13.0 | 19.1 | −50.1 | −7.4 | 7.4 |

Every batter's `T_swing` is still 0.150 s: `QuickHands` does not vary it yet.

## 3. What the collision reads at the contact instant (full swings with contact, n = 43,590)

The two sets' means agree to within 0.1 in every row except launch angle (15.9 / 16.9°) and spray (−13.8 / −14.6°).

| Quantity | mean | sd | p5 | p50 | p95 |
| :-- | --: | --: | --: | --: | --: |
| contact − t_c (ms; positive = the bat further round than planned, an early swing) | 1.2 | 5.6 | −8.0 | 1.2 | 10.4 |
| sweet-spot speed (mph) | 71.2 | 4.3 | 64.3 | 71.1 | 78.4 |
| speed of the bat at the contact point (mph) | 73.1 | 7.9 | 60.0 | 73.1 | 85.9 |
| knob (hands) speed (m/s) | 6.39 | 0.96 | 4.86 | 6.37 | 8.01 |
| bat angular speed (°/s) | 2,046 | 142 | 1,818 | 2,042 | 2,284 |
| attack angle (°) | 9.1 | 5.9 | −0.7 | 9.1 | 18.7 |
| attack direction, pull + (°) | 2.2 | 9.2 | −12.2 | 1.8 | 17.9 |
| bat elevation (°) | −29.5 | 3.5 | −35.4 | −29.5 | −24.0 |
| bat azimuth (°) | +7.7 | 13.4 | −14.6 | +7.7 | +29.6 |
| bat axis against the sweet spot's path (°, + = tip turned ahead, hooked) | 0.27 | 1.52 | −2.34 | 0.34 | 2.65 |
| contact station: sweet spot − contact (in, + = toward the knob) | −0.95 | 3.23 | −5.91 | −1.01 | 4.54 |
| ball centre above the bat's axis (cm, + = undercut) | 2.10 | 2.88 | −3.21 | 2.38 | 6.32 |
| exit speed (mph) | 84.1 | 15.9 | 55.4 | 86.9 | 105.7 |
| launch angle (°) | 16.4 | 30.6 | −39.7 | 19.9 | 60.5 |
| spray, pull + (°) | −14.2 | 47.2 | −107.6 | −8.4 | 45.9 |

Fair balls only (n = 30,828): exit speed 88.9 mph (sd 13.8, p95 106.9), launch angle 8.3° (sd 28.2), spray +3.5° (sd 23.9), sweet-spot speed 71.3 mph, attack angle 9.6°.

**By timing** (contact − t_c):

| ms | n | sweet-spot mph | point mph | attack ° | direction ° | azimuth ° | hooked ° | station in | EV mph | LA ° | spray ° | foul % |
| :-- | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: |
| −26 to −10 | 1,003 | 68.6 | 65.8 | −3.3 | −17.4 | −23.4 | −3.6 | 1.43 | 80.5 | 9.1 | −60.0 | 68.5 |
| −10 to −3 | 8,885 | 70.9 | 69.0 | 3.0 | −8.4 | −8.3 | −1.6 | 0.90 | 83.6 | 15.2 | −38.9 | 47.9 |
| −3 to 3 | 17,176 | 71.7 | 72.2 | 8.2 | 0.1 | 5.2 | 0.0 | −0.31 | 86.8 | 17.5 | −16.7 | 25.9 |
| 3 to 10 | 14,042 | 71.3 | 76.2 | 13.3 | 9.4 | 18.6 | 1.6 | −2.40 | 83.7 | 17.2 | 4.1 | 17.0 |
| 10 to 26 | 2,484 | 69.4 | 79.1 | 18.1 | 20.6 | 32.5 | 3.0 | −4.69 | 70.6 | 11.8 | 6.6 | 40.0 |

**By location across the plate** (x inside of the plate's centre: outside < −0.072 m < middle < +0.072 m < inside):

| location | n | sweet-spot mph | attack ° | direction ° | station in | EV mph | LA ° | spray ° | foul % |
| :-- | --: | --: | --: | --: | --: | --: | --: | --: | --: |
| outside | 14,653 | 71.2 | 9.4 | 0.3 | −3.35 | 80.9 | 13.5 | −19.9 | 30.8 |
| middle | 15,890 | 71.2 | 9.2 | 2.4 | −0.91 | 87.6 | 17.0 | −11.4 | 27.5 |
| inside | 13,047 | 71.2 | 8.5 | 3.9 | 1.71 | 83.4 | 19.0 | −11.2 | 29.8 |

**Mechanism slopes** (least squares over full-swing contact):

| relation | slope | r |
| :-- | --: | --: |
| spray per ms of contact − t_c | +3.19°/ms | 0.38 |
| bat-against-path (hooked) per ms of contact − t_c | +0.271°/ms | 0.99 |
| launch angle per cm of undercut | +10.1°/cm | 0.95 |
| exit speed per mph of sweet-spot speed | 1.01 | 0.27 |
| spray per degree of attack direction | 1.97 | 0.38 |

## 4. Misses and checked swings

Full swings missed (n = 23,591): closest surface gap 21.1 cm mean (p5 0.4, p50 11.7, p95 72.7); closest approach 10.5 ms after t_c on average (the window's edge, +25 ms, at p95); Δ −25.1 ms mean (misses are early swings).

Checked swings (n = 10,253): the brake starts 82 ms before t_c on average (sd 32, p5 −132, p95 −29); the bat stops at azimuth −87° mean (p50 −98°, p95 −21°), having turned 31° (p50 19°, p95 96°) horizontally from its start pose.

## 5. What the rebuild must do with these

- **Match by construction** (the contact jet is kept, swing-model-v3 §5.2): at the planned contact instant the sweet-spot speed (71.2 mph on average, flat across the plate), attack angle (9.1°), attack direction (+2.2°, rising inside), bat elevation (−29.5°), the bat square to its path (0.0° at t_c) and the planned contact station; and to first order in the timing error, the rows of the timing table (direction, azimuth and hooked-ness per ms) and the slopes in §3.
- **Expect to move a little:** contact more than about 10 ms from t_c (8% of contacts) through the shape of the path's last 25 ms, and so the far rows of the timing table, fouls and misses there.
- **Expect to move a lot, knowingly:** checked swings. The rebuilt bat is much further back when the batter brakes (about −150° of azimuth instead of −87°), so the 310 touches and 36 yaw rulings (3.4% of checked swings, about 0.6 strikes a game) should almost vanish.
- **Not read by any outcome but a check of the new body**: the knob speed (6.4 m/s) and the bat's angular speed at contact (2,046°/s; Fortenbaugh 2,233°/s).
