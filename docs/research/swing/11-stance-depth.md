# Where MLB batters stand and where they meet the ball, relative to the plate (2026-11-24, worker stance-depth)

Question (spec [swing-model-v3.md](../../swing-model-v3.md) §5.3, §14 Q46, D26): MLB meets the ball 0.775 m in front of the batter's centre of mass ([9](9-step0-data.md) §3); the game meets it 0.05 m in front of its stance spot. Does MLB stand deeper in the box, or meet the ball further out front? No sim source was changed.

## 1. The source exists

Baseball Savant, **Batting Stance & Intercept** leaderboard: <https://baseballsavant.mlb.com/visuals/batting-stance> (`?seasonStart=2025&seasonEnd=2025&minSwings=q&gameType=Regular`). The per-batter table is embedded in the page as `const vizData = [...]` (the page's own CSV button was not needed). Saved under `sim/target/research-dl/` (`stance_2025.html`, `stance_2024.html`, `stance.html` = 2026 to date; not committed). Script: `analysis/statcast/savant_stance_depth.py <page.html>`.

Definitions, verbatim from the page's text:

- "This page allows you to see the average positioning within the batter's box for all hitters, as determined by their center of mass (defined as the midpoint of their hips)."
- "Batting stance is defined by two parameters: **Depth in Box**, which measures (in inches) how far behind the front of the plate the batter stands, and **Distance off Plate**, which measures how far off the "inside" edge of the plate the batter stands."
- "…the visualization also shows the average Intercept Point for each hitter, indicating the "average" point at which their bat is nearest to the baseball on a swing (whether or not they make contact). This can be measured either relative to the front of home plate (closest to the pitcher) or to the batter's center of mass."
- Column headers (page script): "Depth in Box" `avg_batter_y_position`, "Distance off Plate" `avg_batter_x_position`, "Distance between Feet" `avg_foot_sep0`, "Stance Angle" `avg_foot_angle0`, "Intercept vs. Front of Plate" `avg_intercept_y_in_vs_plate`, "Intercept vs. Center of Mass" `avg_intercept_y_vs_batter`.

What the definitions settle: the batter's point is the hip midpoint; the plate reference for depth and intercept is the **front edge** of the plate (the closest point to the pitcher); distance off plate is to the **inside edge**. What they do not say: **at which instant** the centre of mass is taken (stance, or contact) — the page calls the table "average positioning" and the foot columns are suffixed 0/1/2 ("how his foot placement progresses through a swing"; stages unlabelled). The columns' own suffix `0` on the foot-separation and stance-angle columns, and the page's title "Batting Stance", point to the stance, but this is an inference.

## 2. The table (2025, qualified batters, one row per batter and side, n = 253: 226 batters, switch hitters in both sides, traded players' total rows; 2024 in brackets)

Inches as on the page; metres in the game's frame for the means.

| column | all: mean (sd) | LHB | RHB | m |
|---|---|---|---|---|
| Depth in box (behind plate front) | 27.87 in (3.80) [27.78 (3.65)] | 28.23 (3.47) | 27.57 (4.04) | 0.708 (sd 0.097) |
| Distance off plate (inside edge to COM) | 27.63 in (2.57) [27.76 (2.51)] | 27.47 (2.75) | 27.77 (2.41) | 0.702 (sd 0.065) |
| Intercept vs front of plate (+ = toward pitcher) | +2.89 in (4.73) [+2.90 (4.32)] | +2.32 (4.25) | +3.36 (5.07) | +0.073 (sd 0.120) |
| Intercept vs centre of mass | 30.76 in (3.48) [30.68 (3.27)] | 30.56 (3.67) | 30.93 (3.31) | 0.781 (sd 0.088) |
| Feet apart / stance angle | 29.6 in (5.8) / −12° (12) | | | |

Handedness makes no real difference (≤ 0.7 in); 2024 and 2025 agree within 0.2 in; the per-pitch six-date mean of [9] (30.5 in) agrees with the table's 30.8.

Two facts about the columns:

1. **The table's columns tie out exactly.** For every row, `intercept vs centre of mass − intercept vs front of plate = depth in box` (to 1e-14), and `avg_intercept_y_in = vs_plate + 17` (the plate's 17 in depth). So the leaderboard's "intercept vs centre of mass" is the intercept measured from the *same* centre of mass that gives the depth. If that is the stance position, the 30.5 in of [9] is ball minus *stance* hips, and the stride is not inside it. If the depth is instead a contact-time position, the stride is inside it. Savant does not say which.
2. Deep stances go with balls met nearer the plate's front: corr(depth, intercept vs plate) = −0.69 (2024: −0.67). Batters who stand deep meet the ball farther out in front of the plate (as they must to be 30 in ahead of themselves); the spread of the body-relative intercept (sd 3.5 in) is much smaller than the spread of either plate-relative figure (3.8 and 4.7 in).

