# Independent Review: A4 — Throw Choices (`00445f5...0a17409`)

Reviewer: Gemini  
Date: 2026-11-14  
Branch reviewed: `work/a4-throw-choices` at `0a17409` (`git diff 00445f5...0a17409`)

---

## Findings

### [P1] 1. `phi` saturates to `1.0` in `f64` for margins `>= 1.00 s`, causing two-out force plays to tie at `value == 1.0` and pick a throw to first (`k = 0`) over stepping on a nearby force bag

- **File & Line**:
  - `sim/crates/bb_game/src/live/holder.rs:113, 125, 143`
  - `sim/crates/bb_game/src/live.rs:498-500, 2363-2366, 2665-2666`
- **Evidence**:
  - In `Sim::out_weight` (`sim/crates/bb_game/src/live.rs:2364-2366`), when `self.st.outs == 2`, every forced play (`1B`, `2B`, `3B`, `Home`) is assigned the exact same weight `1.0`, and `force_continuation` (`sim/crates/bb_game/src/live/holder.rs:84`) returns `None`.
  - In `holder_options` (`sim/crates/bb_game/src/live/holder.rs:125`), `p_out = phi((runner_at - ball_at) / 0.12)` computes `0.5 * (1.0 + m::erf(x / SQRT_2))`. In IEEE-754 `f64`, `m::erf(z)` rounds to exact `1.0` for `z >= 5.8925` (`x >= 8.333`, i.e., `runner_at - ball_at >= 1.00 s`).
  - Consequently, with two outs, **every** forced option with at least `1.00 s` in hand evaluates to `p_out == 1.0` and `option.value == 1.0` bit-for-bit.
  - `holder_options` iterates `self.r.iter().enumerate()` (`holder.rs:113`), where `self.r[0]` is always the batter-runner (`start == 0`, advancing to `base = 1`) and `self.r[1..]` are the preceding baserunners (`live.rs:2913`). `holder_act` selects `best` using strict `>` (`o.value > options[j].value`, `live.rs:2666`).
  - While `docs/agent-models.md` §5.6 states *"An exact tie keeps the carry, which is added before the throw for the same runner"*, that ordering only holds within the *same* runner `k`. Across runners at two outs, `options` places the **throw to first** for the batter (`k = 0`, `carry = false`, `value = 1.0`) *before* the **carry to second or third** for a forced baserunner (`k = 1`, `carry = true`, `value = 1.0`). Whenever both margins are `>= 1.00 s` (for example, a shortstop or third baseman fielding a ball `1 m` from second or third base with a forced runner `2.0 s` away and the batter `1.2 s` from first), `1.0 > 1.0` is `false`, so the fielder standing `1 m` from a force bag refuses to step on it and instead throws across the diamond to first base.
- **Suggested Fix**:
  - In `holder_act` (`sim/crates/bb_game/src/live.rs:2665-2666`), break exact (or near-exact) `value` ties in favor of `o.carry` over `!o.carry`, and then in favor of the larger timing margin `o.runner_at - o.ball_at`:
    ```rust
    let better = |o: &holder::HolderOption, cur: &holder::HolderOption| {
        if (o.value - cur.value).abs() > 1e-12 {
            o.value > cur.value
        } else {
            (o.carry, o.runner_at - o.ball_at) > (cur.carry, cur.runner_at - cur.ball_at)
        }
    };
    ```

---

### [P1] 2. `force_continuation` over-estimates the relay out at first (`p2`) and double-counts lead-out value, driving double plays to `2.75/game` vs ~`1.7/game` in MLB

- **File & Line**:
  - `sim/crates/bb_game/src/live/holder.rs:83-107, 146-150`
  - `sim/crates/bb_game/src/live.rs:2083-2085, 2368, 2682, 2711-2718`
