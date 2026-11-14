//! Where balls in play go by where the pitch crossed the plate (and by pitch type): the
//! pattern fielders anticipate from (agent-models §5.10). Spray > 0 is toward right field
//! for a right-handed batter; left-handed batters are mirrored in x.
//! `cargo run --release -p bb_game --example spray_by_location -- [games]`
use bb_rules::Event;
fn main() {
    let games: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(64);
    // plate_x bins, 10 cm wide from -0.35 m (catcher's view; +x toward first base).
    let mut bins: Vec<Vec<f64>> = vec![Vec::new(); 7];
    let mut by_type: std::collections::BTreeMap<String, Vec<f64>> = Default::default();
    let mut all: Vec<f64> = Vec::new();
    for seed in 0..games {
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            let Some((_, _, spray)) = step.record.batted else { continue };
            if !step.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: true })) {
                continue;
            }
            let Some(p) = step.record.plate.or(step.record.contact_state.map(|c| c.1.r)) else { continue };
            // A left-handed batter (hand-arc center on +x) is mirrored in x, so every row reads
            // as a right-hander's: −x inside, negative spray pulled.
            let mir = if step.record.swing.is_some_and(|s| s.pivot.x > 0.0) { -1.0 } else { 1.0 };
            let (px, spray) = (mir * p.x, mir * spray);
            let k = (((px + 0.35) / 0.10).floor() as isize).clamp(0, 6) as usize;
            bins[k].push(spray.to_degrees());
            all.push(spray.to_degrees());
            by_type.entry(format!("{:?}", step.record.ty)).or_default().push(spray.to_degrees());
        }
    }
    let stats = |v: &[f64]| {
        let n = v.len().max(1) as f64;
        let mean = v.iter().sum::<f64>() / n;
        let sd = (v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / n).sqrt();
        // Thirds of fair territory for a right-handed batter: pull < -15°, oppo > +15°.
        let pull = v.iter().filter(|&&x| x < -15.0).count() as f64 / n;
        let oppo = v.iter().filter(|&&x| x > 15.0).count() as f64 / n;
        (v.len(), mean, sd, pull, oppo)
    };
    let (n, mean, sd, _, _) = stats(&all);
    println!("all: n {n} mean spray {mean:+.2} sd {sd:.2} (batter-relative: left-handers mirrored)");
    println!("plate_x bin (m, as a right-hander's)   n    mean spray  sd   pull%  oppo%");
    for (k, v) in bins.iter().enumerate() {
        let (n, mean, sd, pull, oppo) = stats(v);
        println!("{:+.2}..{:+.2}  {:5}  {:+6.1}  {:5.1}  {:4.0}  {:4.0}", -0.35 + 0.1 * k as f64, -0.25 + 0.1 * k as f64, n, mean, sd, 100.0 * pull, 100.0 * oppo);
    }
    println!("by pitch type:");
    for (t, v) in &by_type {
        let (n, mean, sd, pull, oppo) = stats(v);
        println!("{t:4} {n:5}  {mean:+6.1}  {sd:5.1}  {:4.0}  {:4.0}", 100.0 * pull, 100.0 * oppo);
    }
}
