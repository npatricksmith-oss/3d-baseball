//! Plate-appearance rates for the four pairings of pitcher's hand and batter's side
//! (owner backlog A1). Average players; only the hands differ. A left-hander is the
//! right-hander's mirror image, so same-side pairings (R/R, L/L) should agree with each
//! other and opposite-side pairings (R/L, L/R) with each other; any gap between the two
//! groups is a platoon split that comes out of the geometry, not a setting.
//!
//! `cargo run --release -p bb_game --example platoon -- [n_pa] [seed]`

use bb_game::pa::{Knobs, Matchup, PaResult, simulate_pa};
use bb_observe::Hand;

fn main() {
    let mut args = std::env::args().skip(1);
    let n: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4000);
    let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    println!("{n} PA each, seed {seed}, calibrated knobs");
    println!("P/B     K%    BB%   HBP%  in play%  swing%  whiff/swing%  zone%   spray° (+ = right field)");
    for (throws, bats) in [(Hand::Right, Hand::Right), (Hand::Left, Hand::Left), (Hand::Right, Hand::Left), (Hand::Left, Hand::Right)] {
        let mut mu = Matchup::for_hands(throws, bats);
        mu.knobs = Knobs::calibrated_m3();
        let (mut k, mut bb, mut hbp, mut bip) = (0u32, 0u32, 0u32, 0u32);
        let (mut pitches, mut swings, mut whiffs, mut zone) = (0u32, 0u32, 0u32, 0u32);
        let (mut spray, mut n_spray) = (0.0, 0u32);
        for i in 0..n {
            let pa = simulate_pa(&mu, seed, i * bb_game::pa::PA_PLAY_STRIDE);
            match pa.result {
                PaResult::Strikeout => k += 1,
                PaResult::Walk => bb += 1,
                PaResult::HitByPitch => hbp += 1,
                PaResult::InPlay => bip += 1,
                PaResult::Unresolved => {}
            }
            for p in &pa.pitches {
                pitches += 1;
                swings += p.swung as u32;
                whiffs += (p.swung && p.contact_state.is_none()) as u32;
                zone += p.in_zone as u32;
            }
            if pa.result == PaResult::InPlay {
                if let Some((_, _, s)) = pa.pitches.last().and_then(|p| p.batted) {
                    spray += s.to_degrees();
                    n_spray += 1;
                }
            }
        }
        let pct = |a: u32, b: u32| 100.0 * a as f64 / b.max(1) as f64;
        let h = |x: Hand| if x == Hand::Right { 'R' } else { 'L' };
        println!(
            "{}/{}   {:5.1}  {:5.1}  {:5.2}  {:7.1}  {:6.1}  {:11.1}  {:5.1}  {:6.1}",
            h(throws),
            h(bats),
            pct(k, n),
            pct(bb, n),
            pct(hbp, n),
            pct(bip, n),
            pct(swings, pitches),
            pct(whiffs, swings),
            pct(zone, pitches),
            spray / n_spray.max(1) as f64
        );
    }
}
