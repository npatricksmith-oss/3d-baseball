//! Batter perception and decisions (agent-models §3.1–§3.5).
//!
//! The batter never sees the pitch's true state. It keeps one hypothesis per pitch type in the
//! pitcher's arsenal (from public scouting), each an extended Kalman filter on where and when
//! the ball will cross the plate plane, s = (x_p, z_p, t_p). Observations are noisy eye
//! angles, a time-to-arrival estimate, and a spin cue that sharpens with viewing time.

use bb_math::{Mat3, Vec3, m};
use bb_observe::{BallObs, Count, Hand, PitchType, ReleaseObs};
use bb_physics::flight::{Crossing, Trajectory};
use bb_physics::zone::{HALF_WIDTH, Y_JUDGE};

/// Perception parameters from `Vision` and `SpinRecognition` (agent-models §10).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Perception {
    /// Angular σ, rad.
    pub sigma_ang: f64,
    /// Weber fraction on time-to-arrival.
    pub weber: f64,
    /// Spin-cue σ at zero viewing time and its sharpening time constant, s.
    pub sigma_s0: f64,
    pub tau_s: f64,
    /// Observations stop when the ball is this close to the eye plane, m.
    pub blank_dist: f64,
    /// Decision-to-motor latency, s.
    pub latency: f64,
    /// Share of perceptual noise variance that persists for the whole pitch (a per-pitch
    /// misjudgment the batter cannot average away) [E]. The rest is independent per sample.
    pub persistent_share: f64,
    /// The batter's belief about how much a type's break varies pitch to pitch: σ of the
    /// extra plate displacement, m [E].
    pub break_sigma: f64,
    /// Smooth-pursuit limit, rad/s [E, ~70°/s; R9]. Above it the eyes fall behind the ball
    /// and angular noise grows as (ω/ω_p)² (Bahill & LaRitz 1984).
    pub pursuit_limit: f64,
}

impl Perception {
    /// Angular σ for a ball moving at angular speed `omega` (rad/s) relative to the eye.
    pub fn angular_sigma(&self, omega: f64) -> f64 {
        let r = (omega / self.pursuit_limit).max(1.0);
        self.sigma_ang * r * r
    }
}

impl Default for Perception {
    fn default() -> Self {
        Self {
            sigma_ang: 0.001,
            weber: 0.10,
            sigma_s0: 1.0,
            tau_s: 0.08,
            blank_dist: 1.5,
            latency: 0.05,
            persistent_share: 0.1,
            break_sigma: 0.06,
            pursuit_limit: 1.2,
        }
    }
}

/// `Discipline`: logit(P̃) = c_D·logit(P) + b_D (agent-models §3.5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Discipline {
    pub c: f64,
    pub b: f64,
}

impl Default for Discipline {
    fn default() -> Self {
        Self { c: 0.5, b: 0.9 }
    }
}

impl Discipline {
    pub fn distort(&self, p: f64) -> f64 {
        let p = p.clamp(1e-6, 1.0 - 1e-6);
        let l = self.c * m::ln(p / (1.0 - p)) + self.b;
        1.0 / (1.0 + m::exp(-l))
    }
}

/// The batter's eye point (agent-models §3.2).
pub fn eye_point(bats: Hand, height: f64) -> Vec3 {
    let side = match bats {
        Hand::Right => -1.0,
        Hand::Left => 1.0,
    };
    Vec3::new(side * 0.75, 0.30, 0.86 * height)
}

/// Eye angles of a point (azimuth toward +X, elevation).
pub fn angles(eye: Vec3, p: Vec3) -> (f64, f64) {
    let d = p - eye;
    (m::atan2(d.x, d.y), m::atan2(d.z, m::hypot(d.x, d.y)))
}

/// What the batter believes about one pitch type before seeing it (public scouting plus
/// in-game memory): a nominal path from a typical release to a typical target, and the
/// type's spin signature.
#[derive(Clone, Debug)]
pub struct TypeModel {
    pub ty: PitchType,
    /// Nominal predicted flight (release at t = 0) to the plate plane.
    pub nominal: Trajectory,
    /// Where the nominal path crosses the plate plane.
    pub x0: f64,
    pub z0: f64,
    pub t0: f64,
    /// Release point of the nominal path.
    pub release: Vec3,
    pub signature: [f64; 3],
    /// Expected release speed of this type, m/s.
    pub speed: f64,
}

