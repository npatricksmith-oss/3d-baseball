# Review: the runner's late-flight catch read and the tagging runner's relook (rev-latecatch, 2026-11-23)

Reviewer: rev-latecatch (Claude Opus, high effort). I wrote none of this change.
Range: `main` `3de8698` → `work/late-catch-read` `274958f` (sim code as `1a78964`). Files: `sim/crates/bb_game/src/live.rs`, `docs/agent-models.md` §6.3, the [late-catch note](../calibration/2026-11-23-late-catch-read.md), the two new rows in [the ledger](../metrics-ledger.md) §3.

**Verdict.** Invariants hold, and the read is a clear improvement: no NaN, no outfield home run or ball off the wall reads more catchable than before, and pop-up landings are about twice as accurate. There are two P2s:

- a runner on third (or second) freed from his tag-up leaves the bag on deep flies that are then caught, and loses the sacrifice fly a real runner would take;
- the seen spin axis is unreliable when his looks straddle a pop-up's apex, the late window at the railing.

Neither blocks a merge if the Coordinator accepts them as follow-ups. P2-1 is the one a watching fan would notice.

## Findings

### P2-1: A runner on third freed from his tag-up gives up the sacrifice fly

`live.rs:3889–3896`, `fly_relook`, the `tagging` branch.

**What happens.** At a catch score of 0.2 or less, a tagging runner "decides as at contact" (`runner_decide_with_contact`). That decision only weighs going now against staying on the bag. It never weighs keeping the tag-up, which is worth the most to a runner on third with fewer than two out.

**Probe.** Seeds 0–299 and 1000–1299, average fielders: 522 tagging runners were freed.

- 37 of them were on balls that were then caught: 18 from third, 15 from second, 4 from first.
- Of the 18 from third, 4 scored.
- 8 of the 14 who did not score were on flies of 30–41°, freed 3.5–5.0 s after contact while still standing on third (s = 82.3 m) with reads of 0.03–0.18. Examples: 139/147 (0 out, 39.7°, freed at 5.0 s), 1183/380, 1125/151, 1118/95, 197/224, 1105/88.
- From second: 15 freed on balls that were then caught lost the tag to third, e.g. 1078/25 (0 out, 35.7°).

**What it gains.** Freeing a runner on third gains almost nothing when the ball drops. A tagging runner on third already goes at `no_catch_read` (0.25 s after the touch), and from third the race home after a deep drop is easy. Of the 215 freed from third on balls not caught, the 100 who did not score were mostly on foul flies (65 at 30–45°, 20 at 45°+).

**Counts.**

| Caught flies, 0–1 out, runner on third | Main | Step 1 | Step 2 |
|---|---|---|---|
| Worker's seeds | 248/532 advanced | 251/527 | 245/534 |
| My fresh seeds 2000–2299 | 133/273 (48.7%) | – | 124/264 (47.0%) |

Both sets move down. Each move is within noise, but the direction matches the mechanism.

**Would a real runner do this?** No. With fewer than two out, a runner on third on a fly to the outfield stays on the bag until the ball is caught or drops. Leaving early only pays when a catch would be too shallow to score after it.

**Fix.** Free a runner only when leaving now is worth more than staying on the bag with the tag-up's option. From third, keep him planted whenever a plausible catch is deep enough for a tag to beat the throw home: the hold logic's own third-base test, "far enough out for an advance from rest to compete with the throw home". Better, compare P̂(catch)·ΔRE(tag) + (1 − P̂)·value(go at the drop) against value(go now). The same applies, more weakly, from second with none out.

### P2-2: The seen spin axis is unreliable when his looks straddle a pop-up's apex

`live.rs:417–445`, `ContactLook::seen_axis`.

**The math.** He measures the push from his first look (0.5 s) to now, and takes the axis as J × push.

- For a constant spin, the integrated push is ω̂ × J, where J = Σ k·û·h. On a pop-up, the rising and falling legs cancel in J's vertical part.
- Once now is about as far past the apex as his first look was before it, J is mostly the small horizontal drift.
- ω̂'s component along that drift cannot be seen at all.
- The part of the push across J is then mostly the two looks' read errors.

The absolute `axis.norm() > 1e-9` guard never catches this.

