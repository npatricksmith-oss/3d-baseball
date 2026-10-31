//! Pitch release and the aiming solve (agent-models §2.2–§2.3).
//!
//! These are pure functions: the pitcher's *intent* is solved exactly here (pitchers know
//! their own pitches); execution noise is applied by `bb_agents::motor` before the world
//! receives the release state.

use bb_math::{Vec3, m};

use crate::flight::{Crossing, DT_PITCH, FlightEnv, Trajectory, fly};
use crate::spin::spin_from_axis;
use crate::zone::Y_JUDGE;
use crate::BallState;

/// Front edge of the pitching rubber, m from the rear apex of home (physics-spec §11).
pub const RUBBER_Y: f64 = 18.440;
/// Front-of-mound height at the landing foot [E] (agent-models §2.2).
pub const H_FRONT: f64 = 0.10;

/// Release position from body and delivery attributes (agent-models §2.2).
/// `arm_side` is −1 for right-handers (arm side toward third base, −X), +1 for left-handers.
pub fn release_point(height: f64, arm_slot_rad: f64, extension: f64, arm_side: f64) -> Vec3 {
    let l = 0.40 * height;
    Vec3::new(
        arm_side * (0.10 * height + l * m::cos(arm_slot_rad)),
        RUBBER_Y - extension,
        H_FRONT + 0.65 * height + l * m::sin(arm_slot_rad),
    )
}

/// The physical definition of one thrown pitch, apart from its direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchShape {
    /// Release speed, m/s.
    pub speed: f64,
    /// Total spin rate, rad/s.
    pub spin_rate: f64,
    /// Savant spin axis, degrees (180 = pure backspin).
    pub spin_axis_deg: f64,
    /// Transverse fraction of spin, 0–1.
    pub efficiency: f64,
    /// Direction of the gyro component along v̂, ±1.
    pub gyro_sign: f64,
}

/// Clock-face tilt (pitcher's perspective, 12:00 = backspin) to Savant spin axis, degrees.
pub fn tilt_to_axis_deg(tilt_hours: f64) -> f64 {
    let a = (180.0 + 30.0 * tilt_hours) % 360.0;
    if a < 0.0 { a + 360.0 } else { a }
}

/// Release direction: `yaw` rotates from −Y toward +X, `elev` is above horizontal.
pub fn direction(yaw: f64, elev: f64) -> Vec3 {
    let ce = m::cos(elev);
    Vec3::new(m::sin(yaw) * ce, -m::cos(yaw) * ce, m::sin(elev))
}

/// The release state for a shape thrown from `r` in direction (yaw, elev).
pub fn launch(r: Vec3, shape: &PitchShape, yaw: f64, elev: f64) -> BallState {
    let v = direction(yaw, elev) * shape.speed;
    let w = spin_from_axis(shape.spin_axis_deg, shape.efficiency, shape.spin_rate, shape.gyro_sign, v);
    BallState { r, v, w }
}

/// Fly a pitch from release to the judgment plane, or to the ground if it bounces first.
/// `stop == Some(0)` means it reached the plate plane; `Some(1)` means it hit the ground.
pub fn fly_pitch(env: &FlightEnv, s: BallState) -> Trajectory {
    let stops = [Crossing::YBelow(Y_JUDGE), Crossing::ZBelow(env.ball.radius)];
    fly(env, s, 0.0, DT_PITCH, &stops, 2.0)
}

