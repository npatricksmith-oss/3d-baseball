//! One line a viewer can read for each pitch: the call, and what happened if the plate
//! appearance ended. Built only from the pitch record, the rules events, and the facts of
//! the play. The hit / fielder's-choice wording uses the provisional ruling
//! [`crate::game::classify_bip`]; the official scorer will replace it.

use bb_rules::{AgentId, Event, Input, OutKind, PaResult};

use crate::pa::Call;
use crate::roster::Rosters;

const POSITION: [&str; 9] = ["the pitcher", "the catcher", "first", "second", "third", "short", "left", "center", "right"];

/// Where the ball was first fielded, as a phrase ("to short", "to left").
fn fielded_to(facts: &[Input]) -> Option<String> {
    let pos = facts.iter().find_map(|f| if let Input::GloveSecured { fielder, .. } = f { Some((fielder & 0x7F) as usize) } else { None })?;
    POSITION.get(pos.checked_sub(1)?).map(|p| format!("to {p}"))
}

fn runs_phrase(n: usize) -> String {
    match n {
        0 => String::new(),
        1 => ", 1 run scores".to_string(),
        n => format!(", {n} runs score"),
    }
}

/// Describe one pitch. `launch_deg` is the batted ball's launch angle, if it was hit. With
/// the game's rosters, a line that ends a plate appearance starts with the batter's family
/// name ("Ramirez: Single to left"; family names are unique in a game, data-formats §2.4).
pub fn describe(call: Call, launch_deg: Option<f64>, events: &[Event], facts: &[Input], rosters: Option<&Rosters>) -> String {
    let runs = events.iter().filter(|e| matches!(e, Event::RunScored { .. })).count();
    let outs: Vec<(AgentId, OutKind)> = events.iter().filter_map(|e| if let Event::RunnerOut { runner, kind, .. } = e { Some((*runner, *kind)) } else { None }).collect();
    let ended = events.iter().find_map(|e| if let Event::PlateAppearanceEnded { batter, result } = e { Some((*batter, *result)) } else { None });
    let Some((batter, result)) = ended else {
        // The plate appearance goes on (or a foul with two strikes).
        return match call {
            Call::Ball => "Ball",
            Call::CalledStrike => "Called strike",
            Call::SwingingStrike | Call::FoulTip => "Swinging strike",
            Call::Foul => "Foul",
            Call::InPlay => "In play",
            Call::HitByPitch => "Hit by pitch",
        }
        .to_string();
    };
    let line = match result {
        PaResult::Strikeout => if call == Call::CalledStrike { "Strikeout looking" } else { "Strikeout swinging" }.to_string(),
        PaResult::Walk => "Walk".to_string(),
        PaResult::HitByPitch => "Hit by pitch".to_string(),
        PaResult::HomeRun => "Home run".to_string(),
        PaResult::BattedBall => {
            let la = launch_deg.unwrap_or(0.0);
            let to = fielded_to(facts).map(|t| format!(" {t}")).unwrap_or_default();
            let batter_out = outs.iter().find(|(r, _)| *r == batter).map(|x| x.1);
            let (reached, hit) = crate::game::classify_bip(batter, events, facts);
            let multi = match outs.len() {
                2 => "Double play: ",
                3 => "Triple play: ",
                _ => "",
            };
            match batter_out {
                Some(OutKind::FlyOut) => {
                    let kind = if la > 50.0 { "Pop out" } else if la < 20.0 { "Line out" } else { "Fly out" };
                    format!("{multi}{}{to}", if multi.is_empty() { kind.to_string() } else { kind.to_lowercase() })
                }
                Some(_) if !reached => format!("{multi}{}{to}", if multi.is_empty() { "Ground out" } else { "ground out" }),
                _ if hit => {
                    // The furthest base the batter reached safely names the hit.
                    let bases = events.iter().filter_map(|e| if let Event::RunnerAdvanced { runner, to, .. } = e { (*runner == batter).then_some(*to) } else { None }).max().unwrap_or(1);
                    let name = match bases {
                        2 => "Double",
                        3 => "Triple",
                        4 => "Inside-the-park home run",
                        _ => "Single",
                    };
                    let extra = if batter_out.is_some() { ", batter out stretching" } else if outs.len() == 1 { ", runner out on the bases" } else { "" };
                    format!("{name}{to}{extra}")
                }
                _ if reached => format!("{multi}Fielder's choice{to}"),
                // The batter was not put out but never reached: the inning ended on another
                // runner, forced or tagged.
                _ if outs.iter().any(|(r, k)| *r != batter && *k == OutKind::Force) => format!("{multi}Force out{to}"),
                _ => format!("{multi}Fielder's choice{to}, runner tagged out"),
            }
        }
    };
    let who = rosters.and_then(|r| r.player(batter)).map(|p| format!("{}: ", p.family)).unwrap_or_default();
    format!("{who}{line}{}", runs_phrase(runs))
}

