"""The written swing made by muscles: the force-driven body run forward while its muscles follow swing_author.py's motion
(PLAN D48; docs/body-motion.md 4.17).

    python3 analysis/research/body/swing_track.py --practice [--out plan.json] [--rounds N]    practise it, keep the plan
    python3 analysis/research/body/swing_track.py <plan.json> --table                          the kept swing, every 10 ms
    uv run --with matplotlib python3 analysis/research/body/swing_track.py <plan.json> --draw <dir>
    python3 analysis/research/body/swing_track.py <plan.json> --swing <file.swing>             for the Unreal viewer
    python3 analysis/research/body/swing_track.py --check                                      the compiled run against the Python one

What moves the body. Nothing but muscles and the ground. Every joint has a pair of muscles about each of its axes
(MUSCLES: the strengths of body-motion 4.5 where it has one, the rest listed in 4.17), each with a strength that falls
with its shortening speed, a level that rises in 15 ms and falls in 50, and the joint's own friction. The feet stand on
the engine's ground. There is no force on the body from outside and nothing puts a body part anywhere.

What decides the muscles' levels. At each instant, from what the body is doing at that instant and from a plan made
beforehand, never from anything later:
- a planned torque about each joint axis for that instant of the swing (the plan: a table in time, learned in practice);
- a correction in proportion to how far the joint is from where the written motion has it and how fast it is
  parting from it (the muscles' own stiffness and reflexes, with no delay but the muscles' own rise time: 4.17);
- for the legs that stand on the ground, a correction that pushes the pelvis back toward where the written motion
  has it (balance).
Practice (--practice) is how the plan is learned: the swing is run, and what the corrections had to add at each instant
is put into the plan for the next run, a part at a time from the stance onward. It changes the plan between swings,
never anything inside a swing. The kept swing is ONE forward run of the kept plan.
"""
import json
import math
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "..", "oracle"))
from body3d import engine as E, body as B, drive as Dr   # noqa: E402
import swing_author as A                                  # noqa: E402

D2R = math.pi / 180.0
C = B.C
OUT = os.path.join(ROOT, "sim", "target", "codex-probes", "swing-look")
BIN = os.path.join(OUT, "swing_track")
SRC = os.path.join(HERE, "swing_track.rs")
# The integration step (classical RK4). A tenth of a millisecond, and not the half the earlier sessions' runs used: a
# muscle's strength falls with its shortening speed, which acts on its joint as a brake, and on the lightest thing a
# muscle turns (a foot about its own length, 0.0014 kg m^2, by the muscles that roll the sole) that brake is too stiff
# for a longer step: at 0.5 ms the foot's turning rate flips sign every step while he stands on one leg, and the
# books of energy no longer close (body-motion 4.17; test_swing_look holds the books to a hundredth).
H_STEP = 1e-4
REC_EVERY = 10                 # a record every millisecond
TREE_ORDER = (("shank_L", "foot_L"), ("thigh_L", "shank_L"), ("pelvis", "thigh_L"),
              ("shank_R", "foot_R"), ("thigh_R", "shank_R"), ("pelvis", "thigh_R"),
              ("farm_L", "bat"), ("uarm_L", "farm_L"), ("trunk", "uarm_L"),
              ("farm_R", "bat"), ("uarm_R", "farm_R"), ("trunk", "uarm_R"), ("pelvis", "trunk"))

# ---- the muscles (body-motion 4.17) ------------------------------------------------------------------------------------
# Strengths the body already had (body-motion 4.5, drive.bounds) are taken by name. The rest are what a body has and this
# one lacked; each is [E], listed in 4.17 with its reason:
ADDED = {"hip_flex_R": ("hip_flex_L", 1.0),      # the rear hip's flexor: the lead's (the body had none: 4.15)
         "hip_rot_back": ("hip_rot_L", 1.0),     # each hip's rotators the other way: as strong as the one way (itself [E])
         "ankle_df": 50.0,                       # N m: lifting the toes [E: 40-60, a man's dorsiflexors; not read]
         "ankle_roll": 50.0,                     # N m: rolling the sole in or out [E: 30-50; not read]
         "sh_other": None,                       # each shoulder's sweep the other way: the other arm's measured value
         "wrist_radial": 0.75}                   # the wrist cocked toward the thumb: 0.75 of the measured tilt the other way [E]
STRENGTH = 1.0                 # every muscle's strength x this (1 = body-motion 4.5's bounds)
HIP_SIDE = 1.0                 # and the hips' sideways muscles and rotators x this more (their bounds are the weakest sourced: 4.5)
# how hard each joint is held to the written motion: N m per radian off, N m per rad/s of parting [E]
GAINS = {"lumbar": (2500.0, 120.0), "hip": (2500.0, 110.0), "knee": (2000.0, 80.0), "ankle": (1500.0, 50.0),
         "shoulder": (300.0, 14.0), "elbow": (150.0, 6.0), "wrist": (40.0, 1.2)}
AIR_ANKLE = 0.2                # the lead ankle's hold while that foot is in the air, as a share of its gains (a foot weighs little)
# balance: the pelvis pushed back toward the written motion through the legs that stand on the ground [E]
ROOT_GAINS = dict(kpos_h=15000.0, kpos_v=20000.0, kdpos=2000.0, krot=3000.0, kdrot=250.0)
REG3 = 0.02                    # the shoulder's three muscle axes are not square to each other: a small ridge in the split
DAMP = Dr.DAMPING["read"]      # the joints' own friction (body-motion 4.13, research note 14)
BRAKE = 0.9                    # [E: 0.8-1]: under 1 so that a steady torque is held by one muscle of the pair, not both
SAT_EPS = 1e-3                 # N m: a muscle with less strength than this left at its speed is taken as having none


def _bd():
    b = Dr.bounds(A.H, A.M)
    b["hip_flex_R"] = b[ADDED["hip_flex_R"][0]] * ADDED["hip_flex_R"][1]
    sch = Dr.HOLD_ONLY
    b.update({"trunk_fwd": sch["trunk_fwd"][5], "trunk_ext": sch["trunk_ext"][5], "trunk_side_R": sch["trunk_side_R"][5], "trunk_side_L": sch["trunk_side_L"][5]})
    return b


