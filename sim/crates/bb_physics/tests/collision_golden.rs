//! Bat and collision tests: physics-spec §14.2 A7–A9, A10a, A11, A12, A14, and the oracle's
//! `collision.json` / `bat_default.json` / `tangential_restitution.json` fixtures.

use bb_math::units::{INCH, MPH};
use bb_math::{Mat3, Vec3};
use bb_physics::bat::{BatBallParams, BatProfile, sweet_spot};
use bb_physics::contact::{BatKinematics, ContactPoint, resolve_contact};
use bb_physics::{BallParams, BallState};
use bb_physics::collision::{
    Body, Branch, Material, axisymmetric_inv_inertia, collision_efficiency, resolve, sphere_inv_inertia,
};
use serde_json::Value;

fn golden(name: &str) -> Value {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../analysis/oracle/golden/");
    serde_json::from_str(&std::fs::read_to_string(format!("{p}{name}")).unwrap()).unwrap()
}
fn v3(v: &Value) -> Vec3 {
    Vec3::new(v[0].as_f64().unwrap(), v[1].as_f64().unwrap(), v[2].as_f64().unwrap())
}
fn m3(v: &Value) -> Mat3 {
    let mut r = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] = v[i][j].as_f64().unwrap();
        }
    }
    Mat3::from_rows(r)
}

#[test]
fn a12_bat_mass_properties_match_oracle() {
    let mp = BatProfile::default_34in().mass_props();
    let g = golden("bat_default.json");
    let rel = |a: f64, b: &Value| (a - b.as_f64().unwrap()).abs() / b.as_f64().unwrap();
    println!("bat: M {:.4} kg ({:.1} oz), u_cm {:.2} in, I_cm {:.5}, I_ax {:.3e}, MOI@6in {:.4}",
        mp.mass, mp.mass / 0.028349523, mp.u_cm / INCH, mp.i_cm, mp.i_ax, mp.moi_about(6.0 * INCH));
    assert!(rel(mp.mass, &g["M"]) < 1e-9);
    assert!(rel(mp.u_cm, &g["u_cm"]) < 1e-9);
    assert!(rel(mp.i_cm, &g["I_cm"]) < 1e-8);
    assert!(rel(mp.i_ax, &g["I_ax"]) < 1e-8);
    assert!(rel(mp.moi_about(6.0 * INCH), &g["moi_6in"]) < 1e-8);
    // §6.2 cross-check values (rounded in the spec).
    assert!((mp.mass - 0.965).abs() < 0.002);
    assert!((mp.u_cm / INCH - 23.5).abs() < 0.1);
}

#[test]
fn collisions_match_oracle() {
    for c in golden("collision.json").as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let b1 = Body {
            inv_mass: 1.0 / c["m"].as_f64().unwrap(),
            inv_inertia: m3(&c["inv_I1"]),
            v: v3(&c["v"]),
            w: v3(&c["w"]),
            r: v3(&c["r1"]),
        };
        let b2 = Body {
            inv_mass: c["inv_M2"].as_f64().unwrap(),
            inv_inertia: m3(&c["inv_I2"]),
            v: v3(&c["V"]),
            w: v3(&c["W"]),
            r: v3(&c["r2"]),
        };
        let mat = Material {
            e_n: c["e_n"].as_f64().unwrap(),
            e_t: c["e_t"].as_f64().unwrap(),
            mu: c["mu"].as_f64().unwrap(),
        };
        let out = resolve(b1, b2, v3(&c["n"]), mat).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let res = &c["result"];
        let close = |a: Vec3, b: &Value| (a - v3(b)).norm() <= 1e-9 * (1.0 + v3(b).norm());
        assert!(close(out.j, &res["J"]), "{name}: J {:?} vs {}", out.j, res["J"]);
        assert!(close(out.b1.v, &res["v"]), "{name}: v");
        assert!(close(out.b1.w, &res["w"]), "{name}: w");
        assert!(close(out.b2.v, &res["V"]), "{name}: V");
        assert!(close(out.b2.w, &res["W"]), "{name}: W");
        let br = if out.branch == Branch::Stick { "stick" } else { "slide" };
        assert_eq!(br, res["branch"].as_str().unwrap(), "{name}: branch");
        assert_eq!(out.energy_guard, res["energy_guard"].as_bool().unwrap(), "{name}: guard");
    }
}

