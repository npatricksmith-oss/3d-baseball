//! Throwing ratings in whole games (owner backlog A3; agent-models §5.5, §10).
use bb_game::game::Game;
use bb_game::play::FielderAttrs;

/// Outs per ball in play over `games` games with every fielder's three throwing ratings at
/// the given grades.
fn outs_per_ball_in_play(games: u64, strength: f64, accuracy: f64, transfer: f64) -> f64 {
    let (mut bip, mut outs) = (0u64, 0u64);
    for seed in 0..games {
        let mut g = Game::new(seed, false).with_fielders(core::array::from_fn(|i| FielderAttrs::average(i).with_throwing(strength, accuracy, transfer)));
        while let Some(step) = g.step_pitch() {
            if step.batted.is_some() {
                bip += 1;
                outs += step.calls.len() as u64;
            }
        }
    }
    outs as f64 / bip as f64
}

// Direction check: a defense of grade-80 arms (strength, accuracy and transfer) turns more
// balls in play into outs than one of grade-20 arms. (100 games: .606 against .548.)
#[test]
fn better_arms_make_more_outs() {
    let (weak, strong) = (outs_per_ball_in_play(30, 20.0, 20.0, 20.0), outs_per_ball_in_play(30, 80.0, 80.0, 80.0));
    assert!(strong > weak + 0.02, "grade 80: {strong:.3} outs per ball in play; grade 20: {weak:.3}");
}

// Grade 50 is the average fielder: the ratings change nothing about him.
#[test]
fn grade_50_is_the_average_fielder() {
    for i in 0..9 {
        let (a, b) = (FielderAttrs::average(i), FielderAttrs::average(i).with_throwing(50.0, 50.0, 50.0));
        assert!((a.arm - b.arm).abs() < 1e-12 && (a.arm_sigma - b.arm_sigma).abs() < 1e-12 && (a.transfer - b.transfer).abs() < 1e-12);
    }
}
