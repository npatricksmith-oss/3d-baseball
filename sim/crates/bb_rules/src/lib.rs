//! Rules engine (rules-spec.md): a pure, deterministic step function over facts reported by
//! the world. No randomness, no physics, no agents (D1, PLAN §15).
//!
//! `step(state, input) → events`, where the input is an [`Input`] fact or a
//! [`RunnerPosition`] sample. The scenario corpus (`scenarios/**/*.toml`) drives it with
//! hand-written facts.
//!
//! The engine judges pitches, counts, fair and foul balls, catches and retouches, force
//! plays, tags, overruns of first, two runners on a base, and passing. It handles forced
//! awards, home runs, ground-rule doubles, time plays, walk-offs, and inning and game ends.
//! The scorer in `bb_stats` classifies the resulting events (rules-spec §10).

mod engine;
mod state;

pub use engine::step;
pub use state::*;
