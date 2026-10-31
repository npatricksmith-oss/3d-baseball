//! Pitch-flight lab commands (M1).

use std::path::PathBuf;

use bb_agents::motor::{ReleaseNoise, execute_release};
use bb_agents::pitcher::{PitchDef, PitchIntent, PitcherAttrs, plan_release};
use bb_math::units::{FT, m_to_ft, m_to_in, mps_to_mph, radps_to_rpm};
use bb_math::{KeyedRng, Purpose, RngKey, Vec3};
use bb_observe::{Hand, PitchType};
use bb_physics::flight::Trajectory;
use bb_physics::pitch::{Y_TUNNEL, aim_at_plane, fly_pitch, launch, movement};
use bb_physics::spin::{spin_axis_deg, tilt_display, tilt_hours};
use bb_physics::zone::{Zone, ZoneRule};
use bb_physics::FlightEnv;
use clap::Args;
use serde_json::json;

fn fail(msg: &str) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(2)
}

fn parse_type(s: &str) -> Result<PitchType, String> {
    PitchType::from_code(s).ok_or_else(|| format!("unknown pitch type {s}; use FF, SI, FC, SL, ST, SV, CU, KC, CH, FS"))
}

#[derive(Args)]
pub struct PitchArgs {
    /// Savant pitch-type code.
    #[arg(long = "type", default_value = "FF", value_parser = parse_type)]
    ty: PitchType,
    #[arg(long, default_value_t = 10)]
    n: u32,
    #[arg(long, default_value_t = 1)]
    seed: u64,
    /// Target x at the plate, m (+ toward first base).
    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    x: f64,
    /// Target z at the plate, m.
    #[arg(long, default_value_t = 0.75)]
    z: f64,
    /// Command grade multiplier on σ (1 = average).
    #[arg(long, default_value_t = 1.0)]
    command_scale: f64,
    #[arg(long)]
    lefty: bool,
    /// Write trajectories as JSON (for the viewer).
    #[arg(long)]
    json: Option<PathBuf>,
    /// Write a Rerun recording (.rrd).
    #[arg(long)]
    rrd: Option<PathBuf>,
}

fn pitcher(lefty: bool) -> PitcherAttrs {
    let mut p = PitcherAttrs::average_rhp();
    if lefty {
        p.throws = Hand::Left;
    }
    p
}

pub fn traj_json(tr: &Trajectory, step: usize) -> serde_json::Value {
    let pts: Vec<_> = tr
        .knots
        .iter()
        .enumerate()
        .filter(|(i, _)| i % step == 0 || *i == tr.knots.len() - 1)
        .map(|(_, k)| json!([k.t, k.s.r.x, k.s.r.y, k.s.r.z]))
        .collect();
    json!(pts)
}

pub fn pitch(a: PitchArgs) {
    let env = FlightEnv::default_env();
    let p = pitcher(a.lefty);
    let def = PitchDef::template(a.ty);
    let intent = PitchIntent { ty: a.ty, target_x: a.x, target_z: a.z };
    let plan = plan_release(&env, &p, &def, &intent).unwrap_or_else(|e| fail(&format!("cannot aim there: {e:?}")));
    let noise = ReleaseNoise::new(p.command, def.command_mult, a.command_scale);
    let zone = Zone::for_batter(1.85, &ZoneRule::default());
    println!("{} from a {} average pitcher, target ({:+.2}, {:.2}) m, seed {}", a.ty.code(), if a.lefty { "LHP" } else { "RHP" }, a.x, a.z, a.seed);
    println!(" #   mph    rpm  tilt    IVB    HB   plate_x plate_z  call");
    let mut out = Vec::new();
    let mut strikes = 0;
    #[cfg(feature = "rerun")]
    let rec = a.rrd.as_ref().map(|p| {
        let r = crate::rr::open(p, "sim-cli pitch");
        crate::rr::log_field(&r, &zone);
        r
    });
    for i in 0..a.n {
        let mut rng = KeyedRng::new(RngKey { game_seed: a.seed, play_index: i, agent_id: 0x0100, purpose: Purpose::PitchRelease, sub_index: 0 });
        let ex = execute_release(&plan, &noise, &mut rng);
        let tr = fly_pitch(&env, ex.state);
        let end = tr.last().s;
        let (hb, ivb) = movement(&env, ex.state).unwrap_or((f64::NAN, f64::NAN));
        let axis = spin_axis_deg(ex.state.w, ex.state.v);
        let bounced = tr.stop == Some(1);
        let strike = !bounced && zone.is_strike(end.r.x, end.r.z, env.ball.radius);
        strikes += strike as u32;
        println!(
            "{:2} {:5.1} {:6.0} {:>5} {:6.1} {:6.1} {:+7.2} {:7.2}  {}",
            i,
            mps_to_mph(ex.shape.speed),
            radps_to_rpm(ex.state.w.norm()),
            tilt_display(tilt_hours(axis)),
            m_to_in(ivb),
            m_to_in(hb),
            m_to_ft(end.r.x),
            m_to_ft(end.r.z),
            if bounced { "dirt" } else if strike { "STRIKE" } else { "ball" }
        );
        #[cfg(feature = "rerun")]
        if let Some(rec) = &rec {
            let label = format!("{} #{i} {:.1} mph", a.ty.code(), mps_to_mph(ex.shape.speed));
            crate::rr::log_flight(rec, &format!("world/pitches/{i:03}"), &tr, crate::rr::color_for(a.ty.code()), &label);
        }
        out.push(json!({"type": a.ty.code(), "strike": strike, "path": traj_json(&tr, 5)}));
    }
    println!("strikes {strikes}/{}", a.n);
    if let Some(path) = a.json {
        std::fs::write(&path, serde_json::to_string(&json!({"pitches": out})).unwrap()).unwrap();
        println!("wrote {}", path.display());
    }
}