/// Scouting knowledge of a pitcher: one model per arsenal pitch plus usage.
#[derive(Clone, Debug)]
pub struct Scouting {
    pub types: Vec<TypeModel>,
    /// Usage share per type (same order as `types`).
    pub usage: Vec<f64>,
}

/// Prior location spread of a pitch around the nominal target, m [E].
pub const PRIOR_LOC_SIGMA: f64 = 0.30;
/// Prior arrival-time spread, s [E] (≈ 5% speed uncertainty).
pub const PRIOR_T_SIGMA: f64 = 0.02;
/// Velocity anchoring rate a_anchor (agent-models §3.1) [E].
pub const ANCHOR: f64 = 0.2;

/// Release-point estimate error at grade 50 and K_perception 1, m (agent-models §3.2) [E].
pub const RELEASE_SIGMA: f64 = 0.05;
const SPIN_WEIGHT_FLOOR: f64 = -50.0;

/// Filter state: plate crossing (x_p, z_p), arrival t_p (from release), the break
/// deviation (b_x, b_z) at the plate, the release point (r_x, r_z), and the batter's own
/// persistent perceptual biases (azimuth, elevation, relative time-to-arrival). The path is
/// (1−f)·release + f·plate + the type's nominal curvature + f²·break, with f the flight
/// progress: early flight pins the release, late flight the plate, and the break shows
/// only late. Modeling the biases as states keeps correlated errors from being counted as
/// fresh evidence on every sample (agent-models §3.3, M3 extension; Codex M3 review #5).
pub const NS: usize = 10;

/// Prior σ of the batter's persistent perceptual biases (it knows its own acuity).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BiasPrior {
    pub ang: f64,
    pub tau_rel: f64,
}
pub type SVec = [f64; NS];
pub type SMat = [[f64; NS]; NS];

#[derive(Clone, Debug)]
pub struct Hypothesis {
    pub ty: PitchType,
    /// log prior weight
    pub log_prior: f64,
    /// accumulated log likelihood of position/time observations
    pub log_lik: f64,
    /// log likelihood of the latest spin cue (replaced, not accumulated)
    pub log_spin: f64,
    /// state mean and covariance
    pub s: SVec,
    pub p: SMat,
    model: usize,
}

impl Hypothesis {
    /// Predicted plate crossing (x, z) including the break deviation, and t_p.
    pub fn plate(&self) -> Vec3 {
        Vec3::new(self.s[0] + self.s[3], self.s[1] + self.s[4], self.s[2])
    }
    /// Covariance of [`Hypothesis::plate`].
    pub fn plate_cov(&self) -> Mat3 {
        // plate = A s with rows picking (x_p + b_x), (z_p + b_z), t_p, so (A P Aᵀ)ᵢⱼ is the
        // sum of P over the state indices each row picks.
        const PICK: [&[usize]; 3] = [&[0, 3], &[1, 4], &[2]];
        let mut c = Mat3::ZERO;
        for i in 0..3 {
            for j in 0..3 {
                let mut v = 0.0;
                for &k in PICK[i] {
                    for &l in PICK[j] {
                        v += self.p[k][l];
                    }
                }
                c.r[i][j] = v;
            }
        }
        c
    }
}

/// The per-pitch hypothesis bank.
#[derive(Clone, Debug)]
pub struct Bank {
    pub hyps: Vec<Hypothesis>,
    pub t_release: f64,
    pub eye: Vec3,
    pub n_obs: u32,
}

/// One mixture component of the posterior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Component {
    pub ty: PitchType,
    pub w: f64,
    /// Predicted plate crossing (x, z) and time.
    pub mean: Vec3,
    pub cov: Mat3,
}

impl Bank {
    /// Start a pitch: priors π_k and break-spread beliefs per type (same order as scouting
    /// types; priors ≤ 0.01 are dropped), plus the batter's own perceptual bias spread.
    pub fn new(sc: &Scouting, prior: &[f64], rel: &ReleaseObs, eye: Vec3, break_sigma: &[f64], bias: BiasPrior) -> Bank {
        Self::new_anchored(sc, prior, rel, eye, break_sigma, bias, None)
    }

