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
const TYPICAL_ARM: f64 = 37.0;
const TYPICAL_TRANSFER: f64 = 0.75;
/// A runner's read of a batted ball, m at 30 m [E].
const RUNNER_READ_SIGMA0: f64 = 2.0;
/// Agents' internal flight step, s.
const AGENT_DT: f64 = 0.1;
const BASE_L: f64 = 27.432;
const BATTER_START: f64 = 0.30;
const PLAY_LIMIT_S: f64 = 40.0;
/// Fraction of arm strength used on throws [E].
const THROW_FRAC: f64 = 0.92;
const RECEIVE_REACH: f64 = 1.5;
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
}

pub struct LiveInput<'a> {
    pub env: &'a FlightEnv,
    pub field: &'a Field,
    pub ball: BallState,
    pub t_contact: f64,
    pub runners: Vec<RunnerIn>,
    pub fielders: [AgentId; 9],
    pub fattrs: [FielderAttrs; 9],
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
fn path_pos(s: f64) -> Vec3 {
    let s = s.clamp(0.0, 4.0 * BASE_L);
    let k = ((s / BASE_L).floor() as u8).min(3);
    let f = (s - k as f64 * BASE_L) / BASE_L;
    let a = base_pos(k);
    let b = base_pos(k + 1);
    a + (b - a) * f
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

    /// A runner's read of the ball (keyed noise per runner and decision).
    fn runner_predict(&self, k: usize, decision: u32) -> BallPath {
        let r = &self.r[k];
        let mut g = rng(self.inp.seed, self.inp.play, r.id, Purpose::RunnerPerception, 100 + decision);
        let err = [g.truncated_normal(4.0), g.truncated_normal(4.0), g.truncated_normal(4.0), g.truncated_normal(4.0)];
        let view = (self.t - self.inp.t_contact).max(0.0);
        self.predict_from(path_pos(r.s), RUNNER_READ_SIGMA0, 0.6, view, err)
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
        (t_end.max(start + run_time(d, fd.a.speed, fd.a.tau_a)), Vec3::new(rest.x, rest.y, 0.0))
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
        let mut t = self.t + 0.05;
        while t <= t_end {
            let (b, _) = pred.at(t);
            if b.z <= CATCH_MAX_Z {
                let d = m::hypot(b.x - fd.pos.x, b.y - fd.pos.y);
                let covered = run_dist(t - start, fd.a.speed, fd.a.tau_a);
                let can = |reach: f64| t >= start && (d - reach <= 0.0 || covered >= d - reach);
                // Head for the ball's line itself, not the edge of reach.
                let here = Some((t, Vec3::new(b.x, b.y, 0.0)));
                if can(REACH * 0.7) {
                    return (here, dive);
                }
                if dive.is_none() && b.z <= DIVE_MAX_Z && can(REACH + 0.6 * DIVE) {
                    dive = here;
                }
            }
            t += 0.12;
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
        // for it at once; nobody waits for a call (agent-models §5.7, ground-ball case).
        let grounder = self.f[self.primary].belief.as_ref().and_then(|p| p.first_bounce()).map(|b| m::hypot(b.1, b.2) < 25.0).unwrap_or(false);
        if grounder {
            let t_primary = self.f[self.primary].t_int;
            for i in [3usize, 4, 5] {
                let fd = &self.f[i];
                if i != self.primary && fd.t_int.is_finite() && fd.t_int <= t_primary + 0.6 && m::hypot(fd.target.x, fd.target.y) < 45.0 && fd.belief.is_some() {
                    self.f[i].role = Role::Chase;
                }
            }
            self.cover_roles_after_chasers();
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
                self.f[i].role = if i >= 6 { Role::Backup } else { Role::Idle };
                if i >= 6 {
                    // Back up the play: head toward the chase point, staying ~8 m behind it.
                    let from = Vec3::new(0.0, 0.0, 0.0);
                    let dir = (tgt - from).normalized();
                    self.f[i].target = tgt + dir * 8.0;
                }
            }
        }
    }

    fn move_fielders(&mut self) {
        for fd in self.f.iter_mut() {
            if self.t < self.inp.t_contact + fd.a.reaction {
                continue;
            }
            if fd.dove {
                if self.t < fd.ready_at {
                    continue;
                }
                fd.dove = false; // recovered
            }
            let to = Vec3::new(fd.target.x - fd.pos.x, fd.target.y - fd.pos.y, 0.0);
            let d = to.norm();
            let brake = fd.speed * fd.speed / (2.0 * 6.0);
            if d > brake + 0.05 {
                fd.speed += (fd.a.speed - fd.speed) * (DT / fd.a.tau_a);
            } else {
                fd.speed = (fd.speed - 6.0 * DT).max(0.0);
            }
            let stepd = (fd.speed * DT).min(d);
            if d > 1e-6 {
                fd.pos += to * (stepd / d);
            }
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
        let near_wall = {
            let th = m::atan2(b.x, b.y);
            self.inp.field.fence_at(th) - m::hypot(b.x, b.y) < 3.0
        };
        let zmax = if near_wall { WALL_CATCH_MAX_Z } else { CATCH_MAX_Z };
        for i in 0..9 {
            let fd = &self.f[i];
            if self.t < fd.blocked_until || self.t < self.inp.t_contact + fd.a.reaction {
                continue;
            }
            // On a throw only the receiver, and nobody else in its path, tries.
            if receiver.is_some() && receiver != Some(i) {
                continue;
            }
            if b.z < 0.0 {
                continue; // never capture a ball below the surface
            }
            let horiz = m::hypot(b.x - fd.pos.x, b.y - fd.pos.y);
            let reach = if receiver.is_some() { RECEIVE_REACH } else { REACH };
            let standing = horiz <= reach && b.z <= zmax;
            // The glove meets the ball where it passes closest (the capture instant,
            // agent-models §5.4), not where it first enters reach: wait while it still comes
            // closer within reach and catchable height, and does not reach the ground first.
            if standing {
                let (bn, _) = self.ball_at(self.t + DT);
                let lands = match &self.ball {
                    Ball::Free { path, .. } => path.bounces.iter().any(|x| x.0 > self.t && x.0 <= self.t + DT),
                    _ => false,
                };
                if !lands && m::hypot(bn.x - fd.pos.x, bn.y - fd.pos.y) < horiz && bn.z <= zmax && bn.z >= 0.0 {
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
                    fd.ready_at = self.t + 0.25;
                }
                let id = fd.id;
                self.ball = Ball::Held { by: i };
                self.secured_at = self.t;
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
    fn ball_time_to(&self, k: usize, b: u8) -> f64 {
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
                let pred = self.runner_predict(k, b as u32);
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
        let r = &self.r[k];
        let t_run = run_time(BASE_L, r.vmax, 1.0) * if r.v > 1.0 { 0.75 } else { 1.0 };
        let t_ball = self.ball_time_to(k, next);
        let mut rr = rng(self.inp.seed, self.inp.play, r.id, Purpose::RunnerPerception, next as u32);
        let noise = 0.15 * rr.truncated_normal(4.0);
        let must = forced && at_base == r.start && !self.caught();
        // A runner may only take a base once every runner ahead of him has visibly cleared it;
        // until then he waits at his base.
        let blocked = !must && self.base_uncleared_ahead(k, next);
        let go = !blocked && (must || t_ball - (self.t + t_run) + noise > 0.35);
        let r = &mut self.r[k];
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
            // Tag-up: hold the base until the catch, then decide.
            if self.r[k].tag_up {
                if self.caught() {
                    self.r[k].tag_up = false;
                    let b = self.r[k].last_base;
                    self.emit(Input::BaseTouched { t: self.t, agent: self.r[k].id, base: b, holding_ball: false });
                    self.runner_decide(k, b);
                } else if !matches!(self.ball, Ball::Free { .. }) || self.out.facts.iter().any(|f| matches!(f, Input::BallTouched { what: Touch::Ground { .. }, .. })) {
                    // The ball dropped in: run.
                    self.r[k].tag_up = false;
                    let b = self.r[k].last_base;
                    self.runner_decide(k, b);
                } else {
                    continue;
                }
            }
            // Waiting at a base: decide once the runner ahead has cleared the next one.
            if self.r[k].waiting && (self.r[k].s - self.r[k].goal as f64 * BASE_L).abs() < 1e-6 {
                let b = self.r[k].goal;
                if !self.base_uncleared_ahead(k, b + 1) {
                    self.runner_decide(k, b);
                }
            }
            // A caught fly: a runner who left his base goes back to retouch it.
            if self.caught() && !self.r[k].tag_up && !self.r[k].retouch && self.r[k].start > 0 && self.r[k].s > self.r[k].start as f64 * BASE_L + 1e-6 {
                self.r[k].retouch = true;
                self.r[k].goal = self.r[k].start;
            }
            // Heading for a base a runner ahead has stopped at (or is retreating to): go back,
            // unless this is a forced advance (then the runner ahead is forced too).
            let g = self.r[k].goal;
            let forced_here = g == self.r[k].start + 1 && self.forced(k);
            if g as f64 * BASE_L > self.r[k].s && self.base_held_ahead(k, g) && !forced_here {
                self.r[k].goal = (self.r[k].s / BASE_L).floor() as u8;
            }
            let goal_s = self.r[k].goal as f64 * BASE_L;
            let r = &mut self.r[k];
            let dir = (goal_s - r.s).signum();
            if (goal_s - r.s).abs() < 1e-6 {
                r.v = 0.0;
                continue;
            }
            if r.v == 0.0 {
                r.moving_since = self.t;
            }
            r.v += (r.vmax - r.v) * (DT / 1.0);
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
            if !on_base && (path_pos(r.s) - fd.pos).norm() < 1.2 {
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
            let t_runner = (goal as f64 * BASE_L - r.s).abs() / r.v.max(5.0);
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
            let t_runner = self.t + (goal as f64 * BASE_L - r.s).abs() / r.v.max(5.0);
            let d = (base_pos(goal) - fd.pos).norm();
            // The ball beats the runner only once someone is at the bag to take it.
            let recv = self.receiver_for(by, goal);
            let t_ball = (self.t + d / (THROW_FRAC * fd.a.arm)).max(self.receiver_ready(recv, goal)) + 0.05;
            let p_out = phi((t_runner - t_ball) / 0.12);
            let w = [0.9, 1.0, 1.1, 1.3][(goal as usize).saturating_sub(1).min(3)];
            if p_out > 0.3 && best.map(|b| p_out * w > b.0).unwrap_or(true) {
                best = Some((p_out * w, goal));
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
        self.f[receiver].target = tgt;
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

    fn record(&mut self) {
        let (b, _) = self.ball_at(self.t);
        self.out.ball_track.push((self.t, b));
        if ((self.t - self.inp.t_contact) / 0.1).fract() < DT / 0.1 {
            for (i, fd) in self.f.iter().enumerate() {
                self.out.actors[i].1.push((self.t, fd.pos));
            }
            for (k, r) in self.r.iter().enumerate() {
                if !r.out {
                    self.out.actors[9 + k].1.push((self.t, path_pos(r.s)));
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

fn simulate_live_inner(inp: &LiveInput, st: &mut RulesState) -> LiveOutput {
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
            }
        })
        .collect();
    let r: Vec<Runner> = inp
        .runners
        .iter()
        .map(|x| Runner {
            id: x.id,
            start: x.base,
            s: x.base as f64 * BASE_L,
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
        })
        .collect();
    let mut actors: Vec<(AgentId, Vec<(f64, Vec3)>)> = inp.fielders.iter().map(|&id| (id, Vec::new())).collect();
    actors.extend(inp.runners.iter().map(|x| (x.id, Vec::new())));
    let mut sim = Sim {
        inp,
        st,
        t: t_c,
        ball: Ball::Free { path: path.clone(), next_touch: 0, next_wall: 0, settled_sent: false },
        f,
        r,
        primary: 0,
        out: LiveOutput { facts: Vec::new(), events: Vec::new(), path, ball_track: Vec::new(), actors },
        bounce_k: 1000,
        done: false,
        last_fact_t: t_c,
        secured_at: f64::INFINITY,
        catch_made: false,
    };
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
                // Each runner reads the ball himself: if a visible fielder can get under it
                // before it lands, he expects a catch and tags up.
                let likely_catch = airborne && sim.r[k].start > 0 && {
                    let pred = sim.runner_predict(k, 0);
                    let land = pred.first_bounce().map(|x| x.0).unwrap_or(pred.t_end());
                    sim.believed_field_time(&pred).0 <= land + 0.3
                };
                if sim.r[k].start == 0 {
                    sim.r[k].goal = 1;
                } else if likely_catch {
                    sim.r[k].tag_up = true;
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
            sim.record();
        }
        prof::TICKS.with(|c| c.set(c.get() + 1));
        sim.t += DT;
    }
    sim.out
}
