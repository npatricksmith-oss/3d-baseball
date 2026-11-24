//! Baseline of today's swing at contact (PLAN D25; docs/swing-model-v3.md, results in
//! docs/calibration/2026-11-24-swing-contact-baseline.md): over the standard run's games (`sim-cli games --n 300` at
//! `--seed 0` and `--seed 1000`, average fielders), what the collision reads from the executed
//! swing at the contact instant and what it gives. Read-only: it plays the games exactly as the
//! standard run does and re-reads each executed swing (`PitchRecord::swing`) at the recorded
//! contact time; nothing it computes feeds back into a game.
//!
//! Every batter is mirrored to a right-hander (he stands at x < 0, the plate toward +x, the
//! pitcher toward +y). Angles: bat elevation = the bat's angle above horizontal, knob → tip;
//! bat azimuth in Fortenbaugh 2011's convention (−90° = tip at the catcher, 0° = across the
//! plate, −180° = tip away from the plate behind him); attack angle = elevation of the sweet
//! spot's velocity; attack direction and spray pull-positive. Times from the executed swing's
//! nominal contact t_c unless stated; T_a = when the ball's centre reaches the contact plane
//! y = 0.35 m (`SwingTemplate::contact_y0`).
//!
//! `cargo run --release -p bb_game --example swing_contact_baseline -- [games per set] [threads]`
use bb_agents::swing::{BRAKE_S, Swing};
use bb_game::pa::Call;
use bb_math::Vec3;
use bb_physics::Trajectory;
use bb_physics::contact::{BatPose, Detection, detect, gap};
use bb_world::pitch::BatModel;

const CONTACT_Y0: f64 = 0.35;
const DEG: f64 = std::f64::consts::PI / 180.0;
const MPH: f64 = 0.44704;

/// One swing's readings. `None` fields do not apply to it.
#[derive(Clone, Debug, Default)]
struct Row {
    set: usize,
    checked: bool,
    judged: bool,
    t_swing: f64,
    tilt: f64,
    rho: f64,
    delta: Option<f64>,
    aim_dt: Option<f64>,
    x_in: Option<f64>,
    // Contact.
    contact: bool,
    fair: bool,
    dt: f64,
    ss_speed: f64,
    pt_speed: f64,
    attack: f64,
    direction: f64,
    elev: f64,
    azim: f64,
    face: f64,
    station: f64,
    undercut: f64,
    knob_speed: f64,
    omega: f64,
    ev: f64,
    la: f64,
    spray: f64,
    // Miss.
    miss_gap: f64,
    miss_dt: f64,
    // Checked swing.
    brake: f64,
    stop_azim: f64,
    yaw_turned: f64,
}

/// When the ball's centre reaches y = `CONTACT_Y0` (as `swing_pose_probe`): on the track if it
/// gets there, otherwise extrapolated from its last 30 ms.
fn t_cross(tr: &Trajectory) -> Option<f64> {
    let end = tr.last().t;
    let (r_end, v_end) = tr.at(end);
    if r_end.y <= CONTACT_Y0 {
        let (mut lo, mut hi) = (tr.first().t, end);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if tr.at(mid).0.y > CONTACT_Y0 { lo = mid } else { hi = mid }
        }
        return Some(0.5 * (lo + hi));
    }
    let (_, v_before) = tr.at(end - 0.03);
    let a = (v_end.y - v_before.y) / 0.03;
    let (y, v) = (r_end.y - CONTACT_Y0, v_end.y);
    if a.abs() < 1e-9 {
        return (v < 0.0).then(|| end + y / -v);
    }
    let disc = v * v - 2.0 * a * y;
    if disc < 0.0 {
        return None;
    }
    let s = disc.sqrt();
    [(-v + s) / a, (-v - s) / a].into_iter().filter(|dt| *dt >= 0.0).fold(None, |m: Option<f64>, dt| Some(m.map_or(dt, |m| m.min(dt)))).map(|dt| end + dt)
}

/// Fortenbaugh's bat azimuth of a (mirrored) axis, deg, in (−300, 60].
fn azimuth(a: Vec3) -> f64 {
    let mut az = (-a.x).atan2(a.y) / DEG - 270.0;
    while az <= -300.0 {
        az += 360.0;
    }
    while az > 60.0 {
        az -= 360.0;
    }
    az
}

