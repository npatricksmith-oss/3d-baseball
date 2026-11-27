"""Inverse dynamics of the real hitter's recorded swing on HIS OWN body (docs/body-motion.md 4.16, the design of
session 14 onward; PLAN D45-D47). A read-only probe: no search, no fit, nothing of the force-driven engine. It asks
what joint torques his recorded motion needs, joint by joint, against the measured strengths the force-driven body
is held to (PLAN D34, analysis/oracle/body3d/drive.py), and whether the dataset's force plates agree with his motion.

    uv run --with numpy --with scipy --with c3d python3 analysis/research/body/real_inverse.py [<swing id>] [--cut <Hz>]
        [--to <ms before contact, default -20>] [--com-markers] [--parts] > sim/target/codex-probes/body3d/s14/real_inverse.out

THE DATA ARE NOT IN THIS REPOSITORY (CC BY-NC-SA 4.0, PLAN D46); see real_swing.py for where they come from. What this
prints is derived from them: keep its output under sim/target, out of git.

How (each choice [E] where it is one):
- His joint centres and segment frames as real_swing.Swing reads them, at his own size (the figure's uniform scale
  undone; the bat his own). Lab frame turned to the sim's.
- Segment masses: de Leva's male shares of his 214 lb; centres of mass and radii of gyration as de Leva's shares of
  HIS measured segment lengths (median over the swing) [E: de Leva's shares on a stocky professional]. The trunk is
  de Leva's middle + upper trunk, its centre placed along the lumbar-to-shoulders line by de Leva's shares; the head
  is its own body on his four head markers. Both hands ride on the bat at his hands' centres. The bat: 31 oz, 34 in,
  the game's bat's balance point and inertia scaled to its mass [E].
- Every position is low-pass filtered (fourth-order Butterworth forward and back, --cut Hz, default 15 [E]) on the
  samples UP TO CONTACT ONLY, so the impact is not smeared back into the swing (PLAN D35); angular velocity is read
  from the filtered frames. Read nothing in the last ~15 ms before contact as more than indicative.
- Top down, no ground needed: the torque at the lumbar joint that the trunk, head, arms and bat need. The two arms
  and the bat are a closed loop, so how the bat's load is shared between the hands is not determined: the hands are
  taken as two points carrying the least forces that give the bat's needed force and moment (no wrist torque) [E];
  the elbow and shoulder torques below rest on that split.
- Bottom up, from the force plates: each plate taken as one foot's (true only from the lead foot's landing on; the
  dataset's README warns the stance may have both feet on one plate). The plates give forces only: no centre of
  pressure and no twisting moment. The centre of pressure is put mid-sole, or under the ball of the foot while the
  heel is up [E], and moved 4 cm either way as the sensitivity; the twisting moment is taken as zero [E].
- The whole body's motion against the plates: his centre of mass's acceleration times his mass, plus his weight,
  against the two plates' forces (a check of the plates and of the mass model together).
"""
import math
import os
import sys

import numpy as np
from scipy.signal import butter, filtfilt

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import real_swing as RS                      # noqa: E402
import common as C                           # noqa: E402
from body3d import drive as DR               # noqa: E402  (real_swing put analysis/oracle on the path)

G = 9.81
LB, OZ = 0.45359237, 0.028349523
T_FROM = -0.300                               # s: the stretch read from here to --to (default -20 ms)


def unit(v):
    return v / np.linalg.norm(v, axis=-1, keepdims=True)


def lowpass(x, cut, rate):
    b, a = butter(4, cut / (rate / 2.0))
    return filtfilt(b, a, x, axis=0, padtype="odd", padlen=min(3 * max(len(a), len(b)), len(x) - 1))


def frames(Q):
    """n x 3 x 3 rotation matrices, columns the body's x, y, z in the world."""
    return np.stack(Q, axis=-1)


def reorth(R):
    u, _, vt = np.linalg.svd(R)
    return u @ vt


def omega(R, dt):
    """angular velocity (world) from rotation matrices: vee(dR/dt R^T)."""
    Rd = np.gradient(R, dt, axis=0)
    W = Rd @ np.transpose(R, (0, 2, 1))
    W = 0.5 * (W - np.transpose(W, (0, 2, 1)))
    return np.stack([W[:, 2, 1], W[:, 0, 2], W[:, 1, 0]], axis=1)