## 3. In the game's frame (origin rear apex of the plate, +Y to the pitcher, +X to first base; plate front y = 0.432 m)

| | MLB (2025 mean) | game | difference |
|---|---|---|---|
| Hips at stance, y | 0.432 − 0.708 = **−0.276 m** (sd 0.097 across batters) | `pivot_y: 0.30` ([swing.rs:109](../../../sim/crates/bb_agents/src/swing.rs), the swing template's pivot; spec §5.3 "0.30 m from the plate's back tip, set by eye") | **0.576 m (22.7 in) deeper** |
| Hips at stance, across | 0.702 + 0.2159 (half the plate) = **0.918 m** off the middle line (sd 0.065) | `pivot_x: -0.95` ([swing.rs:108](../../../sim/crates/bb_agents/src/swing.rs)), signed by side | 0.03 m nearer the plate than the game's 0.95 m: no change needed |
| Ball met, y | 0.432 + 0.073 = **0.505 m** (sd 0.120 across batters) | `contact_y0: 0.35` ([swing.rs:122](../../../sim/crates/bb_agents/src/swing.rs), middle of the plate; slope `contact_k` 0.125 across the plate) | **0.155 m (6.1 in) further out front** |
| Ball met, across | 0.004 m from the middle line (sd 0.027; L −0.015, R +0.020, sign convention not stated) | aim-dependent, centred | none |
| Ball minus hips (the 0.775 m of [9]) | 0.781 m | 0.05 m (0.35 − 0.30) | 0.73 m |

The two shifts add: 0.576 + 0.155 = 0.731 m = the body-relative gap (0.781 − 0.05). **About four fifths of the gap is where he stands (deeper), about one fifth is where the ball is met (further out front).** The game's `contact_y0` of 0.35 m (unsourced, [E]) is therefore too deep by only ~0.15 m; the stance spot is the large error.

Caveat on the meet point: Savant's intercept is the bat's *nearest point to the ball* averaged over all swings (misses included, on all pitch locations), not the contact point of the middle pitch only; [9] found y by location 29.8 (outside), 30.4 (middle), 32.1 (inside) in. The game's 0.35 m is a middle-pitch figure, so the 0.155 m is an average over the zone, up to ~0.04 m too high for a middle pitch if the same pattern holds in plate terms (not computed; plate-relative intercept by location is not published).

## 4. Cross-check against the body-relative intercept

Plate-relative stance (−27.87 in) plus plate-relative intercept (+2.89 in) gives 30.76 in; the table's body-relative figure is 30.76 in. They add up, but by construction (§2 fact 1), so this is a consistency check on the data, not an independent confirmation.

The stride question (the gap between "ball minus stance hips" and "ball minus hips at contact"): the feet columns show the front foot moves forward ~19 in and the back foot ~4 in between stage 1 and stage 2 (centroid of the six foot points: 0.86 ft = 10.3 in = 0.26 m from stage 0 to stage 2; stage definitions not stated, so this is an upper bound for the hips, which travel less than the feet). [4] has the pelvis travelling about 0.17 m ([9] §2: pelvis X −51 → −34 cm). If the leaderboard's centre of mass is the stance one, then at contact the ball is about 0.78 − 0.17 ≈ **0.61 m** ahead of the hips; if it is the contact one, 0.78 m. [9]/§5.3's hand-reach argument used 0.7 m: it sits between them. Which is true is **not established**.

## 5. What is established, what is not

Established (from the quoted definitions and a table that ties out exactly): MLB batters' hip midpoint averages 27.9 in (0.71 m) behind the front edge of the plate, i.e. at y = −0.28 m in the game's frame, and 27.6 in off the inside edge (0.92 m off the middle line); bat-to-ball nearest approach averages 2.9 in (0.07 m) in front of the plate's front edge (y = 0.505 m) and 30.8 in (0.78 m) ahead of that same hip midpoint. Stable 2024→2025, same for lefties and righties. Across: the game's batter is right.

Not established: the instant of Savant's hip position (stance or contact) and therefore how much of the 0.78 m the stride accounts for; the time stamp of foot stages 0/1/2; how the intercept (nearest point of approach) relates to the true contact point on the middle pitch; any sd within a batter (only per-batter averages are published). The sds quoted are across batters.

Suggested reading for the step-5 decision (the Coordinator's, not decided here): move the stance spot about 0.58 m toward the catcher and contact_y0 out by about 0.15 m. If the leaderboard's position is the stance one, the stride then carries the hips ~0.17 m forward by contact, as [4] has it. Check the three deeper-stance results in spec §5.3 (lag, bat-tip speed peaking at 1.14–1.22 times contact) again at this split; they were computed with the body 0.55–0.70 m deeper and the ball unmoved, so a 0.15 m shift of the ball changes them.
