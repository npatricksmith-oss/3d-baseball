//! Attribute scales (agent-models §1.2, §10). Attributes are stored in physical units; the
//! 20–80 grade is a display transform.

/// "More is better": value(g) = μ + σ_pop·(g − 50)/10.
pub fn linear_from_grade(grade: f64, mu: f64, sigma_pop: f64) -> f64 {
    mu + sigma_pop * (grade - 50.0) / 10.0
}
pub fn grade_from_linear(value: f64, mu: f64, sigma_pop: f64) -> f64 {
    50.0 + 10.0 * (value - mu) / sigma_pop
}

/// "Less is better", multiplicative: value(g) = value_50·ρ^(−(g − 50)/10).
pub fn ratio_from_grade(grade: f64, value_50: f64, rho: f64) -> f64 {
    value_50 * bb_math::m::pow(rho, -(grade - 50.0) / 10.0)
}
pub fn grade_from_ratio(value: f64, value_50: f64, rho: f64) -> f64 {
    50.0 - 10.0 * bb_math::m::ln(value / value_50) / bb_math::m::ln(rho)
}

/// Display clamp (stored values are never clamped).
pub fn display_grade(g: f64) -> f64 {
    g.clamp(20.0, 80.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_10_spot_checks() {
        // ArmSpeed: 41.8 m/s, σ 1.1 → grade 80 = 45.1.
        assert!((linear_from_grade(80.0, 41.8, 1.1) - 45.1).abs() < 1e-9);
        // Command: 10 mrad, ρ 1.2 → grade 80 ≈ 5.8, grade 20 ≈ 17.3.
        assert!((ratio_from_grade(80.0, 10.0, 1.2) - 5.787).abs() < 1e-3);
        assert!((ratio_from_grade(20.0, 10.0, 1.2) - 17.28).abs() < 1e-2);
        let g = grade_from_ratio(ratio_from_grade(63.0, 10.0, 1.2), 10.0, 1.2);
        assert!((g - 63.0).abs() < 1e-9);
    }
}

use crate::batter::{Discipline, Perception};
use crate::pitcher::PitcherAttrs;
use crate::swing::BatterAttrs;

/// Batter attributes on the 20–80 scale (agent-models §10). 50 everywhere is league average.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BatterGrades {
    pub bat_speed: f64,
    pub quick_hands: f64,
    pub bat_control: f64,
    pub timing: f64,
    pub vision: f64,
    pub spin_recognition: f64,
    pub discipline: f64,
    pub adjustability: f64,
}

impl Default for BatterGrades {
    fn default() -> Self {
        Self { bat_speed: 50.0, quick_hands: 50.0, bat_control: 50.0, timing: 50.0, vision: 50.0, spin_recognition: 50.0, discipline: 50.0, adjustability: 50.0 }
    }
}

impl BatterGrades {
    /// Apply to physical attributes, keeping height and handedness.
    pub fn apply(&self, b: &mut BatterAttrs, p: &mut Perception, d: &mut Discipline) {
        b.bat_speed = linear_from_grade(self.bat_speed, 32.0, 1.1);
        b.t_swing = ratio_from_grade(self.quick_hands, 0.150, 1.05);
        b.bat_control = ratio_from_grade(self.bat_control, 0.012, 1.25);
        b.timing = ratio_from_grade(self.timing, 0.006, 1.25);
        b.adjustability = linear_from_grade(self.adjustability, 0.6, 0.15).max(0.0);
        p.sigma_ang = ratio_from_grade(self.vision, 0.001, 1.2);
        p.weber = ratio_from_grade(self.vision, 0.10, 1.2);
        p.sigma_s0 = ratio_from_grade(self.spin_recognition, 1.0, 1.25);
        p.tau_s = ratio_from_grade(self.spin_recognition, 0.08, 1.25);
        d.c = linear_from_grade(self.discipline, 1.0, 0.15);
        // A high grade removes the "everything looks like a strike" bias (§10 note).
        d.b = -linear_from_grade(self.discipline, 0.0, 0.25);
    }
}

/// Pitcher attributes on the 20–80 scale (agent-models §10).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitcherGrades {
    pub arm_speed: f64,
    pub spin_capacity: f64,
    pub command: f64,
    pub extension: f64,
}

impl Default for PitcherGrades {
    fn default() -> Self {
        Self { arm_speed: 50.0, spin_capacity: 50.0, command: 50.0, extension: 50.0 }
    }
}

impl PitcherGrades {
    pub fn apply(&self, p: &mut PitcherAttrs) {
        p.arm_speed = linear_from_grade(self.arm_speed, 41.8, 1.1);
        p.spin_capacity = linear_from_grade(self.spin_capacity, 24.2, 1.4);
        p.command = ratio_from_grade(self.command, 0.010, 1.2);
        p.extension = linear_from_grade(self.extension, 1.95, 0.12);
    }
}

/// Named archetypes for labs and tests (M3 gate: "archetypes produce distinct profiles").
pub mod archetypes {
    use super::{BatterGrades, PitcherGrades};

    pub fn batters() -> Vec<(&'static str, BatterGrades)> {
        let d = BatterGrades::default();
        vec![
            ("Average", d),
            ("Power", BatterGrades { bat_speed: 72.0, bat_control: 38.0, timing: 45.0, discipline: 45.0, ..d }),
            ("Contact", BatterGrades { bat_speed: 38.0, bat_control: 70.0, timing: 68.0, vision: 60.0, ..d }),
            ("Patient", BatterGrades { discipline: 75.0, vision: 60.0, ..d }),
            ("Free swinger", BatterGrades { discipline: 25.0, ..d }),
            ("Elite", BatterGrades { bat_speed: 68.0, quick_hands: 65.0, bat_control: 68.0, timing: 68.0, vision: 68.0, spin_recognition: 68.0, discipline: 68.0, adjustability: 68.0 }),
            ("Weak", BatterGrades { bat_speed: 35.0, quick_hands: 35.0, bat_control: 35.0, timing: 35.0, vision: 35.0, spin_recognition: 35.0, discipline: 35.0, adjustability: 35.0 }),
        ]
    }

    pub fn pitchers() -> Vec<(&'static str, PitcherGrades)> {
        let d = PitcherGrades::default();
        vec![
            ("Average", d),
            ("Flamethrower", PitcherGrades { arm_speed: 75.0, spin_capacity: 60.0, command: 38.0, ..d }),
            ("Control artist", PitcherGrades { arm_speed: 38.0, command: 75.0, ..d }),
        ]
    }
}
