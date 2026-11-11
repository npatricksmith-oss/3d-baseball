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
    // Out-of-zone pitches by how far the ball's edge missed the zone: (pitches, swings) for
    // Statcast's attack zones (shadow to one ball width, chase to three, then waste).
    let edges_cm = [8.5, 25.5, f64::INFINITY];
    let mut miss = [(0u32, 0u32); 3];
    let r = mu.env.ball.radius;
    for i in 0..a.n {
        let pa = simulate_pa(&mu, a.seed, i * bb_game::pa::PA_PLAY_STRIDE);
        for p in pa.pitches.iter().filter(|p| !p.in_zone) {
            let Some(at) = p.plate else { continue };
            let dx = (at.x.abs() - mu.zone.half_width).max(0.0);
            let dz = (mu.zone.bottom - at.z).max(at.z - mu.zone.top).max(0.0);
            let out_cm = 100.0 * (dx.hypot(dz) - r);
            let k = edges_cm.iter().position(|&e| out_cm < e).unwrap();
            miss[k].0 += 1;
            miss[k].1 += p.swung as u32;
        }
        rep.add(&pa);
    }
    let el = t0.elapsed();
    println!("{} PA, average RHP vs average RHB, knobs {:?}", a.n, mu.knobs);
    rep.print();
    // MLB reference: Statcast attack zones, league shares and swing rates, approximate.
    let total: u32 = miss.iter().map(|x| x.0).sum();
    println!("out-of-zone pitches by miss distance: share of them, swing rate (MLB ≈)");
    for (k, (name, share, swing)) in [("shadow  0–8.5 cm", 40.0, 43.0), ("chase 8.5–25.5 cm", 41.0, 23.5), ("waste   > 25.5 cm", 19.0, 5.5)].iter().enumerate() {
        println!("  {name}  share {:4.1}% ({share:.0})  swing {:4.1}% ({swing})", 100.0 * miss[k].0 as f64 / total.max(1) as f64, 100.0 * miss[k].1 as f64 / miss[k].0.max(1) as f64);
    }
    println!("time {:.2} s ({:.1} µs/pitch)", el.as_secs_f64(), el.as_secs_f64() * 1e6 / rep.pitches.max(1) as f64);
}
