//! Scoring decisions derived only from serialized play facts. No game state or RNG is used.

use bb_schema::ev;

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
    /// The batter reached only because a thrown ball left the field and bases were awarded
    /// (OBR 5.06(b)(3)(D)). Not a hit. This is the seam for the error ruling (rules-spec
    /// §10.2, A17): once the sim has fielding errors the throw is ruled an error and this
    /// becomes reached on error. Until then it is an at-bat without a hit, and no error is
    /// charged.
    ReachedOnThrowAward,
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
    let run_ids: Vec<u16> = events.iter().filter_map(|e| e.body_as_run_scored()).map(|r| r.runner()).collect();
    let outs: Vec<_> = events.iter().filter_map(|e| e.body_as_runner_out()).collect();
    let evidence = events.iter().find_map(|e| e.body_as_scorer_evidence());
    // Evidence written before A12 lacks the runner list (the batter is always in it): its
    // flags read as "false", which must not be read as facts.
    let evidence = evidence.filter(|e| e.runners().is_some_and(|r| !r.is_empty()));
    let start_of = |id: u16| evidence.and_then(|e| e.runners()).and_then(|rs| rs.iter().find(|r| r.agent() == id)).map(|r| r.base());
    let flyout = outs.iter().any(|o| o.runner() == batter && o.kind() == ev::OutKind::FlyOut);
    let walk_off = events.iter().filter_map(|e| e.body_as_game_ended()).any(|g| g.reason() == ev::GameEndReason::WalkOff);
    let caught_foul = events.iter().filter_map(|e| e.body_as_batted_ball_ruling()).any(|r| r.caught() && r.fair_foul() == ev::FairFoul::Foul);
    let thrown_award = events.iter().filter_map(|e| e.body_as_ball_dead()).any(|d| d.reason() == ev::DeadBallReason::OutOfPlay);
    // A throw out of play: when the last throw left (rules-spec §10.2).
    let award_cutoff_s = if thrown_award { evidence.and_then(|e| e.throws()).and_then(|ts| ts.iter().map(|t| t.t_release()).reduce(f64::max)) } else { None };
    let before_throw = |t: f64| award_cutoff_s.is_none_or(|c| t <= c);
    let home_touch = |id: u16| evidence.and_then(|e| e.touches()).is_some_and(|ts| ts.iter().any(|t| t.runner() == id && t.base() == 4));
    // The batter's touch of first, or the throws and carries on a preceding runner while he was
    // still short of it.
    let first_touch = evidence.filter(|e| e.has_first_touch()).map(|e| e.first_touch_s());
    let preceding_target = |runner: u16, fallback_base: u8, base: u8| -> bool {
        if runner == 0 { base != fallback_base } else { runner != batter && start_of(runner).is_some_and(|b| b > 0) }
    };
    let while_batter_short = |t: f64| first_touch.is_none_or(|ft| t < ft);

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
            if outs_before < 2 && flyout && !run_ids.is_empty() && evidence.is_none() {
                // Without the catch's place the scorer cannot tell a sacrifice fly (9.08(d)).
                Outcome::Undetermined
            } else if flyout || outs.iter().any(|o| o.runner() == batter && (o.kind() == ev::OutKind::Force || o.kind() == ev::OutKind::Strikeout)) {
                Outcome::Out
            } else if let Some(evidence) = evidence {
                let award = events.iter().filter_map(|e| e.body_as_runner_advanced()).find(|a| a.runner() == batter && a.cause() == ev::AdvanceCause::Award);
                let margin = evidence.has_margin().then(|| evidence.clean_play_margin_s());
                let last_throw = evidence.throws().and_then(|ts| ts.iter().max_by(|a, b| a.t_release().total_cmp(&b.t_release())));
                if let Some(a) = award.filter(|_| !thrown_award) {
                    // A ground-rule double.
                    Outcome::Hit(a.to_base().clamp(1, 4))
                } else if !evidence.has_first_touch() && !walk_off && !thrown_award {
                    // A third out can end the inning before first (9.02(g)).
                    Outcome::Out
                } else if thrown_award && last_throw.is_some_and(|t| t.base() == 1) && margin.is_none() {
                    Outcome::Undetermined
                } else if thrown_award && last_throw.is_some_and(|t| t.base() == 1) && margin.is_some_and(|m| m >= options.choice_margin_s) {
                    // 9.12(a)(1): the ordinary throw would have retired him; the bases are the
                    // thrower's (error ruling pending, A17).
                    Outcome::ReachedOnThrowAward
                } else {
                    let mut preceding = PrecedingPlay::None;
                    for o in outs.iter().filter(|o| o.runner() != batter) {
                        let start = start_of(o.runner());
                        let infield = evidence.has_handling() && (1..=6).contains(&(evidence.first_handler() & 0x7f));
                        let one_or_return = start.is_some_and(|b| o.base() == b || (o.base() == b + 1 && !evidence.touches().is_some_and(|ts| ts.iter().any(|t| t.runner() == o.runner() && t.base() > b + 1))));
                        let p = if o.kind() == ev::OutKind::Force {
                            PrecedingPlay::ForceOut
                        } else if o.kind() == ev::OutKind::Appeal && start.is_some_and(|b| o.base() == b + 1) {
                            // 9.05(b)(2): out for missing the first base to which he advanced.
                            PrecedingPlay::ForceOut
                        } else if infield && one_or_return {
                            PrecedingPlay::InfieldOneBaseOut
                        } else {
                            PrecedingPlay::OtherOut
                        };
                        if matches!(p, PrecedingPlay::ForceOut | PrecedingPlay::InfieldOneBaseOut) { preceding = p; break; }
                        preceding = p;
                    }
                    // A play chosen on a runner ahead of the batter, made while he was still
                    // short of first (9.05(a)(6)); one meant for the batter-runner is no choice.
                    let attempt_throw = |t: ev::ScorerThrow| t.attempted_out() && preceding_target(t.runner(), 1, t.base()) && while_batter_short(t.t_release());
                    let attempt_carry = |c: ev::ScorerCarry| preceding_target(c.runner(), 1, c.base()) && while_batter_short(c.t());
                    if preceding == PrecedingPlay::None && (evidence.throws().is_some_and(|ts| ts.iter().any(attempt_throw)) || evidence.carries().is_some_and(|cs| cs.iter().any(attempt_carry))) {
                        preceding = PrecedingPlay::UnsuccessfulAttempt;
                    }
                    match judge_hit(HitEvidence { preceding_play: preceding, reference_margin_s: margin }, options.choice_margin_s) {
                        HitDecision::FieldersChoice => Outcome::FieldersChoice,
                        HitDecision::Undetermined => Outcome::Undetermined,
                        HitDecision::Hit => {
                            let reached = events.iter().filter_map(|e| e.body_as_runner_advanced())
                                .filter(|a| a.runner() == batter && a.cause() != ev::AdvanceCause::Award).map(|a| a.to_base()).max().unwrap_or(1);
                            // A runner put out stretching gets the last base he safely passed.
                            let before_out = outs.iter().filter(|o| o.runner() == batter && o.base() > 1).map(|o| o.base() - 1).max().unwrap_or(1);
                            let touched = evidence.touches().and_then(|ts| ts.iter().filter(|t| t.runner() == batter && before_throw(t.t())).map(|t| t.base()).max()).unwrap_or(1);
                            let value = reached.max(before_out).max(touched).clamp(1, 4);
                            // Advancement begun only after an attempted play on a preceding
                            // runner is on that throw. A goal already chosen at release stands.
                            let on_throw_cap = evidence.throws().and_then(|ts| ts.iter().find(|t| t.attempted_out() && preceding_target(t.runner(), 1, t.base()) && while_batter_short(t.t_release()))).map(|t| t.batter_goal().max(1));
                            let mut value = on_throw_cap.map_or(value, |cap| value.min(cap));
                            // 9.06(f): a walk-off hit is worth the bases the winning runner advanced.
                            if walk_off {
                                if let Some(b) = run_ids.last().and_then(|&id| start_of(id)).filter(|&b| b >= 1) {
                                    value = value.min(4 - b);
                                }
                            }
                            Outcome::Hit(value)
                        }
                    }
                }
            } else {
                Outcome::Undetermined
            }
        }
        _ => Outcome::Out,
    };
    // Runs that count. A run scored only because a thrown ball went out of play was given by
    // the award, not driven in, unless the batter's hit would have scored him anyway (9.04(a)(1)).
    let hit_bases = if let Outcome::Hit(k) = outcome { Some(k) } else { None };
    let credited: Vec<u16> = run_ids.iter().copied()
        .filter(|&id| id != batter || outcome == Outcome::Hit(4))
        .filter(|&id| !thrown_award || home_touch(id) || hit_bases.is_some_and(|k| start_of(id).is_some_and(|b| b >= 1 && b + k >= 4)))
        .collect();
    // 9.08(d): a runner scored after a caught fly, fewer than two out, the ball handled by an
    // outfielder or an infielder running in the outfield, in fair or foul territory.
    let catcher = evidence.and_then(|e| if e.has_secured() { Some(e.reference_fielder()) } else if e.has_handling() { Some(e.first_handler()) } else { None });
    let by_outfielder = catcher.is_some_and(|f| (7..=9).contains(&(f & 0x7f)));
    let caught_in_outfield = evidence.and_then(|e| e.secured_at()).is_some_and(|p| p.x() * p.x() + p.y() * p.y() >= options.outfield_radius_m * options.outfield_radius_m);
    let sf = flyout && outs_before < 2 && !credited.is_empty() && (caught_foul || by_outfielder || caught_in_outfield);
    // 9.04(b)(1): no RBI when the batter grounds into a force or reverse-force double play.
    let forced_start = |id: u16| -> bool {
        match start_of(id) {
            Some(b) if b >= 1 => (1..b).all(|k| evidence.and_then(|e| e.runners()).is_some_and(|rs| rs.iter().any(|r| r.base() == k))),
            _ => false,
        }
    };
    let ground_double_play = !flyout && !matches!(outcome, Outcome::Hit(_) | Outcome::Walk | Outcome::HitByPitch | Outcome::Strikeout)
        && outs.len() >= 2
        && outs.iter().any(|o| o.kind() == ev::OutKind::Force)
        && outs.iter().all(|o| o.kind() == ev::OutKind::Force || (o.runner() != batter && forced_start(o.runner())));
    let rbi = if matches!(outcome, Outcome::Strikeout) || ground_double_play { 0 } else { credited.len() as u32 };
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
