//! The limits of the play area (physics-spec §9.5, rules-spec §5.8, A10), over whole games:
//! a ball that leaves the field ends the play at once with a `BallDead` rules event and a
//! `BallDead` timeline marker at that moment, and no fielder's recorded track ever goes
//! through a wall.

use bb_game::game::Game;
use bb_physics::ground::Field;
use bb_rules::{Event, Input, Touch};

#[test]
fn balls_out_of_play_end_the_play_and_fielders_stay_inside_the_walls() {
    let field = Field::default();
    let (mut out_of_play, mut foul_out_of_play, mut tracks) = (0, 0, 0);
    for seed in 0..12u64 {
        let mut g = Game::new(seed, true);
        while let Some(s) = g.step_pitch() {
            // Every fielder (ids with 0x80 set) inside the walls, to the body radius 0.3 m, at
            // every recorded sample.
            for (id, track) in &s.viz.actors {
                if id & 0x80 == 0 {
                    continue;
                }
                tracks += 1;
                for &(t, p) in track {
                    let d = field.body_clearance(p.x, p.y, 0.3).dist;
                    assert!(d > -1e-6, "seed {seed} play {}: fielder {id:#x} {:.3} m into a wall at t {t:.2}", s.play, -d);
                }
            }
            let Some(t_out) = s.facts.iter().find_map(|f| match f {
                Input::BallTouched { t, what: Touch::OutOfPlay { .. } } => Some(*t),
                _ => None,
            }) else {
                continue;
            };
            out_of_play += 1;
            // The play ended there: no later fact, and the rules engine said why.
            let last = s.facts.iter().filter_map(fact_t).fold(f64::NEG_INFINITY, f64::max);
            assert!(last <= t_out + 1e-9, "seed {seed} play {}: facts go on to {last:.2} after the ball left at {t_out:.2}", s.play);
            assert!(s.rules_events.iter().any(|e| matches!(e, Event::BallDead { .. })), "seed {seed} play {}: {:?}", s.play, s.rules_events);
            if s.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: false })) {
                foul_out_of_play += 1;
            }
            // The timeline marks the moment for the client.
            let bytes = bb_game::timeline::encode(&s, &g.st, &g.rosters);
            let tl = flatbuffers::root::<bb_schema::tl::PlayTimeline>(&bytes).unwrap();
            let marked = tl.markers().and_then(|m| m.iter().find(|m| m.kind() == bb_schema::tl::MarkerKind::BallDead && (m.t() as f64 - t_out).abs() < 1e-3));
            let m = marked.unwrap_or_else(|| panic!("seed {seed} play {}: no BallDead marker at {t_out:.3}", s.play));
            // Its point is where the ball crossed the boundary: a ball's radius inside a wall's
            // plane or on the fence's curve, at or above the ground.
            let p = m.p().unwrap();
            let w = field.ball_clearance(p.x() as f64, p.y() as f64, 0.036888);
            assert!(w.dist.abs() < 2e-3 && p.z() >= 0.0, "seed {seed} play {}: dead at ({:.2}, {:.2}, {:.2}): {w:?}", s.play, p.x(), p.y(), p.z());
        }
    }
    assert!(out_of_play > 20 && foul_out_of_play > 0 && tracks > 1000, "{out_of_play} out of play ({foul_out_of_play} foul), {tracks} fielder tracks");
}

fn fact_t(f: &Input) -> Option<f64> {
    match *f {
        Input::PitchDelivered { t }
        | Input::PitchCrossed { t, .. }
        | Input::SwingMeasured { t, .. }
        | Input::BatContact { t }
        | Input::BallTouched { t, .. }
        | Input::BallPassedBase { t, .. }
        | Input::BallSettled { t, .. }
        | Input::PlayTimeout { t }
        | Input::GloveSecured { t, .. }
        | Input::BallReleasedFromGlove { t, .. }
        | Input::BaseTouched { t, .. }
        | Input::BaseLeft { t, .. }
        | Input::TagContact { t, .. }
        | Input::PlayEndCheck { t } => Some(t),
        Input::StartGame => None,
    }
}
