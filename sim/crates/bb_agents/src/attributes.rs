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
