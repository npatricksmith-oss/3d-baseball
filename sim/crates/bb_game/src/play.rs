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
    /// How hard he commits to his read before he has seen the ball off the bat (1: as the
    /// odds justify; 0: he waits, no jump), [E].
    pub jump_gain: f64,
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
            arm: if outfield { 39.0 } else { 36.0 },
            arm_sigma: 1.2f64.to_radians(),
            transfer: if outfield { 0.85 } else { 0.65 },
            tau_read: 0.6,
            read_sigma0: if outfield { 3.0 } else { 0.3 },
            antic_sigma: ANTIC_SIGMA_50,
            swing_sigma: bat_read_sigma(BAT_READ_TIME_SIGMA_50_S),
            jump_gain: 1.0,
        }
    }

    /// The same fielder with `Anticipation` at `grade` (20–80 scale, agent-models §10): both
    /// read σs by the ρ rule (less is better).
    pub fn with_anticipation(self, grade: f64) -> Self {
        Self {
            antic_sigma: bb_agents::attributes::ratio_from_grade(grade, ANTIC_SIGMA_50, ANTIC_RHO),
            swing_sigma: bat_read_sigma(bb_agents::attributes::ratio_from_grade(grade, BAT_READ_TIME_SIGMA_50_S, ANTIC_RHO)),
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
/// How fast the bat's horizontal angle turns at contact, deg/s [E]: the mean over fair balls in
/// play (sd 143; `examples/bat_read_geometry.rs`, 48 games, 2026-11-13). A property of the
/// swing model: refit when it changes.
pub const BAT_YAW_RATE_DPS: f64 = 1915.0;
/// Ratio per 10 grades for both read σs (less is better), [E].
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

/// Standard positions, normal alignment: polar (distance m, angle rad from +Y, + = 1B side),
/// indexed by position number − 1 (P, C, 1B, 2B, 3B, SS, LF, CF, RF).
pub fn home_positions() -> [Vec3; 9] {
    let p = |d_ft: f64, deg: f64| {
        let d = d_ft * 0.3048;
        let a = deg.to_radians();
        Vec3::new(d * m::sin(a), d * m::cos(a), 0.0)
    };
    [p(60.0, 0.0), Vec3::new(0.0, -0.9, 0.0), p(105.0, 38.0), p(150.0, 13.0), p(110.0, -36.0), p(150.0, -13.0), p(290.0, -30.0), p(320.0, 0.0), p(290.0, 30.0)]
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
    let home = home_positions();

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
        facts.push((path.t_end(), Input::BallTouched { t: path.t_end(), what: Touch::OutOfPlay { x: p.x, y: p.y, in_flight } }));
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
        let cf = FielderAttrs::average(7);
        assert_eq!(FielderAttrs::average(6), cf);
        assert_eq!(FielderAttrs::average(8), cf);
        for i in 0..6 {
            assert_ne!(FielderAttrs::average(i).read_sigma0, cf.read_sigma0, "position index {i}");
        }
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
    }
}
