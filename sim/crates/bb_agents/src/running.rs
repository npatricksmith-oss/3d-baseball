//! Runners' starts (agent-models §6.4): parameters only; the live simulator runs the legs
//! and the runner's decision from the same start model (`bb_game::live`).

/// League-average runner's top speed, m/s: Statcast's 27.0 ft/s sprint speed is 8.23 m/s
/// (8.2 is 26.9 ft/s). The game initializes runners to this speed, and fielders use it
/// for runners they do not know.
pub const LEAGUE_TOP_SPEED_MPS: f64 = 8.2;
/// League-average time constant of a runner's acceleration, s [E]: v = v_max(1 − e^(−t/τ)).
/// Set so the league-average runner covers a standing 90 ft in 4.10 s, the owner's target
/// (Q1, 2026-11-14). The published figure is a 4.03 s mean for runners on third in
/// sacrifice-fly situations, who are faster than average; how much slower the average
/// runner is has no verified source (docs/calibration/2026-11-15-runner-acceleration.md §2).
/// The game initializes each runner's `RunnerIn::tau_s` to this value.
pub const LEAGUE_TAU_S: f64 = 0.76;

/// A runner who has tagged up: set on his bag, watching the catch, leaving on it
/// (agent-models §6.4; values from docs/calibration/2026-11-14-tag-ups.md).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TagUpParams {
    /// From the catch to his first step, s [E]: 0. He is watching the ball arrive and the
    /// rolling-start technique is built to cut his reaction to about nothing (note §1); the
    /// Statcast tag-up clock starts at the catch.
    pub react_s: f64,
    /// His set start's time constant as a multiple of his own τ [E]: 1, his ordinary start.
    /// A rolling start's gain has no published measurement, so it is set to nothing (note
    /// §1, §7); a measured gain would lower this.
    pub tau_gain: f64,
    /// His belief of how long a fielder who has the ball at the base needs to put the tag
    /// on him, s [E]: the quoted 0.25 s (the fielder's actual time is
    /// `FielderAttrs::tag_apply_s`).
    pub typical_tag_s: f64,
}

impl Default for TagUpParams {
    fn default() -> Self {
        Self { react_s: 0.0, tau_gain: 1.0, typical_tag_s: 0.25 }
    }
}

// ---------------------------------------------------------------------------------------------
// The advance decision by run value (agent-models §6.3; sources, arithmetic and the owner's
// rulings Q2, Q8, Q9 in docs/calibration/2026-11-15-runner-decision-model.md).
// ---------------------------------------------------------------------------------------------

/// Run expectancy to the end of the half-inning, by the bases occupied (bit 0 first, bit 1
/// second, bit 2 third) and the outs: the 2021–24 major-league average, first eight innings
/// (Clemens, FanGraphs, 29 May 2025; note §1.5). A belief table every runner holds
/// (agent-models §8, `RE`); the world never reads it.
pub const RUN_EXPECTANCY: [[f64; 3]; 8] = [
    [0.50, 0.27, 0.10], // none
    [0.90, 0.54, 0.23], // 1--
    [1.14, 0.71, 0.33], // -2-
    [1.51, 0.94, 0.46], // 12-
    [1.37, 0.98, 0.38], // --3
    [1.82, 1.19, 0.51], // 1-3
    [2.04, 1.41, 0.57], // -23
    [2.38, 1.63, 0.82], // 123
];

/// Expected runs from here to the end of the half-inning; nothing with three out.
pub fn run_expectancy(bases: u8, outs: u8) -> f64 {
    if outs >= 3 { 0.0 } else { RUN_EXPECTANCY[(bases & 7) as usize][outs as usize] }
}

/// What a runner weighs: the value (runs) of staying, of going and being safe, and of going
/// and being out. Runs already home on the play are in all three or in none.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdvanceValues {
    pub hold: f64,
    pub safe: f64,
    pub out: f64,
}

impl AdvanceValues {
    /// The chance of being safe at which going and staying are worth the same:
    /// (hold − out) / (safe − out). Above 1 (or with nothing to gain) he should never go.
    pub fn break_even(&self) -> f64 {
        let gain = self.safe - self.hold;
        let cost = self.hold - self.out;
        if gain <= 0.0 {
            return f64::INFINITY;
        }
        if cost <= 0.0 {
            return 0.0;
        }
        cost / (gain + cost)
    }

    /// The same with the gain worth `w` times as much (`run_weight`): the scoreboard changes
    /// what the extra base is worth against the out.
    pub fn weighted(&self, w: f64) -> Self {
        Self { safe: self.hold + w * (self.safe - self.hold), ..*self }
    }
}

