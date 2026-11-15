//! Swing model (agent-models §3.6), v2 kinematics (2026-11-14).
//!
//! A kinematic double pendulum in one swing plane: the hands (knob) move on an arc of radius ρ
//! about the hand-arc center C while the bat turns about the knob, lagging the hands early and
//! releasing by the nominal contact instant t_c. The hands' and the bat's angular rates are
//! Gaussian bumps in time, the hands' peaking first. The contact pose is planned in a
//! contact-relative frame (PLAN §8.3): the sweet spot passes through the aim point at t_c,
//! moving in the planned attack angle and direction, in a plane of the planned tilt, at the
//! batter's `BatSpeed` whatever the reach (v1 turned at a fixed angular speed, so its sweet-spot
//! speed grew with reach). Early or late contact meets the ball at another speed, another station,
//! and with the bat's face turned against its path.

use bb_math::units::DEG;
use bb_math::{Vec3, m};
use bb_observe::Hand;
use bb_physics::bat::{BatProfile, MassProps};
use bb_physics::contact::{BatKinematics, BatPose};

/// Ratio of a bump's half-width at 10% of its peak to its Gaussian width: √(2 ln 10).
pub const START_WIDTHS: f64 = 2.145_966_026_289_347;
/// The swing time (start to contact) at which the template's time constants are authored, s.
pub const T_SWING_REF: f64 = 0.150;

/// Physical batter attributes in internal units (agent-models §10).
#[derive(Clone, Debug, PartialEq)]
pub struct BatterAttrs {
    pub bats: Hand,
    /// Standing height, m.
    pub height: f64,
    /// Sweet-spot speed at the nominal contact instant, m/s (`BatSpeed`; any aim, §3.6 v2).
    pub bat_speed: f64,
    /// Swing start (hands at 10% of their peak rate) to contact, s (`QuickHands`).
    pub t_swing: f64,
    /// σ of the vertical offset δ_v, m (`BatControl`).
    pub bat_control: f64,
    /// σ of the contact time, s (`Timing`).
    pub timing: f64,
    /// Mid-swing correction speed of the aim point, m/s (`Adjustability`).
    pub adjustability: f64,
}

impl BatterAttrs {
    pub fn average_rhb() -> Self {
        Self { bats: Hand::Right, height: 1.85, bat_speed: 32.0, t_swing: 0.150, bat_control: 0.012, timing: 0.006, adjustability: 0.6 }
    }
}

/// Template geometry and timing [E] (agent-models §3.6), for a right-handed batter (mirrored
/// in x for left-handers). Angles in rad, lengths in m, times in s at T_swing = 0.150 s.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwingTemplate {
    /// Stance reference (the batter's box, the client's stance pose): x from the plate's
    /// center line, y, and z as a fraction of height. Not the swing's center in v2.
    pub pivot_x: f64,
    pub pivot_y: f64,
    pub pivot_z_frac: f64,
    /// Lateral position of the hand-arc center C, m (right-handed; mirrored).
    pub center_x: f64,
    /// Hand radius limits as fractions of height (hands in tight, arms fully extended).
    pub rho_min_frac: f64,
    pub rho_max_frac: f64,
    /// Partial reach adjustment g (agent-models §3.6 step 7) [E]: the batter changes his hand
    /// radius from the one a middle-middle aim needs by this fraction of the change the aim
    /// needs; the rest moves the planned contact along the bat (jammed inside, toward the tip
    /// outside). 1 = every aim met at the sweet spot.
    pub reach_gain: f64,
    /// Hands' angular rate over the bat's at t_c (q).
    pub hand_rate_ratio: f64,
    /// Lag θ − φ at t_c, rad (λ_c).
    pub lag_c: f64,
    /// Hands' bump width, bat's bump width, bat's peak offset from t_c (at T_SWING_REF).
    pub hand_width: f64,
    pub bat_width: f64,
    pub bat_peak: f64,
    /// Contact depth y_c(x) = y0 + k·x_inside (inside pitches met out front), m.
    pub contact_y0: f64,
    pub contact_k: f64,
    /// Attack angle at a middle-height aim, and its change per zone height, rad.
    pub attack_angle: f64,
    pub attack_angle_h: f64,
    /// Swing path tilt at a middle aim, per zone height, per m inside, rad.
    pub tilt: f64,
    pub tilt_h: f64,
    pub tilt_x: f64,
    /// Attack direction (pull-positive) at the plate's center line, and per m inside, rad.
    pub direction: f64,
    pub direction_x: f64,
    /// The batter's strike zone as fractions of height (normalizes the aim's height).
    pub zone_bottom_frac: f64,
    pub zone_top_frac: f64,
    /// Vertical offset of the aim point from the predicted ball center, m (negative = aim
    /// under the ball to lift it) [E, authored against mean launch angle, calibration §4.4;
    /// re-checked on v2 with per-player bat speed, docs/calibration/2026-11-14-swing-recalibration.md
    /// §2.3: −2.0 cm gives 11.8° in games against Statcast's 12.2°].
    pub aim_offset_z: f64,
    /// Mid-swing adjustment stops this long before the aim time, s [E] (agent-models §3.6
    /// `t_cut`; PLAN range 50–80 ms).
    pub adj_cutoff_s: f64,
    /// The swing's phase rate may change by at most this fraction over the swing still to
    /// come after motor latency [E] (agent-models §3.6: ±10%).
    pub adj_phase_frac: f64,
    /// A check swing can start only before this fraction of the swing's start-to-contact
    /// *time* [E] (agent-models §3.6; the spec's phase `s_abort` = 0.45 is not the same thing).
    pub abort_time_frac: f64,
}

