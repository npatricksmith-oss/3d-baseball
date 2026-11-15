//! Diagnostic: games in a seed range whose rules engine warned, and on which play.
//! `-- <games> <first seed>`
use bb_rules::Event;
fn main() {
    let a: Vec<u64> = std::env::args().skip(1).map(|s| s.parse().unwrap()).collect();
    let (n, first) = (a[0], a[1]);
    std::thread::scope(|sc| {
        let hs: Vec<_> = (0..2u64)
            .map(|t| {
                sc.spawn(move || {
                    for seed in first + t * n.div_ceil(2)..(first + (t + 1) * n.div_ceil(2)).min(first + n) {
                        let mut g = bb_game::game::Game::new(seed, false).with_fielders(core::array::from_fn(bb_game::play::FielderAttrs::average));
                        while let Some(step) = g.step_pitch() {
                            for e in &step.rules_events {
                                if let Event::RulesWarning { code } = e {
                                    println!("seed {seed} play {} warning {code}", step.play);
                                }
                            }
                        }
                    }
                })
            })
            .collect();
        for h in hs {
            h.join().unwrap();
        }
    });
}
