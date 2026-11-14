//! OBR 9.05 examples expressed as independent serialized play fixtures.
use bb_schema::ev::{self, EventBody};
use bb_stats::scorer::{score_play, Outcome};
use flatbuffers::{FlatBufferBuilder, UnionWIPOffset, WIPOffset};

const BATTER: u16 = 7;
const RUNNER: u16 = 4;
fn fixture(handler: u16, prior_out: Option<(u8, ev::OutKind)>, throw_choice: bool, margin: Option<f64>, batter_out: Option<ev::OutKind>, runs: bool) -> Vec<u8> {
    fixture_extended(handler, prior_out, throw_choice, margin, batter_out, runs, 1, 1)
}
#[allow(clippy::too_many_arguments)]
fn fixture_extended(handler: u16, prior_out: Option<(u8, ev::OutKind)>, throw_choice: bool, margin: Option<f64>, batter_out: Option<ev::OutKind>, runs: bool, reached: u8, goal: u8) -> Vec<u8> {
    fixture_at(handler, prior_out, throw_choice, margin, batter_out, runs, reached, goal, if handler >= 7 { 75.0 } else { 25.0 })
}
#[allow(clippy::too_many_arguments)]
fn fixture_at(handler: u16, prior_out: Option<(u8, ev::OutKind)>, throw_choice: bool, margin: Option<f64>, batter_out: Option<ev::OutKind>, runs: bool, reached: u8, goal: u8, catch_y: f64) -> Vec<u8> {
    let mut f = FlatBufferBuilder::new();
    let runners = [ev::RunnerOnBase::create(&mut f, &ev::RunnerOnBaseArgs { agent: BATTER, base: 0 }), ev::RunnerOnBase::create(&mut f, &ev::RunnerOnBaseArgs { agent: RUNNER, base: 1 })];
    let runners = f.create_vector(&runners);
    let throw = ev::ScorerThrow::create(&mut f, &ev::ScorerThrowArgs { base: 2, attempted_out: throw_choice, batter_goal: goal, ..Default::default() });
    let throws = f.create_vector(&[throw]);
    let catch_at = bb_schema::vec3d([0.0, catch_y, 0.0]);
    let evidence = ev::ScorerEvidence::create(&mut f, &ev::ScorerEvidenceArgs { has_secured: true, reference_fielder: handler, secured_at: Some(&catch_at), batter: BATTER, runners: Some(runners), has_first_touch: batter_out.is_none(), first_touch_s: 4.5, has_handling: true, first_handler: handler, has_margin: margin.is_some(), clean_play_margin_s: margin.unwrap_or_default(), throws: Some(throws), ..Default::default() });
    let mut bodies: Vec<(EventBody, WIPOffset<UnionWIPOffset>)> = vec![(EventBody::ScorerEvidence, evidence.as_union_value())];
    if let Some((base, kind)) = prior_out {
        let o = ev::RunnerOut::create(&mut f, &ev::RunnerOutArgs { runner: RUNNER, base, kind, ..Default::default() }); bodies.push((EventBody::RunnerOut, o.as_union_value()));
    }
    if let Some(kind) = batter_out {
        let o = ev::RunnerOut::create(&mut f, &ev::RunnerOutArgs { runner: BATTER, base: 0, kind, ..Default::default() }); bodies.push((EventBody::RunnerOut, o.as_union_value()));
    } else {
        let a = ev::RunnerAdvanced::create(&mut f, &ev::RunnerAdvancedArgs { runner: BATTER, to_base: reached, ..Default::default() }); bodies.push((EventBody::RunnerAdvanced, a.as_union_value()));
    }
    if runs { let r = ev::RunScored::create(&mut f, &ev::RunScoredArgs { runner: RUNNER, ..Default::default() }); bodies.push((EventBody::RunScored, r.as_union_value())); }
    let pa = ev::PlateAppearanceEnded::create(&mut f, &ev::PlateAppearanceEndedArgs { batter: BATTER, result: ev::PaResult::Unknown, ..Default::default() }); bodies.push((EventBody::PlateAppearanceEnded, pa.as_union_value()));
    let es: Vec<_> = bodies.into_iter().enumerate().map(|(seq,(ty,body))| ev::Event::create(&mut f, &ev::EventArgs { seq: seq as u32, body_type: ty, body: Some(body), ..Default::default() })).collect();
    let es = f.create_vector(&es);
    let p = ev::PlayRecord::create(&mut f, &ev::PlayRecordArgs { play_index: 1, events: Some(es) });
    f.finish(p, None); f.finished_data().to_vec()
}
fn judge(bytes: &[u8], outs: u32) -> bb_stats::scorer::PlayRuling { score_play(flatbuffers::root::<ev::PlayRecord>(bytes).unwrap(), outs, 0.15).unwrap() }

