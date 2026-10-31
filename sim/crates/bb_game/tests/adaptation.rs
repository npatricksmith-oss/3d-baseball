//! Times through the order (calibration-targets §5.2): with adaptation on and fatigue off,
//! the batter does better the third time than the first; with adaptation off, no trend.

use bb_game::pa::{Call, Knobs, Matchup, PaRecord, simulate_times_through};

fn quality(paths: &[Vec<PaRecord>], t: usize) -> (f64, f64) {
    // (contact rate per swing, mean exit speed on contact) for the t-th time through.
    let (mut sw, mut ct, mut ev, mut n) = (0u32, 0u32, 0.0, 0u32);
    for g in paths {
        for p in &g[t].pitches {
            if p.swung {
                sw += 1;
                if p.contact {
                    ct += 1;
                }
            }
            if let (Some((e, _, _)), true) = (p.batted, p.call == Call::InPlay) {
                ev += e;
                n += 1;
            }
        }
    }
    (ct as f64 / sw.max(1) as f64, ev / n.max(1) as f64)
}

fn run(adapt: bool) -> Vec<Vec<PaRecord>> {
    let mut mu = Matchup::average();
    mu.knobs = Knobs::calibrated_m3();
    if let Ok(v) = std::env::var("BB_SIGMA0_SPEED") { mu.model_error.as_mut().unwrap().speed = v.parse().unwrap(); }
    if let Ok(v) = std::env::var("BB_SIGMA0_AXIS") { mu.model_error.as_mut().unwrap().spin_axis_deg = v.parse().unwrap(); }
    let ng: u32 = std::env::var("BB_GAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(2000);
    let games: Vec<u32> = (0..ng).collect();
    std::thread::scope(|s| {
        let chunks: Vec<_> = games
            .chunks((ng as usize / 16).max(1))
            .map(|c| {
                let mu = &mu;
                s.spawn(move || c.iter().map(|&g| simulate_times_through(mu, 21, g, 3, 30, adapt)).collect::<Vec<_>>())
            })
            .collect();
        chunks.into_iter().flat_map(|h| h.join().unwrap()).collect()
    })
}

#[test]
fn third_time_better_than_first_with_adaptation() {
    let on = run(true);
    let (c1, e1) = quality(&on, 0);
    let (c3, e3) = quality(&on, 2);
    println!("adapt on: contact {c1:.3} -> {c3:.3}, EV {e1:.2} -> {e3:.2} m/s");
    let off = run(false);
    let (d1, f1) = quality(&off, 0);
    let (d3, f3) = quality(&off, 2);
    println!("adapt off: contact {d1:.3} -> {d3:.3}, EV {f1:.2} -> {f3:.2} m/s");
    assert!(c3 > c1, "contact should rise with exposure");
    // The adaptation gain must exceed whatever drift exists without adaptation.
    assert!((c3 - c1) > (d3 - d1) + 0.01, "gain {} vs no-adapt drift {}", c3 - c1, d3 - d1);
}
