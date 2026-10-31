//! Bat–ball contact detection and resolution (physics-spec §7, §8).

use bb_math::{Vec3, m};

use crate::ball::BallParams;
use crate::bat::{BatBallParams, BatProfile, MassProps};
use crate::collision::{Body, Material, axisymmetric_inv_inertia, resolve, sphere_inv_inertia};
use crate::flight::Trajectory;
use crate::BallState;

/// The bat's rigid-body state at one instant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BatKinematics {
    /// Knob end of the bat axis.
    pub knob: Vec3,
    /// Unit axis, knob → tip.
    pub axis: Vec3,
    /// Velocity of the bat's center of mass.
    pub v_cm: Vec3,
    /// Angular velocity.
    pub omega: Vec3,
}

impl BatKinematics {
    pub fn point(&self, u: f64) -> Vec3 {
        self.knob + self.axis * u
    }
    /// Velocity of a world point rigidly attached to the bat.
    pub fn point_velocity(&self, p: Vec3, u_cm: f64) -> Vec3 {
        self.v_cm + self.omega.cross(p - self.point(u_cm))
    }
}

/// Anything that can say where the bat is at time `t` (the swing model implements this).
pub trait BatPose {
    fn at(&self, t: f64) -> BatKinematics;
    /// Nominal contact time: the center of the search window.
    fn nominal_contact(&self) -> f64;
}

