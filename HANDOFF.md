# Handoff, 2026-11-22 morning

The detail behind TASKS.md "Resume here". Delete this file when the three questions are answered and both branches are merged.

**State.** Nothing is running: no workers, no jobs in the queue. The owner is starting herdr; run workers there as their own sessions (panes), not as Agent-tool subagents: an interrupted Coordinator turn (Esc, or a message mid-turn) killed subagent workers three times this morning (claude-code issue #78151).

**Waiting on the owner (asked 2026-11-22, not answered):**
1. Merge `work/worker-hooks` into `main`? Recommended yes.
2. Q33: replace CLAUDE.md's two watch rules ("Watch every worker you start", "Know each worker's real state") with: workers as their own herdr sessions started with `BB_WORKER_NAME=<name>`; the Coordinator waits with bin/worker-wait.sh; long runs live in the job queue; a look at the branch, worktree and the queue's status about every 30 minutes and on every notice; restart-without-asking and the warning before `/reload-plugins` kept, plus a warning that interrupting a turn kills subagent workers. Recommended yes; the Coordinator writes the exact text for him to read first.
3. Restart the relay worker (below) in a herdr pane. Recommended yes.

**`work/worker-hooks`** (worktree `.claude/worktrees/worker-hooks`, `ae461ab`, unmerged, no review: tooling that cannot alter a game).
- Status hooks (Sonnet worker): .claude/settings.json, bin/worker-status-hook.sh, bin/worker-wait.sh, docs/worker-status-hooks.md. A session started with `BB_WORKER_NAME` writes `started/working/blocked/stopped/failed/ended` to `.claude/worker-status/<name>` in the main checkout; `stopped bg=N` counts its own background jobs. Six Haiku test sessions. Gaps: a killed session leaves its last line; no file for subagent workers.
- Job queue, owner Q34 (written by the Coordinator on the owner's order "Stop all workers. Do q34"): bin/job.sh (`run`, `wait`, `status`, `log`, `kill`) over pueue 4.0.5, docs/job-queue.md, AGENTS.md rule marked "on trial". Installed on the host, outside git: `~/.local/bin/pueue`, `~/.local/bin/pueued`, `~/.config/systemd/user/pueued.service` (enabled). Group `sim`: one job at a time under `taskset -c 0-7 nice`, `CARGO_BUILD_JOBS=2`. Gaps: restarting the daemon kills the running job; nothing times out a hung job.
- Until it is merged, other worktrees have no bin/job.sh: call it by its path in the `worker-hooks` worktree from the directory the run should start in.

**`work/relay-handovers`** (worktree `.claude/worktrees/runner-decision`, `ea66e2b`, unmerged; job file WORKER-RELAY-HANDOVERS.md there; §3 "For the relay review").
- Committed: items 1, 2, 3, 5, 6 and a relay-base fix (`git log 7622093..ea66e2b`). `cargo test --workspace` on `ea66e2b` through the queue, 09:15–09:26: 523 passed, 0 failed.
- Uncommitted in the worktree, written by a worker killed at 09:07 before its last measurements: docs/calibration/2026-11-22-relay-handovers.md (17 KB) and a `TASKS.md` edit (will conflict with this handoff; take this section and the worker's §3 changes). Check every figure against `sim/target/codex-probes/handover/` (`hc-*.txt`, `tr-*.log`, `std*-*.txt`) before committing.
- Missing: the standard run (in that probe directory `std2-prev-0.txt` and `std2-final-0.txt` are complete, `std2-prev-1000.txt` is empty, binary `sim-cli-final2`; `std-*` is an earlier set, check which build), the ledger row, fixture README entries, item 4's owner question (double cut on a ball past the outfielders: does the corner man stay out as the cutoff man home behind the middle infielder's relay, and which corner man on a ball to left), then an Opus review.
- To continue: an Opus worker (high) in a herdr pane on that worktree, long runs through the job queue; its prompt is the one the Coordinator used on 2026-11-22 ("replacing a worker that was killed": read the job file, STOPPED-REPORT.md in the probe directory, the log since `40102f7`, commit the note first).

**Open (ledger §5):** runs fell 0.21 [0.18] with dp-cover, about 0.04 explained: check noise on more seeds before tracing. Double plays 12.3% per opportunity (MLB 9.8%). Standard runs on main 4.05 [4.39].

**After that:** play 71 (§2: no dive at a ground ball 1.9 m away).

## Update, 2026-11-22 midday (owner's answers)

- **Merge `work/worker-hooks`: yes**, after a Sonnet review (`hooks-review`, review file `docs/reviews/2026-11-22-sonnet-worker-hooks-review.md` on that branch).
- **Q33: yes**, wording approved as shown to him: one bullet "Workers are their own sessions, and the Coordinator knows each one's real state" replacing CLAUDE.md's "Watch every worker you start" and "Know each worker's real state" (own herdr sessions with `BB_WORKER_NAME`, never subagents; `bin/worker-wait.sh`; long runs in the job queue, `bin/job.sh status` is the truth; a look every 30 minutes and on every notice; restart without asking; warn before `/reload-plugins`, `/clear` or an interrupt while a subagent runs). Put it in CLAUDE.md with the hooks merge.
- **Q35 (item 4 of the relay job): (c), the textbook double cut.** On a deep ball with a play at home a middle infielder goes out as the relay man and the first baseman lines up behind him as the cutoff man home, whichever field the ball is in; the third baseman stays at third, the other middle infielder covers second. Its own job after `work/relay-handovers` merges: spec agent-models §5.7 first, then code, ledger row, Opus review (relay note §4, its calibration note of 2026-11-22 on the branch).
- Relay worker finished: `work/relay-handovers` at `c5eb654`, 523 tests, standard run 4.05 → 4.05 [4.39 → 4.33]. Opus review running (`relay-review`), its review file will be under docs/reviews on the branch.
- **Evening:** `work/worker-hooks` merged (`8ab3308`) with the review's eight fixes (`edd4cd9`) and the Q33 text in CLAUDE.md. Everything above this update about that branch and its three questions is done; what remains in this file is the relay branch and Q35.
