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

## Trial 2 (cm-trial-2, exec isolation)

Job: make AGENTS invariant 3 ("decision code never receives an `Executed*`") real and tested.

1. **Programs run:** 10 REPL programs. Two read the job file and the code (specs, `motor.rs`, TASKS) in one fan-out each. One wrote the test file and submitted its first queue job. One checked the result. One planted a violation in `pitcher.rs`, ran the real scan, and reverted it with `git checkout`. One made the four doc edits. One ran the final checks; the rest were small follow-ups.
2. **Job queue from a program:** it worked as for a human. `bin/job.sh run` returned an id, and `bin/job.sh wait <id>` blocked inside the program and returned the exit code (0 here). Two jobs were submitted together and the second queued behind the first. The longest program ran 35.5 s (two queued test runs plus the docs check) and came back normally, so the 30 s figure is when a program is detached and keeps running, not a kill. The hard stop is the 10 minutes the docs give. I never reached it, and `wait` itself caps at 540 s, so a program that calls it once stays inside it. No timeout message was seen.
3. **In the way:** nothing blocked. Writing Rust source inside a JS template literal needed a placeholder for backticks (`@@` replaced after the fact), and the display shows `&`, `<`, `>` as entities, which makes Rust code in output hard to read. A mutation check (plant, run, revert) was easy to do in one program.

**Coordinator, after trial 2:** a full cycle (implementation, Code Mode reviewer, fixes in the resumed worker session) ran without a tool error. Resuming needs the variable on the start line (`CLAUDE_CODE_EXPERIMENTAL_CODE_MODE=1 claude --resume <id>`). After each turn Claude Code may leave a suggested next prompt in a worker's input box (e.g. "Merge the trial branch into main"); it is not sent, and it is not the owner's. Clear it (`ctrl+u`) before quitting the pane.

## Trial 4 (researcher, swing research)

Job: distil eight sources (papers, a dissertation, a dataset, four videos) into notes under `docs/research/swing/` and a synthesis.

1. **Programs run:** about 14 REPL programs. One read the job and docs. One read the current swing code and specs in a single fan-out. Eight launched the source subagents, one or two per program, with a shared prompt held in a REPL variable. The rest read the notes, wrote the synthesis, ran the docs check and committed.
2. **Subagents from Code Mode: they worked.** `tools.Agent` returns at once with `async_launched` and an id; the completion arrives later as a task notification, not as the program's return value. So a program cannot `await` an agent's result, and a `Promise.all` over agents returns in under a second. I kept at most 3 running by launching the next as each notification came. Each subagent wrote its own note and returned about 10 lines, so the raw sources never entered my context (source 6 took a subagent about 4 minutes; the others 25–100 s).
3. **Helped:** a prompt template in a variable, launched with a one-line call per source; reading all notes in one program; running the docs check and the commit in the same program.
4. **Got in the way:** the task notifications are the only way to learn that an agent finished, so the wait is a `Poll`, not a script. Output shows `&amp;`, `&lt;` and `&gt;` as entities, which makes the notes' quoted figures harder to read. Writing the synthesis as a JS template literal needed care with backticks.
5. **Outcome:** two of the eight sources gave almost nothing (a vendor promo with no speech, and a paper blocked by the publisher so only the abstract was read); that is stated in their notes, not filled from memory.
