//! Ground-ball diagnostics: the fact timeline of each grounder in a game.
use bb_rules::{Event, Input};
fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(7);
    let mut g = bb_game::game::Game::new(seed, false);
    let mut shown = 0;
    while let Some(step) = g.step_pitch() {
        let Some((ev, la, spray)) = step.record.batted else { continue };
        if la.to_degrees() >= 10.0 || shown >= 14 {
            continue;
        }
        let reached = !step.rules_events.iter().any(|e| matches!(e, Event::RunnerOut { .. }));
        let fair = step.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: true }));
        if !fair {
            continue;
        }
        shown += 1;
        println!("GB EV {:.0} mph LA {:.0} spray {:.0} -> {}", ev / 0.44704, la.to_degrees(), spray.to_degrees(), if reached { "SAFE" } else { "out" });
        if reached && la.to_degrees() > -30.0 {
            for f in &step.facts {
                match f {
                    Input::GloveSecured { t, fielder, x, y } => println!("    {t:.2} secured by {:x} at ({x:.1},{y:.1})", fielder),
                    Input::BaseTouched { t, agent, base, holding_ball } => println!("    {t:.2} {:x} touched {base}{}", agent, if *holding_ball { " WITH BALL" } else { "" }),
                    Input::BallTouched { t, what } => println!("    {t:.2} ball {:?}", what),
                    Input::TagContact { t, .. } => println!("    {t:.2} tag"),
                    _ => {}
                }
            }
        }
        let _ = Input::StartGame;
    }
}
