//! Tier A flight golden tests (physics-spec §14.2 A1–A6) and §14.3 plausibility bands.

use bb_math::Vec3;
use bb_math::units::{DEG, FT, G, MPH, RPM, m_to_ft, m_to_in, mps_to_mph};
use bb_physics::flight::{Crossing, DT_BATTED, DT_PITCH, fly};
use bb_physics::{BallState, FlightEnv};

/// Batted ball toward +Y (center field) from (0, 0, z0) with pure backspin (ω ∥ +X).
fn batted(ev_mph: f64, la_deg: f64, rpm: f64, z0_ft: f64) -> BallState {
    let v = ev_mph * MPH;
    let la = la_deg * DEG;
    BallState {
        r: Vec3::new(0.0, 0.0, z0_ft * FT),
        v: Vec3::new(0.0, v * la.cos(), v * la.sin()),
        w: Vec3::new(rpm * RPM, 0.0, 0.0),
    }
}

fn carry_ft(env: &FlightEnv, s: BallState, dt: f64) -> (f64, f64, f64) {
    let tr = fly(env, s, 0.0, dt, &[Crossing::ZBelow(0.0)], 20.0);
    assert_eq!(tr.stop, Some(0));
    let end = tr.last();
    let d = (end.s.r.x.powi(2) + end.s.r.y.powi(2)).sqrt();
    (m_to_ft(d), end.t, m_to_ft(tr.apex_z))
}

#[test]
fn a1_reference_fly_ball() {
    let env = FlightEnv::calculator();
    let (carry, t, apex) = carry_ft(&env, batted(100.0, 30.0, 2500.0, 3.0), DT_BATTED);
    println!("A1: carry {carry:.2} ft, flight {t:.3} s, apex {apex:.1} ft");
    assert!((carry - 406.0).abs() <= 3.0, "carry {carry}");
    assert!((t - 5.6).abs() <= 0.1, "time {t}");
}

#[test]
fn a2_spin_decay_barely_matters() {
    let env = FlightEnv::calculator();
    let (c1, _, _) = carry_ft(&env, batted(100.0, 30.0, 2500.0, 3.0), DT_BATTED);
    let mut env2 = env;
    env2.aero.tau100 = Some(20.0);
    let (c2, _, _) = carry_ft(&env2, batted(100.0, 30.0, 2500.0, 3.0), DT_BATTED);
    println!("A2: Δcarry {:.3} ft", c2 - c1);
    assert!((c2 - c1).abs() < 1.0);
}

#[test]
fn a3_step_size_convergence() {
    let env = FlightEnv::calculator();
    let (c10, _, _) = carry_ft(&env, batted(100.0, 30.0, 2500.0, 3.0), 0.010);
    let (c1, _, _) = carry_ft(&env, batted(100.0, 30.0, 2500.0, 3.0), 0.001);
    println!("A3: |Δcarry| {:.6} ft", (c10 - c1).abs());
    assert!((c10 - c1).abs() < 0.01);
}

#[test]
fn a4_vacuum_matches_analytic() {
    let mut env = FlightEnv::calculator();
    env.rho = 0.0;
    let s = batted(90.0, 25.0, 3000.0, 3.0);
    let tr = fly(&env, s, 0.0, DT_BATTED, &[Crossing::ZBelow(0.0)], 20.0);
    // Check every knot against r0 + v0 t − ½ g t² (RK4 is exact for constant acceleration).
    for k in &tr.knots {
        let t = k.t;
        let exp = s.r + s.v * t + Vec3::new(0.0, 0.0, -0.5 * G * t * t);
        let err = (k.s.r - exp).norm() / exp.norm().max(1.0);
        assert!(err < 1e-9, "t {t}: rel err {err}");
    }
    let vz = s.v.z;
    let t_land = (vz + (vz * vz + 2.0 * G * s.r.z).sqrt()) / G;
    assert!((tr.last().t - t_land).abs() < 2e-9, "{} vs {t_land}", tr.last().t);
}

#[test]
fn a5_magnus_sign_table() {
    let env = FlightEnv::calculator();
    let up = |v: Vec3, w: Vec3| {
        let a_spin = env.accel(v, w);
        let a_none = env.accel(v, Vec3::ZERO);
        // Remove the (spin-dependent) drag part: compare components ⟂ v.
        (a_spin - a_none).reject(v.normalized())
    };
    // Batted ball +Y with backspin: ω ∥ +X → lift.
    assert!(up(Vec3::new(0.0, 40.0, 0.0), Vec3::new(250.0, 0.0, 0.0)).z > 0.0);
    // Pitch −Y with backspin: ω ∥ −X → lift.
    assert!(up(Vec3::new(0.0, -40.0, 0.0), Vec3::new(-250.0, 0.0, 0.0)).z > 0.0);
    // Pitch −Y with topspin: ω ∥ +X → drop.
    assert!(up(Vec3::new(0.0, -40.0, 0.0), Vec3::new(250.0, 0.0, 0.0)).z < 0.0);
    // Pure gyro spin produces no Magnus force.
    assert!(up(Vec3::new(0.0, -40.0, 0.0), Vec3::new(0.0, -250.0, 0.0)).norm() < 1e-12);
}

