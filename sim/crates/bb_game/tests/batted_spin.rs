//! physics-spec §14.2 A10b and A10c: batted spin at game speed against Statcast, as published
//! by Nathan ("Why Does a Fly Ball Carry Better to Centerfield?", 2020). Trackman measures total
//! spin and infers its axis, so both components are derived; the bands are wide accordingly.
//! A10d (sidespin zero near −10° of adjusted spray) is reported only: it belongs to the open
//! direction question (§17 item 8).

use bb_agents::pitcher::PitchIntent;
use bb_agents::swing::{Placement, apply_placement};
use bb_game::cage::Cage;
use bb_math::Vec3;
use bb_observe::PitchType;
use bb_physics::contact::{Detection, detect, resolve_contact};
use bb_physics::flight::{Crossing, DT_BATTED, fly};
use bb_world::pitch::pitch_flight;

const RPM: f64 = 60.0 / (2.0 * std::f64::consts::PI);
const MPH: f64 = 1.0 / 0.44704;

/// Fly balls at 25–30° launch and 90+ mph from an average RHP four-seamer at the middle, the
/// swing timed −15…+15 ms and met 1.5–4.5 cm under the ball: (adjusted spray °, + = opposite
/// field for this right-handed batter; sidespin rpm, + = breaking toward the left-field line;
/// backspin rpm).
fn fly_balls() -> Vec<(f64, f64, f64)> {
    let cage = Cage::average();
    let rel = cage.release(&PitchIntent { ty: PitchType::FourSeam, target_x: 0.0, target_z: 0.75 });
    let plan = cage.plan_perfect(rel, None).unwrap();
    let tr = pitch_flight(&cage.env, rel);
    let mut pts = Vec::new();
    for it in -10..=10 {
        for k in 0..=24 {
            let p = Placement { v_offset: -0.015 - 0.00125 * k as f64, t_shift: it as f64 * 0.0015, ..Default::default() };
            let sw = apply_placement(&plan, &p);
            let Detection::Contact(cp) = detect(&tr, &sw, &cage.bat.profile, &cage.env.ball) else { continue };
            let s = resolve_contact(&cp, &cage.bat.profile, &cage.bat.mass, &cage.bat.contact, &cage.env.ball).ball_after;
            let la = s.v.z.atan2(s.v.x.hypot(s.v.y)).to_degrees();
            if !(25.0..30.0).contains(&la) || s.v.norm() * MPH < 90.0 {
                continue;
            }
            let land = fly(&cage.env, s, cp.t, DT_BATTED, &[Crossing::ZBelow(0.0)], 15.0).last().s.r;
            // Backspin about the horizontal axis ⊥ to v (+X for a ball moving +Y); sidespin
            // about v̂ × that axis (+ = breaking toward +X, the right-field line).
            let vh = s.v.normalized();
            let b = Vec3::new(s.v.y, -s.v.x, 0.0).normalized();
            let spray_pull = (-land.x).atan2(land.y).to_degrees();
            pts.push((-spray_pull, -s.w.dot(vh.cross(b)) * RPM, s.w.dot(b) * RPM));
        }
    }
    pts
}

#[test]
fn a10b_a10c_batted_spin_at_game_speed() {
    let pts = fly_balls();
    assert!(pts.len() >= 40, "only {} fly balls in the sweep", pts.len());
    let n = pts.len() as f64;
    let (mx, my) = (pts.iter().map(|p| p.0).sum::<f64>() / n, pts.iter().map(|p| p.1).sum::<f64>() / n);
    let slope = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum::<f64>() / pts.iter().map(|p| (p.0 - mx).powi(2)).sum::<f64>();
    let back = pts.iter().map(|p| p.2).sum::<f64>() / n;
    println!(
        "{} fly balls: backspin {back:.0} rpm (A10b: 2,300 ± 600); sidespin {slope:.0} rpm per ° of adjusted spray (A10c: −65 ± 30); zero at {:+.1}° (A10d, report only: −10 ± 5)",
        pts.len(),
        mx - my / slope
    );
    // A10b, Nathan 2020 Fig. 4: about 2,300 rpm at 25–30° over the middle of the field.
    assert!((back - 2300.0).abs() <= 600.0, "backspin {back:.0}");
    // A10c, Nathan 2020 Fig. 2: about −2,000 rpm by +20° from a zero near −10°.
    assert!((slope + 65.0).abs() <= 30.0, "sidespin slope {slope:.0}");
}
