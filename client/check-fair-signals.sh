#!/bin/bash
# Headless check of the umpire's fair-ball signal on the owner's seed 51001 plays
# (player-visuals section 12): 89 and 146 (fair down the third-base line) must each give one
# `fair` signal; 66 (ground out to short) and 102 (single up the middle) must give none.
# Run from anywhere; builds nothing (build the client first). Exits 1 on any mismatch.
#   client/check-fair-signals.sh
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
UE="${UE_ROOT:-$HOME/UnrealEngine/5.8.4}"
fail=0
for entry in "89:1" "146:1" "66:0" "102:0"; do
  play="${entry%%:*}"; want="${entry##*:}"
  toml=$(ls "$HERE"/../saved-plays/seed51001-play${play}-*.toml | head -1)
  log=$(mktemp)
  taskset -c 0-7 nice timeout 180 "$UE/Engine/Binaries/Linux/UnrealEditor" "$HERE/BaseballClient.uproject" -game -windowed -ResX=888 -ResY=500 -nosplash \
    -RenderOffscreen -BbSavedPlay="$toml" -BbStagingLog -BbShots=1 > "$log" 2>&1
  got=$(grep "BB_SIGNAL" "$log" | grep -c " fair ")
  grep "BB_SIGNAL" "$log" | grep " fair " | sed 's/.*BB_SIGNAL/  BB_SIGNAL/'
  if [ "$got" -eq "$want" ]; then echo "play $play: $got fair signal(s), expected $want: ok"; else echo "play $play: $got fair signal(s), expected $want: FAIL"; fail=1; fi
  rm -f "$log"
done
exit $fail
