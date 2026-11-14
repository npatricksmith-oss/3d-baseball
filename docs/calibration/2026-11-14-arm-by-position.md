# Arm strength by position (2026-11-14)

**Change.** `FielderAttrs::average(pos)` gave every infielder 36 m/s and every outfielder 39 m/s of grade-50 `ArmStrength`. It now takes the position's mean from Statcast's arm-strength leaderboard (`bb_game::play::ARM_MEAN_MPH`). Pitcher and catcher have no entry in that table and keep 36 m/s (`ARM_UNSOURCED_MPS`, [E]).

**Definition (agent-models §10, "Arm strength by position"); the mapping to a set throw is an estimate [E], not settled (Codex review, P2).**
- Statcast: per player, the mean speed of his max-effort throws, which Savant and MLB's glossary define as his hardest throws: the top 1% for a first baseman, 5% for the other infielders, 10% for an outfielder (minimums 100 / 75 / 50 throws), at the release point. Outfielders' running starts are part of it. (Fetched 2026-11-14 from baseballsavant.mlb.com/leaderboard/arm-strength and mlb.com/glossary/statcast/arm-strength.)
- Sim: `ArmStrength` is "max throw speed"; every set throw is released at it (`THROW_EFFORT` 1.0, §5.5), hurried throws less.
- Both describe a hard throw, not a routine one, so the conversion is the unit change only (× 0.44704 m/s per mph).

| Position | mph | m/s | was |
|---|---|---|---|
| 1B | 79.3 | 35.45 | 36 |
| 2B | 79.1 | 35.36 | 36 |
| 3B | 84.9 | 37.95 | 36 |
| SS | 86.1 | 38.49 | 36 |
| LF | 87.4 | 39.07 | 39 |
| CF | 89.6 | 40.05 | 39 |
| RF | 90.7 | 40.55 | 39 |

**Still an estimate.** That a hard-throw mean in each position's own throwing context equals a common set full-effort throw (the methodology's position equivalencies suggest first base could be about 2.7 m/s low and the outfield 1.3 to 1.8 m/s high as set-throw limits; not measured for stationary throws, so not applied) (a set throw is probably a little slower; outfielders' running starts are not modelled, so the outfield values may be a few mph high); the catcher and pitcher value; the spread between players of one position (unchanged, unsourced).

**Effect.** See the [ledger row](../metrics-ledger.md). Every digest changed. The two 300-game sets traded values (5.53/5.40 → 5.40/5.53); their mean did not move. Nothing was tuned.

**Test.** `play::attr_tests::arm_strength_follows_the_published_positional_means` checks each position against the published mph × 0.44704, worked by hand in its comment. The shortstop timing helper in `live.rs` uses the shortstop's 38.490 m/s.
