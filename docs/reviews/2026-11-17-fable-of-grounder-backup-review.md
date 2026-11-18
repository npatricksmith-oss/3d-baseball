# Review: an outfielder takes a grounder through the infield that the outfield left (saved play 65147/10)

**No P1. Merge.** One P2 to put on the backlog (the same misplay on a ball still in the air is not covered) and four P3s, none blocking.

Reviewer: Claude Fable 5.2, author of none of it (Codex near its weekly limit). 2026-11-17.

## What was reviewed

Commits `d32d065..e048a01` on `work/of-mutual-defer` (not `312cb87`, `bef0fa8`): `Sim::outfielder_to_reconsider` and the filter in `Sim::hear` (`sim/crates/bb_game/src/live.rs`), the two new tests, the new bullet in `docs/agent-models.md` §5.7, and the newest row in `docs/metrics-ledger.md` §3.

**What the code does, in one paragraph.** At a hearing where someone's known chase or call has changed, if the ball has bounced, all three outfielders are known to have left this ball, at least one infielder is known to be going, and the ball as all see it (`public_path`) stops beyond `INFIELD_BREAK_R`, then the outfielder first in the shared order among those known to be idle or backing up is picked. If he himself is idle or backing up and has a read of this ball no older than `HEAR_READ_AGE_S`, he runs `decide`. Because every other outfielder is known not to be going, `first_by_own_estimate` leaves them all out, so `decide` returns Go unless he has heard a sure call. In effect the picked man goes, at his own read's intercept.

## Answers to the questions

**1. The gate on ordinary grounders: right.** Probe, 250 real games (seeds 0–149 and 1000–1099, 18,577 balls in play):

| | Plays |
|---|---|
| Gate sent an outfielder | 914 (4.9% of balls in play, 3.7 a game) |
| ... an infielder fielded it | 818 (89%) |
| ... the sent outfielder fielded it | 54 (5.9%, 0.22 a game) |
| ... nobody secured it (the one traced was a foul grounder) | 40 |
| ... another outfielder fielded it (the sent man missed it) | 2 |

