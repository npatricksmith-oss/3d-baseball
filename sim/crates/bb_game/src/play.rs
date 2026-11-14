//! Live-ball plays (M4 walking skeleton): the world's batted-ball path plus simple fielders,
//! throws, and runners, reported to the rules engine as time-stamped facts (rules-spec §2).
//!
//! Every outcome is geometric or a race between physical times; randomness enters only as
//! noise on perception, glove placement, and throws (keyed RNG), and as bounce roughness.
//!
//! **Skeleton simplifications (documented for M5):** a fielder's belief of the ball's path
//! is the true path plus a smooth error that shrinks with viewing time (standing in for the
//! batted-ball atlas and launch-estimate model of agent-models §5.1); one primary fielder
//! chases (earliest believed intercept), others cover bases; throws travel in straight lines
//! at a fixed fraction of arm strength; runners run straight segments with the §5.3
//! acceleration model; no dives, relays, slides, or collisions.

use bb_math::{KeyedRng, Purpose, RngKey, Vec3, m};
use bb_physics::ground::{BallPath, Field, SegmentEnd, batted_path};
use bb_physics::{BallState, FlightEnv};
use bb_rules::{AgentId, Input, Touch};

/// Base positions (corner of the base nearest the field's interior is close enough for the
/// skeleton): 1B, 2B, 3B, home.
pub fn base_pos(b: u8) -> Vec3 {
    let d = 27.432 / core::f64::consts::SQRT_2;
    match b {
        1 => Vec3::new(d, d, 0.0),
        2 => Vec3::new(0.0, 2.0 * d, 0.0),
        3 => Vec3::new(-d, d, 0.0),
        _ => Vec3::ZERO,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FielderAttrs {
    pub speed: f64,
    pub tau_a: f64,
    /// Lateral acceleration when changing direction, m/s² (`Agility`, agent-models §10).
    pub a_lat: f64,
    pub reaction: f64,
    pub hands: f64,
    pub arm: f64,
    /// Throw direction σ, rad.
    pub arm_sigma: f64,
    pub transfer: f64,
    pub tau_read: f64,
    /// Lateral read error at the start, m; depth error is `DEPTH_STRETCH` times larger.
    pub read_sigma0: f64,
    /// `Anticipation` (agent-models §5.10): σ of his read of where the pitch crosses the
    /// plate (horizontal), m.
    pub antic_sigma: f64,
    /// `Anticipation`: σ of his read of the bat's horizontal angle at the instant of contact,
    /// rad of bat angle: his timing error for the contact instant times how fast the bat is
    /// turning then ([`bat_read_sigma`]).
    pub swing_sigma: f64,
    /// `Anticipation`: σ of his timing of the bat's launch against the ball in flight (the
    /// swing's lead, agent-models §5.10), s.
    pub lead_sigma: f64,
    /// How hard he commits to his read before he has seen the ball off the bat (1: as the
    /// odds justify; 0: he waits, no jump), [E].
    pub jump_gain: f64,
    /// The hand he throws with; his glove is on the other (not graded).
    pub throws: bb_observe::Hand,
    /// Configurable A4 option/observation thresholds [E] (agent-models §5.5–5.7).
    pub throw_choice: bb_agents::throwing::ChoiceParams,
    /// From securing the ball to the earliest tag he can put on a runner, s [E] (agent-models
    /// §6.4): 0.25 s, the one quoted receive-and-tag time (a catcher, MLB.com via the tag-up
    /// sources note §3; low confidence), for every fielder, the same catch-and-sweep. Before,
    /// a tag landed on the tick the ball was secured.
    pub tag_apply_s: f64,
}

/// Grade-50 `ArmStrength` by position, mph: the league means of Statcast's arm-strength
/// leaderboard (Baseball Savant, 2026 as displayed; docs/calibration/2026-11-14-sources.md),
/// which is each player's mean over his max-effort throws (the top 10% of his throws), at the
/// release point. Indexed P0 C1 1B2 2B3 3B4 SS5 LF6 CF7 RF8. `None`: the leaderboard has no
/// pitcher, and catchers are on a separate one, so those two keep [`ARM_UNSOURCED_MPS`]. The
/// sim's value is a full-effort throw (agent-models §5.5), so the conversion is only the unit
/// change (agent-models §10, "Arm strength by position").
pub const ARM_MEAN_MPH: [Option<f64>; 9] = [None, None, Some(79.3), Some(79.1), Some(84.9), Some(86.1), Some(87.4), Some(89.6), Some(90.7)];
/// `ArmStrength` of a pitcher or catcher at grade 50, m/s [E]: the infield value used before
/// the positional means, with no measured replacement.
pub const ARM_UNSOURCED_MPS: f64 = 36.0;

/// The grade-50 `ArmStrength` of position `pos` (zero-based), m/s.
pub fn arm_strength_50_mps(pos: usize) -> f64 {
    ARM_MEAN_MPH[pos].map_or(ARM_UNSOURCED_MPS, bb_math::units::mph_to_mps)
}

impl FielderAttrs {
    pub fn average(pos: usize) -> Self {
        // `pos` is the zero-based index used everywhere (P0 C1 1B2 2B3 3B4 SS5 LF6 CF7 RF8).
        let outfield = pos >= 6;
        Self {
            // Outfielders chase a fly while tracking it, a little under sprint speed [E].
            speed: if outfield { 8.2 } else { 7.9 },
            // Infielders start from an athletic ready position: a quicker first step [E].
            tau_a: if outfield { 1.0 } else { 0.7 },
            a_lat: 7.0,
            // An outfielder needs longer to read a ball off the bat before his first step [E].
            reaction: if outfield { 0.35 } else { 0.25 },
            hands: 0.018,
            arm: arm_strength_50_mps(pos),
            arm_sigma: 1.2f64.to_radians(),
            transfer: if outfield { 0.85 } else { 0.65 },
            tau_read: 0.6,
            read_sigma0: if outfield { 3.5 } else { 0.3 },
            antic_sigma: ANTIC_SIGMA_50,
            swing_sigma: bat_read_sigma(BAT_READ_TIME_SIGMA_50_S),
            lead_sigma: LEAD_READ_TIME_SIGMA_50_S,
            jump_gain: 1.0,
            throws: bb_observe::Hand::Right,
            throw_choice: bb_agents::throwing::ChoiceParams::default(),
            tag_apply_s: 0.25,
        }
    }

    /// The same fielder with his throwing ratings at these grades (20–80 scale, agent-models
    /// §10): `ArmStrength` moves his top throw speed about his position's average, `ArmAccuracy`
    /// sets his direction σ, and `Transfer` scales his position's catch-to-release.
    pub fn with_throwing(self, strength: f64, accuracy: f64, transfer: f64) -> Self {
        use bb_agents::throwing::{ARM_STRENGTH_50_MPS, arm_accuracy, arm_strength, transfer_scale};
        Self {
            arm: self.arm + arm_strength(strength) - ARM_STRENGTH_50_MPS,
            arm_sigma: arm_accuracy(accuracy),
            transfer: self.transfer * transfer_scale(transfer),
            ..self
        }
    }

    /// The same fielder with `Anticipation` at `grade` (20–80 scale, agent-models §10): all
    /// three read σs by the ρ rule (less is better).
    pub fn with_anticipation(self, grade: f64) -> Self {
        Self {
            antic_sigma: bb_agents::attributes::ratio_from_grade(grade, ANTIC_SIGMA_50, ANTIC_RHO),
            swing_sigma: bat_read_sigma(bb_agents::attributes::ratio_from_grade(grade, BAT_READ_TIME_SIGMA_50_S, ANTIC_RHO)),
            lead_sigma: bb_agents::attributes::ratio_from_grade(grade, LEAD_READ_TIME_SIGMA_50_S, ANTIC_RHO),
            ..self
        }
    }
}

/// `Anticipation` at grade 50: pitch-location read σ, m [E].
pub const ANTIC_SIGMA_50: f64 = 0.15;
/// `Anticipation` at grade 50: σ of his timing of the contact instant against the bat's sweep,
/// s [E]. Judging where a moving object was at the moment of a cue on its own path (Linares,
/// Holcombe & White 2009, Exp. 8: 40 ms); with [`ANTIC_RHO`] it gives 20.5 ms at grade 80 and
/// 78 ms at grade 20, both published values too (agent-models §5.10, the derivation).
pub const BAT_READ_TIME_SIGMA_50_S: f64 = 0.040;
/// `Anticipation` at grade 50: σ of his timing of the bat's launch against the ball in flight,
/// s [E]. Judging where a moving object (the ball) was at the moment of a cue away from it (the
/// bat starting forward): Linares, Holcombe & White (2009), Exp. 1, 75 ms; with [`ANTIC_RHO`]
/// 38 ms at grade 80 and 146 ms at grade 20 (agent-models §5.10, the swing's lead).
pub const LEAD_READ_TIME_SIGMA_50_S: f64 = 0.075;
/// How fast the bat's horizontal angle turns at contact, deg/s [E]: the mean over fair balls in
/// play (sd 143; `examples/bat_read_geometry.rs`, 48 games, 2026-11-13). A property of the
/// swing model: refit when it changes.
pub const BAT_YAW_RATE_DPS: f64 = 1915.0;
/// Ratio per 10 grades for every Anticipation read σ (less is better), [E].
pub const ANTIC_RHO: f64 = 1.25;

/// The σ of a fielder's read of the bat's angle at contact (rad) from his timing σ for the
/// contact instant (s): the bat turns `BAT_YAW_RATE_DPS` meanwhile (agent-models §5.10). Distance
/// and viewing angle add at most a few percent at the standard positions, and are left out.
pub fn bat_read_sigma(time_sigma_s: f64) -> f64 {
    (BAT_YAW_RATE_DPS * time_sigma_s).to_radians()
}

const DEPTH_STRETCH: f64 = 3.0;
/// Distance at which `read_sigma0` applies, m [E].
const READ_REF_DIST: f64 = 30.0;
/// Re-planning interval, s (agent-models §5.2 asks 20–50 Hz; the skeleton uses 5 Hz).
const REPLAN_S: f64 = 0.2;
const REACH: f64 = 1.0;
/// Extra reach from a dive or full lunge, m [E] (agent-models §5.3), and its costs.
const DIVE: f64 = 1.5;
const DIVE_MAX_Z: f64 = 1.2;
const DIVE_GETUP_S: f64 = 1.5;
const CATCH_MAX_Z: f64 = 2.4;
const POCKET: f64 = 0.06;
const RIM: f64 = 0.12;
const DT: f64 = 1.0 / 30.0;
/// The batter-runner's first step after contact (follow-through) [E].
const BATTER_START: f64 = 0.30;
/// Rounding a base costs this much time [E].
const ROUND_S: f64 = 0.15;

/// Pre-pitch infield alignment selected from base-out state (agent-models §5.9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InfieldAlignment {
    /// Standard alignment (no runner on first, or two outs; Savant `firstBase=0`).
    Standard,
    /// Double-play depth with a runner on first and fewer than two out (Savant `firstBase=1`).
    DoublePlay,
}

/// League-average MLB infield starting positions `(depth_ft, angle_deg)` for `(1B, 2B, 3B, SS)`,
/// where `0°` = CF (`+Y`), `-45°` = 3B line, `+45°` = 1B line.
///
/// Source: Baseball Savant Fielder Positioning (`season=2024`, `shift=0` = `Shaded: No` standard
/// non-shifted alignment under the 2023+ two-infielders-per-side rule, PA-weighted across all
/// fielders with `attempts=1`), <https://baseballsavant.mlb.com/visuals/fielder-positioning>.
/// - RHB Standard (`batSide=R&firstBase=0&shift=0`, n=53,775)
/// - RHB DP       (`batSide=R&firstBase=1&shift=0`, n=17,792)
/// - LHB Standard (`batSide=L&firstBase=0&shift=0`, n=19,287)
/// - LHB DP       (`batSide=L&firstBase=1&shift=0`, n=6,172)
///
/// Note: Savant `firstBase=1` pools 0/1/2 outs (applied here when `runner_on_first && outs < 2`).
/// `Infield In` has no separate published Savant filter and is omitted rather than guessed.
pub const SAVANT_INFIELD_RHB_STANDARD: [(f64, f64); 4] = [
    (114.1, 29.6),  // 1B
    (152.9, 6.8),   // 2B
    (122.0, -36.4), // 3B
    (148.3, -18.3), // SS
];
pub const SAVANT_INFIELD_RHB_DP: [(f64, f64); 4] = [
    (88.0, 40.5),   // 1B holding runner near 1B bag
    (148.7, 6.7),   // 2B
    (118.8, -36.5), // 3B
    (144.8, -17.3), // SS
];
pub const SAVANT_INFIELD_LHB_STANDARD: [(f64, f64); 4] = [
    (123.9, 37.6),  // 1B
    (148.2, 19.1),  // 2B
    (116.1, -27.1), // 3B
    (152.6, -6.2),  // SS
];
pub const SAVANT_INFIELD_LHB_DP: [(f64, f64); 4] = [
    (88.9, 41.0),   // 1B holding runner near 1B bag
    (143.1, 19.0),  // 2B
    (113.8, -27.2), // 3B
    (148.8, -5.7),  // SS
];

fn polar_m(d_ft: f64, deg: f64) -> Vec3 {
    let d = d_ft * 0.3048;
    let a = deg.to_radians();
    Vec3::new(d * m::sin(a), d * m::cos(a), 0.0)
}

/// Pre-pitch positions indexed by position number − 1 (P, C, 1B, 2B, 3B, SS, LF, CF, RF)
/// for batter plate side `bats` and base-out state `(runner_on_first, outs)` (agent-models §5.9).
pub fn home_positions_for(bats: bb_observe::Hand, runner_on_first: bool, outs: u8) -> [Vec3; 9] {
    let align = if runner_on_first && outs < 2 {
        InfieldAlignment::DoublePlay
    } else {
        InfieldAlignment::Standard
    };
    let inf = match (bats, align) {
        (bb_observe::Hand::Right, InfieldAlignment::Standard) => SAVANT_INFIELD_RHB_STANDARD,
        (bb_observe::Hand::Right, InfieldAlignment::DoublePlay) => SAVANT_INFIELD_RHB_DP,
        (bb_observe::Hand::Left, InfieldAlignment::Standard) => SAVANT_INFIELD_LHB_STANDARD,
        (bb_observe::Hand::Left, InfieldAlignment::DoublePlay) => SAVANT_INFIELD_LHB_DP,
    };
    [
        polar_m(60.0, 0.0),          // P
        Vec3::new(0.0, -0.9, 0.0),   // C
        polar_m(inf[0].0, inf[0].1), // 1B
        polar_m(inf[1].0, inf[1].1), // 2B
        polar_m(inf[2].0, inf[2].1), // 3B
        polar_m(inf[3].0, inf[3].1), // SS
        polar_m(290.0, -30.0),       // LF
        polar_m(320.0, 0.0),         // CF
        polar_m(290.0, 30.0),        // RF
    ]
}

/// Standard pre-pitch positions for a neutral right-handed batter with bases empty.
pub fn home_positions() -> [Vec3; 9] {
    home_positions_for(bb_observe::Hand::Right, false, 0)
}

/// Distance covered in time `t` from a standing start (the curve [`run_time`] inverts).
pub fn run_dist(t: f64, v_max: f64, tau: f64) -> f64 {
    if t <= 0.0 { 0.0 } else { v_max * (t - tau * (1.0 - m::exp(-t / tau))) }
}

/// Time to cover distance `d` from rest with v(t) = v_max(1 − e^(−t/τ)) (agent-models §5.3).
/// Newton's method on the closed-form distance, from the near and far asymptotes.
pub fn run_time(d: f64, v_max: f64, tau: f64) -> f64 {
    if d <= 0.0 {
        return 0.0;
    }
    let dist = |t: f64| v_max * (t - tau * (1.0 - m::exp(-t / tau)));
    // Early: x ≈ v t²/(2τ); late: x ≈ v (t − τ).
    let mut t = (m::sqrt(2.0 * d * tau / v_max)).max(d / v_max + tau).min(m::sqrt(2.0 * d * tau / v_max).max(d / v_max));
    for _ in 0..8 {
        let f = dist(t) - d;
        let fp = v_max * (1.0 - m::exp(-t / tau));
        if fp <= 1e-12 {
            break;
        }
        let nt = (t - f / fp).max(1e-9);
        if (nt - t).abs() < 1e-9 {
            t = nt;
            break;
        }
        t = nt;
    }
    t
}

#[cfg(test)]
mod run_time_tests {
    use super::*;

    fn bisect(d: f64, v: f64, tau: f64) -> f64 {
        let dist = |t: f64| v * (t - tau * (1.0 - m::exp(-t / tau)));
        let (mut lo, mut hi) = (0.0, d / v + 3.0 * tau);
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if dist(mid) < d { lo = mid } else { hi = mid }
        }
        hi
    }

    #[test]
    fn newton_matches_bisection() {
        for &d in &[0.01, 0.3, 1.0, 3.0, 10.0, 27.432, 60.0, 120.0] {
            for &(v, tau) in &[(8.2, 1.0), (7.9, 0.7), (8.4, 1.0), (6.0, 1.3)] {
                let (a, b) = (run_time(d, v, tau), bisect(d, v, tau));
                assert!((a - b).abs() < 1e-6, "d {d} v {v} tau {tau}: {a} vs {b}");
            }
        }
        // agent-models §5.3 check: 27.3 m in ~4.3 s at 8.23 m/s, τ 1.0.
        assert!((run_time(27.3, 8.23, 1.0) - 4.3).abs() < 0.05);
    }
}

