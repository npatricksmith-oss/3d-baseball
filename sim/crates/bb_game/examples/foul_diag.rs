//! Foul diagnostics: where contact goes (spray, launch), what timing error produced it, and
//! the Statcast-style swing measures of research note R13 (docs/research/r13-foul-direction.md)
//! with the MLB values beside them.
//!
//! Usage: `cargo run --release -p bb_game --example foul_diag -- [PAs] [timing σ ms] [bat-control σ cm] [K_perception] [bat angle °]`
//!
//! The optional bat angle is a counterfactual (R13 check 4): the swing template's pivot is
//! swung about a middle-middle aim point so the bat meets it at that vertical angle
//! (`SwingTemplate::with_bat_angle`); `-` keeps an earlier argument's default.
//!
//! Definitions (R13 "Sources"): attack direction (AD) is the horizontal direction of the
//! sweet spot's velocity at the bat-ball intercept, pull-positive; attack angle (AA) its
//! elevation. For a whiff the intercept is the instant of closest approach. Intercept depth
//! is the ball's y at that instant relative to the batter's mean, positive out front. The
//! vertical bat angle is the bat axis' elevation (negative = barrel below the hands); swing
//! path tilt is the swing plane's angle to the ground; swing length is the sweet spot's
//! path over the last 150 ms. Check swings are left out of the swing measures (Statcast
//! keeps competitive swings only). The pitch is re-resolved from the recorded release and
//! executed swing, which reproduces the world's result exactly (asserted).
use bb_game::pa::{B1Report, Call, Knobs, Matchup, PA_PLAY_STRIDE, PitchRecord, simulate_pa};
use bb_math::{Vec3, m};
use bb_observe::{Hand, PitchType};
use bb_physics::contact::BatPose;
use bb_world::pitch::{SwingResult, resolve_pitch};

const DEG: f64 = 180.0 / core::f64::consts::PI;
const IN: f64 = 0.0254;

fn pct(v: &[f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let mut v = v.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    v[((v.len() - 1) as f64 * q) as usize]
}
fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len().max(1) as f64
}
fn sd(v: &[f64]) -> f64 {
    let mu = mean(v);
    m::sqrt(v.iter().map(|x| (x - mu) * (x - mu)).sum::<f64>() / v.len().max(1) as f64)
}
/// Least-squares slope of y on x.
fn slope(x: &[f64], y: &[f64]) -> (f64, f64) {
    let (mx, my) = (mean(x), mean(y));
    let sxy: f64 = x.iter().zip(y).map(|(a, b)| (a - mx) * (b - my)).sum();
    let sxx: f64 = x.iter().map(|a| (a - mx) * (a - mx)).sum();
    let syy: f64 = y.iter().map(|b| (b - my) * (b - my)).sum();
    (sxy / sxx, sxy / m::sqrt(sxx * syy))
}

#[derive(Clone, Copy, PartialEq)]
enum Outcome {
    FairPull,
    FairOppo,
    Foul,
    Whiff,
}

/// One competitive swing, measured at the intercept.
struct Sw {
    out: Outcome,
    ty: PitchType,
    /// Normalized pitch height at the plate: 0 = zone bottom, 1 = zone top.
    h: f64,
    ad: f64,
    aa: f64,
    vba: f64,
    tilt: f64,
    length_ft: f64,
    bat_mph: f64,
    /// Intercept y, m (made relative to the mean later).
    y: f64,
    /// Contact only: exit speed mph, launch angle °, landing direction ° (pull-positive).
    ev: f64,
    la: f64,
    land_dir: f64,
    /// Whiff only: how it missed. 0 = over or under the bat, 1 = past the tip, 2 = inside the
    /// knob, 3 = the closest approach is at the edge of the contact search window.
    miss: u8,
    /// Intercept time minus the swing's nominal contact time, ms.
    dt_ms: f64,
}

