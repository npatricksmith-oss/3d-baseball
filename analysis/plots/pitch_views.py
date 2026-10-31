# /// script
# requires-python = ">=3.11"
# dependencies = ["matplotlib>=3.8"]
# ///
"""Render sim-cli pitch JSON (``--json``) as side, top, and catcher views.

Usage: uv run analysis/plots/pitch_views.py sim/out/tunnel.json sim/out/tunnel.png
"""
import json
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

FT = 0.3048
COLORS = {"FF": "#eb5046", "SI": "#fa9632", "FC": "#b4643c", "SL": "#e6c81e", "ST": "#a0c83c",
          "SV": "#5ab45a", "CU": "#46b4f0", "KC": "#5078e6", "CH": "#3cc8a0", "FS": "#aa78e6"}


def main(src, dst):
    data = json.load(open(src))
    fig, (side, top, catcher) = plt.subplots(1, 3, figsize=(16, 5), gridspec_kw={"width_ratios": [2.2, 2.2, 1]})
    seen = set()
    for p in data["pitches"]:
        t, x, y, z = zip(*p["path"])
        c = COLORS.get(p["type"], "#888")
        lab = p["type"] if p["type"] not in seen else None
        seen.add(p["type"])
        side.plot([v / FT for v in y], [v / FT for v in z], color=c, lw=1.2, label=lab)
        top.plot([v / FT for v in y], [v / FT for v in x], color=c, lw=1.2)
        catcher.plot(x[-1] / FT, z[-1] / FT, "o", color=c, ms=7, alpha=0.8)
    for ax in (side, top):
        ax.invert_xaxis()
        ax.set_xlabel("distance from rear of plate (ft)")
        ax.axvline(23.8, color="#999", ls=":", lw=1)
        ax.grid(alpha=0.3)
    side.set_ylabel("height (ft)")
    side.set_title("side view (pitcher on the left)")
    top.set_ylabel("x (ft, + toward 1B)")
    top.set_title("top view")
    side.legend()
    hw = 8.5 / 12
    zb, zt = 0.27 * 1.85 / FT, 0.535 * 1.85 / FT
    catcher.plot([-hw, hw, hw, -hw, -hw], [zb, zb, zt, zt, zb], color="#5aa0ff")
    catcher.set_xlim(-2.5, 2.5)
    catcher.set_ylim(0, 5)
    catcher.set_aspect("equal")
    catcher.set_title("catcher's view at plate")
    catcher.grid(alpha=0.3)
    fig.tight_layout()
    fig.savefig(dst, dpi=90)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
