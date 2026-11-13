//! Swing model (agent-models §3.6), v1 kinematics.
//!
//! v1 is the zero-lag special case of the spec's template (λ ≡ 0): the bat is rigid and
//! radial from a rotation center C near the batter's spine and sweeps about a fixed axis n̂
//! through C. The contact pose fixes n̂: the bat points from C to the aim point, and the
//! sweet spot moves "forward, tilted up by the attack angle". Angular speed follows a
//! Gaussian bump in time, so early or late contact happens at lower bat speed, and the ball
//! meets a different station on the bat (jammed / end of the bat). A biomechanical model can
//! replace this behind the same [`BatPose`] interface.

use bb_math::units::DEG;
use bb_math::{Vec3, m};
use bb_observe::Hand;
use bb_physics::bat::{BatProfile, MassProps};
use bb_physics::contact::{BatKinematics, BatPose};

/// Ratio of the bump half-width-at-10%-speed to the Gaussian width: √(2 ln 10).
const START_WIDTHS: f64 = 2.145_966_026_289_347;

/// Physical batter attributes in internal units (agent-models §10).
#[derive(Clone, Debug, PartialEq)]
pub struct BatterAttrs {
    pub bats: Hand,
    /// Standing height, m.
    pub height: f64,
    /// Sweet-spot speed at nominal reach, m/s (`BatSpeed`).
    pub bat_speed: f64,
    /// Swing start (10% speed) to contact, s (`QuickHands`).
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

/// Template geometry [E], for a right-handed batter (mirrored in x for left-handers).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwingTemplate {
    /// Rotation center: x from the plate's center line, y, and z as a fraction of height.
    pub pivot_x: f64,
    pub pivot_y: f64,
    pub pivot_z_frac: f64,
    /// Nominal sweet-spot radius from the pivot, m (where `bat_speed` is defined).
    pub r_nominal: f64,
    /// Allowed knob distance from the pivot (arm reach), m.
    pub r_knob_min: f64,
    pub r_knob_max: f64,
    /// Contact depth y_c(x) = y0 − k·x_inside (inside pitches met out front), m.
    pub contact_y0: f64,
    pub contact_k: f64,
    /// Default attack angle, rad.
    pub attack_angle: f64,
    /// Vertical offset of the aim point from the predicted ball center, m (negative = aim
    /// under the ball to lift it) [E, authored against mean launch angle, calibration §4.4].
    pub aim_offset_z: f64,
    /// Mid-swing adjustment stops this long before the aim time, s [E] (agent-models §3.6
    /// `t_cut`; PLAN range 50–80 ms).
    pub adj_cutoff_s: f64,
    /// The swing's phase rate may change by at most this fraction over the swing still to
    /// come after motor latency [E] (agent-models §3.6: ±10%).
    pub adj_phase_frac: f64,
    /// A check swing can start only before this fraction of the swing's start-to-contact
    /// *time* [E]. The spec gives the abort point as a phase, `s_abort` = 0.45 (agent-models
    /// §3.6); under v1's Gaussian speed bump a time fraction is not the same thing, and v1's
    /// 0.6 of the time is kept as built until a change is validated.
    pub abort_time_frac: f64,
}

