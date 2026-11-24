#!/bin/bash
# Headless check of the batter's bat before the swing (player-visuals section 13, build step 1):
# seeds 7 and 6 (a right- and a left-hander among the batters) are played fast with
# -BbBatCheck, which logs one BB_BATCHECK line per pitch (the drawn bat at 1 ms: hand speed
# before the sim's bat moves <= 1.5 m/s, the bat within 2 cm and 2 degrees of the motion carried
# on at both ends of the handover, the same bat to 0.1 cm with the swing withheld, D22), and
# -BbStagingLog, which adds BB_BATHANDS (the worst a hand has been from the handle through the
# handover and the swing). Exits 1 if any line says FAIL, if fewer than 10 swings were checked,
# or if a hand was ever more than 0.1 cm off the handle. Builds nothing (build the client first).
#   client/check-bat-step1.sh [seconds per seed, default 90]
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
UE="${UE_ROOT:-$HOME/UnrealEngine/5.8.4}"
SECS="${1:-90}"
fail=0
for seed in 7 6; do
  log=$(mktemp)
  taskset -c 0-7 nice timeout $((SECS + 120)) "$UE/Engine/Binaries/Linux/UnrealEditor" "$HERE/BaseballClient.uproject" -game -windowed -ResX=888 -ResY=500 -nosplash \
    -RenderOffscreen -BbSeed=$seed -BbSpeed=4 -BbBatCheck -BbStagingLog -BbBenchSeconds=$SECS > "$log" 2>&1
  swings=$(grep "BB_BATCHECK" "$log" | grep -c "swing=1")
  takes=$(grep "BB_BATCHECK" "$log" | grep -c "swing=0")
  fails=$(grep "BB_BATCHECK" "$log" | grep -c " FAIL")
  worst=$(grep "BB_BATHANDS" "$log" | sed 's/.*worst_hand_off=\([-0-9.]*\) cm.*/\1/' | sort -g | tail -1)
  echo "seed $seed: $swings swings and $takes takes checked, $fails FAIL, worst hand off the handle ${worst:-none} cm"
  grep "BB_BATCHECK" "$log" | grep " FAIL" | sed 's/.*BB_BATCHECK/  BB_BATCHECK/'
  if [ "$fails" -ne 0 ] || [ "$swings" -lt 10 ]; then fail=1; fi
  if [ -n "$worst" ] && awk "BEGIN{exit !($worst > 0.1)}"; then echo "  a hand was $worst cm off the handle"; fail=1; fi
  cp "$log" "${BB_CHECK_LOG_DIR:-/tmp}/bat-step1-seed$seed.log" 2>/dev/null
  rm -f "$log"
done
exit $fail
