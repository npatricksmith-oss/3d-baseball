//! Diagnostic: outfielders who read a batted ball with no track at its first bounce (their
//! first look came after it; agent-models §5.1, the depth read of a ball first seen on the
//! ground). For each such outfielder, from the first bounce until someone secures the ball or
//! it reaches a wall: how far his read of where the ball is now is from where it is, along his
//! level line of sight (depth) and across it, by time since the bounce.
//! `-- <seed base> <n games> [threads]` (game seeds as `sim-cli games`: base·1000003 + i);
//! `-- play <seed> <play> <fielder 7-9>` prints one fielder's read every 0.1 s.
use bb_game::live::read_trace;
use bb_math::Vec3;
use bb_rules::{Input, Touch};

struct Case {
    seed: u64,
    play: u32,
    i: usize,
    // (t since bounce, depth err, lateral err, dist, role, phase)
    rows: Vec<(f64, f64, f64, f64, u8, u8)>,
}

fn stop_time(step: &bb_game::game::PitchStep) -> f64 {
    step.facts.iter().filter_map(|f| match f {
        Input::GloveSecured { t, .. } => Some(*t),
        Input::BallTouched { t, what: Touch::Wall { .. } | Touch::OutOfPlay { .. } } => Some(*t),
        _ => None,
    }).fold(f64::INFINITY, f64::min)
}

fn cases_of_game(seed: u64) -> Vec<Case> {
    read_trace::ON.with(|x| x.set(true));
    let mut g = bb_game::game::Game::new(seed, false);
    let mut out = Vec::new();
    loop {
        read_trace::take();
        let Some(step) = g.step_pitch() else { break };
        let recs = read_trace::take();
        let Some(path) = step.batted.as_ref() else { continue };
        let Some((tb, _, _)) = path.first_bounce() else { continue };
        let stop = stop_time(&step);
        for i in 6..9 {
            // No track at the first bounce: his first record after it has phase 0 and a read.
            let mine: Vec<_> = recs.iter().filter(|r| r.i == i && r.t >= tb && r.t < stop).collect();
            let Some(first) = mine.first() else { continue };
            if first.phase != 0 {
                continue;
            }
            let rows = mine.iter().filter(|r| r.phase == 0).filter_map(|r| {
                let est = r.est?;
                let to = Vec3::new(r.ball.x - r.pos.x, r.ball.y - r.pos.y, 0.0);
                let d = to.norm().max(1.0);
                let u = to * (1.0 / d);
                let lat = Vec3::new(-u.y, u.x, 0.0);
                let e = Vec3::new(est.x - r.ball.x, est.y - r.ball.y, 0.0);
                Some((r.t - tb, e.dot(u), e.dot(lat), d, r.role, r.phase))
            }).collect::<Vec<_>>();
            if !rows.is_empty() {
                out.push(Case { seed, play: step.play, i, rows });
            }
        }
    }
    out
}

