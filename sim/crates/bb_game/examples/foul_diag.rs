//! Foul diagnostics: where contact goes (spray, launch) and what timing error produced it.
//! Usage: `cargo run --release -p bb_game --example foul_diag -- [PAs] [timing σ ms] [bat-control σ cm] [K_perception]`
use bb_game::pa::{B1Report, Call, Knobs, Matchup, PA_PLAY_STRIDE, simulate_pa};

fn pct(v: &mut [f64], q: f64) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[((v.len() - 1) as f64 * q) as usize]
}

fn main() {
    let n: u32 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3000);
    let mut mu = Matchup::average();
    mu.knobs = Knobs::calibrated_m3();
    if let Some(t) = std::env::args().nth(2).and_then(|s| s.parse::<f64>().ok()) {
        mu.batter.timing = t / 1000.0;
    }
    if let Some(c) = std::env::args().nth(3).and_then(|s| s.parse::<f64>().ok()) {
        mu.batter.bat_control = c / 100.0;
    }
    if let Some(k) = std::env::args().nth(4).and_then(|s| s.parse::<f64>().ok()) {
        mu.knobs.perception = k;
    }
    println!("timing σ {:.1} ms, bat-control σ {:.2} cm, K_perception {}", 1000.0 * mu.batter.timing, 100.0 * mu.batter.bat_control, mu.knobs.perception);
    let mut rep = B1Report::default();
    let (mut spray, mut la, mut terr, mut zerr) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut contacts, mut fouls, mut back, mut pull, mut oppo, mut swings) = (0u32, 0u32, 0u32, 0u32, 0u32, 0u32);
    for i in 0..n {
        let pa = simulate_pa(&mu, 1, i * PA_PLAY_STRIDE);
        rep.add(&pa);
        for p in &pa.pitches {
            swings += p.swung as u32;
            let Some((_, l, s)) = p.batted else { continue };
            contacts += 1;
            let (l, s) = (l.to_degrees(), s.to_degrees());
            spray.push(s);
            la.push(l);
            if let Some(e) = p.aim_error {
                terr.push(1000.0 * e.z);
                zerr.push(100.0 * e.y);
            }
            if matches!(p.call, Call::Foul | Call::FoulTip) {
                fouls += 1;
                if s.abs() > 90.0 || l > 90.0 {
                    back += 1;
                } else if s < 0.0 {
                    pull += 1;
                } else {
                    oppo += 1;
                }
            }
        }
    }
    let c = contacts.max(1) as f64;
    println!("swings {swings}, contact {contacts} ({:.1}%), foul {:.1}% of contact: pull side {:.1}, opposite {:.1}, backward {:.1}", 100.0 * c / swings as f64, 100.0 * fouls as f64 / c, 100.0 * pull as f64 / c, 100.0 * oppo as f64 / c, 100.0 * back as f64 / c);
    for (name, v) in [("spray °", &mut spray), ("launch °", &mut la), ("aim timing error ms", &mut terr), ("aim height error cm", &mut zerr)] {
        println!("{name:20} p5 {:7.1}  p25 {:7.1}  p50 {:7.1}  p75 {:7.1}  p95 {:7.1}", pct(v, 0.05), pct(v, 0.25), pct(v, 0.5), pct(v, 0.75), pct(v, 0.95));
    }
    let out: Vec<String> = rep.rows().iter().map(|(_, name, v, _, lo, hi)| format!("{name} {v:.1}{}", if v < lo || v > hi { "*" } else { "" })).collect();
    println!("{}", out.join(" | "));
}
