//! C ABI for engine clients (data-formats §6.4, PLAN §3.1). Opaque handles; every payload is
//! a FlatBuffers blob owned by Rust and released with [`bb_blob_free`].
//!
//! M4 skeleton surface (the M0.8 Unreal spike decides the final list):
//! `bb_game_create`, `bb_game_advance`, `bb_game_score`, `bb_blob_free`, `bb_game_free`.
//! The header is `include/bb_ffi.h`.
#![allow(unsafe_code)]

use bb_game::game::Game;
use bb_schema::tl::MarkerKind;

/// Opaque game handle.
pub struct BbGame {
    game: Option<Game>,
}

/// Create a new AI-vs-AI game with average players. Free with [`bb_game_free`].
#[unsafe(no_mangle)]
pub extern "C" fn bb_game_create(seed: u64) -> *mut BbGame {
    // Recording is on so the timelines carry the realized ball and everyone's movement.
    Box::into_raw(Box::new(BbGame { game: Some(Game::new(seed, true)) }))
}

/// Advance one pitch. On success writes a `PlayTimeline` blob (`BBTL`) and returns 1;
/// returns 0 when the game is over and −1 on a null argument.
///
/// # Safety
/// `g` must come from [`bb_game_create`]; `out_blob` and `out_len` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bb_game_advance(g: *mut BbGame, out_blob: *mut *mut u8, out_len: *mut usize) -> i32 {
    if g.is_null() || out_blob.is_null() || out_len.is_null() {
        return -1;
    }
    let g = unsafe { &mut *g };
    let Some(game) = g.game.as_mut() else { return 0 };
    match game.step_pitch() {
        Some(step) => {
            let bytes = bb_game::timeline::encode(&step, &game.st, &game.rosters).into_boxed_slice();
            unsafe {
                *out_len = bytes.len();
                *out_blob = Box::into_raw(bytes) as *mut u8;
            }
            1
        }
        None => 0,
    }
}

/// Current score. Returns 1 if the game is final, 0 if in progress, −1 on a null argument.
///
/// # Safety
/// `g` must come from [`bb_game_create`]; `away` and `home` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bb_game_score(g: *const BbGame, away: *mut u16, home: *mut u16) -> i32 {
    if g.is_null() || away.is_null() || home.is_null() {
        return -1;
    }
    let g = unsafe { &*g };
    let Some(game) = g.game.as_ref() else { return -1 };
    let s = game.st.score();
    unsafe {
        *away = s[0];
        *home = s[1];
    }
    game.finished() as i32
}

/// Size of the name fields of [`BbPlayer`], including the terminating NUL.
pub const BB_NAME_LEN: usize = 32;

/// Who a player is (data-formats §2.4). Identity only.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BbPlayer {
    pub player_id: u32,
    /// The agent id he bats and runs under, and the one he fields under.
    pub bat_agent: u16,
    pub field_agent: u16,
    /// 0 away, 1 home.
    pub side: u8,
    /// Uniform number, 1–99.
    pub number: u8,
    /// Fielding position, scorer's numbering: 1 pitcher … 9 right field.
    pub position: u8,
    /// Place in the batting order, 1–9.
    pub batting_order: u8,
    /// Hands: 1 left, 2 right; `bats` may also be 3, a switch hitter, who bats from the side
    /// opposite the pitcher's hand (on a given play, his box is on the first-base side, sim
    /// x > 0, when he bats left).
    pub bats: u8,
    pub throws: u8,
    pub _pad: [u8; 2],
    /// NUL-terminated, ASCII.
    pub given: [u8; BB_NAME_LEN],
    pub family: [u8; BB_NAME_LEN],
}

fn name_field(s: &str) -> [u8; BB_NAME_LEN] {
    let mut out = [0u8; BB_NAME_LEN];
    // Names are ASCII (data-formats §2.4), so a cut at any byte leaves valid text.
    let n = s.len().min(BB_NAME_LEN - 1);
    out[..n].copy_from_slice(&s.as_bytes()[..n]);
    out
}

fn player(team: &bb_game::roster::TeamRoster, p: &bb_game::roster::Player) -> BbPlayer {
    let hand = |h: bb_observe::Hand| match h {
        bb_observe::Hand::Left => 1,
        bb_observe::Hand::Right => 2,
    };
    let bats = match p.bats {
        bb_observe::Bats::Left => 1,
        bb_observe::Bats::Right => 2,
        bb_observe::Bats::Switch => 3,
    };
    BbPlayer {
        player_id: p.player_id,
        bat_agent: team.bat_agent(p),
        field_agent: team.field_agent(p),
        side: team.side,
        number: p.number,
        position: p.position,
        batting_order: p.batting_order,
        bats,
        throws: hand(p.throws),
        _pad: [0; 2],
        given: name_field(p.given),
        family: name_field(p.family),
    }
}

/// One player of a side's roster, by place in the batting order: `side` 0 away, 1 home;
/// `batting_slot` 0–8. Returns 1 and fills `out`; 0 if there is no such side or slot; −1 on
/// a null argument.
///
/// # Safety
/// `g` must come from [`bb_game_create`]; `out` must be a valid pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bb_game_roster(g: *const BbGame, side: u32, batting_slot: u32, out: *mut BbPlayer) -> i32 {
    if g.is_null() || out.is_null() {
        return -1;
    }
    let Some(game) = unsafe { &*g }.game.as_ref() else { return -1 };
    let Some(team) = game.rosters.teams.get(side as usize) else { return 0 };
    let Some(p) = team.batting_order().get(batting_slot as usize).copied() else { return 0 };
    unsafe { *out = player(team, p) };
    1
}

/// The player behind an agent id, batting or fielding (a `BbActor::agent`, a
/// `BbPlayInfo::batter`, a `BbCall::runner`). Returns 1 and fills `out`; 0 if the id is not
/// a player; −1 on a null argument.
///
/// # Safety
/// `g` must come from [`bb_game_create`]; `out` must be a valid pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bb_game_player(g: *const BbGame, agent: u16, out: *mut BbPlayer) -> i32 {
    if g.is_null() || out.is_null() {
        return -1;
    }
    let Some(game) = unsafe { &*g }.game.as_ref() else { return -1 };
    let Some((team, p)) = game.rosters.find(agent) else { return 0 };
    unsafe { *out = player(team, p) };
    1
}

/// Release a blob returned by this library.
///
/// # Safety
/// `blob`/`len` must be exactly what a `bb_*` call returned, freed once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bb_blob_free(blob: *mut u8, len: usize) {
    if blob.is_null() {
        return;
    }
    drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(blob, len)) });
}

/// Destroy a game handle.
///
/// # Safety
/// `g` must come from [`bb_game_create`] and not be used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bb_game_free(g: *mut BbGame) {
    if !g.is_null() {
        drop(unsafe { Box::from_raw(g) });
    }
}

/// One actor in a sampled frame (sim frame, metres; yaw in radians from +Y toward +X).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BbActor {
    pub agent: u16,
    pub _pad: u16,
    pub pos: [f32; 3],
    pub yaw: f32,
}

/// Maximum actors in a frame: nine fielders, the batter, three runners, and room to spare.
pub const BB_MAX_ACTORS: usize = 16;

/// A fielder off his feet from a dive at this instant: from the moment he leaves them until
/// he is back on them.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BbDive {
    pub agent: u16,
    pub _pad: u16,
    /// Seconds since he left his feet.
    pub since_s: f32,
    /// Seconds until he is back on them.
    pub up_in_s: f32,
    /// Direction of the dive on the ground (sim frame, unit; zero if he went nowhere).
    pub dir: [f32; 2],
}