def muscles(m, strength=None, hip_side=None):
    """the joints and, about each axis, its pair of muscles: [(joint, parent body, child body, family,
    [(kind, axis, axis in the child for a floating axis, +name, +strength, +curve, -name, -strength, -curve, friction)])].
    kind 0: the axis is fixed in the parent; 1: fixed in the child; 2: floating, along (parent's axis) x (child's axis).
    The torque about an axis acts on the child about +axis and its reaction on the parent."""
    bd = _bd()
    k = STRENGTH if strength is None else strength
    hs = HIP_SIDE if hip_side is None else hip_side
    T = lambda n: bd[n] * k * (hs if n.startswith(("hip_abd", "hip_add", "hip_rot")) else 1.0)
    side = Dr.HIP_SIDE_CURVE
    out = []
    dj = lambda j, i: (DAMP[j][i] if isinstance(DAMP[j], (tuple, list)) else DAMP[j])
    out.append(("lumbar", "pelvis", "trunk", "lumbar", [
        (0, (1, 0, 0), None, "trunk_side_R", T("trunk_side_R"), "trunk_bend", "trunk_side_L", T("trunk_side_L"), "trunk_bend", dj("lumbar", 0)),
        (0, (0, 1, 0), None, "trunk_fwd", T("trunk_fwd"), "trunk_bend", "trunk_ext", T("trunk_ext"), "trunk_bend", dj("lumbar", 1)),
        (0, (0, 0, 1), None, "core", T("core"), "trunk", "core_back", T("core_back"), "trunk_back", dj("lumbar", 2))]))
    for sd, sg in B.SIDES:
        L = sg > 0
        rot = T("hip_rot_" + sd)
        out.append(("hip_" + sd, "pelvis", "thigh_" + sd, "hip", [
            (1, (0, 1, 0), None, "hip_ext_" + sd, T("hip_ext_" + sd), "hip_ext", "hip_flex_" + sd, T("hip_flex_" + sd), "hip_flex", dj("hip_" + sd, 1)),
            (1, (1, 0, 0), None, ("hip_abd_" if L else "hip_add_") + sd, T("hip_abd_" + sd), side, ("hip_add_" if L else "hip_abd_") + sd, T("hip_add_" + sd), side, dj("hip_" + sd, 0)),
            (1, (0, 0, 1), None, "hip_rot+_" + sd, rot, side, "hip_rot-_" + sd, rot * ADDED["hip_rot_back"][1], side, dj("hip_" + sd, 2))]))
        out.append(("knee_" + sd, "thigh_" + sd, "shank_" + sd, "knee", [
            (0, (0, 1, 0), None, "knee_flex_" + sd, T("knee_flex_" + sd), "knee_flex", "knee_ext_" + sd, T("knee_ext_" + sd), "knee_ext", DAMP["knee_" + sd])]))
        out.append(("ankle_" + sd, "shank_" + sd, "foot_" + sd, "ankle", [
            (0, (0, 1, 0), None, "ankle_pf_" + sd, T("ankle_pf_" + sd), "ankle_pf", "ankle_df_" + sd, ADDED["ankle_df"] * k, "ankle_pf", dj("ankle_" + sd, 1)),
            (1, (1, 0, 0), None, "ankle_roll+_" + sd, ADDED["ankle_roll"] * k, "ankle_pf", "ankle_roll-_" + sd, ADDED["ankle_roll"] * k, "ankle_pf", dj("ankle_" + sd, 0))]))
    for sd, sg in B.SIDES:
        L = sg > 0
        habd, hadd = Dr.SHOULDER_HABD * k, Dr.SHOULDER_HADD * k
        irot, erot = T("sh_irot_" + sd), T("sh_erot_" + sd)
        out.append(("shoulder_" + sd, "trunk", "uarm_" + sd, "shoulder", [
            (0, (0, 0, 1), None) + (("sh_habd_" + sd, habd, "sh_habd", "sh_hadd_" + sd, hadd, "sh_hadd") if L else ("sh_hadd_" + sd, hadd, "sh_hadd", "sh_habd_" + sd, habd, "sh_habd")) + (DAMP["shoulder_" + sd],),
            (2, (0, 0, 1), (0, 0, 1), "sh_raise_" + sd, T("sh_raise_" + sd), "sh_raise", "sh_lower_" + sd, T("sh_lower_" + sd), "sh_lower", DAMP["shoulder_" + sd]),
            (1, (0, 0, 1), None) + (("sh_erot_" + sd, erot, "sh_erot", "sh_irot_" + sd, irot, "sh_irot") if L else ("sh_irot_" + sd, irot, "sh_irot", "sh_erot_" + sd, erot, "sh_erot")) + (DAMP["shoulder_" + sd],)]))
        pron, sup = T("pron_" + sd), T("sup_" + sd)
        out.append(("elbow_" + sd, "uarm_" + sd, "farm_" + sd, "elbow", [
            (0, (0, 1, 0), None, "elbow_flex_" + sd, T("elbow_flex_" + sd), "elbow_flex", "elbow_ext_" + sd, T("elbow_ext_" + sd), "elbow_ext", dj("elbow_" + sd, 1)),
            (1, (0, 0, 1), None) + (("sup_" + sd, sup, "sup", "pron_" + sd, pron, "pron") if L else ("pron_" + sd, pron, "pron", "sup_" + sd, sup, "sup")) + (dj("elbow_" + sd, 0),)]))
        fl, ex = T("wrist_flex_" + sd), T("wrist_ext_" + sd)
        uln = T("wrist_" + sd)
        rad = uln * ADDED["wrist_radial"]
        wc = "wrist_" + Dr.WRIST_READING
        out.append(("wrist_" + sd, "farm_" + sd, "bat", "wrist", [
            (0, (1, 0, 0), None) + (("wrist_ext_" + sd, ex, "wrist_ext", "wrist_flex_" + sd, fl, "wrist_flex") if L else ("wrist_flex_" + sd, fl, "wrist_flex", "wrist_ext_" + sd, ex, "wrist_ext")) + (dj("wrist_" + sd, 0),),
            (1, tuple(m.palm[sd]), None) + (("wrist_rad_" + sd, rad, wc, "wrist_uln_" + sd, uln, wc) if L else ("wrist_uln_" + sd, uln, wc, "wrist_rad_" + sd, rad, wc)) + (dj("wrist_" + sd, 1),)]))
    return out


# ---- the body for the forward run: the written motion's body, with its joints' ends ---------------------------------------
STOP_MARGIN = 3.0              # deg: a joint's end is put this far beyond what the written motion itself reaches, where it reaches past
# The wrists' two angles are held each at its own ends (body-motion 4.11's box; the ellipse of 4.12 is not used: where the
# written motion leaves a wrist's range after contact its ends are moved out, and an ellipse has no single end to move).


def add_stops(m, ts, states, log=None):
    """every joint range of body.range_specs as a one-sided stop at its measured end (body.ROM), moved out to STOP_MARGIN
    beyond the written motion wherever the written motion itself goes past the measured end (after contact the wrists
    and forearms do: hands welded to the handle cannot roll over as a hitter's top hand does). Returns {stop: (lo, hi,
    how far it was moved out)} deg."""
    s, b = m.sys, m.b
    specs = B.range_specs(None, "hands", "cleats", "box")
    reach = {}
    for i in range(0, len(states), 5):
        a = B.range_angles(m, states[i])
        for name, *_r in specs:
            v = a[name]
            lo, hi = reach.get(name, (v, v))
            reach[name] = (min(lo, v), max(hi, v))
    out = {}
    for name, a, b_, ax, ea, fb, lo, hi, size, kind in specs:
        if size is None:
            continue
        lo2, hi2 = min(lo, reach[name][0] - STOP_MARGIN), max(hi, reach[name][1] + STOP_MARGIN)
        moved = max(lo - lo2, hi2 - hi)
        s.stop(name, b[a], b[b_], ax, ea, fb, lo2 * D2R, hi2 * D2R, B.RANGE_K[size], B.RANGE_C[size], kind)
        out[name] = (lo2, hi2, moved)
        if log is not None and moved > 0.0:
            log("   the end of %s moved out %.1f deg (the written motion reads %.1f to %.1f; measured range %.1f to %.1f)" % (name, moved, reach[name][0], reach[name][1], lo, hi))
    return out


