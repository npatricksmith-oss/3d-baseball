//! Diagnostic (day-fixes review #1): plays where a runner touched home before a fly was
//! caught, and what became of him: back to retouch and home again, out on appeal, or still
//! scored with no retouch. `-- <games> [first seed]`
use bb_rules::{Base, Event, Input, OutKind};

fn main() {
    let a: Vec<u64> = std::env::args().skip(1).map(|s| s.parse().unwrap()).collect();
    let games = a.first().copied().unwrap_or(200);
    let seed0 = a.get(1).copied().unwrap_or(0);
    let (mut plays, mut rescored, mut held, mut appealed, mut stayed, mut inning_over) = (0, 0, 0, 0, 0, 0);
    for seed in seed0..seed0 + games {
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            let Some(t_catch) = step.facts.iter().find_map(|f| match f {
                Input::GloveSecured { t, .. } => Some(*t),
                _ => None,
            }) else {
                continue;
            };
            if !step.rules_events.iter().any(|e| matches!(e, Event::Catch { .. })) {
                continue;
            }
            let early: Vec<_> = step
                .facts
                .iter()
                .filter_map(|f| match f {
                    Input::BaseTouched { t, agent, base: 4, holding_ball: false } if *t < t_catch => Some(*agent),
                    _ => None,
                })
                .collect();
            // A catch for the third out ends the inning: no run, and nobody owes a retouch.
            let third_out = step.rules_events.iter().any(|e| matches!(e, Event::HalfInningEnded { .. } | Event::GameEnded { .. }));
            for id in early {
                if third_out {
                    inning_over += 1;
                    continue;
                }
                plays += 1;
                let touched = |b: Base| step.facts.iter().any(|f| matches!(f, Input::BaseTouched { t, agent, base, holding_ball: false } if *agent == id && *base == b && *t > t_catch));
                let out = step.rules_events.iter().any(|e| matches!(e, Event::RunnerOut { runner, kind: OutKind::Appeal, .. } if *runner == id));
                let scored = step.rules_events.iter().any(|e| matches!(e, Event::RunScored { runner } if *runner == id));
                let what = if out {
                    appealed += 1;
                    "out on appeal"
                } else if touched(4) && scored {
                    rescored += 1;
                    "retouched, scored"
                } else if !scored && step.facts.iter().any(|f| matches!(f, Input::BaseTouched { agent, holding_ball: false, t, .. } if *agent == id && *t > t_catch)) {
                    held += 1;
                    "retouched, held"
                } else {
                    stayed += 1;
                    "no retouch"
                };
                println!("seed {seed} play {} runner {id}: catch {t_catch:.2} s, {what} (scored {scored}, home again {})", step.play, touched(4));
            }
        }
    }
    println!("{inning_over} runners home before a third-out catch (no run, no retouch owed)");
    println!("{plays} runners home before a catch with the inning still on: {rescored} retouched and scored, {held} retouched and held, {appealed} out on appeal, {stayed} no retouch");
}
