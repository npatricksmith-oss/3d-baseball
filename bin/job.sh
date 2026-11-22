#!/usr/bin/env bash
# Job queue for long builds, test runs and measurement runs (owner Q34, 2026-11-22).
# The queue (pueue) owns the process, so a run outlives the agent that started it and
# anyone can read its state, exit code and log. See docs/job-queue.md.
#
#   bin/job.sh run [--group sim|light] [--label TEXT] -- COMMAND [ARGS...]   prints the job id
#   bin/job.sh wait ID [--max SECONDS]     0 succeeded, 1 failed or killed, 3 still going at the limit
#   bin/job.sh status [ID]                 one line per job
#   bin/job.sh log ID [--tail N]
#   bin/job.sh kill ID
#
# Group "sim" runs one job at a time under `taskset -c 0-7 nice` with CARGO_BUILD_JOBS=2;
# group "light" (docs checks, the Python oracle) runs two at a time with no wrapping.
set -u
PATH="$HOME/.local/bin:$PATH"

die() { echo "job.sh: $*" >&2; exit 2; }
need() { [ $# -ge 2 ] || die "$1 needs a value"; }                 # need OPT VALUE...
num() { case $2 in ''|*[!0-9]*) die "$1 needs a number, got '$2'" ;; esac; }   # num WHAT VALUE

ensure_daemon() {
  command -v pueue >/dev/null || die "pueue is not installed (docs/job-queue.md)"
  if ! pueue status >/dev/null 2>&1; then
    systemctl --user start pueued.service >/dev/null 2>&1 || pueued -d >/dev/null 2>&1
    for _ in 1 2 3 4 5 6 7 8 9 10; do pueue status >/dev/null 2>&1 && break; sleep 0.5; done
    pueue status >/dev/null 2>&1 || die "cannot reach or start the pueue daemon"
  fi
  local groups; groups=$(pueue group 2>/dev/null)
  case "$groups" in *'"sim"'*) ;; *) pueue group add sim >/dev/null; pueue parallel 1 -g sim >/dev/null ;; esac
  case "$groups" in *'"light"'*) ;; *) pueue group add light >/dev/null; pueue parallel 2 -g light >/dev/null ;; esac
  # pueue pauses a group whose running job was killed by a daemon stop; a paused queue would
  # stall every agent silently, so resume it and say so.
  groups=$(pueue group 2>/dev/null)
  local g
  for g in sim light; do
    case "$groups" in *"\"$g\" ("*"): paused"*)
      if printf '%s\n' "$groups" | grep -q "\"$g\".*: paused"; then
        echo "job.sh: group $g was paused (daemon restarted while a job ran?); resuming it" >&2
        pueue start -g "$g" >/dev/null
      fi ;;
    esac
  done
}

# report MODE [ID] [TAIL]: mode is "status" or "wait"; prints lines, exit code 0/1/3 (2 = no such job).
report() {
  pueue status --json | JOB_MODE="$1" JOB_ID="${2:-}" python3 -I -c '
import json, os, sys, datetime
d = json.load(sys.stdin)
tasks = d.get("tasks", {})
mode, want = os.environ["JOB_MODE"], os.environ["JOB_ID"]
def ts(s):
    try: return datetime.datetime.fromisoformat(s[:19])
    except Exception: return None
def info(t):
    st = t["status"]
    name, body = (st, {}) if isinstance(st, str) else next(iter(st.items()))
    body = body or {}
    rc, state = "-", name.lower()
    if name == "Done":
        res = body.get("result")
        if res == "Success": state, rc = "succeeded", "0"
        elif isinstance(res, dict) and "Failed" in res: state, rc = "failed", str(res["Failed"])
        else: state = str(res if isinstance(res, str) else next(iter(res))).lower()   # killed, dependencyfailed, ...
    start, end = ts(body.get("start", "")), ts(body.get("end", ""))
    run = "-"
    if start: run = "%ds" % int(((end or datetime.datetime.now()) - start).total_seconds())
    return state, rc, (start.strftime("%H:%M:%S") if start else "-"), run
def line(t):
    state, rc, start, run = info(t)
    extra = ""
    if state == "queued":
        ahead = [o for o in tasks.values() if o["group"] == t["group"] and o["id"] != t["id"]
                 and (info(o)[0] == "running" or (info(o)[0] == "queued" and o["id"] < t["id"]))]
        if ahead:
            a = sorted(ahead, key=lambda o: (info(o)[0] != "running", o["id"]))[0]
            extra = "  queued behind %d %s" % (a["id"], a.get("label") or "-")
    return "%d %s rc=%s group=%s start=%s run=%s label=%s%s" % (
        t["id"], state, rc, t["group"], start, run, (t.get("label") or "-"), extra), state
if want:
    t = tasks.get(want)
    if t is None:
        print("job.sh: no job %s" % want, file=sys.stderr); sys.exit(2)
    text, state = line(t)
    print(text)
    sys.exit(0 if state == "succeeded" else 3 if state in ("running", "queued", "paused", "stashed", "locked") else 1)
for k in sorted(tasks, key=int):
    print(line(tasks[k])[0])
'
}

