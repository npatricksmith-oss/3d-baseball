//! Diagnostic: the batter-runner's overrun of first and the rules calls that position
//! samples make possible (rules-spec §5.5). Per batted ball: did the batter-runner touch
//! first, leave it, and touch it again with no other base between (an overrun and return)?
//! Also counts passing outs (`RunnerPassed`) and every rules warning. `-- [games] [first seed]`
use bb_rules::{Event, Input, Touch};
use std::collections::BTreeMap;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let games: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(256);
    let first: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let (mut bip, mut reached_first, mut overruns, mut went_on, mut ended_off) = (0u64, 0u64, 0u64, 0u64, 0u64);
    let (mut passed, mut stayed_out, mut stayed_other, mut other, mut ended_off_third, mut ended_off_out) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    let mut ended_off_dead = 0u64;
    let mut warnings: BTreeMap<&'static str, u64> = BTreeMap::new();
    for seed in first..first + games {
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            for e in &step.rules_events {
                match e {
                    Event::RunnerPassed { .. } => passed += 1,
                    Event::RulesWarning { code } => *warnings.entry(code).or_default() += 1,
                    _ => {}
                }
            }
            if step.record.batted.is_none() {
                continue;
            }
            let Some(batter) = step.rules_events.iter().find_map(|e| match e {
                Event::PlateAppearanceEnded { batter, .. } => Some(*batter),
                _ => None,
            }) else {
                continue;
            };
            bip += 1;
            // The batter-runner's base facts, in order: (touched?, base).
            let seq: Vec<(bool, u8)> = step
                .facts
                .iter()
                .filter_map(|f| match *f {
                    Input::BaseTouched { agent, base, holding_ball: false, .. } if agent == batter => Some((true, base)),
                    Input::BaseLeft { agent, base, .. } if agent == batter => Some((false, base)),
                    _ => None,
                })
                .collect();
            if !seq.contains(&(true, 1)) {
                continue;
            }
            reached_first += 1;
            let i = seq.iter().position(|&x| x == (true, 1)).unwrap();
            let after = &seq[i + 1..(i + 3).min(seq.len())];
            match after {
                [(false, 1), (true, 1)] => overruns += 1,
                [(false, 1), (true, b)] if *b > 1 => went_on += 1,
                [(false, 1)] => {
                    ended_off += 1;
                    // What ended it: his own out (a fly caught while he was past the bag), the
                    // third out, the game, or none of these (a bug to look at).
                    if step.rules_events.iter().any(|e| matches!(e, Event::RunnerOut { runner, .. } if *runner == batter)) {
                        ended_off_out += 1;
                    } else if step.rules_events.iter().any(|e| matches!(e, Event::HalfInningEnded { .. } | Event::GameEnded { .. })) {
                        ended_off_third += 1;
                    } else if step.facts.iter().any(|f| matches!(f, Input::BallTouched { what: Touch::OutOfPlay { .. }, .. })) {
                        ended_off_dead += 1;
                    } else if ended_off - ended_off_third - ended_off_out - ended_off_dead <= 3 {
                        println!("ended off the bag, no third out: seed {seed} play {}", step.play);
                    }
                }
                [] => {
                    // Touched first and stayed on it: out later in the play (a caught fly),
                    // or the play ended with him on the bag.
                    if step.rules_events.iter().any(|e| matches!(e, Event::RunnerOut { runner, .. } if *runner == batter)) {
                        stayed_out += 1;
                    } else {
                        stayed_other += 1;
                    }
                }
                _ => other += 1,
            }
        }
    }
    println!("{games} games from seed {first}: {bip} batted balls ending a PA");
    println!("batter-runner touched first: {reached_first}");
    println!("  overran and came back:      {overruns}");
    println!("  left first for second+:      {went_on}");
    println!("  left first, play ended off: {ended_off} (he was put out: {ended_off_out}; third out elsewhere or game end: {ended_off_third}; ball out of play: {ended_off_dead})");
    println!("  never left it, put out:     {stayed_out}");
    println!("  never left it, not out:     {stayed_other}");
    println!("  anything else:              {other}");
    println!("passing outs (RunnerPassed): {passed}");
    println!("rules warnings: {warnings:?}");
}
