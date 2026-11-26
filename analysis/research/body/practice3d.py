"""The batter's swing learned by practice on his own body (docs/body-motion.md 4.10; PLAN D32, D36, D37).

    python3 analysis/research/body/practice3d.py --main            four starts, the best continued (job queue)
    python3 analysis/research/body/practice3d.py --vary a,b,...    one thing changed at a time, from the main result
    python3 analysis/research/body/practice3d.py --show <json>     the checks for a saved plan

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
T0, T_END, H_STEP, EVERY = -0.250, 0.100, 5e-4, 10     # T0 is only the placed start's label on [4]'s clock. The step is 0.5 ms:
#   with the joints' stiff stops a violent swing does not integrate at 1 ms (body-motion 4.10); every result is rerun at 0.125 ms
VERIFY_H = 1.25e-4
WORKERS = 4                                            # processes for the search's finite differences (inside one queued job)
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


class Ctx:
    """one body with its ranges, its start and the pitch, reused for every swing of a search."""

    def __init__(self, **o):
        self.o = o
        H, M = o.get("H", 1.84), o.get("M", 92.0)
        self.m = m = B.Model(H, M, ranges=())
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
        st = Dr.add_drives(m, st, {n: (0.0, 0.0, 0.05, 0.1, 0.0) for n in CH})
        Dr.BOUND_SCALE.clear()
        self.limits = B.add_ranges(m, st, k_big=o.get("k_big"), wrists=o.get("wrists", "cocked"), rom=o.get("rom"))
        self.tone, self.tone_rest = Dr.solve_tone(m, st, T0, aux=list(self.aux0))
        Dr.add_damping(m, st)
        self.st = st
        self.BW = M * E.G
        self.order = P.tree_order(m)
        self.box = []
        for n in CH:
            self.box += [(n, "T", 0.0, self.bd[n]), (n, "on", T0, 0.050), (n, "rise", Dr.RISE_MIN, 0.250),
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
        self.t_avail = BALL_FLIGHT - T_VIEW - T_LATENCY
        self.z0 = B.com(m, st)[0][2]
        self.evals = 0

    plan = F.Ctx.plan
    u_of = F.Ctx.u_of
    prepare = F.Ctx.prepare

    def run(self, u, every=EVERY, t_end=T_END, h=H_STEP):
        st, aux = self.prepare(u)
        self.evals += 1
        n = int(round((t_end - T0) / h))
        return P.run_raw(self.m.sys, st, T0, n, h, every, aux, self.order)

    def contact(self, recs):
        """(t_x, sweet spot position, velocity, index of the record before, came round?) where the sweet spot crosses
        the plane square to the ball's line at the meeting point, by cubic interpolation between records."""
        m = self.m
        prev = None
        for i, rec in enumerate(recs):
            p, v = m.bat_point(rec[1], C.BAT_SS)
            a = -E.dot(E.sub(p, self.Pc), self.nb)
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
        return recs[-1][0], prev[1], prev[2], len(recs) - 2, False

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
        for t, state, con, act, stop, sec in recs:
            if t > tx + 1e-9:
                break
            k += 1
            for n, a in zip(CH, act):
                acc[n] += (a[2] * pl[n][0] / self.bd[n]) ** 2 if self.bd[n] > 0 else 0.0
        return {n: v / max(1, k) for n, v in acc.items()}

    def residuals(self, u):
        try:
            recs = self.run(u)
        except E.SolveError:
            return None
        pl = self.plan(u)
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
             0.0 if came else E.dot(d, self.nb) / MISS_UNIT]          # never came round: how far short along the line
        eff = self.effort(recs, tx, pl)
        r += [math.sqrt(self.effort_w * eff[n] / len(CH)) for n in CH]
        xo, least, zr = self.balance(recs, tx)
        r += [xo / XCOM_UNIT, max(0.0, JUMP_MIN - least) / JUMP_UNIT, max(0.0, SINK_MIN - zr) * self.z0 / SINK_UNIT]
        on = [pl[n][1] for n in ARMS if pl[n][0] > 0.01 * self.bd[n]]
        r.append(max(0.0, (tx - self.t_avail) - min(on)) / CLOCK_UNIT if on else 0.0)
        if self.strain_w > 0.0:
            names = [s_[0] for s_ in self.m.sys.stops]
            for j, nm in enumerate(names):
                pk = max(abs(rec[4][j][0]) for rec in recs if rec[0] <= tx + AFTER + 1e-9)
                r.append(0.0 if nm == "shoulder_L" else math.sqrt(self.strain_w) * max(0.0, pk - STRAIN_FREE) / STRAIN_UNIT)
        return None if any(x != x for x in r) else r


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
        hist.append(cost)
        log("      iteration %2d: cost %.3f (miss %.1f, %.1f cm; speed along the line %.2f m/s), lambda %.0e"
            % (it + 1, cost, r[0] * MISS_UNIT * 100, r[1] * MISS_UNIT * 100, SPEED_ASK - r[5] ** 2 / SPEED_W, lam))
        if not improved or gain < 1e-6 * cost:
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
    recs = ctx.run(u, every=int(round(1e-3 / H_STEP)))
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
    log("   FROM THE START PLACED ON [4]'s STATE (an input, flagged): contact %.0f ms after the start%s" % ((tx - T0) * 1e3, "" if came else "  (THE BAT NEVER REACHED THE BALL; figures are at the run's end)"))
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
    # balance, effort, the clock
    xo, least, zr = ctx.balance(recs, tx)
    out["balance"] = (xo * 100, least, zr)
    log("   the aim's balance: the centre of mass's heading %.1f cm outside the feet's area at worst (0 = inside); the feet never carry under %.2f of body weight; the centre of mass ends at %.2f of its starting height" % out["balance"])
    eff = ctx.effort(recs, tx, pl)
    out["effort"] = sum(eff.values()) / len(CH)
    out["effort_by_channel"] = eff
    on = [pl[n][1] for n in ARMS if pl[n][0] > 0.01 * ctx.bd[n]]
    out["clock_ms"] = (tx - min(on)) * 1e3 if on else None
    log("   effort %.3f (0 = nothing, 1 = everything flat out all swing); the first arm drive of any size rises %.0f ms before contact (the clock allows %.0f)" % (out["effort"], out["clock_ms"] or 0.0, ctx.t_avail * 1e3))
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
        tau = max(abs(rec[4][j][0]) for rec in recs[:ic + 51])
        over = max(max(rec[4][j][1] / B.D2R - hi, lo - rec[4][j][1] / B.D2R) for rec in recs[:ic + 51])
        before_c = max(abs(rec[4][j][0]) for rec in recs[:ic + 1])
        out["stops"][nm] = (tau, max(0.0, over), before_c)
    lean = {k_: x for k_, x in out["stops"].items() if x[0] > 1.0}
    log("   the joints' stops that carried load to 50 ms after contact (N m at most; deg past the end; N m at most up to contact): "
        + ("; ".join("%s %.0f, %.1f, %.0f" % ((k_,) + x) for k_, x in sorted(lean.items(), key=lambda kv: -kv[1][0])) or "none"))
    out["ligaments_over_50"] = sorted(k_ for k_, x in out["stops"].items() if k_ != "shoulder_L" and x[0] > STRAIN_FREE)
    log("   LEANING ON A LIGAMENT (a stop other than the lead arm against the chest over %.0f N m): %s" % (STRAIN_FREE, ", ".join(out["ligaments_over_50"]) or "none"))
    out["unheld"] = {}
    for key, rng_ in (("pron_L", 161.9), ("pron_R", 161.9), ("sh_rot_L", 173.0), ("sh_rot_R", 173.0), ("ankle_L", 67.3), ("ankle_R", 67.3)):
        vv = unwrap([a[key] for a in ang])
        out["unheld"][key] = (max(vv) - min(vv), rng_)
    log("   not held, travel to 50 ms after contact against the real range, deg: " + "; ".join("%s %.0f of %.0f%s" % (k_, x[0], x[1], "  MORE THAN A BODY HAS" if x[0] > x[1] else "") for k_, x in out["unheld"].items()))
    onb = []
    for q, (nm, f, lo, hi) in enumerate(ctx.box):
        if q in ctx.free and f == "T" and u[q] >= 1.0 - 1e-6:
            onb.append(nm)
    out["at_full_strength"] = onb
    out["rises_at_floor"] = sum(1 for q, (nm, f, lo, hi) in enumerate(ctx.box) if f == "rise" and u[q] <= 1e-6 and used(nm))
    log("   at full strength (%d of %d): %s;  %d used muscles rise at the 50 ms floor" % (len(onb), len(CH), ", ".join(onb) or "none", out["rises_at_floor"]))
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