def support(t):
    """(lead, rear): each leg's share in pushing the pelvis back toward the written motion: by which feet the written motion
    has on the ground (the lead foot is in the air between A.T_OFF and A.T_TOUCH), with smooth hand-overs."""
    ramp = lambda x, a, b_: min(1.0, max(0.0, (x - a) / (b_ - a)))
    sm = lambda u: u * u * (3.0 - 2.0 * u)
    if t < A.T_OFF - 0.25:
        l = 0.45
    elif t < A.T_OFF - 0.02:
        l = 0.45 * (1.0 - sm(ramp(t, A.T_OFF - 0.25, A.T_OFF - 0.02)))
    elif t < A.T_TOUCH + 0.01:
        l = 0.0
    elif t < A.T_CONTACT:
        l = 0.8 * sm(ramp(t, A.T_TOUCH + 0.01, A.T_HEEL + 0.05))
    else:
        l = 0.8
    return l, 1.0 - l


class Track:
    """the body, its muscles, the written motion as each joint's turn in time, and a plan (a torque about every muscle
    axis in time)."""

    def __init__(self, strength=None, gains=None, root_gains=None, motion=None, log=None, hip_side=None):
        self.m, self.ts, self.states, worst = A.build(motion=motion, log=log)
        m, s = self.m, self.m.sys
        self.stops = add_stops(m, self.ts, self.states, log)
        self.dt = self.ts[1] - self.ts[0]
        self.t0 = self.ts[0]
        self.n = len(self.ts)
        self.gains = dict(GAINS, **(gains or {}))
        self.root_gains = dict(ROOT_GAINS, **(root_gains or {}))
        self.strength = STRENGTH if strength is None else strength
        self.hip_side = HIP_SIDE if hip_side is None else hip_side
        # strength-at-speed curves, by index
        self.curves, self.tabs = [], []
        tab = lambda name: self.curves.index(name) if name in self.curves else (self.curves.append(name) or self.tabs.append((tuple(Dr.CURVES[name]), Dr.curve_ecc(name))) or len(self.curves) - 1)
        self.joints, self.axes = [], []           # joints: (name, a, b, family, first axis index, number of axes)
        for name, pa, ch, fam, axs in muscles(m, self.strength, self.hip_side):
            self.joints.append((name, m.b[pa], m.b[ch], fam, len(self.axes), len(axs)))
            for kind, ax, axb, np_, Tp, cp, nm_, Tm, cm, damp in axs:
                self.axes.append(dict(joint=name, kind=kind, ax=E.unit(ax), axb=E.unit(axb) if axb else (0.0, 0.0, 0.0), plus=np_, Tp=Tp, tabp=tab(cp),
                                      minus=nm_, Tm=Tm, tabm=tab(cm), damp=float(damp)))
        self.nax = len(self.axes)
        self.pelvis = m.b["pelvis"]
        self.legs = [[k for k, j in enumerate(self.joints) if j[0] == n + sd][0] for sd in "LR" for n in ("hip_", "knee_", "ankle_")]   # L: hip, knee, ankle; R: ...
        # the written motion as each joint sees it: the child's turn in the parent's frame, and its rate there
        self.qref, self.wref = [], []
        for st in self.states:
            qr, wr = [], []
            for name, a, b_, fam, k0, na in self.joints:
                xa, qa, va, wa = s.pose(st, a)
                xb, qb, vb, wb = s.pose(st, b_)
                qr.append(E.qmul(E.qconj(qa), qb))
                wr.append(E.mtv(E.qR(qa), E.sub(wb, wa)))
            self.qref.append(qr)
            self.wref.append(wr)
        for k in range(1, self.n):                # one sign all the way, so a straight blend between two samples is a turn between them
            for j in range(len(self.joints)):
                if sum(x * y for x, y in zip(self.qref[k - 1][j], self.qref[k][j])) < 0.0:
                    self.qref[k][j] = tuple(-x for x in self.qref[k][j])
        self.root = []
        qprev = None
        for st in self.states:
            x, q, v, w = s.pose(st, self.pelvis)
            if qprev is not None and sum(a_ * b_ for a_, b_ in zip(qprev, q)) < 0.0:
                q = tuple(-c for c in q)
            qprev = q
            self.root.append((x, q, v, w))
        self.sup = [support(t) for t in self.ts]
        self.gain = [[1.0] * len(self.joints) for _ in self.ts]     # each joint's hold, as a share of its GAINS, in time
        ja = [k for k, j in enumerate(self.joints) if j[0] == "ankle_L"][0]
        for k, t in enumerate(self.ts):
            l = self.sup[k][0]
            self.gain[k][ja] = AIR_ANKLE + (1.0 - AIR_ANKLE) * min(1.0, l / 0.2)
        self.ff = [[0.0] * self.nax for _ in self.ts]               # the plan: N m about each muscle axis, in time
        self.fb_scale = 1.0

    # ---- one state for the engine: the bodies, the work integrals, then every muscle's level (+ then - for each axis)
    def start_state(self, k=0):
        s = self.m.sys
        st = list(self.states[k]) + [0.0] * (s.n_state - len(self.states[k]))
        return st + [0.0] * (2 * self.nax)

    def system(self):
        return TrackSystem(self)


def _lerp(a, b, f):
    return tuple(x + f * (y - x) for x, y in zip(a, b))