**Probe.** Synthetic launches, the runner on first, his own read draws on both looks. Typical-size spin ×0.6–1.6, the axis rolled up to ±60° and pitched up to ±30° toward gyro. 300 balls per band.

| Pop-ups 60–88° | Looks | Flipped (> 90°) | Off by > 45° |
|---|---:|---:|---:|
| Looks before the apex | 1,225 | 21 (1.7%) | 113 |
| Up to 0.6 of the rise past it | 821 | 6 (0.7%) | 21 |
| Straddling it (0.6–1.4 of the rise) | 936 | **152 (16%)** | **288 (31%)** |

The noise-free straight-up ball (spin +X) shows the same thing: its axis is right at 2.0 and 3.5 s and comes out −X at 5.0 s. Liners (5–20°) flip on about 8% of looks throughout, and the axis is more than 90° off in over 10% of second looks.

**Net effect.** It is still a win on pop-ups: landing error median 1.7 m against 3.2 m with the typical axis, 90th percentile 4.7 against 9.3, and the seen axis is worse by more than 2 m on 74 looks against better on 945. At 20–40° it is neutral (median 1.7 against 1.7; p90 5.3 against 5.1; worse on 130, better on 150).

**Why it matters.** The straddling window is the last 1–2 s of a pop-up: the railing decision the change was built for. The named-play tests pass only because they use noise-free looks.

**Fix.** Keep the last look as well as the first, and estimate the axis by least squares over consecutive look intervals: each interval gives push_i = ω̂ × J_i, and the J_i of the rising and falling legs point different ways, so the system is well conditioned exactly where the whole-interval estimate is not. At the least, test conditioning relative to size (|J × push| against |J|·|push| and the read error across J), and keep the previous or typical axis when it is poor.

### P3-1: The seen axis is recomputed in each of a look's projections; its cost grows with the interval

`live.rs:468` (`project`), called eight times by `runner_catch_paths` and once more for the decision.

Measured on a 35° fly (release build, `zz_probe_time`):

| Look at | One `seen_axis` | ×8 | One look, with | One look, without |
|---|---:|---:|---:|---:|
| 1 s | 3.8 µs | 30 µs | 1,304 µs | 1,272 µs |
| 4 s | 17 µs | 137 µs | 814 µs | 678 µs (+20%) |

So the note's once-per-look fix is right, and it changes no game: compute the axis once in `observe_ball` (or store it in `ContactLook`). The whole-interval integration from the first look could also be carried forward look to look; with P2-2's per-interval fix that comes for free. The ledger's figures (+3% [+2%] for step 1, +3.5% [+4.3%] for step 2) are within the 1–2% run-to-run timing noise visible between rows: the step-1 row's "before" (4,793) differs from the previous row's "after" for the same sim code (4,856). So "about 6–7% in all" is a fair statement.

### P3-2: A freed runner who decides to stay stops tagging and stops looking

`live.rs:3893–3896`.

When `runner_decide_with_contact` keeps him at his base, `tag_up` stays false and `fly_look` becomes `None`. If the ball is then caught, he decides from rest (the "planted without having set up a tag-up" path), without the tag-up's anticipated start (`set_start`). If he is still on his way back, he never looks again.

It happened to 9 runners in 600 games (8 from second, 1 from third).

**Fix.** On a stay decision, keep `tag_up = true` and `fly_look = next`.

### P3-3: Free and re-tag can alternate

12 runners in 600 games went free → retag; 2 of them (145/51, 1023/24) went free → retag → free → retag. Each move is a 0.5 s look with fresh draws crossing 0.2 and then 0.8. Real runners do occasionally start and come back, so this is not wrong in itself. If P2-1's fix weighs the tag-up's value, some hysteresis comes with it.

### P3-4: Spec text in agent-models §6.3

`docs/agent-models.md:801`, "He keeps watching while it is up".

- The old *Why:* ("a runner who read a home run at 0.5 s ran flat out…") now follows the new tagging *Why:*, away from the going-on relook it explains. Move it back after the holding sentences.
- "A relook never sends him on again from a hold" is still true word for word, but hold → sure → tag → free → go is now a path (seen in the probe: 166/100). Say so.

### P3-5: The tests check the claim but only one side of it, and only with noise-free looks

