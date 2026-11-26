// A compiled copy of the three-dimensional body's checking engine (docs/body-motion.md 8.1, 8.2), so
// that fits can run. NOT sim code and not the reference: analysis/oracle/body3d/engine.py is the
// checking model, and analysis/research/body/probe3d.py --check reruns the state-for-state comparison.
// It mirrors engine.py's deriv, step and anchors operation for operation (same order of arithmetic),
// reads a model written by probe3d.py and prints the recorded states and forces.
// The only thing it does differently: the joint rows may be reordered (the model's `order` line) so
// that the Cholesky factor can skip each row's leading zeros (an envelope factorisation that follows
// the body's tree). With no order line the solve is the dense one, in engine.py's row order.
// f64, std sqrt, atan2, asin and (for a held muscle's release) exp only; no SIMD intrinsics, no mul_add.
// Build:  rustc -C opt-level=3 -C linker=$HOME/.local/bin/zigcc -o sim/target/codex-probes/body3d/body3d_probe \
//           analysis/research/body/body3d_probe.rs
// Run:    body3d_probe <model file> [repeats]     (repeats > 0: time the run, print microseconds per step)
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
struct Act { a: i32, b: usize, ax: V3, t: f64, w: f64, ton: f64, rise: f64, toff: f64, a0: f64, tab: i32, ksec: f64, axb: V3, held: i32 }
struct Stop { a: i32, b: usize, ax: V3, ea: V3, fb: V3, lo: f64, hi: f64, k: f64, c: f64, kind: i32, axb: V3 }
const STOP_SINGULAR: f64 = 0.0872;   // engine.py's: a stop's direction this near its axis has no usable angle
struct Hold { a: i32, b: usize, qref: [f64; 4], k: V3, c: V3, cap: f64 }
struct Tone { a: i32, b: usize, tl: V3 }
struct Con { b: usize, p: V3, kn: f64, cn: f64, kt: f64, ct: f64, mu: f64 }
#[derive(Clone, Copy)]
struct Row { a: i32, ja: V6, b: i32, jb: V6, c: f64, cd: f64, gam: f64 }

struct Sys {
    g: V3, baum_a: f64, baum_b: f64,
    ta_up: f64, ta_dn: f64, k_fv: f64, ecc: f64, ecc_slope: f64, d0: f64, ta_rel: f64,
    m: Vec<f64>, inr: Vec<V3>,
    pts: Vec<Pt>, dots: Vec<Dt>, order: Vec<usize>, first: Vec<usize>,
    acts: Vec<Act>, stops: Vec<Stop>, holds: Vec<Hold>, tones: Vec<Tone>, cons: Vec<Con>,
    fvtabs: Vec<Vec<[f64; 2]>>, fvecc: Vec<f64>,
}

const SEC_ITERS: usize = 40;
const A_MIN: f64 = 1e-3;

#[derive(Default)]
struct Diag { con: Vec<V3>, act: Vec<V3>, stop: Vec<[f64; 2]> }

