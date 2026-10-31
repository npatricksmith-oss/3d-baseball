//! Ground balls over many games: through the infield vs infield hits vs outs.
use bb_rules::{Event, Input};
fn main() {
    let (mut through, mut infield_hit, mut out, mut n) = (0, 0, 0, 0);
    let mut ih_ev = Vec::new();
    let mut ih_la = Vec::new();
    for seed in 0..24u64 {
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            let Some((ev, la, _)) = step.record.batted else { continue };
            if la.to_degrees() >= 10.0 || !step.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: true })) {
                continue;
            }
            n += 1;
            let batter_out = step.rules_events.iter().any(|e| matches!(e, Event::RunnerOut { .. }));
            let first = step.facts.iter().find_map(|f| if let Input::GloveSecured { fielder, .. } = f { Some(fielder & 0x7F) } else { None });
            if batter_out {
                out += 1;
            } else if first.map(|p| p >= 7).unwrap_or(true) {
                through += 1;
            } else {
                infield_hit += 1;
                ih_ev.push(ev / 0.44704);
                ih_la.push(la.to_degrees());
            }
        }
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    println!("{n} fair grounders: out {:.0}%, through to OF {:.0}%, infield hit {:.0}% (mean EV {:.0} mph, LA {:.0})", 100.0 * out as f64 / n as f64, 100.0 * through as f64 / n as f64, 100.0 * infield_hit as f64 / n as f64, mean(&ih_ev), mean(&ih_la));
}
