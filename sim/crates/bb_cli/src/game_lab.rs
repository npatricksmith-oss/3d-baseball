//! `sim-cli game` and `sim-cli games` (M4 walking skeleton).

use bb_game::game::{GameResult, simulate_game_fielders};
use clap::Args;

#[derive(Args)]
pub struct GameArgs {
    #[arg(long, default_value_t = 1)]
    seed: u64,
    /// Write the game as a Rerun recording (scrub the `game_time` timeline).
    #[arg(long)]
    rrd: Option<std::path::PathBuf>,
    /// Print every pitch.
    #[arg(long)]
    pbp: bool,
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

/// Each side's batting lines, derived from the event log by `bb_stats`.
fn batting_lines(r: &GameResult) {
    let b = match bb_stats::box_score(&r.log) {
        Ok(b) => b,
        Err(e) => return println!("box score unavailable: {e:?}"),
    };
    for (side, team) in ["Away", "Home"].iter().enumerate() {
        println!("\n{:<36}{:>4}{:>4}{:>4}{:>4}{:>4}{:>4}{:>4}{:>5}{:>4}{:>4}{:>5}", format!("{team} batting (bats/throws)"), "PA", "AB", "R", "H", "2B", "3B", "HR", "RBI", "BB", "K", "HBP");
        for l in &b.batting[side] {
            let who = format!("{} {:<2} #{:<2} {}/{} {}", l.batting_order, l.position_code(), l.number, l.bats, l.throws, l.name);
            println!("{who:<36}{:>4}{:>4}{:>4}{:>4}{:>4}{:>4}{:>4}{:>5}{:>4}{:>4}{:>5}", l.pa, l.ab, l.runs, l.h, l.doubles, l.triples, l.hr, l.rbi, l.bb, l.k, l.hbp);
        }
        let t = &b.teams[side];
        println!("{:<36}{:>4}{:>4}{:>4}{:>4}{:>4}{:>4}{:>4}{:>5}{:>4}{:>4}{:>5}   LOB {}", "Totals", t.pa, t.ab, t.runs, t.h, t.doubles, t.triples, t.hr, t.rbi, t.bb, t.k, t.hbp, t.left_on_base);
        if t.undetermined > 0 { println!("{} batted-ball rulings lack evidence", t.undetermined); }
    }
    println!();
}

pub fn game(a: GameArgs) {
    let t0 = std::time::Instant::now();
    let r = bb_game::game::simulate_game_with(a.seed, a.rrd.is_some() || a.pbp);
    if a.pbp {
        for v in &r.viz {
            println!("{}", v.label);
        }
    }
    #[cfg(feature = "rerun")]
    if let Some(p) = &a.rrd {
        crate::rr::log_game(p, &r);
        println!("wrote {}", p.display());
    }
    linescore(&r);
    batting_lines(&r);
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
    /// Every fielder's `Anticipation` grade (20–80; default: the average fielder, 50).
    #[arg(long)]
    anticipation: Option<f64>,
    /// Every fielder's `ArmStrength`, `ArmAccuracy` and `Transfer` grades (20–80; default 50).
    #[arg(long)]
    arm_strength: Option<f64>,
    #[arg(long)]
    arm_accuracy: Option<f64>,
    #[arg(long)]
    transfer: Option<f64>,
    /// Fielders get no jump: they wait for their read of the ball (agent-models §5.10 off).
    #[arg(long)]
    no_jump: bool,
}

pub fn games(a: GamesArgs) {
    let t0 = std::time::Instant::now();
    let chunk = (a.n as usize).div_ceil(16);
    let seeds: Vec<u64> = (0..a.n).map(|i| a.seed * 1_000_003 + i).collect();
    let rs: Vec<GameResult> = std::thread::scope(|s| {
        let mut fattrs: [bb_game::play::FielderAttrs; 9] = core::array::from_fn(bb_game::play::FielderAttrs::average);
        for f in fattrs.iter_mut() {
            if let Some(g) = a.anticipation {
                *f = f.with_anticipation(g);
            }
            if a.arm_strength.is_some() || a.arm_accuracy.is_some() || a.transfer.is_some() {
                *f = f.with_throwing(a.arm_strength.unwrap_or(50.0), a.arm_accuracy.unwrap_or(50.0), a.transfer.unwrap_or(50.0));
            }
            if a.no_jump {
                f.jump_gain = 0.0;
            }
        }
        let hs: Vec<_> = seeds.chunks(chunk.max(1)).map(|c| s.spawn(move || c.iter().map(|&sd| simulate_game_fielders(sd, fattrs)).collect::<Vec<_>>())).collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    let n = rs.len() as f64;
    let sum = |f: &dyn Fn(&GameResult) -> f64| rs.iter().map(f).sum::<f64>();
    let pa = sum(&|r| r.pa as f64);
    println!("{} games, average players{}, interim M3 knobs, reference park 330/400/330", rs.len(), match (a.anticipation, a.no_jump) { (_, true) => " (no jumps)".to_string(), (Some(g), _) => format!(" (Anticipation {g})"), _ => String::new() });
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
    // By batted-ball type: hit rate, the batter reaching on a fielder's choice, and plays with
    // two or more outs. Hits and fielder's choices are the provisional lab ruling
    // `bb_game::game::classify_bip`, not the official scorer (no errors, no judgment calls).
    println!("by batted-ball type (hit / FC are a provisional ruling, not the scorer's):");
    let mut by = [(0u32, 0u32, 0u32, 0u32); 4];
    for r in &rs {
        for b in &r.bip_detail {
            let d = b.launch_rad.to_degrees();
            let k = if d < 10.0 { 0 } else if d < 25.0 { 1 } else if d < 50.0 { 2 } else { 3 };
            by[k].0 += 1;
            by[k].1 += b.hit as u32;
            by[k].2 += b.fielders_choice() as u32;
            by[k].3 += (b.outs >= 2) as u32;
        }
    }
    let names = ["GB (<10°)", "LD (10–25°)", "FB (25–50°)", "PU (>50°)"];
    let mlb = [0.24, 0.68, 0.13, 0.02];
    let total = by.iter().map(|x| x.0).sum::<u32>().max(1) as f64;
    for k in 0..4 {
        let n = by[k].0.max(1) as f64;
        println!(
            "  {:12} share {:4.1}%  hit {:5.3} (MLB BABIP ≈ {:.2})  FC {:5.3}  2+ outs {:5.3}",
            names[k],
            100.0 * by[k].0 as f64 / total,
            by[k].1 as f64 / n,
            mlb[k],
            by[k].2 as f64 / n,
            by[k].3 as f64 / n
        );
    }
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
