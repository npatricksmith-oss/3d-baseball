//! Rerun debug-visualizer output (D15). Everything is logged in the sim frame (PLAN §5):
//! right-handed, Z up, meters.

use bb_math::units::{FT, INCH};
use bb_physics::flight::Trajectory;
use bb_physics::zone::Zone;
use rerun::{RecordingStream, RecordingStreamBuilder};

pub type Rec = RecordingStream;

pub fn open(path: &std::path::Path, app: &'static str) -> Rec {
    let rec = RecordingStreamBuilder::new(app).save(path).expect("open .rrd");
    rec.log_static("world", &rerun::ViewCoordinates::RIGHT_HAND_Z_UP()).unwrap();
    rec
}

fn p3(x: f64, y: f64, z: f64) -> [f32; 3] {
    [x as f32, y as f32, z as f32]
}

/// Static field furniture near the battery: plate, rubber, mound outline, base lines, zone.
pub fn log_field(rec: &Rec, zone: &Zone) {
    let hw = 8.5 * INCH;
    let plate = vec![p3(0.0, 0.0, 0.0), p3(-hw, 0.5 * hw, 0.0), p3(-hw, 2.0 * hw, 0.0), p3(hw, 2.0 * hw, 0.0), p3(hw, 0.5 * hw, 0.0), p3(0.0, 0.0, 0.0)];
    rec.log_static("world/field/plate", &rerun::LineStrips3D::new([plate]).with_colors([[230, 230, 230]])).unwrap();
    let ry = bb_physics::pitch::RUBBER_Y;
    let rubber = vec![p3(-12.0 * INCH, ry, 0.254), p3(12.0 * INCH, ry, 0.254), p3(12.0 * INCH, ry + 6.0 * INCH, 0.254), p3(-12.0 * INCH, ry + 6.0 * INCH, 0.254), p3(-12.0 * INCH, ry, 0.254)];
    rec.log_static("world/field/rubber", &rerun::LineStrips3D::new([rubber]).with_colors([[230, 230, 230]])).unwrap();
    let mound: Vec<_> = (0..=48)
        .map(|i| {
            let a = i as f64 / 48.0 * core::f64::consts::TAU;
            p3(9.0 * FT * a.cos(), 59.0 * FT + 9.0 * FT * a.sin(), 0.0)
        })
        .collect();
    rec.log_static("world/field/mound", &rerun::LineStrips3D::new([mound]).with_colors([[150, 110, 70]])).unwrap();
    let b = 90.0 * FT / 2f64.sqrt();
    let lines = vec![vec![p3(0.0, 0.0, 0.0), p3(b, b, 0.0)], vec![p3(0.0, 0.0, 0.0), p3(-b, b, 0.0)]];
    rec.log_static("world/field/lines", &rerun::LineStrips3D::new(lines).with_colors([[200, 200, 200]])).unwrap();
    let y = bb_physics::zone::Y_JUDGE;
    let zbox = vec![p3(-zone.half_width, y, zone.bottom), p3(zone.half_width, y, zone.bottom), p3(zone.half_width, y, zone.top), p3(-zone.half_width, y, zone.top), p3(-zone.half_width, y, zone.bottom)];
    rec.log_static("world/zone", &rerun::LineStrips3D::new([zbox]).with_colors([[90, 160, 255]])).unwrap();
}

/// Pitch-type colors (fixed so the same type always looks the same).
pub fn color_for(code: &str) -> [u8; 3] {
    match code {
        "FF" => [235, 80, 70],
        "SI" => [250, 150, 50],
        "FC" => [180, 100, 60],
        "SL" => [240, 220, 60],
        "ST" => [200, 230, 80],
        "SV" => [120, 200, 120],
        "CU" => [70, 180, 240],
        "KC" => [80, 120, 230],
        "CH" => [80, 210, 160],
        "FS" => [170, 120, 230],
        _ => [220, 220, 220],
    }
}

