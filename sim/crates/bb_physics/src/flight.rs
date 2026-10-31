//! Ball flight (physics-spec §4–§5): forces, fixed-step RK4, cubic Hermite dense output, and
//! event location by bisection on the interpolant.

use bb_math::{Vec3, m};

use crate::ball::{AeroParams, BallParams};

/// Airspeed at which τ₁₀₀ is defined: 100 mph in m/s.
const V100: f64 = 44.704;

/// Position, velocity, spin (rad/s, right-hand rule; physics-spec §1.3).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BallState {
    pub r: Vec3,
    pub v: Vec3,
    pub w: Vec3,
}

/// Everything flight needs besides the state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlightEnv {
    pub ball: BallParams,
    pub aero: AeroParams,
    /// Air density, kg/m³.
    pub rho: f64,
    /// Air velocity, m/s (zero in v1).
    pub wind: Vec3,
}

impl FlightEnv {
    pub fn new(rho: f64) -> Self {
        Self { ball: BallParams::default(), aero: AeroParams::default(), rho, wind: Vec3::ZERO }
    }
    /// Calculator-default atmosphere with default aero (spin decay on).
    pub fn default_env() -> Self {
        Self::new(crate::atmosphere::air_density(&Default::default()))
    }
    /// Calculator-default atmosphere and aero, no spin decay: the golden-test setup.
    pub fn calculator() -> Self {
        Self {
            ball: BallParams::default(),
            aero: AeroParams::calculator(),
            rho: crate::atmosphere::air_density(&Default::default()),
            wind: Vec3::ZERO,
        }
    }

    /// Total acceleration (physics-spec §4.2).
    pub fn accel(&self, v: Vec3, w: Vec3) -> Vec3 {
        let g = Vec3::new(0.0, 0.0, -bb_math::units::G);
        let u_vec = v - self.wind;
        let u = u_vec.norm();
        if u == 0.0 || self.rho == 0.0 {
            return g;
        }
        let uh = u_vec / u;
        let k = self.rho * self.ball.area() / (2.0 * self.ball.mass);
        let rpm = bb_math::units::radps_to_rpm(w.norm());
        let cd = self.aero.cd0 + self.aero.cdspin * (rpm / 1000.0);
        let mut a = g - uh * (k * cd * u * u);
        if self.aero.magnus {
            let w_perp = w.reject(uh);
            let wp = w_perp.norm();
            if wp > 0.0 {
                let s = self.ball.radius * wp / u;
                let cl = self.aero.cl2 * s / (self.aero.cl0 + self.aero.cl1 * s);
                let dir = w_perp.cross(uh).normalized();
                a += dir * (k * cl * u * u);
            }
        }
        a
    }

    /// Spin decay rate dω/dt (physics-spec §4.4): −u/(τ₁₀₀·44.704)·ω.
    #[inline]
    fn spin_rate(&self, v: Vec3, w: Vec3) -> Vec3 {
        match self.aero.tau100 {
            Some(tau) => w * (-(v - self.wind).norm() / (tau * V100)),
            None => Vec3::ZERO,
        }
    }

    /// One classic RK4 step of the full state (r, v, ω). Spin decay is integrated with the
    /// rest of the state rather than applied once per step: this matches the continuous
    /// ODE to ~1e-9 m at production step sizes (checked against the oracle).
    pub fn step(&self, s: &BallState, dt: f64) -> BallState {
        let h2 = dt / 2.0;
        let (r0, v0, w0) = (s.r, s.v, s.w);
        let a1 = self.accel(v0, w0);
        let o1 = self.spin_rate(v0, w0);
        let (v2, w2) = (v0 + a1 * h2, w0 + o1 * h2);
        let a2 = self.accel(v2, w2);
        let o2 = self.spin_rate(v2, w2);
        let (v3, w3) = (v0 + a2 * h2, w0 + o2 * h2);
        let a3 = self.accel(v3, w3);
        let o3 = self.spin_rate(v3, w3);
        let (v4, w4) = (v0 + a3 * dt, w0 + o3 * dt);
        let a4 = self.accel(v4, w4);
        let o4 = self.spin_rate(v4, w4);
        BallState {
            r: r0 + (v0 + v2 * 2.0 + v3 * 2.0 + v4) * (dt / 6.0),
            v: v0 + (a1 + a2 * 2.0 + a3 * 2.0 + a4) * (dt / 6.0),
            w: w0 + (o1 + o2 * 2.0 + o3 * 2.0 + o4) * (dt / 6.0),
        }
    }
}

/// A dense-output knot: the state at one integrator step boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Knot {
    pub t: f64,
    pub s: BallState,
}

