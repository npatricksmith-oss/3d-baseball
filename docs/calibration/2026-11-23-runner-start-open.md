# What is left of the runner's start (Q27): diagnosis and plan (2026-11-23)

Diagnosis only. No sim source changed. Main at `bda4b46`. The probes ran on direct game seeds 0–299 and 1000–1299 (600 games, average fielders, as in the [runner-timing note](2026-11-17-runner-timing.md#final-measurements)). Throwaway files are in `sim/target/codex-probes/rs/` and outside the tracked files: the runner-timing CSVs, `zz_runner_open.rs`, `probe-hooks.diff` (the uncommitted trace hooks it needs), and the Retrosheet scripts `retro_send2.py` and `retro_back.py`. The hooks left the games unchanged: the probe CSV and the seed-0 digest `f3a147d0…` were identical with and without them.

## Summary

1. **Two-out sends to center are not too high. The corners are too low.** The 75–91% band came from a blog that gives no direction and no count. Retrosheet 2024, by fielder, shows two-out runners from second sent home on **91.3% of singles to left, 96.2% to center and 92.9% to right**. The sim now sends **96.5% to center** (109/113) but **81% to left and 82% to right**. At the corners, the held runners face even races that they would win, because of the catcher's tag time. Too much caution holds them, not the race.
2. **The single noisy look barely matters. The runner who tags up never looking again does.** With a noise-free look, the rates do not move. If a tagging runner kept looking, second to home with 0–1 out would rise by 3.8 points, toward MLB. But his in-flight catch read collapses to about 0 in mid-flight on high flies and pop-ups, even with no noise. Relooks would then double runners off on pop-ups, unless that read is fixed first.
3. **Braking onto the bag costs 0.45–0.75 s on a contested return.** About 90 runners per 600 games are tagged on grounders 1.2 m short of their own bag while braking. MLB has about 3 per 600. A dive or slide back would save nearly all of them, and 60–80 of the 151 appeal outs on low liners.
4. **Double-offs at 25–50°**: none of the fixes above raises them, and the dive back lowers them a little more.
5. **Of the deferred P3s**, two go with the fixes and three can wait.

## 1. Two-out sends from second on singles

### The MLB figure

The 74.6–90.7% range comes from [Malter Analytics, 11 Aug 2026](https://www.malteranalytics.com/blog/2026-08-11-baserunning-aggression-re24) (read again today). The page says only "74.6–90.7% of the time here depending on direction". It does not say which direction has which rate, gives no counts, and does not say whether infield singles are included. Its interactive tables did not load. The [decision-model note](2026-11-15-runner-decision-model.md#12-cross-check-and-further-splits-malter-analytics) already recorded that the by-location rates were a gap.

I counted the rates myself from Retrosheet's 2024 parsed play-by-play (`2024plays.csv`, 2,429 regular-season games; [downloads](https://www.retrosheet.org/downloads/plays.html)). The opportunities are a runner on second on a single coded `S7`, `S8` or `S9`, by the fielder who fielded it. "Sent" means an advance of `2-H` or `2XH`. "Out" means `2XH` with a recorded baserunning out.

| Retrosheet 2024, runner on second, outfield single | Left | Center | Right |
|---|---:|---:|---:|
| Two out: sent | **504/552 (91.3%)** | **525/546 (96.2%)** | **498/536 (92.9%)** |
| Two out: out at home, share of those sent | 28 (5.6%) | 3 (0.6%) | 11 (2.2%) |
| 0–1 out: sent | 395/757 (52.2%) | 479/765 (62.6%) | 447/800 (55.9%) |

### The sim

All rows are outfield singles credited by the scorer (`runner_timing`; the runner reached third, so every row is a chance).

| Sim, 600 games | Left | Center | Right |
|---|---:|---:|---:|
| Two out: sent | 77/95 (81.1%) | **109/113 (96.5%)** | 91/111 (82.0%) |
| Two out: out at home | 0 | 2 | 4 |
| 0–1 out: sent | 63/134 (47.0%) | 117/176 (66.5%) | 65/157 (41.4%) |

The 117/118 figure was measured on the merged-base build of 2026-11-17. Since then the relay exchange (0.75 s), the double cut and the approach fixes have slowed the defense's side. Center now matches MLB, within one standard error (about 1.7 points). **The gap is at the corners: about 10 points with two out, and 5 (left) and 14 (right) points with 0–1 out.**

### Each leg, two out (medians over the chances)

| Leg | Left | Center | Right | Source |
|---|---:|---:|---:|---|
| Runner's lead at contact | 8.0 m | 8.0 m | 8.0 m | Two-out secondary lead +1.5 m at 5 m/s (`RunnerStartParams`). No MLB average is published (runner-timing note) |
| Runner, contact to third base | 2.75 s | 2.75 s | 2.75 s | — |
| Runner, contact to home, when he scored | 6.12 s | 6.12 s | 6.12 s | 8.2 m/s ≈ Statcast's 27 ft/s. Statcast's "extra bases" clock runs from contact, but no league average is published. Search extracts give Duffy's 5.46 s as the fastest of 2016 and a 2025 clip of 6.57 s (`snippet`; mlb.com refuses automated reads) |
| Ball fielded (secured) | 3.05 s | 3.32 s | 3.12 s | No published distribution |
| Fielder's run to the ball | 13.3 m | 16.1 m | 14.1 m | — |
| Exchange, catch to release | 0.95 s | 1.10 s | 1.03 s | No outfield figure. The catcher's average is 0.73 s ([MLB.com, 2017](https://www.mlb.com/news/statcast-adds-pop-time-exchange-arm-strength-c268064274), `snippet`) |
| First throw's speed | 88.7 mph | 89.7 mph | 92.3 mph | Savant 2025 top-10% means: 87.1 / 89.6 / 90.5 mph |
| Clean throw home, flight | 2.60 s | 2.77 s | 2.60 s | Lang 2014: 280 ft in 2.47–2.57 s ([throw-race note §4](2026-11-19-throw-race.md#4-the-balls-parts-against-the-sources)) |
| Throws through a cutoff | 0 | 0 | 0 | — |
| Clean ball home (benchmark) | 6.51 s | 7.27 s | 6.50 s | — |
| Runner minus benchmark | −0.39 s | −1.15 s | −0.38 s | — |

The center fielder fields later, runs farther and has a longer throw. Center's race is about 0.75 s easier than the corners', and MLB's center rate is also the highest of the three. No leg is out of line with its source. The parts with no source (pickup, exchange) are those the [throw-race note](2026-11-19-throw-race.md#6-conclusion-stop-and-report-job-step-2) already left open.

### Cause at the corners: the runner holds on races he would win

Two-out chances at the corners, grouped by the runner's margin against the clean throw home (negative means the runner is ahead):

| Runner minus clean ball | Chances | Sent | Out |
|---|---:|---:|---:|
| ahead by more than 0.3 s | 129 | 128 | 0 |
| 0.3 s ahead to even | 52 | 40 | 0 |
| even to 0.15 s behind | 30 | **8** | 0 |
| 0.15–0.30 s behind | 6 | 2 | 0 |

- At center only 12 of 113 chances fall within ±0.5 s. At the corners, 73 of 206 fall within ±0.3 s.
- The 42 held runners at the corners were all on balls secured 2.4–2.7 s after contact, before the runner reached third. Their margins ranged from 0.11 s ahead to 0.22 s behind (for example seed 0 play 34, seed 1013 play 152, seed 1283 play 28).
- A runner who reaches the plate up to about 0.15 s after the ball is still safe, because the catcher needs `tag_apply_s` = 0.25 s to put on the tag. None of the 10 runners sent from that band was out.
- The decision rule explains the cut-off. Break-even is 41% with two out. With `caution_z` = 1.0 and σ 0.20 s, he goes only when he believes he is about 0.16 s ahead of ball plus tag, which puts the cut-off near zero against the ball alone.
- MLB sends these runners and loses 2–6% of them. The sim loses none at left and 4 of 91 at right, and those 4 were trapped runners, not lost races (below).

The race is not the cause here. If the ball were too quick but the caution right, the marginal runners the sim does send would be thrown out at about MLB's rate. **The cause is how cautious the runner's decision is**, as the [decision-model note](2026-11-15-runner-decision-model.md) foresaw ("`caution_z` … not yet fitted to attempts and success by situation"; TASKS item "Fit `AdvanceParams::caution_z`"). Coaching agrees: "with two out the coach decides before the pitch to send the runner from second home on a single" (Guilliams, decision-model note §1.6).

With 0–1 out the corners are short for other reasons. Of the held runners, 34 of 71 at left and 36 of 92 at right were on ground balls through the infield; at center only 7 of 59. Their average losses were 1.35 s (left) and 1.88 s (right) to air checks, which covers ground given up and waiting for the ball to drop, and 0.32 s to the read. Holds on liners are partly a real "freeze on a liner". The right-field deficit also includes the ball behind the runner, which the coach should read for him (owner's Q10 (a), not built).

**Trapped runners.** Six of the eight "outs" on sends from second had a benchmark margin 4–5 s behind, for example seed 82 play 83 and seed 1199 play 67. The runner went past third, the ball beat him home by about 2 s, and he was tagged 7.3–8.1 s after contact on his way back. This is the rundown stand-in (`MAX_TURNS`, Q11), not the start.

### Proposed fix

**Fit `caution_z` by outs** to Retrosheet's attempt and success rates by situation, starting with two-out sends from second, at about 92% sent at the corners and 2–6% of sends out. The present cut-off would need to move about 0.15 s; at the present σ that is `caution_z` near 0.3 with two out.
- **Expected effect:** the corners rise from 81–82% to about 92% (estimated from the table above). Center cannot rise much from 96.5%.
- **Risk:** `caution_z` governs every advance and tag-up. Changing one value for every situation would also send runners from first to third more often (now 36.4% against 31.7%) and raise center with 0–1 out (already 66.5% against 62.6%). The change should therefore depend on outs, or on the coach's two-out pre-decision, not be global.
- **Needs the owner:** it changes the shape of the decision model, not only a number.

## 2. The catch forecast from a single noisy look

**What the sim does.** With 0–1 out, each runner takes one keyed noisy look 0.5 s after contact and scores the catch over eight error paths.
- A runner who **goes** or **holds partway** looks again every 0.5 s (`fly_relook`).
- A runner who **tags up**, because the read is sure or because no hold is recoverable, never looks again. He waits for the public catch or ground ruling.

**Measured**, using a trace of every forecast and a noise-free counterfactual look (the same observation with zero draws):

| On 6,022 first looks at fielded balls | |
|---|---|
| How far the noise moves the catch score | median 0.018, 90th percentile 0.105, 99th percentile 0.29 |
| First looks whose action (tag, hold, go) the noise changes | 360 (6.0%) |
| Tagged up, but the ball dropped | 280: 254 in the outfield, 111 from second. 122 were sure-catch reads, mostly low liners at 7–15° sinking in front of the outfielder (for example seed 228 play 220, seed 1249 play 103); a real runner freezes on these too. 158 had no hold he could get back from |
| Went, but the ball was caught | 55, of which 15 were doubled off |

Effect on the rates (600 games; second to home and first to third on singles):

| | Second to home, 0 / 1 / 2 out | First to third, 0 / 1 / 2 out | Doubled off: <10°, 10–25°, 25–50°, 50°+ |
|---|---|---|---|
| As built | 86/219, 165/407, 305/419 | 169/547, 243/920, 379/851 | 121, 30, 1, 0 |
| Noise-free looks | 83/211, 164/409, 301/409 | 166/533, 241/929, 382/849 | 121, 29, 1, 0 |
| Tagging runner keeps looking (goes at a catch score ≤ 0.2) | **99/218, 175/406**, 298/413 | 176/552, 249/938, 378/847 | 120, 33, 3, **8** |

- **The noise is not the problem.** With a noise-free look nothing moves beyond run-to-run noise.
- **Never looking again after a tag-up costs runs.** With relooks, second to home with 0–1 out rises from 40.1% to 43.9%. The 2024 Retrosheet figure is 49.1% on all singles (runner-timing note).
- **The relook has a fault of its own.** Its read of a high fly or pop-up collapses during the flight, and a noise-free look does the same. Examples:
  - Seed 1040 play 214, 77°, caught by the catcher: the score reads 0.71 at 0.5 s, 0.000 from 2.0 to 3.5 s, and 1.00 at 4.5 s.
  - Seed 280 play 10, 61°, to left: 0.71, falling to 0.004 at 5.0 s, then 0.79 at 6.0 s.
  - Seeds 1103/34, 1256/243 and 1032/220 behave the same way.
- **The likely cause** is the known fault in the plate-cue projection: "liners still up after 3–4 s miss by 20–40 m when flown back with the typical spin" ([fly-read note, Open](2026-11-19-fly-read.md#open)). The runner flies a late look with the typical spin of a much lower ball. This has not been traced to the line.
- **Why it shows only in the counterfactual.** Today, a runner already going on a pop-up keeps going until the last look and gets back in time. A runner freed from a tag-up by that read goes, and is doubled off: 8 at 50° and above, against MLB's roughly 2 per 600 games on infield pop-ups ([fly-read note §3](2026-11-19-fly-read.md#mlb-rates-retrosheet-2024-parsed-play-by-play)).

**Proposed fix, in this order:**
1. **Trace and fix the late-flight read.** A late look should fly with the spin it has already seen, or with no spin prior once the apex is passed, much as the fielders' tracking reads do. Test: the noise-free score never falls below 0.5 on a ball that the fielder standing under it catches.
2. **Then let the tagging runner keep looking** (agent-models §6.3, "He keeps watching while it is up"): every `RUNNER_FLY_LOOK_S` while the ball is up, and at a score of 0.2 or less he decides as usual. Expected effect: second to home with 0–1 out about +4 points; first to third +0.5. Risk: double-offs on pop-ups if step 1 is not done first.

## 3. Returns brake onto the bag; no dive back

**What the sim does.** A runner going back to a base he will stop on limits his speed so that he can stop at the bag (`approach_speed`, `BRAKE` = 4.75 m/s²), and arrives at 1 m/s. He cannot dive or slide back.

**What a real runner does.** On a contested return he does not stop short. He dives headfirst at the bag, or slides into it, and keeps his speed to the touch. This is the ordinary pickoff and line-drive return. Using the sim's own laws of motion (`dive.py`), returning 4–15 m to the bag, braking costs:

| Return of | From rest | Moving away at 3 m/s | Moving away at 5 m/s |
|---|---:|---:|---:|
| 4 m | 0.45 s | 0.52 s | 0.59 s |
| 8 m | 0.63 s | 0.66 s | 0.70 s |
| 15 m | 0.74 s | 0.74 s | 0.76 s |

The comparison is a run through to the bag, with the hand reaching it from 2 m out at the speed he has then. A real dive loses a little speed in the air, so these savings are upper bounds of about 0.1 s.

**How often it would change an out** (600 games, outs on the runner at the base he started from):

| | Sim | Within 0.3 s | 0.3–0.5 s | 0.5–0.7 s | MLB 2024 (Retrosheet, per 600 games) |
|---|---:|---:|---:|---:|---|
| Tagged going back on a ground ball | **90** | all 90 (margins 0.09–0.17 s) | — | — | about 3 (`G`, runner out at his own base, no catch) |
| Appeal outs on catches | 152 (121 below 10°, 30 at 10–25°) | 26 | 34 | 20 | about 51 on liners, about 8 on flies |

- **Ground balls.** In the typical case the runner is caught 1.15 m from his bag at −3.3 m/s, still braking. Examples: seed 95 play 36, and seeds 1239/60, 287/10, 276/181 and 1024/188 with identical numbers. A runner on third breaks about 15 m toward home on a grounder to first or third, turns back at 2.6 s, starts braking 4.4 m out at 7.4 m/s, and is tagged after the throw across. He is out by 0.13 s. A dive back saves about 0.5 s, so all 90 would be safe. Part of the cause is his 15 m break on a ball in front of him ("ball in front, hold", §1.6 of the decision-model note).
- **Appeals.** A 0.5–0.7 s saving would rescue about 60–80 of the 152, which brings the sim from about 3 times MLB's liner total to about 1.5 times.

**Proposed fix.** Add a dive or slide back to agent-models §6.4.
- **When:** a returning runner whose estimated race to the bag is close (`run_time_to_stop` against his read of the ball's arrival) stops braking. He leaves his feet `DIVE_BACK_M` from the bag ([E], about 2–2.5 m; to be sourced) and touches it with his hand at the speed he has.
- **Cost:** he is down for a get-up time before he can go again.
- **The rules:** they judge the touch from the hand's position. The tag geometry needs a lying runner, as for a slide (TASKS §1: "no slide into a tag").
- **Expected effect:** about 90 fewer tag outs on grounders and 60–80 fewer appeals per 600 games, or about 0.25 fewer outs on the bases per game. Runs per game rise by roughly 0.1, against the unexplained −0.21 in ledger §5.
- **Risk:**
  - Double-offs at 10–25° fall below MLB's 8.0% (now 6.6%).
  - Double-offs at 25–50° stay at about 0.
  - A runner who dives cannot go on a misplay until he is up.
- **Needs the owner:** it is a new player action. He decides whether to build it, and how often it should happen; it is something a real runner does.

## 4. Double-offs at 25–50°

They are still 1 per 600 games (MLB about 8 per 600, the fly label). None of the fixes above brings them back:
- **Caution by outs:** applies only to the two-out send, where no runner tags up.
- **The tag-up relook:** took them from 1 to 3 in the probe, but only through the faulty late read of §2, which will be fixed first.
- **The dive back:** lowers them slightly.

The levers remain those in the [fly-read note](2026-11-19-fly-read.md#open) and its review (P3-5): a cut-off for each type of catch, or a slower relook.

## 5. The deferred P3s

| P3 | With the fix? |
|---|---|
| A comment at `awaiting_call`'s no-catch clause (catch-read re-review) | **Yes, with the tag-up relook.** A runner freed from a tag-up while the call is pending is exactly the stranded-runner case that the comment warns about |
| A pending-catch variant of the 3,000-layout property test | **Yes, with the tag-up relook**, for the same reason |
| A test tagging a runner between bases after a non-goal crossing | **Yes, with the dive back**, which changes how a returning runner reaches the base and where he can be tagged |
| P3-1: the third-base route blend turns the lateral velocity in one tick | No. It is cosmetic and moves no race measurably |
| P3-2: the first decision sees one tick less coasting than its clock | No. A 1/60 s bias, which matters only if the read clock changes |

## 6. Plan, ranked

**Build first:**
1. **The late-flight catch read (§2, step 1).** A defect with a clear test, it needs no owner decision, and it blocks the tag-up relook. It should also make the going runners' relooks right. Opus: it touches the projection. It needs its own ledger row.
2. **The tag-up relook (§2, step 2)**, with the two P3s above: about +4 points on second to home with 0–1 out. It brings the sim's behaviour in line with agent-models §6.3's own description of a runner who keeps watching. The spec needs one line saying that it applies to tagging runners too.

**Needs the owner (proposed questions; the Coordinator numbers them):**
- **Two-out sends at the corners.**
  - *Now:* with two out, a runner on second holds at third on about one single in five to left or right, on plays where he would have scored. Big-league runners go on about 92% and are thrown out on 2–6% of those sends. To center both the sim and MLB send 96%.
  - *Options:*
    - (a) The third-base coach's two-out pre-decision: send unless the runner believes he is clearly beaten (`caution_z` set by outs and fitted to these rates).
    - (b) A single `caution_z` fitted to all situations together.
    - (c) Leave it until per-runner caution exists.
  - *Recommended:* (a). It is sourced (Retrosheet 2024 by fielder, the coaching rule), and it does not disturb first to third or center with 0–1 out.
- **A dive back.**
  - *Now:* a runner going back to his base slows to a stop at the bag, so he can be tagged out a step short. On grounders that happens about 90 times in 600 games, against about 3 in MLB, and runners on low liners are doubled off about 3 times as often as in MLB.
  - *Options:*
    - (a) Build a dive or slide back for close returns.
    - (b) Let him run into the bag faster but stay on his feet.
    - (c) Leave it.
  - *Recommended:* (a). It is what real runners do, and it closes most of both gaps. Separately, the runner from third breaking 15 m toward home on a grounder in front of him is worth a look in the same job.

**Leave:**
- P3-1 and P3-2.
- The 25–50° double-offs (§4).
- The trapped runners between third and home (the rundown, Q11).
