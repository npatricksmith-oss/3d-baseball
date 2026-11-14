//! Scoring decisions over observed evidence. Constructing this evidence from a persisted
//! play is not implemented yet; in particular the current log lacks a reference margin.

/// The outcome of a play on a preceding runner, under OBR 9.05(b).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrecedingPlay {
    None,
    /// A preceding runner was forced out.
    ForceOut,
    /// An infielder handled the ball and retired a runner advancing one base or returning.
    InfieldOneBaseOut,
    /// A play was attempted, but no preceding runner was retired.
    UnsuccessfulAttempt,
    /// Another preceding-runner out, e.g. an outfielder retiring a runner taking extra bases.
    OtherOut,
}

/// Available evidence for a batter who reached first on a fair ball.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitEvidence {
    pub preceding_play: PrecedingPlay,
    /// Runner arrival minus reference throw arrival; positive means the throw was earlier.
    /// `None` means the required counterfactual has not been recorded.
    pub reference_margin_s: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitDecision {
    Hit,
    FieldersChoice,
    /// The missing margin must not be silently interpreted as either a hit or a choice.
    Undetermined,
}

/// Apply the no-error part of OBR 9.05. The choice margin is supplied by the caller so the
/// rules-spec §10.1 parameter is configurable. Error rulings require a separate ordinary
/// effort/misplay check before this helper; the sim does not yet provide that evidence.
pub fn judge_hit(e: HitEvidence, choice_margin_s: f64) -> HitDecision {
    match e.preceding_play {
        PrecedingPlay::ForceOut | PrecedingPlay::InfieldOneBaseOut => HitDecision::FieldersChoice,
        PrecedingPlay::UnsuccessfulAttempt => match e.reference_margin_s.filter(|m| m.is_finite()) {
            Some(margin) if margin >= choice_margin_s => HitDecision::FieldersChoice,
            Some(_) => HitDecision::Hit,
            None => HitDecision::Undetermined,
        },
        PrecedingPlay::None | PrecedingPlay::OtherOut => HitDecision::Hit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // OBR 9.05(b)(1), (3): a force, and an infielder's putout of a runner trying to
    // advance one base, both deny the hit even when the batter would have beaten a throw.
    #[test]
    fn successful_force_or_infield_one_base_play_denies_hit() {
        for play in [PrecedingPlay::ForceOut, PrecedingPlay::InfieldOneBaseOut] {
            assert_eq!(judge_hit(HitEvidence { preceding_play: play, reference_margin_s: Some(-1.0) }, 0.15), HitDecision::FieldersChoice);
        }
    }

    // OBR 9.05(a)(6), (b)(4), with the spec's 0.15 s margin. A failed play on
    // another runner is a hit if an ordinary throw to first would not get the batter.
    #[test]
    fn failed_choice_uses_reference_throw_and_preserves_unknown() {
        let evidence = |margin| HitEvidence { preceding_play: PrecedingPlay::UnsuccessfulAttempt, reference_margin_s: margin };
        assert_eq!(judge_hit(evidence(Some(-0.20)), 0.15), HitDecision::Hit);
        assert_eq!(judge_hit(evidence(Some(0.15)), 0.15), HitDecision::FieldersChoice);
        assert_eq!(judge_hit(evidence(None), 0.15), HitDecision::Undetermined);
    }

    #[test]
    fn outfielder_retires_runner_taking_extra_base() {
        assert_eq!(judge_hit(HitEvidence { preceding_play: PrecedingPlay::OtherOut, reference_margin_s: None }, 0.15), HitDecision::Hit);
    }
}
