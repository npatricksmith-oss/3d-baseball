//! Spin-axis conventions (data-formats §9). Derived for display and export only; the sim
//! stores spin as the world-frame vector ω.

use bb_math::{Vec3, m};

/// Spin decomposition at release.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpinInfo {
    /// Transverse spin ω⊥ (rad/s vector).
    pub transverse: Vec3,
    /// Gyro component ω·v̂ (rad/s, signed).
    pub gyro: f64,
    /// |ω⊥| / |ω|; 0 for a ball with no spin.
    pub efficiency: f64,
    /// Unit Magnus direction normalize(ω⊥ × v̂); zero when there is no transverse spin.
    pub magnus_dir: Vec3,
}

pub fn decompose(w: Vec3, v: Vec3) -> SpinInfo {
    let vh = v.normalized();
    let gyro = w.dot(vh);
    let transverse = w - vh * gyro;
    let wn = w.norm();
    let efficiency = if wn > 0.0 { transverse.norm() / wn } else { 0.0 };
    SpinInfo { transverse, gyro, efficiency, magnus_dir: transverse.cross(vh).normalized() }
}

/// Savant `spin_axis` in degrees [0, 360): 180 = pure backspin, 0 = pure topspin (§9.2).
pub fn spin_axis_deg(w: Vec3, v: Vec3) -> f64 {
    let md = decompose(w, v).magnus_dir;
    let a = 180.0 + bb_math::units::rad_to_deg(m::atan2(-md.x, md.z));
    let a = a % 360.0;
    if a < 0.0 { a + 360.0 } else { a }
}

/// Clock-face tilt in hours (0, 12], pitcher's perspective, 12 = pure backspin (§9.3).
pub fn tilt_hours(spin_axis_deg: f64) -> f64 {
    let h = ((spin_axis_deg - 180.0) / 30.0) % 12.0;
    let h = if h < 0.0 { h + 12.0 } else { h };
    if h == 0.0 { 12.0 } else { h }
}

/// Format a tilt as `h:mm`, rounded to 15 minutes (§9.3).
pub fn tilt_display(hours: f64) -> String {
    let q = (hours * 4.0).round() as i64; // quarter hours
    let mut h = (q / 4) % 12;
    let mm = (q % 4) * 15;
    if h == 0 {
        h = 12;
    }
    format!("{h}:{mm:02}")
}

/// Build ω from a Savant spin axis, efficiency, and rate for a ball moving along `v`.
/// `gyro_sign` (+1/−1) picks the direction of the gyro component along v̂.
/// The Magnus direction is the unit vector ⟂ v̂ whose X–Z projection has the requested
/// spin-axis angle (inverse of [`spin_axis_deg`]); the transverse spin is (v̂ × m̂)·|ω⊥| so that ω⊥ × v̂ = m̂ |ω⊥|.
pub fn spin_from_axis(spin_axis_deg: f64, efficiency: f64, rate_radps: f64, gyro_sign: f64, v: Vec3) -> Vec3 {
    let vh = v.normalized();
    let phi = bb_math::units::deg_to_rad(spin_axis_deg - 180.0);
    // Choose m̂ ⟂ v̂ whose X–Z projection points at angle φ: m = (−sin φ, b, cos φ) with b
    // solving m·v̂ = 0. Needs a Y velocity component, which every pitch and throw toward the
    // plate has; otherwise fall back to the frame built around v̂.
    let (mx, mz) = (-m::sin(phi), m::cos(phi));
    let mdir = if vh.y.abs() > 1e-6 {
        Vec3::new(mx, -(mx * vh.x + mz * vh.z) / vh.y, mz).normalized()
    } else {
        let ez = Vec3::Z.reject(vh).normalized();
        let ex = ez.cross(vh);
        ex * mx + ez * mz
    };
    let eff = efficiency.clamp(0.0, 1.0);
    let wt = rate_radps * eff;
    let wg = rate_radps * m::sqrt((1.0 - eff * eff).max(0.0)) * gyro_sign.signum();
    vh.cross(mdir) * wt + vh * wg
}

#[cfg(test)]
mod tests {
    use super::*;

    const V: Vec3 = Vec3::new(0.0, -40.0, 0.0);

    #[test]
    fn savant_axis_checks() {
        // data-formats §9.2: backspin 180, topspin 0, RHP four-seam ≈ 210, LHP mirror ≈ 150.
        assert!((spin_axis_deg(Vec3::new(-200.0, 0.0, 0.0), V) - 180.0).abs() < 1e-9);
        assert!(spin_axis_deg(Vec3::new(200.0, 0.0, 0.0), V).abs() < 1e-9);
        // Magnus (−0.5, 0, 0.866): ω⊥ = v̂ × m̂.
        let rhp = Vec3::new(0.0, -1.0, 0.0).cross(Vec3::new(-0.5, 0.0, 0.75f64.sqrt())) * 230.0;
        assert!((spin_axis_deg(rhp, V) - 210.0).abs() < 1e-9);
        let lhp = Vec3::new(0.0, -1.0, 0.0).cross(Vec3::new(0.5, 0.0, 0.75f64.sqrt())) * 230.0;
        assert!((spin_axis_deg(lhp, V) - 150.0).abs() < 1e-9);
        assert!((tilt_hours(210.0) - 1.0).abs() < 1e-12);
        assert!((tilt_hours(150.0) - 11.0).abs() < 1e-12);
        assert_eq!(tilt_hours(180.0), 12.0);
        assert_eq!(tilt_display(tilt_hours(210.0)), "1:00");
        assert_eq!(tilt_display(tilt_hours(150.0)), "11:00");
        assert_eq!(tilt_display(tilt_hours(0.0)), "6:00");
    }

    #[test]
    fn round_trip_axis_and_efficiency() {
        let v = Vec3::new(1.5, -40.0, -2.0); // slightly off-axis release
        for &axis in &[0.0, 45.0, 135.0, 180.0, 210.0, 275.0, 330.0] {
            for &eff in &[1.0, 0.9, 0.5, 0.2] {
                for &gs in &[1.0, -1.0] {
                    let w = spin_from_axis(axis, eff, 250.0, gs, v);
                    let back = spin_axis_deg(w, v);
                    let d = ((back - axis + 540.0) % 360.0) - 180.0;
                    assert!(d.abs() < 1e-9, "axis {axis} -> {back}");
                    let info = decompose(w, v);
                    assert!((info.efficiency - eff).abs() < 1e-12);
                    assert!((w.norm() - 250.0).abs() < 1e-9);
                    if eff < 1.0 {
                        assert_eq!(info.gyro.signum(), gs);
                    }
                }
            }
        }
    }

    #[test]
    fn backspin_pitch_is_minus_x() {
        // physics-spec §1.3: a pitch moving −Y with backspin has ω ∥ −X.
        let w = spin_from_axis(180.0, 1.0, 1.0, 1.0, V);
        assert!((w - Vec3::new(-1.0, 0.0, 0.0)).norm() < 1e-12);
    }
}