/// Most dives shown at one instant.
pub const BB_MAX_DIVES: usize = 4;

/// A `Dive` or `DiveUp` marker, decoded.
struct DiveMark {
    up: bool,
    t: f32,
    agent: u16,
    p: [f32; 3],
}

/// The dives under way at `t`: each `Dive` pairs with that fielder's first `DiveUp` strictly
/// after it (a second dive can start the instant he is up from the first, and must not pair
/// with that first get-up: Codex dive animation review #2).
fn active_dives(marks: &[DiveMark], t: f32) -> Vec<BbDive> {
    let mut out = Vec::new();
    for mk in marks.iter().filter(|m| !m.up && m.t <= t) {
        let up = marks.iter().filter(|u| u.up && u.agent == mk.agent && u.t > mk.t).min_by(|a, b| a.t.total_cmp(&b.t));
        let Some(up) = up.filter(|u| u.t > t) else { continue };
        let (dx, dy) = (up.p[0] - mk.p[0], up.p[1] - mk.p[1]);
        let n = (dx * dx + dy * dy).sqrt();
        let dir = if n > 1e-4 { [dx / n, dy / n] } else { [0.0, 0.0] };
        out.push(BbDive { agent: mk.agent, _pad: 0, since_s: t - mk.t, up_in_s: up.t - t, dir });
    }
    out
}

/// Everything's position at one instant of a play (sim frame, metres).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BbFrame {
    pub ball: [f32; 3],
    pub n_actors: u32,
    pub actors: [BbActor; BB_MAX_ACTORS],
    /// 1 if the bat is in someone's hands at this instant (it is dropped after contact).
    pub has_bat: u32,
    pub bat_knob: [f32; 3],
    pub bat_tip: [f32; 3],
    /// The fielder who has the ball at this instant (agent id), or -1 if nobody does.
    pub holder: i32,
    /// Seconds until that holder releases a throw, or -1 if he does not throw it.
    pub release_in_s: f32,
    /// Fielders off their feet from a dive.
    pub n_dives: u32,
    pub dives: [BbDive; BB_MAX_DIVES],
}

/// A play's length, the scoreboard after it, and its headline measurements (SI).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BbPlayInfo {
    pub play_index: u32,
    pub duration_s: f32,
    pub inning: u8,
    pub top: u8,
    pub outs: u8,
    pub balls: u8,
    pub strikes: u8,
    pub away_runs: u8,
    pub home_runs: u8,
    /// Runners on base after the play: bit 0 first, bit 1 second, bit 2 third.
    pub bases: u8,
    pub release_speed_mps: f32,
    pub exit_velocity_mps: f32,
    pub launch_angle_rad: f32,
    /// Play time of contact, s, or −1 if the bat never touched the ball. Show contact
    /// measurements (exit velocity, launch angle) only from then on.
    pub contact_s: f32,
    /// Play time from which the result (`summary`, count, bases, score) may be shown, s: on
    /// a pitch not hit, the umpire's call (the catcher has the ball, plus the signal latency,
    /// rules-spec §1.3); on contact, the end of the play.
    pub call_s: f32,
    /// The play in words, NUL-terminated UTF-8 (ASCII in practice), truncated to fit.
    pub summary: [u8; BB_SUMMARY_LEN],
    /// The umpire's out calls during the play, in time order (`calls[..n_calls]`).
    pub n_calls: u32,
    pub calls: [BbCall; BB_MAX_CALLS],
    /// The batter the pitch was thrown to: his batting id (look him up with
    /// [`bb_game_player`]).
    pub batter: u16,
    pub _pad: u16,
    /// Play time at which the ball left the playing field and was dead (rules-spec §5.8), s,
    /// or −1 if it never left: the moment a client can cut back to the field.
    pub dead_s: f32,
    /// Where it left: the ball's centre as it crossed the wall or fence (sim
    /// frame, m; zeros if it never left).
    pub dead_at: [f32; 3],
    /// Why: 0 it never left; 1 out of play (a foul ball or a throw); 2 home run; 3 ground-rule
    /// double (the event log's `DeadBallReason`).
    pub dead_reason: u8,
    pub _pad2: [u8; 3],
    /// On contact, the projected distance (physics-spec §12): from home to where the batted
    /// ball would have come down to field level unobstructed, m; 0 without contact.
    pub projected_distance_m: f32,
}

/// One out as the umpire signals it (rules-spec §1.3).
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct BbCall {
    /// When the umpire signals it, s from release (within a second of the out).
    pub t_s: f32,
    /// How close the play was, s: the runner's time short of the base (0 bang-bang; very
    /// large for a routine catch).
    pub margin_s: f32,
    /// The runner retired.
    pub runner: u16,
    /// Where: 0 the batter at the plate (a caught fly or a strikeout), 1-3 a base, 4 home.
    pub base: u8,
    pub _pad: u8,
}

/// Room for every out a play can make.
pub const BB_MAX_CALLS: usize = 4;

/// Size of [`BbPlayInfo::summary`], including the terminating NUL.
pub const BB_SUMMARY_LEN: usize = 96;

impl Default for BbPlayInfo {
    fn default() -> Self {
        // SAFETY: every field is a plain number or byte array; all-zero is a valid value.
        unsafe { std::mem::zeroed() }
    }
}

fn timeline<'a>(blob: *const u8, len: usize) -> Option<bb_schema::tl::PlayTimeline<'a>> {
    // Too short to hold a FlatBuffers header and identifier: not a timeline (and the
    // identifier check below would panic on it).
    if blob.is_null() || len < 8 {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts(blob, len) };
    if !flatbuffers::buffer_has_identifier(bytes, "BBTL", false) {
        return None;
    }
    flatbuffers::root::<bb_schema::tl::PlayTimeline>(bytes).ok()
}

/// Read a timeline's summary. Returns 1 on success, −1 on bad arguments or a blob that is
/// not a `BBTL` timeline.
///
/// # Safety
/// `blob`/`len` must be a blob from [`bb_game_advance`]; `out` must be a valid pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bb_timeline_info(blob: *const u8, len: usize, out: *mut BbPlayInfo) -> i32 {
    let (Some(tl), false) = (timeline(blob, len), out.is_null()) else { return -1 };
    let mut info = BbPlayInfo { play_index: tl.play_index(), duration_s: tl.duration_s(), batter: tl.batter(), ..Default::default() };
    if let Some(v) = tl.view_after() {
        info.inning = v.inning();
        info.top = v.top() as u8;
        info.outs = v.outs();
        info.balls = v.balls();
        info.strikes = v.strikes();
        info.away_runs = v.away().map(|t| t.runs()).unwrap_or(0);
        info.home_runs = v.home().map(|t| t.runs()).unwrap_or(0);
        let on = |id: u16, bit: u8| if id != 0xFFFF { bit } else { 0 };
        info.bases = on(v.runner_1b(), 1) | on(v.runner_2b(), 2) | on(v.runner_3b(), 4);
    }
    if let Some(t) = tl.telemetry() {
        info.release_speed_mps = t.release_speed_mps();
        info.exit_velocity_mps = t.exit_velocity_mps();
        info.launch_angle_rad = t.launch_angle_rad();
        info.projected_distance_m = t.projected_distance_m();
    }
    let contact = tl.markers().and_then(|m| m.iter().find(|m| m.kind() == MarkerKind::Contact)).map(|m| m.t());
    info.contact_s = contact.unwrap_or(-1.0);
    // The pitch ends in the catcher's glove, on the ground, or at contact.
    let pitch_end = tl.ball().and_then(|b| b.segments()).and_then(|s| s.iter().next()).and_then(|s| s.knots()).and_then(|k| k.iter().last()).map(|k| k.t()).unwrap_or(0.0);
    info.call_s = if contact.is_some() { info.duration_s } else { pitch_end + bb_game::timeline::UMPIRE_SIGNAL_S as f32 };
    if let Some(s) = tl.summary() {
        let n = s.len().min(BB_SUMMARY_LEN - 1);
        info.summary[..n].copy_from_slice(&s.as_bytes()[..n]);
    }
    if let Some(mks) = tl.markers() {
        let mut calls: Vec<BbCall> = mks
            .iter()
            .filter(|m| m.kind() == MarkerKind::UmpireCall)
            .map(|m| BbCall { t_s: m.t(), margin_s: m.margin_s(), runner: m.agent(), base: m.base(), _pad: 0 })
            .collect();
        calls.sort_by(|a, b| a.t_s.total_cmp(&b.t_s));
        let n = calls.len().min(BB_MAX_CALLS);
        info.calls[..n].copy_from_slice(&calls[..n]);
        info.n_calls = n as u32;
    }
    info.dead_s = -1.0;
    if let Some(m) = tl.markers().and_then(|m| m.iter().find(|m| m.kind() == MarkerKind::BallDead)) {
        info.dead_s = m.t();
        info.dead_at = m.p().map(v3).unwrap_or_default();
        info.dead_reason = m.base();
    }
    unsafe { *out = info };
    1
}

