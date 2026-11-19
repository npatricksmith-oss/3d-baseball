#!/bin/bash
# Pre-build step of both client targets: writes Intermediate/BbStamp/BbBuildStamp.h with the
# commit this build is made from, "<short sha>" or "<short sha>-dirty" (any uncommitted or
# untracked change), or "unknown" when git cannot say. UBT runs pre-build steps on every build,
# so the stamp is current even when UBT reuses its cached rules (which would keep a stale
# value read once). The header is rewritten only when its text changes, so BbSavePlay.cpp
# alone is recompiled when the commit or the dirtiness changed.
#   stamp-commit.sh <project dir>
set -u
PROJECT="${1:-.}"
OUT="$PROJECT/Intermediate/BbStamp"
mkdir -p "$OUT"
TOP="$(git -C "$PROJECT" rev-parse --show-toplevel 2>/dev/null)" || TOP=""
COMMIT="unknown"
if [ -n "$TOP" ]; then
  SHA="$(git -C "$TOP" rev-parse --short HEAD 2>/dev/null)" || SHA=""
  if [[ "$SHA" =~ ^[0-9a-fA-F]+$ ]]; then
    if STATUS="$(git -C "$TOP" status --porcelain 2>/dev/null)"; then
      if [ -z "$STATUS" ]; then COMMIT="$SHA"; else COMMIT="$SHA-dirty"; fi
    else
      # status failed: say dirty rather than claim clean
      COMMIT="$SHA-dirty"
    fi
  fi
fi
NEW="// Written by client/Build/stamp-commit.sh before every build. Not edited by hand.
#pragma once
#define BB_BUILD_COMMIT \"$COMMIT\"
"
if [ "$(cat "$OUT/BbBuildStamp.h" 2>/dev/null)" != "${NEW%$'\n'}" ]; then
  printf '%s' "$NEW" > "$OUT/BbBuildStamp.h"
fi