#[test]
fn force_and_one_base_infield_putout_are_not_hits_even_with_a_late_throw() {
    assert_eq!(judge(&fixture(6, Some((2, ev::OutKind::Force)), false, Some(-1.0), None, false), 0).outcome, Outcome::FieldersChoice);
    assert_eq!(judge(&fixture(6, Some((2, ev::OutKind::Tag)), false, Some(-1.0), None, false), 0).outcome, Outcome::FieldersChoice);
}
#[test]
fn outfielder_puts_runner_out_and_hit_stands() {
    assert_eq!(judge(&fixture(8, Some((3, ev::OutKind::Tag)), true, Some(1.0), None, false), 0).outcome, Outcome::Hit(1));
}
#[test]
fn unsuccessful_choice_depends_on_reference_and_routine_throw_is_not_choice() {
    assert_eq!(judge(&fixture(6, None, true, Some(-0.2), None, false), 0).outcome, Outcome::Hit(1));
    assert_eq!(judge(&fixture(6, None, true, Some(0.2), None, false), 0).outcome, Outcome::FieldersChoice);
    assert_eq!(judge(&fixture(6, None, true, None, None, false), 0).outcome, Outcome::Undetermined);
    assert_eq!(judge(&fixture(6, None, false, Some(1.0), None, false), 0).outcome, Outcome::Hit(1));
}
#[test]
fn fly_to_outfield_scores_runner_as_sacrifice_and_rbi() {
    let p = fixture(8, None, false, None, Some(ev::OutKind::FlyOut), true);
    let r = judge(&p, 1); assert_eq!((r.outcome, r.sacrifice_fly, r.rbi), (Outcome::Out, true, 1));
    assert!(!judge(&p, 2).sacrifice_fly);
}

#[test]
fn extra_base_chosen_only_after_throw_is_scored_as_advance_on_throw() {
    let after = fixture_extended(8, Some((3, ev::OutKind::Tag)), true, Some(-1.0), None, false, 2, 1);
    assert_eq!(judge(&after, 0).outcome, Outcome::Hit(1));
    let before = fixture_extended(8, Some((3, ev::OutKind::Tag)), true, Some(-1.0), None, false, 2, 2);
    assert_eq!(judge(&before, 0).outcome, Outcome::Hit(2));
}

#[test]
fn run_on_double_play_has_no_rbi() {
    let p = fixture(6, Some((2, ev::OutKind::Force)), true, Some(0.4), Some(ev::OutKind::Force), true);
    assert_eq!(judge(&p, 0).rbi, 0);
}

