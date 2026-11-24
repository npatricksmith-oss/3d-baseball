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

The rule's reason, "an outfielder has nobody behind him", is that a ball he dives for and does not hold gets by him. For a ball in the air that holds: only a catch is an out, and a liner off his glove is still a hit, so a dive that will probably not catch it trades a single for extra bases. For a ball on the ground it does not: the ball is going past him anyway (`dive_gains` has already said he cannot get it on his feet), and a knock-down stops it as surely as a clean pick. §5.4 already says so ("bobbles that stay close can be re-fielded"), and the sim's deflection keeps a third of the ball's speed plus a push of 0.15 of it, so a 16 m/s ball leaves the glove at 5–7 m/s and stays near him. A real outfielder lays out to keep a hard grounder in the gap from reaching the wall. At 2.11 m his chance of a glove on it (within the 0.12 m rim) is 1 − exp(−½(0.12/0.063)²) = 0.84.

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
