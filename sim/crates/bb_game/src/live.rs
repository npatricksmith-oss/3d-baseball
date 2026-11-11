//! Live-ball plays, causal version (M4b): the world advances in fixed ticks; fielders and
//! runners decide only from what they have observed so far; the rules engine is stepped
//! with each fact as it happens, so players learn of outs the way they would on a field.
//!
//! Replaces the skeleton in `play.rs` (kept for the diagnostics examples), which planned from
//! the realized future (Codex M4 review #5, #6, #9, #10, #21, #24).
//!
//! Remaining simplifications, documented for M5: fielder perception is a noisy estimate of
//! the ball's current state fed into the agent's own physics (smooth ground, typical spin)
//! rather than the full atlas model of agent-models §5.1; throws fly without drag; no
//! relays, slides, rundown tactics, or collisions; coverage follows a small fixed table.

use bb_math::units::G;
use bb_math::{KeyedRng, Purpose, RngKey, Vec3, m};
use bb_physics::ground::{BallPath, Field, SegmentEnd, batted_path, batted_path_dt};
use bb_physics::{BallState, FlightEnv};
use bb_rules::{AgentId, Event, Input, PlayState, RulesState, Touch, step};

use crate::play::{FielderAttrs, RunnerIn, base_pos, home_positions, run_dist, run_time};

const DT: f64 = 1.0 / 60.0;
const REPLAN_S: f64 = 0.4;
const REACH: f64 = 1.0;
const DIVE: f64 = 1.5;
const DIVE_MAX_Z: f64 = 1.2;
const DIVE_GETUP_S: f64 = 1.5;
const CATCH_MAX_Z: f64 = 2.4;
/// Leaping reach at the wall [E].
const WALL_CATCH_MAX_Z: f64 = 3.0;
const POCKET: f64 = 0.06;
const RIM: f64 = 0.12;
const DEPTH_STRETCH: f64 = 3.0;
const READ_REF_DIST: f64 = 30.0;
/// League-typical fielder values a runner assumes (scouting-level knowledge) [E].
const TYPICAL_SPEED: f64 = 8.0;
/// League-typical runner top speed a fielder assumes, m/s [E].
const TYPICAL_RUNNER_SPEED: f64 = 8.2;
const TYPICAL_ARM: f64 = 37.0;
const TYPICAL_TRANSFER: f64 = 0.75;
/// A runner's read of a batted ball, m at 30 m [E].
const RUNNER_READ_SIGMA0: f64 = 2.0;
/// Agents' internal flight step, s.
const AGENT_DT: f64 = 0.1;
const BASE_L: f64 = 27.432;
const BATTER_START: f64 = 0.25;
/// Time constant of a runner's acceleration, s [E]. With the batter's start delay an
/// average runner (8.2 m/s) reaches first 4.45 s after contact (MLB averages about 4.4).
const RUNNER_TAU: f64 = 0.85;
/// On a ball in the air that may or may not be caught, with fewer than two out, a runner
/// goes this far past his lead and waits to see it land or be caught, m [E].
const HALFWAY_M: f64 = 5.0;
const PLAY_LIMIT_S: f64 = 40.0;
/// Fraction of arm strength used on throws [E].
const THROW_FRAC: f64 = 0.92;
const RECEIVE_REACH: f64 = 1.5;
/// Running distance over which the batter leaves his box and joins the base path, m [E].
const BOX_EXIT_M: f64 = 4.0;
/// How far behind a base its backup stands, on the line of the throw, m [E] (coaching
/// guides give 15–18 m for an outfielder; foul ground allows less).
const BACKUP_M: f64 = 13.0;
/// A runner's distance off his base as the ball is hit, m [E]: his lead plus the shuffle he
/// takes as the pitch is delivered. Index = base (none at home).
const LEAD_M: [f64; 4] = [0.0, 5.5, 6.5, 4.5];

/// Where a runner on `base` stands as the pitch is hit, as base-path distance from home.
pub(crate) fn lead_s(base: u8) -> f64 {
    base as f64 * BASE_L + LEAD_M[(base as usize).min(3)]
}
/// A fielder taking a throw already faces his next target: his catch-to-release is this
/// fraction of his transfer time on a batted ball [E] (about 0.45 s for an infielder; it was
/// a flat 0.25 s, which turned one double play in ten ground balls and a triple play every
/// eight games).
const RELAY_TRANSFER_FRAC: f64 = 0.7;
/// Infield radius for the settle rule (rules-spec §5.8), m. The spec's 30 m excluded second
/// base (38.8 m from home); the infield dirt reaches ~45 m.
const INFIELD_R: f64 = 45.0;

#[derive(Clone, Debug)]
enum Ball {
    /// Following physics (batted, deflected, or a throw that got away).
    Free { path: BallPath, next_touch: usize, next_wall: usize, settled_sent: bool },
    Held { by: usize },
    Thrown { r0: Vec3, v0: Vec3, t0: f64, to_base: u8, receiver: usize },
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Role {
    Chase,
    Cover(u8),
    Backup,
    Idle,
}

#[derive(Clone, Debug)]
struct Fielder {
    id: AgentId,
    a: FielderAttrs,
    pos: Vec3,
    speed: f64,
    role: Role,
    target: Vec3,
    next_plan: f64,
    /// Fixed per-play draws for the read error (depth, lateral, velocity depth, velocity lateral).
    err: [f64; 4],
    attempts: u32,
    blocked_until: f64,
    ready_at: f64,
    dove: bool,
    /// The fielder's own prediction of the ball, refreshed at replans.
    belief: Option<BallPath>,
    /// Believed intercept time.
    t_int: f64,
    /// Last close-range re-read of the ball.
    last_close_read: f64,
    /// Throws made this play (keys the throw-execution noise).
    throws: u32,
    /// The base whose throws he is backing up, if that is his job on this play.
    backs: Option<u8>,
}

#[derive(Clone, Debug)]
struct Runner {
    id: AgentId,
    start: u8,
    /// Distance along the base path from home, m (base k at k·BASE_L).
    s: f64,
    v: f64,
    vmax: f64,
    /// Base the runner is heading for (may be behind it on a retreat).
    goal: u8,
    out: bool,
    scored: bool,
    /// Holding at a base until a catch is made (tag-up).
    tag_up: bool,
    moving_since: f64,
    last_base: u8,
    /// Going back to the start base to retouch after a caught fly.
    retouch: bool,
    /// Holding at a base until the runner ahead clears the next one.
    waiting: bool,
    /// Waiting partway to the next base (base-path distance) to see whether a ball in the
    /// air is caught.
    hold_s: Option<f64>,
    /// The timing-error draw of the latest advance decision (standard normal; diagnostics).
    last_timing_draw: f64,
    /// The runner has seen the catch and settled whether he owes a retouch (decided once:
    /// off his base at the catch ⇒ owes it; on it ⇒ free to leave).
    catch_seen: bool,
    /// Advance decisions made this play. Decision n (from 1) draws its timing error from
    /// `RunnerPerception` sub-index 2n and its read of a free ball from 2n + 1; sub-index 1
    /// is the first read at contact (tag up or go).
    decisions: u32,
}

pub struct LiveInput<'a> {
    pub env: &'a FlightEnv,
    pub field: &'a Field,
    pub ball: BallState,
    pub t_contact: f64,
    pub runners: Vec<RunnerIn>,
    pub fielders: [AgentId; 9],
    pub fattrs: [FielderAttrs; 9],
    /// Where the batter stands in his box (ground point). He runs out of it: see `Sim::runner_pos`.
    pub batter_box: Vec3,
    pub seed: u64,
    pub play: u32,
    /// Record ball and actor tracks (timelines and visualization only).
    pub record: bool,
}

pub struct LiveOutput {
    pub facts: Vec<Input>,
    pub events: Vec<Event>,
    /// The batted ball's free path at contact (for timelines).
    pub path: BallPath,
    /// Realized ball positions (t, position) every tick.
    pub ball_track: Vec<(f64, Vec3)>,
    /// Every fielder's and runner's position every 0.1 s.
    pub actors: Vec<(AgentId, Vec<(f64, Vec3)>)>,
    /// Every time a fielder secured the ball (t, fielder), in time order.
    pub secured: Vec<(f64, AgentId)>,
    /// Every throw release (t, thrower), in time order.
    pub released: Vec<(f64, AgentId)>,
}

/// Profiling counters (BB_PROF=1): nanoseconds and calls for predict / intercept / ticks.
pub mod prof {
    use std::cell::Cell;
    thread_local! {
        pub static PRED: Cell<(u64, u64)> = const { Cell::new((0, 0)) };
        pub static INTER: Cell<(u64, u64)> = const { Cell::new((0, 0)) };
        pub static RUNNER: Cell<(u64, u64)> = const { Cell::new((0, 0)) };
        pub static LIVE: Cell<(u64, u64)> = const { Cell::new((0, 0)) };
        pub static TICKS: Cell<u64> = const { Cell::new(0) };
    }
    /// Profiling is on only when BB_PROF is set (checked once per thread).
    pub fn enabled() -> bool {
        thread_local! { static ON: bool = std::env::var("BB_PROF").is_ok(); }
        ON.with(|x| *x)
    }
    pub fn add(c: &'static std::thread::LocalKey<Cell<(u64, u64)>>, ns: u64) {
        c.with(|x| {
            let (a, b) = x.get();
            x.set((a + ns, b + 1));
        });
    }
}

/// Throw-decision tracing (BB_DEBUG_THROW), checked once per thread: `holder_act` runs
/// every tick the ball is held.
fn debug_throw() -> bool {
    thread_local! { static ON: bool = std::env::var("BB_DEBUG_THROW").is_ok(); }
    ON.with(|x| *x)
}

fn rng(seed: u64, play: u32, agent: AgentId, purpose: Purpose, sub: u32) -> KeyedRng {
    KeyedRng::new(RngKey { game_seed: seed, play_index: play, agent_id: agent, purpose, sub_index: sub })
}

fn fact_time(f: &Input) -> Option<f64> {
    match *f {
        Input::StartGame => None,
        Input::PitchDelivered { t }
        | Input::PitchCrossed { t, .. }
        | Input::SwingMeasured { t, .. }
        | Input::BatContact { t }
        | Input::BallTouched { t, .. }
        | Input::BallPassedBase { t, .. }
        | Input::BallSettled { t, .. }
        | Input::PlayTimeout { t }
        | Input::GloveSecured { t, .. }
        | Input::BaseTouched { t, .. }
        | Input::BaseLeft { t, .. }
        | Input::TagContact { t, .. }
        | Input::PlayEndCheck { t } => Some(t),
    }
}

fn phi(x: f64) -> f64 {
    0.5 * (1.0 + m::erf(x / core::f64::consts::SQRT_2))
}

/// The base a moving runner is visibly heading for: the next one in his direction of motion.
fn visible_target(r: &Runner) -> u8 {
    let q = r.s / BASE_L;
    if r.goal as f64 * BASE_L < r.s { q.ceil() as u8 - 1 } else { (q.floor() as u8 + 1).min(4) }
}

/// Position on the base path at distance `s` from home.
pub(crate) fn path_pos(s: f64) -> Vec3 {
    let s = s.clamp(0.0, 4.0 * BASE_L);
    let k = ((s / BASE_L).floor() as u8).min(3);
    let f = (s - k as f64 * BASE_L) / BASE_L;
    let a = base_pos(k);
    let b = base_pos(k + 1);
    a + (b - a) * f
}

/// Distance covered in time `t` by a fielder already moving at `v0`, with the speed law the
/// movement step uses (v → v_max with time constant τ; [`run_dist`] is the `v0` = 0 case).
/// Like the movement step, it takes a change of direction to cost nothing.
fn run_dist_from(t: f64, v0: f64, v_max: f64, tau: f64) -> f64 {
    if t <= 0.0 { 0.0 } else { v_max * t - (v_max - v0) * tau * (1.0 - m::exp(-t / tau)) }
}

/// Time to cover distance `d` when already moving at `v0`: the inverse of [`run_dist_from`]
/// (which is increasing in t), by bisection.
fn run_time_from(d: f64, v0: f64, v_max: f64, tau: f64) -> f64 {
    if d <= 0.0 {
        return 0.0;
    }
    // From rest the run takes at most d / v_max + τ; any starting speed only shortens it.
    let (mut lo, mut hi) = (0.0, d / v_max + tau);
    for _ in 0..50 {
        let mid = 0.5 * (lo + hi);
        if run_dist_from(mid, v0, v_max, tau) < d { lo = mid } else { hi = mid }
    }
    0.5 * (lo + hi)
}

