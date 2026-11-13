//! Every out call over a few games: when the out was made, when the umpire signals it, and how
//! close the play was. `cargo run --release -p bb_game --example umpire_calls -- [games]`
use bb_game::game::Game;
fn main() {
    let n: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(2);
    let mut hist = [0u32; 8];
    for seed in 0..n {
        let mut g = Game::new(seed, false);
        while let Some(s) = g.step_pitch() {
            for c in &s.calls {
                let d = c.t_call() - c.t_out;
                hist[((c.margin_s.min(3.5) / 0.5) as usize).min(7)] += 1;
                if seed == 0 {
                    println!("play {:4} runner {:#06x} base {} t_out {:6.2} call {:6.2} (+{:.2}) margin {:.2}", s.play, c.runner, c.base, c.t_out, c.t_call(), d, c.margin_s);
                }
            }
        }
    }
    println!("margin histogram (0.5 s bins, last = 3.5+ incl. routine catches): {hist:?}");
}
