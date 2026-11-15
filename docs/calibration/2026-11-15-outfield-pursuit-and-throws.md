# Outfield pursuit and outfield throws: research against the sim (2026-11-15)

Research only. No sim code, no builds, no runs. The sim is described from `main` (`c730555`).
Where `work/live-comm`'s agent-models §5.7 differs, it is said so (§3.6, §6). A play-by-play
diagnosis is a separate job.

**Why.** The owner sees outfielders miss easy-looking flies and sees them throw to the wrong
base or miss the cutoff. 21% of the sim's fly balls fall for hits against about 13% in MLB (the
owner's brief; I did not re-measure either number).

**Evidence labels.** SOURCED: I opened the page and read the passage. SECONDARY: opened, but
the page reports someone else's number, or is a blog with no citation. ESTIMATE: my own
arithmetic or judgment, with the basis given. UNVERIFIED: seen only in a search snippet, or
cited from memory and not opened. No Driveline OpenBiomechanics data is used.

---

## Part 1. Pursuit

### 1. How fielders get to a fly ball: the research

Three families of theory (Fink, Foo & Warren 2009 name them: trajectory prediction TP, optical
acceleration cancellation OAC, linear optical trajectory LOT).

| Theory | Claim | Predicts for the path run |
|---|---|---|
| **TP**, trajectory prediction | The fielder forecasts the landing point from the early flight and runs there. | Direct to the predicted spot; corrected by re-prediction. |
| **OAC**, Chapman 1968 | He runs so the ball's optical acceleration (vertical angle α) stays zero: tan α rises linearly in time. A ball that seems to accelerate will go over his head, one that seems to slow will fall short. | Constant-ish speed, near-straight path; arrives when the ball does because the cancellation is continuous. |
| **LOT**, McBeath, Shaffer & Kaiser 1995 | He picks a path that keeps the ball's image on a straight line, so tan α and tan β grow in proportion. Needs no distance estimate. | A **curved** path ("a sort of crescent-moon arc"), with an N-shaped speed profile; arrives under the ball. |

