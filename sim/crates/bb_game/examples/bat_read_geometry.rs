//! What a fielder has to read the bat's angle at contact from (agent-models §5.10): for each
//! fair ball in play, the rate at which the bat's horizontal angle (yaw ψ = atan2(axis.x,
//! axis.y)) is sweeping at the contact instant, the bat's tilt out of the horizontal, and how
//! far from the swing's vertical axis the contact point is; and, for each fielder at his
//! standard position, his distance from the contact point and how much of the barrel's
//! sideways motion under a change of yaw he sees across his line of sight (1: all of it; 0:
//! the barrel moves straight toward or away from him). The numbers behind `BAT_YAW_RATE_DPS`
//! and the viewing term in `live.rs`.
//! `cargo run --release -p bb_game --example bat_read_geometry -- [games]`
use bb_math::Vec3;
use bb_physics::contact::BatPose;
use bb_rules::Event;

fn pct(v: &mut Vec<f64>, q: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[((v.len() - 1) as f64 * q).round() as usize]
}

fn main() {
    let games: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(32);
    let homes = bb_game::play::home_positions();
    let names = ["P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"];
    // Fielder eye height, m (assumption: a set fielder, slightly crouched).
    let eye = 1.5;
    let (mut rate, mut tilt, mut rad) = (Vec::new(), Vec::new(), Vec::new());
    let mut vis: Vec<Vec<f64>> = vec![Vec::new(); 9];
    let mut dist: Vec<Vec<f64>> = vec![Vec::new(); 9];
    for seed in 0..games {
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            if step.record.batted.is_none() || !step.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: true })) {
                continue;
            }
            let (Some(sw), Some((t_c, st))) = (step.record.swing, step.record.contact_state) else { continue };
            let yaw = |t: f64| {
                let a = sw.at(t).axis;
                bb_math::m::atan2(a.x, a.y)
            };
            let h = 1e-5;
            rate.push(((yaw(t_c + h) - yaw(t_c - h)) / (2.0 * h)).abs().to_degrees());
            let a = sw.at(t_c).axis;
            tilt.push(bb_math::m::asin(a.z).to_degrees());
            let c = st.r;
            let (dx, dy) = (c.x - sw.pivot.x, c.y - sw.pivot.y);
            rad.push((dx * dx + dy * dy).sqrt());
            // A change of yaw moves the barrel horizontally, at right angles to the bat's
            // horizontal direction.
            let ah = Vec3::new(a.x, a.y, 0.0).normalized();
            let u = Vec3::new(ah.y, -ah.x, 0.0);
            for i in 2..9 {
                let e = (Vec3::new(homes[i].x, homes[i].y, eye) - c).normalized();
                let along = u.dot(e);
                vis[i].push((1.0 - along * along).max(0.0).sqrt());
                dist[i].push((Vec3::new(homes[i].x, homes[i].y, eye) - c).norm());
            }
        }
    }
    let n = rate.len();
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let sd = |v: &[f64]| {
        let m = mean(v);
        (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / v.len() as f64).sqrt()
    };
    println!("{n} fair balls in play ({games} games)");
    println!("yaw rate at contact, deg/s: mean {:.0} sd {:.0}; p10 {:.0} p50 {:.0} p90 {:.0}", mean(&rate), sd(&rate), pct(&mut rate, 0.1), pct(&mut rate, 0.5), pct(&mut rate, 0.9));
    println!("bat tilt at contact (deg, + = tip up): mean {:+.1} sd {:.1}", mean(&tilt), sd(&tilt));
    println!("contact point's horizontal distance from the swing pivot, m: mean {:.2} sd {:.2}", mean(&rad), sd(&rad));
    println!("pos  dist m   visible fraction: mean  p10  p50  p90");
    for i in 2..9 {
        let d = mean(&dist[i]);
        let m = mean(&vis[i]);
        let v = &mut vis[i];
        println!("{:>3}  {:6.1}   {:.2}  {:.2}  {:.2}  {:.2}", names[i], d, m, pct(v, 0.1), pct(v, 0.5), pct(v, 0.9));
    }
}
