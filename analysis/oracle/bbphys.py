"""Independent, standard-library reference for docs/physics-spec.md.

Units are SI. Vectors are length-three tuples; matrices are row-major tuples.
See NOTES.md for interpretation choices. No simulation implementation is used.
"""
from dataclasses import dataclass
import math

MPH = 0.44704
FT = 0.3048
INCH = 0.0254
RPM = math.pi / 30.0
OZ = 0.028349523
G = 9.80665
MASS = 0.14529
CIRCUMFERENCE = 0.231775
RADIUS = CIRCUMFERENCE / (2.0 * math.pi)
AREA = math.pi * RADIUS**2
ALPHA = 0.4
BALL_INERTIA = ALPHA * MASS * RADIUS**2
PLATE_Y = 0.2159
ZERO = (0.0, 0.0, 0.0)
IDENTITY = ((1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0))
ZERO_MATRIX = ((0.0, 0.0, 0.0),) * 3


def add(a, b):
    return tuple(x + y for x, y in zip(a, b))


def sub(a, b):
    return tuple(x - y for x, y in zip(a, b))


def scale(a, s):
    return tuple(x * s for x in a)


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def norm(a):
    return math.sqrt(dot(a, a))


def unit(a):
    length = norm(a)
    if length == 0:
        raise ValueError('zero vector has no direction')
    return scale(a, 1.0 / length)


def cross(a, b):
    return (a[1]*b[2] - a[2]*b[1], a[2]*b[0] - a[0]*b[2],
            a[0]*b[1] - a[1]*b[0])


def matvec(a, v):
    return tuple(dot(row, v) for row in a)


def matmul(a, b):
    return tuple(tuple(sum(a[i][k]*b[k][j] for k in range(3))
                       for j in range(3)) for i in range(3))


def diagonal(s):
    return tuple(tuple(s if i == j else 0.0 for j in range(3)) for i in range(3))


def skew(v):
    x, y, z = v
    return ((0.0, -z, y), (z, 0.0, -x), (-y, x, 0.0))


def solve(a, b):
    """Partial-pivot Gaussian elimination, independent of third-party algebra."""
    rows = [list(a[i]) + [b[i]] for i in range(3)]
    for i in range(3):
        pivot = max(range(i, 3), key=lambda k: abs(rows[k][i]))
        rows[i], rows[pivot] = rows[pivot], rows[i]
        if rows[i][i] == 0:
            raise ValueError('singular matrix')
        for j in range(i + 1, 3):
            ratio = rows[j][i] / rows[i][i]
            for k in range(i, 4):
                rows[j][k] -= ratio * rows[i][k]
    out = [0.0] * 3
    for i in range(2, -1, -1):
        out[i] = (rows[i][3] - sum(rows[i][j]*out[j] for j in range(i+1, 3))) / rows[i][i]
    return tuple(out)


def air_density(temp_f=70.0, pressure_inhg=29.92, rh_percent=50.0, elev_ft=15.0):
    tc = (temp_f - 32.0) * 5.0 / 9.0
    svp = 4.5841 * math.exp((18.687 - tc/234.5) * tc / (257.14 + tc))
    return (1.2929 * 273.0/(tc + 273.0)
            * (pressure_inhg*25.4*math.exp(-1.217e-4*elev_ft*FT)
               - 0.3783*rh_percent*svp/100.0) / 760.0)


@dataclass(frozen=True)
class Aero:
    cd0: float = 0.3008
    cdspin: float = 0.0292
    cl0: float = 0.583
    cl1: float = 2.333
    cl2: float = 1.120


@dataclass(frozen=True)
class State:
    r: tuple
    v: tuple
    w: tuple

    def packed(self):
        return tuple(self.r) + tuple(self.v) + tuple(self.w)

    @classmethod
    def unpack(cls, values):
        return cls(tuple(values[:3]), tuple(values[3:6]), tuple(values[6:9]))


def flight_rhs(s, rho, tau100=None, wind=ZERO, aero=Aero(), magnus=True):
    """Full nine-component ODE, including continuous spin decay."""
    vx, vy, vz = s[3:6]
    wx, wy, wz = s[6:9]
    ux, uy, uz = vx-wind[0], vy-wind[1], vz-wind[2]
    speed = math.sqrt(ux*ux + uy*uy + uz*uz)
    spin = math.sqrt(wx*wx + wy*wy + wz*wz)
    ax, ay, az = 0.0, 0.0, -G
    if speed > 0.0 and rho != 0.0:
        k = rho * AREA / (2.0*MASS)
        drag = -k * (aero.cd0 + aero.cdspin*spin/(1000.0*RPM)) * speed
        ax, ay, az = drag*ux, drag*uy, az + drag*uz
        # omega x u is exactly omega_perp x u; avoids cancellation in projection.
        cx, cy, cz = wy*uz-wz*uy, wz*ux-wx*uz, wx*uy-wy*ux
        cmag = math.sqrt(cx*cx + cy*cy + cz*cz)
        if magnus and cmag > 0.0:
            sf = RADIUS * cmag / (speed*speed)
            cl = aero.cl2 * sf / (aero.cl0 + aero.cl1*sf)
            lift = k * cl * speed*speed / cmag
            ax, ay, az = ax + lift*cx, ay + lift*cy, az + lift*cz
    decay = 0.0 if tau100 is None else -speed/(tau100*100.0*MPH)
    return (vx, vy, vz, ax, ay, az, decay*wx, decay*wy, decay*wz)


