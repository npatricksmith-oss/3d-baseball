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
