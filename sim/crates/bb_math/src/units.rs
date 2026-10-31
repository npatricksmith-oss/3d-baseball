//! Unit conversions (physics-spec §1.1). Imperial units exist only at config, UI, and report
//! boundaries; inside the sim everything is SI.

use core::f64::consts::PI;

pub const MPH: f64 = 0.44704; // m/s per mph
pub const FT: f64 = 0.3048; // m per ft
pub const INCH: f64 = 0.0254; // m per in
pub const RPM: f64 = PI / 30.0; // rad/s per rpm
pub const OZ: f64 = 0.028349523; // kg per oz
pub const DEG: f64 = PI / 180.0; // rad per degree

/// Standard gravity, m/s² [S].
pub const G: f64 = 9.80665;

#[inline]
pub fn mph_to_mps(x: f64) -> f64 {
    x * MPH
}
#[inline]
pub fn mps_to_mph(x: f64) -> f64 {
    x / MPH
}
#[inline]
pub fn ft_to_m(x: f64) -> f64 {
    x * FT
}
#[inline]
pub fn m_to_ft(x: f64) -> f64 {
    x / FT
}
#[inline]
pub fn in_to_m(x: f64) -> f64 {
    x * INCH
}
#[inline]
pub fn m_to_in(x: f64) -> f64 {
    x / INCH
}
#[inline]
pub fn rpm_to_radps(x: f64) -> f64 {
    x * RPM
}
#[inline]
pub fn radps_to_rpm(x: f64) -> f64 {
    x / RPM
}
#[inline]
pub fn deg_to_rad(x: f64) -> f64 {
    x * DEG
}
#[inline]
pub fn rad_to_deg(x: f64) -> f64 {
    x / DEG
}
#[inline]
pub fn f_to_c(x: f64) -> f64 {
    (x - 32.0) * 5.0 / 9.0
}
