//! Prove that logging new facts did not change play: hash every pitch's record, rules events
//! and rules facts (everything except the event-log digest) for a range of seeds, so two
//! builds can be compared line by line.
//!   cargo run --release -p bb_game --example a12_gameplay_compare -- <first> <count>
//! With no arguments it covers seeds 0, 1, 7 and 42 and prints the play-by-play hash only.
use bb_game::game::Game;
use std::hash::{Hash, Hasher};

fn run(seed: u64) {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    let mut game = Game::new(seed, false);
    while let Some(p) = game.step_pitch() {
        format!("{} {:?} {:?} {:?}", p.play, p.record, p.rules_events, p.facts).hash(&mut h);
    }
    let g = game.finish();
    println!("seed {seed} stream {:016x} runs {:?} linescore {:?}", h.finish(), g.runs, g.linescore);
}

fn main() {
    let args: Vec<u64> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    match args[..] {
        [first, count] => (first..first + count).for_each(run),
        _ => [0, 1, 7, 42].into_iter().for_each(run),
    }
}
