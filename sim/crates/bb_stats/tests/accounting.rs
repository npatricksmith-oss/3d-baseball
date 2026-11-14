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
        for (side, t) in b.teams.iter().enumerate() {
            assert_eq!(t.pa, t.k + t.bb + t.hbp + t.hr + t.batted);
            assert_eq!(t.pa, t.ab + t.bb + t.hbp + t.sf + t.sh, "I2 seed side {side}");
            let outs = b.outs_by_half.iter().skip(side).step_by(2).sum::<u32>();
            assert_eq!(t.pa, t.runs + t.left_on_base + outs, "I1 seed side {side}, line {t:?}");
            assert_eq!(t.h, t.singles + t.doubles + t.triples + t.hr, "I7");
            assert_eq!(t.undetermined, 0, "unresolved rulings");
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
    for seed in 0..40u64 {
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
                assert_eq!((l.agent, l.runs, l.pa), (slot_id, runs, pa), "seed {seed}: {}", l.name);
                assert!(l.hr >= hr, "inside-the-park HR also count");
                assert_eq!(l.pa, l.ab + l.bb + l.hbp + l.sf + l.sh);
            }
            // The players' lines add up to the team's.
            let t = &b.teams[side];
            let sum = |f: fn(&bb_stats::BatterLine) -> u32| lines.iter().map(f).sum::<u32>();
            assert_eq!((sum(|l| l.pa), sum(|l| l.runs), sum(|l| l.k), sum(|l| l.bb), sum(|l| l.hbp), sum(|l| l.hr), sum(|l| l.batted)), (t.pa, t.runs, t.k, t.bb, t.hbp, t.hr, t.batted), "seed {seed} side {side}");
            assert_eq!((sum(|l| l.ab), sum(|l| l.h), sum(|l| l.singles), sum(|l| l.doubles), sum(|l| l.triples), sum(|l| l.rbi), sum(|l| l.sf)), (t.ab, t.h, t.singles, t.doubles, t.triples, t.rbi, t.sf));
        }
    }
    // The games include the awkward endings.
    assert!(walk_offs >= 1 && extras >= 1, "{walk_offs} walk-offs, {extras} extra-inning games");
}

/// Log-only checks for the pitch and pitcher accounting identities. There is one pitcher
/// per team today; the pitcher on the deciding pitch is charged with that PA and its outs.
#[test]
fn pitch_and_pitcher_accounting_over_games() {
    use bb_schema::{ev, log};
    for seed in 0..40 {
        let game = simulate_game(seed);
        let b = box_score(&game.log).unwrap();
        let view = log::read(&game.log).unwrap();
        let mut batting_side = 0usize;
        let mut faced = [0u32; 2];
        let mut pitcher_outs = [0u32; 2];
        let mut pitches = [0u32; 2];
        let mut classified = [0u32; 2];
        for play in &view.plays {
            let events = play.events().unwrap();
            let pitch = events.iter().find_map(|e| e.body_as_pitch_released()).unwrap();
            // In this sim fielding agent IDs encode the defending side in the high byte.
            let defense = (pitch.pitcher() >> 8) as usize;
            assert!(defense < 2);
            pitches[defense] += 1;
            for e in events.iter() {
                match e.body_type() {
                    ev::EventBody::HalfInningStarted => batting_side = usize::from(e.body_as_half_inning_started().unwrap().half() == ev::HalfInning::Bottom),
                    ev::EventBody::PitchResult => {
                        let outcome = e.body_as_pitch_result().unwrap().outcome();
                        assert!(matches!(outcome, ev::PitchOutcome::Ball | ev::PitchOutcome::CalledStrike | ev::PitchOutcome::SwingingStrike | ev::PitchOutcome::Foul | ev::PitchOutcome::FoulTip | ev::PitchOutcome::InPlay | ev::PitchOutcome::HitByPitch));
                        classified[defense] += 1;
                    }
                    ev::EventBody::RunnerOut => { assert_eq!(defense, 1 - batting_side); pitcher_outs[defense] += 1; }
                    ev::EventBody::PlateAppearanceEnded => { assert_eq!(defense, 1 - batting_side); faced[defense] += 1; }
                    _ => {}
                }
            }
        }
        for defense in 0..2 {
            // I4: IP in outs equals putouts; I5: batters faced equals opponent PAs.
            let expected_outs = b.outs_by_half.iter().skip(1 - defense).step_by(2).sum::<u32>();
            assert_eq!(pitcher_outs[defense], expected_outs, "I4 seed {seed}");
            assert_eq!(faced[defense], b.teams[1 - defense].pa, "I5 seed {seed}");
            // I8: exactly one known outcome for each released pitch (HBP included).
            assert_eq!(pitches[defense], classified[defense], "I8 seed {seed}");
        }
    }
}
