// The written swing made by muscles: the compiled forward run of analysis/research/body/swing_track.py (PLAN D48;
// docs/body-motion.md 4.17). NOT sim code and not the reference: swing_track.py's TrackSystem (on the checking engine
// analysis/oracle/body3d/engine.py) is the checking implementation, and `swing_track.py --check` and
// test_swing_track.py hold this file to it state for state.
// The body engine below is body3d_probe.rs's (the constrained rigid bodies, the joint ends, the feet on the ground,
// RK4), without that probe's time-scheduled drives, tendon springs, holds and tones. What is added is `control`: every
// joint axis's pair of muscles (strength falling with speed, a level that rises and falls at a muscle's rate, the
// joint's own friction), asked at each instant for the plan's torque for that instant plus a correction toward the
// written motion, and the legs' correction that pushes the pelvis back toward it. swing_track.py's docstring says it
// in full. f64, std sqrt and atan2/asin (the joint ends only); no SIMD intrinsics, no mul_add.
// Build:  rustc -C opt-level=3 -C linker=$HOME/.local/bin/zigcc -o sim/target/codex-probes/swing-look/swing_track \
//           analysis/research/body/swing_track.rs
// Run:    swing_track <model file>
use std::io::Write;

type V3 = [f64; 3];
type M9 = [f64; 9];
type V6 = [f64; 6];