fn read(set: usize, step: &bb_game::game::PitchStep, sw: &Swing, bat: &BatModel, ball_r: f64) -> Row {
    let rec = &step.record;
    // A right-handed batter's swing turns about an up-normal (agent-models §3.6 step 5).
    let s = if sw.n.z >= 0.0 { 1.0 } else { -1.0 };
    let mir = |v: Vec3| Vec3::new(s * v.x, v.y, v.z);
    let u_ss = bat.profile.length() - bat.contact.ss_from_tip;
    let mut r = Row {
        set,
        checked: sw.t_brake.is_some(),
        judged: rec.swung,
        t_swing: sw.t_c - sw.t_start(),
        tilt: (s * sw.n.z).clamp(-1.0, 1.0).acos(),
        rho: sw.rho,
        aim_dt: rec.aim_error.map(|e| e.z),
        x_in: rec.plate.map(|p| -s * p.x),
        ..Default::default()
    };
    // On a ball hit the track stops at contact and T_a is extrapolated, as the client does.
    r.delta = t_cross(&step.pitch_flight).map(|ta| sw.t_c - ta);
    if let Some(tb) = sw.t_brake {
        r.brake = tb - sw.t_c;
        let a0 = mir(sw.at(sw.t_start()).axis);
        let a1 = mir(sw.at(tb + BRAKE_S).axis);
        r.stop_azim = azimuth(a1);
        // Horizontal yaw turned from the start pose, in the swing's direction (counter-clockwise
        // seen from above for a right-hander).
        let (h0, h1) = (a0.y.atan2(a0.x), a1.y.atan2(a1.x));
        let mut d = (h1 - h0) / DEG;
        while d < -180.0 {
            d += 360.0;
        }
        while d > 180.0 {
            d -= 360.0;
        }
        r.yaw_turned = d;
    }
    if let Some((tc, after)) = rec.contact_state {
        let k = sw.at(tc);
        let ball = after.r;
        let (_, u, p) = gap(&k, &bat.profile, ball, ball_r);
        let v_ss = k.point_velocity(k.point(u_ss), bat.mass.u_cm);
        let v_pt = k.point_velocity(k.point(u), bat.mass.u_cm);
        let (a, vs) = (mir(k.axis), mir(v_ss));
        let w = (ball - p).reject(k.axis);
        let z = Vec3::new(0.0, 0.0, 1.0);
        let up = z.reject(k.axis).normalized();
        let (ev, la, spray) = rec.batted.unwrap_or((0.0, 0.0, 0.0));
        r.contact = true;
        r.fair = rec.call == Call::InPlay;
        r.dt = tc - sw.t_c;
        r.ss_speed = v_ss.norm();
        r.pt_speed = v_pt.norm();
        r.attack = vs.z.atan2(vs.x.hypot(vs.y));
        r.direction = (-vs.x).atan2(vs.y);
        r.elev = a.z.asin();
        r.azim = azimuth(a);
        // The bat's axis against its sweet spot's path: 0 = square (⊥), positive = the tip
        // turned ahead of the path (hooked).
        r.face = (a.dot(vs.normalized())).clamp(-1.0, 1.0).asin();
        r.station = u_ss - u;
        r.undercut = w.dot(up);
        r.knob_speed = (k.v_cm - k.omega.cross(k.axis * bat.mass.u_cm)).norm();
        r.omega = k.omega.norm();
        r.ev = ev;
        r.la = la;
        r.spray = -s * spray;
    } else {
        match detect(&step.pitch_flight, sw, &bat.profile, &bb_physics::flight::FlightEnv::default_env().ball) {
            Detection::Miss(c) => {
                r.miss_gap = c.gap;
                r.miss_dt = c.t - sw.t_c;
            }
            Detection::Contact(_) => r.miss_gap = f64::NAN,
        }
    }
    r
}

fn run_games(set: usize, seeds: Vec<u64>) -> Vec<Row> {
    let bat = BatModel::default_34in();
    let ball_r = bb_physics::flight::FlightEnv::default_env().ball.radius;
    let fattrs: [bb_game::play::FielderAttrs; 9] = core::array::from_fn(bb_game::play::FielderAttrs::average);
    let mut out = Vec::new();
    for seed in seeds {
        let mut g = bb_game::game::Game::new(seed, false).with_fielders(fattrs);
        while let Some(step) = g.step_pitch() {
            if let Some(sw) = step.record.swing {
                out.push(read(set, &step, &sw, &bat, ball_r));
            }
        }
    }
    out
}

