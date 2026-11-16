//! Statistics as pure functions of the event log (PLAN §4.3, data-formats §4). Nothing here
//! looks at the simulation; it reads only the verified log.

pub mod scorer;

use bb_schema::ev::{self, EventBody};
use bb_schema::log::{self, LogError};

/// Per-team batting line derived from the log.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TeamLine {
    pub runs: u32,
    pub pa: u32,
    pub ab: u32,
    pub h: u32,
    pub singles: u32,
    pub doubles: u32,
    pub triples: u32,
    pub rbi: u32,
    pub sf: u32,
    pub sh: u32,
    pub undetermined: u32,
    /// At-bats ended by a base awarded on a thrown ball out of play: no hit, and not yet
    /// ruled an error (rules-spec §10.2).
    pub thrown_award: u32,
    pub k: u32,
    pub bb: u32,
    pub hbp: u32,
    pub hr: u32,
    /// Plate appearances ending on a batted ball (to be split by the official scorer).
    pub batted: u32,
    pub pitches_seen: u32,
    pub left_on_base: u32,
}

/// One player's batting line, with who he is from the log header's roster (data-formats
/// §2.4). Batting statistics are derived by the log-backed scorer (rules-spec §10).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BatterLine {
    /// The id he bats and runs under.
    pub agent: u16,
    /// Full name, given name first.
    pub name: String,
    pub family: String,
    pub number: u8,
    /// Fielding position, scorer's numbering 1–9.
    pub position: u8,
    /// Place in the batting order, 1–9.
    pub batting_order: u8,
    /// The side he bats from and the hand he throws with: 'L', 'R', or 'S' for a switch
    /// hitter ('?' if the log gave none).
    pub bats: char,
    pub throws: char,
    pub pa: u32,
    pub runs: u32,
    pub ab: u32,
    pub h: u32,
    pub singles: u32,
    pub doubles: u32,
    pub triples: u32,
    pub rbi: u32,
    pub sf: u32,
    pub sh: u32,
    pub undetermined: u32,
    /// At-bats ended by a base awarded on a thrown ball out of play: no hit, and not yet
    /// ruled an error (rules-spec §10.2).
    pub thrown_award: u32,
    pub k: u32,
    pub bb: u32,
    pub hbp: u32,
    pub hr: u32,
    /// Plate appearances ending on a batted ball (to be split by the official scorer).
    pub batted: u32,
}

impl BatterLine {
    /// The scorer's abbreviation of his position ("SS"), or "" if the log gave none.
    pub fn position_code(&self) -> &'static str {
        ["", "P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF", "DH", "PH", "PR"].get(self.position as usize).copied().unwrap_or("")
    }
}

/// One pitcher's line: who he is (from the roster in the log's header) and the pitches he
/// has thrown, counted from `PitchReleased` events.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PitcherLine {
    /// The id he fields under (the event's `pitcher`).
    pub agent: u16,
    /// Full name, given name first; empty if the log's roster has no pitcher for his side.
    pub name: String,
    pub family: String,
    pub number: u8,
    pub pitches: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BoxScore {
    pub scorer_version: u16,
    pub teams: [TeamLine; 2],
    /// Each side's batters in batting order (away, home); empty if the log has no rosters.
    pub batting: [Vec<BatterLine>; 2],
    /// Each side's pitchers in the order they first pitched (away, home).
    pub pitching: [Vec<PitcherLine>; 2],
    pub linescore: [Vec<u32>; 2],
    /// Hits by inning, parallel to `linescore` (one entry per half-inning the side batted).
    pub hits_by_inning: [Vec<u32>; 2],
    /// Outs recorded per half-inning, in order (index = half-inning number).
    pub outs_by_half: Vec<u32>,
    /// Runners left on base per half-inning as the scorer counts them from the events (OBR
    /// 9.02(g)), parallel to `outs_by_half`.
    pub lob_by_half: Vec<u32>,
    /// What the engine claimed in `HalfInningEnded.left_on_base` for each half (`None` where the
    /// game ended first). Readers do not use it; tests cross-check it against `lob_by_half`.
    pub engine_lob_by_half: Vec<Option<u32>>,
    pub final_score: Option<(u32, u32)>,
    pub innings: u32,
}

/// Build the box score from a finished log.
pub fn box_score(bytes: &[u8]) -> Result<BoxScore, LogError> {
    box_score_with_options(bytes, scorer::ScorerOptions::default())
}

