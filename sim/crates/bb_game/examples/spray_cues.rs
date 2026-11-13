//! Which honest cues carry information about where a ball in play goes (agent-models §5.10,
//! docs/calibration/2026-11-13-anticipation-cue.md). For each fair ball in play it records the
//! spray and candidate cues a fielder might perceive before or at contact, and prints, for each
//! cue, its spread, its correlation with spray, the spray scatter left after a straight-line
//! fit, and what is left after a joint fit with the pitch's horizontal location (the existing
//! prior), so a cue's *added* information shows. Batter-relative signs: negative spray = pull
//! side (every game batter is right-handed, so this is the raw spray).
//! `cargo run --release -p bb_game --example spray_cues -- [games]`
use bb_physics::contact::BatPose;
use bb_rules::Event;

struct Row {
    spray: f64,
    cues: Vec<f64>,
    ty: String,
    fooled: bool,
    /// Game and batter id (the batter's history in this game).
    who: (u64, u16),
}

const NAMES: [&str; 15] = [
    "plate x (m)",
    "plate z (m)",
    "target x, called (m)",
    "target z, called (m)",
    "release speed (m/s)",
    "bat face at contact (deg)",
    "contact minus swing's nominal t (ms)",
    "contact depth y (m)",
    "swing start minus plate t (ms)",
    "bat tilt at contact (deg)",
    "balls",
    "strikes",
    "launch angle (deg) [outcome]",
    "speed minus previous pitch in PA (m/s)",
    "swing start since release (ms)",
];

fn ls(x: &[Vec<f64>], y: &[f64]) -> (Vec<f64>, f64) {
    // Least squares with intercept via normal equations (small k).
    let n = y.len();
    let k = x.len() + 1;
    let col = |j: usize, i: usize| if j == 0 { 1.0 } else { x[j - 1][i] };
    let mut a = vec![vec![0.0; k + 1]; k];
    for i in 0..n {
        for r in 0..k {
            for c in 0..k {
                a[r][c] += col(r, i) * col(c, i);
            }
            a[r][k] += col(r, i) * y[i];
        }
    }
    for p in 0..k {
        let piv = (p..k).max_by(|&u, &v| a[u][p].abs().partial_cmp(&a[v][p].abs()).unwrap()).unwrap();
        a.swap(p, piv);
        for r in 0..k {
            if r != p {
                let f = a[r][p] / a[p][p];
                for c in p..=k {
                    a[r][c] -= f * a[p][c];
                }
            }
        }
    }
    let b: Vec<f64> = (0..k).map(|r| a[r][k] / a[r][r]).collect();
    let res = (0..n).map(|i| {
        let f: f64 = (0..k).map(|j| b[j] * col(j, i)).sum();
        (y[i] - f) * (y[i] - f)
    });
    let s = (res.sum::<f64>() / n as f64).sqrt();
    (b, s)
}

