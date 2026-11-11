#!/bin/bash
# Build the sim library and the client, then play a game in a window.
#   client/run.sh [seed] [extra UnrealEditor arguments]
# Unattended checks: add -RenderOffscreen and -BbShots=6 (screenshots to client/Saved/Shots,
# then quit) or -BbBenchSeconds=60 (log frame rate and memory as BB_RESULT, then quit).
set -euo pipefail
UE="${UE_ROOT:-$HOME/UnrealEngine/5.8.4}"
HERE="$(cd "$(dirname "$0")" && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
(cd "$HERE/../sim" && cargo build --release -p bb_ffi)
"$UE/Engine/Build/BatchFiles/Linux/Build.sh" BaseballClientEditor Linux Development -project="$HERE/BaseballClient.uproject"
SEED="${1:-7}"
shift || true
exec "$UE/Engine/Binaries/Linux/UnrealEditor" "$HERE/BaseballClient.uproject" -game -windowed -ResX=1600 -ResY=900 -nosplash -BbSeed="$SEED" "$@"