/// A fielder's estimate of how long a runner needs to cover `d` metres: he sees the runner's
/// speed now and assumes a league-typical runner getting up to speed (not the runner's own
/// attributes, which he does not know).
fn runner_eta(d: f64, v_now: f64) -> f64 {
    run_time_from(d, v_now.min(TYPICAL_RUNNER_SPEED), TYPICAL_RUNNER_SPEED, RUNNER_TAU)
}

/// Where the backup of a throw from `from` to `base` stands: on the line of the throw,
/// [`BACKUP_M`] beyond the base.
fn backup_spot(from: Vec3, base: u8) -> Vec3 {
    let b = base_pos(base);
    let dir = Vec3::new(b.x - from.x, b.y - from.y, 0.0);
    if dir.norm() < 1e-6 { b } else { b + dir.normalized() * BACKUP_M }
}

/// A fielder's position and speed after the movement step of the tick at time `t`: toward
/// his target, accelerating to top speed and braking to stop on it. Not yet reacting, or on
/// the ground after a dive: unchanged.
fn fielder_step(fd: &Fielder, t: f64, t_contact: f64) -> (Vec3, f64) {
    if t < t_contact + fd.a.reaction || (fd.dove && t < fd.ready_at) {
        return (fd.pos, fd.speed);
    }
    let to = Vec3::new(fd.target.x - fd.pos.x, fd.target.y - fd.pos.y, 0.0);
    let d = to.norm();
    let brake = fd.speed * fd.speed / (2.0 * 6.0);
    // A chaser fields the ball on the run: his intercept plan (`run_dist`) has him arriving at
    // speed, so he slows for the spot only if he will be there before the ball.
    let early = fd.role != Role::Chase || !fd.t_int.is_finite() || t + d / fd.speed.max(0.5) + 0.1 < fd.t_int;
    let speed = if d > brake + 0.05 || !early { fd.speed + (fd.a.speed - fd.speed) * (DT / fd.a.tau_a) } else { (fd.speed - 6.0 * DT).max(0.0) };
    // Reaching the target ends the run: a fielder standing on his spot has no speed to carry
    // into the next plan.
    if speed * DT >= d {
        return (Vec3::new(fd.target.x, fd.target.y, fd.pos.z), 0.0);
    }
    (fd.pos + to * (speed * DT / d), speed)
}

struct Sim<'a> {
    inp: &'a LiveInput<'a>,
    /// Latest fact time emitted (facts never go backward, rules-spec §1.2).
    last_fact_t: f64,
    /// When the ball was last caught (a visible event).
    secured_at: f64,
    /// A fly was caught (the rules engine signalled `Catch`).
    catch_made: bool,
    st: &'a mut RulesState,
    t: f64,
    ball: Ball,
    f: Vec<Fielder>,
    r: Vec<Runner>,
    primary: usize,
    out: LiveOutput,
    bounce_k: u32,
    done: bool,
}

impl<'a> Sim<'a> {
    fn emit(&mut self, fact: Input) {
        if let Some(t) = fact_time(&fact) {
            self.last_fact_t = self.last_fact_t.max(t);
        }
        let evs = step(self.st, &fact);
        if evs.iter().any(|e| matches!(e, Event::RulesWarning { .. })) && std::env::var("BB_DEBUG_ORDER").is_ok() {
            eprintln!("warning after {fact:?} at sim t {:.3}: {:?}", self.t, evs);
            if let Some(last) = self.out.facts.last() {
                eprintln!("   previous fact {last:?}");
            }
            for r in &self.r {
                eprintln!("   runner {} start {} s {:.2} goal {} v {:.1} out {} scored {} tag_up {} last {}", r.id, r.start, r.s, r.goal, r.v, r.out, r.scored, r.tag_up, r.last_base);
            }
            for f in &self.out.facts {
                if matches!(f, Input::BaseTouched { .. } | Input::BaseLeft { .. } | Input::GloveSecured { .. }) {
                    eprintln!("     {f:?}");
                }
            }
        }
        for e in &evs {
            if let Event::RunnerOut { runner, .. } = e {
                if let Some(r) = self.r.iter_mut().find(|r| r.id == *runner) {
                    r.out = true;
                }
            }
            if matches!(e, Event::Catch { .. }) {
                self.catch_made = true;
            }
        }
        self.out.facts.push(fact);
        self.out.events.extend(evs);
        if !matches!(self.st.play, PlayState::Live(_)) {
            self.done = true;
        }
    }

    fn ball_at(&self, t: f64) -> (Vec3, Vec3) {
        match &self.ball {
            Ball::Free { path, .. } => path.at(t),
            Ball::Held { by } => (self.f[*by].pos + Vec3::new(0.0, 0.0, 1.2), Vec3::ZERO),
            Ball::Thrown { r0, v0, t0, .. } => {
                let dt = t - t0;
                (*r0 + *v0 * dt + Vec3::new(0.0, 0.0, -0.5 * G * dt * dt), *v0 + Vec3::new(0.0, 0.0, -G * dt))
            }
        }
    }

    /// The fielder's noisy estimate of the ball's current state, then its own prediction
    /// (smooth ground, typical spin for the trajectory) — never the true future.
    fn predict(&self, i: usize) -> BallPath {
        if !prof::enabled() {
            return self.predict_inner(i);
        }
        let t0 = std::time::Instant::now();
        let r = self.predict_inner(i);
        prof::add(&prof::PRED, t0.elapsed().as_nanos() as u64);
        r
    }

    fn predict_inner(&self, i: usize) -> BallPath {
        let fd = &self.f[i];
        let view = (self.t - self.inp.t_contact - fd.a.reaction).max(0.0);
        self.predict_from(fd.pos, fd.a.read_sigma0, fd.a.tau_read, view, fd.err)
    }

    /// An observer's prediction of the ball: a noisy estimate of its current state (error
    /// grows with distance, shrinks with viewing time, larger in depth), flown forward with
    /// the observer's own physics on smooth ground with typical spin.
    fn predict_from(&self, from: Vec3, sigma0: f64, tau_read: f64, view: f64, err: [f64; 4]) -> BallPath {
        let (p, v) = self.ball_at(self.t);
        let dist = m::hypot(p.x - from.x, p.y - from.y).max(1.0);
        let shrink = 1.0 / m::sqrt(1.0 + view / tau_read);
        let sp = sigma0 * (dist / READ_REF_DIST) * shrink;
        let los = Vec3::new(p.x - from.x, p.y - from.y, 0.0).normalized();
        let lat = Vec3::new(-los.y, los.x, 0.0);
        // Height is read with the lateral precision (an angular measurement).
        let pe = p + los * (err[0] * sp * DEPTH_STRETCH) + lat * (err[1] * sp) + Vec3::new(0.0, 0.0, err[1] * sp * 0.5);
        let sv = 0.06 * v.norm() * shrink;
        let ve = v + los * (err[2] * sv * DEPTH_STRETCH * 0.5) + lat * (err[3] * sv) + Vec3::new(0.0, 0.0, err[3] * sv * 0.5);
        // Typical spin: backspin on rising balls, topspin on balls hit down.
        let h = Vec3::new(ve.x, ve.y, 0.0).normalized();
        let axis = Vec3::new(h.y, -h.x, 0.0);
        let rpm = if ve.z > 2.0 { 1800.0 } else { -600.0 };
        let w = axis * (rpm * core::f64::consts::PI / 30.0);
        batted_path_dt(self.inp.env, self.inp.field, BallState { r: pe, v: ve, w }, self.t, AGENT_DT, |_, _| (0.0, 0.0))
    }

    /// A runner's read of the ball: a fresh look per decision, keyed by `sub` (see
    /// `Runner::decisions` for the sub-index layout).
    fn runner_predict(&self, k: usize, sub: u32) -> BallPath {
        let r = &self.r[k];
        let mut g = rng(self.inp.seed, self.inp.play, r.id, Purpose::RunnerPerception, sub);
        let err = [g.truncated_normal(4.0), g.truncated_normal(4.0), g.truncated_normal(4.0), g.truncated_normal(4.0)];
        let view = (self.t - self.inp.t_contact).max(0.0);
        self.predict_from(self.runner_pos(r), RUNNER_READ_SIGMA0, 0.6, view, err)
    }

    /// When the runner believes the defense can field the ball on `pred`: the earliest
    /// point some visible fielder could reach with league-typical speed.
    fn believed_field_time(&self, pred: &BallPath) -> (f64, Vec3) {
        let mut t = self.t + 0.05;
        let t_end = pred.t_end();
        while t <= t_end {
            let (b, _) = pred.at(t);
            if b.z <= CATCH_MAX_Z {
                let reachable = self.f.iter().any(|fd| {
                    let d = m::hypot(b.x - fd.pos.x, b.y - fd.pos.y) - REACH;
                    d <= 0.0 || run_dist(t - self.t, TYPICAL_SPEED, 0.9) >= d
                });
                if reachable {
                    return (t, Vec3::new(b.x, b.y, 0.0));
                }
            }
            t += 0.15;
        }
        let (rest, _) = pred.at(t_end);
        (t_end + 1.0, Vec3::new(rest.x, rest.y, 0.0))
    }

    /// Earliest point on a predicted path the fielder can reach in time: (time, point).
    fn intercept(&self, i: usize, pred: &BallPath) -> (f64, Vec3) {
        if !prof::enabled() {
            return self.intercept_inner(i, pred);
        }
        let t0 = std::time::Instant::now();
        let r = self.intercept_inner(i, pred);
        prof::add(&prof::INTER, t0.elapsed().as_nanos() as u64);
        r
    }

    fn intercept_inner(&self, i: usize, pred: &BallPath) -> (f64, Vec3) {
        // Plan to field it standing; count on a dive only if no standing play exists.
        let (standing, dive) = self.intercept_scan(i, pred);
        if let Some(x) = standing.or(dive) {
            return x;
        }
        let fd = &self.f[i];
        let t_end = pred.t_end();
        let start = self.t.max(self.inp.t_contact + fd.a.reaction);
        let (rest, _) = pred.at(t_end);
        let d = m::hypot(rest.x - fd.pos.x, rest.y - fd.pos.y);
        (t_end.max(start + run_time_from(d, fd.speed, fd.a.speed, fd.a.tau_a)), Vec3::new(rest.x, rest.y, 0.0))
    }

    /// One scan of the predicted path for the earliest point the fielder can reach standing,
    /// and the earliest he can reach with a dive (found on the way; used only without a
    /// standing play).
    #[allow(clippy::type_complexity)]
    fn intercept_scan(&self, i: usize, pred: &BallPath) -> (Option<(f64, Vec3)>, Option<(f64, Vec3)>) {
        let fd = &self.f[i];
        let t_end = pred.t_end();
        let start = self.t.max(self.inp.t_contact + fd.a.reaction);
        let mut dive = None;
        // Shortfall at time `t` for a ball at `b`: how far short of the ball's line the
        // fielder still is (≤ 0: he can field it standing, with the planning reach).
        let gap = |t: f64, b: Vec3| {
            let d = m::hypot(b.x - fd.pos.x, b.y - fd.pos.y);
            d - REACH * 0.7 - if t >= start { run_dist_from(t - start, fd.speed, fd.a.speed, fd.a.tau_a) } else { 0.0 }
        };
        // The dive's extra reach over the standing plan.
        let dive_extra = REACH + 0.6 * DIVE - REACH * 0.7;
        const STEP: f64 = 0.12;
        const SUB: u32 = 6;
        let mut t = self.t + 0.05;
        let mut prev: Option<(f64, Vec3, f64)> = None;
        while t <= t_end {
            let (b, _) = pred.at(t);
            let g = gap(t, b);
            // Between two samples a hard grounder travels several metres, more than a
            // fielder's reach. The shortfall cannot fall faster than the ball moves plus the
            // fielder runs, so an interval is searched at a finer step (on the predicted path
            // itself) only when the shortfall at its start is within that bound.
            let mut pts: Vec<(f64, Vec3, f64)> = Vec::new();
            if let Some((tp, bp, gp)) = prev {
                let travel = m::hypot(b.x - bp.x, b.y - bp.y) + fd.a.speed * (t - tp);
                if gp - dive_extra <= travel {
                    for k in 1..SUB {
                        let ts = tp + (t - tp) * k as f64 / SUB as f64;
                        let (bs, _) = pred.at(ts);
                        pts.push((ts, bs, gap(ts, bs)));
                    }
                }
            }
            pts.push((t, b, g));
            prev = Some((t, b, g));
            for (t, b, g) in pts {
                if b.z > CATCH_MAX_Z || t < start {
                    continue;
                }
                // Head for the ball's line itself, not the edge of reach.
                let here = Some((t, Vec3::new(b.x, b.y, 0.0)));
                if g <= 0.0 {
                    return (here, dive);
                }
                if dive.is_none() && b.z <= DIVE_MAX_Z && g - dive_extra <= 0.0 {
                    dive = here;
                }
            }
            t += STEP;
        }
        (None, dive)
    }