impl Default for SwingTemplate {
    fn default() -> Self {
        Self {
            pivot_x: -0.95,
            pivot_y: 0.30,
            pivot_z_frac: 0.60,
            r_nominal: 1.00,
            r_knob_min: 0.15,
            r_knob_max: 0.70,
            contact_y0: 0.35,
            contact_k: 0.6,
            attack_angle: 10.0 * DEG,
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
    pub fn pivot(&self, b: &BatterAttrs) -> Vec3 {
        Vec3::new(Self::side(b.bats) * self.pivot_x, self.pivot_y, self.pivot_z_frac * b.height)
    }
    /// Contact depth for a pitch at plate x (agent-models §3.6).
    pub fn contact_y(&self, b: &BatterAttrs, x: f64) -> f64 {
        // "Inside" is toward the batter: −X for a RHB.
        let x_inside = -Self::side(b.bats) * x;
        self.contact_y0 + self.contact_k * x_inside
    }
    /// The bat's vertical angle at contact for an aim point (rad, negative = barrel below
    /// the hands): the elevation of the pivot-to-aim line, since v1's bat is radial.
    pub fn bat_angle(&self, b: &BatterAttrs, aim: Vec3) -> f64 {
        let d = aim - self.pivot(b);
        m::atan2(d.z, m::hypot(d.x, d.y))
    }
    /// Counterfactual geometry (R13 check 4): the same template with the pivot swung about
    /// `aim` in its vertical plane, at the same distance, so that the bat meets `aim` at the
    /// vertical angle `angle` (rad). Reach and bat speed at that aim point are unchanged; the
    /// swing plane tilts with the bat (`plane_normal`).
    pub fn with_bat_angle(&self, b: &BatterAttrs, aim: Vec3, angle: f64) -> Self {
        let d = self.pivot(b) - aim;
        let (r, h) = (d.norm(), m::hypot(d.x, d.y));
        let (ux, uy) = (d.x / h, d.y / h);
        // The pivot is above the aim point for a negative bat angle.
        let (hz, dz) = (r * m::cos(angle), -r * m::sin(angle));
        let p = Vec3::new(aim.x + ux * hz, aim.y + uy * hz, aim.z + dz);
        Self { pivot_x: Self::side(b.bats) * p.x, pivot_y: p.y, pivot_z_frac: p.z / b.height, ..*self }
    }
}

/// Placement θ (agent-models §3.6), as deviations applied by execution noise.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Placement {
    /// Swing-plane tilt change (attack angle), rad.
    pub plane_tilt: f64,
    /// Vertical offset of the whole swing, m.
    pub v_offset: f64,
    /// Reach change (knob distance from the pivot), m.
    pub reach: f64,
    /// Contact-time shift, s.
    pub t_shift: f64,
}

/// An analytic swing: the bat's pose is a closed-form function of time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Swing {
    pub pivot: Vec3,
    /// Rotation axis (unit).
    pub n: Vec3,
    /// Bat axis at the nominal contact instant (unit, knob → tip).
    pub axis_c: Vec3,
    /// Knob distance from the pivot, m.
    pub r_knob: f64,
    /// Peak angular speed, rad/s.
    pub omega: f64,
    /// Gaussian width of the speed bump, s.
    pub width: f64,
    /// Nominal contact time (peak speed), s.
    pub t_c: f64,
    /// Bat center of mass from the knob, m.
    pub u_cm: f64,
    /// Check swing: when braking starts (agent-models §3.6). The phase rate then decays
    /// linearly to zero over [`BRAKE_S`].
    pub t_brake: Option<f64>,
}

/// Braking duration of a check swing, s [E] (agent-models §3.6).
pub const BRAKE_S: f64 = 0.040;

impl Swing {
    fn free_angle(&self, t: f64) -> f64 {
        let tau = (t - self.t_c) / (self.width * core::f64::consts::SQRT_2);
        self.omega * self.width * m::sqrt(core::f64::consts::FRAC_PI_2) * m::erf(tau)
    }
    fn free_speed(&self, t: f64) -> f64 {
        let tau = (t - self.t_c) / self.width;
        self.omega * m::exp(-0.5 * tau * tau)
    }
    /// Rotation angle relative to the contact pose (braking included).
    pub fn angle(&self, t: f64) -> f64 {
        match self.t_brake {
            Some(tb) if t > tb => {
                let tau = (t - tb).min(BRAKE_S);
                self.free_angle(tb) + self.free_speed(tb) * (tau - tau * tau / (2.0 * BRAKE_S))
            }
            _ => self.free_angle(t),
        }
    }
    pub fn angular_speed(&self, t: f64) -> f64 {
        match self.t_brake {
            Some(tb) if t > tb => self.free_speed(tb) * (1.0 - ((t - tb) / BRAKE_S).min(1.0)),
            _ => self.free_speed(t),
        }
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
    /// Swing start (10% of peak speed).
    pub fn t_start(&self) -> f64 {
        self.t_c - START_WIDTHS * self.width
    }
}

impl BatPose for Swing {
    fn at(&self, t: f64) -> BatKinematics {
        let a = self.axis_c.rotate(self.n, self.angle(t));
        let knob = self.pivot + a * self.r_knob;
        let omega = self.n * self.angular_speed(t);
        let cm = knob + a * self.u_cm;
        BatKinematics { knob, axis: a, v_cm: omega.cross(cm - self.pivot), omega }
    }
    fn nominal_contact(&self) -> f64 {
        self.t_c
    }
}

/// The intended swing (noise-free): what the batter plans for an aim point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntendedSwing {
    pub swing: Swing,
    /// Planned attack angle, rad (plane-tilt noise is added to it).
    pub attack_angle: f64,
}

