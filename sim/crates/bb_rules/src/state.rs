//! Rules state, inputs (facts), and events (rules-spec §2–§3).

/// Agent ids follow data-formats §2.1: `side << 8 | slot`.
pub type AgentId = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Half {
    Top,
    Bottom,
}

impl Half {
    /// Index of the batting team: 0 away (top), 1 home (bottom).
    pub fn batting(self) -> usize {
        match self {
            Half::Top => 0,
            Half::Bottom => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Pregame,
    InProgress,
    Final,
}

/// Bases are numbered 1–3; 4 is home (scored).
pub type Base = u8;

/// A runner during a live play (rules-spec §5.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunnerState {
    pub runner: AgentId,
    /// Base occupied at the time of the pitch (0 = the batter).
    pub start_base: Base,
    /// Highest base reached so far (bases must be touched in order).
    pub last_base: Base,
    /// The base the runner most recently touched (moves backward on a retreat).
    pub at: Base,
    /// Base the runner is currently touching, if any.
    pub on_base: Option<Base>,
    /// Base the runner is forced to, while the force stands.
    pub forced_to: Option<Base>,
    /// Must retouch `start_base` after the first touch of a caught fly.
    pub retouch_required: bool,
    pub left_start_at: Option<f64>,
    pub retouched: bool,
    pub out: bool,
    pub scored_at: Option<f64>,
    /// The batter-runner's overrun of first base (rules-spec §5.5; OBR 5.09(b)(4)).
    pub overrun: Overrun,
    /// Latest `RunnerPosition` sample: (time, path progress in bases, rules-spec §5.5).
    pub sample: Option<(f64, f64)>,
}

/// The batter-runner's protection after overrunning or oversliding first base (rules-spec
/// §5.5, OBR 5.09(b)(4) exception).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Overrun {
    /// No overrun seen (yet).
    #[default]
    NotSeen,
    /// Past first base down the line after touching it, not yet back: not liable to a tag.
    Protected,
    /// Back on first, or made an attempt toward second: an ordinary runner from now on.
    Ended,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FairFoul {
    Pending,
    Fair,
    Foul,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlayState {
    Dead,
    /// A pitch is in flight / being judged.
    Pitch(PitchState),
    /// A batted ball is live.
    Live(LivePlay),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PitchState {
    pub in_zone: Option<bool>,
    pub bounced: bool,
    pub swung: Option<bool>,
    pub contact: bool,
    /// The pitch touched the batter; judged once the swing measurement is in (§4.4).
    pub hit_body: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LivePlay {
    pub fair: FairFoul,
    /// The ball has touched the ground or an object (so no catch is possible).
    pub grounded: bool,
    /// First fielder touch time of a fly that was then caught (retouch reference).
    pub first_touch: Option<f64>,
    pub caught: bool,
    pub runners: Vec<RunnerState>,
    /// Outs recorded on this play: (time, runner, kind).
    pub outs: Vec<(f64, AgentId, OutKind)>,
    pub runs_pending: Vec<(f64, AgentId)>,
    /// Latest fact time seen in this play (facts must not go backward).
    pub last_t: f64,
    /// Passes made while the batted ball was in doubt, in fact order (rules-spec §5.5). The
    /// passer is already off the play (`out`); the out is recorded, at the pass's own time, once
    /// the ball is fair or caught, and dropped with the play on an uncaught foul.
    pub pending_passes: Vec<PendingPass>,
}

/// A pass made while the batted ball was in doubt (rules-spec §5.5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PendingPass {
    pub t: f64,
    pub runner: AgentId,
    pub passed: AgentId,
    /// The base the passer was heading for (the out's `base`).
    pub base: Base,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutKind {
    Strikeout,
    FlyOut,
    Force,
    Tag,
    Appeal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PitchCall {
    Ball,
    CalledStrike,
    SwingingStrike,
    Foul,
    InPlay,
    HitByPitch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaResult {
    Walk,
    HitByPitch,
    Strikeout,
    /// The batter reached or was retired on a batted ball; the scorer classifies it later.
    BattedBall,
    HomeRun,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdvanceCause {
    Walk,
    HitByPitch,
    Award,
    OnPlay,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    GameStarted,
    HalfInningStarted { inning: u8, half: Half },
    PlateAppearanceStarted { batter: AgentId },
    PitchResult { call: PitchCall, balls: u8, strikes: u8 },
    FairFoulDecided { fair: bool },
    Catch { fielder: AgentId },
    RunnerAdvanced { runner: AgentId, from: Base, to: Base, cause: AdvanceCause },
    RunnerOut { runner: AgentId, base: Base, kind: OutKind },
    RunScored { runner: AgentId },
    PlateAppearanceEnded { batter: AgentId, result: PaResult },
    HalfInningEnded { runs: u8, left_on_base: u8 },
    GameEnded { away: u16, home: u16, innings: u8, walk_off: bool },
    /// `runner` passed the preceding runner `passed` and is out (OBR 5.09(b)(9)); the
    /// `RunnerOut` that follows carries `OutKind::Tag`, the same time play for run counting
    /// (rules-spec §5.5, §6.4).
    RunnerPassed { runner: AgentId, passed: AgentId },
    RulesWarning { code: &'static str },
}

/// Where a ball touched (rules-spec §2 `BallTouched`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Touch {
    /// Ground at (x, y), m.
    Ground { x: f64, y: f64 },
    /// Out of the playing field: over the fence in flight (`in_flight`) or after bouncing.
    OutOfPlay { x: f64, y: f64, in_flight: bool },
    /// The batter's body (pitch only, before bat contact).
    Batter,
    FoulPole,
    /// The outfield wall below its top (a catch is no longer possible, rules-spec §5.3).
    Wall { x: f64, y: f64 },
}

/// Rules inputs: facts from the world and harness pseudo-facts (rules-spec §2).
#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    StartGame,
    PitchDelivered { t: f64 },
    PitchCrossed { t: f64, in_zone: bool, bounced: bool },
    /// End of swing: whether it counts as a swing (rules-spec §4.5 yaw judgment, or contact).
    SwingMeasured { t: f64, swung: bool },
    BatContact { t: f64 },
    BallTouched { t: f64, what: Touch },
    /// The ball passed 1B or 3B on or near the ground, on the fair or foul side.
    BallPassedBase { t: f64, fair: bool },
    BallSettled { t: f64, x: f64, y: f64 },
    /// The orchestrator could not settle the play in time (rules-spec §5.8 item 4).
    PlayTimeout { t: f64 },
    /// A fielder secured the ball at (x, y) (rules-spec §5.3).
    GloveSecured { t: f64, fielder: AgentId, x: f64, y: f64 },
    BaseTouched { t: f64, agent: AgentId, base: Base, holding_ball: bool },
    BaseLeft { t: f64, agent: AgentId, base: Base },
    TagContact { t: f64, fielder: AgentId, runner: AgentId, holding_ball: bool },
    /// Harness/orchestrator pseudo-fact: the play has settled (rules-spec §5.8 item 3).
    PlayEndCheck { t: f64 },
}

/// A sampled runner position (rules-spec §2 `RunnerPosition`, about 30 Hz while a play is
/// live): the runner's ground point in the sim frame, m. It is a stream rather than a
/// discrete fact, so it is its own type; `step` takes it like any `Input`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunnerPosition {
    pub t: f64,
    pub runner: AgentId,
    pub x: f64,
    pub y: f64,
}

/// Anything `step` accepts: a discrete fact or command (`Input`) or a position sample.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fact<'a> {
    Input(&'a Input),
    Position(&'a RunnerPosition),
}

impl<'a> From<&'a Input> for Fact<'a> {
    fn from(i: &'a Input) -> Self {
        Fact::Input(i)
    }
}

impl<'a> From<&'a RunnerPosition> for Fact<'a> {
    fn from(p: &'a RunnerPosition) -> Self {
        Fact::Position(p)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RulesOptions {
    pub innings: u8,
    /// An overrunning batter-runner more than this far into fair territory (m, from the
    /// first-base line) has made an attempt toward second (rules-spec §5.5) [E].
    pub overrun_attempt_m: f64,
    /// A following runner whose path position exceeds a preceding runner's by more than this
    /// (m) has passed him (rules-spec §5.5) [E].
    pub pass_margin_m: f64,
}

impl Default for RulesOptions {
    fn default() -> Self {
        Self { innings: 9, overrun_attempt_m: 1.5, pass_margin_m: 0.5 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RulesState {
    pub options: RulesOptions,
    pub status: Status,
    pub inning: u8,
    pub half: Half,
    /// Runs by inning for [away, home].
    pub runs_by_inning: [Vec<u8>; 2],
    pub outs: u8,
    pub balls: u8,
    pub strikes: u8,
    /// Runner on 1B, 2B, 3B.
    pub bases: [Option<AgentId>; 3],
    pub lineup: [[AgentId; 9]; 2],
    pub next_up: [usize; 2],
    pub batter: AgentId,
    pub play: PlayState,
    /// A winning run has scored in the bottom of the last inning or later; the game ends as
    /// soon as the current plate appearance is complete.
    pub walk_off: bool,
}

impl RulesState {
    pub fn new(lineup: [[AgentId; 9]; 2], options: RulesOptions) -> Self {
        Self {
            options,
            status: Status::Pregame,
            inning: 1,
            half: Half::Top,
            runs_by_inning: [Vec::new(), Vec::new()],
            outs: 0,
            balls: 0,
            strikes: 0,
            bases: [None; 3],
            lineup,
            next_up: [0, 0],
            batter: lineup[0][0],
            play: PlayState::Dead,
            walk_off: false,
        }
    }

    /// The default 9-v-9 fixture: away slots 1–9 are agents 0x0001–0x0009, home 0x0101–0x0109.
    pub fn default_fixture() -> Self {
        let a: [AgentId; 9] = core::array::from_fn(|i| (i + 1) as AgentId);
        let h: [AgentId; 9] = core::array::from_fn(|i| 0x0100 | (i + 1) as AgentId);
        Self::new([a, h], RulesOptions::default())
    }

    /// Innings at which the liveness warning fires (rules-spec §4.1); play continues.
    pub const LIVENESS_INNING: u8 = 30;

    pub fn score(&self) -> [u16; 2] {
        [0, 1].map(|s| self.runs_by_inning[s].iter().map(|&r| r as u16).sum())
    }
}