fn main() {
    let games: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(64);
    let mut rows = Vec::new();
    for seed in 0..games {
        let mut g = bb_game::game::Game::new(seed, false);
        let mut prev_v: Option<f64> = None;
        loop {
            let batter = g.st.batter;
            let Some(step) = g.step_pitch() else { break };
            let r = &step.record;
            if r.count.balls == 0 && r.count.strikes == 0 {
                prev_v = None;
            }
            let dv = prev_v.map(|v| r.release.v.norm() - v).unwrap_or(0.0);
            prev_v = Some(r.release.v.norm());
            let Some((_, la, spray)) = r.batted else { continue };
            if !step.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: true })) {
                continue;
            }
            let (Some(sw), Some((t_c, st))) = (r.swing, r.contact_state) else { continue };
            let Some(p) = r.plate.or(Some(st.r)) else { continue };
            let a = sw.at(t_c).axis;
            let face = bb_math::m::atan2(a.x, a.y).to_degrees() - 90.0;
            let t_plate = r.t_plate.unwrap_or(t_c);
            rows.push(Row {
                spray: spray.to_degrees(),
                cues: vec![
                    p.x,
                    p.z,
                    r.target.0,
                    r.target.1,
                    r.release.v.norm(),
                    face,
                    1000.0 * (t_c - sw.t_c),
                    st.r.y,
                    1000.0 * (sw.t_start() - t_plate),
                    a.z.asin().to_degrees(),
                    r.count.balls as f64,
                    r.count.strikes as f64,
                    la.to_degrees(),
                    dv,
                    1000.0 * sw.t_start(),
                ],
                ty: r.ty.code().to_string(),
                fooled: r.read != r.ty,
                who: (seed, batter),
            });
        }
    }
    let n = rows.len();
    let y: Vec<f64> = rows.iter().map(|r| r.spray).collect();
    let my = y.iter().sum::<f64>() / n as f64;
    let sy = (y.iter().map(|v| (v - my) * (v - my)).sum::<f64>() / n as f64).sqrt();
    println!("{n} fair balls in play ({games} games); spray mean {my:+.1}° sd {sy:.1}°");
    let px: Vec<f64> = rows.iter().map(|r| r.cues[0]).collect();
    let (_, res_px) = ls(&[px.clone()], &y);
    println!("pitch x alone (the existing prior's input): residual sd {res_px:.2}°\n");
    println!("{:<40} {:>9} {:>9} {:>7} {:>10} {:>9} {:>11}", "cue", "mean", "sd", "r", "slope", "resid sd", "+px resid");
    for (j, name) in NAMES.iter().enumerate() {
        let x: Vec<f64> = rows.iter().map(|r| r.cues[j]).collect();
        let mx = x.iter().sum::<f64>() / n as f64;
        let sx = (x.iter().map(|v| (v - mx) * (v - mx)).sum::<f64>() / n as f64).sqrt();
        let cov = x.iter().zip(&y).map(|(a, b)| (a - mx) * (b - my)).sum::<f64>() / n as f64;
        let rr = cov / (sx * sy);
        let (b, res) = ls(&[x.clone()], &y);
        let res2 = if j == 0 { res } else { ls(&[px.clone(), x.clone()], &y).1 };
        println!("{name:<40} {mx:>9.3} {sx:>9.3} {rr:>7.3} {:>10.3} {res:>9.2} {res2:>11.2}", b[1]);
    }
    // Pitch type (categorical: known from the catcher's sign before the pitch).
    let mut by: std::collections::BTreeMap<String, Vec<f64>> = Default::default();
    for r in &rows {
        by.entry(r.ty.clone()).or_default().push(r.spray);
    }
    println!("\nby called pitch type:  n    mean    sd");
    let mut within = 0.0;
    for (t, v) in &by {
        let m = v.iter().sum::<f64>() / v.len() as f64;
        let sd2 = v.iter().map(|x| (x - m) * (x - m)).sum::<f64>();
        within += sd2;
        println!("{t:<4} {:6} {m:+7.2} {:6.2}", v.len(), (sd2 / v.len() as f64).sqrt());
    }
    println!("type-mean residual sd {:.2}° (vs {sy:.2}° overall)", (within / n as f64).sqrt());
    // Type dummies plus pitch x.
    let types: Vec<&String> = by.keys().collect();
    let mut xs = vec![px.clone()];
    for t in types.iter().skip(1) {
        xs.push(rows.iter().map(|r| if &r.ty == *t { 1.0 } else { 0.0 }).collect());
    }
    println!("pitch x + type: residual sd {:.2}°", ls(&xs, &y).1);
    let f: Vec<&Row> = rows.iter().filter(|r| r.fooled).collect();
    let nf: Vec<&Row> = rows.iter().filter(|r| !r.fooled).collect();
    let mean = |v: &[&Row]| v.iter().map(|r| r.spray).sum::<f64>() / v.len().max(1) as f64;
    println!("batter misread the type: n {} mean spray {:+.2}°; read it: n {} mean {:+.2}°", f.len(), mean(&f), nf.len(), mean(&nf));
    // Everything visible before the ball leaves the bat, jointly (an upper bound on the
    // honest prior with perfect reads of these): plate x, z, target x, z, speed, count, type.
    let mut all = vec![];
    for j in [0usize, 1, 2, 3, 4, 10, 11] {
        all.push(rows.iter().map(|r| r.cues[j]).collect::<Vec<f64>>());
    }
    for t in types.iter().skip(1) {
        all.push(rows.iter().map(|r| if &r.ty == *t { 1.0 } else { 0.0 }).collect());
    }
    println!("all pre-pitch/in-flight cues jointly, read perfectly: residual sd {:.2}°", ls(&all, &y).1);
    let mut with_face = all.clone();
    with_face.push(rows.iter().map(|r| r.cues[5]).collect());
    println!("  ... plus the bat face read perfectly: residual sd {:.2}°", ls(&with_face, &y).1);
    let mut with_t = all.clone();
    with_t.push(rows.iter().map(|r| r.cues[6]).collect());
    println!("  ... plus the swing's timing offset read perfectly: residual sd {:.2}°", ls(&with_t, &y).1);
    let mut with_y = all.clone();
    with_y.push(rows.iter().map(|r| r.cues[7]).collect());
    println!("  ... plus the contact depth read perfectly: residual sd {:.2}°", ls(&with_y, &y).1);
    let mut with_la = with_face.clone();
    with_la.push(rows.iter().map(|r| r.cues[12]).collect());
    println!("  ... plus face and launch angle [outcome, not a cue]: residual sd {:.2}°", ls(&with_la, &y).1);
    honest(&rows, &y);
    history(&rows);
    // The swing's lead (how long before the ball reaches the plate the bat starts, ms) as a
    // second read of the bat's face at contact: face = c + d × lead (the belief behind
    // `LEAD_FACE_*` in live.rs).
    let lead: Vec<f64> = rows.iter().map(|r| -r.cues[8]).collect();
    let face: Vec<f64> = rows.iter().map(|r| r.cues[5]).collect();
    let (b, res) = ls(&[lead.clone()], &face);
    let ml = lead.iter().sum::<f64>() / n as f64;
    println!("\nface = {:+.3}° + {:.4}°/ms × lead; residual sd {res:.2}°; lead mean {ml:.2} ms", b[0], b[1]);
    let (b2, res2) = ls(&[lead.clone()], &y);
    println!("spray = {:+.3}° + {:.4}°/ms × lead; residual sd {res2:.2}°", b2[0], b2[1]);
}