#[test]
fn forced_walk_and_hit_batter_credit_the_run() {
    for (result, expected) in [(ev::PaResult::Walk, Outcome::Walk), (ev::PaResult::HitByPitch, Outcome::HitByPitch)] {
        let mut f = FlatBufferBuilder::new();
        let run = ev::RunScored::create(&mut f, &ev::RunScoredArgs { runner: RUNNER, ..Default::default() });
        let pa = ev::PlateAppearanceEnded::create(&mut f, &ev::PlateAppearanceEndedArgs { batter: BATTER, result, ..Default::default() });
        let e1 = ev::Event::create(&mut f, &ev::EventArgs { body_type: EventBody::RunScored, body: Some(run.as_union_value()), ..Default::default() });
        let e2 = ev::Event::create(&mut f, &ev::EventArgs { body_type: EventBody::PlateAppearanceEnded, body: Some(pa.as_union_value()), ..Default::default() });
        let events = f.create_vector(&[e1, e2]);
        let play = ev::PlayRecord::create(&mut f, &ev::PlayRecordArgs { events: Some(events), ..Default::default() });
        f.finish(play, None);
        let ruling = judge(f.finished_data(), 1);
        assert_eq!((ruling.outcome, ruling.rbi), (expected, 1));
    }
}

#[test]
fn walk_off_before_batter_touches_first_remains_undetermined() {
    let mut f = FlatBufferBuilder::new();
    let sc = ev::ScorerEvidence::create(&mut f, &ev::ScorerEvidenceArgs { batter: BATTER, has_first_touch: false, ..Default::default() });
    let pa = ev::PlateAppearanceEnded::create(&mut f, &ev::PlateAppearanceEndedArgs { batter: BATTER, result: ev::PaResult::Unknown, ..Default::default() });
    let end = ev::GameEnded::create(&mut f, &ev::GameEndedArgs { reason: ev::GameEndReason::WalkOff, ..Default::default() });
    let events = [
        ev::Event::create(&mut f, &ev::EventArgs { body_type: EventBody::ScorerEvidence, body: Some(sc.as_union_value()), ..Default::default() }),
        ev::Event::create(&mut f, &ev::EventArgs { body_type: EventBody::PlateAppearanceEnded, body: Some(pa.as_union_value()), ..Default::default() }),
        ev::Event::create(&mut f, &ev::EventArgs { body_type: EventBody::GameEnded, body: Some(end.as_union_value()), ..Default::default() }),
    ];
    let events = f.create_vector(&events);
    let play = ev::PlayRecord::create(&mut f, &ev::PlayRecordArgs { events: Some(events), ..Default::default() });
    f.finish(play, None);
    assert_eq!(judge(f.finished_data(), 0).outcome, Outcome::Undetermined);
}

// OBR 9.08(d): "handled by an outfielder or an infielder running in the outfield". An infielder who
// catches the fly well out in the grass has a sacrifice fly; an outfielder who takes it in the
// infield has one too (it is the fielder that counts, and where an infielder is); an infielder
// catching it in the infield has none. (Review P2-3 corrected the first version of this test.)
#[test]
fn sacrifice_fly_by_who_caught_it_and_where() {
    let infielder_in_outfield = fixture_at(6, None, false, None, Some(ev::OutKind::FlyOut), true, 1, 1, 75.0);
    let outfielder_in_infield = fixture_at(8, None, false, None, Some(ev::OutKind::FlyOut), true, 1, 1, 25.0);
    let infielder_in_infield = fixture_at(6, None, false, None, Some(ev::OutKind::FlyOut), true, 1, 1, 25.0);
    assert!(judge(&infielder_in_outfield, 1).sacrifice_fly);
    assert!(judge(&outfielder_in_infield, 1).sacrifice_fly);
    assert!(!judge(&infielder_in_infield, 1).sacrifice_fly);
}