The named-play tests (`late_read_scores`, the two `a_runner_watching_…` tests) are independent of the implementation: "a fielder standing where it comes down, or running in on time, should not read as unlikely" is a physical claim. Turning the five launches through seven sprays does vary the spin axis against the field and the heading. Three gaps remain:

- **The floor at the wall is weak.** At the railing it is 0.2 (the advance cut-off), set below what the build gives there (0.39–0.50). A regression to 0.21 would pass while turning a tag-up into a hold.
- **Nothing pins the other side.** No test checks that the same balls with no fielder within reach, or a ball 5 m into the seats, read low late in flight. Today the scan's "0.8+ on balls that dropped" (167 → 173) is the only guard against a read that is now too generous.
- **No test uses noisy looks or checks the seen axis itself.** Nothing compares it with the true spin's direction across the flight, and nothing covers P2-2's regime. All five plays are 55–77°.

Add a mirror test (fielders out of reach, or a ball into the seats: score ≤ 0.2 from some time on) and a noisy seen-axis property test by flight phase.

### P3-6: Ledger wording on noise

"Each arm is about 2 points of binomial noise on ~575 chances, so both agree with the note" is correct but undersells the uncertainty. The difference between two arms has about 2.9 points of noise (second to home) and 1.8 points (first to third), so neither +2.2 nor +1.3 is distinguishable from zero on 600 games.

My fresh set (seeds 2000–2299, 300 games each, main against the branch):

| | Main | Branch |
|---|---|---|
| Second to home, 0–1 out | 158/329 (48.0%) | 173/323 (53.6%) |
| First to third, 0–1 out | 246/757 (32.5%) | 253/767 (33.0%) |

Pooled with the worker's 600 games, second to home with 0–1 out goes from 44.9% to 48.7% (+3.8, about 1.6σ). The direction holds and the size matches the runner-start note's +3.8. Suggested wording: "within noise on 600 games; a fresh 300-game set moved the same way (+5.6)".

## Answers to the job's questions

1. **Invariants 1–3.** Hold.
   - The seen axis uses only `ContactLook.motion`, which is the truth after `noisy_ball_motion` with his own draws (`observe_ball`, `live.rs:3697–3712`).
   - `SeenLook.v` is that noisy velocity, and its covariance comes from his own `scales`. Nothing reads the ball's true spin, acceleration or path.
   - "Visible momentum" is `fd.vel`, the same visible fielder motion the read used within 5 m before. It is noise-free at any range, as the spec says; that was the existing model's choice, not something this change introduced.
   - No new `Purpose`. Tagging runners' looks draw `RunnerPerception` sub-indices `FLY_LOOK_SUB + n`, fresh counters that are unique per look, so no existing stream's draw order changes. Nothing here touches `bb_agents` or `motor`.
2. **Determinism and math.** Holds. The new code uses only `m::sqrt` and `m::hypot` (and `m::cos`/`m::sin` in tests), `Vec3::normalized` (sqrt) and `Mat3::inverse`. There is no `mul_add`, no SIMD, no hashed iteration. The workspace suite, which includes the replay and digest tests, passes (Verification).
3. **The seen-axis estimate.**
   - No NaN in 8,056 noisy looks or the degenerate cases: `normalized()` returns zero for a zero vector, and `inverse()?` returns `None`.
   - A second look 1 ms after the first returns `Some` with an arbitrary axis. Harmless: the remaining push estimate is the typical push, and the projection is almost the same.
   - A first velocity read exactly vertical returns `None`, which is correct.
   - Near the apex it is the problem in P2-2. On low liners, and on balls with only two looks, the second look's axis is often far off (liners: median 34°, p90 101°), but the landing effect is small (median 0.8 against 1.0 m).
4. **The graded wall.** It cannot stop a runner on an outfield home run or a ball off the wall. A fair ball's path ends at the fence plane (`ground.rs:484`), so points past the outfield fence are never sampled.

   Over 3,000 synthetic flies, runner on first, fielders at their spots:

   | Ball | Read at 0.5 s | ≥ 0.8 on any look |
   |---|---|---|
   | Home runs (330) | 0.10 under the new rule and under the old | none, either rule |
   | Off the wall (56) | 0.42, against 0.45 under the old rule | none, either rule |
   | Foul into the seats (530) | 0.33 mean, against 0.05; ≤ 0.2 for 105, against 503 | none |

   The ball off the wall reads slightly lower because the wall margin now also caps points just in front of the fence early in flight, which makes runners go a little more, not less. The foul fly into the seats now reads as a possible catch for the first 2 s because the curve allowance (½·3·t², 24 m at 4 s) puts the seats within reach. Runners hold partway on those balls instead of going, which changes no outcome (the ball is dead).