- The rule is not a rare rescue. All three outfielders leave a bounced ball to each other on about one ball in play in twenty. That is the poor post-bounce read already on the fielding backlog.
- He does go after balls an infielder fields, nine times in ten. That is the ordinary outfield backup: on every other grounder one outfielder goes at the first read and "does not stop for an infielder" (`first_by_own_estimate`'s comment). The gate gives the same picture 0.3 s late.
- He never left a backup job: the sent man was Idle in all 914 cases. The gate opened 0.55 s after contact in 97% of cases, at the first hearing, before the backups are dealt.
- No flipping. Once sent he is a chaser, and a chasing outfielder stops only for another outfielder known to be going who is ahead of him in the shared order. 0.3 s later he is known to be going and the gate shuts. 43 sent men stopped later; the two traced were a new loose ball after a throw (seed 22 play 73) and a recall after the ball had got past him (seed 13 play 9). No hearing sent two outfielders.
- **The trigger is the right public one.** I would not add "the infielder cannot reach it". It could be worked out publicly (no known infield chaser has a standing intercept of `public_path` inside `INFIELD_BREAK_R`, by `seen_intercept` on that path), but it would hold the outfielder back on exactly the balls an infielder then boots, and it would make this path differ from the first-read rule.

**2. Isolation: holds.** `outfielder_to_reconsider` reads `ball_still_up` (the ball's own bounce, seen by all), `known`, `known_job`, `public_path` and `publicly_before` (`public_rank` uses visible positions, velocities, known speed and reaction). The filter then reads only the picked man's own `role`, `read_at`, `read_ball`, `belief` and `call`. `decide` for him reads his own read and `known`. When the picked man is privately unable (stale read, a job just taken), nobody else is sent in his place, so his private state moves no one else. The harness covers it: the gate sent a man 8 times in seeds 0–1, a hearing that changes a job is checked every time, and the run made 452 `hear` snapshots with 0 violations.

**3. `read_ball == ball_gen`: harmless, and in games it never decides anything.** Every ball that comes free goes through `set_free`, which calls `assign`, which has all nine read the new ball at once. So a man with a belief always holds a read of the ball now loose, and after a deflection everyone can still decide. See P3-2.

**4. The tests.** Both older tests are unchanged in the diff and still test what their comments say: with nobody known going the gate needs a known infield chaser, and then the short roller's path ends inside the infield; with two outfielders chasing, the gate needs all three known not going. The new roller test is a real check: without the `INFIELD_BREAK_R` condition `decide` returns Go for the picked outfielder and the assertion fails. The saved-play test is acceptable as a pin of the owner's play, as others in the file are. See P3-3 for what neither test pins.

**5. The spec line: loose.** See P3-1.

**6. Invariants: clean.** `INFIELD_BREAK_R` and `HEAR_READ_AGE_S` are existing named parameters. No new draw (the sent man goes on the read he has). `m::hypot`. No hash containers; the pick is a `reduce` over 6..9 in order with the shared tie-break.

## Findings

### P1

None.

### P2

**P2-1. The same misplay on a ball still in the air is not covered.** `live.rs:2641` (`ball_still_up()` shuts the gate) and `live.rs:2887–2905` (the gate is looked at only at a hearing where a known chase or call changed, so a ball that lands later is not looked at again).

Evidence: seed 43 play 74 (`BB_DEBUG_COMM=1 ... trace_play -- 43 74`). A low fly comes down at 2.07 s after contact at (18, 57). At the first read all three outfielders stay (centre and right each leave it to the other) and the second baseman goes. At the 1.004 s hearing the ball is up, so nobody is sent. The right fielder goes only at the recall at 2.47 s and secures the ball at 8.77 s at (41.5, 106.0), by the wall. This is the owner's saved play again with the ball in the air. It is not a regression: `d32d065` does the same. In the 250 games I found four candidates of this kind against the 54 the gate now saves; only this one was traced.

Suggested fix, on its own branch with its own ledger row: open the gate for a ball in the air whose public first bounce (`public_path().first_bounce()`) is beyond `INFIELD_BREAK_R`, and look at the gate when the ball comes down as well as when a call changes. Pop-ups stay out because they come down inside. Until then, list it in TASKS.md beside the post-bounce read item.

### P3

**P3-1. The spec bullet says more choice than the code gives.** `docs/agent-models.md:566`. "A chance to back up the play" and "decides again on his own recent read": the man becomes a chaser (`Role::Chase`), not a backup, and with the other two outfielders known not to be going his decision is always to go unless he has heard a sure call. His read supplies where he runs, not whether. "Yields to a sure call that outranks his": his own call is tier 0, so he yields to any sure call. The bullet also omits that the gate is looked at only when a known chase or call changes. Suggested text: "the first outfielder in the order ... goes for it, running at his own recent read's intercept, unless he has heard a sure call; this is looked at when a chase or call becomes known".

**P3-2. `fd.read_ball == self.ball_gen` is undocumented and changes nothing in games.** `live.rs:2913`. It also applies to the old `call >= 1` path, and `grounder_break` and `partner_fields_it` test recency without it. Established by reading (`set_free` → `assign`), not by a probe. Either drop it or give it a comment saying it is a guard for a state games do not reach.

**P3-3. The positive case is pinned only by one seed.** `live.rs:12743`. The saved-play test checks who secured the ball and where. It does not pin that exactly one outfielder is sent, that he is the first in the shared order, or that the others stay; and it needs finding afresh whenever the game moves. Suggested: a fixture twin of the roller test (`live.rs:8786`) with a hard grounder through the hole and a known infield chaser, expecting one outfielder, the first by `publicly_before`, to chase through a second of hearings and the other two never to.

**P3-4. The ledger's "unexplained" can be narrowed.** `docs/metrics-ledger.md:175` and `:378`. The table above gives the mechanism's size: an outfielder reaches a ball through the infield 0.3 s after the first hearing instead of at the recall on about 0.22 plays a game, which fits runs −0.02 with reached-on-BIP flat. The HR/PA movement is still not traced.

**P3-5. A comment was lost.** `live.rs:2906–2908`. The rewritten comment no longer says that a ball every man has left goes to one man by `fallback_chaser`, which the block below still does.

## Verification

- Read the diff and the surrounding code (`decide`, `first_by_own_estimate`, `ahead_of`, `public_rank`, `known`, `set_free`, `assign`, the test helpers, the isolation harness).
- `trace_play -- 65147 10` with `BB_DEBUG_COMM=1`: both outfielders stay at 0.661 s, the right fielder is sent at 0.961 s and secures at (29.4, 89.3) at 5.08 s, as the ledger row says.
- Throwaway probe (an example marking each play's end after its `BB_DEBUG_COMM` lines, with a parser; both deleted, nothing of them committed): games 0–149 and 1000–1099, the figures above. Traced seed 43 play 74, seed 22 play 73, seed 13 play 9, seed 1012 play 116 (the old `call >= 1` path on a fly, not the gate) and seed 0 play 103.
- After `touch live.rs`, a fresh `cargo test --release -p bb_game --lib` for the two new tests, the two older tests and `isolation_no_private_state_reaches_a_teammate_in_real_games`: 5 passed. Isolation: 1,030,608 checks, 0 violations, 452 `hear` snapshots.
- Not run: the whole workspace and the standard run (the lead ran both on `e048a01`), `isolation_many_seeds`, and a before/after comparison on `d32d065`.

## Resolution

Resolved by the lead (Opus 5.5), 2026-11-17. No re-review: every change below is a comment or docs edit, with no behaviour change.

- **P2-1 (a ball still in the air): deferred.** It is not a regression (`d32d065` does the same). It needs its own branch, ledger row and review. It is in TASKS.md under "2. Fielding", next to the post-bounce read, and noted at the end of the agent-models §5.7 bullet.
- **P3-1: fixed.** The agent-models §5.7 bullet now says that the man goes unless he has heard a sure call, that his read sets only where he runs, and that the rule is looked at only at a hearing. It gives the probe's size and names the ball-in-the-air gap.
- **P3-2: kept, with a comment.** The guard is documented in `Sim::hear` as one games never reach. It is kept rather than dropped so that a later path to a stale read stays safe.
- **P3-3: deferred.** The fixture twin of the roller test (one outfielder sent, the first by `publicly_before`, the others never chase) is in TASKS.md under "2. Fielding".
- **P3-4: fixed.** The ledger row and the §5 line give the mechanism's size from this review; HR/PA is still listed as not traced.
- **P3-5: fixed.** The `fallback_chaser` sentence is restored in `Sim::hear`'s comment.
