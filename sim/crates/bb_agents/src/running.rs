//! Runners' starts (agent-models §6.4): parameters only; the live simulator runs the legs
//! and the runner's decision from the same start model (`bb_game::live`).

/// A runner who has tagged up: set on his bag, watching the catch, leaving on it
/// (agent-models §6.4). [E], **values pending** the tag-up sources note
/// (docs/calibration/2026-11-14-tag-ups.md). The defaults are neutral: they reproduce the
/// game as it was before the tag-up model (he leaves on the catch tick and accelerates
/// like any runner from rest; no time allowed for the tag).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TagUpParams {
    /// From the catch to his first step, s. He is watching the ball arrive, so this is an
    /// anticipated start, not a reaction to a surprise.
    pub react_s: f64,
    /// Time constant of his acceleration from that set start, s (a runner from rest
    /// otherwise uses the generic 0.85 s, agent-models §6).
    pub tau_s: f64,
    /// His belief of how long a fielder who has the ball at the base needs to put the tag
    /// on him, s (the fielder's actual time is `FielderAttrs::tag_apply_s`).
    pub typical_tag_s: f64,
}

impl Default for TagUpParams {
    fn default() -> Self {
        Self { react_s: 0.0, tau_s: 0.85, typical_tag_s: 0.0 }
    }
}