impl Sys {
    fn nb(&self) -> usize { self.m.len() }
    fn fv(&self, w: f64) -> f64 {
        if w >= 1.0 { return 0.0; }
        if w >= 0.0 { return (1.0 - w) / (1.0 + w / self.k_fv); }
        self.ecc - (self.ecc - 1.0) / (1.0 - self.ecc_slope * w)
    }
    /// engine.py's hold_share: the share of a held muscle's own torque that reaches its joint.
    fn hold_share(&self, t: f64, ton: f64, a0: f64, act: f64) -> f64 {
        if a0 <= 0.0 { return 1.0; }
        let left = if t < ton { 1.0 } else { (-(t - ton) / self.ta_rel).exp() };
        if act <= a0 * left { return 0.0; }
        1.0 - a0 * left / act
    }
    fn excitation(&self, j: usize, t: f64) -> f64 {
        let a = &self.acts[j];
        if t >= a.toff { return 0.0; }
        if t < a.ton { return a.a0; }
        if a.rise > 0.0 && t < a.ton + a.rise { return a.a0 + (1.0 - a.a0) * (t - a.ton) / a.rise; }
        1.0
    }
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
    fn fv_tab_inv(&self, tab: &[[f64; 2]], ecc: f64, f: f64) -> f64 {
        let top = tab[tab.len() - 1][0];
        if f <= 0.0 { return top; }
        if f <= 1.0 {
            for k in 0..tab.len() - 1 {
                let (wa, fa, wb, fb) = (tab[k][0], tab[k][1], tab[k + 1][0], tab[k + 1][1]);
                if f >= fb { return if fa == fb { wa } else { wa + (wb - wa) * (fa - f) / (fa - fb) }; }
            }
            return top;
        }
        top * (1.0 - (ecc - 1.0) / (ecc - f)) / self.ecc_slope
    }
    fn act_speed(&self, j: usize, s: &[f64]) -> f64 {
        let a = &self.acts[j];
        let (mut n, wa) = if a.a >= 0 {
            let o = 13 * a.a as usize;
            (mv(&qr(&[s[o + 3], s[o + 4], s[o + 5], s[o + 6]]), &a.ax), [s[o + 10], s[o + 11], s[o + 12]])
        } else { (a.ax, ZERO) };
        let o = 13 * a.b;
        if a.axb != ZERO { n = float_axis(&n, &mv(&qr(&[s[o + 3], s[o + 4], s[o + 5], s[o + 6]]), &a.axb)); }
        dot(&sub(&[s[o + 10], s[o + 11], s[o + 12]], &wa), &n)
    }
    /// engine.py's sec_update: one implicit step of each series spring, by bisection.
    fn sec_update(&self, s: &[f64], sec: &mut [f64], h: f64) {
        let oa = 13 * self.nb();
        for (j, a) in self.acts.iter().enumerate() {
            let k = a.ksec;
            if k <= 0.0 { continue; }
            let tab = &self.fvtabs[a.tab as usize];
            let ecc = self.fvecc[a.tab as usize];
            let cap = a.t * f64::max(s[oa + j], A_MIN);
            let wj = self.act_speed(j, s);
            let old = sec[j];
            let g = |th: f64| th - old - h * (self.fv_tab_inv(tab, ecc, k * th / cap) - wj);
            let mut hi = (ecc - 1e-6) * cap / k;
            if cap <= 0.0 || g(0.0) >= 0.0 { sec[j] = 0.0; }
            else if g(hi) <= 0.0 { sec[j] = hi; }
            else {
                let mut lo = 0.0;
                for _ in 0..SEC_ITERS {
                    let mid = 0.5 * (lo + hi);
                    if g(mid) > 0.0 { hi = mid; } else { lo = mid; }
                }
                sec[j] = 0.5 * (lo + hi);
            }
        }
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

    fn deriv(&self, t: f64, s: &[f64], aux: &[Option<[f64; 2]>], sec: &[f64], mut diag: Option<&mut Diag>) -> Vec<f64> {
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
        // ---- forces, in engine.py's order: drives, stops, holds, tones, contacts ----
        let oa = 13 * nb;
        let ow = oa + self.acts.len();
        let (mut p_act, mut p_stop, mut p_hold, mut p_con) = (0.0, 0.0, 0.0, 0.0);
        if let Some(d) = diag.as_deref_mut() { d.con.clear(); d.act.clear(); d.stop.clear(); }
        for (j, a) in self.acts.iter().enumerate() {
            let act = s[oa + j];
            let u = self.excitation(j, t);
            ds[oa + j] = (u - act) / (if u > act { self.ta_up } else { self.ta_dn });
            let mut n = if a.a >= 0 { mv(&r[a.a as usize], &a.ax) } else { a.ax };
            if a.axb != ZERO { n = float_axis(&n, &mv(&r[a.b], &a.axb)); }
            let wa = if a.a >= 0 { w[a.a as usize] } else { ZERO };
            let wrel = dot(&sub(&w[a.b], &wa), &n);
            let mut tau = if a.ksec > 0.0 { a.ksec * sec[j] } else if a.tab >= 0 { a.t * act * self.fv_tab(&self.fvtabs[a.tab as usize], self.fvecc[a.tab as usize], wrel) } else { a.t * act * self.fv(wrel / a.w) };
            if a.held != 0 { tau *= self.hold_share(t, a.ton, a.a0, act); }
            let tv = scale(&n, tau);
            tq[a.b] = add(&tq[a.b], &tv);
            if a.a >= 0 { tq[a.a as usize] = sub(&tq[a.a as usize], &tv); }
            p_act += tau * wrel;
            if let Some(d) = diag.as_deref_mut() { d.act.push([tau, wrel, act]); }
        }
        for st in &self.stops {
            let n = if st.a >= 0 { mv(&r[st.a as usize], &st.ax) } else { st.ax };
            let e = if st.a >= 0 { mv(&r[st.a as usize], &st.ea) } else { st.ea };
            let fb = mv(&r[st.b], &st.fb);
            // engine.py's stop_angle: the angle and its own gradient g (d(angle)/dt = w_rel . g); the torque acts along g
            let nf = dot(&n, &fb);
            let (th, g) = if st.kind == 2 {
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
        for h in &self.holds {
            let o = 13 * h.b;
            let qb = [s[o + 3], s[o + 4], s[o + 5], s[o + 6]];
            let qrel = if h.a >= 0 {
                let o = 13 * h.a as usize;
                qmul(&qconj(&[s[o + 3], s[o + 4], s[o + 5], s[o + 6]]), &qb)
            } else { qb };
            let mut qe = qmul(&qrel, &qconj(&h.qref));
            if qe[0] < 0.0 { qe = [-qe[0], -qe[1], -qe[2], -qe[3]]; }
            let sn = (qe[1] * qe[1] + qe[2] * qe[2] + qe[3] * qe[3]).sqrt();
            let rv = if sn > 1e-12 {
                let ang = 2.0 * sn.atan2(qe[0]);
                [qe[1] / sn * ang, qe[2] / sn * ang, qe[3] / sn * ang]
            } else { [2.0 * qe[1], 2.0 * qe[2], 2.0 * qe[3]] };
            let wa = if h.a >= 0 { w[h.a as usize] } else { ZERO };
            let wr = sub(&w[h.b], &wa);
            let wl = if h.a >= 0 { mtv(&r[h.a as usize], &wr) } else { wr };
            let mut tl = [-h.k[0] * rv[0] - h.c[0] * wl[0], -h.k[1] * rv[1] - h.c[1] * wl[1], -h.k[2] * rv[2] - h.c[2] * wl[2]];
            let mag = norm(&tl);
            if mag > h.cap { tl = scale(&tl, h.cap / mag); }
            let tv = if h.a >= 0 { mv(&r[h.a as usize], &tl) } else { tl };
            tq[h.b] = add(&tq[h.b], &tv);
            if h.a >= 0 { tq[h.a as usize] = sub(&tq[h.a as usize], &tv); }
            p_hold += dot(&tv, &wr);
        }
        for tn in &self.tones {
            let tv = if tn.a >= 0 { mv(&r[tn.a as usize], &tn.tl) } else { tn.tl };
            tq[tn.b] = add(&tq[tn.b], &tv);
            if tn.a >= 0 { tq[tn.a as usize] = sub(&tq[tn.a as usize], &tv); }
            let wa = if tn.a >= 0 { w[tn.a as usize] } else { ZERO };
            p_hold += dot(&tv, &sub(&w[tn.b], &wa));
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
        ds[ow] = p_act; ds[ow + 1] = p_stop; ds[ow + 2] = p_hold; ds[ow + 3] = p_con;
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

    fn step(&self, t: f64, s: &[f64], aux: &mut [Option<[f64; 2]>], sec: &mut [f64], h: f64) -> Vec<f64> {
        let n = s.len();
        let k1 = self.deriv(t, s, aux, sec, None);
        let mut tmp = vec![0.0; n];
        for i in 0..n { tmp[i] = s[i] + 0.5 * h * k1[i]; }
        let k2 = self.deriv(t + 0.5 * h, &tmp, aux, sec, None);
        for i in 0..n { tmp[i] = s[i] + 0.5 * h * k2[i]; }
        let k3 = self.deriv(t + 0.5 * h, &tmp, aux, sec, None);
        for i in 0..n { tmp[i] = s[i] + h * k3[i]; }
        let k4 = self.deriv(t + h, &tmp, aux, sec, None);
        let h6 = h / 6.0;
        let mut out = vec![0.0; n];
        for i in 0..n { out[i] = s[i] + h6 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]); }
        for i in 0..self.nb() {
            let o = 13 * i + 3;
            let qn = (out[o] * out[o] + out[o + 1] * out[o + 1] + out[o + 2] * out[o + 2] + out[o + 3] * out[o + 3]).sqrt();
            out[o] /= qn; out[o + 1] /= qn; out[o + 2] /= qn; out[o + 3] /= qn;
        }
        self.anchors(&out, aux);
        self.sec_update(&out, sec, h);
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
    let reps: usize = if args.len() > 2 { args[2].parse().unwrap() } else { 0 };
    let mut tk = Tok { v: text.split_whitespace().map(|s| s.to_string()).collect(), i: 0 };
    tk.key("g"); let g = tk.v3();
    tk.key("baum"); let (baum_a, baum_b) = (tk.f(), tk.f());
    tk.key("consts"); let (ta_up, ta_dn, k_fv, ecc, ecc_slope, d0, ta_rel) = (tk.f(), tk.f(), tk.f(), tk.f(), tk.f(), tk.f(), tk.f());
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
    tk.key("acts"); let n = tk.u();
    let acts: Vec<Act> = (0..n).map(|_| Act { a: tk.i(), b: tk.u(), ax: tk.v3(), t: tk.f(), w: tk.f(), ton: tk.f(), rise: tk.f(), toff: tk.f(),
                                               a0: tk.f(), tab: tk.i(), ksec: tk.f(), axb: tk.v3(), held: tk.i() }).collect();
    tk.key("stops"); let n = tk.u();
    let stops: Vec<Stop> = (0..n).map(|_| Stop { a: tk.i(), b: tk.u(), ax: tk.v3(), ea: tk.v3(), fb: tk.v3(), lo: tk.f(), hi: tk.f(), k: tk.f(), c: tk.f(), kind: tk.i(), axb: tk.v3() }).collect();
    tk.key("holds"); let n = tk.u();
    let holds: Vec<Hold> = (0..n).map(|_| Hold { a: tk.i(), b: tk.u(), qref: [tk.f(), tk.f(), tk.f(), tk.f()], k: tk.v3(), c: tk.v3(), cap: tk.f() }).collect();
    tk.key("tones"); let n = tk.u();
    let tones: Vec<Tone> = (0..n).map(|_| Tone { a: tk.i(), b: tk.u(), tl: tk.v3() }).collect();
    tk.key("cons"); let n = tk.u();
    let cons: Vec<Con> = (0..n).map(|_| Con { b: tk.u(), p: tk.v3(), kn: tk.f(), cn: tk.f(), kt: tk.f(), ct: tk.f(), mu: tk.f() }).collect();
    tk.key("anchors");
    let mut aux0: Vec<Option<[f64; 2]>> = Vec::new();
    for _ in 0..cons.len() { let flag = tk.u(); let (ax, ay) = (tk.f(), tk.f()); aux0.push(if flag == 1 { Some([ax, ay]) } else { None }); }
    tk.key("sec"); let n = tk.u();
    let sec0: Vec<f64> = (0..n).map(|_| tk.f()).collect();
    tk.key("run"); let (t0, nsteps, h, every) = (tk.f(), tk.u(), tk.f(), tk.u());
    tk.key("state"); let n = tk.u();
    let s0: Vec<f64> = (0..n).map(|_| tk.f()).collect();
    let nrows = 3 * pts.len() + dots.len();
    let mut sys = Sys { g, baum_a, baum_b, ta_up, ta_dn, k_fv, ecc, ecc_slope, d0, ta_rel, m, inr, pts, dots, order, first: vec![0; nrows],
                        acts, stops, holds, tones, cons, fvtabs, fvecc };
    if !sys.order.is_empty() {
        // each row's first structural non-zero: the earliest row sharing a body with it; then the
        // envelope is closed under fill (a row's envelope reaches at least as far left as ... its own first)
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
    if reps > 0 {
        let t = std::time::Instant::now();
        let mut chk = 0.0;
        for _ in 0..reps {
            let mut s = s0.clone();
            let mut aux = aux0.clone();
            let mut sec = sec0.clone();
            sys.anchors(&s, &mut aux);
            for i in 0..nsteps { s = sys.step(t0 + (i as f64) * h, &s, &mut aux, &mut sec, h); }
            chk += s[0];
        }
        let us = t.elapsed().as_secs_f64() * 1e6 / (reps * nsteps) as f64;
        println!("{:.2} us per RK4 step ({} rows, {} bodies, {} steps x {} runs; checksum {:e})", us, nrows, sys.nb(), nsteps, reps, chk);
        return;
    }
    let out = std::io::stdout();
    let mut out = std::io::BufWriter::new(out.lock());
    let mut s = s0.clone();
    let mut aux = aux0.clone();
    let mut sec = sec0.clone();
    sys.anchors(&s, &mut aux);
    let mut diag = Diag::default();
    let mut record = |t: f64, s: &[f64], aux: &[Option<[f64; 2]>], sec: &[f64], out: &mut dyn Write| {
        sys.deriv(t, s, aux, sec, Some(&mut diag));
        write!(out, "{:e}", t).unwrap();
        for x in s { write!(out, " {:e}", x).unwrap(); }
        for c in &diag.con { write!(out, " {:e} {:e} {:e}", c[0], c[1], c[2]).unwrap(); }
        for a in &diag.act { write!(out, " {:e} {:e} {:e}", a[0], a[1], a[2]).unwrap(); }
        for st in &diag.stop { write!(out, " {:e} {:e}", st[0], st[1]).unwrap(); }
        for x in sec { write!(out, " {:e}", x).unwrap(); }
        writeln!(out).unwrap();
    };
    record(t0, &s, &aux, &sec, &mut out);
    for i in 0..nsteps {
        s = sys.step(t0 + (i as f64) * h, &s, &mut aux, &mut sec, h);
        if (i + 1) % every == 0 { record(t0 + ((i + 1) as f64) * h, &s, &aux, &sec, &mut out); }
    }
}
