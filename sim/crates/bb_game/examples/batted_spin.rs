//! Batted-ball spin by launch angle in games (the "typical spin" fielders assume, agent-models §5.1).
fn main() {
    let games: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(48);
    let mut bins: Vec<Vec<f64>> = vec![Vec::new(); 9];
    for seed in 0..games {
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            let (Some((_, la, _)), Some((_, b))) = (step.record.batted, step.record.contact_state) else { continue };
            let _ = b;
            let Some(path) = &step.batted else { continue };
            let Some(bb_physics::ground::Segment::Flight { tr, .. }) = path.segments.first() else { continue };
            let s0 = tr.first().s;
            // Backspin component: spin about the horizontal axis perpendicular to the flight direction (+ = backspin).
            let h = bb_math::Vec3::new(s0.v.x, s0.v.y, 0.0).normalized();
            let axis = bb_math::Vec3::new(h.y, -h.x, 0.0);
            let back = s0.w.dot(axis) * 30.0 / std::f64::consts::PI;
            let k = (((la.to_degrees() + 30.0) / 10.0).floor() as isize).clamp(0, 8) as usize;
            bins[k].push(back);
        }
    }
    for (k, v) in bins.iter().enumerate() {
        let mut v = v.clone();
        v.sort_by(|a, b| a.total_cmp(b));
        if v.is_empty() { continue; }
        println!("LA {:+3}..{:+3}: n {:5}  backspin rpm median {:6.0}  quartiles {:6.0} {:6.0}", -30 + 10 * k as i32, -20 + 10 * k as i32, v.len(), v[v.len() / 2], v[v.len() / 4], v[3 * v.len() / 4]);
    }
}
