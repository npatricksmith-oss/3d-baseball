use bb_game::game::Game;
fn main() {
    for seed in [0, 1, 7, 42] {
        let mut game = Game::new(seed, false);
        while let Some(p) = game.step_pitch() {
            println!("{seed} {} {:?} {:?} {:?}", p.play, p.record, p.rules_events, p.facts);
        }
        let g = game.finish();
        println!("end {seed} {:?} {:?} {:?}", g.runs, g.linescore, g.digest);
    }
}
