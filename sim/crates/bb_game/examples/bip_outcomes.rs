//! Diagnostic: one line per ball in play (seed play type safe), to diff two builds/modes.
use bb_rules::Event;
fn main() {
    let games: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(32);
    for seed in 0..games {
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            let Some((ev, la, spray)) = step.record.batted else { continue };
            if !step.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: true })) { continue; }
            let out = step.rules_events.iter().any(|e| matches!(e, Event::RunnerOut { .. }));
            println!("{seed} {} {:.0} {:.0} {:.0} {}", step.play, ev / 0.44704, la.to_degrees(), spray.to_degrees(), if out { "out" } else { "SAFE" });
        }
    }
}