/// Cubic Hermite interpolation of position (and its derivative) between two knots.
pub fn hermite(a: &Knot, b: &Knot, t: f64) -> (Vec3, Vec3) {
    let h = b.t - a.t;
    let s = (t - a.t) / h;
    let s2 = s * s;
    let s3 = s2 * s;
    let h00 = 2.0 * s3 - 3.0 * s2 + 1.0;
    let h10 = s3 - 2.0 * s2 + s;
    let h01 = -2.0 * s3 + 3.0 * s2;
    let h11 = s3 - s2;
    let r = a.s.r * h00 + a.s.v * (h10 * h) + b.s.r * h01 + b.s.v * (h11 * h);
    let d00 = (6.0 * s2 - 6.0 * s) / h;
    let d10 = 3.0 * s2 - 4.0 * s + 1.0;
    let d01 = (-6.0 * s2 + 6.0 * s) / h;
    let d11 = 3.0 * s2 - 2.0 * s;
    let v = a.s.r * d00 + a.s.v * d10 + b.s.r * d01 + b.s.v * d11;
    (r, v)
}

/// A stop condition: the event fires when `g(r)` goes from positive to ≤ 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Crossing {
    /// Ball center reaches `y` while moving toward −Y (plate planes).
    YBelow(f64),
    /// Ball center reaches `y` while moving toward +Y.
    YAbove(f64),
    /// Ball center descends to height `z` (ground contact uses z = R).
    ZBelow(f64),
    /// Horizontal distance from home exceeds the fence distance at the ball's spray angle
    /// (quadratic through left-line, center, right-line distances; see `ground::Field`).
    RadiusAbove { lf: f64, cf: f64, rf: f64 },
}

impl Crossing {
    #[inline]
    fn g(&self, r: Vec3) -> f64 {
        match *self {
            Crossing::YBelow(y) => r.y - y,
            Crossing::YAbove(y) => y - r.y,
            Crossing::ZBelow(z) => r.z - z,
            Crossing::RadiusAbove { lf, cf, rf } => {
                let theta = m::atan2(r.x, r.y);
                let s = (theta / core::f64::consts::FRAC_PI_4).clamp(-1.0, 1.0);
                let side = if s < 0.0 { lf } else { rf };
                (cf + (side - cf) * s * s) - m::hypot(r.x, r.y)
            }
        }
    }
}

/// Bisection tolerance on event time, s (physics-spec §5.3 asks for ≤ 1 µs).
pub const EVENT_TOL_S: f64 = 1e-9;

/// A recorded flight: knots at every step plus the exact event state as the last knot.
#[derive(Clone, Debug, Default)]
pub struct Trajectory {
    pub knots: Vec<Knot>,
    /// Index into the caller's `stops` of the crossing that ended the flight, if any.
    pub stop: Option<usize>,
    /// Highest point reached by the ball center (z, m) and when.
    pub apex_z: f64,
    pub apex_t: f64,
}

impl Trajectory {
    pub fn first(&self) -> &Knot {
        &self.knots[0]
    }
    pub fn last(&self) -> &Knot {
        self.knots.last().unwrap()
    }
    pub fn duration(&self) -> f64 {
        self.last().t - self.first().t
    }
    /// Position and velocity at time `t` (clamped to the recorded span).
    pub fn at(&self, t: f64) -> (Vec3, Vec3) {
        let k = &self.knots;
        if t <= k[0].t {
            return (k[0].s.r, k[0].s.v);
        }
        if t >= self.last().t {
            return (self.last().s.r, self.last().s.v);
        }
        // Binary search for the containing segment.
        let i = k.partition_point(|x| x.t <= t) - 1;
        hermite(&k[i], &k[i + 1], t)
    }
    /// Time at which the path first satisfies `c`, located on the interpolant.
    pub fn find(&self, c: Crossing) -> Option<f64> {
        self.knots.windows(2).find_map(|w| crossing_in(&w[0], &w[1], c))
    }
}

/// Interior samples per step used to bracket a crossing on the interpolant, so a path that
/// starts on a boundary, leaves it, and returns within one step is still caught.
const BRACKET_SAMPLES: usize = 4;
/// Interior sampling happens only when an endpoint is within this distance of the boundary.
const BRACKET_MARGIN_M: f64 = 1e-3;

/// First downward crossing of `c` (g from > 0 to ≤ 0) on the Hermite segment a→b.
fn crossing_in(a: &Knot, b: &Knot, c: Crossing) -> Option<f64> {
    let h = b.t - a.t;
    let (ga, gb) = (c.g(a.s.r), c.g(b.s.r));
    // A cubic segment over one step departs from its chord by at most h²·|a|max/8, far
    // below this margin for any ball; well clear of the boundary, only the endpoints matter.
    if ga > BRACKET_MARGIN_M && gb > BRACKET_MARGIN_M {
        return None;
    }
    if ga > 0.0 && gb <= 0.0 && ga > BRACKET_MARGIN_M {
        return Some(bisect_between(a, b, c, a.t, b.t));
    }
    let mut tp = a.t;
    let mut gp = ga;
    for i in 1..=BRACKET_SAMPLES + 1 {
        let (t, g) = if i == BRACKET_SAMPLES + 1 {
            (b.t, c.g(b.s.r))
        } else {
            let t = a.t + h * i as f64 / (BRACKET_SAMPLES + 1) as f64;
            (t, c.g(hermite(a, b, t).0))
        };
        if gp > 0.0 && g <= 0.0 {
            return Some(bisect_between(a, b, c, tp, t));
        }
        tp = t;
        gp = g;
    }
    None
}

