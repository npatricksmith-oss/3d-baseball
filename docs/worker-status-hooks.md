# Worker status hooks

Each worker session reports its own state through Claude Code hooks, so the Coordinator waits on one file per worker instead of guessing from pane labels, file times or process names. Written against Claude Code 2.1.330.

## The files

`<main repo>/.claude/worker-status/<name>`, one line, replaced atomically (temp file, then `mv`). Git-ignored (`/.claude/worker-status/` in `.gitignore`). Worktrees find the main repo with `git rev-parse --git-common-dir`, so every worker writes to the same directory.

```text
<state> <ISO UTC time> <session id> bg=<n> why=<token> <cwd, rest of line>
```

| State | Written on | Meaning |
| :- | :- | :- |
| `started` | SessionStart | session is up, no prompt yet (`why` = startup/resume/clear/...) |
| `working` | UserPromptSubmit, PostToolUse | a prompt was submitted or a tool just finished; PostToolUse also refreshes the file time as a heartbeat |
| `blocked` | PermissionRequest, Notification (`permission_prompt`, `elicitation_dialog`, `elicitation_url_dialog`, `agent_needs_input`) | a person has to answer (`why` = tool name or notification type) |
| `stopped` | Stop | the turn finished; `bg=<n>` is the number of the session's own background tasks still running (from the Stop input's `background_tasks`). `bg>0` means its own shell or task is still going: look before treating it as done |
| `failed` | StopFailure | the turn ended on an API error (`why` = error type) |
| `ended` | SessionEnd | session exited (`why` = reason, e.g. `prompt_input_exit`, `other` in `-p` mode) |

`idle_prompt` notifications (about 60 s after a turn) change nothing. Hooks that fire inside a subagent (input has `agent_id`) are ignored, except a PermissionRequest.

## Starting a worker

The hooks live in the checked-in `.claude/settings.json`, which a session reads from its own working directory, so every worktree made from a commit containing it has them. The script does nothing unless `BB_WORKER_NAME` is set (`[A-Za-z0-9._-]`, anything else becomes `_`), so the Coordinator's and the owner's sessions are unaffected.

```bash
BB_WORKER_NAME=worker-1 CARGO_BUILD_JOBS=2 claude --model <id> --effort <e> --permission-mode auto -n worker-1 "<prompt>"
```

A worker branch made before this change was merged has no `.claude/settings.json`; merge main into it or pass `--settings "$PWD/.claude/settings.json"` (not tested).

## Waiting

```bash
bin/worker-wait.sh worker-1 worker-2 [--timeout S] [--stale S] [--any-age]
```

Blocks until one named worker is `stopped`, `blocked`, `failed` or `ended`, then prints `<name>: <why returned>: <status line>` and exits 0. Exit 3 on timeout, 2 on bad usage. Polls every 5 s (`inotifywait` is not installed here). Terminal lines whose file is older than the script's own start are ignored (`--any-age` turns that off), so yesterday's files cannot fire it. A `working` or `started` file untouched for `--stale` seconds (default 1800) returns `stale`: a worker that died, or one inside a single tool call longer than that (a long build). Either way the Coordinator looks at the branch log, `git status` and the process list itself. Run it with `run_in_background` to be woken.

`BB_WORKER_TRACE=<file>` makes the hook append every raw event to a file, for debugging.

## What was tested (haiku, 6 live sessions)

Seen happen:
- `-p` session: `started`, `working`, `stopped`, `ended` all written; the wait script returned on it.
- Interactive tmux session, default permission mode: a shell command raised PermissionRequest at once and a `permission_prompt` Notification about 6 s later (both give `blocked`); after approval PostToolUse then Stop gave `stopped`.
- Background shell (`run_in_background`, `sleep 150`) then Stop: `stopped bg=1`; the idle_prompt notification arrived about 60 s later and was ignored. `/exit` gave `ended why=prompt_input_exit`.
- `--permission-mode auto`: no PermissionRequest or Notification, the file went `working` to `stopped`; `blocked` never appears in auto mode.
- A session without `BB_WORKER_NAME` wrote nothing. Hand-fed input: bad JSON, empty stdin, a name with spaces and `/`, outside a git repo: always exit 0, name sanitised, no crash.
- `kill -9` of an interactive session: the file stays at its last state (`stopped`), no `ended`.
- Worktree sessions loaded the worktree's own `.claude/settings.json` with no trust prompt (the project was already trusted).

Only read in the docs: settings are read from the session's primary working directory; StopFailure matcher values; SessionEnd default timeout 1.5 s (the script is fast, set to 5 s in settings); `PermissionDenied` exists for auto mode and is not used.

## Known gaps

- SIGKILL, power loss or a crash fires no hook: the last state stays. A killed `working` worker is caught only by the stale limit; a killed `stopped` worker looks finished. The timed check-in covers this.
- A very long single tool call (no PostToolUse) can reach the stale limit while healthy.
- `stopped` says the turn ended, not that the job is done: the worker may have asked a question. Read its output.
- The first start in a new, untrusted folder shows the trust dialog before hooks run (not hit here).
- `-p` mode SessionEnd arrives in the same second as Stop; a 5 s poll sees only `ended`.

## Subagent workers (no herdr)

Tested with a headless parent and one Agent-tool subagent, hooks registered via `--settings`: SubagentStart and SubagentStop fire in the parent with `agent_id` (unique, e.g. `a1f3a76bc120c36fa`), `agent_type` (the agent kind, `general-purpose`, not the description or a name), `agent_transcript_path`, `last_assistant_message` and `background_tasks`. Nothing carries a per-subagent name. The same status file could be written as `sub-<agent_id>` by registering SubagentStart (`working`) and SubagentStop (`stopped`) in the parent's settings and having the Coordinator note the agent id the Agent call returns; this needs a hook that does not require `BB_WORKER_NAME` and so is not built. A subagent killed by the harness fires no SubagentStop.
