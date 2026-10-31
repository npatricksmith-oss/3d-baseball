//! Collision impulse solver (physics-spec §8.1): instantaneous rigid-body impulse with
//! normal and tangential restitution and a Coulomb friction limit. Used for bat–ball contact
//! and, with an immovable second body, for ground, wall, base, and pole contacts.

use bb_math::{Mat3, Vec3};

/// One body's state at the contact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    /// 1/mass; 0 for an immovable body.
    pub inv_mass: f64,
    /// World-frame inverse inertia; zero for an immovable body.
    pub inv_inertia: Mat3,
    pub v: Vec3,
    pub w: Vec3,
    /// Contact point relative to the body's center of mass.
    pub r: Vec3,
}

impl Body {
    pub fn immovable(r: Vec3) -> Body {
        Body { inv_mass: 0.0, inv_inertia: Mat3::ZERO, v: Vec3::ZERO, w: Vec3::ZERO, r }
    }
    /// Velocity of the contact point.
    pub fn point_velocity(&self) -> Vec3 {
        self.v + self.w.cross(self.r)
    }
    fn kinetic_energy(&self) -> f64 {
        let mut e = 0.0;
        if self.inv_mass > 0.0 {
            e += 0.5 * self.v.norm_sq() / self.inv_mass;
        }
        if let Some(i) = self.inv_inertia.inverse() {
            e += 0.5 * self.w.dot(i * self.w);
        }
        e
    }
}

/// World inverse inertia of an axisymmetric body (a bat) with long axis `a`.
pub fn axisymmetric_inv_inertia(i_transverse: f64, i_axial: f64, a: Vec3) -> Mat3 {
    let aa = Mat3::outer(a, a);
    (Mat3::IDENTITY - aa) * (1.0 / i_transverse) + aa * (1.0 / i_axial)
}

/// Inverse inertia of a sphere (the ball).
pub fn sphere_inv_inertia(i: f64) -> Mat3 {
    Mat3::scale(1.0 / i)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    /// Normal coefficient of restitution.
    pub e_n: f64,
    /// Tangential restitution.
    pub e_t: f64,
    /// Coulomb friction coefficient.
    pub mu: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Branch {
    Stick,
    Slide,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Impact {
    /// Impulse on body 1 (body 2 receives −J).
    pub j: Vec3,
    pub b1: Body,
    pub b2: Body,
    pub branch: Branch,
    /// The first solution gained energy and was redone with e_t = 0 (§8.1 step 5).
    pub energy_guard: bool,
    /// Even the e_t = 0 solution gained energy, so a frictionless normal impulse was used
    /// (always dissipative for e_n ≤ 1).
    pub frictionless_fallback: bool,
}

/// Why the solver declined to produce an impulse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NoImpact {
    /// The contact points are separating (c_n ≥ 0).
    Separating,
    /// K was singular (both bodies immovable).
    Singular,
}

/// The K matrix: Δc = K·J for impulse J on body 1 and −J on body 2.
pub fn k_matrix(b1: &Body, b2: &Body) -> Mat3 {
    let s1 = Mat3::skew(b1.r);
    let s2 = Mat3::skew(b2.r);
    Mat3::scale(b1.inv_mass + b2.inv_mass) - s1 * b1.inv_inertia * s1 - s2 * b2.inv_inertia * s2
}

/// Resolve one contact with normal `n` pointing from body 2 into body 1.
///
/// Energy guard (physics-spec §8.1 step 5): if the solution gains kinetic energy, retry with
/// e_t = 0; if that still gains energy (possible through normal–tangential coupling in K),
/// fall back to a frictionless normal impulse, which cannot gain energy for e_n ≤ 1.
pub fn resolve(b1: Body, b2: Body, n: Vec3, mat: Material) -> Result<Impact, NoImpact> {
    let e0 = b1.kinetic_energy() + b2.kinetic_energy();
    let gains = |o: &Impact| o.b1.kinetic_energy() + o.b2.kinetic_energy() > e0 + 1e-12 * e0.max(1e-30);
    let first = solve(&b1, &b2, n, mat)?;
    if !gains(&first) {
        return Ok(first);
    }
    if mat.e_t != 0.0 {
        let mut retry = solve(&b1, &b2, n, Material { e_t: 0.0, ..mat })?;
        retry.energy_guard = true;
        if !gains(&retry) {
            return Ok(retry);
        }
    }
    let mut fb = frictionless(&b1, &b2, n, mat.e_n)?;
    fb.energy_guard = true;
    fb.frictionless_fallback = true;
    Ok(fb)
}

fn frictionless(b1: &Body, b2: &Body, n: Vec3, e_n: f64) -> Result<Impact, NoImpact> {
    let cn = (b1.point_velocity() - b2.point_velocity()).dot(n);
    if cn >= 0.0 {
        return Err(NoImpact::Separating);
    }
    let k = k_matrix(b1, b2);
    let nkn = n.dot(k * n);
    if !(nkn > 0.0) {
        return Err(NoImpact::Singular);
    }
    let j = n * (-(1.0 + e_n) * cn / nkn);
    Ok(Impact { j, b1: apply(b1, j), b2: apply(b2, -j), branch: Branch::Slide, energy_guard: false, frictionless_fallback: false })
}

fn solve(b1: &Body, b2: &Body, n: Vec3, mat: Material) -> Result<Impact, NoImpact> {
    let c = b1.point_velocity() - b2.point_velocity();
    let cn = c.dot(n);
    if cn >= 0.0 {
        return Err(NoImpact::Separating);
    }
    let ct = c - n * cn;
    let k = k_matrix(b1, b2);
    let kinv = k.inverse().ok_or(NoImpact::Singular)?;
    // 1. Sticking solution.
    let c_target = n * (-mat.e_n * cn) - ct * mat.e_t;
    let j_stick = kinv * (c_target - c);
    let jn = j_stick.dot(n);
    let jt = (j_stick - n * jn).norm();
    let (j, branch) = if jt <= mat.mu * jn {
        (j_stick, Branch::Stick)
    } else {
        // 3. Sliding along the initial slip direction.
        let ctn = ct.norm();
        let t = if ctn > 0.0 { ct / ctn } else { Vec3::ZERO };
        let dir = n - t * mat.mu;
        let jmag = -(1.0 + mat.e_n) * cn / n.dot(k * dir);
        (dir * jmag, Branch::Slide)
    };
    Ok(Impact { j, b1: apply(b1, j), b2: apply(b2, -j), branch, energy_guard: false, frictionless_fallback: false })
}

fn apply(b: &Body, j: Vec3) -> Body {
    Body { v: b.v + j * b.inv_mass, w: b.w + b.inv_inertia * b.r.cross(j), ..*b }
}

/// Collision efficiency q = (e − r)/(1 + r), r = m_ball / M_eff (physics-spec §6.3, §8.1).
pub fn collision_efficiency(e: f64, m_ball: f64, inv_m_eff: f64) -> f64 {
    let r = m_ball * inv_m_eff;
    (e - r) / (1.0 + r)
}
