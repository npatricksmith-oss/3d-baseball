"""The batter's swing learned by practice on his own body (docs/body-motion.md 4.10; PLAN D32, D36, D37).

    python3 analysis/research/body/practice3d.py --staged main --reach 15 --iters 25    four starts: reach the ball, then the whole aim (job queue)
    python3 analysis/research/body/practice3d.py --continue <json> --iters 30           a start's result carried on; writes practice3d_main.json
    python3 analysis/research/body/practice3d.py --vary a,b,...    one thing changed at a time, from practice3d_main.json
    python3 analysis/research/body/practice3d.py --show <json>     the checks for a saved plan
    python3 analysis/research/body/practice3d.py --main            the search as first written: it parks the bat at the run's end (4.10)

NOTHING HERE IS FITTED TO A RECORDED SWING. The search is asked for what a hitter is trying to do with a stated
pitch: meet it, the bat on the ball's line, square, as fast as the body allows, effort counted, on his feet, no arm
drive before he could have decided (body-motion 4.10, step 1, written before any run). What keeps it a human swing
is the body: strengths inside 4.5's bounds, rises of at least 50 ms, the joints' ranges as stops (body.add_ranges).
The checks at the end are never given to the search.

The start is still the state placed at -250 ms from [4] (body.place): AN INPUT, FLAGGED AS [4]'s IN EVERY RESULT;
the --vary runs named start_* move it. Times are printed relative to contact, which is a result.
"""
import json
import math
import multiprocessing
import os
import random
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import probe3d as P                                    # noqa: E402
from probe3d import E, B, Dr                           # noqa: E402
import fit3d as F                                      # noqa: E402  (the channel lists and the saved plans' format only)

C = B.C
CH, NP, LOWER, ARMS = F.CH, F.NP, F.LOWER, F.ARMS
T0, T_END, H_STEP, EVERY = -0.250, 0.200, 5e-4, 10     # T0 is only the placed start's label on [4]'s clock. The step is 0.5 ms:
#   with the joints' stiff stops a violent swing does not integrate at 1 ms (body-motion 4.10); every result is rerun at 0.125 ms
#   The run is 450 ms long (350 until session 8's fix): a search on the corrected body brought the bat to the ball exactly at
#   the end of a 350 ms run, where nothing after contact is seen. Contact counts only if the 50 ms after it lie inside the run.
VERIFY_H = 1.25e-4
WORKERS = 4                                            # processes for the search's finite differences (inside one queued job)
FALLBACK = False                                       # lm(): take the best single difference when no damped step is accepted (--fallback)
# ---- the pitch (4.10): the game's average four-seam fastball where it is met (sim-cli pitch --type FF, 200 pitches)
BALL_SPEED, BALL_DESCENT, BALL_ACROSS, BALL_FLIGHT, BALL_Z = 37.98, 5.27, 1.62, 0.403, 0.78
MEET_ACROSS, MEET_AHEAD = 0.92, 0.72                   # m from the hips' midpoint at the start (PLAN D28, D29; AHEAD [E: 0.61-0.84])
# ---- the aim's units (4.10) ----
MISS_UNIT = 0.01              # m [E]
TIMING_ERR = 0.007            # s: the measured spread of where hitters meet the ball, as a time (note 6)
SQUARE_UNIT = 10.0            # deg [E]
SPEED_ASK, SPEED_W = 45.0, 1.0   # m/s out of reach; 1 m/s is worth 1 cm of miss [E]
EFFORT_W = 4.0                # one unit of effort (every muscle at full strength all swing) costs 4 m/s [E]; tried at 0, 1, 16
XCOM_UNIT, JUMP_MIN, JUMP_UNIT, SINK_MIN, SINK_UNIT, AFTER = 0.02, 0.10, 0.05, 0.90, 0.02, 0.050
T_VIEW, T_LATENCY, CLOCK_UNIT = 0.100, 0.050, 0.002   # s: [E 0.05-0.15]; agent-models 3.4 [E]; per 2 ms
STRAIN_FREE, STRAIN_UNIT = 50.0, 50.0                 # N m [E]: the diagnosis run that counts a ligament's load
HANDS_U = 0.15                # m from the knob: where the hands' speed is read (Blast's point [E])
USED = 0.05                   # a muscle group counts as used above this share of its bound
# the strongest batter: each bound at the top of its tested range in body-motion 4.5
STRONGEST = {"hip_ext_R": 524.0 / 430.1, "hip_ext_L": 524.0 / 430.1, "core": 290.0 / 260.0, "shoulder_R": 136.0 / Dr.SHOULDER_HADD,
             "shoulder_L": 112.0 / Dr.SHOULDER_HABD, "hip_add_R": 1.4, "hip_add_L": 1.4, "hip_rot_R": 110.0 / 78.0, "hip_rot_L": 110.0 / 78.0}
FORWARD_ARMS = ("shoulder_L", "shoulder_R", "elbow_ext_L", "elbow_ext_R", "wrist_L", "wrist_R", "sh_irot_R", "sh_erot_L",
                "sh_lower_L", "sh_lower_R", "pron_R", "sup_L")
VARY = {
    "effort_0": {"effort_w": 0.0}, "effort_1": {"effort_w": 1.0}, "effort_16": {"effort_w": 16.0},
    "stops_2000": {"k_big": 2000.0}, "stops_6000": {"k_big": 6000.0},
    "hips_turn_44": {"rom": {"hip_irot": 44.4, "hip_erot": 44.2}},
    "wrists_middle": {"wrists": "middle"},
    "start_trunk_coiled_10_more": {"start": {"trunk_rot": -10.0}},
    "start_rear_elbow_15_more_bent": {"start": {"trail_elbow_flex": 15.0}},
    "start_weight_5cm_back": {"start": {"pelvis_x_cm": -5.0}},
    "start_velocities_halved": {"vel": 0.5}, "start_at_rest": {"vel": 0.0},
    "strongest": {"bound_scale": STRONGEST}, "small": {"H": 1.75, "M": 77.0},
    "strain_counted": {"strain_w": 1.0},
    "meet_061": {"ahead": 0.61}, "meet_084": {"ahead": 0.84},
    "wrists_mirrored": {"wrists": "mirrored"}, "wrists_square": {"wrists": "square"},
    "arms_held_by_resolving": {"hold": "resolve"},
    # body-motion 4.11, part 2: a tensed muscle held until its onset, the clock reading onsets, at the two viewing times
    "view_010": {"hold": "let_go", "t_view": 0.10}, "view_020": {"hold": "let_go", "t_view": 0.20},
    # part 3: the feet on dirt, and the same with the old grip alone put back (to see the grip's share apart)
    "cleats": {"hold": "let_go", "t_view": 0.10, "feet": "cleats"}, "cleats_grip_08": {"hold": "let_go", "t_view": 0.10, "feet": "cleats", "mu": 0.8},
    # the reach one coordinate at a time, from the high-and-close ball (review P2-1)
    "height_alone_078": {"z": 0.78, "across": 0.75, "ahead": 0.40},
    "ahead_alone_072": {"z": 1.20, "across": 0.75, "ahead": 0.72},
    "across_alone_092": {"z": 1.20, "across": 0.92, "ahead": 0.40},
    # diagnoses added after the main run (body-motion 4.10, step 2): is it where the ball is that defeats the body?
    "ball_at_the_top_of_the_zone": {"z": 1.05},
    "ball_high_and_close": {"z": 1.20, "across": 0.75, "ahead": 0.40},     # about where the earlier swings' bats went; no pitch a hitter is thrown
}


