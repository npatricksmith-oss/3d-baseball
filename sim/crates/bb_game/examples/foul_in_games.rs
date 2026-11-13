//! Diagnostic: the foul share of contact in full games, where the rules engine judges each
//! batted ball from the live play's facts (rules-spec §5.2: an infield ball stays pending
//! until it settles, is touched, or passes first or third), against the plate-appearance
//! model's own call (`pa.rs`: fair or foul where the ball first comes down), by launch angle,
//! with the MLB share from R13 F3 beside it. A caught foul fly counts as a foul (Statcast's
//! caught fouls are in play with "in foul territory"; R13 F1 gives 51.2% without them and
//! 52.5% with them). `-- [games] [first seed]`
use bb_game::pa::Call;
use bb_rules::Event;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let games: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(128);
    let first: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    // LA bins (lower edges) and MLB P(foul | contact) per bin, R13 F3.
    const EDGES: [f64; 9] = [-1e9, -30.0, -10.0, 10.0, 25.0, 40.0, 50.0, 60.0, 70.0];
    const MLB: [f64; 9] = [57.0, 41.0, 22.0, 37.0, 47.0, 66.0, 74.0, 78.0, 87.0];
    // [bin] -> [contacts, PA-model fouls, game fouls, game fouls the PA model called fair, PA fouls the game called fair]
    let mut n = [[0u64; 5]; 9];
    let mut undecided = 0u64;
    for seed in first..first + games {
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            let Some((_, la, _)) = step.record.batted else { continue };
            let la = la.to_degrees();
            let k = EDGES.iter().rposition(|e| la >= *e).unwrap();
            let pa_foul = matches!(step.record.call, Call::Foul | Call::FoulTip);
            let fair = step.rules_events.iter().find_map(|e| if let Event::FairFoulDecided { fair } = e { Some(*fair) } else { None });
            // A play timeout forces a fair decision (rules-spec §5.8): not a judgment.
            let timeout = step.rules_events.iter().any(|e| matches!(e, Event::RulesWarning { code: "play_timeout" }));
            let Some(fair) = fair.filter(|_| !timeout) else {
                undecided += 1;
                continue;
            };
            n[k][0] += 1;
            n[k][1] += pa_foul as u64;
            n[k][2] += !fair as u64;
            n[k][3] += (!fair && !pa_foul) as u64;
            n[k][4] += (fair && pa_foul) as u64;
        }
    }
    println!("{games} games from seed {first}; contacts with no fair/foul decision, or a play timeout: {undecided}");
    println!("{:>10} | {:>6} | {:>8} {:>8} {:>6} | {:>14} {:>14}", "LA bin °", "n", "PA call", "game", "MLB", "foul, PA fair", "fair, PA foul");
    let mut tot = [0u64; 5];
    for (k, row) in n.iter().enumerate() {
        let c = row[0].max(1) as f64;
        let label = if k == 0 { "< -30".to_string() } else if k == 8 { "≥ 70".to_string() } else { format!("{} to {}", EDGES[k], EDGES[k + 1]) };
        println!(
            "{label:>10} | {:6} | {:8.1} {:8.1} {:6.0} | {:14} {:14}",
            row[0],
            100.0 * row[1] as f64 / c,
            100.0 * row[2] as f64 / c,
            MLB[k],
            row[3],
            row[4]
        );
        for i in 0..5 {
            tot[i] += row[i];
        }
    }
    let c = tot[0].max(1) as f64;
    println!("{:>10} | {:6} | {:8.1} {:8.1} {:>6} | {:14} {:14}", "all", tot[0], 100.0 * tot[1] as f64 / c, 100.0 * tot[2] as f64 / c, "52.5", tot[3], tot[4]);
    println!("(MLB all: 52.5% counts caught fouls as fouls, as this table does; 51.2% without them, R13 F1)");
}