fn bisect_between(a: &Knot, b: &Knot, c: Crossing, lo: f64, hi: f64) -> f64 {
    let (mut lo, mut hi) = (lo, hi);
    while hi - lo > EVENT_TOL_S {
        let mid = 0.5 * (lo + hi);
        if c.g(hermite(a, b, mid).0) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    hi
}

/// Integrate from `s0` at time `t0` with fixed step `dt` until one of `stops` fires or
/// `t_max` elapses. The final knot is the exact state at the event, rebuilt by a partial
/// RK4 step from the start of the step in which it happened (physics-spec §5.3).
///
/// Panics on a non-finite or non-positive `dt`, or a negative or non-finite `t_max`: those
/// are programming errors, not physical situations.
pub fn fly(env: &FlightEnv, s0: BallState, t0: f64, dt: f64, stops: &[Crossing], t_max: f64) -> Trajectory {
    assert!(dt.is_finite() && dt > 0.0, "fly: step must be finite and positive, got {dt}");
    assert!(t_max.is_finite() && t_max >= 0.0 && t0.is_finite(), "fly: bad time span {t0} + {t_max}");
    let mut tr = Trajectory { knots: Vec::with_capacity(256), stop: None, apex_z: s0.r.z, apex_t: t0 };
    tr.knots.push(Knot { t: t0, s: s0 });
    let mut cur = Knot { t: t0, s: s0 };
    // A stop already satisfied at the start fires only if the path leaves it and returns.
    while cur.t < t0 + t_max {
        let h = dt.min(t0 + t_max - cur.t);
        if cur.t + h <= cur.t {
            break; // the step no longer advances time at this magnitude
        }
        let next = Knot { t: cur.t + h, s: env.step(&cur.s, h) };
        // Earliest event inside this step.
        let mut hit: Option<(usize, f64)> = None;
        for (i, c) in stops.iter().enumerate() {
            if let Some(te) = crossing_in(&cur, &next, *c) {
                if hit.map(|(_, t)| te < t).unwrap_or(true) {
                    hit = Some((i, te));
                }
            }
        }
        let end = match hit {
            Some((_, te)) => Knot { t: te, s: env.step(&cur.s, te - cur.t) },
            None => next,
        };
        track_apex(&mut tr, &cur, &end);
        tr.knots.push(end);
        if let Some((i, _)) = hit {
            tr.stop = Some(i);
            return tr;
        }
        cur = end;
    }
    tr
}

fn track_apex(tr: &mut Trajectory, a: &Knot, b: &Knot) {
    if a.s.v.z > 0.0 && b.s.v.z <= 0.0 {
        // Bisect the interpolant's vertical velocity for the apex.
        let (mut lo, mut hi) = (a.t, b.t);
        while hi - lo > EVENT_TOL_S {
            let mid = 0.5 * (lo + hi);
            if hermite(a, b, mid).1.z > 0.0 { lo = mid } else { hi = mid }
        }
        let z = hermite(a, b, lo).0.z;
        if z > tr.apex_z {
            tr.apex_z = z;
            tr.apex_t = lo;
        }
    }
    if b.s.r.z > tr.apex_z {
        tr.apex_z = b.s.r.z;
        tr.apex_t = b.t;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_to_boundary_within_first_step_is_caught() {
        // Codex review #5: start on z = R, rise 0.01 m/s, come back down inside one 5 ms step.
        let mut env = FlightEnv::calculator();
        env.rho = 0.0;
        let r = env.ball.radius;
        let s0 = BallState { r: Vec3::new(0.0, 0.0, r), v: Vec3::new(0.0, 0.0, 0.01), w: Vec3::ZERO };
        let tr = fly(&env, s0, 0.0, 0.005, &[Crossing::ZBelow(r)], 0.01);
        assert_eq!(tr.stop, Some(0));
        let expect = 2.0 * 0.01 / bb_math::units::G;
        assert!((tr.last().t - expect).abs() < 1e-8, "{} vs {expect}", tr.last().t);
        assert_eq!(tr.find(Crossing::ZBelow(r)).map(|t| (t - expect).abs() < 1e-8), Some(true));
    }

    #[test]
    #[should_panic(expected = "finite and positive")]
    fn zero_step_is_rejected() {
        fly(&FlightEnv::calculator(), BallState::default(), 0.0, 0.0, &[], 1.0);
    }
}

/// Default integrator steps, s (physics-spec §5.2).
pub const DT_PITCH: f64 = 0.002;
pub const DT_BATTED: f64 = 0.005;
