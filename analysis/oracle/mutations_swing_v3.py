"""Run the v3 oracle tests (test_swing_v3.py) under deliberate mistakes and list the tests that catch
each: a wrong frame, a wrong sign, a wrong anchor instant, a wrong s_T, a wrong mirror, and the swing
family's (§6.7): effort in the travel instead of the time, the travel without the batter's own time,
no fade, the template at one time for every aim, the re-plan's travel held, the step-1 anchor back;
and the stance refit's (PLAN D28): today's stance spot, the rate shape on the azimuth rate, the hands'
travel uncapped, the κ-scaled elevation, the offset finished at contact.
Run from the repository root: python3 analysis/oracle/mutations_swing_v3.py"""
import sys, math, unittest, io
sys.path.insert(0, 'analysis/oracle')
import swing_v3 as s
import test_swing_v3 as t

orig = {}
_MISSING = object()
def patch(obj, name, val):
    # the raw attribute (a class's staticmethod stays a staticmethod when restored; an inherited
    # method is removed again rather than copied down)
    raw = obj.__dict__.get(name, _MISSING) if isinstance(obj, type) else getattr(obj, name)
    orig.setdefault((obj, name), raw); setattr(obj, name, val)
def restore():
    for (obj, name), v in orig.items():
        if v is _MISSING: delattr(obj, name)
        else: setattr(obj, name, v)
    orig.clear()

def m_reading2():
    f = orig.get((s.Body, 'upper_arm_dir')) or s.Body.upper_arm_dir
    patch(s.Body, 'upper_arm_dir', lambda self, e, a, reading=1: f(self, e, a, 2 if reading == 1 else 1))
def m_frame_swap():
    init = s.Body.__init__
    def new(self, *a, **k):
        init(self, *a, **k); self.xT, self.yT = self.yT, s.sc(self.xT, -1.0)
    patch(s.Body, '__init__', new)
def m_azimuth_sign():
    patch(s, 'bat_dir', lambda A, E: (math.cos(E) * math.cos(A), -math.cos(E) * math.sin(A), math.sin(E)))
def m_anchor_80():
    patch(s, 'T4_93', -0.080)
def m_anchor_unscaled():
    b = s.SwingStep1._build
    def new(self):
        self.tau93 = -0.093; b(self)
    patch(s.SwingStep1, '_build', new)
def m_sT_150():
    patch(s, 'T4', 0.150)
def m_sT_none():
    init = s.Swing.__init__
    def new(self, *a, **k):
        init(self, *a, **k)
    b = s.SwingStep1._build
    def nb(self):
        self.sT = 1.0; self.tau93 = s.T4_93; b(self)
    patch(s.SwingStep1, '_build', nb)
def m_omega_mirror():
    pose = s.Swing.pose
    def new(self, t, tau_b=None):
        p = pose(self, t, tau_b)
        if self.side == -1: p['omega'] = (-p['omega'][0], -p['omega'][1], p['omega'][2])
        return p
    patch(s.Swing, 'pose', new)

# The step-1 review's mutations (sim/target/review-probes/my_mutations*.py), the elevation join's
# ported to the blend that replaced δ_E.
def m_join_chain():
    f = s.Template._baseA
    def g(self, u, k=0):
        if k == 2: return self.A1s * self.T * s.RATE4_RATIO(1e3 * s.T4 * u, 1)
        return f(self, u, k)
    patch(s.Template, '_baseA', g)
def m_blend_rate():
    f = s.SwingStep1._seg2E
    def g(self, u, k=0):
        if k == 1 and u > self.uw:
            L = -self.uw; x = (u - self.uw) / L; w = s.smoothstep(x)
            return (1 - w) * self._baseE(u, 1) + w * self._v2_elevation(u)[1]   # the w' term dropped
        return f(self, u, k)
    patch(s.SwingStep1, '_seg2E', g)
def _dA_variant(at93, acc_less_shape=True):
    b = s.Template._build_v3
    def g(self):
        b(self)
        T, u93 = self.T, self.u93
        A0, A1, A2 = self.A_c
        bA = self._baseA
        v93 = (A0 - s.SWEEP_93 * s.DEG - bA(u93)) if at93 else 0.0
        self.dA = s.poly_fit([(u93, 0, v93), (u93, 1, 0.0), (u93, 2, 0.0), (0.0, 0, A0 - bA(0.0)), (0.0, 1, A1 * T - bA(0.0, 1)),
                              (0.0, 2, A2 * T * T - (bA(0.0, 2) if acc_less_shape else 0.0))])
        self.c1A = s.poly_fit([(-1.0, 0, self.A_L), (-1.0, 1, 0.0), (-1.0, 2, 0.0)] + [(u93, k, self._seg2A(u93, k)) for k in range(3)])
    patch(s.Template, '_build_v3', g)
