# Rules Specification

**Last updated:** 2026-11-13
**Status:** Draft v1. Gates milestone M4 (walking-skeleton game). M5 additions and substitution rules (M6) are specified here too so the state machine is designed for them from the start.
**Parent:** [PLAN.md](../PLAN.md) §9 · **Siblings:** [physics-spec.md](physics-spec.md), [agent-models.md](agent-models.md), [data-formats.md](data-formats.md), [calibration-targets.md](calibration-targets.md)

This document defines the **rules engine** (`bb_rules`): the game state machine, every umpire judgment and the geometry or timing it rests on, base awards, substitution legality (including the DH and the three-batter minimum), the official scorer, the accounting identities, and the **scenario corpus** that tests all of it without physics.

Status tags: **[S]** sourced (here, the Official Baseball Rules, "OBR", cited by rule number; numbers re-checked in task **R11**), **[D]** derived, **[E]** estimate (a named, configurable parameter), **[C]** design choice the owner may revisit.

**D1 reminder.** The rules engine contains **no randomness at all**. It does not hold an RNG handle. Every call is a deterministic function of geometric and timing facts reported by the world. The umpire is exact (D11); a fallible human-umpire mode is deferred.

---

## 1. Architecture

### 1.1 Boundaries

```
bb_world ──(physical events)──▶ bb_game ──(RulesFacts)──▶ bb_rules ──(rules events, RulesState)──▶ bb_game
                                                 ▲                         │
                     manager commands ───────────┘                         └──▶ legal-command set, umpire signals
```

- `bb_rules` depends only on `bb_math` and its own fact types. It **cannot see** world state, agents, or physics (PLAN §15). This is what lets the scenario corpus (§12) drive it with hand-written facts.
- `bb_game` translates world contacts into **RulesFacts** (§2): "ball touched ground at (x, y)", "runner R's body entered base 2's volume at t". The translation is mechanical geometry; any judgment belongs in `bb_rules`.
- The core is a pure step function:
  `step(RulesState, RulesInput) → (RulesState′, Vec<RulesEvent>)`
  where `RulesInput` is a fact or a validated command. No other entry point mutates state.
- Rules events are the "rules" rows of data-formats §4.3 (`PitchResult`, `RunnerAdvanced`, `RunnerOut`, `RunScored`, `PlateAppearanceEnded`, …). New events this spec needs are listed in §15.

### 1.2 Time and ordering

- Facts carry play time `t` (s) and arrive sorted by `(t, source order)`. `bb_game` guarantees ordering; `bb_rules` asserts it.
- Two facts with **exactly equal** `t` are ordered by a fixed priority: base touches before tags and force-outs, which come before ball-dead facts [C]. In effect, an exact tie goes to the runner. With continuous-time event detection (physics spec §5.3), exact ties essentially never happen; the rule exists so the outcome is defined, not to model "tie goes to the runner" folklore.
- `bb_rules` never compares times it was not given. A question like "who arrived first" is answered by which fact arrived first.

### 1.3 Umpire signals

The umpire's calls are information that agents act on (a runner tags up after seeing "catch"). When `bb_rules` emits a judgment, `bb_game` publishes an **umpire signal** to agents' observations after a latency `T_signal` = 0.2 s [E]. Agents never read `RulesState` directly; they see the scoreboard (count, outs, bases) and these signals, like real players.

**Visible out calls (owner request, 2026-11-12).** Each out also gets the umpire's visible signal, which the timeline carries as an `UmpireCall` marker (data-formats §5.4) and the client shows as it happens, not at the end of the play. Its delay after the out (the deciding fact's time) depends on how close the play was. The margin is the runner's time short of the base he needed: his base-path distance from it at the out over his top speed (the base forced to or appealed at; for a tag, the base he was heading for). A caught fly is routine unless the catch was diving or below `z_low_catch` = 0.4 m [E] (margin 0), and a strikeout is called with the pitch (margin 0). Delay = `T_call_close` = 0.2 s [E] for margins up to 0.15 s [E], rising linearly to `T_call_routine` = 0.7 s [E] at 0.65 s [E] and beyond: at once on a bang-bang play, within a second on a routine one. This is presentation only: the out was made when its fact arrived, and agents still see outs at once (`bb_game::umpire`).

---

## 2. Input Facts

The complete vocabulary `bb_game` may send. Each is a plain record; positions in the sim frame (physics spec §1.2), SI units.

| Fact | Fields | Produced when |
| :--- | :--- | :--- |
| `PitchDelivered` | pitcher, t_release | Release |
| `PitchCrossed` | t, ball center at the judgment plane (x, z), zone top/bottom, `bounced_before_plane` | Physics spec §10 |
| `SwingMeasured` | max bat-yaw angle reached, t_contact or none, `offered_bunt` | End of swing (agent-models §3.6) |
| `BatContact` | t, contact point | Physics spec §7 |
| `BallTouched` | t, what (`Ground{surface}`, `Wall{segment}`, `FoulPole`, `Base{n}`, `Body{agent}`, `Glove{agent}`, `Umpire`, `OutOfPlay{area}`), ball position, ball radius-edge distance to each foul line | `Wall`: any wall of the playing-field boundary below its top, the backstop and foul-territory walls included. `OutOfPlay`: the ball crossed the boundary over a wall's top or into a dugout (physics-spec §9.5), with the exit point and `in_flight` (no ground touch since the bat or the last throw's release) | Every ball contact |
| `BallPassedBase` | t, base (1 or 3), side (fair/foul) | Ball center crosses the line through 1B or 3B perpendicular to the foul line, while on or near the ground |
| `BallSettled` | t, position | Ball at rest |
| `GloveSecured` | t, fielder, `ball_in_hand_or_glove` | Capture held continuously for `t_secure` = 0.3 s [E] after capture (agent-models §5.4) |
| `BallReleasedFromGlove` | t, fielder, kind (`Throw`, `Drop`, `Knocked`), `infielder` (the fielder plays P, C, 1B, 2B, 3B or SS) | Loss of possession. The live sim sends `Throw` at every throw's release (since 2026-11-14, for §7.2); `Drop` and `Knocked` have no producer yet |
| `BaseTouched` / `BaseLeft` | t, agent, base | Body capsule enters or leaves the base volume (physics spec §9.4) |
| `TagContact` | t, fielder, runner, `holding_ball` | Glove or ball-holding hand touches a runner capsule |
| `RunnerPosition` | t, runner, ground point (x, y) | About 30 Hz per runner while a play is live, every runner at the same tick times (§5.5 compares samples with equal t); for the overrun of first and passing (§5.5), later the base-path check (M5). In `bb_rules` it is its own type (`RunnerPosition`), not an `Input` variant; `step` takes either. (Changed 2026-11-13 from "nearest base path point, heading": the engine projects the point itself, and no rule needs the heading.) |
| `PitcherOnRubber` / `PitcherOffRubber` | t | Set/step-off (M5 balks and pickoffs) |
| `BodyContact` | t, agent A, agent B | Fielder–runner contact (M5 obstruction/interference) |

Commands arrive as already-parsed `CommandEnvelope`s (data-formats §6.2) and are validated by §9.

---

## 3. Rules State

