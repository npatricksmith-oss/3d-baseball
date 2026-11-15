//! Diagnostic: routine fly balls the outfield does not catch, and why (agent-models §5.1;
//! docs/calibration/2026-11-15-outfield-read.md). `-- [first seed] [games] [list]`
//!
//! A fly is a fair batted ball launched at 25° or more that comes down in the park, in the
//! outfield (beyond the infield dirt), not within 3 m of a wall. It is *routine* when the
//! outfielder nearest its landing point at contact needs to cover no more than
//! `ROUTINE_FRAC` of what an average real outfielder covers in the opportunity time (pitch
//! release to landing, Statcast's clock): 33 ft in the first 3 s (the middle of the 2022
//! single-season range 29–38 ft, FanGraphs) and 27 ft/s (Statcast's average sprint speed)
//! after. The fraction is anchored on Statcast's 99% example (39 ft in 4.2 s: 0.60 of that
//! coverage; FanGraphs, Renfroe). Those are ≥ 95% plays in Statcast's terms.
//!
//! For each routine fly not caught, the responsible man (that nearest outfielder): his read's
//! landing-point error at 2.0, 1.0 and 0.5 s before it lands (signed along home → landing:
//! + = believed deeper), seconds since his last read at landing, his speed then, his role,
//! how close his body came to the ball's track below 2.4 m, and whether he dove.
use bb_game::live::read_trace::{self, Rec};
use bb_game::live::read_prediction;
use bb_math::Vec3;
use bb_rules::{Event, Input};

const ROUTINE_FRAC: f64 = 0.6;
const FT: f64 = 0.3048;

