//! Chase diagnostics: what the batter believed at commit on pitches near the zone edge.
//! Usage: `cargo run --release -p bb_game --example chase_diag -- [PAs] [c_D] [b_D]`
use bb_game::pa::{B1Report, Knobs, Matchup, PA_PLAY_STRIDE, simulate_pa};

fn main() {
    let n: u32 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3000);
    let mut mu = Matchup::average();
    mu.knobs = Knobs::calibrated_m3();
    if let Some(c) = std::env::args().nth(2).and_then(|s| s.parse().ok()) {
        mu.discipline.c = c;
    }
    if let Some(b) = std::env::args().nth(3).and_then(|s| s.parse().ok()) {
        mu.discipline.b = b;
    }
    println!("{:?}", mu.discipline);
    let mut rep = B1Report::default();
    let r = mu.env.ball.radius;
    // Signed distance of the ball's edge from the zone edge, cm (negative = inside).
    let edges = [-100.0, -8.5, 0.0, 8.5, 25.5, 1e9];
    let names = ["heart", "shadow in", "shadow out", "chase", "waste"];
    // n, swings, Σ p_strike, Σ (ev_swing − ev_take), Σ commit err x², z², n_err, decided
    let mut acc = [[0.0f64; 8]; 5];
    let mut by_strikes = [[0.0f64; 2]; 3];
    for i in 0..n {
        let pa = simulate_pa(&mu, 1, i * PA_PLAY_STRIDE);
        rep.add(&pa);
        for p in &pa.pitches {
            let Some(at) = p.plate else { continue };
            let dx = at.x.abs() - mu.zone.half_width;
            let dz = (mu.zone.bottom - at.z).max(at.z - mu.zone.top);
            let d = if dx <= 0.0 && dz <= 0.0 { dx.max(dz) } else { dx.max(0.0).hypot(dz.max(0.0)) };
            let cm = 100.0 * (d - r);
            let k = (0..5).find(|&k| cm < edges[k + 1]).unwrap();
            let a = &mut acc[k];
            a[0] += 1.0;
            a[1] += p.swung as u32 as f64;
            if let Some(dc) = &p.decision {
                a[2] += dc.p_strike;
                a[3] += dc.ev_swing - dc.ev_take;
                a[7] += 1.0;
            }
            if let Some(e) = p.commit_error {
                a[4] += e.x * e.x;
                a[5] += e.y * e.y;
                a[6] += 1.0;
            }
            if k == 2 {
                let s = p.count.strikes.min(2) as usize;
                by_strikes[s][0] += 1.0;
                by_strikes[s][1] += p.swung as u32 as f64;
            }
        }
    }
    println!("zone band      n   swing%  mean P(strike)  mean EVswing−EVtake  commit err σx cm  σz cm");
    for k in 0..5 {
        let a = acc[k];
        println!("{:11} {:5.0}   {:5.1}      {:5.3}          {:+.4}            {:5.1}        {:5.1}", names[k], a[0], 100.0 * a[1] / a[0].max(1.0), a[2] / a[7].max(1.0), a[3] / a[7].max(1.0), 100.0 * (a[4] / a[6].max(1.0)).sqrt(), 100.0 * (a[5] / a[6].max(1.0)).sqrt());
    }
    for s in 0..3 {
        println!("shadow-out swing% with {s} strikes: {:.1} (n {:.0})", 100.0 * by_strikes[s][1] / by_strikes[s][0].max(1.0), by_strikes[s][0]);
    }
    let out: Vec<String> = rep.rows().iter().map(|(_, name, v, _, lo, hi)| format!("{name} {v:.1}{}", if v < lo || v > hi { "*" } else { "" })).collect();
    println!("{}", out.join(" | "));
}