/// Cubic Hermite between two knots (position, velocity) at fraction `u` of the interval `h`.
fn hermite(p0: [f32; 3], v0: [f32; 3], p1: [f32; 3], v1: [f32; 3], h: f32, u: f32) -> [f32; 3] {
    let (u2, u3) = (u * u, u * u * u);
    let (h00, h10, h01, h11) = (2.0 * u3 - 3.0 * u2 + 1.0, u3 - 2.0 * u2 + u, -2.0 * u3 + 3.0 * u2, u3 - u2);
    core::array::from_fn(|k| h00 * p0[k] + h10 * h * v0[k] + h01 * p1[k] + h11 * h * v1[k])
}

fn v3(v: &bb_schema::Vec3f) -> [f32; 3] {
    [v.x(), v.y(), v.z()]
}

/// Sample a timeline at play time `t` (seconds from release; clamped to the play): the ball
/// and every actor. Returns 1 on success, −1 on bad arguments.
///
/// # Safety
/// `blob`/`len` must be a blob from [`bb_game_advance`]; `out` must be a valid pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bb_timeline_sample(blob: *const u8, len: usize, t: f32, out: *mut BbFrame) -> i32 {
    let (Some(tl), false) = (timeline(blob, len), out.is_null()) else { return -1 };
    let mut frame = BbFrame { holder: -1, release_in_s: -1.0, ..Default::default() };
    // Possession, from the markers: the latest securing of the ball not yet followed by a
    // release, and the first release after now.
    if let Some(marks) = tl.markers() {
        let (mut caught, mut let_go, mut next) = (None::<(f32, u16)>, f32::NEG_INFINITY, f32::INFINITY);
        for mk in marks.iter() {
            match mk.kind() {
                MarkerKind::Catch if mk.t() <= t && caught.is_none_or(|c| mk.t() >= c.0) => caught = Some((mk.t(), mk.agent())),
                MarkerKind::ThrowRelease if mk.t() <= t => let_go = let_go.max(mk.t()),
                MarkerKind::ThrowRelease => next = next.min(mk.t()),
                _ => {}
            }
        }
        if let Some((_, who)) = caught.filter(|c| c.0 >= let_go) {
            frame.holder = who as i32;
            frame.release_in_s = if next.is_finite() { next - t } else { -1.0 };
        }
        // Dives under way.
        let dm: Vec<DiveMark> = marks
            .iter()
            .filter(|m| matches!(m.kind(), MarkerKind::Dive | MarkerKind::DiveUp))
            .filter_map(|m| m.p().map(|p| DiveMark { up: m.kind() == MarkerKind::DiveUp, t: m.t(), agent: m.agent(), p: v3(p) }))
            .collect();
        for d in active_dives(&dm, t).into_iter().take(BB_MAX_DIVES) {
            frame.dives[frame.n_dives as usize] = d;
            frame.n_dives += 1;
        }
    }
    // Ball: the last segment that has started by `t` (segments are in time order).
    if let Some(segs) = tl.ball().and_then(|b| b.segments()) {
        let mut best = None;
        for seg in segs.iter() {
            if let Some(k) = seg.knots().filter(|k| !k.is_empty()) {
                if best.is_none() || k.get(0).t() <= t {
                    best = Some(k);
                }
            }
        }
        if let Some(k) = best {
            let n = k.len();
            let i = (0..n).take_while(|&i| k.get(i).t() <= t).count();
            frame.ball = if i == 0 {
                v3(k.get(0).r())
            } else if i >= n {
                v3(k.get(n - 1).r())
            } else {
                let (a, b) = (k.get(i - 1), k.get(i));
                let h = (b.t() - a.t()).max(1e-6);
                hermite(v3(a.r()), v3(a.v()), v3(b.r()), v3(b.v()), h, (t - a.t()) / h)
            };
        }
    }
    if let Some(actors) = tl.actors() {
        for a in actors.iter().take(BB_MAX_ACTORS) {
            let Some(k) = a.root().filter(|k| !k.is_empty()) else { continue };
            let n = k.len();
            let i = (0..n).take_while(|&i| k.get(i).t() <= t).count();
            let (pos, yaw) = if i == 0 {
                (v3(k.get(0).p()), k.get(0).yaw())
            } else if i >= n {
                (v3(k.get(n - 1).p()), k.get(n - 1).yaw())
            } else {
                let (p, q) = (k.get(i - 1), k.get(i));
                let u = (t - p.t()) / (q.t() - p.t()).max(1e-6);
                let (a0, a1) = (v3(p.p()), v3(q.p()));
                (core::array::from_fn(|j| a0[j] + (a1[j] - a0[j]) * u), if u < 0.5 { p.yaw() } else { q.yaw() })
            };
            frame.actors[frame.n_actors as usize] = BbActor { agent: a.agent(), _pad: 0, pos, yaw };
            frame.n_actors += 1;
        }
    }
    if let Some(k) = tl.bat().and_then(|b| b.samples()).filter(|k| !k.is_empty()) {
        let n = k.len();
        let i = (0..n).take_while(|&i| k.get(i).t() <= t).count().clamp(1, n) - 1;
        // A swing's track ends with the follow-through; after that the bat is on the ground.
        if n == 1 || t <= k.get(n - 1).t() + 0.25 {
            let s = k.get(i);
            let (q, knob) = (s.rot(), v3(s.knob()));
            // Bat axis = rot · +X (first column of the rotation matrix).
            let axis = [1.0 - 2.0 * (q.y() * q.y() + q.z() * q.z()), 2.0 * (q.x() * q.y() + q.w() * q.z()), 2.0 * (q.x() * q.z() - q.w() * q.y())];
            frame.has_bat = 1;
            frame.bat_knob = knob;
            frame.bat_tip = core::array::from_fn(|j| knob[j] + BAT_LENGTH_M * axis[j]);
        }
    }
    unsafe { *out = frame };
    1
}

/// Length of the stock bat (34 in), m: the only bat model so far (timeline `model_id` 0).
const BAT_LENGTH_M: f32 = 0.8636;

/// Most batting lines per side the box score carries. A lineup is nine today; substitutes
/// will need more.
pub const BB_BOX_MAX_BATTERS: usize = 12;
/// Most innings in the linescore. A longer game keeps its last `BB_BOX_MAX_INNINGS` and sets
/// `first_inning`.
pub const BB_BOX_MAX_INNINGS: usize = 30;