```
RulesState {
  game:       { inning, half, status: Pregame|InProgress|Final, score_by_inning[2][], visits_used[2] }
  lineup[2]:  { slots[9]: PlayerSlot, dh_active: bool, pitcher: AgentId, used: Set<PlayerId> }
  defense[2]: { position → AgentId }                         // 9 positions
  pa:         { batter, pitcher, balls, strikes, pitches_this_pa, charged_batter, charged_pitcher }
  outs:       0..=3
  bases[3]:   Option<{ runner, responsible_pitcher, reached_on_error: bool }>
  play:       PlayState                                     // §5
  pitcher_obligation: { batters_faced_this_stint, half_inning_started_with: bool, satisfied: bool }
  next_up[2]: slot index
}
```

`PlayerSlot` holds the current player in that batting slot and their position (`DH` is a position). `charged_batter` and `charged_pitcher` implement mid-PA substitution charging (§9.5). `responsible_pitcher` implements inherited runners (§10.6).

Invariants checked after every `step` (debug builds and scenario tests): outs ≤ 3; no two runners on one base when no play is live; every runner on base is in the batting team's lineup; nine distinct fielders; `used` players never reappear.

---

## 4. Game and Plate-Appearance Flow

### 4.1 Game state machine

```
Pregame ─▶ HalfInningStart ─▶ PlateAppearance ⟲ ─▶ HalfInningEnd ─┬─▶ HalfInningStart
                                                                     └─▶ Final
```