5. **The tagging relook.**
   - **Leaving before a catch.** 37 of 522 freed runners were on balls then caught. 2 were doubled off, both on liners freed at 1.0 s (1136/197 at 11.8°, 1019/65 at 11.1°): mistakes a real runner can make. The sacrifice-fly cost is P2-1.
   - **Stranding a planted runner.** The `awaiting_call` comment is accurate. I found no case where a planted tag-up runner was stranded, and the pending-catch property test passes.
   - **The base relook.** `fly_relook` runs before `runner_relook` in `move_runners` and only while the ball is `Free`. The base relook (`runner_relook`, `throw_watch`) acts on secured and released counts, so the two do not overlap in time.
   - **A freed runner who stays.** He loses the tag state (P3-2).
6. **Would a real runner do this?**
   - **Yes:** from second, freed on balls that dropped, 184 went for third and 64 scored. 272/140 (0 out, 22.4°, freed at 3.0 s reading 0.08) is the runner taking off when he sees the ball will fall in.
   - **Yes:** the two liner double-offs.
   - **No:** a runner on third leaving the bag on a deep fly with fewer than two out (P2-1, 139/147).
   - **Sometimes:** a runner going and coming back twice on one ball (P3-3).
7. **Tests.** They test the claim, not the code. Their gaps are in P3-5.
8. **The ledger.** "Within noise" is fair (P3-6), and a fresh set agrees in direction. The time figures are fair, and once per look is the right fix (P3-1).

## Verification

- `cargo test -p bb_game --lib` with the five new or touched tests (the two `a_runner_watching_…`, `a_tagging_runner_leaves_his_bag…`, `random_runner_layouts_with_a_catch_call_pending…`, `a_runner_holding_partway_goes_back_to_tag…`), after `touch live.rs`, in `sim/target`: 5 passed (job 103).
- Full `cargo test --workspace` on `274958f`, after a touch, in `sim/target`: 543 passed, 0 failed (job 109), the worker's count.
- Probes, in a copy of the tree built into `sim/target/review-probes/`; tests and example sources are in `sim/target/review-probes/tree/crates/bb_game/`, nothing committed:
  - `zz_probe_seen_axis` (P2-2, Q3): 1,200 synthetic balls in four launch-angle bands, noisy looks, breakdown by flight phase, plus degenerate cases. Logs: `probeAB.log`, `probeA2.log`.
  - `zz_probe_wall` (Q4): 3,000 flies, the new rule against the old (a toggle that skips points past reach), at each 0.5 s look.
  - `zz_probe_time` (P3-1).
  - `examples/zz_rev_tag.rs` (P2-1, P3-2, P3-3, Q5, Q6): a log hook in `fly_relook` and `read_contact`, seeds 0–299 and 1000–1299. Output: `tag.csv`.
  - `examples/zz_rev_rates.rs`: the worker's `zz_lc_rates` taking a seed range. Seeds 2000–2299 on the branch and on a main copy (`tree-main`, `live.rs` from `main`). Outputs: `rates-branch-2000.txt`, `rates-main-2000.txt`.
- Read: the full diff, `believed_flight`, `read_cov`, `noisy_ball_motion`, `FlightEnv::accel`, the tag-up and catch-call paths in `move_runners`, `runner_decide_with_contact`, and `batted_path`'s fence and stands handling.

## Worker follow-up