    /// [`Bank::new`] with velocity anchoring (agent-models §3.1): each type's expected speed
    /// moves toward the speed perceived on the previous pitch, μ ← μ + a·(v_last − μ).
    #[allow(clippy::too_many_arguments)]
    pub fn new_anchored(sc: &Scouting, prior: &[f64], rel: &ReleaseObs, eye: Vec3, break_sigma: &[f64], bias: BiasPrior, last_speed: Option<f64>) -> Bank {
        let mut hyps = Vec::new();
        for (k, tm) in sc.types.iter().enumerate() {
            if prior[k] <= 0.01 {
                continue;
            }
            // Anchored speed shortens or lengthens the expected flight time proportionally.
            let t_scale = match last_speed {
                Some(v) => tm.speed / (tm.speed + ANCHOR * (v - tm.speed)),
                None => 1.0,
            };
            let s = [tm.x0, tm.z0, tm.t0 * t_scale, 0.0, 0.0, rel.r.x, rel.r.z, 0.0, 0.0, 0.0];
            let l2 = PRIOR_LOC_SIGMA * PRIOR_LOC_SIGMA;
            let b2 = break_sigma[k] * break_sigma[k];
            let mut p = [[0.0; NS]; NS];
            p[0][0] = l2;
            p[1][1] = l2;
            p[2][2] = PRIOR_T_SIGMA * PRIOR_T_SIGMA;
            p[3][3] = b2;
            p[4][4] = b2;
            p[5][5] = rel.sigma * rel.sigma;
            p[6][6] = rel.sigma * rel.sigma;
            p[7][7] = bias.ang * bias.ang;
            p[8][8] = bias.ang * bias.ang;
            p[9][9] = bias.tau_rel * bias.tau_rel;
            hyps.push(Hypothesis { ty: tm.ty, log_prior: m::ln(prior[k]), log_lik: 0.0, log_spin: 0.0, s, p, model: k });
        }
        Bank { hyps, t_release: rel.t, eye, n_obs: 0 }
    }

    /// Predicted ball position at play time `t` under hypothesis state `s`.
    fn predict(sc: &Scouting, h: &Hypothesis, s: &SVec, t_rel: f64, t: f64) -> Vec3 {
        Self::predict_d(sc, h, s, t_rel, t).0
    }

    /// [`Bank::predict`], plus the flight progress f, ∂p/∂f, and ∂f/∂t_p (state s[2]).
    fn predict_d(sc: &Scouting, h: &Hypothesis, s: &SVec, t_rel: f64, t: f64) -> (Vec3, f64, Vec3, f64) {
        let tm = &sc.types[h.model];
        // Progress along the flight, scaled by the hypothesized arrival time.
        let raw = (t - t_rel) / s[2];
        let f = raw.max(0.0);
        let tn = f * tm.t0;
        let (n, nv) = tm.nominal.at(tn);
        // The nominal path's deviation from its own release→plate chord: gravity and break.
        let chord = tm.release * (1.0 - f) + Vec3::new(tm.x0, n.y, tm.z0) * f;
        let curve = Vec3::new(n.x - chord.x, 0.0, n.z - chord.z);
        let x = (1.0 - f) * s[5] + f * s[0] + curve.x + s[3] * f * f;
        let z = (1.0 - f) * s[6] + f * s[1] + curve.z + s[4] * f * f;
        // The nominal is clamped (flat) outside its time span.
        let inside = raw > 0.0 && tn > tm.nominal.first().t && tn < tm.nominal.last().t;
        let dn = if inside { nv * tm.t0 } else { Vec3::ZERO };
        let dp_df = Vec3::new(
            s[0] - s[5] + dn.x + tm.release.x - tm.x0 + 2.0 * s[3] * f,
            dn.y,
            s[1] - s[6] + dn.z + tm.release.z - tm.z0 + 2.0 * s[4] * f,
        );
        let df_ds2 = if raw > 0.0 { -f / s[2] } else { 0.0 };
        (Vec3::new(x, n.y, z), f, dp_df, df_ds2)
    }

