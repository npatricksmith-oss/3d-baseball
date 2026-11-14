//! Fly outs with a runner on third and fewer than two out: what the runner does.
//!   cargo run --release -p bb_game --example sf_probe -- <first seed> <games> [verbose]
use bb_game::game::Game;
use bb_rules::{Event, OutKind};

fn main() {
    let a: Vec<u64> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    let (first, n) = (a[0], a[1]);
    let verbose = std::env::args().any(|x| x == "v");
    let third = bb_game::play::base_pos(3);
    let (mut fly, mut with3, mut scored, mut out, mut held) = (0, 0, 0, 0, 0);
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
            if verbose { println!("seed {seed} play {} outs {outs_before} runner {id} {what}", p.play); }
        }
    }
    println!("{n} games from {first}: fly outs {fly}; with runner on third and <2 out {with3}: scored {scored}, put out {out}, held {held}");
}