fn measure(mu: &Matchup, p: &PitchRecord) -> Option<Sw> {
    let sw = p.swing?;
    if !p.swung || p.check_swing || sw.t_brake.is_some() {
        return None;
    }
    let side = match mu.batter.bats {
        Hand::Right => 1.0,
        Hand::Left => -1.0,
    };
    let res = resolve_pitch(&mu.env, p.release, Some(&sw), &mu.bat);
    let (t, y, contact) = match &res.swing {
        SwingResult::Contact(b) => {
            let (ev, la, _) = p.batted.expect("re-resolved contact matches the record");
            assert!((b.exit_speed - ev).abs() < 1e-9 && (b.launch_angle - la).abs() < 1e-9, "re-resolution differs from the world");
            (b.contact.point.t, b.contact.point.ball.r.y, Some(b))
        }
        SwingResult::Miss(ca) => {
            assert!(p.batted.is_none(), "re-resolved miss matches the record");
            (ca.t, res.pitch.at(ca.t).0.y, None)
        }
        SwingResult::NoSwing => return None,
    };
    let uss = mu.bat.profile.length() - mu.bat.contact.ss_from_tip;
    let v_ss = |t: f64| {
        let k = sw.at(t);
        k.point_velocity(k.point(uss), mu.bat.mass.u_cm)
    };
    let k = sw.at(t);
    let v = v_ss(t);
    let ad = m::atan2(-side * v.x, v.y) * DEG;
    let aa = m::atan2(v.z, m::hypot(v.x, v.y)) * DEG;
    let vba = m::asin(k.axis.z) * DEG;
    let tilt = m::acos(sw.n.z.abs().min(1.0)) * DEG;
    let steps = 150;
    let mut len = 0.0;
    for i in 0..steps {
        let tt = t - 0.150 + (i as f64 + 0.5) * 0.150 / steps as f64;
        len += v_ss(tt).norm() * 0.150 / steps as f64;
    }
    let plate = p.plate.unwrap_or(Vec3::ZERO);
    let h = (plate.z - mu.zone.bottom) / (mu.zone.top - mu.zone.bottom);
    let miss = match &res.swing {
        SwingResult::Miss(ca) => {
            let (lo, hi) = (sw.t_c - bb_physics::contact::WINDOW_S, sw.t_c + bb_physics::contact::WINDOW_S);
            if (ca.t - lo).abs() < 2e-4 || (ca.t - hi).abs() < 2e-4 {
                3
            } else if ca.u >= mu.bat.profile.length() - 1e-6 {
                1
            } else if ca.u <= 1e-6 {
                2
            } else {
                0
            }
        }
        _ => 0,
    };
    let dt_ms = 1000.0 * (t - sw.t_c);
    let (out, ev, la, land_dir) = match contact {
        Some(b) => {
            let land = b.flight.last().s.r;
            let spray_pull = -side * b.spray_angle * DEG;
            let out = if matches!(p.call, Call::Foul | Call::FoulTip) {
                Outcome::Foul
            } else if spray_pull >= 0.0 {
                Outcome::FairPull
            } else {
                Outcome::FairOppo
            };
            (out, b.exit_speed / 0.44704, b.launch_angle * DEG, m::atan2(-side * land.x, land.y) * DEG)
        }
        None => (Outcome::Whiff, f64::NAN, f64::NAN, f64::NAN),
    };
    Some(Sw { out, ty: p.ty, h, ad, aa, vba, tilt, length_ft: len / 0.3048, bat_mph: v.norm() / 0.44704, y, ev, la, land_dir, miss, dt_ms })
}

