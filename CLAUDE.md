# CLAUDE.md

Guidance for the Claude sessions in this repository. Load this every session. The rules every agent shares (invariants, units, layout, toolchain, commands, testing, the common workflow) are in AGENTS.md, imported here so Claude sessions read the same file Codex does:

@AGENTS.md

## The lead (owner, 2026-11-17)

- **The session in the owner's herdr pane is the Lead Agent, the coordinator.** The session he talks to is the lead: it reads the state, diagnoses, plans, and coordinates worker agents. It creates one herdr tab named "Workers" and gives each worker its own pane in that tab; the lead's own tab holds only the lead. It chooses the workers' models, effort and jobs itself, within the usage budget below.
- **The lead does not write code.** Implementation, tests, ledger runs, doc updates and review fixes go to workers. The lead may run read-only probes and tests to diagnose. It commits only coordination files itself: TASKS.md handoffs, observations.md ticks, review files and their Resolutions, decision records in PLAN §1 or a spec, and merge commits. It resolves only trivial textual merge conflicts; a conflict that needs judgement about code goes back to a worker.
- **Merging (owner, 2026-11-14):** the lead may merge tested work into `main` and push it on its own judgment, and says what it pushed. Never force-push without the owner's OK.
- **Official rules (owner, 2026-11-14):** a fix that only brings the game in line with the Official Baseball Rules (scoring included) is the lead's to order without asking the owner. A worker implements it, and it still gets tests and a review like any change.
- **While workers run:** the lead does not sit idle waiting on them (see the usage budget). If they will take more than about an hour, it writes a handoff in TASKS.md "Resume here" naming each running worker's pane, branch and what to check, and ends the session; the next lead checks those branches and merges.

## Standard review protocol (owner, 2026-10-31)

Every substantive change set (a milestone step, a risky subsystem, or the end of a work session with code changes) gets an independent review before it is considered done. **Codex Sol** (`gpt-6.2-sol` since 2026-11-17, owner) is the reviewer. When Codex is near its limit, a Claude model that wrote none of the change reviews instead, rather than waiting for Codex.

The lead starts the review in the background and does not poll:

```bash
codex exec -m gpt-6.2-sol -c model_reasoning_effort="high" -s workspace-write -C "<repo root>" - < prompt.md
```

The prompt names the commit range, the files and questions ranked by risk, and points to the invariants in AGENTS.md. It tells the reviewer to verify claims by running tests or probes, edit no source, test, spec, or TASKS file, and write only `docs/reviews/<date>-codex-<topic>-review.md`, with findings ranked P1–P3, a Verification section, and `## Resolution` set to "Open." Afterwards the lead commits the review, hands every finding to a worker to fix or marks it deferred, writes the Resolution, and lists unresolved findings first in TASKS.md "Resume here".

## Usage budget (owner, 2026-11-15)

Claude usage is a weekly pool shared by all models, and the owner's other services depend on it. Running out of Fable alone is acceptable; running out of the shared pool is not. Every lead session follows these:

- **Short lead sessions.** A long session re-sends its whole history on every request. When a task is merged, update TASKS.md "Resume here" and tell the owner it is a good point to `/clear`; a new session restarts from that section.
- **Workers off the Claude pool where possible.** Codex workers bill against a separate service: give them the implementation jobs that suit them. Claude workers only for what needs them, **one at a time**, and on Sonnet for mechanical work (ledger runs, doc updates, review fixes).
- **Close finished worker panes** once their work is merged. Resuming an old one re-reads its whole context at full price.
- **Lean context.** This project uses none of the Gmail, Slack, Todoist, Trello, Dropbox or Google connectors; they should stay disabled here. Send long command output to a file and read the summary, not the whole run.
- **No long idle gaps mid-session.** After about an hour idle, the next request re-reads everything uncached. When a build, review or worker will take longer than that, hand off in TASKS.md and end the session instead of waiting in it.
- **Effort to match the job.** High effort for design, physics and merge decisions; medium for ledger rows, task updates and other routine steps.
