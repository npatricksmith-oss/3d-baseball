//! Thrown balls (agent-models §5.5): a ballistic solve with drag and typical throw backspin.
//!
//! A throw is the same ball under the same aerodynamics as a batted one (physics-spec §4);
//! only the spin differs. With no wind the forces do not depend on the horizontal direction,
//! so every solve runs in a frame where the throw heads along +Y and is turned afterwards.

use crate::flight::{BallState, Crossing, FlightEnv, Trajectory, fly};
use bb_math::units::rpm_to_radps;
use bb_math::{Vec3, m};

/// Typical throw backspin, rpm [E] (agent-models §5.5).
pub const THROW_SPIN_RPM: f64 = 1500.0;
/// Integration step for throws, s. Throws last at most a few seconds; at 10 ms the RK4
/// position error is far below a millimetre.
pub const THROW_DT: f64 = 0.01;
/// Steepest launch the solve considers, rad [E]: past about 35° a drag-affected throw gains
/// little range and hangs in the air, which no fielder chooses.
pub const THROW_MAX_ELEV: f64 = 35.0 * core::f64::consts::PI / 180.0;
const THROW_MIN_ELEV: f64 = -30.0 * core::f64::consts::PI / 180.0;
/// The aim solve stops within this height of the target, m: far inside a glove.
const SOLVE_TOL_M: f64 = 0.01;
/// The aim solve's integration step, s. It only has to find the launch angle; the thrown
/// ball itself flies at `THROW_DT`. At 20 ms RK4 is still well under a millimetre off.
const SOLVE_DT: f64 = 0.02;

/// The launch state of a throw released at `r0` along the horizontal unit direction `dir`
/// at `speed` (m/s) and elevation `elev` (rad), with backspin about `dir × Z`.
pub fn throw_state(r0: Vec3, dir: Vec3, speed: f64, elev: f64) -> BallState {
    let d = Vec3::new(dir.x, dir.y, 0.0).normalized();
    let v = d * (speed * m::cos(elev)) + Vec3::new(0.0, 0.0, speed * m::sin(elev));
    // Backspin: a ball moving +Y with backspin spins about +X (physics-spec §1.3).
    let w = d.cross(Vec3::Z) * rpm_to_radps(THROW_SPIN_RPM);
    BallState { r: r0, v, w }
}

