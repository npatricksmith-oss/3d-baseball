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

/// Provisional hit / fielder's-choice ruling for one ball in play, from the play's facts and
/// rules events: `(reached_first, hit)`. **A lab diagnostic, not a statistic:** it reads
/// facts that are not in the event log, has no errors, and makes none of the scorer's
/// judgments; the official scorer (rules-spec §10, with `ScorerEvidence`) replaces it.
///
/// - The batter **reached first** if he touched it (or was awarded it) without having been
///   put out on the way: not on a fly out, a force, or a tag short of the bag. A batter
///   tagged out after touching first (stretching a hit) did reach. A batter still short of
///   first when the third out was made elsewhere did not.
/// - It is a **hit** unless another runner was forced out (OBR 9.05(b)(1)), or the batted
///   ball was first fielded by the pitcher, catcher, or an infielder and another runner was
///   put out (9.05(b)(3); the "advancing one base or returning" qualifier and the failed
///   choices of 9.05(b)(4) are not judged). A runner thrown out on a ball first fielded by an
///   outfielder is an out on the bases, and the hit stands.
pub fn classify_bip(batter: bb_rules::AgentId, events: &[Event], facts: &[Input]) -> (bool, bool) {
    use bb_rules::OutKind;
    let touched_first = facts.iter().any(|f| matches!(f, Input::BaseTouched { agent, base: 1, holding_ball: false, .. } if *agent == batter));
    let awarded = events.iter().any(|e| matches!(e, Event::RunnerAdvanced { runner, cause: bb_rules::AdvanceCause::Award, .. } if *runner == batter));
    let outs = || events.iter().filter_map(|e| if let Event::RunnerOut { runner, kind, .. } = e { Some((*runner, *kind)) } else { None });
    let reached_first = (touched_first || awarded) && !matches!(outs().find(|(r, _)| *r == batter), Some((_, OutKind::FlyOut | OutKind::Force | OutKind::Strikeout)));
    // Fielding positions are the low seven bits of a fielder's id: 1–6 are the infield.
    let infield_handled = facts.iter().find_map(|f| if let Input::GloveSecured { fielder, .. } = f { Some(fielder & 0x7F) } else { None }).is_some_and(|pos| pos <= 6);
    let taken_instead = outs().filter(|(r, _)| *r != batter).any(|(_, kind)| kind == OutKind::Force || infield_handled);
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
    /// Where everyone stands as the pitch is thrown: nine fielders, runners on their bases,
    /// the batter in his box.
    pub standing: Vec<(u16, bb_math::Vec3)>,
    /// Possession during a live ball: when each fielder secured the ball, and each throw's release.
    pub secured: Vec<(f64, u16)>,
    pub released: Vec<(f64, u16)>,
    /// Dives (t, fielder, where he left his feet, where his body lands); he is up
    /// [`crate::live::DIVE_GETUP_S`] later.
    pub dives: Vec<(f64, u16, bb_math::Vec3, bb_math::Vec3)>,
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
    /// Every out on this pitch, in time order, as the umpire calls it.
    pub calls: Vec<crate::umpire::OutCall>,
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

    /// The same game with these fielders on both sides (labs; agent-models §10 grades).
    pub fn with_fielders(mut self, fattrs: [FielderAttrs; 9]) -> Game {
        self.fattrs = fattrs;
        self
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
            let fside = 1 - side as u16;
            let home = crate::play::home_positions();
            viz.standing.extend((0..9).map(|i| ((fside << 8) | 0x80 | (i as u16 + 1), home[i])));
            // Runners stand off their bases with their leads.
            viz.standing.extend(st.bases.iter().enumerate().filter_map(|(i, r)| r.map(|id| (id, crate::live::path_pos(crate::live::lead_s(i as u8 + 1))))));
            // The batter stands where his swing pivots (agent-models §3.6).
            let pivot = mu.template.pivot(&mu.batter);
            viz.standing.push((st.batter, bb_math::Vec3::new(pivot.x, pivot.y, 0.0)));
        }
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
        let mut calls = Vec::new();
        for f in &facts {
            let ev = step(st, f);
            // An out on the pitch (a strikeout): the umpire calls it with the pitch.
            for e in &ev {
                if let Event::RunnerOut { runner, base, .. } = e {
                    let pivot = mu.template.pivot(&mu.batter);
                    let t_out = fact_t(f).unwrap_or(t_plate);
                    calls.push(crate::umpire::OutCall { t_out, runner: *runner, base: *base, margin_s: 0.0, at: bb_math::Vec3::new(pivot.x, pivot.y, 0.0) });
                }
            }
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
            // What the fielders can see of the swing: the bat's horizontal angle at the instant
            // of contact, from the executed swing (world truth; each fielder reads it with his
            // own error inside the play, agent-models §5.10). Never the ball's launch.
            let bat_yaw = rec.swing.map(|s| {
                let a = bb_physics::contact::BatPose::at(&s, t_c).axis;
                bb_math::m::atan2(a.x, a.y)
            });
            // And the swing's lead: how long before the pitch reached the plate the bat started
            // forward (world truth, read with each fielder's error, agent-models §5.10).
            let swing_lead = rec.swing.map(|s| t_plate - s.t_start());
            let out = crate::live::simulate_live(
                &crate::live::LiveInput { env: &mu.env, field: &self.field, ball, t_contact: t_c, runners, fielders, fattrs: self.fattrs, batter_box: { let p = mu.template.pivot(&mu.batter); bb_math::Vec3::new(p.x, p.y, 0.0) }, bats: mu.batter.bats, pitch_x: rec.plate.map(|p| p.x).or(Some(ball.r.x)), bat_yaw, swing_lead, seed, play, record },
                st,
            );
            if record {
                // Every other tick, and always the last one (the instant the play ended).
                let n = out.ball_track.len();
                for (i, (t, p)) in out.ball_track.iter().enumerate() {
                    if i % 2 == 0 || i + 1 == n {
                        viz.batted.push([*t, p.x, p.y, p.z]);
                    }
                }
                viz.actors = out.actors.clone();
                viz.secured = out.secured.clone();
                viz.released = out.released.clone();
                viz.dives = out.dives.clone();
            }
            calls.extend(out.calls.iter().copied());
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
        Some(PitchStep { play, record: rec, pitch_flight, batted, rules_events, facts, calls, viz })
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

/// [`simulate_game`] with these fielders on both sides.
pub fn simulate_game_fielders(seed: u64, fattrs: [FielderAttrs; 9]) -> GameResult {
    let mut g = Game::new(seed, false).with_fielders(fattrs);
    while g.step_pitch().is_some() {}
    g.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bb_rules::OutKind;

    const BATTER: u16 = 2;
    const R3: u16 = 5;
    const SHORTSTOP: u16 = 0x0186;
    const CENTER: u16 = 0x0188;
    // Agent-models §3.4: no swing starts before the batter has decided on it. Seed 11 has a
    // swing (play 227) whose plan and execution noise put its start 7 ms before the commit;
    // it now starts at the commit.
    #[test]
    fn no_swing_starts_before_the_batter_decides() {
        let mut g = Game::new(11, false);
        let mut swings = 0;
        while let Some(step) = g.step_pitch() {
            if let Some(s) = &step.record.swing {
                swings += 1;
                assert!(s.t_start() >= step.record.t_decide, "play {}: swing starts at {:.4}, decision at {:.4}", step.play, s.t_start(), step.record.t_decide);
            }
        }
        assert!(swings > 100);
    }

    fn touch(t: f64, agent: u16, base: u8) -> Input {
        Input::BaseTouched { t, agent, base, holding_ball: false }
    }
    fn fielded(t: f64, fielder: u16) -> Input {
        Input::GloveSecured { t, fielder, x: 0.0, y: 40.0 }
    }
    fn out(runner: u16, base: u8, kind: OutKind) -> Event {
        Event::RunnerOut { runner, base, kind }
    }

    /// Scoring expectations come from OBR 9.05, not from the code: each case names the ruling
    /// an official scorer would make (no errors in the sim).
    #[test]
    fn provisional_hit_rulings_follow_obr_9_05() {
        // Clean single to center.
        assert_eq!(classify_bip(BATTER, &[], &[touch(4.3, BATTER, 1), fielded(4.6, CENTER)]), (true, true));
        // Single to center, batter thrown out stretching to second: a hit, out on the bases
        // (day review #4, seed 0 play 37).
        assert_eq!(classify_bip(BATTER, &[out(BATTER, 2, OutKind::Tag)], &[touch(5.01, BATTER, 1), fielded(5.04, CENTER)]), (true, true));
        // Single to right, runner thrown out at home, before or after the batter reaches
        // first: a hit either way (day review #4, seed 2 play 209).
        assert_eq!(classify_bip(BATTER, &[out(R3, 4, OutKind::Tag)], &[touch(5.05, BATTER, 1), fielded(5.58, CENTER)]), (true, true));
        assert_eq!(classify_bip(BATTER, &[out(R3, 4, OutKind::Tag)], &[fielded(3.0, CENTER), touch(5.05, BATTER, 1)]), (true, true));
        // Grounder to short, runner forced at second, batter beats the relay: fielder's choice.
        assert_eq!(classify_bip(BATTER, &[out(R3, 2, OutKind::Force)], &[fielded(1.2, SHORTSTOP), touch(4.4, BATTER, 1)]), (true, false));
        // Single to center but the runner from first is forced at second: no hit (9.05(b)(1)).
        assert_eq!(classify_bip(BATTER, &[out(R3, 2, OutKind::Force)], &[touch(4.4, BATTER, 1), fielded(4.5, CENTER)]), (true, false));
        // An infielder fields it and a runner is tagged out advancing, even after the batter
        // has reached first: fielder's choice (9.05(b)(3); day-fixes review #4, seed 6 play 112).
        assert_eq!(classify_bip(BATTER, &[out(R3, 3, OutKind::Tag)], &[fielded(0.96, SHORTSTOP), touch(5.01, BATTER, 1)]), (true, false));
        // Ground out at first; fly out (even if the batter crossed first before the catch);
        // batter tagged on the line.
        assert_eq!(classify_bip(BATTER, &[out(BATTER, 1, OutKind::Force)], &[fielded(1.2, SHORTSTOP)]), (false, false));
        assert_eq!(classify_bip(BATTER, &[out(BATTER, 0, OutKind::FlyOut)], &[touch(4.4, BATTER, 1), fielded(5.0, CENTER)]), (false, false));
        assert_eq!(classify_bip(BATTER, &[out(BATTER, 1, OutKind::Tag)], &[fielded(1.2, SHORTSTOP)]), (false, false));
        // Third out made on another runner before the batter got to first: he did not reach
        // (day-fixes review #4, seed 0 play 7).
        assert_eq!(classify_bip(BATTER, &[out(R3, 2, OutKind::Force)], &[fielded(1.2, SHORTSTOP)]), (false, false));
    }
}