/// One runner in the play: id, base at the pitch (0 = batter), speed.
#[derive(Clone, Copy, Debug)]
pub struct RunnerIn {
    pub id: AgentId,
    pub base: u8,
    pub speed: f64,
}

pub struct PlayInput<'a> {
    pub env: &'a FlightEnv,
    pub field: &'a Field,
    pub ball: BallState,
    pub t_contact: f64,
    pub runners: Vec<RunnerIn>,
    pub outs: u8,
    /// Defensive agent ids by position number − 1, and their attributes.
    pub fielders: [AgentId; 9],
    pub fattrs: [FielderAttrs; 9],
    pub seed: u64,
    pub play: u32,
}

pub struct PlayOutput {
    pub facts: Vec<Input>,
    pub path: BallPath,
}

fn rng(seed: u64, play: u32, agent: AgentId, purpose: Purpose, sub: u32) -> KeyedRng {
    KeyedRng::new(RngKey { game_seed: seed, play_index: play, agent_id: agent, purpose, sub_index: sub })
}

/// Fielder's believed position of the ball at `t_ball` as judged at time `t_now`: truth plus
/// a fixed-direction error shrinking with viewing time, stretched along the line of sight.
fn believed(path: &BallPath, f_pos: Vec3, fa: &FielderAttrs, err: (f64, f64), t_c: f64, t_now: f64, t_ball: f64) -> Vec3 {
    let (p, _) = path.at(t_ball);
    let view = (t_now - t_c - fa.reaction).max(0.0);
    // Error grows with distance (a fixed angular error), shrinks with viewing time.
    let dist = m::hypot(p.x - f_pos.x, p.y - f_pos.y);
    let s = fa.read_sigma0 * (dist / READ_REF_DIST) / m::sqrt(1.0 + view / fa.tau_read);
    let los = Vec3::new(p.x - f_pos.x, p.y - f_pos.y, 0.0).normalized();
    let lat = Vec3::new(-los.y, los.x, 0.0);
    p + los * (err.0 * s * DEPTH_STRETCH) + lat * (err.1 * s)
}