def hull(pts):
    pts = sorted(set(pts))
    if len(pts) < 3:
        return pts
    cr = lambda o, a, b: (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    lo, up = [], []
    for p in pts:
        while len(lo) >= 2 and cr(lo[-2], lo[-1], p) <= 0:
            lo.pop()
        lo.append(p)
    for p in reversed(pts):
        while len(up) >= 2 and cr(up[-2], up[-1], p) <= 0:
            up.pop()
        up.append(p)
    return lo[:-1] + up[:-1]


def outside(poly, p):
    """how far the point is outside the convex polygon (counter-clockwise), m; 0 inside."""
    d = 0.0
    n = len(poly)
    for i in range(n):
        a, b = poly[i], poly[(i + 1) % n]
        ex, ey = b[0] - a[0], b[1] - a[1]
        l = math.hypot(ex, ey)
        d = max(d, ((p[0] - a[0]) * ey - (p[1] - a[1]) * ex) / l)      # + to the right of the edge = outside
    if d <= 0.0:
        return 0.0
    best = 1e9
    for i in range(n):
        a, b = poly[i], poly[(i + 1) % n]
        ex, ey = b[0] - a[0], b[1] - a[1]
        t = max(0.0, min(1.0, ((p[0] - a[0]) * ex + (p[1] - a[1]) * ey) / (ex * ex + ey * ey)))
        best = min(best, math.hypot(p[0] - a[0] - t * ex, p[1] - a[1] - t * ey))
    return best


def clock_residuals(fired, tx, t_avail, form="each"):
    """the score's charge for arm muscles that fire before the batter could have decided (body-motion 4.10 item 7, 4.11).
    fired: [(when the arm muscle first drives, s; its asked strength as a share of its bound)] for each arm muscle used;
    tx: contact; t_avail: how long before contact an arm muscle may fire. Returns one residual per entry.
    form "each" (since session 9's review, P1-1): each muscle's OWN overrun, max(0, (tx - t_avail) - its onset), per
    CLOCK_UNIT, weighted by its share of the arms' asked strength (the weights sum to 1; the residual carries the
    weight's square root). With every muscle tied at one onset the sum of squares is the old form's.
    form "earliest" (sessions 8 and 9): the earliest onset's overrun alone, in the first entry. With two muscles tied
    at the earliest, moving one of them later changes nothing in it, so a search saw no slope there."""
    if not fired:
        return []
    if form == "earliest":
        return [max(0.0, (tx - t_avail) - min(t for t, w in fired)) / CLOCK_UNIT] + [0.0] * (len(fired) - 1)
    tot = sum(w for t, w in fired)
    return [math.sqrt(w / tot) * max(0.0, (tx - t_avail) - t) / CLOCK_UNIT for t, w in fired]


class Ctx:
    """one body with its ranges, its start and the pitch, reused for every swing of a search."""

    def __init__(self, **o):
        self.o = o
        H, M = o.get("H", 1.84), o.get("M", 92.0)
        # grip: "weld", the body of sessions 1-8 (the default, so that a saved result without the option replays on the body
        # it was found on); "hands", each hand's own grip (body-motion 4.11, session 9; --grip hands)
        # feet: "plate" (sessions 1-8, the default) or "cleats" (4.11, part 3: the measured grip, the ankles' ends; --feet cleats)
        # wrist: "box" (session 9, the default) or "oval" (4.12: both of a wrist's angles inside its ellipse; --wrist oval)
        # clear: the cleared start of 4.12 (body.START_CLEAR: [4]'s launch posture moved by the rule so that every joint is
        # 5 deg inside its end; no stop moved; --clear). STILL [4]'s LAUNCH POSTURE, MOVED BY A RULE, AND FLAGGED AS THAT.
        self.m = m = B.Model(H, M, ranges=(), grip=o.get("grip", "weld"), feet=o.get("feet", "plate"), mu=o.get("mu"),
                             wrist=o.get("wrist", "box"), shift=B.START_CLEAR if o.get("clear") else None)
        for k, shift in o.get("start", {}).items():          # the start moved by a stated amount (a sensitivity run)
            B.CURVE_OVERRIDE[k] = (lambda kk, sh: (lambda t: C.resample(B._CURVES.setdefault(kk, C.load(kk)), t, t, 1.0)[0] + sh))(k, shift)
        vel = o.get("vel", 1.0)
        st = B.place(m, T0, moving=vel != 0.0)
        B.CURVE_OVERRIDE.clear()
        if vel not in (0.0, 1.0):
            for i in range(m.sys.nb):
                for c in range(7, 13):
                    st[13 * i + c] *= vel
        self.aux0 = list(m.start_aux)
        Dr.BOUND_SCALE.clear(); Dr.BOUND_SCALE.update(o.get("bound_scale", {}))
        self.bd = Dr.bounds(H, M)
        # hold="let_go" (body-motion 4.11, part 2): every muscle's level beforehand is held with no net torque until its
        # onset, and the hold then lets go (engine.actuator's held). The clock then reads onsets only.
        st = Dr.add_drives(m, st, {n: (0.0, 0.0, 0.05, 0.1, 0.0) for n in CH}, held=o.get("hold") == "let_go")
        Dr.BOUND_SCALE.clear()
        self.limits = B.add_ranges(m, st, k_big=o.get("k_big"), wrists=o.get("wrists", "cocked"), rom=o.get("rom"))
        self.tone, self.tone_rest = Dr.solve_tone(m, st, T0, aux=list(self.aux0))
        # An arm muscle tensed beforehand (review P2-4). It COUNTS UNDER THE CLOCK: a starting level is drive from the
        # first instant, so in residuals() it is an arm drive that rose at the start. The other way, hold="resolve",
        # re-solves the hold-still torques for each plan's starting levels at the arms so that a tensed muscle moves
        # nothing at the start (drive.hold_response; linear, so its pieces are found once here). It is not the default
        # because the hold-still torque that balanced the muscle then stays on all swing and takes that share of the
        # muscle's strength away (the fit to [4]'s plan replayed that way has the bat at 2.4 m/s). A muscle that is
        # tensed, holds, and then fires with all its strength needs a drive that carries no net torque before its
        # onset, which the engine does not have (body-motion 4.10).
        if o.get("hold") == "resolve":
            self.hold_w = Dr.hold_response(m, st, T0, list(self.aux0), ARMS, self.bd)
            self.tone0 = [c for j in Dr.JOINTS for c in self.tone[j]]
        # stand: "tone" (sessions 2-10, the default: solve_tone's constant hold-still torques, under no muscle rule) or
        # "muscle" (body-motion 4.13, rung 2): each hold-still torque handed to the muscle group that acts that way, as a
        # holding drive under that group's strength-at-speed curve (drive.solve_hold_muscles). hold_T: each group's
        # holding strength; the plan's burst may ask at most the bound less it. What no measured muscle takes stays a
        # torque (tone_left) and is reported apart.
        self.hold_T, self.hold_at_start, self.tone_left, self.hold_ix = {}, {}, {}, {}
        if o.get("stand") == "muscle":
            holds, self.tone_left, self.hold_at_start = Dr.solve_hold_muscles(m, st, self.tone, self.bd)
            st = Dr.add_hold_muscles(m, st, holds, self.tone_left)
            self.hold_T = {n: v[0] for n, v in holds.items()}
            self.hold_ix = {a_[0][len(Dr.HOLD_PREFIX):]: j for j, a_ in enumerate(m.sys.acts) if a_[0].startswith(Dr.HOLD_PREFIX)}
            self.hold_bd = dict(self.bd, **{n: v[5] * (M * E.G * H) / Dr.REF_WH for n, v in Dr.HOLD_ONLY.items()})
        self.cap = {n: self.bd[n] - self.hold_T.get(n, 0.0) for n in CH}      # the most a burst may ask of each group
        # damp: None (sessions 1-10: 1.0 N m s/rad at every joint, capped at 30 N m) or drive.damping_of()'s other forms
        # (a set of drive.DAMPING by name; body-motion 4.13)
        Dr.add_damping(m, st, o.get("damp"))
        self.damp_table, self.damp_cap = Dr.damping_of(o.get("damp"))
        self.st = st
        self.BW = M * E.G
        self.order = P.tree_order(m)
        self.box = []
        for n in CH:
            self.box += [(n, "T", 0.0, self.cap[n]), (n, "on", T0, 0.050), (n, "rise", Dr.RISE_MIN, 0.250),
                         (n, "off", -0.150, 0.150), (n, "start", 0.0, 1.0)]
        self.free = [i for i, (n, f, lo, hi) in enumerate(self.box) if hi > lo and self.bd[n] > 0.0]
        k = H / 1.84                                          # [E] a smaller batter stands and is pitched to in proportion
        xp = m.sys.pose(st, m.b["pelvis"])[0]
        self.Pc = (xp[0] + o.get("across", MEET_ACROSS) * k, xp[1] + o.get("ahead", MEET_AHEAD) * k, o.get("z", BALL_Z) * k)
        d, a = BALL_DESCENT * B.D2R, BALL_ACROSS * B.D2R
        self.nb = (math.cos(d) * math.sin(a), -math.cos(d) * math.cos(a), -math.sin(d))
        self.ev = E.unit(E.sub((0.0, 0.0, 1.0), E.scale(self.nb, self.nb[2])))    # up, square to the ball's line
        self.el = E.cross(self.nb, self.ev)                                       # level, square to it
        self.effort_w = o.get("effort_w", EFFORT_W)
        self.strain_w = o.get("strain_w", 0.0)
        self.t_avail = BALL_FLIGHT - o.get("t_view", T_VIEW) - T_LATENCY      # how long before contact an arm muscle may fire
        self.z0 = B.com(m, st)[0][2]
        self.evals = 0
        # h: the integration step of this search's runs (body-motion 4.12: 0.25 ms; sessions 8 and 9: H_STEP, 0.5 ms).
        # Records stay 5 ms apart whatever the step.
        self.h = o.get("h", H_STEP)

    plan = F.Ctx.plan
    u_of = F.Ctx.u_of

    def prepare(self, u):
        st, aux = F.Ctx.prepare(self, u)
        if self.hold_ix:
            # a holding drive goes when the plan switches its muscle off (4.13, rung 2); a hold-only group never does
            pl = self.plan(u)
            for n, j in self.hold_ix.items():
                self.m.sys.acts[j][8] = pl[n][3] if n in pl else 1e9
        if self.o.get("hold") != "resolve":
            return st, aux
        pl = self.plan(u)
        tv = list(self.tone0)
        for n in ARMS:
            x = pl[n][0] * pl[n][4] / self.bd[n] if self.bd[n] > 0 else 0.0
            if x > 0.0:
                tv = [a + x * b for a, b in zip(tv, self.hold_w[n])]
        s = self.m.sys
        s.tones = [(nm, a, b, tuple(tv[3 * k:3 * k + 3])) for k, (nm, a, b, _) in enumerate(s.tones)]
        return st, aux

    run_end, failed_after = T_END, None

    def run(self, u, every=None, t_end=T_END, h=None):
        """the plan's run, as records. A run is not failed by what happens after the last instant the score reads
        (body-motion 4.11; session 9's review, P2-2): if the run fails but the sweet spot crossed the ball's plane at
        least AFTER before the last record made, the records to the failure are returned (they hold everything the score
        and the report read) and failed_after says how long after contact the last record was, with the failure's
        message. A failure before that is raised, as before. run_end is where the records end, for contact() and nearest()."""
        st, aux = self.prepare(u)
        self.evals += 1
        h = getattr(self, "h", H_STEP) if h is None else h
        every = int(round(EVERY * H_STEP / h)) if every is None else every
        n = int(round((t_end - T0) / h))
        self.run_end, self.failed_after = t_end, None
        try:
            return P.run_raw(self.m.sys, st, T0, n, h, every, aux, self.order)
        except E.SolveError as e:
            recs = getattr(e, "recs", None)
            if recs:
                self.run_end = recs[-1][0]
                tx, p, v, i, came = self.contact(recs)
                if came:
                    self.failed_after = (recs[-1][0] - tx, str(e))
                    return recs
            self.run_end = t_end
            raise

    def contact(self, recs):
        """(t_x, sweet spot position, velocity, index of the record before, came round?) where the sweet spot crosses
        the plane square to the ball's line at the meeting point, by cubic interpolation between records."""
        m = self.m
        prev = None
        for i, rec in enumerate(recs):
            p, v = m.bat_point(rec[1], C.BAT_SS)
            a = -E.dot(E.sub(p, self.Pc), self.nb)
            if rec[0] > self.run_end - AFTER + 1e-9:
                break                                    # too late for the 50 ms after contact to be seen: not a contact
            if prev is not None and prev[0] < 0.0 <= a and -E.dot(v, self.nb) > 0.0:
                a0, p0, v0, t0 = prev
                h = rec[0] - t0
                herm = lambda q: tuple((2 * q ** 3 - 3 * q ** 2 + 1) * p0[c] + (q ** 3 - 2 * q ** 2 + q) * h * v0[c]
                                       + (-2 * q ** 3 + 3 * q ** 2) * p[c] + (q ** 3 - q ** 2) * h * v[c] for c in range(3))
                lo, hi = 0.0, 1.0
                for _ in range(40):
                    q = 0.5 * (lo + hi)
                    lo, hi = (q, hi) if -E.dot(E.sub(herm(q), self.Pc), self.nb) < 0.0 else (lo, q)
                q = 0.5 * (lo + hi)
                vq = tuple(v0[c] + q * (v[c] - v0[c]) for c in range(3))
                return t0 + q * h, herm(q), vq, i - 1, True
            prev = (a, p, v, rec[0])
        p, v = m.bat_point(recs[-1][1], C.BAT_SS)
        return recs[-1][0], p, v, len(recs) - 2, False

    def nearest(self, recs):
        """(time, sweet spot position, velocity, index of the record before) where the sweet spot comes nearest the
        ball's place in the run, by cubic interpolation between records: the reach stage's reading (4.10)."""
        m = self.m
        recs = [rec for rec in recs if rec[0] <= self.run_end - AFTER + 1e-9]      # the 50 ms after it must lie inside the run
        pv = [m.bat_point(rec[1], C.BAT_SS) for rec in recs]
        k = min(range(len(recs)), key=lambda q: E.norm(E.sub(pv[q][0], self.Pc)))
        best = (E.norm(E.sub(pv[k][0], self.Pc)), recs[k][0], pv[k][0], pv[k][1], max(0, k - 1))
        for i in (k - 1, k):
            if i < 0 or i + 1 >= len(recs):
                continue
            (p0, v0), (p1, v1) = pv[i], pv[i + 1]
            h = recs[i + 1][0] - recs[i][0]
            herm = lambda q: tuple((2 * q ** 3 - 3 * q ** 2 + 1) * p0[c] + (q ** 3 - 2 * q ** 2 + q) * h * v0[c]
                                   + (-2 * q ** 3 + 3 * q ** 2) * p1[c] + (q ** 3 - q ** 2) * h * v1[c] for c in range(3))
            dist = lambda q: E.norm(E.sub(herm(q), self.Pc))
            lo, hi = 0.0, 1.0
            for _ in range(40):
                q1, q2 = lo + (hi - lo) / 3.0, hi - (hi - lo) / 3.0
                lo, hi = (lo, q2) if dist(q1) < dist(q2) else (q1, hi)
            q = 0.5 * (lo + hi)
            if dist(q) < best[0]:
                best = (dist(q), recs[i][0] + q * h, herm(q), tuple(v0[c] + q * (v1[c] - v0[c]) for c in range(3)), i)
        return best[1], best[2], best[3], best[4]

    def balance(self, recs, tx):
        """(the extrapolated centre of mass's largest distance outside the feet's area from contact to AFTER, m;
        the least the feet carry between them at any instant, of body weight; the centre of mass's height at the
        end of that, of its height at the start)."""
        m, s = self.m, self.m.sys
        worst, least, zend = 0.0, 1e9, None
        for t, state, con, act, stop, sec in recs:
            if t > tx + AFTER + 1e-9:
                break
            least = min(least, sum(f[2] for f in con) / self.BW)
            if t >= tx - H_STEP * EVERY:
                x, v = B.com(m, state)
                w0 = math.sqrt(E.G / max(0.3, x[2]))
                pts = []
                for name, b, p, *_ in s.contacts:
                    xb, qb, _, _ = s.pose(state, b)
                    pc = E.add(xb, E.mv(E.qR(qb), p))
                    pts.append((pc[0], pc[1]))
                worst = max(worst, outside(hull(pts), (x[0] + v[0] / w0, x[1] + v[1] / w0)))
                zend = x[2]
        return worst, least, (zend if zend is not None else B.com(m, recs[-1][1])[0][2]) / self.z0

    def effort(self, recs, tx, pl):
        """{channel: mean over the swing of (level x strength / bound) squared}."""
        acc = {n: 0.0 for n in CH}
        k = 0
        held = self.o.get("hold") == "let_go"
        for t, state, con, act, stop, sec in recs:
            if t > tx + 1e-9:
                break
            k += 1
            for n, a in zip(CH, act):
                if self.bd[n] <= 0:
                    continue
                # with the posture held by muscle (4.13, rung 2) the group's level is its burst's and its holding drive's together
                acc[n] += ((a[2] * pl[n][0] + (act[self.hold_ix[n]][2] * self.hold_T[n] if n in self.hold_ix else 0.0)) / self.bd[n]) ** 2
                if held and pl[n][4] > 0.0:
                    # what holds a tensed muscle is working too, at the muscle's own level, until it has let go (4.11)
                    left = 1.0 if t < pl[n][1] else math.exp(-(t - pl[n][1]) / E.TA_REL)
                    acc[n] += (min(a[2], pl[n][4] * left) * pl[n][0] / self.bd[n]) ** 2
            for n, j in self.hold_ix.items():
                if n not in acc:                        # a hold-only group (the trunk's bending): booked with the trunk's rotators
                    acc["core"] += (act[j][2] * self.hold_T[n] / self.hold_bd[n]) ** 2
        return {n: v / max(1, k) for n, v in acc.items()}

    def arm_fired(self, pl):
        """[(channel, when it first drives, its asked strength over its bound)] for every arm muscle used (strength over
        1 % of its bound). When it first drives is its onset; under the rule of sessions 8 and part 1 (no hold) a muscle
        tensed beforehand drives from the first instant (a starting level over 1 % of its bound; review P2-4 of session 8)."""
        held = self.o.get("hold") in ("resolve", "let_go")          # a held muscle has not fired until its onset
        out = []
        for n in ARMS:
            if self.bd[n] > 0 and pl[n][0] > 0.01 * self.bd[n]:
                t = pl[n][1] if (held or pl[n][0] * pl[n][4] <= 0.01 * self.bd[n]) else T0
                out.append((n, t, pl[n][0] / self.bd[n]))
        return out

    def arm_onsets(self, pl):
        """when each arm muscle used first drives (arm_fired)."""
        return [t for n, t, w in self.arm_fired(pl)]

    def clock(self, pl, tx, on=True):
        """the clock's residuals, one for each of the arm channels (zero for one not used, or when on is False):
        clock_residuals() in the form this search uses (clock="each" unless the options say "earliest")."""
        fired = self.arm_fired(pl) if on else []
        vals = clock_residuals([(t, w) for n, t, w in fired], tx, self.t_avail, self.o.get("clock", "each"))
        by = {n: v for (n, t, w), v in zip(fired, vals)}
        if self.o.get("clock", "each") == "earliest":               # the one term, in the first place
            return [vals[0] if vals else 0.0] + [0.0] * (len(ARMS) - 1)
        return [by.get(n, 0.0) for n in ARMS]

    def residuals(self, u):
        try:
            recs = self.run(u)
        except E.SolveError:
            return None
        pl = self.plan(u)
        if self.o.get("stage") == "reach":
            # the first stage of a staged search (4.10): only bring the sweet spot to the ball's place, in balance, inside
            # the clock, effort counted. Read where the sweet spot comes nearest the ball, so there is no cliff between a
            # bat that reaches the ball's plane and one that does not. No speed, no line, no squareness asked.
            tx, p, v, i = self.nearest(recs)
            d = E.sub(p, self.Pc)
            r = [E.dot(d, self.ev) / MISS_UNIT, E.dot(d, self.el) / MISS_UNIT, 0.0, 0.0, 0.0, 0.0, E.dot(d, self.nb) / MISS_UNIT]
            eff = self.effort(recs, tx, pl)
            r += [math.sqrt(self.effort_w * eff[n] / len(CH)) for n in CH]
            xo, least, zr = self.balance(recs, tx)
            r += [xo / XCOM_UNIT, max(0.0, JUMP_MIN - least) / JUMP_UNIT, max(0.0, SINK_MIN - zr) * self.z0 / SINK_UNIT]
            r += self.clock(pl, tx)
            return None if any(x != x for x in r) else r
        if self.o.get("contact") == "nearest":
            # the whole aim with no cliff (4.10, session 8's fix): contact is read where the sweet spot comes nearest the
            # ball's place, which is the plane's crossing when the bat passes through the ball, so a bat that has not
            # reached the plane is scored by the same rule as one that has. The report still judges by the crossing.
            tx, p, v, i = self.nearest(recs)
            came = True
        else:
            tx, p, v, i, came = self.contact(recs)
        d = E.sub(p, self.Pc)
        vpar = -E.dot(v, self.nb)
        share = BALL_SPEED / (BALL_SPEED + max(1.0, vpar))
        bz = self.m.axis(recs[i][1], "bat", (0, 0, 1))
        sq = math.asin(max(-1.0, min(1.0, (bz[0] * self.nb[0] + bz[1] * self.nb[1]) / max(1e-9, math.hypot(bz[0], bz[1]) * math.hypot(self.nb[0], self.nb[1]))))) / B.D2R
        r = [E.dot(d, self.ev) / MISS_UNIT, E.dot(d, self.el) / MISS_UNIT,
             E.dot(v, self.ev) * TIMING_ERR * share / MISS_UNIT, E.dot(v, self.el) * TIMING_ERR * share / MISS_UNIT,
             sq / SQUARE_UNIT,
             math.sqrt(SPEED_W * max(0.0, SPEED_ASK - vpar)),
             E.dot(d, self.nb) / MISS_UNIT if (not came or self.o.get("contact") == "nearest") else 0.0]   # how far short along the line
        eff = self.effort(recs, tx, pl)
        r += [math.sqrt(self.effort_w * eff[n] / len(CH)) for n in CH]
        xo, least, zr = self.balance(recs, tx)
        r += [xo / XCOM_UNIT, max(0.0, JUMP_MIN - least) / JUMP_UNIT, max(0.0, SINK_MIN - zr) * self.z0 / SINK_UNIT]
        # the clock is read against a contact that happened; a bat that never comes round is charged for that, above,
        # and not for the clock against the run's end (review P2-3)
        r += self.clock(pl, tx, came)
        if self.strain_w > 0.0:
            names = [s_[0] for s_ in self.m.sys.stops]
            for j, nm in enumerate(names):
                pk = max(abs(rec[4][j][0]) for rec in recs if rec[0] <= tx + AFTER + 1e-9)
                r.append(0.0 if nm == "shoulder_L" else math.sqrt(self.strain_w) * max(0.0, pk - STRAIN_FREE) / STRAIN_UNIT)
        return None if any(x != x for x in r) else r


NEAR_BOUND = 0.90             # a muscle group asked for this share of its bound or more is reported as near its full strength
NEAR_FLOOR = 0.010            # s: a rise within this of the 50 ms floor is reported as near it


def hold_work(s, recs, t_end):
    """the work of the two things the engine books as one integral, apart, each by name, from the first record to
    t_end (session 10's review, P1-1 and P1-2): the constant hold-still torques (system.tones: fixed in the parent's
    frame, no strength-at-speed law, no onset, not counted as effort or under the clock) and the joints' own damping
    (system.holds: -C x the joint's speed, its size capped). Each power is worked from the record's state by the engine's
    own law and integrated by the trapezoid rule over the records given, so give it every step.
    Returns ({hold-still torque: J}, {damping: J}, {damping: its largest torque, N m})."""
    Rof = lambda st, i: E.qR(tuple(st[13 * i + 3:13 * i + 7]))
    wrel = lambda st, a, b: E.sub(tuple(st[13 * b + 10:13 * b + 13]), tuple(st[13 * a + 10:13 * a + 13]) if a >= 0 else E.ZERO)
    tone, damp, peak = {x[0]: 0.0 for x in s.tones}, {x[0]: 0.0 for x in s.holds}, {x[0]: 0.0 for x in s.holds}
    prev = None
    for rec in recs:
        t, st = rec[0], rec[1]
        if t > t_end + 1e-12:
            break
        pt, pd = {}, {}
        for name, a, b, tl in s.tones:
            pt[name] = E.dot(E.mv(Rof(st, a), tl) if a >= 0 else tl, wrel(st, a, b))
        for name, a, b, qref, K, Cc, cap in s.holds:
            wr = wrel(st, a, b)
            wl = E.mtv(Rof(st, a), wr) if a >= 0 else wr
            tl = (-Cc[0] * wl[0], -Cc[1] * wl[1], -Cc[2] * wl[2])       # the damping alone (the body's holds have no stiffness)
            mag = E.norm(tl)
            if mag > cap:
                tl = E.scale(tl, cap / mag)
            pd[name] = E.dot(tl, wl)
            peak[name] = max(peak[name], min(mag, cap))
        if prev is not None:
            dt = t - prev[0]
            for k in pt:
                tone[k] += 0.5 * (pt[k] + prev[1][k]) * dt
            for k in pd:
                damp[k] += 0.5 * (pd[k] + prev[2][k]) * dt
        prev = (t, pt, pd)
    return tone, damp, peak


def terms(ctx, r):
    """the score (the sum of the residuals' squares) by term."""
    n = len(CH)
    sq = lambda xs: sum(x * x for x in xs)
    out = {"miss": sq(r[0:2]), "off the ball's line": sq(r[2:4]), "bat not square": sq(r[4:5]), "speed short of 45 m/s": sq(r[5:6]),
           "never came round": sq(r[6:7]), "effort": sq(r[7:7 + n]), "balance": sq(r[7 + n:10 + n]),
           "the arms' clock": sq(r[10 + n:10 + n + len(ARMS)])}
    if len(r) > 10 + n + len(ARMS):
        out["load on a joint's end"] = sq(r[10 + n + len(ARMS):])
    return out


_CTX = None


def _res(u):
    return _CTX.residuals(u)


def lm(ctx, u, iters, log, pool):
    """Levenberg-Marquardt on the residuals over the free numbers, projected onto the box (fit3d.lm, with the
    finite differences spread over the pool). Returns (u, cost, the cost after each iteration)."""
    free = ctx.free
    r = ctx.residuals(u)
    cost = sum(x * x for x in r)
    lam, fd, hist = 1e-2, 0.02, [cost]
    for it in range(iters):
        ups, steps = [], []
        for i in free:
            up = list(u)
            step = fd if u[i] + fd <= 1.0 else -fd
            up[i] = u[i] + step
            ups.append(up); steps.append(step)
        rps = pool.map(_res, ups, chunksize=max(1, len(ups) // (4 * WORKERS)))
        J = [[0.0] * len(r) if rp is None else [(a - b) / step for a, b in zip(rp, r)] for rp, step in zip(rps, steps)]
        n = len(free)
        JtJ = [[sum(a * b for a, b in zip(J[i], J[k])) for k in range(i + 1)] for i in range(n)]
        g = [sum(a * b for a, b in zip(J[i], r)) for i in range(n)]
        improved, gain = False, 0.0
        for _ in range(8):
            A = [[JtJ[i][k] + (lam * (JtJ[i][i] + 1e-6) if i == k else 0.0) for k in range(i + 1)] for i in range(n)]
            try:
                dlt = E.chol_solve(A, [-x for x in g])
            except E.SolveError:
                lam *= 10.0
                continue
            un = list(u)
            for i, di in zip(free, dlt):
                un[i] = min(1.0, max(0.0, u[i] + max(-0.25, min(0.25, di))))
            rn = ctx.residuals(un)
            cn = None if rn is None else sum(x * x for x in rn)
            if cn is not None and cn < cost:
                gain = cost - cn
                u, r, cost = un, rn, cn
                lam = max(1e-6, lam / 3.0)
                improved = True
                break
            lam *= 4.0
        if not improved and FALLBACK:
            # no damped step was accepted. Before taking the differences closer in: the differences themselves are 205
            # plans already run, each one number away; if the best of them is lower, take it (one coordinate step). Added
            # in session 9 after searches on the hands' body stopped within 3-16 iterations with lower scores beside them
            # and no failed runs (body-motion 4.11); off (FALLBACK = False) it is the search of session 8.
            cs = [(sum(x * x for x in rp), k) for k, rp in enumerate(rps) if rp is not None]
            if cs and min(cs)[0] < cost:
                cb, kb = min(cs)
                gain = cost - cb
                u, r, cost = ups[kb], rps[kb], cb
                improved = True
                lam = 1e-2
                log("      (no damped step accepted: the best single difference taken, %s of %s)" % (ctx.box[free[kb]][1], ctx.box[free[kb]][0]))
        hist.append(cost)
        log("      iteration %2d: cost %.3f (miss %.1f, %.1f cm; speed along the line %.2f m/s), lambda %.0e"
            % (it + 1, cost, r[0] * MISS_UNIT * 100, r[1] * MISS_UNIT * 100, SPEED_ASK - r[5] ** 2 / SPEED_W, lam))
        if not improved:
            if fd <= 0.0051:
                break
            fd *= 0.5                                    # a refused step: the differences are taken closer in, twice, before giving up
            lam = 1e-2
            log("      (no step accepted: the differences now over %.3f of each number's range)" % fd)
        elif gain < 1e-6 * cost:
            break
    return u, cost, hist


def search(ctx, plan, iters, log=print):
    global _CTX
    _CTX = ctx
    t = time.time()
    with multiprocessing.get_context("fork").Pool(WORKERS) as pool:
        u, cost, hist = lm(ctx, ctx.u_of(plan), iters, log, pool)
    log("   %d iterations, %.0f s; cost %.3f; over the last 5 iterations it fell by %.3f"
        % (len(hist) - 1, time.time() - t, cost, hist[max(0, len(hist) - 6)] - hist[-1]))
    return u, cost, hist


def hand_plan(ctx, level, seed=None):
    """a plan that owes nothing to any fit: the legs and trunk at `level` of their bounds from the start, the arm
    muscles that swing the bat forward at `level` from 150 ms after it, rises 80 ms; with a seed, EVERY muscle at
    `level` and every timing moved at random by up to 50 ms."""
    rng = random.Random(seed)
    j = (lambda: rng.uniform(-0.050, 0.050)) if seed is not None else (lambda: 0.0)
    pl = {}
    for n in CH:
        fwd = n in LOWER[:-1] or n in FORWARD_ARMS or seed is not None
        on = T0 if n in LOWER else T0 + 0.150
        pl[n] = ((level if fwd else 0.0) * ctx.bd[n], max(T0, on + j()), max(Dr.RISE_MIN, 0.080 + j()), 0.100 + j(), 0.0)
    return pl


def unwrap(v):
    out = [v[0]]
    for x in v[1:]:
        while x - out[-1] > 180.0:
            x -= 360.0
        while x - out[-1] < -180.0:
            x += 360.0
        out.append(x)
    return out


def report(ctx, u, log=print):
    """the practised swing against the checks of body-motion 4.10 (never given to the search). One run, 1 ms records."""
    m, s = ctx.m, ctx.m.sys
    recs = ctx.run(u, every=int(round(1e-3 / ctx.h)))
    pl = ctx.plan(u)
    tx, p, v, i, came = ctx.contact(recs)
    out = {"plan": {n: list(x) for n, x in pl.items()}, "bounds": dict(ctx.bd), "options": {k: x for k, x in ctx.o.items()},
           "came_round": came, "start_to_contact_ms": (tx - T0) * 1e3, "limits": ctx.limits}
    rel = lambda t: (t - tx) * 1e3
    d = E.sub(p, ctx.Pc)
    sp = E.norm(v)
    vpar = -E.dot(v, ctx.nb)
    obs = [(t, m.observe(st)) for t, st, *_ in recs]
    at = lambda t_: min(range(len(recs)), key=lambda k: abs(recs[k][0] - t_))
    ic = at(tx)
    out["miss_cm"] = (E.dot(d, ctx.ev) * 100, E.dot(d, ctx.el) * 100)
    out["sweet_speed"] = sp
    out["sweet_speed_along"] = vpar
    out["attack_deg"] = math.asin(max(-1.0, min(1.0, v[2] / sp))) / B.D2R
    out["across_deg"] = math.asin(max(-1.0, min(1.0, E.dot(v, ctx.el) / sp))) / B.D2R
    bz = m.axis(recs[ic][1], "bat", (0, 0, 1))
    out["bat_square_deg"] = math.asin(max(-1.0, min(1.0, (bz[0] * ctx.nb[0] + bz[1] * ctx.nb[1]) / (math.hypot(bz[0], bz[1]) * math.hypot(ctx.nb[0], ctx.nb[1]))))) / B.D2R
    out["bat_elev_deg"] = obs[ic][1]["bat_elev"]
    log("   FROM THE START PLACED ON [4]'s STATE%s (an input, flagged): contact %.0f ms after the start%s" % (", MOVED OFF ITS JOINTS' ENDS BY THE RULE OF 4.12" if ctx.o.get("clear") else "", (tx - T0) * 1e3, "" if came else "  (THE BAT NEVER REACHED THE BALL; figures are at the run's end)"))
    out["failed_after_contact_ms"] = None if ctx.failed_after is None else ctx.failed_after[0] * 1e3
    if ctx.failed_after is not None:
        log("   THE RUN FAILED %.0f ms AFTER CONTACT, after everything the score and these checks read (\"%s\"); the records to there are used"
            % (ctx.failed_after[0] * 1e3, ctx.failed_after[1][:90]))
    log("   the ball: missed by %+.1f cm up, %+.1f cm along the bat; the bat %+.1f deg from square; barrel %.0f deg below level" % (out["miss_cm"] + (out["bat_square_deg"], -out["bat_elev_deg"])))
    sw = [(E.norm(m.bat_point(st, C.BAT_SS)[1]), t) for t, st, *_ in recs]
    before = max(x for x in sw if x[1] <= tx)
    after = max(sw)
    out["sweet_fastest_before"] = (before[0], rel(before[1]))
    out["sweet_fastest"] = (after[0], rel(after[1]))
    log("   CHECK the sweet spot at contact: %.1f m/s (%.0f mph), %.1f along the ball's line   [29-34 m/s, 65-76 mph: %s]" % (sp, sp / 0.44704, vpar, "inside" if 29.0 <= sp <= 34.0 else "OUTSIDE"))
    log("   CHECK the bat at its fastest at contact: %.0f %% of its fastest before (%.1f m/s at %.0f ms); fastest of the whole run %.1f at %+.0f ms   [at least 98 %%: %s]"
        % (100 * sp / before[0], before[0], rel(before[1]), after[0], rel(after[1]), "met" if sp >= 0.98 * before[0] else "NOT MET"))
    hs = [(E.norm(m.bat_point(st, HANDS_U)[1]), t) for t, st, *_ in recs]
    hpk = max(x for x in hs if x[1] <= tx + 1e-9)
    out["hands_fastest"] = (hpk[0], rel(hpk[1]))
    out["hands_at_contact"] = hs[ic][0]
    log("   CHECK the hands' fastest speed: %.1f m/s (%.0f mph) at %.0f ms; %.1f at contact   [10.3-13.0 m/s: %s]" % (hpk[0], hpk[0] / 0.44704, rel(hpk[1]), hs[ic][0], "inside" if 10.3 <= hpk[0] <= 13.0 else "OUTSIDE"))
    pg, vg = m.bat_point(recs[ic][1], B.GRIP_L)
    dirn = E.unit(v)
    out["split"] = (E.dot(vg, dirn), sp - E.dot(vg, dirn))
    log("      the split at contact: the hands' travel %.1f m/s + the bat's turn about them %.1f" % out["split"])
    log("   CHECK the attack angle: %+.1f deg (the ball's line asks +%.1f)   [major leaguers +8 to +10; OpenBiomechanics +4.1 +- 5.9, p10-p90 -3.3 to +10.7]" % (out["attack_deg"], BALL_DESCENT))
    # swing time: from when the hands last rise through a tenth of their fastest speed
    k = max(range(len(hs)), key=lambda q: hs[q][0] if hs[q][1] <= tx + 1e-9 else -1.0)
    while k > 0 and hs[k][0] > 0.1 * hpk[0]:
        k -= 1
    used = lambda n: ctx.bd[n] > 0 and pl[n][0] > USED * ctx.bd[n]
    first = {}
    for grp, names in (("legs", [n for n in LOWER if not n.startswith("core")]), ("trunk", ["core"]), ("arms", list(ARMS))):
        ons = [(pl[n][1], n) for n in names if used(n)]
        first[grp] = min(ons) if ons else (float("nan"), "-")
    out["bat_first_motion_ms"] = rel(hs[k][1])
    out["first_rise_ms"] = {g: (rel(x[0]), x[1]) for g, x in first.items()}
    log("   CHECK swing time: the hands first move (a tenth of their fastest) %.0f ms before contact%s; the first arm muscle rises %.0f ms before   [Blast 149 ms, major leaguers 130-170; college hitters off a tee 215 +- 44; the pitch allows the arms %.0f]"
        % (-rel(hs[k][1]), " (already moving at the start: a floor)" if k == 0 else "", -rel(first["arms"][0]), ctx.t_avail * 1e3))
    order = first["legs"][0] <= first["trunk"][0] <= first["arms"][0]
    log("   CHECK the order of first rises: legs %.0f ms (%s), trunk %.0f (%s), arms %.0f (%s)   [legs, trunk, arms: %s]"
        % (rel(first["legs"][0]), first["legs"][1], rel(first["trunk"][0]), first["trunk"][1], rel(first["arms"][0]), first["arms"][1], "met" if order else "NOT MET"))
    tensed = {n: pl[n][4] for n in CH if used(n) and pl[n][4] > 0.05}
    log("      tensed beforehand: " + (", ".join("%s %.2f" % kv for kv in tensed.items()) or "none"))
    ix = {n: j for j, n in enumerate(CH)}
    eff_at = lambda n, t_: recs[at(t_)][3][ix[n]][2] * (pl[n][0] / ctx.bd[n] if ctx.bd[n] > 0 else 0.0)
    mean_eff = lambda n, a, b_: (lambda vv: sum(vv) / max(1, len(vv)))([rec[3][ix[n]][2] for rec in recs if a <= rec[0] <= b_]) * (pl[n][0] / ctx.bd[n] if ctx.bd[n] > 0 else 0.0)
    t1 = hs[k][1]
    th = (tx - t1) / 3.0
    out["emg"] = {"lead_sweep": [eff_at("shoulder_L", T0)] + [eff_at("shoulder_L", t1 + q * th) for q in (0, 1, 2, 3)],
                  "lead_triceps": [mean_eff("elbow_ext_L", t1 + q * th, t1 + (q + 1) * th) for q in (0, 1, 2)],
                  "lead_raise_peak": max(rec[3][ix["sh_raise_L"]][2] for rec in recs if rec[0] <= tx) * (pl["sh_raise_L"][0] / ctx.bd["sh_raise_L"])}
    log("   CHECK the lead shoulder's sweep muscle (recorded 76-101 %% from before the swing): %.2f at the start; %s through the swing" % (out["emg"]["lead_sweep"][0], ", ".join("%.2f" % x for x in out["emg"]["lead_sweep"][1:])))
    log("   CHECK the lead triceps (recorded 92 %% early, 73 %% in the middle): early, middle, late %.2f, %.2f, %.2f;   the muscle raising the lead arm (recorded under 40 %%): peak %.2f"
        % (tuple(out["emg"]["lead_triceps"]) + (out["emg"]["lead_raise_peak"],)))
    # the legs
    fz = [(sum(f[2] for nm, f in zip(s.contacts, rec[2]) if nm[0][0] == "L") / ctx.BW, rec[0]) for rec in recs]
    fzr = [(sum(f[2] for nm, f in zip(s.contacts, rec[2]) if nm[0][0] == "R") / ctx.BW, rec[0]) for rec in recs]
    pk = max(x for x in fz if x[1] <= tx)
    kn = [o["lead_knee_flex"] for t, o in obs]
    i50 = at(tx - 0.050)
    rate = max((kn[q - 5] - kn[q + 5]) / 0.010 for q in range(5, ic - 4)) if ic > 12 else 0.0
    out["lead_foot_peak"] = (pk[0], rel(pk[1]))
    out["feet_at_contact"] = (fz[ic][0], fzr[ic][0])
    out["lead_knee"] = (kn[0], kn[i50], kn[ic], rate)
    out["trail_knee"] = (obs[0][1]["trail_knee_flex"], obs[ic][1]["trail_knee_flex"])
    log("   CHECK the front leg braced: the lead knee %.0f deg bent at the start, %.0f at -50 ms, %.0f at contact, straightening at up to %.0f deg/s   [46 at foot plant, 14 +- 12 at contact, 350 deg/s (OpenBiomechanics); 15 at contact ([3])]" % out["lead_knee"])
    log("   CHECK the front foot's greatest load: %.2f x body weight at %.0f ms; at contact front %.2f, rear %.2f   [2.0 +- 0.4, p10-p90 1.5-2.6]; the rear knee %.0f -> %.0f deg" % (pk[0], rel(pk[1]), fz[ic][0], fzr[ic][0], out["trail_knee"][0], out["trail_knee"][1]))
    # what each foot did on the ground (4.11, part 3): its heel's greatest height before contact, how far it turned on the
    # ground from the start to contact (+ is the toe going toward the pitcher), and the pelvis's turn over the same time
    out["feet"] = {}
    yaw_of = lambda body, state: math.atan2(*reversed(m.axis(state, body, (1, 0, 0))[:2])) / B.D2R
    for sd in "RL":
        fb = m.b["foot_" + sd]
        heel = [c_[2] for c_ in s.contacts if c_[0].startswith(sd + "_heel")]
        hz = []
        for rec in recs[:ic + 1]:
            xb, qb, _, _ = s.pose(rec[1], fb)
            Rf = E.qR(qb)
            hz.append(min(E.add(xb, E.mv(Rf, p_))[2] for p_ in heel))
        yw = unwrap([yaw_of("foot_" + sd, rec[1]) for rec in recs[:ic + 1]])
        out["feet"][sd] = (max(hz) * 100, hz[-1] * 100, yw[-1] - yw[0], min(yw) - yw[0], max(yw) - yw[0])
    pyaw = unwrap([o["pelvis_rot"] for t, o in obs[:ic + 1]])
    out["pelvis_turn"] = pyaw[-1] - pyaw[0]
    log("   the feet on the ground: the REAR heel's greatest height before contact %.1f cm (%.1f at contact), the rear foot turned %+.0f deg on the ground by contact (between %+.0f and %+.0f; + is the toe toward the pitcher); the FRONT heel %.1f cm (%.1f), turned %+.0f deg; the pelvis turned %+.0f deg in that time"
        % (out["feet"]["R"][:5] + out["feet"]["L"][:3] + (out["pelvis_turn"],)))
    for key, label, band in (("pelvis_rot_vel", "the hips' fastest turning", "705 +- 75 (OpenBiomechanics); 714 +- 76 at -75 ms ([3])"), ("trunk_rot_vel", "the trunk's fastest turning", "915 +- 80"), ("bat_az_vel", "the bat's fastest turning (seen from above)", "no direct measurement kept")):
        x = max((o[key], t) for t, o in obs if t <= tx + 1e-9)
        out[key + "_peak"] = (x[0], rel(x[1]))
        log("   CHECK %s: %.0f deg/s at %.0f ms; %.0f at contact   [%s]" % (label, x[0], rel(x[1]), obs[ic][1][key], band))
    ang = [B.range_angles(m, st) for t, st, *_ in recs[:ic + 51]]
    tw = [(a["trunk_twist"], recs[q][0]) for q, a in enumerate(ang) if recs[q][0] <= tx]
    sep = min(tw)
    out["separation"] = (-sep[0], rel(sep[1]), -tw[0][0], -tw[-1][0])
    log("   CHECK the trunk twisted back against the hips: most %.0f deg at %.0f ms (%.0f at the start, %.0f at contact)   [21 +- 6 at about -120 ms]" % out["separation"])
    out["elbows_at_contact"] = (obs[ic][1]["trail_elbow_flex"], obs[ic][1]["lead_elbow_flex"])
    log("   CHECK the elbows at contact: rear %.0f deg, lead %.0f   [79 +- 13, 53 +- 15]" % out["elbows_at_contact"])
    az = unwrap([o["lead_arm_az"] for t, o in obs])
    ar = max(((az[q + 5] - az[q - 5]) / 0.010, recs[q][0]) for q in range(5, ic - 4)) if ic > 12 else (0.0, tx)
    out["lead_arm_rate"] = (ar[0], rel(ar[1]))
    log("   CHECK the lead arm's turning rate: %.0f deg/s at %.0f ms   [1,160 +- 96 at -65 ms ([3])]" % out["lead_arm_rate"])
    # the work done on the body up to contact, from the engine's own integrals: no passive element may do net positive work
    ow = 13 * s.nb + len(s.acts)
    wk = [recs[ic][1][ow + q] - recs[0][1][ow + q] for q in range(4)]
    wk_end = recs[min(len(recs) - 1, ic + 50)][1][ow + 1] - recs[0][1][ow + 1]
    out["work_J"] = {"drives": wk[0], "stops": wk[1], "damping and hold-still torques": wk[2], "ground": wk[3], "stops to +50 ms": wk_end}
    log("   CHECK no passive element does net positive work: to contact the muscles did %.0f J, the joints' stops %+.0f J (%+.0f by 50 ms after), the ground %+.0f J; damping and the hold-still torques %+.0f J   [stops and ground at most 0: %s]"
        % (wk[0], wk[1], wk_end, wk[3], wk[2], "met" if wk[1] <= 0.5 and wk_end <= 0.5 and wk[3] <= 0.5 else "NOT MET"))
    # the hold-still torques' work apart from the damping's, by joint (session 10's review, P1-1, P1-2): read at EVERY step
    fine = ctx.run(u, every=1)
    tone_by, damp_by, damp_pk = hold_work(s, fine, tx)
    tw, dw = sum(tone_by.values()), sum(damp_by.values())
    out["hold_still_J"], out["damping_J"], out["damping_peak_Nm"] = tone_by, damp_by, damp_pk
    out["work_J"].update({"hold-still torques": tw, "damping": dw, "hold-still share of the positive work": tw / (wk[0] + tw) if wk[0] + tw > 0 else 0.0})
    srt = lambda d_: ", ".join("%s %+.1f" % kv for kv in sorted(d_.items(), key=lambda kv: -abs(kv[1])) if abs(kv[1]) >= 2.0)
    log("   THE HOLD-STILL TORQUES (constant, one a joint; they stand for muscle and follow no muscle rule: no strength-at-speed law, no onset, no effort, no clock) did %+.1f J to contact, %.0f %% of what they and the muscles put in; by joint: %s"
        % (tw, 100 * out["work_J"]["hold-still share of the positive work"], srt(tone_by)))
    dsc = ("%s" % (ctx.o.get("damp"),)) if ctx.o.get("damp") is not None else "[E], unsourced: %.1f N m s/rad on every joint axis" % Dr.JOINT_DAMP
    log("   THE JOINTS' OWN DAMPING (%s; %s) took %+.1f J to contact, %.0f %% of the muscles' work; by joint: %s; its largest torque: %s   (the two sum to %+.1f J; the engine's own integral of both: %+.1f)"
        % (dsc, "capped at %.0f N m" % ctx.damp_cap if ctx.damp_cap < Dr.NO_CAP else "no cap", dw, -100 * dw / wk[0] if wk[0] > 0 else 0.0, srt(damp_by), ", ".join("%s %.0f" % kv for kv in sorted(damp_pk.items(), key=lambda kv: -kv[1])[:6]), tw + dw, wk[2]))
    # balance, effort, the clock
    xo, least, zr = ctx.balance(recs, tx)
    out["balance"] = (xo * 100, least, zr)
    out["left_the_ground"] = least < 0.02
    if out["left_the_ground"]:
        log("   BOTH FEET LEFT THE GROUND at some instant before 50 ms after contact")
    rr = ctx.residuals(u)
    if rr is not None:
        out["score_terms"] = terms(ctx, rr)
        log("   the score, %.1f, by term: %s" % (sum(out["score_terms"].values()), "; ".join("%s %.1f" % kv for kv in out["score_terms"].items())))
    log("   the aim's balance: the centre of mass's heading %.1f cm outside the feet's area at worst (0 = inside); the feet never carry under %.2f of body weight; the centre of mass ends at %.2f of its starting height" % out["balance"])
    eff = ctx.effort(recs, tx, pl)
    out["effort"] = sum(eff.values()) / len(CH)
    out["effort_by_channel"] = eff
    on = ctx.arm_onsets(pl)
    out["clock_ms"] = (tx - min(on)) * 1e3 if on else None
    out["arms_tensed_beforehand"] = sorted(n for n in ARMS if pl[n][0] * pl[n][4] > 0.01 * ctx.bd[n])
    log("   effort %.3f (0 = nothing, 1 = everything flat out all swing); the first arm drive of any size, %s, is %.0f ms before contact (the clock allows %.0f); arm muscles tensed from the start: %s"
        % (out["effort"], "a tensed muscle held until its onset and not counted" if ctx.o.get("hold") == "let_go" else "a muscle tensed from the start counted",
           out["clock_ms"] or 0.0, ctx.t_avail * 1e3, ", ".join(out["arms_tensed_beforehand"]) or "none"))
    pairs = (("core", "core_back"), ("sh_raise_L", "sh_lower_L"), ("sh_raise_R", "sh_lower_R"), ("sh_irot_L", "sh_erot_L"), ("sh_irot_R", "sh_erot_R"),
             ("elbow_ext_L", "elbow_flex_L"), ("elbow_ext_R", "elbow_flex_R"), ("wrist_flex_L", "wrist_ext_L"), ("wrist_flex_R", "wrist_ext_R"),
             ("pron_L", "sup_L"), ("pron_R", "sup_R"), ("hip_abd_L", "hip_add_L"), ("hip_abd_R", "hip_add_R"), ("knee_ext_L", "knee_flex_L"),
             ("knee_ext_R", "knee_flex_R"), ("hip_ext_L", "hip_flex_L"))
    both = [(a, b_, math.sqrt(eff[a]), math.sqrt(eff[b_])) for a, b_ in pairs if eff[a] > 0.04 and eff[b_] > 0.04]
    out["opposed_pairs"] = both
    log("   opposing muscles both on (each over 0.2 of its bound, root mean square over the swing): " + ("; ".join("%s %.2f against %s %.2f" % (a, x, b_, y) for a, b_, x, y in both) or "none"))
    # the stops
    names = [x[0] for x in s.stops]
    out["stops"] = {}
    for j, nm in enumerate(names):
        lo, hi = ctx.limits[nm]
        # read at every step (session 10's review, P3-1: 1 ms records understate a peak by up to a tenth)
        to_after = [rec[4][j] for rec in fine if rec[0] <= tx + AFTER + 1e-9]
        to_contact = [rec[4][j] for rec in fine if rec[0] <= tx + 1e-9]
        tau = max(abs(x[0]) for x in to_after)
        over = max(max(x[1] / B.D2R - hi, lo - x[1] / B.D2R) for x in to_after)
        before_c = max(abs(x[0]) for x in to_contact)
        over_c = max(max(x[1] / B.D2R - hi, lo - x[1] / B.D2R) for x in to_contact)
        out["stops"][nm] = (tau, max(0.0, over), before_c, max(0.0, over_c))
    lean = {k_: x for k_, x in out["stops"].items() if x[0] > 1.0}
    log("   the joints' stops that carried load to 50 ms after contact (N m at most; deg past the end; then up to contact: N m at most; deg past the end): "
        + ("; ".join("%s %.0f, %.1f, %.0f, %.1f" % ((k_,) + x) for k_, x in sorted(lean.items(), key=lambda kv: -kv[1][0])) or "none"))
    out["ligaments_over_50"] = sorted(k_ for k_, x in out["stops"].items() if k_ != "shoulder_L" and x[0] > STRAIN_FREE)
    log("   LEANING ON A LIGAMENT (a stop other than the lead arm against the chest over %.0f N m): %s" % (STRAIN_FREE, ", ".join(out["ligaments_over_50"]) or "none"))
    out["unheld"] = {}
    hands = m.grip == "hands"
    if hands:
        # the hands' body (4.11): each wrist, forearm and upper arm in its clinical sense at the start, 50 ms before contact
        # and at contact, with hitters measured directly beside them (never given to the search; research note 11 A5)
        cl = [B.clinical(m, recs[q][1]) for q in (0, i50, ic)]
        out["arms_clinical"] = {"%s_%s" % k: [c_[k] for c_ in cl] for k in cl[0]}
        log("   the arm joints, clinical senses (toward the thumb +, toward the palm +, palm-down +, turned in +), at the start, -50 ms and contact:")
        meas = {("wrist_dev", "L"): "hitters +10 +- 10 at foot plant, -16 +- 9 at contact", ("wrist_flex", "L"): "-19 +- 18, +8 +- 10",
                ("wrist_dev", "R"): "+5 +- 12, -8 +- 12", ("wrist_flex", "R"): "-52 +- 14, -47 +- 17",
                ("forearm_to_barrel", "L"): "138-157 at contact (Gelinas 1988)"}
        ar = B.arm_range()
        for k in sorted(cl[0], key=lambda kk: (kk[1], kk[0])):
            log("      %-18s %s: %6.1f %6.1f %6.1f   range %s   %s" % (k[0], k[1], cl[0][k], cl[1][k], cl[2][k],
                                                                    "%.0f to %.0f" % ar[k[0]] if k[0] in ar else "-", "[" + meas[k] + "]" if k in meas else ""))
        # each wrist's place in its ellipse (4.12): 1 is the edge; at the start, at its largest up to contact, at contact; and on an ellipse tilted 8 deg
        out["wrist_oval"] = {}
        for sd in "LR":
            place_of = lambda st_: (lambda c_: B.oval_place(c_[("wrist_flex", sd)], c_[("wrist_dev", sd)])[0])(B.clinical(m, st_))
            rho = [place_of(rec[1]) for rec in recs[:ic + 1]]              # every 1 ms record to contact (P3-3: it was every fifth, and missed contact itself)
            after = [place_of(rec[1]) for rec in recs[ic:ic + 51]]
            c0_, c1_ = cl[0], cl[2]
            out["wrist_oval"][sd] = (rho[0], max(rho), rho[-1], B.oval_place(c0_[("wrist_flex", sd)], c0_[("wrist_dev", sd)], tilt=8.0)[0], max(after))
        log("   each wrist's place in its ellipse (1 = its edge): lead %.2f at the start, %.2f at most to contact, %.2f at contact, %.2f at most in the 50 ms after; rear %.2f, %.2f, %.2f, %.2f; at the start on an ellipse tilted 8 deg: lead %.2f, rear %.2f"
            % (out["wrist_oval"]["L"][:3] + out["wrist_oval"]["L"][4:] + out["wrist_oval"]["R"][:3] + out["wrist_oval"]["R"][4:] + (out["wrist_oval"]["L"][3], out["wrist_oval"]["R"][3])))
    for key, rng_ in ((("ankle_L", 67.3), ("ankle_R", 67.3)) if hands else (("pron_L", 161.9), ("pron_R", 161.9), ("sh_rot_L", 173.0), ("sh_rot_R", 173.0), ("ankle_L", 67.3), ("ankle_R", 67.3))):
        vv = unwrap([a[key] for a in ang])
        out["unheld"][key] = (max(vv) - min(vv), rng_)
    out["unheld_overrun"] = sorted(k_ for k_, x in out["unheld"].items() if x[0] > x[1])
    log("   not held, travel to 50 ms after contact against the real range, deg: " + "; ".join("%s %.0f of %.0f%s" % (k_, x[0], x[1], "  MORE THAN A BODY HAS" if x[0] > x[1] else "") for k_, x in out["unheld"].items()))
    onb = []
    for q, (nm, f, lo, hi) in enumerate(ctx.box):
        if q in ctx.free and f == "T" and u[q] >= 1.0 - 1e-6:
            onb.append(nm)
    out["at_full_strength"] = onb
    out["rises_at_floor"] = sum(1 for q, (nm, f, lo, hi) in enumerate(ctx.box) if f == "rise" and u[q] <= 1e-6 and used(nm))
    log("   at full strength (%d of %d): %s;  %d used muscles rise at the 50 ms floor" % (len(onb), len(CH), ", ".join(onb) or "none", out["rises_at_floor"]))
    # "exactly on its bound" says little of a search that is charged for effort and does not settle on a box's edge
    # (session 10's review, P1-3): the groups near their bounds, their largest activation, the rises near the floor
    near = {n: (pl[n][0] + ctx.hold_T.get(n, 0.0)) / ctx.bd[n] for n in CH if ctx.bd[n] > 0 and pl[n][0] + ctx.hold_T.get(n, 0.0) >= NEAR_BOUND * ctx.bd[n]}
    act_pk = {n: max(rec[3][ix[n]][2] for rec in recs[:ic + 1]) for n in near}
    floor = {n: pl[n][2] * 1e3 for n in CH if used(n) and pl[n][2] <= Dr.RISE_MIN + NEAR_FLOOR}
    out["near_full_strength"], out["near_full_strength_activation"], out["rises_near_floor_ms"] = near, act_pk, floor
    log("   asked for %.0f %% or more of its bound (%d of %d; its largest activation to contact): %s" % (100 * NEAR_BOUND, len(near), len(CH), ", ".join("%s %.3f (%.2f)" % (n, x, act_pk[n]) for n, x in sorted(near.items(), key=lambda kv: -kv[1])) or "none"))
    log("   used muscles whose rise is within %.0f ms of the %.0f ms floor: %s" % (NEAR_FLOOR * 1e3, Dr.RISE_MIN * 1e3, ", ".join("%s %.0f ms" % kv for kv in sorted(floor.items(), key=lambda kv: kv[1])) or "none"))
    if ctx.hold_T:
        out["holding_strength_Nm"], out["holding_at_start_Nm"] = dict(ctx.hold_T), dict(ctx.hold_at_start)
        out["hold_still_left_Nm"] = {j: E.norm(v) for j, v in ctx.tone_left.items()}
        log("   THE POSTURE HELD BY MUSCLE (4.13, rung 2): each group's holding torque at the start, N m (of its bound): "
            + ", ".join("%s %.1f (%.2f)" % (n, x, ctx.hold_T[n] / ctx.hold_bd[n]) for n, x in sorted(ctx.hold_at_start.items(), key=lambda kv: -kv[1]))
            + ";   left as a constant torque, with no measured muscle that way (N m): " + ", ".join("%s %.1f" % kv for kv in sorted(out["hold_still_left_Nm"].items(), key=lambda kv: -kv[1]) if kv[1] >= 0.5))
    log("   the plan: channel, strength of bound, onset and switch-off relative to contact ms, rise ms, level beforehand")
    for n in CH:
        x = pl[n]
        if used(n) or x[4] > 0.05:
            log("      %-14s %4.0f of %4.0f  %5.0f %5.0f  %4.0f  %.2f" % (n, x[0], ctx.bd[n], rel(x[1]), rel(x[3]), x[2] * 1e3, x[4]))
    # for the eye: the motion at a few instants
    log("   for the eye (times before contact): pelvis, trunk turned toward the pitcher (deg from facing the plate); lead knee; hands' and sweet spot's speed; share of weight on the front foot")
    for tt in (T0 - tx, -0.150, -0.100, -0.050, -0.020, 0.0, 0.030):
        q = at(tx + tt)
        o = obs[q][1]
        tot = fz[q][0] + fzr[q][0]
        log("      %5.0f ms: pelvis %4.0f, trunk %4.0f; lead knee %3.0f; hands %4.1f, sweet spot %4.1f m/s; front foot %.2f of the %.2f x body weight on the ground"
            % (tt * 1e3, o["pelvis_rot"], o["trunk_rot"], o["lead_knee_flex"], hs[q][0], sw[q][0], fz[q][0], tot))
    return out


def failed_differences(ctx, u, log=print):
    """how many of the plan's one-number differences (each free number moved 2 % of its range, as the search moves it)
    fail to run. A failed run is a refused step, so a count above zero means the search could not see its slope there:
    the check that session 9's first runs lacked (body-motion 4.11)."""
    bad = 0
    for i in ctx.free:
        un = list(u)
        un[i] = u[i] + (0.02 if u[i] + 0.02 <= 1.0 else -0.02)
        try:
            ctx.run(un)
        except E.SolveError:
            bad += 1
    log("   of the plan's %d one-number differences, %d fail to run%s" % (len(ctx.free), bad, "" if bad == 0 else "  (THE SEARCH COULD NOT SEE ITS SLOPE HERE)"))
    return bad


def verify(ctx, u, log=print):
    """the same plan at 0.5, 0.25 and 0.125 ms: what the result's numbers are worth."""
    out = {}
    for h in (H_STEP, 2.5e-4, VERIFY_H):
        try:
            recs = ctx.run(u, every=int(round(1e-3 / h)), h=h)
        except E.SolveError:
            log("   at a %.3f ms step: THE RUN FAILS" % (h * 1e3))
            out[h] = None
            continue
        tx, p, v, i, came = ctx.contact(recs)
        d = E.sub(p, ctx.Pc)
        out[h] = ((tx - T0) * 1e3, E.norm(v), E.dot(d, ctx.ev) * 100, E.dot(d, ctx.el) * 100, came)
        log("   at a %.3f ms step: contact %.1f ms after the start, the sweet spot at %.2f m/s, the miss %+.1f cm up and %+.1f cm along the bat%s" % ((h * 1e3,) + out[h][:4] + ("" if came else "  (never crossed the ball's plane)",)))
    return {"%.3f ms" % (h * 1e3): v for h, v in out.items()}


def cost_of(r):
    return float("inf") if r is None else sum(x * x for x in r)


CMA_LAMBDA, CMA_SIGMA, CMA_OUT = 24, 0.03, 1e4          # plans a generation; the first spread, of each number's range; the charge for leaving the box [E]


def cma(ctx, u0, gens, log, pool, seed=1, lam=CMA_LAMBDA, sigma0=CMA_SIGMA):
    """The practice search without slopes (body-motion 4.12): the separable form of CMA-ES (Ros & Hansen 2008) on the free
    numbers of the plan, in the unit box. Each generation `lam` plans are drawn about a centre with one spread per number,
    all are run, they are RANKED by score, the centre moves to the weighted mean of the better half and each number's
    spread follows how the better plans were spread. Only the order of the scores is used. A plan outside the box is run
    at the box's edge and charged CMA_OUT x its squared distance outside; a failed run ranks last. Deterministic (seed).
    Returns (the best plan ever run, its score, history [(generation, best ever, this generation's best, median, the
    overall step, failed runs)], the last centre, the spreads at the end)."""
    free = ctx.free
    n = len(free)
    rng = random.Random(seed)
    mu = lam // 2
    w = [math.log(mu + 0.5) - math.log(i + 1) for i in range(mu)]
    sw = sum(w)
    w = [x / sw for x in w]
    mueff = 1.0 / sum(x * x for x in w)
    cc = (4.0 + mueff / n) / (n + 4.0 + 2.0 * mueff / n)
    cs = (mueff + 2.0) / (n + mueff + 5.0)
    c1 = 2.0 / ((n + 1.3) ** 2 + mueff)
    cmu = min(1.0 - c1, 2.0 * (mueff - 2.0 + 1.0 / mueff) / ((n + 2.0) ** 2 + mueff))
    c1 *= (n + 2.0) / 3.0                                   # the separable form learns one number each, so faster
    cmu = min(1.0 - c1, cmu * (n + 2.0) / 3.0)
    damps = 1.0 + 2.0 * max(0.0, math.sqrt((mueff - 1.0) / (n + 1.0)) - 1.0) + cs
    chi = math.sqrt(n) * (1.0 - 1.0 / (4.0 * n) + 1.0 / (21.0 * n * n))
    m = [u0[i] for i in free]
    C = [1.0] * n
    ps, pc = [0.0] * n, [0.0] * n
    sigma = sigma0
    r0 = ctx.residuals(u0)
    best_u, best = list(u0), cost_of(r0)
    hist = [(0, best, best, best, sigma, 0)]
    for g in range(1, gens + 1):
        zs = [[rng.gauss(0.0, 1.0) for _ in range(n)] for _ in range(lam)]
        ys = [[math.sqrt(C[k]) * z[k] for k in range(n)] for z in zs]
        us, pens = [], []
        for y in ys:
            u = list(u0)
            pen = 0.0
            for k, i in enumerate(free):
                x = m[k] + sigma * y[k]
                xc = min(1.0, max(0.0, x))
                pen += (x - xc) ** 2
                u[i] = xc
            us.append(u); pens.append(CMA_OUT * pen)
        rs = pool.map(_res, us, chunksize=max(1, lam // WORKERS))
        raw = [cost_of(r) for r in rs]
        fs = [c + p_ for c, p_ in zip(raw, pens)]
        failed = sum(1 for c in raw if c == float("inf"))
        order = sorted(range(lam), key=lambda k: fs[k])
        kb = min(range(lam), key=lambda k: raw[k])
        if raw[kb] < best:
            best, best_u = raw[kb], us[kb]
        yw = [sum(w[j] * ys[order[j]][k] for j in range(mu)) for k in range(n)]
        zw = [sum(w[j] * zs[order[j]][k] for j in range(mu)) for k in range(n)]
        m = [m[k] + sigma * yw[k] for k in range(n)]
        ps = [(1.0 - cs) * ps[k] + math.sqrt(cs * (2.0 - cs) * mueff) * zw[k] for k in range(n)]
        pn = math.sqrt(sum(x * x for x in ps))
        hsig = 1.0 if pn / math.sqrt(1.0 - (1.0 - cs) ** (2 * g)) / chi < 1.4 + 2.0 / (n + 1.0) else 0.0
        pc = [(1.0 - cc) * pc[k] + hsig * math.sqrt(cc * (2.0 - cc) * mueff) * yw[k] for k in range(n)]
        for k in range(n):
            C[k] = ((1.0 - c1 - cmu) * C[k] + c1 * (pc[k] * pc[k] + (1.0 - hsig) * cc * (2.0 - cc) * C[k])
                    + cmu * sum(w[j] * ys[order[j]][k] ** 2 for j in range(mu)))
        sigma *= math.exp((cs / damps) * (pn / chi - 1.0))
        srt = sorted(c for c in raw)
        hist.append((g, best, raw[kb], srt[lam // 2], sigma, failed))
        if g % 10 == 0 or g == gens:
            log("      generation %3d: best ever %.2f; this generation's best %.2f, median %.2f; overall step %.4f; failed runs %d" % (g, best, raw[kb], srt[lam // 2], sigma, failed))
    mfull = list(u0)
    for k, i in enumerate(free):
        mfull[i] = min(1.0, max(0.0, m[k]))
    return best_u, best, hist, mfull, [sigma * math.sqrt(c) for c in C]


def search_checks(o, u0, u, log=print, pool=None):
    """the search itself, checked (body-motion 4.12, rule 6), at a result u found from u0 under options o:
    (1) the result at three steps; (2) the score along the line from u0 to u and one step beyond it, at 0.25 and at
    0.125 ms; (3) slopes forward and backward for the eight numbers moved furthest, at two differences. Printed and
    returned; nothing here feeds the search."""
    out = {}
    ctxs = {h: Ctx(**dict(o, h=h)) for h in (5e-4, 2.5e-4, 1.25e-4)}
    out["steps"] = {}
    for h, c in ctxs.items():
        r = c.residuals(u)
        out["steps"]["%.3f" % (h * 1e3)] = None if r is None else (cost_of(r), r[0] * MISS_UNIT * 100, r[1] * MISS_UNIT * 100, SPEED_ASK - r[5] ** 2 / SPEED_W)
    log("   CHECK OF THE SEARCH 1, the result's score at 0.5, 0.25, 0.125 ms (with its miss up and along the bat, cm, and its speed along the line): "
        + "; ".join("%s ms: %s" % (k, "FAILS" if v is None else "%.1f (%+.1f, %+.1f, %.1f m/s)" % v) for k, v in out["steps"].items()))
    out["line"] = {}
    alphas = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1]
    for h in (2.5e-4, 1.25e-4):
        vals = []
        for a in alphas:
            ua = [min(1.0, max(0.0, x0 + a * (x1 - x0))) for x0, x1 in zip(u0, u)]
            vals.append(cost_of(ctxs[h].residuals(ua)))
        out["line"]["%.3f" % (h * 1e3)] = vals
        log("   CHECK OF THE SEARCH 2, the score along the line from the start (0) to the result (1) and beyond (1.1), at %.3f ms: %s" % (h * 1e3, " ".join("%.0f" % v for v in vals)))
    c = ctxs[2.5e-4]
    moved = sorted(c.free, key=lambda i: -abs(u[i] - u0[i]))[:8]
    f0 = cost_of(c.residuals(u))
    out["slopes"] = []
    for i in moved:
        row = []
        for fd in (0.02, 0.005):
            for sgn in (1.0, -1.0):
                un = list(u); un[i] = min(1.0, max(0.0, u[i] + sgn * fd))
                hh = un[i] - u[i]
                row.append(float("nan") if hh == 0 else (cost_of(c.residuals(un)) - f0) / hh)
        out["slopes"].append((c.box[i][0], c.box[i][1], u[i] - u0[i]) + tuple(row))
    log("   CHECK OF THE SEARCH 3, slopes of the score at the result (forward, backward at 0.02; forward, backward at 0.005), the eight numbers moved furthest:")
    for row in out["slopes"]:
        log("      %-14s %-5s moved %+.3f:  %9.0f %9.0f | %9.0f %9.0f" % row)
    return out


def calf_plan(plan, bd):
    """start K (4.12): a plan with the rear calf at half its strength from 100 ms after the start, rising in 80 ms [E]."""
    out = {n: list(v) for n, v in plan.items()}
    out["ankle_pf_R"] = [0.5 * bd["ankle_pf_R"], T0 + 0.100, 0.080, 0.100, 0.0]
    return out


def main():
    P.build()
    a = sys.argv[1:]
    arg = lambda k, d: a[a.index(k) + 1] if k in a else d
    main_file = os.path.join(HERE, "practice3d_main.json")
    body = {"grip": arg("--grip", None)} if "--grip" in a else {}      # the body a new search runs on; kept in the result's options
    global FALLBACK
    FALLBACK = "--fallback" in a
    if "--hold" in a:
        body["hold"] = arg("--hold", None)
    if "--view" in a:
        body["t_view"] = float(arg("--view", T_VIEW))
    if "--feet" in a:
        body["feet"] = arg("--feet", None)
    if "--wrist" in a:
        body["wrist"] = arg("--wrist", None)
    if "--clear" in a:
        body["clear"] = True
    if "--mu" in a:
        body["mu"] = float(arg("--mu", B.MU))
    if "--show" in a:
        d = json.load(open(arg("--show", main_file)))
        ctx = Ctx(**dict(d.get("options", {}), **body))
        out = report(ctx, ctx.u_of(d["plan"]))
        if "--update" in a:
            # a kept result re-read (no search): what the report now adds or reads more finely is written into its file
            for k in ("work_J", "hold_still_J", "damping_J", "damping_peak_Nm", "stops", "ligaments_over_50", "wrist_oval", "near_full_strength",
                      "near_full_strength_activation", "rises_near_floor_ms"):
                if k in out:
                    d[k] = out[k]
            json.dump(d, open(arg("--show", main_file), "w"), indent=1)
        return
    iters = int(arg("--iters", 30))
    if "--main" in a:
        ctx = Ctx()
        print("the practice search of body-motion 4.10: the average batter (1.84 m, 92 kg), %d free numbers, %d iterations a start" % (len(ctx.free), iters))
        print("the joints' stops (deg): " + "; ".join("%s %.0f to %.0f" % ((k,) + v) for k, v in ctx.limits.items()))
        print("the ball is met %.2f m toward the plate and %.2f m toward the pitcher from the hips at the start, %.2f m up" % (ctx.Pc[0] - ctx.m.sys.pose(ctx.st, ctx.m.b["pelvis"])[0][0], MEET_AHEAD, ctx.Pc[2]))
        starts = (("A: session 7's fit to [4]", json.load(open(os.path.join(HERE, "fit3d_d34_all.json")))["plan"]),
                  ("B: the swing asked only for speed (body-motion 4.9)", json.load(open(os.path.join(P.OUT, "fit3d_all_strongest_on_time.json")))["plan"]),
                  ("C: written by hand, 0.6 of every forward muscle", hand_plan(ctx, 0.6)),
                  ("D: written by hand, 0.3 of every muscle, timings at random", hand_plan(ctx, 0.3, seed=1)))
        only = arg("--starts", "ABCD")
        best = None
        for label, plan in starts:
            if label[0] not in only:
                continue
            print("\n== start %s" % label)
            u, cost, hist = search(ctx, plan, iters)
            out = report(ctx, u)
            out.update(cost=cost, start=label, history=hist, step_check=verify(ctx, u))
            json.dump(out, open(os.path.join(P.OUT, "practice3d_start_%s.json" % label[0]), "w"), indent=1)
            if best is None or cost < best["cost"]:
                best = out
        more = int(arg("--more", 20))
        print("\n== the best (start %s, cost %.3f) continued %d iterations" % (best["start"], best["cost"], more))
        u, cost, hist = search(ctx, best["plan"], more)
        out = report(ctx, u)
        out.update(cost=cost, start=best["start"] + ", continued", history=hist, step_check=verify(ctx, u))
        json.dump(out, open(main_file, "w"), indent=1)
        return
    if "--cma" in a:
        # the search of body-motion 4.12: no slopes, every run at 0.25 ms, on the cleared start with the wrists' ellipses
        o = dict(grip="hands", feet="cleats", hold="let_go", wrist="oval", clear=True, h=2.5e-4)
        o.update(body)
        o["t_view"] = float(arg("--view", T_VIEW))
        label = arg("--cma", "S")
        g1, g2 = int(arg("--reach", 100)), int(arg("--gens", 250))
        seed = int(arg("--seed", 1))
        ctx = Ctx(contact="nearest", **o)
        reach = Ctx(stage="reach", **o)
        s9 = json.load(open(os.path.join(HERE, "practice3d_s9_cleats_S.json")))["plan"]
        plan = {"S": s9, "K": calf_plan(s9, ctx.bd), "C": hand_plan(ctx, 0.6), "D": hand_plan(ctx, 0.3, seed=1)}[label] if label in "SKCD" else json.load(open(label))["plan"]
        plan = {n: [min(v[0], ctx.bd[n])] + list(v[1:]) for n, v in plan.items()}
        u0 = ctx.u_of(plan)
        print("== the search of 4.12 from start %s, watching %.2f s (the arms may fire %.0f ms before contact); %s; %d generations of the reach stage, then %d of the whole aim, %d plans each, every run at %.3f ms; seed %d"
              % (label, o["t_view"], ctx.t_avail * 1e3, o, g1, g2, CMA_LAMBDA, o["h"] * 1e3, seed), flush=True)
        r0 = ctx.residuals(u0)
        print("   the starting plan's score on this body: %s; by term: %s" % ("THE RUN FAILS" if r0 is None else "%.1f" % cost_of(r0), "" if r0 is None else "; ".join("%s %.1f" % kv for kv in terms(ctx, r0).items())), flush=True)
        global _CTX
        t = time.time()
        _CTX = reach
        with multiprocessing.get_context("fork").Pool(WORKERS) as pool:
            print("-- the reach stage", flush=True)
            u1, c1_, h1, m1, sp1 = cma(reach, u0, g1, lambda s_: print(s_, flush=True), pool, seed)
        rr = reach.residuals(u1)
        print("   nearest the ball: %.1f cm up, %.1f cm along the bat, %.1f cm along the ball's line; %.0f s" % (rr[0] * MISS_UNIT * 100, rr[1] * MISS_UNIT * 100, rr[6] * MISS_UNIT * 100, time.time() - t), flush=True)
        _CTX = ctx
        with multiprocessing.get_context("fork").Pool(WORKERS) as pool:
            print("-- the whole aim", flush=True)
            u, cost, hist, mfull, spread = cma(ctx, u1, g2, lambda s_: print(s_, flush=True), pool, seed + 1)
        print("   %d generations, %.0f s; best score %.2f (the last centre's %.2f); it fell %.2f over the last 50 generations; the spreads at the end: largest %.4f, median %.4f of a number's range; failed runs in all %d"
              % (g2, time.time() - t, cost, cost_of(ctx.residuals(mfull)), hist[max(0, len(hist) - 51)][1] - hist[-1][1], max(spread), sorted(spread)[len(spread) // 2], sum(h_[5] for h_ in hist)), flush=True)
        out = report(ctx, u)
        out.update(cost=cost, start=label + ", the search of 4.12", history=hist, reach_history=h1, step_check=verify(ctx, u), failed_differences=failed_differences(ctx, u),
                   search_checks=search_checks(dict(o, contact="nearest"), u0, u), start_plan=plan, spread_end=(max(spread), sorted(spread)[len(spread) // 2]))
        os.makedirs(os.path.join(P.OUT, "s10"), exist_ok=True)
        json.dump(out, open(os.path.join(P.OUT, "s10", "cma_%s_view%03d%s.json" % (label if label in "SKCD" else "F", int(round(o["t_view"] * 1e3)), arg("--tag", ""))), "w"), indent=1)
        return
    if "--staged" in a:
        # a staged search (4.10): from each start, first the reach stage, then the whole aim from where that ended
        names = arg("--staged", "main").split(",")
        n1, n2 = int(arg("--reach", 15)), iters
        for name in names:
            o = dict(body) if name == "main" else dict(VARY[name], **body)
            ctx = Ctx(contact="nearest", **o)
            reach = Ctx(stage="reach", **o)
            print("\n== staged, %s: %s; from each start %d iterations of the reach stage, then %d of the whole aim" % (name, o, n1, n2))
            starts = [("A", json.load(open(os.path.join(HERE, "fit3d_d34_all.json")))["plan"]),
                      ("C", hand_plan(ctx, 0.6)), ("D", hand_plan(ctx, 0.3, seed=1)),
                      ("S", json.load(open(os.path.join(HERE, "practice3d_main.json")))["plan"])]     # S: session 8's practised swing (found from A, on the weld)
            if "B" in arg("--starts", "ABCD"):
                starts.append(("B", json.load(open(os.path.join(P.OUT, "fit3d_all_strongest_on_time.json")))["plan"]))
            if "--from-file" in a:                               # a start from a saved result, labelled F
                starts.append(("F", json.load(open(arg("--from-file", "")))["plan"]))
            best = None
            tag = arg("--tag", "")
            for label, plan in starts:
                if label not in arg("--starts", "ABCD"):
                    continue
                plan = {n: [min(v[0], ctx.bd[n])] + list(v[1:]) for n, v in plan.items()}
                print("\n-- start %s, the reach stage" % label)
                u, cost, hist = search(reach, plan, n1)
                rr = reach.residuals(u)
                print("   nearest the ball: %.1f cm up, %.1f cm along the bat, %.1f cm along the ball's line" % (rr[0] * MISS_UNIT * 100, rr[1] * MISS_UNIT * 100, rr[6] * MISS_UNIT * 100))
                print("-- start %s, the whole aim" % label)
                u, cost, hist2 = search(ctx, reach.plan(u), n2)
                out = report(ctx, u)
                out.update(cost=cost, start=label + ", staged", history=hist2, reach_history=hist, step_check=verify(ctx, u), failed_differences=failed_differences(ctx, u))
                json.dump(out, open(os.path.join(P.OUT, "practice3d_staged_%s%s_%s.json" % (name, tag, label)), "w"), indent=1)
                if best is None or cost < best["cost"]:
                    best = out
            json.dump(best, open(os.path.join(P.OUT, "practice3d_staged_%s%s.json" % (name, tag)), "w"), indent=1)
            print("\nthe best of the staged starts for %s: start %s, cost %.3f" % (name, best["start"], best["cost"]))
        return
    if "--continue" in a:
        # the search carried on from a saved result (contact read at the nearest approach, as in the staged search)
        d = json.load(open(arg("--continue", main_file)))
        o = {k: v for k, v in d.get("options", {}).items() if k not in ("contact", "stage")}
        o.update(body)
        ctx = Ctx(contact="nearest", **o)
        print("== continued %d iterations from %s (start %s, cost %.3f)" % (iters, os.path.basename(arg("--continue", main_file)), d.get("start"), d.get("cost", float("nan"))))
        u, cost, hist = search(ctx, d["plan"], iters)
        out = report(ctx, u)
        out.update(cost=cost, start=str(d.get("start")) + ", continued", history=hist, step_check=verify(ctx, u), failed_differences=failed_differences(ctx, u))
        json.dump(out, open(arg("--out", main_file), "w"), indent=1)
        return
    if "--vary" in a:
        base = json.load(open(arg("--from", main_file)))
        for name in arg("--vary", "").split(","):
            o = dict(VARY[name], **body)
            print("\n== %s: %s; %d iterations from the main result" % (name, o, iters))
            ctx = Ctx(contact="nearest", **o)
            plan = {n: [min(v[0], ctx.bd[n])] + list(v[1:]) for n, v in base["plan"].items()}
            if name in ("strongest",):                       # his strengths start where the average batter used the same share of his own
                plan = {n: [v[0] * ctx.bd[n] / base["bounds"][n] if base["bounds"][n] > 0 else 0.0] + list(v[1:]) for n, v in base["plan"].items()}
            print("   the main result's plan on this body, before any search:")
            report(ctx, ctx.u_of(plan), log=lambda s_: print(s_) if "CHECK the sweet" in s_ or "the ball:" in s_ or "FROM THE START" in s_ else None)
            u, cost, hist = search(ctx, plan, iters)
            out = report(ctx, u)
            out.update(cost=cost, start="main result", history=hist, step_check=verify(ctx, u))
            json.dump(out, open(os.path.join(P.OUT, "practice3d_%s.json" % name), "w"), indent=1)
        return
    print(__doc__)


if __name__ == "__main__":
    main()