class TrackSystem(E.System):
    """the engine with the muscles and their control (the checking implementation; swing_track.rs is the compiled copy,
    held to this one by --check and the tests)."""

    def __init__(self, tr):
        s = tr.m.sys
        E.System.__init__(self, s.g)
        for k, v in s.__dict__.items():
            setattr(self, k, v)
        self.tr = tr
        self.trace = None          # set to a list to have _forces append, per call, each axis's (asked, plan, given, + level, - level, speed)

    def ref_at(self, t):
        tr = self.tr
        u = (t - tr.t0) / tr.dt
        k = int(math.floor(u))
        k = max(0, min(tr.n - 2, k))
        f = max(0.0, min(1.0, u - k))
        return k, f

    def _forces(self, t, s, aux, X, R, V, Wv, F, Tq, ds):
        E.System._forces(self, t, s, aux, X, R, V, Wv, F, Tq, ds)
        tr = self.tr
        nb = self.nb
        ow = 13 * nb + len(self.acts)
        oa = ow + E.NW
        k, f = self.ref_at(t)
        nj = len(tr.joints)
        # balance: the push and twist on the pelvis that would take it back to the written motion
        rg = tr.root_gains
        p = tr.pelvis
        (x0, q0, v0, w0), (x1, q1, v1, w1) = tr.root[k], tr.root[k + 1]
        xr, vr, wr = _lerp(x0, x1, f), _lerp(v0, v1, f), _lerp(w0, w1, f)
        qr = _lerp(q0, q1, f)
        qn = math.sqrt(sum(c * c for c in qr))
        qr = tuple(c / qn for c in qr)
        o = 13 * p
        qp = (s[o + 3], s[o + 4], s[o + 5], s[o + 6])
        qe = E.qmul(qr, E.qconj(qp))
        sg = -2.0 if qe[0] < 0.0 else 2.0
        er = (sg * qe[1], sg * qe[2], sg * qe[3])                 # the turn from where the pelvis is to where it should be (world)
        ep = E.sub(xr, X[p]); ev = E.sub(vr, V[p]); ew = E.sub(wr, Wv[p])
        Fr = (rg["kpos_h"] * ep[0] + rg["kdpos"] * ev[0], rg["kpos_h"] * ep[1] + rg["kdpos"] * ev[1], rg["kpos_v"] * ep[2] + rg["kdpos"] * ev[2])
        Mr = (rg["krot"] * er[0] + rg["kdrot"] * ew[0], rg["krot"] * er[1] + rg["kdrot"] * ew[1], rg["krot"] * er[2] + rg["kdrot"] * ew[2])
        wl = tr.sup[k][0] + f * (tr.sup[k + 1][0] - tr.sup[k][0])
        wsup = (wl, wl, wl, 1.0 - wl, 1.0 - wl, 1.0 - wl)
        extra = [E.ZERO] * nj
        for li, j in enumerate(tr.legs):
            name, a, b, fam, k0, na = tr.joints[j]
            # the joint's place: the child's end of it (the engine's own joint point)
            ja, ra, jb, rb = tr.m.jp[name]
            xj = E.add(X[jb], E.mv(R[jb], rb))
            mom = E.add(Mr, E.cross(E.sub(X[p], xj), Fr))
            extra[j] = E.scale(mom, -wsup[li] * tr.fb_scale)       # on the child: the reaction of what the pelvis's side is to get
        p_mus = p_damp = 0.0
        row = [] if self.trace is not None else None
        for j, (name, a, b, fam, k0, na) in enumerate(tr.joints):
            kp, kd = tr.gains[fam]
            g = tr.gain[k][j] + f * (tr.gain[k + 1][j] - tr.gain[k][j])
            kp *= g * tr.fb_scale; kd *= g * tr.fb_scale
            oa_, ob_ = 13 * a, 13 * b
            qa = (s[oa_ + 3], s[oa_ + 4], s[oa_ + 5], s[oa_ + 6])
            qb = (s[ob_ + 3], s[ob_ + 4], s[ob_ + 5], s[ob_ + 6])
            qrel = E.qmul(E.qconj(qa), qb)
            qref = _lerp(tr.qref[k][j], tr.qref[k + 1][j], f)
            qn = math.sqrt(sum(c * c for c in qref))
            qref = tuple(c / qn for c in qref)
            qe = E.qmul(qref, E.qconj(qrel))
            sg = -2.0 if qe[0] < 0.0 else 2.0
            e_par = (sg * qe[1], sg * qe[2], sg * qe[3])           # in the parent's frame
            wrel_w = E.sub(Wv[b], Wv[a])
            wrel = E.mtv(R[a], wrel_w)
            wref = _lerp(tr.wref[k][j], tr.wref[k + 1][j], f)
            tau_par = (kp * e_par[0] + kd * (wref[0] - wrel[0]), kp * e_par[1] + kd * (wref[1] - wrel[1]), kp * e_par[2] + kd * (wref[2] - wrel[2]))
            tau = E.add(E.mv(R[a], tau_par), extra[j])            # the correction, as a torque on the child (world)
            ns = []
            for q in range(na):
                ax = tr.axes[k0 + q]
                if ax["kind"] == 0:
                    n = E.mv(R[a], ax["ax"])
                elif ax["kind"] == 1:
                    n = E.mv(R[b], ax["ax"])
                else:
                    n = self.float_axis(E.mv(R[a], ax["ax"]), E.mv(R[b], ax["axb"]))
                ns.append(n)
            # the correction split over the joint's muscle axes
            if na == 1:
                c = [E.dot(tau, ns[0])]
            elif na == 2:
                g11, g12, g22 = E.dot(ns[0], ns[0]), E.dot(ns[0], ns[1]), E.dot(ns[1], ns[1])
                b1, b2 = E.dot(tau, ns[0]), E.dot(tau, ns[1])
                det = g11 * g22 - g12 * g12
                c = [(g22 * b1 - g12 * b2) / det, (g11 * b2 - g12 * b1) / det]
            else:
                g = [[E.dot(ns[i], ns[jj]) + (REG3 if i == jj else 0.0) for jj in range(3)] for i in range(3)]
                bb = [E.dot(tau, ns[i]) for i in range(3)]
                c00 = g[1][1] * g[2][2] - g[1][2] * g[2][1]
                c01 = g[1][2] * g[2][0] - g[1][0] * g[2][2]
                c02 = g[1][0] * g[2][1] - g[1][1] * g[2][0]
                det = g[0][0] * c00 + g[0][1] * c01 + g[0][2] * c02
                c11 = g[0][0] * g[2][2] - g[0][2] * g[2][0]
                c12 = g[0][1] * g[2][0] - g[0][0] * g[2][1]
                c22 = g[0][0] * g[1][1] - g[0][1] * g[1][0]
                c = [(c00 * bb[0] + c01 * bb[1] + c02 * bb[2]) / det, (c01 * bb[0] + c11 * bb[1] + c12 * bb[2]) / det, (c02 * bb[0] + c12 * bb[1] + c22 * bb[2]) / det]
            for q in range(na):
                iax = k0 + q
                ax = tr.axes[iax]
                n = ns[q]
                ffv = tr.ff[k][iax] + f * (tr.ff[k + 1][iax] - tr.ff[k][iax])
                want = ffv + c[q]
                om = E.dot(wrel_w, n)
                tp, ep_ = tr.tabs[ax["tabp"]]
                tm, em_ = tr.tabs[ax["tabm"]]
                availp = ax["Tp"] * E.fv_tab(tp, om, ep_)
                availm = ax["Tm"] * E.fv_tab(tm, -om, em_)
                ap, am = s[oa + 2 * iax], s[oa + 2 * iax + 1]
                # each muscle of the pair is asked for what the torque wanted needs beyond what its opposite number is
                # giving now (BRAKE of it: a muscle still tensed from a moment ago is braked by the other, as a quick
                # movement's antagonist burst does, which is how the pair's torque can fall as fast as it can rise)
                up = max(0.0, min(1.0, (want + BRAKE * am * availm) / availp)) if availp > SAT_EPS else (1.0 if want > 0.0 else 0.0)
                um = max(0.0, min(1.0, (BRAKE * ap * availp - want) / availm)) if availm > SAT_EPS else (1.0 if want < 0.0 else 0.0)
                ds[oa + 2 * iax] = (up - ap) / (E.TA_UP if up > ap else E.TA_DN)
                ds[oa + 2 * iax + 1] = (um - am) / (E.TA_UP if um > am else E.TA_DN)
                tmus = ap * availp - am * availm
                tdamp = -ax["damp"] * om
                tv = E.scale(n, tmus + tdamp)
                Tq[b] = E.add(Tq[b], tv)
                Tq[a] = E.sub(Tq[a], tv)
                p_mus += tmus * om
                p_damp += tdamp * om
                if row is not None:
                    row.append((want, ffv, tmus, ap, am, om))
        ds[ow] += p_mus
        ds[ow + 2] += p_damp
        if row is not None:
            self.trace.append(row)


