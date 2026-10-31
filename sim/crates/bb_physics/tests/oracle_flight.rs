//! Cross-check against the independent Python oracle (`analysis/oracle/golden/flight.json`).
//! The oracle integrates the full (r, v, ω) state at Δt = 0.1 ms with continuous spin decay;
//! the sim uses the spec's production steps (2 ms pitches, 5 ms batted balls). Observed
//! agreement is ~3e-8 m; the tolerances leave headroom without hiding a real model change.

use bb_math::Vec3;
use bb_physics::flight::{Crossing, DT_BATTED, DT_PITCH, fly};
use bb_physics::{AeroParams, BallState, FlightEnv};
use serde_json::Value;

fn golden(name: &str) -> Value {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../analysis/oracle/golden/");
    serde_json::from_str(&std::fs::read_to_string(format!("{p}{name}")).unwrap()).unwrap()
}

fn v3(v: &Value) -> Vec3 {
    Vec3::new(v[0].as_f64().unwrap(), v[1].as_f64().unwrap(), v[2].as_f64().unwrap())
}

#[test]
fn flight_matches_oracle() {
    let cases = golden("flight.json");
    let mut worst_pos: f64 = 0.0;
    let mut worst_t: f64 = 0.0;
    for c in cases.as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let s0 = BallState { r: v3(&c["r0"]), v: v3(&c["v0"]), w: v3(&c["w0"]) };
        let env = FlightEnv {
            rho: c["rho"].as_f64().unwrap(),
            aero: AeroParams { tau100: c["tau100"].as_f64(), ..AeroParams::default() },
            ..FlightEnv::calculator()
        };
        let (stop, dt) = match c["stop"]["kind"].as_str().unwrap() {
            "ground" => (Crossing::ZBelow(0.0), DT_BATTED),
            "plane_y" => (Crossing::YBelow(c["stop"]["y"].as_f64().unwrap()), DT_PITCH),
            k => panic!("unknown stop {k}"),
        };
        let tr = fly(&env, s0, 0.0, dt, &[stop], 20.0);
        let res = &c["result"];
        let end = tr.last();
        let dp = (end.s.r - v3(&res["r"])).norm();
        let dv = (end.s.v - v3(&res["v"])).norm();
        let dw = (end.s.w - v3(&res["w"])).norm();
        let dtt = (end.t - res["t"].as_f64().unwrap()).abs();
        let dapex = (tr.apex_z - res["apex_z"].as_f64().unwrap()).abs();
        println!("{name:28} |dr| {dp:.2e} m  |dv| {dv:.2e}  |dw| {dw:.2e}  dt {dtt:.2e} s  dapex {dapex:.2e}");
        worst_pos = worst_pos.max(dp);
        worst_t = worst_t.max(dtt);
        assert!(dp < 1e-6, "{name}: position differs by {dp} m");
        assert!(dtt < 1e-7, "{name}: time differs by {dtt} s");
        assert!(dv < 1e-6, "{name}: velocity differs by {dv} m/s");
        assert!(dw < 1e-6, "{name}: spin differs by {dw} rad/s");
        assert!(dapex < 1e-6, "{name}: apex differs by {dapex} m");
    }
    println!("worst |dr| {worst_pos:.2e} m, worst dt {worst_t:.2e} s");
}

#[test]
fn air_density_matches_oracle() {
    use bb_physics::atmosphere::{Weather, air_density};
    for c in golden("air_density.json").as_array().unwrap() {
        let w = Weather {
            temp_f: c["temp_f"].as_f64().unwrap(),
            pressure_inhg: c["pressure_inhg"].as_f64().unwrap(),
            rh_pct: c["rh_pct"].as_f64().unwrap(),
            elev_ft: c["elev_ft"].as_f64().unwrap(),
        };
        let rho = air_density(&w);
        let exp = c["rho"].as_f64().unwrap();
        assert!((rho - exp).abs() < 1e-12, "{w:?}: {rho} vs {exp}");
    }
}

#[test]
fn spin_axis_matches_oracle() {
    use bb_physics::spin::{decompose, spin_axis_deg, tilt_hours};
    for c in golden("spin_axis.json").as_array().unwrap() {
        let (w, v) = (v3(&c["w"]), v3(&c["v"]));
        let a = spin_axis_deg(w, v);
        let exp = c["spin_axis_deg"].as_f64().unwrap();
        let d = ((a - exp + 540.0) % 360.0) - 180.0;
        assert!(d.abs() < 1e-9, "axis {a} vs {exp}");
        // Oracle tilt is in [0, 12) with 0 = 12:00; ours is (0, 12].
        let t = tilt_hours(a) % 12.0;
        let te = c["tilt_hours"].as_f64().unwrap() % 12.0;
        let dt = ((t - te + 18.0) % 12.0) - 6.0;
        assert!(dt.abs() < 1e-9, "tilt {t} vs {te}");
        assert!((decompose(w, v).efficiency - c["efficiency"].as_f64().unwrap()).abs() < 1e-12);
    }
}
