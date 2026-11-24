#!/usr/bin/env python3
"""Digitise the curves of Fortenbaugh 2011 (source [4]) Appendix E.

Appendix E (PDF pages 234-249, printed 220-235) holds 32 raster graphs, two to a page, of the
"typical" batter against time (ms, -1000..+200, contact = 0). This script pulls them out of the
PDF with `pdfimages`, finds each graph's time axis from its five vertical event lines
(-621, -340, -170, -93, 0 ms) and its value axis from the frame (top and bottom gridlines), and
reads the thick mean curve by colour, one point per column (about 1.35 ms per pixel).

Usage:  python3 analysis/research/fortenbaugh_appendix_e.py [PDF] [OUT_DIR]
Defaults: sim/target/research-dl/4/s.pdf and sim/target/research-dl/4/ (PNGs in g/, CSVs in csv/).
Needs Pillow and poppler's pdfimages. The data stay under sim/target; only this script is committed.
Reading error: one pixel is 0.87 deg (azimuth), 0.29 deg (elevation), 0.31 deg (lag), 4.8 deg/s
(azimuth velocity); the curve is 6 px thick, so allow 1-2 px, more on steep parts (time, 1 px = 1.35 ms).
"""
import csv, os, subprocess, sys
from PIL import Image

PDF = sys.argv[1] if len(sys.argv) > 1 else "sim/target/research-dl/4/s.pdf"
OUT = sys.argv[2] if len(sys.argv) > 2 else os.path.dirname(PDF)
EVENTS = [-621, -340, -170, -93, 0]
GREEN, ORANGE = (0, 176, 80), (247, 150, 70)
TOP, BOT_SYM, BOT_ZERO = 92.5, 719.5, 672.5   # frame rows: value max; symmetric min; 0-based min
# image index (page order from page 234) -> (name, line colour, value at top, value at bottom, bottom row)
GRAPHS = {
    9:  ("pelvis_x_cm",      ORANGE, 0, -100, BOT_SYM),
    19: ("bat_lag",          ORANGE, 180, 0, BOT_ZERO),
    20: ("trail_sh_elev",    GREEN, 180, 0, BOT_ZERO),
    21: ("lead_sh_elev",     ORANGE, 180, 0, BOT_ZERO),
    22: ("trail_sh_az",      GREEN, 120, -120, BOT_SYM),
    23: ("lead_sh_az",       ORANGE, 120, -120, BOT_SYM),
    26: ("trail_elbow_flex", GREEN, 180, 0, BOT_ZERO),
    27: ("lead_elbow_flex",  ORANGE, 180, 0, BOT_ZERO),
    28: ("bat_elev",         GREEN, 90, -90, BOT_SYM),
    29: ("bat_az",           ORANGE, 270, -270, BOT_SYM),
    30: ("bat_az_vel",       GREEN, 2700, -300, BOT_SYM),
}

def close(a, b, tol=34):
    return sum((a[i] - b[i]) ** 2 for i in range(3)) < tol * tol

def event_columns(px, w):
    cols = []
    for x in range(40, w - 40):
        c = 0
        for y in range(100, 700, 3):
            a, b, d = px[x, y], px[x - 4, y], px[x + 4, y]
            if sum(abs(a[i] - b[i]) for i in range(3)) > 60 and sum(abs(a[i] - d[i]) for i in range(3)) > 60:
                c += 1
        cols.append((c, x))
    xs = sorted(x for c, x in sorted(cols, reverse=True)[:30] if c > 100)
    groups, cur = [], [xs[0]]
    for x in xs[1:]:
        if x - cur[-1] <= 2: cur.append(x)
        else: groups.append(cur); cur = [x]
    groups.append(cur)
    return [sum(g) / len(g) for g in groups]

def digitise(path, colour, vtop, vbot, rbot):
    im = Image.open(path).convert("RGB"); px = im.load(); w, h = im.size
    g = event_columns(px, w)
    ev = [x for x in g if 300 < x < 900][:5]        # the five event lines, left to right
    x0 = g[0]                                        # left frame = -1000 ms
    k = (ev[-1] - x0) / 1000.0                       # px per ms (0 line is the last event line)
    rows = []
    for x in range(int(x0) + 1, int(x0 + 1200 * k)):
        ys = [y for y in range(int(TOP) + 2, int(rbot)) if close(px[x, y], colour)]
        if not ys: continue
        # largest contiguous run
        runs, cur = [], [ys[0]]
        for y in ys[1:]:
            if y - cur[-1] <= 2: cur.append(y)
            else: runs.append(cur); cur = [y]
        runs.append(cur); best = max(runs, key=len)
        yc = sum(best) / len(best)
        t = (x - ev[-1]) / k
        v = vtop + (vbot - vtop) * (yc - TOP) / (rbot - TOP)
        rows.append((t, v))
    return ev, k, rows

def main():
    gdir = os.path.join(OUT, "g"); cdir = os.path.join(OUT, "csv")
    os.makedirs(gdir, exist_ok=True); os.makedirs(cdir, exist_ok=True)
    if not os.listdir(gdir):
        subprocess.check_call(["pdfimages", "-png", "-f", "234", "-l", "249", PDF, os.path.join(gdir, "i")])
    for idx, (name, col, vt, vb, rb) in GRAPHS.items():
        ev, k, rows = digitise(os.path.join(gdir, "i-%03d.png" % idx), col, vt, vb, rb)
        with open(os.path.join(cdir, name + ".csv"), "w", newline="") as f:
            wr = csv.writer(f); wr.writerow(["t_ms", name]); wr.writerows((round(t, 2), round(v, 2)) for t, v in rows)
        print("%-18s events px %s  px/ms %.4f  %d points" % (name, [round(e) for e in ev], k, len(rows)))

if __name__ == "__main__":
    main()
