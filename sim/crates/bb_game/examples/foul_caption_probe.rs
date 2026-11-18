//! Diagnostic: a hit pitch's call, rules events and facts, to see when a foul is decided.
//! `-- <seed> <play> [<play> ...]`
fn main() {
    let a: Vec<u64> = std::env::args().skip(1).map(|s| s.parse().unwrap()).collect();
    let seed = a[0];
    let plays: Vec<u32> = a[1..].iter().map(|&p| p as u32).collect();
    let mut g = bb_game::game::Game::new(seed, true);
    while let Some(step) = g.step_pitch() {
        if !plays.contains(&step.play) {
            continue;
        }
        println!("== play {} call {:?} batted {:?}", step.play, step.record.call, step.record.batted.map(|b| (b.0, b.1.to_degrees(), b.2.to_degrees())));
        for e in &step.rules_events {
            println!("  ev {e:?}");
        }
        for f in &step.facts {
            println!("  fact {f:?}");
        }
        if step.play >= *plays.iter().max().unwrap() {
            break;
        }
    }
}
