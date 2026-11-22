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
  input=$(timeout 2 cat 2>/dev/null)
  BB_DIR="$dir" BB_NAME="$name" BB_PPID="$PPID" BB_IN="$input" BB_WORKER_TRACE="${BB_WORKER_TRACE:-}" timeout 5 python3 -I - <<'PY'
import json, os, time, subprocess
try:
    d = json.loads(os.environ.get("BB_IN") or "")
    if not isinstance(d, dict): d = {}
except Exception:
    d = {}
tr = os.environ.get("BB_WORKER_TRACE")   # debugging: append every raw event to this file
if tr:
    with open(tr, "a") as f: f.write(time.strftime("%H:%M:%S ") + (os.environ.get("BB_IN") or "")[:600].replace("\n", " ") + "\n")
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
# background shell jobs / tasks still running: the Stop input lists them
bt = d.get("background_tasks")
bg = len(bt) if isinstance(bt, list) else 0
line = "%s %s %s bg=%d why=%s %s\n" % (state, time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        clean(str(d.get("session_id", "-"))), bg, clean(why), str(d.get("cwd", os.getcwd())).replace("\n", " "))
path = os.path.join(os.environ["BB_DIR"], os.environ["BB_NAME"])
if ev == "PostToolUse":
    # async: a late one must not overwrite stopped/blocked/ended; refresh only a live state
    try:
        with open(path) as f: cur = f.readline().split(" ", 1)[0]
    except Exception:
        cur = ""
    if cur not in ("working", "started"):
        raise SystemExit(0)
tmp = "%s.tmp.%d" % (path, os.getpid())
with open(tmp, "w") as f: f.write(line)
os.replace(tmp, path)
PY
} >/dev/null 2>&1
exit 0