/// The batter's own history in this game: is there a persistent per-batter (or per batter and
/// pitch type) spray component, e.g. from his model error about this pitcher? Between-group
/// variance (group means less their sampling variance), and a causal predictor: the shrunk
/// mean of his previous fair balls in play this game.
fn history(rows: &[Row]) {
    use std::collections::BTreeMap;
    let n = rows.len() as f64;
    let my = rows.iter().map(|r| r.spray).sum::<f64>() / n;
    for (label, by_type) in [("batter", false), ("batter x type", true)] {
        let mut grp: BTreeMap<(u64, u16, String), Vec<f64>> = BTreeMap::new();
        for r in rows {
            grp.entry((r.who.0, r.who.1, if by_type { r.ty.clone() } else { String::new() })).or_default().push(r.spray);
        }
        let (mut within, mut nw) = (0.0, 0.0);
        let (mut between, mut nb, mut samp) = (0.0, 0.0, 0.0);
        for v in grp.values().filter(|v| v.len() >= 2) {
            let m = v.iter().sum::<f64>() / v.len() as f64;
            within += v.iter().map(|x| (x - m) * (x - m)).sum::<f64>();
            nw += (v.len() - 1) as f64;
            between += (m - my) * (m - my);
            nb += 1.0;
            samp += 1.0 / v.len() as f64;
        }
        let sw2 = within / nw;
        let comp = between / nb - sw2 * samp / nb;
        println!("per-{label} component: sd {:.2} deg (within sd {:.2}, {} groups)", comp.max(0.0).sqrt(), sw2.sqrt(), nb);
    }
    for n0 in [2.0, 5.0, 10.0, 20.0] {
        let mut hist: BTreeMap<(u64, u16), Vec<f64>> = BTreeMap::new();
        let mut se = 0.0;
        for r in rows {
            let h = hist.entry(r.who).or_default();
            let k = h.len() as f64;
            let pred = if h.is_empty() { my } else { my + (h.iter().sum::<f64>() / k - my) * k / (k + n0) };
            se += (r.spray - pred) * (r.spray - pred);
            h.push(r.spray);
        }
        println!("previous-BIP predictor (shrink n0 = {n0}): residual sd {:.2} deg", (se / n).sqrt());
    }
}

/// Spray residual sd of the best linear fit on *noisy* reads, at the Anticipation grades'
/// timing σ (78, 40, 20.5, 5.4 ms at grades 20, 50, 80, 160) and pitch-location σ.
fn honest(rows: &[Row], y: &[f64]) {
    let mut s: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut gauss = move || {
        let mut u = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            ((s >> 11) as f64 + 0.5) / (1u64 << 53) as f64
        };
        let (a, b) = (u(), u());
        (-2.0 * a.ln()).sqrt() * (core::f64::consts::TAU * b).cos()
    };
    println!("\nhonest reads: spray residual sd (deg) of the best linear fit on noisy reads");
    println!("grade  sigma_t  px only  +face  +depth  +onset(plate)  +face+onset  +onset(release)+type");
    let types: Vec<String> = {
        let mut t: Vec<String> = rows.iter().map(|r| r.ty.clone()).collect();
        t.sort();
        t.dedup();
        t
    };
    for (g, st_ms, spx) in [(20, 78.0, 0.293), (50, 40.0, 0.15), (80, 20.5, 0.0768), (160, 5.37, 0.0201)] {
        let px: Vec<f64> = rows.iter().map(|r| r.cues[0] + spx * gauss()).collect();
        let face: Vec<f64> = rows.iter().map(|r| r.cues[5] + 1.915 * st_ms * gauss()).collect();
        let depth: Vec<f64> = rows.iter().map(|r| r.cues[7] + 0.038 * st_ms * gauss()).collect();
        let onset: Vec<f64> = rows.iter().map(|r| r.cues[8] + st_ms * gauss()).collect();
        let onset_rel: Vec<f64> = rows.iter().map(|r| r.cues[14] + st_ms * gauss()).collect();
        let mut tr = vec![px.clone(), onset_rel];
        for t in types.iter().skip(1) {
            tr.push(rows.iter().map(|r| if &r.ty == t { 1.0 } else { 0.0 }).collect());
        }
        println!(
            "{g:5}  {st_ms:6.1}  {:7.2}  {:5.2}  {:6.2}  {:13.2}  {:11.2}  {:20.2}",
            ls(&[px.clone()], y).1,
            ls(&[px.clone(), face.clone()], y).1,
            ls(&[px.clone(), depth], y).1,
            ls(&[px.clone(), onset.clone()], y).1,
            ls(&[px.clone(), face, onset], y).1,
            ls(&tr, y).1,
        );
    }
}