/// What the extra base is worth against the out, by the scoreboard [E]: 1 is the run table's
/// own trade. `margin` is the batting team's runs less the fielding team's before the play.
///
/// Shape from Silver's marginal win value of a run after the top of the seventh (Baseball
/// Prospectus, 4 May 2005; note §1.5): 24.1% tied or one behind, 11.9% up one or down two,
/// 6.6% up two or down three, 3.6%, 1.9%. The weight is the next run's value over the mean
/// of the two after it: tied 24.1 / 9.3 = 2.6, one behind 24.1 / 18.0 = 1.3, two behind
/// 11.9 / 24.1 = 0.5, three or more behind about 0.4 (a team far behind needs the big
/// inning: do not risk the out); ahead it is left at the run table's trade but for a
/// one-run lead (11.9 / 5.1 = 2.3, taken as 1.5: the insurance run). Early innings count
/// the scoreboard for nothing, the middle innings for half (in the exponent); Silver's
/// first-run-over-second ratio is 1.09 in the first and 3.64 in the eighth. Estimates until
/// the learned win-expectancy table exists (owner's Q9 (a); agent-models §8).
pub fn run_weight(inning: u8, margin: i32) -> f64 {
    let late = match margin {
        i32::MIN..=-3 => RUN_WEIGHT_FAR_BEHIND,
        -2 => RUN_WEIGHT_TWO_BEHIND,
        -1 => RUN_WEIGHT_ONE_BEHIND,
        0 => RUN_WEIGHT_TIED,
        1 => RUN_WEIGHT_ONE_AHEAD,
        _ => 1.0,
    };
    match inning {
        0..=3 => 1.0,
        4..=6 => bb_math::m::sqrt(late),
        _ => late,
    }
}
pub const RUN_WEIGHT_FAR_BEHIND: f64 = 0.4;
pub const RUN_WEIGHT_TWO_BEHIND: f64 = 0.5;
pub const RUN_WEIGHT_ONE_BEHIND: f64 = 1.3;
pub const RUN_WEIGHT_TIED: f64 = 2.6;
pub const RUN_WEIGHT_ONE_AHEAD: f64 = 1.5;

/// How a runner turns his read of a race into a decision (agent-models §6.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdvanceParams {
    /// The spread he believes his read of a race has, s [E]: his timing read (0.15 s, the
    /// noise his reads actually carry), the fielder's execution (a bobble, a throw off
    /// line) and his own legs, taken together. P̂(safe) = Φ(margin / σ).
    pub sigma_race_s: f64,
    /// Caution: how many of those spreads of cushion he wants beyond the break-even [E].
    /// Major-league runners are safe 94–97% of the time they go, at every number of outs,
    /// though the run table's break-evens run from 34% to 94% (note §1.5): they are more
    /// careful than the table. Provisional: 1.0 puts outs on the bases at the league's 0.27
    /// a team-game; the fit to the league's attempt and success rates (owner's Q8 (c)) waits
    /// for the race itself to be right (note §9). A team's and a runner's aggressiveness
    /// lower it, timidity raises it.
    pub caution_z: f64,
    /// Caution for the send home with two out [E] (agent-models §6.3, the third-base coach's
    /// two-out send; owner's Q38 (a)): with two out the coach decides before the pitch to
    /// send the runner home and holds him only when he is clearly beaten (Guilliams,
    /// decision-model note §1.6). It replaces `caution_z` when the runner who started the play
    /// on second decides on home with two out on the board as he sees them. −0.2, fitted to
    /// Retrosheet 2024's two-out sends from second on outfield singles at the corners: 92.1%
    /// sent, 3.9% of those sent put out (left 91.3% and 5.6%, right 92.9% and 2.2%; center
    /// 96.2%, which the sim already matched; `docs/calibration/2026-11-23-two-out-send.md`).
    /// Below zero: he goes on a race he believes slightly lost, as the coach's "send unless
    /// clearly beaten" does. An outcome fit: values from about 0.1 to −0.3 fit within noise,
    /// and his read at third of a charging outfielder's throw is about 0.2 s hopeful (note
    /// §2a), so a corrected read needs a new fit (Q38 review P2-1, P3-3).
    pub send_two_out_z: f64,
}

impl Default for AdvanceParams {
    fn default() -> Self {
        Self { sigma_race_s: 0.20, caution_z: 1.0, send_two_out_z: -0.2 }
    }
}

impl AdvanceParams {
    /// The parameters for a runner who started the play on base `start` deciding to go on to
    /// base `next` (4: home) with `outs` on the board as he sees them: the two-out send's
    /// caution in place of `caution_z` for the runner from second going home.
    pub fn for_decision(&self, start: u8, next: u8, outs: u8) -> Self {
        if start == 2 && next == 4 && outs >= 2 { Self { caution_z: self.send_two_out_z, ..*self } } else { *self }
    }