fn main() {
    let n: u32 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3000);
    let mut mu = Matchup::average();
    mu.knobs = Knobs::calibrated_m3();
    if let Some(t) = std::env::args().nth(2).and_then(|s| s.parse::<f64>().ok()) {
        mu.batter.timing = t / 1000.0;
    }
    if let Some(c) = std::env::args().nth(3).and_then(|s| s.parse::<f64>().ok()) {
        mu.batter.bat_control = c / 100.0;
    }
    if let Some(k) = std::env::args().nth(4).and_then(|s| s.parse::<f64>().ok()) {
        mu.knobs.perception = k;
    }
    let mid = Vec3::new(0.0, mu.template.contact_y(&mu.batter, 0.0), 0.5 * (mu.zone.bottom + mu.zone.top));
    if let Some(a) = std::env::args().nth(5).and_then(|s| s.parse::<f64>().ok()) {
        mu.template = mu.template.with_bat_angle(&mu.batter, mid, a.to_radians());
    }
    println!(
        "timing σ {:.1} ms, bat-control σ {:.2} cm, K_perception {}, bat angle at a middle-middle aim {:.1}°",
        1000.0 * mu.batter.timing,
        100.0 * mu.batter.bat_control,
        mu.knobs.perception,
        mu.template.bat_angle(&mu.batter, mid) * DEG
    );
    let mut rep = B1Report::default();
    let (mut spray, mut la, mut terr, mut zerr) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut contacts, mut fouls, mut back, mut pull, mut oppo, mut swings) = (0u32, 0u32, 0u32, 0u32, 0u32, 0u32);
    let mut sws: Vec<Sw> = Vec::new();
    for i in 0..n {
        let pa = simulate_pa(&mu, 1, i * PA_PLAY_STRIDE);
        rep.add(&pa);
        for p in &pa.pitches {
            swings += p.swung as u32;
            if let Some(s) = measure(&mu, p) {
                sws.push(s);
            }
            let Some((_, l, s)) = p.batted else { continue };
            contacts += 1;
            let (l, s) = (l * DEG, s * DEG);
            spray.push(s);
            la.push(l);
            if let Some(e) = p.aim_error {
                terr.push(1000.0 * e.z);
                zerr.push(100.0 * e.y);
            }
            if matches!(p.call, Call::Foul | Call::FoulTip) {
                fouls += 1;
                if s.abs() > 90.0 || l > 90.0 {
                    back += 1;
                } else if s < 0.0 {
                    pull += 1;
                } else {
                    oppo += 1;
                }
            }
        }
    }
    let c = contacts.max(1) as f64;
    println!("swings {swings}, contact {contacts} ({:.1}%), foul {:.1}% of contact: pull side {:.1}, opposite {:.1}, backward {:.1}", 100.0 * c / swings as f64, 100.0 * fouls as f64 / c, 100.0 * pull as f64 / c, 100.0 * oppo as f64 / c, 100.0 * back as f64 / c);
    for (name, v) in [("spray °", &spray), ("launch °", &la), ("aim timing error ms", &terr), ("aim height error cm", &zerr)] {
        println!("{name:20} p5 {:7.1}  p25 {:7.1}  p50 {:7.1}  p75 {:7.1}  p95 {:7.1}", pct(v, 0.05), pct(v, 0.25), pct(v, 0.5), pct(v, 0.75), pct(v, 0.95));
    }
    let out: Vec<String> = rep.rows().iter().map(|(_, name, v, _, lo, hi)| format!("{name} {v:.1}{}", if v < lo || v > hi { "*" } else { "" })).collect();
    println!("{}", out.join(" | "));
    statcast_tables(&sws);
}

fn share(num: usize, den: usize) -> String {
    if den == 0 { "    —".into() } else { format!("{:5.1}", 100.0 * num as f64 / den as f64) }
}

