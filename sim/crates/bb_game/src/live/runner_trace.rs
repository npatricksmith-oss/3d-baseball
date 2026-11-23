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
    /// The batted ball has visibly touched the ground or a wall.
    pub ball_down: bool,
}

/// A look from a base a runner was holding (agent-models §6.4, "Looking again from a base").
#[derive(Clone, Copy, Debug)]
pub struct BaseLook {
    pub runner: u16,
    pub t: f64,
    pub base: u8,
    /// He left for the next base.
    pub went: bool,
    /// A runner ahead had not cleared the next base: he waits on him.
    pub waiting: bool,
    /// The ball was in the air on a throw (otherwise held).
    pub ball_thrown: bool,
}

thread_local! {
    static ON: Cell<bool> = const { Cell::new(false) };
    static TICKS: RefCell<Vec<Tick>> = const { RefCell::new(Vec::new()) };
    static LOOKS: RefCell<Vec<BaseLook>> = const { RefCell::new(Vec::new()) };
}

pub(super) fn enabled() -> bool {
    ON.with(Cell::get)
}

pub(super) fn push(tick: Tick) {
    TICKS.with_borrow_mut(|ticks| ticks.push(tick));
}

pub(super) fn push_look(look: BaseLook) {
    LOOKS.with_borrow_mut(|looks| looks.push(look));
}

/// The base looks recorded during the last `capture`, taken (cleared) by this call.
pub fn take_looks() -> Vec<BaseLook> {
    LOOKS.with_borrow_mut(core::mem::take)
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