def m_join_Aacc():
    _dA_variant(False, acc_less_shape=False)
def m_uw20():
    patch(s, 'T4_W', -20.0)
def m_brake_ruling():
    def A_max(self, tau_b=None):
        end = s.TRACK_AFTER_S if tau_b is None else min(s.TRACK_AFTER_S, tau_b + s.BRAKE_S)
        return self.state_rh(end)[0] / s.DEG
    patch(s.Swing, 'A_max', A_max)
def m_brake_rate():
    sg = s.Swing.sigma
    def g(self, tau, tau_b=None):
        x, r = sg(self, tau, tau_b)
        return x, (1.0 if r > 0 else 0.0)
    patch(s.Swing, 'sigma', g)
def m_brake_travel():
    def g(self, tau, tau_b=None):
        if tau_b is None or tau <= tau_b: return tau, 1.0
        s_ = min(tau - tau_b, s.BRAKE_S)
        return tau_b + s_ - s_ * s_ / (4 * s.BRAKE_S), 1.0 - s_ / s.BRAKE_S
    patch(s.Swing, 'sigma', g)
def m_offset_dropped():
    # the aim's contact-elevation offset left out of [4]'s fall (step 1's κ = 1): the blend alone brings
    # the bat to the jet
    b = s.Template._build_v3
    def g(self):
        b(self); self.E_off = 0.0
        self.c1E = s.poly_fit([(-1.0, 0, self.E_L), (-1.0, 1, 0.0), (-1.0, 2, 0.0)] + [(self.u93, k, self._seg2E(self.u93, k)) for k in range(3)])
    patch(s.Template, '_build_v3', g)

# The stance refit's (PLAN D28).
def m_stance_today():
    patch(s, 'P_STANCE', s.P_TODAY)
def m_rate_on_azimuth():
    # the rate shape scaled to the contact azimuth rate, as before the refit
    patch(s, 'h_scale', lambda E_c: 1.0)
def m_hands_uncapped():
    patch(s, 'LAM_W_MAX', 10.0)
def m_kappa_back():
    # step 1's κ-scaled fall instead of the offset
    def g(self, u, k=0):
        t = 1e3 * s.T4 * u
        kap = (self.E_c[0] - s.ARMS['93']['bat_elev'] * s.DEG) / ((s.EL4(0.0) - s.EL4(1e3 * s.T4_93)) * s.DEG)
        if k == 0: return s.ARMS['93']['bat_elev'] * s.DEG + kap * (s.EL4(t) - s.EL4(1e3 * s.T4_93)) * s.DEG
        return kap * (1e3 * s.T4) ** k * s.EL4(t, k) * s.DEG
    patch(s.Template, '_baseE', g)
def m_offset_to_contact():
    # the offset finished at contact instead of at u_w
    f = s.Template._baseE
    def g(self, u, k=0):
        uw = self.uw; self.uw = 0.0
        try: return f(self, u, k)
        finally: self.uw = uw
    patch(s.Template, '_baseE', g)

MUT_STANCE = [('stance: today\'s spot (P_y +0.30)', m_stance_today), ('stance: rate shape on the contact azimuth rate (no h_scale)', m_rate_on_azimuth),
              ('stance: the hands\' travel not capped', m_hands_uncapped), ('stance: κ-scaled elevation fall instead of the offset', m_kappa_back),
              ('stance: the elevation offset finished at contact', m_offset_to_contact)]
def m_W93_unscaled():
    c = s.SwingStep1._conds_W
    def g(self, i):
        out = c(self, i); out[3] = (-0.093 / self.T, 0, out[3][2]); return out
    patch(s.SwingStep1, '_conds_W', g)
def m_S_linear():
    def g(self, tau, k=0):
        (t0, a), (t1, b), (t2, c) = self.S_knots
        if tau <= t1:
            w = (tau - t0) / (t1 - t0); return tuple(x + w * (y - x) for x, y in zip(a, b)) if k == 0 else tuple((y - x) / (t1 - t0) for x, y in zip(a, b))
        w = (tau - t1) / (t2 - t1); return tuple(x + w * (y - x) for x, y in zip(b, c)) if k == 0 else tuple((y - x) / (t2 - t1) for x, y in zip(b, c))
    patch(s.Swing, 'S_rh', g)

