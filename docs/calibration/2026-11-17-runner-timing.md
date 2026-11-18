# Where the runner loses the race home

**Q27.** How much could a moving secondary lead help a runner score from second on an outfield single? First to third is the check on whether the same change helps an advance the sim already gets closer to MLB.

## Method

The probe ran 600 games after merging `main` into `work/runner-start`. It used direct game seeds 0–299 and 1000–1299, with the same park, matchups and fielders as `sim-cli games --n 300 --seed 0`. The CLI multiplies its seed argument by 1,000,003, so `--seed 1000` selects a different second range. Each runner on first or second on a scorer-credited single first secured by LF, CF or RF gets a row. A play may contribute a runner from each base. GB is launch below 10°, LD is 10–25°, and FB is 25° or more.

All times start at contact. The probe reads the first movement and the first forward movement from roughly 0.10 s actor samples. Bag touches, ground contact, first glove contact, secure possession and releases come from play facts or the event log. A runner counts as sent if he touches the target or is seen more than 5 cm down the path past the intervening bag. A run must count in the rules to be safe. An actual delivery requires a throw toward the target and secure possession within 2 m of it. The probe also records relay flags and the time of the final release.

For a hold, the projected finish is the observed intervening touch plus 90 ft at the runner's full speed (3.35 s). For a tag beyond that bag, the projection resumes from the out spot at full speed. Where the intervening touch is absent, the CSV gives an uninterrupted running-model ETA from contact. The held-runner and flip estimates require an observed intervening touch and exclude a third base still occupied by a runner ahead.

The defense benchmark starts at measured outfield possession and adds 0.85 s of transfer plus a clean throw at the position's grade-50 arm. It takes the quicker of the direct flight and two flights via a shortstop halfway to the bag, with 0.455 s for his exchange. It assumes a ready receiver, and uses the sim's drag and backspin. The same benchmark applies regardless of where the defense actually threw. A positive margin means the runner is late. The benchmark counts who beats possession, with a separate sensitivity allowing 0.25 s for the tag.

```sh
cd sim
taskset -c 0-7 nice cargo run --release -q -p bb_game --example runner_timing > target/runner-timing.csv
```

## Results

