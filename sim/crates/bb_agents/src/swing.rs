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
}

impl BatterAttrs {
    pub fn average_rhb() -> Self {
        Self { bats: Hand::Right, height: 1.85, bat_speed: 32.0, t_swing: 0.150, bat_control: 0.012, timing: 0.006 }
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
}

impl Swing {
    /// Rotation angle relative to the contact pose.
    pub fn angle(&self, t: f64) -> f64 {
        let tau = (t - self.t_c) / (self.width * core::f64::consts::SQRT_2);
        self.omega * self.width * m::sqrt(core::f64::consts::FRAC_PI_2) * m::erf(tau)
    }
    pub fn angular_speed(&self, t: f64) -> f64 {
        let tau = (t - self.t_c) / self.width;
        self.omega * m::exp(-0.5 * tau * tau)
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
    /// Attack direction before tilt noise (unit, ⟂ axis): kept so noise can tilt the plane.
    pub e_v: Vec3,
}

/// Inverse: plan a swing whose sweet spot passes through `aim` at time `t_aim`, with the
/// given attack angle (agent-models §3.6). Reach limits are respected, so an aim point out
/// of reach yields a swing that misses its target (and the world decides the result).
pub fn plan_swing(
    b: &BatterAttrs,
    tpl: &SwingTemplate,
    profile: &BatProfile,
    mp: &MassProps,
    aim: Vec3,
    t_aim: f64,
    attack_angle: f64,
) -> IntendedSwing {
    let c = tpl.pivot(b);
    let to_aim = aim - c;
    let axis_c = to_aim.normalized();
    let uss = bb_physics::bat::sweet_spot(profile.length());
    let r_knob = (to_aim.norm() - uss).clamp(tpl.r_knob_min, tpl.r_knob_max);
    let d = Vec3::new(0.0, m::cos(attack_angle), m::sin(attack_angle));
    let e_v = d.reject(axis_c).normalized();
    let n = axis_c.cross(e_v);
    let swing = Swing {
        pivot: c,
        n,
        axis_c,
        r_knob,
        omega: b.bat_speed / tpl.r_nominal,
        width: b.t_swing / START_WIDTHS,
        t_c: t_aim,
        u_cm: mp.u_cm,
    };
    IntendedSwing { swing, e_v }
}

/// Apply a placement deviation (used by `motor::execute_swing`, and by the cage lab to
/// sweep offsets deterministically).
pub fn apply_placement(i: &IntendedSwing, p: &Placement) -> Swing {
    let mut s = i.swing;
    s.pivot.z += p.v_offset;
    s.r_knob += p.reach;
    s.t_c += p.t_shift;
    if p.plane_tilt != 0.0 {
        // Tilt the attack direction about the bat axis' horizontal normal: rotate e_v toward
        // vertical by plane_tilt (positive = more uppercut).
        let side = s.axis_c.cross(Vec3::Z).normalized();
        let e_v = i.e_v.rotate(side, -p.plane_tilt).reject(s.axis_c).normalized();
        s.n = s.axis_c.cross(e_v);
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
        let i = plan_swing(&b, &tpl, &prof, &mp, aim, 0.42, tpl.attack_angle);
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
    fn lefty_mirrors() {
        let r = BatterAttrs::average_rhb();
        let l = BatterAttrs { bats: Hand::Left, ..r.clone() };
        let tpl = SwingTemplate::default();
        let prof = BatProfile::default_34in();
        let mp = prof.mass_props();
        let ir = plan_swing(&r, &tpl, &prof, &mp, Vec3::new(0.1, 0.3, 0.8), 0.4, tpl.attack_angle);
        let il = plan_swing(&l, &tpl, &prof, &mp, Vec3::new(-0.1, 0.3, 0.8), 0.4, tpl.attack_angle);
        for t in [0.3, 0.38, 0.4, 0.41] {
            let (a, b) = (ir.swing.at(t), il.swing.at(t));
            assert!((a.knob.x + b.knob.x).abs() < 1e-12 && (a.knob.y - b.knob.y).abs() < 1e-12);
        }
    }
}
