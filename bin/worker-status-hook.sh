#!/usr/bin/env bash
# Claude Code hook: record this session's state in <main repo>/.claude/worker-status/<name>.
# Does nothing unless BB_WORKER_NAME is set. Never fails a session (always exit 0).
# See docs/worker-status-hooks.md. Event JSON arrives on stdin.
# Line format:  <state> <ISO UTC time> <session id> bg=<n> why=<token> <cwd, rest of line>
{
  [ -n "${BB_WORKER_NAME:-}" ] || exit 0
  name=$(printf '%s' "$BB_WORKER_NAME" | tr -c 'A-Za-z0-9._-' '_')
  name=${name#.}; name=${name:-worker}
  common=$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || exit 0
  dir="$(dirname "$common")/.claude/worker-status"
  mkdir -p "$dir" 2>/dev/null || exit 0
  input=$(cat 2>/dev/null)
  BB_DIR="$dir" BB_NAME="$name" BB_PPID="$PPID" BB_IN="$input" timeout 5 python3 -I - <<'PY'
import json, os, time, subprocess
try:
    d = json.loads(os.environ.get("BB_IN") or "")
    if not isinstance(d, dict): d = {}
except Exception:
    d = {}
ev = str(d.get("hook_event_name", ""))
nt = str(d.get("notification_type", ""))
why = ""
state = {"SessionStart": "started", "UserPromptSubmit": "working", "PostToolUse": "working",
         "Stop": "stopped", "SessionEnd": "ended", "PermissionRequest": "blocked",
         "StopFailure": "failed"}.get(ev)
if ev == "Notification":
    if nt in ("permission_prompt", "elicitation_dialog", "elicitation_url_dialog", "agent_needs_input"):
        state, why = "blocked", nt
    else:
        state = None            # idle_prompt etc.: no change
elif ev == "PermissionRequest":
    why = str(d.get("tool_name", ""))
elif ev == "StopFailure":
    why = str(d.get("error_type") or d.get("error") or "")
elif ev == "SessionEnd":
    why = str(d.get("reason", ""))
elif ev == "SessionStart":
    why = str(d.get("source", ""))
if state is None or d.get("agent_id"):
    # events fired inside a subagent do not describe the worker session
    if not (ev == "PermissionRequest" and state):
        raise SystemExit(0)
def clean(s): return "".join(c if c.isalnum() or c in "._-:" else "_" for c in s)[:60] or "-"
# background jobs: descendants of the claude process, excluding this hook's own chain
bg = 0
try:
    rows = {}
    for l in subprocess.run(["ps", "-eo", "pid=,ppid=,comm="], capture_output=True, text=True, timeout=2).stdout.splitlines():
        p = l.split(None, 2)
        if len(p) == 3: rows[int(p[0])] = (int(p[1]), p[2])
    me = os.getpid(); chain = {me}; x = me
    while x in rows and rows[x][0] > 1:
        x = rows[x][0]; chain.add(x)
        if rows[x][1].startswith(("claude", "node")): break
    root = x
    kids = {}
    for pid, (pp, c) in rows.items(): kids.setdefault(pp, []).append(pid)
    stack, seen = [root], set()
    while stack:
        for k in kids.get(stack.pop(), []):
            if k in chain or k in seen: continue
            seen.add(k); stack.append(k)
            if rows[k][1] not in ("ps", "sleep") or True: bg += 1
except Exception:
    pass
line = "%s %s %s bg=%d why=%s %s\n" % (state, time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        clean(str(d.get("session_id", "-"))), bg, clean(why), str(d.get("cwd", os.getcwd())).replace("\n", " "))
path = os.path.join(os.environ["BB_DIR"], os.environ["BB_NAME"])
tmp = "%s.tmp.%d" % (path, os.getpid())
with open(tmp, "w") as f: f.write(line)
os.replace(tmp, path)
PY
} >/dev/null 2>&1
exit 0
