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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BoxScore {
    pub scorer_version: u16,
    pub teams: [TeamLine; 2],
    /// Each side's batters in batting order (away, home); empty if the log has no rosters.
    pub batting: [Vec<BatterLine>; 2],
    pub linescore: [Vec<u32>; 2],
    /// Outs recorded per half-inning, in order (index = half-inning number).
    pub outs_by_half: Vec<u32>,
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
                    b.outs_by_half.push(0);
                    b.innings = b.innings.max(h.inning() as u32);
                    outs_in_half = 0;
                    runners.clear();
                    half_ended = false;
                    stranded_batter = false;
                }
                EventBody::PitchReleased => b.teams[side].pitches_seen += 1,
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
                    if (1..=3).contains(&a.to_base()) { runners.insert(a.runner()); } else { runners.remove(&a.runner()); }
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
                    let pending_walk_off = ruling.outcome == scorer::Outcome::Undetermined
                        && events.iter().filter_map(|e| e.body_as_game_ended()).any(|g| g.reason() == ev::GameEndReason::WalkOff);
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
                            match base {
                                1 => { t.singles += 1; l.singles += 1; }
                                2 => { t.doubles += 1; l.doubles += 1; }
                                3 => { t.triples += 1; l.triples += 1; }
                                4 => { t.hr += 1; l.hr += 1; }
                                _ => {}
                            }
                            if base != 4 { t.batted += 1; l.batted += 1; }
                        }
                        Outcome::Out | Outcome::FieldersChoice | Outcome::Undetermined => {
                            if !ruling.sacrifice_fly { t.ab += 1; l.ab += 1; }
                            t.batted += 1; l.batted += 1;
                            if ruling.outcome == Outcome::Undetermined { t.undetermined += 1; l.undetermined += 1; }
                        }
                    }
                }
                EventBody::HalfInningEnded => {
                    b.teams[side].left_on_base += runners.len() as u32 + u32::from(stranded_batter);
                    half_ended = true;
                }
                EventBody::GameEnded => {
                    let g = e.body_as_game_ended().unwrap();
                    if !half_ended { b.teams[side].left_on_base += runners.len() as u32; half_ended = true; }
                    b.final_score = Some((g.score_away() as u32, g.score_home() as u32));
                }
                _ => {}
            }
        }
    }
    Ok(b)
}
