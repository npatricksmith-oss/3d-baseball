//! Scoring decisions derived only from serialized play facts. No game state or RNG is used.

use bb_schema::ev::{self, EventBody};

/// Version of the scoring rules applied to logged evidence.
pub const SCORER_VERSION: u16 = 1;

/// Configurable rules-spec §10.1 threshold for an unsuccessful choice, seconds.
pub const DEFAULT_CHOICE_MARGIN_S: f64 = 0.15;
/// Provisional infield/outfield boundary, matching the live-play settle radius.
pub const DEFAULT_OUTFIELD_RADIUS_M: f64 = 45.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScorerOptions {
    pub choice_margin_s: f64,
    pub outfield_radius_m: f64,
}
impl Default for ScorerOptions {
    fn default() -> Self { Self { choice_margin_s: DEFAULT_CHOICE_MARGIN_S, outfield_radius_m: DEFAULT_OUTFIELD_RADIUS_M } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Hit(u8), FieldersChoice, Out, Walk, HitByPitch, Strikeout, Undetermined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayRuling {
    pub batter: u16,
    pub outcome: Outcome,
    pub sacrifice_fly: bool,
    pub rbi: u32,
}

/// Judge a completed plate appearance from a verified play record and pre-play outs.
/// The caller may use a different threshold for a future scorer version.
pub fn score_play(play: ev::PlayRecord<'_>, outs_before: u32, choice_margin_s: f64) -> Option<PlayRuling> {
    score_play_with_options(play, outs_before, ScorerOptions { choice_margin_s, ..Default::default() })
}

pub fn score_play_with_options(play: ev::PlayRecord<'_>, outs_before: u32, options: ScorerOptions) -> Option<PlayRuling> {
    let events = play.events()?;
    let pa = events.iter().find_map(|e| e.body_as_plate_appearance_ended())?;
    let batter = pa.batter();
    let runs = events.iter().filter(|e| e.body_type() == EventBody::RunScored).count() as u32;
    let outs: Vec<_> = events.iter().filter_map(|e| e.body_as_runner_out()).collect();
    let evidence = events.iter().find_map(|e| e.body_as_scorer_evidence());
    let flyout = outs.iter().any(|o| o.runner() == batter && o.kind() == ev::OutKind::FlyOut);
    let caught_foul = events.iter().filter_map(|e| e.body_as_batted_ball_ruling()).any(|r| r.caught() && r.fair_foul() == ev::FairFoul::Foul);
    let caught_in_outfield = evidence.and_then(|e| e.secured_at()).is_some_and(|p| p.x() * p.x() + p.y() * p.y() >= options.outfield_radius_m * options.outfield_radius_m);
    let sf = flyout && outs_before < 2 && runs > 0 && (caught_foul || caught_in_outfield);
    let outcome = match pa.result() {
        ev::PaResult::Walk | ev::PaResult::IntentionalWalk => Outcome::Walk,
        ev::PaResult::HitByPitch => Outcome::HitByPitch,
        ev::PaResult::Strikeout => Outcome::Strikeout,
        ev::PaResult::HomeRun => Outcome::Hit(4),
        ev::PaResult::Single => Outcome::Hit(1),
        ev::PaResult::Double => Outcome::Hit(2),
        ev::PaResult::Triple => Outcome::Hit(3),
        ev::PaResult::FieldersChoice => Outcome::FieldersChoice,
        ev::PaResult::Unknown => {
            if flyout || outs.iter().any(|o| o.runner() == batter && (o.kind() == ev::OutKind::Force || o.kind() == ev::OutKind::Strikeout)) {
                Outcome::Out
            } else if let Some(evidence) = evidence {
                let award = events.iter().filter_map(|e| e.body_as_runner_advanced()).find(|a| a.runner() == batter && a.cause() == ev::AdvanceCause::Award);
                if let Some(a) = award {
                    Outcome::Hit(a.to_base().clamp(1, 4))
                } else if !evidence.has_first_touch() {
                    // A third out can end the inning before first. A walk-off that ends
                    // before the batter arrives leaves the hit judgment unresolved.
                    if events.iter().filter_map(|e| e.body_as_game_ended()).any(|g| g.reason() == ev::GameEndReason::WalkOff) { Outcome::Undetermined } else { Outcome::Out }
                } else {
                    let mut preceding = PrecedingPlay::None;
                    for o in outs.iter().filter(|o| o.runner() != batter) {
                        let start = evidence.runners().and_then(|rs| rs.iter().find(|r| r.agent() == o.runner())).map(|r| r.base());
                        let infield = evidence.has_handling() && (1..=6).contains(&(evidence.first_handler() & 0x7f));
                        let one_or_return = start.is_some_and(|b| o.base() == b || (o.base() == b + 1 && !evidence.touches().is_some_and(|ts| ts.iter().any(|t| t.runner() == o.runner() && t.base() > b + 1))));
                        let p = if o.kind() == ev::OutKind::Force { PrecedingPlay::ForceOut }
                            else if infield && one_or_return { PrecedingPlay::InfieldOneBaseOut }
                            else { PrecedingPlay::OtherOut };
                        if matches!(p, PrecedingPlay::ForceOut | PrecedingPlay::InfieldOneBaseOut) { preceding = p; break; }
                        preceding = p;
                    }
                    if preceding == PrecedingPlay::None && (evidence.throws().is_some_and(|ts| ts.iter().any(|t| t.attempted_out() && t.base() != 1)) || evidence.carries().is_some_and(|cs| cs.iter().any(|c| c.base() != 1))) {
                        preceding = PrecedingPlay::UnsuccessfulAttempt;
                    }
                    let margin = evidence.has_margin().then(|| evidence.clean_play_margin_s());
                    match judge_hit(HitEvidence { preceding_play: preceding, reference_margin_s: margin }, options.choice_margin_s) {
                        HitDecision::FieldersChoice => Outcome::FieldersChoice,
                        HitDecision::Undetermined => Outcome::Undetermined,
                        HitDecision::Hit => {
                            let reached = events.iter().filter_map(|e| e.body_as_runner_advanced())
                                .filter(|a| a.runner() == batter).map(|a| a.to_base()).max().unwrap_or(1);
                            // A runner put out stretching gets the last base he safely passed.
                            let before_out = outs.iter().filter(|o| o.runner() == batter && o.base() > 1).map(|o| o.base() - 1).max().unwrap_or(1);
                            let touched = evidence.touches().and_then(|ts| ts.iter().filter(|t| t.runner() == batter).map(|t| t.base()).max()).unwrap_or(1);
                            let value = reached.max(before_out).max(touched).clamp(1, 4);
                            // Advancement begun only after an attempted play on a preceding
                            // runner is on that throw. A goal already chosen at release stands.
                            let on_throw_cap = evidence.throws().and_then(|ts| ts.iter().find(|t| t.attempted_out() && t.base() != 1)).map(|t| t.batter_goal().max(1));
                            Outcome::Hit(on_throw_cap.map_or(value, |cap| value.min(cap)))
                        }
                    }
                }
            } else { Outcome::Undetermined }
        }
        _ => Outcome::Out,
    };
    let rbi = if outs.len() >= 2 { 0 } else { runs };
    Some(PlayRuling { batter, outcome, sacrifice_fly: sf, rbi })
}


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
        PrecedingPlay::UnsuccessfulAttempt => match e.reference_margin_s.filter(|m| !m.is_nan()) {
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
        assert_eq!(judge_hit(evidence(Some(0.20)), 0.25), HitDecision::Hit);
        assert_eq!(judge_hit(evidence(None), 0.15), HitDecision::Undetermined);
    }

    #[test]
    fn outfielder_retires_runner_taking_extra_base() {
        assert_eq!(judge_hit(HitEvidence { preceding_play: PrecedingPlay::OtherOut, reference_margin_s: None }, 0.15), HitDecision::Hit);
    }
}
