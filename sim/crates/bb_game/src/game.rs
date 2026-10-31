//! A full game, pitch by pitch (M4 walking skeleton): the plate-appearance engine for each
//! pitch, the live-ball play simulator for balls in play, and the rules engine judging every
//! fact. AI vs AI, one pitcher per side, average players, no substitutions.

use bb_agents::batter::Scouting;
use bb_observe::PitchType;
use bb_physics::ground::Field;
use bb_rules::{Event, Input, PaResult, RulesState, Status, step};

use crate::pa::{Knobs, Matchup, scouting_for_pa, simulate_pitch_ctx};
use crate::play::{FielderAttrs, PlayInput, RunnerIn, simulate_play};

/// Game summary for reports and tests.
#[derive(Clone, Debug, Default)]
pub struct GameResult {
    pub linescore: [Vec<u8>; 2],
    pub runs: [u16; 2],
    pub innings: u8,
    pub pitches: u32,
    pub pa: u32,
    pub k: u32,
    pub bb: u32,
    pub hbp: u32,
    pub hr: u32,
    /// Batted-ball PAs where the batter reached safely (hits plus fielder's choices; the
    /// official scorer comes later).
    pub reached_bip: u32,
    pub bip: u32,
    pub walk_off: bool,
    pub warnings: Vec<&'static str>,
    pub events: Vec<Event>,
    /// The game's event log (data-formats §4) and its digest (§4.4).
    pub log: Vec<u8>,
    pub digest: [u8; 32],
    /// Visualization record, filled only when requested.
    pub viz: Vec<PlayViz>,
}

/// One pitch's paths for visualization: samples are (t, x, y, z) in play time.
#[derive(Clone, Debug, Default)]
pub struct PlayViz {
    pub play: u32,
    pub label: String,
    pub pitch: Vec<[f64; 4]>,
    pub batted: Vec<[f64; 4]>,
}

/// Simulate one game with average players on both sides.
pub fn simulate_game(seed: u64) -> GameResult {
    simulate_game_with(seed, false)
}

