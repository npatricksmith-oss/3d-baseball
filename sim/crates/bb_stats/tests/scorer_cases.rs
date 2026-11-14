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
    let mut f = FlatBufferBuilder::new();
    let runners = [ev::RunnerOnBase::create(&mut f, &ev::RunnerOnBaseArgs { agent: BATTER, base: 0 }), ev::RunnerOnBase::create(&mut f, &ev::RunnerOnBaseArgs { agent: RUNNER, base: 1 })];
    let runners = f.create_vector(&runners);
    let throw = ev::ScorerThrow::create(&mut f, &ev::ScorerThrowArgs { base: 2, attempted_out: throw_choice, batter_goal: goal, ..Default::default() });
    let throws = f.create_vector(&[throw]);
    let evidence = ev::ScorerEvidence::create(&mut f, &ev::ScorerEvidenceArgs { batter: BATTER, runners: Some(runners), has_first_touch: batter_out.is_none(), first_touch_s: 4.5, has_handling: true, first_handler: handler, has_margin: margin.is_some(), clean_play_margin_s: margin.unwrap_or_default(), throws: Some(throws), ..Default::default() });
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
