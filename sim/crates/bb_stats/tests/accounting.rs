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

/// Per-player batting lines (data-formats §2.4): the names come from the log's header, and
/// the lines obey what a batting order forces, whatever happened in the game.
#[test]
fn batting_lines_name_the_roster_and_follow_the_order() {
    let (mut walk_offs, mut extras) = (0, 0);
    // Forty games, and on until both awkward endings have been seen (a walk-off comes in
    // about one game in ten, so forty can miss it; capped at 200).
    for seed in 0..200u64 {
        if seed >= 40 && walk_offs >= 1 && extras >= 1 {
            break;
        }
        let g = simulate_game(seed);
        let b = box_score(&g.log).expect("log verifies");
        walk_offs += g.walk_off as u32;
        extras += (g.innings > 9) as u32;
        let rosters = bb_game::roster::Rosters::generate(seed);
        for side in 0..2 {
            let lines = &b.batting[side];
            let want = rosters.teams[side].batting_order();
            assert_eq!(lines.len(), 9);
            for (slot, (l, p)) in lines.iter().zip(want).enumerate() {
                assert_eq!((l.batting_order as usize, l.name.as_str(), l.family.as_str(), l.number, l.position), (slot + 1, format!("{} {}", p.given, p.family).as_str(), p.family, p.number, p.position));
                assert_eq!(l.position_code(), ["P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"][p.position as usize - 1]);
                assert_eq!(l.pa, l.k + l.bb + l.hbp + l.hr + l.batted);
            }
            // The order goes round: nobody bats again until the other eight have, so plate
            // appearances never rise down the order and differ by at most one.
            for w in lines.windows(2) {
                assert!(w[0].pa >= w[1].pa, "seed {seed}: {:?}", lines.iter().map(|l| l.pa).collect::<Vec<_>>());
            }
            assert!(lines[0].pa - lines[8].pa <= 1);
            // Each man's line against the sim's own rules events, counted by the id on the
            // event (its high byte is the side, its low byte the batting slot), not by which
            // half-inning the box score thinks it is: a run credited to a teammate, or to the
            // other side across a half-inning change or a walk-off, shows here.
            for l in lines {
                let slot_id = ((side as u16) << 8) | l.batting_order as u16;
                let runs = g.events.iter().filter(|e| matches!(e, bb_rules::Event::RunScored { runner } if *runner == slot_id)).count() as u32;
                let pa = g.events.iter().filter(|e| matches!(e, bb_rules::Event::PlateAppearanceEnded { batter, .. } if *batter == slot_id)).count() as u32;
                let hr = g.events.iter().filter(|e| matches!(e, bb_rules::Event::PlateAppearanceEnded { batter, result: bb_rules::PaResult::HomeRun } if *batter == slot_id)).count() as u32;
                assert_eq!((l.agent, l.runs, l.pa, l.hr), (slot_id, runs, pa, hr), "seed {seed}: {}", l.name);
            }
            // The players' lines add up to the team's.
            let t = &b.teams[side];
            let sum = |f: fn(&bb_stats::BatterLine) -> u32| lines.iter().map(f).sum::<u32>();
            assert_eq!((sum(|l| l.pa), sum(|l| l.runs), sum(|l| l.k), sum(|l| l.bb), sum(|l| l.hbp), sum(|l| l.hr), sum(|l| l.batted)), (t.pa, t.runs, t.k, t.bb, t.hbp, t.hr, t.batted), "seed {seed} side {side}");
        }
    }
    // The games include the awkward endings.
    assert!(walk_offs >= 1 && extras >= 1, "{walk_offs} walk-offs, {extras} extra-inning games");
}
