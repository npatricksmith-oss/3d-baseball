//! Field surfaces, fence, bounces, and rolling (physics-spec §9, §11). M4 skeleton: flat
//! ground, two surfaces (infield dirt, grass), a fence described by three distances.

use bb_math::units::{FT, G};
use bb_math::{Vec3, m};

use crate::BallState;
use crate::collision::{Body, Material, resolve, sphere_inv_inertia};
use crate::flight::{Crossing, DT_BATTED, FlightEnv, Trajectory, fly};

/// Surface parameters (physics-spec §9.1, all [E]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Surface {
    pub e_ref: f64,
    /// e_n = e_ref / (1 + k_s·|c_n|).
    pub k_s: f64,
    pub mu: f64,
    /// Rolling resistance (deceleration μ_r·g).
    pub mu_r: f64,
    /// σ of the bounce-normal tilt, rad.
    pub roughness: f64,
}

// Rolling resistance [E, M4]: physics-spec §9.1's 0.09/0.12 let a 25 m/s grounder roll
// ~250 m; 0.22/0.28 gives ~110 m, closer to observed roll-outs (calibrate in M5, R4).
pub const DIRT: Surface = Surface { e_ref: 0.50, k_s: 0.03, mu: 0.55, mu_r: 0.22, roughness: 0.026 };
pub const GRASS: Surface = Surface { e_ref: 0.45, k_s: 0.03, mu: 0.45, mu_r: 0.28, roughness: 0.010 };

/// A simple park: fence distances down the lines and to center, wall height, infield dirt.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Field {
    pub fence_lf: f64,
    pub fence_cf: f64,
    pub fence_rf: f64,
    pub wall_height: f64,
    /// Infield dirt extends this far from home, m.
    pub dirt_radius: f64,
    /// Wall padding restitution [E] (physics-spec §9.4).
    pub wall_e: f64,
}

impl Default for Field {
    /// A symmetric reference park: 330 / 400 / 330 ft, 8-ft wall.
    fn default() -> Self {
        Self { fence_lf: 330.0 * FT, fence_cf: 400.0 * FT, fence_rf: 330.0 * FT, wall_height: 8.0 * FT, dirt_radius: 95.0 * FT, wall_e: 0.35 }
    }
}

impl Field {
    /// Fence distance at spray angle θ (rad from +Y, + toward first base); quadratic through
    /// the three distances, extended into foul ground at the line value.
    pub fn fence_at(&self, theta: f64) -> f64 {
        let s = (theta / core::f64::consts::FRAC_PI_4).clamp(-1.0, 1.0);
        let side = if s < 0.0 { self.fence_lf } else { self.fence_rf };
        self.fence_cf + (side - self.fence_cf) * s * s
    }
    pub fn surface_at(&self, x: f64, y: f64) -> Surface {
        if m::hypot(x, y) < self.dirt_radius { DIRT } else { GRASS }
    }
    pub fn fair(x: f64, y: f64) -> bool {
        y >= x.abs()
    }
}

/// What ended a flight segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SegmentEnd {
    Ground,
    /// Cleared the fence in flight (`fair` decides home run vs foul out of play).
    OverFence { fair: bool },
    /// Hit the wall below its top.
    Wall,
    Rest,
    Timeout,
}

/// One phase of a batted ball's motion.
#[derive(Clone, Debug)]
pub enum Segment {
    Flight { tr: Trajectory, end: SegmentEnd },
    /// Rolling on the ground from `r0` with velocity `v0` (horizontal) at `t0`, decelerating
    /// at `decel` until rest at `t_stop`.
    Roll { t0: f64, r0: Vec3, v0: Vec3, decel: f64, t_stop: f64 },
}

/// The world's full path of a batted ball until it rests, leaves the field, or times out.
#[derive(Clone, Debug)]
pub struct BallPath {
    pub segments: Vec<Segment>,
    /// Ground touches (time, x, y) in order.
    pub bounces: Vec<(f64, f64, f64)>,
    /// Wall touches (time, x, y) in order.
    pub walls: Vec<(f64, f64, f64)>,
}

