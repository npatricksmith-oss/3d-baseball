//! Bat profile and mass properties (physics-spec §6.1–§6.2).
//!
//! The bat is a solid of revolution about its long axis. `u` is the distance from the knob
//! end (m). Mass properties are integrated from the profile and wood density so geometry and
//! inertia always agree. v1 uses a flat end (no end-cap rounding or cup).

use bb_math::m;
use bb_math::units::INCH;

/// One profile segment: diameter goes from `d0` to `d1` over [u0, u1] as
/// `d0 + (d1 − d0)·s^exponent`, s = (u − u0)/(u1 − u0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub u0: f64,
    pub u1: f64,
    pub d0: f64,
    pub d1: f64,
    pub exponent: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BatProfile {
    pub segments: Vec<Segment>,
    /// Wood density, kg/m³.
    pub density: f64,
}

/// Integrated mass properties. All about the transverse or long axis as noted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MassProps {
    /// Total length, m.
    pub length: f64,
    pub mass: f64,
    /// Center of mass, m from the knob.
    pub u_cm: f64,
    /// Transverse moment of inertia about the center of mass, kg·m².
    pub i_cm: f64,
    /// Moment of inertia about the long axis, kg·m².
    pub i_ax: f64,
}

impl MassProps {
    /// Transverse MOI about the point `u` from the knob (parallel-axis theorem).
    pub fn moi_about(&self, u: f64) -> f64 {
        self.i_cm + self.mass * (u - self.u_cm) * (u - self.u_cm)
    }
    /// Inverse effective mass for a normal impulse at distance `b` from the center of mass
    /// along the axis (physics-spec §6.3 / §8.1): 1/M + b²/I_cm.
    pub fn inv_eff_mass(&self, u: f64) -> f64 {
        let b = u - self.u_cm;
        1.0 / self.mass + b * b / self.i_cm
    }
}

impl BatProfile {
    /// Default 34-inch wood bat [E] (physics-spec §6.1), ρ_w = 660 kg/m³.
    pub fn default_34in() -> BatProfile {
        let i = INCH;
        BatProfile {
            segments: vec![
                Segment { u0: 0.0, u1: 0.5 * i, d0: 1.35 * i, d1: 1.35 * i, exponent: 1.0 },
                Segment { u0: 0.5 * i, u1: 1.0 * i, d0: 1.35 * i, d1: 0.93 * i, exponent: 1.0 },
                Segment { u0: 1.0 * i, u1: 11.0 * i, d0: 0.93 * i, d1: 0.93 * i, exponent: 1.0 },
                Segment { u0: 11.0 * i, u1: 22.0 * i, d0: 0.93 * i, d1: 2.50 * i, exponent: 1.6 },
                Segment { u0: 22.0 * i, u1: 34.0 * i, d0: 2.50 * i, d1: 2.50 * i, exponent: 1.0 },
            ],
            density: 660.0,
        }
    }

    pub fn length(&self) -> f64 {
        self.segments.last().map(|s| s.u1).unwrap_or(0.0)
    }

    fn segment_at(&self, u: f64) -> Option<&Segment> {
        let i = self.segments.partition_point(|s| s.u1 < u);
        self.segments.get(i)
    }

    /// Profile radius R_bat(u); zero outside [0, L].
    pub fn radius(&self, u: f64) -> f64 {
        if u < 0.0 || u > self.length() {
            return 0.0;
        }
        match self.segment_at(u) {
            Some(s) => {
                let sp = ((u - s.u0) / (s.u1 - s.u0)).clamp(0.0, 1.0);
                0.5 * (s.d0 + (s.d1 - s.d0) * m::pow(sp, s.exponent))
            }
            None => 0.0,
        }
    }

    /// Profile slope dR/du (one-sided inside a segment).
    pub fn slope(&self, u: f64) -> f64 {
        match self.segment_at(u.clamp(0.0, self.length())) {
            Some(s) => {
                let len = s.u1 - s.u0;
                let sp = ((u - s.u0) / len).clamp(0.0, 1.0);
                if s.d1 == s.d0 || (sp == 0.0 && s.exponent > 1.0) {
                    return 0.0;
                }
                0.5 * (s.d1 - s.d0) * s.exponent * m::pow(sp, s.exponent - 1.0) / len
            }
            None => 0.0,
        }
    }

    /// Integrate mass properties with composite Simpson's rule on each segment.
    pub fn mass_props(&self) -> MassProps {
        const N: usize = 2000; // even, per segment
        let rho = self.density;
        let pi = core::f64::consts::PI;
        // Moments: ∫ dm, ∫ u dm, ∫ u² dm, ∫ R² dm.
        let (mut m0, mut m1, mut m2, mut mr2) = (0.0, 0.0, 0.0, 0.0);
        for s in &self.segments {
            let h = (s.u1 - s.u0) / N as f64;
            for k in 0..=N {
                let u = s.u0 + h * k as f64;
                let sp = k as f64 / N as f64;
                let r = 0.5 * (s.d0 + (s.d1 - s.d0) * m::pow(sp, s.exponent));
                let w = if k == 0 || k == N { 1.0 } else if k % 2 == 1 { 4.0 } else { 2.0 };
                let dm = w * h / 3.0 * rho * pi * r * r;
                m0 += dm;
                m1 += u * dm;
                m2 += u * u * dm;
                mr2 += r * r * dm;
            }
        }
        let u_cm = m1 / m0;
        MassProps {
            length: self.length(),
            mass: m0,
            u_cm,
            i_cm: m2 - m0 * u_cm * u_cm + 0.25 * mr2,
            i_ax: 0.5 * mr2,
        }
    }
}