/// Score a log with an explicitly chosen counterfactual threshold.
pub fn box_score_with_options(bytes: &[u8], options: scorer::ScorerOptions) -> Result<BoxScore, LogError> {
    let hand = |h: bb_schema::Hand| match h {
        bb_schema::Hand::Left => 'L',
        bb_schema::Hand::Right => 'R',
        bb_schema::Hand::Switch => 'S',
        _ => '?',
    };
    let v = log::read(bytes)?;
    let mut b = BoxScore { scorer_version: scorer::SCORER_VERSION, ..Default::default() };
    if let Some(cfg) = v.header.config() {
        for (side, team) in [cfg.away(), cfg.home()].into_iter().enumerate() {
            let Some(roster) = team.and_then(|t| t.roster()) else { continue };
            b.batting[side] = roster
                .iter()
                .filter(|e| e.batting_order() > 0)
                .map(|e| BatterLine {
                    agent: e.agent_id(),
                    name: e.name().unwrap_or_default().to_string(),
                    family: e.family_name().unwrap_or_default().to_string(),
                    number: e.uniform_number(),
                    position: e.position().0,
                    batting_order: e.batting_order(),
                    bats: hand(e.bats()),
                    throws: hand(e.throws()),
                    ..Default::default()
                })
                .collect();
            b.batting[side].sort_by_key(|l| l.batting_order);
        }
    }
    let mut side = 0usize;
    let mut outs_in_half = 0u32;
    let mut runners = std::collections::BTreeSet::new();
    let mut half_ended = true;
    let mut stranded_batter = false;
    for play in &v.plays {
        let Some(events) = play.events() else { continue };
        for e in events.iter() {
            match e.body_type() {
                EventBody::HalfInningStarted => {
                    let h = e.body_as_half_inning_started().unwrap();
                    side = if h.half() == ev::HalfInning::Top { 0 } else { 1 };
                    b.linescore[side].push(0);
                    b.hits_by_inning[side].push(0);
                    b.outs_by_half.push(0);
                    b.lob_by_half.push(0);
                    b.engine_lob_by_half.push(None);
                    b.innings = b.innings.max(h.inning() as u32);
                    outs_in_half = 0;
                    runners.clear();
                    half_ended = false;
                    stranded_batter = false;
                }
                EventBody::PitchReleased => {
                    b.teams[side].pitches_seen += 1;
                    // The fielding side's pitcher. Who he is comes from the roster: the man
                    // listed at pitcher on his side (there are no substitutions yet).
                    let pitcher = e.body_as_pitch_released().unwrap().pitcher();
                    let fielding = 1 - side;
                    let lines = &mut b.pitching[fielding];
                    let at = match lines.iter().position(|l| l.agent == pitcher) {
                        Some(i) => i,
                        None => {
                            let who = b.batting[fielding].iter().find(|l| l.position == 1);
                            lines.push(PitcherLine {
                                agent: pitcher,
                                name: who.map(|l| l.name.clone()).unwrap_or_default(),
                                family: who.map(|l| l.family.clone()).unwrap_or_default(),
                                number: who.map_or(0, |l| l.number),
                                pitches: 0,
                            });
                            lines.len() - 1
                        }
                    };
                    lines[at].pitches += 1;
                }
                EventBody::RunScored => {
                    let runner = e.body_as_run_scored().unwrap().runner();
                    runners.remove(&runner);
                    if let Some(l) = b.batting[side].iter_mut().find(|l| l.agent == runner) {
                        l.runs += 1;
                    }
                    b.teams[side].runs += 1;
                    if let Some(x) = b.linescore[side].last_mut() {
                        *x += 1;
                    }
                }
                EventBody::RunnerAdvanced => {
                    let a = e.body_as_runner_advanced().unwrap();
                    // Only `RunScored` or `RunnerOut` ends a runner's trip: an advance to home is not a run
                    // (a walk-off's later pending touches never complete, OBR 5.08(b), 9.02(g)).
                    if (1..=3).contains(&a.to_base()) { runners.insert(a.runner()); }
                }
                EventBody::RunnerOut => {
                    outs_in_half += 1;
                    runners.remove(&e.body_as_runner_out().unwrap().runner());
                    if let Some(x) = b.outs_by_half.last_mut() {
                        *x += 1;
                    }
                }
                EventBody::PlateAppearanceEnded => {
                    let pa = e.body_as_plate_appearance_ended().unwrap();
                    let mut blank = BatterLine::default();
                    let l = b.batting[side].iter_mut().find(|l| l.agent == pa.batter()).unwrap_or(&mut blank);
                    let t = &mut b.teams[side];
                    t.pa += 1;
                    l.pa += 1;
                    let play_outs = events.iter().filter(|e| e.body_type() == EventBody::RunnerOut).count() as u32;
                    let ruling = scorer::score_play_with_options(*play, outs_in_half.saturating_sub(play_outs), options).expect("PA event exists");
                    // A walk-off that ends the game before the batter touches first: he is on base
                    // when it ends (the scorer rules his hit from where he was running).
                    let pending_walk_off = pa.result() == ev::PaResult::Unknown
                        && events.iter().filter_map(|e| e.body_as_game_ended()).any(|g| g.reason() == ev::GameEndReason::WalkOff)
                        && events.iter().filter_map(|e| e.body_as_scorer_evidence()).any(|sc| sc.batter() == pa.batter() && !sc.has_first_touch());
                    if (pending_walk_off || events.iter().filter_map(|e| e.body_as_scorer_evidence()).any(|sc| sc.batter() == pa.batter() && sc.has_first_touch()))
                        && !events.iter().filter_map(|e| e.body_as_runner_out()).any(|o| o.runner() == pa.batter())
                        && !events.iter().filter_map(|e| e.body_as_run_scored()).any(|r| r.runner() == pa.batter()) {
                        runners.insert(pa.batter());
                    }
                    // OBR box-score LOB includes a batter still short of first when a
                    // preceding runner makes the third out (9.02(g)), even though the
                    // rules engine's occupied-base count does not.
                    stranded_batter = outs_in_half >= 3 && ruling.outcome == scorer::Outcome::Out
                        && pa.result() == ev::PaResult::Unknown
                        && !events.iter().filter_map(|e| e.body_as_runner_out()).any(|o| o.runner() == pa.batter());
                    if ruling.sacrifice_fly { t.sf += 1; l.sf += 1; }
                    t.rbi += ruling.rbi;
                    l.rbi += ruling.rbi;
                    use scorer::Outcome;
                    match ruling.outcome {
                        Outcome::Walk => { t.bb += 1; l.bb += 1; }
                        Outcome::HitByPitch => { t.hbp += 1; l.hbp += 1; }
                        Outcome::Strikeout => { t.k += 1; l.k += 1; t.ab += 1; l.ab += 1; }
                        Outcome::Hit(base) => {
                            t.ab += 1; l.ab += 1; t.h += 1; l.h += 1;
                            if let Some(x) = b.hits_by_inning[side].last_mut() { *x += 1; }
                            match base {
                                1 => { t.singles += 1; l.singles += 1; }
                                2 => { t.doubles += 1; l.doubles += 1; }
                                3 => { t.triples += 1; l.triples += 1; }
                                4 => { t.hr += 1; l.hr += 1; }
                                _ => {}
                            }
                            if base != 4 { t.batted += 1; l.batted += 1; }
                        }
                        Outcome::Out | Outcome::FieldersChoice | Outcome::Undetermined | Outcome::ReachedOnThrowAward => {
                            if !ruling.sacrifice_fly { t.ab += 1; l.ab += 1; }
                            t.batted += 1; l.batted += 1;
                            if ruling.outcome == Outcome::ReachedOnThrowAward { t.thrown_award += 1; l.thrown_award += 1; }
                            if ruling.outcome == Outcome::Undetermined { t.undetermined += 1; l.undetermined += 1; }
                        }
                    }
                }
                EventBody::HalfInningEnded => {
                    let lob = runners.len() as u32 + u32::from(stranded_batter);
                    b.teams[side].left_on_base += lob;
                    if let Some(x) = b.lob_by_half.last_mut() { *x = lob; }
                    if let Some(x) = b.engine_lob_by_half.last_mut() { *x = Some(e.body_as_half_inning_ended().unwrap().left_on_base() as u32); }
                    half_ended = true;
                }
                EventBody::GameEnded => {
                    let g = e.body_as_game_ended().unwrap();
                    if !half_ended {
                        b.teams[side].left_on_base += runners.len() as u32;
                        if let Some(x) = b.lob_by_half.last_mut() { *x = runners.len() as u32; }
                        half_ended = true;
                    }
                    b.final_score = Some((g.score_away() as u32, g.score_home() as u32));
                }
                _ => {}
            }
        }
    }
    Ok(b)
}