/// One batter's line of the box score (rules-spec §10, `bb_stats::BatterLine`). Counts are
/// capped at 255.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BbBatLine {
    /// His batting id, as in [`BbPlayer`].
    pub agent: u16,
    pub number: u8,
    /// Fielding position, scorer's numbering 1–9.
    pub position: u8,
    pub batting_order: u8,
    pub _pad: [u8; 3],
    pub pa: u8,
    pub ab: u8,
    pub r: u8,
    pub h: u8,
    pub doubles: u8,
    pub triples: u8,
    pub hr: u8,
    pub rbi: u8,
    pub bb: u8,
    pub k: u8,
    pub hbp: u8,
    pub sf: u8,
    pub sh: u8,
    pub _pad2: [u8; 3],
    /// NUL-terminated, ASCII.
    pub given: [u8; BB_NAME_LEN],
    pub family: [u8; BB_NAME_LEN],
}

/// A side's totals.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BbBoxTotals {
    pub pa: u16,
    pub ab: u16,
    pub r: u16,
    pub h: u16,
    pub doubles: u16,
    pub triples: u16,
    pub hr: u16,
    pub rbi: u16,
    pub bb: u16,
    pub k: u16,
    pub hbp: u16,
    pub sf: u16,
    pub sh: u16,
    /// Runners left on base at the end of each half-inning played so far.
    pub lob: u16,
    /// Batted-ball plays the scorer could not rule (a walk-off before the batter reached
    /// first); not counted as hits.
    pub undetermined: u16,
    pub _pad: u16,
}

/// One side of the box score.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BbBoxTeam {
    /// Batting lines filled in `lines`, in batting order.
    pub n_lines: u8,
    /// Half-innings this side has batted (entries of `runs` and `hits` used, counting from
    /// `first_inning` of the box score).
    pub n_batted: u8,
    pub _pad: [u8; 2],
    pub totals: BbBoxTotals,
    pub lines: [BbBatLine; BB_BOX_MAX_BATTERS],
    /// Runs and hits in each inning, from the box score's `first_inning`.
    pub runs: [u8; BB_BOX_MAX_INNINGS],
    pub hits: [u8; BB_BOX_MAX_INNINGS],
}

/// The box score of a game so far, a pure function of its event log (data-formats §4;
/// computed when asked). Sides: 0 away, 1 home.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BbBoxScore {
    /// 1 once the log holds the end of the game.
    pub is_final: u8,
    /// Innings played or begun, and the first one in the linescore arrays (1 unless the game
    /// ran past `BB_BOX_MAX_INNINGS`).
    pub innings: u8,
    pub first_inning: u8,
    pub _pad: u8,
    pub teams: [BbBoxTeam; 2],
}

fn sat8(x: u32) -> u8 {
    x.min(255) as u8
}
fn sat16(x: u32) -> u16 {
    x.min(65535) as u16
}

