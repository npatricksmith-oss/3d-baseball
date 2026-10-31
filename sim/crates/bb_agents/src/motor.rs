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
