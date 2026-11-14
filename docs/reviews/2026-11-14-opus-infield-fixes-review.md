# Opus review: infield alignment review fixes (`4d3bf1b`)

- **Reviewed:** `git diff fb6dbf7 4d3bf1b`. That is `work/infield-review-fixes` (`41f7a3a`, written by the Gemini analyst) merged into main `fb6dbf7`, on `work/infield-fixes-on-main`.
- **Claimed:** fixes for the findings of [the Gemini infield alignment review](2026-11-14-gemini-infield-alignment-review.md) (P2-1, P2-2, P2-3, P3-1).
- **Reviewer:** Claude Opus 5.5, on `work/review-infield-fixes`. I wrote the speed-for-accuracy rule that question 3 involves, and finding P2-1 is against my own code.

**Verdict: not fit to merge as it stands.** The first baseman now breaks for grounders that he cannot be the one to field. When the pitcher or another infielder fields the ball, nobody covers first (P1-1). On seeds 0–299 that adds about 80 infield hits. The other fixes are sound or need only small changes.

## Findings

### P1-1. The first baseman leaves first base uncovered (`live.rs:1603`, `live.rs:1425`, `live.rs:1949–1985`)

**What the code does.**
- `assign_among` now puts the first baseman (index 2) in the secondary grounder break: `for i in [2usize, 3, 4, 5]`. He breaks whenever his own intercept is finite and lies inside `INFIELD_R`, which is now 51.6 m.
- Nothing checks which side the ball is on, or who will take first.
- `cover_roles_after_chasers` fills first base from `[2, 0, 3]`:
  - the first baseman is taken, because he is chasing;
  - the pitcher is taken, because he is the primary on a ball he fields;
  - the second baseman is taken too, because the new reservation step keeps his `Cover(2)`.
- So first base stays empty.
- A secondary chaser keeps re-aiming at his own prediction while the ball is free (`live.rs:1425`). Nothing sends him back to his bag once someone else has the ball.

**Evidence: a probe over seeds 0–299** (built for this review, untracked). It counts fair grounders on which the batter reached first safely, nobody threw, and no fielder was within 6 m of first when the ball was secured. Columns are the fielder who secured the ball:

| | P | C | 1B | 2B | 3B | SS | Total |
|---|---|---|---|---|---|---|---|
| fb6dbf7 | 0 | 0 | 1 | 9 | 22 | 30 | 62 |
| 4d3bf1b | 47 | 1 | 5 | 25 | 34 | 45 | 157 |

- All 47 pitcher cases were fielded inside 45 m of home.
- `run_env_diag` (300 games, seeds 0–299) agrees:
  - infield hits 854 → 934;
  - infield hits fielded by the pitcher 56 → 103;
  - infield hits with no throw 257 → 539.
- The standard run's grounder hit rate goes .327 → .334 [.333 → .335], already against MLB's ≈ .24.

**One play, seed 250 play 109** (left-handed batter, so the first baseman starts at his Standard spot (23.0, 29.9)).
- The ball goes toward the third-base side, and the pitcher secures it at (−5, 14) at 2.01 s.
- The first baseman has broken across the infield. At 2.13 s he is at (16.2, 29.3), 10.4 m from first, and he keeps running to (−17.5, 31.6) by 6.9 s.
- The second baseman goes to second.
- The batter touches first at 4.66 s. The pitcher held the ball 2.65 s before that and never threw.

**Is it something a real first baseman does?** No.
- A first baseman breaks to his right only when the pitcher can cover.
- On a ball to the left of the mound he goes to the bag.
- If the pitcher or second baseman calls the ball, he retreats to the bag.

The new test `first_baseman_breaks_on_right_side_infield_grounder` checks only the right-side geometry, where the pitcher does cover. No test covers the other side.

**Fix suggestion.**
- Give first base the same protection the change gives second. The first baseman breaks only when another fielder (the pitcher, not primary and not chasing) can be assigned `Cover(1)`.
- A secondary chaser who sees another fielder secure the ball returns to his base.
- Then measure again.

The ledger row and the calibration note §6 saw part of this ("`P` IFH `19 → 29`") and called it seed divergence. It is a structural effect.

### P2-1. Fixture 23-109 pins a lost double play caused by speed for accuracy (`live.rs:3084`; `tests/throw_choices.rs:105`)

**Question 3.** I traced the fixture on the infield parent `41f7a3a` and on the merge `4d3bf1b`.
- **Same choice, same estimates.** On both, the first baseman chooses the throw home with the same estimates: runner 3.916 s, ball 3.481 s. The second out at first is estimated at p 0.106 (ball 4.817 s, batter 4.668 s), with the pitcher covering first at 4.477 s.
- **On `41f7a3a`:**
  - the throw goes at full effort and reaches the catcher at 3.426 s;
  - the catcher's own read at release is p 0.457;
  - he throws at 3.710 s;
  - the pitcher has the ball on first at 4.626 s, out by 0.04 s (calls `[4, 1]`).
