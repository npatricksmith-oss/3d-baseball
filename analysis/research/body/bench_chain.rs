// Cost of one driven swing in Rust (docs/body-motion.md 8.2): the same planar chain as chain.py
// (5 DOF: base, pelvis, trunk, arms, bat; 6 drive channels with activation and force-velocity; the
// friction caps, the wrist stop; RK4), integrated 0.27 s at 0.5, 1 and 2 ms, repeated to time it.
// Not sim code: a stand-alone timing probe. f64 libm-free (std sin/cos: timing only, not replay).
// Build and run:  rustc -C opt-level=3 -C linker=$HOME/.local/bin/zigcc -o sim/target/codex-probes/body/bench_chain \
//                   analysis/research/body/bench_chain.rs && sim/target/codex-probes/body/bench_chain
use std::time::Instant;

const N: usize = 5;
struct Pt { m: f64, terms: [(usize, f64, f64); 3], nt: usize }

fn eom(x: &[f64; 10], pts: &[Pt], rot: &[f64; 5], act: &[f64; 6], tq: &[f64; 6], w: &[f64; 6]) -> [f64; 10] {
    let (q, qd) = (&x[..5], &x[5..]);
    let mut mm = [[0.0f64; N]; N];
    let mut h = [0.0f64; N];
    for p in pts {
        let mut jx = [0.0f64; N]; let mut jy = [0.0f64; N]; jy[0] = 1.0;
        let (mut cx, mut cy) = (0.0, 0.0);
        for t in &p.terms[..p.nt] {
            let a = q[t.0] + t.2; let (s, c) = a.sin_cos();
            jx[t.0] += -t.1 * s; jy[t.0] += t.1 * c;
            let ww = qd[t.0] * qd[t.0];
            cx += -t.1 * ww * c; cy += -t.1 * ww * s;
        }
        for i in 0..N {
            h[i] += p.m * (jx[i] * cx + jy[i] * cy);
            for j in 0..N { mm[i][j] += p.m * (jx[i] * jx[j] + jy[i] * jy[j]); }
        }
    }
    for i in 1..N { mm[i][i] += rot[i]; }
    let rel = [qd[1], qd[2] - qd[1], qd[3] - qd[2], qd[4] - qd[3], qd[0], -qd[0]];
    let mut tau = [0.0f64; 6];
    for k in 0..6 {
        let u = rel[k] / w[k];
        let f = if u >= 1.0 { 0.0 } else if u >= 0.0 { (1.0 - u) / (1.0 + u / 0.25) } else { 1.5 - 0.5 / (1.0 - 4.0 * u) };
        tau[k] = tq[k] * act[k] * f;
    }
    let cap = 280.0; tau[0] = tau[0].max(-cap).min(cap);
    let lag = q[3] - q[4];
    if lag > 2.32 { tau[3] += (2.0e4 * (lag - 2.32) + 40.0 * (qd[3] - qd[4])).max(0.0); }   // one-sided: never pulls
    let qf = [tau[4] - tau[5], tau[0] - tau[1], tau[1] - tau[2], tau[2] - tau[3], tau[3]];
    // Gaussian elimination
    let mut a = [[0.0f64; N + 1]; N];
    for i in 0..N { for j in 0..N { a[i][j] = mm[i][j]; } a[i][N] = qf[i] - h[i]; }
    for c in 0..N {
        let mut p = c; for r in c + 1..N { if a[r][c].abs() > a[p][c].abs() { p = r; } }
        a.swap(c, p);
        for r in c + 1..N { let f = a[r][c] / a[c][c]; for k in c..=N { a[r][k] -= f * a[c][k]; } }
    }
    let mut qdd = [0.0f64; N];
    for r in (0..N).rev() { let mut s = a[r][N]; for k in r + 1..N { s -= a[r][k] * qdd[k]; } qdd[r] = s / a[r][r]; }
    let mut d = [0.0f64; 10];
    for i in 0..5 { d[i] = qd[i]; d[5 + i] = qdd[i]; }
    d
}

fn swing(dt: f64, pts: &[Pt], rot: &[f64; 5]) -> f64 {
    let tq = [1200.0, 352.0, 253.0, 60.0, 166.0, 960.0];
    let w = [20.0, 25.0, 30.0, 40.0, 3.0, 3.0];
    let on = [-0.228, -0.238, -0.231, 0.074, -1.0, -0.095];
    let mut x = [0.318, -0.36, -0.36, -1.9, -4.0, 0.6, -0.07, -0.84, -0.17, -0.17];
    let mut act = [0.0f64; 6];
    let mut t = -0.25;
    while t < 0.02 {
        for k in 0..6 { let u = if t >= on[k] { 1.0 } else { 0.0 }; let tc = if u > act[k] { 0.015 } else { 0.05 }; act[k] = u + (act[k] - u) * (-dt / tc).exp(); }
        let k1 = eom(&x, pts, rot, &act, &tq, &w);
        let mut y = x; for i in 0..10 { y[i] = x[i] + dt / 2.0 * k1[i]; }
        let k2 = eom(&y, pts, rot, &act, &tq, &w);
        for i in 0..10 { y[i] = x[i] + dt / 2.0 * k2[i]; }
        let k3 = eom(&y, pts, rot, &act, &tq, &w);
        for i in 0..10 { y[i] = x[i] + dt * k3[i]; }
        let k4 = eom(&y, pts, rot, &act, &tq, &w);
        for i in 0..10 { x[i] += dt / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]); }
        t += dt;
    }
    x[4]
}

fn main() {
    let h = 1.84; let rsh = 0.112 * h; let la = 0.30 * h;
    let sh = (2usize, rsh, std::f64::consts::FRAC_PI_2);
    let pts = vec![
        Pt { m: 70.4, terms: [(0, 0.0, 0.0); 3], nt: 0 },
        Pt { m: 4.99, terms: [sh, (3, 0.17, 0.0), (0, 0.0, 0.0)], nt: 2 },
        Pt { m: 2.98, terms: [sh, (3, 0.42, 0.0), (0, 0.0, 0.0)], nt: 2 },
        Pt { m: 1.12, terms: [sh, (3, la, 0.0), (0, 0.0, 0.0)], nt: 2 },
        Pt { m: 0.965, terms: [sh, (3, la, 0.0), (4, 0.517, 0.0)], nt: 3 },
    ];
    let rot = [0.0, 0.27, 0.41, 0.11, 0.038];
    for dt in [0.0005, 0.001, 0.002] {
        let n = 2000;
        let t0 = Instant::now();
        let mut s = 0.0;
        for _ in 0..n { s += swing(dt, &pts, &rot); }
        let el = t0.elapsed().as_secs_f64() / n as f64;
        println!("dt {:.1} ms: {:.1} us per 0.27 s swing ({:.0} steps, {:.2} us per RK4 step)  [check {:.3}]",
                 dt * 1e3, el * 1e6, 0.27 / dt, el * 1e6 / (0.27 / dt), s / n as f64);
    }
}
