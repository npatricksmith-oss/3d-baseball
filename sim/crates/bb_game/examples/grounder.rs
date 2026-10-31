//! Trace one ground ball: path samples and who fields it.
use bb_game::play::{FielderAttrs, PlayInput, RunnerIn, home_positions, simulate_play};
use bb_math::units::{DEG, MPH, RPM};
use bb_math::{Vec3, m};
use bb_physics::ground::Field;
use bb_physics::{BallState, FlightEnv};

fn main() {
    let env = FlightEnv::default_env();
    let field = Field::default();
    let fattrs: [FielderAttrs; 9] = core::array::from_fn(FielderAttrs::average);
    let fielders: [u16; 9] = core::array::from_fn(|i| 0x0180 | (i as u16 + 1));
    let (l, s) = (0.0 * DEG, -15.0 * DEG);
    let v = Vec3::new(m::sin(s) * m::cos(l), m::cos(s) * m::cos(l), m::sin(l)) * (90.0 * MPH);
    let ball = BallState { r: Vec3::new(0.0, 0.4, 0.9), v, w: Vec3::new(m::cos(s), -m::sin(s), 0.0) * (-800.0 * RPM) };
    let out = simulate_play(&PlayInput { env: &env, field: &field, ball, t_contact: 0.45, runners: vec![RunnerIn { id: 1, base: 0, speed: 8.2 }], outs: 0, fielders, fattrs, seed: 1, play: 5 });
    for t in [0.6, 0.8, 1.0, 1.2, 1.5, 2.0, 3.0] {
        let (p, v) = out.path.at(t);
        println!("t {t:.1}: ball ({:6.1},{:6.1},{:4.2}) speed {:4.1}", p.x, p.y, p.z, v.norm());
    }
    let hp = home_positions();
    println!("SS home {:?}, 3B home {:?}", hp[5], hp[4]);
    for f in &out.facts {
        println!("{f:?}");
    }
}
