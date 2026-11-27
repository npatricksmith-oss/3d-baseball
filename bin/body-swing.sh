#!/bin/bash
# The force-driven body's practised swing in the Unreal viewer (PLAN D43; docs/player-visuals.md §14).
#
#   bin/body-swing.sh                          the newest kept swing of this checkout
#   bin/body-swing.sh <plan.json>              that kept swing
#   bin/body-swing.sh --tree <checkout>        the newest kept swing of another checkout, run on that
#                                              checkout's body code (read only; e.g. a research worktree)
#   bin/body-swing.sh --real [<swing id>]      a real hitter's recorded swing on the figure (PLAN D45; default
#                                              OpenBiomechanics swing 478_5): analysis/research/body/real_swing.py,
#                                              whose data must be fetched first (its docstring; never committed)
#   bin/body-swing.sh ... -BbBodySwingShots    any argument starting with - goes to the viewer
#
# Exports the swing every time (about 2 s: the file then always shows the plan run on the body code
# as it is now) into analysis/research/body/swings/<checkout's directory name>/<plan>.swing, then
# opens the viewer (client/run.sh, which builds what has changed).
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
UE="${UE_ROOT:-$HOME/UnrealEngine/5.8.4}"
TREE="$HERE"
PLAN=""
REAL=""
EXTRA=()
while [ $# -gt 0 ]; do
  case "$1" in
    --real)
      REAL="478_5"
      if [ $# -ge 2 ] && [[ "$2" != -* ]]; then REAL="$2"; shift; fi
      shift ;;
    --tree)
      if [ $# -lt 2 ] || [[ "$2" == -* ]]; then
        echo "--tree needs the path of a checkout after it, e.g. bin/body-swing.sh --tree .claude/worktrees/swing-research" >&2
        exit 1
      fi
      if [ ! -d "$2" ]; then
        echo "--tree $2: there is no such directory" >&2
        exit 1
      fi
      TREE="$(cd "$2" && pwd)"
      shift 2 ;;
    -*) EXTRA+=("$1"); shift ;;
    *) PLAN="$1"; shift ;;
  esac
done
if [ ! -x "$UE/Engine/Build/BatchFiles/Linux/Build.sh" ]; then
  echo "Unreal Engine is not at $UE: install Unreal Engine 5.8.4 at ~/UnrealEngine/5.8.4 (or set UE_ROOT to where it is)" >&2
  exit 1
fi
if [ -n "$REAL" ]; then
  # a recorded swing: numpy and the c3d reader through uv (~/.local/bin); about 3 s
  export PATH="$HOME/.local/bin:$PATH"
  RS="$HERE/analysis/research/body/real_swing.py"
  OUT="$(uv run -q --with numpy --with c3d python3 "$RS" "$REAL" --print-out)"
  uv run -q --with numpy --with c3d python3 "$RS" "$REAL" --out "$OUT"
  echo "showing $OUT"
  exec "$HERE/client/run.sh" 7 -BbBodySwing="$OUT" "${EXTRA[@]}"
fi
EXPORTER="$HERE/analysis/research/body/swing_export.py"
if [ -z "$PLAN" ]; then
  PLAN="$(python3 -c 'import sys; sys.path.insert(0, sys.argv[1]); import swing_export as S; print(S.newest_plan(sys.argv[2]) or "")' \
    "$HERE/analysis/research/body" "$TREE")"
  if [ -z "$PLAN" ]; then
    echo "there is no kept swing (analysis/research/body/practice3d_s*_*.json) in $TREE" >&2
    exit 1
  fi
fi
if [ ! -f "$PLAN" ] && [ -f "$TREE/$PLAN" ]; then
  PLAN="$TREE/$PLAN"
fi
if [ ! -f "$PLAN" ]; then
  echo "there is no plan file $PLAN" >&2
  exit 1
fi
PLAN="$(cd "$(dirname "$PLAN")" && pwd)/$(basename "$PLAN")"
OUT="$(python3 "$EXPORTER" "$PLAN" --tree "$TREE" --print-out)"
python3 "$EXPORTER" "$PLAN" --tree "$TREE" --out "$OUT"
echo "showing $OUT"
exec "$HERE/client/run.sh" 7 -BbBodySwing="$OUT" "${EXTRA[@]}"