#[cfg(test)]
mod tests {
    use super::*;

    const B: AgentId = 3;
    const R: AgentId = 5;
    fn ended(result: PaResult) -> Event {
        Event::PlateAppearanceEnded { batter: B, result }
    }
    fn out(runner: AgentId, kind: OutKind) -> Event {
        Event::RunnerOut { runner, base: 1, kind }
    }
    fn by(pos: u16) -> Input {
        Input::GloveSecured { t: 1.0, fielder: 0x0180 | pos, x: 0.0, y: 30.0 }
    }
    fn first(t: f64) -> Input {
        Input::BaseTouched { t, agent: B, base: 1, holding_ball: false }
    }

    // The expected strings are how a radio announcer would call each play.
    #[test]
    fn plays_read_the_way_they_are_called() {
        assert_eq!(describe(Call::Ball, None, &[], &[], None), "Ball");
        assert_eq!(describe(Call::Foul, Some(40.0), &[], &[], None), "Foul");
        assert_eq!(describe(Call::CalledStrike, None, &[out(B, OutKind::Strikeout), ended(PaResult::Strikeout)], &[], None), "Strikeout looking");
        assert_eq!(describe(Call::SwingingStrike, None, &[out(B, OutKind::Strikeout), ended(PaResult::Strikeout)], &[], None), "Strikeout swinging");
        assert_eq!(describe(Call::Ball, None, &[ended(PaResult::Walk)], &[], None), "Walk");
        assert_eq!(describe(Call::InPlay, Some(-5.0), &[out(B, OutKind::Force), ended(PaResult::BattedBall)], &[by(6)], None), "Ground out to short");
        assert_eq!(describe(Call::InPlay, Some(32.0), &[out(B, OutKind::FlyOut), ended(PaResult::BattedBall)], &[by(8)], None), "Fly out to center");
        assert_eq!(describe(Call::InPlay, Some(65.0), &[out(B, OutKind::FlyOut), ended(PaResult::BattedBall)], &[by(4)], None), "Pop out to second");
        assert_eq!(describe(Call::InPlay, Some(12.0), &[out(B, OutKind::FlyOut), ended(PaResult::BattedBall)], &[by(5)], None), "Line out to third");
        // A sacrifice fly.
        assert_eq!(describe(Call::InPlay, Some(30.0), &[out(B, OutKind::FlyOut), Event::RunScored { runner: R }, ended(PaResult::BattedBall)], &[by(9)], None), "Fly out to right, 1 run scores");
        let adv = |to| Event::RunnerAdvanced { runner: B, from: 0, to, cause: bb_rules::AdvanceCause::OnPlay };
        assert_eq!(describe(Call::InPlay, Some(15.0), &[adv(1), ended(PaResult::BattedBall)], &[first(4.2), by(7)], None), "Single to left");
        assert_eq!(describe(Call::InPlay, Some(18.0), &[adv(2), Event::RunScored { runner: R }, Event::RunScored { runner: 6 }, ended(PaResult::BattedBall)], &[first(4.2), by(8)], None), "Double to center, 2 runs score");
        assert_eq!(describe(Call::InPlay, Some(-3.0), &[out(R, OutKind::Force), out(B, OutKind::Force), ended(PaResult::BattedBall)], &[by(6)], None), "Double play: ground out to short");
        assert_eq!(describe(Call::InPlay, Some(-3.0), &[out(R, OutKind::Force), adv(1), ended(PaResult::BattedBall)], &[by(6), first(4.3)], None), "Fielder's choice to short");
        // The third out made on another runner before the batter reached first: a force at
        // second, or (seed 7475 play 26) a tag on the runner from third at home.
        assert_eq!(describe(Call::InPlay, Some(-50.0), &[out(R, OutKind::Force), ended(PaResult::BattedBall)], &[by(1)], None), "Force out to the pitcher");
        assert_eq!(describe(Call::InPlay, Some(-54.0), &[out(R, OutKind::Tag), ended(PaResult::BattedBall)], &[by(1)], None), "Fielder's choice to the pitcher, runner tagged out");
        assert_eq!(describe(Call::InPlay, Some(28.0), &[ended(PaResult::HomeRun), Event::RunScored { runner: B }], &[], None), "Home run, 1 run scores");
    }
}
