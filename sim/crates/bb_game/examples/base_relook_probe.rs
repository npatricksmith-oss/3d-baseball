//! Diagnostic: the look a runner standing on a base he held takes when a throw is released or
//! the ball is secured (agent-models §6.4, "Looking again from a base"). Counts the looks, the
//! ones that sent him (by base, ball thrown or held) and how each sent runner ended the play:
//! scored or safe at the next base or beyond, back on his base, or put out. `-- [games]
//! [first seed]`; the standard sets are `300 0` and `300 1000003000`.
use bb_game::live::runner_trace;
use bb_game::play::FielderAttrs;
use bb_rules::Event;
use std::collections::BTreeMap;

#[derive(Default, Clone, Copy)]
struct Counts {
    looks: u64,
    waiting: u64,
    sent: u64,
    sent_thrown: u64,
    advanced: u64,
    back: u64,
    out_next: u64,
    out_other: u64,
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let games: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(300);
    let first: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let threads = 2u64;
    let chunk = games.div_ceil(threads);
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            std::thread::spawn(move || {
                // Per base looked from (1..=3).
                let mut c = [Counts::default(); 4];
                let mut lines = Vec::new();
                for seed in first + t * chunk..(first + (t + 1) * chunk).min(first + games) {
                    let fattrs: [FielderAttrs; 9] = core::array::from_fn(FielderAttrs::average);
                    let mut g = bb_game::game::Game::new(seed, false).with_fielders(fattrs);
                    loop {
                        let (step, ticks) = runner_trace::capture(|| g.step_pitch());
                        let looks = runner_trace::take_looks();
                        let Some(step) = step else { break };
                        // His first look that sent him, per runner.
                        let mut sent: BTreeMap<u16, (u8, bool)> = BTreeMap::new();
                        for l in &looks {
                            let b = l.base as usize;
                            c[b].looks += 1;
                            c[b].waiting += u64::from(l.waiting);
                            if l.went && !sent.contains_key(&l.runner) {
                                sent.insert(l.runner, (l.base, l.ball_thrown));
                            }
                        }
                        for (&runner, &(base, thrown)) in &sent {
                            let b = base as usize;
                            c[b].sent += 1;
                            c[b].sent_thrown += u64::from(thrown);
                            let out_ev = step.rules_events.iter().find_map(|e| match e {
                                Event::RunnerOut { runner: r, base, kind } if *r == runner => Some((*base, *kind)),
                                _ => None,
                            });
                            let out = out_ev.map(|o| o.0);
                            let reached = step.rules_events.iter().filter_map(|e| match e {
                                Event::RunnerAdvanced { runner: r, to, .. } if *r == runner => Some(*to),
                                Event::RunScored { runner: r } if *r == runner => Some(4),
                                _ => None,
                            }).max();
                            let how = match (out, reached) {
                                (Some(ob), _) if ob == base + 1 => {
                                    c[b].out_next += 1;
                                    "out at the next base"
                                }
                                (Some(_), _) => {
                                    c[b].out_other += 1;
                                    "out elsewhere"
                                }
                                (None, Some(to)) if to > base => {
                                    c[b].advanced += 1;
                                    "advanced"
                                }
                                _ => {
                                    c[b].back += 1;
                                    "stayed"
                                }
                            };
                            let t_look = looks.iter().find(|l| l.runner == runner && l.went).map_or(f64::NAN, |l| l.t);
                            lines.push(format!("seed {seed} play {} runner {runner:#06x} from {base} ({}) at {t_look:.2} s: {how} {out_ev:?}", step.play, if thrown { "throw" } else { "held" }));
                            if std::env::var("RELOOK_DUMP").is_ok() {
                                for e in &step.rules_events {
                                    lines.push(format!("    {e:?}"));
                                }
                                for tk in ticks.iter().filter(|tk| tk.runner == runner && ((tk.t * 60.0).round() as i64) % 15 == 0) {
                                    lines.push(format!("    tick t {:.2} s {:.1} v {:.1} goal {} {:?}", tk.t, tk.s, tk.signed_mps, tk.goal, tk.cause));
                                }
                                for f in &step.facts {
                                    let s = format!("{f:?}");
                                    if !s.starts_with("RunnerPosition") {
                                        lines.push(format!("    fact {s}"));
                                    }
                                }
                            }
                        }
                    }
                }
                (c, lines)
            })
        })
        .collect();
    let mut c = [Counts::default(); 4];
    let mut lines = Vec::new();
    for h in handles {
        let (m, l) = h.join().unwrap();
        lines.extend(l);
        for b in 0..4 {
            let (x, y) = (&mut c[b], m[b]);
            x.looks += y.looks;
            x.waiting += y.waiting;
            x.sent += y.sent;
            x.sent_thrown += y.sent_thrown;
            x.advanced += y.advanced;
            x.back += y.back;
            x.out_next += y.out_next;
            x.out_other += y.out_other;
        }
    }
    for l in &lines {
        println!("{l}");
    }
    println!("games {games} from seed {first}");
    for b in 1..=3 {
        let x = c[b];
        println!(
            "  from base {b}: looks {}, waiting {}, sent {} (throw in the air {}): advanced {}, back on his base {}, out at the next base {}, out elsewhere {}",
            x.looks, x.waiting, x.sent, x.sent_thrown, x.advanced, x.back, x.out_next, x.out_other
        );
    }
}