- **Pregame:** validate both lineups (§9.1). Emit `GameStarted`.
- **HalfInningStart:** outs = 0, bases empty (classic extra innings: no automatic runner, D10). Reset `pitcher_obligation.half_inning_started_with` (§9.3).
- **HalfInningEnd:** on the third out, after run-scoring resolution (§6.4). Emit `HalfInningEnded` with LOB: the number of **runners**, by identity, left standing on a base (a protected overrunner counts on first, §5.5), so two runners sharing a base at the third out are 2, not 1 [S: OBR 5.06(a)(2); I1, I10] (fixed 2026-11-13, Codex rules-passing review #5: LOB was counted from one slot per base; 10 of 512 games had such a third out).
- **Game end** [S: OBR 7.01]:
  - After the top of the 9th or any later inning, if the home team leads: **Final**, bottom not played.
  - After the bottom of the 9th or later, if the score is not tied: **Final**.
  - **Walk-off:** in the bottom of the 9th or later, the moment the home team takes the lead the game ends. Only the runs needed to win count, and the batter is credited with only as many bases as the winning runner needed, **except** a fair ball out of the park, where the batter and all runners score [S: OBR 7.01(e)(3) in the 2026 edition; earlier editions numbered it 7.01(g)]. The play finishes resolving the winning run's touch of home; later facts are ignored. A winning touch is final only once the run cannot be cancelled: no runner can still be forced out for a third out, no retouch is owed, and the batted ball is down fair or caught. A touch made while a fly is still in the air waits; if the ball drops in fair the game ends then, and if it is caught the runner owes a retouch (§5.3) and wins it only on his next touch of home (runner-return review #1, 2026-11-12).
  - Ties go to extra innings indefinitely. No mercy rule, no ties, no weather.
- **Liveness guard:** a game reaching inning 30 emits a `LivenessWarning` and continues. It is reported by calibration (W7 family) and never ends a game early.

### 4.2 Decision points

Before every pitch the state is at a **decision point** (data-formats §6.1): the ball is dead or "time" is in effect. Commands are validated and applied here, in submission order, then the next pitch is delivered. Commands that need a dead ball (substitutions, alignment, mound visits) are legal only here.

### 4.3 Plate appearance

`PlateAppearanceStarted` → pitches (§4.4) until a PA-ending result → `PlateAppearanceEnded` → `next_up` advances.

**Intentional walk** [S: OBR 5.05(b)(1), no-pitch IBB]: a command at a decision point awards the batter first base with forced advances (§7.1). No pitches are thrown. The PA ends with `IntentionalWalk`.

### 4.4 Judging a pitch

Facts for one pitch are judged in this order. The first rule that applies decides `PitchResult`.

1. **Batter hit by the pitch** (`BallTouched{Body{batter}}` before any bat contact or catcher capture):
   - If the batter swung (§4.5): **strike**, ball dead.
   - Else if the pitch was in the strike zone when it touched (geometric test of physics spec §10 at the touch point projected to the plane): **strike**, ball dead.
   - Else: **hit by pitch**, ball dead; award first base (§7.1), including after a bounce.
   - The OBR requirement that the batter try to avoid the pitch is not judged in v1 [C]: the batter agent always attempts evasion (agent-models §3.7), so the case cannot arise from intent.
2. **Bat contact** (`BatContact`):
   - **Foul tip** if the first touch after the bat is the catcher's glove, the catcher secures it (`GloveSecured`), and the bat-to-glove time is ≤ `t_tip` = 0.12 s [E] ("sharp and direct", OBR definitions). A foul tip is a **strike** and the ball stays live (runners may steal).
   - Otherwise the ball is **batted**: enter the live-ball play (§5). The pitch result (fair, foul, in play) is recorded when §5.2 decides it.
3. **Swing and miss:** **strike**. The ball stays live.
4. **No swing:** **called strike** if the zone test passes and the pitch did not bounce before the plane; otherwise **ball**. A pitch that hits the plate or ground before the plane is always a ball when taken [S].
5. **Pitch touches the umpire:** no special effect; the ball stays live.

**Count transitions:**

| Situation | Result |
| :--- | :--- |
| Ball four | **Walk.** Ball stays live. Batter to first; forced runners advance one base (§7.1); unforced runners may try to advance at their own risk |
| Strike three, caught by the catcher (`GloveSecured` with no ground contact first) | **Strikeout**, batter out |
| Strike three, not caught (bounced first, or not secured), and (first base unoccupied **or** two outs) | **Uncaught third strike**: batter becomes a runner; ball live. The batter is out if tagged or if first base is touched by a fielder holding the ball before the batter reaches it [S: OBR 5.05(a)(2), 5.09(a)(3)] |
| Strike three, not caught, first base occupied, fewer than two outs | **Strikeout**, batter out; ball live for runners |
| Foul with fewer than two strikes | Strike |
| Foul with two strikes | No change (a foul bunt with two strikes is a strikeout, M5) |
| Foul tip with two strikes | Strikeout (caught by definition); ball live |

### 4.5 Check swing

A batter has **swung** if the bat's horizontal yaw, measured from its pose at the start of the swing, reaches the point where the bat is **perpendicular to the pitcher–home line** (barrel head level with the front of the batter) [C]: `max_yaw ≥ θ_swing`, with θ_swing a configurable angle defaulting to that geometry. The swing model reports `max_yaw` (agent-models §3.6). Any bat contact is a swing regardless of yaw, unless it is a bunt attempt pulled back (M5).

---

## 5. The Live-Ball Play

### 5.1 Play state

```
PlayState = Dead
          | Live { ball: BallStatus, runners: Vec<RunnerState>, forces: ForceMap, t_start }

BallStatus = BattedUnresolved { first_touch: Option<Touch> }   // fair/foul not yet known
           | Fair { caught_fly: Option<CatchState> }
           | Thrown | Pitched                                  // live after a pitch (steals, uncaught strike 3)
RunnerState = { runner, last_base_touched, next_base_obligation, forced_to: Option<base>,
                retouch_required: bool, out: bool, scored: bool }
```

A live play ends (§5.8) when the ball becomes dead or "time" is in effect.

### 5.2 Fair or foul [S: OBR definitions "Fair Ball", "Foul Ball"]

**Geometry.** Fair territory is the quarter-plane between the foul lines, **lines included** (the lines' outer edges are the boundary). A ball **in the air** is over fair territory if **any part** of it is: signed distance from the ball's center to either foul line, positive into fair territory, ≥ −R (ball radius). A ball **touching the ground** is judged by its **contact point** (under the center on flat ground), which must be on fair ground: signed distance ≥ 0 [2026 OBR "Fair Ball" clarification; changed 2026-10-31 after the Codex M4 review]. Foul poles are fair, but a ball that has already touched the ground and then hits a pole is not a home run. The decision uses the **ball's** position, never the fielder's.

**Decision rules** (first that applies):

| Case | Ruling |
| :--- | :--- |
| Ball goes out of the playing field in flight over fair territory, or touches a foul pole | **Fair: home run** |
| Ball first touches the ground, a fielder, the umpire, or a runner **beyond** first or third base (distance along the foul line from home > the base's distance) | Fair or foul by the ball's position **at that first touch** |
| Ball still in the infield region (not beyond 1B/3B) | **Pending** until one of: it **settles** (`BallSettled`) → judged where it rests; it is **touched** by a fielder, umpire, or runner → judged at the touch; it **passes** 1B or 3B (`BallPassedBase`) → judged by the side it passed on; it touches the base itself → fair |
| Batted ball touches the batter while the batter is still in the batter's box (body contact before any other touch) | Foul [C: OBR 6.03(a)(3) treats a batted ball striking the batter in the box as foul] |
| Ball touches a foul-territory object (backstop, stands, dugout) first | Foul. The backstop and the foul-territory walls stand at least 50 ft off the lines except where they close in on the foul poles (physics-spec §9.5), so a `Wall` or `OutOfPlay` touch there is judged by the same position test as any touch and comes out foul |

A pending ground ball that rolls foul then back fair before reaching a base is fair if it settles, is touched, or passes the base fair. This is the physical behavior the rule describes, and it falls out of judging only at those events.

**Foul ball handling:** if not caught (§5.3), ball dead; runners return to their time-of-pitch bases (§7.4). The count changes per §4.4.

### 5.3 Catches [S: OBR definition "Catch"]

- A **catch** occurs when a fielder secures the ball (`GloveSecured`) before it touches the ground or any object other than a fielder. Deflections off one or more fielders do not void a catch; touching the ground, a wall, the umpire, or a runner does.
- The fielder's **feet must be in the playing field** at the capture moment: the fielder's ground-contact point is inside the field boundary of the park config [C: approximates "in the field of play"]. A fielder reaching over the wall from inside the field may rob a home run.
- A catch of a fair or foul fly is an out (batter out, `FlyOut`/`LineOut`/`PopOut` by launch-angle class, data-formats §4.5).
- If the fielder loses the ball (`BallReleasedFromGlove{Drop|Knocked}`) before `t_secure`, there was no catch.
- **Runners on a caught fly:** each runner must **retouch** their time-of-pitch base after the ball is **first touched** by a fielder (not when secured) [S: OBR 5.09(b)(5) and comments]. A runner who left early is out on appeal (§5.6). Retouch requirement = a `BaseTouched` (or still touching) at or after first-touch time. This includes a runner who **touched home before the first touch**: his run is only pending (§6.4). If he goes back and retouches, the pending run is cancelled and counts only if he touches home again; the bases between are not re-required in reverse order, since missed-base appeals are deferred (§5.6). If he never goes back, the run stands unless he is put out on appeal. (Added 2026-11-12, day-fixes review #1.)

### 5.4 Forces

- A runner is **forced** when the batter becoming a runner (fair ball, walk, HBP, uncaught strike three) takes away their right to their base. Forces chain from first base outward through consecutively occupied bases.
- A force is **removed** for a runner when any following runner (closer to home in the order of the bases) is put out [S: OBR 5.09(b)(6)]. The batter-runner being out removes all forces. Any kind of out counts, a passing out included (§5.5).
- **Force out:** a fielder holding the ball touches the forced base (`BaseTouched` by the fielder with `holding_ball`, or the ball in hand touching the base) before the forced runner touches it. A tag on a forced runner who has not reached his forced base is a force out too, **including while he stands on a base short of it**: a forced runner has lost his right to his time-of-pitch base, alone on it or not [S: OBR 5.09(b)(6), definition "Force Play"]. (Added 2026-11-13; before, a tag on a forced runner standing on his own base did nothing.)
- On a caught fly, forces cease; runners returning to retouch are subject to "reverse force" appeal outs (§5.6) — touching the base before the runner returns retires them.

### 5.5 Tags and touching bases

- **Tag out:** `TagContact` with `holding_ball` while the runner is not touching a base (no open `BaseTouched` interval), and the fielder still holds the ball `t_secure` after the tag (no `Drop` before then) [S: OBR definition "Tag"].
- **Reaching a base:** a runner reaches a base when their capsule enters the base volume (`BaseTouched`). Bases must be touched **in order**; the runner model always paths through the base volume. A runner whose path skips a base is a model bug: v1 treats the skipped base as not reached and logs a `RulesWarning`. (Appeals for missed bases are deferred.)
- **Overrunning first base** [S: OBR 5.09(b)(4) exception]: a batter-runner who touches first and then overruns or overslides it is not liable to be tagged out while returning to it, unless he makes an attempt toward second. Judged from `RunnerPosition` samples, not a heading [C] (rewritten 2026-11-13; the earlier heading test, θ_attempt = 45°, is dropped: a runner rounding the bag toward second soon crosses the attempt line, and one who turns but stays near the line has made no attempt the umpire could see):
  - **Starts:** after his first touch of first and before any other base touch, when he **leaves first** (`BaseLeft`) in a play that carries position samples (any `RunnerPosition` so far in the play). From then until a sample shows an attempt (below) he is protected, so a tag that falls between his leaving the bag and his next sample is judged as it would be after that sample, not by the sampling phase [C] (added 2026-11-13, Codex rules-passing review #3: protection used to start only at an off-bag sample, so a tag before the next ~30 Hz tick retired a runner coming straight back). The cost is the other way round: a runner rounding the bag who is tagged before any sample shows him past the attempt line is not out; at running speed that window is a few tenths of a second. If samples begin only after he has left the bag, protection starts at a sample while he is off the bag and **past first base down the line** (right-field side, distance along the line from home beyond 90 ft).
  - **Attempt toward second:** a sample more than `overrun_attempt_m` = 1.5 m [E] into fair territory from the first-base line. The protection is then gone for good, even if he comes back.
  - **Ends** also when he touches first again; stepping off later, he is an ordinary runner.
  - **While protected:** a tag is no out; a fielder with the ball touching first is no force (he has reached it; the OBR 5.09(c)(3) appeal does not apply to a runner who returns at once); if the third out is made elsewhere he is left on first; for passing he is at first.
  - Not judged in v1: whether he returns **immediately** (the runner model always comes straight back; one who loiters stays protected until he touches first or crosses the attempt line). The 5.09(c)(3) appeal for not returning at once is deferred with the other appeals (§5.6). In a play with no samples, a batter-runner off first is an ordinary runner (a play with no samples gives no evidence either way, and protecting every batter-runner who leaves first would stop tag plays on runners rounding it). The live sim sends samples at 30 Hz from contact (since 2026-11-13, agent-models §6.2), so in games every play carries them; the unsampled case is left for scenarios and harnesses.
- **Two runners on one base** [S: OBR 5.06(a)(2), 5.06(b)(2); the citation was 5.06(a)(3) until 2026-11-13]: while two runners both touch a base, the preceding runner is entitled to it unless the following runner is forced there. The runner not entitled is out when tagged; a tag on the entitled one is no out. Kind: a preceding runner still forced onward is out **by force** at the base he is forced to (§5.4, so no run scores on it as a third out, §6.4); otherwise the out is a **tag** at the shared base. The liability lasts only while both are touching it: once the entitled runner leaves, a tag on the other one standing there is no out. If the third out is made elsewhere while both stand on it, both are left on base (§4.1).
- **Passing a runner** [S: OBR 5.09(b)(9) and its comment]: a following runner who passes a preceding runner before that runner is out is out. The pass can come from either runner's actions: a lead runner who retreats behind a following runner (the comment's rundown play, a lead runner running back past third toward left field) retires the following runner. It is proven two ways:
  - **Base touches** (always on): a following runner touching a base beyond the last base a preceding runner touched has passed him, and is out before the touch counts (a home touch scores nothing). A runner retreating to a base behind the last base a following runner touched has been passed by him; his own touch counts as usual. An exact time tie between the two runners' touches of one base relies on §1.2 ordering: the orchestrator must deliver the preceding runner's touch first. (Continuous-time detection makes such ties vanishingly rare; 2,560 full games on 2026-11-13 produced no passing outs.)
  - **Position samples:** a sample becomes path progress in bases (0 = home, 1 = first, ..., 4 = home again) as the nearest point on the two base lines that meet at the runner's last touched base. It is not clamped behind that base, so a runner beyond third toward left field is short of third. A runner touching a base is at that base, wherever his center is within it; a protected overrunner (above) is at first. A following runner whose progress exceeds a preceding runner's by more than `pass_margin_m` = 0.5 m [E] has passed him; level is not past. Runners compare with others sampled at the same t or standing on a base.
  - **Who can pass or be passed:** not a runner already out, and not a runner who has touched home (§6.4), **except** one whose home touch is pending while he owes a retouch after a catch (§5.3). The pending run and the runner's place on the base paths are separate: his run stands until he retouches (or is out), but while going back he is a preceding runner like any other, judged from his samples (path progress from home, his last touch) and from the bases he touches on the way, which move him back (`at`) without changing what he still owes [S: OBR 5.09(b)(9) and comment]. A following runner who crosses him on his way back is out at the crossing, before the retouch; if that is the third out, the pending run is a time play like any other (§6.4; the fourth-out appeal that could still take it away is not modeled). (Changed 2026-11-13, Codex rules-passing review #2: he used to be off the base paths until he retouched, so a crossing during the return was missed or judged late.)
  - **When:** a pass made while the batted ball is in doubt (fair/foul pending, not caught) is **held** with its time, and the passer leaves the play at once. When the ball is ruled fair or caught, every held pass stands and its out is recorded at its own time, in order, before anything the ruling itself brings (the batter's fly out at a catch, a home run's awards); a held pass can be the third out, and runs then count by its time (§6.4). An uncaught foul ball is dead and the held passes are dropped with the play: nothing the runners did while it was in doubt counts [S: OBR 5.06(c)(5)]. The pass is judged from what happened when it happened, so a crossing that has ended by the time of the ruling still counts. (Changed 2026-11-13, Codex rules-passing review #1: passes made before the ruling used to be discarded.)
  - **Several passes at once:** one touch or sample can establish more than one pass (a lead runner retreating behind two followers retires both). Every pass is judged, one at a time in runner order (batter-runner, then runners from first, second, third), re-reading the play after each out, until none remain, the touching or sampled runner is himself out, or the inning ends. A retreating lead runner's own touch is applied before the followers' outs, so he stands on the base he reached if one of them ends the inning. (Added 2026-11-13, Codex rules-passing review #4: only the first pass was recorded.)
  - **The out:** `RunnerPassed { runner, passed }` (§15), then `RunnerOut { kind: Tag }` at the base the passer was heading for. For run counting it is a time play, not a force (§6.4); it removes the forces of runners ahead of him (§5.4).
  - **Deferred:** passing during a home-run trot or any other award (the engine awards every runner at once; judging a runner passed on a home-run trot needs the trot modeled).
- **Out of the base path (M5):** a runner more than 3 ft (0.914 m) from their direct line to the base while avoiding a tag is out [S: OBR 5.09(b)(1)]. Uses `RunnerPosition` relative to the line from the runner's position when the tag attempt began to the base.

### 5.6 Appeals

Only one appeal is in v1: **leaving early on a caught fly.** It is resolved automatically, as if the defense always appeals: the runner is out if a fielder holding the ball touches the runner's original base, or tags the runner, before the runner legally retouches. All other appeals (missed bases, batting out of order) are deferred.

### 5.7 Interference by the ball and runners

- **Fair batted ball touches a runner** before passing an infielder (other than the pitcher) and before any fielder touches it: the runner is out, the ball is dead, the batter is credited with a hit and awarded first, and other runners advance only if forced [S: OBR 5.09(b)(7)].
- **Fair batted ball touches the umpire** before passing an infielder (other than the pitcher) and before any fielder touches it: ball dead, batter awarded first, forced runners advance [S: OBR 5.05(b)(4)].
- **"Passed an infielder"** [C]: the ball has crossed the line through that infielder perpendicular to the ball's horizontal path, or has been touched by a fielder. The infielder must be within 3 m of that crossing point to count; a ball that passes far from everyone has not "passed" anyone.
- Fielder–runner obstruction and runner–fielder interference are **not judged in v1**; collisions resolve physically. M5 adds basic obstruction (§13).

### 5.8 Ending the play ("time")

The play ends at the first of:
1. **Ball dead:** foul (uncaught), out of play, home run, HBP, batter hit by own batted ball, §5.7 cases, ball lodged (not modeled). "Out of play" is a `BallTouched{OutOfPlay}` fact: the ball left the playing field (physics-spec §9.5). The engine then emits `BallDead { reason }` (`HomeRun`, `GroundRuleDouble`, or `OutOfPlay` for a foul ball or a throw leaving the field) before the awards, and the play is over: no later fact changes it. The orchestrator stops the play at that fact, and the play's timeline carries a `BallDead` marker at that time and exit point (data-formats §5.4) so a client can cut away from a ball in the seats. (Added 2026-11-14, A10; before, foul ground had no walls and a ball out of play after a throw was ruled a ground-rule double.)
2. **Third out** made and no run is still being decided by §6.4 (a runner touching home in the same fact window still gets judged by time).
3. **Settled play:** a fielder holds the ball (secured) **inside the infield** (within 45 m of home [C]; changed from 30 m on 2026-10-31 because 30 m excluded second base, 38.8 m from home, where a fielder with no play returns the ball; the infield dirt reaches about 45 m), and every runner is either touching a base or has had no movement toward the next base for `t_settle` = 1.0 s [E]. A runner caught between bases when time is called returns to the last base legally touched.
4. **Play timeout:** sim time since contact (or since the pitch, for non-batted plays) exceeds `T_play_max` = 60 s [E]. Runners return to the last base touched; `PlayTimeout` is emitted. A timeout is a liveness bug; calibration W7 checks it never happens.

---

## 6. Outs and Runs

### 6.1 Batter outs

Strikeout (§4.4); caught fly (§5.3); force at first or tag before reaching first; batter touched by own fair ball outside the box (dead, out) [S: OBR 5.09(a)(7)]; infield fly (M5).

### 6.2 Runner outs

Force out (§5.4), including a tag on a forced runner standing on a base short of his forced base; tag out (§5.5), including the runner not entitled to a base two runners touch (§5.5); appeal on leaving early (§5.6); passing a runner (§5.5: a tag-kind out preceded by `RunnerPassed`); fair ball touching runner (§5.7); out of base path (M5); pickoff (a tag, M5). An overrunning batter-runner returning to first is exempt from tags (§5.5).

### 6.3 Multiple outs on one play

Outs are recorded in fact-time order. Once the third out is recorded, later outs on the same play are ignored for the count but **kept** as appeal-style records only if they change which runs count (§6.4). v1 does not model the "fourth-out" appeal.

### 6.4 When a run counts [S: OBR 5.08]

A run scores when a runner legally touches home before the third out, **except** no run scores if the third out is:
- the batter-runner put out before touching first base;
- any runner **forced** out (including the batter on a force at first);
- a preceding runner declared out for missing a base on appeal (deferred).

- a **preceding** runner put out on appeal for leaving early on a caught fly: no **following** runner (one who started behind him, including the batter) scores on the play [S: OBR 5.09(d)]. Runners ahead of the appealed runner are judged as a time play.

Otherwise it is a **time play**: the run counts if the home-plate `BaseTouched` precedes the third-out fact (a tag, or an appeal on a caught fly for the runners ahead of the appealed runner; by the OBR comment, an appeal out for leaving early on a fly is a time play). (Corrected 2026-11-12, runner-return review #2: this sentence used to make every caught-fly appeal a time play for all runners.) Fourth-out appeals (OBR 5.09(c)) are not modeled: a run by a runner who still owes a retouch when another out ends the inning stands.

---

## 7. Awards and Dead-Ball Placement

### 7.1 Forced awards

When a batter is awarded first (walk, IBB, HBP, catcher's interference, §5.7 cases), each runner advances one base **only if forced**, in order from third backward. On a walk the ball stays live; on HBP and interference it is dead.

### 7.2 Base award table

| Event | Award | Measured from |
| :--- | :--- | :--- |
| Fair ball out of park in flight | Home for batter and all runners | — |
| Fair ball bounces out of play / over the fence (**ground-rule double**), including a fair ball that leaves the field after touching a fielder or a wall | Two bases | Runners' positions at the time of the pitch [S: OBR 5.05(a)(6), 5.06(b)(4)(F)] |
| Fair fly deflected by a fielder out of the field in flight | Over fair territory: home run. Over foul territory: two bases | Time of the pitch [S: OBR 5.05(a)(8)–(9); numbering to verify, R11]. The sim sends no glove-deflection fact yet, so a deflected fly still pending fair/foul is judged by where it leaves; the foul-side row applies only once the ball has been ruled fair |
| Thrown ball out of play, first play by an infielder | Two bases | Time of the **pitch** [S: OBR 5.06(b)(4)(G)], unless every runner, the batter-runner included, had advanced at least one base when the throw was released: then the time of the throw [S: the rule's Approved Ruling] |
| Thrown ball out of play, any other throw | Two bases | Time of the **throw** [S: OBR 5.06(b)(4)(G)] (base last legally touched at release — v1 uses the base each runner had last touched then, `at`, not crediting a base he was past but had not reached) [C] |
| Pitch or pickoff throw by the pitcher from the rubber out of play | One base | Time of the pitch/throw [S] |
| Balk (M5) | One base, all runners | — |
| Catcher's interference (M5) | Batter first; others only if forced or stealing | — |

Awarded runners are placed with `RunnerAdvanced{cause: Award}`. Runs forced in by an award score immediately. On a walk-off (§4.1) the awarding stops at the winning run: runners behind it and the batter are given nothing more, as on any walk-off; a home run is the exception (all score). (Fixed 2026-11-14, Gemini play-area review P1-2: trailing runners were sent home without scoring.)

**Thrown ball out of play, in detail** (added 2026-11-14, A10):
- **A thrown ball** is one released by a `BallReleasedFromGlove{Throw}` and not secured by a fielder since (`GloveSecured`). A throw that bounces, caroms off a wall, or rolls before leaving the field is still the throw.
- **Infielder:** the thrower plays P, C, 1B, 2B, 3B or SS [C: OBR 5.06(b)(4)(G) says "infielder"; MLB practice counts the pitcher and catcher; verify, R11]. The pitcher's throw from the rubber (one base, §7.2) is a pickoff and comes with M5.
- **First play:** the play's first throw, made with no out yet recorded on the play [C: a fielder's force or tag out is a play, so a wild relay on a double play is from the time of the throw, as in the common interpretation]. On a caught fly the catch is the first out, so every throw after it is from the time of the throw.
- **A runner's base at the release:** the base he last touched (one who went back counts from the base he went back to), except that a runner who owes a retouch after a catch (§5.3) holds his time-of-pitch base until he has retouched it, and one who has legally touched home has scored [C, from OBR 5.09(b)(5): a base reached early on a caught fly is not legally held]. (Changed 2026-11-14, Gemini play-area review P1-3: the highest base ever touched and early touches used to count.)
- **Who has advanced:** at the release, every runner not out (the batter-runner included) holds a base past the one he held at the pitch, or has scored.
- **The award:** each runner gets two bases from his base (time of the pitch: the base he held then; time of the throw: his base at the release, above), and a runner who had legally scored before the release keeps his run either way (`RunnerAdvanced{cause: OnPlay}`; he scored on the play, not by the award). Runners are awarded from the lead runner back, and a following runner gets at most the base before the one awarded to the runner ahead of him unless that runner scores (two runners cannot hold one base; OBR 5.06(b)(4)(G) comment, where a runner held between first and second goes to third and the batter-runner who came round first behind him is held at second). Runs score in that order; a runner who owes a retouch after a catch is awarded like any other (the appeal on a dead ball is deferred, §5.6).
- The batter's plate appearance ends with the award (result `BattedBall`; the scorer charges the error later, §10).

### 7.3 Ball dead with no award

Foul ball, batter hit by pitch (beyond the award), HBP with batter swinging. Runners return to their time-of-pitch bases.

### 7.4 Return on foul

On an uncaught foul, every runner returns to the base occupied at the time of the pitch. A runner who stole during the pitch returns too.

---

## 8. Steals, Pickoffs, Balks (M5)

Designed fully in M5. Fixed now:
- A **stolen-base attempt** is a runner leaving their base toward the next one with the pitcher's commitment to the plate (or earlier) and no batted ball. The rules engine records attempts from `BaseLeft` timing; scoring is §10.4.
- **Balks** are judged only from discrete pitcher-motion facts (`PitcherOnRubber`, a pickoff feint flag, a stop in the set position). The pitcher agent never balks by accident in v1; balks exist only if the M5 pickoff model can produce illegal motions.
- **Pickoffs** are tags (§5.5).

---

## 9. Substitutions and Lineups

### 9.1 Lineup validity (pregame and after every substitution batch)

- Nine batting slots. With the DH active, one slot holds the **DH** and the pitcher does not bat (universal DH, D10).
- Nine distinct defensive positions filled; the pitcher is in the defense map, not in a batting slot.
- No player in the lineup is in `used` (removed players).
- A player who has been removed may not re-enter the game [S: OBR 5.10(d)].
- **Two-way players** (a starting pitcher who also bats as DH): not supported in v1 [C]. Open item.

### 9.2 Substitution types

All substitutions are applied at a decision point as one **batch** (data-formats `Substitution` command, "double switch as one batch"). Legality is checked on the **end state** of the batch, not after each piece.

| Type | Effect |
| :--- | :--- |
| Pinch hitter | Replaces the player in the slot due up; may enter mid-PA (count carries over, §9.5) |
| Pinch runner | Replaces a runner on base; takes the replaced player's slot and base |
| Defensive replacement | New player takes a slot and a position |
| Position switch | Players already in the game swap positions (no one leaves) |
| Pitching change | New pitcher replaces the current pitcher on the defense map (§9.3) |
| Double switch | Any combination of the above in one batch, typically a pitching change plus a slot swap |

A pinch hitter or pinch runner who is not given a defensive position before the next defensive half-inning is replaced at that position by a required substitution; until the manager provides one, the command validator rejects `advance` with `LineupIncomplete` [C]. The AI manager always fills it.

### 9.3 Pitching changes and the three-batter minimum (D17)

**Starting pitcher** [S: OBR 5.10(f)]: must pitch to the first batter until that batter is put out or reaches base (injury exception later).

**Relievers (D17)** [S: OBR 5.10(g)]: a relief pitcher must pitch until **three consecutive batters** have completed their plate appearances (out or reached base, counting pinch hitters), **or** the half-inning ends, before being removed. Removal before that is rejected with `ThreeBatterMinimum`.

- **Batters faced** counts PAs that end while the pitcher is in the game. A PA ended by the inning ending on a caught stealing or pickoff does not count, but the half-inning ending satisfies the obligation anyway.
- **Inning boundary:** a reliever whose obligation is unmet when the half-inning ends has satisfied it; they may be removed between innings. If they **begin** the next half-inning (throws its first pitch, or is on the mound at the start), a fresh obligation starts: three batters or the end of that half-inning [C: follows MLB's published interpretation that a pitcher who takes the mound for an inning must complete it or face three; R11 confirms].
- **Injury or illness exception:** the field exists in the command (`reason`) but no reason is legal in v1 (injuries are after M8, D18).
- **D17 scope:** as decided, the three-batter rule applies to relievers. MLB also applies it to starters; the owner can extend it with a config flag `three_batter_min_applies_to_starters` [C], default false.

**Mound visits** [S: OBR 5.10(l)]: a manager or coach's **second trip** to the same pitcher in the same inning requires that pitcher's removal; the rules engine rejects a second visit while the pitcher's three-batter obligation is unmet, with `ThreeBatterMinimum`. Visits per team per nine innings: configurable `mound_visits_per_9` = 5 [C: recent MLB value, R11 confirms], plus one per extra inning. Exceeding the limit is rejected (`IllegalNow`).

**Warm-up:** the rules require nothing. A cold reliever may enter; the consequence is physical (agent-models §2.5, PLAN §10.1).

### 9.4 The DH [S: OBR 5.11]

- The DH may be replaced by a pinch hitter or pinch runner, who becomes the DH.
- The team **loses the DH** for the rest of the game if: the DH moves to a defensive position; the pitcher moves to another defensive position; a pinch hitter or runner for any player then becomes the pitcher; or the pitcher bats for anyone.
- **v1 rule [C]:** any substitution batch that would cause DH loss is **rejected** with `DhRule`. Neither manager can lose the DH. This removes the hardest substitution edge cases with no effect on normal play. Allowing it (with the pitcher entering the batting order in the DH's slot) is an open item.

### 9.5 Mid-PA substitutions and charging [S: OBR 9.15(b), 9.16(h)]

- **Pinch hitter with two strikes:** if the PA ends in a strikeout, it is charged to the **original** batter. Any other result goes to the pinch hitter. Stored as `pa.charged_batter`.
- **Relief mid-PA:** if the count was 2-0, 2-1, 3-0, 3-1, or 3-2 when the pitcher was replaced and the batter walks, the walk is charged to the **previous** pitcher. Any other result goes to the reliever. Stored as `pa.charged_pitcher`.

### 9.6 Command validation

Each command is validated against the current `RulesState` at a decision point. Results map to data-formats `RejectReason`:

| Check | Reason |
| :--- | :--- |
| `expected_play_index` mismatch | `Stale` |
| Requires a dead ball / wrong phase (substitution during a live play, pinch runner for an empty base) | `IllegalNow` |
| Reliever obligation unmet | `ThreeBatterMinimum` |
| Player in `used`, or not on the roster | `PlayerUnavailable` |
| Would lose the DH | `DhRule` |
| Wrong team | `NotYourTeam` |
| Batch leaves an invalid lineup | `LineupIncomplete` (new, §15) |

The rules engine also publishes the **legal-command set** at each decision point, which feeds `ViewState` (data-formats §7.2) so the HUD greys out illegal options and the AI manager plans around D17.

---

## 10. Official Scorer

The scorer is a **pure function of the event log** (PLAN §4.3), versioned by `scorer_version`, living in `bb_stats` with its decision helpers in `bb_rules`. Every ruling writes `ScorerRuling` plus `ScorerEvidence` (data-formats §4.3) so a later scorer version can re-rule from the log.

### 10.1 The counterfactual test

PLAN §9 asks: "would an ordinary clean play have retired the runner?" It is answered with the **reference fielder**: the actual fielder, at the actual intercept, with the actual glove pose target, but **noise-free execution** and grade-50 `Transfer`, `ArmStrength`, `ArmAccuracy` [C]. `bb_game` computes the reference play's timing with the same physics functions at the end of each play and records it as `ScorerEvidence` (the scorer itself never runs physics).

Key margins (all [E], configurable):
- `m_err` = 0.15 s: a misplay is an error only if the clean play would have retired the runner by at least this margin.
- `m_fc` = 0.15 s: a fielder's choice is scored only if a throw to first would have retired the batter by at least this margin.
- **Ordinary effort** [S: OBR definition]: the chance is ordinary if the fielder was set (arrived ≥ 0.2 s [E] before the ball), not diving, and the ball was within normal reach (`reach_frac` ≤ 0.7 [E], agent-models §5.4), and not a short hop with ball speed > 35 m/s [E].

### 10.2 Hit, error, or fielder's choice [S: OBR 9.05, 9.06, 9.12]

For each batter-runner reaching base on a fair ball:
1. If a fielder **misplayed an ordinary chance** (capture failure or errant throw on an ordinary chance) and the reference play retires the batter by ≥ `m_err`: **reached on error** (fielding or throwing, by which act failed).
2. Else if the defense **chose** to play on another runner and the reference throw to first retires the batter by ≥ `m_fc`: **fielder's choice** (whether or not the other runner was retired).
3. Else: **hit**. The hit's value is the last base the batter reached **without** the help of an error or a throw to another base (advance on the throw) [S: OBR 9.06].
- A batter who would have reached anyway is credited with a hit even if a fielder misplayed; the misplay may still be an error if it let a runner advance extra (§10.3).
- A **dropped foul fly** that was an ordinary catch is an error that prolongs the PA.
- **Infield hit:** the ball was fielded cleanly but the reference throw was late.
- **Ground-rule double**, **HR** by award. A walk-off hit is limited per §4.1.

### 10.3 Errors on runners and extra bases

A misplay that is ordinary and lets any runner (including the batter-runner after reaching) advance one or more bases that the reference play would have prevented, by ≥ `m_err`, is an error. A wild throw that does not change any outcome is not an error [S: OBR 9.12(d) on mental mistakes and no-effect misplays].

### 10.4 Pitching and baserunning plays [S: OBR 9.07, 9.13]

- **Wild pitch vs passed ball:** scored only when a runner advances (or the batter reaches on an uncaught strike three) because the catcher did not hold the pitch. **Wild pitch** if the pitch bounced before the catcher, or its crossing point was outside the catcher's ordinary reach (`reach_frac` > 0.7 at the receiving target). Otherwise **passed ball**.
- **Stolen base** if a runner advances on an attempt with no hit, error, WP, or PB; **caught stealing** if retired on the attempt. **Defensive indifference** (no SB credited) if the defense made no throw or move and the runner's team was not in a scoring-relevant spot: 9th inning or later, defense leading by ≥ 3 [C].
- **Sacrifice fly:** fewer than two outs, a fly or line drive caught (or dropped on error) in the outfield or foul territory, and a runner scores. **Sacrifice bunt** (M5).
- **GIDP:** ground ball, two outs recorded on force or batter outs on the play.

### 10.5 RBI [S: OBR 9.04]

Credited for each run that scores because of the batter's PA: a hit, a sacrifice, an out other than a double play, or a bases-loaded walk, HBP, or interference. **No RBI** on a double play, or for a run that scored only because of an error (§10.3: the reference play would have prevented it).

### 10.6 Earned runs and inherited runners [S: OBR 9.16]

- Reconstruct each half-inning **without errors and passed balls**: count the outs the defense would have had. A run is **unearned** if it scores after the reconstructed third out, or if the runner reached on an error, a catcher's interference, or would not have reached or scored without one.
- The reconstruction places the "would-be" outs using `ScorerEvidence` from §10.2–10.3; the standard OBR heuristics decide runner advancement in the reconstruction (no physics re-simulation).
- **Inherited runners:** each runner on base keeps `responsible_pitcher`, the pitcher who allowed them to reach. Runs are charged to that pitcher. On a fielder's choice that retires a runner and puts the batter on base, the new runner inherits the retired runner's responsible pitcher [S: OBR 9.16(g)].
- A relief pitcher's earned runs use their own reconstruction: errors before they entered do not count in their favor for the outs they would have had [S: OBR 9.16(i)].

### 10.7 Win, loss, save, hold [S: OBR 9.17, 9.19]

Standard rules: the **winning pitcher** is the pitcher of record when their team took the lead for good, except a starter must pitch at least 5 innings (4 in a game of 5 innings) and otherwise the scorer chooses the most effective reliever [v1: the reliever with the most outs recorded among those after the starter, ties to the earliest]. **Loss** to the pitcher charged with the go-ahead run that the team never recovers. **Save** per OBR 9.19 (finishes a win, not the winner, at least ⅓ inning, and one of: lead ≤ 3 runs and ≥ 1 inning; tying run on base, at bat, or on deck; ≥ 3 innings). **Hold** (unofficial): a save-situation reliever who leaves with the lead intact and records at least one out.

---

## 11. Accounting Identities

Checked by `bb_stats` tests on every generated game (calibration-targets: 1,000-game M4 gate). A failure means a rules or scorer bug, never a calibration issue.

| ID | Identity |
| :--- | :--- |
| I1 | Box-score proof: AB + BB + HBP + SH + SF + CI = R + LOB + opponent putouts (per team) |
| I2 | PA = AB + BB + HBP + SF + SH + CI (per batter and team; IBB counted in BB) |
| I3 | Team putouts = 3 × innings played on defense, minus outs not made in a walk-off bottom half |
| I4 | Σ pitchers' IP × 3 = team putouts |
| I5 | Σ batters faced by pitchers = opponent PA |
| I6 | Runs = count of `RunScored` = Σ inning line score |
| I7 | H = 1B + 2B + 3B + HR |
| I8 | Pitches = balls + strikes + in-play pitches (strikes include fouls and foul tips) |
| I9 | Earned runs ≤ runs per pitcher; team ER ≤ team R |
| I10 | Every runner who reached base ends the half-inning scored, out, or left on base, exactly once |
| I11 | Assists and putouts: every out has exactly one putout; total chances = PO + A + E |
| I12 | Wins = losses across the league; at most one W, one L, one S per game |

---

## 12. Scenario Corpus

### 12.1 Purpose

The rules engine is tested **without physics**: each scenario is a scripted sequence of RulesFacts and commands with the expected outcome. Scenarios are the specification in executable form; every rule in §4–§10 has at least one, and every bug found later adds one.

### 12.2 Location and format

`sim/crates/bb_rules/scenarios/<category>/<id>.toml` [C: TOML, matching the other config files; one scenario per file]. A test harness loads every file, runs `step` for each input, and checks expectations.

```toml
id = "force-002"
title = "Runner on 1st, grounder to short, throw to 2nd beats the runner: force out, batter safe at 1st"
rules = ["OBR 5.09(b)(6)"]          # rule references, for humans
milestone = "M4"
tags = ["force", "fielders-choice"]

[initial]                            # anything omitted takes the fixture default
fixture = "default-9v9"              # lineups, DH on, home batting second
inning = 3
half = "top"
outs = 0
count = [1, 1]
bases = { first = "A.6" }            # team.slot of the runner
score = [0, 0]

[[step]]
fact = "PitchDelivered"
t = 0.0
[[step]]
fact = "BatContact"
t = 0.42
[[step]]
fact = "BallTouched"
t = 0.71
what = "Ground"
pos = [-6.0, 25.0]                   # x, y in m; fair, in the infield
[[step]]
fact = "GloveSecured"
t = 1.30
fielder = "H.SS"                     # team.position
[[step]]
fact = "BallReleasedFromGlove"
t = 1.95
fielder = "H.SS"
kind = "Throw"
[[step]]
fact = "GloveSecured"
t = 2.30
fielder = "H.2B"
[[step]]
fact = "BaseTouched"
t = 2.32
agent = "H.2B"
base = 2
holding_ball = true
[[step]]
fact = "BaseTouched"
t = 2.55
agent = "A.6"
base = 2
[[step]]
fact = "BaseTouched"
t = 4.21
agent = "A.7"                        # batter-runner
base = 1
expect_events = [                    # subsequence match, in order, fields optional
  { event = "RunnerOut", runner = "A.6", base = 2, kind = "Force" },
]
[[step]]
fact = "PlayEndCheck"                # harness pseudo-fact: advance time to trigger §5.8 settle
t = 6.0

[expect]
outs = 1
bases = { first = "A.7" }
pa_result = "ForceOut"
play = "Dead"
scorer = { batter = "FieldersChoice" }   # optional; requires ScorerEvidence steps
```

Rules of the format:
- `expect_events` on a step checks the events emitted **by that step**, as an ordered subsequence. `expect_no_events` lists events that step must not emit (any field match fails). `[expect]` checks the final state.
- Commands are steps too: `command = "PitchingChange"`, with fields, and `expect_reject = "ThreeBatterMinimum"` or `expect_accept = true`.
- `ScorerEvidence` can be supplied as a step (`fact = "ScorerEvidence"`, reference-play margins) for scorer scenarios.
- Agent names are `team.slot` (`A.6`) or `team.position` (`H.SS`), resolved through the fixture.
- Unknown keys are errors, so typos fail loudly.

### 12.3 Coverage targets (M4)

At least **150 scenarios** by the M4 gate, at least one per row below; M5 and M6 add their own.

| Category | Examples | Min |
| :--- | :--- | :--- |
| `count` | every count transition; foul with two strikes; foul tip live; uncaught third strike with 1B occupied, 0 and 2 outs | 20 |
| `pitch` | called strike edge (ball touching zone corner, circle-rectangle); bounced pitch taken; HBP in zone; HBP while swinging; check swing at, just under θ_swing | 12 |
| `fairfoul` | each row of §5.2; roll foul then fair; hit the bag; pole; ball on the line; fielder in foul territory touching a fair-side ball | 15 |
| `catch` | catch after deflection; drop before t_secure; wall-robbed HR; foul fly caught; foot out of field | 10 |
| `tagup` | leaves early → appeal out; leaves on first touch (legal) before secure; time play on sac fly with third out elsewhere | 10 |
| `force` | force chains; force removed after following runner out; reverse force | 12 |
| `tag` | tag before base; base before tag; tag then drop | 12 |
| `overrun` | overrun tagged on the way straight back (no out); attempt toward second; protection spent once back on first; oversliding second (no exemption); ball to first while overrunning; third out elsewhere (left on first); drift short of the attempt line; overrunner a step fair of a runner standing on first | 8 |
| `twoonbase` | preceding runner entitled; following runner forced there; force at the next base; force removed first; caught fly ends the forces; liability ends when one leaves; batter-runner forced onto first | 7 |
| `passing` | by base touch; by position (level is not past); time play counts / not counts; lead runner's actions (OBR comment play, retreat to second, retreat after a home touch before a catch); passer's home touch scores nothing; out or scored runner cannot be passed; force removed by the passing out; both on one base; fly in doubt lands foul | 12 |
| `runs` | time play counts / not counts; third out a force; batter out before first | 10 |
| `awards` | walk with bases loaded; ground-rule double from time of pitch; throw out of play (infielder first play vs other; first play when all have advanced; after an out; runner held behind a lead runner; after a caught fly); foul ball out of play; fair deflected fly out over foul ground | 12 |
| `interference` | fair ball hits runner; hits umpire before/after passing an infielder | 6 |
| `gameflow` | home leads after top 9; walk-off single with runner from 2nd (batter credited with 1B); walk-off HR all score; extra innings | 10 |
| `subs` | pinch hitter with two strikes → K charged; relief mid-count walk charged; three-batter minimum accept/reject incl. inning boundary; second mound visit; DH-loss rejection; double switch batch; re-entry rejected | 20 |
| `scorer` | error vs hit by margin; FC vs hit; WP vs PB; defensive indifference; earned-run reconstruction; inherited runner after FC | 15 |

### 12.4 Property tests

In addition to scenarios, a fuzzer generates random **well-formed** fact sequences (ordered times, physically consistent: a runner cannot touch base 3 before base 2) and checks the §3 invariants and identities I6, I10, I11 after every play. Any failing sequence is minimized and saved as a new scenario.

### 12.5 Integration checks (M4)

With physics and agents connected, `sim-cli game` runs assert after every play that the rules engine reached a decision point (no stuck live play) and that every `RulesWarning` is zero. 1,000 games with zero warnings and zero identity failures is the M4 accounting gate.

---

## 13. Scope by Milestone

| Milestone | Rules |
| :--- | :--- |
| **M4** | §4 (all, IBB command included); §5 except out-of-base-path; §6; §7 except balk and catcher's interference; §9.1 lineup validity; §10 hits/errors/FC, WP/PB, RBI, earned runs, W/L/S; §11; §12 corpus ≥ 150 |
| **M5** | Infield fly [S: OBR definition; declared when a fair fly, not a line drive or bunt, can be caught with ordinary effort by an infielder with runners on 1st and 2nd or bases loaded, fewer than two outs; judged from the batted-ball atlas at the moment of apex]; steals, pickoffs, balks (§8); out of base path; bunts (foul bunt with two strikes, sacrifice scoring); catcher's interference (bat contacts catcher's glove capsule); basic obstruction (fielder without the ball contacting a runner's path: runner awarded the base they would have reached, v1 = next base) |
| **M6** | §9.2–9.6 substitutions, D17, mound visits, mid-PA charging, legal-command set in `ViewState` |
| **Deferred** | Other appeals (missed bases, batting out of order), manager challenges, pitch clock (pacing is presentation, PLAN §9), runner's lane interference, spectator interference, ejections, protests, suspended games, two-way players, DH-loss substitutions, human-umpire mode |

---

## 14. Validation Summary

- Scenario corpus (§12): every scenario passes; coverage targets met at each milestone.
- Property fuzzing (§12.4): no invariant failures in 10⁶ random plays.
- Accounting (§11): zero failures over 1,000 full games (M4) and over every calibration season (calibration-targets §9).
- Determinism: `bb_rules` is RNG-free; a test asserts it does not depend on the RNG module (crate dependency check).
- Performance: rules, scoring, and event log within 10 ms per game (agent-models §12).

---

## 15. Open Items

1. **R11 (new):** verify every OBR rule number and wording cited here against the current Official Baseball Rules, and the MLB interpretations for: the three-batter minimum at inning boundaries (§9.3), mound-visit limits, and the walk-off base credit. **Before M4.**
2. **Data-formats additions** (append-only, data-formats §8): events `BattedBallRuling` (fair/foul and catch decisions with the deciding fact), `PlayTimeout`, `RulesWarning`, `LivenessWarning`, `UmpireSignal`; reject reason `LineupIncomplete`; `RunnerAdvanced.cause = Award`; a `reason` field on `PitchingChange` for the future injury exception.
3. **DH loss** (§9.4) is rejected in v1. Allowing it needs the pitcher-enters-the-order logic and AI-manager awareness.
4. **Two-way players** (a pitcher who is also the DH) are unsupported.
5. The reference-fielder definition (§10.1) and the ordinary-effort thresholds are first drafts; M5 reviews scorer rulings against watched plays.
6. "Passed an infielder" (§5.7) and "attempt toward second" (§5.5) are geometric stand-ins for umpire judgment; tune in M5 if they produce calls that look wrong on replay.
7. **Passing and overrun (2026-11-13).** `RunnerPassed { runner, passed }` is a new rules event; it needs a data-formats §8 append-only addition before the event log records it (today `bb_game` logs only the `RunnerOut`, kind `Tag`). An `OutKind::PassedRunner` would let the scorer and box score name the out; until then it is a tag. Simulator side done 2026-11-13: `bb_game::live` sends `RunnerPosition` samples at 30 Hz during live plays (§2), and a batter-runner who stops at first runs through it and walks back (agent-models §6.2). In 512 games, 8,421 batter-runners overran first and came back, protected; there were no passing outs and no rules warnings (`examples/overrun_probe.rs`).
8. Infield-fly declaration timing (M5): real umpires call it near the apex; the rule uses the atlas prediction then. Check that agents' behavior (runners holding) matches.
