//! Diagnostic: pop-ups (launch > 50°) that were not caught: seed, play, who touched it first.
use bb_rules::{Event, Input};
fn main() {
    let games: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(16);
    for seed in 0..games {
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            let Some((ev, la, spray)) = step.record.batted else { continue };
            if la.to_degrees() < 50.0 || !step.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: true })) {
                continue;
            }
            if step.rules_events.iter().any(|e| matches!(e, Event::Catch { .. })) {
                continue;
            }
            let first = step.facts.iter().find_map(|f| if let Input::GloveSecured { fielder, t, .. } = f { Some((fielder & 0x7F, *t)) } else { None });
            println!("seed {seed} play {} EV {:.0} mph LA {:.0} spray {:.0}: first secured {:?}", step.play, ev / 0.44704, la.to_degrees(), spray.to_degrees(), first);
        }
    }
}
