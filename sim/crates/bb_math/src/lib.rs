//! Foundation math for the sim: SI vector types, unit conversions, deterministic
//! transcendental wrappers, and the keyed counter-based RNG.
//!
//! Determinism rules (physics-spec §5.4, PLAN §4.2): every transcendental goes through
//! [`m`] (the `libm` crate), and nothing here uses `mul_add` or SIMD intrinsics.

pub mod m;
pub mod mat3;
pub mod rng;
pub mod units;
pub mod vec3;

pub use mat3::Mat3;
pub use rng::{KeyedRng, Purpose, RngKey};
pub use vec3::Vec3;
