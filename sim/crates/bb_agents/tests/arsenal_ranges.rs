//! Arsenal templates land in public pitch-type ranges (agent-models §2.1 [E] values).
//!
//! Reference bands: approximate MLB right-handed averages from Baseball Savant pitch-movement
//! leaderboards (speed, induced vertical break, horizontal break in inches, catcher's view;
//! arm side is negative for a right-hander). Movement is the internal full-flight definition
//! (physics-spec §12), which tracks Savant's IVB/HB scale; R8 confirms sign conventions.

use bb_agents::pitcher::{PitchDef, PitchIntent, PitcherAttrs, plan_release};
use bb_math::units::{m_to_in, mps_to_mph, radps_to_rpm};
use bb_observe::{Hand, PitchType};
use bb_physics::FlightEnv;
use bb_physics::pitch::{launch, movement};
use bb_physics::spin::{spin_axis_deg, tilt_display, tilt_hours};

// (type, mph band, IVB band, HB band)
const BANDS: &[(PitchType, (f64, f64), (f64, f64), (f64, f64))] = &[
    (PitchType::FourSeam, (92.0, 96.0), (13.0, 19.0), (-11.0, -3.0)),
    (PitchType::Sinker, (91.0, 95.0), (4.0, 12.0), (-19.0, -11.0)),
    (PitchType::Cutter, (86.0, 91.0), (4.0, 12.0), (-1.0, 6.0)),
    (PitchType::Slider, (82.0, 88.0), (-3.0, 6.0), (1.0, 10.0)),
    (PitchType::Sweeper, (79.0, 85.0), (-4.0, 4.0), (10.0, 19.0)),
    (PitchType::Slurve, (78.0, 84.0), (-8.0, 0.0), (6.0, 15.0)),
    (PitchType::Curveball, (76.0, 82.0), (-15.0, -5.0), (3.0, 13.0)),
    (PitchType::KnuckleCurve, (77.0, 83.0), (-14.0, -4.0), (1.0, 11.0)),
    (PitchType::Changeup, (82.0, 88.0), (2.0, 10.0), (-18.0, -10.0)),
    (PitchType::Splitter, (83.0, 89.0), (-2.0, 6.0), (-14.0, -6.0)),
];

fn measure(p: &PitcherAttrs, ty: PitchType) -> (f64, f64, f64, f64, f64) {
    let env = FlightEnv::default_env();
    let def = PitchDef::template(ty);
    let rel = plan_release(&env, p, &def, &PitchIntent { ty, target_x: 0.0, target_z: 0.75 }).unwrap();
    let s = launch(rel.r, &rel.shape, rel.yaw, rel.elev);
    let (hb, ivb) = movement(&env, s).unwrap();
    (mps_to_mph(rel.shape.speed), m_to_in(ivb), m_to_in(hb), radps_to_rpm(rel.shape.spin_rate), spin_axis_deg(s.w, s.v))
}

#[test]
fn templates_land_in_public_ranges() {
    let p = PitcherAttrs::average_rhp();
    let mut bad = Vec::new();
    for &(ty, sp, ivb_b, hb_b) in BANDS {
        let (mph, ivb, hb, rpm, axis) = measure(&p, ty);
        let ok = (sp.0..=sp.1).contains(&mph) && (ivb_b.0..=ivb_b.1).contains(&ivb) && (hb_b.0..=hb_b.1).contains(&hb);
        println!(
            "{:2} {:5.1} mph {:5.0} rpm axis {:5.1}° ({}) IVB {:6.1} in HB {:6.1} in {}",
            ty.code(), mph, rpm, axis, tilt_display(tilt_hours(axis)), ivb, hb, if ok { "" } else { "<-- OUT OF BAND" }
        );
        if !ok {
            bad.push(ty);
        }
    }
    assert!(bad.is_empty(), "out of band: {bad:?}");
}

#[test]
fn lefty_mirrors_righty() {
    let r = PitcherAttrs::average_rhp();
    let l = PitcherAttrs { throws: Hand::Left, ..r.clone() };
    for ty in PitchType::ALL {
        let (_, ivb_r, hb_r, _, _) = measure(&r, ty);
        let (_, ivb_l, hb_l, _, _) = measure(&l, ty);
        assert!((ivb_r - ivb_l).abs() < 0.05, "{ty:?} ivb {ivb_r} vs {ivb_l}");
        assert!((hb_r + hb_l).abs() < 0.05, "{ty:?} hb {hb_r} vs {hb_l}");
    }
}