def acceleration(v, w, rho, wind=ZERO, aero=Aero(), magnus=True):
    return flight_rhs(ZERO + tuple(v) + tuple(w), rho, None, wind, aero, magnus)[3:6]


def rk4_step(s, dt, rho, tau100=None, wind=ZERO, aero=Aero(), magnus=True):
    def rhs(x):
        return flight_rhs(x, rho, tau100, wind, aero, magnus)
    k1 = rhs(s)
    k2 = rhs(tuple(s[i] + dt*0.5*k1[i] for i in range(9)))
    k3 = rhs(tuple(s[i] + dt*0.5*k2[i] for i in range(9)))
    k4 = rhs(tuple(s[i] + dt*k3[i] for i in range(9)))
    return tuple(s[i] + dt/6.0*(k1[i] + 2*k2[i] + 2*k3[i] + k4[i]) for i in range(9))


def hermite(p0, v0, p1, v1, h, q):
    return ((2*q**3-3*q*q+1)*p0 + (q**3-2*q*q+q)*h*v0
            + (-2*q**3+3*q*q)*p1 + (q**3-q*q)*h*v1)


def hermite_velocity(p0, v0, p1, v1, h, q):
    return ((6*q*q-6*q)*p0/h + (3*q*q-4*q+1)*v0
            + (-6*q*q+6*q)*p1/h + (3*q*q-2*q)*v1)


def bisect_fraction(function, h, tolerance=1e-10):
    """Root on [0,1], accepting either orientation, to time tolerance."""
    lo, hi = 0.0, 1.0
    flo = function(lo)
    if flo == 0.0:
        return lo
    if function(hi) == 0.0:
        return hi
    if flo * function(hi) > 0:
        raise ValueError('root not bracketed')
    while (hi-lo)*h > tolerance:
        mid = (lo+hi)*0.5
        fm = function(mid)
        if fm == 0.0:
            return mid
        if (flo > 0) == (fm > 0):
            lo, flo = mid, fm
        else:
            hi = mid
    return (lo+hi)*0.5


def integrate(state, rho, tau100=None, stop=None, dt=1e-4, max_time=60.0,
              wind=ZERO, aero=Aero(), magnus=True):
    """RK4 flight until a decreasing ground or Y-plane crossing.

    Position events and apex use cubic Hermite dense output; event state is
    always rebuilt by an RK4 partial step. Ground means center z=0, per TASK.
    """
    if dt <= 0 or (tau100 is not None and tau100 <= 0):
        raise ValueError('dt and finite decay time must be positive')
    stop = {'kind': 'ground'} if stop is None else stop
    if stop['kind'] == 'ground':
        axis, target = 2, 0.0
    elif stop['kind'] == 'plane_y':
        axis, target = 1, stop['y']
    else:
        raise ValueError('unknown event')
    s = state.packed()
    if s[axis] < target:
        raise ValueError('initial state is beyond event plane')
    if s[axis] == target and s[axis+3] < 0:
        return dict(t=0.0, r=s[:3], v=s[3:6], w=s[6:9], apex_z=s[2])
    apex = s[2]
    for index in range(math.ceil(max_time/dt)):
        h = min(dt, max_time-index*dt)
        end = rk4_step(s, h, rho, tau100, wind, aero, magnus)
        event_q = None
        if s[axis] >= target and end[axis] <= target:
            q = bisect_fraction(lambda q: hermite(s[axis], s[axis+3], end[axis],
                                                 end[axis+3], h, q)-target, h)
            if hermite_velocity(s[axis], s[axis+3], end[axis], end[axis+3], h, q) < 0:
                event_q = q
        if s[5] > 0 and end[5] <= 0:
            aq = bisect_fraction(lambda q: hermite_velocity(s[2], s[5], end[2], end[5], h, q), h)
            if event_q is None or aq <= event_q:
                apex = max(apex, rk4_step(s, aq*h, rho, tau100, wind, aero, magnus)[2])
        if event_q is not None:
            event = rk4_step(s, event_q*h, rho, tau100, wind, aero, magnus)
            return dict(t=index*dt + event_q*h, r=event[:3], v=event[3:6],
                        w=event[6:9], apex_z=max(apex, event[2]))
        apex = max(apex, end[2])
        s = end
    raise RuntimeError('event not reached before max_time')


