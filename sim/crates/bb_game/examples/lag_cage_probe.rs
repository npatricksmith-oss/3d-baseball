//! The bat's lag at contact λ_c in the cage (docs/calibration/2026-11-14-bat-angle-at-contact.md):
//! perfect perception, on time, an average RHP four-seamer at middle height against an average
//! RHB at five horizontal locations, with the swing met at the ball's center and at the
//! game's aim (2 cm under it). All directions pull-positive, degrees.
//!
//! Part 1: a sweep of λ_c. Part 2: where the opposite-field offset at λ_c = 0 comes from,
//! by re-resolving one contact with one thing changed at a time.
//! `cargo run --release -p bb_game --example lag_cage_probe`
use bb_agents::pitcher::PitchIntent;
use bb_agents::swing::{Placement, apply_placement};
use bb_game::cage::Cage;
use bb_math::Vec3;
use bb_math::units::DEG;
use bb_observe::PitchType;
use bb_physics::contact::{BatPose, ContactPoint, Detection, detect, resolve_contact};
use bb_physics::flight::{Crossing, DT_BATTED, fly};
use bb_world::pitch::pitch_flight;

const MPH: f64 = 1.0 / 0.44704;
/// Pull-positive azimuth of a horizontal direction for a right-handed batter (pull = −X).
fn az(v: Vec3) -> f64 {
    (-v.x).atan2(v.y) / DEG
}

struct Out {
    spray_land: f64,
    spray_launch: f64,
    ev: f64,
    la: f64,
}

fn outcome(cage: &Cage, cp: &ContactPoint) -> Out {
    let r = resolve_contact(cp, &cage.bat.profile, &cage.bat.mass, &cage.bat.contact, &cage.env.ball);
    let s = r.ball_after;
    let f = fly(&cage.env, s, cp.t, DT_BATTED, &[Crossing::ZBelow(0.0)], 15.0);
    let land = f.last().s.r;
    Out { spray_land: az(land), spray_launch: az(s.v), ev: s.v.norm() * MPH, la: s.v.z.atan2(s.v.x.hypot(s.v.y)) / DEG }
}