#[derive(Args)]
pub struct TunnelArgs {
    /// Second pitch type to pair with the fastball.
    #[arg(long = "with", default_value = "SL", value_parser = parse_type)]
    with: PitchType,
    /// Fastball target (x, z) at the plate, m.
    #[arg(long, default_value_t = 0.15, allow_hyphen_values = true)]
    x: f64,
    #[arg(long, default_value_t = 0.95)]
    z: f64,
    #[arg(long)]
    lefty: bool,
    #[arg(long)]
    json: Option<PathBuf>,
    /// Write a Rerun recording (.rrd).
    #[arg(long)]
    rrd: Option<PathBuf>,
}

/// Separation between two trajectories at a given y, m.
fn at_y(tr: &Trajectory, y: f64) -> Vec3 {
    let t = tr.find(bb_physics::flight::Crossing::YBelow(y)).unwrap_or(tr.last().t);
    tr.at(t).0
}

pub fn tunnel(a: TunnelArgs) {
    let env = FlightEnv::default_env();
    let p = pitcher(a.lefty);
    let ff = PitchDef::template(PitchType::FourSeam);
    let other = PitchDef::template(a.with);
    let plan = plan_release(&env, &p, &ff, &PitchIntent { ty: PitchType::FourSeam, target_x: a.x, target_z: a.z }).unwrap_or_else(|e| fail(&format!("cannot aim there: {e:?}")));
    let tr_ff = fly_pitch(&env, launch(plan.r, &plan.shape, plan.yaw, plan.elev));
    // Aim the second pitch through the fastball's position at the tunnel point.
    let tun = at_y(&tr_ff, Y_TUNNEL);
    let shape2 = other.shape(&p);
    let r2 = other.release_point(&p);
    let aim2 = aim_at_plane(&env, r2, &shape2, Y_TUNNEL, tun.x, tun.z, None);
    let tr2 = fly_pitch(&env, launch(r2, &shape2, aim2.yaw, aim2.elev));
    println!("FF aimed at ({:+.2}, {:.2}) m; {} aimed through the FF's tunnel point ({:.1} ft from the rear of the plate)", a.x, a.z, a.with.code(), m_to_ft(Y_TUNNEL));
    println!(" dist_from_plate_ft  separation_in");
    for ft in [50.0, 45.0, 40.0, 35.0, 30.0, 23.8, 20.0, 15.0, 10.0, 5.0, 1.417] {
        let y = ft * FT;
        let d = (at_y(&tr_ff, y) - at_y(&tr2, y)).norm();
        println!(" {ft:17.1}  {:13.2}", m_to_in(d));
    }
    let e1 = tr_ff.last().s.r;
    let e2 = tr2.last().s.r;
    println!("plate: FF ({:+.2}, {:.2}) m, {} ({:+.2}, {:.2}) m", e1.x, e1.z, a.with.code(), e2.x, e2.z);
    #[cfg(feature = "rerun")]
    if let Some(p) = &a.rrd {
        let rec = crate::rr::open(p, "sim-cli tunnel");
        crate::rr::log_field(&rec, &Zone::for_batter(1.85, &ZoneRule::default()));
        crate::rr::log_flight(&rec, "world/pitches/FF", &tr_ff, crate::rr::color_for("FF"), "FF");
        crate::rr::log_flight(&rec, &format!("world/pitches/{}", a.with.code()), &tr2, crate::rr::color_for(a.with.code()), a.with.code());
        let y = Y_TUNNEL as f32;
        let ring: Vec<[f32; 3]> = (0..=32).map(|i| {
            let ang = i as f32 / 32.0 * std::f32::consts::TAU;
            [tun.x as f32 + 0.1 * ang.cos(), y, tun.z as f32 + 0.1 * ang.sin()]
        }).collect();
        rec.log_static("world/tunnel_point", &rerun::LineStrips3D::new([ring]).with_colors([[255, 255, 255]])).unwrap();
        println!("wrote {}", p.display());
    }
    if let Some(path) = a.json {
        let v = json!({"pitches": [
            {"type": "FF", "path": traj_json(&tr_ff, 5)},
            {"type": a.with.code(), "path": traj_json(&tr2, 5)},
        ]});
        std::fs::write(&path, serde_json::to_string(&v).unwrap()).unwrap();
        println!("wrote {}", path.display());
    }
}

#[derive(Args)]
pub struct ArsenalArgs {
    #[arg(long)]
    lefty: bool,
}

pub fn arsenal(a: ArsenalArgs) {
    let env = FlightEnv::default_env();
    let p = pitcher(a.lefty);
    println!("type   mph    rpm  tilt  eff    IVB_in  HB_in   flight_s");
    for ty in PitchType::ALL {
        let def = PitchDef::template(ty);
        let plan = plan_release(&env, &p, &def, &PitchIntent { ty, target_x: 0.0, target_z: 0.75 }).expect("middle of the zone is reachable");
        let s = launch(plan.r, &plan.shape, plan.yaw, plan.elev);
        let (hb, ivb) = movement(&env, s).unwrap();
        let tr = fly_pitch(&env, s);
        println!(
            "{:4} {:5.1} {:6.0} {:>5} {:4.2} {:7.1} {:6.1} {:9.3}",
            ty.code(),
            mps_to_mph(plan.shape.speed),
            radps_to_rpm(plan.shape.spin_rate),
            tilt_display(tilt_hours(plan.shape.spin_axis_deg)),
            plan.shape.efficiency,
            m_to_in(ivb),
            m_to_in(hb),
            tr.duration()
        );
    }
}
