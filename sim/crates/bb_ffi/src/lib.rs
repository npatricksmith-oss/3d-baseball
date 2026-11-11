//! C ABI for engine clients (data-formats §6.4, PLAN §3.1). Opaque handles; every payload is
//! a FlatBuffers blob owned by Rust and released with [`bb_blob_free`].
//!
//! M4 skeleton surface (the M0.8 Unreal spike decides the final list):
//! `bb_game_create`, `bb_game_advance`, `bb_game_score`, `bb_blob_free`, `bb_game_free`.
//! The header is `include/bb_ffi.h`.
#![allow(unsafe_code)]

use bb_game::game::Game;

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
            let bytes = bb_game::timeline::encode(&step, &game.st).into_boxed_slice();
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
    pub _pad: u8,
    pub release_speed_mps: f32,
    pub exit_velocity_mps: f32,
    pub launch_angle_rad: f32,
    /// The play in words, NUL-terminated UTF-8 (ASCII in practice), truncated to fit.
    pub summary: [u8; BB_SUMMARY_LEN],
}

/// Size of [`BbPlayInfo::summary`], including the terminating NUL.
pub const BB_SUMMARY_LEN: usize = 96;

impl Default for BbPlayInfo {
    fn default() -> Self {
        // SAFETY: every field is a plain number or byte array; all-zero is a valid value.
        unsafe { std::mem::zeroed() }
    }
}

fn timeline<'a>(blob: *const u8, len: usize) -> Option<bb_schema::tl::PlayTimeline<'a>> {
    if blob.is_null() {
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
    let mut info = BbPlayInfo { play_index: tl.play_index(), duration_s: tl.duration_s(), ..Default::default() };
    if let Some(v) = tl.view_after() {
        info.inning = v.inning();
        info.top = v.top() as u8;
        info.outs = v.outs();
        info.balls = v.balls();
        info.strikes = v.strikes();
        info.away_runs = v.away().map(|t| t.runs()).unwrap_or(0);
        info.home_runs = v.home().map(|t| t.runs()).unwrap_or(0);
    }
    if let Some(t) = tl.telemetry() {
        info.release_speed_mps = t.release_speed_mps();
        info.exit_velocity_mps = t.exit_velocity_mps();
        info.launch_angle_rad = t.launch_angle_rad();
    }
    if let Some(s) = tl.summary() {
        let n = s.len().min(BB_SUMMARY_LEN - 1);
        info.summary[..n].copy_from_slice(&s.as_bytes()[..n]);
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
    let mut frame = BbFrame::default();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plays_a_game_through_the_abi() {
        let g = bb_game_create(3);
        let mut n = 0;
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
    }

    // A client samples timelines without parsing them. Everything checked here is a fact
    // about baseball geometry, not about the encoder: a pitch starts near the mound and
    // ends near the plate, nine fielders plus a batter are always on the field, and on a
    // ball in play the batter-runner goes somewhere.
    #[test]
    fn timelines_sample_to_plausible_frames() {
        let g = bb_game_create(5);
        let (mut plays, mut moved, mut contacts) = (0, 0, 0);
        let mut seen = std::collections::BTreeMap::new();
        loop {
            let (mut p, mut len) = (std::ptr::null_mut(), 0usize);
            if unsafe { bb_game_advance(g, &mut p, &mut len) } != 1 {
                break;
            }
            let mut info = BbPlayInfo::default();
            assert_eq!(unsafe { bb_timeline_info(p, len, &mut info) }, 1);
            assert!(info.duration_s > 0.3 && info.duration_s < 60.0 && info.outs <= 3 && info.inning >= 1);
            let text = std::ffi::CStr::from_bytes_until_nul(&info.summary).unwrap().to_str().unwrap();
            assert!(!text.is_empty() && text.is_ascii(), "summary {text:?}");
            *seen.entry(text.split(',').next().unwrap().to_string()).or_insert(0u32) += 1;
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
            // within arm's reach of the plate, off the ground.
            assert_eq!(f0.has_bat, 1);
            let bat_len = (0..3).map(|j| (f0.bat_tip[j] - f0.bat_knob[j]).powi(2)).sum::<f32>().sqrt();
            assert!((bat_len - 0.8636).abs() < 1e-3, "bat length {bat_len}");
            assert!(f0.bat_knob[0].abs() < 1.6 && f0.bat_knob[1].abs() < 1.6 && f0.bat_knob[2] > 0.3 && f0.bat_knob[2] < 2.0, "knob at {:?}", f0.bat_knob);
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
        // A whole game has the ordinary calls in it.
        for call in ["Ball", "Called strike", "Foul", "Walk"] {
            assert!(seen.contains_key(call), "no {call:?} in {seen:?}");
        }
        assert!(seen.keys().any(|k| k.starts_with("Ground out")) && seen.keys().any(|k| k.starts_with("Single")), "{seen:?}");
        assert!(contacts > 30);
        assert!(plays > 150 && moved > 20, "{plays} pitches, {moved} with a runner covering 10 m");
        // Bad input is refused, not read.
        let junk = [0u8; 64];
        let mut f = BbFrame::default();
        assert_eq!(unsafe { bb_timeline_sample(junk.as_ptr(), junk.len(), 0.0, &mut f) }, -1);
        assert_eq!(unsafe { bb_timeline_sample(std::ptr::null(), 0, 0.0, &mut f) }, -1);
        unsafe { bb_game_free(g) };
    }
}
