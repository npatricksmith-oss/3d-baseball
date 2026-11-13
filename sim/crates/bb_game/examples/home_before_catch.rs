//! Diagnostic (day-fixes review #1): plays where a runner touched home before a fly was
//! caught, and what became of him: back to retouch and home again, back and holding, out
//! on appeal, or no retouch. `-- <games> [first seed]`
use bb_rules::{Event, Input, OutKind};

fn main() {
    let a: Vec<u64> = std::env::args().skip(1).map(|s| s.parse().unwrap()).collect();
    let games = a.first().copied().unwrap_or(200);
    let seed0 = a.get(1).copied().unwrap_or(0);
    let (mut plays, mut rescored, mut held, mut appealed, mut stayed, mut inning_over) = (0, 0, 0, 0, 0, 0);
    for seed in seed0..seed0 + games {
        let mut g = bb_game::game::Game::new(seed, false);
        loop {
            let outs_before = g.st.outs;
            let Some(step) = g.step_pitch() else { break };
            let Some(t_catch) = step.facts.iter().find_map(|f| match f {
                Input::GloveSecured { t, .. } => Some(*t),
                _ => None,
            }) else {
                continue;
            };
            // Which out the catch was: count the outs on this play in order (runner-return
            // review #3: an inning ending later in the play is not a third-out catch).
            let mut n = outs_before;
            let mut catch_out = None;
            for e in &step.rules_events {
                if let Event::RunnerOut { kind, .. } = e {
                    n += 1;
                    if *kind == OutKind::FlyOut && catch_out.is_none() {
                        catch_out = Some(n);
                    }
                }
            }
            let Some(catch_out) = catch_out else { continue };
            let early: Vec<_> = step
                .facts
                .iter()
                .filter_map(|f| match f {
                    Input::BaseTouched { t, agent, base: 4, holding_ball: false } if *t < t_catch => Some(*agent),
                    _ => None,
                })
                .collect();
            for id in early {
                // A catch for the third out ends the inning: no run, and nobody owes a retouch.
                if catch_out >= 3 {
                    inning_over += 1;
                    continue;
                }
                plays += 1;
                // His start base: the first base he left (only bases 1-3 report leaving).
                let start = step.facts.iter().find_map(|f| match f {
                    Input::BaseLeft { agent, base, .. } if *agent == id => Some(*base),
                    _ => None,
                });
                let touched_after = |b: u8| step.facts.iter().any(|f| matches!(f, Input::BaseTouched { t, agent, base, holding_ball: false } if *agent == id && *base == b && *t > t_catch));
                let retouched = start.map(touched_after).unwrap_or(false);
                let out = step.rules_events.iter().any(|e| matches!(e, Event::RunnerOut { runner, kind: OutKind::Appeal, .. } if *runner == id));
                let scored = step.rules_events.iter().any(|e| matches!(e, Event::RunScored { runner } if *runner == id));
                let what = if out {
                    appealed += 1;
                    "out on appeal"
                } else if retouched && touched_after(4) && scored {
                    rescored += 1;
                    "retouched, scored"
                } else if retouched && !scored {
                    held += 1;
                    "retouched, held"
                } else {
                    stayed += 1;
                    "no retouch"
                };
                println!("seed {seed} play {} runner {id} from {start:?}: catch {t_catch:.2} s (out {catch_out}), {what} (scored {scored}, home again {})", step.play, touched_after(4));
            }
        }
    }
    println!("{inning_over} runners home before a third-out catch (no run, no retouch owed)");
    println!("{plays} runners home before a catch with the inning still on: {rescored} retouched and scored, {held} retouched and held, {appealed} out on appeal, {stayed} no retouch");
}