def verify(ctx, u, log=print):
    """the same plan at a quarter of the step: what the result's numbers are worth."""
    out = {}
    for h in (H_STEP, VERIFY_H):
        try:
            recs = ctx.run(u, every=int(round(1e-3 / h)), h=h)
        except E.SolveError:
            log("   at a %.3f ms step: THE RUN FAILS" % (h * 1e3))
            out[h] = None
            continue
        tx, p, v, i, came = ctx.contact(recs)
        d = E.sub(p, ctx.Pc)
        out[h] = ((tx - T0) * 1e3, E.norm(v), E.dot(d, ctx.ev) * 100)
        log("   at a %.3f ms step: contact %.1f ms after the start, the sweet spot at %.2f m/s, the miss %+.1f cm up" % ((h * 1e3,) + out[h]))
    return {"%.3f ms" % (h * 1e3): v for h, v in out.items()}


def main():
    P.build()
    a = sys.argv[1:]
    arg = lambda k, d: a[a.index(k) + 1] if k in a else d
    main_file = os.path.join(HERE, "practice3d_main.json")
    if "--show" in a:
        d = json.load(open(arg("--show", main_file)))
        ctx = Ctx(**d.get("options", {}))
        report(ctx, ctx.u_of(d["plan"]))
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
    if "--vary" in a:
        base = json.load(open(arg("--from", main_file)))
        for name in arg("--vary", "").split(","):
            o = dict(VARY[name])
            print("\n== %s: %s; %d iterations from the main result" % (name, o, iters))
            ctx = Ctx(**o)
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