/// Where a pitch crosses the plate plane (x, z), flying through the ground if needed.
/// Used by the aiming solve, which must converge even for pitches aimed in the dirt.
fn plate_xz(env: &FlightEnv, s: BallState) -> Option<(f64, f64, f64)> {
    let tr = fly(env, s, 0.0, DT_PITCH, &[Crossing::YBelow(Y_JUDGE)], 2.0);
    tr.stop.map(|_| {
        let k = tr.last();
        (k.s.r.x, k.s.r.z, k.t)
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aim {
    pub yaw: f64,
    pub elev: f64,
    /// Residual miss at the plate plane after the last iteration, m.
    pub residual: f64,
    pub iterations: u32,
    /// Flight time to the plate plane, s.
    pub flight_time: f64,
}

/// Solve release (yaw, elev) so the noise-free pitch crosses the plate plane at
/// (target_x, target_z). Fixed-point iteration with the small-angle Jacobian 1/D
/// (agent-models §2.3); `guess` seeds it (e.g. the previous solve for this pitch type).
pub fn aim(env: &FlightEnv, r: Vec3, shape: &PitchShape, target_x: f64, target_z: f64, guess: Option<(f64, f64)>) -> Aim {
    // Straight-line guess, plus a gravity allowance: drop ≈ ½ g t² over t ≈ D / v.
    let d0 = Vec3::new(target_x - r.x, Y_JUDGE - r.y, target_z - r.z);
    let (mut yaw, mut elev) = guess.unwrap_or_else(|| {
        let t = d0.norm() / shape.speed;
        let lift = 0.5 * bb_math::units::G * t * t;
        (m::atan2(d0.x, -d0.y), m::atan2(d0.z + lift, m::hypot(d0.x, d0.y)))
    });
    let mut out = Aim { yaw, elev, residual: f64::INFINITY, iterations: 0, flight_time: 0.0 };
    for it in 1..=12 {
        let Some((x, z, t)) = plate_xz(env, launch(r, shape, yaw, elev)) else { break };
        let (dx, dz) = (x - target_x, z - target_z);
        let res = m::hypot(dx, dz);
        out = Aim { yaw, elev, residual: res, iterations: it, flight_time: t };
        if res < 1e-4 {
            break;
        }
        let dist = m::hypot(m::hypot(x - r.x, Y_JUDGE - r.y), z - r.z);
        yaw -= dx / dist;
        elev -= dz / dist;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use bb_math::units::{MPH, RPM, DEG};

    #[test]
    fn release_point_check() {
        // agent-models §2.2: H 1.88, slot 45°, extension 1.95 → (−0.72, 16.49, 1.85).
        let r = release_point(1.88, 45.0 * DEG, 1.95, -1.0);
        assert!((r.x + 0.72).abs() < 0.01, "{r:?}");
        assert!((r.y - 16.49).abs() < 0.01);
        assert!((r.z - 1.85).abs() < 0.01);
    }

    #[test]
    fn tilt_conversions() {
        assert_eq!(tilt_to_axis_deg(12.0), 180.0);
        assert_eq!(tilt_to_axis_deg(1.0), 210.0);
        assert_eq!(tilt_to_axis_deg(6.0), 0.0);
        assert_eq!(tilt_to_axis_deg(11.0), 150.0);
    }

    #[test]
    fn aim_converges_for_varied_pitches() {
        let env = FlightEnv::default_env();
        let r = release_point(1.88, 45.0 * DEG, 1.95, -1.0);
        let shapes = [
            PitchShape { speed: 95.0 * MPH, spin_rate: 2300.0 * RPM, spin_axis_deg: 210.0, efficiency: 0.95, gyro_sign: 1.0 },
            PitchShape { speed: 86.0 * MPH, spin_rate: 2500.0 * RPM, spin_axis_deg: 90.0, efficiency: 0.35, gyro_sign: 1.0 },
            PitchShape { speed: 79.0 * MPH, spin_rate: 2700.0 * RPM, spin_axis_deg: 30.0, efficiency: 0.85, gyro_sign: -1.0 },
            PitchShape { speed: 84.0 * MPH, spin_rate: 1750.0 * RPM, spin_axis_deg: 235.0, efficiency: 0.9, gyro_sign: 1.0 },
        ];
        for s in &shapes {
            for &(tx, tz) in &[(0.0, 0.75), (-0.25, 0.45), (0.3, 1.1), (0.0, 0.05)] {
                let a = aim(&env, r, s, tx, tz, None);
                assert!(a.residual < 1e-4, "{s:?} → ({tx},{tz}): residual {} after {}", a.residual, a.iterations);
                assert!(a.iterations <= 6, "{} iterations", a.iterations);
            }
        }
    }
}

/// Pitch movement (physics-spec §12): displacement at the judgment plane between the real
/// pitch and its no-Magnus twin (same release state; drag and gravity only), over the full
/// flight. `(horizontal, vertical)` in m, catcher's view (+x toward first base).
pub fn movement(env: &FlightEnv, s: BallState) -> Option<(f64, f64)> {
    let stops = [Crossing::YBelow(Y_JUDGE)];
    let real = fly(env, s, 0.0, DT_PITCH, &stops, 2.0);
    let twin_env = FlightEnv { aero: env.aero.no_magnus(), ..*env };
    let twin = fly(&twin_env, s, 0.0, DT_PITCH, &stops, 2.0);
    if real.stop.is_none() || twin.stop.is_none() {
        return None;
    }
    let d = real.last().s.r - twin.last().s.r;
    Some((d.x, d.z))
}

/// Savant-comparable movement variant: the twin is launched from the real pitch's state at
/// y = 40 ft and both are compared at the front of the plate (the PITCHf/x-era window).
/// Used only for sanity comparisons with public data (R8); not a primary metric.
pub fn movement_40ft(env: &FlightEnv, s: BallState) -> Option<(f64, f64)> {
    use crate::zone::Y_FRONT;
    let y40 = 40.0 * bb_math::units::FT;
    let to40 = fly(env, s, 0.0, DT_PITCH, &[Crossing::YBelow(y40)], 2.0);
    to40.stop?;
    let s40 = to40.last().s;
    let stops = [Crossing::YBelow(Y_FRONT)];
    let real = fly(env, s40, 0.0, DT_PITCH, &stops, 2.0);
    let twin = fly(&FlightEnv { aero: env.aero.no_magnus(), ..*env }, s40, 0.0, DT_PITCH, &stops, 2.0);
    let d = real.last().s.r - twin.last().s.r;
    Some((d.x, d.z))
}
