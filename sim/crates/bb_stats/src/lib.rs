//! Statistics as pure functions of the event log (PLAN §4.3, data-formats §4). Nothing here
//! looks at the simulation; it reads only the verified log.

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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BoxScore {
    pub teams: [TeamLine; 2],
    pub linescore: [Vec<u32>; 2],
    /// Outs recorded per half-inning, in order (index = half-inning number).
    pub outs_by_half: Vec<u32>,
    pub final_score: Option<(u32, u32)>,
    pub innings: u32,
}

/// Build the box score from a finished log.
pub fn box_score(bytes: &[u8]) -> Result<BoxScore, LogError> {
    let v = log::read(bytes)?;
    let mut b = BoxScore::default();
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
                    let t = &mut b.teams[side];
                    t.pa += 1;
                    match e.body_as_plate_appearance_ended().unwrap().result() {
                        ev::PaResult::Strikeout => t.k += 1,
                        ev::PaResult::Walk => t.bb += 1,
                        ev::PaResult::HitByPitch => t.hbp += 1,
                        ev::PaResult::HomeRun => t.hr += 1,
                        _ => t.batted += 1,
                    }
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