/// Earliest believed intercept for a fielder: (ball time, point). Catchable means in reach
/// height on any part of the path.
fn intercept(path: &BallPath, f_pos: Vec3, fa: &FielderAttrs, err: (f64, f64), t_c: f64, t_now: f64, moving_since: f64) -> Option<(f64, Vec3)> {
    let t_end = path.t_end();
    let mut t = (t_now + 0.05).max(t_c + 0.1);
    let mut best: Option<(f64, Vec3, f64)> = None;
    while t <= t_end + 1e-9 {
        let b = believed(path, f_pos, fa, err, t_c, t_now, t);
        if b.z <= CATCH_MAX_Z {
            let reach = if b.z <= DIVE_MAX_Z { REACH + 0.6 * DIVE } else { REACH * 0.7 };
            let d = (Vec3::new(b.x, b.y, 0.0) - Vec3::new(f_pos.x, f_pos.y, 0.0)).norm() - reach;
            let start = moving_since.max(t_c + fa.reaction);
            let reach_t = start + run_time(d.max(0.0), fa.speed, fa.tau_a) - (t_now - start).max(0.0).min(0.0);
            if reach_t <= t {
                return Some((t, b));
            }
            let gap = reach_t - t;
            if best.map(|x| gap < x.2).unwrap_or(true) {
                best = Some((t, b, gap));
            }
        }
        t += 0.1;
    }
    // Nothing reachable in time: go to where the ball ends up (rest point).
    let (rest, _) = path.at(t_end);
    Some((t_end.max(t_now), Vec3::new(rest.x, rest.y, 0.0))).or(best.map(|b| (b.0, b.1)))
}

