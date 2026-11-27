"""A real hitter's recorded swing put on the batter's figure, for the Unreal viewer (PLAN D45; docs/player-visuals.md 14.1;
docs/research/swing/12-real-swing-on-the-figure.md).

    uv run --with numpy --with c3d python3 analysis/research/body/real_swing.py [<swing id>] [--out <file>]
        writes analysis/research/body/swings/real/<swing id>.swing (bb-body-swing 1, the viewer's file); default swing 478_5
    ... real_swing.py <swing id> --report      the mapping's figures only (lengths, scale, reach, feet, placement)
    ... real_swing.py <swing id> --draw <dir> [--every-ms 10]   (add --with matplotlib) the recorded hitter and the figure as
        stick figures from the viewer's three sides at the same instants as -BbBodySwingShots=<ms>, side by side

THE DATA ARE NOT IN THIS REPOSITORY. OpenBiomechanics' hitting data (Driveline, github.com/drivelineresearch/openbiomechanics,
release dataset-v1) are CC BY-NC-SA 4.0: non-commercial, and anything derived from them under the same licence. They and every
file this script writes stay out of git (sim/target/..., the ignored analysis/research/body/swings/). Fetch once, from the
repository root:
    mkdir -p sim/target/codex-probes/obp && cd sim/target/codex-probes/obp
    curl -sLO https://github.com/drivelineresearch/openbiomechanics/releases/download/dataset-v1/hitting_c3d.zip && unzip -q hitting_c3d.zip
    curl -sLO https://github.com/drivelineresearch/openbiomechanics/releases/download/dataset-v1/hitting_landmarks.zip && unzip -q hitting_landmarks.zip
    for f in metadata.csv poi/hittrax.csv; do curl -sLO https://raw.githubusercontent.com/drivelineresearch/openbiomechanics/main/baseball_hitting/data/$f; done
(BB_OBP_DIR names another directory holding the same files.)

What is read, per swing: the C3D's 55 markers at 360 Hz (the body's skin markers, the head's four, the heels and toes, the bat's
end markers Marker1 (knob end) and Marker2/3 (barrel end)); from landmarks.csv the dataset's own joint centres where the
markers do not give one (shoulders, elbows, wrists, hands, knees, ankles: Visual3D's), its sweet spot and its events
(fp_10: the lead foot carries 10 % of the body's weight; fp_100: all of it; contact); from metadata.csv his height and level;
from hittrax.csv where the ball was met, relative to the plate, and the pitch's speed and angle.

THE MAPPING (the note, section 2, says why each choice and what was tried first):
  1. Lab to sim frame: the lab's +x (toward the pitcher) is the sim's +Y, its +y (toward the right-hander's box) the sim's -X,
     +z up in both: a rotation, no mirror. Heights are the lab's (its z = 0 is the floor the plates are in).
  2. One uniform scale s about the floor: s = the body model's leg (de Leva at 1.84 m: thigh + shank, analysis/oracle/body3d)
     over his (median over the swing), so the figure stands as tall in the legs as the force-driven batter (1.81 m in all for
     478_5, from 1.75 m). Still points stay still under a uniform scale. The bat is not scaled: it is his 34-in bat.
  3. Placed by one translation in x and y so that the sweet spot at contact is where HitTrax saw the ball met (inches from the
     plate's back tip), the only placement the data give; z untouched.
  4. The figure has HIS proportions and HIS joint centres at every sample, not the model's segment lengths: with the model's
     (upper arm 5 cm shorter, shoulders 9 cm wider) his hands could not be reached on his bat by up to 11 cm, and fixed
     lengths of his own moved his lead knee up to 7 cm off (his markers ride on skin: his thigh reads 0.44-0.49 m). So:
       hips: Harrington 2007's hip centres from his four pelvis markers (the dataset's left_hip/right_hip are 0.31 m apart,
         pelvis landmarks, not hip centres); knees, ankles, elbows, wrists, shoulders and hands: the dataset's own joint
         centres (Visual3D's), scaled and placed; each segment drawn at the length they give at that sample (the viewer
         stretches each piece to it);
       pelvis: rigid, its axis fixed in his pelvis's own frame (from the four pelvis markers), aimed at his shoulders'
         midpoint at the first sample; the lumbar joint up it at the model's share of his hips-to-shoulders distance;
       trunk: from the lumbar joint to his shoulders' midpoint; his shoulders that far apart along his shoulder line squared
         to the trunk's axis (the viewer's chest piece can stretch but not shear; the most a shoulder moves is 22 mm);
       hands: from his wrist to his hand's centre, which is on the handle as long as he holds it;
       feet: heading and pitch from his heel and toe markers, no roll (the foot's left kept level); the heel's sole under
         the heel marker by its flat height; the ball of the foot where the viewer's shoe has it for his ankle's height;
       head: his four head markers' frame set right by the headband's offset in his standing calibration trial;
       bat: from his knob-end marker toward his barrel-end markers, its length theirs.
  5. Each limb's turn about its own axis: the thigh's from his knee's two markers, the shank's from his ankle's two, the
     upper arm's from his elbow's two, the forearm's toward his thumb from his wrist's two; pelvis and trunk from the joints.
  6. The ball: a machine pitch at HitTrax's speed and angle, under gravity, whose centre arrives touching the barrel's face at
     the sweet spot at his contact instant; after contact the batted ball at HitTrax's exit speed, launch angle and spray,
     under gravity. Pictures for the eye only.
Nothing is smoothed, fitted or made up beyond this.
"""
import csv
import math
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
OBP = os.environ.get("BB_OBP_DIR", os.path.join(ROOT, "sim", "target", "codex-probes", "obp"))
OUT_DIR = os.path.join(HERE, "swings", "real")
FORMAT = "bb-body-swing 1"
DEFAULT_SWING = "478_5"
IN = 0.0254
MPH = 0.44704
G = 9.81