/// A play in which a thrown ball leaves the field (OBR 5.06(b)(3)(D)): the batter is
/// awarded bases from the time of the throw. `touch_s` is when he touched first, if he did;
/// the throw to first leaves at 4.0 s and `margin_s` is the reference throw's margin. A runner
/// on second scores by the award.
fn thrown_ball_out_of_play(touch_s: Option<f64>, margin_s: f64) -> Vec<u8> {
    let mut f = FlatBufferBuilder::new();
    let runners = [ev::RunnerOnBase::create(&mut f, &ev::RunnerOnBaseArgs { agent: BATTER, base: 0 }), ev::RunnerOnBase::create(&mut f, &ev::RunnerOnBaseArgs { agent: RUNNER, base: 2 })];
    let runners = f.create_vector(&runners);
    let throw = ev::ScorerThrow::create(&mut f, &ev::ScorerThrowArgs { base: 1, t_release: 4.0, attempted_out: true, runner: BATTER, ..Default::default() });
    let throws = f.create_vector(&[throw]);
    let touches = [ev::ScorerTouch::create(&mut f, &ev::ScorerTouchArgs { runner: BATTER, base: 1, t: touch_s.unwrap_or(0.0) })];
    let touches = f.create_vector(&touches);
    let ev_ = ev::ScorerEvidence::create(&mut f, &ev::ScorerEvidenceArgs { batter: BATTER, runners: Some(runners), has_first_touch: touch_s.is_some(), first_touch_s: touch_s.unwrap_or(0.0), has_margin: true, clean_play_margin_s: margin_s, throws: Some(throws), touches: Some(touches), ..Default::default() });
    let dead = ev::BallDead::create(&mut f, &ev::BallDeadArgs { reason: ev::DeadBallReason::OutOfPlay });
    let adv = ev::RunnerAdvanced::create(&mut f, &ev::RunnerAdvancedArgs { runner: BATTER, to_base: 3, cause: ev::AdvanceCause::Award, ..Default::default() });
    let run = ev::RunScored::create(&mut f, &ev::RunScoredArgs { runner: RUNNER, ..Default::default() });
    let pa = ev::PlateAppearanceEnded::create(&mut f, &ev::PlateAppearanceEndedArgs { batter: BATTER, result: ev::PaResult::Unknown, ..Default::default() });
    let bodies = [(EventBody::ScorerEvidence, ev_.as_union_value()), (EventBody::BallDead, dead.as_union_value()), (EventBody::RunnerAdvanced, adv.as_union_value()), (EventBody::RunScored, run.as_union_value()), (EventBody::PlateAppearanceEnded, pa.as_union_value())];
    let es: Vec<_> = bodies.into_iter().enumerate().map(|(seq, (ty, body))| ev::Event::create(&mut f, &ev::EventArgs { seq: seq as u32, body_type: ty, body: Some(body), ..Default::default() })).collect();
    let es = f.create_vector(&es);
    let p = ev::PlayRecord::create(&mut f, &ev::PlayRecordArgs { play_index: 1, events: Some(es) });
    f.finish(p, None);
    f.finished_data().to_vec()
}

// OBR 9.05(a)(1), 9.12(a)(1), 5.06(b)(3)(D): bases given for a throw out of play are not a hit
// when the batter would have been put out by an ordinary throw to first (the reference throw
// was 0.30 s early): he reaches on the thrower's error. When he would have beaten the ordinary
// throw (by a full second here) the hit is his whenever he touches first, before or after the
// throw left, and the bases awarded are not part of its value. (The first version of this test
// had the batter beat the reference throw by 1.0 s and still ruled no hit; Gemini review P1-3.)
#[test]
fn bases_awarded_for_a_throw_out_of_play_are_a_hit_only_if_he_would_have_beaten_the_throw() {
    assert_eq!(judge(&thrown_ball_out_of_play(None, 0.30), 0).outcome, Outcome::ReachedOnThrowAward);
    assert_eq!(judge(&thrown_ball_out_of_play(Some(5.0), 0.30), 0).outcome, Outcome::ReachedOnThrowAward);
    for touch in [None, Some(5.0), Some(3.5)] {
        assert_eq!(judge(&thrown_ball_out_of_play(touch, -1.0), 0).outcome, Outcome::Hit(1), "touch {touch:?}");
    }
}

// The runner from second scored by the award, because of the throw and not the batter: no RBI
// (second base plus a single is third, short of home).
#[test]
fn a_run_scoring_by_the_throw_award_is_not_an_rbi() {
    assert_eq!(judge(&thrown_ball_out_of_play(Some(3.5), -1.0), 0).rbi, 0);
}
