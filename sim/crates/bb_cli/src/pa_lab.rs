//! Plate-appearance lab (M3): B1 rates for a matchup.

use bb_game::pa::{B1Report, Matchup, simulate_pa};
use clap::Args;

#[derive(Args)]
pub struct PaArgs {
    #[arg(long, default_value_t = 2000)]
    n: u32,
    #[arg(long, default_value_t = 1)]
    seed: u64,
    /// Global knobs (PLAN §13.2).
    #[arg(long, default_value_t = 1.0)]
    k_perception: f64,
    #[arg(long, default_value_t = 1.0)]
    k_command: f64,
    #[arg(long, default_value_t = 1.0)]
    k_swing: f64,
}

pub fn pa(a: PaArgs) {
    let mut mu = Matchup::average();
    mu.knobs.perception = a.k_perception;
    mu.knobs.command = a.k_command;
    mu.knobs.swing = a.k_swing;
    let t0 = std::time::Instant::now();
    let mut rep = B1Report::default();
    for i in 0..a.n {
        rep.add(&simulate_pa(&mu, a.seed, i * bb_game::pa::PA_PLAY_STRIDE));
    }
    let el = t0.elapsed();
    println!("{} PA, average RHP vs average RHB, knobs {:?}", a.n, mu.knobs);
    rep.print();
    println!("time {:.2} s ({:.1} µs/pitch)", el.as_secs_f64(), el.as_secs_f64() * 1e6 / rep.pitches.max(1) as f64);
}
