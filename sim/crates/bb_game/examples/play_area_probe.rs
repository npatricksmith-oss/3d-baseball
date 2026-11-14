//! Where balls leave the playing field (A10, physics-spec §9.5): by kind (foul, home run,
//! after a throw), by the wall they leave over (fence, backstop, side wall, dugout, taper),
//! and for throws, by the thrower's position.
//!
//! `cargo run --release -p bb_game --example play_area_probe -- <games> <seed set> [list]`
//! (with `list` the second argument is the first game seed itself, as `sim-cli game --seed` and
//! the client's `-BbSeed` take it)

use std::collections::BTreeMap;

use bb_game::game::Game;
use bb_physics::ground::{Field, WallKind};
use bb_rules::{Event, Input, Touch};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(300);
    let set: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    // `list`: print each ball out of play (seed, play index, time) to find one to watch.
    let list = args.get(3).is_some_and(|a| a == "list");
    let field = Field::default();
    let mut by: BTreeMap<(String, String), u32> = BTreeMap::new();
    let mut throwers: BTreeMap<String, u32> = BTreeMap::new();
    for i in 0..n {
        let seed = if list { set + i } else { set * 1_000_003 + i };
        let mut g = Game::new(seed, false);
        while let Some(s) = g.step_pitch() {
            let Some((t, x, y)) = s.facts.iter().find_map(|f| match f {
                Input::BallTouched { t, what: Touch::OutOfPlay { x, y, .. } } => Some((*t, *x, *y)),
                _ => None,
            }) else {
                continue;
            };
            let thrower = s.facts.iter().filter_map(|f| match f {
                Input::BallReleasedFromGlove { t: tr, fielder, .. } if *tr <= t => Some(*fielder),
                _ => None,
            });
            let thrower = thrower.last();
            let kind = if thrower.is_some() {
                "throw"
            } else if s.rules_events.iter().any(|e| matches!(e, Event::BallDead { reason: bb_rules::DeadBall::HomeRun })) {
                "home run"
            } else if s.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: false })) {
                "foul"
            } else {
                "fair, bounced"
            };
            // The wall it left over: the piece nearest where it ended, a dugout by its place.
            let w = field.body_clearance(x, y, 0.0);
            let wall = match w.kind {
                WallKind::Side if field.wall_top(&w) == 0.0 => "dugout".to_string(),
                k => format!("{k:?}"),
            };
            if list {
                println!("seed {seed} play {} t {t:.2} {kind} over {wall} at ({x:.1}, {y:.1})", s.play);
            }
            *by.entry((kind.to_string(), wall)).or_default() += 1;
            if let Some(f) = thrower {
                let pos = ["P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"][((f & 0x7F) as usize).saturating_sub(1).min(8)];
                *throwers.entry(pos.to_string()).or_default() += 1;
            }
        }
    }
    println!("{n} games, seed set {set}: balls out of play per game");
    for ((k, w), c) in &by {
        println!("  {k:14} {w:9} {:.3}", *c as f64 / n as f64);
    }
    println!("throws out of play by thrower (count): {throwers:?}");
}