fn pct(v: &mut [f64], p: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * p).round() as usize]
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a[0] == "play" {
        let (seed, play, who): (u64, u32, usize) = (a[1].parse().unwrap(), a[2].parse().unwrap(), a[3].parse::<usize>().unwrap() - 1);
        read_trace::ON.with(|x| x.set(true));
        let mut g = bb_game::game::Game::new(seed, false);
        loop {
            read_trace::take();
            let Some(step) = g.step_pitch() else { break };
            if step.play != play {
                continue;
            }
            let recs = read_trace::take();
            let path = step.batted.as_ref().unwrap();
            println!("first bounce {:?}, stop {:.2}", path.first_bounce(), stop_time(&step));
            let mut next = 0.0;
            for r in recs.iter().filter(|r| r.i == who) {
                if r.t + 1e-9 < next {
                    continue;
                }
                next = r.t + 0.1;
                let est = r.est.unwrap_or(Vec3::ZERO);
                let to = Vec3::new(r.ball.x - r.pos.x, r.ball.y - r.pos.y, 0.0);
                let u = to.normalized();
                let e = Vec3::new(est.x - r.ball.x, est.y - r.ball.y, 0.0);
                println!("t {:.2} phase {} role {} pos ({:5.1},{:5.1}) v {:4.1} ball ({:5.1},{:5.1},{:4.2}) est ({:5.1},{:5.1}) depth {:+6.1} lat {:+5.1} d {:5.1} target ({:.1},{:.1}) read {:.2} ago",
                    r.t, r.phase, r.role, r.pos.x, r.pos.y, r.vel.norm(), r.ball.x, r.ball.y, r.ball.z, est.x, est.y, e.dot(u), e.dot(Vec3::new(-u.y, u.x, 0.0)), to.norm(), r.target.x, r.target.y, r.t - r.read_at);
            }
            break;
        }
        return;
    }
    let (base, n): (u64, u64) = (a[0].parse().unwrap(), a[1].parse().unwrap());
    let threads: usize = a.get(2).map_or(8, |s| s.parse().unwrap());
    let seeds: Vec<u64> = (0..n).map(|i| base * 1_000_003 + i).collect();
    let chunk = seeds.len().div_ceil(threads);
    let cases: Vec<Case> = std::thread::scope(|s| {
        let hs: Vec<_> = seeds.chunks(chunk).map(|c| s.spawn(move || c.iter().flat_map(|&sd| cases_of_game(sd)).collect::<Vec<_>>())).collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    let plays: std::collections::BTreeSet<(u64, u32)> = cases.iter().map(|c| (c.seed, c.play)).collect();
    println!("{n} games: {} outfielders' plays with no track at the first bounce, in {} plays ({:.2} a game)", cases.len(), plays.len(), plays.len() as f64 / n as f64);
    let chasers = cases.iter().filter(|c| c.rows.iter().any(|r| r.4 == 1)).count();
    println!("  of them, {chasers} chase the ball at some point");
    for (label, only_chase) in [("all", false), ("while chasing", true)] {
        println!("{label}: |depth err| and |lateral err| of his read of the ball now (m): median / 90th pct / max, n");
        for &ts in &[0.0, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0] {
            let pick: Vec<_> = cases.iter().filter_map(|c| c.rows.iter().find(|r| r.0 >= ts && r.0 < ts + 0.1 && (!only_chase || r.4 == 1))).collect();
            let mut dep: Vec<f64> = pick.iter().map(|r| r.1.abs()).collect();
            let mut lat: Vec<f64> = pick.iter().map(|r| r.2.abs()).collect();
            let mut dist: Vec<f64> = pick.iter().map(|r| r.3).collect();
            let k = dep.len();
            println!("  +{ts:.1} s: depth {:5.1} / {:5.1} / {:5.1}  lateral {:4.1} / {:4.1} / {:4.1}  (dist median {:4.1} m)  n {k}",
                pct(&mut dep, 0.5), pct(&mut dep, 0.9), pct(&mut dep, 1.0), pct(&mut lat, 0.5), pct(&mut lat, 0.9), pct(&mut lat, 1.0), pct(&mut dist, 0.5));
        }
    }
    // The worst chasers' cases.
    let mut worst: Vec<(f64, &Case)> = cases.iter().map(|c| (c.rows.iter().filter(|r| r.4 == 1).map(|r| r.1.abs()).fold(0.0, f64::max), c)).collect();
    worst.sort_by(|a, b| b.0.total_cmp(&a.0));
    println!("worst depth errors while chasing:");
    for (e, c) in worst.iter().take(12) {
        let first = c.rows.first().unwrap();
        println!("  seed {} play {} fielder {}: {e:.1} m (dist {:.0} m at the bounce)", c.seed, c.play, c.i + 1, first.3);
    }
    // Counts over thresholds while chasing, per game.
    for th in [5.0, 10.0, 20.0] {
        let k = cases.iter().filter(|c| c.rows.iter().any(|r| r.4 == 1 && r.1.abs() > th)).count();
        println!("chasers ever more than {th:.0} m off in depth: {k} ({:.2} a game)", k as f64 / n as f64);
    }
}
