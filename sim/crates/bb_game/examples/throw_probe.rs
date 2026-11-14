//! Diagnostic: every throw in a run of games (owner backlog A3). How hurried each was, who
//! was off balance, and what came of it: taken in the air by another fielder, or not (it
//! bounced or got away).
//! `-- [games] [first seed] [strength accuracy transfer]`: the three grades (20–80) give
//! every fielder those throwing ratings.
use std::collections::BTreeMap;

const POS: [&str; 10] = ["?", "P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"];

#[derive(Default, Clone, Copy)]
struct Tally {
    n: u64,
    clean: u64,
    hurry_sum: f64,
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let games: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(100);
    let first: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let grades: Option<[f64; 3]> = (a.len() >= 5).then(|| core::array::from_fn(|i| a[2 + i].parse().expect("a grade")));
    // By how hurried: set (0), then thirds of the way to the earliest release.
    let mut by_hurry = [Tally::default(); 4];
    let mut by_balance = [Tally::default(); 2];
    let mut by_pos: BTreeMap<u16, Tally> = BTreeMap::new();
    let (mut bip, mut outs_on_bip) = (0u64, 0u64);
    for seed in first..first + games {
        let mut g = bb_game::game::Game::new(seed, true);
        if let Some([s, ac, t]) = grades {
            g = g.with_fielders(core::array::from_fn(|i| bb_game::play::FielderAttrs::average(i).with_throwing(s, ac, t)));
        }
        while let Some(step) = g.step_pitch() {
            if step.batted.is_none() {
                continue;
            }
            bip += 1;
            outs_on_bip += step.calls.len() as u64;
            for (k, &(t, id)) in step.viz.released.iter().enumerate() {
                let (hurry, off) = step.viz.hurry[k];
                // The next man to have the ball, and whether it touched the ground on the way.
                let next = step.viz.secured.iter().find(|s| s.0 > t);
                let low = |until: f64| step.viz.batted.iter().any(|s| s[0] > t && s[0] < until && s[3] < 0.1);
                let clean = next.is_some_and(|&(ts, who)| who != id && !low(ts));
                let bin = if hurry <= 0.0 { 0 } else { 1 + ((hurry * 3.0) as usize).min(2) };
                for tl in [&mut by_hurry[bin], &mut by_balance[off as usize], by_pos.entry(id & 0x7F).or_default()] {
                    tl.n += 1;
                    tl.clean += clean as u64;
                    tl.hurry_sum += hurry;
                }
            }
        }
    }
    let g = games as f64;
    let line = |name: &str, t: &Tally| println!("  {name:<22} {:>6.2} a game   taken in the air {:>5.1}%   mean hurry {:.2}", t.n as f64 / g, 100.0 * t.clean as f64 / t.n.max(1) as f64, t.hurry_sum / t.n.max(1) as f64);
    println!("{games} games from seed {first}: {:.1} balls in play a game, {:.3} outs per ball in play", bip as f64 / g, outs_on_bip as f64 / bip as f64);
    println!("throws by how hurried:");
    for (name, t) in ["set", "hurry up to 1/3", "hurry 1/3 to 2/3", "hurry over 2/3"].iter().zip(&by_hurry) {
        line(name, t);
    }
    println!("throws by balance:");
    line("on balance", &by_balance[0]);
    line("off balance", &by_balance[1]);
    println!("throws by position:");
    for (pos, t) in &by_pos {
        line(POS[(*pos as usize).min(9)], t);
    }
}