    /// His belief that he is safe with `margin_s` in hand (the ball's believed arrival, tag
    /// included, less his own).
    pub fn p_safe(&self, margin_s: f64) -> f64 {
        phi(margin_s / self.sigma_race_s)
    }

    /// Whether he goes on, standing on a base: `chooses_on` with the way back certain.
    pub fn goes(&self, margin_s: f64, values: &AdvanceValues) -> bool {
        self.chooses_on(margin_s, f64::INFINITY, values)
    }

    /// Whether he takes the base ahead rather than the one behind (the one he stands on, or
    /// the one he left: then `back_margin_s` is his margin getting back to it). Each chance
    /// of being safe is judged with his cushion taken off, and the better expected value
    /// wins; standing on a base this is "his cautious chance beats the break-even". The
    /// cushion is in spreads of the race, not in probability, so a sure thing is always
    /// taken when it gains anything and a break-even near 1 does not become "never".
    pub fn chooses_on(&self, on_margin_s: f64, back_margin_s: f64, values: &AdvanceValues) -> bool {
        self.chooses_between(on_margin_s, back_margin_s, values, self.caution_z, self.caution_z)
    }

    /// [`AdvanceParams::chooses_on`] with its own caution for each way: the two-out send's
    /// for the way home, the general one for the way back to third (review 2026-11-23, P3-1:
    /// the coach's send makes going home look better, not the return).
    pub fn chooses_between(&self, on_margin_s: f64, back_margin_s: f64, values: &AdvanceValues, caution_on: f64, caution_back: f64) -> bool {
        let p_on = phi(on_margin_s / self.sigma_race_s - caution_on);
        let p_back = if back_margin_s == f64::INFINITY { 1.0 } else { phi(back_margin_s / self.sigma_race_s - caution_back) };
        p_on * values.safe + (1.0 - p_on) * values.out > p_back * values.hold + (1.0 - p_back) * values.out
    }
}

/// The standard normal distribution function.
pub fn phi(z: f64) -> f64 {
    0.5 * (1.0 + bb_math::m::erf(z / core::f64::consts::SQRT_2))
}

#[cfg(test)]
mod advance_tests {
    use super::*;

    fn re(b: u8, o: u8, runs: f64) -> f64 {
        run_expectancy(b, o) + runs
    }

    // Note §1.5's break-even table (column A), computed there by an independent script from
    // the same run table, and agreeing with the published values it lists (Malter 41.3% and
    // 33.4%, Olszewski 75.1% and 32.5%).
    #[test]
    fn break_evens_of_the_note() {
        let be = |hold, safe, out| AdvanceValues { hold, safe, out }.break_even();
        // First to third on a single: hold 12-, safe 1-3, out leaves the batter on first.
        for (o, want) in [(0u8, 0.758), (1, 0.740), (2, 0.902)] {
            assert!((be(re(0b011, o, 0.0), re(0b101, o, 0.0), re(0b001, o + 1, 0.0)) - want).abs() < 0.0015, "1st to 3rd, {o} out");
        }
        // Second to home on a single: hold 1-3, safe a run and a man on first.
        for (o, want) in [(0u8, 0.941), (1, 0.733), (2, 0.415)] {
            assert!((be(re(0b101, o, 0.0), re(0b001, o, 1.0), re(0b001, o + 1, 0.0)) - want).abs() < 0.0015, "2nd to home, {o} out");
        }
        // First to home on a double.
        for (o, want) in [(0u8, 0.930), (1, 0.783), (2, 0.429)] {
            assert!((be(re(0b110, o, 0.0), re(0b010, o, 1.0), re(0b010, o + 1, 0.0)) - want).abs() < 0.0015, "1st to home, {o} out");
        }
        // Tag from third, alone, o outs before the fly.
        for (o, want) in [(0u8, 0.752), (1, 0.345)] {
            assert!((be(re(0b100, o + 1, 0.0), re(0, o + 1, 1.0), re(0, o + 2, 0.0)) - want).abs() < 0.0015, "tag from third, {o} out");
        }
    }

    #[test]
    fn nothing_to_gain_never_goes_and_nothing_to_lose_always() {
        let p = AdvanceParams::default();
        assert!(!p.goes(10.0, &AdvanceValues { hold: 1.0, safe: 1.0, out: 0.0 }));
        assert!(p.goes(-0.2, &AdvanceValues { hold: 0.5, safe: 1.0, out: 0.5 }));
        // Between bases with no way back, any chance ahead is the better one.
        assert!(p.chooses_on(-0.1, -1.0, &AdvanceValues { hold: 1.0, safe: 1.2, out: 0.3 }));
        assert!(!p.chooses_on(-1.0, -0.1, &AdvanceValues { hold: 1.0, safe: 1.2, out: 0.3 }));
        // A sure thing that gains anything is taken, however dear the out.
        assert!(p.goes(5.0, &AdvanceValues { hold: 1.82, safe: 1.83, out: 0.0 }));
    }