/// A head-on impact on the bat's axis-normal line, slender-bat idealization (contact offset
/// along the axis only), with bat axis +X and normal +Y.
fn head_on(u: f64, e: f64, v_pitch: f64, v_bat_cm: f64) -> (f64, f64) {
    let ball = BallParams::default();
    let mp = BatProfile::default_34in().mass_props();
    let b1 = Body {
        inv_mass: 1.0 / ball.mass,
        inv_inertia: sphere_inv_inertia(ball.inertia()),
        v: Vec3::new(0.0, -v_pitch, 0.0),
        w: Vec3::ZERO,
        r: Vec3::new(0.0, -ball.radius, 0.0),
    };
    // Bat translating +Y at the given center-of-mass speed, no rotation.
    let b2 = Body {
        inv_mass: 1.0 / mp.mass,
        inv_inertia: axisymmetric_inv_inertia(mp.i_cm, mp.i_ax, Vec3::X),
        v: Vec3::new(0.0, v_bat_cm, 0.0),
        w: Vec3::ZERO,
        r: Vec3::new(u - mp.u_cm, 0.0, 0.0),
    };
    let out = resolve(b1, b2, Vec3::Y, Material { e_n: e, e_t: 0.4, mu: 0.5 }).unwrap();
    let q = collision_efficiency(e, ball.mass, mp.inv_eff_mass(u));
    (out.b1.v.y, q * v_pitch + (1.0 + q) * v_bat_cm)
}

#[test]
fn a7_head_on_analytic() {
    for &u_in in &[22.0, 26.0, 28.0, 30.0, 32.5] {
        let (got, want) = head_on(u_in * INCH, 0.45, 85.0 * MPH, 30.0);
        assert!((got - want).abs() < 1e-9 * want, "u {u_in}: {got} vs {want}");
    }
}

#[test]
fn a8_sweet_spot_efficiency() {
    // physics-spec §8.2 / §14.2 A8 (2026-11-15): the default bat's sweet-spot collision
    // efficiency at game speed is the sourced q = 0.215, inside the range 0.21–0.23 (lab q of
    // 34-in wood bats 0.214 and 0.227, Nathan et al. 2010 Table I; best 1999 MLB wood bat
    // 0.228, NCAA BESR; real Statcast p90 squared-up ratio, docs/calibration/2026-11-15-
    // collision-efficiency-sources.md). The default parameters are checked, not a derived e.
    // Game speed: an 85 mph pitch at the plate and a 75 mph bat, head-on (160 mph closing).
    let ball = BallParams::default();
    let mp = BatProfile::default_34in().mass_props();
    let params = BatBallParams::default();
    let u = sweet_spot(mp.length);
    let cn = (85.0 + 75.0) * MPH;
    let e = params.e_n(u, mp.length, cn);
    let q = collision_efficiency(e, ball.mass, mp.inv_eff_mass(u));
    let ev = q * 85.0 + (1.0 + q) * 75.0;
    println!("A8: u_ss {:.2} in, e {e:.4}, q {q:.4}, EV {ev:.1} mph", u / INCH);
    assert!((0.21..=0.23).contains(&q), "sweet-spot q {q} outside the sourced 0.21–0.23");
    assert!((107.5..=111.0).contains(&ev), "EV {ev}");
    // At the COR's reference speed the target is the sourced central value itself.
    let q_ref = collision_efficiency(params.e_n(u, mp.length, params.v_ref), ball.mass, mp.inv_eff_mass(u));
    assert!((q_ref - 0.215).abs() < 0.002, "q at v_ref {q_ref}");
}

