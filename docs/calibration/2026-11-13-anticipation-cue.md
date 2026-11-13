# A further honest anticipation cue: what carries spray information (2026-11-13)

**Band and mechanism.** No calibration knob was turned toward a target. By owner decision (2026-11-13), elite fielders should get a further *honest* cue so that `Anticipation` matters, and (coordinator decision, following from it) a fielder whose read is implausible should fall back on his other reads instead of standing still. This note records which cues a fielder could honestly have, how much they tell him about the spray in this sim, the one chosen, and how the run environment moved. Every error is derived from perception, not fitted to a run total (agent-models §5.10).

## 1. Which cues carry information about spray

`examples/spray_cues.rs`, 200 games (seed 0–199, base `coord/review-fixes`), 11,588 fair balls in play, every batter right-handed: spray mean −4.8°, sd 22.48°. For each candidate, its spread, its correlation with spray, the spray scatter left after a straight-line fit, and what is left after a joint fit with the pitch's horizontal location (the existing prior's input; alone it leaves 22.12°):

| Cue | When a fielder has it | sd | r | Left alone | Left with location |
| :--- | :--- | ---: | ---: | ---: | ---: |
| Pitch location, horizontal | in flight | 0.143 m | 0.18 | 22.12° | 22.12° |
| Pitch location, height | in flight | 0.166 m | 0.02 | 22.48° | 22.11° |
| Called target, horizontal (catcher's sign) | before the pitch | 0.132 m | 0.11 | 22.35° | 22.12° |
| Called target, height | before the pitch | 0.174 m | −0.01 | 22.48° | 22.12° |
| Release speed | in flight | 2.58 m/s | 0.00 | 22.48° | 22.12° |
| Speed change from the previous pitch | in flight | 3.05 m/s | 0.03 | 22.48° | 22.11° |
| Balls, strikes | before the pitch | | −0.01, −0.02 | 22.48° | 22.11° |
| Called pitch type (CH/CU/FF/SL) | before the pitch | means −5.0, −3.8, −4.9, −4.7° | | 22.48° | 22.12° |
| **Bat face at contact** | at contact | 9.38° | 0.58 | 18.36° | 18.32° |
| **Contact minus the swing's nominal time** | during the swing | 4.12 ms | −0.51 | 19.38° | 18.35° |
| **Contact depth (y of contact)** | at contact | 0.157 m (4.1 ms of ball travel) | −0.57 | 18.42° | 18.42° |
| **The swing's lead (plate time − bat launch)** | during the swing | 8.15 ms | 0.56 | 18.60° | 18.32° |
| Bat tilt at contact | at contact | 8.56° | −0.12 | 22.31° | 21.80° |
| Swing start since release | during the swing | 30.4 ms | 0.15 | 22.23° | 21.88° |
| Launch angle (outcome, not a cue) | after contact | 29.3° | 0.59 | 18.18° | 17.23° |

- **The called pitch carries nothing.** Everything a fielder can know before the ball is hit, read perfectly and fitted jointly (horizontal and vertical location, called target, speed, count, type), leaves 22.02° against 22.12° for horizontal location alone. Pitch type moves the mean spray by 1.3° at most. The batter was fooled on the type (read it wrong) on 0.8% of balls in play: those were pulled 8° more, too rare to matter. Batters do not pull offspeed more in this sim, because the batter's timing barely depends on the type he gets. The called location is a noisier version of where the pitch actually crossed, which he already reads.
- **The batter's history carries nothing.** A per-batter (per game) spray component of about 3° is within sampling noise, and predicting a ball from the batter's previous balls in play that game makes things worse at every shrinkage tried (22.50° to 23.30°).
- **Everything informative is the batter's timing.** The face, the timing offset, the contact depth and the lead each leave about 18.3° with location, and any two together are no better than one (with everything visible before contact, the face leaves 18.24°, the timing offset 18.27°, the depth 18.35°). They are four views of one thing: how early the batter was. The remaining 18° is the collision (launch angle and spray correlate at r = 0.59: grounders are pulled), which nothing before contact shows.

## 2. Honesty: what a fielder can read of the batter's timing

Every view of the batter's timing is a timing judgment, and a fielder's temporal precision (20–80 ms, agent-models §5.10 (a)) is several times the batter's timing spread (4–8 ms):

- **Bat angle at contact** (already built): 40 ms at grade 50 × 1,915°/s = 77° against 9.4° of spread.
- **Contact depth relative to the plate**: the ball moves 38 m/s, so a 40 ms error is 1.5 m against 0.157 m of spread. Equivalent to the bat angle, not better.
- **The swing's timing as it develops**, judged over the visible swing rather than at one instant: whether the bat is ahead of or behind the ball. For one look, the error is set by when the look is taken, which is the same limit. Several looks would help only if their errors were independent, which is not established (the published values are single judgments, and §5.10 (c) already takes one judgment per swing). Comparing two objects seen together cancels the shared timing error only when they move together (Chung, Levi & Bedell 1996, as analysed by Brenner & Smeets 2010), which a bat and an incoming ball do not (near contact they close at about 70 m/s). So it is not honestly better than the instant read, and fusing it as a separate cue next to the bat read would count the batter's timing twice.
- **The swing's lead** (chosen): when the bat starts forward, against the ball's flight. It carries both halves of being early (starting early and meeting the ball out front), so its spread is 8.2 ms, twice the timing offset's, with the same correlation with spray. But the read pairs an event on one object (the bat's launch) with where another object (the ball, 6 m out) is: Linares, Holcombe & White (2009) with the cue away from the moving object (Exp. 1), **75 ms** at a practised observer, not their 40 ms with the cue on the object's own path. In face units it is then as good as the bat-angle read, no better: 1,063°/s × 75 ms = 80° at grade 50 against the bat read's 77°.

