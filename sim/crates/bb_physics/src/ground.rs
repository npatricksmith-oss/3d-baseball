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

/// A simple park: fence distances down the lines and to center, wall height, infield dirt,
/// and the walls of foul territory (physics-spec §9.5).
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
    /// Home (rear apex) to the backstop, m (physics-spec §9.5, B).
    pub backstop_m: f64,
    /// The foul-territory walls stand this far off the foul lines, m (§9.5, C).
    pub foul_clear_m: f64,
    /// Distance along the line where the side wall starts closing in on the pole, m (s_T).
    pub foul_taper_m: f64,
    /// Height of the foul-territory walls (the front of the stands), m [E].
    pub foul_wall_height: f64,
    /// Height of the backstop screen, which covers the walls behind home (s < 0), m [E].
    pub backstop_height: f64,
    /// Each dugout opens in the side wall from this distance along the line... (m) [E]
    pub dugout_from_m: f64,
    /// ...to this one (m) [E].
    pub dugout_to_m: f64,
    /// Restitution of the backstop and foul-territory walls [E] (§9.5).
    pub foul_wall_e: f64,
}

impl Default for Field {
    /// A symmetric reference park: 330 / 400 / 330 ft, 8-ft wall; backstop 60 ft, foul walls
    /// 50 ft off the lines closing in from 100 ft to the poles (physics-spec §9.5).
    fn default() -> Self {
        Self {
            fence_lf: 330.0 * FT,
            fence_cf: 400.0 * FT,
            fence_rf: 330.0 * FT,
            wall_height: 8.0 * FT,
            dirt_radius: 95.0 * FT,
            wall_e: 0.35,
            backstop_m: 60.0 * FT,
            foul_clear_m: 50.0 * FT,
            foul_taper_m: 100.0 * FT,
            foul_wall_height: 4.0 * FT,
            backstop_height: 9.0,
            dugout_from_m: 20.0 * FT,
            dugout_to_m: 80.0 * FT,
            foul_wall_e: 0.35,
        }
    }
}

/// A piece of the playing-field boundary (physics-spec §9.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallKind {
    Fence,
    Backstop,
    /// The side wall parallel to a foul line (dugouts open in it).
    Side,
    /// The stands closing in from the side wall to the foul pole.
    Taper,
}

