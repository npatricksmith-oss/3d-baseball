//! Diagnostic: runners put out on the bases after a hit (tag or force), over many games, and
//! runs a team-game as a cross-check against `sim-cli games`. Counts, on plays where the
//! batter got a hit (`classify_bip`): the batter put out advancing past first (a tag at
//! second or beyond), other runners tagged out, other runners forced out. `-- [games]
//! [first seed]`; the standard sets are `300 0` and `300 1000003000`.
use bb_rules::{Event, OutKind};

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let games: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(300);
    let first: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let threads = 2u64;
    let chunk = games.div_ceil(threads);
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            std::thread::spawn(move || {
                // Per class (0 hit, 1 reached by a fielder's choice or error): [plays, batter
                // tagged out, others tagged out, others forced out]; then runs.
                let mut n = [[0u64; 4]; 2];
                let mut runs = 0u64;
                for seed in first + t * chunk..(first + (t + 1) * chunk).min(first + games) {
                    // Average fielders on both sides, as `sim-cli games`.
                    let fattrs: [bb_game::play::FielderAttrs; 9] = core::array::from_fn(bb_game::play::FielderAttrs::average);
                    let mut g = bb_game::game::Game::new(seed, false).with_fielders(fattrs);
                    while let Some(step) = g.step_pitch() {
                        runs += step.rules_events.iter().filter(|e| matches!(e, Event::RunScored { .. })).count() as u64;
                        let batter = step.rules_events.iter().find_map(|e| if let Event::PlateAppearanceEnded { batter, .. } = e { Some(*batter) } else { None });
                        let Some(batter) = batter else { continue };
                        let (reached, hit) = bb_game::game::classify_bip(batter, &step.rules_events, &step.facts);
                        if !reached {
                            continue;
                        }
                        let c = if hit { 0 } else { 1 };
                        n[c][0] += 1;
                        for e in &step.rules_events {
                            if let Event::RunnerOut { runner, kind, .. } = e {
                                match (*runner == batter, kind) {
                                    (true, OutKind::Tag) => n[c][1] += 1,
                                    (false, OutKind::Tag) => n[c][2] += 1,
                                    (false, OutKind::Force) => n[c][3] += 1,
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                (n, runs)
            })
        })
        .collect();
    let mut n = [[0u64; 4]; 2];
    let mut runs = 0u64;
    for h in handles {
        let (m, r) = h.join().unwrap();
        runs += r;
        for c in 0..2 {
            for i in 0..4 {
                n[c][i] += m[c][i];
            }
        }
    }
    println!("games {games} from seed {first}: runs/team-game {:.3}", runs as f64 / (2.0 * games as f64));
    for (name, c) in [("hits", 0), ("reached otherwise", 1)] {
        println!("  {name}: {} plays; batter tagged out {}; other runners tagged out {}; other runners forced out {}", n[c][0], n[c][1], n[c][2], n[c][3]);
    }
}
