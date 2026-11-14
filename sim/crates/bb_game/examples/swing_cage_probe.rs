//! The v2 swing in the cage (agent-models §3.6): perfect perception, a four-seamer at three
//! horizontal locations, on time and with timing shifts. Prints the sweet spot's speed, attack
//! direction and angle at contact, and the batted ball (exit speed, launch, spray,
//! batter-relative: negative = pulled). `cargo run --release -p bb_game --example swing_cage_probe`
use bb_agents::pitcher::PitchIntent;
use bb_agents::swing::Placement;
use bb_game::cage::Cage;
use bb_observe::PitchType;
use bb_physics::contact::BatPose;
use bb_world::pitch::SwingResult;
fn main() {
    let cage = Cage::average();
    let uss = cage.bat.profile.length() - cage.bat.contact.ss_from_tip;
    println!("  x (m)  dt ms |   bat mph   AD°   AA° |  EV mph   LA°  spray° | station from tip (in)");
    for x in [-0.20, -0.10, 0.0, 0.10, 0.20] {
        let rel = cage.release(&PitchIntent { ty: PitchType::FourSeam, target_x: x, target_z: 0.75 });
        for dt in [-0.012, -0.006, 0.0, 0.006, 0.012] {
            let p = Placement { t_shift: dt, ..Default::default() };
            let plan = cage.plan_perfect(rel, None).unwrap();
            let sw = bb_agents::swing::apply_placement(&plan, &p);
            let r = cage.swing_with(rel, &p);
            match r.swing {
                SwingResult::Contact(b) => {
                    let k = sw.at(b.contact.point.t);
                    let pnt = k.point(uss);
                    let v = k.point_velocity(pnt, cage.bat.mass.u_cm);
                    let ad = bb_math::m::atan2(-v.x, v.y).to_degrees();
                    let aa = bb_math::m::atan2(v.z, v.x.hypot(v.y)).to_degrees();
                    let u = (b.contact.point.ball.r - k.knob).dot(k.axis);
                    println!("{x:+.2}  {:+5.0} | {:7.1} {:5.1} {:5.1} | {:6.1} {:5.1} {:6.1} | {:5.1}", dt * 1000.0, v.norm() / 0.44704, ad, aa, b.exit_speed / 0.44704, b.launch_angle.to_degrees(), b.spray_angle.to_degrees(), (cage.bat.profile.length() - u) / 0.0254);
                }
                _ => println!("{x:+.2}  {:+5.0} | miss", dt * 1000.0),
            }
        }
    }
}