What a fielder's noisy reads are worth, as the spray scatter left by the best linear fit on them (`spray_cues.rs`, `honest()`, the same 200 games, regenerated with the change in place, so the base numbers differ from the table above by a tenth of a degree):

| Grade (timing σ) | Location only | + bat angle | + depth | + lead at the bat read's σ | + bat angle + lead at that σ | **As built: + bat angle + lead at 75/40 of it** |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| 20 (78 ms) | 22.56° | 22.55° | 22.55° | 22.52° | 22.50° | **22.54°** (lead 146 ms) |
| 50 (40 ms) | 22.44° | 22.39° | 22.39° | 22.29° | 22.24° | **22.36°** (lead 75 ms) |
| 80 (20.5 ms) | 22.36° | 22.18° | 22.23° | 21.94° | 21.80° | **22.06°** (lead 38 ms) |
| 160, lab (5.4 ms) | 22.26° | 20.77° | 21.09° | 19.77° | 19.53° | **20.20°** (lead 10 ms) |

**The chosen cue is the swing's lead**, because it carries the most spray information per millisecond of reading error of the honest views, and it is a second, independent judgment (a different event, a different object), so its error is independent of the bat read's. It is modelled as a second read of the bat's face, not a second spray cue (no double counting): agent-models §5.10, "During the swing: the swing's lead".

**What it is worth, honestly:** at grade 80 it takes the spray scatter from 22.18° to 22.06°; at grade 50 from 22.39° to 22.36°. Nothing honest in this sim gives a fielder on the 20–80 scale much more than the pitch prior. Even reading every timing view perfectly would only bring the scatter to 18.3°.

## 3. The model

- **The read.** `game.rs` computes the swing's true lead, L = t_plate − t_start (the pitch's free flight crossing the judgment plane, less the executed swing's start at 10% of peak speed), and passes it into the play (`LiveInput::swing_lead`). `jump_at_contact` adds each fielder's error: σ `lead_sigma` × truncated normal (4σ), from purpose `FielderAnticipation` sub-index 2, keyed per fielder (data-formats §2.3). Existing streams (sub-indices 0, 1) are unchanged.
- **Error per grade.** 75 ms at grade 50 (`LEAD_READ_TIME_SIGMA_50_S` [E], Linares Exp. 1), by the Anticipation ρ of 1.25: 38.4 ms at 80, 146 ms at 20, 10 ms at 160. Elite fielders gain more because their error is smaller, not by fiat.
- **The belief.** Face ≈ 161.93° − 1,063.1°/s × L, 3.6° scatter (`LEAD_FACE_*` [E], r = 0.93, fitted on the base). The face estimate from the lead is fused with the one from the bat angle (precision-weighted, on the circle), and only the fused face is mapped to spray (5.2° + 1.36 × face, 18.6° scatter), then fused with the pitch prior.
- **The back-half policy.** A face read that by itself puts the ball more than 90° from straightaway is discarded, and he goes on with his other reads: the other face read if it is plausible, and the pitch prior in any case. Before, an implausible bat read made him stand still. That happened on 39% of reads at grade 50, so a poor reader leaned less often rather than leaning on what he did know. The lead's face is not wrapped, so a wild read cannot come back round into the forward half.

## 4. Measured (1,024 games, `sim-cli games --n 1024 --seed 5`, every fielder at one grade)

"Before" is the base (`coord/review-fixes`, `437c314`); it reproduces the bat-read derivation note's table to the digit. "Fallback only" is the revised back-half policy without the lead read (a scratch build with `swing_lead` set to `None`, not committed); "after" is the full change.

