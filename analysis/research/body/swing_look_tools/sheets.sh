#!/bin/bash
# The viewer's pictures of a swing (client/Saved/Shots/bodyswing_<n>_<m|p><ms>ms_<pitcher|open|above>.png, made by
# bin/body-swing.sh ... -RenderOffscreen -BbBodySwingShots=10) tiled into sheets to read: nine instants a sheet, one
# column an instant, from the pitcher's side, the open side and above; each picture cut to the batter.
#   analysis/research/body/swing_look_tools/sheets.sh <shots dir> <out dir> [instants a sheet]
set -euo pipefail
IN="$1"; OUT="$2"; PER="${3:-9}"
mkdir -p "$OUT"
TMP="$(mktemp -d "$OUT/.tiles.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT
n=0; sheet=1; cols=()
for f in "$IN"/bodyswing_*_pitcher.png; do
  base="${f%_pitcher.png}"; tag="$(basename "$base")"; ms="${tag##*_}"
  label="$(echo "$ms" | sed -e 's/^m0*\([0-9]\)/-\1/' -e 's/^p0*\([0-9]\)/+\1/' -e 's/ms$/ ms/')"
  magick \( "$base"_pitcher.png -crop 400x400+250+60 +repage -resize 170x170 \) \
         \( "$base"_open.png -crop 400x400+240+60 +repage -resize 170x170 \) \
         \( "$base"_above.png -crop 400x400+240+80 +repage -resize 170x170 \) -append \
         -gravity north -fill yellow -undercolor '#0008' -pointsize 15 -annotate +0+2 "$label" "$TMP/col_$(printf %04d $n).png"
  cols+=("$TMP/col_$(printf %04d $n).png"); n=$((n + 1))
  if [ "${#cols[@]}" -eq "$PER" ]; then
    magick "${cols[@]}" +append "$OUT/sheet_$(printf %02d $sheet).jpg"; sheet=$((sheet + 1)); cols=()
  fi
done
if [ "${#cols[@]}" -gt 0 ]; then magick "${cols[@]}" +append "$OUT/sheet_$(printf %02d $sheet).jpg"; fi
echo "$n instants on $(ls "$OUT"/sheet_*.jpg | wc -l) sheets in $OUT"