    /// The observation model h(s) — eye angles plus the batter's bias states, and the
    /// perceived time to arrival — with its Jacobian ∂h/∂s (3 × NS). The predicted position is
    /// linear in every state but t_p, which enters through the flight progress f.
    fn h_jac(sc: &Scouting, hyp: &Hypothesis, s: &SVec, t_rel: f64, eye: Vec3, t: f64) -> (Vec3, [[f64; NS]; 3]) {
        let (p, f, dp_df, df_ds2) = Self::predict_d(sc, hyp, s, t_rel, t);
        // Eye angles and their gradients with respect to the ball position.
        let d = p - eye;
        let rho2 = d.x * d.x + d.y * d.y;
        let rho = m::sqrt(rho2);
        let r2 = rho2 + d.z * d.z;
        let (az, el) = (m::atan2(d.x, d.y), m::atan2(d.z, rho));
        let g_az = Vec3::new(d.y / rho2, -d.x / rho2, 0.0);
        let g_el = Vec3::new(-d.z * d.x / (rho * r2), -d.z * d.y / (rho * r2), rho / r2);
        // Time to arrival at the plate plane under this hypothesis, as perceived (with the
        // batter's own persistent relative bias).
        let tau = ((t_rel + s[2]) - t) * (1.0 + s[9]);
        let mut j = [[0.0; NS]; 3];
        // Position columns: x depends on s0, s3, s5; z on s1, s4, s6; all on s2 through f.
        let dp = |k: usize| -> Vec3 {
            match k {
                0 => Vec3::new(f, 0.0, 0.0),
                1 => Vec3::new(0.0, 0.0, f),
                2 => dp_df * df_ds2,
                3 => Vec3::new(f * f, 0.0, 0.0),
                4 => Vec3::new(0.0, 0.0, f * f),
                5 => Vec3::new(1.0 - f, 0.0, 0.0),
                6 => Vec3::new(0.0, 0.0, 1.0 - f),
                _ => Vec3::ZERO,
            }
        };
        for k in 0..7 {
            let v = dp(k);
            j[0][k] = g_az.dot(v);
            j[1][k] = g_el.dot(v);
        }
        j[0][7] = 1.0;
        j[1][8] = 1.0;
        j[2][2] = 1.0 + s[9];
        j[2][9] = (t_rel + s[2]) - t;
        (Vec3::new(az + s[7], el + s[8], tau), j)
    }

    /// EKF update with one observation (agent-models §3.3).
    pub fn observe(&mut self, sc: &Scouting, o: &BallObs) {
        let eye = self.eye;
        let t_rel = self.t_release;
        for hyp in &mut self.hyps {
            let s = hyp.s;
            let (z, hm) = Self::h_jac(sc, hyp, &s, t_rel, eye, o.t);
            let z = z.to_array();
            // S = H P Hᵀ + R
            let mut ph = [[0.0; 3]; NS]; // P Hᵀ
            for i in 0..NS {
                for j in 0..3 {
                    ph[i][j] = (0..NS).map(|k| hyp.p[i][k] * hm[j][k]).sum();
                }
            }
            let mut ss = Mat3::ZERO;
            for i in 0..3 {
                for j in 0..3 {
                    ss.r[i][j] = (0..NS).map(|k| hm[i][k] * ph[k][j]).sum();
                }
            }
            ss.r[0][0] += o.sigma_ang * o.sigma_ang;
            ss.r[1][1] += o.sigma_ang * o.sigma_ang;
            ss.r[2][2] += o.sigma_tau * o.sigma_tau;
            let Some(si) = ss.inverse() else { continue };
            let innov = Vec3::new(o.az - z[0], o.el - z[1], o.tau - z[2]);
            let det = ss.det().max(1e-300);
            hyp.log_lik += -0.5 * (innov.dot(si * innov) + m::ln(det)) - 1.5 * m::ln(2.0 * core::f64::consts::PI);
            // K = P Hᵀ S⁻¹ (NS × 3)
            let mut k = [[0.0; 3]; NS];
            for i in 0..NS {
                for j in 0..3 {
                    k[i][j] = (0..3).map(|l| ph[i][l] * si.r[l][j]).sum();
                }
            }
            let iv = innov.to_array();
            for i in 0..NS {
                hyp.s[i] = s[i] + (0..3).map(|j| k[i][j] * iv[j]).sum::<f64>();
            }
            // P ← (I − K H) P
            let mut np = [[0.0; NS]; NS];
            for i in 0..NS {
                for j in 0..NS {
                    let mut v = hyp.p[i][j];
                    for l in 0..3 {
                        v -= k[i][l] * ph[j][l];
                    }
                    np[i][j] = v;
                }
            }
            // Symmetrize against round-off.
            for i in 0..NS {
                for j in 0..i {
                    let a = 0.5 * (np[i][j] + np[j][i]);
                    np[i][j] = a;
                    np[j][i] = a;
                }
            }
            hyp.p = np;
            // Spin cue: weight-only, latest reading replaces the previous one.
            let sig = sc.types[hyp.model].signature;
            let mut q = 0.0;
            for c in 0..3 {
                let d = o.spin_cue[c] - sig[c];
                q += d * d;
            }
            hyp.log_spin = (-0.5 * q / (o.sigma_spin * o.sigma_spin)).max(SPIN_WEIGHT_FLOOR);
        }
        self.n_obs += 1;
    }

