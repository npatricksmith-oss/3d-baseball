//! Ground balls that become hits: how close the nearest infielder came to the ball while it
//! was low and inside the infield, versus grounders turned into outs. Separates positioning
//! (nobody near) from range (a few metres short) from glove work (within reach, missed).
//! Usage: `cargo run --release -p bb_game --example gb_through -- [games]`
use bb_game::game::Game;
use bb_game::play::home_positions;
use bb_rules::{Event, Input};

fn main() {
    let games: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(48);
    // Buckets of the nearest infielder's closest approach, m.
    let edges = [1.0, 1.5, 2.5, 4.0, 7.0, f64::INFINITY];
    let mut hist = [[0u32; 6]; 3]; // through, infield hit, out
    let mut ev_sum = [0.0; 3];
    let mut n = [0u32; 3];
    let mut touched_through = 0;
    // Outcome class by exit-speed bin (mph edges).
    let ev_edges = [80.0, 90.0, 95.0, 100.0, 105.0, f64::INFINITY];
    let mut by_ev = [[0u32; 3]; 6];
    // Routine grounders (calibration-targets §4.6): under 95 mph and passing within 3 m of an
    // infielder's starting position while low; success = an out on the play.
    let (mut routine, mut routine_out) = (0u32, 0u32);
    let home = home_positions();
    for seed in 0..games {
        let mut g = Game::new(seed, true);
        while let Some(st) = g.step_pitch() {
            let Some((ev, la, _)) = st.record.batted else { continue };
            if la.to_degrees() >= 10.0 || !st.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: true })) {
                continue;
            }
            let outs = st.rules_events.iter().filter(|e| matches!(e, Event::RunnerOut { .. })).count();
            let first = st.facts.iter().find_map(|f| if let Input::GloveSecured { fielder, .. } = f { Some(fielder & 0x7F) } else { None });
            let class = if outs > 0 {
                2
            } else if first.map(|p| p >= 7).unwrap_or(true) {
                0
            } else {
                1
            };
            // Closest approach of any infielder (P, 1B, 2B, 3B, SS; the catcher starts beside
            // the ball at contact) to the low ball in the infield.
            // Only the free ball: before anyone secured it.
            let t_secure = st.facts.iter().find_map(|f| if let Input::GloveSecured { t, .. } = f { Some(*t) } else { None }).unwrap_or(f64::INFINITY);
            let track = &st.viz.batted;
            let mut best = f64::INFINITY;
            for (id, samples) in &st.viz.actors {
                if id & 0x80 == 0 || (id & 0x7F) > 6 || (id & 0x7F) == 2 {
                    continue;
                }
                // Fielder position interpolated at each ball sample (fielders every 0.1 s).
                for b in track.iter().filter(|b| b[0] < t_secure && b[3] <= 2.4 && b[1].hypot(b[2]) <= 45.0) {
                    let k = samples.partition_point(|s| s.0 <= b[0]);
                    if k == 0 || k >= samples.len() {
                        continue;
                    }
                    let (a, c) = (samples[k - 1], samples[k]);
                    let w = (b[0] - a.0) / (c.0 - a.0);
                    let p = a.1.lerp(c.1, w);
                    best = best.min((b[1] - p.x).hypot(b[2] - p.y));
                }
            }
            if class == 0 && first.is_some_and(|p| p < 7) {
                touched_through += 1;
            }
            if let Some(path) = &st.batted {
                let near = (0..6).filter(|&i| i != 1).any(|i| {
                    let h = home[i];
                    let mut t = path.t_start();
                    let mut hit = false;
                    while t < path.t_end() && !hit {
                        let p = path.at(t).0;
                        hit = p.z <= 2.4 && p.x.hypot(p.y) <= 45.0 && (p.x - h.x).hypot(p.y - h.y) <= 3.0;
                        t += 0.01;
                    }
                    hit
                });
                if near && ev / 0.44704 < 95.0 {
                    routine += 1;
                    routine_out += (outs > 0) as u32;
                }
            }
            by_ev[ev_edges.iter().position(|&e| ev / 0.44704 < e).unwrap()][class] += 1;
            let k = edges.iter().position(|&e| best < e).unwrap();
            hist[class][k] += 1;
            ev_sum[class] += ev / 0.44704;
            n[class] += 1;
        }
    }
    let names = ["through to OF", "infield hit", "out(s) made"];
    println!("nearest infielder's closest approach to the low ball inside 45 m ({games} games)");
    println!("{:15} {:>5} {:>7}  {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}", "", "n", "EV mph", "<1", "1-1.5", "1.5-2.5", "2.5-4", "4-7", ">7");
    for c in 0..3 {
        let tot = n[c].max(1) as f64;
        let pct: Vec<String> = hist[c].iter().map(|&h| format!("{:5.0}%", 100.0 * h as f64 / tot)).collect();
        println!("{:15} {:5} {:7.1}  {}", names[c], n[c], ev_sum[c] / tot, pct.join(" "));
    }
    println!("through balls first secured by an infielder (deflected on): {touched_through}");
    println!("routine grounders (< 95 mph, within 3 m of an infielder's start): {routine}, converted to an out {:.1}% (target ≥ 96%, envelope ≥ 93%)", 100.0 * routine_out as f64 / routine.max(1) as f64);
    println!("by exit speed: through / infield hit / out");
    let mut lo = 0.0;
    for (k, hi) in ev_edges.iter().enumerate() {
        let tot = by_ev[k].iter().sum::<u32>().max(1) as f64;
        println!("  {lo:3.0}-{hi:3.0} mph  n {:4}  {:.3} / {:.3} / {:.3}", tot, by_ev[k][0] as f64 / tot, by_ev[k][1] as f64 / tot, by_ev[k][2] as f64 / tot);
        lo = *hi;
    }
}