impl BallPath {
    pub fn t_end(&self) -> f64 {
        match self.segments.last() {
            Some(Segment::Flight { tr, .. }) => tr.last().t,
            Some(Segment::Roll { t_stop, .. }) => *t_stop,
            None => 0.0,
        }
    }
    pub fn t_start(&self) -> f64 {
        match self.segments.first() {
            Some(Segment::Flight { tr, .. }) => tr.first().t,
            Some(Segment::Roll { t0, .. }) => *t0,
            None => 0.0,
        }
    }
    /// Ball position and velocity at time `t` (clamped to the path).
    pub fn at(&self, t: f64) -> (Vec3, Vec3) {
        for seg in &self.segments {
            match seg {
                Segment::Flight { tr, .. } => {
                    if t <= tr.last().t {
                        return tr.at(t);
                    }
                }
                Segment::Roll { t0, r0, v0, decel, t_stop } => {
                    if t <= *t_stop {
                        let dt = (t - t0).max(0.0);
                        let sp = v0.norm();
                        let dir = v0.normalized();
                        let s = sp * dt - 0.5 * decel * dt * dt;
                        return (*r0 + dir * s, dir * (sp - decel * dt).max(0.0));
                    }
                }
            }
        }
        match self.segments.last() {
            Some(Segment::Flight { tr, .. }) => (tr.last().s.r, tr.last().s.v),
            Some(Segment::Roll { t0, r0, v0, decel, t_stop }) => {
                let dt = t_stop - t0;
                let sp = v0.norm();
                (*r0 + v0.normalized() * (sp * dt - 0.5 * decel * dt * dt), Vec3::ZERO)
            }
            None => (Vec3::ZERO, Vec3::ZERO),
        }
    }
    pub fn first_bounce(&self) -> Option<(f64, f64, f64)> {
        self.bounces.first().copied()
    }
    pub fn end(&self) -> SegmentEnd {
        match self.segments.last() {
            Some(Segment::Flight { end, .. }) => *end,
            Some(Segment::Roll { .. }) => SegmentEnd::Rest,
            None => SegmentEnd::Timeout,
        }
    }
}

/// Bounce the ball off flat ground with the surface model; `tilt` is the roughness
/// perturbation of the normal (radians about x and y), drawn by the caller from the keyed
/// RNG (physics-spec §9.3, the only world-side randomness).
pub fn bounce(env: &FlightEnv, s: BallState, surf: &Surface, tilt: (f64, f64)) -> BallState {
    let n = Vec3::new(m::sin(tilt.0), m::sin(tilt.1), 1.0).normalized();
    let cn = s.v.dot(n).abs();
    let e_n = surf.e_ref / (1.0 + surf.k_s * cn);
    let ball = Body {
        inv_mass: 1.0 / env.ball.mass,
        inv_inertia: sphere_inv_inertia(env.ball.inertia()),
        v: s.v,
        w: s.w,
        r: n * -env.ball.radius,
    };
    match resolve(ball, Body::immovable(Vec3::ZERO), n, Material { e_n, e_t: 0.0, mu: surf.mu }) {
        Ok(out) => BallState { r: Vec3::new(s.r.x, s.r.y, env.ball.radius), v: out.b1.v, w: out.b1.w },
        Err(_) => s,
    }
}

/// Vertical speed below which a bounce turns into rolling, m/s [E] (physics-spec §9.2).
pub const ROLL_VZ: f64 = 0.6;
/// Below this speed a rolling ball is at rest, m/s.
pub const REST_SPEED: f64 = 0.05;

