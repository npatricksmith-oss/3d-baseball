//! Pure physical models (PLAN §6, physics-spec.md): ball flight, atmosphere, spin
//! conventions, strike-zone geometry, bat profile, and the collision impulse solver.
//!
//! Nothing here rolls dice (D1). The only world-side randomness, bounce-normal tilt
//! (physics-spec §9.3), is passed in by the caller as an explicit perturbation.

pub mod atmosphere;
pub mod ball;
pub mod bat;
pub mod collision;
pub mod contact;
pub mod flight;
pub mod pitch;
pub mod spin;
pub mod zone;

pub use ball::{AeroParams, BallParams};
pub use flight::{BallState, FlightEnv, Knot, Trajectory};
