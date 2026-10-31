//! M2 exit-gate sweeps (PLAN §14): early → pull, late → opposite field, undercut → backspin
//! fly, overcut → topspin grounder, off-barrel → weak.

use bb_agents::pitcher::PitchIntent;
use bb_agents::swing::Placement;
use bb_game::cage::Cage;
use bb_math::units::{m_to_ft, mps_to_mph, rad_to_deg, radps_to_rpm};
use bb_observe::PitchType;
use bb_world::pitch::{BattedBall, SwingResult};

fn hit(cage: &Cage, p: Placement) -> Option<BattedBall> {
    let rel = cage.release(&PitchIntent { ty: PitchType::FourSeam, target_x: 0.0, target_z: 0.75 });
    match cage.swing_with(rel, &p).swing {
        SwingResult::Contact(b) => Some(*b),
        _ => None,
    }
}

fn show(label: &str, b: &Option<BattedBall>) {
    match b {
        Some(b) => {
            let w = b.contact.ball_after.w;
            println!(
                "{label:22} EV {:5.1} mph LA {:6.1}° spray {:6.1}° dist {:5.0} ft u {:5.2} in e_n {:.3} spin {:5.0} rpm (ωx {:+6.0}) bat {:4.1} mph",
                mps_to_mph(b.exit_speed),
                rad_to_deg(b.launch_angle),
                rad_to_deg(b.spray_angle),
                m_to_ft(b.projected_distance),
                b.contact.point.u / 0.0254,
                b.contact.e_n,
                radps_to_rpm(w.norm()),
                radps_to_rpm(w.x),
                mps_to_mph(b.contact.bat_speed)
            );
        }
        None => println!("{label:22} miss"),
    }
}

#[test]
fn sweeps_print() {
    let cage = Cage::average();
    show("perfect", &hit(&cage, Placement::default()));
    for ms in [-20.0, -10.0, -5.0, 5.0, 10.0, 20.0] {
        show(&format!("timing {ms:+} ms"), &hit(&cage, Placement { t_shift: ms / 1000.0, ..Default::default() }));
    }
    for cm in [-4.0, -3.0, -2.0, -1.0, 1.0, 2.0, 3.0, 4.0] {
        show(&format!("v_offset {cm:+} cm"), &hit(&cage, Placement { v_offset: cm / 100.0, ..Default::default() }));
    }
    for cm in [-15.0, -10.0, -5.0, 5.0, 10.0] {
        show(&format!("reach {cm:+} cm"), &hit(&cage, Placement { reach: cm / 100.0, ..Default::default() }));
    }
}

fn must(cage: &Cage, p: Placement) -> BattedBall {
    hit(cage, p).expect("contact")
}

#[test]
fn early_pulls_late_goes_opposite() {
    // Right-handed batter: pull field is −X (negative spray).
    let cage = Cage::average();
    let s = |ms: f64| must(&cage, Placement { t_shift: ms / 1000.0, ..Default::default() }).spray_angle;
    let (early, on, late) = (s(-10.0), s(0.0), s(10.0));
    assert!(early < on && on < late, "{early} {on} {late}");
    assert!(early < 0.0 && late > 0.0);
}

#[test]
fn undercut_backspin_fly_overcut_topspin_grounder() {
    let cage = Cage::average();
    let base = must(&cage, Placement::default());
    let under = must(&cage, Placement { v_offset: -0.02, ..Default::default() });
    let over = must(&cage, Placement { v_offset: 0.02, ..Default::default() });
    // A ball moving +Y with backspin has ω_x > 0 (physics-spec §1.3).
    assert!(under.contact.ball_after.w.x > 0.0 && under.launch_angle > base.launch_angle + 0.15);
    assert!(over.contact.ball_after.w.x < 0.0 && over.launch_angle < 0.0);
}

#[test]
fn spin_rises_with_undercut_and_overcut() {
    // A10 shape: backspin grows monotonically with undercut, topspin with overcut.
    let cage = Cage::average();
    let wx = |cm: f64| must(&cage, Placement { v_offset: cm / 100.0, ..Default::default() }).contact.ball_after.w.x;
    let under: Vec<f64> = [-0.5, -1.0, -2.0, -3.0, -4.0].map(wx).to_vec();
    let over: Vec<f64> = [0.5, 1.0, 2.0, 3.0, 4.0].map(wx).to_vec();
    assert!(under.windows(2).all(|w| w[1] > w[0]), "{under:?}");
    assert!(over.windows(2).all(|w| w[1] < w[0]), "{over:?}");
}

#[test]
fn off_barrel_is_weak() {
    let cage = Cage::average();
    let base = must(&cage, Placement::default()).exit_speed;
    let tip = must(&cage, Placement { reach: -0.15, ..Default::default() });
    assert!(tip.exit_speed < 0.75 * base, "tip {} vs {}", tip.exit_speed, base);
    let handle = must(&cage, Placement { reach: 0.25, ..Default::default() });
    assert!(handle.contact.point.u < 0.50, "u {}", handle.contact.point.u);
    assert!(handle.exit_speed < 0.80 * base, "handle {} vs {}", handle.exit_speed, base);
}

#[test]
fn whiff_when_far_off() {
    let cage = Cage::average();
    assert!(hit(&cage, Placement { v_offset: 0.15, ..Default::default() }).is_none());
    assert!(hit(&cage, Placement { t_shift: 0.08, ..Default::default() }).is_none());
}
