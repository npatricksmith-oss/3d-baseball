//! Fly outs with a runner on third and fewer than two out: what the runner does.
//!   cargo run --release -p bb_game --example sf_probe -- <first seed> <games> [verbose]
use bb_game::game::Game;
use bb_rules::{Event, Input, OutKind};
use std::collections::BTreeMap;

/// Mean of a list (NaN when empty).
fn mean(x: &[f64]) -> f64 {
    x.iter().sum::<f64>() / x.len() as f64
}

fn main() {
    let a: Vec<u64> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    let (first, n) = (a[0], a[1]);
    let verbose = std::env::args().any(|x| x == "v");
    let third = bb_game::play::base_pos(3);
    let (mut fly, mut with3, mut scored, mut out, mut held) = (0, 0, 0, 0, 0);
    // The race on these plays (agent-models §6.4): by catch depth (m from home, 15 m bins),
    // outcome counts; and the times from the catch of each leg.
    let mut by_depth: BTreeMap<u32, [u32; 3]> = BTreeMap::new();
    let (mut run_home, mut of_release, mut flight, mut catcher_to_home_touch) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for seed in first..first + n {
        let mut g = Game::new(seed, true);
        let mut outs = 0u32;
        while let Some(p) = g.step_pitch() {
            let outs_before = outs;
            for e in &p.rules_events {
                match e {
                    Event::HalfInningStarted { .. } => outs = 0,
                    Event::RunnerOut { .. } => outs += 1,
                    _ => {}
                }
            }
            let batter_fly = p.rules_events.iter().any(|e| matches!(e, Event::RunnerOut { kind: OutKind::FlyOut, .. }));
            if !batter_fly { continue; }
            fly += 1;
            if std::env::var("DBG").is_ok() && p.viz.standing.len() > 11 { println!("{:?} outs_before {outs_before}", p.viz.standing); }
            if outs_before >= 2 { continue; }
            // The runner standing on third at the pitch.
            let id = p.viz.standing.iter().filter(|(id, _)| *id != p.batter && (*id & 0x80) == 0)
                .map(|(id, pos)| (*id, (*pos - third).norm())).filter(|x| x.1 < 6.0).min_by(|x, y| x.1.total_cmp(&y.1)).map(|x| x.0);
            let Some(id) = id else { continue };
            with3 += 1;
            let sc = p.rules_events.iter().any(|e| matches!(e, Event::RunScored { runner } if *runner == id));
            let ot = p.rules_events.iter().any(|e| matches!(e, Event::RunnerOut { runner, .. } if *runner == id));
            let what = if sc { scored += 1; "scored" } else if ot { out += 1; "OUT" } else { held += 1; "held" };
            // The catch: the first glove securing; its depth from home.
            if let Some((t_catch, depth)) = p.facts.iter().find_map(|f| match *f {
                Input::GloveSecured { t, x, y, .. } => Some((t, x.hypot(y))),
                _ => None,
            }) {
                let bin = (depth / 15.0) as u32 * 15;
                by_depth.entry(bin).or_default()[if sc { 0 } else if ot { 1 } else { 2 }] += 1;
                let home = p.facts.iter().find_map(|f| match *f {
                    Input::BaseTouched { t, agent, base: 4, holding_ball: false } if agent == id => Some(t),
                    _ => None,
                });
                if sc || ot {
                    if let Some(h) = home {
                        run_home.push(h - t_catch);
                    }
                    if let Some(&(r0, _)) = p.viz.released.first() {
                        of_release.push(r0 - t_catch);
                        if let Some(&(c, _)) = p.viz.secured.iter().find(|s| s.0 > r0 && s.1 & 0x7f == 2) {
                            flight.push(c - r0);
                            if let Some(h) = home {
                                catcher_to_home_touch.push(h - c);
                            }
                        }
                    }
                }
            }
            if verbose { println!("seed {seed} play {} outs {outs_before} runner {id} {what}", p.play); }
        }
    }
    println!("{n} games from {first}: fly outs {fly}; with runner on third and <2 out {with3}: scored {scored}, put out {out}, held {held}");
    println!("by catch depth (m from home): scored / put out / held");
    for (bin, c) in &by_depth {
        println!("  {bin:3}-{:3} m  {:3} {:3} {:3}", bin + 15, c[0], c[1], c[2]);
    }
    println!("runner went: catch to home touch {:.2} s (n {}); catch to outfielder's release {:.2} s (n {}); release to the catcher's catch {:.2} s (n {}); catcher's catch to the runner's touch {:+.2} s (n {})",
        mean(&run_home), run_home.len(), mean(&of_release), of_release.len(), mean(&flight), flight.len(), mean(&catcher_to_home_touch), catcher_to_home_touch.len());
}
