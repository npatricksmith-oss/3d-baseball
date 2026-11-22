# Job queue for long runs

Owner decision Q34 (2026-11-22). Builds of more than a few minutes, full test runs and measurement runs go through a queue that owns the process. The agent that asked for the run holds only a job id.

Why: an agent that starts a long run as its own background shell and idles looks, from outside, the same as one that has finished or died, and when the agent is killed its run dies with it (both happened on 2026-11-20 and 2026-11-22). With the queue, anyone can read a run's state, exit code and log, and the run outlives the agent. The `sim` group runs one job at a time, so two measurement runs can never overlap, whichever agents submit them.

The queue is [pueue](https://github.com/Nukesor/pueue) 4.0.5; agents use only the wrapper `bin/job.sh`. The worker's own turn (stopped, blocked, ended) is reported separately: [worker-status-hooks.md](worker-status-hooks.md).

## Use

```bash
id=$(bin/job.sh run -- 'cd sim && cargo test --workspace 2>&1')   # prints the job id
bin/job.sh wait "$id"            # blocks up to 540 s; exit 0 succeeded, 1 failed or killed, 3 still going
bin/job.sh status                # every job, one line each
bin/job.sh log "$id" --tail 50
bin/job.sh kill "$id"
```

- Run `bin/job.sh run` from the directory the command should start in (your worktree). One argument after `--` is a shell line (`'a && b'`); several are a command and its arguments.
- `--group sim` (the default): one job at a time, wrapped in `taskset -c 0-7 nice` with `CARGO_BUILD_JOBS=2` and `~/.cargo/bin` on the path. Do not add those yourself. `--group light`: two at a time, no wrapping; for docs checks and the Python oracle.
- `--label` says whose run it is and what; the default is `$BB_WORKER_NAME` and the branch.
- `wait` returns 3 at its limit because a foreground command in Claude Code is capped at 10 minutes: call it again. Never end your turn to wait; never start the run in a background shell of your own.
- A status line reads `3 queued rc=- group=sim start=- run=- label=…  queued behind 2 <label>`: you are waiting on someone else's run.
- A job's environment is the submitter's at the time of `run`.

For the Coordinator: `bin/job.sh status` is the truth about a run. A worker whose turn has stopped while its job is running or queued is waiting, not done.

## Set-up on this machine

- `~/.local/bin/pueue` and `~/.local/bin/pueued`: the release's prebuilt `x86_64-unknown-linux-musl` binaries, SHA-256 checked against the release's digests.
- `~/.config/systemd/user/pueued.service` (enabled, `systemctl --user`), so the daemon starts at login and does not belong to any terminal or Claude session. `bin/job.sh` starts it if it is not running and creates the groups if they are missing.
- State and logs: `~/.local/share/pueue/` (`task_logs/<id>.log`). Config: `~/.config/pueue/pueue.yml` (defaults).
- To remove: `systemctl --user disable --now pueued`, then delete those two binaries, the unit file and the two pueue directories.

## Tested (2026-11-22)

Seen happening, with light jobs:

- success and failure: `python3 -m unittest analysis/oracle/test_oracle.py` gave `succeeded rc=0`; `'echo to-fail; exit 7'` gave `failed rc=7` and `wait` exit 1, each with the log's tail;
- two `sim` jobs submitted together ran one after the other, the second showing `queued behind 2 first run`;
- `wait --max 5` on a 25 s job returned 3, and a later wait returned the result;
- the `sim` wrapping: affinity mask `ff` (CPUs 0–7), niceness 10, `CARGO_BUILD_JOBS=2`, and the job's directory was the worktree (a path with a space);
- a job submitted from a detached shell that was then killed with `kill -9` still ran and finished with its log;
- `kill` gave `killed` and `wait` exit 1; an unknown id gives exit 2.

## Gaps

- **Stopping or restarting the daemon kills the running job** (seen: `systemctl --user restart pueued` left the job `killed`) and pueue then pauses that group. `bin/job.sh` resumes a paused group on its next call and says so; queued jobs survive. Do not restart the daemon while a job runs. A reboot or a crash does the same.
- A job that hangs holds the `sim` queue for ever: nothing times it out. The Coordinator's periodic look (`bin/job.sh status`, the `run=` time) is the check.
- `pueue clean` removes finished jobs and ids start again from 0 once the list is empty, so clean only when no agent holds an id.
- Logs are never pruned by themselves (`pueue clean` deletes the logs of finished jobs).
- pueue stores each job's full environment in `~/.local/share/pueue/state.json`.
