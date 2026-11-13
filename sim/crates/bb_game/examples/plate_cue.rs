//! How good an observer's knowledge that a batted ball started at the plate is, by itself
//! (agent-models §5.1, `live::position_from_plate`): for every batted ball still on its first
//! flight `s` seconds after contact, its true velocity flown back `s` seconds with the
//! observer's physics and typical spin, against where it really is. Median and 90th
//! percentile of the miss, m, by launch angle. This is the floor under the plate cue's
//! error (`PLATE_SPIN_M`): with the true spin the miss is 0, so all of it is the spin he
//! cannot see. A real read adds his velocity error times `s`. `-- [games] [true]`
use bb_game::live::{believed_spin, position_from_plate};

fn q(v: &mut [f64], f: f64) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[((v.len() as f64 * f) as usize).min(v.len() - 1)]
}

fn main() {
    let games: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(48);
    // `-- <games> true`: fly back with the ball's true spin instead (the method alone: 0 m).
    let true_spin = std::env::args().nth(2).is_some_and(|a| a == "true");
    let env = bb_game::pa::Matchup::average().env;
    const TIMES: [f64; 6] = [0.25, 0.5, 1.0, 2.0, 3.0, 4.0];
    const BINS: [(f64, f64); 5] = [(-90.0, 10.0), (10.0, 25.0), (25.0, 40.0), (40.0, 55.0), (55.0, 91.0)];
    let mut acc = vec![vec![Vec::new(); TIMES.len()]; BINS.len()];
    for seed in 0..games {
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            let (Some((_, la, _)), Some(path)) = (step.record.batted, &step.batted) else { continue };
            let Some(bb_physics::ground::Segment::Flight { tr, .. }) = path.segments.first() else { continue };
            let s0 = tr.first();
            let t_end = path.first_bounce().map(|b| b.0).unwrap_or(f64::INFINITY).min(path.walls.first().map(|w| w.0).unwrap_or(f64::INFINITY));
            let la = la.to_degrees();
            let k = BINS.iter().position(|b| la >= b.0 && la < b.1).unwrap_or(0);
            for (j, &s) in TIMES.iter().enumerate() {
                let t = s0.t + s;
                if t >= t_end {
                    continue;
                }
                let (p, v) = path.at(t);
                let wt = tr.knots.iter().min_by(|a, b| (a.t - t).abs().total_cmp(&(b.t - t).abs())).unwrap().s.w;
                let est = if true_spin { position_from_plate(&env, s0.s.r, v, wt, s) } else { position_from_plate(&env, s0.s.r, v, believed_spin(v, Some(s)), s) };
                acc[k][j].push((est - p).norm());
            }
        }
    }
    println!("plate cue with the true velocity{}:", if true_spin { " and spin" } else { "" });
    println!("miss, m, median / 90th percentile (n); {games} games");
    print!("{:>10}", "launch");
    for s in TIMES {
        print!("{:>18}", format!("s = {s} s"));
    }
    println!();
    for (k, row) in acc.iter_mut().enumerate() {
        print!("{:>10}", format!("{:.0}..{:.0}°", BINS[k].0, BINS[k].1.min(90.0)));
        for v in row.iter_mut() {
            if v.len() < 10 {
                print!("{:>18}", "-");
                continue;
            }
            let n = v.len();
            print!("{:>18}", format!("{:.2} / {:.2} ({n})", q(v, 0.5), q(v, 0.9)));
        }
        println!();
    }
}
