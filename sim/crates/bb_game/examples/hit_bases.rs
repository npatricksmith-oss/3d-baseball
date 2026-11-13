//! Diagnostic: fair balls in play by batted-ball type, and how many bases the batter's hits
//! reached (single, double, triple, home run of either kind; "hit" counts all four, unlike
//! `sim-cli games`, which leaves home runs out of its BABIP), to tell whether a change in
//! runs comes from more hits or from longer ones. `-- [games] [first seed]`
use bb_rules::Event;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let games: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(256);
    let first: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    const NAMES: [&str; 4] = ["GB (<10°)", "LD (10–25°)", "FB (25–50°)", "PU (>50°)"];
    // [type] -> [balls, 1B, 2B, 3B, HR]
    let mut n = [[0u64; 5]; 4];
    let mut runs = 0u64;
    for seed in first..first + games {
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            runs += step
                .rules_events
                .iter()
                .filter(|e| matches!(e, Event::RunScored { .. }))
                .count() as u64;
            let Some((_, la, _)) = step.record.batted else {
                continue;
            };
            if !step
                .rules_events
                .iter()
                .any(|e| matches!(e, Event::FairFoulDecided { fair: true }))
            {
                continue;
            }
            let la = la.to_degrees();
            let k = if la < 10.0 {
                0
            } else if la < 25.0 {
                1
            } else if la < 50.0 {
                2
            } else {
                3
            };
            n[k][0] += 1;
            let batter = step.rules_events.iter().find_map(|e| {
                if let Event::PlateAppearanceEnded { batter, .. } = e {
                    Some(*batter)
                } else {
                    None
                }
            });
            let Some(batter) = batter else { continue };
            let (_, hit) = bb_game::game::classify_bip(batter, &step.rules_events, &step.facts);
            if !hit {
                continue;
            }
            let bases = step
                .rules_events
                .iter()
                .filter_map(|e| {
                    if let Event::RunnerAdvanced { runner, to, .. } = e {
                        (*runner == batter).then_some(*to)
                    } else {
                        None
                    }
                })
                .max()
                .unwrap_or(1);
            n[k][bases.clamp(1, 4) as usize] += 1;
        }
    }
    println!(
        "{games} games from seed {first}: runs/team-game {:.2}",
        runs as f64 / (2 * games) as f64
    );
    println!(
        "{:>12} {:>6} {:>7} {:>7} {:>7} {:>7} {:>7}",
        "type", "balls", "hit", "1B", "2B", "3B", "HR"
    );
    for (k, row) in n.iter().enumerate() {
        let b = row[0].max(1) as f64;
        let h: u64 = row[1..].iter().sum();
        println!(
            "{:>12} {:>6} {:>7.3} {:>7.3} {:>7.3} {:>7.3} {:>7.3}",
            NAMES[k],
            row[0],
            h as f64 / b,
            row[1] as f64 / b,
            row[2] as f64 / b,
            row[3] as f64 / b,
            row[4] as f64 / b
        );
    }
}