- **On the merge:**
  - the holder's p 0.106 is below the 0.3 that makes a feed go at full effort;
  - speed for accuracy eases the throw to effort 0.85, the least that keeps the 0.25 s margin on the first out;
  - the ball reaches the catcher at 3.610 s, 0.18 s later;
  - the catcher makes no throw (calls `[4]`).

**So the defect is in the speed-for-accuracy piece, not the infield fixes.**
- The effort rule (`Some(o) if o.continuation.is_some_and(|(p2, _)| p2 > 0.3) => 1.0`) ignores a second out it judges unlikely. It then makes that out impossible by easing the throw.
- A first baseman throwing home with the bases loaded and one out throws firmly, so the catcher has a chance at first.
- The holder's estimate also runs pessimistic here. It said 0.11; the play was made by 0.04 s; the catcher's own read was 0.46. Part of that is the throw-home estimate (3.481 s against an actual 3.426 s), part the catcher's release and throw model. Both predate this change.

**What to do.**
- The infield merge should not freeze `vec![4]` as expected behaviour. Assert only the lead force (`calls[0].base == 4`), or keep `[4, 1]` once the effort rule is fixed.
- The effort rule should throw at full effort whenever the chosen force has a continuation, or whenever easing would lower that continuation. That is my code, and I can take the fix as a separate change.

### P2-2. One radius now serves four purposes, and the rules spec and scorer were not updated (`live.rs:203–204`, `1619`, `1837`, `3186`, `3390–3394`; `docs/rules-spec.md:243`; `bb_stats/src/scorer.rs:11`)

`INFIELD_R` became `MAX_INFIELD_DEPTH_M + INFIELD_DEPTH_MARGIN_M` = 51.6 m. Gemini P2-2 was about the secondary break gate, but the same constant also sets:
- **The settle rule.** It is now a rules constant derived from the alignment table, so adding `shift=1` positions would move when "time" is called. 51.6 m reaches about 4 m onto the outfield grass; the dirt arc is 95 ft from the rubber, about 47.3 m from home.
- **Backup assignment** (`outfield_hit`).
- **The runner's read of a fly.** A blooper landing 45–51.6 m from home is no longer "deep", so with fewer than two out the runner decides at once instead of going partway.

None of these side effects is described or measured. The documents now disagree:
- rules-spec §5.8 still says "within 45 m of home [C]";
- the scorer's `DEFAULT_OUTFIELD_RADIUS_M = 45.0` still says it matches the live-play settle radius;
- this breaks CLAUDE.md invariant 7 (if code deviates from the spec, update the spec in the same change).

**Fix suggestion.** Give the secondary-break gate its own named radius, derived from the Savant depth. Leave the settle and scorer boundary at 45 m. Or else move the settle rule, scorer and rules spec together and measure them.

### P2-3. The `Cover(2)` guard reads a teammate's decision the instant he makes it (`live.rs:1609`)

**Question 1.**
- A `Cover(2)` middle infielder may break only `if self.f[other_mi].role != Role::Chase`.
- His partner's role was set a few lines earlier, in the same tick, from the partner's own private intercept.
- A real partner's break becomes visible only after a reaction time, or is agreed by a pre-pitch sign.
- This is the same class as the Codex live-play review's P1-1 (decisions consume another fielder's private state). It adds one more location to that list.
- It is not an outcome roll and reads no world state outside the fielders.
- The relay-man choice now reads `self.home`, the public pre-pitch spots, which is fine.
- There are no RNG or draw-order changes. The one new parameter, `INFIELD_DEPTH_MARGIN_M`, is named and tagged [E].

**Recommendation.** Record this location under the Codex P1-1 resolution so the communication fix covers it.

### P3-1. Three new tests cannot fail on the defect they name (`live.rs:7303–7406`, `play.rs:931–949`)

- **`carry_non_second_force_dp_uses_carrier_post_touch_setup_clock`** asserts `ball_at_1 > carry3.ball_at + 0.5`. The 38.8 m throw from third to first alone takes more than 1 s, so the bound also holds for the non-carry branch. It cannot detect the branch confusion it is named for. A bound would need the carrier's turn, footwork and set-up from his own state, computed independently.
- **`implied_w`** checks in both carry tests divide the value back by `p_out² · p2²` and compare with 0.9 or 1.0. That restates the formula, not an independent expectation. The other part of `carry_double_play_at_second_…` is a real behavioural check: the carry wins with `continuation_weight` 1 and loses with 0.
- **`simulate_play_honors_batter_hand`** passes if anything at all changes between the two batter sides.

### P3-2. The reason for dropping fixture 13-140 no longer holds (`tests/throw_choices.rs:85–91`)