Second to home was **42.3%** across outfield singles and **52.7%** with two outs. First to third was **28.3%**. The 2024 MLB figures of 61.7% and 31.7% are for singles generally. The MLB two-out figure of 75–91%, by hit direction, is the share sent. [Earlier runner-decision calibration](2026-11-15-runner-decision-model.md#part-1-sources).

| Advance | Outs | Chances | Sent | Safe | Outs on play | Held | Eligible holds beating ball / eligible |
|---|---:|---:|---:|---:|---:|---:|---:|
| Second → home | 0–1 | 549 | 198 | 194 (35.3%) | 2 | 351 | 21/311 (6.8%) |
| Second → home | 2 | 368 | 201 | 194 (52.7%) | 5 | 167 | 13/167 (7.8%) |
| First → third | 0–1 | 1356 | 441 | 409 (30.2%) | 20 | 915 | 27/733 (3.7%) |
| First → third | 2 | 618 | 172 | 149 (24.1%) | 16 | 446 | 27/339 (8.0%) |

Timing cells give the median [25th, 75th percentile] in seconds. Bag times use actual touches. Race arrival includes projected touches. The 0.30 s sample for the nominal 0.25 s two-out start reflects the actor sampling interval.

| Advance | Outs | First movement | Intervening bag | Target touch | Race arrival | Ball benchmark | Margin |
|---|---:|---:|---:|---:|---:|---:|---:|
| Second → home | 0–1 | 0.50 [0.50, 0.50] | 4.70 [3.78, 6.41] | 8.18 [7.15, 9.82] | 7.95 [7.13, 9.51] | 7.46 [6.59, 9.94] | 0.37 [-0.68, 1.18] |
| Second → home | 2 | 0.30 [0.30, 0.30] | 3.53 [3.53, 3.53] | 6.90 [6.90, 6.90] | 6.88 [6.88, 6.90] | 6.92 [6.37, 7.47] | -0.04 [-0.59, 0.52] |
| First → third | 0–1 | 0.50 [0.50, 0.50] | 4.87 [3.90, 6.58] | 8.73 [8.05, 9.75] | 8.27 [7.25, 10.06] | 7.40 [6.19, 9.88] | 0.84 [-0.09, 1.71] |
| First → third | 2 | 0.30 [0.30, 0.30] | 3.65 [3.65, 3.65] | 7.02 [7.02, 7.02] | 7.00 [7.00, 7.00] | 6.30 [5.82, 6.98] | 0.69 [0.01, 1.23] |

Each field and ball-type cell gives sample size, race margin, and eligible holds that beat the ball over eligible holds.

| Advance | Outs | Fielder | Grounders | Liners | Flies |
|---|---:|---|---|---|---|
| Second → home | 0–1 | LF | 63 · 0.32 [-0.71, 0.91] · 3/40 | 60 · 0.71 [-0.52, 1.21] · 2/33 | 17 · 0.34 [-0.32, 1.60] · 0/8 |
| Second → home | 0–1 | CF | 71 · -0.33 [-0.80, 0.08] · 14/34 | 115 · 0.25 [-1.68, 1.16] · 1/56 | 14 · 1.00 [-0.26, 1.80] · 0/9 |
| Second → home | 0–1 | RF | 71 · 0.64 [0.08, 1.06] · 1/51 | 119 · 0.96 [-0.28, 1.79] · 0/70 | 19 · 1.03 [-0.07, 1.84] · 0/10 |
| Second → home | 2 | LF | 48 · 0.56 [-0.13, 0.78] · 1/33 | 52 · 0.31 [-0.58, 0.60] · 1/35 | 3 · -1.50 [-1.56, -1.27] · 0/0 |
| Second → home | 2 | CF | 68 · -0.41 [-0.88, -0.14] · 6/13 | 70 · -0.18 [-0.60, 0.19] · 4/24 | 4 · -0.83 [-1.38, -0.66] · 0/0 |
| Second → home | 2 | RF | 65 · 0.43 [-0.20, 0.76] · 1/36 | 47 · 0.11 [-0.27, 0.61] · 0/24 | 11 · -0.13 [-0.23, 0.08] · 0/2 |
| First → third | 0–1 | LF | 151 · 1.61 [1.04, 1.94] · 4/104 | 196 · 1.44 [0.13, 2.29] · 3/108 | 27 · 2.26 [1.40, 2.84] · 0/19 |
| First → third | 0–1 | CF | 162 · 0.50 [0.11, 0.93] · 5/89 | 291 · 0.17 [-1.28, 1.80] · 6/109 | 33 · 1.57 [1.18, 2.09] · 0/24 |
| First → third | 0–1 | RF | 199 · 0.72 [0.36, 1.10] · 4/134 | 244 · 0.68 [-0.72, 1.60] · 3/107 | 53 · 1.53 [0.59, 2.22] · 2/39 |
| First → third | 2 | LF | 96 · 1.57 [0.97, 1.82] · 3/60 | 88 · 1.54 [0.88, 1.86] · 5/45 | 11 · 0.17 [-0.40, 0.78] · 1/3 |
| First → third | 2 | CF | 116 · 0.32 [-0.16, 0.67] · 5/67 | 108 · 0.57 [-0.01, 0.91] · 5/70 | 9 · -0.07 [-0.76, 0.22] · 1/3 |
| First → third | 2 | RF | 95 · 0.65 [-0.29, 0.98] · 3/45 | 80 · 0.63 [-0.03, 0.88] · 4/45 | 15 · -0.30 [-0.74, 0.14] · 0/1 |

- **Second → home.** With an extra 0.25 s for a tag, 77/478 (16.1%) of eligible holds beat the benchmark. The sim recorded 3 deliveries to this base, including 2 via another fielder. The ball arrived at 6.85 [6.61, 6.92] s. All three runners fell short of touching home. First releases, including throws elsewhere, were at 4.35 [3.80, 5.58] s.
- **First → third.** With an extra 0.25 s for a tag, 117/1072 (10.9%) of eligible holds beat the benchmark. The sim recorded 42 deliveries to this base, including 3 via another fielder. The ball arrived at 6.49 [6.33, 6.68] s. The 6 paired runner touches give a margin of 0.30 [0.25, 0.36] s. First releases, including throws elsewhere, were at 4.63 [3.92, 6.28] s.

Actual target-base deliveries are sparse and selected. Most throws went elsewhere.

## Where the time goes

Two bases take **6.69 s** at full speed. Leads of **6.50 m from second and 5.50 m from first** save **0.79 s and 0.67 s**, respectively. Standing acceleration adds **0.76 s** and the named read adds **0.50 or 0.25 s**. The table uses means for the pieces so the arithmetic adds up. It covers held runners who reached the intervening bag and lost to the benchmark.

| Advance | Outs | Losing holds | Read | Other runner time¹ | Ball secured | Exchange² | Flight² | Mean deficit | Median deficit [Q1, Q3] |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Second → home | 0–1 | 290 | 0.50 | 1.43 | 3.57 | 1.02 | 2.76 | 1.25 | 1.10 [0.68, 1.68] |
| Second → home | 2 | 154 | 0.25 | -0.03 | 2.84 | 0.91 | 2.59 | 0.54 | 0.55 [0.33, 0.76] |
| First → third | 0–1 | 706 | 0.50 | 1.31 | 3.87 | 0.96 | 2.34 | 1.42 | 1.35 [0.74, 2.00] |
| First → third | 2 | 312 | 0.25 | -0.06 | 3.10 | 0.85 | 2.09 | 0.92 | 0.84 [0.55, 1.28] |

¹ Compared with an uninterrupted run after the named read. This includes checks, traffic, early movement on grounders and a small tick-versus-curve difference. ² The full modeled route, with both legs and exchanges when relayed.

| Ball type | Landed (s) | Landed to secure (s) | Fielder traveled (m) | Secured (s) | Actual exchange (s) |
|---|---:|---:|---:|---:|---:|
| Grounder through | 0.35 [0.10, 1.00] | 2.84 [2.17, 3.35] | 15.17 [12.55, 18.37] | 3.22 [2.83, 3.62] | 0.88 [0.80, 0.97] |
| Liner | 2.44 [2.03, 3.13] | 1.45 [0.83, 2.61] | 21.34 [14.76, 30.85] | 4.17 [3.13, 5.58] | 0.98 [0.85, 1.07] |
| Fly | 4.40 [3.64, 5.02] | 0.77 [0.41, 1.33] | 28.96 [22.86, 35.74] | 5.63 [4.42, 6.25] | 1.02 [0.88, 1.08] |

The outfielder had possession by the time the runner touched third on 621/876 observed trips from second, and had released a throw on 315 of them. The modeled flight home took 2.68 [2.55, 2.93] s. The biggest runner-side difference by outs was the extra delay with fewer than two outs. Among losing holds from second, it averaged 1.43 s beyond the named read and normal acceleration. With two outs, the earlier start and uninterrupted run brought the median deficit of those losing holds to 0.55 s.

Estimated first-release speeds in the sim were LF 86.57 [84.72, 88.23] mph, CF 88.50 [86.47, 90.47] mph, RF 90.34 [88.13, 92.19] mph.

- **Runner.** The sim's 8.2 m/s (26.9 ft/s) is close to Statcast's competitive-play average of 27 ft/s. A standing 45/90 ft in the sim takes 2.40/4.10 s, before reaction. Bailey et al.'s worked Statcast example takes 2.45/4.13 s, and Coleman and Dupler measured a 4.32 s mean contact-to-first in MLB games. The sim's batter, with the separate 0.20 s start and route out of the box, is about 4.20–4.37 s. These comparisons cover straight running, with different starting conditions from a runner rounding third. [Savant sprint speed](https://baseballsavant.mlb.com/leaderboard/sprint_speed), [Savant 90-ft splits](https://baseballsavant.mlb.com/running_splits), [Bailey et al., Table 1](https://journal.iusca.org/index.php/Journal/article/download/163/279), [Coleman and Dupler, MLB game study](https://www.researchgate.net/publication/289906280_Differences_in_running_speed_among_Major_League_Baseball_players_in_game_situations).
- **Lead and first step.** MLB defines first step from contact to first movement toward the next base, and secondary lead at pitch release. The sim measures its fixed lead at bat contact. I found no published MLB average for the first-step delay on these singles, or for the secondary lead or speed at contact from second. MLB's published 11.5 ft gained between primary and secondary lead on steal attempts describes a different play. [MLB tracking glossary](https://www.mlb.com/news/major-league-baseballs-statcast-glossary-of-terms-of-state-of-the-art-tracking-technology/c-118508858), [MLB Statcast baserunning data](https://www.mlb.com/news/breaking-down-statcast-s-new-baserunning-stats).
- **Outfield.** Savant's 2025 LF/CF/RF arm-strength averages are 87.1/89.6/90.5 mph, the inputs the sim uses. Savant averages the top 10% of qualifying outfield throws. The probe's speeds include routine returns and are estimated over the first recorded span of flight. Statcast Jump measures distance over three seconds from pitch release on difficult catch chances. I found no published MLB pickup-time distribution for these singles, no comparable flight-time distribution, and no published outfield exchange-time average. [Savant arm strength](https://baseballsavant.mlb.com/leaderboard/arm-strength?minThrows=50&pos=&team=110&type=player&year=2025), [MLB Jump glossary](https://www.mlb.com/glossary/statcast/jump), [MLB tracking glossary](https://www.mlb.com/news/major-league-baseballs-statcast-glossary-of-terms-of-state-of-the-art-tracking-technology/c-118508858).

## Q27 estimates

Each cell counts **held / out** runners whose margin crosses from behind to ahead. Both are projected to continue through the intervening bag at full speed. For an out, this assumes the earlier runner avoids a later check or reversal. Chances blocked at the target or missing an intervening touch are excluded.

| Change from the merged branch | Time gained, 0–1 / 2 outs | Second → home, 0–1 | Second → home, 2 | First → third, 0–1 | First → third, 2 |
|---|---:|---:|---:|---:|---:|
| Lead +0.5 / +1.0 m, same start | 0.06 / 0.12 s | 3 / 0 | 11 / 1 | 4 / 0 | 12 / 3 |
| Start at 2 / 4 m/s, same read | 0.19 / 0.37 s | 15 / 2 | 44 / 3 | 21 / 4 | 55 / 11 |
| Remove the read only | up to 0.50 / 0.25 s | 39 / 1 | 23 / 3 | 56 / 5 | 29 / 5 |
| Moving at contact with both lead and speed changes | median 0.75 / 0.74 s | 72 / 2 | 112 / 4 | 130 / 16 | 128 / 15 |
| Defense 0.25 s later | 0.25 / 0.25 s | 19 / 2 | 24 / 3 | 34 / 6 | 29 / 7 |

**Two-out read alone, 0.50 → 0.25 s.** On the same plays, the current clock flips 33/368 second → home (9 holds, 0 outs) and 22/510 first → third (11 holds, 0 outs). An early ground touch caps the gain. The branch experiment on an earlier main, across all singles, raised two-out scoring from second from 35–37% to 43–45%. [Metrics ledger](../metrics-ledger.md).

The moving starts are illustrative. The added distance gives contact leads of 6.0/7.0 m from first/second with fewer than two outs, and 6.5/7.5 m with two outs. These estimates keep delays up to the intervening bag and hold the defense clock fixed. The removal of the read is capped at the first recorded forward movement where that was earlier. An early ground touch can already release a runner before the named read. The counts measure physically winnable races against this throw benchmark. Send decisions, the defense’s response and the risk of being doubled off remain open. Contact position and speed by outs, comparable MLB pickup and exchange times, and the cost of returning on caught liners and flies are still unknown.

## Q27 trace before moving starts

The merged base (`3b5dd55`) reproduced the original 290 losing holds from second and 706 from first. The opt-in runner trace records path position, signed velocity, goal and active movement branch at 60 Hz. Each tick is charged the change in its projected uninterrupted arrival at the intervening bag. That captures the distance and speed lost to a retreat as well as the time spent standing. Results below are seconds per losing hold. In the bulk capture the initial stationary hold shared the air-hold label. The first ground-touch tick, capped at the scheduled half-second read, separates those clocks to within a tick. The probe now emits those two causes separately.

| Start | Ball | n | Read vs 0.50 s | Air check | Ground hold | Traffic | Intervening bag | Other hold | Running step | Other runner time |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Second | Grounder through | 107 | −0.14 | 0.04 | 0.15 | 0.01 | 0.00 | 0.00 | −0.02 | 0.04 |
| Second | Liner | 156 | 0.00 | 1.86 | 0.00 | <0.01 | 0.00 | 0.00 | −0.02 | 1.83 |
| Second | Fly | 27 | 0.00 | 4.69 | 0.00 | 0.01 | 0.00 | 0.00 | −0.02 | 4.67 |
| Second | All | 290 | −0.05 | 1.45 | 0.05 | 0.01 | 0.00 | 0.00 | −0.02 | **1.43** |
| First | Grounder through | 314 | −0.20 | 0.01 | 0.00 | 0.00 | 0.00 | 0.00 | −0.03 | −0.22 |
| First | Liner | 312 | 0.00 | 2.10 | 0.00 | 0.00 | 0.00 | 0.00 | −0.02 | 2.07 |
| First | Fly | 80 | 0.00 | 4.38 | 0.00 | 0.00 | 0.00 | 0.00 | −0.02 | 4.35 |
| First | All | 706 | −0.09 | 1.43 | 0.00 | 0.00 | 0.00 | 0.00 | −0.02 | **1.31** |

The residual is about −0.01 s in each group. The historical projection uses a full-speed leg after the bag and a continuous curve from contact to the target. Walking the tick-by-tick ETA to the first touch uses the shorter leg to the intervening bag. A hold on that bag therefore adds zero to this particular deficit. It may still explain the decision to stay, and the projection is optimistic for any runner who has to stop there before going on.

The ground-hold column is the result of the ordinary advance decision on a ball already down. The merged base had no separate rule for the ball hit in front of an unforced runner from second. Watching a ground ball in front until it gets through, checking a liner and giving way to a lead runner are baseball decisions. The false start on the early bounce is a separate timing fault, described below. [Todd Guilliams, runner on second](https://coachesinsider.com/baseball/baserunning-runner-on-second-base/).

Every sample can be reproduced with `runner_timing -- <seed> <play> <runner>`. These are changes in the 60 Hz samples. Positions are metres along the path from home, and times start at contact.

| Seed / play / runner | Ball / outs | Movement and decisions |
|---|---|---|
| 1040 / 326 / 3 | Grounder / 0 | Stationary at 61.36 m through 0.05 s. The ball bounces at 0.037 s and the 0.05 s tick commands a return. At 0.50 s the scheduled read changes the goal to third while he is still going −3.69 m/s. At 1.28 s he stops at 59.01 m, then goes forward. He touches third at 4.85 s and holds. |
| 275 / 136 / 6 | Liner / 1 | At 0.50 s he chooses to go partway. At 1.70 s he is 0.02 m short of the hold point at 6.54 m/s. At 1.72 s he is stopped exactly on it. At 2.57 s, after the bounce at 2.558 s, he goes on. He touches third at 5.22 s and holds. |
| 141 / 63 / 263 | Fly / 1 | The 0.50 s read sends him back to tag. He reaches second at 2.43 s and waits there until the 5.82 s tick sees the ball down. He touches third at 9.90 s and holds. |
| 1010 / 107 / 263 | Grounder / 1 | The 0.10 s tick sees the ground touch and sends him back. At 0.50 s the read is blocked by the runner ahead. At 1.12 s the path clears while he is moving −5.19 m/s. He stops and reverses at 2.22 s, reaches third at 6.30 s and holds. |

Keep the liner and fly reads, and the traffic wait. Fix the premature bounce decision, which can send a runner back within 50 ms of contact and ask him to decide again at half a second. Also brake into the partway hold. A runner can choose to stop and wait in that spot, but he cannot lose 6.54 m/s in a single tick. The standing stop at the intervening bag is tracked separately in TASKS.md and contributes zero to the timing projection above.

## Q27 moving-start fit

The fit kept the primary leads of 5.5, 6.5 and 4.5 m. Each runner coasts at his contact speed for the read or reaction and then accelerates or brakes from that state. The runner on third shuffles in foul ground. The same contact parameters apply to all three bases. A force constrains the choice after the ball is read and ends on a catch.

The initial experiments used the same direct game seeds 0–119. The scores below include **all scorer-credited singles**. The two-out sends from second use only singles first secured by an outfielder. In the first two trials the read used the live out count. The last trial and final run use the outs at contact throughout the initial read.

| Additional metres / contact m/s, 0–1 out and 2 out | First to third safe | Second to home safe | Two-out outfield sends, LF / CF / RF |
|---|---:|---:|---:|
| 1.0 / 2 and 1.5 / 4 | 147/424 (34.7%) | 96/200 (48.0%) | 79% / 100% / 71% |
| 1.5 / 3 and 1.5 / 5 | 165/429 (38.5%) | 122/213 (57.3%) | 79% / 100% / 77% |
| 1.0 / 2.5 and 1.5 / 4.5 | 152/423 (35.9%) | 100/204 (49.0%) | 78% / 100% / 71% |

The final fit keeps the smaller 1 m shuffle with 0–1 out and uses 3 m/s at contact. It takes the stronger trial's 1.5 m and 5 m/s with two out, which helped the right-field send without making the 0–1-out lead larger. These are empirical inputs. There is no change to the run-expectancy values, tag allowance, or throw times.

### MLB check for runners doubled off

I counted **208 runners on 207 plays** across 2,429 regular-season games in Retrosheet's 2024 parsed play-by-play. That's **25.7 runners per 300 games**. The count uses `gametype=regular`, `hittype=L`, a catch coded with a single fielder number and a credited batting out, fewer than two outs, and a runner on base. For each runner, the advance field must show `1X1(...)`, `2X2(...)` or `3X3(...)`, with a recorded baserunning out in the corresponding `brout` column. There were 2,596 catches meeting the opportunity definition, of which 207 (8.0%) produced a double-off. Retrosheet labels line drives from play-by-play. The sim identifies them by launch angle, so the comparison also reports very low catches separately. [Retrosheet 2024 parsed play-by-play and field definitions](https://www.retrosheet.org/downloads/plays.html), [event coding](https://www.retrosheet.org/eventfile.htm).
