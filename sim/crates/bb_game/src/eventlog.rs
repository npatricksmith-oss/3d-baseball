//! Event-log encoding for games (data-formats §4): one `PlayRecord` per pitch with the
//! physical facts and the rules events, chained through the BLAKE3 digest.

use bb_rules::{Event as RE, OutKind, PaResult, PitchCall};
use bb_schema::ev::{self, EventBody, EventArgs};
use bb_schema::log::{self, LogWriter};
use bb_schema::{ProducerInfo, ProducerInfoArgs, vec3d};
use flatbuffers::{FlatBufferBuilder, WIPOffset};

use crate::pa::PitchRecord;
use crate::roster::Rosters;

/// The log header: producer, seed, and the game's configuration, which today is its rosters
/// (data-formats §2.4, §4.1). The header is outside the digest (§4.4).
pub fn header(game_seed: u64, rosters: &Rosters) -> Vec<u8> {
    use bb_schema::cfg;
    let mut fbb = FlatBufferBuilder::new();
    let hand = |h: bb_observe::Hand| match h {
        bb_observe::Hand::Left => bb_schema::Hand::Left,
        bb_observe::Hand::Right => bb_schema::Hand::Right,
    };
    let bats = |b: bb_observe::Bats| match b {
        bb_observe::Bats::Left => bb_schema::Hand::Left,
        bb_observe::Bats::Right => bb_schema::Hand::Right,
        bb_observe::Bats::Switch => bb_schema::Hand::Switch,
    };
    let teams = [0, 1].map(|s| {
        let t = &rosters.teams[s];
        let entries: Vec<_> = t
            .players
            .iter()
            .map(|p| {
                let name = fbb.create_string(&p.full_name());
                let family = fbb.create_string(p.family);
                cfg::RosterEntry::create(
                    &mut fbb,
                    &cfg::RosterEntryArgs {
                        agent_id: t.bat_agent(p),
                        field_agent_id: t.field_agent(p),
                        player_id: p.player_id,
                        name: Some(name),
                        family_name: Some(family),
                        name_origin: p.origin as u8,
                        position: bb_schema::Position(p.position),
                        batting_order: p.batting_order,
                        uniform_number: p.number,
                        bats: bats(p.bats),
                        throws: hand(p.throws),
                        ..Default::default()
                    },
                )
            })
            .collect();
        let roster = fbb.create_vector(&entries);
        let name = fbb.create_string(t.name);
        let pitcher = t.players.iter().find(|p| p.position == 1).map(|p| t.field_agent(p)).unwrap_or(0);
        cfg::TeamConfig::create(&mut fbb, &cfg::TeamConfigArgs { name: Some(name), roster: Some(roster), starting_pitcher: pitcher, ..Default::default() })
    });
    let config = cfg::GameConfig::create(&mut fbb, &cfg::GameConfigArgs { game_seed, away: Some(teams[0]), home: Some(teams[1]), ..Default::default() });
    let sv = fbb.create_string(env!("CARGO_PKG_VERSION"));
    let p = ProducerInfo::create(
        &mut fbb,
        &ProducerInfoArgs { schema_major: bb_schema::SCHEMA_MAJOR, schema_minor: bb_schema::SCHEMA_MINOR, sim_version: Some(sv), ..Default::default() },
    );
    let h = ev::LogHeader::create(&mut fbb, &ev::LogHeaderArgs { producer: Some(p), game_id: 0, game_seed, config: Some(config) });
    fbb.finish_size_prefixed(h, Some(log::HEADER_ID));
    fbb.finished_data().to_vec()
}

fn out_kind(k: OutKind) -> ev::OutKind {
    match k {
        OutKind::Strikeout => ev::OutKind::Strikeout,
        OutKind::FlyOut => ev::OutKind::FlyOut,
        OutKind::Force => ev::OutKind::Force,
        OutKind::Tag => ev::OutKind::Tag,
        OutKind::Appeal => ev::OutKind::Appeal,
    }
}

fn call(c: PitchCall) -> ev::PitchOutcome {
    match c {
        PitchCall::Ball => ev::PitchOutcome::Ball,
        PitchCall::CalledStrike => ev::PitchOutcome::CalledStrike,
        PitchCall::SwingingStrike => ev::PitchOutcome::SwingingStrike,
        PitchCall::Foul => ev::PitchOutcome::Foul,
        PitchCall::InPlay => ev::PitchOutcome::InPlay,
        PitchCall::HitByPitch => ev::PitchOutcome::HitByPitch,
    }
}

fn pa_result(r: PaResult) -> ev::PaResult {
    match r {
        PaResult::Walk => ev::PaResult::Walk,
        PaResult::HitByPitch => ev::PaResult::HitByPitch,
        PaResult::Strikeout => ev::PaResult::Strikeout,
        PaResult::HomeRun => ev::PaResult::HomeRun,
        // Classified by the official scorer later (rules-spec §10).
        PaResult::BattedBall => ev::PaResult::Unknown,
    }
}

