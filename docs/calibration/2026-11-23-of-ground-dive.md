# The outfielder's dive at a ground ball (2026-11-23)

Worker `of-dive` on `work/of-ground-dive` (from main `f6eba9b`). The owner's saved play 71 of seed 51001 (`saved-plays/seed51001-play71-20261120-001602.toml`; observations.md item 71) after the [bounce track](2026-11-21-bounce-track.md) §4 was still a triple: the right fielder, a step short after a believable in-flight misjudgment, let a hard grounder roll past him about 2 m away without diving. The question: is the outfielder's dive at a ground ball missing, or wrongly gated? Spec: [agent-models §5.3](../agent-models.md#53-movement), "An outfielder has nobody behind him".

## 1. What stopped him: `OF_DIVE_MIN_P`

A throwaway probe (not committed) printed every tick of the play-71 fixture test (`the_owners_rolling_ball_seed_51001_play_71_is_played_at_its_line`, the play's own contact and streams) from 2.9 to 3.7 s: the right fielder's distance to the ball, his dive envelope, what `dive_reads`/`dive_gains` answered and the `OF_DIVE_MIN_P` test's numbers.

| t (s) | ball from him (m) | his reach + extension (m) | `try_captures` | his read: σ_g (m), P(pocket) |
|---|---|---|---|---|
| 3.002 | 7.70 | 1.00 | out of reach | 0.070, 0.31 |
| 3.035 | 7.02 | 1.00 | `dive_gains`: no standing take by his read (earliest dive 3.41 s): `must_dive_since` = 3.035 | 0.070, 0.31 |
| 3.135 | 5.06 | 2.50 (full: 0.3 s lunge earned) | out of reach | 0.070, 0.31 |
| 3.318 | 2.33 | 2.42 | a dive offers; **P below 0.5: stays up** | 0.068, 0.32 |
| 3.368 | **2.11** (closest) | 2.50 | a dive offers; **stays up** | 0.063, 0.37 |
| 3.435 | 2.40 | 2.50 | a dive offers; **stays up** | 0.068, 0.32 |
| 3.452 | 2.54 | 2.50 | gone by | |

So the dive was not missing and the lunge was not short: his read said from 3.035 s that he could not take the ball on his feet (`dive_gains` held without a break at every 0.1 s read after), the full 1.5 m extension was his from 3.135 s, and the ball was inside his 2.5 m envelope for 0.12 s, rolling at 16.1 m/s. **The gate that stopped him is `OF_DIVE_MIN_P`:** with average hands (σ_hands 0.018 m) laid out 2.11 m from a 16 m/s ball, σ_g = 0.018 × 3.47 = 0.063 m, and the chance that the glove is within the 0.06 m pocket is 1 − exp(−½(0.06/0.063)²) = 0.37, below 0.5. The rule counted only a clean catch.

## 2. Judged as a player would judge it

The rule's reason, "an outfielder has nobody behind him", is that a ball he dives for and does not hold gets by him. For a ball in the air that holds: only a catch is an out, and a liner off his glove is still a hit, so a dive that will probably not catch it trades a single for extra bases. For a ball on the ground it does not: the ball is going past him anyway (`dive_gains` has already said he cannot get it on his feet), and a knock-down stops it as surely as a clean pick. §5.4 already says so ("bobbles that stay close can be re-fielded"), and the sim's deflection keeps a third of the ball's speed plus a push of 0.15 of it, so a ball leaves the glove at about a quarter of its speed (measured by the review on outfield ground-ball rims: a median 3.8 m/s out of 15.9) and stays near him. A real outfielder lays out to keep a hard grounder in the gap from reaching the wall. At 2.11 m his chance of a glove on it (within the 0.12 m rim) is 1 − exp(−½(0.12/0.063)²) = 0.84.

**The change** (spec first, agent-models §5.3): an outfielder weighs a dive at a ball still in the air by the pocket, as before, and at a ball on the ground (it has come down: the public fact `dive_gains` also uses) by the rim. The threshold stays `OF_DIVE_MIN_P` 0.5, the radii are §5.4's, and there is no new parameter or fudge factor. The in-flight misjudgment is untouched. With average hands the rim chance is at least 0.74 anywhere in the dive envelope (σ_g at most 0.018 × 4.05 = 0.073 m), so an average outfielder lays out for a ground ball he can reach in the lunge. A glove worse than σ_hands about 0.025 m stays up at full stretch.

## 3. Play 71 now

He lays out at 3.368 s, the ball 2.11 m from him. **His glove draw on this play is a clean miss**: 2.2σ (`GlovePlacement` for his first attempt: e = (2.10, 0.68)), 0.137 m from the ball against the 0.12 m rim, about a one-in-six chance at that σ. The ball goes on to the wall and the batter still reaches third (the centre fielder takes it at 8.67 s by the wall). So the play is now decided by the glove, not by his choice: he does what a real outfielder does, and on these draws misses by 1.7 cm. With a true read from the bounce (the bounce-track note's counterfactual) he dove from 1.4 m and knocked it down. The fixture test now asserts the dive (once, before 3.5 s, from beyond his stretch and within his dive) and prints the outcome without asserting it.

## 4. Tests

- `an_outfielder_lays_out_for_a_grounder_passing_within_his_dive` (new): a 20 m/s grounder passing 2.3 m from an average centre fielder (out of reach on his feet, the lunge earned). Over ten plays, each with its own reads and glove draws, he dives on 9 (on one his read tells him too late that he cannot take it standing) and keeps the ball by him (caught, or knocked down below 8 m/s) on most. The test asks for at least 8 dives and 6 balls kept. It fails on the old rule (0 dives).
- `an_outfielder_stays_up_for_a_grounder_he_can_take_on_his_feet_or_cannot_reach` (new): average hands; 1.3 m is taken with a stretch, 2.8 m is beyond any dive, and 2.2 m seen 0.15 s ahead is too late for the 0.24 s lunge. No dive in any of them.
- `an_outfielder_does_not_dive_for_a_ball_he_will_probably_miss`: now a low liner still in the air as it passes 2.3 m away (fixture-checked: first bounce after it passes, 0.43 m up). The centre fielder stays up and the shortstop dives. `an_outfielder_weighs_a_dive_by_his_own_read_of_the_ball` likewise moved to a ball in the air (0.5 m up), unchanged otherwise.
- The play-71 fixture: as §3. Both the new grounder test and the fixture fail with the old rule put back (a mutation check: pocket for both).
- `throw_choices::a_ball_past_the_outfielders_adds_the_relay_man_and_keeps_the_first_baseman`: seed 118 play 162 dropped. The right fielder now lays out for that grounder at 3.56 s and knocks it down (re-fielded 6.07 s), so it no longer gets past the outfielders, the test's premise. The other two plays remain.

## 5. Measurements (standard run, `sim-cli` rebuilt for each side)

Main `f6eba9b` → `work/of-ground-dive` `5c2c556` (`14d1030` after it changes only a test), seeds 0 [1000]; dives from `examples/dive_probe` and hits by bases from `examples/hit_bases` on the same games (per team-game = per 600).

| Measure | main | branch |
|---|---|---|
| runs/team-game | 4.14 [4.45] | 4.11 [4.39] |
| dives a game (MLB about 2–3) | 2.5 [2.5] | 3.0 [3.0] |
| outfield dives at ground balls a game (LF+CF+RF) | 0.21 [0.23] | 0.71 [0.65] |
| caught at the dive, outfield ground balls | 57% [51%] | 42% [42%] |
| outfield dives at balls in the air a game | 0.04 [0.04] | 0.03 [0.04] |
| singles / doubles / triples a team-game | 5.05 / 1.99 / 0.68 [5.10 / 2.06 / 0.71] | 5.10 / 2.05 / 0.59 [5.16 / 2.08 / 0.60] |
| GB hits (MLB about .245) | .308 [.311] | .309 [.311] |
| LD / FB / PU hits | .672 / .099 / .006 [.663 / .101 / .008] | .671 / .099 / .006 [.661 / .100 / .008] |
| HR/PA, K%, reached on BIP | 2.74%, 22.6, 33.8% [2.88%, 22.0, 33.8%] | 2.75%, 22.6, 33.8% [2.88%, 22.0, 33.8%] |

The intended change: outfield dives at ground balls triple (about 0.5 more a game). Triples fall by 0.09 [0.11] a team-game (13% [15%]); singles rise by 0.05 [0.06] and doubles by 0.06 [0.02]: a ball knocked down near the outfielder holds the batter to fewer bases. Runs −0.03 [−0.06], within the paired noise (about 0.15 for 300 games that diverge early). Ground-ball hits do not move (.308 against .245 is an infield matter: the dive does not stop a ball getting through the infield, only how far it goes after). Dives a game reach 3.0, the top of the real 2–3. "Caught" at the dive counts only clean catches; the knock-downs `dive_probe` does not count are the point of the change.

## 6. Open

- Play 71 on its own draws: the dive is taken and misses by 1.7 cm (§3), so the owner's play is still a triple.
- Dives 3.0 a game, at the top of MLB's 2–3. If it should come down, the infield's 2.2 a game is the larger share (not touched here).
- Triples at 0.59 [0.60] a team-game are still well above MLB's rate.
- The deep-call test lost its seed 118 play 162 case (§4), which was the one where the call must not flip back as the centre fielder takes over from the right fielder. That needs a new frozen play of the same kind.
- A missed glove does not hit a laid-out body (§5.4's "may hit the body capsules"). Here the body is a metre from the ball's line, so it would not matter.

## 7. After the review (2026-11-24)

The [Opus review](../reviews/2026-11-23-rev-of-dive-of-ground-dive-review.md) found one design gap (P2-1). `dive_gains` charged every dive at a ball on the ground the 1.5 s get-up, the cost of a catch. With the rim gate on the ground, outfielders laid out at balls they would have had on their feet within about 2.7 s, where a knock-down gains nothing and a miss costs extra bases.

**Built** (agent-models §5.3, "What a dive at a ball on the ground costs him"; `Sim::dive_pays`, `ef1e83b`, `4d4ff4b`). On the ground he weighs the dive by its expected time to have the ball, by his own read and hands:
- **A catch** (pocket): up after `DIVE_GETUP_S`.
- **A knock-down** (rim): up, then his run from where he lies to a ball that left his glove along its path at `RIM_KEEP` 0.3 of its speed. That is the deflection law's along-path part, now named; the push's direction is his own unknown glove error, so he expects none.
- **A miss:** up, then his chase of the ball as his read has it.

He dives only if that is earlier than his standing take. No new knob. The paired probe then showed three gaps, each built with a spec line:
1. A read whose standing take misses the ball before it comes to rest used to send him in at once. Now such a take ends at the resting point, reached at his top speed.
2. His plan schedules dives only to `REACH` + 0.6·`DIVE`, while the capture allows the full `DIVE`. A dive the plan has no point for is now weighed where his read has the ball pass closest while it is faster than `LUNGE_MIN_MPS` 3 m/s (the capture's existing floor, now named).
3. A dive his plan makes only after the ball has come down (a short hop) was decided by the in-air rule. It is now weighed as a dive at a ball on the ground. This reaches infielders too.

**The review's paired table, again** (an uncommitted probe: each game stepped in lockstep under this rule and under main's until the first play that differs; the outcome of the new dive by the standing take, i.e. when the same outfielder took the ball on his feet in main's game, counted from the dive; better / worse by batter's bases + 2 × runs − 2 × outs on the play; seeds 0–299 and `sim-cli`'s seed-1000 set, game seeds from 1,000,003,000, which this section first called "1000–1299"):

| Standing take | The review (rim gate, get-up only) | Now |
|---|---|---|
| under 1.5 s | 24 dives: 1 better, 9 worse | 16: 0 better, 5 worse |
| 1.5–2.7 s | 41: 6 better, 6 worse | 5: 1 better, 1 worse |
| 2.7 s or more | 93: 60 better, 3 worse | 72: 55 better, 2 worse |
| another fielder took it | 66: 40 better, 1 worse | 57: 33 better, 1 worse |
| all new outfield ground dives | 224: 107 better, 19 worse; batter's bases −118, runs −24 | 150: 89 better, 9 worse; bases −121, runs −18 |

The dives at balls he would have had on his feet within 2.7 s fell from 65 to 21, and those made worse from 15 to 6. They are still net worse (1 better, 6 worse), so this part of the job's aim is not fully met. *(Corrected after the second review, P3-4: this account was half right; see §9.)* The rest are almost all reads that put his standing take 2–4 s away when he took the ball on his feet 0.1–0.7 s later in main's game. The far cells keep most of their gain (88 better and 3 worse of 129, against the review's 100 and 2 of 159). The rule also changes 102 other first plays, mostly infielders' dives at short hops, and the defence gained on those too: 30 better, 18 worse, 12 more outs in 600 games. Against the reviewed branch, the 78 dives the new rule removes split 18 the dive would have made better and 14 worse.

**Measurements** (standard run, `sim-cli` rebuilt; `dive_probe`, `hit_bases`; main `f6eba9b`, the reviewed branch `5c2c556`, now `7715575`):

| Measure | main | reviewed | now |
|---|---|---|---|
| runs/team-game | 4.14 [4.45] | 4.11 [4.39] | 4.11 [4.33] |
| dives a game | 2.5 [2.5] | 3.0 [3.0] | 2.6 [2.6] |
| outfield dives at ground balls a game | 0.21 [0.23] | 0.71 [0.65] | 0.51 [0.52] |
| infield dives a game (P, 1B, 2B, 3B, SS) | 2.21 [2.21] | 2.22 [2.26] | 2.06 [2.04] |
| singles / doubles / triples a team-game | 5.05 / 1.99 / 0.68 [5.10 / 2.06 / 0.71] | 5.10 / 2.05 / 0.59 [5.16 / 2.08 / 0.60] | 5.11 / 2.05 / 0.59 [5.17 / 2.07 / 0.61] |
| GB / LD / FB / PU hits | .308 / .672 / .099 / .006 [.311 / .663 / .101 / .008] | .309 / .671 / .099 / .006 [.311 / .661 / .100 / .008] | .308 / .668 / .100 / .005 [.311 / .663 / .100 / .008] |
| HR/PA, K%, BB% | 2.74%, 22.6, 8.1 [2.88%, 22.0, 8.5] | 2.75%, 22.6, 8.1 [2.88%, 22.0, 8.5] | 2.74%, 22.6, 8.0 [2.79%, 22.1, 8.4] |

Dives are back inside the real 2–3 a game. The triples saved are kept (0.59 [0.61] against main's 0.68 [0.71]). Runs at seed 1000 are 0.12 under main. HR/PA moved by as much (2.88% → 2.79%) though no dive touches a home run: these are 300 games that diverge early. *(Corrected after the second review, P3-3: the "18 fewer runs in 600 games, about 0.015 a team-game" first given here counted only each game's first changed play; played to the end, pooled over three sets, the effect is about −0.05 ± 0.02 runs a team-game, §9.)*

**The other findings.** P3-1: `an_outfielder_with_poor_hands_stays_up_for_a_grounder_he_would_rarely_glove` pins the gate's ground side. It uses σ_hands 0.04 m, because at 0.03 m his read on that play put the ball near enough for a better-than-even rim chance. The lay-out test now checks where the ball ends up (in his glove or within 8 m of his take-off, the unplayed ball 16–17 m on) over 20 plays: 17 dives, 16 kept. `a_dive_at_a_ground_ball_pays_only_when_its_expected_cost_beats_his_standing_take` covers the cost rule: a 10 m/s grounder he would take standing 1.8 s on, which the get-up alone let him dive at, against a 14 m/s one 4 s on. It fails with the get-up-only charge. P3-2: the spec's poor-hands figure (0.025–0.027 m) and the rim's exit speed are corrected. P3-4: seed 36 play 158 (two out, a runner on first, a liner through the right side the centre fielder takes over from the right fielder) is frozen into the double-cut test in place of 118/162, which now passes on three plays. P3-3 (the rim's fixed-axis push) is deferred to TASKS §2.

## 8. Open after the review

- Dives at balls he would have had on his feet within 2.7 s: 21 in 600 games. The second review's account (P3-4) of the four it found worse: two lunges at slow balls with no standing take and no pass within reach (its P3-1, fixed in §9), one dive the in-air rule allowed while the ball was up that came after the bounce (seed 195 play 76), and one misread of the standing take (seed 1031 play 293). In main's game the same fielder took the ball on his feet at 0.95–1.0 m in 9 of the 21, by a stretch in 1, and dove too in 5. So the plan's 0.7 reach hides takes inside his 1.0 m standing reach, not the stretch, and accounts for about one bad play in 600 games: not worth a task beyond the existing planner-reach line.
- The short-hop weighing changed infield dives too (2.2 → 2.05 a game). The paired probe has it helping the defence, but it has had no review of its own.
- Play 71 is still a triple on its glove draw. Triples are still well above MLB's rate. P3-3 is open in TASKS §2.

## 9. After the second review (2026-11-24)

The [second Opus review](../reviews/2026-11-24-rev-of-dive-2-of-ground-dive-review.md) found two defects in §7's rules. Built in `32104b7` (spec agent-models §5.3), each with the fix the review suggested:
- **P2-1, catchable pop-ups and liners dropping.** A ball still up stays on the in-air rule whenever his read has it pass within the capture's envelope (`REACH` + `DIVE`, at or below `DIVE_MAX_Z`, from where his movement will have him) before its first bounce. A wall touch counts as a bounce: a ball off the wall is no catch for an out. Only a dive he can make nowhere but after the bounce is weighed as a ground dive.
- **P2-2 and P3-5, the dive weighed.** `dive_pays` weighs the earlier of the plan's first dive point and the ball's first pass within his dive reach, at its closest. His dive reach there is his arm plus the layout his lunge time allows, counted as the capture counts it; the ball must be low enough, not past a wall, and faster than `LUNGE_MIN_MPS`. Both points are measured from where his own movement will have him, and his landing is kept off the wall (`lunge_land`, now shared with the capture).
- **P3-1.** With no standing take and no dive to price, the resting-point take stands for it: he may go only if that is more than a catch's get-up away.
- **P3-2.** The cost test has its middle case (10.5 m/s: standing take 2.07 s, a catch up at 2.03 s, expected 2.31 s, so he stays up).

Tests, each failing under its own mutation (an old rule put back in a copy of `live.rs`):
- `an_infielder_lays_out_for_a_catchable_pop_up`: seed 106 play 191, the third baseman's foul pop. He now dives and catches it. Fails with the old short-hop split.
- `the_pitcher_lays_out_at_a_chopper_up_the_middle`: seed 290 play 242. He dives and catches it. Fails with the plan's dive point alone.
- `nobody_lays_out_at_a_ball_coming_to_rest_out_of_his_reach`: constructed. Fails with the old "no standing take, so go".
- The cost test's middle case: fails with the get-up from the dive point alone.

The fixtures are frozen from `0da2212`, and the replay matches the game's play there.

**Not restored, and not built** (the job asked for the review's fixes only):
- **Seed 1043 play 201,** the second baseman's liner. His own read at 1.17 s has the liner bouncing 0.07 s before it reaches him, so by the review's rule it is a short hop and the hold breaks. He takes it on the hop at 1.56 s.
- **Seed 64 play 57** is no longer a P3-1 case. With the in-air rule held (his read has the fly come down within his reach), the right fielder lays out at it a tick after it lands. This is the review's "mirror" case (seed 195 play 76: a hold earned in the air carries past the bounce), which P3-4 asked for no code. The P3-1 test is the constructed one.

**The paired probe, whole games** (each game played in lockstep under two rules to the end; the first differing play classified as before; seeds 0–299 and 1000–1299 unless named):

| Against main's rule | The second review (branch before this round) | Now |
|---|---|---|
| runs a team-game, pooled over three sets (0–299, 1000–1299, `sim-cli` 1000) | about −0.06 ± 0.02 | **−0.049 ± 0.016** (−0.042 ± 0.018, −0.028 ± 0.025, −0.078 ± 0.037) |
| new outfield ground dives on first changed plays | 146: 90 better, 5 worse; bases −111, runs −26 | 191: 104 better, 12 worse; bases −126, runs −22 |
| standing take under 1.5 s / 1.5–2.7 s | 12 (0/4) / 9 (3/0) | 16 (0/4) / 16 (3/2) |
| 2.7 s or more / another fielder | 63 (48/1) / 62 (39/0) | 92 (62/5) / 66 (39/1) |
| diving catches in the air lost | 24, 10 worse | **5, 1 worse** (78/74 SS, 1112/263 3B, 1188/174 1B, 1245/194 2B, 1258/140 2B) |
| infield dives lost on first changed plays | 28: 2 better, 5 worse | 36: 7 better, 3 worse (pitcher 4, catcher 5, first 11, second 4, third 8, short 4) |
| all first changed plays | | 234: 115 better, 16 worse, 8 more outs, bases −132, runs −24 |

Against the branch before this round (`0da2212`): runs −0.017 ± 0.019 and +0.010 ± 0.018, no difference. Air catches: none lost, 12 regained. Infield dives regained on first changed plays: 67 (pitcher 14, first 20, second 13, third 11, short 9), 2 lost. Over all 127 first changed plays, 31 better and 24 worse. The cost is early outfield dives: the pass point now prices the imminent dive, so 20 new outfield ground dives fall in the under-2.7 s cells (1 better, 5 worse), at balls his read had later than the world.

**Standard run** (`sim-cli` rebuilt at `32104b7`; main → reviewed rounds → now):

| Measure | main | after §7 | now |
|---|---|---|---|
| runs/team-game | 4.14 [4.45] | 4.11 [4.33] | 4.09 [4.37] |
| dives a game | 2.5 [2.5] | 2.6 [2.6] | 2.8 [2.8] |
| outfield dives at ground balls a game | 0.21 [0.23] | 0.51 [0.52] | 0.59 [0.58] |
| infield dives a game | 2.21 [2.21] | 2.06 [2.04] | 2.15 [2.20] |
| singles / doubles / triples a team-game | 5.05 / 1.99 / 0.68 [5.10 / 2.06 / 0.71] | 5.11 / 2.05 / 0.59 [5.17 / 2.07 / 0.61] | 5.08 / 2.04 / 0.56 [5.18 / 2.08 / 0.60] |
| GB / LD / FB / PU hits | .308 / .672 / .099 / .006 [.311 / .663 / .101 / .008] | .308 / .668 / .100 / .005 [.311 / .663 / .100 / .008] | .307 / .670 / .098 / .005 [.312 / .662 / .101 / .007] |

§7's "the dive's own effect … about 0.015 a team-game" counted only each game's first changed play. Played to the end, the rule's effect against main is −0.05 ± 0.02 runs a team-game (P3-3).

Open: seed 1043 play 201; the mirror case (a hold earned in the air carried past the bounce: seeds 64/57, 195/76); the early outfield dives at balls his read has later than the world.
