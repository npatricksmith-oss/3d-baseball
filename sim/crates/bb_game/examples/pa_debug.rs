//! Calibration debugging: where are swings lost?
use bb_game::pa::{Matchup, scouting_for_pa, simulate_pitch};
use bb_observe::Count;

fn main() {
    let mut mu = Matchup::average();
    if let Some(k) = std::env::args().nth(2).and_then(|s| s.parse().ok()) { mu.knobs.perception = k; }
    if let Some(k) = std::env::args().nth(3).and_then(|s| s.parse().ok()) { mu.knobs.command = k; }
    let n: u32 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(500);
    let (mut zone, mut dec_sw_z, mut dec_sw_o, mut chk, mut ps_hi_z, mut ps_hi_o) = (0, 0, 0, 0, 0, 0);
    let mut err_z = Vec::new();
    let (mut fe, mut ce) = (Vec::new(), Vec::new());
    for i in 0..n {
        let sc = scouting_for_pa(&mu, 7, i * 64);
        let r = simulate_pitch(&mu, &sc, Count { balls: 1, strikes: 1 }, 7, i * 64);
        let d = r.decision.unwrap();
        if r.in_zone { zone += 1 }
        if d.swing { if r.in_zone { dec_sw_z += 1 } else { dec_sw_o += 1 } }
        if r.check_swing { chk += 1 }
        if d.p_strike > 0.5 { if r.in_zone { ps_hi_z += 1 } else { ps_hi_o += 1 } }
        if let Some(e) = r.aim_error { fe.push(e); }
        if let Some(e) = r.commit_error { ce.push(e); }
        if let Some(p) = r.plate { err_z.push((d.aim.mean.y - p.z, d.aim.mean.x - p.x, d.aim.cov.r[1][1].sqrt())); }
        if i < 15 {
            let p = r.plate.unwrap_or_default();
            println!("{:?} plate ({:+.2},{:.2}) z{} read {:?} {:.2} aim ({:+.2},{:.2}) σ({:.3},{:.3}) Ps {:.2} EVs {:+.3} EVt {:+.3} sw {} chk {}",
                r.ty, p.x, p.z, r.in_zone as u8, r.read, r.read_weight, d.aim.mean.x, d.aim.mean.y, d.aim.cov.r[0][0].sqrt(), d.aim.cov.r[1][1].sqrt(), d.p_strike, d.ev_swing, d.ev_take, d.swing as u8, r.check_swing as u8);
        }
    }
    let o = n - zone;
    println!("zone {zone}/{n}; decided swing: zone {:.0}%, out {:.0}%; checks {chk}; Ps>0.5: zone {:.0}%, out {:.0}%",
        100.0 * dec_sw_z as f64 / zone as f64, 100.0 * dec_sw_o as f64 / o as f64, 100.0 * ps_hi_z as f64 / zone as f64, 100.0 * ps_hi_o as f64 / o as f64);
    let k = err_z.len() as f64;
    let _ = k;
    let rms = |f: &dyn Fn(&(f64, f64, f64)) -> f64| (err_z.iter().map(|e| f(e) * f(e)).sum::<f64>() / k).sqrt();
    let rms3 = |v: &[bb_math::Vec3]| {
        let n = v.len().max(1) as f64;
        ((v.iter().map(|e| e.x * e.x).sum::<f64>() / n).sqrt(), (v.iter().map(|e| e.y * e.y).sum::<f64>() / n).sqrt(), (v.iter().map(|e| e.z * e.z).sum::<f64>() / n).sqrt() * 1000.0)
    };
    println!("swing aim error rms (x m, z m, t ms): commit {:?} final {:?} (n={})", rms3(&ce), rms3(&fe), fe.len());
    println!("aim error rms: z {:.3} m, x {:.3} m; mean believed σz {:.3}", rms(&|e| e.0), rms(&|e| e.1), err_z.iter().map(|e| e.2).sum::<f64>() / k);
}

