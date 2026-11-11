//! A full game, pitch by pitch (M4 walking skeleton): the plate-appearance engine for each
//! pitch, the live-ball play simulator for balls in play, and the rules engine judging every
//! fact. AI vs AI, one pitcher per side, average players, no substitutions.

use bb_agents::batter::Scouting;
use bb_observe::PitchType;
use bb_physics::ground::Field;
use bb_rules::{Event, Input, PaResult, RulesState, Status, step};

use crate::pa::{Knobs, Matchup, scouting_for_pa, simulate_pitch_ctx};
use crate::play::{FielderAttrs, RunnerIn};

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
    /// False if the game hit the operational inning budget before ending.
    pub complete: bool,
    /// Every ball in play, in order.
    pub bip_detail: Vec<BipDetail>,
}

/// One ball in play and what the defense did with it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BipDetail {
    pub exit_speed_mps: f64,
    pub launch_rad: f64,
    pub spray_rad: f64,
    /// The batter was not put out on the play.
    pub batter_safe: bool,
    /// The batter reached first base safely (he may have been put out later, stretching).
    pub reached_first: bool,
    /// Interim hit ruling (see [`classify_bip`]); `reached_first && !hit` is a fielder's choice.
    pub hit: bool,
    /// Outs recorded on the play (any runner).
    pub outs: u8,
}

impl BipDetail {
    /// The batter reached first because the defense put out another runner instead.
    pub fn fielders_choice(&self) -> bool {
        self.reached_first && !self.hit
    }
}

/// Interim hit / fielder's-choice ruling for one ball in play, from the play's ordered facts
/// and rules events: `(reached_first, hit)`. The official scorer (rules-spec §10, with
/// `ScorerEvidence` and errors) replaces this; until then the sim has no errors, so:
///
/// - The batter **reached first** unless he was put out before getting there: a fly out or a
///   force, or a tag before he touched the bag. A batter tagged out after touching first
///   (stretching a hit) did reach.
/// - It is a **hit** unless another runner was forced out (OBR 9.05(b)(1)), or another runner
///   was tagged out before the batter touched first (the defense took that out while the
///   batter could still have been played on). A runner thrown out advancing after the batter
///   was already on first is an out on the bases, and the hit stands.
///
/// A tag with no timed fact, or a batter who never touched first (a base award), counts the
/// conservative way (not a hit) if another runner was tagged out.
pub fn classify_bip(batter: bb_rules::AgentId, events: &[Event], facts: &[Input]) -> (bool, bool) {
    use bb_rules::OutKind;
    let t_first = facts.iter().find_map(|f| match f {
        Input::BaseTouched { t, agent, base: 1, holding_ball: false } if *agent == batter => Some(*t),
        _ => None,
    });
    let outs = || events.iter().filter_map(|e| if let Event::RunnerOut { runner, kind, .. } = e { Some((*runner, *kind)) } else { None });
    let reached_first = match outs().find(|(r, _)| *r == batter) {
        None => true,
        Some((_, OutKind::Tag | OutKind::Appeal)) => t_first.is_some(),
        Some(_) => false,
    };
    let tag_time = |runner: bb_rules::AgentId| {
        facts.iter().find_map(|f| match f {
            Input::TagContact { t, runner: r, holding_ball: true, .. } if *r == runner => Some(*t),
            _ => None,
        })
    };
    let taken_instead = outs().filter(|(r, _)| *r != batter).any(|(r, kind)| match kind {
        OutKind::Force => true,
        OutKind::Tag => match (tag_time(r), t_first) {
            (Some(t_tag), Some(t1)) => t_tag < t1,
            _ => true,
        },
        _ => false,
    });
    (reached_first, reached_first && !taken_instead)
}

/// One pitch's paths for visualization: samples are (t, x, y, z) in play time.
#[derive(Clone, Debug, Default)]
pub struct PlayViz {
    pub play: u32,
    pub label: String,
    pub pitch: Vec<[f64; 4]>,
    pub batted: Vec<[f64; 4]>,
    /// Fielder and runner positions (agent id, samples) during a live ball.
    pub actors: Vec<(u16, Vec<(f64, bb_math::Vec3)>)>,
}

/// Everything one pitch produced, for timelines and visualization.
#[derive(Clone, Debug)]
pub struct PitchStep {
    pub play: u32,
    pub record: crate::pa::PitchRecord,
    /// The pitch's flight (to contact, the ground, or past the catcher).
    pub pitch_flight: bb_physics::Trajectory,
    /// The batted ball's full path, if there was contact.
    pub batted: Option<bb_physics::ground::BallPath>,
    pub rules_events: Vec<Event>,
    /// Every fact sent to the rules engine for this pitch.
    pub facts: Vec<Input>,
    pub viz: PlayViz,
}

