//! A complete scripted game ending in a walk-off with two pending home touches (Codex LOB recheck
//! P1-3): bottom ninth, tied, runners on second and third touch home before the ball is ruled
//! fair. Only the first run counts (OBR 5.08(b)); the trailing runner and the batter neither
//! scored nor were out, so LOB is 2 (9.02(g)) and I1 holds (9.03(c)).
use bb_rules::*;
use bb_stats::box_score;
use bb_schema::{ev, log};
use flatbuffers::FlatBufferBuilder;

fn append(w: &mut log::LogWriter, index: u32, rules: &[Event], evidence: Option<(u16,bool,Vec<(u16,u8)>)>, corrupt: bool) {
    w.write_play(index, |f| {
        use ev::EventBody as T;
        let mut bodies = Vec::new();
        if let Some((batter,first,rs))=evidence {
            let rs:Vec<_>=rs.iter().map(|&(agent,base)|ev::RunnerOnBase::create(f,&ev::RunnerOnBaseArgs{agent,base})).collect();
            let rs=f.create_vector(&rs);
            let sc=ev::ScorerEvidence::create(f,&ev::ScorerEvidenceArgs{batter,runners:Some(rs),has_first_touch:first,first_touch_s:3.,..Default::default()});
            bodies.push((T::ScorerEvidence,sc.as_union_value()));
        }
        for r in rules {
            let x=match *r {
                Event::HalfInningStarted{inning,half}=>(T::HalfInningStarted,ev::HalfInningStarted::create(f,&ev::HalfInningStartedArgs{inning,half:if half==Half::Top{ev::HalfInning::Top}else{ev::HalfInning::Bottom}}).as_union_value()),
                Event::RunnerAdvanced{runner,from,to,cause}=>(T::RunnerAdvanced,ev::RunnerAdvanced::create(f,&ev::RunnerAdvancedArgs{runner,from_base:from,to_base:to,cause:match cause{AdvanceCause::Walk=>ev::AdvanceCause::Walk,AdvanceCause::HitByPitch=>ev::AdvanceCause::HitByPitch,AdvanceCause::Award=>ev::AdvanceCause::Award,_=>ev::AdvanceCause::Unknown},..Default::default()}).as_union_value()),
                Event::RunnerOut{runner,base,kind}=>(T::RunnerOut,ev::RunnerOut::create(f,&ev::RunnerOutArgs{runner,base,kind:match kind{OutKind::Strikeout=>ev::OutKind::Strikeout,OutKind::FlyOut=>ev::OutKind::FlyOut,OutKind::Force=>ev::OutKind::Force,OutKind::Tag=>ev::OutKind::Tag,OutKind::Appeal=>ev::OutKind::Appeal},..Default::default()}).as_union_value()),
                Event::RunScored{runner}=>(T::RunScored,ev::RunScored::create(f,&ev::RunScoredArgs{runner,..Default::default()}).as_union_value()),
                Event::PlateAppearanceEnded{batter,result}=>(T::PlateAppearanceEnded,ev::PlateAppearanceEnded::create(f,&ev::PlateAppearanceEndedArgs{batter,result:match result{PaResult::Walk=>ev::PaResult::Walk,PaResult::HitByPitch=>ev::PaResult::HitByPitch,PaResult::Strikeout=>ev::PaResult::Strikeout,PaResult::HomeRun=>ev::PaResult::HomeRun,PaResult::BattedBall=>ev::PaResult::Unknown},..Default::default()}).as_union_value()),
                Event::HalfInningEnded{runs,left_on_base}=>(T::HalfInningEnded,ev::HalfInningEnded::create(f,&ev::HalfInningEndedArgs{runs,left_on_base:if corrupt{99}else{left_on_base}}).as_union_value()),
                Event::GameEnded{away,home,walk_off,..}=>(T::GameEnded,ev::GameEnded::create(f,&ev::GameEndedArgs{score_away:away as u8,score_home:home as u8,reason:if walk_off{ev::GameEndReason::WalkOff}else{ev::GameEndReason::Regulation},..Default::default()}).as_union_value()),
                _=>continue,
            }; bodies.push(x);
        }
        let es:Vec<_>=bodies.into_iter().enumerate().map(|(i,(body_type,body))|ev::Event::create(f,&ev::EventArgs{seq:i as u32,body_type,body:Some(body),..Default::default()})).collect();
        let es=f.create_vector(&es);
        es
    });
}
#[test]
fn a_walk_off_with_two_pending_home_touches_leaves_the_second_runner_on_base() {
    let mut f=FlatBufferBuilder::new();
    let p=bb_schema::ProducerInfo::create(&mut f,&bb_schema::ProducerInfoArgs{schema_major:bb_schema::SCHEMA_MAJOR,schema_minor:bb_schema::SCHEMA_MINOR,..Default::default()});
    let h=ev::LogHeader::create(&mut f,&ev::LogHeaderArgs{producer:Some(p),..Default::default()});
    f.finish_size_prefixed(h,Some(log::HEADER_ID));
    let mut w=log::LogWriter::new(f.finished_data());
    let mut st=RulesState::default_fixture();let mut index=0;
    let start=step(&mut st,&Input::StartGame);append(&mut w,index,&start,None,false);index+=1;
    let mut homers=[0;2];
    while !(st.inning==9 && st.half==Half::Bottom) {
        let mut es=Vec::new();
        let side=st.half.batting();
        if st.inning==1 && homers[side]<2 {
            for input in [Input::PitchDelivered{t:0.},Input::BatContact{t:0.4},Input::BallTouched{t:3.,what:Touch::FoulPole}] {es.extend(step(&mut st,&input));}
            homers[side]+=1;
        } else {
            for _ in 0..3 {for input in [Input::PitchDelivered{t:0.},Input::PitchCrossed{t:0.4,in_zone:true,bounced:false},Input::SwingMeasured{t:0.45,swung:false}] {es.extend(step(&mut st,&input));}}
        }
        append(&mut w,index,&es,None,false);index+=1;
    }
    assert_eq!(st.score(),[2,2]);
    let lead=st.batter;let mut es=Vec::new();
    for _ in 0..4 {for input in [Input::PitchDelivered{t:0.},Input::PitchCrossed{t:0.4,in_zone:false,bounced:false},Input::SwingMeasured{t:0.45,swung:false}] {es.extend(step(&mut st,&input));}}
    append(&mut w,index,&es,None,false);index+=1;
    let follow=st.batter;let mut es=Vec::new();
    for input in [Input::PitchDelivered{t:0.},Input::BatContact{t:0.4},Input::BaseLeft{t:0.5,agent:lead,base:1},Input::BallTouched{t:1.,what:Touch::Ground{x:20.,y:70.}},Input::BaseTouched{t:2.,agent:lead,base:2,holding_ball:false},Input::BaseLeft{t:2.1,agent:lead,base:2},Input::BaseTouched{t:3.,agent:follow,base:1,holding_ball:false},Input::BaseLeft{t:3.1,agent:follow,base:1},Input::BaseTouched{t:4.,agent:lead,base:3,holding_ball:false},Input::BaseTouched{t:4.5,agent:follow,base:2,holding_ball:false},Input::PlayEndCheck{t:5.}] {es.extend(step(&mut st,&input));}
    append(&mut w,index,&es,Some((follow,true,vec![(lead,1),(follow,0)])),false);index+=1;
    assert_eq!(st.bases,[None,Some(follow),Some(lead)]);
    let batter=st.batter;let mut es=Vec::new();
    for input in [Input::PitchDelivered{t:0.},Input::BatContact{t:0.4},Input::BaseLeft{t:0.5,agent:lead,base:3},Input::BaseLeft{t:0.5,agent:follow,base:2},Input::BaseTouched{t:3.,agent:batter,base:1,holding_ball:false},Input::BaseTouched{t:4.,agent:lead,base:4,holding_ball:false},Input::BaseTouched{t:4.5,agent:follow,base:3,holding_ball:false},Input::BaseLeft{t:4.6,agent:follow,base:3},Input::BaseTouched{t:7.,agent:follow,base:4,holding_ball:false},Input::BallTouched{t:8.,what:Touch::Ground{x:20.,y:70.}}] {es.extend(step(&mut st,&input));}
    assert_eq!(st.status,Status::Final);assert_eq!(st.score(),[2,3]);
    append(&mut w,index,&es,Some((batter,true,vec![(lead,3),(follow,2),(batter,0)])),false);
    let bytes=w.finish(2,3).0;let view=log::read(&bytes).unwrap();assert!(view.plays.len() > 50);
    let b=bb_stats::box_score(&bytes).unwrap();
    for side in 0..2 {let t=&b.teams[side];let outs=b.outs_by_half.iter().skip(side).step_by(2).sum::<u32>();assert_eq!(t.ab+t.bb+t.hbp+t.sh+t.sf,t.runs+t.left_on_base+outs,"I1 side {side}");}
    assert_eq!(b.teams[1].left_on_base,2,"winning run only; the trailing runner and batter survive");
}