/// Simulate one live-ball play and return its rules facts in time order.
pub fn simulate_play(inp: &PlayInput) -> PlayOutput {
    let t_c = inp.t_contact;
    let path = batted_path(inp.env, inp.field, inp.ball, t_c, |k, surf| {
        let mut r = rng(inp.seed, inp.play, 0x0200, Purpose::SurfaceBounce, k);
        (surf.roughness * r.truncated_normal(4.0), surf.roughness * r.truncated_normal(4.0))
    });
    let mut facts: Vec<(f64, Input)> = Vec::new();
    let home = home_positions_for(bb_observe::Hand::Right, inp.runners.iter().any(|r| r.base == 1), inp.outs);

    // Ball facts from the path: ground touches, fence.
    for &(t, x, y) in &path.bounces {
        facts.push((t, Input::BallTouched { t, what: Touch::Ground { x, y } }));
    }
    for &(t, x, y) in &path.walls {
        facts.push((t, Input::BallTouched { t, what: Touch::Wall { x, y } }));
    }
    let end = path.end();
    if let SegmentEnd::OverFence { .. } = end {
        let (p, _) = path.at(path.t_end());
        let in_flight = path.bounces.is_empty();
        facts.push((path.t_end(), Input::BallTouched { t: path.t_end(), what: Touch::OutOfPlay { x: p.x, y: p.y, z: p.z, in_flight } }));
        return finish(facts, path);
    }

    if end == SegmentEnd::Rest {
        let (p, _) = path.at(path.t_end());
        facts.push((path.t_end(), Input::BallSettled { t: path.t_end(), x: p.x, y: p.y }));
    }

    // Perception errors per fielder (fixed per play; FielderPerception).
    let errs: Vec<(f64, f64)> = (0..9)
        .map(|i| {
            let mut r = rng(inp.seed, inp.play, inp.fielders[i], Purpose::FielderPerception, 0);
            (r.truncated_normal(4.0), r.truncated_normal(4.0))
        })
        .collect();

    // Primary fielder: earliest believed intercept from home position at reaction time.
    let mut best = (f64::INFINITY, 0usize, Vec3::ZERO);
    for i in 0..9 {
        let fa = &inp.fattrs[i];
        if let Some((t, p)) = intercept(&path, home[i], fa, errs[i], t_c, t_c + fa.reaction, t_c + fa.reaction) {
            if t < best.0 {
                best = (t, i, p);
            }
        }
    }
    let pi = best.1;
    let fa = inp.fattrs[pi];
    let fid = inp.fielders[pi];
    let mut others: Vec<Vec3> = home.to_vec();

    // Chase at 30 Hz, re-planning; attempt a capture whenever the ball is in reach.
    let mut pos = home[pi];
    let mut speed = 0.0;
    let mut t = t_c + fa.reaction;
    let mut secured: Option<(f64, Vec3)> = None;
    let mut attempts = [0u32; 9]; // capture attempts per fielder (keyed per agent)
    let mut blocked_until = 0.0;
    let t_limit = path.t_end() + 12.0;
    let mut target = best.2;
    let mut next_plan = t;
    let mut dove = false;
    let mut next_select = t_c + 1.0;
    let mut pi = pi;
    let mut fa = fa;
    let mut fid = fid;
    while t < t_limit {
        // Check reach at sub-steps so a fast ball cannot slip between ticks.
        let mut caught_at: Option<(f64, Vec3)> = None;
        for k in 0..5 {
            let ts = t + DT * k as f64 / 5.0;
            let (ball, bv) = path.at(ts);
            let horiz = m::hypot(ball.x - pos.x, ball.y - pos.y);
            // Dive only when the ball will not come within standing reach in the next 0.3 s.
            let diving = horiz > REACH && horiz <= REACH + DIVE && ball.z <= DIVE_MAX_Z && {
                let mut closest = f64::INFINITY;
                for j in 1..=6 {
                    let (b2, _) = path.at(ts + 0.05 * j as f64);
                    closest = closest.min(if b2.z <= CATCH_MAX_Z { m::hypot(b2.x - pos.x, b2.y - pos.y) } else { f64::INFINITY });
                }
                closest > REACH
            };
            if ts >= blocked_until && (horiz <= REACH && ball.z <= CATCH_MAX_Z || diving) {
                // Capture test (agent-models §5.4): pure geometry with glove error.
                let k = (pi as usize).min(8);
                let mut r = rng(inp.seed, inp.play, fid, Purpose::GlovePlacement, attempts[k]);
                attempts[k] += 1;
                let reach_frac = (horiz / (REACH + DIVE)).min(1.0);
                let sg = fa.hands * (1.0 + 0.5 * bv.norm() / 40.0 + 0.6 * reach_frac + if diving { 1.0 } else { 0.0 });
                if diving {
                    dove = true;
                }
                let d = sg * m::hypot(r.truncated_normal(4.0), r.truncated_normal(4.0));
                if d <= POCKET {
                    caught_at = Some((ts, Vec3::new(ball.x, ball.y, 0.0)));
                    break;
                } else if d <= RIM {
                    blocked_until = ts + 0.6; // bobble: re-field after a moment
                } else {
                    blocked_until = ts + 0.3;
                }
            }
        }
        if let Some(c) = caught_at {
            secured = Some(c);
            break;
        }
        let (ball, _) = path.at(t);
        let horiz = m::hypot(ball.x - pos.x, ball.y - pos.y);
        if std::env::var("BB_TRACE").is_ok() && t < t_c + 4.0 {
            eprintln!("t {t:.2} fielder {pi} pos ({:.1},{:.1}) target ({:.1},{:.1}) ball ({:.1},{:.1},{:.2}) horiz {horiz:.2} speed {speed:.1}", pos.x, pos.y, target.x, target.y, ball.x, ball.y, ball.z);
        }
        // One second in, with better reads, the defense re-decides who takes it: the fielder
        // with the earliest believed intercept from where everyone now is (calls, §5.7).
        if t >= next_select {
            next_select = t + 1.0;
            others[pi] = pos;
            // Teammates have been drifting toward the ball at a jog since reacting.
            for i in 0..9 {
                if i != pi {
                    let to = Vec3::new(ball.x - home[i].x, ball.y - home[i].y, 0.0);
                    let jog = 0.5 * inp.fattrs[i].speed * (t - t_c - inp.fattrs[i].reaction).max(0.0);
                    others[i] = home[i] + to.normalized() * jog.min((to.norm() - 2.0).max(0.0));
                }
            }
            let mut b2 = (f64::INFINITY, pi);
            for i in 0..9 {
                if let Some((ti, _)) = intercept(&path, others[i], &inp.fattrs[i], errs[i], t_c, t, t) {
                    if ti < b2.0 - 0.3 || (i == pi && ti <= b2.0 + 0.3) {
                        b2 = (ti, i);
                    }
                }
            }
            if b2.1 != pi {
                pi = b2.1;
                fa = inp.fattrs[pi];
                fid = inp.fielders[pi];
                pos = others[pi];
                speed = 0.5 * fa.speed;
                next_plan = t;
            }
        }
        // Re-plan at REPLAN_S, then move. Once the ball is at rest or close, go straight to it.
        if t >= next_plan {
            next_plan = t + REPLAN_S;
            target = if t >= path.t_end() || horiz < 3.0 {
                ball
            } else {
                match intercept(&path, pos, &fa, errs[pi], t_c, t, t) {
                    Some((_, p)) => p,
                    None => ball,
                }
            };
        }
        let to = Vec3::new(target.x - pos.x, target.y - pos.y, 0.0);
        let dist = to.norm();
        speed += (fa.speed - speed) * (DT / fa.tau_a);
        let step = (speed * DT).min(dist);
        if dist > 1e-6 {
            pos += to * (step / dist);
        }
        if dist < 0.3 {
            speed *= 0.5;
        }
        t += DT;
    }
    let Some((t_sec, sec_pos)) = secured else {
        // Never secured in time: a liveness failure. Report it; never invent possession
        // (Codex M4 review #7).
        facts.push((t_limit, Input::PlayTimeout { t: t_limit }));
        return finish(facts, path);
    };
    facts.push((t_sec, Input::GloveSecured { t: t_sec, fielder: fid, x: sec_pos.x, y: sec_pos.y }));
    // Once the ball is held, the free path's later ground and wall contacts did not happen.
    facts.retain(|(t, f)| !(matches!(f, Input::BallTouched { .. } | Input::BallSettled { .. }) && *t > t_sec));
    // A fly is caught only if nothing but a fielder touched it first (rules-spec §5.3).
    let first_touch = path.bounces.first().map(|b| b.0).into_iter().chain(path.walls.first().map(|w| w.0)).fold(f64::INFINITY, f64::min);
    let caught_fly = first_touch >= t_sec;

    let fa = FielderAttrs { transfer: fa.transfer + if dove { DIVE_GETUP_S } else { 0.0 }, ..fa };

    // ---- Runners. Forced runners run; on a likely catch, runners hold and tag up.
    let forced: Vec<bool> = {
        let occ = |b: u8| inp.runners.iter().any(|r| r.base == b);
        inp.runners.iter().map(|r| r.base == 0 || (1..r.base).all(occ)).collect()
    };
    // Runner perception noise: one keyed stream per runner and decision (base) index.
    let rnoise = |id: AgentId, decision: u8| rng(inp.seed, inp.play, id, Purpose::RunnerPerception, decision as u32).truncated_normal(4.0);
    // Ball arrival estimate at base b from the fielder holding it at time t_sec.
    let ball_at = |b: u8, from: Vec3, t0: f64, arm: f64, transfer: f64| -> f64 {
        let d = (base_pos(b) - from).norm();
        t0 + transfer + d / (0.92 * arm) + 0.05
    };
    let mut plans: Vec<(RunnerIn, Vec<(u8, f64)>)> = Vec::new(); // base touches (base, time)
    for (k, r) in inp.runners.iter().enumerate() {
        let mut touches = Vec::new();
        let start_t = if r.base == 0 { t_c + BATTER_START } else { t_c };
        if caught_fly && r.base > 0 {
            // Tag up: return/stay, leave at the catch, advance one base if believed safe.
            let next = r.base + 1;
            let leg = 27.432;
            let t_run = t_sec + run_time(leg, r.speed, 1.0);
            let t_ball = ball_at(next, sec_pos, t_sec, fa.arm, fa.transfer);
            let noise = 0.15 * rnoise(r.id, next);
            if t_ball - t_run + noise > 0.3 && inp.outs < 2 {
                touches.push((r.base, t_sec + 0.001));
                touches.push((next, t_run));
            }
            plans.push((*r, touches));
            continue;
        }
        if caught_fly {
            plans.push((*r, touches));
            continue;
        }
        let mut b = r.base;
        let mut t_at = start_t;
        let first_leg = if r.base == 0 { 27.432 } else { 27.432 - 3.0 }; // runners have a lead
        loop {
            let next = b + 1;
            let leg = if b == r.base { first_leg } else { 27.432 };
            let tr = if b == r.base { run_time(leg, r.speed, 1.0) } else { leg / r.speed + ROUND_S };
            let t_next = t_at + tr;
            let must = forced[k] && b == r.base;
            let t_ball = ball_at(next, sec_pos, t_sec.max(t_at), fa.arm, fa.transfer);
            let noise = 0.15 * rnoise(r.id, next);
            let go = must || (t_ball - t_next + noise > 0.35);
            if !go || next > 4 {
                break;
            }
            touches.push((next, t_next));
            b = next;
            t_at = t_next;
            if next == 4 {
                break;
            }
        }
        plans.push((*r, touches));
    }
    for (r, touches) in &plans {
        if !touches.is_empty() && r.base > 0 && !(caught_fly && touches.first().map(|x| x.0) == Some(r.base)) {
            facts.push((t_c + 0.01, Input::BaseLeft { t: t_c + 0.01, agent: r.id, base: r.base }));
        }
        for (i, &(b, tb)) in touches.iter().enumerate() {
            facts.push((tb, Input::BaseTouched { t: tb, agent: r.id, base: b, holding_ball: false }));
            if b < 4 && i + 1 < touches.len() {
                facts.push((tb + 0.02, Input::BaseLeft { t: tb + 0.02, agent: r.id, base: b }));
            }
        }
    }

    // ---- Throw decision (agent-models §5.6, simplified): best believed out among runners'
    // target bases; the lead runner is worth more.
    let mut fr = rng(inp.seed, inp.play, fid, Purpose::ThrowExecution, 0);
    let mut choice: Option<(u8, f64, AgentId, f64, bool)> = None; // base, value, runner, runner time, force
    for (k, (r, touches)) in plans.iter().enumerate() {
        let Some(&(b, tb)) = touches.iter().find(|x| x.0 > r.base || (r.base == 0 && x.0 >= 1)).or(touches.last()) else { continue };
        let target = touches.iter().rev().find(|x| x.0 > r.base).map(|x| (x.0, x.1)).unwrap_or((b, tb));
        let (tb_base, tb_time) = if forced[k] && !caught_fly { touches[0] } else { target };
        let t_ball = ball_at(tb_base, sec_pos, t_sec, fa.arm, fa.transfer);
        let margin = tb_time - t_ball;
        let p_out = 0.5 * (1.0 + m::erf(margin / (0.12 * core::f64::consts::SQRT_2)));
        let weight = [0.9, 1.0, 1.1, 1.3][(tb_base as usize).saturating_sub(1).min(3)];
        let v = p_out * weight;
        if p_out > 0.3 && choice.map(|c| v > c.1).unwrap_or(true) {
            choice = Some((tb_base, v, r.id, tb_time, forced[k] && !caught_fly));
        }
    }
    let mut t_settle = plans.iter().flat_map(|(_, t)| t.iter().map(|x| x.1)).fold(t_sec, f64::max) + 1.0;
    if let Some((b, _, runner, t_runner, force)) = choice {
        let d = (base_pos(b) - sec_pos).norm();
        let t_arr = ball_at(b, sec_pos, t_sec, fa.arm, fa.transfer);
        let lateral = d * fa.arm_sigma * fr.truncated_normal(4.0);
        // Receiver: the covering fielder (1B, SS/2B, 3B, C) by base.
        let recv_pos = match b {
            1 => 2,
            2 => if sec_pos.x < 0.0 { 3 } else { 5 },
            3 => 4,
            _ => 1,
        };
        let recv = inp.fielders[recv_pos];
        let t_cover = t_c + inp.fattrs[recv_pos].reaction + run_time((base_pos(b) - home[recv_pos]).norm(), inp.fattrs[recv_pos].speed, 1.0);
        let t_catch = t_arr.max(t_cover);
        if recv == fid || (base_pos(b) - sec_pos).norm() < 6.0 {
            // Closer to take it himself: run the ball to the base (or tag the runner there).
            let t_self = t_sec + 0.2 + run_time((base_pos(b) - sec_pos).norm(), fa.speed, fa.tau_a);
            if force {
                facts.push((t_self, Input::BaseTouched { t: t_self, agent: fid, base: b, holding_ball: true }));
            } else if t_self + 0.05 < t_runner {
                // The tag happens as the runner arrives, after the fielder is there with the ball.
                facts.push((t_runner - 0.03, Input::TagContact { t: t_runner - 0.03, fielder: fid, runner, holding_ball: true }));
            }
        } else if lateral.abs() <= 1.2 {
            if force {
                facts.push((t_catch, Input::BaseTouched { t: t_catch, agent: recv, base: b, holding_ball: true }));
            } else if t_catch + 0.05 < t_runner {
                // Tag the arriving runner just before the base, after the catch.
                facts.push((t_runner - 0.03, Input::TagContact { t: t_runner - 0.03, fielder: recv, runner, holding_ball: true }));
            }
            // Caught fly: a throw beating a runner back to his base is the leave-early appeal.
            if caught_fly {
                facts.push((t_catch, Input::BaseTouched { t: t_catch, agent: recv, base: b, holding_ball: true }));
            }
            // Batter-runner after a force at second: the relay to first (double play) if time.
            if force && b == 2 {
                let t2 = t_catch + 0.45 + (base_pos(1) - base_pos(2)).norm() / (0.92 * inp.fattrs[recv_pos].arm);
                let batter_first = plans.iter().find(|(r, _)| r.base == 0).and_then(|(_, t)| t.first().map(|x| x.1));
                // The first baseman must be at the bag to take the relay.
                let t_1b = t_c + inp.fattrs[2].reaction + run_time((base_pos(1) - home[2]).norm(), inp.fattrs[2].speed, 1.0);
                if let Some(tf) = batter_first {
                    if t2.max(t_1b) < tf && fid != inp.fielders[2] {
                        let t2 = t2.max(t_1b);
                        facts.push((t2, Input::BaseTouched { t: t2, agent: inp.fielders[2], base: 1, holding_ball: true }));
                    }
                }
            }
        }
        t_settle = t_settle.max(t_catch + 1.0);
    }
    facts.push((t_settle, Input::PlayEndCheck { t: t_settle }));
    finish(facts, path)
}