/// A game that advances one pitch at a time (the engine behind `simulate_game` and the
/// client API, data-formats §6.4).
pub struct Game {
    mu: Matchup,
    field: Field,
    pub st: RulesState,
    res: GameResult,
    log: Vec<Event>,
    writer: bb_schema::log::LogWriter,
    pending_rules: Vec<Event>,
    sc: Vec<Vec<Scouting>>,
    bs: Vec<f64>,
    fattrs: [FielderAttrs; 9],
    play: u32,
    prev: Option<PitchType>,
    last_v: Option<f64>,
    last_batter: u16,
    seed: u64,
    record: bool,
}

impl Game {
    pub fn new(seed: u64, record: bool) -> Game {
        let mut mu = Matchup::average();
        mu.knobs = Knobs::calibrated_m3();
        let mut st = RulesState::default_fixture();
        let log = step(&mut st, &Input::StartGame);
        // Scouting per batting slot, built once per game (model error per batter–pitcher pair).
        let sc: Vec<Vec<Scouting>> = (0..2u32).map(|side| (0..9u32).map(|slot| scouting_for_pa(&mu, seed, 9_000_000 + side * 100 + slot).into_owned()).collect()).collect();
        let bs = vec![mu.perception.break_sigma; mu.arsenal.len()];
        Game {
            field: Field::default(),
            res: GameResult { complete: true, ..Default::default() },
            pending_rules: log.clone(),
            log,
            writer: bb_schema::log::LogWriter::new(&crate::eventlog::header(seed)),
            sc,
            bs,
            fattrs: core::array::from_fn(FielderAttrs::average),
            play: 0,
            prev: None,
            last_v: None,
            last_batter: 0,
            seed,
            record,
            st,
            mu,
        }
    }

    pub fn finished(&self) -> bool {
        self.st.status != Status::InProgress || self.st.inning > MAX_INNINGS
    }