- McBeath, Shaffer & Kaiser (1995), *How baseball outfielders determine where to run to catch
  fly balls*, Science 268(5210):569–573. SOURCED (abstract and body excerpt read at
  https://bioteach.ubc.ca/TeachingResources/GeneralScience/BaseballPaper.pdf and
  https://www.science.org/doi/10.1126/science.7725104). Two students, tower camera: paths were
  curved, speed varied. LOT is "error-nulling", so it tolerates "perceptual error or changes in
  ball direction due to ... spin, air resistance, and gusts". The same text says LOT solutions
  with non-linear optical speed "typically result in more running path curvature and more
  extreme running accelerations".
- Fink, Foo & Warren (2009), *Catching fly balls in virtual reality: a critical test of the
  outfielder problem*, J. Vision 9(13):14. SOURCED (PMC3816735, https://pmc.ncbi.nlm.nih.gov/articles/PMC3816735/).
  Virtual-reality balls whose vertical motion was altered mid-flight (descent made linear at the
  25 m apogee). Fielders caught perturbed balls as often as normal ones. Their tan α went back to
  a straight line after the change (R² > .99 in most conditions): confirms OAC for the
  front-back movement. The perturbation did not change the side-to-side movement: refutes LOT's
  coupling. Responses were fast and continuous: contrary to a one-shot prediction (TP). All three
  theories "predict successful catches and similar running paths" in normal flight, so path shape
  alone cannot separate them.
- Chapman (1968), *Catching a baseball*, Am. J. Phys. 36:868. UNVERIFIED (cited from memory, not
  opened; described only through the two papers above). McLeod & Dienes (1996), *Do fielders
  know where to go to catch the ball or only how to get there?*, J. Exp. Psych: HPP. UNVERIFIED
  (not opened; Fink et al. cite McLeod, Reed & Dienes 2001 for the ±4 m landing layout).

**What this means for a sim.** Whatever the control law, the fielder is *continuously coupled to
the ball*, his information about where it lands gets sharp late in the flight (the optical angle
is a large, easily read signal near the end), and he arrives about when the ball does. Two
consequences hold under any of the three theories:
1. The effective error of the landing estimate falls steeply in the last second or two.
2. The target is not a fixed spot chosen early; the fielder is steering toward a moving
   estimate, so the *path* bends when the estimate bends.

**What coaches teach** (all SECONDARY: coaching pages, no measurements unless noted).
- *Read, then first step.* Drop step toward the side the ball is hit when it goes over the
  head, crossover step to the side, charge (come-in step) on a ball in front. "Wait a split
  second" is the advanced version; keep weight on the balls of the feet; move toward the
  batter as the pitch arrives. Wilton Little League,
  https://www.wiltonlittleleague.org/coaching-resources/outfield/17280 ; Dugout Edge,
  https://www.dugoutedge.com/coaching-handbook/mechanics/fielding-fly-balls ; Palo Alto girls'
  softball, Kemp, https://www.paloaltogirlssoftball.org/wp/wp-content/uploads/2018/02/PAGS-Fundamentals-of-Outfield-V3.pdf .
- *Run to the spot, then find the ball.* "Drop-and-run: turn and run to the spot... better to
  overrun and come back" (Dugout Edge); "eyes follow the ball's early flight only; shift to
  destination spot" (In The Zone, https://www.inthezonenj.com/free-resources/outfield-play-communication-priority-angles-trust/).
- *Round the ball (the "banana").* With runners on, approach on an angle so you catch with
  momentum toward the throw; "5 to 10 feet behind where the ball will land" (Wilton). One blog
  claims the arc adds "about five feet" and saves time: UNVERIFIED claim, no source
  (https://battingleadoff.com/how-to-play-outfield-in-baseball/); the same page's "0.3 s gain
  from the drop step" is also unsupported. Do not use either number.
- *Pursuit angle.* Take an angle that keeps the body behind the ball (Wilton: "approach the ball
  at an angle that will keep your body behind the ball").
- *Communication priority.* Wording varies: "first call man is the outfielder closest to the
  fielder" (Richter, below); the centre fielder has priority over the corner men is common
  convention. I found no authoritative source for the CF > corner rule in the pages I opened
  (the sim's own `CALL_PRIORITY` says CF > LF, RF): treat as UNVERIFIED convention.

### 2. What Statcast measures, and numbers a sim can be checked against

Definitions (SOURCED, MLB.com glossary, https://www.mlb.com/glossary/statcast/jump and Petriello,
2019-06-10, https://www.mlb.com/news/new-outfield-jump-burst-route-metrics-at-baseball-savant):
- **Jump** = feet covered *in the right direction* in the first 3 s after **pitch release**.
  **Reaction** = feet covered (any direction) in the first 1.5 s; **Burst** = feet in the next
  1.5 s; **Route** = distance covered over 3 s compared with a direct route. All three are
  reported as feet above or below the league average *for that kind of play* (direction,
  wall, hang time). Leaderboards use only plays at 90% catch probability or harder.
- Route efficiency (outfield, older glossary): distance covered / straight-line distance from
  the fielder's position at contact to where he fielded the ball. SOURCED,
  https://www.mlb.com/news/major-league-baseballs-statcast-glossary-of-terms-of-state-of-the-art-tracking-technology/c-118508858 .
  The same glossary lists "First step" (time from contact to first movement toward the ball) but
  no league value is published on the pages I read. I found **no published reaction-time number**:
  Petriello explains Statcast moved to distance "because it's not a lot of fun to say that
  everyone had a first step of 0.2 or 0.3 seconds" (SOURCED, same article). So 0.2–0.3 s is the
  only quoted order of magnitude.
- **Catch probability** from distance needed (optimal route), opportunity time from **pitch
  release**, direction, wall. Five-star = 0–25%, 4-star 30–50%, 3-star 55–75%, 2-star 80–90%,
  1-star 95%; reported in 5% bands since 2019. SOURCED,
  https://www.mlb.com/glossary/statcast/catch-probability . Worked example there: 52 ft in
  3.3 s of opportunity time = 10% for a typical outfielder.

Numbers:

| Quantity | Value | Label |
|---|---|---|
| Example 30% play (Inciarte 2019) | 51 ft needed in 3.4 s opportunity time; he covered **43 ft in the first 3 s** = +8.2 ft above the average for that play; McCutchen on a 54 ft / 3.5 s play was +2.3 | SOURCED (Petriello 2019) |
| Season leader spreads, 2019 | Reaction best +2.9 ft, Burst best +2.1, Route best +1.2; Route range top to bottom ~3 ft vs 5–6 ft for Reaction and Burst | SOURCED (Petriello) |
| Feet covered in 3 s, 2022 | Siri (sprint speed 30.4 ft/s) **38.3 ft**; Vaughn (26.2 ft/s) **29.3 ft** (a 9 ft spread across the best and worst) | SECONDARY (FanGraphs, Andrews 2022, https://blogs.fangraphs.com/learning-from-statcasts-outfield-jump-metrics/) |
| Correlation with OAA per chance (607 seasons) | Burst r .82, Reaction .41, Route -.13, Sprint speed .50 | SECONDARY (same) |
| League-average sprint speed | 27 ft/s (8.2 m/s) used as the default for players with no data | SOURCED (OAA glossary, https://www.mlb.com/glossary/statcast/outs-above-average); "Bolt" = 30+ ft/s, 2018 leader 30.5 | 
| Outfield catch rates | Trout's expected catch % 89 vs Aoki's 77 (2017, the extremes among 150+ chance outfielders) | SOURCED (OAA glossary) |
| Fraction of fly balls that are hits | 13% (owner's brief) | not re-sourced |

What the numbers say for a sim: **range is mostly acceleration after the first 1.5 s, not route
and not first step.** Route is nearly a constant across MLB because "it's very hard to run in the
wrong direction for a sustained time" (Petriello). A good test statistic is therefore
*feet covered in the first 3 s from release, on plays of 90% catch probability or harder*, with
about **29–38 ft** the observed range of single-season means in 2022 (secondary), and
**~43 ft** for one good jump on a 30% play (sourced). I did not find the league mean; do not
assume it.

### 3. What the sim's outfielder does now (main, `c730555`)

Files: `docs/agent-models.md` §5.1–5.4, §5.7, §5.10; `sim/crates/bb_game/src/live.rs`,
`play.rs`, `live/holder.rs`.

1. **Perception** (`read_prediction`, live.rs:841). Four normal draws `fd.err` are fixed per
   fielder per play (live.rs:1340, `predict_inner`). The position error is
   σ = `read_sigma0` × (distance to ball / 30 m) / √(1 + view/τ_read), `view` = seconds since
   contact − reaction. Outfielder `read_sigma0` = 3.5 m, `tau_read` = 0.6 s (`FielderAttrs::average`,
   play.rs:96–110). Depth (along the line of sight) is ×3 (`DEPTH_STRETCH`, live.rs:66). Velocity
   error 4% per axis (`READ_VEL_FRAC` 0.04, live.rs:232). While the ball is on its first flight he
   also flies the velocity read back to the plate and weights it against the direct read
   (`plate_weighing`, live.rs:773). He assumes typical spin for the launch angle, not the true
   spin (`typical_batted_spin_rpm`). **Refresh:** the primary chaser replans every
   `REPLAN_S` = 0.4 s (live.rs:30, :1742–1750; a non-primary chaser the same, live.rs:3487–3498);
   the belief is recomputed from the true ball state with the *same four draws* at the smaller σ,
   so his estimate converges toward truth smoothly rather than jittering.
   *The spec says "at 30 Hz" (agent-models §5.2) and an Ornstein–Uhlenbeck error with τ_c 0.3 s
   (§5.1). The code has neither: 0.4 s replans, frozen draws.*
2. **Target choice** (`intercept_scan`, live.rs:1447). Samples his predicted path every 0.12 s
   (finer when closing) and takes the **earliest** time at which the gap, his shortfall from
   the ball's line given his current velocity (`run_gap`), is ≤ 0 with planning reach
   `REACH × 0.7`; catchable height ≤ ceiling. If none, a dive candidate (`DIVE`, 1.5 m,
   `DIVE_MAX_Z` 1.2 m); if neither, the ball's resting point. Spec: "prefer arriving early and
   settling" (§5.2). The target is a **point**; he is steered at it by `fielder_step`
   (live.rs:1160–1215), so the path is a straight line toward the spot, with curvature only from
   turn limits and re-targeting.
3. **Reaction delay.** Until `t_contact + reaction` (outfield 0.35 s, infield 0.25 s) he moves
   only on his *jump*, if any (live.rs:1160–1167); then he reads the ball. The jump
   (`jump_at_contact`, §5.10) is a lean toward the side he expects from the pitch location and
   the bat's angle at contact. The sim found that the lean is worth about 0.05 runs or less
   (agent-models §5.10) and that leaning wrong costs a stop and turn.
4. **Acceleration and top speed.** v(t) = v_max(1 − e^(−t/τ_a)), outfield `speed` 8.2 m/s,
   τ_a = 1.0 s; braking 6 m/s² (`BRAKE`), lateral acceleration 7 m/s² (`a_lat`). A turn he
   cannot make at speed forces him to slow (`want ≤ √(a_lat·d/(2 sin θ))`). He slows for a spot he
   reaches early; a late chaser runs through (live.rs:1181–1190).
5. **Replans and the route.** Straight line to the current target; no rounding of the ball for
   a throw and no drop step as such. A reversal costs a full stop first (from 7 m/s: 1.17 s and
   4.1 m, agent-models §5.3).
6. **Catch reach and dive.** `POCKET` 0.06 m and `RIM` 0.12 m for the glove at capture, σ_g from
   hands, ball speed, lateness, reach used, dive (agent-models §5.4). Planned reach is
   `REACH × 0.7`. A stretch on his feet adds 0.4 m at a 0.3 s throw cost; a dive up to 1.5 m for a
   ball at or below 1.2 m. An **outfielder dives only when his own estimate gives himself at
   least `OF_DIVE_MIN_P` = 0.5** of the ball, decided once (live.rs:48; §5.3).
7. **Calls and priority** (`call_value`, `assign_among`, `recall`, live.rs:1665 and following; §5.7). On
   a ball he believes he catches in the air with at least `CALL_SLACK_S` 0.3 s to spare, he goes
   for it; he **calls it only if sure**: spare ≥ `CALL_SURE_SIGMAS` 1.0 σ of his own depth error
   run off at his speed (`landing_depth_sigma`, live.rs:796). Priority CF > corners > infield on
   balls in between. **`work/live-comm` §5.7 differs** (`git diff main..work/live-comm`): a
   teammate's chase and call tier become known 0.3 s after it began (`CALL_HEARD_S`,
   `Sim::known`), priority is the table `CALL_PRIORITY` (CF > LF, RF; SS > 2B > 3B > 1B > C > P),
   and cutoffs/backups use `fielding_point` (the man known to be going) instead of the
   chaser's own target. I only read the doc diff, not that branch's code.

### 4. Side by side: pursuit

| Real principle | Sim now | Match? |
|---|---|---|
| Continuous coupling to the ball; late information is sharp (OAC; Fink 2009) | Belief re-read every **0.4 s** from true state; error shrinks as 1/√(1 + view/0.6 s); draws frozen | Partial. Error falls, but slowly, and the *last* correction can come up to 0.4 s stale |
| Path curves; arrival matches the ball's (LOT/OAC) | Straight to an **earliest-arrival point**, then wait | No. Not a flaw in itself for catching; matters for throws (no rounding) |
| Range = acceleration after 1.5 s (Burst r .82), speed 27 ft/s mean | v_max 8.2 m/s (26.9 ft/s), τ_a 1.0 s | Looks fine, see below |
| First step ≈ 0.2–0.3 s | Outfield 0.35 s gate plus a jump lean | Slightly slow at the gate; offset by the lean |
| Drop step, then run; wrong read costs a few steps | A wrong lean costs a full stop and turn at 6 m/s² | Probably harsher than reality |
| Dive when the ball is only just out of reach | An outfielder dives only at ≥ 0.5 of his own estimate | Plausible but untested against catch-probability bands |
| Calls: closest or centre fielder takes it, loudly | Sure-call rule with 1σ, priority table, 0.3 s call delay | Reasonable; interacts with the noisy belief |

**Check on the speed law against Statcast (ESTIMATE).** From contact the sim's average outfielder
covers 8.2·t − 8.2·(1 − e^(−t/1.0)) metres. Statcast's 3 s starts at pitch release; a pitch takes
roughly 0.4–0.45 s to reach the plate (ESTIMATE, pitch flight, not sourced here), leaving about
2.55 s after contact; the sim fielder spends 0.35 s of that before his first step, so his
running time is about 2.2 s: 8.2·2.2 − 8.2·(1 − e^(−2.2)) = 18.0 − 7.3 = **10.7 m = 35 ft**
traveling toward a target. Real 2022 single-season means were 29.3–38.3 ft (secondary). So the
sim's average outfielder covers about what an **elite** real one does (Siri 38.3), before any
anticipation lean. That makes "top speed or acceleration too low" the *least* likely cause
of missed flies; if anything the sim fielder is quick. The caveat: this is a tuned-in-air
ESTIMATE on one assumed pitch flight time.

**Hypotheses for "missed an easy fly"** (each: test from game logs; the log fields would be
the fielder's belief landing point, his intercept time/point, replan times, and the true landing
point; `Fielder.belief`, `t_int`, `target`, `next_plan` exist in live.rs:330 and following).

- **P1. Belief error falls too slowly and is frozen per play.** One σ of depth at first read is
  3.5 m × 3 ≈ 10 m at 30 m of ball distance (live.rs:861–866), then ÷√(1 + view/0.6). For a ball
  in the air 4.5 s, with the fielder 20 m from it at view 2 s the depth σ is still about
  3.5·3·(20/30)/2.08 = **3.4 m**; the glove reaches 0.7 m (planning) to 1.0 m. Real fielders
  nearly null the error in the last second. *Test:* for each caught/dropped fly, |believed −
  true landing| at 1.0 s, 0.5 s before landing, against the final gap; look for dropped balls
  with a late belief error above ~1.5 m. Parameters: `read_sigma0` (play.rs:96), `tau_read`
  (0.6), `DEPTH_STRETCH` (3.0), the frozen draws `fd.err` (live.rs:1340). A real signal that the
  *shape* is wrong: real error should fall faster than 1/√t near the end (optical angles get
  large), while the sim's σ also scales with distance-to-ball, which helps only in the last few
  metres.
- **P2. Replan every 0.4 s, not continuously.** `REPLAN_S` (live.rs:30). In the final second a
  belief change of ~3 m, which can happen between replans, cannot be run off at 0.4 s intervals,
  and a fielder arriving at a spot early "settles" (live.rs:1187–1190) and then re-targets late.
  Spec says 30 Hz. *Test:* gap at landing vs time since the last replan; replay a seed with
  `REPLAN_S` 0.1 in a lab run (not here).
- **P3. Earliest-arrival target wastes the fielder.** He heads for the *earliest* reachable
  point and settles there if early; a later correction then starts from a stop (BRAKE 6, a turn
  from rest is cheap but accelerating again is τ_a 1.0 s). A real fielder arriving just in time
  never stops (OAC). *Test:* fraction of dropped flies whose chaser had zero speed within 1 s
  of landing; time between "arrived" and the ball's landing.
- **P4. Wrong-lean and reversal cost.** The jump lean plus a full-stop reversal (agent-models
  §5.3, 1.17 s, 4.1 m from 7 m/s) is harsher than a drop step and re-acceleration. A fly hit
  over the head after a forward lean is the failure. *Test:* split misses by the jump's direction
  against the true landing direction; Statcast says going back is the hard direction
  (catch-probability adjustment for "going back", MLB glossary).
- **P5. Dive threshold.** `OF_DIVE_MIN_P` 0.5 and a one-time decision: a ball a real fielder
  would dive for at 25–40% p is played on the feet and falls. *Test:* hit fly balls the fielder
  reached within 1.5 m of his glove; compare dive attempts against the catch-probability stars
  (a 2-star ball, 80–90%, should be caught ~85% of the time).
- **P6. Calls.** The sure-call rule needs 1σ of depth spare. If his σ is ~3 m (P1), a
  routine ball between two fielders is not "sure" for either, and each recall may peel one off at
  the last moment. *Test:* flies where two outfielders both had a believed catch and nobody
  caught it; the time of the last peel-off.
- **P7. Speed/acceleration too low:** *unlikely* (see the check above); confirm with
  feet covered in 3 s from release in the logs.
- **P8. Positioning depth** (not pursuit, but 21% vs 13% may start here): starting depth of
  outfielders against the contact distribution; Statcast opportunity time includes the pitch.

---

## Part 2. Throws

### 5. What real outfielders are taught and do

Coaching pages (SECONDARY unless noted; no authority beyond the page):
- *Throw one base ahead* of the lead runner; "outfielders go to two, i.e. +2" with a ball
  cleanly fielded on the ground; lead runner on first → third unless a force at first or second is
  clean; lead runner on second and a shallow ball → home, deep → the cutoff. Kemp (PAGS above,
  citing probaseballinsider.com, which I did not open: that citation is UNVERIFIED) and the
  Jugs-style drill sheet (https://jugssports.com/content/Casey/Outfield%20Play-Individual%20Drills.pdf:
  "ball hit at you: throw two bases ahead; ball hit away from the play side: to 2B (keep the DP
  in order)").
- *Cutoff mechanics.* Throw on a line, chest to head high, "through the cutoff, not to it"; low
  enough to be cut and carry enough to reach the base on a line or one hop if uncut (Richter,
  *Coach's Cheat Sheet: Cut-offs and Relays*, https://wrssba.com/wp-content/uploads/2019/04/Cheat-Sheet-Cutoffs-and-Relays-v-1.6.pdf;
  Wilton: "throw to the cutoff's knee area" is a youth variant). **Who decides to cut:** the
  *infielder* covering the base calls the base number (let it through) or "Cut" plus a number
  (cut it, hold the trailing runner). The *outfielder does not choose to cut*; the relay is
  not a free decision of the thrower (Richter, SOURCED as that cheat sheet). If there is no play
  at any base, the call man yells "Cut" and the relay man runs the ball in.
- *Hit the cutoff when there is no direct play.* Dugout Edge's table: runner tagging → through
  the cutoff to the base ahead; no runners → cutoff; deep fly → cutoff; shallow fly → direct
  throw may be right.
- *Backups.* RF backs first-base throws, CF second, LF third (In The Zone, URL above).
- *Which infielder:* standard assignments are 1B for throws home from right/centre, 3B for left
  field and down the line; SS for throws to third; I did not open a page with a definitive table:
  treat as common convention (the sim's table agrees with it).

Measured rates:
- **Extra bases taken (B-R XBT%)** league 2024 = **42%** (SOURCED,
  https://www.baseball-reference.com/leagues/majors/2024-baserunning-batting.shtml, "League
  Average" row). **First-to-third on a single** from that table's columns: 1stS (opportunity) 8,241
  and 1stS3 (reached third) 2,615 = **31.7%** (ESTIMATE; my reading of the column meanings, the page
  glossary was not opened); the brief's "28–30%" is a little lower. 2011's league XBT% was 41%
  (B-R blog, https://www.baseball-reference.com/blog/archives/10867.html, snippet only: UNVERIFIED).
- **Arm strength** (Statcast, mean of the top 10% of throws): 2025 league averages LF 87.1, CF 89.6,
  RF 90.5 mph (SECONDARY, https://sportsorca.com/mlb/outfield-arm-value-run-prevention/); the sim's
  `ARM_MEAN_MPH` has 87.4 / 89.6 / 90.7 (play.rs:68), consistent.
- **Holds, not just assists.** Statcast's arm value (SOURCED, MLB.com glossary
  https://www.mlb.com/glossary/statcast/baserunning and Petriello 2023,
  https://www.mlb.com/news/most-valuable-outfielder-throwing-arms-of-2023): credits throw-outs,
  **holds** (runners who do not try) and advances. Example: an average outfielder would see a runner
  go 38% of the time; Acuña saw 32% (same article). So **a large part of an arm's value is the
  runner staying put**, not an assist. Over 2022–25 the best hold values quoted are 17–21 runs (secondary).
- **How often throws are cut:** I found no published rate. UNVERIFIED / not found.

### 6. What the sim does now for the outfielder's throw

Files: agent-models §5.5–5.7 (A4 contract); `live/holder.rs`; `live.rs` `holder_act` 2973–3190,
`relay_job` 1765, `update_relay` 1850; `docs/calibration/2026-11-14-throw-choices.md`,
`2026-11-13-throwing.md`, `2026-11-14-arm-by-position.md`.

1. **Options.** For each live runner, the base he is *visibly* heading for (public position and signed
   velocity, `visible_target`, live.rs:592; never his private intention), plus carries.
2. **Choice.** p_out = Φ((runner time − ball time)/0.12 s); keep options with p > 0.3; score
   p²·w, w = 0.9 / 1.0 / 1.1 / 1.3 for first / second / third / home (lead-runner weight; with two
   out 1.0 for a force, 0.9 for a tag, `TWO_OUT_TAG_W`); highest wins (live.rs:3050–3052;
   agent-models §5.7). **There is no value for the trailing runner or batter advancing on the
   throw, no run-expectancy term and no throw-error risk term.** Agent-models §5.6 first describes
   ΔRE and an error term; the A4 contract says explicitly "does not introduce score/run-expectancy or
   throw-error value".
3. **No admitted play.** An outfielder with the ball ≥ `INFIELD_R` (47.4 m) from home throws
   back to **second** by the quicker route, direct or relay (live.rs:3070–3075).
4. **Direct or via the cutoff.** `throw_race` (holder.rs:41): `via` exists only if a Relay-role
   fielder is > 10 m from the holder and closer to the base than the holder; the throw goes
   *through* the relay man iff `via < direct` by clock. So **the cutoff is chosen by arrival time,
   not by whether to hold a trailing runner.** If the throw goes direct, the relay man "pulls up,
   free to back it up" (live.rs:3150s comment: "A throw that goes past the cutoff man ends his
   job"). There is no "cut" decision anywhere: the relay man is a pure relay.
5. **Who the cutoff is** (`relay_job`, live.rs:1765–1830): ball fielded more than 50 m
   (`RELAY_FROM_M`) from home. Target base b = home if a runner is on 2nd or 3rd, third if on 1st
   or the ball is deep (> 80 m), else second; raised to any base a runner visibly heads past.
   Toward home: first baseman (third baseman if the ball is left of x = −15 m); toward third: SS;
   deep (> 80 m): whichever of SS/2B starts closer to the spot. **Toward second with a shallow
   ball nobody is assigned** (`_ => return None`). He lines up on the ball-to-base line, `CUTOFF_M`
   14 m from the base (live.rs:294). `work/live-comm` §5.7 changes who counts as "fielding the ball"
   (`fielding_point`) and waits for a known chaser (0.3 s) before the relay goes.
6. **Accuracy.** Throw direction σ `arm_sigma` 1.2° (play.rs:97); the receiver has a reach of
   `RECEIVE_REACH` 1.5 m (live.rs:176); a throw that goes by is a miss. 
7. **Runner side** (not the fielder's decision but half of the story): runners read the ball and
   `believed_field_time` with `TYPICAL_SPEED` 8.0 and `TYPICAL_ARM` 37 m/s league values (live.rs:69–72),
   not the actual fielder's arm.

### 7. Side by side: throws

| Real principle | Sim now | Match? |
|---|---|---|
| Throw to the base ahead only with a play; else cutoff, holding the trailing runner | Throw for the lead runner whenever p > 0.3 on his own 0.12 s model, weight up to 1.3 for home; no trailing-runner cost | **Differs**; the main source of wrong-base throws |
| Infielder at the base decides *cut or let through* | Cutoff chosen automatically by whichever route is faster | **Differs**; no deliberate cut |
| Throw on a line through the cutoff, chest high, low | Aim at the relay or base; error 1.2° σ; hurried throws are wilder | Partial |
| No play anywhere: hit the cutoff and run it in | No play: throw to **second** (even from a shallow ball) | Roughly the same intent |
| Hold the runner by arm reputation | Runner decisions use typical arm, not the fielder's (live.rs:72) | **Differs** |
| Shallow ball with a runner on second: throw home | Same, if p > 0.3 | Match |
| Back up the base | Backup table exists (§5.7) | Match |

**Hypotheses** (log fields: `choice_trace` JSONL rows from `throw_choice_probe` already carry
options, p, w, chosen base, through/direct times):
- **T1. Lead-runner bias.** p²·w with a floor of 0.3 sends throws at a long-shot home when the
  real play is to cut and hold the batter at first or second. *Test:* of all throws with
  p_out < 0.6, the fraction after which the trailing runner or batter took an extra base,
  against p_out ≥ 0.8; the A4 note's "unweighed_throw" rows give the estimates.
  Parameters/rules: threshold `p > 0.3` (`ChoiceParams`), w table, the missing ΔRE term.
- **T2. No deliberate cut.** The relay man cannot cut a throw the holder chose direct; a good
  direct throw that is late leaves the trailing runner free. *Test:* extra-base events after a
  direct throw with a Relay-role fielder within reach of the throw line; count "ball passed relay
  man" events.
- **T3. σ = 0.12 s on p_out is overconfident.** Real arrival-time uncertainty for a long throw
  plus transfer is several tenths; Φ(Δ/0.12) calls a 0.25 s race 98% certain. *Test:*
  calibration curve: predicted p_out vs realized out rate across throws, in 0.1 bins. Parameter:
  the 0.12 in `p_out` (and in `force_continuation`).
- **T4. No cutoff for a shallow ball toward second.** `relay_job` returns `None` for b = 2
  below 80 m; extra base to third is then unguarded by a relay. *Test:* extra bases on
  singles to the outfield, split by whether a Relay role existed.
- **T5. Arm not used by runners.** Runners estimate with `TYPICAL_ARM`, so no hold value: first-to-third rates
  and outfield assists do not respond to arm. *Test:* first-to-third by arm grade at fixed distances;
  compare to the real 31.7% (2024, estimate) and to the 38% → 32% effect of an elite arm.
- **T6. Cutoff man position.** Spot is `CUTOFF_M` 14 m from the base on the line; a relay man
  reaching late or the throw's σ makes "missed cutoff" a geometry event. *Test:* lateral miss at
  the relay man in the log (aim minus actual at his position).

---

## Conclusion for the lead (ranked)

1. **Landing-point belief converges too slowly and is stale between replans (P1 + P2).**
   Confirm: for every fly, log |belief − true landing| at 2.0 / 1.0 / 0.5 s before landing and the
   time since the fielder's last replan, then compare dropped and caught balls; the signal is
   dropped balls with ≥ 1.5 m error at 1 s or a gap that grows in the last replan interval.
   Touches `read_sigma0` (3.5 m), `tau_read` (0.6 s), `DEPTH_STRETCH` (3), frozen `fd.err`,
   `REPLAN_S` (0.4 s vs spec 30 Hz), `READ_VEL_FRAC`. The main reason I rank it first: the
   science (Fink 2009) says error is cancelled continuously; the sim's is corrected 0.4 s at a time
   from a frozen draw.
2. **Early-arrival target plus stop-and-turn cost (P3 + P4).** Confirm: dropped flies whose
   chaser had speed < 1 m/s in the last second, and misses split by whether the first lean
   matched the true direction (going back worse). Touches `intercept_scan` ("earliest
   reachable"), `BRAKE` (6), `a_lat` (7), `settle` rule, `JUMP_*` constants.
3. **Throw choice ignores the trailing runner and has no deliberate cut (T1 + T2 + T3).**
   Confirm: extra-base-after-throw rate by p_out bin, and the p_out calibration curve (predicted
   vs realized). Touches `p > 0.3`, w table (0.9/1.0/1.1/1.3), the 0.12 s in Φ, the unbuilt ΔRE
   term of agent-models §5.6, `throw_race`'s `through` rule, `relay_job`'s "no relay toward second".
   Real basis: the call to cut belongs to the infielder at the base (Richter), not the thrower.
4. **Dive/stretch threshold and the call rule (P5 + P6).** Confirm: for balls ending within 1.5 m of
   the glove, dive attempts vs catches, and flies where two outfielders both had a believed catch.
   Touches `OF_DIVE_MIN_P` (0.5), `CALL_SLACK_S` 0.3 s, `CALL_SURE_SIGMAS` 1.0, and on
   `work/live-comm` `CALL_HEARD_S`/`CALL_PRIORITY`.

Not a likely cause: top speed or acceleration (the sim's average outfielder covers ~35 ft in
Statcast's 3 s window, about an elite real one, ESTIMATE above).

**Not found or not opened:** a published Statcast reaction-time number; the league mean of
feet-covered-in-3 s; route-efficiency league mean; a rate of cut throws; Chapman 1968 and
McLeod & Dienes 1996 (cited second-hand only).