    /// Posterior mixture, weights normalized.
    pub fn posterior(&self) -> Vec<Component> {
        let lw: Vec<f64> = self.hyps.iter().map(|h| h.log_prior + h.log_lik + h.log_spin).collect();
        let mx = lw.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let ws: Vec<f64> = lw.iter().map(|l| m::exp(l - mx)).collect();
        let tot: f64 = ws.iter().sum();
        self.hyps
            .iter()
            .zip(ws)
            .map(|(h, w)| Component { ty: h.ty, w: w / tot, mean: h.plate(), cov: h.plate_cov() })
            .collect()
    }

    /// The speed the batter perceived for this pitch: the dominant type's expected speed
    /// scaled by how much sooner or later it is now believed to arrive.
    pub fn perceived_speed(&self, sc: &Scouting) -> f64 {
        let d = self.dominant();
        match sc.types.iter().find(|t| t.ty == d.ty) {
            Some(tm) => tm.speed * tm.t0 / d.mean.z.max(1e-3),
            None => 0.0,
        }
    }

    /// Weighted mean arrival time at the plate plane (play time).
    pub fn arrival(&self) -> f64 {
        let lw: Vec<f64> = self.hyps.iter().map(|h| h.log_prior + h.log_lik + h.log_spin).collect();
        let mx = lw.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let ws: Vec<f64> = lw.iter().map(|l| m::exp(l - mx)).collect();
        let tot: f64 = ws.iter().sum();
        self.t_release + self.hyps.iter().zip(ws).map(|(h, w)| w / tot * h.s[2]).sum::<f64>()
    }

    /// The dominant mode (agent-models §3.5): highest-weight component, merged with a close
    /// runner-up.
    pub fn dominant(&self) -> Component {
        let mut post = self.posterior();
        post.sort_by(|a, b| b.w.total_cmp(&a.w));
        let a = post[0];
        if let Some(b) = post.get(1) {
            let close = (a.w - b.w) < 0.1
                && m::hypot(a.mean.x - b.mean.x, a.mean.y - b.mean.y) < 0.03
                && (a.mean.z - b.mean.z).abs() < 0.005;
            if close {
                let w = a.w + b.w;
                let mean = (a.mean * a.w + b.mean * b.w) / w;
                return Component { ty: a.ty, w, mean, cov: a.cov };
            }
        }
        a
    }

    /// The predicted path point at depth `y` under a component's hypothesis (for the aim).
    pub fn point_at_depth(&self, sc: &Scouting, ty: PitchType, y: f64) -> Option<(Vec3, f64)> {
        let h = self.hyps.iter().find(|h| h.ty == ty)?;
        // The predicted depth is the nominal path's at the same flight progress, so the
        // crossing is the nominal's own, rescaled to this hypothesis' arrival time. Searched
        // from mid-flight to 50 ms past the believed arrival.
        let tm = &sc.types[h.model];
        let tn = tm.nominal.find(Crossing::YBelow(y))?;
        let t = self.t_release + tn / tm.t0 * h.s[2];
        if t < self.t_release + 0.5 * h.s[2] || t >= self.t_release + h.s[2] + 0.05 {
            return None;
        }
        Some((Self::predict(sc, h, &h.s, self.t_release, t), t))
    }
}

