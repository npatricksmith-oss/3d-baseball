# Independent review: an outfielder takes a grounder through the infield that the outfield left (saved play 65147/10)

You are reviewing, not authoring. The first draft (`1186254`) was written by an Opus lead, and the version under review (`75f50a9`) by a Codex worker. Codex is near its weekly limit, so by the owner's rule this review goes to a Claude model that wrote none of it. Read `CLAUDE.md` first: its invariants bind this review. You are in the worktree of branch `work/of-mutual-defer` (`/home/natex/Projects/bb-wt-of-defer`).

**Commit range:** `d32d065..e048a01`. Ignore `312cb87` and `bef0fa8`: they are the owner's CLAUDE.md lines. The code is `sim/crates/bb_game/src/live.rs`: `Sim::outfielder_to_reconsider` (new), the filter in `Sim::hear`, and two new tests (`the_outfield_leaves_a_short_roller_to_the_known_infield_chaser`, `an_outfielder_takes_a_grounder_through_the_hole_both_left_to_the_other`). The spec line is the new bullet in `docs/agent-models.md` §5.7, "A ground ball through the infield". The ledger row is the newest in `docs/metrics-ledger.md` §3.

**The play.** In seed 65147 play 10, a chopper goes through the right side. At the first read, the centre and right fielders each estimated that the other got there first, so both stayed (their snapshot reads of the bounced ball are poor at that range). The first and second basemen broke for it. That kept `fallback_chaser` from acting, since it acts only when nobody is going. `hear` re-decided only chasers and men with a call ≥ 1, so no outfielder went until the once-a-second re-call. The ball rolled to the wall for an inside-the-park home run. Trace: `cargo run --release -p bb_game --example trace_play -- 65147 10`, with `BB_DEBUG_COMM=1` for the calls.

**Open with one line: any P1 or not, and merge or do not merge.** Keep it short and proportionate: the owner asked the team not to overtest. The lead has already run the whole workspace and the standard run on `e048a01`, so do not repeat them. Use targeted probes only.

**Questions, ranked by risk:**

1. **Is the gate right on ordinary grounders?** The public path ignores fielders, so almost every grounder's predicted end lies past `INFIELD_BREAK_R`. On a routine grounder at short, with the outfield known to have left it and the shortstop known to be going, the first outfielder in the shared order now decides again at every hearing. Check whether he ever goes after balls an infielder fields, flips between going and staying, or leaves a backup job that should hold. Probe a few hundred real grounders. Also judge whether "every outfielder known not going, plus an infielder known going" is the right public trigger, or whether a ball the infielder cannot reach should be part of it, and say how that could be worked out from public facts.
2. **Isolation (CLAUDE.md invariant 2, agent-models §5.7 "no man's job is ever re-worked because of another man's private state").** `outfielder_to_reconsider` must read only what all can know: `known`, `known_job`, `public_path` and `publicly_before`. The filter then reads the chosen man's own `role`, `read_at` and `read_ball`. Confirm that nothing private to another man decides who reconsiders. Check whether the isolation harness (`isolation_no_private_state_reaches_a_teammate_in_real_games`) covers this hearing path on plays where the gate opens.
3. **`recent` now also requires `fd.read_ball == self.ball_gen`.** This changes the old `call >= 1` path too: a man holding a read of an earlier ball no longer re-decides. Is that right, is it documented, and could it stop a man who should decide (for example after a deflection starts a new ball)?
4. **The tests.** `a_ball_everyone_left_goes_to_one_man_and_first_stays_covered` and `two_chasers_who_each_think_the_other_is_first_do_not_both_stop` failed with the first draft and pass now, unchanged. Confirm that they still test what their comments say. Are the two new tests checked against independent expectations, or do they restate the code? The saved-play test is tied to one seed; say whether that is acceptable here.
5. **The spec line** in agent-models §5.7 says the man is given "a chance to back up the play", but the code sends him to decide whether to chase. Does the text match what the code does?
6. Invariants: named [E] parameters, no new randomness, determinism, math through `bb_math::m`, no HashMap iteration.

**How to work.** The laptop overheats. Prefix every cargo build, test and sim run with `taskset -c 0-5 nice`. Set `export PATH=$HOME/.cargo/bin:$HOME/.local/bin:$PATH CARGO_BUILD_JOBS=2`. Run one heavy job at a time, and read the CPU temperature first (`k10temp` under `/sys/class/hwmon`; above 90 C, wait). Put probes under `sim/target/`, as throwaway examples you delete before committing. Do not run `isolation_many_seeds`. Note: an earlier test binary in this worktree was stale. If a result looks odd, `touch crates/bb_game/src/live.rs` and rebuild.

**Rules.** Edit no source, test, spec, ledger or TASKS file. Write exactly one file, `docs/reviews/2026-11-17-fable-of-grounder-backup-review.md`, containing:
- the one-line verdict;
- what was reviewed;
- findings ranked P1 (wrong results or a broken invariant), P2 and P3, each with file and line, evidence or a probe, and a suggested fix;
- a concise `## Verification` section;
- a final `## Resolution` that says only "Open.".

Commit that one file on this branch and nothing else. Do not merge or push. Then stop.
