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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;
    use crate::roster::Rosters;

    const MPH: f64 = 1.0 / 0.44704;

    /// Seed 7's eighteen bat speeds, [side][position − 1], from the independent stdlib-Python
    /// generator written from data-formats §2.4 step 5 (`analysis/oracle/roster_oracle.py
    /// batspeed 7`: its own Philox4x32-10, key layout and Box–Muller). A change to the stream,
    /// the draw or the mapping breaks this.
    #[test]
    fn seed_7_bat_speeds_match_the_oracle() {
        let want = [
            [33.877951779155, 34.936996705716, 34.027854162417, 30.623577521801, 33.956454381771, 31.925139230348, 30.564962319325, 31.821108493760, 30.858394154139],
            [33.228535128360, 33.275330650712, 31.837004273530, 31.109388911274, 31.484850131259, 32.243743848490, 31.409776315828, 32.847105558285, 30.864795884488],
        ];
        let pop = Population::default();
        for side in 0..2u16 {
            for pos in 1..=9u16 {
                let got = pop.bat_speed(7, (side << 8) | 0x80 | pos);
                assert!((got - want[side as usize][pos as usize - 1]).abs() < 1e-9, "side {side} position {pos}: {got}");
            }
        }
    }

    /// The league the draw makes, against Statcast's 2025 sample (agent-models §10, D19): mean of
    /// batters' mean competitive bat speed 71.7 mph (grade 50 = 71.58), SD of true means 2.81 mph
    /// (2.77 after the ±3σ truncation), and no batter outside grades 20–80 (63.2–80.0 mph).
    #[test]
    fn the_league_has_statcasts_bat_speed_spread() {
        let pop = Population::default();
        let v: Vec<f64> = (0..2000u64).flat_map(|seed| (0..2u16).flat_map(move |s| (1..=9u16).map(move |p| (seed, (s << 8) | 0x80 | p)))).map(|(seed, a)| pop.bat_speed(seed, a) * MPH).collect();
        let n = v.len() as f64;
        let mean = v.iter().sum::<f64>() / n;
        let sd = (v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / n).sqrt();
        // 36,000 draws: the mean's standard error is 0.015 mph, the SD's 0.010.
        assert!((mean - 71.6).abs() < 0.1, "mean {mean:.2} mph");
        assert!((sd - 2.77).abs() < 0.06, "SD {sd:.3} mph (Statcast 2.81 before truncation)");
        assert!(v.iter().all(|&x| (63.1..=80.1).contains(&x)), "a batter outside grades 20–80");
    }

    /// Without a spread every batter is the grade-50 batter of the league before D19, and each
    /// batting slot carries the draw of the player batting there.
    #[test]
    fn each_slot_bats_with_its_players_draw() {
        for seed in [0u64, 7, 1000] {
            let flat = Game::with_population(seed, false, &Population::average());
            let game = Game::new(seed, false);
            let r = Rosters::generate(seed);
            for side in 0..2 {
                for (slot, p) in r.teams[side].batting_order().iter().enumerate() {
                    assert_eq!(flat.bat_speed(side, slot), BAT_SPEED_MU_MPS);
                    assert_eq!(game.bat_speed(side, slot), Population::default().bat_speed(seed, r.teams[side].field_agent(p)));
                }
            }
        }
    }
}
