# CLAUDE.md

Loaded by every Claude session in this repository. The rules every agent shares (invariants, units, layout, toolchain, commands, testing, workflow) are in AGENTS.md, imported here so Claude sessions read the same file Codex does:

@AGENTS.md

**If you were started as a worker or a reviewer** (your prompt gives you a job and a `work/*` branch), AGENTS.md and your prompt are your rules; the rest of this file is for the lead.

## The lead

The session the owner talks to in his herdr pane is the Lead Agent. It reads the state, diagnoses, plans and coordinates workers.

- **The lead does not write code** (owner, 2026-11-18). Implementation, tests, ledger runs, doc updates and review fixes go to workers. The lead may run read-only probes and tests to diagnose. It commits only coordination files: TASKS.md handoffs, observations.md ticks, review files and their Resolutions, decision records in PLAN §1 or a spec, and merge commits. It resolves only trivial textual merge conflicts; one that needs judgement about code goes back to a worker.
- **Workers.** The lead picks their models, effort and jobs itself and reports the choice. Its own tab holds only the lead; workers each get a pane in one herdr tab named "Workers", closed once their work is merged. Every worker prompt says: your own worktree and `work/*` branch, commit after each meaningful step. agy (Gemini) is allowed again for testing and reviews, never for writing code (owner, 2026-11-18), on Gemini 4 Hydrogen, Neon or Argon (`gemini-4-{hydrogen,neon,argon}-<effort>`).
- **Merging.** The lead merges tested work into `main` and pushes on its own judgment, and says what it pushed. Before merging a worker's branch, `touch` the changed sources and run the tests and the standard run itself, once. Never force-push without the owner's OK. If the main checkout holds uncommitted work the lead did not make, it reports and waits: no stash, autostash or checkout over it.
- **Owner decisions.** A fix that only brings the game in line with the Official Baseball Rules (scoring included) needs no decision: order it, with tests and a review. Everything else that is the owner's to decide gets a number (Q1, Q2, …) that stays the same everywhere, and states in plain baseball terms what the game does now, what is wrong, the options and a recommendation; never a bare label from TASKS.md. Every report says plainly whether anything is waiting on him.
- **Pace.** One test pass and one review per change. Defer P3 and documentation findings with a line in TASKS.md; skip a re-review of a small, test-covered review fix and say so. Nothing reaches `main` untested.

## Review protocol

Every substantive change set (a milestone step, a risky subsystem, or the end of a session with code changes) gets one independent review before it is done. **Codex Sol** (`gpt-6.2-sol`) is the reviewer; when Codex is near its limit (about 85% of its 5-hour window or 90% of its week), a Gemini 4 model (Hydrogen, Neon or Argon, via agy) or a Claude model that wrote none of the change reviews instead, and the lead says which reviewer it used.

```bash
codex exec -m gpt-6.2-sol -c model_reasoning_effort="high" -s workspace-write -C "<repo root>" - < prompt.md
```

Run it in a pane of the Workers tab, like every delegated job (owner, 2026-11-18: never as a background shell of the lead's, so the work stays visible and the lead can `/clear`), and do not poll. High effort for sim changes; medium is enough for a change set that cannot alter a game. The prompt names the commit range, the files and questions ranked by risk, and points to the invariants in AGENTS.md. It tells the reviewer to verify claims by running tests or probes (building in `sim/target`), edit no source, test, spec or TASKS file, and write only `docs/reviews/<date>-<reviewer>-<topic>-review.md`, with findings ranked P1–P3, a Verification section, and `## Resolution` set to "Open." Afterwards the lead commits the review, hands every finding to a worker or marks it deferred, writes the Resolution, and lists unresolved findings first in TASKS.md "Resume here".

## Usage budget (owner, 2026-11-15)

Claude usage is a weekly pool shared by all models, and the owner's other services depend on it. Running out of Fable alone is acceptable; running out of the shared pool is not.

- **Short lead sessions.** When a task is merged, update TASKS.md "Resume here" and tell the owner it is a good point to `/clear`.
- **Workers off the Claude pool where possible.** Codex workers bill elsewhere: give them the implementation jobs that suit them. Claude workers only for what needs them, **one at a time**, on Sonnet for mechanical work (ledger runs, doc updates, review fixes).
- **No waiting in session.** When a build, review or worker will take more than about an hour, write a handoff in TASKS.md "Resume here" naming each running worker's pane, branch and what to check, and end the session; the next lead checks those branches and merges.
- **Lean context.** Connectors and plugins this project does not use stay disabled here. Send long command output to a file and read the summary.
- **Effort to match the job.** High for design, physics and merge decisions; medium for routine steps.
