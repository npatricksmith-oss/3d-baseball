//! Runners' starts (agent-models §6.4): parameters only; the live simulator runs the legs
//! and the runner's decision from the same start model (`bb_game::live`).

/// A runner who has tagged up: set on his bag, watching the catch, leaving on it
/// (agent-models §6.4; values from docs/calibration/2026-11-14-tag-ups.md).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TagUpParams {
    /// From the catch to his first step, s [E]: 0. He is watching the ball arrive and the
    /// rolling-start technique is built to cut his reaction to about nothing (note §1); the
    /// Statcast tag-up clock starts at the catch.
    pub react_s: f64,
    /// Time constant of his acceleration from that set start, s [E]: the generic runner's
    /// 0.68 s. A rolling start's gain has no published measurement, so it is set to nothing
    /// (note §1, §7); a measured gain would lower this.
    pub tau_s: f64,
    /// His belief of how long a fielder who has the ball at the base needs to put the tag
    /// on him, s [E]: the quoted 0.25 s (the fielder's actual time is
    /// `FielderAttrs::tag_apply_s`).
    pub typical_tag_s: f64,
}

impl Default for TagUpParams {
    fn default() -> Self {
        Self { react_s: 0.0, tau_s: 0.68, typical_tag_s: 0.25 }
    }
}
