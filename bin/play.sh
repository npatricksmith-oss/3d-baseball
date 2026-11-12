#!/usr/bin/env bash
# Watch one AI-vs-AI game.
#   bin/play.sh            a new seed each launch (shown in the notification, so a game can be replayed)
#   bin/play.sh 42         replay seed 42
#   bin/play.sh --ask      ask for a seed
#   bin/play.sh --rerun    use the Rerun viewer even when Unreal is installed
# With Unreal Engine at $UE_ROOT (default ~/UnrealEngine/5.8.4) it builds and opens the 3D client
# (client/run.sh). Without it, it records the game and opens it in the Rerun viewer instead.
# Output of the last launch: ${XDG_STATE_HOME:-~/.local/state}/3d-baseball/launch.log
set -euo pipefail

ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd -P)"
UE="${UE_ROOT:-$HOME/UnrealEngine/5.8.4}"
STATE="${XDG_STATE_HOME:-$HOME/.local/state}/3d-baseball"
OUT="$ROOT/sim/out"
LOG="$STATE/launch.log"
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
mkdir -p "$STATE" "$OUT"

say() {
  echo "$1" >&2
  command -v notify-send >/dev/null && notify-send -a "3D Baseball" "3D Baseball" "$1" || true
}
fail() {
  say "$1 See $LOG"
  exit 1
}

viewer=auto
seed=""
for a in "$@"; do
  case "$a" in
    --rerun) viewer=rerun ;;
    --ask)
      seed="$(zenity --entry --title="3D Baseball" --text="Seed to play:" --entry-text="7" 2>/dev/null)" || exit 0 ;;
    *) seed="$a" ;;
  esac
done
[[ -z "$seed" ]] && seed=$(( $(date +%s) % 100000 ))
[[ "$seed" =~ ^[0-9]+$ ]] || fail "Seed must be a whole number, got \"$seed\"."

if pgrep -f "UnrealEditor.*BaseballClient.uproject" >/dev/null; then
  say "A game is already open. Close it (Esc or Q) before starting another."
  exit 0
fi

exec >"$LOG" 2>&1
echo "launch $(date -Is) seed=$seed viewer=$viewer root=$ROOT"

if [[ "$viewer" == auto && -x "$UE/Engine/Binaries/Linux/UnrealEditor" ]]; then
  say "Building and opening seed $seed (about 20 s; the first launch takes a few minutes)."
  "$ROOT/client/run.sh" "$seed" || fail "The 3D client stopped with an error."
  exit 0
fi

if [[ "$viewer" == auto ]]; then
  say "Unreal Engine is not installed at $UE, so seed $seed opens in the Rerun viewer."
else
  say "Recording seed $seed for the Rerun viewer."
fi
command -v rerun >/dev/null || fail "The Rerun viewer is missing: uv tool install rerun-sdk==0.41.0"
(cd "$ROOT/sim" && cargo build --release -p bb_cli) || fail "The sim failed to build."
rrd="$OUT/game-$seed.rrd"
"$ROOT/sim/target/release/sim-cli" game --seed "$seed" --pbp --rrd "$rrd" >"$OUT/game-$seed.txt" \
  || fail "The game failed to simulate."
tail -n 15 "$OUT/game-$seed.txt"
say "Seed $seed: play-by-play in sim/out/game-$seed.txt. Scrub the game_time timeline in Rerun."
# Local only: by default the viewer listens on every network interface.
exec rerun --bind 127.0.0.1 "$rrd"
