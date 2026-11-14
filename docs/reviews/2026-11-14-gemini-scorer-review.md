# Independent Review: A12 / B5 — Official Scorer and Box Score (`0a17409...05666c0`)

**Branch:** `work/review-scorer` (checked out at `7e30d52`, reviewing `0a17409...05666c0`)  
**Scope:** `sim/crates/bb_stats/src/scorer.rs`, `sim/crates/bb_stats/src/lib.rs`, `sim/crates/bb_stats/tests/scorer_cases.rs`, `sim/crates/bb_stats/tests/accounting.rs`, `sim/crates/bb_stats/examples/scorer_audit.rs`, `sim/crates/bb_game/src/game.rs`, `sim/crates/bb_game/src/live.rs`, `sim/crates/bb_game/src/eventlog.rs`, `sim/crates/bb_game/src/describe.rs`, `sim/schemas/events.fbs`, `docs/rules-spec.md` (§9.3–§15), `docs/data-formats.md`, `docs/calibration/2026-11-14-scorer.md`, `docs/metrics-ledger.md`

---

## Case-by-Case Matrix of `score_play_with_options` Against OBR Rule 9

Before the ranked findings, here is the branch-by-branch audit of [`score_play_with_options`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L47-L129) and [`box_score_with_options`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/lib.rs#L100-L238) against Official Baseball Rules (OBR) Rule 9:

| Branch / Line in `scorer.rs` | OBR Clause | Ruling in Code | Assessment |
| :--- | :--- | :--- | :--- |
| `sf` ([`scorer.rs:54-56, 66`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L54-L66)) | **OBR 9.08(d)** (Sacrifice Flies), **9.02(a)(1)** | `flyout && outs_before < 2 && runs > 0 && (caught_foul \|\| caught_in_outfield)` | **Flawed (see P1-1, P1-2, P2-3)**: (1) When a second runner is tagged out or retired on appeal on the play (`outs.len() == 2`), [`scorer.rs:127`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L127) zeroes `rbi = 0` while `sf = true`, violating both **OBR 9.04(b)(1)** and **OBR 9.08(d)**. (2) Conversely, if a runner on a caught fly scores *only* because a subsequent throw went out of play (`runs == runs_after_dead`), `sf` is still set to `true` with `rbi = 0`, violating **OBR 9.08(d)** (*"wait until after the play... run is scored without an error"*). (3) Uses fixed radial threshold `r >= 45 m` instead of OBR 9.08(d)'s functional criterion (any fair or foul fly ball after whose catch a runner tags and scores), denying a sacrifice fly on a shallow fair fly (`r < 45 m`) caught by a retreating infielder or charging outfielder where a runner tags and scores cleanly. |
| `PaResult::Walk \| IntentionalWalk`, `HitByPitch` ([`scorer.rs:68-69`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L68-L69)) | **OBR 9.02(a)(2)–(3)**, **9.04(a)(2)** | `Outcome::Walk` / `Outcome::HitByPitch`; `rbi = runs` (no outs on play); excluded from `AB` in [`lib.rs:193-194`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/lib.rs#L193-L194) | **Correct**: A bases-loaded walk or hit batter forcing a run home credits `rbi = 1` and charges `PA = 1, AB = 0`. |
| `PaResult::Strikeout` ([`scorer.rs:70`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L70)) | **OBR 9.02(a)**, **9.15** | `Outcome::Strikeout`, `AB += 1, K += 1` | **Correct** (sim does not yet model uncaught third strikes). |
| `PaResult::HomeRun` ([`scorer.rs:71`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L71)) | **OBR 9.05(a)(1)**, **9.06(g)**, **9.04(a)(1)** | `Outcome::Hit(4)`, `rbi = runs` | **Correct for clean over-the-fence HRs** under **OBR 9.06(g)** (all runners ahead + batter score). Note: if a runner passes a preceding runner during a home run trot (`Event::RunnerPassed`), [`bb_rules`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L725-L746) still emits `PaResult::HomeRun`, and if two runners passed each other (`outs.len() >= 2`), `rbi` would be zeroed at line 127. |
| `flyout \|\| outs(batter, Force \| Strikeout)` ([`scorer.rs:77-78`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L77-L78)) | **OBR 9.02(a)**, **9.05(b)** | `Outcome::Out` | **Correct** for fly outs (including infield flies declared under **OBR 5.09(a)(5)**, which [`bb_rules`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L653) records as `OutKind::FlyOut`) and batter force/tag outs before 1st base (which [`bb_rules`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L395) records as `OutKind::Force` because `r.forced_to == Some(1) && r.at < 1`). |
| `award.filter(\|_\| !thrown_award)` ([`scorer.rs:82-83`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L82-L83)) | **OBR 9.05(a)(5)**, **9.06(d)** (Ground-rule double) | `Outcome::Hit(2)` | **Partially flawed**: If an infielder/outfielder made an earlier throw on the play before a fair batted ball bounced over the fence (not possible once `GloveSecured` clears `lp.throw`, or if `BallDead::GroundRuleDouble` occurs), `!thrown_award` is true. However, [`scorer.rs:62`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L62) only sets `thrown_award` on `DeadBallReason::OutOfPlay`, so ground-rule doubles get `Outcome::Hit(2)` and credit RBIs for runners awarded home. |
| `thrown_award && !(has_first_touch && before_throw(first_touch_s))` ([`scorer.rs:84-85`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L84-L85)) | **OBR 9.05(a)(1)**, **9.06(e)**, **9.12(a)(1)** | `Outcome::ReachedOnThrowAward` | **Wrong (see P1-3)**: Compares `first_touch_s` (`~4.3 s`, when the batter arrives at 1st) against `t_release` (`~2.5–3.5 s`, when the infielder *releases* the throw to 1st) without checking `clean_play_margin_s`. Because an infielder's throw to 1st is always released *before* the batter touches 1st, **every** infield single where the throw to 1st goes out of play is misruled `ReachedOnThrowAward` even when the batter would have beaten a clean throw (`clean_play_margin_s < 0.15 s`). |
| `!evidence.has_first_touch()` ([`scorer.rs:86-90`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L86-L90)) | **OBR 9.02(g)**, **9.06(f)** | `Outcome::Undetermined` if `WalkOff`, else `Outcome::Out` | **Wrong on walk-offs and in `box_score` (see P1-4)**: When `outs_before == 0` and a runner on 3rd scores the winning run on a clean base hit, [`try_walk_off`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L966) terminates the simulation at `t ≈ 3.8 s` before the batter can reach 1st (`t ≈ 4.4 s`). `scorer.rs` returns `Outcome::Undetermined`, and [`box_score`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/lib.rs#L207-L212) charges the batter with **0-for-1 (`AB = 1, H = 0`)** on his walk-off hit. |
| Preceding runner retired ([`scorer.rs:91-101`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L91-L101)) | **OBR 9.05(b)(1)–(3)** | `ForceOut` or `InfieldOneBaseOut` $\to$ `FieldersChoice`; else `OtherOut` $\to$ `Hit` | **Flawed in two cases (see P1-5)**: (1) When an unforced runner on base $b$ touches $b+1$, overslides, and is tagged out by an infielder returning to $b+1$, [`bb_rules`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L394) records `o.base() = r.at + 1 = b + 2`, so `one_or_return` is `false` and the scorer credits a `Hit` instead of `FieldersChoice` (**OBR 9.05(b)(3)**). (2) Under **OBR 9.05(b)(2)**, a preceding runner called out on appeal (`OutKind::Appeal`) for failing to touch the first base to which he is advancing (`b + 1`) denies a hit even when handled by an outfielder. |
| Unsuccessful play on preceding runner ([`scorer.rs:102-104`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L102-L104)) | **OBR 9.05(a)(6)**, **9.05(b)(4)** | Any throw (`attempted_out && base != 1`) or carry (`base != 1`) sets `UnsuccessfulAttempt` | **Wrong (see P1-6)**: `ScorerThrow` and `ScorerCarry` do not record which runner they targeted. Any throw or carry to 2nd, 3rd, or home aimed at retiring the **batter-runner himself** (e.g., stretching a single into a double with the bases empty!) or thrown *after* the batter already reached 1st base is misclassified as an `UnsuccessfulAttempt` on a preceding runner. |
| Hit value (`value` & `on_throw_cap`, [`scorer.rs:110-120`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L110-L120)) | **OBR 9.06(a)–(f)** | `reached.max(before_out).max(touched)` capped by `on_throw_cap` | **Flawed in three cases (see P1-4, P1-6, P2-1)**: (1) Missing **OBR 9.06(f)** cap limiting a non-HR walk-off hit's value to the bases advanced by the winning run. (2) `on_throw_cap` triggers on throws to `base != 1` aimed at the **batter-runner himself**, wrongly docking a triple/double when the batter beats a throw to 2nd and continues. (3) If a batter touches base $B$, retreats/overslides, and is tagged out before holding $B$ (**OBR 9.06(c)**), `touched` still includes $B$. |
| `rbi` ([`scorer.rs:62-65, 127`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L62-L65)) | **OBR 9.04(a)–(b)** | `if outs.len() >= 2 { 0 } else { runs - runs_after_dead }` | **Multiple P1 violations (see P1-1, P1-2)**: (1) Blanket-zeroes `rbi` whenever `outs.len() >= 2`, denying RBIs on sacrifice flies with a 2nd out on the bases, base hits with two baserunner outs, and non-force/non-reverse-force two-out plays (**OBR 9.04(b)(1)**). (2) Because [`bb_rules`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L774-L808) emits `BallDead` *before* `RunScored` for all pending runs on a throw out of play, `runs_after_dead == runs`, stripping RBIs for runs that crossed home plate *before* the throw went out of play or that would have scored on a base hit without the errant throw. |

---

## Findings

### P1 — Wrong Behavior or Broken Invariant

#### P1-1. `scorer.rs:127` blanket-zeroes `rbi` whenever `outs.len() >= 2`, violating OBR 9.04(a)(1), 9.04(b)(1), and 9.08(d) on sacrifice flies, base hits, and non-force double plays
- **File & Line:** [`sim/crates/bb_stats/src/scorer.rs:66, 127`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L127)
- **Evidence:**
  Line 127 computes:
  ```rust
  let rbi = if outs.len() >= 2 { 0 } else { runs - runs_after_dead };
  ```
  Under **OBR 9.04(a)(1)** and **OBR 9.04(b)(1)**, the Official Scorer withholds an RBI on a two-out play **only** when the batter *"grounds into a force double play or a reverse-force double play."* (In a force double play, both outs are `OutKind::Force`; in a reverse-force double play, an infielder steps on a force base for `OutKind::Force` and then tags a runner who would have been forced had the first out not removed the force.)
  Because line 127 checks only `outs.len() >= 2`, it produces wrong rulings on three classes of plays:
  1. **Sacrifice fly with a second runner tagged out (or retired on appeal) after a run scores (`outs_before == 0, outs.len() == 2`):** Suppose with 0 outs and runners on 1st and 3rd, the batter hits a deep fly to right field (`Out #1 = FlyOut`), the runner on 3rd tags up and scores (`RunScored`), and the runner on 1st is tagged out trying for 2nd (or doubled off on a timing-play appeal) for `Out #2`. Under **OBR 9.08(d)** and **OBR 9.04(a)(1)**, this is a sacrifice fly and 1 RBI. Yet [`scorer.rs:66`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L66) sets `sacrifice_fly: true` while line 127 sets `rbi: 0` — an impossible box-score combination under **OBR 9.08(d)**, which requires an RBI on every sacrifice fly.
  2. **Base hit where two runners (or a runner and the batter stretching) are thrown out on the bases after a run scores:** With 0 outs and runners on 2nd and 3rd, if the batter hits a double (`Outcome::Hit(2)`), the runner from 3rd scores, the runner from 2nd is tagged out at home (`Out #1`), and the batter is tagged out stretching to 3rd (`Out #2`), `outs.len() == 2` zeroes the batter's RBI on a clean extra-base hit.
  3. **Ground ball with a tag out that is not a reverse-force double play (OBR 9.04(b)(1) Comment):** With 0 outs and runners on 1st and 3rd, if the batter grounds out to 1st (`Out #1 = Force` on the batter, removing the force on R1) or if an unforced runner is tagged out after the batter is retired, **OBR 9.04(b)(1) Comment** explicitly rules that when the second out is a tag on a runner who was never forced (e.g., R2 attempting 3rd), the run scores an RBI.
- **Suggested Fix:**
  Deny the RBI under **OBR 9.04(b)(1)** only when the batter grounded into a force or reverse-force double play — i.e., `!flyout`, `!matches!(outcome, Outcome::Hit(_))`, `outs.len() >= 2`, at least one out is `OutKind::Force`, and every other out in `outs` is on a runner who started on a forced base (`1`, or `2` with R1, or `3` with R1+R2).

#### P1-2. `runs_after_dead` ([`scorer.rs:62-65`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L62-L65)) strips ALL RBIs whenever a throw goes out of play because `bb_rules` emits `BallDead` before `RunScored`, and still sets `sacrifice_fly = true` when `rbi == 0`
- **File & Line:** [`sim/crates/bb_stats/src/scorer.rs:62-66, 127`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L62-L66), [`sim/crates/bb_rules/src/engine.rs:774-808, 930`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L774-L808)
- **Evidence:**
  [`scorer.rs:62-65`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L62-L65) attempts to identify runs scored via a thrown-ball award by counting `RunScored` events that appear after `EventBody::BallDead` in the play's event vector:
  ```rust
  let runs_after_dead = if thrown_award {
      let dead_at = events.iter().position(|e| e.body_type() == EventBody::BallDead).unwrap_or(0);
      events.iter().skip(dead_at).filter(|e| e.body_type() == EventBody::RunScored).count() as u32
  } else { 0 };
  ```
  However, in [`bb_rules/src/engine.rs:924-931`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L924-L931), when a runner touches home plate during live play, `base_touched` does **not** emit `Event::RunScored` immediately; it pushes `(t, id)` to `lp.runs_pending` (waiting to see whether a third force/fly/appeal out nullifies the run). When a subsequent throw goes out of play, [`throw_out_of_play` (`engine.rs:774-808`)](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L774-L808) pushes `Event::BallDead { reason: DeadBall::OutOfPlay }` **first** (line 774) and then iterates `order` (including runners who already crossed home plate in live play, with `from = 4, to = 4, cause = Play`) and emits `Event::RunScored` **after** `Event::BallDead` (line 805)!
  Consequently:
  1. On **every** real sim play where a throw goes out of play, **every** `Event::RunScored` appears after `Event::BallDead`, so `runs_after_dead == runs` and `rbi` is **always** `0` — even for a runner who had already touched home plate in live play (`RunnerAdvanced { from_base: 4, to_base: 4, cause: Play }` or `ScorerTouch { base: 4 }` with `t <= award_cutoff_s`) before the throw left the park, or a runner from 3rd who scored on a clean single/double before an errant throw to 2nd!
  2. Conversely, when a runner on 2nd tags up on a caught outfield fly and scores *only* because the throw to 3rd goes out of play (so `rbi` is legitimately `0`), [`scorer.rs:66`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L66) checks `runs > 0` instead of `runs > runs_after_dead` (or `rbi > 0`), setting `sacrifice_fly = true` with `rbi = 0` in violation of **OBR 9.08(d)**.
- **Suggested Fix:**
  Identify runs scored by the award using `EventBody::RunnerAdvanced` where `to_base() == 4 && cause() == ev::AdvanceCause::Award` (and even then, under **OBR 9.04(a)(1)**, if the batter's play was a `Hit(k)` and the runner started at `start_base` such that `start_base + k >= 4`, the run would have scored on the hit without the throw award and still earns an RBI). Require `rbi > 0` (or at least one non-award run) in `sf` at line 66.

#### P1-3. `scorer.rs:84` compares `first_touch_s` against `t_release` (`award_cutoff_s`), misruling every infield hit with a wild throw to 1st as `ReachedOnThrowAward`
- **File & Line:** [`sim/crates/bb_stats/src/scorer.rs:60, 81, 84-85`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L84-L85), [`sim/crates/bb_stats/tests/scorer_cases.rs:131, 153`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/tests/scorer_cases.rs#L131)
- **Evidence:**
  Lines 84–85 rule:
  ```rust
  } else if thrown_award && !(evidence.has_first_touch() && before_throw(evidence.first_touch_s())) {
      Outcome::ReachedOnThrowAward
  ```
  where `before_throw(t)` checks `t <= award_cutoff_s`, and `award_cutoff_s` is `t.t_release()` of the throw.
  Consider a routine infield single deep in the shortstop hole where the batter would have beaten any ordinary-effort throw (`clean_play_margin_s = -0.35 s < 0.15 s`), and the shortstop's desperation throw to 1st goes into the dugout:
  - At `t = 2.4 s`, the shortstop fields the ball (`GloveSecured`).
  - At `t_release = 3.1 s`, the shortstop releases the throw toward 1st base.
  - At `first_touch_s = 4.2 s`, the batter touches 1st base safely.
  - At `t = 4.5 s`, the wide throw enters the dugout (`BallDead { reason: OutOfPlay }`), awarding the batter 2nd base.
  Because an infielder's throw to 1st is *always* released (`t ≈ 2.5–3.5 s`) before the batter arrives at 1st (`t ≈ 4.1–4.5 s`), `before_throw(evidence.first_touch_s())` (`4.2 <= 3.1`) is **always `false`**! Thus `scorer.rs:84` rules **every** infield single followed by a throw out of play at 1st as `Outcome::ReachedOnThrowAward` (`AB = 1, H = 0`), violating **OBR 9.05(a)(1)** and [`docs/rules-spec.md:451`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/docs/rules-spec.md#L451) (*"A batter who would have reached anyway is credited with a hit even if a fielder misplayed; the misplay may still be an error if it let a runner advance extra"*).
  In fact, the unit test [`bases_awarded_for_a_throw_out_of_play_are_not_a_hit` (`scorer_cases.rs:131, 153`)](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/tests/scorer_cases.rs#L131) explicitly sets `clean_play_margin_s: -1.0` (the batter beat the reference throw by a full second!), `t_release: 4.0`, and `touch_s: Some(5.0)`, and asserts `Outcome::ReachedOnThrowAward` — testing and enshrining the exact inverse of `rules-spec §10.2` and **OBR 9.05(a)(1)**.
- **Suggested Fix:**
  When `thrown_award` is true:
  - If the throw that went out of play was aimed at 1st base (`t.base() == 1`) before `first_touch_s`, deny the hit (`Outcome::ReachedOnThrowAward`) **only** when `clean_play_margin_s >= options.choice_margin_s` (i.e. an ordinary reference throw to 1st would have retired the batter). When `clean_play_margin_s < options.choice_margin_s`, credit the batter with `Outcome::Hit(1)` (and exclude the awarded 2nd/3rd base via `cause() != ev::AdvanceCause::Award`).
  - Furthermore, for the **A17 error seam** (**OBR 9.12**): making `ReachedOnThrowAward` a mutually exclusive `Outcome` variant cannot represent a `Hit(1)` or `FieldersChoice` plus an extra-base error on the same play, nor does `PlayRuling` identify the charged fielder or `ErrorKind`. `PlayRuling` will need a separate `errors: Vec<ChargedError>` (or `extra_base_on_throw_award: bool`) alongside `outcome: Outcome`.

#### P1-4. Walk-off base hits violate OBR 9.06(f): (1) with 0 outs, `try_walk_off` ends the play before the batter reaches 1st, causing `box_score` to charge the batter 0-for-1 (`AB = 1, H = 0`); (2) with 1–2 outs, `scorer.rs` has no OBR 9.06(f) cap on extra-base walk-off hits
- **File & Line:** [`sim/crates/bb_stats/src/scorer.rs:86-90, 110-120`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L86-L90), [`sim/crates/bb_stats/src/lib.rs:201-221`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/lib.rs#L201-L221), [`sim/crates/bb_rules/src/engine.rs:962-975`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L962-L975)
- **Evidence:**
  Under **OBR 9.06(f)**:
  > *"Subject to the provisions of Rule 9.06(g), when a batter ends a game with a safe hit that drives in as many runs as are necessary to put his team in the lead, the Official Scorer shall credit such batter with only as many bases on his hit as are advanced by the runner who scores the winning run, and then only if the batter runs out his hit for as many bases as are advanced by the runner who scores the winning run."*
  Two separate bugs break walk-off hit scoring:
  1. **Walk-off hits with 0 outs are scored 0-for-1 (`Outcome::Undetermined`, `AB = 1, H = 0`):** In [`bb_rules/src/engine.rs:966`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L966), `try_walk_off` checks `let irrevocable = !owes_retouch && (outs_left >= 3 || !open_force);`. When there are 0 outs (`outs_left == 3`) and a runner on 3rd scores on a clean single or double into the outfield at `t ≈ 3.8 s`, `try_walk_off` immediately ends the play and game **before** the batter-runner reaches 1st base (`t ≈ 4.4 s`). Because `evidence.has_first_touch()` is `false`, [`scorer.rs:89`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L89) returns `Outcome::Undetermined`, and [`box_score` (`lib.rs:216-220`)](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/lib.rs#L216-L220) increments `t.ab += 1; l.ab += 1` without incrementing `h`! Thus, in the 600 audited games (2 plays in seeds 0–299 and 4 plays in seeds 1000–1299), a batter who hits a clean game-winning single to drive in the walk-off run from 3rd is charged with **0-for-1 (`AB = 1, H = 0`)** in the box score and captioned `"Ball hit to left, 1 run scores"`.
  2. **Missing OBR 9.06(f) cap when the batter touches an extra base before the winning run touches home:** When there are 1 or 2 outs (`outs_left < 3`), `open_force` keeps the play alive until the batter touches 1st base. If the winning run started on 3rd base (`1` base needed to score) or 2nd base (`2` bases needed to score) and held/tagged up on a deep fly ball off the wall while the batter ran full speed and touched 2nd base (or 3rd base) *before* the winning run touched home plate, `evidence.touches()` records `base = 2` (or `3`) for the batter. [`scorer.rs:110-120`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L110-L120) credits `Outcome::Hit(2)` (or `Hit(3)`), with no cap limiting `value` to `4 - winning_runner_start_base` as required by **OBR 9.06(f)**.
- **Suggested Fix:**
  1. Either require the batter-runner to touch 1st base before ending a walk-off in `try_walk_off` (matching **OBR 9.06(f)**: *"only if the batter runs out his hit for as many bases as are advanced by the runner who scores the winning run"*), or when `WalkOff` occurs before `first_touch_s` on an unfielded/uncontested ball in play (`clean_play_margin_s < choice_margin_s` or `!evidence.has_secured()`), score `Outcome::Hit(1)`.
  2. On any non-home-run walk-off hit (`events` contains `GameEnded { reason: WalkOff }` and `pa.result() != PaResult::HomeRun`), find the runner whose `RunScored` put the home team ahead, look up his starting base `b` in `evidence.runners()`, and cap `value` at `(4 - b).max(1)` per **OBR 9.06(f)**.

#### P1-5. Preceding-runner putouts misclassified under OBR 9.05(b)(2) and 9.05(b)(3): (1) an unforced runner tagged out by an infielder after oversliding $b+1$ has `o.base() == b + 2`; (2) appeal outs at $b+1$ ignore outfield handlers
- **File & Line:** [`sim/crates/bb_stats/src/scorer.rs:91-101`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L91-L101), [`sim/crates/bb_rules/src/engine.rs:394-398, 922`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L394-L398)
- **Evidence:**
  Lines 93–98 classify a preceding runner's out as:
  ```rust
  let infield = evidence.has_handling() && (1..=6).contains(&(evidence.first_handler() & 0x7f));
  let one_or_return = start.is_some_and(|b| o.base() == b || (o.base() == b + 1 && !evidence.touches().is_some_and(|ts| ts.iter().any(|t| t.runner() == o.runner() && t.base() > b + 1))));
  let p = if o.kind() == ev::OutKind::Force { PrecedingPlay::ForceOut }
      else if infield && one_or_return { PrecedingPlay::InfieldOneBaseOut }
      else { PrecedingPlay::OtherOut };
  ```
  1. **Unforced runner tagged out by an infielder after touching/oversliding $b + 1$ (OBR 9.05(b)(3)):** Suppose a runner starts on 2nd (`start = 2`, unforced), advances one base to 3rd on an infield grounder, touches 3rd base ([`engine.rs:922`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L922) updates `r.at = 3`), overslides 3rd base (never touching home), and is tagged out by the third baseman before getting back to 3rd base. In [`bb_rules/src/engine.rs:394`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_rules/src/engine.rs#L394), `tag_contact` records the out with `let base = r.at + 1;` — which is `3 + 1 = 4` (`b + 2`)! Consequently, in `scorer.rs:95`, `o.base() == b || o.base() == b + 1` evaluates to `false` (`4 != 2 && 4 != 3`), `p` becomes `PrecedingPlay::OtherOut`, and the batter is wrongly credited with a `Hit` instead of `FieldersChoice` under **OBR 9.05(b)(3)**.
  2. **Runner out on appeal at $b + 1$ (OBR 9.05(b)(2)):** Under **OBR 9.05(b)(2)**, a base hit is not credited when *"a runner is called out for failing to touch the first base to which such runner is advancing, whether forced or not"* — regardless of whether an infielder or an outfielder handled the ball. In `scorer.rs:95-97`, a non-force `OutKind::Appeal` at `o.base() == b + 1` only denies the hit if `infield` is true.
- **Suggested Fix:**
  1. Determine whether a runner `o.runner()` was attempting to advance at most one base (or return to `b` or `b + 1`) from the highest base he touched (`max_touched = touches.filter(|t| t.runner() == o.runner()).map(|t| t.base()).max().unwrap_or(b)`): if `max_touched <= b + 1` and `o.base() <= b + 1` (or `max_touched == b + 1 && o.base() == b + 2` when tagged returning to `b + 1` without advancing toward `b + 2`), treat it as `one_or_return`.
  2. Treat `o.kind() == ev::OutKind::Appeal && o.base() == b + 1` as `PrecedingPlay::ForceOut` per **OBR 9.05(b)(2)**.

#### P1-6. `scorer.rs:102-104` and `118` treat throws and carries aimed at the batter-runner himself at 2nd, 3rd, or home (or thrown after `first_touch_s`) as plays on a preceding runner
- **File & Line:** [`sim/crates/bb_stats/src/scorer.rs:102-104, 118-119`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L102-L104), [`sim/crates/bb_game/src/live.rs:2705, 2811`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/live.rs#L2705)
- **Evidence:**
  Look at [`scorer.rs:102-104`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L102-L104):
  ```rust
  if preceding == PrecedingPlay::None && (evidence.throws().is_some_and(|ts| ts.iter().any(|t| t.attempted_out() && t.base() != 1)) || evidence.carries().is_some_and(|cs| cs.iter().any(|c| c.base() != 1))) {
      preceding = PrecedingPlay::UnsuccessfulAttempt;
  }
  ```
  and [`scorer.rs:118-119`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L118-L119):
  ```rust
  let on_throw_cap = evidence.throws().and_then(|ts| ts.iter().find(|t| t.attempted_out() && t.base() != 1)).map(|t| t.batter_goal().max(1));
  Outcome::Hit(on_throw_cap.map_or(value, |cap| value.min(cap)))
  ```
  Neither `ScorerThrow` nor `ScorerCarry` records *which runner* (`o.runner` in `choose_play`) the fielder was trying to retire, nor does `scorer.rs:102` check whether the throw/carry occurred **before `first_touch_s`** or whether any preceding runner was even on base!
  This produces three distinct scoring errors:
  1. **Throw to 1st is late (infield hit), followed by a throw from 1B to 2nd/3rd/home:** Suppose an infielder fields a slow roller (`clean_play_margin_s = +0.20 s`), throws to **1st base (`base == 1`)**, the batter beats the throw (because of a slow turn/release), and **after** `first_touch_s` the first baseman throws to 3rd base (`base == 3, attempted_out == true`) trying to get a lead runner taking an extra base. Even though the defense threw to 1st base on the batter and did *not* choose a preceding runner over the batter, `ts.iter().any(|t| t.attempted_out() && t.base() != 1)` matches the subsequent throw to 3rd and misrules the batter's infield hit as `Outcome::FieldersChoice`!
  2. **Play at 2nd base on the batter-runner himself with bases empty (or runners on base):** Suppose with the bases empty an infielder fields a ball where `clean_play_margin_s >= 0.15 s` (e.g., the fielder held or bobbled or took a slow turn, so the batter beat the play at 1st), and as the batter-runner rounds 1st toward 2nd, a fielder throws or carries to 2nd base (`base == 2`) to retire the **batter-runner**. Because `base != 1`, line 102 classifies the play on the batter-runner at 2nd as `PrecedingPlay::UnsuccessfulAttempt` and rules `Outcome::FieldersChoice` with **nobody on base**!
  3. **Triple/double capped by `on_throw_cap` on a throw aimed at the batter-runner himself:** Suppose the batter hits a ball into the gap (`value = 3`, triple), the outfielder throws to **2nd base (`t.base() == 2, attempted_out == true`)** to try to get the **batter-runner** while `t.batter_goal() == 2`, the batter beats the throw to 2nd base, and then continues to 3rd base (e.g., after an offline throw or relay). Because `scorer.rs:118` matches any throw with `t.attempted_out() && t.base() != 1`, it treats the throw to 2nd base **at the batter-runner** as a throw at a preceding runner and caps his triple (`Hit(3)`) to a double (`Hit(2)`), violating **OBR 9.06(b)**.
- **Suggested Fix:**
  1. Include `target_runner: u16` in `ScorerThrow` and `ScorerCarry` (or check that `target_runner != batter`).
  2. In `scorer.rs:102`, only classify a throw or carry as `PrecedingPlay::UnsuccessfulAttempt` (for **OBR 9.05(b)(4)**) if it targeted a preceding runner (`target_runner != batter`), occurred **before** ` evidence.first_touch_s()`, and preceded any throw/carry to 1st base (`base == 1`).
  3. In `scorer.rs:118`, only apply `on_throw_cap` (**OBR 9.06(b)**) to throws targeting a runner other than `batter` (`target_runner != batter`).

---

### P2 — Material Spec / Physics / Accounting / Audit Discrepancies

#### P2-1. `reference_throw_time` in [`game.rs:386-394`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/game.rs#L386-L394) omits dive recovery (`1.5 s`), turn time (`0.15 s`), moving-away footwork (`0.2 s`), and first-base bag coverage when 1B fields the ball
- **File & Line:** [`sim/crates/bb_game/src/game.rs:386-394`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/game.rs#L386-L394), [`docs/rules-spec.md:430-445`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/docs/rules-spec.md#L430-L445)
- **Evidence:**
  [`game.rs:386-394`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/game.rs#L386-L394) computes the counterfactual reference throw margin as:
  ```rust
  let flight = reference_throw_time(&mu.env, &self.field, origin, target, avg.arm);
  Some(touch - (t + avg.transfer + flight))
  ```
  Compare this against the actual throw physics in [`live.rs:2743-2750`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/live.rs#L2743-L2750) (`agent-models §5.5`):
  1. When a fielder dives to stop a ground ball (`fd.dove == true`), he is prone on the ground at `t` (`GloveSecured`) and cannot release a throw for `DIVE_GETUP_S + DIVE_TRANSFER_EXTRA_S = 1.80 s`, plus `turn_extra_s` (up to `0.15 s`). `game.rs:393` assumes the diving fielder releases the throw at `t + avg.transfer` (`t + 0.65 s`, while still lying flat on the turf!) — making the reference throw arrive up to **1.95 seconds** earlier than physically possible. If the diving shortstop gets up and makes an unsuccessful throw to 2nd or 3rd, `clean_play_margin_s` is inflated by `~1.95 s`, stripping the batter of a clear hit.
  2. When the **first baseman (`pos == 2`)** fields a ground ball wide of first base and throws to 2nd base, `game.rs:393` assumes a throw to 1st base arrives at `t + avg.transfer + flight` without checking whether the pitcher (`f[0]`) or second baseman (`f[3]`) could reach 1st base in time to receive the throw.
- **Suggested Fix:**
  Include the fielder's physical posture/orientation penalty from `GloveSecured` (dive recovery, moving-away footwork, and turn angle toward 1st base) and, when `pos == 2`, lower-bound the reference arrival time at 1st by the nearest covering teammate's run time to `base_pos(1)`.

#### P2-2. Audit Table 2 vs Table 3 in [`2026-11-14-scorer.md:71-90`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/docs/calibration/2026-11-14-scorer.md#L71-L90): inside-the-park home runs (`34` in seeds 0–299, `42` in seeds 1000–1299) are counted IN play in Table 2's BABIP but excluded as `HR` in Table 3's BABIP
- **File & Line:** [`docs/calibration/2026-11-14-scorer.md:71-90`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/docs/calibration/2026-11-14-scorer.md#L71-L90), [`sim/crates/bb_stats/examples/scorer_audit.rs:56, 68-74, 132`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/examples/scorer_audit.rs#L56), [`sim/crates/bb_stats/src/lib.rs:204-214`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/lib.rs#L204-L214)
- **Evidence:**
  Let's check the arithmetic across Tables 1, 2, and 3 in `docs/calibration/2026-11-14-scorer.md`:
  - **Table 1 & Table 2 internal arithmetic holds:**
    - Seeds 0–299: Old FC = `422 = 15 (now hit) + 399 (still FC) + 8 (now throw award)`. Old BIP hits = `5,535 = 2,940 + 2,083 + 484 + 28`. New BIP hits = `5,535 + 15 - 62 (old hit -> FC) - 22 (old hit -> throw award) = 5,466` (`2,871 + 2,083 + 484 + 28`). Total new FC = `399 + 62 = 461` (`303 + 91 + 61 + 6`). Total throw awards = `22 + 8 = 30` (`27 + 2 + 1 + 0`). Total BIP = `8,477 + 2,885 + 3,501 + 1,415 = 16,278`. `5,466 / 16,278 = .33579 ≈ .336`.
    - Seeds 1000–1299: Old FC = `420 = 28 + 387 + 5`. Old BIP hits = `5,522`. New BIP hits = `5,522 + 28 - 54 - 26 = 5,470`. New FC = `387 + 54 = 441`. Throw awards = `26 + 5 = 31`. Total BIP = `16,327`. `5,470 / 16,327 = .33503 ≈ .335`.
  - **Discrepancy between Table 2 ("Hit rate per ball in play (home runs are not in play)") and Table 3 (`BABIP = (H - HR) / (AB - K - HR + SF)`):**
    Compare the hit and BIP totals between Table 2 and Table 3:
    - On seeds 0–299:
      - Table 2 has `scorer hits = 5,466` and `BIP = 16,278` (`.3358`).
      - Table 3 has `H - HR = 6,605 - 1,173 = 5,432` (**`34` fewer** than `5,466`!) and `AB - K - HR + SF = 21,094 - 3,722 - 1,173 + 45 = 16,244` (**`34` fewer** than `16,278`!), giving `5,432 / 16,244 = .3344`.
    - On seeds 1000–1299:
      - Table 2 has `scorer hits = 5,470` and `BIP = 16,327` (`.3350`).
      - Table 3 has `H - HR = 6,618 - 1,190 = 5,428` (**`42` fewer** than `5,470`!) and `AB - K - HR + SF = 21,158 - 3,735 - 1,190 + 52 = 16,285` (**`42` fewer** than `16,327`!), giving `5,428 / 16,285 = .3333`.
    **Why do both numerator and denominator differ by exactly `34` (seeds 0–299) and `42` (seeds 1000–1299)?**
    Because in [`scorer_audit.rs:56, 68`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/examples/scorer_audit.rs#L56), Table 2 filters by `PaResult::BattedBall` (which excludes only over-the-fence home runs) and counts `Outcome::Hit(4)` (**inside-the-park home runs**: `34` in seeds 0–299 and `42` in seeds 1000–1299!) in `c.bip[t]` and `c.new_hits[t]` — despite Table 2's title claiming *"home runs are not in play"* and `scorer_audit.rs:122` labeling the column `"new BABIP"`. Meanwhile, [`box_score` (`lib.rs:210`)](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/lib.rs#L210) folds `Outcome::Hit(4)` into `t.hr`, so Table 3's `(H - HR) / (AB - K - HR + SF)` subtracts all `1,139 + 34 = 1,173` home runs (including the `34` inside-the-park HRs). Standard BABIP (`(H - HR) / (AB - K - HR + SF)`) excludes all `HR`; Table 2's per-type breakdown should either exclude `Outcome::Hit(4)` to match BABIP or state that inside-the-park HRs are included in Table 2 and excluded in Table 3.

#### P2-3. Backward compatibility on pre-A12 logs: `score_play` mis-scores sacrifice flies as `Outcome::Out, sacrifice_fly: false` (`AB += 1`) instead of `Undetermined`, and mis-scores all hits as `Out` if a 4-field `ScorerEvidence` is present
- **File & Line:** [`sim/crates/bb_stats/src/scorer.rs:55-56, 66, 77-90`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L55-L90), [`docs/calibration/2026-11-14-scorer.md:134-136`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/docs/calibration/2026-11-14-scorer.md#L134-L136)
- **Evidence:**
  The calibration note ([`2026-11-14-scorer.md:134-136`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/docs/calibration/2026-11-14-scorer.md#L134-L136)) claims that on an old log without the new fields, the scorer returns `Undetermined` rather than a wrong ruling. In fact:
  1. On a pre-A12 log where `ScorerEvidence` and `BattedBallRuling` are absent (`evidence == None`), when a batter hits a **sacrifice fly** (`flyout == true, outs_before < 2, runs > 0`), [`scorer.rs:55-56`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L55-L56) evaluates `caught_foul = false` and `caught_in_outfield = false`, so `sf = false`, and [`scorer.rs:77`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L77) returns `Outcome::Out` (charging the batter with an At-Bat instead of `Outcome::Undetermined` or `sacrifice_fly: true`).
  2. If a log contains a `ScorerEvidence` table written with only the original 4 schema fields (`id: 0..3`, before `has_first_touch` was added at `id: 7`), `evidence` is `Some(e)` and FlatBuffers defaults the missing `has_first_touch()` field to `false`. Then [`scorer.rs:86`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/scorer.rs#L86) (`else if !evidence.has_first_touch()`) returns **`Outcome::Out` for every single, double, triple, and fielder's choice** in that log!
- **Suggested Fix:**
  1. When `flyout && outs_before < 2 && runs > 0 && evidence.is_none()`, return `Outcome::Undetermined` because catch location / foul status cannot be determined from the log.
  2. Check `evidence.runners().is_some()` (an A12 presence indicator, since A12 always populates `runners` with at least the batter) before trusting `evidence.has_first_touch() == false` as proof that the batter failed to reach 1st base.

#### P2-4. Accounting tests in [`accounting.rs`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/tests/accounting.rs#L31-L40) test internal `box_score` tautologies (`I1`, `I2`, `I7`) that still hold when `score_play` rulings are completely wrong
- **File & Line:** [`sim/crates/bb_stats/src/lib.rs:184-226`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/lib.rs#L184-L226), [`sim/crates/bb_stats/tests/accounting.rs:31-40`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/tests/accounting.rs#L31-L40)
- **Evidence:**
  Question 4 asks whether the accounting tests are independent of the scorer's own arithmetic and to name an identity that holds while a ruling is wrong:
  1. **`I2` (`PA = AB + BB + HBP + SF + SH`, [`accounting.rs:33`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/tests/accounting.rs#L33)) is a structural tautology inside `box_score` ([`lib.rs:200-221`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/lib.rs#L200-L221)):** Every `PlateAppearanceEnded` increments `pa += 1` and then branches on `ruling.outcome`, incrementing `bb`, `hbp`, `sf`, or `ab`. Even when `ruling.outcome == Outcome::Undetermined`, [`lib.rs:216-217`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/lib.rs#L216-L217) increments `ab += 1` so `I2` cannot fail.
  2. **`I1` (`AB + BB + HBP + SH + SF + CI = R + LOB + opponent putouts`) is tested at [`accounting.rs:35`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/tests/accounting.rs#L35) as `t.pa == t.runs + t.left_on_base + outs`, and [`lib.rs:184-196`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/lib.rs#L184-L196) tracks `runners` (`LOB`) from `sc.has_first_touch()` and `RunnerAdvanced`, NOT from `ruling.outcome`!** Moreover, on an `Undetermined` walk-off before `first_touch`, [`lib.rs:184-189`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/lib.rs#L184-L189) artificially inserts the batter into `runners` (`left_on_base`) specifically to force `I1` to balance.
  3. **Identities that hold while rulings are completely wrong:**
     - If `score_play` misruled **every** single, double, and triple as `Outcome::FieldersChoice` (or `Outcome::Out`), **every** identity in `accounting.rs` (`I1`, `I2`, `I4`, `I5`, `I7`, `I8`, `I10`) would still pass (`H = HR`, `1B = 2B = 3B = 0`, `AB` unchanged, `LOB` unchanged).
     - If `score_play` misruled **every** double and triple as `Outcome::Hit(1)`, `I7` (`H = 1B + 2B + 3B + HR`) and all other identities would still pass.
     - If `score_play` misruled **every** sacrifice fly as `sacrifice_fly: false` (`Outcome::Out`), `SF` would drop by 1 and `AB` would rise by 1, so `I1` and `I2` would still hold.
     - No identity in `accounting.rs` checks `t.rbi` at all (e.g., `t.rbi <= t.runs`).
- **Suggested Fix:**
  Add independent cross-checks in `accounting.rs`:
  1. Verify `I1` using its actual left-hand side (`t.ab + t.bb + t.hbp + t.sf + t.sh == t.runs + t.left_on_base + outs`) and assert `t.undetermined == 0`.
  2. Assert `t.rbi <= t.runs` and `t.rbi >= t.hr` (every home run drives in at least the batter).
  3. Independently recompute `LOB` from the rules engine's `HalfInningEnded.left_on_base()` plus the count of third-out plays made on a preceding runner before the batter reached 1st base (**OBR 9.02(g)**), rather than relying solely on `box_score`'s internal `BTreeSet`.

---

### P3 — Captions (`describe.rs`) and Minor Issues

#### P3-1. Captions in [`describe.rs`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/describe.rs#L35-L118) omit sacrifice flies, double plays on base hits, and ground-rule doubles, and read in-memory `facts` (`GloveSecured`) instead of `ScorerEvidence` (`first_handler`)
- **File & Line:** [`sim/crates/bb_game/src/describe.rs:14-17, 58, 71-90, 105-107, 159`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/describe.rs#L58-L107)
- **Evidence:**
  1. **Sacrifice flies are never captioned as `"Sacrifice fly"` ([`describe.rs:71-74, 159`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/describe.rs#L71-L74)):** Even though `ruling: Option<PlayRuling>` is passed to `describe_scored`, `ruling.sacrifice_fly` is never inspected. A sacrifice fly is captioned `"Fly out to right, 1 run scores"` (see [`describe.rs:159`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/describe.rs#L159) and `2026-11-14-scorer.md` samples) instead of `"Sacrifice fly to right, 1 run scores"`.
  2. **Base hits with two outs on the bases drop both `{multi}` (`"Double play: "`) and the out description ([`describe.rs:88-89`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/describe.rs#L88-L89)):**
     ```rust
     let extra = if batter_out.is_some() { ", batter out stretching" } else if outs.len() == 1 { ", runner out on the bases" } else { "" };
     format!("{name}{to}{extra}")
     ```
     If `outs.len() == 2` on a base hit (e.g., two preceding runners thrown out on the bases), `batter_out.is_some()` is `false` and `outs.len() == 1` is `false`, so `extra` is `""` and `{multi}` is omitted — producing just `"Single to left"` for a double play! If one runner is thrown out on the bases and the batter is out stretching (`outs.len() == 2`), it prints `"Single to left, batter out stretching"`, omitting the other out and `"Double play"`.
  3. **Ground-rule doubles are captioned `"Double"` instead of `"Ground-rule double"` ([`describe.rs:76-90`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/describe.rs#L76-L90)):** `events` contains `Event::BallDead { reason: DeadBall::GroundRuleDouble }`, which `describe_scored` ignores.
  4. **Fielder attribution reads in-memory `facts` (`Input::GloveSecured`) instead of `first_handler` ([`describe.rs:14-17, 58`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/describe.rs#L14-L17)):** When a pitcher deflects a comebacker (`first_handler = 1`) and the shortstop picks it up (`GloveSecured = 6`), the scorer judges infield handling from the pitcher while `describe_scored` says `"to short"`, and `describe_scored` still depends on in-memory `facts` (`&[bb_rules::Input]`) rather than being reconstructible purely from the event log.
- **Suggested Fix:**
  1. When `ruling.is_some_and(|r| r.sacrifice_fly)`, format the caption as `"{multi}Sacrifice fly{to}..."`.
  2. In the `_ if hit =>` arm (`describe.rs:76-90`), include `{multi}` and handle `outs.len() >= 2` (and check `Event::BallDead { reason: DeadBall::GroundRuleDouble }` to print `"Ground-rule double{to}"`).

---

## Answers to Review Questions Not Covered Above

- **Invariant 6 (Statistics as pure functions of the event log) & Invariant 2 (Agent isolation from `reference_throw_time`):**
  - `bb_stats::box_score` and `bb_stats::scorer::score_play` take only `&[u8]` / `ev::PlayRecord<'_>` from the FlatBuffers log and use no simulation state or RNG.
  - `reference_throw_time` ([`sim/crates/bb_game/src/game.rs:17-39, 386-394`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_game/src/game.rs#L17-L39)) is computed in `Game::step_pitch` *after* `simulate_live` has returned. Nothing in `bb_agents` or `live.rs` calls `reference_throw_time` or reads `ScorerPlayEvidence`.
- **Replayability, schema wire compatibility, and digest chain:**
  - All new fields in `ScorerEvidence` ([`sim/schemas/events.fbs:248-267`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/schemas/events.fbs#L248-L267)) are appended with monotonically increasing FlatBuffers field IDs (`id: 4..17`), preserving wire compatibility.
  - The BLAKE3 digest chain changes because `PlayRecord` now carries `ScorerEvidence`, `BattedBallRuling`, and `RunnerAdvanced.cause`; this is accurately documented in [`docs/metrics-ledger.md:193`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/docs/metrics-ledger.md#L193) and [`docs/calibration/2026-11-14-scorer.md:136`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/docs/calibration/2026-11-14-scorer.md#L136).
  - [`box_score`](file:///home/natex/Projects/3D%20Baseball/.claude/worktrees/review-a10/sim/crates/bb_stats/src/lib.rs#L134) uses `BTreeSet` and slice iteration; there is no `HashMap` or non-deterministic iteration.

---

## Verification

### Executed Commands
- `export PATH=$HOME/.cargo/bin:$PATH && CARGO_BUILD_JOBS=2 taskset -c 0-3 nice cargo test --release --manifest-path sim/Cargo.toml -p bb_stats`
  - Unit tests (`src/lib.rs`): 3 passed (`failed_choice_uses_reference_throw_and_preserves_unknown`, `outfielder_retires_runner_taking_extra_base`, `successful_force_or_infield_one_base_play_denies_hit`).
  - `tests/accounting.rs`: 4 passed (`identities_hold_over_many_games`, `pitch_and_pitcher_accounting_over_games`, `live_ball_runner_terminal_events_are_disjoint`, `batting_lines_name_the_roster_and_follow_the_order`).
  - `tests/rosterless.rs`: 1 passed (`a_log_without_rosters_still_reads_and_counts`).
  - `tests/scorer_cases.rs`: 11 passed.
- `export PATH=$HOME/.cargo/bin:$PATH && CARGO_BUILD_JOBS=2 taskset -c 0-3 nice cargo test --release --manifest-path sim/Cargo.toml -p bb_game --test scorer_evidence`
  - `tests/scorer_evidence.rs`: 1 passed (`live_contacts_and_throws_survive_log_round_trip`).

### Files Read and Inspected
- `CLAUDE.md`
- `docs/calibration/2026-11-14-scorer.md`
- `docs/rules-spec.md` (§9.3–§15)
- `docs/data-formats.md`
- `docs/metrics-ledger.md`
- `sim/schemas/events.fbs`
- `sim/crates/bb_stats/src/scorer.rs`
- `sim/crates/bb_stats/src/lib.rs`
- `sim/crates/bb_stats/tests/scorer_cases.rs`
- `sim/crates/bb_stats/tests/accounting.rs`
- `sim/crates/bb_stats/tests/rosterless.rs`
- `sim/crates/bb_stats/examples/scorer_audit.rs`
- `sim/crates/bb_game/src/game.rs`
- `sim/crates/bb_game/src/live.rs`
- `sim/crates/bb_game/src/eventlog.rs`
- `sim/crates/bb_game/src/describe.rs`
- `sim/crates/bb_game/tests/scorer_evidence.rs`
- `sim/crates/bb_rules/src/engine.rs`

---

## Resolution

Resolved on branch `work/scorer-review-fixes` (2026-11-14). Every finding was verified first with a
failing case built from the rule's text (`sim/crates/bb_stats/tests/scorer_rules.rs`; 17 of its 23 cases fail on
the scorer as reviewed, the other six are guards for rulings that were already right), then fixed or
deferred. No finding was wrong; one (P1-5, part 1) cannot be tested in the sim as it stands.

| Finding | Verdict | What was done |
|---|---|---|
| **P1-1** RBI zeroed on any two outs | confirmed (OBR 9.04(b)(1)) | RBI withheld only for a force or reverse-force double play (every out a force, or a tag on a runner who was forced until the first out removed it); a sacrifice fly or hit with a later out on the bases keeps it. Four cases (sacrifice fly with a tag on R1; hit with two outs on the bases; unforced second out; the two double plays). A strikeout drives in nothing. |
| **P1-2** throw out of play strips RBI; sacrifice fly with zero RBI | confirmed | Runs are told apart by whether the runner touched home before the dead ball (`ScorerTouch`), not by event order: such a run is an ordinary one; a run given by the award counts only if the batter's hit would have scored him (start base + hit bases ≥ 4, 9.04(a)(1)). A sacrifice fly needs a credited run. Three cases. |
| **P1-3** infield hit + wild throw ruled "reached on throw" | confirmed, and the old test enshrined it | When the throw that went out was at first, the batter's reference margin decides: ≥ `m_fc` (0.15 s) → `ReachedOnThrowAward`; less → a hit with the award out of its value, whether or not he touched first before the throw left. The play ending before his touch uses his estimated arrival (`batter_eta_s`, new evidence). The test `bases_awarded_for_a_throw_out_of_play_are_not_a_hit` had the batter beat the throw by 1.0 s and expected no hit; rewritten. A `PlayRuling` that carries an error plus a hit is for A17 (the seam stays one outcome until errors exist). |
| **P1-4** walk-off hits | confirmed, both parts | A walk-off with none out that ends before first is ruled from his ETA (a hit, at least a single; no longer 0-for-1); a walk-off hit is capped at the bases the winning runner advanced (9.06(f)); a home run is not capped (9.06(g)). The box score counts him on base (LOB) as before. Three cases. |
| **P1-5** preceding-runner putouts | part 2 confirmed and fixed; part 1 **deferred** | (2) An appeal out at the next base denies the hit whoever handled the ball (9.05(b)(2)). (1) An unforced runner oversliding the base and tagged out returning (9.05(b)(3)) is recorded with the same base as an extra-base try (`r.at + 1`), and the sim's runners stop dead on a base, so the case cannot occur; it needs the runner's direction at the tag in the log. Written in rules-spec §10.2. |
| **P1-6** plays on the batter-runner counted as plays on a preceding runner | confirmed | `ScorerThrow.runner` and `ScorerCarry.runner` added to the schema (appended) and filled from the option the holder chose; the scorer counts a throw or carry only if it was meant for a runner ahead of the batter, made while he was still short of first, and caps his hit only on such a throw. Five cases (play on the batter-runner, carry on him, after he was safe, before and after, triple not capped). |
| **P2-1** reference throw omits the set-up | confirmed | The reference now has the actual fielder's get-up after a dive, turn and footwork at the catch (`LiveOutput.ref_set_s`) with the grade-50 transfer, and a first baseman may take the bag himself (the quicker of that and the flip). Not covered: a covering pitcher or second baseman on a first-baseman play (the carry bound stands in). |
| **P2-2** audit Tables 2 and 3 disagree by the inside-the-park home runs | confirmed | The audit leaves inside-the-park home runs out of balls in play; the two tables now agree (16,172 balls in play and 5,184 hits both ways, seeds 0–299). The note says so. |
| **P2-3** pre-A12 logs | confirmed | A fly out with a run and no evidence is `Undetermined`; evidence without a runner list (the first four fields) is read as absent. Sacrifice flies now follow who handled the ball (outfielder, or infielder in the outfield, 9.08(d)); the 45 m radius covers the infielder-in-the-outfield case only. The existing location-only test was wrong and was rewritten. |
| **P2-4** accounting tests are tautologies | confirmed | I1 is now written with its own left side (AB + BB + HBP + SH + SF); RBI ≤ runs, RBI ≥ HR, SF ≤ RBI; no unresolved rulings. Three whole games were scored by hand and pinned as golden logs (seeds 3, 7, 1001; `tests/hand_scored.rs`, judgement calls in its header): 54 lines, all matching. **Not done:** recomputing LOB from the engine's own `HalfInningEnded.left_on_base` plus stranded batters as a separate path. |
| **P3-1** captions | confirmed except one point | Sacrifice fly, ground-rule double, and every out on the bases of a hit are now said. **Deferred:** reading the fielder from `ScorerEvidence` instead of the live facts (a captions-from-the-log change; "to short" after a pitcher's deflection is the fielder who has it, which is what a caption says). |

Verification: `cd sim && cargo test --workspace --release` passes (281 tests); the play-by-play of seeds 0–299, 1000–1299 and 0, 1, 7, 42 is identical to main `f3dc029` (event-log digests change: the log carries the new facts); audit rerun in `docs/calibration/2026-11-14-scorer.md`.