/// The box score of the game so far: batting lines (name, position, AB, R, H, RBI, BB, K,
/// the extra-base hits and sacrifice flies), team totals with runners left on, and the
/// linescore with runs and hits by inning. Fills `out` and returns 1; 0 if the log cannot be
/// read; −1 on a null argument. `through_plays` counts the pitches to include (the play
/// index of a timeline is the number of pitches before it): a client a pitch behind the sim
/// asks for the pitches it has shown, and `u32::MAX` asks for all so far. It costs one pass over the log, so ask when it is shown, not
/// every frame.
///
/// # Safety
/// `g` must come from [`bb_game_create`]; `out` must be a valid pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bb_game_box_score(g: *const BbGame, through_plays: u32, out: *mut BbBoxScore) -> i32 {
    if g.is_null() || out.is_null() {
        return -1;
    }
    let Some(game) = unsafe { &*g }.game.as_ref() else { return -1 };
    let Ok(b) = bb_stats::box_score(&game.log_through(through_plays)) else { return 0 };
    let mut box_: BbBoxScore = unsafe { std::mem::zeroed() };
    box_.is_final = b.final_score.is_some() as u8;
    let longest = b.linescore[0].len().max(b.linescore[1].len());
    let first = longest.saturating_sub(BB_BOX_MAX_INNINGS);
    box_.innings = b.innings.min(255) as u8;
    box_.first_inning = (first + 1).min(255) as u8;
    for side in 0..2 {
        let t = &mut box_.teams[side];
        let line = |l: &bb_stats::BatterLine| BbBatLine {
            agent: l.agent,
            number: l.number,
            position: l.position,
            batting_order: l.batting_order,
            _pad: [0; 3],
            pa: sat8(l.pa),
            ab: sat8(l.ab),
            r: sat8(l.runs),
            h: sat8(l.h),
            doubles: sat8(l.doubles),
            triples: sat8(l.triples),
            hr: sat8(l.hr),
            rbi: sat8(l.rbi),
            bb: sat8(l.bb),
            k: sat8(l.k),
            hbp: sat8(l.hbp),
            sf: sat8(l.sf),
            sh: sat8(l.sh),
            _pad2: [0; 3],
            given: name_field(l.name.strip_suffix(l.family.as_str()).unwrap_or(&l.name).trim_end()),
            family: name_field(&l.family),
        };
        let n = b.batting[side].len().min(BB_BOX_MAX_BATTERS);
        t.n_lines = n as u8;
        for (dst, src) in t.lines.iter_mut().zip(&b.batting[side][..n]) {
            *dst = line(src);
        }
        let x = &b.teams[side];
        t.totals = BbBoxTotals {
            pa: sat16(x.pa),
            ab: sat16(x.ab),
            r: sat16(x.runs),
            h: sat16(x.h),
            doubles: sat16(x.doubles),
            triples: sat16(x.triples),
            hr: sat16(x.hr),
            rbi: sat16(x.rbi),
            bb: sat16(x.bb),
            k: sat16(x.k),
            hbp: sat16(x.hbp),
            sf: sat16(x.sf),
            sh: sat16(x.sh),
            lob: sat16(x.left_on_base),
            undetermined: sat16(x.undetermined),
            _pad: 0,
        };
        let runs = &b.linescore[side];
        let hits = &b.hits_by_inning[side];
        t.n_batted = runs.len().saturating_sub(first).min(BB_BOX_MAX_INNINGS) as u8;
        for i in 0..t.n_batted as usize {
            t.runs[i] = sat8(runs[first + i]);
            t.hits[i] = sat8(hits.get(first + i).copied().unwrap_or(0));
        }
    }
    unsafe { *out = box_ };
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plays_a_game_through_the_abi() {
        let g = bb_game_create(3);
        let mut n = 0;
        let mut dead = [0u32; 4];
        loop {
            let (mut p, mut len) = (std::ptr::null_mut(), 0usize);
            let r = unsafe { bb_game_advance(g, &mut p, &mut len) };
            if r != 1 {
                assert_eq!(r, 0);
                break;
            }
            let bytes = unsafe { std::slice::from_raw_parts(p, len) };
            let tl = flatbuffers::root::<bb_schema::tl::PlayTimeline>(bytes).expect("valid timeline");
            assert!(flatbuffers::buffer_has_identifier(bytes, "BBTL", false));
            assert!(tl.ball().unwrap().segments().unwrap().len() >= 1);
            // A ball that left the field: when and where it did, against the timeline's own
            // ball track (it ends there, within a sample), and why.
            let mut info = BbPlayInfo::default();
            assert_eq!(unsafe { bb_timeline_info(p, len, &mut info) }, 1);
            if info.dead_s >= 0.0 {
                dead[info.dead_reason as usize] += 1;
                assert!((1..=3).contains(&info.dead_reason) && info.dead_s <= info.duration_s + 1e-4, "{}", info.dead_s);
                let last = tl.ball().unwrap().segments().unwrap().iter().filter_map(|s| s.knots()).flat_map(|k| k.iter()).map(|k| k.t()).fold(0.0f32, f32::max);
                assert!(last >= info.dead_s - 0.05, "the track ends at {last} before the ball was dead at {}", info.dead_s);
                // Where it crossed: on the boundary (a ball radius inside a wall's plane, or on
                // the fence's curve), at or above the ground.
                let w = bb_physics::ground::Field::default().ball_clearance(info.dead_at[0] as f64, info.dead_at[1] as f64, 0.036888);
                assert!(w.dist.abs() < 2e-3 && info.dead_at[2] >= 0.0, "dead at {:?}: {w:?}", info.dead_at);
                if info.dead_reason == 2 {
                    // Over the 8-ft wall in flight, it would have come down beyond the fence.
                    let fence = bb_physics::ground::Field::default().fence_at((info.dead_at[0] as f64).atan2(info.dead_at[1] as f64));
                    assert!(info.projected_distance_m as f64 > fence, "home run projected {} m, fence {fence} m", info.projected_distance_m);
                }
            } else {
                assert_eq!((info.dead_reason, info.dead_at), (0, [0.0; 3]));
            }
            // A distance only with contact.
            assert_eq!(info.projected_distance_m > 0.0, info.contact_s >= 0.0, "projected {} contact {}", info.projected_distance_m, info.contact_s);
            unsafe { bb_blob_free(p, len) };
            n += 1;
        }
        let (mut a, mut h) = (0u16, 0u16);
        assert_eq!(unsafe { bb_game_score(g, &mut a, &mut h) }, 1);
        assert_ne!(a, h);
        unsafe { bb_game_free(g) };
        // Same seed through the plain API gives the same final score.
        let r = bb_game::game::simulate_game(3);
        assert_eq!((a, h), (r.runs[0], r.runs[1]));
        assert!(n > 150);
        // A game has about ten foul balls into the seats. (Home runs, when there are any, are
        // checked above; whether this one game has one changes with the game, and home runs
        // are covered over many games by `bb_game/tests/home_run_trot.rs`.)
        assert!(dead[1] >= 3, "dead balls by reason {dead:?}");
    }

    fn text(field: &[u8]) -> &str {
        std::ffi::CStr::from_bytes_until_nul(field).unwrap().to_str().unwrap()
    }

    // The roster read through the C ABI is the game's roster (data-formats §2.4): nine a
    // side in batting order, each found again under both of his agent ids; every play names
    // the batter it was thrown to, and a play that ends his plate appearance says his name.
    #[test]
    fn rosters_and_batters_cross_the_abi() {
        let seed = 7;
        let want = bb_game::roster::Rosters::generate(seed);
        let g = bb_game_create(seed);
        // SAFETY: an all-zero BbPlayer is a valid value (plain numbers and bytes).
        let mut p: BbPlayer = unsafe { std::mem::zeroed() };
        for side in 0..2u32 {
            let team = &want.teams[side as usize];
            for (slot, w) in team.batting_order().into_iter().enumerate() {
                assert_eq!(unsafe { bb_game_roster(g, side, slot as u32, &mut p) }, 1);
                assert_eq!((text(&p.given), text(&p.family)), (w.given, w.family));
                assert_eq!((p.player_id, p.side as u32, p.number, p.position, p.batting_order as usize), (w.player_id, side, w.number, w.position, slot + 1));
                assert_eq!((p.bat_agent, p.field_agent), (((side as u16) << 8) | (slot as u16 + 1), ((side as u16) << 8) | 0x80 | w.position as u16));
                // Hands as the header documents them: 1 left, 2 right, 3 a switch hitter.
                let bats = match w.bats {
                    bb_observe::Bats::Left => 1,
                    bb_observe::Bats::Right => 2,
                    bb_observe::Bats::Switch => 3,
                };
                assert_eq!((p.bats, p.throws), (bats, if w.throws == bb_observe::Hand::Left { 1 } else { 2 }));
                for id in [p.bat_agent, p.field_agent] {
                    let mut q: BbPlayer = unsafe { std::mem::zeroed() };
                    assert_eq!(unsafe { bb_game_player(g, id, &mut q) }, 1);
                    assert_eq!((q.player_id, text(&q.family)), (w.player_id, w.family));
                }
            }
        }
        // Not players, and bad arguments.
        assert_eq!(unsafe { bb_game_roster(g, 2, 0, &mut p) }, 0);
        assert_eq!(unsafe { bb_game_roster(g, 0, 9, &mut p) }, 0);
        assert_eq!(unsafe { bb_game_player(g, 0x0200, &mut p) }, 0);
        assert_eq!(unsafe { bb_game_player(g, 0x00FF, &mut p) }, 0);
        assert_eq!(unsafe { bb_game_roster(std::ptr::null(), 0, 0, &mut p) }, -1);
        assert_eq!(unsafe { bb_game_roster(g, 0, 0, std::ptr::null_mut()) }, -1);
        assert_eq!(unsafe { bb_game_player(g, 1, std::ptr::null_mut()) }, -1);
        // The first batters of the game, by the order: away 1, 2, 3, ... until the half ends.
        let (mut blob, mut len) = (std::ptr::null_mut(), 0usize);
        let (mut up, mut named) = (0usize, 0);
        while unsafe { bb_game_advance(g, &mut blob, &mut len) } == 1 && named < 3 {
            let mut info = BbPlayInfo::default();
            assert_eq!(unsafe { bb_timeline_info(blob, len, &mut info) }, 1);
            let batter = want.teams[0].batting_order()[up];
            assert_eq!(info.batter, up as u16 + 1);
            let summary = text(&info.summary);
            if summary.contains(':') {
                assert!(summary.starts_with(&format!("{}: ", batter.family)), "{summary}");
                up += 1;
                named += 1;
            }
            unsafe { bb_blob_free(blob, len) };
        }
        assert_eq!(named, 3);
        unsafe { bb_game_free(g) };
    }

    // What a viewer sees before the batter has decided must not depend on what he will
    // decide. Through his decision (the sim's commit tick, read from the game, not from the
    // timeline), the bat stays in the stance it had at the release, upright enough that its
    // horizontal reach stays inside the 45 cm at which the client turns his body after the
    // bat. The result is not announced before the umpire's call (the catcher has the ball,
    // plus the 0.2 s signal latency), nor contact measurements before contact. And on ball
    // four nobody on offense moves or turns until the call has reached them.
    #[test]
    fn nothing_is_given_away_before_the_decision_or_the_call() {
        let mut game = Game::new(11, true);
        let (mut swings, mut takes, mut walks, mut loads) = (0, 0, 0, 0);
        while let Some(step) = game.step_pitch() {
            let bytes = bb_game::timeline::encode(&step, &game.st, &game.rosters);
            let (p, len) = (bytes.as_ptr(), bytes.len());
            let rec = &step.record;
            let mut info = BbPlayInfo::default();
            assert_eq!(unsafe { bb_timeline_info(p, len, &mut info) }, 1);
            let text = std::ffi::CStr::from_bytes_until_nul(&info.summary).unwrap().to_str().unwrap().to_string();
            let t_pitch_end = step.pitch_flight.last().t as f32;
            match rec.contact_state {
                Some((tc, _)) => assert!(info.contact_s >= tc as f32 - 1e-6 && info.call_s >= tc as f32, "{text}: contact at {tc}, shown from {} / {}", info.contact_s, info.call_s),
                None => assert!(info.contact_s < 0.0 && info.call_s >= t_pitch_end + 0.2 - 1e-5, "{text}: call shown at {} s, the umpire's call reaches them at {}", info.call_s, t_pitch_end + 0.2),
            }
            let mut f0 = BbFrame::default();
            unsafe { bb_timeline_sample(p, len, 0.0, &mut f0) };
            let t_decide = rec.t_decide as f32;
            let mut t = 0.0;
            while t <= t_decide {
                let mut f = BbFrame::default();
                unsafe { bb_timeline_sample(p, len, t, &mut f) };
                assert_eq!(f.has_bat, 1);
                let a: [f32; 3] = core::array::from_fn(|j| f.bat_tip[j] - f.bat_knob[j]);
                assert!(a[0].hypot(a[1]) < 0.45 && a[2] > 0.7, "play {} ({text}) at {t:.3} s: bat out of the stance, axis {a:?}", info.play_index);
                assert!(f.bat_knob[2] > 1.0, "play {} at {t:.3} s: knob at {:?}", info.play_index, f.bat_knob);
                assert_eq!((f.bat_knob, f.bat_tip), (f0.bat_knob, f0.bat_tip), "play {} ({text}) at {t:.3} s, decision at {t_decide:.3} s: the stance moved", info.play_index);
                t += 0.0025;
            }
            if let Some(sw) = &rec.swing {
                swings += 1;
                // The swing starts less than the 50 ms latency after the decision: a shorter load.
                loads += (sw.t_start() - rec.t_decide < 0.05) as u32;
            } else {
                takes += 1;
            }
            // A line that ends a plate appearance starts with the batter's name.
            if text.split_once(": ").is_some_and(|x| x.1.starts_with("Walk")) {
                let t_call = t_pitch_end + 0.2;
                let mut t = 0.0;
                while t <= t_call {
                    let mut f = BbFrame::default();
                    unsafe { bb_timeline_sample(p, len, t, &mut f) };
                    for i in 0..f.n_actors as usize {
                        if f.actors[i].agent & 0x80 == 0 {
                            let (a, b) = (f.actors[i], f0.actors[i]);
                            let d = (a.pos[0] - b.pos[0]).hypot(a.pos[1] - b.pos[1]);
                            assert!(d < 0.01 && a.yaw == b.yaw, "{text}: agent {:#x} has moved {d:.2} m or turned to {} at {t:.2} s, before the call at {t_call:.2} s", a.agent, a.yaw);
                        }
                    }
                    t += 0.02;
                }
                walks += 1;
            }
        }
        // Enough of each, including swings that start inside the latency after the decision.
        assert!(swings > 50 && takes > 50 && walks >= 2 && loads > 5, "{swings} swings ({loads} short loads), {takes} takes, {walks} walks");
    }

    // A client samples timelines without parsing them. Everything checked here is a fact
    // about baseball geometry, not about the encoder: a pitch starts near the mound and
    // ends near the plate, nine fielders plus a batter are always on the field, and on a
    // ball in play the batter-runner goes somewhere.
    #[test]
    fn timelines_sample_to_plausible_frames() {
        let g = bb_game_create(5);
        let (mut plays, mut moved, mut contacts, mut awards) = (0, 0, 0, 0);
        let mut any_on = false;
        let (mut held, mut thrown, mut dives) = (0u32, 0u32, 0u32);
        let mut seen = std::collections::BTreeMap::new();
        // The scoreboard before each play, to count the outs it made.
        let (mut prev_half, mut prev_outs, mut calls_seen) = ((1u8, 1u8), 0u8, 0u32);
        loop {
            let (mut p, mut len) = (std::ptr::null_mut(), 0usize);
            if unsafe { bb_game_advance(g, &mut p, &mut len) } != 1 {
                break;
            }
            let mut info = BbPlayInfo::default();
            assert_eq!(unsafe { bb_timeline_info(p, len, &mut info) }, 1);
            assert!(info.duration_s > 0.3 && info.duration_s < 60.0 && info.outs <= 3 && info.inning >= 1 && info.bases < 8);
            any_on |= info.bases != 0;
            // The umpire calls each out the play made, in order, while the clip is running.
            let made = if (info.inning, info.top) == prev_half { info.outs - prev_outs } else { 3 - prev_outs };
            assert_eq!(info.n_calls, made as u32, "play {}: {made} outs, {} calls", info.play_index, info.n_calls);
            let calls = &info.calls[..info.n_calls as usize];
            assert!(calls.windows(2).all(|w| w[0].t_s <= w[1].t_s) && calls.iter().all(|c| c.t_s > 0.3 && c.t_s <= info.duration_s && c.base <= 4));
            calls_seen += info.n_calls;
            // (After a third out the scoreboard already shows the next half inning.)
            (prev_half, prev_outs) = ((info.inning, info.top), info.outs);
            let text = std::ffi::CStr::from_bytes_until_nul(&info.summary).unwrap().to_str().unwrap();
            assert!(!text.is_empty() && text.is_ascii(), "summary {text:?}");
            // Tallied without the batter's name, which starts a line that ends his plate appearance.
            *seen.entry(text.split_once(": ").map_or(text, |x| x.1).split(',').next().unwrap().to_string()).or_insert(0u32) += 1;
            let (mut f0, mut f1, mut fe) = (BbFrame::default(), BbFrame::default(), BbFrame::default());
            unsafe {
                assert_eq!(bb_timeline_sample(p, len, 0.0, &mut f0), 1);
                assert_eq!(bb_timeline_sample(p, len, 0.35, &mut f1), 1);
                assert_eq!(bb_timeline_sample(p, len, info.duration_s, &mut fe), 1);
            }
            // Release: 15–18 m from the plate, 1–2.5 m up. A third of a second later the
            // ball is most of the way home.
            assert!(f0.ball[1] > 15.0 && f0.ball[1] < 18.5 && f0.ball[2] > 1.0 && f0.ball[2] < 2.5, "release at {:?}", f0.ball);
            assert!(f1.ball[1] < 6.0, "0.35 s after release the ball is at y = {}", f1.ball[1]);
            assert!(f0.n_actors >= 10 && f0.n_actors <= 13, "{} actors", f0.n_actors);
            // A bat is in the batter's hands as the pitch is thrown: 34 inches long, held
            // within arm's reach of the batter (his swing pivot is about 0.95 m off the
            // plate's centre line; an arm's length more is about 1.7 m: 1.75 m allowed; the
            // swing's starting pose varies with where it is aimed, and one starts with the
            // knob 1.61 m out), off the ground.
            assert_eq!(f0.has_bat, 1);
            let bat_len = (0..3).map(|j| (f0.bat_tip[j] - f0.bat_knob[j]).powi(2)).sum::<f32>().sqrt();
            assert!((bat_len - 0.8636).abs() < 1e-3, "bat length {bat_len}");
            assert!(f0.bat_knob[0].abs() < 1.75 && f0.bat_knob[1].abs() < 1.6 && f0.bat_knob[2] > 0.3 && f0.bat_knob[2] < 2.0, "knob at {:?}", f0.bat_knob);
            if info.exit_velocity_mps > 0.0 {
                // On contact the barrel is where the ball is: some point of the bat is within
                // a bat-plus-ball radius (7 cm, with slack for 5 ms sampling at 30 m/s) of
                // the ball at some instant near the plate.
                let mut best = f32::MAX;
                let mut t = 0.30;
                while t < 0.60 {
                    let mut f = BbFrame::default();
                    unsafe { bb_timeline_sample(p, len, t, &mut f) };
                    if f.has_bat == 1 {
                        for s in 0..=20 {
                            let u = s as f32 / 20.0;
                            let d = (0..3).map(|j| (f.bat_knob[j] + u * (f.bat_tip[j] - f.bat_knob[j]) - f.ball[j]).powi(2)).sum::<f32>().sqrt();
                            best = best.min(d);
                        }
                    }
                    t += 0.001;
                }
                assert!(best < 0.25, "bat never came within {best} m of the ball on a batted ball");
                contacts += 1;
            }
            let fielders = f0.actors[..f0.n_actors as usize].iter().filter(|a| a.agent & 0x80 != 0).count();
            assert_eq!(fielders, 9);
            // Nobody on offense moves faster than a sprinter between samples: no jumps (the
            // batter runs out of his box; a runner on a walk starts from his lead).
            let mut prev: Option<BbFrame> = None;
            let mut t = 0.0;
            while t <= info.duration_s {
                let mut f = BbFrame::default();
                unsafe { bb_timeline_sample(p, len, t, &mut f) };
                if let Some(q) = prev {
                    for i in 0..f.n_actors as usize {
                        if f.actors[i].agent & 0x80 == 0 {
                            let d = (f.actors[i].pos[0] - q.actors[i].pos[0]).hypot(f.actors[i].pos[1] - q.actors[i].pos[1]);
                            assert!(d / 0.05 < 13.0, "agent {:#x} moved {d:.2} m in 0.05 s at {t:.2} s of play {} ({text})", f.actors[i].agent, info.play_index);
                        }
                    }
                }
                prev = Some(f);
                t += 0.05;
            }
            // Possession: whoever is reported holding the ball is a fielder on the field with
            // the ball about him, and a throw reported as coming does leave him.
            let mut t = 0.0;
            while t <= info.duration_s {
                let mut f = BbFrame::default();
                unsafe { bb_timeline_sample(p, len, t, &mut f) };
                if f.holder >= 0 {
                    let near = |f: &BbFrame| {
                        let a = f.actors[..f.n_actors as usize].iter().find(|a| a.agent as i32 == f.holder).expect("the holder is on the field");
                        assert!(a.agent & 0x80 != 0, "holder {:#x} is not a fielder", a.agent);
                        (0..3).map(|j| (f.ball[j] - a.pos[j] - if j == 2 { 1.2 } else { 0.0 }).powi(2)).sum::<f32>().sqrt()
                    };
                    assert!(near(&f) < 2.0, "play {} at {t:.2} s: the ball is {} m from its holder", info.play_index, near(&f));
                    held += 1;
                    if f.release_in_s > 0.0 && f.release_in_s < 0.1 && t + f.release_in_s + 0.3 < info.duration_s {
                        let mut g = BbFrame::default();
                        unsafe { bb_timeline_sample(p, len, t + f.release_in_s + 0.3, &mut g) };
                        g.holder = f.holder;
                        assert!(near(&g) > 3.0, "play {} at {t:.2} s: the announced throw did not leave", info.play_index);
                        thrown += 1;
                    }
                } else {
                    assert_eq!(f.release_in_s, -1.0);
                }
                t += 0.05;
            }
            // Dives: the diver is a fielder on the field, down for the sim's get-up time; his
            // lunge ends within 0.35 s, and after it he lies where he landed until he is up
            // (to within one 0.1 s track sample, which blends in his first step).
            let mut t = 0.0;
            let mut lying: Vec<(u16, [f32; 3])> = Vec::new();
            while t <= info.duration_s {
                let mut f = BbFrame::default();
                unsafe { bb_timeline_sample(p, len, t, &mut f) };
                for d in &f.dives[..f.n_dives as usize] {
                    assert!(d.agent & 0x80 != 0, "diver {:#x} is not a fielder", d.agent);
                    assert!((d.since_s + d.up_in_s - 1.5).abs() < 1e-3, "down {} + {} s", d.since_s, d.up_in_s);
                    let n = d.dir[0].hypot(d.dir[1]);
                    assert!(n == 0.0 || (n - 1.0).abs() < 1e-4, "dive direction {:?}", d.dir);
                    let a = f.actors[..f.n_actors as usize].iter().find(|a| a.agent == d.agent).expect("the diver is on the field");
                    if d.since_s > 0.45 && d.up_in_s > 0.1 {
                        match lying.iter().find(|l| l.0 == d.agent) {
                            Some(l) => assert!((a.pos[0] - l.1[0]).hypot(a.pos[1] - l.1[1]) < 1e-3, "play {} at {t:.2} s: {:#x} moved while down", info.play_index, d.agent),
                            None => lying.push((d.agent, a.pos)),
                        }
                    }
                    dives += 1;
                }
                lying.retain(|l| f.dives[..f.n_dives as usize].iter().any(|d| d.agent == l.0));
                t += 0.05;
            }
            // Every dive is over before the clip ends: a play that ended during one lasts until
            // the diver is up (Codex dive animation review #3).
            let mut f = BbFrame::default();
            unsafe { bb_timeline_sample(p, len, info.duration_s, &mut f) };
            assert!(f.dives[..f.n_dives as usize].iter().all(|d| d.up_in_s < 1e-3), "play {}: the clip ends with a fielder still down", info.play_index);
            // The ball never goes under the turf, at any instant of any play.
            let mut t = 0.0;
            while t <= info.duration_s {
                let mut f = BbFrame::default();
                unsafe { bb_timeline_sample(p, len, t, &mut f) };
                assert!(f.ball[2] > -0.005, "ball at height {} m, {t} s into play {}", f.ball[2], info.play_index);
                t += 0.004;
            }
            // Whoever is credited with a walk or a home run ends the clip on his base.
            // A line that ends a plate appearance starts with the batter's name.
            let text = text.split_once(": ").map_or(text, |x| x.1);
            if text.starts_with("Walk") || text.starts_with("Home run") {
                let batter = fe.actors[..fe.n_actors as usize].iter().filter(|a| a.agent & 0x80 == 0).map(|a| (a.pos[0].hypot(a.pos[1]), a.pos)).collect::<Vec<_>>();
                let want: [f32; 2] = if text.starts_with("Walk") { [19.4, 19.4] } else { [0.0, 0.0] };
                assert!(batter.iter().any(|(_, p)| (p[0] - want[0]).hypot(p[1] - want[1]) < 0.3), "{text}: nobody ends at {want:?}: {batter:?}");
                awards += 1;
            }
            for a in &fe.actors[..fe.n_actors as usize] {
                assert!(a.pos.iter().all(|x| x.is_finite()) && a.pos[0].hypot(a.pos[1]) < 140.0);
            }
            if info.exit_velocity_mps > 0.0 && info.duration_s > 3.0 {
                let far = (0..f0.n_actors as usize).any(|i| {
                    let (a, b) = (f0.actors[i], fe.actors[i]);
                    a.agent & 0x80 == 0 && (a.pos[0] - b.pos[0]).hypot(a.pos[1] - b.pos[1]) > 10.0
                });
                moved += far as u32;
            }
            plays += 1;
            unsafe { bb_blob_free(p, len) };
        }
        assert!(held > 100 && thrown > 10, "held {held} frames, {thrown} throws");
        assert!(dives > 20, "{dives} frames with a fielder down from a dive");
        // A whole game has the ordinary calls in it.
        for call in ["Ball", "Called strike", "Foul", "Walk"] {
            assert!(seen.contains_key(call), "no {call:?} in {seen:?}");
        }
        assert!(seen.keys().any(|k| k.starts_with("Ground out")) && seen.keys().any(|k| k.starts_with("Single")), "{seen:?}");
        assert!(any_on);
        assert!(calls_seen >= 51, "{calls_seen} out calls in a game");
        assert!(contacts > 30 && awards > 3, "{contacts} batted balls, {awards} walks and home runs");
        assert!(plays > 150 && moved > 20, "{plays} pitches, {moved} with a runner covering 10 m");
        // Bad input is refused, not read.
        let junk = [0u8; 64];
        let mut f = BbFrame::default();
        assert_eq!(unsafe { bb_timeline_sample(junk.as_ptr(), junk.len(), 0.0, &mut f) }, -1);
        assert_eq!(unsafe { bb_timeline_sample(std::ptr::null(), 0, 0.0, &mut f) }, -1);
        assert_eq!(unsafe { bb_timeline_sample(junk.as_ptr(), 7, 0.0, &mut f) }, -1);
        let mut info = BbPlayInfo::default();
        assert_eq!(unsafe { bb_timeline_info(junk.as_ptr(), 7, &mut info) }, -1);
        unsafe { bb_game_free(g) };
    }

    // Codex dive animation review #2: a second dive that starts the instant the fielder is up
    // from the first is reported, with its own direction, not paired with the first get-up.
    #[test]
    fn a_dive_starting_as_the_last_one_ends_is_reported() {
        let mk = |up: bool, t: f32, p: [f32; 3]| DiveMark { up, t, agent: 0x186, p };
        let marks = [mk(false, 1.0, [0.0, 30.0, 0.0]), mk(true, 2.5, [1.0, 30.0, 0.0]), mk(false, 2.5, [1.0, 30.0, 0.0]), mk(true, 4.0, [1.0, 31.5, 0.0])];
        for t in [2.5f32, 3.0, 3.9] {
            let d = active_dives(&marks, t);
            assert_eq!(d.len(), 1, "at {t}: {} dives", d.len());
            assert_eq!(d[0].dir, [0.0, 1.0], "at {t}: the second dive's direction");
            assert!((d[0].since_s - (t - 2.5)).abs() < 1e-6 && (d[0].up_in_s - (4.0 - t)).abs() < 1e-6);
        }
        assert_eq!(active_dives(&marks, 1.2)[0].dir, [1.0, 0.0]);
        assert!(active_dives(&marks, 4.0).is_empty());
    }
    /// The box score across the ABI is `bb_stats::box_score` of the same log (a pure function
    /// of the log, computed when asked), however often it is asked, and it adds up: lines to
    /// totals, innings to the score, and the same numbers at the end as from the finished game.
    #[test]
    fn the_box_score_is_the_stats_crates_reading_of_the_log() {
        for seed in [3u64, 11] {
            let g = bb_game_create(seed);
            let mut asked_midway = None;
            let (mut blob, mut len) = (std::ptr::null_mut(), 0usize);
            let mut n = 0;
            while unsafe { bb_game_advance(g, &mut blob, &mut len) } == 1 {
                unsafe { bb_blob_free(blob, len) };
                n += 1;
                if n == 90 {
                    let mut a: BbBoxScore = unsafe { std::mem::zeroed() };
                    let mut b: BbBoxScore = unsafe { std::mem::zeroed() };
                    assert_eq!(unsafe { bb_game_box_score(g, u32::MAX, &mut a) }, 1);
                    assert_eq!(unsafe { bb_game_box_score(g, u32::MAX, &mut b) }, 1);
                    // Asking twice gives the same answer (no state is kept or advanced).
                    assert_eq!(unsafe { std::slice::from_raw_parts(&a as *const _ as *const u8, std::mem::size_of::<BbBoxScore>()) }, unsafe { std::slice::from_raw_parts(&b as *const _ as *const u8, std::mem::size_of::<BbBoxScore>()) });
                    assert_eq!(a.is_final, 0);
                    asked_midway = Some(a);
                }
            }
            let mut f: BbBoxScore = unsafe { std::mem::zeroed() };
            assert_eq!(unsafe { bb_game_box_score(g, u32::MAX, &mut f) }, 1);
            assert_eq!(f.is_final, 1);
            let (mut away, mut home) = (0u16, 0u16);
            assert_eq!(unsafe { bb_game_score(g, &mut away, &mut home) }, 1);
            // The same game played to the end by the library's own driver, read directly.
            let log = bb_game::game::simulate_game(seed).log;
            let want = bb_stats::box_score(&log).unwrap();
            for side in 0..2 {
                let t = &f.teams[side];
                let runs: u32 = t.runs[..t.n_batted as usize].iter().map(|&x| x as u32).sum();
                let hits: u32 = t.hits[..t.n_batted as usize].iter().map(|&x| x as u32).sum();
                assert_eq!(runs, if side == 0 { away } else { home } as u32, "seed {seed} side {side}");
                assert_eq!(t.n_batted as usize, want.linescore[side].len());
                assert_eq!(t.totals.r as u32, want.teams[side].runs);
                assert_eq!(hits, want.teams[side].h);
                // Lines add up to the totals.
                let lines = &t.lines[..t.n_lines as usize];
                assert_eq!(lines.len(), 9);
                assert_eq!(lines.iter().map(|l| l.ab as u32).sum::<u32>(), t.totals.ab as u32);
                assert_eq!(lines.iter().map(|l| l.h as u32).sum::<u32>(), hits);
                assert_eq!(lines.iter().map(|l| l.r as u32).sum::<u32>(), runs);
                assert_eq!(lines.iter().map(|l| l.rbi as u32).sum::<u32>(), t.totals.rbi as u32);
                for (l, w) in lines.iter().zip(&want.batting[side]) {
                    assert_eq!((l.agent, l.ab as u32, l.h as u32, l.rbi as u32, l.bb as u32, l.k as u32), (w.agent, w.ab, w.h, w.rbi, w.bb, w.k));
                    assert_eq!(text(&l.family), w.family);
                    assert_eq!(format!("{} {}", text(&l.given), text(&l.family)), w.name);
                }
            }
            // Midway the lines can only be shorter than at the end.
            let m = asked_midway.expect("asked at pitch 90");
            for side in 0..2 {
                assert!(m.teams[side].totals.pa <= f.teams[side].totals.pa);
                assert!(m.teams[side].n_batted <= f.teams[side].n_batted);
            }
            unsafe { bb_game_free(g) };
        }
        let mut x: BbBoxScore = unsafe { std::mem::zeroed() };
        assert_eq!(unsafe { bb_game_box_score(std::ptr::null(), 0, &mut x) }, -1);
    }
    /// `through_plays` shows a client only what it has shown: the plate appearances in a box
    /// score through k pitches are exactly those ended in the first k plays of the log.
    #[test]
    fn the_box_score_stops_where_asked() {
        let g = bb_game_create(5);
        let (mut blob, mut len) = (std::ptr::null_mut(), 0usize);
        while unsafe { bb_game_advance(g, &mut blob, &mut len) } == 1 {
            unsafe { bb_blob_free(blob, len) };
        }
        let log = bb_game::game::simulate_game(5).log;
        let view = bb_schema::log::read(&log).unwrap();
        for k in [0u32, 1, 37, 120, 10_000] {
            let mut b: BbBoxScore = unsafe { std::mem::zeroed() };
            assert_eq!(unsafe { bb_game_box_score(g, k, &mut b) }, 1);
            let ended = view.plays.iter().take(k as usize).filter(|p| p.events().unwrap().iter().any(|e| e.body_as_plate_appearance_ended().is_some())).count();
            let pa: usize = b.teams.iter().map(|t| t.totals.pa as usize).sum();
            assert_eq!(pa, ended, "through {k} pitches");
            assert_eq!(b.teams[0].n_lines, 9, "the lineups are there before a pitch");
            assert_eq!(b.is_final == 1, k as usize >= view.plays.len());
        }
        unsafe { bb_game_free(g) };
    }
}
