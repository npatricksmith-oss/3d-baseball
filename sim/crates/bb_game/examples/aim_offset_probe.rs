//! The batted ball against the swing's vertical offset from the ball's center, in the cage
//! (perfect perception, on time, average RHP four-seamer and curveball at three heights over
//! the middle). Answers: what offset under the ball gives which launch angle and exit speed,
//! and where the barrel window (Statcast: 98+ mph at 26–30°) lies.
//! `cargo run --release -p bb_game --example aim_offset_probe`
use bb_agents::pitcher::PitchIntent;
use bb_agents::swing::Placement;
use bb_game::cage::Cage;
use bb_observe::PitchType;
use bb_world::pitch::SwingResult;
fn main() {
    let cage = Cage::average();
    for ty in [PitchType::FourSeam, PitchType::Curveball] {
        for z in [0.55, 0.75, 0.95] {
            println!("\n{ty:?} at z {z:.2} m, x 0, on time: offset (cm, − = bat under the ball) → EV mph, LA°, carry ft, spin rpm | carry at 2500 rpm");
            let rel = cage.release(&PitchIntent { ty, target_x: 0.0, target_z: z });
            for k in 0..=12 {
                let off = -0.005 * k as f64;
                let p = Placement { v_offset: off, ..Default::default() };
                match cage.swing_with(rel, &p).swing {
                    SwingResult::Contact(b) => {
                        let ev = b.exit_speed / 0.44704;
                        let la = b.launch_angle.to_degrees();
                        let dist = b.projected_distance / 0.3048;
                        let spin = b.contact.ball_after.w.norm() * 60.0 / (2.0 * std::f64::consts::PI);
                        // The same launch with its spin scaled to 2,500 rpm (Nathan's A1 case), for scale only.
                        let mut s = b.contact.ball_after;
                        s.w = s.w * (2500.0 / spin);
                        let f = bb_physics::flight::fly(&cage.env, s, 0.0, bb_physics::flight::DT_BATTED, &[bb_physics::flight::Crossing::ZBelow(0.0)], 15.0);
                        let e = f.last();
                        let d2500 = e.s.r.x.hypot(e.s.r.y) / 0.3048;
                        println!("{:6.1} | {:6.1} {:6.1} {:6.0} {:6.0} | {:6.0}", off * 100.0, ev, la, dist, spin, d2500);
                    }
                    _ => println!("{:6.1} | miss", off * 100.0),
                }
            }
        }
    }
}
