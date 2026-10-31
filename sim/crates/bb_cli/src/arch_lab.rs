//! `sim-cli archetypes`: B1 profile per batter archetype vs an average pitcher, and per
//! pitcher archetype vs an average batter.

use bb_agents::attributes::{BatterGrades, PitcherGrades, archetypes};
use bb_game::pa::{B1Report, Matchup, run_pas};
use clap::Args;

#[derive(Args)]
pub struct ArchArgs {
    #[arg(long, default_value_t = 1500)]
    n: u32,
    #[arg(long, default_value_t = 5)]
    seed: u64,
}

fn row(name: &str, r: &B1Report) {
    let pct = |a: u32, b: u32| 100.0 * a as f64 / b.max(1) as f64;
    let ev = r.bbe_ev.iter().sum::<f64>() / r.bbe_ev.len().max(1) as f64;
    let hard = r.bbe_ev.iter().filter(|&&e| e >= 95.0).count() as u32;
    println!(
        "{name:16} {:5.1} {:5.1} {:7.1} {:7.1} {:6.1} {:6.1} {:6.1}",
        pct(r.k, r.pa),
        pct(r.bb, r.pa),
        pct(r.o_swings, r.pitches - r.zone),
        pct(r.contacts, r.swings),
        ev,
        pct(hard, r.bbe_ev.len() as u32),
        r.pitches as f64 / r.pa.max(1) as f64
    );
}

pub fn run(a: ArchArgs) {
    let (n, seed) = (a.n, a.seed);
    let header = || println!("{:16} {:>5} {:>5} {:>7} {:>7} {:>6} {:>6} {:>6}", "", "K%", "BB%", "O-Sw%", "Cont%", "EV", "Hard%", "P/PA");
    println!("Batter archetypes vs an average RHP ({n} PA each)");
    header();
    let bs = archetypes::batters();
    let res: Vec<B1Report> = std::thread::scope(|s| {
        let hs: Vec<_> = bs.iter().map(|(_, g)| s.spawn(move || run_pas(&Matchup::with_grades(g, &PitcherGrades::default()), n, seed))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for ((name, _), r) in bs.iter().zip(&res) {
        row(name, r);
    }
    println!("\nPitcher archetypes vs an average RHB ({n} PA each)");
    header();
    let ps = archetypes::pitchers();
    let res: Vec<B1Report> = std::thread::scope(|s| {
        let hs: Vec<_> = ps.iter().map(|(_, g)| s.spawn(move || run_pas(&Matchup::with_grades(&BatterGrades::default(), g), n, seed))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for ((name, _), r) in ps.iter().zip(&res) {
        row(name, r);
    }
}
