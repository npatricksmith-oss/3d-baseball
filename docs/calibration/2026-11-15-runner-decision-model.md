# The runner's decision model: sources, run-value arithmetic and a design proposal (2026-11-15)

Research and design only. No sim code was changed, built or run for this note. Every number is tagged:
**SOURCED** (the page was opened and read; the arithmetic, if any, is mine and shown),
**CARRIED** (read and recorded in an earlier note of this repository; the earlier note is named),
**ESTIMATE** (my inference or a judgment, stated as such), **UNVERIFIED** (seen only in a search extract or a page I could not open; carries no conclusion).
Pages that load their data by script (Baseball Savant leaderboards, the interactive part of Malter Analytics) could not be read; where that leaves a gap it is listed in §1.11.

## Summary

1. **What real runners do** (B-R 2024, league totals, SOURCED): a runner on first goes to third on a single **31.7 %** of the time (the sim: 28.7 %), a runner on second scores on a single **61.7 %** (sim 34.9 %), a runner on first scores on a double **38.1 %** (sim 54.1 %). Runners put out trying are **1.1 %, 3.3 % and 2.5 %** of those opportunities. When they try, they are safe **95–97 %** of the time. The brief's "first to third about 28 %" is not what the 2024 table gives; 31.7 % is the figure to test against.
2. **Real runners are more careful than the run-expectancy break-even says they should be.** Break-evens from the sourced 2021–24 table: 1st→3rd on a single 74–76 % (0–1 out) and 90 % (2 out); 2nd→home 94 % / 73 % / 41 % (0 / 1 / 2 out); tag from third 75 % (0 out) and 34 % (1 out). Observed success when sent is 94–97 % in every case. So the Q2 rule (go when p(safe) beats break-even) is the right *shape*, but run alone with an unbiased belief it would send far more runners at 1 and 2 out than MLB does, and fewer at 0 out. The model needs a named risk margin, calibrated to the observed rates (owner question Q1 below).
3. **The sim's largest misses probably share a cause** (mechanism read from the code, not yet traced play by play): the runner assumes the defense throws to the base *he* is running to (`ball_time_to`, live.rs:3404). A real runner reads the throw the defense should make from the situation. With a man on second, a single to center goes home, and the trailing runner is not the target at all. That prior (owner, 2026-11-15) is what the proposal builds first.
4. **No third-base coach exists in the sim**, and no rundown. Both are costed in §3(e)–(f); a staged build is recommended.
5. **Rundown outcome rates are not published** that I could find. The proposal tests a rundown for plausibility only.

---

# Part 1: Sources

## 1.1 Extra bases taken, from Baseball-Reference's league baserunning table

