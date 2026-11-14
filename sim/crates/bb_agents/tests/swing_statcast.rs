//! The v2 swing (agent-models §3.6) against Statcast, not against its own code.
//!
//! Every expectation below is a figure from the swing note (`docs/calibration/2026-11-14-swing-model.md`
//! §1 and §3: a Baseball Savant 2025 sample of competitive swings, with R13's twelve-date sample
//! and Blast's vendor figures where named). The assertions are the unit-level swing: a swing is
//! planned at an aim point, optionally given the execution noise `motor::execute_swing` draws,
//! and measured the way `foul_diag` measures it (sweet-spot velocity at the intercept, swing
//! length over the last 150 ms, plane tilt from the plane normal, vertical bat angle from the
//! bat axis). `swing.rs`'s own tests already check the kinematics against hand calculations
//! (speed = `BatSpeed` at every aim, the planned α, β, ψ, the vertical-angle formula), and
//! `swing_oracle.rs` against the Python oracle; these tests add the Statcast comparison.
//!
//! For each test the comment gives the v1 swing's value (the note's §1 and §3 columns, measured
//! on main `bcbb91f` by `foul_diag` / `platoon_diag`; v1 is not in this tree, so those numbers
//! are quoted, not rerun), and the tolerance is chosen so that value fails.
//!
//! The x axis is +X toward first base: for the right-handed batter, +x is outside and −x inside.

use bb_agents::motor::{SwingNoise, execute_swing};
use bb_agents::swing::{BatterAttrs, IntendedSwing, Swing, SwingTemplate, plan_swing};
use bb_math::units::DEG;
use bb_math::{KeyedRng, Purpose, RngKey, Vec3, m};
use bb_physics::bat::{BatProfile, MassProps};
use bb_physics::contact::BatPose;

const MPH: f64 = 0.44704;
const FT: f64 = 0.3048;
/// Lateral aim of the inside and outside thirds of the plate (±0.145 m: the centers of the
/// outer thirds of the 0.432 m plate).
const X_THIRD: f64 = 0.145;
const THIRDS: [(&str, f64); 3] = [("inside", -X_THIRD), ("middle", 0.0), ("outside", X_THIRD)];
const HEIGHTS: [(&str, f64); 3] = [("bottom", 1.0 / 6.0), ("middle", 0.5), ("top", 5.0 / 6.0)];

struct Rig {
    b: BatterAttrs,
    tpl: SwingTemplate,
    prof: BatProfile,
    mp: MassProps,
    uss: f64,
}

fn rig() -> Rig {
    let prof = BatProfile::default_34in();
    let mp = prof.mass_props();
    let uss = prof.length() - 0.15;
    Rig { b: BatterAttrs::average_rhb(), tpl: SwingTemplate::default(), prof, mp, uss }
}

impl Rig {
    /// An aim point at lateral position `x` and zone height `h` (0 bottom, 1 top).
    fn aim(&self, x: f64, h: f64) -> Vec3 {
        let z = (self.tpl.zone_bottom_frac + h * (self.tpl.zone_top_frac - self.tpl.zone_bottom_frac)) * self.b.height;
        Vec3::new(x, self.tpl.contact_y(&self.b, x), z)
    }
    fn plan(&self, aim: Vec3, t: f64) -> IntendedSwing {
        plan_swing(&self.b, &self.tpl, &self.prof, &self.mp, 0.15, aim, t, self.tpl.attack_angle_at(&self.b, aim))
    }
    fn vel(&self, s: &Swing, t: f64) -> Vec3 {
        let k = s.at(t);
        k.point_velocity(k.point(self.uss), self.mp.u_cm)
    }
    /// Sweet-spot path length over the 150 ms before `t`, ft (foul_diag's swing length).
    fn length_ft(&self, s: &Swing, t: f64) -> f64 {
        let n = 150;
        let len: f64 = (0..n).map(|i| self.vel(s, t - 0.150 + (i as f64 + 0.5) * 0.150 / n as f64).norm() * 0.150 / n as f64).sum();
        len / FT
    }
    /// Sweet-spot speeds, mph, of `n` swings with the full execution noise, intercepted at the
    /// planned instant.
    fn noisy_speeds_mph(&self, aim: Vec3, n: u32, noise: &SwingNoise) -> Vec<f64> {
        (0..n)
            .map(|play| {
                let i = self.plan(aim, 0.4);
                let key = |purpose| RngKey { game_seed: 5, play_index: play, agent_id: 1, purpose, sub_index: 0 };
                let ex = execute_swing(&i, noise, &mut KeyedRng::new(key(Purpose::SwingExecution)), &mut KeyedRng::new(key(Purpose::SwingEffort)));
                self.vel(&ex.swing, 0.4).norm() / MPH
            })
            .collect()
    }
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}
fn sd(v: &[f64]) -> f64 {
    let mu = mean(v);
    m::sqrt(v.iter().map(|x| (x - mu) * (x - mu)).sum::<f64>() / v.len() as f64)
}

