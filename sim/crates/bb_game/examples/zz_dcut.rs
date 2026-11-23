//! Double-cut probe (throwaway, untracked; C lines: throw choices). Review probe (mid-infield jobs; throwaway, untracked). Per batted ball: one P line on
//! stderr after the play (seed, play, bases mask at the pitch, outs, first down, first secure,
//! outs on the play). Run with BB_COVER_TRACE=1 to interleave the R job lines. `-- games first`
use bb_game::game::Game;
use bb_rules::{Event, Input, OutKind, Touch};
fn main() {
    let a: Vec<u64> = std::env::args().skip(1).map(|s| s.parse().unwrap()).collect();
    let (games, first) = (a[0], a[1]);
    for seed in first..first + games {
        let mut g = Game::new(seed, false);
        loop {
            let bases = g.st.bases;
            let outs = g.st.outs;
            let (step, choices) = bb_game::live::choice_trace::capture(|| g.step_pitch());
            let Some(step) = step else { break };
            for c in &choices { if c.action == "throw" || c.action == "return" { eprintln!("C,{seed},{},{:.3},{},{},{},{},{:.1}", step.play, c.t, c.holder & 0x7f, c.action, c.base, c.through as u8, c.from.x.hypot(c.from.y)); } }
            if step.batted.is_none() { continue; }
            let mask = bases.iter().enumerate().map(|(i, b)| if b.is_some() { 1 << i } else { 0 }).sum::<u8>();
            let mut down: Option<(f64, f64, f64)> = None;
            let mut sec: Option<(f64, u16, f64, f64)> = None;
            for f in &step.facts {
                match *f {
                    Input::BallTouched { t, what: Touch::Ground { x, y } | Touch::Wall { x, y } } if down.is_none() && sec.is_none() => down = Some((t, x, y)),
                    Input::GloveSecured { t, fielder, x, y } if sec.is_none() => sec = Some((t, fielder & 0x7f, x, y)),
                    _ => {}
                }
            }
            let mut outs_s = String::new();
            for e in &step.rules_events {
                if let Event::RunnerOut { runner, base, kind } = e {
                    let k = match kind { OutKind::Force => "F", OutKind::Tag => "T", OutKind::FlyOut => "A", OutKind::Strikeout => "K", OutKind::Appeal => "P" };
                    let who = if *runner == step.batter { "b".to_string() } else { bases.iter().position(|x| *x == Some(*runner)).map(|i| format!("r{}", i + 1)).unwrap_or("?".into()) };
                    outs_s.push_str(&format!("{who}{k}{base};"));
                }
            }
            let reached = |id: u16| step.facts.iter().filter_map(|f| match *f { Input::BaseTouched { agent, base, holding_ball: false, .. } if agent == id => Some(base), _ => None }).max().unwrap_or(0);
            let rr: Vec<String> = bases.iter().map(|b| b.map(|id| { let sc = step.rules_events.iter().any(|e| matches!(e, Event::RunScored { runner, .. } if *runner == id)); if sc { "4".to_string() } else { reached(id).to_string() } }).unwrap_or("-".into())).collect();
            let (dt, dx, dy) = down.unwrap_or((-1.0, f64::NAN, f64::NAN));
            let (st, sf, sx, sy) = sec.unwrap_or((-1.0, 0, f64::NAN, f64::NAN));
            eprintln!("P,{seed},{},{mask},{outs},{dt:.3},{dx:.2},{dy:.2},{st:.3},{sf},{sx:.2},{sy:.2},{},{outs_s},{}", step.play, reached(step.batter), rr.join("/"));
        }
    }
}