[Baseball-Reference, "2024 Major League Baseball Baserunning/Misc"](https://www.baseball-reference.com/leagues/majors/2024-baserunning-batting.shtml): SOURCED, the whole table read, league-total row. The page also gives the 2026 partial season ([2026 table](https://www.baseball-reference.com/leagues/majors/2026-baserunning-batting.shtml), search extract of the whole table, used only as a check). Column meanings: XBT% is "the percentage of time that a baserunner advances more than 1 base on a single or more than 2 bases on a double" ([B-R blog, Andy, 27 Apr 2011](https://www.baseball-reference.com/blog/archives/10867.html), SOURCED). `1stS` = opportunities (runner on first, single), `1stS2` = stopped at second, `1stS3` = reached third; `1stD`, `1stD3`, `1stDH` the same for a double (stopped at third, scored); `2ndS`, `2ndS3`, `2ndSH` for a runner on second on a single. The page does not print the column definitions; this is my reading of the headers and of the 2011 blog table. Denominator details (whether infield singles, or singles with a runner also on second, are included) are **not** confirmed: ESTIMATE.

League totals, 2024 (SOURCED) and the rates I derive:

| Situation | Opportunities | Stopped at the next base | Took the extra base | Neither (my reading: thrown out going) | Rate taken | Out going / opportunity | Attempts that were safe |
|---|---:|---:|---:|---:|---:|---:|---:|
| 1st→3rd on a single (`1stS`) | 8,241 | 5,534 (to 2nd) | 2,615 | 92 | **31.7 %** | 1.1 % | 96.6 % |
| 1st→home on a double (`1stD`) | 2,455 | 1,457 (at 3rd) | 936 | 62 | **38.1 %** | 2.5 % | 93.8 % |
| 2nd→home on a single (`2ndS`) | 4,881 | 1,711 (at 3rd) | 3,010 | 160 | **61.7 %** | 3.3 % | 95.0 % |

"Neither" is the column that does not add up. The B-R blog's own commenters read the gap the same way: "when they don't add up it means runner(s) were lost trying to take the extra base" (comment 17 on the XBT post, SOURCED as a reader's statement, not B-R's). A cross-check from an independent source agrees to within about one point (§1.2). The 2026 partial season gives 31.8 % (2,728 / 8,565), 38.3 % (913 / 2,382) and 60.9 % (3,087 / 5,067): no drift.

League XBT% is 42 % in 2024 (SOURCED; 2026 partial 42 %); team range 36–49 %.

**Outs on the bases (OOB).** League total 1,302 in 2024 (SOURCED) = 0.268 per team-game (30 teams × 162 = 4,860 team-games; my division). B-R's OOB is the set of "other events when a runner is called out on the bases": out advancing on a fly ball, trying for the extra base on a hit, advancing on a wild pitch ([Fire Jim Bowden blog, 25 Jul 2010](http://firejimbowden.blogspot.com/2010/07/out-rate-new-measure-of-offensive-value.html), search extract of the page: UNVERIFIED as B-R's own definition; the blog says it uses B-R's data). The table splits OOB by base: OOB1 178, OOB2 406, OOB3 297, OOBHm 421 (total 1,302, SOURCED). Three situations above account for 92 + 62 + 160 = **314 outs = 0.065 per team-game**; the rest of the 0.268 is other plays (tag-ups, doubled off, batters stretching, the bases-loaded and third-base plays). So the brief's "outs on the bases after a hit about 0.1 a team-game" is consistent with this table (ESTIMATE: 0.065 for these three situations plus the other hit plays), and the sim's 0.23–0.27 should be compared on the **same classification**, not against the 0.268 total of all OOB. (If the sim's figure is all outs on the bases by any cause, 0.27 is MLB's own number.) Splitting the sim's outs by cause is the first thing to do before tuning (§4, test T4).

## 1.2 Cross-check and further splits: Malter Analytics

[Malter Analytics, "When to Send the Runner" (11 Aug 2026)](https://www.malteranalytics.com/blog/2026-08-11-baserunning-aggression-re24): SOURCED, blog read in full; it is a small commercial analytics site with its own play-by-play, 2021–2026 regular seasons, so treat as secondary. Method is the standard break-even formula (§1.5).

| Transition | Opportunities | Attempt % | Success % (of attempts) |
|---|---:|---:|---:|
| 3rd→home, tag-up | 8,986 | 81.6 | 97.5 |
| 2nd→home on a single | 28,376 | 64.0 | 96.7 |
| 1st→home on a double | 14,239 | 42.3 | 93.8 |
| 1st→3rd on a single | 47,292 | 32.4 | 96.2 |
| 2nd→3rd, tag-up | 18,750 | 35.4 | 96.3 |

Agreement with the B-R derivation: 1st→3rd attempt (2,615 + 92) / 8,241 = 32.8 % and 96.6 % safe; 2nd→home (3,010 + 160) / 4,881 = 64.9 % and 95.0 %; 1st→home on a double (936 + 62) / 2,455 = 40.7 % and 93.8 % (identical). I take the "neither = thrown out" reading as confirmed to about a point.

Other numbers from the same post (SOURCED): with two outs and a runner on second on a single, runners go **74.6–90.7 %** of the time depending on the direction of the hit; the break-even is 41.3 %. Tag-up from third on a shallow fly to right with one out: attempt rate 50.2 %, safe 88.6 % when they go, against a 33.4 % break-even. Tag-up from second to third on a shallow fly with one out has a break-even of 88.6–89.7 % and observed success of 48.7 % (shallow left, 661 plays) and 75.9 % (shallow center, 404 plays), "attempt rates of 6–11 %". The post's interactive tables (by direction, distance, outs, trailing runner) did not load (page says "Unable to load data"), so the by-outs and by-location rates are a gap (§1.11).

**Stolen-base break-evens, for comparison** (same post, SOURCED): runner on first, steal second 73.9 / 73.4 / 69.9 % (0 / 1 / 2 out); runner on second, steal third 77.7 / 67.8 / 88.6 %; double steal 62.0 / 56.5 / 77.4 %. Bill James's old rule of thumb, "a steal must succeed about two-thirds of the time" ([Whitney-Epstein, Sissman and Dodson, Wharton, 31 Oct 2025](https://wsb.wharton.upenn.edu/wp-content/uploads/2025/12/WHITNEY-EPSTEIN_Lead_Distance_Optimization.pdf), SOURCED, a student paper, intro read), matches.

## 1.3 Tag-ups and sacrifice flies

CARRIED from [2026-11-14-tag-ups.md](2026-11-14-tag-ups.md) §2, §7 (each read there; sources named there): with a runner on third and fewer than two out, caught outfield flies end 75–80 % scoring, 17–23 % held, 2–3.5 % doubled off; success when sent 94–97 %; league send rate about 81–84 %; flies of 225–275 ft: sent 41.4 %, safe 90.2 % (n = 3,174, FanGraphs Community, Olszewski, 2021), 250–275 ft sent about half, 275–300 ft about 90 %, over 350 ft always; centre field holds 12 % against 20 % for a corner (THT, Fox, 2006). Sim now (note §6.4 measured): scored 53.9 %, put out 1.8 %, sacrifice flies 0.205 a team-game against 0.24–0.27; between 225 and 300 ft runners almost always hold.

## 1.4 Outfield arms and the "hold"

- [MLB.com glossary, Baserunning/Throwing Value](https://www.mlb.com/glossary/statcast/baserunning) (SOURCED): Statcast gives each extra-base chance a success probability from runner speed, outfielder arm, runner position on the base path, and the outfielder's distance from the ball and the bases; a runner who does not go is a "hold" and credits the arm. Acuña's throw-out of a stretch attempt carried a 5 % pre-play chance. Taylor's 2022: +4 runs for throw-outs, +6 for holds, −5 for advances allowed.
- [Petriello, MLB.com, 14 Jul 2023](https://www.mlb.com/news/most-valuable-outfielder-throwing-arms-of-2023) (SOURCED): on comparable chances an average outfielder sees the runner advance 38 % of the time; against Acuña (arm 96.7 mph) 32 %. Gavin Sheets: expected 36 %, actual 43 %. Same 269-ft ball: Henderson held against Acuña, Edman scored on Melendez (92.4 mph, same distance). The arm is **known** to the runner by reputation; this is the "scouting knows an arm" assumption the sim already makes (`ARM_READ_SIGMA_MPS`).
- [Tango, "Statcast Lab: Batter-Runner v Outfielder, Play at 2B, part 2", 26 Oct 2020](https://tangotiger.com/index.php/site/comments/statcast-lab-batter-runner-v-outfielder-play-at-2b-part-2) (SOURCED): a batter's single can be placed in the plane of (fielder-to-second distance against runner-to-second time). Above a "no-go line" even the weakest arm throws him out and over 99 % hold; below a "go line" even the strongest arm cannot and over 99 % go. In the zone between, the league average of runners who go is **35 %**, and arms differ: Jackie Bradley Jr. 2 of 13 (15 %), Merrifield 0 of 11, Dickerson's runners went every time. This is the shape the decision model must produce: a wide band where the outcome depends on a belief about a near-even race.
- Aaron Judge: runners went 33 % where 38 % was expected, 2022–25 ([SportsOrca, 18 Apr 2026](https://sportsorca.com/mlb/outfield-arm-value-run-prevention/), SOURCED but a secondary summary of Statcast/FanGraphs).
- Arm strength, league means 87.1 / 89.6 / 90.5 mph (LF / CF / RF, 2025), CARRIED from the tag-ups note §3.

## 1.5 Run-expectancy arithmetic

**The table.** [Clemens, "The Run Expectancy Matrix, Reloaded for the 2020s", FanGraphs, 29 May 2025](https://blogs.fangraphs.com/the-run-expectancy-matrix-reloaded-for-the-2020s/): SOURCED, read in full; first eight innings only, 2021–2024 regular seasons (2020 excluded), 2025 partial. The 2021–24 average (runners on, 0 / 1 / 2 outs):

| Runners | 0 out | 1 out | 2 out |
|---|---:|---:|---:|
| none | 0.50 | 0.27 | 0.10 |
| 1– | 0.90 | 0.54 | 0.23 |
| –2– | 1.14 | 0.71 | 0.33 |
| ––3 | 1.37 | 0.98 | 0.38 |
| 12– | 1.51 | 0.94 | 0.46 |
| 1–3 | 1.82 | 1.19 | 0.51 |
| –23 | 2.04 | 1.41 | 0.57 |
| 123 | 2.38 | 1.63 | 0.82 |

The same page gives each of 2021, 2022, 2023, 2024 and 2015–19 and 2000–04 separately; the 2015–19 and 2021–24 tables differ by at most 0.09 (bases loaded, 0 out) and mostly by 0.00–0.02, so the table can be treated as stable. An independent derivation of 2024 (`mlb_re24_matrix_2024.csv`, the sportsdataverse "mlb_game_state" release, downloaded and read; unknown reliability, used only to check) reproduces the FanGraphs 2024 row to within 0.05 (for example none-on 0.495 / 0.264, runner on first 0.896 / 0.531, 1–3 2 out 0.511). Older tables used in the sources below (0.52 / 0.28 / 0.11 for none-on, 2004; the 2009 numbers in Carleton) differ by about 0.02.

**The break-even.** For each decision three values exist: hold, go-and-safe, go-and-out. The runner should go if p(safe) exceeds

  `be = (RE_hold − RE_out) / (RE_safe − RE_out)`

with a run scored added to the state's value (this is the formula in Malter, Olszewski, Carleton, DraysBay and SkyKing162 below). My computation from the 2021–24 table above (script in §8), with the failure state defined as: runner out, batter-runner stays where the single left him (column A); and, for the three hit plays, a second column where the throw lets the batter take second whether the runner is safe or out (column B, because a throw to the front base is the usual way a batter advances):

| Advance | Outs | Hold state | Safe state | Out state | Break-even A | Break-even B |
|---|---:|---|---|---|---:|---:|
| 1st→3rd on a single | 0 | 12– | 1–3 | batter on 1st (A) / on 2nd (B) | 75.8 % | 60.2 % |
| | 1 | | | | 74.0 % | 56.5 % |
| | 2 | | | | 90.2 % | 80.7 % |
| 2nd→home on a single | 0 | 1–3 | run + batter on 1st | batter on 1st (A) / 2nd (B) | 94.1 % | 77.6 % |
| | 1 | | | | 73.3 % | 62.3 % |
| | 2 | | | | 41.5 % | 38.3 % |
| 1st→home on a double | 0 / 1 / 2 | –23 | run + –2– | –2– | 93.0 / 78.3 / 42.9 % | not computed |
| batter stretches a single to a double, bases empty | 0 / 1 / 2 | 1– | –2– | none, one more out | 72.4 / 72.1 / 69.7 % | n/a |
| tag-up 3rd→home, alone (outs before the fly) | 0 / 1 | ––3 | run | double play | 75.2 / 34.5 % | n/a |
| tag-up 2nd→3rd, alone | 0 / 1 | –2– | ––3 | double play | 69.3 / 86.8 % | n/a |
| tag-up 1st→2nd, alone | 0 / 1 | 1– | –2– | double play | 72.1 / 69.7 % | n/a |

Agreement with published values (all SOURCED, each with a different year's table, so a point or two is expected): tag from third alone, 0 out 75.1 %, 1 out 32.5 % ([Olszewski, FanGraphs Community, 1 Oct 2021](https://community.fangraphs.com/are-third-base-coaches-too-hesitant-in-sacrifice-fly-situations/), search extract of the table, also read in the tag-ups note); the same, 73.2 % at no out (Carleton, [ESPN, 17 Feb 2010](https://www.espn.co.uk/mlb/hotstove09/insider/news/story?id=4920847), extract: UNVERIFIED beyond that number); 2nd→home on a single with two out 41.3 % and the tag from third with one out 33.4 % (Malter, read); 2nd→3rd tag with one out 88.6–89.7 % (Malter; mine 86.8 %); 1st→3rd on a single with two out "92 %" ([Baseball Musings quoting Roley, Baseball Analysts, 2007](https://www.baseballmusings.com/archives/023933.php), extract: UNVERIFIED; mine 90.2 %). The structure of the answer is stable: **0 out is "go only if sure" at the home plate (93–94 %), 1 out is about three in four, 2 out is "go on about two in five" except at third base, where 90 % is needed because the gain is small.**

**What the sourced behaviour says about the break-even.** Success when sent is 94–97 % at *every* outs state (§§1.1–1.3), far above the break-even at 1 and 2 outs (34–78 %). Carleton's own conclusion in 2010: "The 'break-even' point is usually around 75 percent, but actual success rates are well above 90 percent" and the average team leaves "about four runs on the table per season on potential sac flies alone" (extract; UNVERIFIED as a number, quoted for the direction, which the Olszewski table (Safe 90.2 % against break-evens 32.5–77.5 %) confirms). The 3rd-base coach (human) is a conservative decision-maker relative to the run table, and a model that matches *what MLB does* needs a margin above break-even or a belief noisier than the truth.

**Score and inning.** Run expectancy treats every run alike; the win value of the run does not. [Silver, "Introducing ORVY", Baseball Prospectus, 4 May 2005](https://www.baseballprospectus.com/news/article/4003/lies-damned-lies-introducing-orvy/) (SOURCED, read; a model on a historical run distribution, so an approximation): after the top of the 7th the marginal win probability of one more home run is **24.1 %** when the home team is tied or down one, 11.9 % when up one or down two, 6.6 % up two or down three, 3.6 % up three or down four, 1.9 % up four or down five. So the first run when tied or one behind is worth twice the second, and a run when four behind is worth 8 % of a tying run. His ORVY (first run's value over second's) is 1.09 for a tied game in the bottom of the first and 3.64 in the bottom of the eighth (extract of the same page: UNVERIFIED). The consequence for the break-even: late and close, the run is worth more than the run-expectancy table says, so the break-even **falls**, and the out costs less in comparison (the best case is the tying or winning run). Down by many runs, one run is worth little and the break-even **rises past 100 %**: do not risk the out. My attempt to get these numbers from a 2024 win-expectancy table (`mlb_we_table_2024.csv`, sportsdataverse, downloaded) failed: the cells are too sparse (a handful of plate appearances each; it returned negative and >100 % break-evens), so no number from it is used. Win-expectancy tables are what agent-models §8 plans to learn (`WE`); until then a one-run weight table like Silver's is the interim.

## 1.6 What coaching says a runner knows and looks at

These are coaching conventions, not measurements. Pages are youth or amateur instruction (sources opened and read unless marked); the professional rule is the same in kind.

- **Before the pitch:** outs, count, score, inning, who is on, where the defense plays. "The 1st and 3rd base coach should know the count and number of outs ... Communicate this constantly with base runners" ([Base running Fundamentals and Coaching Tips, Sportngin PDF, undated](https://cdn1.sportngin.com/attachments/document/28c5-2360751/Base_running_Fundamentals_and_Coaching_Tips.pdf), search extract: UNVERIFIED), "Knows the score, inning, outs and count at all times" ([Coach and Play Baseball checklist](https://www.coachandplaybaseball.com/baserunning.html), extract: UNVERIFIED).
- **Read the ball yourself in front, the coach behind** ([Wilton Little League, Baserunning clinic](https://www.wiltonlittleleague.org/coaching-resources/base-running/17285), SOURCED): "use his own judgment when the ball is put in play in front of him ... and ... rely on the third base coach when the ball is put in play behind him"; pick up the third-base coach halfway between first and second; after rounding, square the shoulders to the ball. The Sportngin sheet says the same for first base: "ball in front of you, your decision. Directly at an OF, get to 2B. If the ball is hit behind you, pick up your 3B coach" (extract: UNVERIFIED).
- **The coach's job** ([Guilliams, Base-Coaching Absolutes, Human Kinetics via Coaches Insider, 2014](https://coachesinsider.com/baseball/base-coaching-absolutes-todd-guilliams-article/), SOURCED): runners to second must pick up the third-base coach halfway on balls to right field; the "coach–base–coach" rule (look at the coach halfway, look down to touch the base, look at the coach again after rounding); the coach "is a stop sign only. He is not a go sign" to remove hesitation; with two out the coach decides before the pitch to send the runner from second home on a single and warns the first-base coach so the trail runner does not hesitate; the coach's position (behind third toward the outfield = "you are scoring", down the line = "watch my hands").
- **Ground balls:** with two out every runner goes on contact; with a runner on first behind him the runner on second is forced and goes; with fewer than two out and first base open, "ball in front, hold; ball behind, go" ([Marietta Baseball Academy, 30 Oct 2026](https://mbaseballacademy.com/runner-on-second-ground-ball/), SOURCED: a youth guide on 60-ft bases). From the same page: a ground ball up the middle is read, and the runner keeps his eyes on the ball until it is cleanly fielded. A runner on third with fewer than two out goes if the ball goes to the shortstop or second baseman unless the infield is in (Wilton, search extract of the same page: UNVERIFIED).
- **Line drives:** "Line Drives – Freeze" with fewer than two out, to avoid being doubled off ([Sportngin sheet](https://cdn1.sportngin.com/attachments/document/28c5-2360751/Base_running_Fundamentals_and_Coaching_Tips.pdf); [NFCA drill page](https://nfca.org/item/live-batting-practice-bunting-baserunning), "on a line drive freeze and get back"; both extracts: UNVERIFIED). Guilliams in the other direction: on a steal or hit-and-run "the runner should be taught not to try to get back on line drives ... he cannot get back anyway" (SOURCED): the rule is decided by whether he is already moving.
- **Fly balls:** with fewer than two out go partway, "with the lead lengthening if the ball is hit to center field or left field", then tag or run depending on the catch; with a runner on third return and tag, taking instruction from the coach (Wilton, search extract of the same page: UNVERIFIED). Two out: run on contact.
- **When he commits.** The only quantified account I found is Statcast's own: lead distance is measured when "the pitcher makes his first movement" ([MLB.com glossary, Lead Distance](https://www.mlb.com/glossary/statcast/lead-distance), extract: UNVERIFIED beyond the definition), secondary lead at release ([Statcast glossary of terms, MLB.com, 15 Apr 2015](https://www.mlb.com/news/major-league-baseballs-statcast-glossary-of-terms-of-state-of-the-art-tracking-technology/c-118508858), extract, UNVERIFIED), first step as "the time elapsed from time of bat-on-ball contact to the runner's first movement toward next base", and "extra bases": contact to the point the runner advances an extra base. **No value** for the first-step time or for the point of no return was found. ESTIMATE: the point of no return is physical, the place where his stopping distance reaches the base; the sim already computes it (`run_time_to_stop`, `settle_guard`).
- **Fielder tells and arm:** the runner uses the outfielder's speed to the ball, his arm, and "whether he's pacing on a diagonal ... and has to plant and throw off his back foot ... or ... circle it, set his feet, crow-hop, and fire" ([Rivera, FanGraphs, 11 Jul 2024](https://blogs.fangraphs.com/baserunning-is-all-about-taking-calculated-risks/), extract: UNVERIFIED but descriptive).

## 1.7 Leads

- Definitions (UNVERIFIED, extracts): lead distance is the distance from the base to the runner's centre of mass at the pitcher's first movement; the secondary lead is the same at release ([MLB.com, Statcast glossary of terms, 2015](https://www.mlb.com/news/major-league-baseballs-statcast-glossary-of-terms-of-state-of-the-art-tracking-technology/c-118508858); [MLB.com glossary, Lead Distance](https://www.mlb.com/glossary/statcast/lead-distance)).
- Numbers: on steal attempts the runner covers "11.5 feet" on average between the pitcher's first move and release, Volpe 15.1 and Elly De La Cruz 8.9 ([Adler, MLB.com, 16 Dec 2024](https://www.mlb.com/news/breaking-down-statcast-s-new-baserunning-stats); the page was opened but the extract of that figure came from the search result: UNVERIFIED). Primary lead of one runner on a named play 11.46 ft; the model's optimal lead for that runner 10.4 ft ([Whitney-Epstein et al., 2025](https://wsb.wharton.upenn.edu/wp-content/uploads/2025/12/WHITNEY-EPSTEIN_Lead_Distance_Optimization.pdf), SOURCED for those two numbers; the paper does not give a league mean in the part I read). A page claiming league means of 11–12 ft (RHP) and 9–10 ft (LHP) at first and 18–20 ft at second ([Legends Deck, 2026](https://legendsdeck.com/blog/what-is-lead-distance)) is a low-quality aggregator: UNVERIFIED, no conclusion.
- The sim's leads at contact are `LEAD_M` = 5.5 / 6.5 / 4.5 m = **18 / 21 / 15 ft** at first / second / third (live.rs:214). Against a primary lead of about 10–12 ft at first plus a shuffle of a few feet that is probably 1–2 m too long at first; the data are missing to say more. Every metre of lead is about 0.12 s of race (at 8.2 m/s): a lead error is as large as the whole 0.15 s timing noise the runner has now. Not part of this proposal; flagged for a sourcing job (§6, first item).

## 1.8 Rundowns

I found **no published rate** for how rundowns end (searches of Retrosheet, SABR, FanGraphs and Baseball Prospectus returned only coaching and anecdote). What exists:
- [Baseball-Reference Bullpen, Run down](https://www.baseball-reference.com/bullpen/Run_down) (extract: UNVERIFIED): the runner "has little chance to escape safely unless there is an error"; the defense chases him back to the base he left.
- Coaching standard: get the out in two throws or fewer, ideally zero or one; run the runner back toward the base he came from; the holder runs at the runner so the runner commits ([Batting Leadoff, 23 and 26 Sep 2026](https://battingleadoff.com/rundown-execution-pickle-two-throws/), extract: UNVERIFIED; NFCA Tip Tuesday, "Get to the Chase", extract: UNVERIFIED; a fielder crossing the baseline commits obstruction ([Keep Playing Baseball](https://keepplayingbaseball.org/the-rundown/), extract: UNVERIFIED)).
- A Quora guess "over 99 %" end in an out: UNVERIFIED, mentioned only to say there is no better figure.
- Run values of a rundown: one out against a safe return is the 0.6–0.9 run swing from the table in §1.5 (my arithmetic: runner on first, 0 out, 0.90 → bases empty 1 out, 0.27, is −0.63 for the out; runner on second 1.14 for the runner safe on second).

## 1.9 Third-base coach send decisions

CARRIED from [2026-11-14-tag-ups.md](2026-11-14-tag-ups.md) §2 (each read there): coaches send on a sacrifice fly 81–84 % of the time (THT, Fox), a send is safe 94–98 %; at 250–275 ft the coach "throws up the stop sign almost half the time, despite a five-in-six chance of scoring" (BP, Dubuque, 2025); success falls with slower runners "less steep than one might imagine" (Olszewski). Plus SOURCED here: Guilliams (§1.6), and the Carleton/Olszewski conclusion that the human coach sends only near-certainties (§1.5). Statcast's runner success probability is modelled from runner speed, outfielder arm, runner position and the outfielder's distance from ball and bases (glossary, §1.4): those four are the inputs the coach and runner act on.

## 1.10 Ground ball, force removed, "going on contact with two out"

Coaching rule only (§1.6): with two out every runner goes on contact; with a runner behind him the runner is forced. No rate was found for how often runners on third score on grounders by outs and infield depth; B-R's own method list ([Position Player WAR, Baseball-Reference](https://www.baseball-reference.com/about/war_explained_position.shtml), search extract: UNVERIFIED) shows B-R tracks "runner advances on infield ground balls" for a runner on second with first open, as an opportunity type, so the data exist in play-by-play but not in a page I could read.

## 1.11 Gaps (what a model should be tested against but I could not source)

1. Attempt and success rates **by outs, by direction and distance of the hit** (Malter's interactive tables, Savant's Extra Bases Taken leaderboard). A worker with Retrosheet event files could compute them from first principles in a day; I did not. This is the table T2 in §4 and the largest gap.
2. League mean primary and secondary leads (Savant's leaderboards load by script).
3. Catch-to-first-step time for a runner reading a ball, and the reaction time of the third-base coach.
4. Rundown outcomes; speed of the throw to the cutoff (carried: no outfielder exchange time published).
5. Win-expectancy-based send decisions by inning and score.

---

# Part 2: What the sim does now, and a design proposal

## 2. What the sim does now (file and line, `sim/crates/bb_game/src/live.rs`, main at 1a06d22)

| Piece | Where | What it does |
|---|---|---|
| First look | 4628–4660 | At contact + 0.5 s every runner decides once: a batter goes to first; a runner who believes the ball will be caught (`runner_predict`, `believed_field_time` ≤ landing + 0.3 s) tags up; on a deep fly that may drop with fewer than two out he goes `HALFWAY_M` = 5 m (line 96) past his lead and waits; otherwise `runner_decide`. |
| `runner_decide` | 3506–3545 | One base at a time: `go` iff `t_ball − (t + t_run) + noise > 0.35` (3533), `t_run` the runner's run time to the next base at his own `vmax` and `RUNNER_TAU` (×0.75 if already moving), `t_ball = ball_time_to(next) + typical_tag_s` (unforced), noise `0.15 · truncated normal(4)` per decision, drawn from `Purpose::RunnerPerception` at sub-index 2n. A forced runner at his own base always goes (`must`). The 0.35 s margin is 2.3σ of the noise: roughly a 99 % rule, as the brief says. No outs, score or run value enters. |
| `ball_time_to` | 3404–3459 | The runner's belief of the ball's arrival at base `b`. Held ball: his read of the holder (position's grade-50 transfer, own arm ± `ARM_READ_SIGMA_MPS` = 1.1 m/s, set-up and turn) then `read_throw_time` straight to `b`. Free ball: `believed_field_time` + `TYPICAL_TRANSFER` 0.75 s + `typical_throw_time` (a typical arm, relayed if quicker) to `b`. Throw in the air: `thrown_read`. **In the first two branches the throw is assumed to go to the base the runner is running to.** |
| `thrown_read` | 3367–3402 | Predicts a throw's flight from its observed position and velocity with a typical backspin (`THROW_SPIN_RPM` 1500) and reference drag; takes the base nearest where it comes down at `RUNNER_READ_CATCH_Z` 1.3 m (one hop if it lands short); any base passed within `PASS_REACH_M` 2.5 m at under `PASS_MAX_Z` 3 m can be caught there on the way (`pass_time`, `PASS_CATCH_S` 0.1 s). A note at line 3363 names this function as the place a situational prior would feed. |
| `runner_relook` | 3677–3745 | A runner between bases decides again when a ball is secured or a throw released: time to the next base and back (`run_time_from`, `run_time_to_stop`, BRAKE 6 m/s²), each against the ball's time + tag, same 0.35 s margin (3715–3716); neither safe ⇒ the base the ball takes longer to reach; `MAX_TURNS` = 1 (line 823) is the stand-in for a rundown. |
| `move_runners` | 3747–3960 | Movement, tag-up start (`set_start`, `react_s` 0, `tau_s` 0.68; agent-models §6.4), the halfway hold, the retouch after a catch, waiting for a runner ahead, the overrun; arrival at a base calls `runner_decide` (3928). `settle_guard` (3573) keeps two runners from sharing a base. |
| Constants | 71, 85, 214, 96 | `TYPICAL_RUNNER_SPEED` 8.2 (what fielders assume); `RUNNER_TAU` 0.68 s (global, not per runner); `LEAD_M` 5.5 / 6.5 / 4.5 m; `HALFWAY_M` 5 m. |
| Per-runner data | 489, 4432 | `Runner.vmax` comes from the input (`x.speed`, so speed is already per runner); τ, reaction and the noise scale are not. |
| Base coach | none | `grep coach live.rs` finds only comments about cutoff/backup coaching guides (lines 211, 330, 6586). |
| Rundown | none | The fielders do not close on a trapped runner (agent-models §6.4, "Not yet"). |
| Fielders' throw choice | 4013 (`holder_act`), `live/holder.rs`, agent-models §5.6 | Options per base with a runner heading to it, p² × w, weights `[0.9, 1.0, 1.1, 1.3]` by base (`out_weight`, line 3461), `TWO_OUT_TAG_W` 0.9; "this change does not introduce score/run-expectancy" (§5.6). **Both sides are therefore ignorant of the run table.** |

Agent-models as specified: §6.3 prescribes `P̂(safe) = Φ((t̂_ball − t̂_self)/σ_BRIQ)` and `advance if P̂ · ΔRE_safe + (1 − P̂) · ΔRE_out > ΔRE_hold + θ_aggr`, with base coaches making the send-or-hold call at third (and at second when the runner cannot see the ball) with the same rule. §6.4 holds the interim rules. §8 lists `RE` and `WE` as learned belief tables, bootstrapped from "a public-style RE/WE table [E]".

**Reading the miss.** Three of the brief's four measured misses follow from this table of facts: (i) the 0.35 s ≈ 99 % margin and no run value make *every* decision a near-certainty rule, so second to home (a 41–94 % decision by outs) is sent only when the ball is far away: 34.9 % against 61.7 %; (ii) the runner has no prior on the throw, so a runner on first to third on a single to centre with a man on second races a throw to third that the defense never makes; (iii) the sac-fly case (§1.3) is the same rule plus, as the tag-ups note found, throws that leave faster than real ones. The over-take of first to home on a double (54.1 % against 38.1 %) and the outs on the bases are the other face: the runners who *do* go are the ones whose timing noise puts them in a spot where the race is closer than they think, and the fielders' own throw is the only check. That is a hypothesis; the T4 split (§4) tests it before any change (§4, T4).

## 3. The proposed model, in the project's terms

One decision function for hits, flies, ground balls and throws, in `bb_agents::running` (a pure function from the runner's beliefs and the public situation to a go/hold/back choice and an updated goal; `bb_game::live` supplies the beliefs). That keeps invariants 2 and 3 (no `bb_world` in agents; no private fielder state) and puts both sides' value rule in one place.

### (a) The prior over the defense's play

`defense_prior(situation, ball_belief, fielder_beliefs) → distribution over {throw to base b, cut/relay to b, hold the ball}`, with the situation = outs, score margin, inning, runners and their positions, and the public ball and fielders. It runs the *same value rule the fielders use* (agent-models §5.6, `ChoiceParams`) on the runner's own estimates: his predicted fielder (`believed_field_time`), his scouted arm and set-up (the `Held` branch of `ball_time_to`), each runner's visible position and signed velocity, `out_weight`. When §5.6 later gains run value (ΔRE, which A4 explicitly deferred), the prior inherits it, so the two sides cannot drift. Noise: the runner's read of the fielder's arm already carries σ 1.1 m/s per runner, fielder and play; the prior averages over the runner's remaining uncertainty by evaluating the value rule at the arm and set-up reads he drew, so different runners get different priors from the same play, and a runner with poor reads is *wrong* sometimes in ways a real runner is (mistakes a real player could make).

The prior replaces the "throw goes to my base" assumption in `ball_time_to`: the time the ball reaches base `x` is `Σ_b P(b) · T(x | throw to b)` for the believed destinations, where T is direct if `x = b` and otherwise the cut-and-relay time or never. The decision then uses p(safe) as a mixture, not a mean time. Examples that must come out right: single to centre, nobody on: the throw goes to second, the batter-runner's race is real. Man on second, single to centre: the throw goes home (or to the cutoff on the home line), the batter-runner is expected to take second on the throw (they are on one line: the `thrown_read` fix handles the *observed* throw, this handles the throw before release). Two out, tying run on second: the throw goes home; the trailing runner is freed. The one-hop cases (arm length, a ball past the outfielder) fall out of the arm and transfer numbers.

### (b) His belief of the two races

- **His own time** `t̂_self`: from his own `vmax`, own acceleration (Q1: per-runner τ), his own signed velocity and his start; plus a small self-error σ_self [E, ≈ 0.05–0.1 s: he knows his legs well; ESTIMATE, no source]. The 0.75 factor for a running start (3521) is replaced by the actual `run_time_from` as the relook already does.
- **The ball's time** `t̂_ball(x)` as in (a), with his own perception noise: the timing draw (now `0.15 s`, `RUNNER_TIMING_SIGMA_S`), the arm read, `RUNNER_READ_SIGMA0` for the batted-ball read (2.0 m at 30 m, exists), plus a term for the fielder's execution risk (a bad throw, a bobble): σ_exec [E]; this is the runner's *belief* about throw accuracy and bobbles, which the world produces as physical noise. Combined `σ_race = sqrt(σ_time² + σ_exec² + σ_self²)`; `p̂_safe = Φ((t̂_ball − t̂_self − t_tag)/σ_race)`.
- Perception scales with a per-runner baserunning-IQ factor (agent-models §6.3's `σ_BRIQ`).

### (c) The decision by run value

`go iff p̂_safe > be + θ_risk`, `be = (V_hold − V_out)/(V_safe − V_out)`, with V from the `RE` belief table (§1.5, 24 values as data, 2021–24 average; the sources and its hash recorded as agent-models §8 asks) and a run-weight `w(inning, margin)` added to the immediate run (`V_safe = RE_safe + w`, Silver's marginal-win table gives the shape: ≈ 1 when tied or one behind late, ≈ 0.5 up one, ≈ 0.1 down five; ESTIMATE until a `WE` table exists). The failure state includes the batter-runner's likely advance on the throw (column B of §1.5's table; the prior in (a) knows the throw goes there). Two outs: tag-ups do not exist and the runner goes on contact on a ground ball (forced or not: coaching rule §1.6, no decision); a runner at third with two out on a ground ball goes on contact.

- **Why a margin θ_risk:** the observed success when sent is 94–97 % at every outs state (§1.1–§1.3), so a model that goes at the break-even with a noisy belief would be thrown out far more than MLB's 1–3 % (the runners who go on a 41 % belief are out 59 % of the time on that belief). Two ways to reproduce MLB: a margin on p̂ (aggressiveness, per runner and per team) or a heavier cost for the first and last out. I propose the first, one named parameter with the average as default, **calibrated to the three sourced rates of §1.1** (not to the break-even), and kept visible as MLB's measured conservatism relative to the table (owner question Q1).
- Late, close, tying or winning run: `w` rises and the margin shrinks naturally. Far behind: `V_safe − V_out` falls, `be` exceeds 1 and he holds, "do not risk the out" without a special case.
- Aggressiveness enters here and only here: θ_risk = θ_team + θ_runner.

### (d) Updates, observation, and the commit point

- **Observation events** (existing hooks): ball secured, ball released (`self.out.secured`, `released`), thrown ball's predicted destination (`thrown_read`, kept), ball not held (a bobble, a miss: the `Ball::Free` branch after a touch), the cutoff man's decision (a throw passing the cut man is observed, not predicted). On each, the prior is replaced by the observed thing (a delta on the thrown destination) and p̂ is recomputed from his position and signed velocity (the existing `runner_relook`, with p̂ replacing the 0.35 s test).
- **Look schedule:** (1) at contact + `RUNNER_READ_DELAY_S` [E: the first-step delay; Statcast measures it but I found no value; keep the 0.5 s of today until sourced], (2) as he approaches a base: at a distance equal to his stopping distance plus one stride, the turn decision (where `runner_decide` is called on arrival today (3928), a runner who knows the ball is at the fence has made it a step earlier), (3) each event above.
- **Commit point.** The point past which he cannot stop short of the next base (the existing `settles_on`/`BRAKE` arithmetic), and the point past which he cannot get back before the ball reaches the base behind. Between those a runner who finds "neither safe" is a trapped runner: today `MAX_TURNS`; with a rundown (f) the physical consequence.
- **The sac-fly start:** unchanged (§6.4, `react_s`, `tag_up_time`); the decision at the catch uses (a)–(c) with the prior "throw home".

### (e) The third-base coach

Real: the coach decides for a runner whose back is to the ball; the runner "reads the ball in front, the coach behind" (§1.6); the coach is a stop sign more than a go sign; with two out the send from second is pre-decided. Staged proposal:

- **Stage 1, no new agent (recommended now).** A view gate: if the ball is within his field of view (a cone about his direction of travel, ESTIMATE ±100° pending a source), he decides from his own read, σ_front; if it is behind him he uses the *coach's read*: the same p̂ computed with the coach's vantage (σ_coach, a team parameter [E, average]), plus a short signal delay (a visual reaction, ≈ 0.2–0.3 s, `HOLD_READ_S` is the project's analogue, ESTIMATE). One extra function, two parameters (σ_coach, cone), one new RNG purpose (append-only), and no client change.
- **Stage 2, a coach object** at his box with his own `BaseCoach` attributes (aggressiveness, σ), his own decision draws, and a signal timeline (windmill, stop) the client can play; the runner follows him only when the ball is behind (cone). Cost: a new agent kind and its place in the event log (`CoachSignal`), the client animation, tests that the coach never sees private state. ESTIMATE: three to four times stage 1. It adds the 2-out pre-decision (a team plan: "send on any single from second") which then needs a team plan input.

### (f) What a rundown needs at minimum

1. Detection of "trapped" (no base safe, or the base he left is behind a fielder with the ball): exists in the relook.
2. The defense: the holder runs at the runner while a receiver stands at the base the runner left (the *trailing* base, coaching: chase him back); the holder throws when the runner is committed to a direction; two fielders at least (the two nearest to the bases); the throw choices of §5.6 gain an option "run at runner / tag". The holder-with-ball-chases-runner behavior is the new piece; throws, tags, catches, drops and errors are physical already.
3. The runner: sprints at the holder to force an early throw, reverses on the release (he can only reverse within the existing braking and acceleration law); the trail runner keeps advancing (the existing `base_held_ahead`).
4. Rules: obstruction (a fielder without the ball impeding him; rules-spec) and out of the baseline; I did not check whether `bb_rules` has an obstruction fact.
5. Test: plausibility only (no published rate): nearly all end in an out within three throws; the safe fraction is a few percent and comes from errors and trail-runner scores; the rule "chase him back to the base he left".
This is a separate branch of about the size of the relay work, and the model above is useful without it (MAX_TURNS stays).

### (g) The per-runner attributes this exposes

| Attribute | Used for | Default (the average today) | Source |
|---|---|---|---|
| Top speed `vmax` | `t̂_self` and the legs | 8.2 m/s (`TYPICAL_RUNNER_SPEED`), per runner already in `Runner.vmax` | Q1 ruling; league 27 ft/s |
| Acceleration τ | same | 0.68 s (`RUNNER_TAU`), per runner needed (Q1: 0.75 s refit) | tag-ups note §1, §5 |
| Jump / first-step delay | contact + `RUNNER_READ_DELAY_S` | 0.5 s look today | [E]; Statcast "first step" metric exists, no value found |
| Baserunning IQ (read) | σ of the timing and arm reads; coach σ for his team | 0.15 s today (`RUNNER_TIMING_SIGMA_S`) | [E] |
| Aggressiveness | θ_risk | 0 plus the calibrated margin | agent-models §6.3 `θ_aggr`; no per-player data |
| Slide / turn efficiency | turn arcs, stopping | `BRAKE` 6 m/s² (TASKS Q5 b rules 4.75 m/s² for fielders; the runners' figure is unsourced) | agent-models §6.2 [E] |

## 4. Element by element: parameters, what it replaces, how it is tested

"Named" means a constant or an `[E]` in `bb_agents::running`, not a literal in `live.rs`.

| Element | Parameters it adds | Sourced / [E] | Replaces (live.rs) | Test against Part 1 |
|---|---|---|---|---|
| Prior over the defense | none new (reuses `ChoiceParams`, arm read σ 1.1 m/s, `TYPICAL_*`); `PRIOR_*` for any smoothing | reuses A4's [E]s | `ball_time_to` Held/Free branches (3404–3459); the "my base" assumption | T1, T5 |
| Two races | σ_self [E, 0.05–0.1 s], σ_exec [E], `RUNNER_TIMING_SIGMA_S` (0.15 s, exists as a literal) | [E], none sourced | the `0.15·draw` and the 0.75 factor (3521, 3528) | T1 (mean attempt and out rates), sensitivity of the rates to σ |
| Decision by run value | `RE` table (24 values, SOURCED §1.5), `w(inning, margin)` table [E; shape SOURCED, Silver], θ_risk default [calibrated to §1.1] | RE sourced; w and θ_risk [E] | `> 0.35` at 3533 and 3715–3716, and `MAX_TURNS`' role in some cases | T2 (by outs), T3 (by score), the break-even table of §1.5 reproduced as a unit test from the table |
| Updates and commit | `RUNNER_READ_DELAY_S` [E], look distance [E] | [E] | the single look at 4628–4660 and arrival decide at 3928 | T4 (outs by cause), sac-fly probe |
| Coach, stage 1 | `COACH_SIGMA_S` [E], `VIEW_CONE_DEG` [E], `COACH_SIGNAL_DELAY_S` [E] | [E] | none (adds) | T6 |
| Rundown | `RUNDOWN_*` (chase, throw distance) [E] | [E], no published rates | `MAX_TURNS` and the "stay committed" rule (3737) | plausibility only |
| Attributes | per-runner τ, delay, IQ scale, θ | Q1 | `RUNNER_TAU`, `LEAD_M` and `TYPICAL_RUNNER_SPEED` as global constants | parameter-sweep: faster ⇒ more attempts, fewer outs |

**The tables the sim should reproduce** (sets of 300 games give about 600 team-games; per team-season MLB has 275 first-on-single, 163 second-on-single and 82 first-on-double opportunities, so a 300-game set sees about 1,000, 600 and 300 respectively. At those n the two-sigma half-widths of a rate are about ±3 points, ±4 and ±6; fewer outs on the bases than that cannot be tested at 300 games: 1.1 % of 1,000 is 11 outs; use at least five 300-game sets for T4).

- **T1: attempt and success rates (§1.1, §1.2).** First to third on a single 31.7 % (attempt 32.8 %, safe 96.6 %); second to home on a single 61.7 % (attempt 64.9 %, safe 95.0 %); first to home on a double 38.1 % (attempt 40.7 %, safe 93.8 %). Compare on the same denominator (all singles with a runner on that base, however fielded; B-R's exact denominator is unconfirmed).
- **T2: by outs.** The attempt rate must follow the break-evens' direction: second to home on a single sent more at 2 out than at 0; first to third at 2 out sent less than at 1; the by-outs *values* are a gap (§1.11): get them from Retrosheet before accepting any number.
- **T3: by score and inning.** Down five in the seventh the second-to-home attempt rate falls to near zero at 0 out and 1 out; tied in the ninth it rises (no sourced rates: directions only).
- **T4: outs on the bases by cause.** MLB: 0.268 a team-game of all OOB; 314 of 1,302 (24 %) from the three hit situations above, i.e. 0.065 a team-game, plus the tag-ups and the rest. The sim's 0.23–0.27 must be split into the same causes (hit advance, tag-up, doubled off, stretching a single, caught in a rundown, other) before it is compared.
- **T5: outfield arm.** Runners advance on a strong-arm outfielder less than on a weak: Petriello's 32 % against 38 % on matched chances (a ratio of 0.84); Tango's decision-zone league mean of 35 % and a strong arm at 15 %. With the sim's arm grades the effect should be of this size, not zero and not total.
- **T6: sacrifice flies** (§1.3): scored 75–80 %, doubled off 2–3.5 %, 0.24–0.27 a team-game; no sends under 225 ft, 40 % sent and 90 % safe at 225–275 ft; with the Q1 and tag-up refits the 53.9 % scored today should rise as the margin goes from 99 % to the break-even.
- **T7: units:** every break-even in §1.5 reproduced from the table by the function, for 0 / 1 / 2 outs; a forced runner never uses it; two out with a runner on third on a grounder goes on contact.

## 5. Open questions for the owner

**Q1. Should runners play the run table, or play like MLB runners?**
*Now:* a 99 % rule with no outs or score in it.
*Problem:* the run-expectancy break-evens are 94 % / 73 % / 41 % (second to home on a single, 0 / 1 / 2 out) and 75 % / 34 % (tag from third), yet MLB runners are safe 94–97 % of the time when they go at every outs state. They are more careful than the table says the game rewards (Carleton, Olszewski). A model that goes at the break-even and is right about its odds would be thrown out far more often than MLB runners are.
*Options:* (a) go at the break-even exactly, accept more outs on the bases and more extra bases than MLB; (b) go at the break-even plus a calibrated margin so the sim reproduces MLB's attempt and success rates, which is MLB's actual behaviour; (c) (b) as the default with the margin a per-runner and per-team aggressiveness, so aggressive and timid managers exist.
*Recommendation:* (c). It matches the "imperfect but believable" rule and the §6.3 `θ_aggr` already specified; (a) is what a perfect agent does, and the project's agents are not perfect.

**Q2. Score and inning: a small weight table now, or wait for the learned win-expectancy table?**
*Now:* nothing; the same run value at 2–0 and 0–5.
*Problem:* a run down five in the seventh is worth a fortieth of a tying run (1.9 % against 24.1 %, Silver); a runner who risks the out there is making a mistake no real runner makes, and the "tied in the ninth" runner who does not is making one too.
*Options:* (a) RE only, scoreboard later; (b) RE plus a short table `w(inning, margin)` (about twenty values) built from Silver's shape, replaced by the learned `WE` when it exists; (c) wait for `WE`.
*Recommendation:* (b). It is twenty numbers and one function, it gives the owner's "do not risk the out when far behind" behaviour now, and agent-models §8 already plans the replacement. The numbers are ESTIMATES until then.

**Q3. The third-base coach: a view gate now (stage 1) or a coach on the field (stage 2)?**
*Now:* none; every runner reads the ball himself, however it is behind him.
*Problem:* a runner on first going to third on a single to right, or a runner rounding third with the ball behind him, cannot see the play and real runners follow the coach. Stage 2 is what the client would show (the windmill, the stop) but it is an agent kind, an event and an animation.
*Options:* (a) none; (b) stage 1: an approximation, the coach's read replaces his when the ball is behind; (c) stage 2 now.
*Recommendation:* (b) now and (c) with the client animation work (L2), since stage 1's decision function is the same one a coach object would call.

**Q4. A rundown: now or after the decision model?**
*Now:* a runner who finds no safe base turns round once and stays committed (`MAX_TURNS` = 1), and nothing hunts him; 91 of 300 plays once ran to the 40 s limit without it.
*Problem:* the runner decision model creates more "neither base is safe" moments (the prior makes the race honest), and real baseball resolves them with a rundown nobody has measured. A rundown is also a visible, watchable piece of baseball.
*Options:* (a) build with the decision model on one branch; (b) the decision model first, rundown on its own branch afterwards, `MAX_TURNS` meanwhile; (c) never (keep the stand-in).
*Recommendation:* (b). Measuring the decision model alone (T1–T5) needs the sim to stay stable, and the rundown has no published rates to test against, so it can only be reviewed for plausibility.

## 6. Notes for the lead (not owner decisions)

- **Leads** (`LEAD_M`, live.rs:214): 18 / 21 / 15 ft against about 10–12 ft primary at first (§1.7). Source from Savant before touching; each metre is 0.12 s of race.
- **Retrosheet job** for the by-outs, by-location table (§1.11 item 1), a day for a worker in Python; it would turn T2 and T3 from directions into numbers.
- **B-R's denominators** for `1stS`, `1stD`, `2ndS` should be settled by recomputing from Retrosheet rather than reading column headers.
- **Fielders' side** (agent-models §5.6) has no run value either; the prior in §3(a) is best built when A4's weights `[0.9, 1.0, 1.1, 1.3]` are replaced with ΔRE, or the two sides disagree about what an out is worth.
- A change touching `runner_decide` needs a metrics-ledger row (CLAUDE.md), a Codex review, and the 0.35 s margin test's replacement.

## 7. Sources, with how each was read

SOURCED (opened and read): [B-R 2024 baserunning table](https://www.baseball-reference.com/leagues/majors/2024-baserunning-batting.shtml); [B-R blog, XBT%](https://www.baseball-reference.com/blog/archives/10867.html); [FanGraphs, Clemens, RE matrix](https://blogs.fangraphs.com/the-run-expectancy-matrix-reloaded-for-the-2020s/); [Malter Analytics, 11 Aug 2026](https://www.malteranalytics.com/blog/2026-08-11-baserunning-aggression-re24); [MLB.com glossary, Baserunning](https://www.mlb.com/glossary/statcast/baserunning); [Petriello, 2023](https://www.mlb.com/news/most-valuable-outfielder-throwing-arms-of-2023); [Tango, Statcast Lab 2020](https://tangotiger.com/index.php/site/comments/statcast-lab-batter-runner-v-outfielder-play-at-2b-part-2); [Silver, BP, ORVY](https://www.baseballprospectus.com/news/article/4003/lies-damned-lies-introducing-orvy/) (read to the second table; the ORVY values 1.09 and 3.64 come from a search extract of the same page); [Guilliams, Coaches Insider](https://coachesinsider.com/baseball/base-coaching-absolutes-todd-guilliams-article/); [Wilton Little League](https://www.wiltonlittleleague.org/coaching-resources/base-running/17285) and [Marietta Baseball Academy](https://mbaseballacademy.com/runner-on-second-ground-ball/) (first screens); [Whitney-Epstein et al., Wharton, 2025](https://wsb.wharton.upenn.edu/wp-content/uploads/2025/12/WHITNEY-EPSTEIN_Lead_Distance_Optimization.pdf) (introduction and case study); [SportsOrca, 2026](https://sportsorca.com/mlb/outfield-arm-value-run-prevention/) (secondary); the `mlb_re24_matrix_2024.csv` and `mlb_we_table_2024.csv` files of the sportsdataverse "mlb_game_state" release (parsed; RE used as a cross-check, WE rejected as too sparse).
CARRIED: [2026-11-14-tag-ups.md](2026-11-14-tag-ups.md) (THT Fox 2006, FanGraphs Olszewski 2021, BP Dubuque 2025, BP Carleton 2010, B-R league sacrifice flies), [2026-11-15-outfield-pursuit-and-throws.md](2026-11-15-outfield-pursuit-and-throws.md) §5 (B-R 2024 XBT% 42 %, Statcast arm strength and hold value).
UNVERIFIED (search extract only; no conclusion rests on them): Olszewski's table (the numbers I quote are also in the tag-ups note, read there), Carleton's ESPN article, Baseball Musings/Roley, SkyKing162, DraysBay, Fire Jim Bowden's OOB description, Adler's 11.5 ft, MLB.com's lead-distance glossary page and its 2015 glossary of terms, the Sportngin sheet, the NFCA drill page, Coach and Play checklist, B-R Bullpen on rundowns, Batting Leadoff and NFCA rundown pages, Keep Playing Baseball, Legends Deck, Quora, Rivera/FanGraphs 2024, B-R's WAR method page.
Not read: Baseball Savant leaderboards (Extra Bases Taken, baserunning run value, lead distance; script-loaded), Malter's interactive tables, [Koenig 1988] and the 2025 rolling-start preprint (carried as unread in the tag-ups note), Driveline OpenBiomechanics (excluded by licence, not used).

## 8. The arithmetic

The break-even script (stdlib, `python3 -I`), with the 2021–24 table of §1.5:

```python
RE = {'---':(0.50,0.27,0.10),'1--':(0.90,0.54,0.23),'-2-':(1.14,0.71,0.33),'--3':(1.37,0.98,0.38),
      '12-':(1.51,0.94,0.46),'1-3':(1.82,1.19,0.51),'-23':(2.04,1.41,0.57),'123':(2.38,1.63,0.82)}
def re(b, o, runs=0): return runs if o >= 3 else RE[b][o] + runs
def be(hold, safe, out): return (hold - out) / (safe - out)
# 1st->3rd on a single, outs o:        be(re('12-',o), re('1-3',o),      re('1--',o+1))   # A; re('-2-',o+1) for B
# 2nd->home on a single:               be(re('1-3',o), re('1--',o,1),    re('1--',o+1))   # B: re('-2-',o,1), re('-2-',o+1)
# 1st->home on a double:               be(re('-23',o), re('-2-',o,1),    re('-2-',o+1))
# stretch single to double:            be(re('1--',o), re('-2-',o),      re('---',o+1))
# tag-up 3rd->home (o outs before the fly): be(re('--3',o+1), re('---',o+1,1), re('---',o+2))
```

Rates from the B-R table (`1stS` 8,241; `1stS2` 5,534; `1stS3` 2,615; `1stD` 2,455; `1stD3` 1,457; `1stDH` 936; `2ndS` 4,881; `2ndS3` 1,711; `2ndSH` 3,010): taken = `…3` or `…H` over opportunities; out going = opportunities − stopped − taken; attempts = taken + out going. OOB per team-game = 1,302 / 4,860.

---

# Part 3: What was built and what it did (2026-11-15, branch `work/runner-decision`)

## 9. The first part: the decision by run value and the expected play

Owner's rulings applied: Q2 and Q8 (c) (the run table's break-even plus a caution fitted to MLB), Q9 (a) (a small scoreboard weight), and "the situation first" for the throw. Not built here: the coach's read (Q10), the rundown (Q11), per-runner caution (it arrives with ratings; the parameter is `AdvanceParams::caution_z`).

**Built** (spec: agent-models §6.3, "Built 2026-11-15"):

1. `bb_agents::running`: the run-expectancy table of §1.5, `AdvanceValues` and its break-even (the table of §1.5 is the unit test), `run_weight(inning, margin)`, `AdvanceParams { sigma_race_s 0.20, caution_z 1.0 }`.
2. `runner_decide` and `runner_relook` use it in place of the flat 0.35 s margin. The runner's own time is his law of motion from his speed now (the 0.75 running-start factor is gone).
3. `with_expected_play`: before a throw is in the air the runner runs the fielders' rule on what he sees and expects the throw to go to the best play; his own base then gets the ball by a cut or a second throw.
4. Two faults found by tracing single plays, both mistakes no real runner makes: a runner arriving at third **stopped** behind a runner a few strides from scoring (he waited for home to be "cleared", then restarted from rest, 0.75 s lost); and with **two out** a runner went back to tag up on a fly he expected to be caught.

**Differences from §3 above, and why:** the caution is in spreads of the race, not in probability or runs (a fixed amount added to a 94% break-even is "never"); the scoreboard weight scales the gain of the base instead of the run (`V_safe = RE_safe + w` would stop a runner far behind at third on a ball at the wall); the expected play is a single best play at the runner's own noisy reads, not a distribution.

**Measured** (`cargo run --release -p bb_game --example runner_rates -- 300 <seed>`; main `a9b3341` → branch, seeds 0–299 [1000–1299]):

| | main | branch | MLB |
|---|---|---|---|
| 1st → 3rd on a single: took / put out | 25.7 / 2.4 % [23.4 / 1.7] | 23.5 / 0.9 % [21.3 / 1.8] | 31.7 / 1.1 % |
| 2nd → home on a single | 34.6 / 1.2 % [32.2 / 2.5] | 34.2 / 0.2 % [33.4 / 1.1] | 61.7 / 3.3 % |
| 1st → home on a double | 52.9 / 0.7 % [44.2 / 1.0] | 54.5 / 0.4 % [52.0 / 1.0] | 38.1 / 2.5 % |
| tag from third scores, fly caught beyond 55 m, 0 / 1 out | 31 / 34 % [39 / 35] | 35 / 37 % [37 / 46] | 75–80 % of all outfield catches |
| outs on the bases, a team-game (tags and appeals, and forces after a catch) | 0.497 [0.507] | **0.273 [0.280]** | 0.268 |
| of which on a hit: batter / other runners | 0.125 / 0.057 [0.155 / 0.058] | 0.063 / 0.022 [0.047 / 0.050] | 0.065 for the three advances |
| runs a team-game (standard run) | 3.85 [3.99] | **4.11 [4.19]** | 4.39 |

(The tag-up line is this probe's own denominator, every fly caught more than 55 m from the plate; the 53.9 % of the tag-ups note counted differently and the two are not comparable.)

**Reading.**

- **Outs on the bases halved, to MLB's figure**, and runs rose about a quarter of a run: the runners who went on the old rule and were out were the cost.
- **The extra bases taken did not move.** Caution 0.5 in place of 1.0 was tried (seeds 0–299): second to home 34.0 %, outs on the bases 0.318. So the rate is not held down by the threshold.
- **It is held down by the race.** In 60 traced games, of 106 singles with a runner on second, he scored on 40. Of the 66 he did not: 15 never reached third on the play (infield singles and the like), 26 believed they would lose the race home by less than 0.5 s, 10 by 0.5–1.0 s, 10 by more, 5 believed they had it by too little. A race 0.5 s kinder to the runner would send about 70 of 106 (66 %; MLB 62–65 % attempts). The sim's runner from second touches third 3.78 s after contact and home at 7.13 s: every runner's first look is a flat 0.5 s after contact, from a standstill at his lead (no secondary lead, no moving start, and with two out nothing to read), and the ball is in an outfielder's hand or already thrown when he reaches third. Which side holds the 0.5 s (the runner's start, or the outfield's time to field and throw, which the tag-ups note already found quick) is not settled here: it needs sources (Statcast first-step and secondary-lead figures, outfield exchange times), and it is the next piece of work, ahead of fitting the caution.
- **By outs** the model orders the decisions where the race is close (unit and live tests), but the rates by outs above are flat for the same reason: few races are close enough for the break-even to matter.
- **First to home on a double is still too frequent** (about 53 % against 38 %) and nearly never an out: these are balls to the wall on which the runner is right to go; the count of doubles with a runner on first, two out, nearly doubled on the branch (88 → 167 [79 → 153]). Not explained: either the scorer credits a double where the batter took second on the throw home (he now expects that throw), or running on contact with two out changes how those plays end. Open (ledger §5).
