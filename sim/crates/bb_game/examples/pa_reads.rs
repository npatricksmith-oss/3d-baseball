//! Calibration debugging: read accuracy and chase by pitch type.
use bb_game::pa::{Matchup, scouting_for_pa, simulate_pitch};
use bb_observe::Count;
use std::collections::BTreeMap;

fn main() {
    let mut mu = Matchup::average();
    mu.knobs.perception = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(2.5);
    mu.knobs.swing = 2.0;
    let mut by: BTreeMap<String, [u32; 6]> = BTreeMap::new(); // n, misread, out, chase, zone, zswing
    let mut ps_hist = [0u32; 10];
    for i in 0..1500u32 {
        let sc = scouting_for_pa(&mu, 9, i * 64);
        let r = simulate_pitch(&mu, &sc, Count { balls: 1, strikes: 2 }, 9, i * 64);
        let e = by.entry(format!("{:?}", r.ty)).or_default();
        e[0] += 1;
        e[1] += (r.read != r.ty) as u32;
        if r.in_zone { e[4] += 1; e[5] += r.swung as u32 } else { e[2] += 1; e[3] += r.swung as u32 }
        if !r.in_zone {
            let d = r.decision.unwrap();
            ps_hist[((d.p_strike * 10.0) as usize).min(9)] += 1;
        }
    }
    for (k, e) in by {
        println!("{k:10} n {:4} misread {:4.1}% chase {:4.1}% zswing {:4.1}%", e[0], 100.0 * e[1] as f64 / e[0] as f64, 100.0 * e[3] as f64 / e[2].max(1) as f64, 100.0 * e[5] as f64 / e[4].max(1) as f64);
    }
    println!("believed P(strike) for out-of-zone pitches (deciles): {ps_hist:?}");
}
