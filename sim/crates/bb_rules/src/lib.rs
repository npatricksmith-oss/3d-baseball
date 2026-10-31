//! Rules engine (rules-spec.md): a pure, deterministic step function over facts reported by
//! the world. No randomness, no physics, no agents (D1, PLAN §15).
//!
//! `step(state, input) → events`. The scenario corpus (`scenarios/**/*.toml`) drives it with
//! hand-written facts.
//!
//! **M4-skeleton scope (2026-10-31).** Implemented: pitch judgment and counts (§4.4–§4.5),
//! walks/HBP with forced awards (§7.1), fair/foul (§5.2), catches with retouch and the
//! automatic leave-early appeal (§5.3, §5.6), forces and force removal (§5.4), tags (§5.5),
//! run counting with time plays (§6.4), home runs and ground-rule doubles (§7.2), play end
//! (§5.8 settle), half-inning and game end including walk-offs (§4.1). Deferred: uncaught
//! third strikes and caught foul tips (need the catcher model), steals, interference
//! (§5.7), overruns of first, two-runners-on-a-base, passing runners, substitutions (§9),
//! the official scorer (§10).

mod engine;
mod state;

pub use engine::step;
pub use state::*;
