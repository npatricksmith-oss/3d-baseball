//! Live-ball plays, causal version (M4b): the world advances in fixed ticks; fielders and
//! runners decide only from what they have observed so far; the rules engine is stepped
//! with each fact as it happens, so players learn of outs the way they would on a field.
//!
//! Replaces the skeleton in `play.rs` (kept for the diagnostics examples), which planned from
//! the realized future (Codex M4 review #5, #6, #9, #10, #21, #24).
//!
//! Remaining simplifications, documented for M5: fielder perception is a noisy estimate of
//! the ball's current state fed into the agent's own physics (smooth ground, typical spin)
//! rather than the full atlas model of agent-models §5.1; throws never choose a one-hop
//! trajectory; one cutoff or relay man at most (no double cut); no slides, rundown tactics,
//! or collisions; coverage follows a small fixed table.

use bb_math::{KeyedRng, Mat3, Purpose, RngKey, Vec3, m};
use bb_physics::ground::{BallPath, Field, SegmentEnd, WallContact, batted_path, batted_path_dt, over_the_wall, wall_contact};
use bb_physics::throw::{THROW_MAX_ELEV, THROW_SPIN_RPM, ThrowTable, fly_throw, fly_throw_in, solve_throw, throw_state};
use bb_physics::{BallState, FlightEnv, Trajectory};
use bb_rules::{AgentId, Event, Input, OutKind, PlayState, ReleaseKind, RulesState, RunnerPosition, Touch, step};

use bb_agents::motor::{ThrowNoise, execute_throw};
use bb_agents::throwing::{self, SetUp};

use crate::play::{FielderAttrs, RunnerIn, base_pos, home_positions, run_dist, run_time};

/// Opt-in, read-only A4 evidence. Never consulted by the option evaluation.
pub mod choice_trace;
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
/// A catch below this height is a shoestring catch: a close call for the umpire, m [E].
const LOW_CATCH_Z: f64 = 0.4;
pub(crate) const CATCH_MAX_Z: f64 = 2.4;
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
/// Time constant of a runner's acceleration, s, fitted to measurements (tag-up sources note
/// §1, §5): at 8.2 m/s a standing 90 ft then takes 4.02 s, the mean 90-ft split of runners
/// on third in sacrifice-fly situations (4.03 s, Statcast 2017–2020, n = 848), and with
/// the batter's start delay an average runner reaches first about 4.28 s after contact
/// (2016 means 4.26 s left-handed, 4.30 s right-handed). Was 0.85 s (4.19 s; 4.45 s).
const RUNNER_TAU: f64 = 0.68;
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

/// A runner's estimate of how long the defense needs to get the ball over horizontal
/// distance `d` by throwing: a league-typical arm, relayed through a man halfway when that is
/// quicker or the throw cannot carry.
/// A runner's read of how long the man holding the ball needs to get it `d` m by throwing:
/// his read of that fielder's arm, relayed through a man halfway (a league-typical arm) when
/// that is quicker or the throw cannot carry (agent-models §6.4).
fn read_throw_time(arm: f64, d: f64) -> f64 {
    let v = THROW_EFFORT * arm;
    throw_time(v, d).min(throw_time(v, 0.5 * d) + RELAY_TRANSFER_FRAC * TYPICAL_TRANSFER + throw_time(THROW_EFFORT * TYPICAL_ARM, 0.5 * d))
}

/// A runner's read of a fielder's arm is his position's grade-50 arm (scouting-level
/// knowledge) off by this σ, m/s [E, unsourced]: half the population spread of arm grades
/// (`ARM_STRENGTH_SIGMA_MPS` 2.2), so he knows a strong arm from a weak one only roughly.
const ARM_READ_SIGMA_MPS: f64 = 1.1;
/// Sub-indices of the runner-perception stream used for his read of the holder's arm: this
/// base plus the fielder's position index (far above the per-decision subs 2n, 2n + 1).
const ARM_READ_SUB: u32 = 1 << 20;

fn typical_throw_time(d: f64) -> f64 {
    let v = THROW_EFFORT * TYPICAL_ARM;
    throw_time(v, d).min(2.0 * throw_time(v, 0.5 * d) + RELAY_TRANSFER_FRAC * TYPICAL_TRANSFER)
}
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
/// With two out, a tag play's weight against a force's 1.0 when the holder picks his throw
/// [E]: an equally sure tag loses to the force (a run can score ahead of a tag for the
/// third out, never ahead of a force), but a sure tag still beats a doubtful force.
const TWO_OUT_TAG_W: f64 = 0.9;
/// Infield radius for the settle rule (rules-spec §5.8), m. The spec's 30 m excluded second
/// base (38.8 m from home); the infield dirt reaches ~45 m.
const INFIELD_R: f64 = 45.0;
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
/// axis (stretched in depth like the position error), shrinking with viewing time like it;
/// interim stand-in for §5.1's launch-state errors, refitted with momentum 2026-11-12 [E].
const READ_VEL_FRAC: f64 = 0.04;
/// What an observer allows for the spin he cannot see when he works out where a ball must be
/// from the plate (agent-models §5.1): σ per axis, m, at 1 s since contact, growing as s^1.5.
/// `examples/plate_cue.rs` (2026-11-13): flying the true velocity back with typical spin
/// misses by a median 0.6 m at 0.5 s, 2.3 m at 1 s and 4.5–9 m at 2 s (0 with the true
/// spin); a 3D miss's median is about 1.5 σ per axis [E].
const PLATE_SPIN_M: f64 = 1.5;
/// Below this speed a fielder reaching his spot simply stops on it (one tick's step), m/s [E].
const SETTLE_V: f64 = 0.5;
/// The batter's typical pull: how far to his pull side of straightaway his balls in play go
/// on average, degrees (right-handers −5.0° in `examples/spray_by_location.rs`) [E].
const MEAN_PULL_DEG: f64 = 5.0;
/// The scatter of balls in play about the direction a pitch location suggests, degrees
/// (σ within a 10 cm bin, `examples/spray_by_location.rs`) [E].
const SPRAY_SCATTER_DEG: f64 = 22.0;
/// A fielder's belief about what the bat's angle at contact says about the ball's direction
/// (agent-models §5.10): spray ≈ `SWING_SPRAY_A_DEG` + `SWING_SPRAY_B` × face, with
/// `SWING_SPRAY_RESID_DEG` of scatter about the line, batter-relative (negative = his pull
/// side). The face is the barrel's yaw less 90° (mirrored for a left-hander). Least-squares
/// fit to 5,386 fair balls in play, `examples/spray_by_bat_angle.rs` (96 games, 2026-11-12);
/// refit when the swing or contact model changes [E].
const SWING_SPRAY_A_DEG: f64 = 5.2;
const SWING_SPRAY_B: f64 = 1.36;
const SWING_SPRAY_RESID_DEG: f64 = 18.6;
/// A fielder's belief about what the swing's lead says about the bat's face at contact
/// (agent-models §5.10): face ≈ `LEAD_FACE_A_DEG` + `LEAD_FACE_B_DPS` × lead (s), with
/// `LEAD_FACE_RESID_DEG` of scatter, batter-relative. Least-squares fit to 11,588 fair balls in
/// play, `examples/spray_cues.rs` (200 games, 2026-11-13); refit when the swing model changes
/// [E].
const LEAD_FACE_A_DEG: f64 = 161.93;
const LEAD_FACE_B_DPS: f64 = -1063.1;
const LEAD_FACE_RESID_DEG: f64 = 3.6;
/// A fielder jumps only for a ball he expects within this angle of his line from home,
/// degrees (infield, outfield) [E].
const JUMP_ZONE_DEG: [f64; 2] = [20.0, 25.0];
/// He commits fully when he expects the ball this many of his σs to one side, [E].
const JUMP_FULL_SIGMAS: f64 = 2.0;
/// A fielder's braking deceleration, m/s² [E].
const BRAKE: f64 = 6.0;
/// Time to spare a fielder needs to believe he has before he calls a ball in the air, s [E].
const CALL_SLACK_S: f64 = 0.3;
/// On a re-call, a fielder in the chaser's tier takes the ball only with this much more time
/// to spare (or an earlier intercept), s [E]: calls are not traded on noise.
const RECALL_MARGIN_S: f64 = 0.3;
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum Role {
    Chase,
    Cover(u8),
    /// Cutoff or relay man for throws to this base (agent-models §5.7).
    Relay(u8),
    Backup,
    Idle,
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
    /// He took it on the ground from a dive (set at each securing; `dove` is his recovery
    /// state and is cleared while he still holds the ball: Codex hurried-throws review #1).
    took_diving: bool,
    /// The base whose throws he is backing up, if that is his job on this play.
    backs: Option<u8>,
    /// Called off a ball in the air by a sure call (agent-models §5.7): the spot he backs up
    /// the catch from, short of where his own read has it coming down. Kept across re-calls
    /// he is not part of, until the ball comes down or he is given another job.
    peel: Option<Vec3>,
    /// The bag he is carrying the ball to (agent-models §5.5 A4), so that abandoning the
    /// carry for a throw or for no play brakes him instead of leaving him running at it.
    carry_to: Option<u8>,
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
    /// Base the runner is heading for (may be behind it on a retreat).
    goal: u8,
    out: bool,
    scored: bool,
    /// Holding at a base until a catch is made (tag-up).
    tag_up: bool,
    /// He tagged up and is going on the catch: when his first step comes. Until he next
    /// stops he runs from the set start (agent-models §6.4); no earlier speed carries over.
    set_start: Option<f64>,
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
    /// How runners who tag up start, and what they believe of the tag (agent-models §6.4).
    pub tag_up: bb_agents::running::TagUpParams,
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
}

#[derive(Clone, Copy, Debug)]
pub struct ScorerCarryFact { pub fielder: AgentId, pub t: f64, pub base: u8, pub batter_goal: u8 }