#[test]
fn a8_lab_bbcor_at_29_in() {
    // Lab cross-check (physics-spec §8.2): Nathan, Smith, Faber & Russell 2010 (arXiv:1009.2549)
    // measured the ball–bat COR of 34-in wood bats 29 in from the knob, a ball fired at about
    // 110 mph onto a bat at rest: 0.490 and 0.494 for two bats, 0.489 (SD 0.009) over 54
    // impacts on four. The model's e_n there and at that normal speed must sit within 0.02 of
    // 0.489 (the band allows the unmeasured speed slope and the rigid-bat profile). The old
    // value (e_ref 0.442, q 0.20) gives 0.460 and fails, which was the sourcing gap.
    let mp = BatProfile::default_34in().mass_props();
    let e = BatBallParams::default().e_n(29.0 * INCH, mp.length, 110.0 * MPH);
    assert!((e - 0.489).abs() <= 0.02, "lab BBCOR at 29 in {e}");
    let old = BatBallParams { e_ref: 0.442, ..BatBallParams::default() }.e_n(29.0 * INCH, mp.length, 110.0 * MPH);
    assert!((old - 0.489).abs() > 0.02, "the check must separate the old value ({old})");
}

#[test]
fn a9_kns_gripping_spin() {
    // Stationary ball struck obliquely by a bat moving +Y; the contact normal is tilted by ψ
    // in the Y–Z plane. In the gripping regime KNS gives ω = (1+e_t)·v_bat·sinψ / (R(1+r_t)(1+α))
    // with r_x the tangential mass ratio. Here the bat is a free particle at the contact
    // (contact on the center-of-mass line, no rotation coupling along the tangent).
    let ball = BallParams::default();
    let psi: f64 = 0.35;
    let n = Vec3::new(0.0, psi.cos(), psi.sin());
    let m_bat = 0.9;
    let v_bat = 30.0;
    let b1 = Body {
        inv_mass: 1.0 / ball.mass,
        inv_inertia: sphere_inv_inertia(ball.inertia()),
        v: Vec3::ZERO,
        w: Vec3::ZERO,
        r: n * -ball.radius,
    };
    let b2 = Body { inv_mass: 1.0 / m_bat, inv_inertia: Mat3::ZERO, v: Vec3::new(0.0, v_bat, 0.0), w: Vec3::ZERO, r: Vec3::ZERO };
    let e_t = 0.4;
    let out = resolve(b1, b2, n, Material { e_n: 0.5, e_t, mu: 10.0 }).unwrap();
    assert_eq!(out.branch, Branch::Stick);
    // For a free-particle bat the tangential mass ratio is r_x = m·α / ((1+α)·M).
    let alpha = ball.inertia_factor;
    let rx = ball.mass / m_bat * alpha / (1.0 + alpha);
    let w_kns = (1.0 + e_t) * v_bat * psi.sin() / (ball.radius * (1.0 + rx) * (1.0 + alpha));
    let w = out.b1.w.norm();
    println!("A9: ω {w:.3} rad/s vs KNS {w_kns:.3}");
    assert!((w - w_kns).abs() < 1e-9 * w_kns);
}

#[test]
fn a11_energy_never_increases() {
    // Sweep ordinary bat-ball geometries; with the guard, no collision gains energy.
    let ball = BallParams::default();
    let mp = BatProfile::default_34in().mass_props();
    let mut worst = f64::NEG_INFINITY;
    for i in 0..12 {
        for k in 0..12 {
            let tilt = -0.6 + 1.2 * i as f64 / 11.0;
            let u = 20.0 * INCH + (13.0 * INCH) * k as f64 / 11.0;
            let n = Vec3::new(0.0, tilt.cos(), tilt.sin());
            let rb = BatProfile::default_34in().radius(u);
            let b1 = Body {
                inv_mass: 1.0 / ball.mass,
                inv_inertia: sphere_inv_inertia(ball.inertia()),
                v: Vec3::new(1.0, -38.0, -3.0),
                w: Vec3::new(-220.0, 10.0, 40.0),
                r: n * -ball.radius,
            };
            let b2 = Body {
                inv_mass: 1.0 / mp.mass,
                inv_inertia: axisymmetric_inv_inertia(mp.i_cm, mp.i_ax, Vec3::X),
                v: Vec3::new(0.0, 15.0, 2.0),
                w: Vec3::new(0.0, 0.0, -35.0),
                r: Vec3::new(u - mp.u_cm, 0.0, 0.0) + n * rb,
            };
            let ke = |b: &Body| {
                let i = b.inv_inertia.inverse().unwrap();
                0.5 * b.v.norm_sq() / b.inv_mass + 0.5 * b.w.dot(i * b.w)
            };
            let out = resolve(b1, b2, n, Material { e_n: 0.45, e_t: 0.4, mu: 0.5 }).unwrap();
            let gain = (ke(&out.b1) + ke(&out.b2)) - (ke(&b1) + ke(&b2));
            worst = worst.max(gain);
            assert!(gain <= 1e-9, "tilt {tilt} u {u}: gained {gain} J");
        }
    }
    println!("A11: worst ΔKE {worst:.3e} J");
}

