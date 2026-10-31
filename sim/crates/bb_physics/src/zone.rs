//! Plate crossing and the ABS-style strike zone (physics-spec §10, D11).

use bb_math::units::INCH;

/// Judgment plane: midpoint of the plate's 17-in depth, m [S].
pub const Y_JUDGE: f64 = 8.5 * INCH;
/// Front edge of the plate (Statcast `plate_x`/`plate_z` reference), m [S].
pub const Y_FRONT: f64 = 17.0 * INCH;
/// Half-width of the plate, m [S].
pub const HALF_WIDTH: f64 = 8.5 * INCH;

/// Zone fractions of batter standing height (league rules config; 2026 ABS values).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZoneRule {
    pub bottom_frac: f64,
    pub top_frac: f64,
}

impl Default for ZoneRule {
    fn default() -> Self {
        Self { bottom_frac: 0.27, top_frac: 0.535 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Zone {
    pub bottom: f64,
    pub top: f64,
    pub half_width: f64,
}

impl Zone {
    pub fn for_batter(height_m: f64, rule: &ZoneRule) -> Zone {
        Zone { bottom: rule.bottom_frac * height_m, top: rule.top_frac * height_m, half_width: HALF_WIDTH }
    }

    /// Strike iff the ball's cross-section circle (radius `ball_r`) centered at (x, z)
    /// overlaps the zone rectangle (rounded-corner test).
    pub fn is_strike(&self, x: f64, z: f64, ball_r: f64) -> bool {
        let cx = x.clamp(-self.half_width, self.half_width);
        let cz = z.clamp(self.bottom, self.top);
        let dx = x - cx;
        let dz = z - cz;
        dx * dx + dz * dz <= ball_r * ball_r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounded_corner_overlap() {
        let z = Zone::for_batter(1.85, &ZoneRule::default());
        let r = 0.0369;
        assert!(z.is_strike(0.0, 0.75, r));
        // Just touching the side edge.
        assert!(z.is_strike(HALF_WIDTH + r * 0.999, 0.75, r));
        assert!(!z.is_strike(HALF_WIDTH + r * 1.001, 0.75, r));
        // Corner: diagonal offset within r counts, beyond does not.
        let d = r * 0.7;
        assert!(z.is_strike(HALF_WIDTH + d, z.top + d, r)); // distance 0.99 r
        assert!(!z.is_strike(HALF_WIDTH + 0.75 * r, z.top + 0.75 * r, r)); // 1.06 r
    }
}