pub struct LiveOutput {
    pub scorer_carries: Vec<ScorerCarryFact>,
    /// First glove contact with the batted ball, including a deflection.
    pub first_handling: Option<(f64, AgentId)>,
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
        | Input::BallReleasedFromGlove { t, .. }
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
fn visible_target(r: &Runner, params: throwing::ChoiceParams) -> Option<u8> {
    params.runner_target(r.s, r.direction * r.v, BASE_L)
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

/// How long a runner leaving from a tag-up needs to cover `d` m: the wait for his first step,
/// then the speed law from rest with the set start's time constant (agent-models §6.4). His
/// decision uses this and his legs run the same law; no earlier speed carries over.
fn tag_up_time(tag: bb_agents::running::TagUpParams, wait_s: f64, d: f64, v_max: f64) -> f64 {
    wait_s + run_time(d, v_max, tag.tau_s)
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

/// The backspin (rpm; negative: topspin) a batted ball typically has at launch angle `la`
/// (rad): a fielder's belief about the contact model. Medians by 10° launch-angle bin from
/// `examples/batted_spin.rs` (48 games, 2026-11-12), interpolated linearly at the bin
/// centres; refit when the contact model changes.
fn typical_batted_spin_rpm(la: f64) -> f64 {
    const DEG: [f64; 9] = [-25.0, -15.0, -5.0, 5.0, 15.0, 25.0, 35.0, 45.0, 55.0];
    const RPM: [f64; 9] = [-7060.0, -4750.0, -2820.0, -710.0, 1330.0, 3290.0, 4800.0, 5990.0, 6470.0];
    let x = la.to_degrees().clamp(DEG[0], DEG[8]);
    let k = (((x - DEG[0]) / 10.0).floor() as usize).min(7);
    let f = (x - DEG[k]) / 10.0;
    RPM[k] + (RPM[k + 1] - RPM[k]) * f
}

/// The spin an observer believes a ball has (agent-models §5.1): straight off the bat, the
/// spin the contact model typically gives a ball at the launch angle he estimates (his read of
/// its velocity `ve` with gravity's pull over the `off_bat_since` seconds since contact added
/// back); after a bounce or a deflection, a little topspin.
pub fn believed_spin(ve: Vec3, off_bat_since: Option<f64>) -> Vec3 {
    let h = Vec3::new(ve.x, ve.y, 0.0).normalized();
    let axis = Vec3::new(h.y, -h.x, 0.0);
    let rpm = match off_bat_since {
        Some(since) => {
            let la = m::atan2(ve.z + bb_math::units::G * since, m::hypot(ve.x, ve.y));
            typical_batted_spin_rpm(la)
        }
        None => -600.0,
    };
    axis * (rpm * core::f64::consts::PI / 30.0)
}

/// Where a ball going `ve` now (spin `w`) must be if it left `p0` `s` seconds ago: its
/// velocity flown backward through those seconds with the observer's own physics (drag and
/// gravity act on the velocity alone, not on where the ball is), adding up the distance.
/// The knowledge that a batted ball started at the plate (agent-models §5.1).
pub fn position_from_plate(env: &FlightEnv, p0: Vec3, ve: Vec3, w: Vec3, s: f64) -> Vec3 {
    let mut st = BallState { r: Vec3::ZERO, v: ve, w };
    let mut left = s;
    while left > 1e-9 {
        let h = left.min(AGENT_DT);
        st = env.step(&st, -h);
        left -= h;
    }
    p0 - st.r
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

/// The σ of an observer's read of where the ball comes down along his line of sight, m: a
/// fielder's knowledge of how good his own read is (a belief about himself, not about the
/// ball; agent-models §5.7 calls). He runs his own read ([`read_prediction`]) from what he
/// believes the ball is doing now (`belief` at `t`, seen from `from`), once as it is and once
/// with each of its four error draws at one σ, and adds up in quadrature how far each moves
/// where he predicts it comes down, along his line of sight to it; for a ball still on its
/// first flight, with the spin he cannot see (`PLATE_SPIN_M`) in the plate's share of his
/// depth estimate. Running his own physics carries the drag, the plate cue's use of the same
/// velocity he flies forward, and the coupled lateral and height draws (Codex plate-cue review
/// #1: the first version added a position term and a straight-line velocity term in
/// quadrature, which missed the coupling and ignored drag).
#[allow(clippy::too_many_arguments)]
pub fn landing_depth_sigma(env: &FlightEnv, field: &Field, belief: (Vec3, Vec3), t: f64, from: Vec3, sigma0: f64, tau_read: f64, view: f64, off_bat: Option<(f64, Vec3)>) -> f64 {
    let land = |err: [f64; 4]| {
        let path = read_prediction(env, field, belief, t, from, sigma0, tau_read, view, err, off_bat);
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
        let shrink = 1.0 / m::sqrt(1.0 + view / tau_read);
        let sp = sigma0 * (m::hypot(p.x - from.x, p.y - from.y).max(1.0) / READ_REF_DIST) * shrink;
        let (_, _, k) = plate_weighing(los, sp, READ_VEL_FRAC * v.norm() * shrink, s);
        let spin = k * PLATE_SPIN_M * s * m::sqrt(s);
        var += spin * spin;
    }
    m::sqrt(var)
}

/// An observer's prediction of the ball (agent-models §5.1, interim model): a noisy estimate
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
    let (p, v) = truth;
    let dist = m::hypot(p.x - from.x, p.y - from.y).max(1.0);
    let shrink = 1.0 / m::sqrt(1.0 + view / tau_read);
    let sp = sigma0 * (dist / READ_REF_DIST) * shrink;
    let los = Vec3::new(p.x - from.x, p.y - from.y, 0.0).normalized();
    let lat = Vec3::new(-los.y, los.x, 0.0);
    // Height is read with the lateral precision (an angular measurement).
    let mut pe = p + los * (err[0] * sp * DEPTH_STRETCH) + lat * (err[1] * sp) + Vec3::new(0.0, 0.0, err[1] * sp * 0.5);
    let sv = READ_VEL_FRAC * v.norm() * shrink;
    let ve = v + los * (err[2] * sv * DEPTH_STRETCH * 0.5) + lat * (err[3] * sv) + Vec3::new(0.0, 0.0, err[3] * sv * 0.5);
    let off_bat_since = off_bat.map(|(s, _)| s);
    let w = believed_spin(ve, off_bat_since);
    if let Some((s, p0)) = off_bat
        && s > 0.0
    {
        // The plate: where a ball going as fast as he reads it must be, weighed against his
        // direct read by how good each is.
        let (gain, _, _) = plate_weighing(los, sp, sv, s);
        pe += gain * (position_from_plate(env, p0, ve, w, s) - pe);
    }
    batted_path_dt(env, field, BallState { r: pe, v: ve, w }, t, AGENT_DT, |_, _| (0.0, 0.0))
}

/// How much farther toward the batter's pull side than his average a ball is hit, by where
/// the pitch is (agent-models §5.10): degrees of spray (+ = pulled more), against the pitch's
/// distance toward the batter from the middle of the plate, m (+ = inside). A fielder's
/// belief about the contact model: means by 10 cm bin from `examples/spray_by_location.rs`
/// (96 games of right-handed batters, 2026-11-12) less the overall mean (−5.0°), linear
/// between bin centres, flat beyond the last. Refit when the contact model changes.
fn anticipated_pull_deg(x_inside: f64) -> f64 {
    const X: [f64; 7] = [-0.30, -0.20, -0.10, 0.0, 0.10, 0.20, 0.30];
    const PULL: [f64; 7] = [-6.5, -4.5, -3.6, 0.0, 3.4, 7.2, 7.7];
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
fn run_gap(fd: &Fielder, goal: Vec3, t: f64, reach: f64) -> f64 {
    let to = Vec3::new(goal.x - fd.pos.x, goal.y - fd.pos.y, 0.0);
    let d = to.norm();
    let (u, w) = if d < 1e-9 { (0.0, fd.vel.norm()) } else {
        let u = fd.vel.dot(to) / d;
        (u, (fd.vel - to * (u / d)).norm())
    };
    let (vmax, tau, a1) = (fd.a.speed, fd.a.tau_a, fd.a.a_lat);
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

/// Time for a fielder to come within `reach` of `goal` from his current motion: the first
/// zero of [`run_gap`].
fn run_time_to(fd: &Fielder, goal: Vec3, reach: f64) -> f64 {
    if run_gap(fd, goal, 0.0, reach) <= 0.0 {
        return 0.0;
    }
    let (mut lo, mut hi) = (0.0, 1.0);
    while run_gap(fd, goal, hi, reach) > 0.0 && hi < 120.0 {
        lo = hi;
        hi *= 2.0;
    }
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if run_gap(fd, goal, mid, reach) > 0.0 { lo = mid } else { hi = mid }
    }
    hi
}

/// The glove σ multiplier (agent-models §5.4 form) for a ball at speed `v` taken `horiz`
/// from the body. k_v 0.25 and k_reach 0.3 [E, M4b]: the spec's 0.5/0.6 made routine catches
/// fail ~8% of the time against the ≥ 98% routine-play band. Off his feet or off balance
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
/// reach; physics-spec §9.5). Players see the walls, so plans use it.
fn beyond_reach_over_wall(field: &Field, b: Vec3) -> f64 {
    -field.body_clearance(b.x, b.y, 0.0).dist - (REACH - BODY_R)
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
    // Within a few centimetres and nearly stopped, he is on his spot.
    // Within a few centimetres and nearly stopped, he is on his spot. Moving faster, he
    // carries on (Codex momentum review #1: arriving never stops him dead).
    if d < 0.05 && fd.vel.norm() < SETTLE_V {
        return (Vec3::new(fd.target.x, fd.target.y, fd.pos.z), Vec3::ZERO);
    }
    if d < 1e-9 {
        return (fd.pos + fd.vel * DT, fd.vel);
    }
    let dir = to * (1.0 / d);
    let along = fd.vel.dot(dir);
    // A chaser fields the ball on the run: his intercept plan (`run_dist`) has him arriving at
    // speed, so he slows for the spot only if he will be there before the ball.
    let early = fd.role != Role::Chase || !fd.t_int.is_finite() || t + d / along.max(0.5) + 0.1 < fd.t_int;
    let mut want = if early { fd.a.speed.min(m::sqrt(2.0 * BRAKE * d)) } else { fd.a.speed };
    // He slows into a turn he cannot make at speed: reaching a spot d away at an angle θ off
    // his heading takes a curve of radius d/(2 sin θ), which his lateral acceleration allows
    // up to √(a_lat·d/(2 sin θ)); a spot behind him, a stop first.
    let speed_now = fd.vel.norm();
    if speed_now > 1e-6 {
        let cos_t = fd.vel.dot(dir) / speed_now;
        let sin_t = if cos_t >= 0.0 { m::sqrt((1.0 - cos_t * cos_t).max(0.0)) } else { 1.0 };
        if sin_t > 1e-6 {
            want = want.min(m::sqrt(fd.a.a_lat * d / (2.0 * sin_t)));
        }
    }
    // Speeding up in his direction of motion follows the speed law; slowing (braking at most
    // `BRAKE`) and turning (at most `a_lat`) take what the change needs, up to those limits.
    let delta = dir * want - fd.vel;
    let speed = fd.vel.norm();
    let acc = if speed < 1e-6 {
        delta * (1.0 / fd.a.tau_a)
    } else {
        let vh = fd.vel * (1.0 / speed);
        let dpar = delta.dot(vh);
        let dperp = delta - vh * dpar;
        let par = if dpar >= 0.0 { dpar / fd.a.tau_a } else { -(BRAKE.min(-dpar / DT)) };
        let np = dperp.norm();
        let perp = if np < 1e-12 { Vec3::ZERO } else { dperp * (fd.a.a_lat.min(np / DT) / np) };
        vh * par + perp
    };
    let vel = fd.vel + acc * DT;
    // Reaching the target ends the run: a fielder standing on his spot has no speed to carry
    // into the next plan.
    if vel.dot(dir) * DT >= d && vel.norm() < SETTLE_V {
        return (Vec3::new(fd.target.x, fd.target.y, fd.pos.z), Vec3::ZERO);
    }
    (fd.pos + vel * DT, vel)
}

struct Sim<'a> {
    inp: &'a LiveInput<'a>,
    /// Latest fact time emitted (facts never go backward, rules-spec §1.2).
    last_fact_t: f64,
    /// When the ball was last caught (a visible event).
    secured_at: f64,
    /// A fly was caught (the rules engine signalled `Catch`).
    catch_made: bool,
    /// The latest catch was low or diving: a close call for the umpire.
    low_catch: bool,
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
        let t_fact = fact_time(&fact).unwrap_or(self.t);
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
                self.catch_made = true;
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

    /// An observer's prediction of the ball from where he stands: [`read_prediction`] of its
    /// state now.
    fn predict_from(&self, from: Vec3, sigma0: f64, tau_read: f64, view: f64, err: [f64; 4]) -> BallPath {
        read_prediction(self.inp.env, self.inp.field, self.ball_at(self.t), self.t, from, sigma0, tau_read, view, err, self.off_bat())
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
        let field = self.inp.field;
        while t <= t_end {
            let (b, _) = pred.at(t);
            // Not where nobody can reach: farther past a wall than a fielder reaches over it.
            if b.z <= CATCH_MAX_Z && beyond_reach_over_wall(field, b) <= 0.0 {
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
        let (fd, start) = self.planning_state(i);
        let t_end = pred.t_end();
        let (rest, _) = pred.at(t_end);
        (t_end.max(start + run_time_to(&fd, rest, 0.0)), Vec3::new(rest.x, rest.y, 0.0))
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
        let fd: &Fielder = &fd;
        let t_end = pred.t_end();
        let mut dive = None;
        // Shortfall at time `t` for a ball at `b`: how far short of the ball's line the
        // fielder still is (≤ 0: he can field it standing, with the planning reach).
        let gap = |t: f64, b: Vec3| run_gap(fd, b, (t - start).max(0.0), REACH * 0.7);
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
                // Players see the walls: no intercept past a wall, beyond his reach over it
                // (Gemini play-area review P2-3).
                if b.z > CATCH_MAX_Z || t < start || beyond_reach_over_wall(self.inp.field, b) > 0.0 {
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
        // The fielders nearest where the chaser is going (which everyone can see), not the
        // ball's position now: a pop-up is over the infield while it climbs.
        let b = self.f[self.primary].target;
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
        // Calls on a ball in the air (agent-models §5.7): (tier, value) per candidate.
        let mut calls: Vec<(usize, (u8, f64))> = Vec::new();
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
            calls.push((i, self.call_value(i, &p, ti, pt, bias)));
            self.f[i].t_int = ti;
            self.f[i].target = pt;
            self.f[i].belief = Some(p);
            self.f[i].next_plan = self.t + REPLAN_S;
        }
        // The best call: a sure catch in the air (outfielders first, then the most time to
        // spare), then a catch in the air nobody is sure of yet (the most time to spare), over
        // everything else; otherwise the earliest intercept. On a re-call the
        // chaser keeps the ball unless someone calls it in a higher tier, or in his tier with
        // `RECALL_MARGIN_S` more to spare.
        let key = |c: &(usize, (u8, f64))| c.1;
        let top = calls.iter().copied().max_by(|a, b| key(a).0.cmp(&key(b).0).then(key(a).1.total_cmp(&key(b).1)).then(b.0.cmp(&a.0)));
        let mut chosen = top.map(|c| c.0).unwrap_or(best.1);
        if let (Some(cur), Some(top)) = (fresh.and_then(|p| calls.iter().copied().find(|c| c.0 == p)), top) {
            if top.0 != cur.0 && top.1.0 == cur.1.0 && top.1.1 <= cur.1.1 + RECALL_MARGIN_S {
                chosen = cur.0;
            }
        }
        self.primary = chosen;
        // Each fielder's own intercept, before cover jobs replace some targets with bases.
        let own: Vec<Vec3> = self.f.iter().map(|fd| fd.target).collect();
        self.cover_roles();
        // Ground balls: every infielder who believes he can get to it in the infield breaks
        // for it at once; nobody waits for a call (agent-models §5.7, ground-ball case). Each
        // judges from his own read: that it is a ground ball (it comes down within 25 m in
        // the first `GROUNDER_BOUNCE_S`; an infield pop-up also lands in the infield, seconds
        // later) and that his intercept is in the infield. He runs at that intercept, not at
        // the base the cover step just gave him.
        let mut broke = false;
        let t_c = self.inp.t_contact;
        // A ball in the air that nobody has called (the chaser is not sure of it): everyone
        // else who believes he catches it in the air keeps going for it too, until someone
        // is sure and calls it; then the others peel off (agent-models §5.7).
        if calls.iter().any(|c| c.0 == chosen && c.1.0 == 1) {
            for c in &calls {
                if c.0 != chosen && c.1.0 == 1 {
                    self.f[c.0].role = Role::Chase;
                    self.f[c.0].target = own[c.0];
                    broke = true;
                }
            }
        }
        // A sure call: an infielder who was going for it himself (he believes he catches it in
        // the air) hears it and peels off to back up the catch (agent-models §5.7). He stops
        // `PEEL_BACKUP_M` short of where his own read has it coming down, on the side he is
        // coming from; `assign_backups` gives him the job unless a cover job took him first.
        // Each infielder in this call is judged afresh: with no sure call, nobody is called off;
        // a man in it who no longer believes he catches it keeps the spot he had.
        let sure = calls.iter().any(|c| c.0 == chosen && c.1.0 >= 2);
        self.f[chosen].peel = None;
        for c in &calls {
            let i = c.0;
            if i == chosen || i >= 6 {
                continue;
            }
            if !sure {
                self.f[i].peel = None;
            } else if c.1.0 >= 1 {
                let fd = &self.f[i];
                let at = Vec3::new(own[i].x, own[i].y, 0.0);
                let back = Vec3::new(fd.pos.x - at.x, fd.pos.y - at.y, 0.0);
                let dir = if back.norm() > 1e-6 { back.normalized() } else { Vec3::new(-at.x, -at.y, 0.0).normalized() };
                self.f[i].peel = Some(at + dir * PEEL_BACKUP_M);
            }
        }
        for i in [3usize, 4, 5] {
            let fd = &self.f[i];
            let grounder = fd.belief.as_ref().and_then(|p| p.first_bounce()).map(|b| m::hypot(b.1, b.2) < 25.0 && b.0 - t_c < GROUNDER_BOUNCE_S).unwrap_or(false);
            // Only a fielder who read the ball for this call (Codex momentum review #5: another's
            // saved target may be a base from an earlier job).
            let fresh_read = calls.iter().any(|c| c.0 == i);
            if i != self.primary && fresh_read && grounder && fd.t_int.is_finite() && m::hypot(own[i].x, own[i].y) < INFIELD_R {
                self.f[i].role = Role::Chase;
                self.f[i].target = own[i];
                broke = true;
            }
        }
        if broke {
            self.cover_roles_after_chasers();
        }
        self.assign_relay();
        self.assign_backups();
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
            let spare = ti - (start + run_time_to(&fd, pt, REACH * 0.7));
            if spare >= CALL_SLACK_S {
                // Where he believes the ball is now, and how long he has watched it.
                let view = (self.t - self.inp.t_contact - fd.a.reaction).max(0.0);
                let sigma = landing_depth_sigma(self.inp.env, self.inp.field, pred.at(self.t), self.t, fd.pos, fd.a.read_sigma0, fd.a.tau_read, view, self.off_bat());
                let doubt = CALL_SURE_SIGMAS * sigma / fd.a.speed;
                if spare < CALL_SLACK_S + doubt {
                    return (1, spare);
                }
                return (if i >= 6 { 3 } else { 2 }, spare);
            }
        }
        (0, -(ti + bias))
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
    /// thrown (Codex peel-off review #2): he is free again, and a loose ball is re-called at
    /// once (after the chaser's replan, as the once-a-second re-call does), so the man who
    /// was backing it up can field it.
    fn expire_peels(&mut self) {
        if self.ball_still_up() || !self.f.iter().any(|fd| fd.peel.is_some()) {
            return;
        }
        for fd in self.f.iter_mut() {
            if fd.peel.take().is_some() && fd.role == Role::Backup && fd.backs.is_none() {
                go_idle(fd);
            }
        }
        if matches!(self.ball, Ball::Free { .. }) {
            self.replan_primary();
            self.recall();
        }
    }

    /// The chaser's replan from a fresh read.
    fn replan_primary(&mut self) {
        let pi = self.primary;
        let p = self.predict(pi);
        let (ti, pt) = self.intercept(pi, &p);
        self.f[pi].t_int = ti;
        self.f[pi].target = pt;
        self.f[pi].belief = Some(p);
        self.f[pi].next_plan = self.t + REPLAN_S;
    }

    /// Where the ball is being fielded, as the defense sees it: the holder, or the chaser's
    /// intercept point while it is loose.
    fn fielding_point(&self) -> Vec3 {
        match self.ball {
            Ball::Held { by } => self.f[by].pos,
            _ => self.f[self.primary].target,
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
            _ => self.primary,
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
        let on = |b: u8| self.r.iter().any(|r| r.start == b && !r.out && !r.scored);
        let mut b = if on(2) || on(3) { 4 } else if on(1) || deep { 3 } else { 2 };
        for r in self.r.iter().filter(|r| !r.out && !r.scored) {
            if let Some(target) = visible_target(r, self.f[fielder].a.throw_choice) {
                b = b.max(target);
            }
        }
        let who = if deep {
            if from.x <= 10.0 { 5 } else { 3 }
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
        // before the man chasing it (a grounder he can cut off before it reaches the
        // outfield): the relay job is for a ball that is past him.
        if matches!(self.ball, Ball::Free { .. }) && self.f[who].t_int <= self.f[self.primary].t_int {
            return None;
        }
        Some((who, b))
    }

    /// The cutoff man's spot: on the line from the ball to the base.
    fn relay_spot(from: Vec3, b: u8) -> Vec3 {
        let base = base_pos(b);
        let line = Vec3::new(from.x - base.x, from.y - base.y, 0.0);
        let d = line.norm();
        let s = (d - RELAY_OF_THROW_M).max(CUTOFF_M).min(0.5 * d);
        base + line.normalized() * s
    }

    /// Give the cutoff or relay job to its man (taking him off any base he was covering,
    /// which someone else then covers).
    fn assign_relay(&mut self) {
        let Some((i, b)) = self.relay_job(self.fielding_point()) else { return };
        let was_cover = matches!(self.f[i].role, Role::Cover(_));
        self.f[i].role = Role::Relay(b);
        self.f[i].backs = None;
        self.f[i].target = Self::relay_spot(self.fielding_point(), b);
        if was_cover {
            self.cover_roles_after_chasers();
        }
    }

    /// Each tick the cutoff man keeps to the line as the fielding point and the runners'
    /// intentions change (not while a throw is coming to him).
    fn update_relay(&mut self) {
        let Some(i) = (0..9).find(|&i| matches!(self.f[i].role, Role::Relay(_))) else {
            // The job may begin now (the ball has come down in the outfield).
            if !matches!(self.ball, Ball::Thrown { .. }) {
                self.assign_relay();
            }
            return;
        };
        match self.ball {
            Ball::Thrown { receiver, .. } if receiver == i => return,
            // A throw to someone else has gone by him (see `holder_act`).
            Ball::Thrown { .. } => {
                go_idle(&mut self.f[i]);
                return;
            }
            _ => {}
        }
        let from = self.fielding_point();
        let b = match self.relay_job(from) {
            Some((who, b)) if who == i => b,
            // His job is over (or someone else's): he goes back to covering.
            _ => {
                go_idle(&mut self.f[i]);
                self.cover_roles_after_chasers();
                if matches!(self.ball, Ball::Free { .. }) {
                    self.assign_relay();
                }
                return;
            }
        };
        self.f[i].role = Role::Relay(b);
        self.f[i].target = Self::relay_spot(from, b);
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
                // Nobody on: the pitcher backs up third, the base a batter-runner stretching
                // a hit in the gap or down a line goes for (owner, 2026-11-13; agent-models
                // §5.7; pitcher-outfield diagnosis Part B). He is no longer a fallback for
                // second, which left him without a job behind the far outfielder.
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
            self.f[i].peel = None;
        }
    }

    /// Re-assign base coverage among fielders who are not chasing.
    fn cover_roles_after_chasers(&mut self) {
        let busy = |f: &Vec<Fielder>, i: usize| matches!(f[i].role, Role::Chase | Role::Relay(_));
        let left = self.f[self.primary].target.x < 0.0;
        let mut taken: Vec<usize> = (0..9).filter(|&i| busy(&self.f, i)).collect();
        // The first baseman out as the cutoff to home: on a ball to the right side the second
        // baseman takes first and the pitcher stays free to back up home; to the left side
        // the second baseman is needed at second, so the pitcher takes first.
        let first = match (matches!(self.f[2].role, Role::Relay(_)), left) {
            (true, false) => vec![3, 0],
            (true, true) => vec![0, 3],
            _ => vec![2, 0, 3],
        };
        let order: [(u8, Vec<usize>); 4] = [
            (1, first),
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
        for i in 0..9 {
            if i == pi {
                self.f[i].role = Role::Chase;
            } else if !taken.contains(&i) {
                // Fielders with no job yet; `assign_backups` gives most of them one. Until
                // then he pulls up where braking stops him: `assign_among` left his own
                // intercept of the ball in `target`, and a man with no job does not chase it
                // (agent-models §5.7; docs/calibration/2026-11-12-pitcher-outfield-diagnosis.md).
                // The ground-ball break reads its own saved copy of the intercepts.
                go_idle(&mut self.f[i]);
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
            // He sees the wall: never coming at it faster than he can stop before it at the
            // braking limit (Gemini play-area review P2-3: chasers ran into it at full speed).
            let w = field.body_clearance(fd.pos.x, fd.pos.y, BODY_R);
            let into = -fd.vel.dot(w.inward);
            let allowed = m::sqrt(2.0 * BRAKE * w.dist.max(0.0));
            if into > allowed {
                fd.vel = fd.vel + w.inward * (into - allowed);
            }
            // Nobody runs through a wall (physics-spec §9.5): he stops at it, losing the speed
            // he had into it.
            let w = field.body_clearance(fd.pos.x, fd.pos.y, BODY_R);
            if w.dist < 0.0 {
                fd.pos = field.keep_inside(fd.pos, BODY_R);
                let vn = fd.vel.dot(w.inward);
                if vn < 0.0 {
                    fd.vel = fd.vel - w.inward * vn;
                }
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
        let (b, bv) = self.ball_at(self.t);
        let receiver = match self.ball {
            Ball::Thrown { receiver, .. } => Some(receiver),
            Ball::Free { .. } => None,
            Ball::Held { .. } => return,
        };
        // Catchable height where the ball is: a fielder at the wall can leap.
        let field = self.inp.field;
        let zmax_at = |p: Vec3| if field.body_clearance(p.x, p.y, 0.0).dist < 3.0 { WALL_CATCH_MAX_Z } else { CATCH_MAX_Z };
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
                fd.set_base = fd.a.transfer;
                fd.catch_vel = fd.vel;
                fd.face = Some(toward(fd.vel).or(toward(-bv)).unwrap_or(Vec3::new(-fd.pos.x, -fd.pos.y, 0.0)));
                // Having caught it, he plants: his run dies out where braking stops him, not
                // back at the spot he planned (which a running catch has carried him past).
                fd.target = stop_point(fd);
                fd.carry_to = None;
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
                    // (Taking it on the ground from a dive, he is up first.)
                    fd.ready_at = up_at + RELAY_TRANSFER_FRAC * fd.a.transfer;
                    fd.set_base = RELAY_TRANSFER_FRAC * fd.a.transfer;
                    fd.face = None;
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
        for fd in self.f.iter_mut() {
            fd.peel = None;
            fd.must_dive_since = f64::INFINITY;
            fd.last_dive_read = f64::NEG_INFINITY;
            fd.will_dive.set(false);
        }
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
    fn receiver_read(&self, i: usize, aim: Vec3, err: [f64; 4], t_release: f64) -> (Vec3, Vec3) {
        let fd = &self.f[i];
        let (p, v) = self.ball_at(self.t);
        let view = (self.t - t_release - fd.a.reaction).max(0.0);
        let shrink = 1.0 / m::sqrt(1.0 + view / fd.a.tau_read);
        let dist = m::hypot(p.x - fd.pos.x, p.y - fd.pos.y).max(1.0);
        let sp = fd.a.read_sigma0 * (dist / READ_REF_DIST) * shrink;
        let los = Vec3::new(p.x - fd.pos.x, p.y - fd.pos.y, 0.0).normalized();
        let lat = Vec3::new(-los.y, los.x, 0.0);
        let pe = p + los * (err[0] * sp * DEPTH_STRETCH) + lat * (err[1] * sp) + Vec3::new(0.0, 0.0, err[1] * sp * 0.5);
        let sv = READ_VEL_FRAC * v.norm() * shrink;
        let ve = v + los * (err[2] * sv * DEPTH_STRETCH * 0.5) + lat * (err[3] * sv) + Vec3::new(0.0, 0.0, err[3] * sv * 0.5);
        // He assumes a throw's typical backspin.
        let w = Vec3::new(ve.x, ve.y, 0.0).normalized().cross(Vec3::Z) * bb_math::units::rpm_to_radps(THROW_SPIN_RPM);
        let r = Vec3::new(pe.x, pe.y, pe.z.max(self.inp.env.ball.radius));
        // Flown until a second past the spot he was thrown to (he cares about nothing later).
        let vh = m::hypot(ve.x, ve.y).max(5.0);
        let pred = fly_throw(self.inp.env, BallState { r, v: ve, w }, self.t, m::hypot(aim.x - r.x, aim.y - r.y) / vh + 1.0);
        let raw = pred
            .knots
            .iter()
            .filter(|k| k.s.r.z >= THROW_SCOOP_Z && k.s.r.z <= CATCH_MAX_Z)
            .min_by(|a, b| m::hypot(a.s.r.x - aim.x, a.s.r.y - aim.y).total_cmp(&m::hypot(b.s.r.x - aim.x, b.s.r.y - aim.y)))
            .map(|k| k.s.r)
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

    fn update_thrown(&mut self) {
        // The receiver watches the throw and goes to where he believes he can take it,
        // re-reading it as it comes (he starts on the spot it was thrown to).
        if let Ball::Thrown { path, aim, read_err, next_read, receiver, .. } = &self.ball {
            if self.t >= *next_read {
                let (aim, err, i, t0) = (*aim, *read_err, *receiver, path.first().t);
                let (at, raw) = self.receiver_read(i, aim, err, t0);
                if let Ball::Thrown { catch_at, read_at, next_read, .. } = &mut self.ball {
                    *catch_at = at;
                    *read_at = raw;
                    *next_read = self.t + RECEIVE_REPLAN_S;
                }
            }
        }
        let Ball::Thrown { path, catch_at, read_at, receiver, to_base, relay, .. } = &self.ball else { return };
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
        if self.t >= path.last().t {
            let (s, t_end, at_wall) = (path.last().s, path.last().t, path.stop == Some(1));
            if at_wall {
                // The throw reached a wall in the air (physics-spec §9.5): off it, or over it
                // and out of play.
                match wall_contact(self.inp.field, s, self.inp.env.ball.radius) {
                    (WallContact::Out, _) => {
                        // A throw over the wall is dead as it crosses: nobody catches a throw
                        // in the seats (Gemini play-area review P2-2: it used to stay live
                        // until it came down there). The ball flies on into the seats for the
                        // picture only.
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

    /// Runners' believed time for the ball to reach base `b` (they watch the ball and the
    /// fielders; agent-models §6.3).
    fn ball_time_to(&self, k: usize, b: u8, read_sub: u32) -> f64 {
        let target = base_pos(b);
        match &self.ball {
            Ball::Held { by } => {
                // The runner sees where the ball is, when it was caught, and who has it: his
                // read of that fielder is the position's grade-50 transfer and arm, the arm
                // off by his own read error (agent-models §6.4; one draw per runner, fielder
                // and play).
                let (p, _) = self.ball_at(self.t);
                let mean = FielderAttrs::average(*by);
                let err = rng(self.inp.seed, self.inp.play, self.r[k].id, Purpose::RunnerPerception, ARM_READ_SUB + *by as u32).truncated_normal(4.0);
                let ready = (self.secured_at + mean.transfer).max(self.t);
                ready + read_throw_time(mean.arm + ARM_READ_SIGMA_MPS * err, m::hypot(target.x - p.x, target.y - p.y))
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
                if dest == b { arrive } else { arrive + 0.6 + typical_throw_time((target - base_pos(dest)).norm()) }
            }
            Ball::Free { .. } => {
                let t0 = prof::enabled().then(std::time::Instant::now);
                let pred = self.runner_predict(k, read_sub);
                let (tf, at) = self.believed_field_time(&pred);
                if let Some(t0) = t0 {
                    prof::add(&prof::RUNNER, t0.elapsed().as_nanos() as u64);
                }
                tf + TYPICAL_TRANSFER + typical_throw_time(m::hypot(target.x - at.x, target.y - at.y))
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

    /// How much the holder values an out on runner `k` at base `goal`. With fewer than two out
    /// the lead runner is worth more. With two out every out ends the inning, so the sure out
    /// is the play, and a force is preferred to a tag (it needs no tag, and a run cannot score
    /// on it; OBR 5.08(a)). It is a force only while he is still short of the base: a runner
    /// coming back to a base he has already reached must be tagged (Codex dive review #1).
    fn out_weight(&self, k: usize, goal: u8) -> f64 {
        let r = &self.r[k];
        if self.st.outs >= 2 {
            if self.forced(k) && goal == r.start + 1 && r.s < goal as f64 * BASE_L { 1.0 } else { TWO_OUT_TAG_W }
        } else {
            [0.9, 1.0, 1.1, 1.3][(goal as usize).saturating_sub(1).min(3)]
        }
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
        let d_run = (next as f64 * BASE_L - r.s).max(0.0);
        let t_run = match r.set_start {
            // Going from a tag-up: the same start his legs will run (§6.4).
            Some(leave) => tag_up_time(self.inp.tag_up, (leave - self.t).max(0.0), d_run, r.vmax),
            None => run_time(d_run, r.vmax, RUNNER_TAU) * if r.v > 1.0 { 0.75 } else { 1.0 },
        };
        // Unforced, he must beat the tag, not only the ball: his belief of the time it takes.
        let tag_s = if forced { 0.0 } else { self.inp.tag_up.typical_tag_s };
        let t_ball = self.ball_time_to(k, next, 2 * n + 1) + tag_s;
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
            // A runner who has touched home is done, unless the catch is still to be seen: his
            // run is only pending until then (day-fixes review #1).
            if self.r[k].out || self.done || (self.r[k].scored && (self.r[k].catch_seen || !self.caught())) {
                continue;
            }
            let start_t = if self.r[k].start == 0 { self.inp.t_contact + BATTER_START } else { self.inp.t_contact };
            if self.t < start_t {
                continue;
            }
            // The catch, seen once: a runner off his base at that moment owes a retouch; one
            // standing on it (tagging up or not) owes nothing and may leave for good. That
            // includes a runner who already crossed the plate: he goes back from home, is a
            // live runner again (tags and throws can retire him), and must touch home again.
            if self.caught() && !self.r[k].catch_seen {
                self.r[k].catch_seen = true;
                if self.r[k].start > 0 && self.r[k].s > self.r[k].start as f64 * BASE_L + 1e-6 {
                    self.r[k].scored = false;
                    self.r[k].tag_up = false;
                    self.r[k].hold_s = None;
                    self.r[k].retouch = true;
                    self.r[k].waiting = false;
                    self.r[k].goal = self.r[k].start;
                }
            }
            if self.r[k].scored {
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
                if self.caught() {
                    // (A runner still off his base at the catch was sent back above.)
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
            let (now, tag) = (self.t, self.inp.tag_up);
            let r = &mut self.r[k];
            let dir = (goal_s - r.s).signum();
            if (goal_s - r.s).abs() < 1e-6 {
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
            r.direction = dir;
            if r.v == 0.0 {
                r.moving_since = now;
            }
            let tau = if r.set_start.is_some() { tag.tau_s } else { RUNNER_TAU };
            r.v += (r.vmax - r.v) * (DT / tau);
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
                    // A batter-runner stopping at first runs through it (agent-models §6.2).
                    let r = &mut self.r[k];
                    if r.start == 0 && b == 1 && dir > 0.0 && r.goal == 1 {
                        r.overrun = OverrunPhase::Out;
                        r.over_m = 0.0;
                    }
                }
            }
            // End the physical stop on arrival, before the next holder reads its
            // speed. Protected overruns retain the crossing speed for braking.
            let r = &mut self.r[k];
            let stop_s = r.hold_s.unwrap_or(r.goal as f64 * BASE_L);
            if r.overrun == OverrunPhase::None && (stop_s - r.s).abs() < 1e-6 {
                r.v = 0.0;
                r.set_start = None;
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
                r.v += (OVERRUN_RETURN_MPS - r.v) * (DT / RUNNER_TAU);
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
        let fd = self.f[by].clone();
        // Tag any runner within reach who is off a base.
        for k in 0..self.r.len() {
            let r = &self.r[k];
            // A batter-runner coming straight back from his overrun of first is no play: he
            // cannot be tagged out (OBR 5.09(b)(4)), and the fielders see he made no turn.
            if r.out || r.scored || r.overrun != OverrunPhase::None {
                continue;
            }
            let on_base = (r.s / BASE_L - (r.s / BASE_L).round()).abs() * BASE_L < 0.3;
            // A tag takes him `tag_apply_s` from securing the ball (agent-models §6.4).
            if !on_base && self.t >= self.secured_at + fd.a.tag_apply_s && (self.runner_pos(r) - fd.pos).norm() < fd.a.throw_choice.tag_reach_m {
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
        let best = options.iter().enumerate().filter(|(_, o)| o.p_out > 0.3)
            .fold(None, |best: Option<usize>, (i, o)| if best.is_none_or(|j| o.value > options[j].value) { Some(i) } else { best });
        if debug_throw() {
            eprintln!("t {:.2} holder {} best {:?}; options {:?}", self.t, by, best, options);
        }
        let (b, through, margins) = if let Some(i) = best {
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
                        self.out.scorer_carries.push(ScorerCarryFact { fielder: fd.id, t: self.t, base: o.base, batter_goal });
                    }
                }
                return;
            }
            (o.base, o.through, Some((o.runner_at - o.ball_at, o.runner_at - o.set_ball_at)))
        } else if m::hypot(fd.pos.x, fd.pos.y) >= INFIELD_R {
            // No play: get the ball back to second base, the quicker way.
            (2, self.throw_race(by, 2).through, None)
        } else {
            self.abandon_carry(by);
            return;
        };
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
            self.throw_to(by, leg, b, i, true);
            if let Some(fact) = self.out.scorer_throws.last_mut() { fact.attempted_out = margins.is_some(); }
            return;
        }
        let receiver = race.receiver;
        // Preserve the existing release gate for a receiver still covering.
        if race.direct < race.receiver_at - 0.05 {
            self.f[receiver].target = tgt;
            return;
        }
        self.trace_choice(by, action, b, false, best, &options);
        self.throw_to(by, tgt, b, receiver, false);
        if let Some(fact) = self.out.scorer_throws.last_mut() { fact.attempted_out = margins.is_some(); }
        self.f[receiver].target = tgt;
        // A throw that goes past the cutoff man ends his job: he pulls up, free to back it up.
        for i in 0..9 {
            if matches!(self.f[i].role, Role::Relay(_)) {
                go_idle(&mut self.f[i]);
            }
        }
        self.back_up_throw(fd.pos, b);
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
        SetUp::new(fd.set_base, turn, throwing::footwork_s(fd.catch_vel, to, fd.took_diving))
    }

    /// Fielder `by` throws to ground point `tgt` for `receiver` (the ball is headed for base
    /// `b`, through a relay man when `relay`). Let go before he is set, the throw is slower
    /// and wilder (agent-models §5.5).
    fn throw_to(&mut self, by: usize, tgt: Vec3, b: u8, receiver: usize, relay: bool) {
        let fd = self.f[by].clone();
        let mut r = rng(self.inp.seed, self.inp.play, fd.id, Purpose::ThrowExecution, fd.throws);
        self.f[by].throws += 1;
        let su = self.set_up_for(by, tgt);
        let hurry = su.hurry(self.t - fd.up_at);
        // Aim with the thrower's own throw physics (drag and backspin, agent-models §5.5);
        // a throw beyond his reach on the fly goes at the steepest useful angle and bounces.
        let r0 = throw_release_point(fd.pos, tgt, fd.a.throws);
        let v_aim = THROW_EFFORT * fd.a.arm * su.speed_mult(hurry);
        let d = m::hypot(tgt.x - r0.x, tgt.y - r0.y);
        let elev = solve_throw(self.inp.env, THROW_RELEASE_Z, THROW_CATCH_Z, d, v_aim).map(|s| s.elev).unwrap_or(THROW_MAX_ELEV);
        // Execution noise: speed 3%, then direction (the draw order of the vacuum model).
        let ex = execute_throw(v_aim, &ThrowNoise::new(fd.a.arm_sigma * su.sigma_mult(hurry)), &mut r);
        let h = Vec3::new(tgt.x - r0.x, tgt.y - r0.y, 0.0).normalized().rotate(Vec3::Z, ex.yaw_err);
        self.launch_throw(throw_state(r0, h, ex.speed, elev), tgt, b, receiver, relay);
        self.out.released.push((self.t, fd.id));
        let batter_goal = self.r.iter().find(|r| r.start == 0).map(|r| r.goal).unwrap_or(0);
        self.out.scorer_throws.push(ScorerThrowFact { thrower: fd.id, t: self.t, origin: r0, target: tgt, base: b, attempted_out: false, batter_goal });
        // The rules engine needs the release for a throw that goes out of play (rules-spec §7.2);
        // fielders 0–5 are P, C, 1B, 2B, 3B, SS.
        let t_rel = self.t.max(self.last_fact_t);
        self.emit(Input::BallReleasedFromGlove { t: t_rel, fielder: fd.id, kind: ReleaseKind::Throw, infielder: by < 6 });
        self.out.hurry.push((hurry, su.off_balance));
    }

    /// Who takes a throw at base `b`: its cover man, else the nearest other fielder who is
    /// not the cutoff man (he keeps to his line).
    fn receiver_for(&self, by: usize, b: u8) -> usize {
        let tgt = base_pos(b);
        (0..9).find(|&i| i != by && self.f[i].role == Role::Cover(b)).unwrap_or_else(|| {
            (0..9)
                .filter(|&i| i != by && !matches!(self.f[i].role, Role::Relay(_)))
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
        let calm = self.r.iter().all(|r| r.out || r.scored || ((r.s - r.goal as f64 * BASE_L).abs() < 1e-6 && r.overrun == OverrunPhase::None));
        let in_infield = m::hypot(fd.pos.x, fd.pos.y) < INFIELD_R;
        calm && in_infield && self.t >= fd.ready_at
    }

    /// Where a runner is. Runners move along the base path (`path_pos`), except that the
    /// batter starts in his box, off the path: his offset from the plate shrinks linearly to
    /// nothing over his first `BOX_EXIT_M` of running. Tags, his own view of the ball, and the
    /// recorded track all use this one position. The distance he is charged for is still the
    /// base-path distance, which is within a few centimetres of this route's length. On his
    /// overrun of first he is past the bag on the first-base line.
    fn runner_pos(&self, r: &Runner) -> Vec3 {
        if r.overrun != OverrunPhase::None {
            // Past first, straight down the first-base line.
            return base_pos(1) * (1.0 + r.over_m / BASE_L);
        }
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
                vel: Vec3::ZERO,
                jump: jump_at_contact(inp, i, home[i]),
                role: Role::Idle,
                target: home[i],
                next_plan: 0.0,
                err: [r.truncated_normal(4.0), r.truncated_normal(4.0), r.truncated_normal(4.0), r.truncated_normal(4.0)],
                attempts: 0,
                blocked_until: 0.0,
                ready_at: 0.0,
                dove: false,
                lunge: None,
                belief: None,
                t_int: f64::INFINITY,
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
                backs: None,
                peel: None,
                carry_to: None,
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
            direction: 1.0,
            vmax: x.speed,
            goal: x.base,
            out: false,
            scored: false,
            tag_up: false,
            set_start: None,
            moving_since: 0.0,
            last_base: x.base,
            retouch: false,
            waiting: false,
            // Until his first read of the ball he stays where his lead put him.
            hold_s: if x.base == 0 { None } else { Some(lead_s(x.base)) },
            last_timing_draw: 0.0,
            catch_seen: false,
            decisions: 0,
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
        f,
        r,
        primary: 0,
        out: LiveOutput { scorer_carries: Vec::new(), first_handling: None, scorer_throws: Vec::new(), facts: Vec::new(), events: Vec::new(), path, ball_track: Vec::new(), actors, secured: Vec::new(), released: Vec::new(), hurry: Vec::new(), dives: Vec::new(), calls: Vec::new() },
        bounce_k: 1000,
        done: false,
        last_fact_t: t_c,
        secured_at: f64::INFINITY,
        catch_made: false,
        low_catch: false,
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
    let mut tick: u64 = 0;
    while !sim.done {
        if sim.t - t_c > PLAY_LIMIT_S {
            if std::env::var("BB_DEBUG_LIVE").is_ok() {
                eprintln!("timeout play {}: ball {:?}", play, match &sim.ball { Ball::Free { .. } => "free".to_string(), Ball::Held { by } => format!("held by {} at {:?} ready {}", by, sim.f[*by].pos, sim.f[*by].ready_at), Ball::Thrown { to_base, receiver, .. } => format!("thrown to {to_base} recv {receiver}") });
                let (bp, bv) = sim.ball_at(sim.t);
                eprintln!("  ball at ({:.2},{:.2},{:.2}) v ({:.2},{:.2},{:.2}); primary {} blocked_until {:.2} dove {} next_plan {:.2}", bp.x, bp.y, bp.z, bv.x, bv.y, bv.z, sim.primary, sim.f[sim.primary].blocked_until, sim.f[sim.primary].dove, sim.f[sim.primary].next_plan);
                if let Ball::Thrown { path, .. } = &sim.ball { eprintln!("  throw from {:?} at {:.2}, lands {:.2}; now {:.2}", path.first().s.r, path.first().t, path.last().t, sim.t); }
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
            sim.replan_primary();
            let pi = sim.primary;
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
        if assigned {
            sim.expire_peels();
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
        sim.update_relay();
        sim.move_fielders();
        sim.move_runners();
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
        prof::TICKS.with(|c| c.set(c.get() + 1));
        tick += 1;
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
            bats: bb_observe::Hand::Right,
            pitch_x: None,
            bat_yaw: None,
            swing_lead: None,
            seed: 11,
            play: 3,
            record: false,
            tag_up: Default::default(),
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

    /// The center fielder on the ground from a dive at time 1.0 (get-up ends at 2.5), and a
    /// ball rolling across in front of him at 6 m/s, passing `pass_m` from him at time 2.0.
    fn down_after_dive(pass_m: f64) -> (Vec<Input>, f64) {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
        let mut inp = input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let at = |th: f64, inset: f64| {
            let r = fx.field.fence_at(th) - inset;
            Vec3::new(r * m::sin(th), r * m::cos(th), 0.0)
        };
        let (fielder, ball) = (at(core::f64::consts::FRAC_PI_4 - 0.006, 0.001), at(core::f64::consts::FRAC_PI_4 + 0.018, 0.001));
        sim.f[CF].pos = fielder;
        loose_ball(&mut sim, 2.0, ball + Vec3::new(0.0, 0.0, 0.05), (ball - fielder).normalized() * 4.0);
        sim.f[CF].role = Role::Chase;
        // He has long known that only a dive gets it (and is not asked again).
        sim.f[CF].must_dive_since = 0.0;
        sim.f[CF].last_dive_read = f64::INFINITY;
        for _ in 0..40 {
            world_tick(&mut sim);
            let p = sim.f[CF].pos;
            assert!(m::hypot(p.x, p.y) <= fx.field.fence_at(m::atan2(p.x, p.y)) + 1e-9, "at {:.2} s the body is {:.3} m past the wall", sim.t, m::hypot(p.x, p.y) - fx.field.fence_at(m::atan2(p.x, p.y)));
        }
        assert!(!sim.out.dives.is_empty(), "he never dove (gap {:.2} m)", (ball - fielder).norm());
    }

        // Codex dive review #2: a low ball in reach of a fielder who is down is taken now, not
    // deferred to the next tick, when it will have risen past what he can reach lying down.
    #[test]
    fn a_fielder_down_from_a_dive_takes_a_rising_ball_while_it_is_low() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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

    /// A runner standing on third, tagging up, with `tag` for his start; the ball caught by
    /// the center fielder `cf_y` m out at 5.0 s. Runs the runners only.
    fn tag_up_from_third(tag: bb_agents::running::TagUpParams, cf_y: f64, v_before: f64) -> (Sim<'static>, Vec<f64>) {
        const R: AgentId = 0x0007;
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&[(R, 3)])));
        let mut inp = input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R, base: 3, speed: 8.2 }]);
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
        let tag = bb_agents::running::TagUpParams { react_s: 0.2, tau_s: 0.6, typical_tag_s: 0.0 };
        // A throw from 140 m needs a relay: the run is clearly on, whatever his timing draw.
        let (sim, _) = tag_up_from_third(tag, 140.0, 0.0);
        let t = home_touch(&sim).expect("he went");
        assert!((t - (5.0 + 0.2 + 3.94453)).abs() < 2.0 * DT, "home at {t:.4}");
        assert!((tag_up_time(tag, 0.2, BASE_L, 8.2) - (0.2 + 3.94453)).abs() < 1e-3);
    }

    // §6.4: standing on the bag he has no speed, whatever the state said before (before A4 a
    // tagging runner kept 6.19 m/s from coming back to third). His first moving tick starts
    // from rest: v = 8.2 · DT / τ.
    #[test]
    fn no_speed_carries_over_into_a_tag_up() {
        let tag = bb_agents::running::TagUpParams::default();
        let (_, speeds) = tag_up_from_third(tag, 140.0, 6.19);
        assert!((speeds[0] - 8.2 * DT / tag.tau_s).abs() < 1e-9, "first tick {:.4} m/s", speeds[0]);
    }

    // §6.4: his decision uses his own start. A runner who would need 3 s more to get going
    // does not try it on a ball caught 95 m out; with a 0.2 s first step he does.
    #[test]
    fn the_decision_uses_the_set_start() {
        let quick = bb_agents::running::TagUpParams { react_s: 0.2, tau_s: 0.6, typical_tag_s: 0.0 };
        let slow = bb_agents::running::TagUpParams { react_s: 3.2, ..quick };
        assert!(home_touch(&tag_up_from_third(quick, 140.0, 0.0).0).is_some());
        assert!(home_touch(&tag_up_from_third(slow, 95.0, 0.0).0).is_none(), "he should hold");
    }

    /// The catcher on the plate takes the ball at 2.0 s with an unforced runner 1.0 m from it,
    /// coming in at 8 m/s; the catcher needs `tag_s` to put a tag on.
    fn tag_at_home(tag_s: f64) -> Sim<'static> {
        const R: AgentId = 0x0007;
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&[(R, 3)])));
        let mut inp = input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R, base: 3, speed: 8.2 }]);
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
            let belief = TYPICAL_TRANSFER + typical_throw_time(d);
            let run = run_time(BASE_L, 8.2, RUNNER_TAU);
            println!("{ft:4} ft {d:6.1} m  CF arm {cf:.2}: flight {flight:.2} s, +0.85 transfer {:.2} s | runner believes ball {belief:.2} s (37 m/s, relay if quicker); runner {run:.2} s", 0.85 + flight);
        }
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

    // Day-fixes review #1 (P1): a runner from third who crossed the plate before the catch
    // (the review's seed 59 play 186 shape) still owes the retouch. Before the fix he stood on
    // home for good (`scored`, never saw the catch) and the defense's appeal took the run
    // off. Now he goes back, retouches third, and touches home again; the rules engine holds
    // one pending run for him, from the second touch.
    #[test]
    fn runner_home_before_the_catch_goes_back_and_touches_again() {
        const R: AgentId = 0x0007;
        let fx = fixture(&[(R, 3)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R, base: 3, speed: 16.0 }]);
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

    // Codex momentum review #3: the planner's reach time must follow the movement step, also
    // for a fielder moving across his goal. Over a grid of distances and motions (toward,
    // away, across), the closed form [`run_time_to`] is compared with the step itself
    // (a late chaser running at a fixed spot until within the planning reach).
    #[test]
    fn reach_estimate_tracks_the_movement_step() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
                    let est = run_time_to(&sim.f[SS], sim.f[SS].target, REACH * 0.7);
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
        assert!(worst_early > -0.35 && worst_late < 0.35 && rms < 0.15, "estimate − step: worst {worst_early:+.3} / {worst_late:+.3} s, rms {rms:.3} s");
    }

    // agent-models §5.10, worked by hand. Fielders' lines from home, from the home positions:
    // 1B 38°, 2B 13°, SS −13°, 3B −36°, LF −30°, CF 0°, RF 30° of spray. Perfect reads. A
    // right-hander, pitch down the middle: the prior is his typical pull, −5°, σ 22°. A bat
    // read at yaw 60.4412° (face −29.5588°) is a cue at 5.2 + 1.36 × (−29.5588) = −35.0°, with
    // σ 18.6° (the scatter about the fit; no read error). k = 22²/(22² + 18.6²) = 0.58316, so
    // the estimate is −5 − 0.58316 × 30 = −22.495°, σ = 22 × 18.6 / √(22² + 18.6²) = 14.204°.
    // Offsets against 2σ = 28.408° (zones 20° infield, 25° outfield): 3B +13.505° → +0.4754;
    // SS −9.495° → −0.3342; LF +7.505° → +0.2642; CF −22.495° → −0.7919; 1B, 2B and RF outside
    // their zones; the pitcher and catcher never. A left-hander's mirror image (yaw −60.4412°,
    // prior +5°) gives +22.495°: 1B −0.5458, 2B +0.3342, CF +0.7919, RF −0.2642, the rest 0.
    // With no bat read, the prior alone: the shortstop, 8° off it, at 8/(2·22) = 0.1818.
    #[test]
    fn fielders_jump_toward_the_side_they_read() {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        for a in inp.fattrs.iter_mut() {
            a.antic_sigma = 0.0;
            a.swing_sigma = 0.0;
        }
        inp.pitch_x = Some(0.0);
        let home = home_positions();
        // Unit vector across fielder i's line, toward right field, from his published angle.
        let deg = [0.0, 0.0, 38.0, 13.0, -36.0, -13.0, -30.0, 0.0, 30.0];
        let toward_rf = |i: usize| {
            let phi: f64 = f64::to_radians(deg[i]);
            Vec3::new(phi.cos(), -phi.sin(), 0.0)
        };
        let jumps = |inp: &LiveInput| -> Vec<f64> { (0..9).map(|i| jump_at_contact(inp, i, home[i]).dot(toward_rf(i)) / inp.fattrs[i].speed).collect() };
        inp.bat_yaw = Some(60.4412_f64.to_radians());
        let rhb = jumps(&inp);
        let expect_r = [0.0, 0.0, 0.0, 0.0, 0.4754, -0.3342, 0.2642, -0.7919, 0.0];
        inp.bats = bb_observe::Hand::Left;
        inp.bat_yaw = Some((-60.4412_f64).to_radians());
        let lhb = jumps(&inp);
        let expect_l = [0.0, 0.0, -0.5458, 0.3342, 0.0, 0.0, 0.0, 0.7919, -0.2642];
        for i in 0..9 {
            assert!((rhb[i] - expect_r[i]).abs() < 1e-3, "right-hander, fielder {i}: {:.4}", rhb[i]);
            assert!((lhb[i] - expect_l[i]).abs() < 1e-3, "left-hander, fielder {i}: {:.4}", lhb[i]);
        }
        inp.bats = bb_observe::Hand::Right;
        inp.bat_yaw = None;
        let blind = jumps(&inp);
        assert!((blind[5] - 8.0 / 44.0).abs() < 1e-3, "shortstop on the prior alone: {:.4}", blind[5]);
    }

    // Codex anticipation review #1: before his reaction time a fielder reads the bat, not the
    // ball. Two contacts with the same bat angle but different outgoing directions (the
    // review's spin probe: +4.07° and −3.24° of spray from identical bat kinematics) must give
    // every fielder the same jump. Reading the ball's launch direction, as the first version
    // did, fails this.
    #[test]
    fn the_jump_reads_the_bat_not_the_ball() {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
    // lead read perfectly: 0.180123 s says face = 161.93° − 1063.1 × 0.180123 = −29.5588°, with
    // the fit's 3.6° of scatter; that face is a cue at −35.0° (as there), σ = √((1.36 × 3.6)² +
    // 18.6²) = 19.2336°. Against the −5° prior (σ 22°): k = 0.56678, estimate −22.0037°, σ
    // 14.4801°, 2σ = 28.9602°: 3B +0.4833, SS −0.3109, LF +0.2761, CF −0.7598, the rest 0. The
    // left-hander's mirror image (the lead is the same for either side; prior +5°): 1B −0.5524,
    // 2B +0.3109, CF +0.7598, RF −0.2761. With the bat-angle read too, at the same face and σ
    // 10°, the two face reads fuse to σ = 10 × 3.6 / √(10² + 3.6²) = 3.3872°: estimate −22.0586°,
    // σ 14.4495°: 3B +0.4824, SS −0.3135, LF +0.2748, CF −0.7633. One read of the face, not two
    // cues: had the bat read been fused as a second spray cue the estimate would be −25.7°.
    #[test]
    fn the_swing_lead_moves_the_jump_toward_the_side_it_reads() {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        for a in inp.fattrs.iter_mut() {
            a.antic_sigma = 0.0;
            a.lead_sigma = 0.0;
            a.swing_sigma = 10.0_f64.to_radians();
        }
        inp.pitch_x = Some(0.0);
        inp.swing_lead = Some(0.180123);
        let home = home_positions();
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
        check(&jumps(&inp), [0.0, 0.0, 0.0, 0.0, 0.4833, -0.3109, 0.2761, -0.7598, 0.0], "right-hander, lead alone");
        inp.bats = bb_observe::Hand::Left;
        check(&jumps(&inp), [0.0, 0.0, -0.5524, 0.3109, 0.0, 0.0, 0.0, 0.7598, -0.2761], "left-hander, lead alone");
        // The belief side directly: a bat read that landed exactly on 60.4412°, believed to
        // carry 10° of error (through `jump_at_contact` the read itself would be scattered).
        let reads = JumpReads { pitch_x: 0.0, bat_yaw: Some(60.4412_f64.to_radians()), lead: Some(0.180123), bats: bb_observe::Hand::Right };
        let both: Vec<f64> = (0..9).map(|i| jump_from_reads(&reads, &inp.fattrs[i], i, home[i]).dot(toward_rf(i)) / inp.fattrs[i].speed).collect();
        check(&both, [0.0, 0.0, 0.0, 0.0, 0.4824, -0.3135, 0.2748, -0.7633, 0.0], "right-hander, lead and bat");
    }

    // The back-half policy, revised 2026-11-13: a read that by itself puts the ball behind the
    // batter is discarded and the fielder falls back on his other reads, instead of standing
    // still. (a) A bat read perfectly at 179.999° (barrel at the catcher) and no lead: the jump
    // is the pitch prior's alone, the shortstop at 8 / (2 × 22) = 0.1818 as in
    // `fielders_jump_toward_the_side_they_read`. (b) A wild lead read (0.40 s: face −263°)
    // beside a good bat read: the bat read alone decides (−0.3342 for the shortstop, from that
    // test). (c) The good lead read beside the bad bat read: the lead alone decides (−0.3109,
    // from `the_swing_lead_moves_the_jump_toward_the_side_it_reads`).
    #[test]
    fn an_implausible_read_is_discarded_for_the_other_reads() {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        for a in inp.fattrs.iter_mut() {
            a.antic_sigma = 0.0;
            a.swing_sigma = 0.0;
            a.lead_sigma = 0.0;
        }
        inp.pitch_x = Some(0.0);
        let home = home_positions();
        let toward_rf = Vec3::new(f64::to_radians(-13.0).cos(), -f64::to_radians(-13.0).sin(), 0.0);
        let ss = |inp: &LiveInput| jump_at_contact(inp, SS, home[SS]).dot(toward_rf) / inp.fattrs[SS].speed;
        inp.bat_yaw = None;
        let prior_only = ss(&inp);
        assert!((prior_only - 8.0 / 44.0).abs() < 1e-3, "prior alone: {prior_only:.4}");
        inp.bat_yaw = Some(179.999_f64.to_radians());
        assert_eq!(ss(&inp), prior_only, "a bat read behind the batter is discarded, not a reason to stand still");
        inp.bat_yaw = Some(60.4412_f64.to_radians());
        inp.swing_lead = Some(0.40);
        let got = ss(&inp);
        assert!((got + 0.3342).abs() < 1e-3, "the wild lead is discarded, the bat read decides: {got:.4}");
        inp.bat_yaw = Some(179.999_f64.to_radians());
        inp.swing_lead = Some(0.180123);
        let got = ss(&inp);
        assert!((got + 0.3109).abs() < 1e-3, "the bad bat read is discarded, the lead decides: {got:.4}");
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
    // right-hander's bat square to the pitch line (yaw 90°) says +5.2° (the fit's intercept);
    // a left-hander's mirror image (−90°) says −5.2°.
    #[test]
    fn anticipation_has_no_seam_behind_the_plate() {
        let r = f64::to_radians;
        let (e1, s1) = fuse_on_circle(r(170.0), r(22.0), r(-170.0), r(18.6));
        let (e2, s2) = fuse_on_circle(r(-5.0), r(22.0), r(15.0), r(18.6));
        assert!((e1.to_degrees() + 178.337).abs() < 1e-3, "across the seam: {:.4}°", e1.to_degrees());
        assert!((e2.to_degrees() - 6.663).abs() < 1e-3, "rotated: {:.4}°", e2.to_degrees());
        assert!((s1 - s2).abs() < 1e-12 && (s1.to_degrees() - 14.204).abs() < 1e-3);
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
        assert!((c_r.to_degrees() - 5.2).abs() < 1e-9 && (c_l.to_degrees() + 5.2).abs() < 1e-9, "{c_r} {c_l}");
        assert!(swing_spray_cue(r(179.999), bb_observe::Hand::Right, 0.0).is_none());
    }

    // Codex anticipation review #2: the defense's first call is made at the quickest
    // reaction time, while a slower fielder is still carried by his jump. What he believes he
    // has to spare on a fly must not depend on whether he works it out then or at his own
    // gate, from where he actually is: nothing new has been read in between. Centre fielder,
    // 0.6 s reaction, leaning toward left field at full speed; the call is first made at the
    // infielders' 0.25 s. The first version planned from his state at 0.25 s (0.35 s of lean
    // missing) and was optimistic by tenths of a second.
    #[test]
    fn plans_before_the_gate_count_the_rest_of_the_jump() {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
    // tick, inside `CALL_SLACK_S`: no confident call, early or at the gate (the first version
    // timed the state at 1.007 s and called it with 0.305 s to spare).
    #[test]
    fn the_projected_jump_is_timed_by_the_tick_that_produced_it() {
        let fx = fixture(&[]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
    // braking 2 m/s at 6 m/s² takes 0.33 s and loses 0.33 m, where from rest he would have
    // covered 0.54 m, so he is about 0.9 m behind at several m/s: hundredths to tenths.
    #[test]
    fn a_lean_the_right_way_gains_ground_and_the_wrong_way_costs() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
    // published laws, worked by hand: braking at 6 m/s² from 7 m/s takes 7/6 s and carries
    // him 7²/12 = 4.08 m past his start; then 9.08 m from rest (run_time). The movement step
    // integrates the same laws tick by tick.
    #[test]
    fn running_the_wrong_way_costs_the_turn() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
        let expect = 7.0 / 6.0 + run_time(5.0 + 49.0 / 12.0, a.speed, a.tau_a);
        assert!((took - expect).abs() < 0.05, "took {took:.3} s, expected {expect:.3} s");
        assert!((run_time_from(5.0, -7.0, a.speed, a.tau_a) - expect).abs() < 1e-6);
        assert!((farthest - 49.0 / 12.0).abs() < 0.1, "carried {farthest:.2} m");
    }

    // agent-models §5.3: a fielder at 7 m/s whose target swings 90° carries on the old way
    // while he turns: some drift, but less than a straight stop at 6 m/s² would take
    // (7²/12 = 4.08 m), and by 2 s he is running at the target near top speed.
    #[test]
    fn a_turn_carries_the_fielder_on_a_curve() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
        assert!(drift > 1.0 && drift < 49.0 / 12.0, "drifted {drift:.2} m (a_lat {a_lat})");
        let v = sim.f[SS].vel;
        assert!(v.x.abs() < 0.05 && v.y > 0.9 * sim.f[SS].a.speed, "velocity after 2 s {v:?}");
    }

    // Fielding review #2 and #4: a fielder who has reached his spot is standing still, and a
    // running fielder's time to a resting ball counts his speed. Codex momentum review #1: he
    // gets there by braking at 6 m/s², so from 7 m/s the stop takes 7/6 s and 49/12 m; a late
    // chaser runs through his spot at speed.
    #[test]
    fn stopped_fielders_have_no_speed_and_running_ones_keep_it() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 2.0;
        let p0 = sim.f[SS].pos;
        sim.f[SS].role = Role::Cover(2);
        sim.f[SS].vel = Vec3::new(7.0, 0.0, 0.0);
        sim.f[SS].target = p0 + Vec3::new(49.0 / 12.0, 0.0, 0.0);
        let mut stopped = None;
        for k in 1..=120 {
            sim.move_fielders();
            sim.t += DT;
            if stopped.is_none() && sim.f[SS].vel == Vec3::ZERO {
                stopped = Some(k as f64 * DT);
            }
        }
        let stopped = stopped.expect("he stops");
        assert!((stopped - 7.0 / 6.0).abs() < 0.1, "stopped after {stopped:.3} s");
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

    /// The ball in fielder `holder`'s hands at `at`, the defense's jobs set as at assignment.
    fn held_at(sim: &mut Sim, holder: usize, at: Vec3) {
        sim.primary = holder;
        sim.f[holder].pos = at;
        sim.f[holder].target = at;
        sim.ball = Ball::Held { by: holder };
        sim.cover_roles();
        sim.assign_relay();
        sim.assign_backups();
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R, base: 2, speed: 8.2 }]);
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
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R, base: 2, speed: 8.2 }]);
        inp.ball = BallState { r: Vec3::new(0.0, 0.5, 1.0), v: Vec3::new(-4.0, 28.0, 22.0), w: Vec3::ZERO };
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 1.0;
        sim.assign();
        let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
        assert!(path.bounces[0].0 > sim.t + 2.0, "the ball must still be well up");
        assert!(m::hypot(sim.fielding_point().x, sim.fielding_point().y) > RELAY_DEEP_M);
        assert_eq!(sim.f[SS].role, Role::Relay(4));
        // A short fly an infielder can catch keeps him on the ball.
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R, base: 2, speed: 8.2 }]);
        inp.ball = BallState { r: Vec3::new(0.0, 0.5, 1.0), v: Vec3::new(-2.0, 14.0, 24.0), w: Vec3::ZERO };
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 1.0;
        sim.assign();
        assert!(!(0..9).any(|i| matches!(sim.f[i].role, Role::Relay(_))));
    }

    // Codex review 2026-11-12 #1: the receiver plans from his own read of the throw, never
    // from its realized future. At the release he has not seen it yet: whatever the
    // executed speed, he is on the spot it was thrown to.
    #[test]
    fn receiver_plans_from_his_read_not_the_throws_future() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let from = Vec3::new(0.0, 100.0, 0.0);
        let second = base_pos(2);
        let d = (second - from).norm();
        let elev = solve_throw(&fx.env, THROW_RELEASE_Z, THROW_CATCH_Z, d, 37.0).unwrap().elev;
        let mut reads = Vec::new();
        for v in [35.0, 37.0, 39.0] {
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
    }

    // Codex momentum review #4: a throw aimed at second that passes 1.7 m to the side of the
    // bag (beyond the 1.5 m receiving reach from it) is caught by a second baseman who steps
    // off; weighing his read toward the aim must not pin him to the bag. Perfect read and
    // hands, so only the plan is tested.
    #[test]
    fn a_throw_passing_beyond_reach_of_the_bag_draws_the_receiver_off_it() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 5.0;
        let second = base_pos(2);
        sim.f[3].pos = second;
        sim.f[3].target = second;
        let from = second + Vec3::new(0.0, 20.0, 0.0);
        let off = second + Vec3::new(1.7, 0.0, 0.0);
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
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R, base: 2, speed: 8.2 }]);
        inp.fattrs[CF].arm = 30.0;
        inp.fattrs[CF].arm_sigma = 0.0;
        inp.fattrs[SS].arm_sigma = 0.0;
        let deep = Vec3::new(-5.0, 112.0, 0.0);
        assert!(solve_throw(&fx.env, THROW_RELEASE_Z, THROW_CATCH_Z, deep.norm(), 30.0).is_none(), "the direct throw must be out of reach");
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        sim.t = 5.0;
        held_at(&mut sim, CF, deep);
        // The relay man is already on his spot; the runner is past third, jogging home (he
        // is not moved by this test, so the play on him stays open).
        sim.f[SS].pos = sim.f[SS].target;
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
        assert_eq!(who, vec![sim.f[CF].id, sim.f[SS].id], "throws by {who:?}; facts {:?}; catcher at {:?}", sim.out.facts, sim.f[1].pos);
        assert!(matches!(sim.ball, Ball::Held { by: 1 }), "ball: {:?}", match &sim.ball { Ball::Held { by } => format!("held by {by}"), _ => "loose".into() });
    }

    /// Seed 7475 play 26 (2026-11-13): runners on first and third, a chopper to the
    /// pitcher. Where does he throw with `outs` out?
    fn pitcher_throws_to(outs: u8) -> u8 {
        const R1: AgentId = 0x0007;
        const R3: AgentId = 0x0008;
        let fx = fixture(&[(R1, 1), (R3, 3)]);
        let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R1, base: 1, speed: 8.2 }, RunnerIn { id: R3, base: 3, speed: 8.2 }]);
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R1, base: 1, speed: 8.2 }]);
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

    // A fielder left with no job does not chase the ball (pitcher-outfield diagnosis,
    // 2026-11-12). `assign_among` writes every candidate's own intercept into his target
    // before the cover and backup steps; on a fly to right-center with nobody on, the
    // second baseman gets no job and must pull up where braking stops him instead of
    // running out toward the ball. (The pitcher was idle here too until he was given third
    // to back up, owner 2026-11-13; `idle_fielders_back_up_the_play` checks that.)
    #[test]
    fn idle_fielders_do_not_chase_the_ball() {
        let fx = fixture(&[]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        let at = Vec3::new(62.0, 88.0, 0.0);
        // The second baseman is already moving toward the ball at 7 m/s: he stops within
        // v²/(2·6 m/s²) ≈ 4.1 m of where he is, on his line of motion.
        const B2: usize = 3;
        let toward = (at - sim.f[B2].pos).normalized();
        sim.f[B2].vel = toward * 7.0;
        for fd in sim.f.iter_mut() {
            fd.target = at;
        }
        sim.primary = 8;
        sim.cover_roles();
        sim.assign_backups();
        let idle: Vec<usize> = (0..9).filter(|&i| sim.f[i].role == Role::Idle).collect();
        assert!(idle.contains(&B2) && !idle.contains(&0), "idle: {idle:?}");
        for &i in &idle {
            let fd = &sim.f[i];
            let off = Vec3::new(fd.target.x - fd.pos.x, fd.target.y - fd.pos.y, 0.0);
            assert!(off.norm() < 5.0, "f{i} idle at {:?} aims at {:?}", fd.pos, fd.target);
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
        assert!(sim.f[B2].vel.norm() < 1e-9 && (sim.f[B2].pos - start).norm() < 5.0, "2B at {:?} moving {:?}", sim.f[B2].pos, sim.f[B2].vel);
    }

    /// A ball hit from the plate with velocity `v`, every fielder reading it without error
    /// (the tests below are about the call rule, not the draws), at the outfielders'
    /// reaction time.
    fn hit(fx: &Fixture, v: Vec3) -> LiveInput<'_> {
        let mut inp = input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
        // The spin fielders expect at this launch angle, so their reads match the ball.
        let h = Vec3::new(v.x, v.y, 0.0).normalized();
        let rpm = typical_batted_spin_rpm(m::atan2(v.z, m::hypot(v.x, v.y)));
        inp.ball = BallState { r: Vec3::new(0.0, 0.5, 1.0), v, w: Vec3::new(h.y, -h.x, 0.0) * (rpm * core::f64::consts::PI / 30.0) };
        inp
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
        let inp = hit(&fx, Vec3::new(7.0, 22.0, 26.0));
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        at_first_read(&mut sim);
        let land = landing(&sim);
        assert!((land - Vec3::new(21.0, 67.0, 0.0)).norm() < 3.0, "lands at {land:?}");
        assert_eq!((tier(&sim, B2), tier(&sim, RF)), (2, 1));
        sim.assign();
        assert_eq!(sim.primary, B2);
        assert_ne!(sim.f[RF].role, Role::Chase, "the infielder has called it");
        // A fly hit at the centre fielder he has only to wait for: he calls it at once.
        let inp = hit(&fx, Vec3::new(0.0, 33.0, 22.0));
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        at_first_read(&mut sim);
        let land = landing(&sim);
        assert!((land - sim.f[CF].pos).norm() < 4.0, "lands at {land:?}");
        assert_eq!(tier(&sim, CF), 3);
        sim.assign();
        assert_eq!(sim.primary, CF);
        assert_eq!((0..9).filter(|&i| sim.f[i].role == Role::Chase).count(), 1);
    }

    // A fly into the right-center gap that the centre and right fielders both believe they
    // can run down, neither surely at first look: both go for it. A second and a half on, the
    // centre fielder's read has firmed up enough to call it, and the right fielder peels off.
    // (Re-chosen 2026-11-13 with the plate cue of §5.1, which made the centre fielder sure of
    // the old ball, landing at (18, 91), at once; the right fielder now clears the chase
    // threshold by 0.4 s, where the old ball left him 30 ms.)
    #[test]
    fn until_someone_is_sure_everyone_who_can_catch_it_goes() {
        const RF: usize = 8;
        let fx = fixture(&[]);
        let inp = hit(&fx, Vec3::new(9.0, 34.0, 24.0));
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
        let pi = sim.primary;
        let p = sim.predict(pi);
        (sim.f[pi].t_int, sim.f[pi].target) = sim.intercept(pi, &p);
        sim.f[pi].belief = Some(p);
        sim.recall();
        assert_eq!(sim.primary, CF);
        assert_eq!(tier(&sim, CF), 3);
        assert_eq!(sim.f[CF].role, Role::Chase);
        assert_ne!(sim.f[RF].role, Role::Chase, "called off");
    }

    // Agent-models §5.1, the plate cue: flying a ball's true velocity and spin back to contact
    // lands on the contact point (the world's flight, run forward with its own step, is the
    // independent reference), for a liner, a fly and a pop-up, at a time between steps too.
    // With the typical spin in place of the true one it misses by about the spin floor.
    #[test]
    fn the_plate_cue_flies_the_ball_back_to_contact() {
        let env = FlightEnv::default_env();
        let p0 = Vec3::new(0.0, 0.5, 1.0);
        for (v, rpm) in [(Vec3::new(-8.0, 40.0, 7.0), 2500.0), (Vec3::new(9.0, 33.0, 22.0), 4800.0), (Vec3::new(4.0, 14.0, 30.0), 6400.0)] {
            let h = Vec3::new(v.x, v.y, 0.0).normalized();
            let w0 = Vec3::new(h.y, -h.x, 0.0) * (rpm * core::f64::consts::PI / 30.0);
            let mut st = BallState { r: p0, v, w: w0 };
            let mut t = 0.0f64;
            for s in [0.4f64, 1.0, 1.85, 3.0] {
                while t < s - 1e-9 {
                    let h = (s - t).min(0.01);
                    st = env.step(&st, h);
                    t += h;
                }
                let back = position_from_plate(&env, p0, st.v, st.w, s);
                assert!((back - st.r).norm() < 1e-3, "v {v:?} at {s} s: {back:?} against {:?}", st.r);
                let typical = position_from_plate(&env, p0, st.v, believed_spin(st.v, Some(s)), s);
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
            let belief = landing_depth_sigma(inp.env, inp.field, (p, v), t, cf, a.read_sigma0, a.tau_read, view, Some((s, inp.ball.r)));
            // This ball has the typical spin, so the spread has no spin error; the belief
            // also allows for the spin he cannot see, in the plate's share of his depth.
            let shrink = 1.0 / m::sqrt(1.0 + view / a.tau_read);
            let sp = a.read_sigma0 * m::hypot(p.x - cf.x, p.y - cf.y) / READ_REF_DIST * shrink;
            let (_, _, k) = plate_weighing(los, sp, READ_VEL_FRAC * v.norm() * shrink, s);
            let expect = m::hypot(spread, k * PLATE_SPIN_M * s * m::sqrt(s));
            assert!(belief > 0.85 * expect && belief < 1.15 * expect, "at {s} s: believes {belief:.2} m, spread {spread:.2} m, with spin {expect:.2} m");
        }
    }

    // Agent-models §5.7, "peels off to back up": a pop-up coming down at (20, 36) m, between
    // first and second, that both the first and second basemen are sure of. The first baseman, nearer, has more time to
    // spare and calls it; the second baseman, with no base to cover (the shortstop takes
    // second on a ball to the right side), does not stop where he is but backs up the catch
    // from 5 m short of where his own read has it coming down, on his side of it. A re-call he
    // is not part of leaves him that job; once the ball is down it ends.
    #[test]
    fn a_called_off_infielder_backs_up_the_catch() {
        const B1: usize = 2;
        const B2: usize = 3;
        let fx = fixture(&[]);
        let inp = hit(&fx, Vec3::new(7.0, 12.5, 28.0));
        let mut st = fx.st.clone();
        let mut sim = new_sim(&inp, &mut st);
        at_first_read(&mut sim);
        let land = landing(&sim);
        assert!((land - Vec3::new(20.0, 36.0, 0.0)).norm() < 2.0, "lands at {land:?}");
        assert!((sim.f[B1].pos - land).norm() < (sim.f[B2].pos - land).norm(), "1B nearer");
        assert_eq!((tier(&sim, B1), tier(&sim, B2)), (2, 2), "both sure");
        sim.assign();
        assert_eq!(sim.primary, B1);
        assert_eq!(sim.f[5].role, Role::Cover(2));
        let fd = &sim.f[B2];
        assert_eq!(fd.role, Role::Backup, "2B");
        // Error-free read: his read of where it comes down is where it does (to the
        // intercept's 0.12 s sampling and the planning reach).
        let off = fd.target - land;
        assert!((off.norm() - PEEL_BACKUP_M).abs() < 1.5, "2B backs up from {:?}, {:.1} m from the landing", fd.target, off.norm());
        let side = Vec3::new(fd.pos.x - land.x, fd.pos.y - land.y, 0.0).normalized();
        assert!(off.normalized().dot(side) > 0.9, "on his side of it");
        // A re-call among others leaves him the job.
        let spot = fd.target;
        sim.assign_among(vec![B1, 0, 1], Some(B1));
        assert_eq!((sim.f[B2].role, sim.f[B2].target), (Role::Backup, spot));
        // While the call stands he leaves the ball alone, even one that comes within his reach
        // (a misread can put him under it: Codex peel-off review #1): no glove attempt. The
        // first baseman is moved away so that only the second baseman could reach it.
        let Ball::Free { path, .. } = &sim.ball else { unreachable!() };
        let t_down = path.bounces[0].0;
        sim.t = t_down - 0.05;
        let (b, _) = sim.ball_at(sim.t);
        assert!(b.z > 0.0 && b.z < CATCH_MAX_Z, "ball at {b:?}");
        let b2_at = sim.f[B2].pos;
        sim.f[B1].pos = Vec3::new(b.x + 10.0, b.y, 0.0);
        sim.f[B2].pos = Vec3::new(b.x, b.y, 0.0);
        sim.try_captures();
        assert_eq!((sim.f[B2].attempts, matches!(sim.ball, Ball::Free { .. })), (0, true));
        // Once the ball is down, the tick's expiry step ends the job (Codex peel-off review
        // #2): he has no spot to go to any more, and the ball is re-called among the fielders
        // near it, so he can field it.
        sim.f[B2].pos = b2_at;
        sim.t = t_down + DT;
        sim.expire_peels();
        assert_eq!(sim.f[B2].peel, None);
        assert_ne!((sim.f[B2].role, sim.f[B2].target), (Role::Backup, spot));
    }

    // Codex idle-fielder review #1: every way into `Role::Idle` brakes the fielder, not only
    // the calls. Runner on second, ball held by the right fielder at (25, 62) m: the first
    // baseman is the cutoff to home, his spot ~12.5 m in from where he stands. His job then
    // ends three ways: the right fielder throws straight to second past him (`holder_act`),
    // a throw to another man is in the air (`update_relay`), or an infielder has the ball
    // (`update_relay`). Nobody gives him another job (every base is covered and the throw's
    // backup is someone else), so a man standing still stays where he is, and a man already
    // running toward his old spot at 7 m/s pulls up within v²/(2·6 m/s²) ≈ 4.08 m along his
    // line instead of going on to it.
    #[test]
    fn a_retired_cutoff_man_with_no_job_stops() {
        const R: AgentId = 0x0007;
        const B1: usize = 2;
        let fx = fixture(&[(R, 2)]);
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: R, base: 2, speed: 8.2 }]);
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
                    sim.throw_to(8, base_pos(2), 2, c2, false);
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
            let want = run * run / (2.0 * 6.0);
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
        let inp = input(fx, vec![RunnerIn { id: bat, base: 0, speed: 8.2 }]);
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

    /// The ball leaves the throwing hand (owner backlog A1): facing his target, a
    /// right-hander's hand is on his right and a left-hander's on his left, `THROW_ARM_OFFSET_M`
    /// out, at release height. Worked in the sim frame (+X first base, +Y center field).
    /// The center fielder at (0, 80) with the dive flag up and his get-up due at `up_s`,
    /// the shortstop on second, nobody running. A slow roller passes half a metre in front
    /// of him from time 1.5, starting `x0` along its path. Runs whole ticks (captures, the
    /// holder, movement) until he throws; returns the sim and the release time.
    fn center_fielder_takes_a_roller(up_s: f64, x0: f64) -> (Sim<'static>, f64) {
        let fx: &'static Fixture = Box::leak(Box::new(fixture(&[])));
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }])));
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: 7, base: 2, speed: 8.2 }])));
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
    // The old proximity shortcut ran to first without looking at home.
    #[test]
    fn a_sure_force_at_home_beats_carrying_first() {
        let fx = fixture(&[(5, 1), (6, 2), (7, 3)]);
        let runners = vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: 5, base: 1, speed: 8.2 }, RunnerIn { id: 6, base: 2, speed: 8.2 }, RunnerIn { id: 7, base: 3, speed: 8.2 }];
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
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }])));
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

    // Taking the ball moving away from first at 5 m/s he owes 0.2 s of footwork: set at
    // 0.85 s when he has the time.
    #[test]
    fn a_fielder_moving_away_needs_longer_to_get_set() {
        let away = (Vec3::new(-8.0, 38.0, 0.0) - base_pos(1)).normalized() * 5.0;
        let (sim, after) = shortstop_to_first(3.0, away);
        assert!(after >= 0.85 - 1e-9 && after < 0.85 + DT + 1e-9, "let go after {after:.3} s");
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
        let inp = Box::leak(Box::new(input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }, RunnerIn { id: 2, base: 1, speed: 8.2 }])));
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
        let inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
        let mut inp = input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
        let mut inp = input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
        let inp: &'static LiveInput<'static> = Box::leak(Box::new(input(fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }])));
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
        assert!(sim.primary == 6 && sim.f[7].role == Role::Chase, "fixture: left field has it and center field chases too");
        sim.replan_primary();
        sim.assign_among(vec![6, 5, 4, 3], Some(6));
        assert!(sim.f[7].role == Role::Chase, "called off with nobody sure: {:?}", sim.f[7].role);
    }

    /// Foul pops that come down in the first-base seats, from just past the stands wall to
    /// 10 m in (Gemini play-area review P2-3): nobody catches one beyond his reach over the
    /// wall, and a fielder who goes after one sees the wall and stops at it. Arriving at the
    /// wall he is no faster than stopping from where he could still see it allows: from at
    /// most 0.35 m out at the braking limit (6 m/s², BRAKE), √(2·6·0.35) ≈ 2.05 m/s, plus one
    /// 0.1 s sample's slack (0.6 m/s).
    #[test]
    fn nobody_runs_into_the_stands_after_a_ball_out_of_reach() {
        let fx = fixture(&[]);
        let mut worst = (0.0f64, 0.0f64);
        for k in 0..8 {
            let mut inp = input(&fx, vec![RunnerIn { id: fx.st.batter, base: 0, speed: 8.2 }]);
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
        assert!(worst.0 <= 2.05 + 0.6, "a fielder reached the wall at {:.2} m/s (ball {:.1} m into the seats)", worst.0, worst.1);
    }

}
