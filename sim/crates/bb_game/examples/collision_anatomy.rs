//! The bat–ball collision taken apart (docs/calibration/2026-11-14-collision-sloped-barrel.md).
//!
//! Part A: the Kensrud–Nathan–Smith tee experiment rebuilt in the sim's collision: a level bat
//! swung at 77 mph at the impact point (6.8 in beyond the center of mass, KNS Table II's wood bat)
//! into a ball at rest, offset across the barrel by ψ. Compares spin and scattering angle with
//! KNS Eq. (8) and the measured 179 rpm per degree (KNS Fig. 3).
//! Part B: the game contact (cage, average RHP four-seamer, middle, on time) at 0–4 cm under the
//! ball: what the normal, the tangential impulse and the bat's slope each do.
//! Part C: the same contacts with e_t and μ changed (counterfactuals; the code's values are
//! e_t(|c_n|) of physics-spec §8.2.1, μ 0.50).
//! `cargo run --release -p bb_game --example collision_anatomy`
use bb_agents::pitcher::PitchIntent;
use bb_agents::swing::{Placement, apply_placement};
use bb_game::cage::Cage;
use bb_math::Vec3;
use bb_math::units::{DEG, INCH};
use bb_observe::PitchType;
use bb_physics::BallState;
use bb_physics::bat::BatBallParams;
use bb_physics::collision::{Body, Branch, Material, axisymmetric_inv_inertia, resolve, sphere_inv_inertia};
use bb_physics::contact::{BatKinematics, ContactPoint, Detection, detect, resolve_contact};
use bb_physics::flight::{Crossing, DT_BATTED, fly};
use bb_world::pitch::pitch_flight;

const MPH: f64 = 1.0 / 0.44704;
const RPM: f64 = 60.0 / (2.0 * std::f64::consts::PI);

/// Pull-positive azimuth for a right-handed batter (pull = −X), degrees.
fn az(v: Vec3) -> f64 {
    (-v.x).atan2(v.y) / DEG
}

/// (backspin, sidespin, gyro) in rpm, as in `aim_offset_probe`: backspin about the horizontal
/// axis ⊥ to v (+X for a ball moving +Y), sidespin about v̂ × that axis (+ = breaking toward the
/// right-field line), gyro along v̂.
fn spin_parts(v: Vec3, w: Vec3) -> (f64, f64, f64) {
    let vh = v.normalized();
    let b = Vec3::new(v.y, -v.x, 0.0).normalized();
    let s = vh.cross(b);
    (w.dot(b) * RPM, w.dot(s) * RPM, w.dot(vh) * RPM)
}

/// The bodies `resolve_contact` builds, so the branch and the impulse can be read.
fn first_impact(cage: &Cage, cp: &ContactPoint, p: &BatBallParams) -> (Vec3, Branch, f64) {
    let ball = &cage.env.ball;
    let mp = &cage.bat.mass;
    let len = cage.bat.profile.length();
    let contact_pt = cp.ball.r - cp.n * ball.radius;
    let b1 = Body { inv_mass: 1.0 / ball.mass, inv_inertia: sphere_inv_inertia(ball.inertia()), v: cp.ball.v, w: cp.ball.w, r: cp.n * -ball.radius };
    let b2 = Body { inv_mass: 1.0 / mp.mass, inv_inertia: axisymmetric_inv_inertia(mp.i_cm, mp.i_ax, cp.bat.axis), v: cp.bat.v_cm, w: cp.bat.omega, r: contact_pt - cp.bat.point(mp.u_cm) };
    let rel = b1.point_velocity() - b2.point_velocity();
    let cn_abs = rel.dot(cp.n).abs();
    let e_n = p.e_n(cp.u, len, cn_abs);
    let imp = resolve(b1, b2, cp.n, Material { e_n, e_t: p.e_t(cn_abs), mu: p.mu }).expect("approaching");
    (imp.j, imp.branch, e_n)
}

struct Out {
    ev: f64,
    la: f64,
    launch_az: f64,
    land_az: f64,
    carry_ft: f64,
    back: f64,
    side: f64,
    gyro: f64,
    w_vs_axis: f64,
}

