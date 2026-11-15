//! Runners' starts (agent-models §6.4): parameters only; the live simulator runs the legs
//! and the runner's decision from the same start model (`bb_game::live`).

/// League-average runner's top speed, m/s: Statcast's 27.0 ft/s sprint speed is 8.23 m/s
/// (8.2 is 26.9 ft/s). Each runner carries his own; this is today's default for all of them
/// and what a fielder assumes of a runner he does not know.
pub const LEAGUE_TOP_SPEED_MPS: f64 = 8.2;
/// League-average time constant of a runner's acceleration, s [E]: v = v_max(1 − e^(−t/τ)).
/// Fitted so the league-average runner covers a standing 90 ft in 4.10 s: Statcast 2017–2020
/// runners on third in sacrifice-fly situations average 4.03 s at a mean sprint speed of
/// 27.62 ft/s, and the sample's slope of −0.115 s per ft/s puts a 27.0 ft/s runner at 4.10 s
/// (docs/calibration/2026-11-14-tag-ups.md §1; 2026-11-15-runner-acceleration.md). Was 0.68 s
/// (4.02 s), fitted to the sample mean as if it were the league-average runner. Each runner
/// carries his own τ (`RunnerIn::tau_s`); this is today's default for all of them.
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
