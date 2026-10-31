//! Resolve one pitch physically: flight, the swing's bat against the ball, and the batted
//! ball's flight (physics-spec §5, §7, §8, §12). No outcome is ever rolled (D1).

use bb_math::Vec3;
use bb_physics::bat::{BatBallParams, BatProfile, MassProps};
use bb_physics::contact::{BatPose, ClosestApproach, ContactResult, Detection, detect, launch_angles, resolve_contact};
use bb_physics::flight::{Crossing, DT_BATTED, DT_PITCH, Trajectory, fly};
use bb_physics::zone::Y_JUDGE;
use bb_physics::{BallState, FlightEnv};

/// The pitch keeps flying past the plate (late contact, the catcher) until this y.
pub const Y_PITCH_END: f64 = -1.2;

/// The bat in use: geometry, integrated mass properties, and contact parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct BatModel {
    pub profile: BatProfile,
    pub mass: MassProps,
    pub contact: BatBallParams,
}

impl BatModel {
    pub fn default_34in() -> Self {
        let profile = BatProfile::default_34in();
        let mass = profile.mass_props();
        Self { profile, mass, contact: BatBallParams::default() }
    }
}

/// Where and when the pitch crossed the judgment plane, if it got there in flight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlateCrossing {
    pub t: f64,
    pub r: Vec3,
    pub v: Vec3,
}

/// Statcast-style batted-ball facts (physics-spec §12).
#[derive(Clone, Debug)]
pub struct BattedBall {
    pub contact: ContactResult,
    pub exit_speed: f64,
    pub launch_angle: f64,
    pub spray_angle: f64,
    /// Unobstructed flight to z = 0.
    pub flight: Trajectory,
    pub projected_distance: f64,
    pub hang_time: f64,
    pub apex: f64,
}

#[derive(Clone, Debug)]
pub enum SwingResult {
    NoSwing,
    Miss(ClosestApproach),
    Contact(Box<BattedBall>),
}

#[derive(Clone, Debug)]
pub struct PitchResolution {
    /// Pitch flight from release until contact, the ground, or `Y_PITCH_END`.
    pub pitch: Trajectory,
    pub crossing: Option<PlateCrossing>,
    /// The pitch reached the ground before the plate plane.
    pub bounced: bool,
    pub swing: SwingResult,
}

/// Resolve a released pitch against an optional swing.
pub fn resolve_pitch(env: &FlightEnv, release: BallState, swing: Option<&dyn BatPose>, bat: &BatModel) -> PitchResolution {
    let stops = [Crossing::YBelow(Y_PITCH_END), Crossing::ZBelow(env.ball.radius)];
    let mut pitch = fly(env, release, 0.0, DT_PITCH, &stops, 3.0);
    let crossing = pitch.find(Crossing::YBelow(Y_JUDGE)).map(|t| {
        let (r, v) = pitch.at(t);
        PlateCrossing { t, r, v }
    });
    let bounced = crossing.is_none() && pitch.stop == Some(1);
    let swing_res = match swing {
        None => SwingResult::NoSwing,
        Some(s) => match detect(&pitch, s, &bat.profile, &env.ball) {
            Detection::Miss(c) => SwingResult::Miss(c),
            Detection::Contact(cp) => {
                let res = resolve_contact(&cp, &bat.profile, &bat.mass, &bat.contact, &env.ball);
                // Truncate the pitch at contact.
                pitch.knots.retain(|k| k.t < cp.t);
                pitch.knots.push(bb_physics::Knot { t: cp.t, s: cp.ball });
                pitch.stop = None;
                SwingResult::Contact(Box::new(batted_ball(env, res)))
            }
        },
    };
    PitchResolution { pitch, crossing, bounced, swing: swing_res }
}

fn batted_ball(env: &FlightEnv, contact: ContactResult) -> BattedBall {
    let s = contact.ball_after;
    let (exit_speed, launch_angle, spray_angle) = launch_angles(s.v);
    let flight = fly(env, s, contact.point.t, DT_BATTED, &[Crossing::ZBelow(0.0)], 15.0);
    let end = flight.last();
    BattedBall {
        exit_speed,
        launch_angle,
        spray_angle,
        projected_distance: bb_math::m::hypot(end.s.r.x, end.s.r.y),
        hang_time: end.t - contact.point.t,
        apex: flight.apex_z,
        flight,
        contact,
    }
}
