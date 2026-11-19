//! Batted-ball spin in games: the "typical spin" fielders and runners assume (agent-models §5.1),
//! for balls launched toward the field (spray within the foul lines), by launch angle (10° bins centred −25…55°) × exit speed (10 mph bins centred 65…105 mph; the
//! outer columns take everything below 70 and from 100 up). Prints the tables `SPIN_BACK_RPM`
//! and `SPIN_REST_RPM` of live.rs:
//! - backspin: the median of each ball's backspin (its spin about the horizontal axis square to
//!   its flight);
//! - rest: the spin besides the backspin (side and gyro, whose axis an observer cannot tell),
//!   as he flies it (`FlightEnv::unseen_spin`: it counts in the drag, gives no Magnus force): the
//!   value with which the cell's balls, each flown from its own launch velocity with the belief
//!   as it is looked up (both tables interpolated at the ball's own launch angle and speed),
//!   carry as far as they truly do in the median (signed carry miss to the ground, open air, no
//!   fence, median zero; bisection per cell, repeated over all cells, since each
//!   cell's value reaches the balls of its neighbours, until no cell moves by 50 rpm). Each ball's own rest varies widely
//!   (quartiles about ±35% of the median) and carry is not linear in it, so the median rest flies
//!   short. The median of each ball's own rest, √(|ω|² − backspin²), is printed for comparison.
//! Arguments: first and end seed (default 0 96).
use bb_math::Vec3;
use bb_physics::flight::BallState;

const LA: [f64; 9] = [-25.0, -15.0, -5.0, 5.0, 15.0, 25.0, 35.0, 45.0, 55.0];
const EV: [f64; 5] = [65.0, 75.0, 85.0, 95.0, 105.0];

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

/// Horizontal carry from `s` to the ground (ball centre at its radius), open air.
fn carry(env: &bb_physics::FlightEnv, s: BallState) -> f64 {
    let mut st = s;
    let r0 = s.r;
    for _ in 0..4000 {
        let next = env.step(&st, 0.01);
        if next.r.z <= env.ball.radius && next.v.z < 0.0 {
            let f = (st.r.z - env.ball.radius) / (st.r.z - next.r.z);
            let p = st.r + (next.r - st.r) * f;
            return (p.x - r0.x).hypot(p.y - r0.y);
        }
        st = next;
    }
    (st.r.x - r0.x).hypot(st.r.y - r0.y)
}

