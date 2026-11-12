#!/usr/bin/env bash
# Add "3D Baseball" to the desktop's app menu (GNOME Activities / app grid).
# Pin it from there with right-click > Pin to Dash. Right-click also offers a chosen seed
# and the Rerun viewer.
# Remove it with: bin/install_launcher.sh --uninstall
set -euo pipefail

ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd -P)"
APPS="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
DESKTOP="${APPS}/3d-baseball.desktop"

if [[ "${1:-}" == "--uninstall" ]]; then
  rm -f "$DESKTOP"
  update-desktop-database "$APPS" 2>/dev/null || true
  echo "Removed ${DESKTOP}"
  exit 0
fi

chmod +x "${ROOT}/bin/play.sh" "${ROOT}/client/run.sh"
mkdir -p "$APPS"
# Desktop-entry Exec quoting: the path has a space, so it is double-quoted.
cat >"$DESKTOP" <<EOF
[Desktop Entry]
Type=Application
Name=3D Baseball
GenericName=Baseball Simulation
Comment=Watch an AI-vs-AI game from the physics sim
Exec="${ROOT}/bin/play.sh"
Icon=${ROOT}/bin/icon.svg
Terminal=false
Categories=Game;SportsGame;
Keywords=baseball;sim;
StartupNotify=true
Actions=Seed;Rerun;

[Desktop Action Seed]
Name=Play a Chosen Seed…
Exec="${ROOT}/bin/play.sh" --ask

[Desktop Action Rerun]
Name=Open in the Rerun Viewer
Exec="${ROOT}/bin/play.sh" --rerun
EOF
chmod +x "$DESKTOP"
desktop-file-validate "$DESKTOP" 2>/dev/null || true
update-desktop-database "$APPS" 2>/dev/null || true
echo "Installed ${DESKTOP}"
echo "Find \"3D Baseball\" in Activities; right-click it and choose Pin to Dash to keep it handy."