/// Inverse: plan a swing whose sweet spot passes through `aim` at time `t_aim`, with the
/// given attack angle (agent-models §3.6). Reach limits are respected, so an aim point out
/// of reach yields a swing that misses its target (and the world decides the result).
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
    let c = tpl.pivot(b);
    let to_aim = aim - c;
    let axis_c = to_aim.normalized();
    let uss = profile.length() - ss_from_tip;
    let r_knob = (to_aim.norm() - uss).clamp(tpl.r_knob_min, tpl.r_knob_max);
    let n = plane_normal(axis_c, attack_angle);
    let swing = Swing {
        pivot: c,
        n,
        axis_c,
        r_knob,
        omega: b.bat_speed / tpl.r_nominal,
        width: b.t_swing / START_WIDTHS,
        t_c: t_aim,
        u_cm: mp.u_cm,
        t_brake: None,
    };
    IntendedSwing { swing, attack_angle }
}

/// Swing-plane normal for a bat axis and an attack angle: the sweet spot moves "forward
/// (+Y), tilted up by the attack angle", made perpendicular to the bat.
fn plane_normal(axis: Vec3, attack_angle: f64) -> Vec3 {
    let d = Vec3::new(0.0, m::cos(attack_angle), m::sin(attack_angle));
    let e_v = d.reject(axis).normalized();
    axis.cross(e_v)
}

