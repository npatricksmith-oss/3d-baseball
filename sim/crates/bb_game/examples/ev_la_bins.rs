//! Read-only diagnostic: one CSV row per fair batted ball (exit speed, launch angle, spray,
//! home run or not) over the standard run's seeds, for the exit-speed × launch-angle table.
//! Changes no game; prints a digest fold so a before/after check is possible.
//! `cargo run --release -p bb_game --example ev_la_bins -- <seed-base> <n> <out.csv>`
//! (the standard run's seeds are `base * 1_000_003 + i`, as in `sim-cli games`).
use bb_rules::{Event, PaResult};
use std::io::Write;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let base: u64 = a[1].parse().unwrap();
    let n: u64 = a[2].parse().unwrap();
    let mut out = std::io::BufWriter::new(std::fs::File::create(&a[3]).unwrap());
    writeln!(out, "ev_mph,la_deg,spray_pull_deg,hr").unwrap();
    let (mut pa, mut hr_pa) = (0u64, 0u64);
    let mut fold = [0u8; 32];
    for i in 0..n {
        let seed = base * 1_000_003 + i;
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            for e in &step.rules_events {
                if let Event::PlateAppearanceEnded { result, .. } = e {
                    pa += 1;
                    if matches!(result, PaResult::HomeRun) {
                        hr_pa += 1;
                    }
                }
            }
            let Some((ev, la, spray)) = step.record.batted else { continue };
            if !step.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: true })) {
                continue;
            }
            let lefty = step.record.swing.map(|s| s.pivot.x > 0.0).unwrap_or(false);
            let pull = if lefty { spray } else { -spray };
            let hr = step.rules_events.iter().any(|e| matches!(e, Event::PlateAppearanceEnded { result: PaResult::HomeRun, .. }));
            writeln!(out, "{:.3},{:.3},{:.3},{}", ev / 0.44704, la.to_degrees(), pull.to_degrees(), hr as u8).unwrap();
        }
        let d = g.finish().digest;
        for (f, b) in fold.iter_mut().zip(d.iter()) {
            *f ^= b;
        }
    }
    let hex: String = fold.iter().map(|b| format!("{b:02x}")).collect();
    println!("games {n} PA {pa} HR {hr_pa} HR/PA {:.3}% fold {hex}", 100.0 * hr_pa as f64 / pa as f64);
}
