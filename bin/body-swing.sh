#!/bin/bash
# The force-driven body's practised swing in the Unreal viewer (PLAN D43; docs/player-visuals.md §14).
#
#   bin/body-swing.sh                          the newest kept swing of this checkout
#   bin/body-swing.sh <plan.json>              that kept swing
#   bin/body-swing.sh --tree <checkout>        the newest kept swing of another checkout, run on that
#                                              checkout's body code (read only; e.g. a research worktree)
#   bin/body-swing.sh ... -BbBodySwingShots    any argument starting with - goes to the viewer
#
# Exports the swing (analysis/research/body/swing_export.py) when its file is missing or older than
# the plan or the exporter, then opens the viewer (client/run.sh, which builds what has changed).
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
TREE="$HERE"
PLAN=""
EXTRA=()
while [ $# -gt 0 ]; do
  case "$1" in
    --tree) TREE="$(cd "$2" && pwd)"; shift 2 ;;
    -*) EXTRA+=("$1"); shift ;;
    *) PLAN="$1"; shift ;;
  esac
done
EXPORTER="$HERE/analysis/research/body/swing_export.py"
if [ -z "$PLAN" ]; then
  PLAN="$(python3 -c 'import sys; sys.path.insert(0, sys.argv[1]); import swing_export as S; print(S.newest_plan(sys.argv[2]) or "")' \
    "$HERE/analysis/research/body" "$TREE")"
  if [ -z "$PLAN" ]; then
    echo "no kept swing (analysis/research/body/practice3d_s*_*.json) in $TREE" >&2
    exit 1
  fi
fi
PLAN="$(cd "$(dirname "$PLAN")" && pwd)/$(basename "$PLAN")"
OUT="$HERE/analysis/research/body/swings/$(basename "${PLAN%.json}").swing"
if [ ! -f "$OUT" ] || [ "$PLAN" -nt "$OUT" ] || [ "$EXPORTER" -nt "$OUT" ]; then
  python3 "$EXPORTER" "$PLAN" --tree "$TREE" --out "$OUT"
fi
echo "showing $OUT"
exec "$HERE/client/run.sh" 7 -BbBodySwing="$OUT" "${EXTRA[@]}"