/// Evidence gathered after a live play, independent of the rules engine and agents.
pub struct ScorerPlayEvidence {
    pub batter: u16,
    pub runners: Vec<crate::play::RunnerIn>,
    pub first_touch: Option<f64>,
    pub first_handling: Option<(f64, u16)>,
    pub secured: Option<(f64, u16, bb_math::Vec3)>,
    pub margin: Option<f64>,
    pub throws: Vec<crate::live::ScorerThrowFact>,
    pub carries: Vec<crate::live::ScorerCarryFact>,
    pub touches: Vec<(u16, u8, f64)>,
}

/// Append one pitch's record: physical facts first, then the rules events it produced.
pub fn write_pitch(w: &mut LogWriter, play: u32, pitcher: u16, rec: &PitchRecord, rules: &[RE], scorer: Option<&ScorerPlayEvidence>) {
    w.write_play(play, |fbb| {
        let mut items: Vec<(f64, EventBody, WIPOffset<flatbuffers::UnionWIPOffset>)> = Vec::new();
        let (r, v, wv) = (vec3d(rec.release.r.to_array()), vec3d(rec.release.v.to_array()), vec3d(rec.release.w.to_array()));
        let pr = ev::PitchReleased::create(
            fbb,
            &ev::PitchReleasedArgs {
                pitcher,
                pitch_number: play,
                pitch_type: bb_schema::PitchType(rec.ty as u8),
                t_release: 0.0,
                r: Some(&r),
                v: Some(&v),
                w: Some(&wv),
                intent_type: bb_schema::PitchType(rec.ty as u8),
                intent_target_x_m: rec.target.0,
                intent_target_z_m: rec.target.1,
                ..Default::default()
            },
        );
        items.push((0.0, EventBody::PitchReleased, pr.as_union_value()));
        if let Some(p) = rec.plate {
            let pp = vec3d(p.to_array());
            let pc = ev::PitchCrossed::create(fbb, &ev::PitchCrossedArgs { r: Some(&pp), in_zone: rec.in_zone, bounced_before_plate: rec.bounced, ..Default::default() });
            items.push((0.0, EventBody::PitchCrossed, pc.as_union_value()));
        }
        let sw = ev::SwingRecorded::create(
            fbb,
            &ev::SwingRecordedArgs {
                swung: rec.swung,
                check_swing: rec.check_swing,
                judged_swing: rec.swung,
                contact: rec.contact,
                read_type: bb_schema::PitchType(rec.read as u8),
                read_weight: rec.read_weight as f32,
                ..Default::default()
            },
        );
        items.push((0.0, EventBody::SwingRecorded, sw.as_union_value()));
        if let (Some((t, b)), Some((ev_, la, spray))) = (rec.contact_state, rec.batted) {
            let (bv, bw) = (vec3d(b.v.to_array()), vec3d(b.w.to_array()));
            let bb = ev::BattedBall::create(
                fbb,
                &ev::BattedBallArgs { v: Some(&bv), w: Some(&bw), exit_velocity_mps: ev_, launch_angle_rad: la, spray_angle_rad: spray, projected_distance_m: 0.0 },
            );
            items.push((t, EventBody::BattedBall, bb.as_union_value()));
        }
        if let Some(sc) = scorer {
            let runners: Vec<_> = sc.runners.iter().map(|r| ev::RunnerOnBase::create(fbb, &ev::RunnerOnBaseArgs { base: r.base, agent: r.id })).collect();
            let runners = fbb.create_vector(&runners);
            let throws: Vec<_> = sc.throws.iter().map(|t| {
                let origin = vec3d(t.origin.to_array());
                let target = vec3d(t.target.to_array());
                ev::ScorerThrow::create(fbb, &ev::ScorerThrowArgs { thrower: t.thrower, t_release: t.t, origin: Some(&origin), target: Some(&target), base: t.base, attempted_out: t.attempted_out, batter_goal: t.batter_goal })
            }).collect();
            let throws = fbb.create_vector(&throws);
            let carries: Vec<_> = sc.carries.iter().map(|c| ev::ScorerCarry::create(fbb, &ev::ScorerCarryArgs { fielder: c.fielder, t: c.t, base: c.base, batter_goal: c.batter_goal })).collect();
            let carries = fbb.create_vector(&carries);
            let touches: Vec<_> = sc.touches.iter().map(|&(runner, base, t)| ev::ScorerTouch::create(fbb, &ev::ScorerTouchArgs { runner, base, t })).collect();
            let touches = fbb.create_vector(&touches);
            let secured_at = sc.secured.map(|(_, _, p)| vec3d(p.to_array()));
            let x = ev::ScorerEvidence::create(fbb, &ev::ScorerEvidenceArgs {
                batter: sc.batter, runners: Some(runners),
                has_first_touch: sc.first_touch.is_some(), first_touch_s: sc.first_touch.unwrap_or_default(),
                has_handling: sc.first_handling.is_some(), first_handler: sc.first_handling.map(|x| x.1).unwrap_or_default(), handling_s: sc.first_handling.map(|x| x.0).unwrap_or_default(),
                has_secured: sc.secured.is_some(), reference_fielder: sc.secured.map(|x| x.1).unwrap_or_default(), secured_s: sc.secured.map(|x| x.0).unwrap_or_default(), secured_at: secured_at.as_ref(),
                has_margin: sc.margin.is_some(), clean_play_margin_s: sc.margin.unwrap_or_default(), throws: Some(throws), carries: Some(carries), touches: Some(touches), ..Default::default()
            });
            items.push((0.0, EventBody::ScorerEvidence, x.as_union_value()));
        }
        if let Some(fair) = rules.iter().find_map(|e| match e { RE::FairFoulDecided { fair } => Some(*fair), _ => None }) {
            let caught = rules.iter().any(|e| matches!(e, RE::Catch { .. }));
            let ruling = ev::BattedBallRuling::create(fbb, &ev::BattedBallRulingArgs { fair_foul: if fair { ev::FairFoul::Fair } else { ev::FairFoul::Foul }, caught, ..Default::default() });
            items.push((0.0, EventBody::BattedBallRuling, ruling.as_union_value()));
        }
        for e in rules {
            match e {
                RE::PitchResult { call: c, balls, strikes } => {
                    let x = ev::PitchResult::create(fbb, &ev::PitchResultArgs { outcome: call(*c), balls: *balls, strikes: *strikes });
                    items.push((0.0, EventBody::PitchResult, x.as_union_value()));
                }
                RE::RunnerAdvanced { runner, from, to, cause } => {
                    let cause = match cause { bb_rules::AdvanceCause::Walk => ev::AdvanceCause::Walk, bb_rules::AdvanceCause::HitByPitch => ev::AdvanceCause::HitByPitch, bb_rules::AdvanceCause::Award => ev::AdvanceCause::Award, bb_rules::AdvanceCause::OnPlay => ev::AdvanceCause::Unknown };
                    let x = ev::RunnerAdvanced::create(fbb, &ev::RunnerAdvancedArgs { runner: *runner, from_base: *from, to_base: *to, cause, ..Default::default() });
                    items.push((0.0, EventBody::RunnerAdvanced, x.as_union_value()));
                }
                RE::RunnerOut { runner, base, kind } => {
                    let x = ev::RunnerOut::create(fbb, &ev::RunnerOutArgs { runner: *runner, base: *base, kind: out_kind(*kind), ..Default::default() });
                    items.push((0.0, EventBody::RunnerOut, x.as_union_value()));
                }
                RE::RunScored { runner } => {
                    let x = ev::RunScored::create(fbb, &ev::RunScoredArgs { runner: *runner, ..Default::default() });
                    items.push((0.0, EventBody::RunScored, x.as_union_value()));
                }
                RE::PlateAppearanceStarted { batter } => {
                    let x = ev::PlateAppearanceStarted::create(fbb, &ev::PlateAppearanceStartedArgs { batter: *batter, pitcher, ..Default::default() });
                    items.push((0.0, EventBody::PlateAppearanceStarted, x.as_union_value()));
                }
                RE::PlateAppearanceEnded { batter, result } => {
                    let x = ev::PlateAppearanceEnded::create(fbb, &ev::PlateAppearanceEndedArgs { batter: *batter, result: pa_result(*result), ..Default::default() });
                    items.push((0.0, EventBody::PlateAppearanceEnded, x.as_union_value()));
                }
                RE::HalfInningStarted { inning, half } => {
                    let h = if *half == bb_rules::Half::Top { ev::HalfInning::Top } else { ev::HalfInning::Bottom };
                    let x = ev::HalfInningStarted::create(fbb, &ev::HalfInningStartedArgs { inning: *inning, half: h });
                    items.push((0.0, EventBody::HalfInningStarted, x.as_union_value()));
                }
                RE::HalfInningEnded { runs, left_on_base } => {
                    let x = ev::HalfInningEnded::create(fbb, &ev::HalfInningEndedArgs { runs: *runs, left_on_base: *left_on_base });
                    items.push((0.0, EventBody::HalfInningEnded, x.as_union_value()));
                }
                RE::GameEnded { away, home, innings, walk_off } => {
                    let reason = if *walk_off { ev::GameEndReason::WalkOff } else if *innings > 9 { ev::GameEndReason::ExtraInnings } else { ev::GameEndReason::Regulation };
                    let x = ev::GameEnded::create(fbb, &ev::GameEndedArgs { score_away: (*away).min(255) as u8, score_home: (*home).min(255) as u8, innings: *innings, reason });
                    items.push((0.0, EventBody::GameEnded, x.as_union_value()));
                }
                RE::RulesWarning { .. } => {
                    let x = ev::RulesWarning::create(fbb, &ev::RulesWarningArgs { code: 1, detail: None });
                    items.push((0.0, EventBody::RulesWarning, x.as_union_value()));
                }
                _ => {}
            }
        }
        let out: Vec<WIPOffset<ev::Event>> = items
            .into_iter()
            .enumerate()
            .map(|(seq, (t, ty, body))| ev::Event::create(fbb, &EventArgs { seq: seq as u32, t, body_type: ty, body: Some(body) }))
            .collect();
        fbb.create_vector(&out)
    });
}