#[test]
fn a11_coupled_guard_case_from_review() {
    // Codex review #3: e_t = 0 alone still gains energy through K's normal–tangential
    // coupling; the frictionless fallback must engage and dissipate.
    let b1 = Body {
        inv_mass: 1.0,
        inv_inertia: Mat3::diag(1.0, 1.0, 1000.0),
        v: Vec3::new(-1.0, -100.0 / 11.0, 0.0),
        w: Vec3::ZERO,
        r: Vec3::new(1.0, -0.1, 0.0),
    };
    let b2 = Body::immovable(Vec3::ZERO);
    let ke = |b: &Body| 0.5 * b.v.norm_sq() / b.inv_mass + 0.5 * b.w.dot(b.inv_inertia.inverse().unwrap() * b.w);
    for e_t in [0.15, 0.0] {
        let out = resolve(b1, b2, Vec3::X, Material { e_n: 0.55, e_t, mu: 0.5 }).unwrap();
        assert!(out.energy_guard && out.frictionless_fallback, "e_t {e_t}");
        assert!(ke(&out.b1) <= ke(&b1) + 1e-12, "gained {} J", ke(&out.b1) - ke(&b1));
    }
}

#[test]
fn a14_tangential_restitution_law_matches_oracle() {
    // physics-spec §8.2.1, oracle `tangential_restitution.json`: the law's values, then
    // bat–ball contacts below, inside and above the ramp resolved with e_t(|c_n|).
    let g = golden("tangential_restitution.json");
    let params = BatBallParams::default();
    for row in g["law"].as_array().unwrap() {
        let cn = row["cn_abs"].as_f64().unwrap();
        let want = row["e_t"].as_f64().unwrap();
        assert!((params.e_t(cn) - want).abs() < 1e-15, "e_t({cn}) {} vs oracle {want}", params.e_t(cn));
    }
    let mut regimes = std::collections::BTreeSet::new();
    for c in g["collisions"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let b1 = Body { inv_mass: 1.0 / c["m"].as_f64().unwrap(), inv_inertia: m3(&c["inv_I1"]), v: v3(&c["v"]), w: v3(&c["w"]), r: v3(&c["r1"]) };
        let b2 = Body { inv_mass: c["inv_M2"].as_f64().unwrap(), inv_inertia: m3(&c["inv_I2"]), v: v3(&c["V"]), w: v3(&c["W"]), r: v3(&c["r2"]) };
        let n = v3(&c["n"]);
        let cn = (b1.point_velocity() - b2.point_velocity()).dot(n).abs();
        let res = &c["result"];
        assert!((cn - res["cn_abs"].as_f64().unwrap()).abs() < 1e-12, "{name}: |c_n|");
        let e_t = params.e_t(cn);
        assert!((e_t - res["e_t"].as_f64().unwrap()).abs() < 1e-15, "{name}: e_t");
        regimes.insert(if cn <= params.v_t_lab { "lab" } else if cn >= params.v_t_game { "game" } else { "ramp" });
        let mat = Material { e_n: c["e_n"].as_f64().unwrap(), e_t, mu: c["mu"].as_f64().unwrap() };
        let out = resolve(b1, b2, n, mat).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let close = |a: Vec3, b: &Value| (a - v3(b)).norm() <= 1e-9 * (1.0 + v3(b).norm());
        assert!(close(out.j, &res["J"]), "{name}: J");
        assert!(close(out.b1.v, &res["v"]), "{name}: v");
        assert!(close(out.b1.w, &res["w"]), "{name}: w");
        assert!(close(out.b2.v, &res["V"]), "{name}: V");
        assert!(close(out.b2.w, &res["W"]), "{name}: W");
    }
    assert_eq!(regimes.len(), 3, "the fixture covers the lab value, the ramp and the game value");
}

