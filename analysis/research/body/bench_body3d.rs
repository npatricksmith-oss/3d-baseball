// What the dominant arithmetic of one 3D body step costs in Rust (docs/body-motion.md 8.2).
// The checking model (analysis/oracle/body3d/engine.py) solves, four times per RK4 step, a dense
// symmetric positive-definite system of 49 joint rows by Cholesky; that is about two thirds of its
// multiply-adds (20,000 of 32,000 per evaluation). This times exactly that kernel, with plain scalar
// f64 arithmetic in a fixed order (no SIMD intrinsics, no mul_add: invariant 5). It is NOT the engine:
// the measured step time waits for the engine's port. Not sim code.
// Build and run:  rustc -C opt-level=3 -C linker=$HOME/.local/bin/zigcc -o sim/target/codex-probes/body3d/bench_body3d \
//                   analysis/research/body/bench_body3d.rs && sim/target/codex-probes/body3d/bench_body3d
use std::time::Instant;

const N: usize = 49;

fn chol_solve(a: &[[f64; N]; N], b: &[f64; N]) -> [f64; N] {
    let mut l = [[0.0f64; N]; N];
    for i in 0..N {
        for j in 0..i {
            let mut s = a[i][j];
            for k in 0..j { s -= l[i][k] * l[j][k]; }
            l[i][j] = s / l[j][j];
        }
        let mut d = a[i][i];
        for k in 0..i { d -= l[i][k] * l[i][k]; }
        l[i][i] = d.sqrt();
    }
    let mut y = [0.0f64; N];
    for i in 0..N {
        let mut s = b[i];
        for k in 0..i { s -= l[i][k] * y[k]; }
        y[i] = s / l[i][i];
    }
    let mut x = [0.0f64; N];
    for i in (0..N).rev() {
        let mut s = y[i];
        for k in i + 1..N { s -= l[k][i] * x[k]; }
        x[i] = s / l[i][i];
    }
    x
}

fn main() {
    // a dense SPD matrix: G G^T + n I from a fixed pseudo-random G
    let mut g = [[0.0f64; N]; N];
    let mut seed: u64 = 12345;
    for i in 0..N { for j in 0..N {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        g[i][j] = ((seed >> 33) as f64) / 2147483648.0 - 1.0;
    } }
    let mut a = [[0.0f64; N]; N];
    for i in 0..N { for j in 0..N { let mut s = 0.0; for k in 0..N { s += g[i][k] * g[j][k]; } a[i][j] = s; } a[i][i] += N as f64; }
    let mut b = [0.0f64; N];
    for i in 0..N { b[i] = (i as f64).sin(); }
    let reps = 200_000;
    let t = Instant::now();
    let mut acc = 0.0;
    for r in 0..reps {
        b[0] = 1.0 + (r as f64) * 1e-9;
        let x = chol_solve(&a, &b);
        acc += x[N - 1];
    }
    let us = t.elapsed().as_secs_f64() * 1e6 / reps as f64;
    println!("49x49 Cholesky factor + solve: {:.2} us each ({:.1} us for the four of one RK4 step); checksum {:.6}", us, 4.0 * us, acc);
}
