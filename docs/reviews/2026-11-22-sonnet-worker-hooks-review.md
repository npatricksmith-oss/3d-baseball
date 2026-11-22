# Review: worker status hooks and job queue (`work/worker-hooks`)

Reviewer: Sonnet (wrote none of the change). Range `fbe5c15..ae461ab`. Tooling only; no sim build or test run. The `sim` group was idle during the review (one tiny job, `rev-sim`, to check the wrapper); my other jobs ran in `light`.

## Findings

### P2-1 `bin/worker-wait.sh:68-79`, `bin/job.sh:203-204,227,251`: an option with no value loops forever
`--timeout`, `--stale`, `--dir` (worker-wait) and `--group`, `--label`, `--max`, `--tail` (job.sh) do `x=$2; shift 2`. With the option last, `shift 2` fails with one argument left, nothing is consumed, and the `while` repeats for ever (100% CPU).
Checked: `timeout 5 bin/worker-wait.sh a --timeout` and `timeout 4 bin/job.sh run --group` both ran until `timeout` killed them (rc 124). A `--timeout abc` also hangs (no numeric check).
Fix: `[ $# -ge 2 ] || die "option needs a value"` before each `shift 2` (or `shift 2 || break`), and validate numbers with `case $v in ''|*[!0-9]*) die ...`.

### P2-2 `bin/worker-wait.sh:94`, docs/worker-status-hooks.md "Waiting": a stale `working`/`started` file of an earlier worker with the same name fires "stale" at once
The start-time filter applies only to terminal states. A previous worker killed with SIGKILL leaves `working`; a new `worker-wait.sh worker-1` started before the new worker's SessionStart writes returns immediately with `stale (no update for 10800 s)`.
Checked: file `working …` aged 3 h, `--dir` pointed at it: returned at once, rc 0. The doc says "yesterday's files cannot fire it", which is true only for `stopped/blocked/failed/ended`.
Fix: apply the `m < start` skip to working/started too (a file older than the wait's start and older than the stale limit is not this worker's), or have the Coordinator delete the worker's file before starting it, and say so in the doc.

### P2-3 `bin/worker-status-hook.sh` + `.claude/settings.json` (PostToolUse `async: true`): a late PostToolUse can overwrite `stopped`
PostToolUse is async, Stop is synchronous. Both write the same file with last-writer-wins and no ordering. If the async hook for the last tool finishes after Stop's (slow start under load, or parallel tool calls), the file reads `working` after the turn ended; `worker-wait.sh` then waits until the stale limit (30 min).
Checked by feeding Stop then PostToolUse to the hook: the file ends `working`. The window is small in practice (the final model reply sits between the last tool and Stop), so P2, not P1.
Fix: in the hook, do not let `working` from PostToolUse replace a `stopped` line written after the event's own time (put an event timestamp in the input if available), or make PostToolUse update only when the current state is `working`/`started`; UserPromptSubmit and SessionStart still always write.

### P3-1 `bin/worker-status-hook.sh:13`: `input=$(cat)` is not covered by the 5 s `timeout`
If stdin never closes, the hook blocks until Claude Code's 10 s hook timeout (5 s for SessionEnd, 1.5 s default is overridden). Checked with stdin from `<(sleep 10)`: still running at 3 s. Claude supplies and closes stdin, so unlikely; fix is `input=$(timeout 2 cat)`. Not a session-blocker beyond the hook timeout; Stop/SessionStart wait for sync hooks, so a hang would delay them up to 10 s.

### P3-2 `bin/job.sh`: `--group light` runs any command with no `taskset`/`nice` and no one-at-a-time limit
Checked: a `light` job showed affinity `ffff` and niceness 0; two run in parallel. The `sim` limit holds for `sim` jobs (affinity `ff`, niceness 10, `CARGO_BUILD_JOBS=2`, cargo found on PATH), but nothing stops an agent putting a measurement run in `light`, nor using `pueue` directly. By design and documented as "docs checks, the Python oracle"; say in docs/job-queue.md that `light` is not for sim builds or runs (one line), since the laptop crashes when hot.