impl Default for SwingTemplate {
    fn default() -> Self {
        Self {
            pivot_x: -0.95,
            pivot_y: 0.30,
            pivot_z_frac: 0.60,
            center_x: -1.10,
            rho_min_frac: 0.15,
            rho_max_frac: 0.46,
            // [E], Proposed 2026-11-15 (agent-models §3.6 step 7; docs/calibration/
            // 2026-11-15-collision-q-and-poor-contact.md).
            reach_gain: 0.6,
            hand_rate_ratio: 0.32,
            lag_c: 0.0,
            hand_width: 0.055,
            bat_width: 0.055,
            bat_peak: 0.010,
            contact_y0: 0.35,
            contact_k: 0.125,
            attack_angle: 8.0 * DEG,
            attack_angle_h: -8.7 * DEG,
            tilt: 32.0 * DEG,
            tilt_h: -9.9 * DEG,
            tilt_x: 11.0 * DEG,
            direction: 0.0,
            direction_x: 16.6 * DEG,
            zone_bottom_frac: 0.27,
            zone_top_frac: 0.535,
            aim_offset_z: -0.020,
            adj_cutoff_s: 0.060,
            adj_phase_frac: 0.10,
            abort_time_frac: 0.6,
        }
    }
}

impl SwingTemplate {
    fn side(bats: Hand) -> f64 {
        // RHB stands on the third-base side (−X); mirror for LHB.
        match bats {
            Hand::Right => 1.0,
            Hand::Left => -1.0,
        }
    }
    /// The stance reference point (batter's box), not the swing's center.
    pub fn pivot(&self, b: &BatterAttrs) -> Vec3 {
        Vec3::new(Self::side(b.bats) * self.pivot_x, self.pivot_y, self.pivot_z_frac * b.height)
    }
    /// Contact depth for a pitch at plate x (agent-models §3.6).
    pub fn contact_y(&self, b: &BatterAttrs, x: f64) -> f64 {
        self.contact_y0 + self.contact_k * self.x_inside(b, x)
    }
    /// Distance inside of the plate's center line (toward the batter), m.
    pub fn x_inside(&self, b: &BatterAttrs, x: f64) -> f64 {
        -Self::side(b.bats) * x
    }
    /// The aim's height in the batter's zone: 0 at the bottom, 1 at the top.
    pub fn zone_height(&self, b: &BatterAttrs, z: f64) -> f64 {
        let (lo, hi) = (self.zone_bottom_frac * b.height, self.zone_top_frac * b.height);
        (z - lo) / (hi - lo)
    }
    /// Planned attack angle at an aim point (rad).
    pub fn attack_angle_at(&self, b: &BatterAttrs, aim: Vec3) -> f64 {
        self.attack_angle + self.attack_angle_h * (self.zone_height(b, aim.z) - 0.5)
    }
    /// Planned swing path tilt at an aim point (rad), before the |α| < β guard.
    pub fn tilt_at(&self, b: &BatterAttrs, aim: Vec3) -> f64 {
        self.tilt + self.tilt_h * (self.zone_height(b, aim.z) - 0.5) + self.tilt_x * self.x_inside(b, aim.x)
    }
    /// Planned attack direction at an aim point (rad, pull-positive).
    pub fn direction_at(&self, b: &BatterAttrs, aim: Vec3) -> f64 {
        self.direction + self.direction_x * self.x_inside(b, aim.x)
    }
    /// The bat's vertical angle at contact for an aim point with the planned attack angle (rad,
    /// negative = barrel below the hands). With the bat ⊥ to the path in a plane of tilt β,
    /// sin²(angle) = sin²β − sin²α.
    pub fn bat_angle(&self, b: &BatterAttrs, aim: Vec3) -> f64 {
        let a = self.attack_angle_at(b, aim);
        let beta = guard_tilt(self.tilt_at(b, aim), a);
        -m::asin(m::sqrt((m::sin(beta) * m::sin(beta) - m::sin(a) * m::sin(a)).max(0.0)))
    }
    /// Counterfactual geometry (R13 check 4): the same template with its tilt shifted so the
    /// bat meets `aim` at the vertical angle `angle` (rad, negative); the height and location
    /// slopes are kept.
    pub fn with_bat_angle(&self, b: &BatterAttrs, aim: Vec3, angle: f64) -> Self {
        let a = self.attack_angle_at(b, aim);
        let beta = m::asin(m::sqrt(m::sin(angle) * m::sin(angle) + m::sin(a) * m::sin(a)).min(1.0));
        Self { tilt: self.tilt + beta - self.tilt_at(b, aim), ..*self }
    }
}

