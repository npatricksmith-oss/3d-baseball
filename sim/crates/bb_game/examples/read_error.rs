//! Diagnostic: how far off an outfielder's read of a fly ball's landing point is, by launch
//! angle and time since contact, and which part of the read model (agent-models §5.1,
//! `live::read_prediction`) the error comes from. `-- [games] [sigma0 m] [tau_read s]`
//!
//! For every fair ball launched at 25° or more that comes down in the park, the outfielder
//! nearest the landing point reads it from his starting position (he does not move here, so
//! late errors are upper bounds). Each column is the shift of the predicted landing point, m,
//! for a one-σ draw of one error term alone; "spin" is the miss with no read error at all
//! (typical spin and the agent's flight step against the true path).
use bb_game::live::read_prediction;
use bb_game::play::{FielderAttrs, home_positions};
use bb_math::Vec3;

fn median(v: &mut [f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let games: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(48);
    let of = FielderAttrs::average(7);
    let sigma0: f64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(of.read_sigma0);
    let tau_read: f64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(of.tau_read);
    let mu = bb_game::pa::Matchup::average();
    let field = bb_physics::ground::Field::default();
    let home = home_positions();
    const BINS: [(f64, f64); 4] = [(25.0, 35.0), (35.0, 50.0), (50.0, 60.0), (60.0, 91.0)];
    const TIMES: [f64; 6] = [0.4, 0.7, 1.0, 2.0, 3.0, 4.0];
    // [bin][time][term] -> samples; terms: spin (signed, + = predicted deeper), pos depth,
    // pos lateral/height, vel depth, vel lateral/vertical, all four at one σ in quadrature.
    let mut acc: Vec<Vec<Vec<Vec<f64>>>> = vec![vec![vec![Vec::new(); 6]; TIMES.len()]; BINS.len()];
    let mut hang: Vec<Vec<f64>> = vec![Vec::new(); BINS.len()];
    for seed in 0..games {
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            let (Some((_, la, _)), Some(path)) = (step.record.batted, &step.batted) else { continue };
            let la = la.to_degrees();
            let Some(k) = BINS.iter().position(|b| la >= b.0 && la < b.1) else { continue };
            if !step.rules_events.iter().any(|e| matches!(e, bb_rules::Event::FairFoulDecided { fair: true })) {
                continue;
            }
            let Some((t_land, lx, ly)) = path.first_bounce() else { continue };
            let Some(bb_physics::ground::Segment::Flight { tr, .. }) = path.segments.first() else { continue };
            let t_c = tr.first().t;
            let land = Vec3::new(lx, ly, 0.0);
            let i = (6..9).min_by(|&a, &b| (home[a] - land).norm().total_cmp(&(home[b] - land).norm())).unwrap();
            let from = home[i];
            // Depth is measured along the fielder's line to the landing point (+ = farther from him).
            let out = (land - from).normalized();
            hang[k].push(t_land - t_c);
            for (j, &tau) in TIMES.iter().enumerate() {
                let t = t_c + tau;
                if t >= t_land - 0.3 {
                    continue;
                }
                let view = (tau - of.reaction).max(0.0);
                let read = |err: [f64; 4]| read_prediction(&mu.env, &field, path.at(t), t, from, sigma0, tau_read, view, err, Some(tau)).first_bounce().map(|b| Vec3::new(b.1, b.2, 0.0));
                let Some(base) = read([0.0; 4]) else { continue };
                acc[k][j][0].push((base - land).dot(out));
                let mut ss = 0.0;
                let mut ok = true;
                for c in 0..4 {
                    let mut e = [0.0; 4];
                    e[c] = 1.0;
                    match read(e) {
                        Some(p) => {
                            let d = (p - base).norm();
                            acc[k][j][1 + c].push(d);
                            ss += d * d;
                        }
                        None => ok = false,
                    }
                }
                if ok {
                    acc[k][j][5].push(ss.sqrt());
                }
            }
        }
    }
    println!("outfielder read: sigma0 {sigma0} m at 30 m, tau_read {tau_read} s; {games} games; medians, m");
    for (k, b) in BINS.iter().enumerate() {
        println!("\nlaunch {:.0}–{:.0}°: n {}, median hang time {:.1} s", b.0, b.1.min(90.0), hang[k].len(), median(&mut hang[k]));
        println!("  t since contact    n   spin(signed)  pos depth  pos lat/ht  vel depth  vel lat/vert  all 1σ");
        for (j, tau) in TIMES.iter().enumerate() {
            let n = acc[k][j][0].len();
            if n < 5 {
                continue;
            }
            let c: Vec<f64> = (0..6).map(|c| median(&mut acc[k][j][c])).collect();
            println!("  {tau:>6.1} s      {n:>5}   {:>+8.1}     {:>7.1}    {:>7.1}    {:>7.1}     {:>7.1}    {:>7.1}", c[0], c[1], c[2], c[3], c[4], c[5]);
        }
    }
}
