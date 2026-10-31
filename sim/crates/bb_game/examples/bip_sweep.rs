//! Diagnostics: outcome of synthetic batted balls by launch angle, bases empty.
use bb_game::play::{FielderAttrs, PlayInput, RunnerIn, simulate_play};
use bb_math::units::{DEG, MPH, RPM};
use bb_math::{Vec3, m};
use bb_physics::ground::Field;
use bb_physics::{BallState, FlightEnv};
use bb_rules::{Event, Input, RulesState, step};

fn main() {
    let env = FlightEnv::default_env();
    let field = Field::default();
    let fattrs: [FielderAttrs; 9] = core::array::from_fn(FielderAttrs::average);
    let fielders: [u16; 9] = core::array::from_fn(|i| 0x0180 | (i as u16 + 1));
    for la in [-10.0, 0.0, 8.0, 15.0, 22.0, 30.0, 40.0, 55.0] {
        let (mut outs, mut n, mut secured, mut caught) = (0, 0, 0, 0);
        let mut tsec = 0.0;
        for sp in [-35.0, -25.0, -15.0, -5.0, 5.0, 15.0, 25.0, 35.0] {
            for ev in [80.0, 90.0, 100.0] {
                let (l, s) = (la * DEG, sp * DEG);
                let v = Vec3::new(m::sin(s) * m::cos(l), m::cos(s) * m::cos(l), m::sin(l)) * (ev * MPH);
                let w = Vec3::new(m::cos(s), -m::sin(s), 0.0) * (if la > 5.0 { 2000.0 } else { -800.0 } * RPM);
                let ball = BallState { r: Vec3::new(0.0, 0.4, 0.9), v, w };
                let mut st = RulesState::default_fixture();
                step(&mut st, &Input::StartGame);
                step(&mut st, &Input::PitchDelivered { t: 0.0 });
                step(&mut st, &Input::BatContact { t: 0.45 });
                let out = simulate_play(&PlayInput { env: &env, field: &field, ball, t_contact: 0.45, runners: vec![RunnerIn { id: 1, base: 0, speed: 8.2 }], outs: 0, fielders, fattrs, seed: 1, play: (la * 100.0 + sp * 10.0 + ev) as u32 });
                let mut evs = Vec::new();
                for f in &out.facts {
                    if let Input::GloveSecured { t, .. } = f { secured += 1; tsec += t; }
                    evs.extend(step(&mut st, f));
                }
                if evs.iter().any(|e| matches!(e, Event::Catch { .. })) { caught += 1; }
                n += 1;
                if evs.iter().any(|e| matches!(e, Event::RunnerOut { runner: 1, .. })) { outs += 1; }
            }
        }
        println!("LA {la:5.0}: out {outs:2}/{n}  secured {secured}  caught-fly {caught}  mean t_secure {:.2}", tsec / secured.max(1) as f64);
    }
}
