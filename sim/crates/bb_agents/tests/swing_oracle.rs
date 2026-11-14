//! Cross-check of the v2 swing kinematics (agent-models §3.6) against the independent Python
//! oracle `analysis/oracle/swing_oracle.py` (fixture `golden/swing.json`), which integrates the
//! rate bumps numerically and differentiates positions by finite differences.

use bb_agents::swing::{BatterAttrs, SwingTemplate, plan_swing};
use bb_math::Vec3;
use bb_observe::Hand;
use bb_physics::bat::BatProfile;
use bb_physics::contact::BatPose;
use serde_json::Value;

fn v3(v: &Value) -> Vec3 {
    Vec3::new(v[0].as_f64().unwrap(), v[1].as_f64().unwrap(), v[2].as_f64().unwrap())
}

#[test]
fn swing_kinematics_match_oracle() {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../analysis/oracle/golden/swing.json");
    let cases: Value = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    let prof = BatProfile::default_34in();
    let mp = prof.mass_props();
    let tpl = SwingTemplate::default();
    let f = |c: &Value, k: &str| c[k].as_f64().unwrap();
    let mut n = 0;
    for c in cases.as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let b = BatterAttrs {
            bats: if c["bats"] == "R" { Hand::Right } else { Hand::Left },
            height: f(c, "height"),
            bat_speed: f(c, "bat_speed"),
            t_swing: f(c, "t_swing"),
            ..BatterAttrs::average_rhb()
        };
        let u_ss = f(c, "u_ss");
        let i = plan_swing(&b, &tpl, &prof, &mp, prof.length() - u_ss, v3(&c["aim"]), f(c, "t_c"), f(c, "attack_angle"));
        let s = i.swing;
        assert!((s.rho - f(c, "rho")).abs() < 1e-12, "{name}: rho {} vs {}", s.rho, f(c, "rho"));
        assert!((s.pivot - v3(&c["center"])).norm() < 1e-12, "{name}: center");
        assert!((s.n - v3(&c["n"])).norm() < 1e-12 && (s.axis_c - v3(&c["axis_c"])).norm() < 1e-12, "{name}: plane");
        assert!((s.t_start() - f(c, "t_start")).abs() < 1e-12, "{name}: start");
        for smp in c["samples"].as_array().unwrap() {
            let t = f(smp, "t");
            let k = s.at(t);
            assert!((k.knob - v3(&smp["knob"])).norm() < 1e-9, "{name} t {t}: knob {:?} vs {:?}", k.knob, v3(&smp["knob"]));
            assert!((k.axis - v3(&smp["axis"])).norm() < 1e-9, "{name} t {t}: axis");
            let v = k.point_velocity(k.point(u_ss), mp.u_cm);
            assert!((v - v3(&smp["sweet_spot_velocity"])).norm() < 1e-6, "{name} t {t}: velocity {v:?} vs {:?}", v3(&smp["sweet_spot_velocity"]));
            n += 1;
        }
        // Swing length over the last 150 ms, against the oracle's chord sum.
        let steps = 3000;
        let mut len = 0.0;
        let mut prev = s.at(f(c, "t_c") - 0.150).point(u_ss);
        for j in 1..=steps {
            let p = s.at(f(c, "t_c") - 0.150 + 0.150 * j as f64 / steps as f64).point(u_ss);
            len += (p - prev).norm();
            prev = p;
        }
        assert!((len - f(c, "swing_length_150ms")).abs() < 1e-4, "{name}: length {len} vs {}", f(c, "swing_length_150ms"));
    }
    assert!(n >= 40);
}