    /// Who chases: everyone predicts; the earliest believed intercept takes it (calls, §5.7;
    /// outfielders win ties against infielders on balls in between). Others cover or back up.
    fn assign(&mut self) {
        self.assign_among((0..9).collect(), None);
    }

    /// Re-call among the current chaser and the four fielders nearest the ball. Runs right
    /// after the chaser's replan in the same tick, so his fresh prediction is reused.
    fn recall(&mut self) {
        let (b, _) = self.ball_at(self.t);
        let mut idx: Vec<usize> = (0..9).collect();
        idx.sort_by(|&a, &c| {
            let da = (Vec3::new(b.x, b.y, 0.0) - self.f[a].pos).norm();
            let dc = (Vec3::new(b.x, b.y, 0.0) - self.f[c].pos).norm();
            da.total_cmp(&dc).then(a.cmp(&c))
        });
        let mut cands: Vec<usize> = idx.into_iter().take(4).collect();
        if !cands.contains(&self.primary) {
            cands.push(self.primary);
        }
        self.assign_among(cands, Some(self.primary));
    }

    /// `fresh`: a fielder whose belief, intercept, and target were computed this tick from
    /// the same observation (reused instead of predicted again).
    fn assign_among(&mut self, cands: Vec<usize>, fresh: Option<usize>) {
        let mut best = (f64::INFINITY, self.primary);
        for i in cands {
            let reuse = if fresh == Some(i) { self.f[i].belief.take() } else { None };
            let (p, ti, pt) = match reuse {
                Some(p) => (p, self.f[i].t_int, self.f[i].target),
                None => {
                    let p = self.predict(i);
                    let (ti, pt) = self.intercept(i, &p);
                    (p, ti, pt)
                }
            };
            let bias = if i >= 6 { -0.15 } else { 0.0 };
            if ti + bias < best.0 {
                best = (ti + bias, i);
            }
            self.f[i].t_int = ti;
            self.f[i].target = pt;
            self.f[i].belief = Some(p);
            self.f[i].next_plan = self.t + REPLAN_S;
        }
        self.primary = best.1;
        self.cover_roles();
        // Ground balls: every infielder who believes he can get to it in the infield breaks
        // for it at once; nobody waits for a call (agent-models §5.7, ground-ball case). Each
        // judges from his own read: that it is a ground ball (first bounce in the infield)
        // and that his intercept is in the infield.
        let mut broke = false;
        for i in [3usize, 4, 5] {
            let fd = &self.f[i];
            let grounder = fd.belief.as_ref().and_then(|p| p.first_bounce()).map(|b| m::hypot(b.1, b.2) < 25.0).unwrap_or(false);
            if i != self.primary && grounder && fd.t_int.is_finite() && m::hypot(fd.target.x, fd.target.y) < INFIELD_R {
                self.f[i].role = Role::Chase;
                broke = true;
            }
        }
        if broke {
            self.cover_roles_after_chasers();
        }
        self.assign_backups();
    }

    /// Fielders with no part in the play back it up (agent-models §5.7): one behind the man
    /// fielding the ball, and one behind each base a throw is likely to go to, on the line
    /// of that throw. One backup per base. Decided from what every fielder can see: where
    /// the ball is being fielded and which bases are occupied.
    fn assign_backups(&mut self) {
        let pi = self.primary;
        let from = self.f[pi].target;
        let depth = m::hypot(from.x, from.y);
        let outfield_hit = depth > INFIELD_R;
        let on = |b: u8| self.r.iter().any(|r| r.start == b && !r.out && !r.scored);
        let empty = !(on(1) || on(2) || on(3));
        for fd in self.f.iter_mut() {
            fd.backs = None;
            if fd.role == Role::Backup {
                fd.role = Role::Idle;
            }
        }
        // With nobody on base the catcher is not needed at home: he trails the batter to
        // back up first.
        if empty && self.f[1].role == Role::Cover(4) {
            self.f[1].role = Role::Idle;
        }
        let mut free: Vec<usize> = (0..9).filter(|&i| self.f[i].role == Role::Idle).collect();
        // Behind the fielder: the nearest free outfielder gets into the ball's line beyond
        // him (deeper than the chase point, and no shallower than he already is).
        let line = Vec3::new(from.x, from.y, 0.0).normalized();
        let ofs: Vec<usize> = free.iter().copied().filter(|&i| i >= 6).collect();
        if let Some(&i) = ofs.iter().min_by(|&&a, &&c| (self.f[a].pos - from).norm().total_cmp(&(self.f[c].pos - from).norm()).then(a.cmp(&c))) {
            let own = m::hypot(self.f[i].pos.x, self.f[i].pos.y);
            self.f[i].target = line * (depth + 8.0).max(own * 0.85);
            self.f[i].role = Role::Backup;
            free.retain(|&x| x != i);
        }
        // Behind the bases. The likeliest throw first; candidates in order of preference
        // (P 0, C 1, LF 6, CF 7, RF 8).
        let far_of = if from.x > 0.0 { 6 } else { 8 };
        let mut jobs: Vec<(u8, Vec<usize>)> = Vec::new();
        if outfield_hit {
            if on(2) || on(3) {
                jobs.push((4, vec![0]));
                jobs.push((2, vec![far_of, 7]));
            } else if on(1) {
                jobs.push((3, vec![0, 6]));
                jobs.push((2, vec![far_of, 7]));
            } else {
                jobs.push((2, vec![far_of, 0, 7]));
                jobs.push((1, vec![1]));
            }
        } else {
            jobs.push((1, vec![1, 8]));
            if on(1) {
                jobs.push((2, vec![7, far_of]));
            }
            if on(2) {
                jobs.push((3, vec![6, 0]));
            }
        }
        for (base, cands) in jobs {
            let Some(&i) = cands.iter().find(|c| free.contains(c)) else { continue };
            // A throw to first with nobody on comes from the infield or from second base's
            // side; other throws come from where the ball is fielded.
            let origin = if base == 1 && outfield_hit { base_pos(2) } else { from };
            self.f[i].target = backup_spot(origin, base);
            self.f[i].role = Role::Backup;
            self.f[i].backs = Some(base);
            free.retain(|&x| x != i);
        }
    }

    /// With the bases empty the catcher left home to back up first. Once any runner has
    /// rounded first he is a threat to score, and the catcher goes back (a visible fact:
    /// where the runners are).
    fn cover_home_again(&mut self) {
        if (0..9).any(|i| self.f[i].role == Role::Cover(4)) || matches!(self.ball, Ball::Held { by: 1 }) || self.f[1].role == Role::Chase {
            return;
        }
        if self.r.iter().any(|r| !r.out && !r.scored && r.s > 1.25 * BASE_L) {
            self.f[1].role = Role::Cover(4);
            self.f[1].backs = None;
            self.f[1].target = base_pos(4);
        }
    }

    /// A throw has been released toward `base`: its backup lines up with the real throw. If
    /// nobody has that job, the nearest fielder with nothing to do takes it.
    fn back_up_throw(&mut self, from: Vec3, base: u8) {
        let spot = backup_spot(from, base);
        let who = (0..9).find(|&i| self.f[i].backs == Some(base) && self.f[i].role == Role::Backup).or_else(|| {
            (0..9)
                .filter(|&i| matches!(self.f[i].role, Role::Idle | Role::Backup) && self.f[i].backs.is_none())
                .min_by(|&a, &c| (self.f[a].pos - spot).norm().total_cmp(&(self.f[c].pos - spot).norm()).then(a.cmp(&c)))
        });
        if let Some(i) = who {
            self.f[i].target = spot;
            self.f[i].role = Role::Backup;
            self.f[i].backs = Some(base);
        }
    }

    /// Re-assign base coverage among fielders who are not chasing.
    fn cover_roles_after_chasers(&mut self) {
        let chasing = |f: &Vec<Fielder>, i: usize| f[i].role == Role::Chase;
        let left = self.f[self.primary].target.x < 0.0;
        let mut taken: Vec<usize> = (0..9).filter(|&i| chasing(&self.f, i)).collect();
        let order: [(u8, Vec<usize>); 4] = [
            (1, vec![2, 0, 3]),
            (2, if left { vec![3, 5] } else { vec![5, 3] }),
            (3, vec![4, 5, 0]),
            (4, vec![1, 0]),
        ];
        for (base, cands) in order.iter() {
            // Keep an existing non-chasing cover if there is one.
            if let Some(c) = (0..9).find(|&i| self.f[i].role == Role::Cover(*base) && !taken.contains(&i)) {
                taken.push(c);
                continue;
            }
            if let Some(&c) = cands.iter().find(|c| !taken.contains(c)) {
                self.f[c].role = Role::Cover(*base);
                self.f[c].target = base_pos(*base);
                taken.push(c);
            }
        }
    }

    fn cover_roles(&mut self) {
        let pi = self.primary;
        let left = self.f[pi].target.x < 0.0;
        let pick = |cands: &[usize], taken: &[usize]| cands.iter().copied().find(|c| *c != pi && !taken.contains(c));
        let mut taken: Vec<usize> = Vec::new();
        let set = |base: u8, cands: &[usize], f: &mut Vec<Fielder>, taken: &mut Vec<usize>| {
            if let Some(c) = pick(cands, taken) {
                f[c].role = Role::Cover(base);
                f[c].target = base_pos(base);
                taken.push(c);
            }
        };
        // Indices are position − 1: P0 C1 1B2 2B3 3B4 SS5 LF6 CF7 RF8.
        set(1, &[2, 0, 3], &mut self.f, &mut taken);
        set(2, if left { &[3, 5] } else { &[5, 3] }, &mut self.f, &mut taken);
        set(3, &[4, 5], &mut self.f, &mut taken);
        set(4, &[1, 0], &mut self.f, &mut taken);
        let tgt = self.f[pi].target;
        for i in 0..9 {
            if i == pi {
                self.f[i].role = Role::Chase;
            } else if !taken.contains(&i) {
                // Fielders with no job yet; `assign_backups` gives most of them one.
                self.f[i].role = Role::Idle;
                let _ = tgt;
            }
        }
    }

    fn move_fielders(&mut self) {
        let (t, t_contact) = (self.t, self.inp.t_contact);
        for fd in self.f.iter_mut() {
            if fd.dove && t >= fd.ready_at {
                fd.dove = false; // recovered
            }
            (fd.pos, fd.speed) = fielder_step(fd, t, t_contact);
        }
    }