# ---- the compiled run -------------------------------------------------------------------------------------------------
def build(force=False):
    os.makedirs(OUT, exist_ok=True)
    if force or not os.path.isfile(BIN) or os.path.getmtime(BIN) < os.path.getmtime(SRC):
        subprocess.run(["rustc", "-C", "opt-level=3", "-C", "linker=" + os.path.expanduser("~/.local/bin/zigcc"), "-o", BIN, SRC],
                       check=True, env=dict(os.environ, PATH=os.path.expanduser("~/.cargo/bin") + ":" + os.environ["PATH"]))


def tree_order(m):
    s, b = m.sys, m.b
    pairs = [(p[0], p[1]) for p in s.pts for _ in range(3)] + [(d[0], d[1]) for d in s.dots]
    order = []
    for na, nb_ in TREE_ORDER:
        order += [k for k, pr in enumerate(pairs) if pr == (b[na], b[nb_])]
    assert sorted(order) == list(range(len(pairs)))
    return order


def write_model(path, tr, st, aux, t0, nsteps, h, every):
    s = tr.m.sys
    w = []
    f = lambda *xs: " ".join(repr(float(x)) for x in xs)
    w.append("g " + f(*s.g))
    w.append("baum " + f(s.baum_a, s.baum_b))
    w.append("consts " + f(E.TA_UP, E.TA_DN, E.ECC_SLOPE, E.CONTACT_D0, REG3, SAT_EPS, tr.fb_scale, BRAKE))
    w.append("bodies %d" % s.nb)
    for mm, I in zip(s.m, s.I):
        w.append(f(mm, *I))
    w.append("pts %d" % len(s.pts))
    for a, b, ra, rb in s.pts:
        w.append("%d %d %s %s" % (a, b, f(*ra), f(*rb)))
    w.append("dots %d" % len(s.dots))
    for a, b, ua, wb in s.dots:
        w.append("%d %d %s %s" % (a, b, f(*ua), f(*wb)))
    order = tree_order(tr.m)
    w.append("order %d %s" % (len(order), " ".join(str(i) for i in order)))
    w.append("fvtabs %d" % len(tr.tabs))
    for tab, ecc in tr.tabs:
        w.append("%d %s %s" % (len(tab), f(ecc), " ".join(f(*pt) for pt in tab)))
    w.append("stops %d" % len(s.stops))
    for name, a, b, ax, ea, fb, lo, hi, k, c, kind, axb in s.stops:
        assert kind != 3
        w.append("%d %d %s %s %s %s %d %s" % (a, b, f(*ax), f(*ea), f(*fb), f(lo, hi, k, c), kind, f(*axb)))
    w.append("cons %d" % len(s.contacts))
    for name, b, p, kn, cn, kt, ct, mu in s.contacts:
        w.append("%d %s %s" % (b, f(*p), f(kn, cn, kt, ct, mu)))
    nc = len(s.contacts)
    w.append("anchors " + " ".join("0 0.0 0.0" if a is None else "1 " + f(*a) for a in aux[:nc]))
    w.append("joints %d" % len(tr.joints))
    for name, a, b, fam, k0, na in tr.joints:
        kp, kd = tr.gains[fam]
        ja, ra, jb, rb = tr.m.jp[name]
        w.append("%d %d %s %d %d %s" % (a, b, f(kp, kd), na, jb, f(*rb)))
        for ax in tr.axes[k0:k0 + na]:
            w.append("%d %s %s %s %d %d %s" % (ax["kind"], f(*ax["ax"]), f(*ax["axb"]), f(ax["Tp"], ax["Tm"]), ax["tabp"], ax["tabm"], f(ax["damp"])))
    rg = tr.root_gains
    w.append("root %d %s %s" % (tr.pelvis, f(rg["kpos_h"], rg["kpos_v"], rg["kdpos"], rg["krot"], rg["kdrot"]), " ".join(str(j) for j in tr.legs)))
    w.append("ref %s %s %d" % (f(tr.t0), f(tr.dt), tr.n))
    for k in range(tr.n):
        row = []
        for j in range(len(tr.joints)):
            row += list(tr.qref[k][j]) + list(tr.wref[k][j]) + [tr.gain[k][j]]
        row += tr.ff[k]
        x, q, v, wv = tr.root[k]
        row += list(x) + list(q) + list(v) + list(wv) + [tr.sup[k][0]]
        w.append(f(*row))
    w.append("run %s %d %s %d" % (f(t0), nsteps, f(h), every))
    w.append("state %d %s" % (len(st), f(*st)))
    with open(path, "w") as fh:
        fh.write("\n".join(w) + "\n")


class Rec:
    __slots__ = ("t", "st", "con", "ax", "stops")


def run(tr, t_end=None, k0=0, st=None, h=H_STEP, every=REC_EVERY, tag="run"):
    """one forward run on the compiled engine from the written motion's sample k0 (at rest there unless st is given) to
    t_end. Returns (records, the failure's message or None). A record: .t, .st (the bodies' state), .con (each sole
    point's force), .ax (each muscle axis: asked N m, the plan's part, the muscles' torque, + level, - level, speed),
    .stops (each joint end's torque and reading)."""
    build()
    s = tr.m.sys
    st = tr.start_state(k0) if st is None else st
    aux = s.new_aux()
    s.anchors(st, aux)
    t0 = tr.ts[k0]
    t_end = tr.ts[-1] if t_end is None else t_end
    nsteps = int(round((t_end - t0) / h))
    path = os.path.join(OUT, "model_%s_%d.txt" % (tag, os.getpid()))
    write_model(path, tr, st, aux, t0, nsteps, h, every)
    try:
        r = subprocess.run([BIN, path], capture_output=True, text=True)
    finally:
        if "BB_KEEP_MODEL" not in os.environ:
            os.remove(path)
    n, nc, ns = len(st), len(s.contacts), len(s.stops)
    width = 1 + n + 3 * nc + 6 * tr.nax + 2 * ns
    recs = []
    for line in r.stdout.splitlines():
        v = [float(x) for x in line.split()]
        if len(v) != width:
            break
        rec = Rec()
        o = 1 + n
        rec.t, rec.st = v[0], v[1:o]
        rec.con = [tuple(v[o + 3 * i:o + 3 * i + 3]) for i in range(nc)]
        o += 3 * nc
        rec.ax = [tuple(v[o + 6 * i:o + 6 * i + 6]) for i in range(tr.nax)]
        o += 6 * tr.nax
        rec.stops = [tuple(v[o + 2 * i:o + 2 * i + 2]) for i in range(ns)]
        recs.append(rec)
    return recs, (r.stderr.strip() or "the compiled run failed") if r.returncode != 0 else None


