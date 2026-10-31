//! Observation types: the only window agents get onto the world (PLAN §4.1, §15).
//!
//! Also holds small domain enums that both agents and the orchestrator name (pitch types,
//! handedness), since `bb_agents` may not depend on `bb_world`.

/// Savant-style pitch type codes (data-formats §9.4; same values as `bb.PitchType` in the schemas).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PitchType {
    FourSeam = 1,
    Sinker = 2,
    Cutter = 3,
    Slider = 4,
    Sweeper = 5,
    Slurve = 6,
    Curveball = 7,
    KnuckleCurve = 8,
    Changeup = 9,
    Splitter = 10,
}

impl PitchType {
    pub const ALL: [PitchType; 10] = [
        PitchType::FourSeam,
        PitchType::Sinker,
        PitchType::Cutter,
        PitchType::Slider,
        PitchType::Sweeper,
        PitchType::Slurve,
        PitchType::Curveball,
        PitchType::KnuckleCurve,
        PitchType::Changeup,
        PitchType::Splitter,
    ];

    /// Savant `pitch_type` code.
    pub fn code(self) -> &'static str {
        match self {
            PitchType::FourSeam => "FF",
            PitchType::Sinker => "SI",
            PitchType::Cutter => "FC",
            PitchType::Slider => "SL",
            PitchType::Sweeper => "ST",
            PitchType::Slurve => "SV",
            PitchType::Curveball => "CU",
            PitchType::KnuckleCurve => "KC",
            PitchType::Changeup => "CH",
            PitchType::Splitter => "FS",
        }
    }

    pub fn from_code(s: &str) -> Option<PitchType> {
        PitchType::ALL.into_iter().find(|p| p.code().eq_ignore_ascii_case(s))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hand {
    Left,
    Right,
}

impl Hand {
    /// Sign of the arm side along X: right-handers' arm side is third base (−X).
    pub fn arm_side(self) -> f64 {
        match self {
            Hand::Right => -1.0,
            Hand::Left => 1.0,
        }
    }
}

use bb_math::Vec3;

/// What a batter perceives at the pitcher's release (agent-models §3.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReleaseObs {
    /// Play time of release, s.
    pub t: f64,
    /// Estimated release point (noisy).
    pub r: Vec3,
    /// σ of that estimate per axis, m (the batter knows its own acuity).
    pub sigma: f64,
}

/// One perceptual sample of the ball in flight (agent-models §3.3), every ~10 ms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BallObs {
    pub t: f64,
    /// Azimuth of the ball from the eye, rad: atan2(dx, dy) (+ toward first base).
    pub az: f64,
    /// Elevation of the ball from the eye, rad.
    pub el: f64,
    /// σ of the sample-to-sample (independent) part of the angular noise, rad. The
    /// persistent part is a per-pitch bias the agent models separately.
    pub sigma_ang: f64,
    /// Perceived time to arrival at the plate plane, s (optical expansion).
    pub tau: f64,
    /// σ of the independent part of `tau`'s noise, s, derived from the *perceived* τ.
    pub sigma_tau: f64,
    /// Perceived spin signature (see [`spin_signature`]) and its per-component σ.
    pub spin_cue: [f64; 3],
    pub sigma_spin: f64,
}

/// The spin signature a batter reads from seams: the catcher-view Magnus direction scaled by
/// spin efficiency, and the spin rate in units of 2500 rpm. Pitch types differ in it.
pub fn spin_signature(magnus_dir_x: f64, magnus_dir_z: f64, efficiency: f64, rate_rpm: f64) -> [f64; 3] {
    [magnus_dir_x * efficiency, magnus_dir_z * efficiency, rate_rpm / 2500.0]
}

/// Ball-strike count.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Count {
    pub balls: u8,
    pub strikes: u8,
}

impl Count {
    pub fn index(self) -> usize {
        (self.balls as usize) * 3 + self.strikes as usize
    }
}