#[test]
fn a6_mirror_symmetry() {
    let env = FlightEnv::default_env();
    let base = BallState {
        r: Vec3::new(0.5, 16.5, 1.8),
        v: Vec3::new(-1.2, -38.0, -1.0),
        w: Vec3::new(-150.0, 20.0, 120.0),
    };
    // Mirror through the x = 0 plane: x → −x for positions/velocities; spin is a
    // pseudovector, so ω_x stays and ω_y, ω_z flip.
    let mirror = BallState {
        r: Vec3::new(-base.r.x, base.r.y, base.r.z),
        v: Vec3::new(-base.v.x, base.v.y, base.v.z),
        w: Vec3::new(base.w.x, -base.w.y, -base.w.z),
    };
    let a = fly(&env, base, 0.0, DT_PITCH, &[Crossing::YBelow(0.2159)], 2.0);
    let b = fly(&env, mirror, 0.0, DT_PITCH, &[Crossing::YBelow(0.2159)], 2.0);
    let (ea, eb) = (a.last(), b.last());
    assert_eq!(ea.t, eb.t);
    assert_eq!(ea.s.r.x, -eb.s.r.x);
    assert_eq!(ea.s.r.z, eb.s.r.z);
}

/// §14.3 pitch plausibility bands: release at y = 54 ft, level, pure spin axes.
fn pitch_case(mph: f64, rpm: f64, backspin: bool) -> (f64, f64, f64) {
    let env = FlightEnv::calculator();
    let sgn = if backspin { -1.0 } else { 1.0 };
    let s = BallState {
        r: Vec3::new(0.0, 54.0 * FT, 6.0 * FT),
        v: Vec3::new(0.0, -mph * MPH, 0.0),
        w: Vec3::new(sgn * rpm * RPM, 0.0, 0.0),
    };
    // The spec's cross-check stops at the front edge of the plate (17 in).
    let stop = [Crossing::YBelow(bb_physics::zone::Y_FRONT)];
    let real = fly(&env, s, 0.0, DT_PITCH, &stop, 2.0);
    let twin = fly(&FlightEnv { aero: env.aero.no_magnus(), ..env }, s, 0.0, DT_PITCH, &stop, 2.0);
    let ivb = m_to_in(real.last().s.r.z - twin.last().s.r.z);
    (real.duration(), mps_to_mph(real.last().s.v.norm()), ivb)
}

#[test]
fn plausibility_pitches() {
    for &(name, mph, rpm, back, t_ref, v_ref, ivb_ref) in &[
        ("4S 95/2300", 95.0, 2300.0, true, 0.397, 86.0, 19.9),
        ("CU 80/2600", 80.0, 2600.0, false, 0.475, 73.2, -23.5),
        ("CH 85/1700", 85.0, 1700.0, true, 0.443, 77.5, 17.9),
    ] {
        let (t, v, ivb) = pitch_case(mph, rpm, back);
        println!("{name}: t {t:.3} s (ref {t_ref}), plate {v:.1} mph (ref {v_ref}), IVB {ivb:.1} in (ref {ivb_ref})");
        assert!((t - t_ref).abs() < 0.01, "{name} t {t}");
        assert!((v - v_ref).abs() < 1.0, "{name} v {v}");
        assert!((ivb - ivb_ref).abs() < 1.5, "{name} ivb {ivb}");
    }
}

#[test]
fn plausibility_batted_balls() {
    let env = FlightEnv::calculator();
    for &(ev, la, rpm, c_ref, t_ref, apex_ref) in &[
        (100.0, 28.0, 2200.0, 410.0, 5.37, 88.0),
        (105.0, 28.0, 2400.0, 433.0, 5.67, 97.0),
        (95.0, 15.0, 1500.0, 319.0, 3.27, 31.0),
        (90.0, 45.0, 3500.0, 308.0, 6.13, 129.0),
    ] {
        // The spec's cross-check starts the ball at y0 = 2 ft (physics-spec §14.2 A1 note).
        let mut s0 = batted(ev, la, rpm, 3.0);
        s0.r.y = 2.0 * FT;
        let (c, t, apex) = carry_ft(&env, s0, DT_BATTED);
        println!("{ev}/{la}/{rpm}: carry {c:.1} ft (ref {c_ref}), {t:.2} s (ref {t_ref}), apex {apex:.0} ft (ref {apex_ref})");
        assert!((c - c_ref).abs() < 0.6);
        assert!((t - t_ref).abs() < 0.05);
        assert!((apex - apex_ref).abs() < 2.0);
    }
}