/// Build a type model from a nominal predicted trajectory (computed by the caller with
/// `bb_physics` from scouting data: typical release, typical target, the type's shape).
pub fn type_model(ty: PitchType, nominal: Trajectory, signature: [f64; 3]) -> TypeModel {
    let t0 = nominal.find(Crossing::YBelow(Y_JUDGE)).unwrap_or(nominal.last().t);
    let (r0, _) = nominal.at(t0);
    let release = nominal.first().s.r;
    let speed = nominal.first().s.v.norm();
    TypeModel { ty, x0: r0.x, z0: r0.z, t0, release, nominal, signature, speed }
}

/// Standard normal CDF.
pub fn phi(x: f64) -> f64 {
    0.5 * (1.0 + m::erf(x / core::f64::consts::SQRT_2))
}

/// P(strike) for a Gaussian plate location with full x–z covariance: the zone rectangle
/// grown by the ball radius (rounded corners ignored; this is a belief, not the umpire).
/// Integrates the conditional z-distribution over x with Simpson's rule.
pub fn p_strike(mean_x: f64, mean_z: f64, sxx: f64, sxz: f64, szz: f64, bottom: f64, top: f64, ball_r: f64) -> f64 {
    let sx = m::sqrt(sxx.max(1e-12));
    let sz = m::sqrt(szz.max(1e-12));
    let rho = (sxz / (sx * sz)).clamp(-0.999, 0.999);
    let (x0, x1) = (-HALF_WIDTH - ball_r, HALF_WIDTH + ball_r);
    let (z0, z1) = (bottom - ball_r, top + ball_r);
    let lo = x0.max(mean_x - 5.0 * sx);
    let hi = x1.min(mean_x + 5.0 * sx);
    if lo >= hi {
        return 0.0;
    }
    const N: usize = 24;
    let h = (hi - lo) / N as f64;
    let sc = sz * m::sqrt(1.0 - rho * rho);
    let mut acc = 0.0;
    for i in 0..=N {
        let x = lo + h * i as f64;
        let u = (x - mean_x) / sx;
        let pdf = m::exp(-0.5 * u * u) / (sx * m::sqrt(2.0 * core::f64::consts::PI));
        let mz = mean_z + rho * sz * u;
        let pz = phi((z1 - mz) / sc) - phi((z0 - mz) / sc);
        let w = if i == 0 || i == N { 1.0 } else if i % 2 == 1 { 4.0 } else { 2.0 };
        acc += w * pdf * pz;
    }
    (acc * h / 3.0).clamp(0.0, 1.0)
}

/// Run values used before learned tables exist (agent-models §8 bootstrap) [E].
pub mod values {
    use bb_observe::Count;

    /// Expected runs of the rest of the PA from each count, relative to 0-0 (public-style
    /// count values).
    pub fn count_value(c: Count) -> f64 {
        const V: [[f64; 3]; 4] = [
            [0.000, -0.040, -0.090],
            [0.035, -0.010, -0.065],
            [0.100, 0.050, -0.025],
            [0.190, 0.150, 0.060],
        ];
        V[c.balls.min(3) as usize][c.strikes.min(2) as usize]
    }
    pub const WALK: f64 = 0.32;
    pub const STRIKEOUT: f64 = -0.27;

    pub fn after_ball(c: Count) -> f64 {
        if c.balls >= 3 { WALK } else { count_value(Count { balls: c.balls + 1, strikes: c.strikes }) }
    }
    pub fn after_strike(c: Count) -> f64 {
        if c.strikes >= 2 { STRIKEOUT } else { count_value(Count { balls: c.balls, strikes: c.strikes + 1 }) }
    }
    /// Value of putting a pitch at this location in play, relative to a 0-0 start [E]:
    /// best in the heart of the zone, falling off outside it (public wOBAcon-style scale).
    pub fn bip_value(dist_out_of_heart: f64) -> f64 {
        0.10 - 0.6 * dist_out_of_heart.max(0.0)
    }
    /// Share of contact the batter expects to go foul [E] (calibration-targets B1.15 ≈ 50%).
    pub const FOUL_SHARE: f64 = 0.5;
    /// Value of making contact: a foul (a strike before two strikes, nothing after) or a
    /// ball in play.
    pub fn contact_value(c: Count, dist_out_of_heart: f64) -> f64 {
        let foul = if c.strikes < 2 { after_strike(c) } else { count_value(c) };
        FOUL_SHARE * foul + (1.0 - FOUL_SHARE) * bip_value(dist_out_of_heart)
    }
}

