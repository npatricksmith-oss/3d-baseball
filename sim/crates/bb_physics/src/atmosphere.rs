//! Air density (physics-spec §3), the Trajectory Calculator formula via `pybbda` [S, verify: R1].

use bb_math::m;
use bb_math::units::{FT, f_to_c};

/// Weather and site inputs, in the calculator's units (config boundary).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weather {
    pub temp_f: f64,
    /// Sea-level-referenced barometric reading, inHg.
    pub pressure_inhg: f64,
    pub rh_pct: f64,
    pub elev_ft: f64,
}

impl Default for Weather {
    /// The calculator's default environment: 70 °F, 29.92 inHg, 50% RH, 15 ft.
    fn default() -> Self {
        Self { temp_f: 70.0, pressure_inhg: 29.92, rh_pct: 50.0, elev_ft: 15.0 }
    }
}

const BETA_PER_M: f64 = 1.217e-4;

/// Air density, kg/m³.
pub fn air_density(w: &Weather) -> f64 {
    let tc = f_to_c(w.temp_f);
    let p_mmhg = w.pressure_inhg * 25.4;
    let elev_m = w.elev_ft * FT;
    let svp = 4.5841 * m::exp((18.687 - tc / 234.5) * tc / (257.14 + tc));
    1.2929 * (273.0 / (tc + 273.0)) * (p_mmhg * m::exp(-BETA_PER_M * elev_m) - 0.3783 * w.rh_pct * svp / 100.0)
        / 760.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculator_default_density() {
        let rho = air_density(&Weather::default());
        assert!((rho - 1.1938).abs() < 5e-5, "rho = {rho}");
    }

    #[test]
    fn denser_when_cold_thinner_when_high() {
        let base = air_density(&Weather::default());
        assert!(air_density(&Weather { temp_f: 40.0, ..Default::default() }) > base);
        let denver = air_density(&Weather { elev_ft: 5280.0, ..Default::default() });
        assert!(denver < 0.85 * base && denver > 0.78 * base, "denver {denver}");
    }
}