sys.path.insert(0, os.path.join(ROOT, "analysis", "oracle"))
sys.path.insert(0, HERE)
import common as C                     # noqa: E402  de Leva's table
from body3d import body as BODY       # noqa: E402  the model batter's geometry (constants only)

# ---- the figure: the body model's batter (analysis/oracle/body3d/body.py Model, its defaults) ----------------------------
FIG_H = 1.84
SHOE_BALL_PER_ANKLE = 1.8232   # the viewer's shoe: the ball of the foot 15.02 cm ahead of the ankle, the ankle 8.24 cm up (its BB_BODYSWING log)


def figure_lengths(H=FIG_H):
    """the body model's segment lengths (m), as body3d.body.Model builds them from de Leva and its [E] constants."""
    seg = lambda n: C.segment(n, H, 92.0)[0]
    return {
        "thigh": seg("thigh"), "shank": seg("shank"), "uarm": seg("upperarm"), "farm": seg("forearm"),
        "pelvis": seg("lpt"),                                         # hip centres' midpoint to the lumbar joint
        "trunk": seg("mpt") + BODY.SHOULDER_UP * seg("upt"),          # lumbar joint to the shoulders' midpoint
        "hip_w": 2.0 * BODY.HIP_HALF * H, "shoulder_w": 2.0 * BODY.SHOULDER_HALF * H,
        "hand": BODY.WRIST_TO_GRIP,                                   # wrist centre to the handle's axis
    }


# ---- reading the data ------------------------------------------------------------------------------------------------------
def _csv_rows(path, key=None):
    with open(path, newline="") as f:
        r = csv.reader(f)
        head = next(r)
        if key is None:
            return head, list(r)
        out = []
        for row in r:
            if row and row[0] == key:
                out.append(row)
            elif out:
                break                   # a swing's rows are together
        return head, out


def landmarks(sid):
    """the swing's rows of landmarks.csv as {column: array}; cached per swing under OBP/swings/<sid>/ (the full file is 212 MB)."""
    cache = os.path.join(OBP, "swings", sid, "landmarks.csv")
    if not os.path.isfile(cache):
        src = os.path.join(OBP, "landmarks.csv")
        if not os.path.isfile(src):
            raise SystemExit("no %s: fetch hitting_landmarks.zip (this script's docstring)" % src)
        head, rows = _csv_rows(src, sid)
        if not rows:
            raise SystemExit("swing %s is not in %s" % (sid, src))
        os.makedirs(os.path.dirname(cache), exist_ok=True)
        with open(cache, "w", newline="") as f:
            w = csv.writer(f)
            w.writerow(head)
            w.writerows(rows)
    head, rows = _csv_rows(cache)
    cols = {}
    for j, h in enumerate(head):
        if h == "session_swing":
            continue
        cols[h] = np.array([float(r[j]) if r[j] not in ("", "NA", "nan") else np.nan for r in rows])
    return cols


def table_row(name, sid):
    for d in (OBP, os.path.join(OBP, "poi")):
        p = os.path.join(d, name)
        if os.path.isfile(p):
            head, rows = _csv_rows(p, sid)
            if not rows:
                raise SystemExit("swing %s is not in %s" % (sid, p))
            return dict(zip(head, rows[0]))
    raise SystemExit("no %s in %s: fetch it (this script's docstring)" % (name, OBP))


def c3d_path(sid, meta):
    """the swing's C3D: c3d/<user>/<user>_<session>_<height>_<mass>_<side>_<number>_<exit velocity x10>.c3d, found by its
    session and exit velocity (the metadata's swing number is its order in the session, not the C3D's number)."""
    user, sess = meta["user"], meta["session"]
    d = os.path.join(OBP, "c3d", user)
    if not os.path.isdir(d):
        raise SystemExit("no %s: fetch hitting_c3d.zip (this script's docstring)" % d)
    ev = int(round(float(meta["exit_velo_mph_x"]) * 10))
    hits = [f for f in sorted(os.listdir(d)) if f.startswith("%s_%s_" % (user, sess)) and f.endswith("_%d.c3d" % ev)]
    if len(hits) != 1:
        raise SystemExit("swing %s: %d C3D files in %s end in _%d.c3d, want one" % (sid, len(hits), d, ev))
    return os.path.join(d, hits[0])


def markers(path):
    import c3d
    with open(path, "rb") as f:
        r = c3d.Reader(f)
        labels = [l.strip() for l in r.point_labels]
        rate = float(r.point_rate)
        X = np.array([p[:, :4].copy() for _, p, _ in r.read_frames()])
    if (X[:, :, 3] < 0).any():
        bad = sorted({labels[k] for k in np.nonzero((X[:, :, 3] < 0).any(axis=0))[0]})
        raise SystemExit("%s has gaps in %s: choose another swing" % (path, ", ".join(bad)))
    return {l: X[:, k, :3].astype(float) for k, l in enumerate(labels)}, rate


# ---- small vector helpers (rows are samples) ----------------------------------------------------------------------------
def unit(v):
    return v / np.linalg.norm(v, axis=-1, keepdims=True)


def across(v, a):
    """the part of v square to the unit axis a, made unit."""
    return unit(v - a * np.sum(v * a, axis=-1, keepdims=True))


def dot(a, b):
    return np.sum(a * b, axis=-1)