def run_python(tr, nsteps, k0=0, st=None, h=H_STEP, trace=False):
    """the same run on the Python engine (slow: the check). Returns the states after each step (and the traces)."""
    sy = tr.system()
    st = tr.start_state(k0) if st is None else list(st)
    aux = sy.new_aux()
    sy.anchors(st, aux)
    out = [list(st)]
    t0 = tr.ts[k0]
    for i in range(nsteps):
        st = sy.step(t0 + i * h, st, aux, h)
        out.append(list(st))
    return out


# ---- practice: the plan learned from run to run ------------------------------------------------------------------------
FF_EVERY = 4                   # the plan is kept every 4 ms and read between by straight lines
LEARN, LEAD_S, LEARN_SMOOTH_S = 0.4, 0.012, 0.010     # [E] the share of a correction put into the plan; how much earlier (a muscle's rise); smoothing
FF_CAP = 1.3                   # the plan never asks a muscle for more than this x its strength held still
OK_PELVIS, OK_TURN = 0.05, 0.35                        # m, rad: the body is "still with the written motion" inside these
TIGHT_PELVIS, TIGHT_JOINT = 0.02, 12.0 * D2R             # m, rad: followed closely (the wrists are not counted: the bat is whipped, not placed)
EXTEND_S = 0.05                # s: each practice run goes this far past where the last one was still with the motion