fn main() {
    let (a, b): (u64, u64) = (std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0), std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(96));
    let env = bb_physics::FlightEnv::default_env();
    let rpm = 30.0 / std::f64::consts::PI;
    // Per cell: (launch state, backspin rpm, rest rpm).
    let mut cells: Vec<Vec<(BallState, f64, f64)>> = vec![Vec::new(); LA.len() * EV.len()];
    for seed in a..b {
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            let Some((ev, la, _)) = step.record.batted else { continue };
            let Some(path) = &step.batted else { continue };
            let Some(bb_physics::ground::Segment::Flight { tr, .. }) = path.segments.first() else { continue };
            let s0 = tr.first().s;
            // Launched toward the field (spray within the foul lines), as an observer sees it go.
            if s0.v.x.atan2(s0.v.y).abs() > std::f64::consts::FRAC_PI_4 {
                continue;
            }
            let h = Vec3::new(s0.v.x, s0.v.y, 0.0).normalized();
            let back = s0.w.dot(Vec3::new(h.y, -h.x, 0.0)) * rpm;
            let total = s0.w.norm() * rpm;
            let k = (((la.to_degrees() + 30.0) / 10.0).floor() as isize).clamp(0, 8) as usize;
            let e = (((ev / bb_math::units::MPH - 60.0) / 10.0).floor() as isize).clamp(0, 4) as usize;
            cells[k * EV.len() + e].push((s0, back, (total * total - back * back).max(0.0).sqrt()));
        }
    }
    let mut back_t = [[0.0; 5]; 9];
    let mut rest_t = [[0.0; 5]; 9];
    let mut own_t = [[0.0; 5]; 9];
    for k in 0..LA.len() {
        for e in 0..EV.len() {
            let c = &cells[k * EV.len() + e];
            back_t[k][e] = median(c.iter().map(|x| x.1).collect());
            own_t[k][e] = median(c.iter().map(|x| x.2).collect());
        }
    }
    let truth: Vec<Vec<f64>> = cells.iter().map(|c| c.iter().map(|x| carry(&env, x.0)).collect()).collect();
    // The median miss of cell (k, e)'s balls with the belief as looked up.
    let miss = |k: usize, e: usize, rest_t: &[[f64; 5]; 9]| {
        let c = &cells[k * EV.len() + e];
        median(c.iter().zip(&truth[k * EV.len() + e]).map(|(x, d)| {
            let v = x.0.v;
            let h = Vec3::new(v.x, v.y, 0.0).normalized();
            let la = v.z.atan2(v.x.hypot(v.y));
            let back = bb_game::live::spin_table(&back_t, la, v.norm());
            let rest = bb_game::live::spin_table(rest_t, la, v.norm());
            let w = Vec3::new(h.y, -h.x, 0.0) * (back / rpm);
            carry(&bb_physics::FlightEnv { unseen_spin: rest.max(0.0) / rpm, ..env }, BallState { r: x.0.r, v, w }) - d
        }).collect())
    };
    // Sweep until no cell moves by more than SETTLED_RPM (review 2026-11-19 P3-1: four sweeps
    // left a cell moving by 700 rpm), at most MAX_SWEEPS.
    const SETTLED_RPM: f64 = 50.0;
    const MAX_SWEEPS: usize = 40;
    for sweep in 0..MAX_SWEEPS {
        let mut worst: f64 = 0.0;
        for k in 0..LA.len() {
            for e in 0..EV.len() {
                if cells[k * EV.len() + e].is_empty() {
                    continue;
                }
                let mut t = rest_t;
                let mut at = |r: f64| {
                    t[k][e] = r;
                    miss(k, e, &t)
                };
                // More rest, more drag, less carry: bisect for a median miss of zero.
                let fitted = if at(0.0) <= 0.0 {
                    0.0
                } else {
                    let (mut lo, mut hi) = (0.0, 20000.0);
                    for _ in 0..30 {
                        let mid = 0.5 * (lo + hi);
                        if at(mid) > 0.0 { lo = mid } else { hi = mid }
                    }
                    0.5 * (lo + hi)
                };
                worst = worst.max((fitted - rest_t[k][e]).abs());
                rest_t[k][e] = fitted;
            }
        }
        println!("sweep {sweep}: largest change {worst:.0} rpm");
        if worst < SETTLED_RPM {
            break;
        }
    }
    println!("cell: n, backspin median, own-rest median, fitted rest (rpm), median miss as looked up (m)");
    for k in 0..LA.len() {
        for e in 0..EV.len() {
            let n = cells[k * EV.len() + e].len();
            let m = if n > 0 { miss(k, e, &rest_t) } else { f64::NAN };
            println!("LA {:+3} EV {:3}: n {:4}  back {:6.0}  own rest {:6.0}  fitted rest {:6.0}  miss {:+.2}", LA[k], EV[e], n, back_t[k][e], own_t[k][e], rest_t[k][e], m);
        }
    }
    let print = |name: &str, t: &[[f64; 5]; 9]| {
        println!("const {name}: [[f64; 5]; 9] = [");
        for row in t {
            println!("    [{}],", row.iter().map(|x| format!("{:.0}.0", x)).collect::<Vec<_>>().join(", "));
        }
        println!("];");
    };
    print("SPIN_BACK_RPM", &back_t);
    print("SPIN_REST_RPM", &rest_t);
}