def quat(x, y, z):
    """(w, x, y, z) of the turn whose columns are the unit axes x, y, z (rows are samples), sign kept continuous."""
    R = np.stack([x, y, z], axis=-1)                    # n x 3 x 3, columns
    q = np.zeros((len(R), 4))
    for i, m in enumerate(R):
        t = m[0, 0] + m[1, 1] + m[2, 2]
        if t > 0:
            s = math.sqrt(t + 1.0) * 2
            q[i] = (0.25 * s, (m[2, 1] - m[1, 2]) / s, (m[0, 2] - m[2, 0]) / s, (m[1, 0] - m[0, 1]) / s)
        elif m[0, 0] > m[1, 1] and m[0, 0] > m[2, 2]:
            s = math.sqrt(1.0 + m[0, 0] - m[1, 1] - m[2, 2]) * 2
            q[i] = ((m[2, 1] - m[1, 2]) / s, 0.25 * s, (m[0, 1] + m[1, 0]) / s, (m[0, 2] + m[2, 0]) / s)
        elif m[1, 1] > m[2, 2]:
            s = math.sqrt(1.0 + m[1, 1] - m[0, 0] - m[2, 2]) * 2
            q[i] = ((m[0, 2] - m[2, 0]) / s, (m[0, 1] + m[1, 0]) / s, 0.25 * s, (m[1, 2] + m[2, 1]) / s)
        else:
            s = math.sqrt(1.0 + m[2, 2] - m[0, 0] - m[1, 1]) * 2
            q[i] = ((m[1, 0] - m[0, 1]) / s, (m[0, 2] + m[2, 0]) / s, (m[1, 2] + m[2, 1]) / s, 0.25 * s)
        if i and np.dot(q[i], q[i - 1]) < 0:
            q[i] = -q[i]
    return q


def frame_zx(z, xhint):
    """unit axes (x, y, z) with z given and x as near xhint as can be."""
    z = unit(z)
    x = across(xhint, z)
    return x, np.cross(z, x), z


def head_frame(lf, rf, lb, rb):
    """(x forward, y left, z up) of the head from its four markers (rows are samples)."""
    hf = (lf + rf) / 2 - (lb + rb) / 2
    hl = (lf + lb) / 2 - (rf + rb) / 2
    return frame_zx(np.cross(hf, hl), hf)


def head_offset(static_path):
    """the turn from the head markers' frame to the head's own, in the markers' frame (3 x 3), from the standing
    calibration trial: there the head is taken as level and facing as the pelvis does (forward from its back markers to
    its front ones, made level)."""
    M, _ = markers(static_path)
    H = np.stack(head_frame(M["LFHD"], M["RFHD"], M["LBHD"], M["RBHD"]), axis=-1).mean(axis=0)
    u_, _, vt = np.linalg.svd(H)
    H = u_ @ vt                                                       # the mean of the trial's frames, made a turn again
    pf = (M["LASI"] + M["RASI"] - M["LPSI"] - M["RPSI"]).mean(axis=0)
    x = np.array([pf[0], pf[1], 0.0]) / math.hypot(pf[0], pf[1])
    z = np.array([0.0, 0.0, 1.0])
    T = np.stack([x, np.cross(z, x), z], axis=-1)
    return H.T @ T


# ---- the swing ------------------------------------------------------------------------------------------------------------
LAB_TO_SIM = np.array([[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]])