def set_plan(tr, knots):
    """the plan from its knots (every FF_EVERY samples; the last knot at or after the end)."""
    tr.knots = [list(r) for r in knots]
    nk = len(knots)
    for k in range(tr.n):
        i = min(nk - 2, k // FF_EVERY)
        f = (k - i * FF_EVERY) / FF_EVERY
        tr.ff[k] = [a + f * (b - a) for a, b in zip(knots[i], knots[i + 1])]


def zero_plan(tr):
    nk = (tr.n - 1 + FF_EVERY - 1) // FF_EVERY + 1
    set_plan(tr, [[0.0] * tr.nax for _ in range(nk)])


def start_levels(tr, st, k=0):
    """the muscles' levels at the start: what the plan asks at that instant, held."""
    oa = len(st) - 2 * tr.nax
    for i, ax in enumerate(tr.axes):
        v = tr.ff[k][i]
        st[oa + 2 * i] = min(1.0, v / ax["Tp"]) if v > 0.0 else 0.0
        st[oa + 2 * i + 1] = min(1.0, -v / ax["Tm"]) if v < 0.0 else 0.0
    return st


def off_motion(tr, rec):
    """how far a record is from the written motion: (the pelvis's place m, the pelvis's turn rad, the worst joint rad)."""
    s = tr.m.sys
    k = min(tr.n - 1, max(0, int(round((rec.t - tr.t0) / tr.dt))))
    x, q, v, w = s.pose(rec.st, tr.pelvis)
    xr, qr = tr.root[k][0], tr.root[k][1]
    qe = E.qmul(qr, E.qconj(q))
    turn = 2.0 * math.sqrt(qe[1] ** 2 + qe[2] ** 2 + qe[3] ** 2)
    worst = 0.0
    for j, (name, a, b, fam, k0, na) in enumerate(tr.joints):
        qa, qb = s.pose(rec.st, a)[1], s.pose(rec.st, b)[1]
        qe = E.qmul(tr.qref[k][j], E.qconj(E.qmul(E.qconj(qa), qb)))
        if fam != "wrist":
            worst = max(worst, 2.0 * math.sqrt(qe[1] ** 2 + qe[2] ** 2 + qe[3] ** 2))
    return E.norm(E.sub(x, xr)), turn, worst


def forward(tr, t_end=None):
    """one forward run of the plan from the stance. Returns (records, failure or None)."""
    st = start_levels(tr, tr.start_state(0))
    return run(tr, t_end=t_end, st=st)


def practice(tr, rounds=60, log=print, t_first=None, learn=LEARN, until=None, patience=10):
    """practice: run, put a share of what the corrections added into the plan, run again. A run goes EXTEND_S further
    into the swing than the last one only once he follows the written motion closely that far (TIGHT), or after
    `patience` runs of the same length if he is at least still with it (OK_*). Returns the last run's records."""
    if not hasattr(tr, "knots"):
        zero_plan(tr)
    t_last = tr.ts[-1] if until is None else until
    horizon = min(t_last, (tr.t0 + 0.30) if t_first is None else t_first)
    best = None
    lead = int(round(LEAD_S / tr.dt))
    recs, same = None, 0
    for it in range(rounds):
        recs, err = forward(tr, t_end=horizon)
        n = len(recs)
        # how far into the run he is still with the written motion
        t_ok, worst_p, worst_j = recs[-1].t, 0.0, 0.0
        for rec in recs[::5]:
            dp, dq, dj = off_motion(tr, rec)
            if dp > OK_PELVIS or dq > OK_TURN:
                t_ok = rec.t
                break
            worst_p, worst_j = max(worst_p, dp), max(worst_j, dj)
        whole = t_ok >= recs[-1].t - 1e-9 and err is None
        # what the corrections added, axis by axis, smoothed, taken a muscle's rise earlier
        k_ok = min(n - 1, int(round((t_ok - tr.t0) / tr.dt)))
        size = 0.0
        new = [list(r) for r in tr.knots]
        for i, ax in enumerate(tr.axes):
            fb = [recs[k].ax[i][0] - recs[k].ax[i][1] for k in range(k_ok + 1)]
            fb = A._gauss(fb, LEARN_SMOOTH_S / tr.dt)
            size += sum(x * x for x in fb) / len(fb)
            cap_p, cap_m = FF_CAP * ax["Tp"], FF_CAP * ax["Tm"]
            for kn in range(len(new)):
                k = kn * FF_EVERY + lead
                if k > k_ok:
                    break
                fade = 1.0 if whole and recs[-1].t >= t_last - 1e-9 else min(1.0, max(0.0, (k_ok - k) / 40.0))   # no learning from the last 40 ms before he parted from it
                v = new[kn][i] + learn * fade * fb[k]
                new[kn][i] = max(-cap_m, min(cap_p, v))
        set_plan(tr, new)
        if log is not None:
            log("round %3d: ran to %+5.0f ms%s; with the written motion to %+5.0f ms (pelvis within %.1f cm, joints within %.1f deg); corrections %.1f N m rms"
                % (it + 1, 1e3 * recs[-1].t, " (FAILED: %s)" % err[:60] if err else "", 1e3 * t_ok, 100 * worst_p, worst_j / D2R, math.sqrt(size / tr.nax)))
        same += 1
        if whole and horizon < t_last - 1e-9 and ((worst_p < TIGHT_PELVIS and worst_j < TIGHT_JOINT) or same >= patience):
            horizon = min(t_last, horizon + EXTEND_S)
            same = 0
    return recs


def save_plan(tr, path, summary=None):
    d = {"kind": "swing-look plan 1", "what": "the torque planned about every muscle axis, every %d ms from %.3f s (swing_track.py); the motion it follows is swing_author.py's" % (FF_EVERY, tr.t0),
         "options": {"strength": tr.strength, "hip_side": tr.hip_side, "gains": tr.gains, "root_gains": tr.root_gains, "h": H_STEP},
         "t0": tr.t0, "dt_knot": tr.dt * FF_EVERY, "axes": [ax["plus"] + " / " + ax["minus"] for ax in tr.axes],
         "knots": [[round(x, 3) for x in r] for r in tr.knots], "summary": summary or {}}
    with open(path, "w") as fh:
        json.dump(d, fh, separators=(",", ":"))
    return path


def load_plan(path, log=None):
    d = json.load(open(path))
    o = d["options"]
    tr = Track(strength=o["strength"], hip_side=o.get("hip_side"), gains=o["gains"], root_gains=o["root_gains"], log=log)
    if len(d["knots"][0]) != tr.nax:
        raise SystemExit("the plan has %d muscle axes, the body %d" % (len(d["knots"][0]), tr.nax))
    set_plan(tr, d["knots"])
    return tr, d


# ---- the kept swing: one forward run, looked at ---------------------------------------------------------------------------
def polish(tr, rounds=12, window=(-0.30, 0.20), learn=LEARN, log=print):
    """practice over the swing itself only (the stance, load and stride are followed closely by the first plan): the
    corrections of each run inside `window` (s from the written contact) go into the plan, and of all the runs the plan
    is kept under which the barrel came squarest to the written contact with him still on the written path. Returns
    (the best run's records, its figures)."""
    lead = int(round(LEAD_S / tr.dt))
    best = None
    for it in range(rounds + 1):
        recs, err = forward(tr)
        fig = contact_of(tr, recs)
        end = off_motion(tr, recs[-1])[0] if err is None else 9.0
        score = abs(fig["late_ms"]) + 100.0 * fig["pelvis_off_max"] + 100.0 * end if fig["came"] else 1e9
        if log is not None:
            log("round %2d: the barrel square to the pitch %+.0f ms from the written contact at %.1f m/s (%.0f mph); the pelvis within %.1f cm of the written path to +100 ms, %.1f cm off it at the end%s"
                % (it, fig["late_ms"], fig["speed"], fig["speed"] / 0.44704, 100 * fig["pelvis_off_max"], 100 * end, "" if err is None else " FAILED"))
        if best is None or score < best[0]:
            best = (score, [list(r) for r in tr.knots], recs, fig)
        if it == rounds:
            break
        new = [list(r) for r in tr.knots]
        n = len(recs)
        for i, ax in enumerate(tr.axes):
            fb = A._gauss([recs[k].ax[i][0] - recs[k].ax[i][1] for k in range(n)], LEARN_SMOOTH_S / tr.dt)
            cap_p, cap_m = FF_CAP * ax["Tp"], FF_CAP * ax["Tm"]
            for kn in range(len(new)):
                k = kn * FF_EVERY + lead
                if k >= n:
                    break
                t = tr.ts[min(tr.n - 1, kn * FF_EVERY)]
                win = min(1.0, max(0.0, (t - window[0]) / 0.05)) * min(1.0, max(0.0, (window[1] - t) / 0.05))
                new[kn][i] = max(-cap_m, min(cap_p, new[kn][i] + learn * win * fb[k]))
        set_plan(tr, new)
    set_plan(tr, best[1])
    return best[2], best[3]


def contact_of(tr, recs):
    """when the barrel comes square to the pitch in a forward run: the instant the barrel's direction round (seen from
    above) passes the direction the written motion has it at its contact. {came, k (the record), late_ms (after the
    written contact), speed (the sweet spot's, m/s), at (where the sweet spot is), pelvis_off_max (m, to +100 ms)}."""
    m = tr.m
    k0 = int(round((0.0 - tr.t0) / tr.dt))
    want = m.observe(tr.states[k0])["bat_az"]
    out = dict(came=False, k=k0, late_ms=0.0, speed=0.0, at=(0.0, 0.0, 0.0), pelvis_off_max=0.0)
    prev = None
    for k in range(max(1, k0 - 80), min(len(recs), k0 + 150)):
        az = m.observe(recs[k].st)["bat_az"]
        d = (az - want + 180.0) % 360.0 - 180.0
        if prev is not None and prev < 0.0 <= d and abs(d - prev) < 90.0:
            p, v = m.bat_point(recs[k].st, C.BAT_SS)
            out.update(came=True, k=k, late_ms=1e3 * (recs[k].t - 0.0), speed=E.norm(v), at=p, attack=math.asin(max(-1.0, min(1.0, v[2] / E.norm(v)))) / D2R)
            break
        prev = d
    k1 = min(len(recs) - 1, k0 + 100)
    out["pelvis_off_max"] = max(off_motion(tr, recs[k])[0] for k in range(0, k1, 10))
    return out


def loads_of(tr, rec):
    """{L, R}: the share of his weight each foot carries in a record."""
    s = tr.m.sys
    BW = sum(s.m) * E.G
    fz = {"L": 0.0, "R": 0.0}
    for c, f in zip(s.contacts, rec.con):
        fz[c[0][0]] += f[2] / BW
    return fz


def swing_states(tr, recs, fig):
    """(times from the run's own contact, the bodies' states with their velocities) of a forward run."""
    tc = recs[fig["k"]].t if fig["came"] else 0.0
    nb13 = 13 * tr.m.sys.nb
    return [r.t - tc for r in recs], [r.st[:nb13] for r in recs]


def report(tr, recs, fig, log=print, step_ms=10.0):
    """the forward run, as a baseball man would read it (swing_author.table's columns), then where it parts from the
    written motion and which muscles are flat out."""
    ts, states = swing_states(tr, recs, fig)
    ts = [round(t, 6) for t in ts]
    rows = A.read(tr.m, ts, states, step_ms)
    A.table(rows, log)
    log("")
    log("t ms | front; rear foot (x his weight) | pelvis from the written path cm | worst joint from the written motion deg | muscles flat out")
    for k, rec in enumerate(recs):
        t_ms = ts[k] * 1e3
        if abs(t_ms / step_ms - round(t_ms / step_ms)) > 1e-6:
            continue
        fz = loads_of(tr, rec)
        dp, dq, dj = off_motion(tr, rec)
        sat = [tr.axes[i]["plus" if a[3] > a[4] else "minus"] for i, a in enumerate(rec.ax) if max(a[3], a[4]) > 0.97]
        log("%+5.0f | %.2f %.2f | %5.1f | %5.1f | %s" % (t_ms, fz["L"], fz["R"], 100 * dp, dj / D2R, " ".join(sat)))


def summary(tr, recs, fig):
    m = tr.m
    k0 = fig["k"]
    o = m.observe(recs[k0].st)
    fz = loads_of(tr, recs[k0])
    fastest = max(range(max(0, k0 - 60), min(len(recs), k0 + 80)), key=lambda k: m.observe(recs[k].st)["sweet_speed"])
    end = off_motion(tr, recs[-1])
    return {"contact_after_written_ms": round(fig["late_ms"], 1), "sweet_speed_mps": round(fig["speed"], 2), "sweet_speed_mph": round(fig["speed"] / 0.44704, 1),
            "sweet_at": [round(x, 3) for x in fig["at"]], "attack_deg": round(fig.get("attack", 0.0), 1), "barrel_below_level_deg": round(-o["bat_elev"], 1),
            "fastest_mps": round(m.observe(recs[fastest].st)["sweet_speed"], 2), "fastest_ms_from_contact": round(1e3 * (recs[fastest].t - recs[k0].t)),
            "buckle_deg": round(o["pelvis_rot"]), "chest_deg": round(o["trunk_rot"]), "front_foot_share": round(fz["L"], 2), "rear_foot_share": round(fz["R"], 2),
            "front_knee_deg": round(o["lead_knee_flex"]), "rear_knee_deg": round(o["trail_knee_flex"]), "pelvis_off_at_end_cm": round(100 * end[0], 1),
            "ran_to_ms": round(1e3 * recs[-1].t)}


def export_swing(tr, recs, fig, path, plan_name="swing_look"):
    ts, states = swing_states(tr, recs, fig)
    s = tr.m.sys
    loads = [[c[2] for c in r.con] for r in recs]
    tc = recs[fig["k"]].t if fig["came"] else 0.0
    ev = [(n, t - tc, why) for n, t, why in A.EVENTS if n not in ("contact", "end", "start")]
    ev = [("start", ts[0], "the stance (the run's first instant)")] + ev + [("contact", 0.0, "the barrel comes square to the pitch (seen from above)"), ("end", ts[-1], "the run's last record")]
    extra = ["# THE FORCE-DRIVEN RUN: every pose below was computed by the body engine with muscles and the ground only (swing_track.py).",
             "strength_x %.3g hips_sideways_x %.3g" % (tr.strength, tr.hip_side), "contact_after_written_ms %.1f" % fig["late_ms"]]
    return A.write_swing(path, tr.m, ts, states, "THE FORCE-DRIVEN SWING (PLAN D48): muscles and the ground move him; the motion they follow was written by hand",
                         "computed", ev, loads=loads, pc=fig["at"] if fig["came"] else None, extra=extra, name=plan_name)


def frames(tr, recs, fig, step_ms=10.0, t_from=None, t_to=None):
    ts, states = swing_states(tr, recs, fig)
    out = []
    for k, rec in enumerate(recs):
        t_ms = ts[k] * 1e3
        if abs(t_ms / step_ms - round(t_ms / step_ms)) > 1e-6:
            continue
        if (t_from is not None and ts[k] < t_from - 1e-9) or (t_to is not None and ts[k] > t_to + 1e-9):
            continue
        o = tr.m.observe(states[k])
        out.append((ts[k], states[k], loads_of(tr, rec), "buckle %+.0f  chest %+.0f" % (o["pelvis_rot"], o["trunk_rot"])))
    return out


def main():
    a = sys.argv[1:]
    arg = lambda k, dflt: a[a.index(k) + 1] if k in a else dflt
    if not a:
        print(__doc__)
        return
    if "--practice" in a:
        if "--from" in a:
            tr, d = load_plan(arg("--from", None), log=print)
        else:
            tr = Track(log=print)
        recs = practice(tr, rounds=int(arg("--rounds", 60)), t_first=float(arg("--first", tr.t0 + 0.30)), until=float(arg("--until", tr.ts[-1])),
                        patience=int(arg("--patience", 10)))
        out = arg("--out", os.path.join(OUT, "plan.json"))
        save_plan(tr, out)
        print("kept " + out)
        return
    if "--polish" in a:
        tr, d = load_plan(arg("--polish", None), log=print)
        recs, fig = polish(tr, rounds=int(arg("--rounds", 12)), window=(float(arg("--from", -0.30)), float(arg("--to", 0.20))), learn=float(arg("--learn", LEARN)))
        out = arg("--out", os.path.join(OUT, "plan.json"))
        save_plan(tr, out, summary(tr, recs, fig))
        print("kept " + out)
        return
    if "--check" in a:
        tr = Track()
        zero_plan(tr)
        k0 = min(range(tr.n), key=lambda k: abs(tr.ts[k] + 0.060))
        n = 40
        recs, err = run(tr, t_end=tr.ts[k0] + n * H_STEP, k0=k0, every=1)
        py = run_python(tr, n, k0=k0)
        worst = max(max(abs(x - y) for x, y in zip(recs[i].st, py[i])) for i in range(n + 1))
        print("the compiled run against the Python one over %d steps from %+.0f ms: the largest difference of any state entry %.2e (%s)" % (n, 1e3 * tr.ts[k0], worst, "ok" if worst < 1e-9 else "NOT OK"))
        sys.exit(0 if worst < 1e-9 else 1)
    tr, d = load_plan(a[0], log=None)
    recs, err = forward(tr)
    fig = contact_of(tr, recs)
    if err is not None:
        print("THE RUN FAILED at %+.0f ms: %s" % (1e3 * recs[-1].t, err))
    sm = summary(tr, recs, fig)
    print("%s: the barrel comes square to the pitch %.0f ms after the written contact; the sweet spot there %.1f m/s (%.0f mph), rising %.0f deg, the barrel %.0f deg below level; fastest %.1f m/s %+d ms from it"
          % (os.path.basename(a[0]), sm["contact_after_written_ms"], sm["sweet_speed_mps"], sm["sweet_speed_mph"], sm["attack_deg"], sm["barrel_below_level_deg"], sm["fastest_mps"], sm["fastest_ms_from_contact"]))
    step = float(arg("--step", 10))
    if "--table" in a:
        report(tr, recs, fig, step_ms=step)
    if "--draw" in a:
        fr = frames(tr, recs, fig, step, float(arg("--from", -9)), float(arg("--to", 9)))
        for p_ in A.draw(tr.m, fr, arg("--draw", "."), pc=fig["at"] if fig["came"] else None, tag="forces",
                         title="THE FORCE-DRIVEN RUN: muscles and the ground move him (under each foot: the share of his weight it carries)"):
            print("drew " + p_)
    if "--swing" in a:
        print("wrote " + export_swing(tr, recs, fig, arg("--swing", None), os.path.basename(a[0])))


if __name__ == "__main__":
    main()
