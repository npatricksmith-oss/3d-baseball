//! Diagnostic: every dive in a run of games (owner backlog A5). Who dove, at what ball
//! (in the air before its first bounce, or on the ground), how fast it was, how far from
//! his body, how far he had run, and what came of it (caught, off the glove, missed).
//! `-- [games] [first seed] [list]`: `list` prints each dive.
use bb_math::Vec3;
use std::collections::BTreeMap;

const POS: [&str; 10] = ["?", "P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"];

#[derive(Default, Clone, Copy)]
struct Tally {
    n: u64,
    caught: u64,
    air: u64,
    air_caught: u64,
}

/// The ball just before `t`: the last sample of a (t, p) track before it (at `t` itself a
/// caught ball is already in the hand), moved on to `t` at the finite-difference velocity.
fn ball_at(track: &[[f64; 4]], t: f64) -> Option<(Vec3, Vec3)> {
    let i = track.iter().rposition(|s| s[0] < t - 1e-9)?;
    if i == 0 {
        return None;
    }
    let v3 = |s: &[f64; 4]| Vec3::new(s[1], s[2], s[3]);
    let v = (v3(&track[i]) - v3(&track[i - 1])) * (1.0 / (track[i][0] - track[i - 1][0]).max(1e-9));
    Some((v3(&track[i]) + v * (t - track[i][0]), v))
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let games: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(100);
    let first: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let list = a.get(2).is_some_and(|s| s == "list");
    let mut by_pos: BTreeMap<u16, Tally> = BTreeMap::new();
    let (mut bip, mut plays_with_dive) = (0u64, 0u64);
    // Histograms: gap from body to ball (0.25 m bins from 1.0), ball speed (10 m/s bins),
    // ball height (0.3 m bins), each as (dives, caught).
    let mut gap_h = [(0u64, 0u64); 7];
    let mut spd_h = [(0u64, 0u64); 7];
    let mut z_h = [(0u64, 0u64); 5];
    // How far the fielder had moved from where he stood at the pitch (5 m bins).
    let mut run_h = [(0u64, 0u64); 6];
    for seed in first..first + games {
        let mut g = bb_game::game::Game::new(seed, true);
        while let Some(step) = g.step_pitch() {
            let Some(path) = &step.batted else { continue };
            bip += 1;
            if step.viz.dives.is_empty() {
                continue;
            }
            plays_with_dive += 1;
            let first_bounce = path.bounces.first().map_or(f64::INFINITY, |b| b.0);
            let first_touch = step.viz.secured.first().map_or(f64::INFINITY, |s| s.0);
            for &(t, id, from, _) in &step.viz.dives {
                let pos = id & 0x7F;
                let caught = step.viz.secured.iter().any(|&(ts, who)| who == id && (ts - t).abs() < 1e-6);
                // In the air: the batted ball before it has touched the ground or a glove.
                let air = t < first_bounce && t <= first_touch;
                let (b, v) = ball_at(&step.viz.batted, t).unwrap_or((from, Vec3::ZERO));
                let gap = ((b.x - from.x).powi(2) + (b.y - from.y).powi(2)).sqrt();
                let stood = step.viz.standing.iter().find(|s| s.0 == id).map_or(from, |s| s.1);
                let ran = (from - stood).norm();
                let e = by_pos.entry(pos).or_default();
                e.n += 1;
                e.caught += caught as u64;
                e.air += air as u64;
                e.air_caught += (air && caught) as u64;
                let bump = |h: &mut [(u64, u64)], i: usize| {
                    let i = i.min(h.len() - 1);
                    h[i].0 += 1;
                    h[i].1 += caught as u64;
                };
                bump(&mut gap_h, ((gap - 1.0).max(0.0) / 0.25) as usize);
                bump(&mut spd_h, (v.norm() / 10.0) as usize);
                bump(&mut z_h, (b.z.max(0.0) / 0.3) as usize);
                bump(&mut run_h, (ran / 5.0) as usize);
                if list {
                    println!(
                        "seed {seed} play {} t {:.2} {} {} gap {:.2} m z {:.2} m ball {:.0} m/s ran {:.1} m: {}",
                        step.play,
                        t,
                        POS[pos.min(9) as usize],
                        if air { "air" } else { "ground" },
                        gap,
                        b.z,
                        v.norm(),
                        ran,
                        if caught { "caught" } else { "not caught" }
                    );
                }
            }
        }
    }
    let total: u64 = by_pos.values().map(|t| t.n).sum();
    let caught: u64 = by_pos.values().map(|t| t.caught).sum();
    let g = games as f64;
    println!("{games} games from seed {first}: {bip} balls in play, {total} dives ({:.1} a game), {plays_with_dive} plays with a dive ({:.1}% of balls in play), {:.0}% caught", total as f64 / g, 100.0 * plays_with_dive as f64 / bip.max(1) as f64, 100.0 * caught as f64 / total.max(1) as f64);
    println!("pos   dives/game  caught   in the air/game  caught   on the ground/game  caught");
    for (pos, t) in &by_pos {
        let gr = t.n - t.air;
        let gc = t.caught - t.air_caught;
        println!(
            "{:<4}  {:>10.2}  {:>5.0}%   {:>15.2}  {:>5.0}%   {:>18.2}  {:>5.0}%",
            POS[(*pos).min(9) as usize],
            t.n as f64 / g,
            100.0 * t.caught as f64 / t.n.max(1) as f64,
            t.air as f64 / g,
            100.0 * t.air_caught as f64 / t.air.max(1) as f64,
            gr as f64 / g,
            100.0 * gc as f64 / gr.max(1) as f64
        );
    }
    let show = |name: &str, h: &[(u64, u64)], lo: f64, w: f64| {
        println!("{name}");
        for (i, (n, c)) in h.iter().enumerate() {
            println!("  {:>5.2}+  {:>6.2}/game  {:>3.0}% caught", lo + w * i as f64, *n as f64 / g, 100.0 * *c as f64 / (*n).max(1) as f64);
        }
    };
    show("gap from body to ball at the dive, m", &gap_h, 1.0, 0.25);
    show("ball speed at the dive, m/s", &spd_h, 0.0, 10.0);
    show("ball height at the dive, m", &z_h, 0.0, 0.3);
    show("how far the fielder had moved from his position, m", &run_h, 0.0, 5.0);
}