/// The nearest boundary piece to a point: which, its signed distance (positive inside the
/// field; the fence's is from its curve, the walls' from their planes), the inward horizontal
/// unit normal, and the point's distance along its foul line (s, from the rear apex of home).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallAt {
    pub kind: WallKind,
    pub dist: f64,
    pub inward: Vec3,
    pub s: f64,
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

    /// Every boundary piece at (x, y) with its signed distance (positive inside), in the order
    /// fence, backstop, side wall, taper (physics-spec §9.5). The walls are those on the
    /// point's side of the field (x ≥ 0: first base).
    pub fn pieces(&self, x: f64, y: f64) -> [WallAt; 4] {
        let r2 = core::f64::consts::SQRT_2;
        let sg = if x < 0.0 { -1.0 } else { 1.0 };
        let xa = x.abs();
        let s = (xa + y) / r2;
        let c = (xa - y) / r2;
        let rho = m::hypot(x, y);
        let radial = Vec3::new(x, y, 0.0).normalized();
        let fence = WallAt { kind: WallKind::Fence, dist: self.fence_at(m::atan2(x, y)) - rho, inward: -radial, s };
        let back = WallAt { kind: WallKind::Backstop, dist: y + self.backstop_m, inward: Vec3::Y, s };
        let side = WallAt { kind: WallKind::Side, dist: self.foul_clear_m - c, inward: Vec3::new(-sg, 1.0, 0.0) * (1.0 / r2), s };
        let s_pole = if x < 0.0 { self.fence_lf } else { self.fence_rf };
        let k = self.foul_clear_m / (s_pole - self.foul_taper_m);
        let norm = m::sqrt(1.0 + k * k);
        let taper_d = (k * (s_pole - s) - c) / norm;
        // Inward in (s, c) is (−k, −1)/norm; s = (1, 1)/√2 and c = (1, −1)/√2 on this side.
        let inward = Vec3::new(sg * (-k - 1.0), -k + 1.0, 0.0) * (1.0 / (r2 * norm));
        let taper = WallAt { kind: WallKind::Taper, dist: taper_d, inward, s };
        [fence, back, side, taper]
    }

    /// The boundary piece a ball of radius `r` centered at (x, y) is nearest to touching, and
    /// its clearance (≥ 0 inside the field). A ball meets a wall when its center comes within
    /// `r` of the plane; the outfield fence skeleton keeps using the center on its curve
    /// (physics-spec §9.5).
    pub fn ball_clearance(&self, x: f64, y: f64, r: f64) -> WallAt {
        let mut best = WallAt { kind: WallKind::Fence, dist: f64::INFINITY, inward: Vec3::Y, s: 0.0 };
        for mut w in self.pieces(x, y) {
            if w.kind != WallKind::Fence {
                w.dist -= r;
            }
            if w.dist < best.dist {
                best = w;
            }
        }
        best
    }

    /// The boundary piece nearest a body of radius `r` centered at (x, y) (every piece
    /// measured from its plane or curve, minus `r`), for keeping players inside the field.
    pub fn body_clearance(&self, x: f64, y: f64, r: f64) -> WallAt {
        let mut best = WallAt { kind: WallKind::Fence, dist: f64::INFINITY, inward: Vec3::Y, s: 0.0 };
        for mut w in self.pieces(x, y) {
            w.dist -= r;
            if w.dist < best.dist {
                best = w;
            }
        }
        best
    }

    /// `p` moved straight back inside the field until a body of radius `r` there is clear of
    /// every wall (unchanged if it already is).
    pub fn keep_inside(&self, p: Vec3, r: f64) -> Vec3 {
        let mut q = p;
        for _ in 0..4 {
            let w = self.body_clearance(q.x, q.y, r);
            if w.dist >= 0.0 {
                break;
            }
            q = q + w.inward * (-w.dist + 1e-6);
        }
        q
    }

    /// Height of a wall piece at distance `s` along its foul line: the backstop screen behind
    /// home, an opening (0) at a dugout, the stands wall elsewhere (physics-spec §9.5).
    pub fn wall_top(&self, w: &WallAt) -> f64 {
        match w.kind {
            WallKind::Fence => self.wall_height,
            _ if w.s < 0.0 => self.backstop_height,
            WallKind::Side if w.s >= self.dugout_from_m && w.s <= self.dugout_to_m => 0.0,
            _ => self.foul_wall_height,
        }
    }
}

/// A ball meeting the boundary of the playing field (physics-spec §9.5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WallContact {
    /// Over the top of the wall, or into a dugout: the ball has left the field.
    Out,
    /// Below the top: it caroms off with this state (stepped back inside).
    Carom(BallState),
}

/// What happens to a ball of radius `r` in state `s` that has just reached the boundary.
pub fn wall_contact(field: &Field, s: BallState, r: f64) -> (WallContact, WallAt) {
    let w = field.ball_clearance(s.r.x, s.r.y, r);
    let top = field.wall_top(&w);
    if s.r.z >= top + r {
        return (WallContact::Out, w);
    }
    let e = if w.kind == WallKind::Fence { field.wall_e } else { field.foul_wall_e };
    // The normal velocity reverses with the wall's restitution; the fence skeleton reflects
    // the radial velocity (its curve's normal is nearly radial).
    let n = w.inward;
    let vn = s.v.dot(n);
    let v = if vn < 0.0 { s.v - n * ((1.0 + e) * vn) } else { s.v };
    // Step the ball back inside so it does not re-trigger (at the crossing the clearance is
    // zero to the event tolerance).
    let r_in = s.r + n * 0.01;
    (WallContact::Carom(BallState { r: r_in, v, w: s.w * 0.5 }), w)
}