fn finish(mut facts: Vec<(f64, Input)>, path: BallPath) -> PlayOutput {
    facts.sort_by(|a, b| a.0.total_cmp(&b.0));
    PlayOutput { facts: facts.into_iter().map(|f| f.1).collect(), path }
}

#[cfg(test)]
mod attr_tests {
    use super::*;

    // Fielding review #1: positions are indexed from zero (P0 … LF6 CF7 RF8); all three
    // outfielders share the outfield values and no infielder has them.
    #[test]
    fn all_three_outfielders_get_outfield_values() {
        // (They differ in arm: each position has its own measured mean.)
        let cf = FielderAttrs::average(7);
        for i in [6, 8] {
            assert_eq!(FielderAttrs { arm: cf.arm, ..FielderAttrs::average(i) }, cf);
        }
        for i in 0..6 {
            assert_ne!(FielderAttrs::average(i).read_sigma0, cf.read_sigma0, "position index {i}");
        }
    }

    // agent-models §10, "Arm strength by position": the positional means of Statcast's
    // leaderboard (mph) × 0.44704 m/s per mph, worked by hand:
    //   1B 79.3 → 35.450   2B 79.1 → 35.361   3B 84.9 → 37.954   SS 86.1 → 38.490
    //   LF 87.4 → 39.071   CF 89.6 → 40.055   RF 90.7 → 40.547
    // (e.g. CF: 89 × 0.44704 = 39.787, 0.6 × 0.44704 = 0.268, total 40.055). The pitcher and the
    // catcher have no entry and keep 36 m/s. Pitcher is index 0, catcher 1.
    #[test]
    fn arm_strength_follows_the_published_positional_means() {
        let want = [36.0, 36.0, 35.450, 35.361, 37.954, 38.490, 39.071, 40.055, 40.547];
        for (i, w) in want.iter().enumerate() {
            let a = FielderAttrs::average(i).arm;
            assert!((a - w).abs() < 5e-4, "position {i}: {a:.4} m/s, want {w}");
        }
        // The order the leaderboard shows: first and second weakest, then third, short,
        // left, center, right strongest.
        let arm = |i| FielderAttrs::average(i).arm;
        assert!(arm(3) < arm(2) && arm(2) < arm(4) && arm(4) < arm(5) && arm(5) < arm(6) && arm(6) < arm(7) && arm(7) < arm(8));
        // A grade-50 `with_throwing` leaves the positional mean alone (grade 50 = 37.0 generic).
        assert!((FielderAttrs::average(8).with_throwing(50.0, 50.0, 50.0).arm - arm(8)).abs() < 1e-12);
    }