/// The batter's swing-or-take decision at commit (agent-models §3.5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decision {
    pub swing: bool,
    pub aim: Component,
    pub p_strike: f64,
    pub p_strike_perceived: f64,
    pub ev_swing: f64,
    pub ev_take: f64,
}

/// Half-height of the band in which the bat still meets the ball (barrel radius + ball
/// radius, slightly less to allow for glancing contact), m [D/E].
pub const CONTACT_TOL_Z: f64 = 0.06;
/// Timing window for usable contact, s [E].
pub const CONTACT_TOL_T: f64 = 0.025;
/// Horizontal window along the bat in which contact is still on the barrel, m [E]
/// (about ±5 in around the sweet spot).
pub const CONTACT_TOL_X: f64 = 0.12;

/// The batter's belief about its own execution noise: vertical (m) and timing (s). Agents
/// know their own attributes; they never see the executed noise sample.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OwnExecution {
    pub vertical: f64,
    pub timing: f64,
    /// How much aim error the batter expects to correct between commit and the cutoff:
    /// vertical (m) and timing (s). From `Adjustability` and the time left; discounted
    /// because late information is itself noisy (agent-models §3.6).
    pub correct_z: f64,
    pub correct_t: f64,
}

/// Share of the in-swing correction capacity a batter counts on when deciding [E].
pub const CORRECTION_CREDIT: f64 = 0.5;

/// Fraction of the zone's half-sizes the batter treats as the "heart" for contact value [E].
pub const HEART: f64 = 0.7;

/// P(|e| < tol) for e ~ N(mu, sigma²).
fn p_within(mu: f64, sigma: f64, tol: f64) -> f64 {
    phi((tol - mu) / sigma) - phi((-tol - mu) / sigma)
}

#[allow(clippy::too_many_arguments)]
pub fn decide(post: &[Component], aim: Component, count: Count, zone: (f64, f64), disc: &Discipline, own: OwnExecution, threshold: f64, ball_r: f64) -> Decision {
    let (bottom, top) = zone;
    let mid_z = 0.5 * (bottom + top);
    let half_h = 0.5 * (top - bottom);
    let mut ps = 0.0;
    let mut ev_swing = 0.0;
    let v0 = values::count_value(count);
    for c in post {
        let sx = m::sqrt(c.cov.r[0][0].max(1e-8));
        let sz = m::sqrt(c.cov.r[1][1].max(1e-8));
        let st = m::sqrt(c.cov.r[2][2].max(1e-10));
        let p = p_strike(c.mean.x, c.mean.y, c.cov.r[0][0], c.cov.r[0][1], c.cov.r[1][1], bottom, top, ball_r);
        ps += c.w * p;
        // Contact chance if this hypothesis is true while aiming at the dominant mode: the
        // ball must pass within the contact band, in time, and on the barrel.
        let sv = m::hypot(sz, own.vertical);
        let stt = m::hypot(st, own.timing);
        let pc = p_within(c.mean.y - aim.mean.y, sv, CONTACT_TOL_Z + own.correct_z)
            * p_within(c.mean.z - aim.mean.z, stt, CONTACT_TOL_T + own.correct_t)
            * p_within(c.mean.x - aim.mean.x, sx, CONTACT_TOL_X + own.correct_z);
        let out_x = (c.mean.x.abs() - HEART * HALF_WIDTH).max(0.0);
        let out_z = ((c.mean.y - mid_z).abs() - HEART * half_h).max(0.0);
        let contact = values::contact_value(count, m::hypot(out_x, out_z)) - v0;
        let whiff = values::after_strike(count) - v0;
        ev_swing += c.w * (pc * contact + (1.0 - pc) * whiff);
    }
    let pt = disc.distort(ps);
    let ev_take = pt * (values::after_strike(count) - v0) + (1.0 - pt) * (values::after_ball(count) - v0);
    Decision { swing: ev_swing - ev_take > threshold, aim, p_strike: ps, p_strike_perceived: pt, ev_swing, ev_take }
}

/// Batting-count helper used by priors: the usage group of a count.
pub fn count_group(c: Count) -> usize {
    if c.strikes == 2 {
        3
    } else if c.balls > c.strikes {
        2
    } else if c.strikes > c.balls {
        0
    } else {
        1
    }
}