/// Fly a throw until it reaches the ground (ball centre at radius height) or `t_max` passes.
pub fn fly_throw(env: &FlightEnv, s0: BallState, t0: f64, t_max: f64) -> Trajectory {
    fly(env, s0, t0, THROW_DT, &[Crossing::ZBelow(env.ball.radius)], t_max)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThrowSolution {
    /// Launch elevation, rad.
    pub elev: f64,
    /// Time from release until the ball is above the target, s.
    pub t_flight: f64,
}

/// Height (m) of a throw at horizontal distance `d` from its release, or, when it lands
/// short, the ball radius minus the shortfall, so the result keeps rising with elevation.
fn height_at(env: &FlightEnv, z0: f64, speed: f64, elev: f64, d: f64) -> (f64, f64) {
    let s0 = throw_state(Vec3::new(0.0, 0.0, z0), Vec3::Y, speed, elev);
    let tr = fly(env, s0, 0.0, SOLVE_DT, &[Crossing::YAbove(d), Crossing::ZBelow(env.ball.radius)], 30.0);
    let end = tr.last();
    match tr.stop {
        Some(0) => (end.s.r.z, end.t),
        _ => (env.ball.radius - (d - end.s.r.y).max(0.0), f64::INFINITY),
    }
}

/// The flatter launch elevation that carries a throw from release height `z0` to height `zt`
/// at horizontal distance `d`, released at `speed`. `None` when no elevation up to
/// `THROW_MAX_ELEV` gets there on the fly.
pub fn solve_throw(env: &FlightEnv, z0: f64, zt: f64, d: f64, speed: f64) -> Option<ThrowSolution> {
    if !(d > 0.0 && speed > 0.0) {
        return None;
    }
    let (mut lo, mut hi) = (THROW_MIN_ELEV, THROW_MAX_ELEV);
    let (mut f_lo, _) = height_at(env, z0, speed, lo, d);
    let (mut f_hi, t_hi) = height_at(env, z0, speed, hi, d);
    f_lo -= zt;
    f_hi -= zt;
    if f_hi < 0.0 {
        return None;
    }
    if f_hi == 0.0 {
        return Some(ThrowSolution { elev: hi, t_flight: t_hi });
    }
    if f_lo >= 0.0 {
        // Even the steepest downward throw arrives high: the target is under the hand.
        let (_, t) = height_at(env, z0, speed, lo, d);
        return Some(ThrowSolution { elev: lo, t_flight: t });
    }
    // Illinois regula falsi on a bracketed, increasing function.
    let mut side = 0i8;
    for _ in 0..60 {
        let x = (lo * f_hi - hi * f_lo) / (f_hi - f_lo);
        let (fx, t) = height_at(env, z0, speed, x, d);
        let fx = fx - zt;
        if fx.abs() < SOLVE_TOL_M || hi - lo < 1e-9 {
            return Some(ThrowSolution { elev: x, t_flight: t });
        }
        if fx > 0.0 {
            hi = x;
            f_hi = fx;
            if side == 1 {
                f_lo *= 0.5;
            }
            side = 1;
        } else {
            lo = x;
            f_lo = fx;
            if side == -1 {
                f_hi *= 0.5;
            }
            side = -1;
        }
    }
    let x = 0.5 * (lo + hi);
    let (_, t) = height_at(env, z0, speed, x, d);
    Some(ThrowSolution { elev: x, t_flight: t })
}

/// Flight time of a throw over horizontal distance, tabulated by release speed: a fielder's
/// working knowledge of how long his throws take (a belief, built once for a reference
/// atmosphere; agent-models §5.6). Throws leave the hand at `z0` and arrive at `zt`.
#[derive(Clone, Debug)]
pub struct ThrowTable {
    speeds: Vec<f64>,
    dists: Vec<f64>,
    /// `t[i][j]`: speed i, distance j; infinite where the throw cannot carry on the fly.
    t: Vec<Vec<f64>>,
}

impl ThrowTable {
    /// Release speeds 20–48 m/s and distances 0–130 m.
    pub fn build(env: &FlightEnv, z0: f64, zt: f64) -> Self {
        let speeds: Vec<f64> = (0..=14).map(|i| 20.0 + 2.0 * i as f64).collect();
        let dists: Vec<f64> = (0..=26).map(|j| 5.0 * j as f64).collect();
        let t = speeds
            .iter()
            .map(|&v| dists.iter().map(|&d| if d == 0.0 { 0.0 } else { solve_throw(env, z0, zt, d, v).map(|s| s.t_flight).unwrap_or(f64::INFINITY) }).collect())
            .collect();
        Self { speeds, dists, t }
    }

    /// Believed flight time at release speed `v` over horizontal distance `d` (bilinear
    /// interpolation, clamped to the table's speeds); infinite past the throw's reach.
    pub fn time(&self, v: f64, d: f64) -> f64 {
        let d = d.max(0.0);
        let (dn, vn) = (self.dists.len(), self.speeds.len());
        if d > self.dists[dn - 1] {
            return f64::INFINITY;
        }
        let v = v.clamp(self.speeds[0], self.speeds[vn - 1]);
        let fi = (v - self.speeds[0]) / (self.speeds[1] - self.speeds[0]);
        let fj = d / (self.dists[1] - self.dists[0]);
        let (i, j) = ((fi as usize).min(vn - 2), (fj as usize).min(dn - 2));
        let (a, b) = (fi - i as f64, fj - j as f64);
        // Near the limit of reach the cell is treated as out of reach (conservative).
        let row = |i: usize| {
            let (t0, t1) = (self.t[i][j], self.t[i][j + 1]);
            if b == 0.0 { t0 } else { t0 + (t1 - t0) * b }
        };
        let (r0, r1) = (row(i), row(i + 1));
        let t = if a == 0.0 { r0 } else { r0 + (r1 - r0) * a };
        if t.is_finite() { t } else { f64::INFINITY }
    }

    /// Longest distance (m) a throw at speed `v` carries on the fly, to the table's 5 m grid.
    pub fn reach(&self, v: f64) -> f64 {
        let i = ((v.clamp(self.speeds[0], *self.speeds.last().unwrap()) - self.speeds[0]) / (self.speeds[1] - self.speeds[0])).floor() as usize;
        let row = &self.t[i.min(self.speeds.len() - 1)];
        let j = row.iter().rposition(|t| t.is_finite()).unwrap_or(0);
        self.dists[j]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vacuum() -> FlightEnv {
        FlightEnv::new(0.0)
    }

    #[test]
    fn vacuum_solve_matches_the_parabola() {
        // No air: z(x) = z0 + x·tanθ − g x² / (2 v² cos²θ). For v = 30 m/s, d = 40 m, z0 = 1.8,
        // zt = 1.3 the flatter root is θ = atan(T) with T solving
        // (g d²/2v²) T² − d T + (g d²/2v² + zt − z0) = 0.
        let (v, d, z0, zt, g) = (30.0, 40.0, 1.8, 1.3, bb_math::units::G);
        let a = g * d * d / (2.0 * v * v);
        let tan = (d - (d * d - 4.0 * a * (a + zt - z0)).sqrt()) / (2.0 * a);
        let th = tan.atan();
        let s = solve_throw(&vacuum(), z0, zt, d, v).unwrap();
        assert!((s.elev - th).abs() < 5e-4, "{} vs {}", s.elev, th);
        assert!((s.t_flight - d / (v * th.cos())).abs() < 2e-3);
    }

    #[test]
    fn drag_slows_a_long_throw_and_the_solution_hits_its_target() {
        let env = FlightEnv::default_env();
        let (v, d) = (34.0, 70.0);
        let s = solve_throw(&env, 1.8, 1.3, d, v).unwrap();
        let s_vac = solve_throw(&vacuum(), 1.8, 1.3, d, v).unwrap();
        // Drag costs a 70 m throw several tenths of a second (backspin lift means it need not
        // launch higher than in a vacuum).
        assert!(s.t_flight > s_vac.t_flight + 0.2, "{} vs {}", s.t_flight, s_vac.t_flight);
        // Fly the solved launch in an arbitrary direction: it passes the target point.
        let dir = Vec3::new(-0.6, 0.8, 0.0);
        let tr = fly_throw(&env, throw_state(Vec3::new(5.0, 10.0, 1.8), dir, v, s.elev), 0.0, 10.0);
        let (p, _) = tr.at(s.t_flight);
        let want = Vec3::new(5.0, 10.0, 1.3) + dir * d;
        assert!((p - want).norm() < 0.03, "{p:?} vs {want:?}");
    }

    #[test]
    fn throw_spin_is_backspin() {
        let s = throw_state(Vec3::ZERO, Vec3::Y, 30.0, 0.0);
        assert!(s.w.x > 0.0 && s.w.y.abs() < 1e-12 && s.w.z.abs() < 1e-12);
        // Backspin lifts: the same throw without spin drops further.
        let env = FlightEnv::default_env();
        let spun = fly_throw(&env, BallState { r: Vec3::new(0.0, 0.0, 1.8), ..s }, 0.0, 1.0).at(0.5).0.z;
        let flat = fly_throw(&env, BallState { r: Vec3::new(0.0, 0.0, 1.8), w: Vec3::ZERO, ..s }, 0.0, 1.0).at(0.5).0.z;
        assert!(spun > flat + 0.03, "{spun} vs {flat}");
    }

    #[test]
    fn out_of_reach_is_none_and_the_table_agrees_with_the_solve() {
        let env = FlightEnv::default_env();
        assert!(solve_throw(&env, 1.8, 1.3, 125.0, 26.0).is_none());
        let tab = ThrowTable::build(&env, 1.8, 1.3);
        assert!(tab.time(26.0, 125.0).is_infinite());
        for &(v, d) in &[(33.0, 37.0), (37.0, 61.0), (41.0, 83.0)] {
            let direct = solve_throw(&env, 1.8, 1.3, d, v).unwrap().t_flight;
            assert!((tab.time(v, d) - direct).abs() < 0.02, "v {v} d {d}: {} vs {direct}", tab.time(v, d));
        }
        assert!(tab.reach(30.0) < tab.reach(40.0));
    }
}