/// How much steeper than |α| a plane raised by the tilt guard is, rad [E] (agent-models §3.6
/// step 5).
pub const TILT_GUARD_MARGIN: f64 = DEG;

/// The plane must be steeper than the path it contains (agent-models §3.6 step 5).
fn guard_tilt(beta: f64, alpha: f64) -> f64 {
    if alpha.abs() >= beta { alpha.abs() + TILT_GUARD_MARGIN } else { beta }
}

/// Placement θ (agent-models §3.6), as deviations applied by execution noise.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Placement {
    /// Swing-plane tilt change (attack angle), rad.
    pub plane_tilt: f64,
    /// Vertical offset of the whole swing, m.
    pub v_offset: f64,
    /// Reach change: the whole swing moved along the bat's contact axis, m.
    pub reach: f64,
    /// Contact-time shift, s.
    pub t_shift: f64,
    /// Relative change of the swing's speed (effort): both rate bumps scale by 1 + effort.
    pub effort: f64,
}

/// One Gaussian rate bump about t_c: rate(τ) = peak·exp(−(τ − τ_p)²/2w²).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bump {
    /// Peak rate, rad/s.
    pub peak: f64,
    /// Width, s.
    pub width: f64,
    /// Peak time relative to t_c, s.
    pub at: f64,
}

impl Bump {
    /// A bump whose rate at τ = 0 is `rate_c`.
    fn through(rate_c: f64, width: f64, at: f64) -> Self {
        let g = (at / width) * (at / width);
        Bump { peak: rate_c / m::exp(-0.5 * g), width, at }
    }
    fn rate(&self, tau: f64) -> f64 {
        let x = (tau - self.at) / self.width;
        self.peak * m::exp(-0.5 * x * x)
    }
    /// Angle turned from τ = 0 to τ.
    fn angle(&self, tau: f64) -> f64 {
        let c = self.peak * self.width * m::sqrt(core::f64::consts::FRAC_PI_2);
        let s = self.width * core::f64::consts::SQRT_2;
        c * (m::erf((tau - self.at) / s) - m::erf(-self.at / s))
    }
}

/// An analytic swing: the bat's pose is a closed-form function of time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Swing {
    /// Hand-arc center C.
    pub pivot: Vec3,
    /// Swing-plane normal, the direction of the angular velocity (unit).
    pub n: Vec3,
    /// Bat axis at the nominal contact instant (unit, knob → tip).
    pub axis_c: Vec3,
    /// Direction from C to the knob at the nominal contact instant (unit, in the plane).
    pub hand_c: Vec3,
    /// Hand radius ρ, m.
    pub rho: f64,
    /// Hands' and bat's angular-rate bumps.
    pub hands: Bump,
    pub bat: Bump,
    /// Nominal contact time, s.
    pub t_c: f64,
    /// Bat center of mass from the knob, m.
    pub u_cm: f64,
    /// Check swing: when braking starts (agent-models §3.6). Both rates then decay linearly
    /// to zero over [`BRAKE_S`].
    pub t_brake: Option<f64>,
}

/// Braking duration of a check swing, s [E] (agent-models §3.6).
pub const BRAKE_S: f64 = 0.040;

