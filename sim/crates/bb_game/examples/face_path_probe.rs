//! The bat's face ahead of its path, revisited (docs/calibration/2026-11-14-collision-sloped-barrel.md
//! §7.2): the lag at contact λ_c swept against the hands' speed at contact (set through the
//! hands' rate ratio q, solved per case so the knob moves at the target speed at t_c; bat speed
//! stays at `BatSpeed` by construction). Cage, average RHP four-seamer at the middle, on time,
//! squared up and 2 cm under. Pull-positive degrees. Diagnostic only: defaults unchanged.
//! `cargo run --release -p bb_game --example face_path_probe`
use bb_agents::pitcher::PitchIntent;
use bb_agents::swing::{IntendedSwing, Placement, apply_placement};
use bb_game::cage::Cage;
use bb_math::Vec3;
use bb_math::units::DEG;
use bb_observe::PitchType;
use bb_physics::contact::{BatPose, Detection, detect, resolve_contact};
use bb_physics::flight::{Crossing, DT_BATTED, fly};
use bb_world::pitch::pitch_flight;

const MPH: f64 = 1.0 / 0.44704;

fn az(v: Vec3) -> f64 {
    (-v.x).atan2(v.y) / DEG
}

fn main() {
    let base = Cage::average();
    let uss = base.bat.profile.length() - base.bat.contact.ss_from_tip;
    let u_cm = base.bat.mass.u_cm;
    let rel = base.release(&PitchIntent { ty: PitchType::FourSeam, target_x: 0.0, target_z: 0.75 });
    let tr = pitch_flight(&base.env, rel);
    let plan_for = |lag: f64, q: f64| -> (Cage, IntendedSwing) {
        let mut c = Cage::average();
        c.template.lag_c = lag * DEG;
        c.template.hand_rate_ratio = q;
        let p = c.plan_perfect(rel, None).unwrap();
        (c, p)
    };
    let hand_speed = |p: &IntendedSwing| {
        let k = p.swing.at(p.swing.t_c);
        k.point_velocity(k.knob, u_cm).norm() * MPH
    };
    println!("Default: λ_c 0, q 0.32 → hands {:.1} mph at contact.", hand_speed(&plan_for(0.0, 0.32).1));
    println!("\nλ_c°  hands mph   q    | length ft | planned AD, actual AD, face − AD | AA actual | under | bat mph  EV mph  LA° | spray − AD: launch / landing");
    for lag in [0.0, 15.0, 30.0, 45.0, 60.0] {
        for target in [14.0, 18.0, 21.0, 24.0] {
            // Bisect q so the knob moves at the target speed at t_c.
            let (mut lo, mut hi) = (0.02, 2.5);
            for _ in 0..60 {
                let mid = 0.5 * (lo + hi);
                if hand_speed(&plan_for(lag, mid).1) < target { lo = mid } else { hi = mid }
            }
            let q = 0.5 * (lo + hi);
            let (cage, plan) = plan_for(lag, q);
            let sw0 = &plan.swing;
            let steps = 300;
            let len: f64 = (0..steps)
                .map(|j| {
                    let tt = sw0.t_c - 0.150 + (j as f64 + 0.5) * 0.150 / steps as f64;
                    let kk = sw0.at(tt);
                    kk.point_velocity(kk.point(uss), u_cm).norm() * 0.150 / steps as f64
                })
                .sum::<f64>()
                / 0.3048;
            let planned_ad = plan.geometry.direction / DEG;
            for d in [0.0, -0.02] {
                let sw = apply_placement(&plan, &Placement { v_offset: d, ..Default::default() });
                let Detection::Contact(cp) = detect(&tr, &sw, &cage.bat.profile, &cage.env.ball) else {
                    println!("{lag:4.0}  {target:5.1}  miss");
                    continue;
                };
                let k = sw.at(cp.t);
                let v = k.point_velocity(k.point(uss), u_cm);
                let ad = az(v);
                let aa = v.z.atan2(v.x.hypot(v.y)) / DEG;
                let face = az(sw.n.cross(k.axis));
                let r = resolve_contact(&cp, &cage.bat.profile, &cage.bat.mass, &cage.bat.contact, &cage.env.ball);
                let s = r.ball_after;
                let f = fly(&cage.env, s, cp.t, DT_BATTED, &[Crossing::ZBelow(0.0)], 15.0);
                let land = f.last().s.r;
                println!(
                    "{lag:4.0}  {target:5.1}  {q:5.3}  |   {len:5.2}   |   {planned_ad:+5.1}  {ad:+6.1}   {:+6.1}     |  {aa:5.1}    | {:3.0} cm | {:6.1}  {:6.1} {:5.1} | {:+6.1} / {:+6.1}",
                    face - ad,
                    -100.0 * d,
                    v.norm() * MPH,
                    s.v.norm() * MPH,
                    s.v.z.atan2(s.v.x.hypot(s.v.y)) / DEG,
                    az(s.v) - ad,
                    az(land) - ad
                );
            }
        }
    }
}