    /// Glove contact test for any fielder the ball passes (agent-models §5.4).
    fn try_captures(&mut self) {
        // At close range a chaser re-reads the ball (perception error scales with distance).
        if let Ball::Free { .. } = self.ball {
            let (b, _) = self.ball_at(self.t);
            for i in 0..9 {
                if self.f[i].role == Role::Chase && self.t >= self.f[i].last_close_read + 0.1 {
                    let d = m::hypot(b.x - self.f[i].pos.x, b.y - self.f[i].pos.y);
                    if d < 4.0 {
                        self.f[i].last_close_read = self.t;
                        let p = self.predict(i);
                        let (ti, pt) = self.intercept(i, &p);
                        self.f[i].t_int = ti;
                        self.f[i].target = pt;
                        self.f[i].belief = Some(p);
                    }
                }
            }
        }
        let (b, bv) = self.ball_at(self.t);
        let receiver = match self.ball {
            Ball::Thrown { receiver, .. } => Some(receiver),
            Ball::Free { .. } => None,
            Ball::Held { .. } => return,
        };
        // Catchable height where the ball is: a fielder at the wall can leap.
        let field = self.inp.field;
        let zmax_at = |p: Vec3| if field.fence_at(m::atan2(p.x, p.y)) - m::hypot(p.x, p.y) < 3.0 { WALL_CATCH_MAX_Z } else { CATCH_MAX_Z };
        let zmax = zmax_at(b);
        for i in 0..9 {
            let fd = &self.f[i];
            if self.t < fd.blocked_until || self.t < self.inp.t_contact + fd.a.reaction {
                continue;
            }
            // On a throw only the receiver tries, and nobody in its path in front of him; the
            // base's backup may take it only once it is beyond the receiver's reach on the far
            // side of him, so the receiver always has his own chance first.
            if let Some(rcv) = receiver.filter(|&rcv| rcv != i) {
                let past_receiver = match self.ball {
                    Ball::Thrown { to_base, .. } if fd.backs == Some(to_base) => {
                        let at = self.f[rcv].pos;
                        let vh = m::hypot(bv.x, bv.y).max(1e-9);
                        ((b.x - at.x) * bv.x + (b.y - at.y) * bv.y) / vh > RECEIVE_REACH
                    }
                    _ => false,
                };
                if !past_receiver {
                    continue;
                }
            }
            if b.z < 0.0 {
                continue; // never capture a ball below the surface
            }
            let horiz = m::hypot(b.x - fd.pos.x, b.y - fd.pos.y);
            let reach = if receiver.is_some() { RECEIVE_REACH } else { REACH };
            let standing = horiz <= reach && b.z <= zmax;
            // The glove meets the ball where it passes closest (the capture instant,
            // agent-models §5.4), not where it first enters reach. Wait a tick only if the
            // next tick's contact is certain to be there and better: the ball touches nothing
            // before then (ground, wall, fence, or coming to rest), and against where the
            // fielder will then be it is closer, still within reach, and at catchable height.
            if standing {
                let t1 = self.t + DT;
                let (bn, _) = self.ball_at(t1);
                let within = |x: f64| x > self.t && x <= t1;
                let boundary = match &self.ball {
                    Ball::Free { path, .. } => path.bounces.iter().any(|x| within(x.0)) || path.walls.iter().any(|x| within(x.0)) || path.t_end() <= t1,
                    // A throw that reaches the ground becomes a bouncing ball before the next test.
                    Ball::Thrown { .. } => bn.z <= self.inp.env.ball.radius,
                    Ball::Held { .. } => false,
                };
                let (pn, _) = fielder_step(fd, self.t, self.inp.t_contact);
                let hn = m::hypot(bn.x - pn.x, bn.y - pn.y);
                if !boundary && hn < horiz && hn <= reach && bn.z <= zmax_at(bn) && bn.z >= 0.0 {
                    continue;
                }
            }
            // Dive only for a ball the fielder's own prediction says won't come closer.
            // Nobody dives at a slow or resting ball: they walk up and field it.
            let diving = !standing && self.f[i].role == Role::Chase && bv.norm() > 3.0 && horiz <= REACH + DIVE && b.z <= DIVE_MAX_Z && {
                let mut closest = f64::INFINITY;
                if let Some(p) = &fd.belief {
                    for j in 1..=6 {
                        let (q, _) = p.at(self.t + 0.05 * j as f64);
                        if q.z <= CATCH_MAX_Z {
                            closest = closest.min(m::hypot(q.x - fd.pos.x, q.y - fd.pos.y));
                        }
                    }
                }
                closest > REACH
            };
            if !(standing || diving) {
                continue;
            }
            let mut r = rng(self.inp.seed, self.inp.play, fd.id, Purpose::GlovePlacement, fd.attempts);
            // agent-models §5.4 form; k_v 0.25 and k_reach 0.3 [E, M4b]: the spec's 0.5/0.6 made
            // routine catches fail ~8% of the time against the ≥ 98% routine-play band.
            let sg = fd.a.hands * (1.0 + 0.25 * bv.norm() / 40.0 + 0.3 * (horiz / (REACH + DIVE)).min(1.0) + if diving { 1.0 } else { 0.0 });
            let (ex, ez) = (r.truncated_normal(4.0), r.truncated_normal(4.0));
            let d = sg * m::hypot(ex, ez);
            let fd = &mut self.f[i];
            fd.attempts += 1;
            if diving {
                // A committed dive puts the fielder on the ground whatever happens next.
                fd.dove = true;
                fd.ready_at = self.t + DIVE_GETUP_S;
            }
            if d <= POCKET {
                fd.ready_at = self.t + fd.a.transfer + if diving { DIVE_GETUP_S } else { 0.0 };
                if let Ball::Thrown { to_base, .. } = self.ball {
                    // A receiver steps straight to the bag; a relay goes on quickly.
                    fd.target = base_pos(to_base);
                    fd.ready_at = self.t + RELAY_TRANSFER_FRAC * fd.a.transfer;
                }
                let id = fd.id;
                self.ball = Ball::Held { by: i };
                self.secured_at = self.t;
                self.out.secured.push((self.t, id));
                self.emit(Input::GloveSecured { t: self.t, fielder: id, x: b.x, y: b.y });
                return;
            } else if d <= RIM {
                // Deflection off the glove rim: the ball keeps a fraction of its speed in a
                // direction pushed by the miss (glove e_n ≈ 0.15, physics-spec §9.4).
                let push = Vec3::new(ex, -ez.abs(), ez).normalized();
                let nv = (bv * 0.3 + push * (0.15 * bv.norm())).reject(Vec3::Z) + Vec3::new(0.0, 0.0, 1.0);
                fd.blocked_until = self.t + 0.5;
                let s0 = BallState { r: Vec3::new(b.x, b.y, b.z.max(self.inp.env.ball.radius + 0.01)), v: nv, w: Vec3::ZERO };
                self.set_free(s0);
                return;
            } else {
                fd.blocked_until = self.t + 0.25;
            }
            if diving {
                // A missed dive leaves the fielder on the ground for the get-up time.
                fd.blocked_until = fd.ready_at;
            }
        }
    }

    fn set_free(&mut self, s0: BallState) {
        let (seed, play) = (self.inp.seed, self.inp.play);
        let k0 = self.bounce_k;
        let path = batted_path(self.inp.env, self.inp.field, s0, self.t, |k, surf| {
            let mut r = rng(seed, play, 0x0200, Purpose::SurfaceBounce, k0 + k);
            (surf.roughness * r.truncated_normal(4.0), surf.roughness * r.truncated_normal(4.0))
        });
        self.bounce_k += path.bounces.len() as u32 + 1;
        self.ball = Ball::Free { path, next_touch: 0, next_wall: 0, settled_sent: false };
        self.assign();
    }

    /// Emit ground, wall, fence, and settle facts as the free ball reaches them.
    fn ball_facts(&mut self) {
        let t = self.t;
        let floor = self.last_fact_t;
        let mut facts = Vec::new();
        if let Ball::Free { path, next_touch, next_wall, settled_sent } = &mut self.ball {
            while *next_touch < path.bounces.len() && path.bounces[*next_touch].0 <= t {
                let (tb, x, y) = path.bounces[*next_touch];
                facts.push(Input::BallTouched { t: tb.max(floor), what: Touch::Ground { x, y } });
                *next_touch += 1;
            }
            while *next_wall < path.walls.len() && path.walls[*next_wall].0 <= t {
                let (tw, x, y) = path.walls[*next_wall];
                facts.push(Input::BallTouched { t: tw.max(floor), what: Touch::Wall { x, y } });
                *next_wall += 1;
            }
            if t >= path.t_end() && !*settled_sent {
                *settled_sent = true;
                let (p, _) = path.at(path.t_end());
                match path.end() {
                    SegmentEnd::OverFence { .. } => {
                        let in_flight = path.bounces.is_empty();
                        facts.push(Input::BallTouched { t: path.t_end().max(floor), what: Touch::OutOfPlay { x: p.x, y: p.y, in_flight } });
                    }
                    SegmentEnd::Rest => facts.push(Input::BallSettled { t: path.t_end().max(floor), x: p.x, y: p.y }),
                    _ => {}
                }
            }
        }
        // Ground and wall contacts inside one tick go out in time order.
        facts.sort_by(|a, b| fact_time(a).unwrap_or(0.0).total_cmp(&fact_time(b).unwrap_or(0.0)));
        for f in facts {
            self.emit(f);
            if self.done {
                return;
            }
        }
    }

    fn update_thrown(&mut self) {
        // The receiver watches the throw and moves to where it will come down to catch height.
        if let Ball::Thrown { r0, v0, t0, receiver, .. } = self.ball {
            let a = 0.5 * G;
            let (b, c) = (-v0.z, 1.3 - r0.z);
            // z(t) = r0.z + v0.z·τ − ½gτ² = 1.3  →  aτ² + bτ + c = 0, take the later root.
            let disc = b * b - 4.0 * a * c;
            if disc >= 0.0 {
                let tau = (-b + m::sqrt(disc)) / (2.0 * a);
                let p = r0 + v0 * tau;
                let _ = t0;
                self.f[receiver].target = Vec3::new(p.x, p.y, 0.0);
            }
        }
        if let Ball::Thrown { r0, v0, t0, .. } = self.ball {
            let (b, _) = self.ball_at(self.t);
            if b.z <= self.inp.env.ball.radius {
                // The throw reached the ground: find when, and continue from there with the
                // bounce model (a short hop or a throw that got by).
                let rr = self.inp.env.ball.radius;
                let (a, bq, c) = (0.5 * G, -v0.z, rr - r0.z);
                let disc = (bq * bq - 4.0 * a * c).max(0.0);
                let tau = (-bq + m::sqrt(disc)) / (2.0 * a);
                let p = r0 + v0 * tau + Vec3::new(0.0, 0.0, -0.5 * G * tau * tau);
                let v = v0 + Vec3::new(0.0, 0.0, -G * tau);
                let _ = t0;
                self.set_free(BallState { r: Vec3::new(p.x, p.y, rr), v, w: Vec3::ZERO });
            }
        }
    }

    /// Runners' believed time for the ball to reach base `b` (they watch the ball and the
    /// fielders; agent-models §6.3).
    fn ball_time_to(&self, k: usize, b: u8, read_sub: u32) -> f64 {
        let target = base_pos(b);
        match &self.ball {
            Ball::Held { .. } => {
                // The runner sees where the ball is and when it was caught.
                let (p, _) = self.ball_at(self.t);
                let ready = (self.secured_at + TYPICAL_TRANSFER).max(self.t);
                ready + m::hypot(target.x - p.x, target.y - p.y) / (THROW_FRAC * TYPICAL_ARM)
            }
            Ball::Thrown { .. } => {
                // A throw in the air: where it is and how fast it is coming.
                let (p, v) = self.ball_at(self.t);
                let vh = m::hypot(v.x, v.y).max(5.0);
                let to = |q: Vec3| m::hypot(q.x - p.x, q.y - p.y);
                let heading = Vec3::new(v.x, v.y, 0.0).normalized();
                // The base the throw is going to: the one most nearly along its heading.
                let dest = (1..=4u8)
                    .max_by(|&a, &c| {
                        let da = (base_pos(a) - p).normalized().dot(heading);
                        let dc = (base_pos(c) - p).normalized().dot(heading);
                        da.total_cmp(&dc)
                    })
                    .unwrap();
                let arrive = self.t + to(base_pos(dest)) / vh;
                if dest == b { arrive } else { arrive + 0.6 + (target - base_pos(dest)).norm() / (THROW_FRAC * TYPICAL_ARM) }
            }
            Ball::Free { .. } => {
                let t0 = prof::enabled().then(std::time::Instant::now);
                let pred = self.runner_predict(k, read_sub);
                let (tf, at) = self.believed_field_time(&pred);
                if let Some(t0) = t0 {
                    prof::add(&prof::RUNNER, t0.elapsed().as_nanos() as u64);
                }
                tf + TYPICAL_TRANSFER + (target - at).norm() / (THROW_FRAC * TYPICAL_ARM)
            }
        }
    }

    /// Forced while the batter is still a runner and every base behind this runner is held by
    /// a runner still in play (the runner sees outs as they are signalled).
    fn forced(&self, k: usize) -> bool {
        let r = &self.r[k];
        let batter_alive = self.r.iter().any(|x| x.start == 0 && !x.out);
        r.start == 0 || (batter_alive && (1..r.start).all(|b| self.r.iter().any(|x| x.start == b && !x.out)))
    }

    /// Whether a runner ahead of runner `k` holds base `b`: he is at or short of it and not
    /// visibly moving forward (standing, or going back). Direction of motion is visible.
    fn base_held_ahead(&self, k: usize, b: u8) -> bool {
        let r = &self.r[k];
        self.r.iter().any(|o| {
            o.id != r.id && !o.out && !o.scored && o.start > r.start && o.s <= b as f64 * BASE_L + 0.5 && o.goal as f64 * BASE_L <= o.s + 1e-6
        })
    }

    /// Whether a runner ahead of runner `k` has not yet passed base `b`.
    fn base_uncleared_ahead(&self, k: usize, b: u8) -> bool {
        let r = &self.r[k];
        self.r.iter().any(|o| o.id != r.id && !o.out && !o.scored && o.start > r.start && o.s <= b as f64 * BASE_L + 0.5)
    }

    fn runner_decide(&mut self, k: usize, at_base: u8) {
        let forced = self.forced(k);
        let next = at_base + 1;
        if next > 4 {
            return;
        }
        // Every decision is a new look at the play: its own noise draws (data-formats §2.3).
        self.r[k].decisions += 1;
        let n = self.r[k].decisions;
        let r = &self.r[k];
        let t_run = run_time((next as f64 * BASE_L - r.s).max(0.0), r.vmax, RUNNER_TAU) * if r.v > 1.0 { 0.75 } else { 1.0 };
        let t_ball = self.ball_time_to(k, next, 2 * n + 1);
        let mut rr = rng(self.inp.seed, self.inp.play, r.id, Purpose::RunnerPerception, 2 * n);
        let draw = rr.truncated_normal(4.0);
        let noise = 0.15 * draw;
        let must = forced && at_base == r.start && !self.caught();
        // A runner may only take a base once every runner ahead of him has visibly cleared it;
        // until then he waits at his base.
        let blocked = !must && self.base_uncleared_ahead(k, next);
        let go = !blocked && (must || t_ball - (self.t + t_run) + noise > 0.35);
        let r = &mut self.r[k];
        r.last_timing_draw = draw;
        r.waiting = blocked;
        if go {
            r.goal = next;
        } else {
            r.goal = at_base;
        }
    }

