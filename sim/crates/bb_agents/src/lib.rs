//! Players and managers (PLAN §8, agent-models.md).
//!
//! Agents see only `bb_observe` types and predict with `bb_physics`; this crate cannot depend
//! on `bb_world` (PLAN §15), which makes the belief/world split of D1 a compile-time rule.
//! Execution noise lives in [`motor`]; decision code never receives an `Executed*` value.
//! `motor` is `pub`, so Rust itself does not stop a sibling module from naming it;
//! `tests/exec_isolation.rs` scans the sources of every other module and fails if one does.

pub mod attributes;
pub mod batter;
pub mod motor;
pub mod pitcher;
pub mod running;
pub mod swing;
pub mod throwing;