/// Run one walk-off play from a constructed state (the runners already on base are logged as
/// having reached it) and return the scorer's LOB for the home team. `evidence` is the batter,
/// whether he touched first, and each runner's base at the pitch.
fn walk_off_lob(setup: impl FnOnce(&mut RulesState), inputs: Vec<Input>, evidence: Option<(u16, Vec<(u16, u8)>)>) -> u32 {
    let mut f = FlatBufferBuilder::new();
    let p = bb_schema::ProducerInfo::create(&mut f, &bb_schema::ProducerInfoArgs { schema_major: bb_schema::SCHEMA_MAJOR, schema_minor: bb_schema::SCHEMA_MINOR, ..Default::default() });
    let h = ev::LogHeader::create(&mut f, &ev::LogHeaderArgs { producer: Some(p), ..Default::default() });
    f.finish_size_prefixed(h, Some(log::HEADER_ID));
    let mut w = log::LogWriter::new(f.finished_data());
    let mut st = RulesState::default_fixture();
    st.status = Status::InProgress;
    st.half = Half::Bottom;
    st.runs_by_inning = [vec![0; 9], vec![0; 9]];
    setup(&mut st);
    let mut es = vec![Event::HalfInningStarted { inning: st.inning, half: st.half }];
    for (i, b) in st.bases.iter().enumerate() {
        if let Some(r) = b { es.push(Event::RunnerAdvanced { runner: *r, from: 0, to: i as u8 + 1, cause: AdvanceCause::Walk }); }
    }
    append(&mut w, 0, &es, None, false);
    let mut es = Vec::new();
    for i in &inputs { es.extend(step(&mut st, i)); }
    assert_eq!(st.status, Status::Final);
    let batter = evidence.as_ref().map(|e| e.0);
    append(&mut w, 1, &es, evidence.map(|(b, rs)| (b, true, rs)), false);
    let _ = batter;
    let [a, h] = st.score();
    let bytes = w.finish(a as u8, h as u8).0;
    let b = box_score(&bytes).unwrap();
    b.teams[1].left_on_base
}