/// In-game memory of one opposing pitcher (agent-models §3.8). Per pitch type: exposure
/// count and the batter's believed shape offsets (spin axis °, spin rate fraction,
/// efficiency, speed m/s) relative to what it has been told by scouting. Only the
/// orchestrator, which owns the truth, can say what the offsets are relative to; the
/// batter's decisions see only the resulting [`Scouting`] models.
#[derive(Clone, Debug, PartialEq)]
pub struct PitcherMemory {
    pub n: Vec<f64>,
    pub offsets: Vec<[f64; 4]>,
}

impl PitcherMemory {
    pub fn new(offsets: Vec<[f64; 4]>) -> Self {
        Self { n: vec![0.0; offsets.len()], offsets }
    }

    /// Learn from one pitch of type index `k` seen with weight `weight` (1 at the plate,
    /// 0.3 from the on-deck circle). `cue_noise` is the perceptual noise of this sighting in
    /// the same units as the offsets; the belief moves toward (truth + noise) at rate
    /// η·weight/(1 + n_k).
    pub fn learn(&mut self, k: usize, weight: f64, eta: f64, cue_noise: [f64; 4]) {
        let lr = (eta * weight / (1.0 + self.n[k])).min(1.0);
        for (o, c) in self.offsets[k].iter_mut().zip(cue_noise) {
            *o += lr * (c - *o);
        }
        self.n[k] += weight;
    }

    /// Within-type spread belief after exposure: σ_∞ + (σ_0 − σ_∞)·exp(−η·n), σ_∞ = σ_0/2.
    pub fn break_sigma(&self, k: usize, sigma0: f64, eta: f64) -> f64 {
        let s_inf = 0.5 * sigma0;
        s_inf + (sigma0 - s_inf) * m::exp(-eta * self.n[k])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitcher::{PitchDef, PitchIntent, PitcherAttrs, plan_release};
    use bb_physics::flight::{DT_PITCH, FlightEnv, fly};
    use bb_physics::pitch::launch;

    /// The analytic observation Jacobian agrees with central finite differences of h itself,
    /// early, mid, and late in flight, for a state away from the prior mean.
    #[test]
    fn observation_jacobian_matches_finite_differences() {
        let env = FlightEnv::default_env();
        let p = PitcherAttrs::average_rhp();
        let def = PitchDef::template(PitchType::Slider);
        let rel = plan_release(&env, &p, &def, &PitchIntent { ty: PitchType::Slider, target_x: 0.1, target_z: 0.7 }).unwrap();
        let nominal = fly(&env, launch(rel.r, &rel.shape, rel.yaw, rel.elev), 0.0, DT_PITCH, &[Crossing::YBelow(-0.5)], 2.0);
        let sc = Scouting { types: vec![type_model(PitchType::Slider, nominal, [0.0; 3])], usage: vec![1.0] };
        let tm = &sc.types[0];
        let s: SVec = [tm.x0 + 0.05, tm.z0 - 0.08, tm.t0 * 1.02, 0.03, -0.02, rel.r.x + 0.04, rel.r.z - 0.03, 0.002, -0.001, 0.03];
        let hyp = Hypothesis { ty: PitchType::Slider, log_prior: 0.0, log_lik: 0.0, log_spin: 0.0, s, p: [[0.0; NS]; NS], model: 0 };
        let eye = eye_point(Hand::Right, 1.88);
        for frac in [0.15, 0.5, 0.85] {
            let t = frac * tm.t0;
            let (_, j) = Bank::h_jac(&sc, &hyp, &s, 0.0, eye, t);
            for k in 0..NS {
                let e = 1e-6;
                let (mut sp, mut sm) = (s, s);
                sp[k] += e;
                sm[k] -= e;
                let hp = Bank::h_jac(&sc, &hyp, &sp, 0.0, eye, t).0.to_array();
                let hm = Bank::h_jac(&sc, &hyp, &sm, 0.0, eye, t).0.to_array();
                for i in 0..3 {
                    let fd = (hp[i] - hm[i]) / (2.0 * e);
                    assert!((j[i][k] - fd).abs() <= 1e-6 * (1.0 + fd.abs()), "t {t:.3} ∂h{i}/∂s{k}: analytic {} vs fd {fd}", j[i][k]);
                }
            }
        }
    }
}