impl Swing {
    /// (angle, rate) of one bump at time t, braking included.
    fn turn(&self, b: &Bump, t: f64) -> (f64, f64) {
        match self.t_brake {
            Some(tb) if t > tb => {
                let s = (t - tb).min(BRAKE_S);
                let r0 = b.rate(tb - self.t_c);
                (b.angle(tb - self.t_c) + r0 * (s - s * s / (2.0 * BRAKE_S)), r0 * (1.0 - s / BRAKE_S))
            }
            _ => (b.angle(t - self.t_c), b.rate(t - self.t_c)),
        }
    }
    /// The bat's rotation in the plane relative to its contact pose (braking included).
    pub fn angle(&self, t: f64) -> f64 {
        self.turn(&self.bat, t).0
    }
    /// The bat's angular speed.
    pub fn angular_speed(&self, t: f64) -> f64 {
        self.turn(&self.bat, t).1
    }
    /// Largest rotation the bat reaches (for the check-swing judgment, rules-spec §4.5).
    pub fn max_angle(&self) -> f64 {
        match self.t_brake {
            Some(tb) => self.angle(tb + BRAKE_S),
            None => f64::INFINITY,
        }
    }
    /// Rules-spec §4.5: the bat's horizontal direction at its maximum rotation has reached
    /// perpendicular to the pitcher–home line (barrel no longer pointing back toward the
    /// catcher).
    pub fn judged_swing(&self) -> bool {
        let Some(tb) = self.t_brake else { return true };
        let a = self.axis_c.rotate(self.n, self.angle(tb + BRAKE_S));
        a.y >= 0.0
    }
    /// Swing start: the hands at 10% of their peak rate.
    pub fn t_start(&self) -> f64 {
        self.t_c + self.hands.at - START_WIDTHS * self.hands.width
    }
    /// Delay the whole swing, if need be, so it starts no earlier than `t` (agent-models
    /// §3.4): a batter cannot start a swing before he has decided on it. One whose plan and
    /// execution noise would start it earlier starts at `t` and meets the ball that much later.
    pub fn start_no_earlier_than(&mut self, t: f64) {
        let early = t - self.t_start();
        if early > 0.0 {
            self.t_c += early;
            // Exactly: rounding can leave the start short of `t` (by many ulps when `t_c` is large against the widths).
            while self.t_start() < t {
                self.t_c = self.t_c.next_up();
            }
        }
    }
}

impl BatPose for Swing {
    fn at(&self, t: f64) -> BatKinematics {
        let (phi, phi_dot) = self.turn(&self.hands, t);
        let (th, th_dot) = self.turn(&self.bat, t);
        let e = self.hand_c.rotate(self.n, phi);
        let a = self.axis_c.rotate(self.n, th);
        let knob = self.pivot + e * self.rho;
        let v_knob = self.n.cross(e) * (self.rho * phi_dot);
        let omega = self.n * th_dot;
        BatKinematics { knob, axis: a, v_cm: v_knob + omega.cross(a * self.u_cm), omega }
    }
    fn nominal_contact(&self) -> f64 {
        self.t_c
    }
}

/// What the plan fixed besides the swing itself, so execution can rebuild the plane.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwingGeometry {
    pub aim: Vec3,
    pub tilt: f64,
    pub direction: f64,
    pub side: f64,
    pub center_x: f64,
    pub rho_min: f64,
    pub rho_max: f64,
    /// The hand radius a middle-middle aim needs (step 7's reference), m, and the reach gain.
    pub rho_ref: f64,
    pub reach_gain: f64,
    pub u_ss: f64,
    pub bat_speed: f64,
    pub hand_rate_ratio: f64,
    pub lag_c: f64,
}

/// The intended swing (noise-free): what the batter plans for an aim point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntendedSwing {
    pub swing: Swing,
    /// Planned attack angle, rad (plane-tilt noise is added to it).
    pub attack_angle: f64,
    pub geometry: SwingGeometry,
}

/// The contact pose (agent-models §3.6 steps 4–7): plane normal, bat axis at contact, hand
/// direction, hand radius, hand-arc center, and the bat's angular rate at t_c.
fn contact_pose(g: &SwingGeometry, alpha: f64) -> (Vec3, Vec3, Vec3, f64, Vec3, f64) {
    let beta = guard_tilt(g.tilt, alpha);
    let z = Vec3::new(0.0, 0.0, 1.0);
    // Horizontal direction at ψ, pull-positive: pull is −X for a right-handed batter.
    let h = Vec3::new(-g.side * m::sin(g.direction), m::cos(g.direction), 0.0);
    let v_hat = h * m::cos(alpha) + z * m::sin(alpha);
    let p = z.cross(h);
    let ck = -m::tan(alpha) / m::tan(beta);
    let sk = -g.side * m::sqrt((1.0 - ck * ck).max(0.0));
    let nu = z * m::cos(beta) + (h * ck + p * sk) * m::sin(beta);
    let n = nu * g.side;
    let axis = v_hat.cross(n).normalized();
    let hand = axis.rotate(n, -g.lag_c);
    let x_c = g.side * g.center_x;
    // With λ_c = 0 the knob and the sweet spot lie on the line from C along the axis. In
    // general the sweet spot is C + ρ·hand + u_ss·axis; solve ρ from the lateral constraint.
    let rho_aim = (g.aim.x - x_c - g.u_ss * axis.x) / hand.x;
    // Partial reach (step 7): only the fraction g of the change from the middle-middle radius.
    // With g = 1 this is exactly ρ_aim.
    let rho_raw = rho_aim - (1.0 - g.reach_gain) * (rho_aim - g.rho_ref);
    let rho = rho_raw.clamp(g.rho_min, g.rho_max);
    let c = g.aim - hand * rho - axis * g.u_ss;
    // A clamped swing keeps C's lateral position: it is short or long along the bat axis.
    let c = c + axis * ((x_c - c.x) / axis.x);
    // Sweet-spot speed at t_c: |ρ·q·k·(n×hand) + u_ss·k·(n×axis)| = BatSpeed.
    let unit_v = n.cross(hand) * (rho * g.hand_rate_ratio) + n.cross(axis) * g.u_ss;
    let k = g.bat_speed / unit_v.norm();
    (n, axis, hand, rho, c, k)
}

