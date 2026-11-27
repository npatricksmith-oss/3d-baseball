"""The recorded hitter as a stick figure, through the Unreal viewer's own three cameras, at the instants of
-BbBodySwingShots=<ms> (docs/research/swing/12-real-swing-on-the-figure.md): the reference to set the viewer's pictures beside.

    uv run --with numpy --with c3d --with matplotlib python3 analysis/research/body/real_swing.py <swing id> --draw <dir> [--every-ms 10]

Draws HIS points, not the figure's: his joint centres and markers scaled and placed as the file is (real_swing.Swing.rec), the
bat from its end markers, his head from its four markers, each foot from heel to toe marker, and the file's ball. Black: his
left (lead) side and trunk; red: his right (rear) side; brown: the bat. The figure's shoulders (the only joints the file moves
off his, squared to the trunk) are drawn as small blue rings. Pictures: <dir>/stick_<nnn>_<m|p><ms>ms_<pitcher|open|above>.png,
numbered and named as the viewer's bodyswing_* pictures, so that the two sort side by side.

The cameras are client/Source/BaseballClient/BbBodySwing.cpp SetView's: the target between his hips at the first sample and the
ball at contact, at 1 m; eyes at the target + (0.3, 7.5, 0.35) m (pitcher's side, 36 deg), + (4.8, 0.5, 0.25) (open side,
52 deg), + (0, -0.4, 6) (above, 50 deg); Unreal's field of view is horizontal; 16:9.
"""
import math
import os

import numpy as np

VIEWS = (("pitcher", (0.3, 7.5, 0.35), 36.0), ("open", (4.8, 0.5, 0.25), 52.0), ("above", (0.0, -0.4, 6.0), 50.0))
W, H = 800, 450


def camera(eye, target, fov_deg):
    f = target - eye
    f /= np.linalg.norm(f)
    up = np.array([0.0, 0.0, 1.0])
    r = np.cross(f, up)
    if np.linalg.norm(r) < 1e-6:                     # looking straight down: screen up is toward the pitcher (+Y)
        r = np.cross(f, np.array([0.0, 1.0, 0.0]))
    r /= np.linalg.norm(r)
    u = np.cross(r, f)
    foc = (W / 2) / math.tan(math.radians(fov_deg) / 2)

    def proj(p):
        d = np.asarray(p) - eye
        z = d @ f
        return np.stack([W / 2 + foc * (d @ r) / z, H / 2 - foc * (d @ u) / z], axis=-1)
    return proj


def draw(sw, out_dir, every_ms=10.0):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    os.makedirs(out_dir, exist_ok=True)
    r, o, t = sw.rec, sw.out, sw.t
    hips0 = (o["hip_L"][0] + o["hip_R"][0]) / 2
    target = np.array([(hips0[0] + sw.ball_at_contact[0]) / 2, (hips0[1] + sw.ball_at_contact[1]) / 2, 1.0])
    k0 = int(math.ceil(1e3 * t[0] / every_ms - 1e-6))
    k1 = int(math.floor(1e3 * t[-1] / every_ms + 1e-6))
    cams = [(name, camera(target + np.array(off), target, fov)) for name, off, fov in VIEWS]
    at = lambda arr, tt: np.array([np.interp(tt, t, arr[:, c]) for c in range(3)])
    lines = [("hip_L", "knee_L", "k"), ("knee_L", "ankle_L", "k"), ("hip_R", "knee_R", "r"), ("knee_R", "ankle_R", "r"),
             ("hip_L", "hip_R", "k"), ("shoulder_L", "shoulder_R", "k"), ("shoulder_L", "elbow_L", "k"), ("elbow_L", "wrist_L", "k"),
             ("wrist_L", "hand_L", "k"), ("shoulder_R", "elbow_R", "r"), ("elbow_R", "wrist_R", "r"), ("wrist_R", "hand_R", "r"),
             ("LHEE", "LTOE", "k"), ("RHEE", "RTOE", "r"), ("Marker1", "bat_far", "#8b5a2b")]
    n = 0
    for k in range(k0, k1 + 1):
        tt = k * every_ms / 1e3
        n += 1
        P = {key: at(v, tt) for key, v in r.items() if isinstance(v, np.ndarray) and v.ndim == 2 and v.shape[1] == 3}
        hm, sm = (P["hip_L"] + P["hip_R"]) / 2, (P["shoulder_L"] + P["shoulder_R"]) / 2
        head = (P["LFHD"] + P["RFHD"] + P["LBHD"] + P["RBHD"]) / 4
        ball = at(sw.ball, tt)
        tag = "%s%03dms" % ("m" if k * every_ms < 0 else "p", abs(int(round(k * every_ms))))
        for name, proj in cams:
            fig = plt.figure(figsize=(W / 100, H / 100), dpi=100)
            ax = fig.add_axes([0, 0, 1, 1])
            ax.set_xlim(0, W)
            ax.set_ylim(H, 0)
            ax.axis("off")
            # the ground's lines: the plate and the box's inner line on the third-base side, for scale
            pl = np.array([[0, 0, 0], [0.216, 0.216, 0], [0.216, 0.432, 0], [-0.216, 0.432, 0], [-0.216, 0.216, 0], [0, 0, 0]])
            q = proj(pl)
            ax.plot(q[:, 0], q[:, 1], color="0.6", lw=1)
            for a, b, c in lines:
                q = proj(np.stack([P[a], P[b]]))
                ax.plot(q[:, 0], q[:, 1], color=c, lw=2.5 if c != "#8b5a2b" else 4, solid_capstyle="round")
            q = proj(np.stack([hm, sm]))
            ax.plot(q[:, 0], q[:, 1], color="k", lw=2.5)
            q = proj(np.stack([sm, head]))
            ax.plot(q[:, 0], q[:, 1], color="k", lw=1.5)
            # the head: its four markers and the face's forward
            hq = proj(np.stack([P["LFHD"], P["RFHD"], P["RBHD"], P["LBHD"], P["LFHD"]]))
            ax.plot(hq[:, 0], hq[:, 1], color="k", lw=1.5)
            fq = proj(np.stack([head, head + 0.15 * (P["LFHD"] + P["RFHD"] - P["LBHD"] - P["RBHD"]) / np.linalg.norm(P["LFHD"] + P["RFHD"] - P["LBHD"] - P["RBHD"])]))
            ax.plot(fq[:, 0], fq[:, 1], color="g", lw=1.5)
            for s_ in ("shoulder_L", "shoulder_R"):
                q = proj(at(o[s_], tt)[None, :])
                ax.plot(q[:, 0], q[:, 1], "o", mfc="none", mec="b", ms=6)
            q = proj(ball[None, :])
            ax.plot(q[:, 0], q[:, 1], "o", color="0.85", mec="0.3", ms=6)
            ax.text(8, 18, "%s  %+.0f ms  his markers (stick)" % (name, k * every_ms), fontsize=11)
            fig.savefig(os.path.join(out_dir, "stick_%03d_%s_%s.png" % (n, tag, name)))
            plt.close(fig)
    print("drew %d instants x %d views into %s" % (n, len(cams), out_dir))
