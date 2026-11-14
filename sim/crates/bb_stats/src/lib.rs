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
/// §2.4). At-bats and hits wait for the official scorer (rules-spec §10), as in [`TeamLine`].
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
    let hand = |h: bb_schema::Hand| match h {
        bb_schema::Hand::Left => 'L',
        bb_schema::Hand::Right => 'R',
        bb_schema::Hand::Switch => 'S',
        _ => '?',
    };
    let v = log::read(bytes)?;
    let mut b = BoxScore::default();
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
                }
                EventBody::PitchReleased => b.teams[side].pitches_seen += 1,
                EventBody::RunScored => {
                    let runner = e.body_as_run_scored().unwrap().runner();
                    if let Some(l) = b.batting[side].iter_mut().find(|l| l.agent == runner) {
                        l.runs += 1;
                    }
                    b.teams[side].runs += 1;
                    if let Some(x) = b.linescore[side].last_mut() {
                        *x += 1;
                    }
                }
                EventBody::RunnerOut => {
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
                    let (tc, lc) = match pa.result() {
                        ev::PaResult::Strikeout => (&mut t.k, &mut l.k),
                        ev::PaResult::Walk => (&mut t.bb, &mut l.bb),
                        ev::PaResult::HitByPitch => (&mut t.hbp, &mut l.hbp),
                        ev::PaResult::HomeRun => (&mut t.hr, &mut l.hr),
                        _ => (&mut t.batted, &mut l.batted),
                    };
                    *tc += 1;
                    *lc += 1;
                }
                EventBody::HalfInningEnded => {
                    b.teams[side].left_on_base += e.body_as_half_inning_ended().unwrap().left_on_base() as u32;
                }
                EventBody::GameEnded => {
                    let g = e.body_as_game_ended().unwrap();
                    b.final_score = Some((g.score_away() as u32, g.score_home() as u32));
                }
                _ => {}
            }
        }
    }
    Ok(b)
}
