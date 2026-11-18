//! Opt-in, read-only runner movement trace for the timing probe.
use std::cell::{Cell, RefCell};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    Read,
    Air,
    GroundHold,
    Traffic,
    BagHold,
    OtherHold,
    Run,
}

#[derive(Clone, Copy, Debug)]
pub struct Tick {
    pub runner: u16,
    pub t: f64,
    pub s: f64,
    pub signed_mps: f64,
    pub goal: u8,
    pub cause: Cause,
    pub ball_down: bool,
}

thread_local! {
    static ON: Cell<bool> = const { Cell::new(false) };
    static TICKS: RefCell<Vec<Tick>> = const { RefCell::new(Vec::new()) };
}

pub(super) fn enabled() -> bool {
    ON.with(Cell::get)
}

pub(super) fn push(tick: Tick) {
    TICKS.with_borrow_mut(|ticks| ticks.push(tick));
}

pub fn capture<T>(f: impl FnOnce() -> T) -> (T, Vec<Tick>) {
    ON.with(|on| assert!(!on.replace(true), "nested runner capture"));
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            ON.with(|on| on.set(false));
            TICKS.with_borrow_mut(Vec::clear);
        }
    }
    let _guard = Guard;
    let result = f();
    (result, TICKS.with_borrow_mut(core::mem::take))
}