- The 23-109 test comment says 13-140 was replaced because the first baseman held near the bag with the bases loaded. P2-1 removed that.
- Replayed on the merge (fixture from `d565cf6^`), 13-140 again satisfies its original choice assertions: a force throw home with 0.354 s in hand, not a carry to first.
- It could be restored next to 23-109.
- On the merge its outcome was a throw that crossed the plate about 1.9 m high and deflected off the catcher, and two runs scored. That is throw execution noise in one play, not a finding.

### P3-3. The ledger row does not measure the merge (`docs/metrics-ledger.md:134`, `:152`)

- The row measures `8a7e938 → 41f7a3a`. Main has since moved to `fb6dbf7`, which includes speed for accuracy.
- Measured here, `fb6dbf7 → 4d3bf1b`:
  - runs 5.83 → 5.95 [6.21 → 6.14];
  - grounder hits .327 → .334 [.333 → .335];
  - double plays 1.76 → 1.83 per game (`dp_probe`, seeds 0–299).
- The row's "within seed divergence" should not cover the infield-hit rise (P1-1).
- The row should name the merge it describes, or a second row should be added at merge time.

### P3-4. The relay-man rule differs from the usual convention (`live.rs:1742`)

- The deep relay man is now whichever middle infielder's pre-pitch spot is nearer the relay spot.
- Against a right-handed batter, that sends the second baseman out on balls just left of straightaway center, which the shortstop usually takes.
- The rule is physically defensible and matches the spec text, but it uses the pre-pitch spot rather than where the fielder stands now (also public).
- Low risk; worth noting in §5.7.

## Answers to the questions

1. **Invariants.**
   - No outcome rolls.
   - No new or reordered RNG draws.
   - No `HashMap` iteration.
   - The new parameter is named [E]; `MAX_INFIELD_DEPTH_M` is derived from the data table.
   - One new instance of a teammate's private decision used as an input (P2-3).
2. **Does each fix resolve its finding, and is it what a real infielder does?**
   - **First baseman behind the runner with second occupied: yes and yes.** The positions were checked through `new_sim` in the new test. The test suite passes.
   - **`INFIELD_R`:** fixes the break gate, but reaches into the settle rule, backups and runner reads (P2-2).
   - **`Cover(2)` protection: yes.** Double plays did not fall (1.76 → 1.83).
   - **First baseman in the break list:** follows the letter of §5.7 but leaves first uncovered (P1-1).
   - **Carry beyond `carry_max_m`: yes**, and it is realistic: a first baseman takes the bag himself when the pitcher is late.
   - **Relay:** acceptable (P3-4).
   - **Lost continuation coverage:** partly restored (P3-1).
   - **`bats` on `PlayInput`, `source_commit` and spec text:** yes.
3. **Fixture 23-109:** the lost double play is a defect in speed for accuracy, not in the infield fixes (P2-1).
4. **Tests:**
   - The fixture provenance is sound: `source_commit` `d565cf6` is the commit that created both fixtures.
   - The new position, settle, relay and carry tests check expectations about play.
   - Three tests are weak (P3-1).
   - The `vec![4]` change pins a defect (P2-1).
5. **Ledger, spec, and the Gemini review's Resolution:**
   - The spec text in agent-models §5.6, §5.7 and §5.9 matches the code.
   - The rules spec and scorer do not (P2-2).
   - The Resolution claims P2-2 is fixed "consistently". In fact it introduced P1-1 and the settle-rule side effects.
   - The measurements are of the branch, not the merge (P3-3).

## Verification

All builds used `CARGO_BUILD_JOBS=2` and `taskset -c 0-7 nice`, one at a time.

**Tests.**
- `cargo test --workspace` on `4d3bf1b`: **305 passed, 0 failed, 3 ignored.**
- I did not rerun the new tests against `fb6dbf7` to confirm they failed before the fixes.

**Standard run**, `sim-cli games --n 300 --seed 0` and `--seed 1000`, with sim-cli built separately at each commit:

| | Runs / team-game | Grounder hits |
|---|---|---|
| fb6dbf7 | 5.83 [6.21] | .327 [.333] |
| 4d3bf1b | 5.95 [6.14] | .334 [.335] |

**Probes.**
- **`run_env_diag 300 0`** on both commits: infield hits, infield hits by fielder, no-throw counts.
- **`dp_probe 300 0`** on both commits: double plays.
- **Temporary probes**, untracked and copied to `sim/target/meas/review_probes/`; nothing was committed or left in the tree:
  - `zz_review_probe.rs`: replays fixtures 23-109 and 13-140 through the `throw_choices` harness and prints choices, options, effort, releases, touches and the ball track. Run on `4d3bf1b`, and for 23-109 also on `41f7a3a`.
  - `zz_ifh_probe.rs`: counts batter-safe grounders with no throw, and who was near first.
  - `zz_play_track.rs`: prints fielder tracks for seed 250 play 109.

**Not run:** `sf_probe`, and the new tests against `fb6dbf7`.

## Resolution

Open.
