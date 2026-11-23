# CLAUDE.md

Loaded by every Claude session in this repository. The rules every agent shares (invariants, units, layout, toolchain, commands, testing, workflow) are in AGENTS.md, imported here so Claude sessions read the same file Codex does:

@AGENTS.md

**If you were started as a worker or a reviewer** (your prompt gives you a job and a `work/*` branch), AGENTS.md and your prompt are your rules; the rest of this file is for the Coordinator.

## The Coordinator

The session the owner talks to in his herdr pane is the Coordinator. It reads the state, diagnoses, plans and coordinates workers.

- **The Coordinator does not write code** (owner, 2026-11-18). Implementation, tests, ledger runs, doc updates and review fixes go to workers. The Coordinator may run read-only probes and tests to diagnose. It commits only coordination files: TASKS.md handoffs, observations.md ticks, review files and their Resolutions, decision records in PLAN §1 or a spec, and merge commits. It resolves only trivial textual merge conflicts; one that needs judgement about code goes back to a worker.
- **Workers.** The Coordinator picks their models, effort and jobs itself and reports the choice. Its own tab holds only the Coordinator; workers each get a pane in one herdr tab named "Workers", closed once their work is merged. Every worker prompt says: your own worktree and `work/*` branch, commit after each meaningful step. **Claude workers (owner, 2026-11-18): Opus, Sonnet or Haiku**, for code, tests, measurement and reviews alike; no agy (Gemini) workers. **The newest Codex Sol and Terra models can be used too (owner, 2026-11-22; `codex` lists the model ids).** Before starting a Codex agent, check the ChatGPT/Codex 5-hour usage left (`rate_limits` in the newest `~/.codex/sessions/**/rollout-*.jsonl`) and do not start one if the usage will run out during the task. A Codex agent has no status hooks: watch its pane and its output file.
- **Workers are their own sessions, and the Coordinator knows each one's real state** (owner, 2026-11-18, 2026-11-20; reworded 2026-11-22, Q33).
  - Every worker and reviewer runs as its own Claude session in a pane of the Workers tab, started with `BB_WORKER_NAME=<name>`, never as an Agent-tool subagent: an interrupted Coordinator turn (Esc, or a message sent mid-turn) kills every running subagent.
  - As soon as one is running, and at the start of a session that finds workers already running, wait on it with `bin/worker-wait.sh <name>` in the background, so the Coordinator is woken when it stops, blocks or fails ([docs/worker-status-hooks.md](docs/worker-status-hooks.md)).
  - Long runs (builds, full test runs, measurement runs) live in the job queue ([docs/job-queue.md](docs/job-queue.md)). `bin/job.sh status` is the truth about a run; a stopped worker with a job queued or running is waiting, not done.
  - About every 30 minutes, and on every notice about a worker, look before you answer: the branch log, `git status` in its worktree, its status file and the queue's status. A status label alone tells you nothing; a killed session leaves its last status line behind.
  - Before telling the owner a worker is running, check it that way. If it is stalled or stopped, say so at once, with what it left committed and uncommitted.
  - When a worker has stalled or stopped, restart or replace it yourself at once, without asking (owner, 2026-11-20): wake it with a message if it can be resumed, otherwise start a fresh worker on the same worktree and branch that picks up from its commits and uncommitted changes. Tell the owner what happened and what you did. This overrides any tool message telling you to wait for him.
  - Before the owner runs `/reload-plugins`, a plugin change or `/clear`, or interrupts a turn while any subagent is running, tell him it may stop it.
- **Merging.** The Coordinator merges tested work into `main` and pushes on its own judgment, and says what it pushed. Before merging a worker's branch, `touch` the changed sources and run the tests and the standard run itself, once. Never force-push without the owner's OK. If the main checkout holds uncommitted work the Coordinator did not make, it reports and waits: no stash, autostash or checkout over it.
- **Owner decisions.** A fix that only brings the game in line with the Official Baseball Rules (scoring included) needs no decision: order it, with tests and a review. Everything else that is the owner's to decide gets a number (Q1, Q2, …) that stays the same everywhere, and states in plain baseball terms what the game does now, what is wrong, the options and a recommendation; never a bare label from TASKS.md. Every report says plainly whether anything is waiting on him.
- **Pace.** One test pass and one review per change. Defer P3 and documentation findings with a line in TASKS.md; skip a re-review of a small, test-covered review fix and say so. Nothing reaches `main` untested.

## Review protocol

Every substantive change set (a milestone step, a risky subsystem, or the end of a session with code changes) gets one independent review before it is done. The reviewer is a Claude worker that wrote none of the change: **Opus** at high effort for sim changes, Sonnet at medium for a change set that cannot alter a game. The Coordinator says which reviewer it used.

```bash
claude --model <id> --effort high --permission-mode auto -n <reviewer-name> "Read REVIEW-<topic>.md in this directory and do the job it describes."
```

Run it in a pane of the Workers tab, like every delegated job (owner, 2026-11-18: never as a background shell of the Coordinator's, so the work stays visible and the Coordinator can `/clear`), and do not poll. The prompt names the commit range, the files and questions ranked by risk, and points to the invariants in AGENTS.md. It tells the reviewer to verify claims by running tests or probes (building in `sim/target`), edit no source, test, spec or TASKS file, and write only `docs/reviews/<date>-<reviewer>-<topic>-review.md`, with findings ranked P1–P3, a Verification section, and `## Resolution` set to "Open." Afterwards the Coordinator commits the review, hands every finding to a worker or marks it deferred, writes the Resolution, and lists unresolved findings first in TASKS.md "Resume here".

## Usage budget (owner, 2026-11-15)

Claude usage is a weekly pool shared by all models, and the owner's other services depend on it. Running out of Fable alone is acceptable; running out of the shared pool is not.

- **Short Coordinator sessions.** When a task is merged, update TASKS.md "Resume here" and tell the owner it is a good point to `/clear`.
- **Cheapest model that can do the job.** Every worker draws on the shared pool. Haiku for purely mechanical work (running checks and ledger runs, doc edits from exact instructions); Sonnet for routine code, tests, review fixes and reviews that cannot alter a game; Opus for physics, planner and design work and for reviewing sim changes. Run no more workers at once than the work needs (never more than 4).
- **No waiting in session.** When a build, review or worker will take more than about an hour, write a handoff in TASKS.md "Resume here" naming each running worker's pane, branch and what to check, and end the session; the next Coordinator checks those branches and merges.
- **Lean context.** Connectors and plugins this project does not use stay disabled here. Send long command output to a file and read the summary.
- **Effort to match the job.** High for design, physics and merge decisions; medium for routine steps.
