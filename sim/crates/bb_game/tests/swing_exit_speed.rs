//! Exit speed from the v2 swing in the cage, against Statcast (swing note §1, §5).
//!
//! Perfect perception, a four-seamer at the inside, middle and outside thirds, the full swing
//! and release execution noise (`Cage::swing_noisy`, `Cage::release_noisy`), 600 pitches per
//! third. Needs under a second in release mode.
//!
//! Expectations (independent of the code):
//! - Nothing leaves the bat faster than the real maximum band: MLB's fastest ball in play in
//!   the 2025 sample was 116.7 mph, and the record in the Statcast era is about 122. v1's
//!   outer-third swing (sweet spot 83.5 mph at the intercept) sent balls out at up to 136.2 mph
//!   in 100 games (`run_env_diag`, note §1), so a ceiling of 122 fails it by 14 mph.
//! - The exit speed does not climb with the reach. Statcast's exit speed peaks over the middle of
//!   the plate (79.3 inside, 89.6, 93.1 middle, 90.9, 83.9 outside by bin; note §1); v1's grew from 84.0 inside to
//!   99.0 outside (`platoon_diag`), 15 mph, with the outer third the best. The cage has no
//!   perception error, so the test checks that the outer third is not better than the middle
//!   by more than 3 mph, that the middle is the best third, as in Statcast, and that the
//!   spread is below Statcast's own (13.8 mph from the inside bin to the middle). *Changed
//!   2026-11-15:* it asked the thirds to agree within 4 mph, an assumption from before
//!   agent-models §3.6 step 7's partial reach, which makes location matter as Statcast's
//!   contact quality by location says it does (cage: about 91 / 97 / 94 mph).

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