### P3-3 `bin/job.sh log` and `kill` with bad ids
`bin/job.sh log 9999` prints nothing and exits 0 (should say "no job" and exit 2 like `status`/`wait`). `kill` on a finished or unknown id prints pueue's "The command failed for tasks: N" and exits 1 (acceptable); `kill abc` prints pueue's usage and exits 2. Optional: validate the id is numeric in all subcommands (`wait abc` gives "no job abc" plus a blank line, exit 2).

### P3-4 `bin/job.sh:215`: several arguments are not run through a shell
`run -- FOO=1 env` fails (`exec: FOO=1: not found`, rc 127) because the line is `exec FOO=1 env`. Matches "several are a command and its arguments" but trips the natural `VAR=x cmd` form. Doc note or use `exec env -- ...`-free form (`bash -c`) for everything.

### P3-5 `bin/worker-wait.sh:65-66`: header comment is garbled
"are ignored unless --any-age (the stale check always applies). A "working" …" is one overlong, awkward sentence; `--since-start` is accepted but undocumented in the doc. Cosmetic.

## Answers to the ranked questions

1. Hook safety: PASS. With `BB_WORKER_NAME` unset it exits 0 in 4 ms (no git, no python). Also exit 0 and quiet for: run from the worktree, a subdirectory, the path with a space, outside a git repo, `PATH=/nonexistent` (no git/python/timeout), bad or empty JSON, odd names (`../../x` becomes `._.._x`, no escape from the directory), and a missing status directory (created). It uses python3, not `jq`, so a missing `jq` is irrelevant; a missing python3 or `timeout` just means no write. Only P3-1 can delay it.
2. `job.sh`: quoting is right (a command with spaces, single and double quotes, and the worktree path with a space all arrived intact; multi-argument form kept `a b` and `c"d`). `run` printed only the id. `wait` exits 0, 1 (failed or killed), 3 (by the code; I did not wait out a 540 s limit, the doc records a `--max 5` test), 2 for no such job. The daemon-down path was read, not run (restarting is forbidden): `ensure_daemon` starts it, retries 5 s, and dies with exit 2 and a message. The `sim` group's wrapping holds as above; P3-2 is the one way around it.
3. `worker-wait.sh`: cannot say "stopped" for a running job (it knows nothing about jobs; the docs say so). The killed-session gap is documented; a killed `working` worker returns "stale" after 30 min, a killed `stopped` one looks finished. P2-2 and P2-3 are the misreads.
4. Status path: the main checkout's `.claude/worker-status/` is written from a worktree (verified: the file appeared there, not in the worktree), and `git check-ignore -v` confirms the directory is ignored.
5. Docs: match the scripts (states, line format, exit codes 0/1/3/2, defaults 540 s and 1800 s) except the "yesterday's files" claim (P2-2) and the missing mention of `--since-start`.
6. Hygiene: `set -u` only in `job.sh`, none in the hook (it must not fail) or `worker-wait.sh`; no trap issues found. Writes use temp file plus `os.replace`, so a reader never sees a partial line; the only race is the ordering one (P2-3). Leftover `<name>.tmp.<pid>` files remain if the hook is killed mid-write (harmless).

## Verification

- `bash -n` on the three scripts: clean. `shellcheck` is not installed.
- Hook, fed sample JSON with `BB_WORKER_NAME=zz-rev`: Stop gave `stopped … bg=2`; an event with `agent_id` was ignored; a later PostToolUse overwrote it with `working` (P2-3); subdirectory and outside-repo and no-tools runs exited 0. Files removed afterwards (the status directory did not exist before and was removed).
- `worker-wait.sh` against a temp `--dir`: stale file fired at once (P2-2); an old `stopped` file was ignored until the timeout; trailing `--timeout` hung (P2-1).
- `job.sh` against pueue 4.0.5, `light` jobs 6–9 and 12 and one `sim` job 10, all trivial: results as in the answers above. Jobs 6–12 remain in the queue's list; I did not run `pueue clean` (agents may hold ids).

## Resolution

Open.
