# Chase rate (O1): diagnosis, 2026-11-11

Owner decision O1 asked how to raise the average hitter's chase rate (O-Swing 11.6% vs MLB 29%). The owner approved trying **perceptual overconfidence**: the batter's filter believes its persistent biases are smaller than they are. It was implemented, measured, and **backed out**: it is the wrong lever. This report records why and what the measurements point to instead. No calibration values changed.

All runs: `sim-cli pa --n 3000 --k-perception 2.25 --k-command 0.8` (average RHP vs average RHB, seed 1) and `cargo run --release -p bb_game --example chase_diag -- 3000 [c_D] [b_D]`.

## 1. Pitchers are not missing too far outside

Out-of-zone pitches by how far the ball's edge missed the zone (Statcast attack zones; MLB values are approximate league figures):

| Band | Share of out-of-zone pitches | MLB ≈ | Swing rate | MLB ≈ |
|---|---|---|---|---|
| Shadow, 0–8.5 cm out | 42.7% | 40% | 20.1% | 43% |
| Chase, 8.5–25.5 cm | 48.2% | 41% | 5.7% | 23.5% |
| Waste, > 25.5 cm | 9.1% | 19% | 3.7% | 5.5% |

Pitches are, if anything, closer to the zone than in MLB. The gap is in the swing rates band by band, so it is on the batter's side. (`sim-cli pa` now prints this table.)

## 2. Overconfidence about perceptual bias: tried, rejected

Believed bias σ = trust × true σ:

| trust | O-Swing% | Contact% | Z-Contact% | SwStr% | K% |
|---|---|---|---|---|---|
| 1.0 (now) | 11.6 | 75.2 | 79.5 | 10.2 | 14.1 |
| 0.5 | 15.2 | 68.0 | 73.7 | 13.1 | 19.0 |
| 0.25 | 16.9 | 63.5 | 69.6 | 14.7 | 22.1 |
| 0.0 | 18.0 | 61.3 | 67.9 | 15.8 | 23.6 |

Even a batter with no idea his eyes can be wrong chases only 18%, and he pays for it everywhere else: the same overconfident estimate aims the swing, so contact falls out of band (Z-Contact 68–74 vs envelope 79–91). The mechanism mostly makes hitters miss, not chase.

## 3. Where the chase gap actually is

What the batter believed at commit, by where the pitch really crossed:

| Band | Swing% | MLB ≈ | Mean believed P(strike) | Mean EV(swing) − EV(take) |
|---|---|---|---|---|
| Heart | 76.4 | 73 | 0.82 | +0.041 |
| Shadow, inside the zone | 50.5 | 63 | 0.61 | +0.006 |
| Shadow, outside | 20.1 | 43 | 0.37 | −0.035 |
| Chase | 5.7 | 23.5 | 0.22 | −0.076 |
| Waste | 3.7 | 5.5 | 0.18 | −0.102 |

The batter is already uncertain where it matters: on a pitch just off the plate it gives 37% to "strike". Perception is not too sharp. The decision is: with that belief, the hand-written value model (agent-models §3.5, the [E] count values, `bip_value`, `FOUL_SHARE`) says taking is better, and the batter obeys it exactly. The sim's swing rate falls from 76% to 6% across the zone edge; MLB's falls from 73% to 23.5%.

## 4. Discipline's own parameters do move it

`logit(P̃) = c_D·logit(P) + b_D`. Grade 50 is c_D 1.0, b_D 0.0 today.

| c_D, b_D | Heart | Shadow in | Shadow out | Chase | Waste | O-Swing% | Z-Swing% | K% | BB% | Pitches/PA | Called-K share |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1.0, 0.0 (now) | 76.4 | 50.5 | 20.1 | 5.7 | 3.7 | 11.6 | 65.0 | 14.1 | 10.1 | 3.1 | 41.7 |
| 1.0, 1.0 | 83.6 | 63.9 | 28.6 | 8.2 | 4.7 | 16.6 | 74.9 | 11.8 | 7.5 | 2.9 | 21.4 |
| 0.6, 1.0 | 82.4 | 65.6 | 34.1 | 12.5 | 6.8 | 21.1 | 75.0 | 11.8 | 7.0 | 2.8 | 13.3 |
| 0.4, 0.8 | 79.4 | 63.2 | 35.3 | 14.2 | 7.6 | 22.6 | 72.2 | 12.9 | 7.1 | 2.8 | 11.1 |
| MLB ≈ | 73 | 63 | 43 | 23.5 | 5.5 | 29 | 67 | 22.5 | 8.3 | 3.9 | 25 |

A blurrier, swing-leaning strike judgment (c_D below 1, b_D above 0: the opposite direction from the "c_D > 1" floated in the M3 interim report) brings O-Swing into its envelope and gives roughly the MLB shape. But K% and pitches per PA get **worse**, because every extra swing that makes contact ends the plate appearance.

## 5. The binding problem is fouls

Foul share of contact is 21% against MLB's 50%. With so few fouls, contact almost always ends the PA, so plate appearances are short (3.1 pitches vs 3.9), two-strike counts are rare, and strikeouts are low whatever the chase rate. More chasing without more fouls shortens PAs further. Chase rate and foul rate have to be calibrated together, and fouls come first: they belong to the contact model (timing and bat-angle errors at contact), the same area as the "too many topped balls" item in TASKS (run environment 2(c)).

## 6. Proposed (owner to rule)

1. Fix the foul share in the contact model first (mechanism, not a knob).
2. Then set Discipline's grade-50 values to a blurrier, swing-leaning judgment (about c_D 0.5, b_D 0.9; re-fit after step 1), keeping the grade spreads, and update agent-models §10.
3. Do not add a perceptual-overconfidence parameter.
