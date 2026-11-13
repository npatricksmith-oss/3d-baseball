//! Diagnostic: where the pitcher ends each ball in play (TASKS "Pitcher in the outfield",
//! docs/calibration/2026-11-12-pitcher-outfield-diagnosis.md).
//!
//! `cargo run --release -p bb_game --example pitcher_roam -- <seed0> <n_games> [all]`
//!
//! For each ball in play it takes the pitcher's track (position 1 of the fielding side) and
//! reports where he ends, his speed over the last recorded step, and his distance from the
//! mound and from the nearest base. A play is flagged FAR when he ends more than `FAR_M` from
//! the mound and from every base: a pitcher's jobs on a ball in play are the mound area,
//! covering a base, or backing one up a few metres beyond it. With `all`, every ball in play
//! is listed. The last line counts flagged plays over balls in play.
use bb_game::play::base_pos;
use bb_math::Vec3;

const FAR_M: f64 = 30.0;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let seed0: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(7);
    let n: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let all = a.iter().any(|s| s == "all");
    let mound = Vec3::new(0.0, 18.44, 0.0);
    let (mut bip, mut far, mut far_secured) = (0u32, 0u32, 0u32);
    for seed in seed0..seed0 + n {
        let mut g = bb_game::game::Game::new(seed, true);
        while let Some(step) = g.step_pitch() {
            if step.batted.is_none() {
                continue;
            }
            let v = &step.viz;
            let Some((_, tr)) = v.actors.iter().find(|(id, _)| id & 0xFF == 0x81) else { continue };
            if tr.len() < 2 {
                continue;
            }
            bip += 1;
            let (t1, p1) = tr[tr.len() - 1];
            let (t0, p0) = tr[tr.len() - 2];
            let speed = if t1 > t0 { Vec3::new(p1.x - p0.x, p1.y - p0.y, 0.0).norm() / (t1 - t0) } else { 0.0 };
            let flat = Vec3::new(p1.x, p1.y, 0.0);
            let d_mound = (flat - mound).norm();
            let d_base = (1..=4).map(|b| (flat - base_pos(b)).norm()).fold(f64::INFINITY, f64::min);
            let flag = d_mound > FAR_M && d_base > FAR_M;
            if flag {
                far += 1;
            }
            // Did the pitcher field the ball out there (a stray man near the ball can catch it)?
            if v.secured.iter().any(|(_, id)| id & 0xFF == 0x81) && d_mound > FAR_M {
                far_secured += 1;
            }
            if flag || all {
                println!(
                    "seed {seed} play {} {:<3} P end ({:.0},{:.0}) {:.1} m/s, mound {:.0} m, base {:.0} m  {}",
                    step.play,
                    if flag { "FAR" } else { "" },
                    p1.x,
                    p1.y,
                    speed,
                    d_mound,
                    d_base,
                    v.label
                );
            }
        }
    }
    println!("balls in play {bip}; pitcher ends > {FAR_M} m from the mound and every base: {far} ({:.1}%); pitcher secured the ball and ended > {FAR_M} m from the mound: {far_secured}", 100.0 * far as f64 / bip.max(1) as f64);
}
