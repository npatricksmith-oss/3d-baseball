# /// script
# requires-python = ">=3.11"
# dependencies = ["matplotlib>=3.8"]
# ///
"""Top-down plots of live-ball plays from `cargo run --example dump_plays`.

Usage: uv run analysis/plots/plays_topdown.py plays.json out.png
"""
import json
import math
import sys
import textwrap

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402


def field(ax):
    d = 27.432 / math.sqrt(2)
    ax.plot([0, d, 0, -d, 0], [0, d, 2 * d, d, 0], color="#b08050", lw=1)
    for sgn in (-1, 1):
        ax.plot([0, sgn * 100], [0, 100], color="#bbb", lw=0.8)
    fence = [(330 / 3.281, -45), (400 / 3.281, 0), (330 / 3.281, 45)]
    th = [math.radians(-45 + 90 * i / 60) for i in range(61)]
    def dist(t):
        s = max(-1, min(1, t / math.radians(45)))
        side = fence[0][0] if s < 0 else fence[2][0]
        return fence[1][0] + (side - fence[1][0]) * s * s
    ax.plot([dist(t) * math.sin(t) for t in th], [dist(t) * math.cos(t) for t in th], color="#3a3", lw=1)


def main(src, dst):
    plays = json.load(open(src))
    n = len(plays)
    cols = 4
    rows = (n + cols - 1) // cols
    fig, axes = plt.subplots(rows, cols, figsize=(4.2 * cols, 4.4 * rows))
    for ax, p in zip(axes.flat, plays):
        field(ax)
        bx = [b[0] for b in p["ball"]]
        by = [b[1] for b in p["ball"]]
        ax.plot(bx, by, color="#e0a000", lw=1.5)
        for a in p["actors"]:
            xs = [q[0] for q in a["p"]]
            ys = [q[1] for q in a["p"]]
            if not xs:
                continue
            fielder = a["id"] & 0x80
            c = "#3070ff" if fielder else "#e03030"
            ax.plot(xs, ys, color=c, lw=1)
            ax.plot(xs[-1], ys[-1], "o", color=c, ms=3)
        ax.set_xlim(-90, 90)
        ax.set_ylim(-10, 130)
        ax.set_aspect("equal")
        ax.set_title("\n".join(textwrap.wrap(p["label"], 48)), fontsize=7)
        ax.set_xticks([])
        ax.set_yticks([])
    for ax in list(axes.flat)[n:]:
        ax.axis("off")
    fig.tight_layout()
    fig.savefig(dst, dpi=80)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
