//! Velocity anchoring (agent-models §3.1): after a perceived fastball, the batter expects
//! the next pitch sooner, so a changeup finds it early; after a changeup, it is ready later.

use bb_agents::pitcher::PitchIntent;
use bb_game::pa::{Knobs, Matchup, simulate_pitch_ctx};
use bb_observe::{Count, PitchType};

#[test]
fn changeup_after_fastball_is_timed_early() {
    let mut mu = Matchup::average();
    mu.knobs = Knobs::calibrated_m3();
    let bs = vec![mu.perception.break_sigma; mu.arsenal.len()];
    let ff_speed = mu.scouting.types.iter().find(|t| t.ty == PitchType::FourSeam).unwrap().speed;
    let ch_speed = mu.scouting.types.iter().find(|t| t.ty == PitchType::Changeup).unwrap().speed;
    // Same changeups (common seeds), different anchors; compare commit-time timing error.
    let (mut after_ff, mut after_ch, mut n) = (0.0, 0.0, 0);
    for play in 0..4000u32 {
        let rec_ff = simulate_pitch_ctx(&mu, &mu.scouting, Count::default(), 77, play, &bs, None, Some(ff_speed));
        if rec_ff.ty != PitchType::Changeup {
            continue;
        }
        let rec_ch = simulate_pitch_ctx(&mu, &mu.scouting, Count::default(), 77, play, &bs, None, Some(ch_speed));
        if let (Some(a), Some(b)) = (rec_ff.commit_error, rec_ch.commit_error) {
            after_ff += a.z;
            after_ch += b.z;
            n += 1;
        }
    }
    let _ = PitchIntent { ty: PitchType::Changeup, target_x: 0.0, target_z: 0.7 };
    assert!(n > 30, "too few swung-at changeups: {n}");
    let (a, b) = (after_ff / n as f64, after_ch / n as f64);
    println!("mean commit timing error on changeups: after FF {:.1} ms, after CH {:.1} ms (n={n})", a * 1e3, b * 1e3);
    // Negative = aiming earlier than the ball arrives.
    assert!(a < b, "anchoring to a fastball should make the batter early on a changeup");
}
