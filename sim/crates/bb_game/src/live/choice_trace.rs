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
/// All rows now come from the same actual carry/throw comparison; no shadow formulas.
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
    pub second_out_p: Option<f64>,
    pub second_ball_at: Option<f64>,
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
    /// `carry` and `throw` are weighed out attempts; `return` is no-play control.
    /// The original `near_carry` shortcut and its shadow estimates are retired.
    pub action: &'static str,
    pub base: u8,
    pub through: bool,
    pub runners: Vec<RunnerView>,
    pub selected_option: Option<usize>,
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
            if c.action.contains("carry")
                && last.action == c.action
                && last.play == c.play
                && last.holder == c.holder
                && last.possession == c.possession
                && last.base == c.base
                && last.outs == c.outs
                && last.selected_option.and_then(|i| last.options.get(i)).map(|o| o.runner) == c.selected_option.and_then(|i| c.options.get(i)).map(|o| o.runner)
            {
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
