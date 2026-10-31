//! Release execution noise (agent-models §1.3, §2.3; Codex review #9).

use bb_agents::motor::{ReleaseNoise, TRUNC_SIGMA, execute_release};
use bb_agents::pitcher::{PitchDef, PitchIntent, PitcherAttrs, plan_release};
use bb_math::{KeyedRng, Purpose, RngKey};
use bb_observe::PitchType;
use bb_physics::FlightEnv;
use bb_physics::pitch::launch;

fn rng(play: u32) -> KeyedRng {
    KeyedRng::new(RngKey { game_seed: 99, play_index: play, agent_id: 0x0100, purpose: Purpose::PitchRelease, sub_index: 0 })
}

fn plan() -> (bb_agents::pitcher::IntendedRelease, PitcherAttrs) {
    let env = FlightEnv::default_env();
    let p = PitcherAttrs::average_rhp();
    let def = PitchDef::template(PitchType::Slider);
    (plan_release(&env, &p, &def, &PitchIntent { ty: PitchType::Slider, target_x: 0.1, target_z: 0.6 }).unwrap(), p)
}

#[test]
fn zero_noise_is_identity() {
    let (pl, _) = plan();
    let z = ReleaseNoise { angle: 0.0, speed: 0.0, spin_rate_rel: 0.0, spin_axis_deg: 0.0, point: 0.0 };
    let ex = execute_release(&pl, &z, &mut rng(0));
    assert_eq!(ex.state, launch(pl.r, &pl.shape, pl.yaw, pl.elev));
}

#[test]
fn constructor_scales_command() {
    let (pl, p) = plan();
    let n = ReleaseNoise::new(p.command, pl.command_mult, 1.3);
    assert!((n.angle - p.command * 1.15 * 1.3).abs() < 1e-15);
    assert_eq!((n.speed, n.spin_rate_rel, n.spin_axis_deg, n.point), (0.45, 0.02, 3.0, 0.02));
}

#[test]
fn per_component_scales_bounds_and_draw_order() {
    let (pl, p) = plan();
    let noise = ReleaseNoise::new(p.command, pl.command_mult, 1.0);
    let n = 20_000;
    let mut sums = [0.0f64; 7];
    for i in 0..n {
        let ex = execute_release(&pl, &noise, &mut rng(i));
        let d = [
            (ex.yaw - pl.yaw) / noise.angle,
            (ex.elev - pl.elev) / noise.angle,
            (ex.shape.speed - pl.shape.speed) / noise.speed,
            (ex.shape.spin_rate / pl.shape.spin_rate - 1.0) / noise.spin_rate_rel,
            (ex.shape.spin_axis_deg - pl.shape.spin_axis_deg) / noise.spin_axis_deg,
            (ex.state.r.x - pl.r.x) / noise.point,
            (ex.state.r.z - pl.r.z) / noise.point,
        ];
        // Draw order is the replay contract: component k uses the k-th truncated normal.
        if i == 0 {
            let mut r = rng(0);
            for (k, dk) in d.iter().enumerate() {
                let z = r.truncated_normal(TRUNC_SIGMA);
                assert!((dk - z).abs() < 1e-9, "component {k}: {dk} vs draw {z}");
            }
        }
        for (k, dk) in d.iter().enumerate() {
            assert!(dk.abs() <= TRUNC_SIGMA + 1e-9, "component {k} beyond 4σ: {dk}");
            sums[k] += dk * dk;
        }
    }
    for (k, s) in sums.iter().enumerate() {
        let sd = (s / n as f64).sqrt();
        assert!((sd - 1.0).abs() < 0.03, "component {k}: unit-σ estimate {sd}");
    }
}
