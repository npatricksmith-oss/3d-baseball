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

The seen-axis variant in this table took the push between the last two looks, 0.5 s apart. That version swung near the apex, where the push is weak and turns quickly: in seed 1256 play 243 at 3.0 s its axis came out 60° wrong, and the descent was flown about it. The built version measures the push from his first look on, with the fielders' tracking-read weighing (agent-models §6.3, "The curve he has seen").

## 3. The built read

The same scan on the built read (branch `b58dd0b`; the games differ a little from main's, so the counts are not paired):

| | Caught, min < 0.5 | Caught, min ≤ 0.2 | 50°+ caught, min < 0.5 | 50°+ caught, min ≤ 0.2 | Not caught (10°+), max ≥ 0.8 | Landing error, median / 90th pct (50°+) |
|---|---:|---:|---:|---:|---:|---|
| Main `3de8698` | 1,742 of 6,075 | 731 | 370 of 1,504 | 197 | 167 of 9,139 | 1.6 / 5.3 m (1.3 / 4.0 m) |
| Built | 650 of 5,981 | 49 | 105 of 1,507 | 3 | 173 of 8,656 | 1.3 / 3.9 m (0.8 / 2.7 m) |

The tests (`live.rs`, `late_read_scores`) fly the five named plays' launches, each turned through −30° to 30° of spray in 10° steps. Thirty-five balls are flown, and 23 come down in play.
- **Fielder standing where the ball comes down.** The noise-free read stays at 0.5 or more on every look from 0.5 s to 0.3 s before the landing, for every ball that comes down 3 m or more from a wall. Within 3 m of a wall (a catch at the railing: 1256/243, 108/142 and 8/75 come down 0.4–0.6 m from the stands), it stays above the advance cut-off of 0.2; it was 0.39–0.50 there. A metre of curve that he cannot see decides whether such a ball is in the seats.
- **Fielder running in at 7 m/s from 33 m.** The read stays above the same floors from 1 s on.

On main these plays read 0.000–0.18 in mid-flight.

## 4. Step 1 in games

Standard run (`sim-cli games --n 300`), main `3de8698` → `b58dd0b`, seeds 0 [1000]: runs 4.12 → 4.14 [4.45 → 4.41]. The other figures are in the [ledger](../metrics-ledger.md).

Runner outcomes (`zz_lc_rates`, seeds 0–299 and 1000–1299 direct, average fielders):

| | Second to home on a single, 0 / 1 / 2 out | First to third on a single, 0 / 1 / 2 out | Doubled off on catches: <10°, 10–25°, 25–50°, 50°+ | Advanced on a caught fly, 0–1 out, from first / second / third |
|---|---|---|---|---|
| Main `3de8698` | 84/205, 165/373, 307/407 | 186/548, 268/871, 397/807 | 118, 31, 1, 0 | 8, 123, 248 |
| Step 1 | 85/207, 168/370, 311/410 | 190/541, 263/869, 400/809 | 114, 26, 0, 0 | 7, 132, 251 |

The read matters today only to runners already going or holding partway, who relook. A runner who tags up never looks again, so the change to his read cannot show yet. Every figure is within run-to-run noise.

## 5. Step 2: the tagging runner keeps looking

A runner tagging up on a ball still in the air now looks again every `RUNNER_FLY_LOOK_S`, on his bag or on his way back to it (agent-models §6.3). At a catch score of 0.2 or less, he gives up the tag-up and decides as at contact. Before, he never looked again. The fixture `a_tagging_runner_leaves_his_bag_when_he_sees_nobody_can_get_there` fails without the change.

| | Second to home on a single, 0 / 1 / 2 out | First to third on a single, 0 / 1 / 2 out | Doubled off on catches: <10°, 10–25°, 25–50°, 50°+ | Runs, seeds 0 [1000] |
|---|---|---|---|---|
| Main `3de8698` | 84/205, 165/373, 307/407 | 186/548, 268/871, 397/807 | 118, 31, 1, 0 | 4.12 [4.45] |
| Step 1 | 85/207, 168/370, 311/410 | 190/541, 263/869, 400/809 | 114, 26, 0, 0 | 4.14 [4.41] |
| Step 2 | 88/206, 176/368, 309/414 | 204/553, 274/879, 399/811 | 115, 28, 0, 0 | 4.18 [4.43] |

- **Second to home with 0–1 out:** 43.8% → 46.0% (main 43.1%; Retrosheet 2024 49.1%). That is +2.2 points, against the runner-start note's +3.8 from its probe on the old read.
- **First to third with 0–1 out:** 32.1% → 33.4% (note: +0.5).
- **Noise:** about 2 points per arm on about 575 chances, so both moves agree with the note's estimate.
- **Pop-up double-offs stay at 0.** The probe on the old read freed tagging runners on caught pop-ups and doubled off 8 at 50° and above. MLB has about 2 per 600 games, so 0 is near it.
- **25–50° double-offs:** also still 0, against MLB's about 8 (runner-start note §4).
- **Cost:** time per game +3.5% [+4.3%], from the tagging runners' relooks.

The two deferred P3s from the catch-fixes re-review went in with this step:
- the comment at `awaiting_call`'s no-catch clause, which adds that a tagging runner's relooks stop at the glove's touch, so they would not free a stranded runner either;
- `random_runner_layouts_with_a_catch_call_pending_never_leave_two_on_a_base`: 3,000 layouts after a fly caught 0.1 s before, with the planted tag-up runners seeing the call at once and the others not yet. It passes.

## 6. Open

- **25–50° double-offs**, at 0 against MLB's 8 per 600 games: runner-start note §4.
- **Second to home with 0–1 out**, still 3 points under Retrosheet. The note's remaining levers are caution by outs (Q38) and the coach reading a ball behind the runner (Q10 (a)).
- **The spin's size.** He flies the typical size about the axis he has seen. A ball with much more spin than typical (1040/214: 8,900 rpm against 5,400) still lands a metre or two off. At the railing, that decides whether a catch reads likely.
- **Time.** The seen axis is worked out again in each of a look's nine projections. Working it out once per look would win back most of step 1's +2–3% without changing a game.