- **Evidence**:
  - **Published MLB benchmark**: In MLB, teams turn `0.70` GDP per team-game (`1.40/game` combined; `docs/calibration-targets.md:195`) and roughly `0.80–0.85` total double plays per team-game (`~1.6–1.7/game` combined, Baseball-Reference MLB league averages, as cited in `docs/calibration/2026-11-14-throw-choices.md:335` and `docs/metrics-ledger.md:48`). On this branch, double plays rise from `2.30` to `2.75` per game (`+0.45/game`).
  - **Four specific causes in the continuation model and pivot mechanics**:
    1. **Hurried-arrival mismatch between `force_continuation` and `holder_act` (`holder.rs:102` vs `live.rs:2682, 2711-2718`)**: For a throw to second or third, `force_continuation` starts the relay pivot clock at `option.ball_at` (`race.ball_at`), which assumes the first holder releases **immediately at `su.earliest_s` (maximum hurry)**. However, `holder_act` (`live.rs:2682, 2713`) passes only the *first* out's margins `(o.runner_at - o.ball_at, o.runner_at - o.set_ball_at)` into the `hurry` check (`set < SAFE_MARGIN_S`). Whenever the lead force at second has `set >= 0.25 s` (which is true on almost all routine double-play grounders), **the first holder does not hurry his throw to second — he waits until `su.set_s` (`0.26–0.56 s` later than `option.ball_at`!)**. Thus `force_continuation` systematically under-predicts `ball_at` at first base by `0.26–0.56 s`.
    2. **Zero pivot turn/footwork cost and a `273 ms` hurried relay transfer (`holder.rs:101-103`, `live.rs:2083-2085`)**: `force_continuation` assumes the pivot receiver at second or third releases the relay in `RELAY_TRANSFER_FRAC * next.a.transfer` (`0.7 * 0.65 = 0.455 s`) at `100%` `THROW_EFFORT * next.a.arm`, with zero turn (`turn_extra_s = 0`) and zero footwork penalty, even when a middle infielder is running toward third-base side of second or taking a throw from the pitcher/first baseman with his back to first. In actual execution (`try_captures`, `live.rs:2083-2085`), setting `fd.set_base = 0.455 s` and `fd.face = None` allows the pivot man to hurry the relay throw at `RUSH_MIN_FRAC * 0.455 = 0.273 s` (`273 ms` catch-to-release, roughly twice as fast as Statcast MLB double-play pivot exchanges of `~0.55–0.70 s`).
    3. **Zero-second release after carrying to second base (`holder.rs:93-100`)**: When `option.carry == true` (e.g., `seed 70 / play 157`), `force_continuation` (and `holder_act` on the tick after touching second) computes `release_at = option.ball_at.max(fd.up_at + su.earliest_s)` using `fd.up_at`, `fd.face`, and `fd.catch_vel` from when the batted ball was *originally fielded*. Because `option.ball_at - fd.up_at` (`~1.0–1.5 s` of running to the bag) already exceeds `su.set_s`, `release_at == option.ball_at` (`0.00 s` after touching second), so the carrier releases a full-strength set throw to first on the **exact tick he touches second base at full running speed**, with zero plant, turn, or transfer time at the bag.
    4. **Unfiltered `p_second <= 0.3` and double-counting with base weights (`holder.rs:146-150`, `live.rs:2368`)**: `holder_options` adds `continuation_weight * p_out² * p_second² * w_second` even when `p_second <= 0.3` (a relay throw the pivot man will never attempt because `holder_act` filters `o.p_out > 0.3`). Moreover, with `< 2` outs, the single-out base weights `[0.9, 1.0, 1.1, 1.3]` already give second (`1.0`) and third (`1.1`) a premium over first (`0.9`) in part because retiring the lead runner preserves force/double-play state. Adding a full `1.0 * p1² * p2² * w2` (`+0.9` at 0 outs, `+1.0` at 1 out) gives a 2B–1B double play a value of `1.9–2.0` against `1.3` for a sure out at home — meaning with 0 outs, the defense **always** concedes a run from third to turn two (`1.9 > 1.3`), even in the bottom of the 9th of a tied game.