    // agent-models §5.10 and §10 (the bat-read derivation, 2026-11-13): the bat-angle read σ is
    // the bat's 1,915°/s at contact times his timing σ for the contact instant, 40 ms at grade
    // 50 and a factor 1.25³ = 1.953125 per 30 grades. By hand: grade 50, 1915 × 0.040 = 76.6°;
    // grade 80, 76.6 / 1.953125 = 39.2192°; grade 20, 76.6 × 1.953125 = 149.609375°. The
    // pitch-location read keeps its 0.15 m at grade 50.
    #[test]
    fn bat_read_sigma_follows_the_timing_derivation() {
        let a = FielderAttrs::average(5);
        assert!((a.swing_sigma.to_degrees() - 76.6).abs() < 1e-9, "{}", a.swing_sigma.to_degrees());
        assert!((a.with_anticipation(50.0).swing_sigma.to_degrees() - 76.6).abs() < 1e-9);
        let g80 = a.with_anticipation(80.0).swing_sigma.to_degrees();
        let g20 = a.with_anticipation(20.0).swing_sigma.to_degrees();
        assert!((g80 - 39.2192).abs() < 1e-4, "grade 80: {g80}");
        assert!((g20 - 149.609375).abs() < 1e-6, "grade 20: {g20}");
        assert_eq!(a.antic_sigma, 0.15);
        // The swing's lead (2026-11-13): 75 ms at grade 50 (Linares Exp. 1), 75 / 1.953125 =
        // 38.4 ms at grade 80, 75 × 1.953125 = 146.484375 ms at grade 20.
        assert!((a.lead_sigma - 0.075).abs() < 1e-12);
        assert!((a.with_anticipation(80.0).lead_sigma - 0.0384).abs() < 1e-9);
        assert!((a.with_anticipation(20.0).lead_sigma - 0.146484375).abs() < 1e-9);
    }