/// Log one flight: a static trail plus the ball on the `t` timeline (scrub to follow it).
pub fn log_flight(rec: &Rec, path: &str, tr: &Trajectory, color: [u8; 3], label: &str) {
    let trail: Vec<_> = tr.knots.iter().map(|k| p3(k.s.r.x, k.s.r.y, k.s.r.z)).collect();
    rec.log_static(
        format!("{path}/trail"),
        &rerun::LineStrips3D::new([trail]).with_colors([color]).with_radii([0.006f32]).with_labels([label]),
    )
    .unwrap();
    for k in &tr.knots {
        rec.set_duration_secs("t", k.t);
        rec.log(
            format!("{path}/ball"),
            &rerun::Points3D::new([p3(k.s.r.x, k.s.r.y, k.s.r.z)]).with_colors([color]).with_radii([0.0369f32]),
        )
        .unwrap();
    }
    rec.reset_time();
}

/// A whole game on one `game_time` timeline: each pitch, each batted ball, a text log.
pub fn log_game(path: &std::path::Path, g: &bb_game::game::GameResult) {
    let rec = open(path, "sim-cli game");
    log_field(&rec, &bb_physics::zone::Zone::for_batter(1.85, &Default::default()));
    let f = bb_physics::ground::Field::default();
    let fence: Vec<[f32; 3]> = (0..=60)
        .map(|i| {
            let th = -core::f64::consts::FRAC_PI_4 + core::f64::consts::FRAC_PI_2 * i as f64 / 60.0;
            let d = f.fence_at(th);
            p3(d * th.sin(), d * th.cos(), f.wall_height)
        })
        .collect();
    rec.log_static("world/field/fence", &rerun::LineStrips3D::new([fence]).with_colors([[60, 160, 60]])).unwrap();
    let mut clock = 0.0;
    for v in &g.viz {
        let dur = v.batted.last().map(|s| s[0]).unwrap_or(v.pitch.last().map(|s| s[0]).unwrap_or(0.5)) + 1.0;
        rec.set_duration_secs("game_time", clock);
        rec.log("log", &rerun::TextLog::new(v.label.clone())).unwrap();
        let ps: Vec<[f32; 3]> = v.pitch.iter().map(|s| p3(s[1], s[2], s[3])).collect();
        rec.log("world/play/pitch", &rerun::LineStrips3D::new([ps]).with_colors([[235, 80, 70]])).unwrap();
        let bs: Vec<[f32; 3]> = v.batted.iter().map(|s| p3(s[1], s[2], s[3])).collect();
        rec.log("world/play/batted", &rerun::LineStrips3D::new([bs]).with_colors([[255, 255, 255]])).unwrap();
        for s in v.pitch.iter().chain(v.batted.iter()).step_by(2) {
            rec.set_duration_secs("game_time", clock + s[0]);
            rec.log("world/play/ball", &rerun::Points3D::new([p3(s[1], s[2], s[3])]).with_radii([0.15f32]).with_colors([[255, 230, 120]])).unwrap();
        }
        // Fielders (defense ids have 0x80 set in the slot byte) and runners, as capsule-ish dots.
        for (id, samples) in &v.actors {
            let fielder = id & 0x80 != 0;
            let color = if fielder { [80, 140, 255] } else { [255, 90, 90] };
            let path = format!("world/play/{}/{:04x}", if fielder { "fielders" } else { "runners" }, id);
            for (t, p) in samples {
                rec.set_duration_secs("game_time", clock + t);
                rec.log(path.clone(), &rerun::Points3D::new([p3(p.x, p.y, 0.9)]).with_radii([0.45f32]).with_colors([color])).unwrap();
            }
        }
        rec.set_duration_secs("game_time", clock + dur - 0.01);
        rec.log("world/play/fielders", &rerun::Clear::new(true)).unwrap();
        rec.log("world/play/runners", &rerun::Clear::new(true)).unwrap();
        clock += dur;
    }
}