    /// Advance one pitch (and the play it starts). Returns `None` once the game is over.
    pub fn step_pitch(&mut self) -> Option<PitchStep> {
        if self.st.status != Status::InProgress {
            return None;
        }
        if self.st.inning > MAX_INNINGS {
            // Operational budget, not a baseball rule: the result is marked incomplete.
            self.res.complete = false;
            return None;
        }
        let (mu, st, seed, play, record) = (&self.mu, &mut self.st, self.seed, self.play, self.record);
        let side = st.half.batting();
        let slot = st.next_up[side];
        if st.batter != self.last_batter {
            self.prev = None;
            self.last_v = None;
            self.last_batter = st.batter;
        }
        let count = bb_observe::Count { balls: st.balls, strikes: st.strikes };
        let rec = simulate_pitch_ctx(mu, &self.sc[side][slot], count, seed, play, &self.bs, self.prev, self.last_v);
        self.prev = Some(rec.read);
        self.last_v = Some(rec.perceived_speed);
        self.res.pitches += 1;
        let mut viz = PlayViz { play, ..Default::default() };
        let pitch_flight = bb_world::pitch::pitch_flight(&mu.env, rec.release);
        if record {
            let t_end = rec.contact_state.map(|c| c.0).unwrap_or(pitch_flight.last().t);
            let mut t = 0.0;
            while t <= t_end {
                let (p, _) = pitch_flight.at(t);
                viz.pitch.push([t, p.x, p.y, p.z]);
                t += 0.01;
            }
        }
        let mut facts = vec![Input::PitchDelivered { t: 0.0 }];
        // Facts carry real times (Codex M4 review #22). A pitch hit before the plate never
        // crossed it.
        let t_plate = rec.t_plate.unwrap_or_else(|| pitch_flight.last().t);
        let t_contact = rec.contact_state.map(|c| c.0);
        if t_contact.map(|tc| tc > t_plate).unwrap_or(true) {
            facts.push(Input::PitchCrossed { t: t_plate, in_zone: rec.in_zone, bounced: rec.bounced });
        }
        let mut batted = None;
        if rec.hit_batter {
            facts.push(Input::BallTouched { t: t_plate, what: bb_rules::Touch::Batter });
            facts.push(Input::SwingMeasured { t: t_plate + 0.03, swung: rec.swung });
        } else if rec.contact_state.is_none() {
            facts.push(Input::SwingMeasured { t: t_plate + 0.03, swung: rec.swung });
        } else if let Some((t_c, _)) = rec.contact_state {
            facts.push(Input::BatContact { t: t_c });
        }
        for f in &facts {
            let ev = step(st, f);
            self.pending_rules.extend(ev.iter().cloned());
            self.log.extend(ev);
            if st.status == Status::Final {
                break;
            }
        }
        if let (Some((t_c, ball)), bb_rules::PlayState::Live(_)) = (rec.contact_state, &st.play) {
            // Runners: batter plus those on base (ids from the rules state).
            let mut runners = vec![RunnerIn { id: st.batter, base: 0, speed: 8.2 }];
            for (i, r) in st.bases.iter().enumerate() {
                if let Some(r) = r {
                    runners.push(RunnerIn { id: *r, base: (i + 1) as u8, speed: 8.2 });
                }
            }
            let fside = 1 - side as u16;
            let fielders: [u16; 9] = core::array::from_fn(|i| (fside << 8) | 0x80 | (i as u16 + 1));
            let out = crate::live::simulate_live(
                &crate::live::LiveInput { env: &mu.env, field: &self.field, ball, t_contact: t_c, runners, fielders, fattrs: self.fattrs, seed, play, record },
                st,
            );
            if record {
                for (t, p) in out.ball_track.iter().step_by(2) {
                    viz.batted.push([*t, p.x, p.y, p.z]);
                }
                viz.actors = out.actors.clone();
            }
            self.pending_rules.extend(out.events.iter().cloned());
            self.log.extend(out.events);
            facts.extend(out.facts);
            batted = Some(out.path);
        }
        if record {
            let who = if side == 0 { "Away" } else { "Home" };
            let mut lbl = format!("{} {}{} {} #{}: {} {:.0} mph", play, if st.half == bb_rules::Half::Top { "T" } else { "B" }, st.inning, who, slot + 1, rec.ty.code(), bb_math::units::mps_to_mph(rec.release.v.norm()));
            if let Some((ev_, la, spray)) = rec.batted {
                lbl += &format!(" -> EV {:.0} mph, LA {:.0}°, spray {:.0}°", bb_math::units::mps_to_mph(ev_), la.to_degrees(), spray.to_degrees());
            } else {
                lbl += &format!(" -> {:?}", rec.call);
            }
            for e in &self.pending_rules {
                match e {
                    Event::PlateAppearanceEnded { result, .. } => lbl += &format!(" | PA: {result:?}"),
                    Event::RunScored { .. } => lbl += " | run scores",
                    Event::RunnerOut { kind, .. } => lbl += &format!(" | out ({kind:?})"),
                    _ => {}
                }
            }
            viz.label = lbl;
            self.res.viz.push(viz.clone());
        }
        if matches!(st.play, bb_rules::PlayState::Live(_)) {
            // The play did not settle from its facts: a liveness failure, reported to the rules
            // engine as a timeout rather than repaired here (rules-spec §1.1, §5.8).
            if std::env::var("BB_DEBUG").is_ok() && self.res.warnings.iter().filter(|w| **w == "unsettled_play").count() < 2 {
                eprintln!("--- unsettled play {play}: facts");
                for f in &facts {
                    eprintln!("  {f:?}");
                }
                eprintln!("  state: {:?}", st.play);
            }
            self.res.warnings.push("unsettled_play");
            let t_end = facts.iter().filter_map(fact_t).fold(0.0, f64::max) + 1.0;
            let ev = step(st, &Input::PlayTimeout { t: t_end });
            self.log.extend(ev.iter().cloned());
            self.pending_rules.extend(ev);
        }
        let pitcher = ((1 - side as u16) << 8) | 0x81;
        crate::eventlog::write_pitch(&mut self.writer, play, pitcher, &rec, &self.pending_rules);
        let rules_events = std::mem::take(&mut self.pending_rules);
        if let Some((ev_, la, spray)) = rec.batted {
            if let Some(Event::PlateAppearanceEnded { batter, result: PaResult::BattedBall }) = rules_events.iter().find(|e| matches!(e, Event::PlateAppearanceEnded { .. })) {
                let out = rules_events.iter().any(|e| matches!(e, Event::RunnerOut { runner, .. } if runner == batter));
                let outs = rules_events.iter().filter(|e| matches!(e, Event::RunnerOut { .. })).count() as u8;
                let (reached_first, hit) = classify_bip(*batter, &rules_events, &facts);
                self.res.bip_detail.push(BipDetail { exit_speed_mps: ev_, launch_rad: la, spray_rad: spray, batter_safe: !out, reached_first, hit, outs });
            }
        }
        self.play += 1;
        Some(PitchStep { play, record: rec, pitch_flight, batted, rules_events, facts, viz })
    }