| Mode | Runs/team-game: before → fallback only → after | GB hit | LD hit | FB hit | PU hit |
| :--- | :--- | :--- | :--- | :--- | :--- |
| No jump | 5.57 → 5.57 → 5.57 | .412 → .412 → .412 | .726 → .726 → .726 | .133 → .133 → .133 | .046 → .046 → .046 |
| Anticipation 20 | 5.59 → 5.53 → 5.52 | .411 → .411 → .411 | .719 → .721 → .720 | .135 → .135 → .135 | .044 → .047 → .047 |
| Anticipation 50 (default) | 5.52 → 5.55 → 5.55 | .411 → .411 → .410 | .720 → .721 → .719 | .134 → .134 → .134 | .042 → .046 → .047 |
| Anticipation 80 | 5.54 → 5.57 → 5.49 | .414 → .414 → .412 | .723 → .722 → .720 | .134 → .133 → .133 | .045 → .047 → .045 |
| Anticipation 160 (lab) | 5.36 → 5.36 → 5.47 | .405 → .405 → .406 | .717 → .717 → .727 | .131 → .131 → .129 | .044 → .044 → .045 |

Because 1,024 games cannot separate a few hundredths, the same three builds were also run over 4,096 games (`--n 4096 --seed 5`):

| Mode | Runs/team-game: before → fallback only → after | GB hit before → after |
| :--- | :--- | :--- |
| No jump | 5.55 → 5.55 → 5.55 | .410 → .410 |
| Anticipation 20 | 5.57 → 5.57 → 5.56 | .411 → .412 |
| Anticipation 50 (default) | 5.55 → 5.58 → 5.58 | .411 → .413 |
| Anticipation 80 | 5.54 → 5.55 → 5.51 | .412 → .412 |
| Anticipation 160 (lab) | 5.47 → 5.47 → 5.49 | .408 → .408 |

**What this says.**

- **The grades still do not rank on the 20–80 scale.** Over 4,096 games, after the change: no jump 5.55, grade 20 5.56, grade 50 5.58, grade 80 5.51 runs per team-game. Grade 80 is the best of the three, by 0.04–0.07 runs, but grade 50 is no better than grade 20 or than no jump. Paired whole-game comparisons at this size still carry a few hundredths of noise, so the honest reading is that grade 80 may gain a few hundredths of a run per team-game and grades 20 and 50 gain nothing. This is what the information measurement predicted: the lead takes a grade-80 fielder's spray scatter from 22.18° to 22.06°, and a grade-50 fielder's from 22.39° to 22.36°. No retuning was done to make them rank.
- **The lead is worth a few hundredths at grade 80 and nothing below.** Fallback only → after: grade 80 5.55 → 5.51; grade 50 5.58 → 5.58; grade 20 5.57 → 5.56.
- **The fallback policy changes little, and if anything costs.** Before → fallback only: grade 50 5.55 → 5.58, grade 80 5.54 → 5.55, grade 20 5.57 → 5.57. A fielder whose bat read is implausible now leans on the pitch prior instead of standing still. The earlier finding (agent-models §5.10, "Why the swing as well") was that the location prior alone is worth no more than waiting, because location moves the expected direction only about 7° against 22° of scatter, and that holds here: the extra leans on the prior gain nothing. Grade 160 is identical to the digit before and after the fallback, as it must be: its 6.6° bat read is almost never implausible, and identical draws mean identical games.
- **Grade 160 (lab)** improves no further with the lead (5.47 → 5.49 over 4,096 games, within noise). Its bat read is already sharp, and the lead's 10 ms adds little to it.
- **The default run environment** (grade 50) moves 5.52 → 5.55 at 1,024 games and 5.55 → 5.58 at 4,096 games, through the fallback policy.

## Not done

- No retuning. The jump rule (`JUMP_ZONE_DEG`, `JUMP_FULL_SIGMAS`, `jump_gain`) and the read errors were not adjusted toward a run total.
- The lead–face fit (and the older spray–face fit) are marginal fits that ignore pitch location, as before. The lead's dependence on location (inside pitches are met out front, so the bat starts earlier) is part of the small overlap with the location prior already noted in §5.10.
- If anticipation is to matter on the 20–80 scale, the information has to exist in the world first. In this sim the batter's timing varies by only 4–8 ms and pitch type does not move it, so there is nothing for a fielder to read. MLB batters pull offspeed pitches because they are fooled on time. A batter model whose timing depends on the type and the sequence would make the called pitch an honest pre-pitch cue. The sim anchors the batter's speed expectation on the previous pitch, but the speed change from the previous pitch shows no spray effect (r = 0.03), and a misread type, which does pull the ball 8° more, happens on 0.8% of balls in play. That is batter-model work (pa.rs, bb_agents), outside this change.