    // agent-models §5.9: Baseball Savant 2024 league-average infield positions (`shift=0`),
    // converted from (depth ft, angle deg) via d = depth_ft * 0.3048, x = d*sin(a), y = d*cos(a).
    //
    // Worked by hand (RHB Standard, firstBase=0):
    //   1B (114.1 ft, +29.6°): d = 34.7777 m, x = +17.1782 m, y = 30.2390 m (sin +0.493942, cos 0.869495)
    //   2B (152.9 ft,  +6.8°): d = 46.6039 m, x =  +5.5181 m, y = 46.2761 m (sin +0.118404, cos 0.992966)
    //   3B (122.0 ft, -36.4°): d = 37.1856 m, x = -22.0666 m, y = 29.9305 m (sin -0.593419, cos 0.804894)
    //   SS (148.3 ft, -18.3°): d = 45.2018 m, x = -14.1930 m, y = 42.9158 m (sin -0.313992, cos 0.949425)
    //
    // Worked by hand (LHB Standard, firstBase=0):
    //   1B (123.9 ft, +37.6°): d = 37.7647 m, x = +23.0420 m, y = 29.9206 m (sin +0.610145, cos 0.792290)
    //   2B (148.2 ft, +19.1°): d = 45.1714 m, x = +14.7809 m, y = 42.6846 m (sin +0.327218, cos 0.944949)
    //   3B (116.1 ft, -27.1°): d = 35.3873 m, x = -16.1205 m, y = 31.5022 m (sin -0.455545, cos 0.890213)
    //   SS (152.6 ft,  -6.2°): d = 46.5125 m, x =  -5.0233 m, y = 46.2404 m (sin -0.107999, cos 0.994151)
    #[test]
    fn savant_standard_positions_match_hand_conversions() {
        let rhb = home_positions_for(bb_observe::Hand::Right, false, 0);
        assert_eq!(rhb, home_positions());
        let want_rhb = [
            (2, 17.1782, 30.2390),
            (3, 5.5181, 46.2761),
            (4, -22.0666, 29.9305),
            (5, -14.1930, 42.9158),
        ];
        for (i, wx, wy) in want_rhb {
            assert!((rhb[i].x - wx).abs() < 1e-3 && (rhb[i].y - wy).abs() < 1e-3 && rhb[i].z == 0.0,
                "RHB pos {i}: ({:.4}, {:.4}), want ({wx:.4}, {wy:.4})", rhb[i].x, rhb[i].y);
        }

        let lhb = home_positions_for(bb_observe::Hand::Left, false, 0);
        let want_lhb = [
            (2, 23.0420, 29.9206),
            (3, 14.7809, 42.6846),
            (4, -16.1205, 31.5022),
            (5, -5.0233, 46.2404),
        ];
        for (i, wx, wy) in want_lhb {
            assert!((lhb[i].x - wx).abs() < 1e-3 && (lhb[i].y - wy).abs() < 1e-3 && lhb[i].z == 0.0,
                "LHB pos {i}: ({:.4}, {:.4}), want ({wx:.4}, {wy:.4})", lhb[i].x, lhb[i].y);
        }

        // Every infielder shades further toward LF (-X) vs RHB than vs LHB.
        for i in 2..=5 {
            assert!(rhb[i].x < lhb[i].x, "infielder {i} pull shade: rhb x {} vs lhb x {}", rhb[i].x, lhb[i].x);
        }
        // P, C, and outfield positions stay identical across batter side and base state.
        let lhb_dp = home_positions_for(bb_observe::Hand::Left, true, 1);
        for i in [0, 1, 6, 7, 8] {
            assert_eq!(rhb[i], lhb[i]);
            assert_eq!(rhb[i], lhb_dp[i]);
        }
    }