fn stats(v: &[f64]) -> Option<(f64, f64, f64, f64, f64, usize)> {
    let mut v: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    let m = v.iter().sum::<f64>() / n as f64;
    let sd = (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / n as f64).sqrt();
    let q = |p: f64| v[((n - 1) as f64 * p).round() as usize];
    Some((m, sd, q(0.05), q(0.5), q(0.95), n))
}

fn line(name: &str, v: &[f64], by_set: Option<(&[f64], &[f64])>) {
    match stats(v) {
        None => println!("| {name} | (none) | | | | | | |"),
        Some((m, sd, p5, p50, p95, n)) => {
            let sets = by_set.map_or(String::new(), |(a, b)| {
                let (ma, mb) = (stats(a).map_or(f64::NAN, |s| s.0), stats(b).map_or(f64::NAN, |s| s.0));
                format!("{ma:.2} / {mb:.2}")
            });
            println!("| {name} | {m:.2} | {sd:.2} | {p5:.2} | {p50:.2} | {p95:.2} | {n} | {sets} |");
        }
    }
}

/// Least-squares slope of y on x and the correlation.
fn slope(xs: &[f64], ys: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    let (mx, my) = (xs.iter().sum::<f64>() / n, ys.iter().sum::<f64>() / n);
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (x, y) in xs.iter().zip(ys) {
        sxy += (x - mx) * (y - my);
        sxx += (x - mx) * (x - mx);
        syy += (y - my) * (y - my);
    }
    (sxy / sxx, sxy / (sxx * syy).sqrt())
}