    fn caught(&self) -> bool {
        self.catch_made
    }

    fn move_runners(&mut self) {
        for k in 0..self.r.len() {
            if self.r[k].out || self.r[k].scored || self.done {
                continue;
            }
            let start_t = if self.r[k].start == 0 { self.inp.t_contact + BATTER_START } else { self.inp.t_contact };
            if self.t < start_t {
                continue;
            }
            // The catch, seen once: a runner off his base at that moment owes a retouch; one
            // standing on it (tagging up or not) owes nothing and may leave for good.
            if self.caught() && !self.r[k].catch_seen {
                self.r[k].catch_seen = true;
                if self.r[k].start > 0 && self.r[k].s > self.r[k].start as f64 * BASE_L + 1e-6 {
                    self.r[k].tag_up = false;
                    self.r[k].hold_s = None;
                    self.r[k].retouch = true;
                    self.r[k].waiting = false;
                    self.r[k].goal = self.r[k].start;
                }
            }
            // Tag-up: go back to the base and hold it until the catch, then decide.
            if self.r[k].tag_up {
                let on_base = (self.r[k].s - self.r[k].start as f64 * BASE_L).abs() < 1e-6;
                if self.caught() {
                    // (A runner still off his base at the catch was sent back above.)
                    self.r[k].tag_up = false;
                    let b = self.r[k].last_base;
                    self.emit(Input::BaseTouched { t: self.t, agent: self.r[k].id, base: b, holding_ball: false });
                    self.runner_decide(k, b);
                } else if !matches!(self.ball, Ball::Free { .. }) || self.out.facts.iter().any(|f| matches!(f, Input::BallTouched { what: Touch::Ground { .. }, .. })) {
                    // The ball dropped in: run.
                    self.r[k].tag_up = false;
                    let b = self.r[k].last_base;
                    self.runner_decide(k, b);
                } else if on_base {
                    continue;
                } else {
                    self.r[k].goal = self.r[k].start;
                }
            }
            // Halfway on a ball in the air: once it is down (or fielded on a hop), decide.
            if self.r[k].hold_s.is_some() && !self.caught() && (!matches!(self.ball, Ball::Free { .. }) || self.out.facts.iter().any(|f| matches!(f, Input::BallTouched { what: Touch::Ground { .. }, .. }))) {
                self.r[k].hold_s = None;
                let b = self.r[k].start;
                self.runner_decide(k, b);
            }
            // Waiting at a base: decide once the runner ahead has cleared the next one.
            if self.r[k].waiting && (self.r[k].s - self.r[k].goal as f64 * BASE_L).abs() < 1e-6 {
                let b = self.r[k].goal;
                if !self.base_uncleared_ahead(k, b + 1) {
                    self.runner_decide(k, b);
                }
            }
            // Heading for a base a runner ahead has stopped at (or is retreating to): go back,
            // unless this is a forced advance (then the runner ahead is forced too).
            let g = self.r[k].goal;
            let forced_here = g == self.r[k].start + 1 && self.forced(k);
            if g as f64 * BASE_L > self.r[k].s && self.base_held_ahead(k, g) && !forced_here {
                self.r[k].goal = (self.r[k].s / BASE_L).floor() as u8;
            }
            let goal_s = self.r[k].hold_s.unwrap_or(self.r[k].goal as f64 * BASE_L);
            let r = &mut self.r[k];
            let dir = (goal_s - r.s).signum();
            if (goal_s - r.s).abs() < 1e-6 {
                r.v = 0.0;
                continue;
            }
            if r.v == 0.0 {
                r.moving_since = self.t;
            }
            r.v += (r.vmax - r.v) * (DT / RUNNER_TAU);
            let prev = r.s;
            let nxt = prev + dir * r.v * DT;
            let crossed = |b: u8| {
                let bs = b as f64 * BASE_L;
                (prev < bs && nxt >= bs) || (prev > bs && nxt <= bs)
            };
            let mut touched: Option<u8> = None;
            for b in 0..=4u8 {
                if b as f64 * BASE_L != prev && crossed(b) {
                    touched = Some(b);
                }
            }
            // Leaving a base.
            let left = (0..=3u8).find(|&b| (prev - b as f64 * BASE_L).abs() < 1e-9 && b >= 1);
            r.s = if (dir > 0.0 && nxt >= goal_s) || (dir < 0.0 && nxt <= goal_s) { goal_s } else { nxt };
            let id = r.id;
            if let Some(b) = left {
                self.emit(Input::BaseLeft { t: self.t, agent: id, base: b });
            }
            if let Some(b) = touched {
                let r = &mut self.r[k];
                r.last_base = if dir > 0.0 { b } else { b };
                if b == 4 {
                    r.scored = true;
                }
                self.emit(Input::BaseTouched { t: self.t, agent: id, base: b, holding_ball: false });
                if self.done {
                    return;
                }
                let retouched = dir < 0.0 && self.r[k].retouch && b == self.r[k].start;
                if retouched {
                    self.r[k].retouch = false;
                }
                if b < 4 && (dir > 0.0 || retouched) && b == self.r[k].goal {
                    // Arrived where it was going (or retouched after a catch): decide whether
                    // to keep going (leaving the base is reported on the next step).
                    self.runner_decide(k, b);
                }
            }
        }
    }

    /// The fielder holding the ball: throw, run it to a base, tag a runner, or hold.
    fn holder_act(&mut self) {
        let Ball::Held { by } = self.ball else { return };
        let fd = self.f[by].clone();
        // Tag any runner within reach who is off a base.
        for k in 0..self.r.len() {
            let r = &self.r[k];
            if r.out || r.scored {
                continue;
            }
            let on_base = (r.s / BASE_L - (r.s / BASE_L).round()).abs() * BASE_L < 0.3;
            if !on_base && (self.runner_pos(r) - fd.pos).norm() < 1.2 {
                let id = r.id;
                self.emit(Input::TagContact { t: self.t, fielder: fd.id, runner: id, holding_ball: true });
                if self.done {
                    return;
                }
            }
        }
        // Standing on a base with the ball: a force or appeal is made by touching it.
        for b in 1..=4u8 {
            if (base_pos(b) - fd.pos).norm() < 0.6 {
                let base = if b == 4 { 4 } else { b };
                let pending = self.r.iter().any(|r| !r.out && !r.scored && r.goal >= base && r.s < base as f64 * BASE_L - 0.1)
                    || self.r.iter().any(|r| !r.out && r.start == base && r.s > base as f64 * BASE_L + 0.1);
                if pending && !self.out.facts.iter().any(|f| matches!(f, Input::BaseTouched { agent, base: bb, holding_ball: true, .. } if *agent == fd.id && *bb == base)) {
                    self.emit(Input::BaseTouched { t: self.t, agent: fd.id, base, holding_ball: true });
                    if self.done {
                        return;
                    }
                }
            }
        }
        // A forced runner heading for a base this fielder is closer to: run it there now
        // (stepping on a bag needs no transfer).
        for r in &self.r {
            if r.out || r.scored || r.v < 0.5 {
                continue;
            }
            let goal = visible_target(r);
            let tgt = base_pos(goal);
            let d_me = (tgt - fd.pos).norm();
            let t_me = run_time(d_me, fd.a.speed, fd.a.tau_a);
            let t_runner = runner_eta((goal as f64 * BASE_L - r.s).abs(), r.v);
            if d_me < 8.0 && t_me + 0.1 < t_runner {
                self.f[by].target = tgt;
                return;
            }
        }
        if self.t < fd.ready_at {
            return;
        }
        // Options: each base a running runner is heading to.
        let mut best: Option<(f64, u8)> = None;
        for r in &self.r {
            if r.out || r.scored || r.v < 0.5 {
                continue; // standing runners are not a play
            }
            // The base a moving runner is heading for, and when he gets there at his
            // current speed (both visible).
            let goal = visible_target(r);
            let t_runner = self.t + runner_eta((goal as f64 * BASE_L - r.s).abs(), r.v);
            let d = (base_pos(goal) - fd.pos).norm();
            // The ball beats the runner only once someone is at the bag to take it.
            let recv = self.receiver_for(by, goal);
            let t_ball = (self.t + d / (THROW_FRAC * fd.a.arm)).max(self.receiver_ready(recv, goal)) + 0.05;
            let p_out = phi((t_runner - t_ball) / 0.12);
            let w = [0.9, 1.0, 1.1, 1.3][(goal as usize).saturating_sub(1).min(3)];
            // Take the sure out: an uncertain play on a lead runner loses to a certain one
            // elsewhere (the chance is squared before the base's weight is applied).
            if p_out > 0.3 && best.map(|b| p_out * p_out * w > b.0).unwrap_or(true) {
                best = Some((p_out * p_out * w, goal));
            }
        }
        let infield = m::hypot(fd.pos.x, fd.pos.y) < INFIELD_R;
        if debug_throw() {
            eprintln!("t {:.2} holder {} at ({:.1},{:.1}) best {:?}; runners {:?}", self.t, by, fd.pos.x, fd.pos.y, best, self.r.iter().map(|r| (r.start, r.s, r.goal, r.out, r.scored)).collect::<Vec<_>>());
        }
        let target_base = match best {
            Some((_, b)) => Some(b),
            // No play: get the ball back to the infield.
            None if !infield => Some(2),
            None => None,
        };
        let Some(b) = target_base else { return };
        let tgt = base_pos(b);
        if (tgt - fd.pos).norm() < 6.0 {
            // Closer to take it himself.
            self.f[by].target = tgt;
            return;
        }
        let receiver = self.receiver_for(by, b);
        // Time the release so the ball arrives when the receiver can be at the bag.
        if self.t + (tgt - fd.pos).norm() / (THROW_FRAC * fd.a.arm) < self.receiver_ready(receiver, b) - 0.05 {
            self.f[receiver].target = tgt;
            return;
        }
        let mut r = rng(self.inp.seed, self.inp.play, fd.id, Purpose::ThrowExecution, fd.throws);
        self.f[by].throws += 1;
        let r0 = fd.pos + Vec3::new(0.0, 0.0, 1.8);
        let aim = tgt + Vec3::new(0.0, 0.0, 1.3);
        let d = m::hypot(aim.x - r0.x, aim.y - r0.y);
        let vh = THROW_FRAC * fd.a.arm * (1.0 + 0.03 * r.truncated_normal(4.0));
        let tf = d / vh;
        let vz = (aim.z - r0.z + 0.5 * G * tf * tf) / tf;
        let h = Vec3::new(aim.x - r0.x, aim.y - r0.y, 0.0).normalized().rotate(Vec3::Z, fd.a.arm_sigma * r.truncated_normal(4.0));
        let v0 = h * vh + Vec3::new(0.0, 0.0, vz);
        self.ball = Ball::Thrown { r0, v0, t0: self.t, to_base: b, receiver };
        self.out.released.push((self.t, fd.id));
        self.f[receiver].target = tgt;
        self.back_up_throw(fd.pos, b);
    }

    /// Who takes a throw at base `b`: its cover man, else the nearest other fielder.
    fn receiver_for(&self, by: usize, b: u8) -> usize {
        let tgt = base_pos(b);
        (0..9).find(|&i| i != by && self.f[i].role == Role::Cover(b)).unwrap_or_else(|| {
            (0..9).filter(|&i| i != by).min_by(|&a, &c| (self.f[a].pos - tgt).norm().total_cmp(&(self.f[c].pos - tgt).norm())).unwrap()
        })
    }

    /// When fielder `i` can take a throw at base `b`, as a teammate judges it: his visible
    /// distance from the bag at league-typical speed from a standing start.
    fn receiver_ready(&self, i: usize, b: u8) -> f64 {
        let d = (base_pos(b) - self.f[i].pos).norm() - 0.5;
        if d <= 0.0 { self.t } else { self.t + run_time(d, TYPICAL_SPEED, 0.9) }
    }

    fn settled(&self) -> bool {
        let Ball::Held { by } = self.ball else { return false };
        let fd = &self.f[by];
        let calm = self.r.iter().all(|r| r.out || r.scored || (r.s - r.goal as f64 * BASE_L).abs() < 1e-6);
        let in_infield = m::hypot(fd.pos.x, fd.pos.y) < INFIELD_R;
        calm && in_infield && self.t >= fd.ready_at
    }