class Swing:
    """one recorded swing read and mapped onto the figure. Arrays are samples x 3 in the sim frame (m) unless named rec_*
    (his own recorded points, scaled and placed the same way, for comparison)."""

    def __init__(self, sid=DEFAULT_SWING):
        self.sid = sid
        self.meta = table_row("metadata.csv", sid)
        if self.meta["hitter_side"] != "R":
            raise SystemExit("swing %s is a left-hander's: this script maps right-handers only" % sid)
        self.hit = table_row("hittrax.csv", sid)
        self.lm = lm = landmarks(sid)
        self.c3d = c3d_path(sid, self.meta)
        M, self.rate = markers(self.c3d)
        static = os.path.join(os.path.dirname(self.c3d), "%s_%s_model.c3d" % (self.meta["user"], self.meta["session"]))
        if not os.path.isfile(static):
            raise SystemExit("no standing calibration trial %s (the head's offset is read from it)" % static)
        self.head_offset = head_offset(static)
        self.static = static
        n = len(lm["time"])
        if any(len(v) != n for v in M.values()):
            raise SystemExit("the C3D has %d samples, landmarks.csv %d" % (len(next(iter(M.values()))), n))
        if abs(self.rate - 360.0) > 1e-6 or np.max(np.abs(np.diff(lm["time"]) - 1 / 360.0)) > 2e-4:
            raise SystemExit("not 360 Hz throughout")
        self.tc = float(lm["contact_time"][0])
        self.t = lm["time"] - self.tc
        self.ic = int(np.argmin(np.abs(self.t)))
        self.ev_lab = {"front_foot_down": float(lm["fp_10_time"][0]) - self.tc, "foot_plant": float(lm["fp_100_time"][0]) - self.tc}
        self.i_plant = int(np.argmin(np.abs(self.t - self.ev_lab["foot_plant"])))
        J = lambda k: np.stack([lm[k + "_x"], lm[k + "_y"], lm[k + "_z"]], axis=1)
        # his own points, lab frame
        mk = {k: v for k, v in M.items()}
        # Harrington 2007 hip centres from the four pelvis markers (the dataset's left_hip/right_hip are 0.31 m apart:
        # pelvis landmarks, not hip centres)
        O = (mk["LASI"] + mk["RASI"]) / 2
        P = (mk["LPSI"] + mk["RPSI"]) / 2
        zr = unit(mk["RASI"] - mk["LASI"])                      # to his right
        xa = across(O - P, zr)                                   # forward
        ys = np.cross(zr, xa)                                    # up
        PW = np.linalg.norm(mk["RASI"] - mk["LASI"], axis=1)[:, None]
        PD = np.linalg.norm(O - P, axis=1)[:, None]
        hjc = lambda sg: O + xa * (-0.24 * PD - 0.0099) + ys * (-0.30 * PW - 0.0109) + zr * (sg * (0.33 * PW + 0.0073))
        lab = {
            "hip_L": hjc(-1.0), "hip_R": hjc(1.0),
            "knee_L": J("lkjc"), "knee_R": J("rkjc"), "ankle_L": J("lajc"), "ankle_R": J("rajc"),
            "shoulder_L": J("lsjc"), "shoulder_R": J("rsjc"), "elbow_L": J("lejc"), "elbow_R": J("rejc"),
            "wrist_L": J("lwjc"), "wrist_R": J("rwjc"), "hand_L": J("lhjc"), "hand_R": J("rhjc"),
            "sweet": J("sweet_spot"), "thorax_prox": J("thorax_prox"),
            "pelvis_fwd": xa, "pelvis_up": ys,
        }
        for k in ("LKNE", "LMKNE", "RKNE", "RMKNE", "LANK", "LMANK", "RANK", "RMANK", "LELB", "LMELB", "RELB", "RMELB",
                  "LWRA", "LWRB", "RWRA", "RWRB", "LHEE", "LTOE", "RHEE", "RTOE", "LFHD", "RFHD", "LBHD", "RBHD",
                  "LASI", "RASI", "LPSI", "RPSI", "C7", "CLAV", "STRN", "T10", "Marker1", "Marker2", "Marker3", "LFIN", "RFIN"):
            lab[k] = mk[k]
        lab["bat_far"] = (mk["Marker2"] + mk["Marker3"]) / 2
        self.lab = lab
        self.model = figure_lengths()
        self.height_in = float(self.meta["session_height_in"])
        # 2. one scale from the legs
        leg = lambda sd: (np.median(np.linalg.norm(lab["hip_" + sd] - lab["knee_" + sd], axis=1))
                          + np.median(np.linalg.norm(lab["knee_" + sd] - lab["ankle_" + sd], axis=1)))
        self.his_leg = 0.5 * (leg("L") + leg("R"))
        self.s = s = (self.model["thigh"] + self.model["shank"]) / self.his_leg
        self.fig = F = self.his_lengths()
        # 1-3. lab to sim, scaled about the floor, placed by HitTrax's point of impact
        rot = lambda v: v @ LAB_TO_SIM.T
        sweet_c = rot(lab["sweet"][self.ic]) * s
        self.poi = np.array([float(self.hit["poi_x"]), float(self.hit["poi_z"]), float(self.hit["poi_y"])]) * IN
        # HitTrax: poi_x + toward a left-hander (the sim's +X), poi_z from the plate's back tip toward the pitcher, poi_y height
        self.shift = np.array([self.poi[0] - sweet_c[0], self.poi[1] - sweet_c[1], 0.0])
        self.place = lambda v: rot(v) * s + self.shift
        self.vec = lambda v: rot(v)                              # directions: turned, not scaled
        rec = {k: self.place(v) for k, v in lab.items() if k not in ("pelvis_fwd", "pelvis_up")}
        rec["pelvis_fwd"], rec["pelvis_up"] = self.vec(lab["pelvis_fwd"]), self.vec(lab["pelvis_up"])
        self.rec = rec
        self.build()

    def his_lengths(self):
        """his segment lengths as the markers give them, scaled: (median, lowest, highest) over the swing, m."""
        lab, s = self.lab, self.s
        L = {}
        for sd in "LR":
            for nm, a, b in (("thigh", "hip", "knee"), ("shank", "knee", "ankle"), ("uarm", "shoulder", "elbow"), ("farm", "elbow", "wrist"), ("hand", "wrist", "hand")):
                v = np.linalg.norm(lab[a + "_" + sd] - lab[b + "_" + sd], axis=1) * s
                L[nm + "_" + sd] = (float(np.median(v)), float(v.min()), float(v.max()))
        for nm, a, b in (("hips", "hip_L", "hip_R"), ("shoulders", "shoulder_L", "shoulder_R")):
            v = np.linalg.norm(lab[a] - lab[b], axis=1) * s
            L[nm] = (float(np.median(v)), float(v.min()), float(v.max()))
        return L

    # 4-5. the figure
    def build(self):
        r = self.rec
        out = {}
        # his own joint centres, scaled and placed: hips (Harrington), knees, ankles, elbows, wrists; his hands' centres
        for k in ("hip_L", "hip_R", "knee_L", "knee_R", "ankle_L", "ankle_R", "elbow_L", "elbow_R", "wrist_L", "wrist_R"):
            out[k] = r[k]
        # the pelvis, rigid: its axis fixed in his pelvis's own frame (its forward and up from the four pelvis markers), aimed at
        # his shoulders' midpoint at the first sample; the lumbar joint (he has no marker there) up it at the model's share of
        # his hips-to-shoulders distance (de Leva's lower trunk over lower + middle + 0.8 upper trunk). Harrington's hips are
        # symmetric in that frame, so the axis is square to the hips' line.
        hm = (r["hip_L"] + r["hip_R"]) / 2
        sm = (r["shoulder_L"] + r["shoulder_R"]) / 2
        pf, pu = r["pelvis_fwd"], r["pelvis_up"]
        up0 = unit(sm[0] - hm[0])
        ax_p = unit(pf * np.dot(up0, pf[0]) + pu * np.dot(up0, pu[0]))
        share = self.model["pelvis"] / (self.model["pelvis"] + self.model["trunk"])
        self.pelvis_len = share * float(np.median(np.linalg.norm(sm - hm, axis=1)))
        out["lumbar"] = hm + ax_p * self.pelvis_len
        left_p = across(r["hip_L"] - r["hip_R"], ax_p)
        self.pelvis_frame = (np.cross(left_p, ax_p), left_p, ax_p)
        # the trunk from the lumbar joint to his shoulders' midpoint; his shoulders that far apart along his shoulder line
        # squared to the trunk's axis (the viewer's chest piece can stretch but not shear: the most a shoulder moves is
        # reported)
        ax_t = unit(sm - out["lumbar"])
        left_t = across(r["shoulder_L"] - r["shoulder_R"], ax_t)
        half = np.linalg.norm(r["shoulder_L"] - r["shoulder_R"], axis=1)[:, None] / 2
        out["shoulder_L"], out["shoulder_R"] = sm + left_t * half, sm - left_t * half
        self.shoulder_move = np.maximum(np.linalg.norm(out["shoulder_L"] - r["shoulder_L"], axis=1), np.linalg.norm(out["shoulder_R"] - r["shoulder_R"], axis=1))
        self.trunk_frame = (np.cross(left_t, ax_t), left_t, ax_t)
        fwd = dot(self.pelvis_frame[0], r["pelvis_fwd"])
        chest = dot(self.trunk_frame[0], (r["CLAV"] + r["STRN"]) / 2 - (r["C7"] + r["T10"]) / 2)
        if (fwd <= 0).any() or (chest <= 0).any():
            raise SystemExit("the pelvis's or the trunk's forward points behind him at %d samples" % int((fwd <= 0).sum() + (chest <= 0).sum()))
        # bat: his, placed (not scaled: his bat is the figure's)
        knob = r["Marker1"]
        axis = unit(r["bat_far"] - r["Marker1"])
        self.bat_len = float(np.median(np.linalg.norm(self.lab["bat_far"] - self.lab["Marker1"], axis=1)))
        self.sweet_u = float(np.median(dot(self.lab["sweet"] - self.lab["Marker1"], unit(self.lab["bat_far"] - self.lab["Marker1"]))))
        out["knob"], out["tip"], out["sweet"] = knob, knob + axis * self.bat_len, knob + axis * self.sweet_u
        self.bat_axis = axis
        # each hand from his wrist to his hand's centre (the dataset's), wherever that is: on the handle while he holds it
        for sd in "LR":
            out["grip_" + sd] = r["hand_" + sd]
        self.out = out
        # turns (x forward, y left, z the segment's axis from its far joint to its near one, as body3d)
        Q = {}
        Q["pelvis"] = self.pelvis_frame
        Q["trunk"] = self.trunk_frame
        for sd, sg in (("L", 1.0), ("R", -1.0)):
            # the knee's axis from its two markers, the ankle's likewise; lateral is +y for the left side, -y for the right
            lat_k = sg * (r["LKNE" if sd == "L" else "RKNE"] - r["LMKNE" if sd == "L" else "RMKNE"])
            lat_a = sg * (r["LANK" if sd == "L" else "RANK"] - r["LMANK" if sd == "L" else "RMANK"])
            lat_e = sg * (r["LELB" if sd == "L" else "RELB"] - r["LMELB" if sd == "L" else "RMELB"])
            radial = r["LWRA" if sd == "L" else "RWRA"] - r["LWRB" if sd == "L" else "RWRB"]   # toward the thumb
            for seg, a, b, lat in (("thigh", "hip", "knee", lat_k), ("shank", "knee", "ankle", lat_a), ("uarm", "shoulder", "elbow", lat_e)):
                z = unit(out[a + "_" + sd] - out[b + "_" + sd])
                y = across(lat, z)                                   # the body's left
                Q[seg + "_" + sd] = (np.cross(y, z), y, z)
            z = unit(out["elbow_" + sd] - out["wrist_" + sd])
            Q["farm_" + sd] = frame_zx(z, radial)                    # x toward the thumb
            heel, toe = r["LHEE" if sd == "L" else "RHEE"], r["LTOE" if sd == "L" else "RTOE"]
            # heel to toe from his two foot markers (heading and pitch); no roll: the foot's left kept level. (A roll from
            # the ankle's centre against that line was 40 degrees off in the stance; one from the ankle's two markers is the
            # shin's sideways lean, which the flat foot under it does not share.)
            x = unit(toe - heel)
            y = unit(np.cross(np.array([0.0, 0.0, 1.0]), x))
            Q["foot_" + sd] = (x, y, np.cross(x, y))
        # the bat: z knob to tip, x toward the wrists' side (body3d's -x points to the wrists)
        wmid = (out["wrist_L"] + out["wrist_R"]) / 2 - out["knob"]
        Q["bat"] = frame_zx(axis, -wmid)
        # the head: its four markers' frame (forward from the back pair to the front pair, up square to that and to the
        # left), set right by the headband's own tilt and turn on his head, read from his standing calibration trial (the
        # dataset's <user>_<session>_model.c3d), where he stands looking straight ahead: there the head's true frame is
        # taken as level and facing as his pelvis faces (Plug-in Gait's head offset does the same)
        Hm = np.stack(head_frame(r["LFHD"], r["RFHD"], r["LBHD"], r["RBHD"]), axis=-1)            # n x 3 x 3, columns
        Q["head"] = tuple((Hm @ self.head_offset)[:, :, k] for k in range(3))
        self.Q = Q
        # the feet's soles. The heel's under his heel marker by that marker's height when the foot is flat (the 10th
        # percentile of its height over the swing). The ball of the foot is where the viewer's shoe has it for his ankle's
        # height above that sole: SHOE_BALL_PER_ANKLE times it ahead of the ankle along his foot, so the shoe is stretched
        # alike forward and down (an Unreal piece cannot be sheared; the viewer's turn check holds it to 3 degrees).
        self.sole = {}
        for sd in "LR":
            fx, fy, fz = Q["foot_" + sd]
            heel = r["LHEE" if sd == "L" else "RHEE"]
            h_heel = np.percentile(heel[:, 2], 10)
            hs = heel - fz * h_heel
            h_ank = dot(out["ankle_" + sd] - hs, fz)[:, None]
            ball = out["ankle_" + sd] - fz * h_ank + fx * (SHOE_BALL_PER_ANKLE * h_ank)
            self.sole[sd] = (hs, ball, h_heel, h_ank)
        # the sweet spot's speed
        dt = 1.0 / self.rate
        self.sweet_speed = np.linalg.norm(np.gradient(out["sweet"], dt, axis=0), axis=1)
        self.events = self._events()
        self.ball = self._ball()

    def _events(self):
        r, t = self.rec, self.t
        ev = [("start", t[0], "the recording's first sample")]
        lh = r["LHEE"][:, 2]
        base = np.median(lh[: max(10, int(0.2 * self.rate))])
        up = np.nonzero(lh > base + 0.02)[0]
        if len(up):
            ev.append(("lead_heel_up", t[up[0]], "his lead heel marker first 2 cm above where it stood"))
        la = r["ankle_L"][:, 2]
        k = int(np.argmax(la[: self.ic]))
        ev.append(("leg_lift_top", t[k], "his lead ankle at its highest"))
        ev.append(("front_foot_down", self.ev_lab["front_foot_down"], "the dataset's fp_10: the lead foot carries 10 % of his weight"))
        ev.append(("foot_plant", self.ev_lab["foot_plant"], "the dataset's fp_100: the lead foot carries his whole weight"))
        # the hands start, by the force exporter's rule: the bat's point 0.15 m from the knob last rises through a tenth of its
        # fastest before contact
        hp = self.out["knob"] + self.bat_axis * 0.15
        hs = np.linalg.norm(np.gradient(hp, 1.0 / self.rate, axis=0), axis=1)
        kpk = int(np.argmax(hs[: self.ic + 1]))
        k = kpk
        while k > 0 and hs[k] > 0.1 * hs[kpk]:
            k -= 1
        ev.append(("launch", t[k], "the hands start: the bat's point 0.15 m from the knob last rises through a tenth of its fastest before contact"))
        ev.append(("contact", 0.0, "the dataset's contact instant"))
        ev.append(("end", t[-1], "the recording's last sample"))
        return sorted(ev, key=lambda e: e[1])

    def _ball(self):
        """the ball's centre at every sample: the machine pitch, then the batted ball (HitTrax's figures, gravity only)."""
        out, t, ic = self.out, self.t, self.ic
        vs = np.gradient(out["sweet"], 1.0 / self.rate, axis=0)[ic]
        a = self.bat_axis[ic]
        face = vs - a * np.dot(vs, a)
        face /= np.linalg.norm(face)
        self.bat_r = 0.0333                                    # a 34-in wood bat's barrel, 2.62 in across [E]
        self.ball_r = 0.0366
        pc = out["sweet"][ic] + face * (self.bat_r + self.ball_r)
        # the exact contact instant lies between samples; the ball meets the face there: take the sample's own
        sp = float(self.hit["pitch"]) * MPH
        dd = math.radians(float(self.hit["pitch_angle"]))      # negative: coming down
        vin = np.array([0.0, -sp * math.cos(dd), sp * math.sin(dd)])
        release_y = 40 * 12 * IN                               # the machine, about 40 ft from the plate (the dataset's README)
        t_rel = -(release_y - pc[1]) / sp / math.cos(dd)
        ev = float(self.hit.get("exit_velo_mph_x") or self.meta["exit_velo_mph_x"]) * MPH
        la = math.radians(float(self.hit["la"]))
        sp_ = math.radians(float(self.hit["bearing"]))          # 0 at the pitcher; negative to a right-hander's pull side (-X)
        vout = ev * np.array([math.cos(la) * math.sin(sp_), math.cos(la) * math.cos(sp_), math.sin(la)])
        b = np.zeros((len(t), 3))
        for i, ti in enumerate(t):
            if ti <= 0:
                tt = max(ti, t_rel)
                b[i] = pc + vin * tt + np.array([0, 0, -0.5 * G * tt * tt])
            else:
                b[i] = pc + vout * ti + np.array([0, 0, -0.5 * G * ti * ti])
        self.ball_at_contact, self.ball_release_t = pc, t_rel
        self.ball_in, self.ball_out = vin, vout
        return b

    # ---- what the mapping did ----
    def report(self, log=print):
        F, r, o = self.fig, self.rec, self.out
        L = lambda a, b: np.linalg.norm(a - b, axis=1)
        log("swing %s: %s, %s in, %s lb, %s; bat %s in %s oz; max bat speed %.1f mph, exit velocity %s mph, launch %s deg, spray %s deg, zone %s"
            % (self.sid, os.path.basename(self.c3d), self.meta["session_height_in"], self.meta["session_mass_lbs"], self.meta["highest_playing_level"],
               self.meta["bat_length_in"], self.meta["bat_weight_oz"], float(self.meta["bat_speed_mph_max_x"]), self.meta["exit_velo_mph_x"],
               self.hit["la"], self.hit["bearing"], self.hit["strike_zone"]))
        log("samples %d at %.0f Hz, %.0f ms before contact to %.0f ms after" % (len(self.t), self.rate, -1e3 * self.t[0], 1e3 * self.t[-1]))
        log("scale s = %.4f (the body model's leg %.4f m over his %.4f m); his height %.3f m -> %.3f m on the figure; the model is %.2f m"
            % (self.s, self.model["thigh"] + self.model["shank"], self.his_leg, self.height_in * IN, self.height_in * IN * self.s, FIG_H))
        lab = self.lab
        for k, (m_, lo, hi) in F.items():
            mk = {"thigh": "thigh", "shank": "shank", "uarm": "uarm", "farm": "farm", "hips": "hip_w", "shoulders": "shoulder_w", "hand": "hand"}[k.split("_")[0]]
            log("  %-10s his (scaled) median %.3f [%.3f-%.3f] m, drawn as recorded; body model %.3f" % (k, m_, lo, hi, self.model[mk]))
        for sd in "LR":
            ax = unit(lab["bat_far"] - lab["Marker1"])
            along = dot(lab["hand_" + sd] - lab["Marker1"], ax)
            off = np.linalg.norm(lab["hand_" + sd] - lab["Marker1"] - ax * along[:, None], axis=1)
            for lab_, sl in (("to contact", slice(0, self.ic + 1)), ("after", slice(self.ic + 1, None))):
                log("  hand_%s %-10s: his hand's centre %.3f-%.3f m from the knob along the bat, %.3f-%.3f m off its axis"
                    % (sd, lab_, along[sl].min(), along[sl].max(), off[sl].min(), off[sl].max()))
        log("pelvis: lumbar joint %.3f m up the pelvis's axis; shoulders squared to the trunk's axis: moved %.1f mm at most (median %.1f, at %+.0f ms)"
            % (self.pelvis_len, 1e3 * self.shoulder_move.max(), 1e3 * np.median(self.shoulder_move), 1e3 * self.t[int(np.argmax(self.shoulder_move))]))
        log("bat: end markers %.4f m apart; sweet spot %.4f m from the knob marker" % (self.bat_len, self.sweet_u))
        for k in ("hip_L", "hip_R", "knee_L", "knee_R", "ankle_L", "ankle_R", "shoulder_L", "shoulder_R", "elbow_L", "elbow_R", "wrist_L", "wrist_R"):
            d = L(o[k], r[k])
            log("  figure's %-10s from his (scaled): median %5.1f mm, max %5.1f mm" % (k, 1e3 * np.median(d), 1e3 * d.max()))
        log("placed: shift %+.3f %+.3f m; HitTrax point of impact x %+.3f y %.3f z %.3f m; his sweet spot at contact (scaled) z %.3f m"
            % (self.shift[0], self.shift[1], self.poi[0], self.poi[1], self.poi[2], o["sweet"][self.ic][2]))
        log("feet at the start: lead ankle x %+.3f y %+.3f, rear ankle x %+.3f y %+.3f (the plate's edge is x = +-0.216, the box's inner line -0.368)"
            % (o["ankle_L"][0][0], o["ankle_L"][0][1], o["ankle_R"][0][0], o["ankle_R"][0][1]))
        for sd in "LR":
            hs, ball, hh, ha = self.sole[sd]
            log("  foot_%s: ankle %.3f-%.3f m above the sole; lowest sole point z: heel %+.3f, ball %+.3f m; ball of the foot moves %.1f mm (heel %.1f) from foot plant on"
                % (sd, ha.min(), ha.max(), hs[:, 2].min(), ball[:, 2].min(), 1e3 * np.ptp(ball[self.i_plant:], axis=0).max(), 1e3 * np.ptp(hs[self.i_plant:], axis=0).max()))
        log("sweet spot speed at contact %.1f m/s (%.1f mph), fastest %.1f mph at %+.0f ms"
            % (self.sweet_speed[self.ic], self.sweet_speed[self.ic] / MPH, self.sweet_speed.max() / MPH, 1e3 * self.t[int(np.argmax(self.sweet_speed))]))
        for e in self.events:
            log("  event %-15s %+7.1f ms  %s" % (e[0], 1e3 * e[1], e[2]))

    # ---- the viewer's file ----
    BODIES = ("pelvis", "trunk", "thigh_L", "shank_L", "foot_L", "thigh_R", "shank_R", "foot_R",
              "uarm_L", "farm_L", "uarm_R", "farm_R", "bat", "head")
    JOINTS = ("lumbar", "hip_L", "knee_L", "ankle_L", "hip_R", "knee_R", "ankle_R",
              "shoulder_L", "elbow_L", "wrist_L", "shoulder_R", "elbow_R", "wrist_R")
    POINTS = ("knob", "sweet", "tip", "grip_L", "grip_R")
    CONTACTS = ("L_heel_in", "L_heel_out", "L_ball_in", "L_ball_out", "R_heel_in", "R_heel_out", "R_ball_in", "R_ball_out")
    SOLE_HALF_W = 0.04                                       # m, half the sole's width (body3d's SOLE_HALF_W) [E]

    def body_pos(self, name):
        o, r = self.out, self.rec
        mids = {"pelvis": ("hip_L", "hip_R", "lumbar"), "trunk": ("lumbar", "shoulder_L", "shoulder_R"), "bat": ("knob", "tip"),
                "hand_L": ("wrist_L", "grip_L"), "hand_R": ("wrist_R", "grip_R")}
        if name in mids:
            return sum(o[k] for k in mids[name]) / len(mids[name])
        if name == "head":
            return (r["LFHD"] + r["RFHD"] + r["LBHD"] + r["RBHD"]) / 4
        if name.startswith("foot_"):
            sd = name[-1]
            return (self.sole[sd][0] + self.sole[sd][1]) / 2
        seg, sd = name.split("_")
        a, b = {"thigh": ("hip", "knee"), "shank": ("knee", "ankle"), "uarm": ("shoulder", "elbow"), "farm": ("elbow", "wrist")}[seg]
        return (o[a + "_" + sd] + o[b + "_" + sd]) / 2

    def contacts(self):
        cols = []
        for sd in "LR":
            fx, fy, fz = self.Q["foot_" + sd]
            heel, ball = self.sole[sd][0], self.sole[sd][1]
            sg = 1.0 if sd == "L" else -1.0
            for p in (heel, ball):
                for side in (-sg, sg):                       # in (toward the other foot), out
                    cols.append(p + fy * (side * self.SOLE_HALF_W))
        return cols

    def write(self, out=None, log=print):
        out = out or os.path.join(OUT_DIR, self.sid + ".swing")
        os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
        o = self.out
        con = self.contacts()
        quats = {b: quat(*self.Q[b]) for b in self.BODIES}
        rows = []
        for i in range(len(self.t)):
            row = [self.t[i]]
            for b in self.BODIES:
                row += list(self.body_pos(b)[i]) + list(quats[b][i])
            for j in self.JOINTS:
                row += list(o[j][i])
            for p in self.POINTS:
                row += list(o[p][i])
            row.append(self.sweet_speed[i])
            for c in con:
                row += list(c[i]) + [0.0]
            row += list(self.ball[i])
            rows.append(row)
        F = self.fig
        m = self.meta
        w = []
        w.append("# %s" % FORMAT)
        w.append("# A REAL HITTER'S RECORDED SWING put on the batter's figure (PLAN D45), by analysis/research/body/real_swing.py, for the")
        w.append("# Unreal viewer (client/run.sh -BbBodySwing=<file>). A VIEWING AID: no game reads it.")
        w.append("# Source: OpenBiomechanics Project (Driveline), baseball_hitting, dataset-v1, CC BY-NC-SA 4.0: non-commercial; this file")
        w.append("# is derived from it and under the same licence. Keep it out of git and out of anything shipped.")
        w.append("# Sim frame (Statcast): metres, +X toward first base, +Y toward the pitcher, +Z up; a right-handed batter on the third-base")
        w.append("# side (-X). L is his lead (left) side. Quaternions (w, x, y, z): world = R(q) local. Times are seconds from contact.")
        w.append("# The mapping (real_swing.py's docstring; docs/research/swing/12): his joint centres, scaled by `scale` about the floor and")
        w.append("# placed by `placed`; his own segment lengths at every sample. Each body's x is its forward (the forearm's and hand's: toward the thumb),")
        w.append("# z its axis; the foot's x heel to toe; the bat's z knob to tip; the head's x the face's forward. Contacts carry no load (0).")
        w.append("format %s" % FORMAT)
        w.append("kind recorded")
        w.append("title A REAL HITTER'S SWING ON THE FIGURE (PLAN D45): OpenBiomechanics swing %s, %s in, %s, %s mph exit velocity (CC BY-NC-SA)"
                 % (self.sid, m["session_height_in"], m["highest_playing_level"].upper() if m["highest_playing_level"] == "milb" else m["highest_playing_level"], m["exit_velo_mph_x"]))
        w.append("plan OBP_%s" % self.sid)
        w.append("tree %s" % ROOT)
        w.append("commit %s" % _git("rev-parse", "--short", "HEAD"))
        w.append("source OpenBiomechanics baseball_hitting dataset-v1 %s (CC BY-NC-SA 4.0)" % os.path.basename(self.c3d))
        w.append("hitter height_in %s mass_lb %s level %s side %s bat_in %s bat_oz %s max_bat_speed_mph %s exit_velo_mph %s"
                 % (m["session_height_in"], m["session_mass_lbs"], m["highest_playing_level"], m["hitter_side"], m["bat_length_in"], m["bat_weight_oz"],
                    m["bat_speed_mph_max_x"], m["exit_velo_mph_x"]))
        w.append("hittrax pitch_mph %s pitch_angle_deg %s launch_deg %s spray_deg %s zone %s poi_in %s %s %s"
                 % (self.hit["pitch"], self.hit["pitch_angle"], self.hit["la"], self.hit["bearing"], self.hit["strike_zone"], self.hit["poi_x"], self.hit["poi_y"], self.hit["poi_z"]))
        w.append("scale %.5f    # his leg (thigh + shank, median) to the figure's" % self.s)
        w.append("placed %.5f %.5f 0    # added after the scale (m): the sweet spot at contact to HitTrax's point of impact" % tuple(self.shift[:2]))
        w.append("batter_height_m %.4f" % FIG_H)
        w.append("figure " + " ".join("%s %.4f" % (k, v[0]) for k, v in sorted(F.items())) + "    # his segments' median lengths (m, scaled); each sample draws them as recorded")
        w.append("bat length_m %.4f sweet_from_knob_m %.4f    # his bat's end markers and the dataset's sweet spot" % (self.bat_len, self.sweet_u))
        w.append("contact_speed_mps %.4f" % self.sweet_speed[self.ic])
        w.append("contact_speed_mph %.2f" % (self.sweet_speed[self.ic] / MPH))
        w.append("ball radius_m %.4f pitch_mps %.3f release_t_s %.4f exit_mps %.3f    # touching the barrel's face at contact; drawn after it"
                 % (self.ball_r, np.linalg.norm(self.ball_in), self.ball_release_t, np.linalg.norm(self.ball_out)))
        w.append("ball_after_contact shown")
        for e in self.events:
            w.append("event %s %.6f    # %s" % e)
        w.append("rate_hz %.0f" % self.rate)
        w.append("bodies %d %s" % (len(self.BODIES), " ".join(self.BODIES)))
        w.append("joints %d %s" % (len(self.JOINTS), " ".join(self.JOINTS)))
        w.append("points %d %s" % (len(self.POINTS), " ".join(self.POINTS)))
        w.append("contacts %d %s" % (len(self.CONTACTS), " ".join(self.CONTACTS)))
        w.append("# each sample row: s t | bodies x y z qw qx qy qz | joints x y z | points x y z | sweet_speed_mps | contacts x y z fz_N | ball x y z")
        w.append("columns %d" % (1 + 7 * len(self.BODIES) + 3 * len(self.JOINTS) + 3 * len(self.POINTS) + 1 + 4 * len(self.CONTACTS) + 3))
        w.append("samples %d" % len(rows))
        for row in rows:
            w.append("s " + " ".join("%.7g" % x if abs(x) >= 1e-3 or x == 0.0 else "%.6e" % x for x in row))
        with open(out, "w") as f:
            f.write("\n".join(w) + "\n")
        log("wrote %s: %d samples, %.0f ms before contact to %.0f ms after" % (os.path.relpath(out, ROOT), len(rows), -1e3 * self.t[0], 1e3 * self.t[-1]))
        return out


def _git(*a):
    import subprocess
    try:
        return subprocess.run(["git", "-C", ROOT] + list(a), capture_output=True, text=True, check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def main():
    a = sys.argv[1:]
    arg = lambda k, d=None: a[a.index(k) + 1] if k in a else d
    pos = [x for j, x in enumerate(a) if not x.startswith("--") and (j == 0 or a[j - 1] not in ("--out", "--draw", "--every-ms"))]
    sid = pos[0] if pos else DEFAULT_SWING
    if "--print-out" in a:
        print(arg("--out") or os.path.join(OUT_DIR, sid + ".swing"))
        return
    sw = Swing(sid)
    if "--report" in a:
        sw.report()
        return
    if "--draw" in a:
        import real_swing_draw
        real_swing_draw.draw(sw, arg("--draw"), float(arg("--every-ms", "10")))
        return
    sw.write(arg("--out"))


if __name__ == "__main__":
    main()
