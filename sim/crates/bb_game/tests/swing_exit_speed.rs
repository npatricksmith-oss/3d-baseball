//! Exit speed in the cage against Statcast (swing note §1, §5).
//!
//! The cage uses perfect perception and execution noise on 600 four-seamers per plate third.
//! The upper bound is about 122 mph, the Statcast-era record. The middle third should
//! produce the fastest contact, within a 3 mph tolerance for the outer third, and the
//! spread across thirds should stay below Statcast's 13.8 mph middle-to-inside gap.

use bb_agents::pitcher::PitchIntent;
use bb_game::cage::Cage;
use bb_observe::PitchType;
use bb_world::pitch::SwingResult;

const MPH: f64 = 0.44704;

fn exit_speeds_mph(cage: &Cage, x: f64) -> Vec<f64> {
    (0..600u32)
        .filter_map(|play| {
            let rel = cage.release_noisy(&PitchIntent { ty: PitchType::FourSeam, target_x: x, target_z: 0.75 }, 9, play, 1.0);
            match cage.swing_noisy(rel, 9, play, 1.0).swing {
                SwingResult::Contact(b) => Some(b.exit_speed / MPH),
                _ => None,
            }
        })
        .collect()
}

#[test]
fn exit_speed_stays_in_the_real_band_and_does_not_climb_with_reach() {
    let cage = Cage::average();
    let mut means = vec![];
    for (name, x) in [("inside", -0.145), ("middle", 0.0), ("outside", 0.145)] {
        let v = exit_speeds_mph(&cage, x);
        assert!(v.len() > 550, "{name}: only {} contacts in 600 on-time swings", v.len());
        let max = v.iter().cloned().fold(f64::MIN, f64::max);
        assert!(max < 122.0, "{name}: fastest ball {max:.1} mph (MLB max about 117–122; v1 reached 136.2)");
        means.push(v.iter().sum::<f64>() / v.len() as f64);
    }
    let (lo, hi) = (means.iter().cloned().fold(f64::MAX, f64::min), means.iter().cloned().fold(f64::MIN, f64::max));
    assert!(hi - lo < 13.8, "mean exit speed by third {means:?}: spread {:.1} mph (Statcast 13.8, v1 15.0)", hi - lo);
    assert!(means[1] >= means[0] && means[1] >= means[2], "the middle third must be the best: {means:?} (Statcast 79.3 / 93.1 / 83.9)");
    assert!(means[2] < means[1] + 3.0, "outside {:.1} against middle {:.1} mph (v1: outside 8.7 mph better)", means[2], means[1]);
}
