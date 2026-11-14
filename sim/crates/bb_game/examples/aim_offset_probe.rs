//! The batted ball against the swing's vertical offset from the ball's center, in the cage
//! (perfect perception, on time, average RHP four-seamer and curveball at three heights over
//! the middle). Answers: what offset under the ball gives which launch angle and exit speed,
//! and where the barrel window (Statcast: 98+ mph at 26–30°) lies.
//!
//! The spin is decomposed along the exit velocity v (physics-spec §1.3: a batted ball moving
//! +Y with backspin has ω ∥ +X): gyro g = v̂ (spin about the direction of travel), backspin
//! axis b = normalize(v.y, −v.x, 0) (horizontal, ⊥ to v; +X for a ball moving +Y; positive =
//! backspin), and sidespin axis s = g × b (⊥ to both; for a ball moving +Y it is −Z, so a
//! positive sidespin is a clockwise-from-above, "hooking to the pull side for a right-handed
//! batter" … see the sign note in the report). Components are ω·g, ω·b, ω·s, in rpm.
//!
//! Carry (ft from the plate to the first z = 0, no wall, same flight model) is printed for the
//! ball as hit and for counterfactuals that change only the spin: the whole vector scaled to
//! 2,500 rpm (the older probe's "carry at 2,500"), the backspin component alone set to 2,500
//! rpm with the sidespin and gyro components kept, the backspin kept with sidespin and gyro
//! zeroed, and, to attribute the loss, sidespin alone zeroed and gyro alone zeroed.
//! `cargo run --release -p bb_game --example aim_offset_probe`
use bb_agents::pitcher::PitchIntent;
use bb_agents::swing::Placement;
use bb_game::cage::Cage;
use bb_math::Vec3;
use bb_observe::PitchType;
use bb_physics::BallState;
use bb_physics::flight::{Crossing, DT_BATTED, fly};
use bb_world::pitch::SwingResult;

const RPM: f64 = 60.0 / (2.0 * std::f64::consts::PI);

fn carry_ft(cage: &Cage, s: BallState, t0: f64) -> f64 {
    let f = fly(&cage.env, s, t0, DT_BATTED, &[Crossing::ZBelow(0.0)], 15.0);
    let e = f.last();
    e.s.r.x.hypot(e.s.r.y) / 0.3048
}

fn main() {
    let cage = Cage::average();
    for ty in [PitchType::FourSeam, PitchType::Curveball] {
        for z in [0.55, 0.75, 0.95] {
            println!("\n{ty:?} at z {z:.2} m, x 0, on time. offset cm (− = bat under the ball) | EV mph, LA°, spray° (world: + = toward first base / the right-handed batter's opposite field), carry ft | spin rpm: total, back, side, gyro, ω_z (about the vertical) | carry ft with: whole vector at 2,500, back alone 2,500, side+gyro zeroed, side zeroed, gyro zeroed");
            let rel = cage.release(&PitchIntent { ty, target_x: 0.0, target_z: z });
            for k in 0..=12 {
                let off = -0.005 * k as f64;
                let p = Placement { v_offset: off, ..Default::default() };
                match cage.swing_with(rel, &p).swing {
                    SwingResult::Contact(b) => {
                        let ev = b.exit_speed / 0.44704;
                        let la = b.launch_angle.to_degrees();
                        let s0 = b.contact.ball_after;
                        let t0 = b.contact.point.t;
                        let g = s0.v.normalized();
                        let back_axis = Vec3::new(s0.v.y, -s0.v.x, 0.0).normalized();
                        let side_axis = g.cross(back_axis);
                        let (wg, wb, ws) = (s0.w.dot(g), s0.w.dot(back_axis), s0.w.dot(side_axis));
                        let total = s0.w.norm();
                        let with = |w: Vec3| carry_ft(&cage, BallState { w, ..s0 }, t0);
                        let c_hit = with(s0.w);
                        let c_whole = with(s0.w * (2500.0 / (total * RPM)));
                        let c_back = with(s0.w + back_axis * (2500.0 / RPM - wb));
                        let c_nosg = with(back_axis * wb);
                        let c_noside = with(s0.w - side_axis * ws);
                        let c_nogyro = with(s0.w - g * wg);
                        println!(
                            "{:6.1} | {:6.1} {:6.1} {:6.1} {:6.1} | {:6.0} {:6.0} {:6.0} {:6.0} {:6.0} | {:6.1} {:6.1} {:6.1} {:6.1} {:6.1}   (probe flight {:.1}, world {:.1})",
                            off * 100.0, ev, la, b.spray_angle.to_degrees(), c_hit, total * RPM, wb * RPM, ws * RPM, wg * RPM, s0.w.z * RPM, c_whole, c_back, c_nosg, c_noside, c_nogyro, c_hit, b.projected_distance / 0.3048
                        );
                    }
                    _ => println!("{:6.1} | miss", off * 100.0),
                }
            }
        }
    }
}