    /// Where a runner is. Runners move along the base path (`path_pos`), except that the
    /// batter starts in his box, off the path: his offset from the plate shrinks linearly to
    /// nothing over his first `BOX_EXIT_M` of running. Tags, his own view of the ball, and the
    /// recorded track all use this one position. The distance he is charged for is still the
    /// base-path distance, which is within a few centimetres of this route's length.
    fn runner_pos(&self, r: &Runner) -> Vec3 {
        let p = path_pos(r.s);
        if r.start == 0 && r.s < BOX_EXIT_M {
            let b = self.inp.batter_box;
            p + Vec3::new(b.x, b.y, 0.0) * (1.0 - r.s / BOX_EXIT_M)
        } else {
            p
        }
    }

    /// Record the ball every tick and everyone's position every 0.1 s; `last` forces both,
    /// so the track ends on the instant that ended the play.
    fn record(&mut self, last: bool) {
        let (b, _) = self.ball_at(self.t);
        self.out.ball_track.push((self.t, b));
        if last || ((self.t - self.inp.t_contact) / 0.1).fract() < DT / 0.1 {
            for (i, fd) in self.f.iter().enumerate() {
                self.out.actors[i].1.push((self.t, fd.pos));
            }
            for k in 0..self.r.len() {
                if !self.r[k].out {
                    let p = self.runner_pos(&self.r[k]);
                    self.out.actors[9 + k].1.push((self.t, p));
                }
            }
        }
    }
}

/// Simulate one live-ball play, stepping the rules engine as facts occur. The caller has
/// already sent `PitchDelivered` and `BatContact`.
pub fn simulate_live(inp: &LiveInput, st: &mut RulesState) -> LiveOutput {
    if !prof::enabled() {
        return simulate_live_inner(inp, st);
    }
    let t0 = std::time::Instant::now();
    let out = simulate_live_inner(inp, st);
    prof::add(&prof::LIVE, t0.elapsed().as_nanos() as u64);
    out
}

/// The play at bat contact: the batted ball's true path, fielders at their positions, runners
/// on their bases.
fn new_sim<'a>(inp: &'a LiveInput<'a>, st: &'a mut RulesState) -> Sim<'a> {
    let t_c = inp.t_contact;
    let (seed, play) = (inp.seed, inp.play);
    let path = batted_path(inp.env, inp.field, inp.ball, t_c, |k, surf| {
        let mut r = rng(seed, play, 0x0200, Purpose::SurfaceBounce, k);
        (surf.roughness * r.truncated_normal(4.0), surf.roughness * r.truncated_normal(4.0))
    });
    let home = home_positions();
    let f: Vec<Fielder> = (0..9)
        .map(|i| {
            let mut r = rng(seed, play, inp.fielders[i], Purpose::FielderPerception, 0);
            Fielder {
                id: inp.fielders[i],
                a: inp.fattrs[i],
                pos: home[i],
                speed: 0.0,
                role: Role::Idle,
                target: home[i],
                next_plan: 0.0,
                err: [r.truncated_normal(4.0), r.truncated_normal(4.0), r.truncated_normal(4.0), r.truncated_normal(4.0)],
                attempts: 0,
                blocked_until: 0.0,
                ready_at: 0.0,
                dove: false,
                belief: None,
                t_int: f64::INFINITY,
                last_close_read: 0.0,
                throws: 0,
                backs: None,
            }
        })
        .collect();
    let r: Vec<Runner> = inp
        .runners
        .iter()
        .map(|x| Runner {
            id: x.id,
            start: x.base,
            s: if x.base == 0 { 0.0 } else { lead_s(x.base) },
            v: 0.0,
            vmax: x.speed,
            goal: x.base,
            out: false,
            scored: false,
            tag_up: false,
            moving_since: 0.0,
            last_base: x.base,
            retouch: false,
            waiting: false,
            // Until his first read of the ball he stays where his lead put him.
            hold_s: if x.base == 0 { None } else { Some(lead_s(x.base)) },
            last_timing_draw: 0.0,
            catch_seen: false,
            decisions: 0,
        })
        .collect();
    let mut actors: Vec<(AgentId, Vec<(f64, Vec3)>)> = inp.fielders.iter().map(|&id| (id, Vec::new())).collect();
    actors.extend(inp.runners.iter().map(|x| (x.id, Vec::new())));
    Sim {
        inp,
        st,
        t: t_c,
        ball: Ball::Free { path: path.clone(), next_touch: 0, next_wall: 0, settled_sent: false },
        f,
        r,
        primary: 0,
        out: LiveOutput { facts: Vec::new(), events: Vec::new(), path, ball_track: Vec::new(), actors, secured: Vec::new(), released: Vec::new() },
        bounce_k: 1000,
        done: false,
        last_fact_t: t_c,
        secured_at: f64::INFINITY,
        catch_made: false,
    }
}

fn simulate_live_inner(inp: &LiveInput, st: &mut RulesState) -> LiveOutput {
    let t_c = inp.t_contact;
    let play = inp.play;
    let mut sim = new_sim(inp, st);
    // Runners are off their bases with their leads as the ball is hit: the rules hear of it.
    for k in 0..sim.r.len() {
        if sim.r[k].start > 0 {
            let (id, base) = (sim.r[k].id, sim.r[k].start);
            sim.emit(Input::BaseLeft { t: t_c, agent: id, base });
        }
    }
    // Reactions: the defense reads the ball after its reaction time; runners decide at once
    // from what they see (a ball in the air that a fielder is settling under: tag up).
    let react = inp.fattrs.iter().map(|a| a.reaction).fold(f64::INFINITY, f64::min);
    let mut assigned = false;
    let mut runners_started = false;
    let mut next_recall = t_c + 1.0;
    while !sim.done {
        if sim.t - t_c > PLAY_LIMIT_S {
            if std::env::var("BB_DEBUG_LIVE").is_ok() {
                eprintln!("timeout play {}: ball {:?}", play, match &sim.ball { Ball::Free { .. } => "free".to_string(), Ball::Held { by } => format!("held by {} at {:?} ready {}", by, sim.f[*by].pos, sim.f[*by].ready_at), Ball::Thrown { to_base, receiver, .. } => format!("thrown to {to_base} recv {receiver}") });
                let (bp, bv) = sim.ball_at(sim.t);
                eprintln!("  ball at ({:.2},{:.2},{:.2}) v ({:.2},{:.2},{:.2}); primary {} blocked_until {:.2} dove {} next_plan {:.2}", bp.x, bp.y, bp.z, bv.x, bv.y, bv.z, sim.primary, sim.f[sim.primary].blocked_until, sim.f[sim.primary].dove, sim.f[sim.primary].next_plan);
                if let Ball::Thrown { r0, v0, t0, .. } = &sim.ball { eprintln!("  throw r0 {:?} v0 {:?} t0 {:.2} now {:.2}", r0, v0, t0, sim.t); }
                for r in &sim.r {
                    eprintln!("  runner {} start {} s {:.1} goal {} v {:.1} out {} scored {} tag_up {}", r.id, r.start, r.s, r.goal, r.v, r.out, r.scored, r.tag_up);
                }
                for (i, fd) in sim.f.iter().enumerate() {
                    eprintln!("  f{i} pos ({:.1},{:.1}) target ({:.1},{:.1}) role {:?}", fd.pos.x, fd.pos.y, fd.target.x, fd.target.y, fd.role);
                }
            }
            let t = sim.t;
            sim.emit(Input::PlayTimeout { t });
            break;
        }
        if !assigned && sim.t >= t_c + react {
            sim.assign();
            assigned = true;
        }
        if assigned && !runners_started && sim.t >= t_c + 0.5 {
            runners_started = true;
            let (b, bv) = sim.ball_at(sim.t);
            let airborne = matches!(sim.ball, Ball::Free { .. }) && (b.z > 1.5 || bv.z > 0.0);
            for k in 0..sim.r.len() {
                sim.r[k].hold_s = None;
                // Each runner reads the ball himself. If a visible fielder can get under it
                // before it lands, he expects a catch and goes back to tag up. If it is a
                // fly or liner to the outfield that might drop, with fewer than two out, he
                // goes partway and watches. Otherwise he decides now.
                let (likely_catch, deep_fly) = if airborne && sim.r[k].start > 0 {
                    let pred = sim.runner_predict(k, 1);
                    let first = pred.first_bounce();
                    let land = first.map(|x| x.0).unwrap_or(pred.t_end());
                    (sim.believed_field_time(&pred).0 <= land + 0.3, first.map(|x| m::hypot(x.1, x.2) > INFIELD_R).unwrap_or(true))
                } else {
                    (false, false)
                };
                if sim.r[k].start == 0 {
                    sim.r[k].goal = 1;
                } else if likely_catch {
                    sim.r[k].tag_up = true;
                } else if deep_fly && sim.st.outs < 2 {
                    let base = sim.r[k].start;
                    sim.r[k].hold_s = Some((lead_s(base) + HALFWAY_M).min((base as f64 + 0.6) * BASE_L));
                } else {
                    let b0 = sim.r[k].start;
                    sim.runner_decide(k, b0);
                }
            }
        }
        if !runners_started {
            // Before decisions everyone forced still runs on contact: batter to first.
            if let Some(k) = sim.r.iter().position(|r| r.start == 0) {
                sim.r[k].goal = 1;
            }
        }
        // Close to a slow ball, the chaser simply goes to it (seen directly at short range).
        if assigned && matches!(sim.ball, Ball::Free { .. }) {
            let (b, bv) = sim.ball_at(sim.t);
            let pi = sim.primary;
            if bv.norm() < 3.0 && (Vec3::new(b.x, b.y, 0.0) - sim.f[pi].pos).norm() < 5.0 {
                sim.f[pi].target = Vec3::new(b.x, b.y, 0.0);
            }
        }
        // Other chasers re-aim at their own predictions as the ball comes on.
        if assigned && matches!(sim.ball, Ball::Free { .. }) {
            for i in 0..9 {
                if i != sim.primary && sim.f[i].role == Role::Chase && sim.t >= sim.f[i].next_plan {
                    let p = sim.predict(i);
                    let (ti, pt) = sim.intercept(i, &p);
                    sim.f[i].t_int = ti;
                    sim.f[i].target = pt;
                    sim.f[i].belief = Some(p);
                    sim.f[i].next_plan = sim.t + REPLAN_S;
                }
            }
        }
        // Replans for the primary, and reassignment when the ball is loose again.
        if assigned && sim.t >= sim.f[sim.primary].next_plan && matches!(sim.ball, Ball::Free { .. }) {
            let pi = sim.primary;
            let p = sim.predict(pi);
            let (ti, pt) = sim.intercept(pi, &p);
            sim.f[pi].t_int = ti;
            sim.f[pi].target = pt;
            sim.f[pi].belief = Some(p);
            sim.f[pi].next_plan = sim.t + REPLAN_S;
            // Once a second, a ball the chaser is still far from is re-called by the defense
            // (calls, agent-models §5.7).
            let (b, _) = sim.ball_at(sim.t);
            if sim.t >= next_recall && (Vec3::new(b.x, b.y, 0.0) - sim.f[pi].pos).norm() > 8.0 {
                next_recall = sim.t + 1.0;
                sim.recall();
            }
        }
        sim.ball_facts();
        if sim.done {
            break;
        }
        sim.update_thrown();
        sim.try_captures();
        if sim.done {
            break;
        }
        sim.holder_act();
        if sim.done {
            break;
        }
        sim.cover_home_again();
        sim.move_fielders();
        sim.move_runners();
        if sim.done {
            break;
        }
        // A runner who reached base and stopped while a throw comes in stays put.
        if sim.settled() {
            let t = sim.t;
            sim.emit(Input::PlayEndCheck { t });
            if sim.done {
                break;
            }
        }
        if inp.record {
            sim.record(false);
        }
        prof::TICKS.with(|c| c.set(c.get() + 1));
        sim.t += DT;
    }
    if inp.record {
        // The tick that ended the play (the catch, the force, the ball leaving the park).
        sim.record(true);
    }
    sim.out
}

#[cfg(test)]
mod tests {
    use super::*;
    use bb_math::units::MPH;

    const CF: usize = 7;

    struct Fixture {
        env: FlightEnv,
        field: Field,
        st: RulesState,
    }

    /// A rules state with the ball just put in play, runners (id, base) on base.
    fn fixture(on_base: &[(AgentId, u8)]) -> Fixture {
        let mut st = RulesState::default_fixture();
        step(&mut st, &Input::StartGame);
        for &(id, b) in on_base {
            st.bases[b as usize - 1] = Some(id);
        }
        step(&mut st, &Input::PitchDelivered { t: 0.0 });
        step(&mut st, &Input::BatContact { t: 0.4 });
        assert!(matches!(st.play, PlayState::Live(_)));
        Fixture { env: FlightEnv::default_env(), field: Field::default(), st }
    }