fn statcast_tables(sws: &[Sw]) {
    let y_mean = mean(&sws.iter().map(|s| s.y).collect::<Vec<_>>());
    let depth_in = |s: &Sw| (s.y - y_mean) / IN;
    let is_contact = |s: &Sw| s.out != Outcome::Whiff;
    let is_foul = |s: &Sw| s.out == Outcome::Foul;
    let n_sw = sws.len();
    println!("\n== Statcast-style measures ({n_sw} competitive swings; MLB from R13) ==");

    println!("\nF3 by attack direction (pull-positive): share of swings, whiff per swing, foul per contact");
    println!("{:>12} | {:>6} {:>6} | {:>6} {:>6} | {:>6} {:>6}", "AD bin °", "sim", "MLB", "sim", "MLB", "sim", "MLB");
    let bins: [(f64, f64, f64, f64, f64); 9] = [
        (-1e9, -20.0, 13.7, 32.8, 75.8),
        (-20.0, -12.0, 12.3, 17.5, 56.7),
        (-12.0, -6.0, 11.7, 13.4, 49.2),
        (-6.0, 0.0, 12.8, 10.9, 42.7),
        (0.0, 6.0, 12.4, 11.8, 39.4),
        (6.0, 12.0, 11.0, 13.6, 42.4),
        (12.0, 20.0, 11.2, 22.6, 54.0),
        (20.0, 30.0, 8.5, 39.9, 76.7),
        (30.0, 1e9, 6.2, 73.6, 85.9),
    ];
    for (lo, hi, m_sh, m_wh, m_fo) in bins {
        let b: Vec<&Sw> = sws.iter().filter(|s| s.ad >= lo && s.ad < hi).collect();
        let con = b.iter().filter(|s| is_contact(s)).count();
        let label = if lo < -1e8 { format!("< {hi}") } else if hi > 1e8 { format!("> {lo}") } else { format!("{lo} to {hi}") };
        println!(
            "{label:>12} | {} {m_sh:6.1} | {} {m_wh:6.1} | {} {m_fo:6.1}",
            share(b.len(), n_sw),
            share(b.len() - con, b.len()),
            share(b.iter().filter(|s| is_foul(s)).count(), con)
        );
    }
    let near: Vec<&Sw> = sws.iter().filter(|s| s.ad.abs() < 6.0).collect();
    let near_con = near.iter().filter(|s| is_contact(s)).count();
    println!("|AD| < 6°: foul per contact {} (MLB ≈ 40), whiff per swing {} (MLB ≈ 11)", share(near.iter().filter(|s| is_foul(s)).count(), near_con), share(near.len() - near_con, near.len()));

    println!("\nF3 P(foul | contact) by launch angle: all contact, and |AD| < 6°");
    println!("{:>10} | {:>6} {:>6} {:>6} | {:>6} {:>6} {:>6}", "LA bin °", "n", "sim", "MLB", "n", "sim", "MLB");
    let la_bins: [(f64, f64, f64, f64); 9] = [
        (-1e9, -30.0, 57.0, 62.0),
        (-30.0, -10.0, 41.0, 20.0),
        (-10.0, 10.0, 22.0, 5.0),
        (10.0, 25.0, 37.0, 20.0),
        (25.0, 40.0, 47.0, 24.0),
        (40.0, 50.0, 66.0, 46.0),
        (50.0, 60.0, 74.0, 69.0),
        (60.0, 70.0, 78.0, 81.0),
        (70.0, 1e9, 87.0, 88.0),
    ];
    let con: Vec<&Sw> = sws.iter().filter(|s| is_contact(s)).collect();
    for (lo, hi, m_all, m_near) in la_bins {
        let b: Vec<&&Sw> = con.iter().filter(|s| s.la >= lo && s.la < hi).collect();
        let bn: Vec<&&&Sw> = b.iter().filter(|s| s.ad.abs() < 6.0).collect();
        let label = if lo < -1e8 { format!("< {hi}") } else if hi > 1e8 { format!("≥ {lo}") } else { format!("{lo} to {hi}") };
        println!(
            "{label:>10} | {:6} {} {m_all:6.0} | {:6} {} {m_near:6.0}",
            b.len(),
            share(b.iter().filter(|s| is_foul(s)).count(), b.len()),
            bn.len(),
            share(bn.iter().filter(|s| is_foul(s)).count(), bn.len())
        );
    }

    println!("\nF6 by outcome: AD and intercept depth (relative to this batter's mean, + = out front)");
    println!("{:>14} | {:>6} | {:>13} {:>13} | {:>15} {:>15}", "outcome", "n", "AD p50 (MLB)", "AD SD (MLB)", "depth p50 (MLB)", "depth SD (MLB)");
    for (o, name, m_ad, m_adsd, m_d, m_dsd) in [
        (Outcome::FairPull, "fair, pulled", 5.7, 14.0, 3.4, 6.5),
        (Outcome::FairOppo, "fair, opposite", -10.3, 19.0, -5.4, 7.5),
        (Outcome::Foul, "foul", -2.6, 18.0, -2.2, 10.2),
        (Outcome::Whiff, "whiff", 8.0, 26.0, 3.3, 15.6),
    ] {
        let ad: Vec<f64> = sws.iter().filter(|s| s.out == o).map(|s| s.ad).collect();
        let d: Vec<f64> = sws.iter().filter(|s| s.out == o).map(depth_in).collect();
        println!("{name:>14} | {:6} | {:6.1} ({m_ad:5.1}) {:6.1} ({m_adsd:4.0}) | {:7.1} ({m_d:5.1}) {:7.1} ({m_dsd:5.1})", ad.len(), pct(&ad, 0.5), sd(&ad), pct(&d, 0.5), sd(&d));
    }
    let x: Vec<f64> = sws.iter().map(depth_in).collect();
    let y: Vec<f64> = sws.iter().map(|s| s.ad).collect();
    let (b, r) = slope(&x, &y);
    println!("AD against depth: {b:+.2}° per inch out front, corr {r:+.2} (MLB −1.48 per inch deeper = +1.48 out front, corr ≈ 0.87)");
    let aa: Vec<f64> = sws.iter().map(|s| s.aa).collect();
    let (b, r) = slope(&y, &aa);
    println!("AA against AD: {b:+.2}° per degree toward pull, corr {r:+.2} (MLB +0.41, corr 0.71)");

    println!("\nF6 bat kinematics at the intercept (all competitive swings)");
    let q = |name: &str, v: Vec<f64>, mlb: &str| {
        println!("{name:>24}: mean {:6.1}  p10 {:6.1}  p50 {:6.1}  p90 {:6.1}   MLB {mlb}", mean(&v), pct(&v, 0.1), pct(&v, 0.5), pct(&v, 0.9));
    };
    q("bat speed mph", sws.iter().map(|s| s.bat_mph).collect(), "mean 69.9, p10 62.3, p50 71.2, p90 77.5");
    q("attack angle °", aa.clone(), "mean 9.0, p10 −5, p90 23");
    q("attack direction °", y.clone(), "p10 −25, p50 ≈ 0, p90 23");
    q("swing path tilt °", sws.iter().map(|s| s.tilt).collect(), "mean 32, p10 23, p90 41");
    q("swing length ft", sws.iter().map(|s| s.length_ft).collect(), "mean 7.2, p10 6.0, p90 8.4");
    q("vertical bat angle °", sws.iter().map(|s| s.vba).collect(), "−25 to −35 (Blast, vendor)");
    println!("By pitch height (normalized; thirds of the zone): vertical bat angle p50 / swing path tilt p50 / foul per contact");
    for (lo, hi, name, m_tilt, m_foul) in [
        (-1e9, 0.0, "below zone", "", "52.2"),
        (0.0, 1.0 / 3.0, "bottom third", "37", "46.7"),
        (1.0 / 3.0, 2.0 / 3.0, "middle third", "32", "47.3"),
        (2.0 / 3.0, 1.0, "top third", "25", "57.8"),
        (1.0, 1e9, "above zone", "", "69.1"),
    ] {
        let b: Vec<&Sw> = sws.iter().filter(|s| s.h >= lo && s.h < hi).collect();
        let vba: Vec<f64> = b.iter().map(|s| s.vba).collect();
        let tilt: Vec<f64> = b.iter().map(|s| s.tilt).collect();
        let con = b.iter().filter(|s| is_contact(s)).count();
        println!(
            "{name:>14}: n {:5}  VBA {:6.1}  tilt {:5.1} (MLB {m_tilt:>2})  foul {} (MLB {m_foul})",
            b.len(),
            pct(&vba, 0.5),
            pct(&tilt, 0.5),
            share(b.iter().filter(|s| is_foul(s)).count(), con)
        );
    }

    println!("\nHow whiffs miss, by attack direction (share of whiffs in the bin)");
    println!("{:>12} | {:>6} | {:>8} {:>8} {:>8} {:>8}", "AD bin °", "whiffs", "over/und", "past tip", "in knob", "window");
    for (lo, hi) in [(-1e9, -12.0), (-12.0, 12.0), (12.0, 30.0), (30.0, 1e9)] {
        let b: Vec<&Sw> = sws.iter().filter(|s| s.out == Outcome::Whiff && s.ad >= lo && s.ad < hi).collect();
        let k = |m: u8| share(b.iter().filter(|s| s.miss == m).count(), b.len());
        let label = if lo < -1e8 { format!("< {hi}") } else if hi > 1e8 { format!("> {lo}") } else { format!("{lo} to {hi}") };
        println!("{label:>12} | {:6} | {:>8} {:>8} {:>8} {:>8}", b.len(), k(0), k(1), k(2), k(3));
    }
    let dts: Vec<f64> = sws.iter().filter(|s| is_contact(s)).map(|s| s.dt_ms).collect();
    println!("contact time minus nominal, ms: p1 {:.1}  p5 {:.1}  p50 {:.1}  p95 {:.1}  p99 {:.1} (search window ±{:.0})", pct(&dts, 0.01), pct(&dts, 0.05), pct(&dts, 0.5), pct(&dts, 0.95), pct(&dts, 0.99), 1000.0 * bb_physics::contact::WINDOW_S);

    println!("\nF4 early / late fouls (attack direction)");
    let fouls: Vec<&Sw> = sws.iter().filter(|s| is_foul(s)).collect();
    let early: Vec<f64> = fouls.iter().filter(|s| s.ad > 10.0).map(|s| s.la).collect();
    let late: Vec<f64> = fouls.iter().filter(|s| s.ad < -10.0).map(|s| s.la).collect();
    println!(
        "fouls with pull-side AD {} % (MLB 45), opposite {} % (MLB 55)",
        share(fouls.iter().filter(|s| s.ad > 0.0).count(), fouls.len()),
        share(fouls.iter().filter(|s| s.ad <= 0.0).count(), fouls.len())
    );
    let below = |v: &[f64]| share(v.iter().filter(|l| **l < -10.0).count(), v.len());
    println!("early fouls (AD > +10): n {}, LA p50 {:.0}° (MLB 0), below −10° {} % (MLB 42)", early.len(), pct(&early, 0.5), below(&early));
    println!("late fouls (AD < −10): n {}, LA p50 {:.0}° (MLB 33), below −10° {} % (MLB 6)", late.len(), pct(&late, 0.5), below(&late));

    println!("\nF2 foul quality");
    let fev: Vec<f64> = fouls.iter().map(|s| s.ev).collect();
    let fla: Vec<f64> = fouls.iter().map(|s| s.la).collect();
    println!("foul EV p25 / p50 / p75: {:.0} / {:.0} / {:.0} mph (MLB 70 / 77 / 84)", pct(&fev, 0.25), pct(&fev, 0.5), pct(&fev, 0.75));
    println!("foul LA p5 / p25 / p50 / p75 / p95: {:.0} / {:.0} / {:.0} / {:.0} / {:.0}° (MLB −43 / 1 / 30 / 50 / 71)", pct(&fla, 0.05), pct(&fla, 0.25), pct(&fla, 0.5), pct(&fla, 0.75), pct(&fla, 0.95));
    let fair: Vec<f64> = sws.iter().filter(|s| matches!(s.out, Outcome::FairPull | Outcome::FairOppo)).map(|s| s.la).collect();
    println!("{:>12} | {:>12} | {:>12}", "LA band °", "fouls (MLB)", "fair (MLB)");
    for (lo, hi, m_f, m_fair) in [(-1e9, -10.0, 21.6, 21.1), (-10.0, 10.0, 7.8, 25.6), (10.0, 25.0, 14.2, 22.1), (25.0, 50.0, 30.4, 24.0), (50.0, 1e9, 26.0, 7.2)] {
        let label = if lo < -1e8 { format!("< {hi}") } else if hi > 1e8 { format!("> {lo}") } else { format!("{lo} to {hi}") };
        let inb = |v: &[f64]| share(v.iter().filter(|l| **l >= lo && **l < hi).count(), v.len());
        println!("{label:>12} | {} ({m_f:4.1}) | {} ({m_fair:4.1})", inb(&fla), inb(&fair));
    }

    println!("\nF1 foul share of contact by pitch type");
    for (ty, mlb) in [(PitchType::FourSeam, "57.2"), (PitchType::Slider, "46–48"), (PitchType::Changeup, "46–48"), (PitchType::Curveball, "46–48")] {
        let b: Vec<&Sw> = sws.iter().filter(|s| s.ty == ty && is_contact(s)).collect();
        println!("{:>4}: n {:5}  foul {} (MLB {mlb})", ty.code(), b.len(), share(b.iter().filter(|s| is_foul(s)).count(), b.len()));
    }

    println!("\nF5 foul landing direction (pull-positive, from straightaway; R13 has no MLB split by side)");
    let dirs: Vec<f64> = fouls.iter().map(|s| s.land_dir).collect();
    for (lo, hi) in [(45.0, 60.0), (60.0, 90.0), (90.0, 181.0)] {
        let pull = dirs.iter().filter(|d| **d >= lo && **d < hi).count();
        let oppo = dirs.iter().filter(|d| -**d >= lo && -**d < hi).count();
        println!("|dir| {lo:3.0}–{hi:3.0}°: pull side {} %, opposite {} %", share(pull, dirs.len()), share(oppo, dirs.len()));
    }
}
