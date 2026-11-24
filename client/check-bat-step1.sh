#!/bin/bash
# Headless check of the batter's bat before the swing (player-visuals section 13, build step 1):
# seeds 7 and 6 (a right- and a left-hander among the batters) are played fast with
# -BbBatSim -BbBatCheck (the step 1 bat, off by default until the owner's Q44), which logs one BB_BATCHECK line per pitch (the drawn bat at 1 ms: hand speed
# before the sim's bat moves <= 1.5 m/s, the bat within 2 cm and 2 degrees of the motion carried
# on at both ends of the handover, the same bat to 0.1 cm with the swing withheld, D22), and
# -BbStagingLog, which adds BB_BATHANDS (the worst a hand has been from the handle from the
# moment the sim's bat first moves to the end of the swing, per swing).
# Exits 1 if any BB_BATCHECK line says FAIL, if fewer than 10 swings were checked, or if a hand
# was ever more than 1 cm further from the handle than in the same swing with the bat as it was
# before step 1 (the default bat, run too and compared by play): some swings reach beyond today's
# body (the arms are too short for a far or low aim, up to 10 cm; the base swing's step 3 is for
# that), and they do so with either bat. Builds nothing (build the client first).
# The handover's hand speed (the drawn knob's, while the client's bat closes on the sim's start
# pose) is printed over all the swings, with how many exceed 10 m/s; it is not a failure, the gap
# to close being the sim's (player-visuals 13.10). A long list of seeds without the hand
# comparison shows its tail:
#   BB_SEEDS="1 2 3 4 5 6 7 8 9 10 11 12" BB_SPEED=10 BB_HANDS=0 BB_MIN_SWINGS=3 client/check-bat-step1.sh 60
#   client/check-bat-step1.sh [seconds per run, default 90]
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
UE="${UE_ROOT:-$HOME/UnrealEngine/5.8.4}"
SECS="${1:-90}"
OUT="${BB_CHECK_LOG_DIR:-${TMPDIR:-/tmp}}"
SEEDS="${BB_SEEDS:-7 6}"
PACE="${BB_SPEED:-4}"
HANDS="${BB_HANDS:-1}"    # 0: no hand comparison, no run with the old bat
fail=0
allspeeds=$(mktemp)
run() { # seed, extra args, log
  taskset -c 0-7 nice timeout $((SECS + 120)) "$UE/Engine/Binaries/Linux/UnrealEditor" "$HERE/BaseballClient.uproject" -game -windowed -ResX=888 -ResY=500 -nosplash \
    -RenderOffscreen -BbSeed=$1 -BbSpeed=$PACE -BbStagingLog -BbBenchSeconds=$SECS $2 > "$3" 2>&1
}
for seed in $SEEDS; do
  new="$OUT/bat-step1-seed$seed.log"; old="$OUT/bat-step1-seed$seed-legacy.log"
  run $seed "-BbBatSim -BbBatCheck" "$new"; [ "$HANDS" = 1 ] && run $seed "" "$old"
  swings=$(grep "BB_BATCHECK" "$new" | grep -c "swing=1")
  takes=$(grep "BB_BATCHECK" "$new" | grep -c "swing=0")
  fails=$(grep "BB_BATCHECK" "$new" | grep -c " FAIL")
  echo "seed $seed: $swings swings and $takes takes checked, $fails FAIL"
  grep "BB_BATCHECK" "$new" | grep " FAIL" | sed 's/.*BB_BATCHECK/  BB_BATCHECK/'
  if [ "$fails" -ne 0 ] || [ "$swings" -lt "${BB_MIN_SWINGS:-10}" ]; then fail=1; fi
  grep "BB_BATCHECK" "$new" | grep "swing=1" | sed 's/.*speed_handover=\([0-9.]*\) m\/s.*gap=\([0-9.]*\) cm.*/\1 \2/' >> "$allspeeds"
  [ "$HANDS" = 1 ] || continue
  # The hands, per swing: new bat against old, by play.
  hands() { grep "BB_BATHANDS" "$1" | sed 's/.*play=\([0-9]*\) worst_hand_off=\([-0-9.]*\) cm.*/\1 \2/'; }
  report=$(join <(hands "$new" | sort -k1,1) <(hands "$old" | sort -k1,1) | awk '{n++; if ($2 > 0.1) off++; if ($3 > 0.1) offold++; d = $2 - $3; if (d > worse) worse = d; if ($2 > mx) mx = $2; if ($3 > mxold) mxold = $3} END {printf "%d swings in both runs; a hand more than 0.1 cm off the handle in %d (before step 1: %d), worst %.2f cm (before: %.2f cm), worst increase %.2f cm", n, off, offold, mx, mxold, worse; if (worse > 1.0) exit 1}')
  rc=$?
  echo "  hands: $report"
  if [ $rc -ne 0 ]; then echo "  a hand is more than 1 cm further from the handle than before step 1"; fail=1; fi
done
sort -g "$allspeeds" | awk '{n++; v[n]=$1; if ($1 > 10) hi++; if ($2 > g) g = $2} END {if (n) printf "handover hand speed over %d swings: median %.1f m/s, p95 %.1f, worst %.1f m/s (%d above 10 m/s); largest gap closed %.1f cm\n", n, v[int((n+1)/2)], v[int(0.95*n)+ (0.95*n==int(0.95*n)?0:1)], v[n], hi, g}'
rm -f "$allspeeds"
exit $fail
