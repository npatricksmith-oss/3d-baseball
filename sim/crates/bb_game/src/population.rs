//! Abilities of a generated game's players (data-formats §2.4 step 5; calibration-targets
//! §11.1). Interim until the league generator (M7): only `BatSpeed` is drawn (owner ruling D19,
//! 2026-11-14); every other ability stays at grade 50.

use bb_agents::attributes::{BAT_SPEED_MU_MPS, BAT_SPEED_SIGMA_POP_MPS, linear_from_grade};
use bb_math::{KeyedRng, Purpose, RngKey};
use bb_rules::AgentId;

/// A latent grade's z is truncated at ±3, so grades run 20–80 (calibration-targets §11.1) [C].
pub const GRADE_Z_TRUNC: f64 = 3.0;

/// The population a game's players are drawn from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Population {
    /// σ_pop of `BatSpeed`, m/s per 10 grade points. 0 gives every batter grade 50.
    pub bat_speed_sigma_mps: f64,
}

impl Default for Population {
    fn default() -> Self {
        Self { bat_speed_sigma_mps: BAT_SPEED_SIGMA_POP_MPS }
    }
}

impl Population {
    /// Everyone at grade 50 (the league before D19; labs and counterfactuals).
    pub fn average() -> Self {
        Self { bat_speed_sigma_mps: 0.0 }
    }

    /// The `BatSpeed` grade of the player whose fielding id is `field_agent` in the game with
    /// this seed (data-formats §2.4 step 5): stream `RosterAbilities`, `sub_index` 0, first draw.
    pub fn bat_speed_grade(seed: u64, field_agent: AgentId) -> f64 {
        let mut rng = KeyedRng::new(RngKey { game_seed: seed, play_index: 0, agent_id: field_agent, purpose: Purpose::RosterAbilities, sub_index: 0 });
        50.0 + 10.0 * rng.truncated_normal(GRADE_Z_TRUNC)
    }

    /// His `BatSpeed`, m/s.
    pub fn bat_speed(&self, seed: u64, field_agent: AgentId) -> f64 {
        linear_from_grade(Self::bat_speed_grade(seed, field_agent), BAT_SPEED_MU_MPS, self.bat_speed_sigma_mps)
    }
}
