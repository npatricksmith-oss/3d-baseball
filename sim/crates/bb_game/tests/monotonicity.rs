//! Monotonicity (calibration-targets §7.1, agent-models §11): grade 30 vs grade 70 with
//! common random numbers moves each attribute's primary metric in the physical direction.
//! Weak or unresolved attributes (SpinCapacity, SpinRecognition, QuickHands) are tracked by
//! `sim-cli sensitivity` and the calibration report rather than asserted here.

use bb_agents::attributes::{BatterGrades, PitcherGrades};
use bb_game::pa::{B1Report, Matchup, run_pas};

const N: u32 = 900;
const SEED: u64 = 11;

fn report(set: fn(&mut BatterGrades, &mut PitcherGrades, f64), g: f64) -> B1Report {
    let (mut b, mut p) = (BatterGrades::default(), PitcherGrades::default());
    set(&mut b, &mut p, g);
    run_pas(&Matchup::with_grades(&b, &p), N, SEED)
}

fn pair(set: fn(&mut BatterGrades, &mut PitcherGrades, f64)) -> (B1Report, B1Report) {
    std::thread::scope(|s| {
        let lo = s.spawn(move || report(set, 30.0));
        let hi = s.spawn(move || report(set, 70.0));
        (lo.join().unwrap(), hi.join().unwrap())
    })
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len().max(1) as f64
}
fn contact(r: &B1Report) -> f64 {
    r.contacts as f64 / r.swings.max(1) as f64
}

#[test]
fn bat_speed_raises_exit_velocity() {
    let (lo, hi) = pair(|b, _, g| b.bat_speed = g);
    assert!(mean(&hi.bbe_ev) > mean(&lo.bbe_ev) + 4.0, "{} vs {}", mean(&lo.bbe_ev), mean(&hi.bbe_ev));
}

#[test]
fn vision_raises_contact() {
    let (lo, hi) = pair(|b, _, g| b.vision = g);
    assert!(contact(&hi) > contact(&lo) + 0.05);
}

#[test]
fn bat_control_raises_contact() {
    let (lo, hi) = pair(|b, _, g| b.bat_control = g);
    assert!(contact(&hi) > contact(&lo));
}

#[test]
fn discipline_lowers_chase() {
    let (lo, hi) = pair(|b, _, g| b.discipline = g);
    let chase = |r: &B1Report| r.o_swings as f64 / (r.pitches - r.zone).max(1) as f64;
    assert!(chase(&hi) < chase(&lo), "{} vs {}", chase(&lo), chase(&hi));
}

#[test]
fn command_lowers_walks() {
    let (lo, hi) = pair(|_, p, g| p.command = g);
    assert!(hi.bb * 2 < lo.bb, "{} vs {}", lo.bb, hi.bb);
}

#[test]
fn arm_speed_lowers_contact() {
    let (lo, hi) = pair(|_, p, g| p.arm_speed = g);
    assert!(contact(&hi) < contact(&lo));
}