def launch_velocity(speed_mph, elevation_deg, spray_deg=0.0):
    theta, phi = math.radians(elevation_deg), math.radians(spray_deg)
    speed = speed_mph * MPH
    return (speed*math.cos(theta)*math.sin(phi), speed*math.cos(theta)*math.cos(phi),
            speed*math.sin(theta))


def spin_metrics(w, v):
    vh = unit(v)
    wp = sub(w, scale(vh, dot(w, vh)))
    efficiency = min(1.0, norm(wp)/norm(w)) if norm(w) else 0.0
    direction = cross(w, vh)
    # A vanishing transverse spin (or X-Z projection) has no display axis.
    if norm(direction) <= 1e-14*norm(w) or math.hypot(direction[0], direction[2]) == 0:
        return dict(spin_axis_deg=None, tilt_hours=None, efficiency=efficiency)
    axis = (180.0 + math.degrees(math.atan2(-direction[0], direction[2]))) % 360.0
    return dict(spin_axis_deg=axis, tilt_hours=((axis-180.0)/30.0) % 12.0,
                efficiency=efficiency)


def spin_from_tilt(tilt_hours, efficiency, rate_rpm, gyro_sign=1.0):
    """Inverse display mapping for a level pitch along -Y."""
    if not 0 <= efficiency <= 1:
        raise ValueError('efficiency must be in [0,1]')
    a, rate = tilt_hours*math.pi/6.0, rate_rpm*RPM
    return (-rate*efficiency*math.cos(a),
            -math.copysign(rate*math.sqrt(1-efficiency**2), gyro_sign),
            -rate*efficiency*math.sin(a))


def bat_radius(u):
    inches = u/INCH
    if not 0.0 <= inches <= 34.0 + 1e-12:
        raise ValueError('station outside bat')
    if inches <= 0.5:
        diameter = 1.35
    elif inches <= 1.0:
        diameter = 1.35 + (0.93-1.35)*(inches-0.5)/0.5
    elif inches <= 11.0:
        diameter = 0.93
    elif inches <= 22.0:
        diameter = 0.93 + 1.57*((inches-11.0)/11.0)**1.6
    else:
        diameter = 2.50
    return 0.5*diameter*INCH


def _simpson(function, a, b, tolerance=1e-13, depth=30):
    """Adaptive Simpson quadrature, independently split at profile boundaries."""
    def recurse(a, b, fa, fm, fb, whole, tol, left):
        m = (a+b)*0.5
        fl, fr = function((a+m)*0.5), function((m+b)*0.5)
        sl = (m-a)/6*(fa + 4*fl + fm)
        sr = (b-m)/6*(fm + 4*fr + fb)
        error = sl+sr-whole
        if abs(error) <= 15*tol:
            return sl+sr+error/15
        if left == 0:
            raise RuntimeError('bat quadrature did not converge')
        return (recurse(a, m, fa, fl, fm, sl, tol/2, left-1)
                + recurse(m, b, fm, fr, fb, sr, tol/2, left-1))
    fa, fm, fb = function(a), function((a+b)*0.5), function(b)
    return recurse(a, b, fa, fm, fb, (b-a)/6*(fa+4*fm+fb), tolerance, depth)


def bat_mass_properties(wood_density=660.0):
    breaks = [x*INCH for x in (0, 0.5, 1, 11, 22, 34)]
    def integral(f):
        return sum(_simpson(lambda u: wood_density*math.pi*bat_radius(u)**2*f(u), a, b)
                   for a, b in zip(breaks, breaks[1:]))
    mass = integral(lambda u: 1.0)
    cm = integral(lambda u: u)/mass
    icm = integral(lambda u: (u-cm)**2 + bat_radius(u)**2/4.0)
    iax = integral(lambda u: bat_radius(u)**2/2.0)
    return dict(M=mass, u_cm=cm, I_cm=icm, I_ax=iax,
                moi_6in=icm + mass*(cm-6*INCH)**2, L=34*INCH)


def bat_inverse_inertia(axis, I_cm, I_ax):
    a = unit(axis)
    return tuple(tuple((1.0/I_cm if i == j else 0.0)
                       + (1.0/I_ax-1.0/I_cm)*a[i]*a[j]
                       for j in range(3)) for i in range(3))