# The swing family's (§6.7, PLAN D27).
def m_effort_left_out_of_time():
    patch(s.Swing, 'swing_time', staticmethod(lambda T_t, t_swing, effort: T_t * (t_swing / s.T_MEAN)))
def m_effort_in_travel():
    # effort multiplies the travel at a fixed time (the launch moves with effort; review P2-5)
    patch(s.Swing, 'swing_time', staticmethod(lambda T_t, t_swing, effort: T_t * (t_swing / s.T_MEAN)))
    init = s.Swing.__init__
    trav = s.Swing.__dict__['travel']
    def new(self, aim, *a, **k):
        e = k.get('effort', 0.0)
        s.Swing.travel = staticmethod(lambda v, t: trav.__func__(v, t) * (1.0 + e))
        try:
            init(self, aim, *a, **k)
        finally:
            s.Swing.travel = trav
    patch(s.Swing, '__init__', new)
def m_travel_without_own_time():
    patch(s.Swing, 'travel', staticmethod(lambda v, t_swing: v / s.V_TEMPLATE))
def m_no_fade():
    patch(s, 'h_fade', lambda u, u93, k=0: 0.0)
def m_template_one_time():
    patch(s.Template, 'template_time', staticmethod(lambda A0, A1, A_L: s.T_MEAN))
def m_replan_travel_held():
    patch(s.Swing, 'replanned_travel', staticmethod(lambda lam_nom, T, T_nom: lam_nom))
def m_step1_anchor():
    _dA_variant(True)

MUT_FAMILY = [('family: effort left out of the time (the contact correction absorbs the rate)', m_effort_left_out_of_time),
              ('family: effort multiplies the travel at a fixed time (the launch moves)', m_effort_in_travel),
              ('family: travel follows BatSpeed only (not his own time)', m_travel_without_own_time),
              ('family: no fade (the launch moves with the pitch)', m_no_fade),
              ('family: the template at T_mean for every aim', m_template_one_time),
              ('family: a re-plan holds the travel', m_replan_travel_held),
              ('family: the step-1 anchor (168 deg at -93 ms) in the template', m_step1_anchor)]

MUT_REVIEW = [('review: rate shape 2nd derivative chain factor dropped', m_join_chain),
              ('review: elevation blend rate without the smoothstep-rate term', m_blend_rate),
              ('review: δ_A acceleration not less the shape', m_join_Aacc), ('review: u_w at -20 ms', m_uw20),
              ('review: ruling on the unbraked path', m_brake_ruling), ('review: braked velocity not scaled by σ̇', m_brake_rate),
              ('review: brake travel 3B/4', m_brake_travel), ('review: the aim\'s elevation offset dropped (κ = 1)', m_offset_dropped),
              ('review: hands pass W_93 at the unscaled -93 ms', m_W93_unscaled), ('review: S piecewise linear', m_S_linear)]

MUT = [('frame: shoulder azimuth reading 2', m_reading2), ('frame: trunk x_T and y_T swapped', m_frame_swap),
       ('sign: azimuth sin A negated', m_azimuth_sign), ('anchor instant: -80 ms instead of -93', m_anchor_80),
       ('anchor instant: -93 ms unscaled by s_T', m_anchor_unscaled), ('s_T = T/0.150 (T4 0.150)', m_sT_150),
       ('s_T = 1 (no scaling)', m_sT_none), ('mirror: omega as a polar vector', m_omega_mirror)]
only = sys.argv[1:]
for name, m in [('none', lambda: None)] + MUT + MUT_REVIEW + MUT_FAMILY + MUT_STANCE:
    if only and not any(o in name for o in only):
        continue
    m()
    s._CHI.clear(); s._CHI1.clear(); t._cache.clear(); t._diag.clear()
    suite = unittest.defaultTestLoader.loadTestsFromModule(t)
    r = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(suite)
    caught = sorted({x[0].id().split('.', 1)[1] for x in r.failures + r.errors} | {'UNEXPECTED SUCCESS ' + x.id().split('.', 1)[1] for x in r.unexpectedSuccesses})
    print(f'== {name}: {len(caught)} caught')
    for c in caught: print('   ', c)
    restore()