/// Fly a fair/foul batted ball over the field: flights, bounces, wall caroms, and the final
/// roll. `tilts` supplies one roughness tilt per bounce (index = bounce number).
pub fn batted_path(env: &FlightEnv, field: &Field, s0: BallState, t0: f64, mut tilts: impl FnMut(u32, &Surface) -> (f64, f64)) -> BallPath {
    let mut segs = Vec::new();
    let mut bounces = Vec::new();
    let mut walls = Vec::new();
    let mut s = s0;
    let mut t = t0;
    let r = env.ball.radius;
    for k in 0..40u32 {
        let fence = Crossing::RadiusAbove { lf: field.fence_lf, cf: field.fence_cf, rf: field.fence_rf };
        let tr = fly(env, s, t, DT_BATTED, &[Crossing::ZBelow(r), fence], 15.0);
        let last = *tr.last();
        match tr.stop {
            Some(0) => {
                bounces.push((last.t, last.s.r.x, last.s.r.y));
                let surf = field.surface_at(last.s.r.x, last.s.r.y);
                segs.push(Segment::Flight { tr, end: SegmentEnd::Ground });
                let nb = bounce(env, last.s, &surf, tilts(k, &surf));
                t = last.t;
                if nb.v.z < ROLL_VZ {
                    // Roll (physics-spec §9.2): constrained to the surface, decelerating at μ_r·g.
                    let vh = Vec3::new(nb.v.x, nb.v.y, 0.0);
                    let decel = surf.mu_r * G;
                    let t_stop = t + vh.norm() / decel;
                    // A rolling ball stops at the wall (skeleton: no carom off the base of the wall).
                    let r0 = Vec3::new(nb.r.x, nb.r.y, r);
                    let dir = vh.normalized();
                    let sp = vh.norm();
                    let (mut lo, mut hi) = (0.0, t_stop - t);
                    let pos = |dt: f64| r0 + dir * (sp * dt - 0.5 * decel * dt * dt);
                    let beyond = |p: Vec3| m::hypot(p.x, p.y) > field.fence_at(m::atan2(p.x, p.y));
                    if beyond(pos(hi)) {
                        for _ in 0..40 {
                            let mid = 0.5 * (lo + hi);
                            if beyond(pos(mid)) { hi = mid } else { lo = mid }
                        }
                        let p = pos(lo);
                        let v_wall = (sp - decel * lo).max(0.0);
                        // Treat the stop as abrupt: model the roll as decelerating to rest there.
                        let d = (p - r0).norm();
                        let decel2 = if d > 0.0 { sp * sp / (2.0 * d) } else { decel };
                        let _ = v_wall;
                        segs.push(Segment::Roll { t0: t, r0, v0: vh, decel: decel2, t_stop: t + sp / decel2 });
                        return BallPath { segments: segs, bounces, walls };
                    }
                    segs.push(Segment::Roll { t0: t, r0, v0: vh, decel, t_stop });
                    return BallPath { segments: segs, bounces, walls };
                }
                s = nb;
            }
            Some(1) => {
                let p = last.s.r;
                if p.z >= field.wall_height + r {
                    segs.push(Segment::Flight { tr, end: SegmentEnd::OverFence { fair: Field::fair(p.x, p.y) } });
                    return BallPath { segments: segs, bounces, walls };
                }
                // Wall carom: reverse the radial velocity with the padding COR.
                walls.push((last.t, p.x, p.y));
                segs.push(Segment::Flight { tr, end: SegmentEnd::Wall });
                let radial = Vec3::new(p.x, p.y, 0.0).normalized();
                let vr = last.s.v.dot(radial);
                let v = last.s.v - radial * ((1.0 + field.wall_e) * vr);
                // Step the ball back inside the fence so it does not re-trigger.
                s = BallState { r: p - radial * 0.01, v, w: last.s.w * 0.5 };
                t = last.t;
            }
            _ => {
                segs.push(Segment::Flight { tr, end: SegmentEnd::Timeout });
                return BallPath { segments: segs, bounces, walls };
            }
        }
    }
    BallPath { segments: segs, bounces, walls }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bb_math::units::{DEG, MPH, RPM};

    fn launch(ev_mph: f64, la_deg: f64, spray_deg: f64, rpm: f64) -> BallState {
        let v = ev_mph * MPH;
        let (la, sp) = (la_deg * DEG, spray_deg * DEG);
        let dir = Vec3::new(m::sin(sp) * m::cos(la), m::cos(sp) * m::cos(la), m::sin(la));
        let w = Vec3::new(m::cos(sp), -m::sin(sp), 0.0) * (rpm * RPM); // backspin about the horizontal ⟂
        BallState { r: Vec3::new(0.0, 0.5, 0.9), v: dir * v, w }
    }

    #[test]
    fn a13_rolling_stops_at_predicted_distance() {
        let env = FlightEnv::default_env();
        let p = BallPath {
            segments: vec![Segment::Roll { t0: 1.0, r0: Vec3::new(0.0, 10.0, 0.037), v0: Vec3::new(0.0, 5.0, 0.0), decel: 0.09 * G, t_stop: 1.0 + 5.0 / (0.09 * G) }],
            bounces: vec![],
            walls: vec![],
        };
        let (r_end, v_end) = p.at(100.0);
        let d = 25.0 / (2.0 * 0.09 * G);
        assert!((r_end.y - 10.0 - d).abs() < 1e-9 && v_end.norm() == 0.0);
        let _ = env;
    }

    #[test]
    fn bounce_dissipates_and_reverses_vertical() {
        let env = FlightEnv::default_env();
        let s = BallState { r: Vec3::new(0.0, 20.0, 0.037), v: Vec3::new(0.0, 30.0, -10.0), w: Vec3::ZERO };
        let b = bounce(&env, s, &GRASS, (0.0, 0.0));
        assert!(b.v.z > 0.0 && b.v.z < 10.0 * GRASS.e_ref);
        let ke = |s: &BallState| 0.5 * env.ball.mass * s.v.norm_sq() + 0.5 * env.ball.inertia() * s.w.norm_sq();
        assert!(ke(&b) < ke(&s));
        // Friction makes a skidding ball pick up topspin (ω_x < 0 for +Y motion).
        assert!(b.w.x < 0.0);
    }

    #[test]
    fn homer_line_drive_grounder() {
        let env = FlightEnv::default_env();
        let f = Field::default();
        let hr = batted_path(&env, &f, launch(108.0, 28.0, 0.0, 2500.0), 0.0, |_, _| (0.0, 0.0));
        assert_eq!(hr.end(), SegmentEnd::OverFence { fair: true });
        let wall = batted_path(&env, &f, launch(100.0, 18.0, -30.0, 2000.0), 0.0, |_, _| (0.0, 0.0));
        assert!(wall.segments.iter().any(|s| matches!(s, Segment::Flight { end: SegmentEnd::Wall, .. })) || wall.end() == SegmentEnd::Rest);
        let gb = batted_path(&env, &f, launch(95.0, -5.0, 10.0, 0.0), 0.0, |_, _| (0.0, 0.0));
        assert_eq!(gb.end(), SegmentEnd::Rest);
        assert!(gb.bounces.len() >= 2);
        let (rest, _) = gb.at(1e9);
        assert!(rest.y > 30.0, "a 95 mph grounder rolls well into the outfield: {rest:?}");
    }
}