    // Worked by hand (Double-Play Depth with runner on 1B and < 2 out, Savant firstBase=1):
    //   RHB 1B ( 88.0 ft, +40.5°): d = 26.8224 m, x = +17.4198 m, y = 20.3959 m (2.215 m from 1B bag (19.3974, 19.3974))
    //   RHB 2B (148.7 ft,  +6.7°): d = 45.3238 m, x =  +5.2880 m, y = 45.0142 m
    //   RHB 3B (118.8 ft, -36.5°): d = 36.2102 m, x = -21.5387 m, y = 29.1078 m
    //   RHB SS (144.8 ft, -17.3°): d = 44.1350 m, x = -13.1247 m, y = 42.1384 m
    //   LHB 1B ( 88.9 ft, +41.0°): d = 27.0967 m, x = +17.7770 m, y = 20.4502 m (1.932 m from 1B bag)
    //   LHB 2B (143.1 ft, +19.0°): d = 43.6169 m, x = +14.2003 m, y = 41.2406 m
    //   LHB 3B (113.8 ft, -27.2°): d = 34.6862 m, x = -15.8550 m, y = 30.8505 m
    //   LHB SS (148.8 ft,  -5.7°): d = 45.3542 m, x =  -4.5046 m, y = 45.1300 m
    #[test]
    fn savant_dp_depth_holds_first_and_shortens_middle_only_before_two_outs() {
        let bag1 = base_pos(1);
        for (bats, want, std_want_d) in [
            (
                bb_observe::Hand::Right,
                [(2, 17.4198, 20.3959), (3, 5.2880, 45.0142), (4, -21.5387, 29.1078), (5, -13.1247, 42.1384)],
                SAVANT_INFIELD_RHB_STANDARD,
            ),
            (
                bb_observe::Hand::Left,
                [(2, 17.7770, 20.4502), (3, 14.2003, 41.2406), (4, -15.8550, 30.8505), (5, -4.5046, 45.1300)],
                SAVANT_INFIELD_LHB_STANDARD,
            ),
        ] {
            let std_p = home_positions_for(bats, false, 0);
            let dp_0 = home_positions_for(bats, true, 0);
            let dp_1 = home_positions_for(bats, true, 1);
            let two_out = home_positions_for(bats, true, 2);
            assert_eq!(dp_0, dp_1);
            assert_eq!(two_out, std_p, "two outs reverts to standard depth");
            for (i, wx, wy) in want {
                assert!((dp_0[i].x - wx).abs() < 1e-3 && (dp_0[i].y - wy).abs() < 1e-3,
                    "{bats:?} DP pos {i}: ({:.4}, {:.4}), want ({wx:.4}, {wy:.4})", dp_0[i].x, dp_0[i].y);
                assert!(dp_0[i].norm() < std_want_d[i - 2].0 * 0.3048, "infielder {i} plays shallower at DP depth");
            }
            // 1B holds near 1B bag (~1.9–2.2 m vs ~11 m standard).
            assert!((dp_0[2] - bag1).norm() < 2.3);
            assert!((std_p[2] - bag1).norm() > 10.5);
        }
    }
}