/// Sweet-spot station u_ss = L − 0.15 m [E] (physics-spec §8.2; Statcast's 6-in-from-tip point).
pub fn sweet_spot(length: f64) -> f64 {
    length - 0.15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_shape() {
        let b = BatProfile::default_34in();
        assert!((b.length() - 34.0 * INCH).abs() < 1e-12);
        assert!((b.radius(5.0 * INCH) - 0.465 * INCH).abs() < 1e-12);
        assert!((b.radius(30.0 * INCH) - 1.25 * INCH).abs() < 1e-12);
        assert!(b.slope(16.0 * INCH) > 0.0);
        assert_eq!(b.slope(30.0 * INCH), 0.0);
        assert_eq!(b.radius(35.0 * INCH), 0.0);
    }
}

/// Bat–ball contact parameters (physics-spec §8.2). All [E] except as noted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BatBallParams {
    /// Sweet spot, m back from the tip (u_ss = L − this).
    pub ss_from_tip: f64,
    /// Station where the barrel starts to taper toward the handle, m from the knob; f falls
    /// to `f_taper` there.
    pub u_taper: f64,
    pub f_tip: f64,
    pub f_taper: f64,
    pub f_handle: f64,
    /// Normal COR at the sweet spot at `v_ref` relative normal speed [S, derived]: set so the
    /// default bat's sweet-spot collision efficiency is the sourced q = 0.215 (physics-spec §8.2).
    pub e_ref: f64,
    pub v_ref: f64,
    /// COR decrease per m/s of extra normal speed (sign [S], size [E]).
    pub s_e: f64,
    /// Tangential restitution at and below `v_t_lab` normal speed (physics-spec §8.2.1) [S]:
    /// KNS 2017's ⟨e_x⟩ for baseballs, from a ball at rest on a tee at 77 mph.
    pub e_t_lab: f64,
    /// Normal approach speed, m/s, up to which `e_t_lab` holds [E], anchored on KNS's 77 mph.
    pub v_t_lab: f64,
    /// Tangential restitution at and above `v_t_game` [S, derived]: Statcast's batted backspin
    /// and sidespin (Nathan 2020 Figs. 2, 4) read through this model.
    pub e_t_game: f64,
    /// Normal approach speed, m/s, from which `e_t_game` holds [E], anchored on the game
    /// contact that makes a 25–30° fly ball. Between the two anchors e_t is linear in |c_n|:
    /// an assumption with no source.
    pub v_t_game: f64,
    /// Ball–bat friction [S].
    pub mu: f64,
}

impl Default for BatBallParams {
    fn default() -> Self {
        Self {
            ss_from_tip: 0.15,
            u_taper: 22.0 * INCH,
            f_tip: 0.75,
            f_taper: 0.55,
            f_handle: 0.35,
            // A8 (physics-spec §8.2): the sourced sweet-spot collision efficiency q = 0.215
            // (range 0.21–0.23; docs/calibration/2026-11-15-collision-efficiency-sources.md)
            // at u_ss for the default bat (r = m/M_eff = 0.2015) needs e = q(1 + r) + r ≈ 0.460.
            // Owner, 2026-11-15 (Q6; was 0.442, q = 0.20).
            e_ref: 0.460,
            v_ref: 70.0,
            s_e: 0.001,
            e_t_lab: 0.405,
            v_t_lab: 34.0,
            e_t_game: 0.0,
            v_t_game: 65.0,
            mu: 0.50,
        }
    }
}

impl BatBallParams {
    /// Location factor f(u) (physics-spec §8.2): 1 at the sweet spot, quadratic falloff to
    /// `f_tip` at the tip and to `f_taper` where the barrel starts to taper, never below
    /// `f_handle`. (The spec's "taper start" is read as the barrel end of the taper.)
    pub fn location_factor(&self, u: f64, length: f64) -> f64 {
        let uss = length - self.ss_from_tip;
        let f = if u >= uss {
            let s = (u - uss) / self.ss_from_tip;
            1.0 - (1.0 - self.f_tip) * s * s
        } else {
            let s = (uss - u) / (uss - self.u_taper);
            1.0 - (1.0 - self.f_taper) * s * s
        };
        f.max(self.f_handle)
    }

    /// Tangential restitution e_t(|c_n|) (physics-spec §8.2.1): `e_t_lab` up to `v_t_lab`,
    /// linear in the normal approach speed to `e_t_game` at `v_t_game`, constant beyond.
    pub fn e_t(&self, cn_abs: f64) -> f64 {
        if cn_abs <= self.v_t_lab {
            self.e_t_lab
        } else if cn_abs >= self.v_t_game {
            self.e_t_game
        } else {
            self.e_t_lab + (self.e_t_game - self.e_t_lab) * (cn_abs - self.v_t_lab) / (self.v_t_game - self.v_t_lab)
        }
    }

    /// The same parameters with one e_t at every speed (counterfactuals and diagnostics).
    pub fn with_constant_e_t(self, e_t: f64) -> Self {
        Self { e_t_lab: e_t, e_t_game: e_t, ..self }
    }

    /// Normal COR e_n(u, c_n) = e_peak(c_n)·f(u).
    pub fn e_n(&self, u: f64, length: f64, cn_abs: f64) -> f64 {
        let e_peak = self.e_ref - self.s_e * (cn_abs - self.v_ref);
        (e_peak * self.location_factor(u, length)).clamp(0.05, 0.9)
    }
}
