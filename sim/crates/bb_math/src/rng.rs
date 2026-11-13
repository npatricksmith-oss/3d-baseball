//! Keyed counter-based RNG (PLAN §4.2, data-formats §2.3).
//!
//! Generator: **Philox4x32-10** (Salmon et al., "Parallel Random Numbers: As Easy as 1, 2, 3",
//! SC'11), checked against the Random123 known-answer vectors. Its 64-bit key and 128-bit
//! counter hold the data-formats §2.3 key layout exactly:
//!
//! ```text
//! key     = game_seed (u64)
//! counter = [play_index, agent_id << 16 | purpose, sub_index, block]
//! ```
//!
//! `block` counts 128-bit output blocks within one stream, so a stream yields 2⁶⁴ bits
//! before repeating. Because every draw is a pure function of its key, adding a draw in
//! one place never shifts any other stream, and no RNG state is ever saved.

use crate::m;

/// Draw purposes. **Append-only registry:** never renumber or reuse a value (data-formats §2.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum Purpose {
    PitchRelease = 1,
    PitchSelection = 2,
    BatterPerception = 3,
    SwingExecution = 4,
    FielderPerception = 5,
    GlovePlacement = 6,
    ThrowExecution = 7,
    RunnerPerception = 8,
    SurfaceBounce = 9,
    AgentPolicy = 10,
    /// A fielder's read of the pitch location before contact (agent-models §5.10).
    FielderAnticipation = 11,
}

/// The full key of one random stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RngKey {
    pub game_seed: u64,
    pub play_index: u32,
    pub agent_id: u16,
    pub purpose: Purpose,
    pub sub_index: u32,
}

const M0: u32 = 0xD251_1F53;
const M1: u32 = 0xCD9E_8D57;
const W0: u32 = 0x9E37_79B9;
const W1: u32 = 0xBB67_AE85;

#[inline]
fn mulhilo(a: u32, b: u32) -> (u32, u32) {
    let p = (a as u64) * (b as u64);
    ((p >> 32) as u32, p as u32)
}

/// One Philox4x32-10 block.
pub fn philox4x32_10(ctr: [u32; 4], key: [u32; 2]) -> [u32; 4] {
    let mut c = ctr;
    let mut k = key;
    for round in 0..10 {
        if round > 0 {
            k[0] = k[0].wrapping_add(W0);
            k[1] = k[1].wrapping_add(W1);
        }
        let (hi0, lo0) = mulhilo(M0, c[0]);
        let (hi1, lo1) = mulhilo(M1, c[2]);
        c = [hi1 ^ c[1] ^ k[0], lo1, hi0 ^ c[3] ^ k[1], lo0];
    }
    c
}

/// Map a 64-bit word to the open interval (0, 1): (k + ½)·2⁻⁵² with k the top 52 bits.
/// Every result is exactly representable, so the endpoints 0 and 1 can never occur.
#[inline]
pub fn word_to_open_unit(w: u64) -> f64 {
    ((w >> 12) as f64 + 0.5) * (1.0 / (1u64 << 52) as f64)
}

/// A stream of draws for one [`RngKey`].
#[derive(Clone, Debug)]
pub struct KeyedRng {
    key: [u32; 2],
    ctr: [u32; 3],
    block: u32,
    buf: [u32; 4],
    used: usize,
    spare_normal: Option<f64>,
}