/// Signed gap between the ball surface and the bat surface (physics-spec §7 step 2), the
/// station u used, and the closest axis point.
pub fn gap(bat: &BatKinematics, profile: &BatProfile, ball_center: Vec3, ball_r: f64) -> (f64, f64, Vec3) {
    let len = profile.length();
    let u = (ball_center - bat.knob).dot(bat.axis).clamp(0.0, len);
    let p = bat.point(u);
    let d = (ball_center - p).norm() - profile.radius(u) - ball_r;
    (d, u, p)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContactPoint {
    pub t: f64,
    /// Axial station of the contact, m from the knob.
    pub u: f64,
    /// Unit normal from the bat into the ball.
    pub n: Vec3,
    pub ball: BallState,
    pub bat: BatKinematics,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClosestApproach {
    pub t: f64,
    pub u: f64,
    /// Minimum surface gap, m (≥ 0 for a miss).
    pub gap: f64,
    /// Ball center minus closest bat-axis point, at closest approach.
    pub offset: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Detection {
    Contact(ContactPoint),
    Miss(ClosestApproach),
}

/// Search window half-width and sampling step (physics-spec §7).
pub const WINDOW_S: f64 = 0.025;
pub const SAMPLE_S: f64 = 0.0001;
/// Upper bound on how fast the ball–bat surface gap can close, m/s (ball ≤ 50 m/s plus any
/// bat point ≤ 60 m/s, with margin).
const V_CLOSE_MAX: f64 = 150.0;
const BISECT_TOL_S: f64 = 1e-7;

/// Find the first bat–ball contact on the ball's trajectory within the window around the
/// swing's nominal contact time.
pub fn detect(tr: &Trajectory, swing: &dyn BatPose, profile: &BatProfile, ball: &BallParams) -> Detection {
    let t0 = (swing.nominal_contact() - WINDOW_S).max(tr.first().t);
    let t1 = (swing.nominal_contact() + WINDOW_S).min(tr.last().t);
    let g = |t: f64| {
        let (b, _) = tr.at(t);
        gap(&swing.at(t), profile, b, ball.radius)
    };
    let mut best = ClosestApproach { t: t0, u: 0.0, gap: f64::INFINITY, offset: Vec3::ZERO };
    // Adaptive scan: the gap cannot shrink faster than the closing-speed bound, so a step of
    // gap / V_CLOSE_MAX can never jump over a contact. Near the bat the step is SAMPLE_S.
    let mut t = t0;
    let mut prev: Option<(f64, f64)> = None;
    loop {
        let (d, u, p) = g(t);
        if d < best.gap {
            best = ClosestApproach { t, u, gap: d, offset: tr.at(t).0 - p };
        }
        if let Some((tp, dp)) = prev {
            if dp > 0.0 && d <= 0.0 {
                // Bisect for the touch time.
                let (mut lo, mut hi) = (tp, t);
                while hi - lo > BISECT_TOL_S {
                    let mid = 0.5 * (lo + hi);
                    if g(mid).0 > 0.0 { lo = mid } else { hi = mid }
                }
                return Detection::Contact(contact_at(tr, swing, profile, ball, hi));
            }
        }
        prev = Some((t, d));
        if t >= t1 {
            break;
        }
        t = (t + (d / V_CLOSE_MAX).max(SAMPLE_S)).min(t1);
    }
    if best.gap <= 0.0 {
        // Overlapping already at the window start: treat the start as the contact.
        return Detection::Contact(contact_at(tr, swing, profile, ball, t0));
    }
    Detection::Miss(best)
}

fn contact_at(tr: &Trajectory, swing: &dyn BatPose, profile: &BatProfile, ball: &BallParams, t: f64) -> ContactPoint {
    let bat = swing.at(t);
    let (r, v) = tr.at(t);
    // Spin is piecewise constant between knots; take it from the knot at or before t.
    let i = tr.knots.partition_point(|k| k.t <= t).saturating_sub(1);
    let w = tr.knots[i].s.w;
    let (_, u, p) = gap(&bat, profile, r, ball.radius);
    let radial = (r - p).reject(bat.axis).normalized();
    // Tilt by the profile slope so the normal is perpendicular to the tapered surface.
    let n = (radial - bat.axis * profile.slope(u)).normalized();
    ContactPoint { t, u, n, ball: BallState { r, v, w }, bat }
}

/// Outcome of resolving a contact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContactResult {
    pub point: ContactPoint,
    pub ball_after: BallState,
    pub e_n: f64,
    /// Speed of the bat's sweet spot at contact (Statcast "bat speed").
    pub bat_speed: f64,
    /// Elevation of the sweet-spot velocity (attack angle), rad.
    pub attack_angle: f64,
    pub impulses: u32,
}

/// Apply the impulse model (with at most one extra impulse if still approaching; §7 step 7).
pub fn resolve_contact(
    cp: &ContactPoint,
    profile: &BatProfile,
    mp: &MassProps,
    params: &BatBallParams,
    ball: &BallParams,
) -> ContactResult {
    let len = profile.length();
    let c_bat = cp.bat.point(mp.u_cm);
    let contact_pt = cp.ball.r - cp.n * ball.radius;
    let mut b1 = Body {
        inv_mass: 1.0 / ball.mass,
        inv_inertia: sphere_inv_inertia(ball.inertia()),
        v: cp.ball.v,
        w: cp.ball.w,
        r: cp.n * -ball.radius,
    };
    let mut b2 = Body {
        inv_mass: 1.0 / mp.mass,
        inv_inertia: axisymmetric_inv_inertia(mp.i_cm, mp.i_ax, cp.bat.axis),
        v: cp.bat.v_cm,
        w: cp.bat.omega,
        r: contact_pt - c_bat,
    };
    let rel = b1.point_velocity() - b2.point_velocity();
    let e_n = params.e_n(cp.u, len, rel.dot(cp.n).abs());
    let mat = Material { e_n, e_t: params.e_t, mu: params.mu };
    let mut impulses = 0;
    for _ in 0..2 {
        match resolve(b1, b2, cp.n, mat) {
            Ok(out) => {
                b1 = out.b1;
                b2 = out.b2;
                impulses += 1;
            }
            Err(_) => break,
        }
    }
    let uss = len - params.ss_from_tip;
    let vss = cp.bat.point_velocity(cp.bat.point(uss), mp.u_cm);
    ContactResult {
        point: *cp,
        ball_after: BallState { r: cp.ball.r, v: b1.v, w: b1.w },
        e_n,
        bat_speed: vss.norm(),
        attack_angle: m::atan2(vss.z, m::hypot(vss.x, vss.y)),
        impulses,
    }
}

/// Statcast-style batted-ball angles (physics-spec §12): (exit speed, launch angle,
/// spray angle), angles in rad; spray is positive toward +X (first-base side).
pub fn launch_angles(v: Vec3) -> (f64, f64, f64) {
    (v.norm(), m::atan2(v.z, m::hypot(v.x, v.y)), m::atan2(v.x, v.y))
}
