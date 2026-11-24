# Code Mode trial notes (Q40, worker cm-trial, 2026-11-23)

Job: audit TASKS.md §7 (26 bullets) against the code.

1. **Programs run:** 8 REPL scripts. One read §7. Five fanned out about 50 `grep`/`sed`/`ls` probes in parallel through `Promise.all` (one object key per bullet) and returned them as one object. Two were follow-ups on bullets that were still doubtful. Two were edits and writes.
2. **Helped:** one script replaced about 10 sequential shell calls, and I read the probe results together. Cutting each probe to 2,500 characters kept the context small. **Got in the way:** `grep` over `sim/crates` pulls in `generated/` and `target`-adjacent files, so each probe needed `head` or its own filter. Output shows `&`, `<` and `>` as HTML entities. That is display only, but it makes Rust generics and closures look wrong. I hit no errors or refusals. Each `$` command ran in the worktree, which has spaces in its path, and I had no quoting trouble because I passed commands through `bash -c` as a single value.
3. **Before you use it:**
   - A script has a 30 s default timeout, and a long `cargo` run needs the job queue, not a script.
   - `$` merges stderr into stdout.
   - Keep probes read-only and cheap.
   - Write the probe table as data (name → command) so one loop runs and prints it.
   - `grep` exit 1 does not throw, but other non-zero exits do, so use `.nothrow()`.

**Audit result:** 0 of the 26 bullets were done, and I deleted none. Evidence for each "still open" call:
- Composure: `motor.rs` is the only file that mentions it.
- HBP evasion: `pa.rs:89-92, 373` has the body capsule for contact only, with no evasion.
- Executed isolation: there is no `trybuild` or `compile_fail` anywhere.
- `criterion`: it is in no `Cargo.toml`.
- Corpus: 136 TOML files.
- Interference, infield fly, bunts, balks, uncaught third strikes and catcher blocking: there is no code for any of them.
- Passing log: `eventlog.rs` never writes `RunnerPassed`.
- Research items R8–R11, the flight/bat/restitution sourcing and the Savant fixtures: the specs still list them as open.

I changed the heading date and added the `RunnerPassed` detail to one bullet.

## Coordinator's findings (Q40, 2026-11-23)

Owner decision Q40 (2026-11-23): trial Claude Code's experimental Code Mode on workers, one at a time, started per worker with `CLAUDE_CODE_EXPERIMENTAL_CODE_MODE=1` on the start line (never in a settings file). Claude Code 2.1.331.

- **Status hooks still work.** With `BB_WORKER_TRACE` on, `PostToolUse` fired for every call a program made (each `Bash`, `Read`) and once more for the program itself (`REPL`; also `Poll`). The heartbeat and the `working` → `stopped` change behaved as in [worker-status-hooks](worker-status-hooks.md). Some trace lines were not valid JSON (trace only; the status file was fine).
- **Parallel commands.** One program ran 12 shell commands at once. Harmless for probes; a program must never start `cargo` builds or tests in parallel (the laptop overheats). Builds and test runs stay in the job queue.
- **Start-up.** The first Code Mode session on the machine shows a one-time confirmation and restarts; the worker's prompt survived it. A Code Mode session resumes only with the variable set.
- **Guardrail.** Auto mode checks a program once, not each call, so the worker was started with `--settings '{"permissions":{"ask":["Bash(git push *)"]}}'`.
- **Unchecked:** the worker's "30 s default timeout" per program; the docs give 10 minutes. Check before relying on either.
- **Cost:** the audit took about a minute of work over 8 programs for 26 items. There was no control run, so no token comparison yet; the next trial job should be one with a comparable past run.
