//! The umpire's out calls (rules-spec §1.3; owner request 2026-11-12): every out is called,
//! within a second of the moment it was made, and almost at once on a close play. "The moment
//! it was made" and "how close" are checked against what the play recorded independently of
//! the call code: the facts sent to the rules engine and the runners' recorded tracks.

use bb_game::game::Game;
use bb_rules::{Event, Input};

const BASE_L: f64 = 27.432;

/// A fact that can retire a runner, and its time.
fn deciding_t(f: &Input) -> Option<f64> {
    match *f {
        Input::PitchCrossed { t, .. } | Input::SwingMeasured { t, .. } | Input::GloveSecured { t, .. } | Input::TagContact { t, .. } => Some(t),
        Input::BaseTouched { t, holding_ball: true, .. } => Some(t),
        _ => None,
    }
}

/// A runner's recorded position at time t (linear between the 0.1 s samples; a retired
/// runner's track stops at the sample before his out, so up to 0.2 s past its end it is
/// carried on from its last two samples).
fn track_at(track: &[(f64, bb_math::Vec3)], t: f64) -> Option<bb_math::Vec3> {
    let n = track.len();
    if n < 2 || t > track[n - 1].0 + 0.2 {
        return None;
    }
    let i = track.iter().position(|k| k.0 >= t).unwrap_or(n - 1).max(1);
    let (a, b) = (track[i - 1], track[i]);
    let u = ((t - a.0) / (b.0 - a.0).max(1e-9)).max(0.0);
    Some(a.1 + (b.1 - a.1) * u)
}

fn base_xy(base: u8) -> bb_math::Vec3 {
    let h = BASE_L / std::f64::consts::SQRT_2;
    match base {
        1 => bb_math::Vec3::new(h, h, 0.0),
        2 => bb_math::Vec3::new(0.0, 2.0 * h, 0.0),
        3 => bb_math::Vec3::new(-h, h, 0.0),
        _ => bb_math::Vec3::ZERO,
    }
}

#[test]
fn every_out_is_called_promptly_and_close_plays_at_once() {
    let (mut outs, mut close, mut checked_margin) = (0, 0, 0);
    for seed in 0..6 {
        let mut g = Game::new(seed, true);
        while let Some(s) = g.step_pitch() {
            // One call per out, in time order.
            let n_outs = s.rules_events.iter().filter(|e| matches!(e, Event::RunnerOut { .. })).count();
            assert_eq!(s.calls.len(), n_outs, "play {}: {} outs, {} calls", s.play, n_outs, s.calls.len());
            for w in s.calls.windows(2) {
                assert!(w[0].t_out <= w[1].t_out);
            }
            for c in &s.calls {
                outs += 1;
                // The out was made at the instant of a fact that can make one.
                assert!(
                    s.facts.iter().filter_map(deciding_t).any(|t| (t - c.t_out).abs() < 1e-9),
                    "play {}: out of {:#x} at {:.3} s matches no deciding fact",
                    s.play,
                    c.runner,
                    c.t_out
                );
                // Called within a second, never before it happened; at once when close.
                let delay = c.t_call() - c.t_out;
                assert!(delay > 0.1 && delay <= 1.0, "play {}: called {delay:.2} s after the out", s.play);
                if c.margin_s <= 0.1 {
                    assert!(delay <= 0.25, "play {}: a bang-bang play called {delay:.2} s after", s.play);
                    close += 1;
                }
                // On a play at a base, the margin is the runner's distance from the bag, as
                // his recorded track has it, at an average runner's speed (8.2 m/s; the
                // tracks are 0.1 s samples, so allow 1.5 m).
                if (1..=3).contains(&c.base) && c.margin_s.is_finite() && c.margin_s < 1.0 {
                    if let Some(track) = s.viz.actors.iter().find(|(id, _)| *id == c.runner).map(|x| &x.1) {
                        if let Some(p) = track_at(track, c.t_out) {
                            let d = (p - base_xy(c.base)).norm();
                            assert!((d - c.margin_s * 8.2).abs() < 1.5, "play {}: runner {d:.2} m from base {} but margin {:.2} s", s.play, c.base, c.margin_s);
                            checked_margin += 1;
                        }
                    }
                }
            }
            // A viewer sees every call before the clip ends.
            let tl = bb_game::timeline::encode(&s, &g.st);
            let tl = flatbuffers::root::<bb_schema::tl::PlayTimeline>(&tl).unwrap();
            for c in &s.calls {
                assert!(c.t_call() as f32 <= tl.duration_s() + 1e-4);
            }
        }
    }
    assert!(outs > 300 && close > 50 && checked_margin > 30, "{outs} outs, {close} close, {checked_margin} margins checked");
}
