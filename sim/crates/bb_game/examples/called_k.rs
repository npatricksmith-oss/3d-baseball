//! Calibration debugging: anatomy of called third strikes.
use bb_game::pa::{Call, Knobs, Matchup, PaResult, simulate_pa};

fn main() {
    let mut mu = Matchup::average();
    mu.knobs = Knobs::calibrated_m3();
    let (mut ck, mut ck_check, mut edge, mut ps_lo) = (0, 0, 0, 0);
    let mut two_strike_takes_zone = 0;
    let mut two_strike_zone = 0;
    for i in 0..3000u32 {
        let pa = simulate_pa(&mu, 4, i * 1000);
        for p in &pa.pitches {
            if p.count.strikes == 2 && p.in_zone {
                two_strike_zone += 1;
                if !p.swung { two_strike_takes_zone += 1; }
            }
        }
        if pa.result == PaResult::Strikeout && pa.called_third {
            ck += 1;
            let p = pa.pitches.last().unwrap();
            if p.check_swing { ck_check += 1; }
            let pl = p.plate.unwrap();
            let z = mu.zone;
            let dx = z.half_width - pl.x.abs();
            let dz = (pl.z - z.bottom).min(z.top - pl.z);
            if dx.min(dz) < 0.05 { edge += 1; }
            if p.decision.unwrap().p_strike < 0.3 { ps_lo += 1; }
            let _ = Call::CalledStrike;
        }
    }
    println!("called K {ck}: check-swing {ck_check}, within 5 cm of an edge {edge}, believed P(strike) < 0.3 at commit {ps_lo}");
    println!("two-strike in-zone pitches {two_strike_zone}, taken {two_strike_takes_zone} ({:.0}%)", 100.0 * two_strike_takes_zone as f64 / two_strike_zone as f64);
}
