# A4: throw-choice evidence (2026-11-14)

## Measurement contract

This is the measuring half of owner backlog A4, from `a69dd50`. No option,
weight, timing estimate, random stream, or game fact is changed. The reference
is agent-models §5.5–5.7 and the owner's two-out example: take an available
force at first or second rather than a less secure tag at home.

The diagnostic will opt into a thread-local, read-only decision trace. It copies
the holder's existing calculations at the point he commits to a throw or carry,
including nearby carries that bypass the weighted throw loop. Consecutive carry
ticks toward the same base in the same possession are one commitment, with a
first/last time and tick count. Unselected alternatives are estimates, never
counterfactual realized outcomes. Score, inning, visible runners and eventual
facts are diagnostic context only; none flows back into a decision.

`throw_choice_probe` will run several hundred deterministic games, emit optional
JSONL for each commitment, classify conservative choice flags from the estimates,
and identify seed/play/time for watching. Rates count commitments (classes can
overlap); failed execution alone is not a wrong choice. Every measured seed is
also run with tracing disabled and its event-log digest compared. A pre-hook
digest sample guards against a change shared by the enabled and disabled paths.

## Reproduction and interpretation

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cd sim
# In the restricted worktree sandbox the linker's usual home cache is read-only.
export ZIG_GLOBAL_CACHE_DIR="$PWD/target/zig-global-cache"
export ZIG_LOCAL_CACHE_DIR="$PWD/target/zig-local-cache"
# Leave CARGO_BUILD_JOBS=2 in effect.
cargo run --release -p bb_game --example throw_choice_probe -- 256 0 /tmp/a4-choices.jsonl
cargo test -p bb_game --release --test determinism --example throw_choice_probe
cargo test --workspace --release
```

Games use the existing average abilities, generated handed rosters and calibrated
M3 environment, with no substitutions. Play indices are the game's pitch/play
indices, not the ordinal fair ball. Times are seconds from pitch release. Scores
are `[away, home]`; `batting=0` means top, `1` bottom. A watch reference is therefore
seed + play + time; no client was built or launched for this investigation.

The sink is `live::choice_trace::capture`, off by default and isolated per thread.
It records actual releases, the first nearby-carry commitment, and short carries
substituted for selected throws. Repeated carries toward the same bag in the same
possession retain their first options, with `until_s` and `ticks` recording the
duration. A carry can be abandoned before arrival. A zero-distance nearby carry
is effectively waiting at the bag, often correctly waiting to tag at home.
The probe asserts that every recorded release has exactly one choice record.
It does not count waiting for setup as another throw or count each movement tick
as a new tactical mistake.

Each JSONL row contains outs, inning, batting side, score, holder and position,
runner path positions and speeds, the tested options in iteration order, the
chosen destination and runner, direct/relay/receiver/setup times, probability,
weight and value where the chooser actually calculated them. Nearby carries
have no probability or value in the real chooser. For these alone, additional
`unweighed_throw` rows show the weighted loop's estimates at that instant using
the same observer inputs. **These alternatives were bypassed, not weighed.**
Their formulas deliberately live only in the diagnostic module; if the chooser
changes later, this shadow must be updated or retired. Missing/infinite numeric
estimates serialize as JSON `null`; `kind` distinguishes a branch that did not
calculate a quantity from an unavailable direct/relay route.

Outcomes include the next possession, subsequent bag touches and outs with
times, and the play's run scorers. `selected_runner_out_later` means that runner
was retired later in this play, possibly after another decision; it is **not**
causal credit for this particular commitment. Play run scorers likewise include
the whole play. No alternate option is simulated to manufacture its outcome.

The independent tactical screen calls an estimated margin of at least **0.25 s**
"sure" (the existing §5.5 setup safety margin); under **0.15 s** is close (the
umpire's close-play boundary). These are diagnostic judgment thresholds, not
new simulation parameters. Bad execution does not by itself fail the screen.
Flags are deliberately broader than confirmed tactical mistakes: a double play
can justify taking a trailing force, a close throw can be the only useful try,
and a temporary wrong-way carry can be corrected. The table reports screening
counts and the case assessment separately, rather than claiming every flag is
a lost out or run. Classes overlap.

The additional `audit_intended_goal` is copied from runner internals **only for
retrospective diagnosis**, never offered to the holder or used to pick an option.
It identifies the one-tick stale-speed case described below; it is not a proposal
to give defenders knowledge of a runner's future decision.

## Results

**256 games, seeds 0–255:** 18,318 balls in play and **14,238 commitments**:
5,523 throws attempting an out, 7,020 no-play returns to second, 1,652 nearby
carry/hold episodes, and 43 carries substituted for short throws. There were
404 commitments with at least one flag. These are counts per **whole game**,
not per team-game. No gameplay number moved: the traced and untraced runs both
produced **2,880 runs, 56,455 pitches and 13,739 outs**.

| Class / screen definition | Count | Per game | Judgment; cause and proposed priority |
|---|---:|---:|---|
| Carry behind a sure lead out: fewer than two outs, carry margin ≥0.25 s, another destination also ≥0.25 s | **146** | **0.5703** | All from the nearby-carry shortcut. 127 have a sure lead **force**, 27 a sure force home. Compare the options before committing (**1**). Not all 146 are bad tactics: 81 plays produced two outs. |
| Ahead of a stopped runner: out-attempt throw targets a base beyond the runner's internal stopping goal while he is already exactly on that goal | **124** | **0.4844** | Clear target-generation mismatch, from stale arrival speed and `visible_target` (**2**). 86 to home, 38 to third; **none** of the targeted runners is retired later in that play. These occur in 94 games. |
| Cutoff carrying that same nonexistent advance (subset of the preceding row) | **28** | **0.1094** | 28 of 42 relay **out attempts** target a runner already stopped short of that destination. Gate the destination before optimizing the route (**2**, then **5**); this is not 28 additional independent mistakes. |
| Nonforce carry rather than sure force: carry starts outside 1.2 m tag reach, selected runner unforced, another force has ≥0.25 s | **17** | **0.0664** | Missing force guard / tag-completion plan (**3**). 13 are one-tick episodes; three of the four sustained carries get a tag out. The fourth recovers by throwing first (66/59). Do not label the successful tags wrong merely because a force existed. |
| Risky lead rather than sure trailing out: chosen margin <0.15 s, trailing alternative ≥0.25 s | **1** | **0.0039** | Initial carry miscomparison (**1**); 21/111 recovers the out at first. No lost out established. |
| Early/leading home gamble: fewer than two outs, unforced home margin <0.15 s against a sure force, inning ≤6 or defensive lead ≥2 | **0** | **0.0000** | No measured instance. Fixed home weight / missing situation value remain a model limitation (**4**); no tuning justified from this sample. |
| Late close-score home option bypassed: inning ≥8, defensive lead 0–1, fewer than two outs, home alternative ≥0.25 s | **3** | **0.0117** | All nearby carries; **all three allow no run and finish with two outs**. No confirmed late-score mistake. Continuation value must accompany score awareness (**1**, **4**). |
| Two-out home tag over available force at first/second with margin ≥0.15 s | **0** | **0.0000** | Existing force preference works in this sample (**5**). Do not retune `TWO_OUT_TAG_W` on this evidence. |
| Truly hopeless by the holder's own timing: selected throw margin ≤−0.25 s | **0** | **0.0000** | The `p_out > 0.3` admission rule prevents this by construction. It cannot prevent the nonexistent-target cases above. No threshold change justified (**4**). |
| Speculative out-attempt throw: selected margin <0 | **120** | **0.4688** | Timing says more likely safe; 34 targeted runners nevertheless get out later. Margins −0.063 to −0.001 s, not hopeless. Missing error/advance/hold value prevents calling all 120 wrong (**4**). |
| Slower available cutoff route: chosen route >0.05 s slower than the other route's own estimate | **0** | **0.0000** | Structural consistency check of the minimum-time comparison, not a tactical endorsement of cutoffs (**5**). |

The force-home subset of the first row has **10 plays with a run scoring**
(0.0391/game). That is an exposure count, **not 10 proved preventable runs**:
the alternative throw was not executed, and a two-out trade can be rational.
The clearest tied-game case is 13/140 below. There are 90 one-out and 56
zero-out cases among the 146; only two produce no out at all.

The owner's situation is not a common wrong choice under today's weighted
rule: there are **119 two-out commitments** offering a home tag and a force
at first/second with ≥0.15 s in hand; 103 choose first and 16 second. The
broader cohort has 519 commitments with two outs and a live runner who began
on third (including no-play returns). There are also **37** choices taking
a ≥0.25 s out elsewhere when the home estimate is already late: conceding
the run for an out is often appropriate, not a class to "fix" indiscriminately.
The continuation of 7/129 below is an example.

**Validation:** all 256 traced/untraced **log bytes and digests** match. The
eight saved pre-hook digests (seeds 0–7) also match after instrumentation.
Seed 0's digest is
`d3e0ba5a999fa3d1e0d99c0e1a0315634ca7404798c846d58c9961823c647a1e`.
`cd sim && cargo test --workspace --release`: **196 passed, 0 failed, 0 ignored**
(42 suite results, including doc-test suites). The explicit focused command
above also passes all four determinism tests and the two example tests. Those
example expectations come from the owner's two-out case and the distinction
between an out attempt and a control return, not snapshots of implementation.
The full 14,238-row JSONL used here has SHA-256
`7212b63bcf94affcae9a4a81e1fee765ab8d979c1954c9ba3f38109e2511d8f8`;
it is reproducible with the command above and is not checked in as a large log.

### Plays to watch

| Seed / play / time | Situation and choice | Evidence and assessment |
|---|---|---|
| **13 / 140 / 1.667 s** | Top 6, tied 4–4, one out, loaded. 1B carries to first. | Skipped force-home margin **+0.694 s**, **+0.571 s even fully set**. First is touched at 4.000 s; the go-ahead run scores and there is only one out on the play. This is the clearest close-score example of the shortcut choosing behind the lead runner. |
| **1 / 77 / 1.756 s** | Bottom 3, away ahead 3–0, one out, loaded. 1B carries to first. | Carry margin +1.836 s; skipped forces at second +0.553, third +0.398, home **+0.967** (+0.818 fully set). First is touched at 3.689 s and the runner scores at 4.089 s. Conceding a run with a lead can be sound, but here the evaluator never compares an apparently routine home force. |
| **7 / 129 / 1.754 s** | Bottom 5, home ahead 4–2, no outs, loaded. 2B carries to second. | Home force estimate +1.116 s. The actual second/first double play succeeds and a run scores. A useful **counterexample to blanket adjudication**: two outs for a run may be competent with none out in the fifth. |
| **21 / 111 / 2.183 s** | Bottom 4, two out, score 2–3. SS initially carries toward second. | Carry margin only +0.126 s versus a throw-first margin +1.100 s. After three carry ticks he abandons it and throws first at 2.800 s; third out at 3.733 s. Wrong initial margin preference, recovered without a lost out. |
| **17 / 203 / 1.932 s** | Bottom 8, zero outs, score 8–6. 3B briefly heads to third for an unforced runner. | Nonforce carry from 2.56 m away; first force estimate +1.316 s. The runner is already back at second and his arrival speed has not yet reset. One carry tick, then throw-first at 2.349 s and an out at 3.582 s. This exposes both the missing force guard and stale movement cue; it does not lose the sure out. |
| **1 / 64 / 3.824 s** | Top 3, two out, away ahead 3–0. After taking first, 1B throws home. | Runner 5 has stopped at third (`s=82.296 m`, intended goal 3), but residual speed 8.053 m/s makes `visible_target` say home. The fictitious home-out margin is **+2.479 s**, `p_out=1`. The catcher takes it at 4.674 s; that runner never tries home and no further out occurs. A clear ahead-of-runner target-generation error. |
| **54 / 182 / 1.800 s** | Bottom 8, tied 2–2, one out, loaded. SS carries to second while a home force is estimated +0.961 s. | The probe's late-score screen flags it, but the actual second/first double play ends the inning without a run. **Do not count it as a proven score-awareness error.** The other late-score flags, 70/197 and 132/256, also recover and allow no run. |
| **0 / 20 / 3.981 s** | Top 2, one out, score 1–0. 2B throws first. | First estimate −0.019 s, second −0.187 s; no out results. A low-percentage attempt rather than a truly hopeless throw. Holding might or might not be better once error/advance risk is modelled. |
| **3 / 19 / 3.160 s** | Top 1, two out, runner from third, tied 0–0. Throw first. | First-force estimate +0.523 s and home-tag +0.452 s. **Correct choice by the owner's criterion**, even though execution produces no out and a run scores. This is why failed execution must not be counted as a wrong destination. |


## Why these choices arise; proposed changes in priority order

1. **Compare nearby carries with the other outs, including a continuation.**
   `holder_act`'s first carry loop accepts the first eligible runner with a bag
   within 8 m and a run-to-bag margin above 0.1 s, then returns before
   `out_weight`, `p_out`, score or any other destination can be consulted. The
   vector starts with the batter. That explains first basemen taking first
   automatically, even with other forces available. With fewer than two outs,
   keeping the lead force alive matters: retiring the batter removes it. The
   later weighted loop cannot correct a choice it never sees. Proposed fix:
   offer carry and throw in the same comparison, with a modest two-out sequence
   estimate where a double play is available. Do not simply ban taking first:
   there are successful carry-second/throw-first double plays in these samples.

2. **Stop valuing a base ahead of a runner who has stopped as an out attempt.**
   `visible_target` uses the next integer base whenever the runner's intended
   goal is not strictly behind his current path position. On the tick after
   arriving exactly at a bag, a runner can have decided to stop there while
   retaining his arrival `v` until `move_runners` next resets it. `holder_act`
   runs before `move_runners`, so it admits him as a moving runner and gives
   him an ETA to the *following* base. A high `p_out` can then send a throw home
   with nobody actually trying to score. This is an observation/target-generation
   defect before weighting, not evidence that a run-saving weight is too high.
   Proposed fix: derive visible movement toward a destination from recent
   observed displacement/heading and deceleration, distinguish arrival from
   leaving the bag, and retain a control/hold option. Never read intended goal
   as a defender's oracle. Measure these throws again after that work.

3. **Make tag and carry completion honest.** The first carry loop's comment
   says "forced runner", but its condition never checks `forced`. Stepping on
   third does not retire an unforced runner; a home catcher may legitimately
   wait to tag, while a third baseman can briefly run to a bag that offers no
   out. The later throw loop also uses ball arrival plus the same 0.05 s for
   both forces and tags. Proposed fix: compare force touch, actual reachable
   tag plan, and carry time with heading, get-up and stopping costs. Preserve
   legitimate waits for tags; do not count every catcher at home as a bad carry.

4. **Replace fixed destination rewards with situation value and error cost.**
   Below two outs the weights are `[0.9, 1.0, 1.1, 1.3]`, regardless of inning,
   lead, who else is on, or whether allowing one run is acceptable. Squaring
   `p_out` favors certainty but is not run expectancy. `p_out > 0.3` admits
   more-likely-safe throws even when controlling the ball might be worth more;
   there is no cost for overthrow, missed catch, extra advancement or relay
   handling. Proposed fix: use the §5.6/§8 out/safe/hold values and throw-error
   cost, informed by outs and base occupancy; use late close-game run/win value
   when preserving a particular run matters. Judge uncertainty from fielding
   IQ and known arm/capture ability rather than a universal 0.12 s. The screen
   alone does **not** justify raising the 0.3 cutoff: sometimes trying the only
   out is sound.

5. **Keep the two-out force preference; validate routing before changing it.**
   With two outs, an eligible force gets 1.0 and a tag 0.9 (`TWO_OUT_TAG_W`).
   That already expresses the owner's normal example. A tag is also a time
   play for run counting; a force or batter out before first cancels preceding
   runs on the third out. The sample does not justify tuning the 0.9. A better
   evaluator would explicitly value inning termination and the timing of runs,
   especially when a high-confidence tag competes with a close force.
   Direct-versus-relay routing currently minimizes believed travel time, with
   a fixed relay transfer fraction and the final receiver's arrival. It lacks
   each leg's catch/error cost and a tactical cut/hold decision; it chooses
   among at most one currently assigned relay man. Proposed fix, after higher
   priorities: compare completion and control value, cover continuity and both
   catches, rather than merely changing the route-time comparison. A zero
   count in the slower-route screen only verifies this narrow minimization;
   it cannot vindicate all cutoff positioning or tactical choices.

No proposed change above is implemented here. In particular, `out_weight`,
`TWO_OUT_TAG_W`, the 0.3 admission cutoff, the probabilities, evaluation order,
movement and random draws are unchanged.

## What the fielder would need to know

Outs, score, inning and runner occupancy already exist in `RulesState`; this is
mostly **unused context**, not information the game lacks. The current choice
function consumes outs, and otherwise largely reduces the situation to target
base and estimated arrival times.

What is missing from the decision interface/model is a runner observation with
reliable current direction, slowing/stopping and uncertainty; a belief about
whether an unforced runner will actually challenge the next base; the team's
preferred play and communicated cutoff instruction; teammate receiving readiness
with motion and ability rather than league-typical speed from rest; the holder's
own throw/catch reliability and the possibility of advancement on an error; a
tag plan's time and geometry; and the continuation value of a second out. A
fielder knows the public situation and his own abilities, can observe motion
and hear teammates, and can hold beliefs about all of these. He must never be
told the actual future path, realized execution noise, or a runner's private
planned destination. The diagnostic's retrospective state audit grants none
of that information to gameplay.

## Fix (2026-11-14): priorities 2, 1 and 3

Priorities 4 and 5 (score and situation value, `TWO_OUT_TAG_W`, the 0.3 cutoff,
route choice) are untouched. Spec: agent-models §5.5 "A4 carries", §5.6 "A4
implementation contract", §5.7 "A4 destination gate". Commits `98c58ed` (Sol,
priority 2), Sol's WIP `3d76e3b` reworked in `fe4b7cd` and `c2c58cb`, measured
on main `69826e4` merged (main `00445f5` changes only the client side).

**2. Targets from what the fielder can see.** A runner's out destination comes
from his path position and signed current velocity (`ChoiceParams::runner_target`).
Within `on_bag_m` = 0.05 m of a bag, or under `moving_mps` = 0.5 m/s, he offers no
moving-runner play; between bags positive motion targets the next bag, negative the
one behind. The world now zeroes a stopping runner's speed on his arrival tick,
which removes the stale arrival speed that produced the error (positions are
unchanged; the next tick zeroed it before). The private `goal` is read only by the
probe's retrospective audit. The same gate feeds the cutoff/relay target and the
holder's force-touch check, which had read `r.goal >= base`.

**1. One comparison.** `live/holder.rs` builds a table with, for every visibly
moving runner, a carry (within `carry_max_m` = 8 m) and a direct-or-relay throw,
each with its own completion time, in the existing Φ(·/0.12 s), p > 0.3, p² × w
rule. Carries come before throws in the table, so an exact tie keeps the carry
(no throw for the same out). A force beyond first with fewer than two out adds
`continuation_weight` (1.0) × p₁² × p₂² × w₂ for the force on the batter at first,
from the next holder's own throw (a carrier keeps his set-up clock, a receiver
needs 0.7 × transfer). The table is rebuilt every tick; a holder who gives up a
carry brakes (`Fielder::carry_to`), anyone else keeps his target. The automatic
nearby carry and the automatic swap of a throw within 6 m for a carry are gone.

**3. Non-force carries.** Implemented differently from the first A4 spec text,
which allowed a non-force carry only inside the 1.2 m tag reach. On main `52d6632`,
37 sustained non-force carries started farther out and **36 were tag outs at that
bag** (31 at second, mostly batters stretching a single, the middle infielder 1.3–
7.8 m from the bag). A ban gives those away. A carry to an unforced runner's bag is
now a tag plan weighed as a tag (two out: 0.9 against a force's 1.0; no
continuation) inside the same comparison, so it can no longer bypass a force. The
spec records the deviation. A counterfactual build with the strict rule gave the
same runs (5.06 against 5.07, seeds 0–299).

### The classes before and after (seeds 0–255)

Same probe, same classes. Before is main `69826e4` (arm strength by position and
A10 merged, so it differs from the 256 games in the Results table above). Games
diverge after the first changed choice, so a row compares rates, not the same plays.

| Class | Before | After | What is left |
|---|---:|---:|---|
| Ahead of a stopped runner | 148 (103 home, 45 third) | **0** | — |
| … of which through the cutoff | 23 | **0** | — |
| Carry behind a sure lead out | 129, all from the shortcut; 73 plays got two outs | 120 | 119 are double plays the holder weighed (continuation p₂ > 0.3); 111 of the 120 plays got two outs. The one other is 197/122: second 1.09 against third 1.08 |
| Non-force carry over a sure force | 23 | **1** | 187/132: none out, tag plan at third 6.45 m away with 1.4 s in hand against the force at first; the lead runner is tagged out. The weights intend this |
| Non-force carries starting beyond 1.2 m | (37 sustained on `52d6632`, 36 tag outs) | 44 | 40 tagged out at that bag |
| Risky lead over a sure trailing out | 2 | 2 | 63/108 and 102/36: weighed home tags at +0.15 / +0.12 s against a sure force at first (p² × 1.3 beats 0.9). Both outs made at home. Priority 4 |
| Early/leading home gamble | 0 | 2 | The same two plays |
| Late close-score home bypassed | 2 | 2 | 231/223 and 248/232: weighed double plays, two outs, no run |
| Speculative throw (margin < 0) | 132 | 100 | Priority 4; unchanged rule |
| Two-out home tag over a force; no-chance throw; slower cutoff route | 0 / 0 / 0 | 0 / 0 / 0 | — |
| Flagged commitments | 426 | 223 | |

Commitments 14,158 → 14,069: nearby-carry episodes 1,713 + 44 replacement
carries → 1,438 weighed carries; throws 5,678 → 5,876. Traced and untraced logs and
digests match on all 256 seeds in both builds.

The shadow formulas are retired: `choice_trace::skipped_throw_options` and the
`unweighed_throw` rows are gone, because every option in the trace is now one the
holder weighed (`selected_option` names it). The probe reports the sub-counts above.

### Tests

`tests/throw_choices.rs` replays the frozen contacts. 1/64: no throw home. 17/203:
no option toward third for the runner already back on second; throw first. 70/157:
carry second for the double play; with `continuation_weight` 0 the same play goes
home instead. 13/140: no carry to first; a lead force with at least 0.25 s in hand.
Since arm-by-position the first baseman is 9.1 m from his bag when he can throw, and
the comparison picks a 3-6-3 double play that ends the inning, so the fixture checks
the lead force, not home. The literal case (a sure force home beats carrying first)
is a constructed unit test, and so is the non-force weighing (two out: throw first;
none out: tag at third). `ChoiceParams::runner_target` has its own geometry test.

### What it did to the game

Standard run, main `69826e4` → this branch, seeds 0–299 [1000–1299]: runs
5.41 → 5.07 [5.62 → 5.28]; double plays (probe, seeds 0–255) 2.30 → 2.75 a game.
Counterfactuals on seeds 0–299: continuation weight 0 gives 5.27, the strict
non-force rule 5.06, both 5.26. So the **double-play continuation is −0.20 runs**
(grounders with 2+ outs .079 → .097). *Corrected after the follow-up below:* the
remaining −0.14 is **not** the choice fixes. It is the world change in fix 2 (a
stopping runner's speed zeroed on his arrival tick), which removed the flying start
of runners tagging up: without that zeroing the branch gives 5.32 [5.42], so it is
worth about −0.25 [−0.14] runs, and the holder's fixes themselves are within noise
of zero. Nothing was tuned to bring runs back.

**Open** (both answered in the follow-up below). Double plays 2.75 a game against
MLB's about 1.7. Line-drive and fly-ball 2+ outs fell (.011 → .006 [.011 → .008];
.014 → .007 [.013 → .006]). My first reading of "8 of main's 12 outs at a runner's
own base were batter-runners called out with no base play" was wrong: those are
caught fly balls (follow-up, A).

## Follow-up (2026-11-14): eight base-0 outs, double plays, sacrifice flies

Measured on the branch after the A4 merge (main `0a17ae8`) against main `69826e4`,
seeds 0–255 unless stated, with the new `examples/dp_probe.rs`
(`-- [games] [first_seed]`; `DP_LIST=1` lists the plays on stderr).

### A. The eight batter-runners "out at base 0"

Traced on `69826e4` (fact and rules-event dump of each play): 33/211, 55/159, 88/60,
101/22, 101/153, 155/101, 178/123, 235/220. **All eight are caught fly balls**
(`Catch`, then `RunnerOut { base: 0, kind: FlyOut }` on the `GloveSecured` tick;
three foul, five fair). Base 0 is the umpire's record for "no base play: at the
plate, a caught fly or a strikeout". The batter was standing on first because he
had run while the ball was in the air. They are **real outs under the rules, worth
0 free outs a game**; nothing was handed to the defense.

What tied them to a nearby carry: on the catch tick the fielder who caught the ball
committed a carry to a **different** runner, one standing on a bag with his stale
arrival speed (the "ahead of a stopped runner" defect). On the branch no commitment
falls on a caught-fly tick (8 → 0 in the sample). That path is the stale-speed
target, pinned by `ChoiceParams::runner_target`'s geometry test (a runner on a bag
at 8 m/s gives no target) and the 1/64 fixture. No rules or live test is added:
there is no rules defect to pin. My count of "outs at a runner's own base 12 → 3"
counted these eight because a batter starts at base 0; genuine doubled-off runners
were 4 → 3.

### B. Double plays: 2.75 a game against about 1.7

**Real figures.**
- Baseball-Reference 2024 situational batting
  ([page](https://www.baseball-reference.com/leagues/majors/2024-situational-batting.shtml)):
  33,097 GIDP opportunities (PAs with a runner on first and fewer than two out),
  3,227 GIDP, **9.8% per opportunity**; 13.6 opportunities and 1.33 GIDP a game.
  High confidence in the totals.
- FanGraphs, Baumann, 2 July 2025
  ([article](https://blogs.fangraphs.com/if-junior-caminero-had-any-more-gdps-hed-be-macroeconomics/)),
  Savant 2023–2025: ground balls with a runner on first and fewer than two out turn
  into double plays "about a third" of the time above 60 mph (buckets 32.9–38.4%).
  By the fielder who took it: P 19.6%, C 3.8%, 1B 21.5%, 2B 44.7%, 3B 37.8%, SS 54.1%
  (weighted over the infield, about 40%). Medium confidence: its exit-velocity
  shares do not sum to 100, no sample sizes are printed, and real pitcher, catcher
  and first-base grounders include sacrifice bunts, which the sim does not have.
- Home to first: the same article's 416 Savant sprint-speed players average
  **4.47 s**.
- Pivot catch-to-release: **no Statcast figure found**. A coaching site says 0.7–0.9 s
  at college and pro level, unsourced (low confidence). The 2016 Statcast figure in
  the sources note (second baseman, 1.2 s) is his exchange on all plays where the
  batter was out on his throw, grounders he fielded plus pivots, so it is not a
  pivot time.

**The sim** (branch; old main in brackets where measured):

| | Sim | Real |
|---|---:|---:|
| PA opportunities a game | 14.1 [14.1] | 13.6 |
| Grounder double plays per opportunity | **19.5%** [16.2%] | 9.8% |
| Chance grounders a game (fair, launch < 10°) | 5.2 [5.4] | |
| Converted per chance grounder | **52.5%** [42.8%] | about 33% (≥ 61 mph) |
| By first fielder, infield: P / 1B / 2B / 3B / SS | 70 / 73 / 62 / 75 / 72% | 20 / 22 / 45 / 38 / 54% |
| Pitcher's share of infield chance grounders | 20% | 8.2% |
| 2B exchange on the Statcast play set (batter out on his throw) | **0.70 s** (fielded 0.78, pivots 0.44) | 1.2 s |
| Pivot catch-to-release, fielder → second → first | 0.43 s (sd 0.07) | 0.7–0.9 s (unsourced) |
| Feed glove-to-release | 1.05 s | |
| Contact to the ball at first | 4.35 s (sd 1.4) | |
| Batter contact to first | 4.417 s (**sd 0**) | 4.47 s mean |

Applying FanGraphs' per-fielder rates to the sim's own fielder mix gives about 400
double plays in the sample instead of 703, so **the excess is in conversion per
chance, not in the number of chances** (+4%) or who fields them.

**Ranked causes.**

1. **Infield exchanges about half a second quicker than Statcast's on the matching
   play set** (0.70 s against 1.2 s), in grounders the second baseman fields (0.78 s)
   and in pivots (0.44 s). Counterfactual: the receiver's catch-to-release
   (`RELAY_TRANSFER_FRAC`) × 1.43 (to 1.0) or × 1.79 (to 1.25) gives conversion
   46.5% or 39.0% and 2.57 or 2.16 double plays a game. The pivot itself only moves
   0.43 → 0.52–0.53 s because the hurry rule lets him release early, so most of the
   effect comes through slower hurried throws. **Not a plain mis-set parameter:**
   the 1.2 s is not split by play type, `RELAY_TRANSFER_FRAC` also sets every relay
   and receiver throw, and raising the infield transfer itself would slow the throw
   on every grounder while grounder hits are already .35 against .24. It needs a
   pivot model (catch, foot on the bag or across it, clear the runner, throw) and a
   real pivot-time source. **Model change; stop.**
2. **No slide and no pivot clearance.** The runner from first never breaks up the
   play and the pivot man never has to avoid him (`live.rs` header: "no slides,
   rundown tactics, or collisions"). A real pivot man throws while avoiding the
   runner, which costs time and accuracy. Size not measured. **Model change.**
3. **The A4 continuation values the double play** (2.30 → 2.75; conversion 42.8% →
   52.5%). Turning two when it is likely is sound baseball; it amplifies cause 1 and
   is not itself the defect.
4. **The pitcher fields 20% of infield chances** (real 8.2%) and converts 70% (real
   19.6%). Part of the real figure is sacrifice bunts; the rest is the pitcher's
   reach after his follow-through. Not measured further.
5. **Every batter is the same runner** (4.417 s, sd 0) against a 4.47 s mean.
   0.05 s quick on average; the missing spread means no slow-footed double-play
   candidates and no burners. Small, and by design for average players.

Nothing in B was changed: no cause is a single mis-set parameter with a measured
value for the same quantity.

### C. Sacrifice flies and the line-drive and fly-ball double plays: one cause

After a caught line drive or fly ball with runners on, out attempts on advancing
runners fell 163 → 53 (sample) and the holder's first act became the return throw
(throws 71 → 9). Caught flies with a run scoring: **62.5 per 100 games on `69826e4`,
15.6 on the branch**. The real figure the lead gives is about 50.

Cause: fix 2's world change. `move_runners` now zeroes a stopping runner's speed on
the tick he arrives (`98c58ed`). Before it, a runner who went back to his base to
tag up kept his arrival speed while standing on it, because the tag-up branch
`continue`s before speed is ever reset. At the catch he then (1) judged the advance
with `runner_decide`'s running-start discount (`t_run × 0.75` when `v > 1`) and
(2) left the bag at that leftover speed, a flying start from standing still. That
was a bug, and it had been carrying the sacrifice-fly rate. With it gone, runners
on third start from rest (time constant 0.85 s), get no discount, need a 0.35 s
margin, and rarely go.

Counterfactual, the branch with only the arrival-tick zeroing removed: caught flies
with a run **63.3 per 100 games**; line-drive and fly-ball plays with 2+ outs 67
(old main 76, branch 37); grounder conversion unchanged (52.1%); runs 5.32 [5.42]
against the branch's 5.07 [5.28]. The holder's fix does not need the zeroing: with
it removed, throws ahead of a stopped runner stay at **0** and non-force carries
over a sure force at 1, because the target already ignores a runner within
`on_bag_m` of a bag.

Two ways forward; **the lead or owner decides**:
- **Remove the arrival-tick zeroing** (one block in `move_runners`): restores main's
  runner physics exactly, sacrifice flies and air double plays included, and keeps
  the A4 choice fix. It brings back the flying-start bug that was hiding the slow
  standing start.
- **Keep it and model the tag-up start**: a runner set on the bag who times his
  leave with the catch, from a measured third-to-home time on tag-ups (unsourced
  here), and a `runner_decide` that does not depend on a stale speed. Model change.

Either way, my ledger attribution of −0.14 runs to "visible targets plus one
comparison" was wrong; it was mostly this. Corrected in the ledger. The diagnosis
below (`work/sf-diagnosis`, found in parallel) traces the same mechanism with
`sf_probe` and its own figures; the lead has asked for the tag-up start to be
modelled (C), waiting on the tag-up sources note.

## Diagnosis: why sacrifice flies fell by three quarters (2026-11-14, `work/sf-diagnosis`)

Diagnosis only; no sim behavior was edited. The scorer audit found sacrifice flies at 178 [157]
per 300 games on `69826e4` and 45 [57] on `0a17409`, against about 150 per 300 in MLB. Fly
outs with a run scoring, 100 games (seeds 0–99), counted by `sim/crates/bb_game/examples/sf_probe.rs`:

| | fly outs, runner on third, <2 out | runner scores | put out | holds |
|---|---:|---:|---:|---:|
| main `00445f5` (before A4) | 108 | 60 | 5 | 43 |
| main `05666c0` (A4) | 98 | 16 | 2 | 80 |
| `05666c0` plus the experiment below | 114 | 64 | 4 | 46 |

### The cause: the runner, not the defense

The runner on third **holds** after the catch where he used to go, because his own race estimate
got 1.05 s worse. At the catch, `runner_decide` (`live.rs:2421`) estimates

```rust
let t_run = run_time((next as f64 * BASE_L - r.s).max(0.0), r.vmax, RUNNER_TAU) * if r.v > 1.0 { 0.75 } else { 1.0 };
```

and goes if `t_ball − (t + t_run) + noise > 0.35`. The factor 0.75 is a running start for a runner
already in motion (`r.v > 1.0`). A runner tagging up is not in motion, but before A4 he looked like
he was:

- **Before** (`00445f5`, `live.rs:2462`, the tag-up branch of `move_runners`): a runner who has come
  back to his base to tag up reaches `else if on_base { continue; }` on every tick, which skips the
  code that zeroes `r.v` (`live.rs:2491`, `if (goal_s − r.s).abs() < 1e-6 { r.v = 0.0; continue; }`).
  He stands on the bag with the speed he had when he got back to it, frozen: **6.19 m/s** in 102 of
  the 108 traced decisions at a catch (the other six 6.85–7.36 m/s), never zero. At the catch `r.v > 1.0` is true, so `t_run` for the 27.43 m
  from third to home is **3.14 s**.
- **After** (A4, `live.rs:2578–2584`, "End the physical stop on arrival, before the next holder reads
  its speed"): `r.v = 0.0` is set on the arrival tick, for every runner whose goal is reached,
  including a tag-up runner. At the catch `r.v = 0.00`, the factor is 1.0, and `t_run` is **4.19 s**:
  a standing start over 27.43 m (`run_time`, `RUNNER_TAU` 0.85 s).

So the 25 % running start the tagging runner used to get was an accident of a stale speed, and A4's
correct fix to the speed removed it. Nothing was meant to touch the sacrifice fly. The zeroing is
needed by A4 (the fielders' gate reads `direction × v`); it is the runner's decision rule that was
leaning on the bug.

### Three clear examples (identical in both builds up to the decision)

Same seed, same play number, same catching fielder, same catch tick, same noise draw, so the only
difference is `r.v`. `t` is play seconds at the catch (tick = t × 60). "ball needs" is `t_ball − t`:
the runner's estimate of when the ball reaches home, from the catch. margin is
`t_ball − (t + t_run)`; he goes if margin + noise > 0.35.

| Seed, play | catch t (tick) | build | `r.v` | `t_run` | ball needs | margin | noise | result |
|---|---|---|---:|---:|---:|---:|---:|---|
| 1, 54 | 4.53 s (272) | before | 6.19 | 3.14 | 3.79 | +0.65 | +0.12 | **goes, scores** |
| | | after | 0.00 | 4.19 | 3.79 | −0.40 | +0.12 | holds |
| 23, 111 | 6.37 s (382) | before | 6.19 | 3.14 | 4.17 | +1.03 | +0.05 | **goes, scores** |
| | | after | 0.00 | 4.19 | 4.17 | −0.02 | +0.05 | holds |
| 45, 74 | 5.41 s (325) | before | 6.19 | 3.14 | 4.16 | +1.01 | −0.03 | **goes, scores** |
| | | after | 0.00 | 4.19 | 4.16 | −0.03 | −0.03 | holds |

There are 28 such plays in seeds 0–99 where both builds make the same catch: the runner scored in 25
and was put out in 3 before; he held in 27 and was put out in 1 after. Others: (4, 59), (8, 85),
(23, 117), (30, 94), (44, 55), (45, 46), (46, 152), (47, 103), (49, 40), (53, 37), (57, 147),
(64, 76), (66, 42), (67, 187), (67, 203), (76, 213), (87, 95), (90, 107), (92, 163), (99, 179).
Most have a ball-needs time of 3.5–4.7 s, between the two `t_run` values, which is why the runner
went before and holds now.

### What it is not

- **The defense.** A runner who goes is put out at about the same rate (before 5 of 65, after 2 of 18,
  with the experiment 4 of 68), and a runner who holds is standing on his base with the ball just caught, so there is no
  play on him in either build (I did not trace the fielders' choices in the held plays; the
  unchanged out rate and the restored scoring rate with only the runner's rule changed say the
  defense is not the cause). The gating of
  fielders' targets on visible motion (`visible_target`, `runner_target`) does not enter, because the
  decision comes at the catch, before anyone is moving. The decision is the runner's, and it is
  made from `Ball::Held`, which A4 did not touch.
- **The runner's read of the ball.** `ball_time_to` (`live.rs:2336`) gives the same `t_ball − t` in
  both builds for the same play (the "ball needs" column).
- **Noise.** The draw is the same in both builds; margin shifts by exactly the `t_run` difference of
  1.05 s.

### Experiment (not committed) and how much of A4's run drop it explains

One edit restores the old behavior for a tagging runner only: at `live.rs:2583` add `&& !r.tag_up` to
the condition that zeroes `r.v` on arrival. Result, seeds 0–99 for the probe and 300 games per set
for the runs:

| | probe (scored / out / held) | runs / team-game, seeds 0–299 [1000–1299] |
|---|---|---|
| before A4 (`00445f5`, `980a7dc` row) | 60 / 5 / 43 | 5.41 [5.62] |
| A4 (`05666c0`) | 16 / 2 / 80 | 5.07 [5.28] |
| A4 with the edit | 64 / 4 / 46 | 5.32 [5.42] |

Sacrifice flies come back to the old rate, and **about 0.25 [0.14] of A4's 0.34 [0.34] run drop is this
runner bug, not better defense** (set noise is about 0.1). What is left, about 0.1–0.2 runs, is
A4 itself plus noise. The edit is a diagnostic, not the fix: it also leaves the speed frozen for the
fielders' gate, which A4 means to be zero.

### The fix I would make (for opus-a4)

1. Keep `r.v = 0.0` on arrival (A4's change is right: a runner standing on a bag is not moving).
2. Make the tag-up start an explicit rule in `runner_decide`, not a side effect of a stale speed.
   Give it a named [E] parameter, for example `TAG_UP_START_FACTOR` (initially 0.75, which reproduces
   the old behavior exactly: `t_run` 3.14 s from third). Pass a `leaving_on_catch: bool` from the
   tag-up branch (`live.rs:2462–2470`, where `self.runner_decide(k, b)` is called right after
   `tag_up = false`) and write
   `* if leaving_on_catch { TAG_UP_START_FACTOR } else if r.v > 1.0 { 0.75 } else { 1.0 }`.
   Better, replace the speed proxy (`r.v > 1.0`) with the fact it stands for: the runner is moving
   toward `next` (`r.direction > 0.0 && r.v > 1.0`).
3. Choose the factor from the sacrifice-fly rate: fly outs with a run on third and <2 out should score
   about 55–60 % (the old 60 of 108); that is in line with the count of about 150 sacrifice flies per
   300 games. 0.75 has no source; it gives 27.4 m in 3.14 s, against 4.19 s from a
   standing start. A tag-up is neither (the runner leans and leaves on the catch), so the right value
   is somewhere between and should be measured. The decision threshold (0.35 s) and runner noise (0.15 s) move the rate the same way, so fit them
   together or not at all.
4. Tests: a runner tagging up on third with a fly caught 4.1 s of throw from home goes and scores; the
   same with a 5.0 s throw holds; and a regression that `sf_probe 0 100` stays near 60 scored of 108.

To reproduce: `cargo build --release -p bb_game --example sf_probe` then
`BB_SF_TRACE=1 sf_probe 0 100 v` after adding the one-line trace below to `runner_decide` (local
only; the trace is not committed). The example prints one line per qualifying play
(`seed N play M outs O runner R scored|OUT|held`).

```rust
if std::env::var("BB_SF_TRACE").is_ok() && r.start == 3 {
    eprintln!("SF play {} t {:.2} runner {} v {:.2} t_run {:.2} t_ball-t {:.2} margin {:.2} noise {:.2} go {}",
        self.inp.play, self.t, r.id, r.v, t_run, t_ball - self.t, t_ball - (self.t + t_run), noise, go);
}
```

## Where things stand (2026-11-14, about 05:15; branch `work/a4-throw-choices` at main `a9a95b2` + fixes)

**Job: the 225–300 ft tag-up gap, speed for accuracy, and the plate.** Seeds 0–299 [1000–1299] unless stated; rows in docs/metrics-ledger.md §3.

1. *Arm strength's definition* (Baseball Savant leaderboard, read 2026-11-14): the mean of a player's hardest throws, top 1% at first base, 5% at second, short and third, 10% in the outfield (minimums 100 / 75 / 50 throws); it notes that many lobs are captured. So the sim's arm is near max effort, and a throw home on a tag-up is right at it.
2. *At the plate* (`sf_probe`, 600 games): only 9 of 271 sent runners faced a throw the catcher took. The 4 caught 2.3–3.6 m off line were all safe (the runner never came within tag reach); the rest were out, tags 0.28–0.73 s after the catch. Off-line throws already cost the defense. How often real throws home are cut off: unsourced (tag-ups note §3).
3. *The runner's belief was the gap's main side* (`1677fc2`): he read the outfielder's release as 0.85 s where it is 0.98 s with the turn and footwork of a throw home. He now reads the set-up as the fielder's own model builds it from what he sees. 275–300 ft sent 7% → 35% (safe 91%).
4. *Speed for accuracy* (`4477892`, `c8ab5de`, `6b7e1ac`; agent-models §5.5): effort 0.80–1.0 of the arm, direction σ × 0.768 at 0.80 (Freeston 2014). The least effort that keeps the 0.25 s margin; full for a double-play feed with a likely second out, for relays, and for a no-play throw while any runner moves; 0.80 only with every runner standing. Two defects were found and fixed by measurement on the way: returns at the floor let runners take bases (+0.24 runs), and feeds at the floor lost double plays (1.73 → 1.55 a game).
5. *Now:* runs 5.81 [6.25] (main `a9a95b2` 5.84 [6.11]); GB hits .327 [.332]; double plays 1.70 a game (MLB about 1.7); set throws taken in the air 96.9% (from 96.1%); sacrifice flies 0.257 a team-game (MLB 0.24–0.27), 55.8% of outfield catches with a runner on third scoring (MLB 75–80%), 1.4% put out (2–3.5%).

**Still open, in order:**
- *225–275 ft still holds* (MLB sends about 40%). What is left is the decision rule: a 0.35 s margin against 0.15 s noise asks for a 99% safe estimate, where agent-models §6.3's own rule is run expectancy (break-even about a third with one out). Replacing it is a model change and an owner decision (Gemini tag-up review P1-1).
- Gemini tag-up review P2-2 (tag geometry, the carry's double tag time), P2-3 (τ fitted to faster-than-average runners: 4.10 s at league speed would mean τ about 0.75 s; batter box offset gives left- and right-handed batters the same time), P2-4's restored counterfactual: deferred, see the review's Resolution.
- Runs remain 1.4–1.9 above MLB; nothing was tuned.