/// [`simulate_game`], optionally recording paths for visualization (recording never changes
/// the simulation: data-formats §5.5).
pub fn simulate_game_with(seed: u64, record: bool) -> GameResult {
    let mut mu = Matchup::average();
    mu.knobs = Knobs::calibrated_m3();
    let field = Field::default();
    let mut st = RulesState::default_fixture();
    let mut res = GameResult::default();
    let mut log = step(&mut st, &Input::StartGame);
    let mut writer = bb_schema::log::LogWriter::new(&crate::eventlog::header(seed));
    let mut pending_rules: Vec<Event> = log.clone();
    // Scouting per batting slot, built once per game (model error per batter–pitcher pair).
    let sc: Vec<Vec<Scouting>> = (0..2u32).map(|side| (0..9u32).map(|slot| scouting_for_pa(&mu, seed, 9_000_000 + side * 100 + slot).into_owned()).collect()).collect();
    let bs = vec![mu.perception.break_sigma; mu.arsenal.len()];
    let fattrs: [FielderAttrs; 9] = core::array::from_fn(FielderAttrs::average);
    let mut play: u32 = 0;
    let mut prev: Option<PitchType> = None;
    let mut last_batter = 0;
    while st.status == Status::InProgress {
        if st.inning > 30 {
            res.warnings.push("liveness_inning_30");
            break;
        }
        let side = st.half.batting();
        let slot = st.next_up[side];
        if st.batter != last_batter {
            prev = None;
            last_batter = st.batter;
        }
        let count = bb_observe::Count { balls: st.balls, strikes: st.strikes };
        let rec = simulate_pitch_ctx(&mu, &sc[side][slot], count, seed, play, &bs, prev);
        prev = Some(rec.read);
        res.pitches += 1;
        let mut viz = PlayViz { play, ..Default::default() };
        if record {
            let fl = bb_world::pitch::pitch_flight(&mu.env, rec.release);
            let t_end = rec.contact_state.map(|c| c.0).unwrap_or(fl.last().t);
            let mut t = 0.0;
            while t <= t_end {
                let (p, _) = fl.at(t);
                viz.pitch.push([t, p.x, p.y, p.z]);
                t += 0.01;
            }
        }
        let mut facts = vec![Input::PitchDelivered { t: 0.0 }];
        let t_plate = 0.42;
        facts.push(Input::PitchCrossed { t: t_plate, in_zone: rec.in_zone, bounced: rec.bounced });
        if rec.hit_batter {
            facts.push(Input::BallTouched { t: t_plate, what: bb_rules::Touch::Batter });
        } else if let Some((t_c, ball)) = rec.contact_state {
            facts.push(Input::BatContact { t: t_c });
            // Runners: batter plus those on base (ids from the rules state).
            let mut runners = vec![RunnerIn { id: st.batter, base: 0, speed: 8.2 }];
            for (i, r) in st.bases.iter().enumerate() {
                if let Some(r) = r {
                    runners.push(RunnerIn { id: *r, base: (i + 1) as u8, speed: 8.2 });
                }
            }
            let fside = 1 - side as u16;
            let fielders: [u16; 9] = core::array::from_fn(|i| (fside << 8) | 0x80 | (i as u16 + 1));
            let out = simulate_play(&PlayInput {
                env: &mu.env,
                field: &field,
                ball,
                t_contact: t_c,
                runners,
                outs: st.outs,
                fielders,
                fattrs,
                seed,
                play,
            });
            if record {
                let (t0, t1) = (out.path.t_start(), out.path.t_end().min(out.path.t_start() + 15.0));
                let mut t = t0;
                while t <= t1 {
                    let (p, _) = out.path.at(t);
                    viz.batted.push([t, p.x, p.y, p.z]);
                    t += 0.03;
                }
            }
            facts.extend(out.facts);
        } else {
            facts.push(Input::SwingMeasured { t: 0.45, swung: rec.swung });
        }
        for f in &facts {
            let ev = step(&mut st, f);
            pending_rules.extend(ev.iter().cloned());
            log.extend(ev);
            if st.status == Status::Final {
                break;
            }
        }
        if record {
            let who = if side == 0 { "Away" } else { "Home" };
            let mut lbl = format!("{} {}{} {} #{}: {} {:.0} mph", play, if st.half == bb_rules::Half::Top { "T" } else { "B" }, st.inning, who, slot + 1, rec.ty.code(), bb_math::units::mps_to_mph(rec.release.v.norm()));
            if let Some((ev_, la, spray)) = rec.batted {
                lbl += &format!(" -> EV {:.0} mph, LA {:.0}°, spray {:.0}°", bb_math::units::mps_to_mph(ev_), la.to_degrees(), spray.to_degrees());
            } else {
                lbl += &format!(" -> {:?}", rec.call);
            }
            for e in &pending_rules {
                match e {
                    Event::PlateAppearanceEnded { result, .. } => lbl += &format!(" | PA: {result:?}"),
                    Event::RunScored { .. } => lbl += " | run scores",
                    Event::RunnerOut { kind, .. } => lbl += &format!(" | out ({kind:?})"),
                    _ => {}
                }
            }
            viz.label = lbl;
            res.viz.push(viz);
        }
        let pitcher = ((1 - side as u16) << 8) | 0x81;
        crate::eventlog::write_pitch(&mut writer, play, pitcher, &rec, &pending_rules);
        pending_rules.clear();
        if matches!(st.play, bb_rules::PlayState::Live(_)) {
            // The play did not settle from its facts: report and close it out.
            if std::env::var("BB_DEBUG").is_ok() && res.warnings.iter().filter(|w| **w == "unsettled_play").count() < 2 {
                eprintln!("--- unsettled play {play}: facts");
                for f in &facts {
                    eprintln!("  {f:?}");
                }
                eprintln!("  state: {:?}", st.play);
            }
            res.warnings.push("unsettled_play");
            st.play = bb_rules::PlayState::Dead;
        }
        play += 1;
    }
    // Tally from the rules events (event sourcing, PLAN §4.3).
    let mut pa_events: Vec<Event> = Vec::new();
    for e in &log {
        match e {
            Event::PlateAppearanceEnded { batter, result } => {
                res.pa += 1;
                match result {
                    PaResult::Strikeout => res.k += 1,
                    PaResult::Walk => res.bb += 1,
                    PaResult::HitByPitch => res.hbp += 1,
                    PaResult::HomeRun => res.hr += 1,
                    PaResult::BattedBall => {
                        res.bip += 1;
                        let out = pa_events.iter().any(|x| matches!(x, Event::RunnerOut { runner, .. } if runner == batter));
                        if !out {
                            res.reached_bip += 1;
                        }
                    }
                }
                pa_events.clear();
            }
            Event::GameEnded { walk_off, .. } => res.walk_off = *walk_off,
            Event::RulesWarning { code } => res.warnings.push(code),
            other => pa_events.push(other.clone()),
        }
    }
    res.linescore = st.runs_by_inning.clone();
    res.runs = st.score();
    let (bytes, digest) = writer.finish(res.runs[0].min(255) as u8, res.runs[1].min(255) as u8);
    res.log = bytes;
    res.digest = digest;
    res.innings = st.inning;
    res.events = log;
    res
}
