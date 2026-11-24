"""Run the v3 oracle tests (test_swing_v3.py) under deliberate mistakes and list the tests that catch
each: a wrong frame, a wrong sign, a wrong anchor instant, a wrong s_T, a wrong mirror. Run from the
repository root: python3 analysis/oracle/mutations_swing_v3.py"""
import sys, math, unittest, io
sys.path.insert(0, 'analysis/oracle')
import swing_v3 as s
import test_swing_v3 as t

orig = {}
def patch(obj, name, val):
    orig.setdefault((obj, name), getattr(obj, name)); setattr(obj, name, val)
def restore():
    for (obj, name), v in orig.items(): setattr(obj, name, v)
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
    b = s.Swing._build
    def new(self):
        self.tau93 = -0.093; b(self)
    patch(s.Swing, '_build', new)
def m_sT_150():
    patch(s, 'T4', 0.150)
def m_sT_none():
    init = s.Swing.__init__
    def new(self, *a, **k):
        init(self, *a, **k)
    b = s.Swing._build
    def nb(self):
        self.sT = 1.0; self.tau93 = s.T4_93; b(self)
    patch(s.Swing, '_build', nb)
def m_omega_mirror():
    pose = s.Swing.pose
    def new(self, t, tau_b=None):
        p = pose(self, t, tau_b)
        if self.side == -1: p['omega'] = (-p['omega'][0], -p['omega'][1], p['omega'][2])
        return p
    patch(s.Swing, 'pose', new)

MUT = [('frame: shoulder azimuth reading 2', m_reading2), ('frame: trunk x_T and y_T swapped', m_frame_swap),
       ('sign: azimuth sin A negated', m_azimuth_sign), ('anchor instant: -80 ms instead of -93', m_anchor_80),
       ('anchor instant: -93 ms unscaled by s_T', m_anchor_unscaled), ('s_T = T/0.150 (T4 0.150)', m_sT_150),
       ('s_T = 1 (no scaling)', m_sT_none), ('mirror: omega as a polar vector', m_omega_mirror)]
for name, m in [('none', lambda: None)] + MUT:
    m()
    s._CHI.clear(); t._cache.clear(); t._diag.clear()
    suite = unittest.defaultTestLoader.loadTestsFromModule(t)
    r = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(suite)
    caught = sorted({x[0].id().split('.', 1)[1] for x in r.failures + r.errors} | {'UNEXPECTED SUCCESS ' + x.id().split('.', 1)[1] for x in r.unexpectedSuccesses})
    print(f'== {name}: {len(caught)} caught')
    for c in caught: print('   ', c)
    restore()