/// A14 through the production entry point: each oracle contact state (`contacts` in
/// `tangential_restitution.json`) is rebuilt as a `ContactPoint` and resolved by
/// `resolve_contact`, which selects e_t from the contact point's own normal approach speed.
/// e_n is held at the fixture's 0.45 (no location falloff, no speed slope), since the oracle
/// is given e_n rather than computing it. Returns the worst ball-velocity and ball-spin
/// errors, each relative to 1 + |oracle value|, and the number of contacts in the ramp.
fn a14_contact_errors(params: &BatBallParams) -> (f64, f64, usize) {
    let g = golden("tangential_restitution.json");
    let prof = BatProfile::default_34in();
    let mp = prof.mass_props();
    let ball = BallParams::default();
    let (mut ev, mut ew, mut ramp) = (0.0_f64, 0.0_f64, 0);
    for c in g["contacts"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let e_n = c["e_n"].as_f64().unwrap();
        let held = BatBallParams { e_ref: e_n, s_e: 0.0, f_tip: 1.0, f_taper: 1.0, f_handle: 1.0, mu: c["mu"].as_f64().unwrap(), ..*params };
        let bat = BatKinematics { knob: v3(&c["knob"]), axis: v3(&c["axis"]), v_cm: v3(&c["v_cm"]), omega: v3(&c["omega"]) };
        let cp = ContactPoint {
            t: 0.0,
            u: c["u"].as_f64().unwrap(),
            n: v3(&c["n"]),
            ball: BallState { r: v3(&c["ball_r"]), v: v3(&c["ball_v"]), w: v3(&c["ball_w"]) },
            bat,
        };
        let r = resolve_contact(&cp, &prof, &mp, &held, &ball);
        let res = &c["result"];
        assert_eq!(r.e_n, e_n, "{name}: e_n held");
        assert_eq!(r.impulses as u64, res["impulses"].as_u64().unwrap(), "{name}: impulses");
        assert!((r.bat_speed - res["bat_speed"].as_f64().unwrap()).abs() < 1e-9, "{name}: bat speed");
        assert!((r.attack_angle - res["attack_angle"].as_f64().unwrap()).abs() < 1e-9, "{name}: attack angle");
        let rel = |a: Vec3, b: &Value| (a - v3(b)).norm() / (1.0 + v3(b).norm());
        ev = ev.max(rel(r.ball_after.v, &res["v"]));
        ew = ew.max(rel(r.ball_after.w, &res["w"]));
        let cn = res["cn_abs"].as_f64().unwrap();
        if cn > 34.0 && cn < 65.0 {
            ramp += 1;
        }
    }
    (ev, ew, ramp)
}

