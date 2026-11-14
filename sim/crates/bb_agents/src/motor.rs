//! Execution: intended motor commands plus execution noise (agent-models §1.1, §1.3, §2.3).
//!
//! `Executed*` values go only to `bb_game`, which hands them to the world. Decision code must
//! never read them; an agent learns its own error only by observing the result.

use bb_math::KeyedRng;
use bb_physics::BallState;
use bb_physics::pitch::{PitchShape, launch};

use crate::pitcher::IntendedRelease;

/// Truncation of all execution and perception noise, in σ (agent-models §1.3).
pub const TRUNC_SIGMA: f64 = 4.0;

/// Execution noise scales for a pitch release (agent-models §2.3). All [E] except `angle`,
/// which comes from `Command`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReleaseNoise {
    /// Release-direction σ per axis (yaw and elevation), rad: σ_cmd·command_mult·factors.
    pub angle: f64,
    /// Release-speed σ, m/s.
    pub speed: f64,
    /// Relative spin-rate σ.
    pub spin_rate_rel: f64,
    /// Spin-axis σ, degrees.
    pub spin_axis_deg: f64,
    /// Release-point σ in x and z, m.
    pub point: f64,
}

impl ReleaseNoise {
    /// Noise for a pitcher with base command σ (rad) on a pitch, with the combined
    /// global/fatigue/composure factor `k` (1 = none).
    pub fn new(command: f64, command_mult: f64, k: f64) -> Self {
        Self { angle: command * command_mult * k, speed: 0.45, spin_rate_rel: 0.02, spin_axis_deg: 3.0, point: 0.02 }
    }
}

/// The physical release handed to the world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExecutedRelease {
    pub state: BallState,
    pub shape: PitchShape,
    pub yaw: f64,
    pub elev: f64,
}

/// Apply release noise. Draw order is fixed (yaw, elev, speed, spin rate, spin axis,
/// point x, point z) and must never change: it is part of the replay contract.
pub fn execute_release(intended: &IntendedRelease, noise: &ReleaseNoise, rng: &mut KeyedRng) -> ExecutedRelease {
    let mut z = || rng.truncated_normal(TRUNC_SIGMA);
    let yaw = intended.yaw + noise.angle * z();
    let elev = intended.elev + noise.angle * z();
    let mut shape = intended.shape;
    shape.speed += noise.speed * z();
    shape.spin_rate *= 1.0 + noise.spin_rate_rel * z();
    shape.spin_axis_deg += noise.spin_axis_deg * z();
    let mut r = intended.r;
    r.x += noise.point * z();
    r.z += noise.point * z();
    ExecutedRelease { state: launch(r, &shape, yaw, elev), shape, yaw, elev }
}

use crate::swing::{BatterAttrs, IntendedSwing, Placement, Swing, apply_placement};

/// Swing execution noise σ (agent-models §3.6, §10).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwingNoise {
    pub v_offset: f64,
    /// Plane tilt σ per meter of δ_v σ (fixed ratio [E]): rad/m.
    pub tilt_per_m: f64,
    pub reach: f64,
    pub timing: f64,
}

impl SwingNoise {
    /// From the batter's attributes with combined factor `k` (global × composure).
    pub fn new(b: &BatterAttrs, k: f64) -> Self {
        Self { v_offset: b.bat_control * k, tilt_per_m: 1.5, reach: 0.7 * b.bat_control * k, timing: b.timing * k }
    }
}

/// The executed swing handed to the world. Draw order (δ_v, α, δ_r, t) is part of the
/// replay contract.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExecutedSwing {
    pub swing: Swing,
    pub placement: Placement,
}

pub fn execute_swing(intended: &IntendedSwing, noise: &SwingNoise, rng: &mut KeyedRng) -> ExecutedSwing {
    let mut z = || rng.truncated_normal(TRUNC_SIGMA);
    let v_offset = noise.v_offset * z();
    let plane_tilt = noise.v_offset * noise.tilt_per_m * z();
    let reach = noise.reach * z();
    let t_shift = noise.timing * z();
    let placement = Placement { plane_tilt, v_offset, reach, t_shift };
    ExecutedSwing { swing: apply_placement(intended, &placement), placement }
}

/// Execution noise of a throw (agent-models §5.5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThrowNoise {
    /// Direction σ about the vertical, rad: `ArmAccuracy` times the hurry and balance factors.
    pub direction: f64,
    /// Relative release-speed σ [E].
    pub speed_rel: f64,
}

impl ThrowNoise {
    pub const SPEED_REL: f64 = 0.03;
    pub fn new(direction: f64) -> Self {
        Self { direction, speed_rel: Self::SPEED_REL }
    }
}

/// The throw as it left the hand: release speed, and the error of its horizontal direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExecutedThrow {
    pub speed: f64,
    pub yaw_err: f64,
}

/// Apply throw noise to an intended release speed. Draw order (speed, direction) is part of
/// the replay contract.
pub fn execute_throw(speed: f64, noise: &ThrowNoise, rng: &mut KeyedRng) -> ExecutedThrow {
    let speed = speed * (1.0 + noise.speed_rel * rng.truncated_normal(TRUNC_SIGMA));
    let yaw_err = noise.direction * rng.truncated_normal(TRUNC_SIGMA);
    ExecutedThrow { speed, yaw_err }
}