    /// A fly ball to center; perfect hands, so glove noise decides nothing in these tests.
    fn input<'a>(fx: &'a Fixture, runners: Vec<RunnerIn>) -> LiveInput<'a> {
        let mut fattrs: [FielderAttrs; 9] = core::array::from_fn(FielderAttrs::average);
        for a in fattrs.iter_mut() {
            a.hands = 0.0;
        }
        LiveInput {
            env: &fx.env,
            field: &fx.field,
            ball: BallState { r: Vec3::new(0.0, 0.5, 1.0), v: Vec3::new(0.0, 30.0, 30.0) * (95.0 * MPH / 42.43), w: Vec3::ZERO },
            t_contact: 0.4,
            runners,
            fielders: core::array::from_fn(|i| 0x0180 | (i as u16 + 1)),
            fattrs,
            batter_box: Vec3::new(-0.95, 0.30, 0.0),
            seed: 11,
            play: 3,
            record: false,
        }
    }

    /// Put a free ball at `r` with velocity `v` at time `t`, with nobody assigned to it.
    fn loose_ball(sim: &mut Sim, t: f64, r: Vec3, v: Vec3) {
        sim.t = t;
        sim.set_free(BallState { r, v, w: Vec3::ZERO });
        for fd in sim.f.iter_mut() {
            fd.role = Role::Idle;
            fd.target = fd.pos;
            fd.speed = 0.0;
        }
    }

    /// The world part of a tick, as in `simulate_live_inner` (no planning, no holder).
    fn world_tick(sim: &mut Sim) {
        sim.ball_facts();
        sim.try_captures();
        sim.move_fielders();
        sim.t += DT;
    }

    fn caught(sim: &Sim) -> bool {
        sim.out.events.iter().any(|e| matches!(e, Event::Catch { .. }))
    }

    // Codex day review #2: a fly a fielder can catch just before it reaches the wall is an out,
    // not a carom (rules-spec §5.3: after the wall a catch is no longer possible).
    #[test]
    fn fly_catchable_just_before_the_wall_is_caught() {
        let mut fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let fence = fx.field.fence_at(0.0);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.f[CF].pos = Vec3::new(0.0, fence - 0.02, 0.0);
        loose_ball(&mut sim, 2.0, Vec3::new(0.0, fence - 0.15, 1.3), Vec3::new(0.0, 10.0, 0.0));
        for _ in 0..6 {
            world_tick(&mut sim);
        }
        assert!(caught(&sim), "facts: {:?}", sim.out.facts);
        assert!(!sim.out.facts.iter().any(|f| matches!(f, Input::BallTouched { what: Touch::Wall { .. }, .. })));
        fx.st = st;
    }

    // Codex day review #3: a ball within reach of a fielder who is running away from it faster
    // than it follows is caught now; a tick later it is out of reach.
    #[test]
    fn ball_in_reach_of_a_fielder_moving_away_is_caught_now() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.f[CF].pos = Vec3::new(0.0, 80.0, 0.0);
        loose_ball(&mut sim, 2.0, Vec3::new(0.0, 79.05, 1.0), Vec3::new(0.0, 1.0, 0.0));
        sim.f[CF].speed = 8.0;
        sim.f[CF].target = Vec3::new(0.0, 110.0, 0.0);
        world_tick(&mut sim);
        let secured = sim.out.facts.iter().find_map(|f| if let Input::GloveSecured { t, .. } = f { Some(*t) } else { None });
        assert_eq!(secured, Some(2.0), "facts: {:?}", sim.out.facts);
    }

    // agent-models §5.4: the glove meets the ball where it passes closest, not where it first
    // comes within reach. A liner passing a standing fielder is taken within one tick's travel
    // of his position, not at the edge of his reach.
    #[test]
    fn standing_catch_happens_at_closest_approach() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.f[CF].pos = Vec3::new(0.0, 80.0, 0.0);
        let speed = 20.0;
        loose_ball(&mut sim, 2.0, Vec3::new(0.0, 78.5, 1.5), Vec3::new(0.0, speed, 1.0));
        for _ in 0..12 {
            world_tick(&mut sim);
        }
        let y = sim.out.facts.iter().find_map(|f| if let Input::GloveSecured { y, .. } = f { Some(*y) } else { None });
        let y = y.expect("the ball passed through the fielder's reach");
        assert!((y - 80.0).abs() <= speed * DT, "caught {:.3} m from the fielder", (y - 80.0).abs());
    }

    /// A deep fly to center is caught at time 5.0 with the given runners in the given states;
    /// then only the runners act (nobody throws) for 12 s.
    fn after_catch(sim: &mut Sim) {
        sim.t = 5.0;
        sim.f[CF].pos = Vec3::new(0.0, 120.0, 0.0);
        let id = sim.f[CF].id;
        sim.emit(Input::GloveSecured { t: 5.0, fielder: id, x: 0.0, y: 120.0 });
        sim.ball = Ball::Held { by: CF };
        sim.secured_at = 5.0;
        assert!(sim.catch_made);
        for _ in 0..720 {
            if sim.done {
                break;
            }
            sim.move_runners();
            sim.t += DT;
        }
    }

    fn count(sim: &Sim, pred: impl Fn(&Input) -> bool) -> usize {
        sim.out.facts.iter().filter(|f| pred(f)).count()
    }

    // Codex day review #1: a runner who tags up legally leaves once and scores (a sacrifice
    // fly); he is not sent back to retouch. The runner is fast enough that the decision to go
    // does not hang on his timing noise.
    #[test]
    fn legal_tag_up_from_third_scores() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 3)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R, base: 3, speed: 12.0 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.r[1].tag_up = true;
        after_catch(&mut sim);
        // He touches home (the run is credited when the play ends) having left third once.
        assert_eq!(count(&sim, |f| matches!(f, Input::BaseTouched { agent: R, base: 4, .. })), 1, "facts: {:?}", sim.out.facts);
        assert_eq!(count(&sim, |f| matches!(f, Input::BaseTouched { agent: R, base: 3, .. })), 1);
        assert_eq!(count(&sim, |f| matches!(f, Input::BaseLeft { agent: R, base: 3, .. })), 1);
        assert!(!sim.out.events.iter().any(|e| matches!(e, Event::RulesWarning { .. })));
    }

    // A runner caught off his base by the catch goes back, retouches once, and is then free:
    // given a clear chance he takes the next base and is not sent back again (review #1; the
    // day-fixes review #6 asked for the advance after the retouch). He left second before the
    // catch, as the rules engine was told. His speed is far above a real runner's so that the
    // decision to go does not hang on his timing noise.
    #[test]
    fn runner_off_base_at_the_catch_retouches_once_then_advances() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R, base: 2, speed: 16.0 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.emit(Input::BaseLeft { t: 4.0, agent: R, base: 2 });
        sim.r[1].s = 2.0 * BASE_L + 8.0;
        sim.r[1].goal = 3;
        sim.r[1].v = 6.0;
        after_catch(&mut sim);
        // Second: the one retouch. Third: reached once. He left second twice (before the
        // catch and after the retouch) and never came back to it a second time.
        assert_eq!(count(&sim, |f| matches!(f, Input::BaseTouched { agent: R, base: 2, holding_ball: false, .. })), 1, "facts: {:?}", sim.out.facts);
        assert_eq!(count(&sim, |f| matches!(f, Input::BaseTouched { agent: R, base: 3, holding_ball: false, .. })), 1, "facts: {:?}", sim.out.facts);
        assert_eq!(count(&sim, |f| matches!(f, Input::BaseLeft { agent: R, base: 2, .. })), 2);
        assert!(!sim.out.events.iter().any(|e| matches!(e, Event::RulesWarning { .. })));
    }

    // Day-fixes review #3: within 3 m of the wall a fielder can catch higher (a leap). A ball
    // leaving that strip must be judged by the height limit where it will be, so a catch
    // available now is not put off to a tick where it is out of reach.
    #[test]
    fn catch_is_not_deferred_out_of_the_wall_leap_zone() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let fence = fx.field.fence_at(0.0);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.f[CF].pos = Vec3::new(0.0, fence - 3.8, 0.0);
        loose_ball(&mut sim, 2.0, Vec3::new(0.0, fence - 2.99, 2.9), Vec3::new(0.0, -20.0, 0.0));
        for _ in 0..10 {
            world_tick(&mut sim);
        }
        assert!(caught(&sim), "facts: {:?}", sim.out.facts);
    }

    /// A grounder rolling up the middle line (x = 0) at 30 m/s from `y0` at the sim's time.
    fn grounder(fx: &Fixture, t: f64, y0: f64) -> BallPath {
        let r = fx.env.ball.radius;
        batted_path_dt(&fx.env, &fx.field, BallState { r: Vec3::new(0.0, y0, r), v: Vec3::new(0.0, 30.0, 0.0), w: Vec3::ZERO }, t, AGENT_DT, |_, _| (0.0, 0.0))
    }

    const SS: usize = 5;

    // A hard grounder passing 2 m to an infielder's side is his to field where it passes: from
    // a standing start he covers 1.46 m by then (run_dist with his reaction time) and needs
    // 1.3 m. The window is 0.05 s of ball travel, shorter than the scan's step, so the answer
    // must not depend on where the scan's samples fall.
    #[test]
    fn intercept_finds_a_grounder_passing_between_scan_samples() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        for phase in 0..12 {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            // The ball reaches the fielder's depth (y = 30) 0.83 s after contact in every
            // case; only the scan's phase changes.
            let dt = 0.01 * phase as f64;
            sim.t = inp.t_contact + dt;
            sim.f[SS].pos = Vec3::new(2.0, 30.0, 0.0);
            let path = grounder(&fx, sim.t, 5.0 + 30.0 * dt);
            let (t_int, at) = sim.intercept(SS, &path);
            assert!((at.y - 30.0).abs() < 1.5 && t_int - inp.t_contact < 0.95, "phase {phase}: intercept at y {:.1}, +{:.2} s", at.y, t_int - inp.t_contact);
        }
    }

    // A fielder already running keeps his speed into a replan. At 6 m/s he covers 3.27 m in
    // the 0.5 s the ball takes to reach his depth, enough for a ball 3.5 m away (2.8 m needed);
    // from a standing start he would cover 1.1 m and have to give the near point up.
    #[test]
    fn replan_counts_the_speed_the_fielder_already_has() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = inp.t_contact + 1.0;
        sim.f[SS].pos = Vec3::new(3.5, 30.0, 0.0);
        let path = grounder(&fx, sim.t, 15.0);
        sim.f[SS].speed = 6.0;
        let (_, running) = sim.intercept(SS, &path);
        assert!((running.y - 30.0).abs() < 2.0, "running fielder plans for y {:.1}", running.y);
        sim.f[SS].speed = 0.0;
        let (_, standing) = sim.intercept(SS, &path);
        assert!(standing.y > running.y + 3.0, "standing fielder plans for y {:.1}", standing.y);
        // The speed law itself: from rest it is `run_dist`; at top speed it is v·t.
        assert!((run_dist_from(0.8, 0.0, 7.9, 0.7) - run_dist(0.8, 7.9, 0.7)).abs() < 1e-12);
        assert!((run_dist_from(0.8, 7.9, 7.9, 0.7) - 7.9 * 0.8).abs() < 1e-12);
    }

    // Fielding review #3: the moment a fielder can first reach a rolling ball comes after its
    // closest pass, as he gains ground. Expected value from a 1 ms search of the same reach
    // rule (the review's scenario): standing at 2.296 s, not the late point near 4.7 s.
    #[test]
    fn intercept_finds_the_first_reachable_moment_on_a_slowing_ball() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        sim.f[SS].pos = Vec3::new(1.1, 30.0, 0.0);
        let r = fx.env.ball.radius;
        let path = batted_path_dt(&fx.env, &fx.field, BallState { r: Vec3::new(0.0, 27.4, r), v: Vec3::new(0.0, 10.0, 0.0), w: Vec3::ZERO }, 2.0, AGENT_DT, |_, _| (0.0, 0.0));
        // Independent fine search with the published rule: within 0.7 m after running
        // run_dist from rest (he has reacted long ago).
        let a = sim.f[SS].a;
        let mut first = f64::NAN;
        let mut t = 2.0;
        while t < path.t_end() {
            let b = path.at(t).0;
            if (b.x - 1.1).hypot(b.y - 30.0) - 0.7 <= run_dist(t - 2.0, a.speed, a.tau_a) {
                first = t;
                break;
            }
            t += 0.001;
        }
        let (t_int, _) = sim.intercept(SS, &path);
        assert!(first.is_finite() && (t_int - first).abs() <= 0.03, "scan {t_int:.3} s, fine search {first:.3} s");
    }

    // Fielding review #2 and #4: a fielder who has reached his spot is standing still, and a
    // running fielder's time to a resting ball counts his speed.
    #[test]
    fn stopped_fielders_have_no_speed_and_running_ones_keep_it() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        sim.f[SS].role = Role::Chase;
        sim.f[SS].speed = 7.0;
        sim.f[SS].target = sim.f[SS].pos + Vec3::new(0.05, 0.0, 0.0);
        sim.f[SS].t_int = 2.0;
        for _ in 0..30 {
            sim.move_fielders();
            sim.t += DT;
        }
        assert_eq!(sim.f[SS].speed, 0.0);
        // 6 m at 6 m/s already (top speed 7.9, τ 0.7): quicker than 6 m from rest, slower
        // than 6 m at top speed, and exactly inverse to the distance law.
        let t = run_time_from(6.0, 6.0, 7.9, 0.7);
        assert!(t < run_time(6.0, 7.9, 0.7) && t > 6.0 / 7.9);
        assert!((run_dist_from(t, 6.0, 7.9, 0.7) - 6.0).abs() < 1e-9);
        assert!((run_time_from(6.0, 0.0, 7.9, 0.7) - run_time(6.0, 7.9, 0.7)).abs() < 1e-9);
    }

    /// Set up who fields the ball where, then hand out cover and backup jobs.
    fn jobs(sim: &mut Sim, fielder: usize, at: Vec3) {
        sim.primary = fielder;
        sim.f[fielder].target = at;
        sim.cover_roles();
        sim.assign_backups();
    }

    /// The backup of `base` stands beyond it on the line of a throw from `from`.
    fn behind(sim: &Sim, i: usize, base: u8, from: Vec3) -> bool {
        let b = base_pos(base);
        let beyond = sim.f[i].target - b;
        let throw = (b - from).normalized();
        sim.f[i].role == Role::Backup && sim.f[i].backs == Some(base) && beyond.norm() > 8.0 && beyond.normalized().dot(throw) > 0.95
    }

    // Backup jobs as coaching guides give them (Spiders "Cutoff & Backup Responsibilities",
    // the owner's reference). Indices: P 0, C 1, 1B 2, 2B 3, 3B 4, SS 5, LF 6, CF 7, RF 8.
    #[test]
    fn idle_fielders_back_up_the_play() {
        let batter = |fx: &Fixture| RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 };
        // Grounder to short, nobody on: the catcher trails the batter to back up first.
        let fx = fixture(&[]);
        let inp = input(&fx, vec![batter(&fx)]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let at = Vec3::new(-9.0, 40.0, 0.0);
        jobs(&mut sim, SS, at);
        assert!(behind(&sim, 1, 1, at), "catcher: {:?} {:?} {:?}", sim.f[1].role, sim.f[1].backs, sim.f[1].target);

        // Single to left, nobody on: the center fielder backs up the left fielder, the
        // right fielder backs up the throw into second, the catcher backs up first.
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let at = Vec3::new(-40.0, 65.0, 0.0);
        jobs(&mut sim, 6, at);
        assert!(sim.f[7].role == Role::Backup && sim.f[7].backs.is_none() && sim.f[7].target.norm() > at.norm(), "CF: {:?}", sim.f[7].target);
        assert!(behind(&sim, 8, 2, at), "RF: {:?} {:?}", sim.f[8].backs, sim.f[8].target);
        assert!(sim.f[1].backs == Some(1) && sim.f[1].role == Role::Backup);

        // Single to left, runner on first: the pitcher backs up third from foul ground, the
        // right fielder still backs up second, and the catcher stays home.
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 1)]);
        let inp = input(&fx, vec![batter(&fx), RunnerIn { id: R, base: 1, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        jobs(&mut sim, 6, at);
        assert!(behind(&sim, 0, 3, at), "P: {:?} {:?}", sim.f[0].backs, sim.f[0].target);
        assert!(!Field::fair(sim.f[0].target.x, sim.f[0].target.y), "pitcher backs up third in foul ground: {:?}", sim.f[0].target);
        assert!(behind(&sim, 8, 2, at));
        assert_eq!(sim.f[1].role, Role::Cover(4));
        // Nobody has two jobs and no base has two backups.
        for base in 1..=4u8 {
            assert!(sim.f.iter().filter(|f| f.backs == Some(base)).count() <= 1);
        }

        // When the throw actually goes, the backup lines up with it.
        let thrower = Vec3::new(-30.0, 50.0, 0.0);
        sim.back_up_throw(thrower, 3);
        assert!(behind(&sim, 0, 3, thrower));
    }

    // Watch-fixes review #1–#3: the catcher goes back to cover home once a runner rounds
    // first; a backup catches an overthrow that reaches him in the air; a runner holds his
    // lead (he does not drift back to the base) until he has read the ball.
    #[test]
    fn catcher_returns_backup_catches_and_runners_hold_their_leads() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        jobs(&mut sim, 6, Vec3::new(-40.0, 65.0, 0.0));
        assert_eq!(sim.f[1].backs, Some(1));
        sim.cover_home_again();
        assert_eq!(sim.f[1].backs, Some(1), "nobody has rounded first yet");
        sim.r[0].s = 1.4 * BASE_L;
        sim.cover_home_again();
        assert_eq!(sim.f[1].role, Role::Cover(4));
        assert!((sim.f[1].target - base_pos(4)).norm() < 1e-9);

        // A throw to second sails over the receiver and comes down to the backup 13 m behind
        // the base (the review's scenario): he catches it.
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let second = base_pos(2);
        sim.t = 2.0;
        sim.f[3].role = Role::Cover(2);
        sim.f[3].pos = second;
        sim.f[0].role = Role::Backup;
        sim.f[0].backs = Some(2);
        sim.f[0].pos = second + Vec3::new(0.0, -BACKUP_M, 0.0);
        sim.f[0].target = sim.f[0].pos;
        // From deep center toward home, 4 m over the base, falling to chest height at the backup.
        let (r0, tf) = (Vec3::new(0.0, 80.0, 1.8), 1.6);
        let end = sim.f[0].pos + Vec3::new(0.0, 0.0, 1.3);
        let v0 = Vec3::new(0.0, (end.y - r0.y) / tf, (end.z - r0.z + 0.5 * G * tf * tf) / tf);
        sim.ball = Ball::Thrown { r0, v0, t0: 2.0, to_base: 2, receiver: 3 };
        for _ in 0..110 {
            sim.update_thrown();
            sim.try_captures();
            sim.t += DT;
        }
        assert!(matches!(sim.ball, Ball::Held { by: 0 }), "ball: {:?}", sim.ball);

        // A runner on first stays at his lead until the runners read the ball at +0.5 s.
        let fx = fixture(&[(R, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R, base: 1, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        for _ in 0..25 {
            sim.move_runners();
            sim.t += DT;
        }
        assert!((sim.r[1].s - lead_s(1)).abs() < 1e-9, "runner drifted to {:.2} m", sim.r[1].s - BASE_L);
    }

    // Runners take leads, and on a fly that might drop with fewer than two out they go
    // partway rather than all the way: a runner from first is never past halfway to second
    // while a deep fly is still in the air.
    #[test]
    fn runner_leads_off_and_goes_halfway_on_a_fly() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R, base: 1, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let out = simulate_live(&LiveInput { record: true, ..inp }, &mut st);
        let track = &out.actors.iter().find(|(id, _)| *id == R).unwrap().1;
        let first = base_pos(1);
        let lead = (track[0].1 - first).norm();
        assert!(lead > 3.0 && lead < 7.5, "lead {lead:.1} m");
        // The fixture's ball is a fly to center that is in the air for several seconds.
        let landed = out.facts.iter().find_map(|f| match f {
            Input::BallTouched { t, what: Touch::Ground { .. } } | Input::GloveSecured { t, .. } => Some(*t),
            _ => None,
        });
        let t_down = landed.expect("the ball comes down");
        assert!(t_down > inp.t_contact + 2.5, "fly was up for {:.1} s", t_down - inp.t_contact);
        for (t, p) in track.iter().filter(|(t, _)| *t < t_down) {
            assert!((*p - first).norm() <= 0.6 * BASE_L, "{:.1} m off first at +{:.2} s with the ball in the air", (*p - first).norm(), t - inp.t_contact);
        }
    }

    // A chaser's plan has him reaching the ball at speed, so he must get there when the plan
    // says: 6 m from rest takes `run_time(6 m)`. A fielder going to cover a base stops on it,
    // which takes longer.
    #[test]
    fn chaser_arrives_on_the_plans_schedule() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let arrive = |role: Role| {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = inp.t_contact + 1.0;
            let a = sim.f[SS].a;
            let plan = run_time(6.0, a.speed, a.tau_a);
            sim.f[SS].role = role;
            sim.f[SS].target = sim.f[SS].pos + Vec3::new(6.0, 0.0, 0.0);
            sim.f[SS].t_int = sim.t + plan;
            let t0 = sim.t;
            while (sim.f[SS].target - sim.f[SS].pos).norm() > 1e-6 {
                sim.move_fielders();
                sim.t += DT;
            }
            (sim.t - t0, plan)
        };
        let (chase, plan) = arrive(Role::Chase);
        assert!((chase - plan).abs() <= 2.0 * DT, "chaser took {chase:.3} s, plan {plan:.3} s");
        let (cover, _) = arrive(Role::Cover(2));
        assert!(cover > plan + 0.2, "a fielder stopping on a base took {cover:.3} s");
    }

    // Codex day review #5: each advance decision is a fresh look at the play. Repeated
    // decisions about the same base at the same instant must each use a new timing draw (read
    // back from the decision itself), none of them the draw of the first read at contact.
    #[test]
    fn repeated_runner_decisions_draw_fresh_noise() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R, base: 1, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 1.0;
        let mut draws = Vec::new();
        for _ in 0..4 {
            sim.runner_decide(1, 1);
            draws.push(sim.r[1].last_timing_draw);
        }
        for i in 0..4 {
            for j in 0..i {
                assert_ne!(draws[i], draws[j], "decisions {j} and {i} used the same timing draw");
            }
        }
    }

    // Review of 2026-11-11 (motions), finding 1: a backup standing between the base and the
    // receiver (who has stepped past the bag to meet the throw) must leave a catchable throw
    // to the receiver.
    #[test]
    fn backup_in_front_of_the_receiver_leaves_him_the_throw() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let second = base_pos(2);
        sim.t = 2.0;
        sim.f[3].role = Role::Cover(2);
        sim.f[3].pos = second + Vec3::new(0.0, -0.5, 0.0);
        sim.f[3].target = sim.f[3].pos;
        sim.f[0].role = Role::Backup;
        sim.f[0].backs = Some(2);
        sim.f[0].pos = second + Vec3::new(0.0, -0.2, 0.0);
        sim.f[0].target = sim.f[0].pos;
        // From deep center, arriving chest-high at the receiver.
        let (r0, tf) = (Vec3::new(0.0, 80.0, 1.8), 1.6);
        let end = sim.f[3].pos + Vec3::new(0.0, 0.0, 1.3);
        let v0 = Vec3::new(0.0, (end.y - r0.y) / tf, (end.z - r0.z + 0.5 * G * tf * tf) / tf);
        sim.ball = Ball::Thrown { r0, v0, t0: 2.0, to_base: 2, receiver: 3 };
        for _ in 0..110 {
            sim.update_thrown();
            sim.try_captures();
            sim.t += DT;
        }
        assert!(matches!(sim.ball, Ball::Held { by: 3 }), "ball: {:?}", sim.ball);
    }

    // Finding 2 of the same review: the batter's realized track starts in his box and joins
    // the first-base line; nothing downstream has to move him.
    #[test]
    fn batter_runs_out_of_his_box_onto_the_line() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let out = simulate_live(&LiveInput { record: true, ..inp }, &mut st);
        let track = &out.actors.iter().find(|(id, _)| *id == fx.st.batter).unwrap().1;
        let p0 = track[0].1;
        assert!((p0 - Vec3::new(-0.95, 0.30, 0.0)).norm() < 0.05, "starts at {p0:?}");
        // Past his first few strides he is on the line from home to first (x = y).
        let on_line: Vec<_> = track.iter().filter(|(_, p)| p.x > 4.0 && p.x < 19.0 && p.y < 19.0).collect();
        assert!(!on_line.is_empty());
        assert!(on_line.iter().all(|(_, p)| (p.x - p.y).abs() < 1e-9), "{on_line:?}");
        // And no step along the way is longer than a sprinter covers in 0.1 s.
        assert!(track.windows(2).all(|w| (w[1].1 - w[0].1).norm() <= 11.0 * (w[1].0 - w[0].0) + 1e-9));
    }
}