/// Apply a placement deviation (used by `motor::execute_swing`, and by the cage lab to
/// sweep offsets deterministically).
pub fn apply_placement(i: &IntendedSwing, p: &Placement) -> Swing {
    let mut s = i.swing;
    s.pivot.z += p.v_offset;
    s.r_knob += p.reach;
    s.t_c += p.t_shift;
    if p.plane_tilt != 0.0 {
        // Positive tilt = more uppercut (Codex review: rebuild from the tilted attack angle).
        s.n = plane_normal(s.axis_c, i.attack_angle + p.plane_tilt);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweet_spot_hits_aim_at_t_aim_with_bat_speed() {
        let b = BatterAttrs::average_rhb();
        let tpl = SwingTemplate::default();
        let prof = BatProfile::default_34in();
        let mp = prof.mass_props();
        let aim = Vec3::new(0.05, tpl.contact_y(&b, 0.05), 0.80);
        let i = plan_swing(&b, &tpl, &prof, &mp, 0.15, aim, 0.42, tpl.attack_angle);
        let k = i.swing.at(0.42);
        let uss = bb_physics::bat::sweet_spot(prof.length());
        assert!((k.point(uss) - aim).norm() < 1e-9);
        let v = k.point_velocity(k.point(uss), mp.u_cm);
        let r = (aim - tpl.pivot(&b)).norm();
        assert!((v.norm() - b.bat_speed * r / tpl.r_nominal).abs() < 1e-9);
        // Attack angle as planned; sweet spot moving toward the pitcher.
        let aa = m::atan2(v.z, m::hypot(v.x, v.y));
        assert!(v.y > 0.0);
        assert!((aa - tpl.attack_angle).abs() < 0.05, "attack {aa}");
        // The bat starts well behind the batter and turns counter-clockwise from above (RHB).
        assert!(i.swing.n.z > 0.9);
        let start = i.swing.at(i.swing.t_start());
        assert!(start.axis.y < -0.3, "start axis {:?}", start.axis);
    }

    #[test]
    fn plane_tilt_changes_attack_angle_with_sign() {
        let b = BatterAttrs::average_rhb();
        let tpl = SwingTemplate::default();
        let prof = BatProfile::default_34in();
        let mp = prof.mass_props();
        let aim = Vec3::new(0.0, 0.35, 0.8);
        let i = plan_swing(&b, &tpl, &prof, &mp, 0.15, aim, 0.4, tpl.attack_angle);
        let aa = |tilt: f64| {
            let s = apply_placement(&i, &Placement { plane_tilt: tilt, ..Default::default() });
            let k = s.at(0.4);
            let v = k.point_velocity(k.point(prof.length() - 0.15), mp.u_cm);
            m::atan2(v.z, m::hypot(v.x, v.y))
        };
        let (lo, mid, hi) = (aa(-0.05), aa(0.0), aa(0.05));
        assert!(lo < mid && mid < hi, "{lo} {mid} {hi}");
        assert!(((hi - mid) - 0.05).abs() < 0.01);
    }

    #[test]
    fn braking_stops_the_bat_and_is_judged() {
        let b = BatterAttrs::average_rhb();
        let tpl = SwingTemplate::default();
        let prof = BatProfile::default_34in();
        let mp = prof.mass_props();
        let i = plan_swing(&b, &tpl, &prof, &mp, 0.15, Vec3::new(0.0, 0.35, 0.8), 0.4, tpl.attack_angle);
        let mut early = i.swing;
        early.t_brake = Some(early.t_start() + 0.03);
        assert!(!early.judged_swing());
        assert!(early.angular_speed(early.t_brake.unwrap() + BRAKE_S + 0.01) == 0.0);
        let mut late = i.swing;
        late.t_brake = Some(0.39);
        assert!(late.judged_swing());
    }

    #[test]
    fn bat_angle_counterfactual_keeps_reach_and_sets_the_angle() {
        let b = BatterAttrs::average_rhb();
        let tpl = SwingTemplate::default();
        let prof = BatProfile::default_34in();
        let mp = prof.mass_props();
        let aim = Vec3::new(0.0, 0.35, 0.75);
        // Default geometry: pivot 0.95 m to the side and 0.36 m above the aim point.
        let a0 = tpl.bat_angle(&b, aim);
        assert!((a0 - m::atan2(0.75 - 0.6 * 1.85, m::hypot(0.95, 0.05))).abs() < 1e-12);
        for (deg, bats) in [(-30.0, Hand::Right), (-35.0, Hand::Left), (-10.0, Hand::Right)] {
            let b = BatterAttrs { bats, ..b.clone() };
            let aim = Vec3::new(if bats == Hand::Left { 0.03 } else { -0.03 }, 0.35, 0.75);
            let t2 = tpl.with_bat_angle(&b, aim, deg * DEG);
            assert!(((aim - t2.pivot(&b)).norm() - (aim - tpl.pivot(&b)).norm()).abs() < 1e-12);
            // The planned bat axis at contact has that elevation (barrel below the hands).
            let i = plan_swing(&b, &t2, &prof, &mp, 0.15, aim, 0.4, t2.attack_angle);
            let ax = i.swing.at(0.4).axis;
            assert!((m::asin(ax.z) - deg * DEG).abs() < 1e-9, "{deg}: {}", m::asin(ax.z) / DEG);
            // Same side of the plate as before: the batter did not move across it.
            assert!(t2.pivot(&b).x * tpl.pivot(&b).x > 0.0);
        }
    }

    #[test]
    fn lefty_mirrors() {
        let r = BatterAttrs::average_rhb();
        let l = BatterAttrs { bats: Hand::Left, ..r.clone() };
        let tpl = SwingTemplate::default();
        let prof = BatProfile::default_34in();
        let mp = prof.mass_props();
        let ir = plan_swing(&r, &tpl, &prof, &mp, 0.15, Vec3::new(0.1, 0.3, 0.8), 0.4, tpl.attack_angle);
        let il = plan_swing(&l, &tpl, &prof, &mp, 0.15, Vec3::new(-0.1, 0.3, 0.8), 0.4, tpl.attack_angle);
        for t in [0.3, 0.38, 0.4, 0.41] {
            let (a, b) = (ir.swing.at(t), il.swing.at(t));
            assert!((a.knob.x + b.knob.x).abs() < 1e-12 && (a.knob.y - b.knob.y).abs() < 1e-12);
        }
    }
}
