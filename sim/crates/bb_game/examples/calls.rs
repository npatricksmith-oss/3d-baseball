//! The play-by-play of one game, one line per pitch as a viewer would read it.
//! Usage: `cargo run --release -p bb_game --example calls -- [seed]`
fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(7);
    let mut g = bb_game::game::Game::new(seed, false);
    while let Some(step) = g.step_pitch() {
        let la = step.record.batted.map(|b| b.1.to_degrees());
        let text = bb_game::describe::describe(step.record.call, la, &step.rules_events, &step.facts, Some(&g.rosters));
        let s = g.st.score();
        println!("{:3}  {}{:<2} {}-{}  {}", step.play, if g.st.half == bb_rules::Half::Top { "T" } else { "B" }, g.st.inning, s[0], s[1], text);
    }
}
