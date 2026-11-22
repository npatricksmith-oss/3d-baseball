#!/usr/bin/env bash
# Block until one of the named workers reports stopped, blocked, failed or ended
# (see docs/worker-status-hooks.md). Usage:
#   bin/worker-wait.sh <name>... [--timeout S] [--stale S] [--any-age] [--dir D]
# Prints "<name>: <reason>: <status line>" and exits 0 on the first hit.
# Exit 3 on timeout, 2 on usage error.
# A status file last written before this script started is an earlier worker's and is
# ignored until it is rewritten (--since-start, the default); --any-age counts it too.
# A "working" or "started" line whose file has not been touched for --stale
# seconds (default 1800; PostToolUse refreshes it) returns "stale".
need() { [ $# -ge 2 ] || { echo "worker-wait.sh: $1 needs a value" >&2; exit 2; }; }
num() { case $2 in ''|*[!0-9]*) echo "worker-wait.sh: $1 needs a whole number of seconds, got '$2'" >&2; exit 2;; esac; }
names=(); timeout=0; stale=1800; anyage=0; dir=""
while [ $# -gt 0 ]; do
  case $1 in
    --timeout) need "$@"; num "$1" "$2"; timeout=$2; shift 2;;
    --stale) need "$@"; num "$1" "$2"; stale=$2; shift 2;;
    --any-age) anyage=1; shift;;
    --since-start) anyage=0; shift;;
    --dir) need "$@"; dir=$2; shift 2;;
    -*) echo "unknown option $1" >&2; exit 2;;
    *) names+=("$(printf '%s' "$1" | tr -c 'A-Za-z0-9._-' '_')"); shift;;
  esac
done
[ ${#names[@]} -gt 0 ] || { echo "usage: $0 <name>... [--timeout S] [--stale S] [--any-age]" >&2; exit 2; }
if [ -z "$dir" ]; then
  common=$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || { echo "not in a git repo" >&2; exit 2; }
  dir="$(dirname "$common")/.claude/worker-status"
fi
start=$(date +%s); deadline=$(( timeout > 0 ? start + timeout : 0 ))
while :; do
  now=$(date +%s)
  for n in "${names[@]}"; do
    f="$dir/$n"; [ -f "$f" ] || continue
    m=$(stat -c %Y "$f" 2>/dev/null) || continue
    line=$(head -n1 "$f" 2>/dev/null); state=${line%% *}
    case $state in
      stopped|blocked|failed|ended) if [ "$anyage" = 0 ] && [ "$m" -lt "$start" ]; then continue; fi; echo "$n: $state: $line"; exit 0;;
      working|started) if [ "$anyage" = 0 ] && [ "$m" -lt "$start" ]; then continue; fi
        if [ $(( now - m )) -ge "$stale" ]; then echo "$n: stale (no update for $(( now - m )) s): $line"; exit 0; fi;;
    esac
  done
  if [ "$deadline" -gt 0 ] && [ "$now" -ge "$deadline" ]; then echo "timeout after ${timeout}s"; exit 3; fi
  sleep 5
done