const H1: u16 = 257;
const H2: u16 = 258;
const H3: u16 = 259;
const H4: u16 = 260;

/// The walk-off scenarios Codex named (awards-014, awards-015, gameflow-005, gameflow-007): the
/// runners who neither scored nor were out are left on base, by 9.02(g): the batter-runner
/// included, and none of them is lost because the game ended.
#[test]
fn walk_off_scenarios_count_the_runners_left_on_base() {
    let bases = |s: &mut RulesState, inning: u8, outs: u8, score: [u8; 2], b: [Option<u16>; 3], batter_slot: usize| {
        s.inning = inning; s.outs = outs; s.runs_by_inning[0][0] = score[0]; s.runs_by_inning[1][0] = score[1]; s.bases = b; s.next_up[1] = batter_slot - 1; s.batter = s.lineup[1][batter_slot - 1];
    };
    // awards-014: ground-rule double, runners on second and third, one out: H.3 scores, H.2 and
    // the batter (awarded second) are left.
    let lob = walk_off_lob(|s| bases(s, 9, 1, [3, 3], [None, Some(H2), Some(H3)], 4), vec![
        Input::PitchDelivered { t: 0.0 }, Input::BatContact { t: 0.42 }, Input::BallTouched { t: 3.0, what: Touch::Ground { x: 30.0, y: 80.0 } },
        Input::BallTouched { t: 4.4, what: Touch::OutOfPlay { x: 45.0, y: 102.0, z: 0.0, in_flight: false } }], Some((H4, vec![(H3, 3), (H2, 2), (H4, 0)])));
    assert_eq!(lob, 2, "awards-014");
    // awards-015: throw into the dugout, runners on first and third: H.3 scores, H.1 and the batter are left.
    let lob = walk_off_lob(|s| bases(s, 10, 0, [5, 5], [Some(H1), None, Some(H3)], 2), vec![
        Input::PitchDelivered { t: 0.0 }, Input::BatContact { t: 0.42 }, Input::BallTouched { t: 0.8, what: Touch::Ground { x: -6.0, y: 25.0 } },
        Input::GloveSecured { t: 1.5, fielder: (0 << 8) | 0x80 | 6, x: -10.0, y: 38.0 },
        Input::BallReleasedFromGlove { t: 2.0, fielder: (0 << 8) | 0x80 | 6, kind: ReleaseKind::Throw, infielder: true },
        Input::BallTouched { t: 2.8, what: Touch::OutOfPlay { x: 25.0, y: 5.0, z: 0.0, in_flight: true } }], Some((H2, vec![(H3, 3), (H1, 1), (H2, 0)])));
    assert_eq!(lob, 2, "awards-015");
    // gameflow-005: bases-loaded walk: H.3 scores; H.1, H.2 and the batter are left.
    let lob = walk_off_lob(|s| { bases(s, 9, 1, [3, 3], [Some(H1), Some(H2), Some(H3)], 4); s.balls = 3; s.strikes = 2; }, vec![
        Input::PitchDelivered { t: 0.0 }, Input::PitchCrossed { t: 0.4, in_zone: false, bounced: false }, Input::SwingMeasured { t: 0.45, swung: false }], None);
    assert_eq!(lob, 3, "gameflow-005");
    // gameflow-007: single, H.3 scores from third, H.2 and the batter are left.
    let lob = walk_off_lob(|s| bases(s, 9, 0, [2, 2], [None, Some(H2), Some(H3)], 4), vec![
        Input::PitchDelivered { t: 0.0 }, Input::BatContact { t: 0.42 }, Input::BaseLeft { t: 0.5, agent: H3, base: 3 }, Input::BaseLeft { t: 0.5, agent: H2, base: 2 },
        Input::BallTouched { t: 1.6, what: Touch::Ground { x: -20.0, y: 55.0 } }, Input::BaseTouched { t: 2.2, agent: H3, base: 4, holding_ball: false },
        Input::BaseTouched { t: 2.4, agent: H2, base: 3, holding_ball: false }, Input::BaseLeft { t: 2.5, agent: H2, base: 3 },
        Input::BaseTouched { t: 4.0, agent: H2, base: 4, holding_ball: false }, Input::BaseTouched { t: 4.1, agent: H4, base: 1, holding_ball: false }, Input::PlayEndCheck { t: 6.0 }], Some((H4, vec![(H3, 3), (H2, 2), (H4, 0)])));
    assert_eq!(lob, 2, "gameflow-007");
}