def kinetic_energy(m, inv_I1, v, w, inv_M2, inv_I2, V, W):
    def rotation(inv_i, omega):
        if all(x == 0 for row in inv_i for x in row):
            return 0.0
        return 0.5*dot(omega, solve(inv_i, omega))
    return (0.5*m*dot(v, v) + rotation(inv_I1, w)
            + (0.5*dot(V, V)/inv_M2 if inv_M2 else 0.0) + rotation(inv_I2, W))


def collision(m, inv_I1, v, w, r1, inv_M2, inv_I2, V, W, r2, n, e_n, e_t, mu):
    """Section 8.1 sticking/cone/sliding impulse and one e_t=0 energy retry.

    Requires approaching contact and positive-definite inertias (or body 2's
    zero inverse inertia). The supplied normal must be a unit vector.
    """
    if abs(norm(n)-1.0) > 1e-12:
        raise ValueError('contact normal must be unit length')
    if m <= 0 or inv_M2 < 0 or mu < 0:
        raise ValueError('invalid mass or friction')
    c = sub(add(v, cross(w, r1)), add(V, cross(W, r2)))
    cn = dot(c, n)
    if cn >= 0:
        raise ValueError('collision requires approaching contact')
    ct = sub(c, scale(n, cn))
    a = matmul(matmul(skew(r1), inv_I1), skew(r1))
    b = matmul(matmul(skew(r2), inv_I2), skew(r2))
    K = tuple(tuple(((1.0/m + inv_M2) if i == j else 0.0) - a[i][j] - b[i][j]
                    for j in range(3)) for i in range(3))
    before = kinetic_energy(m, inv_I1, v, w, inv_M2, inv_I2, V, W)
    def attempt(et):
        delta = sub(add(scale(n, -e_n*cn), scale(ct, -et)), c)
        impulse = solve(K, delta)
        jn = dot(impulse, n)
        jt = sub(impulse, scale(n, jn))
        branch = 'stick'
        if norm(jt) > mu*jn:
            if norm(ct) == 0:
                raise ValueError('sliding direction undefined for zero tangential speed')
            direction = sub(n, scale(unit(ct), mu))
            denominator = dot(n, matvec(K, direction))
            if denominator <= 0:
                raise ValueError('nonpositive sliding effective inverse mass')
            impulse = scale(direction, -(1+e_n)*cn/denominator)
            branch = 'slide'
        return dict(J=impulse, v=add(v, scale(impulse, 1.0/m)),
                    w=add(w, matvec(inv_I1, cross(r1, impulse))),
                    V=sub(V, scale(impulse, inv_M2)),
                    W=sub(W, matvec(inv_I2, cross(r2, impulse))), branch=branch)
    result = attempt(e_t)
    def energy(result):
        return kinetic_energy(m, inv_I1, result['v'], result['w'], inv_M2,
                              inv_I2, result['V'], result['W'])
    # Tolerance is floating-point roundoff only, not a physical energy allowance.
    tolerance = 64*math.ulp(max(1.0, before))
    guarded = energy(result) > before + tolerance
    if guarded:
        result = attempt(0.0)
        if energy(result) > before + tolerance:
            raise ValueError('energy still increases after prescribed e_t=0 retry')
    result['energy_guard'] = guarded
    return result


def plate_strike(position, batter_height, radius=RADIUS):
    """Circle/rectangle overlap at the midpoint plane, including rounded corners."""
    dx = max(abs(position[0])-0.2159, 0.0)
    dz = max(0.27*batter_height-position[2], position[2]-0.535*batter_height, 0.0)
    return dx*dx + dz*dz <= radius*radius


def flight_telemetry(result, r0=ZERO):
    """Section 12 unobstructed-flight distance, time and apex."""
    return dict(projected_distance=math.hypot(result['r'][0], result['r'][1]),
                travel_distance=math.hypot(result['r'][0]-r0[0], result['r'][1]-r0[1]),
                hang_time=result['t'], apex=result['apex_z'])


def velocity_telemetry(v):
    return dict(speed=norm(v), launch_angle_deg=math.degrees(math.atan2(v[2], math.hypot(v[0], v[1]))),
                spray_angle_deg=math.degrees(math.atan2(v[0], v[1])))


def pitch_movement(state, rho, tau100=None, y_target=PLATE_Y, dt=1e-4):
    """Full-flight movement: each twin crosses the same plane at its own time."""
    stop = dict(kind='plane_y', y=y_target)
    actual = integrate(state, rho, tau100, stop, dt=dt)
    twin = integrate(state, rho, tau100, stop, dt=dt, magnus=False)
    return dict(actual=actual, no_magnus=twin,
                horizontal=actual['r'][0]-twin['r'][0],
                vertical=actual['r'][2]-twin['r'][2])
