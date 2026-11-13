//! Where balls in play go by the bat's visible angle at contact: the pattern fielders read
//! the swing from (agent-models §5.10). For each fair ball in play, the bat's horizontal
//! direction at the actual contact instant (yaw ψ = atan2(axis.x, axis.y), knob → tip) gives
//! the "face" angle, the horizontal direction the barrel squares to: ψ − 90° for a
//! right-handed batter (mirrored for a left-hander, so negative = his pull side). Prints the
//! spray of balls in play by face-angle bin, and a least-squares line spray = a + b·face with
//! its residual σ: the fit a fielder's belief uses (`SWING_SPRAY_*` in `live.rs`).
//! `cargo run --release -p bb_game --example spray_by_bat_angle -- [games]`
use bb_physics::contact::BatPose;
use bb_rules::Event;
fn main() {
    let games: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(64);
    // (face, spray) in degrees, batter-relative: negative = pull side.
    let mut pts: Vec<(f64, f64)> = Vec::new();
    for seed in 0..games {
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            let Some((_, _, spray)) = step.record.batted else { continue };
            if !step.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: true })) {
                continue;
            }
            let (Some(sw), Some((t_c, _))) = (step.record.swing, step.record.contact_state) else { continue };
            let a = sw.at(t_c).axis;
            // Right-handed batters only stand on −x (every game batter so far); a left-hander's
            // swing is mirrored in x: knob on +x side.
            let lefty = sw.pivot.x > 0.0;
            let (ax, sp) = if lefty { (-a.x, -spray) } else { (a.x, spray) };
            let yaw = bb_math::m::atan2(ax, a.y).to_degrees();
            pts.push((yaw - 90.0, sp.to_degrees()));
        }
    }
    let n = pts.len() as f64;
    let mx = pts.iter().map(|p| p.0).sum::<f64>() / n;
    let my = pts.iter().map(|p| p.1).sum::<f64>() / n;
    let sxx = pts.iter().map(|p| (p.0 - mx) * (p.0 - mx)).sum::<f64>();
    let sxy = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum::<f64>();
    let syy = pts.iter().map(|p| (p.1 - my) * (p.1 - my)).sum::<f64>();
    let b = sxy / sxx;
    let a = my - b * mx;
    let res = (pts.iter().map(|p| (p.1 - a - b * p.0) * (p.1 - a - b * p.0)).sum::<f64>() / n).sqrt();
    println!("{} fair balls in play; face mean {:+.1}° sd {:.1}°; spray mean {:+.1}° sd {:.1}°", pts.len(), mx, (sxx / n).sqrt(), my, (syy / n).sqrt());
    println!("fit: spray = {a:+.2}° + {b:.3} × face; residual sd {res:.1}°; r = {:.3}", sxy / (sxx * syy).sqrt());
    println!("face bin (deg)     n   mean spray   sd");
    let mut lo = -60.0;
    while lo < 60.0 {
        let v: Vec<f64> = pts.iter().filter(|p| p.0 >= lo && p.0 < lo + 10.0).map(|p| p.1).collect();
        if !v.is_empty() {
            let m = v.iter().sum::<f64>() / v.len() as f64;
            let sd = (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / v.len() as f64).sqrt();
            println!("{:+4.0}..{:+4.0}  {:6}   {:+6.1}   {:5.1}", lo, lo + 10.0, v.len(), m, sd);
        }
        lo += 10.0;
    }
    let out = pts.iter().filter(|p| p.0 < -60.0 || p.0 >= 60.0).count();
    println!("outside ±60°: {out}");
}
