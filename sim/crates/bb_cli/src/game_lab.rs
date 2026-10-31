//! `sim-cli game` and `sim-cli games` (M4 walking skeleton).

use bb_game::game::{GameResult, simulate_game};
use clap::Args;

#[derive(Args)]
pub struct GameArgs {
    #[arg(long, default_value_t = 1)]
    seed: u64,
}

fn linescore(r: &GameResult) {
    let n = r.linescore[0].len().max(r.linescore[1].len());
    print!("{:6}", "");
    for i in 1..=n {
        print!("{i:>3}");
    }
    println!("  |  R");
    for (side, name) in [(0, "Away"), (1, "Home")] {
        print!("{name:6}");
        for i in 0..n {
            match r.linescore[side].get(i) {
                Some(x) => print!("{x:>3}"),
                None => print!("{:>3}", "x"),
            }
        }
        println!("  | {:2}", r.runs[side]);
    }
}

pub fn game(a: GameArgs) {
    let t0 = std::time::Instant::now();
    let r = simulate_game(a.seed);
    linescore(&r);
    println!(
        "{} PA, {} pitches; K {} BB {} HBP {} HR {}; balls in play {} (batter reached {}){}",
        r.pa, r.pitches, r.k, r.bb, r.hbp, r.hr, r.bip, r.reached_bip, if r.walk_off { "; walk-off" } else { "" }
    );
    if !r.warnings.is_empty() {
        println!("rules warnings: {:?}", r.warnings);
    }
    println!("time {:.0} ms", t0.elapsed().as_secs_f64() * 1e3);
}

#[derive(Args)]
pub struct GamesArgs {
    #[arg(long, default_value_t = 100)]
    n: u64,
    #[arg(long, default_value_t = 1)]
    seed: u64,
}

pub fn games(a: GamesArgs) {
    let t0 = std::time::Instant::now();
    let chunk = (a.n as usize).div_ceil(16);
    let seeds: Vec<u64> = (0..a.n).map(|i| a.seed * 1_000_003 + i).collect();
    let rs: Vec<GameResult> = std::thread::scope(|s| {
        let hs: Vec<_> = seeds.chunks(chunk.max(1)).map(|c| s.spawn(move || c.iter().map(|&sd| simulate_game(sd)).collect::<Vec<_>>())).collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    let n = rs.len() as f64;
    let sum = |f: &dyn Fn(&GameResult) -> f64| rs.iter().map(f).sum::<f64>();
    let pa = sum(&|r| r.pa as f64);
    println!("{} games, average players, interim M3 knobs, reference park 330/400/330", rs.len());
    println!("runs/team-game {:.2}", sum(&|r| (r.runs[0] + r.runs[1]) as f64) / (2.0 * n));
    println!("PA/team-game {:.1}, pitches/game {:.0}", pa / (2.0 * n), sum(&|r| r.pitches as f64) / n);
    println!(
        "K% {:.1}  BB% {:.1}  HBP% {:.1}  HR/PA {:.2}%  reached on BIP {:.1}% of BIP",
        100.0 * sum(&|r| r.k as f64) / pa,
        100.0 * sum(&|r| r.bb as f64) / pa,
        100.0 * sum(&|r| r.hbp as f64) / pa,
        100.0 * sum(&|r| r.hr as f64) / pa,
        100.0 * sum(&|r| r.reached_bip as f64) / sum(&|r| r.bip as f64)
    );
    println!("extra-inning games {}, walk-offs {}", rs.iter().filter(|r| r.innings > 9).count(), rs.iter().filter(|r| r.walk_off).count());
    let mut w: std::collections::BTreeMap<&str, usize> = Default::default();
    for r in &rs {
        for x in &r.warnings {
            *w.entry(x).or_default() += 1;
        }
    }
    println!("rules warnings: {w:?}");
    let el = t0.elapsed().as_secs_f64();
    println!("time {:.1} s wall on 16 threads ({:.0} ms per game per core)", el, el * 1e3 * 16.0 / n);
}
