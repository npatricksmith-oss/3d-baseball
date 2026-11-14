//! M0 exit gate on real games (PLAN §14, data-formats §4.4): the same seed gives the same
//! game digest, run to run and thread to thread; the log reads back and verifies.

use bb_game::game::simulate_game;

#[test]
fn same_seed_same_digest_across_threads() {
    let a = simulate_game(42);
    let b = std::thread::spawn(|| simulate_game(42)).join().unwrap();
    assert_eq!(a.digest, b.digest);
    assert_eq!(a.log, b.log);
    assert_ne!(a.digest, simulate_game(43).digest);
}

#[test]
fn game_log_reads_back_and_verifies() {
    let g = simulate_game(5);
    let v = bb_schema::log::read(&g.log).expect("valid log");
    assert_eq!(v.game_digest, g.digest);
    assert_eq!(v.plays.len() as u32, g.pitches);
    // The last record carries the GameEnded event with the final score.
    let last = v.plays.last().unwrap().events().unwrap();
    let ended = last.iter().find_map(|e| e.body_as_game_ended()).expect("GameEnded in the last play");
    assert_eq!((ended.score_away() as u16, ended.score_home() as u16), (g.runs[0], g.runs[1]));
}

#[test]
fn recording_does_not_change_the_game() {
    // data-formats §5.5: visualization recording must never change sim behavior.
    let plain = simulate_game(9);
    let recorded = bb_game::game::simulate_game_with(9, true);
    assert_eq!(plain.digest, recorded.digest);
    assert!(!recorded.viz.is_empty());
}

#[test]
fn choice_evidence_does_not_change_the_game_or_leak_between_captures() {
    use bb_game::live::choice_trace::capture;
    // PLAN §4.2: a diagnostic observer must preserve the entire deterministic log.
    let plain = simulate_game(7);
    let (traced, choices) = capture(|| simulate_game(7));
    assert_eq!(plain.log, traced.log);
    assert_eq!(plain.digest, traced.digest);
    assert!(!choices.is_empty());
    let (_, empty) = capture(|| ());
    assert!(empty.is_empty());
    // The sink is thread-local, like keyed diagnostic profiling, never global state.
    let (other, local) = capture(|| std::thread::spawn(|| simulate_game(7)).join().unwrap());
    assert_eq!(plain.log, other.log);
    assert!(local.is_empty());
}
