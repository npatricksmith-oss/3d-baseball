"""Summaries of OpenBiomechanics' hitting joint angles that docs/research/body notes 11 and 12 quote.

Run:  uv run --with pandas --with numpy python3 analysis/research/body/obp_hitting_angles.py [path/to/joint_angles.csv]

The DATA ARE NOT IN THIS REPOSITORY (the dataset's licence is cross-check only, note 7).  Fetch once:
  mkdir -p sim/target/codex-probes/obp && cd sim/target/codex-probes/obp
  curl -sLO https://github.com/drivelineresearch/openbiomechanics/releases/download/dataset-v1/hitting_joint_angles.zip
  unzip -o hitting_joint_angles.zip          # gives joint_angles.csv (677 swings, 360 Hz)
The script reads sim/target/codex-probes/obp/joint_angles.csv by default (relative to the repository root).

Events (columns of the file, seconds from the start of each swing's record): fp_10_time (lead foot at 10% of body
weight), fp_100_time (100%: "foot plant"), contact_time (the dataset's bat-ball contact estimate).  Swings missing
an event are skipped (650 of 677).  Angle conventions are the dataset's own (its README; zeros not stated).
Prints, per channel: mean +- SD at foot plant and at contact; mean lowest and highest value in the last 0.5 s
before contact; the per-swing range from foot plant to contact as median [p10-p90] and the last-0.5 s range.
"""
import os, sys
import numpy as np, pandas as pd

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
path = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "sim/target/codex-probes/obp/joint_angles.csv")
df = pd.read_csv(path)

CH = ["lead_wrist_angle_x", "lead_wrist_angle_y", "rear_wrist_angle_x", "rear_wrist_angle_y",
      "lead_elbow_angle_z", "rear_elbow_angle_z", "torso_angle_x", "torso_angle_y", "pelvis_angle_x", "pelvis_angle_y",
      "torso_pelvis_angle_y", "rear_ankle_angle_x", "rear_ankle_angle_y", "rear_ankle_angle_z",
      "lead_ankle_angle_x", "lead_ankle_angle_y", "lead_ankle_angle_z", "rear_hip_angle_z", "lead_hip_angle_z"]

def at(d, t, c):
    return d.loc[(d.time - t).abs().idxmin(), c]

rows = []
for k, d in df.groupby("session_swing"):
    d = d.reset_index(drop=True)
    tc, t10, t100 = d.contact_time.iloc[0], d.fp_10_time.iloc[0], d.fp_100_time.iloc[0]
    if not (np.isfinite(tc) and np.isfinite(t10) and np.isfinite(t100)):
        continue
    last = d[(d.time >= tc - 0.5) & (d.time <= tc)]
    sw = d[(d.time >= t100) & (d.time <= tc)]
    r = {}
    for c in CH:
        r[c + "|fp10"] = at(d, t10, c); r[c + "|fp100"] = at(d, t100, c); r[c + "|contact"] = at(d, tc, c)
        r[c + "|min05"] = last[c].min(); r[c + "|max05"] = last[c].max()
        r[c + "|range05"] = last[c].max() - last[c].min()
        r[c + "|rangeFP"] = (sw[c].max() - sw[c].min()) if len(sw) else np.nan
    rows.append(r)
R = pd.DataFrame(rows)
print(f"{path}: {df.session_swing.nunique()} swings, {len(R)} with all three events\n")
ms = lambda v: f"{v.mean():+.1f} ± {v.std():.1f}"
pr = lambda v: (lambda a: f"{np.median(a):.1f} [{np.quantile(a, .1):.1f}-{np.quantile(a, .9):.1f}]")(v[np.isfinite(v)].values)
print(f"{'channel':22s} {'fp10':>14s} {'foot plant':>14s} {'contact':>14s} {'low05':>8s} {'high05':>8s}  range plant->contact | range last 0.5 s (median [p10-p90])")
for c in CH:
    print(f"{c:22s} {ms(R[c+'|fp10']):>14s} {ms(R[c+'|fp100']):>14s} {ms(R[c+'|contact']):>14s} "
          f"{R[c+'|min05'].mean():+8.1f} {R[c+'|max05'].mean():+8.1f}  {pr(R[c+'|rangeFP'])} | {pr(R[c+'|range05'])}")
