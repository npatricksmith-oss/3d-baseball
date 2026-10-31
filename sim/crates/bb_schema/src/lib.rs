//! FlatBuffers schemas (data-formats §3–§7) and the event-log container (§4.1, §4.4).
//!
//! The `generated` module is produced by `sim/schemas/gen.sh` from `sim/schemas/*.fbs` and committed.
//! Never edit it by hand.

#[allow(unsafe_code, unused_imports, dead_code, clippy::all, mismatched_lifetime_syntaxes, non_camel_case_types)]
#[rustfmt::skip]
mod generated;

pub use generated::bb::*;

pub mod log;

/// Shared schema version for all six schema files (data-formats §8).
pub const SCHEMA_MAJOR: u16 = 0;
pub const SCHEMA_MINOR: u16 = 1;

/// Convert between `bb_math`-free plain arrays and the schema structs.
pub fn vec3d(a: [f64; 3]) -> Vec3d {
    Vec3d::new(a[0], a[1], a[2])
}
pub fn vec3f(a: [f32; 3]) -> Vec3f {
    Vec3f::new(a[0], a[1], a[2])
}
