//! Live-ball plays. The world advances in fixed ticks. Fielders and runners decide from
//! their observations, and the rules engine judges each fact as it happens.
//!
//! Fielders project observed ball states using their own physics (agent-models §5.1).
//! The defense uses at most one cutoff or relay man, and coverage follows the assignments
//! in agent-models §5.7.

use bb_math::{KeyedRng, Mat3, Purpose, RngKey, Vec3, m};
use bb_physics::ground::{BallPath, Field, SegmentEnd, WallContact, batted_path, batted_path_dt, over_the_wall, wall_contact};
use bb_physics::throw::{THROW_MAX_ELEV, THROW_SPIN_RPM, ThrowTable, cut_elev, fly_throw, fly_throw_in, solve_throw, throw_state};
use bb_physics::{BallState, FlightEnv, Trajectory};
use bb_rules::{AgentId, Event, Input, OutKind, PlayState, ReleaseKind, RulesState, RunnerPosition, Touch, step};

use bb_agents::motor::{ThrowNoise, execute_throw};
use bb_agents::throwing::{self, SetUp};

use crate::play::{FielderAttrs, RunnerIn, base_pos, home_positions_for, run_dist, run_time};

/// Opt-in, read-only A4 evidence. Never consulted by the option evaluation.
pub mod choice_trace;
pub mod runner_trace;
mod holder;

const DT: f64 = 1.0 / 60.0;
const REPLAN_S: f64 = 0.4;
pub(crate) const REACH: f64 = 1.0;
const DIVE: f64 = 1.5;
const DIVE_MAX_Z: f64 = 1.2;
pub const DIVE_GETUP_S: f64 = 1.5;
/// A dive's lunge: the body travels until the glove is at the ball, at most [`DIVE`], over
/// this long [E] (a layout dive covers 1.5 m in about 0.3 s), then lies there.
pub const DIVE_LUNGE_S: f64 = 0.3;
/// A stretch on his feet: how far past his reach a fielder on the move can get his glove
/// without going to the ground, m [E], and what it costs him before he can throw (he is
/// off balance: agent-models §5.5), s [E].
const STRETCH: f64 = 0.4;
const STRETCH_COST_S: f64 = 0.3;
/// How much harder the glove is to place the further he is laid out: added to the glove σ
/// multiplier at full extension [E] (agent-models §5.4).
const K_EXTEND: f64 = 1.5;
/// An outfielder has nobody behind him: he leaves his feet only when he gives himself at
/// least this chance of the ball [E]; otherwise he plays it on his feet (agent-models §5.3).
const OF_DIVE_MIN_P: f64 = 0.5;
/// The pitcher is finishing his delivery when the ball is hit: he cannot lay out until this
/// long after contact, s [E].
const PITCHER_RECOVER_S: f64 = 0.7;
/// How often a chaser near the ball asks himself whether he must dive, s, and how near, m.
const DIVE_READ_S: f64 = 0.1;
const DIVE_READ_M: f64 = 15.0;
/// Points checked along a dive's lunge against the wall, and the body's half-width kept off
/// it, m [E].
const LUNGE_CHECKS: usize = 15;
const BODY_R: f64 = 0.3;
/// The farthest a fielder's glove gets past a wall's plane, m: his body stops [`BODY_R`] inside
/// it and his arm reaches [`REACH`] from the body's centre (he can lean over a rail an arm's
/// length; he cannot stretch his body through it). The one over-the-wall limit, for plans
/// and for every kind of capture (physics-spec §9.5, agent-models §5.3).
pub const OVER_WALL_REACH_M: f64 = REACH - BODY_R;
/// A catch below this height is a shoestring catch: a close call for the umpire, m [E].
const LOW_CATCH_Z: f64 = 0.4;
pub(crate) const CATCH_MAX_Z: f64 = 2.4;
/// Leaping reach at the wall [E].
const WALL_CATCH_MAX_Z: f64 = 3.0;
/// Distance inside the wall where a fielder can use the leaping ceiling, m [E].
const WALL_CATCH_ZONE_M: f64 = 3.0;
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
/// Time scale for a runner's ball read to sharpen, s [E].
const RUNNER_READ_TAU_S: f64 = 0.6;
/// Typical contact point used by a runner as the plate cue, m [E].
const RUNNER_PLATE_PRIOR: Vec3 = Vec3::new(0.0, 0.5, 0.9);
/// Agents' internal flight step, s.
const AGENT_DT: f64 = 0.1;
const BASE_L: f64 = 27.432;
/// From contact to the batter's first step out of the box, s [E]: he finishes his swing and
/// drops the bat. Fitted with the league τ and the boxes' real distances to first (27.98 m
/// right-handed, 26.58 m left-handed) so the average runner's home to first is 4.37 s
/// right-handed and 4.20 s left-handed, mean 4.29 s (2016 means 4.30 s and 4.26 s; the scouting
/// scale's average is 4.3 and 4.2). Was 0.25 s with both charged 27.43 m.
const BATTER_START: f64 = 0.20;
/// The league-typical time constant of a runner's acceleration, s: what a fielder assumes of
/// a runner (he does not know the runner's own). A runner's legs and his own judgment use
/// his own `Runner::tau`.
const RUNNER_TAU: f64 = bb_agents::running::LEAGUE_TAU_S;
/// A batter-runner who stops at first runs through the bag down the line and brakes at this
/// rate, m/s² [E] (agent-models §6.2): from 8.2 m/s he pulls up 8.4 m (28 ft) past the bag.
const OVERRUN_DECEL: f64 = 4.0;
/// The speed he walks back to first at after an overrun, m/s [E].
const OVERRUN_RETURN_MPS: f64 = 2.5;
/// Runner positions go to the rules engine every this many ticks (30 Hz, rules-spec §2
/// `RunnerPosition`).
const SAMPLE_EVERY_TICKS: u64 = 2;
/// On a ball in the air that may or may not be caught, with fewer than two out, a runner
/// goes this far past his lead and waits to see it land or be caught, m [E].
const HALFWAY_M: f64 = 5.0;
/// A runner's belief of how long a fielder who takes a throw at one base needs before his
/// own throw to another leaves, s [E].
const RELAY_READ_S: f64 = 0.6;
/// A runner's belief of what a cut costs: how much later than a straight throw the ball
/// reaches his base when the throw was sent to another base and the cutoff man takes it on
/// the way and turns, s [E]: the relay man's exchange (`throwing::RELAY_S_50`, 0.75 s) and the
/// longer way round (0.3 s). It was 0.9 s with the relay man's 0.6 s, before his exchange was
/// sourced (agent-models §5.5).
const CUT_EXTRA_S: f64 = throwing::RELAY_S_50 + 0.3;
/// How far ahead of him, in time at the plate, a runner going home must be for the runner
/// behind to follow him in, s [E].
const FOLLOW_GAP_S: f64 = 0.3;
/// From contact to a runner's first look at the ball, s [E]: his read of where it is going
/// (no published value; Statcast's "first step" is not released as a league figure).
const RUNNER_READ_DELAY_S: f64 = 0.5;
/// Reaction to a public catch or no-catch ruling, s [E]: the same quarter second as the
/// two-out contact cue (`RUNNER_CONTACT_REACT_S`).
const RUNNER_CATCH_CALL_REACT_S: f64 = 0.25;
/// Uncertainty in a runner's estimate of where a fielder's glove can get to, m [E, unsourced:
/// about a glove's width]. His doubts about the fielder's speed, the ball's curve and his
/// own look come on top (agent-models §6.3).
const RUNNER_CATCH_SIGMA_M: f64 = 0.30;
/// Where a runner weighs the reads his own error could have given him, in σ of each error
/// axis: √n for the unscented transform with n = 4 axes and κ = 0 [E].
const RUNNER_CATCH_READ_POINT: f64 = 2.0;
/// The spread of fielders' top speed about the typical speed a runner assumes, m/s [E]:
/// Statcast sprint speeds spread about 1.5 ft/s about the league's 27 ft/s.
const FIELDER_SPEED_SPREAD_MPS: f64 = 0.45;
/// Confidence needed to commit to either outcome on the initial catch read [E]. Below
/// 1 − this he goes: roughly the run-value break-even of a full start against a double-off
/// from first (12% with none out, 18% with one; agent-models §6.3). The return side mirrors
/// it from first and third (checked against MLB tag-ups 2026-11-19); from second it is
/// `RUNNER_TAG_FROM_SECOND`.
const RUNNER_CATCH_CONFIDENCE: f64 = 0.80;
/// The catch score at which a runner on second goes back to tag up, by the outs before the
/// catch: none, one (agent-models §6.3, "What he does") [E].
/// - None out, 50% [fitted 2026-11-19: docs/calibration/2026-11-19-fly-read.md §3]. At 80% a
///   runner on second held partway on nearly every outfield fly and never tagged to third,
///   where major-league runners advance on 37% of outfield catches with none out (Retrosheet
///   2024). Run values agree: a tag that gains third is worth 0.27 runs (`RUN_EXPECTANCY`
///   0.71 → 0.98, one out), and on a ball that drops he loses little from the bag (scoring
///   rather than stopping at third is worth 0.08 with the batter on first), so going back pays
///   from about even odds.
/// - One out, 60% [fitted 2026-11-19, fly-read note §4]. A tag that gains third is worth only
///   0.05 runs (0.33 → 0.38, two out), while a ball that drops can cost a runner back on the
///   bag the run he scores from halfway (0.35: 1.54 against 1.19), so the one-out value sits
///   above the none-out one (Opus review 2026-11-19, P2-1: 50% with one out stranded runners
///   on drops). Swept at 60/65/70/80% on 100 games: from 80% to 60% the one-out tag-advance
///   rises from 1% to 10% of outfield catches (major-league runners: 26%), and the runners it
///   sends back on balls that drop scored no less (they stop at third from a hold too), with
///   second to home on singles unchanged.
const RUNNER_TAG_FROM_SECOND: [f64; 2] = [0.50, 0.60];
/// A runner who went at his first look, or holds partway, keeps looking at a ball still in the
/// air this often, s
/// (agent-models §6.3, the in-flight relook) [E, unsourced: the first look's own read time,
/// `RUNNER_READ_DELAY_S`].
const RUNNER_FLY_LOOK_S: f64 = 0.5;
/// Sub-indices of the runner-perception stream for those looks: this base plus the look's
/// number (data-formats §2.3).
const FLY_LOOK_SUB: u32 = 1 << 21;
/// From contact to a two-out runner's break from shuffle speed into a full run, s [E].
const RUNNER_CONTACT_REACT_S: f64 = 0.25;
/// The error of a runner's read of a race, one draw per look, s [E] (agent-models §6.3).
const RUNNER_TIMING_SIGMA_S: f64 = 0.15;
const PLAY_LIMIT_S: f64 = 40.0;
/// Fraction of arm strength used on throws [E].
/// Release speed of a throw as a fraction of the fielder's arm strength [E]. Throws fly with
/// drag and backspin (agent-models §5.5); before that, a vacuum throw at 0.92 of the arm stood
/// in for the speed drag takes off. Every throw is full effort until speed-for-accuracy is
/// modelled.
const THROW_EFFORT: f64 = 1.0;
/// A holder with at least this much time in hand on the runner, by his own judgment, gets
/// set before he throws; with less he lets it go early, as early as keeps this much, s [E]
/// (about two of his timing σ, agent-models §5.5).
const SAFE_MARGIN_S: f64 = 0.25;
/// Hurrying must buy at least this much to be worth it, s.
const HURRY_GAIN_MIN_S: f64 = 0.01;
/// A fielder running, or a ball coming, faster than this sets which way he faces as he
/// takes it, m/s [E].
const FACE_MIN_MPS: f64 = 3.0;
/// The same for a pivot man coming onto the bag: he brakes onto it (√(2·BRAKE·0.6 m) = 2.39 m/s
/// at the touch radius), under the running threshold above, and still faces the way he is
/// moving (agent-models §5.5, Gemini pivot review P2-3): a walk's pace, m/s [E].
const PIVOT_FACE_MIN_MPS: f64 = 1.0;
/// Release and catch heights of a throw, m.
const THROW_RELEASE_Z: f64 = 1.8;
const THROW_CATCH_Z: f64 = 1.3;
/// How far to his throwing-arm side of his body's centre a fielder lets go of a throw, m [E]
/// (a three-quarter arm: the hand is about a forearm outside the shoulder).
pub const THROW_ARM_OFFSET_M: f64 = 0.35;

/// Where a fielder standing at `pos` releases a throw toward `tgt`: at release height, off
/// to his throwing-arm side. He faces his target, so a right-hander's hand is to the right
/// of the line to it and a left-hander's to the left.
pub fn throw_release_point(pos: Vec3, tgt: Vec3, throws: bb_observe::Hand) -> Vec3 {
    let h = Vec3::new(tgt.x - pos.x, tgt.y - pos.y, 0.0);
    let n = h.norm();
    // The right of a man facing along h (unit): h × Z.
    let right = if n > 1e-9 { Vec3::new(h.y / n, -h.x / n, 0.0) } else { Vec3::ZERO };
    let side = if throws == bb_observe::Hand::Right { 1.0 } else { -1.0 };
    Vec3::new(pos.x, pos.y, pos.z + THROW_RELEASE_Z) + right * (side * THROW_ARM_OFFSET_M)
}
/// How often a receiver re-reads a throw coming to him, s [E].
const RECEIVE_REPLAN_S: f64 = 0.25;
/// Lowest a receiver takes a throw in the air (a scoop), m [E].
const THROW_SCOOP_Z: f64 = 0.15;
/// Step of a receiver's read of a throw on from where it comes down, s.
const THROW_HOP_DT: f64 = 0.01;
/// A receiver at a base steps in up to this far to take a throw in the air; one coming down
/// farther short he takes on the hop (agent-models §5.5), m [E].
const STEP_IN_M: f64 = 3.0;

/// How long a throw takes, as a fielder or runner believes it: tabulated once from the throw
/// physics in a reference atmosphere (a learned belief, not the world's air).
fn throw_table() -> &'static ThrowTable {
    static T: std::sync::OnceLock<ThrowTable> = std::sync::OnceLock::new();
    T.get_or_init(|| ThrowTable::build(&FlightEnv::default_env(), THROW_RELEASE_Z, THROW_CATCH_Z))
}

/// Believed flight time of a throw at release speed `v` over horizontal distance `d`
/// (infinite beyond the arm's reach on the fly).
fn throw_time(v: f64, d: f64) -> f64 {
    throw_table().time(v, d)
}

/// The cutoff man's head: an outfield throw with a cutoff man lined up passes him no higher,
/// so that he can take it (agent-models §5.5, the catch sets up the throw; owner's Q32: "low,
/// at his head or chest") [E], m.
const CUT_HEIGHT_M: f64 = 2.0;

/// A cutoff man is in a throw when he has the job for that base and stands within this of
/// its line, m [E]: he keeps to the line as it moves (`update_relay`).
const CUT_LINE_M: f64 = 5.0;

/// Where the relay man stands on a deep ball's throw of length `d` to a base: this far short
/// of the base, m (`Sim::relay_spot`): the outfielder's throw to him is `RELAY_OF_THROW_M`.
fn cut_spot(d: f64) -> f64 {
    (d - RELAY_OF_THROW_M).max(CUTOFF_M).min(0.5 * d)
}

/// Where the cutoff man stands for a throw through him (agent-models §5.5; owner's
/// docs/baseball-throw.md): `CUTOFF_M` (about 45 ft) short of the base, or halfway on a short
/// throw, so that a throw passing him at his head carries to the base on the fly or one hop.
fn through_spot(d: f64) -> f64 {
    CUTOFF_M.min(0.5 * d)
}

/// A man lined up in a throw, as someone on the field sees him (agent-models §5.5): none; the
/// cutoff man of a throw through (`through_spot`); or the relay man of a deep ball (`cut_spot`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cut {
    No,
    Through,
    Relay,
}

impl Cut {
    /// Which spot a man `short` metres short of the base on a throw of length `d` is at: the
    /// nearer of the two.
    fn of(short: f64, d: f64) -> Self {
        if short <= 0.5 * (through_spot(d) + cut_spot(d)) { Cut::Through } else { Cut::Relay }
    }
}

/// How long a cuttable throw takes, as fielders and runners believe it: tabulated once from
/// the throw and bounce physics in a reference atmosphere and park, for each of the two spots
/// the man in it can stand at (a learned belief, like [`throw_table`]; `ThrowTable::build_cut`).
fn cut_table(cut: Cut) -> &'static ThrowTable {
    static THROUGH: std::sync::OnceLock<ThrowTable> = std::sync::OnceLock::new();
    static RELAY: std::sync::OnceLock<ThrowTable> = std::sync::OnceLock::new();
    let build = |spot: fn(f64) -> f64| ThrowTable::build_cut(&FlightEnv::default_env(), &Field::default(), THROW_RELEASE_Z, THROW_CATCH_Z, CUT_HEIGHT_M, spot);
    if cut == Cut::Relay { RELAY.get_or_init(|| build(cut_spot)) } else { THROUGH.get_or_init(|| build(through_spot)) }
}

/// Believed time for a throw at release speed `v` to go `d` metres to a base: the cuttable
/// throw when a man stands in it (`cut`), else the plain one.
fn direct_time(v: f64, d: f64, cut: Cut) -> f64 {
    if cut == Cut::No { throw_time(v, d) } else { cut_table(cut).time(v, d) }
}

/// The expected time a receiver loses to a throw `len` metres long off its line by direction
/// σ `sigma` (rad): the miss X ~ N(0, len·σ), and he runs from a stand at a league-typical
/// speed whatever of it is beyond his receiving reach. E[run_time((|X| − reach)⁺)] by the
/// midpoint rule over [reach, reach + 6 len·σ] (agent-models §5.5, the route: accuracy)
/// [belief]. (Costing the run of the mean excess, as first built, overstated it about twice:
/// a run from a stand is concave in distance; Opus catch-throw review P3-2.)
fn miss_cost_s(len: f64, sigma: f64) -> f64 {
    let s = len * sigma;
    if s < 1e-9 {
        return 0.0;
    }
    const N: usize = 48;
    let h = 6.0 * s / N as f64;
    let norm = 1.0 / (s * m::sqrt(2.0 * core::f64::consts::PI));
    (0..N)
        .map(|k| {
            let x = RECEIVE_REACH + (k as f64 + 0.5) * h;
            2.0 * norm * m::exp(-0.5 * (x / s) * (x / s)) * run_time(x - RECEIVE_REACH, TYPICAL_SPEED, 0.9) * h
        })
        .sum()
}

/// A runner's read of how long the man holding the ball needs to get it `d` m by throwing:
/// his read of that fielder's arm, relayed through a man halfway (a league-typical arm and
/// relay exchange, `throwing::RELAY_S_50`) when that is quicker or the throw cannot carry
/// (agent-models §5.5, §6.4).
fn read_throw_time(arm: f64, d: f64, cut: Cut) -> f64 {
    let v = THROW_EFFORT * arm;
    direct_time(v, d, cut).min(throw_time(v, 0.5 * d) + throwing::RELAY_S_50 + throw_time(THROW_EFFORT * TYPICAL_ARM, 0.5 * d))
}

/// A runner's read of a fielder's arm is his position's grade-50 arm (scouting-level
/// knowledge) off by this σ, m/s [E, unsourced]: half the population spread of arm grades
/// (`ARM_STRENGTH_SIGMA_MPS` 2.2), so he knows a strong arm from a weak one only roughly.
const ARM_READ_SIGMA_MPS: f64 = 1.1;
/// Sub-indices of the runner-perception stream used for his read of the holder's arm: this
/// base plus the fielder's position index (far above the per-decision subs 2n, 2n + 1).
const ARM_READ_SUB: u32 = 1 << 20;

/// A runner's estimate of how long the defense needs to get the ball over horizontal
/// distance `d` by throwing: a league-typical arm, relayed through a man halfway (the league's
/// relay exchange, `throwing::RELAY_S_50`) when that is quicker or the throw cannot carry.
fn typical_throw_time(d: f64, cut: Cut) -> f64 {
    let v = THROW_EFFORT * TYPICAL_ARM;
    direct_time(v, d, cut).min(2.0 * throw_time(v, 0.5 * d) + throwing::RELAY_S_50)
}

/// A runner's look at a loose ball (his first look and every relook): noisy motion,
/// viewpoint, how long he has watched it since contact, and the public contact clock
/// while no ground, wall or glove touch has been seen.
#[derive(Clone, Copy)]
struct ContactLook {
    motion: (Vec3, Vec3),
    from: Vec3,
    t: f64,
    view: f64,
    contact_since: Option<f64>,
}

impl ContactLook {
    fn scales(self, motion: (Vec3, Vec3)) -> (Vec3, f64, f64) {
        let to = Vec3::new(motion.0.x - self.from.x, motion.0.y - self.from.y, 0.0);
        let plate = self.contact_since.map(|s| (s, RUNNER_PLATE_PRIOR));
        let shrink = read_shrink(motion, self.view, RUNNER_READ_TAU_S, plate);
        let sp = RUNNER_READ_SIGMA0 * (to.norm().max(1.0) / READ_REF_DIST) * shrink;
        let sv = READ_VEL_FRAC * motion.1.norm() * shrink;
        (to.normalized(), sp, sv)
    }

    fn project(self, field: &Field, err: [f64; 4]) -> BallPath {
        let env = reference_env();
        let (los, sp, sv) = self.scales(self.motion);
        let (mut r, v) = noisy_ball_motion(self.motion, los, sp, sv, err);
        let (env, w) = believed_flight(env, v, self.contact_since);
        if let Some(s) = self.contact_since.filter(|&s| s > 0.0) {
            let (gain, _, _) = plate_weighing(los, sp, sv, s);
            r += gain * (position_from_plate(&env, RUNNER_PLATE_PRIOR, v, w, s) - r);
        }
        batted_path_dt(&env, field, BallState { r, v, w }, self.t, AGENT_DT, |_, _| (0.0, 0.0))
    }
}

/// One observed path's best catch, with its time and appeal origin paired.
#[derive(Clone, Copy, Debug, PartialEq)]
struct CatchRead {
    score: f64,
    t: f64,
    at: Vec3,
}

/// What the runner saw of the holder receiving or fielding the ball.
#[derive(Clone, Copy, Debug, PartialEq)]
struct HolderLook {
    pos: Vec3,
    secured: Option<f64>,
    heading: Option<Vec3>,
    motion: Vec3,
    received_throw: bool,
    /// He received it as the cutoff man, by the job he visibly showed (agent-models §6.4).
    shown_relay: bool,
    stretched: bool,
    lunge_at: Option<f64>,
}

impl HolderLook {
    fn release_time(self, now: f64, base: u8, mean: FielderAttrs, hand: bb_observe::Hand) -> f64 {
        let Some(secure) = self.secured else { return now };
        let rise = self.lunge_at.map(|t| t + DIVE_GETUP_S).filter(|&feet| feet > secure);
        let from_feet = rise.unwrap_or(secure + if self.stretched { STRETCH_COST_S } else { 0.0 });
        let near = |b| (self.pos - base_pos(b)).norm() <= RECEIVE_REACH;
        // A relay, as he can see it: the man was visibly out as the cutoff man, or took it
        // farther from every bag than a base receiver goes (review P3-1, 2026-11-19). Any
        // other reception is a feed: a receiver at a bag or pulled a few metres off it.
        let relay_seen = self.received_throw && (self.shown_relay || (1..=4).all(|b| (self.pos - base_pos(b)).norm() > RELAY_SEEN_M));
        let feed = self.received_throw && !relay_seen;
        let pivot = feed && near(2) && base == 1;
        let transfer = if pivot {
            throwing::pivot_s(mean.transfer)
        } else if relay_seen {
            // The relay man's exchange, his turn included (agent-models §5.5, §6.4).
            throwing::relay_s(mean.transfer)
        } else if feed {
            RELAY_TRANSFER_FRAC * mean.transfer
        } else {
            mean.transfer
        };
        let to = base_pos(base) - self.pos;
        let turn = if !relay_seen {
            self.heading.map_or(0.0, |f| throwing::turn_extra_s(f, to, hand))
        } else {
            // A relay normally turns through the reception (inside `relay_s`).
            0.0
        };
        let footwork = throwing::footwork_s(self.motion, to, rise.is_some());
        (from_feet + SetUp::new(transfer, turn, footwork).set_s).max(now)
    }
}

/// A runner's read of a throw in the air: his own prediction of its flight from what he sees
/// of it now (position and velocity), a typical throw's backspin and the reference
/// atmosphere's drag, and one bounce if it comes down short (agent-models §6.4). Never the
/// throw's true spin, destination, receiver or future path.
struct ThrownRead {
    /// The base he takes it to be going to: the one nearest where it comes to catching height.
    dest: u8,
    /// When it gets there.
    t_arrive: f64,
    /// His predicted track up to then: time and position.
    track: Vec<(f64, Vec3)>,
}

impl ThrownRead {
    /// The runner times a direct delivery, a relay, or a catch along the way.
    fn time_to(&self, base: u8) -> f64 {
        let est = if self.dest == base {
            self.t_arrive
        } else {
            self.t_arrive + RELAY_READ_S + typical_throw_time((base_pos(base) - base_pos(self.dest)).norm(), Cut::No)
        };
        self.pass_time(base).map_or(est, |pass| est.min(pass))
    }

    /// When the throw can be caught at base `b` on its way, if its predicted track passes
    /// within `PASS_REACH_M` of the bag, at its closest, no higher than `PASS_MAX_Z` and not
    /// underground: the time of that closest pass plus `PASS_CATCH_S`.
    fn pass_time(&self, b: u8) -> Option<f64> {
        let bp = base_pos(b);
        let (t, q, d) = self.track.iter().map(|&(t, q)| (t, q, m::hypot(q.x - bp.x, q.y - bp.y))).min_by(|x, y| x.2.total_cmp(&y.2))?;
        (d <= PASS_REACH_M && (0.0..=PASS_MAX_Z).contains(&q.z)).then_some(t + PASS_CATCH_S)
    }
}

/// The atmosphere a runner believes in when he predicts a throw (a reference, not the
/// world's air).
fn reference_env() -> &'static FlightEnv {
    static E: std::sync::OnceLock<FlightEnv> = std::sync::OnceLock::new();
    E.get_or_init(FlightEnv::default_env)
}

const RECEIVE_REACH: f64 = 1.5;
/// A runner reads a reception at least this far from every bag as a relay, whatever job the
/// man showed, m [E]: farther than a base receiver goes off his bag for a throw (they took
/// throws 1.5–6 m off it in 100 traced games; Opus relay review P3-1, 2026-11-19).
const RELAY_SEEN_M: f64 = 6.0;
/// How far behind a base its backup stands, on the line of the throw, m [E] (coaching
/// guides give 15–18 m for an outfielder; foul ground allows less).
const BACKUP_M: f64 = 13.0;
/// A runner's primary lead at pitch release, m [E]. Index = base (none at home).
const LEAD_M: [f64; 4] = [0.0, 5.5, 6.5, 4.5];

/// Where a runner on `base` starts his secondary lead, as base-path distance from home.
pub(crate) fn lead_s(base: u8) -> f64 {
    base as f64 * BASE_L + LEAD_M[(base as usize).min(3)]
}
/// The secondary lead at contact (agent-models §6.1), independent of whether a force develops.
#[derive(Clone, Copy, Debug)]
pub struct SecondaryLead {
    /// Distance beyond the primary lead, m [E].
    pub extra_m: f64,
    /// Forward speed on contact, m/s [E].
    pub speed_mps: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct RunnerStartParams {
    pub fewer_than_two: SecondaryLead,
    pub two_out: SecondaryLead,
}

impl Default for RunnerStartParams {
    fn default() -> Self {
        Self {
            fewer_than_two: SecondaryLead { extra_m: 1.0, speed_mps: 3.0 },
            two_out: SecondaryLead { extra_m: 1.5, speed_mps: 5.0 },
        }
    }
}

/// Clearance from the third-base foul line during the lead, m [E].
const THIRD_LEAD_FOUL_M: f64 = 0.6;
/// Distance to blend between the foul-side lead and the line through the bags, m [E].
const THIRD_LEAD_BLEND_M: f64 = 2.0;
/// The foul-side route has joined the line this far from third, m [E].
const THIRD_LEAD_END_M: f64 = 12.0;

pub(crate) fn runner_path_pos(start: u8, s: f64) -> Vec3 {
    let on_path = path_pos(s);
    if start != 3 {
        return on_path;
    }
    let from_third = s - 3.0 * BASE_L;
    let blend = (from_third.min(THIRD_LEAD_END_M - from_third) / THIRD_LEAD_BLEND_M).clamp(0.0, 1.0);
    let foul = THIRD_LEAD_FOUL_M * blend * core::f64::consts::FRAC_1_SQRT_2;
    on_path - Vec3::new(foul, foul, 0.0)
}

/// Catch-to-release of a fielder who takes a throw at a base and throws on, as a fraction of
/// his batted-ball transfer time [E, unsourced], about 0.45 s for an infielder. The cutoff
/// man taking a relay has his own sourced exchange (`throwing::relay_s`), and the pivot man
/// his (`throwing::pivot_s`; agent-models §5.5).
const RELAY_TRANSFER_FRAC: f64 = 0.7;
/// With two out, a tag play's weight against a force's 1.0 when the holder picks his throw
/// [E]: an equally sure tag loses to the force (a run can score ahead of a tag for the
/// third out, never ahead of a force), but a sure tag still beats a doubtful force.
const TWO_OUT_TAG_W: f64 = 0.9;
/// The radial infield/outfield boundary, m [C]: the edge of the infield dirt on the line
/// through second base. The grass line is an arc of 95 ft about the pitcher's plate, whose
/// front is 60.5 ft from home (OBR 2.01, diagram 1), so 155.5 ft = 47.40 m. It sets the settle
/// rule (rules-spec §5.8), infield or outfield backups, the no-play return throw and the
/// runner's deep-fly read; the scorer's outfield boundary is the same value
/// (`bb_stats::scorer::DEFAULT_OUTFIELD_RADIUS_M`).
const INFIELD_R: f64 = 155.5 * 0.3048;
/// How far behind the deepest Savant pre-pitch infield position
/// ([`crate::play::MAX_INFIELD_DEPTH_M`], 152.9 ft = 46.60 m) a ground ball is still in a
/// secondary infielder's range, m [E].
const INFIELD_DEPTH_MARGIN_M: f64 = 5.0;
/// The secondary ground-ball break's range (agent-models §5.7): an infielder who is not the
/// chaser breaks for a ground ball he believes he can field within this distance of home.
const INFIELD_BREAK_R: f64 = crate::play::MAX_INFIELD_DEPTH_M + INFIELD_DEPTH_MARGIN_M;
/// A ball an infielder believes comes down in the infield within this time of contact is a
/// ground ball to him (he breaks for it without waiting for a call); later, it is a pop-up
/// or a fly, s [E].
const GROUNDER_BOUNCE_S: f64 = 1.0;
/// How far a throw typically misses the spot it was aimed at, as a fraction of its length
/// (a receiver's belief: about 1.2° of direction and 3% of speed from a league-typical arm),
/// [E].
const THROW_MISS_FRAC: f64 = 0.05;
/// The smallest typical miss a receiver allows any throw, m [E].
const THROW_MISS_MIN_M: f64 = 0.5;
/// A receiver stays on the bag for a throw he believes comes within this fraction of his
/// receiving reach of it, [E].
const BAG_REACH_FRAC: f64 = 0.9;
/// A fielder's first read of the ball's velocity is off by this fraction of its speed per
/// axis [E]. The error is stretched in depth and shrinks with viewing time like the
/// position error (agent-models §5.1).
pub(crate) const READ_VEL_FRAC: f64 = 0.04;
/// The read closes as the ball comes down (agent-models §5.1, 2026-11-15): for a ball on its
/// first flight off the bat, every term of the read error is scaled by T/`READ_CLOSE_S` (at
/// most 1), T the time the ball needs to come down from where it is and how it is moving now
/// (in a vacuum). A threshold on the optical acceleration of the ball's image (Chapman 1968;
/// Fink, Foo & Warren 2009) makes the landing miss a fielder can tell apart shrink with the
/// square of the time left; the velocity term flown over T and scaled by T does that. Where
/// the closing starts is fitted to the owner's band for a routine fly (90–99% by fielding
/// ability; docs/calibration/2026-11-15-outfield-read.md §2), s [E].
const READ_CLOSE_S: f64 = 3.0;
/// A chaser's reads come faster as the ball comes down (agent-models §5.2, 2026-11-15): the
/// next one after this fraction of the time his own read leaves before it comes down, between
/// `REPLAN_MIN_S` and `REPLAN_S` [E].
const REPLAN_FRAC: f64 = 0.25;
const REPLAN_MIN_S: f64 = 0.1;
/// What an observer allows for the spin he cannot see when he works out where a ball must be
/// from the plate (agent-models §5.1): σ per axis, m, at 1 s since contact, growing as s^1.5.
/// `examples/plate_cue.rs` (2026-11-13): flying the true velocity back with typical spin
/// misses by a median 0.6 m at 0.5 s, 2.3 m at 1 s and 4.5–9 m at 2 s (0 with the true
/// spin); a 3D miss's median is about 1.5 σ per axis [E].
const PLATE_SPIN_M: f64 = 1.5;
/// The tracking read (agent-models §5.1, 2026-11-15): a fielder looks at a batted ball on its
/// first flight off the bat every `LOOK_S` seconds, however often he replans, s [E].
const LOOK_S: f64 = 0.1;
/// His eyes above his feet, m [E].
const EYE_H: f64 = 1.7;
/// How good one look after the first is, against the snapshot read's σ at the same distance
/// (agent-models §5.1): an outfielder's look is good to 0.2 × 3.5 m at 30 m across his line of
/// sight (1.3°), three times that along it. Fitted with `TRACK_BIAS_FRAC` to the owner's band
/// for a routine fly (grade 50 .967 in 100 games; docs/calibration/2026-11-15-tracking-read.md);
/// kept when `TRACK_BIAS_FRAC` was refitted (docs/calibration/2026-11-17-track-bias.md) [E].
const LOOK_SCALE: f64 = 0.2;
/// What his flight model may be missing, as an extra acceleration he can learn (agent-models
/// §5.1): its prior σ per axis, m/s², the plate cue's spin allowance (`PLATE_SPIN_M` at 1 s)
/// as a constant acceleration, ½·σ·1² = 1.5 m.
const CURVE_PRIOR: f64 = 2.0 * PLATE_SPIN_M;
/// How fast that missing acceleration changes along the flight (white noise intensity per
/// axis, m²/s⁵): the Magnus force, and so its error, roughly halves over a fly's six seconds
/// as the ball slows, so σ changes by about half the prior's over 6 s: (σ/2)²/6 [E].
const CURVE_DRIFT: f64 = CURVE_PRIOR * CURVE_PRIOR / 24.0;
/// How well he knows where the ball was hit (he saw it), σ per axis, m [E].
const TRACK_CONTACT_M: f64 = 0.3;
/// The persistent part of a look's error (agent-models §5.1): this fraction of the play's
/// fixed draws at the snapshot read's σ, on every look (a misjudged angle and fraction of the
/// range); the rest is fresh each look. The filter carries it as a state. Refitted with that
/// to the owner's band for a routine fly (90–99% by ability; grade 50 .967 on seeds 0–99 when
/// fitted, .979 with the look's full Jacobian; docs/calibration/2026-11-17-track-bias.md) [E].
pub(crate) const TRACK_BIAS_FRAC: f64 = 0.6;
/// Arriving under control (agent-models §5.2): a chaser with time in hand keeps this many of
/// his doubts (the σ of where he believes it comes down) between where he can stop and his
/// spot [E].
const ARRIVE_SIGMAS: f64 = 1.0;
/// Under this distance the reach estimate steps the movement itself (agent-models §5.2; Opus
/// wall-braking review P3-1), m [E].
const STEP_REACH_M: f64 = 5.0;
/// Below this speed a fielder reaching his spot simply stops on it (one tick's step), m/s [E].
const SETTLE_V: f64 = 0.5;
/// The batter's typical pull: how far to his pull side of straightaway his balls in play go
/// on average, degrees (batter-relative −1.23° in `examples/spray_by_location.rs`, 96 games) [E].
const MEAN_PULL_DEG: f64 = 1.23;
/// The scatter of balls in play about the direction a pitch location suggests, degrees
/// (σ within a 10 cm bin, 19.4–22.0° in `examples/spray_by_location.rs`) [E].
const SPRAY_SCATTER_DEG: f64 = 20.5;
/// A fielder's belief about what the bat's angle at contact says about the ball's direction
/// (agent-models §5.10): spray ≈ `SWING_SPRAY_A_DEG` + `SWING_SPRAY_B` × face, with
/// `SWING_SPRAY_RESID_DEG` of scatter about the line, batter-relative (negative = his pull
/// side). The face is the barrel's yaw less 90° (mirrored for a left-hander). Least-squares
/// fit to 5,521 fair balls in play, `examples/spray_by_bat_angle.rs` (96 games). Refit when the
/// swing or contact model changes [E].
const SWING_SPRAY_A_DEG: f64 = 6.91;
const SWING_SPRAY_B: f64 = 0.939;
const SWING_SPRAY_RESID_DEG: f64 = 18.0;
/// A fielder's belief about what the swing's lead says about the bat's face at contact
/// (agent-models §5.10): face ≈ `LEAD_FACE_A_DEG` + `LEAD_FACE_B_DPS` × lead (s), with
/// `LEAD_FACE_RESID_DEG` of scatter, batter-relative. Least-squares fit to 11,603 fair balls in
/// play, `examples/spray_cues.rs` (200 games, left-handers mirrored). Refit when the swing
/// model changes [E].
const LEAD_FACE_A_DEG: f64 = 184.10;
const LEAD_FACE_B_DPS: f64 = -1213.3;
const LEAD_FACE_RESID_DEG: f64 = 3.1;
/// A fielder jumps only for a ball he expects within this angle of his line from home,
/// degrees (infield, outfield) [E].
const JUMP_ZONE_DEG: [f64; 2] = [20.0, 25.0];
/// He commits fully when he expects the ball this many of his σs to one side, [E].
const JUMP_FULL_SIGMAS: f64 = 2.0;
/// Braking deceleration of an athlete stopping as fast as he can, m/s²: the mean of the
/// measured maximal decelerations of team-sport athletes, 4.2 to 5.0 (Harper, Cohen, Carling
/// & Kiely 2020, *Sports* 8(6), 76; SOURCED, docs/calibration/2026-11-15-wall-approach-
/// speed.md). One stopping law for every athlete: fielders (the wall cap, the stop point,
/// slowing, turning) and runners (braking onto a base, turning round); agent-models §5.3.
const BRAKE: f64 = throwing::BRAKE_MPS2;
/// How fast a fielder may still be moving when he reaches a wall, m/s [E, unsourced: no
/// public measurement of speed at wall contact exists; the owner chose "about 2 m/s", Q5,
/// 2026-11-15]. His speed toward the nearest wall is capped at
/// `sqrt(v_c² + 2·BRAKE·distance)` ([`wall_speed_cap`]). Planned: a per-fielder value.
pub const WALL_CONTACT_SPEED_MPS: f64 = 2.0;
/// Time to spare a fielder needs to believe he has before he calls a ball in the air, s [E].
const CALL_SLACK_S: f64 = 0.3;
/// A teammate's chase, and his call, reach the others this long after he begins it
/// (agent-models §5.7 `T_call`): the call is heard, or he is seen to go, s [E].
/// docs/calibration/2026-11-14-fielder-communication.md.
const CALL_HEARD_S: f64 = 0.3;
/// Who keeps a ball when two sure calls of the same tier meet (agent-models §5.7, the
/// priority table) [E]: CF over the corner outfielders; SS, 2B, 3B, 1B, C, P in the infield.
/// Indexed P C 1B 2B 3B SS LF CF RF; compared only within a tier.
const CALL_PRIORITY: [u8; 9] = [0, 1, 3, 5, 4, 6, 1, 2, 1];
/// A fielder re-decides, when what he knows of his teammates changes, only on a read of the
/// ball this recent, s [E]: an older read is not his view of the ball now.
const HEAR_READ_AGE_S: f64 = 1.0;
/// How sure a fielder is before he calls a ball in the air (agent-models §5.7): the time to
/// spare must also cover this many σ of his read's depth error where it comes down
/// ([`landing_depth_sigma`]), run off at his speed [E].
const CALL_SURE_SIGMAS: f64 = 1.0;
/// An infielder called off a ball in the air (agent-models §5.7, "peels off to back up")
/// stops this far short of where his own read has it coming down, on his side of it, m [E]:
/// out of the caller's way and of reach, close enough to field a ball that drops or pops out.
const PEEL_BACKUP_M: f64 = 5.0;
/// Cutoffs and relays (agent-models §5.7) [E]: a ball fielded beyond `RELAY_FROM_M` from home
/// gets a cutoff man on the line to the base; beyond `RELAY_DEEP_M` a middle infielder goes
/// out as the relay man instead. He stands short of the base by the throw's length less
/// `RELAY_OF_THROW_M` (what the outfielder throws), at least `CUTOFF_M` (about 45 ft, the
/// coaching guides' cutoff spot) and at most halfway.
const RELAY_FROM_M: f64 = 50.0;
const RELAY_DEEP_M: f64 = 80.0;
const RELAY_OF_THROW_M: f64 = 45.0;
const CUTOFF_M: f64 = 14.0;

/// Whether sure call `a` = (fielder, tier) outranks call `b` (agent-models §5.7): the higher
/// tier, then the priority table (`CALL_PRIORITY`), then, between the corner outfielders who
/// share a priority, the lower position number (left field over right), a convention every
/// fielder knows before the pitch (Codex fielder-communication review P2-2: two sure corner
/// calls both kept going).
fn outranks(a: (usize, u8), b: (usize, u8)) -> bool {
    (a.1, CALL_PRIORITY[a.0], std::cmp::Reverse(a.0)) > (b.1, CALL_PRIORITY[b.0], std::cmp::Reverse(b.0))
}

#[derive(Clone, Debug)]
enum Ball {
    /// Following physics (batted, deflected, or a throw that got away).
    Free { path: BallPath, next_touch: usize, next_wall: usize, settled_sent: bool },
    Held { by: usize },
    /// A throw in flight until it reaches the ground (`path` ends there; world-side only).
    /// The receiver's plan is his own: `aim` is the spot he was thrown to, `catch_at` where
    /// he believes he can take it (the aim until his first read), `read_err` his read's
    /// error draws for this throw, and `next_read` when he looks again.
    /// `relay`: thrown to the cutoff or relay man, who throws on to `to_base`.
    Thrown { path: Trajectory, aim: Vec3, catch_at: Vec3, read_at: Vec3, read_err: [f64; 4], next_read: f64, to_base: u8, receiver: usize, relay: bool },
}

/// A fielder's own decision on a loose ball (`Sim::decide`).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Decision {
    /// He goes for it.
    Go,
    /// He leaves it to a sure call he has heard.
    Yield,
    /// He leaves it: by his own estimate someone else gets to it first, or it is not his.
    Stay,
    /// A middle infielder leaves a ground ball to his partner, who by his own estimate fields
    /// it, and covers second.
    CoverSecond,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Role {
    Chase,
    Cover(u8),
    /// Cutoff or relay man for throws to this base (agent-models §5.7).
    Relay(u8),
    Backup,
    Idle,
}

/// What a fielder shows his teammates from time `at` (agent-models §5.7): the job he is
/// running to do (where he is going is visible; a chase also by his call), the base whose
/// throws he backs up, and his call tier while chasing. Teammates know it `CALL_HEARD_S` later.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Shown {
    at: f64,
    role: Role,
    backs: Option<u8>,
    tier: u8,
    /// A holder's call for a man to get to a bag for his throw ("two!", pointing): the base
    /// and the man (agent-models §5.7). Only while he holds the ball.
    asks: Option<(u8, usize)>,
}

/// The tracking read's state size: position, velocity and the learned acceleration (three
/// each), then the persistent part of his looks' error (`TB` and on, four).
const TN: usize = 13;
/// Where the persistent error's four components start in the state: position along his line
/// of sight, position across it (level), velocity along, velocity across, each in units of
/// its own σ (agent-models §5.1).
const TB: usize = 9;

/// A fielder's running estimate of a batted ball on its first flight (agent-models §5.1, the
/// tracking read): position, velocity and the extra acceleration his model is missing (the
/// curvature of the spin he cannot see) at `t`, `x` = [r, v, a], and the persistent part of
/// his own looks' error `b` (what stays the same on every look: a misjudged angle and range,
/// in units of its σ), with the covariance of all thirteen, `p` (rows and columns 0–8 the
/// ball's, `TB`.. his error's), kept while the ball is `ball` (`Sim::ball_gen`) and updated
/// by each look.
#[derive(Clone, Copy, Debug)]
struct Track {
    ball: u64,
    t: f64,
    x: [Vec3; 3],
    b: [f64; 4],
    p: [[f64; TN]; TN],
    /// Looks taken so far.
    looks: u32,
    next_look: f64,
}

fn comps(v: Vec3) -> [f64; 3] {
    [v.x, v.y, v.z]
}

impl Track {
    /// A new estimate: the ball's state `x` with covariance blocks `pb` (3×3 blocks of r, v and
    /// a), and his persistent error unknown, one σ on each of its four components, independent
    /// of the ball.
    fn new(ball: u64, t: f64, x: [Vec3; 3], pb: [[Mat3; 3]; 3]) -> Track {
        let mut p = [[0.0; TN]; TN];
        for (bi, row) in pb.iter().enumerate() {
            for (bj, m3) in row.iter().enumerate() {
                for i in 0..3 {
                    for j in 0..3 {
                        p[3 * bi + i][3 * bj + j] = m3.r[i][j];
                    }
                }
            }
        }
        for k in 0..4 {
            p[TB + k][TB + k] = 1.0;
        }
        Track { ball, t, x, b: [0.0; 4], p, looks: 0, next_look: t }
    }

    fn r(&self) -> Vec3 {
        self.x[0]
    }
    fn v(&self) -> Vec3 {
        self.x[1]
    }

    /// The covariance block of state blocks `i` and `j` (0 position, 1 velocity, 2 the
    /// learned acceleration).
    #[cfg(test)]
    fn blk(&self, i: usize, j: usize) -> Mat3 {
        Mat3::from_rows(core::array::from_fn(|a| core::array::from_fn(|c| self.p[3 * i + a][3 * j + c])))
    }

    /// The estimate run on to `t` with the observer's own physics (the spin he believes for
    /// it, `believed_flight`) plus the extra acceleration he has learned, and its covariance:
    /// F = [[I, h, h²/2], [0, I, h], [0, 0, I]] on the ball's part, that acceleration drifting
    /// as white noise of intensity `CURVE_DRIFT`, and his persistent error unchanged
    /// (agent-models §5.1).
    fn run_to(&self, env: &FlightEnv, t: f64, t_contact: f64) -> Track {
        let mut tr = *self;
        while t - tr.t > 1e-9 {
            let h = (t - tr.t).min(AGENT_DT);
            let [r, v, a] = tr.x;
            let (env1, w) = believed_flight(env, v, Some(tr.t - t_contact));
            let st = FlightEnv { extra: a, ..env1 }.step(&BallState { r, v, w }, h);
            tr.x = [st.r, st.v, a];
            let p = &mut tr.p;
            // F P, then (F P) Fᵀ.
            for c in 0..TN {
                for ax in 0..3 {
                    let (pr, pv, pa) = (p[ax][c], p[3 + ax][c], p[6 + ax][c]);
                    p[ax][c] = pr + h * pv + 0.5 * h * h * pa;
                    p[3 + ax][c] = pv + h * pa;
                }
            }
            for row in p.iter_mut() {
                for ax in 0..3 {
                    let (pr, pv, pa) = (row[ax], row[3 + ax], row[6 + ax]);
                    row[ax] = pr + h * pv + 0.5 * h * h * pa;
                    row[3 + ax] = pv + h * pa;
                }
            }
            let q = CURVE_DRIFT;
            let qb = [
                [q * h * h * h * h * h / 20.0, q * h * h * h * h / 8.0, q * h * h * h / 6.0],
                [q * h * h * h * h / 8.0, q * h * h * h / 3.0, q * h * h / 2.0],
                [q * h * h * h / 6.0, q * h * h / 2.0, q * h],
            ];
            for (bi, qrow) in qb.iter().enumerate() {
                for (bj, &qv) in qrow.iter().enumerate() {
                    for ax in 0..3 {
                        p[3 * bi + ax][3 * bj + ax] += qv;
                    }
                }
            }
            sym_n(p);
            tr.t += h;
        }
        tr
    }

    /// A look `z` (agent-models §5.1): a Kalman update with what `look` says he should see
    /// from the estimate as it stands now (so a velocity look taken after a position look is
    /// worked out from the corrected position) and the fresh part's covariance there, with H
    /// the full Jacobian of that expected look in the ball's position and velocity and in his
    /// persistent error (the persistent error's size and directions depend on where the ball
    /// is and how fast it goes; an extended Kalman update, the Jacobian by central
    /// differences).
    fn see(&mut self, look: &Look, z: Vec3) {
        let (r, v, b) = (self.x[0], self.x[1], self.b);
        let (zhat, rm) = look.expect(r, v, b);
        // H, 3 × TN (the learned acceleration does not enter a look).
        let mut h = [[0.0; TN]; 3];
        let unit = [Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0)];
        for k in 0..3 {
            let dr = unit[k] * LOOK_JAC_M;
            let jr = comps((look.expect(r + dr, v, b).0 - look.expect(r - dr, v, b).0) * (0.5 / LOOK_JAC_M));
            let jv = comps((look.expect(r, v + dr, b).0 - look.expect(r, v - dr, b).0) * (0.5 / LOOK_JAC_M));
            for ax in 0..3 {
                h[ax][k] = jr[ax];
                h[ax][3 + k] = jv[ax];
            }
        }
        for j in 0..4 {
            let (mut bp, mut bm) = (b, b);
            bp[j] += 1.0;
            bm[j] -= 1.0;
            let jb = comps((look.expect(r, v, bp).0 - look.expect(r, v, bm).0) * 0.5);
            for ax in 0..3 {
                h[ax][TB + j] = jb[ax];
            }
        }
        let p = &self.p;
        // P Hᵀ, TN × 3.
        let pht: [[f64; 3]; TN] = core::array::from_fn(|i| core::array::from_fn(|ax| (0..TN).map(|j| p[i][j] * h[ax][j]).sum()));
        let s = Mat3::from_rows(core::array::from_fn(|ax| core::array::from_fn(|bx| (0..TN).map(|j| h[ax][j] * pht[j][bx]).sum()))) + rm;
        let Some(si) = s.inverse() else { return };
        let e = comps(z - zhat);
        let k: [[f64; 3]; TN] = core::array::from_fn(|i| core::array::from_fn(|c| (0..3).map(|ax| pht[i][ax] * si.r[ax][c]).sum()));
        let dx: [f64; TN] = core::array::from_fn(|i| k[i][0] * e[0] + k[i][1] * e[1] + k[i][2] * e[2]);
        for (bi, xv) in self.x.iter_mut().enumerate() {
            *xv = *xv + Vec3::new(dx[3 * bi], dx[3 * bi + 1], dx[3 * bi + 2]);
        }
        for (j, bv) in self.b.iter_mut().enumerate() {
            *bv += dx[TB + j];
        }
        let mut np = self.p;
        for i in 0..TN {
            for j in 0..TN {
                np[i][j] -= k[i][0] * pht[j][0] + k[i][1] * pht[j][1] + k[i][2] * pht[j][2];
            }
        }
        sym_n(&mut np);
        self.p = np;
    }
}

/// The step for a look's Jacobian in the ball's position (m) and velocity (m/s) [E: numerical,
/// small against every distance and speed a look sees].
const LOOK_JAC_M: f64 = 1e-3;

/// What a fielder knows about one of his looks (agent-models §5.1): from his eyes, at the
/// ball's position (`vel` false) or its velocity (`vel` true), with the read's σ0 per metre
/// of distance (`sigma0 / READ_REF_DIST`), its velocity fraction, the scale of the fresh part
/// (`LOOK_SCALE`, or the first look's shrink) and of the persistent part (`TRACK_BIAS_FRAC`,
/// times the first look's shrink on the first look).
#[derive(Clone, Copy, Debug)]
struct Look {
    eye: Vec3,
    vel: bool,
    sigma_per_m: f64,
    vel_frac: f64,
    scale: f64,
    kb: f64,
}

impl Look {
    /// What he expects to see if the ball is at `r` going `v` and his persistent error is `b`
    /// (in units of its σ): the ball's position (or velocity) moved by the persistent error
    /// along his line of sight (×`DEPTH_STRETCH`; ×½ of it for velocity) and across it,
    /// level, at the σ of a look there; and the fresh part's covariance there. The world
    /// draws a look the same way about the true ball (`Sim::take_looks`).
    fn expect(&self, r: Vec3, v: Vec3, b: [f64; 4]) -> (Vec3, Mat3) {
        let (d, u, a, w) = sight_frame(self.eye, r);
        if self.vel {
            let base = self.vel_frac * v.norm();
            let mean = v + u * (0.5 * DEPTH_STRETCH * self.kb * base * b[2]) + a * (self.kb * base * b[3]);
            (mean, sight_cov(u, a, w, base * self.scale, 0.5 * DEPTH_STRETCH))
        } else {
            let base = self.sigma_per_m * d;
            let mean = r + u * (DEPTH_STRETCH * self.kb * base * b[0]) + a * (self.kb * base * b[1]);
            (mean, sight_cov(u, a, w, base * self.scale, DEPTH_STRETCH))
        }
    }
}

/// A square matrix made exactly symmetric (the mean of it and its transpose).
fn sym_n(p: &mut [[f64; TN]; TN]) {
    for i in 0..TN {
        for j in 0..i {
            let s = 0.5 * (p[i][j] + p[j][i]);
            p[i][j] = s;
            p[j][i] = s;
        }
    }
}

#[cfg(test)]
thread_local! {
    /// Tests only: multipliers on a look's fresh noise and on its persistent part.
    static LOOK_NOISE: std::cell::Cell<(f64, f64)> = const { std::cell::Cell::new((1.0, 1.0)) };
}

/// A look's noise multipliers, fresh and persistent: (1, 1) outside tests.
fn look_noise() -> (f64, f64) {
    #[cfg(test)]
    {
        LOOK_NOISE.with(|c| c.get())
    }
    #[cfg(not(test))]
    {
        (1.0, 1.0)
    }
}

/// An observer's line of sight from `eye` to `ball` (agent-models §5.1): the distance, the
/// unit line of sight u, and two unit directions across it, a level and b = u × a.
fn sight_frame(eye: Vec3, ball: Vec3) -> (f64, Vec3, Vec3, Vec3) {
    let dv = ball - eye;
    let d = dv.norm();
    let u = if d > 1e-9 { dv * (1.0 / d) } else { Vec3::new(0.0, 1.0, 0.0) };
    let h = Vec3::new(-u.y, u.x, 0.0);
    let a = if h.norm() > 1e-6 { h.normalized() } else { Vec3::new(1.0, 0.0, 0.0) };
    (d.max(1.0), u, a, u.cross(a))
}

/// A look's error covariance: σ across the line of sight on both axes, `along` × σ on it.
fn sight_cov(u: Vec3, a: Vec3, b: Vec3, sigma: f64, along: f64) -> Mat3 {
    (Mat3::outer(a, a) + Mat3::outer(b, b)) * (sigma * sigma) + Mat3::outer(u, u) * (along * along * sigma * sigma)
}

#[derive(Clone, Debug)]
struct Fielder {
    id: AgentId,
    a: FielderAttrs,
    pos: Vec3,
    /// Velocity on the ground, m/s.
    vel: Vec3,
    /// The velocity he goes for until he has read the ball (his jump, agent-models §5.10).
    jump: Vec3,
    role: Role,
    target: Vec3,
    next_plan: f64,
    /// Fixed per-play draws for the read error (depth, lateral, velocity depth, velocity lateral).
    err: [f64; 4],
    /// His running estimate of a ball on its first flight (agent-models §5.1).
    track: Option<Track>,
    /// The σ of where his latest read has the ball coming down, along his line of sight to it
    /// (his doubt; 0 without a track), and when it was worked out.
    doubt: f64,
    doubt_at: f64,
    attempts: u32,
    blocked_until: f64,
    ready_at: f64,
    dove: bool,
    /// His dive's lunge (start time, from, to): the body goes toward the ball, then lies
    /// there until he is up.
    lunge: Option<(f64, Vec3, Vec3)>,
    /// The fielder's own prediction of the ball, refreshed at replans.
    belief: Option<BallPath>,
    /// Believed intercept time.
    t_int: f64,
    /// His own intercept point from that read (his `target` may be a base or a backup spot).
    int_pt: Vec3,
    /// When he last read the ball (`belief`, `t_int`, `int_pt`).
    read_at: f64,
    /// Which ball that read was of (`Sim::ball_gen` then): a ball that comes loose again in
    /// the same tick (a deflection, or a throw that comes down and then comes off a glove) is
    /// a new ball to read (Codex fielder-communication review P2-1, recheck P2-1).
    read_ball: u64,
    /// His own call on the ball from his latest call read (agent-models §5.7): 3 or 2 a sure
    /// call (outfielder, infielder), 1 going for a ball in the air without calling it, 0 no
    /// catch in the air.
    call: u8,
    /// What he has shown his teammates, one entry whenever it changed (`Sim::publish`).
    /// Others know it `CALL_HEARD_S` later (`Sim::known`, `Sim::seen`).
    shown: Vec<Shown>,
    /// Last close-range re-read of the ball.
    last_close_read: f64,
    /// Since when his read has said, without a break, that only a dive gets this ball
    /// (`dive_gains`); infinite while he believes he has a play on his feet. He can be laid
    /// out only as far as the time since then allows.
    must_dive_since: f64,
    last_dive_read: f64,
    /// An outfielder has weighed this ball and will dive for it (`OF_DIVE_MIN_P`): decided
    /// once, when a dive first offers, and kept until the ball changes.
    will_dive: std::cell::Cell<bool>,
    /// Throws made this play (keys the throw-execution noise).
    throws: u32,
    /// Since he last secured the ball: when his set-up for a throw began (on his feet with
    /// it), his catch-to-release for this kind of throw, the way he faced taking it (`None`
    /// for a throw he received: he set up to it, facing his next target), and how he was
    /// moving (agent-models §5.5).
    up_at: f64,
    set_base: f64,
    face: Option<Vec3>,
    catch_vel: Vec3,
    /// Visible heading, motion and reception cue at the catch, published before choosing
    /// footwork. A later set-up (the pivot on reaching second) changes his footwork, not
    /// what the runners saw at the reception (catch-read fixes review P1-2).
    catch_heading: Option<Vec3>,
    catch_seen_vel: Vec3,
    catch_received_throw: bool,
    /// He received that throw visibly as the cutoff man (his shown job was a relay; agent-models
    /// §6.4): with a reception far from every bag, how a runner tells a relay from a feed.
    catch_shown_relay: bool,
    /// He took it on the ground from a dive (set at each securing; `dove` is his recovery
    /// state and is cleared while he still holds the ball: Codex hurried-throws review #1).
    took_diving: bool,
    /// A stretch at the catch was visible to the runner.
    catch_stretched: bool,
    /// The base whose throws he is backing up, if that is his job on this play.
    backs: Option<u8>,
    /// Called off a ball in the air by a sure call (agent-models §5.7): the spot he backs up
    /// the catch from, short of where his own read has it coming down. Kept across re-calls
    /// he is not part of, until the ball comes down or he is given another job.
    peel: Option<Vec3>,
    /// The bag he is carrying the ball to (agent-models §5.5 A4), so that abandoning the
    /// carry for a throw or for no play brakes him instead of leaving him running at it.
    carry_to: Option<u8>,
    /// Holding the ball and waiting on a teammate to get to the bag he means to throw to: the
    /// base and the man he calls to it (agent-models §5.7). Private until published; the man
    /// acts on it only when he hears it (`Sim::heed_requests`).
    asks: Option<(u8, usize)>,
    /// His set-up is a double-play pivot at second (agent-models §5.5): its own time, turn
    /// and footwork from how he got the bag, no early release.
    pivot: bool,
    /// A pivot man who took the feed off the bag: his pivot starts when he reaches it.
    pivot_wait_bag: bool,
    /// He took a relay as the cutoff man (agent-models §5.5, the relay man's exchange): his
    /// set-up is `throwing::relay_s` from his catch, turn included, no early release.
    relayed: bool,
}

#[derive(Clone, Debug)]
struct Runner {
    id: AgentId,
    start: u8,
    /// Distance along the base path from home, m (base k at k·BASE_L).
    s: f64,
    v: f64,
    /// Last physical movement direction. Public signed velocity is direction × v;
    /// private `goal` is never offered to the fielder's target builder.
    direction: f64,
    vmax: f64,
    /// His own acceleration time constant, s.
    tau: f64,
    /// Base the runner is heading for (may be behind it on a retreat).
    goal: u8,
    out: bool,
    scored: bool,
    /// Holding at a base until a catch is made (tag-up).
    tag_up: bool,
    /// He tagged up and is going on the catch: when his first step comes. Until he next
    /// stops he runs from the set start (agent-models §6.4); no earlier speed carries over.
    set_start: Option<f64>,
    /// Since when he has visibly not been moving forward (signed speed at most
    /// `SEEN_MOVING_MPS`), from the previous tick's motion; `None` while he is. What a
    /// runner behind him can see (agent-models §6.4).
    still_since: Option<f64>,
    /// The base he turned back from because the runner ahead was seen holding it, while he
    /// is on his way back; he resumes if that base visibly clears.
    turned_back: Option<u8>,
    moving_since: f64,
    last_base: u8,
    /// Going back to the start base to retouch after a caught fly.
    retouch: bool,
    /// Holding at a base until the runner ahead clears the next one.
    waiting: bool,
    /// Coasting at the secondary lead's speed while picking up the contact cue.
    reading_contact: bool,
    /// Waiting partway to the next base (base-path distance) to see whether a ball in the
    /// air is caught.
    hold_s: Option<f64>,
    /// The timing-error draw of the latest advance decision (standard normal; diagnostics).
    last_timing_draw: f64,
    /// The runner has seen the catch and settled whether he owes a retouch (decided once:
    /// off his base at the catch ⇒ owes it; on it ⇒ free to leave).
    catch_seen: bool,
    /// His own position on the catch tick was off his original bag.
    off_base_at_catch: bool,
    /// Advance decisions made this play. Decision n (from 1) draws its timing error from
    /// `RunnerPerception` sub-index 2n and its read of a loose or thrown ball from 2n + 1; sub-index 1
    /// is the first read at contact (tag up or go).
    decisions: u32,
    /// Times he has turned round between bases since he last stood on one (`MAX_TURNS`).
    turns: u8,
    /// How many times the ball had been secured and thrown when he last looked at the play
    /// (his last decision): a change is something new to see (agent-models §6.4).
    looked: (usize, usize),
    /// He went at his first look on a ball in the air, or holds partway, and keeps watching it
    /// (agent-models §6.3, the in-flight relook): when he looks next, and that look's number.
    fly_look: Option<(f64, u32)>,
    /// The batter-runner's overrun of first (agent-models §6.2): his phase, and how far past
    /// the bag down the first-base line he is, m (his `s` stays at first meanwhile).
    overrun: OverrunPhase,
    over_m: f64,
}

/// A batter-runner who stopped at first runs through it, pulls up, and walks back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverrunPhase {
    None,
    /// Through the bag, braking.
    Out,
    /// Walking back to it.
    Back,
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
    /// The batter's side of the plate (public).
    pub bats: bb_observe::Hand,
    /// Where the pitch reached the batter, horizontally (sim x, m): what fielders read,
    /// with noise, before contact to anticipate the ball (agent-models §5.10). `None`: no
    /// pitch was seen (lab set-ups), so nobody leans.
    pub pitch_x: Option<f64>,
    /// The bat's horizontal angle at the instant of contact (yaw of its axis, knob to tip,
    /// `atan2(x, y)`, rad), sampled by the orchestrator from the executed swing. World truth:
    /// a fielder gets it only through his noisy read in `jump_at_contact` (agent-models
    /// §5.10). `None`: no swing seen (lab set-ups); fielders lean on the pitch alone.
    pub bat_yaw: Option<f64>,
    /// The swing's lead: how long before the pitch reached the plate (its free flight's
    /// crossing of the judgment plane) the bat started forward (`Swing::t_start`, 10% of peak
    /// speed), s. World truth, read with each fielder's error in `jump_at_contact`
    /// (agent-models §5.10). `None`: no swing seen (lab set-ups).
    pub swing_lead: Option<f64>,
    pub seed: u64,
    pub play: u32,
    /// Record ball and actor tracks (timelines and visualization only).
    pub record: bool,
    /// How runners arrive at contact with their secondary leads (agent-models §6.1).
    pub runner_start: RunnerStartParams,
    /// How runners who tag up start, and what they believe of the tag (agent-models §6.4).
    pub tag_up: bb_agents::running::TagUpParams,
    /// How runners weigh a race against the run value of the base (agent-models §6.3).
    pub advance: bb_agents::running::AdvanceParams,
}

/// Physical throw facts retained for the scorer, not passed to an agent.
#[derive(Clone, Copy, Debug)]
pub struct ScorerThrowFact {
    pub thrower: AgentId,
    pub t: f64,
    pub origin: Vec3,
    pub target: Vec3,
    pub base: u8,
    /// The throw was selected as a play on a runner, not a routine return.
    pub attempted_out: bool,
    pub batter_goal: u8,
    /// The runner it was meant to retire (0: a routine return, or unknown).
    pub runner: AgentId,
}

#[derive(Clone, Copy, Debug)]
pub struct ScorerCarryFact { pub fielder: AgentId, pub t: f64, pub base: u8, pub batter_goal: u8, pub runner: AgentId }

#[derive(Clone)]
pub struct LiveOutput {
    pub scorer_carries: Vec<ScorerCarryFact>,
    /// First glove contact with the batted ball, including a deflection.
    pub first_handling: Option<(f64, AgentId)>,
    /// The batter had not touched first when the play ended: when he would have, running as he
    /// was (a world-side estimate for the scorer; no agent reads it).
    pub batter_first_eta: Option<f64>,
    /// For the first fielding of a batted ball: seconds after he secures it until a grade-50
    /// fielder in his place is set to throw to first (the get-up after a dive, transfer, turn,
    /// footwork), noise-free. Scorer evidence; no agent reads it.
    pub ref_set_s: Option<f64>,
    /// Physical releases, independent of visualization recording.
    pub scorer_throws: Vec<ScorerThrowFact>,
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
    /// For each release: how hurried it was (0 set, 1 the earliest he could let go) and
    /// whether he was off balance (agent-models §5.5). Diagnostics.
    pub hurry: Vec<(f64, bool)>,
    /// For each release: its effort, the fraction of the thrower's arm (agent-models §5.5).
    pub effort: Vec<f64>,
    /// Every dive (t, fielder, where he left his feet, where his body lands), in time order.
    /// He is back on his feet [`DIVE_GETUP_S`] after the dive.
    pub dives: Vec<(f64, AgentId, Vec3, Vec3)>,
    /// Every out, with how close the play was, in time order (the umpire's calls).
    pub calls: Vec<crate::umpire::OutCall>,
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

/// Read tracing for diagnostics (`examples/fly_misses.rs`): while a batted ball is on its
/// first flight, one record per fielder per tick of what he believes and does. Off unless a
/// diagnostic turns it on for its own thread; written only, never read by any decision.
pub mod read_trace {
    use bb_math::Vec3;
    use std::cell::{Cell, RefCell};
    #[derive(Clone, Debug)]
    pub struct Rec {
        pub t: f64,
        pub i: usize,
        /// 0 idle, 1 chase, 2 cover, 3 relay, 4 backup.
        pub role: u8,
        pub pos: Vec3,
        pub vel: Vec3,
        /// Where his latest read has the ball coming down (time, x, y).
        pub land: Option<(f64, f64, f64)>,
        pub read_at: f64,
        pub target: Vec3,
        pub call: u8,
        pub peel: bool,
        pub must_dive_since: f64,
        pub dove: bool,
    }
    thread_local! {
        pub static ON: Cell<bool> = const { Cell::new(false) };
        pub static RECS: RefCell<Vec<Rec>> = const { RefCell::new(Vec::new()) };
    }
    pub fn on() -> bool {
        ON.with(|x| x.get())
    }
    /// Takes this thread's records (the last play's, if the caller clears between plays).
    pub fn take() -> Vec<Rec> {
        RECS.with(|r| std::mem::take(&mut *r.borrow_mut()))
    }
}

/// Fielder-communication tracing (BB_DEBUG_COMM): each decision and what each man knows.
fn debug_comm() -> bool {
    thread_local! { static ON: bool = std::env::var("BB_DEBUG_COMM").is_ok(); }
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
        | Input::BallReleasedFromGlove { t, .. }
        | Input::BaseTouched { t, .. }
        | Input::BaseLeft { t, .. }
        | Input::TagContact { t, .. }
        | Input::PlayEndCheck { t } => Some(t),
    }
}

fn out_weight_at(goal: u8, outs: u8, force: bool) -> f64 {
    if outs >= 2 {
        if force { 1.0 } else { TWO_OUT_TAG_W }
    } else {
        [0.9, 1.0, 1.1, 1.3][(goal as usize).saturating_sub(1).min(3)]
    }
}

fn phi(x: f64) -> f64 {
    0.5 * (1.0 + m::erf(x / core::f64::consts::SQRT_2))
}

/// The base a moving runner is visibly heading for: the next one in his direction of motion.
fn visible_target(r: &Runner, params: throwing::ChoiceParams) -> Option<u8> {
    params.runner_target(r.s, r.direction * r.v, BASE_L)
}

/// Position on the base path at distance `s` from home.
/// How far up the first-base line the batter joins it from his box, m [E].
const BOX_EXIT_M: f64 = 4.0;

/// The batter's path coordinate in his box. He runs straight from the box to the point
/// `BOX_EXIT_M` up the first-base line, and his coordinate there is `BOX_EXIT_M`, so he
/// starts that straight line's length short of it. A right-handed batter starts behind the
/// plate's zero (about −0.55 m: 27.98 m to first), a left-handed one ahead of it (about
/// +0.85 m: 26.58 m): the left-handed batter's 1.4 m shorter run (tag-up review P2-3).
pub(crate) fn batter_start_s(batter_box: Vec3) -> f64 {
    let d = path_pos(BOX_EXIT_M) - Vec3::new(batter_box.x, batter_box.y, 0.0);
    BOX_EXIT_M - m::sqrt(d.x * d.x + d.y * d.y)
}

pub(crate) fn path_pos(s: f64) -> Vec3 {
    let s = s.clamp(0.0, 4.0 * BASE_L);
    let k = ((s / BASE_L).floor() as u8).min(3);
    let f = (s - k as f64 * BASE_L) / BASE_L;
    let a = base_pos(k);
    let b = base_pos(k + 1);
    a + (b - a) * f
}

/// A feed receiver at second who took it off the bag stops being a pivot man (the force is
/// gone, or he throws instead of going to the bag): he is an ordinary receiver, set up from his
/// catch (`RELAY_TRANSFER_FRAC` of his transfer), not facing a feed, and no longer heading for
/// the bag.
fn end_pivot_wait(f: &mut Fielder) {
    (f.pivot, f.pivot_wait_bag) = (false, false);
    f.set_base = RELAY_TRANSFER_FRAC * f.a.transfer;
    f.face = None;
    f.target = stop_point(f);
}

/// Whether a throw to `tgt` by a pivot man is the pivot (a throw to first after a force at
/// second, agent-models §5.5); a throw to third or home is an ordinary throw.
fn pivot_throw(fd: &Fielder, tgt: Vec3) -> bool {
    fd.pivot && (tgt - base_pos(1)).norm() < 5.0
}

/// Distance covered toward a goal in time `t` by a fielder whose velocity has the component
/// `v0` toward it, with the movement step's laws (agent-models §5.3): from `v0` ≥ 0 the
/// speed law v → v_max with time constant τ ([`run_dist`] is the `v0` = 0 case); from
/// `v0` < 0, moving away, he brakes at [`BRAKE`] to a stop and then runs from rest. Sideways
/// speed is left out (the step sheds it at the fielder's lateral acceleration).
fn run_dist_from(t: f64, v0: f64, v_max: f64, tau: f64) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    if v0 < 0.0 {
        let t_stop = -v0 / BRAKE;
        if t <= t_stop {
            return v0 * t + 0.5 * BRAKE * t * t;
        }
        return -v0 * v0 / (2.0 * BRAKE) + run_dist(t - t_stop, v_max, tau);
    }
    v_max * t - (v_max - v0) * tau * (1.0 - m::exp(-t / tau))
}

/// Time to cover distance `d` toward a goal: the inverse of [`run_dist_from`] (increasing in
/// t once he is coming toward it), by bisection.
fn run_time_from(d: f64, v0: f64, v_max: f64, tau: f64) -> f64 {
    if d <= 0.0 {
        return 0.0;
    }
    // From rest the run takes at most d / v_max + τ; moving away adds the stop and the
    // ground lost.
    let back = if v0 < 0.0 { -v0 / BRAKE } else { 0.0 };
    let (mut lo, mut hi) = (back, back + (d + v0.min(0.0).powi(2) / (2.0 * BRAKE)) / v_max + tau);
    for _ in 0..50 {
        let mid = 0.5 * (lo + hi);
        if run_dist_from(mid, v0, v_max, tau) < d { lo = mid } else { hi = mid }
    }
    0.5 * (lo + hi)
}

/// The slowest a runner holds on his way to a base he will stop at, m/s [E]: he does not
/// crawl the last metre. The speed at which he arrives, then planted.
const APPROACH_MIN_MPS: f64 = 1.0;

/// One tick of a runner's speed on his way back to a base he will stop on, `rem` m from it:
/// the speed law, limited by braking distance after this tick's movement, and slowing by
/// at most `BRAKE` a tick when he is already too fast
/// (agent-models §6.4, Codex recheck P2-2).
fn approach_speed(v: f64, v_max: f64, tau: f64, rem: f64) -> f64 {
    let free = v + (v_max - v) * (DT / tau);
    let brake_step = BRAKE * DT;
    // Leave enough room after travelling v_next·DT to brake from v_next.
    let cap = (m::sqrt(brake_step * brake_step + 2.0 * BRAKE * rem.max(0.0)) - brake_step).max(APPROACH_MIN_MPS);
    free.min((v - brake_step).max(cap))
}

/// One step toward a stopping point. If a new goal is too close to brake for, the runner
/// carries his remaining speed past it and comes back.
fn step_to_stop(s: f64, signed_v: f64, goal: f64, vmax: f64, tau: f64) -> (f64, f64) {
    let gap = goal - s;
    if gap.abs() < 1e-9 && signed_v.abs() <= APPROACH_MIN_MPS {
        return (goal, 0.0);
    }
    let dir = gap.signum();
    let v = if gap.abs() < 1e-9 || signed_v * dir < 0.0 {
        signed_v.signum() * (signed_v.abs() - BRAKE * DT).max(0.0)
    } else {
        dir * approach_speed(signed_v.abs(), vmax, tau, gap.abs())
    };
    let next = s + v * DT;
    if v.abs() <= APPROACH_MIN_MPS && v * dir > 0.0 && (goal - next) * dir <= 0.0 {
        (goal, 0.0)
    } else {
        (next, v)
    }
}

/// Time to plant at a stopping point d metres ahead (v0 is signed toward it).
fn run_time_to_stop(d: f64, v0: f64, v_max: f64, tau: f64) -> f64 {
    let (mut s, mut v, mut t) = (0.0, v0, 0.0);
    while (s != d || v != 0.0) && t < 60.0 {
        (s, v) = step_to_stop(s, v, d, v_max, tau);
        t += DT;
    }
    t
}

/// A runner moving forward faster than this is seen to be advancing, m/s [E] (the fielders'
/// `ChoiceParams::moving_mps`, agent-models §5.6).
const SEEN_MOVING_MPS: f64 = 0.5;
/// How long a runner must see the runner ahead not moving forward before he takes it that
/// the base ahead is held, s [E, unsourced: a visual reaction time]. A runner who has just
/// decided to go is still under `SEEN_MOVING_MPS` for about three ticks.
const HOLD_READ_S: f64 = 0.2;
/// Height at which a runner takes a throw in the air to come down, m [E]: chest height at the
/// receiver, where throws are aimed (`THROW_CATCH_Z`). His read of where it is going is the
/// ballistic projection of its position and velocity to this height (agent-models §6.4).
const RUNNER_READ_CATCH_Z: f64 = THROW_CATCH_Z;
/// How far ahead a runner flies his read of a throw, s [E]: far more than any throw takes.
const THROW_READ_S: f64 = 6.0;
/// How near a throw's line must pass a base for him to take it that it can be caught there on
/// the way, m [E, unsourced]: a receiver's reach (`RECEIVE_REACH`) and a step.
const PASS_REACH_M: f64 = 2.5;
/// How high the ball may be over that base for a fielder to take it, m [E]: a leap over the
/// catching ceiling's usual reach (`CATCH_MAX_Z` is the fielder-side figure).
const PASS_MAX_Z: f64 = 3.0;
/// A catch at a base on the way, s [E, unsourced]: the receiver's handling before he can
/// act on the ball (the receiving model's 0.05 s, with a tag-less margin).
const PASS_CATCH_S: f64 = 0.1;
/// How often a runner who believes himself trapped (safe at neither base) turns round between
/// two bases on a fresh look at the play, per stretch between bases [E]. A rundown is not
/// modelled (the fielders do not close on him), and a runner who turned at every throw played
/// catch with them for the whole 40 s; so he turns once, then commits to the base he is going
/// back to or on to (agent-models §6.4). A look that finds a base he believes he reaches
/// safely is not counted against him.
const MAX_TURNS: u8 = 1;

/// How long a runner leaving from a tag-up needs to cover `d` m: the wait for his first step,
/// then the speed law from rest with the set start's time constant (agent-models §6.4). His
/// decision uses this and his legs run the same law; no earlier speed carries over.
fn tag_up_time(tag: bb_agents::running::TagUpParams, wait_s: f64, d: f64, v_max: f64, tau: f64) -> f64 {
    wait_s + run_time(d, v_max, tau * tag.tau_gain)
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

/// Where a fielder comes to rest if he brakes now ([`BRAKE`] along his motion).
fn stop_point(fd: &Fielder) -> Vec3 {
    let v = fd.vel.norm();
    // v²/(2·BRAKE) along his motion (Codex momentum review #2: the factor v was missing).
    let p = fd.pos + fd.vel * (v / (2.0 * BRAKE));
    Vec3::new(p.x, p.y, 0.0)
}

/// A fielder with no job (agent-models §5.7) pulls up where braking stops him
/// ([`stop_point`]) instead of running on toward whatever his last job (a cutoff spot, a backup
/// spot) or his own intercept of the ball left in `target`. Every transition into `Role::Idle`
/// goes through here; a caller with a real job for him (a base to cover, a throw to back up)
/// assigns it afterwards. Called on the transition only, never per tick: an idle man keeps
/// braking to the spot he chose when his job ended (Codex idle-fielder review #1).
fn go_idle(fd: &mut Fielder) {
    fd.role = Role::Idle;
    fd.backs = None;
    fd.target = stop_point(fd);
}

/// The typical spin of a batted ball, an observer's belief about the contact model (agent-
/// models §5.1, "The unseen spin"), by launch angle (rows: 10° bins centred −25° … 55°) and
/// exit speed (columns: 10 mph bins centred 65 … 105 mph; the outer columns take everything
/// below 70 and from 100 up), rpm, from `examples/batted_spin.rs` (96 games, seeds 0–95,
/// balls launched within the foul lines). `SPIN_BACK_RPM`: the median backspin.
/// `SPIN_REST_RPM`: the side and gyro spin he cannot see, as a drag-only allowance
/// (`FlightEnv::unseen_spin`), fitted so each cell's balls carry as far as they truly do in the
/// median; the fit sweeps all cells until none moves by 50 rpm (five sweeps on 2026-11-19,
/// on the games of `925f8ea`). Refit both when the contact model changes.
const SPIN_LA_DEG: [f64; 9] = [-25.0, -15.0, -5.0, 5.0, 15.0, 25.0, 35.0, 45.0, 55.0];
const SPIN_EV_MPH: [f64; 5] = [65.0, 75.0, 85.0, 95.0, 105.0];
pub const SPIN_BACK_RPM: [[f64; 5]; 9] = [
    [-6681.0, -5454.0, -4905.0, -4417.0, -4169.0],
    [-3575.0, -3252.0, -3154.0, -2959.0, -2807.0],
    [-1641.0, -1933.0, -1857.0, -1770.0, -1720.0],
    [-703.0, -843.0, -613.0, -557.0, -481.0],
    [585.0, 571.0, 614.0, 764.0, 658.0],
    [1868.0, 1720.0, 1840.0, 2009.0, 1855.0],
    [2773.0, 2819.0, 3004.0, 3066.0, 2846.0],
    [3730.0, 3894.0, 4251.0, 4225.0, 4121.0],
    [5505.0, 5630.0, 5677.0, 5379.0, 5230.0],
];
pub const SPIN_REST_RPM: [[f64; 5]; 9] = [
    [0.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 0.0, 0.0],
    [4067.0, 3020.0, 4501.0, 4315.0, 3321.0],
    [5422.0, 5415.0, 4353.0, 3591.0, 2748.0],
    [6045.0, 3496.0, 3410.0, 3293.0, 2818.0],
    [5090.0, 2558.0, 4390.0, 3080.0, 1349.0],
    [0.0, 0.0, 556.0, 1767.0, 2329.0],
];

/// Bilinear look-up of a [`SPIN_LA_DEG`] × [`SPIN_EV_MPH`] table at launch angle `la` (rad)
/// and exit speed `ev` (m/s), flat beyond the outer bin centres.
pub fn spin_table(t: &[[f64; 5]; 9], la: f64, ev: f64) -> f64 {
    let at = |x: f64, c: &[f64]| {
        let x = x.clamp(c[0], c[c.len() - 1]);
        let k = (((x - c[0]) / (c[1] - c[0])).floor() as usize).min(c.len() - 2);
        (k, (x - c[k]) / (c[1] - c[0]))
    };
    let (i, fi) = at(la.to_degrees(), &SPIN_LA_DEG);
    let (j, fj) = at(ev / bb_math::units::MPH, &SPIN_EV_MPH);
    let row = |i: usize| t[i][j] + (t[i][j + 1] - t[i][j]) * fj;
    row(i) + (row(i + 1) - row(i)) * fi
}

/// How an observer believes a ball he reads going `ve` now flies (agent-models §5.1): the
/// spin he believes it has, and his atmosphere `env` with the unseen spin's drag allowance
/// (`FlightEnv::unseen_spin`). Straight off the bat: the typical spin (`SPIN_BACK_RPM`,
/// `SPIN_REST_RPM`) at the launch angle and exit speed he estimates, decayed since contact as
/// his own physics decays it. He estimates the launch by flying his read of the velocity now
/// back the `off_bat_since` seconds to the plate with his own physics ([`fly_back`]; drag and
/// gravity act on the velocity alone), with the typical spin at a first-guess launch of
/// (v_h, v_z + g·s), then looks the tables up at the launch that back-flight gives and scales
/// them by the decay it undid (2026-11-15; docs/calibration/2026-11-15-outfield-read.md: the
/// first guess alone is too steep by mid-flight, and the launch spin is more than the ball
/// still has). After a bounce or a deflection, a little topspin and no allowance.
pub fn believed_flight(env: &FlightEnv, ve: Vec3, off_bat_since: Option<f64>) -> (FlightEnv, Vec3) {
    let rpm = core::f64::consts::PI / 30.0;
    let h = Vec3::new(ve.x, ve.y, 0.0).normalized();
    let axis = Vec3::new(h.y, -h.x, 0.0);
    let Some(since) = off_bat_since else {
        return (*env, axis * (-600.0 * rpm));
    };
    let typical = |v: Vec3, decay: f64| {
        let la = m::atan2(v.z, m::hypot(v.x, v.y));
        let (back, rest) = (spin_table(&SPIN_BACK_RPM, la, v.norm()), spin_table(&SPIN_REST_RPM, la, v.norm()));
        (FlightEnv { unseen_spin: rest * rpm * decay, ..*env }, axis * (back * rpm * decay))
    };
    let (env0, w0) = typical(Vec3::new(ve.x, ve.y, ve.z + bb_math::units::G * since), 1.0);
    if since <= 0.0 || w0.norm() == 0.0 {
        return (env0, w0);
    }
    let launch = fly_back(&env0, ve, w0, since);
    typical(launch.v, w0.norm() / launch.w.norm().max(1e-9))
}

/// The spin part of [`believed_flight`].
pub fn believed_spin(env: &FlightEnv, ve: Vec3, off_bat_since: Option<f64>) -> Vec3 {
    believed_flight(env, ve, off_bat_since).1
}

/// A ball going `ve` now with spin `w`, flown backward `s` seconds with the observer's own
/// physics: its state then, relative to where it is now (`r` is the displacement back).
fn fly_back(env: &FlightEnv, ve: Vec3, w: Vec3, s: f64) -> BallState {
    let mut st = BallState { r: Vec3::ZERO, v: ve, w };
    let mut left = s;
    while left > 1e-9 {
        let h = left.min(AGENT_DT);
        st = env.step(&st, -h);
        left -= h;
    }
    st
}

/// Where a ball going `ve` now (spin `w`) must be if it left `p0` `s` seconds ago: its
/// velocity flown backward through those seconds with the observer's own physics (drag and
/// gravity act on the velocity alone, not on where the ball is), adding up the distance.
/// The knowledge that a batted ball started at the plate (agent-models §5.1).
pub fn position_from_plate(env: &FlightEnv, p0: Vec3, ve: Vec3, w: Vec3, s: f64) -> Vec3 {
    p0 - fly_back(env, ve, w, s).r
}

/// An observer's error covariance for a read of a ball's position or velocity whose σ per
/// lateral axis is `sigma` (agent-models §5.1): `depth`× that along his line of sight `los`
/// (horizontal unit), and across it one draw that moves the read sideways by σ and up by σ/2
/// together, as [`read_prediction`] samples it (Codex plate-cue review #2: lateral and height
/// errors were first taken as independent).
fn read_cov(los: Vec3, sigma: f64, depth: f64) -> Mat3 {
    let across = Vec3::new(-los.y, los.x, 0.5);
    let sd = depth * sigma;
    Mat3::outer(los, los) * (sd * sd) + Mat3::outer(across, across) * (sigma * sigma)
}

/// For a ball still on its first flight off the bat `s` seconds ago, how an observer weighs
/// the two things he knows about where it is now (agent-models §5.1): his direct read
/// (position σ `sp` per lateral axis) and where a ball going as fast as he reads it (σ `sv`)
/// must be, having left the plate `s` seconds ago ([`position_from_plate`]; that error grows
/// as `s` times the velocity error, plus the spin he cannot see). Returns the gain applied to
/// (plate − direct), his remaining depth σ along `los`, and the depth gain (how much of his
/// depth estimate comes from the plate).
fn plate_weighing(los: Vec3, sp: f64, sv: f64, s: f64) -> (Mat3, f64, f64) {
    let r_read = read_cov(los, sp, DEPTH_STRETCH);
    let spin = PLATE_SPIN_M * s * m::sqrt(s);
    let r_plate = read_cov(los, sv * s, DEPTH_STRETCH * 0.5) + Mat3::scale(spin * spin);
    let gain = match (r_read + r_plate).inverse() {
        Some(inv) => r_read * inv,
        None => Mat3::scale(0.0),
    };
    let post = r_read - gain * r_read;
    (gain, m::sqrt((post * los).dot(los).max(0.0)), (gain * los).dot(los))
}

/// How much of an observer's read error is left (agent-models §5.1): his viewing time `view`
/// against `tau_read` shrinks it as 1/√(1 + view/τ_read); for a ball still on its first
/// flight off the bat (`off_bat`), it also closes as the ball comes down: × T/`READ_CLOSE_S`
/// (at most 1), T the time it takes to fall from its height now with its vertical velocity
/// now, in a vacuum (both seen now, nothing of its future).
fn read_shrink(truth: (Vec3, Vec3), view: f64, tau_read: f64, off_bat: Option<(f64, Vec3)>) -> f64 {
    let base = 1.0 / m::sqrt(1.0 + view / tau_read);
    if off_bat.is_none() {
        return base;
    }
    let (p, v) = truth;
    let g = bb_math::units::G;
    let t_down = (v.z + m::sqrt(v.z * v.z + 2.0 * g * p.z.max(0.0))) / g;
    base * (t_down.max(0.0) / READ_CLOSE_S).min(1.0)
}

/// The σ of an observer's read of where the ball comes down along his line of sight, m: a
/// fielder's knowledge of how good his own read is (a belief about himself, not about the
/// ball; agent-models §5.7 calls). He runs his own read ([`read_prediction`]) from what he
/// believes the ball is doing now (`belief` at `t`, seen from `from`), once as it is and once
/// with each of its four error draws at one σ, and adds up in quadrature how far each moves
/// where he predicts it comes down, along his line of sight to it; for a ball still on its
/// first flight, with the spin he cannot see (`PLATE_SPIN_M`) in the plate's share of his
/// depth estimate. Running his own physics carries the drag, the plate cue's use of the same
/// velocity he flies forward, and the coupling between lateral and height draws.
#[allow(clippy::too_many_arguments)]
pub fn landing_depth_sigma(env: &FlightEnv, field: &Field, belief: (Vec3, Vec3), t: f64, from: Vec3, sigma0: f64, vel_frac: f64, tau_read: f64, view: f64, off_bat: Option<(f64, Vec3)>) -> f64 {
    let land = |err: [f64; 4]| {
        let path = read_prediction_k(env, field, belief, t, from, sigma0, vel_frac, tau_read, view, err, off_bat);
        match path.first_bounce() {
            Some((_, x, y)) => Vec3::new(x, y, 0.0),
            None => path.at(path.t_end()).0,
        }
    };
    let (p, v) = belief;
    let los = Vec3::new(p.x - from.x, p.y - from.y, 0.0).normalized();
    let base = land([0.0; 4]);
    let mut var = 0.0;
    for k in 0..4 {
        let mut e = [0.0; 4];
        e[k] = 1.0;
        let d = (land(e) - base).dot(los);
        var += d * d;
    }
    if let Some((s, _)) = off_bat
        && s > 0.0
    {
        let shrink = read_shrink(belief, view, tau_read, off_bat);
        let sp = sigma0 * (m::hypot(p.x - from.x, p.y - from.y).max(1.0) / READ_REF_DIST) * shrink;
        let (_, _, k) = plate_weighing(los, sp, vel_frac * v.norm() * shrink, s);
        let spin = k * PLATE_SPIN_M * s * m::sqrt(s);
        var += spin * spin;
    }
    m::sqrt(var)
}

/// The snapshot read (agent-models §5.1): an observer's prediction from a noisy estimate
/// of its state now (`truth` = position and velocity at `t`; the error grows with distance,
/// shrinks with viewing time `view`, and is larger in depth), flown forward with the
/// observer's own physics on smooth ground with typical spin. `err`: his error draws (depth,
/// lateral, velocity depth, velocity lateral; standard normal). `off_bat`: for a ball still
/// on its first flight off the bat, the time since contact, s, and the contact point (he saw
/// where it was hit).
///
/// For such a ball he also knows it started at the plate (agent-models §5.1): a ball going as
/// fast as he reads it, having left the contact point `s` seconds ago, must be at
/// [`position_from_plate`]. Early on that is far better than his direct read of where it is
/// (a ball 16 m from home is not read 22 m deep), and its error is his velocity read's,
/// times `s`; he weighs the two by how good each is ([`plate_weighing`]), so as the ball
/// comes nearer his direct read takes over.
#[allow(clippy::too_many_arguments)]
pub fn read_prediction(env: &FlightEnv, field: &Field, truth: (Vec3, Vec3), t: f64, from: Vec3, sigma0: f64, tau_read: f64, view: f64, err: [f64; 4], off_bat: Option<(f64, Vec3)>) -> BallPath {
    read_prediction_k(env, field, truth, t, from, sigma0, READ_VEL_FRAC, tau_read, view, err, off_bat)
}

/// Perturb position and velocity along and across the observer's line of sight.
fn noisy_ball_motion((p, v): (Vec3, Vec3), los: Vec3, sp: f64, sv: f64, err: [f64; 4]) -> (Vec3, Vec3) {
    let lat = Vec3::new(-los.y, los.x, 0.0);
    // Height is read with the lateral precision of the angular measurement.
    let pe = p + los * (err[0] * sp * DEPTH_STRETCH) + lat * (err[1] * sp) + Vec3::new(0.0, 0.0, err[1] * sp * 0.5);
    let ve = v + los * (err[2] * sv * DEPTH_STRETCH * 0.5) + lat * (err[3] * sv) + Vec3::new(0.0, 0.0, err[3] * sv * 0.5);
    (pe, ve)
}

/// [`read_prediction`] by an observer whose first read of the ball's velocity is off by
/// `vel_frac` of its speed per axis (a fielder's own read rating, agent-models §10).
#[allow(clippy::too_many_arguments)]
pub fn read_prediction_k(env: &FlightEnv, field: &Field, truth: (Vec3, Vec3), t: f64, from: Vec3, sigma0: f64, vel_frac: f64, tau_read: f64, view: f64, err: [f64; 4], off_bat: Option<(f64, Vec3)>) -> BallPath {
    let (p, v) = truth;
    let dist = m::hypot(p.x - from.x, p.y - from.y).max(1.0);
    let shrink = read_shrink(truth, view, tau_read, off_bat);
    let sp = sigma0 * (dist / READ_REF_DIST) * shrink;
    let los = Vec3::new(p.x - from.x, p.y - from.y, 0.0).normalized();
    let sv = vel_frac * v.norm() * shrink;
    let (mut pe, ve) = noisy_ball_motion(truth, los, sp, sv, err);
    let off_bat_since = off_bat.map(|(s, _)| s);
    let (env, w) = believed_flight(env, ve, off_bat_since);
    if let Some((s, p0)) = off_bat
        && s > 0.0
    {
        // The plate: where a ball going as fast as he reads it must be, weighed against his
        // direct read by how good each is.
        let (gain, _, _) = plate_weighing(los, sp, sv, s);
        pe += gain * (position_from_plate(&env, p0, ve, w, s) - pe);
    }
    batted_path_dt(&env, field, BallState { r: pe, v: ve, w }, t, AGENT_DT, |_, _| (0.0, 0.0))
}

/// How much farther toward the batter's pull side than his average a ball is hit, by where
/// the pitch is (agent-models §5.10): degrees of spray (+ = pulled more), against the pitch's
/// distance toward the batter from the middle of the plate, m (+ = inside). A fielder's
/// belief about the contact model: means by 10 cm bin from `examples/spray_by_location.rs`
/// (96 games, left-handers mirrored) less the overall mean (−1.23°), linear between
/// bin centres and flat beyond the last. Refit when the contact model changes.
fn anticipated_pull_deg(x_inside: f64) -> f64 {
    const X: [f64; 7] = [-0.30, -0.20, -0.10, 0.0, 0.10, 0.20, 0.30];
    const PULL: [f64; 7] = [-2.8, -0.5, 0.1, 0.6, 0.4, 0.2, -0.7];
    let x = x_inside.clamp(X[0], X[6]);
    let k = (((x - X[0]) / 0.10).floor() as usize).min(5);
    let f = (x - X[k]) / 0.10;
    PULL[k] + (PULL[k + 1] - PULL[k]) * f
}

/// An angle brought into (−π, π].
fn wrap_pi(a: f64) -> f64 {
    use core::f64::consts::{PI, TAU};
    let x = a % TAU;
    if x > PI {
        x - TAU
    } else if x <= -PI {
        x + TAU
    } else {
        x
    }
}

/// Precision-weighted fusion of two direction estimates on the circle (agent-models §5.10):
/// the prior moved by k = σp²/(σp² + σc²) of the shortest angle to the cue, and the posterior
/// σ. Rotating both inputs rotates the estimate by the same amount, so there is no seam at
/// ±180°.
fn fuse_on_circle(prior: f64, sp: f64, cue: f64, sc: f64) -> (f64, f64) {
    let k = sp * sp / (sp * sp + sc * sc);
    (wrap_pi(prior + k * wrap_pi(cue - prior)), sp * sc / m::sqrt(sp * sp + sc * sc))
}

/// The bat's face at contact (rad, batter-relative: 0 square to the pitch line, negative his
/// pull side) for a bat he saw at yaw `seen_yaw` (agent-models §5.10): the yaw less 90° for a
/// right-handed batter, mirrored for a left-hander.
fn face_from_yaw(seen_yaw: f64, bats: bb_observe::Hand) -> f64 {
    let yaw = if bats == bb_observe::Hand::Right { seen_yaw } else { -seen_yaw };
    wrap_pi(yaw - core::f64::consts::FRAC_PI_2)
}

/// What the swing's lead tells a fielder about the bat's face at contact (agent-models §5.10),
/// from his read of it (`seen_lead`, s, read with error σ `read_sigma`): face ≈
/// `LEAD_FACE_A_DEG` + `LEAD_FACE_B_DPS` × lead, with `LEAD_FACE_RESID_DEG` of scatter (rad,
/// batter-relative, unwrapped: a wild read can say more than half a turn, and
/// [`face_spray_cue`] then rejects it).
fn face_from_lead(seen_lead: f64, read_sigma: f64) -> (f64, f64) {
    let b = LEAD_FACE_B_DPS.to_radians();
    let resid = LEAD_FACE_RESID_DEG.to_radians();
    (LEAD_FACE_A_DEG.to_radians() + b * seen_lead, m::sqrt(b * b * read_sigma * read_sigma + resid * resid))
}

/// What an estimate of the bat's face at contact (rad, batter-relative, σ `face_sigma`) tells a
/// fielder about the spray (agent-models §5.10): his belief (rad, + toward right field) and
/// its σ. An estimate that puts the ball outside the forward half (more than 90° from
/// straightaway) is off his model: `None`, and he discards that read.
fn face_spray_cue(face: f64, face_sigma: f64, bats: bb_observe::Hand) -> Option<(f64, f64)> {
    // Unwrapped on purpose: a face beyond ±180° (a wild lead read) must not wrap back into
    // the forward half. A yaw read's face is within ±180°, so its spray is within ±250°.
    let spray_r = SWING_SPRAY_A_DEG.to_radians() + SWING_SPRAY_B * face;
    if spray_r.abs() > core::f64::consts::FRAC_PI_2 {
        return None;
    }
    let spray = if bats == bb_observe::Hand::Right { spray_r } else { -spray_r };
    let resid = SWING_SPRAY_RESID_DEG.to_radians();
    let sigma = m::sqrt(SWING_SPRAY_B * SWING_SPRAY_B * face_sigma * face_sigma + resid * resid);
    Some((spray, sigma))
}

/// What the bat's angle at contact alone tells a fielder (agent-models §5.10), from his read of
/// it (`seen_yaw`, rad, read with error σ `read_sigma`): [`face_spray_cue`] of its face.
#[cfg(test)]
fn swing_spray_cue(seen_yaw: f64, bats: bb_observe::Hand, read_sigma: f64) -> Option<(f64, f64)> {
    face_spray_cue(face_from_yaw(seen_yaw, bats), read_sigma, bats)
}

/// A fielder's own reads at contact (agent-models §5.10), all with his error already in
/// them: where he saw the pitch cross (sim x, m), the bat's angle he saw at contact (rad),
/// the swing's lead he timed (s; both `None` without a swing to see), and the batter's side
/// (public).
struct JumpReads {
    pitch_x: f64,
    bat_yaw: Option<f64>,
    lead: Option<f64>,
    bats: bb_observe::Hand,
}

/// A fielder's jump (agent-models §5.10): the velocity he goes for between contact and the
/// end of his reaction time, before he has read the ball itself, decided from his own reads
/// alone. The pitch location and the batter's side give a prior on the ball's direction (the
/// batter's typical pull plus [`anticipated_pull_deg`], scattered by `SPRAY_SCATTER_DEG`). The
/// bat's angle at contact ([`face_from_yaw`]) and the swing's lead ([`face_from_lead`]) are two
/// reads of one thing, the bat's face at contact: he fuses them into one estimate of it, maps
/// that once to a spray cue ([`face_spray_cue`]) and fuses the cue with the prior, all on the
/// circle ([`fuse_on_circle`]). A read that by itself puts the ball behind the batter is off
/// his model and he discards it, falling back on his other reads (back-half policy, revised
/// 2026-11-13). If the estimate puts the ball within `JUMP_ZONE_DEG` of his line from home
/// but off to one side, he starts toward that side across his line, committing in proportion
/// to how far off it is against his uncertainty (fully at `JUMP_FULL_SIGMAS`). A ball at him,
/// or well away from him, and he stays set. The pitcher and catcher do not jump.
fn jump_from_reads(reads: &JumpReads, a: &FielderAttrs, i: usize, home: Vec3) -> Vec3 {
    if i < 2 || a.jump_gain <= 0.0 {
        return Vec3::ZERO;
    }
    let right = reads.bats == bb_observe::Hand::Right;
    // A right-handed batter stands on the −x side: inside is −x, and pulling is −spray.
    let pull = (MEAN_PULL_DEG + anticipated_pull_deg(if right { -reads.pitch_x } else { reads.pitch_x })).to_radians();
    let prior = if right { -pull } else { pull };
    let sp = SPRAY_SCATTER_DEG.to_radians();
    // The two reads of the bat's face, each kept only if it alone leaves the ball in front.
    let face_reads = [reads.bat_yaw.map(|y| (face_from_yaw(y, reads.bats), a.swing_sigma)), reads.lead.map(|l| face_from_lead(l, a.lead_sigma))];
    let mut face: Option<(f64, f64)> = None;
    for (f, s) in face_reads.into_iter().flatten() {
        if face_spray_cue(f, s, reads.bats).is_none() {
            continue;
        }
        face = Some(match face {
            None => (f, s),
            Some((f0, s0)) => fuse_on_circle(f0, s0, f, s),
        });
    }
    let (est, s_post) = match face.and_then(|(f, s)| face_spray_cue(f, s, reads.bats)) {
        Some((cue, sc)) => fuse_on_circle(prior, sp, cue, sc),
        None => (prior, sp),
    };
    let phi = m::atan2(home.x, home.y);
    let off = wrap_pi(est - phi);
    let zone = if i >= 6 { JUMP_ZONE_DEG[1] } else { JUMP_ZONE_DEG[0] }.to_radians();
    if off.abs() > zone {
        return Vec3::ZERO;
    }
    let commit = a.jump_gain * (off.abs() / (JUMP_FULL_SIGMAS * s_post)).min(1.0);
    // Toward larger spray angle (toward right field) across his line from home.
    let across = Vec3::new(m::cos(phi), -m::sin(phi), 0.0);
    across * (off.signum() * a.speed * commit)
}

/// The orchestrator's side of a fielder's jump (agent-models §5.10): his reads of the true
/// pitch location (σ `antic_sigma`, purpose `FielderAnticipation` sub-index 0), of the
/// bat's true angle at contact (σ `swing_sigma`, sub-index 1) and of the swing's true lead
/// (σ `lead_sigma`, sub-index 2), handed to his belief ([`jump_from_reads`]). Nothing about
/// the ball after contact reaches him here.
fn jump_at_contact(inp: &LiveInput, i: usize, home: Vec3) -> Vec3 {
    let a = &inp.fattrs[i];
    let Some(px) = inp.pitch_x else { return Vec3::ZERO };
    if i < 2 || a.jump_gain <= 0.0 {
        return Vec3::ZERO;
    }
    let mut r0 = rng(inp.seed, inp.play, inp.fielders[i], Purpose::FielderAnticipation, 0);
    let pitch_x = px + a.antic_sigma * r0.truncated_normal(4.0);
    let bat_yaw = inp.bat_yaw.map(|y| {
        let mut r1 = rng(inp.seed, inp.play, inp.fielders[i], Purpose::FielderAnticipation, 1);
        wrap_pi(y + a.swing_sigma * r1.truncated_normal(4.0))
    });
    let lead = inp.swing_lead.map(|l| {
        let mut r2 = rng(inp.seed, inp.play, inp.fielders[i], Purpose::FielderAnticipation, 2);
        l + a.lead_sigma * r2.truncated_normal(4.0)
    });
    jump_from_reads(&JumpReads { pitch_x, bat_yaw, lead, bats: inp.bats }, a, i, home)
}

/// How far a fielder is still short of `goal` (less `reach`) after `t` seconds of running
/// at it from his current motion (agent-models §5.2: reach from the movement model, not a
/// straight line). In the goal's frame his velocity is u toward it and w across. With no
/// sideways speed it is the straight-line law ([`run_dist_from`]: the speed law, or a stop
/// first when moving away). With sideways speed, the movement step first turns him: the
/// sideways speed brakes off at [`BRAKE`] (taking w/BRAKE and drifting w²/(2·BRAKE)) while
/// his speed toward the goal grows at his lateral acceleration (the turn), up to top speed;
/// then the straight-line law from the end of the turn over the remaining distance, drift
/// included. Checked against the stepped movement in `reach_estimate_tracks_the_movement_step`.
///
/// Within `STEP_REACH_M` of the goal, moving, he is stepped with the movement step itself
/// ([`step_toward`], as a late chaser) until he is within reach, for at most `STEP_REACH_CAP_S`;
/// the closed form only from there (agent-models §5.2; Opus wall-braking review P3-1: the
/// closed form turns in the goal's frame and the step brakes along the velocity, so a man
/// close to the ball and moving away from it was planned up to 0.42 s early; Codex
/// tracking-read review P2-3: switching to the closed form once he was heading for it, close
/// to it, let its sideways drift turn him round again).
fn run_gap(field: &Field, fd: &Fielder, goal: Vec3, t: f64, reach: f64) -> f64 {
    if stepped_reach(fd, goal) && t > 0.0 {
        let (tt, pos, vel, reached) = step_short(field, fd, goal, reach, t);
        let left = m::hypot(goal.x - pos.x, goal.y - pos.y);
        // Within reach by `t` (he has got there), or stepped to his time: how far short then.
        if reached || tt >= t - 1e-12 {
            return left - reach;
        }
        return closed_gap(pos, vel, &fd.a, goal, t - tt, reach);
    }
    closed_gap(fd.pos, fd.vel, &fd.a, goal, t, reach)
}

/// The walls in a fielder's movement step (agent-models §5.3): he sees the wall and never comes
/// at it faster than he can stop before it at the braking limit (Gemini play-area review
/// P2-3: chasers ran into it at full speed); nobody runs through a wall (physics-spec §9.5):
/// he stops at it, losing the speed he had into it.
fn wall_rule(field: &Field, pos: Vec3, vel: Vec3) -> (Vec3, Vec3) {
    let (mut pos, mut vel) = (pos, vel);
    let w = field.body_clearance(pos.x, pos.y, BODY_R);
    let into = -vel.dot(w.inward);
    let allowed = wall_speed_cap(w.dist);
    if into > allowed {
        vel = vel + w.inward * (into - allowed);
    }
    let w = field.body_clearance(pos.x, pos.y, BODY_R);
    if w.dist < 0.0 {
        pos = field.keep_inside(pos, BODY_R);
        let vn = vel.dot(w.inward);
        if vn < 0.0 {
            vel = vel - w.inward * vn;
        }
    }
    (pos, vel)
}

/// The most a short reach estimate steps the movement, s [E: a 5 m run from any motion is
/// within reach well inside it].
const STEP_REACH_CAP_S: f64 = 2.5;

/// A fielder stepped toward `goal` as a late chaser, walls and all ([`wall_rule`]), until within `reach`, for at most `t` and
/// `STEP_REACH_CAP_S`: the time stepped, where he is then, how he is moving, and whether he
/// got within reach.
fn step_short(field: &Field, fd: &Fielder, goal: Vec3, reach: f64, t: f64) -> (f64, Vec3, Vec3, bool) {
    let (mut pos, mut vel, mut tt) = (fd.pos, fd.vel, 0.0);
    while tt < t.min(STEP_REACH_CAP_S) - 1e-12 {
        if m::hypot(goal.x - pos.x, goal.y - pos.y) <= reach {
            return (tt, pos, vel, true);
        }
        (pos, vel) = step_toward(pos, vel, goal, &fd.a, false, 0.0);
        (pos, vel) = wall_rule(field, pos, vel);
        tt += DT;
    }
    let reached = m::hypot(goal.x - pos.x, goal.y - pos.y) <= reach;
    (tt, pos, vel, reached)
}

/// Whether [`run_gap`] steps the movement for this run (and [`wall_delay`] adds nothing: the
/// walls are in the step): within `STEP_REACH_M`, moving.
fn stepped_reach(fd: &Fielder, goal: Vec3) -> bool {
    m::hypot(goal.x - fd.pos.x, goal.y - fd.pos.y) < STEP_REACH_M && fd.vel.norm() > 1e-6
}

/// [`run_gap`]'s closed form, from a body at `pos` moving `vel` with abilities `a`.
fn closed_gap(pos: Vec3, vel: Vec3, a: &FielderAttrs, goal: Vec3, t: f64, reach: f64) -> f64 {
    let to = Vec3::new(goal.x - pos.x, goal.y - pos.y, 0.0);
    let d = to.norm();
    let (u, w) = if d < 1e-9 { (0.0, vel.norm()) } else {
        let u = vel.dot(to) / d;
        (u, (vel - to * (u / d)).norm())
    };
    let (vmax, tau, a1) = (a.speed, a.tau_a, a.a_lat);
    if w < 1e-6 {
        return d - reach - run_dist_from(t, u, vmax, tau);
    }
    // Toward the goal during the turn: u + a1·t, held at top speed once reached.
    let toward = |t: f64| {
        let t_cap = if u < vmax { (vmax - u) / a1 } else { 0.0 };
        if t <= t_cap { u * t + 0.5 * a1 * t * t } else { u * t_cap + 0.5 * a1 * t_cap * t_cap + vmax.max(u) * (t - t_cap) }
    };
    let t_turn = w / BRAKE;
    if t <= t_turn {
        let across = w * t - 0.5 * BRAKE * t * t;
        return m::hypot((d - toward(t)).max(0.0), across) - reach;
    }
    let u_end = (u + a1 * t_turn).min(vmax.max(u));
    let left = m::hypot((d - toward(t_turn)).max(0.0), w * w / (2.0 * BRAKE));
    left - reach - run_dist_from(t - t_turn, u_end, vmax, tau)
}

/// The fastest a fielder may come at a wall `dist` away (his body's clearance), m/s: what he
/// can still brake to the contact speed from, `sqrt(v_c² + 2·BRAKE·dist)` (agent-models §5.3).
fn wall_speed_cap(dist: f64) -> f64 {
    m::sqrt(WALL_CONTACT_SPEED_MPS * WALL_CONTACT_SPEED_MPS + 2.0 * BRAKE * dist.max(0.0))
}

/// Seconds the wall costs a fielder running at `goal`, to be added to his unbraked run. He
/// sees the wall and knows what `move_fielders` will do: his speed into it is capped at
/// [`wall_speed_cap`] of his body's clearance, so a stand point (`reach` short of `goal`) near
/// the wall is reached late. Into a wall at normal speed v with clearance c at the stand point
/// (v_c the contact speed), the cap bites from (v² − v_c²)/(2·BRAKE) − c out, and the time lost
/// against running on is (v − √(v_c² + 2·BRAKE·c))/BRAKE − ((v² − v_c²)/(2·BRAKE) − c)/v
/// (at the wall itself, c = 0: (v − v_c)/BRAKE − (v² − v_c²)/(2·BRAKE·v), 0.47 s at 8 m/s). The
/// speed is the normal component of his top speed along the line to the stand point, held
/// to what he can build up over the distance he has (a late start is not braked from 8 m/s);
/// only the normal component is capped, so a run along the wall loses nothing. Checked
/// against the stepped movement in `the_plan_includes_the_slow_down_for_the_wall`.
fn wall_delay(field: &Field, fd: &Fielder, goal: Vec3, reach: f64) -> f64 {
    let to = Vec3::new(goal.x - fd.pos.x, goal.y - fd.pos.y, 0.0);
    let d = to.norm();
    if d <= reach + 1e-9 || stepped_reach(fd, goal) {
        return 0.0;
    }
    let dir = to * (1.0 / d);
    let stand = Vec3::new(goal.x - dir.x * reach, goal.y - dir.y * reach, 0.0);
    let w = field.body_clearance(stand.x, stand.y, BODY_R);
    let cos = -dir.dot(w.inward);
    if cos <= 0.0 {
        return 0.0;
    }
    // Speed into the wall: top speed, or what he builds over the way (initial acceleration
    // v_max/τ) if that is less, but never less than he already carries into it.
    let carried = (-fd.vel.dot(w.inward)).max(0.0);
    let built = m::sqrt(2.0 * fd.a.speed / fd.a.tau_a * (d - reach) * cos);
    let v = (fd.a.speed * cos).min(built.max(carried));
    let c = w.dist.max(0.0);
    let vc2 = WALL_CONTACT_SPEED_MPS * WALL_CONTACT_SPEED_MPS;
    let bites_at = (v * v - vc2) / (2.0 * BRAKE);
    if v <= 0.0 || c >= bites_at {
        return 0.0;
    }
    (v - wall_speed_cap(c)) / BRAKE - (bites_at - c) / v
}

/// Highest ball a fielder can take where it is: he can leap at a wall (agent-models §5.4).
fn catch_ceiling(field: &Field, p: Vec3) -> f64 {
    catch_ceiling_at_clearance(field.body_clearance(p.x, p.y, 0.0).dist)
}

fn catch_ceiling_at_clearance(clearance: f64) -> f64 {
    if clearance < WALL_CATCH_ZONE_M { WALL_CATCH_MAX_Z } else { CATCH_MAX_Z }
}

/// Time for a fielder to come within `reach` of `goal` from his current motion: the first
/// zero of [`run_gap`].
///
/// The first crossing: the gap is not monotone in time for a man turning round close to his
/// goal (he can be within reach, run on, and come back), so the first three seconds are
/// scanned at `RUN_SCAN_S` before the bracket doubles (Codex tracking-read review P2-3).
fn run_time_to(field: &Field, fd: &Fielder, goal: Vec3, reach: f64) -> f64 {
    const RUN_SCAN_S: f64 = 0.05;
    if run_gap(field, fd, goal, 0.0, reach) <= 0.0 {
        return 0.0;
    }
    // A short run from motion: the first tick he is within reach, stepped.
    if stepped_reach(fd, goal) {
        let (tt, _, _, reached) = step_short(field, fd, goal, reach, STEP_REACH_CAP_S);
        if reached {
            return tt;
        }
    }
    let (mut lo, mut hi) = (0.0, RUN_SCAN_S);
    while run_gap(field, fd, goal, hi, reach) > 0.0 && hi < 3.0 {
        lo = hi;
        hi += RUN_SCAN_S;
    }
    while run_gap(field, fd, goal, hi, reach) > 0.0 && hi < 120.0 {
        lo = hi;
        hi *= 2.0;
    }
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if run_gap(field, fd, goal, mid, reach) > 0.0 { lo = mid } else { hi = mid }
    }
    hi
}

/// The glove σ multiplier (agent-models §5.4 form) for a ball at speed `v` taken `horiz`
/// from the body. k_v = 0.25 and k_reach = 0.3 [E] fit the routine-play catch band.
/// Off his feet or off balance
/// (`extending`) the glove is harder to place, more so the further out (`K_EXTEND`).
fn glove_sigma_mult(v: f64, horiz: f64, extending: bool) -> f64 {
    1.0 + 0.25 * v / 40.0 + 0.3 * (horiz / (REACH + DIVE)).min(1.0) + if extending { 1.0 + K_EXTEND * ((horiz - REACH) / DIVE).clamp(0.0, 1.0) } else { 0.0 }
}

/// A fielder's position and velocity after the movement step of the tick at time `t`
/// (agent-models §5.3). His velocity goes toward a wanted velocity: top speed toward his
/// target, or the braking curve that stops him on the spot (a chaser late for the ball runs
/// through). Speeding up along his direction of motion follows the speed law (time constant
/// τ); slowing takes at most [`BRAKE`] and turning at most his lateral acceleration `a_lat`.
/// So a reversal means a full stop, and a sharp turn at speed a curve.
/// Not yet reacting, or on the ground after a dive: unchanged.
/// How far past a wall a ball at `b` is beyond a fielder's reach over it: his body stops its
/// radius inside the wall and his glove reaches `REACH` from his body's centre (> 0: out of
/// reach; physics-spec §9.5). Players see the walls, so plans use it, and so does capture:
/// one limit, [`OVER_WALL_REACH_M`], for a catch on his feet, a stretch and a dive.
fn beyond_reach_over_wall(field: &Field, b: Vec3) -> f64 {
    -field.body_clearance(b.x, b.y, 0.0).dist - OVER_WALL_REACH_M
}

fn fielder_step(fd: &Fielder, t: f64, t_contact: f64) -> (Vec3, Vec3) {
    if fd.dove && t < fd.ready_at {
        // Down from a dive: the lunge carries him toward the ball, then he lies still.
        return match fd.lunge {
            Some((t0, a, b)) if t < t0 + DIVE_LUNGE_S => {
                let u = ((t + DT - t0) / DIVE_LUNGE_S).min(1.0);
                (a + (b - a) * u, (b - a) * (1.0 / DIVE_LUNGE_S))
            }
            Some((_, _, b)) => (b, Vec3::ZERO),
            None => (fd.pos, Vec3::ZERO),
        };
    }
    // Until he has read the batted ball he goes with his jump (agent-models §5.10: the speed
    // law toward it from contact); with none, he stays set.
    if t < t_contact + fd.a.reaction {
        if t < t_contact || (fd.jump == Vec3::ZERO && fd.vel == Vec3::ZERO) {
            return (fd.pos, fd.vel);
        }
        let vel = fd.vel + (fd.jump - fd.vel) * (DT / fd.a.tau_a);
        return (fd.pos + vel * DT, vel);
    }
    let to = Vec3::new(fd.target.x - fd.pos.x, fd.target.y - fd.pos.y, 0.0);
    let d = to.norm();
    let along = if d < 1e-9 { 0.0 } else { fd.vel.dot(to) / d };
    // A chaser fields the ball on the run: his intercept plan (`run_dist`) has him arriving at
    // speed, so he slows for the spot only if he will be there before the ball. His time to
    // it: at the speed he has toward it, or, slower than that, from rest by the speed law
    // (he does not take a man standing 2 m short to be 4 s away).
    let eta = (d / along.max(0.5)).min(m::sqrt(2.0 * d * fd.a.tau_a / fd.a.speed) + d / fd.a.speed);
    let early = fd.role != Role::Chase || !fd.t_int.is_finite() || t + eta + 0.1 < fd.t_int;
    // Early for a ball in the air, he keeps a margin of his doubt short of where he can stop
    // (agent-models §5.2, arriving under control).
    let margin = if early && fd.role == Role::Chase { ARRIVE_SIGMAS * fd.doubt } else { 0.0 };
    step_toward(fd.pos, fd.vel, fd.target, &fd.a, early, margin)
}

/// The movement step toward `target` for a body at `pos` moving `vel` with abilities `a`
/// (agent-models §5.3): `early`, the speed that stops him `margin` short of it at `BRAKE`;
/// otherwise his top speed; either held to the turn he can make.
fn step_toward(pos: Vec3, vel: Vec3, target: Vec3, a: &FielderAttrs, early: bool, margin: f64) -> (Vec3, Vec3) {
    let to = Vec3::new(target.x - pos.x, target.y - pos.y, 0.0);
    let d = to.norm();
    // Within a few centimetres and nearly stopped, he is on his spot. Moving faster, he
    // carries on (Codex momentum review #1: arriving never stops him dead).
    if d < 0.05 && vel.norm() < SETTLE_V {
        return (Vec3::new(target.x, target.y, pos.z), Vec3::ZERO);
    }
    if d < 1e-9 {
        return (pos + vel * DT, vel);
    }
    let dir = to * (1.0 / d);
    let mut want = if early { a.speed.min(m::sqrt(2.0 * BRAKE * (d - margin).max(0.0))) } else { a.speed };
    // He slows into a turn he cannot make at speed: reaching a spot d away at an angle θ off
    // his heading takes a curve of radius d/(2 sin θ), which his lateral acceleration allows
    // up to √(a_lat·d/(2 sin θ)); a spot behind him, a stop first.
    let speed_now = vel.norm();
    if speed_now > 1e-6 {
        let cos_t = vel.dot(dir) / speed_now;
        let sin_t = if cos_t >= 0.0 { m::sqrt((1.0 - cos_t * cos_t).max(0.0)) } else { 1.0 };
        if sin_t > 1e-6 {
            want = want.min(m::sqrt(a.a_lat * d / (2.0 * sin_t)));
        }
    }
    // Speeding up in his direction of motion follows the speed law; slowing (braking at most
    // `BRAKE`) and turning (at most `a_lat`) take what the change needs, up to those limits.
    let delta = dir * want - vel;
    let speed = vel.norm();
    let acc = if speed < 1e-6 {
        delta * (1.0 / a.tau_a)
    } else {
        let vh = vel * (1.0 / speed);
        let dpar = delta.dot(vh);
        let dperp = delta - vh * dpar;
        let par = if dpar >= 0.0 { dpar / a.tau_a } else { -(BRAKE.min(-dpar / DT)) };
        let np = dperp.norm();
        let perp = if np < 1e-12 { Vec3::ZERO } else { dperp * (a.a_lat.min(np / DT) / np) };
        vh * par + perp
    };
    let vel2 = vel + acc * DT;
    // Reaching the target ends the run: a fielder standing on his spot has no speed to carry
    // into the next plan.
    if vel2.dot(dir) * DT >= d && vel2.norm() < SETTLE_V {
        return (Vec3::new(target.x, target.y, pos.z), Vec3::ZERO);
    }
    (pos + vel2 * DT, vel2)
}

struct Sim<'a> {
    inp: &'a LiveInput<'a>,
    /// Latest fact time emitted (facts never go backward, rules-spec §1.2).
    last_fact_t: f64,
    /// When the ball was last caught (a visible event).
    secured_at: f64,
    /// Public catch and no-catch ruling times. The rules take effect on the fact;
    /// a runner who is still reading it carries his movement through his reaction.
    caught_at: Option<f64>,
    no_catch_at: Option<f64>,
    /// The first glove possession with no ground or wall touch ruled before it: a possible
    /// catch, visible to every runner, whose call he waits for whatever the ruling is.
    flight_secured_at: Option<f64>,
    /// The latest catch was low or diving: a close call for the umpire.
    low_catch: bool,
    st: &'a mut RulesState,
    t: f64,
    ball: Ball,
    home: [Vec3; 9],
    f: Vec<Fielder>,
    r: Vec<Runner>,
    /// When the ball now loose became loose: calls made before it do not rank (a new ball, a
    /// new call; `set_free`).
    ball_since: f64,
    /// Counts the loose balls of this play (`set_free`): which ball a read was of.
    ball_gen: u64,
    /// `public_rank` for all nine at (time, ball, positions).
    rank_cache: std::cell::RefCell<Option<(f64, u64, [Vec3; 9], [Vec3; 9], [f64; 9])>>,
    /// What each fielder's teammates knew of him at the last tick (`hear` acts on changes):
    /// his chase and call, and the job he was seen going to.
    last_known: Vec<(bool, u8, bool)>,
    last_jobs: Vec<(Role, Option<u8>)>,
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
        let t_fact = fact_time(&fact).unwrap_or(self.t);
        if self.caught_at.is_none() && matches!(fact, Input::BallTouched {
            what: Touch::Ground { .. } | Touch::Wall { .. }, ..
        }) {
            self.no_catch_at.get_or_insert(t_fact);
        }
        if matches!(fact, Input::GloveSecured { .. }) && self.no_catch_at.is_none() {
            self.flight_secured_at.get_or_insert(t_fact);
        }
        self.out.facts.push(fact);
        self.absorb(evs, t_fact);
    }

    /// Send every runner still on the base paths to the rules engine as a position sample
    /// (rules-spec §2 `RunnerPosition`), all at this tick's time.
    fn sample_runners(&mut self) {
        for k in 0..self.r.len() {
            let r = &self.r[k];
            if r.out || (r.scored && (r.catch_seen || !self.caught())) {
                continue;
            }
            let p = self.runner_pos(r);
            let sample = RunnerPosition { t: self.t, runner: r.id, x: p.x, y: p.y };
            let evs = step(self.st, &sample);
            self.absorb(evs, self.t);
            if self.done {
                return;
            }
        }
    }

    /// Book the rules engine's events for a fact at `t_fact`: outs (with the umpire's call),
    /// the catch, the end of the play.
    fn absorb(&mut self, evs: Vec<Event>, t_fact: f64) {
        for e in &evs {
            if let Event::RunnerOut { runner, base, kind } = e {
                if let Some(k) = self.r.iter().position(|r| r.id == *runner) {
                    // How close it was: the runner's time short of the base he needed (back
                    // to on an appeal, ahead to on a force or a tag), at his top speed.
                    let r = &self.r[k];
                    let margin_s = match kind {
                        OutKind::FlyOut | OutKind::Strikeout => if self.low_catch { 0.0 } else { f64::INFINITY },
                        OutKind::Force | OutKind::Appeal => (*base as f64 * BASE_L - r.s).abs() / r.vmax.max(1.0),
                        OutKind::Tag => (r.goal as f64 * BASE_L - r.s).abs() / r.vmax.max(1.0),
                    };
                    let at = self.runner_pos(r);
                    self.out.calls.push(crate::umpire::OutCall { t_out: t_fact, runner: *runner, base: *base, margin_s, at });
                    self.r[k].out = true;
                }
            }
            if matches!(e, Event::Catch { .. }) {
                self.caught_at = Some(t_fact);
                for runner in &mut self.r {
                    runner.off_base_at_catch = runner.start > 0
                        && (runner.s - runner.start as f64 * BASE_L).abs() > 1e-6;
                }
            }
        }
        self.out.events.extend(evs);
        if !matches!(self.st.play, PlayState::Live(_)) {
            self.done = true;
        }
    }

    fn ball_at(&self, t: f64) -> (Vec3, Vec3) {
        match &self.ball {
            Ball::Free { path, .. } => path.at(t),
            // In his hand at chest height, or by the turf while he is down from a dive.
            Ball::Held { by } => {
                let fd = &self.f[*by];
                let down = fd.dove && fd.lunge.is_some_and(|(t0, _, _)| t < t0 + DIVE_GETUP_S);
                (fd.pos + Vec3::new(0.0, 0.0, if down { 0.3 } else { 1.2 }), Vec3::ZERO)
            }
            Ball::Thrown { path, .. } => path.at(t),
        }
    }

    /// The fielder's noisy estimate of the ball's current state, then its own prediction
    /// (smooth ground, typical spin for the trajectory) — never the true future.
    fn predict(&self, i: usize) -> BallPath {
        let t0 = prof::enabled().then(std::time::Instant::now);
        let fd = &self.f[i];
        let r = if let Some(tr) = self.track_now(i) {
            // His running estimate, flown on with his own physics (agent-models §5.1).
            self.fly_estimate(tr.r(), tr.v(), tr.x[2])
        } else {
            let view = (self.t - self.inp.t_contact - fd.a.reaction).max(0.0);
            read_prediction_k(self.inp.env, self.inp.field, self.ball_at(self.t), self.t, fd.pos, fd.a.read_sigma0, fd.a.read_vel_frac, fd.a.tau_read, view, fd.err, self.off_bat())
        };
        if let Some(t0) = t0 {
            prof::add(&prof::PRED, t0.elapsed().as_nanos() as u64);
        }
        r
    }

    /// Fielder `i`'s running estimate of the ball, run on to now: while it is on its first
    /// flight off the bat and he has looked at it (agent-models §5.1).
    fn track_now(&self, i: usize) -> Option<Track> {
        let tr = self.f[i].track?;
        if tr.ball != self.ball_gen || tr.looks == 0 || self.off_bat().is_none() {
            return None;
        }
        Some(tr.run_to(self.inp.env, self.t, self.inp.t_contact))
    }

    /// A ball believed at `r` going `v` now, flown on with the observer's own physics, the
    /// spin he believes for it (as the snapshot read is flown) and the extra acceleration `a`
    /// he has learned his model is missing.
    fn fly_estimate(&self, r: Vec3, v: Vec3, a: Vec3) -> BallPath {
        let (env, w) = believed_flight(self.inp.env, v, Some(self.t - self.inp.t_contact));
        batted_path_dt(&FlightEnv { extra: a, ..env }, self.inp.field, BallState { r, v, w }, self.t, AGENT_DT, |_, _| (0.0, 0.0))
    }

    /// Every fielder who has reacted looks at a ball on its first flight off the bat, once
    /// each `LOOK_S` (agent-models §5.1, the tracking read): what he sees of its position and
    /// velocity, good across his line of sight and poor along it, with fresh noise each look
    /// (`FielderLook`) and a persistent part (`TRACK_BIAS_FRAC` of the play's fixed draws). His
    /// first look counts as τ_read of watching: velocity as seen, position where a ball going
    /// that fast must be having left the plate (the plate cue), weighed with what he sees.
    fn take_looks(&mut self) {
        let Some((since, p0)) = self.off_bat() else { return };
        let (t, t_c, ball) = (self.t, self.inp.t_contact, self.ball_gen);
        let env = self.inp.env;
        let (bp, bv) = self.ball_at(t);
        for i in 0..9 {
            let fd = &self.f[i];
            if t < t_c + fd.a.reaction {
                continue;
            }
            let prev = fd.track.filter(|tr| tr.ball == ball);
            if prev.is_some_and(|tr| t < tr.next_look - 1e-9) {
                continue;
            }
            let k = prev.map_or(0, |tr| tr.looks);
            let (d, u, a, b) = sight_frame(fd.pos + Vec3::new(0.0, 0.0, EYE_H), bp);
            // His first look is the snapshot read's first read, its shrink included (for a ball
            // about to come down, the closing of `READ_CLOSE_S`: a grounder before its first
            // bounce is read as sharply as before); later looks, `LOOK_SCALE` of the plain σ.
            let view = (t - t_c - fd.a.reaction).max(0.0);
            let scale = if k == 0 { read_shrink((bp, bv), view, fd.a.tau_read, Some((since, p0))) } else { LOOK_SCALE };
            let mut g = rng(self.inp.seed, self.inp.play, fd.id, Purpose::FielderLook, k);
            let (fresh, bias) = look_noise();
            let e: [f64; 6] = core::array::from_fn(|_| fresh * g.truncated_normal(4.0));
            let (bias, err) = (bias * TRACK_BIAS_FRAC, fd.err);
            let base_p = fd.a.read_sigma0 * d / READ_REF_DIST;
            let sp = base_p * scale;
            // Scale the persistent error by a single look's σ, including the first look's
            // shrink so grounders get the same initial precision as the snapshot read.
            let bias = bias * if k == 0 { scale } else { 1.0 };
            let zp = bp + u * (DEPTH_STRETCH * (sp * e[0] + bias * base_p * err[0])) + a * (sp * e[1] + bias * base_p * err[1]) + b * (sp * e[2]);
            let base_v = fd.a.read_vel_frac * bv.norm();
            let sv = base_v * scale;
            let zv = bv + u * (0.5 * DEPTH_STRETCH * (sv * e[3] + bias * base_v * err[2])) + a * (sv * e[4] + bias * base_v * err[3]) + b * (sv * e[5]);
            // What he expects before this look: his estimate run on to now, or on his first
            // look the velocity as seen and the position a ball going that fast must have,
            // having left the plate `since` ago (the plate cue).
            let (env0, w0) = believed_flight(env, zv, Some(since));
            let prior = prev.map(|tr| tr.run_to(env, t, t_c));
            let (re, ve) = match &prior {
                Some(tr) => (tr.r(), tr.v()),
                None => (position_from_plate(&env0, p0, zv, w0, since), zv),
            };
            // What he knows of his looks (the world draws them about the ball itself).
            let eye = fd.pos + Vec3::new(0.0, 0.0, EYE_H);
            let kb = TRACK_BIAS_FRAC * if k == 0 { scale } else { 1.0 };
            let lp = Look { eye, vel: false, sigma_per_m: fd.a.read_sigma0 / READ_REF_DIST, vel_frac: fd.a.read_vel_frac, scale, kb };
            let lv = Look { vel: true, ..lp };
            let mut tr = match prior {
                Some(mut tr) => {
                    tr.see(&lp, zp);
                    tr.see(&lv, zv);
                    tr
                }
                None => {
                    let c = TRACK_CONTACT_M * TRACK_CONTACT_M;
                    let pa = Mat3::scale(CURVE_PRIOR * CURVE_PRIOR);
                    let s2 = since * since;
                    // His velocity read's error: the fresh part and the persistent part (how
                    // each of its two components moves the look, per σ).
                    let (_, rv) = lv.expect(re, ve, [0.0; 4]);
                    let hv: [Vec3; 2] = core::array::from_fn(|c| {
                        let mut b = [0.0; 4];
                        b[2 + c] = 1.0;
                        lv.expect(re, ve, b).0 - ve
                    });
                    let gg = Mat3::outer(hv[0], hv[0]) + Mat3::outer(hv[1], hv[1]);
                    let rv = rv + gg;
                    let mut p = [[Mat3::ZERO; 3]; 3];
                    // Velocity as seen; position where a ball going that fast must be, having
                    // left the plate `since` ago; the extra acceleration unknown, and what it
                    // has done since contact folded into both.
                    p[1][1] = rv + pa * s2;
                    p[0][1] = rv * since + pa * (0.5 * s2 * since);
                    p[0][0] = rv * s2 + Mat3::scale(c) + pa * (0.25 * s2 * s2);
                    p[0][2] = pa * (0.5 * s2);
                    p[1][2] = pa * since;
                    p[2][2] = pa;
                    for i in 0..3 {
                        for j in 0..i {
                            p[i][j] = p[j][i].transpose();
                        }
                    }
                    let mut tr = Track::new(ball, t, [re, ve, Vec3::ZERO], p);
                    // Both carry his persistent velocity error (his estimate of it is 0, its
                    // error −b): the velocity by `hv`, the position by `hv` × `since`.
                    for (c, h) in hv.iter().enumerate() {
                        let (hc, hr) = (comps(*h * -1.0), comps(*h * -since));
                        for ax in 0..3 {
                            tr.p[3 + ax][TB + 2 + c] = hc[ax];
                            tr.p[TB + 2 + c][3 + ax] = hc[ax];
                            tr.p[ax][TB + 2 + c] = hr[ax];
                            tr.p[TB + 2 + c][ax] = hr[ax];
                        }
                    }
                    tr.see(&lp, zp);
                    tr
                }
            };
            tr.looks = k + 1;
            tr.next_look = t + LOOK_S;
            self.f[i].track = Some(tr);
        }
    }

    /// Fielder `i`'s doubt about where the ball comes down (agent-models §5.1): the σ along his
    /// line of sight from `from`, of his estimate's covariance (position, velocity and the
    /// learned acceleration, with every cross term) carried to the landing point by his own
    /// physics (linearised: each of the nine state components moved and flown; Codex
    /// tracking-read review P2-1). What his persistent error can do is in that covariance
    /// already: the filter carries it as a state, so it needs no term of its own.
    fn track_doubt(&self, tr: &Track, from: Vec3) -> f64 {
        let land_a = |r: Vec3, v: Vec3, acc: Vec3| {
            let p = self.fly_estimate(r, v, acc);
            match p.first_bounce() {
                Some((_, x, y)) => Vec3::new(x, y, 0.0),
                None => {
                    let q = p.at(p.t_end()).0;
                    Vec3::new(q.x, q.y, 0.0)
                }
            }
        };
        let land = |r: Vec3, v: Vec3| land_a(r, v, tr.x[2]);
        let (r, v) = (tr.r(), tr.v());
        let base = land(r, v);
        let los = Vec3::new(r.x - from.x, r.y - from.y, 0.0).normalized();
        let unit = [Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0)];
        let h = 0.5;
        let mut j = [0.0; 9];
        for k in 0..3 {
            j[k] = (land(r + unit[k] * h, v) - base).dot(los) / h;
            j[3 + k] = (land(r, v + unit[k] * h) - base).dot(los) / h;
            j[6 + k] = (land_a(r, v, tr.x[2] + unit[k] * h) - base).dot(los) / h;
        }
        let mut var = 0.0;
        for a_ in 0..9 {
            for b_ in 0..9 {
                var += j[a_] * tr.p[a_][b_] * j[b_];
            }
        }
        m::sqrt(var.max(0.0))
    }

    /// For a ball still on its first flight off the bat: the time since contact and the
    /// contact point. A carom off the wall counts as a deflection (Codex momentum review #6).
    fn off_bat(&self) -> Option<(f64, Vec3)> {
        let off_the_bat = match &self.ball {
            Ball::Free { path, .. } => {
                let t0 = match path.segments.first() { Some(bb_physics::ground::Segment::Flight { tr, .. }) => tr.first().t, _ => f64::NAN };
                (t0 - self.inp.t_contact).abs() < 1e-9 && path.first_bounce().is_none_or(|b| self.t < b.0) && path.walls.first().is_none_or(|w| self.t < w.0)
            }
            _ => false,
        };
        off_the_bat.then(|| (self.t - self.inp.t_contact, self.inp.ball.r))
    }

    /// A runner's read of the ball: a fresh look per decision, keyed by `sub` (see
    /// `Runner::decisions` for the sub-index layout).
    fn runner_ball_errors(&self, k: usize, sub: u32) -> [f64; 4] {
        let mut g = rng(self.inp.seed, self.inp.play, self.r[k].id, Purpose::RunnerPerception, sub);
        [g.truncated_normal(4.0), g.truncated_normal(4.0), g.truncated_normal(4.0), g.truncated_normal(4.0)]
    }

    /// A runner's relook at a loose ball: the same observation boundary as his first look,
    /// with this decision's draws (P1-1 of the catch-read fixes review).
    fn runner_predict(&self, k: usize, sub: u32) -> BallPath {
        self.observe_ball(k, sub).project(self.inp.field, [0.0; 4])
    }

    /// Seconds since contact while the ball is visibly still on its first flight: no
    /// ground or wall touch has been ruled and no glove has touched it. The world's own
    /// path and the true contact point play no part.
    fn public_contact_since(&self) -> Option<f64> {
        (self.no_catch_at.is_none() && self.caught_at.is_none() && self.out.first_handling.is_none())
            .then_some((self.t - self.inp.t_contact).max(0.0))
    }

    /// Earliest pickup on the runner's own projection using typical fielder speed.
    fn believed_field_intercept(&self, pred: &BallPath) -> Option<(f64, Vec3)> {
        let mut t = self.t + 0.05;
        while t <= pred.t_end() {
            let (b, _) = pred.at(t);
            if b.z <= CATCH_MAX_Z && beyond_reach_over_wall(self.inp.field, b) <= 0.0 {
                let reachable = self.f.iter().any(|fd| {
                    let d = m::hypot(b.x - fd.pos.x, b.y - fd.pos.y) - REACH;
                    d <= 0.0 || run_dist(t - self.t, TYPICAL_SPEED, 0.9) >= d
                });
                if reachable {
                    return Some((t, Vec3::new(b.x, b.y, 0.0)));
                }
            }
            t += 0.15;
        }
        None
    }

    /// World boundary: sample the four keyed errors once, then hand off the noisy motion.
    fn observe_contact(&self, k: usize) -> ContactLook {
        self.observe_ball(k, 1)
    }

    /// One look at the loose ball, with the four draws of sub-index `sub`.
    fn observe_ball(&self, k: usize, sub: u32) -> ContactLook {
        let mut look = ContactLook {
            motion: self.ball_at(self.t),
            from: self.runner_pos(&self.r[k]),
            t: self.t,
            view: (self.t - self.inp.t_contact).max(0.0),
            contact_since: self.public_contact_since(),
        };
        let (los, sp, sv) = look.scales(look.motion);
        look.motion = noisy_ball_motion(look.motion, los, sp, sv, self.runner_ball_errors(k, sub));
        look
    }

    fn runner_catch_paths(&self, look: ContactLook) -> [Option<CatchRead>; 8] {
        core::array::from_fn(|i| {
            let mut error = [0.0; 4];
            error[i / 2] = [-RUNNER_CATCH_READ_POINT, RUNNER_CATCH_READ_POINT][i % 2];
            self.runner_catch_read(&look.project(self.inp.field, error))
        })
    }

    fn catch_score(paths: &[Option<CatchRead>; 8]) -> f64 {
        paths.iter().flatten().map(|read| read.score).sum::<f64>() / paths.len() as f64
    }

    #[cfg(test)]
    fn runner_catch_probability(&self, pred: &BallPath) -> f64 {
        self.runner_catch_read(pred).map_or(0.0, |read| read.score)
    }

    /// The best catch chance over a projection's first flight, and where on the ground it
    /// is. At each point it is the chance a fielder's glove gets there, from visible fielder
    /// motion, in spreads of his doubt about the glove (`RUNNER_CATCH_SIGMA_M`), about the
    /// fielder's speed (`FIELDER_SPEED_SPREAD_MPS` over the run) and about the ball's curve
    /// from the spin he cannot see (`CURVE_PRIOR`). The tighter of the height and horizontal
    /// margins sets the score, treating residual glove reach in the two as one error.
    fn runner_catch_read(&self, pred: &BallPath) -> Option<CatchRead> {
        let ground = pred.first_bounce().map_or(f64::INFINITY, |(t, ..)| t);
        let wall = pred.walls.first().map_or(f64::INFINITY, |(t, ..)| *t);
        let mut best: Option<CatchRead> = None;
        let mut t = self.t;
        while t <= pred.t_end() && t < ground.min(wall) {
            let (b, _) = pred.at(t);
            let clearance = self.inp.field.body_clearance(b.x, b.y, 0.0).dist;
            if clearance >= -OVER_WALL_REACH_M {
                let dt = t - self.t;
                let curve_m = 0.5 * CURVE_PRIOR * dt * dt;
                let height_margin = (catch_ceiling_at_clearance(clearance) - b.z) / RUNNER_CATCH_SIGMA_M;
                let mut horizontal_margin = f64::NEG_INFINITY;
                for fd in &self.f {
                    let to = Vec3::new(b.x - fd.pos.x, b.y - fd.pos.y, 0.0);
                    let d = m::hypot(to.x, to.y);
                    let v0 = if d > 1e-9 && d <= STEP_REACH_M { Some(fd.vel.dot(to) / d) } else { None };
                    let travel = |v_max: f64| match v0 {
                        Some(v0) => run_dist_from(dt, v0, v_max, 0.9),
                        None => run_dist(dt, v_max, 0.9),
                    };
                    let typical = travel(TYPICAL_SPEED);
                    // The run is linear in top speed, so this is its exact spread.
                    let speed_m = travel(TYPICAL_SPEED + FIELDER_SPEED_SPREAD_MPS) - typical;
                    let sigma = m::sqrt(RUNNER_CATCH_SIGMA_M * RUNNER_CATCH_SIGMA_M + speed_m * speed_m + curve_m * curve_m);
                    horizontal_margin = horizontal_margin.max((REACH + typical - d) / sigma);
                }
                let score = phi(horizontal_margin.min(height_margin));
                if score > 0.0 && best.is_none_or(|read| score > read.score) {
                    best = Some(CatchRead { score, t, at: Vec3::new(b.x, b.y, 0.0) });
                }
            }
            t += DT;
        }
        best
    }

    fn catch_hold_s(&self, k: usize, paths: &[Option<CatchRead>]) -> Option<f64> {
        let r = &self.r[k];
        let bag_s = r.start as f64 * BASE_L;
        let off = r.s - bag_s;
        let speed = r.v * r.direction;
        let bag = base_pos(r.start);
        let adv = &self.inp.advance;
        let caution = adv.caution_z * adv.sigma_race_s;
        let far = (lead_s(r.start) + HALFWAY_M).min((r.start as f64 + 0.6) * BASE_L);
        let plausible = paths.iter().flatten().filter(|read| read.score >= 1.0 - RUNNER_CATCH_CONFIDENCE);
        if r.start == 3 {
            let tag = self.inp.tag_up;
            let to_home = tag_up_time(tag, tag.react_s, BASE_L, r.vmax, r.tau);
            let to_bag = run_time_to_stop(off, -speed, r.vmax, r.tau);
            if plausible.clone().any(|read| {
                read.t - self.t >= to_bag
                    && TYPICAL_TRANSFER + typical_throw_time(read.at.norm(), self.cut_seen(None, read.at, 4)) + tag.typical_tag_s >= to_home
            }) {
                return None;
            }
        }

        let races: Vec<_> = plausible.map(|read| {
            let appeal = TYPICAL_TRANSFER + typical_throw_time((read.at - bag).norm(), self.cut_seen(None, read.at, r.start));
            (read.t, appeal - caution - RUNNER_CATCH_CALL_REACT_S)
        }).collect();
        if races.is_empty() {
            return None;
        }
        let recoverable = |target: f64| races.iter().all(|&(catch, budget)| {
            let mut s = r.s;
            let mut v = speed;
            // Carry the intended hold through the catch and the reaction. The first
            // step in the return then uses whatever signed speed remains.
            let mut t = self.t;
            let react = catch + RUNNER_CATCH_CALL_REACT_S;
            while t < react - 1e-9 && (s != target || v != 0.0) {
                (s, v) = step_to_stop(s, v, target, r.vmax, r.tau);
                t += DT;
            }
            run_time_to_stop(s - bag_s, -v, r.vmax, r.tau) <= budget
        });

        if recoverable(far) {
            return Some(far);
        }
        if !recoverable(bag_s) {
            return None;
        }
        let (mut lo, mut hi) = (bag_s, far);
        for _ in 0..20 {
            let mid = 0.5 * (lo + hi);
            if recoverable(mid) { lo = mid } else { hi = mid }
        }
        (lo > bag_s + 1e-6).then_some(lo)
    }

    /// The catch score at which runner `k` goes back to tag up (agent-models §6.3, "What he
    /// does"): `RUNNER_TAG_FROM_SECOND` by the outs from second, `RUNNER_CATCH_CONFIDENCE`
    /// otherwise. Outs are those before the catch, as he counts them; with two out he never reads
    /// the catch (he runs on contact), and the none-out value would be moot.
    fn tag_confidence(&self, k: usize) -> f64 {
        if self.r[k].start == 2 {
            RUNNER_TAG_FROM_SECOND[usize::from(self.runner_outs(k).min(1))]
        } else {
            RUNNER_CATCH_CONFIDENCE
        }
    }

    fn read_contact(&mut self, k: usize, look: ContactLook) {
        let paths = self.runner_catch_paths(look);
        let score = Self::catch_score(&paths);
        if score >= self.tag_confidence(k) {
            self.r[k].tag_up = true;
        } else if score > 1.0 - RUNNER_CATCH_CONFIDENCE {
            self.set_catch_hold(k, &paths);
            if self.r[k].hold_s.is_some() {
                self.r[k].fly_look = Some((self.t + RUNNER_FLY_LOOK_S, 0));
            }
        } else {
            let pred = look.project(self.inp.field, [0.0; 4]);
            self.runner_decide_with_contact(k, self.r[k].start, Some(&pred));
            if self.r[k].goal > self.r[k].start {
                self.r[k].fly_look = Some((self.t + RUNNER_FLY_LOOK_S, 0));
            }
        }
    }

    /// A runner who went at his first look on a ball in the air, or holds partway, looks at it
    /// again (agent-models §6.3, the in-flight relook): through the same observation as his
    /// first look, with look `n`'s own draws. Going on, a catch he is sure of sends him back to
    /// tag up, a close one pulls him up into a hold he can get back from, and otherwise he keeps
    /// going and looks again. Holding partway, a catch he is sure of sends him back to tag up if
    /// he can be on the bag before it; otherwise he holds and looks again. Once the ball is seen
    /// to come down or be touched, or he has stopped going on or holding, he stops looking.
    fn fly_relook(&mut self, k: usize, n: u32) {
        let r = &self.r[k];
        let free = !r.tag_up && !r.retouch && !r.scored && !r.out;
        let going = free && r.goal > r.start && r.hold_s.is_none();
        let holding = free && r.hold_s.is_some();
        if !(going || holding) || self.public_contact_since().is_none() || !matches!(self.ball, Ball::Free { .. }) {
            self.r[k].fly_look = None;
            return;
        }
        let look = self.observe_ball(k, FLY_LOOK_SUB + n);
        let paths = self.runner_catch_paths(look);
        let score = Self::catch_score(&paths);
        let next = Some((self.t + RUNNER_FLY_LOOK_S, n + 1));
        if holding {
            if score >= self.tag_confidence(k) {
                // Back on the bag before the earliest catch he reads as plausible.
                let r = &self.r[k];
                let to_bag = run_time_to_stop(r.s - r.start as f64 * BASE_L, -(r.v * r.direction), r.vmax, r.tau);
                let catch = paths.iter().flatten().filter(|read| read.score > 1.0 - RUNNER_CATCH_CONFIDENCE).map(|read| read.t).fold(f64::INFINITY, f64::min);
                if self.t + to_bag <= catch {
                    let r = &mut self.r[k];
                    r.fly_look = None;
                    r.hold_s = None;
                    r.tag_up = true;
                    return;
                }
            }
            self.r[k].fly_look = next;
            return;
        }
        if score >= self.tag_confidence(k) {
            let r = &mut self.r[k];
            r.fly_look = None;
            r.tag_up = true;
            r.waiting = false;
            r.turned_back = None;
        } else if score > 1.0 - RUNNER_CATCH_CONFIDENCE {
            self.r[k].waiting = false;
            self.r[k].turned_back = None;
            self.set_catch_hold(k, &paths);
            self.r[k].fly_look = if self.r[k].hold_s.is_some() { next } else { None };
        } else {
            self.r[k].fly_look = next;
        }
    }

    fn set_catch_hold(&mut self, k: usize, paths: &[Option<CatchRead>]) {
        let hold = self.catch_hold_s(k, paths);
        self.r[k].hold_s = hold;
        if hold.is_none() {
            self.r[k].tag_up = true;
        }
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
        let (fd, start) = self.planning_state(i);
        let t_end = pred.t_end();
        let (rest, _) = pred.at(t_end);
        (t_end.max(start + run_time_to(self.inp.field, &fd, rest, 0.0)), Vec3::new(rest.x, rest.y, 0.0))
    }

    /// Where a fielder's plans start from (agent-models §5.10, Codex anticipation review #2):
    /// once his reaction gate has opened, his state now; before it, the state his jump will
    /// have carried him to, stepped tick by tick with the movement step itself (position and
    /// velocity) up to the first tick at or after his gate. The start time is the time of
    /// that state: now, or that tick (not the gate itself, which for a reaction off the tick
    /// grid lies up to a tick earlier; Codex coord review #1).
    fn planning_state(&self, i: usize) -> (std::borrow::Cow<'_, Fielder>, f64) {
        let fd = &self.f[i];
        let t_c = self.inp.t_contact;
        let gate = t_c + fd.a.reaction;
        if self.t >= gate || (fd.jump == Vec3::ZERO && fd.vel == Vec3::ZERO) {
            return (std::borrow::Cow::Borrowed(fd), self.t.max(gate));
        }
        let mut g = fd.clone();
        g.belief = None;
        let mut t = self.t;
        while t < gate {
            (g.pos, g.vel) = fielder_step(&g, t, t_c);
            t += DT;
        }
        (std::borrow::Cow::Owned(g), t)
    }

    /// One scan of the predicted path for the earliest point the fielder can reach standing,
    /// and the earliest he can reach with a dive (found on the way; used only without a
    /// standing play).
    #[allow(clippy::type_complexity)]
    fn intercept_scan(&self, i: usize, pred: &BallPath) -> (Option<(f64, Vec3)>, Option<(f64, Vec3)>) {
        let (fd, start) = self.planning_state(i);
        self.scan_from(&fd, start, pred)
    }

    /// [`Self::intercept_scan`] for a body in state `fd` that can start at `start`.
    #[allow(clippy::type_complexity)]
    fn scan_from(&self, fd: &Fielder, start: f64, pred: &BallPath) -> (Option<(f64, Vec3)>, Option<(f64, Vec3)>) {
        let t_end = pred.t_end();
        let mut dive = None;
        // Shortfall at time `t` for a ball at `b`: how far short of the ball's line the
        // fielder still is (≤ 0: he can field it standing, with the planning reach).
        // The wall he must slow for is part of the plan (Gemini play-area recheck P2-1).
        let gap = |t: f64, b: Vec3| run_gap(self.inp.field, fd, b, (t - start - wall_delay(self.inp.field, fd, b, REACH * 0.7)).max(0.0), REACH * 0.7);
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
            // At most SUB points (SUB − 1 between samples, then the sample): a fixed buffer, no
            // allocation per sample (Codex fielder-communication recheck P3-1).
            let mut pts = [(0.0, Vec3::ZERO, 0.0); SUB as usize];
            let mut n = 0;
            if let Some((tp, bp, gp)) = prev {
                let travel = m::hypot(b.x - bp.x, b.y - bp.y) + fd.a.speed * (t - tp);
                if gp - dive_extra <= travel {
                    for k in 1..SUB {
                        let ts = tp + (t - tp) * k as f64 / SUB as f64;
                        let (bs, _) = pred.at(ts);
                        pts[n] = (ts, bs, gap(ts, bs));
                        n += 1;
                    }
                }
            }
            pts[n] = (t, b, g);
            n += 1;
            prev = Some((t, b, g));
            for &(t, b, g) in &pts[..n] {
                // Players see the walls: no intercept past a wall, beyond his reach over it
                // (Gemini play-area review P2-3).
                if b.z > catch_ceiling(self.inp.field, b) || t < start || beyond_reach_over_wall(self.inp.field, b) > 0.0 {
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

    /// The first read of the ball: every fielder reads it and decides for himself
    /// (agent-models §5.7; docs/calibration/2026-11-14-fielder-communication.md).
    fn assign(&mut self) {
        self.assign_among((0..9).collect());
    }

    /// Once a second while the ball is loose and the man known to be going for it is far
    /// from it: the four fielders nearest where everyone sees it being fielded (the ball
    /// itself while nobody is known to be going for it), and every man going for it, read it
    /// again and decide again.
    fn recall(&mut self) {
        let b = self.fielding_point().unwrap_or_else(|| self.ball_at(self.t).0);
        let b = Vec3::new(b.x, b.y, 0.0);
        let mut idx: Vec<usize> = (0..9).collect();
        idx.sort_by(|&a, &c| (b - self.f[a].pos).norm().total_cmp(&(b - self.f[c].pos).norm()).then(a.cmp(&c)));
        let mut cands: Vec<usize> = idx.into_iter().take(4).collect();
        for i in 0..9 {
            if self.f[i].role == Role::Chase && !cands.contains(&i) {
                cands.push(i);
            }
        }
        self.assign_among(cands);
    }

    /// Fielder `i` reads the ball (agent-models §5.1) and plans his intercept (§5.2). A read
    /// already made this tick of this same ball is reused: it depends only on where he is,
    /// the tick and the ball.
    fn read(&mut self, i: usize) {
        let (t, ball) = (self.t, self.ball_gen);
        if self.f[i].read_at == t && self.f[i].read_ball == ball && self.f[i].belief.is_some() {
            return;
        }
        let p = self.predict(i);
        let (ti, pt) = self.intercept(i, &p);
        let doubt = self.track_now(i).map_or(0.0, |tr| self.track_doubt(&tr, self.f[i].pos));
        // His reads come faster as his own read has the ball coming down sooner.
        let down = p.first_bounce().map_or(p.t_end(), |b| b.0) - t;
        let every = (REPLAN_FRAC * down).clamp(REPLAN_MIN_S, REPLAN_S);
        let fd = &mut self.f[i];
        (fd.t_int, fd.int_pt, fd.belief, fd.read_at, fd.read_ball, fd.next_plan) = (ti, pt, Some(p), t, ball, t + every);
        (fd.doubt, fd.doubt_at) = (doubt, t);
    }

    /// A chaser's replan from a fresh read: he runs at his own new intercept, and his call
    /// follows the read (`refresh_call`).
    fn replan(&mut self, i: usize) {
        self.read(i);
        self.f[i].target = self.f[i].int_pt;
        self.refresh_call(i);
    }

    /// A chaser's call follows his own latest read (agent-models §5.7; Codex
    /// fielder-communication fourth review P2-2): a man whose fresh read no longer has him
    /// sure, or no longer has him catching it in the air, stops calling it, and one who has
    /// become sure calls it. Teammates hear the change `CALL_HEARD_S` later (`publish`).
    fn refresh_call(&mut self, i: usize) {
        let fd = &self.f[i];
        let Some(belief) = fd.belief.as_ref() else { return };
        let bias = if i >= 6 { -0.15 } else { 0.0 };
        self.f[i].call = self.call_value(i, belief, fd.t_int, fd.int_pt, bias).0;
    }

    /// Each chaser re-reads the ball on his own clock and runs at his new intercept; close to
    /// a slow ball, a chaser simply goes to it (seen directly at short range).
    fn replan_chasers(&mut self) {
        for i in 0..9 {
            if self.f[i].role == Role::Chase && self.t >= self.f[i].next_plan {
                self.replan(i);
            }
        }
        let (b, bv) = self.ball_at(self.t);
        if bv.norm() < 3.0 {
            for fd in self.f.iter_mut() {
                if fd.role == Role::Chase && (Vec3::new(b.x, b.y, 0.0) - fd.pos).norm() < 5.0 {
                    fd.target = Vec3::new(b.x, b.y, 0.0);
                }
            }
        }
    }

    /// Each candidate reads the ball, makes his own call on it, and decides alone whether to
    /// go for it (`decide`). Then the staff's tables give the others their jobs.
    fn assign_among(&mut self, cands: Vec<usize>) {
        for &i in &cands {
            self.read(i);
            let fd = &self.f[i];
            let bias = if i >= 6 { -0.15 } else { 0.0 };
            let call = self.call_value(i, fd.belief.as_ref().expect("just read"), fd.t_int, fd.int_pt, bias).0;
            self.f[i].call = call;
        }
        let decisions: Vec<(usize, Decision)> = cands.iter().map(|&i| (i, self.decide(i))).collect();
        // Each man's own choice to cover second, for his own view of the table only.
        let second: Vec<usize> = decisions.iter().filter(|d| d.1 == Decision::CoverSecond).map(|d| d.0).collect();
        if debug_comm() {
            eprintln!("comm t {:.3} play {} call: {:?}", self.t, self.inp.play, decisions.iter().map(|&(i, d)| (i, self.f[i].call, d, (self.f[i].t_int * 100.0).round() / 100.0)).collect::<Vec<_>>());
        }
        for (i, d) in decisions {
            self.apply(i, d);
        }
        self.cover_roles(&second);
        self.assign_relay();
        self.assign_backups();
        if debug_comm() {
            eprintln!("comm t {:.3}   roles {:?} side {:.1}", self.t, self.f.iter().map(|f| f.role).collect::<Vec<_>>(), self.ball_side_x());
        }
    }

    /// Fielder `i`'s own decision on the ball (agent-models §5.7), from his own read and call,
    /// the calls he has heard, and his own estimates of his teammates from where they are and
    /// how they are moving. Never from a teammate's plan.
    ///
    /// A ball he believes he catches in the air (call tier ≥ 1): he goes, unless he has heard
    /// a sure call that outranks his ([`outranks`]). A ball he cannot catch in the air: he
    /// leaves it to any sure call he has heard; otherwise an infielder breaks under the
    /// ground-ball rule (the first baseman whom it sends back to his bag stays there). A man
    /// not yet going goes if by his own estimate he gets to it before every teammate not known
    /// to be on another job. A man already going keeps going unless a teammate known to be
    /// going for it both gets there first by his own estimate and is ahead of him by the order
    /// everyone can work out (`ahead_of`). The man first in that order is never stopped by the
    /// others, so two men who each think the other is first do not both stop (Codex
    /// fielder-communication recheck P1-2: a chase was dropped whenever more men became
    /// known to be going, and all could stop at once).
    fn decide(&self, i: usize) -> Decision {
        let tier = self.f[i].call;
        let heard = self.heard_call(i);
        if tier >= 1 {
            return match heard {
                Some(h) if outranks(h, (i, tier)) => Decision::Yield,
                _ => Decision::Go,
            };
        }
        if heard.is_some() {
            return Decision::Yield;
        }
        // A first baseman going for a ball the pitcher is known to be going for too, ahead of
        // him by the order both can work out, goes back to his bag: one of the two covers first
        // (recheck P1-2). The pitcher, behind, stops by the general rule below.
        if i == 2 && self.f[i].role == Role::Chase && self.ahead_of(0, 2) {
            return Decision::Stay;
        }
        if (2..=5).contains(&i) {
            // A middle infielder's estimate of his partner, made once for both rules.
            let partner = (i == 3 || i == 5).then(|| self.partner_fields_it(i));
            match self.grounder_break(i, partner) {
                Some(true) => return Decision::Go,
                // The first-base rule leaves this ground ball to someone else: he stays with
                // his bag.
                Some(false) if i == 2 => return Decision::Stay,
                _ => {}
            }
            if partner == Some(true) {
                return Decision::CoverSecond;
            }
        }
        let goes = if self.f[i].role == Role::Chase { !(0..9).any(|j| self.ahead_of(j, i) && self.beats_by_own_estimate(i, j)) } else { self.first_by_own_estimate(i) };
        if goes { Decision::Go } else { Decision::Stay }
    }

    /// By fielder `i`'s own read, teammate `j` gets to the ball before he does (his estimate of
    /// `j` from where he is and how he moves, `seen_intercept`; outfielders win near-ties of
    /// 0.15 s; a dead heat to the lower index).
    fn beats_by_own_estimate(&self, i: usize, j: usize) -> bool {
        let fd = &self.f[i];
        let Some(pred) = fd.belief.as_ref() else { return false };
        let bias = |k: usize| if k >= 6 { -0.15 } else { 0.0 };
        let (mine, theirs) = (fd.t_int + bias(i), self.seen_intercept(j, pred).0 + bias(j));
        theirs < mine || (theirs == mine && j < i)
    }

    /// Fielder `j`, known to be going for this ball, has it ahead of chaser `i` by an order
    /// every fielder can work out from what all can see (`public_rank`; a dead heat to the
    /// call priority, `publicly_before`). As in `first_by_own_estimate`, an outfielder
    /// weighs only outfielders.
    fn ahead_of(&self, j: usize, i: usize) -> bool {
        if j == i || (i >= 6 && j < 6) || !matches!(self.known(j), (true, _, true)) {
            return false;
        }
        self.publicly_before(j, i)
    }

    /// How soon fielder `j` can field the loose ball, as every teammate can work it out alike
    /// (agent-models §5.7): his earliest standing intercept, or dive, of `public_path` (the
    /// ball as all see it now, run on with the agreed model: its descent and where it stops),
    /// from where he is and how he is moving, at his known speed after what is left of his
    /// known reaction (`seen_intercept`). Never the world's future or anyone's own read.
    /// Computed once a tick for all nine and kept while the ball and bodies are unchanged.
    fn public_rank(&self, j: usize) -> f64 {
        let pos: [Vec3; 9] = core::array::from_fn(|k| self.f[k].pos);
        let vel: [Vec3; 9] = core::array::from_fn(|k| self.f[k].vel);
        if let Some((t, g, p, v, r)) = *self.rank_cache.borrow() {
            if t == self.t && g == self.ball_gen && p == pos && v == vel {
                return r[j];
            }
        }
        let path = self.public_path();
        let r: [f64; 9] = core::array::from_fn(|k| self.seen_intercept(k, &path).0);
        *self.rank_cache.borrow_mut() = Some((self.t, self.ball_gen, pos, vel, r));
        r[j]
    }

    /// The shared order for the loose ball (agent-models §5.7): `j` comes before `i` when
    /// his `public_rank` is sooner; a dead heat (the intercept scan's grid makes exact ties,
    /// most often on pop-ups) goes by the call priority everyone knows before the pitch, an
    /// outfielder over an infielder and then `CALL_PRIORITY`, so the pitcher is the last
    /// choice (Opus fourth review P3-1: the tie went to the lower position number, the
    /// pitcher, on a pop-up over the mound), then the lower position.
    fn publicly_before(&self, j: usize, i: usize) -> bool {
        let key = |k: usize| (std::cmp::Reverse(k >= 6), std::cmp::Reverse(CALL_PRIORITY[k]), k);
        let (rj, ri) = (self.public_rank(j), self.public_rank(i));
        rj < ri || (rj == ri && key(j) < key(i))
    }

    /// The loose ball as every fielder can work it out alike: its position and velocity now,
    /// the typical spin for how it is moving (`believed_flight`), run on with the fielders' own
    /// physics and no bounce scatter, as a read with no error is (`read_prediction`).
    fn public_path(&self) -> BallPath {
        let (b, v) = self.ball_at(self.t);
        let (env, w) = believed_flight(self.inp.env, v, self.off_bat().map(|x| x.0));
        batted_path_dt(&env, self.inp.field, BallState { r: b, v, w }, self.t, AGENT_DT, |_, _| (0.0, 0.0))
    }

    /// Who goes for a loose ball that every fielder has left to another (agent-models §5.7;
    /// Codex fielder-communication recheck P1-2). Once every man is known not to be going for
    /// it, the man first by `public_rank`, which each works out alike, goes; nobody else
    /// moves. A man known to be covering first is left out, so first stays covered. Before
    /// this every man with a recent read decided again with every teammate known not to be
    /// going left out of his comparison, so all nine went, then all stopped, every 0.6 s.
    fn fallback_chaser(&self) -> Option<usize> {
        if !(0..9).all(|j| matches!(self.known(j), (false, _, true))) {
            return None;
        }
        (0..9).filter(|&j| self.known_job(j).0 != Role::Cover(1)).reduce(|best, j| if self.publicly_before(j, best) { j } else { best })
    }

    /// Back up a bounced ball that could go through the infield (agent-models §5.7).
    /// If every outfielder is known to have left it and only infielders are known to be
    /// chasing, the outfield's first man in the shared order reconsiders.
    fn outfielder_to_reconsider(&self) -> Option<usize> {
        if self.ball_still_up() || !(6..9).all(|j| matches!(self.known(j), (false, _, true))) {
            return None;
        }
        if !(0..6).any(|j| matches!(self.known(j), (true, _, true))) {
            return None;
        }
        let path = self.public_path();
        let (end, _) = path.at(path.t_end());
        if m::hypot(end.x, end.y) <= INFIELD_BREAK_R {
            return None;
        }
        (6..9)
            .filter(|&j| matches!(self.known_job(j).0, Role::Idle | Role::Backup))
            .reduce(|best, j| if self.publicly_before(j, best) { j } else { best })
    }

    /// Carry out fielder `i`'s decision. A man who goes runs at his own intercept. A chaser who
    /// stops leaves the ball: an infielder who yields a ball in the air to a sure call backs up
    /// the catch from `PEEL_BACKUP_M` short of where his own read has it coming down, on his
    /// side of it (`assign_backups` gives him the job unless a cover job takes him); a middle
    /// infielder who leaves a ground ball to his partner covers second (the man who does not
    /// field it covers); anyone else waits for the tables to give him a job.
    fn apply(&mut self, i: usize, d: Decision) {
        match d {
            Decision::Go => {
                let fd = &mut self.f[i];
                fd.role = Role::Chase;
                fd.target = fd.int_pt;
                fd.peel = None;
            }
            Decision::CoverSecond => {
                let fd = &mut self.f[i];
                if fd.role != Role::Cover(2) {
                    go_idle(fd);
                    fd.role = Role::Cover(2);
                    fd.target = base_pos(2);
                }
                fd.peel = None;
            }
            Decision::Yield | Decision::Stay => {
                let was_chasing = self.f[i].role == Role::Chase;
                let fd = &mut self.f[i];
                if d == Decision::Stay {
                    fd.peel = None;
                } else if i < 6 && fd.call >= 1 {
                    let at = Vec3::new(fd.int_pt.x, fd.int_pt.y, 0.0);
                    let back = Vec3::new(fd.pos.x - at.x, fd.pos.y - at.y, 0.0);
                    let dir = if back.norm() > 1e-6 { back.normalized() } else { Vec3::new(-at.x, -at.y, 0.0).normalized() };
                    fd.peel = Some(at + dir * PEEL_BACKUP_M);
                }
                if was_chasing {
                    go_idle(fd);
                }
            }
        }
    }

    /// A ground ball, by this read: it comes down within 25 m in the first
    /// `GROUNDER_BOUNCE_S` (an infield pop-up also lands within 25 m, seconds later).
    fn is_grounder(&self, p: &BallPath) -> bool {
        p.first_bounce().is_some_and(|b| m::hypot(b.1, b.2) < 25.0 && b.0 - self.inp.t_contact < GROUNDER_BOUNCE_S)
    }

    /// Infielders break for a ground ball without a call (agent-models §5.7), each from his own
    /// recent read: it is a ground ball and his intercept is in the infield. He runs at that
    /// intercept. The first baseman goes only for a ball on the first-base side of the line
    /// through the mound (the side on which the pitcher's job is to cover first), and not
    /// when by his own estimate the pitcher gets to it first (the pitcher who fields it
    /// cannot cover first; Opus infield-fixes review P1-1). A middle infielder goes only if
    /// his partner does not field it by his own estimate (`partner_fields_it`); if the partner
    /// does, he covers second, in range or not (the man who does not field it covers;
    /// `Decision::CoverSecond`). Each judges this himself,
    /// at once, without waiting to see the other go (Opus infield-fixes review P2-3). A
    /// teammate known to be on another job does not count. `partner`: for a middle
    /// infielder, his `partner_fields_it`. `None` when the rule does not apply (not a ground
    /// ball he can reach in the infield by a recent read), else whether he breaks.
    fn grounder_break(&self, i: usize, partner: Option<bool>) -> Option<bool> {
        let fd = &self.f[i];
        let pred = fd.belief.as_ref()?;
        let recent = self.t - fd.read_at <= HEAR_READ_AGE_S;
        if !(recent && self.is_grounder(pred) && fd.t_int.is_finite() && m::hypot(fd.int_pt.x, fd.int_pt.y) < INFIELD_BREAK_R) {
            return None;
        }
        // By his own read, `j` gets to the ball before he does (a dead heat to the lower index),
        // and is not known to be doing something else.
        let beats_me = |j: usize| {
            let (chasing, _, this_ball) = self.known(j);
            let theirs = self.seen_intercept(j, pred).0;
            !(this_ball && !chasing) && (theirs < fd.t_int || (theirs == fd.t_int && j < i))
        };
        Some(match i {
            2 => fd.int_pt.x > 0.0 && !beats_me(0),
            3 | 5 => partner == Some(false),
            _ => true,
        })
    }

    /// By middle infielder `i`'s own recent read, a ground ball his partner fields: the
    /// partner (not known to be on another job) gets to it in the infield, and before `i`
    /// does (a dead heat to the lower index). Then `i` covers second.
    fn partner_fields_it(&self, i: usize) -> bool {
        let fd = &self.f[i];
        let Some(pred) = fd.belief.as_ref() else { return false };
        if self.t - fd.read_at > HEAR_READ_AGE_S || !self.is_grounder(pred) {
            return false;
        }
        let partner = 8 - i;
        // A partner known to be on a job he cannot leave for it (another base, the cutoff)
        // does not field it; one idle, backing up, or covering second may, on the same reread
        // (Codex fielder-communication third review P2-1: leaving out every partner not known
        // to be going sent both, with second empty, even on error-free reads).
        if matches!(self.known_job(partner).0, Role::Cover(b) if b != 2) || matches!(self.known_job(partner).0, Role::Relay(_)) {
            return false;
        }
        let (tp, pp) = self.seen_intercept(partner, pred);
        m::hypot(pp.x, pp.y) < INFIELD_BREAK_R && (tp < fd.t_int || (tp == fd.t_int && partner < i))
    }

    /// By fielder `i`'s own read, he fields the ball before every teammate who could (his
    /// estimate of each from where that man is and how he is moving, at his known speed,
    /// after what is left of his known reaction; outfielders win near-ties of 0.15 s on balls
    /// in between). A teammate known to be on another job does not count. An outfielder
    /// weighs only the other outfielders: he comes in on any ball hit his way and does not
    /// stop for an infielder his own read, from 60 m and more, says might reach it (the
    /// infielder, near it, judges that himself; if he fields it the outfielder's run is the
    /// backup). Weighing the infielders too, outfielders left 4.5% of balls to an infielder
    /// whose own read had no play, and nobody went for them until the 0.3 s had passed (docs/
    /// calibration/2026-11-14-fielder-communication.md §7).
    fn first_by_own_estimate(&self, i: usize) -> bool {
        let fd = &self.f[i];
        let Some(pred) = fd.belief.as_ref() else { return false };
        let bias = |j: usize| if j >= 6 { -0.15 } else { 0.0 };
        let mine = fd.t_int + bias(i);
        (0..9).filter(|&j| j != i && (i < 6 || j >= 6)).all(|j| {
            let (chasing, _, this_ball) = self.known(j);
            if this_ball && !chasing {
                return true;
            }
            let theirs = self.seen_intercept(j, pred).0 + bias(j);
            let first = mine < theirs || (mine == theirs && i < j);
            if !first && debug_comm() {
                eprintln!("comm t {:.3} play {} f{i} ({mine:.2}) defers to f{j} ({theirs:.2}) at {:?}", self.t, self.inp.play, self.seen_intercept(j, pred).1);
            }
            first
        })
    }

    /// Fielder `j`'s earliest intercept as a teammate judges it, on the teammate's read
    /// `pred`: from `j`'s visible position and velocity, at his known speed, starting once his
    /// known reaction time is over. Nothing of `j`'s own plan is used.
    fn seen_intercept(&self, j: usize, pred: &BallPath) -> (f64, Vec3) {
        let fd = &self.f[j];
        let start = self.t.max(self.inp.t_contact + fd.a.reaction);
        let (standing, dive) = self.scan_from(fd, start, pred);
        standing.or(dive).unwrap_or_else(|| {
            let t_end = pred.t_end();
            let (rest, _) = pred.at(t_end);
            (t_end.max(start + run_time_to(self.inp.field, fd, rest, 0.0)), Vec3::new(rest.x, rest.y, 0.0))
        })
    }

    /// Seconds a teammate judges fielder `j` needs to get to `pt`: what is left of his known
    /// reaction time, then his run from his visible position and velocity at his known speed.
    fn eta(&self, j: usize, pt: Vec3) -> f64 {
        let fd = &self.f[j];
        (self.inp.t_contact + fd.a.reaction - self.t).max(0.0) + run_time_to(self.inp.field, fd, pt, REACH * 0.7) + wall_delay(self.inp.field, fd, pt, REACH * 0.7)
    }

    /// What fielder `j`'s teammates know of him now (agent-models §5.7): whether he is going
    /// for the ball and his call tier, as he showed it `CALL_HEARD_S` ago, and whether that
    /// was on the ball now loose. A call made on an earlier ball does not rank (tier 0).
    /// Before he has shown anything he is known only to be at his spot.
    fn known(&self, j: usize) -> (bool, u8, bool) {
        match self.shown_then(j) {
            Some(e) => {
                let this_ball = e.at + 1e-9 >= self.ball_since;
                (e.role == Role::Chase, if this_ball { e.tier } else { 0 }, this_ball)
            }
            None => (false, 0, false),
        }
    }

    /// What fielder `j` showed `CALL_HEARD_S` ago: all his teammates can know of him now.
    fn shown_then(&self, j: usize) -> Option<&Shown> {
        let horizon = self.t - CALL_HEARD_S + 1e-9;
        self.f[j].shown.iter().rev().find(|e| e.at <= horizon)
    }

    /// The job fielder `j` is known to be doing (and the base he backs up), as he showed it
    /// `CALL_HEARD_S` ago; before he has shown anything, none.
    fn known_job(&self, j: usize) -> (Role, Option<u8>) {
        self.shown_then(j).map(|e| (e.role, e.backs)).unwrap_or((Role::Idle, None))
    }

    /// Fielder `j`'s job as fielder `o` knows it (Codex fielder-communication review P1-1):
    /// his own job when `j` is himself, a teammate's as it is known (`known_job`). Every
    /// table a fielder works out for himself (covers, backups, the receiver of a throw) uses
    /// this, never a teammate's choice not yet seen or heard.
    fn seen(&self, o: usize, j: usize) -> (Role, Option<u8>) {
        if o == j { (self.f[o].role, self.f[o].backs) } else { self.known_job(j) }
    }

    /// The best sure call fielder `i` has heard from a teammate: (who, tier).
    fn heard_call(&self, i: usize) -> Option<(usize, u8)> {
        (0..9)
            .filter(|&j| j != i)
            .filter_map(|j| {
                let (chasing, tier, _) = self.known(j);
                (chasing && tier >= 2).then_some((j, tier))
            })
            .max_by(|&a, &b| if outranks(a, b) { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Less })
    }

    /// End of a tick: each fielder shows his teammates what he is doing (heard or seen
    /// `CALL_HEARD_S` later): whether he is going for the ball and his call, or the base he
    /// is covering, the cutoff he is going out to, the backup he is running to. Everyone
    /// shows himself once on each new ball, so that teammates come to know who is not going
    /// for it.
    fn publish(&mut self) {
        let (t, since) = (self.t, self.ball_since);
        let holder = match self.ball {
            Ball::Held { by } => Some(by),
            _ => None,
        };
        for (i, fd) in self.f.iter_mut().enumerate() {
            let tier = if fd.role == Role::Chase { fd.call } else { 0 };
            // A man's call for a teammate to a bag stands only while he has the ball.
            let asks = if holder == Some(i) { fd.asks } else { None };
            let push = match fd.shown.last() {
                Some(e) => e.at + 1e-9 < since || (e.role, e.backs, e.tier, e.asks) != (fd.role, fd.backs, tier, asks),
                None => true,
            };
            if push {
                fd.shown.push(Shown { at: t, role: fd.role, backs: fd.backs, tier, asks });
            }
        }
    }

    /// When what the fielders know of each other changes (a call heard, a man seen going or
    /// stopping), each chaser decides again whether to yield, a man with a recent read whether
    /// to go (see below), and the tables fill the bases and backups with what is now known.
    /// A change only in the jobs men are known to be doing (a cover, a cutoff, a backup) has
    /// each man work out his own cover and backup jobs again.
    fn hear(&mut self) {
        let now: Vec<(bool, u8, bool)> = (0..9).map(|j| self.known(j)).collect();
        let jobs: Vec<(Role, Option<u8>)> = (0..9).map(|j| self.known_job(j)).collect();
        let calls = now != self.last_known;
        if !calls && jobs == self.last_jobs {
            return;
        }
        self.last_known = now;
        self.last_jobs = jobs;
        if !matches!(self.ball, Ball::Free { .. }) {
            // With the ball in a fielder's hands the bases are still re-worked (a cover man
            // seen going out as the cutoff is replaced at his base).
            if matches!(self.ball, Ball::Held { .. }) {
                self.cover_roles_after_chasers();
            }
            return;
        }
        if !calls {
            self.cover_roles_after_chasers();
            self.assign_backups();
            return;
        }
        // A man not going for it reconsiders a ball in the air he believes he catches (a call
        // that outranked his may be gone). On a bounced ball that could get through, the
        // shared order allows one outfielder to reconsider even with an infielder chasing
        // (`outfielder_to_reconsider`). A ball every man has left to another is sent to one
        // man by the shared convention below (`fallback_chaser`).
        let outfielder = self.outfielder_to_reconsider();
        let decisions: Vec<(usize, Decision)> = (0..9)
            .filter(|&i| {
                let fd = &self.f[i];
                // The read must be of the ball now loose. This is a guard only: every new loose
                // ball goes through `set_free` → `assign`, which has all nine read it, so games
                // never reach this with a stale read (Fable review P3-2).
                let recent = self.t - fd.read_at <= HEAR_READ_AGE_S && fd.read_ball == self.ball_gen;
                let sent = outfielder == Some(i) && matches!(fd.role, Role::Idle | Role::Backup);
                fd.belief.is_some() && (fd.role == Role::Chase || (recent && (fd.call >= 1 || sent)))
            })
            .map(|i| (i, self.decide(i)))
            .collect();
        if debug_comm() {
            eprintln!("comm t {:.3} play {} hear: known {:?} decide {:?}", self.t, self.inp.play, self.last_known, decisions);
        }
        for (i, d) in decisions {
            // A man not chasing who stays out of it keeps the job he has.
            if self.f[i].role == Role::Chase || d == Decision::Go {
                self.apply(i, d);
            }
        }
        if let Some(f) = self.fallback_chaser() {
            if self.f[f].role != Role::Chase {
                if debug_comm() {
                    eprintln!("comm t {:.3} play {} nobody going: f{f} goes by the convention", self.t, self.inp.play);
                }
                self.read(f);
                self.apply(f, Decision::Go);
            }
        }
        self.cover_roles_after_chasers();
        self.assign_backups();
        if debug_comm() {
            eprintln!("comm t {:.3}   roles {:?} lead {:?}", self.t, self.f.iter().map(|f| f.role).collect::<Vec<_>>(), self.lead());
        }
    }

    /// The man the defense knows is going for the ball (the lead chaser): of those known to be
    /// chasing, the highest known call, then (sure calls) the order of [`outranks`], then the
    /// one nearest the ball now. `None` until someone's chase is known.
    fn lead(&self) -> Option<usize> {
        let (b, _) = self.ball_at(self.t);
        let d = |j: usize| m::hypot(b.x - self.f[j].pos.x, b.y - self.f[j].pos.y);
        (0..9)
            .filter_map(|j| {
                let (chasing, tier, _) = self.known(j);
                chasing.then_some((j, (tier, if tier >= 2 { (CALL_PRIORITY[j], 9 - j) } else { (0, 0) })))
            })
            .max_by(|a, c| a.1.cmp(&c.1).then(d(c.0).total_cmp(&d(a.0))).then(c.0.cmp(&a.0)))
            .map(|x| x.0)
    }

    /// Where the ball is being fielded, as everyone sees it: the holder; while it is loose,
    /// the point on the ball's ground track (where it is now, on along its ground velocity)
    /// nearest the man known to be going for it: at his feet when he is camped under a fly,
    /// where his path meets its line when he is cutting it off. `None` while it is loose and
    /// nobody is known to be going for it, or in flight as a throw.
    fn fielding_point(&self) -> Option<Vec3> {
        match self.ball {
            Ball::Held { by } => Some(self.f[by].pos),
            Ball::Free { .. } => self.lead().map(|l| self.track_point(self.f[l].pos)),
            Ball::Thrown { .. } => None,
        }
    }

    /// The point of the loose ball's ground track nearest `p`.
    fn track_point(&self, p: Vec3) -> Vec3 {
        let (b, v) = self.ball_at(self.t);
        let g = Vec3::new(b.x, b.y, 0.0);
        let h = Vec3::new(v.x, v.y, 0.0);
        if h.norm() < 0.5 {
            return g;
        }
        let u = h.normalized();
        g + u * Vec3::new(p.x - g.x, p.y - g.y, 0.0).dot(u).max(0.0)
    }

    /// Which side the ball is on, as everyone sees it (x < 0: the left side): where it is
    /// being fielded once that is known, else where it is heading (its ground position and
    /// one second of its ground velocity).
    fn ball_side_x(&self) -> f64 {
        if let Some(p) = self.fielding_point() {
            return p.x;
        }
        let (b, v) = self.ball_at(self.t);
        b.x + v.x
    }

    /// The pitcher's own read says the first baseman will go for this ball: a ground ball
    /// the first baseman (by the pitcher's estimate of him) reaches in the infield on the
    /// first-base side of the mound. The pitcher then breaks for first at once, as the
    /// convention has him do, without waiting to see the first baseman go.
    fn pitcher_expects_first_baseman_break(&self) -> bool {
        let p = &self.f[0];
        let Some(pred) = p.belief.as_ref() else { return false };
        if self.t - p.read_at > HEAR_READ_AGE_S || !self.is_grounder(pred) {
            return false;
        }
        let (_, pt) = self.seen_intercept(2, pred);
        pt.x > 0.0 && m::hypot(pt.x, pt.y) < INFIELD_BREAK_R
    }

    /// A fielder's call on the ball from his own read (agent-models §5.7). He believes he
    /// catches it in the air with at least `CALL_SLACK_S` to spare: if that margin also covers
    /// `CALL_SURE_SIGMAS` of his read's depth error (the distance run off at his speed), he is
    /// sure and calls it, tier 3 (outfielder) or 2 (infielder); if not, he goes for it without
    /// calling, tier 1. Both are valued by the time to spare. Otherwise tier 0, valued by how
    /// early he fields it (outfielders win near-ties on balls in between).
    fn call_value(&self, i: usize, pred: &BallPath, ti: f64, pt: Vec3, bias: f64) -> (u8, f64) {
        let in_air = ti.is_finite() && pred.first_bounce().map(|b| ti < b.0).unwrap_or(true);
        if in_air {
            let (fd, start) = self.planning_state(i);
            let spare = ti - (start + run_time_to(self.inp.field, &fd, pt, REACH * 0.7) + wall_delay(self.inp.field, &fd, pt, REACH * 0.7));
            if spare >= CALL_SLACK_S {
                // Where he believes the ball is now, and how long he has watched it.
                let view = (self.t - self.inp.t_contact - fd.a.reaction).max(0.0);
                let sigma = match self.track_now(i) {
                    // His running estimate's own doubt (agent-models §5.1), as worked out at
                    // this tick's read when there was one.
                    Some(tr) => {
                        if self.f[i].doubt_at == self.t && self.f[i].read_ball == self.ball_gen { self.f[i].doubt } else { self.track_doubt(&tr, fd.pos) }
                    }
                    None => landing_depth_sigma(self.inp.env, self.inp.field, pred.at(self.t), self.t, fd.pos, fd.a.read_sigma0, fd.a.read_vel_frac, fd.a.tau_read, view, self.off_bat()),
                };
                let doubt = CALL_SURE_SIGMAS * sigma / fd.a.speed;
                if spare < CALL_SLACK_S + doubt {
                    return (1, spare);
                }
                return (if i >= 6 { 3 } else { 2 }, spare);
            }
        }
        (0, -(ti + bias))
    }

    /// A fielder still going for the ball who sees another fielder secure it (a visible
    /// fact, seen after his reaction time) gives up the chase: an infielder, the pitcher or
    /// the catcher goes to his own base if nobody else covers it, anyone else pulls up
    /// (agent-models §5.7; Opus infield-fixes review P1-1: a first baseman who had broken
    /// for a grounder the pitcher fielded ran on across the infield and nobody took first).
    /// A man who has had the ball himself this play is left alone.
    fn chasers_see_the_ball_secured(&mut self) {
        let Ball::Held { by } = self.ball else { return };
        for i in 0..9 {
            let fd = &self.f[i];
            if i == by || fd.role != Role::Chase || self.t < self.secured_at + fd.a.reaction || self.out.secured.iter().any(|s| s.1 == fd.id) {
                continue;
            }
            // Indices are position − 1: P0 C1 1B2 2B3 3B4 SS5.
            let base = match i {
                0 | 2 => Some(1),
                1 => Some(4),
                3 | 5 => Some(2),
                4 => Some(3),
                _ => None,
            };
            go_idle(&mut self.f[i]);
            // Nobody he knows of is covering it (a cover taken this instant is not yet seen).
            if let Some(b) = base.filter(|&b| !(0..9).any(|j| j != by && j != i && self.seen(i, j).0 == Role::Cover(b))) {
                self.f[i].role = Role::Cover(b);
                self.f[i].target = base_pos(b);
            }
        }
    }

    /// The ball is in the air off the bat or off a glove and has not come down yet (a fact
    /// everyone can see): while it is, a called-off infielder's backup job stands.
    fn ball_still_up(&self) -> bool {
        match &self.ball {
            Ball::Free { path, .. } => path.bounces.first().map(|x| x.0 > self.t).unwrap_or(true),
            _ => false,
        }
    }

    /// A called-off infielder's backup job ends when the ball comes down, is caught, or is
    /// thrown (Codex peel-off review #2): he is free again, and on a loose ball he reads it
    /// at once and takes his own job (`retake`), so the man who was backing it up can field
    /// it. His teammates keep their reads and jobs until they observe a change.
    fn expire_peels(&mut self) {
        if self.ball_still_up() {
            return;
        }
        let ended: Vec<usize> = (0..9).filter(|&i| self.f[i].peel.is_some()).collect();
        for &i in &ended {
            let fd = &mut self.f[i];
            fd.peel = None;
            if fd.role == Role::Backup && fd.backs.is_none() {
                go_idle(fd);
            }
        }
        if matches!(self.ball, Ball::Free { .. }) {
            for i in ended {
                if self.f[i].role == Role::Idle {
                    self.retake(i);
                }
            }
        }
    }

    /// Fielder `i`, with no job, reads the ball, makes his call and decides alone whether to
    /// go for it; if not, he takes the cover or backup job his own working of the tables
    /// gives him (`cover_after_chasers`, `backup_job`).
    fn retake(&mut self, i: usize) {
        self.read(i);
        let fd = &self.f[i];
        let bias = if i >= 6 { -0.15 } else { 0.0 };
        self.f[i].call = self.call_value(i, fd.belief.as_ref().expect("just read"), fd.t_int, fd.int_pt, bias).0;
        match self.decide(i) {
            d @ (Decision::Go | Decision::CoverSecond) => self.apply(i, d),
            _ => self.take_own_job(i),
        }
    }

    /// The cutoff or relay job for a ball fielded at `from`: who takes it and toward which
    /// base, from the standard assignments (the likeliest throw: home with a runner in scoring
    /// position, third with a runner on first, otherwise second, or third on a deep ball;
    /// raised to any base a runner is visibly heading past that).
    fn relay_job(&self, from: Vec3) -> Option<(usize, u8)> {
        // Only for a ball an outfielder fields: once an infielder has it (the relay man
        // himself, or a middle infielder on a shallow fly), nobody else goes out.
        let fielder = match self.ball {
            Ball::Held { by } => by,
            _ => self.lead()?,
        };
        if fielder < 6 {
            return None;
        }
        let depth = m::hypot(from.x, from.y);
        if depth < RELAY_FROM_M {
            return None;
        }
        let deep = depth > RELAY_DEEP_M;
        // Infielders go out to the cutoff spot once the ball is down in the outfield (or in a
        // fielder's hands). While it is in the air they play the ball; only on a ball going
        // deep does the relay man go out early, and only if his own read says he has no
        // play on it.
        let down = match &self.ball {
            Ball::Free { path, .. } => path.bounces.first().map(|x| x.0 <= self.t).unwrap_or(false),
            _ => true,
        };
        let on = |b: u8| self.occupied(b);
        let mut b = if on(2) || on(3) { 4 } else if on(1) || deep { 3 } else { 2 };
        for r in self.r.iter().filter(|r| !r.out && !r.scored) {
            if let Some(target) = visible_target(r, self.f[fielder].a.throw_choice) {
                b = b.max(target);
            }
        }
        let who = if deep {
            let spot = Self::relay_spot(from, b);
            if (self.home[5] - spot).norm() <= (self.home[3] - spot).norm() { 5 } else { 3 }
        } else {
            match b {
                4 => if from.x < -15.0 { 4 } else { 2 },
                3 => 5,
                _ => return None,
            }
        };
        if matches!(self.f[who].role, Role::Chase) || matches!(self.ball, Ball::Held { by } if by == who) {
            return None;
        }
        // "No play on it": by his own read he cannot be there before it lands (his
        // intercept time is when he could field it, possibly long after the bounce).
        let no_air_play = || {
            let fd = &self.f[who];
            fd.belief.as_ref().and_then(|p| p.first_bounce()).map(|b| fd.t_int > b.0).unwrap_or(true)
        };
        if !down && !(deep && no_air_play()) {
            return None;
        }
        // While the ball is loose he plays it himself if his own read says he gets there
        // before the man chasing it could get to the same spot (a grounder he can cut off
        // before it reaches the outfield): the relay job is for a ball that is past him.
        if matches!(self.ball, Ball::Free { .. }) && self.f[who].t_int <= self.t + self.eta(fielder, self.f[who].int_pt) {
            return None;
        }
        Some((who, b))
    }

    /// The cutoff man's spot: on the line from the ball to the base.
    fn relay_spot(from: Vec3, b: u8) -> Vec3 {
        let base = base_pos(b);
        let line = Vec3::new(from.x - base.x, from.y - base.y, 0.0);
        // A deep ball's relay man goes out; on a shallower one the cutoff man stands where a
        // throw through him carries to the base (agent-models §5.5, §5.7).
        let short = if m::hypot(from.x, from.y) > RELAY_DEEP_M { cut_spot(line.norm()) } else { through_spot(line.norm()) };
        base + line.normalized() * short
    }

    /// Give the cutoff or relay job to its man (taking him off any base he was covering;
    /// his teammates learn it when his new job is known, `hear`, and one of them covers it:
    /// his private change of job refreshes nobody else's; Codex fielder-communication third
    /// review P1-1).
    fn assign_relay(&mut self) {
        let Some(from) = self.fielding_point() else { return };
        let Some((i, b)) = self.relay_job(from) else { return };
        self.f[i].role = Role::Relay(b);
        self.f[i].backs = None;
        self.f[i].target = Self::relay_spot(from, b);
    }

    /// Fielder `i`, with no job now, takes the cover or backup job his own working of the
    /// tables gives him (`cover_after_chasers`, `backup_job`), and nobody else is moved.
    fn take_own_job(&mut self, i: usize) {
        if let Some(b) = self.cover_after_chasers(i) {
            self.f[i].role = Role::Cover(b);
            self.f[i].target = base_pos(b);
        } else if let Some((target, backs)) = self.fielding_point().and_then(|from| self.backup_job(i, from)) {
            self.f[i].role = Role::Backup;
            self.f[i].target = target;
            self.f[i].backs = backs;
        }
    }

    /// Each tick each man works out the cutoff job for himself (`relay_job`: from what
    /// everyone sees, and for its man his own chase and read): its man takes it, or keeps to
    /// its line as the fielding point and the runners' intentions change; a man who had it
    /// and is no longer its man goes back to covering. A throw to someone else that goes by
    /// him ends it; a throw coming to him keeps it. The man leaving takes his own next job.
    /// Teammates update their jobs when they learn of the change (`hear`).
    fn update_relay(&mut self) {
        let from = self.fielding_point();
        let job = match self.ball {
            Ball::Thrown { .. } => None,
            _ => from.and_then(|f| self.relay_job(f)),
        };
        for i in 0..9 {
            if !matches!(self.f[i].role, Role::Relay(_)) {
                continue;
            }
            match self.ball {
                Ball::Thrown { receiver, .. } if receiver == i => continue,
                // A throw to someone else has gone by him (see `holder_act`).
                Ball::Thrown { .. } => {
                    go_idle(&mut self.f[i]);
                    continue;
                }
                _ => {}
            }
            // Nobody known to be going for the ball just now: he holds his line.
            if from.is_some() && !matches!(job, Some((who, _)) if who == i) {
                go_idle(&mut self.f[i]);
                self.take_own_job(i);
                // The isolation harness's negative control: `f3d9e7a` re-worked every
                // teammate's cover here, on this man's private retirement.
                #[cfg(test)]
                if ISO_REVERT_GATE.with(|c| c.get()) {
                    self.cover_roles_after_chasers();
                }
            }
        }
        let (Some(from), Some((who, b))) = (from, job) else { return };
        #[cfg(test)]
        let was_cover = matches!(self.f[who].role, Role::Cover(_));
        self.f[who].role = Role::Relay(b);
        self.f[who].backs = None;
        self.f[who].target = Self::relay_spot(from, b);
        #[cfg(test)]
        if was_cover && ISO_REVERT_GATE.with(|c| c.get()) {
            self.cover_roles_after_chasers();
        }
    }

    /// Fielders with no part in the play back it up (agent-models §5.7): one behind the man
    /// fielding the ball, and one behind each base a throw is likely to go to, on the line
    /// of that throw. One backup per base. Each man works out his own job (`backup_job`) from
    /// what he can see and know: where the ball is being fielded, which bases are occupied,
    /// and who is known to be free. Until someone is known to be going for a loose ball nobody
    /// knows where that is, and only the called-off infielders' jobs are given.
    fn assign_backups(&mut self) {
        let fp = self.fielding_point();
        let empty = !(self.occupied(1) || self.occupied(2) || self.occupied(3));
        for i in 0..9 {
            self.f[i].backs = None;
            if self.f[i].role == Role::Backup {
                go_idle(&mut self.f[i]);
            }
        }
        // Infielders called off a ball in the air back up the catch first (see
        // `assign_among`), while it is still in the air (its landing is a visible fact) and
        // only if no cover job took them.
        let in_air = self.ball_still_up();
        for i in 0..9 {
            match self.f[i].peel {
                Some(spot) if in_air && self.f[i].role == Role::Idle => {
                    self.f[i].target = spot;
                    self.f[i].role = Role::Backup;
                }
                _ => self.f[i].peel = None,
            }
        }
        // With nobody on base the catcher is not needed at home: he trails the batter to
        // back up first.
        if empty && self.f[1].role == Role::Cover(4) {
            go_idle(&mut self.f[1]);
        }
        let Some(from) = fp else { return };
        let jobs: Vec<(usize, Vec3, Option<u8>)> = (0..9).filter(|&o| self.f[o].role == Role::Idle).filter_map(|o| self.backup_job(o, from).map(|(t, b)| (o, t, b))).collect();
        for (o, target, backs) in jobs {
            self.f[o].target = target;
            self.f[o].role = Role::Backup;
            self.f[o].backs = backs;
        }
    }

    /// The backup job free fielder `o` takes, as he works out the staff's table himself
    /// (Codex fielder-communication review P1-1): the men free for it are he and every
    /// teammate he knows to have no other job (known idle, or known backing up: backups are
    /// dealt again here). A teammate's chase or cover not yet seen counts him as free. The
    /// spot and, for a base, which base.
    fn backup_job(&self, o: usize, from: Vec3) -> Option<(Vec3, Option<u8>)> {
        let occ = [self.occupied(1), self.occupied(2), self.occupied(3)];
        let on = |b: u8| occ[(b - 1) as usize];
        let depth = m::hypot(from.x, from.y);
        let outfield_hit = depth > INFIELD_R;
        let mut free: Vec<usize> = (0..9).filter(|&j| j == o || matches!(self.seen(o, j).0, Role::Idle | Role::Backup)).collect();
        // Behind the fielder: the nearest free outfielder gets into the ball's line beyond
        // him (deeper than the chase point, and no shallower than he already is).
        let line = Vec3::new(from.x, from.y, 0.0).normalized();
        let ofs: Vec<usize> = free.iter().copied().filter(|&i| i >= 6).collect();
        if let Some(&i) = ofs.iter().min_by(|&&a, &&c| (self.f[a].pos - from).norm().total_cmp(&(self.f[c].pos - from).norm()).then(a.cmp(&c))) {
            if i == o {
                let own = m::hypot(self.f[i].pos.x, self.f[i].pos.y);
                return Some((line * (depth + 8.0).max(own * 0.85), None));
            }
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
                // Nobody on: the pitcher backs up third, the base a batter-runner stretching
                // a hit in the gap or down a line goes for (owner, 2026-11-13; agent-models
                // §5.7; pitcher-outfield diagnosis Part B).
                jobs.push((2, vec![far_of, 7]));
                jobs.push((3, vec![0]));
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
            if i == o {
                // A throw to first with nobody on comes from the infield or from second base's
                // side; other throws come from where the ball is fielded.
                let origin = if base == 1 && outfield_hit { base_pos(2) } else { from };
                return Some((backup_spot(origin, base), Some(base)));
            }
            free.retain(|&x| x != i);
        }
        None
    }

    /// With the bases empty the catcher left home to back up first. Once any runner has
    /// rounded first he is a threat to score, and the catcher goes back (a visible fact:
    /// where the runners are).
    fn cover_home_again(&mut self) {
        if (0..9).any(|i| self.seen(1, i).0 == Role::Cover(4)) || matches!(self.ball, Ball::Held { by: 1 }) || self.f[1].role == Role::Chase {
            return;
        }
        if self.r.iter().any(|r| !r.out && !r.scored && r.s > 1.25 * BASE_L) {
            self.f[1].role = Role::Cover(4);
            self.f[1].backs = None;
            self.f[1].target = base_pos(4);
        }
    }

    /// A throw has been released toward `base` (a visible fact): its backup lines up with the
    /// real throw. A man with nothing to do who knows of nobody with that job, and is the
    /// nearest to its spot of the men he knows to be free, takes it himself.
    fn back_up_throw(&mut self, from: Vec3, base: u8) {
        let spot = backup_spot(from, base);
        let free = |o: usize, j: usize| {
            let (role, backs) = self.seen(o, j);
            matches!(role, Role::Idle | Role::Backup) && backs.is_none()
        };
        let takers: Vec<usize> = (0..9)
            .filter(|&o| {
                if self.f[o].role == Role::Backup && self.f[o].backs == Some(base) {
                    return true;
                }
                if !free(o, o) || (0..9).any(|j| j != o && self.seen(o, j) == (Role::Backup, Some(base))) {
                    return false;
                }
                (0..9).filter(|&j| free(o, j)).min_by(|&a, &c| (self.f[a].pos - spot).norm().total_cmp(&(self.f[c].pos - spot).norm()).then(a.cmp(&c))) == Some(o)
            })
            .collect();
        for i in takers {
            self.f[i].target = spot;
            self.f[i].role = Role::Backup;
            self.f[i].backs = Some(base);
            self.f[i].peel = None;
        }
    }

    /// Whether fielder `o`, working out the tables, counts `j` as going for the ball: a
    /// teammate as he is known to be, himself as he is. A man who has just stopped takes his
    /// table job at once; a teammate who still counts him as going may head for the same bag
    /// for up to `CALL_HEARD_S`, and the man earlier in the table's order keeps it. (A first
    /// version kept him out of his own table until his stop was known; on fly balls that
    /// several infielders start after, that left bases with nobody assigned for 0.6 s:
    /// Codex fielder-communication recheck P1-2.)
    fn chasing_as_known_by(&self, o: usize, j: usize) -> bool {
        if o == j { self.f[o].role == Role::Chase } else { self.known(j).0 }
    }

    /// The staff's coverage table (agent-models §5.7) as fielder `o` works it out for himself:
    /// from the side the ball is on, who he knows is going for it, and a middle infielder he
    /// knows is going to second; for himself, his own choice (`own_second`: he has left a
    /// ground ball to his partner and covers second; Codex fielder-communication review P1-1,
    /// Opus infield-fixes review P2-3). Each base's man, first base to home. Teammates work it
    /// out alike from what they know, so nobody has to be told it; what nobody can know is a
    /// choice not yet seen or heard, and the table may name such a man (he goes on with his
    /// own choice, and the base waits until it is known).
    fn cover_table(&self, o: usize, own_second: bool) -> [Option<usize>; 4] {
        let left = self.ball_side_x() < 0.0;
        // First base's last man, the second baseman, is dealt after the other bases: he takes
        // first only when second is someone else's (recheck P1-2: while the first baseman and
        // pitcher were still counted as going, he went to first and left second open).
        let order: [(usize, &[usize]); 5] = [(0, &[2, 0]), (1, if left { &[3, 5] } else { &[5, 3] }), (2, &[4, 5]), (3, &[1, 0]), (0, &[3])];
        let mut second: Vec<usize> = Vec::new();
        if own_second {
            second.push(o);
        }
        second.extend([3usize, 5].into_iter().filter(|&c| c != o && self.seen(o, c).0 == Role::Cover(2)));
        let mut taken: Vec<usize> = second.clone();
        let mut out = [None; 4];
        for &(k, cands) in order.iter() {
            if out[k].is_some() {
                continue;
            }
            if k == 1 && !second.is_empty() {
                out[1] = Some(second[0]);
                continue;
            }
            if let Some(c) = cands.iter().copied().find(|&c| !self.chasing_as_known_by(o, c) && !taken.contains(&c)) {
                taken.push(c);
                out[k] = Some(c);
            }
        }
        out
    }

    /// Re-assign base coverage among fielders who are not chasing, keeping the covers already
    /// given. Each man works out his own job (`cover_after_chasers`) from what he knows of
    /// the others, not from a choice one of them made a moment ago.
    /// The man holding the ball keeps what he is doing.
    fn cover_roles_after_chasers(&mut self) {
        let holder = match self.ball {
            Ball::Held { by } => Some(by),
            _ => None,
        };
        let jobs: Vec<(usize, Option<u8>)> = (0..9).filter(|&o| Some(o) != holder && !matches!(self.f[o].role, Role::Chase | Role::Relay(_))).map(|o| (o, self.cover_after_chasers(o))).collect();
        for (o, b) in jobs {
            match b {
                Some(b) if self.f[o].role != Role::Cover(b) => {
                    self.f[o].role = Role::Cover(b);
                    self.f[o].target = base_pos(b);
                }
                // A teammate he now knows of has his base: he leaves it.
                None if matches!(self.f[o].role, Role::Cover(_)) => go_idle(&mut self.f[o]),
                _ => {}
            }
        }
    }

    /// The base fielder `o` (not chasing, not the cutoff man) covers by the table kept as he
    /// knows it: the covers already taken stand, the man first in the table's order for a
    /// base keeping it when two are known to be on their way to one bag; open bases go to the
    /// first free man in their order.
    fn cover_after_chasers(&self, o: usize) -> Option<u8> {
        let job = |j: usize| self.seen(o, j).0;
        let left = self.ball_side_x() < 0.0;
        let mut taken: Vec<usize> = (0..9).filter(|&i| self.chasing_as_known_by(o, i) || matches!(job(i), Role::Relay(_))).collect();
        // The first baseman out as the cutoff to home: on a ball to the right side the second
        // baseman takes first and the pitcher stays free to back up home; to the left side
        // the second baseman is needed at second, so the pitcher takes first.
        // With the first baseman not out as the cutoff, the second baseman is first base's
        // last man and is dealt after the other bases (see `cover_table`).
        let (first, first_last) = match (matches!(job(2), Role::Relay(_)), left) {
            (true, false) => (vec![3, 0], vec![]),
            (true, true) => (vec![0, 3], vec![]),
            _ => (vec![2, 0], vec![3]),
        };
        let order: [(u8, Vec<usize>); 5] = [
            (1, first),
            (2, if left { vec![3, 5] } else { vec![5, 3] }),
            (3, vec![4, 5, 0]),
            (4, vec![1, 0]),
            (1, first_last),
        ];
        // Reserve kept non-chasing covers first so base 1's fallback never steals Cover(2).
        let mut filled = [false; 5];
        for base in 1..=4u8 {
            let cands: Vec<usize> = order.iter().filter(|e| e.0 == base).flat_map(|e| e.1.iter().copied()).collect();
            let rank = |i: usize| cands.iter().position(|&c| c == i).unwrap_or(cands.len());
            if let Some(c) = (0..9).filter(|&i| job(i) == Role::Cover(base) && !taken.contains(&i)).min_by_key(|&i| (rank(i), i)) {
                if c == o {
                    return Some(base);
                }
                taken.push(c);
                filled[base as usize] = true;
            }
        }
        for (base, cands) in order.iter() {
            if filled[*base as usize] {
                continue;
            }
            filled[*base as usize] = cands.iter().any(|c| !taken.contains(c));
            if let Some(&c) = cands.iter().find(|c| !taken.contains(c)) {
                // A man whose chase is not known yet goes on with it; the base waits.
                if c == o {
                    return Some(*base);
                }
                taken.push(c);
            }
        }
        None
    }

    /// The tables' cover jobs after a call (`cover_table`), each man by his own working of
    /// the table. A man who has just chosen to chase keeps chasing; a middle infielder in
    /// `second` keeps the bag he chose. The pitcher breaks for first on a ground ball his own
    /// read says the first baseman will go for. Everyone else with no job pulls up where
    /// braking stops him until `assign_backups` gives him one (agent-models §5.7;
    /// docs/calibration/2026-11-12-pitcher-outfield-diagnosis.md).
    fn cover_roles(&mut self, second: &[usize]) {
        let jobs: Vec<(usize, Option<u8>)> = (0..9)
            .filter(|&o| self.f[o].role != Role::Chase)
            .map(|o| {
                let table = self.cover_table(o, second.contains(&o));
                let base = table.iter().position(|&c| c == Some(o)).map(|k| k as u8 + 1);
                let breaks_for_first = o == 0 && base.is_none() && table[0] == Some(2) && self.pitcher_expects_first_baseman_break();
                (o, if breaks_for_first { Some(1) } else { base })
            })
            .collect();
        for (o, base) in jobs {
            match base {
                Some(b) => {
                    self.f[o].role = Role::Cover(b);
                    self.f[o].target = base_pos(b);
                }
                None => go_idle(&mut self.f[o]),
            }
        }
    }

    fn move_fielders(&mut self) {
        let (t, t_contact) = (self.t, self.inp.t_contact);
        let field = self.inp.field;
        for fd in self.f.iter_mut() {
            if fd.dove && t >= fd.ready_at {
                fd.dove = false; // recovered
            }
            (fd.pos, fd.vel) = fielder_step(fd, t, t_contact);
            (fd.pos, fd.vel) = wall_rule(field, fd.pos, fd.vel);
        }
    }

    /// At close range a chaser re-reads the ball (perception error scales with distance),
    /// runs at his new intercept and makes his call again from it.
    fn close_reads(&mut self) {
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
                        let (t, ball) = (self.t, self.ball_gen);
                        let fd = &mut self.f[i];
                        (fd.t_int, fd.int_pt, fd.target, fd.belief, fd.read_at, fd.read_ball) = (ti, pt, pt, Some(p), t, ball);
                        self.refresh_call(i);
                    }
                }
            }
        }
    }

    /// A chaser near the ball keeps asking whether he can still take it on his feet.
    fn dive_reads(&mut self) {
        // A chaser near the ball keeps asking whether he can still take it on his feet. Once
        // his read says no, he may leave them; the answer has to hold until the ball arrives.
        if let Ball::Free { .. } = self.ball {
            let (b, _) = self.ball_at(self.t);
            for i in 0..9 {
                let fd = &self.f[i];
                if fd.role != Role::Chase || self.t < fd.last_dive_read + DIVE_READ_S || m::hypot(b.x - fd.pos.x, b.y - fd.pos.y) > DIVE_READ_M {
                    continue;
                }
                let must = self.dive_gains(i, DIVE_GETUP_S);
                let fd = &mut self.f[i];
                fd.last_dive_read = self.t;
                fd.must_dive_since = if must { fd.must_dive_since.min(self.t) } else { f64::INFINITY };
            }
        }
    }

    /// Glove contact test for any fielder the ball passes (agent-models §5.4).
    fn try_captures(&mut self) {
        #[cfg(test)]
        iso_check(self, "close_reads");
        self.close_reads();
        #[cfg(test)]
        iso_check(self, "dive_reads");
        self.dive_reads();
        let (b, bv) = self.ball_at(self.t);
        let receiver = match self.ball {
            Ball::Thrown { receiver, .. } => Some(receiver),
            Ball::Free { .. } => None,
            Ball::Held { .. } => return,
        };
        // Catchable height where the ball is: a fielder at the wall can leap.
        let field = self.inp.field;
        let zmax_at = |p: Vec3| catch_ceiling(field, p);
        // A force at second still to be made: a feed there is a double-play pivot (§5.5).
        let force_at_second = self.force_pending_at_second();
        let zmax = zmax_at(b);
        for i in 0..9 {
            let fd = &self.f[i];
            if self.t < fd.blocked_until || self.t < self.inp.t_contact + fd.a.reaction {
                continue;
            }
            // Called off (Codex peel-off review #1): while the call stands he leaves the ball
            // to the caller, however close it comes; once it is down, off a glove, or handed
            // to him on a re-call, his job has ended and he may play it.
            if fd.peel.is_some() && fd.role == Role::Backup {
                continue;
            }
            // On a throw only the receiver tries, and nobody in its path in front of him; the
            // base's backup may take it only once it is beyond the receiver's reach on the far
            // side of him, so the receiver always has his own chance first.
            if let Some(rcv) = receiver.filter(|&rcv| rcv != i) {
                let past_receiver = match self.ball {
                    Ball::Thrown { to_base, relay: false, .. } if fd.backs == Some(to_base) => {
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
            // Nobody gets the glove farther past a wall than [`OVER_WALL_REACH_M`], standing,
            // stretching or diving: his body stops at the wall and cannot go through it. The
            // same limit the plan uses (`beyond_reach_over_wall`).
            if beyond_reach_over_wall(self.inp.field, b) > 0.0 {
                continue;
            }
            let horiz = m::hypot(b.x - fd.pos.x, b.y - fd.pos.y);
            let reach = if receiver.is_some() { RECEIVE_REACH } else { REACH };
            // Still on the ground from a dive (a rimmed ball that stayed close, agent-models
            // §5.4): he can grab a low ball within arm's reach, but cannot dive again.
            let down = fd.dove && self.t < fd.ready_at;
            let standing = horiz <= reach && b.z <= if down { DIVE_MAX_Z } else { zmax };
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
                    Ball::Thrown { path, .. } => t1 >= path.last().t,
                    Ball::Held { .. } => false,
                };
                let (pn, _) = fielder_step(fd, self.t, self.inp.t_contact);
                let hn = m::hypot(bn.x - pn.x, bn.y - pn.y);
                // (Still down then, he is held to a low ball then too: Codex dive review #2.)
                let zmax_next = if fd.dove && t1 < fd.ready_at { DIVE_MAX_Z } else { zmax_at(bn) };
                if !boundary && hn < horiz && hn <= reach && bn.z <= zmax_next && bn.z >= 0.0 {
                    continue;
                }
            }
            // Past his reach there are two ways to the ball (agent-models §5.3). A stretch on
            // his feet gets the glove `STRETCH` further. A dive gets it up to `DIVE` further,
            // but the body takes `DIVE_LUNGE_S` to lay out, so he has only the extension that
            // the time since he knew he must dive allows (and the pitcher, none until he has
            // recovered from his delivery). Either way the glove meets the ball where it
            // passes closest, as standing. Only a chaser, only a batted or loose ball, and
            // nobody lunges at a slow or resting ball: they walk up and field it.
            let can_go = |t: f64| t.max(self.inp.t_contact + if i == 0 { PITCHER_RECOVER_S } else { fd.a.reaction });
            let ext_at = |t: f64| DIVE * ((t - can_go(fd.must_dive_since)) / DIVE_LUNGE_S).clamp(0.0, 1.0);
            let how = |t: f64, h: f64, p: Vec3, zmax: f64| {
                if p.z < 0.0 {
                    None
                } else if h <= REACH + STRETCH && p.z <= zmax {
                    Some(false)
                } else if h <= REACH + ext_at(t) && p.z <= DIVE_MAX_Z {
                    Some(true)
                } else {
                    None
                }
            };
            let extending = if !standing && !down && receiver.is_none() && fd.role == Role::Chase && bv.norm() > 3.0 { how(self.t, horiz, b, zmax) } else { None };
            if let Some(dive) = extending {
                // An outfielder weighs a dive first: a ball that gets by him rolls to the
                // wall. He knows his own hands, and judges the ball by his own read of it
                // (where it is and how fast, from his belief, not the world's): the chance
                // that a glove placed with his error σ per axis is within the pocket. Below
                // `OF_DIVE_MIN_P` he stays on his feet. Once he has decided to go he goes
                // (Codex dive-rate review #1, #2).
                if dive && i >= 6 && !fd.will_dive.get() {
                    let Some((q, qv)) = fd.belief.as_ref().map(|p| p.at(self.t)) else { continue };
                    let h = m::hypot(q.x - fd.pos.x, q.y - fd.pos.y);
                    let sg = fd.a.hands * glove_sigma_mult(qv.norm(), h, true);
                    if 1.0 - m::exp(-0.5 * (POCKET / sg) * (POCKET / sg)) < OF_DIVE_MIN_P {
                        continue;
                    }
                    fd.will_dive.set(true);
                }
                let t1 = self.t + DT;
                let (bn, _) = self.ball_at(t1);
                let within = |x: f64| x > self.t && x <= t1;
                let boundary = match &self.ball {
                    Ball::Free { path, .. } => path.bounces.iter().any(|x| within(x.0)) || path.walls.iter().any(|x| within(x.0)) || path.t_end() <= t1,
                    _ => true,
                };
                let (pn, _) = fielder_step(fd, self.t, self.inp.t_contact);
                let hn = m::hypot(bn.x - pn.x, bn.y - pn.y);
                // Closer next tick, and still his to go for then: wait for it.
                if !boundary && hn < horiz && how(t1, hn, bn, zmax_at(bn)).is_some() {
                    continue;
                }
                // A stretch must gain him something too: not for a ball he can pick up on
                // his feet as soon as he would have his balance back.
                if !dive && !self.dive_gains(i, STRETCH_COST_S) {
                    continue;
                }
            }
            let diving = extending == Some(true);
            if !(standing || extending.is_some()) {
                continue;
            }
            let sg = fd.a.hands * glove_sigma_mult(bv.norm(), horiz, extending.is_some());
            let mut r = rng(self.inp.seed, self.inp.play, fd.id, Purpose::GlovePlacement, fd.attempts);
            let (ex, ez) = (r.truncated_normal(4.0), r.truncated_normal(4.0));
            let d = sg * m::hypot(ex, ez);
            // The job he was visibly doing as he took it (a cutoff man out on his relay spot):
            // what a runner can see of a reception, never the throw's private route.
            let shown_relay = matches!(self.known_job(i).0, Role::Relay(_));
            let fd = &mut self.f[i];
            fd.attempts += 1;
            if diving {
                // A committed dive puts the fielder on the ground whatever happens next. His
                // body goes until the glove is at the ball (arm's reach short of it).
                fd.dove = true;
                fd.ready_at = self.t + DIVE_GETUP_S;
                let to_ball = Vec3::new(b.x - fd.pos.x, b.y - fd.pos.y, 0.0);
                let mut land = fd.pos + to_ball * ((horiz - REACH).max(0.0) / horiz.max(1e-9));
                // Never through the wall: the body stops short of the first point of its
                // path that would put him within his body's width of it (the fair and foul
                // fences meet at an inward corner a straight lunge can cut; Codex dive
                // animation review #1).
                let field = self.inp.field;
                let inside = |p: Vec3| field.body_clearance(p.x, p.y, BODY_R).dist >= 0.0;
                let from = fd.pos;
                let mut ok = from;
                for j in 1..=LUNGE_CHECKS {
                    let q = from + (land - from) * (j as f64 / LUNGE_CHECKS as f64);
                    if !inside(q) {
                        break;
                    }
                    ok = q;
                }
                land = ok;
                fd.lunge = Some((self.t, fd.pos, land));
                let (id, from) = (fd.id, fd.pos);
                self.out.dives.push((self.t, id, from, land));
            }
            if d <= RIM && self.out.first_handling.is_none() {
                self.out.first_handling = Some((self.t, fd.id));
            }
            if d <= POCKET {
                // Up off the ground first (the get-up runs from his dive, whichever attempt
                // secured the ball), then the transfer.
                // (After a stretch he has to get his balance back.)
                let up_at = if fd.dove { fd.ready_at.max(self.t) } else if extending.is_some() { self.t + STRETCH_COST_S } else { self.t };
                fd.ready_at = up_at + fd.a.transfer;
                // What his set-up for a throw starts from (agent-models §5.5): he faces the
                // way he is running, or the ball coming to him, or the plate.
                let toward = |v: Vec3| (m::hypot(v.x, v.y) > FACE_MIN_MPS).then(|| Vec3::new(v.x, v.y, 0.0));
                fd.up_at = up_at;
                // Down as he takes it: diving now, or not yet up from an earlier dive (a
                // man whose get-up is over is on his feet, whatever the flag still says).
                fd.took_diving = fd.dove && up_at > self.t;
                fd.catch_stretched = extending == Some(false);
                fd.set_base = fd.a.transfer;
                fd.catch_vel = fd.vel;
                fd.catch_seen_vel = fd.vel;
                fd.catch_heading = Some(toward(fd.vel).or(toward(-bv)).unwrap_or(Vec3::new(-fd.pos.x, -fd.pos.y, 0.0)));
                fd.catch_received_throw = receiver.is_some();
                fd.catch_shown_relay = receiver.is_some() && shown_relay;
                fd.face = fd.catch_heading;
                if matches!(self.ball, Ball::Free { .. }) && self.out.ref_set_s.is_none() {
                    // The reference play's set-up for a throw to first: the grade-50 fielder of
                    // this position, from this catch (rules-spec §10.2).
                    let avg = FielderAttrs::average(((fd.id & 0x7f) as usize).clamp(1, 9) - 1);
                    let to = base_pos(1) - fd.pos;
                    let turn = fd.face.map(|f| throwing::turn_extra_s(f, to, fd.a.throws)).unwrap_or(0.0);
                    let su = SetUp::new(avg.transfer, turn, throwing::footwork_s(fd.catch_vel, to, fd.took_diving));
                    self.out.ref_set_s = Some(up_at - self.t + su.set_s);
                }
                // Having caught it, he plants: his run dies out where braking stops him, not
                // back at the spot he planned (which a running catch has carried him past).
                fd.target = stop_point(fd);
                fd.carry_to = None;
                (fd.pivot, fd.pivot_wait_bag, fd.relayed) = (false, false, false);
                // No longer racing the ball: he brakes onto that spot.
                fd.t_int = f64::INFINITY;
                if let Ball::Thrown { to_base, relay, .. } = self.ball {
                    // A receiver steps straight to the bag; a relay man stays put. Either goes
                    // on quickly, already facing his next target.
                    if relay {
                        go_idle(fd);
                    } else {
                        fd.target = base_pos(to_base);
                    }
                    // (Taking it on the ground from a dive, he is up first.) A receiver at a
                    // base needs 0.7 of his transfer; the cutoff man taking a relay has his own
                    // exchange, turn included (agent-models §5.5).
                    fd.set_base = if relay { throwing::relay_s(fd.a.transfer) } else { RELAY_TRANSFER_FRAC * fd.a.transfer };
                    fd.ready_at = up_at + fd.set_base;
                    fd.relayed = relay;
                    fd.face = None;
                    if !relay && to_base == 2 && force_at_second {
                        // The feed at second: a double-play pivot (agent-models §5.5). He faces
                        // the way he is moving, or the feed coming to him; his pivot starts
                        // when he has the bag.
                        fd.pivot = true;
                        fd.set_base = throwing::pivot_s(fd.a.transfer);
                        let moving = (m::hypot(fd.vel.x, fd.vel.y) > PIVOT_FACE_MIN_MPS).then(|| Vec3::new(fd.vel.x, fd.vel.y, 0.0));
                        fd.face = Some(moving.unwrap_or(Vec3::new(-bv.x, -bv.y, 0.0)));
                        fd.pivot_wait_bag = (base_pos(2) - fd.pos).norm() >= 0.6;
                    }
                }
                let id = fd.id;
                self.ball = Ball::Held { by: i };
                self.secured_at = self.t;
                self.out.secured.push((self.t, id));
                self.low_catch = diving || b.z < LOW_CATCH_Z;
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

    /// Whether going for the ball now, at a cost of `cost_s` before he can play on (the
    /// get-up from a dive, the balance lost to a stretch), gains the fielder anything, by
    /// his own read of the ball and his own running (agent-models §5.3). A ball still in the
    /// air: only a catch is an out, so it does unless he can catch it on his feet before it
    /// comes down. A ball on the ground: only if staying on his feet would get him the ball
    /// later than the cost.
    fn dive_gains(&self, i: usize, cost_s: f64) -> bool {
        let fd = &self.f[i];
        let Some(p) = &fd.belief else { return false };
        let Some((ts, _)) = self.intercept_scan(i, p).0 else { return true };
        if self.ball_still_up() {
            // Everyone sees that it has not come down; when it will is his own belief.
            p.first_bounce().is_some_and(|b| ts >= b.0)
        } else {
            ts - self.t > cost_s
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
        // A new ball (off a glove, or a throw on the ground): any call off the old one is over.
        self.ball_since = self.t;
        self.ball_gen += 1;
        for fd in self.f.iter_mut() {
            fd.peel = None;
            fd.must_dive_since = f64::INFINITY;
            fd.last_dive_read = f64::NEG_INFINITY;
            fd.will_dive.set(false);
        }
        #[cfg(test)]
        iso_check(self, "new_ball_assign");
        self.assign();
    }

    /// Emit ground, wall, fence, settle and passed-the-base facts as the free ball reaches them.
    fn ball_facts(&mut self) {
        let t = self.t;
        let floor = self.last_fact_t;
        let mut facts = Vec::new();
        let pending = matches!(&self.st.play, bb_rules::PlayState::Live(lp) if lp.fair == bb_rules::FairFoul::Pending);
        if let Ball::Free { path, next_touch, next_wall, settled_sent } = &mut self.ball {
            if pending {
                if let Some(f) = passed_base(path, self.inp.field, (t - DT).max(floor), t.min(path.t_end()), self.inp.env.ball.radius) {
                    facts.push(f);
                }
            }
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
                        // Where it left the field (fair or foul is judged there, rules-spec
                        // §5.2). A batted ball is dead only once it is down in the seats: until
                        // then a fielder at the rail may reach over for it (rules-spec §5.3).
                        let r = self.inp.env.ball.radius;
                        let exit = match path.segments.last() {
                            Some(bb_physics::ground::Segment::Flight { tr, .. }) => tr.find(bb_physics::flight::Crossing::Boundary { field: *self.inp.field, r }).map(|tc| tr.at(tc).0).or_else(|| {
                                // A path that starts at the wall (a throw over it): it left there.
                                let a = tr.first().s.r;
                                (self.inp.field.ball_clearance(a.x, a.y, r).dist <= 1e-6).then_some(a)
                            }),
                            _ => None,
                        };
                        let q = exit.unwrap_or(p);
                        facts.push(Input::BallTouched { t: path.t_end().max(floor), what: Touch::OutOfPlay { x: q.x, y: q.y, z: q.z, in_flight } });
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

    /// Put a throw aimed at ground point `aim` in the air from launch state `s0` now.
    fn launch_throw(&mut self, s0: BallState, aim: Vec3, to_base: u8, receiver: usize, relay: bool) {
        // The throw flies inside the park: it ends at the ground or at a wall (physics-spec §9.5).
        let path = fly_throw_in(self.inp.env, self.inp.field, s0, self.t, 15.0);
        // The receiver's read errors for this throw: a fresh look per throw, keyed by how
        // many throws the play has seen (sub-index 0 is his read of the batted ball).
        let mut r = rng(self.inp.seed, self.inp.play, self.f[receiver].id, Purpose::FielderPerception, 1 + self.out.released.len() as u32);
        let read_err = [r.truncated_normal(4.0), r.truncated_normal(4.0), r.truncated_normal(4.0), r.truncated_normal(4.0)];
        let next_read = self.t + self.f[receiver].a.reaction;
        self.ball = Ball::Thrown { path, aim, catch_at: aim, read_at: aim, read_err, next_read, to_base, receiver, relay };
    }

    /// The receiver's read of a throw in the air (agent-models §5.5 receiving): a noisy
    /// estimate of where the ball is and how it is moving now (the batted-ball read model:
    /// error grows with distance and shrinks with viewing time), flown on with his own throw
    /// physics. He takes it where it is catchable (from a scoop to a reach overhead) nearest
    /// the spot he was thrown to: on the bag when he believes it comes through there.
    /// Returns (where he believes he takes it, weighed against the aim; his read alone).
    fn receiver_read(&self, i: usize, aim: Vec3, err: [f64; 4], t_release: f64, relay: bool) -> (Vec3, Vec3) {
        let fd = &self.f[i];
        let (p, v) = self.ball_at(self.t);
        let view = (self.t - t_release - fd.a.reaction).max(0.0);
        let shrink = 1.0 / m::sqrt(1.0 + view / fd.a.tau_read);
        let dist = m::hypot(p.x - fd.pos.x, p.y - fd.pos.y).max(1.0);
        let sp = fd.a.read_sigma0 * (dist / READ_REF_DIST) * shrink;
        let los = Vec3::new(p.x - fd.pos.x, p.y - fd.pos.y, 0.0).normalized();
        let lat = Vec3::new(-los.y, los.x, 0.0);
        let pe = p + los * (err[0] * sp * DEPTH_STRETCH) + lat * (err[1] * sp) + Vec3::new(0.0, 0.0, err[1] * sp * 0.5);
        let sv = fd.a.read_vel_frac * v.norm() * shrink;
        let ve = v + los * (err[2] * sv * DEPTH_STRETCH * 0.5) + lat * (err[3] * sv) + Vec3::new(0.0, 0.0, err[3] * sv * 0.5);
        // He assumes a throw's typical backspin.
        let w = Vec3::new(ve.x, ve.y, 0.0).normalized().cross(Vec3::Z) * bb_math::units::rpm_to_radps(THROW_SPIN_RPM);
        let r = Vec3::new(pe.x, pe.y, pe.z.max(self.inp.env.ball.radius));
        // Flown until a second past the spot he was thrown to (he cares about nothing later).
        let vh = m::hypot(ve.x, ve.y).max(5.0);
        let horizon = m::hypot(aim.x - r.x, aim.y - r.y) / vh + 1.0;
        let pred = fly_throw(self.inp.env, BallState { r, v: ve, w }, self.t, horizon);
        let near = |q: &Vec3| m::hypot(q.x - aim.x, q.y - aim.y);
        let mut spots: Vec<Vec3> = pred.knots.iter().filter(|k| k.s.r.z >= THROW_SCOOP_Z && k.s.r.z <= CATCH_MAX_Z).map(|k| k.s.r).collect();
        let in_air = spots.iter().map(near).fold(f64::INFINITY, f64::min);
        // A throw to a base he reads coming down well short of him skips on: he takes it on
        // the hop or rolling where that is nearest his spot (agent-models §5.5: the low throw
        // through a cutoff man is meant to arrive on a hop), not where it last was at scoop
        // height. Within a step or two he steps in and takes it in the air; a relay man, out
        // in the open, always goes to take it in the air.
        if !relay && in_air > STEP_IN_M && pred.stop == Some(0) && pred.last().t < self.t + horizon {
            let land = pred.last();
            let on = batted_path_dt(self.inp.env, self.inp.field, land.s, land.t, THROW_HOP_DT, |_, _| (0.0, 0.0));
            let mut t = land.t;
            while t <= on.t_end().min(self.t + horizon) {
                let (q, _) = on.at(t);
                if q.z <= CATCH_MAX_Z {
                    spots.push(q);
                }
                t += THROW_HOP_DT;
            }
        }
        let raw = spots
            .into_iter()
            .min_by(|a, b| near(a).total_cmp(&near(b)))
            .unwrap_or(aim);
        // He knows throws mostly come where they are aimed, so he weighs his read against
        // that: a throw typically misses its spot by `THROW_MISS_FRAC` of its length; his
        // read's own error is its depth error now plus the velocity error flown to the spot.
        // Early, from far off, he barely moves; as the read firms up he goes where it says.
        let from = match &self.ball { Ball::Thrown { path, .. } => path.first().s.r, _ => p };
        let s_prior = (THROW_MISS_FRAC * m::hypot(aim.x - from.x, aim.y - from.y)).max(THROW_MISS_MIN_M);
        let t_left = m::hypot(aim.x - p.x, aim.y - p.y) / vh;
        let s_read = m::hypot(sp * DEPTH_STRETCH, sv * DEPTH_STRETCH * 0.5 * t_left);
        let k = s_prior * s_prior / (s_prior * s_prior + s_read * s_read);
        let at = aim + Vec3::new(raw.x - aim.x, raw.y - aim.y, 0.0) * k;
        (if m::hypot(at.x - aim.x, at.y - aim.y) < 0.5 { aim } else { at }, raw)
    }

    /// The receiver of a throw watches it and goes to where he believes he can take it,
    /// re-reading it as it comes (he starts on the spot it was thrown to).
    fn receiver_reads(&mut self) {
        // The receiver watches the throw and goes to where he believes he can take it,
        // re-reading it as it comes (he starts on the spot it was thrown to).
        if let Ball::Thrown { path, aim, read_err, next_read, receiver, relay, .. } = &self.ball {
            if self.t >= *next_read {
                let (aim, err, i, t0, relay) = (*aim, *read_err, *receiver, path.first().t, *relay);
                let (at, raw) = self.receiver_read(i, aim, err, t0, relay);
                if let Ball::Thrown { catch_at, read_at, next_read, .. } = &mut self.ball {
                    *catch_at = at;
                    *read_at = raw;
                    *next_read = self.t + RECEIVE_REPLAN_S;
                }
            }
        }
        let Ball::Thrown { catch_at, read_at, receiver, to_base, relay, .. } = &self.ball else { return };
        // Taking a throw at a bag, he keeps his foot on it and stretches for any throw he
        // believes he can reach from there (agent-models §5.8); he leaves it only for one
        // he cannot.
        let bag = base_pos(*to_base);
        // Both his weighed belief and his read alone must put it within reach of the bag
        // (Codex momentum review #4: the weighing alone can pin him to a bag a throw passes
        // out of reach of); when they disagree he steps to the weighed spot.
        let near = |p: &Vec3| m::hypot(p.x - bag.x, p.y - bag.y) <= BAG_REACH_FRAC * RECEIVE_REACH;
        let on_bag = !*relay && (1..=4).contains(to_base) && near(catch_at) && near(read_at);
        self.f[*receiver].target = if on_bag { Vec3::new(bag.x, bag.y, 0.0) } else { Vec3::new(catch_at.x, catch_at.y, 0.0) };
    }

    fn update_thrown(&mut self) {
        #[cfg(test)]
        iso_check(self, "receiver_reads");
        self.receiver_reads();
        let Ball::Thrown { path, .. } = &self.ball else { return };
        if self.t >= path.last().t {
            let (s, t_end, at_wall) = (path.last().s, path.last().t, path.stop == Some(1));
            if at_wall {
                // The throw reached a wall in the air (physics-spec §9.5): off it, or over it
                // and out of play.
                match wall_contact(self.inp.field, s, self.inp.env.ball.radius) {
                    (WallContact::Out, _) => {
                        // A throw over the wall is dead as it crosses. Its flight into the
                        // seats is for playback only.
                        let path = over_the_wall(self.inp.env, self.inp.field, s, t_end);
                        self.ball = Ball::Free { path, next_touch: 0, next_wall: 0, settled_sent: true };
                        let floor = self.last_fact_t;
                        self.emit(Input::BallTouched { t: t_end.max(floor), what: Touch::OutOfPlay { x: s.r.x, y: s.r.y, z: s.r.z, in_flight: true } });
                    }
                    (WallContact::Carom(ns), _) => {
                        let floor = self.last_fact_t;
                        self.emit(Input::BallTouched { t: t_end.max(floor), what: Touch::Wall { x: s.r.x, y: s.r.y } });
                        if !self.done {
                            self.set_free(ns);
                        }
                    }
                }
                return;
            }
            // The throw reached the ground: continue from there with the bounce model (a
            // short hop or a throw that got by).
            let mut s = s;
            s.r.z = s.r.z.max(self.inp.env.ball.radius);
            self.set_free(s);
        }
    }

    /// The runner's read of the throw in the air now (`ThrownRead`). One place that names the
    /// base a throw is going to, where a situational expectation (agent-models §6.4: the
    /// throw the defense should make from the outs, the runners and the fielder) can later
    /// feed it.
    fn thrown_read(&self, k: usize, sub: u32) -> ThrownRead {
        let (p, v) = self.runner_throw_observation(k, sub);
        self.project_runner_throw(p, v)
    }

    fn runner_throw_observation(&self, k: usize, sub: u32) -> (Vec3, Vec3) {
        let truth = self.ball_at(self.t);
        let from = self.runner_pos(&self.r[k]);
        let to = Vec3::new(truth.0.x - from.x, truth.0.y - from.y, 0.0);
        let view = self.out.released.last().map_or(0.0, |(t, _)| (self.t - t).max(0.0));
        let shrink = read_shrink(truth, view, RUNNER_READ_TAU_S, None);
        let sp = RUNNER_READ_SIGMA0 * (to.norm().max(1.0) / READ_REF_DIST) * shrink;
        let sv = READ_VEL_FRAC * truth.1.norm() * shrink;
        noisy_ball_motion(truth, to.normalized(), sp, sv, self.runner_ball_errors(k, sub))
    }

    /// Project only the observed motion, with a typical throw's spin and drag.
    fn project_runner_throw(&self, p: Vec3, v: Vec3) -> ThrownRead {
        let env = reference_env();
        let w = Vec3::new(v.x, v.y, 0.0).normalized().cross(Vec3::Z) * bb_math::units::rpm_to_radps(THROW_SPIN_RPM);
        let s0 = BallState { r: Vec3::new(p.x, p.y, p.z.max(env.ball.radius)), v, w };
        let tr = if s0.r.z == env.ball.radius && v.z <= 0.0 {
            // If his look puts a descending throw on the turf, he reads the hop now.
            Trajectory::single(self.t, s0)
        } else {
            fly_throw(env, s0, self.t, THROW_READ_S)
        };
        let mut track: Vec<(f64, Vec3)> = tr.knots.iter().map(|k| (k.t, k.s.r)).collect();
        // He takes it where it comes down through catching height; a throw that never gets
        // above it lands short, hops once, and he takes it where the hop ends.
        let down = (1..track.len()).find(|&i| track[i].1.z <= RUNNER_READ_CATCH_Z && track[i - 1].1.z > RUNNER_READ_CATCH_Z);
        let arrive = match down {
            Some(i) => i,
            None => {
                let last = *tr.last();
                let surf = self.inp.field.surface_at(last.s.r.x, last.s.r.y);
                let nb = bb_physics::ground::bounce(env, last.s, &surf, (0.0, 0.0));
                if nb.v.z > 0.0 {
                    let hop = fly_throw(env, nb, last.t, THROW_READ_S);
                    track.extend(hop.knots.iter().map(|k| (k.t, k.s.r)));
                }
                track.len() - 1
            }
        };
        let near = |q: Vec3| {
            (1..=4u8)
                .map(|b| (b, m::hypot(base_pos(b).x - q.x, base_pos(b).y - q.y)))
                .min_by(|x, y| x.1.total_cmp(&y.1))
                .unwrap()
        };
        let (t_down, q_down) = track[arrive];
        // A throw coming down well short of every bag is a low throw meant to carry on along
        // the ground (agent-models §5.5, through the cutoff man): he follows it on through the
        // bounce, with the same physics, and takes the last base it comes within reach of at
        // catching height as the one it is going to (Opus catch-throw review P3-1). Otherwise
        // the base nearest where it comes down.
        if down.is_some() && near(q_down).1 > 2.0 * RECEIVE_REACH {
            let land = (arrive..tr.knots.len()).map(|i| tr.knots[i]).find(|k| k.s.r.z <= env.ball.radius + 1e-6).or_else(|| (tr.stop == Some(0)).then(|| *tr.last()));
            if let Some(land) = land {
                let on = batted_path_dt(env, self.inp.field, land.s, land.t, THROW_HOP_DT, |_, _| (0.0, 0.0));
                let mut full: Vec<(f64, Vec3)> = track[..=arrive].to_vec();
                full.extend(tr.knots.iter().filter(|k| k.t > t_down && k.t < land.t).map(|k| (k.t, k.s.r)));
                let mut t = land.t;
                while t <= on.t_end().min(self.t + THROW_READ_S) {
                    full.push((t, on.at(t).0));
                    t += THROW_HOP_DT;
                }
                let reached = full.iter().filter(|(_, q)| q.z <= PASS_MAX_Z).filter_map(|&(t, q)| {
                    let (b, d) = near(q);
                    (d <= PASS_REACH_M).then_some((t, b))
                }).last();
                if let Some((t_b, b)) = reached {
                    // When it is nearest that bag.
                    let bp = base_pos(b);
                    let (t_arrive, _) = full.iter().filter(|(t, _)| *t <= t_b + 1e-9).map(|&(t, q)| (t, m::hypot(q.x - bp.x, q.y - bp.y))).filter(|x| x.1 <= PASS_REACH_M).min_by(|x, y| x.1.total_cmp(&y.1)).unwrap_or((t_b, 0.0));
                    let cut = full.iter().position(|(t, _)| *t > t_arrive).unwrap_or(full.len());
                    full.truncate(cut);
                    return ThrownRead { dest: b, t_arrive, track: full };
                }
            }
        }
        track.truncate(arrive + 1);
        let dest = near(q_down).0;
        ThrownRead { dest, t_arrive: t_down, track }
    }

    fn observe_holder(&self, by: usize) -> HolderLook {
        let fd = &self.f[by];
        HolderLook {
            pos: fd.pos,
            secured: self.out.secured.last().map(|(t, _)| *t)
                .or_else(|| self.secured_at.is_finite().then_some(self.secured_at)),
            heading: fd.catch_heading,
            motion: fd.catch_seen_vel,
            received_throw: fd.catch_received_throw,
            shown_relay: fd.catch_shown_relay,
            stretched: fd.catch_stretched,
            lunge_at: fd.lunge.map(|(t, _, _)| t),
        }
    }

    /// Runners' believed time for the ball to reach base `b` (they watch the ball and the
    /// fielders; agent-models §6.3).
    fn ball_time_to(&self, k: usize, b: u8, read_sub: u32, my_at: f64) -> f64 {
        match &self.ball {
            Ball::Held { by } => {
                // The runner sees where the ball is, when it was caught, and who has it: his
                // read of that fielder is the position's grade-50 transfer and the fielder's
                // own arm (scouting knows an arm), off by his own read error (agent-models
                // §6.4; Gemini tag-up review P2-1; one draw per runner, fielder and play).
                let look = self.observe_holder(*by);
                let mean = FielderAttrs::average(*by);
                let err = rng(self.inp.seed, self.inp.play, self.r[k].id, Purpose::RunnerPerception, ARM_READ_SUB + *by as u32).truncated_normal(4.0);
                let fd = &self.f[*by];
                let arm = fd.a.arm + ARM_READ_SIGMA_MPS * err;
                let direct = |base: u8| {
                    let to = base_pos(base) - look.pos;
                    // He sees the fielder come through the catch toward that base (agent-models
                    // §5.5): his arm and that momentum; and whether a cutoff man stands in the throw.
                    let gain = if look.lunge_at.is_some() { 0.0 } else { throwing::momentum_gain_mps(look.motion, to) };
                    look.release_time(self.t, base, mean, fd.a.throws)
                        + read_throw_time(arm + gain / THROW_EFFORT, m::hypot(to.x, to.y), self.cut_seen(None, look.pos, base))
                };
                self.with_expected_play(k, b, my_at, &direct)
            }
            Ball::Thrown { .. } => {
                // One keyed look feeds every base race in this decision.
                self.thrown_read(k, read_sub).time_to(b)
            }
            Ball::Free { .. } => {
                let t0 = prof::enabled().then(std::time::Instant::now);
                let pred = self.runner_predict(k, read_sub);
                let time = self.pickup_time_to(k, b, my_at, &pred);
                if let Some(t0) = t0 {
                    prof::add(&prof::RUNNER, t0.elapsed().as_nanos() as u64);
                }
                time
            }
        }
    }

    /// Time both races from the same projection of an airborne throw.
    fn runner_ball_races(&self, k: usize, sub: u32, ahead: (u8, f64), behind: (u8, f64)) -> (f64, f64) {
        if matches!(self.ball, Ball::Thrown { .. }) {
            let read = self.thrown_read(k, sub);
            (read.time_to(ahead.0), read.time_to(behind.0))
        } else {
            (self.ball_time_to(k, ahead.0, sub, ahead.1), self.ball_time_to(k, behind.0, sub, behind.1))
        }
    }

    fn pickup_time_to(&self, k: usize, b: u8, my_at: f64, pred: &BallPath) -> f64 {
        let end = pred.t_end();
        let (tf, at) = self.believed_field_intercept(pred).unwrap_or_else(|| {
            let (rest, _) = pred.at(end);
            (end + 1.0, rest)
        });
        self.with_expected_play(k, b, my_at, &|base| {
            let target = base_pos(base);
            tf + TYPICAL_TRANSFER + typical_throw_time(m::hypot(target.x - at.x, target.y - at.y), self.cut_seen(None, at, base))
        })
    }

    /// The ball's time to base `b` for runner `k`, given the play he expects the defense to
    /// make (agent-models §6.3, "the defense's play"; owner, 2026-11-15: situation first).
    /// `direct(base)` is his belief of a throw straight to a base; `my_at` is when he would
    /// reach `b`. He runs the fielders' own value rule (§5.6: an out attempt needs
    /// p > 0.3, the best p² × weight wins) on what he can see: every other runner's visible
    /// heading and speed, a league-typical runner's legs for them, his own for himself. If
    /// the best play is on another runner at another base, the throw goes there, and the
    /// ball reaches `b` only by a second throw from that base, or by the cutoff man taking
    /// it on the way and turning (`CUT_EXTRA_S` after a straight throw would have). If the
    /// defense has no play anywhere, or its best play is on him, the throw is his to beat.
    fn with_expected_play(&self, k: usize, b: u8, my_at: f64, direct: &dyn Fn(u8) -> f64) -> f64 {
        let own = direct(b);
        let tag = self.inp.tag_up.typical_tag_s;
        let value = |j: usize, base: u8, runner_at: f64, ball_at: f64| {
            let r = &self.r[j];
            let force = self.runner_forced(k, j) && base == r.start + 1 && r.s < base as f64 * BASE_L;
            let p = phi((runner_at - ball_at - if force { 0.0 } else { tag }) / 0.12);
            if p > 0.3 { p * p * out_weight_at(base, self.runner_outs(k), force) } else { 0.0 }
        };
        let mine = value(k, b, my_at, own);
        let params = throwing::ChoiceParams::default();
        let mut best: Option<(f64, u8)> = None;
        for (j, o) in self.r.iter().enumerate() {
            if j == k || self.runner_sees_out(k, o) || o.scored || o.overrun != OverrunPhase::None {
                continue;
            }
            let Some(base) = visible_target(o, params) else { continue };
            if base == b {
                continue;
            }
            let v = value(j, base, self.t + runner_eta((base as f64 * BASE_L - o.s).abs(), o.v), direct(base));
            if v > mine && best.is_none_or(|x| v > x.0) {
                best = Some((v, base));
            }
        }
        match best {
            None => own,
            Some((_, dest)) => (direct(dest) + RELAY_READ_S + typical_throw_time((base_pos(b) - base_pos(dest)).norm(), Cut::No)).min(own + CUT_EXTRA_S),
        }
    }

    /// Forced while the batter is still a runner and every base behind this runner is held by
    /// a runner still legally in play.
    fn forced(&self, k: usize) -> bool {
        self.force_with(k, |r| r.out)
    }

    /// The rules have recorded a catch runner `k` has not yet seen called: its out and the
    /// force it removes are hidden from his reads.
    fn awaiting_catch_call(&self, k: usize) -> bool {
        self.caught() && !self.r[k].catch_seen
    }

    /// Runner `k` saw a glove take the ball in flight and has not yet reacted to the call,
    /// whichever way it went: he holds new decisions until then (catch-read fixes review
    /// P1-3). With no catch ruled, the call he sees is that possession was no catch.
    fn awaiting_call(&self, k: usize) -> bool {
        self.awaiting_catch_call(k)
            || (!self.caught() && self.flight_secured_at.is_some_and(|ts| self.t + 1e-9 < ts + RUNNER_CATCH_CALL_REACT_S))
    }

    fn runner_sees_out(&self, k: usize, other: &Runner) -> bool {
        other.out && !(other.start == 0 && self.awaiting_catch_call(k))
    }

    fn runner_outs(&self, k: usize) -> u8 {
        self.st.outs.saturating_sub(u8::from(self.awaiting_catch_call(k)))
    }

    fn runner_forced(&self, observer: usize, k: usize) -> bool {
        self.force_with(k, |r| self.runner_sees_out(observer, r))
    }

    fn force_with(&self, k: usize, is_out: impl Fn(&Runner) -> bool) -> bool {
        let has_runner = |base| self.r.iter().any(|r| r.start == base && !is_out(r));
        let start = self.r[k].start;
        start == 0 || (0..start).all(has_runner)
    }

    /// How much the holder values an out on runner `k` at base `goal`. With fewer than two out
    /// the lead runner is worth more. With two out every out ends the inning, so the sure out
    /// is the play, and a force is preferred to a tag (it needs no tag, and a run cannot score
    /// on it; OBR 5.08(a)). It is a force only while he is still short of the base: a runner
    /// coming back to a base he has already reached must be tagged (Codex dive review #1).
    fn out_weight(&self, k: usize, goal: u8) -> f64 {
        let r = &self.r[k];
        out_weight_at(goal, self.st.outs, self.forced(k) && goal == r.start + 1 && r.s < goal as f64 * BASE_L)
    }

    /// Whether a runner is on base `b` or between it and the next, used to choose cutoff
    /// and backup assignments from visible runner positions.
    fn occupied(&self, b: u8) -> bool {
        self.r.iter().any(|r| !r.out && !r.scored && (r.s / BASE_L + 1e-6).floor() as u8 == b)
    }

    /// A runner forced to second who has not yet reached it (a force there is still to be
    /// made, §5.4): what makes a feed to second a double-play pivot.
    fn force_pending_at_second(&self) -> bool {
        (0..self.r.len()).any(|k| !self.r[k].out && self.r[k].start == 1 && self.forced(k) && self.r[k].s < 2.0 * BASE_L - 0.1)
    }

    /// Whether a runner ahead of runner `k` holds base `b`, as `k` can see it: he is at or
    /// short of it and has visibly not been moving forward (standing, or going back) for
    /// `HOLD_READ_S`. His intended goal is private (Gemini A4 review; agent-models §6.4).
    fn base_held_ahead(&self, k: usize, b: u8) -> bool {
        let r = &self.r[k];
        self.r.iter().any(|o| {
            o.id != r.id
                && !self.runner_sees_out(k, o)
                && !o.scored
                && o.start > r.start
                && ((o.s <= b as f64 * BASE_L + 0.5 && o.still_since.is_some_and(|t0| self.t - t0 >= HOLD_READ_S - 1e-9))
                    // Visibly coming back to it from beyond: it is his.
                    || (o.direction * o.v < -SEEN_MOVING_MPS && (o.s / BASE_L).floor() as u8 == b))
        })
    }

    /// Whether a runner ahead of runner `k` has not yet passed base `b`. Nobody stops at
    /// home. A runner ahead who is past third and visibly running in is clear if he will be
    /// home first: by a typical runner's legs from where he is and how
    /// fast he is moving, `FOLLOW_GAP_S` ahead of this runner's own best time there, so that
    /// following him is never passing him (Codex runner-decision review P1).
    fn base_uncleared_ahead(&self, k: usize, b: u8) -> bool {
        let r = &self.r[k];
        let mine = run_time_from((b as f64 * BASE_L - r.s).max(0.0), (r.direction * r.v).max(0.0), r.vmax, r.tau);
        self.r.iter().any(|o| {
            let running_home = b == 4 && o.s > 3.0 * BASE_L + 0.5 && o.direction * o.v > SEEN_MOVING_MPS && runner_eta(4.0 * BASE_L - o.s, o.v) + FOLLOW_GAP_S <= mine;
            o.id != r.id && !self.runner_sees_out(k, o) && !o.scored && o.start > r.start && o.s <= b as f64 * BASE_L + 0.5 && !running_home
        })
    }

    /// What runner `k` weighs in going from base `from` to the next (agent-models §6.3): the
    /// run expectancy of staying, of making it and of being put out, from the outs on the
    /// board, where the other runners visibly are and are heading, and the score. All public:
    /// a runner sees the bases and the scoreboard. Another runner is counted at the base he
    /// is moving toward (or the one he is on), never past the base this runner would have: a
    /// trailing runner stays behind him, a lead runner ahead. Runs home earlier in the play
    /// are in none of the three. A runner ahead who is visibly on his way home is a run in
    /// all three, except that with two out it is lost if this runner is put out first:
    /// `out_at` is when that out would come (his own arrival at the base), against a typical
    /// runner's time home (Codex runner-decision review P2).
    fn advance_values(&self, k: usize, from: u8, out_at: f64) -> bb_agents::running::AdvanceValues {
        use bb_agents::running::{run_expectancy, run_weight, AdvanceValues};
        let me = &self.r[k];
        let next = from + 1;
        // The others' bases with this runner at `mine` (0: he is out), and the runs of any
        // runner ahead whom his advance sends home.
        let state = |mine: u8| -> (u8, f64) {
            let (mut bits, mut runs) = (0u8, 0.0);
            for o in self.r.iter().filter(|o| o.id != me.id && !self.runner_sees_out(k, o) && !o.scored) {
                let at = o.s / BASE_L;
                let fwd = o.direction * o.v;
                let seen = if fwd > SEEN_MOVING_MPS {
                    (at - 1e-6).ceil()
                } else if fwd < -SEEN_MOVING_MPS {
                    (at + 1e-6).floor()
                } else {
                    at.round()
                } as i32;
                let trailing = o.start < me.start;
                let b = if mine == 0 {
                    seen
                } else if trailing {
                    seen.min(mine as i32 - 1)
                } else {
                    seen.max(mine as i32 + 1)
                }
                .max(1);
                if (1..=3).contains(&b) {
                    bits |= 1 << (b - 1);
                } else if seen < 4 {
                    runs += 1.0;
                } else if mine != 0 || self.runner_outs(k) < 2 || self.t + runner_eta((4.0 * BASE_L - o.s).max(0.0), o.v) < out_at {
                    // Already heading home: his run counts unless the third out comes first.
                    runs += 1.0;
                }
            }
            if (1..=3).contains(&mine) {
                bits |= 1 << (mine - 1);
            }
            (bits, runs)
        };
        let outs = self.runner_outs(k);
        let value = |mine: u8, outs: u8| {
            let (bits, runs) = state(mine);
            run_expectancy(bits, outs) + runs
        };
        let v = AdvanceValues { hold: value(from, outs), safe: value(next, outs) + if next == 4 { 1.0 } else { 0.0 }, out: value(0, outs + 1) };
        let bat = if self.st.half == bb_rules::Half::Top { 0 } else { 1 };
        let runs = |side: usize| self.st.runs_by_inning[side].iter().map(|&x| x as i32).sum::<i32>();
        v.weighted(run_weight(self.st.inning, runs(bat) - runs(1 - bat)))
    }

    fn runner_decide(&mut self, k: usize, at_base: u8) {
        self.runner_decide_with_contact(k, at_base, None);
    }

    fn runner_decide_with_contact(&mut self, k: usize, at_base: u8, contact: Option<&BallPath>) {
        if self.awaiting_call(k) {
            return;
        }
        let forced = self.runner_forced(k, k);
        let next = at_base + 1;
        if next > 4 {
            return;
        }
        // Every decision is a new look at the play: its own noise draws (data-formats §2.3).
        self.r[k].decisions += 1;
        self.r[k].looked = (self.out.secured.len(), self.out.released.len());
        let n = self.r[k].decisions;
        let r = &self.r[k];
        let d_run = (next as f64 * BASE_L - r.s).max(0.0);
        let t_run = match r.set_start {
            // Going from a tag-up: the same start his legs will run (§6.4).
            Some(leave) => tag_up_time(self.inp.tag_up, (leave - self.t).max(0.0), d_run, r.vmax, r.tau),
            // From his own speed now, by the law his legs run (a runner arriving at a base
            // at speed carries it on; standing, it is a start from rest).
            None => run_time_from(d_run, r.direction * r.v, r.vmax, r.tau),
        };
        // Unforced, he must beat the tag, not only the ball: his belief of the time it takes.
        let tag_s = if forced { 0.0 } else { self.inp.tag_up.typical_tag_s };
        let t_ball = match contact {
            Some(pred) => self.pickup_time_to(k, next, self.t + t_run, pred),
            None => self.ball_time_to(k, next, 2 * n + 1, self.t + t_run),
        } + tag_s;
        let mut rr = rng(self.inp.seed, self.inp.play, r.id, Purpose::RunnerPerception, 2 * n);
        let draw = rr.truncated_normal(4.0);
        let noise = RUNNER_TIMING_SIGMA_S * draw;
        let must = forced && at_base == r.start && !r.catch_seen;
        // A runner may only take a base once every runner ahead of him has visibly cleared it;
        // until then he waits at his base.
        let blocked = !must && self.base_uncleared_ahead(k, next);
        // His margin by his read, weighed against what the base is worth (§6.3).
        let go = !blocked && (must || self.inp.advance.goes(t_ball - (self.t + t_run) + noise, &self.advance_values(k, at_base, self.t + t_run)));
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
        self.caught_at.is_some()
    }

    fn no_catch_read(&self) -> bool {
        self.no_catch_at.is_some_and(|call| self.t + 1e-9 >= call + RUNNER_CATCH_CALL_REACT_S)
    }

    /// Whether runner `j` is at the point of settling on base `b`, as runner `k` sees it.
    /// His own (`j == k`): it is his goal and he stands on it or is too near to stop short of
    /// it. Another runner's goal is private (catch-read fixes review P1-4), so for him it is
    /// what `k` can see: standing on the bag, or coming at it too fast to stop short of it
    /// (his speed over `HOLD_READ_S`, then `BRAKE`), whether he means to stop there or not.
    /// Only bases 1–3: any number of runners may score.
    fn settles_on(&self, k: usize, j: usize, b: u8) -> bool {
        let o = &self.r[j];
        if self.runner_sees_out(k, o) || o.scored || !(1..=3).contains(&b) || (j == k && o.goal != b) {
            return false;
        }
        let bs = b as f64 * BASE_L;
        let gap = bs - o.s;
        if gap.abs() < 1e-6 {
            return true;
        }
        let toward = o.direction * gap.signum() * o.v;
        toward > SEEN_MOVING_MPS && gap.abs() <= toward * HOLD_READ_S + toward * toward / (2.0 * BRAKE) + 0.3
    }

    /// Whether runner `t`, standing on `b`, gives way there, as runner `k` sees it: no force
    /// on him to it and a free base behind him.
    fn gives_way(&self, k: usize, t: usize, b: u8) -> bool {
        let standing = (self.r[t].s - b as f64 * BASE_L).abs() < 1e-6;
        let forced_to_b = b == self.r[t].start + 1 && self.runner_forced(k, t);
        standing && !forced_to_b && self.refuge_free(k, t, b)
    }

    /// The invariant at the place bodies stop: two live runners never come to rest on one
    /// base (agent-models §6.4; Codex runner-reads rechecks). Run on the final goals of the
    /// tick, whatever decision path set them (look, held base, turn-back, cap), so none can
    /// bypass it. Each runner changes only his own goal, from what he sees of the others
    /// (positions, signed speeds, original bases; catch-read fixes review P1-4). Of two
    /// runners settling on a base the trailing one gives way if he is standing on it with
    /// a free base behind him and no force on him; otherwise the lead runner goes on to the
    /// next base (a force pushes him, and a runner who cannot stop short cannot give way).
    fn settle_guard(&mut self, k: usize) {
        for _ in 0..4 {
            let b = self.r[k].goal;
            if !self.settles_on(k, k, b) {
                return;
            }
            let start = self.r[k].start;
            let others: Vec<usize> = (0..self.r.len()).filter(|&j| j != k && self.settles_on(k, j, b)).collect();
            let behind_him = others.iter().any(|&j| self.r[j].start > start);
            let goal = if behind_him && self.gives_way(k, k, b) {
                b - 1
            } else if others.iter().any(|&j| self.r[j].start < start && !self.gives_way(k, j, b)) {
                (b + 1).min(4)
            } else {
                return;
            };
            let r = &mut self.r[k];
            r.goal = goal;
            r.waiting = false;
            r.turned_back = None;
        }
    }

    /// Whether the base behind runner `j` standing on `b` is free for him to go back to, as
    /// runner `k` sees it: not home (the batter-runner has none), no other runner on it, and
    /// none visibly on his way to it.
    fn refuge_free(&self, k: usize, j: usize, b: u8) -> bool {
        let back = b - 1;
        let bs = back as f64 * BASE_L;
        back >= 1 && !self.r.iter().enumerate().any(|(i, x)| {
            i != j && !self.runner_sees_out(k, x) && !x.scored && {
                let gap = bs - x.s;
                gap.abs() < 0.5 || (gap.abs() < BASE_L && x.direction * gap.signum() * x.v > SEEN_MOVING_MPS)
            }
        })
    }

    /// Whether base `b`, behind runner `k`, is no refuge for him, as he can see it: a runner
    /// who started behind him stands on it or is past it, or is on the way to it (it is the next
    /// base ahead of him) and cannot stop short of it or has no way back himself (positions and
    /// speeds, never goals; agent-models §6.4). A trailing runner who can stop is expected to
    /// give way; a distant one threatens nothing.
    fn return_crowded(&self, k: usize, b: u8) -> bool {
        let lead = &self.r[k];
        let bs = b as f64 * BASE_L;
        self.r.iter().enumerate().any(|(j, o)| {
            if j == k || self.runner_sees_out(k, o) || o.scored || o.start >= lead.start {
                return false;
            }
            if o.s + 1e-6 >= bs {
                return true;
            }
            let v_fwd = o.direction * o.v;
            if v_fwd <= SEEN_MOVING_MPS || (o.s / BASE_L).floor() as u8 + 1 != b {
                return false;
            }
            // He reads the lead runner's turn after `HOLD_READ_S`, then brakes at `BRAKE`.
            let stop = o.s + v_fwd * HOLD_READ_S + v_fwd * v_fwd / (2.0 * BRAKE);
            stop + 0.3 >= bs || self.no_way_back(k, j)
        })
    }

    /// The trailing runner giving way: a lead runner visibly coming back to the base he is
    /// heading for, and a base of his own to go back to (the guard `settle_guard` is the
    /// last word at the base).
    fn runner_yields(&mut self, k: usize) {
        let r = &self.r[k];
        if r.out || r.scored || r.retouch || r.tag_up || r.overrun != OverrunPhase::None || r.hold_s.is_some() {
            return;
        }
        let from = (r.s / BASE_L).floor() as u8;
        let between = (r.s - from as f64 * BASE_L).abs() > 1e-6;
        let Some(g) = (r.goal as f64 * BASE_L > r.s + 1e-6).then_some(r.goal) else { return };
        let (id, start, s) = (r.id, r.start, r.s);
        let coming_back = self.r.iter().any(|o| {
            o.id != id && !self.runner_sees_out(k, o) && !o.scored && o.start > start && o.direction * o.v < -SEEN_MOVING_MPS && o.s > s && ((o.s / BASE_L).floor() as u8) == g
        });
        let forced_here = g == start + 1 && self.runner_forced(k, k);
        if coming_back && !forced_here && between && !self.no_way_back(k, k) {
            self.r[k].goal = from;
            self.r[k].turned_back = Some(g);
        }
    }

    /// Whether runner `j`, between bases, has no base to go back to: the batter-runner (home
    /// is not his), or one with another runner standing on the base he left.
    fn no_way_back(&self, k: usize, j: usize) -> bool {
        let o = &self.r[j];
        let f = (o.s / BASE_L).floor();
        o.start == 0 && f < 1.0 || self.r.iter().enumerate().any(|(i, x)| i != j && !self.runner_sees_out(k, x) && !x.scored && (x.s - f * BASE_L).abs() < 0.5)
    }

    /// A runner between bases looks again when the play has changed under him: the ball
    /// secured by a fielder, or a throw released, since he last looked (agent-models §6.4).
    /// He is between bases, on his way (a runner at a base, tagging up, waiting on one
    /// ahead of him, holding on a ball in the air, or coming back to retouch has his own
    /// rules). Returns whether he decided.
    fn runner_relook(&mut self, k: usize) -> bool {
        let seen = (self.out.secured.len(), self.out.released.len());
        let r = &self.r[k];
        if r.looked == seen {
            return false;
        }
        let nearest = (r.s / BASE_L).round();
        let between = (r.s - nearest * BASE_L).abs() > 1e-6;
        // Behind his own base (carried past it coming back) he has no base behind him to
        // choose: his own is ahead, and he goes on to it.
        let behind_own = r.s < r.start as f64 * BASE_L - 1e-6;
        if !between || behind_own || r.tag_up || r.retouch || r.waiting || r.hold_s.is_some() || r.turned_back.is_some() || r.overrun != OverrunPhase::None || r.set_start.is_some() {
            return false;
        }
        let (start, s_now) = (r.start, r.s);
        self.r[k].looked = seen;
        let from = (s_now / BASE_L).floor() as u8;
        let next = from + 1;
        // Forced to the next base, he has no choice (the force is on him as long as it is the
        // base after his own); a runner ahead who has not cleared it also leaves him no go.
        if next > 4 || (next == start + 1 && self.runner_forced(k, k)) || self.base_uncleared_ahead(k, next) {
            return false;
        }
        self.r[k].decisions += 1;
        let n = self.r[k].decisions;
        let r = &self.r[k];
        let (d_on, d_back) = (next as f64 * BASE_L - r.s, r.s - from as f64 * BASE_L);
        // From his own signed velocity (forward positive): with a running start only if he is
        // moving that way; moving the other way the time includes the stop, as the movement
        // step makes him brake (`run_time_from`, the same laws and `BRAKE`).
        let v_fwd = r.direction * r.v;
        let t_on = run_time_from(d_on, v_fwd, r.vmax, r.tau);
        let t_back = run_time_to_stop(d_back, -v_fwd, r.vmax, r.tau);
        // Unforced, he must beat the tag at either base, not only the ball.
        let tag_s = self.inp.tag_up.typical_tag_s;
        let (ball_on, ball_back) = self.runner_ball_races(k, 2 * n + 1, (next, self.t + t_on), (from, self.t + t_back));
        let mut rr = rng(self.inp.seed, self.inp.play, r.id, Purpose::RunnerPerception, 2 * n);
        let draw = rr.truncated_normal(4.0);
        let noise = RUNNER_TIMING_SIGMA_S * draw;
        let margin = |t_ball: f64, t_self: f64| t_ball + tag_s - (self.t + t_self) + noise;
        let (m_on, m_back) = (margin(ball_on, t_on), margin(ball_back, t_back));
        // Whether he believes he makes each base at all (better than even), and which he
        // chooses by what each is worth (§6.3).
        let adv = &self.inp.advance;
        let (go_on, go_back) = (adv.p_safe(m_on) > 0.5, adv.p_safe(m_back) > 0.5);
        let prefers_on = adv.chooses_on(m_on, m_back, &self.advance_values(k, from, self.t + t_on.min(t_back)));
        // Safe at neither: he turns toward the base the ball is farther from (a rundown is
        // not modelled; either way he is probably out).
        // The base behind him is no refuge once a runner who started behind him stands on it
        // or has passed it (two on a base; the force is off, but he is not alone on the path).
        let crowded = self.return_crowded(k, from);
        let goal = if crowded {
            next
        } else if go_on || go_back {
            if prefers_on { next } else { from }
        } else if ball_back >= ball_on {
            from
        } else {
            next
        };
        let r = &mut self.r[k];
        r.last_timing_draw = draw;
        // Trapped (he believes he reaches neither base safely), he turns round only so often
        // between bases: past that he stays committed. A look that finds a base he believes
        // he reaches is not a rundown, and he may take it.
        let heading_on = r.goal as f64 * BASE_L > r.s;
        let turning = (goal > from) != heading_on;
        let trapped = !go_on && !go_back && !crowded;
        if turning && trapped && r.turns >= MAX_TURNS {
            return true;
        }
        if turning {
            r.turns += 1;
        }
        r.goal = goal;
        true
    }

    fn trace_runner(&self, k: usize) {
        use runner_trace::{Cause, Tick};
        let r = &self.r[k];
        if !runner_trace::enabled() || r.start == 0 {
            return;
        }
        let ball_down = self.no_catch_at.is_some();
        let cause = if r.reading_contact {
            Cause::Read
        } else if r.tag_up || r.hold_s.is_some() || r.retouch {
            Cause::Air
        } else if r.last_base > r.start && r.goal == r.last_base && (r.s - r.last_base as f64 * BASE_L).abs() < 1e-6 {
            Cause::BagHold
        } else if r.waiting || r.turned_back.is_some() {
            Cause::Traffic
        } else if r.goal <= r.start {
            if r.decisions == 0 {
                Cause::Read
            } else if ball_down {
                Cause::GroundHold
            } else {
                Cause::OtherHold
            }
        } else {
            Cause::Run
        };
        runner_trace::push(Tick { runner: r.id, t: self.t, s: r.s, signed_mps: r.direction * r.v, goal: r.goal, cause, ball_down });
    }

    fn move_runners(&mut self) {
        for k in 0..self.r.len() {
            // A runner who has touched home is done, unless the catch is still to be seen: his
            // run is only pending until then (day-fixes review #1).
            if self.r[k].out || self.done || (self.r[k].scored && (self.r[k].catch_seen || !self.caught())) {
                continue;
            }
            let start_t = if self.r[k].start == 0 { self.inp.t_contact + BATTER_START } else { self.inp.t_contact };
            if self.t < start_t {
                continue;
            }
            // What the runners behind can see of him once the ball is live: moving forward, or not (§6.4).
            let r = &mut self.r[k];
            if r.direction * r.v > SEEN_MOVING_MPS {
                r.still_since = None;
            } else if r.still_since.is_none() {
                r.still_since = Some(self.t);
            }
            // The rules judge the catch immediately. A runner reacts to the public
            // call after carrying his old movement through the reaction window.
            let at_start = (self.r[k].s - self.r[k].start as f64 * BASE_L).abs() < 1e-6;
            if let Some(call) = self.caught_at
                && !self.r[k].catch_seen
                && (self.t + 1e-9 >= call + RUNNER_CATCH_CALL_REACT_S
                    || (self.r[k].tag_up && at_start && !self.r[k].off_base_at_catch))
            {
                self.r[k].catch_seen = true;
                self.r[k].reading_contact = false;
                let id = self.r[k].id;
                let start = self.r[k].start;
                let retouched = self.out.facts.iter().any(|f| matches!(f, Input::BaseTouched {
                    t, agent, base, holding_ball: false
                } if *t >= call && *agent == id && *base == start));
                if self.r[k].off_base_at_catch {
                    if !retouched {
                        let r = &mut self.r[k];
                        r.scored = false;
                        r.tag_up = false;
                        r.hold_s = None;
                        r.retouch = true;
                        r.waiting = false;
                        r.goal = start;
                    } else if at_start && self.r[k].v == 0.0 {
                        // He returned while the call was pending. The existing touch
                        // is legal and his next leg is now a decision from rest.
                        let r = &mut self.r[k];
                        r.tag_up = false;
                        r.hold_s = None;
                        r.retouch = false;
                        r.turned_back = None;
                        self.runner_decide(k, start);
                    } else {
                        // He touched the bag legally while the call was pending but carried
                        // on past it (catch-read fixes review P2-1). The touch stands; his
                        // legs bring him back to plant, and the plant is where he decides.
                        let r = &mut self.r[k];
                        r.tag_up = false;
                        r.hold_s = None;
                        r.retouch = true;
                        r.waiting = false;
                        r.turned_back = None;
                        r.goal = start;
                    }
                } else if !self.r[k].tag_up && at_start && self.r[k].v == 0.0 {
                    // Planted on his base at the catch without having set up a tag-up: he is
                    // legally on it, and the out and the lifted force are a new play to judge.
                    self.r[k].hold_s = None;
                    self.r[k].waiting = false;
                    self.runner_decide(k, start);
                }
            }
            if self.r[k].scored {
                continue;
            }
            if let Some((at, n)) = self.r[k].fly_look
                && self.t + 1e-9 >= at
            {
                self.fly_relook(k, n);
            }
            if self.r[k].reading_contact {
                self.trace_runner(k);
                let r = &mut self.r[k];
                r.s += r.v * DT;
                continue;
            }
            if self.r[k].overrun != OverrunPhase::None {
                self.step_overrun(k);
                if self.done {
                    return;
                }
                continue;
            }
            // Tag-up: go back to the base and hold it until the catch, then decide.
            if self.r[k].tag_up {
                let on_base = (self.r[k].s - self.r[k].start as f64 * BASE_L).abs() < 1e-6;
                if self.r[k].catch_seen && !on_base {
                    // Off the bag when he sees the call: back to it first; the plant is
                    // where he decides (a touch is only reported where his body is).
                    let r = &mut self.r[k];
                    r.tag_up = false;
                    r.retouch = true;
                    r.goal = r.start;
                } else if self.r[k].catch_seen {
                    // (A runner who still owes the retouch was sent back above.)
                    self.r[k].tag_up = false;
                    let b = self.r[k].last_base;
                    self.emit(Input::BaseTouched { t: self.t, agent: self.r[k].id, base: b, holding_ball: false });
                    // Set on the bag and watching the catch, he leaves on it: an anticipated
                    // start, judged with the same model his legs then run (§6.4).
                    self.r[k].set_start = Some(self.t + self.inp.tag_up.react_s);
                    self.r[k].v = 0.0; // standing on the bag: no earlier speed carries over
                    self.runner_decide(k, b);
                    if self.r[k].goal <= b {
                        self.r[k].set_start = None;
                    }
                } else if self.no_catch_read() {
                    // The ball dropped in: run.
                    self.r[k].tag_up = false;
                    let b = self.r[k].last_base;
                    self.runner_decide(k, b);
                } else if on_base {
                    self.trace_runner(k);
                    continue;
                } else {
                    self.r[k].goal = self.r[k].start;
                }
            }
            // A partway hold ends when he sees the ball come down. Only the contact read sets
            // one in a live play (`start_runners` clears the lead).
            if self.r[k].hold_s.is_some_and(|s| s > self.r[k].start as f64 * BASE_L) && self.no_catch_read() {
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
            // Something new to see since his last look: he decides again, from where he is.
            let awaiting_call = self.awaiting_call(k);
            if !awaiting_call {
                self.runner_relook(k);
            }
            self.runner_yields(k);
            // Heading for a base a runner ahead has stopped at (or is retreating to): go back,
            // unless this is a forced advance (then the runner ahead is forced too).
            let g = self.r[k].goal;
            let forced_here = g == self.r[k].start + 1 && self.runner_forced(k, k);
            // (Never behind his own base: carried past it, that is the base he goes back to.)
            let back = ((self.r[k].s / BASE_L).floor() as u8).max(self.r[k].start);
            if g as f64 * BASE_L > self.r[k].s && self.base_held_ahead(k, g) && !forced_here && !self.return_crowded(k, back) {
                self.r[k].goal = back;
                self.r[k].turned_back = Some(g);
            } else if let Some(g0) = self.r[k].turned_back {
                // He keeps reading the runner ahead on his way back: once that base is
                // visibly no longer held, he goes for it again (Gemini tag-up review P1-2).
                if !self.base_held_ahead(k, g0) {
                    self.r[k].goal = g0;
                    self.r[k].turned_back = None;
                }
            }
            // The last word on where he may stop, whatever set his goal this tick.
            self.settle_guard(k);
            self.trace_runner(k);
            let goal_s = self.r[k].hold_s.unwrap_or(self.r[k].goal as f64 * BASE_L);
            let (now, tag) = (self.t, self.inp.tag_up);
            let r = &mut self.r[k];
            let gap = goal_s - r.s;
            let dir = gap.signum();
            let stopping = r.hold_s.is_some() || r.tag_up || r.retouch || dir < 0.0;
            if gap.abs() < 1e-6 && (!stopping || r.v == 0.0) {
                r.v = 0.0;
                continue;
            }
            match r.set_start {
                // Turning back ends the set start.
                Some(_) if dir < 0.0 => r.set_start = None,
                // Still waiting for his first step after the catch.
                Some(leave) if now < leave => continue,
                _ => {}
            }
            if r.v == 0.0 {
                r.moving_since = now;
            }
            let tau = if r.set_start.is_some() { r.tau * tag.tau_gain } else { r.tau };
            let prev = r.s;
            let (nxt, stop_speed) = if stopping {
                step_to_stop(prev, r.direction * r.v, goal_s, r.vmax, tau)
            } else {
                // For a running arrival, keep his crossing speed. If turning, brake in the
                // old direction until he stops, then accelerate toward the new goal.
                let signed = if r.v > 1e-9 && r.direction != dir {
                    r.direction * (r.v - BRAKE * DT).max(0.0)
                } else {
                    dir * (r.v + (r.vmax - r.v) * (DT / tau))
                };
                (prev + signed * DT, signed)
            };
            let mdir = (nxt - prev).signum();
            if mdir != 0.0 {
                r.direction = mdir;
            }
            r.v = stop_speed.abs();
            let crossed = |b: u8| {
                let bs = b as f64 * BASE_L;
                (prev < bs && nxt >= bs) || (prev > bs && nxt <= bs)
            };
            let mut touched: Option<u8> = None;
            // Bases one to home: coordinate zero is only where the batter's distance is counted
            // from, not a bag he touches on his way out of the box (Codex review P2-1).
            for b in 1..=4u8 {
                if b as f64 * BASE_L != prev && crossed(b) {
                    touched = Some(b);
                }
            }
            // Leaving a base.
            let left = (1..=3u8).find(|&b| nxt != prev && (prev - b as f64 * BASE_L).abs() < 1e-9);
            r.s = if !stopping && mdir == dir && crossed(r.goal) { goal_s } else { nxt };
            let id = r.id;
            if let Some(b) = left {
                self.emit(Input::BaseLeft { t: self.t, agent: id, base: b });
            }
            if let Some(b) = touched {
                let r = &mut self.r[k];
                r.last_base = b;
                if b == 4 {
                    r.scored = true;
                }
                self.emit(Input::BaseTouched { t: self.t, agent: id, base: b, holding_ball: false });
                if self.done {
                    return;
                }
                // Through the bag and on past it in the same tick: he has left it (the rules
                // hold a runner who touched a bag on it until he leaves).
                if b < 4 && (self.r[k].s - b as f64 * BASE_L).abs() > 1e-9 {
                    self.emit(Input::BaseLeft { t: self.t, agent: id, base: b });
                    if self.done {
                        return;
                    }
                }
                if b == self.r[k].goal {
                    self.r[k].turns = 0;
                }
                let retouched = self.r[k].retouch && b == self.r[k].start && self.r[k].v == 0.0;
                if retouched {
                    self.r[k].retouch = false;
                }
                if b < 4 && (!stopping && mdir > 0.0 || retouched) && b == self.r[k].goal && !awaiting_call {
                    // Arrived where it was going (or retouched after a catch): decide whether
                    // to keep going (leaving the base is reported on the next step). Coming
                    // back he braked into the bag, and is planted on it: his next leg starts
                    // from rest, in the estimate and in his legs (§6.4).
                    if mdir < 0.0 {
                        self.r[k].v = 0.0;
                    }
                    self.runner_decide(k, b);
                    let r = &mut self.r[k];
                    // A batter-runner stopping at first runs through it (agent-models §6.2).
                    if r.start == 0 && b == 1 && mdir > 0.0 && r.goal == 1 {
                        r.overrun = OverrunPhase::Out;
                        r.over_m = 0.0;
                    }
                }
            }
            // End the physical stop on arrival, before the next holder reads its
            // speed. Protected overruns retain the crossing speed for braking.
            let r = &mut self.r[k];
            let stop_s = r.hold_s.unwrap_or(r.goal as f64 * BASE_L);
            if r.overrun == OverrunPhase::None && (stop_s - r.s).abs() < 1e-6 && (!stopping || r.v == 0.0) {
                r.v = 0.0;
                r.set_start = None;
                r.turned_back = None;
            }
        }
    }

    /// One tick of the batter-runner's overrun of first: through the bag at his crossing
    /// speed, braking at `OVERRUN_DECEL` down the line, then walking back at
    /// `OVERRUN_RETURN_MPS` until he touches it again. He leaves the bag on the first tick
    /// after his touch, like any runner (rules-spec §5.5 judges his protection from that
    /// `BaseLeft` and the position samples).
    fn step_overrun(&mut self, k: usize) {
        let id = self.r[k].id;
        let r = &mut self.r[k];
        match r.overrun {
            OverrunPhase::None => {}
            OverrunPhase::Out => {
                let leaving = r.over_m == 0.0;
                r.v = (r.v - OVERRUN_DECEL * DT).max(0.0);
                r.over_m += r.v * DT;
                if r.v == 0.0 {
                    r.overrun = OverrunPhase::Back;
                }
                if leaving {
                    self.emit(Input::BaseLeft { t: self.t, agent: id, base: 1 });
                }
            }
            OverrunPhase::Back => {
                r.v += (OVERRUN_RETURN_MPS - r.v) * (DT / r.tau);
                r.over_m -= r.v * DT;
                if r.over_m <= 0.0 {
                    r.over_m = 0.0;
                    r.v = 0.0;
                    r.overrun = OverrunPhase::None;
                    r.last_base = 1;
                    self.emit(Input::BaseTouched { t: self.t, agent: id, base: 1, holding_ball: false });
                }
            }
        }
    }

    fn trace_choice(&self, by: usize, action: &'static str, base: u8, through: bool, selected_option: Option<usize>, options: &[holder::HolderOption]) {
        if !choice_trace::enabled() {
            return;
        }
        choice_trace::push(choice_trace::Choice {
            play: self.inp.play,
            t: self.t,
            until: self.t,
            ticks: 1,
            holder: self.f[by].id,
            possession: self.secured_at,
            from: self.f[by].pos,
            outs: self.st.outs,
            inning: self.st.inning,
            batting: self.st.half.batting(),
            score: self.st.score(),
            action,
            base,
            through,
            runners: self.r.iter().map(|r| choice_trace::RunnerView {
                id: r.id, start: r.start, s: r.s, speed: r.v, goal: visible_target(r, self.f[by].a.throw_choice).unwrap_or((r.s / BASE_L).round() as u8), intended_goal: r.goal,
                out: r.out, scored: r.scored, protected_overrun: r.overrun != OverrunPhase::None,
            }).collect(),
            selected_option,
            options: options.iter().map(|o| o.evidence(self)).collect(),
        });
    }

    /// The fielder holding the ball: throw, run it to a base, tag a runner, or hold.
    fn holder_act(&mut self) {
        let Ball::Held { by } = self.ball else { return };
        // Getting the bag at second with the ball starts a double-play pivot (agent-models
        // §5.5): for a man who took the feed off the bag, and for one who carried it there
        // himself (he plants, turns and throws; no release on the touch tick).
        {
            let t = self.t;
            let force_at_second = self.force_pending_at_second();
            let f = &mut self.f[by];
            // With the force at second gone before he got the bag (taken off), there is no
            // pivot: he is an ordinary receiver, set up from his catch.
            if f.pivot_wait_bag && !force_at_second {
                end_pivot_wait(f);
            }
            let on_bag = (base_pos(2) - f.pos).norm() < 0.6;
            // (A carrier to second caught nothing there: he keeps the set-up he spent holding
            // the ball, agent-models §5.6, and is an ordinary holder on the bag. Gemini pivot
            // review P2-2.)
            if on_bag && f.pivot_wait_bag {
                let moving = (m::hypot(f.vel.x, f.vel.y) > PIVOT_FACE_MIN_MPS).then(|| Vec3::new(f.vel.x, f.vel.y, 0.0));
                f.face = moving.or(f.face);
                f.catch_vel = f.vel;
                f.took_diving = false;
                f.set_base = throwing::pivot_s(f.a.transfer);
                f.up_at = f.up_at.max(t);
                (f.pivot, f.pivot_wait_bag) = (true, false);
            }
        }
        // His call for a man to a bag stands only while he is waiting on that man (below).
        self.f[by].asks = None;
        let fd = self.f[by].clone();
        // Tag any runner within reach who is off a base.
        for k in 0..self.r.len() {
            let r = &self.r[k];
            // A batter-runner coming straight back from his overrun of first is no play: he
            // cannot be tagged out (OBR 5.09(b)(4)), and the fielders see he made no turn.
            if r.out || r.scored || r.overrun != OverrunPhase::None {
                continue;
            }
            // On a bag: first, second or third. Coordinate zero protects nobody; the batter
            // passes it leaving his box (Codex runner-acceleration review P1-1).
            let nearest = (r.s / BASE_L).round();
            let on_base = nearest >= 1.0 && (r.s / BASE_L - nearest).abs() * BASE_L < 0.3;
            // A tag takes him `tag_apply_s` from securing the ball (agent-models §6.4).
            if !on_base && self.t + 1e-9 >= self.secured_at + fd.a.tag_apply_s && (self.runner_pos(r) - fd.pos).norm() < fd.a.throw_choice.tag_reach_m {
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
                let pending = self.r.iter().enumerate().any(|(k, r)| !r.out && !r.scored && self.forced(k) && r.start + 1 == base && r.s < base as f64 * BASE_L - 0.1)
                    || self.r.iter().any(|r| !r.out && r.start == base && r.s > base as f64 * BASE_L + 0.1);
                if pending && !self.out.facts.iter().any(|f| matches!(f, Input::BaseTouched { agent, base: bb, holding_ball: true, .. } if *agent == fd.id && *bb == base)) {
                    self.emit(Input::BaseTouched { t: self.t, agent: fd.id, base, holding_ball: true });
                    if self.done {
                        return;
                    }
                }
            }
        }
        // Carry and throw are alternatives in ONE table. In particular, taking
        // the nearby bag cannot bypass an available lead force or double play.
        let options = self.holder_options(by);
        // Equal values (p_out saturates to 1.0 for a second or more in hand, and with two out
        // every out weighs the same) go to the carry, which has no throw to miss, then to the
        // larger margin.
        let better = |o: &holder::HolderOption, cur: &holder::HolderOption| {
            if (o.value - cur.value).abs() > 1e-9 {
                o.value > cur.value
            } else if o.carry != cur.carry {
                o.carry
            } else {
                o.runner_at - o.ball_at > cur.runner_at - cur.ball_at
            }
        };
        let best = options.iter().enumerate().filter(|(_, o)| o.p_out > 0.3)
            .fold(None, |best: Option<usize>, (i, o)| if best.is_none_or(|j| better(o, &options[j])) { Some(i) } else { best });
        let (b, through, margins, target_runner) = if let Some(i) = best {
            let o = options[i];
            if o.carry {
                // Running with the ball needs no transfer, only his feet (a diver's get-up).
                let ready = if fd.dove { fd.ready_at } else { fd.up_at };
                if self.t >= ready {
                    self.trace_choice(by, "carry", o.base, false, Some(i), &options);
                    self.f[by].target = base_pos(o.base);
                    self.f[by].carry_to = Some(o.base);
                    // Scorer evidence (rules-spec §10.2): he is carrying the ball to put a runner out.
                    if !self.out.scorer_carries.iter().any(|c| c.fielder == fd.id && c.base == o.base) {
                        let batter_goal = self.r.iter().find(|r| r.start == 0).map(|r| r.goal).unwrap_or(0);
                        self.out.scorer_carries.push(ScorerCarryFact { fielder: fd.id, t: self.t, base: o.base, batter_goal, runner: self.r[o.runner].id });
                    }
                }
                return;
            }
            (o.base, o.through, Some((o.runner_at - o.ball_at, o.runner_at - o.set_ball_at)), self.r[o.runner].id)
        } else if m::hypot(fd.pos.x, fd.pos.y) >= INFIELD_R {
            // No play: get the ball back to second base, the quicker way.
            (2, self.throw_race(by, 2).through, None, 0)
        } else {
            self.abandon_carry(by);
            return;
        };
        // A pivot man who took the feed off the bag and now throws rather than carry it to the
        // bag (the runner will beat him there, or the play is elsewhere) is an ordinary
        // receiver, not locked out until he has the bag (Gemini pivot review P2-1).
        if fd.pivot_wait_bag {
            end_pivot_wait(&mut self.f[by]);
        }
        // He plants for the throw: a carry he gave up does not keep him running at its bag
        // while his set-up clock runs (§5.6 A4).
        self.abandon_carry(by);
        if self.t < fd.up_at {
            return;
        }
        let race = self.throw_race(by, b);
        let tgt = base_pos(b);
        let leg = match race.relay {
            Some(i) if through => self.f[i].pos,
            _ => tgt,
        };
        // Get set, or hurry? (agent-models §5.5) With no play on, and with time in hand, he
        // gets set. Short of time he lets go as soon as waiting longer would cost him his
        // margin, and as soon as he can if he has none; but only if that gets the ball
        // there sooner (not when it would wait for the man covering the bag anyway).
        let su = self.set_up_for(by, leg);
        let since = self.t - fd.up_at;
        if since < su.earliest_s {
            return;
        }
        if since < su.set_s {
            let hurry = match margins {
                Some((now, set)) => set < SAFE_MARGIN_S && now <= SAFE_MARGIN_S && now > set + HURRY_GAIN_MIN_S,
                None => false,
            };
            if !hurry {
                return;
            }
        }
        let action = if margins.is_some() { "throw" } else { "return" };
        if let (true, Some(i)) = (through, race.relay) {
            self.trace_choice(by, action, b, true, best, &options);
            // Through a cutoff man: full effort (the relay is a race by construction).
            self.throw_to(by, leg, b, i, true, 1.0);
            if let Some(fact) = self.out.scorer_throws.last_mut() { fact.attempted_out = margins.is_some(); fact.runner = if margins.is_some() { target_runner } else { 0 }; }
            return;
        }
        let receiver = race.receiver;
        // His man is not at the bag yet: he waits for him and calls him to it ("two!",
        // pointing). The call is his to make, but it reaches the man only when heard
        // (`heed_requests`, `CALL_HEARD_S`); his private choice moves nobody (Codex
        // fielder-communication fourth review P1-1).
        if race.direct < race.receiver_at - 0.05 {
            self.f[by].asks = Some((b, receiver));
            return;
        }
        self.trace_choice(by, action, b, false, best, &options);
        // As hard as the play needs and no harder (agent-models §5.5). With no out to try
        // for, a runner he can see moving could still take a base on the throw: that is the
        // play, full effort; with every runner standing, the floor. For an out, the least
        // effort that still keeps the safe margin by his own read of the race, else everything.
        let effort = match best.filter(|_| margins.is_some()).map(|i| options[i]) {
            None if self.r.iter().any(|r| !r.out && !r.scored && (r.overrun != OverrunPhase::None || r.v > SEEN_MOVING_MPS)) => 1.0,
            None => throwing::EFFORT_MIN,
            // A force with a second out possible behind it (a force continuation exists): the
            // feed is a race for that one, however unlikely he reads it, since easing the
            // throw would only make it less likely.
            Some(o) if o.continuation.is_some() => 1.0,
            Some(o) => {
                let v = self.throw_speed(by, tgt) * su.speed_mult(su.hurry(since));
                let d = (tgt - fd.pos).norm();
                let tag = if o.force { 0.0 } else { self.f[receiver].a.tag_apply_s };
                let cut = self.cut_seen(Some(by), fd.pos, b);
                let ball_at = |e: f64| (self.t + direct_time(e * v, d, cut)).max(race.receiver_at) + 0.05 + tag;
                (0..=20).map(|i| throwing::EFFORT_MIN + (1.0 - throwing::EFFORT_MIN) * i as f64 / 20.0).find(|&e| o.runner_at - ball_at(e) >= SAFE_MARGIN_S).unwrap_or(1.0)
            }
        };
        self.throw_to(by, tgt, b, receiver, false, effort);
        if let Some(fact) = self.out.scorer_throws.last_mut() { fact.attempted_out = margins.is_some(); fact.runner = if margins.is_some() { target_runner } else { 0 }; }
        self.f[receiver].target = tgt;
        // A throw that goes past the cutoff man ends his job: he pulls up, free to back it up.
        for i in 0..9 {
            if matches!(self.f[i].role, Role::Relay(_)) {
                go_idle(&mut self.f[i]);
            }
        }
        #[cfg(test)]
        iso_check(self, "back_up_throw");
        self.back_up_throw(fd.pos, b);
    }

    /// A man the holder has called to a bag for his throw goes to it once he hears the call
    /// (`CALL_HEARD_S` after it was made, agent-models §5.7), while that man still has the
    /// ball and still calls for him. Until then he does his own job; when the call is
    /// withdrawn he keeps the bag until his job is re-worked. Reads only what was shown.
    fn heed_requests(&mut self) {
        let Ball::Held { by } = self.ball else { return };
        let Some(e) = self.shown_then(by) else { return };
        // A call he made with the ball before he last secured it is not this one.
        if e.at + 1e-9 < self.ball_since.max(self.secured_at) {
            return;
        }
        if let Some((b, r)) = e.asks {
            if r != by {
                self.f[r].target = base_pos(b);
            }
        }
    }

    /// A holder who gives up a carry pulls up where braking stops him (agent-models §5.6 A4);
    /// any other target (a receiver's step to his bag, the catch's own stop) is left alone.
    fn abandon_carry(&mut self, by: usize) {
        if self.f[by].carry_to.take().is_some() {
            self.f[by].target = stop_point(&self.f[by]);
        }
    }

    /// Fielder `by`'s set-up for a throw to ground point `tgt` with the ball he holds: the
    /// turn from the way he faced taking it, and the footwork he owes if he took it moving
    /// away or diving (agent-models §5.5).
    fn set_up_for(&self, by: usize, tgt: Vec3) -> SetUp {
        let fd = &self.f[by];
        let to = tgt - fd.pos;
        let turn = fd.face.map(|f| throwing::turn_extra_s(f, to, fd.a.throws)).unwrap_or(0.0);
        let footwork = throwing::footwork_s(fd.catch_vel, to, fd.took_diving);
        if pivot_throw(fd, tgt) {
            SetUp::pivot(fd.set_base, turn, footwork)
        } else if fd.relayed {
            // The relay man's exchange: turn included, no early release (agent-models §5.5).
            SetUp::relay(fd.set_base, footwork)
        } else if fd.pivot {
            // At the bag with the feed, throwing elsewhere: an ordinary throw from his catch.
            SetUp::new(RELAY_TRANSFER_FRAC * fd.a.transfer, turn, footwork)
        } else {
            SetUp::new(fd.set_base, turn, footwork)
        }
    }

    /// Fielder `by`'s full-effort release speed for a throw to ground point `tgt`: his arm, and
    /// what coming through the catch toward it adds (agent-models §5.5, the catch sets up the
    /// throw; none after taking it on the ground from a dive). His own body, known to him.
    fn throw_speed(&self, by: usize, tgt: Vec3) -> f64 {
        let fd = &self.f[by];
        let gain = if fd.took_diving { 0.0 } else { throwing::momentum_gain_mps(fd.catch_vel, tgt - fd.pos) };
        THROW_EFFORT * fd.a.arm + gain
    }

    /// The cutoff man lined up between fielder `by` and base `b`, as `by` knows him: how far
    /// out along the throw he stands, m (agent-models §5.5: the throw must pass him low).
    fn cutoff_out(&self, by: usize, b: u8) -> Option<f64> {
        self.man_in_throw(Some(by), self.f[by].pos, b)
    }

    /// The man with the cutoff or relay job for base `b` standing in a throw from `from` to it
    /// (between, more than 10 m out, within `CUT_LINE_M` of the line), as `observer` knows the
    /// jobs (`seen`; `None`: what everyone can see, `known_job`): how far out along the throw
    /// he stands, m.
    fn man_in_throw(&self, observer: Option<usize>, from: Vec3, b: u8) -> Option<f64> {
        let base = base_pos(b);
        let line = Vec3::new(base.x - from.x, base.y - from.y, 0.0);
        let len = line.norm();
        if len < 1e-9 {
            return None;
        }
        (0..9)
            .filter(|&i| {
                let job = match observer {
                    Some(o) => self.seen(o, i).0,
                    None => self.known_job(i).0,
                };
                observer != Some(i) && job == Role::Relay(b)
            })
            .map(|i| self.f[i].pos)
            .filter(|p| (*p - from).norm() > 10.0 && (base - *p).norm() < len && ((p.x - from.x) * line.y - (p.y - from.y) * line.x).abs() / len < CUT_LINE_M)
            .map(|p| ((p.x - from.x) * line.x + (p.y - from.y) * line.y) / len)
            .next()
    }

    /// Whether a throw from `from` to base `b` would go through a man lined up in it, and which
    /// kind, as `observer` sees it (`None`: a runner, from the jobs the fielders show and where
    /// they stand; agent-models §5.5, §6.4).
    fn cut_seen(&self, observer: Option<usize>, from: Vec3, b: u8) -> Cut {
        let len = m::hypot(base_pos(b).x - from.x, base_pos(b).y - from.y);
        self.man_in_throw(observer, from, b).map_or(Cut::No, |out| Cut::of(len - out, len))
    }

    /// Fielder `by` throws to ground point `tgt` for `receiver` (the ball is headed for base
    /// `b`, through a relay man when `relay`). Let go before he is set, the throw is slower
    /// and wilder (agent-models §5.5).
    fn throw_to(&mut self, by: usize, tgt: Vec3, b: u8, receiver: usize, relay: bool, effort: f64) {
        let fd = self.f[by].clone();
        let mut r = rng(self.inp.seed, self.inp.play, fd.id, Purpose::ThrowExecution, fd.throws);
        self.f[by].throws += 1;
        let su = self.set_up_for(by, tgt);
        let hurry = su.hurry(self.t - fd.up_at);
        // Aim with the thrower's own throw physics (drag and backspin, agent-models §5.5);
        // a throw beyond his reach on the fly goes at the steepest useful angle and bounces.
        let r0 = throw_release_point(fd.pos, tgt, fd.a.throws);
        let v_aim = effort * self.throw_speed(by, tgt) * su.speed_mult(hurry);
        let d = m::hypot(tgt.x - r0.x, tgt.y - r0.y);
        // Straight to a base with a cutoff man in the throw: low enough for him to cut it
        // (agent-models §5.5); a throw to the cutoff man himself is aimed at him.
        let cut = if relay { None } else { self.cutoff_out(by, b) };
        let elev = match cut {
            Some(out) => cut_elev(self.inp.env, THROW_RELEASE_Z, THROW_CATCH_Z, CUT_HEIGHT_M, d, out, v_aim),
            None => solve_throw(self.inp.env, THROW_RELEASE_Z, THROW_CATCH_Z, d, v_aim).map(|s| s.elev),
        }
        .unwrap_or(THROW_MAX_ELEV);
        // Execution noise: speed 3%, then direction (the draw order of the vacuum model).
        // A pivot man's throw: the slide's seam (nothing until A6 builds slides).
        let slide = if pivot_throw(&fd, tgt) { throwing::PIVOT_SLIDE_SIGMA_MULT } else { 1.0 };
        let ex = execute_throw(v_aim, &ThrowNoise::new(fd.a.arm_sigma * su.sigma_mult(hurry) * slide * throwing::effort_sigma_mult(effort)), &mut r);
        let h = Vec3::new(tgt.x - r0.x, tgt.y - r0.y, 0.0).normalized().rotate(Vec3::Z, ex.yaw_err);
        self.launch_throw(throw_state(r0, h, ex.speed, elev), tgt, b, receiver, relay);
        self.out.released.push((self.t, fd.id));
        let batter_goal = self.r.iter().find(|r| r.start == 0).map(|r| r.goal).unwrap_or(0);
        self.out.scorer_throws.push(ScorerThrowFact { thrower: fd.id, t: self.t, origin: r0, target: tgt, base: b, attempted_out: false, batter_goal, runner: 0 });
        // The rules engine needs the release for a throw that goes out of play (rules-spec §7.2);
        // fielders 0–5 are P, C, 1B, 2B, 3B, SS.
        let t_rel = self.t.max(self.last_fact_t);
        self.emit(Input::BallReleasedFromGlove { t: t_rel, fielder: fd.id, kind: ReleaseKind::Throw, infielder: by < 6 });
        self.out.hurry.push((hurry, su.off_balance));
        self.out.effort.push(effort);
    }

    /// Who takes a throw at base `b`: its cover man, else the nearest other fielder who is
    /// not the cutoff man (he keeps to his line).
    fn receiver_for(&self, by: usize, b: u8) -> usize {
        let tgt = base_pos(b);
        // Two men can be on their way to one bag for a moment (a chase not yet known, the
        // pitcher breaking for first): he throws to the one nearer it.
        let bag = base_pos(b);
        // He knows a teammate's cover or cutoff job as it is known (`seen`).
        (0..9).filter(|&i| i != by && self.seen(by, i).0 == Role::Cover(b)).min_by(|&a, &c| (self.f[a].pos - bag).norm().total_cmp(&(self.f[c].pos - bag).norm()).then(a.cmp(&c))).unwrap_or_else(|| {
            (0..9)
                .filter(|&i| i != by && !matches!(self.seen(by, i).0, Role::Relay(_)))
                .min_by(|&a, &c| (self.f[a].pos - tgt).norm().total_cmp(&(self.f[c].pos - tgt).norm()).then(a.cmp(&c)))
                .unwrap()
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
        let calm = self.r.iter().all(|r| {
            r.out || ((!self.caught() || r.catch_seen) && (r.scored
                || ((r.s - r.goal as f64 * BASE_L).abs() < 1e-6
                    && r.overrun == OverrunPhase::None && !r.tag_up && !r.reading_contact
                    && r.hold_s.is_none())))
        });
        let in_infield = m::hypot(fd.pos.x, fd.pos.y) < INFIELD_R;
        calm && in_infield && self.t >= fd.ready_at
    }

    /// Where a runner is. Runners move along the base path (`path_pos`), except that the
    /// batter starts in his box, off the path, and runs straight to the point `BOX_EXIT_M`
    /// up the first-base line. His path coordinate starts at [`batter_start_s`], so the
    /// distance he is charged is the length of the route he runs. Tags, his own view of the ball, and the recorded track all use
    /// this one position. On his overrun of first he is past the bag on the first-base line.
    fn runner_pos(&self, r: &Runner) -> Vec3 {
        if r.overrun != OverrunPhase::None {
            // Past first, straight down the first-base line.
            return base_pos(1) * (1.0 + r.over_m / BASE_L);
        }
        if r.start == 0 && r.s < BOX_EXIT_M {
            let b = self.inp.batter_box;
            let (b, s0) = (Vec3::new(b.x, b.y, 0.0), batter_start_s(b));
            b + (path_pos(BOX_EXIT_M) - b) * ((r.s - s0) / (BOX_EXIT_M - s0)).clamp(0.0, 1.0)
        } else {
            runner_path_pos(r.start, r.s)
        }
    }

    /// Give the runners their contact state and, when recording, the approach from release.
    fn start_runners(&mut self) {
        let lead = if self.st.outs >= 2 {
            self.inp.runner_start.two_out
        } else {
            self.inp.runner_start.fewer_than_two
        };
        let contact = self.inp.t_contact;
        for (k, r) in self.r.iter_mut().enumerate() {
            r.goal = r.start + 1;
            if r.start == 0 {
                continue;
            }
            let primary = lead_s(r.start);
            r.s = primary + lead.extra_m;
            r.v = lead.speed_mps.min(r.vmax);
            r.hold_s = None;
            r.reading_contact = true;
            if self.inp.record && contact > 0.0 {
                let steps = (contact / DT).ceil() as usize;
                let track = &mut self.out.actors[9 + k].1;
                for i in 0..steps {
                    let u = i as f64 / steps as f64;
                    // A constant-acceleration approach joins the release position to
                    // the contact position and speed. A long flight with a fast finish
                    // uses a rising speed curve so the runner never backs up at release.
                    let end_slope = contact * r.v;
                    let distance = if end_slope <= 2.0 * lead.extra_m {
                        u * (lead.extra_m + (1.0 - u) * (lead.extra_m - end_slope))
                    } else {
                        lead.extra_m * m::pow(u, end_slope / lead.extra_m)
                    };
                    track.push((u * contact, runner_path_pos(r.start, primary + distance)));
                }
            }
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
/// The `BallPassedBase` fact (rules-spec §2, §5.2) for a batted ball that has touched the
/// ground and, between `t0` and `t1`, passes first or third base: its center crosses the line
/// through the bag square to that foul line, on the bag's side of the field. Fair or foul is
/// judged where it crosses: by the contact point if it is rolling, by any part of the ball if
/// it is in the air on a bound (OBR "Fair Ball", "Foul Ball": "bounds past first or third base
/// on or over fair/foul territory"). Before this the sim judged such a ball only where it came
/// to rest, so a roller past third in foul ground stayed live until it stopped. A crossing
/// made outside the playing field does not count: a bouncer that went into a dugout or over a
/// foul-ground wall short of the bag is out of play where it left, and its path in the stands
/// may still cross the bag's line before it ends (agy review 2026-11-16 P2-3).
fn passed_base(path: &BallPath, field: &Field, t0: f64, t1: f64, r_ball: f64) -> Option<Input> {
    let Some(&(t_bounce, _, _)) = path.bounces.first() else { return None };
    let t0 = t0.max(t_bounce);
    if t1 <= t0 {
        return None;
    }
    // Distance past each bag along its own foul line, counted only on that bag's side.
    let past = |t: f64, side: f64| {
        let p = path.at(t).0;
        if p.x * side < 0.0 {
            return f64::NEG_INFINITY;
        }
        (p.y + side * p.x) / core::f64::consts::SQRT_2 - BASE_L
    };
    for side in [1.0, -1.0] {
        if past(t0, side) >= 0.0 || past(t1, side) < 0.0 {
            continue;
        }
        // Bisect for the crossing (within a tick the ball's speed barely changes).
        let (mut a, mut b) = (t0, t1);
        for _ in 0..40 {
            let m = 0.5 * (a + b);
            if past(m, side) >= 0.0 {
                b = m;
            } else {
                a = m;
            }
        }
        let p = path.at(b).0;
        if field.ball_clearance(p.x, p.y, r_ball).dist < 0.0 {
            return None;
        }
        let line = (p.y - p.x.abs()) / core::f64::consts::SQRT_2;
        let airborne = p.z > r_ball + 0.01;
        let fair = if airborne { line >= -r_ball } else { line >= 0.0 };
        return Some(Input::BallPassedBase { t: b, fair });
    }
    None
}

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
    let home = home_positions_for(inp.bats, st.bases[0].is_some(), st.bases[1].is_some(), st.outs);
    let f: Vec<Fielder> = (0..9)
        .map(|i| {
            let mut r = rng(seed, play, inp.fielders[i], Purpose::FielderPerception, 0);
            Fielder {
                id: inp.fielders[i],
                a: inp.fattrs[i],
                pos: home[i],
                vel: Vec3::ZERO,
                jump: jump_at_contact(inp, i, home[i]),
                role: Role::Idle,
                target: home[i],
                next_plan: 0.0,
                err: [r.truncated_normal(4.0), r.truncated_normal(4.0), r.truncated_normal(4.0), r.truncated_normal(4.0)],
                track: None,
                doubt: 0.0,
                doubt_at: f64::NEG_INFINITY,
                attempts: 0,
                blocked_until: 0.0,
                ready_at: 0.0,
                dove: false,
                lunge: None,
                belief: None,
                t_int: f64::INFINITY,
                int_pt: home[i],
                read_at: f64::NEG_INFINITY,
                read_ball: u64::MAX,
                call: 0,
                shown: Vec::new(),
                last_close_read: 0.0,
                must_dive_since: f64::INFINITY,
                last_dive_read: f64::NEG_INFINITY,
                will_dive: std::cell::Cell::new(false),
                throws: 0,
                up_at: 0.0,
                set_base: 0.0,
                face: None,
                catch_vel: Vec3::ZERO,
                took_diving: false,
                catch_stretched: false,
                catch_heading: None,
                catch_seen_vel: Vec3::ZERO,
                catch_received_throw: false,
                catch_shown_relay: false,
                backs: None,
                peel: None,
                carry_to: None,
                asks: None,
                pivot: false,
                pivot_wait_bag: false,
                relayed: false,
            }
        })
        .collect();
    let r: Vec<Runner> = inp
        .runners
        .iter()
        .map(|x| Runner {
            id: x.id,
            start: x.base,
            s: if x.base == 0 { batter_start_s(inp.batter_box) } else { lead_s(x.base) },
            v: 0.0,
            direction: 1.0,
            vmax: x.speed,
            tau: x.tau_s,
            goal: x.base,
            out: false,
            scored: false,
            tag_up: false,
            set_start: None,
            still_since: None,
            turned_back: None,
            moving_since: 0.0,
            last_base: x.base,
            retouch: false,
            waiting: false,
            reading_contact: false,
            hold_s: if x.base == 0 { None } else { Some(lead_s(x.base)) },
            last_timing_draw: 0.0,
            catch_seen: false,
            off_base_at_catch: false,
            decisions: 0,
            looked: (0, 0),
            turns: 0,
            fly_look: None,
            overrun: OverrunPhase::None,
            over_m: 0.0,
        })
        .collect();
    let mut actors: Vec<(AgentId, Vec<(f64, Vec3)>)> = inp.fielders.iter().map(|&id| (id, Vec::new())).collect();
    actors.extend(inp.runners.iter().map(|x| (x.id, Vec::new())));
    Sim {
        inp,
        st,
        t: t_c,
        ball: Ball::Free { path: path.clone(), next_touch: 0, next_wall: 0, settled_sent: false },
        home,
        f,
        r,
        ball_since: t_c,
        ball_gen: 0,
        rank_cache: Default::default(),
        last_known: vec![(false, 0, false); 9],
        last_jobs: vec![(Role::Idle, None); 9],
        out: LiveOutput { scorer_carries: Vec::new(), first_handling: None, scorer_throws: Vec::new(), batter_first_eta: None, ref_set_s: None, facts: Vec::new(), events: Vec::new(), path, ball_track: Vec::new(), actors, secured: Vec::new(), released: Vec::new(), hurry: Vec::new(), effort: Vec::new(), dives: Vec::new(), calls: Vec::new() },
        bounce_k: 1000,
        done: false,
        last_fact_t: t_c,
        secured_at: f64::INFINITY,
        caught_at: None,
        no_catch_at: None,
        flight_secured_at: None,
        low_catch: false,
    }
}

// Information-isolation harness (tests only; Codex fielder-communication third review
// P1-1): a hook called before each coordination step of a live play, with the simulation as
// it stands. `tests::isolation_*` forks it, changes one man's private state, runs the step
// on both forks and compares everyone else.
#[cfg(test)]
type IsoHook = Box<dyn FnMut(&Sim<'_>, &'static str)>;
#[cfg(test)]
thread_local! { static ISO_HOOK: std::cell::RefCell<Option<IsoHook>> = const { std::cell::RefCell::new(None) }; }
// The harness's negative control (`isolation_harness_finds_the_third_reviews_gate`): with
// this set, `update_relay` behaves as on `f3d9e7a`, re-working every teammate's cover when one
// man privately leaves or takes the cutoff job (Codex third review P1-1).
#[cfg(test)]
thread_local! { static ISO_REVERT_GATE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
#[cfg(test)]
fn iso_check(sim: &Sim<'_>, step: &'static str) {
    // The hook is taken out while it runs: steps it runs on its forks reach their own hooks
    // (a release's backup inside `holder_act`), which must do nothing then.
    let hook = ISO_HOOK.with(|h| h.borrow_mut().take());
    if let Some(mut f) = hook {
        f(sim, step);
        ISO_HOOK.with(|h| *h.borrow_mut() = Some(f));
    }
}

#[cfg(test)]
impl<'a> Sim<'a> {
    /// A copy of the simulation with its own rules state, for the isolation harness.
    fn fork<'b>(&'b self, st: &'b mut RulesState) -> Sim<'b> {
        Sim {
            inp: self.inp,
            last_fact_t: self.last_fact_t,
            secured_at: self.secured_at,
            caught_at: self.caught_at,
            no_catch_at: self.no_catch_at,
            flight_secured_at: self.flight_secured_at,
            low_catch: self.low_catch,
            st,
            t: self.t,
            ball: self.ball.clone(),
            home: self.home,
            f: self.f.clone(),
            r: self.r.clone(),
            ball_since: self.ball_since,
            ball_gen: self.ball_gen,
            rank_cache: Default::default(),
            last_known: self.last_known.clone(),
            last_jobs: self.last_jobs.clone(),
            // The recorded tracks are written only by `record`, never read by a decision:
            // left out to keep forks cheap.
            out: {
                let o = &self.out;
                LiveOutput {
                    scorer_carries: o.scorer_carries.clone(),
                    first_handling: o.first_handling,
                    batter_first_eta: o.batter_first_eta,
                    ref_set_s: o.ref_set_s,
                    scorer_throws: o.scorer_throws.clone(),
                    facts: o.facts.clone(),
                    events: o.events.clone(),
                    path: o.path.clone(),
                    ball_track: Vec::new(),
                    actors: Vec::new(),
                    secured: o.secured.clone(),
                    released: o.released.clone(),
                    hurry: o.hurry.clone(),
                    effort: o.effort.clone(),
                    dives: o.dives.clone(),
                    calls: o.calls.clone(),
                }
            },
            bounce_k: self.bounce_k,
            done: self.done,
        }
    }

    /// Run the named coordination step (as `simulate_live_inner` does).
    fn run_step(&mut self, step: &str) {
        match step {
            "assign" | "new_ball_assign" => self.assign(),
            "replan_chasers" => self.replan_chasers(),
            "close_reads" => self.close_reads(),
            "dive_reads" => self.dive_reads(),
            "receiver_reads" => self.receiver_reads(),
            "back_up_throw" => {
                // Just after a release in `holder_act`: the thrower and the base of his throw.
                let Ball::Thrown { to_base, .. } = self.ball else { unreachable!("back_up_throw without a throw") };
                let id = self.out.released.last().expect("a release").1;
                let by = (0..9).find(|&i| self.f[i].id == id).expect("the thrower");
                self.back_up_throw(self.f[by].pos, to_base)
            }
            "recall" => self.recall(),
            "expire_peels" => self.expire_peels(),
            "chasers_see_the_ball_secured" => self.chasers_see_the_ball_secured(),
            "holder_act" => self.holder_act(),
            "heed_requests" => self.heed_requests(),
            "cover_home_again" => self.cover_home_again(),
            "hear" => self.hear(),
            "update_relay" => self.update_relay(),
            _ => unreachable!("{step}"),
        }
    }
}

fn simulate_live_inner(inp: &LiveInput, st: &mut RulesState) -> LiveOutput {
    let t_c = inp.t_contact;
    let mut sim = new_sim(inp, st);
    sim.start_runners();
    // Runners are off their bases with their leads as the ball is hit: the rules hear of it.
    for k in 0..sim.r.len() {
        if sim.r[k].start > 0 {
            let (id, base) = (sim.r[k].id, sim.r[k].start);
            sim.emit(Input::BaseLeft { t: t_c, agent: id, base });
        }
    }
    // Fielders and runners have separate reaction clocks.
    let react = inp.fattrs.iter().map(|a| a.reaction).fold(f64::INFINITY, f64::min);
    let mut assigned = false;
    let mut runners_started = false;
    let two_out = sim.st.outs >= 2;
    let read_s = if two_out { RUNNER_CONTACT_REACT_S } else { RUNNER_READ_DELAY_S };
    let mut next_recall = t_c + 1.0;
    let mut tick: u64 = 0;
    // Coverage trace (`BB_COVER_TRACE`; `examples/coverage_trace.rs`, `analysis/coverage_gaps.py`):
    // one line per change of the defense's jobs. Reads only; changes no decision.
    let cover_trace = std::env::var("BB_COVER_TRACE").is_ok();
    let mut cover_last = String::new();
    while !sim.done {
        if sim.t - t_c > PLAY_LIMIT_S {
            let t = sim.t;
            sim.emit(Input::PlayTimeout { t });
            break;
        }
        // Every fielder who has reacted keeps looking at a ball on its first flight
        // (agent-models §5.1, the tracking read).
        if matches!(sim.ball, Ball::Free { .. }) {
            sim.take_looks();
        }
        if !assigned && sim.t >= t_c + react {
            #[cfg(test)]
            iso_check(&sim, "assign");
            sim.assign();
            assigned = true;
        }
        if assigned && matches!(sim.ball, Ball::Free { .. }) {
            #[cfg(test)]
            iso_check(&sim, "replan_chasers");
            sim.replan_chasers();
            let (b, _) = sim.ball_at(sim.t);
            // Once a second, while the man known to be going for the ball is still far from it
            // (or nobody is known to be), the men near it read it again and decide again
            // (calls, agent-models §5.7).
            let far = sim.lead().is_none_or(|l| (Vec3::new(b.x, b.y, 0.0) - sim.f[l].pos).norm() > 8.0);
            if sim.t >= next_recall && far {
                next_recall = sim.t + 1.0;
                #[cfg(test)]
                iso_check(&sim, "recall");
                sim.recall();
            }
        }
        sim.ball_facts();
        if sim.done {
            break;
        }
        // A runner on a grounder that came down near his first look still has
        // to react to the public ruling. Two outs leaves only the contact clock.
        if !runners_started && !sim.caught() && sim.t - t_c >= read_s - 1e-9
            && (two_out || sim.no_catch_at.is_none() || sim.no_catch_read())
        {
            runners_started = true;
            let check_catch = !two_out && sim.no_catch_at.is_none() && matches!(sim.ball, Ball::Free { .. });
            for k in 0..sim.r.len() {
                sim.r[k].reading_contact = false;
                if sim.r[k].start == 0 || two_out {
                    continue;
                }
                if check_catch {
                    let look = sim.observe_contact(k);
                    sim.read_contact(k, look);
                } else {
                    sim.runner_decide(k, sim.r[k].start);
                }
            }
        }
        if assigned {
            #[cfg(test)]
            iso_check(&sim, "expire_peels");
            sim.expire_peels();
            #[cfg(test)]
            iso_check(&sim, "chasers_see_the_ball_secured");
            sim.chasers_see_the_ball_secured();
        }
        sim.update_thrown();
        sim.try_captures();
        if sim.done {
            break;
        }
        #[cfg(test)]
        iso_check(&sim, "holder_act");
        sim.holder_act();
        if sim.done {
            break;
        }
        #[cfg(test)]
        iso_check(&sim, "heed_requests");
        sim.heed_requests();
        #[cfg(test)]
        iso_check(&sim, "cover_home_again");
        sim.cover_home_again();
        if assigned {
            #[cfg(test)]
            iso_check(&sim, "hear");
            sim.hear();
        }
        #[cfg(test)]
        iso_check(&sim, "update_relay");
        sim.update_relay();
        sim.move_fielders();
        if tick == 0 {
            // The contact position is a real track sample. The first elapsed tick
            // of runner movement comes after it.
            for k in 0..sim.r.len() {
                sim.trace_runner(k);
            }
        } else {
            sim.move_runners();
        }
        if sim.done {
            break;
        }
        if tick % SAMPLE_EVERY_TICKS == 0 {
            sim.sample_runners();
            if sim.done {
                break;
            }
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
        if assigned {
            sim.publish();
        }
        if read_trace::on() && sim.off_bat().is_some() {
            let recs: Vec<read_trace::Rec> = sim.f.iter().enumerate().map(|(i, fd)| read_trace::Rec {
                t: sim.t,
                i,
                role: match fd.role { Role::Idle => 0, Role::Chase => 1, Role::Cover(_) => 2, Role::Relay(_) => 3, Role::Backup => 4 },
                pos: fd.pos,
                vel: fd.vel,
                land: fd.belief.as_ref().and_then(|p| p.first_bounce()),
                read_at: fd.read_at,
                target: fd.target,
                call: fd.call,
                peel: fd.peel.is_some(),
                must_dive_since: fd.must_dive_since,
                dove: fd.dove,
            }).collect();
            read_trace::RECS.with(|r| r.borrow_mut().extend(recs));
        }
        if cover_trace && assigned {
            let (phase, holder) = match sim.ball {
                Ball::Free { .. } => ("F", -1),
                Ball::Held { by } => ("H", by as i32),
                Ball::Thrown { .. } => ("T", -1),
            };
            let grounder = sim.out.path.first_bounce().is_some_and(|b| m::hypot(b.1, b.2) < 25.0 && b.0 - inp.t_contact < GROUNDER_BOUNCE_S);
            let counts: Vec<usize> = (1..=4).map(|b| sim.f.iter().filter(|f| f.role == Role::Cover(b)).count()).collect();
            let chasing: u16 = (0..9).filter(|&i| sim.f[i].role == Role::Chase).map(|i| 1u16 << i).sum();
            let roles: Vec<i32> = sim.f.iter().map(|f| match f.role { Role::Idle => 0, Role::Chase => 1, Role::Cover(b) => 10 + b as i32, Role::Relay(b) => 20 + b as i32, Role::Backup => 30 }).collect();
            let key = format!("{:.6},{},{},{},{},{},{},{},{},{},{:?}", sim.ball_since, phase, holder, sim.lead().map(|i| i as i32).unwrap_or(-1), grounder as u8, counts[0], counts[1], counts[2], counts[3], chasing, roles);
            if key != cover_last {
                eprintln!("R,{},{},{:.6},{}", inp.seed, inp.play, sim.t, key);
                cover_last = key;
            }
        }
        prof::TICKS.with(|c| c.set(c.get() + 1));
        tick += 1;
        sim.t += DT;
    }
    if cover_trace {
        eprintln!("E,{},{},{:.6}", inp.seed, inp.play, sim.t);
    }
    if inp.record {
        // The tick that ended the play (the catch, the force, the ball leaving the park).
        sim.record(true);
    }
    // The batter-runner, if the play ended before he touched first: when he would have.
    if let Some(r) = sim.r.iter().find(|r| r.start == 0) {
        let touched = sim.out.facts.iter().any(|f| matches!(f, Input::BaseTouched { agent, base: 1, holding_ball: false, .. } if *agent == r.id));
        if !touched && !r.out && r.s < BASE_L {
            let along = r.v.max(0.0) * r.direction.max(0.0);
            sim.out.batter_first_eta = Some(sim.t + run_time_from(BASE_L - r.s, along, r.vmax, r.tau));
        }
    }
    sim.out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::home_positions;
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

    // Owner's play, seed 3647 play 15: a bouncer that rolled past third in foul ground stayed
    // live until it stopped 40 m on, 5.3 s after contact, and only then was called foul. The
    // rules call it as it passes the bag (rules-spec §5.2, OBR "Foul Ball"). Three bouncers off
    // the plate along the third-base side, judged by the line they pass the bag on: foul,
    // fair, and one that never comes near the bag. Expectations from the geometry: the bag is
    // 27.432 m down the line, and a ball on the ray at angle θ from the line is foul when θ
    // points outside it.
    #[test]
    fn a_ball_rolling_past_third_is_judged_as_it_passes_the_bag() {
        let fx = fixture(&[]);
        let judge = |dir_deg: f64, speed: f64| {
            // Heading measured from +Y toward −X (third-base side); the line is at 45°.
            let h = dir_deg.to_radians();
            let d = Vec3::new(-h.sin(), h.cos(), 0.0);
            let ball = BallState { r: Vec3::new(0.0, 0.6, 0.6), v: d * speed + Vec3::new(0.0, 0.0, -4.0), w: Vec3::ZERO };
            let path = batted_path(&fx.env, &fx.field, ball, 0.4, |_, _| (0.0, 0.0));
            let mut t = 0.4;
            while t < path.t_end() {
                if let Some(f) = passed_base(&path, &fx.field, t, (t + DT).min(path.t_end()), fx.env.ball.radius) {
                    let Input::BallPassedBase { t: tp, fair } = f else { unreachable!() };
                    let p = path.at(tp).0;
                    let along = (p.y - p.x) / core::f64::consts::SQRT_2;
                    assert!((along - BASE_L).abs() < 1e-6, "crossing {along} m down the line, not at the bag");
                    return Some(fair);
                }
                t += DT;
            }
            None
        };
        assert_eq!(judge(48.0, 30.0), Some(false), "3° outside the third-base line");
        assert_eq!(judge(42.0, 30.0), Some(true), "3° inside it");
        assert_eq!(judge(42.0, 3.0), None, "a dribbler that stops short of the bag");
    }

    // agy review 2026-11-16 P2-3: a hard bouncer down the first-base side that leaves the field
    // over the foul-ground wall short of the bag (the reviewer's ball: 35 m/s, 12° off the
    // first-base axis toward foul ground, driven down). Its path past the wall crosses the
    // bag's line in the stands; that crossing must not be sent, or the play ends as a foul
    // without the ball ever being dead out of play.
    #[test]
    fn a_ball_gone_out_of_play_short_of_the_bag_does_not_pass_it() {
        // The reviewer's ball, through the whole live play; whether it goes into the dugout and
        // on past the bag's line there depends on the bounce tilts, so on the seed. Search seeds
        // for plays of that shape (every bounce short of the bag, out of play, the path ending
        // past the line) and check none sends a crossing from outside the field.
        let fx = fixture(&[]);
        let a = 12f64.to_radians();
        let along = |x: f64, y: f64| (x + y) / core::f64::consts::SQRT_2;
        let mut found = 0;
        for seed in 0..300u64 {
            let mut st = fx.st.clone();
            let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
            inp.ball = BallState { r: Vec3::new(0.0, 0.5, 0.5), v: Vec3::new(35.0 * a.cos(), 35.0 * a.sin(), -14.0), w: Vec3::ZERO };
            inp.seed = seed;
            let path = batted_path(&fx.env, &fx.field, inp.ball, inp.t_contact, |k, surf| {
                let mut r = rng(seed, inp.play, 0x0200, Purpose::SurfaceBounce, k);
                (surf.roughness * r.truncated_normal(4.0), surf.roughness * r.truncated_normal(4.0))
            });
            let end = path.at(path.t_end()).0;
            if !(matches!(path.end(), SegmentEnd::OverFence { .. }) && path.bounces.iter().all(|&(_, x, y)| along(x, y) < BASE_L) && along(end.x, end.y) > BASE_L) {
                continue;
            }
            found += 1;
            let out = simulate_live(&inp, &mut st);
            for f in &out.facts {
                if let Input::BallPassedBase { t, .. } = *f {
                    let p = out.path.at(t).0;
                    assert!(fx.field.ball_clearance(p.x, p.y, fx.env.ball.radius).dist >= 0.0, "seed {seed}: passed the bag at {t:.3} s from outside the field, {p:?}");
                }
            }
            assert!(out.events.iter().any(|e| matches!(e, Event::BallDead { reason: bb_rules::DeadBall::OutOfPlay })), "seed {seed}: never dead out of play: {:?}", out.events);
        }
        assert!(found > 0, "no seed gave the reviewer's case");
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
            bats: bb_observe::Hand::Right,
            pitch_x: None,
            bat_yaw: None,
            swing_lead: None,
            seed: 11,
            play: 3,
            record: false,
            runner_start: Default::default(),
            tag_up: Default::default(), advance: Default::default(),
        }
    }

    /// Put a free ball at `r` with velocity `v` at time `t`, with nobody assigned to it.
    fn loose_ball(sim: &mut Sim, t: f64, r: Vec3, v: Vec3) {
        sim.t = t;
        sim.set_free(BallState { r, v, w: Vec3::ZERO });
        for fd in sim.f.iter_mut() {
            fd.role = Role::Idle;
            fd.target = fd.pos;
            fd.vel = Vec3::ZERO;
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

    /// Fielder `i` going for the ball with call `tier`, shown long enough ago that his
    /// teammates know it (`Sim::known`).
    fn known_chaser(sim: &mut Sim, i: usize, tier: u8) {
        sim.f[i].role = Role::Chase;
        sim.f[i].call = tier;
        let at = sim.t - CALL_HEARD_S - DT;
        sim.ball_since = sim.ball_since.min(at);
        sim.f[i].shown.push(Shown { at, role: Role::Chase, backs: None, tier, asks: None });
    }

    /// Fielder `i` doing job `role`, shown long enough ago that his teammates know it
    /// (`Sim::seen`).
    fn known_doing(sim: &mut Sim, i: usize, role: Role) {
        sim.f[i].role = role;
        if let Role::Cover(b) = role {
            sim.f[i].target = base_pos(b);
        }
        let at = sim.t - CALL_HEARD_S - DT;
        let backs = sim.f[i].backs;
        sim.f[i].shown.push(Shown { at, role, backs, tier: 0, asks: None });
    }

    /// Let `CALL_HEARD_S` pass with the coordination steps of a tick (`simulate_live_inner`):
    /// each fielder shows what he is doing, hears what the others did, and moves.
    fn let_calls_be_heard(sim: &mut Sim) {
        let until = sim.t + CALL_HEARD_S + DT;
        while sim.t < until {
            sim.publish();
            sim.move_fielders();
            sim.t += DT;
            sim.hear();
            sim.update_relay();
        }
    }

    /// Codex live-play review P1-1 probe: everything visible held fixed, only the centre
    /// fielder's private target moved from left-centre to right-centre. Who covers second
    /// must not change: nobody can see where another man means to go.
    #[test]
    fn second_base_cover_does_not_read_a_private_target() {
        let fx = fixture(&[(0x0007, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0007, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut cover2 = Vec::new();
        for x in [-40.0, 40.0] {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = 1.0;
            known_chaser(&mut sim, CF, 0);
            sim.f[CF].target = Vec3::new(x, 85.0, 0.0);
            sim.cover_roles(&[]);
            cover2.push((0..9).find(|&i| sim.f[i].role == Role::Cover(2)));
        }
        assert_eq!(cover2[0], cover2[1], "second's cover followed the centre fielder's private target");
    }

    /// Codex fielder-communication review P1-1, first probe: the centre fielder's chase is
    /// known; the left fielder has just chosen to chase too, but nobody can know that yet.
    /// The right fielder's backup job is the same whether or not the left fielder has
    /// privately chosen to go: he cannot hear a choice before it is shown.
    #[test]
    fn a_backup_job_does_not_read_an_unheard_chase() {
        const LF: usize = 6;
        const RF: usize = 8;
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut rf = Vec::new();
        for lf_chasing in [false, true] {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = 1.0;
            sim.f[CF].pos = Vec3::new(0.0, 85.0, 0.0);
            sim.f[LF].pos = Vec3::new(0.0, 80.0, 0.0);
            sim.f[RF].pos = Vec3::new(10.0, 80.0, 0.0);
            known_chaser(&mut sim, CF, 0);
            if lf_chasing {
                sim.f[LF].role = Role::Chase;
            }
            assert_eq!(sim.known(LF), (false, 0, false), "the left fielder's choice is not shown");
            sim.assign_backups();
            rf.push((sim.f[RF].role, sim.f[RF].target));
        }
        assert_eq!(rf[0], rf[1], "the right fielder's backup job followed the left fielder's unheard chase");
    }

    /// Codex fielder-communication review P1-1, second probe (and Opus infield-fixes review
    /// P2-3): a ground ball up the middle. Only the second baseman's private intercept time
    /// changes, which turns his own choice from going for it to covering second. Nobody can
    /// know that choice yet, so the shortstop's job must not change with it.
    #[test]
    fn second_base_cover_does_not_read_a_partners_unheard_choice() {
        const B2: usize = 3;
        const SS: usize = 5;
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut seen = Vec::new();
        for t_int in [0.8, 20.0] {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.f[SS].pos = Vec3::new(1.0, 25.0, 0.0);
            loose_ball(&mut sim, 0.4, Vec3::new(0.0, 0.5, 0.5), Vec3::new(1.5, 24.0, -3.0));
            sim.f[B2].t_int = t_int;
            let d = sim.decide(B2);
            assert_eq!(sim.known(B2), (false, 0, false), "the second baseman's choice is not shown");
            // Only he reads the ball again (a re-call, `recall`); the shortstop keeps out of it.
            sim.assign_among(vec![B2]);
            seen.push((d, sim.f[SS].role));
        }
        assert_eq!((seen[0].0, seen[1].0), (Decision::Go, Decision::CoverSecond), "the probe's premise: his own choice changes");
        assert_eq!(seen[0].1, seen[1].1, "the shortstop's job followed the second baseman's unheard choice");
    }

    // Codex day review #2: a fly a fielder can catch just before it reaches the wall is an out,
    // not a carom (rules-spec §5.3: after the wall a catch is no longer possible).
    #[test]
    fn fly_catchable_just_before_the_wall_is_caught() {
        let mut fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.f[CF].pos = Vec3::new(0.0, 80.0, 0.0);
        loose_ball(&mut sim, 2.0, Vec3::new(0.0, 79.05, 1.0), Vec3::new(0.0, 1.0, 0.0));
        sim.f[CF].vel = Vec3::new(0.0, 1.0, 0.0) * 8.0;
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
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

    /// The center fielder on the ground from a dive at time 1.0 (get-up ends at 2.5), and a
    /// ball rolling across in front of him at 6 m/s, passing `pass_m` from him at time 2.0.
    fn down_after_dive(pass_m: f64) -> (Vec<Input>, f64) {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.f[CF].pos = Vec3::new(0.0, 80.0, 0.0);
        loose_ball(&mut sim, 1.5, Vec3::new(-3.0, 80.0 - pass_m, 0.04), Vec3::new(6.0, 0.0, 0.0));
        sim.f[CF].role = Role::Chase;
        sim.f[CF].dove = true;
        sim.f[CF].ready_at = 1.0 + DIVE_GETUP_S;
        for _ in 0..60 {
            world_tick(&mut sim);
        }
        (sim.out.facts.clone(), sim.f[CF].ready_at)
    }

    // Seed 7475 play 25 (2026-11-13): the center fielder's dive rimmed a 123 mph grounder off
    // his glove, he dove again for the dribbler from the ground half a second later, and the
    // catch started a second full get-up, so he held the ball 2.4 s. Down from a dive he can
    // grab a ball within arm's reach (agent-models §5.4: a bobble that stays close can be
    // re-fielded) but cannot dive again, and his get-up runs from the first dive.
    #[test]
    fn a_fielder_down_from_a_dive_does_not_dive_again() {
        let (facts, _) = down_after_dive(1.6);
        assert!(!facts.iter().any(|f| matches!(f, Input::GloveSecured { .. })), "dove from the ground: {facts:?}");
    }

    /// Fielder `i` (average hands unless `sure`) standing at (0, 80), chasing a ground ball
    /// that crosses in front of him at 20 m/s, `pass_m` away at its closest, `lead_s` after
    /// it appears at time `t0`. Runs 0.8 s of world ticks.
    fn crossing_ball(i: usize, pass_m: f64, lead_s: f64, t0: f64, sure: bool) -> (Sim<'static>, f64) {
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&[])));
        let mut inp = input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        if !sure {
            inp.fattrs = core::array::from_fn(FielderAttrs::average);
        }
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(inp));
        let st: &'static mut RulesState = Box::leak(Box::new(fx.st.clone()));
        let mut sim = new_sim(inp, st);
        sim.f[i].pos = Vec3::new(0.0, 80.0, 0.0);
        loose_ball(&mut sim, t0, Vec3::new(-20.0 * lead_s, 80.0 - pass_m, 0.04), Vec3::new(20.0, 0.0, 0.0));
        sim.f[i].role = Role::Chase;
        sim.f[i].belief = Some(sim.predict(i));
        for _ in 0..48 {
            world_tick(&mut sim);
        }
        let transfer = sim.f[i].a.transfer;
        (sim, transfer)
    }

    fn secured_at(sim: &Sim) -> Option<f64> {
        sim.out.facts.iter().find_map(|f| if let Input::GloveSecured { t, .. } = f { Some(*t) } else { None })
    }

    // Owner backlog A5 (2026-11-13): fielders dove 22 times a game, mostly for balls a real
    // fielder takes with a stretch. A ball passing 1.3 m away (reach is 1.0 m, a stretch
    // 0.4 m more) is taken on his feet: no dive, and he can throw after his transfer plus
    // the 0.3 s it takes to get his balance back, not after a 1.5 s get-up.
    #[test]
    fn a_ball_just_past_reach_is_taken_with_a_stretch_not_a_dive() {
        let (sim, transfer) = crossing_ball(CF, 1.3, 0.15, 1.5, true);
        let t = secured_at(&sim).expect("he never got it");
        assert!(sim.out.dives.is_empty(), "he dove: {:?}", sim.out.dives);
        assert!((sim.f[CF].ready_at - (t + transfer + 0.3)).abs() < 1e-9, "secured {t:.3}, ready {:.3}, transfer {transfer:.2}", sim.f[CF].ready_at);
    }

    // The body takes 0.3 s to lay out 1.5 m. A ball passing 2.2 m away needs 1.2 m of that,
    // 0.24 s: seen 0.15 s ahead it goes by; seen 0.4 s ahead he gets there.
    #[test]
    fn a_dive_needs_its_lunge_time() {
        let (late, _) = crossing_ball(CF, 2.2, 0.15, 1.5, true);
        assert!(late.out.dives.is_empty() && secured_at(&late).is_none(), "laid out 1.2 m in 0.15 s: {:?}", late.out.dives);
        let (early, _) = crossing_ball(CF, 2.2, 0.4, 1.5, true);
        assert_eq!(early.out.dives.len(), 1, "with 0.4 s he should dive");
        assert!(secured_at(&early).is_some());
    }

    // An outfielder has nobody behind him: at nearly full stretch (2.3 m, where an average
    // glove gets about one ball in three) he stays on his feet. A shortstop, with the
    // outfield behind him, goes for the same ball.
    #[test]
    fn an_outfielder_does_not_dive_for_a_ball_he_will_probably_miss() {
        const SS: usize = 5;
        let (cf, _) = crossing_ball(CF, 2.3, 0.5, 1.5, false);
        assert!(cf.out.dives.is_empty(), "the center fielder dove: {:?}", cf.out.dives);
        let (ss, _) = crossing_ball(SS, 2.3, 0.5, 1.5, false);
        assert_eq!(ss.out.dives.len(), 1, "the shortstop should dive");
    }

    /// The center fielder (hands `hands`) at (0, 80) with a dive long earned and his reads
    /// frozen: the world's ball is at `r` moving `world_v`, and he believes it is the same
    /// ball moving `believed_v`. Runs `ticks` world ticks.
    fn weighed_dive(hands: f64, r: Vec3, world_v: Vec3, believed_v: Vec3, ticks: usize) -> usize {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.fattrs[CF].hands = hands;
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.f[CF].pos = Vec3::new(0.0, 80.0, 0.0);
        loose_ball(&mut sim, 2.0, r, believed_v);
        let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
        let believed = path.clone();
        loose_ball(&mut sim, 2.0, r, world_v);
        let fd = &mut sim.f[CF];
        (fd.role, fd.belief) = (Role::Chase, Some(believed));
        (fd.must_dive_since, fd.last_dive_read, fd.last_close_read) = (0.0, f64::INFINITY, f64::INFINITY);
        for _ in 0..ticks {
            world_tick(&mut sim);
        }
        sim.out.dives.len()
    }

    // Codex dive-rate review #1: the outfielder weighed his dive with the world's ball speed.
    // It is his read that counts: believing the ball slow he likes his chances 1.5 m out
    // (53% with average hands) and goes; believing it fast (48%) he does not, whatever the
    // ball is really doing.
    #[test]
    fn an_outfielder_weighs_a_dive_by_his_own_read_of_the_ball() {
        let hands = FielderAttrs::average(CF).hands;
        let r = Vec3::new(0.0, 78.5, 0.04);
        let v = |s: f64| Vec3::new(s, 0.0, 0.0);
        for world in [5.0, 20.0, 40.0] {
            assert_eq!(weighed_dive(hands, r, v(world), v(5.0), 1), 1, "believed slow, really {world} m/s");
            assert_eq!(weighed_dive(hands, r, v(world), v(40.0), 1), 0, "believed fast, really {world} m/s");
        }
    }

    // Codex dive-rate review #2: a ball coming down gains speed, so a tick later the same
    // dive can look worse. Having decided to go while waiting for it to come closest, he
    // goes. Hands chosen so the first tick is just over an even chance and the next under.
    #[test]
    fn an_outfielder_who_has_decided_to_dive_goes() {
        let (r, v) = (Vec3::new(-0.02, 78.4, 1.0), Vec3::new(1.0, 0.0, -10.0));
        let chance = |hands: f64, dt: f64| {
            let (q, qv) = (r + v * dt, v + Vec3::new(0.0, 0.0, -9.8 * dt));
            let sg = hands * glove_sigma_mult(qv.norm(), m::hypot(q.x, q.y - 80.0), true);
            1.0 - m::exp(-0.5 * (POCKET / sg) * (POCKET / sg))
        };
        let mut hands = 0.018;
        while chance(hands, 0.0) < 0.5 {
            hands *= 0.9999;
        }
        assert!(chance(hands, 0.0) >= 0.5 && chance(hands, DT) < 0.5, "fixture: {} then {}", chance(hands, 0.0), chance(hands, DT));
        assert_eq!(weighed_dive(hands, r, v, v, 3), 1);
    }

    // The pitcher is finishing his delivery at contact (0.4 s here): a ball by him 0.5 s
    // after contact finds him not yet able to lay out (0.7 s). The shortstop, set and
    // reacting in 0.25 s, dives for the same ball.
    #[test]
    fn the_pitcher_cannot_dive_until_he_has_recovered_from_his_delivery() {
        const SS: usize = 5;
        let (p, _) = crossing_ball(0, 1.8, 0.3, 0.6, true);
        assert!(p.out.dives.is_empty(), "the pitcher dove 0.5 s after contact");
        let (ss, _) = crossing_ball(SS, 1.8, 0.3, 0.6, true);
        assert_eq!(ss.out.dives.len(), 1);
    }

    // Before 2026-11-13 a fielder running down a slow roller dove at it from 2.5 m (4 dives
    // a game at balls under 10 m/s). He runs on and picks it up.
    #[test]
    fn nobody_dives_at_a_ball_he_is_about_to_run_down() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.f[CF].pos = Vec3::new(0.0, 80.0, 0.0);
        loose_ball(&mut sim, 1.5, Vec3::new(0.0, 84.0, 0.04), Vec3::new(0.0, 5.0, 0.0));
        sim.f[CF].role = Role::Chase;
        sim.f[CF].vel = Vec3::new(0.0, 7.0, 0.0);
        let pred = sim.predict(CF);
        let (ti, pt) = sim.intercept(CF, &pred);
        (sim.f[CF].t_int, sim.f[CF].target, sim.f[CF].belief) = (ti, pt, Some(pred));
        for _ in 0..180 {
            world_tick(&mut sim);
        }
        assert!(secured_at(&sim).is_some(), "he never picked it up");
        assert!(sim.out.dives.is_empty(), "he dove at a 5 m/s roller: {:?}", sim.out.dives);
        // And with no stretch either: ready after his plain transfer.
        assert!((sim.f[CF].ready_at - (secured_at(&sim).unwrap() + sim.f[CF].a.transfer)).abs() < 1e-9);
    }

    // Codex dive animation review #1: where the fair and foul fences meet, a straight lunge
    // between two points inside the wall can cut the corner. The body stops short of it.
    #[test]
    fn a_dive_never_carries_the_body_through_the_wall() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let at = |th: f64, inset: f64| {
            let r = fx.field.fence_at(th) - inset;
            Vec3::new(r * m::sin(th), r * m::cos(th), 0.0)
        };
        let fielder = at(core::f64::consts::FRAC_PI_4 - 0.017, 0.001);
        // The ball is across the corner on the foul side, inside every wall there: the taper wall
        // runs inside the fence's curve at that angle, so the point on the curve is 1.8 m past it
        // and out of anyone's reach over the wall (`OVER_WALL_REACH_M`), then moved in 5 cm.
        let on_curve = at(core::f64::consts::FRAC_PI_4 + 0.018, 0.001);
        let w = fx.field.body_clearance(on_curve.x, on_curve.y, 0.0);
        let ball = on_curve + w.inward * (-w.dist + 0.05);
        assert!(fx.field.body_clearance(ball.x, ball.y, 0.0).dist > 0.04, "fixture: the ball is inside the walls");
        sim.f[CF].pos = fielder;
        loose_ball(&mut sim, 2.0, ball + Vec3::new(0.0, 0.0, 0.05), (ball - fielder).normalized() * 4.0);
        sim.f[CF].role = Role::Chase;
        // He has long known that only a dive gets it (and is not asked again).
        sim.f[CF].must_dive_since = 0.0;
        sim.f[CF].last_dive_read = f64::INFINITY;
        for _ in 0..40 {
            world_tick(&mut sim);
            let p = sim.f[CF].pos;
            let c = fx.field.body_clearance(p.x, p.y, BODY_R).dist;
            assert!(c >= -1e-6, "at {:.2} s the body is {:.3} m through a wall", sim.t, -c);
        }
        assert!(!sim.out.dives.is_empty(), "he never dove (gap {:.2} m)", (ball - fielder).norm());
    }

        // Codex dive review #2: a low ball in reach of a fielder who is down is taken now, not
    // deferred to the next tick, when it will have risen past what he can reach lying down.
    #[test]
    fn a_fielder_down_from_a_dive_takes_a_rising_ball_while_it_is_low() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.f[CF].pos = Vec3::new(0.0, 80.0, 0.0);
        loose_ball(&mut sim, 1.5, Vec3::new(-0.3, 80.0, DIVE_MAX_Z - 0.01), Vec3::new(6.0, 0.0, 2.0));
        sim.f[CF].dove = true;
        sim.f[CF].ready_at = 2.5;
        for _ in 0..20 {
            world_tick(&mut sim);
        }
        assert!(sim.out.facts.iter().any(|f| matches!(f, Input::GloveSecured { .. })), "facts: {:?}", sim.out.facts);
    }

    #[test]
    fn a_fielder_down_from_a_dive_grabs_a_ball_in_reach_and_gets_up_on_time() {
        let (facts, ready_at) = down_after_dive(0.5);
        assert!(facts.iter().any(|f| matches!(f, Input::GloveSecured { .. })), "facts: {facts:?}");
        // Up at 2.5 (the first dive's get-up), then the average outfielder's 0.85 s transfer.
        assert!((ready_at - (2.5 + 0.85)).abs() < 1e-9, "ready to throw at {ready_at}");
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
        assert!(sim.caught());
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 3, speed: 12.0, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
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

    /// A runner standing on third, tagging up, with `tag` for his start; the ball caught by
    /// the center fielder `cf_y` m out at 5.0 s. Runs the runners only.
    fn tag_up_from_third(tag: bb_agents::running::TagUpParams, cf_y: f64, v_before: f64) -> (Sim<'static>, Vec<f64>) {
        const R: AgentId = 0x0007;
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&[(R, 3)])));
        let mut inp = input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.tag_up = tag;
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(inp));
        let st: &'static mut RulesState = Box::leak(Box::new(fx.st.clone()));
        let mut sim = new_sim(inp, st);
        (sim.r[1].s, sim.r[1].v, sim.r[1].goal, sim.r[1].tag_up, sim.r[1].hold_s) = (3.0 * BASE_L, v_before, 3, true, None);
        sim.t = 5.0;
        sim.f[CF].pos = Vec3::new(0.0, cf_y, 0.0);
        let id = sim.f[CF].id;
        sim.emit(Input::GloveSecured { t: 5.0, fielder: id, x: 0.0, y: cf_y });
        sim.ball = Ball::Held { by: CF };
        sim.secured_at = 5.0;
        let mut speeds = Vec::new();
        for _ in 0..720 {
            if sim.done {
                break;
            }
            sim.move_runners();
            speeds.push(sim.r[1].v);
            sim.t += DT;
        }
        (sim, speeds)
    }

    fn home_touch(sim: &Sim) -> Option<f64> {
        sim.out.facts.iter().find_map(|f| match *f {
            Input::BaseTouched { t, agent: 0x0007, base: 4, .. } => Some(t),
            _ => None,
        })
    }

    // agent-models §6.4: from the set start his legs run the law his decision assumed. With
    // a 0.2 s first step and τ 0.6 s, 90 ft at 8.2 m/s from rest takes t with
    // t − 0.6·(1 − e^(−t/0.6)) = 27.432/8.2 = 3.34537, so t = 3.94453 (hand iteration): he
    // touches home 0.2 + 3.94453 s after the 5.0 s catch, within the 60 Hz step's error.
    #[test]
    fn a_tag_up_runs_the_set_start_he_judged() {
        let tag = bb_agents::running::TagUpParams { react_s: 0.2, tau_gain: 0.6 / RUNNER_TAU, typical_tag_s: 0.0 };
        // A throw from 140 m needs a relay: the run is clearly on, whatever his timing draw.
        let (sim, _) = tag_up_from_third(tag, 140.0, 0.0);
        let t = home_touch(&sim).expect("he went");
        assert!((t - (5.0 + 0.2 + 3.94453)).abs() < 2.0 * DT, "home at {t:.4}");
        assert!((tag_up_time(tag, 0.2, BASE_L, 8.2, RUNNER_TAU) - (0.2 + 3.94453)).abs() < 1e-3);
    }

    // §6.4: standing on the bag he has no speed, whatever the state said before (before A4 a
    // tagging runner kept 6.19 m/s from coming back to third). His first moving tick starts
    // from rest: v = 8.2 · DT / τ.
    #[test]
    fn no_speed_carries_over_into_a_tag_up() {
        let tag = bb_agents::running::TagUpParams::default();
        let (_, speeds) = tag_up_from_third(tag, 140.0, 6.19);
        assert!((speeds[0] - 8.2 * DT / (RUNNER_TAU * tag.tau_gain)).abs() < 1e-9, "first tick {:.4} m/s", speeds[0]);
    }

    // §6.4: his decision uses his own start. A runner who would need 3 s more to get going
    // does not try it on a ball caught 95 m out; with a 0.2 s first step he does.
    #[test]
    fn the_decision_uses_the_set_start() {
        let quick = bb_agents::running::TagUpParams { react_s: 0.2, tau_gain: 0.6 / RUNNER_TAU, typical_tag_s: 0.0 };
        let slow = bb_agents::running::TagUpParams { react_s: 3.2, ..quick };
        assert!(home_touch(&tag_up_from_third(quick, 140.0, 0.0).0).is_some());
        assert!(home_touch(&tag_up_from_third(slow, 95.0, 0.0).0).is_none(), "he should hold");
    }

    /// The catcher on the plate takes the ball at 2.0 s with an unforced runner 1.0 m from it,
    /// coming in at 8 m/s; the catcher needs `tag_s` to put a tag on.
    fn tag_at_home(tag_s: f64) -> Sim<'static> {
        const R: AgentId = 0x0007;
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&[(R, 3)])));
        let mut inp = input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.fattrs[1].tag_apply_s = tag_s;
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(inp));
        let st: &'static mut RulesState = Box::leak(Box::new(fx.st.clone()));
        let mut sim = new_sim(inp, st);
        sim.t = 2.0;
        held_at(&mut sim, 1, base_pos(4));
        sim.secured_at = 2.0;
        (sim.r[1].s, sim.r[1].v, sim.r[1].goal, sim.r[1].hold_s) = (4.0 * BASE_L - 1.0, 8.0, 4, None);
        for _ in 0..30 {
            if sim.done {
                break;
            }
            sim.holder_act();
            sim.move_runners();
            sim.t += DT;
        }
        sim
    }

    // §6.4: a tag is not instant. With none needed he is tagged on the catch; given 0.3 s,
    // the runner covers his last metre in 1/8 s and is home first (hand calculation).
    #[test]
    fn a_tag_needs_its_time() {
        let tags = |sim: &Sim| count(sim, |f| matches!(f, Input::TagContact { runner: 0x0007, .. }));
        let instant = tag_at_home(0.0);
        assert!(tags(&instant) >= 1, "tagged on the catch");
        let real = tag_at_home(0.3);
        assert_eq!(tags(&real), 0);
        assert!(home_touch(&real).is_some_and(|t| t < 2.0 + 0.3));
    }

    // Diagnostic (agent-models §6.4, tag-up sources note §3): the ball's side of a tag-up
    // race by depth. `cargo test -p bb_game --release --lib tag_up_race_table -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn tag_up_race_table() {
        let cf = crate::play::arm_strength_50_mps(8);
        for ft in [200.0, 225.0, 250.0, 275.0, 280.0, 300.0, 325.0, 350.0] {
            let d = ft * 0.3048;
            let flight = throw_time(THROW_EFFORT * cf, d);
            let belief = TYPICAL_TRANSFER + typical_throw_time(d, Cut::No);
            let run = run_time(BASE_L, 8.2, RUNNER_TAU);
            println!("{ft:4} ft {d:6.1} m  CF arm {cf:.2}: flight {flight:.2} s, +0.85 transfer {:.2} s | runner believes ball {belief:.2} s (37 m/s, relay if quicker); runner {run:.2} s", 0.85 + flight);
        }
    }

    // Gemini A4 review (agent-models §6.4): whether the runner ahead holds a base is read
    // from what can be seen, never his private goal. The runner from second stands on second;
    // the runner from first is coming. Seen standing 0.1 s: not yet taken as holding (a runner
    // who has just decided to go is still slow for a few ticks); 0.25 s: holding, whatever
    // he privately intends. Seen running on toward third: never.
    #[test]
    fn the_base_ahead_is_held_by_what_the_runner_sees() {
        let fx = fixture(&[(6, 1), (7, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 6, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 7, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 3.0;
        (sim.r[1].s, sim.r[1].v, sim.r[1].goal) = (BASE_L + 15.0, 8.0, 2);
        (sim.r[2].s, sim.r[2].v) = (2.0 * BASE_L, 0.0);
        for goal in [2, 3] {
            sim.r[2].goal = goal;
            sim.r[2].still_since = Some(2.9);
            assert!(!sim.base_held_ahead(1, 2), "goal {goal}: seen still for 0.1 s");
            sim.r[2].still_since = Some(2.75);
            assert!(sim.base_held_ahead(1, 2), "goal {goal}: seen still for 0.25 s");
            sim.r[2].still_since = None;
            assert!(!sim.base_held_ahead(1, 2), "goal {goal}: seen running");
        }
    }

    // Gemini tag-up review P1-2: the runner from second has stood on his bag for a second
    // and now breaks for third; the runner from first (unforced: the batter is out) is
    // coming to second. On the tick he leaves he still looks to be holding, so the runner
    // behind may start back, but he keeps reading: second visibly clears and he takes it.
    // (Before the fix a turn-back was final and he never reached second.)
    #[test]
    fn a_runner_ahead_breaking_forward_does_not_turn_the_runner_behind_back() {
        let fx = fixture(&[(6, 1), (7, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 6, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 7, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 3.0;
        sim.r[0].out = true;
        (sim.r[1].s, sim.r[1].v, sim.r[1].goal, sim.r[1].hold_s) = (BASE_L + 10.0, 8.0, 2, None);
        (sim.r[2].s, sim.r[2].v, sim.r[2].goal, sim.r[2].hold_s, sim.r[2].still_since) = (2.0 * BASE_L, 0.0, 3, None, Some(2.0));
        for _ in 0..180 {
            sim.move_runners();
            sim.t += DT;
        }
        assert!(sim.out.facts.iter().any(|f| matches!(f, Input::BaseTouched { agent: 6, base: 2, .. })), "the runner behind never reached second: s {:.2} goal {}", sim.r[1].s, sim.r[1].goal);
    }

    // Tag-up sources note §1, §7 (agent-models §6.4): an average runner (8.2 m/s) tagging
    // from third reaches home about 3.9–4.3 s after the catch (standing 90-ft mean 4.03 s,
    // Statcast 2017–2020). The decision's start model, at its set values, must land there.
    #[test]
    fn the_tag_up_time_is_inside_the_measured_range() {
        let t = tag_up_time(bb_agents::running::TagUpParams::default(), 0.0, BASE_L, 8.2, RUNNER_TAU);
        assert!((3.9..=4.3).contains(&t), "third to home {t:.3} s");
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 2, speed: 16.0, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
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

    #[test]
    fn an_early_legal_retouch_keeps_its_time_and_gets_a_decision_on_the_call() {
        const R: AgentId = 7;
        let fx = fixture(&[(R, 3)]);
        let inp = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: R, base: 3, speed: 9.6, tau_s: RUNNER_TAU },
        ]);
        for gap in [0.08, 0.45] {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.start_runners();
            sim.r[1].reading_contact = false;
            let call = 5.0;
            sim.t = call;
            sim.emit(Input::BaseLeft { t: call, agent: R, base: 3 });
            let r = &mut sim.r[1];
            (r.s, r.v, r.direction, r.goal) = (3.0 * BASE_L + gap, 1.0, -1.0, 3);
            sim.emit(Input::GloveSecured { t: call, fielder: sim.f[CF].id, x: 0.0, y: 120.0 });
            assert!(sim.caught());
            sim.ball = Ball::Held { by: CF };
            sim.f[CF].pos = Vec3::new(0.0, 120.0, 0.0);
            for tick in 0..36 {
                sim.t = call + f64::from(tick) / 60.0;
                sim.move_runners();
                if tick < 15 {
                    assert_eq!(sim.r[1].decisions, 0);
                }
            }
            let touches: Vec<_> = sim.out.facts.iter().filter_map(|fact| match *fact {
                Input::BaseTouched { agent: R, base: 3, t, holding_ball: false } => Some(t),
                _ => None,
            }).collect();
            assert_eq!(touches.len(), 1);
            if gap < 0.1 {
                // For an 8 cm gap at the final walking speed, expect about 80 ms.
                assert!((0.04..0.10).contains(&(touches[0] - call)));
            } else {
                assert!(touches[0] >= call + 0.25);
            }
            assert_eq!(sim.r[1].decisions, 1);
            assert_eq!(sim.r[1].goal, 4, "the deep fly is an advance from third");
        }
    }

    /// Catch-read fixes review P2-1. A runner coming back to third at 2.5 m/s, 8 cm off at
    /// the catch, touches it legally within a tick or two and carries on past it, with and
    /// without an anticipated tag-up. Physics and the rules, independent of the planner: a
    /// body crossing the bag and ending the tick off it has left it; a touch happens only on
    /// a tick whose movement reaches the bag. His first touch is the legal retouch, and he
    /// gets one advance decision, when he has come back and planted. A runner already
    /// planted on third at the catch, with no tag-up set, also gets that one decision once
    /// he sees the call.
    #[test]
    fn a_retouch_at_speed_keeps_one_touch_reports_the_departure_and_decides_at_the_plant() {
        const R: AgentId = 7;
        let fx = fixture(&[(R, 3)]);
        let inp = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: R, base: 3, speed: 9.6, tau_s: RUNNER_TAU },
        ]);
        let third = 3.0 * BASE_L;
        let run = |gap: f64, speed: f64, tag_up: bool| {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.start_runners();
            sim.r[1].reading_contact = false;
            let call = 5.0;
            sim.t = call;
            if gap > 0.0 {
                sim.emit(Input::BaseLeft { t: call, agent: R, base: 3 });
            }
            let r = &mut sim.r[1];
            (r.s, r.v, r.direction, r.goal, r.tag_up) = (third + gap, speed, -1.0, 3, tag_up);
            sim.emit(Input::GloveSecured { t: call, fielder: sim.f[CF].id, x: 0.0, y: 120.0 });
            assert!(sim.caught());
            sim.ball = Ball::Held { by: CF };
            sim.f[CF].pos = Vec3::new(0.0, 120.0, 0.0);
            let mut decided_at = None;
            for tick in 0..90 {
                sim.t = call + f64::from(tick) * DT;
                let (prev, facts) = (sim.r[1].s, sim.out.facts.len());
                sim.move_runners();
                let now = sim.r[1].s;
                for fact in &sim.out.facts[facts..] {
                    if let Input::BaseTouched { agent: R, base: 3, .. } = fact {
                        assert!(prev.min(now) <= third + 1e-9 && prev.max(now) >= third - 1e-9, "touched third from {prev:.3} to {now:.3}, tag-up {tag_up}");
                    }
                }
                if decided_at.is_none() && sim.r[1].decisions > 0 {
                    decided_at = Some((sim.t, prev, sim.r[1].decisions));
                }
            }
            let warned = sim.out.events.iter().any(|e| matches!(e, Event::RulesWarning { .. }));
            (sim.out.facts.clone(), warned, sim.r[1].decisions, sim.r[1].goal, decided_at)
        };
        for tag_up in [false, true] {
            let (facts, warned, decisions, _, decided) = run(0.08, 2.5, tag_up);
            let touches: Vec<f64> = facts.iter().filter_map(|f| match *f {
                Input::BaseTouched { agent: R, base: 3, t, .. } => Some(t),
                _ => None,
            }).collect();
            let leaves: Vec<f64> = facts.iter().filter_map(|f| match *f {
                Input::BaseLeft { agent: R, base: 3, t } => Some(t),
                _ => None,
            }).collect();
            assert!(touches[0] > 5.0 && touches[0] < 5.1, "the legal retouch, tag-up {tag_up}: {touches:?}");
            assert!(leaves.contains(&touches[0]), "the drift past the bag is a departure, tag-up {tag_up}: {leaves:?}");
            let (t, from, n) = decided.expect("a decision once back on the bag");
            assert_eq!(n, 1, "one decision, tag-up {tag_up}");
            assert!(t >= 5.0 + RUNNER_CATCH_CALL_REACT_S - 1e-9, "after the call is seen");
            assert!(from < third - 1e-6 && t > touches[0] + 0.1, "decided on coming back onto the bag from past it, not while drifting, tag-up {tag_up}: {decided:?}, touches {touches:?}");
            assert_eq!(decisions, 1, "and no second decision, tag-up {tag_up}");
            assert!(!warned);
        }
        let (_, _, _, goal, decided) = run(0.0, 0.0, false);
        assert!(decided.is_some_and(|(t, _, n)| n == 1 && (t - 5.0 - RUNNER_CATCH_CALL_REACT_S).abs() < DT), "planted at the catch: {decided:?}");
        assert_eq!(goal, 4, "the deep fly is an advance from third");
    }

    /// Catch-read fixes review P1-3: the same visible glove possession near third, with
    /// and without the catch ruled (the second arm's ground touch reaches only the rules,
    /// so nothing public differs). A runner 4 m off second going to third sees the ball
    /// taken in flight either way and reacts only to the call, so his movement through
    /// the reaction window must match. Once the window ends, the possession without a catch
    /// is a new thing to see: he looks again.
    #[test]
    fn a_possession_in_flight_moves_runners_the_same_until_the_call_is_seen() {
        const R: AgentId = 7;
        const B3: usize = 4;
        let fx = fixture(&[(R, 2)]);
        let inp = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: R, base: 2, speed: 8.2, tau_s: RUNNER_TAU },
        ]);
        let run = |ruled: bool| {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.start_runners();
            sim.r[0].reading_contact = false;
            sim.r[1].reading_contact = false;
            let call = 2.0;
            sim.t = call;
            sim.emit(Input::BaseLeft { t: call - 0.5, agent: R, base: 2 });
            let r = &mut sim.r[1];
            (r.s, r.v, r.direction, r.goal) = (2.0 * BASE_L + 4.0, 3.0, 1.0, 3);
            if !ruled {
                step(sim.st, &Input::BallTouched { t: call - 0.1, what: Touch::Ground { x: -20.0, y: 22.0 } });
            }
            let at = base_pos(3) + Vec3::new(2.0, 2.0, 0.0);
            sim.f[B3].pos = at;
            sim.emit(Input::GloveSecured { t: call, fielder: sim.f[B3].id, x: at.x, y: at.y });
            sim.ball = Ball::Held { by: B3 };
            sim.secured_at = call;
            sim.out.secured.push((call, sim.f[B3].id));
            assert_eq!(sim.caught(), ruled);
            assert!(sim.no_catch_at.is_none());
            let mut seen = Vec::new();
            let mut tick = 0;
            while sim.t + 1e-9 < call + RUNNER_CATCH_CALL_REACT_S {
                sim.move_runners();
                let r = &sim.r[1];
                assert!(!r.catch_seen);
                seen.push((r.s, r.v, r.direction, r.goal, r.decisions));
                tick += 1;
                sim.t = call + f64::from(tick) * DT;
            }
            sim.move_runners();
            (seen, sim.r[1].decisions, sim.r[1].catch_seen)
        };
        let (with_catch, with_decisions, saw_catch) = run(true);
        let (without, without_decisions, _) = run(false);
        assert!(with_catch.len() >= 10);
        assert!(with_catch.iter().all(|x| x.4 == 0), "no decision before the call is seen");
        assert_eq!(with_catch, without);
        assert!(saw_catch && with_decisions == 0, "the catch sends him back to retouch");
        assert_eq!(without_decisions, 1, "possession without a catch is a new look once the call is seen");
    }

    #[test]
    fn a_simultaneous_catch_does_not_remove_the_force_from_runner_traffic() {
        const TRAIL: AgentId = 7;
        const LEAD: AgentId = 8;
        let fx = fixture(&[(TRAIL, 1), (LEAD, 2)]);
        let inp = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: TRAIL, base: 1, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: LEAD, base: 2, speed: 8.2, tau_s: RUNNER_TAU },
        ]);
        let tick = |caught| {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.start_runners();
            sim.r[1].reading_contact = false;
            sim.r[2].reading_contact = false;
            sim.t = 5.0;
            sim.emit(Input::BaseLeft { t: sim.t, agent: TRAIL, base: 1 });
            (sim.r[1].s, sim.r[1].v, sim.r[1].goal) = (BASE_L + 8.0, 5.0, 2);
            (sim.r[2].s, sim.r[2].v, sim.r[2].goal) = (2.0 * BASE_L, 0.0, 2);
            sim.r[2].still_since = Some(4.5);
            if caught {
                sim.emit(Input::GloveSecured { t: sim.t, fielder: sim.f[CF].id, x: 0.0, y: 95.0 });
                sim.ball = Ball::Held { by: CF };
                assert!(sim.r[0].out && !sim.forced(1), "the legal force is removed at the catch");
            }
            sim.move_runners();
            let trail = &sim.r[1];
            assert!(!trail.catch_seen && trail.goal == 2 && trail.v > 5.0);
            (trail.s, trail.v, trail.goal, trail.decisions)
        };
        assert_eq!(tick(true), tick(false));
    }

    // Day-fixes review #1 (P1): a runner from third who crossed the plate before the catch
    // (the review's seed 59 play 186 shape) still owes the retouch. Before the fix he stood on
    // home for good (`scored`, never saw the catch) and the defense's appeal took the run
    // off. Now he goes back, retouches third, and touches home again; the rules engine holds
    // one pending run for him, from the second touch.
    #[test]
    fn runner_home_before_the_catch_goes_back_and_touches_again() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 3)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 3, speed: 16.0, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.emit(Input::BaseLeft { t: 1.0, agent: R, base: 3 });
        sim.t = 4.5;
        sim.emit(Input::BaseTouched { t: 4.5, agent: R, base: 4, holding_ball: false });
        {
            let r = &mut sim.r[1];
            r.s = 4.0 * BASE_L;
            r.goal = 4;
            r.last_base = 4;
            r.scored = true;
            r.v = 0.0;
        }
        after_catch(&mut sim);
        let touches = |b: u8| count(&sim, |f| matches!(f, Input::BaseTouched { agent: R, base, holding_ball: false, .. } if *base == b));
        assert_eq!(touches(3), 1, "the retouch; facts: {:?}", sim.out.facts);
        assert_eq!(touches(4), 2, "home before the catch and again after the retouch");
        let retouch_t = sim.out.facts.iter().find_map(|f| match f {
            Input::BaseTouched { t, agent: R, base: 3, .. } => Some(*t),
            _ => None,
        });
        assert!(retouch_t.unwrap() > 5.0);
        assert!(!sim.out.events.iter().any(|e| matches!(e, Event::RulesWarning { .. })), "events: {:?}", sim.out.events);
        let PlayState::Live(lp) = &sim.st.play else { panic!("play ended: {:?}", sim.st.play) };
        assert_eq!(lp.runs_pending.len(), 1);
        assert_eq!(lp.runs_pending[0].1, R);
        assert!(lp.runs_pending[0].0 > retouch_t.unwrap(), "the pending run is the second home touch");
    }

    // Day-fixes review #3: within 3 m of the wall a fielder can catch higher (a leap). A ball
    // leaving that strip must be judged by the height limit where it will be, so a catch
    // available now is not put off to a tick where it is out of reach.
    #[test]
    fn catch_is_not_deferred_out_of_the_wall_leap_zone() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = inp.t_contact + 1.0;
        sim.f[SS].pos = Vec3::new(3.5, 30.0, 0.0);
        let path = grounder(&fx, sim.t, 15.0);
        // Running at the ball's line (−x).
        sim.f[SS].vel = Vec3::new(-1.0, 0.0, 0.0) * 6.0;
        let (_, running) = sim.intercept(SS, &path);
        assert!((running.y - 30.0).abs() < 2.0, "running fielder plans for y {:.1}", running.y);
        sim.f[SS].vel = Vec3::ZERO;
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
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

    // Gemini play-area recheck P2-1. A fielder sees the wall and knows he must slow for it
    // (physics-spec §9.5; `move_fielders` caps his speed into it at √(v_c² + 2·BRAKE·clearance),
    // BRAKE = 4.75 m/s², v_c = 2 m/s). By hand, for a stand point 0.3 m clear of the wall reached
    // at top speed v = 8 m/s: he must be down to √(4 + 2·4.75·0.3) = 2.62 m/s there; braking
    // starts (v² − v_c²)/(2·4.75) − 0.3 = 6.02 m out, and the time lost against running on is
    // ∫dx/√(4 + 9.5(0.3+x)) over those 6.02 m, minus 6.02/8: 2(8 − 2.62)/9.5 − 0.752 = 1.133 −
    // 0.752 = 0.38 s. At the wall itself (clearance 0) it is (v − v_c)/BRAKE − (v² − v_c²)/(2·BRAKE·v)
    // = 1.263 − 0.789 = 0.47 s. The plan must include that, or he promises a catch (and calls
    // teammates off) about half a second before he can make it.
    // Bug caught: planning with the unbraked run (plan early by ~0.4 s).
    #[test]
    fn the_plan_includes_the_slow_down_for_the_wall() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let field = &fx.field;
        // The stand point on the center-field line whose body is `c` clear of the wall.
        let stand_y = |c: f64| {
            let (mut lo, mut hi) = (60.0, 200.0);
            for _ in 0..60 {
                let mid = 0.5 * (lo + hi);
                if field.body_clearance(0.0, mid, BODY_R).dist > c { lo = mid } else { hi = mid }
            }
            0.5 * (lo + hi)
        };
        for &c in &[0.3, 1.0, 3.0] {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = 2.0;
            let y = stand_y(c);
            let v = sim.f[CF].a.speed;
            sim.f[CF].pos = Vec3::new(0.0, y - 40.0, 0.0);
            sim.f[CF].vel = Vec3::new(0.0, v, 0.0);
            sim.f[CF].target = Vec3::new(0.0, y + REACH * 0.7, 0.0);
            sim.f[CF].role = Role::Chase;
            sim.f[CF].t_int = 0.0;
            let goal = sim.f[CF].target;
            let plan = run_time_to(field, &sim.f[CF], goal, REACH * 0.7) + wall_delay(field, &sim.f[CF], goal, REACH * 0.7);
            let mut truth = f64::NAN;
            for k in 1..4000 {
                sim.move_fielders();
                sim.t += DT;
                if (goal - sim.f[CF].pos).norm() <= REACH * 0.7 + 0.02 {
                    truth = k as f64 * DT;
                    break;
                }
            }
            assert!(truth.is_finite(), "never arrived with {c} m clear");
            assert!((plan - truth).abs() < 0.12, "{c} m clear: plan {plan:.3} s, the movement step {truth:.3} s");
        }
        // The hand figure at the wall itself.
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let v = sim.f[CF].a.speed;
        let y = stand_y(0.0);
        sim.f[CF].pos = Vec3::new(0.0, y - 40.0, 0.0);
        sim.f[CF].vel = Vec3::new(0.0, v, 0.0);
        let d = wall_delay(field, &sim.f[CF], Vec3::new(0.0, y + REACH * 0.7, 0.0), REACH * 0.7);
        let by_hand = (v - 2.0) / 4.75 - (v * v - 4.0) / (2.0 * 4.75 * v);
        assert!((d - by_hand).abs() < 0.03, "at the wall {d:.3} s, expected (v − v_c)/BRAKE − (v² − v_c²)/(2·BRAKE·v) = {by_hand:.3} s");
    }

    // Gemini play-area recheck P2-1, the other half: `try_captures` lets a fielder within 3 m of
    // a wall take a ball up to WALL_CATCH_MAX_Z = 3.0 m (he leaps; agent-models §5.4), so the
    // plan must too. A fly coming down 1 m short of the center-field fence, with the center
    // fielder already standing on its line at the wall: the first moment he can take it is when
    // it passes 3.0 m, not 2.4 m. Bug caught: planning the wall catch with the open-field
    // ceiling (he plans for the ball lower down, 0.1–0.2 s later, than the one he can catch).
    #[test]
    fn a_fielder_plans_a_leaping_catch_at_the_wall() {
        let fx = fixture(&[]);
        let field = &fx.field;
        let fence = field.fence_at(0.0);
        let mut lo = 20.0;
        let mut hi = 90.0;
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut path = None;
        for _ in 0..40 {
            let v = 0.5 * (lo + hi);
            let dir = Vec3::new(0.0, 0.4, 0.92).normalized();
            inp.ball = BallState { r: Vec3::new(0.0, 0.5, 1.0), v: dir * v, w: Vec3::ZERO };
            let p = batted_path(&fx.env, field, inp.ball, inp.t_contact, |_, _| (0.0, 0.0));
            let (land, _) = p.at(p.first_bounce().map(|b| b.0).unwrap_or(p.t_end()));
            if land.y < fence - 1.0 { lo = v } else { hi = v }
            path = Some(p);
        }
        let path = path.unwrap();
        let (land, _) = path.at(path.first_bounce().expect("it lands").0);
        assert!((land.y - (fence - 1.0)).abs() < 0.3 && field.body_clearance(land.x, land.y, 0.0).dist < 3.0, "fixture: lands at y {:.1}, fence {fence:.1}", land.y);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        // He waits where the ball passes 2.7 m on the way down (within 3 m of the wall).
        let mut t = path.t_start();
        while path.at(t).0.z > 2.7 || path.at(t).1.z > 0.0 {
            t += 0.001;
        }
        sim.f[CF].pos = Vec3::new(0.0, path.at(t).0.y, 0.0);
        sim.f[CF].vel = Vec3::ZERO;
        // The scan samples the path every 0.12 s and the ball falls a couple of metres in that,
        // so move the phase of the samples a few ms at a time: some land between 2.4 and 3.0 m.
        let mut heights = Vec::new();
        for k in 0..24 {
            sim.t = path.t_start() + 1.0 + 0.005 * k as f64;
            let (t_int, _) = sim.intercept(CF, &path);
            heights.push(path.at(t_int).0.z);
        }
        assert!(heights.iter().all(|&z| z <= WALL_CATCH_MAX_Z + 1e-9), "planned above the leap: {heights:?}");
        assert!(heights.iter().any(|&z| z > CATCH_MAX_Z + 0.02), "never planned a catch above {CATCH_MAX_Z} m: {heights:?}");
    }

    // Gemini play-area recheck P2-1, the call: a fielder calls a ball only on the time he will
    // really have (agent-models §5.7: a call needs CALL_SLACK_S = 0.3 s to spare). Running at
    // top speed at a stand point 0.3 m from the wall he arrives 0.39 s later than the open run
    // (see above), so a ball he could take with 0.45 s to spare in the open leaves him 0.06 s
    // at the wall and he must not call it, while the same run in the open field is a call.
    // Bug caught: the call counting the unbraked run (he calls off a teammate, then is late).
    #[test]
    fn a_fielder_does_not_call_a_ball_the_wall_would_make_him_late_for() {
        let fx = fixture(&[]);
        let field = &fx.field;
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.ball = BallState { r: Vec3::new(0.0, 0.5, 1.0), v: Vec3::new(0.0, 5.0, 55.0), w: Vec3::ZERO };
        let path = batted_path(&fx.env, field, inp.ball, inp.t_contact, |_, _| (0.0, 0.0));
        let (mut lo, mut hi) = (60.0, 200.0);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if field.body_clearance(0.0, mid, BODY_R).dist > 0.3 { lo = mid } else { hi = mid }
        }
        let y_wall = 0.5 * (lo + hi);
        let tier = |y_stand: f64, x_dir: f64| {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = 3.0;
            let v = sim.f[CF].a.speed;
            sim.f[CF].pos = Vec3::new(0.0, y_stand - 40.0 * x_dir, 0.0);
            sim.f[CF].vel = Vec3::new(0.0, v * x_dir, 0.0);
            let goal = Vec3::new(0.0, y_stand + REACH * 0.7 * x_dir, 0.0);
            let (fd, start) = sim.planning_state(CF);
            let open = start + run_time_to(inp.field, &fd, goal, REACH * 0.7);
            sim.call_value(CF, &path, open + 0.45, goal, 0.0).0
        };
        // Toward the wall: no call. The same run toward home plate's side, in the open: a call.
        assert_eq!(tier(y_wall, 1.0), 0, "called a ball the wall makes him late for");
        assert!(tier(60.0, 1.0) >= 1, "fixture: the same run in the open is a call");
    }

    // Codex momentum review #3: the planner's reach time must follow the movement step, also
    // for a fielder moving across his goal. Over a grid of distances and motions (toward,
    // away, across), the closed form [`run_time_to`] is compared with the step itself
    // (a late chaser running at a fixed spot until within the planning reach).
    #[test]
    fn reach_estimate_tracks_the_movement_step() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let (mut worst_early, mut worst_late, mut sq, mut n) = (0.0f64, 0.0f64, 0.0, 0.0);
        for &d in &[2.0, 5.0, 10.0, 20.0, 40.0] {
            for &u in &[-6.0, -3.0, 0.0, 3.0, 7.0] {
                for &w in &[0.0, 2.0, 4.0, 7.0] {
                    if (u * u + w * w as f64).sqrt() > 7.9 {
                        continue;
                    }
                    let mut st = fx.st.clone();
                    let mut sim = new_sim(&inp, &mut st);
                    sim.t = 2.0;
                    // On open fair ground, clear of every wall (physics-spec §9.5 stops a
                    // fielder at a wall; the movement step is what is measured here).
                    let o = Vec3::new(-20.0, 60.0, 0.0);
                    sim.f[SS].pos = o;
                    sim.f[SS].vel = Vec3::new(u, w, 0.0);
                    sim.f[SS].target = o + Vec3::new(d, 0.0, 0.0);
                    sim.f[SS].role = Role::Chase;
                    sim.f[SS].t_int = 0.0;
                    let est = run_time_to(inp.field, &sim.f[SS], sim.f[SS].target, REACH * 0.7);
                    let mut truth = f64::NAN;
                    for k in 1..2000 {
                        sim.move_fielders();
                        sim.t += DT;
                        if (sim.f[SS].target - sim.f[SS].pos).norm() <= REACH * 0.7 {
                            truth = k as f64 * DT;
                            break;
                        }
                    }
                    assert!(truth.is_finite(), "never arrived: d {d} u {u} w {w}");
                    let e = est - truth;
                    worst_early = worst_early.min(e);
                    worst_late = worst_late.max(e);
                    sq += e * e;
                    n += 1.0;
                }
            }
        }
        let rms = (sq / n).sqrt();
        // At BRAKE = 4.75 the worst case (2 m away, moving off at 6 m/s and 4 m/s sideways) was
        // early by 0.42 s with the closed form; under `STEP_REACH_M` the estimate steps the
        // movement itself (Opus wall-braking review P3-1), and the bound is back to −0.35 s.
        eprintln!("estimate − step: worst {worst_early:+.3} / {worst_late:+.3} s, rms {rms:.3} s");
        assert!(worst_early > -0.35 && worst_late < 0.35 && rms < 0.15, "estimate − step: worst {worst_early:+.3} / {worst_late:+.3} s, rms {rms:.3} s");
    }

    // agent-models §5.10, worked by hand. Fielders' lines from home, from a fixed symmetric
    // test layout (1B 38°, 2B 13°, SS −13°, 3B −36°, LF −30°, CF 0°, RF 30° of spray). Perfect reads. A
    // right-hander, pitch down the middle: the prior is his typical pull plus the middle
    // bin's, −(1.23 + 0.6) = −1.83°, σ 20.5° (v2 swing refit, 2026-11-14). A bat read at yaw
    // 60.4412° (face −29.5588°) is a cue at 6.91 + 0.939 × (−29.5588) = −20.8457°, with σ 18.0°
    // (the scatter about the fit; no read error). k = 20.5²/(20.5² + 18²) = 0.564662, so the
    // estimate is −1.83 − 0.564662 × 19.0157 = −12.5675°, σ = 20.5 × 18 / √(20.5² + 18²) =
    // 13.5259°. Offsets against 2σ = 27.0518° (zones 20° infield, 25° outfield): SS +0.4325° →
    // +0.0160; LF +17.4325° → +0.6444; CF −12.5675° → −0.4646; 1B, 2B, 3B (23.43°) and RF
    // outside their zones; the pitcher and catcher never. A left-hander's mirror image (yaw
    // −60.4412°, prior +1.83°) gives +12.5675°: 2B −0.0160, CF +0.4646, RF −0.6444, the rest 0.
    // With no bat read, the prior alone: the shortstop, 11.17° off it, at 11.17/(2·20.5) =
    // 0.2724. (Worked by `analysis/oracle/anticipation_jumps.py`, from §5.10.)
    fn symmetric_jump_test_positions() -> [Vec3; 9] {
        let p = |d_ft: f64, deg: f64| {
            let d = d_ft * 0.3048;
            let a = deg.to_radians();
            Vec3::new(d * m::sin(a), d * m::cos(a), 0.0)
        };
        [p(60.0, 0.0), Vec3::new(0.0, -0.9, 0.0), p(105.0, 38.0), p(150.0, 13.0), p(110.0, -36.0), p(150.0, -13.0), p(290.0, -30.0), p(320.0, 0.0), p(290.0, 30.0)]
    }

    #[test]
    fn fielders_jump_toward_the_side_they_read() {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        for a in inp.fattrs.iter_mut() {
            a.antic_sigma = 0.0;
            a.swing_sigma = 0.0;
        }
        inp.pitch_x = Some(0.0);
        let home = symmetric_jump_test_positions();
        // Unit vector across fielder i's line, toward right field, from his published angle.
        let deg = [0.0, 0.0, 38.0, 13.0, -36.0, -13.0, -30.0, 0.0, 30.0];
        let toward_rf = |i: usize| {
            let phi: f64 = f64::to_radians(deg[i]);
            Vec3::new(phi.cos(), -phi.sin(), 0.0)
        };
        let jumps = |inp: &LiveInput| -> Vec<f64> { (0..9).map(|i| jump_at_contact(inp, i, home[i]).dot(toward_rf(i)) / inp.fattrs[i].speed).collect() };
        inp.bat_yaw = Some(60.4412_f64.to_radians());
        let rhb = jumps(&inp);
        let expect_r = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0160, 0.6444, -0.4646, 0.0];
        inp.bats = bb_observe::Hand::Left;
        inp.bat_yaw = Some((-60.4412_f64).to_radians());
        let lhb = jumps(&inp);
        let expect_l = [0.0, 0.0, 0.0, -0.0160, 0.0, 0.0, 0.0, 0.4646, -0.6444];
        for i in 0..9 {
            assert!((rhb[i] - expect_r[i]).abs() < 1e-3, "right-hander, fielder {i}: {:.4}", rhb[i]);
            assert!((lhb[i] - expect_l[i]).abs() < 1e-3, "left-hander, fielder {i}: {:.4}", lhb[i]);
        }
        inp.bats = bb_observe::Hand::Right;
        inp.bat_yaw = None;
        let blind = jumps(&inp);
        assert!((blind[5] - 11.17 / 41.0).abs() < 1e-3, "shortstop on the prior alone: {:.4}", blind[5]);
    }

    // Codex anticipation review #1: before his reaction time a fielder reads the bat, not the
    // ball. Two contacts with the same bat angle but different outgoing directions (the
    // review's spin probe: +4.07° and −3.24° of spray from identical bat kinematics) must give
    // every fielder the same jump. Reading the ball's launch direction, as the first version
    // did, fails this.
    #[test]
    fn the_jump_reads_the_bat_not_the_ball() {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.pitch_x = Some(-0.1);
        inp.bat_yaw = Some(80.0_f64.to_radians());
        inp.swing_lead = Some(0.165);
        // A fixed 15° read error rather than the grade-50 one (77°, agent-models §5.10), and a
        // 10 ms lead read, so the reads carry enough for someone to lean on them and the
        // comparison is not vacuous.
        for a in inp.fattrs.iter_mut() {
            a.swing_sigma = 15.0_f64.to_radians();
            a.lead_sigma = 0.010;
        }
        let home = home_positions();
        let ball = |deg: f64| BallState { r: Vec3::new(0.0, 0.5, 1.0), v: Vec3::new(f64::to_radians(deg).sin(), f64::to_radians(deg).cos(), 0.2) * 35.0, w: Vec3::ZERO };
        inp.ball = ball(4.071844);
        let a: Vec<Vec3> = (0..9).map(|i| jump_at_contact(&inp, i, home[i])).collect();
        inp.ball = ball(-3.239788);
        let b: Vec<Vec3> = (0..9).map(|i| jump_at_contact(&inp, i, home[i])).collect();
        assert!(a.iter().any(|v| v.norm() > 0.1), "someone leans on this bat read: {a:?}");
        assert_eq!(a, b, "the jump must not depend on where the ball actually went");
    }

    // agent-models §5.10, the swing's lead, worked by hand like
    // `fielders_jump_toward_the_side_they_read`. Perfect pitch read, no bat-angle read, the
    // lead read perfectly: 0.176097 s says face = 184.10° − 1213.3 × 0.176097 = −29.5585°, with
    // the fit's 3.1° of scatter; that face is a cue at −20.8454°, σ = √((0.939 × 3.1)² + 18²) =
    // 18.2348°. Against the −1.83° prior (σ 20.5°): estimate −12.4464°, σ 13.6243°, 2σ =
    // 27.2486°: SS +0.0203, LF +0.6442, CF −0.4568, the rest 0. The left-hander's mirror image
    // (the lead is the same for either side; prior +1.83°): 2B −0.0203, CF +0.4568, RF −0.6442.
    // With the bat-angle read too, at the same face and σ 10°, the two face reads fuse to σ =
    // 10 × 3.1 / √(10² + 3.1²) = 2.9610°: estimate −12.4569°, σ 13.6158°: SS +0.0199, LF
    // +0.6442, CF −0.4574. One read of the face, not two cues: had the bat read been fused as a
    // second spray cue the estimate would be −15.05°. (v2 swing refit, 2026-11-14.)
    #[test]
    fn the_swing_lead_moves_the_jump_toward_the_side_it_reads() {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        for a in inp.fattrs.iter_mut() {
            a.antic_sigma = 0.0;
            a.lead_sigma = 0.0;
            a.swing_sigma = 10.0_f64.to_radians();
        }
        inp.pitch_x = Some(0.0);
        inp.swing_lead = Some(0.176097);
        let home = symmetric_jump_test_positions();
        let deg = [0.0, 0.0, 38.0, 13.0, -36.0, -13.0, -30.0, 0.0, 30.0];
        let toward_rf = |i: usize| {
            let phi: f64 = f64::to_radians(deg[i]);
            Vec3::new(phi.cos(), -phi.sin(), 0.0)
        };
        let jumps = |inp: &LiveInput| -> Vec<f64> { (0..9).map(|i| jump_at_contact(inp, i, home[i]).dot(toward_rf(i)) / inp.fattrs[i].speed).collect() };
        let check = |got: &[f64], want: [f64; 9], what: &str| {
            for i in 0..9 {
                assert!((got[i] - want[i]).abs() < 1e-3, "{what}, fielder {i}: {:.4} (want {:.4})", got[i], want[i]);
            }
        };
        check(&jumps(&inp), [0.0, 0.0, 0.0, 0.0, 0.0, 0.0203, 0.6442, -0.4568, 0.0], "right-hander, lead alone");
        inp.bats = bb_observe::Hand::Left;
        check(&jumps(&inp), [0.0, 0.0, 0.0, -0.0203, 0.0, 0.0, 0.0, 0.4568, -0.6442], "left-hander, lead alone");
        // The belief side directly: a bat read that landed exactly on 60.4412°, believed to
        // carry 10° of error (through `jump_at_contact` the read itself would be scattered).
        let reads = JumpReads { pitch_x: 0.0, bat_yaw: Some(60.4412_f64.to_radians()), lead: Some(0.176097), bats: bb_observe::Hand::Right };
        let both: Vec<f64> = (0..9).map(|i| jump_from_reads(&reads, &inp.fattrs[i], i, home[i]).dot(toward_rf(i)) / inp.fattrs[i].speed).collect();
        check(&both, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0199, 0.6442, -0.4574, 0.0], "right-hander, lead and bat");
    }

    // A read that puts the ball behind the batter is discarded. The fielder falls back on
    // his other reads. (a) A perfect bat read at 179.999° (barrel at the catcher) with no lead
    // leaves the pitch prior alone, the shortstop at 11.17 / (2 × 20.5) = 0.2724 as in
    // `fielders_jump_toward_the_side_they_read`. (b) A wild lead read (0.40 s: face −301°)
    // beside a good bat read: the bat read alone decides (+0.0160 for the shortstop, from that
    // test). (c) The good lead read beside the bad bat read: the lead alone decides (+0.0203,
    // from `the_swing_lead_moves_the_jump_toward_the_side_it_reads`).
    #[test]
    fn an_implausible_read_is_discarded_for_the_other_reads() {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        for a in inp.fattrs.iter_mut() {
            a.antic_sigma = 0.0;
            a.swing_sigma = 0.0;
            a.lead_sigma = 0.0;
        }
        inp.pitch_x = Some(0.0);
        let home = symmetric_jump_test_positions();
        let toward_rf = Vec3::new(f64::to_radians(-13.0).cos(), -f64::to_radians(-13.0).sin(), 0.0);
        let ss = |inp: &LiveInput| jump_at_contact(inp, SS, home[SS]).dot(toward_rf) / inp.fattrs[SS].speed;
        inp.bat_yaw = None;
        let prior_only = ss(&inp);
        assert!((prior_only - 11.17 / 41.0).abs() < 1e-3, "prior alone: {prior_only:.4}");
        inp.bat_yaw = Some(179.999_f64.to_radians());
        assert_eq!(ss(&inp), prior_only, "a bat read behind the batter is discarded, not a reason to stand still");
        inp.bat_yaw = Some(60.4412_f64.to_radians());
        inp.swing_lead = Some(0.40);
        let got = ss(&inp);
        assert!((got - 0.0160).abs() < 1e-3, "the wild lead is discarded, the bat read decides: {got:.4}");
        inp.bat_yaw = Some(179.999_f64.to_radians());
        inp.swing_lead = Some(0.176097);
        let got = ss(&inp);
        assert!((got - 0.0203).abs() < 1e-3, "the bad bat read is discarded, the lead decides: {got:.4}");
    }

    // Codex anticipation review #3. (a) Fusion on the circle has no seam: fusing a prior at
    // 170° with a cue at −170° (20° apart across ±180°) lands 0.58316 × 20° = 11.663° past the
    // prior, at −178.337°, exactly the same fusion as −5° with +15° (both rotated by −175°),
    // which gives +6.663°. (b) The review's case, at the level a fielder sees: a bat at
    // +179.999° and one at −179.999° (barrel pointing at the catcher, 0.002° apart). Read
    // perfectly, both face behind the batter, are discarded, and everyone jumps on the pitch
    // prior alone (back-half policy, revised 2026-11-13). Read with 29.3° of error (the same
    // draws for both), some reads land in the forward half and move a jump off the prior's,
    // and the jumps must still agree to within what 0.002° can change (the first version's
    // linear fusion swung a third baseman from no jump to 3.6 m/s here). (c) The cue's model: a
    // right-hander's bat square to the pitch line (yaw 90°) says +6.91° (the fit's intercept);
    // a left-hander's mirror image (−90°) says −6.91°.
    #[test]
    fn anticipation_has_no_seam_behind_the_plate() {
        let r = f64::to_radians;
        let (e1, s1) = fuse_on_circle(r(170.0), r(22.0), r(-170.0), r(18.6));
        let (e2, s2) = fuse_on_circle(r(-5.0), r(22.0), r(15.0), r(18.6));
        assert!((e1.to_degrees() + 178.337).abs() < 1e-3, "across the seam: {:.4}°", e1.to_degrees());
        assert!((e2.to_degrees() - 6.663).abs() < 1e-3, "rotated: {:.4}°", e2.to_degrees());
        assert!((s1 - s2).abs() < 1e-12 && (s1.to_degrees() - 14.204).abs() < 1e-3);
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.pitch_x = Some(0.0);
        let home = home_positions();
        let mut leaned = false;
        for sigma in [0.0, r(29.296875)] {
            for a in inp.fattrs.iter_mut() {
                a.antic_sigma = 0.0;
                a.swing_sigma = sigma;
            }
            for bats in [bb_observe::Hand::Right, bb_observe::Hand::Left] {
                inp.bats = bats;
                inp.bat_yaw = None;
                let p: Vec<Vec3> = (0..9).map(|i| jump_at_contact(&inp, i, home[i])).collect();
                inp.bat_yaw = Some(r(179.999));
                let a: Vec<Vec3> = (0..9).map(|i| jump_at_contact(&inp, i, home[i])).collect();
                inp.bat_yaw = Some(r(-179.999));
                let b: Vec<Vec3> = (0..9).map(|i| jump_at_contact(&inp, i, home[i])).collect();
                for i in 0..9 {
                    if sigma == 0.0 {
                        assert!(a[i] == p[i] && b[i] == p[i], "{bats:?}, fielder {i}: {:?} {:?}, prior alone {:?}", a[i], b[i], p[i]);
                    }
                    assert!((a[i] - b[i]).norm() < 1e-3, "{bats:?}, fielder {i}: {:?} vs {:?}", a[i], b[i]);
                    leaned |= (a[i] - p[i]).norm() > 0.5;
                }
            }
        }
        assert!(leaned, "the noisy reads put someone in the forward half (the continuity case)");
        let (c_r, _) = swing_spray_cue(r(90.0), bb_observe::Hand::Right, 0.0).unwrap();
        let (c_l, _) = swing_spray_cue(r(-90.0), bb_observe::Hand::Left, 0.0).unwrap();
        assert!((c_r.to_degrees() - 6.91).abs() < 1e-9 && (c_l.to_degrees() + 6.91).abs() < 1e-9, "{c_r} {c_l}");
        assert!(swing_spray_cue(r(179.999), bb_observe::Hand::Right, 0.0).is_none());
    }

    // Codex anticipation review #2: the defense's first call is made at the quickest
    // reaction time, while a slower fielder is still carried by his jump. What he believes he
    // has to spare on a fly must not depend on whether he works it out then or at his own
    // gate, from where he actually is: nothing new has been read in between. Centre fielder,
    // 0.6 s reaction, leaning toward left field at full speed; the call is first made at the
    // infielders' 0.25 s, so planning must account for the remaining 0.35 s of his jump.
    #[test]
    fn plans_before_the_gate_count_the_rest_of_the_jump() {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.fattrs[CF].reaction = 0.6;
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.f[CF].jump = Vec3::new(-sim.f[CF].a.speed, 0.0, 0.0);
        let t_c = inp.t_contact;
        let react = inp.fattrs.iter().map(|a| a.reaction).fold(f64::INFINITY, f64::min);
        while sim.t < t_c + react {
            world_tick(&mut sim);
        }
        let pred = sim.predict(CF);
        let (ti, pt) = sim.intercept(CF, &pred);
        let early = sim.call_value(CF, &pred, ti, pt, 0.0);
        while sim.t < t_c + inp.fattrs[CF].reaction {
            world_tick(&mut sim);
        }
        assert!(sim.f[CF].vel.norm() > 2.0, "he is leaning: {:?}", sim.f[CF].vel);
        let gate = sim.call_value(CF, &pred, ti, pt, 0.0);
        // Tier 1 or 3: a fly he believes he catches in the air, sure of it or not (how sure
        // depends on where the ball is by then; the time to spare must not).
        assert!(early.0 >= 1 && gate.0 >= 1, "a fly he goes for: {early:?} {gate:?}");
        assert!((early.1 - gate.1).abs() < 1e-9, "time to spare planned early {:.3} s, at his gate {:.3} s", early.1, gate.1);
    }

    // Codex coord review #1: a reaction time off the 1/60 s tick grid. The projected jump
    // state is the one the movement step reaches at the first tick at or after the gate, so
    // its start time must be that tick, not the gate itself. Centre fielder, 0.607 s
    // reaction (gate 1.007 s, first tick at or after it 61/60 s), leaning toward left field
    // at full speed; the call is first made at the infielders' 0.25 s. Expected values come
    // from stepping the real movement up to his gate: his position, velocity and the clock
    // there. A fly he would reach at that spot 0.305 s after his gate is 0.295 s after that
    // tick, inside `CALL_SLACK_S`, so neither estimate yields a confident call.
    #[test]
    fn the_projected_jump_is_timed_by_the_tick_that_produced_it() {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.fattrs[CF].reaction = 0.607;
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.f[CF].jump = Vec3::new(-sim.f[CF].a.speed, 0.0, 0.0);
        let t_c = inp.t_contact;
        let gate = t_c + inp.fattrs[CF].reaction;
        let react = inp.fattrs.iter().map(|a| a.reaction).fold(f64::INFINITY, f64::min);
        while sim.t < t_c + react {
            world_tick(&mut sim);
        }
        let (live_pos, live_vel) = (sim.f[CF].pos, sim.f[CF].vel);
        let (proj, start) = {
            let (g, s) = sim.planning_state(CF);
            (g.into_owned(), s)
        };
        assert_eq!((sim.f[CF].pos, sim.f[CF].vel), (live_pos, live_vel), "projecting must not move him");
        let pred = sim.predict(CF);
        let ti = gate + CALL_SLACK_S + 0.005;
        assert!(pred.first_bounce().map(|b| ti < b.0).unwrap_or(true), "a fly, still in the air at {ti:.3} s");
        let pt = proj.pos;
        let early = sim.call_value(CF, &pred, ti, pt, 0.0);
        while sim.t < gate {
            world_tick(&mut sim);
        }
        assert!(sim.t > gate + 0.005, "the gate is off the tick grid: {:.6} s vs {gate:.6} s", sim.t);
        assert!(sim.f[CF].vel.norm() > 2.0, "he is leaning: {:?}", sim.f[CF].vel);
        assert_eq!(start, sim.t, "the projected state is timed by the tick that produced it");
        assert_eq!((proj.pos, proj.vel), (sim.f[CF].pos, sim.f[CF].vel), "projected vs stepped state");
        assert!(ti - sim.t < CALL_SLACK_S, "{:.6} s to spare from where he is", ti - sim.t);
        let at_gate = sim.call_value(CF, &pred, ti, pt, 0.0);
        assert_eq!(at_gate.0, 0, "not enough to spare for a confident call at his gate: {at_gate:?}");
        assert_eq!(early, at_gate, "the call made early matches the call at his gate");
    }

    // agent-models §5.10: a lean is a bet. A grounder passes 4 m to the shortstop's left; a
    // grade-80 lean (2 m/s) toward it gets him there sooner than standing, one away from it
    // later (the movement laws of §5.3, through the planner's own reach model). By hand:
    // braking 2 m/s at 4.75 m/s² takes 0.42 s and loses 0.42 m, where from rest he would have
    // covered 0.54 m, so he is about 0.9 m behind at several m/s: hundredths to tenths.
    #[test]
    fn a_lean_the_right_way_gains_ground_and_the_wrong_way_costs() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = inp.t_contact + sim.f[SS].a.reaction;
        sim.f[SS].pos = Vec3::new(4.0, 40.0, 0.0);
        let path = grounder(&fx, sim.t, 10.0);
        let mut t_at = |vx: f64| {
            sim.f[SS].vel = Vec3::new(vx, 0.0, 0.0);
            sim.intercept(SS, &path).0
        };
        let (right, none, wrong) = (t_at(-2.0), t_at(0.0), t_at(2.0));
        assert!(right < none - 0.01 && none < wrong - 0.01, "leaning toward {right:.3} s, standing {none:.3} s, leaning away {wrong:.3} s");
    }

    // agent-models §5.3: a fielder running the wrong way must stop and turn. Expected from the
    // published laws, worked by hand: braking at 4.75 m/s² from 7 m/s takes 7/4.75 s and carries
    // him 7²/9.5 = 5.16 m past his start; then 10.16 m from rest (run_time). The movement step
    // integrates the same laws tick by tick.
    #[test]
    fn running_the_wrong_way_costs_the_turn() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        let p0 = Vec3::new(5.0, 30.0, 0.0);
        sim.f[SS].pos = p0;
        sim.f[SS].vel = Vec3::new(7.0, 0.0, 0.0);
        sim.f[SS].target = p0 - Vec3::new(5.0, 0.0, 0.0);
        // A chaser already late for the ball runs through his spot (no braking for it).
        sim.f[SS].role = Role::Chase;
        sim.f[SS].t_int = 2.0;
        let a = sim.f[SS].a;
        let mut farthest: f64 = 0.0;
        let mut n = 0;
        while sim.f[SS].pos.x > 0.0 && n < 1000 {
            sim.move_fielders();
            sim.t += DT;
            farthest = farthest.max(sim.f[SS].pos.x - p0.x);
            n += 1;
        }
        let took = n as f64 * DT;
        let expect = 7.0 / 4.75 + run_time(5.0 + 49.0 / 9.5, a.speed, a.tau_a);
        assert!((took - expect).abs() < 0.05, "took {took:.3} s, expected {expect:.3} s");
        assert!((run_time_from(5.0, -7.0, a.speed, a.tau_a) - expect).abs() < 1e-6);
        assert!((farthest - 49.0 / 9.5).abs() < 0.1, "carried {farthest:.2} m");
    }

    // agent-models §5.3: a fielder at 7 m/s whose target swings 90° carries on the old way
    // while he turns: some drift, but less than a straight stop at 4.75 m/s² would take
    // (7²/9.5 = 5.16 m), and by 2 s he is running at the target near top speed.
    #[test]
    fn a_turn_carries_the_fielder_on_a_curve() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        let p0 = Vec3::new(0.0, 30.0, 0.0);
        sim.f[SS].pos = p0;
        sim.f[SS].vel = Vec3::new(7.0, 0.0, 0.0);
        // Far away, so the direction to it stays fixed while he drifts (the closed form's frame).
        sim.f[SS].target = p0 + Vec3::new(0.0, 4000.0, 0.0);
        let a_lat = sim.f[SS].a.a_lat;
        for _ in 0..120 {
            sim.move_fielders();
            sim.t += DT;
        }
        let drift = sim.f[SS].pos.x - p0.x;
        assert!(drift > 1.0 && drift < 49.0 / 9.5, "drifted {drift:.2} m (a_lat {a_lat})");
        let v = sim.f[SS].vel;
        assert!(v.x.abs() < 0.05 && v.y > 0.9 * sim.f[SS].a.speed, "velocity after 2 s {v:?}");
    }

    // Fielding review #2 and #4: a fielder who has reached his spot is standing still, and a
    // running fielder's time to a resting ball counts his speed. Codex momentum review #1: he
    // gets there by braking at 4.75 m/s², so from 7 m/s the stop takes 7/4.75 s and 49/9.5 m; a late
    // chaser runs through his spot at speed.
    #[test]
    fn stopped_fielders_have_no_speed_and_running_ones_keep_it() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        let p0 = sim.f[SS].pos;
        sim.f[SS].role = Role::Cover(2);
        sim.f[SS].vel = Vec3::new(7.0, 0.0, 0.0);
        sim.f[SS].target = p0 + Vec3::new(49.0 / 9.5, 0.0, 0.0);
        let mut stopped = None;
        for k in 1..=120 {
            sim.move_fielders();
            sim.t += DT;
            if stopped.is_none() && sim.f[SS].vel == Vec3::ZERO {
                stopped = Some(k as f64 * DT);
            }
        }
        let stopped = stopped.expect("he stops");
        assert!((stopped - 7.0 / 4.75).abs() < 0.1, "stopped after {stopped:.3} s");
        assert!((sim.f[SS].pos - sim.f[SS].target).norm() < 0.05);
        // A late chaser whose spot is 5 cm ahead keeps running.
        sim.f[SS].pos = p0;
        sim.f[SS].role = Role::Chase;
        sim.f[SS].t_int = sim.t;
        sim.f[SS].vel = Vec3::new(7.0, 0.0, 0.0);
        sim.f[SS].target = p0 + Vec3::new(0.05, 0.0, 0.0);
        sim.move_fielders();
        assert!(sim.f[SS].vel.norm() > 6.9, "a late chaser kept {:?}", sim.f[SS].vel);
        // 6 m at 6 m/s already (top speed 7.9, τ 0.7): quicker than 6 m from rest, slower
        // than 6 m at top speed, and exactly inverse to the distance law.
        let t = run_time_from(6.0, 6.0, 7.9, 0.7);
        assert!(t < run_time(6.0, 7.9, 0.7) && t > 6.0 / 7.9);
        assert!((run_dist_from(t, 6.0, 7.9, 0.7) - 6.0).abs() < 1e-9);
        assert!((run_time_from(6.0, 0.0, 7.9, 0.7) - run_time(6.0, 7.9, 0.7)).abs() < 1e-9);
    }

    /// Set up who fields the ball where, then hand out cover and backup jobs.
    /// A throw to second (receiver: the second baseman) from `from` aimed to come down to
    /// chest height at ground point `to`, released at `speed`.
    fn aimed_throw(sim: &mut Sim, from: Vec3, to: Vec3, speed: f64) {
        let d = m::hypot(to.x - from.x, to.y - from.y);
        let s = solve_throw(sim.inp.env, THROW_RELEASE_Z, THROW_CATCH_Z, d, speed).expect("in reach");
        let s0 = throw_state(from + Vec3::new(0.0, 0.0, THROW_RELEASE_Z), to - from, speed, s.elev);
        sim.launch_throw(s0, to, 2, 3, false);
    }

    /// The ball in fielder `holder`'s hands at `at`, the defense's jobs set as at assignment
    /// and then, once every man's job is known to the others, re-worked as `hear` does.
    fn held_at(sim: &mut Sim, holder: usize, at: Vec3) {
        known_chaser(sim, holder, 0);
        sim.f[holder].pos = at;
        sim.f[holder].target = at;
        sim.ball = Ball::Held { by: holder };
        sim.cover_roles(&[]);
        sim.assign_relay();
        sim.assign_backups();
        jobs_become_known(sim);
    }

    /// Every fielder's present job, shown long enough ago that his teammates know it, and
    /// what they then do about it (`Sim::hear`), until the jobs settle.
    fn jobs_become_known(sim: &mut Sim) {
        let at = sim.t - CALL_HEARD_S - DT;
        for _ in 0..4 {
            let before: Vec<Role> = sim.f.iter().map(|f| f.role).collect();
            for fd in sim.f.iter_mut() {
                let tier = if fd.role == Role::Chase { fd.call } else { 0 };
                fd.shown.push(Shown { at, role: fd.role, backs: fd.backs, tier, asks: None });
            }
            sim.hear();
            if sim.f.iter().map(|f| f.role).eq(before) {
                return;
            }
        }
        panic!("the jobs did not settle: {:?}", sim.f.iter().map(|f| f.role).collect::<Vec<_>>());
    }

    /// Point `p` lies on the segment a–b (within 1 cm of the line, between the ends).
    fn on_segment(p: Vec3, a: Vec3, b: Vec3) -> bool {
        let (ab, ap) = (b - a, p - a);
        let along = ap.dot(ab) / ab.dot(ab);
        (ap - ab * along).norm() < 0.01 && along > 0.0 && along < 1.0
    }

    // Cutoffs and relays (agent-models §5.7), from the standard assignments.
    #[test]
    fn cutoff_and_relay_men_line_up_between_the_ball_and_the_base() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        // Single to right-center with a runner on second: the first baseman is the cutoff to
        // home, short of the plate on the throw's line; the second baseman takes first and the
        // shortstop second.
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let at = Vec3::new(25.0, 62.0, 0.0);
        held_at(&mut sim, 8, at);
        assert_eq!(sim.f[2].role, Role::Relay(4));
        let spot = sim.f[2].target;
        assert!(on_segment(spot, base_pos(4), at), "cutoff spot {spot:?}");
        assert!((spot - base_pos(4)).norm() >= CUTOFF_M - 1e-9 && (spot - base_pos(4)).norm() <= 0.5 * at.norm() + 1e-9);
        assert_eq!((sim.f[3].role, sim.f[5].role, sim.f[4].role), (Role::Cover(1), Role::Cover(2), Role::Cover(3)));
        // Single to left with a runner on second: the third baseman cuts off, the shortstop
        // takes third.
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        held_at(&mut sim, 6, Vec3::new(-35.0, 60.0, 0.0));
        assert_eq!(sim.f[4].role, Role::Relay(4));
        assert!((0..9).any(|i| sim.f[i].role == Role::Cover(3)) && (0..9).any(|i| sim.f[i].role == Role::Cover(2)));
        // Deep in the left-center gap: the shortstop goes out as the relay man, far enough
        // that the outfielder's throw to him is RELAY_OF_THROW_M, or half the way home when
        // that is longer.
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let deep = Vec3::new(-30.0, 105.0, 0.0);
        held_at(&mut sim, 7, deep);
        assert_eq!(sim.f[5].role, Role::Relay(4));
        assert!(on_segment(sim.f[5].target, base_pos(4), deep));
        assert!(((sim.f[5].target - deep).norm() - RELAY_OF_THROW_M.max(0.5 * deep.norm())).abs() < 1e-6);
        // No cutoff for a ball fielded in the infield.
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        held_at(&mut sim, 5, Vec3::new(-8.0, 38.0, 0.0));
        assert!(!(0..9).any(|i| matches!(sim.f[i].role, Role::Relay(_))));
    }

    // Codex review 2026-11-12 #3: on a fly going deep the relay man goes out before it
    // lands, once his own read says he has no play on it.
    #[test]
    fn relay_man_goes_out_early_on_a_deep_fly() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 2)]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.ball = BallState { r: Vec3::new(0.0, 0.5, 1.0), v: Vec3::new(-4.0, 28.0, 22.0), w: Vec3::ZERO };
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 1.0;
        sim.assign();
        let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
        assert!(path.bounces[0].0 > sim.t + 2.0, "the ball must still be well up");
        // The relay man goes once he knows who is going for it.
        let_calls_be_heard(&mut sim);
        let fp = sim.fielding_point().expect("the centre fielder's chase is known");
        assert!(m::hypot(fp.x, fp.y) > RELAY_DEEP_M, "fielded at {fp:?}");
        let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
        assert!(path.bounces[0].0 > sim.t + 1.5, "still well up");
        // A middle infielder goes out as the relay man to home (runner on second) before it
        // lands. (Which one depends on the line from home to where the centre fielder takes
        // it, against the two men's pre-pitch spots.)
        let relays: Vec<usize> = (0..9).filter(|&i| sim.f[i].role == Role::Relay(4)).collect();
        assert!(relays == [3] || relays == [5], "relay men {relays:?}: {:?}", sim.f.iter().map(|f| f.role).collect::<Vec<_>>());
        // A short fly an infielder can catch keeps him on the ball.
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.ball = BallState { r: Vec3::new(0.0, 0.5, 1.0), v: Vec3::new(-2.0, 14.0, 24.0), w: Vec3::ZERO };
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 1.0;
        sim.assign();
        let_calls_be_heard(&mut sim);
        assert!(!(0..9).any(|i| matches!(sim.f[i].role, Role::Relay(_))));
    }

    // Codex review 2026-11-12 #1: the receiver plans from his own read of the throw, never
    // from its realized future. At the release he has not seen it yet: whatever the
    // executed speed, he is on the spot it was thrown to.
    #[test]
    fn receiver_plans_from_his_read_not_the_throws_future() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let from = Vec3::new(0.0, 100.0, 0.0);
        let second = base_pos(2);
        let d = (second - from).norm();
        let elev = solve_throw(&fx.env, THROW_RELEASE_Z, THROW_CATCH_Z, d, 37.0).unwrap().elev;
        let mut reads = Vec::new();
        for v in [35.5, 37.0, 39.0, 33.0] {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = 5.0;
            sim.f[3].pos = second;
            sim.launch_throw(throw_state(from + Vec3::new(0.0, 0.0, THROW_RELEASE_Z), second - from, v, elev), second, 2, 3, false);
            sim.update_thrown();
            assert!((sim.f[3].target - second).norm() < 1e-9, "v {v}: {:?}", sim.f[3].target);
            // After his reaction he reads it: a throw that dies short draws him in; a hot one
            // still within reach at the bag leaves him on it.
            while sim.t < 5.0 + 1.5 {
                sim.t += DT;
                sim.update_thrown();
            }
            reads.push(sim.f[3].target.y);
        }
        assert!(reads[0] > second.y + 1.0 && (reads[1] - second.y).abs() < 0.5 && reads[2] <= second.y + 0.5, "{reads:?}");
        // One that comes down well short (more than `STEP_IN_M`) he waits for at the bag and
        // takes on the hop (agent-models §5.5), rather than running in to where it was last at
        // scoop height.
        assert!((reads[3] - second.y).abs() < RECEIVE_REACH, "{reads:?}");
    }

    // Codex momentum review #4: a throw aimed at second that passes 1.6 m to the side of the
    // bag (beyond the 1.5 m receiving reach from it) is caught by a second baseman who steps
    // off; weighing his read toward the aim must not pin him to the bag. Perfect read and
    // hands, so only the plan is tested.
    #[test]
    fn a_throw_passing_beyond_reach_of_the_bag_draws_the_receiver_off_it() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 5.0;
        let second = base_pos(2);
        sim.f[3].pos = second;
        sim.f[3].target = second;
        let from = second + Vec3::new(0.0, 20.0, 0.0);
        // 1.6 m off: past the 1.35 m he can take it from the bag (`BAG_REACH_FRAC`), 0.1 m from a
        // step. (At 1.7 m the catch was 1 cm inside `RECEIVE_REACH`, and slower braking lost it.)
        let off = second + Vec3::new(1.6, 0.0, 0.0);
        let elev = solve_throw(&fx.env, THROW_RELEASE_Z, THROW_CATCH_Z, (off - from).norm(), 37.0).unwrap().elev;
        sim.launch_throw(throw_state(from + Vec3::new(0.0, 0.0, THROW_RELEASE_Z), off - from, 37.0, elev), second, 2, 3, false);
        if let Ball::Thrown { read_err, .. } = &mut sim.ball {
            *read_err = [0.0; 4];
        }
        for _ in 0..90 {
            sim.update_thrown();
            sim.try_captures();
            sim.move_fielders();
            sim.t += DT;
            if matches!(sim.ball, Ball::Held { .. }) {
                break;
            }
        }
        assert!(matches!(sim.ball, Ball::Held { by: 3 }), "receiver at {:?}", sim.f[3].pos);
    }

    // Codex momentum review #6: after a carom off the wall a fielder predicts the ball with
    // the post-deflection topspin (600 rpm), not the contact model's spin for a launch angle.
    // The review's case: from (0, 110, 1) m at (0, 25, 3) m/s the ball meets the centre-field
    // wall at 0.89 s, before its first bounce at 1.24 s.
    #[test]
    fn a_carom_off_the_wall_is_not_read_as_straight_off_the_bat() {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.ball = BallState { r: Vec3::new(0.0, 110.0, 1.0), v: Vec3::new(0.0, 25.0, 3.0), w: Vec3::ZERO };
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let wall = match &sim.ball { Ball::Free { path, .. } => path.walls.first().map(|w| w.0), _ => None }.expect("hits the wall");
        let bounce = match &sim.ball { Ball::Free { path, .. } => path.first_bounce().map(|b| b.0), _ => None }.expect("bounces");
        assert!(wall < bounce);
        sim.t = wall + 0.02;
        let pred = sim.predict(7);
        let Some(bb_physics::ground::Segment::Flight { tr, .. }) = pred.segments.first() else { panic!("no flight") };
        let rpm = tr.first().s.w.norm() * 30.0 / core::f64::consts::PI;
        assert!((rpm - 600.0).abs() < 1.0, "predicted with {rpm:.0} rpm");
    }

    // A throw home from the wall is beyond a weak arm on the fly (drag): it goes through the
    // relay man, who catches it and throws on, and the catcher takes it at the plate.
    #[test]
    fn a_throw_beyond_the_arm_goes_through_the_relay_man() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 2)]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        const B2: usize = 3;
        inp.fattrs[CF].arm = 30.0;
        inp.fattrs[CF].arm_sigma = 0.0;
        inp.fattrs[B2].arm_sigma = 0.0;
        let deep = Vec3::new(-5.0, 112.0, 0.0);
        assert!(solve_throw(&fx.env, THROW_RELEASE_Z, THROW_CATCH_Z, deep.norm(), 30.0).is_none(), "the direct throw must be out of reach");
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 5.0;
        held_at(&mut sim, CF, deep);
        // The closer deep relay man (2B against RHB for CF x = -5 m) is already on his spot;
        // the runner is past third, jogging home (not moved by this test, so the play stays open).
        sim.f[B2].pos = sim.f[B2].target;
        sim.f[1].pos = base_pos(4);
        let k = sim.r.iter().position(|r| r.id == R).unwrap();
        (sim.r[k].s, sim.r[k].goal, sim.r[k].v) = (3.2 * BASE_L, 4, 3.0);
        for _ in 0..600 {
            sim.update_relay();
            sim.update_thrown();
            sim.try_captures();
            sim.holder_act();
            sim.move_fielders();
            sim.t += DT;
            if matches!(sim.ball, Ball::Held { by: 1 }) {
                break;
            }
        }
        let who: Vec<AgentId> = sim.out.released.iter().map(|x| x.1).collect();
        assert_eq!(who, vec![sim.f[CF].id, sim.f[B2].id], "throws by {who:?}; facts {:?}; catcher at {:?}", sim.out.facts, sim.f[1].pos);
        assert!(matches!(sim.ball, Ball::Held { by: 1 }), "ball: {:?}", match &sim.ball { Ball::Held { by } => format!("held by {by}"), _ => "loose".into() });
    }

    /// agent-models §5.5, the route (owner's Q32): with a runner racing home and a cutoff man
    /// lined up, a grade-50 centre fielder throws to the relay man from deep (300 ft and more)
    /// and low through the cutoff man from shallower (240 ft and less). The owner put the line at
    /// "about 280 ft"; this checks the route against that, either side of it.
    #[test]
    fn the_route_is_the_relay_from_deep_and_low_through_the_cutoff_from_shallow() {
        const R: AgentId = 0x0007;
        for (ft, relay) in [(230.0, false), (240.0, false), (300.0, true), (320.0, true)] {
            let fx = fixture(&[(R, 3)]);
            let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = 5.0;
            let at = Vec3::new(0.0, ft * 0.3048, 0.0);
            let k = sim.r.iter().position(|r| r.id == R).unwrap();
            (sim.r[k].s, sim.r[k].goal, sim.r[k].v) = (3.0 * BASE_L + 3.0, 4, 7.0);
            held_at(&mut sim, CF, at);
            let cut = (0..9).find(|&i| matches!(sim.f[i].role, Role::Relay(4))).expect("a cutoff man");
            sim.f[cut].pos = sim.f[cut].target;
            jobs_become_known(&mut sim);
            let race = sim.throw_race(CF, 4);
            assert_eq!(race.through, relay, "{ft} ft: direct {:.2} s, through the relay man {:.2} s", race.direct, race.via);
        }
    }

    /// agent-models §5.5: a throw straight home with a cutoff man lined up passes him no higher
    /// than his head (`CUT_HEIGHT_M`), so that he could cut it, where the plain aim from 240 ft
    /// would pass him over 5 m up (catch-throw note §4).
    #[test]
    fn a_throw_through_the_cutoff_man_passes_him_low() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 3)]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.fattrs[CF].arm_sigma = 0.0;
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 5.0;
        let at = Vec3::new(0.0, 240.0 * 0.3048, 0.0);
        let k = sim.r.iter().position(|r| r.id == R).unwrap();
        (sim.r[k].s, sim.r[k].goal, sim.r[k].v) = (3.0 * BASE_L + 3.0, 4, 7.0);
        held_at(&mut sim, CF, at);
        let cut = (0..9).find(|&i| matches!(sim.f[i].role, Role::Relay(4))).expect("a cutoff man");
        sim.f[cut].pos = sim.f[cut].target;
        jobs_become_known(&mut sim);
        let spot = sim.f[cut].pos;
        for _ in 0..120 {
            sim.holder_act();
            if !sim.out.released.is_empty() {
                break;
            }
            sim.t += DT;
        }
        let Ball::Thrown { path, relay: false, to_base: 4, .. } = &sim.ball else { panic!("no direct throw home: {:?}", sim.out.released) };
        let out = (spot - at).norm();
        let h = path.knots.iter().find(|k| (k.s.r - at).norm() >= out).map(|k| k.s.r.z).expect("it reaches him");
        assert!(h <= CUT_HEIGHT_M + 0.05, "passes the cutoff man {h:.2} m up");
        let plain = solve_throw(&fx.env, THROW_RELEASE_Z, THROW_CATCH_Z, at.norm(), path.first().s.v.norm()).unwrap().elev;
        let tr = bb_physics::throw::fly_throw(&fx.env, throw_state(Vec3::new(0.0, 0.0, THROW_RELEASE_Z), Vec3::Y, path.first().s.v.norm(), plain), 0.0, 6.0);
        let h_plain = tr.knots.iter().find(|k| k.s.r.y >= out).map(|k| k.s.r.z).unwrap();
        assert!(h_plain > 4.0, "the plain aim would pass him at {h_plain:.2} m");
    }

    /// How long the movement law takes to stop a body moving at `v` when it brakes as hard as
    /// it can (`step_toward` toward the point where braking stops it), s.
    fn law_stop_s(v: f64) -> f64 {
        let a = FielderAttrs::average(5);
        let (mut pos, mut vel) = (Vec3::ZERO, Vec3::new(v, 0.0, 0.0));
        let target = Vec3::new(v * v / (2.0 * BRAKE), 0.0, 0.0);
        let mut t = 0.0;
        while vel.norm() > 1e-9 && t < 5.0 {
            (pos, vel) = step_toward(pos, vel, target, &a, true, 0.0);
            t += DT;
        }
        t
    }

    /// Opus catch-throw review P3-3(a): the cuttable-throw belief tables against the world's
    /// throw, the same physics flown along the real line in the reference park: home from
    /// across the outfield (spray −40° to +40°) and third from left and centre, through a
    /// cutoff man about 45 ft out; and home from deep through the relay man's spot.
    #[test]
    fn the_cut_tables_match_the_worlds_cuttable_throw() {
        let (env, field) = (FlightEnv::default_env(), Field::default());
        let world = |from: Vec3, to: Vec3, short: f64, v: f64| {
            let d = m::hypot(to.x - from.x, to.y - from.y);
            bb_physics::throw::cut_throw_time_along(&env, &field, from, to, THROW_RELEASE_Z, THROW_CATCH_Z, CUT_HEIGHT_M, d - short, v)
        };
        for deg in [-40.0f64, -20.0, 0.0, 20.0, 40.0] {
            for ft in [230.0, 260.0, 290.0] {
                for v in [34.0, 37.0, 40.0] {
                    let r = ft * 0.3048;
                    let from = Vec3::new(r * deg.to_radians().sin(), r * deg.to_radians().cos(), 0.0);
                    let d_home = from.norm();
                    let (w, b) = (world(from, Vec3::ZERO, through_spot(d_home), v), direct_time(v, d_home, Cut::Through));
                    assert!(w.is_finite() == b.is_finite() && (!w.is_finite() || (w - b).abs() < 0.06), "home from {deg}° {ft} ft at {v}: world {w:.3} s, belief {b:.3} s");
                    if deg <= 0.0 {
                        let third = base_pos(3);
                        let d3 = (third - from).norm();
                        let (w, b) = (world(from, third, through_spot(d3), v), direct_time(v, d3, Cut::Through));
                        assert!(!w.is_finite() || !b.is_finite() || (w - b).abs() < 0.12, "third from {deg}° {ft} ft at {v}: world {w:.3} s, belief {b:.3} s");
                    }
                }
            }
        }
        for (ft, v) in [(300.0, 37.0), (320.0, 40.0)] {
            let from = Vec3::new(0.0, ft * 0.3048, 0.0);
            let d = from.norm();
            let (w, b) = (world(from, Vec3::ZERO, cut_spot(d), v), direct_time(v, d, Cut::Relay));
            assert!((w - b).abs() < 0.06, "home from {ft} ft past the relay man at {v}: world {w:.3} s, belief {b:.3} s");
        }
    }

    /// Opus catch-throw review P3-3(b), P1-1: a runner believes a cuttable throw only where the
    /// world would make one. With a runner on second the relay job lines a man up toward home,
    /// not third: from the centre fielder's spot a runner reads a throw home through him and a
    /// plain throw to third, exactly as the centre fielder will throw them; and before the man
    /// is on the line (still at his base), neither.
    #[test]
    fn a_runners_cut_belief_follows_the_throw_the_world_would_make() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 5.0;
        let at = Vec3::new(10.0, 72.0, 0.0);
        held_at(&mut sim, CF, at);
        let cut = (0..9).find(|&i| matches!(sim.f[i].role, Role::Relay(4))).expect("a cutoff man toward home");
        // Not yet out on the line.
        let mut early = sim.f[cut].pos;
        early.x += 6.0;
        sim.f[cut].pos = early;
        for b in [3u8, 4] {
            assert_eq!((sim.cut_seen(None, at, b), sim.cutoff_out(CF, b).is_some()), (Cut::No, false), "base {b}, man not on the line");
        }
        sim.f[cut].pos = sim.f[cut].target;
        jobs_become_known(&mut sim);
        assert!(((sim.f[cut].pos - base_pos(4)).norm() - through_spot(at.norm())).abs() < 0.5, "the cutoff man stands about 45 ft out: {:?}", sim.f[cut].pos);
        assert_eq!((sim.cut_seen(None, at, 4), sim.cutoff_out(CF, 4).is_some()), (Cut::Through, true));
        assert_eq!((sim.cut_seen(None, at, 3), sim.cutoff_out(CF, 3).is_some()), (Cut::No, false));
    }

    /// Opus catch-throw review P3-3(c), P3-1: a runner's read of the low throw through the
    /// cutoff man names the base it is going to, home, and times it there, on the hop, as the
    /// same physics flies it (not where it comes down through catching height, 10 m short); a
    /// plain throw to second he reads as going to second. (The rolling throw past a deep relay
    /// man 45 m out comes down through catching height over second base, the same in the air
    /// as a throw to second; the holder takes the relay there instead, note §4.)
    #[test]
    fn a_runner_reads_a_low_throw_to_the_base_it_is_going_to() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 5.0;
        let from = Vec3::new(0.0, 88.0, THROW_RELEASE_Z);
        let v = 37.0;
        let short = through_spot(88.0);
        let el = cut_elev(&fx.env, THROW_RELEASE_Z, THROW_CATCH_Z, CUT_HEIGHT_M, 88.0, 88.0 - short, v).unwrap();
        let s0 = throw_state(from, Vec3::new(0.0, -1.0, 0.0), v, el);
        let read = sim.project_runner_throw(s0.r, s0.v);
        let world = bb_physics::throw::cut_throw_time_along(&fx.env, &fx.field, Vec3::new(0.0, 88.0, 0.0), Vec3::ZERO, THROW_RELEASE_Z, THROW_CATCH_Z, CUT_HEIGHT_M, 88.0 - short, v);
        assert_eq!(read.dest, 4, "read as going to {}", read.dest);
        // He times it at the plate (within his catching reach of it), not 10 m short.
        assert!(read.t_arrive - sim.t > world - 0.2 && read.t_arrive - sim.t <= world + 0.02, "read {:.2} s, the world {world:.2} s", read.t_arrive - sim.t);
        let to2 = base_pos(2) - Vec3::new(0.0, 88.0, 0.0);
        let el = solve_throw(&fx.env, THROW_RELEASE_Z, THROW_CATCH_Z, to2.norm(), v).unwrap().elev;
        let s0 = throw_state(from, to2, v, el);
        assert_eq!(sim.project_runner_throw(s0.r, s0.v).dest, 2);
    }

    /// agent-models §5.5, the relay man's exchange: on the throw home from the wall above, with
    /// the runner racing from third, the cutoff man holds the ball for the sourced relay time,
    /// 0.75 s at grade 50 (an analyst's figure for a big-league shortstop; 0.62 s is the one
    /// measured elite relay), within a tick, and lets it go set (no hurry). Before, he could let
    /// it go at 0.6 × 0.7 × 0.65 = 0.27 s, quicker than any relay on record.
    #[test]
    fn the_relay_man_holds_the_ball_for_the_sourced_exchange() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 3)]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        const B2: usize = 3;
        inp.fattrs[CF].arm = 30.0;
        inp.fattrs[CF].arm_sigma = 0.0;
        inp.fattrs[B2].arm_sigma = 0.0;
        let deep = Vec3::new(-5.0, 112.0, 0.0);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 5.0;
        held_at(&mut sim, CF, deep);
        sim.f[B2].pos = sim.f[B2].target;
        sim.f[1].pos = base_pos(4);
        // The runner left third on the catch and is a third of the way home at full speed: a
        // race the relay man reads as close.
        let k = sim.r.iter().position(|r| r.id == R).unwrap();
        (sim.r[k].s, sim.r[k].goal, sim.r[k].v) = (3.0 * BASE_L + 6.0, 4, 8.0);
        for _ in 0..600 {
            sim.update_relay();
            sim.update_thrown();
            sim.try_captures();
            sim.holder_act();
            sim.move_fielders();
            sim.t += DT;
            if sim.out.released.len() >= 2 {
                break;
            }
        }
        let who: Vec<AgentId> = sim.out.released.iter().map(|x| x.1).collect();
        assert_eq!(who, vec![sim.f[CF].id, sim.f[B2].id], "throws by {who:?}; facts {:?}", sim.out.facts);
        let caught = sim.out.secured.iter().find(|x| x.1 == sim.f[B2].id).unwrap().0;
        let hold = sim.out.released[1].0 - caught;
        // The wiring: the world holds for the model's own value.
        assert!((hold - throwing::RELAY_S_50).abs() <= DT + 1e-9, "relay man held it {hold:.3} s, the model's relay_s is {:.3}", throwing::RELAY_S_50);
        // The sourced check: no quicker than the one measured elite relay exchange (Tatis Jr.,
        // 0.62 s). The old code let it go at 0.27–0.455 s.
        assert!(hold >= 0.62, "sourced check: quicker than the quickest relay on record (0.62 s): {hold:.3} s");
        assert_eq!(sim.out.hurry[1].0, 0.0, "a relay is let go set");
    }

    /// The holder predicts the relay the world makes (the twin of
    /// `the_holder_predicts_the_pivot_the_world_makes`; Opus relay review P3-4): on the throw
    /// home from the wall through the cutoff man, the relay man's hold in the holder's route
    /// estimate (`throw_race`) is the one the world then makes, to a tick, and the ball reaches
    /// the catcher within 0.15 s of the estimate (the reference atmosphere's flight table
    /// against the world's flight, two legs). Both sides share `relay_s`, so this guards against
    /// drift, not the value; the value's sourced check is in
    /// `the_relay_man_holds_the_ball_for_the_sourced_exchange`.
    #[test]
    fn the_holder_predicts_the_relay_the_world_makes() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 3)]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        const B2: usize = 3;
        inp.fattrs[CF].arm = 30.0;
        inp.fattrs[CF].arm_sigma = 0.0;
        inp.fattrs[B2].arm_sigma = 0.0;
        let deep = Vec3::new(-5.0, 112.0, 0.0);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 5.0;
        held_at(&mut sim, CF, deep);
        sim.f[B2].pos = sim.f[B2].target;
        sim.f[1].pos = base_pos(4);
        let k = sim.r.iter().position(|r| r.id == R).unwrap();
        (sim.r[k].s, sim.r[k].goal, sim.r[k].v) = (3.0 * BASE_L + 6.0, 4, 8.0);
        let (mut predicted, mut caught) = (None, f64::NAN);
        for _ in 0..600 {
            if matches!(sim.ball, Ball::Held { by: CF }) {
                let race = sim.throw_race(CF, 4);
                predicted = race.through.then(|| (race.via, throwing::relay_s(sim.f[B2].a.transfer)));
            }
            let before = matches!(sim.ball, Ball::Held { by: B2 });
            sim.update_relay();
            sim.update_thrown();
            sim.try_captures();
            if !before && matches!(sim.ball, Ball::Held { by: B2 }) {
                caught = sim.t;
            }
            sim.holder_act();
            sim.move_fielders();
            sim.t += DT;
            if matches!(sim.ball, Ball::Held { by: 1 }) {
                let (at_home, hold) = predicted.expect("the holder chose the relay route");
                let released = sim.out.released[1].0;
                assert!((released - caught - hold).abs() <= DT + 1e-9, "relay man held it {:.3} s, the holder expected {hold:.3}", released - caught);
                let arrived = sim.out.secured.last().unwrap().0;
                assert!((arrived - at_home).abs() <= 0.15, "the catcher has it at {arrived:.3}, the holder expected {at_home:.3}");
                return;
            }
        }
        panic!("the relay never reached the catcher: {:?}", sim.out.facts);
    }

    /// A runner reads a relay only from what he can see (Opus relay review P3-1): a third
    /// baseman who takes a throw 3 m off his bag is a base receiver (0.7 of grade-50 `Transfer`
    /// and his turn); a man who showed the cutoff job, or takes it far from every bag, is a
    /// relay man (`relay_s`, turn included). The throw's private route plays no part.
    #[test]
    fn a_runner_reads_a_relay_only_from_what_he_can_see() {
        const R: AgentId = 7;
        let fx = fixture(&[(R, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU }, RunnerIn { id: R, base: 2, speed: 8.2, tau_s: RUNNER_TAU }]);
        // The runner's read of the holder's release for a throw home, after the catch.
        let read = |by: usize, at: Vec3, to_base: u8, relay: bool, shown: Option<Role>| {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = 5.0;
            if let Some(role) = shown {
                sim.f[by].shown.push(Shown { at: 4.0, role, backs: None, tier: 0, asks: None });
            }
            (sim.f[by].pos, sim.f[by].vel) = (at, Vec3::ZERO);
            sim.launch_throw(BallState { r: at + Vec3::Z * THROW_CATCH_Z, v: Vec3::new(0.0, -20.0, 0.0), w: Vec3::ZERO }, base_pos(to_base), to_base, by, relay);
            sim.try_captures();
            assert!(matches!(sim.ball, Ball::Held { by: b } if b == by), "{:?}", sim.ball);
            let look = sim.observe_holder(by);
            look.release_time(sim.t, 4, FielderAttrs::average(by), sim.f[by].a.throws) - look.secured.unwrap()
        };
        let relay = throwing::relay_s(FielderAttrs::average(5).transfer);
        // A third baseman 3 m off third, taking a throw to third: a feed, not a relay.
        let off_third = read(4, base_pos(3) + Vec3::new(0.0, 3.0, 0.0), 3, false, None);
        assert!(off_third < relay - 0.1, "receiver 3 m off third read as {off_third:.3} s (a relay is {relay:.3})");
        assert!(off_third >= RELAY_TRANSFER_FRAC * FielderAttrs::average(4).transfer - 1e-9, "{off_third:.3}");
        // The shortstop out on the relay spot, shown as the cutoff man: a relay. The same man
        // there with no job shown: a relay by distance. Either way the throw's private route
        // is not read.
        let spot = Vec3::new(-12.0, 55.0, 0.0);
        assert!((1..=4).all(|b| (spot - base_pos(b)).norm() > RELAY_SEEN_M));
        let shown = read(5, spot, 4, true, Some(Role::Relay(4)));
        assert!((shown - relay).abs() < 1e-9, "cutoff man read as {shown:.3} s, relay {relay:.3}");
        assert_eq!(read(5, spot, 4, false, None), shown);
        // Shown as the cutoff man but taking it near a bag (5 m off second): still a relay.
        let near_second = read(5, base_pos(2) + Vec3::new(-3.0, 4.0, 0.0), 4, true, Some(Role::Relay(4)));
        assert!((near_second - relay).abs() < 1e-9, "{near_second:.3}");
        assert!(read(5, base_pos(2) + Vec3::new(-3.0, 4.0, 0.0), 2, false, None) < relay - 0.1);
    }

    /// Seed 7475 play 26 (2026-11-13): runners on first and third, a chopper to the
    /// pitcher. Where does he throw with `outs` out?
    fn pitcher_throws_to(outs: u8) -> u8 {
        const R1: AgentId = 0x0007;
        const R3: AgentId = 0x0008;
        let fx = fixture(&[(R1, 1), (R3, 3)]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R1, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R3, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.fattrs[0].arm_sigma = 0.0;
        let mut st = fx.st.clone();
        st.outs = outs;
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 3.25;
        held_at(&mut sim, 0, Vec3::new(0.9, 18.2, 0.0));
        sim.f[1].pos = base_pos(4);
        sim.f[2].pos = base_pos(1);
        // As the play stood: the batter 12.5 m from first, the runner from first 5.3 m from
        // second, the runner from third 8 m from home, all at 7 m/s. Each is a sure out.
        for (id, s, goal) in [(fx.st.batter, 14.9, 1), (R1, 49.6, 2), (R3, 101.6, 4)] {
            let k = sim.r.iter().position(|r| r.id == id).unwrap();
            (sim.r[k].s, sim.r[k].goal, sim.r[k].v) = (s, goal, 7.0);
        }
        sim.holder_act();
        match sim.ball {
            Ball::Thrown { to_base, .. } => to_base,
            _ => panic!("no throw; facts {:?}", sim.out.facts),
        }
    }

    // With two out every out ends the inning: the pitcher takes the force at first, not the
    // tag play at home (the owner: "a real player would have thrown to first").
    #[test]
    fn with_two_out_the_holder_takes_the_force_over_the_lead_runner() {
        assert_eq!(pitcher_throws_to(2), 1);
    }

    // Codex dive review #1: with two out, a runner who reached second and is coming back to it
    // must be tagged there; short of it he can still be forced.
    #[test]
    fn a_runner_back_to_a_base_he_reached_is_a_tag_play() {
        const R1: AgentId = 0x0007;
        let fx = fixture(&[(R1, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R1, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        st.outs = 2;
        let mut sim = new_sim(&inp, &mut st);
        let k = sim.r.iter().position(|r| r.id == R1).unwrap();
        (sim.r[k].s, sim.r[k].goal) = (2.0 * BASE_L - 4.0, 2);
        assert_eq!(sim.out_weight(k, 2), 1.0, "short of second, forced");
        (sim.r[k].s, sim.r[k].goal) = (2.0 * BASE_L + 4.0, 2);
        assert_eq!(sim.out_weight(k, 2), TWO_OUT_TAG_W, "back to second, a tag");
    }

    // With fewer than two out, cutting down the run at home is still the play.
    #[test]
    fn with_one_out_the_holder_cuts_down_the_lead_runner() {
        assert_eq!(pitcher_throws_to(1), 4);
    }

    fn jobs(sim: &mut Sim, fielder: usize, at: Vec3) {
        known_chaser(sim, fielder, 0);
        (sim.f[fielder].pos, sim.f[fielder].target) = (at, at);
        sim.ball = Ball::Held { by: fielder };
        sim.cover_roles(&[]);
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
        let batter = |fx: &Fixture| RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S };
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
        // ... and the pitcher backs up third, in foul ground (owner, 2026-11-13).
        assert!(behind(&sim, 0, 3, at), "P: {:?} {:?} {:?}", sim.f[0].role, sim.f[0].backs, sim.f[0].target);
        assert!(!Field::fair(sim.f[0].target.x, sim.f[0].target.y), "P: {:?}", sim.f[0].target);

        // Fly to right-center, nobody on (seed 7 play 1 of the pitcher-outfield diagnosis):
        // the right fielder takes it, the center fielder backs him up, the left fielder backs
        // up second, the catcher first, and the pitcher third (owner, 2026-11-13). His spot,
        // worked by hand: third base at 27.43/√2 = 19.40 m along each axis, the throw from
        // (62, 88) runs along (−81.4, −68.6)/106.5, so 13 m past the bag is (−29.3, 11.0),
        // in foul ground beyond third, about 30 m from the mound.
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let rc = Vec3::new(62.0, 88.0, 0.0);
        jobs(&mut sim, 8, rc);
        assert!(behind(&sim, 0, 3, rc), "P: {:?} {:?} {:?}", sim.f[0].role, sim.f[0].backs, sim.f[0].target);
        let want = Vec3::new(-29.3, 11.0, 0.0);
        assert!((Vec3::new(sim.f[0].target.x, sim.f[0].target.y, 0.0) - want).norm() < 0.3, "P: {:?}", sim.f[0].target);
        assert!(sim.f[7].role == Role::Backup && sim.f[7].backs.is_none(), "CF: {:?} {:?}", sim.f[7].role, sim.f[7].backs);
        assert!(behind(&sim, 6, 2, rc), "LF: {:?} {:?}", sim.f[6].backs, sim.f[6].target);
        assert!(sim.f[1].backs == Some(1) && sim.f[1].role == Role::Backup);
        for base in 1..=4u8 {
            assert!(sim.f.iter().filter(|f| f.backs == Some(base)).count() <= 1);
        }

        // Single to left, runner on first: the pitcher backs up third from foul ground, the
        // right fielder still backs up second, and the catcher stays home.
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 1)]);
        let inp = input(&fx, vec![batter(&fx), RunnerIn { id: R, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
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

    // A fielder left with no job does not chase the ball (pitcher-outfield diagnosis,
    // 2026-11-12). `assign_among` writes every candidate's own intercept into his target
    // before the cover and backup steps; on a fly to right-center with nobody on, the
    // second baseman gets no job and must pull up where braking stops him instead of
    // running out toward the ball. (The pitcher was idle here too until he was given third
    // to back up, owner 2026-11-13; `idle_fielders_back_up_the_play` checks that.)
    #[test]
    fn idle_fielders_do_not_chase_the_ball() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let at = Vec3::new(62.0, 88.0, 0.0);
        // The second baseman is already moving toward the ball at 7 m/s: he stops within
        // v²/(2·4.75 m/s²) ≈ 5.2 m of where he is, on his line of motion.
        const B2: usize = 3;
        let toward = (at - sim.f[B2].pos).normalized();
        sim.f[B2].vel = toward * 7.0;
        for fd in sim.f.iter_mut() {
            fd.target = at;
        }
        known_chaser(&mut sim, 8, 0);
        (sim.f[8].pos, sim.f[8].target) = (at, at);
        sim.ball = Ball::Held { by: 8 };
        sim.cover_roles(&[]);
        sim.assign_backups();
        let idle: Vec<usize> = (0..9).filter(|&i| sim.f[i].role == Role::Idle).collect();
        assert!(idle.contains(&B2) && !idle.contains(&0), "idle: {idle:?}");
        for &i in &idle {
            let fd = &sim.f[i];
            let off = Vec3::new(fd.target.x - fd.pos.x, fd.target.y - fd.pos.y, 0.0);
            assert!(off.norm() < 49.0 / 9.5 + 0.1, "f{i} idle at {:?} aims at {:?}", fd.pos, fd.target);
            if fd.vel.norm() > 0.0 {
                assert!(off.normalized().dot(fd.vel.normalized()) > 0.99, "f{i} brakes along his motion");
            }
        }
        // Stepping on, he comes to rest near where he started instead of reaching the ball.
        let start = sim.f[B2].pos;
        sim.t = sim.inp.t_contact + 2.0;
        for _ in 0..300 {
            sim.move_fielders();
            sim.t += DT;
        }
        assert!(sim.f[B2].vel.norm() < 1e-9 && (sim.f[B2].pos - start).norm() < 49.0 / 9.5 + 0.1, "2B at {:?} moving {:?}", sim.f[B2].pos, sim.f[B2].vel);
    }

    /// A ball hit from the plate with velocity `v`, every fielder reading it without error
    /// (the tests below are about the call rule, not the draws), at the outfielders'
    /// reaction time.
    fn hit(fx: &Fixture, v: Vec3) -> LiveInput<'_> {
        let mut inp = input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        // A ball that carries as the fielders expect, so their reads match it.
        inp.ball = typical_ball(&fx.env, Vec3::new(0.0, 0.5, 1.0), v);
        inp
    }

    /// A ball off the bat at `r` going `v` that carries as far as the typical spin says
    /// (`believed_flight` at launch): pure backspin, solved for that carry in open air, so a
    /// read with no error lands where it does and it does not curve sideways. Where the belief
    /// allows no unseen spin, its own spin.
    fn typical_ball(env: &FlightEnv, r: Vec3, v: Vec3) -> BallState {
        let (benv, w) = believed_flight(env, v, Some(0.0));
        if benv.unseen_spin == 0.0 {
            return BallState { r, v, w };
        }
        let target = open_air_carry(&benv, BallState { r, v, w });
        let axis = w.normalized();
        let carry = |b: f64| open_air_carry(env, BallState { r, v, w: axis * b });
        // Within half the believed backspin either way; carry may rise or (a steep pop-up
        // drifting back) fall with backspin, so bisect on whichever side the target is.
        let (mut lo, mut hi) = (0.5 * w.norm(), 1.5 * w.norm());
        let rising = carry(hi) > carry(lo);
        for _ in 0..40 {
            let mid = 0.5 * (lo + hi);
            if (carry(mid) < target) == rising { lo = mid } else { hi = mid }
        }
        BallState { r, v, w: axis * (0.5 * (lo + hi)) }
    }

    fn at_first_read(sim: &mut Sim) {
        for fd in sim.f.iter_mut() {
            fd.err = [0.0; 4];
        }
        sim.t = sim.inp.t_contact + 0.35;
    }

    fn tier(sim: &Sim, i: usize) -> u8 {
        let p = sim.predict(i);
        let (ti, pt) = sim.intercept(i, &p);
        sim.call_value(i, &p, ti, pt, 0.0).0
    }

    fn landing(sim: &Sim) -> Vec3 {
        let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
        Vec3::new(path.bounces[0].1, path.bounces[0].2, 0.0)
    }


    // Agent-models §5.7, calls: a fielder calls a ball in the air only when his time to spare
    // also covers his read's depth error. A pop-up to short right-center, about 70 m out: the
    // right fielder, some 85 m from the ball at his first look, reads its depth to about
    // 3.0 × 85/30 × 3 ≈ 25 m, three seconds of running, more than anyone has to spare on a
    // six-second ball he must run 25 m for; the second baseman, under 40 m from it, reads it
    // to about 0.3 × 40/30 × 3 ≈ 1.2 m. So the second baseman calls it and the right fielder
    // does not (the old rule gave it to the outfielder: Codex idle-fielder review #2).
    #[test]
    fn an_outfielder_does_not_call_a_pop_up_he_cannot_be_sure_of() {
        const B2: usize = 3;
        const RF: usize = 8;
        let fx = fixture(&[]);
        let inp = hit(&fx, Vec3::new(6.62, 20.81, 25.48));
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        at_first_read(&mut sim);
        let land = landing(&sim);
        assert!((land - Vec3::new(21.0, 67.0, 0.0)).norm() < 3.0, "lands at {land:?}");
        assert_eq!((tier(&sim, B2), tier(&sim, RF)), (2, 1));
        sim.assign();
        assert_eq!(sim.f[B2].role, Role::Chase);
        // Until he hears the call, the right fielder goes for it too.
        let_calls_be_heard(&mut sim);
        assert_eq!(sim.f[B2].role, Role::Chase);
        assert_ne!(sim.f[RF].role, Role::Chase, "the infielder has called it");
        // A fly hit at the centre fielder he has only to wait for: he calls it at once.
        let inp = hit(&fx, Vec3::new(0.0, 31.1, 22.0));
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        at_first_read(&mut sim);
        let land = landing(&sim);
        assert!((land - sim.f[CF].pos).norm() < 4.0, "lands at {land:?}");
        assert_eq!(tier(&sim, CF), 3);
        sim.assign();
        assert_eq!(sim.f[CF].role, Role::Chase);
        assert_eq!((0..9).filter(|&i| sim.f[i].role == Role::Chase).count(), 1);
    }

    // A fly into the right-center gap that the centre and right fielders both believe they
    // can run down, neither surely at first look: both go for it. A second and a half on, the
    // centre fielder's read has firmed up enough to call it, and the right fielder peels off.
    #[test]
    fn until_someone_is_sure_everyone_who_can_catch_it_goes() {
        const RF: usize = 8;
        let fx = fixture(&[]);
        let inp = hit(&fx, Vec3::new(8.89, 33.58, 25.28));
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        at_first_read(&mut sim);
        let land = landing(&sim);
        assert!((land - Vec3::new(27.5, 104.5, 0.0)).norm() < 3.0, "lands at {land:?}");
        assert_eq!((tier(&sim, CF), tier(&sim, RF)), (1, 1), "neither is sure yet");
        sim.assign();
        for i in [CF, RF] {
            assert_eq!(sim.f[i].role, Role::Chase, "f{i}");
            assert!((sim.f[i].target - land).norm() < 6.0, "f{i} runs at {:?}", sim.f[i].target);
        }
        assert_eq!((0..9).filter(|&i| sim.f[i].role == Role::Chase).count(), 2, "nobody else can get there");
        for _ in 0..90 {
            sim.move_fielders();
            sim.t += DT;
        }
        sim.recall();
        assert_eq!(tier(&sim, CF), 3);
        assert_eq!(sim.f[RF].role, Role::Chase, "he has not heard the call yet");
        let_calls_be_heard(&mut sim);
        assert_eq!(sim.f[CF].role, Role::Chase);
        assert_ne!(sim.f[RF].role, Role::Chase, "called off");
    }

    // Agent-models §5.1, the plate cue: flying a ball's true velocity and spin back to contact
    // lands on the contact point (the world's flight, run forward with its own step, is the
    // independent reference), for a liner, a fly and a pop-up, at a time between steps too.
    // With the typical spin in place of the true one it misses by about the spin floor. The
    // balls carry as the typical spin says (`typical_ball`), and are looked at only while they
    // are in the air.
    #[test]
    fn the_plate_cue_flies_the_ball_back_to_contact() {
        let env = FlightEnv::default_env();
        let p0 = Vec3::new(0.0, 0.5, 1.0);
        for v in [Vec3::new(-8.0, 40.0, 7.0), Vec3::new(9.0, 33.0, 22.0), Vec3::new(4.0, 14.0, 30.0)] {
            let mut st = typical_ball(&env, p0, v);
            let mut t = 0.0f64;
            for s in [0.4f64, 1.0, 1.85, 3.0] {
                while t < s - 1e-9 {
                    let h = (s - t).min(0.01);
                    st = env.step(&st, h);
                    t += h;
                }
                if st.r.z <= env.ball.radius {
                    break;
                }
                let back = position_from_plate(&env, p0, st.v, st.w, s);
                assert!((back - st.r).norm() < 1e-3, "v {v:?} at {s} s: {back:?} against {:?}", st.r);
                let (benv, w) = believed_flight(&env, st.v, Some(s));
                let typical = position_from_plate(&benv, p0, st.v, w, s);
                let floor = PLATE_SPIN_M * s * m::sqrt(s);
                assert!((typical - st.r).norm() < 4.0 * floor, "typical spin at {s} s misses by {:.2} m", (typical - st.r).norm());
            }
        }
    }

    // The covariance the plate weighing assumes is the one the read samples (Codex plate-cue
    // review #2): every combination of the sampler's depth and across draws at ±1, averaged.
    #[test]
    fn the_read_covariance_is_the_sampled_one() {
        let los = Vec3::new(0.6, 0.8, 0.0);
        let lat = Vec3::new(-los.y, los.x, 0.0);
        let (sigma, depth) = (1.3, 3.0);
        let mut acc = Mat3::scale(0.0);
        for e0 in [-1.0, 1.0] {
            for e1 in [-1.0, 1.0] {
                // As `read_prediction` draws a position error.
                let e = los * (e0 * sigma * depth) + lat * (e1 * sigma) + Vec3::new(0.0, 0.0, e1 * sigma * 0.5);
                acc = acc + Mat3::outer(e, e) * 0.25;
            }
        }
        let c = read_cov(los, sigma, depth);
        for (a, b) in [(Vec3::X, Vec3::X), (Vec3::Y, Vec3::Z), (Vec3::Z, Vec3::Z), (Vec3::X, Vec3::Y), (Vec3::X, Vec3::Z)] {
            assert!(((c * a).dot(b) - (acc * a).dot(b)).abs() < 1e-12, "{a:?}·{b:?}");
        }
    }


    /// The spread (σ, m) of where a fielder at `from` reads `inp`'s ball to come down, along
    /// his line of sight, `before` seconds before it lands, over 600 draws as the simulator
    /// draws them; and the mean miss with every draw at zero.
    fn landing_spread(inp: &LiveInput, a: &FielderAttrs, from: Vec3, before: f64) -> (f64, f64) {
        let mut st = RulesState::default_fixture();
        step(&mut st, &Input::StartGame);
        let sim = new_sim(inp, &mut st);
        let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
        let (t_land, lx, ly) = path.first_bounce().unwrap();
        let land = Vec3::new(lx, ly, 0.0);
        let t = t_land - before;
        let s = t - inp.t_contact;
        let (p, v) = path.at(t);
        let los = Vec3::new(p.x - from.x, p.y - from.y, 0.0).normalized();
        let view = (s - a.reaction).max(0.0);
        let read = |err: [f64; 4]| {
            let path = read_prediction_k(inp.env, inp.field, (p, v), t, from, a.read_sigma0, a.read_vel_frac, a.tau_read, view, err, Some((s, inp.ball.r)));
            let q = path.first_bounce().map_or_else(|| path.at(path.t_end()).0, |(_, x, y)| Vec3::new(x, y, 0.0));
            Vec3::new(q.x, q.y, 0.0)
        };
        let mut d = Vec::new();
        for k in 0..600 {
            let mut r = rng(91, 0, 0, Purpose::FielderPerception, k);
            let err = [r.truncated_normal(4.0), r.truncated_normal(4.0), r.truncated_normal(4.0), r.truncated_normal(4.0)];
            d.push(read(err).dot(los));
        }
        let mean = d.iter().sum::<f64>() / d.len() as f64;
        let sd = m::sqrt(d.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / d.len() as f64);
        (sd, (read([0.0; 4]) - land).norm())
    }

    // Agent-models §5.1, the read closes as the ball comes down (2026-11-15): a 45° fly and a
    // 28° fly to centre, each read by an average outfielder from 12 m short of where it lands.
    // Bands from the spec: the high fly's read is the looser one early (1 s after contact, the
    // velocity error is flown over a longer flight); a second before landing both are within
    // about a metre and a half (σ), half a second before within half a metre: the
    // optical-acceleration threshold (Chapman 1968; Fink, Foo & Warren 2009) closes the miss he
    // can tell apart as the square of the time left. With no draws, the read misses by
    // under 1.5 m 2 s out (the
    // launch angle from the back-flight and the decayed spin, §5.1: this ball has the table's
    // spin, so the table's own bias, 1–3 m in games, is the next test's).
    #[test]
    fn the_read_closes_as_the_ball_comes_down() {
        let fx = fixture(&[]);
        let a = FielderAttrs::average(CF);
        let mut early = Vec::new();
        for (la, speed) in [(45.0f64, 43.0), (28.0, 41.0)] {
            let r = la.to_radians();
            let inp = hit(&fx, Vec3::new(0.0, speed * m::cos(r), speed * m::sin(r)));
            let mut st = fx.st.clone();
            let sim = new_sim(&inp, &mut st);
            let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
            let (t_land, lx, ly) = path.first_bounce().unwrap();
            let from = Vec3::new(lx, ly - 12.0, 0.0);
            let hang = t_land - inp.t_contact;
            let (s_early, _) = landing_spread(&inp, &a, from, hang - 1.0);
            let (s1, _) = landing_spread(&inp, &a, from, 1.0);
            let (s05, _) = landing_spread(&inp, &a, from, 0.5);
            let (_, bias2) = landing_spread(&inp, &a, from, 2.0);
            println!("LA {la}: hang {hang:.2} s; spread 1 s after contact {s_early:.2} m, 1 s before landing {s1:.2}, 0.5 s before {s05:.2}; no-draw miss 2 s before {bias2:.2}");
            assert!(s1 < 1.5, "LA {la}: {s1:.2} m a second before landing");
            assert!(s05 < 0.5, "LA {la}: {s05:.2} m half a second before landing");
            assert!(s05 < s1 && s1 < s_early, "LA {la}: the read closes");
            assert!(bias2 < 1.5, "LA {la}: no-draw miss {bias2:.2} m 2 s out");
            early.push(s_early);
        }
        assert!(early[0] > early[1], "the high fly is the harder read early: {early:?}");
    }

    /// Horizontal carry of a ball from `s` to the ground in open air (no fence), m.
    fn open_air_carry(env: &FlightEnv, s: BallState) -> f64 {
        let mut st = s;
        for _ in 0..4000 {
            let next = env.step(&st, 0.01);
            if next.r.z <= env.ball.radius && next.v.z < 0.0 {
                let p = st.r + (next.r - st.r) * ((st.r.z - env.ball.radius) / (st.r.z - next.r.z));
                return m::hypot(p.x - s.r.x, p.y - s.r.y);
            }
            st = next;
        }
        m::hypot(st.r.x - s.r.x, st.r.y - s.r.y)
    }

    // Agent-models §5.1: the typical spin is a belief about the contact model and must follow
    // it. On the balls launched toward the field in 24 games outside the tables' fitting sample
    // (seeds 100–123; the tables are fitted on 0–95), in each launch-angle × exit-speed cell
    // of the liner and fly-ball rows (15°, 25°, 35°, 45°; 70 mph and up) with at least 30 balls:
    // - the median backspin against `SPIN_BACK_RPM` at the cell: within 15% (in the 15° row,
    //   whose backspin is about 600 rpm, within LINER_BACK_RPM: 15% of it is under the
    //   difference between two 30–70-ball medians there, up to 206 rpm out of sample on
    //   2026-11-19; that row's carry misses were +0.8 to +1.6 m);
    // - the whole belief (backspin and the unseen spin's drag, at each ball's own launch angle
    //   and speed), flown from each ball's own launch, against how far the ball truly carries
    //   (the world's own launch state, flown with its own spin; open air): median miss within
    //   3 m in each fly-ball cell (30–80 balls: the median's sampling error is about 1 m) and
    //   within 1 m over all of them together. The 15° row (10–20°), where the fitted unseen
    //   spin is largest against the backspin, gets LINER_CELL_M per cell and is left out of the
    //   pooled median, which keeps its meaning for flies (Opus review 2026-11-19, P3-2). With
    //   the backspin alone (the belief before 2026-11-19) the reads of these flies carried a
    //   median 3 m long, and of 10–25° liners 2.5–5.7 m long.
    #[test]
    fn the_typical_spin_belief_follows_the_contact_model() {
        let env = FlightEnv::default_env();
        const LINER_CELL_M: f64 = 4.0;
        const LINER_BACK_RPM: f64 = 300.0;
        // [LA 15, 25, 35, 45][EV 75, 85, 95, 105]: (backspin rpm, carry miss m).
        let mut cells: Vec<Vec<Vec<(f64, f64)>>> = vec![vec![Vec::new(); 4]; 4];
        for seed in 100..124 {
            let mut g = crate::game::Game::new(seed, false);
            while let Some(step) = g.step_pitch() {
                let (Some((ev, la, _)), Some(path)) = (step.record.batted, &step.batted) else { continue };
                let Some(bb_physics::ground::Segment::Flight { tr, .. }) = path.segments.first() else { continue };
                let s0 = tr.first().s;
                let (deg, mph) = (la.to_degrees(), ev / MPH);
                if m::atan2(s0.v.x, s0.v.y).abs() > core::f64::consts::FRAC_PI_4 || !(10.0..50.0).contains(&deg) || mph < 70.0 {
                    continue;
                }
                let h = Vec3::new(s0.v.x, s0.v.y, 0.0).normalized();
                let axis = Vec3::new(h.y, -h.x, 0.0);
                let back = s0.w.dot(axis) * 30.0 / core::f64::consts::PI;
                let (benv, w) = believed_flight(&env, s0.v, Some(0.0));
                let miss = open_air_carry(&benv, BallState { w, ..s0 }) - open_air_carry(&env, s0);
                cells[((deg - 10.0) / 10.0) as usize][(((mph - 70.0) / 10.0) as usize).min(3)].push((back, miss));
            }
        }
        let median = |mut v: Vec<f64>| {
            v.sort_by(|a, b| a.total_cmp(b));
            v[v.len() / 2]
        };
        let (mut checked, mut pooled) = (0, Vec::new());
        for (i, row) in cells.iter().enumerate() {
            for (j, c) in row.iter().enumerate() {
                if c.len() < 30 {
                    continue;
                }
                checked += 1;
                let cell = format!("LA {}°, {} mph (n {})", 15 + 10 * i, 75 + 10 * j, c.len());
                let table = SPIN_BACK_RPM[i + 4][j + 1];
                let bound = if i == 0 { LINER_CELL_M } else { 3.0 };
                let (back, miss) = (median(c.iter().map(|x| x.0).collect()), median(c.iter().map(|x| x.1).collect()));
                println!("{cell}: backspin {back:.0} rpm against {table:.0}; carry miss {miss:+.2} m");
                let back_ok = if i == 0 { (table - back).abs() < LINER_BACK_RPM } else { (table / back - 1.0).abs() < 0.15 };
                assert!(back_ok, "{cell}: backspin {back:.0} rpm, belief {table:.0}");
                assert!(miss.abs() < bound, "{cell}: the belief carries {miss:+.2} m against the truth");
                if i > 0 {
                    pooled.extend(c.iter().map(|x| x.1));
                }
            }
        }
        assert!(checked >= 8, "only {checked} cells with 30 balls");
        let all = median(pooled);
        println!("all checked cells: carry miss {all:+.2} m");
        assert!(all.abs() < 1.0, "the belief carries {all:+.2} m against the truth over the checked cells");
    }

    // Agent-models §5.1 and §6.3 (fly-read job, 2026-11-19; Opus catch-fixes2 review P2-1): a
    // runner's first look 0.5 s off the bat, with no read error at all (his plate prior, the
    // reference atmosphere, the typical spin), must not read deep flies long or short across
    // the population. On every fair 25–50° ball in seeds 100–123 (outside the spin tables'
    // fitting sample) that lands in the park 80 m or more from home, the distance from home of
    // the read's landing point against the world's own landing point (the independent
    // reference): median within ±1 m, at least a quarter of the reads on each side, and at
    // most 3% read over the fence. Before, the median was +3.2 m, every read was deeper than
    // the truth, and one deep fly in ten was read as a home run.
    #[test]
    fn a_zero_error_first_look_reads_deep_flies_without_bias() {
        let field = Field::default();
        let (mut miss, mut over) = (Vec::new(), 0usize);
        for seed in 100..124 {
            let mut g = crate::game::Game::new(seed, false);
            while let Some(step) = g.step_pitch() {
                let (Some((_, la, _)), Some(path)) = (step.record.batted, &step.batted) else { continue };
                if !(25.0..50.0).contains(&la.to_degrees()) || !step.rules_events.iter().any(|e| matches!(e, bb_rules::Event::FairFoulDecided { fair: true })) {
                    continue;
                }
                let Some(bb_physics::ground::Segment::Flight { tr, .. }) = path.segments.first() else { continue };
                let Some((_, x, y)) = path.first_bounce() else { continue };
                let truth = m::hypot(x, y);
                if truth < 80.0 {
                    continue;
                }
                let t = tr.first().t + RUNNER_READ_DELAY_S;
                let look = ContactLook { motion: path.at(t), from: base_pos(2), t, view: RUNNER_READ_DELAY_S, contact_since: Some(RUNNER_READ_DELAY_S) };
                let read = look.project(&field, [0.0; 4]);
                match read.first_bounce() {
                    Some((_, rx, ry)) => miss.push(m::hypot(rx, ry) - truth),
                    None => over += 1,
                }
            }
        }
        let n = miss.len() + over;
        miss.sort_by(|a, b| a.total_cmp(b));
        let med = miss[miss.len() / 2];
        let deeper = miss.iter().filter(|&&d| d > 0.0).count() as f64 / n as f64;
        println!("{n} deep flies: median miss {med:+.2} m, read deeper {:.0}%, read over the fence {over}", 100.0 * deeper);
        assert!(n >= 100, "{n} deep flies");
        assert!(med.abs() < 1.0, "median miss {med:+.2} m");
        assert!((0.25..=0.75).contains(&deeper), "{:.0}% read deeper", 100.0 * deeper);
        assert!(over as f64 <= 0.03 * n as f64, "{over} of {n} read over the fence");
    }

    // The owner's play (2026-11-15): seed 23080 play 8. On the integration of the held branches
    // it is 98.5 mph at 45°, 6.5 s in the air, the left fielder 64 ft from where it comes down
    // (a routine fly), and it fell: his read drifted 9 m deep at mid-flight (the outfield-read
    // review's P2-1) and at the sourced BRAKE he turned back too late. Kept as its contact, not
    // its seed's game (the game moves with every change): the ball as it left the bat, the
    // situation (one out, bases empty, a right-handed batter), the swing and pitch the
    // fielders read, and the play's own random streams (seed 23080, play 8). With the tracking
    // read (agent-models §5.1) it is caught.
    #[test]
    fn the_owners_fly_seed_23080_play_8_is_caught() {
        let mut st = RulesState::default_fixture();
        step(&mut st, &Input::StartGame);
        st.outs = 1;
        let t_c = 0.39292835965695994;
        step(&mut st, &Input::PitchDelivered { t: 0.0 });
        step(&mut st, &Input::BatContact { t: t_c });
        let fx = Fixture { env: FlightEnv::default_env(), field: Field::default(), st };
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.fattrs = core::array::from_fn(FielderAttrs::average);
        inp.ball = BallState {
            r: Vec3::new(-0.06743612629505924, 0.7480339821824925, 0.9004703517124966),
            v: Vec3::new(-9.33032521808202, 29.751386604051454, 31.06351597846284),
            w: Vec3::new(442.5716024586851, -39.42614946729392, -38.767477399883774),
        };
        inp.t_contact = t_c;
        inp.pitch_x = Some(-0.053352185811326856);
        inp.bat_yaw = Some(1.0830445788889875);
        inp.swing_lead = Some(0.1740766339469236);
        (inp.seed, inp.play) = (23080, 8);
        let mut st = fx.st.clone();
        let out = simulate_live(&inp, &mut st);
        assert!(out.events.iter().any(|e| matches!(e, Event::Catch { .. })), "not caught: {:?}", out.facts);
    }

    /// Fielder `CF`, standing 12 m short of where a ball hit at `la` degrees and `speed` m/s
    /// to centre comes down, tracks it (agent-models §5.1, the tracking read) on plays 0..`n`
    /// (each its own look draws): the RMS over plays of how far from where it lands his read
    /// has it coming down, 1 s after contact, at the top of its flight, and 2, 1 and 0.5 s
    /// before it lands.
    fn track_rms(la: f64, speed: f64, n: u32) -> [f64; 5] {
        let fx = fixture(&[]);
        let mut sq = [0.0; 5];
        for play in 0..n {
            let r = la.to_radians();
            let mut inp = hit(&fx, Vec3::new(0.0, speed * m::cos(r), speed * m::sin(r)));
            inp.play = play;
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
            let (t_land, lx, ly) = path.first_bounce().unwrap();
            let mut apex = inp.t_contact;
            let mut k = inp.t_contact;
            while k < t_land {
                if path.at(k).0.z > path.at(apex).0.z {
                    apex = k;
                }
                k += 0.01;
            }
            let land = Vec3::new(lx, ly, 0.0);
            sim.f[CF].pos = Vec3::new(lx, ly - 12.0, 0.0);
            let when = [inp.t_contact + 1.0, apex, t_land - 2.0, t_land - 1.0, t_land - 0.5];
            let mut next = 0;
            sim.t = inp.t_contact;
            while next < 5 && sim.t < t_land {
                sim.take_looks();
                while next < 5 && sim.t + DT > when[next] {
                    let p = sim.predict(CF);
                    let q = p.first_bounce().map_or_else(|| p.at(p.t_end()).0, |(_, x, y)| Vec3::new(x, y, 0.0));
                    let e = Vec3::new(q.x - land.x, q.y - land.y, 0.0).norm();
                    sq[next] += e * e;
                    next += 1;
                }
                sim.t += DT;
            }
        }
        sq.map(|x| m::sqrt(x / n as f64))
    }

    // Agent-models §5.1, the tracking read: a 45° and a 28° fly to centre, tracked by an
    // average outfielder from 12 m short of where each lands, over 40 plays of look draws. A
    // high fly is the harder read early and at the top of its flight, by the geometry of what
    // he sees (its landing point is its velocity times a longer time left, and at its top it
    // is farther from him, its hard direction steep); both have converged by a second before
    // landing: within 1.5 m a second out, and half a second out within his planning reach
    // (0.7 × `REACH`), so that the plan he makes then puts the ball in his glove.
    #[test]
    fn the_tracking_read_is_harder_on_a_high_fly_early_and_converges_late() {
        let high = track_rms(45.0, 43.0, 40);
        let low = track_rms(28.0, 41.0, 40);
        println!("RMS landing miss at 1 s after contact, the top, 2 / 1 / 0.5 s before landing: 45° {high:.2?}; 28° {low:.2?}");
        assert!(high[0] > low[0] && high[1] > low[1], "the high fly is the harder read early and at its top: {high:.2?} against {low:.2?}");
        for (la, e) in [(45, high), (28, low)] {
            assert!(e[3] < 1.5 && e[4] < REACH * 0.7, "LA {la}: {e:.2?}");
            assert!(e[4] < e[3] && e[3] < e[2], "LA {la}: the read converges: {e:.2?}");
        }
    }

    // Agent-models §5.1, the persistent part of a look's error (owner's saved play, seed 40580
    // play 15): a filter is honest when the covariance it reports covers the error it actually
    // has. On 120 plays of draws (each its own fixed and fresh draws, as in games) an average
    // centre fielder, standing 12 m short of where a routine fly (38°, 41 m/s) comes down, tracks
    // it. Across his line of sight, level, the mean of (his position error)² / (his own σ)² 2 s,
    // 1 s and 0.6 s before it lands is that of a unit normal, 1, within a factor of two either
    // way (its standard error over 120 plays is about 0.13; the spin he cannot see adds a little
    // he does not model). An estimate that treats a fixed error as fresh on every look grows
    // sure of a wrong line: that mean is many times 1.
    #[test]
    fn the_tracking_read_reports_a_covariance_that_covers_its_error() {
        let fx = fixture(&[]);
        let n = 120;
        let mut nees = [0.0; 3];
        let mut rms = [[0.0; 2]; 3];
        for play in 0..n {
            let r = 38f64.to_radians();
            let mut inp = hit(&fx, Vec3::new(0.0, 41.0 * m::cos(r), 41.0 * m::sin(r)));
            inp.play = play;
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
            let path = path.clone();
            let (t_land, lx, ly) = path.first_bounce().unwrap();
            sim.f[CF].pos = Vec3::new(lx, ly - 12.0, 0.0);
            let when = [t_land - 2.0, t_land - 1.0, t_land - 0.6];
            let mut next = 0;
            sim.t = inp.t_contact;
            while next < 3 && sim.t < t_land {
                sim.take_looks();
                while next < 3 && sim.t + DT > when[next] {
                    let tr = sim.track_now(CF).unwrap();
                    let b = path.at(sim.t).0;
                    let (_, _, a, _) = sight_frame(sim.f[CF].pos + Vec3::new(0.0, 0.0, EYE_H), b);
                    let e = (tr.r() - b).dot(a);
                    let var = a.dot(tr.blk(0, 0) * a);
                    nees[next] += e * e / var;
                    rms[next][0] += e * e;
                    rms[next][1] += var;
                    next += 1;
                }
                sim.t += DT;
            }
        }
        for (k, label) in ["2 s", "1 s", "0.6 s"].iter().enumerate() {
            let q = nees[k] / n as f64;
            let (miss, sd) = (m::sqrt(rms[k][0] / n as f64), m::sqrt(rms[k][1] / n as f64));
            println!("{label} before landing: lateral RMS error {miss:.3} m, RMS σ {sd:.3} m, mean e²/σ² {q:.2}");
            assert!(q > 0.5 && q < 2.0, "{label} before landing: mean e²/σ² {q:.2} (RMS error {miss:.3} m against σ {sd:.3} m)");
        }
    }

    // Agent-models §5.1, one look's update with a persistent error he has learned (Codex
    // track-bias review P2-1). The persistent error's size and directions depend on where the
    // ball is (its range, his line of sight) and how fast it goes, so once his estimate of it
    // is not zero the look's Jacobian is not the identity. Isolated from flight: a belief with
    // the ball's position known to σ 0.5 m per axis and its velocity to 1 m/s, his persistent
    // error known exactly (and not zero), and the ball drawn from that belief; a position
    // look and then a velocity look drawn as the world draws them (about the true ball, with
    // fresh noise and that persistent error). Over 2,000 draws, along each direction of his
    // line of sight (along it, across it level, across it upward) the mean of (error)² / (his
    // own variance) after both looks, for position and for velocity, is that of a unit
    // normal, 1, within 0.85–1.15 (the standard error over 2,000 is about 0.03), for three
    // geometries (near and low, far and high, a liner to his side) and three persistent
    // errors, among them a range error of −2σ (with the identity Jacobian that gave 2.4).
    #[test]
    fn a_look_with_a_learned_persistent_error_reports_a_covariance_that_covers_its_error() {
        let eye = Vec3::new(0.0, 0.0, EYE_H);
        let a = FielderAttrs::average(CF);
        let cases = [
            (Vec3::new(0.0, 10.0, 3.0), Vec3::new(1.0, -6.0, -9.0)),
            (Vec3::new(6.0, 40.0, 22.0), Vec3::new(2.0, 18.0, 4.0)),
            (Vec3::new(-9.0, 25.0, 1.5), Vec3::new(-12.0, 30.0, 2.0)),
        ];
        let biases = [[-2.0, 0.0, 0.0, 0.0], [0.0, 2.0, 0.0, -2.0], [2.0, -2.0, 2.0, 2.0]];
        let n = 2000;
        for (ci, &(r0, v0)) in cases.iter().enumerate() {
            for (bi, &bias) in biases.iter().enumerate() {
                let lp = Look { eye, vel: false, sigma_per_m: a.read_sigma0 / READ_REF_DIST, vel_frac: a.read_vel_frac, scale: LOOK_SCALE, kb: TRACK_BIAS_FRAC };
                let lv = Look { vel: true, ..lp };
                let (_, u, al, w) = sight_frame(eye, r0);
                let dirs = [u, al, w];
                let mut sums = [[0.0; 3]; 2];
                for j in 0..n {
                    let mut g = rng(4242, (ci * 3 + bi) as u32, 0, Purpose::FielderLook, j);
                    let mut nrm = || g.truncated_normal(4.0);
                    let r = r0 + Vec3::new(0.5 * nrm(), 0.5 * nrm(), 0.5 * nrm());
                    let v = v0 + Vec3::new(nrm(), nrm(), nrm());
                    // The world's look about the true ball: its own geometry, fresh and
                    // persistent parts.
                    let (d, ut, at, wt) = sight_frame(eye, r);
                    let bp = a.read_sigma0 * d / READ_REF_DIST;
                    let sp = bp * LOOK_SCALE;
                    let zp = r + ut * (DEPTH_STRETCH * (sp * nrm() + TRACK_BIAS_FRAC * bp * bias[0])) + at * (sp * nrm() + TRACK_BIAS_FRAC * bp * bias[1]) + wt * (sp * nrm());
                    let bv = a.read_vel_frac * v.norm();
                    let sv = bv * LOOK_SCALE;
                    let zv = v + ut * (0.5 * DEPTH_STRETCH * (sv * nrm() + TRACK_BIAS_FRAC * bv * bias[2])) + at * (sv * nrm() + TRACK_BIAS_FRAC * bv * bias[3]) + wt * (sv * nrm());
                    let mut tr = Track::new(0, 0.0, [r0, v0, Vec3::ZERO], [[Mat3::scale(0.25), Mat3::ZERO, Mat3::ZERO], [Mat3::ZERO, Mat3::IDENTITY, Mat3::ZERO], [Mat3::ZERO, Mat3::ZERO, Mat3::IDENTITY]]);
                    tr.b = bias;
                    for k in 0..4 {
                        tr.p[TB + k][TB + k] = 0.0;
                    }
                    tr.see(&lp, zp);
                    tr.see(&lv, zv);
                    for (m_, truth) in [(0, r), (1, v)] {
                        let e = tr.x[m_] - truth;
                        let pm = tr.blk(m_, m_);
                        for (k, dir) in dirs.iter().enumerate() {
                            let q = e.dot(*dir);
                            sums[m_][k] += q * q / dir.dot(pm * *dir);
                        }
                    }
                }
                for (m_, what) in ["position", "velocity"].iter().enumerate() {
                    let q = sums[m_].map(|x| x / n as f64);
                    println!("case {ci}, bias {bias:?}: {what} mean e²/σ² along / across / up {q:.3?}");
                    assert!(q.iter().all(|&x| x > 0.85 && x < 1.15), "case {ci}, bias {bias:?}: {what} mean e²/σ² {q:.3?}");
                }
            }
        }
    }

    // Agent-models §5.1, isolation: the tracking read is made of what he has seen. Two balls
    // from the same contact, one with twice the other's spin, come down far apart; a fielder
    // who has watched the first for two seconds plans exactly the same with the second ball
    // put in its place (his estimate is the same: the second ball's future, and its present,
    // are not his to see until he looks).
    #[test]
    fn the_tracking_read_does_not_see_the_balls_future() {
        let fx = fixture(&[]);
        let r = 45f64.to_radians();
        let inp = hit(&fx, Vec3::new(0.0, 43.0 * m::cos(r), 43.0 * m::sin(r)));
        let mut inp2 = hit(&fx, Vec3::new(0.0, 43.0 * m::cos(r), 43.0 * m::sin(r)));
        inp2.ball.w = inp2.ball.w * 2.0;
        let (mut st, mut st2) = (fx.st.clone(), fx.st.clone());
        let mut a = new_sim(&inp, &mut st);
        let mut b = new_sim(&inp2, &mut st2);
        let land = |sim: &Sim| landing(sim);
        assert!((land(&a) - land(&b)).norm() > 3.0, "the two balls come down {:.1} m apart", (land(&a) - land(&b)).norm());
        a.t = inp.t_contact;
        while a.t < inp.t_contact + 2.0 {
            a.take_looks();
            a.t += DT;
        }
        b.t = a.t;
        b.f = a.f.clone();
        for sim in [&mut a, &mut b] {
            sim.f[CF].read_at = f64::NEG_INFINITY;
            sim.read(CF);
        }
        assert_eq!((a.f[CF].t_int, a.f[CF].int_pt), (b.f[CF].t_int, b.f[CF].int_pt));
        assert_eq!(a.f[CF].doubt, b.f[CF].doubt);
    }

    // Agent-models §5.2, arriving under control: a chaser with time in hand running at 7.8 m/s
    // at a spot 20 m on, with a doubt of 3 m, brakes to a stop that far short of it and does
    // not pass it; with no doubt he stops on it. Late for the ball, he runs on at speed.
    #[test]
    fn a_chaser_with_time_in_hand_stops_his_doubt_short_of_his_spot() {
        let fx = fixture(&[]);
        let inp = hit(&fx, Vec3::new(0.0, 30.0, 30.0));
        let mut st = fx.st.clone();
        let sim = new_sim(&inp, &mut st);
        let run = |doubt: f64, t_int: f64| {
            let mut fd = sim.f[5].clone();
            (fd.role, fd.t_int, fd.doubt) = (Role::Chase, t_int, doubt);
            (fd.pos, fd.vel, fd.target) = (Vec3::new(0.0, 40.0, 0.0), Vec3::new(0.0, 7.8, 0.0), Vec3::new(0.0, 60.0, 0.0));
            let mut far = fd.pos.y;
            let mut t = 5.0;
            for _ in 0..600 {
                (fd.pos, fd.vel) = fielder_step(&fd, t, inp.t_contact);
                far = far.max(fd.pos.y);
                t += DT;
            }
            (far, fd.pos.y, fd.vel.norm())
        };
        let (far, at, v) = run(3.0, 100.0);
        // Within a tick's braking (1/60 s) of the stop the law gives.
        assert!(far <= 57.15 && at > 56.5 && v < 0.05, "doubt 3 m: farthest {far:.2}, stops at {at:.2} at {v:.2} m/s");
        let (far, at, _) = run(0.0, 100.0);
        assert!(far <= 60.15 && at > 59.9, "no doubt: farthest {far:.2}, stops at {at:.2}");
        let (far, _, _) = run(3.0, 5.5);
        assert!(far > 60.0, "late, he runs through: farthest {far:.2}");
    }

    // The pop-up the Opus wall-braking review traced (P2-1, seed 1000003040 play 5), as a
    // constructed ball: 37 m/s at 62° toward short left-centre, coming down at (−32.5, 40.8)
    // after 6.2 s, between the shortstop and the left fielder. Caught on at least 18 of 20
    // plays of draws (the owner's band for a routine play is 90–99% by ability).
    #[test]
    fn the_reviews_high_pop_up_between_short_and_left_is_caught() {
        let fx = fixture(&[]);
        let az = m::atan2(-33.5, 41.5);
        let r = 62f64.to_radians();
        let mut caught = 0;
        for play in 0..20 {
            let mut inp = hit(&fx, Vec3::new(37.0 * m::cos(r) * m::sin(az), 37.0 * m::cos(r) * m::cos(az), 37.0 * m::sin(r)));
            inp.play = play;
            inp.fattrs = core::array::from_fn(FielderAttrs::average);
            let mut st = fx.st.clone();
            let out = simulate_live(&inp, &mut st);
            caught += out.events.iter().any(|e| matches!(e, Event::Catch { .. })) as u32;
        }
        assert!(caught >= 18, "caught {caught} of 20");
    }

    // Agent-models §10, `ReadSkill`: one set of fly balls, chosen by geometry alone (in the air
    // at least 3.5 s, coming down in the park 18–28 m from the nearest outfielder's spot: the
    // harder, running end of the plays of docs/calibration/2026-11-15-outfield-read.md), each
    // played with four sets of read draws: an outfield of grade 20 catches fewer than one of
    // grade 80. Perfect hands (`input`), so only the read differs. The balls are batted balls
    // from games 0–23 and carry their simulated spins.
    #[test]
    fn a_poor_reader_misses_more_flies_than_a_good_one() {
        let fx = fixture(&[]);
        let home = home_positions();
        let mut flies = Vec::new();
        for seed in 0..24 {
            let mut g = crate::game::Game::new(seed, false);
            while let Some(step) = g.step_pitch() {
                let (Some((_, la, _)), Some(path)) = (step.record.batted, &step.batted) else { continue };
                let Some(bb_physics::ground::Segment::Flight { tr, .. }) = path.segments.first() else { continue };
                if !(25.0..60.0).contains(&la.to_degrees()) {
                    continue;
                }
                let ball = BallState { r: Vec3::new(0.0, 0.5, 1.0), ..tr.first().s };
                let mut inp = hit(&fx, ball.v);
                inp.ball = ball;
                let mut st = fx.st.clone();
                let sim = new_sim(&inp, &mut st);
                let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
                let Some((t, x, y)) = path.first_bounce() else { continue };
                let near = (6..9).map(|i| m::hypot(home[i].x - x, home[i].y - y)).fold(f64::INFINITY, f64::min);
                if t - inp.t_contact >= 3.5 && (18.0..28.0).contains(&near) && path.walls.is_empty() {
                    flies.push(ball);
                }
            }
        }
        assert!(flies.len() >= 40, "{} flies", flies.len());
        let catches = |grade: f64| {
            let mut n = 0;
            for &ball in &flies {
                for k in 0..4u32 {
                    let mut inp = hit(&fx, ball.v);
                    inp.ball = ball;
                    for (i, a) in inp.fattrs.iter_mut().enumerate() {
                        *a = FielderAttrs { hands: 0.0, ..FielderAttrs::average(i).with_read(grade) };
                    }
                    inp.seed = 500 + k as u64;
                    let mut st = fx.st.clone();
                    let out = simulate_live(&inp, &mut st);
                    n += out.events.iter().any(|e| matches!(e, Event::Catch { .. })) as u32;
                }
            }
            n
        };
        let (poor, good) = (catches(20.0), catches(80.0));
        println!("caught of {}: grade 20 {poor}, grade 80 {good}", 4 * flies.len());
        assert!(good > poor, "grade 20 caught {poor}, grade 80 {good}");
    }

    // A fielder never reads the ball's future (agent-models §1.1, §5.1): two balls in the same
    // place, moving the same way, one with the contact model's typical backspin and one with
    // twice it, come down metres apart, but the centre fielder's read and his intercept are the
    // same for both.
    #[test]
    fn the_read_does_not_see_the_true_spin() {
        let fx = fixture(&[]);
        let mut plans = Vec::new();
        let mut lands = Vec::new();
        for spin in [1.0, 2.0] {
            let mut inp = hit(&fx, Vec3::new(4.0, 30.0, 26.0));
            inp.ball.w = inp.ball.w * spin;
            inp.fattrs[CF].reaction = 0.0;
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = inp.t_contact;
            sim.f[CF].err = [1.0, -0.5, 0.7, 0.3];
            lands.push(landing(&sim));
            sim.read(CF);
            plans.push((sim.f[CF].int_pt, sim.f[CF].t_int, sim.f[CF].belief.as_ref().and_then(|p| p.first_bounce())));
        }
        assert!((lands[0] - lands[1]).norm() > 3.0, "the spin matters: {lands:?}");
        assert_eq!(plans[0], plans[1]);
    }

    // Agent-models §5.1, the tracking read's doubt (Codex tracking-read review P2-1): a belief
    // whose only uncertainty is the learned acceleration, one m/s² per axis, has the doubt that
    // acceleration's effect on the landing point gives: per axis, a ±1 m/s² change flown with
    // his own physics moves the landing point along his line of sight by what the doubt adds.
    #[test]
    fn a_tracked_doubt_carries_the_learned_acceleration() {
        let fx = fixture(&[]);
        let r = 45f64.to_radians();
        let inp = hit(&fx, Vec3::new(0.0, 43.0 * m::cos(r), 43.0 * m::sin(r)));
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = inp.t_contact + 2.0;
        let (b, bv) = sim.ball_at(sim.t);
        let from = Vec3::new(0.0, 85.0, 0.0);
        let bare = Track { looks: 1, next_look: sim.t + LOOK_S, ..Track::new(sim.ball_gen, sim.t, [b, bv, Vec3::ZERO], [[Mat3::ZERO; 3]; 3]) };
        let d0 = sim.track_doubt(&bare, from);
        let los = Vec3::new(b.x - from.x, b.y - from.y, 0.0).normalized();
        let land = |acc: Vec3| {
            let p = sim.fly_estimate(b, bv, acc);
            let (_, x, y) = p.first_bounce().unwrap();
            Vec3::new(x, y, 0.0).dot(los)
        };
        for k in 0..3 {
            let mut e = [0.0; 3];
            e[k] = 1.0;
            let unit = Vec3::new(e[0], e[1], e[2]);
            let shift = (land(unit) - land(unit * -1.0)) / 2.0;
            let mut tr = bare;
            let uu = Mat3::outer(unit, unit);
            for (a_, row) in uu.r.iter().enumerate() {
                for (c, &x) in row.iter().enumerate() {
                    tr.p[6 + a_][6 + c] = x;
                }
            }
            let d = sim.track_doubt(&tr, from);
            let added = m::sqrt((d * d - d0 * d0).max(0.0));
            assert!((added - shift.abs()).abs() < 0.15 * shift.abs().max(0.2), "axis {k}: the doubt adds {added:.3} m, a ±1 m/s² change moves the landing {shift:.3} m");
        }
    }

    // Agent-models §5.1, the tracking read's doubt against the spread of his reads. Sample
    // contact directly from plate appearances so baserunning and fielding cannot change
    // the flights offered to the test. Track their real spins, unknown to the fielder,
    // from 12 m short of where they land. The RMS of his doubt and of his line-of-sight
    // miss, 2 s and 1 s before landing, agree within a factor of 1.5 either way.
    #[test]
    fn the_tracking_doubt_matches_the_spread_of_tracked_reads() {
        let fx = fixture(&[]);
        use bb_observe::Hand::{Left, Right};
        use crate::pa::{Knobs, Matchup, PA_PLAY_STRIDE, simulate_pa};

        let mut balls = Vec::new();
        for (seed, bats) in [Right, Left].into_iter().enumerate() {
            let mut matchup = Matchup::for_hands(Right, bats);
            matchup.knobs = Knobs::calibrated_m3();
            for pa in 0..600 {
                let record = simulate_pa(&matchup, seed as u64, pa * PA_PLAY_STRIDE);
                for pitch in record.pitches {
                    let (Some((_, launch, _)), Some((_, ball))) = (pitch.batted, pitch.contact_state) else { continue };
                    if (25.0..60.0).contains(&launch.to_degrees()) {
                        balls.push(ball);
                    }
                }
            }
        }
        let mut sums = [[0.0; 2]; 2];
        let mut n = 0.0;
        for (k, ball) in balls.iter().enumerate() {
            let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
            inp.ball = BallState { r: Vec3::new(0.0, 0.5, 1.0), ..*ball };
            inp.play = k as u32;
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
            let Some((t_land, lx, ly)) = path.first_bounce() else { continue };
            if t_land - inp.t_contact < 3.5 || path.walls.first().is_some_and(|w| w.0 < t_land) {
                continue;
            }
            let land = Vec3::new(lx, ly, 0.0);
            let from = land - land.normalized() * 12.0;
            sim.f[CF].pos = from;
            let when = [t_land - 2.0, t_land - 1.0];
            let mut next = 0;
            sim.t = inp.t_contact;
            while next < 2 && sim.t < t_land {
                sim.take_looks();
                if sim.t + DT > when[next] {
                    let tr = sim.track_now(CF).unwrap();
                    let los = Vec3::new(tr.r().x - from.x, tr.r().y - from.y, 0.0).normalized();
                    let p = sim.predict(CF);
                    let q = p.first_bounce().map_or_else(|| p.at(p.t_end()).0, |(_, x, y)| Vec3::new(x, y, 0.0));
                    let miss = (Vec3::new(q.x, q.y, 0.0) - land).dot(los);
                    let doubt = sim.track_doubt(&tr, from);
                    sums[next][0] += miss * miss;
                    sums[next][1] += doubt * doubt;
                    next += 1;
                }
                sim.t += DT;
            }
            n += 1.0;
        }
        for (k, label) in ["2 s", "1 s"].iter().enumerate() {
            let (miss, doubt) = (m::sqrt(sums[k][0] / n), m::sqrt(sums[k][1] / n));
            println!("{label} before landing, {n} balls: RMS miss {miss:.2} m, RMS doubt {doubt:.2} m");
            assert!(n >= 40.0 && doubt > miss / 1.5 && doubt < miss * 1.5, "{label}: RMS miss {miss:.2} m, RMS doubt {doubt:.2} m ({n} balls)");
        }
    }

    // Agent-models §5.1, a grounder's first look (Codex tracking-read review P2-2): the first
    // look is the snapshot read's first read, its shrink included, persistent part and all.
    // The shortstop at his reaction gate, a grounder at 36 m/s (−1, 0 and +2 m/s up), his
    // lateral draws one σ and fresh noise off: his tracked position is no worse than the
    // snapshot read's, within 10% (its plate cue and contact prior are the same weighing).
    #[test]
    fn a_grounders_first_look_is_read_as_sharply_as_the_snapshot_read() {
        const SS: usize = 5;
        LOOK_NOISE.with(|c| c.set((0.0, 1.0)));
        let fx = fixture(&[]);
        for vz in [-1.0, 0.0, 2.0] {
            let inp = hit(&fx, Vec3::new(5.0, 36.0, vz));
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.f[SS].err = [0.0, 1.0, 0.0, 1.0];
            sim.t = inp.t_contact + sim.f[SS].a.reaction;
            let (b, v) = sim.ball_at(sim.t);
            assert!(sim.off_bat().is_some(), "vz {vz}: still in the air at his first look");
            let fd = &sim.f[SS];
            let old = read_prediction_k(inp.env, inp.field, (b, v), sim.t, fd.pos, fd.a.read_sigma0, fd.a.read_vel_frac, fd.a.tau_read, 0.0, fd.err, sim.off_bat());
            let old_err = (old.at(sim.t).0 - b).norm();
            sim.take_looks();
            let new_err = (sim.track_now(SS).unwrap().r() - b).norm();
            println!("vz {vz}: snapshot {old_err:.4} m, tracked {new_err:.4} m");
            assert!(new_err <= 1.1 * old_err + 1e-3, "vz {vz}: snapshot {old_err:.4} m, tracked {new_err:.4} m");
        }
        LOOK_NOISE.with(|c| c.set((1.0, 1.0)));
    }

    // Codex tracking-read review P2-3: below 2 m the reach estimate (`run_time_to`, with
    // `wall_delay`) against the movement itself, a late chaser stepped by `move_fielders` until
    // within the planning reach: every mix of oblique motion up to 8 m/s on open ground, and
    // running into the centre-field wall. Within ±0.35 s, as the main grid.
    #[test]
    fn reach_estimate_tracks_the_movement_step_below_two_metres_and_at_a_wall() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        const SS: usize = 5;
        let fence = fx.field.fence_at(0.0);
        let mut worst = (0.0f64, String::new());
        for wall in [false, true] {
            for &d in &[0.75, 1.0, 1.5, 1.9] {
                for &vx in &[-8.0, -4.0, 0.0, 4.0, 8.0] {
                    for &vy in &[-8.0, -4.0, 0.0, 4.0, 8.0] {
                        if vx * vx + vy * vy > 70.0 {
                            continue;
                        }
                        let mut st = fx.st.clone();
                        let mut sim = new_sim(&inp, &mut st);
                        sim.t = 2.0;
                        // On open ground, or with the goal on the centre-field wall's line, his
                        // body clear of it, coming at it from d short.
                        let goal = if wall { Vec3::new(0.0, fence - BODY_R, 0.0) } else { Vec3::new(-20.0, 60.0, 0.0) };
                        let o = if wall { goal - Vec3::new(0.0, d, 0.0) } else { goal - Vec3::new(d, 0.0, 0.0) };
                        sim.f[SS].pos = o;
                        sim.f[SS].vel = Vec3::new(vx, vy, 0.0);
                        sim.f[SS].target = goal;
                        sim.f[SS].role = Role::Chase;
                        sim.f[SS].t_int = 0.0;
                        let est = run_time_to(inp.field, &sim.f[SS], goal, REACH * 0.7) + wall_delay(inp.field, &sim.f[SS], goal, REACH * 0.7);
                        let mut truth = f64::NAN;
                        for k in 1..2000 {
                            sim.move_fielders();
                            sim.t += DT;
                            if (goal - sim.f[SS].pos).norm() <= REACH * 0.7 {
                                truth = k as f64 * DT;
                                break;
                            }
                        }
                        assert!(truth.is_finite(), "never arrived: wall {wall} d {d} v ({vx}, {vy})");
                        let e = est - truth;
                        if e.abs() > worst.0 {
                            worst = (e.abs(), format!("wall {wall} d {d} v ({vx}, {vy}): estimate {est:.3} s, movement {truth:.3} s"));
                        }
                    }
                }
            }
        }
        println!("worst: {}", worst.1);
        assert!(worst.0 < 0.35, "worst: {}", worst.1);
    }

    // A fielder's belief about how far off his landing point is (`landing_depth_sigma`, used
    // for calls) against the real spread: 600 reads of one fly, drawn as the simulator draws
    // them and flown forward with his physics, their landing points measured along his line
    // of sight. (The first version of the belief, a straight-line sum, was 1.1–1.4× the
    // spread here, with or without the plate/velocity cross term, part of it the spin floor
    // the sample does not contain. Codex plate-cue review #1.)
    #[test]
    fn a_fielders_doubt_matches_the_spread_of_his_reads() {
        let fx = fixture(&[]);
        let inp = hit(&fx, Vec3::new(7.0, 30.0, 24.0));
        let mut st = fx.st.clone();
        let sim = new_sim(&inp, &mut st);
        let cf = home_positions()[CF];
        let a = FielderAttrs::average(CF);
        let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
        for s in [0.7, 1.5, 2.5] {
            let t = inp.t_contact + s;
            let (p, v) = path.at(t);
            let los = Vec3::new(p.x - cf.x, p.y - cf.y, 0.0).normalized();
            let view = s - a.reaction;
            let mut depths = Vec::new();
            for k in 0..600 {
                let mut r = rng(77, 0, 0, Purpose::FielderPerception, k);
                let err = [r.truncated_normal(4.0), r.truncated_normal(4.0), r.truncated_normal(4.0), r.truncated_normal(4.0)];
                let pred = read_prediction(inp.env, inp.field, (p, v), t, cf, a.read_sigma0, a.tau_read, view, err, Some((s, inp.ball.r)));
                let (_, x, y) = pred.first_bounce().unwrap();
                depths.push(Vec3::new(x, y, 0.0).dot(los));
            }
            let mean = depths.iter().sum::<f64>() / depths.len() as f64;
            let spread = m::sqrt(depths.iter().map(|d| (d - mean) * (d - mean)).sum::<f64>() / depths.len() as f64);
            let belief = landing_depth_sigma(inp.env, inp.field, (p, v), t, cf, a.read_sigma0, a.read_vel_frac, a.tau_read, view, Some((s, inp.ball.r)));
            // This ball has the typical spin, so the spread has no spin error; the belief
            // also allows for the spin he cannot see, in the plate's share of his depth.
            let shrink = read_shrink((p, v), view, a.tau_read, Some((s, inp.ball.r)));
            let sp = a.read_sigma0 * m::hypot(p.x - cf.x, p.y - cf.y) / READ_REF_DIST * shrink;
            let (_, _, k) = plate_weighing(los, sp, READ_VEL_FRAC * v.norm() * shrink, s);
            let expect = m::hypot(spread, k * PLATE_SPIN_M * s * m::sqrt(s));
            assert!(belief > 0.85 * expect && belief < 1.15 * expect, "at {s} s: believes {belief:.2} m, spread {spread:.2} m, with spin {expect:.2} m");
        }
    }

    // Agent-models §5.7, "peels off to back up". Part 1: a pop-up coming down at (20, 36) m,
    // between first and second, that both the first and second basemen are sure of. Both call
    // it and go for it until each hears the other (`CALL_HEARD_S`); then the priority table
    // keeps it with the second baseman (neither can hear how much time the other has to
    // spare). The first baseman, called off, takes first base at once: a cover job takes a
    // called-off infielder, and first is his by the table (recheck P1-2: a man's own stop is
    // instant in his own table).
    //
    // Part 2: a fly to shallow right that the second baseman is sure of, with the right
    // fielder's sure call (tier 3) already heard. The second baseman yields, and his table
    // gives him no base (first is the first baseman's, second the shortstop's), so he backs
    // up the catch from 5 m short of where his own read has it coming down, on his side of it.
    // A re-call he is not part of leaves him that job; while the call stands he makes no glove
    // attempt; once the ball is down the job ends.
    #[test]
    fn a_called_off_infielder_covers_his_base_or_backs_up_the_catch() {
        const B1: usize = 2;
        const B2: usize = 3;
        const RF: usize = 8;
        let fx = fixture(&[]);
        let inp = hit(&fx, Vec3::new(6.61, 11.81, 27.38));
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        at_first_read(&mut sim);
        let land = landing(&sim);
        assert!((land - Vec3::new(20.0, 36.0, 0.0)).norm() < 2.0, "lands at {land:?}");
        assert_eq!((tier(&sim, B1), tier(&sim, B2)), (2, 2), "both sure");
        sim.assign();
        assert_eq!((sim.f[B1].role, sim.f[B2].role), (Role::Chase, Role::Chase), "neither has heard the other yet");
        let_calls_be_heard(&mut sim);
        assert_eq!((sim.f[B1].role, sim.f[B2].role), (Role::Cover(1), Role::Chase), "the first baseman, called off, covers first");

        let inp = hit(&fx, Vec3::new(7.0, 17.0, 22.0));
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        at_first_read(&mut sim);
        let land = landing(&sim);
        assert_eq!((tier(&sim, B2), tier(&sim, 0), tier(&sim, 5)), (2, 0, 0), "the second baseman is sure, the pitcher and shortstop are not");
        known_chaser(&mut sim, RF, 3);
        sim.assign();
        let fd = &sim.f[B2];
        assert_eq!(fd.role, Role::Backup, "2B: {:?}", sim.f.iter().map(|f| f.role).collect::<Vec<_>>());
        // Error-free read: his read of where it comes down is where it does (to the
        // intercept's 0.12 s sampling and the planning reach).
        let off = fd.target - land;
        assert!((off.norm() - PEEL_BACKUP_M).abs() < 1.5, "2B backs up from {:?}, {:.1} m from the landing", fd.target, off.norm());
        let side = Vec3::new(fd.pos.x - land.x, fd.pos.y - land.y, 0.0).normalized();
        assert!(off.normalized().dot(side) > 0.9, "on his side of it");
        // A re-call among others leaves him the job.
        let spot = fd.target;
        sim.assign_among(vec![RF, 0, 1]);
        assert_eq!((sim.f[B2].role, sim.f[B2].target), (Role::Backup, spot));
        // While the call stands he leaves the ball alone, even one that comes within his reach
        // (a misread can put him under it: Codex peel-off review #1): no glove attempt. The
        // right fielder is moved away so that only the second baseman could reach it.
        let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
        let t_down = path.bounces[0].0;
        sim.t = t_down - 0.05;
        let (b, _) = sim.ball_at(sim.t);
        assert!(b.z > 0.0 && b.z < CATCH_MAX_Z, "ball at {b:?}");
        let b2_at = sim.f[B2].pos;
        sim.f[RF].pos = Vec3::new(b.x + 10.0, b.y, 0.0);
        sim.f[B2].pos = Vec3::new(b.x, b.y, 0.0);
        sim.try_captures();
        assert_eq!((sim.f[B2].attempts, matches!(sim.ball, Ball::Free { .. })), (0, true));
        // Once the ball is down, the tick's expiry step ends the job (Codex peel-off review
        // #2): he has no spot to go to any more, and he reads the ball and takes his own job,
        // so he can field it.
        sim.f[B2].pos = b2_at;
        sim.t = t_down + DT;
        sim.expire_peels();
        assert_eq!(sim.f[B2].peel, None);
        assert_ne!((sim.f[B2].role, sim.f[B2].target), (Role::Backup, spot));
    }

    /// A slow ball loose on the left side of the infield grass at 1 s, every fielder's job
    /// dealt from nobody known to be going, and every job (none of them a chase) shown on this
    /// ball long enough ago to be known: every man has left the ball to another.
    fn everyone_left_it(sim: &mut Sim, chasing: bool) {
        loose_ball(sim, 1.0, Vec3::new(-8.0, 30.0, 0.05), Vec3::new(-1.0, 2.0, 0.0));
        sim.cover_roles(&[]);
        if chasing {
            for fd in sim.f.iter_mut() {
                fd.role = Role::Chase;
                fd.target = fd.int_pt;
            }
        }
        sim.t += CALL_HEARD_S + 2.0 * DT;
        let at = sim.t - CALL_HEARD_S - DT;
        for fd in sim.f.iter_mut() {
            let tier = if fd.role == Role::Chase { fd.call } else { 0 };
            fd.shown.push(Shown { at, role: fd.role, backs: fd.backs, tier, asks: None });
        }
    }

    /// Run the coordination steps of `simulate_live_inner` for `secs`, recording each tick's
    /// jobs after the hearing.
    fn hearing_cycles(sim: &mut Sim, secs: f64) -> Vec<Vec<Role>> {
        let mut roles = Vec::new();
        let until = sim.t + secs;
        while sim.t < until {
            sim.hear();
            sim.update_relay();
            roles.push(sim.f.iter().map(|f| f.role).collect());
            sim.publish();
            sim.move_fielders();
            sim.t += DT;
        }
        roles
    }

    // Codex fielder-communication recheck P1-2: every man has left the ball to another and
    // all know it. One man goes, by a convention all can work out (agent-models §5.7), and
    // keeps going through repeated hearings over 3 s; nobody else starts, and first base,
    // covered before, stays covered every tick.
    #[test]
    fn a_ball_everyone_left_goes_to_one_man_and_first_stays_covered() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        everyone_left_it(&mut sim, false);
        assert!(sim.f.iter().any(|f| f.role == Role::Cover(1)), "first is covered to start with");
        assert!(sim.f.iter().all(|f| f.role != Role::Chase));
        let roles = hearing_cycles(&mut sim, 3.0);
        let chasers = |r: &Vec<Role>| (0..9).filter(|&i| r[i] == Role::Chase).collect::<Vec<_>>();
        let first = chasers(&roles[0]);
        assert_eq!(first.len(), 1, "one man goes: {:?}", roles[0]);
        for (k, r) in roles.iter().enumerate() {
            assert_eq!(chasers(r), first, "tick {k}: {r:?}");
            assert!(r.contains(&Role::Cover(1)), "tick {k}: first uncovered: {r:?}");
        }
    }

    // The outfield already left a slow roller on the infield grass, and the pitcher is
    // known to be going for it. The new hearing should leave the roller to him.
    #[test]
    fn the_outfield_leaves_a_short_roller_to_the_known_infield_chaser() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        everyone_left_it(&mut sim, false);
        known_chaser(&mut sim, 0, 0);
        let roles = hearing_cycles(&mut sim, 1.0);
        for (k, r) in roles.iter().enumerate() {
            assert_eq!(r[0], Role::Chase, "tick {k}: {r:?}");
            assert!(!r[1..].contains(&Role::Chase), "tick {k}: {r:?}");
        }
    }

    // Recheck P1-2, a chase not dropped only because more men became known to be going: the
    // left and centre fielders are both going for a ball nobody catches in the air, both known
    // to be, and each one's own read has the other getting there first. They do not both
    // stop: the man first in the order they share keeps going through 3 s of hearings, and
    // nobody who stops starts again. (By their own estimates alone both stopped, nobody was
    // going, and every man decided again.)
    #[test]
    fn two_chasers_who_each_think_the_other_is_first_do_not_both_stop() {
        const LF: usize = 6;
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        loose_ball(&mut sim, 1.0, Vec3::new(-20.0, 60.0, 0.05), Vec3::new(-3.0, 4.0, 0.0));
        sim.cover_roles(&[]);
        for i in [LF, CF] {
            sim.f[i].role = Role::Chase;
            sim.f[i].t_int = 100.0;
        }
        sim.t += CALL_HEARD_S + 2.0 * DT;
        let at = sim.t - CALL_HEARD_S - DT;
        for fd in sim.f.iter_mut() {
            fd.shown.push(Shown { at, role: fd.role, backs: fd.backs, tier: 0, asks: None });
        }
        assert!(sim.beats_by_own_estimate(LF, CF) && sim.beats_by_own_estimate(CF, LF), "each thinks the other is first");
        let roles = hearing_cycles(&mut sim, 3.0);
        for (k, r) in roles.iter().enumerate() {
            assert!(r[LF] == Role::Chase || r[CF] == Role::Chase, "tick {k}: neither is going: {r:?}");
        }
        for i in 0..9 {
            if let Some(k) = roles.iter().position(|r| r[i] != Role::Chase) {
                assert!(roles[k..].iter().all(|r| r[i] != Role::Chase), "fielder {i} stopped at tick {k} and started again");
            }
        }
    }

    /// Every man known, on this ball, to be doing the job he has now and none going for it.
    fn all_known_not_going(sim: &mut Sim) {
        let at = sim.t - CALL_HEARD_S - DT;
        assert!(at >= sim.ball_since);
        for fd in sim.f.iter_mut() {
            assert_ne!(fd.role, Role::Chase);
            fd.shown.push(Shown { at, role: fd.role, backs: fd.backs, tier: 0, asks: None });
        }
    }

    // Codex fielder-communication third review P1-2, its slow-ball probe: a ball dying on the
    // infield grass at (0, 30) m, every man known to have left it to another, first covered.
    // The man sent is the one who can get to it soonest, the pitcher (2.1 s by the review's
    // estimate of his run to where it stops), not the centre fielder standing on its line
    // 60 m past where it stops (rank 0 under the first order).
    #[test]
    fn the_man_sent_for_a_ball_nobody_took_can_get_to_it() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        loose_ball(&mut sim, 1.0, Vec3::new(0.0, 30.0, 0.05), Vec3::new(0.0, 2.0, 0.0));
        sim.cover_roles(&[]);
        sim.t += CALL_HEARD_S + 2.0 * DT;
        all_known_not_going(&mut sim);
        let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
        let rest = path.at(path.t_end()).0;
        let nearest = (0..9).filter(|&j| sim.known_job(j).0 != Role::Cover(1)).min_by(|&a, &c| sim.eta(a, rest).total_cmp(&sim.eta(c, rest))).unwrap();
        assert_eq!(nearest, 0, "the pitcher is nearest where it stops");
        assert_eq!(sim.fallback_chaser(), Some(0), "the man sent");
    }

    // Opus fielder-communication fourth review P3-1: a pop-up over the mound (22 m/s at 75°
    // from the plate, the review's probe), every man known to have left it to another. The
    // shared order ties the pitcher, catcher and first baseman on the intercept scan's grid;
    // the pitcher is the last choice on a pop-up (agent-models §5.7's priority table), so
    // the tie does not go to him.
    #[test]
    fn a_tie_for_a_pop_up_nobody_took_does_not_go_to_the_pitcher() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let th = 75.0_f64.to_radians();
        loose_ball(&mut sim, 1.0, Vec3::new(0.0, 0.5, 1.0), Vec3::new(0.0, 22.0 * m::cos(th), 22.0 * m::sin(th)));
        sim.cover_roles(&[]);
        sim.t += CALL_HEARD_S + 2.0 * DT;
        all_known_not_going(&mut sim);
        let ranks: Vec<f64> = (0..9).map(|j| sim.public_rank(j)).collect();
        let best = ranks.iter().cloned().fold(f64::INFINITY, f64::min);
        assert_eq!(ranks[0], best, "the pitcher ties for first: {ranks:?}");
        let sent = sim.fallback_chaser().expect("someone goes");
        assert_eq!(ranks[sent], best);
        assert_ne!(sent, 0, "ranks {ranks:?}");
    }

    // Opus fourth review P3-4: the shared order depends on each man's velocity as well as his
    // position, so a change of velocity alone, in the same tick, is not answered from the
    // cache of the order.
    #[test]
    fn the_shared_order_follows_a_change_of_velocity() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        loose_ball(&mut sim, 1.0, Vec3::new(-8.0, 30.0, 0.05), Vec3::new(-1.0, 2.0, 0.0));
        let before = sim.public_rank(4);
        sim.f[4].vel = Vec3::new(6.0, -6.0, 0.0);
        let path = sim.public_path();
        let fresh = sim.seen_intercept(4, &path).0;
        assert!(fresh != before, "the velocity changes his intercept ({before} both)");
        assert_eq!(sim.public_rank(4), fresh);
    }

    // Third review P1-2, its fly: raw seed 2 play 140 (EV 34.75 m/s, launch 34.0°, spray
    // −9.5°), a fly to left-centre that the centre fielder catches on main. If every man has
    // left it to another, the man sent is an outfielder, not the pitcher (whom the first
    // order sent, his own intercept 12 s, and the ball fell for a hit).
    #[test]
    fn the_man_sent_for_a_fly_nobody_took_is_an_outfielder() {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let (ev, la, spray) = (34.753, 34.008_f64.to_radians(), (-9.505_f64).to_radians());
        inp.ball = BallState { r: Vec3::new(0.0, 0.5, 1.0), v: Vec3::new(ev * m::cos(la) * m::sin(spray), ev * m::cos(la) * m::cos(spray), ev * m::sin(la)), w: Vec3::ZERO };
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = inp.t_contact + 0.55;
        sim.cover_roles(&[]);
        all_known_not_going(&mut sim);
        let sent = sim.fallback_chaser().expect("someone is sent");
        assert!(sent >= 6, "sent f{sent}");
    }

    // Third review P2-1: the double-play pivot grounder with error-free reads; the second
    // baseman known idle and the shortstop known covering second, on this ball. Their reads
    // agree on who gets to it first, so one goes and the other stays at second.
    #[test]
    fn error_free_middle_infielders_keep_one_at_second() {
        const B2: usize = 3;
        const SS: usize = 5;
        let mut fx = fixture(&[(5, 1)]);
        fx.st.outs = 1;
        let mut st = fx.st.clone();
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.ball = BallState { r: Vec3::new(0.0, 0.5, 0.5), v: Vec3::new(-1.5, 24.0, -3.0), w: Vec3::ZERO };
        let mut sim = new_sim(&inp, &mut st);
        for f in sim.f.iter_mut() {
            f.err = [0.0; 4];
        }
        sim.t = inp.t_contact + 0.6;
        known_doing(&mut sim, B2, Role::Idle);
        known_doing(&mut sim, SS, Role::Cover(2));
        for i in [B2, SS] {
            sim.read(i);
            sim.f[i].call = 0;
        }
        let d = (sim.decide(B2), sim.decide(SS));
        assert!(matches!(d, (Decision::Go, Decision::CoverSecond) | (Decision::CoverSecond, Decision::Go)), "2B, SS: {d:?} (fielding times {:.3}, {:.3})", sim.f[B2].t_int, sim.f[SS].t_int);
    }

    /// What the isolation harness compares of fielder `k` after a step: everything he is,
    /// does and has shown (his job, target, backup, call, plans and clocks, his publications),
    /// with his read sampled where the ball is now and where his read has it end. Compared as
    /// values; formatted only to report a difference.
    #[allow(clippy::type_complexity)]
    fn iso_fielder(sim: &Sim, k: usize) -> impl PartialEq + std::fmt::Debug {
        let f = &sim.f[k];
        (
            (f.pos, f.vel, f.jump, f.role, f.target, f.next_plan, f.err, f.attempts, f.blocked_until, f.ready_at, f.dove, f.lunge),
            (f.t_int, f.int_pt, f.read_at, f.read_ball, f.call, f.shown.clone(), f.last_close_read, f.must_dive_since, f.last_dive_read, f.will_dive.get(), f.throws, f.up_at),
            (f.set_base, f.face, f.catch_vel, f.took_diving, f.catch_stretched, f.backs, f.peel, f.carry_to, f.asks, f.pivot, f.pivot_wait_bag, f.relayed),
            f.belief.as_ref().map(|p| (p.at(sim.t), p.at(p.t_end()), p.t_end())),
        )
    }

    /// `iso_fielder` as text (for the reported differences and the third review's probes).
    fn iso_state(sim: &Sim, k: usize) -> String {
        format!("{:?}", iso_fielder(sim, k))
    }

    /// The ball as the isolation harness compares it: loose (its path sampled, and its
    /// bookkeeping), held by whom, or thrown (the whole throw: its path's ends, its aim, its
    /// receiver's read of it and the base and man it is for).
    fn iso_ball(sim: &Sim) -> String {
        match &sim.ball {
            Ball::Free { path, next_touch, next_wall, settled_sent } => format!("free {:?} {:?}", (path.at(sim.t), path.at(path.t_end()), path.t_end()), (next_touch, next_wall, settled_sent)),
            Ball::Held { by } => format!("held {by}"),
            Ball::Thrown { path, aim, catch_at, read_at, read_err, next_read, to_base, receiver, relay } => {
                format!("thrown {:?} {:?}", (path.first(), path.last()), (aim, catch_at, read_at, read_err, next_read, to_base, receiver, relay))
            }
        }
    }

    /// Everything the harness compares after a step but fielder `j`: every other man (as
    /// `iso_fielder`), the ball, the runners, and (at the holder's step, for another man's
    /// change) the holder's options. Kept as the step left it; compared field by field.
    struct IsoWorld<'s, 'a> {
        sim: Sim<'s>,
        j: usize,
        step: &'static str,
        _st: std::marker::PhantomData<&'a ()>,
    }

    impl IsoWorld<'_, '_> {
        fn holder_options(&self) -> Option<String> {
            match self.sim.ball {
                Ball::Held { by } if self.step == "holder_act" && by != self.j => Some(format!("{:?}", self.sim.holder_options(by))),
                _ => None,
            }
        }

        /// The differences from `o`, as `name: self -> o`.
        fn diff(&self, o: &IsoWorld) -> Vec<String> {
            let mut d: Vec<String> = (0..9)
                .filter(|&k| k != self.j && iso_fielder(&self.sim, k) != iso_fielder(&o.sim, k))
                .map(|k| format!("f{k}: {} -> {}", iso_state(&self.sim, k), iso_state(&o.sim, k)))
                .collect();
            let (ba, bb) = (iso_ball(&self.sim), iso_ball(&o.sim));
            if ba != bb {
                d.push(format!("ball: {ba} -> {bb}"));
            }
            let (ra, rb) = (format!("{:?}", self.sim.r), format!("{:?}", o.sim.r));
            if ra != rb {
                d.push(format!("runners: {ra} -> {rb}"));
            }
            let (ha, hb) = (self.holder_options(), o.holder_options());
            if ha != hb {
                d.push(format!("holder options: {ha:?} -> {hb:?}"));
            }
            d
        }
    }

    type IsoFlip = Box<dyn Fn(&mut Fielder)>;

    /// The private changes the harness makes to fielder `j`, one at a time. None of them is
    /// shown: his publications stay as they are. Every job and backup he could have; where he
    /// is going and his plan; his backup of a catch; each call; his read cleared, or replaced
    /// by another valid read of the ball now (perfect, or with his error draws at ±2 σ, with
    /// the call it gives), which can put him on either side of the thresholds the decisions
    /// use; his error draws; his planning clocks (replan, close read, dive read); a dive
    /// decided; a carry; a holder's set-up clocks and his call for a man to a bag.
    fn iso_flips(sim: &Sim, j: usize) -> Vec<(String, IsoFlip)> {
        let mut v: Vec<(String, IsoFlip)> = Vec::new();
        let roles = [Role::Idle, Role::Chase, Role::Cover(1), Role::Cover(2), Role::Cover(3), Role::Cover(4), Role::Relay(1), Role::Relay(2), Role::Relay(3), Role::Relay(4), Role::Backup];
        for r in roles {
            if r != sim.f[j].role {
                v.push((format!("role {r:?}"), Box::new(move |f: &mut Fielder| f.role = r)));
            }
        }
        for b in [None, Some(1), Some(2), Some(3), Some(4)] {
            if b != sim.f[j].backs {
                v.push((format!("backs {b:?}"), Box::new(move |f: &mut Fielder| f.backs = b)));
            }
        }
        v.push(("target".into(), Box::new(|f: &mut Fielder| f.target = f.target + Vec3::new(5.0, 5.0, 0.0))));
        v.push(("plan".into(), Box::new(|f: &mut Fielder| {
            f.int_pt = f.int_pt + Vec3::new(5.0, 0.0, 0.0);
            f.t_int += 1.0;
        })));
        v.push(("peel".into(), Box::new(|f: &mut Fielder| f.peel = if f.peel.is_some() { None } else { Some(f.pos) })));
        for c in 0..=3u8 {
            if c != sim.f[j].call {
                v.push((format!("call {c}"), Box::new(move |f: &mut Fielder| f.call = c)));
            }
        }
        v.push(("read cleared".into(), Box::new(|f: &mut Fielder| {
            f.belief = None;
            f.t_int = f64::INFINITY;
            f.read_at = f64::NEG_INFINITY;
        })));
        if matches!(sim.ball, Ball::Free { .. }) {
            for (name, err) in [("perfect", [0.0; 4]), ("+2σ", [2.0; 4]), ("-2σ", [-2.0; 4])] {
                let mut st = sim.st.clone();
                let mut g = sim.fork(&mut st);
                g.f[j].err = err;
                let p = g.predict(j);
                let (ti, pt) = g.intercept(j, &p);
                let bias = if j >= 6 { -0.15 } else { 0.0 };
                let call = g.call_value(j, &p, ti, pt, bias).0;
                let (t, ball) = (sim.t, sim.ball_gen);
                v.push((format!("read {name} (call {call})"), Box::new(move |f: &mut Fielder| {
                    (f.belief, f.t_int, f.int_pt, f.call, f.read_at, f.read_ball) = (Some(p.clone()), ti, pt, call, t, ball);
                })));
            }
        }
        v.push(("err".into(), Box::new(|f: &mut Fielder| f.err = [4.0; 4])));
        v.push(("read_ball".into(), Box::new(|f: &mut Fielder| f.read_ball = u64::MAX)));
        v.push(("next_plan now".into(), Box::new(|f: &mut Fielder| f.next_plan = f64::NEG_INFINITY)));
        v.push(("next_plan never".into(), Box::new(|f: &mut Fielder| f.next_plan = f64::INFINITY)));
        v.push(("close read due".into(), Box::new(|f: &mut Fielder| f.last_close_read = f64::NEG_INFINITY)));
        v.push(("close read not due".into(), Box::new(|f: &mut Fielder| f.last_close_read = f64::INFINITY)));
        v.push(("dive read due".into(), Box::new(|f: &mut Fielder| f.last_dive_read = f64::NEG_INFINITY)));
        v.push(("must dive".into(), Box::new(|f: &mut Fielder| f.must_dive_since = if f.must_dive_since.is_finite() { f64::INFINITY } else { f64::NEG_INFINITY })));
        v.push(("will dive".into(), Box::new(|f: &mut Fielder| f.will_dive.set(!f.will_dive.get()))));
        v.push(("jump".into(), Box::new(|f: &mut Fielder| f.jump = f.jump + Vec3::new(3.0, -3.0, 0.0))));
        v.push(("carry_to".into(), Box::new(|f: &mut Fielder| f.carry_to = if f.carry_to.is_some() { None } else { Some(4) })));
        v.push(("set_base".into(), Box::new(|f: &mut Fielder| f.set_base += 1.0)));
        v.push(("up_at".into(), Box::new(|f: &mut Fielder| f.up_at += 0.5)));
        v.push(("asks".into(), Box::new(|f: &mut Fielder| f.asks = if f.asks.is_some() { None } else { Some((2, 5)) })));
        v
    }

    /// Run `step` on a fork of `sim` (with its own rules state `st`), with `flip` applied to
    /// fielder `j` first. Also whether he still holds the ball.
    fn iso_run<'s>(sim: &'s Sim, st: &'s mut RulesState, step: &'static str, j: usize, flip: Option<&dyn Fn(&mut Fielder)>) -> (IsoWorld<'s, 's>, bool) {
        let mut b = sim.fork(st);
        if let Some(flip) = flip {
            flip(&mut b.f[j]);
        }
        b.run_step(step);
        let held = matches!(b.ball, Ball::Held { by } if by == j);
        (IsoWorld { sim: b, j, step, _st: std::marker::PhantomData }, held)
    }

    /// Fork `sim`, apply `flip` to fielder `j` in one fork, run `step` on both, and say what
    /// else came out differently.
    fn iso_compare(sim: &Sim, step: &'static str, j: usize, flip: &dyn Fn(&mut Fielder)) -> Vec<String> {
        let (mut sa, mut sb) = (sim.st.clone(), sim.st.clone());
        let (a, _) = iso_run(sim, &mut sa, step, j, None);
        let (b, _) = iso_run(sim, &mut sb, step, j, Some(flip));
        a.diff(&b)
    }

    /// `iso_compare` for the holder at his own step: differences count only when the ball is
    /// still in his hands in both forks (a release is a public event).
    fn iso_compare_pre_release(sim: &Sim, step: &'static str, j: usize, flip: &dyn Fn(&mut Fielder)) -> Vec<String> {
        let (mut sa, mut sb) = (sim.st.clone(), sim.st.clone());
        let (a, held_a) = iso_run(sim, &mut sa, step, j, None);
        let (b, held_b) = iso_run(sim, &mut sb, step, j, Some(flip));
        if !held_a || !held_b {
            return Vec::new();
        }
        a.diff(&b)
    }

    /// What a step changed of the jobs: anyone's role, backup, call, backup of a catch, call
    /// for a man to a bag, publications, or the ball (not a target, which a cutoff man's line
    /// moves every tick). A step that changes them is a transition, checked every time.
    fn iso_jobs(sim: &Sim) -> String {
        format!("{:?} {}", sim.f.iter().map(|f| (f.role, f.backs, f.call, f.peel, f.asks, f.shown.len())).collect::<Vec<_>>(), iso_ball(sim))
    }

    /// Steps the harness checks at every call: rare, and each one a transition.
    const ISO_DENSE_STEPS: [&str; 3] = ["new_ball_assign", "back_up_throw", "recall"];

    /// Run games `seeds` with the harness: before each decision step of a live play, at every
    /// `stride`-th call of the step, at every call of a rare step (`ISO_DENSE_STEPS`), and
    /// at every call that changes anyone's job (a transition), for every fielder and every
    /// private change (`iso_flips`), nothing else (`iso_world`) may change in that step. The
    /// holder's own change at his step may decide his throw: what follows a release is public
    /// (`iso_compare_pre_release`). Returns the checks made, the snapshots checked by step,
    /// and the violations.
    fn isolation_run(seeds: std::ops::Range<u64>, plays: std::ops::Range<u32>, stride: u64) -> (u64, std::collections::BTreeMap<&'static str, u64>, Vec<String>) {
        use std::cell::RefCell;
        use std::rc::Rc;
        let found: Rc<RefCell<(u64, Vec<String>)>> = Rc::new(RefCell::new((0, Vec::new())));
        let calls: Rc<RefCell<std::collections::BTreeMap<&'static str, u64>>> = Rc::default();
        let snaps: Rc<RefCell<std::collections::BTreeMap<&'static str, u64>>> = Rc::default();
        let (f2, c2, s2) = (found.clone(), calls.clone(), snaps.clone());
        ISO_HOOK.with(|h| {
            *h.borrow_mut() = Some(Box::new(move |sim: &Sim<'_>, step: &'static str| {
                if !plays.contains(&sim.inp.play) {
                    return;
                }
                let n = {
                    let mut c = c2.borrow_mut();
                    let e = c.entry(step).or_insert(0);
                    *e += 1;
                    *e
                };
                let due = n % stride == 0 || ISO_DENSE_STEPS.contains(&step) || {
                    let mut st = sim.st.clone();
                    let mut a = sim.fork(&mut st);
                    a.run_step(step);
                    iso_jobs(&a) != iso_jobs(sim)
                };
                if !due {
                    return;
                }
                *s2.borrow_mut().entry(step).or_insert(0) += 1;
                let holder = match sim.ball {
                    Ball::Held { by } => Some(by),
                    _ => None,
                };
                for j in 0..9 {
                    // One unchanged run per man (it leaves him out of the comparison).
                    let mut sa = sim.st.clone();
                    let (a, held_a) = iso_run(sim, &mut sa, step, j, None);
                    let own_throw = step == "holder_act" && holder == Some(j);
                    for (what, flip) in iso_flips(sim, j) {
                        let mut sb = sim.st.clone();
                        let (b, held_b) = iso_run(sim, &mut sb, step, j, Some(flip.as_ref()));
                        let d = if own_throw && (!held_a || !held_b) { Vec::new() } else { a.diff(&b) };
                        let mut fr = f2.borrow_mut();
                        fr.0 += 1;
                        if !d.is_empty() {
                            let first: String = d[0].chars().take(240).collect();
                            fr.1.push(format!("seed {} play {} t {:.6} {step}: f{j} {what}: {} change(s), {first}", sim.inp.seed, sim.inp.play, sim.t, d.len()));
                        }
                    }
                }
            }));
        });
        for seed in seeds {
            crate::game::simulate_game(seed);
        }
        ISO_HOOK.with(|h| *h.borrow_mut() = None);
        let r = found.borrow().clone();
        let s = snaps.borrow().clone();
        (r.0, s, r.1)
    }

    /// The harness's settings from the environment: `ISO_SEEDS=a..b`, `ISO_STRIDE=n`.
    fn iso_settings(seeds: std::ops::Range<u64>, stride: u64) -> (std::ops::Range<u64>, u64) {
        let seeds = std::env::var("ISO_SEEDS").ok().and_then(|s| s.split_once("..").map(|(a, b)| a.parse().unwrap()..b.parse().unwrap())).unwrap_or(seeds);
        let stride = std::env::var("ISO_STRIDE").ok().map(|s| s.parse().unwrap()).unwrap_or(stride);
        (seeds, stride)
    }

    fn iso_assert(seeds: std::ops::Range<u64>, stride: u64, min_checks: u64) {
        let (checks, snaps, bad) = isolation_run(seeds.clone(), 0..u32::MAX, stride);
        // Violations by step, and the snapshots (seed, play, time, step) they occurred at.
        let mut by_step: std::collections::BTreeMap<&str, u64> = Default::default();
        let mut at: std::collections::BTreeSet<&str> = Default::default();
        for b in &bad {
            let head = b.split(": f").next().unwrap_or(b);
            at.insert(head);
            *by_step.entry(head.rsplit(' ').next().unwrap_or("")).or_insert(0) += 1;
        }
        eprintln!("isolation seeds {seeds:?} stride {stride}: {checks} checks, {} violations at {} snapshots, by step {by_step:?}; snapshots checked by step {snaps:?}", bad.len(), at.len());
        assert!(checks > min_checks, "only {checks} checks");
        let shown: Vec<&String> = bad.iter().take(12).collect();
        assert!(bad.is_empty(), "{} of {checks} checks leaked, e.g.:\n{}", bad.len(), shown.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n"));
    }

    // Codex fielder-communication third review P1-1, widened after the fourth reviews (Codex
    // P2-1, Opus P2-2): the general isolation harness on real games. Before every decision
    // step of a live play (the first read; a new loose ball's read; the chasers' replans; the
    // close and dive reads; the re-call; the end of a backup of a catch; a chaser seeing the
    // ball secured; the receiver's read of a throw; the holder's choice; his call heeded; the
    // backup of a released throw; the catcher's return home; hearing; the cutoff job), each
    // man's private state is changed in turn (`iso_flips`; never what he has shown) and
    // nothing else (`iso_world`: every other man's whole state and publications, the ball,
    // the runners, the holder's options) may change in that step. Rare steps and every step
    // that changes a job are checked every time, others every `stride`-th call. The default
    // test covers seeds 0–1. The ignored `isolation_many_seeds` test covers 0–49 and can
    // run as a negative control with `ISO_REVERT=1`. Both take `ISO_SEEDS` and `ISO_STRIDE`.
    #[test]
    fn isolation_no_private_state_reaches_a_teammate_in_real_games() {
        let (seeds, stride) = iso_settings(0..2, 400);
        iso_assert(seeds, stride, 100_000);
    }

    // The harness's negative control (fourth review P2-1): with the cutoff step behaving as
    // on `f3d9e7a` (a man's private leaving or taking of the cutoff job re-works every
    // teammate's cover, Codex third review P1-1), the harness must find leaks, on the first
    // 30 plays of seed 0 at its default stride.
    #[test]
    fn isolation_harness_finds_the_third_reviews_gate() {
        ISO_REVERT_GATE.with(|c| c.set(true));
        let (checks, snaps, bad) = isolation_run(0..1, 0..30, 400);
        ISO_REVERT_GATE.with(|c| c.set(false));
        eprintln!("negative control: {checks} checks, {} violations; snapshots by step {snaps:?}", bad.len());
        let first: Vec<&str> = bad.iter().take(3).map(|s| s.as_str()).collect();
        eprintln!("{}", first.join("\n"));
        assert!(!bad.is_empty(), "the harness missed the restored gate in {checks} checks");
        assert!(bad.iter().all(|b| b.contains("update_relay")), "leaks outside the cutoff step: {:?}", bad.iter().find(|b| !b.contains("update_relay")));
    }

    #[test]
    #[ignore = "long: the isolation harness over seeds 0-49 (about an hour on one core)"]
    fn isolation_many_seeds() {
        // `ISO_REVERT=1` runs it as the negative control (`ISO_REVERT_GATE`): it must fail.
        ISO_REVERT_GATE.with(|c| c.set(std::env::var("ISO_REVERT").is_ok()));
        let (seeds, stride) = iso_settings(0..50, 200);
        iso_assert(seeds, stride, 500_000);
    }

    // In seed 0, change only the second baseman's private job just before the first cutoff
    // update with him idle and the catcher backing up. Nobody else's job may change.
    #[test]
    fn isolation_seed_0_a_private_cutoff_job_moves_nobody() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let hit: Rc<RefCell<Option<(Vec<String>, Vec<String>)>>> = Rc::default();
        let h2 = hit.clone();
        ISO_HOOK.with(|h| {
            *h.borrow_mut() = Some(Box::new(move |sim: &Sim<'_>, step: &'static str| {
                if h2.borrow().is_none() && sim.inp.seed == 0 && step == "update_relay" && sim.f[3].role == Role::Idle && sim.f[1].role == Role::Backup {
                    let flip = |f: &mut Fielder| f.role = Role::Relay(4);
                    let d = iso_compare(sim, step, 3, &flip);
                    ISO_REVERT_GATE.with(|c| c.set(true));
                    let control = iso_compare(sim, step, 3, &flip);
                    ISO_REVERT_GATE.with(|c| c.set(false));
                    *h2.borrow_mut() = Some((d, control));
                }
            }));
        });
        crate::game::simulate_game(0);
        ISO_HOOK.with(|h| *h.borrow_mut() = None);
        let (d, control) = hit.borrow().clone().expect("the snapshot was reached");
        assert!(d.is_empty(), "{}", d.join("; "));
        assert!(!control.is_empty(), "the restored gate does not leak here: the snapshot tests nothing");
    }


    /// Fourth review P1-1: in real game `seed`, at each `holder_act` of play `play` from
    /// `t_from` for `span` seconds, only the holder's private set-up clock changes (no motion,
    /// nothing shown). While the ball is in his hands in both forks, no teammate may change.
    /// Returns the snapshots seen and the leaks.
    fn holder_snapshot_leaks(seed: u64, play: u32, t_from: f64, span: f64) -> (u32, Vec<String>) {
        use std::cell::RefCell;
        use std::rc::Rc;
        let found: Rc<RefCell<(u32, Vec<String>)>> = Rc::default();
        let f2 = found.clone();
        ISO_HOOK.with(|h| {
            *h.borrow_mut() = Some(Box::new(move |sim: &Sim<'_>, step: &'static str| {
                if sim.inp.seed != seed || sim.inp.play != play || step != "holder_act" || sim.t < t_from - 1e-6 || sim.t > t_from + span {
                    return;
                }
                let Ball::Held { by } = sim.ball else { return };
                let d = iso_compare_pre_release(sim, step, by, &|f: &mut Fielder| f.set_base += 1.0);
                let mut fr = f2.borrow_mut();
                fr.0 += 1;
                fr.1.extend(d.into_iter().map(|x| format!("t {:.9} holder f{by}: {x}", sim.t)));
            }));
        });
        crate::game::simulate_game(seed);
        ISO_HOOK.with(|h| *h.borrow_mut() = None);
        let r = found.borrow().clone();
        r
    }

    /// The first (play, time) in real game `seed` at a `holder_act` where `pred` holds of the
    /// sim: how the pinned snapshots below are found again when a change to the runners moves
    /// the games under them.
    fn first_holder_act(seed: u64, pred: impl Fn(&Sim<'_>) -> bool + 'static) -> (u32, f64) {
        use std::cell::RefCell;
        use std::rc::Rc;
        let first: Rc<RefCell<Option<(u32, f64)>>> = Rc::default();
        let f2 = first.clone();
        ISO_HOOK.with(|h| {
            *h.borrow_mut() = Some(Box::new(move |sim: &Sim<'_>, step: &'static str| {
                let mut f = f2.borrow_mut();
                if f.is_none() && sim.inp.seed == seed && step == "holder_act" && pred(sim) {
                    *f = Some((sim.inp.play, sim.t));
                }
            }));
        });
        crate::game::simulate_game(seed);
        ISO_HOOK.with(|h| *h.borrow_mut() = None);
        let r = first.borrow().expect("the situation arises in the game");
        r
    }

    // Codex fielder-communication fourth review P1-1, its first counterexample: seed 1 play
    // 195, the centre fielder holds the ball and waits on a man for second. The left fielder,
    // known to be backing up second, was sent to the bag (13 m) by the holder's private
    // choice, before any throw or call. (Found afresh 2026-11-15, when the runners' looking
    // again moved the game: the first time in seed 1 an outfielder holds the ball with a
    // fielder backing up.)
    #[test]
    fn a_holders_private_choice_moves_nobody_seed_1_outfield_holder_with_a_backup() {
        let (play, t) = first_holder_act(1, |sim| matches!(sim.ball, Ball::Held { by } if by >= 6) && sim.f.iter().any(|x| matches!(x.role, Role::Backup)));
        let (seen, leaks) = holder_snapshot_leaks(1, play, t, 0.05);
        assert!(seen > 0, "the snapshot was not reached");
        assert!(leaks.is_empty(), "play {play} from {t:.4}: {}", leaks.join("\n"));
    }

    // In seed 8, check that an outfield holder's private set-up clock cannot send a cutoff
    // man away from his assignment.
    #[test]
    fn a_holders_private_choice_moves_nobody_seed_8_outfield_holder_with_a_cutoff() {
        let (play, t) = first_holder_act(8, |sim| matches!(sim.ball, Ball::Held { by } if by >= 6) && sim.f.iter().any(|x| matches!(x.role, Role::Relay(_))));
        let (seen, leaks) = holder_snapshot_leaks(8, play, t, 0.2);
        assert!(seen > 0, "the snapshot was not reached");
        assert!(leaks.is_empty(), "play {play} from {t:.4}: {}", leaks.join("\n"));
    }

    // Fourth review P1-1, the call itself: the holder waits on a man for second and calls
    // him. The man goes to the bag only once the call is heard, `CALL_HEARD_S` later.
    #[test]
    fn a_holders_call_to_the_bag_is_heard_before_it_is_heeded() {
        const LF: usize = 6;
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 3.0;
        sim.ball = Ball::Held { by: CF };
        sim.secured_at = 2.5;
        sim.f[LF].role = Role::Backup;
        let spot = Vec3::new(-5.0, 27.0, 0.0);
        sim.f[LF].target = spot;
        sim.f[CF].asks = Some((2, LF));
        sim.publish();
        sim.heed_requests();
        assert_eq!(sim.f[LF].target, spot, "he heard nothing yet");
        sim.t += CALL_HEARD_S - 2.0 * DT;
        sim.heed_requests();
        assert_eq!(sim.f[LF].target, spot, "he heard nothing yet");
        sim.t += 3.0 * DT;
        sim.heed_requests();
        assert_eq!(sim.f[LF].target, base_pos(2), "he heard the call and goes to second");
    }

    // Third review P1-1, its constructed probe: the third baseman holds the ball, the first
    // baseman is known to be chasing, the pitcher idle. Only the shortstop's private job
    // changes, idle or cutoff to home (he is known idle): the pitcher's job must not change.
    #[test]
    fn isolation_a_private_cutoff_retiring_moves_nobody() {
        let fx = fixture(&[(0x0007, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0007, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut jobs = Vec::new();
        for role in [Role::Idle, Role::Relay(4)] {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = 2.0;
            sim.ball = Ball::Held { by: 4 };
            sim.f[4].pos = Vec3::new(-20.0, 25.0, 0.0);
            known_chaser(&mut sim, 2, 0);
            sim.f[0].role = Role::Idle;
            sim.f[5].role = role;
            assert_eq!(sim.known_job(5), (Role::Idle, None));
            sim.update_relay();
            jobs.push((sim.f[0].role, sim.f[0].target));
        }
        assert_eq!(jobs[0], jobs[1], "the pitcher's job followed the shortstop's unheard cutoff job");
    }

    // Codex fielder-communication recheck P1-1, first probe: the centre fielder holds the ball
    // deep, the second baseman stands between him and home, a runner on second. Only the
    // second baseman's private job changes, idle or cutoff to home, and nobody knows it yet:
    // the holder's choice of route home must not change with it.
    #[test]
    fn the_holders_route_does_not_read_an_unheard_cutoff() {
        const B2: usize = 3;
        let fx = fixture(&[(0x0007, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0007, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut races = Vec::new();
        for role in [Role::Idle, Role::Relay(4)] {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = 1.0;
            sim.ball = Ball::Held { by: CF };
            sim.f[CF].pos = Vec3::new(0.0, 125.0, 0.0);
            sim.f[B2].pos = Vec3::new(0.0, 60.0, 0.0);
            sim.f[B2].role = role;
            assert_eq!(sim.known_job(B2), (Role::Idle, None), "his job is not known");
            let r = sim.throw_race(CF, 4);
            races.push((r.relay, r.through, r.ball_at, r.via));
        }
        assert_eq!(races[0], races[1], "the holder's route read the second baseman's unheard cutoff job");
    }

    // Codex fielder-communication recheck P1-1, second probe: the centre fielder holds the ball
    // at (45, 55) with a runner on second. Only the shortstop's private job changes, idle or
    // cutoff to home, and nobody knows it yet: the first baseman's job must not change with it.
    #[test]
    fn the_cutoff_job_does_not_read_an_unheard_cutoff() {
        const B1: usize = 2;
        const SS: usize = 5;
        let fx = fixture(&[(0x0007, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0007, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut jobs = Vec::new();
        for role in [Role::Idle, Role::Relay(4)] {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = 1.0;
            sim.ball = Ball::Held { by: CF };
            sim.f[CF].pos = Vec3::new(45.0, 55.0, 0.0);
            sim.cover_roles(&[]);
            assert_eq!(sim.known_job(SS), (Role::Idle, None), "his job is not known");
            sim.f[SS].role = role;
            sim.update_relay();
            jobs.push((sim.f[B1].role, sim.f[B1].target));
        }
        assert_eq!(jobs[0], jobs[1], "the first baseman's job read the shortstop's unheard cutoff job");
    }

    // Codex fielder-communication recheck P1-1, third probe: a ball on the ground; the
    // pitcher is backing up, and only whether that backup is a called-off man's peel (his
    // private marker; the job shown is the same) changes. His teammates' reads and jobs must
    // not change with it.
    #[test]
    fn a_peel_ending_does_not_move_teammates() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut seen = Vec::new();
        for peeled in [false, true] {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            loose_ball(&mut sim, 1.0, Vec3::new(0.0, 25.0, 0.05), Vec3::new(1.0, 10.0, -2.0));
            sim.t += 0.2;
            sim.f[0].role = Role::Backup;
            if peeled {
                sim.f[0].peel = Some(Vec3::new(0.0, 20.0, 0.0));
            }
            assert!(!sim.ball_still_up());
            assert_eq!(sim.known_job(0), (Role::Idle, None));
            sim.expire_peels();
            seen.push((1..9).map(|i| (sim.f[i].role, sim.f[i].target, sim.f[i].read_at)).collect::<Vec<_>>());
        }
        assert_eq!(seen[0], seen[1], "the pitcher's private peel moved his teammates");
    }

    // Codex fielder-communication recheck P2-1: two new loose balls in one tick (a throw that
    // comes down, then a rim off a glove) are two balls to read, though they share the tick.
    #[test]
    fn a_second_new_ball_in_one_tick_is_read_again() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 1.0;
        let b = Vec3::new(0.0, 35.0, 0.05);
        sim.set_free(BallState { r: b, v: Vec3::new(0.0, 25.0, -1.0), w: Vec3::ZERO });
        assert!(sim.f[CF].int_pt.y > 45.0, "the first ball goes deep: {:?}", sim.f[CF].int_pt);
        sim.set_free(BallState { r: b, v: Vec3::new(0.0, 1.0, 1.0), w: Vec3::ZERO });
        let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
        assert!(path.at(path.t_end()).0.y < 40.0);
        assert!(sim.f[CF].int_pt.y < 40.0, "the second ball was not read: {:?}", sim.f[CF].int_pt);
    }

    // The same through the production path (recheck P2-1): a throw that comes down short
    // becomes a loose ball (`update_thrown`), and in the same tick the pitcher, standing where
    // it is, gets a glove on it and it comes off the rim (`try_captures`; his hands are set so
    // the keyed glove placement lands on the rim). The centre fielder's plan must be for the
    // ball off the rim.
    #[test]
    fn a_grounded_throw_rimmed_in_the_same_tick_is_read_again() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 1.0;
        sim.launch_throw(BallState { r: Vec3::new(0.0, 20.0, 1.7), v: Vec3::new(0.0, 25.0, -5.0), w: Vec3::ZERO }, Vec3::new(0.0, 35.0, 0.0), 2, 3, false);
        let (at, b) = match &sim.ball {
            Ball::Thrown { path, .. } => (path.last().t, path.last().s.r),
            _ => unreachable!(),
        };
        sim.t = at;
        sim.f[0].pos = Vec3::new(b.x, b.y, 0.0);
        sim.f[0].vel = Vec3::ZERO;
        sim.update_thrown();
        let first = match &sim.ball {
            Ball::Free { path, .. } => path.at(path.t_end()).0,
            _ => panic!("the throw did not come loose"),
        };
        let (bp, bv) = sim.ball_at(sim.t);
        sim.f[0].pos = Vec3::new(bp.x, bp.y, 0.0);
        let mut r = rng(inp.seed, inp.play, sim.f[0].id, Purpose::GlovePlacement, sim.f[0].attempts);
        let (ex, ez) = (r.truncated_normal(4.0), r.truncated_normal(4.0));
        sim.f[0].a.hands = 0.5 * (POCKET + RIM) / (glove_sigma_mult(bv.norm(), 0.0, false) * m::hypot(ex, ez));
        sim.try_captures();
        assert_eq!(sim.f[0].attempts, 1, "a glove attempt");
        let second = match &sim.ball {
            Ball::Free { path, .. } => path.at(path.t_end()).0,
            _ => panic!("expected the ball off the rim"),
        };
        assert!((first - second).norm() > 10.0, "two different loose balls: {first:?} {second:?}");
        let pt = sim.f[CF].int_pt;
        assert!((Vec3::new(pt.x, pt.y, 0.0) - Vec3::new(second.x, second.y, 0.0)).norm() < (Vec3::new(pt.x, pt.y, 0.0) - Vec3::new(first.x, first.y, 0.0)).norm(), "his plan {pt:?} is for the first ball {first:?}, not the one off the rim {second:?}");
    }

    // Codex fielder-communication review P2-2: a fly to straightaway centre with the centre
    // fielder far away, and the two corner outfielders 10 m apart under it, both reading it
    // without error. Both are sure (tier 3) and share a priority, so the convention decides
    // (agent-models §5.7: left field over right). Until the calls are heard both go; once each
    // has heard the other, exactly one goes on (the left fielder), and the right fielder,
    // the nearest free outfielder, backs him up. It stays so.
    #[test]
    fn two_sure_corner_outfielders_resolve_their_calls() {
        const LF: usize = 6;
        const RF: usize = 8;
        let fx = fixture(&[]);
        let mut inp = hit(&fx, Vec3::new(0.0, 33.0, 22.0));
        inp.fattrs[LF].read_sigma0 = 0.01;
        inp.fattrs[RF].read_sigma0 = 0.01;
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.f[LF].pos = Vec3::new(-5.0, 92.0, 0.0);
        sim.f[RF].pos = Vec3::new(5.0, 92.0, 0.0);
        sim.f[CF].pos = Vec3::new(70.0, 60.0, 0.0);
        for fd in sim.f.iter_mut() {
            fd.jump = Vec3::ZERO;
        }
        at_first_read(&mut sim);
        sim.t = sim.inp.t_contact + 0.6;
        assert_eq!((tier(&sim, LF), tier(&sim, RF)), (3, 3), "both sure");
        sim.assign();
        assert_eq!((sim.f[LF].role, sim.f[RF].role), (Role::Chase, Role::Chase), "neither has heard the other yet");
        let_calls_be_heard(&mut sim);
        assert_eq!((sim.f[LF].role, sim.f[RF].role), (Role::Chase, Role::Backup), "one goes on, the other backs him up");
        let_calls_be_heard(&mut sim);
        let_calls_be_heard(&mut sim);
        assert_eq!((sim.f[LF].role, sim.f[RF].role), (Role::Chase, Role::Backup), "and it stays so");
    }

    // Codex fielder-communication fourth review P2-2: the left fielder sure-calls the fly of
    // `two_sure_corner_outfielders_resolve_their_calls` and the right fielder backs him up.
    // Then the left fielder is 45 m off it (as if he had broken the wrong way): his next read
    // has no catch in the air for him, and by agent-models §5.7 a man calls only a ball he
    // believes he catches in the air, so he stops calling it. The right fielder still hears
    // the old call until `CALL_HEARD_S` has passed, and keeps backing up; then, nobody's
    // sure call standing and his own read still sure, he goes for it.
    #[test]
    fn a_sure_caller_who_loses_his_air_play_withdraws_his_call_and_the_backup_takes_it() {
        const LF: usize = 6;
        const RF: usize = 8;
        let fx = fixture(&[]);
        let mut inp = hit(&fx, Vec3::new(0.0, 33.0, 22.0));
        inp.fattrs[LF].read_sigma0 = 0.01;
        inp.fattrs[RF].read_sigma0 = 0.01;
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.f[LF].pos = Vec3::new(-5.0, 92.0, 0.0);
        sim.f[RF].pos = Vec3::new(5.0, 92.0, 0.0);
        sim.f[CF].pos = Vec3::new(70.0, 60.0, 0.0);
        for fd in sim.f.iter_mut() {
            fd.jump = Vec3::ZERO;
        }
        at_first_read(&mut sim);
        sim.t = sim.inp.t_contact + 0.6;
        sim.assign();
        let_calls_be_heard(&mut sim);
        assert_eq!((sim.f[LF].role, sim.f[LF].call, sim.f[RF].role), (Role::Chase, 3, Role::Backup));
        // His fresh read, 45 m off the ball.
        sim.f[LF].pos = Vec3::new(-50.0, 92.0, 0.0);
        sim.f[LF].vel = Vec3::ZERO;
        sim.replan(LF);
        assert_eq!(sim.f[LF].call, 0, "his own read has no catch in the air for him");
        let withdrawn = sim.t;
        sim.publish();
        let mut took = None;
        while sim.t < withdrawn + 0.6 && took.is_none() {
            sim.move_fielders();
            sim.t += DT;
            sim.hear();
            if sim.f[RF].role == Role::Chase {
                took = Some(sim.t - withdrawn);
            } else {
                assert_eq!(sim.known(LF).1, 3, "the old call is still what is heard at {:.3} s", sim.t - withdrawn);
            }
            sim.publish();
        }
        let dt = took.expect("the right fielder takes the ball once the call is withdrawn");
        assert!(dt > CALL_HEARD_S - 1e-9 && dt < CALL_HEARD_S + DT + 1e-9, "took it {dt:.3} s after the withdrawal");
    }

    // Fourth review P2-2, the other way: a chaser going for a ball without calling it (tier
    // 1) whose fresh close read makes him sure calls it.
    #[test]
    fn a_chaser_who_becomes_sure_calls_it() {
        let fx = fixture(&[]);
        let mut inp = hit(&fx, Vec3::new(0.0, 33.0, 22.0));
        inp.fattrs[CF].read_sigma0 = 0.01;
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        for fd in sim.f.iter_mut() {
            fd.jump = Vec3::ZERO;
        }
        at_first_read(&mut sim);
        sim.t = sim.inp.t_contact + 0.6;
        sim.assign();
        assert_eq!((sim.f[CF].role, sim.f[CF].call), (Role::Chase, 3), "sure with a near-perfect read");
        sim.f[CF].call = 1;
        sim.replan(CF);
        assert_eq!(sim.f[CF].call, 3, "his fresh read makes him sure again");
    }

    // Codex fielder-communication review P2-1: the centre fielder reads the fly at its first
    // look; in the same tick the ball is deflected (off a glove), keeping 30% of its ground
    // speed and rising at 1 m/s. That is a new ball: his plan must be for it, which comes down
    // within about 45 m of the plate, not for the old one, which was going 99 m.
    #[test]
    fn a_ball_deflected_in_the_tick_of_a_read_is_read_again() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        at_first_read(&mut sim);
        sim.read(CF);
        assert!(sim.f[CF].int_pt.y > 90.0, "first read: {:?}", sim.f[CF].int_pt);
        let (b, v) = sim.ball_at(sim.t);
        sim.set_free(BallState { r: b, v: Vec3::new(0.3 * v.x, 0.3 * v.y, 1.0), w: Vec3::ZERO });
        let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
        let rest = path.at(path.t_end()).0;
        assert!(m::hypot(rest.x, rest.y) < 45.0, "the deflected ball stops at {rest:?}");
        let pt = sim.f[CF].int_pt;
        assert!(pt.y < 45.0, "his plan after the deflection is still for the old ball: {pt:?}");
    }

    // Agent-models §5.7: "A call reaches teammates after T_call = 0.3 s." The first and
    // second basemen are both sure of the pop-up of `a_called_off_infielder_covers_his_base_or_backs_up_the_catch`
    // and both call it; the second baseman has priority. The first baseman goes on for it
    // until the second baseman's call could be heard, and gives it up at the first tick after.
    #[test]
    fn a_call_is_not_heard_before_its_delay() {
        const B1: usize = 2;
        const B2: usize = 3;
        let fx = fixture(&[]);
        let inp = hit(&fx, Vec3::new(6.61, 11.81, 27.38));
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        at_first_read(&mut sim);
        assert_eq!((tier(&sim, B1), tier(&sim, B2)), (2, 2), "both sure");
        sim.assign();
        let t_call = sim.t;
        sim.publish();
        let mut gave_up = None;
        while sim.t < t_call + 0.6 && gave_up.is_none() {
            sim.move_fielders();
            sim.t += DT;
            sim.hear();
            if sim.f[B1].role != Role::Chase {
                gave_up = Some(sim.t - t_call);
            }
            assert_eq!(sim.f[B2].role, Role::Chase, "at {:.3} s", sim.t - t_call);
            sim.publish();
        }
        let dt = gave_up.expect("the first baseman gives it up");
        assert!(dt > 0.3 - 1e-9 && dt < 0.3 + DT + 1e-9, "gave it up {dt:.3} s after the call");
    }

    // Codex idle-fielder review #1: every way into `Role::Idle` brakes the fielder, not only
    // the calls. Runner on second, ball held by the right fielder at (25, 62) m: the first
    // baseman is the cutoff to home, his spot ~12.5 m in from where he stands. His job then
    // ends three ways: the right fielder throws straight to second past him (`holder_act`),
    // a throw to another man is in the air (`update_relay`), or an infielder has the ball
    // (`update_relay`). Nobody gives him another job (every base is covered and the throw's
    // backup is someone else), so a man standing still stays where he is, and a man already
    // running toward his old spot at 7 m/s pulls up within v²/(2·4.75 m/s²) ≈ 5.16 m along his
    // line instead of going on to it.
    #[test]
    fn a_retired_cutoff_man_with_no_job_stops() {
        const R: AgentId = 0x0007;
        const B1: usize = 2;
        let fx = fixture(&[(R, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let rf_at = Vec3::new(25.0, 62.0, 0.0);
        for (case, run) in [(0, 0.0), (1, 0.0), (2, 0.0), (0, 7.0), (2, 7.0)] {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = 5.0;
            held_at(&mut sim, 8, rf_at);
            assert_eq!(sim.f[B1].role, Role::Relay(4), "case {case}: the first baseman is the cutoff to home");
            let spot = sim.f[B1].target;
            let start = sim.f[B1].pos;
            assert!((spot - start).norm() > 10.0, "case {case}: cutoff spot {spot:?} from {start:?}");
            let dir = Vec3::new(spot.x - start.x, spot.y - start.y, 0.0).normalized();
            sim.f[B1].vel = dir * run;
            // The second-base cover is on his bag; the runner on second is standing (no play).
            let c2 = (0..9).find(|&i| sim.f[i].role == Role::Cover(2)).expect("second is covered");
            sim.f[c2].pos = base_pos(2);
            sim.f[c2].target = base_pos(2);
            match case {
                0 => {
                    sim.holder_act();
                    assert!(matches!(sim.ball, Ball::Thrown { receiver, relay: false, .. } if receiver == c2), "case 0: a straight throw to second");
                }
                1 => {
                    sim.throw_to(8, base_pos(2), 2, c2, false, 1.0);
                    sim.update_relay();
                }
                _ => {
                    sim.f[c2].target = sim.f[c2].pos;
                    sim.ball = Ball::Held { by: c2 };
                    sim.update_relay();
                }
            }
            assert_eq!(sim.f[B1].role, Role::Idle, "case {case}: his job is over and he has no other");
            for _ in 0..200 {
                sim.move_fielders();
                sim.t += DT;
            }
            let fd = &sim.f[B1];
            let moved = fd.pos - start;
            assert!(fd.vel.norm() < 1e-9, "case {case} (v0 {run}): still moving {:?} at {:?}", fd.vel, fd.pos);
            let want = run * run / (2.0 * 4.75);
            assert!((moved.norm() - want).abs() < 0.15, "case {case} (v0 {run}): moved {:.3} m, expected {want:.3} m; at {:?}", moved.norm(), fd.pos);
            if run > 0.0 {
                assert!(moved.normalized().dot(dir) > 0.99, "case {case}: stops on his line of motion");
            }
        }
    }

    // Watch-fixes review #1–#3: the catcher goes back to cover home once a runner rounds
    // first; a backup catches an overthrow that reaches him in the air; a runner holds his
    // lead (he does not drift back to the base) until he has read the ball.
    #[test]
    fn catcher_returns_backup_catches_and_runners_hold_their_leads() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
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
        // From deep center toward home, a soft throw over the receiver's head at the base,
        // falling to chest height at the backup.
        let backup = sim.f[0].pos;
        aimed_throw(&mut sim, Vec3::new(0.0, 80.0, 0.0), backup, 29.0);
        let Ball::Thrown { path, .. } = &sim.ball else { unreachable!() };
        let over = path.find(bb_physics::flight::Crossing::YBelow(second.y)).map(|t| path.at(t).0.z).unwrap();
        assert!(over > CATCH_MAX_Z + 0.3, "the throw must sail over the receiver: {over}");
        for _ in 0..300 {
            sim.update_thrown();
            sim.try_captures();
            sim.t += DT;
        }
        assert!(matches!(sim.ball, Ball::Held { by: 0 }), "ball: {:?}", sim.ball);

        // A runner on first stays at his lead until the runners read the ball at +0.5 s.
        let fx = fixture(&[(R, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        for _ in 0..25 {
            sim.move_runners();
            sim.t += DT;
        }
        assert!((sim.r[1].s - lead_s(1)).abs() < 1e-9, "runner drifted to {:.2} m", sim.r[1].s - BASE_L);
    }

    #[test]
    fn an_early_bounce_does_not_skip_the_contact_read() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 2)]);
        let mut inp = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: R, base: 2, speed: 8.2, tau_s: RUNNER_TAU },
        ]);
        inp.ball.v = Vec3::new(10.0, 30.0, -25.0);
        let mut st = fx.st.clone();
        let (out, ticks) = runner_trace::capture(|| simulate_live(&inp, &mut st));
        assert!(out.facts.iter().any(|f| matches!(f,
            Input::BallTouched { t, what: Touch::Ground { .. } } if *t < inp.t_contact + 0.10
        )));
        let reading: Vec<_> = ticks.iter().filter(|t| t.runner == R && t.t < inp.t_contact + 0.49).collect();
        assert!(reading.len() >= 25 && reading.iter().any(|t| t.ball_down));
        assert!(reading.iter().all(|t| t.cause == runner_trace::Cause::Read));
    }

    #[test]
    fn a_runner_brakes_into_a_partway_hold() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 2)]);
        let inp = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: R, base: 2, speed: 8.2, tau_s: RUNNER_TAU },
        ]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let stop_s = sim.r[1].s + 5.0;
        sim.r[1].hold_s = Some(stop_s);
        let mut prior = 0.0;
        for _ in 0..180 {
            sim.move_runners();
            let runner = &sim.r[1];
            assert!(runner.s <= stop_s);
            if runner.s == stop_s {
                // The established slow final step is 1 m/s, so the approach should
                // be down to walking speed when he plants.
                assert!(prior < 1.2, "planted from {prior:.2} m/s");
                assert_eq!(runner.v, 0.0);
                return;
            }
            assert!(prior - runner.v <= 4.75 / 60.0 + 1e-9);
            prior = runner.v;
            sim.t += DT;
        }
        panic!("runner never reached the hold point");
    }

    #[test]
    fn the_runner_credits_motion_toward_a_nearby_catch() {
        use bb_physics::ground::Segment;

        let fx = fixture(&[]);
        let inp = input(&fx, Vec::new());
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let now = 1.0;
        let end = now + 0.15;
        sim.t = now;
        for fd in &mut sim.f {
            fd.pos = Vec3::new(-20.0, 30.0, 0.0);
        }
        sim.f[2].pos = Vec3::new(0.0, 30.0, 0.0);
        let spot = BallState { r: Vec3::new(1.2, 30.0, 1.0), v: Vec3::ZERO, w: Vec3::ZERO };
        let prediction = BallPath {
            segments: vec![Segment::Flight { tr: Trajectory::single(end, spot), end: SegmentEnd::Timeout }],
            bounces: Vec::new(),
            walls: Vec::new(),
        };
        // The fielder needs 0.2 m beyond glove reach. From rest he covers less
        // than 0.1 m in 0.15 s at his initial acceleration of about 9 m/s².
        // Heading toward the spot at 3 m/s, he covers 0.2 m in under 0.07 s.
        for (vx, expected) in [(-3.0, false), (0.0, false), (3.0, true)] {
            sim.f[2].vel = Vec3::new(vx, 0.0, 0.0);
            assert_eq!(sim.runner_catch_probability(&prediction) > 0.5, expected, "visible velocity {vx}");
        }
    }

    #[test]
    fn catch_height_is_continuous_and_a_fence_leap_is_in_reach() {
        use bb_physics::ground::Segment;

        let fx = fixture(&[]);
        let inp = input(&fx, Vec::new());
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        let score = |sim: &mut Sim, y, z| {
            sim.f[CF].pos = Vec3::new(0.0, y, 0.0);
            let ball = BallState { r: Vec3::new(0.0, y, z), v: Vec3::ZERO, w: Vec3::ZERO };
            let path = BallPath {
                segments: vec![Segment::Flight { tr: Trajectory::single(sim.t, ball), end: SegmentEnd::Timeout }],
                bounces: Vec::new(),
                walls: Vec::new(),
            };
            sim.runner_catch_probability(&path)
        };
        let below = score(&mut sim, 60.0, 2.399);
        let above = score(&mut sim, 60.0, 2.401);
        // The physical ceiling lies between these samples. A millimetre each
        // side should not halve the chance over a glove's width of uncertainty.
        assert!(below > above && below - above < 0.01, "{below:.5} vs {above:.5}");
        let fence_y = (80..1400).map(|i| f64::from(i) * 0.1).find(|&y| {
            (0.4..1.0).contains(&inp.field.body_clearance(0.0, y, 0.0).dist)
        }).expect("a point just in front of the center-field fence");
        assert!(score(&mut sim, 60.0, 2.85) < 0.1);
        assert!(score(&mut sim, fence_y, 2.85) > 0.65);
    }

    #[test]
    fn a_fixed_contact_look_hides_a_change_of_flight() {
        const R: AgentId = 7;
        let fx = fixture(&[(R, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: R, base: 2, speed: 8.2, tau_s: RUNNER_TAU }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.start_runners();
        sim.r[0].reading_contact = false;
        sim.t = inp.t_contact + RUNNER_READ_DELAY_S;
        let look = ContactLook {
            motion: (Vec3::new(0.0, 17.0, 5.0), Vec3::new(0.0, 30.0, 8.0)),
            from: sim.runner_pos(&sim.r[0]),
            t: sim.t,
            view: RUNNER_READ_DELAY_S,
            contact_since: Some(RUNNER_READ_DELAY_S),
        };
        for fd in &mut sim.f {
            fd.pos = Vec3::new(-70.0, 30.0, 0.0);
        }
        // Sweep one observer-visible catch opportunity from under the fly to out of reach.
        // Then pin a close read while the world privately replaces the trajectory.
        let flight = look.project(inp.field, [0.0; 4]);
        let (_, x, y) = flight.first_bounce().unwrap();
        let tag = sim.tag_confidence(0);
        let belief = (0..300).find_map(|offset| {
            sim.f[6].pos = Vec3::new(x + f64::from(offset) * 0.1, y, 0.0);
            let p = Sim::catch_score(&sim.runner_catch_paths(look));
            (0.25..tag).contains(&p).then_some(p)
        }).expect("a close catch between the fielder and the gap");

        let initial = sim.r[0].clone();
        sim.read_contact(0, look);
        let action = (sim.r[0].goal, sim.r[0].tag_up, sim.r[0].hold_s);
        assert!(action.2.is_some(), "the matched look must set a recoverable hold");
        let old_motion = sim.ball_at(sim.t);
        sim.set_free(BallState {
            r: Vec3::new(3.0, 12.0, 1.0),
            v: Vec3::new(13.0, 8.0, -3.0),
            w: Vec3::ZERO,
        });
        assert_ne!(sim.ball_at(sim.t), old_motion);
        assert!(sim.no_catch_at.is_none());
        sim.r[0] = initial;
        assert_eq!(Sim::catch_score(&sim.runner_catch_paths(look)), belief);
        sim.read_contact(0, look);
        assert_eq!((sim.r[0].goal, sim.r[0].tag_up, sim.r[0].hold_s), action);
    }

    // Agent-models §6.3, the in-flight relook (fly-read job, 2026-11-19): a runner on first
    // with one out reads a deep fly to center as a home run at his first look (his read of
    // its speed 12% high) and goes. Two worlds differ only in what he can see: the center
    // fielder standing under where it comes down, or about 100 m from it with the rest of the
    // fielders (none can get there). Watching
    // the ball while it is up, he pulls up (back to tag, or into a hold he can get back from)
    // when the fielder is under it, and keeps going when nobody can get there.
    #[test]
    fn a_runner_who_reads_a_home_run_pulls_up_when_the_fielder_settles_under_it() {
        const R: AgentId = 7;
        let mut fx = fixture(&[(R, 1)]);
        fx.st.outs = 1;
        let la = 33f64.to_radians();
        let ball = typical_ball(&fx.env, Vec3::new(0.0, 0.5, 1.0), Vec3::new(0.0, 44.0 * m::cos(la), 44.0 * m::sin(la)));
        let (t_land, lx, ly) = batted_path(&fx.env, &fx.field, ball, 0.0, |_, _| (0.0, 0.0)).first_bounce().expect("in the park");
        let land = Vec3::new(lx, ly, 0.0);
        assert!((95.0..115.0).contains(&ly), "lands at {land:?}");
        let mut seen = Vec::new();
        for cf in [land, Vec3::new(-70.0, 30.0, 0.0)] {
            let mut inp = input(&fx, vec![RunnerIn { id: R, base: 1, speed: 8.2, tau_s: RUNNER_TAU }]);
            inp.ball = ball;
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            for fd in &mut sim.f {
                fd.pos = Vec3::new(-70.0, 30.0, 0.0);
                fd.vel = Vec3::ZERO;
            }
            sim.f[CF].pos = cf;
            sim.start_runners();
            sim.r[0].reading_contact = false;
            sim.t = inp.t_contact + RUNNER_READ_DELAY_S;
            let (p, v) = sim.ball_at(sim.t);
            let look = ContactLook { motion: (p, v * 1.12), from: sim.runner_pos(&sim.r[0]), t: sim.t, view: RUNNER_READ_DELAY_S, contact_since: Some(RUNNER_READ_DELAY_S) };
            assert!(look.project(inp.field, [0.0; 4]).first_bounce().is_none(), "he reads it over the fence");
            sim.read_contact(0, look);
            assert!(sim.r[0].goal == 2 && sim.r[0].fly_look.is_some(), "he goes at his first look");
            while sim.t < inp.t_contact + t_land - 0.5 {
                sim.move_runners();
                sim.t += DT;
            }
            let r = &sim.r[0];
            seen.push((r.tag_up || r.hold_s.is_some(), r.s, r.direction * r.v));
        }
        let ((under_pulled, under_s, under_v), (away_pulled, away_s, _)) = (seen[0], seen[1]);
        assert!(under_pulled && under_v <= 0.0, "fielder under it: still going at {under_s:.1} m, {under_v:.1} m/s");
        assert!(!away_pulled, "nobody near it: he pulled up at {away_s:.1} m");
        assert!(under_s < away_s - 5.0, "{under_s:.1} m against {away_s:.1} m");
    }

    // Agent-models §6.3, the in-flight relook while holding: a runner on first with one out
    // holds partway on a deep fly to center. Two worlds differ only in what he can see: the
    // center fielder standing under where it comes down, or about 100 m from it. Watching it,
    // he goes back to tag up when the fielder is under it and is on the bag before it comes
    // down; with nobody near it he keeps his hold.
    #[test]
    fn a_runner_holding_partway_goes_back_to_tag_when_the_catch_is_sure() {
        const R: AgentId = 7;
        let mut fx = fixture(&[(R, 1)]);
        fx.st.outs = 1;
        let la = 33f64.to_radians();
        let ball = typical_ball(&fx.env, Vec3::new(0.0, 0.5, 1.0), Vec3::new(0.0, 42.0 * m::cos(la), 42.0 * m::sin(la)));
        let (t_land, lx, ly) = batted_path(&fx.env, &fx.field, ball, 0.0, |_, _| (0.0, 0.0)).first_bounce().expect("in the park");
        let land = Vec3::new(lx, ly, 0.0);
        let mut seen = Vec::new();
        for cf in [land, Vec3::new(-70.0, 30.0, 0.0)] {
            let mut inp = input(&fx, vec![RunnerIn { id: R, base: 1, speed: 8.2, tau_s: RUNNER_TAU }]);
            inp.ball = ball;
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            for fd in &mut sim.f {
                fd.pos = Vec3::new(-70.0, 30.0, 0.0);
                fd.vel = Vec3::ZERO;
            }
            sim.f[CF].pos = cf;
            sim.start_runners();
            sim.r[0].reading_contact = false;
            sim.t = inp.t_contact + RUNNER_READ_DELAY_S;
            // Holding partway, as a close read at his first look leaves him.
            let hold = lead_s(1) + 4.0;
            sim.r[0].hold_s = Some(hold);
            sim.r[0].goal = 2;
            sim.r[0].fly_look = Some((sim.t, 0));
            let mut on_bag_at = None;
            while sim.t < inp.t_contact + t_land {
                sim.move_runners();
                sim.t += DT;
                if on_bag_at.is_none() && (sim.r[0].s - BASE_L).abs() < 1e-6 {
                    on_bag_at = Some(sim.t);
                }
            }
            seen.push((sim.r[0].tag_up, sim.r[0].hold_s, on_bag_at));
        }
        let ((under_tag, _, under_bag), (away_tag, away_hold, _)) = (seen[0], seen[1]);
        assert!(under_tag && under_bag.is_some(), "fielder under it: tag-up {under_tag}, on the bag at {under_bag:?}");
        assert!(!away_tag && away_hold.is_some(), "nobody near it: tag-up {away_tag}, hold {away_hold:?}");
    }

    /// Catch-read fixes review P1-1: a runner's relook at a loose ball sees its motion now,
    /// not where it was actually hit or whether the world's path began at contact. Three
    /// worlds with identical ball position and velocity at the look: the batted flight, the
    /// same flight with the contact point moved 15 cm across and up, and a replacement path
    /// starting now with the same motion (as an unseen deflection would leave). No ground,
    /// wall or glove touch is public in any of them, so the runner's prediction and his
    /// pickup estimate at third must agree.
    #[test]
    fn a_loose_ball_relook_ignores_the_private_contact_point_and_path_origin() {
        const R: AgentId = 7;
        let fx = fixture(&[(R, 2)]);
        let runners = vec![RunnerIn { id: R, base: 2, speed: 8.2, tau_s: RUNNER_TAU }];
        let inp = input(&fx, runners.clone());
        let mut moved = input(&fx, runners);
        moved.ball.r += Vec3::new(0.15, 0.0, 0.15);
        let now = inp.t_contact + 0.5;
        let look = |sim: &Sim| {
            let pred = sim.runner_predict(0, 3);
            let land = pred.first_bounce().map(|(t, x, y)| (t, x, y));
            (land, pred.at(pred.t_end()).0, sim.ball_time_to(0, 3, 3, now + 2.0))
        };
        let prepare = |sim: &mut Sim| {
            sim.start_runners();
            sim.r[0].reading_contact = false;
            sim.t = now;
        };

        let mut st = fx.st.clone();
        let mut batted = new_sim(&inp, &mut st);
        prepare(&mut batted);
        let motion = batted.ball_at(now);
        let base = look(&batted);

        let mut st = fx.st.clone();
        let mut shifted = new_sim(&moved, &mut st);
        prepare(&mut shifted);
        shifted.ball = batted.ball.clone();
        assert_eq!(shifted.ball_at(now), motion);
        assert_eq!(look(&shifted), base, "the true contact point moved the relook");

        let mut st = fx.st.clone();
        let mut replaced = new_sim(&inp, &mut st);
        prepare(&mut replaced);
        let w = match &replaced.ball {
            Ball::Free { path, .. } => match path.segments.first() {
                Some(bb_physics::ground::Segment::Flight { tr, .. }) => tr.first().s.w,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
        replaced.set_free(BallState { r: motion.0, v: motion.1, w });
        assert_eq!(replaced.ball_at(now), motion);
        assert!(replaced.no_catch_at.is_none() && replaced.out.first_handling.is_none());
        assert_eq!(look(&replaced), base, "a path replaced without a public touch moved the relook");
    }

    #[test]
    fn a_close_trap_holds_the_runner_partway_until_he_can_react_to_the_call() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 1)]);
        let inp = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: R, base: 1, speed: 8.2, tau_s: RUNNER_TAU },
        ]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.start_runners();
        let look = inp.t_contact + 0.5;
        sim.t = look;
        sim.r[1].reading_contact = false;
        for fd in &mut sim.f {
            fd.pos = Vec3::new(-20.0, 30.0, 0.0);
        }
        let ball = BallState {
            r: Vec3::new(0.0, 30.0, 0.8),
            v: Vec3::new(0.0, 24.0, -4.0),
            w: Vec3::ZERO,
        };
        let seen = batted_path(&fx.env, &fx.field, ball, look, |_, _| (0.0, 0.0));
        let (bounce, x, y) = seen.first_bounce().unwrap();
        assert!(bounce < look + 0.2);

        // Two metres past the landing is outside a standing fielder's reach
        // throughout this short flight. A fielder under the ball has it now.
        sim.f[2].pos = Vec3::new(x, y + 2.0, 0.0);
        assert!(sim.runner_catch_probability(&seen) < 0.05);
        sim.f[2].pos = Vec3::new(ball.r.x, ball.r.y, 0.0);
        assert!(sim.runner_catch_probability(&seen) > 0.95);

        // One metre beyond the landing leaves about one glove length at the
        // end of the flight. The catch is close and the runner remains off first.
        sim.f[2].pos = Vec3::new(x, y + 1.0, 0.0);
        let p = sim.runner_catch_probability(&seen);
        assert!((0.20..0.80).contains(&p), "close catch belief {p:.2}");
        sim.r[1].s = BASE_L + 0.8;
        sim.r[1].v = 0.0;
        sim.set_catch_hold(1, &[sim.runner_catch_read(&seen)]);
        let hold = sim.r[1].hold_s.expect("the close read keeps him partway");
        assert!(hold < 1.5 * BASE_L && !sim.r[1].tag_up);
        let start = sim.r[1].s;
        for _ in 0..12 {
            sim.move_runners();
            sim.t += DT;
        }
        assert!(sim.r[1].s >= start && sim.r[1].hold_s.is_some());

        let call = sim.t;
        sim.emit(Input::BallTouched { t: call, what: Touch::Ground { x, y } });
        for tick in 0..15 {
            sim.t = call + f64::from(tick) / 60.0;
            sim.move_runners();
            assert!(sim.r[1].hold_s.is_some(), "reacted before 250 ms");
        }
        sim.t = call + 0.25;
        sim.move_runners();
        assert!(sim.r[1].hold_s.is_none());
        assert_eq!(sim.r[1].goal, 2, "the forced runner goes on the no-catch call");
    }

    #[test]
    fn a_forced_runner_checks_a_low_liner_before_the_first_baseman_catches_it() {
        const R: AgentId = 0x0007;
        let mut fx = fixture(&[(R, 1)]);
        fx.st.outs = 1;
        let mut inp = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: R, base: 1, speed: 8.2, tau_s: RUNNER_TAU },
        ]);
        // About 44 m/s toward the first baseman. Over the first half second, the
        // 2.7 m/s upward launch gains about 1.35 m and gravity takes about 1.2 m.
        // The descending ball is still catchable at a little over one metre high.
        inp.ball = BallState {
            r: Vec3::new(0.0, 0.5, 1.05),
            v: Vec3::new(29.0, 33.5, 2.7),
            w: Vec3::ZERO,
        };
        inp.runner_start.fewer_than_two = SecondaryLead { extra_m: 1.0, speed_mps: 3.0 };
        let mut st = fx.st.clone();
        let (out, ticks) = runner_trace::capture(|| simulate_live(&inp, &mut st));
        let catch = out.facts.iter().find_map(|fact| match *fact {
            Input::GloveSecured { t, fielder, .. } if fielder == inp.fielders[2] => Some(t),
            _ => None,
        }).expect("the first baseman catches the liner");
        assert!(out.events.contains(&Event::Catch { fielder: inp.fielders[2] }));
        assert!((0.55..0.75).contains(&(catch - inp.t_contact)));
        let after_read: Vec<_> = ticks.iter().filter(|tick| {
            tick.runner == R && tick.t >= inp.t_contact + 0.5 - 1e-9 && tick.t < catch
        }).collect();
        assert!(after_read.len() >= 3 && after_read.iter().all(|tick| !tick.ball_down));
        // The runner starts the read shuffling at 3 m/s. The descending liner
        // should make him check that motion while it is still in the air.
        assert!(
            after_read.iter().all(|tick| tick.cause != runner_trace::Cause::Read),
            "the runner was still in the contact read: {after_read:?}",
        );
        assert!(
            after_read.iter().any(|tick| tick.signed_mps < 2.99),
            "the runner never started braking before the catch: {after_read:?}",
        );
        assert!(after_read.iter().all(|tick| tick.signed_mps <= 3.0 + 1e-9), "{after_read:?}");
    }

    // Catch-read follow-up (2026-11-18): runners from first were doubled off on 10–25°
    // liners that outfielders caught, because the catch belief took the runner's look at the
    // ball as exact. Two liners straight at the left fielder with the contact model's
    // typical spin (`typical_ball`; speeds raised from 38 and 44 m/s when the belief gained the
    // side and gyro spin's drag, 2026-11-19, to keep the landings), one sinking about 11 m in
    // front of where he stands and one carrying about 17 m past him: he catches most of them
    // over 24 perception keys (23 and 17 on 2026-11-19; the deep one 18 or more before). The
    // runner may not be doubled off on any.
    #[test]
    fn a_runner_is_not_doubled_off_on_liners_the_left_fielder_reaches() {
        const R: AgentId = 0x0007;
        let mut fx = fixture(&[(R, 1)]);
        fx.st.outs = 1;
        let lf = home_positions_for(bb_observe::Hand::Right, true, false, 1)[6];
        let h = Vec3::new(lf.x, lf.y, 0.0).normalized();
        for (speed, la_deg, beyond) in [(39.8, 16.0, -11.5), (46.4, 18.0, 16.9)] {
            let la = f64::to_radians(la_deg);
            let ball = typical_ball(&fx.env, Vec3::new(0.0, 0.5, 1.0), h * (speed * m::cos(la)) + Vec3::new(0.0, 0.0, speed * m::sin(la)));
            let (_, x, y) = batted_path(&fx.env, &fx.field, ball, 0.4, |_, _| (0.0, 0.0)).first_bounce().unwrap();
            assert!(((Vec3::new(x, y, 0.0) - lf).dot(h) - beyond).abs() < 1.0, "lands {:.1} m beyond him", (Vec3::new(x, y, 0.0) - lf).dot(h));
            let (mut caught, mut doubled) = (0, 0);
            for key in 0..24u64 {
                let mut inp = input(&fx, vec![
                    RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU },
                    RunnerIn { id: R, base: 1, speed: 8.2, tau_s: RUNNER_TAU },
                ]);
                inp.ball = ball;
                inp.seed = key;
                let mut st = fx.st.clone();
                let out = simulate_live(&inp, &mut st);
                caught += u32::from(out.events.contains(&Event::Catch { fielder: inp.fielders[6] }));
                doubled += out.events.iter().filter(|e| matches!(e, Event::RunnerOut { runner: R, kind: OutKind::Appeal, .. })).count();
            }
            assert!(caught >= 16, "{speed} m/s at {la_deg}°: the left fielder caught {caught} of 24");
            assert_eq!(doubled, 0, "{speed} m/s at {la_deg}°: doubled off {doubled} times in {caught} catches");
        }
    }

    // The owner's close-catch rule (agent-models §6.3): a runner holding on a close catch
    // stands only as far off as he can get back from. On second, a catch at the wall leaves
    // a long throw and the full partway hold; a catch in shallow left leaves a short one and
    // keeps him inside his lead. From that hold, after a catch, the movement step brings
    // him back to the bag (reaction to the call, then braking in) with his caution to spare
    // before a typical transfer and throw from the catch.
    #[test]
    fn a_close_catch_hold_leaves_room_to_get_back() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 2)]);
        let inp = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: R, base: 2, speed: 8.2, tau_s: RUNNER_TAU },
        ]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.start_runners();
        let bag = 2.0 * BASE_L;
        let read_at = |at| [Some(CatchRead { score: 0.5, t: 5.0, at })];
        let deep = sim.catch_hold_s(1, &read_at(Vec3::new(-50.0, 110.0, 0.0))).unwrap();
        assert!((deep - (lead_s(2) + HALFWAY_M)).abs() < 1e-9, "deep hold {:.2} m off", deep - bag);
        let at = Vec3::new(-15.0, 45.0, 0.0);
        let shallow = sim.catch_hold_s(1, &read_at(at)).unwrap();
        assert!(shallow > bag + 1.0 && shallow < lead_s(2), "shallow hold {:.2} m off", shallow - bag);

        let call = 5.0;
        sim.t = call;
        sim.r[0].out = true;
        sim.caught_at = Some(call);
        sim.r[1].off_base_at_catch = true;
        sim.r[1].reading_contact = false;
        (sim.r[1].s, sim.r[1].v, sim.r[1].direction, sim.r[1].goal, sim.r[1].hold_s) = (shallow, 0.0, 1.0, 3, Some(shallow));
        sim.ball = Ball::Held { by: 6 };
        sim.f[6].pos = at;
        while sim.r[1].s > bag + 1e-9 && sim.t < call + 5.0 {
            sim.move_runners();
            sim.t += DT;
        }
        let back = sim.t - call;
        let ball = TYPICAL_TRANSFER + typical_throw_time((at - base_pos(2)).norm(), Cut::No);
        let spare = ball - back;
        let caution = inp.advance.caution_z * inp.advance.sigma_race_s;
        assert!(spare >= caution - DT && spare <= caution + 0.1, "back in {back:.3} s, ball in {ball:.3} s");
    }

    #[test]
    fn a_close_early_catch_sends_the_standard_moving_lead_back_immediately() {
        const R: AgentId = 7;
        let fx = fixture(&[(R, 1)]);
        let inp = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: R, base: 1, speed: 8.2, tau_s: RUNNER_TAU },
        ]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.start_runners();
        sim.r[1].s += 0.5 * sim.r[1].v;
        sim.r[1].reading_contact = false;
        sim.t = 0.9;
        let call = 1.05;
        let at = base_pos(1) - Vec3::X * 30.0;
        let paths = [Some(CatchRead { score: 0.5, t: call, at })];
        let lead = sim.r[1].s;
        assert!((lead - BASE_L - 8.0).abs() < 1e-9);
        sim.set_catch_hold(1, &paths);
        assert!(sim.r[1].tag_up && sim.r[1].hold_s.is_none());
        sim.emit(Input::BaseLeft { t: sim.t, agent: R, base: 1 });
        sim.move_runners();
        assert!((sim.r[1].v - (3.0 - 4.75 / 60.0)).abs() < 1e-9);
        let mut peak = sim.r[1].s;
        for tick in 1..210 {
            sim.t = 0.9 + f64::from(tick) / 60.0;
            if tick == 9 {
                sim.emit(Input::GloveSecured { t: call, fielder: sim.f[CF].id, x: at.x, y: at.y });
                assert!(sim.caught());
                sim.ball = Ball::Held { by: CF };
                sim.f[CF].pos = at;
            }
            sim.move_runners();
            peak = peak.max(sim.r[1].s);
        }
        let retouch = sim.out.facts.iter().find_map(|fact| match *fact {
            Input::BaseTouched { agent: R, base: 1, t, holding_ball: false } if t > call => Some(t),
            _ => None,
        }).expect("legally retouched");
        // The stopping distance from 3 m/s is about 0.95 m. Even an immediate
        // turn from an 8 m lead cannot beat the roughly 1.64 s appeal.
        assert!((0.9..0.95).contains(&(peak - lead)), "braking drift {}", peak - lead);
        // Acceleration to about 6.4 m/s, braking, and the final slow step
        // put the earliest return near 2.9 s after the catch.
        assert!((2.7..3.1).contains(&(retouch - call)), "retouch after {}", retouch - call);
    }

    #[test]
    fn a_runner_does_not_plant_at_a_hold_inside_his_braking_distance() {
        const R: AgentId = 7;
        let fx = fixture(&[(R, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: R, base: 1, speed: 8.2, tau_s: RUNNER_TAU }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 0.9;
        let r = &mut sim.r[0];
        let start = BASE_L + 8.0;
        let target = start + 0.13;
        (r.s, r.v, r.hold_s) = (start, 2.84, Some(target));
        let mut peak = start;
        for tick in 0..120 {
            sim.t = 0.9 + f64::from(tick) / 60.0;
            sim.move_runners();
            peak = peak.max(sim.r[0].s);
        }
        // From 2.84 m/s the ideal braking distance is 0.85 m. At 60 Hz it
        // is about 0.82 m, so planting 0.13 m ahead is physically impossible.
        assert!((0.80..0.85).contains(&(peak - start)), "braking drift {}", peak - start);
        assert_eq!((sim.r[0].s, sim.r[0].v), (target, 0.0));
    }

    // Runners take leads, and on a fly that might drop with fewer than two out they go
    // partway rather than all the way: a runner from first is never past halfway to second
    // while a deep fly is still in the air.
    #[test]
    fn runner_leads_off_and_goes_halfway_on_a_fly() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
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
            let goal = sim.f[SS].target;
            // Arrived: on the spot (stopped there) or past it along the run (running through).
            for _ in 0..600 {
                sim.move_fielders();
                sim.t += DT;
                if (goal - sim.f[SS].pos).x <= 1e-6 {
                    break;
                }
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: R, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
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

    /// A throw from `from` aimed at base `b` (chest height there), at `speed`, released at 5.0 s
    /// by the fielder at `thrower`.
    fn throw_to_base(sim: &mut Sim, from: Vec3, b: u8, speed: f64) {
        let to = base_pos(b);
        let d = m::hypot(to.x - from.x, to.y - from.y);
        let s = solve_throw(sim.inp.env, THROW_RELEASE_Z, THROW_CATCH_Z, d, speed).unwrap_or_else(|| panic!("a {d:.0} m throw is in reach at {speed} m/s"));
        let s0 = throw_state(from + Vec3::new(0.0, 0.0, THROW_RELEASE_Z), to - from, speed, s.elev);
        sim.t = 5.0;
        sim.launch_throw(s0, to, b, 3, false);
    }

    fn project_fixture_throw(sim: &Sim) -> ThrownRead {
        let (p, v) = sim.ball_at(sim.t);
        sim.project_runner_throw(p, v)
    }

    #[test]
    fn a_runner_observes_the_throw_with_one_keyed_noisy_look() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        throw_to_base(&mut sim, Vec3::new(0.0, 80.0, 0.0), 2, 40.0);

        let exact = sim.ball_at(sim.t);
        let look = sim.runner_throw_observation(0, 3);
        let other = sim.runner_throw_observation(0, 5);
        assert_eq!(look, sim.runner_throw_observation(0, 3));
        assert!((look.0 - exact.0).norm() > 0.01, "the look perturbs position");
        assert!((look.1 - exact.1).norm() > 0.01, "the look perturbs velocity");
        assert_ne!(look, other, "a different decision gets its own look");
        let read = sim.thrown_read(0, 3);
        let start = read.track[0].1;
        assert_eq!((start.x, start.y), (look.0.x, look.0.y));
        assert!((start.z - look.0.z.max(reference_env().ball.radius)).abs() < 1e-9);
        assert_ne!(sim.ball_time_to(0, 2, 3, f64::INFINITY), sim.ball_time_to(0, 2, 5, f64::INFINITY));
        // After watching the throw travel toward second, a majority of the
        // snapshots should still read that bag rather than a relay through home.
        sim.t += 0.8;
        let to_second = (0..128).filter(|&n| sim.thrown_read(0, 2 * n + 1).dest == 2).count();
        assert!(to_second > 64, "{to_second} of 128 looks read second");
    }

    #[test]
    fn a_runner_times_the_holder_from_visible_posture() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 5.0;
        sim.secured_at = 5.0;
        sim.ball = Ball::Held { by: CF };
        sim.f[CF].pos = Vec3::new(0.0, 70.0, 0.0);
        let race = |s: &Sim| s.ball_time_to(0, 2, 3, f64::INFINITY);
        let upright = race(&sim);

        sim.f[CF].up_at = 10.0;
        sim.f[CF].ready_at = 11.0;
        assert_eq!(race(&sim), upright, "a private scheduling change is invisible");
        sim.f[CF].catch_stretched = true;
        assert!((race(&sim) - upright - 0.30).abs() < 1e-9);

        // A dive seen 0.2 s before the catch leaves 1.3 s of a typical
        // 1.5 s get-up, plus the fielder's footwork once upright.
        sim.f[CF].catch_stretched = false;
        let pos = sim.f[CF].pos;
        sim.f[CF].lunge = Some((4.8, pos, pos));
        assert!((1.3..1.7).contains(&(race(&sim) - upright)));
    }

    #[test]
    fn the_holders_visible_reception_hides_the_incoming_throws_private_route() {
        const R: AgentId = 7;
        let fx = fixture(&[(R, 1)]);
        let inp = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: R, base: 1, speed: 8.2, tau_s: RUNNER_TAU },
        ]);
        // `off`: how far from second he takes it. Off the bag, a feed waits for the bag;
        // then he reaches it moving across, the same visible arrival under every route.
        let receive = |base, relay, off: f64| {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            if off != 0.0 {
                // A ground ball, so the batter is not out on the reception and the force
                // at second stands.
                sim.emit(Input::BallTouched { t: 1.0, what: Touch::Ground { x: 0.0, y: 30.0 } });
            }
            sim.t = 5.0;
            let by = 3;
            let at = base_pos(2) + Vec3::new(off, 0.0, 0.0);
            sim.f[by].pos = at;
            sim.f[by].vel = Vec3::ZERO;
            sim.launch_throw(BallState {
                r: at + Vec3::Z * THROW_CATCH_Z,
                v: Vec3::new(0.0, 20.0, 0.0),
                w: Vec3::ZERO,
            }, base_pos(base), base, by, relay);
            sim.try_captures();
            assert!(matches!(sim.ball, Ball::Held { by: 3 }));
            let feed = base == 2 && !relay;
            assert_eq!(sim.f[by].pivot || sim.f[by].pivot_wait_bag, feed, "the fixtures change the private setup");
            let received = (sim.observe_holder(by), sim.ball_time_to(0, 1, 3, 5.5));
            if off == 0.0 {
                return (received, None);
            }
            assert_eq!(sim.f[by].pivot_wait_bag, feed);
            sim.t += DT;
            sim.f[by].pos = base_pos(2) + Vec3::new(0.35, 0.0, 0.0);
            sim.f[by].vel = Vec3::new(-4.0, 0.0, 0.0);
            sim.holder_act();
            assert!(matches!(sim.ball, Ball::Held { by: 3 }), "still holding it on the bag");
            assert_eq!(sim.f[by].pivot, feed, "only the feed pivots on the bag");
            (received, Some((sim.observe_holder(by), sim.ball_time_to(0, 1, 3, 5.5))))
        };
        let feed = receive(2, false, 0.0);
        assert_eq!(feed, receive(3, false, 0.0));
        assert_eq!(feed, receive(2, true, 0.0));
        assert!(feed.0.0.received_throw);
        assert!(feed.0.1 > 6.0 && feed.0.1 < 7.5);
        // Catch-read fixes review P1-2: the private pivot on reaching the bag must not
        // rewrite the reception the runner saw.
        let feed = receive(2, false, 0.61);
        assert_eq!(feed, receive(3, false, 0.61));
        assert_eq!(feed, receive(2, true, 0.61));
        assert_eq!(feed.0.0.motion, feed.1.unwrap().0.motion);
    }

    #[test]
    fn a_throw_with_a_recorded_release_times_both_base_races() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let from = Vec3::new(12.0, 85.0, 0.0);
        throw_to_base(&mut sim, from, 2, 40.0);
        let fielder = sim.f[CF].id;
        sim.emit(Input::GloveSecured { t: 5.0, fielder, x: from.x, y: from.y });
        sim.emit(Input::BallReleasedFromGlove { t: 5.0, fielder, kind: ReleaseKind::Throw, infielder: false });
        sim.out.released.push((5.0, fielder));
        sim.t = 5.9;

        let look = sim.runner_throw_observation(0, 3);
        let release = sim.out.released.pop().unwrap();
        let unsharpened = sim.runner_throw_observation(0, 3);
        sim.out.released.push(release);
        let exact = sim.ball_at(sim.t);
        assert!((look.0 - exact.0).norm() < (unsharpened.0 - exact.0).norm());
        assert!((look.1 - exact.1).norm() < (unsharpened.1 - exact.1).norm());

        let projection = sim.project_runner_throw(look.0, look.1);
        assert_eq!(projection.dest, 2);
        let (second, first) = sim.runner_ball_races(0, 3, (2, 7.0), (1, 6.5));
        assert_eq!((second, first), (projection.time_to(2), projection.time_to(1)));
        // Compare to the crossing on the world flight. The runner has a noisy
        // estimate, not the exact time. First is another 27 m and a relay later.
        let Ball::Thrown { path, .. } = &sim.ball else { panic!("still in flight") };
        let actual_second = path.knots.iter().min_by(|a, b| {
            (a.s.r - base_pos(2)).norm_sq().total_cmp(&(b.s.r - base_pos(2)).norm_sq())
        }).unwrap().t;
        assert!(second >= sim.t && (second - actual_second).abs() < 0.5, "arrival at second {second:.2}, actual {actual_second:.2}");
        assert!((1.2..1.7).contains(&(first - second)), "relay delay {:.2}", first - second);
    }

    // Owner's play (seed 24583, play 81): from center field second base and home lie on one
    // line, so a runner who took a throw's destination to be the base most nearly along its
    // heading read a throw to second as going home. He judges where it comes down from what
    // he sees of it (agent-models §6.4), whoever throws and wherever it goes, early in its
    // flight and late.
    #[test]
    fn a_runner_reads_the_base_a_throw_is_coming_down_at() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let origins = [("left", Vec3::new(-40.0, 55.0, 0.0)), ("center", Vec3::new(0.0, 80.0, 0.0)), ("right", Vec3::new(40.0, 55.0, 0.0))];
        for (name, from) in origins {
            for b in [2u8, 3, 4] {
                let mut st = fx.st.clone();
                let mut sim = new_sim(&inp, &mut st);
                throw_to_base(&mut sim, from, b, 40.0);
                for dt in [0.0, 0.3, 0.8] {
                    sim.t = 5.0 + dt;
                    assert_eq!(project_fixture_throw(&sim).dest, b, "a throw from {name} field to base {b}, {dt} s after release");
                }
            }
        }
    }

    // Codex runner-reads review P2-1: the first reader ignored lift and drag, and the error was
    // not a metre or two. These are its counterexamples, each read from release.
    // (a) From shallow right field, (20, 55), a chest-high throw to third at the default
    // right fielder's arm (40.546528 m/s) comes down 13.9 m short of third on a ballistic
    // path, nearer second; with his own drag and backspin he reads third.
    #[test]
    fn a_default_arm_throw_from_shallow_right_field_is_read_as_going_to_third() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let arm = FielderAttrs::average(8).arm;
        assert!((arm - 40.546528).abs() < 1e-5, "the default right fielder's arm: {arm}");
        throw_to_base(&mut sim, Vec3::new(20.0, 55.0, 0.0), 3, arm);
        assert_eq!(project_fixture_throw(&sim).dest, 3);
    }

    // (b) From (−40, 45) to home at 40 m/s the throw passes third 4.2 m up: it cannot be
    // taken there, and so is not timed to third on the way (the ballistic bound said 0.83 s).
    #[test]
    fn a_throw_over_third_on_its_way_home_is_not_catchable_at_third() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        throw_to_base(&mut sim, Vec3::new(-40.0, 45.0, 0.0), 4, 40.0);
        let read = project_fixture_throw(&sim);
        assert_eq!(read.dest, 4);
        assert!(read.pass_time(3).is_none(), "it is {:?} over third", read.track.iter().min_by(|a, c| m::hypot(a.1.x - base_pos(3).x, a.1.y - base_pos(3).y).total_cmp(&m::hypot(c.1.x - base_pos(3).x, c.1.y - base_pos(3).y))).map(|x| x.1.z));
        assert!(read.time_to(3) > read.t_arrive + 0.5, "third by relay, after home");
    }

    // (c) A throw from center that hops 5 m before home reaches it after one bounce, 0.5 m
    // up; read as a flat ballistic throw it landed far short of home and was taken for
    // second's. His own prediction, with the bounce, reads home.
    #[test]
    fn a_throw_that_hops_before_home_is_read_as_going_home() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let from = Vec3::new(0.0, 80.0, 0.0);
        let to = base_pos(4);
        let env = sim.inp.env;
        // The elevation that brings it down 5 m short of home.
        let short = 5.0;
        let d = m::hypot(to.x - from.x, to.y - from.y) - short;
        let (mut lo, mut hi) = (-0.3f64, 0.5f64);
        let land = |elev: f64| fly_throw(env, throw_state(from + Vec3::new(0.0, 0.0, THROW_RELEASE_Z), to - from, 40.0, elev), 0.0, 8.0).last().s.r.y;
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if land(mid) > 80.0 - d { lo = mid } else { hi = mid }
        }
        let s0 = throw_state(from + Vec3::new(0.0, 0.0, THROW_RELEASE_Z), to - from, 40.0, 0.5 * (lo + hi));
        sim.t = 5.0;
        sim.launch_throw(s0, to, 4, 1, false);
        assert!((land(0.5 * (lo + hi)) - (80.0 - d)).abs() < 0.05);
        assert_eq!(project_fixture_throw(&sim).dest, 4);
    }

    // A sweep of the review's kind: throws from the valid outfield (and shallow, where the
    // throws are hardest to read) to second, third and home at several arms, read at release
    // and 25, 50 and 75% of the way. He reads the base each is going to.
    #[test]
    fn a_runner_reads_throws_across_the_outfield_and_arms() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let (mut n, mut bad) = (0, Vec::new());
        for x in [-60.0f64, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0] {
            for y in [45.0, 55.0, 65.0, 80.0, 95.0, 105.0] {
                if x.abs() > y || fx.field.ball_clearance(x, y, 0.037).dist < 0.5 {
                    continue;
                }
                let from = Vec3::new(x, y, 0.0);
                let pos = if x < -15.0 { 6 } else if x > 15.0 { 8 } else { 7 };
                for b in [2u8, 3, 4] {
                    for speed in [FielderAttrs::average(pos).arm, 40.0, 45.0] {
                        let d = m::hypot(base_pos(b).x - x, base_pos(b).y - y);
                        let Some(sol) = solve_throw(&fx.env, THROW_RELEASE_Z, THROW_CATCH_Z, d, speed) else { continue };
                        for frac in [0.0, 0.25, 0.5, 0.75] {
                            let mut st = fx.st.clone();
                            let mut sim = new_sim(&inp, &mut st);
                            throw_to_base(&mut sim, from, b, speed);
                            sim.t = 5.0 + frac * sol.t_flight;
                            n += 1;
                            let got = project_fixture_throw(&sim).dest;
                            if got != b {
                                bad.push(format!("({x},{y}) to {b} at {speed:.1} m/s, {frac}: read {got}"));
                            }
                        }
                    }
                }
            }
        }
        assert!(n > 300, "{n} samples");
        assert!(bad.is_empty(), "{} of {n} misread: {}", bad.len(), bad.join("; "));
    }

    // The time the runner believes the ball needs follows from that: a throw from center to
    // second reaches second in about its flight time. A throw home on the fly reaches home
    // in its flight time and can reach second by relay.
    #[test]
    fn a_throw_from_center_is_timed_to_the_base_it_goes_to() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let from = Vec3::new(0.0, 84.0, 0.0);
        // When the throw's true path is nearest the base: its flight time there.
        let flight = |sim: &Sim, b: u8| {
            let Ball::Thrown { path, .. } = &sim.ball else { unreachable!() };
            let d = |t: f64| {
                let (p, _) = path.at(t);
                m::hypot(base_pos(b).x - p.x, base_pos(b).y - p.y)
            };
            let t = (0..800).map(|i| 5.0 + i as f64 * 0.005).min_by(|&x, &y| d(x).total_cmp(&d(y))).unwrap();
            t - 5.0
        };
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        throw_to_base(&mut sim, from, 2, 40.0);
        let read = project_fixture_throw(&sim);
        let (to_second, to_home) = (read.time_to(2) - 5.0, read.time_to(4) - 5.0);
        assert!((to_second - flight(&sim, 2)).abs() < 0.1, "second: {to_second:.2} s, flight {:.2} s", flight(&sim, 2));
        assert!(to_home > to_second + 0.6, "home {to_home:.2} s must be a relay after second's {to_second:.2} s");
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        throw_to_base(&mut sim, from, 4, 40.0);
        let read = project_fixture_throw(&sim);
        let (to_home, to_second) = (read.time_to(4) - 5.0, read.time_to(2) - 5.0);
        assert!((to_home - flight(&sim, 4)).abs() < 0.1, "home: {to_home:.2} s, flight {:.2} s", flight(&sim, 4));
        // It passes second 8 m up in the air: nobody can take it there.
        assert!(to_second > to_home + 0.6, "second {to_second:.2} s is a relay after home's {to_home:.2} s");
    }

    // A throw whose line passes a base within reach at catchable height can be caught there on
    // the way: a throw aimed home from the right-field corner that comes through the infield
    // at head height past third is taken to reach third at the time it gets there.
    #[test]
    fn a_throw_passing_a_base_low_can_be_caught_there() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        // A hard throw home from 25 m out in center passes second, 11 m from it on the line,
        // at about chest height: it can be taken there on the way.
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        throw_to_base(&mut sim, Vec3::new(0.0, 50.0, 0.0), 4, 40.0);
        let read = project_fixture_throw(&sim);
        let tau = read.pass_time(2).expect("passes second low") - PASS_CATCH_S - 5.0;
        assert!(tau > 0.1 && tau < 0.5, "{tau}");
        assert!(read.time_to(2) < read.time_to(4), "second before home");
        // Lobbed, it comes over second high, and cannot be caught there.
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let from = Vec3::new(0.0, 50.0, 0.0);
        sim.t = 5.0;
        let s0 = throw_state(from + Vec3::new(0.0, 0.0, THROW_RELEASE_Z), base_pos(4) - from, 30.0, 0.7);
        sim.launch_throw(s0, base_pos(4), 4, 1, false);
        assert!(project_fixture_throw(&sim).pass_time(2).is_none());
        // Already past it, or wide of it, the ball is no threat there: a throw from the right
        // field corner to first does not pass third.
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        throw_to_base(&mut sim, Vec3::new(40.0, 55.0, 0.0), 1, 38.0);
        assert!(project_fixture_throw(&sim).pass_time(3).is_none());
    }

    // Regression from the owner's play (seed 24583, play 81; the game moves under any change
    // to the runner, so it is rebuilt from its facts): a clean single to center, the batter
    // touches first 0.35 s after the center fielder's throw to second is released. He holds at
    // first rather than running into the tag at second.
    #[test]
    fn a_batter_does_not_run_into_a_throw_to_second_from_center() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        throw_to_base(&mut sim, Vec3::new(3.4, 84.5, 0.0), 2, 40.0);
        sim.t = 5.0 + 0.35;
        sim.r[0].s = BASE_L;
        sim.r[0].v = 8.0;
        sim.r[0].goal = 1;
        sim.runner_decide(0, 1);
        assert_eq!(sim.r[0].goal, 1, "he stops at first");
    }

    // Looking again (agent-models §6.4): an unforced runner between bases with the ball
    // secured by a fielder at the base ahead turns back; a forced runner has no choice and
    // goes on.
    #[test]
    fn a_runner_between_bases_looks_again_when_the_ball_is_secured_ahead() {
        // Runner from second, 8 m past it, the third baseman standing on third with the ball.
        let fx = fixture(&[(0x0007, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0007, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 6.0;
        sim.r[1].s = 2.0 * BASE_L + 8.0;
        sim.r[1].v = 7.0;
        sim.r[1].goal = 3;
        sim.r[1].hold_s = None;
        sim.f[4].pos = base_pos(3);
        sim.f[4].target = base_pos(3);
        sim.ball = Ball::Held { by: 4 };
        assert!(!sim.runner_relook(1), "nothing new to see: he keeps going");
        assert_eq!(sim.r[1].goal, 3);
        sim.out.secured.push((6.0, sim.f[4].id));
        assert!(sim.runner_relook(1), "the ball is secured ahead of him");
        assert_eq!(sim.r[1].goal, 2, "he cannot beat the ball to third, and goes back");
        assert!(!sim.runner_relook(1), "one look per event");
    }

    // agent-models §6.3: the same race is run with two out and not with none. A runner from
    // second reaches third at speed with the center fielder holding the ball: the run table's
    // break-even home is 41% with two out and 94% with none (runner decision note §1.5), so
    // over the fielder's depths there is a band where he goes only with two out, never one
    // where he goes only with none; in close he holds either way, from far out he scores.
    #[test]
    fn a_runner_rounding_third_goes_on_a_closer_race_with_two_out() {
        let goes = |outs: u8, depth: f64| {
            let mut fx = fixture(&[(0x0007, 2)]);
            fx.st.outs = outs;
            let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0007, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = 4.2;
            (sim.r[0].s, sim.r[0].v, sim.r[0].goal) = (BASE_L, 0.0, 1);
            (sim.r[1].s, sim.r[1].v, sim.r[1].direction, sim.r[1].goal, sim.r[1].hold_s) = (3.0 * BASE_L, 8.1, 1.0, 3, None);
            sim.f[CF].pos = Vec3::new(0.0, depth, 0.0);
            sim.f[CF].target = sim.f[CF].pos;
            sim.ball = Ball::Held { by: CF };
            sim.runner_decide(1, 3);
            sim.r[1].goal == 4
        };
        assert!(!goes(0, 40.0) && !goes(2, 40.0), "the ball in short center: he holds");
        assert!(goes(0, 115.0) && goes(2, 115.0), "the ball at the wall: he scores");
        let (mut only_two, mut only_none) = (0, 0);
        for i in 0..=75 {
            let d = 40.0 + i as f64;
            match (goes(0, d), goes(2, d)) {
                (false, true) => only_two += 1,
                (true, false) => only_none += 1,
                _ => {}
            }
        }
        assert!(only_two >= 3 && only_none == 0, "depths where only two out sends him: {only_two} m; only none out: {only_none} m");
    }

    // Nobody stops at home: a runner arriving at third is not held up by a runner ahead who
    // is past third and running in, but is by one standing on third.
    #[test]
    fn a_runner_ahead_running_home_is_not_in_the_way() {
        let fx = fixture(&[(0x0007, 2), (0x0008, 3)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0007, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0008, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        (sim.r[2].s, sim.r[2].v, sim.r[2].direction) = (3.0 * BASE_L + 12.0, 8.0, 1.0);
        assert!(!sim.base_uncleared_ahead(1, 4));
        // Codex runner-decision review P1: the runner behind arrives at third at speed with
        // the man ahead a metre past it and barely moving. Following him now would be passing
        // him (the rules would call the passer out): he is in the way.
        (sim.r[1].s, sim.r[1].v, sim.r[1].direction) = (3.0 * BASE_L, 8.1, 1.0);
        (sim.r[2].s, sim.r[2].v) = (3.0 * BASE_L + 1.0, 1.0);
        assert!(sim.base_uncleared_ahead(1, 4), "a slow runner just ahead is in the way");
        (sim.r[2].s, sim.r[2].v) = (3.0 * BASE_L + 12.0, 8.0);
        assert!(!sim.base_uncleared_ahead(1, 4), "twelve metres ahead at speed he is not");
        (sim.r[2].s, sim.r[2].v) = (3.0 * BASE_L, 0.0);
        assert!(sim.base_uncleared_ahead(1, 4), "standing on third, he is");
        (sim.r[2].s, sim.r[2].v, sim.r[2].direction) = (3.0 * BASE_L + 12.0, 6.0, -1.0);
        assert!(sim.base_uncleared_ahead(1, 4), "and coming back to third");
    }

    // Codex runner-decision review P2. Two out, the batter retired; a runner from first is
    // rounding second and the runner ahead is a metre past third on his way home, about
    // 3.3 s from it. Staying at second keeps that run: runner on second, two out, 0.33 from
    // the run table, plus the run, 1.33; third safely is 0.38 plus the run. Put out at third
    // before the run is home, the inning ends with nothing; put out after it, the run stands.
    #[test]
    fn a_run_on_its_way_home_is_lost_to_an_earlier_third_out() {
        let mut fx = fixture(&[(0x0006, 1), (0x0008, 3)]);
        fx.st.outs = 2;
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0006, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0008, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 4.0;
        sim.r[0].out = true;
        (sim.r[1].s, sim.r[1].v, sim.r[1].direction) = (2.0 * BASE_L, 8.1, 1.0);
        (sim.r[2].s, sim.r[2].v, sim.r[2].direction) = (3.0 * BASE_L + 1.0, 8.0, 1.0);
        let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
        let v = sim.advance_values(1, 2, sim.t + 2.0);
        assert!(near(v.hold, 1.33) && near(v.safe, 1.38) && near(v.out, 0.0), "{v:?}");
        assert!(v.break_even() > 0.96);
        let v = sim.advance_values(1, 2, sim.t + 5.0);
        assert!(near(v.hold, 1.33) && near(v.safe, 1.38) && near(v.out, 1.0), "{v:?}");
        // With one out the run counts either way, and the out leaves the bases empty with two out.
        sim.st.outs = 1;
        let v = sim.advance_values(1, 2, sim.t + 2.0);
        assert!(near(v.hold, 0.71 + 1.0) && near(v.safe, 0.98 + 1.0) && near(v.out, 0.10 + 1.0), "{v:?}");
    }

    // agent-models §6.3, the defense's play (owner, 2026-11-15: the situation first). The
    // center fielder has the ball 70 m out; a runner is a few steps past third, a close play
    // at the plate. The batter-runner nearing second expects the throw to go home, so the
    // ball reaches second only by the cut (`CUT_EXTRA_S` after a straight throw) or a second
    // throw from the plate. With the lead runner standing on third, or sure to beat the
    // throw home, there is no play ahead and the straight throw is his to beat.
    #[test]
    fn a_trailing_runner_expects_the_throw_to_go_to_the_play_ahead() {
        let fx = fixture(&[(0x0007, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0007, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 5.0;
        sim.f[CF].pos = Vec3::new(0.0, 70.0, 0.0);
        sim.f[CF].target = sim.f[CF].pos;
        sim.ball = Ball::Held { by: CF };
        (sim.r[0].s, sim.r[0].v, sim.r[0].direction, sim.r[0].goal) = (2.0 * BASE_L - 6.0, 8.0, 1.0, 2);
        (sim.r[1].s, sim.r[1].v, sim.r[1].direction, sim.r[1].goal, sim.r[1].hold_s) = (3.0 * BASE_L, 0.0, 1.0, 3, None);
        let straight = sim.ball_time_to(0, 2, 1, sim.t + 0.75);
        assert!((0.7..1.2).contains(&(straight - sim.t)), "a throw of about 31 m, the ball already in his hand: {:.2} s", straight - sim.t);
        let home = sim.ball_time_to(1, 4, 1, f64::INFINITY) - sim.t;
        assert!((1.8..2.6).contains(&home), "70 m to the plate: {home:.2} s");
        // The lead runner where a typical runner's legs bring him home 0.4 s after the ball:
        // tagged out more often than not, a play the defense makes.
        let place = |sim: &mut Sim, after_ball_s: f64| {
            let d = (0..2700).map(|i| i as f64 * 0.01).find(|&d| runner_eta(d, 8.0) >= home + after_ball_s).unwrap();
            (sim.r[1].s, sim.r[1].v, sim.r[1].goal) = (4.0 * BASE_L - d, 8.0, 4);
        };
        place(&mut sim, 0.4);
        let expecting = sim.ball_time_to(0, 2, 1, sim.t + 0.75);
        assert!((expecting - (straight + CUT_EXTRA_S)).abs() < 1e-9, "by the cut: {:.2} s against {:.2} s straight", expecting - sim.t, straight - sim.t);
        // The lead runner home 0.3 s ahead of the ball: no play at the plate, and the throw
        // is the trailing runner's to beat again.
        place(&mut sim, -0.3);
        assert_eq!(sim.ball_time_to(0, 2, 1, sim.t + 0.75), straight);
    }

    #[test]
    fn the_secondary_lead_joins_the_live_runner_track_on_all_three_bases() {
        let runners = [(0x0004, 1), (0x0005, 2), (0x0006, 3)];
        let fx = fixture(&runners);
        let mut entrants = vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU }];
        entrants.extend(runners.map(|(id, base)| RunnerIn { id, base, speed: 8.2, tau_s: RUNNER_TAU }));
        let mut inp = input(&fx, entrants);
        inp.runner_start.fewer_than_two = SecondaryLead { extra_m: 1.0, speed_mps: 2.0 };
        inp.record = true;
        let mut st = fx.st.clone();
        let (out, ticks) = runner_trace::capture(|| simulate_live(&inp, &mut st));
        for (id, base) in runners {
            let track = &out.actors.iter().find(|(agent, _)| *agent == id).unwrap().1;
            let along = (base_pos(base + 1) - base_pos(base)).normalized();
            let contact = track.iter().position(|(t, _)| *t == inp.t_contact).unwrap();
            let (at_release, at_contact) = (track[0], track[contact]);
            assert_eq!(at_release.0, 0.0);
            assert!(track.windows(2).all(|pair| pair[0].0 < pair[1].0));
            assert!(((at_contact.1 - at_release.1).dot(along) - 1.0).abs() < 1e-9);
            let before = track[contact - 1];
            let approach_mps = (at_contact.1 - before.1).dot(along) / (at_contact.0 - before.0);
            assert!((approach_mps - 2.0).abs() < 0.2);
            let start = ticks.iter().find(|tick| tick.runner == id).unwrap();
            let later = ticks.iter().find(|tick| tick.runner == id && tick.t > inp.t_contact + 0.24).unwrap();
            assert_eq!(start.signed_mps, 2.0);
            // At the start of the 15th elapsed tick, fourteen ticks of the 2 m/s
            // shuffle have passed. The runner has covered 14 × 2/60 m.
            assert!((later.s - start.s - 14.0 * 2.0 / 60.0).abs() < 1e-9);
            if base == 3 {
                // The third-base line is x + y = 0. The contact position is
                // 0.6 m to its foul side while the feet head parallel to it.
                assert!((at_contact.1.x + at_contact.1.y + 0.6 * 2.0f64.sqrt()).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn a_catch_during_the_contact_read_sends_the_moving_runner_back() {
        let fx = fixture(&[(0x0007, 2)]);
        let mut inp = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: 0x0007, base: 2, speed: 8.2, tau_s: RUNNER_TAU },
        ]);
        inp.runner_start.fewer_than_two = SecondaryLead { extra_m: 1.0, speed_mps: 2.0 };
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.start_runners();
        sim.caught_at = Some(sim.t);
        sim.r[1].off_base_at_catch = true;
        let start = sim.r[1].s;
        for _ in 0..14 {
            sim.t += DT;
            sim.move_runners();
            assert!(sim.r[1].reading_contact);
        }
        // Fourteen steps at 2 m/s precede the fifteenth tick, the signal reaction.
        assert!((sim.r[1].s - start - 14.0 * 2.0 / 60.0).abs() < 1e-9);
        let reaction_pos = sim.r[1].s;
        for _ in 0..30 {
            sim.t += DT;
            sim.move_runners();
        }
        let runner = &sim.r[1];
        assert!(runner.retouch && !runner.reading_contact);
        assert_eq!(runner.goal, 2);
        // 2 m/s brakes to zero in 0.421 s after 0.421 m, to within one tick's
        // integration error. At 0.5 s he has begun returning.
        assert!(runner.direction < 0.0);
        assert!((0.33..0.44).contains(&(runner.s - reaction_pos)));
    }

    // With two out a catch ends the inning: the runner on second runs on contact on a high
    // fly to center and has touched third before it comes down. With none out he waits.
    #[test]
    fn with_two_out_a_runner_does_not_wait_on_a_fly() {
        let touched_third = |outs: u8| {
            let mut fx = fixture(&[(0x0007, 2)]);
            fx.st.outs = outs;
            let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0007, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
            inp.runner_start.two_out = SecondaryLead { extra_m: 1.5, speed_mps: 4.0 };
            let mut st = fx.st.clone();
            let out = simulate_live(&inp, &mut st);
            let caught = out.facts.iter().find_map(|f| if let Input::GloveSecured { t, .. } = f { Some(*t) } else { None }).expect("the fly is caught");
            out.facts.iter().find_map(|f| match f {
                Input::BaseTouched { t, agent: 0x0007, base: 3, .. } if *t < caught => Some(*t - inp.t_contact),
                _ => None,
            })
        };
        // He coasts 1 m for 0.25 s, then needs 18.432 m starting at 4 m/s.
        // Integrating the exponential speed curve gives 2.625 s for that leg.
        let t = touched_third(2).expect("he runs");
        assert!((t - 2.875).abs() < 0.05, "third {t:.2} s after contact");
        assert!(touched_third(0).is_none());
    }

    // Codex runner-reads review P2-2: turning back is a stop and a restart. A runner 8 m past
    // second at +7 m/s who turns back does not move backward in the first tick, brakes at
    // `BRAKE` through zero, and gets to second when his own estimate says (`run_time_from`).
    #[test]
    fn a_runner_who_turns_back_stops_first_and_arrives_when_he_expected() {
        let fx = fixture(&[(0x0007, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0007, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 6.0;
        let s0 = 2.0 * BASE_L + 8.0;
        sim.r[1].s = s0;
        sim.r[1].v = 7.0;
        sim.r[1].direction = 1.0;
        sim.r[1].goal = 3;
        sim.r[1].hold_s = None;
        sim.f[4].pos = base_pos(3);
        sim.f[4].target = base_pos(3);
        sim.ball = Ball::Held { by: 4 };
        sim.out.secured.push((6.0, sim.f[4].id));
        sim.r[1].v = 7.0;
        assert!(sim.runner_relook(1));
        assert_eq!(sim.r[1].goal, 2);
        let expected = run_time_to_stop(8.0, -7.0, sim.r[1].vmax, RUNNER_TAU);
        let t0 = sim.t;
        sim.move_runners();
        sim.t += DT;
        assert!(sim.r[1].s >= s0, "no backward step in the first tick: {} against {s0}", sim.r[1].s);
        assert!((sim.r[1].v - (7.0 - BRAKE * DT)).abs() < 1e-9, "braking: v = {}", sim.r[1].v);
        let mut peak = sim.r[1].s;
        let mut ticks = 1;
        while sim.r[1].s > 2.0 * BASE_L + 1e-9 && ticks < 600 {
            sim.move_runners();
            sim.t += DT;
            peak = peak.max(sim.r[1].s);
            ticks += 1;
        }
        assert!((peak - s0 - 49.0 / 9.5).abs() < 0.1, "he drifts v²/2·BRAKE = 5.16 m on before reversing: {}", peak - s0);
        assert!(((sim.t - t0) - expected).abs() < 3.0 * DT, "took {:.3} s, expected {expected:.3} s", sim.t - t0);
    }

    // Until the rundown (TASKS.md, Q11), a runner can reverse once between bases. This
    // prevents the defense from keeping him between the same bases until the play times out.
    #[test]
    fn a_runner_turns_round_once_between_bases() {
        let fx = fixture(&[(0x0007, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0007, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 6.0;
        sim.r[1].s = 2.0 * BASE_L + 8.0;
        sim.r[1].v = 7.0;
        sim.r[1].direction = 1.0;
        sim.r[1].goal = 3;
        sim.r[1].hold_s = None;
        sim.f[4].pos = base_pos(3);
        sim.f[4].target = base_pos(3);
        sim.ball = Ball::Held { by: 4 };
        sim.out.secured.push((6.0, sim.f[4].id));
        sim.r[1].turns = MAX_TURNS;
        assert!(sim.runner_relook(1));
        assert_eq!(sim.r[1].goal, 3, "he has turned once already: he stays committed");
        assert_eq!(sim.r[1].turns, MAX_TURNS);
    }

    // Codex runner-reads recheck P2-4 (`review_cap_refuses_believed_safe_escape`): a runner
    // who has used his turn, retreating toward second where the ball is held, 0.5 m short of
    // third at 1 m/s, finds third a sure thing (0.5 s to run it against the ball's 1.1 s). The
    // one-turn cap is for a trapped runner; he takes third.
    #[test]
    fn the_turn_cap_does_not_refuse_a_base_he_believes_safe() {
        let fx = fixture(&[(7, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 7, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = traffic_sim(&inp, &mut st);
        (sim.r[1].s, sim.r[1].v, sim.r[1].direction, sim.r[1].goal) = (3.0 * BASE_L - 0.5, 1.0, -1.0, 2);
        sim.r[1].turns = MAX_TURNS;
        sim.f[3].pos = base_pos(2);
        sim.f[3].target = base_pos(2);
        sim.ball = Ball::Held { by: 3 };
        sim.secured_at = 5.0;
        sim.out.secured.push((sim.t, sim.f[3].id));
        assert!(sim.runner_relook(1));
        assert_eq!(sim.r[1].goal, 3, "a sure base is not a rundown");
    }

    // Codex runner-reads recheck P2-2 (`review_retouch_plant`): a runner coming back to retouch
    // brakes into the bag. He never loses more than BRAKE a tick, arrives at no more than
    // the approach speed, and is planted there (v = 0) when the next leg is decided, so
    // its estimate starts from rest.
    #[test]
    fn a_runner_retouching_a_base_brakes_into_it_and_leaves_from_rest() {
        let fx = fixture(&[(7, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 7, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 6.0;
        sim.r[0].out = true;
        sim.caught_at = Some(sim.t - 0.25);
        sim.r[1].off_base_at_catch = true;
        sim.r[1].catch_seen = true;
        sim.r[1].retouch = true;
        (sim.r[1].s, sim.r[1].v, sim.r[1].direction, sim.r[1].goal, sim.r[1].hold_s) = (2.0 * BASE_L + 12.0, 7.0, -1.0, 2, None);
        sim.f[6].pos = Vec3::new(-120.0, 150.0, 0.0);
        sim.ball = Ball::Held { by: 6 };
        let (mut v_prev, mut worst_drop, mut v_in) = (7.0, 0.0f64, f64::NAN);
        let t0 = sim.t;
        for _ in 0..400 {
            let s_before = sim.r[1].s;
            sim.move_runners();
            sim.t += DT;
            let v = sim.r[1].v;
            if sim.r[1].s - 2.0 * BASE_L > 1e-6 {
                worst_drop = worst_drop.max(v_prev - v);
                v_in = v;
            } else if s_before - 2.0 * BASE_L > 1e-6 {
                break;
            }
            v_prev = v;
        }
        assert!(worst_drop <= BRAKE * DT + 1e-9, "braked {worst_drop} m/s in a tick");
        assert!(v_in <= APPROACH_MIN_MPS + BRAKE * DT, "arrived at {v_in} m/s");
        assert_eq!(sim.r[1].v, 0.0, "planted on the bag");
        // His estimate of the trip is the trip: braking in, from 7 m/s and 12 m out.
        let expected = run_time_to_stop(12.0, 7.0, sim.r[1].vmax, RUNNER_TAU);
        assert!(((sim.t - t0) - expected).abs() < 3.0 * DT, "took {:.3} s, expected {expected:.3} s", sim.t - t0);
    }

    // The same second look while he is already returning: his signed velocity is used on both
    // sides of the race (going on from here would mean a stop first), so he keeps coming back.
    #[test]
    fn a_second_look_while_returning_counts_the_stop() {
        let fx = fixture(&[(0x0007, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0007, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 6.0;
        sim.r[1].s = 2.0 * BASE_L + 4.0;
        sim.r[1].v = 6.0;
        sim.r[1].direction = -1.0;
        sim.r[1].goal = 2;
        sim.r[1].hold_s = None;
        // The ball is thrown (a release) from third toward second as he comes back.
        sim.f[4].pos = base_pos(3);
        sim.f[4].target = base_pos(3);
        sim.ball = Ball::Held { by: 4 };
        sim.out.released.push((6.0, sim.f[4].id));
        assert!(sim.runner_relook(1));
        assert_eq!(sim.r[1].goal, 2, "on the way back, going on means stopping first: he keeps coming");
        // Moving back at 6 m/s, the time on includes the stop and the 4 m given up.
        let on = run_time_from(BASE_L - 4.0, -6.0, sim.r[1].vmax, RUNNER_TAU);
        let from_rest = run_time_from(BASE_L - 4.0, 0.0, sim.r[1].vmax, RUNNER_TAU);
        assert!(on > from_rest + 0.9, "{on} against {from_rest}");
    }

    // Codex runner-reads review P1-1 (its probe `review_late_crowding`): runners from first and
    // second, the batter out, the third baseman holding the ball on third. The lead runner
    // is 8 m past second at 7 m/s; the trailing runner is 0.2 m short of second at 8.2 m/s
    // and arrives after the lead runner's look. The trailing runner cannot stop short of
    // the base, so the lead runner continues and keeps second clear.
    #[test]
    fn a_trailing_runner_arriving_after_the_look_does_not_leave_two_on_a_base() {
        let fx = fixture(&[(8, 1), (7, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 8, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 7, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.emit(Input::BallTouched { t: 0.5, what: Touch::Ground { x: 0.0, y: 1.0 } });
        sim.emit(Input::BallPassedBase { t: 0.55, fair: true });
        sim.emit(Input::BaseLeft { t: 0.6, agent: 8, base: 1 });
        sim.emit(Input::BaseLeft { t: 0.6, agent: 7, base: 2 });
        sim.emit(Input::TagContact { t: 1.0, fielder: sim.f[4].id, runner: fx.st.batter, holding_ball: true });
        sim.t = 6.0;
        assert!(sim.r[0].out);
        for r in &mut sim.r {
            r.hold_s = None;
        }
        (sim.r[1].s, sim.r[1].v, sim.r[1].goal) = (2.0 * BASE_L - 0.2, 8.2, 2);
        (sim.r[2].s, sim.r[2].v, sim.r[2].goal) = (2.0 * BASE_L + 8.0, 7.0, 3);
        sim.f[4].pos = base_pos(3);
        sim.ball = Ball::Held { by: 4 };
        sim.out.secured.push((sim.t, sim.f[4].id));
        for _ in 0..400 {
            sim.move_runners();
            sim.t += DT;
            if sim.done {
                break;
            }
        }
        let t = sim.t;
        sim.emit(Input::PlayEndCheck { t });
        assert!(!sim.out.events.iter().any(|e| matches!(e, Event::RulesWarning { code } if *code == "two_runners_on_base")), "events {:?}", sim.out.events);
    }

    // The other side of it: a trailing runner far enough back to stop gives way to a lead
    // runner visibly coming back, and goes back to his own base if it is free.
    #[test]
    fn a_trailing_runner_gives_way_to_a_lead_runner_coming_back() {
        let fx = fixture(&[(8, 1), (7, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 8, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 7, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 6.0;
        sim.r[0].out = true;
        for r in &mut sim.r {
            r.hold_s = None;
        }
        // Trailing runner 20 m short of second heading there; the lead runner 3 m past it,
        // already coming back at 5 m/s.
        (sim.r[1].s, sim.r[1].v, sim.r[1].direction, sim.r[1].goal) = (BASE_L + 7.0, 8.0, 1.0, 2);
        (sim.r[2].s, sim.r[2].v, sim.r[2].direction, sim.r[2].goal) = (2.0 * BASE_L + 3.0, 5.0, -1.0, 2);
        sim.move_runners();
        assert_eq!(sim.r[1].goal, 1, "he goes back to first, which is free");
        assert_eq!(sim.r[2].goal, 2);
    }

    /// A sim for the traffic cases: the runners left their bases and the batter is out (all
    /// told to the rules engine), the catcher holds the ball, nobody is holding on a fly.
    fn traffic_sim<'a>(inp: &'a LiveInput<'a>, st: &'a mut RulesState) -> Sim<'a> {
        let mut sim = new_sim(inp, st);
        sim.emit(Input::BallTouched { t: 0.5, what: Touch::Ground { x: 0.0, y: 1.0 } });
        sim.emit(Input::BallPassedBase { t: 0.55, fair: true });
        for r in inp.runners.iter().filter(|r| r.base > 0) {
            sim.emit(Input::BaseLeft { t: 0.6, agent: r.id, base: r.base });
        }
        sim.emit(Input::TagContact { t: 1.0, fielder: sim.f[4].id, runner: inp.runners[0].id, holding_ball: true });
        sim.t = 6.0;
        for r in &mut sim.r {
            r.hold_s = None;
            r.tag_up = false;
            r.waiting = false;
        }
        sim.f[1].pos = base_pos(4);
        sim.ball = Ball::Held { by: 1 };
        sim
    }

    fn no_two_on_a_base(sim: &Sim) -> bool {
        !sim.out.events.iter().any(|e| matches!(e, Event::RulesWarning { code } if *code == "two_runners_on_base"))
    }

    // Codex runner-reads recheck P1-1 (its `review_three_runner_traffic`): three runners from
    // first, second and third, the batter out. The one from first holds second, the one from
    // second is 7 m past it at 8 m/s, the one from third holds third. The held-base branch
    // sent the middle runner back to the occupied second, both stopped on it, and the rules
    // engine lost a runner. The guard where bodies stop settles it: the one standing on
    // second gives way to first, which is free.
    #[test]
    fn three_runners_do_not_settle_two_on_second() {
        let fx = fixture(&[(9, 1), (8, 2), (7, 3)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 9, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 8, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 7, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = traffic_sim(&inp, &mut st);
        sim.emit(Input::BaseTouched { t: 5.0, agent: 9, base: 2, holding_ball: false });
        sim.emit(Input::BaseTouched { t: 5.0, agent: 7, base: 3, holding_ball: false });
        (sim.r[1].s, sim.r[1].v, sim.r[1].goal) = (2.0 * BASE_L, 0.0, 2);
        (sim.r[2].s, sim.r[2].v, sim.r[2].goal) = (2.0 * BASE_L + 7.0, 8.0, 3);
        (sim.r[3].s, sim.r[3].v, sim.r[3].direction, sim.r[3].goal) = (3.0 * BASE_L, 0.0, -1.0, 3);
        sim.r[3].still_since = Some(5.0);
        for _ in 0..600 {
            sim.move_runners();
            sim.t += DT;
            if sim.done {
                break;
            }
        }
        let t = sim.t;
        sim.emit(Input::PlayEndCheck { t });
        assert!(no_two_on_a_base(&sim), "events {:?} bases {:?}", sim.out.events, sim.st.bases);
    }

    // Codex runner-reads recheck P2-3 (`review_distant_batter_crowding`): the batter, 5 m from
    // home at 3 m/s, can stop at first and threatens nothing at second, 50 m ahead of him; his
    // having no way back to home must not make second "crowded" for the runner 8 m past it.
    #[test]
    fn a_distant_batter_does_not_crowd_the_return_to_second() {
        let fx = fixture(&[(7, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 7, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 6.0;
        for r in &mut sim.r {
            r.hold_s = None;
        }
        (sim.r[0].s, sim.r[0].v, sim.r[0].goal) = (5.0, 3.0, 1);
        (sim.r[1].s, sim.r[1].v, sim.r[1].goal) = (2.0 * BASE_L + 8.0, 7.0, 3);
        sim.f[4].pos = base_pos(3);
        sim.ball = Ball::Held { by: 4 };
        sim.out.secured.push((6.0, sim.f[4].id));
        assert!(!sim.return_crowded(1, 2));
        assert!(sim.runner_relook(1));
        assert_eq!(sim.r[1].goal, 2, "he returns to second");
    }

    /// A random runner layout, as the property test below builds it (seeded, so a failing case
    /// replays): 1–3 runners from distinct bases, the batter out or still running, each at
    /// a random place along the paths in order of start, with a random speed, direction and
    /// goal; the ball held by a random fielder at a random place. Returns a description.
    fn random_traffic_case(case: u64) -> Result<(), String> {
        let mut x = case.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03;
        let mut rnd = move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x >> 11) as f64 / (1u64 << 53) as f64
        };
        let n_run = 1 + (rnd() * 3.0) as usize;
        // Distinct start bases from {1, 2, 3}, in order.
        let mut bases: Vec<u8> = vec![1, 2, 3];
        while bases.len() > n_run {
            bases.remove((rnd() * bases.len() as f64) as usize % bases.len());
        }
        let ids: Vec<AgentId> = bases.iter().map(|b| 0x0010 + *b as AgentId).collect();
        let on: Vec<(AgentId, u8)> = ids.iter().copied().zip(bases.iter().copied()).collect();
        let fx = fixture(&on);
        let mut runners = vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }];
        runners.extend(on.iter().map(|&(id, b)| RunnerIn { id, base: b, speed: 7.0 + 2.0 * rnd(), tau_s: bb_agents::running::LEAGUE_TAU_S }));
        let inp = input(&fx, runners);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let batter_runs = rnd() < 0.4;
        sim.emit(Input::BallTouched { t: 0.5, what: Touch::Ground { x: 0.0, y: 1.0 } });
        sim.emit(Input::BallPassedBase { t: 0.55, fair: true });
        if !batter_runs {
            sim.emit(Input::TagContact { t: 1.0, fielder: sim.f[4].id, runner: fx.st.batter, holding_ball: true });
        }
        sim.t = 6.0;
        // Places, lead runner furthest along, at least 1.5 m apart.
        let mut at: Vec<f64> = Vec::new();
        for &b in bases.iter().rev() {
            let lo = at.last().map(|p| p + 1.5).unwrap_or(b as f64 * BASE_L).max(b as f64 * BASE_L);
            let hi = ((b + 2).min(4)) as f64 * BASE_L;
            if lo >= hi {
                return Ok(());
            }
            let on_base = rnd() < 0.3;
            let pos = if on_base { (lo / BASE_L).ceil() * BASE_L } else { lo + (hi - lo) * rnd() };
            at.push(pos.min(4.0 * BASE_L - 1.0));
        }
        at.reverse();
        // (`at` is now in increasing start order; the lead runner has the highest position
        // only if the places were built lead-first, so rebuild in that order.)
        let mut places: Vec<(u8, f64)> = Vec::new();
        for (i, &b) in bases.iter().enumerate().rev() {
            places.push((b, at[bases.len() - 1 - i]));
        }
        places.sort_by_key(|p| p.0);
        let mut tt = 1.0;
        for (k, &(b, pos)) in places.iter().enumerate() {
            let _ = k;
            let j = sim.r.iter().position(|r| r.start == b).unwrap();
            let (floor_b, standing) = ((pos / BASE_L + 1e-9).floor() as u8, (pos - (pos / BASE_L).round() * BASE_L).abs() < 1e-6);
            // Tell the rules engine where he has been.
            if pos > b as f64 * BASE_L + 1e-6 {
                tt += 0.01;
                sim.emit(Input::BaseLeft { t: tt, agent: sim.r[j].id, base: b });
            }
            for bb in (b + 1)..=floor_b.min(3) {
                tt += 0.01;
                sim.emit(Input::BaseTouched { t: tt, agent: sim.r[j].id, base: bb, holding_ball: false });
                if !(standing && bb == floor_b) {
                    tt += 0.01;
                    sim.emit(Input::BaseLeft { t: tt, agent: sim.r[j].id, base: bb });
                }
            }
            let v = if rnd() < 0.3 { 0.0 } else { 1.0 + 8.0 * rnd() };
            let dir = if rnd() < 0.5 { 1.0 } else { -1.0 };
            let goal = if standing { floor_b + (rnd() < 0.4) as u8 } else { floor_b + (rnd() < 0.5) as u8 };
            let r = &mut sim.r[j];
            r.s = pos;
            r.v = if standing { 0.0 } else { v };
            r.direction = dir;
            r.goal = goal.clamp(b, 4);
            r.hold_s = None;
            r.tag_up = false;
            r.waiting = false;
        }
        if batter_runs {
            let r = &mut sim.r[0];
            r.s = rnd() * BASE_L * 0.9;
            r.v = 1.0 + 6.0 * rnd();
            r.direction = 1.0;
            r.goal = 1;
        }
        // The ball, held by a random fielder at a random base or spot.
        let by = (rnd() * 9.0) as usize % 9;
        let spot = match (rnd() * 5.0) as u8 {
            0 => base_pos(1),
            1 => base_pos(2),
            2 => base_pos(3),
            3 => base_pos(4),
            _ => Vec3::new(-20.0 + 40.0 * rnd(), 20.0 + 60.0 * rnd(), 0.0),
        };
        sim.f[by].pos = spot;
        sim.f[by].target = spot;
        sim.ball = Ball::Held { by };
        sim.out.secured.push((6.0, sim.f[by].id));
        for tick in 0..1200u32 {
            sim.move_runners();
            if tick % 2 == 0 {
                sim.sample_runners();
            }
            if sim.done {
                break;
            }
            sim.t += DT;
            // The play changes under them now and then: a throw is released.
            if tick % 240 == 120 {
                let id = sim.f[by].id;
                sim.out.released.push((sim.t, id));
            }
        }
        let t = sim.t;
        sim.emit(Input::PlayEndCheck { t });
        if let Some(w) = sim.out.events.iter().find_map(|e| if let Event::RulesWarning { code } = e { Some(*code) } else { None }) {
            return Err(format!("case {case}: rules warning {w}; runners {:?}; bases {:?}", sim.r.iter().map(|r| (r.id, r.start, r.s / BASE_L, r.goal, r.out)).collect::<Vec<_>>(), sim.st.bases));
        }
        for r in sim.r.iter().filter(|r| !r.out && !r.scored) {
            let d = (r.s / BASE_L - (r.s / BASE_L).round()).abs() * BASE_L;
            if d > 1e-6 && r.overrun == OverrunPhase::None {
                return Err(format!("case {case}: runner {} stopped between bases at {:.2} bases (goal {})", r.id, r.s / BASE_L, r.goal));
            }
        }
        let on_bases: Vec<i64> = sim.r.iter().filter(|r| !r.out && !r.scored).map(|r| (r.s / BASE_L).round() as i64).filter(|&b| (1..=3).contains(&b)).collect();
        for (i, a) in on_bases.iter().enumerate() {
            if on_bases[i + 1..].contains(a) {
                return Err(format!("case {case}: two live runners on base {a}"));
            }
        }
        Ok(())
    }

    /// Catch-read fixes review P1-4: the settle guard reads another runner's motion, not his
    /// goal. The runner from second stands on third; the runner from third is coming back
    /// at it, 0.30 m off at 2 m/s, too fast to stop short. Whether that lead runner means
    /// to stay at third or turn for home is his own; the trailing runner gives way either
    /// way. Only a visible change (the lead runner far off and slow) keeps him on third.
    #[test]
    fn the_settle_guard_reads_motion_not_another_runners_goal() {
        const TRAIL: AgentId = 7;
        const LEAD: AgentId = 8;
        let fx = fixture(&[(TRAIL, 2), (LEAD, 3)]);
        let inp = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: TRAIL, base: 2, speed: 8.2, tau_s: RUNNER_TAU },
            RunnerIn { id: LEAD, base: 3, speed: 8.2, tau_s: RUNNER_TAU },
        ]);
        let guard = |lead_goal: u8, off: f64, speed: f64| {
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = 5.0;
            let third = 3.0 * BASE_L;
            let r = &mut sim.r[1];
            (r.s, r.v, r.direction, r.goal, r.still_since) = (third, 0.0, 1.0, 3, Some(4.0));
            let r = &mut sim.r[2];
            (r.s, r.v, r.direction, r.goal) = (third + off, speed, -1.0, lead_goal);
            sim.settle_guard(1);
            sim.r[1].goal
        };
        assert_eq!(guard(3, 0.30, 2.0), 2, "a lead runner coming onto the bag: give way");
        assert_eq!(guard(4, 0.30, 2.0), guard(3, 0.30, 2.0), "the lead runner's private goal moved the guard");
        assert_eq!(guard(3, 8.0, 1.0), 3, "a lead runner far off and slow is no conflict yet");
    }

    // Codex runner-reads recheck P1-1, as a property: over seeded random layouts (runners,
    // places, speeds, directions, goals, a batter out or running so that a force may stand, the
    // ball held anywhere, fresh looks as it is secured and thrown) no two live runners rest on
    // one base, every live runner ends on a base, and the rules engine never warns.
    #[test]
    fn random_runner_layouts_never_leave_two_on_a_base() {
        let mut bad = Vec::new();
        for case in 0..3000u64 {
            if let Err(e) = random_traffic_case(case) {
                bad.push(e);
            }
        }
        assert!(bad.is_empty(), "{} of 3000 failed; first: {}", bad.len(), bad[..bad.len().min(6)].join("\n"));
    }

    // Seed 1000003226 play 194 (bases loaded, the force at second made, the throw home): the
    // runner from third looked again and went back to third as the runner from second reached
    // it, two on one base (rules warning). The base behind him is no refuge when a runner who
    // started behind him is on it.
    #[test]
    fn a_runner_does_not_go_back_to_a_base_a_trailing_runner_stands_on() {
        let fx = fixture(&[(0x0008, 1), (0x0007, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0008, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0007, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 6.0;
        // The batter is out, so no force: the lead runner, 8 m past second, with the trailing
        // runner standing on second behind him and the third baseman holding the ball on third.
        sim.r[0].out = true;
        sim.r[1].s = 2.0 * BASE_L;
        sim.r[1].v = 0.0;
        sim.r[2].s = 2.0 * BASE_L + 8.0;
        sim.r[2].v = 7.0;
        sim.r[2].goal = 3;
        sim.r[2].hold_s = None;
        sim.f[4].pos = base_pos(3);
        sim.f[4].target = base_pos(3);
        sim.ball = Ball::Held { by: 4 };
        sim.out.secured.push((6.0, sim.f[4].id));
        assert!(!sim.forced(2));
        assert!(sim.runner_relook(2));
        assert_eq!(sim.r[2].goal, 3, "second is no refuge with a runner standing on it");
    }

    #[test]
    fn a_forced_runner_between_bases_goes_on_when_the_ball_is_secured_ahead() {
        // Runner from first, forced by the batter, 8 m toward second; the second baseman has the ball on the bag.
        let fx = fixture(&[(0x0007, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 0x0007, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 6.0;
        sim.r[1].s = BASE_L + 8.0;
        sim.r[1].v = 7.0;
        sim.r[1].goal = 2;
        sim.r[1].hold_s = None;
        sim.f[3].pos = base_pos(2);
        sim.f[3].target = base_pos(2);
        sim.ball = Ball::Held { by: 3 };
        sim.out.secured.push((6.0, sim.f[3].id));
        assert!(sim.forced(1));
        assert!(!sim.runner_relook(1), "he has no choice");
        assert_eq!(sim.r[1].goal, 2);
    }

    // Review of 2026-11-11 (motions), finding 1: a backup standing between the base and the
    // receiver (who has stepped past the bag to meet the throw) must leave a catchable throw
    // to the receiver.
    #[test]
    fn backup_in_front_of_the_receiver_leaves_him_the_throw() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
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
        let at = sim.f[3].pos;
        aimed_throw(&mut sim, Vec3::new(0.0, 80.0, 0.0), at, 39.0);
        for _ in 0..110 {
            sim.update_thrown();
            sim.try_captures();
            sim.t += DT;
        }
        assert!(matches!(sim.ball, Ball::Held { by: 3 }), "ball: {:?}", sim.ball);
    }

    /// A grounder to short; the throw pulls the first baseman off the bag into foul ground,
    /// so the batter-runner is safe, stops at first (the ball is a few metres away), and runs
    /// through the bag. The first baseman tags him on his way back, 3 m short of the bag.
    /// Returns whether that tag put him out, his deepest point past the bag (m), his speed
    /// crossing it, whether the rules engine saw him protected, and the facts and events.
    fn overrun_with_tag(fx: &Fixture, samples: bool) -> (bool, f64, f64, bool, Vec<Input>, Vec<Event>) {
        const FB: usize = 2;
        let bat = fx.st.batter;
        let inp = input(fx, vec![RunnerIn { id: bat, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.emit(Input::BallTouched { t: 0.8, what: Touch::Ground { x: -8.0, y: 30.0 } });
        sim.emit(Input::GloveSecured { t: 1.3, fielder: sim.f[5].id, x: -8.0, y: 30.0 });
        let off_bag = base_pos(1) + Vec3::new(1.5, -1.5, 0.0);
        sim.f[FB].pos = off_bag;
        sim.f[FB].target = off_bag;
        sim.emit(Input::GloveSecured { t: 3.9, fielder: sim.f[FB].id, x: off_bag.x, y: off_bag.y });
        sim.ball = Ball::Held { by: FB };
        sim.secured_at = 3.9;
        sim.t = 4.0;
        sim.r[0].s = BASE_L - 3.0;
        sim.r[0].goal = 1;
        sim.r[0].v = 8.0;
        let (mut deepest, mut v_cross, mut tagged, mut protected_seen) = (0.0f64, 0.0, false, false);
        for tick in 0..600u64 {
            let before = sim.r[0].overrun;
            let v = sim.r[0].v;
            sim.move_runners();
            if before == OverrunPhase::None && sim.r[0].overrun == OverrunPhase::Out {
                v_cross = v + (sim.r[0].vmax - v) * (DT / RUNNER_TAU);
            }
            if samples && tick % SAMPLE_EVERY_TICKS == 0 {
                sim.sample_runners();
            }
            if let PlayState::Live(lp) = &sim.st.play {
                protected_seen |= lp.runners.iter().any(|r| r.runner == bat && r.overrun == bb_rules::Overrun::Protected);
            }
            deepest = deepest.max(sim.r[0].over_m);
            if sim.r[0].overrun != OverrunPhase::None {
                // His recorded position is on the first-base line, past the bag.
                let p = sim.runner_pos(&sim.r[0]);
                assert!((p.x - p.y).abs() < 1e-9 && p.x >= base_pos(1).x - 1e-9, "{p:?}");
            }
            if !tagged && sim.r[0].overrun == OverrunPhase::Back && sim.r[0].over_m < 3.0 {
                tagged = true;
                let (fb, t) = (sim.f[FB].id, sim.t);
                sim.emit(Input::TagContact { t, fielder: fb, runner: bat, holding_ball: true });
            }
            if sim.r[0].out || (tagged && sim.r[0].overrun == OverrunPhase::None) {
                break;
            }
            sim.t += DT;
        }
        let t = sim.t + 1.0;
        sim.emit(Input::PlayEndCheck { t });
        let out = sim.out.events.iter().any(|e| matches!(e, Event::RunnerOut { runner, .. } if *runner == bat));
        (out, deepest, v_cross, protected_seen, sim.out.facts.clone(), sim.out.events.clone())
    }

    // Rules-spec §5.5 / OBR 5.09(b)(4), simulator side: the batter-runner who stops at first
    // runs through the bag, pulls up down the line, and walks back; with the live sim's
    // position samples the rules engine sees an overrun and a tag on his way straight back is
    // no out. He is left on first.
    #[test]
    fn batter_runner_overruns_first_and_is_not_liable_coming_back() {
        let fx = fixture(&[]);
        let bat = fx.st.batter;
        let (out, deepest, v_cross, protected_seen, facts, events) = overrun_with_tag(&fx, true);
        assert!(!out, "tagged out coming straight back: {events:?}");
        assert!(protected_seen);
        // Braking from his crossing speed at the overrun rate: v²/(2a) past the bag.
        let stop = v_cross * v_cross / (2.0 * OVERRUN_DECEL);
        assert!(v_cross > 7.5 && (deepest - stop).abs() < 0.15, "deepest {deepest:.2} m, v {v_cross:.2}, kinematic {stop:.2}");
        // First touched, left, touched again, in that order, and nothing else of his.
        let mine: Vec<(bool, u8)> = facts
            .iter()
            .filter_map(|f| match *f {
                Input::BaseTouched { agent, base, .. } if agent == bat => Some((true, base)),
                Input::BaseLeft { agent, base, .. } if agent == bat => Some((false, base)),
                _ => None,
            })
            .collect();
        assert_eq!(mine, vec![(true, 1), (false, 1), (true, 1)]);
        assert!(events.iter().any(|e| matches!(e, Event::RunnerAdvanced { runner, to: 1, .. } if *runner == bat)), "{events:?}");
        assert!(!events.iter().any(|e| matches!(e, Event::RulesWarning { .. })), "{events:?}");
    }

    // The same play without position samples: the rules engine has no evidence of an overrun
    // (rules-spec §5.5), so he is an ordinary runner off the bag and the tag retires him. The
    // samples are what protect him.
    #[test]
    fn without_position_samples_the_overrunner_is_an_ordinary_runner() {
        let fx = fixture(&[]);
        let (out, _, _, protected_seen, _, events) = overrun_with_tag(&fx, false);
        assert!(!protected_seen);
        assert!(out, "{events:?}");
    }

    // Finding 2 of the same review: the batter's realized track starts in his box and joins
    // the first-base line; nothing downstream has to move him.
    #[test]
    fn batter_runs_out_of_his_box_onto_the_line() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
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

    // Q1 (tag-up review P2-3). The league-average runner (8.2 m/s, τ 0.76 s) covers a standing
    // 90 ft in 4.10 s: t − 0.76·(1 − e^(−t/0.76)) = 27.432/8.2 = 3.34537 gives t = 4.1019. And
    // each batter is charged his own route: from the right-handed box (−0.95, 0.30) to the
    // line's 4 m point (2.8284, 2.8284) is √(3.7784² + 2.5284²) = 4.5464 m, from the
    // left-handed box (0.95, 0.30) √(1.8784² + 2.5284²) = 3.1499 m, so they start 0.5464 m
    // behind and 0.8501 m ahead of the plate's zero and the left-hander's run is 1.3965 m
    // shorter. A runner with a quicker τ of his own gets there sooner.
    #[test]
    fn runners_start_by_their_own_tau_and_batters_from_their_own_box() {
        assert!((run_time(BASE_L, 8.2, bb_agents::running::LEAGUE_TAU_S) - 4.1019).abs() < 1e-3);
        assert!((batter_start_s(Vec3::new(-0.95, 0.30, 0.0)) + 0.5464).abs() < 1e-3);
        assert!((batter_start_s(Vec3::new(0.95, 0.30, 0.0)) - 0.8501).abs() < 1e-3);
        let first_touch = |bx: f64, tau_s: f64| {
            let fx = fixture(&[]);
            let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s }]);
            inp.batter_box = Vec3::new(bx, 0.30, 0.0);
            let mut st = fx.st.clone();
            let out = simulate_live(&LiveInput { record: true, ..inp }, &mut st);
            let track = &out.actors.iter().find(|(id, _)| *id == fx.st.batter).unwrap().1;
            // The first recorded instant he is within a stride of the bag.
            track.iter().find(|(_, p)| (*p - base_pos(1)).norm() < 0.5).map(|(t, _)| *t).expect("he reaches first")
        };
        let (right, left, quick) = (first_touch(-0.95, 0.76), first_touch(0.95, 0.76), first_touch(-0.95, 0.60));
        // 1.3965 m at 8.2 m/s is 0.170 s; the track is sampled every 0.1 s.
        assert!((right - left - 0.170).abs() < 0.11, "right {right:.2} s, left {left:.2} s");
        // τ 0.16 s less is 0.16 s sooner once he is at full speed.
        assert!((right - quick - 0.16).abs() < 0.11, "τ 0.76 {right:.2} s, τ 0.60 {quick:.2} s");
    }

    // Codex runner-acceleration review P1-1, P2-1. A right-handed batter's coordinate passes
    // zero about half a second after contact, 0.78 m from the plate (review probe). Zero is
    // not a bag: a catcher holding the ball on the plate, his tag ready and the batter within
    // his reach, tags him, and the batter's run out of the box records no base touched.
    #[test]
    fn the_batter_passing_coordinate_zero_is_on_no_base() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: 0.76 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.r[0].goal = 1;
        sim.t = inp.t_contact;
        sim.emit(Input::BallTouched { t: sim.t, what: Touch::Ground { x: 0.0, y: 0.5 } });
        sim.emit(Input::GloveSecured { t: sim.t, fielder: sim.f[1].id, x: 0.0, y: 0.0 });
        sim.ball = Ball::Held { by: 1 };
        sim.secured_at = sim.t;
        sim.f[1].pos = Vec3::ZERO;
        assert!(sim.r[0].s < 0.0);
        while sim.r[0].s < 0.0 {
            sim.move_runners();
            sim.t += DT;
        }
        assert!(!sim.out.facts.iter().any(|f| matches!(f, Input::BaseTouched { .. })), "{:?}", sim.out.facts);
        assert!(sim.runner_pos(&sim.r[0]).norm() < sim.f[1].a.throw_choice.tag_reach_m);
        assert!(sim.t >= sim.secured_at + sim.f[1].a.tag_apply_s);
        sim.holder_act();
        assert!(sim.out.facts.iter().any(|f| matches!(f, Input::TagContact { .. })), "{:?}", sim.out.facts);
        assert!(sim.r[0].out);
    }

    /// The ball leaves the throwing hand (owner backlog A1): facing his target, a
    /// right-hander's hand is on his right and a left-hander's on his left, `THROW_ARM_OFFSET_M`
    /// out, at release height. Worked in the sim frame (+X first base, +Y center field).
    /// The center fielder at (0, 80) with the dive flag up and his get-up due at `up_s`,
    /// the shortstop on second, nobody running. A slow roller passes half a metre in front
    /// of him from time 1.5, starting `x0` along its path. Runs whole ticks (captures, the
    /// holder, movement) until he throws; returns the sim and the release time.
    fn center_fielder_takes_a_roller(up_s: f64, x0: f64) -> (Sim<'static>, f64) {
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&[])));
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }])));
        let st: &'static mut RulesState = Box::leak(Box::new(fx.st.clone()));
        let mut sim = new_sim(inp, st);
        sim.f[CF].pos = Vec3::new(0.0, 80.0, 0.0);
        loose_ball(&mut sim, 1.5, Vec3::new(x0, 79.5, 0.04), Vec3::new(6.0, 0.0, 0.0));
        (sim.f[5].pos, sim.f[5].target) = (base_pos(2), base_pos(2));
        sim.f[CF].role = Role::Chase;
        sim.f[CF].dove = true;
        sim.f[CF].ready_at = up_s;
        for _ in 0..240 {
            world_tick(&mut sim);
            sim.holder_act();
            if matches!(sim.ball, Ball::Thrown { .. }) {
                let t = sim.t;
                return (sim, t);
            }
        }
        panic!("no throw");
    }

    // Codex hurried-throws review #1. He takes the roller lying down from a dive, up at
    // 2.5 s. Facing the ball (it comes from −X), second base is straight off a
    // right-hander's glove shoulder: set = 0.85 − 0.12 + 0.30 after a dive = 1.03 s after he
    // is up, so the throw leaves at 3.53 s, and it is recorded as an off-balance set-up.
    // (His recovery flag clears at 3.35 s; the footwork must outlast it.)
    #[test]
    fn dive_footwork_lasts_until_the_throw() {
        let (sim, t) = center_fielder_takes_a_roller(2.5, -3.0);
        assert!(t >= 3.53 - 1e-9 && t < 3.53 + 2.0 * DT, "released at {t:.3}");
        assert_eq!(sim.out.hurry, vec![(0.0, true)]);
    }

    // And a man whose get-up ended half a tick before he takes a ball standing owes no dive
    // footwork, though his recovery flag is still up: set 0.85 − 0.12 = 0.73 s after 1.5 s.
    #[test]
    fn a_recovered_fielder_owes_no_dive_footwork() {
        let (sim, t) = center_fielder_takes_a_roller(1.5 - 0.5 * DT, 0.0);
        assert!(secured_at(&sim) == Some(1.5), "secured {:?}", secured_at(&sim));
        assert!(t >= 2.23 - 1e-9 && t < 2.23 + 2.0 * DT, "released at {t:.3}");
        assert_eq!(sim.out.hurry, vec![(0.0, false)]);
    }

    // Codex hurried-throws review #2: a first baseman 5 m from his bag with the ball carries
    // it there as soon as he has it in hand (0.65 s), without first getting set for a throw
    // he will not make (0.77 s, turning off his throwing shoulder).
    #[test]
    fn carrying_the_ball_to_the_bag_waits_for_no_throw_set_up() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 1.0 + 0.66;
        let at = base_pos(1) + Vec3::new(5.0, 0.0, 0.0);
        held_at(&mut sim, 2, at);
        sim.f[0].pos = base_pos(1);
        let fd = &mut sim.f[2];
        (fd.up_at, fd.ready_at, fd.set_base, fd.face) = (1.0, 1.65, 0.65, Some(Vec3::new(0.0, -1.0, 0.0)));
        // A4: a genuinely safe carry. A 1.2 s runner instead makes the throw the
        // better option; proximity no longer overrides that comparison.
        (sim.r[0].s, sim.r[0].v, sim.r[0].goal) = (BASE_L - 8.2 * 3.0, 8.2, 1);
        assert!((sim.set_up_for(2, base_pos(1)).set_s - 0.77).abs() < 1e-9);
        sim.holder_act();
        assert!((sim.f[2].target - base_pos(1)).norm() < 1e-9, "target {:?}", sim.f[2].target);
    }

    /// Third baseman 3 m from third with the ball at 2.0 s, on his feet since 1.0; the first
    /// baseman on his bag; the batter 4 m out of the box and an unforced runner (first is
    /// open) 10 m past second, both running at 6 m/s. Both outs are sure by any timing: the
    /// runner needs about 2.3 s to third against about 0.6 s for the holder, the batter
    /// about 3.3 s to first against a 38 m throw.
    fn third_baseman_with_an_unforced_runner_coming(outs: u8) -> Sim<'static> {
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&[(7, 2)])));
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 7, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }])));
        let st: &'static mut RulesState = Box::leak(Box::new(fx.st.clone()));
        st.outs = outs;
        let mut sim = new_sim(inp, st);
        sim.t = 2.0;
        held_at(&mut sim, 4, base_pos(3) + Vec3::new(3.0, 0.0, 0.0));
        sim.f[2].pos = base_pos(1);
        sim.f[4].up_at = 1.0;
        (sim.r[0].s, sim.r[0].v, sim.r[0].goal) = (4.0, 6.0, 1);
        (sim.r[1].s, sim.r[1].v, sim.r[1].goal) = (2.0 * BASE_L + 10.0, 6.0, 3);
        sim.holder_act();
        sim
    }

    // agent-models §5.6 A4: carrying to an unforced runner's bag is a tag plan weighed as a
    // tag, in the same comparison as the throws. With two out a sure force beats a sure tag
    // (1.0 against `TWO_OUT_TAG_W` 0.9): he throws to first. With fewer than two out the
    // lead runner at third is worth more than the batter (1.1 against 0.9), and a fielder
    // 3 m from the bag with the runner 17 m away tags him there.
    #[test]
    fn an_unforced_runner_is_a_tag_weighed_against_the_force() {
        let sim = third_baseman_with_an_unforced_runner_coming(2);
        assert!(matches!(sim.ball, Ball::Thrown { to_base: 1, .. }), "two out: the force at first");
        let sim = third_baseman_with_an_unforced_runner_coming(0);
        assert!(matches!(sim.ball, Ball::Held { by: 4 }), "none out: he keeps the ball");
        assert_eq!(sim.f[4].carry_to, Some(3));
        assert!((sim.f[4].target - base_pos(3)).norm() < 1e-9, "and takes it to third for the tag");
    }

    // agent-models §5.6 A4 (the measuring note's seed 13 / play 140): bases loaded, one out,
    // the first baseman 3 m from his bag. Carrying it there is a near-sure out on the batter,
    // but it takes the force off every other base. The runner from third is 24 m from home
    // and the catcher is on the plate: a sure force there is worth 1.3 against first's 0.9.
    #[test]
    fn a_sure_force_at_home_beats_carrying_first() {
        let fx = fixture(&[(5, 1), (6, 2), (7, 3)]);
        let runners = vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 6, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 7, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }];
        let inp = input(&fx, runners);
        let mut st = fx.st.clone();
        st.outs = 1;
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        held_at(&mut sim, 2, base_pos(1) + Vec3::new(-1.0, 2.8, 0.0));
        sim.f[1].pos = base_pos(4);
        sim.f[2].up_at = 0.5;
        // The batter 1.2 s from first: no time for a second out there after any force.
        (sim.r[0].s, sim.r[0].v, sim.r[0].goal) = (BASE_L - 8.2 * 1.2, 8.2, 1);
        for k in 1..4 {
            let r = &mut sim.r[k];
            (r.s, r.v, r.goal) = (r.start as f64 * BASE_L + 3.0, 6.0, r.start + 1);
        }
        sim.holder_act();
        assert_eq!(sim.f[2].carry_to, None, "he must not run it to first");
        assert!(matches!(sim.ball, Ball::Thrown { to_base: 4, .. }), "the force at home");
    }

    // agent-models §5.5, speed for accuracy: with the batter 3 s from first the shortstop's
    // 33 m throw beats him by far more than the 0.25 s margin even at 80% of his arm, so he
    // throws at the measured floor (and more accurately); in a race he wins by 0.1 s letting
    // go as early as he can, no effort keeps that margin, so he throws everything he has.
    #[test]
    fn a_fielder_throws_as_hard_as_the_play_needs() {
        let (easy, _) = shortstop_to_first(3.0, Vec3::ZERO);
        assert_eq!(easy.out.effort, vec![throwing::EFFORT_MIN]);
        let (close, _) = shortstop_to_first(eta_for_margin(0.10, false), Vec3::ZERO);
        assert_eq!(close.out.effort, vec![1.0]);
    }

    /// The shortstop holds the ball at (−8, 38) at time 1.0, on his feet, having taken it
    /// standing (no turn counted), the first baseman on his bag, and the batter `eta_s` from
    /// first at full speed. Steps the holder and the runner until the ball is thrown; returns
    /// the sim and how long after he had the ball he let go.
    fn shortstop_to_first(eta_s: f64, catch_vel: Vec3) -> (Sim<'static>, f64) {
        shortstop_to_first_by(eta_s, catch_vel, bb_observe::Hand::Right)
    }

    /// [`shortstop_to_first`] for a shortstop who throws with `hand`.
    fn shortstop_to_first_by(eta_s: f64, catch_vel: Vec3, hand: bb_observe::Hand) -> (Sim<'static>, f64) {
        const SS: usize = 5;
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&[])));
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }])));
        let st: &'static mut RulesState = Box::leak(Box::new(fx.st.clone()));
        let mut sim = new_sim(inp, st);
        sim.f[SS].a.throws = hand;
        sim.t = 1.0;
        held_at(&mut sim, SS, Vec3::new(-8.0, 38.0, 0.0));
        sim.f[2].pos = base_pos(1);
        let fd = &mut sim.f[SS];
        (fd.up_at, fd.set_base, fd.face, fd.catch_vel) = (1.0, fd.a.transfer, None, catch_vel);
        (sim.r[0].s, sim.r[0].v, sim.r[0].goal) = (BASE_L - 8.2 * eta_s, 8.2, 1);
        for _ in 0..120 {
            sim.holder_act();
            if matches!(sim.ball, Ball::Thrown { .. }) {
                let after = sim.t - 1.0;
                return (sim, after);
            }
            sim.r[0].s += 8.2 * DT;
            sim.t += DT;
        }
        panic!("no throw");
    }

    /// The batter's time to first that leaves the shortstop of [`shortstop_to_first`]
    /// `margin_s` in hand if he lets go as early as he can, on balance or not.
    fn eta_for_margin(margin_s: f64, off_balance: bool) -> f64 {
        let d = (base_pos(1) - Vec3::new(-8.0, 38.0, 0.0)).norm();
        // The shortstop's arm: Statcast's 86.1 mph × 0.44704 = 38.490 m/s (agent-models §10).
        let v = 38.490 * if off_balance { 0.82 } else { 0.90 };
        0.6 * 0.65 + throw_time(v, d) + 0.05 + margin_s
    }

    // Owner backlog A3, agent-models §5.5. An average infielder's catch-to-release is 0.65 s;
    // the earliest he can let go is 0.6 of it, 0.39 s. With three seconds on the runner he
    // gets set and throws at 0.65 s, unhurried.
    #[test]
    fn a_fielder_with_time_gets_set_before_he_throws() {
        let (sim, after) = shortstop_to_first(3.0, Vec3::ZERO);
        assert!(after >= 0.65 - 1e-9 && after < 0.65 + DT + 1e-9, "let go after {after:.3} s");
        assert_eq!(sim.out.hurry, vec![(0.0, false)]);
    }

    // With only a tenth of a second in hand at his quickest, he lets go as early as he can.
    #[test]
    fn a_fielder_short_of_time_hurries_his_throw() {
        let (sim, after) = shortstop_to_first(eta_for_margin(0.10, false), Vec3::ZERO);
        assert!(after >= 0.39 - 1e-9 && after < 0.39 + DT + 1e-9, "let go after {after:.3} s");
        let (hurry, off) = sim.out.hurry[0];
        assert!(hurry > 0.93 && !off, "hurry {hurry:.2}");
    }

    // In between he uses the time he has: 0.35 s in hand at his quickest, 0.25 s kept, so
    // he has a tenth to spend. Waiting also gives him his arm back (a set throw of 33 m
    // gets there about 0.1 s sooner than a fully hurried one), so each 0.1 s he waits
    // costs him only about half of it: he lets go some 0.2 s after his earliest, mostly set.
    #[test]
    fn a_fielder_uses_the_time_he_has() {
        let (sim, after) = shortstop_to_first(eta_for_margin(0.35, false), Vec3::ZERO);
        assert!(after > 0.39 + 0.11 && after < 0.65 - 0.03, "let go after {after:.3} s");
        let hurry = sim.out.hurry[0].0;
        assert!(hurry > 0.1 && hurry < 0.6, "hurry {hurry:.2}");
    }

    // Taking the ball moving away from first at 5 m/s he must stop before he plants (agent-
    // models §5.5). By the movement law's own stop (`step_toward` braking a body from 5 m/s
    // and from the 2 m/s no-charge point to rest), he owes the difference: set at 0.65 s plus
    // that when he has the time (the flat rule had 0.85 s).
    #[test]
    fn a_fielder_moving_away_needs_longer_to_get_set() {
        let away = (Vec3::new(-8.0, 38.0, 0.0) - base_pos(1)).normalized() * 5.0;
        let (sim, after) = shortstop_to_first(3.0, away);
        let set = 0.65 + law_stop_s(5.0) - law_stop_s(2.0);
        assert!(after >= set - DT - 1e-9 && after < set + 2.0 * DT + 1e-9, "let go after {after:.3} s, set at {set:.3}");
        assert_eq!(sim.out.hurry, vec![(0.0, true)]);
    }

    // The same hurried throw on balance and off it draws the same noise (same seed, play,
    // thrower and throw number), so the two differ exactly by the model's factors: fully
    // hurried, speed 0.90 against 0.82 of his arm, and direction error ×2 against ×3.
    #[test]
    fn an_off_balance_hurried_throw_is_slower_and_wilder() {
        let from = Vec3::new(-8.0, 38.0, 0.0);
        let away = (from - base_pos(1)).normalized() * 5.0;
        let thrown = |off: bool| {
            let (sim, after) = shortstop_to_first(eta_for_margin(0.05, off), if off { away } else { Vec3::ZERO });
            let (hurry, was_off) = sim.out.hurry[0];
            assert!(was_off == off && after < 0.39 + DT + 1e-9, "after {after:.3}");
            let (_, v) = sim.ball_at(sim.t);
            // Signed angle off the line from his hand to the bag.
            let line = base_pos(1) - throw_release_point(from, base_pos(1), bb_observe::Hand::Right);
            let yaw = m::atan2(line.x * v.y - line.y * v.x, line.x * v.x + line.y * v.y);
            (hurry, v.norm(), yaw)
        };
        let ((h_on, v_on, yaw_on), (h_off, v_off, yaw_off)) = (thrown(false), thrown(true));
        // Each let go on the first tick after his earliest, a shade under full hurry.
        let speed = |h: f64, loss: f64| 1.0 - h * loss;
        assert!((v_off / v_on - speed(h_off, 0.18) / speed(h_on, 0.10)).abs() < 1e-9, "speeds {v_on:.2} and {v_off:.2}");
        assert!((yaw_off / yaw_on - (1.0 + 2.0 * h_off) / (1.0 + h_on)).abs() < 1e-6, "errors {yaw_on:.5} and {yaw_off:.5}");
        assert!(v_off < 0.95 * v_on && yaw_off.abs() > 1.3 * yaw_on.abs());
    }

    // The glove hand (agent-models §5.5): a first baseman who fields a grounder facing the
    // plate turns a half turn to throw to second if he is right-handed, and not at all if
    // he is left-handed: 0.12 s more than his plain transfer against 0.12 s less.
    #[test]
    fn a_left_handed_first_baseman_is_set_sooner_for_second() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        // Placed where the turn is exact: facing −Y at (38.8, 38.8), second base (0, 38.8)
        // is due −X, straight off his right shoulder.
        let at = Vec3::new(38.8, 38.8, 0.0);
        sim.f[2].pos = at;
        (sim.f[2].set_base, sim.f[2].face) = (0.65, Some(Vec3::new(0.0, -1.0, 0.0)));
        sim.f[2].a.throws = bb_observe::Hand::Right;
        let right = sim.set_up_for(2, base_pos(2));
        sim.f[2].a.throws = bb_observe::Hand::Left;
        let left = sim.set_up_for(2, base_pos(2));
        assert!((right.set_s - 0.77).abs() < 1e-3 && (left.set_s - 0.53).abs() < 1e-3, "right {:.3}, left {:.3}", right.set_s, left.set_s);
    }

    #[test]
    fn a_throw_leaves_the_throwing_hand() {
        use bb_observe::Hand;
        let at = Vec3::new(5.0, 30.0, 0.0);
        // Toward center field (+Y): his right is first-base side (+X).
        let r = throw_release_point(at, Vec3::new(5.0, 60.0, 0.0), Hand::Right);
        assert!((r - Vec3::new(5.35, 30.0, 1.8)).norm() < 1e-12, "{r:?}");
        let l = throw_release_point(at, Vec3::new(5.0, 60.0, 0.0), Hand::Left);
        assert!((l - Vec3::new(4.65, 30.0, 1.8)).norm() < 1e-12, "{l:?}");
        // Across the diamond toward first (+X), as from third: his right is toward the plate (−Y).
        let r = throw_release_point(at, Vec3::new(25.0, 30.0, 0.0), Hand::Right);
        assert!((r - Vec3::new(5.0, 29.65, 1.8)).norm() < 1e-12, "{r:?}");
        // Home (toward −Y from the outfield): his right is third-base side (−X).
        let r = throw_release_point(at, Vec3::new(5.0, 0.0, 0.0), Hand::Right);
        assert!((r - Vec3::new(4.65, 30.0, 1.8)).norm() < 1e-12, "{r:?}");
    }

    /// A ball fielded fair by the shortstop (index 5) with a runner (id 2) on first, who has
    /// not left yet; the batter has not reached first.
    fn fielded_by_ss(fx: &Fixture) -> Sim<'_> {
        let inp = Box::leak(Box::new(input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 2, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }])));
        let st = Box::leak(Box::new(fx.st.clone()));
        let mut sim = new_sim(inp, st);
        sim.emit(Input::BallTouched { t: 1.0, what: Touch::Ground { x: 0.0, y: 40.0 } });
        let ss = sim.f[5].id;
        sim.emit(Input::GloveSecured { t: 1.5, fielder: ss, x: -10.0, y: 38.0 });
        sim.ball = Ball::Held { by: 5 };
        sim.t = 2.0;
        sim
    }

    /// From (15, 25), 2.5 m up and fair, a throw at 30 m/s straight off the first-base line
    /// rising at 3 m/s reaches the stands wall (50 ft off the line, 4 ft high; 22.3 m away,
    /// at s = 28.3 m, past the dugout) after about 0.75 s at about 2 m: over the top. That is
    /// a thrown ball out of play on the shortstop's first play with the batter not yet at
    /// first: two bases from the time of the pitch, runner to third, batter to second
    /// (rules-spec §7.2; physics-spec §9.5).
    #[test]
    fn a_throw_over_the_stands_wall_is_dead_with_two_bases_from_the_pitch() {
        let fx = fixture(&[(2, 1)]);
        let mut sim = fielded_by_ss(&fx);
        let ss = sim.f[5].id;
        sim.emit(Input::BallReleasedFromGlove { t: 2.0, fielder: ss, kind: ReleaseKind::Throw, infielder: true });
        let out = Vec3::new(1.0, -1.0, 0.0).normalized();
        sim.launch_throw(BallState { r: Vec3::new(15.0, 25.0, 2.5), v: out * 30.0 + Vec3::new(0.0, 0.0, 3.0), w: Vec3::ZERO }, Vec3::new(40.0, 0.0, 0.0), 1, 2, false);
        let Ball::Thrown { path, .. } = &sim.ball else { panic!("a throw in flight") };
        let t_wall = path.last().t;
        for _ in 0..400 {
            sim.update_thrown();
            if sim.done {
                break;
            }
            world_tick(&mut sim);
            if sim.done {
                break;
            }
        }
        assert!(sim.done, "the play ends when the ball is dead");
        let ev = &sim.out.events;
        assert!(ev.contains(&Event::BallDead { reason: bb_rules::DeadBall::OutOfPlay }), "{ev:?}");
        assert!(ev.contains(&Event::RunnerAdvanced { runner: 2, from: 1, to: 3, cause: bb_rules::AdvanceCause::Award }), "{ev:?}");
        assert!(ev.contains(&Event::RunnerAdvanced { runner: fx.st.batter, from: 0, to: 2, cause: bb_rules::AdvanceCause::Award }), "{ev:?}");
        // It left over the wall: the out-of-play point is where it crossed, its center one
        // ball radius short of the wall's plane, 50 ft off the line.
        // Dead as it crosses the wall (a throw is not caught in the seats), and from above the
        // wall's 4-ft top.
        let oop = sim.out.facts.iter().find_map(|f| match f {
            Input::BallTouched { t, what: Touch::OutOfPlay { x, y, z, .. } } => Some((*t, *x, *y, *z)),
            _ => None,
        });
        let (t, x, y, z) = oop.expect("an out-of-play fact");
        assert!((t - t_wall).abs() < 1e-9, "out of play at {t}, the wall at {t_wall}");
        assert!(z > 4.0 * bb_math::units::FT + fx.env.ball.radius && z < 2.5, "crossed at {z:.2} m");
        let c = (x - y) / core::f64::consts::SQRT_2;
        assert!((c - (50.0 * bb_math::units::FT - fx.env.ball.radius)).abs() < 1e-3, "({x:.2}, {y:.2}): {c:.4} m off the line");
    }

    /// The same throw from 1.0 m, flatter (35 m/s, rising 2 m/s), meets the wall about 0.3 m
    /// up: below the 4-ft top it caroms off with 0.35 of its speed into the wall (the stands
    /// wall's restitution) and stays live.
    #[test]
    fn a_low_throw_caroms_off_the_stands_wall_and_stays_live() {
        let fx = fixture(&[(2, 1)]);
        let mut sim = fielded_by_ss(&fx);
        let ss = sim.f[5].id;
        sim.emit(Input::BallReleasedFromGlove { t: 2.0, fielder: ss, kind: ReleaseKind::Throw, infielder: true });
        let out = Vec3::new(1.0, -1.0, 0.0).normalized();
        sim.launch_throw(BallState { r: Vec3::new(15.0, 25.0, 1.0), v: out * 35.0 + Vec3::new(0.0, 0.0, 2.0), w: Vec3::ZERO }, Vec3::new(40.0, 0.0, 0.0), 1, 2, false);
        let Ball::Thrown { path, .. } = &sim.ball else { panic!("a throw in flight") };
        assert_eq!(path.stop, Some(1), "the throw ends at the wall, not the ground");
        let v_in = path.last().s.v.dot(out);
        for _ in 0..200 {
            sim.update_thrown();
            if matches!(sim.ball, Ball::Free { .. }) {
                break;
            }
            world_tick(&mut sim);
        }
        assert!(!sim.done, "a carom is not out of play");
        assert!(sim.out.facts.iter().any(|f| matches!(f, Input::BallTouched { what: Touch::Wall { .. }, .. })));
        let (_, v) = sim.ball_at(sim.t);
        assert!((v.dot(out) / v_in + 0.35).abs() < 0.02, "rebound {:.2} of {v_in:.1} m/s", v.dot(out) / v_in);
    }

    /// A fielder sent after a ball in the seats stops at the stands wall: his center 0.3 m (his
    /// body radius) inside it, 50 ft off the line (physics-spec §9.5).
    #[test]
    fn a_fielder_chasing_into_the_seats_stops_at_the_wall() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        const B1: usize = 2;
        sim.f[B1].pos = Vec3::new(22.0, 20.0, 0.0);
        sim.f[B1].vel = Vec3::ZERO;
        sim.f[B1].role = Role::Cover(1);
        sim.f[B1].target = Vec3::new(45.0, 0.0, 0.0);
        for _ in 0..300 {
            sim.move_fielders();
            sim.t += DT;
        }
        let p = sim.f[B1].pos;
        let c = (p.x - p.y) / core::f64::consts::SQRT_2;
        assert!((c - (50.0 * bb_math::units::FT - 0.3)).abs() < 1e-3, "{c:.4} m off the line at {p:?}");
        assert!(sim.f[B1].vel.dot(Vec3::new(1.0, -1.0, 0.0).normalized()) <= 1e-9, "{:?}", sim.f[B1].vel);
    }

    // Handedness of a live throw (Codex handedness review: no left-handed thrower had been
    // checked through a play). agent-models §5.5: the ball leaves 0.35 m to his arm side of
    // his body's centre, facing his target. The shortstop at (-8, 38) throws to first base
    // at (d, d), d = 27.432/√2 = 19.3978 m. By hand: the line is (27.3978, -18.6022), length
    // 33.115 m, unit (0.82734, -0.56174). A man facing along (hx, hy) has his right at
    // (hy, -hx), here (-0.56174, -0.82734). Right-hander: (-8, 38) + 0.35·that =
    // (-8.19661, 37.71043); left-hander, the mirror about the body's centre:
    // (-7.80339, 38.28957). Release height 1.8 m (`THROW_RELEASE_Z`).
    #[test]
    fn a_left_handed_fielder_releases_from_his_left_and_a_right_handed_one_from_his_right() {
        use bb_observe::Hand;
        let ss = Vec3::new(-8.0, 38.0, 0.0);
        let release = |hand| {
            let (sim, _) = shortstop_to_first_by(3.0, Vec3::ZERO, hand);
            let Ball::Thrown { path, .. } = &sim.ball else { unreachable!() };
            path.at(sim.t).0
        };
        let (r, l) = (release(Hand::Right), release(Hand::Left));
        assert!((r - Vec3::new(-8.19661, 37.71043, 1.8)).norm() < 2e-5, "right-hander {r:?}");
        assert!((l - Vec3::new(-7.80339, 38.28957, 1.8)).norm() < 2e-5, "left-hander {l:?}");
        // Mirrored about his centre, each 0.35 m out.
        assert!(((r + l) * 0.5 - Vec3::new(ss.x, ss.y, 1.8)).norm() < 1e-9);
        assert!((m::hypot(r.x - ss.x, r.y - ss.y) - 0.35).abs() < 1e-9);
    }

    /// The center fielder (hands σ `hands`, average otherwise) standing at (0, 80) with a
    /// ground ball crossing in front of him at 20 m/s, `pass_m` away at its closest, first
    /// seen `lead_s` before it. His belief is his own read of the true path (`Sim::predict`, with his
    /// perception noise), replaced by `swap` (a ball he
    /// would take on his feet) at the first tick the ball is within `swap_within_m` of him.
    fn crossing_with_mind_change(pass_m: f64, lead_s: f64, hands: Option<f64>, swap_within_m: f64, easy_from_m: f64, swap: bool) -> Sim<'static> {
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&[])));
        let mut inp = input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        if let Some(h) = hands {
            inp.fattrs[CF].hands = h;
        }
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(inp));
        let st: &'static mut RulesState = Box::leak(Box::new(fx.st.clone()));
        let mut sim = new_sim(inp, st);
        let (t0, at) = (1.5, Vec3::new(0.0, 80.0, 0.0));
        sim.f[CF].pos = at;
        // The ball he would rather believe in: rolling straight down on him from
        // `easy_from_m` out at 10 m/s, so he has it on his feet in about easy_from_m/10 s.
        loose_ball(&mut sim, t0, at + Vec3::new(0.0, easy_from_m, 0.04), Vec3::new(0.0, -10.0, 0.0));
        let Ball::Free { path: easy, .. } = &sim.ball else { unreachable!() };
        let easy = easy.clone();
        loose_ball(&mut sim, t0, Vec3::new(-20.0 * lead_s, 80.0 - pass_m, 0.04), Vec3::new(20.0, 0.0, 0.0));
        sim.f[CF].role = Role::Chase;
        sim.f[CF].belief = Some(sim.predict(CF));
        let mut swapped = !swap;
        for _ in 0..48 {
            let (b, _) = sim.ball_at(sim.t);
            if !swapped && m::hypot(b.x - at.x, b.y - at.y) <= swap_within_m {
                sim.f[CF].belief = Some(easy.clone());
                swapped = true;
            }
            world_tick(&mut sim);
        }
        sim
    }

    // agent-models §5.3: "A stretch on his feet, up to STRETCH 0.4 m further" and "Whether it
    // gains him anything is his own judgment". A ball passing 1.3 m away (reach 1.0 + 0.4)
    // is stretched for. Having started to wait for the closest tick, a fielder whose read
    // then says he has the ball on his feet no longer stretches (it would gain him nothing):
    // the ball goes by. Without the change of mind he stretches (the control).
    #[test]
    fn a_stretch_is_dropped_when_his_read_changes_during_the_wait() {
        let kept = crossing_with_mind_change(1.3, 0.15, None, 1.4, 3.0, false);
        assert!(secured_at(&kept).is_some() && kept.out.dives.is_empty(), "control: no stretch");
        let changed = crossing_with_mind_change(1.3, 0.15, None, 1.4, 3.0, true);
        // He does not stretch as it goes by (a stretch costs 0.3 s, a dive's get-up 1.5 s). The
        // ball, now past his stretch, is then reached by a dive a few ticks later (his dive
        // read from before the change still stood): the glove closes after the control's.
        let (t_kept, t_changed) = (secured_at(&kept).unwrap(), secured_at(&changed).expect("he lets it go for good"));
        assert!(t_changed > t_kept, "kept {t_kept:.3}, changed {t_changed:.3}");
        let ready = |s: &Sim| s.f[CF].ready_at - secured_at(s).unwrap() - s.f[CF].a.transfer;
        assert!((ready(&kept) - 0.3).abs() < 1e-9, "control owes a stretch's 0.3 s: {}", ready(&kept));
        assert!(ready(&changed) > 0.3 + 1e-9, "after the change: a dive's get-up, not a stretch: {}", ready(&changed));
    }

    // The dive read (§5.3: "every 0.1 s, with the ball within 15 m, he asks whether going
    // for it gains him anything, and the answer has to have held without a break"). A ball
    // 2.2 m away needs 1.2 m of the lunge; seen 0.4 s ahead he dives (a_dive_needs_its_lunge
    // _time). If one of his reads (0.1 s apart) says he has it on his feet, the clock
    // restarts and the lunge is too late: no dive.
    #[test]
    fn a_dive_read_that_changes_resets_the_lunge() {
        let held = crossing_with_mind_change(2.2, 0.4, None, 1e9, 8.0, false);
        assert_eq!(held.out.dives.len(), 1, "control: he dives");
        // The read flips once the ball is within 4.5 m (0.2 s on): the real ball is by him
        // 0.2 s later, too late to lay out 1.2 m (0.24 s) from the restart.
        let flipped = crossing_with_mind_change(2.2, 0.4, None, 4.5, 8.0, true);
        assert!(flipped.out.dives.is_empty(), "dived after a break in his read: {:?}", flipped.out.dives);
    }

    // agent-models §5.4: a stretch outside the pocket and rim is a miss ("the ball continues").
    // Hands this poor (σ 1 m a side) miss a 1.3 m stretch every time: nothing is secured,
    // no dive is made, he does not go down (`dove` stays false) and the ball is still loose.
    #[test]
    fn a_missed_stretch_leaves_the_ball_loose_and_the_fielder_on_his_feet() {
        let sim = crossing_with_mind_change(1.3, 0.15, Some(1.0), 1e9, 3.0, false);
        assert!(secured_at(&sim).is_none(), "poor hands secured it");
        assert!(sim.out.dives.is_empty() && !sim.f[CF].dove);
        assert!(matches!(sim.ball, Ball::Free { .. }));
        assert!(sim.f[CF].attempts >= 1, "he never tried");
    }

    /// Fielder CF at (0, 80) and a fly or roller seen at time `t`. His belief is his own read
    /// of it (`Sim::predict`: the world's ball with his perception noise), not the world's
    /// path itself, so fixtures built on this leave wide margins.
    fn believing_the_truth(r: Vec3, v: Vec3, t: f64) -> Sim<'static> {
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&[])));
        let mut inp = input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.fattrs = core::array::from_fn(FielderAttrs::average);
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(inp));
        let st: &'static mut RulesState = Box::leak(Box::new(fx.st.clone()));
        let mut sim = new_sim(inp, st);
        sim.f[CF].pos = Vec3::new(0.0, 80.0, 0.0);
        loose_ball(&mut sim, t, r, v);
        sim.f[CF].role = Role::Chase;
        sim.f[CF].belief = Some(sim.predict(CF));
        sim
    }

    // agent-models §5.3: "A ball still in the air: only a catch is an out, so [going for it
    // gains him something] unless he can catch it on his feet before it comes down." By
    // hand, ignoring drag (a fly of 10 m/s up from 2 m comes down at 2.2 s):
    //   - from 3 m behind him, 3 m of running in 2.2 s is easy: standing play, no gain;
    //   - from 40 m out, 40 m in 2.2 s is beyond a man of ~8 m/s: no standing play, gain.
    #[test]
    fn going_for_a_fly_gains_only_when_he_cannot_catch_it_on_his_feet() {
        let near = believing_the_truth(Vec3::new(0.0, 83.0, 2.0), Vec3::new(0.0, 0.0, 10.0), 2.0);
        assert!(near.ball_still_up());
        assert!(!near.dive_gains(CF, DIVE_GETUP_S), "he can wait under it");
        assert!(!near.dive_gains(CF, STRETCH_COST_S));
        // 15 m out: he gets there on his feet, but only after it has landed (15 m in 2.2 s is
        // 6.8 m/s from a standing start, more than he can average), so there is a standing
        // play and it is no catch: going for it gains. This is the airborne comparison itself;
        // the 40 m case below never reaches it (no standing play at all).
        let mid = believing_the_truth(Vec3::new(0.0, 95.0, 2.0), Vec3::new(0.0, 0.0, 10.0), 2.0);
        assert!(mid.ball_still_up());
        let read = mid.f[CF].belief.as_ref().unwrap();
        assert!(mid.intercept_scan(CF, read).0.is_some(), "fixture: he can reach it on his feet");
        assert!(mid.dive_gains(CF, DIVE_GETUP_S) && mid.dive_gains(CF, STRETCH_COST_S), "on his feet he is too late to catch it");
        let far = believing_the_truth(Vec3::new(0.0, 120.0, 2.0), Vec3::new(0.0, 0.0, 10.0), 2.0);
        assert!(far.ball_still_up());
        assert!(far.dive_gains(CF, DIVE_GETUP_S) && far.dive_gains(CF, STRETCH_COST_S), "he cannot get there on his feet");
    }

    // agent-models §5.3: "A ball on the ground: only if staying on his feet would get him
    // the ball later than the cost (the 1.5 s get-up for a dive, the 0.3 s for a stretch)."
    // A roller coming straight at him from about 10 m: he has it on his feet within about
    // a second (bracketed from the ball's own path in the test), before a dive's 1.5 s is
    // paid (no gain) but not within a stretch's 0.3 s (gain).
    #[test]
    fn going_for_a_roller_gains_only_when_the_cost_is_less_than_the_wait() {
        // Already rolling (first bounce long past), so the ground branch is the one tested.
        let mut sim = believing_the_truth(Vec3::new(0.0, 93.8, 0.3), Vec3::new(0.0, -10.0, 0.0), 1.0);
        sim.t = 2.0;
        let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
        // The wait on his feet is bracketed from the ball's own path: no sooner than the
        // distance to his planning reach (0.7 of 1.0 m) over the ball's speed now plus his
        // top speed of 8.5 m/s (he runs flat out at it), no later than the ball takes to
        // reach that point with him standing still.
        // The same bracket must hold for his read of it, which is what he judges by (it
        // differs from the world's path by his perception noise).
        for (name, p) in [("world", path), ("his read", sim.f[CF].belief.as_ref().unwrap())] {
            let (d, v0) = (p.at(2.0).0.y - 80.0 - 0.7, p.at(2.0).1.norm());
            let t_up = (0..400).map(|k| 2.0 + 0.005 * k as f64).find(|&t| p.at(t).0.y - 80.0 <= 0.7).expect("it reaches him") - 2.0;
            let t_low = d / (v0 + 8.5);
            assert!(t_low > STRETCH_COST_S && t_up < DIVE_GETUP_S, "fixture ({name}): wait between {t_low:.2} and {t_up:.2} s");
        }
        assert!(!sim.ball_still_up(), "fixture: on the ground");
        assert!(!sim.dive_gains(CF, DIVE_GETUP_S), "a roller he can run down is not worth a dive");
        assert!(sim.dive_gains(CF, STRETCH_COST_S), "he would not have it within a stretch's 0.3 s");
    }

    /// The center fielder at (0, 80) with an exact read (his belief is the world's own path)
    /// of a fly passing him 1.3 m away at 20 m/s, 1.5 s in, when it is `z0` m up at 0.15 s
    /// before closest approach. Runs 0.8 s of world ticks.
    fn stretch_for_a_fly(z0: f64) -> Sim<'static> {
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&[])));
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }])));
        let st: &'static mut RulesState = Box::leak(Box::new(fx.st.clone()));
        let mut sim = new_sim(inp, st);
        sim.f[CF].pos = Vec3::new(0.0, 80.0, 0.0);
        loose_ball(&mut sim, 1.5, Vec3::new(-3.0, 78.7, z0), Vec3::new(20.0, 0.0, 0.0));
        let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
        let exact = path.clone();
        sim.f[CF].role = Role::Chase;
        sim.f[CF].belief = Some(exact);
        for _ in 0..48 {
            world_tick(&mut sim);
        }
        sim
    }

    // agent-models §5.3–§5.4 and rules-spec (visible out calls): "A caught fly is routine unless
    // the catch was diving or below z_low_catch = 0.4 m (margin 0)." A stretch is upright, so
    // it counts as low only by its height. By hand, the fly (20 m/s across, falling under
    // gravity from z0) passes the fielder at x = 0 after 3 m / 20 m/s = 0.15 s, having fallen
    // ½·9.80665·0.15² = 0.110 m (drag changes it by well under a centimetre): from 0.65 m it is
    // caught at about 0.54 m, from 0.45 m at about 0.34 m. Upright, above the line, the out is
    // routine (margin ∞); upright, below it, a shoestring catch (margin 0).
    #[test]
    fn an_upright_stretch_is_a_low_catch_only_below_the_low_catch_height() {
        for (z0, low) in [(0.65, false), (0.45, true)] {
            let sim = stretch_for_a_fly(z0);
            let t = secured_at(&sim).expect("he got it");
            let Ball::Held { .. } = sim.ball else { panic!("not held") };
            // The world's own height of the ball at the catch, from its path before it was held.
            let z_catch = z0 - 0.5 * 9.80665 * (t - 1.5) * (t - 1.5);
            assert!((z_catch < 0.4) == low && (z_catch - 0.4).abs() > 0.05, "fixture: caught at about {z_catch:.2} m");
            assert!(sim.out.dives.is_empty(), "an upright stretch, not a dive");
            let outs: Vec<_> = sim.out.calls.iter().collect();
            assert_eq!(outs.len(), 1, "one out, the batter's: {outs:?}");
            assert_eq!(outs[0].margin_s, if low { 0.0 } else { f64::INFINITY }, "caught at {z_catch:.2} m");
        }
    }

    /// A fly ball from home (`land` m of vacuum range at `th_deg` elevation, `az_deg` from
    /// center, negative toward third) read without error by everyone (`err` zero: a controlled
    /// belief, not `Sim::predict`'s noisy one) 0.5 s after contact, with a runner on `base`
    /// (0: none), and `place` setting where fielders stand. Runs the first call (`assign`).
    fn popup_after_the_first_call(land: f64, th_deg: f64, az_deg: f64, base: u8, place: &[(usize, Vec3)]) -> Sim<'static> {
        let (th, az) = (th_deg.to_radians(), az_deg.to_radians());
        let v = m::sqrt(land * 9.80665 / m::sin(2.0 * th));
        let on_base: Vec<(AgentId, u8)> = if base > 0 { vec![(0x0005, base)] } else { vec![] };
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&on_base)));
        let mut inp = input(fx, vec![]);
        inp.ball = BallState { r: Vec3::new(0.0, 0.5, 1.0), v: Vec3::new(v * m::cos(th) * m::sin(az), v * m::cos(th) * m::cos(az), v * m::sin(th)), w: Vec3::ZERO };
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(inp));
        let st: &'static mut RulesState = Box::leak(Box::new(fx.st.clone()));
        let mut sim = new_sim(inp, st);
        for fd in sim.f.iter_mut() {
            fd.err = [0.0; 4];
        }
        for &(i, at) in place {
            (sim.f[i].pos, sim.f[i].target) = (at, at);
        }
        sim.t = 0.4 + 0.5;
        sim.assign();
        sim
    }

    fn call_of(sim: &Sim, i: usize) -> (u8, f64) {
        let p = sim.f[i].belief.clone().unwrap();
        sim.call_value(i, &p, sim.f[i].t_int, sim.f[i].target, 0.0)
    }

    // Popup-calls review P2 #1, deferred by the owner 2026-11-13 ("mistakes like these happen in
    // real games"). A 42 m pop-up with a runner on second: the second baseman and the shortstop
    // each believe they catch it in the air with 0.3 s or more to spare, but not by enough to be
    // sure (tier 1, agent-models §5.7), so both go for it, and nobody covers second. The
    // believable behavior: when two infielders chase, someone still takes the bag, because a
    // runner is on second and the third baseman, first baseman, pitcher and catcher are free.
    // Fixture: err = 0, so both read the pop-up exactly; found by search for the tier-1 pair.
    #[test]
    #[ignore = "owner-deferred 2026-11-13 (popup-calls review P2 #1): both middle infielders chase and second base has no cover until the next re-call"]
    fn someone_covers_second_when_both_middle_infielders_chase_a_popup() {
        let sim = popup_after_the_first_call(42.0, 55.0, 0.0, 2, &[]);
        assert_eq!((call_of(&sim, 3).0, call_of(&sim, 5).0), (1, 1), "fixture: both are tier 1");
        assert!(sim.f[3].role == Role::Chase && sim.f[5].role == Role::Chase, "fixture: both chase: {:?}", sim.f.iter().map(|f| f.role).collect::<Vec<_>>());
        assert!(sim.f.iter().any(|f| f.role == Role::Cover(2)), "nobody covers second: {:?}", sim.f.iter().map(|f| f.role).collect::<Vec<_>>());
    }

    // Popup-calls review P2 #2, deferred by the owner 2026-11-13. Left field and center field
    // both go for a fly down the left-field line (tier 1: each believes he catches it, neither
    // is sure), left field as the chaser. A re-call asks only the chaser and the four fielders
    // nearest where he is going, so a chaser who is not among them is called off to back up
    // before anyone is sure. The believable behavior: he keeps going until someone calls it.
    // The re-call is invoked with the candidate list the rule would build when center field is
    // the fifth man (left field, short, third, second): the fixture's reads are exact (err = 0)
    // and the geometry that makes him the fifth nearest is the review's, not reproducible with
    // two outfielders both in the air ball's reach.
    #[test]
    #[ignore = "owner-deferred 2026-11-13 (popup-calls review P2 #2): a re-call sends an active chaser outside its candidate set to backup before anyone is sure"]
    fn a_chaser_outside_the_recall_keeps_going_until_someone_is_sure() {
        let mut sim = popup_after_the_first_call(95.0, 38.0, -35.0, 0, &[(7, Vec3::new(-40.0, 80.0, 0.0))]);
        assert_eq!((call_of(&sim, 6).0, call_of(&sim, 7).0), (1, 1), "fixture: both outfielders are tier 1");
        assert!(sim.f[6].role == Role::Chase && sim.f[7].role == Role::Chase, "fixture: left field and center field both chase");
        sim.replan(6);
        sim.assign_among(vec![6, 5, 4, 3]);
        assert!(sim.f[7].role == Role::Chase, "called off with nobody sure: {:?}", sim.f[7].role);
    }

    /// Foul pops that come down in the first-base seats, from just past the stands wall to
    /// 10 m in (Gemini play-area review P2-3): nobody catches one beyond his reach over the
    /// wall, and a fielder who goes after one sees the wall and stops at it. Arriving at the
    /// wall he is no faster than the wall-contact speed v_c = 2 m/s plus what braking allows
    /// over the last 0.02 m (agent-models §5.3: √(v_c² + 2·4.75·0.02) = 2.05 m/s), plus one
    /// 0.1 s track sample's braking (4.75 × 0.1 = 0.48 m/s).
    #[test]
    fn nobody_runs_into_the_stands_after_a_ball_out_of_reach() {
        let fx = fixture(&[]);
        let mut worst = (0.0f64, 0.0f64);
        for k in 0..8 {
            let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
            // Up 35 m and out toward the first-base seats, a little farther each time.
            let dir = Vec3::new(0.80, 0.25, 0.0).normalized();
            inp.ball = BallState { r: Vec3::new(0.0, 0.5, 1.0), v: dir * (11.0 + 0.5 * k as f64) + Vec3::new(0.0, 0.0, 26.0), w: Vec3::ZERO };
            let path = batted_path(&fx.env, &fx.field, inp.ball, inp.t_contact, |_, _| (0.0, 0.0));
            let (land, _) = path.at(path.t_end());
            let beyond = -fx.field.body_clearance(land.x, land.y, 0.0).dist;
            inp.record = true;
            let mut st = fx.st.clone();
            let out = simulate_live(&inp, &mut st);
            if beyond > REACH {
                assert!(!out.events.iter().any(|e| matches!(e, Event::Catch { .. })), "a catch {beyond:.1} m into the seats");
            }
            for (id, track) in &out.actors {
                if id & 0x80 == 0 {
                    continue;
                }
                for w in track.windows(2) {
                    let (a, b) = (w[0].1, w[1].1);
                    let wall = fx.field.body_clearance(b.x, b.y, BODY_R);
                    if wall.dist < 0.02 {
                        let into = (b - a).dot(-wall.inward) / (w[1].0 - w[0].0);
                        if into > worst.0 {
                            worst = (into, beyond);
                        }
                    }
                }
            }
        }
        assert!(worst.0 <= 2.05 + 0.475, "a fielder reached the wall at {:.2} m/s (ball {:.1} m into the seats)", worst.0, worst.1);
    }


    /// One limit over the wall for plans and every kind of capture (the collision review's
    /// P2-2; agent-models §5.3, physics-spec §9.5): the glove gets at most
    /// `REACH − BODY_R` = 1.0 − 0.3 = 0.7 m past the wall's plane, standing, stretching or
    /// diving, from a body that stops 0.3 m inside it. A ball coming down along the stands
    /// wall 0.68 m past its plane is caught, one 0.72 m past is not, whatever the glove's
    /// extension. Only the fast ball (over 3 m/s: a stretch) exercises the guard: a catch on his
    /// feet is limited to 0.7 m by geometry alone (1.0 m reach from a body 0.3 m inside the wall),
    /// so a slow ball proves nothing. The same guard sits in front of the dive, which reaches only
    /// 1.2 m up, and a ball that low past the 1.2 m stands wall is dead (no dive case).
    /// Built from a constructed state, not a seeded game. Bug caught: capture allowed the
    /// stretch's 1.4 m and the dive's 2.5 m from the body past the wall (a foul fly caught
    /// 0.93 m beyond it).
    #[test]
    fn nobody_catches_a_ball_farther_past_a_wall_than_his_arm_reaches_over_it() {
        assert!((OVER_WALL_REACH_M - 0.7).abs() < 1e-12, "the spec's 0.7 m");
        let fx = fixture(&[]);
        let field = &fx.field;
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let r2 = core::f64::consts::SQRT_2;
        // First-base stands wall, 27 m along the line (past the dugout's 6–24 m, before the taper's 30.5 m): the plane is where
        // c = (x − y)/√2 is the foul clearance, the way in is (−1, 1)/√2.
        let at = |along: f64, past: f64| {
            let c = field.foul_clear_m + past;
            Vec3::new((along + c) / r2, (along - c) / r2, 0.0)
        };
        let inward = Vec3::new(-1.0, 1.0, 0.0) * (1.0 / r2);
        assert!(field.body_clearance(at(27.0, 0.0).x, at(27.0, 0.0).y, 0.0).dist.abs() < 1e-9, "fixture: the plane");
        for (name, speed, z) in [("stretch", 4.0, 1.8)] {
            for (past, catches) in [(0.68, true), (0.72, false)] {
                let mut st = fx.st.clone();
                let mut sim = new_sim(&inp, &mut st);
                // His body stops 0.3 m inside the plane; the ball comes down the wall past him,
                // abreast of him after 0.2 s.
                let fielder = at(27.0, 0.0) + inward * BODY_R;
                sim.f[CF].pos = fielder;
                let ball0 = at(27.0 - 0.2 * speed, past) + Vec3::new(0.0, 0.0, z);
                let v = Vec3::new(1.0, 1.0, 0.0) * (speed / r2);
                loose_ball(&mut sim, 2.0, ball0, v);
                sim.f[CF].pos = fielder;
                sim.f[CF].role = Role::Chase;
                sim.f[CF].must_dive_since = 0.0;
                sim.f[CF].last_dive_read = f64::INFINITY;
                sim.f[CF].will_dive.set(true);
                for _ in 0..60 {
                    world_tick(&mut sim);
                }
                assert_eq!(caught(&sim), catches, "{name}: ball {past} m past the wall (limit {OVER_WALL_REACH_M} m); events {:?}", sim.out.events);
            }
        }
    }

    /// Wall-contact speed (agent-models §5.3): a fielder running flat out at a wall for a ball
    /// he can take there arrives at about `v_c` = 2 m/s. The wall stops him at 0.3 m,
    /// the body's radius.
    #[test]
    fn a_fielder_running_at_the_wall_arrives_at_the_contact_speed() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let field = &fx.field;
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        let v = sim.f[CF].a.speed;
        let (mut lo, mut hi) = (60.0, 200.0);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if field.body_clearance(0.0, mid, BODY_R).dist > 0.0 { lo = mid } else { hi = mid }
        }
        let y_wall = 0.5 * (lo + hi);
        sim.f[CF].pos = Vec3::new(0.0, y_wall - 40.0, 0.0);
        sim.f[CF].vel = Vec3::new(0.0, v, 0.0);
        sim.f[CF].target = Vec3::new(0.0, y_wall + 5.0, 0.0);
        sim.f[CF].role = Role::Chase;
        sim.f[CF].t_int = 0.0;
        let mut at_wall = None;
        let mut slowest_before = f64::INFINITY;
        for _ in 0..1000 {
            let before = sim.f[CF].pos;
            sim.move_fielders();
            sim.t += DT;
            let c = field.body_clearance(sim.f[CF].pos.x, sim.f[CF].pos.y, BODY_R).dist;
            if c < 0.02 && at_wall.is_none() {
                // His speed into the wall over the tick that took him to it.
                at_wall = Some((sim.f[CF].pos.y - before.y) / DT);
            }
            if at_wall.is_none() {
                slowest_before = slowest_before.min(sim.f[CF].vel.y);
            }
            if at_wall.is_some() {
                break;
            }
        }
        let arrival = at_wall.expect("he reached the wall");
        // √(v_c² + 2·4.75·0.02) = 2.05 m/s within 0.02 m of it, up to one tick of braking
        // (4.75/60 = 0.08 m/s) and the tick's position step.
        assert!((arrival - 2.05).abs() < 0.3, "arrived at {arrival:.2} m/s, expected about v_c = 2 m/s");
        assert!(slowest_before > 1.5, "he never stopped short of it: {slowest_before:.2} m/s");
    }

    /// Gemini A4 review P1-1: with two out every out ends the inning, and p_out saturates to
    /// 1.0 for any margin over a second, so a sure force a step away and a sure throw across
    /// the diamond tie exactly. A fielder a step from the bag takes the force with his foot
    /// rather than throw 27 m to first (the throw is the one that can go wrong).
    #[test]
    fn with_two_out_a_sure_force_a_step_away_beats_a_sure_throw_across() {
        const SS: usize = 5;
        let fx = fixture(&[(5, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        st.outs = 2;
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        held_at(&mut sim, SS, base_pos(2) + Vec3::new(-1.0, 0.0, 0.0));
        sim.f[2].pos = base_pos(1);
        let fd = &mut sim.f[SS];
        (fd.up_at, fd.set_base, fd.face, fd.catch_vel) = (1.0, fd.a.transfer, None, Vec3::ZERO);
        // The batter 2.6 s from first, the runner from first 2.0 s from second: both outs sure.
        (sim.r[0].s, sim.r[0].v, sim.r[0].goal) = (BASE_L - 8.2 * 2.6, 8.2, 1);
        (sim.r[1].s, sim.r[1].v, sim.r[1].goal) = (2.0 * BASE_L - 8.2 * 2.0, 8.2, 2);
        let opts = sim.holder_options(SS);
        let first = opts.iter().find(|o| !o.carry && o.base == 1).expect("a throw to first");
        let step = opts.iter().find(|o| o.carry && o.base == 2).expect("a carry to second");
        assert!(first.p_out > 0.999 && step.p_out > 0.999, "fixture: both sure ({} and {})", first.p_out, step.p_out);
        sim.holder_act();
        assert!(matches!(sim.ball, Ball::Held { by: SS }) && sim.f[SS].carry_to == Some(2), "he steps on second: {:?} carry {:?}", sim.ball, sim.f[SS].carry_to);
    }

    /// Gemini A4 review P2-3: the cutoff lines up for the base the lead runner threatens now.
    /// A runner who started on first and stands on third, with the ball held in left field
    /// 70 m out, threatens home, so the cutoff lines up for home.
    #[test]
    fn the_cutoff_lines_up_for_home_with_a_runner_standing_on_third() {
        const LF: usize = 6;
        let fx = fixture(&[(5, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 5.0;
        (sim.r[0].s, sim.r[0].v, sim.r[0].goal) = (BASE_L, 0.0, 1);
        (sim.r[1].s, sim.r[1].v, sim.r[1].goal) = (3.0 * BASE_L, 0.0, 3);
        let from = Vec3::new(-35.0, 60.0, 0.0);
        held_at(&mut sim, LF, from);
        let job = sim.relay_job(from);
        assert_eq!(job.map(|j| j.1), Some(4), "cutoff {job:?}");
    }

    /// Gemini A4 review P1-2 (4): the double-play continuation counts a second out only if
    /// its throw would be attempted, the holder's own p > 0.3 rule. One out, a runner from
    /// first 2 s from second, the shortstop a step from the bag, the batter 0.6 s from
    /// first: no relay can beat him, so stepping on second is worth its own out, p² × 1.0.
    #[test]
    fn a_double_play_nobody_would_throw_for_adds_nothing() {
        const SS: usize = 5;
        let fx = fixture(&[(5, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        st.outs = 1;
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        held_at(&mut sim, SS, base_pos(2) + Vec3::new(-1.0, 0.0, 0.0));
        sim.f[2].pos = base_pos(1);
        let fd = &mut sim.f[SS];
        (fd.up_at, fd.set_base, fd.face, fd.catch_vel) = (1.0, fd.a.transfer, None, Vec3::ZERO);
        (sim.r[0].s, sim.r[0].v, sim.r[0].goal) = (BASE_L - 8.2 * 0.6, 8.2, 1);
        (sim.r[1].s, sim.r[1].v, sim.r[1].goal) = (2.0 * BASE_L - 8.2 * 2.0, 8.2, 2);
        let opts = sim.holder_options(SS);
        let step = opts.iter().find(|o| o.carry && o.base == 2).expect("a carry to second");
        let p2 = step.continuation.expect("a continuation is estimated").0;
        assert!(p2 < 0.3, "fixture: the relay cannot beat him ({p2})");
        assert!((step.value - step.p_out * step.p_out * step.weight).abs() < 1e-12, "value {} with p2 {p2}", step.value);
    }

    /// Gemini A4 review P3-6: the comparison is p² × w (agent-models §5.6), not p × w. Bases
    /// loaded, one out, the first baseman 1 m from his bag with the batter 1.0 s away: the
    /// carry to first is sure, 1.0² × 0.9 = 0.9. The runners from first and second are 0.1 s
    /// from their bases (no play). The runner from third is placed so the force at home is
    /// about 0.8 sure: 0.8² × 1.3 ≈ 0.83 < 0.9, so he takes first. Linear weighing would
    /// give 0.8 × 1.3 = 1.04 > 0.9 and throw home.
    #[test]
    fn the_holder_weighs_p_squared_times_w() {
        const B1: usize = 2;
        let fx = fixture(&[(5, 1), (6, 2), (7, 3)]);
        let runners = vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 6, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 7, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }];
        let inp = input(&fx, runners);
        let mut st = fx.st.clone();
        st.outs = 1;
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        held_at(&mut sim, B1, base_pos(1) + Vec3::new(-0.7, 0.7, 0.0));
        sim.f[1].pos = base_pos(4);
        let fd = &mut sim.f[B1];
        (fd.up_at, fd.set_base, fd.face, fd.catch_vel) = (1.0, fd.a.transfer, None, Vec3::ZERO);
        (sim.r[0].s, sim.r[0].v, sim.r[0].goal) = (BASE_L - 8.2, 8.2, 1);
        for k in 1..3 {
            let r = &mut sim.r[k];
            (r.s, r.v, r.goal) = ((r.start + 1) as f64 * BASE_L - 0.82, 8.2, r.start + 1);
        }
        // Put the runner from third where the force at home has a 0.1 s margin (p ≈ 0.80).
        (sim.r[3].s, sim.r[3].v, sim.r[3].goal) = (3.0 * BASE_L + 10.0, 8.2, 4);
        for _ in 0..6 {
            let home = *sim.holder_options(B1).iter().find(|o| !o.carry && o.base == 4).expect("a throw home");
            sim.r[3].s -= 8.2 * (0.101 - (home.runner_at - home.ball_at));
        }
        let opts = sim.holder_options(B1);
        let home = opts.iter().find(|o| !o.carry && o.base == 4).unwrap();
        let first = opts.iter().find(|o| o.carry && o.base == 1).expect("a carry to first");
        assert!((0.75..0.85).contains(&home.p_out) && first.p_out > 0.999, "fixture: home {} first {}", home.p_out, first.p_out);
        assert!(home.p_out * home.weight > first.p_out * first.weight, "fixture: linear weighing would go home");
        sim.holder_act();
        assert_eq!(sim.f[B1].carry_to, Some(1), "he takes the sure out at first: {:?}", sim.ball);
    }

    /// Gemini A4 review P3-6: no attempt below p = 0.3 (agent-models §5.6). The shortstop of
    /// `shortstop_to_first` with the batter where the throw has a −0.08 s margin (p ≈ 0.25)
    /// holds the ball; at −0.03 s (p ≈ 0.40) he throws.
    #[test]
    fn no_throw_is_attempted_below_p_of_0_3() {
        const SS: usize = 5;
        for (margin, attempt) in [(-0.08, false), (-0.03, true)] {
            let fx = fixture(&[]);
            let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
            let mut st = fx.st.clone();
            let mut sim = new_sim(&inp, &mut st);
            sim.t = 1.0;
            held_at(&mut sim, SS, Vec3::new(-8.0, 38.0, 0.0));
            sim.f[2].pos = base_pos(1);
            let fd = &mut sim.f[SS];
            (fd.up_at, fd.set_base, fd.face, fd.catch_vel) = (1.0, fd.a.transfer, None, Vec3::ZERO);
            (sim.r[0].s, sim.r[0].v, sim.r[0].goal) = (BASE_L - 8.2 * eta_for_margin(margin, false), 8.2, 1);
            let p = sim.holder_options(SS).iter().find(|o| !o.carry && o.base == 1).unwrap().p_out;
            assert!((p > 0.3) == attempt, "fixture: p {p} at margin {margin}");
            let mut thrown = false;
            for _ in 0..60 {
                sim.holder_act();
                if matches!(sim.ball, Ball::Thrown { .. }) {
                    thrown = true;
                    break;
                }
                sim.r[0].s += 8.2 * DT;
                sim.t += DT;
            }
            assert_eq!(thrown, attempt, "margin {margin}, p {p}");
        }
    }

    /// A feed from the second baseman's side (12, 40) to the shortstop at `at`, standing,
    /// with a runner forced from first and the batter `batter_eta_s` from first. Steps the
    /// world until the shortstop has let go of the ball for first; returns the sim, when he
    /// caught it, when he reached the bag with it, and when he let go.
    fn feed_to_shortstop(at: Vec3, batter_eta_s: f64) -> (Sim<'static>, f64, f64, f64) {
        const SS: usize = 5;
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&[(5, 1)])));
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }])));
        let st: &'static mut RulesState = Box::leak(Box::new(fx.st.clone()));
        let mut sim = new_sim(inp, st);
        sim.t = 2.0;
        for fd in sim.f.iter_mut() {
            (fd.vel, fd.role) = (Vec3::ZERO, Role::Idle);
            fd.target = fd.pos;
        }
        (sim.f[SS].pos, sim.f[SS].target) = (at, at);
        (sim.f[2].pos, sim.f[2].target) = (base_pos(1), base_pos(1));
        let (b, r1) = (sim.r.iter().position(|r| r.start == 0).unwrap(), sim.r.iter().position(|r| r.start == 1).unwrap());
        (sim.r[b].s, sim.r[b].v, sim.r[b].goal, sim.r[b].direction) = (BASE_L - 8.2 * batter_eta_s, 8.2, 1, 1.0);
        (sim.r[r1].s, sim.r[r1].v, sim.r[r1].goal, sim.r[r1].direction) = (1.5 * BASE_L, 8.2, 2, 1.0);
        // The second baseman fielded a ground ball at (12, 40) and throws to second.
        let fielder_2b = sim.f[3].id;
        sim.emit(Input::BallTouched { t: 1.2, what: Touch::Ground { x: 10.0, y: 36.0 } });
        sim.emit(Input::GloveSecured { t: 1.7, fielder: fielder_2b, x: 12.0, y: 40.0 });
        sim.emit(Input::BallReleasedFromGlove { t: 2.0, fielder: fielder_2b, kind: ReleaseKind::Throw, infielder: true });
        let from = Vec3::new(12.0, 40.0, 1.5);
        let to = Vec3::new(at.x, at.y, 1.2);
        let h = Vec3::new(to.x - from.x, to.y - from.y, 0.0);
        let tf = h.norm() / 25.0;
        let v = h.normalized() * 25.0 + Vec3::new(0.0, 0.0, (to.z - from.z + 0.5 * 9.81 * tf * tf) / tf);
        sim.launch_throw(BallState { r: from, v, w: Vec3::ZERO }, to, 2, SS, false);
        let (mut caught, mut on_bag) = (f64::NAN, f64::NAN);
        for _ in 0..400 {
            sim.update_thrown();
            if caught.is_nan() && matches!(sim.ball, Ball::Held { by: SS }) {
                caught = sim.t;
            }
            if !caught.is_nan() {
                if on_bag.is_nan() && (base_pos(2) - sim.f[SS].pos).norm() < 0.6 {
                    on_bag = sim.t;
                }
                sim.holder_act();
                if matches!(sim.ball, Ball::Thrown { to_base: 1, .. }) {
                    let t = sim.t;
                    return (sim, caught, on_bag, t);
                }
            }
            let b = sim.r.iter().position(|r| r.start == 0).unwrap();
            sim.r[b].s = (sim.r[b].s + 8.2 * DT).min(BASE_L);
            world_tick(&mut sim);
        }
        panic!("no pivot throw: {:?}", sim.ball);
    }

    /// agent-models §5.5: a grade-50 pivot man on the bag lets go 0.80 s after the feed plus
    /// §5.5's turn. Standing, he faces the feed, which comes from the second baseman's side
    /// (12, 40): his glove side (a right-hander's left) is then about north and first base is
    /// southeast of the bag, φ ≈ 135° off it, so the turn is 0.12·(φ − 90°)/90° ≈ +0.06 s.
    /// With the batter 2.4 s from first at the feed (about 0.16 s in hand at the pivot man's
    /// set release, under the 0.25 s at which the general rule would hurry the throw) he still
    /// takes his full pivot: no early release.
    #[test]
    fn a_pivot_man_on_the_bag_lets_go_after_his_pivot_time() {
        let (sim, caught, _, released) = feed_to_shortstop(base_pos(2), 2.4);
        let feed = Vec3::new(12.0 - base_pos(2).x, 40.0 - base_pos(2).y, 0.0).normalized();
        let glove = Vec3::new(-feed.y, feed.x, 0.0);
        let to_first = (base_pos(1) - base_pos(2)).normalized();
        let phi = glove.dot(to_first).clamp(-1.0, 1.0).acos();
        let expected = 0.80 + 0.12 * (phi - core::f64::consts::FRAC_PI_2) / core::f64::consts::FRAC_PI_2;
        assert!((released - caught - expected).abs() <= 0.02 + DT, "pivot {:.3} s, expected {expected:.3}", released - caught);
        assert_eq!(sim.out.hurry.last().copied(), Some((0.0, false)), "released set, not hurried");
    }

    /// agent-models §5.5: he must have the bag. Taking the feed 2 m off it, his pivot starts
    /// when he reaches it: he lets go at least 0.68 s after he gets there (the 0.80 s pivot
    /// less the most a turn can save, 0.12 s), and more than 0.80 s after the catch.
    #[test]
    fn a_pivot_off_the_bag_starts_when_he_gets_it() {
        let (_, caught, on_bag, released) = feed_to_shortstop(base_pos(2) + Vec3::new(-2.0, 0.0, 0.0), 6.0);
        assert!(on_bag > caught, "he caught it off the bag ({caught:.3}) and got there at {on_bag:.3}");
        assert!(released - on_bag >= 0.80 - 0.12 - 1e-9 && released - caught > 0.80 + DT, "caught {caught:.3}, on the bag {on_bag:.3}, released {released:.3}");
    }

    /// Gemini pivot review P2-2, agent-models §5.6: a shortstop who carries the ball to second
    /// himself (6U-3) caught nothing there, and "a carrier retains setup time already spent
    /// holding the ball". He fielded it a second before he touches the bag, longer than his
    /// 0.65 s transfer, so on the touch he has nothing left to wait for but his turn: he lets
    /// go within 0.4 s of the touch (a turn at most 0.12 s less or more than the quarter turn,
    /// and the 0.05 s of a release tick), not after a fresh 0.80 s pivot. Bug caught: the touch
    /// restarting his clock with the full catch-to-release pivot (0.68 s or more).
    #[test]
    fn a_carrier_to_second_keeps_the_set_up_he_spent_running_with_the_ball() {
        const SS: usize = 5;
        let fx = fixture(&[(5, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        let ss_id = sim.f[SS].id;
        sim.emit(Input::BallTouched { t: 0.6, what: Touch::Ground { x: -5.0, y: 33.0 } });
        sim.emit(Input::GloveSecured { t: 1.0, fielder: ss_id, x: -4.5, y: 36.8 });
        for fd in sim.f.iter_mut() {
            (fd.vel, fd.role) = (Vec3::ZERO, Role::Idle);
            fd.target = fd.pos;
        }
        // (Fielded 4.9 m out: from 5.7 m, the slower braking made throwing to first the better play.)
        held_at(&mut sim, SS, Vec3::new(-4.5, 36.8, 0.0));
        let fd = &mut sim.f[SS];
        (fd.up_at, fd.set_base, fd.face, fd.catch_vel) = (1.0, fd.a.transfer, None, Vec3::ZERO);
        fd.carry_to = Some(2);
        fd.target = base_pos(2);
        let (b, r1) = (sim.r.iter().position(|r| r.start == 0).unwrap(), sim.r.iter().position(|r| r.start == 1).unwrap());
        (sim.r[b].s, sim.r[b].v, sim.r[b].goal, sim.r[b].direction) = (BASE_L - 8.2 * 2.6, 8.2, 1, 1.0);
        (sim.r[r1].s, sim.r[r1].v, sim.r[r1].goal, sim.r[r1].direction) = (1.5 * BASE_L, 8.2, 2, 1.0);
        let mut on_bag = f64::NAN;
        for _ in 0..400 {
            if on_bag.is_nan() && (base_pos(2) - sim.f[SS].pos).norm() < 0.6 {
                on_bag = sim.t;
            }
            sim.holder_act();
            if matches!(sim.ball, Ball::Thrown { .. }) {
                assert!(!on_bag.is_nan(), "threw before the bag");
                assert!(sim.t - on_bag < 0.4, "touched at {on_bag:.3}, let go at {:.3}: a fresh pivot (pivot {} carry {:?} up_at {:.2} set_base {:.2})", sim.t, sim.f[SS].pivot, sim.f[SS].carry_to, sim.f[SS].up_at, sim.f[SS].set_base);
                return;
            }
            for k in [b, r1] {
                let goal = sim.r[k].goal as f64 * BASE_L;
                sim.r[k].s = (sim.r[k].s + 8.2 * DT).min(goal);
            }
            world_tick(&mut sim);
        }
        panic!("never threw: {:?}", sim.ball);
    }

    /// A feed receiver at second, `off_m` off the bag, holding the ball: the state `try_captures` gives
    /// him (pivot, waiting for the bag, facing the feed, heading for the bag).
    fn off_bag_receiver(batter_eta_s: f64, r1_short_m: f64, off_m: f64) -> (Sim<'static>, usize, usize) {
        const SS: usize = 5;
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&[(5, 1)])));
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }])));
        let st: &'static mut RulesState = Box::leak(Box::new(fx.st.clone()));
        let mut sim = new_sim(inp, st);
        sim.t = 2.0;
        let at = base_pos(2) + Vec3::new(-off_m, 0.0, 0.0);
        let ss_id = sim.f[SS].id;
        sim.emit(Input::BallTouched { t: 1.2, what: Touch::Ground { x: 10.0, y: 36.0 } });
        sim.emit(Input::GloveSecured { t: 2.0, fielder: ss_id, x: at.x, y: at.y });
        for fd in sim.f.iter_mut() {
            (fd.vel, fd.role) = (Vec3::ZERO, Role::Idle);
            fd.target = fd.pos;
        }
        (sim.f[2].pos, sim.f[2].target) = (base_pos(1), base_pos(1));
        held_at(&mut sim, SS, at);
        let fd = &mut sim.f[SS];
        fd.up_at = 2.0;
        fd.ready_at = 2.0;
        fd.catch_vel = Vec3::ZERO;
        fd.pivot = true;
        fd.pivot_wait_bag = true;
        fd.set_base = throwing::pivot_s(fd.a.transfer);
        fd.face = Some(Vec3::new(-1.0, 1.0, 0.0).normalized());
        fd.target = base_pos(2);
        let (b, r1) = (sim.r.iter().position(|r| r.start == 0).unwrap(), sim.r.iter().position(|r| r.start == 1).unwrap());
        (sim.r[b].s, sim.r[b].v, sim.r[b].goal, sim.r[b].direction) = (BASE_L - 8.2 * batter_eta_s, 8.2, 1, 1.0);
        (sim.r[r1].s, sim.r[r1].v, sim.r[r1].goal, sim.r[r1].direction) = (2.0 * BASE_L - r1_short_m, 8.2, 2, 1.0);
        (sim, b, r1)
    }

    /// Gemini pivot review P2-1 (a), agent-models §5.5: a feed receiver 4 m off the bag with the
    /// runner from first 5 m short of it (0.6 s away, the receiver 0.75 s from the bag) cannot
    /// get the force, and the batter is 1.9 s from first. He throws to first now, as any
    /// receiver would, instead of waiting for a bag he will not beat the runner to. Bug caught:
    /// the throw held back until the runner passes second and the force is gone.
    #[test]
    fn a_receiver_who_cannot_beat_the_runner_to_the_bag_throws_instead_of_waiting_for_it() {
        const SS: usize = 5;
        let (mut sim, b, r1) = off_bag_receiver(1.9, 5.0, 4.0);
        for _ in 0..200 {
            sim.holder_act();
            if let Ball::Thrown { to_base, .. } = sim.ball {
                assert_eq!(to_base, 1);
                // An ordinary receiver lets go 0.7 × transfer after his catch at 2.0 s (no turn,
                // no footwork: nothing owed); waiting for the bag would be 0.75 s later.
                let ordinary = RELAY_TRANSFER_FRAC * sim.f[SS].a.transfer;
                assert!(sim.t - 2.0 <= ordinary + 2.0 * DT, "let go {:.3} s after the catch, ordinary {ordinary:.3}", sim.t - 2.0);
                assert!(!sim.f[SS].pivot && !sim.f[SS].pivot_wait_bag);
                return;
            }
            for k in [b, r1] {
                let goal = sim.r[k].goal as f64 * BASE_L;
                sim.r[k].s = (sim.r[k].s + 8.2 * DT).min(goal);
            }
            world_tick(&mut sim);
        }
        panic!("never threw: {:?}", sim.ball);
    }

    /// Gemini pivot review P2-1 (c), agent-models §5.5: when the force at second is gone before
    /// the off-bag receiver has the bag he is an ordinary receiver: no pivot, the ordinary
    /// 0.7 × transfer, and not facing the feed or heading for the bag.
    #[test]
    fn a_receiver_whose_force_is_gone_is_an_ordinary_receiver() {
        const SS: usize = 5;
        let (mut sim, _, r1) = off_bag_receiver(6.0, 5.0, 4.0);
        sim.r[r1].s = 2.0 * BASE_L + 0.5; // safe at second
        sim.holder_act();
        let fd = &sim.f[SS];
        assert!(!fd.pivot && !fd.pivot_wait_bag);
        assert!((fd.set_base - RELAY_TRANSFER_FRAC * fd.a.transfer).abs() < 1e-12, "set_base {}", fd.set_base);
        assert!(fd.face.is_none(), "still facing the feed: {:?}", fd.face);
        assert!((fd.target - base_pos(2)).norm() > 0.5, "still heading for the bag: {:?}", fd.target);
    }

    /// Gemini pivot review P2-1 (b), agent-models §5.5: only a throw to first after the force at
    /// second is the pivot ("a force at second followed by a throw to first"). A pivot man who
    /// throws to third or home is an ordinary thrower: 0.7 × transfer, not the 0.80 s pivot.
    #[test]
    fn only_a_throw_to_first_is_a_pivot() {
        const B2: usize = 3;
        let fx = fixture(&[(5, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.f[B2].pos = base_pos(2);
        sim.f[B2].catch_vel = Vec3::ZERO;
        sim.f[B2].face = Some(Vec3::new(0.0, -1.0, 0.0)); // facing the feed, away from both targets' glove sides
        sim.f[B2].pivot = true;
        sim.f[B2].set_base = throwing::pivot_s(sim.f[B2].a.transfer);
        let tr = sim.f[B2].a.transfer;
        let first = sim.set_up_for(B2, base_pos(1)).set_s;
        let third = sim.set_up_for(B2, base_pos(3)).set_s;
        let home = sim.set_up_for(B2, base_pos(4)).set_s;
        assert!(first >= throwing::pivot_s(tr) - 0.12 - 1e-9, "to first {first:.3}");
        for (name, s) in [("third", third), ("home", home)] {
            assert!(s <= RELAY_TRANSFER_FRAC * tr + 0.12 + 0.2 + 1e-9, "to {name} {s:.3} s: a pivot clock");
        }
    }

    /// Gemini pivot review P2-3 and P3-1, agent-models §5.5: a covering man brakes onto the bag
    /// (√(2·BRAKE·0.6 m) = 2.39 m/s at the touch radius, the movement law) and "faces the way
    /// he is moving"; his teammates' estimate (`pivot_release(.., arriving)`) says the same. The
    /// world used the running threshold (3 m/s), so the man who was visibly moving at 2.39
    /// faced the feed instead, and the estimate and the world differed by the turn. Checked for
    /// a second baseman coming across from the left (−1, −0.3), the case the review measured:
    /// he faces his approach, and the pivot he is credited with is the one predicted for him.
    /// Bug caught: the facing threshold above the braking speed at the bag.
    #[test]
    fn a_pivot_man_braking_onto_the_bag_faces_the_way_he_is_moving() {
        const B2: usize = 3;
        let fx = fixture(&[(5, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        let (b, r1) = (sim.r.iter().position(|r| r.start == 0).unwrap(), sim.r.iter().position(|r| r.start == 1).unwrap());
        (sim.r[b].s, sim.r[b].v, sim.r[b].goal, sim.r[b].direction) = (BASE_L - 8.2 * 3.0, 8.2, 1, 1.0);
        (sim.r[r1].s, sim.r[r1].v, sim.r[r1].goal, sim.r[r1].direction) = (1.5 * BASE_L, 8.2, 2, 1.0);
        let approach = Vec3::new(1.0, 0.3, 0.0).normalized();
        let speed = m::sqrt(2.0 * BRAKE * 0.6);
        held_at(&mut sim, B2, base_pos(2) - approach * 0.59);
        let feed_from = Vec3::new(-7.0, 35.0, 0.0);
        let predicted = sim.pivot_release(B2, feed_from, 0.0, true);
        let fd = &mut sim.f[B2];
        fd.vel = approach * speed;
        fd.pivot = true;
        fd.pivot_wait_bag = true;
        fd.face = Some((feed_from - base_pos(2)).normalized()); // as the catch left it: the feed
        fd.up_at = 2.0;
        sim.holder_act();
        let face = sim.f[B2].face.expect("facing");
        assert!(face.dot(approach) > 0.99, "faces {face:?}, not his approach {approach:?}");
        let world = sim.set_up_for(B2, base_pos(1)).set_s;
        // (The world turns toward first from where he stands, 0.59 m off the bag.)
        assert!((world - predicted).abs() < 0.005, "the world's pivot {world:.4} s, the estimate {predicted:.4} s");
    }

    /// agent-models §5.5: the holder's estimate of a double play uses the pivot model the
    /// world uses. The shortstop fields a ground ball at (−7, 35) with a runner forced from
    /// first and the second baseman standing on the bag. As he lets the feed go, the pivot he
    /// predicts (from the second baseman having the ball to letting it go) is the one the
    /// world then makes, to a tick; and the relay reaches first within 0.15 s of his estimate
    /// (each estimated arrival carries the 0.05 s handling allowance of every throw estimate,
    /// two here, and the flight model's difference).
    #[test]
    fn the_holder_predicts_the_pivot_the_world_makes() {
        const SS: usize = 5;
        const B2: usize = 3;
        const B1: usize = 2;
        let fx = fixture(&[(5, 1)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }, RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 1.5;
        let ss_id = sim.f[SS].id;
        sim.emit(Input::BallTouched { t: 0.9, what: Touch::Ground { x: -5.0, y: 30.0 } });
        sim.emit(Input::GloveSecured { t: 1.5, fielder: ss_id, x: -7.0, y: 35.0 });
        held_at(&mut sim, SS, Vec3::new(-7.0, 35.0, 0.0));
        for fd in sim.f.iter_mut() {
            (fd.vel, fd.role) = (Vec3::ZERO, Role::Idle);
            fd.target = fd.pos;
        }
        (sim.f[B2].pos, sim.f[B2].target) = (base_pos(2), base_pos(2));
        (sim.f[B1].pos, sim.f[B1].target) = (base_pos(1), base_pos(1));
        let fd = &mut sim.f[SS];
        (fd.up_at, fd.set_base, fd.face, fd.catch_vel) = (1.5, fd.a.transfer, None, Vec3::ZERO);
        let (b, r1) = (sim.r.iter().position(|r| r.start == 0).unwrap(), sim.r.iter().position(|r| r.start == 1).unwrap());
        (sim.r[b].s, sim.r[b].v, sim.r[b].goal, sim.r[b].direction) = (5.0, 8.2, 1, 1.0);
        (sim.r[r1].s, sim.r[r1].v, sim.r[r1].goal, sim.r[r1].direction) = (BASE_L + 5.0, 8.2, 2, 1.0);
        let (mut predicted, mut caught, mut released) = (None, f64::NAN, f64::NAN);
        for _ in 0..600 {
            if matches!(sim.ball, Ball::Held { by: SS }) {
                let opts = sim.holder_options(SS);
                predicted = opts.iter().find(|o| !o.carry && o.base == 2).and_then(|o| {
                    let pivot = sim.pivot_release(B2, sim.f[SS].pos, o.ball_at, false) - o.ball_at;
                    o.continuation.map(|c| (pivot, c.1))
                });
            }
            let pivoting = matches!(sim.ball, Ball::Held { by: B2 });
            sim.update_thrown();
            sim.holder_act();
            if pivoting && released.is_nan() && matches!(sim.ball, Ball::Thrown { .. }) {
                released = sim.t;
            }
            if matches!(sim.ball, Ball::Held { by: B1 }) {
                let (pivot, at_first) = predicted.expect("a continuation was estimated");
                assert!((released - caught - pivot).abs() <= DT + 1e-6, "pivot {:.3} s, predicted {pivot:.3}", released - caught);
                assert!((sim.t - at_first).abs() <= 0.15, "first base has it at {:.3}, predicted {at_first:.3}", sim.t);
                return;
            }
            for k in [b, r1] {
                let goal = sim.r[k].goal as f64 * BASE_L;
                sim.r[k].s = (sim.r[k].s + 8.2 * DT).min(goal);
            }
            let before = matches!(sim.ball, Ball::Held { by: B2 });
            world_tick(&mut sim);
            if !before && matches!(sim.ball, Ball::Held { by: B2 }) {
                caught = sim.t - DT;
            }
        }
        panic!("no double play attempt: {:?}", sim.ball);
    }

    #[test]
    fn first_baseman_plays_standard_depth_when_second_is_occupied() {
        for bats in [bb_observe::Hand::Right, bb_observe::Hand::Left] {
            let fx_e = fixture(&[]);
            let mut st_e = fx_e.st.clone();
            let mut inp_e = input(&fx_e, vec![RunnerIn { id: 0x0001, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
            inp_e.bats = bats;
            let sim_empty = new_sim(&inp_e, &mut st_e);

            let mut fx13 = fixture(&[(5, 1), (7, 3)]);
            fx13.st.outs = 1;
            let mut st13 = fx13.st.clone();
            let mut inp13 = input(&fx13, vec![
                RunnerIn { id: fx13.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
                RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
                RunnerIn { id: 7, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
            ]);
            inp13.bats = bats;
            let sim_13 = new_sim(&inp13, &mut st13);
            assert!((sim_13.f[2].pos - base_pos(1)).norm() < 2.3, "{bats:?} 1st&3rd should hold runner at 1B");

            for (label, on_base) in [
                ("1st&2nd", vec![(5, 1), (6, 2)]),
                ("loaded", vec![(5, 1), (6, 2), (7, 3)]),
            ] {
                let mut fx = fixture(&on_base);
                fx.st.outs = 1;
                let mut st = fx.st.clone();
                let mut runners = vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }];
                runners.extend(on_base.iter().map(|&(id, base)| RunnerIn { id, base, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }));
                let mut inp = input(&fx, runners);
                inp.bats = bats;
                let sim = new_sim(&inp, &mut st);
                assert!((sim.f[2].pos - sim_empty.f[2].pos).norm() < 1e-9,
                    "{bats:?} {label}: 1B should stay at Standard depth {:?}, got {:?}", sim_empty.f[2].pos, sim.f[2].pos);
                assert!(sim.f[3].pos.norm() < sim_empty.f[3].pos.norm(), "{bats:?} {label}: 2B stays at DP depth");
                assert!(sim.f[5].pos.norm() < sim_empty.f[5].pos.norm(), "{bats:?} {label}: SS stays at DP depth");
            }
        }
    }

    #[test]
    fn middle_infielder_past_45m_breaks_for_infield_grounder() {
        let fx = fixture(&[]);
        let mut st = fx.st.clone();
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.fattrs[0].speed = 14.0;
        inp.fattrs[0].reaction = 0.05;
        inp.fattrs[3].reaction = 0.50;
        inp.ball = BallState { r: Vec3::new(0.0, 0.5, 0.5), v: Vec3::new(6.0, 48.0, -5.0), w: Vec3::ZERO };
        let mut sim = new_sim(&inp, &mut st);
        for f in sim.f.iter_mut() { f.err = [0.0; 4]; }
        sim.t = inp.t_contact + 0.19;
        sim.assign();
        assert_eq!(sim.f[0].role, Role::Chase, "setup expects P to go for it");
        assert!(sim.f[3].t_int.is_finite() && sim.f[3].target.norm() > 45.0,
            "2B intercept should lie past 45 m: {:?}", sim.f[3].target);
        assert_eq!(sim.f[3].role, Role::Chase, "2B starting at 46.6 m must still break for infield grounder");
    }

    #[test]
    fn dp_pivot_man_keeps_covering_second_when_partner_chases() {
        for make_p_primary in [false, true] {
            let mut fx = fixture(&[(5, 1)]);
            fx.st.outs = 1;
            let mut st = fx.st.clone();
            let mut inp = input(&fx, vec![
                RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
                RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
            ]);
            if make_p_primary {
                inp.fattrs[0].speed = 14.0;
                inp.fattrs[0].reaction = 0.1;
            }
            inp.ball = BallState { r: Vec3::new(0.0, 0.5, 0.5), v: Vec3::new(-1.5, 24.0, -3.0), w: Vec3::ZERO };
            let mut sim = new_sim(&inp, &mut st);
            for f in sim.f.iter_mut() { f.err = [0.0; 4]; }
            sim.t = inp.t_contact + 0.19;
            sim.assign();
            assert!((0..9).any(|i| sim.f[i].role == Role::Cover(2)),
                "make_p_primary={make_p_primary}: second base left uncovered (roles={:?})",
                sim.f.iter().map(|f| f.role).collect::<Vec<_>>());
        }
    }

    #[test]
    fn infielder_at_savant_depth_settles_without_return_throw() {
        let fx = fixture(&[]);
        let mut st = fx.st.clone();
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.bats = bb_observe::Hand::Right;
        let mut sim = new_sim(&inp, &mut st);
        let at = sim.f[3].pos;
        assert!(m::hypot(at.x, at.y) > 45.0);
        sim.r[0].s = BASE_L;
        sim.r[0].goal = 1;
        sim.r[0].v = 0.0;
        held_at(&mut sim, 3, at);
        sim.t = sim.f[3].ready_at + 1.0;
        sim.holder_act();
        assert!(sim.settled(), "2B at Savant depth {:.2} m must count as in infield", m::hypot(at.x, at.y));
        assert!(matches!(sim.ball, Ball::Held { by: 3 }), "no return throw from Savant 2B depth: {:?}", sim.ball);
    }

    // Opus infield-fixes review P2-2: the settle rule's boundary is the edge of the infield
    // dirt (155.5 ft = 47.3964 m on the line through second), not the break range. A second
    // baseman holding the ball on the grass 49 m out, inside the break range, does not settle
    // the play; and the scorer's outfield boundary is the same line.
    #[test]
    fn the_settle_boundary_is_the_infield_dirt_not_the_break_range() {
        assert!((INFIELD_R - 47.3964).abs() < 1e-3, "{INFIELD_R}");
        assert_eq!(INFIELD_R, bb_stats::scorer::DEFAULT_OUTFIELD_RADIUS_M);
        assert!(crate::play::MAX_INFIELD_DEPTH_M < INFIELD_R && INFIELD_R < INFIELD_BREAK_R);
        let fx = fixture(&[]);
        let mut st = fx.st.clone();
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        let mut sim = new_sim(&inp, &mut st);
        let at = Vec3::new(5.0, 48.7, 0.0);
        assert!(m::hypot(at.x, at.y) < INFIELD_BREAK_R);
        sim.r[0].s = BASE_L;
        sim.r[0].goal = 1;
        sim.r[0].v = 0.0;
        held_at(&mut sim, 3, at);
        sim.t = sim.f[3].ready_at + 1.0;
        assert!(!sim.settled(), "a fielder on the grass at {:.2} m is in the outfield", m::hypot(at.x, at.y));
    }

    #[test]
    fn first_baseman_breaks_on_right_side_infield_grounder() {
        let fx = fixture(&[]);
        let mut st = fx.st.clone();
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.bats = bb_observe::Hand::Right;
        inp.fattrs[3].speed = 26.0;
        inp.fattrs[3].reaction = 0.05;
        inp.ball = BallState { r: Vec3::new(0.0, 0.5, 0.5), v: Vec3::new(7.5, 19.0, 0.5), w: Vec3::ZERO };
        let mut sim = new_sim(&inp, &mut st);
        for f in sim.f.iter_mut() { f.err = [0.0; 4]; }
        sim.t = inp.t_contact + 0.19;
        sim.assign();
        assert_eq!(sim.f[3].role, Role::Chase, "setup expects 2B to go for this grounder (t_int_1b={:.3})", sim.f[2].t_int);
        assert_eq!(sim.f[2].role, Role::Chase, "1B must break for reachable right-side infield grounder: t_int={:.3}, bounce={:?}, target={:?}", sim.f[2].t_int, sim.f[2].belief.as_ref().and_then(|p| p.first_bounce()), sim.f[2].target);
        assert!(m::hypot(sim.f[2].target.x, sim.f[2].target.y) < INFIELD_BREAK_R, "1B intercept inside infield: {:?}", sim.f[2].target);
        assert!((0..9).any(|i| sim.f[i].role == Role::Cover(1)), "another fielder (P) must cover first");
    }

    #[test]
    fn first_baseman_at_standard_depth_carries_when_faster_than_pitcher() {
        let fx = fixture(&[]);
        let mut st = fx.st.clone();
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.bats = bb_observe::Hand::Right;
        let mut sim = new_sim(&inp, &mut st);
        let std_1b = sim.f[2].pos;
        assert!((std_1b - base_pos(1)).norm() > 10.5);
        sim.r[0].s = 12.0;
        sim.r[0].v = 7.5;
        sim.r[0].goal = 1;
        held_at(&mut sim, 2, std_1b);
        sim.f[0].role = Role::Cover(1);
        sim.f[0].target = base_pos(1);
        sim.t = sim.f[2].ready_at + 1.0;
        let opts = sim.holder_options(2);
        assert!(opts.iter().any(|o| o.base == 1 && o.carry && o.force),
            "carry 1 must be admitted when 1B beats waiting for P: {:#?}",
            opts.iter().map(|o| (o.base, o.carry, o.ball_at, o.receiver_at)).collect::<Vec<_>>());
        sim.holder_act();
        assert_eq!(sim.f[2].carry_to, Some(1), "1B should carry unassisted instead of stalling for P");
    }

    #[test]
    fn deep_relay_picks_closer_middle_infielder_by_alignment() {
        // Runner on 2nd: relay is toward home (0, 0); from RHB (+8, 92), spot is (+4, 46),
        // ~2 m from 2B (+5.5, 46.3) vs ~18 m from SS (-14.2, 42.9).
        let fx = fixture(&[(5, 2)]);
        let mut st_r = fx.st.clone();
        let mut inp_r = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
            RunnerIn { id: 5, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
        ]);
        inp_r.bats = bb_observe::Hand::Right;
        let mut sim_r = new_sim(&inp_r, &mut st_r);
        let from_rc = Vec3::new(8.0, 92.0, 0.0);
        held_at(&mut sim_r, 7, from_rc);
        assert_eq!(sim_r.relay_job(from_rc).map(|j| j.0), Some(3), "RHB (+8, 92) -> home: 2B is closer to relay spot than SS");

        // Against LHB (SS -6.2 deg, 2B +19.1 deg), SS starts closer to straightaway CF (0, 92) -> home.
        let fx_l = fixture(&[(5, 2)]);
        let mut st_l = fx_l.st.clone();
        let mut inp_l = input(&fx_l, vec![
            RunnerIn { id: fx_l.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
            RunnerIn { id: 5, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
        ]);
        inp_l.bats = bb_observe::Hand::Left;
        let mut sim_l = new_sim(&inp_l, &mut st_l);
        let from_cf = Vec3::new(0.0, 92.0, 0.0);
        held_at(&mut sim_l, 7, from_cf);
        assert_eq!(sim_l.relay_job(from_cf).map(|j| j.0), Some(5), "LHB (0, 92) -> home: SS is closer to relay spot than 2B");

        // Runner on 1st only (relay toward 3B): at RHB (+25, 88), relay spot toward 3B is near 2B.
        let fx_1 = fixture(&[(5, 1)]);
        let mut st_1 = fx_1.st.clone();
        let mut inp_1 = input(&fx_1, vec![
            RunnerIn { id: fx_1.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
            RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
        ]);
        inp_1.bats = bb_observe::Hand::Right;
        let mut sim_1 = new_sim(&inp_1, &mut st_1);
        let from_rf = Vec3::new(25.0, 88.0, 0.0);
        held_at(&mut sim_1, 8, from_rf);
        assert_eq!(sim_1.relay_job(from_rf).map(|j| j.0), Some(3), "RHB (+25, 88) -> 3B: 2B is closer to relay spot than SS");
    }

    #[test]
    fn carry_double_play_at_second_uses_carrier_continuation_and_out_weight() {
        const B2: usize = 3;
        const B1: usize = 2;
        const SS: usize = 5;
        let mut bonuses = Vec::new();
        for outs in [0u8, 1u8] {
            let mut fx = fixture(&[(5, 1), (7, 3)]);
            fx.st.outs = outs;
            let mut st = fx.st.clone();
            let mut inp = input(&fx, vec![
                RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
                RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
                RunnerIn { id: 7, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
            ]);
            inp.bats = bb_observe::Hand::Right;
            let mut sim = new_sim(&inp, &mut st);
            sim.t = inp.t_contact + 1.0;
            held_at(&mut sim, B2, Vec3::new(1.8, 38.8, 0.0));
            sim.f[B2].ready_at = sim.t;
            sim.f[B2].up_at = sim.t;
            sim.f[B1].role = Role::Cover(1);
            sim.f[B1].pos = base_pos(1);
            sim.f[B1].target = base_pos(1);
            sim.f[SS].pos = Vec3::new(-14.0, 42.0, 0.0);
            sim.f[1].role = Role::Cover(4);
            sim.f[1].pos = base_pos(4);
            sim.r[0].s = 4.0; sim.r[0].v = 8.2; sim.r[0].goal = 1;
            sim.r[1].s = BASE_L + 22.0; sim.r[1].v = 8.0; sim.r[1].goal = 2;
            sim.r[2].s = 3.0 * BASE_L + 23.0; sim.r[2].v = 7.5; sim.r[2].goal = 4;

            let opts = sim.holder_options(B2);
            let carry2 = opts.iter().find(|o| o.base == 2 && o.carry && o.force).expect("carry 2 option");
            let (p2, _) = carry2.continuation.expect("carry DP at second must evaluate continuation");
            assert!(p2 > 0.3, "expected throwable second out at first, p2={p2}");
            // The second out is counted (and recorded for the comparison across outs below).
            let bonus = carry2.value - carry2.p_out * carry2.p_out * carry2.weight;
            assert!(bonus > 0.0, "outs={outs}: the continuation adds value: {carry2:#?}");
            bonuses.push(bonus);

            sim.holder_act();
            assert_eq!(sim.f[B2].carry_to, Some(2), "outs={outs}: carry DP at 2B must beat single home throw");

            let mut fx0 = fixture(&[(5, 1), (7, 3)]);
            fx0.st.outs = outs;
            let mut st0 = fx0.st.clone();
            let mut inp0 = input(&fx0, vec![
                RunnerIn { id: fx0.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
                RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
                RunnerIn { id: 7, base: 3, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
            ]);
            inp0.bats = bb_observe::Hand::Right;
            inp0.fattrs[B2].throw_choice.continuation_weight = 0.0;
            let mut sim0 = new_sim(&inp0, &mut st0);
            sim0.t = sim.t;
            held_at(&mut sim0, B2, Vec3::new(1.8, 38.8, 0.0));
            sim0.f[B2].ready_at = sim0.t;
            sim0.f[B2].up_at = sim0.t;
            sim0.f[B1].role = Role::Cover(1); sim0.f[B1].pos = base_pos(1);
            sim0.f[SS].pos = Vec3::new(-14.0, 42.0, 0.0);
            sim0.f[1].role = Role::Cover(4); sim0.f[1].pos = base_pos(4);
            sim0.r[0].s = 4.0; sim0.r[0].v = 8.2; sim0.r[0].goal = 1;
            sim0.r[1].s = BASE_L + 22.0; sim0.r[1].v = 8.0; sim0.r[1].goal = 2;
            sim0.r[2].s = 3.0 * BASE_L + 23.0; sim0.r[2].v = 7.5; sim0.r[2].goal = 4;
            sim0.holder_act();
            assert_ne!(sim0.f[B2].carry_to, Some(2), "outs={outs}: without continuation, single home throw (w=1.3) beats carry 2");
        }
        // Same geometry: the second out that ends the inning (one out) is worth more than the
        // second out with none (Opus infield-fixes review P3-1 replaced checks that divided the
        // value back by its own formula).
        assert!(bonuses[1] > bonuses[0], "inning-ending DP worth more: {bonuses:?}");
    }

    #[test]
    fn carry_non_second_force_dp_uses_carrier_post_touch_setup_clock() {
        const B3: usize = 4;
        const B1: usize = 2;
        let mut fx = fixture(&[(5, 1), (6, 2)]);
        fx.st.outs = 0;
        let mut st = fx.st.clone();
        let mut inp = input(&fx, vec![
            RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
            RunnerIn { id: 5, base: 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
            RunnerIn { id: 6, base: 2, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S },
        ]);
        inp.bats = bb_observe::Hand::Right;
        let mut sim = new_sim(&inp, &mut st);
        sim.t = inp.t_contact + 1.0;
        held_at(&mut sim, B3, base_pos(3) + Vec3::new(0.0, 2.5, 0.0));
        // He picked it up a second ago and is ready to go.
        sim.f[B3].ready_at = sim.t;
        sim.f[B3].up_at = sim.t - 1.0;
        sim.f[B1].role = Role::Cover(1);
        sim.f[B1].pos = base_pos(1);
        sim.f[B1].target = base_pos(1);
        sim.r[0].s = 9.0; sim.r[0].v = 8.2; sim.r[0].goal = 1;
        sim.r[1].s = BASE_L + 22.0; sim.r[1].v = 8.0; sim.r[1].goal = 2;
        sim.r[2].s = 2.0 * BASE_L + 23.0; sim.r[2].v = 7.5; sim.r[2].goal = 3;

        let opts = sim.holder_options(B3);
        let carry3 = opts.iter().find(|o| o.base == 3 && o.carry && o.force).expect("carry 3 option");
        let (p2, ball_at_1) = carry3.continuation.expect("carry at 3B must use carrier post-touch continuation");
        assert!(p2 > 0.3, "expected continuation throw to 1B, p2={p2}");
        // The carrier has held the ball for a second before the carry, so he is set when he
        // touches third and throws at once at full speed, the first baseman being on
        // his bag: the ball reaches first one flight of the 38.8 m throw (+0.05 s handling)
        // after the touch. A receiver's catch-to-release (0.7 × transfer, the non-carry
        // branch) would add about 0.45 s (Opus infield-fixes review P3-1: the old bound,
        // touch + 0.5 s, held for either branch).
        let fd = &sim.f[B3];
        assert!(carry3.ball_at - fd.up_at > fd.a.transfer + 0.2, "setup: he is set before the touch");
        let flight = throw_time(THROW_EFFORT * fd.a.arm, (base_pos(1) - base_pos(3)).norm());
        let expect = carry3.ball_at + flight + 0.05;
        assert!((ball_at_1 - expect).abs() < 0.02, "ball at first {ball_at_1:.3}, expected {expect:.3} (touch {:.3}, flight {flight:.3})", carry3.ball_at);
        assert!(RELAY_TRANSFER_FRAC * fd.a.transfer > 0.3, "the branches are told apart");
        assert!(carry3.value > carry3.p_out * carry3.p_out * carry3.weight, "the continuation is counted: {carry3:#?}");
    }

    /// The first baseman's own read of where he would field the ball (his intercept, before
    /// any cover job replaces his target).
    fn own_intercept(sim: &Sim, i: usize) -> (f64, Vec3) {
        let p = sim.f[i].belief.clone().expect("he has read the ball");
        sim.intercept(i, &p)
    }

    // Opus infield-fixes review P1-1: a grounder to the left of the mound that the first
    // baseman could reach inside the break range. The pitcher fields it; the first baseman
    // goes to his bag instead of breaking across the infield.
    #[test]
    fn first_baseman_goes_to_the_bag_on_a_grounder_left_of_the_mound() {
        for bats in [bb_observe::Hand::Right, bb_observe::Hand::Left] {
            let fx = fixture(&[]);
            let mut st = fx.st.clone();
            let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
            inp.bats = bats;
            inp.fattrs[0].speed = 14.0;
            inp.fattrs[0].reaction = 0.05;
            inp.fattrs[2].speed = 14.0;
            inp.ball = BallState { r: Vec3::new(0.0, 0.5, 0.5), v: Vec3::new(-2.5, 13.0, 0.5), w: Vec3::ZERO };
            let mut sim = new_sim(&inp, &mut st);
            for f in sim.f.iter_mut() { f.err = [0.0; 4]; }
            sim.t = inp.t_contact + 0.19;
            sim.assign();
            let (ti, pt) = own_intercept(&sim, 2);
            assert_eq!(sim.f[0].role, Role::Chase, "{bats:?}: setup expects the pitcher to field it");
            assert!(ti.is_finite() && m::hypot(pt.x, pt.y) < INFIELD_BREAK_R && pt.x < 0.0, "{bats:?}: setup expects a 1B intercept left of the mound in range: {ti:.2} {pt:?}");
            assert_eq!(sim.f[2].role, Role::Cover(1), "{bats:?}: 1B must take his bag, roles {:?}", sim.f.iter().map(|f| f.role).collect::<Vec<_>>());
        }
    }

    // P1-1, the other half of "first will be covered": on the right side the pitcher fields
    // it himself and the second baseman is also going for it, so nobody else can take first.
    #[test]
    fn first_baseman_stays_with_the_bag_when_nobody_else_can_take_first() {
        let fx = fixture(&[]);
        let mut st = fx.st.clone();
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
        inp.bats = bb_observe::Hand::Right;
        inp.fattrs[0].speed = 14.0;
        inp.fattrs[0].reaction = 0.05;
        inp.fattrs[2].speed = 14.0;
        inp.fattrs[3].speed = 14.0;
        inp.fattrs[3].reaction = 0.05;
        inp.ball = BallState { r: Vec3::new(0.0, 0.5, 0.5), v: Vec3::new(3.0, 13.0, 0.5), w: Vec3::ZERO };
        let mut sim = new_sim(&inp, &mut st);
        for f in sim.f.iter_mut() { f.err = [0.0; 4]; }
        sim.t = inp.t_contact + 0.19;
        sim.assign();
        let (ti, pt) = own_intercept(&sim, 2);
        let roles: Vec<Role> = sim.f.iter().map(|f| f.role).collect();
        assert_eq!(sim.f[0].role, Role::Chase, "setup expects the pitcher to field it: {roles:?}");
        assert!(ti.is_finite() && m::hypot(pt.x, pt.y) < INFIELD_BREAK_R && pt.x > 0.0, "setup expects a 1B intercept right of the mound in range: {ti:.2} {pt:?}");
        assert!(matches!(sim.f[3].role, Role::Chase | Role::Cover(2)), "setup expects the 2B taken: {roles:?}");
        assert_eq!(sim.f[2].role, Role::Cover(1), "1B must stay with first: {roles:?}");
    }

    // Agent-models §5.7, the owner's saved play (seed 65147 play 10): a chopper off the plate
    // goes through the right side. The centre and right fielders each read it as the other's
    // and both stayed. The right fielder now cuts it off in front of the warning track.
    #[test]
    fn an_outfielder_takes_a_grounder_through_the_hole_both_left_to_the_other() {
        let mut g = crate::game::Game::new(65147, false);
        while let Some(step) = g.step_pitch() {
            if step.play != 10 {
                continue;
            }
            let first = step.facts.iter().find_map(|f| match f {
                Input::GloveSecured { t, fielder, x, y } => Some((*t, *fielder, *x, *y)),
                _ => None,
            });
            let (t, who, x, y) = first.expect("someone fields it");
            let contact = step.facts.iter().find_map(|f| match f {
                Input::BatContact { t } => Some(*t),
                _ => None,
            }).expect("contact on play 10");
            assert_eq!(who & 0x7F, 9, "fielder {who} secured it first at ({x:.1}, {y:.1})");
            assert!(m::hypot(x, y) < 100.0 && t - contact < 6.0, "fielded at ({x:.1}, {y:.1}), {t:.2} s");
            return;
        }
        panic!("no play 10");
    }

    // P1-1: a first baseman who broke for a ball sees the pitcher secure it and, after his
    // reaction time, goes back to his bag; with first already taken he pulls up instead.
    #[test]
    fn a_chaser_who_sees_a_teammate_secure_the_ball_goes_to_his_base() {
        for first_taken in [false, true] {
            let fx = fixture(&[]);
            let mut st = fx.st.clone();
            let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }]);
            let mut sim = new_sim(&inp, &mut st);
            sim.t = inp.t_contact + 2.0;
            held_at(&mut sim, 0, Vec3::new(-5.0, 14.0, 0.0));
            sim.secured_at = sim.t;
            sim.out.secured.push((sim.t, sim.f[0].id));
            for i in 0..9 {
                if sim.f[i].role == Role::Cover(1) {
                    go_idle(&mut sim.f[i]);
                }
            }
            if first_taken {
                known_doing(&mut sim, 3, Role::Cover(1));
            }
            sim.f[2].role = Role::Chase;
            sim.f[2].pos = Vec3::new(16.2, 29.3, 0.0);
            sim.f[2].vel = Vec3::new(-6.0, 0.5, 0.0);
            sim.f[2].target = Vec3::new(-17.5, 31.6, 0.0);
            let seen = sim.secured_at + sim.f[2].a.reaction;
            sim.t = seen - 0.01;
            sim.chasers_see_the_ball_secured();
            assert_eq!(sim.f[2].role, Role::Chase, "he has not seen it yet");
            sim.t = seen;
            sim.chasers_see_the_ball_secured();
            if first_taken {
                assert_eq!(sim.f[2].role, Role::Idle, "first is taken: he pulls up");
                assert!(sim.f[2].target.x > 0.0, "he stops, not on across the infield: {:?}", sim.f[2].target);
            } else {
                assert_eq!(sim.f[2].role, Role::Cover(1));
                assert_eq!(sim.f[2].target, base_pos(1));
            }
            assert!(matches!(sim.ball, Ball::Held { by: 0 }));
            assert_eq!(sim.f[0].role, Role::Chase, "the holder keeps his ball");
        }
    }
}