worker-2, 2026-11-23, `work/late-catch-read` `70347ab`..`a9508e3`. Measurements in [late-catch note §7](../calibration/2026-11-23-late-catch-read.md#7-the-review-fixes) and the ledger's "Late-catch review fixes" row.

- **P2-1.** Fixed (`d02bac4` spec, `a9508e3`). A runner who reads the ball as dropping leaves the bag only when going now is worth more than staying with the tag-up's option: P̂·V_back + (1 − P̂)·V_now against P̂·V_tag + (1 − P̂)·V_drop, each valued as his advance decision values a race, one base or two on (`tag_up_worth_leaving`).
  - The weighing also governs his first look at 0.2 or less. On 139/147's ball over 40 draws, a first look of 0.07 had the runner on third off the bag at the catch.
  - A projection out of play counts the same staying or going.
  - Fixture: `a_runner_on_third_keeps_his_tag_up_on_a_deep_fly`. It fails with the weighing off.
  - Your counts on seeds 0–299 and 1000–1299:
    - Caught flies, 0–1 out, runner on third advanced: 245/534 → 260/523 (main 248/532); on seeds 2000–2299, 124/264 → 128/265 (main 133/273).
    - Freed tagging runners: 522 → 225. On balls then caught: 37 (18 from third) → 5 (none from third); the 2 appeal outs are your two liners, freed at 1.0 s.
- **P2-2.** Fixed (`00ae9b3`). The axis is now a least-squares fit over the intervals from his first catch look to each later one.
  - Each interval is weighed by the looks' read error plus the push's error from the unseen spin size, `SEEN_SPIN_SIZE_SPREAD` = 0.3, which is the contact model's spread (0.29 at 25–50°, 0.20 at 50°+).
  - The typical axis is a unit-spread prior.
  - Property test by flight phase with noisy looks: `the_seen_axis_holds_across_the_apex_under_read_noise`. Straddling pop-up looks flip on 4 of 930 on my metric (the axis's part across the velocity now) and on 15 of 930 on yours, against your 152 of 936.
  - Median landing miss on pop-ups: 1.65 m seen against 3.40 m typical. No band is worse than the typical axis.
- **P3-1.** Fixed (`70347ab`). Computed once per look (`ContactLook::axis`). The digests of 40 games are unchanged by that commit. Time per game 5,129 → 4,954 [5,257 → 5,236] ms over all the fixes.
- **P3-2.** Fixed: on a stay he keeps `tag_up` and `fly_look`.
- **P3-3.** No separate hysteresis needed: free → re-tag fell from 12 to 1, and free → re-tag twice from 2 to 0.
- **P3-4.** Fixed (`d02bac4`).
  - The old *Why* is back after the holding sentences.
  - The hold → tag → free path is stated.
  - The tagging runner is a sub-item of its own.
- **P3-5.** Fixed. Mirror tests `a_runner_watching_a_ball_nobody_can_reach_reads_it_dropping` (≤ 0.2 from 2 s on) and `a_runner_watching_a_ball_into_the_seats_reads_no_catch`. A ball 5 m or more into the seats reads ≤ 0.2 only over its last 1.5 s: at 2 s, 3.6 s from coming down, the 19 m curve allowance still reads 0.35. The railing floor is 0.4 (`LATE_READ_RAIL_FLOOR`; the build gives 0.43 at the least).
- **P3-6.** Fixed: the step-2 ledger row and note §5 now carry your wording and fresh-set figures.
- **Tests and runs.** After a touch: `cargo test --workspace` 547 passed, 0 failed. `sim-cli` rebuilt. Standard run 4.18 → 4.12 [4.43 → 4.43], rules warnings none.

## Resolution

Coordinator, 2026-11-23. Every finding fixed by worker-2 (`70347ab`..`fe34585`; Worker follow-up): P2-1 (a freed tagging runner leaves only when going now beats staying with the tag-up's option; freed runners on caught balls 37 → 5, none from third; runner on third on caught flies, 0–1 out, 245/534 → 260/523), P2-2 (least-squares seen axis over the look intervals; straddling pop-up flips 152/936 → 15/930 on the reviewer's metric), P3-1 to P3-6. No re-review: each fix has a test that fails without it (`a_runner_on_third_keeps_his_tag_up_on_a_deep_fly`, `the_seen_axis_holds_across_the_apex_under_read_noise`, the two mirror tests), and the worker reran the reviewer's own probes; the Coordinator reran the tests and the standard run before merging. Left open: 25–50° double-offs still 0 (MLB about 8 per 600 games); second to home with 0–1 out about 3 points under MLB (Q38, Q10 (a)); the spin's size is not seen.