// Statcast: bat speed is flat across the plate within about ±2 mph (competitive swings, by
// horizontal bin: 71.4 inside, 72.1 middle, 70.1 outside; note §1). v1 turned at a fixed
// angular speed: sweet-spot speed 64.0 / 72.5 / 83.5 mph (note §1, `platoon_diag`), a 19.5 mph
// range, so both the range check and the per-bin checks fail it. Here with the full execution
// noise, 1,500 swings per third.
#[test]
fn bat_speed_is_flat_across_the_plate() {
    let r = rig();
    let noise = SwingNoise::new(&r.b, 1.0);
    let statcast = [71.4, 72.1, 70.1];
    let means: Vec<f64> = THIRDS.iter().map(|&(_, x)| mean(&r.noisy_speeds_mph(r.aim(x, 0.5), 1500, &noise))).collect();
    for (((name, _), mu), want) in THIRDS.iter().zip(&means).zip(statcast) {
        assert!((mu - want).abs() < 2.0, "{name}: {mu:.1} mph against Statcast {want}");
    }
    let (lo, hi) = (means.iter().cloned().fold(f64::MAX, f64::min), means.iter().cloned().fold(f64::MIN, f64::max));
    assert!(hi - lo < 2.0, "inside / middle / outside {means:?}: range {:.1} mph (Statcast 2.0, v1 19.5)", hi - lo);
}

// Statcast: the same batter's bat speed varies 3.2 mph SD (4.5%) swing to swing at a fixed
// depth over the middle of the plate (note §1). The sim draws it as the swing's effort
// (`EFFORT_SIGMA`, truncated at 4σ). v1 had no effort draw: at one aim its speed varied only
// with the timing and placement noise, 0.74 mph SD (v2 with effort set to 0, measured when this
// test was written), so a 3.2 mph SD fails it by a wide margin.
#[test]
fn bat_speed_spread_within_a_batter_is_statcasts() {
    let r = rig();
    let v = r.noisy_speeds_mph(r.aim(0.0, 0.5), 4000, &SwingNoise::new(&r.b, 1.0));
    let (s, mu) = (sd(&v), mean(&v));
    assert!((2.8..3.6).contains(&s), "SD {s:.2} mph against Statcast 3.2");
    assert!((0.040..0.051).contains(&(s / mu)), "relative SD {:.3} against 0.045", s / mu);
    // The mean stays at BatSpeed (32.0 m/s = 71.58 mph) less the small loss to timing noise.
    assert!((70.0..71.7).contains(&mu), "mean {mu:.2} mph");
}

// Statcast swing length 7.2–7.3 ft (league average, ESPN; this sample's competitive swings
// 7.2; foul_diag's MLB column) and longer on outside pitches than inside (7.45 outside, 7.13
// inside by horizontal bin, note §3). v1: 9.5 ft mean (note §3), so the band fails it. The
// sign check has no v1 figure by location in the note.
#[test]
fn swing_length_is_statcasts_and_longer_outside() {
    let r = rig();
    let mut all = vec![];
    for &(_, x) in &THIRDS {
        for &(_, h) in &HEIGHTS {
            all.push(r.length_ft(&r.plan(r.aim(x, h), 0.4).swing, 0.4));
        }
    }
    let mu = mean(&all);
    assert!((6.8..7.7).contains(&mu), "mean swing length {mu:.2} ft over the grid (Statcast 7.2–7.3, v1 9.5)");
    let at = |x: f64| r.length_ft(&r.plan(r.aim(x, 0.5), 0.4).swing, 0.4);
    let (inside, outside) = (at(-X_THIRD), at(X_THIRD));
    assert!(outside > inside + 0.2 && outside < inside + 1.0, "outside {outside:.2} ft, inside {inside:.2} ft (Statcast 7.45 and 7.13)");
}

