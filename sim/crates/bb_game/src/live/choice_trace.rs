//! Read-only throw/carry commitments, for `examples/throw_choice_probe.rs`.
//! This sink is off by default, thread-local, and cannot feed information back to agents.
use bb_math::Vec3;
use std::cell::{Cell, RefCell};

#[derive(Clone, Debug)]
pub struct RunnerView {
    pub id: u16,
    pub start: u8,
    pub s: f64,
    pub speed: f64,
    pub goal: u8,
    /// Diagnostic audit of internal runner intent, NOT information offered to the
    /// holder. `goal` above is the holder's visible_target estimate.
    pub intended_goal: u8,
    pub out: bool,
    pub scored: bool,
    pub protected_overrun: bool,
}

/// Times are absolute play seconds. `None` means this branch did not calculate it.
/// The nearby-carry branch tests candidates in order and stops on its first success;
/// these candidates have no probability/weight/value in the actual decision code.
#[derive(Clone, Debug)]
pub struct OptionEstimate {
    pub runner: u16,
    pub base: u8,
    pub kind: &'static str,
    pub force: bool,
    pub runner_at: f64,
    pub ball_at: f64,
    pub direct_at: Option<f64>,
    pub via_at: Option<f64>,
    pub set_at: Option<f64>,
    pub receiver: Option<u16>,
    pub receiver_at: Option<f64>,
    pub p_out: Option<f64>,
    pub weight: Option<f64>,
    pub value: Option<f64>,
    pub through: bool,
    pub distance: f64,
}

#[derive(Clone, Debug)]
pub struct Choice {
    pub play: u32,
    pub t: f64,
    pub until: f64,
    pub ticks: u32,
    pub holder: u16,
    pub possession: f64,
    pub from: Vec3,
    pub outs: u8,
    pub inning: u8,
    pub batting: usize,
    pub score: [u16; 2],
    /// `near_carry` bypasses weighted evaluation; `carry` replaces a selected short
    /// throw; `return` is the no-play return to second; `throw` targets an out.
    pub action: &'static str,
    pub base: u8,
    pub through: bool,
    pub runners: Vec<RunnerView>,
    pub options: Vec<OptionEstimate>,
}

thread_local! {
    static ON: Cell<bool> = const { Cell::new(false) };
    static RECORDS: RefCell<Vec<Choice>> = const { RefCell::new(Vec::new()) };
}

pub(super) fn enabled() -> bool {
    ON.with(Cell::get)
}

pub(super) fn push(c: Choice) {
    RECORDS.with_borrow_mut(|records| {
        if let Some(last) = records.last_mut() {
            if c.action.contains("carry") && last.action == c.action && last.play == c.play && last.holder == c.holder && last.possession == c.possession && last.base == c.base {
                last.until = c.t;
                last.ticks += 1;
                return;
            }
        }
        records.push(c);
    });
}

/// Capture a synchronous operation on this thread. No RNG, environment change or
/// simulator mutation. The guard also disables and clears the sink on unwinding.
pub fn capture<T>(f: impl FnOnce() -> T) -> (T, Vec<Choice>) {
    ON.with(|on| assert!(!on.replace(true), "nested choice capture"));
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            ON.with(|on| on.set(false));
            RECORDS.with_borrow_mut(Vec::clear);
        }
    }
    let _guard = Guard;
    let result = f();
    let choices = RECORDS.with_borrow_mut(core::mem::take);
    (result, choices)
}

/// The near-carry shortcut never runs the weighted loop. For evidence only, show
/// what that loop would have estimated at the same instant, using its existing
/// observer inputs and formulas. Label these explicitly: the holder did NOT weigh
/// them. No movement, setup, random draw, or option selection happens here.
pub(super) fn skipped_throw_options(sim: &super::Sim<'_>, by: usize) -> Vec<OptionEstimate> {
    use super::*;
    let fd = &sim.f[by];
    let relay = (0..9).find(|&i| i != by && matches!(sim.f[i].role, Role::Relay(_)) && (sim.f[i].pos - fd.pos).norm() > 10.0);
    let release = |target, at: f64| {
        let su = sim.set_up_for(by, target);
        let t = at.max(fd.up_at + su.earliest_s);
        (t, THROW_EFFORT * fd.a.arm * su.speed_mult(su.hurry(t - fd.up_at)))
    };
    let direct = |base, at| {
        let (t, speed) = release(base_pos(base), at);
        t + throw_time(speed, (base_pos(base) - fd.pos).norm())
    };
    let via = |base, at| {
        let Some(i) = relay else { return f64::INFINITY };
        let rp = sim.f[i].pos;
        if (base_pos(base) - rp).norm() >= (base_pos(base) - fd.pos).norm() {
            return f64::INFINITY;
        }
        let (t, speed) = release(rp, at);
        t + throw_time(speed, (rp - fd.pos).norm()) + RELAY_TRANSFER_FRAC * sim.f[i].a.transfer + throw_time(THROW_EFFORT * sim.f[i].a.arm, (base_pos(base) - rp).norm())
    };
    sim.r
        .iter()
        .enumerate()
        .filter(|(_, r)| !r.out && !r.scored && r.overrun == OverrunPhase::None)
        .filter_map(|(k, r)| {
            let goal = visible_target(r, fd.a.throw_choice)?;
            let runner_at = sim.t + runner_eta((goal as f64 * BASE_L - r.s).abs(), r.v);
            let recv = sim.receiver_for(by, goal);
            let recv_at = sim.receiver_ready(recv, goal);
            let (d, v) = (direct(goal, sim.t), via(goal, sim.t));
            let through = v < d;
            let ball_at = d.min(v).max(recv_at) + 0.05;
            let leg = if through { sim.f[relay.unwrap()].pos } else { base_pos(goal) };
            let set = (fd.up_at + sim.set_up_for(by, leg).set_s).max(sim.t);
            let set_at = (if through { via(goal, set) } else { direct(goal, set) }).max(recv_at) + 0.05;
            let p = phi((runner_at - ball_at) / 0.12);
            let weight = sim.out_weight(k, goal);
            Some(OptionEstimate {
                runner: r.id,
                base: goal,
                kind: "unweighed_throw",
                force: sim.forced(k) && goal == r.start + 1 && r.s < goal as f64 * BASE_L,
                runner_at,
                ball_at,
                direct_at: Some(d),
                via_at: Some(v),
                set_at: Some(set_at),
                receiver: Some(sim.f[recv].id),
                receiver_at: Some(recv_at),
                p_out: Some(p),
                weight: Some(weight),
                value: Some(p * p * weight),
                through,
                distance: (base_pos(goal) - fd.pos).norm(),
            })
        })
        .collect()
}
