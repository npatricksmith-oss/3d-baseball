//! Per-component timing for one pitch (agent-models §12 budget).
use bb_agents::batter::{Bank, eye_point};
use bb_agents::pitcher::{PitchDef, PitchIntent, plan_release};
use bb_game::cage::Cage;
use bb_game::pa::{Matchup, simulate_pitch};
use bb_math::Vec3;
use bb_observe::{BallObs, Count, PitchType, ReleaseObs};
use bb_physics::flight::{Crossing, DT_PITCH, fly};
use bb_physics::pitch::launch;
use std::hint::black_box;
use std::time::Instant;

fn t<F: FnMut()>(name: &str, n: u32, mut f: F) {
    let s = Instant::now();
    for _ in 0..n {
        f();
    }
    println!("{name:28} {:8.1} µs", s.elapsed().as_secs_f64() * 1e6 / n as f64);
}

fn main() {
    let mu = Matchup::average();
    let def = PitchDef::template(PitchType::Slider);
    let intent = PitchIntent { ty: PitchType::Slider, target_x: 0.1, target_z: 0.6 };
    t("plan_release (aim solve)", 2000, || { black_box(plan_release(&mu.env, &mu.pitcher, &def, &intent).unwrap()); });
    let plan = plan_release(&mu.env, &mu.pitcher, &def, &intent).unwrap();
    let s0 = launch(plan.r, &plan.shape, plan.yaw, plan.elev);
    t("pitch flight", 5000, || { black_box(fly(&mu.env, s0, 0.0, DT_PITCH, &[Crossing::YBelow(-1.2)], 3.0)); });
    let eye = eye_point(mu.batter.bats, mu.batter.height);
    let prior = vec![0.5, 0.25, 0.13, 0.12];
    t("bank: new + 25 observations", 2000, || {
        let mut b = Bank::new(&mu.scouting, &prior, &ReleaseObs { t: 0.0, r: plan.r }, eye, 0.06);
        for i in 0..25 {
            let o = BallObs { t: 0.01 * (i + 1) as f64, az: 0.05, el: 0.02, sigma_ang: 0.002, tau: 0.3, sigma_tau: 0.03, spin_cue: [0.0, 0.5, 0.9], sigma_spin: 0.5 };
            b.observe(&mu.scouting, &o);
        }
        black_box(b.posterior());
    });
    let cage = Cage::average();
    let rel = cage.release(&intent);
    t("swing + contact + batted flight", 2000, || { black_box(cage.swing_with(rel, &Default::default())); });
    t("whole pitch (simulate_pitch)", 2000, || { black_box(simulate_pitch(&mu, &mu.scouting, Count::default(), 1, 7)); });
    let _ = Vec3::ZERO;
}
