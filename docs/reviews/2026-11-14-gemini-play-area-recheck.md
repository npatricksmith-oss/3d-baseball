# Play Area (A10) Resolution & Client ABI Recheck

**Reviewer:** Gemini  
**Date:** 2026-11-14  
**Branch:** `work/review-a10-fixes` (`870340f`, reviewing `0a17409..88cee6a` merged at `2eb4531` plus client ABI commits `39db36c`, `323e05e`, `bd69a04`)

---

## Per-Finding Recheck (`2026-11-14-gemini-play-area-review.md`)

### P1-1: Taper wall intruded into fair territory by `r` (`BALL_R = 3.7 cm`, `BODY_R = 30 cm`) near the foul poles and along steep tapers — **Confirmed resolved** (`54c45b0`)
- **Fix:** [ground.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_physics/src/ground.rs#L157-L209) moves the radius subtraction (`r_wall`, `r_fence`) into `Field::pieces_for` *before* applying the fair-ground clamp `if -c > taper_d { dist: -c }`. Any ball (`r = 0.036888 m`) or fielder body (`r = 0.30 m`) centered on or fair of the foul line (`c <= 0`) now has `taper.dist >= -c >= 0` everywhere, including within `r` of the foul pole or in parks with steeper tapers.
- **Test quality:** `foul_walls_never_cut_fair_ground` in [ground.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_physics/src/ground.rs#L665-L699) checks real `BALL_R` and `BODY_R` across both the reference park and a steep-taper park (`foul_clear_m = foul_taper_m = 300 ft`), plus the exact corner case `5 cm` short of the fence and `2 cm` fair (`s = 330 ft - 0.05, c = -0.02`), asserting that it meets `WallKind::Fence` and not `WallKind::Taper`. It checks the geometric invariant from [physics-spec.md](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/docs/physics-spec.md#L335-L343) §9.5 and failed on `main` before `54c45b0`.
- **New issues:** None.

### P1-2: Walk-off on `throw_out_of_play` or `ground_rule_double` emitted `RunnerAdvanced { to: 4, cause: Award }` without `RunScored` for trailing runners — **Confirmed resolved** (`3b5b62f`)
- **Fix:** Both `ground_rule_double` ([engine.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L765-L767)) and `throw_out_of_play` ([engine.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L806-L808)) now break out of the award loop as soon as `st.walk_off` becomes true, and `catch_carried_out` ([engine.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L844-L846)) does the same. Leaving the batter's base-hit value credit under `OBR 9.06(f)` to the official scorer (`bb_stats::scorer`) rather than `bb_rules`'s physical base-award events matches how walk-off hits are handled across the engine.
- **Test quality:** [scenarios.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/tests/scenarios.rs#L292-L301) adds `expect_no_events`, and [awards-014.toml](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/scenarios/awards/awards-014.toml#L1-L35) and [awards-015.toml](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/scenarios/awards/awards-015.toml#L1-L45) pin walk-off ground-rule doubles and walk-off throws into the dugout against `OBR 7.01(e)(3)` / `rules-spec` §4.1. Both failed on `main` before `3b5b62f`.
- **New issues:** None.

### P1-3: `BallReleasedFromGlove` used `r.last_base` instead of `r.at`, ignored retouch obligations on caught flies, and erased runs already scored before an infielder's first throw — **Confirmed resolved** (`9d0b17f`; see **Judgment Point 1**)
- **Fix:** [engine.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L330-L338) defines `legal(r) = if r.retouch_required && !r.retouched { r.start_base } else if r.scored_at.is_some() { 4 } else { r.at }`, computes `advanced` from `legal(r) > r.start_base`, sets `from` to `if from_pitch && legal(r) < 4 { r.start_base } else { legal(r) }`, and in `throw_out_of_play` ([engine.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L818)) emits `AdvanceCause::OnPlay` when `from == 4`.
- **Test quality:** [awards-016.toml](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/scenarios/awards/awards-016.toml#L1-L68) (runner who touched 3B and retreated to 2B has not advanced, so infielder's first throw is from the pitch), [awards-017.toml](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/scenarios/awards/awards-017.toml#L1-L51) (runner who left 1B early on a caught fly is awarded from 1B, not 2B), and [awards-018.toml](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/scenarios/awards/awards-018.toml#L1-L63) (runner who legally scored before an infielder's first wild throw keeps his run as `OnPlay`) all check `OBR 5.06(b)(4)(G)` and `5.09(b)(5)` directly and failed before `9d0b17f`.
- **New issues:** None.

### P1-4: Caught fair fly leaving the field without a throw was ruled a home run or ground-rule double — **Confirmed resolved** (`7b1bd03`)
- **Fix:** [engine.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L693) routes `Touch::OutOfPlay { .. } if lp.caught` (when `lp.throw.is_some()` is false) to `catch_carried_out` ([engine.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L834-L865)), which declares the ball dead (`DeadBall::OutOfPlay`) and awards each unretired runner one base from his legally held base (`OBR 5.06(b)(3)(C)`), respecting retouch obligations, blocking, and walk-offs.
- **Test quality:** [awards-019.toml](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/scenarios/awards/awards-019.toml#L1-L36) tests a caught fair fly followed by `OutOfPlay` with no throw against `OBR 5.06(b)(3)(C)` and failed before `7b1bd03`.
- **New issues:** None.

### P2-1: Dugouts had `wall_top == 0.0` with no roof height — **Confirmed resolved** (`9fbf77b`)
- **Fix:** [ground.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_physics/src/ground.rs#L36-L38) adds `dugout_roof_m` (`4 ft` `[E]`, [physics-spec.md](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/docs/physics-spec.md#L337-L343) §9.5) and `Field::landing_top(&w, z, r)` ([ground.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_physics/src/ground.rs#L238-L245)): balls crossing the dugout opening below `dugout_roof_m + r` enter the dugout (`0.0`), while balls crossing above it land on the seats on the dugout roof (`dugout_roof_m`). [play_area_probe.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/examples/play_area_probe.rs#L44-L48) separates `"dugout"` from `"side"`.
- **Test quality:** `a_dugout_takes_low_balls_and_the_roof_takes_high_ones` in [ground.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_physics/src/ground.rs#L701-L716) checks both a low line drive (`z < 4 ft`) and a high popup (`z > 10 m`) across the dugout opening and would have failed before `9fbf77b`.
- **New issues:** None.

### P2-2: `Touch::OutOfPlay` and `MarkerKind::BallDead` timing and 3D crossing height — **Confirmed resolved for throws and 3D coordinates; see Judgment Point 2 and P3-1 for client `dead_s` vs `dead_at` semantics** (`251109d`)
- **Fix:**
  1. In `update_thrown` ([live.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/live.rs#L2331-L2341)), a throw over a wall emits `Touch::OutOfPlay { x: s.r.x, y: s.r.y, z: s.r.z, in_flight: true }` at `t_end` (the exact instant the throw reaches the wall), ending the play immediately while keeping `over_the_wall` in `self.ball` (`settled_sent: true`) for the visual track only.
  2. `Touch::OutOfPlay` now carries `z: f64`, and [timeline.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/timeline.rs#L335-L340) writes the full 3D boundary-crossing point `Vec3::new(x, y, z)` into `MarkerKind::BallDead`.
- **Test quality:** `a_throw_over_the_stands_wall_is_dead_with_two_bases_from_the_pitch` ([live.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/live.rs#L5836-L5859)) and [play_area.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/tests/play_area.rs#L48-L62) check that the throw's `OutOfPlay` timestamp equals the wall-contact time (`(t - wall_t).abs() < 1e-6`), that `z > 1.22 m`, and that every `BallDead` marker's `p` lies within `2 mm` of `Field::ball_clearance(..., BALL_R).dist == 0` with `p.z >= 0`. Both would have failed before `251109d`.

### P2-3: Fielders and runners planned intercepts in the seats and ran into walls at full speed — **Confirmed resolved, with one physical/planning follow-up noted in Judgment Point 3 and P2-1 (new)** (`34726ad`)
- **Fix:** [live.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/live.rs#L1002-L1004) adds `beyond_reach_over_wall(field, b) = -field.body_clearance(b.x, b.y, 0.0).dist - (REACH - BODY_R)`, filtering out unreachable points (`> 0.7 m` past a wall) in `believed_field_time` ([live.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/live.rs#L1241)) and `intercept_scan` ([live.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/live.rs#L1343)). In `move_fielders` ([live.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/live.rs#L1877-L1882)), each fielder's velocity component toward the nearest wall (`into = -fd.vel.dot(w.inward)`) is capped at `allowed = sqrt(2.0 * BRAKE * w.dist.max(0.0))`.
- **Test quality:** `nobody_runs_into_the_stands_after_a_ball_out_of_reach` ([live.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/live.rs#L5897-L5946)) sweeps foul pops landing `0–12 m` into the first-base seats and verifies that no fielder ever approaches the wall faster than `sqrt(2 * BRAKE * 0.35) + 0.1 * BRAKE` when his center is within `BODY_R + 0.35 m` of the wall, nor calls a catch on a ball landing `>= 3 m` into the stands. It failed on `main` (`7.21 m/s` at the wall) before `34726ad`.

### P2-4: (1) Rolling ball convexity check, (2) glove deflections starting beyond a wall, and (3) `Field::fair_in_air` radius — **Confirmed resolved** (`9fbf77b`)
- **Fix & verification:**
  1. **Convexity of the playing field:** The author's geometric proof in `the_playing_field_is_convex` ([ground.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_physics/src/ground.rs#L733-L784)) is right — at the foul poles (`s = 330 ft`), the taper wall's inward normal points at `45° + arctan(30/130) = 57.99°` from the foul line into the field, while the fair fence (`r(theta) = 400 - 70*(theta/45°)^2`) approaches the pole at an interior angle `< 180°`, so the intersection of the half-planes and the concave-inward fence curve is globally convex! Endpoint checking `beyond(pos(hi))` on straight-line rolls is therefore exact.
  2. **Ball starting outside a wall:** `batted_path_dt` ([ground.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_physics/src/ground.rs#L426-L428)) now checks `if field.ball_clearance(s.r.x, s.r.y, r).dist < 0.0 { return over_the_wall(env, field, s, t); }` before the flight loop, verified by `a_ball_starting_beyond_a_wall_is_out_of_play` ([ground.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_physics/src/ground.rs#L718-L731)).
  3. **Airborne fair/foul label:** `Field::fair_in_air(x, y, r)` ([ground.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_physics/src/ground.rs#L120-L124)) includes the ball radius `r`, matching `bb_rules::fair_air`.

### P2-5: Missing scenarios in `scenarios/awards/` — **Confirmed resolved** (`24b58d3`)
- **Fix & test quality:** [awards-020.toml](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/scenarios/awards/awards-020.toml#L1-L35) (fair fly deflected out in flight over foul territory is a `GroundRuleDouble`, `OBR 5.05(a)(8)`), [awards-021.toml](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/scenarios/awards/awards-021.toml#L1-L63) (throw caroming off the stands wall stays live, then rolls into the dugout for a 2-base award from the throw, `OBR 5.06(b)(4)(G)`), and [awards-022.toml](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/scenarios/awards/awards-022.toml#L1-L47) (3-runner blocked award cascade where R2 scores, R1 gets 3B, and BR is capped at 2B, `OBR 5.06(b)(4)(G)` Comment) all check the published rulebook text.

### P3-1: Calibration note and ledger discrepancies — **Confirmed resolved** (`ae06ade`)
- **Fix:** [2026-11-14-play-area.md](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/docs/calibration/2026-11-14-play-area.md#L73-L147) and [metrics-ledger.md](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/docs/metrics-ledger.md#L105-L120) now quote the merged `52d6632 → 980a7dc` deltas primarily (with branch-point deltas in brackets), label the `25fa2dd` table, include the post-review `0a17409 → 27c2494` run and its counterfactual, and close the HR/PA entry (`+0.11` points predicted from composition vs. `+0.08` mean measured on `52d6632 → 980a7dc`).

---

## Three Judgment Points

### 1. P1-3 (`max(start, legal)` vs `from_pitch` in `OBR 5.06(b)(4)(G)`): Who is right?

**The author is right, and my original suggestion to use `r.start_base.max(legal_at)` when `from_pitch` is true was wrong** (except for `legal_at == 4`, a run already legally scored before the throw, which [engine.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L336) handles via `if from_pitch && legal(r) < 4 { r.start_base } else { legal(r) }`).

Here is the exact rule text from the Official Baseball Rules:

1. **OBR 5.06(b)(4)(G)** (base rule):
   > *"Two bases when, with no spectators on the playing field, a thrown ball goes into the stands, or into a bench (whether or not the ball rebounds into the field), or over or under or through a field fence, or on a slanting part of the screen above the backstop, or remains in the meshes of a wire screen protecting spectators. The ball is dead. When such wild throw is the first play by an infielder, the umpire, in awarding such bases, shall be governed by the position of the runners at the time the ball was pitched; in all other cases the umpire shall be governed by the position of the runners at the time the wild throw was made;"*

2. **OBR 5.06(b)(4)(G) Approved Ruling** (when all runners, including the batter-runner, have advanced a base):
   > *"APPROVED RULING: If all runners, including the batter-runner, have advanced at least one base when an infielder makes a wild throw on the first play after the pitch, the award shall be governed by the position of the runners when the wild throw was made."*

3. **OBR 5.06(b)(4)(G) Comment** (when a runner has advanced to the next base before the infielder's first-play throw, but not all runners have advanced):
   > *"Rule 5.06(b)(4)(G) Comment: In certain circumstances it is impossible to award a runner two bases. For example, a runner on first base breaks for second on a pitch, and the batter hits a short pop fly just behind the shortstop, who fields the ball on the bounce and throws wildly into the stands behind first base in an attempt to double off the runner who had started for second (or throws wildly into the stands trying to get the batter-runner at first before the batter-runner reaches first). If the runner has reached second base before the wild throw is made, he is still only awarded third base (two bases from first base, his position at the time of the pitch)."*

Why the author's code (`if from_pitch && legal(r) < 4 { r.start_base } else { legal(r) }`) and [awards-006.toml](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/scenarios/awards/awards-006.toml#L1-L49) are exact:
- In `awards-006.toml`, `A.1` starts on 1B, steals/reaches 2B at `t = 3.2` (`legal(A.1) == 2`), and at `t = 3.5` (before batter `A.2` has reached 1B, so `advanced == false` and `from_pitch == true`) the shortstop's first throw goes into the stands. Under `OBR 5.06(b)(4)(G)` Comment, `A.1` is governed by his position *at the time of the pitch* (`1B + 2 = 3B`), receiving only one additional base beyond the 2B he had already reached. Had `r.start_base.max(legal(r))` been used, `A.1` would have been awarded `2B + 2 = 4B` (home), directly violating the `5.06(b)(4)(G)` Comment.
- Meanwhile, when a runner has already legally scored (`legal(r) == 4`) before the throw, `if from_pitch && legal(r) < 4` preserves `from = 4` (`AdvanceCause::OnPlay`), so a legally scored run is never taken off the board ([awards-018.toml](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/scenarios/awards/awards-018.toml#L1-L63)).

---

### 2. P2-2 (Batted balls into the seats staying live until they come down, and `BallDead` time for the client's camera cut)

1. **Is keeping a batted ball over a foul-territory wall live until it comes down in the seats the Official Baseball Rule?**
   - **Yes.** Under **OBR 5.09(a)(1) Comment**:
     > *"A fielder may reach over a fence, railing, rope or other line of demarcation to make a catch. He may jump on top of a railing, or canvas that may be in foul ground. No interference shall be allowed when a fielder reaches over a fence, railing, rope or into a stand to catch a ball. He does so at his own risk."*
     And under **OBR 5.06(b)(3)(C) & Comment**:
     > *"If a fielder, after having made a legal catch, should step or fall into any out-of-play area, the ball is dead and each runner shall advance one base, without liability to be put out, from his last legally touched base at the time the fielder entered such out-of-play area."*
   - A batted ball in flight over foul ground is not dead when it crosses the vertical plane of the stands wall or dugout lip; it becomes dead only when it touches a spectator, seat, floor, or object in dead-ball territory (`into_the_stands` reaching `landing_top + r`), or when a fielder who caught it falls into dead-ball territory. Because a fielder whose body is stopped inside the wall (`w.dist >= 0`, body center `>= 0.30 m` inside the wall) has `REACH = 1.0 m`, he can legally catch a ball up to `REACH - BODY_R = 0.70 m` past the wall plane while it descends toward the 4-ft rail or dugout roof.
   - And on an uncaught foul ball that travels farther than `0.70 m` into the stands, keeping the play live for the extra `~0.5–1.5 s` until the ball lands in the seats cannot corrupt the rules state: `decide(st, false)` returns all runners to their time-of-pitch bases (`rules-spec` §7.4) and discards any `pending_passes` (which are only judged if `fair == true`, [engine.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L651)).

2. **Is the dead-ball time (`dead_s`) now right for the client's camera cut?**
   - **For throws out of play and fair balls over the outfield fence (home runs):** Yes. Both emit `Touch::OutOfPlay` and `MarkerKind::BallDead` at the exact instant the ball crosses the wall/fence plane (`dead_s`), with `dead_at` equal to the 3D crossing point. On a throw out of play, the ball track continues flying into the seats via `over_the_wall` while `dead_s` tells the client the exact crossing time when the ball became dead.
   - **For batted balls over a foul-territory wall (foul balls into the seats/dugout, or fair bounding balls bouncing over the 4-ft foul wall for a ground-rule double):** `Touch::OutOfPlay` and `MarkerKind::BallDead` are emitted at `t = path.t_end()` (when the ball lands on the seats/roof/dugout floor), which is **also the end of the timeline clip (`dead_s == duration_s`)**, whereas `dead_at` (`p`) is the 3D point where the ball **earlier** crossed the wall plane (`tr.find(Crossing::Boundary)` at `t_cross < dead_s`).
   - That is physically and rule-wise consistent (the ball is dead when it lands in the seats at `dead_s`), **with one caveat for client camera work (noted as P3-1 below)**: at `t = dead_s`, the ball in `bb_timeline_sample` is at its landing spot in the seats, **not** at `dead_at` (which it passed `0.2–1.5 s` earlier), and because `dead_s == duration_s` on a foul ball, a client that waits until `dead_s` to cut back to the field has already reached the end of the clip.

---

### 3. P2-3 (Fielders braking for walls: physical reasonableness vs. caution)

In `move_fielders` ([live.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/live.rs#L1877-L1882)):
```rust
let w = field.body_clearance(fd.pos.x, fd.pos.y, BODY_R);
let into = -fd.vel.dot(w.inward);
let allowed = m::sqrt(2.0 * BRAKE * w.dist.max(0.0));
if into > allowed {
    fd.vel = fd.vel + w.inward * (into - allowed);
}
```
Let's evaluate the three physical questions:

1. **Deceleration value (`BRAKE = 6.0 m/s²` ≈ `0.61 g`):**
   - Biomechanically reasonable. Published human sprint deceleration limits on turf/dirt are `5.5–7.0 m/s²`, and `6.0 m/s²` is the same constant used across `live.rs` for stopping (`stop_point`) and turning (`run_gap`).

2. **When fielders start to brake (`w.dist < into² / (2 * BRAKE)`):**
   - For an outfielder running straight at the wall at `v_max = 8.0 m/s`, `into > allowed` engages when `w.dist < 8.0² / (2 * 6.0) = 5.33 m` (`17.5 ft` from his body's stopping point `BODY_R` inside the wall, or `5.63 m` / `18.5 ft` from the wall face).
   - That `5.33 m` (`17.5 ft`) distance matches the width of a real MLB warning track (`15–20 ft`, or `4.5–6.0 m`), which exists specifically to warn an outfielder at full sprint that he is one stopping distance from the wall.
   - Furthermore, capping only the normal velocity component `into = -fd.vel.dot(w.inward)` allows a fielder running obliquely along the wall or warning track to keep his tangential speed parallel to the wall.

3. **Is it too cautious, and why did it raise runs by `+0.08` and `+0.18` per team-game?**
   - **Yes, it is somewhat too cautious in two specific ways**, which together explain why `+0.08` / `+0.18` R/team-game is larger than real wall-aversion effects:
     - **First (full stop to `0.0 m/s` at `w.dist = 0`, even when the catch is `REACH = 1.0 m` inside the wall):**
       Covering the last `d = 5.33 m` to `w.dist = 0` at constant `8.0 m/s` takes `t_const = d / v = 5.333 / 8.0 = 0.67 s`. Braking all the way to `0.0 m/s` at `w.dist = 0` under `into = sqrt(2 * BRAKE * w.dist)` takes `t_brake = v / BRAKE = 8.0 / 6.0 = 1.33 s` — a **`+0.67 s` penalty** at the wall (`+0.27 s` even at `w.dist = 0.70 m`, where the fielder's `1.0 m` reach already touches the wall!). In reality:
       1. If the ball is `1.0 m` in front of the wall (`w.dist = 0.70 m`), a fielder only needs his *glove* (`REACH = 1.0 m`) to reach the ball before he finishes braking over the remaining `0.70 m` to the wall.
       2. At the wall itself, a real outfielder on the warning track does not brake to `0.0 m/s` before making a play; he braces against the padded wall at `v_wall ≈ 2.5–3.0 m/s` (about a brisk jog), which reduces the warning-track time loss from `v_max / (2 * BRAKE) = 0.67 s` to `(v_max - v_wall)² / (2 * BRAKE * v_max) ≈ 0.26 s` (`2.5×` smaller).
     - **Second (mismatch between `intercept_scan` planning and `move_fielders` execution — see P2-1 below):**
       `intercept_scan` ([live.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/live.rs#L1313)) computes `run_gap` using `run_dist_from`, which assumes the fielder runs at unbraked `v_max` (`8.0 m/s`) all the way to the intercept point, and checks `b.z > CATCH_MAX_Z` (`2.4 m`) instead of `WALL_CATCH_MAX_Z` (`3.0 m`). Because `intercept_scan` does not know that `move_fielders` will bleed off `0.27–0.67 s` over the final `5.33 m` before the wall, a fielder *calls* a ball near the wall (`call_value` tier 2/3, calling off teammates) believing he has `> 0.3 s` to spare, and then arrives `0.3–0.6 s` later than his own plan predicted because the wall brake slows him down!

---

## Client-Facing C ABI Review (`bb_ffi`, `include/bb_ffi.h`, `c/smoke.c`)

Commits reviewed: `39db36c`, `323e05e`, `bd69a04`.

1. **Struct layout (`BbPlayInfo` in [bb_ffi.h](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_ffi/include/bb_ffi.h#L42-L59) and [lib.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_ffi/src/lib.rs#L266-L312)):**
   - Exact byte-for-byte match between C and Rust `#[repr(C)]`, 4-byte aligned, with explicit padding fields (`_pad: u16`, `_pad2: [u8; 3]`) and zero implicit compiler padding:
     - Existing fields occupy bytes `0..188` (`batter: u16` at `184..186`, `_pad: u16` at `186..188`).
     - Appended fields occupy bytes `188..212`:
       - `dead_s: f32` (`188..192`)
       - `dead_at: [f32; 3]` (`192..204`)
       - `dead_reason: u8` (`204..205`)
       - `_pad2: [u8; 3]` (`205..208`)
       - `projected_distance_m: f32` (`208..212`)
   - When no `MarkerKind::BallDead` is present on the play, `bb_timeline_info` ([lib.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_ffi/src/lib.rs#L402-L407)) sets `dead_s = -1.0`, `dead_at = [0.0; 3]`, `dead_reason = 0`; when no contact occurred, `projected_distance_m = 0.0`.

2. **Appended-only compatibility:**
   - **FlatBuffers (`sim/schemas/timeline.fbs`):** `BallDead = 7` is appended to `enum MarkerKind : ubyte` and `projected_distance_m: float = 0.0` is appended at the end of `table Telemetry`. Both are wire-compatible in FlatBuffers.
   - **C ABI (`BbPlayInfo`):** Appending at the end of `BbPlayInfo` preserves source compatibility and all field offsets `0..188`. Because `bb_ffi` is a `staticlib` built alongside `include/bb_ffi.h`, C/C++ clients compiled against the header get the 212-byte struct size automatically. (Note: `bb_timeline_info` writes `*out = info` without a caller-supplied `sizeof(BbPlayInfo)` parameter, so if `bb_ffi` were ever dynamically linked against a pre-compiled binary built with the 188-byte header, writing 212 bytes would overflow a 188-byte caller buffer by 24 bytes.)

3. **C smoke test ([smoke.c](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_ffi/c/smoke.c#L36-L50)) and Rust ABI tests ([lib.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_ffi/src/lib.rs#L543-L579)):**
   - `smoke.c` and `plays_a_game_through_the_abi` both check across a full game (seed 3) that `dead_s <= duration_s + 1e-4`, `dead_reason` is in `1..=3` when `dead_s >= 0` and `0` otherwise, `dead_at[2]` is in `[0, 60]`, `projected_distance_m > 0` iff `contact_s >= 0`, and (`lib.rs:554-560`) that `dead_at` is within `2 mm` of `Field::ball_clearance(..., BALL_R).dist == 0` and home runs have `projected_distance_m > fence_at(theta)`.

---

## Remaining / New Findings

### P2-1 (new): `intercept_scan` does not account for the wall-braking cap in `move_fielders` (and checks `CATCH_MAX_Z` instead of `WALL_CATCH_MAX_Z`), so fielders over-promise on warning-track balls and arrive up to `0.67 s` after their planned `t_int`
- **File & Line:** [live.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/live.rs#L1313-L1355) (`intercept_scan`), [live.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/live.rs#L1877-L1882) (`move_fielders`)
- **Evidence:**
  1. In `intercept_scan` (`live.rs:1313`), `run_gap(fd, b, t - start, REACH * 0.7)` uses `run_dist_from`, which assumes the fielder runs at his full top speed `v_max` (`~8.0 m/s`) all the way to `b`.
  2. In `move_fielders` (`live.rs:1877-1882`), once the fielder enters `w.dist < v_max² / (2 * BRAKE) ≈ 5.33 m` from a wall, his velocity component toward the wall is clamped to `sqrt(2 * BRAKE * w.dist.max(0.0))`, which takes `v_max / BRAKE = 1.33 s` to reach `w.dist = 0` instead of `0.67 s` at `v_max` (`+0.67 s` slower at the wall, and `+0.27 s` slower at `w.dist = 0.70 m`).
  3. Because `intercept_scan` and `call_value` (`live.rs:1500`) use the unbraked `run_gap` / `run_time_to`, a fielder chasing a fly ball near the wall computes an optimistic `t_int` (and `spare = ti - (start + run_time_to)`), calls the ball as a sure catch (calling off teammates who may be running parallel to the wall without normal-component braking), and then arrives `0.3–0.67 s` late as `move_fielders` brakes him to `0 m/s` at the wall.
  4. In addition, `intercept_scan` (`live.rs:1343`) rejects points with `b.z > CATCH_MAX_Z` (`2.4 m`), whereas `try_captures` (`live.rs:1938`) allows a leaping catch up to `WALL_CATCH_MAX_Z` (`3.0 m`) within `3.0 m` of a wall.
- **Suggested Fix:**
  - Either allow a small non-zero wall contact/bracing speed `V_WALL ≈ 2.5 m/s` in `allowed = m::sqrt(V_WALL * V_WALL + 2.0 * BRAKE * w.dist.max(0.0))` (until the final step at `w.dist < 0.15 m`) so a fielder on the warning track does not brake to a dead stop before catching a ball `REACH = 1.0 m` from his body, or include the wall-braking time penalty in `run_gap` / `run_time_to` and use `zmax_at(b)` (`WALL_CATCH_MAX_Z` within `3 m` of a wall) in `intercept_scan`.

### P3-1 (new): On foul balls into the seats, `BbPlayInfo::dead_s` is the seat-landing time (`duration_s`) while `BbPlayInfo::dead_at` is the earlier wall-plane crossing point
- **File & Line:** [live.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/live.rs#L2218-L2229), [timeline.rs](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/timeline.rs#L335-L340), [bb_ffi.h](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_ffi/include/bb_ffi.h#L52-L55)
- **Evidence:**
  - For a batted ball over a foul-territory wall, `live.rs:2228` emits `Touch::OutOfPlay` at `t = path.t_end()` (after `into_the_stands` has flown the ball down to `z = landing_top + r` in the seats) with `(x, y, z)` set to `q = tr.find(Crossing::Boundary)` (where the ball crossed the wall plane `0.2–1.5 s` earlier).
  - Consequently, in `BbPlayInfo`, `dead_s` equals `duration_s` (when the ball lands in the seats and the clip ends), while `dead_at` is the earlier wall-crossing point (so sampling the timeline at `t = dead_s` gives a ball position in the seats, not `dead_at`).
- **Suggested Fix:**
  - Document in `include/bb_ffi.h` (and `data-formats.md` §5.4) that on a foul ball into the seats, `dead_s` is when the ball comes down in the seats (`duration_s`) while `dead_at` is where its trajectory crossed the wall plane earlier.

---

## Verification

Executed under the heat rules (`CARGO_BUILD_JOBS=2 taskset -c 0-3 nice`, one command at a time):
- `cargo test --manifest-path sim/Cargo.toml -p bb_physics --lib` — **33 passed, 0 failed** (including `foul_walls_never_cut_fair_ground`, `a_dugout_takes_low_balls_and_the_roof_takes_high_ones`, `a_ball_starting_beyond_a_wall_is_out_of_play`, `the_playing_field_is_convex`, `stands_wall_carom_and_over_the_top`).
- `cargo test --manifest-path sim/Cargo.toml -p bb_rules --test scenarios -- --nocapture` — **129 scenarios, 0 failed, 2 known gaps** (including `awards-001`..`awards-022`).
- `cargo test --manifest-path sim/Cargo.toml -p bb_game --lib -- nobody_runs_into_the_stands` — **1 passed, 0 failed**.
- `cargo test --manifest-path sim/Cargo.toml -p bb_ffi` — **5 passed, 0 failed** (including `plays_a_game_through_the_abi`, `rosters_and_batters_cross_the_abi`, `nothing_is_given_away_before_the_decision_or_the_call`, `timelines_sample_to_plausible_frames`).

---

## Resolution

Fixed in part; one judgment deferred. Branch `work/a10-recheck-fixes`, `a73c180`, merged with main `f3dc029`; numbers in the metrics ledger.

- **P2-1: confirmed, fixed.** A probe of the stepped movement (`the_plan_includes_the_slow_down_for_the_wall`) showed the plan early by 0.37 s for a stand point 0.3 m from the wall (hand figure 0.39 s; 0.67 s at the wall). `wall_delay` in `live.rs` adds that time to `intercept_scan`'s gap and to `call_value`'s spare; it uses only what the fielder can see (the wall and his own speed): the normal component of his top speed, held to what he can build up over the distance he has, so a run along the wall loses nothing. The plan now also uses `catch_ceiling` (3.0 m within 3 m of a wall, as `try_captures`). Tests: the plan against the movement step at 0.3, 1 and 3 m clear and the 0.67 s figure at the wall; a plan for a catch between 2.4 and 3.0 m at the wall; a fielder not calling a ball that the wall makes him late for. Each was mutation-checked (open-field ceiling, dropped `c/v` term, call without the delay all fail). Not covered: `run_time_to` in `intercept_inner`'s fallback (the fielder going to where the ball comes to rest) and the holder's use of it in `holder.rs` (another worker's code) do not include the wall delay.
- **P3-1: confirmed as a documentation gap, fixed in the docs.** The behaviour is intended (a batted ball over a foul wall is dead once it is down in the seats, rules-spec 5.3), so `bb_ffi.h` and data-formats §5 now say `dead_s` and `dead_at` are different instants for that case. I did not verify Gemini's 0.2–1.5 s figure and left it out. No ABI change.
- **Judgment Point 3 (is the braking too cautious): deferred.** Deceleration 6 m/s² is the value used elsewhere and is reasonable. A wall-contact speed of about 2.5 m/s (Gemini's suggestion) is a behaviour change with no source for the number in the specs; with the plan now consistent with the braking, the run effect of the braking (+0.08, +0.18 runs per team-game found in the A10 note) is the only remaining cost, and the owner should rule on whether to calibrate it. Measured effect of this fix alone: runs +0.03 and −0.12 on the two sets, inside the noise.