fn cross(a: &V3, b: &V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn dot(a: &V3, b: &V3) -> f64 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn add(a: &V3, b: &V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn sub(a: &V3, b: &V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn scale(a: &V3, k: f64) -> V3 { [a[0] * k, a[1] * k, a[2] * k] }
fn norm(a: &V3) -> f64 { dot(a, a).sqrt() }
/// engine.py's float_axis: the unit vector along na x nb, or zero where they are parallel.
fn float_axis(na: &V3, nb: &V3) -> V3 {
    let c = cross(na, nb);
    let l = dot(&c, &c).sqrt();
    if l > 1e-9 { scale(&c, 1.0 / l) } else { [0.0; 3] }
}
fn qmul(p: &[f64; 4], q: &[f64; 4]) -> [f64; 4] {
    [p[0] * q[0] - p[1] * q[1] - p[2] * q[2] - p[3] * q[3],
     p[0] * q[1] + p[1] * q[0] + p[2] * q[3] - p[3] * q[2],
     p[0] * q[2] - p[1] * q[3] + p[2] * q[0] + p[3] * q[1],
     p[0] * q[3] + p[1] * q[2] - p[2] * q[1] + p[3] * q[0]]
}
fn qconj(q: &[f64; 4]) -> [f64; 4] { [q[0], -q[1], -q[2], -q[3]] }
fn qr(q: &[f64; 4]) -> M9 {
    let (w, x, y, z) = (q[0], q[1], q[2], q[3]);
    [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - w * z), 2.0 * (x * z + w * y),
     2.0 * (x * y + w * z), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - w * x),
     2.0 * (x * z - w * y), 2.0 * (y * z + w * x), 1.0 - 2.0 * (x * x + y * y)]
}
fn mv(r: &M9, v: &V3) -> V3 {
    [r[0] * v[0] + r[1] * v[1] + r[2] * v[2], r[3] * v[0] + r[4] * v[1] + r[5] * v[2], r[6] * v[0] + r[7] * v[1] + r[8] * v[2]]
}
fn mtv(r: &M9, v: &V3) -> V3 {
    [r[0] * v[0] + r[3] * v[1] + r[6] * v[2], r[1] * v[0] + r[4] * v[1] + r[7] * v[2], r[2] * v[0] + r[5] * v[1] + r[8] * v[2]]
}
const ZERO: V3 = [0.0; 3];
const E3: [V3; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

struct Pt { a: i32, b: i32, ra: V3, rb: V3 }
struct Dt { a: i32, b: i32, ua: V3, wb: V3 }
struct Stop { a: i32, b: usize, ax: V3, ea: V3, fb: V3, lo: f64, hi: f64, k: f64, c: f64, kind: i32, axb: V3,
              // kind 3 (engine.py's oval): the second angle's axis and direction in b, its direction in a, and the ellipse
              // (centre 1, centre 2, half-axis 1, half-axis 2, cos and sin of its tilt)
              n2: V3, e2: V3, f2: V3, ov: [f64; 6] }
/// engine.py's kind 0 stop_angle: the angle about n from e to f (f projected square to n) and its gradient.
fn angle0(n: &V3, e: &V3, f: &V3) -> (f64, V3) {
    let nf = dot(n, f);
    let fp = sub(f, &scale(n, nf));
    let l2 = dot(&fp, &fp);
    if l2 < STOP_SINGULAR * STOP_SINGULAR { eprintln!("a stop's direction lies along its axis: the angle it reads is undefined there"); std::process::exit(3); }
    (dot(n, &cross(e, f)).atan2(dot(e, f)), sub(n, &scale(&fp, nf / l2)))
}
/// engine.py's oval_reading: how far the pair (t1, t2) lies outside the ellipse along the line from its centre, and
/// that distance's two partial derivatives (zero inside, where they are not asked for).
fn oval_reading(t1: f64, t2: f64, ov: &[f64; 6]) -> (f64, f64, f64) {
    let (c1, c2, h1, h2, ct, sn) = (ov[0], ov[1], ov[2], ov[3], ov[4], ov[5]);
    let (x, y) = (t1 - c1, t2 - c2);
    let (xr, yr) = (x * ct + y * sn, -x * sn + y * ct);
    let rho = ((xr / h1) * (xr / h1) + (yr / h2) * (yr / h2)).sqrt();
    if rho <= 1.0 {
        return (if rho > 1e-12 { (x * x + y * y).sqrt() * (1.0 - 1.0 / rho) } else { -f64::min(h1, h2) }, 0.0, 0.0);
    }
    let l = (x * x + y * y).sqrt();
    let rx = (xr * ct / (h1 * h1) - yr * sn / (h2 * h2)) / rho;
    let ry = (xr * sn / (h1 * h1) + yr * ct / (h2 * h2)) / rho;
    let k1 = (1.0 - 1.0 / rho) / l;
    let k2 = l / (rho * rho);
    (l * (1.0 - 1.0 / rho), x * k1 + k2 * rx, y * k1 + k2 * ry)
}
const STOP_SINGULAR: f64 = 0.0872;   // engine.py's: a stop's direction this near its axis has no usable angle
struct Axis { kind: i32, ax: V3, axb: V3, tp: f64, tm: f64, tabp: usize, tabm: usize, damp: f64 }
struct Joint { a: usize, b: usize, kp: f64, kd: f64, k0: usize, na: usize, jb: usize, rb: V3 }
/// one sample of the written motion and the plan: each joint's (turn of the child in the parent, its rate there, its
/// hold as a share of its gains), the plan's torque about every muscle axis, the pelvis (place, turn, velocity, rate of
/// turn) and the lead leg's share in pushing it back
struct Sample { q: Vec<[f64; 4]>, w: Vec<V3>, g: Vec<f64>, ff: Vec<f64>, px: V3, pq: [f64; 4], pv: V3, pw: V3, wl: f64 }
struct Con { b: usize, p: V3, kn: f64, cn: f64, kt: f64, ct: f64, mu: f64 }
#[derive(Clone, Copy)]
struct Row { a: i32, ja: V6, b: i32, jb: V6, c: f64, cd: f64, gam: f64 }

struct Sys {
    g: V3, baum_a: f64, baum_b: f64,
    ta_up: f64, ta_dn: f64, ecc_slope: f64, d0: f64, reg3: f64, sat_eps: f64, fb_scale: f64, brake: f64,
    m: Vec<f64>, inr: Vec<V3>,
    pts: Vec<Pt>, dots: Vec<Dt>, order: Vec<usize>, first: Vec<usize>,
    stops: Vec<Stop>, cons: Vec<Con>,
    fvtabs: Vec<Vec<[f64; 2]>>, fvecc: Vec<f64>,
    joints: Vec<Joint>, axes: Vec<Axis>,
    pelvis: usize, root: [f64; 5], legs: [usize; 6],
    t0ref: f64, dtref: f64, refs: Vec<Sample>,
}

#[derive(Default)]
struct Diag { con: Vec<V3>, ax: Vec<[f64; 6]>, stop: Vec<[f64; 2]> }

fn lerp3(a: &V3, b: &V3, f: f64) -> V3 { [a[0] + f * (b[0] - a[0]), a[1] + f * (b[1] - a[1]), a[2] + f * (b[2] - a[2])] }
fn lerp4n(a: &[f64; 4], b: &[f64; 4], f: f64) -> [f64; 4] {
    let q = [a[0] + f * (b[0] - a[0]), a[1] + f * (b[1] - a[1]), a[2] + f * (b[2] - a[2]), a[3] + f * (b[3] - a[3])];
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    [q[0] / n, q[1] / n, q[2] / n, q[3] / n]
}

impl Sys {
    fn nb(&self) -> usize { self.m.len() }
    /// engine.py's fv_tab: a measured strength-at-speed curve, straight lines between its points.
    fn fv_tab(&self, tab: &[[f64; 2]], ecc: f64, w: f64) -> f64 {
        let top = tab[tab.len() - 1][0];
        if w >= top { return 0.0; }
        if w >= 0.0 {
            for k in 0..tab.len() - 1 {
                let (wa, fa, wb, fb) = (tab[k][0], tab[k][1], tab[k + 1][0], tab[k + 1][1]);
                if w <= wb { return fa + (fb - fa) * (w - wa) / (wb - wa); }
            }
            return 0.0;
        }
        ecc - (ecc - 1.0) / (1.0 - self.ecc_slope * (w / top))
    }
    /// swing_track.py's TrackSystem._forces after the engine's own forces: the muscles' levels and torques.
    #[allow(clippy::too_many_arguments)]
    fn control(&self, t: f64, s: &[f64], x: &[V3], r: &[M9], v: &[V3], w: &[V3], tq: &mut [V3], ds: &mut [f64], oa: usize,
               mut diag: Option<&mut Diag>) -> (f64, f64) {
        let n = self.refs.len();
        let u = (t - self.t0ref) / self.dtref;
        let mut kf = u.floor();
        if kf < 0.0 { kf = 0.0; }
        if kf > (n - 2) as f64 { kf = (n - 2) as f64; }
        let k = kf as usize;
        let mut f = u - kf;
        if f < 0.0 { f = 0.0; }
        if f > 1.0 { f = 1.0; }
        let (r0, r1) = (&self.refs[k], &self.refs[k + 1]);
        let nj = self.joints.len();
        // balance: the push and twist on the pelvis that would take it back to the written motion
        let p = self.pelvis;
        let xr = lerp3(&r0.px, &r1.px, f); let vr = lerp3(&r0.pv, &r1.pv, f); let wr = lerp3(&r0.pw, &r1.pw, f);
        let qrf = lerp4n(&r0.pq, &r1.pq, f);
        let o = 13 * p;
        let qp = [s[o + 3], s[o + 4], s[o + 5], s[o + 6]];
        let qe = qmul(&qrf, &qconj(&qp));
        let sg = if qe[0] < 0.0 { -2.0 } else { 2.0 };
        let er = [sg * qe[1], sg * qe[2], sg * qe[3]];
        let ep = sub(&xr, &x[p]); let ev = sub(&vr, &v[p]); let ew = sub(&wr, &w[p]);
        let (kh, kv, kdp, krot, kdr) = (self.root[0], self.root[1], self.root[2], self.root[3], self.root[4]);
        let fr = [kh * ep[0] + kdp * ev[0], kh * ep[1] + kdp * ev[1], kv * ep[2] + kdp * ev[2]];
        let mr = [krot * er[0] + kdr * ew[0], krot * er[1] + kdr * ew[1], krot * er[2] + kdr * ew[2]];
        let wl = r0.wl + f * (r1.wl - r0.wl);
        let wsup = [wl, wl, wl, 1.0 - wl, 1.0 - wl, 1.0 - wl];
        let mut extra = vec![ZERO; nj];
        for (li, &j) in self.legs.iter().enumerate() {
            let jt = &self.joints[j];
            let xj = add(&x[jt.jb], &mv(&r[jt.jb], &jt.rb));
            let mom = add(&mr, &cross(&sub(&x[p], &xj), &fr));
            extra[j] = scale(&mom, -wsup[li] * self.fb_scale);
        }
        let (mut p_mus, mut p_damp) = (0.0, 0.0);
        for (j, jt) in self.joints.iter().enumerate() {
            let g = r0.g[j] + f * (r1.g[j] - r0.g[j]);
            let kp = jt.kp * (g * self.fb_scale); let kd = jt.kd * (g * self.fb_scale);
            let (a, b) = (jt.a, jt.b);
            let (oa_, ob_) = (13 * a, 13 * b);
            let qa = [s[oa_ + 3], s[oa_ + 4], s[oa_ + 5], s[oa_ + 6]];
            let qb = [s[ob_ + 3], s[ob_ + 4], s[ob_ + 5], s[ob_ + 6]];
            let qrel = qmul(&qconj(&qa), &qb);
            let qref = lerp4n(&r0.q[j], &r1.q[j], f);
            let qe = qmul(&qref, &qconj(&qrel));
            let sg = if qe[0] < 0.0 { -2.0 } else { 2.0 };
            let e_par = [sg * qe[1], sg * qe[2], sg * qe[3]];
            let wrel_w = sub(&w[b], &w[a]);
            let wrel = mtv(&r[a], &wrel_w);
            let wref = lerp3(&r0.w[j], &r1.w[j], f);
            let tau_par = [kp * e_par[0] + kd * (wref[0] - wrel[0]), kp * e_par[1] + kd * (wref[1] - wrel[1]), kp * e_par[2] + kd * (wref[2] - wrel[2])];
            let tau = add(&mv(&r[a], &tau_par), &extra[j]);
            let mut ns = [ZERO; 3];
            for q in 0..jt.na {
                let ax = &self.axes[jt.k0 + q];
                ns[q] = if ax.kind == 0 { mv(&r[a], &ax.ax) } else if ax.kind == 1 { mv(&r[b], &ax.ax) } else { float_axis(&mv(&r[a], &ax.ax), &mv(&r[b], &ax.axb)) };
            }
            let mut c = [0.0f64; 3];
            if jt.na == 1 {
                c[0] = dot(&tau, &ns[0]);
            } else if jt.na == 2 {
                let (g11, g12, g22) = (dot(&ns[0], &ns[0]), dot(&ns[0], &ns[1]), dot(&ns[1], &ns[1]));
                let (b1, b2) = (dot(&tau, &ns[0]), dot(&tau, &ns[1]));
                let det = g11 * g22 - g12 * g12;
                c[0] = (g22 * b1 - g12 * b2) / det;
                c[1] = (g11 * b2 - g12 * b1) / det;
            } else {
                let mut gm = [[0.0f64; 3]; 3];
                for i in 0..3 { for jj in 0..3 { gm[i][jj] = dot(&ns[i], &ns[jj]) + (if i == jj { self.reg3 } else { 0.0 }); } }
                let bb = [dot(&tau, &ns[0]), dot(&tau, &ns[1]), dot(&tau, &ns[2])];
                let c00 = gm[1][1] * gm[2][2] - gm[1][2] * gm[2][1];
                let c01 = gm[1][2] * gm[2][0] - gm[1][0] * gm[2][2];
                let c02 = gm[1][0] * gm[2][1] - gm[1][1] * gm[2][0];
                let det = gm[0][0] * c00 + gm[0][1] * c01 + gm[0][2] * c02;
                let c11 = gm[0][0] * gm[2][2] - gm[0][2] * gm[2][0];
                let c12 = gm[0][1] * gm[2][0] - gm[0][0] * gm[2][1];
                let c22 = gm[0][0] * gm[1][1] - gm[0][1] * gm[1][0];
                c[0] = (c00 * bb[0] + c01 * bb[1] + c02 * bb[2]) / det;
                c[1] = (c01 * bb[0] + c11 * bb[1] + c12 * bb[2]) / det;
                c[2] = (c02 * bb[0] + c12 * bb[1] + c22 * bb[2]) / det;
            }
            for q in 0..jt.na {
                let iax = jt.k0 + q;
                let ax = &self.axes[iax];
                let nn = ns[q];
                let ffv = r0.ff[iax] + f * (r1.ff[iax] - r0.ff[iax]);
                let want = ffv + c[q];
                let om = dot(&wrel_w, &nn);
                let availp = ax.tp * self.fv_tab(&self.fvtabs[ax.tabp], self.fvecc[ax.tabp], om);
                let availm = ax.tm * self.fv_tab(&self.fvtabs[ax.tabm], self.fvecc[ax.tabm], -om);
                let (ap, am) = (s[oa + 2 * iax], s[oa + 2 * iax + 1]);
                let up = if availp > self.sat_eps { f64::max(0.0, f64::min(1.0, (want + self.brake * am * availm) / availp)) } else if want > 0.0 { 1.0 } else { 0.0 };
                let um = if availm > self.sat_eps { f64::max(0.0, f64::min(1.0, (self.brake * ap * availp - want) / availm)) } else if want < 0.0 { 1.0 } else { 0.0 };
                ds[oa + 2 * iax] = (up - ap) / (if up > ap { self.ta_up } else { self.ta_dn });
                ds[oa + 2 * iax + 1] = (um - am) / (if um > am { self.ta_up } else { self.ta_dn });
                let tmus = ap * availp - am * availm;
                let tdamp = -ax.damp * om;
                let tv = scale(&nn, tmus + tdamp);
                tq[b] = add(&tq[b], &tv);
                tq[a] = sub(&tq[a], &tv);
                p_mus += tmus * om;
                p_damp += tdamp * om;
                if let Some(d) = diag.as_deref_mut() { d.ax.push([want, ffv, tmus, ap, am, om]); }
            }
        }
        (p_mus, p_damp)
    }

    fn rows(&self, x: &[V3], r: &[M9], v: &[V3], w: &[V3]) -> Vec<Row> {
        let mut rows = Vec::with_capacity(3 * self.pts.len() + self.dots.len());
        for p in &self.pts {
            let (sa, pa, va, ca) = if p.a >= 0 {
                let a = p.a as usize;
                let sa = mv(&r[a], &p.ra);
                (sa, add(&x[a], &sa), add(&v[a], &cross(&w[a], &sa)), cross(&w[a], &cross(&w[a], &sa)))
            } else { (p.ra, p.ra, ZERO, ZERO) };
            let (sb, pb, vb, cb) = if p.b >= 0 {
                let b = p.b as usize;
                let sb = mv(&r[b], &p.rb);
                (sb, add(&x[b], &sb), add(&v[b], &cross(&w[b], &sb)), cross(&w[b], &cross(&w[b], &sb)))
            } else { (p.rb, p.rb, ZERO, ZERO) };
            for k in 0..3 {
                let e = E3[k];
                let c1 = cross(&sa, &e);
                let c2 = cross(&e, &sb);
                rows.push(Row { a: p.a, ja: [e[0], e[1], e[2], c1[0], c1[1], c1[2]], b: p.b,
                                jb: [-e[0], -e[1], -e[2], c2[0], c2[1], c2[2]],
                                c: pa[k] - pb[k], cd: va[k] - vb[k], gam: -1.0 * (ca[k] - cb[k]) });
            }
        }
        for d in &self.dots {
            let u = if d.a >= 0 { mv(&r[d.a as usize], &d.ua) } else { d.ua };
            let ww = if d.b >= 0 { mv(&r[d.b as usize], &d.wb) } else { d.wb };
            let wa = if d.a >= 0 { w[d.a as usize] } else { ZERO };
            let wb = if d.b >= 0 { w[d.b as usize] } else { ZERO };
            let n = cross(&u, &ww);
            let wr = sub(&wa, &wb);
            let nd = add(&cross(&cross(&wa, &u), &ww), &cross(&u, &cross(&wb, &ww)));
            rows.push(Row { a: d.a, ja: [0.0, 0.0, 0.0, n[0], n[1], n[2]], b: d.b, jb: [0.0, 0.0, 0.0, -n[0], -n[1], -n[2]],
                            c: dot(&u, &ww), cd: dot(&wr, &n), gam: -dot(&wr, &nd) });
        }
        if self.order.is_empty() { rows } else { self.order.iter().map(|&i| rows[i]).collect() }
    }

    fn winv(&self, r: &M9, i: usize, lin: f64, j: &V6) -> V6 {
        let inr = &self.inr[i];
        let l = mtv(r, &[j[3], j[4], j[5]]);
        let y = mv(r, &[l[0] * 1.0 / inr[0], l[1] * 1.0 / inr[1], l[2] * 1.0 / inr[2]]);
        [j[0] * lin, j[1] * lin, j[2] * lin, y[0], y[1], y[2]]
    }

    fn deriv(&self, t: f64, s: &[f64], aux: &[Option<[f64; 2]>], mut diag: Option<&mut Diag>) -> Vec<f64> {
        let nb = self.nb();
        let mut x = Vec::with_capacity(nb); let mut r = Vec::with_capacity(nb);
        let mut v = Vec::with_capacity(nb); let mut w = Vec::with_capacity(nb);
        for i in 0..nb {
            let o = 13 * i;
            x.push([s[o], s[o + 1], s[o + 2]]);
            r.push(qr(&[s[o + 3], s[o + 4], s[o + 5], s[o + 6]]));
            v.push([s[o + 7], s[o + 8], s[o + 9]]);
            w.push([s[o + 10], s[o + 11], s[o + 12]]);
        }
        let mut ds = vec![0.0; s.len()];
        let g = self.g;
        let mut f: Vec<V3> = (0..nb).map(|i| [self.m[i] * g[0], self.m[i] * g[1], self.m[i] * g[2]]).collect();
        let mut tq: Vec<V3> = vec![ZERO; nb];
        // ---- forces, in swing_track.py's order: joint ends, the ground, then the muscles ----
        let ow = 13 * nb;
        let oa = ow + 4;
        let (mut p_stop, mut p_con) = (0.0, 0.0);
        if let Some(d) = diag.as_deref_mut() { d.con.clear(); d.ax.clear(); d.stop.clear(); }
        for st in &self.stops {
            let n = if st.a >= 0 { mv(&r[st.a as usize], &st.ax) } else { st.ax };
            let e = if st.a >= 0 { mv(&r[st.a as usize], &st.ea) } else { st.ea };
            let fb = mv(&r[st.b], &st.fb);
            // engine.py's stop_angle: the angle and its own gradient g (d(angle)/dt = w_rel . g); the torque acts along g
            let nf = dot(&n, &fb);
            let (th, g) = if st.kind == 3 {
                // both of a two-axis joint's angles held inside an ellipse (engine.py's kind 3, body-motion 4.12)
                let f2 = if st.a >= 0 { mv(&r[st.a as usize], &st.f2) } else { st.f2 };
                let (t1, g1) = angle0(&n, &e, &fb);
                let (t2, g2) = angle0(&mv(&r[st.b], &st.n2), &mv(&r[st.b], &st.e2), &f2);
                let (d, d1, d2) = oval_reading(t1, t2, &st.ov);
                (d, sub(&scale(&g1, d1), &scale(&g2, d2)))
            } else if st.kind == 2 {
                // a limb's turn about its own axis v from its rest direction n (engine.py's kind 2): e carried by the
                // shortest turn n -> v, the angle about v from that to f; the gradient (n + v) / (1 + n.v)
                let v2 = mv(&r[st.b], &st.axb);
                let w2 = add(&n, &v2);
                if dot(&w2, &w2) < STOP_SINGULAR * STOP_SINGULAR { eprintln!("a limb's own axis is turned right over from its rest direction: its turn is undefined there"); std::process::exit(3); }
                let c1 = 1.0 + dot(&n, &v2);
                let kx = cross(&n, &v2);
                let ke = cross(&kx, &e);
                let ec = add(&add(&e, &ke), &scale(&cross(&kx, &ke), 1.0 / (1.0 + dot(&n, &v2))));
                (dot(&v2, &cross(&ec, &fb)).atan2(dot(&ec, &fb)), scale(&w2, 1.0 / c1))
            } else if st.kind == 1 {
                let th = f64::max(-1.0, f64::min(1.0, nf)).asin();
                if st.lo <= th && th <= st.hi { (th, ZERO) } else {
                    let gv = cross(&fb, &n);
                    let l = dot(&gv, &gv).sqrt();
                    if l < STOP_SINGULAR { eprintln!("a stop's direction lies along its axis: its angle has no gradient there"); std::process::exit(3); }
                    (th, scale(&gv, 1.0 / l))
                }
            } else {
                let fp = sub(&fb, &scale(&n, nf));
                let l2 = dot(&fp, &fp);
                if l2 < STOP_SINGULAR * STOP_SINGULAR { eprintln!("a stop's direction lies along its axis: the angle it reads is undefined there"); std::process::exit(3); }
                (dot(&n, &cross(&e, &fb)).atan2(dot(&e, &fb)), sub(&n, &scale(&fp, nf / l2)))
            };
            let wa = if st.a >= 0 { w[st.a as usize] } else { ZERO };
            let rate = dot(&sub(&w[st.b], &wa), &g);
            let mut tau = 0.0;
            if th > st.hi { tau = -f64::max(0.0, st.k * (th - st.hi) + st.c * rate); }
            else if th < st.lo { tau = f64::max(0.0, st.k * (st.lo - th) - st.c * rate); }
            if tau != 0.0 {
                let tv = scale(&g, tau);
                tq[st.b] = add(&tq[st.b], &tv);
                if st.a >= 0 { tq[st.a as usize] = sub(&tq[st.a as usize], &tv); }
                p_stop += tau * rate;
            }
            if let Some(d) = diag.as_deref_mut() { d.stop.push([tau, th]); }
        }
        for (j, c) in self.cons.iter().enumerate() {
            let sp = mv(&r[c.b], &c.p);
            let p = add(&x[c.b], &sp);
            let d = -p[2];
            let mut fc = ZERO;
            if d > 0.0 {
                let vp = add(&v[c.b], &cross(&w[c.b], &sp));
                let nn = f64::max(0.0, c.kn * d - c.cn * f64::min(1.0, d / self.d0) * vp[2]);
                let (mut fx, mut fy) = (0.0, 0.0);
                if let Some(anc) = aux[j] {
                    if nn > 0.0 {
                        fx = -c.kt * (p[0] - anc[0]) - c.ct * vp[0];
                        fy = -c.kt * (p[1] - anc[1]) - c.ct * vp[1];
                        let ft = (fx * fx + fy * fy).sqrt();
                        if ft > c.mu * nn { fx *= c.mu * nn / ft; fy *= c.mu * nn / ft; }
                    }
                }
                fc = [fx, fy, nn];
                f[c.b] = add(&f[c.b], &fc);
                tq[c.b] = add(&tq[c.b], &cross(&sp, &fc));
                p_con += dot(&fc, &vp);
            }
            if let Some(dg) = diag.as_deref_mut() { dg.con.push(fc); }
        }
        let (p_mus, p_damp) = self.control(t, s, &x, &r, &v, &w, &mut tq, &mut ds, oa, diag.as_deref_mut());
        ds[ow] = p_mus; ds[ow + 1] = p_stop; ds[ow + 2] = p_damp; ds[ow + 3] = p_con;
        // ---- free accelerations ----
        let mut a0: Vec<V6> = Vec::with_capacity(nb);
        let lin: Vec<f64> = self.m.iter().map(|m| 1.0 / m).collect();
        for i in 0..nb {
            let inr = &self.inr[i];
            let l = mtv(&r[i], &w[i]);
            let h = mv(&r[i], &[inr[0] * l[0], inr[1] * l[1], inr[2] * l[2]]);
            let tqi = sub(&tq[i], &cross(&w[i], &h));
            let lt = mtv(&r[i], &tqi);
            let al = mv(&r[i], &[lt[0] / inr[0], lt[1] / inr[1], lt[2] / inr[2]]);
            a0.push([f[i][0] * lin[i], f[i][1] * lin[i], f[i][2] * lin[i], al[0], al[1], al[2]]);
        }
        let rows = self.rows(&x, &r, &v, &w);
        let mut acc = a0.clone();
        let n = rows.len();
        if n > 0 {
            // per body: (row, J, W J^T)
            let mut per: Vec<Vec<(usize, V6, V6)>> = vec![Vec::new(); nb];
            for (k, rw) in rows.iter().enumerate() {
                if rw.a >= 0 { let a = rw.a as usize; per[a].push((k, rw.ja, self.winv(&r[a], a, lin[a], &rw.ja))); }
                if rw.b >= 0 { let b = rw.b as usize; per[b].push((k, rw.jb, self.winv(&r[b], b, lin[b], &rw.jb))); }
            }
            let mut am = vec![0.0f64; n * n];
            for lst in &per {
                for p in 0..lst.len() {
                    let (k, _, yk) = &lst[p];
                    for q in 0..=p {
                        let (l, jl, _) = &lst[q];
                        am[k * n + l] += jl[0] * yk[0] + jl[1] * yk[1] + jl[2] * yk[2] + jl[3] * yk[3] + jl[4] * yk[4] + jl[5] * yk[5];
                    }
                }
            }
            let a2 = 2.0 * self.baum_a; let b2 = self.baum_b * self.baum_b;
            let mut rhs = vec![0.0f64; n];
            for (k, rw) in rows.iter().enumerate() {
                let mut rr = rw.gam - a2 * rw.cd - b2 * rw.c;
                if rw.a >= 0 { let a = &a0[rw.a as usize]; let mut sm = 0.0; for c in 0..6 { sm += rw.ja[c] * a[c]; } rr -= sm; }
                if rw.b >= 0 { let a = &a0[rw.b as usize]; let mut sm = 0.0; for c in 0..6 { sm += rw.jb[c] * a[c]; } rr -= sm; }
                rhs[k] = rr;
            }
            let lam = self.chol_solve(&mut am, &rhs, n);
            for (i, lst) in per.iter().enumerate() {
                for (k, _, yk) in lst { let lk = lam[*k]; for c in 0..6 { acc[i][c] += lk * yk[c]; } }
            }
        }
        for i in 0..nb {
            let o = 13 * i;
            let q = [s[o + 3], s[o + 4], s[o + 5], s[o + 6]];
            let qd = qmul(&[0.0, w[i][0], w[i][1], w[i][2]], &q);
            ds[o] = v[i][0]; ds[o + 1] = v[i][1]; ds[o + 2] = v[i][2];
            ds[o + 3] = 0.5 * qd[0]; ds[o + 4] = 0.5 * qd[1]; ds[o + 5] = 0.5 * qd[2]; ds[o + 6] = 0.5 * qd[3];
            for c in 0..6 { ds[o + 7 + c] = acc[i][c]; }
        }
        ds
    }

    /// Cholesky in place (lower triangle of a, row-major n x n), skipping each row's leading zeros
    /// (first[i]; all 0 when no order was given: the dense solve). Same sums in the same order as
    /// engine.py's chol_solve; a skipped term is an exact zero.
    fn chol_solve(&self, a: &mut [f64], b: &[f64], n: usize) -> Vec<f64> {
        let first = &self.first;
        for i in 0..n {
            let fi = first[i];
            for j in fi..i {
                let lo = if first[j] > fi { first[j] } else { fi };
                let mut sm = 0.0;
                for k in lo..j { sm += a[i * n + k] * a[j * n + k]; }
                a[i * n + j] = (a[i * n + j] - sm) / a[j * n + j];
            }
            let mut sm = 0.0;
            for k in fi..i { sm += a[i * n + k] * a[i * n + k]; }
            let d = a[i * n + i] - sm;
            if !(d > 1e-12 * a[i * n + i].abs()) {
                eprintln!("constraint matrix not positive definite at row {} (pivot {:e} of {:e})", i, d, a[i * n + i]);
                std::process::exit(3);
            }
            a[i * n + i] = d.sqrt();
        }
        let mut y = vec![0.0; n];
        for i in 0..n {
            let mut sm = 0.0;
            for k in first[i]..i { sm += a[i * n + k] * y[k]; }
            y[i] = (b[i] - sm) / a[i * n + i];
        }
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let mut sv = y[i];
            for k in i + 1..n { if first[k] <= i { sv -= a[k * n + i] * x[k]; } }
            x[i] = sv / a[i * n + i];
        }
        x
    }

    fn anchors(&self, s: &[f64], aux: &mut [Option<[f64; 2]>]) {
        for (j, c) in self.cons.iter().enumerate() {
            let o = 13 * c.b;
            let r = qr(&[s[o + 3], s[o + 4], s[o + 5], s[o + 6]]);
            let p = add(&[s[o], s[o + 1], s[o + 2]], &mv(&r, &c.p));
            let d = -p[2];
            if d <= 0.0 { aux[j] = None; }
            else if aux[j].is_none() { aux[j] = Some([p[0], p[1]]); }
            else {
                let a = aux[j].unwrap();
                let (dx, dy) = (p[0] - a[0], p[1] - a[1]);
                let rr = (dx * dx + dy * dy).sqrt();
                let lim = c.mu * c.kn * d / c.kt;
                if rr > lim { aux[j] = Some([p[0] - dx * lim / rr, p[1] - dy * lim / rr]); }
            }
        }
    }

    fn step(&self, t: f64, s: &[f64], aux: &mut [Option<[f64; 2]>], h: f64) -> Vec<f64> {
        let n = s.len();
        let k1 = self.deriv(t, s, aux, None);
        let mut tmp = vec![0.0; n];
        for i in 0..n { tmp[i] = s[i] + 0.5 * h * k1[i]; }
        let k2 = self.deriv(t + 0.5 * h, &tmp, aux, None);
        for i in 0..n { tmp[i] = s[i] + 0.5 * h * k2[i]; }
        let k3 = self.deriv(t + 0.5 * h, &tmp, aux, None);
        for i in 0..n { tmp[i] = s[i] + h * k3[i]; }
        let k4 = self.deriv(t + h, &tmp, aux, None);
        let h6 = h / 6.0;
        let mut out = vec![0.0; n];
        for i in 0..n { out[i] = s[i] + h6 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]); }
        for i in 0..self.nb() {
            let o = 13 * i + 3;
            let qn = (out[o] * out[o] + out[o + 1] * out[o + 1] + out[o + 2] * out[o + 2] + out[o + 3] * out[o + 3]).sqrt();
            out[o] /= qn; out[o + 1] /= qn; out[o + 2] /= qn; out[o + 3] /= qn;
        }
        self.anchors(&out, aux);
        out
    }
}

