//! Bat and collision tests: physics-spec §14.2 A7–A9, A11, A12, and the oracle's
//! `collision.json` / `bat_default.json` fixtures.

use bb_math::units::{INCH, MPH};
use bb_math::{Mat3, Vec3};
use bb_physics::BallParams;
use bb_physics::bat::{BatProfile, sweet_spot};
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
    // Tune e so q = 0.20 at u_ss for the default bat, and check the §6.3 sanity anchor:
    // 85 mph pitch, 75 mph bat at the contact point → ~107 mph.
    let ball = BallParams::default();
    let mp = BatProfile::default_34in().mass_props();
    let u = sweet_spot(mp.length);
    let r = ball.mass * mp.inv_eff_mass(u);
    let e = 0.20 * (1.0 + r) + r;
    let q = collision_efficiency(e, ball.mass, mp.inv_eff_mass(u));
    let ev = q * 85.0 + (1.0 + q) * 75.0;
    println!("A8: u_ss {:.2} in, r {r:.3}, e_peak needed {e:.3}, EV {ev:.1} mph", u / INCH);
    assert!((q - 0.20).abs() < 0.01);
    assert!((105.0..=109.0).contains(&ev));
    assert!((0.40..=0.52).contains(&e), "e_peak {e} outside the spec's plausible range");
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