fn coverage_ft(t_opp: f64) -> f64 {
    if t_opp <= 3.0 { 33.0 * t_opp / 3.0 } else { 33.0 + 27.0 * (t_opp - 3.0) }
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let first: u64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let games: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(300);
    let list = a.get(2).is_some_and(|s| s == "list");
    let field = bb_physics::ground::Field::default();
    read_trace::ON.with(|x| x.set(true));
    // [bin] -> (routine flies, caught)
    let bins = [(25.0, 35.0), (35.0, 50.0), (50.0, 91.0)];
    let mut tally = [[0u32; 2]; 3];
    let mut causes: std::collections::BTreeMap<&'static str, u32> = Default::default();
    let mut all_flies = [0u32; 2];
    let mut zero: Vec<Vec<Vec<f64>>> = vec![vec![Vec::new(); 3]; 3];
    let of = bb_game::play::FielderAttrs::average(7);
    let mu = bb_game::pa::Matchup::average();
    let mut signed: Vec<Vec<Vec<f64>>> = vec![vec![Vec::new(); 3]; 3];
    let mut lateral: Vec<Vec<Vec<f64>>> = vec![vec![Vec::new(); 3]; 3];
    for seed in first..first + games {
        let mut g = bb_game::game::Game::new(seed, false);
        loop {
            read_trace::take();
            let Some(step) = g.step_pitch() else { break };
            let recs = read_trace::take();
            let (Some((_, la, _)), Some(path)) = (step.record.batted, &step.batted) else { continue };
            let la = la.to_degrees();
            let Some(k) = bins.iter().position(|b| la >= b.0 && la < b.1) else { continue };
            if !step.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: true })) {
                continue;
            }
            let Some((t_land, lx, ly)) = path.first_bounce() else { continue };
            if path.walls.first().is_some_and(|w| w.0 < t_land) {
                continue;
            }
            let land = Vec3::new(lx, ly, 0.0);
            let Some(bb_physics::ground::Segment::Flight { tr, .. }) = path.segments.first() else { continue };
            let contact = tr.first().s.r;
            if land.norm() < 47.4 || field.ball_clearance(lx, ly, 0.037).dist < 3.0 {
                continue;
            }
            let caught = step.rules_events.iter().any(|e| matches!(e, Event::Catch { .. }));
            all_flies[0] += 1;
            all_flies[1] += caught as u32;
            let Some(t_c) = step.facts.iter().find_map(|f| if let Input::BatContact { t } = f { Some(*t) } else { None }) else { continue };
            let starts: Vec<&Rec> = (6..9).filter_map(|i| recs.iter().find(|r| r.i == i)).collect();
            if starts.len() < 3 {
                continue;
            }
            let resp = starts.iter().min_by(|x, y| (x.pos - land).norm().total_cmp(&(y.pos - land).norm())).unwrap();
            let need_ft = (resp.pos - land).norm() / FT;
            let t_opp = t_land; // the clock starts at the pitch's release (t = 0)
            if need_ft > ROUTINE_FRAC * coverage_ft(t_opp) {
                continue;
            }
            tally[k][0] += 1;
            {
                let i = resp.i;
                let out = land.normalized();
                for (j, dt) in [2.0, 1.0, 0.5].iter().enumerate() {
                    let r = recs.iter().filter(|r| r.i == i && r.t <= t_land).min_by(|x, y| (x.t - (t_land - dt)).abs().total_cmp(&(y.t - (t_land - dt)).abs()));
                    if let Some(rr) = r {
                        let t = t_land - dt;
                        let view = (t - t_c - of.reaction).max(0.0);
                        let p0 = read_prediction(&mu.env, &field, path.at(t), t, rr.pos, of.read_sigma0, of.tau_read, view, [0.0; 4], Some((t - t_c, contact)));
                        if let Some((_, x, y)) = p0.first_bounce() {
                            zero[k][j].push((Vec3::new(x, y, 0.0) - land).dot(out));
                        }
                    }
                    if let Some((_, x, y)) = r.and_then(|r| r.land) {
                        let e = Vec3::new(x, y, 0.0) - land;
                        signed[k][j].push(e.dot(out));
                        lateral[k][j].push(e.dot(Vec3::new(-out.y, out.x, 0.0)));
                    }
                }
            }
            if caught {
                tally[k][1] += 1;
                continue;
            }
            let i = resp.i;
            let mine: Vec<&Rec> = recs.iter().filter(|r| r.i == i && r.t <= t_land).collect();
            let at = |dt: f64| mine.iter().min_by(|x, y| (x.t - (t_land - dt)).abs().total_cmp(&(y.t - (t_land - dt)).abs())).copied();
            let out = Vec3::new(lx, ly, 0.0).normalized();
            let err = |dt: f64| at(dt).and_then(|r| r.land).map(|(_, x, y)| (Vec3::new(x, y, 0.0) - land, (Vec3::new(x, y, 0.0) - land).dot(out)));
            let last = mine.last().copied();
            let Some(last) = last else { continue };
            let gap = (last.pos - land).norm();
            let since_read = last.t - last.read_at;
            let speed = last.vel.norm();
            // Closest his body came to the ball while it was catchable (≤ 2.4 m up).
            let mut closest = f64::INFINITY;
            for r in &mine {
                let (b, _) = path.at(r.t);
                if b.z <= 2.4 {
                    closest = closest.min(Vec3::new(b.x - r.pos.x, b.y - r.pos.y, 0.0).norm());
                }
            }
            let dove = mine.iter().any(|r| r.dove);
            let others_chasing = recs.iter().filter(|r| r.t == last.t && r.i != i && r.role == 1).count();
            let e1 = err(1.0).map(|e| e.0.norm()).unwrap_or(f64::NAN);
            let e05 = err(0.5).map(|e| e.0.norm()).unwrap_or(f64::NAN);
            let cause: &'static str = if last.role != 1 && others_chasing == 0 {
                "nobody chasing at landing"
            } else if last.role != 1 {
                "left it to a teammate, who missed"
            } else if closest <= 1.4 {
                "reached it (within reach + stretch), not caught"
            } else if closest <= 2.5 && !dove {
                "dive range, no dive"
            } else if e05 > 2.0 || e1 > 3.0 {
                "late read error (> 3 m at 1 s or > 2 m at 0.5 s)"
            } else {
                "read good late, too far to make up"
            };
            *causes.entry(cause).or_default() += 1;
            if list {
                let f = |o: Option<(Vec3, f64)>| o.map(|(e, d)| format!("{:4.1}({:+4.1})", e.norm(), d)).unwrap_or("  -".into());
                println!(
                    "seed {seed:4} play {:3} F{} LA {la:4.1} hang {:.1} need {need_ft:3.0} ft of {:3.0} | err 2/1/.5 s {} {} {} | read {:.2} s ago, v {speed:.1}, role {}, gap {gap:.1}, closest {closest:.1}, dove {dove}, others {others_chasing} | {cause}",
                    step.play,
                    i + 1,
                    t_land - t_c,
                    coverage_ft(t_opp),
                    f(err(2.0)),
                    f(err(1.0)),
                    f(err(0.5)),
                    since_read,
                    last.role
                );
            }
        }
    }
    println!("flies in the outfield (in the park, off the wall): {} caught of {} ({:.3} fall)", all_flies[1], all_flies[0], 1.0 - all_flies[1] as f64 / all_flies[0].max(1) as f64);
    for (k, b) in bins.iter().enumerate() {
        let [n, c] = tally[k];
        println!("routine, LA {:>2}-{:<2}: caught {c} of {n} = {:.3}", b.0, b.1, c as f64 / n.max(1) as f64);
    }
    let n: u32 = tally.iter().map(|x| x[0]).sum();
    let c: u32 = tally.iter().map(|x| x[1]).sum();
    println!("routine, all: caught {c} of {n} = {:.3}", c as f64 / n.max(1) as f64);
    let ms = |v: &[f64]| {
        let n = v.len().max(1) as f64;
        let m = v.iter().sum::<f64>() / n;
        (m, (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / n).sqrt())
    };
    println!("read error of the responsible man on routine flies (along home->landing, + = deeper; lateral), mean / sd, m, at 2.0, 1.0, 0.5 s before landing:");
    for (k, b) in bins.iter().enumerate() {
        let d: Vec<String> = (0..3).map(|j| { let (m, s) = ms(&signed[k][j]); format!("{m:+5.2}/{s:4.2}") }).collect();
        let l: Vec<String> = (0..3).map(|j| { let (m, s) = ms(&lateral[k][j]); format!("{m:+5.2}/{s:4.2}") }).collect();
        let z: Vec<String> = (0..3).map(|j| { let (m, s) = ms(&zero[k][j]); format!("{m:+5.2}/{s:4.2}") }).collect();
        println!("  LA {:>2}-{:<2}: depth {}  lateral {}  depth with no draws {}", b.0, b.1, d.join(" "), l.join(" "), z.join(" "));
    }
    for (k, v) in &causes {
        println!("  {v:4}  {k}");
    }
}