- **Suggested Fix**:
  - In `force_continuation` (`holder.rs:83-107`):
    1. Gate the continuation on `p_second > 0.3`.
    2. Use the actual first-throw arrival time (or feed the continuation's `set` margin into `holder_act`'s `hurry` decision at `live.rs:2682, 2713` so the first throw hurries when needed for the double play).
    3. Include `turn_extra_s` for the pivot receiver's turn from the incoming throw vector to `base_pos(1) - base_pos(option.base)`, and reset `up_at` / `set_base` (or add a plant/transfer time) when a carrier touches second base before throwing to first.

---

### [P2] 3. Residual Invariant 2 leak in `base_held_ahead`, noiseless runner/teammate reads, and a cutoff-man regression in `relay_job` when runners stop at an advanced base

- **File & Line**:
  - `sim/crates/bb_game/src/live.rs:504-506, 1558-1564, 2374-2379, 2556-2560`
  - `sim/crates/bb_game/src/live/holder.rs:60, 88-92, 102, 118-119`
- **Evidence**:
  - **Private `o.goal` still read in `Sim::base_held_ahead` (`sim/crates/bb_game/src/live.rs:2374-2379`)**: Although the doc comment at line 2374 states *"he is at or short of it and not visibly moving forward (standing, or going back). Direction of motion is visible"*, line 2377 still checks `o.goal as f64 * BASE_L <= o.s + 1e-6` (the preceding runner's private `goal`) instead of his public signed velocity `o.direction * o.v < moving_mps`.
  - **Noiseless runner and teammate reads in `holder.rs` (`holder.rs:60, 88-92, 102, 118-119`)**: `visible_target` and `runner_eta` read exact `r.s` and `r.direction * r.v` with zero perception noise (`FieldingIQ` is not applied), and `throw_race` / `force_continuation` (`holder.rs:60, 102`) read the relay/pivot teammate's exact `a.transfer` and `a.arm` rather than league-typical estimates (contrast `receiver_ready` at `live.rs:2805`, which uses `TYPICAL_SPEED`). While `docs/agent-models.md` §5.6 explicitly documents the noiseless runner read as an interim simplification, it remains an open gap against §5.6's target (`FieldingIQ`-scaled observation noise).
  - **Cutoff/relay regression in `Sim::relay_job` (`sim/crates/bb_game/src/live.rs:1558-1564`)**: `relay_job` sets the baseline cutoff base from `let on = |b: u8| self.r.iter().any(|r| r.start == b && !r.out && !r.scored)` (**time-of-pitch base `r.start`**, not current base!), and then raises `b` by `visible_target(r)`. Before A4, `visible_target(r)` returned `(k + 1).min(4)` for a stationary runner on base `k`, which masked `on(b)` checking `r.start`. Now that `visible_target` returns `None` when a runner stops on a bag (`r.v == 0.0`), if a runner who started on first (`r.start = 1`) advances to third base and stops on third while the ball is still in the outfield (or a batter-runner stops on second), `visible_target` returns `None` and `b` in `relay_job` drops from `4` back to `3` (or `3` back to `2`), pulling the cutoff man off the home-plate (or third-base) line!
  - **Assessment of zeroing `r.v` on arrival (`sim/crates/bb_game/src/live.rs:2556-2560`)**: Zeroing `r.v` when `(stop_s - r.s).abs() < 1e-6` fixes a genuine tick-ordering artifact: in `simulate_live_inner` (`live.rs:3082-3089`), `holder_act()` runs *before* `move_runners()`, so when `move_runners()` clamped `r.s = goal_s` at tick `t`, `r.v` remained `8.2 m/s` until `move_runners()` zeroed it at the start of tick `t + DT` (`live.rs:2501`). Note, however, two subtleties:
    1. At a base (`b = 1, 2, 3`), `ChoiceParams::runner_target` (`throwing.rs:36`) *already* returns `None` within `on_bag_m = 0.05 m` regardless of `r.v` (as verified by `throwing.rs:57`: `p.runner_target(82.296, 8.0, 27.432) == None`), so zeroing `r.v` on arrival only changes `visible_target` at a mid-basepath `hold_s` stop (`live.rs:2557`).
    2. Physically, runners in `move_runners` (`live.rs:2508-2523`) accelerate toward `8.2 m/s` all the way to `goal_s` and then drop from `8.2 m/s` to `0.0 m/s` instantaneously in `0.0 m` (whereas fielders brake at `6 m/s²` and batter-runners overrun first at `4 m/s²`). Until the slide/braking deceleration in `docs/agent-models.md` §6.2 is implemented, a fielder watching a runner `0.06 m` short of third base still sees him sprinting at `8.2 m/s` with no deceleration cue.
- **Suggested Fix**:
  - In `Sim::base_held_ahead` (`sim/crates/bb_game/src/live.rs:2377`), replace `o.goal as f64 * BASE_L <= o.s + 1e-6` with `o.direction * o.v < self.f[0].a.throw_choice.moving_mps`.
  - In `Sim::relay_job` (`sim/crates/bb_game/src/live.rs:1558`) and `Sim::assign_backups` (`live.rs:1662`), check current visible base progress `(r.s / BASE_L + 1e-6).floor() as u8 == b` instead of `r.start == b`.

---

### [P2] 4. Explanation of the "unexplained" 50% drop in line-drive and fly-ball double plays (`.011 → .006`, `.014 → .007`) and matching rise in fly-ball fielder's choices (`.009 → .017`): abandoning nearby force steps when `force_continuation` only checks first base

- **File & Line**:
  - `sim/crates/bb_game/src/live.rs:2070-2081, 2279-2281, 2649-2692`
  - `sim/crates/bb_game/src/live/holder.rs:83-107`
  - `docs/calibration/2026-11-14-throw-choices.md:325-344`
  - `docs/metrics-ledger.md:106`
- **Evidence**:
  - **Counterfactual arithmetic check**: The arithmetic in `docs/calibration/2026-11-14-throw-choices.md:325-333` is sound (`5.41 → 5.07` is `−0.34` runs/team-game on seeds `0–299` and `5.62 → 5.28` is `−0.34` on seeds `1000–1299`; setting `continuation_weight = 0` yields `5.27`, cleanly partitioning `−0.34` into `−0.20` from the continuation and `−0.14` from the visible-target gate + unified comparison; the `1.2 m` non-force carry rule shifts runs by `−0.01` in both `5.06 vs 5.07` and `5.26 vs 5.27`).
  - **Why line-drive and fly-ball `2+ outs` fell by half in every variant while fly-ball `FC` doubled**:
    1. Look at `docs/metrics-ledger.md:106`: on fly balls, `2+ outs` fell by `.007` (`.014 → .007` on seeds `0–299`, `.013 → .006` on seeds `1000–1299`, ~`40 plays` per 300 games), and **simultaneously `FC` (fielder's choice) on fly balls rose by `.008–.009` (`.009 → .017` on seeds `0–299`, `.008 → .017` on seeds `1000–1299`, ~`45 plays` per 300 games), while `FB hit` was unchanged (`.149 → .148`)**. Similarly, on line drives, `2+ outs` fell from `.011 → .006` (`[.011 → .008]`) while `LD FC` rose from `.014 → .016`.
    2. By `classify_bip` (`sim/crates/bb_game/src/game.rs:101-111`), a fly ball is ruled `FC` only when it is **not caught in flight** (it drops/bounces in the outfield or shallow outfield) and a preceding baserunner is forced out (or put out after an infielder handles the ball) while the batter reaches first safely.
    3. Why did the author's probe query (`2026-11-14-throw-choices.md:340-342`) only see `12 → 3` outs? Because the author queried `step.calls` for `call.base == r.start` ("outs at a runner's own base"). Almost all caught-fly runners in `live.rs:3014, 3021` have `likely_catch == true` and retreat to tag up at `t_c + 0.5 s` (so genuine caught-fly appeal double plays were only `4 → 3` in 256 games, and the other `8 → 0` were batter-runners with `r.start == 0` who had retreated to first when blocked by a runner ahead at `live.rs:2494-2495` and were chased by the old `near_carry` shortcut on their arrival tick).
    4. **Where the other ~`50` LD/FB double plays per 300 games went**: On an uncaught fly ball or line drive to the outfield with multiple runners on base (e.g., runners on first and second with `< 2` outs, where the runners held halfway at `hold_s` while the ball was in the air, `live.rs:3022-3024`, and the batter-runner ran straight to first in `4.45 s`):
       - When the outfielder's throw arrives at second base (or an infielder fields a bounced liner/fly within `0.6–8.0 m` of second base), `update_thrown` (`live.rs:2279-2281`) steps the receiver off the bag to `catch_at` whenever lateral throw error exceeds `BAG_REACH_FRAC * RECEIVE_REACH = 0.77 m` (`try_captures` at `live.rs:2071, 2080` then sets `fd.target = base_pos(to_base)` and `fd.carry_to = None`).
       - **Before A4**: Whenever the fielder holding the ball was `0.6–8.0 m` from second base with a forced runner `r[1]` coming (`t_me + 0.1 < t_runner`), the old `near_carry` shortcut (`live.rs:2663` before `98c58ed`) **always stepped on second base first** (recording **Out #1** by force on `r[1]`), and **on the very next tick after touching second base**, `holder_act` still held the ball and threw to third, home, or second (for a trailing runner stretching) to record **Out #2**!
       - **After A4**: By the time an outfield fly ball or line drive is fielded and thrown in (`t >= 4.0 s`), the batter-runner `r[0]` has already reached first base (`s = BASE_L`). Because `force_continuation` (`holder.rs:83-91`) **only** checks whether `r.start == 0` is advancing to `base == 1`, **carrying `0.7–3.0 m` to touch second base gets `continuation = None` (`value = 1.0`)**, even when touching second base takes `< 0.3 s` and leaves plenty of time to throw out another runner advancing to third (`weight = 1.1`) or home (`weight = 1.3`)! Thus `holder_options` sees `1.3 > 1.0` (or `1.1 > 1.0`), calls `abandon_carry`, and **throws immediately to third or home without stepping on second base first** — converting a 2-out play into at most a 1-out play (or, when the throw goes to second at two outs or after the lead runner stops, converting 2-out plays into 1-out `FC`s).
- **Suggested Fix**:
  - Extend `force_continuation` (or add a carry-then-throw continuation in `holder.rs:83-107`) so that when `option.carry == true` and `option.force == true` (stepping onto a nearby force bag), the holder evaluates whether he can touch `option.base` at `option.ball_at` and *then* throw to retire any other active runner (`base = 1..=4`, not only `r.start == 0` at `base == 1`). Also set `fd.carry_to = Some(to_base)` in `try_captures` (`live.rs:2080`) when a receiver catches a throw off the bag.

---

### [P2] 5. Non-force carry ("tag plan") can oscillate between `carry` and `throw` across ticks while `choice_trace` coalesces and hides the oscillation

- **File & Line**:
  - `sim/crates/bb_game/src/live/holder.rs:157-161`
  - `sim/crates/bb_game/src/live.rs:2639-2641, 2671-2735`
  - `sim/crates/bb_game/src/live/choice_trace.rs:77-87`
  - `sim/crates/bb_game/examples/throw_choice_probe.rs:191-194`
- **Evidence**:
  - **Tick-to-tick flip-flop between `carry` and `throw` (`holder.rs:157-161`, `live.rs:2671-2735`)**:
    1. **At the `carry_max_m = 8.0 m` gate**: If a fielder secures the ball at `7.95 m` from `base` while moving away from `base`, tick `t` admits `carry` (`distance < 8.0`) and sets `fd.target = base_pos(base), fd.carry_to = Some(base)`. On `move_fielders()`, braking at `BRAKE = 6 m/s²` still carries him past `8.0 m` (`e.g. 8.02 m`) on tick `t + DT`, removing `carry` from `holder_options` and switching him to `throw` (`abandon_carry`). Conversely, securing the ball at `8.05 m` while moving toward `base` starts in `throw` (`abandon_carry`) on tick `t` and switches to `carry` on tick `t + DT` once momentum carries him inside `8.0 m`.
    2. **Inside `8.0 m` while waiting for `su.set_s`**: When `carry` wins on tick `t_0` and then `throw` edges ahead on tick `t_1` while `since < su.set_s && !hurry` (`live.rs:2711-2718`), `holder_act` **does not release the throw on tick `t_1`** — it calls `abandon_carry(by)` (setting `fd.target = stop_point(fd)`) and returns early. On tick `t_2`, because waiting pushed the throw's `direct(self.t)` later by `DT` while braking shed away/sideways velocity (or carried the fielder closer to `base`), `carry` can beat `throw` again on tick `t_2`, causing the fielder to alternate between running at `base` and braking toward `stop_point(fd)` while `since < su.set_s`.
  - **`choice_trace` masks `carry → wait-for-throw → carry` oscillations (`live.rs:2676` vs `2709, 2717, 2730` and `choice_trace.rs:77-87`)**:
    - `trace_choice` is called on **every tick** for `"carry"` (`live.rs:2676`), but for `"throw"` it is called **only on the release tick** (`live.rs:2722, 2732`), never on the wait ticks that return early at lines `2709, 2717, 2730` after calling `abandon_carry(by)`.
    - In `choice_trace::push` (`choice_trace.rs:77-87`), two `"carry"` choices with the same `play, holder, possession, base, outs, runner` are coalesced by setting `last.until = c.t; last.ticks += 1` **without checking `(c.t - last.until - DT).abs() < 1e-6`**. Thus, if a holder carries on tick 1, switches to `throw` and brakes (`abandon_carry`) on ticks 2–5 while waiting for `su.set_s`, and switches back to `carry` on tick 6, `choice_trace` merges ticks 1 and 6 into a single `"carry"` record and hides the intermediate switch and braking.
  - **Tag-plan arrival radius and probe out-counting mismatch (`holder.rs:159`, `throw_choice_probe.rs:193`)**:
    - `holder.rs:159` estimates a non-force carry's `ball_at` using `run_time_to(fd, target, 0.6)` (the `0.6 m` force-touch radius), whereas `holder_act` (`live.rs:2639-2641`) tags runners at `tag_reach_m = 1.2 m` while `!on_base` (`>= 0.3 m` from the bag).
    - In `throw_choice_probe.rs:193`, `tag_carry_outs` checks `call.base == c.base`. In `bb_rules` (`sim/crates/bb_rules/src/engine.rs:394`), `TagContact` records `base = r.at + 1`. For a **retreating** unforced runner (`r.direction < 0`, returning to `r.at`), `c.base` is `r.at` while `call.base` is `r.at + 1`, so `call.base == c.base` fails to count tags on retreating runners.
- **Suggested Fix**:
  - In `choice_trace::push` (`sim/crates/bb_game/src/live/choice_trace.rs:77`), require `(c.t - last.until - (1.0 / 60.0)).abs() < 1e-6` before coalescing consecutive `"carry"` ticks.
  - In `holder_options` / `holder_act`, keep `carry` eligible when `fd.carry_to == Some(base)` (avoiding hard cutoff oscillation at `8.0 m`) and require a small hysteresis margin before abandoning an active carry for an unreleased throw.

---

### [P3] 6. New tests do not pin the `p² × w` weighting curve or `0.3` threshold, and `seed_13_play_140` tests a tick where the holder is `9.1 m` (`> 8.0 m`) from first base

- **File & Line**:
  - `sim/crates/bb_game/tests/throw_choices.rs:78-120`
  - `sim/crates/bb_game/src/live.rs:5235-5295`
- **Evidence**:
  - In `seed_13_play_140_takes_a_sure_lead_force_instead_of_automatic_first` (`sim/crates/bb_game/tests/throw_choices.rs:84-92`), `first` is the first baseman's throw release at `t = 2.683 s`, when he is **`9.1 m` from first base (`> carry_max_m = 8.0 m`)** (as noted in `docs/calibration/2026-11-14-throw-choices.md:317`). Because `9.1 m >= 8.0 m`, the carry-to-first option is not even in `first.options` at that tick; and `assert!(first.base >= 2)` would pass even if `out_weight` were flat (`[1.0, 1.0, 1.0, 1.0]`).
  - In both constructed unit tests in `sim/crates/bb_game/src/live.rs:5235-5295` (`an_unforced_runner_is_a_tag_weighed_against_the_force` and `a_sure_force_at_home_beats_carrying_first`), all candidate margins are `>= 1.5 s`, so `p_out == 1.0` for every option (`1.0² == 1.0¹ == 1.0`).
  - In `seed_70_play_157_keeps_the_force_double_play_against_a_single_home_tag` (`tests/throw_choices.rs:95-120`), any `continuation_weight` in `(0.35, 100.0)` passes.
  - Consequently, **replacing `p_out * p_out * weight` with linear `p_out * weight`, changing the `p_out > 0.3` gate, or doubling `continuation_weight` would still pass every test in `tests/throw_choices.rs` and `live::tests`**.
- **Suggested Fix**:
  - Add a unit test in `live::tests` with an intermediate-margin lead play (e.g., `p_home ≈ 0.80` with `weight = 1.3`, so `p² × w = 0.832` while linear `p × w = 1.04`) against a sure force at first (`p_first = 0.99` with `weight = 0.9`, so `p² × w = 0.882` while linear `p × w = 0.891`), verifying that `p² × w` selects first base (`0.882 > 0.832`) whereas linear `p × w` would fail by selecting home (`1.04 > 0.891`).

---

## Verification

### Commands Run
- `git switch -c work/review-a4 0a17409`
- `git log --stat 00445f5..0a17409`
- `git diff 00445f5...0a17409 -- sim docs/agent-models.md docs/calibration/2026-11-14-throw-choices.md docs/metrics-ledger.md`
- `CARGO_BUILD_JOBS=2 taskset -c 0-3 nice cargo test --release --manifest-path sim/Cargo.toml -p bb_agents --lib` (16 passed)
- `CARGO_BUILD_JOBS=2 taskset -c 0-3 nice cargo test --release --manifest-path sim/Cargo.toml -p bb_game --test throw_choices` (4 passed)
- `CARGO_BUILD_JOBS=2 taskset -c 0-3 nice cargo test --release --manifest-path sim/Cargo.toml -p bb_game --lib -- a_sure_force_at_home_beats_carrying_first` (1 passed)
- `CARGO_BUILD_JOBS=2 taskset -c 0-3 nice cargo test --release --manifest-path sim/Cargo.toml -p bb_game --lib -- an_unforced_runner_is_a_tag_weighed_against_the_force` (1 passed)

### Files Read Only
- `CLAUDE.md`
- `docs/calibration/2026-11-14-throw-choices.md`
- `docs/agent-models.md` (§5.1–§13)
- `docs/metrics-ledger.md`
- `docs/calibration-targets.md`
- `sim/crates/bb_game/src/live/holder.rs`
- `sim/crates/bb_game/src/live/choice_trace.rs`
- `sim/crates/bb_game/src/live.rs`
- `sim/crates/bb_agents/src/throwing.rs`
- `sim/crates/bb_game/tests/throw_choices.rs`
- `sim/crates/bb_game/examples/throw_choice_probe.rs`
- `sim/crates/bb_game/src/game.rs`
- `sim/crates/bb_cli/src/game_lab.rs`
- `sim/crates/bb_rules/src/engine.rs`

## Resolution

Open.
