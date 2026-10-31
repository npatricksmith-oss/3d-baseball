//! M1 exit gate: a fastball and a slider share early flight (PLAN §14).
//! A slider aimed through the fastball's tunnel point stays close to it before that point
//! and separates sharply after it.

use bb_agents::pitcher::{PitchDef, PitchIntent, PitcherAttrs, plan_release};
use bb_math::units::{FT, m_to_in};
use bb_observe::PitchType;
use bb_physics::FlightEnv;
use bb_physics::flight::{Crossing, Trajectory};
use bb_physics::pitch::{Y_TUNNEL, aim_at_plane, fly_pitch, launch};

fn at_y(tr: &Trajectory, y: f64) -> bb_math::Vec3 {
    tr.at(tr.find(Crossing::YBelow(y)).unwrap()).0
}

#[test]
fn fastball_and_slider_tunnel() {
    let env = FlightEnv::default_env();
    let p = PitcherAttrs::average_rhp();
    let ff = PitchDef::template(PitchType::FourSeam);
    let plan = plan_release(&env, &p, &ff, &PitchIntent { ty: PitchType::FourSeam, target_x: 0.15, target_z: 0.95 });
    let tr_ff = fly_pitch(&env, launch(plan.r, &plan.shape, plan.yaw, plan.elev));
    let tun = at_y(&tr_ff, Y_TUNNEL);
    let sl = PitchDef::template(PitchType::Slider);
    let (r2, shape2) = (sl.release_point(&p), sl.shape(&p));
    let a = aim_at_plane(&env, r2, &shape2, Y_TUNNEL, tun.x, tun.z, None);
    let tr_sl = fly_pitch(&env, launch(r2, &shape2, a.yaw, a.elev));
    let sep = |ft: f64| m_to_in((at_y(&tr_ff, ft * FT) - at_y(&tr_sl, ft * FT)).norm());
    let early = [50.0, 40.0, 30.0].map(sep);
    let plate = sep(17.0 / 12.0);
    assert!(early.iter().all(|&s| s < 3.0), "early separation {early:?} in");
    assert!(plate > 8.0, "plate separation {plate} in");
}
