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
    Box::into_raw(Box::new(BbGame { game: Some(Game::new(seed, false)) }))
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
}