fn build(g: &SwingGeometry, alpha: f64, hands: (f64, f64), bat: (f64, f64), t_c: f64, u_cm: f64) -> Swing {
    let (n, axis_c, hand_c, rho, pivot, k) = contact_pose(g, alpha);
    Swing {
        pivot,
        n,
        axis_c,
        hand_c,
        rho,
        hands: Bump::through(k * g.hand_rate_ratio, hands.0, hands.1),
        bat: Bump::through(k, bat.0, bat.1),
        t_c,
        u_cm,
        t_brake: None,
    }
}

/// The unclamped hand radius a middle-middle aim needs (agent-models §3.6 step 7's reference):
/// x = 0, the middle of the batter's zone, the template's own tilt, direction and attack
/// angle there. The aim's depth does not enter the radius; `y` is passed only to place it.
fn reference_rho(b: &BatterAttrs, tpl: &SwingTemplate, u_ss: f64, y: f64) -> f64 {
    let z = (tpl.zone_bottom_frac + 0.5 * (tpl.zone_top_frac - tpl.zone_bottom_frac)) * b.height;
    let aim = Vec3::new(0.0, y, z);
    let g = SwingGeometry {
        aim,
        tilt: tpl.tilt_at(b, aim),
        direction: tpl.direction_at(b, aim),
        side: SwingTemplate::side(b.bats),
        center_x: tpl.center_x,
        rho_min: f64::NEG_INFINITY,
        rho_max: f64::INFINITY,
        rho_ref: 0.0,
        reach_gain: 1.0,
        u_ss,
        bat_speed: b.bat_speed,
        hand_rate_ratio: tpl.hand_rate_ratio,
        lag_c: tpl.lag_c,
    };
    contact_pose(&g, tpl.attack_angle_at(b, aim)).3
}

/// Inverse: plan a swing whose sweet spot passes through `aim` at time `t_aim`, with the
/// given attack angle (agent-models §3.6; callers use [`SwingTemplate::attack_angle_at`]).
/// Reach limits are respected, so an aim point out of reach yields a swing that misses its
/// target (and the world decides the result).
pub fn plan_swing(
    b: &BatterAttrs,
    tpl: &SwingTemplate,
    profile: &BatProfile,
    mp: &MassProps,
    ss_from_tip: f64,
    aim: Vec3,
    t_aim: f64,
    attack_angle: f64,
) -> IntendedSwing {
    let side = SwingTemplate::side(b.bats);
    let u_ss = profile.length() - ss_from_tip;
    let rho_ref = if tpl.reach_gain == 1.0 { 0.0 } else { reference_rho(b, tpl, u_ss, aim.y) };
    let geometry = SwingGeometry {
        aim,
        tilt: tpl.tilt_at(b, aim),
        direction: tpl.direction_at(b, aim),
        side,
        center_x: tpl.center_x,
        rho_min: tpl.rho_min_frac * b.height,
        rho_max: tpl.rho_max_frac * b.height,
        rho_ref,
        reach_gain: tpl.reach_gain,
        u_ss,
        bat_speed: b.bat_speed,
        hand_rate_ratio: tpl.hand_rate_ratio,
        lag_c: tpl.lag_c,
    };
    let s = b.t_swing / T_SWING_REF;
    let w_h = tpl.hand_width * s;
    let hands = (w_h, START_WIDTHS * w_h - b.t_swing);
    let bat = (tpl.bat_width * s, tpl.bat_peak * s);
    let swing = build(&geometry, attack_angle, hands, bat, t_aim, mp.u_cm);
    IntendedSwing { swing, attack_angle, geometry }
}