show_log() {  # ID LINES
  pueue log "$1" --json --lines "$2" 2>/dev/null | python3 -I -c '
import json, sys
try:
    for v in json.load(sys.stdin).values(): print(v.get("output") or "")
except Exception: pass'
}

cmd=${1:-}; [ $# -gt 0 ] && shift
case "$cmd" in
  run)
    group=sim; label=""
    while [ $# -gt 0 ]; do
      case "$1" in
        --group) need "$@"; group=$2; shift 2 ;;
        --label) need "$@"; label=$2; shift 2 ;;
        --) shift; break ;;
        *) die "run: unknown option '$1' (the command goes after --)" ;;
      esac
    done
    [ $# -gt 0 ] || die "run: no command after --"
    case "$group" in sim|light) ;; *) die "run: group must be sim or light" ;; esac
    if [ -z "$label" ]; then
      label="${BB_WORKER_NAME:-nobody} $(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo -)"
    fi
    # One argument is a shell line ("a && b"); several are a command and its arguments.
    if [ $# -eq 1 ]; then inner="bash -c $(printf '%q' "$1")"; else inner=$(printf '%q ' "$@"); fi
    if [ "$group" = sim ]; then
      line="export PATH=\"\$HOME/.cargo/bin:\$PATH\" CARGO_BUILD_JOBS=2; exec taskset -c 0-7 nice $inner"
    else
      line="exec $inner"
    fi
    ensure_daemon
    pueue add --print-task-id -w "$PWD" -g "$group" -l "$label" -- "$line" || die "pueue add failed"
    ;;
  wait)
    id=${1:-}; [ -n "$id" ] || die "wait: no job id"; num "job id" "$id"; shift
    max=540
    while [ $# -gt 0 ]; do case "$1" in --max) need "$@"; num --max "$2"; max=$2; shift 2 ;; *) die "wait: unknown option '$1'" ;; esac; done
    ensure_daemon
    end=$(( $(date +%s) + max ))
    while :; do
      out=$(report wait "$id"); rc=$?
      [ "$rc" -ne 3 ] && break
      [ "$(date +%s)" -ge "$end" ] && break
      sleep 3
    done
    echo "$out"
    case "$rc" in
      0|1) echo "--- last 20 lines of the log"; show_log "$id" 20 ;;
      3) echo "still going after ${max}s: run 'bin/job.sh wait $id' again" ;;
    esac
    exit "$rc"
    ;;
  status)
    ensure_daemon
    if [ -n "${1:-}" ]; then num "job id" "$1"; report status "$1"; [ $? -eq 2 ] && exit 2; exit 0
    else report status; fi
    ;;
  log)
    id=${1:-}; [ -n "$id" ] || die "log: no job id"; num "job id" "$id"; shift
    n=200
    while [ $# -gt 0 ]; do case "$1" in --tail) need "$@"; num --tail "$2"; n=$2; shift 2 ;; *) die "log: unknown option '$1'" ;; esac; done
    ensure_daemon
    report status "$id" >/dev/null || exit 2     # unknown id: report says "no job" and exits 2
    show_log "$id" "$n"
    ;;
  kill)
    id=${1:-}; [ -n "$id" ] || die "kill: no job id"; num "job id" "$id"
    ensure_daemon; pueue kill "$id"
    ;;
  *) sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//'; exit 2 ;;
esac