struct Tok { v: Vec<String>, i: usize }
impl Tok {
    fn key(&mut self, k: &str) { if self.v[self.i] != k { panic!("expected {} at token {}, found {}", k, self.i, self.v[self.i]); } self.i += 1; }
    fn f(&mut self) -> f64 { self.i += 1; self.v[self.i - 1].parse::<f64>().expect("number") }
    fn u(&mut self) -> usize { self.i += 1; self.v[self.i - 1].parse::<usize>().expect("count") }
    fn i(&mut self) -> i32 { self.i += 1; self.v[self.i - 1].parse::<i32>().expect("index") }
    fn v3(&mut self) -> V3 { [self.f(), self.f(), self.f()] }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string(&args[1]).expect("model file");
    let mut tk = Tok { v: text.split_whitespace().map(|s| s.to_string()).collect(), i: 0 };
    tk.key("g"); let g = tk.v3();
    tk.key("baum"); let (baum_a, baum_b) = (tk.f(), tk.f());
    tk.key("consts"); let (ta_up, ta_dn, ecc_slope, d0, reg3, sat_eps, fb_scale, brake) = (tk.f(), tk.f(), tk.f(), tk.f(), tk.f(), tk.f(), tk.f(), tk.f());
    tk.key("bodies"); let nb = tk.u();
    let (mut m, mut inr) = (Vec::new(), Vec::new());
    for _ in 0..nb { m.push(tk.f()); inr.push(tk.v3()); }
    tk.key("pts"); let n = tk.u();
    let pts: Vec<Pt> = (0..n).map(|_| Pt { a: tk.i(), b: tk.i(), ra: tk.v3(), rb: tk.v3() }).collect();
    tk.key("dots"); let n = tk.u();
    let dots: Vec<Dt> = (0..n).map(|_| Dt { a: tk.i(), b: tk.i(), ua: tk.v3(), wb: tk.v3() }).collect();
    tk.key("order"); let n = tk.u();
    let order: Vec<usize> = (0..n).map(|_| tk.u()).collect();
    tk.key("fvtabs"); let n = tk.u();
    let mut fvtabs: Vec<Vec<[f64; 2]>> = Vec::new();
    let mut fvecc: Vec<f64> = Vec::new();
    for _ in 0..n { let np = tk.u(); fvecc.push(tk.f()); fvtabs.push((0..np).map(|_| [tk.f(), tk.f()]).collect()); }
    tk.key("stops"); let n = tk.u();
    let stops: Vec<Stop> = (0..n).map(|_| Stop { a: tk.i(), b: tk.u(), ax: tk.v3(), ea: tk.v3(), fb: tk.v3(), lo: tk.f(), hi: tk.f(), k: tk.f(), c: tk.f(),
                                                  kind: tk.i(), axb: tk.v3(), n2: ZERO, e2: ZERO, f2: ZERO, ov: [0.0; 6] }).collect();
    tk.key("cons"); let n = tk.u();
    let cons: Vec<Con> = (0..n).map(|_| Con { b: tk.u(), p: tk.v3(), kn: tk.f(), cn: tk.f(), kt: tk.f(), ct: tk.f(), mu: tk.f() }).collect();
    tk.key("anchors");
    let mut aux0: Vec<Option<[f64; 2]>> = Vec::new();
    for _ in 0..cons.len() { let flag = tk.u(); let (ax, ay) = (tk.f(), tk.f()); aux0.push(if flag == 1 { Some([ax, ay]) } else { None }); }
    tk.key("joints"); let nj = tk.u();
    let mut joints: Vec<Joint> = Vec::new();
    let mut axes: Vec<Axis> = Vec::new();
    for _ in 0..nj {
        let (a, b, kp, kd, na, jb, rb) = (tk.u(), tk.u(), tk.f(), tk.f(), tk.u(), tk.u(), tk.v3());
        joints.push(Joint { a, b, kp, kd, k0: axes.len(), na, jb, rb });
        for _ in 0..na {
            axes.push(Axis { kind: tk.i(), ax: tk.v3(), axb: tk.v3(), tp: tk.f(), tm: tk.f(), tabp: tk.u(), tabm: tk.u(), damp: tk.f() });
        }
    }
    let nax = axes.len();
    tk.key("root"); let pelvis = tk.u();
    let root = [tk.f(), tk.f(), tk.f(), tk.f(), tk.f()];
    let legs = [tk.u(), tk.u(), tk.u(), tk.u(), tk.u(), tk.u()];
    tk.key("ref"); let (t0ref, dtref, nref) = (tk.f(), tk.f(), tk.u());
    let mut refs: Vec<Sample> = Vec::with_capacity(nref);
    for _ in 0..nref {
        let (mut q, mut w, mut gg) = (Vec::with_capacity(nj), Vec::with_capacity(nj), Vec::with_capacity(nj));
        for _ in 0..nj { q.push([tk.f(), tk.f(), tk.f(), tk.f()]); w.push(tk.v3()); gg.push(tk.f()); }
        let ff: Vec<f64> = (0..nax).map(|_| tk.f()).collect();
        refs.push(Sample { q, w, g: gg, ff, px: tk.v3(), pq: [tk.f(), tk.f(), tk.f(), tk.f()], pv: tk.v3(), pw: tk.v3(), wl: tk.f() });
    }
    tk.key("run"); let (t0, nsteps, h, every) = (tk.f(), tk.u(), tk.f(), tk.u());
    tk.key("state"); let n = tk.u();
    let s0: Vec<f64> = (0..n).map(|_| tk.f()).collect();
    let nrows = 3 * pts.len() + dots.len();
    let mut sys = Sys { g, baum_a, baum_b, ta_up, ta_dn, ecc_slope, d0, reg3, sat_eps, fb_scale, brake, m, inr, pts, dots, order, first: vec![0; nrows],
                        stops, cons, fvtabs, fvecc, joints, axes, pelvis, root, legs, t0ref, dtref, refs };
    if !sys.order.is_empty() {
        let zeros = vec![ZERO; sys.nb()];
        let ident = vec![[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]; sys.nb()];
        let rows = sys.rows(&zeros, &ident, &zeros, &zeros);
        let mut first = vec![0usize; nrows];
        for k in 0..nrows {
            let mut fk = k;
            for l in 0..k {
                let share = (rows[k].a >= 0 && (rows[k].a == rows[l].a || rows[k].a == rows[l].b))
                    || (rows[k].b >= 0 && (rows[k].b == rows[l].a || rows[k].b == rows[l].b));
                if share { fk = l; break; }
            }
            first[k] = fk;
        }
        sys.first = first;
    }
    let out = std::io::stdout();
    // a line at a time: a run that fails (exit 3) has then already handed over every record it made
    let mut out = std::io::LineWriter::new(out.lock());
    let mut s = s0.clone();
    let mut aux = aux0.clone();
    sys.anchors(&s, &mut aux);
    let mut diag = Diag::default();
    let record = |t: f64, s: &[f64], aux: &[Option<[f64; 2]>], diag: &mut Diag, out: &mut dyn Write| {
        sys.deriv(t, s, aux, Some(&mut *diag));
        write!(out, "{:e}", t).unwrap();
        for x in s { write!(out, " {:e}", x).unwrap(); }
        for c in &diag.con { write!(out, " {:e} {:e} {:e}", c[0], c[1], c[2]).unwrap(); }
        for a in &diag.ax { write!(out, " {:e} {:e} {:e} {:e} {:e} {:e}", a[0], a[1], a[2], a[3], a[4], a[5]).unwrap(); }
        for st in &diag.stop { write!(out, " {:e} {:e}", st[0], st[1]).unwrap(); }
        writeln!(out).unwrap();
    };
    record(t0, &s, &aux, &mut diag, &mut out);
    for i in 0..nsteps {
        s = sys.step(t0 + (i as f64) * h, &s, &mut aux, h);
        if (i + 1) % every == 0 { record(t0 + ((i + 1) as f64) * h, &s, &aux, &mut diag, &mut out); }
    }
}