/// The path of a ball in state `s` at time `t` that has just gone over a wall or into a dugout
/// (see [`wall_contact`]): on into the seats or the dugout for a foul-territory wall, ending
/// at once at the fence. It ends `OverFence`.
pub fn over_the_wall(env: &FlightEnv, field: &Field, s: BallState, t: f64) -> BallPath {
    let w = field.ball_clearance(s.r.x, s.r.y, env.ball.radius);
    let fair = Field::fair(s.r.x, s.r.y);
    let tr = if w.kind == WallKind::Fence { Trajectory::single(t, s) } else { into_the_stands(env, s, t, DT_BATTED, field.wall_top(&w)) };
    BallPath { segments: vec![Segment::Flight { tr, end: SegmentEnd::OverFence { fair } }], bounces: vec![], walls: vec![] }
}

/// Fly a ball that has left the field over a wall until it comes down into the seats (to
/// the wall top's height) or a dugout, or `t_max` passes. The fence skeleton ends at the
/// fence instead (a home run is over when it clears).
fn into_the_stands(env: &FlightEnv, s: BallState, t: f64, dt: f64, top: f64) -> Trajectory {
    fly(env, s, t, dt, &[Crossing::ZBelow(top.max(0.0) + env.ball.radius)], 15.0)
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
pub fn batted_path(env: &FlightEnv, field: &Field, s0: BallState, t0: f64, tilts: impl FnMut(u32, &Surface) -> (f64, f64)) -> BallPath {
    batted_path_dt(env, field, s0, t0, DT_BATTED, tilts)
}

/// [`batted_path`] with an explicit flight step (agents' internal predictions use a coarser
/// step than the world).
pub fn batted_path_dt(env: &FlightEnv, field: &Field, s0: BallState, t0: f64, dt: f64, mut tilts: impl FnMut(u32, &Surface) -> (f64, f64)) -> BallPath {
    let mut segs = Vec::new();
    let mut bounces = Vec::new();
    let mut walls = Vec::new();
    let mut s = s0;
    let mut t = t0;
    let r = env.ball.radius;
    // A ball starting on the ground (a deflection, a short-hopped throw) bounces or rolls
    // from there; flight would never "cross" the ground it starts on.
    if s.r.z <= r + 1e-3 && s.v.z <= 0.0 {
        s.r.z = r + 1e-3;
        if s.v.z.abs() < 1e-6 {
            s.v.z = -1e-3;
        }
    }
    for k in 0..40u32 {
        let fence = Crossing::Boundary { field: *field, r };
        let tr = fly(env, s, t, dt, &[Crossing::ZBelow(r), fence], 15.0);
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
                    let beyond = |p: Vec3| field.ball_clearance(p.x, p.y, r).dist < 0.0;
                    if beyond(pos(hi)) {
                        for _ in 0..40 {
                            let mid = 0.5 * (lo + hi);
                            if beyond(pos(mid)) { hi = mid } else { lo = mid }
                        }
                        let p = pos(lo);
                        // Rolling into a dugout, it leaves the field (physics-spec §9.5).
                        let w = field.ball_clearance(pos(hi).x, pos(hi).y, r);
                        if field.wall_top(&w) <= 0.0 {
                            segs.push(Segment::Roll { t0: t, r0, v0: vh, decel, t_stop: t + hi });
                            let v_in = dir * (sp - decel * hi).max(0.0);
                            segs.push(Segment::Flight { tr: Trajectory::single(t + hi, BallState { r: pos(hi), v: v_in, w: Vec3::ZERO }), end: SegmentEnd::OverFence { fair: Field::fair(p.x, p.y) } });
                            return BallPath { segments: segs, bounces, walls };
                        }
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
                match wall_contact(field, last.s, r) {
                    (WallContact::Out, w) => {
                        let fair = Field::fair(p.x, p.y);
                        // Over a foul-territory wall it flies on into the seats or the dugout;
                        // over the fence the path ends as it clears.
                        let tr = if w.kind == WallKind::Fence { tr } else { tr.join(into_the_stands(env, last.s, last.t, dt, field.wall_top(&w))) };
                        segs.push(Segment::Flight { tr, end: SegmentEnd::OverFence { fair } });
                        return BallPath { segments: segs, bounces, walls };
                    }
                    (WallContact::Carom(ns), _) => {
                        walls.push((last.t, p.x, p.y));
                        segs.push(Segment::Flight { tr, end: SegmentEnd::Wall });
                        s = ns;
                        t = last.t;
                    }
                }
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

    /// Foul territory inside the walls, by counting a fine grid of points that are inside the
    /// field and foul: physics-spec §9.5 gives 23,885 sq ft (its polygon, by the shoelace
    /// formula, for B = 60 ft, C = 50 ft, s_T = 100 ft, poles at 330 ft).
    #[test]
    fn foul_territory_area_matches_the_spec() {
        let f = Field::default();
        let h = 0.25; // ft
        let mut n = 0u64;
        let (nx, ny) = ((480.0 / h) as i64, (310.0 / h) as i64);
        for i in 0..nx {
            // Sample points off the cell centres, so none lies on a foul line or a wall.
            let x = -240.0 + (i as f64 + 0.37) * h;
            for j in 0..ny {
                let y = -70.0 + (j as f64 + 0.61) * h;
                if y < x.abs() && f.body_clearance(x * FT, y * FT, 0.0).dist >= 0.0 {
                    n += 1;
                }
            }
        }
        let area = n as f64 * h * h;
        // The count's own error on a 0.25 ft grid is a few tens of sq ft; one foot more or
        // less of wall clearance C moves the area by about 450.
        assert!((area - 23_885.0).abs() < 36.0, "foul territory {area:.0} sq ft");
    }

    /// The backstop is 60 ft behind the apex and meets the side walls at x = ±3.26 m
    /// (physics-spec §9.5: s = C − B√2 there, x = (s + C)/√2 = 10.71 ft).
    #[test]
    fn backstop_and_side_walls_meet_where_the_spec_says() {
        let f = Field::default();
        let w = f.body_clearance(0.0, -60.0 * FT, 0.0);
        assert_eq!(w.kind, WallKind::Backstop);
        assert!(w.dist.abs() < 1e-9);
        for sx in [-1.0, 1.0] {
            let p = f.pieces(sx * 10.7106 * FT, -60.0 * FT);
            assert!(p[1].dist.abs() < 1e-3 && p[2].dist.abs() < 1e-3, "{p:?}");
        }
        // 50 ft off the first-base line at 1B (s = 90 ft): on the side wall.
        let (s, c) = (90.0 * FT, 50.0 * FT);
        let (x, y) = ((s + c) / core::f64::consts::SQRT_2, (s - c) / core::f64::consts::SQRT_2);
        let w = f.body_clearance(x, y, 0.0);
        assert_eq!(w.kind, WallKind::Side);
        assert!(w.dist.abs() < 1e-9);
        // The taper reaches the line at the pole, 330 ft out: a point just foul there is out.
        let pole = 330.0 * FT / core::f64::consts::SQRT_2;
        assert!(f.body_clearance(pole - 1.0, pole - 0.5, 0.0).dist > 0.0, "fair side of the pole is in");
        assert!(f.body_clearance(pole + 0.5, pole - 1.5, 0.0).dist < 0.0, "behind the pole is out");
    }

    /// Wall heights by place (physics-spec §9.5): the screen behind home 9.0 m, a dugout
    /// opening (0) from 20 to 80 ft along the line, the stands wall 4 ft elsewhere.
    #[test]
    fn wall_tops_by_place() {
        let f = Field::default();
        let at = |s_ft: f64, c_ft: f64| {
            let (s, c) = (s_ft * FT, c_ft * FT);
            let (x, y) = ((s + c) / core::f64::consts::SQRT_2, (s - c) / core::f64::consts::SQRT_2);
            let w = f.body_clearance(x, y, 0.0);
            (w.kind, f.wall_top(&w))
        };
        assert_eq!(at(-20.0, 50.0), (WallKind::Side, 9.0));
        assert_eq!(at(10.0, 50.0), (WallKind::Side, 4.0 * FT));
        assert_eq!(at(50.0, 50.0), (WallKind::Side, 0.0));
        assert_eq!(at(95.0, 50.0), (WallKind::Side, 4.0 * FT));
        assert_eq!(at(200.0, 50.0 * 130.0 / 230.0), (WallKind::Taper, 4.0 * FT));
        let w = f.body_clearance(0.0, -60.0 * FT, 0.0);
        assert_eq!(f.wall_top(&w), 9.0);
    }

    /// A ball driven straight into the first-base stands wall below its top comes back with
    /// 0.35 of its normal speed, its speed along the wall kept (§9.5); over the top it is out.
    #[test]
    fn stands_wall_carom_and_over_the_top() {
        let f = Field::default();
        let env = FlightEnv::default_env();
        let r = env.ball.radius;
        let n_out = Vec3::new(1.0, -1.0, 0.0).normalized(); // off the line into foul ground
        let along = Vec3::new(1.0, 1.0, 0.0).normalized();
        let (s, c) = (90.0 * FT, 50.0 * FT - r);
        let p = along * s + n_out * c + Vec3::new(0.0, 0.0, 0.6);
        let st = BallState { r: p, v: n_out * 20.0 + along * 5.0, w: Vec3::ZERO };
        let (hit, w) = wall_contact(&f, st, r);
        assert_eq!(w.kind, WallKind::Side);
        let WallContact::Carom(out) = hit else { panic!("below the 4-ft top it caroms") };
        assert!((out.v.dot(n_out) + 7.0).abs() < 1e-9 && (out.v.dot(along) - 5.0).abs() < 1e-9, "{:?}", out.v);
        let high = BallState { r: p + Vec3::new(0.0, 0.0, 4.0 * FT), ..st };
        assert_eq!(wall_contact(&f, high, r).0, WallContact::Out);
    }

    /// Batted balls against the walls, end to end.
    #[test]
    fn foul_balls_meet_the_walls() {
        let env = FlightEnv::default_env();
        let f = Field::default();
        // A high foul fly toward the first-base seats (spray 70°): over the 4-ft wall, then on
        // down into the seats (height of the wall top), foul.
        let pop = batted_path(&env, &f, launch(85.0, 55.0, 70.0, 2500.0), 0.0, |_, _| (0.0, 0.0));
        assert_eq!(pop.end(), SegmentEnd::OverFence { fair: false });
        let (p, _) = pop.at(pop.t_end());
        assert!(f.body_clearance(p.x, p.y, 0.0).dist < 0.0 && (p.z - (4.0 * FT + env.ball.radius)).abs() < 1e-6, "{p:?}");
        // A pop-up straight back over the backstop screen (9 m) is out; a liner into it is not.
        let back = batted_path(&env, &f, launch(80.0, 60.0, 180.0, 2000.0), 0.0, |_, _| (0.0, 0.0));
        assert_eq!(back.end(), SegmentEnd::OverFence { fair: false });
        let liner = batted_path(&env, &f, launch(70.0, 5.0, 180.0, 1500.0), 0.0, |_, _| (0.0, 0.0));
        assert!(!liner.walls.is_empty() && liner.walls[0].2 < -18.2, "{:?}", liner.walls);
        assert_ne!(liner.end(), SegmentEnd::OverFence { fair: false });
        // A grounder rolling foul toward the third-base dugout (spray −80°: 35° off the line,
        // it meets the wall 50 ft off the line 71 ft along it, inside the 20–80 ft opening).
        let gb = batted_path(&env, &f, launch(60.0, -8.0, -80.0, 0.0), 0.0, |_, _| (0.0, 0.0));
        assert_eq!(gb.end(), SegmentEnd::OverFence { fair: false }, "{:?}", gb.at(gb.t_end()));
    }

    #[test]
    fn keep_inside_stops_a_body_short_of_the_wall() {
        let f = Field::default();
        let q = f.keep_inside(Vec3::new(0.0, -20.0, 0.0), 0.3);
        assert!((q.y - (-60.0 * FT + 0.3)).abs() < 1e-5 && q.x == 0.0, "{q:?}");
        let inside = Vec3::new(5.0, 30.0, 0.0);
        assert_eq!(f.keep_inside(inside, 0.3), inside);
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