    // The same race is run with two out and not with none (second to home on a single:
    // break-even 41% against 94%), and far behind late it is not run at all.
    #[test]
    fn outs_and_score_move_the_decision() {
        let p = AdvanceParams::default();
        let v = |o: u8| AdvanceValues { hold: run_expectancy(0b101, o), safe: run_expectancy(0b001, o) + 1.0, out: run_expectancy(0b001, o + 1) };
        let margin = 0.25; // seconds in hand, by his read
        assert!(p.goes(margin, &v(2)) && !p.goes(margin, &v(0)));
        // Five behind in the eighth the same two-out race is no longer worth the out.
        assert!(!p.goes(margin, &v(2).weighted(run_weight(8, -5))));
        assert!(v(1).weighted(run_weight(9, 0)).break_even() < v(1).break_even());
        assert!(v(1).weighted(run_weight(8, -4)).break_even() > v(1).break_even());
        assert_eq!(run_weight(2, 0), 1.0);
        assert!(run_weight(5, 0) > 1.0 && run_weight(5, 0) < run_weight(8, 0));
    }

    // Q38 (a): with two out the runner from second is sent home unless clearly beaten. His
    // read of a race 0.15 s behind the ball alone is 0.10 s in hand once the catcher's tag
    // (0.25 s) is counted. The general caution makes that a 31% chance, clearly under the
    // two-out break-even of 41.5% (note §1.5; it wants about 0.16 s in hand, runner-start note
    // §1), so it held him; the two-out send makes it 76% and takes him; with one out
    // (break-even 73%) the same race still holds him. 0.7 s behind the ball alone (−0.45 s with
    // the tag) he is beaten and holds. The loop at the end only documents the scope: first to
    // third, first to home, the batter-runner and second to third keep the general caution.
    #[test]
    fn the_two_out_send_from_second_takes_the_close_race() {
        let p = AdvanceParams::default();
        let v = |o: u8| AdvanceValues { hold: run_expectancy(0b101, o), safe: run_expectancy(0b001, o) + 1.0, out: run_expectancy(0b001, o + 1) };
        let close = 0.25 - 0.15;
        assert!(!p.goes(close, &v(2)), "the general caution holds him");
        assert!(p.for_decision(2, 4, 2).goes(close, &v(2)), "the two-out send takes him");
        assert!(!p.for_decision(2, 4, 1).goes(close, &v(1)), "one out: he holds");
        assert!(!p.for_decision(2, 4, 2).goes(0.25 - 0.7, &v(2)), "clearly beaten: he holds");
        for (start, next) in [(1u8, 3u8), (1, 4), (0, 2), (2, 3)] {
            assert_eq!(p.for_decision(start, next, 2), p, "{start} to {next} keeps the general caution");
        }
    }

    // Review P3-1: between third and home with two out, the send's caution is on the way home
    // only. Worked by hand from the run table (two out: hold 1-3 0.51, safe a run and a man on
    // first 1.23, out 0): home believed 0.2 s short (margin −0.2 s), third 0.1 s in hand.
    // Home: Φ(−1 + 0.2) = 0.212, worth 0.26. Third at the general caution: Φ(0.5 − 1) = 0.309,
    // worth 0.16: he goes on. Had the way back taken the send's −0.2 too, Φ(0.7) = 0.758,
    // worth 0.39: he would have turned back, the send making the return look safer.
    #[test]
    fn the_two_out_send_does_not_make_the_way_back_look_safer() {
        let p = AdvanceParams::default();
        let v = AdvanceValues { hold: run_expectancy(0b101, 2), safe: run_expectancy(0b001, 2) + 1.0, out: 0.0 };
        assert!((v.hold - 0.51).abs() < 1e-9 && (v.safe - 1.23).abs() < 1e-9);
        let z = p.send_two_out_z;
        assert!(p.chooses_between(-0.2, 0.1, &v, z, p.caution_z), "the send on the way home only: he goes on");
        assert!(!p.chooses_between(-0.2, 0.1, &v, z, z), "the send on both ways: he turns back");
        assert!(!p.chooses_on(-0.2, 0.1, &v), "the general caution both ways: he turns back");
    }

    #[test]
    fn phi_values() {
        assert!((phi(0.0) - 0.5).abs() < 1e-12);
        assert!((phi(1.959964) - 0.975).abs() < 1e-6);
        assert!((phi(-1.0) - 0.158655).abs() < 1e-6);
    }
}