// Swing path tilt by zone third, bottom / middle / top: Statcast 34.8 / 32.2 / 28.2° (this
// sample; R13's twelve dates 37 / 32 / 25). Read from the built swing's plane normal, not from
// the template's formula. v1: 28.2 / 22.9 / 16.7° (note §3), so ±1.5° fails every third.
// Note the template's tilt is set from this sample: the match is by construction, and the test
// guards that the built plane realizes it (guard clamps, mirroring, plane construction).
#[test]
fn path_tilt_by_zone_third_is_statcasts() {
    let r = rig();
    for (&(name, h), want) in HEIGHTS.iter().zip([34.8, 32.2, 28.2]) {
        let s = r.plan(r.aim(0.0, h), 0.4).swing;
        let tilt = m::acos(s.n.z.abs()) / DEG;
        assert!((tilt - want).abs() < 1.5, "{name} third: tilt {tilt:.1}° against Statcast {want}");
    }
}

// Vertical bat angle at contact −25 to −35° (Blast, vendor, via R13). v1: mean −18.5°, p10
// −30.5°, p90 −6.6° (note §3), so most of its swings are outside the band, and the mean is.
// Checked at every cell of a 3 × 3 grid (location × height), from the bat axis itself.
#[test]
fn vertical_bat_angle_is_in_blasts_band() {
    let r = rig();
    let mut angles = vec![];
    for &(xn, x) in &THIRDS {
        for &(hn, h) in &HEIGHTS {
            let s = r.plan(r.aim(x, h), 0.4).swing;
            let a = m::asin(s.axis_c.z) / DEG;
            assert!((-35.0..=-25.0).contains(&a), "{xn} / {hn}: vertical bat angle {a:.1}°");
            angles.push(a);
        }
    }
    assert!((-33.0..-27.0).contains(&mean(&angles)), "grid mean {:.1}° (v1 −18.5)", mean(&angles));
}

// Attack angle rises toward the pull side: +0.41° per degree of attack direction, across
// swings (R13, r 0.71; the note's sample gives 0.44 in the sim's games against 0.26 for v1).
// In the sim the relation is the geometry of contact time: the same swing met later in its arc
// (the batter early on the ball) is more pulled and steeper, met earlier (the batter late)
// more opposite and flatter. Sweeping the contact instant ±12 ms
// about a middle-middle aim gives a slope of 0.58 (a pure-geometry sweep of one swing, steeper
// than the game mixture of timing, location and height). v1's 0.26 fails the lower bound, a
// negative or zero slope fails it by more.
#[test]
fn attack_angle_rises_toward_the_pull_side() {
    let r = rig();
    let s = r.plan(r.aim(0.0, 0.5), 0.4).swing;
    let (mut ad, mut aa) = (vec![], vec![]);
    for i in -12..=12 {
        let v = r.vel(&s, 0.4 + i as f64 * 0.001);
        ad.push(m::atan2(-v.x, v.y) / DEG);
        aa.push(m::atan2(v.z, m::hypot(v.x, v.y)) / DEG);
    }
    let (ma, mb) = (mean(&ad), mean(&aa));
    let slope = ad.iter().zip(&aa).map(|(a, b)| (a - ma) * (b - mb)).sum::<f64>() / ad.iter().map(|a| (a - ma) * (a - ma)).sum::<f64>();
    assert!((0.35..0.80).contains(&slope), "attack angle per degree of attack direction: {slope:.2} (Statcast +0.41, v1 +0.26)");
    // The ends are the opposite-field and pulled ends (hand calculation: the bat's velocity
    // turns toward the pull side as the swing proceeds).
    assert!(ad[24] > ad[0] + 10.0, "AD at −12 ms {:.1}°, at +12 ms {:.1}°", ad[0], ad[24]);
}
