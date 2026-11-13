//! The umpire's visible out calls (rules-spec §1.3): when, after the out, the umpire signals
//! it. Presentation and timing only: the out itself is the rules engine's, made the instant
//! its deciding fact arrived, and nothing here feeds back into the game.

use bb_math::Vec3;
use bb_rules::{AgentId, Base};

/// Delay from the out to the umpire's signal on a bang-bang play, s [E]. On a close play the
/// umpire is set, watching the bag, and calls it at once.
pub const T_CALL_CLOSE_S: f64 = 0.2;
/// Delay on a routine play, s [E]. Under a second: the umpire lets the play finish (the ball
/// held, the throw secure) and signals without hurry.
pub const T_CALL_ROUTINE_S: f64 = 0.7;
/// Margins up to this are bang-bang plays, s [E].
pub const CLOSE_MARGIN_S: f64 = 0.15;
/// Margins from this on are routine, s [E]; the delay is linear in between.
pub const ROUTINE_MARGIN_S: f64 = 0.65;

/// One out as the umpire calls it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutCall {
    /// When the out was made (the deciding fact's time), s from release.
    pub t_out: f64,
    /// The runner retired.
    pub runner: AgentId,
    /// Where: the base the play was on (0: the batter at the plate, a caught fly or a strikeout).
    pub base: Base,
    /// How close the play was, s: the runner's time short of the base he needed (∞ for a
    /// routine catch). 0 is a bang-bang play.
    pub margin_s: f64,
    /// The runner's position at the out (sim frame, m).
    pub at: Vec3,
}

impl OutCall {
    /// When the umpire signals the out, s from release.
    pub fn t_call(&self) -> f64 {
        self.t_out + call_delay(self.margin_s)
    }
}

/// Delay from an out to the umpire's signal, from how close the play was.
pub fn call_delay(margin_s: f64) -> f64 {
    let u = ((margin_s - CLOSE_MARGIN_S) / (ROUTINE_MARGIN_S - CLOSE_MARGIN_S)).clamp(0.0, 1.0);
    T_CALL_CLOSE_S + (T_CALL_ROUTINE_S - T_CALL_CLOSE_S) * u
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_plays_are_called_at_once_and_routine_ones_within_a_second() {
        // Owner request (2026-11-12): within 1 s on a routine play, almost right away on a close one.
        assert!(call_delay(0.0) <= 0.25);
        assert!(call_delay(0.05) <= 0.25);
        assert!(call_delay(f64::INFINITY) <= 1.0);
        assert!(call_delay(3.0) > call_delay(0.0));
        // Never earlier than a person can react, never NaN.
        assert!(call_delay(-1.0) >= 0.15);
        let mut last = 0.0;
        for i in 0..100 {
            let d = call_delay(i as f64 * 0.01);
            assert!(d >= last && d.is_finite());
            last = d;
        }
    }
}
