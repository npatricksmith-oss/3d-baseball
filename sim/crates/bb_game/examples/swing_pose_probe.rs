//! Where the bat's knob is, from the batter's standing spot, through his swings: what the
//! client's coded swing (docs/player-visuals.md, step 4) has to put the hands on. For each
//! swing, mirrored so every batter is right-handed (he stands at x < 0, the plate toward +x,
//! the pitcher toward +y): the knob at the swing's start, at nominal contact and at the end
//! of the bat track (contact + 0.12 s), the hand radius, the height of the hand-arc's centre
//! and its distance from the standing spot, and the times.
//! `cargo run --release -p bb_game --example swing_pose_probe -- [games]`
use bb_math::Vec3;
use bb_physics::contact::BatPose;

fn row(name: &str, v: &mut Vec<f64>) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| v[((v.len() - 1) as f64 * p).round() as usize];
    println!("{name:<28} min {:7.3}  p5 {:7.3}  p50 {:7.3}  p95 {:7.3}  max {:7.3}", q(0.0), q(0.05), q(0.5), q(0.95), q(1.0));
}

fn main() {
    let games: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(8);
    let names = ["start", "contact-0.05", "contact", "end"];
    let mut knob: Vec<[Vec<f64>; 3]> = (0..4).map(|_| Default::default()).collect();
    let mut axis: Vec<[Vec<f64>; 3]> = (0..4).map(|_| Default::default()).collect();
    let mut far: Vec<Vec<f64>> = vec![Vec::new(); 4];
    let (mut rho, mut cz, mut coff, mut t0s, mut tcs, mut checks, mut n) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), 0, 0);
    for seed in 0..games {
        let mut g = bb_game::game::Game::new(seed, true);
        while let Some(step) = g.step_pitch() {
            let (Some(sw), Some((_, at))) = (step.record.swing, step.viz.standing.last()) else { continue };
            n += 1;
            checks += sw.t_brake.is_some() as u32;
            // Mirror a left-handed batter (standing at x > 0) into the right-hander's frame.
            let s = if at.x < 0.0 { 1.0 } else { -1.0 };
            let rel = |p: Vec3| Vec3::new(s * (p.x - at.x), p.y - at.y, p.z);
            for (i, t) in [sw.t_start(), sw.t_c - 0.05, sw.t_c, sw.t_c + 0.12].into_iter().enumerate() {
                let k = sw.at(t);
                let (p, a) = (rel(k.knob), Vec3::new(s * k.axis.x, k.axis.y, k.axis.z));
                for (j, c) in [p.x, p.y, p.z].into_iter().enumerate() {
                    knob[i][j].push(c);
                }
                for (j, c) in [a.x, a.y, a.z].into_iter().enumerate() {
                    axis[i][j].push(c);
                }
                far[i].push(p.x.hypot(p.y));
            }
            rho.push(sw.rho);
            cz.push(sw.pivot.z);
            let c = rel(sw.pivot);
            coff.push(c.x.hypot(c.y));
            t0s.push(sw.t_start());
            tcs.push(sw.t_c);
        }
    }
    println!("{n} swings in {games} games ({checks} checked)");
    for i in 0..4 {
        for (j, c) in ["x (to the plate)", "y (to the pitcher)", "z"].iter().enumerate() {
            row(&format!("knob {} {c}", names[i]), &mut knob[i][j]);
        }
        row(&format!("knob {} from the spot", names[i]), &mut far[i]);
        for (j, c) in ["x", "y", "z"].iter().enumerate() {
            row(&format!("axis {} {c}", names[i]), &mut axis[i][j]);
        }
    }
    row("hand radius", &mut rho);
    row("arc centre height", &mut cz);
    row("arc centre from the spot", &mut coff);
    row("swing start, s", &mut t0s);
    row("nominal contact, s", &mut tcs);
}
