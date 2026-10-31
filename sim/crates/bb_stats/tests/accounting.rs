//! Accounting identities over real games (rules-spec §11), computed only from event logs.

use bb_game::game::simulate_game;
use bb_stats::box_score;

#[test]
fn identities_hold_over_many_games() {
    let games: Vec<_> = std::thread::scope(|s| {
        let hs: Vec<_> = (0..8u64).map(|k| s.spawn(move || (0..10u64).map(|i| simulate_game(1000 + k * 10 + i)).collect::<Vec<_>>())).collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    for g in &games {
        let b = box_score(&g.log).expect("log verifies");
        // Runs from RunScored events = linescore sums = final score = the sim's own tally.
        let (fa, fh) = b.final_score.expect("game ended");
        assert_eq!((b.teams[0].runs, b.teams[1].runs), (fa, fh));
        assert_eq!(b.linescore[0].iter().sum::<u32>(), fa);
        assert_eq!(b.linescore[1].iter().sum::<u32>(), fh);
        assert_eq!((fa as u16, fh as u16), (g.runs[0], g.runs[1]));
        // Every completed half-inning has exactly three outs; the last may end early
        // (walk-off, or the home team not batting is simply absent).
        let n = b.outs_by_half.len();
        for (i, &o) in b.outs_by_half.iter().enumerate() {
            if i + 1 < n {
                assert_eq!(o, 3, "half-inning {i} had {o} outs");
            } else {
                assert!(o <= 3);
            }
        }
        // PA = K + BB + HBP + HR + batted balls; PA matches the sim's own count.
        for t in &b.teams {
            assert_eq!(t.pa, t.k + t.bb + t.hbp + t.hr + t.batted);
        }
        assert_eq!(b.teams[0].pa + b.teams[1].pa, g.pa);
        // Pitches seen = log records.
        assert_eq!(b.teams[0].pitches_seen + b.teams[1].pitches_seen, g.pitches);
        // No ties, and the winner is consistent with the game-end rules.
        assert_ne!(fa, fh);
        assert!(b.innings >= 9);
    }
}