fn main() {
    let xs_in = [-0.20, -0.10, 0.0, 0.10, 0.20]; // m inside of the plate's center (RHB: x = −x_in)
    let names = ["outside", "mid-away", "middle", "mid-in", "inside"];
    let base = Cage::average();
    let uss = base.bat.profile.length() - base.bat.contact.ss_from_tip;
    println!("Part 1. On time; spray by first landing (launch direction in brackets); AD and face at contact; length = sweet-spot path over the last 150 ms.");
    for d in [0.0, -0.02] {
        println!("\n== swing met {} cm under the ball's center ==", -100.0 * d);
        println!("λ_c° loc       | spray (launch) |   AD  face−AD | spray−AD |  EV mph   LA° | bat mph  length ft");
        for lag in [0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 15.0] {
            let mut cage = Cage::average();
            cage.template.lag_c = lag * DEG;
            for (i, xi) in xs_in.iter().enumerate() {
                let rel = cage.release(&PitchIntent { ty: PitchType::FourSeam, target_x: -xi, target_z: 0.75 });
                let plan = cage.plan_perfect(rel, None).unwrap();
                let sw = apply_placement(&plan, &Placement { v_offset: d, ..Default::default() });
                let tr = pitch_flight(&cage.env, rel);
                let Detection::Contact(cp) = detect(&tr, &sw, &cage.bat.profile, &cage.env.ball) else {
                    println!("{lag:4.0} {:9} | miss", names[i]);
                    continue;
                };
                let k = sw.at(cp.t);
                let v = k.point_velocity(k.point(uss), cage.bat.mass.u_cm);
                let ad = az(v);
                // Face: the direction the bat's rotation alone would move the sweet spot (⊥ to the axis, in the plane).
                let face = az(sw.n.cross(k.axis));
                let o = outcome(&cage, &cp);
                let steps = 300;
                let len: f64 = (0..steps)
                    .map(|j| {
                        let tt = cp.t - 0.150 + (j as f64 + 0.5) * 0.150 / steps as f64;
                        let kk = sw.at(tt);
                        kk.point_velocity(kk.point(uss), cage.bat.mass.u_cm).norm() * 0.150 / steps as f64
                    })
                    .sum();
                println!(
                    "{lag:4.0} {:9} | {:+6.1} ({:+5.1}) | {:+5.1}  {:+5.1}  | {:+6.1}   | {:6.1} {:5.1} | {:6.1}  {:5.2}",
                    names[i],
                    o.spray_land,
                    o.spray_launch,
                    ad,
                    face - ad,
                    o.spray_land - ad,
                    o.ev,
                    o.la,
                    v.norm() * MPH,
                    len / 0.3048
                );
            }
        }
    }

    // Part 2: decomposition at λ_c = 0, middle, met 0 and 2 cm under the ball.
    println!("\nPart 2. Decomposition at λ_c = 0 (one change at a time from the actual contact; spray by landing, launch in brackets).");
    for (i, xi) in xs_in.iter().enumerate() {
        for d in [0.0, -0.02] {
            let rel = base.release(&PitchIntent { ty: PitchType::FourSeam, target_x: -xi, target_z: 0.75 });
            let plan = base.plan_perfect(rel, None).unwrap();
            let sw = apply_placement(&plan, &Placement { v_offset: d, ..Default::default() });
            let tr = pitch_flight(&base.env, rel);
            let Detection::Contact(cp) = detect(&tr, &sw, &base.bat.profile, &base.env.ball) else { continue };
            let k = sw.at(cp.t);
            let v = k.point_velocity(k.point(uss), base.bat.mass.u_cm);
            let ad = az(v);
            let pitch_az = az(-cp.ball.v); // where the pitch comes from, as a direction it would be sent back along
            let o0 = outcome(&base, &cp);
            // (a) The pitch's incoming lateral angle removed (same speed, straight down the −Y axis in plan).
            let mut a = cp;
            let vh = a.ball.v.x.hypot(a.ball.v.y);
            a.ball.v = Vec3::new(0.0, -vh, a.ball.v.z);
            let oa = outcome(&base, &a);
            // (b) The pitch's spin removed.
            let mut b = cp;
            b.ball.w = Vec3::ZERO;
            let ob = outcome(&base, &b);
            // (c) The bat made level: rotate the bat (pose, velocity, spin) about the ball's center
            // so its axis is horizontal, keeping its horizontal direction and the contact geometry.
            let ah = Vec3::new(cp.bat.axis.x, cp.bat.axis.y, 0.0).normalized();
            let kx = cp.bat.axis.cross(ah);
            let ang = kx.norm().atan2(cp.bat.axis.dot(ah));
            let kx = kx.normalized();
            let rot = |p: Vec3| p.rotate(kx, ang);
            let mut c = cp;
            c.n = rot(cp.n);
            c.bat.axis = rot(cp.bat.axis);
            c.bat.knob = cp.ball.r + rot(cp.bat.knob - cp.ball.r);
            c.bat.v_cm = rot(cp.bat.v_cm);
            c.bat.omega = rot(cp.bat.omega);
            let oc = outcome(&base, &c);
            let ad_c = az(c.bat.point_velocity(c.bat.point(uss), base.bat.mass.u_cm));
            // (d) The contact normal's horizontal direction set to the face's (the round barrel's
            // up/down offset no longer tips the normal sideways); the bat level as in (c).
            let mut e = c;
            let face_c = base.template.lag_c; // 0: face ⊥ to the axis in the plane
            let _ = face_c;
            let f = sw.n.cross(cp.bat.axis);
            let f = rot(f);
            // Keep the normal's elevation, give it the face's azimuth.
            let nz = e.n.z;
            let fh = Vec3::new(f.x, f.y, 0.0).normalized();
            e.n = (fh * (1.0 - nz * nz).max(0.0).sqrt() + Vec3::new(0.0, 0.0, nz)).normalized();
            let oe = outcome(&base, &e);
            // (e) The attack direction a few ms before contact: the sweet spot's path turns about
            // 1.8° per ms, so a direction measured before the intercept reads more opposite-field.
            let ad_before: Vec<f64> = [1.7e-3, 3.3e-3, 5.0e-3]
                .iter()
                .map(|dt| {
                    let kk = sw.at(cp.t - dt);
                    az(kk.point_velocity(kk.point(uss), base.bat.mass.u_cm))
                })
                .collect();
            println!("{:9} {:.0} cm under | AD 1.7 / 3.3 / 5.0 ms before contact {:+5.1} / {:+5.1} / {:+5.1}", names[i], -100.0 * d, ad_before[0], ad_before[1], ad_before[2]);
            println!(
                "{:9} {:.0} cm under | AD {:+5.1} normal {:+5.1} pitch-from {:+5.1} | actual {:+6.1} ({:+5.1}) LA {:5.1} | (a) no lateral {:+6.1} ({:+5.1}) | (b) no pitch spin {:+6.1} ({:+5.1}) | (c) level bat {:+6.1} ({:+5.1}) AD {:+5.1} normal {:+5.1} | (d) + normal at face {:+6.1} ({:+5.1})",
                names[i], -100.0 * d, ad, az(cp.n), pitch_az, o0.spray_land, o0.spray_launch, o0.la, oa.spray_land, oa.spray_launch, ob.spray_land, ob.spray_launch, oc.spray_land, oc.spray_launch, ad_c, az(c.n), oe.spray_land, oe.spray_launch
            );
        }
    }
}