    /// Close the log and tally the result from the rules events (event sourcing, PLAN §4.3).
    pub fn finish(self) -> GameResult {
        let mut res = self.res;
        let mut pa_events: Vec<Event> = Vec::new();
        for e in &self.log {
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
        res.linescore = self.st.runs_by_inning.clone();
        res.runs = self.st.score();
        let (bytes, digest) = self.writer.finish(res.runs[0].min(255) as u8, res.runs[1].min(255) as u8);
        res.log = bytes;
        res.digest = digest;
        res.innings = self.st.inning;
        res.events = self.log;
        res
    }
}

/// Operational budget: a game still going after this many innings stops and is reported
/// incomplete (the rules engine itself warns at inning 30 and would continue).
pub const MAX_INNINGS: u8 = 99;

fn fact_t(f: &Input) -> Option<f64> {
    match *f {
        Input::PitchDelivered { t }
        | Input::PitchCrossed { t, .. }
        | Input::SwingMeasured { t, .. }
        | Input::BatContact { t }
        | Input::BallTouched { t, .. }
        | Input::BallPassedBase { t, .. }
        | Input::BallSettled { t, .. }
        | Input::PlayTimeout { t }
        | Input::GloveSecured { t, .. }
        | Input::BaseTouched { t, .. }
        | Input::BaseLeft { t, .. }
        | Input::TagContact { t, .. }
        | Input::PlayEndCheck { t } => Some(t),
        Input::StartGame => None,
    }
}

/// Simulate one game with average players on both sides.
pub fn simulate_game(seed: u64) -> GameResult {
    simulate_game_with(seed, false)
}

/// [`simulate_game`], optionally recording paths for visualization (recording never changes
/// the simulation: data-formats §5.5).
pub fn simulate_game_with(seed: u64, record: bool) -> GameResult {
    let mut g = Game::new(seed, record);
    while g.step_pitch().is_some() {}
    g.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bb_rules::OutKind;

    const BATTER: u16 = 2;
    const R3: u16 = 5;
    fn touch(t: f64, agent: u16, base: u8) -> Input {
        Input::BaseTouched { t, agent, base, holding_ball: false }
    }
    fn tag(t: f64, runner: u16) -> Input {
        Input::TagContact { t, fielder: 0x186, runner, holding_ball: true }
    }
    fn out(runner: u16, base: u8, kind: OutKind) -> Event {
        Event::RunnerOut { runner, base, kind }
    }

    /// Scoring expectations come from OBR 9.05, not from the code: each case names the ruling
    /// an official scorer would make (no errors in the sim).
    #[test]
    fn interim_hit_rulings_follow_the_scoring_rules() {
        // Clean single.
        assert_eq!(classify_bip(BATTER, &[], &[touch(4.3, BATTER, 1)]), (true, true));
        // Single, batter thrown out stretching to second: a hit, with an out on the bases
        // (Codex day review #4, seed 0 play 37).
        assert_eq!(classify_bip(BATTER, &[out(BATTER, 2, OutKind::Tag)], &[touch(5.01, BATTER, 1), tag(8.2, BATTER)]), (true, true));
        // Single, runner from second thrown out at home after the batter reached: a hit
        // (review #4, seed 2 play 209).
        assert_eq!(classify_bip(BATTER, &[out(R3, 4, OutKind::Tag)], &[touch(5.05, BATTER, 1), tag(7.9, R3)]), (true, true));
        // Grounder, runner forced at second, batter beats the relay: fielder's choice.
        assert_eq!(classify_bip(BATTER, &[out(R3, 2, OutKind::Force)], &[touch(4.4, BATTER, 1)]), (true, false));
        // Outfield single, but the runner from first is forced at second: no hit (9.05(b)(1)).
        assert_eq!(classify_bip(BATTER, &[out(R3, 2, OutKind::Force)], &[touch(4.4, BATTER, 1)]).1, false);
        // Grounder, runner tagged between third and home before the batter reaches first:
        // fielder's choice.
        assert_eq!(classify_bip(BATTER, &[out(R3, 4, OutKind::Tag)], &[tag(3.1, R3), touch(4.4, BATTER, 1)]), (true, false));
        // Ground out at first; fly out (even if the batter crossed first before the catch);
        // batter tagged on the line.
        assert_eq!(classify_bip(BATTER, &[out(BATTER, 1, OutKind::Force)], &[]), (false, false));
        assert_eq!(classify_bip(BATTER, &[out(BATTER, 0, OutKind::FlyOut)], &[touch(4.4, BATTER, 1)]), (false, false));
        assert_eq!(classify_bip(BATTER, &[out(BATTER, 1, OutKind::Tag)], &[tag(3.9, BATTER)]), (false, false));
    }
}