/// Apply a placement deviation (used by `motor::execute_swing`, and by the cage lab to
/// sweep offsets deterministically).
pub fn apply_placement(i: &IntendedSwing, p: &Placement) -> Swing {
    let mut s = i.swing;
    if p.plane_tilt != 0.0 {
        // Positive tilt = more uppercut: the plane is rebuilt about the same aim point.
        let r = build(&i.geometry, i.attack_angle + p.plane_tilt, (s.hands.width, s.hands.at), (s.bat.width, s.bat.at), s.t_c, s.u_cm);
        s = Swing { t_brake: s.t_brake, ..r };
    }
    s.pivot.z += p.v_offset;
    s.pivot = s.pivot + s.axis_c * p.reach;
    s.t_c += p.t_shift;
    s.hands.peak *= 1.0 + p.effort;
    s.bat.peak *= 1.0 + p.effort;
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (BatterAttrs, SwingTemplate, BatProfile, MassProps) {
        let prof = BatProfile::default_34in();
        let mp = prof.mass_props();
        (BatterAttrs::average_rhb(), SwingTemplate::default(), prof, mp)
    }
    fn plan(b: &BatterAttrs, tpl: &SwingTemplate, prof: &BatProfile, mp: &MassProps, aim: Vec3, t: f64) -> IntendedSwing {
        plan_swing(b, tpl, prof, mp, 0.15, aim, t, tpl.attack_angle_at(b, aim))
    }
    fn ss_vel(s: &Swing, prof: &BatProfile, mp: &MassProps, t: f64) -> (Vec3, Vec3) {
        let k = s.at(t);
        let p = k.point(prof.length() - 0.15);
        (p, k.point_velocity(p, mp.u_cm))
    }
    /// Pull-positive horizontal direction of a velocity, for a right-handed batter.
    fn ad(v: Vec3) -> f64 {
        m::atan2(-v.x, v.y)
    }

    // §3.6: with full reach adjustment (g = 1) every planned swing meets its aim at BatSpeed,
    // with the planned attack angle, attack direction and plane tilt, wherever the aim is (v1's
    // speed grew with reach). With the default partial reach only the station changes (below).
    #[test]
    fn sweet_spot_meets_every_aim_at_bat_speed() {
        let (b, tpl, prof, mp) = setup();
        let tpl = SwingTemplate { reach_gain: 1.0, ..tpl };
        let h = |z: f64| (z - 0.27 * 1.85) / (0.265 * 1.85);
        for (x, z) in [(-0.22, 0.75), (0.0, 0.75), (0.22, 0.75), (0.0, 0.55), (0.0, 0.95)] {
            let aim = Vec3::new(x, tpl.contact_y(&b, x), z);
            let i = plan(&b, &tpl, &prof, &mp, aim, 0.42);
            let (p, v) = ss_vel(&i.swing, &prof, &mp, 0.42);
            assert!((p - aim).norm() < 1e-9, "aim {aim:?}: sweet spot at {p:?}");
            assert!((v.norm() - 32.0).abs() < 1e-9, "aim {aim:?}: speed {}", v.norm());
            // Spec values, computed by hand from the §3.6 formulas.
            let x_in = -x;
            let alpha = (8.0 - 8.7 * (h(z) - 0.5)) * DEG;
            let beta = (32.0 - 9.9 * (h(z) - 0.5) + 11.0 * x_in) * DEG;
            let psi = 16.6 * x_in * DEG;
            assert!((m::atan2(v.z, m::hypot(v.x, v.y)) - alpha).abs() < 1e-9);
            assert!((ad(v) - psi).abs() < 1e-9);
            assert!((m::acos(i.swing.n.z) - beta).abs() < 1e-9, "tilt");
            // The bat is ⊥ to the path at contact (λ_c = 0) and points knob → plate side.
            assert!(i.swing.axis_c.dot(v).abs() < 1e-9 && i.swing.axis_c.x > 0.5);
            // Vertical bat angle: sin² = sin²β − sin²α.
            let vba = -m::asin(m::sqrt(m::sin(beta).powi(2) - m::sin(alpha).powi(2)));
            assert!((m::asin(i.swing.axis_c.z) - vba).abs() < 1e-9);
            assert!((tpl.bat_angle(&b, aim) - vba).abs() < 1e-12);
            // C keeps its lateral position.
            assert!((i.swing.pivot.x - tpl.center_x).abs() < 1e-12);
        }
        // Middle-middle at 8° and 32°: −30.76° (sin = √(0.28081 − 0.01937) = 0.51131).
        let mid = Vec3::new(0.0, 0.35, 0.27 * 1.85 + 0.5 * 0.265 * 1.85);
        assert!((tpl.bat_angle(&b, mid) / DEG + 30.75).abs() < 0.02, "{}", tpl.bat_angle(&b, mid) / DEG);
    }

    // §3.6: swing start is when the hands reach 10% of their peak rate, T_swing before t_c.
    #[test]
    fn swing_starts_t_swing_before_contact() {
        let (b, tpl, prof, mp) = setup();
        for t_sw in [0.150, 0.130, 0.174] {
            let b = BatterAttrs { t_swing: t_sw, ..b.clone() };
            let s = plan(&b, &tpl, &prof, &mp, Vec3::new(0.0, 0.35, 0.75), 0.4).swing;
            assert!((s.t_start() - (0.4 - t_sw)).abs() < 1e-12);
            let (_, r) = s.turn(&s.hands, s.t_start());
            assert!((r / s.hands.peak - 0.1).abs() < 1e-9);
            // The hands peak before contact, the bat after (proximal to distal).
            assert!(s.hands.at < 0.0 && s.bat.at > 0.0);
        }
    }

    // §3.6 kinematics: the knob moves on a circle of radius ρ about C in the plane, and the
    // velocity the pose reports is the time derivative of the positions.
    #[test]
    fn pose_is_a_hand_arc_and_a_turning_bat() {
        let (b, tpl, prof, mp) = setup();
        let s = plan(&b, &tpl, &prof, &mp, Vec3::new(0.1, 0.34, 0.8), 0.4).swing;
        let uss = prof.length() - 0.15;
        for t in [0.27, 0.33, 0.38, 0.4, 0.42, 0.46] {
            let k = s.at(t);
            assert!((((k.knob - s.pivot).norm()) - s.rho).abs() < 1e-12);
            assert!((k.knob - s.pivot).dot(s.n).abs() < 1e-12 && k.axis.dot(s.n).abs() < 1e-12);
            let dt = 1e-6;
            let fd = (s.at(t + dt).point(uss) - s.at(t - dt).point(uss)) * (0.5 / dt);
            let v = k.point_velocity(k.point(uss), mp.u_cm);
            assert!((fd - v).norm() < 1e-5, "t {t}: {fd:?} vs {v:?}");
        }
    }

    // §3.6: away from t_c the bat's face and its path differ: hooked out front, sliced deep.
    #[test]
    fn early_contact_hooks_and_late_contact_slices() {
        let (b, tpl, prof, mp) = setup();
        let s = plan(&b, &tpl, &prof, &mp, Vec3::new(0.0, 0.35, 0.75), 0.4).swing;
        for (dt, sign) in [(0.012, 1.0), (-0.012, -1.0)] {
            let (_, v) = ss_vel(&s, &prof, &mp, 0.4 + dt);
            let face = s.n.cross(s.at(0.4 + dt).axis);
            // Positive when the face has turned past the path in the swing's direction.
            let turn = v.normalized().cross(face).dot(s.n);
            assert!(turn * sign > 0.01, "dt {dt}: {turn}");
        }
        // And the speed falls away on both sides of contact (jammed, out front).
        let sp = |t: f64| ss_vel(&s, &prof, &mp, t).1.norm();
        assert!(sp(0.37) < sp(0.4) && sp(0.43) < sp(0.4));
    }

    // §3.6 step 7, partial reach (2026-11-15): the batter moves his hands only the fraction g
    // of the way from a middle-middle radius toward the one the aim needs. The aim stays on the
    // bat's line at BatSpeed and C keeps its lateral position; the ball meets the bat (1 − g) of
    // the radius change away from the sweet spot: inboard on inside pitches (jammed), toward
    // the tip on outside ones, nowhere else at a middle-middle aim. The radii come from plans
    // with g = 1 (the full adjustment the spec defines the partial one against).
    #[test]
    fn partial_reach_jams_inside_and_reaches_outside() {
        let (b, tpl, prof, mp) = setup();
        assert!(tpl.reach_gain < 1.0);
        let full = SwingTemplate { reach_gain: 1.0, ..tpl };
        let mid_z = (0.27 + 0.5 * 0.265) * 1.85;
        let at = |x: f64| Vec3::new(x, tpl.contact_y(&b, x), mid_z);
        let rho_ref = plan(&b, &full, &prof, &mp, at(0.0), 0.4).swing.rho;
        let mut last = f64::NEG_INFINITY;
        for x in [-0.20, -0.10, 0.0, 0.10, 0.20] {
            let aim = at(x);
            let need = plan(&b, &full, &prof, &mp, aim, 0.4).swing.rho;
            let s = plan(&b, &tpl, &prof, &mp, aim, 0.4).swing;
            let (p, v) = ss_vel(&s, &prof, &mp, 0.4);
            let d = aim - p;
            let station = d.dot(s.axis_c); // + toward the tip
            assert!((d - s.axis_c * station).norm() < 1e-9, "x {x}: the aim is off the bat's line");
            assert!((station - (1.0 - tpl.reach_gain) * (need - rho_ref)).abs() < 1e-9, "x {x}: station {station}");
            assert!((v.norm() - 32.0).abs() < 1e-9 && (s.pivot.x - tpl.center_x).abs() < 1e-12);
            assert!(station > last, "x {x}: the station must move toward the tip as the aim moves out");
            last = station;
            if x == 0.0 {
                assert!(station.abs() < 1e-9);
            } else {
                assert!(station.abs() > 0.02 && station.signum() == x.signum(), "x {x}: {station}");
            }
        }
    }

    // §3.6 step 7: beyond the arms' reach the swing keeps C and comes up short along the bat.
    #[test]
    fn out_of_reach_comes_up_short() {
        let (b, tpl, prof, mp) = setup();
        let tpl = SwingTemplate { reach_gain: 1.0, ..tpl };
        let aim = Vec3::new(0.55, 0.3, 0.75);
        let s = plan(&b, &tpl, &prof, &mp, aim, 0.4).swing;
        assert!((s.rho - 0.46 * 1.85).abs() < 1e-12);
        let (p, v) = ss_vel(&s, &prof, &mp, 0.4);
        let d = aim - p;
        assert!(d.dot(s.axis_c) > 0.05 && (d - s.axis_c * d.dot(s.axis_c)).norm() < 1e-9, "{d:?}");
        assert!((s.pivot.x - tpl.center_x).abs() < 1e-12);
        // Still at BatSpeed.
        assert!((v.norm() - 32.0).abs() < 1e-9);
    }

    #[test]
    fn plane_tilt_changes_attack_angle_with_sign() {
        let (b, tpl, prof, mp) = setup();
        let tpl = SwingTemplate { reach_gain: 1.0, ..tpl };
        let aim = Vec3::new(0.0, 0.35, 0.8);
        let i = plan(&b, &tpl, &prof, &mp, aim, 0.4);
        let aa = |tilt: f64| {
            let s = apply_placement(&i, &Placement { plane_tilt: tilt, ..Default::default() });
            let (p, v) = ss_vel(&s, &prof, &mp, 0.4);
            assert!((p - aim).norm() < 1e-9);
            m::atan2(v.z, m::hypot(v.x, v.y))
        };
        let (lo, mid, hi) = (aa(-0.05), aa(0.0), aa(0.05));
        assert!(((hi - mid) - 0.05).abs() < 1e-9 && ((mid - lo) - 0.05).abs() < 1e-9, "{lo} {mid} {hi}");
        // Reach and vertical offset move the whole swing.
        let s = apply_placement(&i, &Placement { reach: 0.03, v_offset: -0.01, ..Default::default() });
        let (p, _) = ss_vel(&s, &prof, &mp, 0.4);
        assert!((p - (aim + i.swing.axis_c * 0.03 + Vec3::new(0.0, 0.0, -0.01))).norm() < 1e-12);
    }

    #[test]
    fn braking_stops_the_bat_and_is_judged() {
        let (b, tpl, prof, mp) = setup();
        let i = plan(&b, &tpl, &prof, &mp, Vec3::new(0.0, 0.35, 0.8), 0.4);
        let mut early = i.swing;
        early.t_brake = Some(early.t_start() + 0.03);
        assert!(!early.judged_swing());
        let k = early.at(early.t_brake.unwrap() + BRAKE_S + 0.01);
        assert!(early.angular_speed(early.t_brake.unwrap() + BRAKE_S + 0.01) == 0.0 && k.omega.norm() == 0.0 && k.v_cm.norm() == 0.0);
        let mut late = i.swing;
        late.t_brake = Some(0.39);
        assert!(late.judged_swing());
    }

    #[test]
    fn bat_angle_counterfactual_sets_the_angle() {
        let (b, tpl, prof, mp) = setup();
        for (deg, bats) in [(-35.0, Hand::Right), (-25.0, Hand::Left), (-30.0, Hand::Right)] {
            let b = BatterAttrs { bats, ..b.clone() };
            let aim = Vec3::new(if bats == Hand::Left { 0.03 } else { -0.03 }, 0.35, 0.75);
            let t2 = tpl.with_bat_angle(&b, aim, deg * DEG);
            let i = plan(&b, &t2, &prof, &mp, aim, 0.4);
            assert!((m::asin(i.swing.axis_c.z) - deg * DEG).abs() < 1e-9, "{deg}: {}", m::asin(i.swing.axis_c.z) / DEG);
        }
    }

    #[test]
    fn lefty_mirrors() {
        let (r, tpl, prof, mp) = setup();
        let l = BatterAttrs { bats: Hand::Left, ..r.clone() };
        let ir = plan(&r, &tpl, &prof, &mp, Vec3::new(0.1, 0.3, 0.8), 0.4);
        let il = plan(&l, &tpl, &prof, &mp, Vec3::new(-0.1, 0.3, 0.8), 0.4);
        for t in [0.26, 0.3, 0.38, 0.4, 0.41] {
            let (a, b) = (ir.swing.at(t), il.swing.at(t));
            assert!((a.knob.x + b.knob.x).abs() < 1e-12 && (a.knob.y - b.knob.y).abs() < 1e-12 && (a.knob.z - b.knob.z).abs() < 1e-12);
            assert!((a.axis.x + b.axis.x).abs() < 1e-12 && (a.axis.y - b.axis.y).abs() < 1e-12);
        }
    }
}