impl KeyedRng {
    pub fn new(k: RngKey) -> Self {
        Self {
            key: [k.game_seed as u32, (k.game_seed >> 32) as u32],
            ctr: [k.play_index, ((k.agent_id as u32) << 16) | k.purpose as u32, k.sub_index],
            block: 0,
            buf: [0; 4],
            used: 4,
            spare_normal: None,
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        if self.used == 4 {
            self.buf = philox4x32_10([self.ctr[0], self.ctr[1], self.ctr[2], self.block], self.key);
            self.block = self.block.wrapping_add(1);
            self.used = 0;
        }
        let v = self.buf[self.used];
        self.used += 1;
        v
    }

    pub fn next_u64(&mut self) -> u64 {
        let lo = self.next_u32() as u64;
        let hi = self.next_u32() as u64;
        (hi << 32) | lo
    }

    /// Uniform in the open interval (0, 1), 52-bit resolution.
    pub fn uniform(&mut self) -> f64 {
        word_to_open_unit(self.next_u64())
    }

    /// Standard normal via Box–Muller (deterministic through `libm`).
    pub fn normal(&mut self) -> f64 {
        if let Some(z) = self.spare_normal.take() {
            return z;
        }
        let u1 = self.uniform();
        let u2 = self.uniform();
        let r = m::sqrt(-2.0 * m::ln(u1));
        let th = 2.0 * core::f64::consts::PI * u2;
        self.spare_normal = Some(r * m::sin(th));
        r * m::cos(th)
    }

    /// Standard normal truncated at ±`k` (agent-models §1.3 uses k = 4), by rejection.
    pub fn truncated_normal(&mut self, k: f64) -> f64 {
        loop {
            let z = self.normal();
            if z.abs() <= k {
                return z;
            }
        }
    }

    /// Normal with mean `mu` and standard deviation `sigma`.
    pub fn gauss(&mut self, mu: f64, sigma: f64) -> f64 {
        mu + sigma * self.normal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Random123 known-answer tests (kat_vectors, philox4x32_10).
    #[test]
    fn philox_known_answers() {
        assert_eq!(
            philox4x32_10([0, 0, 0, 0], [0, 0]),
            [0x6627_e8d5, 0xe169_c58d, 0xbc57_ac4c, 0x9b00_dbd8]
        );
        assert_eq!(
            philox4x32_10([u32::MAX; 4], [u32::MAX; 2]),
            [0x408f_276d, 0x41c8_3b0e, 0xa20b_c7c6, 0x6d54_51fd]
        );
        assert_eq!(
            philox4x32_10(
                [0x243f_6a88, 0x85a3_08d3, 0x1319_8a2e, 0x0370_7344],
                [0xa409_3822, 0x299f_31d0]
            ),
            [0xd16c_fe09, 0x94fd_cceb, 0x5001_e420, 0x2412_6ea1]
        );
    }

    fn key(sub: u32) -> RngKey {
        RngKey { game_seed: 42, play_index: 7, agent_id: 0x0103, purpose: Purpose::PitchRelease, sub_index: sub }
    }

    #[test]
    fn streams_are_reproducible_and_independent() {
        let a: Vec<u64> = { let mut r = KeyedRng::new(key(0)); (0..8).map(|_| r.next_u64()).collect() };
        let b: Vec<u64> = { let mut r = KeyedRng::new(key(0)); (0..8).map(|_| r.next_u64()).collect() };
        let c: Vec<u64> = { let mut r = KeyedRng::new(key(1)); (0..8).map(|_| r.next_u64()).collect() };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn normal_moments() {
        let mut r = KeyedRng::new(key(3));
        let n = 200_000;
        let (mut s, mut s2) = (0.0, 0.0);
        for _ in 0..n {
            let z = r.normal();
            s += z;
            s2 += z * z;
        }
        let mean = s / n as f64;
        let var = s2 / n as f64 - mean * mean;
        assert!(mean.abs() < 0.01, "mean {mean}");
        assert!((var - 1.0).abs() < 0.01, "var {var}");
    }

    #[test]
    fn uniform_endpoints_are_excluded() {
        let lo = word_to_open_unit(0);
        let hi = word_to_open_unit(u64::MAX);
        assert!(lo > 0.0 && hi < 1.0, "{lo} {hi}");
        assert_eq!(hi, 1.0 - f64::EPSILON / 2.0); // the largest f64 below 1
    }

    #[test]
    fn uniform_in_open_interval() {
        let mut r = KeyedRng::new(key(4));
        for _ in 0..10_000 {
            let u = r.uniform();
            assert!(u > 0.0 && u < 1.0);
        }
    }
}
