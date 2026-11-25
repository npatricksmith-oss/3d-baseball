"""Shared pieces of the body-motion probes (docs/body-motion.md). Stdlib only.

- de Leva 1996 segment table (J Biomech 29(9):1223-1230, Table 4, males: 73.0 kg, 1.741 m),
  read from the paper (https://robotics.unibs.it/Antropometria/deLeva-ParametriDinamiciAntropometria.pdf).
- the game's default bat (physics-spec 6.2).
- loading and smoothing of Fortenbaugh 2011 [4] Appendix E curves, digitised by
  analysis/research/fortenbaugh_appendix_e.py into sim/target/research-dl/4/csv/.
"""
import csv, math, os

G = 9.81
# de Leva 1996 Table 4, males. (length m at 1.741 m stature, mass fraction, CM from origin as fraction of
# length, radius of gyration fractions: sagittal, transverse, longitudinal)
DE_LEVA_H = 1.741
DE_LEVA = {
    "head":     (0.2429, 0.0694, 0.5002, 0.303, 0.315, 0.261),   # vertex -> cervicale
    "upt":      (0.2421, 0.1596, 0.5066, 0.505, 0.320, 0.465),   # cervicale -> xyphion
    "mpt":      (0.2155, 0.1633, 0.4502, 0.482, 0.383, 0.468),
    "lpt":      (0.1457, 0.1117, 0.6115, 0.615, 0.551, 0.587),
    "upperarm": (0.2817, 0.0271, 0.5772, 0.285, 0.269, 0.158),
    "forearm":  (0.2689, 0.0162, 0.4574, 0.276, 0.265, 0.121),
    "hand":     (0.0862, 0.0061, 0.7900, 0.628, 0.513, 0.401),
    "thigh":    (0.4222, 0.1416, 0.4095, 0.329, 0.329, 0.149),
    "shank":    (0.4340, 0.0433, 0.4459, 0.255, 0.249, 0.103),
    "foot":     (0.2581, 0.0137, 0.4415, 0.257, 0.245, 0.124),
}


def segment(name, H, M):
    """(length m, mass kg, cm fraction, I_sag, I_trans, I_long kg m^2) scaled by stature and mass.
    de Leva: I = (M m_rel) (l r_rel)^2. Scaling l with stature is [E]: de Leva gives no scaling rule and suggests
    measuring the individual's segment lengths (docs/body-motion.md 4.1)."""
    l, m, c, rs, rt, rl = DE_LEVA[name]
    L = l * H / DE_LEVA_H
    mk = m * M
    return L, mk, c, mk * (L * rs) ** 2, mk * (L * rt) ** 2, mk * (L * rl) ** 2


# The game's default bat, physics-spec 6.2 (integrated from the profile, rho 660): 34 in.
BAT_L = 0.8636
BAT_M = 0.965
BAT_UCM = 0.597           # cm from the knob, m
BAT_ICM = 0.0382          # transverse, about the cm
BAT_IAX = 3.96e-4
GRIP_FROM_KNOB = 0.08     # [E] the hands' centre on the handle
BAT_SS = BAT_L - 0.15     # sweet spot from the knob (physics-spec 7, Statcast's 6-in point)

DATA = os.environ.get("BB_FORT_CSV", "sim/target/research-dl/4/csv")


def load(name):
    with open(os.path.join(DATA, name + ".csv")) as f:
        rows = list(csv.reader(f))[1:]
    return [(float(a) / 1000.0, float(b)) for a, b in rows]   # (t s, value)


def resample(rows, t0, t1, dt):
    """Linear interpolation of a digitised curve on a uniform grid."""
    out, j, n = [], 0, len(rows)
    t = t0
    while t <= t1 + 1e-12:
        while j < n - 2 and rows[j + 1][0] < t:
            j += 1
        (ta, va), (tb, vb) = rows[j], rows[j + 1]
        out.append(va + (vb - va) * (t - ta) / (tb - ta) if tb != ta else va)
        t += dt
    return out


def smooth(v, half):
    """Centred moving average over 2*half+1 points, applied twice (a triangular kernel)."""
    def ma(x):
        n = len(x); o = []
        for i in range(n):
            a, b = max(0, i - half), min(n, i + half + 1)
            o.append(sum(x[a:b]) / (b - a))
        return o
    return ma(ma(v))


def deriv(v, dt):
    n = len(v)
    return [((v[min(n - 1, i + 1)] - v[max(0, i - 1)]) / (dt * (min(n - 1, i + 1) - max(0, i - 1)))) for i in range(n)]