class Body:
    def __init__(self, name, m, com, R, Ibody, dt):
        self.name, self.m, self.com, self.R = name, m, com, R
        self.a = np.gradient(np.gradient(com, dt, axis=0), dt, axis=0)
        self.v = np.gradient(com, dt, axis=0)
        self.w = omega(R, dt)
        Iw = R @ np.diag(Ibody) @ np.transpose(R, (0, 2, 1))
        self.H = np.einsum("nij,nj->ni", Iw, self.w)
        self.Hd = np.gradient(self.H, dt, axis=0)
        self.Iw = Iw

    def need(self):
        """(force, moment about its own centre) the rest of the world must put on it, gravity taken out."""
        F = self.m * (self.a + np.array([0.0, 0.0, G]))
        return F, self.Hd

    def ke(self):
        return 0.5 * self.m * np.sum(self.v ** 2, axis=1) + 0.5 * np.einsum("ni,ni->n", self.w, self.H)


TRUNK_COM = "joints"     # "joints": de Leva's shares along the lumbar-to-shoulders line; "markers": the thorax's and pelvis's markers


def main(argv):
    global TRUNK_COM
    if "--com-markers" in argv:
        TRUNK_COM = "markers"
    sid = next((a for a in argv if not a.startswith("--") and "_" in a), RS.DEFAULT_SWING)
    cut = float(argv[argv.index("--cut") + 1]) if "--cut" in argv else 15.0
    t_to = float(argv[argv.index("--to") + 1]) / 1e3 if "--to" in argv else -0.020   # the last instant read: the filter's end is not
    sw = RS.Swing(sid)
    s, sh = sw.s, sw.shift
    rate = sw.rate
    dt = 1.0 / rate
    ic = sw.ic
    n = ic + 1                                          # the samples to contact
    t = sw.t[:n]
    true = lambda p: (p[:n] - sh) / s                   # the figure's scale undone: his own size
    O = {k: true(v) for k, v in sw.out.items() if k not in ("tip", "sweet", "knob")}
    knob = true(sw.out["knob"])
    bax = sw.bat_axis[:n]
    O["knob"] = knob
    M = float(sw.meta["session_mass_lbs"]) * LB
    Hh = float(sw.meta["session_height_in"]) * 0.0254
    F = {k: lowpass(v, cut, rate) for k, v in O.items()}
    Q = {k: reorth(lowpass(frames(v)[:n], cut, rate)) for k, v in sw.Q.items()}
    bax = unit(lowpass(bax, cut, rate))
    lab = sw.lab
    medlen = lambda a, b: float(np.median(np.linalg.norm(lab[a][:n] - lab[b][:n], axis=1)))
    seg = lambda nm: C.DE_LEVA[nm]
    bodies = {}

    def add(name, nm, prox, dist, R, length=None, perm=(1, 0, 2)):
        L_, mf, c, rs, rt, rl = seg(nm)
        Lm = length if length is not None else float(np.median(np.linalg.norm(prox - dist, axis=1)))
        m = mf * M
        com = prox + (dist - prox) * c
        r = (rs, rt, rl)       # sagittal (about y), transverse (about x), long (about z)
        Ib = np.array([m * (Lm * rt) ** 2, m * (Lm * rs) ** 2, m * (Lm * rl) ** 2])
        if perm == "foot":     # the foot's x is its long axis
            Ib = np.array([m * (Lm * rl) ** 2, m * (Lm * rs) ** 2, m * (Lm * rt) ** 2])
        bodies[name] = Body(name, m, com, R, Ib, dt)
        return Lm

    lens = {}
    for sd in "LR":
        lens["thigh_" + sd] = add("thigh_" + sd, "thigh", F["hip_" + sd], F["knee_" + sd], Q["thigh_" + sd])
        lens["shank_" + sd] = add("shank_" + sd, "shank", F["knee_" + sd], F["ankle_" + sd], Q["shank_" + sd])
        lens["uarm_" + sd] = add("uarm_" + sd, "upperarm", F["shoulder_" + sd], F["elbow_" + sd], Q["uarm_" + sd])
        lens["farm_" + sd] = add("farm_" + sd, "forearm", F["elbow_" + sd], F["wrist_" + sd], Q["farm_" + sd])
        heel, ball = (lowpass(true(sw.sole[sd][0]), cut, rate), lowpass(true(sw.sole[sd][1]), cut, rate))
        toe = lowpass(true(lab_rot(sw, "LTOE" if sd == "L" else "RTOE")), cut, rate)
        lens["foot_" + sd] = add("foot_" + sd, "foot", heel, toe, Q["foot_" + sd], perm="foot")
        F["heel_" + sd], F["ball_" + sd] = heel, ball
    # pelvis: de Leva's lower trunk, omphalion (the lumbar joint here) down to the hips' midpoint
    hm = (F["hip_L"] + F["hip_R"]) / 2
    lens["pelvis"] = add("pelvis", "lpt", F["lumbar"], hm, Q["pelvis"])
    if TRUNK_COM == "markers":     # the pelvis's centre: its four markers' centroid [E]
        bodies["pelvis"] = Body("pelvis", bodies["pelvis"].m, lowpass(true(sum(lab_rot(sw, k) for k in ("LASI", "RASI", "LPSI", "RPSI")) / 4.0), cut, rate),
                                Q["pelvis"], np.diag(bodies["pelvis"].Iw[0]) * 0 + np.array([1.0, 1.0, 1.0]) * float(np.trace(bodies["pelvis"].Iw[0]) / 3), dt)
    # trunk: middle + upper trunk, lumbar up to the shoulders' midpoint at the model's share (mpt + 0.8 upt)
    smid = (F["shoulder_L"] + F["shoulder_R"]) / 2
    Lmpt, Lupt = seg("mpt")[0], seg("upt")[0]
    span = smid - F["lumbar"]
    scale_t = 1.0 / (Lmpt + 0.8 * Lupt)
    xiph = F["lumbar"] + span * (Lmpt * scale_t)
    cerv = F["lumbar"] + span * ((Lmpt + Lupt) * scale_t)
    lt = float(np.median(np.linalg.norm(span, axis=1)))
    Lm_ = lt * Lmpt * scale_t
    Lu_ = lt * Lupt * scale_t
    _, mm, cm_, rs, rt, rl = seg("mpt")
    _, mu_, cu_, rs2, rt2, rl2 = seg("upt")
    m_mpt, m_upt = mm * M, mu_ * M
    com_m = F["lumbar"] + (xiph - F["lumbar"]) * (1 - cm_)     # de Leva's mpt runs xiphion -> omphalion
    com_u = xiph + (cerv - xiph) * (1 - cu_)                    # upt runs cervicale -> xiphion
    com_t = (m_mpt * com_m + m_upt * com_u) / (m_mpt + m_upt)
    if TRUNK_COM == "markers":     # the centroid of his four thorax markers (C7, T10, clavicle, sternum) [E]
        com_t = lowpass(true(sum(lab_rot(sw, k) for k in ("C7", "T10", "CLAV", "STRN")) / 4.0), cut, rate)
    d_m = np.median(np.linalg.norm(com_m - com_t, axis=1)); d_u = np.median(np.linalg.norm(com_u - com_t, axis=1))
    Ib = np.array([m_mpt * ((Lm_ * rt) ** 2 + d_m ** 2) + m_upt * ((Lu_ * rt2) ** 2 + d_u ** 2),
                   m_mpt * ((Lm_ * rs) ** 2 + d_m ** 2) + m_upt * ((Lu_ * rs2) ** 2 + d_u ** 2),
                   m_mpt * (Lm_ * rl) ** 2 + m_upt * (Lu_ * rl2) ** 2])
    bodies["trunk"] = Body("trunk", m_mpt + m_upt, com_t, Q["trunk"], Ib, dt)
    lens["trunk"] = lt
    # head: its markers' centroid, de Leva's head on his height's share
    hd = lowpass(true(sum(lab_rot(sw, k) for k in ("LFHD", "RFHD", "LBHD", "RBHD")) / 4.0), cut, rate)
    Lh, mh_, ch, rs, rt, rl = seg("head")
    Lh = Lh * Hh / C.DE_LEVA_H
    bodies["head"] = Body("head", mh_ * M, hd, reorth(lowpass(frames(sw.Q["head"])[:n], cut, rate)),
                          np.array([mh_ * M * (Lh * rt) ** 2, mh_ * M * (Lh * rs) ** 2, mh_ * M * (Lh * rl) ** 2]), dt)
    # bat with both hands: 31 oz, the game's bat's balance point and inertias scaled to its mass [E]
    mb = float(sw.meta["bat_weight_oz"]) * OZ
    mhand = seg("hand")[1] * M
    k_b = mb / C.BAT_M
    gL, gR = F["grip_L"], F["grip_R"]
    ucm_b = knob + bax * C.BAT_UCM
    mt = mb + 2 * mhand
    com_b = (mb * ucm_b + mhand * (gL + gR)) / mt
    Rb = Q["bat"]
    It_b = C.BAT_ICM * k_b
    dL = np.median(np.linalg.norm(gL - com_b, axis=1)); dR = np.median(np.linalg.norm(gR - com_b, axis=1))
    dB = np.median(np.linalg.norm(ucm_b - com_b, axis=1))
    It = It_b + mb * dB ** 2 + mhand * (dL ** 2 + dR ** 2)
    bodies["bat"] = Body("bat", mt, com_b, Rb, np.array([It, It, C.BAT_IAX * k_b + 2 * mhand * 0.03 ** 2]), dt)
    sweet = knob + bax * sw.sweet_u
    vs = np.linalg.norm(np.gradient(sweet, dt, axis=0), axis=1)

    i0 = int(np.argmin(np.abs(t - T_FROM)))
    i1 = int(np.argmin(np.abs(t - t_to)))
    rows = list(range(i0, i1 + 1, 4)) + ([i1] if (i1 - i0) % 4 else [])
    say = print
    say("# Inverse dynamics of OpenBiomechanics swing %s on his own body (%.1f in, %.0f lb = %.1f kg), filter %.0f Hz on the samples to contact"
        % (sid, Hh / 0.0254, M / LB, M, cut))
    say("# DERIVED FROM CC BY-NC-SA DATA: keep out of git (PLAN D46).")
    say("his segment lengths (m, own size, median to contact): " + ", ".join("%s %.3f" % (k, v) for k, v in sorted(lens.items())))
    say("hips %.3f apart, shoulders %.3f apart; sum of segment masses %.1f kg (+ bat %.3f kg)"
        % (medlen("hip_L", "hip_R"), medlen("shoulder_L", "shoulder_R"), sum(b.m for b in bodies.values()) - mt + 2 * mhand, mb))
    say("sweet spot speed at the sample before contact (filtered, to contact only): %.1f m/s (%.1f mph); fastest to contact %.1f m/s"
        % (vs[i1], vs[i1] / 0.44704, vs[: i1 + 1].max()))

    # ---- the whole body against the plates ----
    fp = plates(sw, t)
    tot = sum(b.m * b.a for b in bodies.values())
    mass = sum(b.m for b in bodies.values())
    need = tot + mass * np.array([0.0, 0.0, G])
    W = M * G
    say("\n## 1. The whole body against the force plates (body weights; + toward first base, the pitcher, up)")
    say("%8s | %6s %6s %6s | %6s %6s %6s | %5s %5s" % ("t ms", "need x", "y", "z", "plate x", "y", "z", "rear", "lead"))
    for i in rows:
        say("%8.0f | %6.2f %6.2f %6.2f | %6.2f %6.2f %6.2f | %5.2f %5.2f" % (1e3 * t[i], *(need[i] / W), *((fp["R"][i] + fp["L"][i]) / W),
                                                                       fp["R"][i][2] / W, fp["L"][i][2] / W))
    win = slice(i0, i1 + 1)
    if "--parts" in argv:
        say("each body's mass x acceleration, body weights (x, y, z):")
        for i in rows:
            say("%6.0f " % (1e3 * t[i]) + "  ".join("%s %+.2f %+.2f %+.2f" % (nm[:6], *(b.m * b.a[i] / W)) for nm, b in bodies.items()))
    say("over %.0f to %.0f ms: rms of plate minus need, body weights: x %.3f y %.3f z %.3f"
        % (1e3 * t[i0], 1e3 * t[i1], *np.sqrt(np.mean(((fp["R"] + fp["L"] - need)[win] / W) ** 2, axis=0))))
    best = []
    for sh_ms in range(-40, 41, 5):
        fps = plates(sw, t + sh_ms / 1e3)
        best.append((float(np.sqrt(np.mean(np.sum(((fps["R"] + fps["L"] - need)[win] / W) ** 2, axis=1)))), sh_ms))
    say("the plates moved in time against the motion (ms: rms of the whole residual, body weights): " + ", ".join("%+d: %.2f" % (b_, a_) for a_, b_ in best))
    lo = int(np.argmin(np.abs(t - (-0.100)))); win2 = slice(i0, lo + 1)
    say("over %.0f to -100 ms only (clear of the filter's end): rms x %.3f y %.3f z %.3f"
        % (1e3 * t[i0], *np.sqrt(np.mean(((fp["R"] + fp["L"] - need)[win2] / W) ** 2, axis=0))))

    # ---- top down ----
    k = (M * G * Hh) / DR.REF_WH
    bd = DR.bounds(Hh, M)
    # bat + hands: what the two forearms must put on it at the wrists, split as two points (least forces)
    Fb, Mb = bodies["bat"].need()
    wL, wR = F["wrist_L"], F["wrist_R"]
    mid = (wL + wR) / 2
    Mmid = Mb + np.cross(com_b - mid, Fb)              # moment about the wrists' midpoint
    e = unit(wR - wL)
    d = np.linalg.norm(wR - wL, axis=1)[:, None]
    Mperp = Mmid - e * np.sum(Mmid * e, axis=1, keepdims=True)
    dF = 2.0 * np.cross(Mperp, e) / d                  # F_R - F_L: the couple (d/2) e x dF gives Mperp
    hand_F = {"L": Fb / 2 - dF / 2 * 1.0, "R": Fb / 2 + dF / 2}
    # check: the couple gives the moment
    chk = np.cross(wL - mid, hand_F["L"]) + np.cross(wR - mid, hand_F["R"]) - Mperp
    hand_split_err = float(np.max(np.linalg.norm(chk, axis=1)[win]))
    wrist_left = float(np.max(np.abs(np.sum(Mmid * e, axis=1))[win]))   # the moment along the wrists' line no point force carries
    joint = {}
    for sd in "LR":
        fa, ua = bodies["farm_" + sd], bodies["uarm_" + sd]
        # forearm: from the bat at the wrist (hand_F on the bat, its reaction on the forearm), from the upper arm at the elbow
        Fw = hand_F[sd]                                 # force the forearm puts on the bat at the grip
        Ff, Mf = fa.need()
        Fel = Ff + Fw                                   # force the upper arm puts on the forearm at the elbow
        Mel = Mf - np.cross(F["wrist_" + sd] - fa.com, -Fw) - np.cross(F["elbow_" + sd] - fa.com, Fel)
        Fu, Mu = ua.need()
        Fsh = Fu + Fel
        Msh = Mu + Mel - np.cross(F["elbow_" + sd] - ua.com, -Fel) - np.cross(F["shoulder_" + sd] - ua.com, Fsh)
        joint["elbow_" + sd] = (Fel, Mel)
        joint["shoulder_" + sd] = (Fsh, Msh)
    # lumbar from above: everything above it
    above = ["trunk", "head", "uarm_L", "uarm_R", "farm_L", "farm_R", "bat"]
    Ml = np.zeros_like(Mb); Fl = np.zeros_like(Fb)
    P = F["lumbar"]
    for nm in above:
        Fn, Mn = bodies[nm].need()
        Fl = Fl + Fn
        Ml = Ml + Mn + np.cross(bodies[nm].com - P, Fn)
    joint["lumbar"] = (Fl, Ml)

    # ---- bottom up ----
    def legs(cop_shift):
        out = {}
        for sd in "LR":
            ft, sk, th = bodies["foot_" + sd], bodies["shank_" + sd], bodies["thigh_" + sd]
            fx = Q["foot_" + sd][:, :, 0]
            heel, ball = F["heel_" + sd], F["ball_" + sd]
            heel_up = heel[:, 2] > 0.015
            cop = np.where(heel_up[:, None], ball, (heel + ball) / 2) + fx * cop_shift
            cop[:, 2] = 0.0
            Fg = fp[sd]
            Ff, Mf = ft.need()
            Fa = Ff - Fg                                # force the shank puts on the foot at the ankle
            Ma = Mf - np.cross(cop - ft.com, Fg) - np.cross(F["ankle_" + sd] - ft.com, Fa)
            Fs, Ms = sk.need()
            Fk = Fs + Fa
            Mk = Ms + Ma - np.cross(F["ankle_" + sd] - sk.com, -Fa) - np.cross(F["knee_" + sd] - sk.com, Fk)
            Ft, Mt = th.need()
            Fh = Ft + Fk
            Mh = Mt + Mk - np.cross(F["knee_" + sd] - th.com, -Fk) - np.cross(F["hip_" + sd] - th.com, Fh)
            out["ankle_" + sd] = (Fa, Ma); out["knee_" + sd] = (Fk, Mk); out["hip_" + sd] = (Fh, Mh)
        return out
    L0 = legs(0.0); Lp = legs(0.04); Lm = legs(-0.04)
    joint.update(L0)
    # the pelvis's balance: hips from below and lumbar from above against what it needs
    Fp, Mp = bodies["pelvis"].need()
    resF = Fp - (-L0["hip_L"][0] - L0["hip_R"][0] + (-Fl))
    say("\n## 2. Where the two ends meet: the pelvis (hips from the plates below, the lumbar joint from above)")
    Mres = Mp - (np.cross(P - bodies["pelvis"].com, -Fl) - Ml)
    for sd in "LR":
        Mres = Mres - (np.cross(F["hip_" + sd] - bodies["pelvis"].com, -L0["hip_" + sd][0]) - L0["hip_" + sd][1])
    say("force the pelvis lacks, body weights, rms over the stretch: %.2f (x %.2f y %.2f z %.2f)"
        % (np.sqrt(np.mean(np.sum((resF[win] / W) ** 2, axis=1))), *np.sqrt(np.mean((resF[win] / W) ** 2, axis=0))))
    say("moment the pelvis lacks (N m; what the feet's unrecorded twisting moments, the assumed centres of pressure and the model's errors leave):"
        " about the vertical rms %.0f, peak %.0f at %.0f ms; about the horizontal rms %.0f, peak %.0f"
        % (np.sqrt(np.mean(Mres[win, 2] ** 2)), np.abs(Mres[win, 2]).max(), 1e3 * t[i0 + int(np.argmax(np.abs(Mres[win, 2])))],
           np.sqrt(np.mean(np.sum(Mres[win, :2] ** 2, axis=1))), np.linalg.norm(Mres[win, :2], axis=1).max()))
    say("the hands' split meets the bat's moment to %.1e N m; the moment along the wrists' line it leaves to the wrists: peak %.0f N m"
        % (hand_split_err, wrist_left))

    # ---- against the bounds ----
    def comp(name, frame_body, ax, Mrel=None):
        """component of the moment the parent puts on the child at joint `name`, about axis ax of frame_body's frame"""
        R = Q[frame_body] if frame_body in Q else bodies[frame_body].R
        return np.einsum("ni,ni->n", joint[name][1], R[:, :, ax])

    def relw(child, parent, frame_body, ax):
        R = Q[frame_body]
        return np.einsum("ni,ni->n", bodies[child].w - bodies[parent].w, R[:, :, ax])

    def curve(cname, w):
        tab = DR.CURVES[cname]
        if w < 0:
            return 1.5 if not cname.startswith(("sh_", "elbow", "wrist")) else DR.ARM_ECC
        for (w0, f0), (w1, f1) in zip(tab, tab[1:]):
            if w <= w1:
                return f0 + (f1 - f0) * (w - w0) / (w1 - w0)
        return 0.0

    # (label, joint, frame body, axis index, sign so + is the muscle's direction, child, parent, bound name, curve)
    spec = [
        ("trunk twist toward the pitcher", "lumbar", "trunk", 2, +1, "trunk", "pelvis", "core", "trunk"),
        ("trunk twist back (brake)", "lumbar", "trunk", 2, -1, "trunk", "pelvis", "core_back", "trunk_back"),
        ("trunk bend forward", "lumbar", "trunk", 1, +1, "trunk", "pelvis", ("hold", "trunk_fwd"), "trunk_bend"),
        ("trunk bend back", "lumbar", "trunk", 1, -1, "trunk", "pelvis", ("hold", "trunk_ext"), "trunk_bend"),
        ("trunk bend to his right", "lumbar", "trunk", 0, +1, "trunk", "pelvis", ("hold", "trunk_side_R"), "trunk_bend"),
        ("trunk bend to his left", "lumbar", "trunk", 0, -1, "trunk", "pelvis", ("hold", "trunk_side_L"), "trunk_bend"),
    ]
    for sd, sg, side in (("L", 1.0, "lead"), ("R", -1.0, "rear")):
        spec += [
            ("%s hip extension" % side, "hip_" + sd, "thigh_" + sd, 1, +1, "thigh_" + sd, "pelvis", "hip_ext_" + sd, "hip_ext"),
            ("%s hip flexion (bound: the lead's)" % side, "hip_" + sd, "thigh_" + sd, 1, -1, "thigh_" + sd, "pelvis", "hip_flex_L", "hip_flex"),
            ("%s hip abduction" % side, "hip_" + sd, "thigh_" + sd, 0, sg, "thigh_" + sd, "pelvis", "hip_abd_" + sd, "hip_ext"),
            ("%s hip adduction" % side, "hip_" + sd, "thigh_" + sd, 0, -sg, "thigh_" + sd, "pelvis", "hip_add_" + sd, "hip_ext"),
            ("%s hip turning the pelvis toward the pitcher" % side, "hip_" + sd, "thigh_" + sd, 2, -1, "thigh_" + sd, "pelvis", "hip_rot_" + sd, "hip_ext"),
            ("%s hip turning it back" % side, "hip_" + sd, "thigh_" + sd, 2, +1, "thigh_" + sd, "pelvis", "hip_rot_" + sd, "hip_ext"),
            ("%s knee extension" % side, "knee_" + sd, "shank_" + sd, 1, -1, "shank_" + sd, "thigh_" + sd, "knee_ext_" + sd, "knee_ext"),
            ("%s knee flexion" % side, "knee_" + sd, "shank_" + sd, 1, +1, "shank_" + sd, "thigh_" + sd, "knee_flex_" + sd, "knee_flex"),
            ("%s ankle plantar flexion" % side, "ankle_" + sd, "foot_" + sd, 1, +1, "foot_" + sd, "shank_" + sd, "ankle_pf_" + sd, "ankle_pf"),
            ("%s elbow extension" % side, "elbow_" + sd, "uarm_" + sd, 1, +1, "farm_" + sd, "uarm_" + sd, "elbow_ext_" + sd, "elbow_ext"),
            ("%s elbow flexion" % side, "elbow_" + sd, "uarm_" + sd, 1, -1, "farm_" + sd, "uarm_" + sd, "elbow_flex_" + sd, "elbow_flex"),
            ("%s shoulder about the trunk's axis, arm toward the pitcher" % side, "shoulder_" + sd, "trunk", 2, +1, "uarm_" + sd, "trunk",
             "shoulder_" + sd, "sh_habd" if sd == "L" else "sh_hadd"),
        ]
    hold = {nm: v[5] * k for nm, v in DR.HOLD_ONLY.items()}
    say("\n## 3. Each joint's need against the force-driven body's measured strength (PLAN D34; bounds scaled by his weight x height, k = %.3f)" % k)
    say("strength at speed: the bound x drive.CURVES at the joint's own speed that way (stretched: x1.5 legs and trunk, x1.4 arms).")
    say("legs: centre of pressure mid-sole (ball when the heel is up); [range] with it 4 cm back and forward. Arms: the least-force split of the bat's load.")
    say("%-55s %8s %8s %9s %8s %9s  %s" % ("", "bound", "peak", "at ms", "need/bd", "need/at-speed", "legs' range of peak"))
    worst = []
    for lab_, jn, fb, ax, sg, ch, pa, bname, cname in spec:
        tau = sg * comp(jn, fb, ax)
        wv = sg * relw(ch, pa, fb, ax) if (ch in bodies and pa in bodies) else np.zeros(n)
        # the speed that way, as the muscle shortens: the child's turn relative to the parent in the torque's direction
        B = hold[bname[1]] if isinstance(bname, tuple) else bd[bname]
        ratio = np.array([tau[i] / (B * curve(cname, wv[i])) if curve(cname, wv[i]) > 0 else (np.inf if tau[i] > 0 else 0.0) for i in range(n)])
        ii = i0 + int(np.argmax(tau[win]))
        jj = i0 + int(np.argmax(ratio[win]))
        rng = ""
        if jn.startswith(("hip", "knee", "ankle")):
            vals = [sg * np.einsum("ni,ni->n", Lx[jn][1], Q[fb][:, :, ax])[win].max() for Lx in (Lm, Lp)]
            rng = "[%.0f, %.0f]" % (min(vals), max(vals))
        say("%-55s %8.0f %8.0f %9.0f %8.2f %9.2f@%4.0f  %s" % (lab_, B, tau[ii], 1e3 * t[ii], tau[ii] / B, ratio[jj], 1e3 * t[jj], rng))
        worst.append((ratio[jj], lab_))
    say("\nover their measured strength at their speed (need/at-speed > 1), worst first:")
    for r, l in sorted(worst, reverse=True):
        if r > 1.0:
            say("  %.2f  %s" % (r, l))

    say("\n## 4. Through time: the largest needs (N m), every 10 ms")
    keys = [("trunk twist +", "lumbar", "trunk", 2, 1), ("trunk side R", "lumbar", "trunk", 0, 1), ("trunk fwd", "lumbar", "trunk", 1, 1),
            ("lead hip ext", "hip_L", "thigh_L", 1, 1), ("rear hip ext", "hip_R", "thigh_R", 1, 1),
            ("lead knee ext", "knee_L", "shank_L", 1, -1), ("rear knee ext", "knee_R", "shank_R", 1, -1),
            ("lead sh z", "shoulder_L", "trunk", 2, 1), ("rear sh z", "shoulder_R", "trunk", 2, 1),
            ("lead elb ext", "elbow_L", "uarm_L", 1, 1), ("rear elb ext", "elbow_R", "uarm_R", 1, 1)]
    say("%7s " % "t ms" + " ".join("%13s" % k_[0] for k_ in keys) + " %9s" % "sweet m/s")
    for i in rows:
        say("%7.0f " % (1e3 * t[i]) + " ".join("%13.0f" % (k_[4] * comp(k_[1], k_[2], k_[3])[i]) for k_ in keys) + " %9.1f" % vs[i])

    say("\n## 5. Energy (J) at the sample before contact: the bat with the hands %.0f; the arms %.0f; the trunk and head %.0f; the pelvis and legs %.0f"
        % (bodies["bat"].ke()[i1], sum(bodies[b].ke()[i1] for b in ("uarm_L", "uarm_R", "farm_L", "farm_R")),
           bodies["trunk"].ke()[i1] + bodies["head"].ke()[i1],
           sum(bodies[b].ke()[i1] for b in bodies if b.startswith(("pelvis", "thigh", "shank", "foot")))))
    pel = bodies["pelvis"].w[:, 2] / (math.pi / 180); tr = bodies["trunk"].w[:, 2] / (math.pi / 180)
    say("pelvis turning fastest %.0f deg/s at %.0f ms; trunk %.0f deg/s at %.0f ms (filtered %.0f Hz, to contact)"
        % (pel[win].max(), 1e3 * t[i0 + int(np.argmax(pel[win]))], tr[win].max(), 1e3 * t[i0 + int(np.argmax(tr[win]))], cut))


def lab_rot(sw, key):
    """a raw marker in the sim frame, scaled and placed as the figure's points are (true() then undoes it)."""
    return sw.place(sw.lab[key])


def plates(sw, t):
    """the dataset's force plates resampled at the motion's samples, sim frame, N: {"R": rear, "L": lead}."""
    import csv
    rows = list(csv.DictReader(open(os.path.join(RS.OBP, "fp_%s.csv" % sw.sid))))
    tf = np.array([float(r["time"]) for r in rows]) - float(rows[0]["contact_time"])
    out = {}
    for sd, pre in (("R", "rear"), ("L", "lead")):
        lab = np.stack([[float(r["%s_force_%s" % (pre, c)]) for c in "xyz"] for r in rows])
        sim = lab @ RS.LAB_TO_SIM.T
        out[sd] = np.stack([np.interp(t, tf, sim[:, c]) for c in range(3)], axis=1)
    return out


if __name__ == "__main__":
    main(sys.argv[1:])
