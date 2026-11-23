# The runner's late-flight catch read (2026-11-23)

Job: [runner-start note §6](2026-11-23-runner-start-open.md#6-plan-ranked), build first 1 and 2. Branch `work/late-catch-read` from main `3de8698`. Games: direct seeds 0–299 and 1000–1299, average fielders, as in that note. The probe hooks (uncommitted) and scan CSVs are in `sim/target/codex-probes/lc/`: `probe-hooks-lc.diff`, `zz_latecatch.rs` (one play, every 0.5 s: the truth, the runner's noise-free look, its projection and the eight paths' reads), `zz_latecatch_scan.rs`, the `an*.py` summaries.

## 1. Trace: why the noise-free read collapses

The runner-start note suspected the plate cue, the late look flown back to the plate with the typical spin of a lower ball. It is not the cause. On every traced play the plate cue's depth gain is below 0.35 from 2 s on, and a projection without the plate cue goes just as wrong. Three faults cause it, each in the catch read itself:

1. **The typical spin's axis on a steep ball.** The runner flies his look with the typical spin about an axis across the ball's horizontal heading (`believed_flight`). On a pop-up, the heading is only the drift, and the bat's path set the spin. Seed 1040 play 214 (77°, the catcher's catch in foul ground behind third):

   | Since contact | 1.5 s | 2.0 s | 2.5 s | 3.0 s | 3.5 s | 4.0 s | 4.5 s |
   |---|---:|---:|---:|---:|---:|---:|---:|
   | Noise-free score | 0.18 | 0.00 | 0.08 | 0.08 | 0.09 | 0.37 | 0.87 |
   | His projection | seats | seats | seats | seats | seats | seats | (−24, 3) |

   The true spin is about 8,900 rpm about (0.97, −0.19, 0.16), nearly +X. The ball drifts toward third at 4 m/s, so he believes 5,400 rpm about ±Y. Coming down, the true push is toward +Y (back toward the field), and his is toward −X. His projection lands about 6 m off, beyond the third-base stands, at (−26, −1) against the true (−21, 5). Every point past the wall scored nothing, so the read was 0. Seed 1256 play 243 (66°, the same corner) behaves the same way.
2. **A fielder running in from more than 5 m is taken to start from rest.** In the catch read, a fielder's visible speed toward a point counted only within `STEP_REACH_M` = 5 m of it. In seed 280 play 10 (61° to shallow left), the projection lands within 1–4 m of the truth all the way down. At 4.5 s the left fielder is 12.5 m from the ball, running in at about 7 m/s with 1.5 s to go. He was given 6.4 m of run from rest, and the read fell to 0.074, then 0.004 at 5.0 s. He made the catch.
3. **Any point past the wall is out of the question.** On foul pop-ups and flies near the stands, a projection a couple of metres into the seats reads a catch at the railing as impossible, whatever the curve he cannot see might do. Seed 108 play 142 (55°, a foul fly caught by the right fielder at the stands): with the axis fixed (1), the projection still lands 2.5 m into the seats, and the read is 0.000 at 2.5–3.0 s.

The runner-start note's other examples (1103/34, 1032/220) are no longer the same plays on this main.

## 2. The scan: 600 games, every runner, every 0.5 s of a first flight

For each runner on base with the ball on its first flight, the probe scored a noise-free look (zero draws) every 0.5 s with each variant. The table counts runners on balls that were caught. "Min" is the lowest score from 1 s after contact on. "Landing error" is the distance from his projection's first bounce to the true one, over all looks from 1 s on that land.

| Variant | Caught, min < 0.5 | Caught, min ≤ 0.2 | 50°+ caught, min ≤ 0.2 | Not caught (10°+), max ≥ 0.8 | Landing error, median / 90th pct |
|---|---:|---:|---:|---:|---|
| As built | 1,742 | 731 | 197 | 167 | 1.6 / 5.3 m |
| Fielder momentum at any range (2) | 694 | 187 | 114 | 182 | 1.6 / 5.3 m |
| (2) + graded wall (3) | 739 | 71 | 4 | 176 | 1.6 / 5.3 m |
| (2) + seen axis (1) | 694 | 173 | 101 | 181 | 1.0 / 4.0 m |
| **All three** | **676** | **48** | **2** | **166** | **1.0 / 4.0 m** |
| Learned constant acceleration instead of (1) | 3,079 | 1,315 | 530 | 186 | 0.9 / 6.7 m |
| No spin at all instead of (1), with (2) and (3) | 736 | 94 | 8 | 179 | 3.2 / 7.0 m |

6,075 runners were on caught balls (1,504 at 50° and above), and 9,139 on balls of 10° and above that were not caught.

- The fielders' tracker learns a constant extra acceleration. Flown from a single look, that is worse than nothing on a pop-up, because the push turns over at the apex.
- Flying with no spin past the apex lands 3 m off at the median.
- The seen axis, keeping the typical spin's size, is the one that tracks the ball.
- None of the three makes a hit look like a sure catch: the 0.8+ reads on balls that dropped stay at 166 against 167.
- The remaining 676 dips below 0.5 are mostly fielders running or diving for the ball (liners at 10–40°, outfielders on the run), where the doubt is real.