fn outcome(cage: &Cage, cp: &ContactPoint, p: &BatBallParams) -> Out {
    let r = resolve_contact(cp, &cage.bat.profile, &cage.bat.mass, p, &cage.env.ball);
    let s = r.ball_after;
    let f = fly(&cage.env, s, cp.t, DT_BATTED, &[Crossing::ZBelow(0.0)], 15.0);
    let land = f.last().s.r;
    let (back, side, gyro) = spin_parts(s.v, s.w);
    // Angle between the spin vector and the bat's axis (either sense).
    let w_vs_axis = (s.w.normalized().dot(cp.bat.axis).abs().min(1.0)).acos() / DEG;
    Out { ev: s.v.norm() * MPH, la: s.v.z.atan2(s.v.x.hypot(s.v.y)) / DEG, launch_az: az(s.v), land_az: az(land), carry_ft: land.x.hypot(land.y) / 0.3048, back, side, gyro, w_vs_axis }
}

fn main() {
    let cage = Cage::average();
    let prof = &cage.bat.profile;
    let mp = &cage.bat.mass;
    let ball = cage.env.ball;
    let code = cage.bat.contact;

    // ---------------- Part A: the KNS tee experiment ----------------
    println!("Part A. KNS tee replica: level bat, 77 mph at the impact point (b = 6.8 in past the CM), ball at rest.");
    let u_i = mp.u_cm + 6.8 * INCH;
    let rb = prof.radius(u_i);
    let v_imp = 77.0 / MPH;
    let w_bat = v_imp / u_i; // pivot at the knob
    let bat = BatKinematics { knob: Vec3::ZERO, axis: Vec3::new(1.0, 0.0, 0.0), v_cm: Vec3::new(0.0, w_bat * mp.u_cm, 0.0), omega: Vec3::new(0.0, 0.0, w_bat) };
    let alpha = ball.inertia_factor;
    let r_x = ball.mass * alpha / (1.0 + alpha) * (1.0 / mp.mass + (u_i - mp.u_cm).powi(2) / mp.i_cm + rb * rb / mp.i_ax);
    println!("  bat: M {:.3} kg, u_cm {:.3} m, I_cm {:.4}, I_ax {:.2e}; R_bat {:.4} m at the impact; r_x {:.3}", mp.mass, mp.u_cm, mp.i_cm, mp.i_ax, rb, r_x);
    for (p, label) in [(code, "e_t(|c_n|), §8.2.1"), (code.with_constant_e_t(0.464), "e_t 0.464 (KNS wood)")] {
        println!("  {label} (μ {:.2}):  ψ°   spin rpm (KNS Eq. 8)   θ°    rpm/°θ   EV mph  branch", p.mu);
        let mut pts = Vec::new();
        for psi_d in [5.0, 10.0, 15.0, 20.0, 25.0, 30.0] {
            let psi = psi_d * DEG;
            let n = Vec3::new(0.0, psi.cos(), psi.sin());
            let center = bat.point(u_i) + n * (rb + ball.radius);
            let cp = ContactPoint { t: 0.0, u: u_i, n, ball: BallState { r: center, v: Vec3::ZERO, w: Vec3::ZERO }, bat };
            let r = resolve_contact(&cp, prof, mp, &p, &ball);
            let s = r.ball_after;
            let theta = s.v.z.atan2(s.v.y) / DEG;
            let spin = s.w.norm() * RPM;
            let e_t = p.e_t(v_imp * psi.cos());
            let kns = (1.0 + e_t) * v_imp * psi.sin() / (ball.radius * (1.0 + r_x) * (1.0 + alpha)) * RPM;
            let (_, br, _) = first_impact(&cage, &cp, &p);
            pts.push((theta, spin));
            println!("               {psi_d:4.0}   {spin:7.0} ({kns:7.0})      {theta:5.1}   {:6.1}   {:5.1}   {br:?}", spin / theta, s.v.norm() * MPH);
        }
        let n = pts.len() as f64;
        let (mx, my) = (pts.iter().map(|p| p.0).sum::<f64>() / n, pts.iter().map(|p| p.1).sum::<f64>() / n);
        let slope = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum::<f64>() / pts.iter().map(|p| (p.0 - mx).powi(2)).sum::<f64>();
        println!("  fitted slope {slope:.0} rpm per degree of θ (KNS Fig. 3, measured: 179, all bats, ⟨e_x⟩ 0.405)");
    }

    // ---------------- Part B: the game contact ----------------
    let uss = prof.length() - code.ss_from_tip;
    let rel = cage.release(&PitchIntent { ty: PitchType::FourSeam, target_x: 0.0, target_z: 0.75 });
    let plan = cage.plan_perfect(rel, None).unwrap();
    let tr = pitch_flight(&cage.env, rel);
    let contact = |d: f64| -> ContactPoint {
        let sw = apply_placement(&plan, &Placement { v_offset: d, ..Default::default() });
        match detect(&tr, &sw, prof, &ball) {
            Detection::Contact(cp) => cp,
            Detection::Miss(_) => panic!("miss at {d}"),
        }
    };
    println!("\nPart B. Game contact (middle, on time). Azimuths pull-positive, relative to the sweet spot's path (AD).");
    println!(" under |  ψ°   bat slope° | normal az | c: n / across / along m/s | J: n / across / along N·s  branch | launch az  land az |   LA°  EV mph | back side gyro rpm | ω vs axis°");
    for d in [0.0, -0.01, -0.02, -0.03, -0.04] {
        let cp = contact(d);
        let k = cp.bat;
        let v_ss = k.point_velocity(k.point(uss), mp.u_cm);
        let ad = az(v_ss);
        let ball_pt_v = cp.ball.v + cp.ball.w.cross(cp.n * -ball.radius);
        let bat_pt_v = k.point_velocity(cp.ball.r - cp.n * ball.radius, mp.u_cm);
        let c = ball_pt_v - bat_pt_v;
        let across = k.axis.cross(cp.n);
        let psi = ((-c).normalized().dot(cp.n)).acos() / DEG;
        let (j, br, _) = first_impact(&cage, &cp, &code);
        let o = outcome(&cage, &cp, &code);
        println!(
            "{:5.0} cm | {psi:5.1}  {:6.1}   |  {:+6.1}   | {:6.1} {:6.1} {:6.2}      | {:6.3} {:6.3} {:6.3}  {br:?} | {:+6.1}   {:+6.1}  | {:5.1} {:6.1} | {:5.0} {:5.0} {:5.0} | {:5.1}",
            -100.0 * d,
            (k.axis.z).asin() / DEG,
            az(cp.n) - ad,
            c.dot(cp.n),
            c.dot(across),
            c.dot(k.axis),
            j.dot(cp.n),
            j.dot(across),
            j.dot(k.axis),
            o.launch_az - ad,
            o.land_az - ad,
            o.la,
            o.ev,
            o.back,
            o.side,
            o.gyro,
            o.w_vs_axis
        );
    }
    // Where the launch azimuth comes from at 2 and 3 cm under: the incoming ball, plus the
    // normal part of the impulse, plus its tangential part.
    for d in [-0.02, -0.03] {
        let cp = contact(d);
        let ad = az(cp.bat.point_velocity(cp.bat.point(uss), mp.u_cm));
        let (j, _, _) = first_impact(&cage, &cp, &code);
        let jn = cp.n * j.dot(cp.n);
        let jt = j - jn;
        let v_n = cp.ball.v + jn / ball.mass;
        let v_all = cp.ball.v + j / ball.mass;
        println!(
            "  {:.0} cm under: launch az − AD from the normal impulse alone {:+5.1}; with the tangential impulse {:+5.1} (tangential impulse {:.2} N·s, az {:+5.1}, elevation {:+5.1}°)",
            -100.0 * d,
            az(v_n) - ad,
            az(v_all) - ad,
            jt.norm(),
            az(jt) - ad,
            jt.z.atan2(jt.x.hypot(jt.y)) / DEG
        );
    }

    // ---------------- Part C: e_t and μ ----------------
    println!("\nPart C. Counterfactual tangential parameters (middle, on time). Code: e_t(|c_n|) (§8.2.1), μ 0.50; these rows hold e_t constant.");
    println!(" under   e_t    μ   | branch | launch az land az (− AD) | LA°   EV mph | back  side  gyro rpm | carry ft");
    for d in [-0.02, -0.03] {
        let cp = contact(d);
        let ad = az(cp.bat.point_velocity(cp.bat.point(uss), mp.u_cm));
        for (e_t, mu) in [(0.40, 0.50), (0.464, 0.50), (0.16, 0.50), (0.0, 0.50), (0.40, 0.30), (0.40, 0.20), (0.40, 0.15), (0.0, 0.0)] {
            let p = BatBallParams { mu, ..code.with_constant_e_t(e_t) };
            let (_, br, _) = first_impact(&cage, &cp, &p);
            let o = outcome(&cage, &cp, &p);
            println!(
                "{:3.0} cm  {e_t:5.3} {mu:4.2}  | {br:?} | {:+6.1}   {:+6.1}          | {:5.1} {:6.1} | {:5.0} {:5.0} {:5.0} | {:5.0}",
                -100.0 * d,
                o.launch_az - ad,
                o.land_az - ad,
                o.la,
                o.ev,
                o.back,
                o.side,
                o.gyro,
                o.carry_ft
            );
        }
    }

    // ---------------- Part D: sidespin against spray (Nathan 2020, Fig. 2) ----------------
    // Fly balls at 25–30° launch and 90+ mph from timing −15…+15 ms and 1.5–4.5 cm under, on a
    // middle pitch. Nathan's sign: adjusted spray negative to the pull field; sidespin positive
    // breaking toward the left-field line (for a right-handed batter, slicing = negative).
    for (p, label) in [(code, "code, e_t(|c_n|) §8.2.1"), (code.with_constant_e_t(0.40), "e_t 0.40 constant, before"), (code.with_constant_e_t(0.0), "e_t 0")] {
        let mut pts: Vec<(f64, f64, f64)> = Vec::new();
        for it in -10..=10 {
            let dt = it as f64 * 0.0015;
            for k in 0..=24 {
                let d = -0.015 - 0.00125 * k as f64;
                let sw = apply_placement(&plan, &Placement { v_offset: d, t_shift: dt, ..Default::default() });
                let Detection::Contact(cp) = detect(&tr, &sw, prof, &ball) else { continue };
                let o = outcome(&cage, &cp, &p);
                if o.la >= 25.0 && o.la < 30.0 && o.ev >= 90.0 {
                    pts.push((-o.land_az, -o.side, o.back));
                }
            }
        }
        let n = pts.len() as f64;
        let (mx, my) = (pts.iter().map(|p| p.0).sum::<f64>() / n, pts.iter().map(|p| p.1).sum::<f64>() / n);
        let slope = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum::<f64>() / pts.iter().map(|p| (p.0 - mx).powi(2)).sum::<f64>();
        let zero = mx - my / slope;
        println!("\nPart D ({label}): {} fly balls (LA 25–30°, EV 90+). Sidespin = {:.0} rpm per degree of adjusted spray, zero at {:+.1}° (Nathan Fig. 2: zero near −10°, about −2,000 rpm by +20°, i.e. about −65 rpm per degree). Backspin mean {:.0} rpm (Statcast Fig. 4: about 2,300 at 25–30° over the middle).", pts.len(), slope, zero, pts.iter().map(|p| p.2).sum::<f64>() / n);
        for lo in [-30, -20, -10, 0, 10, 20] {
            let b: Vec<&(f64, f64, f64)> = pts.iter().filter(|p| p.0 >= lo as f64 && p.0 < lo as f64 + 10.0).collect();
            if b.is_empty() { continue; }
            let m = b.len() as f64;
            println!("  spray {:+3}…{:+3}: n {:3}  sidespin {:+6.0}  backspin {:5.0}", lo, lo + 10, b.len(), b.iter().map(|p| p.1).sum::<f64>() / m, b.iter().map(|p| p.2).sum::<f64>() / m);
        }
    }

    // ---------------- Part E: spray − AD against launch angle, by bat slope and e_t ----------------
    // The on-time contact at 0–4.5 cm under, with the bat rotated about the ball's center so its
    // slope is a fraction of the actual (1 = the swing's own, 0 = level); fit landing spray − AD
    // (and launch-direction az − AD) on launch angle. MLB (collision-targets note §2): spray − AD
    // falls 0.37° per degree of launch angle, from +13.9° at 0–10° to −0.6° at 25–35°.
    println!("\nPart E. On time, middle: spray − AD (pull +) against launch angle, the bat's slope scaled.");
    println!(" slope× bat slope° e_t  | land: −AD at LA 10 / 27, per °LA | launch dir: at 10 / 27, per °LA | backspin, sidespin at LA 27");
    for (p, e_t) in [(code, "law"), (code.with_constant_e_t(0.0), "0.00")] {
        for frac in [1.0, 0.75, 0.5, 0.25, 0.0] {
            let mut pts: Vec<(f64, f64, f64, f64, f64)> = Vec::new();
            let mut slope_deg = 0.0;
            for k in 0..=18 {
                let d = 0.005 - 0.0025 * k as f64;
                let cp0 = contact(d);
                let ah = Vec3::new(cp0.bat.axis.x, cp0.bat.axis.y, 0.0).normalized();
                let kx = cp0.bat.axis.cross(ah);
                let ang = kx.norm().atan2(cp0.bat.axis.dot(ah)) * (1.0 - frac);
                let kx = kx.normalized();
                let rot = |q: Vec3| if ang == 0.0 { q } else { q.rotate(kx, ang) };
                let mut cp = cp0;
                cp.n = rot(cp0.n);
                cp.bat.axis = rot(cp0.bat.axis);
                cp.bat.knob = cp0.ball.r + rot(cp0.bat.knob - cp0.ball.r);
                cp.bat.v_cm = rot(cp0.bat.v_cm);
                cp.bat.omega = rot(cp0.bat.omega);
                slope_deg = cp.bat.axis.z.asin() / DEG;
                let ad = az(cp.bat.point_velocity(cp.bat.point(uss), mp.u_cm));
                let o = outcome(&cage, &cp, &p);
                pts.push((o.la, o.land_az - ad, o.launch_az - ad, o.back, -o.side));
            }
            let fit = |f: &dyn Fn(&(f64, f64, f64, f64, f64)) -> f64| {
                let q: Vec<&(f64, f64, f64, f64, f64)> = pts.iter().filter(|p| p.0 > -5.0 && p.0 < 40.0).collect();
                let n = q.len() as f64;
                let mx = q.iter().map(|p| p.0).sum::<f64>() / n;
                let my = q.iter().map(|p| f(p)).sum::<f64>() / n;
                let b = q.iter().map(|p| (p.0 - mx) * (f(p) - my)).sum::<f64>() / q.iter().map(|p| (p.0 - mx).powi(2)).sum::<f64>();
                (my + b * (10.0 - mx), my + b * (27.0 - mx), b)
            };
            let (l10, l27, lb) = fit(&|p| p.1);
            let (d10, d27, db) = fit(&|p| p.2);
            let (_, bk27, _) = fit(&|p| p.3);
            let (_, sd27, _) = fit(&|p| p.4);
            println!("  {frac:4.2}   {slope_deg:6.1}   {e_t:>4} |  {l10:+6.1} / {l27:+6.1}, {lb:+5.2}       |  {d10:+6.1} / {d27:+6.1}, {db:+5.2}    | {bk27:5.0}, {sd27:+6.0}");
        }
    }

    // ---------------- Part F: the same fit by pitch height (the swing's own bat slope) ----------------
    // MLB (collision-targets note §2): spray-on-LA slope −0.30 bottom third, −0.35 middle, −0.35 top:
    // no steeper for low pitches, where the bat is steeper.
    println!("\nPart F. On time, x 0, by pitch height (code, e_t(|c_n|)): landing spray − AD on launch angle.");
    for z in [0.55, 0.75, 0.95] {
        let rel = cage.release(&PitchIntent { ty: PitchType::FourSeam, target_x: 0.0, target_z: z });
        let plan = cage.plan_perfect(rel, None).unwrap();
        let tr = pitch_flight(&cage.env, rel);
        let mut pts: Vec<(f64, f64)> = Vec::new();
        let mut slope_deg = 0.0;
        for k in 0..=18 {
            let d = 0.005 - 0.0025 * k as f64;
            let sw = apply_placement(&plan, &Placement { v_offset: d, ..Default::default() });
            let Detection::Contact(cp) = detect(&tr, &sw, prof, &ball) else { continue };
            slope_deg = cp.bat.axis.z.asin() / DEG;
            let ad = az(cp.bat.point_velocity(cp.bat.point(uss), mp.u_cm));
            let o = outcome(&cage, &cp, &code);
            if o.la > -5.0 && o.la < 40.0 {
                pts.push((o.la, o.land_az - ad));
            }
        }
        let n = pts.len() as f64;
        let mx = pts.iter().map(|p| p.0).sum::<f64>() / n;
        let my = pts.iter().map(|p| p.1).sum::<f64>() / n;
        let b = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum::<f64>() / pts.iter().map(|p| (p.0 - mx).powi(2)).sum::<f64>();
        println!("  z {z:.2} m: bat slope {slope_deg:6.1}°, spray − AD {:+6.1} at LA 10, slope {b:+5.2}° per °LA", my + b * (10.0 - mx));
    }
}
