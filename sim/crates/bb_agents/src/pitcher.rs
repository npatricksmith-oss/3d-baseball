//! Pitcher: attributes, arsenal, and the noise-free release plan (agent-models §2.1–§2.3).

use bb_math::Vec3;
use bb_math::units::{DEG, MPH, RPM};
use bb_observe::{Hand, PitchType};
use bb_physics::FlightEnv;
use bb_physics::pitch::{PitchShape, aim, release_point, tilt_to_axis_deg};

/// Physical pitcher attributes in internal units (agent-models §10).
#[derive(Clone, Debug, PartialEq)]
pub struct PitcherAttrs {
    pub throws: Hand,
    /// Standing height, m.
    pub height: f64,
    /// Fastball release speed, m/s.
    pub arm_speed: f64,
    /// Fastball spin per unit release speed, rpm per mph.
    pub spin_capacity: f64,
    /// Release-angle σ per axis, rad.
    pub command: f64,
    /// Release extension, m.
    pub extension: f64,
    /// Arm angle above horizontal at release, rad.
    pub arm_slot: f64,
}

impl PitcherAttrs {
    /// A league-average (grade 50) right-hander.
    pub fn average_rhp() -> Self {
        Self {
            throws: Hand::Right,
            height: 1.88,
            arm_speed: 41.8,
            spin_capacity: 24.2,
            command: 0.010,
            extension: 1.95,
            arm_slot: 45.0 * DEG,
        }
    }
    /// Fastball spin rate, rad/s.
    pub fn fastball_spin(&self) -> f64 {
        self.spin_capacity * (self.arm_speed / MPH) * RPM
    }
    pub fn release_point(&self) -> Vec3 {
        release_point(self.height, self.arm_slot, self.extension, self.throws.arm_side())
    }
}

/// One pitch in an arsenal (agent-models §2.1). Tilt and gyro sign are written for a
/// right-hander; [`PitchDef::shape`] mirrors them for left-handers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchDef {
    pub ty: PitchType,
    pub speed_ratio: f64,
    pub spin_rate_factor: f64,
    /// Clock hours, pitcher's perspective, for a right-hander.
    pub tilt_rhp: f64,
    pub efficiency: f64,
    /// Gyro direction for a right-hander.
    pub gyro_sign_rhp: f64,
    pub command_mult: f64,
    /// Release-point shift (x toward arm side, z), m.
    pub release_offset: (f64, f64),
}

impl PitchDef {
    /// Typical shapes [E], chosen to land in public pitch-type ranges (validation in tests).
    pub fn template(ty: PitchType) -> PitchDef {
        let d = |speed_ratio, spin_rate_factor, tilt_rhp, efficiency, gyro_sign_rhp, command_mult| PitchDef {
            ty,
            speed_ratio,
            spin_rate_factor,
            tilt_rhp,
            efficiency,
            gyro_sign_rhp,
            command_mult,
            release_offset: (0.0, 0.0),
        };
        match ty {
            PitchType::FourSeam => d(1.00, 1.00, 1.0, 0.92, 1.0, 1.0),
            PitchType::Sinker => d(0.985, 0.95, 2.25, 0.88, 1.0, 1.05),
            PitchType::Cutter => d(0.94, 1.05, 11.4, 0.28, -1.0, 1.05),
            PitchType::Slider => d(0.90, 1.10, 9.6, 0.19, -1.0, 1.15),
            PitchType::Sweeper => d(0.87, 1.12, 9.0, 0.54, -1.0, 1.2),
            PitchType::Slurve => d(0.85, 1.12, 8.25, 0.38, -1.0, 1.2),
            PitchType::Curveball => d(0.83, 1.15, 7.3, 0.50, -1.0, 1.25),
            PitchType::KnuckleCurve => d(0.85, 1.12, 7.1, 0.42, -1.0, 1.2),
            PitchType::Changeup => d(0.89, 0.78, 2.0, 0.90, 1.0, 1.1),
            PitchType::Splitter => d(0.92, 0.55, 2.6, 0.62, 1.0, 1.15),
        }
    }

    /// Physical shape for this pitcher (no fatigue yet: M6).
    pub fn shape(&self, p: &PitcherAttrs) -> PitchShape {
        let (tilt, gyro) = match p.throws {
            Hand::Right => (self.tilt_rhp, self.gyro_sign_rhp),
            Hand::Left => (12.0 - self.tilt_rhp, -self.gyro_sign_rhp),
        };
        PitchShape {
            speed: self.speed_ratio * p.arm_speed,
            spin_rate: self.spin_rate_factor * p.fastball_spin(),
            spin_axis_deg: tilt_to_axis_deg(tilt),
            efficiency: self.efficiency,
            gyro_sign: gyro,
        }
    }

    pub fn release_point(&self, p: &PitcherAttrs) -> Vec3 {
        let side = p.throws.arm_side();
        p.release_point() + Vec3::new(side * self.release_offset.0, 0.0, self.release_offset.1)
    }
}

/// What the pitcher means to throw: a pitch and a target on the judgment plane.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchIntent {
    pub ty: PitchType,
    pub target_x: f64,
    pub target_z: f64,
}

/// The noise-free release the pitcher plans (the aiming solve's answer).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntendedRelease {
    pub r: Vec3,
    pub shape: PitchShape,
    pub yaw: f64,
    pub elev: f64,
    /// Execution σ multiplier for this pitch type (command_mult).
    pub command_mult: f64,
}

/// Solve the release for an intent. Pitchers know their own pitches: this uses the true
/// physics and is exact (agent-models §2.3).
pub fn plan_release(env: &FlightEnv, p: &PitcherAttrs, def: &PitchDef, intent: &PitchIntent) -> IntendedRelease {
    let r = def.release_point(p);
    let shape = def.shape(p);
    let a = aim(env, r, &shape, intent.target_x, intent.target_z, None);
    IntendedRelease { r, shape, yaw: a.yaw, elev: a.elev, command_mult: def.command_mult }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bb_physics::pitch::{fly_pitch, launch};

    #[test]
    fn planned_release_hits_target() {
        let env = FlightEnv::default_env();
        let p = PitcherAttrs::average_rhp();
        for ty in PitchType::ALL {
            let def = PitchDef::template(ty);
            let intent = PitchIntent { ty, target_x: 0.1, target_z: 0.7 };
            let rel = plan_release(&env, &p, &def, &intent);
            let tr = fly_pitch(&env, launch(rel.r, &rel.shape, rel.yaw, rel.elev));
            let end = tr.last().s.r;
            assert_eq!(tr.stop, Some(0), "{ty:?} bounced");
            assert!((end.x - 0.1).abs() < 2e-4 && (end.z - 0.7).abs() < 2e-4, "{ty:?}: {end:?}");
        }
    }
}
