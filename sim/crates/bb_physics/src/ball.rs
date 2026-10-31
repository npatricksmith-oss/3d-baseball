//! Baseball properties (physics-spec §2) and aerodynamic coefficients (§4.2, §4.4).

use bb_math::units::INCH;

/// Physical properties of the ball.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BallParams {
    /// Mass, kg [S] 5.125 oz.
    pub mass: f64,
    /// Radius, m [D] from the 9.125 in circumference.
    pub radius: f64,
    /// Moment-of-inertia factor α in I = α m R² [E] (R5).
    pub inertia_factor: f64,
}

impl Default for BallParams {
    fn default() -> Self {
        Self {
            mass: 0.14529,
            radius: 9.125 * INCH / (2.0 * core::f64::consts::PI),
            inertia_factor: 0.4,
        }
    }
}

impl BallParams {
    pub fn area(&self) -> f64 {
        core::f64::consts::PI * self.radius * self.radius
    }
    pub fn inertia(&self) -> f64 {
        self.inertia_factor * self.mass * self.radius * self.radius
    }
}

/// Drag, lift, and spin-decay parameters. Defaults are the Trajectory Calculator values via
/// the `pybbda` port [S, verify: R1].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AeroParams {
    pub cd0: f64,
    /// Drag increase per 1000 rpm of **total** spin (physics-spec §4.2 note).
    pub cdspin: f64,
    pub cl0: f64,
    pub cl1: f64,
    pub cl2: f64,
    /// 1/e spin-decay time at 100 mph, s. `None` disables decay (golden tests).
    pub tau100: Option<f64>,
    /// When false, Magnus force is zeroed (the "no-Magnus twin" used for movement, §12).
    pub magnus: bool,
}

impl Default for AeroParams {
    fn default() -> Self {
        Self { cd0: 0.3008, cdspin: 0.0292, cl0: 0.583, cl1: 2.333, cl2: 1.120, tau100: Some(22.0), magnus: true }
    }
}

impl AeroParams {
    /// Calculator-default aero with no spin decay, as used by the golden tests.
    pub fn calculator() -> Self {
        Self { tau100: None, ..Self::default() }
    }
    pub fn no_magnus(self) -> Self {
        Self { magnus: false, ..self }
    }
}