fn main() {
    let per_set: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(300);
    let threads: usize = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(8);
    // The standard run's game seeds (`sim-cli games`: seed × 1,000,003 + i).
    let mut jobs: Vec<(usize, Vec<u64>)> = Vec::new();
    for (set, base) in [(0usize, 0u64), (1, 1000)] {
        let seeds: Vec<u64> = (0..per_set).map(|i| base * 1_000_003 + i).collect();
        for c in seeds.chunks((per_set as usize).div_ceil(threads / 2).max(1)) {
            jobs.push((set, c.to_vec()));
        }
    }
    let rows: Vec<Row> = std::thread::scope(|s| {
        let hs: Vec<_> = jobs.into_iter().map(|(set, c)| s.spawn(move || run_games(set, c))).collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });

    let col = |f: &dyn Fn(&Row) -> Option<f64>, set: Option<usize>| -> Vec<f64> { rows.iter().filter(|r| set.is_none_or(|s| r.set == s)).filter_map(f).collect() };
    let show = |name: &str, f: &dyn Fn(&Row) -> Option<f64>| {
        let (all, a, b) = (col(f, None), col(f, Some(0)), col(f, Some(1)));
        line(name, &all, Some((&a, &b)));
    };
    let header = || {
        println!("| Quantity | mean | sd | p5 | p50 | p95 | n | mean set 0 / set 1000 |");
        println!("| :-- | --: | --: | --: | --: | --: | --: | :-- |");
    };

    let swings = rows.len();
    let checked = rows.iter().filter(|r| r.checked).count();
    let full: Vec<&Row> = rows.iter().filter(|r| !r.checked).collect();
    let full_contact = full.iter().filter(|r| r.contact).count();
    let full_fair = full.iter().filter(|r| r.fair).count();
    let ck_contact = rows.iter().filter(|r| r.checked && r.contact).count();
    let ck_judged = rows.iter().filter(|r| r.checked && r.judged).count();
    let ck_judged_nocontact = rows.iter().filter(|r| r.checked && r.judged && !r.contact).count();
    println!("# Swing contact baseline: {per_set} games per set, sets --seed 0 and --seed 1000\n");
    println!("## Counts\n");
    println!("| | all | set 0 | set 1000 |\n| :-- | --: | --: | --: |");
    let cnt = |f: &dyn Fn(&Row) -> bool| (rows.iter().filter(|r| f(r)).count(), rows.iter().filter(|r| r.set == 0 && f(r)).count(), rows.iter().filter(|r| r.set == 1 && f(r)).count());
    for (name, f) in [
        ("swings started", &(|_: &Row| true) as &dyn Fn(&Row) -> bool),
        ("full swings (not checked)", &|r: &Row| !r.checked),
        ("full swings with contact", &|r: &Row| !r.checked && r.contact),
        ("  fair", &|r: &Row| !r.checked && r.fair),
        ("  foul", &|r: &Row| !r.checked && r.contact && !r.fair),
        ("full swings missed", &|r: &Row| !r.checked && !r.contact),
        ("checked swings", &|r: &Row| r.checked),
        ("  touched the ball", &|r: &Row| r.checked && r.contact),
        ("  ruled a swing without contact", &|r: &Row| r.checked && r.judged && !r.contact),
        ("  ruled no swing", &|r: &Row| r.checked && !r.judged),
    ] {
        let (a, b, c) = cnt(f);
        println!("| {name} | {a} | {b} | {c} |");
    }
    println!();
    println!(
        "Rates: checked share of swings {:.1}%; contact per full swing {:.1}%; foul share of full-swing contact {:.1}%; checked swings touching {:.1}%, ruled swings {:.1}% ({} without contact).\n",
        100.0 * checked as f64 / swings as f64,
        100.0 * full_contact as f64 / full.len() as f64,
        100.0 * (full_contact - full_fair) as f64 / full_contact as f64,
        100.0 * ck_contact as f64 / checked.max(1) as f64,
        100.0 * ck_judged as f64 / checked.max(1) as f64,
        ck_judged_nocontact
    );

    println!("## Every swing\n");
    header();
    show("T_swing = t_c − t_s (ms)", &|r| Some(r.t_swing * 1e3));
    show("plane tilt β (°)", &|r| Some(r.tilt / DEG));
    show("hand radius ρ (m)", &|r| Some(r.rho));
    show("Δ = t_c − T_a (ms)", &|r| r.delta.map(|d| d * 1e3));
    show("final aim time error, aim − truth (ms)", &|r| r.aim_dt.map(|d| d * 1e3));
    println!();

    println!("## Full swings with contact: what the collision reads at the contact instant\n");
    header();
    let fc = |f: fn(&Row) -> f64| move |r: &Row| (!r.checked && r.contact).then(|| f(r));
    show("contact − t_c (ms)", &fc(|r| r.dt * 1e3));
    show("sweet-spot speed (mph)", &fc(|r| r.ss_speed / MPH));
    show("speed of the bat at the contact point (mph)", &fc(|r| r.pt_speed / MPH));
    show("knob (hands) speed (m/s)", &fc(|r| r.knob_speed));
    show("bat angular speed (°/s)", &fc(|r| r.omega / DEG));
    show("attack angle (°)", &fc(|r| r.attack / DEG));
    show("attack direction, pull + (°)", &fc(|r| r.direction / DEG));
    show("bat elevation (°)", &fc(|r| r.elev / DEG));
    show("bat azimuth, Fortenbaugh (°)", &fc(|r| r.azim));
    show("bat axis against path, hooked + (°)", &fc(|r| r.face / DEG));
    show("station: sweet spot − contact (in, + toward knob)", &fc(|r| r.station / 0.0254));
    show("ball centre above the bat axis (cm, undercut +)", &fc(|r| r.undercut * 100.0));
    show("exit speed (mph)", &fc(|r| r.ev / MPH));
    show("launch angle (°)", &fc(|r| r.la / DEG));
    show("spray, pull + (°)", &fc(|r| r.spray / DEG));
    println!();
    println!("## Fair balls only\n");
    header();
    let ff = |f: fn(&Row) -> f64| move |r: &Row| (!r.checked && r.fair).then(|| f(r));
    show("exit speed (mph)", &ff(|r| r.ev / MPH));
    show("launch angle (°)", &ff(|r| r.la / DEG));
    show("spray, pull + (°)", &ff(|r| r.spray / DEG));
    show("sweet-spot speed (mph)", &ff(|r| r.ss_speed / MPH));
    show("attack angle (°)", &ff(|r| r.attack / DEG));
    println!();

    println!("## Full swings with contact, by timing (contact − t_c)\n");
    println!("| contact − t_c (ms) | n | sweet-spot mph | point mph | attack ° | direction ° | azimuth ° | hooked ° | station in | EV mph | LA ° | spray ° | foul % |");
    println!("| :-- | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: |");
    for (lo, hi) in [(-26.0, -10.0), (-10.0, -3.0), (-3.0, 3.0), (3.0, 10.0), (10.0, 26.0)] {
        let g: Vec<&&Row> = full.iter().filter(|r| r.contact && r.dt * 1e3 >= lo && r.dt * 1e3 < hi).collect();
        if g.is_empty() {
            continue;
        }
        let n = g.len() as f64;
        let mean = |f: &dyn Fn(&Row) -> f64| g.iter().map(|r| f(r)).sum::<f64>() / n;
        println!(
            "| {lo:.0} to {hi:.0} | {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.2} | {:.1} | {:.1} | {:.1} | {:.1} |",
            g.len(),
            mean(&|r| r.ss_speed / MPH),
            mean(&|r| r.pt_speed / MPH),
            mean(&|r| r.attack / DEG),
            mean(&|r| r.direction / DEG),
            mean(&|r| r.azim),
            mean(&|r| r.face / DEG),
            mean(&|r| r.station / 0.0254),
            mean(&|r| r.ev / MPH),
            mean(&|r| r.la / DEG),
            mean(&|r| r.spray / DEG),
            100.0 * g.iter().filter(|r| !r.fair).count() as f64 / n
        );
    }
    println!();
    println!("## Full swings with contact, by location across the plate (x inside of the centre)\n");
    println!("| location | n | sweet-spot mph | attack ° | direction ° | station in | EV mph | LA ° | spray ° | foul % |");
    println!("| :-- | --: | --: | --: | --: | --: | --: | --: | --: | --: |");
    for (name, lo, hi) in [("outside (x_in < −0.072 m)", -9.0, -0.072), ("middle", -0.072, 0.072), ("inside (x_in > 0.072 m)", 0.072, 9.0)] {
        let g: Vec<&&Row> = full.iter().filter(|r| r.contact && r.x_in.is_some_and(|x| x >= lo && x < hi)).collect();
        if g.is_empty() {
            continue;
        }
        let n = g.len() as f64;
        let mean = |f: &dyn Fn(&Row) -> f64| g.iter().map(|r| f(r)).sum::<f64>() / n;
        println!(
            "| {name} | {} | {:.1} | {:.1} | {:.1} | {:.2} | {:.1} | {:.1} | {:.1} | {:.1} |",
            g.len(),
            mean(&|r| r.ss_speed / MPH),
            mean(&|r| r.attack / DEG),
            mean(&|r| r.direction / DEG),
            mean(&|r| r.station / 0.0254),
            mean(&|r| r.ev / MPH),
            mean(&|r| r.la / DEG),
            mean(&|r| r.spray / DEG),
            100.0 * g.iter().filter(|r| !r.fair).count() as f64 / n
        );
    }
    println!();
    println!("## Mechanism slopes (full swings with contact)\n");
    let fcv = |f: &dyn Fn(&Row) -> f64| full.iter().filter(|r| r.contact).map(|r| f(r)).collect::<Vec<f64>>();
    let pairs: [(&str, Vec<f64>, Vec<f64>); 5] = [
        ("spray (°) per ms of contact − t_c", fcv(&|r| r.dt * 1e3), fcv(&|r| r.spray / DEG)),
        ("hooked (°) per ms of contact − t_c", fcv(&|r| r.dt * 1e3), fcv(&|r| r.face / DEG)),
        ("LA (°) per cm of undercut", fcv(&|r| r.undercut * 100.0), fcv(&|r| r.la / DEG)),
        ("EV (mph) per mph of sweet-spot speed", fcv(&|r| r.ss_speed / MPH), fcv(&|r| r.ev / MPH)),
        ("spray (°) per ° of attack direction", fcv(&|r| r.direction / DEG), fcv(&|r| r.spray / DEG)),
    ];
    println!("| relation | slope | r |\n| :-- | --: | --: |");
    for (name, x, y) in pairs.iter() {
        let (b, rr) = slope(x, y);
        println!("| {name} | {b:.3} | {rr:.2} |");
    }
    println!();

    println!("## Full swings missed\n");
    header();
    let fm = |f: fn(&Row) -> f64| move |r: &Row| (!r.checked && !r.contact).then(|| f(r));
    show("closest surface gap (cm)", &fm(|r| r.miss_gap * 100.0));
    show("closest approach − t_c (ms)", &fm(|r| r.miss_dt * 1e3));
    show("Δ = t_c − T_a (ms)", &|r| (!r.checked && !r.contact).then_some(r.delta? * 1e3));
    println!();

    println!("## Checked swings\n");
    header();
    let ck = |f: fn(&Row) -> f64| move |r: &Row| r.checked.then(|| f(r));
    show("brake start − t_c (ms)", &ck(|r| r.brake * 1e3));
    show("bat azimuth where it stops (°)", &ck(|r| r.stop_azim));
    show("horizontal yaw turned from the start pose (°)", &ck(|r| r.yaw_turned));
}
