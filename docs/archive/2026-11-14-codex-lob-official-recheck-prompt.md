# Recheck: left on base by the official rule, after your review

You are reviewing, not authoring. Read `CLAUDE.md` first. You are in the worktree of branch `work/lob-official`. Your review is `docs/reviews/2026-11-14-codex-lob-official-review.md`; read it and its Resolution. The fixes are commit `c35f800` (range `61f9deb..c35f800`; the commit after it only merges main's docs and two read-only examples). **Open with one line: any P1 or not, and merge or do not merge.**

Check, by running tests and your own probes (yours from last time are under `sim/target/lob-review-probe/`):

1. **P1-1:** is "put out" now the outs actually recorded through the third out, in every path, and does your two-held-passes probe give LOB 3? Does scenario `lob/lob-004` derive its expectation from the rule and fail on `cbbbebd`? Any other place where a flag other than a recorded out removes a runner from the count?
2. **P1-2:** `bb_stats` computes left on base from events again (reached, `RunScored`, `RunnerOut`, the stranded batter-runner of 9.02(g)) and does not read `HalfInningEnded.left_on_base`. Is the scorer's own count the official definition in every case of your first review's table (nullified home touch, between bases, cancelled pending run, held passes, ignored later outs, walk-offs, awards)? Do the three committed fixtures balance (I1) with the LOB the old reader gave? Does the new cross-check (scorer's per-half count equals the engine's scalar in new games) hold over your 600-game audit, and is it a real check (two independent computations) rather than one value compared with itself?
3. **P3s** fixed as the Resolution says?
4. Anything the fix broke: full `cargo test --workspace --release`; `sim-cli games --n 300 --seed 0` output identical to main `6ef084a`'s apart from the timing line.

**How to work.** The laptop overheats and another agent is measuring: prefix every cargo build, test and sim run with `taskset -c 0-7 nice`, `export CARGO_BUILD_JOBS=2`, one heavy job at a time. `export PATH=$HOME/.cargo/bin:$HOME/.local/bin:$PATH ZIG_GLOBAL_CACHE_DIR=$PWD/sim/target/zig-global-cache ZIG_LOCAL_CACHE_DIR=$PWD/sim/target/zig-local-cache`. Build in `sim/`. Probes go under `sim/target/`.

**Rules.** Edit no source, test, spec, ledger, or TASKS file, and not the first review. Do not commit. Write exactly one file, `docs/reviews/2026-11-14-codex-lob-official-recheck.md`: the one-line verdict; each earlier finding as confirmed fixed or not, with evidence; any new findings ranked P1–P3 with file and line; `## Verification`; a final `## Resolution` containing only "Open.".