#[test]
fn a14_resolve_contact_matches_oracle() {
    // Codex spin review P2-1: the law must be applied by `resolve_contact` itself, at the
    // contact point's normal speed, not only by the standalone function.
    let (ev, ew, ramp) = a14_contact_errors(&BatBallParams::default());
    println!("A14 contacts: worst relative error, ball v {ev:.2e}, ball ω {ew:.2e}; {ramp} in the ramp");
    assert!(ev < 1e-9 && ew < 1e-9, "resolve_contact against the oracle: v {ev:.2e}, ω {ew:.2e}");
    assert!(ramp >= 3, "the fixture puts at least three contacts inside the ramp, rotating bats among them");
    // Counterfactual: the same contacts resolved with a wrong ramp (34–60 m/s, which the
    // review showed passes A10a–c) must fail. 1e-6 is a thousand times the tolerance above.
    let wrong = BatBallParams { v_t_game: 60.0, ..BatBallParams::default() };
    let (ev, ew, _) = a14_contact_errors(&wrong);
    println!("A14 contacts, wrong 34-60 ramp: ball v {ev:.2e}, ball ω {ew:.2e}");
    assert!(ev > 1e-6 || ew > 1e-6, "a 34–60 m/s ramp would pass A14's contact check");
}

#[test]
fn a10a_kns_tee_spin() {
    // physics-spec §14.2 A10a: KNS 2017's tee. A level default bat swung about its knob at
    // 77 mph at the impact point, 6.8 in past the center of mass (KNS Table II), into a ball at
    // rest, offset across the barrel by ψ. Spin about the bat's axis follows KNS Eq. (8) with
    // the lab e_t (the tee's normal speed, 34.4·cos ψ m/s, is below the ramp for ψ ≥ 10°), and
    // the spin per degree of launch matches KNS Fig. 3's measured 179 rpm/° (±10%).
    let ball = BallParams::default();
    let prof = BatProfile::default_34in();
    let mp = prof.mass_props();
    let params = BatBallParams::default();
    let u_i = mp.u_cm + 6.8 * INCH;
    let rb = prof.radius(u_i);
    let v_imp = 77.0 * MPH;
    let rate = v_imp / u_i;
    let bat = BatKinematics { knob: Vec3::ZERO, axis: Vec3::X, v_cm: Vec3::new(0.0, rate * mp.u_cm, 0.0), omega: Vec3::new(0.0, 0.0, rate) };
    let alpha = ball.inertia_factor;
    let r_x = ball.mass * alpha / (1.0 + alpha) * (1.0 / mp.mass + (u_i - mp.u_cm).powi(2) / mp.i_cm + rb * rb / mp.i_ax);
    let mut pts = Vec::new();
    for psi_d in [10.0, 15.0, 20.0, 25.0, 30.0_f64] {
        let psi = psi_d.to_radians();
        let n = Vec3::new(0.0, psi.cos(), psi.sin());
        let center = bat.point(u_i) + n * (rb + ball.radius);
        let cp = ContactPoint { t: 0.0, u: u_i, n, ball: BallState { r: center, v: Vec3::ZERO, w: Vec3::ZERO }, bat };
        let r = resolve_contact(&cp, &prof, &mp, &params, &ball);
        let s = r.ball_after;
        // Eq. (8) with the across-barrel relative speed at the contact point (the bat's
        // rotation adds a small along-axis part, which spins the ball about another axis).
        let contact_pt = center - n * ball.radius;
        let v_bat_pt = bat.point_velocity(contact_pt, mp.u_cm);
        let v_t = v_bat_pt.dot(Vec3::X.cross(n)).abs();
        let w_kns = (1.0 + params.e_t_lab) * v_t / (ball.radius * (1.0 + r_x) * (1.0 + alpha));
        assert!((s.w.x / w_kns - 1.0).abs() < 1e-9, "ψ {psi_d}: ω_x {} vs KNS Eq. 8 {w_kns}", s.w.x);
        pts.push((s.v.z.atan2(s.v.y).to_degrees(), s.w.norm() * 60.0 / (2.0 * std::f64::consts::PI)));
    }
    let n = pts.len() as f64;
    let (mx, my) = (pts.iter().map(|p| p.0).sum::<f64>() / n, pts.iter().map(|p| p.1).sum::<f64>() / n);
    let slope = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum::<f64>() / pts.iter().map(|p| (p.0 - mx).powi(2)).sum::<f64>();
    println!("A10a: {slope:.0} rpm per degree of launch (KNS Fig. 3: 179)");
    assert!((slope - 179.0).abs() <= 18.0, "spin per degree {slope:.0} outside 179 ± 18");
}
